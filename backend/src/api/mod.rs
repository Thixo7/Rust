use actix_web::{get, web, HttpResponse, Responder};
use std::fs;
use crate::logs::monitor::Alert; // ✅ Utilise le bon Alert (avec alert_type)
use crate::utils::{get_blocked_ips, unblock_ip};
use serde_json;
use serde::Deserialize;

const ALERTS_FILE: &str = "alerts.json";  // Fichier contenant les alertes

// Structure pour les paramètres de requête
#[derive(Deserialize)]
pub struct AlertQuery {
    pub alert_type: Option<String>,
}

#[get("/alerts")]
async fn get_alerts(query: web::Query<AlertQuery>) -> impl Responder {
    let data = fs::read_to_string(ALERTS_FILE).unwrap_or_else(|_| "".to_string());

    let mut alerts: Vec<Alert> = data
        .lines()
        .filter_map(|line| serde_json::from_str::<Alert>(line).ok())
        .collect();

    // Filtrage si alert_type est précisé
    if let Some(ref alert_type) = query.alert_type {
        alerts.retain(|a| a.alert_type == *alert_type);
    }

    HttpResponse::Ok().json(alerts)
}

#[get("/blocked")]
async fn get_blocked() -> impl Responder {
    let ips = get_blocked_ips();
    HttpResponse::Ok().json(ips)
}

async fn delete_blocked(ip: web::Path<String>) -> HttpResponse {
    let ip = ip.into_inner();

    if unblock_ip(&ip) {
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
