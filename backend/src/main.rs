mod api;
mod logs; // Importe le module logs (qui contient monitor.rs)
mod utils;

use actix_web::{App, HttpServer};
use std::thread;
use logs::monitor::start_monitoring;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Rust IDS démarré !");

    // Lancer la surveillance des logs dans un thread séparé
    thread::spawn(|| start_monitoring());

    // Lancer l'API REST
    HttpServer::new(|| {
        App::new()
            .configure(api::routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
