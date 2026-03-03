use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct Recipe {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub instructions: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub servings: Option<i32>,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipeRequest {
    pub title: String,
    pub description: Option<String>,
    pub instructions: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub servings: Option<i32>,
    pub image_url: Option<String>,
    pub ingredients: Vec<RecipeIngredientInput>,
}

#[derive(Debug, Deserialize)]
pub struct RecipeIngredientInput {
    pub ingredient_id: Uuid,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchByIngredientsRequest {
    pub ingredient_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecipeWithAuthor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub title: String,
    pub description: Option<String>,
    pub instructions: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub servings: Option<i32>,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}