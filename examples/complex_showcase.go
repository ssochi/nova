package main

import (
	"bytes"
	"fmt"
	"strings"
)

func describe(left, right string) string {
	return left + ":" + right
}

func collect(prefix string, values ...string) (joined string, count int) {
	joined = prefix
	for _, value := range values {
		joined = joined + "/" + value
		count++
	}
	return
}

func cutLabel(value string) (head, tail string, ok bool) {
	head, tail, ok = strings.Cut(value, "-")
	return
}

func recoverBytes() (label string, size int) {
	label = "panic-safe"
	size = 2
	defer printRecover()
	panic([]byte("go"))
}

func printRecover() {
	fmt.Println(recover())
}

func main() {
	head, tail, ok := cutLabel("nova-go")
	fmt.Println(head, tail, ok)
	fmt.Println(cutLabel("vm-loop"))
	fmt.Println(describe(head, tail))

	joined, count := collect("root", head, tail, strings.TrimPrefix("nova-go", "nova-"))
	fmt.Println(joined, count)

	var missing []byte
	var holder any = missing
	fmt.Println(holder == nil, holder.([]byte) == nil)

	var message any = "boom"
	fmt.Println(message.(string), message == "boom")

	values := []int{1, 2, 3, 4}
	window := values[1:3]
	clear(window)

	counts := map[string]int{"nova": 3, "go": 2}
	total := 0
	for key, value := range counts {
		total += len(key) + value
	}
	alias := counts
	clear(alias)
	fmt.Println(values[0], values[1], values[2], values[3], len(counts), total)

	payload := bytes.TrimPrefix([]byte("nova-go"), []byte("nova-"))
	fmt.Println(string(payload), bytes.Equal(payload, []byte("go")))

	args := []any{head, tail, ok, count, joined}
	fmt.Println(args...)

	ready := make(chan int, 2)
	ready <- 5
	ready <- 7
	queued := len(ready)
	first := <-ready
	close(ready)
	second := <-ready
	zero := <-ready
	fmt.Println(queued, cap(ready), first, second, zero)

	fmt.Println(recoverBytes())
}
