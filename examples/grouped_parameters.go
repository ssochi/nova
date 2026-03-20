package main

func describe(left, right string) string {
    return left + "-" + right
}

func total(base, offset int, values ...int) int {
    var sum = base + offset
    for _, value := range values {
        sum += value
    }
    return sum
}

func main() {
    println(describe("nova", "go"))
    println(total(1, 2))
    println(total(1, 2, 3, 4))
}
