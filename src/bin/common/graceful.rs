// src/bin/common/graceful.rs

use std::time::Duration;
use tokio::time::timeout;

/// Graceful degradation config
#[derive(Debug, Clone)]
pub struct DegradationConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub timeout_seconds: u64,
    pub fallback_enabled: bool,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 100,
            timeout_seconds: 30,
            fallback_enabled: true,
        }
    }
}

/// Result with degradation info
#[derive(Debug, Clone)]
pub struct DegradationResult<T> {
    pub result: Option<T>,
    pub degraded: bool,
    pub retries: u32,
    pub fallback_used: bool,
    pub error: Option<String>,
}

impl<T> DegradationResult<T> {
    pub fn success(value: T) -> Self {
        Self {
            result: Some(value),
            degraded: false,
            retries: 0,
            fallback_used: false,
            error: None,
        }
    }

    pub fn degraded(value: T, retries: u32) -> Self {
        Self {
            result: Some(value),
            degraded: true,
            retries,
            fallback_used: false,
            error: None,
        }
    }

    pub fn fallback(value: T) -> Self {
        Self {
            result: Some(value),
            degraded: true,
            retries: 0,
            fallback_used: true,
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            result: None,
            degraded: true,
            retries: 0,
            fallback_used: false,
            error: Some(error),
        }
    }

    pub fn is_success(&self) -> bool {
        self.result.is_some() && !self.degraded
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded && self.result.is_some()
    }

    pub fn is_failure(&self) -> bool {
        self.result.is_none()
    }
}

/// Execute an async function with retries and timeout
pub async fn with_retries_async<F, Fut, T>(
    mut f: F,
    config: &DegradationConfig,
    fallback: Option<impl Fn() -> T + Send + Sync>,
) -> DegradationResult<T>
where
    F: FnMut() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, String>> + Send,
    T: Clone + Send + Sync + 'static,
{
    let mut retries = 0;

    while retries < config.max_retries {
        let result = timeout(Duration::from_secs(config.timeout_seconds), f()).await;

        match result {
            Ok(Ok(value)) => {
                if retries > 0 {
                    return DegradationResult::degraded(value, retries);
                }
                return DegradationResult::success(value);
            }
            Ok(Err(e)) => {
                if let Some(fallback_fn) = &fallback {
                    if config.fallback_enabled {
                        return DegradationResult::fallback(fallback_fn());
                    }
                }
                retries += 1;
                if retries >= config.max_retries {
                    return DegradationResult::failure(format!("Max retries exceeded: {}", e));
                }
                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
            }
            Err(_) => {
                // Timeout
                if let Some(fallback_fn) = &fallback {
                    if config.fallback_enabled {
                        return DegradationResult::fallback(fallback_fn());
                    }
                }
                retries += 1;
                if retries >= config.max_retries {
                    return DegradationResult::failure("Timeout exceeded".to_string());
                }
            }
        }
    }

    DegradationResult::failure("All retries exhausted".to_string())
}

pub async fn with_retries_blocking<F, T>(
    f: F,
    config: &DegradationConfig,
    fallback: Option<impl Fn() -> T + Send + Sync>,
) -> DegradationResult<T>
where
    F: Fn() -> Result<T, String> + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    let mut retries = 0;
    let f = std::sync::Arc::new(f);

    while retries < config.max_retries {
        let f = f.clone();
        let handle = tokio::task::spawn_blocking(move || f());
        let result = timeout(Duration::from_secs(config.timeout_seconds), handle).await;
        match result {
            Ok(Ok(Ok(value))) => {
                if retries > 0 {
                    return DegradationResult::degraded(value, retries);
                }
                return DegradationResult::success(value);
            }
            Ok(Ok(Err(e))) => {
                if let Some(fallback_fn) = &fallback {
                    if config.fallback_enabled {
                        return DegradationResult::fallback(fallback_fn());
                    }
                }
                retries += 1;
                if retries >= config.max_retries {
                    return DegradationResult::failure(format!("Max retries exceeded: {}", e));
                }
                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
            }
            Ok(Err(e)) => {
                retries += 1;
                if retries >= config.max_retries {
                    return DegradationResult::failure(format!("Task join error: {}", e));
                }
                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
            }
            Err(_) => {
                // Timeout
                if let Some(fallback_fn) = &fallback {
                    if config.fallback_enabled {
                        return DegradationResult::fallback(fallback_fn());
                    }
                }
                retries += 1;
                if retries >= config.max_retries {
                    return DegradationResult::failure("Timeout exceeded".to_string());
                }
            }
        }
    }

    DegradationResult::failure("All retries exhausted".to_string())
}

/// Check if a feature should be degraded based on system conditions
pub fn should_degrade(
    system_load: f64,
    memory_usage_mb: f64,
    memory_limit_mb: Option<usize>,
) -> bool {
    if system_load > 5.0 {
        return true;
    }

    if let Some(limit) = memory_limit_mb {
        if memory_usage_mb > limit as f64 * 0.85 {
            return true;
        }
    }

    false
}

/// Get system load (simplified)
pub fn get_system_load() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = contents.split_whitespace().collect();
            if let Some(load) = parts.first() {
                if let Ok(load) = load.parse::<f64>() {
                    return load;
                }
            }
        }
    }
    0.0
}
