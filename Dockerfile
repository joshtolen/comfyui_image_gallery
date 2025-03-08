FROM rust:1.81-slim as builder

# Set the working directory
WORKDIR /usr/src/comfyui_gallery

# Install build dependencies including FFmpeg development packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    ca-certificates \
    ffmpeg \
    libavformat-dev \
    libavfilter-dev \
    libavdevice-dev \
    libavcodec-dev \
    libavutil-dev \
    libswscale-dev \
    libswresample-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files to leverage Docker caching
COPY Cargo.toml .

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"Hello, world!\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/comfyui_image_gallery*

# Copy the real source code
COPY src/ src/
COPY templates/ templates/
COPY static/ static/

# Build the application
RUN cargo build --release

# Create the production image
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /app

# Install runtime dependencies (ffmpeg, python, and webp tools)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ffmpeg \
    ca-certificates \
    python3 \
    python3-pip \
    webp \  
    libwebp-dev \
    && rm -rf /var/lib/apt/lists/*

# Create directories for images and thumbnails
RUN mkdir -p /app/static/images/output /app/static/thumbnails /app/static/images/output/archive

# Copy the binary and static assets from the builder stage
COPY --from=builder /usr/src/comfyui_gallery/target/release/comfyui_image_gallery /app/
COPY --from=builder /usr/src/comfyui_gallery/templates/ /app/templates/
COPY --from=builder /usr/src/comfyui_gallery/static/ /app/static/

# Copy the WebP conversion Python script
COPY WebP_Animated_Extractor.py /app/
RUN chmod +x /app/WebP_Animated_Extractor.py

# Set proper permissions for the app directories
RUN chmod -R 755 /app

# Expose the port the app runs on
EXPOSE 9999

# Set environment variables
ENV RUST_LOG=info

# Set startup command
CMD ["./comfyui_image_gallery"]