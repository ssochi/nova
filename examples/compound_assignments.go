package main

func main() {
	total := 1
	values := []int{2, 3}
	for i := 0; i < len(values); i += 1 {
		total += values[i]
	}

	words := map[string]string{"lang": "go"}
	words["lang"] += "pher"

	code := []byte("ab")
	code[0] -= "a"[0]
	code[0] += "!"[0]

	factor := 2
	factor *= 3
	factor /= 2

	probe := 1
	if probe += len(values); probe > 2 {
		println(total, words["lang"], code[0], factor, probe)
	}

	switch probe -= 1; {
	case probe == 2:
		println("ready")
	default:
		println("miss")
	}
}
