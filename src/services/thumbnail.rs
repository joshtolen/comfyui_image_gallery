use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, anyhow};
use log::{info, error, debug};

use crate::utils::{file, image, webp};

/// Process thumbnails in the queue
pub async fn process_thumbnail_queue(
    thumbnail_queue: std::sync::Arc<Mutex<Vec<String>>>, 
    is_processing: std::sync::Arc<Mutex<bool>>,
    file_dir: String,
    thumbnail_dir: String,
    archive_dir: String,
) -> Result<()> {
    // Lock the processing flag
    let mut is_processing_guard = is_processing
        .lock()
        .map_err(|_| anyhow!("Failed to lock processing flag"))?;
    
    // Skip if already processing
    if *is_processing_guard {
        return Ok(());
    }
    
    // Set processing flag
    *is_processing_guard = true;
    
    // Release the lock
    drop(is_processing_guard);
    
    // Process the queue in a separate task
    tokio::spawn(async move {
        loop {
            // Get a file from the queue
            let file = {
                let mut queue = match thumbnail_queue.lock() {
                    Ok(queue) => queue,
                    Err(_) => {
                        error!("Failed to lock thumbnail queue");
                        break;
                    }
                };
                
                if queue.is_empty() {
                    break;
                }
                
                queue.remove(0)
            };
            
            debug!("Processing thumbnail for {}", file);
            
            // Check if file exists
            let file_path = PathBuf::from(&file_dir).join(&file);
            if !file_path.exists() {
                info!("File doesn't exist, skipping thumbnail generation: {}", file_path.display());
                continue;
            }
            
            // Generate thumbnail
            match image::generate_thumbnail(&file_path, &thumbnail_dir, &archive_dir) {
                Ok(_) => info!("Generated thumbnail for {}", file),
                Err(e) => error!("Failed to generate thumbnail for {}: {}", file, e),
            }
            
            // Check if WebP needs to be converted to MP4
            let extension = file_path.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            
            if extension == "webp" && webp::is_animated_webp(&file_path) {
                // Start WebP to MP4 conversion
                match webp::convert_webp_to_mp4(&file_path, &archive_dir, &thumbnail_dir).await {
                    Ok(_) => info!("Converted WebP to MP4: {}", file),
                    Err(e) => error!("Failed to convert WebP to MP4: {}: {}", file, e),
                }
            }
            
            // Small delay to avoid CPU overload
            sleep(Duration::from_millis(100)).await;
        }
        
        // Reset processing flag
        if let Ok(mut is_processing) = is_processing.lock() {
            *is_processing = false;
        }
    });
    
    Ok(())
}

/// Start background thumbnail processor if needed
pub async fn start_thumbnail_processor(
    thumbnail_queue: &Mutex<Vec<String>>, 
    is_processing: &Mutex<bool>,
    file_dir: &str,
    thumbnail_dir: &str,
    archive_dir: &str,
) -> Result<()> {
    use std::sync::Arc;
    // Check if there are items in the queue
    let queue_empty = {
        let queue = thumbnail_queue
            .lock()
            .map_err(|_| anyhow!("Failed to lock thumbnail queue"))?;
        queue.is_empty()
    };
    
    if queue_empty {
        return Ok(());
    }
    
    // Check if processor is already running
    let is_processing_value = {
        let is_processing_guard = is_processing
            .lock()
            .map_err(|_| anyhow!("Failed to lock processing flag"))?;
        *is_processing_guard
    };
    
    if !is_processing_value {
        // Start the processor
        process_thumbnail_queue(
            Arc::new(Mutex::new(Vec::new())), 
            Arc::new(Mutex::new(false)),
            file_dir.to_string(),
            thumbnail_dir.to_string(),
            archive_dir.to_string(),
        ).await?;
    }
    
    Ok(())
}

/// Get thumbnails for a list of files
pub fn get_thumbnails(files: &[String], thumbnail_dir: &str) -> Vec<String> {
    files.iter()
        .map(|filename| {
            let base_name = Path::new(filename).file_stem().unwrap_or_default();
            let webp_thumbnail = format!("{}_thumbnail.webp", base_name.to_string_lossy());
            let png_thumbnail = format!("{}_thumbnail.png", base_name.to_string_lossy());
            
            let webp_path = Path::new(thumbnail_dir).join(&webp_thumbnail);
            let png_path = Path::new(thumbnail_dir).join(&png_thumbnail);
            
            // Prefer WebP, fall back to PNG
            if webp_path.exists() {
                webp_thumbnail
            } else if png_path.exists() {
                png_thumbnail
            } else {
                // Default to WebP (will be generated later)
                webp_thumbnail
            }
        })
        .collect()
}