package main

import (
	"bytes"
	"strings"
)

func main() {
	var empty []byte = []byte{}

	println(strings.Clone("nova"))
	println(bytes.Clone(nil) == nil)
	println(bytes.Clone(empty) == nil)
	println(bytes.Clone([]byte("go")))
}
