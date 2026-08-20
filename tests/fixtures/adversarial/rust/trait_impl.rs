// tests/fixtures/adversarial/rust/trait_impl.rs

//! This function looks dead (no callers, unreachable) but is actually
//! used polymorphically through a trait.

pub trait Handler {
    fn handle(&self, request: &str) -> String;
}

pub struct DefaultHandler;

impl Handler for DefaultHandler {
    fn handle(&self, request: &str) -> String {
        format!("Handled: {}", request)
    }
}

// ⚠️ This looks dead but is called via dynamic dispatch
pub struct DynamicHandler;

impl Handler for DynamicHandler {
    fn handle(&self, request: &str) -> String {
        format!("Dynamic: {}", request)
    }
}

// This is the entry point that uses the trait
pub fn process_request(handler: &dyn Handler, request: &str) -> String {
    handler.handle(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler() {
        let handler = DefaultHandler;
        assert_eq!(process_request(&handler, "test"), "Handled: test");
    }
}
