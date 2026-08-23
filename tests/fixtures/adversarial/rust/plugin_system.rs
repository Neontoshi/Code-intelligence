// tests/fixtures/adversarial/rust/plugin_system.rs

//! Plugin system with dynamic dispatch that looks dead but is alive

use std::collections::HashMap;

// ⚠️ Trait that looks unused but is the plugin interface
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn execute(&self, input: &str) -> String;
}

// ⚠️ This looks dead but is registered dynamically
pub struct LoggerPlugin;

impl Plugin for LoggerPlugin {
    fn name(&self) -> &'static str {
        "logger"
    }

    fn execute(&self, input: &str) -> String {
        format!("[LOG] {}", input)
    }
}

// ⚠️ This looks dead but is registered dynamically
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn name(&self) -> &'static str {
        "transform"
    }

    fn execute(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

// ⚠️ This looks dead but is registered dynamically
pub struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn name(&self) -> &'static str {
        "counter"
    }

    fn execute(&self, input: &str) -> String {
        format!("Count: {}", input.len())
    }
}

// Plugin registry - looks dead but is used dynamically
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn execute(&self, name: &str, input: &str) -> Option<String> {
        self.get(name).map(|p| p.execute(input))
    }
}

// Entry point that uses the plugin system
pub fn main() {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(LoggerPlugin));
    registry.register(Box::new(TransformPlugin));
    registry.register(Box::new(CounterPlugin));

    let result = registry.execute("logger", "test message").unwrap();
    println!("{}", result);
}
