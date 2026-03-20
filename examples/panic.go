package main

func inner() {
	defer println("inner defer")
	panic("boom")
}

func main() {
	defer println("outer defer")
	println("body")
	inner()
}
