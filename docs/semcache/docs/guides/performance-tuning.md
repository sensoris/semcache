---
sidebar_position: 3
---

# Performance Tuning

Optimize semcache for maximum effectiveness in your specific use case.

## Understanding Cache Performance

### Key Metrics
- **Cache Hit Rate**: Percentage of requests served from cache (target: 30-70%)
- **Memory Efficiency**: Memory used per cached entry (~1.5KB embeddings + response size)
- **Similarity Distribution**: How similar your queries are to each other

### Performance Trade-offs
- **Higher similarity threshold**: More precise but fewer cache hits
- **Lower similarity threshold**: More cache hits but potential false matches
- **Larger cache size**: Better hit rates but more memory usage
- **Smaller cache size**: Less memory but more evictions

## Optimizing Cache Hit Rates

### 1. Similarity Threshold Tuning

**Current default**: 0.9 (90% similarity)

**Tuning strategies**:
```bash
# Monitor similarity scores in admin dashboard
open http://localhost:8080/admin

# Look for "near misses" - queries with 0.85-0.89 similarity
# Consider if these should be cache hits for your use case
```

**Recommended thresholds by use case**:
- **FAQ/Support**: 0.85-0.9 (users ask similar questions differently)
- **Code assistance**: 0.9-0.95 (programming questions need precision)
- **Content generation**: 0.8-0.9 (creative variations acceptable)
- **Data analysis**: 0.95+ (precision critical)

### 2. Query Pattern Analysis

**Analyze your queries**:
```python
# Example analysis script
def analyze_query_patterns(queries):
    patterns = {
        'question_words': ['what', 'how', 'why', 'when', 'where'],
        'instruction_words': ['create', 'generate', 'write', 'explain'],
        'topics': {}
    }
    
    for query in queries:
        # Categorize query types
        query_lower = query.lower()
        for pattern in patterns['question_words']:
            if pattern in query_lower:
                # Track question patterns
                pass
    
    return patterns
```

**Common high-similarity patterns**:
- Rephrased questions: "What is X?" → "Can you explain X?"
- Different formality: "How do I..." → "How can I..."
- Synonym usage: "large" → "big", "error" → "mistake"

### 3. Cache Warming Strategies

**Identify common queries**:
```bash
# Pre-populate cache with frequent patterns
queries=(
  "What is machine learning?"
  "How does AI work?"
  "Explain neural networks"
  "What are the benefits of cloud computing?"
)

for query in "${queries[@]}"; do
  curl http://localhost:8080/chat/completions \
    -H "Authorization: Bearer $API_KEY" \
    -H "host: api.openai.com" \
    -H "Content-Type: application/json" \
    -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
    -d "{\"model\":\"gpt-4o-mini\",\"messages\":[{\"role\":\"user\",\"content\":\"$query\"}]}"
done
```

## Memory Optimization

### 1. Cache Size Planning

**Memory usage calculation**:
```
Per cached entry:
- Embedding: 384 floats × 4 bytes = 1.5KB
- Response: Variable (typically 1-50KB)
- Metadata: ~1KB
- Total: ~2.5KB + response size

Example for 1000 entries with 10KB avg responses:
1000 × (2.5KB + 10KB) = 12.5MB
```

**Capacity planning**:
| Daily Unique Queries | Recommended Cache Size | Memory Estimate |
|---------------------|----------------------|-----------------|
| 100 | 50-100 entries | 5-10MB |
| 1,000 | 200-500 entries | 20-50MB |
| 10,000 | 1,000-2,000 entries | 100-200MB |
| 100,000 | 5,000-10,000 entries | 500MB-1GB |

### 2. Eviction Strategy Optimization

**Current eviction**: LRU (Least Recently Used) when entry limit reached

**Future eviction strategies**:
- **TTL-based**: Expire entries after time period
- **Size-based**: Evict large responses first
- **Frequency-based**: Keep most frequently accessed
- **Semantic clustering**: Remove similar entries

### 3. Memory Monitoring

```bash
# Monitor memory usage
docker stats semcache

# Check cache metrics
curl http://localhost:8080/api/metrics | grep memory
```

## Latency Optimization

### 1. Embedding Generation Performance

**Current performance**: ~50ms per embedding on CPU

**Optimization techniques**:
- **Batch processing**: Process multiple requests together (future)
- **GPU acceleration**: Use CUDA for embedding generation (future)
- **Model optimization**: Smaller models for speed vs accuracy trade-off

**Concurrent request handling**:
```bash
# Test concurrent performance
for i in {1..10}; do
  curl http://localhost:8080/chat/completions \
    -H "Authorization: Bearer $API_KEY" \
    # ... headers
    -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Test '$i'"}]}' &
done
wait
```

### 2. Network Optimization

**Connection pooling** (automatic):
- HTTP client reuses connections to upstream providers
- Reduces connection establishment overhead
- Maintains persistent connections when possible

**Geographic optimization**:
- Deploy semcache close to upstream providers
- Use CDN/edge deployment for global applications
- Consider regional provider selection

### 3. FAISS Index Optimization

**Current index**: FlatIP (exhaustive search)

**Performance characteristics**:
- **Search time**: O(n) where n = number of cached entries
- **Memory usage**: Linear with cache size
- **Accuracy**: 100% (exhaustive search)

**Future optimizations**:
- **IVF indices**: Faster search for large caches
- **PQ compression**: Reduce memory usage
- **GPU indices**: Accelerated search on GPU

## Application-Specific Tuning

### 1. Chatbot Applications

**Characteristics**:
- High query repetition
- Similar user intents
- Acceptable response variation

**Optimizations**:
```yaml
# Recommended settings
cache:
  similarity_threshold: 0.85  # Higher hit rate
  max_entries: 1000           # Common questions
  eviction_policy: "frequency" # Keep popular answers
```

**Query preprocessing**:
- Normalize user input (lowercase, remove punctuation)
- Extract intent from conversational context
- Group similar conversation flows

### 2. Code Assistant Applications

**Characteristics**:
- Technical precision required
- Context-dependent queries
- Lower tolerance for incorrect matches

**Optimizations**:
```yaml
# Recommended settings
cache:
  similarity_threshold: 0.95  # High precision
  max_entries: 500            # Focused cache
  context_aware: true         # Consider code context
```

**Query enhancement**:
- Include programming language in context
- Cache by problem patterns, not exact code
- Consider function/method level caching

### 3. Content Generation

**Characteristics**:
- Creative variations acceptable
- Topic-based clustering
- High volume, diverse requests

**Optimizations**:
```yaml
# Recommended settings
cache:
  similarity_threshold: 0.8   # Accept variations
  max_entries: 2000          # Large cache
  topic_clustering: true     # Group by topic
```

## Monitoring and Analysis

### 1. Performance Dashboards

**Key metrics to track**:
```
Cache Performance:
- Hit rate over time
- Similarity score distribution
- Cache size growth
- Eviction frequency

Response Times:
- P50, P95, P99 latencies
- Cache hit vs miss latencies
- Embedding generation time
- Upstream provider latency

Resource Usage:
- Memory consumption
- CPU utilization
- Network I/O
- Storage usage (future)
```

### 2. A/B Testing Framework

**Testing similarity thresholds**:
```python
# Example A/B test setup
def test_similarity_thresholds():
    thresholds = [0.85, 0.9, 0.95]
    results = {}
    
    for threshold in thresholds:
        # Configure semcache with threshold
        # Run test queries
        # Measure hit rate, accuracy, user satisfaction
        results[threshold] = measure_performance()
    
    return analyze_results(results)
```

### 3. Cost Analysis

**Calculate ROI**:
```python
def calculate_cache_roi(
    requests_per_day: int,
    cache_hit_rate: float,
    cost_per_request: float,
    cache_infrastructure_cost: float
):
    daily_api_costs_without_cache = requests_per_day * cost_per_request
    daily_api_costs_with_cache = requests_per_day * (1 - cache_hit_rate) * cost_per_request
    daily_savings = daily_api_costs_without_cache - daily_api_costs_with_cache
    monthly_savings = daily_savings * 30
    monthly_infrastructure_cost = cache_infrastructure_cost
    
    net_monthly_savings = monthly_savings - monthly_infrastructure_cost
    roi_percentage = (net_monthly_savings / monthly_infrastructure_cost) * 100
    
    return {
        'monthly_savings': monthly_savings,
        'infrastructure_cost': monthly_infrastructure_cost,
        'net_savings': net_monthly_savings,
        'roi_percentage': roi_percentage
    }

# Example calculation
roi = calculate_cache_roi(
    requests_per_day=10000,
    cache_hit_rate=0.4,  # 40% hit rate
    cost_per_request=0.01,  # $0.01 per request
    cache_infrastructure_cost=50  # $50/month for hosting
)
print(f"Monthly ROI: {roi['roi_percentage']:.1f}%")
```

## Advanced Optimization Techniques

### 1. Query Preprocessing

**Text normalization**:
```python
def normalize_query(query: str) -> str:
    # Convert to lowercase
    query = query.lower()
    
    # Remove extra whitespace
    query = ' '.join(query.split())
    
    # Standardize punctuation
    query = query.replace('?', '').replace('!', '').replace('.', '')
    
    # Expand contractions
    contractions = {
        "what's": "what is",
        "how's": "how is",
        "can't": "cannot"
    }
    for contraction, expansion in contractions.items():
        query = query.replace(contraction, expansion)
    
    return query
```

### 2. Semantic Clustering

**Group similar queries** (future feature):
```python
# Conceptual clustering approach
def cluster_similar_queries(embeddings, threshold=0.85):
    clusters = []
    for embedding in embeddings:
        # Find most similar cluster
        best_cluster = find_most_similar_cluster(embedding, clusters, threshold)
        if best_cluster:
            best_cluster.add(embedding)
        else:
            clusters.append(create_new_cluster(embedding))
    return clusters
```

### 3. Multi-tier Caching

**Implement cache hierarchies** (future):
```yaml
# Conceptual multi-tier configuration
cache:
  tiers:
    - name: "hot"
      similarity_threshold: 0.95
      max_entries: 100
      ttl: "1h"
    - name: "warm"  
      similarity_threshold: 0.9
      max_entries: 1000
      ttl: "24h"
    - name: "cold"
      similarity_threshold: 0.85
      max_entries: 10000
      ttl: "7d"
```

## Troubleshooting Performance Issues

### 1. Low Cache Hit Rates

**Diagnosis**:
```bash
# Check similarity scores in admin dashboard
# Look for queries just below threshold

# Analyze query patterns
curl http://localhost:8080/api/metrics | grep cache_hit_rate
```

**Solutions**:
- Lower similarity threshold gradually
- Analyze failed matches in admin dashboard
- Improve query preprocessing
- Check for diverse vs repetitive query patterns

### 2. High Memory Usage

**Diagnosis**:
```bash
# Monitor memory growth
docker stats semcache

# Check cache size
curl http://localhost:8080/api/metrics | grep cached_entries
```

**Solutions**:
- Reduce cache entry limit
- Implement more aggressive eviction
- Analyze average response sizes
- Consider horizontal scaling

### 3. Slow Response Times

**Diagnosis**:
```bash
# Measure response times
time curl http://localhost:8080/chat/completions # ...

# Check embedding generation time
# Monitor upstream provider latency
```

**Solutions**:
- Optimize embedding model selection
- Improve network connectivity to providers
- Scale semcache horizontally
- Use faster embedding models

## Next Steps

- [Cache Settings](../configuration/cache-settings.md) - Detailed configuration options
- [Monitoring](../monitoring/metrics.md) - Set up performance monitoring
- [Admin Dashboard](../monitoring/admin-dashboard.md) - Use monitoring tools
- [Troubleshooting](../monitoring/troubleshooting.md) - Debug performance issues