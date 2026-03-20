package main

func main() {
	var boxed any = []byte("nova")

	word, ok := boxed.(string)
	println(ok)

	bytes, ok := boxed.([]byte)
	println(ok, string(bytes))

	switch current := boxed.(type) {
	case []byte:
		println("bytes", string(current))
	case string, bool:
		println("multi", current == true)
	default:
		println("default")
	}

	var missing any
	_, ok = missing.(string)
	println(ok)

	switch current := missing.(type) {
	case nil:
		println("nil", current == nil)
	default:
		println("default")
	}

	var flag any = true
	switch current := flag.(type) {
	case string, bool:
		println("multi", current == true)
	default:
		println("default")
	}
}
