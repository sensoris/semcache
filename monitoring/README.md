# Set up Semcache monitoring with Prometheus / Grafana

This project provides an example configuration for setting up a Prometheus and Grafana monitoring stack for you Semcache instance.

## Running

1. Make sure your Semcache instance is running, the files in this project assume it's on `localhost:8080`
2. Run `docker-compose up` to start Prometheus and Grafana
3. Prometheus will begin polling the `/metrics` endpoint on Semcache
4. Navigate to `http://localhost:3000` for Grafana and login with `admin:admin`
5. You will see a custom-built dashboard for Semcache in the dashboards page

## Custom Grafana Dashboard

If you already have a Grafana setup you can use our existing dashboard file: `grafana/provisioning/dashboards/dashboard.json` 
