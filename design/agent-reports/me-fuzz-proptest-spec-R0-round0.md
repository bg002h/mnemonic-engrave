# R0 Architect Review — SPEC_me_fuzz_proptest.md (Cycle C / F18)

- **Artifact:** `design/SPEC_me_fuzz_proptest.md`
- **Worktree/branch:** `me-fuzz-proptest` @ `9fafb6b` (off mnemonic-engrave master)
- **Round:** 0
- **Reviewer role:** R0 architect gate (GREEN = 0 Critical / 0 Important before implementation)
- **Context:** F18 closes an INSURANCE gap (D6-7 + its refuting verdict found NO reachable
  panic / fund-losing misroute). The deliverable is proptest properties (CI-covered, stable)
  + cargo-fuzz targets (local/deep, nightly) sharing invariant checkers.

## Verdict: **NOT GREEN — 0 Critical / 2 Important**

The three invariant families (P2/P4/P5 refusal+no-substitution, P6 round-trip) are all
**TRUE of the current code** — I verified each against the source, so there is no
false-property Critical. The two Importants are pre-implementation design corrections that
change what the implementer builds: the P6 domain is mis-specified against the encoder's real
(charset-agnostic) contract, and the shared-checker-visibility default needlessly widens the
**published** crate's public API when a zero-surface alternative exists and is expressible over
already-public symbols. Both are cleanly fixable; fold and re-dispatch for round 1.

---

## Invariant-correctness verification (the adversarial core)

### P5 "every Some plate.string equals trim() of some input line" — **TRUE as stated.** ✅
Traced `run_bundle` (`bundle.rs:183-329`) end-to-end. Every `PlateEntry.string` that is `Some`
is `s.clone()` where `s` originates from a `Parsed::{Md1Single,Md1Chunk,Mk1Chunk}` variant, and
in each variant `s` is set to `s.to_string()` **after** `parse_line`'s `let s = s.trim();`
(`bundle.rs:101`). Concretely:
- unchunked md1 plate: `string: Some(s.clone())`, `s` from `Md1Single{s}` (`bundle.rs:236-244`);
- chunked md1 plate: `string: Some(s.clone())` (`bundle.rs:261-272`);
- mk1 plate: `string: Some(s.clone())` (`bundle.rs:288-299`);
- ms1 reminder plate: `string: None` (`bundle.rs:303-312`).

Critically, the reassembly calls (`md_codec::chunk::reassemble` at `:252`, `mk_codec::decode`
at `:279`) are used **only as integrity oracles** — their `Ok` value is discarded (`.map_err(…)?`),
so **no reassembled / synthesized / re-serialized string is ever emitted**. There is no chunk-header
rebuild, no policy synthesis, no case-folding of the stored value. So the "no substitution"
property holds verbatim, and the spec's precise assertion
`input.lines().map(str::trim).any(|l| l == s)` is correct **because it mirrors `run_bundle`'s own
line pipeline** (`input.lines().map(str::trim).filter(non-empty)`, `bundle.rs:184-188`). Since `s`
is a trimmed, non-empty line, `.any()` always finds it. **No Critical.** (Faithfulness caveat →
Nit N3.)

### P2 / P4 "ms always refused" — **TRUE; the strategy is right, with one construction constraint.** ✅
`classify` (`classify.rs:40-52`) trims (`s.trim()`), takes everything before the first `1`, and
`.to_ascii_lowercase()`s it before matching `"ms"`. So `"MS1…"`, `" ms1 "`, `"Ms1…"` all →
`Format::Ms`. In `convert` (`lib.rs:56-64`) the `Ms` check (`:59-61`) fires **before** `validate`
and cannot be pre-empted: for any `"ms1"+tail`, `classify` returns `Ok(Ms)` (the `1` is always at
index 2, HRP = `"ms"`), never an `Err` — so `convert` always returns `RefusedSecret`. P2 cannot
false-fail.

For P4, `run_bundle`'s pre-scan (`bundle.rs:194-198`) classifies **every** trimmed non-empty line
and returns `RefusedSecret` on the first `Ms`, before any BCH validation of any line. So an ms line
at any position → refused regardless of the other lines. P4 holds robustly. **Constraint the
strategy MUST honor (→ Nit, folded into N3-adjacent note):** the ms token must be its own
newline-delimited line and be non-whitespace so it survives `.filter(non-empty)`; an ms token
embedded mid-line with other tokens (`"md1… ms1…"`) classifies by the *first* HRP (`md`) and would
NOT trip the refusal — that is correct behavior, but the P4 generator must place ms1 on its own line
or it will (correctly) not refuse and the property (as intended) still holds only because it's
line-scoped. State it so the implementer builds the strategy line-wise.

### P6 "ndef round-trip" — **round-trip is TOTAL on the ≤249-BYTE UTF-8 domain; the spec's domain
description is wrong.** (→ Important I1) ⚠️
I traced `encode_text_tlv`→`decode_text_tlv` (`ndef.rs:60-103`) for a general `t`:
- `text_record` (`:30-43`) copies `text.as_bytes()` **verbatim** — it is **charset-agnostic**; there
  is no "printable set the encoder accepts."
- `decode_text_record` (`:80-103`) slices by explicit lengths (`type_len`, `plen`, `lang_len=0`) and
  ends with `std::str::from_utf8(text_bytes).ok()`. Since `t` is a `&str`, `text_bytes = t.as_bytes()`
  is always valid UTF-8 → `Some(t)`. I checked the empty string and the boundary explicitly.
- Encodable iff `text.len() ≤ 249` **bytes**: `text.len()=249` → NDEF message len `254` (`< 0xFF`, Ok);
  `text.len()=250` → message len `255` (`≥ 0xFF`) → `Err(NdefError::TooLong)` (`ndef.rs:48`,
  confirmed by the existing `ndef_boundary_249/250` tests at `:139-160`).

So P6 is **not vacuous and cannot false-fail** *if* the domain is pinned correctly. The spec's
"charset = the printable set the encoder accepts" is a **factually incorrect claim about the code**
and, if implemented literally, restricts the property to exactly the inputs F18 exists to *not*
restrict (control bytes, multibyte UTF-8) — gutting the insurance value. See I1 for the fix.

**Net: 0 Critical on invariant correctness.** All three families hold against the source.

---

## Findings

### Important

**I1 — P6 domain is mis-specified against the encoder's real contract (resolves open-question #3).**
Spec §C1 P6 / §Open-Q #3 say "charset = the printable set the encoder accepts" and "length ≤ 249".
The encoder (`ndef.rs:30-43`, `:60-62`) imposes **no charset** (raw `str` byte copy) and the length
bound is **byte** length, not char count. Two concrete defects if built as written: (a) a
"printable"/ASCII strategy under-tests precisely the arbitrary-byte inputs this insurance property is
for; (b) a char-count-bounded strategy over multibyte UTF-8 can exceed 249 *bytes*, so a naive
`decode(encode(t).unwrap())` would **panic on `unwrap`** (encode returns `Err(TooLong)`) — a false
failure. **Fix — pin the domain as:** arbitrary UTF-8 `String` (the whole point), keyed on **byte**
length, with a **Result-aware** assertion:
```
match encode_text_tlv(&t) {
    Ok(bytes) => assert_eq!(decode_text_tlv(&bytes).as_deref(), Some(t.as_str())),
    Err(NdefError::TooLong(_)) => assert!(t.len() >= 250),   // t.len() = bytes
}
```
To actually exercise the round-trip branch (not mostly `TooLong`), bias generation to byte-length
`0..=249` (e.g. `proptest::collection::vec(any::<char>(), …)` capped by byte budget, or
`string_regex` with a length bound) — but keep the charset UNRESTRICTED (include control chars +
multibyte). Delete the "printable set the encoder accepts" wording; state explicitly "encoder is
charset-agnostic; bound is 249 bytes; round-trip is total on that domain (verified R0)."

**I2 — shared-checker default widens a PUBLISHED crate's public API; the zero-surface alternative
is available (resolves open-question #4).** Spec §Design + §Open-Q #4 draft "expose minimal `pub`
invariant helpers." But `mnemonic-engrave` is published to crates.io (`Cargo.toml:1-11`,
v0.3.0), so new `pub fn check_*` become a **semver-committed public surface** carried forever for
test scaffolding. This is unnecessary: **every** invariant the checkers need is already expressible
over existing public API — `mnemonic_engrave::convert` + `ConvertError::RefusedSecret`;
`bundle::run_bundle` + `BundleError::RefusedSecret`; `Manifest.plates[*].string` (all `pub`,
`manifest.rs:41-57`); `ndef::{encode_text_tlv,decode_text_tlv}`; `classify::classify` +
`Format::Ms`. **Fix — use a shared file included by both consumers, adding ZERO new pub items:** put
the checkers in e.g. `crates/me-cli/tests/support/invariants.rs` and pull it into both the proptest
integration test and each fuzz target via `#[path = "…/invariants.rs"] mod invariants;` (or
`include!`). The file calls only the already-`pub` API, so it compiles identically in the in-tree
test crate and the separate fuzz crate, and the "two layers can never drift" goal is met without
touching the crate's API contract. Reverse the Q4 default to this.

### Low

**L1 — `cargo test --locked` requires a regenerated, committed `Cargo.lock` (else CI goes red).**
The `test` job runs `cargo test --locked` on stable `1.85.0` (`release.yml:40-64`,
`RUST_TOOLCHAIN=1.85.0`). Adding `proptest` as a dev-dep changes the **root** lockfile (currently
74 packages; no proptest/quickcheck/arbitrary/libfuzzer present — verified). `--locked` **forbids**
mutating the lock, so the plan MUST `cargo generate-lockfile`/build once un-locked and **commit
`Cargo.lock`**. Self-catching via the spec's own `cargo test --locked` verification step, but list
it explicitly. (Also pin a proptest version whose MSRV ≤ 1.85.0 — all 1.x qualify; trivial.)

**L2 — make the fuzz crate self-detaching AND verify the lock isn't polluted (resolves open-Q #2
safely).** The root workspace uses **explicit** `members = ["crates/me-cli"]` (not a glob,
`Cargo.toml:1-3`), so `crates/me-cli/fuzz` is not auto-included and its one-way path-dep
(fuzz → `mnemonic-engrave`) never drags libfuzzer-sys into the root resolution → stable
`cargo test --locked` is unaffected. That is the right direction. **Two requirements to bank it:**
(a) the fuzz `Cargo.toml` MUST carry its own `[workspace]` table (the `cargo fuzz init` default) so
the crate is a standalone workspace with its OWN `fuzz/Cargo.lock` — this, not the root `exclude`, is
the load-bearing detachment (root `exclude = ["crates/me-cli/fuzz"]` is belt-and-suspenders + silences
any "believes it's in a workspace" error). (b) Add an explicit verification: after adding the fuzz
crate, `git diff Cargo.lock` (root) is empty and `cargo metadata --format-version=1` / `cargo tree`
show **no** `libfuzzer-sys`. The spec's "`[workspace] members=[]`-excluded via the root `exclude`"
phrasing conflates the two mechanisms — separate them.

**L3 — P1/P3 strategies: `string_regex(".*")` generates neither newlines nor invalid bytes.** The
`regex` `.` excludes `\n` by default, so P3 ("arbitrary MULTI-line String") and any newline-bearing
P1 input won't be produced — use `(?s).*` or an explicit `vec(line_strategy, 0..N).join("\n")`.
Also "unrestricted bytes" is a misnomer: `convert`/`run_bundle` take `&str`, so the domain is
arbitrary **UTF-8** (proptest `String` strategies already guarantee this) — reword to avoid implying
raw-byte fuzzing at the proptest layer (raw bytes are the cargo-fuzz layer's job via
`String::from_utf8_lossy` on `&[u8]`).

**L4 — the fuzz crate is intentionally NOT CI-built; state the residual explicitly.** With
"proptest carries CI" (open-Q #1 default — which is CORRECT: F18 is insurance, proptest exercises
the invariants every run on stable, a nightly `cargo fuzz` job adds toolchain + flakiness for
marginal gain), the CI `test` job never runs `cargo +nightly fuzz build`, so the fuzz harness can
bit-rot undetected. This is acceptable *because the shared-checker design (I2) puts all invariant
LOGIC behind proptest (CI-covered)* — only the ~10-line `fuzz_target!` wrappers are un-CI'd. Say so,
and keep the spec's local `cargo +nightly fuzz build` pre-merge gate.

### Nit

**N1 — prove teeth for the never-panics properties too.** The spec's perturb-then-revert list covers
P2/P5/P6 but not P1/P3. A never-panics property only has teeth against an introduced panic — add a
one-shot perturbation (e.g. a temporary `panic!`/OOB index on a chosen input class) to demonstrate
proptest observes it as a failure, and record the red in the log.

**N2 — VCS hygiene for generated artifacts.** `.gitignore` the cargo-fuzz `fuzz/corpus/`,
`fuzz/artifacts/`, `fuzz/target/`; and decide `proptest-regressions/` (recommend **commit** it so a
discovered counterexample is replayed deterministically in CI).

**N3 — keep the P5 checker `.lines()`-based (faithfulness across `\r\n`).** `run_bundle` splits with
`str::lines()` (strips a trailing `\r` before `\n`). The P5 checker's
`input.lines().map(str::trim)` already mirrors this — do NOT "simplify" it to `split('\n')`, which
would diverge on CRLF inputs and could spuriously fail P5.

**N4 — treat the "82 tests" baseline as approximate.** §Ordering says "expect 82 + new prop tests";
that's a soft count — have the implementer read the actual green baseline from a clean
`cargo test --locked` run rather than asserting a literal 82.

---

## Open-question dispositions
1. **CI fuzz job → proptest carries CI (no nightly job).** ✅ Confirm the draft — correct for an
   insurance item; the stable `test` job already exercises the invariants every run.
2. **Placement/exclusion → under the crate, root-excluded.** ✅ Direction correct; bank it with L2
   (own `[workspace]` table + empty-lock verification).
3. **P6 domain.** ❌ Draft wording wrong → **I1** (arbitrary UTF-8, byte-length ≤ 249, Result-aware).
4. **Shared-checker visibility.** ❌ Draft (`pub` helpers) reversed → **I2** (shared `#[path]` file
   over already-public API, zero new pub surface).

## TDD integrity
Perturb-then-revert is genuinely provable for P2 (drop the `RefusedSecret` return → red), P5
(fabricate a plate string → red), P6 (break encode/decode symmetry → red); add N1 for P1/P3.
`ProptestConfig::with_cases(256)` keeps CI fast — random inputs almost never form BCH-valid
md1/mk1 (fast classify/validate rejection) and ms-refusal short-circuits before validation, so
per-case cost is trivial.

## Recon-citation spot check
All spec §Recon line refs verified accurate against the worktree: `convert` `lib.rs:56`, ms→
`RefusedSecret` `lib.rs:60`; `run_bundle` `bundle.rs:183`, parse_line ms→refused `bundle.rs:104`;
`PlateEntry.string` `manifest.rs:47`; trim at `bundle.rs:101`; `ndef::{encode,decode}_text_tlv`
present. No drift.

---

### Bottom line
**NOT GREEN (0C / 2I).** Fold I1 (P6 domain: arbitrary UTF-8, ≤249 bytes, Result-aware) and I2
(shared `#[path]` invariant file, no new `pub`), plus the Lows/Nits, then re-dispatch for round 1.
No Critical: the properties are true of the current code — this is a spec-precision pass, not a
design defect.
