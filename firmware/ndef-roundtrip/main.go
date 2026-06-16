// Reads NDEF bytes on stdin (TLV-wrapped, as `me` emits), parses them with
// SeedHammer's own reader, and prints the recovered text record body to stdout.
package main

import (
	"fmt"
	"io"
	"os"

	"seedhammer.com/nfc/ndef"
)

func main() {
	in, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintln(os.Stderr, "read:", err)
		os.Exit(1)
	}
	mr := ndef.NewMessageReader(byteReader(in))
	rr := ndef.NewRecordReader(mr)
	buf := make([]byte, 4096)
	n, err := rr.Read(buf)
	if err != nil && err != io.EOF {
		fmt.Fprintln(os.Stderr, "ndef:", err)
		os.Exit(1)
	}
	os.Stdout.Write(buf[:n])
}

// byteReader adapts a []byte to the io.Reader the ndef package expects.
func byteReader(b []byte) io.Reader { return &reader{b: b} }

type reader struct {
	b   []byte
	pos int
}

func (r *reader) Read(p []byte) (int, error) {
	if r.pos >= len(r.b) {
		return 0, io.EOF
	}
	n := copy(p, r.b[r.pos:])
	r.pos += n
	return n, nil
}
