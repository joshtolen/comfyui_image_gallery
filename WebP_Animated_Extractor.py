#!/usr/bin/env python3

import os
import subprocess
import tempfile
import shutil
import sys
from pathlib import Path

def extract_webp_frames(webp_path, output_dir=None):
    """
    Extract frames from animated WebP and create a WebM video.
    
    Args:
        webp_path (str): Path to the WebP file
        output_dir (str, optional): Directory to save the output. Defaults to the same directory as the WebP.
    
    Returns:
        str: Path to the output WebM file, or None if conversion failed
    """
    webp_path = Path(webp_path)
    
    # Validate input file
    if not webp_path.exists():
        print(f"Error: WebP file not found: {webp_path}")
        return None
    
    if not webp_path.suffix.lower() == '.webp':
        print(f"Error: Input file is not a WebP file: {webp_path}")
        return None
    
    # Determine output directory
    if output_dir is None:
        output_webm = webp_path.with_suffix('.webm')
    else:
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        output_webm = output_dir / f"{webp_path.stem}.webm"
    
    print(f"Converting {webp_path} to {output_webm}")
    
    # Create temporary directory for extracted frames
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_dir_path = Path(temp_dir)
        
        # Using webpmux to extract frames
        try:
            # First, check if webpmux is available
            subprocess.run(['which', 'webpmux'], check=True, capture_output=True, text=True)
            
            # Extract frames
            frame_info_output = subprocess.run(
                ['webpmux', '-info', str(webp_path)], 
                capture_output=True, 
                text=True, 
                check=True
            )
            
            # Parse frame information
            frame_count = 0
            for line in frame_info_output.stdout.splitlines():
                if line.startswith('Number of frames:'):
                    frame_count = int(line.split(':')[1].strip())
                    break
            
            if frame_count == 0:
                print(f"Error: No frames found in WebP file: {webp_path}")
                return None
            
            print(f"Found {frame_count} frames in WebP file")
            
            # Extract each frame
            for i in range(1, frame_count + 1):
                frame_path = temp_dir_path / f"frame_{i:04d}.webp"
                subprocess.run(
                    ['webpmux', '-get', f'frame', str(i), str(webp_path), '-o', str(frame_path)],
                    check=True,
                    capture_output=True
                )
            
            # Convert frames to PNG for better compatibility with ffmpeg
            for frame_path in sorted(temp_dir_path.glob('frame_*.webp')):
                png_path = frame_path.with_suffix('.png')
                subprocess.run(
                    ['dwebp', str(frame_path), '-o', str(png_path)],
                    check=True,
                    capture_output=True
                )
            
            # Create WebM with ffmpeg
            subprocess.run([
                'ffmpeg', '-y',
                '-framerate', '10',  # Adjust as needed
                '-pattern_type', 'glob', 
                '-i', str(temp_dir_path / 'frame_*.png'),
                '-c:v', 'libvpx',
                '-b:v', '1M',
                '-auto-alt-ref', '0',
                str(output_webm)
            ], check=True, capture_output=True)
            
            if output_webm.exists():
                print(f"Successfully created WebM: {output_webm}")
                # Copy file modification time from original file
                shutil.copystat(str(webp_path), str(output_webm))
                return str(output_webm)
            else:
                print(f"Error: Failed to create WebM file: {output_webm}")
                return None
                
        except subprocess.CalledProcessError as e:
            print(f"Error during conversion: {e}")
            return None
        except Exception as e:
            print(f"Unexpected error: {e}")
            return None

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python WebP_Animated_Extractor.py <webp_file> [output_dir]")
        sys.exit(1)
    
    webp_file = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else None
    
    result = extract_webp_frames(webp_file, output_dir)
    if result:
        sys.exit(0)
    else:
        sys.exit(1)