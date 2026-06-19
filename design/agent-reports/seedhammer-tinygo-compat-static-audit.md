# T7-program new code — static TinyGo-compat audit (PROXY for the pico-plus2 gate)

**Agent:** `abcb2760bd710aff9`. **HEAD:** `8eb51d7`. **Date:** 2026-06-19.
**Why a proxy:** the authoritative gate is `nix develop --command tinygo build -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` (`.github/workflows/test.yml:29`), but this environment has neither Nix nor TinyGo, and the bg002h fork's CI has never run (0 check-runs on HEAD; forked-repo Actions dormant until owner-enabled; `test.yml` has no `workflow_dispatch`). This is a static source audit + host-go cross-check — NOT a substitute for a real `tinygo build`.

## Bottom line
The new T1–T7 firmware-production code is **LIKELY TinyGo-clean** for `pico-plus2` (`-gc precise -scheduler tasks`). **Confidence: high** on compile, **moderate-to-high** on goroutine runtime (high because the new concurrency mirrors existing on-device code). Authoritative gate still required before a firmware TAG: a real `tinygo build -target pico-plus2 ./cmd/controller` (+ smoke flash) to (a) confirm the `tinygo,rp`-tagged device-graph subset and (b) exercise the scanner goroutines under the tasks scheduler on hardware.

## Risk findings
### Likely-break (would fail the tinygo build): NONE
No `reflect`, `encoding/json`, `encoding/gob`, `text/template`, `html/template`, `regexp`, `unsafe`, `syscall`, `os`, `io/ioutil`, or `crypto/rand` introduced anywhere in the new firmware-production code (md/, mk/, bundle/, bip85/, codex32/, new gui/ flows). Every grep match for those tokens was a comment. The only `unsafe` in the gui graph is `gui/assets/embed.go` (pre-existing, not T1–T7).

### Heavy-but-OK (compiles; bloat/perf)
- **Concurrency (3 new gui files):** `gui/md1_gather.go:89`, `gui/bundle_flow.go:107`, `gui/mk1_inspect.go:172` each spawn one scanner `go func()` over a buffered `chan scanResult` (cap 1), non-blocking `select`/`default`, cooperative `time.Sleep(1s)`. A verbatim clone of the pre-existing idiom in `gui/gui.go:1550–1608` (already shipping on-device). `-scheduler tasks`-compatible (cooperative yields, no unbuffered-blocking, no `time.After`/ticker). No new risk.
- **`fmt`:** all simple verbs (43×`%d`, 14×`%s`, 1×`%x`); zero `%v`/`%+v`/`%T`. `fmt` already in the device graph.

### Clear
- **Generics:** only `md/canonicalize.go:270 cloneSlice[T any]`, instantiated with concrete slice element types — trivially monomorphizable. The flagged "first firmware-production generic" (10a-M2) is fine.
- No `init()` in the new packages; the one package table (`mk/mk.go:291 standardPaths`) is a small static literal.
- Maps use trivial keys only; the one range-over-map building a `[]string` (`md1_gather.go:59`) is reordered downstream by `md.ExpandWalletPolicyChunks` re-parsing each chunk header — map-iteration nondeterminism irrelevant by construction.
- No defer-in-loop in any new package.

## Stdlib import-graph check
Host `go list -deps ./gui/` pulls `encoding/json` (←btcd chainhash), `text/template` (←btcd txscript), `net`/`net/url` (←btcd wire), `reflect`, `gonum` (←`seedhammer.com/bspline`, B-spline fit). **None originate in new T1–T7 code** — all pre-existing address/bip/curve-math deps already shipping. New code's own imports are conservative: `bytes`, `errors`, `fmt`, `encoding/binary`, `encoding/hex`, `crypto/hmac`, `crypto/sha512`, `strings`, `slices`, `sort`, btcsuite hdkeychain/chaincfg, seedhammer internal. (Caveat: host graph; the `tinygo,rp` device graph is a subset.)

## 32-bit / bit-packing scrutiny — clean
- **md `bitWriter`/`bitReader`/`reEmitBits`** (`md/bits.go`) use a `uint64` accumulator throughout; all shifts `value & ((uint64(1)<<uint(count))-1)`, `count≤64`, inner `chunk≤8`; call sites top out at `w.write(uint64(uint32(b)), 32)`. Nothing packs into `int`.
- **bip85** (`bip85/bip85.go`) does no shift/int math (just HMAC-SHA512). The `entLen` math is in the consumer `gui/bip85.go:79` `(words*11 - words/3)/8`, max 32, guarded to [16,32]∩mod4. `bip85.PathRoot` (0x84F4E468 > int32 max) is held only as `uint32` in a `[]uint32{}` literal passed to `hdkeychain.Derive(uint32)` — never narrowed to signed `int`.
- **codex32 65-bit math** `codex32/mdmk.go:68 unpackSyms(hi, lo uint64, ...)` reconstructs the 65-bit value across two `uint64` words; the `(lo>>shift)|(hi<<(64-shift))` is guarded by explicit `shift>=64`/`==0` cases (avoids undefined `>>64`). Correct two-word handling, not an int-width assumption.
- `mk/encode.go:162 byte(ParentFingerprint()>>24)` — `ParentFingerprint()` returns `uint32`, 32-bit-safe. Other shifts on explicit `uint32`/`byte`/u5, counts <32.

Host `go build ./...` = clean; `go test ./codex32/ ./md/ ./bip85/` = pass.
