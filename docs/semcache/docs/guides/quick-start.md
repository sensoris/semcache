---
sidebar_position: 1
---

# Quick Start Guide

Get up and running with semcache in under 5 minutes.

## 1. Start semcache

### Using Docker (Recommended)
```bash
docker run -p 8080:8080 ghcr.io/sensoris/semcache:latest
```

### Using Local Build
```bash
# If you've built from source
./target/release/semcache
```

semcache will start on `http://localhost:8080`.

## 2. Verify Installation

```bash
# Basic health check
curl http://localhost:8080/
# Expected response: Hello, World!
```

## 3. Your First Cached Request

### Set Your API Key
```bash
export OPENAI_API_KEY="sk-proj-your-key-here"
```

### Send Initial Request
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      {
        "role": "user", 
        "content": "What is the capital of France?"
      }
    ]
  }'
```

This first request will:
- ✅ Forward to OpenAI
- ✅ Generate embedding for your prompt
- ✅ Cache the response
- ✅ Return OpenAI's response

## 4. Test Semantic Caching

Now try a similar but different question:

```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      {
        "role": "user",
        "content": "Tell me the capital city of France"
      }
    ]
  }'
```

If the similarity is above 90%, you'll get:
- ⚡ **Instant response** (cached)
- 💰 **No API cost** (no OpenAI call)
- 📊 **Cache hit** recorded

## 5. Monitor Your Cache

Visit the admin dashboard:
```bash
# Open in browser
open http://localhost:8080/admin
```

You'll see:
- Cache hit/miss statistics
- Stored embeddings and responses
- Memory usage
- Response times

## 6. Using the Python Script

For easier testing, download the convenience script:

```bash
# Download the script
curl -O https://raw.githubusercontent.com/sensoris/semcache/master/scripts/request.py

# Make it executable
chmod +x request.py

# Send a request
python request.py openai $OPENAI_API_KEY "What is machine learning?"
```

The script shows timing and handles the headers automatically.

## 7. Test Different Providers

### DeepSeek
```bash
export DEEPSEEK_API_KEY="sk-your-deepseek-key"

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "host: api.deepseek.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions" \
  -d '{
    "model": "deepseek-chat",
    "messages": [{"role": "user", "content": "Explain quantum computing"}]
  }'
```

### Other Providers
Any OpenAI-compatible API works:
```bash
# Generic provider
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "host: your-provider.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://your-provider.com/v1/chat/completions" \
  -d '{"model": "model-name", "messages": [...]}'
```

## Understanding Cache Behavior

### When Cache Hits Occur
- **Similarity ≥ 90%**: Cached response returned
- **Same meaning, different words**: Usually cached
- **Similar topics**: May be cached depending on wording

### When Cache Misses Occur
- **Similarity < 90%**: New request forwarded to upstream
- **Different topics**: Always forwarded
- **First-time questions**: Always forwarded (nothing to match)

### Example Similarities
```
"What is AI?" vs "What is artificial intelligence?"
→ Similarity: ~0.92 ✅ Cache hit

"What is AI?" vs "How does machine learning work?"
→ Similarity: ~0.75 ❌ Cache miss

"What is AI?" vs "What's the weather like?"
→ Similarity: ~0.15 ❌ Cache miss
```

## Performance Expectations

### Cache Hits
- **Latency**: < 10ms typically
- **Cost**: $0 (no upstream API call)
- **Throughput**: Very high (memory-bound)

### Cache Misses  
- **Latency**: Upstream latency + ~50ms (embedding)
- **Cost**: Standard upstream pricing
- **Throughput**: Limited by upstream provider

## Common Patterns

### Development Workflow
1. Send diverse test queries
2. Monitor cache hit rates in admin dashboard
3. Adjust similarity expectations
4. Iterate on prompt patterns

### Production Integration
1. Replace direct LLM API calls with semcache endpoint
2. Update headers to include upstream routing
3. Monitor cache effectiveness
4. Scale horizontally as needed

## Troubleshooting Quick Issues

### "Connection refused"
```bash
# Check if semcache is running
curl http://localhost:8080/
```

### "Missing required header"
```bash
# Ensure all required headers are present
curl -v http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $API_KEY" \
  -H "host: api.openai.com" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  # ... rest of request
```

### "Invalid API key"
```bash
# Test API key directly with provider
curl https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"test"}]}'
```

### No cache hits
- Try very similar prompts first
- Check admin dashboard for stored embeddings
- Verify questions are semantically similar
- Consider lowering similarity threshold (future feature)

## Next Steps

Now that you're up and running:

1. **[Provider Setup](./provider-setup.md)** - Configure multiple LLM providers
2. **[Performance Tuning](./performance-tuning.md)** - Optimize cache hit rates  
3. **[API Reference](../api/chat-completions.md)** - Learn the complete API
4. **[Docker Installation](../installation/docker.md)** - Production deployment guide
5. **[Monitoring](../monitoring/admin-dashboard.md)** - Set up comprehensive monitoring

## Example Applications

### Simple Node.js Integration
```javascript
// Replace your OpenAI calls with semcache
const response = await fetch('http://localhost:8080/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`,
    'host': 'api.openai.com',
    'Content-Type': 'application/json',
    'X-LLM-Proxy-Upstream': 'https://api.openai.com/v1/chat/completions'
  },
  body: JSON.stringify({
    model: 'gpt-4o-mini',
    messages: [{role: 'user', content: prompt}]
  })
});
```

### Python Integration
```python
import requests

def chat_with_cache(prompt, api_key):
    response = requests.post('http://localhost:8080/chat/completions', 
        headers={
            'Authorization': f'Bearer {api_key}',
            'host': 'api.openai.com',
            'Content-Type': 'application/json',
            'X-LLM-Proxy-Upstream': 'https://api.openai.com/v1/chat/completions'
        },
        json={
            'model': 'gpt-4o-mini',
            'messages': [{'role': 'user', 'content': prompt}]
        }
    )
    return response.json()
```

You're now ready to start building with semantic caching!