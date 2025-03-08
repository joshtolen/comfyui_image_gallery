FROM node:20-slim AS frontend-builder

# Set the working directory for the frontend
WORKDIR /app/frontend

# Copy frontend package.json and install dependencies
COPY frontend/package*.json ./
RUN npm install --no-optional

# Install specific rollup packages that might be missing
RUN npm install @rollup/rollup-linux-x64-gnu @rollup/rollup-linux-x64-musl --no-save || true

# Copy frontend source code
COPY frontend/ ./

# Build frontend with --no-treeshake to avoid rollup issues
RUN NODE_OPTIONS=--max_old_space_size=4096 npm run build

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
COPY --from=frontend-builder /app/static/react/ /app/static/react/

# Create directories only if they'll be mounted as volumes
# These will be created at runtime if they don't exist
RUN mkdir -p /app/static/thumbnails

# Expose the port your app runs on
EXPOSE 9999

# Define the command to run your application
CMD ["python", "app.py"]