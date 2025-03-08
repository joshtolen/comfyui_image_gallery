use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use env_logger::Env;
use std::sync::Mutex;
use tera::Tera;

// Import our modules
mod models;
mod routes;
mod services;
mod utils;

// Import our app state and routes
use models::AppState;
use routes::{gallery, files, favorites};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger with info level
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    // Define paths
    let file_dir = std::env::var("FILE_DIR").unwrap_or_else(|_| "/app/static/images/output".to_string());
    let thumbnail_dir = std::env::var("THUMBNAIL_DIR").unwrap_or_else(|_| "/app/static/thumbnails".to_string());
    let archive_dir = format!("{}/archive", file_dir);
    let favorites_file = std::env::var("FAVORITES_FILE").unwrap_or_else(|_| "/app/static/favorites.json".to_string());

    // Create directories if they don't exist
    std::fs::create_dir_all(&file_dir).expect("Failed to create file directory");
    std::fs::create_dir_all(&thumbnail_dir).expect("Failed to create thumbnail directory");
    std::fs::create_dir_all(&archive_dir).expect("Failed to create archive directory");

    // Initialize template engine
    let mut tera = Tera::new("templates/**/*").expect("Failed to initialize template engine");
    tera.autoescape_on(vec!["html", ".html", ".htm"]);
    
    // Load favorites
    let favorites = services::favorites::load_favorites(&favorites_file)
        .unwrap_or_else(|_| std::collections::HashMap::new());
    
    // Initialize thumbnail processing queue
    let thumbnail_queue = Mutex::new(Vec::new());
    let is_processing = Mutex::new(false);

    // Define and start HTTP server
    log::info!("Starting server at http://0.0.0.0:9999");
    HttpServer::new(move || {
        // Initialize CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            // Configure app state
            .app_data(web::Data::new(AppState {
                file_dir: file_dir.clone(),
                thumbnail_dir: thumbnail_dir.clone(),
                archive_dir: archive_dir.clone(),
                favorites_file: favorites_file.clone(),
                favorites: Mutex::new(favorites.clone()),
                thumbnail_queue: thumbnail_queue.clone(),
                is_processing: is_processing.clone(),
                template_engine: tera.clone(),
            }))
            // Enable logger, compress responses
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(cors)
            // Register services
            .configure(gallery::configure)
            .configure(files::configure)
            .configure(favorites::configure)
            // Serve static files
            .service(Files::new("/static", "static").prefer_utf8(true))
    })
    .bind(("0.0.0.0", 9999))?
    .run()
    .await
}
