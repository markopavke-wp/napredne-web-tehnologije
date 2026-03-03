// Svi modeli podataka za aplikaciju
// Svaki modul odgovara jednoj ili više tabela u bazi

pub mod user;       // korisnici i autentifikacija
pub mod recipe;     // recepti
pub mod ingredient; // sastojci (kalorije, proteini, itd.)
pub mod rating;     // ocjene i komentari za recepte
pub mod fridge;     // virtualni frižider korisnika