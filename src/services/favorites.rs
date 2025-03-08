use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use anyhow::{Result, anyhow};
use log::{info, error};

use crate::utils::file;

/// Load favorites from a JSON file
pub fn load_favorites(favorites_file: &str) -> Result<HashMap<String, bool>> {
    let path = Path::new(favorites_file);
    
    if !path.exists() {
        info!("Favorites file doesn't exist, creating empty favorites");
        return Ok(HashMap::new());
    }
    
    match file::read_json_file::<HashMap<String, bool>>(path) {
        Ok(favorites) => {
            info!("Loaded {} favorites from {}", favorites.len(), favorites_file);
            Ok(favorites)
        },
        Err(e) => {
            error!("Error loading favorites: {}", e);
            Ok(HashMap::new())
        }
    }
}

/// Save favorites to a JSON file
pub fn save_favorites(favorites: &HashMap<String, bool>, favorites_file: &str) -> Result<()> {
    let path = Path::new(favorites_file);
    
    file::write_json_file(path, favorites)
        .map_err(|e| anyhow!("Failed to save favorites: {}", e))
}

/// Toggle a file's favorite status
pub fn toggle_favorite(
    filename: &str,
    is_favorite: bool,
    favorites: &Mutex<HashMap<String, bool>>,
    favorites_file: &str,
) -> Result<bool> {
    // Update the favorites map
    let mut favorites_map = favorites
        .lock()
        .map_err(|_| anyhow!("Failed to lock favorites mutex"))?;
    
    if is_favorite {
        favorites_map.insert(filename.to_string(), true);
    } else {
        favorites_map.remove(filename);
    }
    
    // Save to file
    save_favorites(&favorites_map, favorites_file)?;
    
    Ok(is_favorite)
}