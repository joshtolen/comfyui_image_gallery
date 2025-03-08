# ComfyUI Image Gallery (v3.0)

A modern, responsive image gallery specifically designed for browsing and managing images generated with ComfyUI. Version 3.0 is completely rewritten with React and Tailwind CSS for a more responsive and interactive user experience.

<img src="static/logo.png" width="50%" height="50%">

## Features

- 🖼️ Responsive image gallery with React and Tailwind CSS
- 📹 Improved video support (webm, mp4, animated WebP)
- 🌓 Dark/Light theme toggle
- ⭐ Favorites functionality
- 🔄 Animated WebP to MP4 conversion
- 📱 Mobile-friendly interface
- 🌅 Lazy loading of images
- 🔍 Enhanced lightbox with zoom and navigation
- 📂 Delete single or multiple files
- 📊 Improved sorting by creation time
- 🔄 Real-time conversion progress indicators

## Architecture

Version 3.0 uses a modern architecture:
- **Backend**: Flask API server
- **Frontend**: React with Tailwind CSS
- **State Management**: React Hooks
- **UI Components**: Headless UI components

## Installation and Setup

### Prerequisites

- **Python 3.9+**: For the Flask backend
- **Node.js 16+**: For building the React frontend
- **FFmpeg**: For WebP to MP4 conversion

### Quick Install

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/comfyui_image_gallery.git
   cd comfyui_image_gallery
   ```

2. Install Python dependencies:
   ```bash
   pip install -r requirements.txt
   ```

3. Build the frontend:
   ```bash
   cd frontend
   npm install
   npm run build
   ```

4. Set up a symlink to your ComfyUI output folder:
   ```bash
   ln -s /path/to/your/comfyui/output /path/to/gallery/static/images/output
   ```

### Running the Application

Start the Flask app:
```bash
python app.py
```

The gallery will be available at http://localhost:9999

### Running with Docker

```bash
# Build the Docker image
docker build -t comfyui-gallery .

# Run the container with your images directory mounted
docker run -p 9999:9999 \
  -v /path/to/your/comfyui/images:/app/static/images/output \
  -v /path/to/your/thumbnails:/app/static/thumbnails \
  -v /path/to/your/archive:/app/static/images/output/archive \
  comfyui-gallery
```

## Development

### Frontend Development

The React frontend is in the `frontend` directory. To start the development server:

```bash
cd frontend
npm run dev
```

This starts a hot-reloading development server that proxies API requests to the Flask backend.

### API Endpoints

The backend provides these main API endpoints:

- `GET /api/`: Get gallery data with pagination
- `POST /api/toggle-favorite`: Toggle favorite status for an image
- `POST /api/delete-file`: Delete a single file
- `POST /api/delete-files`: Delete multiple files
- `GET /api/conversion-progress/<filename>`: Check WebP conversion progress
- `GET /api/file-info/<filename>`: Get metadata for a file

## Screenshots

![Dark Theme](https://github.com/Smuzzies/comfyui_image_gallery/assets/110495122/eb8adc34-811e-434b-9ea7-d225f7cc63bb)
![Light Theme](https://github.com/Smuzzies/comfyui_image_gallery/assets/110495122/cf30e7ab-041d-4b9a-99c5-b6d863bb09f8)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
