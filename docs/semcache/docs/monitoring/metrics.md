---
sidebar_position: 1
---

# Prometheus Metrics & Grafana

Semcache exposes Prometheus metrics on the `/metrics` endpoint for monitoring cache performance and system health.

## Available Metrics

- Cache hit/miss rates
- Cache size tracking
- Request latency
- Memory usage

## Setup

Metrics are automatically enabled when semcache starts. Point your Prometheus scraper to:

```
http://your-semcache-host/metrics
```

## Custom Grafana Dashboard 

For detailed monitoring setup including Grafana dashboards and alerting configurations, see the [monitoring directory](https://github.com/sensoris/semcache-rs/tree/main/monitoring) in our GitHub repository.
