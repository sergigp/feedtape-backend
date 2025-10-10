use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use feedtape_backend::infrastructure::config::{Config, LogFormat};
use feedtape_backend::infrastructure::db::{check_connection, create_pool};
use feedtape_backend::infrastructure::http::start_http_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize logging
    init_logging(&config);

    tracing::info!(
        "Starting FeedTape Backend on {}:{}",
        config.host,
        config.port
    );

    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Verify database connection
    check_connection(&pool).await?;
    tracing::info!("Database connection verified");

    // Create AWS Polly client
    tracing::info!("Initializing AWS Polly client with region: {}", config.aws_region);

    // Check for AWS credentials in environment (for debugging)
    let has_access_key = std::env::var("AWS_ACCESS_KEY_ID").is_ok();
    let has_secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").is_ok();
    tracing::info!(
        has_access_key_id = has_access_key,
        has_secret_access_key = has_secret_key,
        "AWS credentials environment check"
    );

    if !has_access_key || !has_secret_key {
        tracing::warn!("AWS credentials not found in environment variables. Will attempt to use other credential providers (instance metadata, etc.)");
    }

    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(config.aws_region.clone()))
        .load()
        .await;

    // Log AWS config details (without exposing credentials)
    tracing::info!(
        region = ?aws_config.region(),
        "AWS configuration loaded"
    );

    let polly_client = aws_sdk_polly::Client::new(&aws_config);
    tracing::info!("AWS Polly client initialized successfully");

    let pool = Arc::new(pool);
    let config = Arc::new(config);
    let polly_client = Arc::new(polly_client);

    // === DEPENDENCY INJECTION SETUP ===
    // 1. Instantiate repositories (inject db pool)
    tracing::info!("Instantiating repositories...");
    let user_repo = Arc::new(feedtape_backend::infrastructure::repositories::UserRepository::new(pool.clone()));
    let feed_repo = Arc::new(feedtape_backend::infrastructure::repositories::FeedRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(feedtape_backend::infrastructure::repositories::RefreshTokenRepository::new(pool.clone()));
    let usage_repo = Arc::new(feedtape_backend::infrastructure::repositories::UsageRepository::new(pool.clone()));

    // 2. Instantiate OAuth clients
    tracing::info!("Instantiating OAuth clients...");
    let github_oauth_client = Arc::new(feedtape_backend::infrastructure::oauth::GitHubOAuthClient::new(
        config.github_client_id.clone(),
        config.github_client_secret.clone(),
        config.github_redirect_uri.clone(),
    ));

    // 3. Instantiate services (inject repositories and clients)
    tracing::info!("Instantiating services...");
    let auth_service = Arc::new(feedtape_backend::domain::auth::AuthService::new(
        user_repo.clone(),
        refresh_token_repo.clone(),
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
        config.refresh_token_expiration_days,
    ));
    let feed_service = Arc::new(feedtape_backend::domain::feed::FeedService::new(
        feed_repo.clone(),
        user_repo.clone(),
    ));
    let user_service = Arc::new(feedtape_backend::domain::user::UserService::new(
        user_repo.clone(),
        usage_repo.clone(),
    ));
    let tts_service = Arc::new(feedtape_backend::domain::tts::TtsService::new(
        user_repo.clone(),
        usage_repo.clone(),
        polly_client.clone(),
        config.tts_cache_enabled,
    ));

    // 4. Instantiate controllers (inject services)
    tracing::info!("Instantiating controllers...");
    let auth_controller = Arc::new(feedtape_backend::controllers::auth::AuthController::new(auth_service.clone()));
    let oauth_controller = Arc::new(feedtape_backend::controllers::oauth::OAuthController::new(
        github_oauth_client,
        user_repo.clone(),
        auth_service,
    ));
    let feed_controller = Arc::new(feedtape_backend::controllers::feed::FeedController::new(feed_service));
    let user_controller = Arc::new(feedtape_backend::controllers::user::UserController::new(user_service.clone()));
    let tts_controller = Arc::new(feedtape_backend::controllers::tts::TtsController::new(
        tts_service,
        user_service,
        usage_repo.clone(),
    ));

    // Start HTTP server with all routes
    start_http_server(pool, config, user_repo, auth_controller, oauth_controller, feed_controller, user_controller, tts_controller).await?;

    Ok(())
}

fn init_logging(config: &Config) {
    if config.log_format == LogFormat::Json {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "feedtape_backend=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "feedtape_backend=debug,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}
