const handlers = {};

function register(name, fn) {
    handlers[name] = fn;
}

register("test", () => console.log("Handler"));
handlers["test"]();
