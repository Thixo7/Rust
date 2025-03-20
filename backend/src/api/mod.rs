use actix_web::{get, web, HttpResponse, Responder};
use std::fs;
use crate::utils::Alert; // Import de la structure Alert
use serde_json;

const ALERTS_FILE: &str = "alerts.json";  // Fichier contenant les alertes

#[get("/alerts")]
async fn get_alerts() -> impl Responder {
    // Lire le fichier alerts.json
    let data = fs::read_to_string(ALERTS_FILE).unwrap_or_else(|_| "[]".to_string());

    // Transformer chaque ligne en objet JSON
    let alerts: Vec<Alert> = data
        .lines()
        .filter_map(|line| serde_json::from_str::<Alert>(line).ok())
        .collect();

    HttpResponse::Ok().json(alerts)  // Retourne les alertes en JSON
}

// Fonction pour enregistrer les routes dans Actix-Web
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_alerts);
}
