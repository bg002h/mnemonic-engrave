# Converter Polish (v0.1.1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clear the 5 converter FOLLOWUP nits (`unreachable!`, a doc comment, a Go short-read loop, `Zeroizing` reads, and an opt-in `--echo` flag) in one PATCH cycle.

**Architecture:** Small, mostly-mechanical edits to the existing `me` crate + the Go round-trip harness. Only `--echo` adds behavior (a success-path stderr line behind a new flag); the rest are behavior-neutral (verified by the existing suite + clippy/gofmt staying green).

**Tech Stack:** Rust (`me` crate, clap, zeroize), Go (round-trip harness). `go` at `~/.local/go/bin` — prefix go commands with `export PATH="$HOME/.local/go/bin:$PATH"`.

> **Spec:** `design/SPEC_me_converter_polish.md` (user-approved); recon `cycle-prep-recon-me-converter-nits.md`. Work in this repo (`mnemonic-engrave`) on a branch off `master`.
>
> **Plan status:** R0 gate **GREEN (0C/0I)** via inline self-review (`design/agent-reports/me-converter-polish-plan-R0-selfreview.md`) — the formal opus-architect subagent R0 was deferred because Agent-API dispatch was failing (500s/529) all session; execution's `cargo build`/`clippy`/`go vet` validate the compile-correctness risks empirically.

---

## File Structure

| File | Change |
|---|---|
| `crates/me-cli/src/validate.rs` | `panic!` → `unreachable!` (1 line). |
| `crates/me-cli/src/ndef.rs` | doc comment on `decode_text_tlv`. |
| `crates/me-cli/src/main.rs` | `Zeroizing` input reads; new `--echo` flag + success-path stderr line. |
| `crates/me-cli/tests/cli.rs` | tests for `--echo` on/off. |
| `firmware/ndef-roundtrip/main.go` | short-read loop. |
| `design/SPEC_seedhammer_engrave.md` | §5 line: canonical string to stderr **only with `--echo`**. |

---

## Task 1: Branch

- [ ] **Step 1:** `cd /scratch/code/shibboleth/mnemonic-engrave && git checkout master && git checkout -b fix/converter-polish`. Expected: on `fix/converter-polish`.

---

## Task 2: `me-validate-ms-unreachable`

**Files:** Modify `crates/me-cli/src/validate.rs:53`.

- [ ] **Step 1: Change the panic to unreachable.** Replace:
```rust
        Format::Ms => panic!("validate() called on ms1 — must be refused before validation"),
```
with:
```rust
        Format::Ms => unreachable!("ms1 is refused before validation"),
```

- [ ] **Step 2: Build + lib tests (no behavior change).** Run: `cargo test -p mnemonic-engrave --lib && cargo clippy --all-targets -- -D warnings`. Expected: pass, 0 warnings.

- [ ] **Step 3: Commit.** `git add crates/me-cli/src/validate.rs && git commit -s -m "fix(me): unreachable! instead of panic! on the refused-ms1 arm"`

---

## Task 3: `me-decode-text-tlv-comment`

**Files:** Modify `crates/me-cli/src/ndef.rs` (the `decode_text_tlv` doc comment at `:64-66`).

- [ ] **Step 1: Extend the doc comment.** The current comment above `pub fn decode_text_tlv` reads:
```rust
/// Minimal decoder mirroring SeedHammer's reader: unwrap the NDEF TLV, parse a
/// single well-known Text record, return the UTF-8 text. Used for the
/// round-trip self-test; `None` on any structural mismatch.
```
Replace it with:
```rust
/// Minimal decoder mirroring SeedHammer's reader: unwrap the NDEF TLV, parse a
/// single well-known Text record, return the UTF-8 text. Used for the
/// round-trip self-test; `None` on any structural mismatch.
///
/// Intentionally handles only the 1-byte TLV length form and does NOT check the
/// `0xFE` terminator — it only needs to round-trip `me`'s own bounded output,
/// which never uses the 3-byte length form. Not a general-purpose NDEF parser.
```

- [ ] **Step 2: Build + tests.** Run: `cargo test -p mnemonic-engrave --lib ndef`. Expected: pass.

- [ ] **Step 3: Commit.** `git add crates/me-cli/src/ndef.rs && git commit -s -m "docs(me): note decode_text_tlv's intentional 1-byte-TLV scope"`

---

## Task 4: `me-in-stdin-intermediate-zeroize`

**Files:** Modify `crates/me-cli/src/main.rs`.

- [ ] **Step 1: Switch the import.** Change line 8 `use zeroize::Zeroize;` to:
```rust
use zeroize::Zeroizing;
```

- [ ] **Step 2: Read into a Zeroizing buffer + scrub on drop.** Replace the block at lines 44-62:
```rust
    let mut input = String::new();
    if let Some(path) = &cli.r#in {
        match std::fs::read_to_string(path) {
            Ok(s) => input = s,
            Err(e) => {
                eprintln!("me: cannot read {}: {e}", path.display());
                return EXIT_USAGE;
            }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    // Capture the plate-budget flag before zeroizing the input buffer.
    let too_long = mnemonic_engrave::exceeds_plate_budget(&input);

    let result = convert(&input);
    input.zeroize(); // scrub the input buffer regardless of outcome
```
with:
```rust
    // Read into a Zeroizing buffer so the input (incl. read_to_string's
    // allocation, which a secret could reach via --in) is scrubbed on drop —
    // defense-in-depth on top of the ms1 refusal.
    let mut input = Zeroizing::new(String::new());
    if let Some(path) = &cli.r#in {
        match std::fs::read_to_string(path) {
            Ok(s) => *input = s, // moves the buffer into the Zeroizing wrapper
            Err(e) => {
                eprintln!("me: cannot read {}: {e}", path.display());
                return EXIT_USAGE;
            }
        }
    } else if let Err(e) = std::io::stdin().read_to_string(&mut *input) {
        eprintln!("me: cannot read stdin: {e}");
        return EXIT_USAGE;
    }

    // Capture the plate-budget flag before the input is dropped.
    let too_long = mnemonic_engrave::exceeds_plate_budget(&input);

    let result = convert(&input);
    drop(input); // Zeroizing scrubs the input buffer here
```

- [ ] **Step 3: Build + full suite (behavior-neutral).** Run: `cargo test -p mnemonic-engrave && cargo clippy --all-targets -- -D warnings`. Expected: all pass, 0 warnings (note: `convert(&input)` / `exceeds_plate_budget(&input)` rely on deref-coercion `&Zeroizing<String>` → `&str`, which compiles).

- [ ] **Step 4: Commit.** `git add crates/me-cli/src/main.rs && git commit -s -m "fix(me): read input into a Zeroizing buffer (scrub intermediate copies)"`

---

## Task 5: `me-canonical-string-stderr` — opt-in `--echo`

**Files:** Modify `crates/me-cli/src/main.rs`, `crates/me-cli/tests/cli.rs`, `design/SPEC_seedhammer_engrave.md`.

- [ ] **Step 1: Write the failing tests.** Add to `crates/me-cli/tests/cli.rs` (it already defines `const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";`):
```rust
#[test]
fn echo_prints_validated_string_to_stderr() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .args(["--hex", "--echo"])
        .write_stdin(MD1_VALID)
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("validated md1:"), "stderr: {stderr}");
    assert!(stderr.contains(MD1_VALID), "stderr: {stderr}");
}

#[test]
fn no_echo_by_default() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .args(["--hex"])
        .write_stdin(MD1_VALID)
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains("validated"), "unexpected echo: {stderr}");
}
```

- [ ] **Step 2: Run — expect FAIL.** Run: `cargo test -p mnemonic-engrave --test cli echo`. Expected: FAIL (`--echo` is an unknown arg / clap error, or no "validated" line).

- [ ] **Step 3: Add the `--echo` flag to the Cli struct.** In `crates/me-cli/src/main.rs`, add after the `base64` field (before the closing `}` of `struct Cli`):
```rust
    /// On success, echo the validated md1/mk1 string to stderr (for pasting
    /// into a phone NFC-writer app). Off by default.
    #[arg(long)]
    echo: bool,
```

- [ ] **Step 4: Capture + print the echo line on success.** In `run()`, immediately after the `let too_long = ...` line and BEFORE `let result = convert(&input);`, capture the echo string (before the input is dropped):
```rust
    let echo_line = if cli.echo {
        let s = input.trim();
        let label = if s.starts_with("mk1") { "mk1" } else { "md1" };
        Some(format!("me: validated {label}: {s}"))
    } else {
        None
    };
```
Then, after the error-handling `match` (i.e. on the success path, right after the `if too_long { ... }` block), add:
```rust
    if let Some(line) = echo_line {
        eprintln!("{line}");
    }
```
(`echo_line` is only printed here, which is reached only on success — `RefusedSecret`/invalid inputs `return` from the match above, so `ms1`/errors never echo.)

- [ ] **Step 5: Run — expect PASS.** Run: `cargo test -p mnemonic-engrave --test cli`. Expected: all CLI tests pass (incl. the 2 new + the existing 3).

- [ ] **Step 6: Amend spec §5.** In `design/SPEC_seedhammer_engrave.md` line 78, change the parenthetical so it reads that the canonical validated string is echoed to stderr **only when `--echo` is given**. Replace:
```
the canonical validated string (for pasting into a phone NFC-writer app) and any guidance go to **stderr**, so binary output and human text never collide on the same stream.
```
with:
```
any guidance goes to **stderr**, never stdout, so binary output and human text never collide; the canonical validated string is echoed to stderr **only when `--echo` is given**.
```

- [ ] **Step 7: Commit.** `git add crates/me-cli/src/main.rs crates/me-cli/tests/cli.rs design/SPEC_seedhammer_engrave.md && git commit -s -m "feat(me): add opt-in --echo to print the validated string to stderr"`

---

## Task 6: `me-go-harness-shortread-loop`

**Files:** Modify `firmware/ndef-roundtrip/main.go:21-27`.

- [ ] **Step 1: Loop the record read.** Replace lines 21-27:
```go
	buf := make([]byte, 4096)
	n, err := rr.Read(buf)
	if err != nil && err != io.EOF {
		fmt.Fprintln(os.Stderr, "ndef:", err)
		os.Exit(1)
	}
	os.Stdout.Write(buf[:n])
```
with:
```go
	var out []byte
	buf := make([]byte, 4096)
	for {
		n, err := rr.Read(buf)
		out = append(out, buf[:n]...)
		if err == io.EOF {
			break
		}
		if err != nil {
			fmt.Fprintln(os.Stderr, "ndef:", err)
			os.Exit(1)
		}
	}
	os.Stdout.Write(out)
```

- [ ] **Step 2: Cross-language round-trip still passes.** Run: `export PATH="$HOME/.local/go/bin:$PATH" && cargo test -p mnemonic-engrave --test cross_lang`. Expected: PASS (the round-trip recovers the md1 string via the Go harness).

- [ ] **Step 3: Commit.** `git add firmware/ndef-roundtrip/main.go && git commit -s -m "fix(ndef-roundtrip): read the NDEF record in a short-read loop"`

---

## Task 7: Full gate

- [ ] **Step 1: Rust gate.** Run: `export PATH="$HOME/.local/go/bin:$PATH" && cargo test -p mnemonic-engrave && cargo clippy --all-targets -- -D warnings && cargo fmt --all --check`. Expected: all tests pass (incl. `cross_lang` with go present), 0 clippy warnings, fmt clean.
- [ ] **Step 2: Go gate.** Run: `cd firmware/ndef-roundtrip && export PATH="$HOME/.local/go/bin:$PATH" && go vet . && gofmt -l main.go`. Expected: no vet output, gofmt clean (no files listed).

---

## Self-Review

- **Spec coverage:** §Scope 1 → Task 2 ✓; 2 → Task 3 ✓; 3 → Task 6 ✓; 4 → Task 4 ✓; 5 (`--echo` + §5 amendment) → Task 5 ✓. Non-goals (no validation/wire/exit-code/other-flag change) respected.
- **Placeholder scan:** none — every step has complete code/commands.
- **Type/behavior consistency:** `--echo` field name matches the `cli.echo` use; `echo_line` captured before `drop(input)` (so it doesn't borrow a dropped value) and printed only on the success path; `Zeroizing<String>` derefs (`*input`, `&mut *input`, deref-coercion in `convert(&input)`) are consistent. `MD1_VALID` already exists in `cli.rs`.

## Open items
- The `--echo` label derives md1/mk1 from the validated string's prefix (`starts_with("mk1")`); convert() guarantees the string is a valid md1/mk1 on the success path, so the binary md1-vs-mk1 choice is sound.
- Task 6's loop is exercised by the existing `cross_lang` test only with a single-read payload; the loop's multi-read path isn't directly unit-tested (the harness is `#[ignore]`-free but Go-gated) — acceptable for a test harness.
