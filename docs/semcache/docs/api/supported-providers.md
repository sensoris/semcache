---
sidebar_position: 3
---

# Supported LLM Providers

Semcache works as a proxy for multiple LLM providers. Here's the current support status and configuration for each.

## OpenAI ✅

**Status**: Fully supported  
**Models**: All chat completion models (GPT-3.5, GPT-4, GPT-4o, etc.)

### Configuration
```bash
# Headers
Authorization: Bearer sk-proj-your-openai-key
host: api.openai.com
X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions
```

### Example Request
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {"role": "user", "content": "What is semantic caching?"}
    ]
  }'
```

### Supported Features
- ✅ Chat completions
- ✅ Streaming responses
- ✅ Function calling
- ✅ System messages
- ✅ Temperature and other parameters

## DeepSeek ✅

**Status**: Fully supported  
**Models**: DeepSeek chat models

### Configuration
```bash
# Headers
Authorization: Bearer sk-your-deepseek-key
host: api.deepseek.com
X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions
```

### Example Request
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "host: api.deepseek.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions" \
  -d '{
    "model": "deepseek-chat",
    "messages": [
      {"role": "user", "content": "Explain machine learning"}
    ]
  }'
```

### Supported Features
- ✅ Chat completions
- ✅ Streaming responses
- ✅ System messages
- ✅ Temperature and other parameters

## Anthropic 🚧

**Status**: Planned (not yet implemented)  
**Models**: Claude 3, Claude 3.5 Sonnet, etc.

### Planned Configuration
```bash
# Headers (future)
Authorization: Bearer sk-ant-api03-your-key
host: api.anthropic.com
X-LLM-Proxy-Upstream: https://api.anthropic.com/v1/messages
```

:::note
Anthropic support is planned for a future release. The API format differs slightly from OpenAI's chat completions format.
:::

## Azure OpenAI 🚧

**Status**: Planned  
**Models**: Azure-hosted OpenAI models

### Planned Configuration
```bash
# Headers (future)
Authorization: Bearer your-azure-key
host: your-resource.openai.azure.com
X-LLM-Proxy-Upstream: https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2024-02-01
```

## Other Providers

### Supported API Format
semcache can work with any provider that implements the OpenAI chat completions API format:

**Required**:
- POST endpoint accepting JSON
- OpenAI-compatible request/response format
- Bearer token authentication

**Compatible Providers**:
- Groq
- Together AI  
- Replicate (with chat completions API)
- Local models via LM Studio, Ollama (with OpenAI API compatibility)

### Generic Configuration
```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "host: your-provider-hostname.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://your-provider-hostname.com/v1/chat/completions" \
  -d '{
    "model": "provider-model-name",
    "messages": [
      {"role": "user", "content": "Your prompt"}
    ]
  }'
```

## Provider Comparison

| Provider | Status | Streaming | Function Calls | Notes |
|----------|--------|-----------|----------------|--------|
| OpenAI | ✅ | ✅ | ✅ | Full support |
| DeepSeek | ✅ | ✅ | ❓ | Testing needed |
| Anthropic | 🚧 | 🚧 | 🚧 | Planned |
| Azure OpenAI | 🚧 | 🚧 | 🚧 | Planned |
| Groq | ✅* | ✅* | ❓ | OpenAI-compatible |
| Together AI | ✅* | ✅* | ❓ | OpenAI-compatible |

*\* Should work but not officially tested*

## Request Format Compatibility

### OpenAI Format (Supported)
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000
}
```

### Anthropic Format (Future)
```json
{
  "model": "claude-3-sonnet-20240229",
  "max_tokens": 1000,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

## Authentication Methods

### Bearer Token (Supported)
```bash
Authorization: Bearer sk-your-api-key
```

### API Key Header (Future)
```bash
X-API-Key: your-api-key
```

### Custom Authentication (Future)
Support for provider-specific authentication methods.

## Error Handling

### Provider-Specific Errors
Semcache forwards upstream errors transparently:

**OpenAI Error**:
```json
{
  "error": {
    "message": "Invalid API key",
    "type": "invalid_request_error",
    "code": "invalid_api_key"
  }
}
```

**DeepSeek Error**:
```json
{
  "error": {
    "message": "Model not found",
    "type": "invalid_request_error"
  }
}
```

### Connection Errors
```json
{
  "error": "Failed to connect to upstream: Connection timeout"
}
```

## Rate Limiting

Semcache respects upstream provider rate limits:
- Forwards rate limit headers from providers
- No additional rate limiting by Semcache
- Cache hits don't count against provider limits

## Adding New Providers

To request support for a new provider:

1. **Check API Compatibility**: Does it support OpenAI chat completions format?
2. **Open GitHub Issue**: Request at [github.com/sensoris/semcache/issues](https://github.com/sensoris/semcache/issues)
3. **Provide Details**: API documentation, authentication method, endpoint URLs

### Contributing
Provider support contributions are welcome! See the development docs for implementation guidelines.

## Next Steps

- [Headers Reference](./headers.md) - Complete header documentation
- [Configuration](../configuration/cache-settings.md) - Optimize cache settings per provider