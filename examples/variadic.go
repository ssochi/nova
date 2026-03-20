package main

func total(prefix int, values ...int) int {
	if values == nil {
		println(true, len(values))
		return prefix
	}

	println(false, len(values))
	var sum = prefix
	for _, value := range values {
		sum = sum + value
	}
	return sum
}

func main() {
	var values = []int{2, 3}
	var empty []int = nil
	println(total(1))
	println(total(1, 2, 3))
	println(total(1, values...))
	println(total(4, empty...))

	var bytes = []byte("go")
	bytes = append(bytes, []byte("-nova")...)
	bytes = append(bytes, "!"...)
	println(string(bytes))
}
