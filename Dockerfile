FROM node:16-alpine AS frontend-builder

# Set environment variables to disable optional rollup builds
ENV VITE_CJS_IGNORE_WARNING=true
ENV VITE_CJS_TRACE_DEPRECATION=false
ENV DISABLE_V8_COMPILE_CACHE=1
ENV ROLLUP_WATCH=false
ENV NODE_OPTIONS=--max-old-space-size=4096

# Set the working directory for the frontend
WORKDIR /app/frontend

# Copy package.json and install dependencies
COPY frontend/package*.json ./

# Install dependencies with npm ci for reproducible builds
RUN npm ci

# Create a Rollup patch to skip the native plugin
RUN mkdir -p /tmp/rollup-patch && \
    echo 'module.exports = {};' > /tmp/rollup-patch/empty.js && \
    mkdir -p node_modules/rollup/dist && \
    cp /tmp/rollup-patch/empty.js node_modules/rollup/dist/native.js || true

# Copy frontend source code
COPY frontend/ ./

# Build frontend
RUN npm run build

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