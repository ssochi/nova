package main

func main() {
	var values = []int{1, 2, 3, 4}
	var head = values[:2]
	println(len(head), cap(head))

	var grown = append(head, 9)
	println(values[2], len(grown), cap(grown))

	var copied = copy(values, values[1:])
	println(copied, values[0], values[1], values[2], values[3])
}
