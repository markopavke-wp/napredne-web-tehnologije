// Model za virtualni frižider korisnika
//
// Frižider je veza između korisnika i sastojaka koje ima kod kuće.
// Ne treba poseban struct za tabelu jer user_fridge ima samo
// foreign key-eve (user_id, ingredient_id) - nema vlastite podatke.
//
// Umjesto toga, koristimo:
// - Ingredient model za prikaz sadržaja frižidera
// - Request strukte za API pozive

use serde::Deserialize;
use uuid::Uuid;

/// Request za dodavanje ili uklanjanje jednog sastojka iz frižidera
/// Primjer: { "ingredient_id": "978ccc66-3d1a-4179-942b-02c6169437af" }
#[derive(Debug, Deserialize)]
pub struct FridgeRequest {
    pub ingredient_id: Uuid,
}

/// Request za filtriranje recepata iz frižidera po kalorijama
/// Oba polja su opciona - korisnik može koristiti jedno, oba ili nijedno
///
/// Primjeri:
/// { "max_calories": 500 }                      - recepti ispod 500 kcal
/// { "min_calories": 200, "max_calories": 600 } - recepti između 200 i 600 kcal
/// {}                                            - svi recepti bez filtera
#[derive(Debug, Deserialize)]
pub struct FridgeFilterRequest {
    pub max_calories: Option<f64>,
    pub min_calories: Option<f64>,
}