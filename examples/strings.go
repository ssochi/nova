package main

func greet(name string) string {
	return "hello, " + name
}

func main() {
	var greeting = greet("nova")
	print(greeting)
	println("!", len(greeting))
	if greeting == "hello, nova" {
		println(true)
	} else {
		println(false)
	}
}
