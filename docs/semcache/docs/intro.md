---
sidebar_position: 1
---

# Introduction to Semcache

**Semcache** is a semantic caching service for LLM APIs that intelligently caches responses based on semantic similarity.

Traditional caching systems only match exact requests. Semcache uses vector embeddings to understand the meaning behind prompts, allowing it to serve cached responses for semantically similar queries even when the wording differs.

For example, these prompts would be considered semantically similar:
- "What is the capital of France?"
- "Tell me the capital city of France"
- "France's capital city is?"

## Key Benefits

- **Token Usage Reduction**: Avoid redundant API calls to expensive LLM providers
- **Performance**: Instant responses for semantically similar queries
- **Intelligent Matching**: Uses vector similarity to match prompts
- **Multi-Provider Support**: Works with major providers like OpenAI, Anthropic, Deepseek etc.
- **Drop-in Replacement**: Minimal changes to existing LLM integration code

## How It Works

Semcache implements semantic caching through a multi-stage pipeline that processes LLM requests and matches them against previously cached responses using vector similarity.

### 1. Request Processing
Semcache intercepts HTTP requests to LLM providers and extracts the prompt content from the request payload. The prompt location is configurable via JSONPath (e.g., `$.messages[-1].content` for OpenAI format).

### 2. Semantic Processing  
To compare prompt similarity, Semcache computes vector embeddings using distilled models from the [model2vec library](https://github.com/MinishLab/model2vec). These models:
- Generate compact vector representations by computing word embeddings for each word in a sentence
- Take the mean of word vectors as the sentence-level representation
- Are optimized for low memory usage, enabling efficient in-memory processing

### 3. Vector Database Search
Semcache uses [FAISS](https://github.com/facebookresearch/faiss) (Facebook AI Similarity Search) as the in-memory vector database to:
- Store vector representations of previously seen prompts
- Perform fast similarity searches against the existing vector index
- Scale efficiently as the cache grows

### 4. Similarity Matching
The system calculates cosine similarity between the incoming prompt vector and stored vectors. If the closest match exceeds the configured similarity threshold, it's considered a cache hit.

### 5. Response Handling
- **Cache Hit**: Returns the stored response immediately with an `X-Cache-Status: hit` header
- **Cache Miss**: Forwards the request to the upstream LLM provider, caches the response with its vector representation, and returns the response to the client

## Use Cases

- **Chatbots**: Cache common questions and variations
- **Content Generation**: Reuse similar content requests
- **Code Assistance**: Cache programming help for similar problems
- **Customer Support**: Serve cached answers for similar inquiries
- **Educational Tools**: Cache explanations for similar concepts

## Getting Started

Ready to reduce your LLM API costs and improve response times? Check out our [Getting Started Guide](./getting-started.md) to set up semcache in minutes.
