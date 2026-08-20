// tests/fixtures/adversarial/go/interface_impl.go

// Go interface implementations that look dead but are used polymorphically

package main

import "fmt"

// Service interface
type Service interface {
    Process(data string) string
}

// ⚠️ This looks dead but implements the interface
type ProductionService struct{}

func (s ProductionService) Process(data string) string {
    return fmt.Sprintf("prod: %s", data)
}

// ⚠️ This looks dead but implements the interface
type StagingService struct{}

func (s StagingService) Process(data string) string {
    return fmt.Sprintf("staging: %s", data)
}

// ⚠️ This looks dead but implements the interface
type MockService struct{}

func (s MockService) Process(data string) string {
    return fmt.Sprintf("mock: %s", data)
}

// Entry point that uses the interface
func process(s Service, data string) string {
    return s.Process(data)
}

// init function looks dead but is called by Go runtime
func init() {
    fmt.Println("Service initialized")
}

func main() {
    service := ProductionService{}
    result := process(service, "test")
    fmt.Println(result)
}
