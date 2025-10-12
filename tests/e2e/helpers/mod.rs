use anyhow::Result;
use axum::Router;
use chrono::{DateTime, Utc};
use feedtape_backend::infrastructure::config::{Config, Environment, LogFormat};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::sync::Arc;
use test_context::AsyncTestContext;
use testcontainers::{clients::Cli, Container};
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;
use uuid::Uuid;

pub mod api_client;
pub mod aws_mocks;
pub mod db_pool;
pub mod fixtures;

use api_client::TestClient;
use db_pool::{DatabasePool, PooledDatabase};
use fixtures::TestFixtures;

// Docker client for test containers
static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);

// Shared PostgreSQL container for all tests
static SHARED_CONTAINER: Lazy<SharedContainer> = Lazy::new(|| SharedContainer::new());

// Global database pool
static DB_POOL: Lazy<DatabasePool> = Lazy::new(|| DatabasePool::new(SHARED_CONTAINER.port));

/// Shared container that lives for the duration of all tests
struct SharedContainer {
    _container: Container<'static, Postgres>,
    port: u16,
}

impl SharedContainer {
    fn new() -> Self {
        let container = DOCKER.run(Postgres::default());
        let port = container.get_host_port_ipv4(5432);

        println!("🐳 Started shared PostgreSQL container on port {}", port);

        Self {
            _container: container,
            port,
        }
    }
}

pub struct TestContext {
    pub client: TestClient,
    #[allow(dead_code)]
    pub pool: PgPool,
    pub config: Config,
    pub fixtures: TestFixtures,
    _db: PooledDatabase,
}

impl AsyncTestContext for TestContext {
    fn setup() -> impl std::future::Future<Output = Self> + Send {
        async {
            // Get a database from the shared pool
            let pooled_db = DB_POOL
                .get_database()
                .await
                .expect("Failed to get database from pool");

            // Create test configuration
            let config = Config {
                database_url: pooled_db.database_url.clone(),
                host: "127.0.0.1".to_string(),
                port: 0, // Will be assigned by the OS
                jwt_secret: "test-jwt-secret-key-for-testing-only".to_string(),
                jwt_expiration_hours: 1,
                refresh_token_expiration_days: 30,
                aws_region: "us-east-1".to_string(),
                environment: Environment::Development,
                log_format: LogFormat::Pretty,
                github_client_id: "test_github_client_id".to_string(),
                github_client_secret: "test_github_client_secret".to_string(),
                github_redirect_uri: "http://localhost:8080/auth/callback/github".to_string(),
                tts_cache_enabled: false, // Disable cache in tests to avoid test pollution
            };

            // Create app with mocked AWS
            let app = create_app_with_mocked_aws(config.clone(), pooled_db.pool.clone())
                .await
                .expect("Failed to create app");

            // Start server
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("Failed to bind listener");
            let addr = listener.local_addr().expect("Failed to get local addr");
            let base_url = format!("http://{}", addr);

            tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });

            // Wait for server to be ready
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Create test client and fixtures
            let client = TestClient::new(&base_url);
            let fixtures = TestFixtures::new(pooled_db.pool.clone());

            Self {
                client,
                pool: pooled_db.pool.clone(),
                config,
                fixtures,
                _db: pooled_db,
            }
        }
    }

    fn teardown(self) -> impl std::future::Future<Output = ()> + Send {
        async {
            // Database cleanup happens automatically via Drop on PooledDatabase
        }
    }
}

async fn create_app_with_mocked_aws(config: Config, pool: PgPool) -> Result<Router> {
    use axum::{middleware, routing::get};
    use feedtape_backend::{
        controllers::{
            auth::AuthController, feed::FeedController,
            feed_suggestions::FeedSuggestionsController, health, oauth::OAuthController,
            tts::TtsController, user::UserController,
        },
        domain::{
            auth::AuthService, feed::FeedService, feed_suggestions::FeedSuggestionsService,
            tts::TtsService, user::UserService,
        },
        infrastructure::{
            auth::{auth_middleware, request_id_middleware},
            oauth::GitHubOAuthClient,
            repositories::{
                FeedRepository, HardcodedFeedSuggestionsRepository, RefreshTokenRepository,
                UsageRepository, UserRepository,
            },
        },
    };
    use tower_http::trace::TraceLayer;

    // Create mocked AWS Polly client
    let polly_client = aws_mocks::create_mock_polly_client().await;

    let pool = Arc::new(pool);
    let config = Arc::new(config);
    let polly_client = Arc::new(polly_client);

    // Instantiate repositories
    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let feed_repo = Arc::new(FeedRepository::new(pool.clone()));
    let feed_suggestions_repo = Arc::new(HardcodedFeedSuggestionsRepository::new());
    let refresh_token_repo = Arc::new(RefreshTokenRepository::new(pool.clone()));
    let usage_repo = Arc::new(UsageRepository::new(pool.clone()));

    // Instantiate OAuth clients
    let github_oauth_client = Arc::new(GitHubOAuthClient::new(
        config.github_client_id.clone(),
        config.github_client_secret.clone(),
        config.github_redirect_uri.clone(),
    ));

    // Instantiate services
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        refresh_token_repo.clone(),
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
        config.refresh_token_expiration_days,
    ));
    let feed_service = Arc::new(FeedService::new(feed_repo.clone(), user_repo.clone()));
    let user_service = Arc::new(UserService::new(user_repo.clone(), usage_repo.clone()));
    let tts_service = Arc::new(TtsService::new(
        user_repo.clone(),
        usage_repo.clone(),
        polly_client.clone(),
        false, // Disable cache in tests
    ));
    let feed_suggestions_service = Arc::new(FeedSuggestionsService::new(feed_suggestions_repo));

    // Instantiate controllers
    let auth_controller = Arc::new(AuthController::new(auth_service.clone()));
    let oauth_controller = Arc::new(OAuthController::new(
        github_oauth_client,
        user_repo.clone(),
        auth_service,
    ));
    let feed_controller = Arc::new(FeedController::new(feed_service));
    let user_controller = Arc::new(UserController::new(user_service.clone()));
    let tts_controller = Arc::new(TtsController::new(
        tts_service,
        user_service,
        usage_repo.clone(),
    ));
    let feed_suggestions_controller =
        Arc::new(FeedSuggestionsController::new(feed_suggestions_service));

    // TTS routes (need auth)
    let tts_routes = Router::new()
        .route(
            "/api/tts/synthesize",
            axum::routing::post(TtsController::synthesize),
        )
        .with_state(tts_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Usage route (needs auth)
    let usage_routes = Router::new()
        .route("/api/tts/usage", get(TtsController::get_usage))
        .with_state(tts_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Auth routes (public - no auth required)
    let auth_routes = Router::new()
        .route(
            "/auth/refresh",
            axum::routing::post(AuthController::refresh),
        )
        .route("/auth/logout", axum::routing::post(AuthController::logout))
        .with_state(auth_controller.clone());

    // OAuth routes (public - no auth required)
    let oauth_routes = Router::new()
        .route("/auth/oauth/github", get(OAuthController::initiate_github))
        .route(
            "/auth/callback/github",
            get(OAuthController::github_callback),
        )
        .with_state(oauth_controller.clone());

    // Logout all requires auth
    let auth_protected_routes = Router::new()
        .route(
            "/auth/logout/all",
            axum::routing::post(AuthController::logout_all),
        )
        .with_state(auth_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // User routes (require authentication)
    let user_routes = Router::new()
        .route(
            "/api/me",
            get(UserController::get_me).patch(UserController::update_me),
        )
        .with_state(user_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Feed routes (require authentication)
    let feed_routes = Router::new()
        .route(
            "/api/feeds",
            get(FeedController::list_feeds).post(FeedController::create_feed),
        )
        .route(
            "/api/feeds/:feedId",
            axum::routing::put(FeedController::update_feed).delete(FeedController::delete_feed),
        )
        .with_state(feed_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Feed suggestions routes (require authentication)
    let feed_suggestions_routes = Router::new()
        .route(
            "/api/feed-suggestions",
            get(FeedSuggestionsController::get_suggestions),
        )
        .with_state(feed_suggestions_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Build application routes
    let app = Router::new()
        .route("/health", get(health::health))
        .route("/health/ready", get(health::health_ready))
        .with_state(pool.clone())
        .merge(auth_routes)
        .merge(oauth_routes)
        .merge(auth_protected_routes)
        .merge(user_routes)
        .merge(feed_routes)
        .merge(feed_suggestions_routes)
        .merge(tts_routes)
        .merge(usage_routes)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

// Test user data for authentication
#[allow(dead_code)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub provider: String,
    pub provider_user_id: String,
    pub token: String,
    pub refresh_token: String,
}

#[allow(dead_code)]
impl TestUser {
    pub fn new(email: &str) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            email: email.to_string(),
            provider: "google".to_string(),
            provider_user_id: format!("provider_{}", id),
            token: String::new(),
            refresh_token: String::new(),
        }
    }

    pub fn with_provider(mut self, provider: &str) -> Self {
        self.provider = provider.to_string();
        self
    }
}

// Helper to generate valid JWT tokens for testing
pub fn generate_test_jwt(user_id: &Uuid, secret: &str) -> String {
    generate_test_jwt_with_email(user_id, "test@example.com", secret)
}

// Helper to generate valid JWT tokens for testing with specific email
pub fn generate_test_jwt_with_email(user_id: &Uuid, email: &str, secret: &str) -> String {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        email: String,
        exp: i64,
        iat: i64,
    }

    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: (now + chrono::Duration::hours(1)).timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

// Helper to create a timestamp
#[allow(dead_code)]
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

// Helper to assert API errors
#[allow(dead_code)]
pub fn assert_error_response(response: &serde_json::Value, expected_code: &str) {
    let error = response.get("error").expect("Missing error field");
    let code = error.get("code").expect("Missing error code");
    assert_eq!(code.as_str().unwrap(), expected_code);
    assert!(error.get("message").is_some(), "Missing error message");
    assert!(response.get("request_id").is_some(), "Missing request_id");
}
