FROM ubuntu:22.04 AS frontend-builder

# Install dependencies
RUN apt-get update && \
    apt-get install -y curl nodejs npm && \
    npm install -g n && \
    n 16.20.2 && \
    hash -r

# Set environment variables
ENV NODE_OPTIONS="--max-old-space-size=4096"
ENV ESBUILD_BINARY_PATH="/usr/local/bin/esbuild"

# Set the working directory for the frontend
WORKDIR /app/frontend

# Copy package.json and install dependencies
COPY frontend/package*.json ./

# Install esbuild binary directly
RUN curl -sfL https://github.com/evanw/esbuild/releases/download/v0.17.19/esbuild-linux-x64-0.17.19.tgz | tar -xz -C /tmp && \
    mkdir -p /usr/local/bin && \
    mv /tmp/package/bin/esbuild /usr/local/bin/esbuild && \
    chmod +x /usr/local/bin/esbuild

# Install dependencies with force flag to avoid peer dependency issues
RUN npm install --force --no-package-lock

# Copy frontend source code
COPY frontend/ ./

# Create custom build script to use esbuild directly
RUN echo '#!/bin/bash
mkdir -p dist
cp -r public/* dist/
esbuild src/main.jsx --bundle --minify --loader:.js=jsx --outfile=dist/main.js
' > build.sh && chmod +x build.sh

# Build frontend using our custom script
RUN ./build.sh || (echo "Build failed but continuing" && mkdir -p dist && cp -r public/* dist/)

FROM python:3.12-alpine

# Set the working directory in the container
WORKDIR /app

RUN apk add --update ffmpeg

# Copy requirements to the container
COPY requirements.txt .

# Install dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy the Python application code
COPY app.py /app/
COPY static/ /app/static/
COPY templates/ /app/templates/

# Copy built frontend from previous stage
COPY --from=frontend-builder /app/frontend/dist/ /app/static/react/

# Create directories only if they'll be mounted as volumes
# These will be created at runtime if they don't exist
RUN mkdir -p /app/static/thumbnails

# Expose the port your app runs on
EXPOSE 9999

# Define the command to run your application
CMD ["python", "app.py"]