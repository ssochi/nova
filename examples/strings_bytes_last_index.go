package main

import (
	"bytes"
	"fmt"
	"strings"
)

func main() {
	var text = "nova-go-go"
	fmt.Println(strings.LastIndex(text, "go"))
	fmt.Println(strings.LastIndex(text, ""))
	fmt.Println(strings.IndexByte(text, text[4]))
	fmt.Println(strings.LastIndexByte(text, text[1]))

	var raw []byte
	fmt.Println(raw == nil, bytes.LastIndex(raw, []byte("")))

	var value = []byte(text)
	fmt.Println(bytes.LastIndex(value, []byte("go")))
	fmt.Println(bytes.IndexByte(value, value[4]))
	fmt.Println(bytes.LastIndexByte(value, value[1]))
}
