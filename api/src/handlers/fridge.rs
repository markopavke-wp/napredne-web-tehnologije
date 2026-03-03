// Handler za virtualni frižider korisnika
//
// Koncept: Korisnik dodaje sastojke koje ima kod kuće u svoj "frižider".
// Onda može filtrirati recepte koje može napraviti sa tim sastojcima,
// i opciono filtrirati po kalorijama.
//
// Endpointi:
// GET  /api/fridge          - lista mojih sastojaka u frižideru
// POST /api/fridge          - dodaj sastojak u frižider
// POST /api/fridge/remove   - ukloni sastojak iz frižidera
// POST /api/fridge/recipes  - pronađi recepte koje mogu napraviti + filter po kalorijama

use axum::{
    extract::State,
    http::StatusCode,
    Extension,
    Json,
};
use std::sync::Arc;

use crate::errors::AppError;
use crate::handlers::middleware::AuthUser;
use crate::models::fridge::{FridgeFilterRequest, FridgeRequest};
use crate::models::ingredient::Ingredient;
use crate::models::recipe::RecipeWithAuthor;
use crate::AppState;

/// Dohvati sve sastojke iz mog frižidera
/// Zahtijeva: JWT token
/// Vraća: listu sastojaka sa nutritivnim vrijednostima
pub async fn get_my_fridge(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<Ingredient>>, AppError> {
    let ingredients = sqlx::query_as::<_, Ingredient>(
        r#"
        SELECT i.*
        FROM ingredients i
        JOIN user_fridge uf ON i.id = uf.ingredient_id
        WHERE uf.user_id = $1
        ORDER BY i.name
        "#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ingredients))
}

/// Dodaj sastojak u moj frižider
/// ON CONFLICT DO NOTHING - ako već postoji, ne radi ništa (nema greške)
/// Zahtijeva: JWT token
pub async fn add_to_fridge(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<FridgeRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query(
        "INSERT INTO user_fridge (user_id, ingredient_id, added_at) VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind(auth_user.user_id)
    .bind(body.ingredient_id)
    .execute(&state.db)
    .await?;

    Ok(StatusCode::CREATED)
}

/// Ukloni sastojak iz mog frižidera
/// Zahtijeva: JWT token
pub async fn remove_from_fridge(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<FridgeRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM user_fridge WHERE user_id = $1 AND ingredient_id = $2")
        .bind(auth_user.user_id)
        .bind(body.ingredient_id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Pronađi recepte koje mogu napraviti sa sastojcima iz mog frižidera
///
/// Logika:
/// 1. Uzmi sve sastojke iz mog frižidera
/// 2. Nađi recepte čiji su SVI sastojci u mom frižideru
///    (HAVING COUNT = ukupan broj sastojaka u receptu)
/// 3. Opciono filtriraj po kalorijama:
///    - max_calories: recepti sa ukupno MANJE kalorija
///    - min_calories: recepti sa ukupno VIŠE kalorija
///
/// Primjer: Imam jaja, brašno, mlijeko → dobijem recepte za palačinke
///          Ali NE dobijem recept za tortu jer nemam čokoladu
///
/// Zahtijeva: JWT token
pub async fn recipes_from_fridge(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<FridgeFilterRequest>,
) -> Result<Json<Vec<RecipeWithAuthor>>, AppError> {
    let recipes = sqlx::query_as::<_, RecipeWithAuthor>(
        r#"
        SELECT r.*, u.username
        FROM recipes r
        JOIN users u ON r.user_id = u.id
        WHERE r.id IN (
            -- Nađi recepte gdje su SVI sastojci u mom frižideru
            SELECT ri.recipe_id
            FROM recipe_ingredients ri
            JOIN user_fridge uf ON ri.ingredient_id = uf.ingredient_id AND uf.user_id = $1
            GROUP BY ri.recipe_id
            HAVING COUNT(DISTINCT ri.ingredient_id) = (
                -- Ukupan broj sastojaka u receptu mora biti jednak
                -- broju poklopljenih sastojaka
                SELECT COUNT(DISTINCT ri2.ingredient_id)
                FROM recipe_ingredients ri2
                WHERE ri2.recipe_id = ri.recipe_id
            )
        )
        -- Filter po maksimalnim kalorijama (opciono)
        AND ($2::DOUBLE PRECISION IS NULL OR (
            SELECT SUM(i.calories_per_100g * ri3.quantity / 100.0)
            FROM recipe_ingredients ri3
            JOIN ingredients i ON ri3.ingredient_id = i.id
            WHERE ri3.recipe_id = r.id
        ) <= $2)
        -- Filter po minimalnim kalorijama (opciono)
        AND ($3::DOUBLE PRECISION IS NULL OR (
            SELECT SUM(i.calories_per_100g * ri4.quantity / 100.0)
            FROM recipe_ingredients ri4
            JOIN ingredients i ON ri4.ingredient_id = i.id
            WHERE ri4.recipe_id = r.id
        ) >= $3)
        ORDER BY r.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(auth_user.user_id)
    .bind(body.max_calories)
    .bind(body.min_calories)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(recipes))
}