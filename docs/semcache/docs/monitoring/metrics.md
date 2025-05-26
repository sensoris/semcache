---
sidebar_position: 1
---

# Metrics Reference

Complete reference for semcache metrics and monitoring.

## Metrics Endpoint

**URL**: `http://localhost:8080/api/metrics`  
**Format**: Prometheus exposition format  
**Update Frequency**: Every 30 seconds  

```bash
# View raw metrics
curl http://localhost:8080/api/metrics

# View specific metrics
curl http://localhost:8080/api/metrics | grep cache_hits
```

## Cache Performance Metrics

### Cache Hit/Miss Counters

```prometheus
# Total cache hits since startup
semcache_cache_hits_total{provider="openai", model="gpt-4o"} 142

# Total cache misses since startup  
semcache_cache_misses_total{provider="openai", model="gpt-4o"} 58

# Cache hit rate (calculated)
rate(semcache_cache_hits_total[5m]) / 
(rate(semcache_cache_hits_total[5m]) + rate(semcache_cache_misses_total[5m])) * 100
```

**Labels**:
- `provider`: LLM provider (openai, deepseek, etc.)
- `model`: Model name (gpt-4o, deepseek-chat, etc.)

### Cache Size Metrics

```prometheus
# Current number of cached entries
semcache_cached_entries_total 45

# Current cache memory usage in bytes
semcache_cache_memory_bytes 12582912

# Cache evictions (when entries removed due to limits)
semcache_cache_evictions_total 8
```

### Similarity Metrics

```prometheus
# Similarity score distribution
semcache_similarity_score_bucket{le="0.5"} 12
semcache_similarity_score_bucket{le="0.8"} 45
semcache_similarity_score_bucket{le="0.9"} 67
semcache_similarity_score_bucket{le="0.95"} 78
semcache_similarity_score_bucket{le="1.0"} 89
semcache_similarity_score_sum 76.5
semcache_similarity_score_count 89
```

## Response Time Metrics

### Request Duration

```prometheus
# Request duration histogram
semcache_request_duration_seconds_bucket{le="0.01"} 45  # Cache hits
semcache_request_duration_seconds_bucket{le="0.1"} 67   # Fast responses
semcache_request_duration_seconds_bucket{le="0.5"} 89   # Normal responses
semcache_request_duration_seconds_bucket{le="1.0"} 95   # Slow responses
semcache_request_duration_seconds_bucket{le="5.0"} 100  # Very slow responses
semcache_request_duration_seconds_sum 45.6
semcache_request_duration_seconds_count 100
```

### Embedding Generation Time

```prometheus
# Time to generate embeddings
semcache_embedding_duration_seconds_bucket{le="0.01"} 0
semcache_embedding_duration_seconds_bucket{le="0.05"} 45
semcache_embedding_duration_seconds_bucket{le="0.1"} 89
semcache_embedding_duration_seconds_bucket{le="0.5"} 100
semcache_embedding_duration_seconds_sum 4.2
semcache_embedding_duration_seconds_count 55
```

### Upstream Response Time

```prometheus
# Time for upstream provider responses
semcache_upstream_duration_seconds_bucket{provider="openai",le="0.5"} 12
semcache_upstream_duration_seconds_bucket{provider="openai",le="1.0"} 34
semcache_upstream_duration_seconds_bucket{provider="openai",le="2.0"} 45
semcache_upstream_duration_seconds_bucket{provider="openai",le="5.0"} 55
semcache_upstream_duration_seconds_sum{provider="openai"} 67.8
semcache_upstream_duration_seconds_count{provider="openai"} 55
```

## System Resource Metrics

### Memory Usage

```prometheus
# Total memory used by semcache
semcache_memory_usage_bytes 134217728

# Memory breakdown by component
semcache_memory_embeddings_bytes 1572864   # Embedding storage
semcache_memory_responses_bytes 104857600  # Response cache
semcache_memory_faiss_bytes 26214400       # FAISS index
semcache_memory_other_bytes 1572864        # Other components
```

### CPU Usage

```prometheus
# CPU time spent on different operations
semcache_cpu_seconds_total{operation="embedding"} 45.6
semcache_cpu_seconds_total{operation="search"} 12.3
semcache_cpu_seconds_total{operation="http"} 8.9
```

## Request Counting Metrics

### Request Totals

```prometheus
# Total requests by endpoint
semcache_requests_total{endpoint="/chat/completions", method="POST"} 200
semcache_requests_total{endpoint="/admin", method="GET"} 15
semcache_requests_total{endpoint="/api/metrics", method="GET"} 48

# Requests by status code
semcache_requests_total{status="200"} 195
semcache_requests_total{status="400"} 3
semcache_requests_total{status="401"} 1
semcache_requests_total{status="500"} 1
```

### Provider-Specific Metrics

```prometheus
# Requests per provider
semcache_provider_requests_total{provider="openai"} 150
semcache_provider_requests_total{provider="deepseek"} 35
semcache_provider_requests_total{provider="groq"} 15

# Provider errors
semcache_provider_errors_total{provider="openai", error="rate_limit"} 2
semcache_provider_errors_total{provider="openai", error="timeout"} 1
```

## Error Metrics

### Error Counters

```prometheus
# Total errors by type
semcache_errors_total{type="upstream_connection"} 3
semcache_errors_total{type="invalid_request"} 5
semcache_errors_total{type="embedding_generation"} 1
semcache_errors_total{type="cache_operation"} 0

# Errors by provider
semcache_provider_errors_total{provider="openai", type="timeout"} 2
semcache_provider_errors_total{provider="deepseek", type="rate_limit"} 1
```

## Custom Business Metrics

### Cost Savings

```prometheus
# Estimated cost savings from cache hits
semcache_cost_savings_usd_total{provider="openai"} 12.45
semcache_cost_savings_usd_total{provider="deepseek"} 3.22

# Requests that would have cost money without cache
semcache_cached_requests_cost_usd{provider="openai"} 8.50
```

### Usage Analytics

```prometheus
# Unique users/sessions (if tracking enabled)
semcache_unique_users_total 45

# Popular models
semcache_model_usage_total{model="gpt-4o"} 89
semcache_model_usage_total{model="gpt-4o-mini"} 156
semcache_model_usage_total{model="deepseek-chat"} 34
```

## Prometheus Query Examples

### Cache Performance Queries

**Cache hit rate over time**:
```promql
rate(semcache_cache_hits_total[5m]) / 
(rate(semcache_cache_hits_total[5m]) + rate(semcache_cache_misses_total[5m])) * 100
```

**Average response time**:
```promql
rate(semcache_request_duration_seconds_sum[5m]) / 
rate(semcache_request_duration_seconds_count[5m])
```

**95th percentile response time**:
```promql
histogram_quantile(0.95, rate(semcache_request_duration_seconds_bucket[5m]))
```

### Resource Monitoring Queries

**Memory growth rate**:
```promql
rate(semcache_memory_usage_bytes[1h])
```

**Cache size trend**:
```promql
semcache_cached_entries_total
```

**Embedding generation performance**:
```promql
rate(semcache_embedding_duration_seconds_sum[5m]) / 
rate(semcache_embedding_duration_seconds_count[5m])
```

### Error Rate Queries

**Overall error rate**:
```promql
rate(semcache_errors_total[5m]) / 
rate(semcache_requests_total[5m]) * 100
```

**Provider-specific error rate**:
```promql
rate(semcache_provider_errors_total{provider="openai"}[5m]) / 
rate(semcache_provider_requests_total{provider="openai"}[5m]) * 100
```

## Grafana Dashboard Configuration

### Dashboard JSON

```json
{
  "dashboard": {
    "title": "semcache Monitoring",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "panels": [
      {
        "title": "Cache Hit Rate",
        "type": "stat",
        "targets": [{
          "expr": "rate(semcache_cache_hits_total[5m]) / (rate(semcache_cache_hits_total[5m]) + rate(semcache_cache_misses_total[5m])) * 100",
          "legendFormat": "Hit Rate %"
        }],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "min": 0,
            "max": 100
          }
        }
      },
      {
        "title": "Response Times",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(semcache_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          },
          {
            "expr": "histogram_quantile(0.95, rate(semcache_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

### Key Dashboard Panels

1. **Cache Hit Rate** - Single stat showing current hit rate
2. **Response Time Distribution** - Histogram of response times
3. **Memory Usage** - Time series of memory consumption
4. **Request Volume** - Requests per second over time
5. **Error Rate** - Percentage of failed requests
6. **Provider Breakdown** - Requests by LLM provider
7. **Cost Savings** - Estimated savings from caching

## Alerting Rules

### Critical Alerts

```yaml
# Service down
- alert: SemcacheDown
  expr: up{job="semcache"} == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "semcache service is down"

# High memory usage
- alert: SemcacheHighMemory
  expr: semcache_memory_usage_bytes > 8e9  # 8GB
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "semcache memory usage is critically high"
```

### Warning Alerts

```yaml
# Low cache hit rate
- alert: SemcacheLowHitRate
  expr: rate(semcache_cache_hits_total[10m]) / (rate(semcache_cache_hits_total[10m]) + rate(semcache_cache_misses_total[10m])) < 0.2
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "semcache hit rate is below 20%"

# High error rate
- alert: SemcacheHighErrorRate
  expr: rate(semcache_errors_total[5m]) / rate(semcache_requests_total[5m]) > 0.05
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "semcache error rate is above 5%"
```

## Metrics Collection Best Practices

### Scraping Configuration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'semcache'
    static_configs:
      - targets: ['semcache:8080']
    metrics_path: '/api/metrics'
    scrape_interval: 15s
    scrape_timeout: 10s
```

### Retention and Storage

```yaml
# Prometheus retention configuration
global:
  retention_time: "30d"
  
# For high-volume environments
storage:
  tsdb:
    retention.time: "7d"
    retention.size: "50GB"
```

### Metric Cardinality

**Be aware of cardinality**:
- Provider labels: Low cardinality (few providers)
- Model labels: Medium cardinality (10-50 models)
- User labels: High cardinality (avoid if possible)

## Troubleshooting Metrics

### Common Issues

**Metrics not appearing**:
```bash
# Check endpoint accessibility
curl http://localhost:8080/api/metrics

# Verify Prometheus scraping
kubectl logs prometheus-pod | grep semcache
```

**High cardinality warnings**:
- Review label usage
- Aggregate high-cardinality metrics
- Use recording rules for complex queries

**Missing historical data**:
- Check Prometheus retention settings
- Verify storage capacity
- Review scraping frequency

## Next Steps

- [Admin Dashboard](./admin-dashboard.md) - Visual monitoring interface
- [Troubleshooting](./troubleshooting.md) - Debug performance issues
- [Configuration](../configuration/monitoring.md) - Set up monitoring