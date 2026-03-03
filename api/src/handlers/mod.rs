// Svi handleri (kontroleri) za API endpointe
// Svaki modul odgovara jednoj grupi ruta

pub mod auth;        // registracija i login (/api/auth/*)
pub mod fridge;      // virtualni frižider korisnika (/api/fridge/*)
pub mod ingredients; // upravljanje sastojcima (/api/ingredients/*)
pub mod middleware;   // JWT autentifikacija - provjerava token
pub mod ratings;     // ocjene i komentari (/api/recipes/:id/rate)
pub mod recipes;     // CRUD za recepte (/api/recipes/*)