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
                                Err(_) => false
                            }
                        },
                        Err(_) => false
                    }
                },
                Err(_) => false
            }
        },
        Err(_) => false
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

/// Convert a WebP animation to MP4
pub async fn convert_webp_to_mp4(
    file_path: &Path, 
    archive_dir: &str,
    thumbnail_dir: &str
) -> Result<Option<PathBuf>> {
    let file_path_str = file_path.to_string_lossy().to_string();
    let filename = file_path.file_name()
        .ok_or_else(|| anyhow!("Invalid file path"))?.to_string_lossy();
    
    info!("convert_webp_to_mp4 called for: {}", file_path_str);
    
    // Define MP4 output path
    let mp4_path = PathBuf::from(format!("{}.mp4", file_path_str));
    
    // Skip if the MP4 already exists
    if mp4_path.exists() {
        info!("MP4 already exists, skipping conversion: {}", mp4_path.display());
        return Ok(Some(mp4_path));
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
    let frame_count = match check_webp_animation_frames(file_path) {
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
    let progress_path_clone = progress_path.clone();
    let filename_clone = filename.to_string();
    
    // Progress monitoring task
    tokio::spawn(async move {
        while let Some(percent) = rx.recv().await {
            update_progress(
                &progress_path_clone,
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
    let mp4_path_clone = mp4_path.clone();
    let temp_dir_clone = temp_dir.clone();
    let progress_path_clone = progress_path.clone();
    let filename_clone = filename.to_string();
    let archive_dir = archive_dir.to_string();
    
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            // Use ffmpeg directly for conversion
            let output = Command::new("ffmpeg")
                .arg("-i")
                .arg(&file_path_str_clone)
                .arg("-c:v")
                .arg("libx264")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-crf")
                .arg("23")
                .arg("-y")
                .arg(&mp4_path_clone)
                .output();
            
            match output {
                Ok(output) => {
                    if output.status.success() {
                        tx_clone.try_send(100).unwrap_or_default();
                        
                        // Copy file timestamps from the original WebP to the MP4
                        if let Ok(webp_metadata) = fs::metadata(&file_path_str_clone) {
                            if let Ok(webp_modified) = webp_metadata.modified() {
                                // Ignore errors as this is not critical
                                let _ = filetime::set_file_mtime(&mp4_path_clone, filetime::FileTime::from_system_time(webp_modified));
                            }
                        }
                        
                        // Move original WebP to archive
                        let filename = Path::new(&file_path_str_clone).file_name().unwrap_or_default();
                        let archive_path = PathBuf::from(&archive_dir).join(filename);
                        
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
                        
                        // Update progress to completed
                        update_progress(
                            &progress_path_clone,
                            "completed",
                            Some(100),
                            Some(100),
                            None,
                            Some(&filename_clone),
                            Some(true),
                        ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                        
                        true
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr).to_string();
                        error!("FFmpeg error: {}", error);
                        
                        update_progress(
                            &progress_path_clone,
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
                    
                    update_progress(
                        &progress_path_clone,
                        "error",
                        None,
                        None,
                        Some(&e.to_string()),
                        Some(&filename_clone),
                        None,
                    ).unwrap_or_else(|e| warn!("Failed to update progress: {}", e));
                    
                    false
                }
            }
        }).await;
        
        // Clean up temp dir
        if let Err(e) = fs::remove_dir_all(&temp_dir_clone) {
            warn!("Failed to clean up temp directory: {}", e);
        }
        
        // Remove progress file after 10 seconds
        sleep(Duration::from_secs(10)).await;
        let _ = fs::remove_file(&progress_path_clone);
    });
    
    Ok(Some(mp4_path))
}

/// Update the progress of WebP to MP4 conversion
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