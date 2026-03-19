package main

func max(left int, right int) int {
	if left > right {
		return left
	} else {
		return right
	}
}

func report(left int, right int) {
	var winner = max(left, right)
	if winner == left {
		println(true, winner)
	} else {
		println(false, winner)
	}
}

func main() {
	report(7, 11)
}
