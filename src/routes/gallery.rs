use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use actix_web::{web, HttpResponse, HttpRequest, Responder, get, cookie::Cookie};
use chrono::{Utc, Duration};
use tera::Context;
use log::{info, error};
use mime_guess::from_path;

use crate::models::AppState;
use crate::models::file::FileInfo;
use crate::services::thumbnail;
use crate::utils::{file, image, response, webp};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}

/// Main gallery index route
#[get("/")]
async fn index(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_time = std::time::Instant::now();
    
    // Get theme preference from cookie (default to dark)
    let theme = req.cookie("theme")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "dark".to_string());
    
    // Get file type filter from query params
    let query_string = req.query_string();
    let file_type = if let Ok(query) = web::Query::<HashMap<String, String>>::from_query(query_string) {
        query.get("type").cloned().unwrap_or_else(|| "all".to_string())
    } else {
        "all".to_string()
    };
    
    // Get page number from query params
    let page = if let Ok(query) = web::Query::<HashMap<String, String>>::from_query(query_string) {
        query.get("page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1)
    } else {
        1
    };
    
    // Get list of files
    let all_files = match get_filtered_files(&data, &file_type) {
        Ok(files) => files,
        Err(e) => {
            error!("Error getting files: {}", e);
            return HttpResponse::InternalServerError().body("Failed to get files");
        }
    };
    
    // Pagination
    let images_per_page = 50;
    let total_pages = (all_files.len() + images_per_page - 1) / images_per_page;
    let page = if page > total_pages && total_pages > 0 { total_pages } else if page < 1 { 1 } else { page };
    
    let start_idx = (page - 1) * images_per_page;
    let end_idx = std::cmp::min(start_idx + images_per_page, all_files.len());
    
    let current_files = if all_files.is_empty() {
        vec![]
    } else {
        all_files[start_idx..end_idx].to_vec()
    };
    
    // Get thumbnails
    let current_thumbnails = thumbnail::get_thumbnails(&current_files, &data.thumbnail_dir);
    
    // Prepare image metadata
    let mut image_data = Vec::new();
    for (i, filename) in current_files.iter().enumerate() {
        let file_path = Path::new(&data.file_dir).join(filename);
        
        if !file_path.exists() {
            continue;
        }
        
        // Get file metadata
        let size = match file::get_file_size_str(&file_path) {
            Ok(size) => size,
            Err(e) => {
                error!("Error getting file size: {}", e);
                "Unknown".to_string()
            }
        };
        
        let mime_type = file::get_file_mime_type(&file_path);
        let is_video = mime_type.starts_with("video/") || 
                       filename.to_lowercase().ends_with(".mp4") || 
                       filename.to_lowercase().ends_with(".avi") || 
                       filename.to_lowercase().ends_with(".mov") || 
                       filename.to_lowercase().ends_with(".mkv") || 
                       filename.to_lowercase().ends_with(".webm");
        
        // Check if it's a WebP and has an MP4 version
        let is_webp = filename.to_lowercase().ends_with(".webp");
        let mp4_path = Path::new(&data.file_dir).join(format!("{}.mp4", filename));
        let has_mp4 = mp4_path.exists();
        
        // Determine source file (MP4 for animated WebP)
        let source = if is_webp && has_mp4 {
            format!("{}.mp4", filename)
        } else {
            filename.clone()
        };
        
        // Determine if this is a converted WebP
        let is_converted_webp = is_webp && has_mp4;
        
        // Check if it's a WebM
        let is_webm = filename.to_lowercase().ends_with(".webm");
        
        // Check if the original WebP is in archive
        let mut in_archive = false;
        if is_converted_webp {
            let archive_path = Path::new(&data.archive_dir).join(filename);
            in_archive = archive_path.exists();
        }
        
        // Check if it's a favorite
        let is_favorite = match data.favorites.lock() {
            Ok(favorites) => favorites.contains_key(filename),
            Err(_) => {
                error!("Failed to lock favorites mutex");
                false
            }
        };
        
        // Add to image data
        image_data.push(FileInfo {
            filename: filename.clone(),
            thumbnail: current_thumbnails.get(i).cloned().unwrap_or_default(),
            file_type: mime_type,
            size,
            is_video: is_video || (is_webp && has_mp4),
            source,
            is_converted_webp,
            is_webm,
            in_archive,
            is_favorite,
        });
    }
    
    // Get counts for different types
    let (image_count, video_count, favorite_count) = get_counts(&all_files, &data);
    
    // Queue thumbnails for background generation
    if !current_files.is_empty() {
        for filename in &current_files {
            let file_path = Path::new(&data.file_dir).join(filename);
            let webp_thumbnail = file::get_thumbnail_path(filename, &data.thumbnail_dir);
            let png_base_name = Path::new(filename).file_stem().unwrap_or_default();
            let png_thumbnail = Path::new(&data.thumbnail_dir)
                .join(format!("{}_thumbnail.png", png_base_name.to_string_lossy()));
            
            if !file_path.exists() {
                continue;
            }
            
            if !webp_thumbnail.exists() && !png_thumbnail.exists() {
                if let Ok(mut queue) = data.thumbnail_queue.lock() {
                    if !queue.contains(filename) {
                        queue.push(filename.clone());
                    }
                } else {
                    error!("Failed to lock thumbnail queue");
                }
            }
        }
        
        // Start background processor if needed
        let _ = thumbnail::start_thumbnail_processor(
            &data.thumbnail_queue,
            &data.is_processing,
            &data.file_dir,
            &data.thumbnail_dir,
            &data.archive_dir,
        ).await;
    }
    
    // Render template
    let mut context = Context::new();
    context.insert("images", &image_data);
    context.insert("total_pages", &total_pages);
    context.insert("page", &page);
    context.insert("theme", &theme);
    context.insert("file_type", &file_type);
    context.insert("image_count", &image_count);
    context.insert("video_count", &video_count);
    context.insert("favorite_count", &favorite_count);
    context.insert("total_count", &all_files.len());
    
    let rendered = match data.template_engine.render("index.html", &context) {
        Ok(r) => r,
        Err(e) => {
            error!("Template rendering error: {}", e);
            return HttpResponse::InternalServerError().body("Template rendering error");
        }
    };
    
    // Create response with shorter cache time (5 minutes)
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered);
    
    let cache_duration = 300; // 5 minutes
    let response = response::add_cache_headers(response, cache_duration);
    
    info!("Gallery rendered in {:?}", start_time.elapsed());
    
    response
}

// Helper function to get filtered files
fn get_filtered_files(
    data: &web::Data<AppState>,
    file_type: &str,
) -> Result<Vec<String>, String> {
    // List files in the directory
    let dir_entries = match fs::read_dir(&data.file_dir) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    };
    
    // Get list of archived files
    let mut archive_files = std::collections::HashSet::new();
    if let Ok(entries) = fs::read_dir(&data.archive_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                archive_files.insert(filename.to_string());
            }
        }
    }
    
    // Filter and collect files
    let mut all_files = Vec::new();
    for entry in dir_entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            // Skip files in archive
            if archive_files.contains(filename) {
                continue;
            }
            
            // Only include supported file types
            if filename.to_lowercase().ends_with(".jpg") ||
               filename.to_lowercase().ends_with(".jpeg") ||
               filename.to_lowercase().ends_with(".png") ||
               filename.to_lowercase().ends_with(".gif") ||
               filename.to_lowercase().ends_with(".bmp") ||
               filename.to_lowercase().ends_with(".webp") ||
               filename.to_lowercase().ends_with(".mp4") ||
               filename.to_lowercase().ends_with(".avi") ||
               filename.to_lowercase().ends_with(".mov") ||
               filename.to_lowercase().ends_with(".mkv") ||
               filename.to_lowercase().ends_with(".webm") {
                all_files.push(filename.to_string());
            }
        }
    }
    
    // Find which WebP files are animated
    let mut animated_webps = std::collections::HashSet::new();
    for filename in &all_files {
        if filename.to_lowercase().ends_with(".webp") {
            let file_path = Path::new(&data.file_dir).join(filename);
            let animated = webp::is_animated_webp(&file_path);
            let mp4_exists = Path::new(&data.file_dir).join(format!("{}.mp4", filename)).exists();
            
            if animated || mp4_exists {
                animated_webps.insert(filename.clone());
                
                // Queue conversion for animated WebPs if MP4 doesn't exist
                if animated && !mp4_exists {
                    if let Ok(mut queue) = data.thumbnail_queue.lock() {
                        if !queue.contains(filename) {
                            queue.push(filename.clone());
                        }
                    } else {
                        error!("Failed to lock thumbnail queue");
                    }
                }
            }
        }
    }
    
    // Filter based on file type
    let filtered_files = match file_type {
        "images" => all_files.into_iter()
            .filter(|f| {
                (f.to_lowercase().ends_with(".jpg") ||
                 f.to_lowercase().ends_with(".jpeg") ||
                 f.to_lowercase().ends_with(".png") ||
                 f.to_lowercase().ends_with(".gif") ||
                 f.to_lowercase().ends_with(".bmp") ||
                 (f.to_lowercase().ends_with(".webp") && !animated_webps.contains(f))) &&
                !f.to_lowercase().ends_with(".webp.mp4")
            })
            .collect(),
        "videos" => all_files.into_iter()
            .filter(|f| {
                f.to_lowercase().ends_with(".mp4") ||
                f.to_lowercase().ends_with(".avi") ||
                f.to_lowercase().ends_with(".mov") ||
                f.to_lowercase().ends_with(".mkv") ||
                f.to_lowercase().ends_with(".webm") ||
                f.to_lowercase().ends_with(".webp.mp4") ||
                animated_webps.contains(f)
            })
            .collect(),
        "favorites" => {
            let favorites_guard = match data.favorites.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    error!("Failed to lock favorites mutex");
                    return Ok(all_files);
                }
            };
            
            all_files.into_iter()
                .filter(|f| favorites_guard.contains_key(f))
                .collect()
        },
        _ => all_files,
    };
    
    // Sort files by creation/modification time
    let mut sorted_files = filtered_files;
    sorted_files.sort_by(|a, b| {
        // Get file time for each file
        let time_a = get_file_time(a, &data.file_dir, &data.archive_dir);
        let time_b = get_file_time(b, &data.file_dir, &data.archive_dir);
        
        // Sort newest first
        time_b.cmp(&time_a)
    });
    
    Ok(sorted_files)
}

// Helper function to get file creation/modification time
fn get_file_time(filename: &str, file_dir: &str, archive_dir: &str) -> std::time::SystemTime {
    let file_path = Path::new(file_dir).join(filename);
    
    // For MP4 files converted from WebP, check if the original WebP exists in archive
    if filename.to_lowercase().ends_with(".mp4") {
        // Check if this might be a converted WebP
        let webp_name = &filename[0..filename.len().saturating_sub(4)]; // Remove .mp4 extension
        if webp_name.to_lowercase().ends_with(".webp") {
            // Check if original WebP is in archive
            let archive_path = Path::new(archive_dir).join(webp_name);
            if archive_path.exists() {
                if let Ok(time) = file::get_file_creation_time(&archive_path) {
                    return time;
                }
            }
        }
    }
    
    // Standard file time logic for non-converted files
    file::get_file_creation_time(&file_path).unwrap_or_else(|_| {
        std::time::SystemTime::now()
    })
}

// Helper function to get counts for different file types
fn get_counts(
    all_files: &[String],
    data: &web::Data<AppState>,
) -> (usize, usize, usize) {
    // Find which WebP files are animated
    let mut animated_webps = std::collections::HashSet::new();
    for filename in all_files {
        if filename.to_lowercase().ends_with(".webp") {
            let file_path = Path::new(&data.file_dir).join(filename);
            let animated = webp::is_animated_webp(&file_path);
            let mp4_exists = Path::new(&data.file_dir).join(format!("{}.mp4", filename)).exists();
            
            if animated || mp4_exists {
                animated_webps.insert(filename.clone());
            }
        }
    }
    
    // Count images
    let image_count = all_files.iter()
        .filter(|f| {
            (f.to_lowercase().ends_with(".jpg") ||
             f.to_lowercase().ends_with(".jpeg") ||
             f.to_lowercase().ends_with(".png") ||
             f.to_lowercase().ends_with(".gif") ||
             f.to_lowercase().ends_with(".bmp") ||
             (f.to_lowercase().ends_with(".webp") && !animated_webps.contains(&f.to_string()))) &&
            !f.to_lowercase().ends_with(".webp.mp4")
        })
        .count();
    
    // Count videos
    let video_count = all_files.iter()
        .filter(|f| {
            f.to_lowercase().ends_with(".mp4") ||
            f.to_lowercase().ends_with(".avi") ||
            f.to_lowercase().ends_with(".mov") ||
            f.to_lowercase().ends_with(".mkv") ||
            f.to_lowercase().ends_with(".webm") ||
            f.to_lowercase().ends_with(".webp.mp4") ||
            animated_webps.contains(&f.to_string())
        })
        .count();
    
    // Count favorites
    let favorite_count = match data.favorites.lock() {
        Ok(favorites) => {
            all_files.iter()
                .filter(|f| favorites.contains_key(*f))
                .count()
        },
        Err(_) => {
            error!("Failed to lock favorites mutex");
            0
        }
    };
    
    (image_count, video_count, favorite_count)
}