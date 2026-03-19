package main

import "fmt"

func label(name string) string {
    return fmt.Sprint("hello, ", name)
}

func main() {
    var message = label("nova")
    fmt.Println(message)
    fmt.Print(fmt.Sprint("bytes=", len(message)))
}
