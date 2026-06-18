# SPEC — SeedHammer m*1 BCH error-correction (suggest → confirm)

**Status:** draft for the opus-architect R0 gate.
**Base:** fork `main` `04a1e95`. Fork-side only (no upstream PR).
**Recon:** `design/cycle-prep-recon-mstar-correction.md` (`71ab189`).
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
   confirm gate MUST show the **per-position diff** (`pos N: 'x' → 'y'`, ≤4 lines) **+ the decoded
   header fields** (`id · thr · share`) — "confirm the EDIT against your card", NOT a blob "looks
   right?". This is the Seed-XOR-fingerprint discipline (verify a small, human-comparable derived
   artifact). The gate is a **new** screen, NOT `confirmCodex32Flow` (whose Button3 engraves).
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
   correctness gate and a loop bound. **Add `tinygo build -target=pico-plus2` of `codex32` to CI**
   (the Slice-1 lesson — host `go test` never compiles the device build).

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

### 4.1 HRP-dispatched typed entry (the new md/mk entry)
md1/mk1 are codex32-family bech32 strings → the existing codex32 keypad (`newCodex32Keyboard`,
bech32 charset, b/i/o dimmed) serves all three; the user types `ms1…`/`md1…`/`mk1…`. The typed
entry validates **per the parsed HRP** (`ParsePrefix` already exposes `HRP`): `ms` → `codex32.New`
(returns `codex32.String`); `md`/`mk` → `ValidMD`/`ValidMK` (returns `mdmkText`). It returns
**`any`** (a `codex32.String` or a `mdmkText`), which the existing `engraveObjectFlow` dispatch
routes (`case codex32.String:` → `engraveCodex32`; `case mdmkText:` → `mdmkFlow`).
**Wiring note for the R0 gate / plan (the inputWordsFlow lesson):** changing `inputCodex32Flow`'s
return from `codex32.String` to `any` ripples to its OTHER caller `recoverCodex32Flow`
(codex32-share recovery, ms-HRP only). Resolve by EITHER (a) a separate generalized entry for the
menu while `recoverCodex32Flow` keeps a codex32-only entry, OR (b) `inputCodex32Flow` returns
`any` and `recoverCodex32Flow` type-asserts `codex32.String` (rejecting a non-ms share as "enter a
codex32 share"). The plan must enumerate all `inputCodex32Flow` callers and keep them compiling +
behavior-preserving. Menu: relabel `case 2` to "Input m\*1 string" (or add an entry) — the plan pins.

### 4.2 The suggest→confirm "Fix?" UX (panel-locked)
- **On-demand, never per-keystroke.** The per-frame validate stays the cheap live gate. The
  decoder fires only when the string is **complete-but-invalid-in-a-valid-length-window** (mirror
  `codex32Feedback`'s suppress-until-window discipline): in that state the OK button is hidden;
  add a **"Fix?"** nav affordance (mirror how the OK nav is conditionally added). Press → run
  `seedxor`-style on-demand `codex32.Correct(fragment)`.
- `Correct` → `(_, false)` → transient "no fix within 4 changes — check your typing"; keep editing.
- `Correct` → `(result, true)` → a **new confirm screen** (NOT `confirmCodex32Flow`): title a
  question ("Apply this correction?"); body = the per-position diff (`pos 17: 'b' → '8'`, one line
  per edit, ≤4) + the decoded header fields (`codex32FieldLine`/`ParsePrefix` of the corrected
  string) as the human-checkable anchor; **Button2 drained every frame** (the R0-C1 idiom);
  Button1 = reject (keep editing, fragment untouched); Button3 = accept.
- **On accept:** replace `kbd.Fragment` with `result.Corrected`, then fall through to the
  **existing** validate gate — the now-valid string flows the normal OK→engrave path (no special
  "trust me" branch). Works for all three HRPs (the corrected string re-validates per its HRP).

### 4.3 Recovery interaction
`recoverCodex32Flow` (codex32 share recovery) routes through the typed entry, so corrected ms1
shares benefit automatically — but each entered share still goes through `ConsistentShares` +
`Interpolate` as today (correction only fixes a single share's transcription before it joins the
set).

---

## 5. Error/UX strings
Decoder failure → keep the existing "bad checksum". "Fix?" with no correction → "No fix within 4
changes — check your typing." Confirm screen → "Apply this correction?" + the diff + header line +
"Compare each changed position to your source."

## 6. File manifest

| File | Phase | Change |
|---|---|---|
| `codex32/gf1024.go` | A | **new** — GF(1024) on the fork's GF(32) (`fe.Mul`). |
| `codex32/correct.go` | A | **new** — `Correct` + syndromes/BM/Chien/Forney + re-verify + per-code dispatch (reusing `checksum.go`/`mdmk.go` params). |
| `codex32/correct_test.go` + `codex32/testdata/` | A | **new** — Rust parity vectors (corrupt→correct), negative cross-constant, field self-tests, orientation pin. |
| `gui/codex32_polish.go` | B | **modify** — the "Fix?" affordance hook + the new diff-confirm screen (Button2-drain). |
| `gui/gui.go` | B | **modify** — `inputCodex32Flow` HRP-dispatch + `any` return (per §4.1, callers reconciled); menu relabel. |
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
