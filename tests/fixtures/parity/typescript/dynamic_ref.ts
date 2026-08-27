const handlers: Record<string, () => void> = {};

function register(name: string, fn: () => void): void {
    handlers[name] = fn;
}

register("test", () => console.log("Handler"));
handlers["test"]();
