package main

import "fmt"

type Handler func()

var handlers = make(map[string]Handler)

func register(name string, fn Handler) {
    handlers[name] = fn
}

func main() {
    register("test", func() { fmt.Println("Handler") })
    handlers["test"]()
}
