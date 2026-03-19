package main

func main() {
	var values = []int{1, 2, 3, 4}
	var head = values[:2]
	var reopen = head[1:4]
	reopen[0] = 9
	reopen[2] = 7
	println(len(reopen), reopen[0], reopen[2])
	println(values[0], values[1], values[2], values[3])
}
