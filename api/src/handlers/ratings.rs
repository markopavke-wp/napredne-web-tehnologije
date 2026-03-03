// Handler za ocjene i komentare recepata
//
// Endpointi:
// POST   /api/recipes/:id/rate    - ocijeni recept (1-5) + opcioni komentar
// GET    /api/recipes/:id/ratings - sve ocjene za recept (javno)
// GET    /api/recipes/:id/stats   - prosječna ocjena + broj glasova (javno)
// DELETE /api/recipes/:id/rate    - obriši svoju ocjenu

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::handlers::middleware::AuthUser;
use crate::models::rating::{CreateRatingRequest, RatingWithUser, RecipeStats};
use crate::AppState;

/// Ocijeni recept ili ažuriraj postojeću ocjenu
/// Ako korisnik već ima ocjenu za taj recept, ažurira je (ON CONFLICT DO UPDATE)
/// Zahtijeva: JWT token
pub async fn rate_recipe(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(recipe_id): Path<Uuid>,
    Json(body): Json<CreateRatingRequest>,
) -> Result<(StatusCode, Json<RatingWithUser>), AppError> {
    // Validacija: ocjena mora biti 1-5
    if body.score < 1 || body.score > 5 {
        return Err(AppError::BadRequest("Score must be between 1 and 5".into()));
    }

    // INSERT ili UPDATE ako već postoji ocjena od ovog korisnika
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

    Ok((StatusCode::CREATED, Json(rating)))
}

/// Dohvati sve ocjene za jedan recept
/// Javni endpoint - ne treba token
/// Vraća ocjene sortirane od najnovije
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

/// Dohvati statistiku ocjena za recept (prosječna ocjena + broj glasova)
/// Javni endpoint - ne treba token
/// Primjer odgovora: { "average_rating": 4.5, "total_ratings": 12 }
pub async fn get_recipe_stats(
    State(state): State<Arc<AppState>>,
    Path(recipe_id): Path<Uuid>,
) -> Result<Json<RecipeStats>, AppError> {
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

    Ok(Json(stats))
}

/// Obriši svoju ocjenu za recept
/// Korisnik može obrisati samo SVOJU ocjenu (WHERE user_id = auth_user)
/// Zahtijeva: JWT token
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

    Ok(StatusCode::NO_CONTENT)
}