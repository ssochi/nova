package main

import (
    "bytes"
    "fmt"
    "strings"
)

func pair() (string, string) {
    return "nova", "go"
}

func joinPair(left string, right string) string {
    return left + ":" + right
}

func describeCut(before string, after string, found bool) string {
    return before + "|" + after + "|" + fmt.Sprint(found)
}

func describeStringFlag(value string, found bool) string {
    if found {
        return value + ":true"
    }
    return value + ":false"
}

func describeBytesFlag(value []byte, found bool) string {
    return string(value) + ":" + fmt.Sprint(found)
}

func main() {
    fmt.Println(joinPair(pair()))
    fmt.Println(describeCut(strings.Cut("nova-go", "-")))
    fmt.Println(describeStringFlag(strings.CutPrefix("nova-go", "nova-")))
    fmt.Println(describeStringFlag(strings.CutSuffix("nova-go", "-go")))
    fmt.Println(describeBytesFlag(bytes.CutPrefix([]byte("nova-go"), []byte("nova-"))))
    fmt.Println(describeBytesFlag(bytes.CutSuffix([]byte("nova-go"), []byte("-go"))))
    fmt.Println(strings.Cut("nova", "-"))
    fmt.Println(bytes.CutPrefix([]byte("nova"), []byte("go")))
}
