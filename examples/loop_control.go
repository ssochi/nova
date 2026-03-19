package main

func main() {
	var total int
	for var i int = 0; i < 5; i = i + 1 {
		if i == 1 {
			continue
		}
		switch i {
		case 3:
			break
		}
		total = total + i
		if total > 4 {
			break
		}
	}
	println(total)

	var counts = map[string]int{"go": 2, "nova": 3}
	var seen string
	for key, value := range counts {
		if value < 3 {
			continue
		}
		seen = seen + key
		break
	}
	println(seen)
}
