package main

func main() {
	var total int
	var ready bool
	var label string
	var values []int
	println(total, ready, len(label), len(values), cap(values))

	values = append(values, 4, 5)
	var head []int = values[:1]
	println(len(values), values[0], values[1])
	println(len(head), cap(head), head[0])
}
