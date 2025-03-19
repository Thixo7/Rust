use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc::channel;
use serde::{Serialize, Deserialize};
use crate::utils::save_alert;

const LOG_PATH: &str = "/var/log/auth.log";  

#[derive(Serialize, Deserialize)]
struct Alert {
    ip: String,
    message: String,
    timestamp: String,
}

pub fn start_monitoring() {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

    watcher.watch(Path::new(LOG_PATH), RecursiveMode::NonRecursive).unwrap();

    println!("📡 Surveillance de {}", LOG_PATH);

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(Event {
                    kind: EventKind::Modify(_),
                    ..
                }) = event
                {
                    let file = File::open(LOG_PATH).unwrap();
                    let reader = BufReader::new(file);

                    for line in reader.lines().filter_map(|l| l.ok()) {
                         if let Some(ip) = parse_failed_ssh_attempt(&line) {
                            println!("🚨 Tentative de brute-force détectée depuis {}", ip);
                            save_alert(ip);  // 🔥 Ajout de l'alerte dans alerts.json
                         }
                    }
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
