// Glavni entry point za API server
//
// Arhitektura:
// - Axum web framework za HTTP handling
// - SQLx za async pristup PostgreSQL bazi
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
    pub db: sqlx::PgPool,      // connection pool prema bazi
    pub jwt_secret: String,     // tajni ključ za potpisivanje JWT tokena
}

#[tokio::main]
async fn main() {
    // Inicijaliziraj logging (tracing)
    tracing_subscriber::fmt::init();

    // Čitaj konfiguraciju iz environment varijabli
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/myapp".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-key".to_string());

    // Spoji se na PostgreSQL bazu sa pool-om od max 10 konekcija
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("Connected to database");

    // Pokreni SQL migracije automatski pri startu servera
    // Ovo kreira/ažurira tabele u bazi prema .sql fajlovima u migrations/
    sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Failed to load migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied successfully");

    // Kreiraj dijeljeno stanje aplikacije
    let state = Arc::new(AppState {
        db: pool,
        jwt_secret,
    });

    // CORS - dozvoli pristup sa bilo kojeg origina (za development)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ═══════════════════════════════════════════════════════════
    // ZAŠTIĆENE RUTE - zahtijevaju JWT token u Authorization headeru
    // Header format: "Authorization: Bearer <token>"
    // ═══════════════════════════════════════════════════════════
    let protected_routes = Router::new()
        // Recepti - kreiranje, izmjena, brisanje (samo vlasnik)
        .route("/api/recipes", post(handlers::recipes::create_recipe))
        .route("/api/recipes/:id", put(handlers::recipes::update_recipe))
        .route("/api/recipes/:id", delete(handlers::recipes::delete_recipe))
        // Sastojci - dodavanje (samo admin)
        .route("/api/ingredients", post(handlers::ingredients::create_ingredient))
        // Ocjene - ocijeni recept ili obriši svoju ocjenu
        .route("/api/recipes/:id/rate", post(handlers::ratings::rate_recipe))
        .route("/api/recipes/:id/rate", delete(handlers::ratings::delete_rating))
        // Frižider - upravljaj svojim sastojcima i filtriraj recepte
        .route("/api/fridge", get(handlers::fridge::get_my_fridge))
        .route("/api/fridge", post(handlers::fridge::add_to_fridge))
        .route("/api/fridge/remove", post(handlers::fridge::remove_from_fridge))
        .route("/api/fridge/recipes", post(handlers::fridge::recipes_from_fridge))
        // Middleware se primjenjuje na SVE rute iznad
        .layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::middleware::auth_middleware,
        ));

    // ═══════════════════════════════════════════════════════════
    // JAVNE RUTE - ne zahtijevaju autentifikaciju
    // ═══════════════════════════════════════════════════════════
    let public_routes = Router::new()
        // Health check - provjera da li server radi
        .route("/api/health", get(|| async { "OK" }))
        // Autentifikacija - registracija i login
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        // Recepti - čitanje (svi mogu gledati)
        .route("/api/recipes", get(handlers::recipes::list_recipes))
        .route("/api/recipes/:id", get(handlers::recipes::get_recipe))
        .route("/api/recipes/search", post(handlers::recipes::search_by_ingredients))
        // Ocjene - čitanje (svi mogu vidjeti ocjene)
        .route("/api/recipes/:id/ratings", get(handlers::ratings::get_recipe_ratings))
        .route("/api/recipes/:id/stats", get(handlers::ratings::get_recipe_stats))
        // Sastojci - čitanje (svi mogu vidjeti listu)
        .route("/api/ingredients", get(handlers::ingredients::list_ingredients));

    // Spoji javne i zaštićene rute u jednu aplikaciju
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