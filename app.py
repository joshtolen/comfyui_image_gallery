import os
import time
import threading
import mimetypes
from datetime import datetime
from functools import wraps
from flask import Flask, render_template, request, jsonify, send_from_directory, Response, make_response
from PIL import Image, ImageFilter
from moviepy import VideoFileClip
from werkzeug.utils import secure_filename

app = Flask(__name__)

# Define the directory where your original images are stored
file_dir = '/app/static/images/output'

# Define the directory where thumbnails will be stored
thumbnail_dir = '/app/static/thumbnails'

# Ensure the thumbnail directory exists
os.makedirs(thumbnail_dir, exist_ok=True)

# Set up caching - 1 week in seconds
CACHE_DURATION = 604800

# Background thumbnail generation queue
thumbnail_queue = []
is_processing = False

# Helper function to add cache headers to responses
def add_cache_headers(response):
    response.headers['Cache-Control'] = f'public, max-age={CACHE_DURATION}'
    response.headers['Expires'] = datetime.utcnow().strftime('%a, %d %b %Y %H:%M:%S GMT')
    return response

# Background thumbnail generator
def process_thumbnail_queue():
    global is_processing, thumbnail_queue
    
    if is_processing:
        return
        
    is_processing = True
    
    try:
        while thumbnail_queue:
            file = thumbnail_queue.pop(0)
            generate_thumbnail(file)
            time.sleep(0.1)  # Small delay to avoid CPU overload
    finally:
        is_processing = False

def start_thumbnail_processor():
    thread = threading.Thread(target=process_thumbnail_queue)
    thread.daemon = True
    thread.start()

@app.route('/file-info/<path:filename>')
def file_info(filename):
    """Endpoint to get file metadata"""
    try:
        file_path = os.path.join(file_dir, filename)
        if not os.path.exists(file_path):
            return jsonify({"error": "File not found"}), 404
            
        # Get file size in KB
        file_size = os.path.getsize(file_path) / 1024
        size_str = f"{file_size:.1f} KB" if file_size < 1024 else f"{file_size/1024:.2f} MB"
        
        # Get file modification time
        mod_time = datetime.fromtimestamp(os.path.getmtime(file_path))
        
        # Get file MIME type
        mime_type, _ = mimetypes.guess_type(file_path)
        
        return jsonify({
            "filename": filename,
            "type": mime_type or "application/octet-stream",
            "size": size_str,
            "modified": mod_time.strftime("%Y-%m-%d %H:%M:%S")
        })
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/delete-files', methods=['POST'])
def delete_files():
    try:
        # Get file list from the request
        data = request.get_json()
        files_to_delete = data.get('files', [])

        # Delete files from the file system
        for filename in files_to_delete:
            file_path = os.path.join(file_dir, filename)
            thumbnail_path = os.path.join(thumbnail_dir, f"{os.path.splitext(filename)[0]}_thumbnail.webp")
            if os.path.exists(file_path):
                os.remove(file_path)
            if os.path.exists(thumbnail_path):
                os.remove(thumbnail_path)
                
            # Check for PNG thumbnail as fallback (for backward compatibility)
            png_thumbnail = os.path.join(thumbnail_dir, f"{os.path.splitext(filename)[0]}_thumbnail.png")
            if os.path.exists(png_thumbnail):
                os.remove(png_thumbnail)

        return jsonify({'message': 'Selected files have been deleted successfully.'})
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/delete-file', methods=['POST'])
def delete_file():
    try:
        # Get file from the request
        data = request.get_json()
        filename = data.get('file')
        
        if not filename:
            return jsonify({'error': 'No file specified'}), 400

        file_path = os.path.join(file_dir, filename)
        thumbnail_path = os.path.join(thumbnail_dir, f"{os.path.splitext(filename)[0]}_thumbnail.webp")
        
        # Delete the original file
        if os.path.exists(file_path):
            os.remove(file_path)
        
        # Delete the thumbnail
        if os.path.exists(thumbnail_path):
            os.remove(thumbnail_path)
            
        # Check for PNG thumbnail as fallback
        png_thumbnail = os.path.join(thumbnail_dir, f"{os.path.splitext(filename)[0]}_thumbnail.png")
        if os.path.exists(png_thumbnail):
            os.remove(png_thumbnail)

        return jsonify({'message': 'File deleted successfully.'})
    except Exception as e:
        return jsonify({'error': str(e)}), 500
@app.route('/')
def image_gallery():
    # Get user preference for theme
    theme = request.cookies.get('theme', 'dark')
    
    # Get a list of image files from the specified directory
    try:
        image_files = [f for f in os.listdir(file_dir) if f.lower().endswith(('.jpg', '.png', '.jpeg', '.gif', '.bmp', '.webp', '.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5'))]
    except FileNotFoundError:
        # Create the directory if it doesn't exist
        os.makedirs(file_dir, exist_ok=True)
        image_files = []
    
    # Sort the list of image files by last modified date (newest first)
    if image_files:
        image_files.sort(key=lambda x: os.path.getmtime(os.path.join(file_dir, x)), reverse=True)

    # Get the page number from the query string (default to 1)
    page = int(request.args.get('page', 1))

    # Calculate the total number of pages
    images_per_page = 50  # Reduced from 100 for better initial load time
    total_pages = (len(image_files) + images_per_page - 1) // images_per_page
    
    # Ensure page is within bounds
    if page > total_pages and total_pages > 0:
        page = total_pages
    elif page < 1:
        page = 1

    # Calculate the start and end indices for the current page
    start_idx = (page - 1) * images_per_page
    end_idx = min(start_idx + images_per_page, len(image_files))

    # Slice the image list to display images for the current page
    current_images = image_files[start_idx:end_idx] if image_files else []
    
    # Get or generate thumbnails
    current_thumbnails = get_thumbnails(current_images)
    
    # Prepare image metadata for display
    image_data = []
    for img, thumb in zip(current_images, current_thumbnails):
        file_path = os.path.join(file_dir, img)
        size_kb = os.path.getsize(file_path) / 1024
        size_text = f"{size_kb:.1f} KB" if size_kb < 1024 else f"{size_kb/1024:.2f} MB"
        
        mime_type, _ = mimetypes.guess_type(file_path)
        
        # Detect if it's a video
        is_video = img.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5'))
        
        image_data.append({
            'filename': img,
            'thumbnail': thumb,
            'type': mime_type or ('video/mp4' if is_video else 'image/jpeg'),
            'size': size_text,
            'is_video': is_video
        })

    # Queue thumbnails for background generation
    if current_images:
        for img in current_images:
            if img not in thumbnail_queue and not thumbnail_exists(img):
                thumbnail_queue.append(img)
        
        # Start the background processor if needed
        if thumbnail_queue and not is_processing:
            start_thumbnail_processor()
    
    # Make response with cache headers for 5 minutes
    response = make_response(render_template('index.html', 
                                           images=image_data,
                                           total_pages=total_pages, 
                                           page=page,
                                           theme=theme))
    
    # Set a shorter cache time for the main page (5 minutes)
    response.headers['Cache-Control'] = 'public, max-age=300'
    response.headers['Expires'] = (datetime.utcnow() + 
                                 datetime.timedelta(seconds=300)).strftime('%a, %d %b %Y %H:%M:%S GMT')
    
    return response


@app.route('/toggle-theme', methods=['POST'])
def toggle_theme():
    """Toggle between light and dark theme"""
    current_theme = request.cookies.get('theme', 'dark')
    new_theme = 'light' if current_theme == 'dark' else 'dark'
    
    response = make_response(jsonify({'theme': new_theme}))
    response.set_cookie('theme', new_theme, max_age=31536000)  # 1 year
    
    return response


@app.route('/serve-thumbnail/<path:filename>')
def serve_thumbnail(filename):
    """Serve thumbnail with proper caching headers"""
    response = send_from_directory(thumbnail_dir, filename)
    return add_cache_headers(response)

def thumbnail_exists(filename):
    """Check if a thumbnail exists for the given file"""
    base_name = os.path.splitext(filename)[0]
    webp_path = os.path.join(thumbnail_dir, f"{base_name}_thumbnail.webp")
    png_path = os.path.join(thumbnail_dir, f"{base_name}_thumbnail.png")
    return os.path.exists(webp_path) or os.path.exists(png_path)

def generate_thumbnail(file):
    """Generate a thumbnail for a single file, optimized for WebP format"""
    # Create a consistent naming convention for thumbnail files
    base_name = os.path.splitext(file)[0]
    thumbnail_path = os.path.join(thumbnail_dir, f"{base_name}_thumbnail.webp")
    
    # Skip if thumbnail already exists
    if os.path.exists(thumbnail_path):
        return thumbnail_path
        
    try:
        # Check if the file exists in the directory
        file_path = os.path.join(file_dir, file)
        if not os.path.exists(file_path):
            print(f"Original file not found: {file_path}")
            return None
            
        # Check if the file is an image or a video (by extension)
        if file.lower().endswith(('.jpg', '.jpeg', '.png', '.bmp', '.gif', '.webp')):
            # Handle image files
            with Image.open(file_path) as original_image:
                # Create blank canvas to ensure square thumbnails
                max_size = 200
                thumb = Image.new('RGBA', (max_size, max_size), (0, 0, 0, 0))
                
                # Resize maintaining aspect ratio
                original_image.thumbnail((max_size, max_size))
                
                # Center the image on the canvas
                offset = ((max_size - original_image.width) // 2,
                         (max_size - original_image.height) // 2)
                
                # Handle images without alpha channel
                if original_image.mode != 'RGBA' and original_image.mode != 'LA':
                    if original_image.mode != 'RGB':
                        original_image = original_image.convert('RGB')
                else:
                    thumb = Image.new('RGBA', (max_size, max_size), (0, 0, 0, 0))
                
                # Paste the resized image onto the center of the canvas
                if 'A' in original_image.mode:
                    thumb.paste(original_image, offset, original_image)
                else:
                    # For non-transparent images, create a background
                    thumb = Image.new('RGB', (max_size, max_size), (30, 30, 30))
                    thumb.paste(original_image, offset)
                
                # Save as WebP with enhanced quality
                thumb.save(thumbnail_path, 'WEBP', quality=85)
                
        elif file.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5')):
            # Handle video files
            try:
                video_path = os.path.join(file_dir, file)
                with VideoFileClip(video_path) as video:
                    # Get video duration
                    duration = video.duration
                    # Capture frame at 10% of the video or at 1 second, whichever is less
                    frame_time = min(duration * 0.1, 1.0)
                    frame = video.get_frame(frame_time)
                    
                    # Convert to PIL Image
                    thumbnail_image = Image.fromarray(frame)
                    
                    # Create the thumbnail
                    max_size = 200
                    thumbnail_image.thumbnail((max_size, max_size))
                    
                    # Add play button overlay
                    # Create a new image with a semi-transparent overlay
                    overlay = Image.new('RGBA', thumbnail_image.size, (0, 0, 0, 0))
                    
                    # Save as WebP
                    thumbnail_image.save(thumbnail_path, 'WEBP', quality=80)
            except Exception as video_error:
                print(f"Video thumbnail error for {file}: {str(video_error)}")
                # Use a default video thumbnail
                default_thumbnail = os.path.join(os.path.dirname(__file__), 'static', 'video_placeholder.png')
                if os.path.exists(default_thumbnail):
                    import shutil
                    shutil.copy(default_thumbnail, thumbnail_path)
                
        return thumbnail_path
            
    except Exception as e:
        print(f"Error generating thumbnail for {file}: {str(e)}")
        return None

def generate_thumbnails(file_list):
    """Generate thumbnails for a list of files (legacy support)"""
    thumbnails = []
    for file in file_list:
        thumbnails.append(generate_thumbnail(file))
    return thumbnails

def get_thumbnails(current_files):
    """Get thumbnail filenames for a list of files, preferring WebP"""
    current_thumbnails = []

    for image in current_files:
        # Get the file name without the extension
        base_name = os.path.splitext(image)[0]
        
        # Check for WebP thumbnail first
        webp_thumbnail = f"{base_name}_thumbnail.webp"
        webp_path = os.path.join(thumbnail_dir, webp_thumbnail)
        
        # Fall back to PNG if WebP doesn't exist
        png_thumbnail = f"{base_name}_thumbnail.png"
        png_path = os.path.join(thumbnail_dir, png_thumbnail)
        
        if os.path.exists(webp_path):
            current_thumbnails.append(webp_thumbnail)
        elif os.path.exists(png_path):
            current_thumbnails.append(png_thumbnail)
        else:
            # No thumbnail exists yet, use the WebP path and queue for generation
            current_thumbnails.append(webp_thumbnail)
            
    return current_thumbnails

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=9999)
