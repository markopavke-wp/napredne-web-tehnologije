// Glavni entry point za API server
//
// Arhitektura:
// - Axum web framework za HTTP handling
// - SQLx za async pristup PostgreSQL bazi
// - Redis za keširanje čestih upita
// - JWT tokeni za autentifikaciju
// - Bcrypt za hashiranje lozinki
//
// Rute su podijeljene na:
// - Javne (public_routes) - ne trebaju autentifikaciju
// - Zaštićene (protected_routes) - zahtijevaju JWT token u Authorization headeru

use axum::{
    middleware,
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

/// Globalno stanje aplikacije - dijeli se između svih handlera
/// Arc<AppState> omogućava thread-safe pristup
pub struct AppState {
    pub db: sqlx::PgPool,                                    // connection pool prema bazi
    pub redis: redis::aio::ConnectionManager,                // Redis connection manager
    pub jwt_secret: String,                                   // tajni ključ za JWT tokene
}

#[tokio::main]
async fn main() {
    // Inicijaliziraj logging (tracing)
    tracing_subscriber::fmt::init();

    // ── Konfiguracija iz environment varijabli ──
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/myapp".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // ── Spoji se na PostgreSQL ──
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    tracing::info!("Connected to PostgreSQL");

    // ── Spoji se na Redis ──
    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
    let redis_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");
    tracing::info!("Connected to Redis");

    // ── Pokreni migracije ──
    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Failed to load migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    tracing::info!("Migrations applied successfully");

    // ── Kreiraj dijeljeno stanje ──
    let state = Arc::new(AppState {
        db: pool,
        redis: redis_manager,
        jwt_secret,
    });

    // CORS - dozvoli pristup sa bilo kojeg origina (za development)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ═══════════════════════════════════════════════════════════
    // ZAŠTIĆENE RUTE - zahtijevaju JWT token
    // ═══════════════════════════════════════════════════════════
    let protected_routes = Router::new()
        // Recepti - kreiranje, izmjena, brisanje
        .route("/api/recipes", post(handlers::recipes::create_recipe))
        .route("/api/recipes/:id", put(handlers::recipes::update_recipe))
        .route("/api/recipes/:id", delete(handlers::recipes::delete_recipe))
        // Sastojci - dodavanje (samo admin)
        .route("/api/ingredients", post(handlers::ingredients::create_ingredient))
        // Ocjene
        .route("/api/recipes/:id/rate", post(handlers::ratings::rate_recipe))
        .route("/api/recipes/:id/rate", delete(handlers::ratings::delete_rating))
        // Frižider
        .route("/api/fridge", get(handlers::fridge::get_my_fridge))
        .route("/api/fridge", post(handlers::fridge::add_to_fridge))
        .route("/api/fridge/remove", post(handlers::fridge::remove_from_fridge))
        .route("/api/fridge/recipes", post(handlers::fridge::recipes_from_fridge))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::middleware::auth_middleware,
        ));

    // ═══════════════════════════════════════════════════════════
    // JAVNE RUTE - ne zahtijevaju autentifikaciju
    // ═══════════════════════════════════════════════════════════
    let public_routes = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/recipes", get(handlers::recipes::list_recipes))
        .route("/api/recipes/:id", get(handlers::recipes::get_recipe))
        .route("/api/recipes/search", post(handlers::recipes::search_by_ingredients))
        .route("/api/recipes/:id/ratings", get(handlers::ratings::get_recipe_ratings))
        .route("/api/recipes/:id/stats", get(handlers::ratings::get_recipe_stats))
        .route("/api/ingredients", get(handlers::ingredients::list_ingredients));

    // Spoji javne i zaštićene rute
    let app = public_routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(state);

    // Pokreni server na portu 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    tracing::info!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}