// tests/fixtures/adversarial/go/plugin_export.go

package main

import (
    "fmt"
    "plugin"
)

// ⚠️ This looks dead but is exported as a plugin symbol
var PluginName = "example-plugin"

// ⚠️ This looks dead but is exported as a plugin symbol
var PluginVersion = "1.0.0"

// ⚠️ This looks dead but is exported as a plugin symbol
func InitPlugin() error {
    fmt.Println("Plugin initialized")
    return nil
}

// ⚠️ This looks dead but is exported as a plugin symbol
func Process(data string) string {
    return fmt.Sprintf("processed: %s", data)
}

// ⚠️ This looks dead but is exported as a plugin symbol
func Cleanup() {
    fmt.Println("Plugin cleanup")
}

// Entry point that loads and uses plugins
func main() {
    // Load a plugin dynamically
    p, err := plugin.Open("plugin.so")
    if err != nil {
        fmt.Println("No plugin found, skipping")
        return
    }

    // Look up symbols
    symInit, err := p.Lookup("InitPlugin")
    if err == nil {
        if initFunc, ok := symInit.(func()); ok {
            initFunc()
        }
    }

    symProcess, err := p.Lookup("Process")
    if err == nil {
        if processFunc, ok := symProcess.(func(string) string); ok {
            result := processFunc("test")
            fmt.Println(result)
        }
    }
}
