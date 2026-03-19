package main

func build() []int {
    var values = []int{1, 2}
    values = append(values, 3, 5)
    return values
}

func main() {
    var values = build()
    println(len(values), values[0], values[3])
    println(values)
}
