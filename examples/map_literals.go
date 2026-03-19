package main

func main() {
	var counts = map[string]int{"nova": 3, "go": 2,}
	delete(counts, "go")
	println(len(counts), counts["nova"], counts["go"])

	var empty = map[string]int{}
	delete(empty, "missing")
	println(len(empty))

	var nilCounts map[string]int
	delete(nilCounts, "ghost")
	println(len(nilCounts), nilCounts["ghost"])
}
