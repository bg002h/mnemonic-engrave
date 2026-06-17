package main

import "fmt"

// version is set at build time via -ldflags "-X main.version=<semver>".
var version string

func main() { fmt.Println("me-preview", version) }
