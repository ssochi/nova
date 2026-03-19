package main

func main() {
	var counts = map[string]int{"nova": 3, "go": 2}
	switch value, ok := counts["nova"]; {
	case ok:
		println(value)
	default:
		println(0)
	}

	var score int = 2
	switch score {
	case 0, 1:
		println("small")
	case 2:
		println("two")
	default:
		println("big")
	}

	switch println("probe"); {
	case false:
		println("skip")
	default:
		println("done")
	}

	var fallback int
	switch fallback = counts["go"]; fallback {
	case 2:
		println("go")
	default:
		println("missing")
	}
}
