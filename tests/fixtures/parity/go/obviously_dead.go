package main

import "fmt"

func helperFunction() {
    fmt.Println("This is never called")
}

func main() {
    fmt.Println("Entry point")
}
