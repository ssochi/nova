package main

func main() {
    var values = make([]int, 2, 4)
    values[0] = 7
    values[1] = 8
    var head = values[:3]
    println(len(values), cap(values), values[0], values[1])
    println(head[2], len(head), cap(head))

    var grown = append(values, 9)
    println(len(grown), cap(grown), grown[2])

    var labels = make([]string, 2)
    println(len(labels), cap(labels), len(labels[0]), len(labels[1]))
}
