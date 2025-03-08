use std::collections::HashMap;
use std::sync::Mutex;
use tera::Tera;

pub mod file;

/// Application state shared across all routes
pub struct AppState {
    /// Directory where original images are stored
    pub file_dir: String,
    
    /// Directory where thumbnails are stored
    pub thumbnail_dir: String,
    
    /// Directory for archived files (converted WebPs)
    pub archive_dir: String,
    
    /// File to store favorites
    pub favorites_file: String,
    
    /// Collection of favorite files
    pub favorites: Mutex<HashMap<String, bool>>,
    
    /// Queue for background thumbnail generation
    pub thumbnail_queue: Mutex<Vec<String>>,
    
    /// Flag to indicate if thumbnail processing is active
    pub is_processing: Mutex<bool>,
    
    /// Template engine
    pub template_engine: Tera,
}