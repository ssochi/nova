package main

import "fmt"
import "strings"

func label(parts []string) string {
	return strings.Join(parts, "-")
}

func main() {
	var parts = []string{"nova", strings.Repeat("go", 2), "vm"}
	var joined = label(parts)
	fmt.Println(joined)
	fmt.Println(strings.Contains(joined, "gogo"))
	fmt.Println(strings.HasPrefix(joined, "nova"))
}
