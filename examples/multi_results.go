package main

import (
    "bytes"
    "fmt"
    "strings"
)

func splitTag(value string) (string, string, bool) {
    return strings.Cut(value, "-")
}

func splitBytes(value []byte) ([]byte, []byte, bool) {
    return bytes.Cut(value, []byte("-"))
}

func project(value string) (string, int) {
    head, tail, found := splitTag(value)
    if found == false {
        return value, 0
    }
    return head, len(tail)
}

func main() {
    head, tail, found := splitTag("nova-go")
    fmt.Println(head, tail, found)

    emptyHead, emptyTail, emptyFound := splitTag("nova")
    fmt.Println(emptyHead, emptyTail, emptyFound)

    byteHead, byteTail, byteFound := splitBytes([]byte("vm-loop"))
    fmt.Println(string(byteHead), string(byteTail), byteFound)

    byteHead, byteTail, byteFound = splitBytes([]byte("vm"))
    fmt.Println(string(byteHead), byteTail == nil, byteFound)

    projectHead, projectLen := project("alpha-beta")
    fmt.Println(projectHead, projectLen)

    projectHead, projectLen = project("alpha")
    fmt.Println(projectHead, projectLen)
}
