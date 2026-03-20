package main

import "fmt"

func wrap(value string) any {
	return any(value)
}

func main() {
	var zero any
	println(zero == nil)

	var boxed interface{} = []byte("go")
	println(boxed == nil)

	var message = wrap("boom")
	println(message == "boom")
	println(message)

	var args = []any{wrap("go"), 7, nil}
	fmt.Println(args...)
}
