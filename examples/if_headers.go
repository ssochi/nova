package main

func main() {
    var counts = map[string]int{"nova": 3, "fallback": 2}

    if value, ok := counts["nova"]; ok {
        println(value, ok)
    } else {
        println(0, ok)
    }

    var fallback int
    if fallback = counts["missing"]; fallback > 0 {
        println(fallback)
    } else if fallback = counts["fallback"]; fallback > 0 {
        println(fallback)
    } else {
        println(0)
    }

    if println("probe"); false {
        println(99)
    } else if var ready bool = false; ready {
        println(1)
    } else {
        println(7)
    }
}
