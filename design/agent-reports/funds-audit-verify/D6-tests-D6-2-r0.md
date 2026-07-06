# Adversarial verification — D6-2 (Cross-language differential tests silently pass as green no-ops when Go is absent)

Verifier: adversarial verifier #0
Date: 2026-07-06
Verdict: **CONFIRMED** (not refuted), severity **important** (unchanged), confidence **high**.

## The claim

`crates/me-cli/tests/cross_lang.rs` and `crates/me-cli/tests/preview_cross_lang.rs`
`return` early (counted as a PASS) when `go` is not on PATH, so the strongest
differential guard — does SeedHammer's real Go NDEF reader / real sidecar parse
what `me` emits — is a vacuous green no-op unless Go is present *and someone runs
it*, and it covers only one md1 string.

## Evidence — the code behaves exactly as cited

`cross_lang.rs:11-14` (the cited location):

```rust
if Command::new("go").arg("version").output().is_err() {
    eprintln!("skipping cross-language round-trip: `go` is not on PATH");
    return;                       // <-- early return, test counts as PASS
}
```

`preview_cross_lang.rs:82-85` (the sibling):

```rust
if !go_available() {              // go_available() == StdCommand::new("go")...output().is_ok()
    eprintln!("skipping cross-language preview round-trip: `go` is not on PATH");
    return;                       // <-- same pattern, counts as PASS
}
```

Neither test uses `#[ignore]`, a `panic!`, or any hard-require gate. A skip is
indistinguishable from a real pass in `cargo test` output.

Coverage of `cross_lang.rs` is a single hard-coded input:
`let input = "md1yqpqqxqq8xtwhw4xwn4qh";` (one 24-char md1). `preview_cross_lang.rs`
drives md1 + 2 mk1 chunks but only asserts *structural* presence (`<svg`, `<path`,
ms1-never-rendered) — it does not assert the SVG encodes the correct string. So the
"real NDEF reader parses me's bytes" property is exercised for exactly one md1.

## Live probe

`go` is not on the default PATH here (`which go` → exit 1; the binary exists only at
`/home/bcg/.local/go/bin`, off-PATH). Running the test with Go absent:

```
$ env -u GOROOT PATH="$(getconf PATH)" cargo test -p mnemonic-engrave --test cross_lang -- --nocapture
running 1 test
skipping cross-language round-trip: `go` is not on PATH
test rust_ndef_parses_in_seedhammer_go_reader ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

This is the finder's "verified live" reproduction, reproduced independently: the
differential test prints "skipping…" and reports `1 passed` — a green no-op.

## Failure-scenario reachability

- **Dev-local (live today):** the current CI (`.github/workflows/release.yml`) runs
  only `cargo build` / `go build`; grep shows no `cargo test`/`go test` (the sole
  `test` hit at line 92 is a shell string-compare of the sidecar version, not a test
  runner). So today the only place these tests run is a developer's machine — and any
  dev without Go on PATH gets a fully-green suite while the strongest device-parity
  guard never executed. Reachable now.
- **Future-CI (the finding's stated scenario):** if D6-1 is fixed by adding a
  cargo-test job whose runner lacks Go (a very easy misconfiguration — the report's
  own P2 has to *specifically* warn "put Go and Rust in the SAME job"), the
  differential tests report green while never touching SeedHammer's real reader.
  Reachable.

## Does another layer already prevent the underlying risk?

Partially, not fully.
- `golden.rs::md1_short_matches_golden` (checked-in `vectors/md1-short.ndef`) and
  `ndef::encodes_expected_bytes` pin me's output bytes and would catch an encoder
  *regression* that changes the emitted bytes. That covers "me's output drifted from
  a known-good vector."
- What they do **not** cover, and only the differential test does: whether a *new or
  changed* NDEF shape that me and its golden both agree on is actually parseable by
  the device's real reader (`third_party/seedhammer/nfc/ndef/ndef.go`). The report's
  own D6-4 (the `>=0xFF` short-record boundary that the real reader misparses via the
  2-byte-length escape at ndef.go:73) is precisely a divergence the golden cannot see
  but the differential reader would — and that guard silently vanishes when Go is
  absent. So the device-parity property is genuinely under-guarded, not redundantly
  covered.

## Severity assessment

The finding is a test-adequacy (meta) gap, not a live code defect: it does not by
itself produce a wrong-but-accepted plate. Its funds-relevance is indirect —
a silently-vanishing differential guard lets a *future* Rust-vs-device NDEF
divergence ship undetected. That indirect impact is real, forward-looking, and
compounds directly with D6-4. Given this project's explicit posture that
device-parity / cross-language guards are load-bearing (the whole reason the
`me-preview` sidecar reuses upstream curve math and the submodule is pinned) and its
stated wariness of "plausible-but-passing" false-green (the "1 valid last word"
class in CLAUDE.md), rating the silent-skip anti-pattern **important** is honest and
consistent with the finder's "severity = severity of the gap" convention. I keep
severity at **important** (no downward adjustment); a case for **moderate** exists
because the golden byte-anchor mitigates encoder regressions and there is no
immediate live-funds path, but the unique, unmitigated device-readability coverage
that the skip erases tips it to important.

## Verdict

Not refuted. The cited code at `cross_lang.rs:11` behaves exactly as claimed
(confirmed live), `preview_cross_lang.rs:82` shares the pattern, the failure
scenario is reachable (dev-local today, future-CI as stated), and no other layer
fully substitutes for the erased device-parity coverage. CONFIRMED, important, high
confidence.
