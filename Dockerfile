FROM python:3.12-alpine

# Set the working directory in the container
WORKDIR /app

# Copy requirements to the container
COPY requirements.txt .

# Install dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy the entire application code
COPY . /app

# Expose the port your app runs on
EXPOSE 9999
USER root
# Define the command to run your application
CMD ["python", "app.py"]