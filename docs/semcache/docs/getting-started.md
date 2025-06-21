---
sidebar_position: 2
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Getting Started

Get Semcache up and running as an HTTP proxy in a few minutes.

## Quick Start

Pull and run the Semcache Docker image:

```bash
docker run -p 8080:8080 semcache/semcache:latest
```

Semcache will start on `http://localhost:8080` and is ready to proxy LLM requests.

## Setting up proxy client

Semcache acts as a drop-in replacement for LLM APIs. Point your existing SDK to Semcache instead of the provider's endpoint:

<Tabs groupId="llm-provider">
  <TabItem value="openai" label="OpenAI SDK" default>
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
  </TabItem>
  <TabItem value="anthropic" label="Anthropic SDK">
    ```python
    import anthropic
    import os

    # Point to Semcache instead of Anthropic directly
    client = anthropic.Anthropic(
        base_url="http://localhost:8080",  # Semcache endpoint
        api_key=os.getenv("ANTHROPIC_API_KEY")  # Your Anthropic API key
    )

    # First request - cache miss, forwards to Anthropic
    response = client.messages.create(
        model="claude-3-sonnet-20240229",
        max_tokens=100,
        messages=[{"role": "user", "content": "What is the capital of France?"}]
    )
    print(f"Response: {response.content[0].text}")
    ```
  </TabItem>
  <TabItem value="langchain" label="LangChain">
    ```python
    from langchain_openai import ChatOpenAI
    import os

    # Point to Semcache instead of OpenAI directly
    llm = ChatOpenAI(
        model="gpt-4o",
        openai_api_base="http://localhost:8080",  # Semcache endpoint
        openai_api_key=os.getenv("OPENAI_API_KEY")  # Your OpenAI API key
    )

    # First request - cache miss, forwards to OpenAI
    response = llm.invoke("What is the capital of France?")
    print(f"Response: {response.content}")
    ```
  </TabItem>
  <TabItem value="litellm" label="LiteLLM">
    ```python
    import litellm
    import os

    # Point LiteLLM to Semcache
    litellm.api_base = "http://localhost:8080"  # Semcache endpoint

    # First request - cache miss, forwards to OpenAI
    response = litellm.completion(
        model="gpt-4o",
        messages=[{"role": "user", "content": "What is the capital of France?"}],
        api_key=os.getenv("OPENAI_API_KEY")
    )
    print(f"Response: {response.choices[0].message.content}")
    ```
  </TabItem>
</Tabs>

This request will:
1. Go to Semcache first
2. Since it's not cached, Semcache forwards it to the upstream provider
3. The provider responds with the answer
4. Semcache caches the response and returns it to you

### Testing Semantic Similarity

Now try a semantically similar but differently worded question:

<Tabs groupId="llm-provider">
  <TabItem value="openai" label="OpenAI SDK" default>
    ```python
    # Second request - semantically similar, should be a cache hit
    response = client.chat.completions.create(
        model="gpt-4o", 
        messages=[{"role": "user", "content": "Tell me France's capital city"}]
    )
    print(f"Response: {response.choices[0].message.content}")
    ```
  </TabItem>
  <TabItem value="anthropic" label="Anthropic SDK">
    ```python
    # Second request - semantically similar, should be a cache hit
    response = client.messages.create(
        model="claude-3-sonnet-20240229",
        max_tokens=100,
        messages=[{"role": "user", "content": "Tell me France's capital city"}]
    )
    print(f"Response: {response.content[0].text}")
    ```
  </TabItem>
  <TabItem value="langchain" label="LangChain">
    ```python
    # Second request - semantically similar, should be a cache hit
    response = llm.invoke("Tell me France's capital city")
    print(f"Response: {response.content}")
    ```
  </TabItem>
  <TabItem value="litellm" label="LiteLLM">
    ```python
    # Second request - semantically similar, should be a cache hit
    response = litellm.completion(
        model="gpt-4o",
        messages=[{"role": "user", "content": "Tell me France's capital city"}],
        api_key=os.getenv("OPENAI_API_KEY")
    )
    print(f"Response: {response.choices[0].message.content}")
    ```
  </TabItem>
</Tabs>

Even though the wording is different, Semcache recognizes the semantic similarity and returns the cached response instantly - no API call to the upstream provider!

### Checking Cache Status

You can verify cache hits by checking the response headers. If there is a cache hit the `X-Cache-Status` header will be set to `hit`:

<Tabs groupId="llm-provider">
  <TabItem value="openai" label="OpenAI SDK" default>
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
  </TabItem>
  <TabItem value="anthropic" label="Anthropic SDK">
    ```python
    import httpx

    # Make request with httpx to access headers
    with httpx.Client() as client:
        response = client.post(
            "http://localhost:8080/v1/messages",
            headers={
                "Authorization": f"Bearer {os.getenv('ANTHROPIC_API_KEY')}",
                "Content-Type": "application/json",
                "anthropic-version": "2023-06-01"
            },
            json={
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 100,
                "messages": [{"role": "user", "content": "What's the capital of France?"}]
            }
        )
        
        # Check if it was a cache hit
        cache_status = response.headers.get("X-Cache-Status")
        print(f"Cache status: {cache_status}")  # Should show "hit"
        print(f"Response: {response.json()['content'][0]['text']}")
    ```
  </TabItem>
  <TabItem value="langchain" label="LangChain">
    ```python
    import requests

    # LangChain doesn't expose headers directly, use requests
    response = requests.post(
        "http://localhost:8080/v1/chat/completions",
        headers={
            "Authorization": f"Bearer {os.getenv('OPENAI_API_KEY')}",
            "Content-Type": "application/json"
        },
        json={
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "What's the capital of France?"}]
        }
    )
    
    # Check if it was a cache hit
    cache_status = response.headers.get("X-Cache-Status")
    print(f"Cache status: {cache_status}")  # Should show "hit"
    print(f"Response: {response.json()['choices'][0]['message']['content']}")
    ```
  </TabItem>
  <TabItem value="litellm" label="LiteLLM">
    ```python
    import requests

    # LiteLLM doesn't expose headers directly, use requests
    response = requests.post(
        "http://localhost:8080/v1/chat/completions",
        headers={
            "Authorization": f"Bearer {os.getenv('OPENAI_API_KEY')}",
            "Content-Type": "application/json"
        },
        json={
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "What's the capital of France?"}]
        }
    )
    
    # Check if it was a cache hit
    cache_status = response.headers.get("X-Cache-Status")
    print(f"Cache status: {cache_status}")  # Should show "hit"
    print(f"Response: {response.json()['choices'][0]['message']['content']}")
    ```
  </TabItem>
</Tabs>


## Setting up cache aside instance

<Tabs groupId="sdk">
  <TabItem value="python" label="Python" default>
    Install with
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
  </TabItem>
  <TabItem value="Node.js" label="Node.js">
    Install with
    ```bash
    npm install semcache
    ```
    ```javascript
    const SemcacheClient = require('semcache');
    
    const client = new SemcacheClient('http://localhost:8080');
    
    (async () => {
      await client.put('What is the capital of France?', 'Paris');
    
      const result = await client.get('What is the capital of France?');
      console.log(result); // => 'Paris'
    })();
    ```
  </TabItem>
</Tabs>

## Monitor Your Cache

Visit the built-in admin dashboard at `http://localhost:8080/admin` to monitor:

- **Cache hit rates** - See how effectively your cache is working
- **Memory usage** - Track resource consumption
- **Number of entries** - Monitor cache size and eviction

The process is identical across all providers - Semcache automatically detects the provider based on the endpoint path and forwards requests appropriately.

## Next Steps

- **[LLM Providers & Tools](./llm-providers-tools.md)** - Configure additional providers like DeepSeek, Mistral, and custom LLMs
- **[Configuration](./configuration/cache-settings.md)** - Adjust similarity thresholds and cache behavior  
- **[Monitoring](./monitoring/metrics.md)** - Set up production monitoring with Prometheus and Grafana
