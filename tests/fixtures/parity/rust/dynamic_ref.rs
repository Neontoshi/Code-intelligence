use std::collections::HashMap;

type Handler = Box<dyn Fn()>;

fn register_handler(handlers: &mut HashMap<String, Handler>, name: &str) {
    handlers.insert(name.to_string(), Box::new(|| println!("Handler")));
}

fn main() {
    let mut handlers: HashMap<String, Handler> = HashMap::new();
    register_handler(&mut handlers, "test");
}
