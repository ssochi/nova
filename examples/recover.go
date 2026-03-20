package main

import "fmt"

func printRecover() {
	fmt.Println(recover())
}

func helperRecover() any {
	return recover()
}

func printHelperRecover() {
	fmt.Println(helperRecover())
}

func recoverToZero() int {
	defer printRecover()
	panic("boom")
}

func recoverNamed() (result int) {
	result = 7
	defer printRecover()
	panic([]byte("go"))
}

func main() {
	fmt.Println(recover())
	printHelperRecover()
	fmt.Println(recoverToZero())
	fmt.Println(recoverNamed())
}
