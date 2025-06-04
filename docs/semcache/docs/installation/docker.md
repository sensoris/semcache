---
sidebar_position: 1
---

# Docker Installation

The recommended way to run semcache is using Docker, which handles all dependencies including FAISS.

## Prerequisites

- Docker installed on your system
- Port 8080 available (or use a different port mapping)

## Using Pre-built Image

The easiest way to get started:

```bash
# Run semcache with port mapping
docker run -p 8080:8080 ghcr.io/sensoris/semcache:latest
```

semcache will be available at `http://localhost:8080`.

## Custom Port

To run on a different port:

```bash
# Run on port 3000 instead
docker run -p 3000:8080 ghcr.io/sensoris/semcache:latest
```

## Environment Variables

Currently, semcache doesn't use environment variables for configuration. All configuration is done through the application state and request headers.

## Persistent Storage

By default, the cache is stored in memory and will be lost when the container stops. For persistent storage across restarts:

```bash
# Mount a volume for cache persistence (future feature)
docker run -p 8080:8080 \
  -v semcache_data:/app/data \
  ghcr.io/sensoris/semcache:latest
```

:::note
Persistent storage is planned for future releases. Currently, the cache is memory-only.
:::

## Building from Source

If you want to build the Docker image yourself:

```bash
# Clone the repository
git clone https://github.com/sensoris/semcache.git
cd semcache

# Build the Docker image
docker build -f docker/Dockerfile -t semcache .
```

The build process uses a multi-stage build with:
1. **FAISS base image**: Pre-built with FAISS dependencies
2. **Rust build stage**: Compiles the semcache binary
3. **Runtime stage**: Minimal Debian image with only necessary dependencies

## Docker Compose

For easier management, use Docker Compose:

```yaml
# docker-compose.yml
version: '3.8'
services:
  semcache:
    image: ghcr.io/sensoris/semcache:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
    # volumes:
    #   - semcache_data:/app/data  # Future persistent storage

# volumes:
#   semcache_data:  # Future persistent storage
```

Run with:

```bash
docker-compose up -d
```

## Health Check

Verify semcache is running:

```bash
# Simple health check
curl http://localhost:8080/

# Should return: Hello, World!
```

## Logs

View container logs:

```bash
# View logs
docker logs <container_id>

# Follow logs
docker logs -f <container_id>
```

## Troubleshooting

### Port Already in Use

If port 8080 is already in use:

```bash
# Check what's using the port
lsof -i :8080

# Use a different port
docker run -p 8081:8080 ghcr.io/sensoris/semcache:latest
```

### Container Won't Start

Check the logs for error messages:

```bash
docker logs <container_id>
```

Common issues:
- Insufficient memory (FAISS requires significant RAM)
- Port conflicts
- Docker daemon not running

## Next Steps

- [API Reference](../api/chat-completions.md) - Learn the API endpoints
- [Provider Setup](../guides/provider-setup.md) - Configure LLM providers
- [Monitoring](../monitoring/admin-dashboard.md) - Use the admin dashboard