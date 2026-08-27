Map<String, Function> handlers = {};

void register(String name, Function fn) {
    handlers[name] = fn;
}

void main() {
    register("test", () => print("Handler"));
    handlers["test"]();
}
