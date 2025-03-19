use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json;

const ALERTS_FILE: &str = "alerts.json";  // Emplacement du fichier JSON

#[derive(Serialize, Deserialize)]
pub struct Alert {
    pub ip: String,
    pub message: String,
    pub timestamp: String,
}

pub fn save_alert(ip: String) {
    let alert = Alert {
        ip: ip.clone(),
        message: "Tentative de brute-force SSH détectée".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    let alert_json = serde_json::to_string(&alert).unwrap();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ALERTS_FILE)
        .unwrap();

    writeln!(file, "{}", alert_json).unwrap();
}
