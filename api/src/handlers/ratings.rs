// Handler za ocjene i komentare recepata
//
// Endpointi:
// POST   /api/recipes/:id/rate    - ocijeni recept (BRIŠE CACHE statistike)
// GET    /api/recipes/:id/ratings - sve ocjene za recept
// GET    /api/recipes/:id/stats   - statistika ocjena (KEŠIRANO)
// DELETE /api/recipes/:id/rate    - obriši svoju ocjenu (BRIŠE CACHE)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use redis::AsyncCommands;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::handlers::middleware::AuthUser;
use crate::models::rating::{CreateRatingRequest, RatingWithUser, RecipeStats};
use crate::AppState;

/// Cache TTL za statistike - 2 minute
const STATS_CACHE_TTL: u64 = 120;

/// Ocijeni recept ili ažuriraj postojeću ocjenu
/// Briše cache statistike za taj recept
pub async fn rate_recipe(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(recipe_id): Path<Uuid>,
    Json(body): Json<CreateRatingRequest>,
) -> Result<(StatusCode, Json<RatingWithUser>), AppError> {
    if body.score < 1 || body.score > 5 {
        return Err(AppError::BadRequest("Score must be between 1 and 5".into()));
    }

    let rating = sqlx::query_as::<_, RatingWithUser>(
        r#"
        INSERT INTO ratings (id, user_id, recipe_id, score, comment, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (user_id, recipe_id) 
        DO UPDATE SET score = $4, comment = $5
        RETURNING ratings.*, (SELECT username FROM users WHERE id = $2) as username
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(auth_user.user_id)
    .bind(recipe_id)
    .bind(body.score)
    .bind(&body.comment)
    .fetch_one(&state.db)
    .await?;

    // Obriši cache statistike za ovaj recept jer se ocjena promijenila
    let mut redis_conn = state.redis.clone();
    let cache_key = format!("cache:recipe:{}:stats", recipe_id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;
    tracing::info!("Cache INVALIDATED for recipe {} stats", recipe_id);

    Ok((StatusCode::CREATED, Json(rating)))
}

/// Dohvati sve ocjene za recept (nije keširano jer se rijetko čita masovno)
pub async fn get_recipe_ratings(
    State(state): State<Arc<AppState>>,
    Path(recipe_id): Path<Uuid>,
) -> Result<Json<Vec<RatingWithUser>>, AppError> {
    let ratings = sqlx::query_as::<_, RatingWithUser>(
        r#"
        SELECT r.*, u.username
        FROM ratings r
        JOIN users u ON r.user_id = u.id
        WHERE r.recipe_id = $1
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(recipe_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ratings))
}

/// Statistika ocjena - KEŠIRANO u Redisu
///
/// Flow:
/// 1. Provjeri cache → ako postoji, vrati odmah
/// 2. Ako nema → dohvati iz baze, spremi u cache (2 min TTL)
/// 3. Cache se briše kad neko ocijeni ili obriše ocjenu
pub async fn get_recipe_stats(
    State(state): State<Arc<AppState>>,
    Path(recipe_id): Path<Uuid>,
) -> Result<Json<RecipeStats>, AppError> {
    let cache_key = format!("cache:recipe:{}:stats", recipe_id);
    let mut redis_conn = state.redis.clone();

    // 1. Pokušaj iz cache-a
    let cached: Result<String, _> = redis_conn.get(&cache_key).await;
    if let Ok(cached_json) = cached {
        tracing::info!("Cache HIT for recipe {} stats", recipe_id);
        let stats: RecipeStats = serde_json::from_str(&cached_json)
            .map_err(|_| AppError::Internal("Cache parse error".into()))?;
        return Ok(Json(stats));
    }

    // 2. Cache MISS - dohvati iz baze
    tracing::info!("Cache MISS for recipe {} stats", recipe_id);
    let stats = sqlx::query_as::<_, RecipeStats>(
        r#"
        SELECT 
            AVG(score)::DOUBLE PRECISION as average_rating,
            COUNT(*) as total_ratings
        FROM ratings
        WHERE recipe_id = $1
        "#,
    )
    .bind(recipe_id)
    .fetch_one(&state.db)
    .await?;

    // 3. Spremi u cache
    let json = serde_json::to_string(&stats)
        .map_err(|_| AppError::Internal("Serialization error".into()))?;
    let _: Result<(), _> = redis_conn.set_ex(&cache_key, &json, STATS_CACHE_TTL).await;

    Ok(Json(stats))
}

/// Obriši svoju ocjenu - briše cache statistike
pub async fn delete_rating(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(recipe_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM ratings WHERE recipe_id = $1 AND user_id = $2")
        .bind(recipe_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    // Obriši cache statistike
    let mut redis_conn = state.redis.clone();
    let cache_key = format!("cache:recipe:{}:stats", recipe_id);
    let _: Result<(), _> = redis_conn.del(&cache_key).await;

    Ok(StatusCode::NO_CONTENT)
}