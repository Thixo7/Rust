use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc::channel;

const LOG_PATH: &str = "/var/log/auth.log"; // Adapter selon ton OS

pub fn start_monitoring() {
    let (tx, rx) = channel();

    // Création du watcher avec la nouvelle syntaxe
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
                    // Lire le fichier et analyser les logs SSH
                    let file = File::open(LOG_PATH).unwrap();
                    let reader = BufReader::new(file);

                    for line in reader.lines().filter_map(|l| l.ok()) {
                        if let Some(ip) = parse_failed_ssh_attempt(&line) {
                            println!("🚨 Tentative de brute-force détectée depuis {}", ip);
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
