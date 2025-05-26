---
sidebar_position: 3
---

# Monitoring Configuration

Set up monitoring and observability for your semcache deployment.

## Built-in Monitoring

semcache includes several monitoring capabilities out of the box.

### Admin Dashboard

**URL**: `http://localhost:8080/admin`

**Features**:
- Real-time cache hit/miss statistics
- Memory usage tracking
- Response time metrics
- Cached entries visualization
- System health indicators

**Access**:
```bash
# Open in browser
open http://localhost:8080/admin

# Or use curl for programmatic access
curl http://localhost:8080/admin
```

### Prometheus Metrics

**Endpoint**: `http://localhost:8080/api/metrics`

**Format**: Prometheus exposition format for scraping

**Sample metrics**:
```
# Cache hit rate
semcache_cache_hits_total 45
semcache_cache_misses_total 15

# Response times
semcache_request_duration_seconds_bucket{le="0.01"} 30
semcache_request_duration_seconds_bucket{le="0.1"} 55
semcache_request_duration_seconds_bucket{le="1.0"} 60

# Memory usage
semcache_memory_usage_bytes 104857600
semcache_cached_entries_total 42
```

## Metrics Collection

### Current Metrics

**Performance Metrics**:
- Request count (total, by endpoint)
- Response latency (cache hits vs misses)
- Embedding generation time
- Cache hit rate percentage

**Resource Metrics**:
- Memory usage (total, by component)
- CPU utilization (embedding generation)
- Cache size (entries, memory footprint)

**Error Metrics**:
- Upstream connection errors
- Embedding generation failures
- Invalid request counts

### Metrics Storage

**Current behavior**:
- Metrics stored in memory
- Historical data kept for runtime only
- No persistence across restarts

**Collection interval**:
- Metrics updated every 30 seconds
- Real-time updates for request counters
- Historical data available via admin dashboard

## Prometheus Integration

### Scraping Configuration

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'semcache'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/api/metrics'
    scrape_interval: 15s
```

### Example Queries

**Cache hit rate**:
```promql
rate(semcache_cache_hits_total[5m]) / 
(rate(semcache_cache_hits_total[5m]) + rate(semcache_cache_misses_total[5m])) * 100
```

**Average response time**:
```promql
rate(semcache_request_duration_seconds_sum[5m]) / 
rate(semcache_request_duration_seconds_count[5m])
```

**Memory growth rate**:
```promql
rate(semcache_memory_usage_bytes[1h])
```

## Grafana Dashboard

### Sample Dashboard

```json
{
  "dashboard": {
    "title": "semcache Monitoring",
    "panels": [
      {
        "title": "Cache Hit Rate",
        "type": "stat",
        "targets": [{
          "expr": "rate(semcache_cache_hits_total[5m]) / (rate(semcache_cache_hits_total[5m]) + rate(semcache_cache_misses_total[5m])) * 100"
        }]
      },
      {
        "title": "Response Times",
        "type": "graph",
        "targets": [{
          "expr": "histogram_quantile(0.95, rate(semcache_request_duration_seconds_bucket[5m]))",
          "legendFormat": "95th percentile"
        }]
      }
    ]
  }
}
```

## Health Checks

### Basic Health Check

```bash
# Simple availability check
curl http://localhost:8080/
# Response: "Hello, World!"
```

### Detailed Health Check

**Future endpoint**: `http://localhost:8080/health`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "cache": {
    "entries": 42,
    "memory_mb": 12.5,
    "hit_rate": 0.65
  },
  "embedding": {
    "model": "all-MiniLM-L6-v2",
    "avg_generation_ms": 45
  },
  "upstream": {
    "last_success": "2024-01-15T10:30:00Z",
    "error_count": 0
  }
}
```

## Logging Configuration

### Current Logging

**Log level**: Info (configurable via environment)
**Format**: Structured JSON logs
**Output**: stdout (containerized deployments)

**Sample log entry**:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "message": "Cache hit for similar query",
  "similarity": 0.92,
  "response_time_ms": 8,
  "cached_entry_age_seconds": 1800
}
```

### Log Levels

```bash
# Set log level via environment
export RUST_LOG=debug  # trace, debug, info, warn, error
cargo run
```

**Log content by level**:
- **Error**: Upstream failures, system errors
- **Warn**: Cache evictions, threshold misses
- **Info**: Cache hits/misses, request processing
- **Debug**: Detailed similarity scores, timing
- **Trace**: Full request/response dumps

### Future Log Configuration

```yaml
# Planned logging configuration
logging:
  level: "info"
  format: "json"  # json, text, compact
  output: "stdout"  # stdout, stderr, file
  file:
    path: "/var/log/semcache/app.log"
    rotation: "daily"
    max_size: "100MB"
  structured_fields:
    - request_id
    - similarity_score
    - cache_hit
    - response_time
```

## Alerting

### Recommended Alerts

**High priority**:
```yaml
# Service down
- alert: SemcacheDown
  expr: up{job="semcache"} == 0
  for: 1m

# Memory usage critical
- alert: SemcacheMemoryHigh
  expr: semcache_memory_usage_bytes > 8e9  # 8GB
  for: 5m
```

**Medium priority**:
```yaml
# Low cache hit rate
- alert: SemcacheLowHitRate
  expr: rate(semcache_cache_hits_total[10m]) / (rate(semcache_cache_hits_total[10m]) + rate(semcache_cache_misses_total[10m])) < 0.2
  for: 10m

# High response times
- alert: SemcacheSlowResponses
  expr: histogram_quantile(0.95, rate(semcache_request_duration_seconds_bucket[5m])) > 0.5
  for: 5m
```

**Low priority**:
```yaml
# Frequent cache evictions
- alert: SemcacheFrequentEvictions
  expr: rate(semcache_cache_evictions_total[1h]) > 10
  for: 30m
```

## Performance Monitoring

**Resource Utilization**:
- Memory growth: Linear with cache size
- CPU spikes: During embedding generation
- Network I/O: Proportional to cache misses

### Monitoring Tools Integration

**Docker monitoring**:
```bash
# Container stats
docker stats semcache

# Container logs
docker logs -f semcache
```

**System monitoring**:
```bash
# Memory usage
ps aux | grep semcache

# Network connections
netstat -an | grep :8080
```

## Troubleshooting Monitoring

### Common Issues

**Metrics not appearing**:
- Check `/api/metrics` endpoint accessibility
- Verify Prometheus scraping configuration
- Ensure no firewall blocking port 8080

**Admin dashboard not loading**:
- Check semcache service status
- Verify port 8080 is accessible
- Check browser console for JavaScript errors

**Missing log entries**:
- Verify log level configuration
- Check stdout/stderr redirection
- Ensure structured logging is enabled

### Debug Commands

```bash
# Check metrics endpoint
curl -s http://localhost:8080/api/metrics | head -20

# Monitor real-time logs
docker logs -f semcache | jq '.'

# Test admin dashboard
curl -I http://localhost:8080/admin
```

## Future Monitoring Features

### Planned Enhancements

**Distributed tracing**:
- OpenTelemetry integration
- Request tracing across cache layers
- Embedding generation spans

**Advanced metrics**:
- Cache warming effectiveness
- Similarity score distributions
- Provider-specific latencies

**Alerting integration**:
- Webhook notifications
- Slack/Discord integration
- PagerDuty escalation

### Custom Metrics

**Future custom metrics endpoint**:
```yaml
# Custom metric configuration
metrics:
  custom:
    - name: "business_cache_savings"
      type: "counter"
      help: "Cost savings from cache hits"
      labels: ["provider", "model"]
```

## Next Steps

- [Admin Dashboard](../monitoring/admin-dashboard.md) - Detailed dashboard guide
- [Metrics Reference](../monitoring/metrics.md) - Complete metrics documentation
- [Troubleshooting](../monitoring/troubleshooting.md) - Common monitoring issues