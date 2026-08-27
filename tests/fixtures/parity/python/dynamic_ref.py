def handler():
    print("Handler")

def register(name):
    handlers = {"test": handler}
    return handlers[name]

register("test")()
