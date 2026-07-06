# R0 architect review — SPEC_me_testing_hardening.md (round 1)

Reviewer: opus architect (R0 gate). Date: 2026-07-06.
Target: `design/SPEC_me_testing_hardening.md` (after round-0 folds).
Prior round: `agent-reports/me-testing-hardening-spec-R0-round0.md` (NOT GREEN, 0C/2I/6L;
all findings folded 2026-07-06). Standard: GREEN = 0 Critical / 0 Important.

Scope of this round: (1) verify each round-0 finding is genuinely closed by its fold and not
paraphrased away; (2) re-verify I1's arithmetic against the actual codec/NDEF code; (3) confirm
A3's canonical definition is precisely implementable and its positive test pins today's behavior;
(4) catch any drift the folds introduced; (5) fresh adversarial pass over the whole document.

## Verdict

**GREEN — 0 Critical / 0 Important / 5 Low-Nit.**

The two round-0 Importants are genuinely and correctly closed, verified against source, not
paraphrased away. All six round-0 Lows were folded and their six FOLLOWUPS entries exist and
match the SPEC's disposition claims. The remaining items are all Low/Nit: two are factual
corrections to claims the round-0 review introduced and the fold propagated verbatim (an inverted
build-tag statement in B5, and a bundle-path test-attribution slip in A3), neither of which
changes feasibility or funds-safety; three are precision tightenings. None blocks the gate. I
recommend folding N1 and N2 (they touch a load-bearing fact and test validity) opportunistically,
but per the standard 0C/0I is GREEN.

---

## Closure verification of the round-0 Importants (re-derived against source)

### I1 (B1/B2 length vectors) — CLOSED, arithmetic independently re-verified
Round-0 I1: the "249-char md1" / `{248,249}`-through-convert vectors were infeasible because a
valid md1/mk1 can never reach the 249/250 SR boundary; the fix retargeted the boundary to the
NDEF layer and capped convert-level goldens at the real codec maxima.

Re-derived against current code:
- **md1 max length.** `md-codec` regular code is BCH(93, 80, 8):
  `REGULAR_DATA_SYMBOLS_MAX = 80`, `REGULAR_CHECKSUM_SYMBOLS = 13`,
  `REGULAR_CODE_SYMBOLS_MAX = 93` (`md-codec-0.40.0/src/codex32.rs:18,25,32-33`; identical
  structure in 0.36). The string is `md` + `1` + ≤93 code symbols ⇒ **max valid md1 = 96 chars**.
  The 0.40 guard `if symbols.len() > REGULAR_CODE_SYMBOLS_MAX` (`codex32.rs:174`) is what A2
  brings in. SPEC's "a valid md1 caps at ~96 chars — codex32 93-symbol limit" (B1) is correct.
- **mk1 max length.** The in-tree `MK1_A` fixture (`bundle.rs:331`) measures **exactly 111 chars**
  and is a valid Long-code chunk (`data_part.len()=108` → `bch_code_for_length` → Long). SPEC's
  "mk1 chunk (111-char)" golden is a real, already-constructible string. Confirmed.
- **NDEF boundary.** `text_record` builds message = `[D1,01,plen,54,00] + text` = 5 + text.len();
  `tlv_wrap` rejects `if message.len() >= 0xFF` (`ndef.rs:48`). So 249-char text → message 254 →
  TLV length byte `0xFE (< 0xFF)`; 250-char text → message 255 → `NdefError::TooLong`. Both are
  properties of `encode_text_tlv`/`text_record`, which take arbitrary `&str` (cf. existing
  `rejects_oversize` on `"a".repeat(255)`), so B1's "ndef-layer boundary tests (unit, on
  `encode_text_tlv` with synthetic text)" and B2's `{1,63,64,96,111,248,249}` synthetic set are
  all `< 250` and feasible.

The fold is complete: the SPEC now explicitly splits **ndef-layer synthetic** (249/250 boundary,
`{1..249}`) from **convert-layer valid strings** (93-symbol md1, 111-char mk1), and no "249-char
md1" or convert-layer `{248,249}` over-constraint survives anywhere. Acceptance criteria are
satisfiable and non-vacuous. **Closed.**

### I2 (A3 canonical definition) — CLOSED, positive test pins real behavior
Round-0 I2: "canonical" was undefined and risked regressing a currently-valid clean md1 piped
with a trailing newline; the specified tests wouldn't catch it.

Re-derived:
- **Definition is now precise and implementable from the text.** A3 states: trim first
  (`str::trim`, as today), then *canonical* = the trimmed string has no interior whitespace and
  no `-` anywhere. Directly implementable as `let t = s.trim(); t.chars().any(|c| c ==
  '-' || c.is_whitespace())` → error. Refusal names the offending char + byte position, redacted
  on the bundle path per A1.
- **Positive trailing-newline test pins today's behavior.** Verified in `main.rs`:
  `stdin().read_to_string(&mut input)` keeps the trailing `\n`; `convert(&input)` does
  `input.trim()` (`lib.rs:57`) before classify/validate/encode, so `echo "md1…" | me` and the
  un-padded form emit byte-identical NDEF and exit 0 today. The A3(b) positive regression guard
  therefore pins existing behavior, not an invented one. **Closed.**
- **A3 is correctly md1-only.** Verified `mk-codec::string_layer::bch::decode_string`
  (`mk-codec-0.4.0/.../bch.rs:645-690`) rejects any non-alphabet char immediately
  (`ALPHABET_INV[c]==0xFF` → `InvalidChar`) — it does **not** strip `-`/whitespace the way
  md-codec's `unwrap_string` does. So mk1 has no F4 divergence and needs no canonical guard;
  scoping A3 to md1 is correct and complete.

### Round-0 Lows L1–L6 — all folded
- **L1** (F11/F13 dispositions): present in the "Finding dispositions not otherwise covered"
  section; the two FOLLOWUPS entries `me-sidecar-discovery-integrity` and
  `me-preview-png-stroke-width` exist and match. ✓
- **L2** (concrete mk-codec decision): A2 now bumps to "latest 0.4.x" (= published 0.4.1) and
  re-runs mk1 fixtures with a STOP-and-flag on any byte change. ✓
- **L3** (tmc2209 host-compile note): folded into B5, but the stated *reason* is factually
  inverted — see **N1**. Conclusion (host-compilable) is nonetheless correct.
- **L4** (alphabet union): B1(c) now "UNION across vectors … every charset symbol must appear in
  ≥1 vector." ✓
- **L5** (submodules + ME_REQUIRE_GO in CI): A5 now mandates `submodules: true` and
  `ME_REQUIRE_GO=1` in the job env. ✓ (Verified no `ME_REQUIRE_GO` reference exists in-tree yet,
  so the acceptance "delete `go` with the var set → suite fails" is non-vacuous.)
- **L6** (descope A6/A7/B4/B7): applied; Open-Q3 kept-set = A1–A5 + B1/B2/B3/B5/B6/B8; four
  FOLLOWUPS entries exist. ✓

### FOLLOWUPS / finding-map cross-check
All F1–F18 are accounted for: F1→A1/B8, F2→A5, F3→A5/B2, F4→A3, F5→A2, F6→B1/B2, F7→A4,
F8/F9→`me-preview-stale-plates-and-sidecar-output-validation`, F10→`me-output-file-permissions`,
F11→`me-sidecar-discovery-integrity`, F12→B5, F13→`me-preview-png-stroke-width`, F14→B1/B2,
F15→`me-preview-render-goldens`(B4), F16→B3, F17→B6, F18→`me-fuzz-proptest-targets`(B7). The six
new `me-*` descope entries in `FOLLOWUPS.md` "## Open" all exist and match the SPEC's disposition
claims. Refuted set (D2-1/D4-1/D6-6-imp/D6-7-mod) correctly in Non-goals.

### Spot-checks that A1/A2 acceptance is feasible and non-vacuous
- **A1** leak sites re-confirmed: `bundle.rs:54,55,60` (`Classify`, `Validate`, `Md1HeaderRead`)
  are the only `BundleError` arms interpolating `{s}`; `Mk1SingleString`/`Md1WireVersion` bind-
  and-discard, `SetIncomplete{Mk,Md}` print a hex id. The `msx1…` path → `classify`
  `UnknownHrp("msx")` → `BundleError::Classify(s,…)` → full body via `main.rs:184` today. A1's
  three-arm target is exact. Convert path already hardened (`ConvertError::Validate(e)` → `{e}`).
- **A2** is feasible: in 0.36, `wrap_payload` has no data-symbol cap, so an 81-data-symbol (⇒
  94-code-symbol) md1 can be built and is BCH-accepted by 0.36's `unwrap_string`; 0.40 rejects it
  at `codex32.rs:174` (`StringSymbolCountOutOfRange`) before BCH. So `me convert <that fixed
  string>` exits 0 today and exit-4 after the bump — fails today, non-vacuous.

---

## LOW / NIT

### N1. B5 states an inverted build-tag fact (carried from round-0 L3); conclusion still correct
**Where:** SPEC §B5 — "confirmed host-compilable: only `uart_pio.go` is tinygo-build-tagged".
**Problem:** the actual tagging in `third_party/seedhammer/driver/tmc2209/` is the opposite:
`uart.go` carries `//go:build tinygo && rp` (imports `machine`, `device/rp`, `runtime`), while
`uart_pio.go` is **UNTAGGED** and is compiled on the host. So the SPEC's stated reason ("the
tinygo-only file is excluded on host") points at the wrong file. An implementer reasoning from
this sentence could wrongly conclude host-compilability from a false premise.
**Why not Important:** I verified the conclusion holds anyway. On a host build the tmc2209 package
compiles from `gen.go` + `tmc2209.go` (stdlib-only: `encoding/binary`, `errors`, `fmt`, `io`,
`math`, `time`) + the untagged `uart_pio.go`, whose only import is `seedhammer.com/driver/pio`
and whose only pio references are `pio.StateMachineConfig` / `pio.DefaultStateMachineConfig` —
both defined in pio's untagged `config.go` (the tinygo-only `pio.go` is `//go:build tinygo &&
rp2350`, excluded on host). No untagged file references `machine`/`device/rp`/`runtime`. So the
package **does** host-compile and B5 (importing `seedhammer.com/driver/tmc2209` in
`params_test.go`, reading `tmc2209.Microsteps`) is feasible; worst case of the wrong fact is a
loud compile error the mandatory `go vet` re-verify catches, never a silent green.
**Fix:** correct the sentence — the tinygo-tagged file is `uart.go` (`//go:build tinygo && rp`);
`uart_pio.go` is untagged but host-safe (uses only the untagged pio `config.go` API). Keep the
"implementer re-verifies with `go vet`/host `go test`" note.

### N2. A3(a) claims the interior-`\n` case exercises the bundle path, but line-splitting prevents it
**Where:** SPEC §A3 acceptance (a): "`md1…x-y…`, `md1…x y…`, `md1…x\ny…` (interior) → error on
convert AND bundle paths".
**Problem:** `run_bundle` splits on `input.lines()` (`bundle.rs:178-182`) BEFORE `parse_line` ever
sees a string, so an interior `\n` is consumed as a line boundary: `md1…x\ny…` becomes two lines
(`md1…x`, `y…`) that fail for unrelated reasons (BCH-invalid truncated md1 / unclassifiable `y…`),
NOT via A3's canonical guard. So `parse_line` can never observe an interior newline, and that
sub-case does not test A3 on the bundle path (it errors either way — a green assertion that
doesn't pin the mechanism, the same test-validity class as round-0 I2). The bundle-path canonical
guard *is* genuinely exercised by the single-line `x y` and `x-y` cases.
**Funds-safety impact:** none — interior space/`-` within one line reach `parse_line` and A3
refuses them; an interior newline cannot reach a single engraved plate from bundle input by
construction. This is a test-attribution defect, not a hole.
**Fix:** scope the interior-`\n` case to the **convert** path only (where whole-stdin is one
string and A3 fires); state that the **bundle**-path canonical guard is exercised by single-line
interior space and/or `-`.

### N3. A3's "ASCII whitespace" wording is narrower than md-codec's strip predicate
**Where:** SPEC §A3 — "trimmed of leading/trailing ASCII whitespace exactly as today (`str::trim`
…)" and "no interior whitespace".
**Problem:** `str::trim` and md-codec's strip both use Unicode `char::is_whitespace()`
(`md-codec …/codex32.rs`: `if c.is_whitespace() || c == '-' { continue }`), not ASCII whitespace.
The F4 divergence set (chars md-codec strips before BCH but `me` emits verbatim) is therefore
exactly `{char::is_whitespace()} ∪ {'-'}`. If an implementer takes "ASCII whitespace" literally
and uses `is_ascii_whitespace`, an interior Unicode-whitespace char (e.g. U+00A0) would pass A3,
be stripped by md-codec, BCH-verify, and be emitted verbatim — a residual F4 case A3 is meant to
close. All *realistic* F4 vectors (newline, space, soft-wrap hyphen) are ASCII and caught either
way, so funds-impact is negligible; this is a precision fix.
**Fix:** define the interior check with the same predicate md-codec strips —
`c.is_whitespace() || c == '-'` — and drop the "ASCII" qualifier (or note non-ASCII whitespace is
otherwise caught downstream by charset validation).

### N4. B6 leaves an "(R0 decides)" that Open-Q4 already decided
**Where:** SPEC §B6 — "Consider `md-codec = "=X.Y.Z"` exact pin (R0 decides)"; Open-Q4 —
"keep caret ranges + Cargo.lock (no `=` pins); the B6 drift-guard test is the load-bearing
protection."
**Problem:** Q4 has adjudicated the question B6 defers; the dangling "(R0 decides)" is stale and
reads as an open decision. Not a substantive contradiction (Q4 is authoritative).
**Fix:** reword B6 to "(resolved at Open-Q4: keep caret + `Cargo.lock`, no `=` pin — the drift
guard is the protection)."

### N5. B1(a) "maximum-length VALID md1 (93-symbol)" may over-imply an exactly-93 requirement
**Where:** SPEC §B1(a).
**Problem:** hitting exactly 93 code symbols requires exactly 80 data symbols (400-bit payload) —
constructible but not trivially so. The test's value (byte-pin + differential decode) does not
depend on the exact symbol count; the intent is the longest practical valid md1.
**Fix (trivial):** read as "a maximum-length (up to 93-symbol / ~96-char) valid md1"; the
implementer can source the longest constructible md1 (e.g. a large multi-key descriptor or a
near-max `md_codec::chunk::split` chunk). No change to feasibility.

---

## Open questions — still soundly adjudicated
Q1 (refuse vs canonicalize → refuse; post-trim/interior-only) — sound, matches F4/Rust-primary.
Q2 (A6 moot — descoped) — consistent. Q3 (descope A6/A7/B4/B7; kept set A1–A5 + B1/B2/B3/B5/B6/B8)
— matches ordering section and FOLLOWUPS. Q4 (caret + Cargo.lock, no `=` pins) — sound; see N4 for
the stale B6 cross-reference. No new contradictions between kept-set lists, ordering, dispositions,
and FOLLOWUPS entries.

## Summary
- Both round-0 Importants (I1 length vectors, I2 canonical definition) are genuinely closed;
  arithmetic and behavior re-verified against `codex32.rs`, `bch.rs`, `ndef.rs`, `lib.rs`,
  `main.rs`. All six round-0 Lows folded; all six FOLLOWUPS entries present and matching; F1–F18
  fully mapped.
- Five Low/Nits remain, none gate-blocking: N1 (inverted build-tag fact in B5 — conclusion still
  correct, recommend fix), N2 (A3 bundle-path interior-`\n` test attribution — recommend fix),
  N3 (A3 ASCII-vs-Unicode whitespace precision), N4 (stale B6 "(R0 decides)"), N5 (B1(a) 93-symbol
  phrasing).

**VERDICT: GREEN (0C / 0I).** Lows may be folded opportunistically (N1/N2 recommended); no
re-dispatch is required by the gate.
