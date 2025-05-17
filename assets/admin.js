// Chart configuration defaults for styling
const defaultChartConfig = {
    line: {
        fill: false,
        borderColor: '#4a6cf7',
        tension: 0.4,
        pointBackgroundColor: '#ffffff',
        pointBorderColor: '#4a6cf7',
        pointBorderWidth: 2,
        pointRadius: 4
    },
    bar: {
        backgroundColor: '#4a6cf7',
        borderColor: '#4a6cf7',
        borderWidth: 1,
        borderRadius: 4,
        maxBarThickness: 40
    },
    doughnut: {
        backgroundColor: ['#4a6cf7', '#28a745', '#ffc107', '#dc3545', '#6c757d'],
        borderColor: '#ffffff',
        borderWidth: 2
    }
};

// Chart options for different chart types
const chartOptions = {
    line: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
            x: {
                grid: {
                    display: false
                }
            },
            y: {
                beginAtZero: true,
                ticks: {
                    precision: 0
                }
            }
        },
        plugins: {
            legend: {
                display: false
            },
            tooltip: {
                backgroundColor: '#1e293b',
                titleColor: '#ffffff',
                bodyColor: '#ffffff',
                titleFont: {
                    size: 14,
                    weight: 'bold'
                },
                bodyFont: {
                    size: 14
                },
                padding: 12,
                displayColors: false
            }
        }
    },
    bar: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
            x: {
                grid: {
                    display: false
                }
            },
            y: {
                beginAtZero: true,
                ticks: {
                    precision: 0
                }
            }
        },
        plugins: {
            legend: {
                display: false
            },
            tooltip: {
                backgroundColor: '#1e293b',
                titleColor: '#ffffff',
                bodyColor: '#ffffff',
                padding: 12
            }
        }
    },
    doughnut: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
            legend: {
                position: 'top'
            },
            tooltip: {
                backgroundColor: '#1e293b',
                titleColor: '#ffffff',
                bodyColor: '#ffffff',
                padding: 12
            }
        }
    }
};

// Store chart instances for updating
const charts = {};
// Store the latest metrics data
let metricsData = { metrics: [] };
// Store historical metrics data
let historicalMetricsData = [];
// Track stats cards that have been created
const createdStatsCards = new Set();

// Function to format time for display
function formatTime(dateString) {
    const date = new Date(dateString);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

// Function to format date and time for last updated display
function formatDateTime(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString();
}

// Function to create or update a chart
function createOrUpdateChart(metric) {
    const chartId = `chart-${metric.name.replace(/\s+/g, '-').toLowerCase()}`;
    const chartType = metric.chart_type || 'line';

    // Default to line chart if chart_type is not recognized
    const actualChartType = ['line', 'bar', 'doughnut'].includes(chartType) ? chartType : 'line';

    // Get historical data for this specific metric
    const metricHistory = getMetricHistory(metric.name);

    // If chart doesn't exist, create container and canvas
    if (!document.getElementById(chartId)) {
        // Create chart container
        const chartContainer = document.createElement('div');
        chartContainer.className = 'chart-container';
        chartContainer.innerHTML = `
            <div class="chart-header">
                <h2 class="chart-title">${metric.name}</h2>
            </div>
            <canvas id="${chartId}"></canvas>
        `;

        // Add to the page
        document.getElementById('charts-area').appendChild(chartContainer);

        // Initialize chart with history data
        const ctx = document.getElementById(chartId);

        // Prepare historical data for the chart
        const labels = metricHistory.timestamps;
        const values = metricHistory.values;

        const chartData = {
            labels: labels,
            datasets: [{
                label: metric.name,
                data: values,
                ...defaultChartConfig[actualChartType]
            }]
        };

        const config = {
            type: actualChartType,
            data: chartData,
            options: chartOptions[actualChartType]
        };

        charts[chartId] = new Chart(ctx, config);
    } else {
        // Update chart with new data
        const chart = charts[chartId];

        // For time-series charts (line, bar)
        if (actualChartType === 'line' || actualChartType === 'bar') {
            // Replace data with historical data
            chart.data.labels = metricHistory.timestamps;
            chart.data.datasets[0].data = metricHistory.values;
        }
        // For doughnut/pie charts, we just use the current value
        else if (actualChartType === 'doughnut') {
            chart.data.labels = [metric.name];
            chart.data.datasets[0].data = [metric.value];
        }

        chart.update();
    }
}

// Function to create or update stat cards
function createOrUpdateStatCard(metric) {
    const cardId = `stat-${metric.name.replace(/\s+/g, '-').toLowerCase()}`;

    // Only create the card if it doesn't exist yet
    if (!createdStatsCards.has(cardId)) {
        const statCard = document.createElement('div');
        statCard.className = 'stat-card';
        statCard.id = cardId;
        statCard.innerHTML = `
            <div class="stat-title">${metric.name}</div>
            <div class="stat-value" id="${cardId}-value">${metric.value}</div>
        `;

        document.getElementById('stats-container').appendChild(statCard);
        createdStatsCards.add(cardId);
    } else {
        // Just update the value if the card already exists
        document.getElementById(`${cardId}-value`).textContent = metric.value;
    }
}

// Process historical metrics to get data for a specific metric
function getMetricHistory(metricName) {
    const timestamps = [];
    const values = [];

    // Extract data for this specific metric from the historical data
    historicalMetricsData.forEach(item => {
        const matchingMetric = item.metrics.find(m => m.name === metricName);
        if (matchingMetric) {
            timestamps.push(formatTime(item.timestamp));
            values.push(matchingMetric.value);
        }
    });

    // Add current data if available
    if (metricsData && metricsData.timestamp) {
        const currentMetric = metricsData.metrics.find(m => m.name === metricName);
        if (currentMetric) {
            timestamps.push(formatTime(metricsData.timestamp));
            values.push(currentMetric.value);
        }
    }

    return { timestamps, values };
}

// Function to refresh all UI elements with new data
function refreshUI() {
    if (!metricsData || !metricsData.metrics) return;

    // Update last updated time
    document.getElementById('last-updated').textContent =
        `Last updated: ${formatDateTime(metricsData.timestamp)}`;

    // Update each metric
    metricsData.metrics.forEach(metric => {
        createOrUpdateStatCard(metric);
        createOrUpdateChart(metric);
    });
}

// Function to fetch metrics data
async function fetchMetrics() {
    try {
        const response = await fetch('/api/metrics');
        if (!response.ok) {
            throw new Error('Network response was not ok');
        }

        metricsData = await response.json();
        refreshUI();
    } catch (error) {
        console.error('Error fetching metrics:', error);
    }
}

// Function to fetch historical metrics data
async function fetchHistoricalMetrics() {
    try {
        const response = await fetch('/static/metrics_history.json');
        if (!response.ok) {
            throw new Error('Network response was not ok');
        }

        historicalMetricsData = await response.json();
        refreshUI();
    } catch (error) {
        console.error('Error fetching historical metrics:', error);
    }
}

// Initialize the page when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    // Fetch historical metrics first
    fetchHistoricalMetrics().then(() => {
        // Then fetch current metrics
        fetchMetrics();
    });

    // Then fetch current metrics every 5 seconds
    setInterval(fetchMetrics, 5000);
});