package main

import (
	"bytes"
	"strings"
)

func main() {
	println(strings.Compare("go", "go"))
	println(strings.Compare("go", "vm"))
	println(strings.Compare("vm", "go"))

	var empty []byte
	println(bytes.Compare(nil, empty))
	println(bytes.Compare([]byte("go"), []byte("vm")))
	println(bytes.Compare([]byte("vm"), nil))
}
