package main

func main() {
	total := 0
	values := []int{1, 2, 3}
	for i := 0; i < len(values); i++ {
		total = total + values[i]
	}

	if count := len(values); count > 2 {
		println(total, count)
	}

	counts := map[string]int{"go": 1}
	counts["go"]++
	current := counts["go"]
	current--
	println(counts["go"], current)

	switch probe := current; {
	case probe == 1:
		println("ready")
	default:
		println("miss")
	}
}
