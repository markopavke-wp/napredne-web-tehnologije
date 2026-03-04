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
use crate::models::ingredient::RecipeIngredient;
use crate::models::recipe::{CreateRecipeRequest, Recipe, RecipeWithAuthor, SearchByIngredientsRequest};
use crate::AppState;

pub async fn list_recipes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RecipeWithAuthor>>, AppError> {
    let recipes = sqlx::query_as::<_, RecipeWithAuthor>(
        r#"
        SELECT r.*, u.username
        FROM recipes r
        JOIN users u ON r.user_id = u.id
        ORDER BY r.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(recipes))
}

pub async fn get_recipe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<RecipeWithAuthor>, AppError> {
    let recipe = sqlx::query_as::<_, RecipeWithAuthor>(
        r#"
        SELECT r.*, u.username
        FROM recipes r
        JOIN users u ON r.user_id = u.id
        WHERE r.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(recipe))
}

pub async fn create_recipe(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateRecipeRequest>,
) -> Result<(StatusCode, Json<Recipe>), AppError> {
    let recipe_id = Uuid::new_v4();

    let recipe = sqlx::query_as::<_, Recipe>(
        r#"
        INSERT INTO recipes (id, user_id, title, description, instructions,
            prep_time_min, cook_time_min, servings, image_url, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
        "#,
    )
    .bind(recipe_id)
    .bind(auth_user.user_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.instructions)
    .bind(body.prep_time_min)
    .bind(body.cook_time_min)
    .bind(body.servings)
    .bind(&body.image_url)
    .fetch_one(&state.db)
    .await?;

    for ing in &body.ingredients {
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, unit) VALUES ($1, $2, $3, $4)",
        )
        .bind(recipe_id)
        .bind(ing.ingredient_id)
        .bind(ing.quantity)
        .bind(&ing.unit)
        .execute(&state.db)
        .await?;
    }

    Ok((StatusCode::CREATED, Json(recipe)))
}

pub async fn update_recipe(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateRecipeRequest>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = sqlx::query_as::<_, Recipe>(
        r#"
        UPDATE recipes SET title = $1, description = $2, instructions = $3,
            prep_time_min = $4, cook_time_min = $5, servings = $6,
            image_url = $7, updated_at = NOW()
        WHERE id = $8 AND user_id = $9
        RETURNING *
        "#,
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.instructions)
    .bind(body.prep_time_min)
    .bind(body.cook_time_min)
    .bind(body.servings)
    .bind(&body.image_url)
    .bind(id)
    .bind(auth_user.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(recipe))
}

pub async fn delete_recipe(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM recipes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_by_ingredients(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SearchByIngredientsRequest>,
) -> Result<Json<Vec<RecipeWithAuthor>>, AppError> {
    if body.ingredient_ids.is_empty() {
        return Err(AppError::BadRequest("At least one ingredient required".into()));
    }

    let recipes = sqlx::query_as::<_, RecipeWithAuthor>(
        r#"
        SELECT r.*, u.username
        FROM recipes r
        JOIN users u ON r.user_id = u.id
        INNER JOIN recipe_ingredients ri ON r.id = ri.recipe_id
        WHERE ri.ingredient_id = ANY($1)
        GROUP BY r.id, u.username
        ORDER BY COUNT(DISTINCT ri.ingredient_id) DESC
        LIMIT 50
        "#,
    )
    .bind(&body.ingredient_ids)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(recipes))
}

pub async fn get_recipe_ingredients(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<RecipeIngredient>>, AppError> {
    let ingredients = sqlx::query_as::<_, RecipeIngredient>(
        r#"
        SELECT ri.recipe_id, ri.ingredient_id, i.name as ingredient_name,
               ri.quantity, ri.unit,
               i.calories_per_100g, i.protein_per_100g, i.carbs_per_100g,
               i.fat_per_100g, i.fiber_per_100g
        FROM recipe_ingredients ri
        JOIN ingredients i ON i.id = ri.ingredient_id
        WHERE ri.recipe_id = $1
        ORDER BY i.name
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(ingredients))
}