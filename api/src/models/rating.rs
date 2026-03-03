// Model za ocjene i komentare recepata
// Korisnici mogu ocjeniti recepte od 1 do 5 i ostaviti komentar

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Osnovna ocjena iz baze
#[derive(Debug, FromRow, Serialize)]
pub struct Rating {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recipe_id: Uuid,
    pub score: i32,          // ocjena 1-5
    pub comment: Option<String>, // opcioni komentar
    pub created_at: NaiveDateTime,
}

/// Ocjena sa imenom korisnika - koristi se za prikaz na frontendu
/// JOIN sa users tabelom da dobijemo username
#[derive(Debug, FromRow, Serialize)]
pub struct RatingWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recipe_id: Uuid,
    pub score: i32,
    pub comment: Option<String>,
    pub created_at: NaiveDateTime,
    pub username: String, // iz JOIN-a sa users tabelom
}

/// Request body za kreiranje/ažuriranje ocjene
#[derive(Debug, Deserialize)]
pub struct CreateRatingRequest {
    pub score: i32,              // obavezno: 1-5
    pub comment: Option<String>, // opciono
}

/// Statistika ocjena za jedan recept
/// Koristi se za prikaz prosječne ocjene i broja glasova
#[derive(Debug, FromRow, Serialize)]
pub struct RecipeStats {
    pub average_rating: Option<f64>, // NULL ako nema ocjena
    pub total_ratings: i64,          // broj ocjena
}