---
sidebar_position: 2
---

# Getting Started

Get Semcache up and running as an HTTP proxy in a few minutes.

## Quick Start

Pull and run the Semcache Docker image:

```bash
docker run -p 8080:8080 ghcr.io/sensoris/semcache:latest
```

Semcache will start on `http://localhost:8080` and is ready to proxy LLM requests.

## Your First Cached Request

Semcache acts as a drop-in replacement for LLM APIs. Point your existing SDK to Semcache instead of the provider's endpoint.

```python
from openai import OpenAI
import os

# Point to Semcache instead of OpenAI directly
client = OpenAI(
    base_url="http://localhost:8080",  # Semcache endpoint
    api_key=os.getenv("OPENAI_API_KEY")  # Your OpenAI API key
)

# First request - cache miss, forwards to OpenAI
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "What is the capital of France?"}]
)
print(f"Response: {response.choices[0].message.content}")
```

This request will:
1. Go to Semcache first
2. Since it's not cached, Semcache forwards it to OpenAI
3. OpenAI responds with the answer
4. Semcache caches the response and returns it to you

## Testing Semantic Similarity

Now try a semantically similar but differently worded question:

```python
# Second request - semantically similar, should be a cache hit
response = client.chat.completions.create(
    model="gpt-4o", 
    messages=[{"role": "user", "content": "Tell me France's capital city"}]
)
print(f"Response: {response.choices[0].message.content}")
```

Even though the wording is different, Semcache recognizes the semantic similarity and returns the cached response instantly - no API call to OpenAI!

## Checking Cache Status

You can verify cache hits by checking the response headers using the OpenAI client. 
If there is a cache hit the `X-Cache-Status` header will be set to `hit`.

```python
# Use with_raw_response to access headers
response = client.chat.completions.with_raw_response.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "What's the capital of France?"}]
)

# Check if it was a cache hit
cache_status = response.headers.get("X-Cache-Status")
print(f"Cache status: {cache_status}")  # Should show "hit"

# Access the actual response content
completion = response.parse()
print(f"Response: {completion.choices[0].message.content}")
```

## Monitor Your Cache

Visit the built-in admin dashboard at `http://localhost:8080/admin` to monitor:

- **Cache hit rates** - See how effectively your cache is working
- **Memory usage** - Track resource consumption
- **Recent queries** - View cached prompts and responses
- **Performance metrics** - Response times and throughput

## Different Providers

Semcache works with multiple LLM providers out of the box:

```python
# Anthropic example
import anthropic

client = anthropic.Anthropic(
    base_url="http://localhost:8080",  # Point to Semcache
    api_key=os.getenv("ANTHROPIC_API_KEY")
)

response = client.messages.create(
    model="claude-3-sonnet-20240229",
    max_tokens=100,
    messages=[{"role": "user", "content": "Explain quantum computing"}]
)
```

The process is identical - Semcache automatically detects the provider based on the endpoint path.

## Next Steps

- **[LLM Providers & Tools](./llm-providers-tools.md)** - Configure additional providers like DeepSeek, Mistral, and custom LLMs
- **[Configuration](./configuration/cache-settings.md)** - Adjust similarity thresholds and cache behavior  
- **[Monitoring](./monitoring/metrics.md)** - Set up production monitoring with Prometheus and Grafana