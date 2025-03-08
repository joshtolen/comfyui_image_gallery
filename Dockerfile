FROM debian:bullseye-slim AS frontend-builder

# Install curl
RUN apt-get update && \
    apt-get install -y curl unzip

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# Set environment variables
ENV NODE_OPTIONS="--max-old-space-size=4096"

# Set the working directory for the frontend
WORKDIR /app/frontend

# Copy package.json file
COPY frontend/package*.json ./

# Install dependencies with Bun (much faster than npm)
RUN bun install --no-save

# Copy frontend source code
COPY frontend/ ./

# Create dist directory and copy static assets
RUN mkdir -p dist && cp -r public/* dist/

# Use our simplified main file without Tailwind imports
RUN cp src/main-simple.jsx src/main.jsx

# Build frontend using Bun with proper output directory
RUN bun build ./src/main.jsx --outdir=dist --minify

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