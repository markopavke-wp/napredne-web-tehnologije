use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;


/// Sastojak sa nutritivnim vrijednostima na 100g
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: Uuid,
    pub name: String,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
}


#[derive(Debug, FromRow, Serialize)]
pub struct RecipeIngredient {
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit: String,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateIngredientRequest {
    pub name: String,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
}