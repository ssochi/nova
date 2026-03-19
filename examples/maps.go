package main

func main() {
	var counts map[string]int
	println(len(counts), counts["nova"])

	counts = make(map[string]int, 2)
	counts["nova"] = 3
	counts["go"] = counts["nova"] + 2
	println(len(counts), counts["nova"], counts["go"], counts["missing"])

	var labels = make(map[bool]string)
	labels[true] = "ready"
	println(labels[true], len(labels))
}
