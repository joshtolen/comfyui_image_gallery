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
    
    // Check if there are items in the queue
    let queue_empty = {
        let queue = match thumbnail_queue.lock() {
            Ok(queue) => queue,
            Err(_) => return Err(anyhow!("Failed to lock thumbnail queue"))
        };
        queue.is_empty()
    };
    
    if queue_empty {
        // No items to process
        info!("No items in thumbnail queue, skipping processing");
        return Ok(());
    }
    
    info!("Starting thumbnail processor with queue");
    
    // Process the queue in a separate task
    tokio::spawn(async move {
        let mut processed_count = 0;
        
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
                
                let queue_size = queue.len();
                
                if queue_size == 0 {
                    info!("Thumbnail queue is empty, processing complete");
                    break;
                }
                
                info!("Processing file {}/{} from thumbnail queue", processed_count + 1, queue_size);
                queue.remove(0)
            };
            
            processed_count += 1;
            
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
                Err(e) => {
                    error!("Failed to generate thumbnail for {}: {}", file, e);
                    
                    // In case of failure, create a basic color placeholder
                    let base_name = file_path.file_stem().unwrap_or_default();
                    let thumbnail_path = Path::new(&thumbnail_dir)
                        .join(format!("{}_thumbnail.webp", base_name.to_string_lossy()));
                    
                    // Only try to create placeholder if the error wasn't that the file doesn't exist
                    if file_path.exists() {
                        if let Err(e2) = image::create_basic_placeholder_thumbnail(&thumbnail_path) {
                            error!("Failed to create placeholder thumbnail: {}", e2);
                        } else {
                            info!("Created basic placeholder thumbnail for {}", file);
                        }
                    }
                }
            }
            
            // Check if WebP needs to be converted to MP4
            let extension = file_path.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            
            if extension == "webp" && webp::is_animated_webp(&file_path) {
                // Check if conversion has already failed
                let failure_marker = format!("{}.conversion_failed", file_path.to_string_lossy());
                if Path::new(&failure_marker).exists() {
                    info!("Skipping previously failed conversion for: {}", file);
                } else {
                    // Start WebP to WebM conversion
                    match webp::convert_webp_to_webm(&file_path, &archive_dir, &thumbnail_dir).await {
                        Ok(Some(output_path)) => {
                            if output_path.exists() {
                                // Check if it's a WebM or a WebP (for fallback static images)
                                let extension = output_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                if extension.eq_ignore_ascii_case("webm") {
                                    info!("Successfully converted WebP to WebM: {}", file);
                                } else if extension.eq_ignore_ascii_case("webp") {
                                    info!("Created static WebP fallback (animation conversion failed): {}", file);
                                } else {
                                    info!("Created conversion output: {} as {}", file, output_path.display());
                                }
                            } else {
                                error!("Output file not created despite successful conversion: {}", file);
                                // Create a failure marker since the file doesn't exist
                                let failure_marker = format!("{}.conversion_failed", file_path.to_string_lossy());
                                if let Err(e) = std::fs::write(&failure_marker, "File not created after conversion") {
                                    error!("Failed to create failure marker: {}", e);
                                }
                            }
                        },
                        Ok(None) => info!("WebP is not animated, no conversion needed: {}", file),
                        Err(e) => error!("Failed to convert WebP to WebM: {}: {}", file, e),
                    }
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
        // Get the current queue contents
        let queue_contents = {
            let queue = thumbnail_queue.lock()
                .map_err(|_| anyhow!("Failed to lock thumbnail queue"))?;
            queue.clone()
        };
        
        // Create new mutex with the queue contents
        let new_queue = Arc::new(Mutex::new(queue_contents));
        let new_processing = Arc::new(Mutex::new(false));
        
        // Start the processor with fresh mutex objects
        process_thumbnail_queue(
            new_queue,
            new_processing,
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
                info!("Found WebP thumbnail for {}", filename);
                webp_thumbnail
            } else if png_path.exists() {
                info!("Found PNG thumbnail for {}", filename);
                png_thumbnail
            } else {
                // Default to WebP (will be generated later)
                info!("No thumbnail found for {}, adding to queue", filename);
                webp_thumbnail
            }
        })
        .collect()
}