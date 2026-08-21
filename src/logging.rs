// src/logging.rs

//! Structured logging for the code-intelligence engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: i64,
    pub level: LogLevel,
    pub event: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub target: String,
    pub span_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct StructuredLogger {
    level: LogLevel,
    events: Vec<LogEvent>,
    json_output: bool,
}

impl StructuredLogger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            events: Vec::new(),
            json_output: false,
        }
    }

    pub fn with_json(mut self, json: bool) -> Self {
        self.json_output = json;
        self
    }

    pub fn log(
        &mut self,
        level: LogLevel,
        event: &str,
        fields: HashMap<String, serde_json::Value>,
    ) {
        if (level.clone() as u8) < self.level.clone() as u8 {
            return;
        }

        let log_event = LogEvent {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            level,
            event: event.to_string(),
            fields,
            target: module_path!().to_string(),
            span_id: None,
        };

        if self.json_output {
            if let Ok(json) = serde_json::to_string(&log_event) {
                println!("{}", json);
            }
        } else {
            let fields_str = log_event
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");

            println!(
                "[{}] {}: {} {}",
                log_event.level.as_str(),
                log_event.timestamp,
                log_event.event,
                fields_str
            );
        }

        self.events.push(log_event);
    }

    pub fn info(&mut self, event: &str, fields: HashMap<String, serde_json::Value>) {
        self.log(LogLevel::Info, event, fields);
    }

    pub fn warn(&mut self, event: &str, fields: HashMap<String, serde_json::Value>) {
        self.log(LogLevel::Warn, event, fields);
    }

    pub fn error(&mut self, event: &str, fields: HashMap<String, serde_json::Value>) {
        self.log(LogLevel::Error, event, fields);
    }

    pub fn debug(&mut self, event: &str, fields: HashMap<String, serde_json::Value>) {
        self.log(LogLevel::Debug, event, fields);
    }

    pub fn get_events(&self) -> &[LogEvent] {
        &self.events
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

impl Default for StructuredLogger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}
