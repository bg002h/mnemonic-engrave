# R0 architect review — IMPLEMENTATION_PLAN_me_testing_hardening.md (round 1)

Reviewer: opus architect (R0 plan gate). Date: 2026-07-06.
Target: `design/IMPLEMENTATION_PLAN_me_testing_hardening.md` (round 1, post-fold).
Prior round: round 0 = **NOT GREEN (0C / 1I / 6L)**; all 7 findings folded 2026-07-06
(`agent-reports/me-testing-hardening-plan-R0-round0.md`).
Executes the GREEN spec `design/SPEC_me_testing_hardening.md` (spec R0 GREEN at round 1).
Standard: GREEN = 0 Critical / 0 Important. This round verifies the folds actually close
their round-0 findings (not paraphrase them away), checks for fold-induced drift, and does a
fresh adversarial pass over the whole plan. It does NOT relitigate spec-GREEN decisions.

## Verdict

**GREEN — 0 Critical / 0 Important / 3 Nit.**

The single round-0 Important (I1, Step 10 test-theater) is genuinely closed by a correct and
implementable fold: I re-traced the reachability argument against `bundle.rs:144-167` and
`md-codec` `chunk.rs`, and confirmed both halves — (a) `ChunkHeaderChunkedFlagMissing` is
unreachable/unobservable through `parse_line`, and (b) it IS constructible and observable
through a direct `ChunkHeader::read` call, using only public API. All six Lows are closed, and
I re-checked each fold's factual claim against source. Every load-bearing fact holds: the
`wrap_payload(vec![0xA5; 51], 405)` fixture recipe is arity/semantics-correct in md-codec 0.36
and reproduces the D1-2 proof byte-for-byte; md-codec 0.40 is published and its over-length
guard + `ChunkHeader::read` identity + full API/`Descriptor` compatibility check out; mk-codec
0.4.1 rejects (not strips) interior separators, so scoping A3 to md1 is correct; and there are
exactly two go-skip sites, both gated by Step 2. The 3 remaining items are cosmetic/precision
nits that do not gate implementation.

---

## Fold-closure verification (each round-0 finding re-checked against source)

### I1 (Important, Step 10) — CLOSED. The `ChunkHeaderChunkedFlagMissing` test-theater is eliminated.
Round-0 defect: the plan told the implementer to hit the `ChunkHeaderChunkedFlagMissing` arm via
a `parse_line` fixture, but that arm is dead relative to `parse_line`'s pre-check, so a literal
implementation is a vacuous test. **Fold (Step 10, lines 147-166):** split into two layers —
`parse_line` fixtures for the reachable/observable arms (Md1Single, Md1Chunk,
WireVersionMismatch→Md1WireVersion via a crafted BCH-valid fixture) plus the funds case
(SetIncompleteMd); and a **direct** `md_codec::chunk::ChunkHeader::read` drift guard for the
unreachable arm, with an explicit in-plan statement that the arm "is provably unreachable AND
unobservable through `parse_line`."

I re-verified both halves against source:
- **Unreachable via `parse_line` — CONFIRMED.** `bundle.rs:145-151` pre-checks
  `probe.read_bits(5).map(|sym| sym & 0x01 != 0)` and early-returns `Md1Single` when bit 0 is
  clear. `ChunkHeader::read` (md-codec `chunk.rs:67-74`) reads `read_bits(4)` (version) then
  `read_bits(1)` (chunked) — the chunked bit is the **same** bit 0 (LSB) of the first symbol.
  `read()` returns `ChunkHeaderChunkedFlagMissing` only when that bit is 0 (`chunk.rs:72-74`),
  but the pre-check has already returned `Md1Single` in that case, so `read()` is reached ONLY
  with bit 0 = 1, where `!chunked` is never taken. The arm is dead relative to the pre-check, and
  even if reached it returns `Ok(Md1Single)` — byte-identical to the pre-check's early return, so
  unobservable. The fold's justification is exactly right.
- **Reachable/observable via direct `ChunkHeader::read` — CONFIRMED.** To hit the arm you need
  first-symbol version nibble = `WF_REDESIGN_VERSION` (=4) AND bit 0 = 0 → e.g. first 5 bits
  `[0][1][0][0][0]` (version 0100=4, chunked=0). Fed to `ChunkHeader::read`, this passes the
  version check and returns `ChunkHeaderChunkedFlagMissing`. This mirrors md-codec's own
  `chunk.rs` unit test `chunk_header_rejects_v0x_version` (which builds `[0000][chunked=1]` and
  asserts `WireVersionMismatch{got:0}`). The direct guard is implementable with **public** API:
  `md_codec::chunk::ChunkHeader::read`, `md_codec::bitstream::{BitReader::with_bit_limit,
  BitWriter}` are all `pub` in 0.40 (verified), and `bundle.rs` already calls the first two.
- **`ChunkHeader::read` is byte-identical 0.36↔0.40 — CONFIRMED** (diffed `chunk.rs` read()
  bodies: only an added doc line + a brace shift; the four-statement logic is unchanged), so the
  "adapt the probe" contingency the plan flags is a genuine no-op, as stated.

Net: the fold turns a vacuous-test hazard into two well-founded, source-accurate tests. The
`parse_line` WireVersionMismatch fixture is also genuinely reachable (first symbol with bit 0 = 1
and version nibble ≠ 4 → pre-check proceeds, `read()` returns `WireVersionMismatch` →
`Md1WireVersion`), constructible with a short BCH-valid 0.36-`wrap_payload` scratch fixture that
survives 0.40's over-length guard. Closed. (One residual precision nit — see N1.)

### L1 (Step 5 mislocated the shared guard) — CLOSED.
Fold: Step 5 (lines 105-110) now names **`validate.rs`** — "`validate::validate` is the single
shared admission path (called from both `lib.rs` and `bundle.rs`; verified at plan-R0)" — adds a
`Format::Md`-gated interior-separator check there as a **new `ValidateError` variant** naming the
offending char + byte position, and says "Do NOT add separate guards in `convert()`/`parse_line()`."
Deliverable table row 5 updated to "validate.rs (+ tests; NOT separate guards in lib.rs/bundle.rs)."
Verified against source: `validate::validate(fmt, s)` (`validate.rs:41`) is called from exactly two
sites — `convert()` (`lib.rs:62`) and `parse_line()` (`bundle.rs:101`) — both passing the trimmed,
separators-intact `s`. A single `Format::Md`-gated check inside `validate::validate` genuinely
guards both paths and leaves `Format::Mk` untouched. Adding a variant is safe: only `validate.rs`'s
own `Display` match is exhaustive over `ValidateError`; callers use `matches!`. Closed.

### L2 (A5 trigger set + branch protection) — CLOSED.
Fold: Step 2 item 2 (lines 55-58) adds the "Trigger reconciliation (R0 L2)" note —
`release.yml` `on:` is currently `push: tags: v*` + `pull_request`, add `push: branches: [master]`;
item 3 (lines 59-63) adds the "Branch-protection note (repo setting, NOT YAML — user action)"
flagging that blocking a red PR from merging needs a required-status-check rule the implementer
cannot set, to be recorded in the PR description. Verified against `release.yml`: current `on:` is
exactly `push: tags: ['v*']` + `pull_request` (lines 8-12); `assemble.needs: [go-build, rust-build]`
(line 190) with `if: startsWith(github.ref,'refs/tags/v')` (line 191). Wiring `needs: [test, …]`
gates tag publish on the new job regardless of branch protection, as the plan states. Closed.
(Feature-branch-push-without-PR remains uncovered, but the fold explicitly documents the PR-based
flow — acceptable and consistent with SPEC A5's "every push/PR" for the master+PR model.)

### L3 (perturb-then-revert for drift guards) — CLOSED.
Fold: Constraints (lines 15-20) add the "Exception for pure drift/coverage guards (Steps 8, 9, 10)"
paragraph — fail-first is satisfied by TEMPORARILY perturbing the guarded constant/behavior (flip
`mm`, flip a discriminator bit, drop the ms1 pre-scan) then reverting, "never manufacture a fake
red" — and note that "Step 8's stderr-canary is regression insurance; the genuine fail-first for
redaction is Step 3's `msx1` test." Matches the round-0 fix verbatim in intent. Closed. (See N2 for
a fresh nit on Steps 6-7, which round 0 did not scope into this exception.)

### L4 (STOP condition framed as byte-diff only) — CLOSED.
Fold: Step 4 (lines 87-93) restates the STOP condition with "two symptoms: (a) the `md1-short.ndef`
and `bundle-md1-mk1.json` goldens are not byte-identical; (b) any previously-passing mk1/md1
admission test newly FAILS after the bump," with the correct rationale ("a codec bump cannot change
emitted bytes — `convert()` emits verbatim input — so the live regression symptom is a
previously-valid fixture newly rejected; mk-codec 0.4.0→0.4.1 is the live risk"). Closed. (Minor
cross-ref inconsistency in the Constraints summary — see N3.)

### L5 (D1-2 fixture not recorded verbatim) — CLOSED, and independently confirmed.
Fold: Step 4 (lines 80-83) now states "The 94-symbol md1 string is NOT recorded verbatim in the D1
report (it is elided there) — generate it in a SCRATCH crate ... pinning `md-codec = "=0.36.0"` via
`wrap_payload(vec![0xA5; 51], 405)`," deleting the round-0 "recorded verbatim" implication.
Verified: `funds-audit-D1-admission-round0.md:80` shows `string (97 chars, 94 symbols):
md15kj6tfd9...5zfqq6yyhmu3j8` — elided with `...`, as the fold claims. The D1 report's own recipe
(`bytes = vec![0xA5u8; (bits + 7) / 8]; wrap_payload(&bytes, bits)` with `bits = 405`) reduces to
`vec![0xA5; (405+7)/8]` = `vec![0xA5; 51]` — **byte-for-byte the plan's recipe.** Closed.

### L6 (mk1 golden wording) — CLOSED.
Fold: Step 4 phrases the golden check as the two named goldens (`md1-short.ndef`,
`bundle-md1-mk1.json`) rather than "every mk1 fixture byte-identical"; Constraints line 24-25 keep
"Goldens change ONLY by adding new vector files; `md1-short.ndef` bytes must remain byte-identical
throughout." Verified: the only committed `.ndef`/manifest goldens are
`tests/vectors/md1-short.ndef` and `tests/vectors/bundle-md1-mk1.json`; mk1 fixtures are inline
string constants. Closed.

---

## Load-bearing facts re-verified against source (no finding)

- **`wrap_payload` arity/semantics in 0.36 — CORRECT.** `codex32.rs:67` =
  `wrap_payload(payload_bytes: &[u8], bit_count: usize) -> Result<String, Error>`. `vec![0xA5; 51]`
  = 408 bits ≥ the requested 405; `bits_to_symbols` reads exactly 405 bits → 81 data symbols; +13
  checksum = 94 code symbols (the "94-symbol md1"). Deterministic. (The `vec!`-vs-`&[…]` shorthand
  in the plan is a doc convenience; the call is `wrap_payload(&bytes, 405)`.)
- **md-codec 0.40 over-length guard — CONFIRMED present and correctly targeted.**
  `REGULAR_DATA_SYMBOLS_MAX = 80` (BCH(93,80,8)); `unwrap_string` rejects `symbols.len() >
  REGULAR_CODE_SYMBOLS_MAX` (=93) with `StringSymbolCountOutOfRange` (`codex32.rs:174-179`), so the
  81-data-symbol/94-symbol fixture is refused on 0.40 and accepted on 0.36 (which lacks the guard —
  proven empirically in D1-2). `wrap_payload` also gained `PayloadTooLongForSingleString` for >80
  data symbols, so the fixture MUST be pre-captured on 0.36 — the plan's ordering is required, not
  merely prudent. Step 4's "confirm the fail-closed `StringSymbolCountOutOfRange` path" is exact.
- **Compile compatibility 0.36→0.40 — CONFIRMED.** `Descriptor` struct is field-identical
  (`n, path_decl, use_site_path, tree, tlv`); `chunk::split`, `chunk::reassemble`,
  `use_site_path::UseSitePath::standard_multipath`, `tlv::TlvSection::new_empty`,
  `tree::Body::MultiKeys`, `PathDeclPaths::Divergent` all present and `pub`, so `bundle.rs`'s
  `chunked_md1_vector()` helper (load-bearing for Step 10's SetIncompleteMd fixture) still compiles.
- **A3 md1-only scoping — CONFIRMED correct.** md-codec `unwrap_string` STRIPS interior
  `is_whitespace()`/`-` (`codex32.rs:160-162`) — the funds bug (validate strips, `convert` engraves
  verbatim). mk-codec's `decode_string` char loop REJECTS any non-bech32 char as
  `Error::InvalidChar` (`string_layer/bch.rs:680-687`) — it does NOT strip. So the plan's "mk1
  untouched (mk-codec already rejects these as InvalidChar)" is source-accurate; A3 correctly scopes
  the interior-separator refusal to `Format::Md`.
- **CI gate has no hole.** Exactly two go-skip sites exist — `cross_lang.rs:11` (early return) and
  `preview_cross_lang.rs:82` (`go_available()` guard) — both named in Step 2 item 1. Step 7 extends
  `cross_lang.rs` (already gated), introducing no new bypass. `ME_REQUIRE_GO` does not yet appear
  in-tree (only in design docs), so the implementer adds it fresh.
- **Step 3 redaction targets are exactly the leaking variants.** Of the nine `BundleError` variants,
  only `Classify(s,e)`, `Validate(s,e)`, `Md1HeaderRead(s,e)` interpolate `'{s}'` (bundle.rs:54,55,60);
  `Mk1SingleString`/`Md1WireVersion` ignore their `String`; `SetIncomplete{Mk,Md}` interpolate a
  formatted chunk_set_id, not the body. Step 3 names precisely those three variants and locates by
  name (line numbers 54-60 are close; the "locate by variant name" instruction absorbs the drift).
  The B8 marker-string unit test over all variants backstops any missed path.

---

## NIT (non-blocking; do not gate GREEN)

### N1. Step 10 direct-guard phrasing omits the version precondition for `ChunkHeaderChunkedFlagMissing`
Step 10 (lines 160-162) says pin "the `ChunkHeaderChunkedFlagMissing` return on a flag-clear
stream." Strictly, `ChunkHeader::read` returns that variant ONLY when version nibble =
`WF_REDESIGN_VERSION` (=4) **and** the chunked bit = 0; a flag-clear stream with version ≠ 4 returns
`WireVersionMismatch` instead. The phrasing could lead an implementer to craft version-0/flag-0 and
be surprised. **Self-correcting** (a wrong craft yields a RED assertion, not a false green — so this
is NOT test-theater), and the round-0 report on file already gives the precise `[WF_REDESIGN_VERSION]
[chunked=0]` recipe. **Fix (cosmetic):** add "(first-symbol version nibble = `WF_REDESIGN_VERSION`
= 4, chunked bit = 0)" so the crafted stream is unambiguous, mirroring md-codec's own `chunk.rs`
unit test.

### N2. The "failing test first at EVERY step" + the Steps 8/9/10 exception leaves Steps 6-7 in a gray zone
The Constraints' blanket fail-first rule is carved out only for Steps 8/9/10, but Step 6 (B1: the
`encode_text_tlv(250)` → `TooLong` pin and the byte-pinned `.ndef` goldens) and Step 7 (B2: Go-oracle
round-trips) are ALSO additive characterization tests over already-correct behavior that pass on
first write — the same "no natural red" situation L3 flagged for 8/9/10. A rigid implementer could
try to manufacture a red for the 250→`TooLong` pin. **Non-blocking:** these are legitimately additive
golden/characterization tests (a well-understood TDD category that captures a baseline), and the
perturb-then-revert technique obviously generalizes. **Fix (cosmetic):** extend the exception note to
say the perturb-then-revert / additive-golden allowance also covers the pure-pin assertions in
Steps 6-7 (byte-pinned goldens and the NDEF boundary), so no fake red is manufactured.

### N3. Constraints line 21 STOP-condition summary is narrower than Step 4's folded rule
Constraints (line 21-23) still summarizes A2's STOP as "any mk1/md1 fixture byte change after codec
bumps = normative drift," i.e. the pre-L4 byte-only framing, whereas Step 4 (the authoritative
statement) now has the broadened two-symptom rule (byte change OR newly-failing admission test).
Not contradictory (Step 4 is a superset the implementer follows), but the cross-ref is stale.
**Fix (cosmetic):** tweak the Constraints parenthetical to "(any mk1/md1 fixture byte change OR any
previously-passing mk1/md1 admission test newly failing — see Step 4)."

---

## Summary

- **I1 (round-0 Important): CLOSED** by a correct two-layer split — `ChunkHeaderChunkedFlagMissing`
  is drift-guarded via a direct `ChunkHeader::read` call (reachable/observable, public API) and is
  explicitly stated unreachable via `parse_line`; both halves re-verified against `bundle.rs` +
  md-codec `chunk.rs`.
- **L1–L6: all CLOSED**, each fold re-checked against source (shared path = `validate.rs`; branch
  triggers + protection note; perturb-then-revert exception; two-symptom STOP; elided-fixture →
  scratch generator with a recipe that reproduces D1-2 exactly; two-golden wording).
- **Fold drift:** none material — deliverable table matches step bodies; only N3's stale Constraints
  cross-ref.
- **Fresh pass:** N1 (version precondition phrasing), N2 (fail-first gray zone for Steps 6-7), N3
  (STOP summary) — all cosmetic, none gating.

**VERDICT: GREEN (0C / 0I / 3 Nit).** Implementation may proceed. The 3 nits are optional
one-line clarity edits the implementer may fold inline; none blocks the gate.
