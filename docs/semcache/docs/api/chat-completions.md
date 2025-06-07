---
sidebar_position: 1
---

# Chat Completions API

The main endpoint for caching LLM chat completion requests.

## Endpoint

```
POST /chat/completions
```

## Request Format

Semcache accepts the same request format as the OpenAI Chat Completions API:

```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "user",
      "content": "Your prompt here"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1000
}
```

## Required Headers

### Authentication
```
Authorization: Bearer YOUR_API_KEY
```
Your LLM provider's API key.

### Host Header
```
host: api.openai.com
```
The hostname of your target LLM provider:
- OpenAI: `api.openai.com`
- DeepSeek: `api.deepseek.com`
- Anthropic: `api.anthropic.com`

### Upstream URL
```
X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions
```
The complete upstream URL for the chat completions endpoint.

## Full Example

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
        "content": "Explain semantic caching in simple terms"
      }
    ],
    "temperature": 0.7,
    "max_tokens": 500
  }'
```

## Response Format

### Cache Hit (Instant Response)

When a semantically similar request is found in cache:

```json
{
  "id": "cached-chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Semantic caching stores responses based on meaning..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 45,
    "total_tokens": 57
  }
}
```

### Cache Miss (Forwarded to Upstream)

When no similar request is found, the response comes from the upstream LLM provider and is cached for future similar requests.

## Caching Behavior

### Similarity Threshold
- Default: **90%** similarity required for cache hit
- Uses cosine similarity on 384-dimensional embeddings
- Configurable in future releases

### Cache Key Generation
1. Extract user message content from the request
2. Generate embedding using FastEmbed (AllMiniLML6V2)
3. Normalize vector for cosine similarity
4. Search existing cache with FAISS

### What Gets Cached
- Complete response body from upstream LLM
- Request embedding vector
- Timestamp for LRU eviction
- Associated metadata

### Cache Eviction
- **Entry Limit**: Default 4 cached responses
- **Memory Limit**: Automatic cleanup when memory threshold reached  
- **Strategy**: Least Recently Used (LRU)

## Error Responses

### Missing Headers

```json
{
  "error": "Missing required header: X-LLM-Proxy-Upstream"
}
```

### Invalid Upstream

```json
{
  "error": "Failed to connect to upstream server"
}
```

### Malformed Request

```json
{
  "error": "Invalid JSON in request body"
}
```

## Performance Characteristics

### Cache Hit
- **Latency**: < 10ms typically
- **Cost**: $0 (no upstream API call)
- **Throughput**: High (memory-bound)

### Cache Miss
- **Latency**: Upstream latency + ~50ms (embedding generation)
- **Cost**: Standard upstream API pricing
- **Throughput**: Limited by upstream provider

## Limitations

### Current Limitations
- Only caches the first user message in a conversation
- No support for system messages in similarity matching
- Memory-only storage (no persistence across restarts)
- Fixed similarity threshold

### Planned Features
- Multi-turn conversation caching
- Configurable similarity thresholds
- Persistent storage options
- Advanced cache warming strategies

## Next Steps

- [Headers Reference](./headers.md) - Detailed header documentation
- [Supported Providers](./supported-providers.md) - Provider-specific setup
