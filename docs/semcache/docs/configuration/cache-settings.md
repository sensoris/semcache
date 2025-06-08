---
sidebar_position: 1
---

# Cache Settings

Configure Semcache's caching behavior to optimize performance for your workload.

## Current Configuration

Semcache currently uses a configuration file to specify runtime configurations.

### Default Settings
```yaml
# Recommended for production (current default values)
log_level: info  # Possible values: debug, info, warning, error, critical
similarity_threshold: 0.90  # Float value between 0 and 1
port: 8080
eviction_policy:
  policy_type: memory_limit_mb # or entry_limit
  value: 4096
```

These values are stored in [config.yaml](https://github.com/sensoris/semcache/blob/master/config.yaml), but can be overriden with a custom file if required.
```bash
docker run -v /path/to/your/config.yaml:/app/config.yaml semcache
```

## Similarity Threshold

### Current Behavior
- **Default**: 0.90 (0.8 cosine similarity required)
- **Range**: 0.0 to 1.0
- **Algorithm**: Cosine similarity


## Entry Limits

### Current Behavior
- **Default**: 4096mb cache size maximum
- **Eviction**: Least Recently Used (LRU)
- **Memory**: Automatic cleanup when memory pressure detected


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


## Storage Configuration

### Current Storage
- **Type**: In-memory only
- **Persistence**: None (lost on restart)
- **Backup**: Not supported


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


### Development configuration
```yaml
# Recommended for development
log_level: debug  # Possible values: debug, info, warning, error, critical
similarity_threshold: 0.90  # Float value between 0 and 1
port: 8080
eviction_policy:
  policy_type: entry_limit # or memory_limit_mb
  value: 100
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

