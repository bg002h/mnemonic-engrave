# SPEC — SeedHammer m*1 BCH error-correction (suggest → confirm)

**Status:** **GREEN (R1, 0C/0I)** — cleared the opus-architect gate after folding R0 (I-1/I-2/I-3 +
M-1/M-3). Reviews verbatim: `design/agent-reports/seedhammer-mstar-correction-spec-review-{R0,R1}.md`.
Next: Phase A plan → its own R0 gate.
**Base:** fork `main` `04a1e95`. Fork-side only (no upstream PR).
**Recon:** `design/cycle-prep-recon-mstar-correction.md` (`47c9ad7`; recon-time source SHA `04a1e95`).
**Architect panel (decisions locked here):** `design/agent-reports/seedhammer-mstar-correction-panel-{crypto-security,firmware-resource,design-decomposition}.md`.
**Scope (user decision 2026-06-18):** correct **all three m\*1** (`ms1`/`md1`/`mk1`) on the
hand-typed path — incl. a new typed md/mk entry — not ms1-only.
**Port source (oracle):** the constellation BCH decoders (`mk-codec/src/string_layer/bch_decode.rs`
+ `bch.rs` `bch_correct_*` re-verify; toolkit `repair.rs` suggest-and-re-verify model).

---

## 0. Decomposition — two gated phases (panel-unanimous), strict A → B

- **Phase A — the BCH decoder (pure `codex32` package; no GUI).** Parameterized over all three
  codes; **merged DORMANT** (no caller) once GREEN — the SLIP-39-D1 precedent. The high-risk,
  authoritative-bound piece; own R0 → review → merge.
- **Phase B — the GUI: HRP-dispatched typed entry + suggest→confirm correction for all m\*1.**
  Adds the typed md/mk entry and wires the decoder into a "Fix?" → confirm-the-edit gate. Built
  against Phase A's frozen API; own R0 → review → merge. (The plan may sub-split B into B1
  =HRP-dispatched entry+validate+engrave and B2 =the correction gate.)

Each phase: full gated pipeline (spec→R0→plan→R0→single-implementer TDD→whole-diff execution
review→merge), reviews persisted verbatim, signed+DCO.

---

## 1. Goal & scope

Teach the device to **suggest a correction** for a mistyped codex32-family string (`ms1`/`md1`/
`mk1`) and engrave it only after the user **confirms the edit** — porting the constellation's BCH
decoder. Today the device only **detects** (verify checksum). Correction is **substitutions-only**
(t=4), and benefits the **hand-typed** path; Phase B adds the typed md/mk entry so all three m\*1
are typeable + correctable on-device.

### In scope
- Phase A: `codex32` decoder — GF(1024)=GF(32²) built on the fork's GF(32); syndromes →
  Berlekamp-Massey → Chien → Forney → apply → **mandatory re-verify**; parameterized per code
  (ms1/md/mk, regular + mk-long); subs-only; **unique-within-radius or return nothing**.
- Phase B: an **HRP-dispatched typed entry** (one keypad, validate per parsed HRP) for all three
  m\*1; the **suggest→confirm "Fix?"** UX (diff display); engrave dispatch by type.

### Out of scope (explicit, deferred to FOLLOWUPS)
- **Erasures** (`?`/wrong-case → erasure): needs marking-UX + a damaged-plate re-read path that
  doesn't exist; the forced-uppercase, b/i/o-dimmed keypad makes substitutions the only realistic
  error. Subs-only v1; keep the BM core erasure-amenable.
- **Auto-apply / list-decoding / multi-candidate** suggestions — never.
- Correcting md/mk arriving over **NFC** — NFC is error-free; no decode on the scan path.

---

## 2. Security invariants (the R0 gate must verify each — Critical if violated)

1. **No auto-apply.** A corrected string MUST NOT reach the engrave path without explicit user
   confirmation of the *resulting* string (BIP-93: *"SHOULD NOT automatically proceed … without
   user confirmation"*; *"a string without a valid checksum MUST NOT be used"*).
2. **Mandatory re-verify after apply.** The decoder re-runs the verifier on the corrected word and
   rejects any non-codeword. (Necessary but NOT sufficient — see #3.)
3. **The residual "wrong-but-valid" hazard — the human diff-gate is the ONLY thing that closes
   it.** A >t-error string can decode to a *different valid* codeword that re-checksums clean
   (re-verify passes it). For seed/key material that = silently engraving the wrong secret. So the
   confirm gate MUST show the **per-position diff** (`pos N: 'x' → 'y'`, ≤4 lines) as the
   **universal human-checkable anchor for all three m\*1** — "confirm the EDIT against your card",
   NOT a blob "looks right?". For **codex32/ms ONLY**, ALSO show the decoded header fields
   (`id · thr · share` via `codex32FieldLine`/`ParsePrefix`) as a secondary anchor; that layout is
   the codex32-**share** schema and does NOT exist for md/mk (the fork has no md/mk header parser
   — `ParsePrefix` returns `errInvalidThreshold` on md/mk data — and adding one is out of scope, so
   for md/mk the per-position diff IS the anchor; see the I-2 fold in §4.2). This is the
   Seed-XOR-fingerprint discipline (verify a small, human-comparable derived artifact). The gate is
   a **new** screen, NOT `confirmCodex32Flow` (whose Button3 engraves).
4. **Suppress unless unique-within-radius.** Offer a suggestion ONLY when the decoder returns a
   correction AND the re-verify passes (⇒ the unique within-radius candidate). If the decoder
   returns nothing (`deg(Λ)=0`/`>4`, Chien root-count mismatch, bad/zero Forney magnitude), or
   re-verify fails → **no suggestion**; show the existing "bad checksum", let the user re-type.
   No "did you mean A or B?".
5. **Per-code-constant integrity (the documented landmine).** ONE source of truth: the decode path
   consumes the verifier's existing parity-tested `residue ⊕ target` — **no second copy** of
   `POLYMOD_INIT` (ms1=`1`; md/mk=`0x23181b3`), the NUMS targets, the mk1 65/75-bit hi/lo splits,
   generators, or length brackets. Pin with **Rust-generated parity vectors ONLY** (never
   Go-self-generated — false-consensus class) + a **negative cross-constant test** (an `ms1`
   string must NOT correct-and-verify under the `md` constants).
6. **Symbol orientation (the firmware-lens #1 landmine).** The Rust unpacks residue coefficients
   **LSB-first** (`coeffs[0]=x⁰`); the fork's `unpackSyms`/`engine.residue` are **MSB-first**. The
   spec mandates ONE canonical orientation + the explicit boundary conversion, pinned by a Rust
   parity vector. A silent mismatch passes symmetric tests but mis-locates real errors.
7. **TinyGo:** decoder internals `uint8`/`uint16` (the residue/target container is `uint64` hi/lo
   at the boundary, per `mdmk.go`); no `math/big`, no 128-bit; fixed-size stack arrays in
   Berlekamp-Massey (no dynamic slices in the hot path); `deg(Λ)>4 ⇒ reject` is both the
   correctness gate and a loop bound. (Plan-sizing note, M-3: `Λ` has degree ≤4 = 5 coeffs, but
   BM's working `prev`/`Λ′` and the `Ω` buffer are length-**8** per the Rust `omega=vec![ZERO;8]`,
   bch_decode.rs:446 — the plan sizes `[8]`-wide arrays, not `[5]`.) **Add `tinygo build
   -target=pico-plus2` of `codex32` to CI** (the Slice-1 lesson — host `go test` never compiles the
   device build).

---

## 3. Phase A — the BCH decoder (`codex32` package)

### 3.1 GF(1024) field — built on the fork's GF(32)
A `gf1024` value = `{lo, hi fe}` (or packed `uint16`), `ζ²=ζ+1`. `add`=componentwise XOR; `mul` =
the 4-subfield-product identity (ll/lh/hl/hh) using the fork's existing **log-table `fe.Mul`**
(`gf32.go` — proven identical to the Rust carryless field via the
`gf32_alpha_powers_match_bech32_log_inv_table` cross-check, which the port replicates as a Go
conformance test); `pow`/`inv` (`inv = a^1022`). ~0 new tables. Constants:
`β = {lo:0,hi:8}` (order 93, regular), `γ = {lo:25,hi:6}` (order 1023, long),
`REGULAR_J_START=77`, `LONG_J_START=1019` — port verbatim, regression-pinned by `β`-order-93,
`γ`-order-1023, ζ³=1, and generator-root self-tests.

### 3.2 The decoder
```go
// CorrectionResult holds a unique within-radius correction (subs-only):
// the corrected string + the per-position diff for the confirm gate.
type CorrectionResult struct {
	Corrected string         // the re-verified valid codeword
	Edits     []Edit         // {Pos int; Was, Now byte} — for the diff display
}
// Correct attempts to error-correct an invalid codex32-family string of the
// given code. Returns (result, true) ONLY for a unique correction within the
// guaranteed radius (≤4 substitutions) that RE-VERIFIES as a valid codeword;
// (_, false) otherwise (uncorrectable / >radius / re-verify fail). NEVER guesses.
func Correct(frag string) (CorrectionResult, bool)
```
`Correct` is parameterized internally by the parsed HRP → the per-code (init residue, target,
generator window, j_start, regular-vs-long) the **verifier already uses** (reuse `checksum.go`/
`mdmk.go`; no second copy). Pipeline (port of `decode_errors`): compute `residue ⊕ target`
(reusing the engine) → unpack to GF(32) coeffs in the canonical orientation (§2.6) → 8 syndromes
in GF(1024) → Berlekamp-Massey (fixed-size, `deg(Λ)>4 ⇒ fail`) → Chien (root-count must equal
`deg(Λ)`) → Forney (magnitude must lie in GF(32), nonzero) → apply XOR at the located positions →
**re-verify via the existing verifier** → on success return the `Corrected` string + `Edits`;
any failure → `(_, false)`. Subs-only; no erasure seeding. Pure (no GUI, no RNG).

### 3.3 Phase A ships dormant
No GUI caller. Merged after TDD (§7) GREEN. Covers all three m\*1 by construction.

---

## 4. Phase B — HRP-dispatched typed entry + suggest→confirm

### 4.1 HRP-dispatched typed entry — a substantive `inputCodex32Flow` rework (NOT just a signature change)
md1/mk1 are codex32-family bech32 strings → the existing codex32 keypad (`newCodex32Keyboard`,
bech32 charset, b/i/o dimmed) serves all three; the user types `ms1…`/`md1…`/`mk1…`. **But
`inputCodex32Flow`'s per-frame validation, length windows, and live feedback are ALL codex32/ms-
specific in the fork** (R0 I-1/I-3) and MUST be reworked HRP-aware — this is not merely a
return-type change. Three coupled changes:

- **(a) Per-frame validation, HRP-dispatched.** Today the flow calls `codex32.New(kbd.Fragment)`
  unconditionally (gui.go:730); `New` applies the codex32 short/long constants (init=1, codex32
  NUMS target) for ANY HRP, so a correct `md1`/`mk1` fails `New` with `errInvalidChecksum`. The
  rework parses the HRP first (`ParsePrefix` exposes `HRP` even on a partial fragment) and
  dispatches the completeness/validity check: `ms` → `codex32.New` (→ `codex32.String`); `md` →
  `ValidMD`; `mk` → `ValidMK` (both → `mdmkText`).
- **(b) HRP-aware length windows (the I-1 + M-1 coordinate-system fix).** The "complete-but-
  invalid-in-a-valid-length-window" arming state — and the live "bad checksum" feedback — MUST
  gate on the **per-HRP** window. Stated in BOTH data-part (chars after the `xx1` prefix) AND
  total-string coordinates, to kill the conflation that caused I-1 (md/mk total = data-part + 3):

  | HRP | code | data-part | total-string | source |
  |---|---|---|---|---|
  | `ms` | short | 45..90 | **48..93** | `ShortCodeMin/MaxLength` (codex32.go:43) |
  | `ms` | long | 122..124 | **125..127** | `LongCodeMin/MaxLength` (codex32.go:44) |
  | `md` | regular | **≥13** | **≥16** | `ValidMD`: `len(data)≥mdmkShortSyms`, no upper bracket (mdmk.go:100,124) |
  | `mk` | regular | 14..93 | 17..96 | `mkRegular{Min,Max}Len` (mdmk.go:47-48) |
  | `mk` | long | 96..108 | 99..111 | `mkLong{Min,Max}Len` (mdmk.go:49-50) |
  | `mk` | reserved-invalid | 94..95 | 97..98 | rejected, never corrected (mdmk.go:146) |

  The codex32-only `48..93`/`125..127` *total* windows MUST NOT gate the md/mk path: `md1Regular`
  =24 (< 48) and `mk1Long`=111 (in the codex32 94..124 dead zone) would otherwise never arm the
  feedback or the "Fix?" trigger, leaving md/mk correction — the user's expanded scope — non-
  functional for those length classes (R0 I-1).
- **(c) Suppress the codex32-schema `ParsePrefix` feedback for md/mk (R0 I-3).** `codex32Feedback`
  surfaces `perr` from `ParsePrefix` eagerly (`if perr != nil { return Describe(perr) }`,
  codex32_polish.go:56-59). `ParsePrefix` applies the codex32 **share** header schema
  (`data[0]`=threshold∈{0,2..9}), so an md/mk fragment like `md1y…` yields `errInvalidThreshold`
  the moment `data[0]` is typed → a spurious "bad threshold" while typing a *valid* md/mk string.
  The rework MUST gate the `ParsePrefix`/`codex32FieldLine`-derived feedback to `HRP=="ms"` only;
  for md/mk the live feedback is HRP + length-window status + the per-HRP validity (no header parse).

**Return type + caller ripple (the inputWordsFlow lesson).** The entry returns **`any`** (a
`codex32.String` or a `mdmkText`); the existing `engraveObjectFlow` dispatch already routes both
(`case codex32.String:` → `engraveCodex32`; `case mdmkText:` → `mdmkFlow`, gui.go:1861-1866).
Changing `inputCodex32Flow`'s return from `codex32.String` to `any` ripples to its OTHER caller
`recoverCodex32Flow` (codex32-share recovery, ms-HRP only). Resolve by EITHER (a) a separate
generalized entry for the menu while `recoverCodex32Flow` keeps a codex32-only entry, OR (b)
`inputCodex32Flow` returns `any` and `recoverCodex32Flow` type-asserts `codex32.String` (rejecting
a non-ms share as "enter a codex32 share"). The plan must enumerate all `inputCodex32Flow` callers
and keep them compiling + behavior-preserving. Menu: relabel `case 2` to "Input m\*1 string" (or
add an entry) — the plan pins.

### 4.2 The suggest→confirm "Fix?" UX (panel-locked)
- **On-demand, never per-keystroke.** The per-frame HRP-aware validate (§4.1) stays the cheap live
  gate. The decoder fires only when the string is **complete-but-invalid-in-the-per-HRP valid-
  length-window** (§4.1(b) table; mirror `codex32Feedback`'s suppress-until-window discipline): in
  that state the OK button is hidden; add a **"Fix?"** nav affordance (mirror how the OK nav is
  conditionally added). Press → run `seedxor`-style on-demand `codex32.Correct(fragment)`.
- `Correct` → `(_, false)` → transient "no fix within 4 changes — check your typing"; keep editing.
- `Correct` → `(result, true)` → a **new confirm screen** (NOT `confirmCodex32Flow`): title a
  question ("Apply this correction?"); body **anchored on the per-position diff** (`pos 17: 'b' →
  '8'`, one line per edit, ≤4) — the **universal anchor for all three m\*1** (§2.3). **For `ms`
  only**, ALSO append the decoded header fields (`codex32FieldLine`/`ParsePrefix` of the corrected
  string). **For md/mk** there is NO fork header parser (`ParsePrefix` returns `errInvalidThreshold`
  on md/mk data — R0 I-2; an md/mk header decoder is out of scope), so the per-position diff IS the
  md/mk anchor — shown with the explicit instruction to compare each changed position to the source
  card. **Button2 drained every frame** (the R0-C1 idiom); Button1 = reject (keep editing, fragment
  untouched); Button3 = accept.
- **On accept:** replace `kbd.Fragment` with `result.Corrected`, then fall through to the
  **existing** HRP-aware validate gate (§4.1) — the now-valid string flows the normal OK→engrave
  path (no special "trust me" branch). Works for all three HRPs (the corrected string re-validates
  per its HRP).

### 4.3 Recovery interaction
`recoverCodex32Flow` (codex32 share recovery) routes through the typed entry, so corrected ms1
shares benefit automatically — but each entered share still goes through `ConsistentShares` +
`Interpolate` as today (correction only fixes a single share's transcription before it joins the
set).

---

## 5. Error/UX strings
Decoder failure → keep the existing "bad checksum". "Fix?" with no correction → "No fix within 4
changes — check your typing." Confirm screen → "Apply this correction?" + the per-position diff
(all m\*1) + the `id · thr · share` header line (**ms only**) + "Compare each changed position to
your source card."

## 6. File manifest

| File | Phase | Change |
|---|---|---|
| `codex32/gf1024.go` | A | **new** — GF(1024) on the fork's GF(32) (`fe.Mul`). |
| `codex32/correct.go` | A | **new** — `Correct` + syndromes/BM/Chien/Forney + re-verify + per-code dispatch (reusing `checksum.go`/`mdmk.go` params). |
| `codex32/correct_test.go` + `codex32/testdata/` | A | **new** — Rust parity vectors (corrupt→correct), negative cross-constant, field self-tests, orientation pin. |
| `gui/codex32_polish.go` | B | **modify** — HRP-aware live validation/status/feedback (`codex32Feedback`/`codex32StatusLine`: per-HRP length window + dispatch `New`/`ValidMD`/`ValidMK`; suppress the codex32 `ParsePrefix` feedback for md/mk — R0 I-1/I-3); the "Fix?" affordance hook; the new diff-confirm screen (universal per-position diff; `id·thr·share` ms-only — R0 I-2; Button2-drain). |
| `gui/gui.go` | B | **modify** — `inputCodex32Flow` HRP-dispatched per-frame validation (`New`/`ValidMD`/`ValidMK`) + HRP-aware length windows (§4.1(b) table) + `any` return (per §4.1, callers reconciled); menu relabel. |
| `gui/*_test.go` | B | **modify** — HRP-dispatch entry + correction-gate + no-hang tests. |
| `.github/workflows/*` | A | **modify** — add `tinygo build -target=pico-plus2 ./codex32/`. |

Unchanged (reused): `gf32.go`, `checksum.go`, `mdmk.go` params/verifiers, `engraveCodex32`,
`mdmkFlow`, `backupSeedStringFlow`, `ParsePrefix`/`Describe`, the keypad, `engraveObjectFlow` dispatch.

## 7. TDD (Phase A first; Rust parity vectors)
**Phase A:** corrupt-then-correct vectors sourced from the Rust `ms/md/mk-codec .../tests/
bch_decode.rs`/`bch_adversarial.rs` (known positions + magnitudes) — for all three codes, 1/2/4
substitution + the 5-error rejection case; the **negative cross-constant** test; the field
self-tests (β/γ order, ζ³=1, generator roots); the **orientation pin** (a vector that fails if
LSB/MSB is flipped). Rust-sourced only (§2.5). `go test ./codex32/` + `tinygo build`.
**Phase B:** HRP-dispatch entry (ms1→engraveCodex32, md/mk→mdmkFlow); the "Fix?" → confirm →
accept→re-validate path (assert the diff is shown + the corrected string engraves); reject→keep
editing; suppress-when-uncorrectable; the Button2-drain no-hang; `recoverCodex32Flow` still green.
Host: `go test ./codex32/ ./gui/`. Existing guards (codex32/mdmk detection, SLIP-39, BIP-39,
backup goldens) stay green.

## 8. Process
Phase A: spec(this §0/§2/§3/§7) → R0 loop → plan R0 loop → single-implementer TDD in worktree
`seedhammer-wt-mstar-a` (branch `feat/mstar-correct-decoder` off `04a1e95`), Rust vectors as
oracle → whole-diff execution review → merge dormant no-ff signed → push `bg002h`. Phase B:
branch off post-A `main` → plan R0 → TDD (`seedhammer-wt-mstar-b`) → review → merge. Reviews →
`design/agent-reports/seedhammer-mstar-correction-*`. Signed+DCO, Brian Goss. No upstream PR.
