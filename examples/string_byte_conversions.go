package main

func main() {
	var text = "nova"
	var bytes = []byte(text)
	println(len(bytes), bytes[0], bytes[3])

	bytes[0] = "X"[0]
	println(text, string(bytes), string([]byte("go")))

	var empty = []byte("")
	println(len(empty), len(string(empty)))
}
