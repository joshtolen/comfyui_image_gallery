# CLAUDE.md - Guidelines for ComfyUI Image Gallery

## Build & Run Commands
- Run server locally: `python app.py`
- Run with Docker: `docker build -t comfyui-gallery . && docker run -p 9999:9999 -v /path/to/images:/app/static/images/output comfyui-gallery`
- Install dependencies: `pip install -r requirements.txt`
- Configure symlink: `ln -s /path/to/your/images /path/to/gallery/static/images/output`

## Code Style Guidelines
- PEP 8 compliant Python code
- Use snake_case for variables and functions
- Use meaningful variable names (e.g., `file_dir` not `dir`)
- Add docstrings for functions
- Error handling with try/except blocks and meaningful error messages
- 4-space indentation
- CSS: Follow existing selector and property ordering patterns

## Import Order
1. Standard libraries (os, sys)
2. Third-party packages (Flask, PIL)
3. Local application imports

## Code Structure
- Flask routes at the top of app.py
- Helper functions below routes
- Constants defined at the top of files
- JSON responses for API endpoints

## Filesystem
- Images stored in `/app/static/images/output`
- Thumbnails stored in `/app/static/thumbnails`
- Static assets in `/static` directory
- HTML templates in `/templates` directory