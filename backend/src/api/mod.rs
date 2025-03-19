use actix_web::{get, web, HttpResponse, Responder};
use std::fs;

const ALERTS_FILE: &str = "alerts.json";  // Stockage des alertes en JSON

#[get("/alerts")]
async fn get_alerts() -> impl Responder {
    let data = fs::read_to_string(ALERTS_FILE).unwrap_or_else(|_| "[]".to_string());
    HttpResponse::Ok().content_type("application/json").body(data)
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_alerts);
}
