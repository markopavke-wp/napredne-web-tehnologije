// Handler za sastojke (ingredients)
//
// Endpointi:
// GET  /api/ingredients - lista svih sastojaka (javno, KEŠIRANO U REDIS-u)
// POST /api/ingredients - dodaj novi sastojak (samo admin, BRIŠE CACHE)

use axum::{
    extract::State,
    http::StatusCode,
    Extension,
    Json,
};
use redis::AsyncCommands;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::handlers::middleware::AuthUser;
use crate::models::ingredient::{CreateIngredientRequest, Ingredient};
use crate::AppState;

/// Ključ pod kojim čuvamo listu sastojaka u Redisu
const INGREDIENTS_CACHE_KEY: &str = "cache:ingredients:all";
/// Koliko sekundi cache traje (5 minuta)
const CACHE_TTL_SECONDS: u64 = 300;

/// Lista svih sastojaka - KEŠIRANO
///
/// Flow:
/// 1. Provjeri da li postoji u Redis cache-u
/// 2. Ako DA → vrati iz cache-a (brzo, ne ide na bazu)
/// 3. Ako NE → dohvati iz baze, spremi u cache, vrati
///
/// Cache se automatski briše kad se doda novi sastojak
pub async fn list_ingredients(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Ingredient>>, AppError> {
    // 1. Pokušaj dohvatiti iz Redis cache-a
    let mut redis_conn = state.redis.clone();
    let cached: Result<String, _> = redis_conn.get(INGREDIENTS_CACHE_KEY).await;

    if let Ok(cached_json) = cached {
        // Cache HIT - vrati podatke iz Redisa bez da idemo na bazu
        tracing::info!("Cache HIT for ingredients");
        let ingredients: Vec<Ingredient> = serde_json::from_str(&cached_json)
            .map_err(|_| AppError::Internal("Cache parse error".into()))?;
        return Ok(Json(ingredients));
    }

    // 2. Cache MISS - dohvati iz baze
    tracing::info!("Cache MISS for ingredients - fetching from DB");
    let ingredients = sqlx::query_as::<_, Ingredient>(
        "SELECT * FROM ingredients ORDER BY name"
    )
    .fetch_all(&state.db)
    .await?;

    // 3. Spremi u Redis cache sa TTL-om (istječe za 5 minuta)
    let json = serde_json::to_string(&ingredients)
        .map_err(|_| AppError::Internal("Serialization error".into()))?;
    let _: Result<(), _> = redis_conn
        .set_ex(INGREDIENTS_CACHE_KEY, &json, CACHE_TTL_SECONDS)
        .await;

    Ok(Json(ingredients))
}

/// Dodaj novi sastojak - samo admin
/// Kad se doda novi sastojak, BRIŠE SE CACHE da lista bude ažurna
pub async fn create_ingredient(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateIngredientRequest>,
) -> Result<(StatusCode, Json<Ingredient>), AppError> {
    // Samo admin može dodavati sastojke
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Provjeri da li već postoji
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ingredients WHERE LOWER(name) = LOWER($1)"
    )
    .bind(&body.name)
    .fetch_one(&state.db)
    .await?;

    if existing > 0 {
        return Err(AppError::BadRequest("Ingredient already exists".into()));
    }

    // Kreiraj novi sastojak
    let ingredient = sqlx::query_as::<_, Ingredient>(
        r#"
        INSERT INTO ingredients (id, name, calories_per_100g, protein_per_100g, carbs_per_100g, fat_per_100g, fiber_per_100g)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&body.name)
    .bind(body.calories_per_100g)
    .bind(body.protein_per_100g)
    .bind(body.carbs_per_100g)
    .bind(body.fat_per_100g)
    .bind(body.fiber_per_100g)
    .fetch_one(&state.db)
    .await?;

    // Obriši cache jer se lista promijenila
    let mut redis_conn = state.redis.clone();
    let _: Result<(), _> = redis_conn.del(INGREDIENTS_CACHE_KEY).await;
    tracing::info!("Cache INVALIDATED for ingredients");

    Ok((StatusCode::CREATED, Json(ingredient)))
}