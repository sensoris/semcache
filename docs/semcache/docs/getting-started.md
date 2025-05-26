---
sidebar_position: 2
---

# Getting Started

Get semcache up and running in minutes with Docker or build from source.

## Quick Start with Docker

The fastest way to try semcache is with Docker:

```bash
# Pull and run semcache
docker run -p 8080:8080 ghcr.io/sensoris/semcache:latest
```

semcache will start on `http://localhost:8080`. 

## Your First Request

Once semcache is running, you can start caching LLM requests. Here's how to send a request through semcache to OpenAI:

```bash
curl http://localhost:8080/chat/completions \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {
        "role": "user",
        "content": "What is the capital of France?"
      }
    ]
  }'
```

## How It Works

When you send this request:

1. **First time**: semcache forwards the request to OpenAI, caches the response, and returns it to you
2. **Similar requests**: If you ask a semantically similar question like "Tell me France's capital city", semcache returns the cached response instantly

## Required Headers

semcache requires these specific headers:

- `Authorization`: Your LLM provider API key (Bearer token)
- `host`: The target LLM provider hostname (e.g., `api.openai.com`)
- `X-LLM-Proxy-Upstream`: Full upstream URL for the completion endpoint

## Testing the Cache

Try sending a similar but different request:

```bash
curl http://localhost:8080/chat/completions \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {
        "role": "user",
        "content": "Tell me the capital city of France"
      }
    ]
  }'
```

If the similarity is above 90% (default threshold), you'll get the cached response instantly!

## Monitor Your Cache

Visit the admin dashboard at `http://localhost:8080/admin` to see:
- Cache hit/miss statistics
- Stored embeddings and responses
- Performance metrics

## Convenience Script

For easier testing, use the included Python script:

```bash
# Download the script
curl -O https://raw.githubusercontent.com/sensoris/semcache/master/scripts/request.py

# Send a request
python scripts/request.py openai $OPENAI_API_KEY "What is the capital of France?"
```

## Next Steps

- [Docker Installation](./installation/docker.md) - Complete Docker setup guide
- [Local Development](./installation/local.md) - Build from source
- [API Reference](./api/chat-completions.md) - Full API documentation
- [Provider Setup](./guides/provider-setup.md) - Configure different LLM providers