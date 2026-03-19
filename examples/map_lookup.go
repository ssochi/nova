package main

func main() {
    var counts map[string]int
    value, ok := counts["nova"]
    println(value, ok)

    counts = map[string]int{"go": 2, "nova": 3}
    value, ok = counts["nova"]
    println(value, ok)

    _, ok = counts["missing"]
    println(ok)

    value, seen := counts["go"]
    println(value, seen)
}
