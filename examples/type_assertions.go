package main

import "fmt"

func main() {
	var text any = "go"
	var word = text.(string)

	var raw []byte
	var payload any = raw
	var bytes = payload.([]byte)

	var count any = 7

	fmt.Println(word)
	fmt.Println(bytes == nil, len(bytes))
	fmt.Println(count.(any) == 7)
}
