// tests/fixtures/adversarial/rust/macro_used.rs

//! Functions that look dead but are used by macros

// ⚠️ This looks dead but is used by a derive macro
#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub value: i32,
}

// ⚠️ This looks dead but is used by the macro
impl Config {
    pub fn new(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

// ⚠️ This looks dead but is invoked by a macro
pub fn validate_config(config: &Config) -> bool {
    !config.name.is_empty() && config.value > 0
}

// Macro that uses functions
#[macro_export]
macro_rules! config {
    ($name:expr, $value:expr) => {{
        let cfg = Config::new($name, $value);
        if validate_config(&cfg) {
            Some(cfg)
        } else {
            None
        }
    }};
}

// Entry point that uses the macro
pub fn main() {
    let _cfg = config!("test", 42);
}
