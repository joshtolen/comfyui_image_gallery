use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use mime_guess::from_path;
use anyhow::{Result, anyhow};

/// Get a file's size formatted as a human-readable string
pub fn get_file_size_str(file_path: &Path) -> Result<String> {
    let metadata = fs::metadata(file_path)?;
    let size_bytes = metadata.len();
    
    // Convert to human-readable format
    if size_bytes < 1024 {
        Ok(format!("{} B", size_bytes))
    } else if size_bytes < 1024 * 1024 {
        let size_kb = size_bytes as f64 / 1024.0;
        Ok(format!("{:.1} KB", size_kb))
    } else {
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        Ok(format!("{:.2} MB", size_mb))
    }
}

/// Get a file's MIME type based on extension
pub fn get_file_mime_type(file_path: &Path) -> String {
    from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}

/// Get a file's modification time as a DateTime
pub fn get_file_mod_time(file_path: &Path) -> Result<DateTime<Utc>> {
    let metadata = fs::metadata(file_path)?;
    let mod_time = metadata.modified()?;
    
    Ok(DateTime::from(mod_time))
}

/// Get a file's creation time if available, falling back to modification time
pub fn get_file_creation_time(file_path: &Path) -> Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, birthtime is available
        if let Ok(created) = metadata.created() {
            return Ok(created);
        }
    }
    
    // Fall back to modification time
    metadata.modified().map_err(|e| anyhow!("Failed to get file time: {}", e))
}

/// Check if a file exists
pub fn file_exists(file_path: &Path) -> bool {
    file_path.exists() && file_path.is_file()
}

/// Read JSON from a file
pub fn read_json_file<T: serde::de::DeserializeOwned>(file_path: &Path) -> Result<T> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let data: T = serde_json::from_str(&contents)?;
    Ok(data)
}

/// Write JSON to a file
pub fn write_json_file<T: serde::Serialize>(file_path: &Path, data: &T) -> Result<()> {
    let contents = serde_json::to_string_pretty(data)?;
    fs::write(file_path, contents)?;
    Ok(())
}

/// Get the thumbnail path for a file
pub fn get_thumbnail_path(filename: &str, thumbnail_dir: &str) -> PathBuf {
    let base_name = Path::new(filename).file_stem().unwrap_or_default();
    let thumbnail_filename = format!("{}_thumbnail.webp", base_name.to_string_lossy());
    PathBuf::from(thumbnail_dir).join(thumbnail_filename)
}

/// Check if a thumbnail exists for a file
pub fn thumbnail_exists(filename: &str, thumbnail_dir: &str) -> bool {
    let webp_thumbnail = get_thumbnail_path(filename, thumbnail_dir);
    
    // Also check for legacy PNG thumbnail
    let base_name = Path::new(filename).file_stem().unwrap_or_default();
    let png_thumbnail_filename = format!("{}_thumbnail.png", base_name.to_string_lossy());
    let png_thumbnail = PathBuf::from(thumbnail_dir).join(png_thumbnail_filename);
    
    file_exists(&webp_thumbnail) || file_exists(&png_thumbnail)
}