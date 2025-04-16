use std::fs::{OpenOptions, self, File};
use std::io::Write;
use chrono::{DateTime, Duration, Utc};
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

pub fn clean_old_alerts(max_age_hours: i64) {
    let Ok(content) = fs::read_to_string(ALERTS_FILE) else {
        return;
    };

    let mut valid_alerts = Vec::new();

    for line in content.lines() {
        if let Ok(alert) = serde_json::from_str::<Alert>(line) {
            if let Ok(ts) = DateTime::parse_from_rfc3339(&alert.timestamp) {
                let ts_utc = ts.with_timezone(&Utc);
                if Utc::now().signed_duration_since(ts_utc) <= Duration::hours(max_age_hours) {
                    valid_alerts.push(alert);
                }
            }
        }
    }

    let Ok(mut file) = File::create(ALERTS_FILE) else {
        return;
    };

    for alert in valid_alerts {
        let Ok(line) = serde_json::to_string(&alert) else { continue; };
        let _ = writeln!(file, "{}", line);
    }
}
