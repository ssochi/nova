package main

func main() {
	var ready chan int
	println(ready == nil)

	ready = make(chan int, 2)
	var alias chan int = ready
	println(len(ready), cap(ready), alias == ready, alias != nil)

	ready <- 4
	ready <- 7
	var queued = len(ready)
	var first = <-ready
	close(ready)
	var second = <-ready
	var zero = <-ready
	println(queued, first, second, zero)
}
