// Reads one descriptor line on stdin, parses it with SeedHammer's own
// nonstandard.OutputDescriptor (the function the device runs when it measures
// and canonicalises a descriptor), and prints the canonical string
// (bip380.Descriptor.Encode) to stdout.
package main

import (
	"fmt"
	"io"
	"os"
	"strings"

	"seedhammer.com/nonstandard"
)

func main() {
	in, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintln(os.Stderr, "read:", err)
		os.Exit(1)
	}
	line := strings.TrimRight(string(in), "\r\n")
	d, err := nonstandard.OutputDescriptor([]byte(line))
	if err != nil {
		fmt.Fprintln(os.Stderr, "parse:", err)
		os.Exit(1)
	}
	fmt.Print(d.Encode())
}
