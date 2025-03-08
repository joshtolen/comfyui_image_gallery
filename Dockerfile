FROM python:3.12-slim

# Set the working directory in the container
WORKDIR /app

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Create directories for images and thumbnails
RUN mkdir -p /app/static/images/output /app/static/thumbnails /app/static/images/output/archive

# Copy requirements to the container
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy application files
COPY app.py /app/
COPY static/ /app/static/
COPY templates/ /app/templates/

# Set proper permissions for the app directories
RUN chmod -R 755 /app

# Expose the port the app runs on
EXPOSE 9999

# Set startup command
CMD ["python", "app.py"]