# ⚡ semcache

`semcache` is a semantic caching layer for your LLM applications. 

- [semcache website](https://semcache.io)
- [semcache docs](https://docs.semcache.io)

## Quick Start

Start the Semcache Docker image:

```bash
docker run -p 8080:8080 semcache/semcache:latest
```

Configure your application e.g with the OpenAI Python SDK:

```python
from openai import OpenAI

# Point to your Semcache host instead of OpenAI
client = OpenAI(base_url="http://localhost:8080", api_key="your-key")

# Cache miss - continues to OpenAI
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "What is the capital of France?"}]
)

# Cache hit - returns instantly 
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Tell me Fracnce's capital city"}]
)
```

Node.js follows a similar pattern of changing the base URL to point to your Semcache host:

```js
const OpenAI = require('openai');

// Point to your Semcache host instead of OpenAI
const openai = new OpenAI({baseURL: 'http://localhost:8080', apiKey: 'your-key'});
```

## Features

- **🧠 Completely in-memory** - Prompts, responses and the vector database are stored in-memory
- **🎯 Flexible by design** - Can work with your custom or private LLM APIs
- **🔌 Support for major LLM APIs** - OpenAI, Anthropic, Gemini, and more
- **⚡ HTTP proxy mode** - Drop-in replacement that reduces costs and latency
- **📈 Prometheus metrics** - Full observability out of the box
- **📊 Build-in dashboard** - Monitor cache performance at `/admin`
- **📤 Smart eviction** - LRU cache eviction policy


**Semcache is still in beta and being actively developed.**

## How it works

Semcache accelerates LLM applications by caching responses based on semantic similarity.

When you make a request Semcache first searches for previously cached answers to similar prompts and delivers them immediately. This eliminates redundant API calls, reducing both latency and costs.

Semcache also operates in a "cache-aside" mode, allowing you to load prompts and responses yourself.

## Example Integrations

For comprehensive provider configuration and detailed code examples, visit our [LLM Providers & Tools documentation](https://docs.semcache.io/docs/llm-providers-tools).

### HTTP Proxy

Point your existing SDK to Semcache instead of the provider's endpoint.

**OpenAI**
```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:8080", api_key="your-key")
```

**Anthropic**
```python
import anthropic

client = anthropic.Anthropic(
    base_url="http://localhost:8080",  # Semcache endpoint
    api_key="your-key"
)
```

**LangChain**
```python
from langchain.llms import OpenAI

llm = OpenAI(
    openai_api_base="http://localhost:8080",
    openai_api_key="your-key"
)
```

**LiteLLM**
```python
import litellm

litellm.api_base = "http://localhost:8080"
```
### Cache-aside

Install with:

```bash
pip install semcache
```

```python
from semcache import Semcache

# Initialize the client
client = Semcache(base_url="http://localhost:8080")

# Store a key-data pair
client.put("What is the capital of France?", "Paris")

# Retrieve data by semantic similarity
response = client.get("Tell me France's capital city.")
print(response)  # "Paris"
```

## Configuration

Configure via environment variables or `config.yaml`:

```yaml
log_level: info
port: 8080
```

Environment variables (prefix with `SEMCACHE_`):
```bash
SEMCACHE_PORT=8080
SEMCACHE_LOG_LEVEL=debug
```

## Monitoring

### Prometheus Metrics

Semcache emits comprehensive Prometheus metrics for production monitoring.

Check out our `/monitoring` directory for our custom Grafana dashboard.

### Built-in Dashboard

Access the admin dashboard at `/admin` to monitor cache performance.

## Enterprise

Our managed version of Semcache provides you with semantic caching as a service.

Features we offer:
- **Custom text embedding models** for your specific business 
- **Persistent storage** allowing you to build application memory over time 
- **In-depth analysis** of your LLM responses
- **SLA support** and dedicated engineering resources

Contact us at [contact@semcache.io](mailto:contact@semcache.io)

## Contributing

Interested in contributing? Contributions to Semcache are welcome! Feel free to make a PR.

---

Built with ❤️ in Rust • [Documentation](https://docs.semcache.io) • [GitHub Issues](https://github.com/sensoris/semcache/issues)
