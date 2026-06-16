# Converter-polish (commit `5086119`) — formal architect review — R0

- **Stage:** deferred formal whole-diff review of the converter-polish cycle (the `firmware-deferred-formal-reviews` FOLLOWUP item (b)). The polish cycle had shipped with only an inline self-review during the 2026-06-16 Agent-API outage; agents recovered, so this is the formal pass.
- **Date:** 2026-06-16
- **Reviewer:** opus `feature-dev:code-reviewer` (adversarial)
- **Scope:** commit `5086119` ("polish(me): clear the 5 converter FOLLOWUP nits (v0.1.1)"), already merged on `master`. Reviewed current source: `crates/me-cli/src/{main.rs,ndef.rs,validate.rs,lib.rs,classify.rs}`, `crates/me-cli/tests/{cli.rs,cross_lang.rs}`, `firmware/ndef-roundtrip/main.go`, `design/SPEC_seedhammer_engrave.md` §5, plus the SeedHammer reference `nfc/ndef/ndef.go`.
- **Verdict:** **NOT-GREEN — 0 Critical / 1 Important** (I-1) + 1 Nit (N-1).

---

## VERBATIM REVIEW OUTPUT

The `RecordReader.Read` can return `(n > 0, nil)` at line 112 when `r.length > 0` (there's more payload to read). In the Go harness, `buf` is 4096 bytes and the NDEF payload is small (< 255 bytes per the 1-byte TLV form used by `me`), so `r.r.Read(buf[:l])` with the in-memory byteReader will return all remaining bytes at once. `r.length` becomes 0, taking the `return n, io.EOF` path, not the `return n, nil` path. But if the record payload were exactly 4096 bytes, then `r.length` might not reach 0 in one read, and we'd get `(n > 0, nil)`. With the harness's fixed 4096-byte buffer and short NDEF records, this path is not reached in practice — but the loop correctly handles it by appending to `out` and continuing.

The `(0, nil)` case: `RecordReader.Read` returns `(n, nil)` only at line 112, where `r.length > 0` and the underlying `r.r.Read(buf[:l])` returned `n` bytes with no error. Since `l = min(len(buf), r.length)` and `l > 0` (because `r.length > 0` and `len(buf) == 4096`), and the `byteReader` returns `n = copy(p, r.b[r.pos:])` which is > 0 as long as there's data... the byteReader cannot return `0, nil`. So `(0, nil)` from `RecordReader.Read` cannot happen in this harness. The loop is correct and not infinite-loop-prone for this usage.

Now let me write the formal review.

---

# Formal Adversarial Code Review — commit `5086119` ("polish(me): clear the 5 converter FOLLOWUP nits (v0.1.1)")

Reviewed files: `crates/me-cli/src/main.rs`, `crates/me-cli/src/ndef.rs`, `crates/me-cli/src/validate.rs`, `crates/me-cli/src/lib.rs`, `crates/me-cli/src/classify.rs`, `crates/me-cli/tests/cli.rs`, `crates/me-cli/tests/cross_lang.rs`, `firmware/ndef-roundtrip/main.go`, `design/SPEC_seedhammer_engrave.md` §5, `design/SPEC_me_converter_polish.md`, and the SeedHammer reference `nfc/ndef/ndef.go`.

---

## Per-nit verdict

**Nit 1 — `me-validate-ms-unreachable`** (`validate.rs:53`): CORRECTLY ADDRESSED. `unreachable!("ms1 is refused before validation")` is in place. `validate::validate` is called only from `lib.rs:59`, which is preceded by the explicit `Format::Ms` guard at `lib.rs:56-58`. The unit tests in `validate.rs` never pass `Format::Ms`. The arm is provably unreachable from all call sites.

**Nit 2 — `me-decode-text-tlv-comment`** (`ndef.rs:65-70`): CORRECTLY ADDRESSED. The doc comment accurately states the decoder handles only the 1-byte TLV length form and does not check the `0xFE` terminator, giving the correct reason (exists solely for round-trip self-test against `me`'s own bounded output). This matches what the code does: `bytes[1] as usize` — 1-byte length unconditionally — and no terminator check. The comment neither over- nor under-claims.

**Nit 3 — `me-go-harness-shortread-loop`** (`firmware/ndef-roundtrip/main.go:22-33`): CORRECTLY ADDRESSED with no infinite-loop risk for this usage. The `RecordReader.Read` contract per `ndef.go:95-229` is: returns `(n > 0, nil)` when payload is partially consumed, `(n, io.EOF)` when the record is fully consumed. The harness `byteReader` (in-memory, bounded) cannot produce `(0, nil)` because `copy` returns `> 0` whenever `pos < len(b)`. The loop accumulates correctly and exits cleanly on `io.EOF`.

**Nit 4 — `me-in-stdin-intermediate-zeroize`** (`main.rs:51-63`): PARTIALLY ADDRESSED — see Important issue below. The stdin path is fully correct: `stdin().read_to_string(&mut input)` writes directly into the `Zeroizing<String>` internal buffer; Zeroizing's `DerefMut` provides `&mut String`; no second copy. The `--in` path uses `*input = s`: `std::fs::read_to_string` returns `Ok(s)` (a freshly allocated `String`), which is then moved into the Zeroizing wrapper via `DerefMut` assignment (the old empty String is dropped trivially, then `s` becomes the wrapper's payload). This is correct for the primary input buffer. However, a secondary un-zeroized copy is created when `--echo` is active — see Important issue.

**Nit 5 — `me-canonical-string-stderr`** (`main.rs:68-74`, `main.rs:94-96`, `cli.rs:37-59`): CORRECTLY ADDRESSED for the primary requirements. `--echo` is opt-in (default false). Output goes to `eprintln!` (stderr). `echo_line` is only printed after `convert()` succeeds — ms1 and all error paths `return` before line 94. The spec §5 has been amended ("only when `--echo` is given"). However, the echo_line allocation is created pre-`convert()`, creating an un-zeroized copy on all paths including the ms1 refusal path — see Important issue.

---

## Issues

### Critical

None.

### Important

**I-1: Un-zeroized copy of input created when `--echo` is active, survives on the refusal path**

File: `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/main.rs`, lines 68-74

The `echo_line` allocation is constructed unconditionally (when `--echo` is set) before `convert()` is called:

```
line 68: let echo_line = if cli.echo {
line 69:     let s = input.trim();    // no alloc — &str borrow
line 71:     Some(format!("me: validated {label}: {s}"))   // HEAP ALLOCATION containing input content
line 72: } else { None };
line 76: let result = convert(&input);
line 77: drop(input);   // zeroizes the primary buffer
...
line 81: Err(ConvertError::RefusedSecret) => {
line 83:     return EXIT_REFUSED;     // echo_line dropped here, un-zeroized
```

When `--echo` is passed with an ms1 input: `echo_line` is `Some(String)` containing the ms1 string. The primary `input` buffer is zeroized at line 77. Then `result` is `Err(RefusedSecret)`, causing `return EXIT_REFUSED` at line 83. The `echo_line` `Some(String)` is dropped by normal Rust drop when `run()`'s stack frame unwinds — without zeroization. The ms1 secret content remains in heap memory until the allocator reclaims or overwrites it.

This directly undermines the stated defense-in-depth goal of nit 4: "input is now read into a `zeroize::Zeroizing<String>` so `read_to_string`'s allocation is scrubbed on drop." The scrub is bypassed by a second heap copy.

The same gap applies for md1/mk1 (public content, lower risk) on all return paths before `echo_line` is printed.

**Why it matters:** The spec explicitly calls out the `--in` flag as the threat vector ("the tool refuses ms1, but `--in` could be pointed at a secret"). With `--echo` set and `--in` pointing at an ms1 file, the ms1 content lives un-zeroized in `echo_line` after `drop(input)`. This is not a theoretical risk — `--echo` could easily be combined with `--in <ms1-file>` in a script, and the refusal path is a common code path.

**Fix:** Move the `echo_line` construction to after the `convert()` result is known, and only populate it on success. This ensures the un-zeroized copy is never created on refusal/error paths:

```rust
let result = convert(&input);
drop(input); // Zeroizing scrubs the input buffer here

let bytes = match result {
    Ok(b) => b,
    Err(ConvertError::RefusedSecret) => { ... return EXIT_REFUSED; }
    Err(e) => { ... return EXIT_INVALID; }
};

// Only reaches here on success (md1/mk1); echo_line contains only public content.
let echo_line = if cli.echo {
    // bytes already contain the encoded string; alternatively, pass the
    // trimmed string out of convert() or re-extract from the NDEF bytes.
    // Simpler: keep a Zeroizing<String> for echo that is scrubbed after use.
    ...
};
```

The cleanest fix without restructuring `convert()`'s return type: compute the echo string from `ndef::decode_text_tlv(&bytes)` post-success (which is already available for the round-trip test), or have `convert()` return the validated string alongside the bytes. Either way, the echo allocation must only be created after `convert()` returns `Ok`.

If the project's position is that the echo string contains only public (md1/mk1) content (since ms1 never converts), then the zeroize gap is real but the secret-exposure risk is nil — ms1 never completes `convert()` successfully, so the echo allocation will never contain secret material. That reasoning is sound given the invariant that ms1 is refused in `convert()`. However, the code as written creates the un-zeroized copy **before** that invariant is enforced, which violates the principle of least surprise and leaves a landmine: if `convert()`'s refusal guard were ever removed or moved, the echo allocation would silently carry ms1 un-zeroized.

**Confidence: 85.** The issue is real. Whether it rises to Important vs. Minor depends on how strictly the project treats "defense-in-depth" — the code comment specifically says "defense-in-depth on top of the ms1 refusal." Given the explicit goal of nit 4 and the fact that the gap exists precisely on the ms1 refusal path, it is Important.

### Minor

None.

### Nit

**N-1: `echo_prints_validated_string_to_stderr` test does not verify stdout is unaffected**

File: `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/tests/cli.rs`, lines 37-47

The test asserts that stderr contains the validated string but does not assert that stdout is unaffected (i.e., that the echo line does not bleed onto stdout). The spec and design invariant "stdout is binary/encoded NDEF only; all human/guidance text goes to stderr" is critical. Given the current implementation uses `eprintln!` (correct), this is a documentation/test coverage gap rather than a bug — but a test that checks `stdout.contains(MD1_VALID)` is false would make the invariant explicit and catch any future regression where `eprintln!` is accidentally changed to `println!`.

**Confidence: 80.** Real gap in test coverage of an explicitly stated invariant.

---

## Assessment

**Zeroizing correctness:** The `*input = s` pattern on the `--in` path is sound: `s` is moved into the Zeroizing wrapper (no heap copy survives outside it), the old empty `String` is dropped trivially. `input.trim()` is a borrow, not an allocation. `convert(&input)` and `exceeds_plate_budget(&input)` take `&str` references — they do not copy the input string to heap. The only un-zeroized heap copy is the `echo_line` String when `--echo` is active, described in I-1.

**`--echo` ms1 and refusal-path safety:** `echo_line` is never printed on the ms1 refusal path (the `return EXIT_REFUSED` at line 83 fires first). So ms1 is never echoed to stderr or stdout. The `--echo` stdout invariant holds: `eprintln!` at line 95 writes to stderr exclusively. The label logic (`starts_with("mk1")`) is safe: it can only fire after `convert()` succeeds, and every successfully-converted string has either an "mk1" or "md1" prefix (guaranteed by `classify()`). A non-md/non-mk string cannot reach this branch.

**Go short-read loop:** The loop idiom is idiomatic and correct for the `io.Reader` contract. The underlying `byteReader` is bounded in-memory, making `(0, nil)` impossible in practice. The loop correctly handles the `RecordReader.Read` semantics where `(n, io.EOF)` is returned when the record payload is exhausted (a valid simultaneous data+EOF delivery per Go's `io.Reader` contract).

**`unreachable!` arm:** Provably correct. `validate::validate` has exactly one non-test caller (`lib.rs:59`), which is guarded by the `Format::Ms` check at `lib.rs:56-58`.

**Doc comment accuracy:** The comment on `decode_text_tlv` is accurate. The decoder does not check `0xFE` (confirmed by reading the code) and uses only the 1-byte length form (line 75: `bytes[1] as usize`). The scope limitation is correctly stated.

**Spec↔impl reconcile:** SPEC_seedhammer_engrave.md §5 now reads "only when `--echo` is given" — confirmed at line 78 of the spec. The implementation matches.

**Test gaps:** The `echo_prints_validated_string_to_stderr` test does not assert stdout cleanliness for the `--echo` case (nit N-1). There is no test exercising `--echo` with an mk1 string (only md1 is tested), but since the label logic is simple and the md1 path exercises the core machinery, this is a low-severity gap outside the scope of the current review's confidence threshold.

---

## Verdict: NOT-GREEN (0 Critical / 1 Important)

**1 Important issue (I-1):** When `--echo` is active, the input string is copied into an un-zeroized heap allocation (`echo_line`) before `convert()` is called, and that allocation is dropped without scrubbing on the ms1-refusal path (and all error paths). The primary `Zeroizing<String>` buffer is correctly scrubbed, but the echo copy escapes the zeroize discipline. The fix is to move the `echo_line` construction to after `convert()` returns `Ok(...)` — at which point the input content is confirmed to be public (md1/mk1) and the ms1-refusal path cannot reach the allocation.

---

## Fold plan (main session)
- **I-1 (Important)** — fold inline: build `echo_line` ONLY after `convert()` returns `Ok`, and make it a `Zeroizing<String>` for belt-and-suspenders (so even the public-string copy is scrubbed and a future reordering can't leak ms1). Restructure so the `--echo` allocation is unreachable on the refusal/error paths.
- **N-1 (Nit)** — fold inline (cheap, hardens the load-bearing stdout-purity invariant): add a `stdout` assertion to `echo_prints_validated_string_to_stderr` confirming the validated string never lands on stdout.
- Re-dispatch R1 for convergence.
