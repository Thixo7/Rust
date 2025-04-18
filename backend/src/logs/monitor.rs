use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::utils::{block_ip, is_ip_blocked};

const LOG_PATH: &str = "/var/log/auth.log";
const MAX_ATTEMPTS: usize = 5;
const TIME_WINDOW_SECONDS: i64 = 120;
const ALERT_COOLDOWN_MINUTES: i64 = 5;

#[derive(Serialize, Deserialize)]
struct Alert {
    ip: String,
    message: String,
    timestamp: String,
    alert_type: String,
}

pub fn start_monitoring() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

    watcher.watch(Path::new(LOG_PATH), RecursiveMode::NonRecursive).unwrap();
    println!("📡 Surveillance de {}", LOG_PATH);

    let attempts: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>> = Arc::new(Mutex::new(HashMap::new()));
    let alerted_ips: Arc<Mutex<HashMap<String, DateTime<Utc>>>> = Arc::new(Mutex::new(HashMap::new()));

    // 🆕 Position de lecture précédente du fichier log
    let mut last_position: u64 = 0;

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(Event {
                    kind: EventKind::Modify(_),
                    ..
                }) = event
                {
                    let mut file = File::open(LOG_PATH).unwrap();
                    file.seek(SeekFrom::Start(last_position)).unwrap(); // 📍 Reprendre à l'endroit précédent

                    let reader = BufReader::new(file);
                    let mut current_position = last_position;

                    for line in reader.lines().flatten() {
                        current_position += line.len() as u64 + 1; // Compte la ligne + \n

                        if let Some(ip) = extract_ip(&line) {
                            if line.contains("Failed password") {
                                let now = Utc::now();
                                let mut alerted = alerted_ips.lock().unwrap();
                                let recently_alerted = match alerted.get(&ip) {
                                    Some(ts) => now.signed_duration_since(*ts) < Duration::minutes(ALERT_COOLDOWN_MINUTES),
                                    None => false,
                                };
                                if recently_alerted {
                                    continue;
                                }

                                let mut map = attempts.lock().unwrap();
                                if should_trigger_alert(&ip, &mut map) {
                                    println!("🚨 Tentative de brute-force détectée depuis {}", ip);
                                    save_alert_with_type(ip.clone(), "Tentative de brute-force SSH détectée", "brute_force");
                                    alerted.insert(ip.clone(), now);
                                    if !is_ip_blocked(&ip) {
                                        block_ip(&ip);
                                    }
                                }
                            } else if line.contains("Accepted password") {
                                println!("✅ Connexion SSH réussie depuis {}", ip);
                                save_alert_with_type(ip, "Connexion SSH réussie", "login_success");
                            } else if line.contains("Invalid user") {
                                println!("❌ Utilisateur invalide depuis {}", ip);
                                save_alert_with_type(ip, "Tentative avec un utilisateur invalide", "invalid_user");
                            } else if line.contains("Disconnected from") {
                                println!("🔌 Déconnexion SSH depuis {}", ip);
                                save_alert_with_type(ip, "Déconnexion SSH", "disconnected");
                            }
                        }
                    }

                    last_position = current_position; // 🔁 Met à jour la position
                }
            }
            Err(e) => eprintln!("❌ Erreur de surveillance : {:?}", e),
        }
    }
}

fn should_trigger_alert(ip: &str, attempts_map: &mut HashMap<String, Vec<DateTime<Utc>>>) -> bool {
    let now = Utc::now();
    let timestamps = attempts_map.entry(ip.to_string()).or_default();
    timestamps.retain(|t| now.signed_duration_since(*t) <= Duration::seconds(TIME_WINDOW_SECONDS));
    timestamps.push(now);
    timestamps.len() >= MAX_ATTEMPTS
}

fn extract_ip(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if let Some(pos) = parts.iter().position(|&x| x == "from") {
        if let Some(ip) = parts.get(pos + 1) {
            if ip.parse::<std::net::IpAddr>().is_ok() {
                return Some(ip.to_string());
            }
        }
    }
    None
}

fn save_alert_with_type(ip: String, message: &str, alert_type: &str) {
    let alert = Alert {
        ip,
        message: message.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        alert_type: alert_type.to_string(),
    };

    let alert_json = serde_json::to_string(&alert).unwrap();

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("alerts.json")
        .unwrap();

    writeln!(file, "{}", alert_json).unwrap();
}
