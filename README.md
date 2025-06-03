# ⚡ semcache

`semcache` is a semantic caching layer between you and your LLM. 

## Quick Start

```bash
docker run -p 8080:8080 -e SEMCACHE_PORT=8080 semcache:latest
```

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
    messages=[{"role": "user", "content": "What's France's capital city?"}]
)
```

## Features

- **🧠 Completely in-memory** - Prompts, responses and the vector database are stored in-memory
- **🎯 Flexible by design** - Can work with your custom or private LLM APIs
- **🔌 Support for major LLM APIs** - OpenAI, Anthropic, Gemini, and more
- **⚡ HTTP proxy mode** - Drop-in replacement that reduces costs and latency
- **🔄 Read-aside cache** - Directly put and get from the cache via an API
- **📈 Prometheus metrics** - Full observability out of the box
- **📊 Build-in dashboard** - Monitor cache performance at `/admin`

For more information and guides refer to our extensive docs: [docs.semcache.io](https://docs.semcache.io)

## How it works?

Semcache accelerates LLM applications by caching responses based on semantic similarity.

When you make a request Semcache first searches for previously cached answers to similar prompts and delivers them immediately. This eliminates redundant API calls, reducing both latency and costs.

Semcache also operates in a "read-aside" mode, allowing you to load prompts and responses yourself thus creating a knowledge base for your applications.

## Example Integrations

### HTTP Proxy

**LangChain**
```python
from langchain.llms import OpenAI

llm = OpenAI(
    openai_api_base="http://localhost:8080",
    openai_api_key="your-openai-key"
)
```

**LiteLLM**
```python
import litellm

# Point LiteLLM to Semcache instead of OpenAI directly
litellm.api_base = "http://localhost:8080"
```
### Read Aside Cache

```python
from semcache import Semcache

# Initialize Semcache with API key
cache = Semcache(api_key="your-api-key-here",
                 host="self-hosted-or-cloud-endpoint.com")

# Cache a LLM response
cache.put("What is the capital of France?",
          "Paris")

# Retrieve cached responses
assert "Paris" == cache.get("What's France's capital city called?")
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

Check out our `/monitoring` directory for Grafana dashboard setup examples.

### Built-in Dashboard
Access the admin dashboard at `http://localhost:8080/admin` to monitor:
- Cache hit rates
- Response times
- Memory usage
- Recent queries

## Enterprise

Our managed version of Semcache provides you with semantic caching as a service.

Features we offer:
- **Custom text embedding models** for your specific business 
- **Persistent storage** allowing you to build application memory over time 
- **In-depth analysis** of your LLM responses
- **SLA support** and dedicated engineering resources

Contact us at [contact@semcache.io](mailto:contact@semcache.io)

## Contributing

Interested in contributing? Contributions to semcache are welcome! Feel free to make a PR.

## Roadmap

See our [full roadmap](https://docs.semcache.io/roadmap) for upcoming features:

---

Built with ❤️ in Rust • [Documentation](https://docs.semcache.io) • [GitHub Issues](https://github.com/sensoris/semcache/issues)