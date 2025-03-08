use actix_web::{web, HttpResponse, Responder, HttpRequest, post, cookie::{Cookie, SameSite}};
use actix_web::cookie::time::Duration;
use serde::{Deserialize, Serialize};
use log::{info, error};

use crate::models::AppState;
use crate::services::favorites;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(toggle_theme)
        .service(toggle_favorite);
}

/// Toggle between light and dark theme
#[post("/toggle-theme")]
async fn toggle_theme(req: HttpRequest) -> impl Responder {
    // Get current theme from cookie
    let current_theme = req.cookie("theme")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "dark".to_string());
    
    // Toggle theme
    let new_theme = if current_theme == "dark" { "light" } else { "dark" };
    
    // Create cookie that expires in 1 year
    let cookie = Cookie::build("theme", new_theme.clone())
        .max_age(Duration::days(365))
        .path("/")
        .secure(false) // Set to true in production with HTTPS
        .http_only(false)
        .same_site(SameSite::Lax)
        .finish();
    
    // Create response with cookie
    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({
            "theme": new_theme
        }))
}

#[derive(Deserialize)]
struct ToggleFavoriteRequest {
    file: String,
    is_favorite: bool,
}

/// Toggle favorite status for a file
#[post("/toggle-favorite")]
async fn toggle_favorite(
    data: web::Data<AppState>,
    req: web::Json<ToggleFavoriteRequest>,
) -> impl Responder {
    let filename = &req.file;
    
    if filename.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No file specified"
        }));
    }
    
    // Update favorites
    match favorites::toggle_favorite(filename, req.is_favorite, &data.favorites, &data.favorites_file) {
        Ok(is_favorite) => {
            return HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "file": filename,
                "is_favorite": is_favorite
            }));
        },
        Err(e) => {
            error!("Error toggling favorite status: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update favorite status: {}", e)
            }));
        }
    }
}