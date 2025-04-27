# YAIRC Test IRC Server

This directory contains Docker configuration for setting up a simple InspIRCd server for testing IRC clients.

## Setup Instructions

### Prerequisites
- Docker
- Docker Compose

### Running the Server

1. Build and start the IRC server:
   ```
   docker-compose up -d
   ```

2. The server will be available at:
   - Host: 192.168.33.10 (or localhost)
   - Port: 6697

3. To stop the server:
   ```
   docker-compose down
   ```

## Configuration

- The InspIRCd configuration is in `inspircd.conf`
- If you make changes to the configuration, rebuild and restart the container:
  ```
  docker-compose down
  docker-compose up -d --build
  ```