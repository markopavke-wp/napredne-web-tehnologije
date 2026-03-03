use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod config;
mod errors;
mod handlers;
mod models;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/myapp".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("Connected to database");

    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Failed to load migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied successfully");

    let state = Arc::new(AppState {
        db: pool,
        jwt_secret,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/recipes", get(handlers::recipes::list_recipes))
        .route("/api/recipes", post(handlers::recipes::create_recipe))
        .route("/api/recipes/:id", get(handlers::recipes::get_recipe))
        .route("/api/recipes/:id", put(handlers::recipes::update_recipe))
        .route("/api/recipes/:id", delete(handlers::recipes::delete_recipe))
        .route("/api/recipes/search", post(handlers::recipes::search_by_ingredients))
        .route("/api/ingredients", get(handlers::ingredients::list_ingredients))
        .route("/api/ingredients", post(handlers::ingredients::create_ingredient))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}