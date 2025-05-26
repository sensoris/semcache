---
sidebar_position: 2
---

# Embedding Model Configuration

Understanding and configuring the embedding model that powers semcache's semantic similarity matching.

## Current Embedding Model

semcache uses **FastEmbed** with the AllMiniLML6V2 model for generating embeddings.

### Model Specifications

```
Model: sentence-transformers/all-MiniLM-L6-v2
- Dimensions: 384
- Max Sequence Length: 256 tokens
- Model Size: ~23MB
- Language: Primarily English
- Performance: ~50ms per embedding on CPU
```

### Why This Model?

**Advantages**:
- **Fast inference**: Optimized for speed
- **Good quality**: Excellent semantic understanding
- **Lightweight**: Small memory footprint
- **Proven**: Widely used in production systems

**Trade-offs**:
- **English-focused**: Limited multilingual support
- **Shorter context**: 256 token limit
- **CPU-only**: No GPU acceleration currently

## Embedding Process

### 1. Text Extraction
```rust
// Extract user message content
let user_message = request.messages
    .iter()
    .find(|m| m.role == "user")
    .map(|m| &m.content)
    .unwrap_or_default();
```

### 2. Embedding Generation
```rust
// Generate 384-dimensional vector
let embedding: Vec<f32> = embedding_service
    .generate_embedding(user_message)
    .await?;
```

### 3. Vector Normalization
```rust
// Normalize for cosine similarity
let normalized = normalize_vector(&embedding);
```

### 4. Similarity Search
```rust
// Search cache with FAISS
let similar_entries = faiss_index
    .search(&normalized, threshold: 0.9)
    .await?;
```

## Performance Characteristics

### Embedding Generation Time

| Input Length | Generation Time | Use Case |
|--------------|----------------|----------|
| 10 words | ~20ms | Short queries |
| 50 words | ~35ms | **Typical prompts** |
| 100 words | ~50ms | Long prompts |
| 200+ words | ~70ms | Very long prompts |

### Memory Usage

```
Per embedding: 384 floats × 4 bytes = 1.5KB
Per cached entry: ~1.5KB + response size

Example:
- 1,000 cached entries: ~1.5MB for embeddings
- Plus response storage: variable (typically 10-100KB each)
```

## Model Comparison

### Available Models (Future Support)

| Model | Dimensions | Speed | Quality | Languages | Size |
|-------|------------|-------|---------|-----------|------|
| **all-MiniLM-L6-v2** | 384 | Fast | Good | EN | 23MB |
| all-mpnet-base-v2 | 768 | Medium | Better | EN | 120MB |
| multilingual-e5-small | 384 | Fast | Good | Multi | 118MB |
| text-embedding-ada-002 | 1536 | API | Best | Multi | API |

### Choosing the Right Model

**For speed (current)**:
- all-MiniLM-L6-v2 ✅
- Best for high-throughput applications

**For quality (future)**:
- all-mpnet-base-v2
- Better semantic understanding, 2x slower

**For multilingual (future)**:
- multilingual-e5-small
- Support for 100+ languages

**For maximum quality (future)**:
- OpenAI API (text-embedding-ada-002)
- Requires API calls for embeddings

## Similarity Calculation

### Cosine Similarity
```
similarity = (a · b) / (||a|| × ||b||)

Where:
- a, b are normalized embedding vectors
- · is dot product
- ||·|| is vector magnitude (1.0 for normalized vectors)
```

## Text Preprocessing

### Current Processing
- **No preprocessing**: Raw user message used directly
- **Token limits**: Truncated at model's max length (256 tokens)
- **Encoding**: UTF-8 text passed to FastEmbed

### Future Preprocessing Options
```yaml
# Planned preprocessing configuration
embedding:
  preprocessing:
    lowercase: true
    remove_punctuation: false
    remove_stopwords: false
    max_length: 256
    truncation: "end"  # start, end, middle
```

## Language Support

### Current Support
- **Primary**: English
- **Limited**: Other Latin-script languages
- **Poor**: Non-Latin scripts (Arabic, Chinese, etc.)

### Example Similarity Scores
```
"What is AI?" vs "What is artificial intelligence?"
→ Similarity: ~0.92 ✅

"Qu'est-ce que l'IA?" vs "What is AI?"  
→ Similarity: ~0.75 (French, may miss cache)

"什么是人工智能?" vs "What is AI?"
→ Similarity: ~0.45 (Chinese, will miss cache)
```

### Future Multilingual Support
```yaml
embedding:
  model: "multilingual-e5-small"
  languages: ["en", "es", "fr", "de", "zh"]
```

## Custom Embedding Models

### Future Support for Custom Models

**Local models**:
```yaml
embedding:
  type: "local"
  model_path: "/path/to/custom/model"
  device: "cpu"  # or "cuda"
```

**API-based models**:
```yaml
embedding:
  type: "api"
  provider: "openai"
  model: "text-embedding-ada-002"
  api_key: "${OPENAI_API_KEY}"
```

**Hugging Face models**:
```yaml
embedding:
  type: "huggingface"
  model: "sentence-transformers/paraphrase-MiniLM-L6-v2"
  revision: "main"
```

## Troubleshooting

### Common Issues

**Slow embedding generation**:
- Monitor CPU usage during embedding generation
- Consider model with fewer parameters
- Check for memory swapping

**Poor similarity matching**:
- Verify language compatibility
- Check similarity threshold setting
- Test with known similar prompts

**Memory issues**:
- Each embedding uses 1.5KB
- Large caches require significant RAM
- Monitor total memory usage

### Debugging Tools

**Admin dashboard**:
```
http://localhost:8080/admin
```

**Manual similarity testing**:
```bash
# Test similarity between prompts (future CLI tool)
semcache similarity "What is AI?" "What is artificial intelligence?"
# Output: 0.92
```

## Performance Optimization

### CPU Optimization
- FastEmbed uses optimized ONNX runtime
- Benefits from modern CPU features (AVX, etc.)
- Multi-core systems can handle concurrent requests

### Memory Optimization
- Embeddings cached in memory
- Model loaded once at startup
- Automatic cleanup of old embeddings

### Future GPU Support
```yaml
embedding:
  device: "cuda"  # Use GPU acceleration
  batch_size: 32  # Process multiple embeddings together
```

## Model Updates

### Current Limitations
- Model fixed at compile time
- No runtime model switching
- Updates require application restart

### Future Flexibility
- Hot-swappable models
- A/B testing different models
- Gradual migration between models

## Next Steps

- [Cache Settings](./cache-settings.md) - Configure caching behavior
- [Monitoring](./monitoring.md) - Track embedding performance
- [Performance Tuning](../guides/performance-tuning.md) - Optimize similarity matching