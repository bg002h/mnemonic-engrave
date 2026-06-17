package main

import (
	"flag"
	"fmt"
	"io"
	"os"

	"seedhammer.com/engrave"
)

// version is set at build time via -ldflags "-X main.version=<semver>".
// Keep this as the -X target; do not remove it.
var version string

func main() {
	os.Exit(run(os.Args[1:], os.Stdin, os.Stdout, os.Stderr))
}

// run is the testable entry point. It returns a process exit code:
//
//	0 on success, 1 on a render/usage error.
func run(args []string, stdin io.Reader, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "usage: me-preview [--version] | render --format <svg|png> --out <FILE|-> [--mode <text+qr|text|qr>]")
		return 1
	}

	// Top-level --version (and -version) short-circuit.
	switch args[0] {
	case "--version", "-version":
		fmt.Fprintln(stdout, "me-preview", version)
		return 0
	case "render":
		return runRender(args[1:], stdin, stdout, stderr)
	default:
		fmt.Fprintf(stderr, "me-preview: unknown command %q\n", args[0])
		return 1
	}
}

func runRender(args []string, stdin io.Reader, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("render", flag.ContinueOnError)
	fs.SetOutput(stderr)
	mode := fs.String("mode", "", "force a plate mode (text+qr|text|qr); empty = auto-select best fitting")
	format := fs.String("format", "", "output format: svg or png")
	out := fs.String("out", "", "output file path, or - for stdout")
	if err := fs.Parse(args); err != nil {
		return 1
	}

	if *format != "svg" && *format != "png" {
		fmt.Fprintf(stderr, "me-preview: --format must be svg or png, got %q\n", *format)
		return 1
	}
	if *out == "" {
		fmt.Fprintln(stderr, "me-preview: --out is required (use - for stdout)")
		return 1
	}

	data, err := io.ReadAll(stdin)
	if err != nil {
		fmt.Fprintf(stderr, "me-preview: read stdin: %v\n", err)
		return 1
	}
	s := string(data)
	// Trim a single trailing newline so piped strings (e.g. `printf '...\n'`)
	// engrave the intended content.
	s = trimTrailingNewline(s)
	if s == "" {
		fmt.Fprintln(stderr, "me-preview: empty input string on stdin")
		return 1
	}

	var eng engrave.Engraving
	var chosenMode string
	if *mode != "" {
		e, err := engraveMode(*mode, s)
		if err != nil {
			fmt.Fprintf(stderr, "me-preview: %v\n", err)
			return 1
		}
		eng, chosenMode = e, *mode
	} else {
		e, m, err := engraveBest(s)
		if err != nil {
			fmt.Fprintf(stderr, "me-preview: %v\n", err)
			return 1
		}
		eng, chosenMode = e, m
	}

	var payload []byte
	switch *format {
	case "svg":
		payload = []byte(renderSVG(eng))
	case "png":
		p, err := renderPNG(eng)
		if err != nil {
			fmt.Fprintf(stderr, "me-preview: png encode: %v\n", err)
			return 1
		}
		payload = p
	}

	if err := writeOut(*out, payload, stdout); err != nil {
		fmt.Fprintf(stderr, "me-preview: write output: %v\n", err)
		return 1
	}

	// Report the chosen mode. When the payload goes to a real file, the mode
	// line goes to stdout (the contract the `me` Rust caller parses). When
	// `--out -` streams the payload itself to stdout, emit the mode line to
	// stderr instead so the stdout stream stays a clean, pipeable SVG/PNG.
	modeSink := stdout
	if *out == "-" {
		modeSink = stderr
	}
	fmt.Fprintln(modeSink, "mode", chosenMode)
	return 0
}

func writeOut(path string, payload []byte, stdout io.Writer) error {
	if path == "-" {
		_, err := stdout.Write(payload)
		return err
	}
	return os.WriteFile(path, payload, 0o644)
}

func trimTrailingNewline(s string) string {
	if n := len(s); n > 0 && s[n-1] == '\n' {
		s = s[:n-1]
		if n := len(s); n > 0 && s[n-1] == '\r' {
			s = s[:n-1]
		}
	}
	return s
}
