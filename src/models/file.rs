use serde::{Deserialize, Serialize};

/// Represents file metadata for the gallery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Filename
    pub filename: String,
    
    /// Path to thumbnail
    pub thumbnail: String,
    
    /// MIME type
    pub file_type: String,
    
    /// File size formatted as string (e.g., "1.2 MB")
    pub size: String,
    
    /// Flag indicating if it's a video file
    pub is_video: bool,
    
    /// Original source file (might be different from filename for converted files)
    pub source: String,
    
    /// Flag indicating if this is a WebP that was converted to MP4
    pub is_converted_webp: bool,
    
    /// Flag indicating if this is a WebM file
    pub is_webm: bool,
    
    /// Flag indicating if the original WebP is in the archive
    pub in_archive: bool,
    
    /// Flag indicating if the file is a favorite
    pub is_favorite: bool,
}

/// Conversion progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionProgress {
    /// Status of the conversion ("not_started", "extracting_frames", "encoding_video", "completed", "error")
    pub status: String,
    
    /// Current progress value
    pub progress: Option<usize>,
    
    /// Total steps or frames
    pub total: Option<usize>,
    
    /// Filename being processed
    pub filename: String,
    
    /// Flag indicating if the file is in the conversion queue
    pub in_queue: Option<bool>,
    
    /// Flag indicating if the file is archived
    pub archived: Option<bool>,
    
    /// Error message, if any
    pub error: Option<String>,
}

/// File status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    /// Filename
    pub filename: String,
    
    /// Flag indicating if the file exists in the original directory
    pub exists_in_original: bool,
    
    /// Flag indicating if the file exists in the archive directory
    pub exists_in_archive: bool,
    
    /// Flag indicating if the file exists as an MP4 (for converted WebPs)
    pub exists_as_mp4: bool,
}