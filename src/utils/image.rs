use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;
use std::process::Command;
use anyhow::{Result, anyhow};
use image::{GenericImageView, DynamicImage, ImageFormat, ImageBuffer, Rgba, RgbaImage};
use image::imageops::{resize, FilterType};
use log::{info, error, warn};

use crate::utils::webp;

const THUMBNAIL_SIZE: u32 = 200;

/// Create a basic colored placeholder thumbnail when no placeholder image is available
pub fn create_basic_placeholder_thumbnail(thumbnail_path: &Path) -> Result<()> {
    // Create a new RGBA image with blue color and a play icon
    let mut img = RgbaImage::new(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    
    // Fill with dark blue background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([41, 98, 255, 255]);
    }
    
    // Draw a basic play triangle in white (simplified)
    let center_x = THUMBNAIL_SIZE / 2;
    let center_y = THUMBNAIL_SIZE / 2;
    let icon_size = THUMBNAIL_SIZE / 3;
    
    for y in center_y - icon_size / 2..center_y + icon_size / 2 {
        for x in center_x - icon_size / 2..center_x + icon_size / 2 {
            // Simple triangle shape - convert u32 to i32 for abs()
            let y_diff = y as i32 - center_y as i32;
            let x_diff = x as i32 - center_x as i32;
            if x >= center_x && y_diff.abs() < x_diff {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
    
    // Save as WebP with good quality
    img.save_with_format(thumbnail_path, ImageFormat::WebP)?;
    
    Ok(())
}

/// Generate a thumbnail for an image file
pub fn generate_image_thumbnail(
    file_path: &Path,
    thumbnail_path: &Path,
) -> Result<()> {
    // Read the image
    let img = image::io::Reader::open(file_path)?
        .with_guessed_format()?
        .decode()?;
    
    // Create a transparent canvas for the thumbnail
    let mut thumbnail = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    
    // Fill with transparent background
    for pixel in thumbnail.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    // Calculate size preserving aspect ratio
    let (width, height) = img.dimensions();
    let ratio = width as f32 / height as f32;
    let (new_width, new_height) = if ratio > 1.0 {
        (THUMBNAIL_SIZE, (THUMBNAIL_SIZE as f32 / ratio) as u32)
    } else {
        ((THUMBNAIL_SIZE as f32 * ratio) as u32, THUMBNAIL_SIZE)
    };
    
    // Resize the image
    let resized = resize(&img, new_width, new_height, FilterType::Lanczos3);
    
    // Calculate offset to center the image
    let x_offset = (THUMBNAIL_SIZE - new_width) / 2;
    let y_offset = (THUMBNAIL_SIZE - new_height) / 2;
    
    // Copy the resized image onto the canvas
    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = *resized.get_pixel(x, y);
            thumbnail.put_pixel(x + x_offset, y + y_offset, pixel);
        }
    }
    
    // Save as WebP with good quality
    thumbnail.save_with_format(thumbnail_path, ImageFormat::WebP)?;
    
    Ok(())
}

/// Generate a thumbnail for a video file
pub fn generate_video_thumbnail(
    file_path: &Path,
    thumbnail_path: &Path,
) -> Result<()> {
    info!("Generating video thumbnail for: {}", file_path.display());
    
    // Create temporary file for the extracted frame
    let temp_frame = format!("/tmp/frame_{}.jpg", uuid::Uuid::new_v4());
    
    // Use FFmpeg to extract a frame at 10% of the video duration
    let output = match Command::new("ffmpeg")
        .arg("-i")
        .arg(file_path)
        .arg("-ss")
        .arg("00:00:01") // Extract frame at 1 second
        .arg("-frames:v")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg(&temp_frame)
        .output() {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute FFmpeg: {}", e);
                // Create a basic colored placeholder instead of failing
                info!("FFmpeg failed, creating basic colored placeholder thumbnail");
                return create_basic_placeholder_thumbnail(thumbnail_path);
            }
        };
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("FFmpeg error: {}", error);
        // Create a basic colored placeholder instead of failing
        info!("FFmpeg error, creating basic colored placeholder thumbnail");
        return create_basic_placeholder_thumbnail(thumbnail_path);
    }
    
    // Create thumbnail from the extracted frame
    let result = generate_image_thumbnail(Path::new(&temp_frame), thumbnail_path);
    
    // Clean up temporary file
    if let Err(e) = fs::remove_file(&temp_frame) {
        warn!("Failed to remove temporary frame file: {}", e);
    }
    
    result
}

/// Generate a thumbnail for a file based on its type
pub fn generate_thumbnail(
    file_path: &Path,
    thumbnail_dir: &str,
    _archive_dir: &str,
) -> Result<PathBuf> {
    // Check if file exists before trying to generate thumbnail
    if !file_path.exists() {
        return Err(anyhow!("File does not exist: {}", file_path.display()));
    }
    
    let _filename = file_path.file_name().ok_or_else(|| anyhow!("Invalid file path"))?;
    let base_name = file_path.file_stem().ok_or_else(|| anyhow!("Invalid file path"))?;
    let thumbnail_filename = format!("{}_thumbnail.webp", base_name.to_string_lossy());
    let thumbnail_path = Path::new(thumbnail_dir).join(&thumbnail_filename);
    
    // Skip if thumbnail already exists
    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }
    
    // Ensure parent directory exists
    if let Some(parent) = thumbnail_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let extension = file_path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    
    if extension == "webp" {
        // Check if it's an animated WebP
        if webp::is_animated_webp(file_path) {
            // Extract first frame for thumbnail
            // This requires loading the WebP and extracting just the first frame
            match image::io::Reader::open(file_path)
                .map_err(|e| anyhow!("Failed to open WebP: {}", e))?
                .with_guessed_format()
                .map_err(|e| anyhow!("Failed to guess format: {}", e))?
                .decode() {
                Ok(_img) => {
                    // Create thumbnail from the first frame
                    generate_image_thumbnail(&file_path, &thumbnail_path)?;
                },
                Err(e) => {
                    error!("Failed to decode WebP: {}", e);
                    return Err(anyhow!("Failed to decode WebP: {}", e));
                }
            }
        } else {
            // Regular WebP image (non-animated)
            generate_image_thumbnail(&file_path, &thumbnail_path)?;
        }
    } else if ["jpg", "jpeg", "png", "bmp", "gif"].contains(&extension.as_str()) {
        // Regular image file
        generate_image_thumbnail(&file_path, &thumbnail_path)?;
    } else if ["mp4", "avi", "mov", "mkv", "webm"].contains(&extension.as_str()) {
        // Video file
        generate_video_thumbnail(&file_path, &thumbnail_path)?;
    } else {
        // Unsupported file type
        return Err(anyhow!("Unsupported file type: {}", extension));
    }
    
    Ok(thumbnail_path)
}