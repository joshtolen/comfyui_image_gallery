use actix_web::{HttpResponse, http::header};
use chrono::{DateTime, Duration, Utc};

/// Add cache headers to an HTTP response
pub fn add_cache_headers(mut response: HttpResponse, duration_secs: u64) -> HttpResponse {
    // Calculate expiration date
    let now = Utc::now();
    let expiration = now + Duration::seconds(duration_secs as i64);
    
    // Format according to HTTP spec
    let expires_header = format_http_date(expiration);
    
    // Add headers to response
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, header::HeaderValue::from_str(&format!("public, max-age={}", duration_secs)).unwrap());
    
    response
        .headers_mut()
        .insert(header::EXPIRES, header::HeaderValue::from_str(&expires_header).unwrap());
    
    response
}

/// Format a DateTime as an HTTP date
fn format_http_date(dt: DateTime<Utc>) -> String {
    // HTTP date format: https://tools.ietf.org/html/rfc7231#section-7.1.1.1
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}