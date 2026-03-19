package main

func sumDown(limit int) int {
	var total = 0
	var current = limit
	for current > 0 {
		total = total + current
		current = current - 1
	}
	return total
}

func climbPast(limit int) int {
	var current = 0
	for true {
		current = current + 1
		if current > limit {
			return current
		}
	}
}

func main() {
	println(sumDown(4), climbPast(3))
}
