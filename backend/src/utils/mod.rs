use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::process::Command;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

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

pub fn block_ip(ip: &str) {
    println!("🛑 [DEBUG] Tentative de blocage de l'IP : '{}'", ip);

    if ip.trim().is_empty() {
        println!("⚠️ IP vide, blocage annulé.");
        return;
    }

    let _ = Command::new("sudo")
        .arg("iptables")
        .arg("-A")
        .arg("INPUT")
        .arg("-s")
        .arg(ip)
        .arg("-j")
        .arg("DROP")
        .output()
        .unwrap_or_else(|e| {
            panic!("❌ Erreur lors du blocage avec iptables : {:?}", e);
        });

    // Enregistrement dans blocked_ips.json
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("blocked_ips.json")
        .unwrap();

    writeln!(file, "{}", ip).unwrap();
}

pub fn is_ip_blocked(ip: &str) -> bool {
    let file = match File::open("blocked_ips.json") {
        Ok(f) => f,
        Err(_) => return false, // Si le fichier n'existe pas encore → pas bloquée
    };

    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        if line.trim() == ip {
            return true;
        }
    }

    false
}

pub fn get_blocked_ips() -> Vec<String> {
    let file = match File::open("blocked_ips.json") {
        Ok(f) => f,
        Err(_) => return vec![], // Si le fichier n'existe pas encore
    };

    let reader = BufReader::new(file);
    let mut ips = Vec::new();

    for line in reader.lines().flatten() {
        let ip = line.trim();
        if !ip.is_empty() {
            ips.push(ip.to_string());
        }
    }

    ips
}

pub fn unblock_ip(ip: &str) -> bool {
    let file = match File::open("blocked_ips.json") {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().flatten().collect();

    let updated: Vec<String> = lines
        .into_iter()
        .filter(|line| line.trim() != ip)
        .collect();

    let mut file = match File::create("blocked_ips.json") {
        Ok(f) => f,
        Err(_) => return false,
    };

    for line in updated {
        let _ = writeln!(file, "{}", line);
    }

    // Supprime la règle iptables (si présente)
    let _ = Command::new("sudo")
        .arg("iptables")
        .arg("-D")
        .arg("INPUT")
        .arg("-s")
        .arg(ip)
        .arg("-j")
        .arg("DROP")
        .output();

    true
}
