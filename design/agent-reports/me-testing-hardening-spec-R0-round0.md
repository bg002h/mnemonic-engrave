# R0 architect review — SPEC_me_testing_hardening.md (round 0)

Reviewer: opus architect (R0 gate). Date: 2026-07-06.
Target: `design/SPEC_me_testing_hardening.md` (round 0, pre-R0 draft).
Standard: GREEN = 0 Critical / 0 Important. Adversarial pass over correctness,
completeness, open-question soundness, scope, and SPEC-vs-code/audit contradictions.

## Verdict

**NOT GREEN — 0 Critical / 2 Important / 6 Low-Nit.**

The SPEC is largely well-grounded: I independently re-verified every load-bearing
code claim it cites and they hold (details under "Claims verified" below). The two
Important findings are (1) a factual impossibility in the B1/B2 golden-length vectors
that makes their acceptance criteria unsatisfiable as written and contradicts the
codec length caps in-tree, and (2) an under-specified "canonical" definition in A3 that
risks a regression for currently-valid clean inputs and whose specified tests would not
necessarily catch it. Both fold quickly.

---

## Claims verified against current code (no finding — recorded for the gate)

- **F1 / A1 leak site.** `bundle.rs:54,55,60` — `Classify(s,e)`, `Validate(s,e)`,
  `Md1HeaderRead(s,e)` are the ONLY `BundleError` Display arms that interpolate the
  input string `{s}`. `Mk1SingleString(_)` and `Md1WireVersion(_)` bind-and-discard;
  `SetIncomplete{Mk,Md}(id,e)` interpolate a hex `chunk_set_id`, not the body. A1's
  redaction target list is therefore exactly correct and sufficient. Confirmed the
  mangled-HRP path: `classify("msx1…")` → `UnknownHrp("msx")` (dodges the `run_bundle`
  ms1 pre-scan at `bundle.rs:188-191`, which only matches `Ok(Format::Ms)`) →
  `BundleError::Classify(s,…)` → full body printed via `main.rs:184`. The convert path
  is hardened: `ConvertError::Classify(e)` prints only `{e}` (HRP only). SPEC framing
  correct.
- **F6 / ndef guard.** `ndef.rs:48` is `if message.len() >= 0xFF`. Boundary arithmetic:
  `message.len() = 5 + text.len()`; `>= 255` ⇒ `text.len() >= 250`. So 249-char text →
  `message.len()=254` (TLV len byte `0xFE < 0xFF`, OK); 250-char text → `255` →
  `TooLong`. The `>=`→`>` mutation passes 255 and pushes a `0xFF` length byte, which the
  SeedHammer reader treats as the 3-byte-length escape → misparse. SPEC's boundary
  numbers are internally consistent. (But see Important #1 on how these are exercised.)
- **F5 / A2 codec bump.** `Cargo.lock` pins `md-codec 0.36.0`; primary Rust
  `descriptor-mnemonic/crates/md-codec` is `0.40.0` and is published on crates.io
  (`cargo search` → `md-codec = "0.40.0"`). The fail-closed guard
  `StringSymbolCountOutOfRange` exists at `codex32.rs:174-179`, on the `unwrap_string`
  path that `me`'s `validate.rs:43` and `bundle.rs:132` call. A2 conforms to the
  Rust-primary rule (guard already landed in the primary crate first). Confirmed the bump
  cannot rebaseline existing goldens: `convert()` emits `encode_text_tlv(s)` on the
  verbatim input, independent of codec version — the bump only changes accept/reject.
- **F2 / A5 CI gap.** `.github/workflows/release.yml` is the only workflow; jobs run
  `cargo build` / `go build` only (+ sign/assemble on tag). No `cargo test` / `go test`
  anywhere; `assemble.needs = [go-build, rust-build]`. Confirmed.
- **F3 / A5 vacuous skip.** `cross_lang.rs:11-14` and `preview_cross_lang.rs:82-85` both
  `eprintln + return` (⇒ PASS) when `go` is absent. Confirmed.
- **F4 / A3.** `md-codec 0.40` `unwrap_string` still strips whitespace and `-` at
  `codex32.rs:159-167` ("tolerate visual separators per D11"), so F4 persists after the
  A2 bump — A3 is genuinely orthogonal to A2. Confirmed.
- **F7 / A4.** `firmware/ndef-roundtrip/go.mod:7` replace = `../../../seedhammer-ref-v1.4.2`
  (outside repo); `preview/go.mod:12` = `../third_party/seedhammer`. The A4 target
  `../../third_party/seedhammer` is the correct relative path from
  `firmware/ndef-roundtrip/` (two levels up to repo root). Confirmed.
- **F12 / B5.** `preview/params.go`: `mm = 6400`, `strokeWidth = 1920`.
  `tmc2209.Microsteps = 1 << 8 = 256` (`third_party/seedhammer/driver/tmc2209/tmc2209.go:23-25`,
  no build tag). So `200/8 * Microsteps = 25*256 = 6400 == mm` ✓ and `mm*3/10 = 1920 ==
  strokeWidth` ✓ — B5's formulas are arithmetically correct and importable (only
  `uart_pio.go` in that package is `//go:build tinygo && rp`).
- **B3 feasibility.** ms1 at any line position → `run_bundle` pre-scan → `RefusedSecret`
  (exit 3); classify lowercases the HRP and is checksum-agnostic, so upper/mixed/padded/
  bad-checksum ms1 all still classify as `Ms` and are refused. Confirmed feasible.

Rust-primary check (task item 3): **no violation.** A2 consumes an already-published
primary guard (conforms). A3-refuse is a `me`-layer admission guard with no Go/codec
counterpart to lead (the fork firmware is out of scope per Non-goals); refusal does not
define new codec canonical-form semantics, so it avoids the md-codec-first dependency the
SPEC itself flags for the canonicalize alternative. Both defensible.

---

## IMPORTANT

### I1. B1(a) and B2 length vectors are infeasible as valid md1/mk1 — acceptance unsatisfiable, contradicts in-tree codec caps
**Where:** SPEC §B1(a) ("249-char **max-length md1**", "250-char input → `NdefError::TooLong`"),
§B2 (round-trip goldens "parameterized over text lengths {0-ish min valid, 63, 64, 111,
248, 249}").
**Problem (contradicted by code):** every md1 string is a codex32 *regular* codeword
capped at `REGULAR_CODE_SYMBOLS_MAX = 80 + 13 = 93` symbols
(`descriptor-mnemonic/crates/md-codec/src/codex32.rs:25-33,174`), i.e. a valid md1 is at
most `3` (HRP `md1`) `+ 93 ≈ 96` chars. mk1's longest in-tree fixture is 111 chars. So a
**valid md1 can never be 249 (or 248) chars**, and `convert()` — which BCH-validates —
will reject any synthetic 248/249/250-char "md1". The SR-boundary (249/250) and the
`TooLong` case are properties of the **NDEF layer** (`text_record`/`tlv_wrap`/
`encode_text_tlv`, which accept arbitrary text — cf. the existing
`ndef.rs:130-133 rejects_oversize` test on `"a".repeat(255)`), not of `convert()`. As
written, an implementer told to "add a byte-pinned 249-char **md1** golden" and to
round-trip goldens at `{248, 249}` through the convert path literally cannot construct the
vectors; the acceptance criteria can never go green. Note the SYNTHESIS finding F6 said
"249-char **golden**" (no "md1") — the SPEC introduced the "md1" over-constraint.
**Fix (concrete):** split the two surfaces explicitly. (a) SR-boundary/`TooLong`: test the
NDEF layer directly — `encode_text_tlv` (or `tlv_wrap(text_record(..))`) on synthetic
249-/250-char strings; pin the 249 golden bytes (TLV len `0xFE`) and assert
`encode_text_tlv(<250 chars>) == Err(TooLong(_))`; round-trip the 249 golden through the
Go oracle (the reader parses NDEF bytes regardless of md1-validity). (b) Real-string
goldens through `convert()`: cap at the longest *constructible* strings — a max-length
md1 (~93 symbols) and the 111-char mk1 chunk — and state that these, not 248/249, are the
"long" convert-level vectors. Reword B2's length list to separate NDEF-layer synthetic
lengths (63/64/248/249) from convert-layer valid-string lengths (min/63-ish/111).

### I2. A3 does not define "canonical"; risks regressing currently-valid clean inputs, and the specified tests would not catch it
**Where:** SPEC §A3 ("refuse non-canonical input by default … naming the offending
char/position"; acceptance: interior `x-y`, `x y`, `x\ny` → error; "clean strings
byte-identical to today").
**Problem:** "non-canonical" is undefined. Every `me` entry path trims first
(`lib.rs:57`, `bundle.rs:95`, `main.rs:110`, `classify.rs:41`), so a clean md1 piped with
a trailing newline (`echo "md1…" | me`) or surrounding spaces is currently accepted and
emitted byte-identically. If A3 is implemented as "reject any string containing whitespace
or `-`" over the **raw** input, it refuses that trailing newline → a regression for the
primary stdin UX and a behavior change for a currently-valid input — exactly the class R0
must guard. The specified acceptance tests only cover **interior** separators plus a vague
"clean strings byte-identical," so a raw-scan regression could ship green.
**Fix (concrete):** state that the canonical check runs **after `trim()`** and rejects only
**interior** `-`/ASCII-whitespace in the trimmed string (surrounding whitespace remains
trimmed, not an error). Add a positive acceptance test: a clean md1 (and mk1) with a
trailing `\n` and with surrounding spaces still exits 0 and produces byte-identical NDEF to
the un-padded form — so the non-regression is pinned, not merely asserted in prose. Keep
the refuse-by-default decision (sound — see Open Questions).

---

## LOW / NIT

### L1. Confirmed-LOW findings F11 and F13 are silently dropped
Neither the SPEC's items nor its Non-goals account for **F11** (PATH-based sidecar
discovery + string-match version gate, no integrity) or **F13** (PNG hairline vs SVG
0.3 mm strokes, legibility-only). For a complete, auditable audit→SPEC mapping, add a
one-line disposition for each: F13 → explicit WONTFIX (centerlines identical, judgment-only,
per SYNTHESIS), F11 → defer to `FOLLOWUPS.md` with rationale. Not blocking.

### L2. A2's mk-codec handling is a non-testable "review"
"Review mk-codec for an equivalent current pin" has no acceptance criterion. Primary
`mk-codec` is `0.4.1` (published); `me` pins `0.4.0`, and the caret `"0.4"` in
`Cargo.toml` means a `cargo update` during the bump can silently pull `0.4.1` and change
mk1 **admission**. Make A2 a concrete decision: either bump to `0.4.1` and re-run the mk1
fixtures (assert no currently-valid mk1 fixture newly rejects), or pin `0.4.0` in
`Cargo.toml` and document why. State the expected outcome (no golden/fixture change,
since `me` emits verbatim input).

### L3. B5 must confirm the tmc2209 package host-compiles
`tmc2209.Microsteps` lives in a build-tag-free file and the SH2 host tests reference the
package, so B5 is feasible — but the package also contains `uart.go`/`gen.go` (no tag)
alongside the `//go:build tinygo && rp` `uart_pio.go`. Add a one-line note that B5's
implementer verifies `go test ./preview/...` (host) still compiles with the
`seedhammer.com/driver/tmc2209` import (i.e. `uart.go` pulls no tinygo-only import).

### L4. B1(c) "full-alphabet coverage string" may be unconstructible as one valid word
A single valid md1/mk1 codeword is not guaranteed to contain all 32 bech32 symbols.
Reword to allow the union across several vectors, or exercise the charset mapping at the
`ndef`/decoder layer (arbitrary text), so the criterion is satisfiable.

### L5. A5 should state the CI test job needs `submodules: true`
The cross-lang oracle builds against `third_party/seedhammer` (via `preview/` and
`firmware/ndef-roundtrip/`), so the new test job — like `go-build`/`assemble` — must check
out submodules, and CI must set `ME_REQUIRE_GO=1` so the F3 skip does not silently no-op.
Self-correcting (red CI if missed) but worth pinning in the item to avoid a churn cycle.

### L6. Scope: descope A6, A7, B7 (and consider B4) to FOLLOWUPS to honor "one tight implementer"
Per CLAUDE.md ("keep implementation TIGHT, favor ONE implementer"), recommend the cycle be
the confirmed I/M fixes **A1–A5** + directly-supporting tests **B1, B2, B3, B8** + the
cheap drift guards **B5, B6**. Defer **A6** (stale-plate/sidecar-output validation),
**A7** (0o600 perms), and **B7** (fuzz/proptest) to `FOLLOWUPS.md`; **B4** (SVG/PNG render
goldens with a `-update` flag) is the heaviest Go item and a reasonable additional
descope. The SPEC already lists A6/A7/B7 as descope candidates — make the cut explicit so
the implementer plan is unambiguous.

---

## Open questions — adjudication

1. **A3 refuse vs canonicalize → REFUSE (endorsed).** Fail-closed, never emits bytes the
   user did not type, and stays within the Rust-primary rule (canonicalization would
   define new codex32 canonical-form semantics that must land in md-codec first;
   md-codec deliberately *tolerates* separators, so `me`-side stripping-and-emitting would
   diverge from the codec's own model). Adopt refuse; must also resolve I2.
2. **A6a refuse-nonempty vs clean-namespace →** if A6 is kept (see L6, recommend defer),
   prefer cleaning only the `plate-N.{svg,png}` namespace `me` itself owns (never arbitrary
   files) before writing; refusal-on-nonempty is the safer fallback. Either is acceptable;
   defer with the decision recorded.
3. **Descope A6/A7/B7 → yes (see L6).** Also consider B4.
4. **Exact `=` pin (B6) vs semver-float + drift guard →** keep caret in `Cargo.toml` with
   `Cargo.lock` as the reproducibility pin (already the case) plus the B6 drift-guard test;
   an exact `=` pin fights `cargo update` and duplicates what the lockfile already gives.
   The drift *guard* is the load-bearing part, not the `=`.

---

## Summary of required changes to reach GREEN
- I1: re-target B1(a)/B2 SR-boundary + `TooLong` vectors to the NDEF layer (synthetic
  text); cap convert-level "long" goldens at the real md1 (~96 char) / mk1 (111 char)
  maxima; fix the B2 length list.
- I2: define "canonical" as post-trim, interior-only; add a positive trailing-newline /
  surrounding-space byte-identity test.
- Fold L1–L6 (cheap: dispositions for F11/F13, concrete mk-codec decision, tmc2209
  host-compile note, B1(c) wording, A5 submodules/ME_REQUIRE_GO note, explicit descope cut).

Re-dispatch after folding (folds can drift).

**VERDICT: NOT GREEN (0C / 2I).**
