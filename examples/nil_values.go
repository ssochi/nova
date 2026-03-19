package main

import "strings"

func acceptNames(values []string) bool {
	return values == nil
}

func nilCounts() map[string]int {
	return nil
}

func main() {
	var values []int = nil
	var counts map[string]int = nil
	println(values == nil, counts == nil)

	values = []int{1, 2}
	counts = map[string]int{"nova": 1}
	println(values == nil, counts == nil)

	values = nil
	counts = nil
	println(values == nil, counts == nil)

	println(acceptNames(nil), nilCounts() == nil, strings.Join(nil, ":") == "")
}
