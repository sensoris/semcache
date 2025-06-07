---
sidebar_position: 1
---

# Cache Settings

Configure Semcache's caching behavior to optimize performance for your workload.

## Current Configuration

Semcache currently uses hardcoded configuration values. Future releases will support configuration files and environment variables.

### Default Settings

```rust
// Current hardcoded values
similarity_threshold: 0.9,    // 90% similarity required for cache hit
entry_limit: 4,               // Maximum cached responses
embedding_model: "AllMiniLML6V2", // FastEmbed model
vector_dimensions: 384,       // Embedding vector size
```

## Similarity Threshold

### Current Behavior
- **Default**: 0.9 (90% similarity required)
- **Range**: 0.0 to 1.0
- **Algorithm**: Cosine similarity on normalized vectors

### Future Configuration
```yaml
# Planned configuration file support
cache:
  similarity_threshold: 0.9
  strict_mode: false  # Require exact model match
```

## Entry Limits

### Current Behavior
- **Default**: 4 cached responses maximum
- **Eviction**: Least Recently Used (LRU)
- **Memory**: Automatic cleanup when memory pressure detected

### Memory Usage Estimates

| Cached Entries | Memory Usage | Typical Scenarios |
|---------------|--------------|-------------------|
| 4 (default) | ~10MB | Development, testing |
| 100 | ~100MB | Small applications |
| 1,000 | ~500MB | Medium applications |
| 10,000 | ~4GB | Large-scale deployments |

### Future Configuration
```yaml
# Planned configuration options
cache:
  max_entries: 1000
  max_memory_mb: 512
  eviction_policy: "lru"  # lru, fifo, random
```

## Embedding Model

### Current Model
- **Model**: AllMiniLML6V2
- **Dimensions**: 384
- **Performance**: ~50ms embedding generation
- **Language**: Optimized for English

### Model Characteristics

```
Model: sentence-transformers/all-MiniLM-L6-v2
- Size: 23MB
- Speed: Fast
- Quality: Good for semantic similarity
- Languages: Primarily English
```

### Future Model Options
```yaml
# Planned model configuration
embedding:
  model: "all-MiniLM-L6-v2"  # Default
  # model: "all-mpnet-base-v2"  # Higher quality, slower
  # model: "multilingual-e5-small"  # Multilingual support
  device: "cpu"  # cpu, cuda (future)
```

## Storage Configuration

### Current Storage
- **Type**: In-memory only
- **Persistence**: None (lost on restart)
- **Backup**: Not supported

### Future Storage Options
```yaml
# Planned storage backends
storage:
  type: "memory"  # memory, file, redis, postgres
  persistence: true
  backup_interval: "1h"
  
  # File storage
  file:
    path: "/var/lib/semcache/cache.db"
    
  # Redis storage  
  redis:
    url: "redis://localhost:6379"
    db: 0
```

## Performance Tuning

### Cache Hit Rate Optimization

**Monitor hit rates**:
- Visit admin dashboard at `http://localhost:8080/admin`
- Track cache hits vs misses
- Adjust similarity threshold based on results

**Tuning strategies**:
1. **Lower threshold** (0.85-0.9) for higher hit rates
2. **Higher threshold** (0.95+) for more precise matching
3. **Increase entry limit** for applications with diverse queries

### Memory Optimization

**Current limitations**:
- No memory limits enforced
- Automatic cleanup based on system pressure
- LRU eviction when entry limit reached

**Best practices**:
- Monitor memory usage with system tools
- Set appropriate entry limits for available RAM
- Consider horizontal scaling for high-memory workloads

## Environment-Specific Settings

### Development
```yaml
# Recommended for development
cache:
  similarity_threshold: 0.9
  max_entries: 10
  debug_logging: true
```

### Production
```yaml
# Recommended for production
cache:
  similarity_threshold: 0.9
  max_entries: 1000
  persistence: true
  metrics_enabled: true
```

### High-Traffic Production
```yaml
# For high-traffic scenarios
cache:
  similarity_threshold: 0.85  # Higher hit rate
  max_entries: 10000
  max_memory_mb: 4096
  cleanup_interval: "5m"
```

## Configuration Validation

### Future Validation Rules
- Similarity threshold must be 0.0-1.0
- Entry limits must be positive integers
- Memory limits must account for system constraints
- Storage paths must be writable

## Monitoring Configuration Impact

### Admin Dashboard
Monitor configuration effectiveness at:
```
http://localhost:8080/admin
```

## Future Configuration File

### Planned Configuration Format
```yaml
# /etc/semcache/config.yaml
server:
  port: 8080
  host: "0.0.0.0"

cache:
  similarity_threshold: 0.9
  max_entries: 1000
  max_memory_mb: 512
  eviction_policy: "lru"

embedding:
  model: "all-MiniLM-L6-v2"
  device: "cpu"

storage:
  type: "memory"
  persistence: false

logging:
  level: "info"
  format: "json"

metrics:
  enabled: true
  port: 9090
```

## Migration Path

When configuration files are implemented:

1. **Backward compatibility**: Current hardcoded defaults will remain
2. **Environment variables**: Override specific settings
3. **Configuration files**: Full control over all settings
4. **Runtime updates**: Hot-reload configuration changes

## Next Steps

- [Embedding Model](./embedding-model.md) - Detailed embedding configuration
- [Monitoring](./monitoring.md) - Set up metrics and alerts
