package main

func main() {
	var values = []int{1, 2, 3, 4}
	var window = values[1:3]
	clear(window)
	println(values[0], values[1], values[2], values[3], len(window), cap(window))

	var missing []int
	clear(missing)
	println(missing == nil, len(missing), cap(missing))

	var labels = []string{"nova", "go"}
	clear(labels)
	println(labels[0] == "", labels[1] == "")

	var counts = map[string]int{"nova": 3, "go": 2}
	var alias = counts
	clear(alias)
	println(len(counts), len(alias), counts == nil)
}
