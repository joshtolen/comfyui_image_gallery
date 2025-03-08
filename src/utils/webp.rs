use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs::{self, File, create_dir_all};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use image::{ImageFormat, DynamicImage};
use serde_json::json;
use anyhow::{Result, anyhow, Context};
use log::{info, error, warn};
use uuid::Uuid;

use crate::models::file::ConversionProgress;
use crate::utils::file;

/// Check if a WebP file is animated
pub fn is_animated_webp(file_path: &Path) -> bool {
    // Check if file exists before trying to process it
    if !file_path.exists() {
        log::warn!("WebP file doesn't exist: {}", file_path.display());
        return false;
    }
    
    if !file_path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("webp")) {
        return false;
    }
    
    match image::io::Reader::open(file_path) {
        Ok(reader) => {
            match reader.with_guessed_format() {
                Ok(reader) => {
                    // Try to decode the image
                    match reader.decode() {
                        Ok(_) => {
                            // Decode succeeded but we need to check for animation frames
                            // This requires doing a bit of binary parsing
                            match check_webp_animation_frames(file_path) {
                                Ok(frame_count) => frame_count > 1,
                                Err(e) => {
                                    log::warn!("Error checking WebP animation frames for {}: {}", file_path.display(), e);
                                    false
                                }
                            }
                        },
                        Err(e) => {
                            log::warn!("Error decoding WebP {}: {}", file_path.display(), e);
                            false
                        }
                    }
                },
                Err(e) => {
                    log::warn!("Error guessing format for {}: {}", file_path.display(), e);
                    false
                }
            }
        },
        Err(e) => {
            log::warn!("Error opening WebP file {}: {}", file_path.display(), e);
            false
        }
    }
}

/// Check number of frames in a WebP file by parsing the binary format
fn check_webp_animation_frames(file_path: &Path) -> Result<usize> {
    // This is a simplified approach that looks for ANIM chunk in the WebP file
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Check if this is a WebP file
    if buffer.len() < 12 || &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WEBP" {
        log::warn!("File is not a valid WebP: {}", file_path.display());
        return Ok(1); // Not a WebP or invalid format, assume 1 frame
    }
    
    // Look for ANIM chunk which indicates animation
    let mut pos = 12;
    while pos + 8 < buffer.len() {
        let chunk_type = &buffer[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]) as usize;
        
        if chunk_type == b"ANMF" {
            // Found an animation frame chunk
            return Ok(count_anmf_chunks(&buffer));
        }
        
        pos += 8 + chunk_size + (chunk_size & 1); // Chunks are padded to even bytes
    }
    
    Ok(1) // No animation frames found, assume 1 frame
}

/// Count ANMF chunks in WebP data to determine frame count
fn count_anmf_chunks(buffer: &[u8]) -> usize {
    let mut pos = 12; // Start after "RIFF" + size + "WEBP"
    let mut frame_count = 0;
    
    while pos + 8 < buffer.len() {
        let chunk_type = &buffer[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]) as usize;
        
        if chunk_type == b"ANMF" {
            frame_count += 1;
        }
        
        pos += 8 + chunk_size + (chunk_size & 1); // Chunks are padded to even bytes
        
        // Safety check to avoid infinite loops on malformed files
        if pos > buffer.len() {
            break;
        }
    }
    
    // If no frames were found but we have an ANIM chunk, assume at least 1 frame
    if frame_count == 0 {
        for i in 12..buffer.len() - 4 {
            if &buffer[i..i + 4] == b"ANIM" {
                return 1;
            }
        }
    }
    
    frame_count.max(1) // Return at least 1 frame
}

/// Convert a WebP animation to WebM
pub async fn convert_webp_to_webm(
    file_path: &Path, 
    archive_dir: &str,
    thumbnail_dir: &str
) -> Result<Option<PathBuf>> {
    let file_path_str = file_path.to_string_lossy().to_string();
    let filename = file_path.file_name()
        .ok_or_else(|| anyhow!("Invalid file path"))?.to_string_lossy();
    
    info!("convert_webp_to_webm called for: {}", file_path_str);
    
    // Define WebM output path
    let webm_path = PathBuf::from(format!("{}.webm", file_path_str));
    
    // Skip if the WebM already exists
    if webm_path.exists() {
        info!("WebM already exists, skipping conversion: {}", webm_path.display());
        return Ok(Some(webm_path));
    }
    
    // Verify it's an animated WebP
    if !is_animated_webp(file_path) {
        info!("Not an animated WebP, skipping conversion: {}", file_path_str);
        return Ok(None);
    }
    
    // Create progress file
    let base_name = file_path.file_stem().unwrap_or_default();
    let progress_path = PathBuf::from(thumbnail_dir)
        .join(format!("{}_progress.json", base_name.to_string_lossy()));
    
    let temp_dir = format!("/tmp/webp_conversion_{}", Uuid::new_v4());
    create_dir_all(&temp_dir)?;
    
    // Create a progress indicator file
    update_progress(
        &progress_path,
        "extracting_frames",
        Some(0),
        Some(0),
        None,
        Some(&filename.to_string()),
        None,
    )?;
    
    // Use ffmpeg to extract frames and create video
    // First, get frame count using our custom parser
    let _frame_count = match check_webp_animation_frames(file_path) {
        Ok(count) => count,
        Err(_) => {
            update_progress(
                &progress_path,
                "error",
                None,
                None,
                Some("Failed to determine frame count"),
                Some(&filename.to_string()),
                None,
            )?;
            return Err(anyhow!("Failed to determine frame count"));
        }
    };
    
    // Start a separate async task to monitor ffmpeg progress
    let (tx, mut rx) = mpsc::channel(32);
    let progress_path_string = progress_path.to_string_lossy().to_string();
    let filename_clone = filename.to_string();
    
    // Progress monitoring task
    tokio::spawn(async move {
        while let Some(percent) = rx.recv().await {
            update_progress(
                &PathBuf::from(&progress_path_string),
                "encoding_video",
                Some(percent),
                Some(100),
                None,
                Some(&filename_clone),
                None,
            ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
        }
    });
    
    // Run ffmpeg conversion in a separate thread to not block
    let tx_clone = tx.clone();
    let file_path_str_clone = file_path_str.clone();
    let webm_path_clone = webm_path.clone();
    let temp_dir_clone = temp_dir.clone();
    let progress_path_string2 = progress_path.to_string_lossy().to_string();
    let filename_clone = filename.to_string();
    let archive_dir_string = archive_dir.to_string();
    
    let temp_dir_for_cleanup = temp_dir.clone();
    tokio::spawn(async move {
        let progress_path_string3 = progress_path_string2.clone();
        let _result = tokio::task::spawn_blocking(move || {
            // Use the Python script to convert animated WebP to WebM
            info!("Using Python extractor script to convert WebP to WebM");
            
            // First check if the Python script exists
            let script_path = PathBuf::from("./WebP_Animated_Extractor.py");
            let docker_script_path = PathBuf::from("/app/WebP_Animated_Extractor.py");
            
            let script_exists = script_path.exists() || docker_script_path.exists();
            let script_to_use = if script_path.exists() {
                script_path.to_string_lossy().to_string()
            } else if docker_script_path.exists() {
                docker_script_path.to_string_lossy().to_string()
            } else {
                "./WebP_Animated_Extractor.py".to_string() // Will likely fail
            };
            
            let output = if script_exists {
                // Extract the parent directory of the output path
                let output_dir = Path::new(&webm_path_clone).parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                
                // Run the Python script
                info!("Running Python script: {} {} {}", script_to_use, file_path_str_clone, output_dir);
                Command::new("python3")
                    .arg(&script_to_use)
                    .arg(&file_path_str_clone)
                    .arg(&output_dir)
                    .output()
            } else {
                error!("WebP_Animated_Extractor.py script not found");
                
                // Fallback to static image extraction if script is missing
                info!("Python script not found, trying to extract a static frame as fallback");
                let static_webp = format!("{}/static.webp", temp_dir_clone);
                let static_created = Command::new("dwebp")
                    .arg(&file_path_str_clone)
                    .arg("-o")
                    .arg(&static_webp)
                    .output();
                
                let static_success = static_created.is_ok() && Path::new(&static_webp).exists();
                if static_success {
                    info!("Created static WebP, using as fallback");
                    // If webm_path_clone ends with .webm, we need a new path ending with .webp
                    let target_path = if webm_path_clone.to_string_lossy().ends_with(".webm") {
                        let new_path = webm_path_clone.to_string_lossy().replace(".webm", ".webp");
                        PathBuf::from(new_path)
                    } else {
                        webm_path_clone.clone()
                    };
                    
                    // Copy the static WebP to the target path
                    if let Err(e) = fs::copy(&static_webp, &target_path) {
                        error!("Failed to copy static WebP: {}", e);
                        Command::new("false").output()
                    } else {
                        info!("Using static WebP as fallback: {}", target_path.display());
                        Command::new("true").output()
                    }
                } else {
                    error!("Failed to extract static WebP frame");
                    Command::new("false").output()
                }
            };
            
            let conversion_success = match output {
                Ok(output) => {
                    if output.status.success() {
                        tx_clone.try_send(100).unwrap_or_default();
                        
                        // Verify the output file exists and is valid
                        if webm_path_clone.exists() && fs::metadata(&webm_path_clone).map(|m| m.len() > 0).unwrap_or(false) {
                            // Copy file timestamps from the original WebP to the WebM
                            if let Ok(webp_metadata) = fs::metadata(&file_path_str_clone) {
                                if let Ok(webp_modified) = webp_metadata.modified() {
                                    // Ignore errors as this is not critical
                                    let _ = filetime::set_file_mtime(&webm_path_clone, filetime::FileTime::from_system_time(webp_modified));
                                }
                            }
                            
                            // Move original WebP to archive
                            let filename = Path::new(&file_path_str_clone).file_name().unwrap_or_default();
                            let archive_path = PathBuf::from(&archive_dir_string).join(filename);
                            
                            if let Some(parent) = archive_path.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            
                            if let Err(e) = fs::rename(&file_path_str_clone, &archive_path) {
                                warn!("Failed to move WebP to archive: {}", e);
                                // Try copying instead
                                if let Err(e) = fs::copy(&file_path_str_clone, &archive_path) {
                                    warn!("Failed to copy WebP to archive: {}", e);
                                } else {
                                    let _ = fs::remove_file(&file_path_str_clone);
                                }
                            }
                            
                            // Remove any existing failure marker
                            let failed_marker = PathBuf::from(format!("{}.conversion_failed", file_path_str_clone));
                            if failed_marker.exists() {
                                if let Err(e) = fs::remove_file(&failed_marker) {
                                    warn!("Failed to remove conversion failure marker: {}", e);
                                } else {
                                    info!("Removed previous conversion failure marker");
                                }
                            }
                            
                            // Update progress to completed
                            update_progress(
                                &PathBuf::from(progress_path_string2.clone()),
                                "completed",
                                Some(100),
                                Some(100),
                                None,
                                Some(&filename_clone),
                                Some(true),
                            ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                            
                            true
                        } else {
                            error!("WebM file is empty or doesn't exist despite successful command");
                            
                            // Remove the empty or invalid file if it exists
                            if webm_path_clone.exists() {
                                let _ = fs::remove_file(&webm_path_clone);
                            }
                            
                            update_progress(
                                &PathBuf::from(&progress_path_string2),
                                "error",
                                None,
                                None,
                                Some("Generated file is invalid or empty"),
                                Some(&filename_clone),
                                None,
                            ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                            
                            false
                        }
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr).to_string();
                        error!("FFmpeg error: {}", error);
                        
                        // Remove any partial output file
                        if webm_path_clone.exists() {
                            let _ = fs::remove_file(&webm_path_clone);
                        }
                        
                        update_progress(
                            &PathBuf::from(&progress_path_string2),
                            "error",
                            None,
                            None,
                            Some(&error),
                            Some(&filename_clone),
                            None,
                        ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                        
                        false
                    }
                },
                Err(e) => {
                    error!("Failed to execute ffmpeg: {}", e);
                    
                    // Remove any partial output file
                    if webm_path_clone.exists() {
                        let _ = fs::remove_file(&webm_path_clone);
                    }
                    
                    update_progress(
                        &PathBuf::from(&progress_path_string2),
                        "error",
                        None,
                        None,
                        Some(&e.to_string()),
                        Some(&filename_clone),
                        None,
                    ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                    
                    false
                }
            };
            
            // If all conversion attempts failed, create a failed marker file
            if !conversion_success {
                let failed_marker = PathBuf::from(format!("{}.conversion_failed", file_path_str_clone));
                let _ = fs::write(&failed_marker, "Conversion failed after multiple attempts");
                info!("Created conversion failure marker: {}", failed_marker.display());
            }
        }).await;
        
        // Clean up temp dir
        if let Err(e) = fs::remove_dir_all(&temp_dir_for_cleanup) {
            warn!("Failed to clean up temp directory: {}", e);
        }
        
        // Remove progress file after 10 seconds
        sleep(Duration::from_secs(10)).await;
        let _ = fs::remove_file(&PathBuf::from(progress_path_string3));
    });
    
    // Return the path to the output file (which might not exist if conversion failed)
    // Check if the file exists before returning
    if webm_path.exists() {
        Ok(Some(webm_path))
    } else {
        // Check if we have a failure marker
        let failure_marker = PathBuf::from(format!("{}.conversion_failed", file_path_str));
        if failure_marker.exists() {
            // Conversion has already failed
            Err(anyhow!("Conversion already failed for this file"))
        } else {
            // No marker yet, let the caller decide how to handle it
            Ok(None)
        }
    }
}

/// Update the progress of WebP to WebM conversion
fn update_progress(
    progress_path: &Path,
    status: &str,
    progress: Option<usize>,
    total: Option<usize>,
    error: Option<&str>,
    filename: Option<&str>,
    archived: Option<bool>,
) -> Result<()> {
    let progress_data = ConversionProgress {
        status: status.to_string(),
        progress,
        total,
        filename: filename.unwrap_or("").to_string(),
        in_queue: None,
        archived,
        error: error.map(|s| s.to_string()),
    };
    
    file::write_json_file(progress_path, &progress_data)
}