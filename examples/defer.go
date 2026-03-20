package main

import "fmt"

func pair() (int, int) {
	fmt.Println("pair")
	return 1, 2
}

func main() {
	value := 1
	defer println("builtin")
	defer pair()
	defer fmt.Println("package", value)
	value = 9
	fmt.Println("body", value)
}
