use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, Duration};
use crate::utils::{save_alert, block_ip, is_ip_blocked};

const LOG_PATH: &str = "/var/log/auth.log";
const MAX_ATTEMPTS: usize = 5;
const TIME_WINDOW_SECONDS: i64 = 120; // 2 minutes
const ALERT_COOLDOWN_MINUTES: i64 = 10; // Délai entre deux alertes pour la même IP

pub fn start_monitoring() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

    watcher
        .watch(Path::new(LOG_PATH), RecursiveMode::NonRecursive)
        .unwrap();
    println!("📡 Surveillance de {}", LOG_PATH);

    let attempts = Arc::new(Mutex::new(HashMap::new()));
    let alerted_ips = Arc::new(Mutex::new(HashMap::new()));
    let mut last_position = 0;

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(Event {
                    kind: EventKind::Modify(_),
                    ..
                }) = event
                {
                    let mut file = File::open(LOG_PATH).unwrap();
                    let metadata = file.metadata().unwrap();
                    let current_len = metadata.len();

                    if current_len < last_position {
                        last_position = 0;
                    }

                    file.seek(SeekFrom::Start(last_position)).unwrap();
                    let reader = BufReader::new(file);

                    for line in reader.lines().filter_map(|l| l.ok()) {
                        if let Some(ip) = parse_failed_ssh_attempt(&line) {
                            let now = Utc::now();

                            // Vérifie si l’IP a été alertée récemment
                            let mut alerted = alerted_ips.lock().unwrap();
                            let recently_alerted = match alerted.get(&ip) {
                                Some(ts) => now.signed_duration_since(*ts) < Duration::minutes(ALERT_COOLDOWN_MINUTES),
                                None => false,
                            };

                            if recently_alerted {
                                continue; // Ne déclenche pas d’alerte
                            }

                            // Sinon, analyse les tentatives
                            let mut map = attempts.lock().unwrap();
                            if should_trigger_alert(&ip, &mut map) {
                                println!("🚨 Tentative de brute-force détectée depuis {}", ip);
                                save_alert(ip.clone());
                                alerted.insert(ip.clone(), now);

                                if !is_ip_blocked(&ip) {
                                    block_ip(&ip);
                                }
                            } 
                        }
                    }

                    last_position = current_len;
                }
            }
            Err(e) => eprintln!("❌ Erreur de surveillance : {:?}", e),
        }
    }
}

fn parse_failed_ssh_attempt(line: &str) -> Option<String> {
    if line.contains("Failed password") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(pos) = parts.iter().position(|&x| x == "from") {
            return parts.get(pos + 1).map(|s| s.to_string());
        }
    }
    None
}

fn should_trigger_alert(
    ip: &str,
    attempts_map: &mut HashMap<String, Vec<DateTime<Utc>>>
) -> bool {
    let now = Utc::now();
    let timestamps = attempts_map.entry(ip.to_string()).or_default();

    // Supprimer les tentatives trop anciennes
    timestamps.retain(|t| now.signed_duration_since(*t) <= Duration::seconds(TIME_WINDOW_SECONDS));

    // Ajouter cette tentative
    timestamps.push(now);

    // Déclencher si dépasse le seuil
    timestamps.len() >= MAX_ATTEMPTS
}
