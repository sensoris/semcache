---
sidebar_position: 3
---

# System Requirements

Hardware and software requirements for running semcache effectively.

## Minimum Requirements

### Hardware
- **CPU**: 2 cores, x86_64 or ARM64
- **RAM**: 2GB available memory
- **Storage**: 1GB free disk space
- **Network**: Internet access for LLM API calls

### Software
- **Docker**: 20.10+ (for container deployment)
- **Operating System**: 
  - Linux (Ubuntu 20.04+, CentOS 8+, Alpine 3.15+)
  - macOS 10.15+ (Intel or Apple Silicon)
  - Windows 10+ with WSL2

## Recommended Specifications

### Production Workloads
- **CPU**: 4+ cores for high concurrency
- **RAM**: 8GB+ for larger cache sizes
- **Storage**: SSD for better I/O performance
- **Network**: Low-latency connection to LLM providers

### Development
- **CPU**: 4+ cores for faster compilation
- **RAM**: 4GB+ for comfortable development
- **Storage**: 10GB+ for source code and dependencies

## Memory Usage Patterns

### Base Memory Usage
- **Application**: ~50MB base memory
- **FAISS Index**: ~10MB per 1000 cached embeddings
- **Response Cache**: Variable based on LLM response sizes

### Scaling Estimates

| Cached Responses | Estimated RAM Usage |
|------------------|---------------------|
| 100              | ~100MB              |
| 1,000            | ~250MB              |
| 10,000           | ~1.5GB              |
| 100,000          | ~12GB               |

:::note
These are rough estimates. Actual usage depends on:
- Average response length
- Embedding dimensions (currently 384)
- FAISS index overhead
:::

## Performance Considerations

### CPU Requirements

**Embedding Generation**:
- CPU-intensive operation for new requests
- Benefits from multiple cores for concurrent requests
- ARM64 processors supported with optimized BLAS

**Vector Search**:
- FAISS operations are CPU-bound
- Single-threaded per query but parallel across queries
- Benefits from modern CPU architectures

### Memory Requirements

**Cache Growth**:
- Linear growth with number of cached responses
- Each embedding: 384 floats × 4 bytes = 1.5KB
- Response storage varies by LLM output length

**Memory Management**:
- Automatic eviction when limits reached
- LRU strategy for cache cleanup
- Configurable entry limits (default: 4)

### Network Requirements

**Bandwidth**:
- Minimal for cache hits (< 1KB response)
- Standard LLM API bandwidth for cache misses
- No persistent connections maintained

**Latency**:
- Cache hits: < 10ms typically
- Cache misses: Upstream LLM latency + ~50ms processing
- Geographic proximity to LLM providers recommended

## Dependencies

### Runtime Dependencies

**Docker Deployment**:
- Docker Engine 20.10+
- No additional dependencies required

**Native Deployment**:
- **FAISS**: Vector similarity search library
- **OpenBLAS**: Optimized linear algebra operations
- **glibc**: Standard C library (Linux)

### Build Dependencies

**System Packages**:
```bash
# Ubuntu/Debian
build-essential cmake libopenblas-dev pkg-config

# CentOS/RHEL
gcc gcc-c++ cmake openblas-devel pkgconfig

# macOS
cmake openblas pkg-config
```

**Rust Toolchain**:
- Rust 1.70+ (stable channel)
- Cargo package manager
- rustc compiler

## Platform-Specific Notes

### Linux
- **Preferred platform** for production deployments
- Best performance with OpenBLAS optimizations
- Standard package managers supported

### macOS
- **Development-friendly** with Homebrew
- Apple Silicon (M1/M2) fully supported
- Performance comparable to Linux

### Windows
- **WSL2 recommended** for best compatibility
- Native Windows support planned
- Docker Desktop provides easiest setup

## Scaling Guidelines

### Horizontal Scaling
- Multiple semcache instances supported
- No shared state between instances
- Load balancer required for distribution

### Vertical Scaling
- Increase memory for larger cache sizes
- More CPU cores for higher concurrency
- SSD storage for faster I/O operations

### Cache Sizing

**Entry-based Limits**:
```rust
// Default configuration
entry_limit: 4,          // Maximum cached responses
similarity_threshold: 0.9 // 90% similarity required
```

**Memory-based Limits**:
- Automatic cleanup when memory pressure detected
- Configurable thresholds in future releases

## Monitoring Requirements

### Resource Monitoring
- **Memory usage**: Track cache growth
- **CPU utilization**: Monitor embedding generation load
- **Network I/O**: Track upstream API calls

### Application Metrics
- **Cache hit rate**: Measure effectiveness
- **Response latency**: Monitor performance
- **Error rates**: Track upstream failures

## Security Considerations

### Network Security
- API keys transmitted to upstream providers
- No persistent storage of sensitive data
- HTTPS recommended for production

### Resource Limits
- Memory exhaustion protection
- Request rate limiting (future feature)
- DoS protection through upstream providers

## Compatibility Matrix

| Component | Version | Status |
|-----------|---------|--------|
| Docker | 20.10+ | ✅ Supported |
| Docker | 19.x | ⚠️ Limited |
| Rust | 1.70+ | ✅ Required |
| Rust | 1.65-1.69 | ⚠️ May work |
| FAISS | 1.7+ | ✅ Required |
| OpenBLAS | 0.3+ | ✅ Recommended |

## Next Steps

- [Docker Installation](./docker.md) - Quick setup with containers
- [Local Setup](./local.md) - Build from source
- [Configuration](../configuration/cache-settings.md) - Optimize for your workload