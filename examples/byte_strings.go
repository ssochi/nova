package main

func main() {
	var marker byte
	var word = "gopher"
	var first byte = word[0]
	var middle = word[1:4]
	var buf = make([]byte, len(word))
	var copied = copy(buf, word)
	println(marker, first, middle, copied, buf[2], len(middle))
	println("é"[0], "é"[1], len("é"))
}
