# R0 Architect Review — SPEC_me_fuzz_proptest.md (Cycle C / F18) — ROUND 1

- **Artifact:** `design/SPEC_me_fuzz_proptest.md`
- **Worktree/branch:** `me-fuzz-proptest` (off mnemonic-engrave master `9fafb6b`)
- **Round:** 1 (round 0 = 0C/2I/4L/4N, all folded 2026-07-09)
- **Reviewer role:** R0 architect gate (GREEN = 0 Critical / 0 Important before implementation)
- **Scope of change:** test-only (proptest dev-dep + `tests/` property file + a shared
  invariant file) plus a workspace-detached `fuzz/` crate. No production/CLI/src behavior change.

## Verdict: **GREEN — 0 Critical / 0 Important** (1 Low, 4 Nit — all foldable, non-blocking)

Round 0's two Importants are **closed**:
- **I1 (P6 domain)** — now correctly specified as charset-agnostic, byte-length-keyed,
  Result-aware, with the exact `≤249 bytes → Ok+round-trip / ≥250 → Err(TooLong)` boundary. I
  re-derived the boundary from `ndef.rs` from scratch (not from the round-0 note) and it is exact,
  **not** off by the 5-byte TLV header.
- **I2 (no new pub API)** — the `#[path]`-included shared `invariants.rs` expresses every invariant
  over already-`pub` symbols; I checked each symbol's visibility in source. No residual `pub`
  helper. Zero new public surface on the published crate.

All three invariant families remain **TRUE of the current code** (re-traced fresh), so there is
still no false-property Critical. Remaining items are one Low (shared-file *location*) and four
Nits; none blocks the gate.

---

## I1 closure — P6 domain + exact boundary (re-derived independently)

Traced `encode_text_tlv` (`ndef.rs:60-62`) = `tlv_wrap(text_record(text)?)`:

- `text_record` (`:30-43`): `payload_len = 1 + text.len()` (BYTES). Errs `TooLong(text.len())`
  only when `payload_len > 255` → i.e. `text.len() >= 255`. Otherwise emits a message of length
  `5 + text.len()` and copies `text.as_bytes()` **verbatim** — **charset-agnostic**, no "printable
  set."
- `tlv_wrap` (`:47-57`): errs `TooLong(message.len())` when `message.len() >= 0xFF`. With
  `message.len() = 5 + text.len()`, that is `text.len() >= 250`.
- Net: `encode_text_tlv(t)` is **Ok iff `t.len() <= 249` bytes**; `t.len() >= 250` → `Err(TooLong)`
  (for `250..=254` the inner value is `TooLong(255)` from `tlv_wrap`; for `>=255` it is
  `TooLong(text.len())` from `text_record` — both are `Err(TooLong(_))`, so a wildcard on the payload
  is correct and the spec's un-pinned "Err(TooLong)" is right).

So the spec's **249/250** boundary is **exact**, NOT off by the 5-byte header (a naive reader might
fear the bound is on `text.len()+5 >= 255`; that IS the mechanism, and it resolves to
`text.len() >= 250`, which the spec states). Cross-checks: existing `ndef_boundary_249/250` tests
(`ndef.rs:139-160`) pin the same values, and each `'a'` there is 1 byte so char==byte there — the
spec correctly *generalizes* to byte length.

Round-trip is **total on the ≤249-byte domain** for arbitrary UTF-8: I walked
`decode_text_tlv`→`decode_text_record` (`:71-103`) symbolically. For any `t: &str` with
`t.len() <= 249`: `bytes[0]=0x03` ✓; `len = 5+t.len()`; `msg = bytes[2..2+len]` is exactly the
NDEF message (terminator excluded — `msg_len` counts only the message) ✓; flags `0xD1 & 0x07 = 1`
✓; `type_len=1`, `plen=1+t.len()`; `typ=[0x54]` ✓; `payload=[0x00]++text_bytes`; `status&0x80=0`,
`lang_len=0`; `text_bytes = t.as_bytes()`; `from_utf8` succeeds because the input was a `&str`
→ `Some(t)`. Verified the empty string (`t=""` → `Some("")`) and the 249-byte boundary. **P6 is
non-vacuous, cannot false-fail, and is fully charset-unrestricted** — exactly what the insurance
property needs. Spec §C1 P6 + §Open-Q #3 wording (charset-agnostic, byte-length, Result-aware,
"do NOT bound on char count") is now correct. **I1 CLOSED.**

## I2 closure — zero new pub API, mechanism workable in BOTH consumers

The spec routes every checker through the shared `#[path]`-included `invariants.rs` that calls only
already-`pub` API. I confirmed each symbol IS public in source:

- `convert` — `pub fn` (`lib.rs:56`); `ConvertError` `pub enum`, `RefusedSecret` variant public
  (`lib.rs:15-19`).
- `bundle::run_bundle` — `pub mod bundle` (`lib.rs:4`) + `pub fn` (`bundle.rs:183`);
  `BundleError::RefusedSecret` public (`bundle.rs:13`).
- `Manifest` `pub struct`, **`plates: Vec<PlateEntry>` is a `pub` field** (`manifest.rs:60,66`);
  `PlateEntry` `pub struct`, **`string: Option<String>` is a `pub` field** (`manifest.rs:42,47`).
  (This was the explicit round-1 ask — both fields are public, so P5 is expressible with no getter.)
- `ndef::{encode_text_tlv,decode_text_tlv}` — `pub mod ndef` (`lib.rs:7`) + `pub fn`
  (`ndef.rs:60,71`); `NdefError::TooLong` public (`ndef.rs:13-16`).
- `classify::classify` + `Format::Ms` — `pub mod classify` (`lib.rs:5`) + `pub fn`
  (`classify.rs:40`); `Format::Ms` public (`classify.rs:6-13`).

Every invariant (P1..P6) is expressible over these — no invariant needs a non-`pub` item, so there
is **no residual `pub` helper** and no widening of the published crate's surface. `proptest` is a
**dev-dependency**, which is not part of the published dependency closure, so downstream consumers
are unaffected.

Cross-context compilability of a single `#[path]`-shared file: it references the crate by its
external name `mnemonic_engrave::…`, which resolves identically (a) in the in-tree integration-test
crate (which links the lib under test as `mnemonic_engrave`) and (b) in the separate fuzz crate
(whose `mnemonic-engrave = { path = … }` dep imports as `mnemonic_engrave`). So the same file
compiles in both. **I2 CLOSED** (with a location caveat → L1-new, and a path-spelling Nit → N-d).

## Invariant families still TRUE (fresh re-trace, not trusting round 0)

- **P2/P4 (ms refused):** `classify` trims, takes the HRP before the first `1`, lowercases, matches
  `"ms"` (`classify.rs:40-52`). `convert` returns `RefusedSecret` on `Format::Ms` **before**
  `validate` (`lib.rs:59-61`); `run_bundle`'s pre-scan refuses on the first `Ms` line **before** any
  BCH validation (`bundle.rs:194-198`). Any `"ms1"+tail` (any case, whitespace-padded) → `Ok(Ms)`
  from classify (never `Err`), so `convert` → `RefusedSecret` unconditionally. TRUE.
- **P5 (no substitution):** every `Some(string)` in `run_bundle` is `s.clone()` of a
  `parse_line`-trimmed line (`bundle.rs:236-244, 261-272, 288-299`); reassembly outputs
  (`chunk::reassemble :252`, `mk_codec::decode :279`) are integrity oracles whose Ok value is
  discarded; the ms1 reminder is `string: None` (`:303-312`). The checker
  `input.lines().map(str::trim).any(|l| l == s)` mirrors `run_bundle`'s own line pipeline
  (`:184-188`), so it always finds `s`. TRUE.
- **P6 (round-trip):** total on ≤249 bytes, `Err(TooLong)` on ≥250 (above). TRUE.

No false property → **0 Critical.**

---

## Findings

### Low

**L1-new — do NOT put the shared file directly in `tests/` (spec's first example creates a spurious
test target).** §Design gives two example locations: `crates/me-cli/tests/invariants.rs` **or**
`crates/me-cli/fuzz/invariants.rs`. The **first is warty**: `crates/me-cli/tests/` already holds
`cli.rs`, `cross_lang.rs`, `golden.rs`, `preview_cross_lang.rs` (each auto-compiled by cargo as its
OWN integration-test binary; the `vectors/` *subdirectory* is not). A file at
`tests/invariants.rs` would therefore be compiled as a standalone `invariants` test target
containing **0 `#[test]`s** and would emit `dead_code` warnings for every (unused, in that context)
checker `fn`. This does **not** red the CI `test` job (`cargo test --locked` does not deny
warnings), so it is not Important — but it would fail a `clippy --all-targets -- -D warnings`
invocation, which the spec's own §Ordering "clippy clean on the non-fuzz crate" step may use, and it
contradicts round 0's I2 resolution (which wrote `tests/support/invariants.rs`).
**Fix:** place the shared file in a `tests/` **subdirectory** (`tests/support/invariants.rs`, per
round 0) or under `fuzz/invariants.rs`, and `#[path]`-include it from both consumers (e.g. the
proptest file uses `#[path = "support/invariants.rs"] mod invariants;` or
`#[path = "../fuzz/invariants.rs"] mod invariants;`). Never a bare `tests/*.rs`. Drop the
`tests/invariants.rs` example from §Design so an implementer can't pick the warty one.

### Nit

**N-a (round-0 N1 residual) — P3 never-panics teeth not stated.** §C1 gives P1 an explicit
perturb-to-panic demo ("temporarily `unwrap()` an internal Result → P1 red"), but P3
(`run_bundle`-never-panics) has no teeth note, and §Ordering's perturb list names only P2/P5/P6.
Add a symmetric one-shot panic injection into a `run_bundle` path to prove P3 has teeth, and record
the red. (Same mechanism as P1; one line.)

**N-b (round-0 N4 residual) — "expect 82" still asserted.** §Ordering line 108 still says "expect
82 + new prop tests." N4 asked to treat this as approximate. The current tree has 54 `#[test]` in
`src/` plus the `tests/*.rs` integration targets, so the exact green count is environment/version
dependent. Soften to "read the actual green baseline from a clean `cargo test --locked` run" rather
than pinning a literal 82.

**N-c (round-0 P2/P4 construction constraint) — make the P4 ms line "leading-token" explicit.** P2
correctly says "input whose **first token** has HRP `ms`." P4 says "ms-HRP **line**." Because
`classify` keys on the HRP before the *first* `1` on the (trimmed) line, an ms1 token that is NOT
the leading token of its line (e.g. `"md1… ms1…"` on one line) classifies as `md` and is (correctly)
NOT refused. State that the P4 generator must emit the ms1 as its own newline-delimited,
non-whitespace line whose HRP is `ms` (mirror P2's "first token" precision), so the property doesn't
false-fail on a co-located token.

**N-d — spell the shared-file crate references as `mnemonic_engrave::…`, not `crate::`.** For the
one `#[path]`-shared file to compile identically in the test crate and the fuzz crate, it must
reference the library by its external name (`use mnemonic_engrave::{convert, run_bundle, …};` or
fully-qualified paths), never `crate::` (which would resolve to the *including* crate in each
context). Implied by "over the pub API," but worth one explicit sentence to prevent a `crate::`
mistake (would be a compile error, so TDD catches it, hence only a Nit).

---

## Round-0 findings closure table

| ID | Round-0 ask | Status |
|----|-------------|--------|
| I1 | P6 = byte-length + Result-aware + charset-agnostic; exact 249/250 | **CLOSED** (boundary re-derived exact) |
| I2 | shared `#[path]` file over pub API, no new pub | **CLOSED** (all symbols verified pub; residual → L1-new location) |
| L1 | commit updated root `Cargo.lock` | **CLOSED** (§C1; lock currently free of proptest — verified) |
| L2 | fuzz own `[workspace]` + verify no libfuzzer-sys in root lock/tree | **CLOSED** (§C2; root uses explicit `members`, no `exclude` yet — both mechanisms specified + verification step) |
| L3 | `string_regex(".*")` emits no newlines | **CLOSED** (P1 `(?s).*`/byte strategy; P3 injects `\n`) |
| L4 | fuzz not CI-built residual | **CLOSED** (stated + justified by proptest CI coverage) |
| N1 | teeth for never-panics (P1 AND P3) | **PARTIAL** — P1 done, P3 residual → N-a |
| N2 | gitignore fuzz artifacts / commit proptest-regressions | **CLOSED** (§C1) |
| N3 | `.lines()` not `split('\n')` | **CLOSED** (P5) |
| N4 | 82 approximate | **OPEN (Nit)** → N-b |

## Executability / TDD / scope
- **Executability:** root `Cargo.lock` present and free of proptest/libfuzzer-sys/arbitrary
  (verified) → L1/L2 are meaningful and correctly specified. Root workspace uses explicit
  `members = ["crates/me-cli"]` (no glob), so `crates/me-cli/fuzz` is not auto-included; the fuzz
  crate's own `[workspace]` table is the load-bearing detach and root `exclude` is
  belt-and-suspenders. The §C2 `cargo tree` / clean-root-lock verification is the right gate for the
  highest-risk failure (nightly fuzz deps leaking to stable CI).
- **TDD integrity:** properties guard already-correct behavior (pass first), teeth proven by
  perturb-then-revert — genuinely red-able for P2 (drop `RefusedSecret`), P5 (fabricate a plate
  string), P6 (break encode/decode symmetry), P1 (inject panic); add P3 (N-a).
- **Scope:** dev-dep + `tests/` files + an excluded `fuzz/` crate only. No `src/` production or CLI
  change; no bleed into A/B/D. Confirmed clean.

---

### Bottom line
**GREEN (0C / 0I).** Both round-0 Importants are closed: the P6 boundary is re-derived exact
(≤249 bytes Ok / ≥250 `Err(TooLong)`, charset-agnostic) and every checker rides already-`pub` API
via the `#[path]`-shared file (no new public surface; both `Manifest.plates` and `PlateEntry.string`
confirmed `pub`). Ship to implementation. Fold the single Low (shared file goes in a `tests/`
subdirectory or `fuzz/`, never a bare `tests/*.rs`) and the four Nits inline — none blocks the gate.
