---
sidebar_position: 2
---

# Required Headers

semcache uses specific HTTP headers to route requests and authenticate with upstream LLM providers.

## Authentication Header

### Authorization
```
Authorization: Bearer YOUR_API_KEY
```

**Purpose**: Authenticate with the upstream LLM provider  
**Required**: Yes  
**Format**: Bearer token with your provider's API key

**Examples**:
```bash
# OpenAI
Authorization: Bearer sk-proj-abc123...

# DeepSeek  
Authorization: Bearer sk-def456...

# Anthropic
Authorization: Bearer sk-ant-api03-789...
```

## Routing Headers

### Host Header
```
host: api.openai.com
```

**Purpose**: Specify the target LLM provider hostname  
**Required**: Yes  
**Values**: Hostname only (no protocol or path)

**Supported hosts**:
- `api.openai.com` - OpenAI GPT models
- `api.deepseek.com` - DeepSeek models  
- `api.anthropic.com` - Anthropic Claude models

### Upstream URL Header
```
X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions
```

**Purpose**: Complete URL for the upstream chat completions endpoint  
**Required**: Yes  
**Format**: Full HTTPS URL with path

**Provider URLs**:
```bash
# OpenAI
X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions

# DeepSeek
X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions

# Anthropic  
X-LLM-Proxy-Upstream: https://api.anthropic.com/v1/messages
```

## Standard HTTP Headers

### Content-Type
```
Content-Type: application/json
```

**Purpose**: Specify request body format  
**Required**: Yes for POST requests  
**Value**: Always `application/json`

### Accept
```
Accept: application/json
```

**Purpose**: Specify expected response format  
**Required**: No (defaults to JSON)  
**Value**: `application/json`

## Complete Example

```bash
curl http://localhost:8080/chat/completions \
  -H "Authorization: Bearer sk-proj-abc123..." \
  -H "host: api.openai.com" \
  -H "Content-Type: application/json" \
  -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Header Validation

semcache validates all required headers before processing requests:

### Missing Header Errors
```json
{
  "error": "Missing required header: Authorization"
}
```

```json
{
  "error": "Missing required header: host"  
}
```

```json
{
  "error": "Missing required header: X-LLM-Proxy-Upstream"
}
```

### Invalid Header Errors
```json
{
  "error": "Invalid Authorization header format"
}
```

```json
{
  "error": "Invalid upstream URL format"
}
```

## Security Considerations

### API Key Handling
- API keys are **not logged** by semcache
- Keys are passed directly to upstream providers
- No persistent storage of credentials
- Use HTTPS in production to protect keys in transit

### Header Validation
- Upstream URLs are validated for proper format
- Only HTTPS URLs accepted for upstream connections
- Host headers validated against known providers

## Provider-Specific Notes

### OpenAI
```bash
# Standard OpenAI setup
-H "Authorization: Bearer sk-proj-..."
-H "host: api.openai.com"  
-H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions"
```

### DeepSeek
```bash
# DeepSeek setup
-H "Authorization: Bearer sk-..."
-H "host: api.deepseek.com"
-H "X-LLM-Proxy-Upstream: https://api.deepseek.com/v1/chat/completions"
```

### Anthropic
```bash
# Anthropic setup (future support)
-H "Authorization: Bearer sk-ant-api03-..."
-H "host: api.anthropic.com"
-H "X-LLM-Proxy-Upstream: https://api.anthropic.com/v1/messages"
```

:::note
Anthropic support is planned but not yet implemented in the current version.
:::

## Custom Headers

### Optional Headers
You can pass additional headers that will be forwarded to the upstream provider:

```bash
# Custom headers forwarded to upstream
-H "OpenAI-Organization: org-123"
-H "User-Agent: MyApp/1.0"
```

### Blocked Headers
Some headers are managed by semcache and cannot be overridden:
- `Host` (managed by semcache)
- `Content-Length` (calculated automatically)
- `Connection` (managed by HTTP client)

## Troubleshooting

### Authentication Issues
```bash
# Test with curl verbose mode
curl -v http://localhost:8080/chat/completions \
  -H "Authorization: Bearer YOUR_KEY" \
  ...

# Look for 401/403 responses from upstream
```

### Routing Issues
```bash
# Verify host header matches upstream URL
host: api.openai.com
X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions
#                              ^^^^^^^^^ must match
```

### Connection Issues
```bash
# Test upstream connectivity directly
curl https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"test"}]}'
```

## Next Steps

- [Supported Providers](./supported-providers.md) - Provider-specific configuration
- [Chat Completions](./chat-completions.md) - Complete API reference  
