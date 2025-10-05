use axum::{middleware, routing::get, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::infrastructure::config::Config;
use crate::infrastructure::db::DbPool;
use crate::{
    controllers::{auth::AuthController, feed::FeedController, health, oauth::OAuthController, tts::TtsController, user::UserController},
    infrastructure::auth::{auth_middleware, request_id_middleware},
};

use crate::infrastructure::repositories::UserRepository;

/// Start the HTTP server with all routes configured
pub async fn start_http_server(
    pool: Arc<DbPool>,
    config: Arc<Config>,
    user_repo: Arc<UserRepository>,
    auth_controller: Arc<AuthController>,
    oauth_controller: Arc<OAuthController>,
    feed_controller: Arc<FeedController>,
    user_controller: Arc<UserController>,
    tts_controller: Arc<TtsController>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TTS routes (need auth)
    let tts_routes = Router::new()
        .route("/api/tts/synthesize", axum::routing::post(TtsController::synthesize))
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
        .route("/auth/refresh", axum::routing::post(AuthController::refresh))
        .route("/auth/logout", axum::routing::post(AuthController::logout))
        .with_state(auth_controller.clone());

    // OAuth routes (public - no auth required)
    let oauth_routes = Router::new()
        .route("/auth/oauth/github", get(OAuthController::initiate_github))
        .route("/auth/callback/github", get(OAuthController::github_callback))
        .with_state(oauth_controller.clone());

    // Logout all requires auth
    let auth_protected_routes = Router::new()
        .route("/auth/logout/all", axum::routing::post(AuthController::logout_all))
        .with_state(auth_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // User routes (require authentication)
    let user_routes = Router::new()
        .route("/api/me", get(UserController::get_me).patch(UserController::update_me))
        .with_state(user_controller.clone())
        .layer(middleware::from_fn_with_state(
            (user_repo.clone(), config.clone()),
            auth_middleware,
        ));

    // Feed routes (require authentication)
    let feed_routes = Router::new()
        .route("/api/feeds", get(FeedController::list_feeds).post(FeedController::create_feed))
        .route(
            "/api/feeds/:feedId",
            axum::routing::put(FeedController::update_feed).delete(FeedController::delete_feed),
        )
        .with_state(feed_controller.clone())
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
        .merge(tts_routes)
        .merge(usage_routes)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
