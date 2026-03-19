package main

func main() {
	var values = []int{3, 5}
	var sum int
	for _, value := range values {
		sum = sum + value
	}

	var indexes int
	for index := range values {
		indexes = indexes + index
	}

	var ticks int
	for range values {
		ticks = ticks + 1
	}

	var counts = map[string]int{"nova": 3, "go": 2}
	var total int
	for _, value := range counts {
		total = total + value
	}

	var seen string
	for key := range counts {
		seen = seen + key
	}

	for key, value := range counts {
		println(key, value)
	}

	var nilValues []int
	var nilCounts map[string]int
	var nilSliceHits int
	for range nilValues {
		nilSliceHits = nilSliceHits + 1
	}
	var nilMapHits int
	for range nilCounts {
		nilMapHits = nilMapHits + 1
	}

	println(sum, indexes, ticks)
	println(total, seen)
	println(nilSliceHits, nilMapHits)
}
