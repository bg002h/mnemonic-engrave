# Verify D6-2 (round 1) — adversarial verifier #1

- Finding: D6-2 (important) — "Cross-language differential tests silently pass as green
  no-ops when Go is absent (verified live)"
- Location: `crates/me-cli/tests/cross_lang.rs:11` (+ `preview_cross_lang.rs:82`)
- Verdict: **CONFIRMED** (refuted = false), confidence **high**, severity **unchanged (important)**.

## What the claim asserts

`cross_lang.rs` and `preview_cross_lang.rs` `return` early — counted as a PASS — when
`go` is not on PATH. The single strongest funds-safety differential guard (does
SeedHammer's *real* Go NDEF reader parse what `me` emits? does the *real* sidecar render
public-only?) is therefore vacuous unless Go is present AND someone runs it, and even then
it covers only one md1 string.

## Evidence

### 1. The cited code behaves exactly as claimed

`crates/me-cli/tests/cross_lang.rs:9-14`:

```rust
#[test]
fn rust_ndef_parses_in_seedhammer_go_reader() {
    if Command::new("go").arg("version").output().is_err() {
        eprintln!("skipping cross-language round-trip: `go` is not on PATH");
        return;                 // <-- early return, NOT #[ignore], NOT a failure
    }
    ...
```

`crates/me-cli/tests/preview_cross_lang.rs:80-85` mirrors it:

```rust
#[test]
fn real_sidecar_renders_public_plates_only() {
    if !go_available() {        // go_available() = Command::new("go").arg("version").output().is_ok()
        eprintln!("skipping cross-language preview round-trip: `go` is not on PATH");
        return;
    }
    ...
```

A bare `return` from a `#[test]` fn is a **pass**, not a skip in the cargo sense (cargo has
no first-class "skipped-but-not-ignored" state for a body-level early return). So a Go-absent
run reports the test as passing.

### 2. `go` is absent in this environment

```
$ which go        -> "go not found", exit 1
$ go version      -> command not found (127)
```

### 3. Live confirmation — the test is a green no-op

```
$ cargo test -p mnemonic-engrave --test cross_lang -- --nocapture
running 1 test
skipping cross-language round-trip: `go` is not on PATH
test rust_ndef_parses_in_seedhammer_go_reader ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

This exactly reproduces the finder's quoted output ("skipping cross-language round-trip:
`go` is not on PATH" then "test result: ok. 1 passed"). The strongest differential guard ran
zero assertions and was counted as green.

### 4. No other layer fully substitutes for the differential guard

The only overlapping coverage is `tests/golden.rs::md1_short_matches_golden`, which compares
`convert("md1yqpqqxqq8xtwhw4xwn4qh")` byte-for-byte against the static
`tests/vectors/md1-short.ndef`. That golden was itself produced by `me` (`convert(...)`), so
it is a *self*-reference: it catches accidental encoder drift away from the known-good bytes,
but it does **not** confirm that the emitted bytes are actually parseable by SeedHammer's real
`nfc/ndef` reader. The cross-lang test is the *only* place the real device-side reader
validates `me`'s output. If someone changes the encoder AND regenerates the golden to match,
`golden.rs` passes and only the cross-lang differential test would catch a
regenerated-but-device-incompatible divergence — and that test is a silent no-op without Go.

So the failure scenario ("CI added but runner lacks Go, or dev runs without Go → Rust-vs-device
NDEF divergence ships undetected") is genuinely reachable; the overlapping golden narrows but
does not close the exposure.

## Adversarial pushback considered

- *Is the exposure overstated?* Partly narrowed: the golden gives strong overlapping protection
  for the common case (accidental encoder regression → golden mismatch → hard fail). The unique
  residual the cross-lang test guards is "encoder changed AND golden deliberately regenerated,
  yet the new bytes are not device-parseable." That is narrower than "the strongest guard is
  vacuous" might imply. But it is still a real residual, and a silently-green test is an active
  false-confidence anti-pattern regardless.
- *Is this a live funds bug?* No. NDEF output is currently correct (the test passes for real
  when Go is present, per the finder's §2). This is a test-adequacy / defense-in-depth gap, and
  the D6 report explicitly scores "severity = severity of the GAP."
- *Severity honesty?* "important" is defensible: it concerns the single strongest funds-relevant
  differential guard (real device reader / real sidecar) being rendered vacuous, and it is
  correctly ranked below the critical D6-1 (no CI at all). It sits at the important/moderate
  boundary given the overlapping golden coverage and the contingent trigger, but I do not
  downgrade — a false-green differential test is precisely the kind of hole a funds audit should
  flag as important. Remediation is cheap and correct (finder's P2: `ME_REQUIRE_GO=1` → hard
  fail in CI; table-drive over more strings).

## Conclusion

The claim is concretely substantiated: the cited code returns early as a PASS on Go-absence,
verified live in this environment; the failure scenario is reachable; no other layer fully
substitutes for the real-device differential check. Confirmed at the stated severity.
