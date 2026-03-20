package main

import (
	b "bytes"
	"fmt"
)

func main() {
	var parts = [][]byte{[]byte("nova"), b.Repeat([]byte("go"), 2)}
	var joined = b.Join(parts, []byte("-"))
	fmt.Println(string(joined))
	fmt.Println(b.Equal(nil, []byte{}))
	fmt.Println(b.Contains(joined, []byte("gogo")))
	fmt.Println(b.HasPrefix(joined, []byte("nova")))
}
