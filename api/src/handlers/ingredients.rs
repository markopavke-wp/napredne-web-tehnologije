use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::ingredient::{CreateIngredientRequest, Ingredient};
use crate::AppState;

pub async fn list_ingredients(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Ingredient>>, AppError> {
    let ingredients = sqlx::query_as::<_, Ingredient>(
        "SELECT * FROM ingredients ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ingredients))
}

pub async fn create_ingredient(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateIngredientRequest>,
) -> Result<(StatusCode, Json<Ingredient>), AppError> {
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
    .await
    .map_err(|_| AppError::Conflict("Ingredient already exists".into()))?;

    Ok((StatusCode::CREATED, Json(ingredient)))
}