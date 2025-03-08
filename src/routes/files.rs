use std::fs;
use std::path::{Path, PathBuf};
use actix_web::{web, HttpResponse, Responder, get, post, error};
use serde::{Deserialize, Serialize};
use log::{info, error};

use crate::models::AppState;
use crate::models::file::{FileStatus, ConversionProgress};
use crate::utils::{file, response, webp};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file_info)
        .service(delete_files)
        .service(delete_file)
        .service(serve_thumbnail)
        .service(check_file_status)
        .service(conversion_progress);
}

/// Get file info
#[get("/file-info/{filename:.*}")]
async fn file_info(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let filename = path.into_inner();
    let file_path = Path::new(&data.file_dir).join(&filename);
    
    if !file_path.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        }));
    }
    
    // Get file size
    let size = match file::get_file_size_str(&file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Error getting file size: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get file size: {}", e)
            }));
        }
    };
    
    // Get file modification time
    let mod_time = match file::get_file_mod_time(&file_path) {
        Ok(t) => t.format("%Y-%m-%d %H:%M:%S").to_string(),
        Err(e) => {
            error!("Error getting file modification time: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get modification time: {}", e)
            }));
        }
    };
    
    // Get file type
    let file_type = file::get_file_mime_type(&file_path);
    
    HttpResponse::Ok().json(serde_json::json!({
        "filename": filename,
        "type": file_type,
        "size": size,
        "modified": mod_time
    }))
}

#[derive(Deserialize)]
struct DeleteFilesRequest {
    files: Vec<String>,
}

/// Delete multiple files
#[post("/delete-files")]
async fn delete_files(
    data: web::Data<AppState>,
    req: web::Json<DeleteFilesRequest>,
) -> impl Responder {
    for filename in &req.files {
        let file_path = Path::new(&data.file_dir).join(filename);
        
        // Get base name for thumbnail
        let base_name = Path::new(filename).file_stem().unwrap_or_default();
        let webp_thumbnail = Path::new(&data.thumbnail_dir)
            .join(format!("{}_thumbnail.webp", base_name.to_string_lossy()));
        let png_thumbnail = Path::new(&data.thumbnail_dir)
            .join(format!("{}_thumbnail.png", base_name.to_string_lossy()));
        
        // Delete original file
        if file_path.exists() {
            if let Err(e) = fs::remove_file(&file_path) {
                error!("Error deleting file {}: {}", filename, e);
            }
        }
        
        // Delete thumbnails
        if webp_thumbnail.exists() {
            if let Err(e) = fs::remove_file(&webp_thumbnail) {
                error!("Error deleting WebP thumbnail for {}: {}", filename, e);
            }
        }
        
        if png_thumbnail.exists() {
            if let Err(e) = fs::remove_file(&png_thumbnail) {
                error!("Error deleting PNG thumbnail for {}: {}", filename, e);
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Selected files have been deleted successfully."
    }))
}

#[derive(Deserialize)]
struct DeleteFileRequest {
    file: String,
}

/// Delete a single file
#[post("/delete-file")]
async fn delete_file(
    data: web::Data<AppState>,
    req: web::Json<DeleteFileRequest>,
) -> impl Responder {
    let filename = &req.file;
    
    if filename.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No file specified"
        }));
    }
    
    let file_path = Path::new(&data.file_dir).join(filename);
    
    // Get base name for thumbnail
    let base_name = Path::new(filename).file_stem().unwrap_or_default();
    let webp_thumbnail = Path::new(&data.thumbnail_dir)
        .join(format!("{}_thumbnail.webp", base_name.to_string_lossy()));
    let png_thumbnail = Path::new(&data.thumbnail_dir)
        .join(format!("{}_thumbnail.png", base_name.to_string_lossy()));
    
    // Delete original file
    if file_path.exists() {
        if let Err(e) = fs::remove_file(&file_path) {
            error!("Error deleting file {}: {}", filename, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete file: {}", e)
            }));
        }
    }
    
    // Delete thumbnails
    if webp_thumbnail.exists() {
        if let Err(e) = fs::remove_file(&webp_thumbnail) {
            error!("Error deleting WebP thumbnail for {}: {}", filename, e);
        }
    }
    
    if png_thumbnail.exists() {
        if let Err(e) = fs::remove_file(&png_thumbnail) {
            error!("Error deleting PNG thumbnail for {}: {}", filename, e);
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "File deleted successfully."
    }))
}

/// Serve a thumbnail with caching
#[get("/serve-thumbnail/{filename:.*}")]
async fn serve_thumbnail(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let filename = path.into_inner();
    let file_path = Path::new(&data.thumbnail_dir).join(&filename);
    
    if !file_path.exists() {
        return HttpResponse::NotFound().finish();
    }
    
    // Read file content
    let content = match fs::read(&file_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Error reading thumbnail {}: {}", filename, e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    // Determine content type
    let content_type = if filename.ends_with(".webp") {
        "image/webp"
    } else if filename.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    };
    
    // Create response with cache headers
    let cache_duration = 604800; // 1 week in seconds
    let response = HttpResponse::Ok()
        .content_type(content_type)
        .body(content);
    
    response::add_cache_headers(response, cache_duration)
}

/// Check file status
#[get("/check-file-status/{filename:.*}")]
async fn check_file_status(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let filename = path.into_inner();
    let original_path = Path::new(&data.file_dir).join(&filename);
    let archive_path = Path::new(&data.archive_dir).join(&filename);
    let mp4_path = Path::new(&data.file_dir).join(format!("{}.mp4", filename));
    
    let status = FileStatus {
        filename: filename.clone(),
        exists_in_original: original_path.exists(),
        exists_in_archive: archive_path.exists(),
        exists_as_mp4: mp4_path.exists(),
    };
    
    HttpResponse::Ok().json(status)
}

/// Get WebP to MP4 conversion progress
#[get("/conversion-progress/{filename:.*}")]
async fn conversion_progress(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let filename = path.into_inner();
    info!("Checking conversion progress for: {}", filename);
    
    // Get base name for progress file
    let base_name = Path::new(&filename).file_stem().unwrap_or_default();
    let progress_path = Path::new(&data.thumbnail_dir)
        .join(format!("{}_progress.json", base_name.to_string_lossy()));
    
    if progress_path.exists() {
        // Read progress file
        match file::read_json_file::<ConversionProgress>(&progress_path) {
            Ok(progress) => {
                return HttpResponse::Ok().json(progress);
            },
            Err(e) => {
                error!("Error reading progress file: {}", e);
                // Fall through to check other states
            }
        }
    }
    
    // Check if MP4 already exists or WebP is in archive
    let mp4_path = Path::new(&data.file_dir).join(format!("{}.mp4", filename));
    let archive_path = Path::new(&data.archive_dir).join(&filename);
    
    info!("No progress file, checking if MP4 exists: {}", mp4_path.display());
    info!("Checking if file is in archive: {}", archive_path.display());
    
    if mp4_path.exists() || archive_path.exists() {
        return HttpResponse::Ok().json(serde_json::json!({
            "status": "completed",
            "progress": 100,
            "total": 100,
            "filename": filename,
            "archived": archive_path.exists()
        }));
    }
    
    // Check if file is in queue
    let in_queue = match data.thumbnail_queue.lock() {
        Ok(queue) => queue.contains(&filename),
        Err(_) => {
            error!("Failed to lock thumbnail queue");
            false
        }
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "not_started",
        "in_queue": in_queue,
        "filename": filename
    }))
}