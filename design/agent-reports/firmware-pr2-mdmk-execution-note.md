# Firmware PR2 — execution note (BCH-validated md1/mk1 engraving)

- **Date:** 2026-06-16
- **Where:** fork `/scratch/code/shibboleth/seedhammer`, branch `feat/engrave-mdmk` off `upstream/main`. Commits `6000f99` (codex32 verifier) + `9ca8661` (gui scanner + engrave).
- **Process note:** subagent/agent **dispatch** was degraded during this work (repeated 500s on the Agent API while direct Bash/Read/Edit/Write + `go test` worked fine). So Task 2's per-phase review was done as a careful **inline self-review** (below), and Tasks 3-4 were implemented inline rather than subagent-driven. The plan itself had already cleared plan-R0→R1 GREEN.

## What was built
- **`codex32/mdmk.go`** — `ValidMD`/`ValidMK`, reusing codex32's private `engine` + `newShortChecksum()/newLongChecksum().generator`, with our NUMS target residues and the `POLYMOD_INIT=0x23181b3` initial residue. `ValidMK` selects regular(13)/long(15) by data-part length matching mk-codec `bch_code_for_length` (14–93 regular, 94–95 reserved, 96–108 long). TinyGo-safe (uint64 only).
- **`codex32/mdmk_test.go`** — parity test vs **Rust-sourced** golden vectors: md1 regular, mk1 regular (dp 77), mk1 **long** (dp 108, from mk-codec `v0.1.json`), plus negatives (single-char tamper rejected — pure verify, no correction; all-zeros rejected; wrong-HRP rejected; length-bracket rejected; no-panic on malformed input).
- **`gui/scan.go`** — md1/mk1 branch after `codex32.New` (which correctly rejects them: md1 is below codex32's 48-char floor; mk1 fails codex32's checksum) → new `mdmkText` type.
- **`gui/gui.go`** — `validateMdmk` + `mdmkFlow` (mirror `validateDescriptor`/`descriptorFlow`) → TEXT+QR / TEXT / QR-ONLY plates; `engraveObjectFlow` `case mdmkText`.

## Discovery beyond the plan reviews
plan-R0/R1 caught the MK_LONG 75-bit `uint64` overflow but **missed that `MK_REGULAR_CONST=0x1062435f91072fa5c` is 65 bits** — it also overflows uint64 and needs a hi/lo split (`hi=0x1, lo=0x62435f91072fa5c`). The implementer caught this while writing mdmk.go; the parity test (mk1 regular passing) confirms the fix.

## Self-review of the BCH verifier (per-phase gate, inline)
Verified against source: (a) **no panic on malformed input** — `splitHRP` uses `strings.Cut` (no-separator → `("",s)`), `feFromRune` returns `(0,false)` on invalid chars, `inputChar` returns errors; `verifyMDMK` always returns `false`, never panics (confirmed by `TestMDMKNoPanicOnMalformed`). (b) **length gate** exactly matches mk-codec `bch_code_for_length`; md1 has no upper bound, matching md-codec `unwrap_string`. (c) Mixed case rejected via `setCase`. Minor (acceptable): mdmk doesn't tolerate visual separators (whitespace/`-`) that md-codec accepts for typed input — fine for the canonical NFC-scanned string. **Verdict: no Critical/Important.**

## Verification
`go test ./codex32/ ./gui/` green (codex32 incl. 6 TestMDMK cases; gui incl. TestScan md1/mk1, TestMdmkEngrave, TestMdmkOversizeRejected, no regression); `go vet` + `gofmt` clean.

## Status / open
- NOT pushed; no PR opened — awaiting user confirmation (outward-facing, third-party repo) and the author/DCO-sign-off rewrite to **Brian Goss** (both commits currently signed `bg`), as done for PR1.
- A formal subagent **final architect review** of the whole PR2 diff is pending Agent-API recovery; the inline self-review + parity test + full-suite green stand in the interim.
- mk-long path is covered by a real Rust-sourced long vector (dp 108).
