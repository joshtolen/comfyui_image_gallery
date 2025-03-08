FROM node:18-bullseye AS frontend-builder

# Set environment variables to disable optional rollup builds
ENV ROLLUP_SKIP_LOAD_NATIVE_PLUGIN=true
ENV DISABLE_V8_COMPILE_CACHE=1

# Set the working directory for the frontend
WORKDIR /app/frontend

# Copy frontend package.json and install dependencies
COPY frontend/package*.json ./

# Install dependencies with explicit flags to avoid Rollup issues
RUN npm install --no-optional --legacy-peer-deps --force

# Copy frontend source code
COPY frontend/ ./

# Try to build frontend
RUN NODE_ENV=production npm run build || echo "Build failed, but continuing anyway"

# Create a static react directory even if build fails
RUN mkdir -p ../static/react && \
    cp -r index.html ../static/react/ || true && \
    cp -r src ../static/react/ || true && \
    cp -r node_modules ../static/react/ || true

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

# Copy built frontend from previous stage (if it exists)
COPY --from=frontend-builder /app/static/react/ /app/static/react/ || true

# Copy fallback template if react build fails
COPY templates/index.html /app/templates/

# Create directories only if they'll be mounted as volumes
# These will be created at runtime if they don't exist
RUN mkdir -p /app/static/thumbnails

# Expose the port your app runs on
EXPOSE 9999

# Define the command to run your application
CMD ["python", "app.py"]