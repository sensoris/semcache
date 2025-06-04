---
sidebar_position: 1
---

# Introduction to Semcache

**Semcache** is a semantic caching proxy for LLM API requests that intelligently caches responses based on semantic similarity.

## What is Semantic Caching?

Traditional caching systems only match exact requests. Semcache uses vector embeddings to understand the meaning behind prompts, allowing it to serve cached responses for semantically similar queries even when the wording differs.

For example, these prompts would be considered similar:
- "What is the capital of France?"
- "Tell me the capital city of France"
- "France's capital city is?"

## Key Benefits

- **Cost Reduction**: Avoid redundant API calls to expensive LLM providers
- **Performance**: Instant responses for semantically similar queries
- **Intelligent Matching**: Uses vector similarity (90% threshold by default)
- **Multi-Provider Support**: Works with OpenAI, DeepSeek, and Anthropic APIs
- **Drop-in Replacement**: Minimal changes to existing LLM integration code

## How It Works

1. **Request Interception**: semcache sits between your application and LLM providers
2. **Embedding Generation**: Converts prompts to 384-dimensional vectors using FastEmbed
3. **Similarity Search**: Uses FAISS to find cached responses above similarity threshold
4. **Smart Caching**: Returns cached response or forwards to upstream and caches new responses

## Architecture

```
Client → semcache → LLM Provider (OpenAI/DeepSeek/Anthropic)
         ↓
    Vector Cache (FAISS)
```

## Use Cases

- **Chatbots**: Cache common questions and variations
- **Content Generation**: Reuse similar content requests
- **Code Assistance**: Cache programming help for similar problems
- **Customer Support**: Serve cached answers for similar inquiries
- **Educational Tools**: Cache explanations for similar concepts

## Getting Started

Ready to reduce your LLM API costs and improve response times? Check out our [Getting Started Guide](./getting-started.md) to set up semcache in minutes.
