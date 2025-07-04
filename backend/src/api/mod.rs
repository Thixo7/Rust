use actix_web::{get, web, HttpResponse, Responder};
use std::fs;
use crate::utils::{Alert, get_blocked_ips}; // Import de la structure Alert
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

#[get("/blocked")]
async fn get_blocked() -> impl Responder {
    let ips = get_blocked_ips();
    HttpResponse::Ok().json(ips)
}

async fn delete_blocked(ip: web::Path<String>) -> HttpResponse {
    let ip = ip.into_inner();

    if crate::utils::unblock_ip(&ip) {
        HttpResponse::Ok().body(format!("✅ IP {} débloquée", ip))
    } else {
        HttpResponse::NotFound().body(format!("❌ IP {} non trouvée ou erreur", ip))
    }
}

// Fonction pour enregistrer les routes dans Actix-Web
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_alerts);
    cfg.service(get_blocked);
    cfg.route("/blocked/{ip}", web::delete().to(delete_blocked));
}
