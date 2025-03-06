import os
import time
import threading
import mimetypes
from datetime import datetime, timedelta
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

# Define directory for archived files (converted WebPs)
archive_dir = '/app/static/images/archive'

# Ensure the thumbnail and archive directories exist
os.makedirs(thumbnail_dir, exist_ok=True)
os.makedirs(archive_dir, exist_ok=True)

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
def is_animated_webp(file_path):
    """Check if a WebP file is animated"""
    if not file_path.lower().endswith('.webp'):
        return False
        
    try:
        with Image.open(file_path) as img:
            # If it has more than one frame, it's an animation
            is_animated = hasattr(img, 'n_frames') and img.n_frames > 1
            if is_animated:
                print(f"Detected animated WebP: {file_path} with {img.n_frames} frames")
            return is_animated
    except Exception as e:
        print(f"Error checking if WebP is animated: {file_path}, Error: {str(e)}")
        return False

@app.route('/')
def image_gallery():
    # Get user preference for theme
    theme = request.cookies.get('theme', 'dark')
    
    # Get file type filter from query string (default to 'all')
    file_type = request.args.get('type', 'all')
    
    # Get a list of all supported files from the specified directory
    try:
        # Get list of files but exclude those in the archive directory
        archive_files = set()
        if os.path.exists(archive_dir):
            try:
                archive_files = set(os.listdir(archive_dir))
                print(f"Found {len(archive_files)} files in archive directory")
            except Exception as e:
                print(f"Error reading archive directory: {str(e)}")
                
        # Only include files that aren't in the archive directory
        all_files = [f for f in os.listdir(file_dir) 
                     if f.lower().endswith(('.jpg', '.png', '.jpeg', '.gif', '.bmp', '.webp', '.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5'))
                     and f not in archive_files]
        print(f"Found {len(all_files)} files in main directory (excluding archived files)")
    except FileNotFoundError:
        # Create the directory if it doesn't exist
        os.makedirs(file_dir, exist_ok=True)
        all_files = []
    
    # Find which WebP files are animated
    animated_webps = set()
    for f in all_files:
        if f.lower().endswith('.webp'):
            file_path = os.path.join(file_dir, f)
            animated = is_animated_webp(file_path)
            mp4_exists = os.path.exists(file_path + '.mp4')
            
            if animated or mp4_exists:
                print(f"Adding {f} to animated WebPs list. Animated: {animated}, MP4 exists: {mp4_exists}")
                animated_webps.add(f)
                
                # Always trigger conversion for animated WebPs if MP4 doesn't exist
                if animated and not mp4_exists and not f in thumbnail_queue:
                    print(f"Queueing conversion for animated WebP: {f}")
                    thumbnail_queue.append(f)
    
    # Filter files based on selected type
    if file_type == 'images':
        image_files = [f for f in all_files if 
                      (f.lower().endswith(('.jpg', '.png', '.jpeg', '.gif', '.bmp')) or 
                       (f.lower().endswith('.webp') and f not in animated_webps)) and 
                      not f.lower().endswith('.webp.mp4')]
    elif file_type == 'videos':
        image_files = [f for f in all_files if 
                      f.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5')) or 
                      f.lower().endswith('.webp.mp4') or 
                      f in animated_webps]
    else:
        # Default: show all files
        image_files = all_files
    
    # Sort the list of files by creation time (if available) or modification time (newest first)
    if image_files:
        def get_file_time(filename):
            file_path = os.path.join(file_dir, filename)
            # Try to get creation time first, fall back to modified time
            try:
                # st_birthtime is available on macOS, not on Linux
                # st_ctime on Linux is inode change time, not creation time
                # On Windows, st_ctime is creation time
                stats = os.stat(file_path)
                if hasattr(stats, 'st_birthtime'):  # macOS
                    return stats.st_birthtime
                else:  # Linux/Windows fallback
                    return stats.st_mtime
            except:
                # Fallback to modification time if there's an error
                return os.path.getmtime(file_path)
                
        image_files.sort(key=get_file_time, reverse=True)
        print(f"Sorted {len(image_files)} files by creation/modification time (newest first)")

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
        
        # Handle original source for display/playback
        # If it's a .webp that should be a video (converted to .webp.mp4), use the MP4
        file_source = img
        if img.lower().endswith('.webp') and os.path.exists(os.path.join(file_dir, img + '.mp4')):
            file_source = img + '.mp4'
            is_video = True
        
        # Check if this is a converted WebP
        is_converted_webp = False
        if is_video and file_source.lower().endswith('.mp4') and img.lower().endswith('.webp'):
            is_converted_webp = True
            print(f"Marking as converted WebP in UI: {img}")
        
        # Check if original WebP is in archive
        in_archive = False
        if is_converted_webp:
            archive_path = os.path.join(archive_dir, img)
            in_archive = os.path.exists(archive_path)
        
        image_data.append({
            'filename': img,
            'thumbnail': thumb,
            'type': mime_type or ('video/mp4' if is_video else 'image/jpeg'),
            'size': size_text,
            'is_video': is_video,
            'source': file_source,
            'is_converted_webp': is_converted_webp,
            'in_archive': in_archive
        })

    # Queue thumbnails for background generation
    if current_images:
        for img in current_images:
            if img not in thumbnail_queue and not thumbnail_exists(img):
                thumbnail_queue.append(img)
        
        # Start the background processor if needed
        if thumbnail_queue and not is_processing:
            start_thumbnail_processor()
    
    # Get counts for tab display based on our classification
    image_count = len([f for f in all_files if 
                     (f.lower().endswith(('.jpg', '.png', '.jpeg', '.gif', '.bmp')) or 
                      (f.lower().endswith('.webp') and f not in animated_webps)) and 
                     not f.lower().endswith('.webp.mp4')])
                     
    video_count = len([f for f in all_files if 
                     f.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5')) or 
                     f.lower().endswith('.webp.mp4') or 
                     f in animated_webps])
    
    # Make response with cache headers for 5 minutes
    response = make_response(render_template('index.html', 
                                           images=image_data,
                                           total_pages=total_pages, 
                                           page=page,
                                           theme=theme,
                                           file_type=file_type,
                                           image_count=image_count,
                                           video_count=video_count,
                                           total_count=len(all_files)))
    
    # Set a shorter cache time for the main page (5 minutes)
    response.headers['Cache-Control'] = 'public, max-age=300'
    response.headers['Expires'] = (datetime.utcnow() + 
                                 timedelta(seconds=300)).strftime('%a, %d %b %Y %H:%M:%S GMT')
    
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
    
@app.route('/check-file-status/<path:filename>')
def check_file_status(filename):
    """Check if a file exists, has been archived, or converted to MP4"""
    original_path = os.path.join(file_dir, filename)
    archive_path = os.path.join(archive_dir, filename)
    mp4_path = os.path.join(file_dir, filename + '.mp4')
    
    result = {
        "filename": filename,
        "exists_in_original": os.path.exists(original_path),
        "exists_in_archive": os.path.exists(archive_path),
        "exists_as_mp4": os.path.exists(mp4_path)
    }
    
    return jsonify(result)
    
@app.route('/conversion-progress/<path:filename>')
def conversion_progress(filename):
    """Get the progress of WebP to MP4 conversion"""
    try:
        print(f"Checking conversion progress for: {filename}")
        base_name = os.path.splitext(filename)[0]
        progress_path = os.path.join(thumbnail_dir, f"{os.path.basename(base_name)}_progress.json")
        print(f"Looking for progress file at: {progress_path}")
        
        if os.path.exists(progress_path):
            print(f"Found progress file: {progress_path}")
            with open(progress_path, 'r') as f:
                import json
                data = json.load(f)
                print(f"Progress data: {data}")
                return jsonify(data)
        else:
            # Check if MP4 already exists or WebP is in archive
            mp4_path = os.path.join(file_dir, filename + '.mp4')
            archive_path = os.path.join(archive_dir, filename)
            
            print(f"No progress file, checking if MP4 exists: {mp4_path}")
            print(f"Checking if file is in archive: {archive_path}")
            
            if os.path.exists(mp4_path) or os.path.exists(archive_path):
                print(f"File is completed: MP4 exists={os.path.exists(mp4_path)}, Archive exists={os.path.exists(archive_path)}")
                return jsonify({
                    "status": "completed", 
                    "progress": 100, 
                    "total": 100,
                    "filename": filename,
                    "archived": os.path.exists(archive_path)
                })
            
            # Check if file is in queue
            in_queue = any(filename == f for f in thumbnail_queue)
            print(f"File {'is' if in_queue else 'is not'} in thumbnail queue")
            
            return jsonify({
                "status": "not_started",
                "in_queue": in_queue,
                "filename": filename
            })
    except Exception as e:
        print(f"Error checking conversion progress: {str(e)}")
        return jsonify({
            "status": "error",
            "error": str(e),
            "filename": filename
        }), 500

def thumbnail_exists(filename):
    """Check if a thumbnail exists for the given file"""
    base_name = os.path.splitext(filename)[0]
    webp_path = os.path.join(thumbnail_dir, f"{base_name}_thumbnail.webp")
    png_path = os.path.join(thumbnail_dir, f"{base_name}_thumbnail.png")
    return os.path.exists(webp_path) or os.path.exists(png_path)

def convert_webp_to_mp4(file_path):
    """Convert a WebP animation file to MP4 video format"""
    print(f"convert_webp_to_mp4 called for: {file_path}")
    mp4_path = file_path + '.mp4'
    
    # Skip if the MP4 already exists
    if os.path.exists(mp4_path):
        print(f"MP4 already exists, skipping conversion: {mp4_path}")
        return mp4_path
        
    try:
        # Check if it's actually an animated WebP by trying to open it with PIL
        print(f"Checking if {file_path} is an animated WebP")
        with Image.open(file_path) as img:
            # If it has more than one frame, it's probably an animation
            is_animated = hasattr(img, 'n_frames') and img.n_frames > 1
            if not is_animated:
                # Not an animation, no need to convert
                print(f"Not an animated WebP, skipping conversion: {file_path}")
                return None
            else:
                print(f"Confirmed animated WebP with {img.n_frames} frames: {file_path}")

        # Convert WebP to MP4 using MoviePy
        # First create a temporary folder for the frames
        import tempfile
        import shutil
        from moviepy import ImageSequenceClip
        
        temp_dir = tempfile.mkdtemp()
        
        # Create a progress indicator file
        base_name = os.path.splitext(file_path)[0]
        progress_path = os.path.join(thumbnail_dir, f"{os.path.basename(base_name)}_progress.json")
        
        try:
            # Extract frames from WebP
            # We'll use PIL to extract frames and save them as PNG
            with Image.open(file_path) as img:
                frame_count = getattr(img, 'n_frames', 1)
                
                # Initialize progress tracking
                with open(progress_path, 'w') as f:
                    import json
                    json.dump({
                        "status": "extracting_frames", 
                        "progress": 0, 
                        "total": frame_count,
                        "filename": os.path.basename(file_path)
                    }, f)
                
                # Extract each frame
                frames = []
                for i in range(frame_count):
                    img.seek(i)
                    frame_path = os.path.join(temp_dir, f"frame_{i:04d}.png")
                    rgb_frame = img.convert('RGB')
                    rgb_frame.save(frame_path)
                    frames.append(frame_path)
                    
                    # Update progress every 5 frames or at first and last frame
                    if i == 0 or i == frame_count - 1 or i % 5 == 0:
                        with open(progress_path, 'w') as f:
                            json.dump({
                                "status": "extracting_frames", 
                                "progress": i + 1, 
                                "total": frame_count,
                                "filename": os.path.basename(file_path)
                            }, f)
                
                # Get the duration of each frame (default to 1/24 seconds if not available)
                try:
                    durations = []
                    for i in range(frame_count):
                        img.seek(i)
                        duration_ms = img.info.get('duration', 41)  # Default to ~24fps
                        durations.append(duration_ms / 1000.0)  # Convert to seconds
                except Exception:
                    durations = [1/24] * frame_count
                
                # Update progress status
                with open(progress_path, 'w') as f:
                    json.dump({
                        "status": "encoding_video", 
                        "progress": 0, 
                        "total": 100,
                        "filename": os.path.basename(file_path)
                    }, f)
                
                # Create a callback for encoding progress
                def encoding_progress_callback(t):
                    try:
                        progress = int(t * 100)
                        print(f"Encoding progress for {os.path.basename(file_path)}: {progress}%")
                        with open(progress_path, 'w') as f:
                            json.dump({
                                "status": "encoding_video", 
                                "progress": progress, 
                                "total": 100,
                                "filename": os.path.basename(file_path)
                            }, f)
                    except Exception as e:
                        print(f"Error in progress callback: {str(e)}")
                
                # Create a video from the frames
                if frames:
                    try:
                        print(f"Creating video clip from {len(frames)} frames")
                        clip = ImageSequenceClip(frames, durations=durations)
                        print(f"Starting video encoding to {mp4_path}")
                        clip.write_videofile(mp4_path, codec='libx264', fps=24, 
                                           audio=False, logger=None)
                        print(f"Successfully encoded video to {mp4_path}")
                    except Exception as e:
                        print(f"Error during video encoding: {str(e)}")
                        raise
                    
                # Update progress to completed
                with open(progress_path, 'w') as f:
                    json.dump({
                        "status": "completed", 
                        "progress": 100, 
                        "total": 100,
                        "filename": os.path.basename(file_path)
                    }, f)
                
                # Archive the original WebP file (move it to archive directory)
                try:
                    import shutil
                    webp_filename = os.path.basename(file_path)
                    archive_path = os.path.join(archive_dir, webp_filename)
                    print(f"Moving WebP file to archive: {file_path} -> {archive_path}")
                    
                    # Only move if archive directory exists and is different from source
                    if os.path.exists(archive_dir) and os.path.dirname(file_path) != archive_dir:
                        shutil.move(file_path, archive_path)
                        print(f"Successfully archived WebP file to: {archive_path}")
                except Exception as e:
                    print(f"Error archiving WebP file: {str(e)}")
                    
                return mp4_path
                
        finally:
            # Clean up temporary directory
            shutil.rmtree(temp_dir, ignore_errors=True)
            
            # Try to remove the progress file when done
            try:
                if os.path.exists(progress_path):
                    os.remove(progress_path)
            except:
                pass
                
    except Exception as e:
        print(f"Error converting WebP to MP4: {str(e)}")
        
        # Update progress to error state
        try:
            base_name = os.path.splitext(file_path)[0]
            progress_path = os.path.join(thumbnail_dir, f"{os.path.basename(base_name)}_progress.json")
            
            with open(progress_path, 'w') as f:
                import json
                json.dump({
                    "status": "error", 
                    "error": str(e),
                    "filename": os.path.basename(file_path)
                }, f)
        except:
            pass
            
        return None

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
        if file.lower().endswith(('.jpg', '.jpeg', '.png', '.bmp', '.gif')):
            # Handle regular image files
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
        
        elif file.lower().endswith('.webp'):
            # Special handling for WebP: check if it's animated
            try:
                with Image.open(file_path) as webp_img:
                    # Check if it has animation frames
                    is_animated = hasattr(webp_img, 'n_frames') and webp_img.n_frames > 1
                    
                    if is_animated:
                        print(f"Starting conversion of animated WebP to MP4: {file_path}")
                        # Convert animated WebP to MP4 in the background
                        # We don't want to block thumbnail generation
                        convert_thread = threading.Thread(
                            target=convert_webp_to_mp4,
                            args=(file_path,)
                        )
                        convert_thread.daemon = True
                        convert_thread.start()
                        print(f"Conversion thread started for: {file_path}")
                        
                        # For animated WebP, extract the first frame for thumbnail
                        webp_img.seek(0)
                        # Create the thumbnail
                        max_size = 200
                        webp_img.thumbnail((max_size, max_size))
                        
                        # Save as WebP
                        webp_img.save(thumbnail_path, 'WEBP', quality=80)
                    else:
                        # Regular WebP image (non-animated)
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
            except Exception as webp_error:
                print(f"WebP processing error for {file}: {str(webp_error)}")
                # Fall back to regular image handling
                with Image.open(file_path) as original_image:
                    max_size = 200
                    original_image.thumbnail((max_size, max_size))
                    original_image.save(thumbnail_path, 'WEBP', quality=85)
                
        elif file.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm', '.mp5')):
            # Handle video files
            try:
                # Check if this is an MP4 that was converted from a WebP
                is_converted_webp = False
                if file.lower().endswith('.mp4'):
                    webp_name = file[:-4]  # Remove .mp4 extension
                    if webp_name.lower().endswith('.webp'):
                        print(f"Found converted WebP->MP4 file: {file}")
                        # Look for the WebP in the archive
                        archive_webp_path = os.path.join(archive_dir, webp_name)
                        if os.path.exists(archive_webp_path):
                            print(f"Original WebP file is in archive: {archive_webp_path}")
                        is_converted_webp = True
                
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
