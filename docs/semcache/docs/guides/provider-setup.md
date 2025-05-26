---
sidebar_position: 2
---

# Provider Setup Guide

Detailed configuration for each supported LLM provider.

## OpenAI Setup

### API Key Setup
1. Visit [OpenAI API Keys](https://platform.openai.com/api-keys)
2. Create a new API key
3. Copy the key (starts with `sk-proj-` or `sk-`)

### Environment Setup
```bash
export OPENAI_API_KEY="sk-proj-your-key-here"
```

### Complete Configuration
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {"role": "user", "content": "Hello from semcache!"}
    ],
    "temperature": 0.7,
    "max_tokens": 1000
  }'
```

### Supported Models
- `gpt-4o` - Latest GPT-4 Omni
- `gpt-4o-mini` - Faster, cheaper GPT-4
- `gpt-4-turbo` - Previous generation GPT-4
- `gpt-3.5-turbo` - Fast and affordable
- All other OpenAI chat models

### Organization Support
```bash
# Include organization header
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "OpenAI-Organization: org-your-org-id" \
  -H "host: api.openai.com" \
  # ... rest of request
```

## DeepSeek Setup

### API Key Setup
1. Visit [DeepSeek Platform](https://platform.deepseek.com/)
2. Create account and generate API key
3. Copy the key (starts with `sk-`)

### Environment Setup
```bash
export DEEPSEEK_API_KEY="sk-your-deepseek-key"
```

### Complete Configuration
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "host: api.deepseek.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions" \
  -d '{
    "model": "deepseek-chat",
    "messages": [
      {"role": "user", "content": "Explain quantum computing"}
    ],
    "temperature": 0.7,
    "max_tokens": 2000
  }'
```

### Supported Models
- `deepseek-chat` - Main chat model
- `deepseek-coder` - Code-focused model
- Check DeepSeek documentation for latest models

## Anthropic Setup (Future)

:::note
Anthropic support is planned but not yet implemented. The API format differs from OpenAI's chat completions.
:::

### Planned Configuration
```bash
# Future Anthropic support
export ANTHROPIC_API_KEY="sk-ant-api03-your-key"

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -H "host: api.anthropic.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.anthropic.com/v1/messages" \
  -d '{
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 1024,
    "messages": [
      {"role": "user", "content": "Hello Claude!"}
    ]
  }'
```

## Azure OpenAI Setup (Future)

:::note
Azure OpenAI support is planned for a future release.
:::

### Planned Configuration
```bash
# Future Azure OpenAI support
export AZURE_OPENAI_KEY="your-azure-key"
export AZURE_ENDPOINT="https://your-resource.openai.azure.com"
export AZURE_DEPLOYMENT="your-deployment-name"

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $AZURE_OPENAI_KEY" \
  -H "host: your-resource.openai.azure.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: $AZURE_ENDPOINT/openai/deployments/$AZURE_DEPLOYMENT/chat/completions?api-version=2024-02-01" \
  -d '{
    "messages": [
      {"role": "user", "content": "Hello from Azure!"}
    ]
  }'
```

## Generic OpenAI-Compatible Providers

### Groq
```bash
export GROQ_API_KEY="gsk-your-groq-key"

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $GROQ_API_KEY" \
  -H "host: api.groq.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.groq.com/openai/v1/chat/completions" \
  -d '{
    "model": "llama-3.1-70b-versatile",
    "messages": [
      {"role": "user", "content": "Hello from Groq!"}
    ]
  }'
```

### Together AI
```bash
export TOGETHER_API_KEY="your-together-key"

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $TOGETHER_API_KEY" \
  -H "host: api.together.xyz" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.together.xyz/v1/chat/completions" \
  -d '{
    "model": "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
    "messages": [
      {"role": "user", "content": "Hello from Together!"}
    ]
  }'
```

### Local Models (Ollama)

If running Ollama with OpenAI API compatibility:

```bash
# Start Ollama with OpenAI API compatibility
ollama serve --openai-api

curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer ollama" \
  -H "host: localhost:11434" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: http://localhost:11434/v1/chat/completions" \
  -d '{
    "model": "llama3.1",
    "messages": [
      {"role": "user", "content": "Hello from local Ollama!"}
    ]
  }'
```

## Multi-Provider Configuration

### Environment Variables File
Create a `.env` file for easy provider switching:

```bash
# .env file
OPENAI_API_KEY="sk-proj-your-openai-key"
DEEPSEEK_API_KEY="sk-your-deepseek-key"
GROQ_API_KEY="gsk-your-groq-key"
TOGETHER_API_KEY="your-together-key"
```

### Provider Switching Script
```bash
#!/bin/bash
# switch-provider.sh

case $1 in
  "openai")
    export API_KEY=$OPENAI_API_KEY
    export HOST="api.openai.com"
    export UPSTREAM="https://api.openai.com/v1/chat/completions"
    export MODEL="gpt-4o-mini"
    ;;
  "deepseek")
    export API_KEY=$DEEPSEEK_API_KEY
    export HOST="api.deepseek.com"
    export UPSTREAM="https://api.deepseek.com/v1/chat/completions"
    export MODEL="deepseek-chat"
    ;;
  "groq")
    export API_KEY=$GROQ_API_KEY
    export HOST="api.groq.com"
    export UPSTREAM="https://api.groq.com/openai/v1/chat/completions"
    export MODEL="llama-3.1-70b-versatile"
    ;;
  *)
    echo "Usage: $0 {openai|deepseek|groq}"
    exit 1
    ;;
esac

echo "Configured for $1"
echo "Host: $HOST"
echo "Model: $MODEL"
```

Usage:
```bash
source switch-provider.sh openai
# Send request with configured provider
```

## Testing Provider Configuration

### Validation Script
```bash
#!/bin/bash
# test-provider.sh

PROVIDER=$1
API_KEY=$2
HOST=$3
UPSTREAM=$4
MODEL=$5

echo "Testing $PROVIDER configuration..."

response=$(curl -s -w "%{http_code}" -o response.json \
  http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $API_KEY" \
  -H "host: $HOST" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: $UPSTREAM" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [{\"role\": \"user\", \"content\": \"Test message\"}]
  }")

if [ "$response" = "200" ]; then
  echo "✅ $PROVIDER configured successfully"
  cat response.json | jq '.choices[0].message.content'
else
  echo "❌ $PROVIDER configuration failed (HTTP $response)"
  cat response.json
fi

rm -f response.json
```

### Test All Providers
```bash
# Test OpenAI
./test-provider.sh "OpenAI" "$OPENAI_API_KEY" "api.openai.com" \
  "https://api.openai.com/v1/chat/completions" "gpt-4o-mini"

# Test DeepSeek
./test-provider.sh "DeepSeek" "$DEEPSEEK_API_KEY" "api.deepseek.com" \
  "https://api.deepseek.com/v1/chat/completions" "deepseek-chat"
```

## Rate Limiting and Quotas

### Provider Limits
- **OpenAI**: Varies by tier (RPM/TPM limits)
- **DeepSeek**: Check platform documentation
- **Groq**: High rate limits, check current quotas
- **Together**: Model-specific limits

### Cache Benefits for Rate Limits
- **Cache hits don't count** against provider rate limits
- Effective rate limit multiplier based on cache hit rate
- Example: 50% hit rate = 2x effective rate limit

### Monitoring Rate Limits
```bash
# Check response headers for rate limit info
curl -v http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $API_KEY" \
  # ... headers
  # Look for: X-RateLimit-* headers in response
```

## Cost Optimization

### Cost Comparison
| Provider | Model | Input ($/1M tokens) | Output ($/1M tokens) |
|----------|-------|-------------------|---------------------|
| OpenAI | gpt-4o-mini | $0.15 | $0.60 |
| OpenAI | gpt-4o | $2.50 | $10.00 |
| DeepSeek | deepseek-chat | ~$0.07 | ~$0.28 |
| Groq | llama-3.1-70b | $0.59 | $0.79 |

### Cache Savings Calculator
```python
# Estimate cost savings from caching
def calculate_savings(requests_per_day, cache_hit_rate, cost_per_request):
    cached_requests = requests_per_day * cache_hit_rate
    daily_savings = cached_requests * cost_per_request
    monthly_savings = daily_savings * 30
    return monthly_savings

# Example: 1000 requests/day, 40% hit rate, $0.01/request
savings = calculate_savings(1000, 0.4, 0.01)
print(f"Monthly savings: ${savings:.2f}")  # $120/month
```

## Error Handling

### Common Provider Errors

**Invalid API Key**:
```json
{
  "error": {
    "message": "Invalid API key provided",
    "type": "invalid_request_error",
    "code": "invalid_api_key"
  }
}
```

**Rate Limit Exceeded**:
```json
{
  "error": {
    "message": "Rate limit exceeded",
    "type": "rate_limit_error"
  }
}
```

**Model Not Found**:
```json
{
  "error": {
    "message": "Model 'invalid-model' not found",
    "type": "invalid_request_error"
  }
}
```

### Error Handling Strategy
1. **Provider errors**: Forwarded directly to client
2. **Network errors**: Returned with connection details
3. **semcache errors**: Returned with debugging info

### Debugging Commands
```bash
# Test provider directly (bypass cache)
curl https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"test"}]}'

# Check semcache logs
docker logs semcache

# Verify headers
curl -v http://localhost:8080/chat/completions -H "..." | head -20
```

## Next Steps

- [Performance Tuning](./performance-tuning.md) - Optimize cache effectiveness
- [API Reference](../api/supported-providers.md) - Complete provider documentation
- [Monitoring](../monitoring/admin-dashboard.md) - Monitor provider performance
- [Configuration](../configuration/cache-settings.md) - Adjust cache settings per provider