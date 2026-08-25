// src/bin/common/monitor.rs

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Performance metrics collector
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, Vec<MetricPoint>>>>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub label: String,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a metric value
    pub async fn record(&self, name: &str, value: f64, label: &str) {
        let mut metrics = self.metrics.lock().await;
        let point = MetricPoint {
            timestamp: Instant::now().duration_since(self.start_time).as_secs(),
            value,
            label: label.to_string(),
        };
        metrics.entry(name.to_string()).or_default().push(point);
    }

    /// Record a metric with current timestamp
    pub async fn record_now(&self, name: &str, value: f64) {
        self.record(name, value, "default").await;
    }

    /// Get metrics for a name
    pub async fn get(&self, name: &str) -> Vec<MetricPoint> {
        let metrics = self.metrics.lock().await;
        metrics.get(name).cloned().unwrap_or_default()
    }

    /// Get all metrics
    pub async fn get_all(&self) -> HashMap<String, Vec<MetricPoint>> {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// Get the latest value for a metric
    pub async fn get_latest(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.lock().await;
        metrics
            .get(name)
            .and_then(|points| points.last())
            .map(|p| p.value)
    }

    /// Get average value for a metric
    pub async fn get_average(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.lock().await;
        metrics.get(name).map(|points| {
            let sum: f64 = points.iter().map(|p| p.value).sum();
            sum / points.len() as f64
        })
    }

    /// Generate a summary report
    pub async fn generate_report(&self) -> String {
        let metrics = self.metrics.lock().await;
        let mut report = String::new();

        report.push_str("📊 Performance Metrics\n");
        report.push_str("=====================\n\n");

        for (name, points) in metrics.iter() {
            if points.is_empty() {
                continue;
            }

            let avg: f64 = points.iter().map(|p| p.value).sum::<f64>() / points.len() as f64;
            let min = points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
            let max = points
                .iter()
                .map(|p| p.value)
                .fold(f64::NEG_INFINITY, f64::max);

            report.push_str(&format!("📈 {}\n", name));
            report.push_str(&format!("   Count: {}\n", points.len()));
            report.push_str(&format!("   Avg: {:.2}\n", avg));
            report.push_str(&format!("   Min: {:.2}\n", min));
            report.push_str(&format!("   Max: {:.2}\n\n", max));
        }

        report
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring execution time
pub struct Timer<'a> {
    collector: &'a MetricsCollector,
    name: String,
    start: Instant,
    labels: HashMap<String, String>,
}

impl<'a> Timer<'a> {
    pub fn new(collector: &'a MetricsCollector, name: &str) -> Self {
        Self {
            collector,
            name: name.to_string(),
            start: Instant::now(),
            labels: HashMap::new(),
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }

    pub async fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        let value = elapsed.as_secs_f64();

        let label = self
            .labels
            .get("operation")
            .unwrap_or(&"default".to_string())
            .clone();

        self.collector
            .record(&format!("timer.{}", self.name), value, &label)
            .await;

        elapsed
    }

    pub async fn stop_and_report(self) {
        self.stop().await;
    }
}

/// Count operations
pub struct Counter<'a> {
    collector: &'a MetricsCollector,
    name: String,
    count: u64,
}

impl<'a> Counter<'a> {
    pub fn new(collector: &'a MetricsCollector, name: &str) -> Self {
        Self {
            collector,
            name: name.to_string(),
            count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub async fn flush(&mut self) {
        if self.count > 0 {
            self.collector
                .record_now(&format!("counter.{}", self.name), self.count as f64)
                .await;
            self.count = 0;
        }
    }
}
