package main

func classify(value int) (sign string, abs int) {
	sign = "non-negative"
	abs = value
	if value < 0 {
		sign = "negative"
		abs = 0 - value
		return
	}
	return
}

func pair() (head, tail string, ok bool) {
	head = "nova"
	tail = "go"
	ok = true
	return
}

func blankLabel(flag bool) (_ int, label string) {
	label = "cold"
	if flag {
		return 1, "warm"
	}
	return
}

func main() {
	println(classify(0 - 3))
	println(classify(5))
	left, right, ok := pair()
	_, label := blankLabel(false)
	println(left, right, ok, label)
}
