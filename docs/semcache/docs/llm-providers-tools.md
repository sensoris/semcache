# LLM Providers & Tools

Semcache as an HTTP proxy supports different LLM providers and tools. It also can be configured to work with your own custom APIs.

## Three Ways to Configure Providers

### 1. Preconfigured Default Routes

Semcache provides built-in routes for major LLM providers. Simply point your existing SDK to Semcache's base URL - no additional configuration needed. Each provider has a dedicated endpoint that automatically routes to the correct upstream API. See the [Providers](#providers) section below for specific examples.

### 2. Header-Based Provider Control

Use HTTP headers to override routing behavior while keeping existing API specifications:

#### `x-llm-proxy-host`
Override the upstream host while keeping the provider's API format.

#### `x-llm-proxy-upstream`
Override the complete upstream URL for custom endpoints.

#### `x-llm-prompt`
Specify where to find the prompt in your request body using [JSONPath syntax](https://jsonpath.com/):

- `$.messages[-1].content` - Last message content (OpenAI/Anthropic default)
- `$.input.text` - Custom field location
- `$.prompt` - Simple prompt field

#### Examples

**Using `x-llm-proxy-host` to route to DeepSeek:**
```python
from openai import OpenAI

client = OpenAI(
    api_key="your-deepseek-key",
    base_url="http://semcache-host-here:8080",  # Replace with your Semcache host
    default_headers={
        "x-llm-proxy-host": "https://api.deepseek.com"
    }
)
```

**Using `x-llm-proxy-upstream` and `x-llm-prompt` for custom LLMs:**
```bash
curl -X POST http://semcache-host-here:8080/semcache/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key" \
  -H "x-llm-proxy-upstream: https://your-custom-llm.com/api/v1/generate" \
  -H "x-llm-prompt: $.input.text" \
  -d '{
    "input": {
      "text": "What is the capital of France?",
      "max_tokens": 100
    }
  }'
```

### 3. Custom Generic Endpoint

For custom LLMs or providers we haven't implemented yet, use the generic endpoint `/semcache/v1/chat/completions` with appropriate headers:

```python
import requests

response = requests.post(
    "http://semcache-host-here:8080/semcache/v1/chat/completions",
    headers={
        "Authorization": "Bearer your-custom-api-key",
        "Content-Type": "application/json",
        "x-llm-proxy-upstream": "https://your-llm-api.com/v1/complete",
        "x-llm-prompt": "$.query"
    },
    json={
        "query": "Explain quantum computing",
        "temperature": 0.7,
        "max_tokens": 500
    }
)
```

## Available Routes

| Route | Provider | Purpose |
|-------|----------|---------|
| `/v1/chat/completions` | OpenAI | Default OpenAI format |
| `/chat/completions` | OpenAI | Alternative OpenAI format |
| `/v1/messages` | Anthropic | Anthropic Claude API |
| `/semcache/v1/chat/completions` | Generic | Custom providers |

## Providers

These are providers we have created a default endpoint for. **Remember you can configure any provider that uses HTTP with the [custom provider endpoint](#3-custom-generic-endpoint)**.

- [OpenAI](#openai)
- [Anthropic](#anthropic)
- [DeepSeek](#deepseek)
- [Mistral](#mistral)

### OpenAI

```python
from openai import OpenAI

client = OpenAI(
    api_key="your-openai-key",
    base_url="http://localhost:8080"  # Point to Semcache
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### Anthropic

```python
import anthropic

client = anthropic.Anthropic(
    api_key="your-anthropic-key",
    base_url="http://localhost:8080"  # Point to Semcache
)

response = client.messages.create(
    model="claude-3-sonnet-20240229",
    max_tokens=1000,
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### DeepSeek

```python
from openai import OpenAI

client = OpenAI(
    api_key="your-deepseek-key",
    base_url="http://semcache-host-here:8080",  # Replace with your Semcache host
    default_headers={
        "x-llm-proxy-host": "https://api.deepseek.com"
    }
)

response = client.chat.completions.create(
    model="deepseek-chat",
    messages=[{"role": "user", "content": "Write a Python function"}]
)
```

### Mistral

```python
from openai import OpenAI

client = OpenAI(
    api_key="your-mistral-key",
    base_url="http://semcache-host-here:8080",  # Replace with your Semcache host
    default_headers={
        "x-llm-proxy-host": "https://api.mistral.ai"
    }
)

response = client.chat.completions.create(
    model="mistral-large-latest",
    messages=[{"role": "user", "content": "Explain machine learning"}]
)
```

## Tools

The following examples show how to configure popular tools to use Semcache as an HTTP proxy.

### LiteLLM

```python
import litellm

# Configure LiteLLM to use Semcache as proxy
litellm.api_base = "http://semcache-host-here:8080"  # Replace with your Semcache host

# Use with different providers
response = litellm.completion(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello"}],
    headers={"x-llm-proxy-host": "https://api.deepseek.com"}
)
```

### LangChain

```python
from langchain_openai import ChatOpenAI

# Standard OpenAI through Semcache
llm = ChatOpenAI(
    model="gpt-4o",
    openai_api_base="http://semcache-host-here:8080",  # Replace with your Semcache host
    openai_api_key="your-openai-key"
)

# Custom provider through Semcache
llm_custom = ChatOpenAI(
    model="gpt-4o",
    openai_api_base="http://semcache-host-here:8080",  # Replace with your Semcache host
    openai_api_key="your-provider-key",
    default_headers={
        "x-llm-proxy-host": "https://api.your-provider.com"
    }
)

response = llm.invoke("What is semantic caching?")
```
