# SPEC — T2a: on-device ms1 decode→display (entropy / BIP-39 words + mnem language + inspect)

**Status:** **GREEN (R1, 0C/0I).** R0 (0C/3I) byte-CONFIRMED the m-format layout vs ms-codec source + 4 Rust vectors; folded I-1 (prefix-vs-tag rename) · I-2 (Rust-sourced non-English vector) · I-3 (unshared-only gate) · M-1/M-2. R1 (0C/1I) closed all but a residual §4.1 comment-rename + §6 cite → fixed + grep-verified mechanically (a 2-word doc fix, no design change; full architect re-dispatch skipped as disproportionate). Reviews: `design/agent-reports/seedhammer-T2a-ms1-spec-review-{R0,R1}.md`. Next: the T2a plan → plan R0.
**Roadmap:** `design/RECON_seedhammer_constellation_terminal.md` (tier T2) · cycle-prep: `design/cycle-prep-recon-T2-decode-display.md`.
**Base:** fork `main` `68e6ead`. Fork-side only (no upstream PR).

---

## 1. Goal & scope

Let the operator **decode a hand-typed ms1 secret on-device and see what it holds** — the recovered BIP-39 seed words (English), or for a non-English seed the entropy + the wordlist language name — plus the "this is the unshared secret vs share k-of-N" inspect line, for verification before engraving. Today the ms1 branch validates the checksum and engraves the string verbatim without ever showing the operator the seed it encodes.

### In scope (T2a)
- Decode the ms1 **m-format payload**: `codex32.String.Seed()` (raw data) → branch on the m-format **prefix byte** `Seed()[0]` (`0x00`=entr / `0x02`=mnem) → for `mnem` read the wordlist-language byte → the BIP-39 entropy. (NB: the discriminator is the **prefix byte**, NOT the codex32 4-char `id`/Tag — which is `"entr"` for BOTH entr and mnem secrets; see §2.3.)
- **Display:** for English (`entr`, or `mnem` language 0) render the BIP-39 words (reuse the existing on-device word rendering); for **non-English `mnem` (language ≥1)** render the **entropy hex + the language NAME + a "words not shown on this device" note** (see §2.4 — the fork ships only the English wordlist).
- The **inspect line** ("Unshared secret (S)" vs "Share X of a k-of-n set", id, char count) — already produced by `confirmCodex32Flow`; surface it alongside the decode.
- **Display only, SECRET.** No engrave change, never over NFC/QR; hand-typed input only.

### Out of scope (explicit)
- Shipping non-English BIP-39 wordlists (deferred enhancement; footprint-bounded on RP2350). T2a only NAMES the language for non-English.
- md1/mk1 decode (T2b/T2c). Address derivation (T1/T3). Any change to the codex32 BCH/string layer (already in-tree, parity-tested) — **T2a ports no checksum/codec code**, only the m-format payload interpretation.

---

## 2. Invariants (R0 must verify each — Critical if violated)

1. **SECRET, display-only.** The decoded entropy/words/hex are shown on-screen only — never engraved, never over NFC/QR, never logged. Hand-typed input only. Reuse the existing seed-word display treatment (`SeedScreen.Draw`). Best-effort scrub the entropy/words buffers after the display screen returns (Go GC won't; the SLIP-39/Cycle-D scrubbing precedent applies).
2. **The `mnem` language byte MUST be read AND surfaced — even when words can't be shown.** It is the whole point of the `mnem` form: a non-English seed silently recovered/displayed as English yields the wrong wallet. So decode MUST branch on the language byte and, for language ≥1, show the language NAME (never English words). Dropping/ignoring the language byte is a Critical correctness failure (the silent-wrong-wallet class).
3. **The m-format payload layout (an EXTERNAL PROTOCOL FACT) — R0-CONFIRMED against ms-codec source.** Layout: `Seed()[0]` is the **prefix byte** — `RESERVED_PREFIX 0x00`=entr / `MNEM_PREFIX 0x02`=mnem (`ms-codec/src/consts.rs:17,39`, `envelope.rs:192-220`); for mnem `data[1]` = language index 0–9 (`MNEM_LANGUAGE_NAMES`, `consts.rs:47-58`); entropy = the remaining 16/20/24/28/32 bytes (`consts.rs:29`, byte-aligned so no `parts.data()` pad artifact). **The prefix byte is NOT the codex32 `id`/Tag**: the 4-char id is `"entr"` for BOTH entr and mnem constellation secrets — the decoder MUST branch on `Seed()[0]`, never on `Split()`'s `id`. (R0 byte-proved this against four Rust-sourced vectors incl. the mnem-English golden; layout-correctness is settled.) **Parity-test provenance (I-2): the test vectors MUST be Rust-sourced, never Go-self-generated.** The corpus has pinned wire strings for all five `entr` lengths (`ms-codec/tests/vectors/v0.1.json`) and ONE mnem-English wire string (`tests/mnem.rs:144`); a **non-English mnem wire string** is NOT pinned in the corpus — the plan MUST obtain it from the **Rust ms-codec encoder** (add a `mnem` language-≥1 entry to `v0.1.json`/a golden and copy the byte-pinned string into the Go test), NOT construct it via the fork's own `codex32.NewSeed` round-trip.
4. **Deterministic, reuse-not-port.** Decode is a pure byte-slice operation; no CSPRNG. It REUSES `codex32.New`/`.Seed()`/`.Split()`, `bip39.New`/`LabelFor`/`.String()`, and the `SeedScreen` word widget — it does NOT reimplement codex32, BCH, or bip39 (all in-tree).
5. **Entropy-length validation + unknown-prefix refusal.** Accept only BIP-39 entropy lengths {16,20,24,28,32} after strip; an unknown prefix byte or a bad length → a clear "can't decode this secret" message, never a panic, never a wrong-length `bip39.New` (R0-confirmed: `bip39.New` PANICS on `len<16||>32` or `len%4!=0`; the {16,20,24,28,32} set covers both).
6. **No regression / no new alloc gate.** The ms1 engrave path stays intact; the decode-display is a new (non-`TestAllocs`-benchmarked) screen, but follows the fixed-slice nav discipline. Existing codex32/bip39/gui tests stay green.
7. **Offer decode ONLY for the UNSHARED SECRET (I-3 gate).** `engraveCodex32`/`confirmCodex32Flow` handle three codex32.String shapes: the unshared ms1 secret (`Fields.Unshared`, id `"entr"`, index `s`), a K-of-N **share** (index ≠ S — carries an SSS-evaluated point, NOT the m-format secret payload), and a recovered secret. `DecodeMS1` is meaningful ONLY for the unshared secret; a raw share's `Seed()[0]` is not a valid m-format prefix and must NOT be decoded. So the decode/"Show secret" affordance is offered **only when `f.Unshared` is true** (mirror the existing Recover-only-for-shares gate, `codex32_polish.go:109,119`); a share gets decode only after Recover→secret re-confirms as unshared.

---

## 3. Source facts (verified against fork `68e6ead`; the m-format layout pending R0 re-verification per §2.3)

- `codex32.String.Seed() []byte` = `parts().data()` (`codex32/codex32.go:386-388`) — the raw codex32 data payload (for ms1 = the m-format `[prefix][lang?][entropy]`). `codex32.String.Split() (id string, threshold int, idx rune)` (`:394`; threshold 0→1 for the unshared secret). `codex32.New` validates the BCH checksum (the existing ms1 entry gate).
- The inspect "unshared-secret vs share-k-of-N" determination uses `codex32.ParsePrefix(s)` → `codex32.Fields{Unshared, Identifier, ShareIndex}` (`codex32/polish.go:63-71`), which is what `confirmCodex32Flow` already renders (`gui/codex32_polish.go:84-96`) — reuse that, not a fresh `Split()`-based parse, for the inspect line and the §2.7 `Unshared` gate. (M-1)
- `bip39.New(entropy []byte) Mnemonic` (`bip39/bip39.go:228`); `Mnemonic.Entropy()` (`:158`), `.String()` (`:166`), `.Valid()` (`:107`); `LabelFor(Word) string` (`:79`). **English-only** wordlist (single generated `bip39/wordlist.txt`, 2048 words — no other language files in-tree).
- `SeedScreen.Draw` (`gui/gui.go:2221`) already renders BIP-39 words on-screen ("1: ABANDON" …) — the secret-word display widget to reuse.
- `confirmCodex32Flow` (`gui/codex32_polish.go:83-141`) already shows the inspect line (Unshared-secret vs Share-X-of-k, id, char count) and is the ms1 pre-engrave confirm. ms1 reaches engrave via the `codex32.String` branch of `inputCodex32Flow`→`engraveCodex32` (`gui/gui.go:1874`).
- ms-codec authoritative source (§2.3, R0-verified): `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/{payload,consts,envelope}.rs` (prefix bytes `consts.rs:17,39`; 10-name language table `consts.rs:47-58`; entropy lengths `consts.rs:29`) + vectors `ms-codec/tests/vectors/v0.1.json` + `tests/mnem.rs:144`.

---

## 4. Design

### 4.1 The payload decoder (deterministic, reuse-not-port)
A small decoder — `codex32` package (new file, m-format-specific, sibling to `mdmk.go`) OR gui-side; the plan pins — exposing roughly:
```
// MStarSecret holds the decoded m-format ms1 payload (the plan pins exact shape).
//   Prefix (entr=0x00 / mnem=0x02 — the Seed()[0] byte, NOT the id/Tag);
//   Language (0..9, 0=English); Entropy []byte (16..32, BIP-39 length).
func DecodeMS1(s String) (prefix, language int, entropy []byte, err error)
```
- Take `s.Seed()`; require `len ≥ 2`; **`data[0]` = the prefix byte** (`entr`=0x00 / `mnem`=0x02 — §2.3-confirmed; NOT the codex32 id/Tag, which is `"entr"` for both); for `mnem`, `data[1]` = language, entropy = `data[2:]`; for `entr`, entropy = `data[1:]`. Validate `len(entropy) ∈ {16,20,24,28,32}`. Unknown prefix / bad length → error (§2.5). `language` is the raw byte (0..9; >9 → error per ms-codec `payload.rs:77`).

### 4.2 The display screen `ms1DecodeFlow` (or fold into the ms1 confirm)
On the ms1 branch, **offered ONLY when `Fields.Unshared` is true** (§2.7 — never on a raw share), after `confirmCodex32Flow` (or as a "Show secret" affordance gated on `f.Unshared`, mirroring the Recover-only-for-shares gate), a display-only screen:
- **English (entr, or mnem lang 0):** `bip39.New(entropy)` → render the words with the `SeedScreen` word treatment (numbered list); show the inspect line (id, unshared/share-k-of-N).
- **Non-English (mnem lang ≥1):** show "**Language: <name>**" (the 10-name table), the **entropy hex**, and "Words not shown on this device — restore with a <name> BIP-39 wallet." (Surfaces the language so the operator can't mistake it for English — §2.2.)
- **Controls:** Button1 = Back (to the confirm/engrave screen); display-only — no engrave, no NFC, no mutation. Scrub buffers on return.
- Decode is computed once on entry (off any hot path).

### 4.3 Wiring
- `inputCodex32Flow`'s `codex32.String` (ms) path / `engraveCodex32` / `confirmCodex32Flow` — add the `f.Unshared`-gated "Show secret"/decode affordance + the display screen. The plan pins whether it's a new Button on `confirmCodex32Flow` (a free button, drained every frame, acted on only when `f.Unshared`) or a step before it. The engrave path is unchanged; md1/mk1 (the `mdmkText` branch) are untouched.

---

## 5. File manifest (indicative; plan pins)

| File | Change |
|---|---|
| `codex32/mspayload.go` (fork tree, pkg `seedhammer.com/codex32`, sibling to `mdmk.go`) | **new** — `DecodeMS1` (branch on prefix byte, strip lang, validate entropy length). |
| `codex32/mspayload_test.go` | **new** — the §2.3 Rust-sourced parity vectors (entr lengths + the mnem-English golden + a **Rust-encoder-sourced** non-English mnem vector per I-2) + length/unknown-prefix-refusal tests. |
| `gui/*ms1 display* .go` | **new/modify** — the decode-display screen (English words / non-English name+hex), reusing `SeedScreen` word rendering; the affordance on the ms1 confirm. |
| `gui/*_test.go` | **modify** — display tests (English words shown; non-English name+hex+warning shown, words NOT shown; inspect line; Back). |

Unchanged/reused: `codex32` BCH/string layer, `bip39`, `SeedScreen`, the ms1 engrave path.

## 6. TDD
- **Parity (§2.3, the load-bearing test):** Rust-sourced ms1 vectors → `DecodeMS1` yields the known prefix/language/entropy (assert byte-for-byte). Use the `entr` vectors from `ms-codec/tests/vectors/v0.1.json` (incl. a non-zero-entropy one to catch bit-ordering) + the mnem-English wire string (`tests/mnem.rs:144`) + a **non-English mnem wire string obtained from the Rust ms-codec encoder** (I-2 — add a language-≥1 entry to the corpus; do NOT round-trip via the fork's own encoder).
- **English display:** decode an English ms1 → the rendered frame shows the expected `bip39.New(entropy).String()` words.
- **Non-English display:** a `mnem` lang≥1 → frame shows the language name + entropy hex + the "words not shown" note, and does NOT show English words.
- **Validation:** unknown prefix byte / non-BIP-39 length → clean error message, no panic, no `bip39.New` on bad length.
- **Inspect:** unshared-secret vs share-k-of-N line correct (reuse/verify `confirmCodex32Flow`/`ParsePrefix`/`Fields`).
- **No regression:** ms1 engrave path unchanged; codex32/bip39/gui suites green. `go test ./codex32/ ./gui/ ./bip39/`.

## 7. Process
cycle-prep (done) → R0 loop → plan → R0 loop → single-implementer TDD in worktree `seedhammer-wt-t2a-ms1` (branch `feat/ms1-decode-display` off `68e6ead`) → whole-diff execution review → merge no-ff signed+DCO → push `bg002h`. **One review agent per gate, agentId tracked on ledger task #2, reconciled before advancing.** Reviews → `design/agent-reports/seedhammer-T2a-ms1-*`. Signed+DCO, Brian Goss.
