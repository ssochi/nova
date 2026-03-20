package main

import (
	"bytes"
	"fmt"
	"strings"
)

func main() {
	var text = "nova-go-go"
	fmt.Println(strings.Index(text, "go"))
	fmt.Println(strings.HasSuffix(text, "go"))
	fmt.Println(strings.TrimPrefix(text, "nova-"))
	fmt.Println(strings.TrimSuffix(text, "-go"))

	var raw []byte
	var trimmedNil = bytes.TrimPrefix(raw, []byte(""))
	fmt.Println(trimmedNil == nil, len(trimmedNil))

	var value = []byte("nova-go")
	fmt.Println(bytes.Index(value, []byte("go")))
	fmt.Println(bytes.HasSuffix(value, []byte("go")))
	fmt.Println(string(bytes.TrimPrefix(value, []byte("nova-"))))
	fmt.Println(string(bytes.TrimSuffix(value, []byte("-go"))))
}
