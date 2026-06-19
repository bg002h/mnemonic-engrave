# SPEC — T2a: on-device ms1 decode→display (entropy / BIP-39 words + mnem language + inspect)

**Status:** draft for the opus-architect R0 gate.
**Roadmap:** `design/RECON_seedhammer_constellation_terminal.md` (tier T2) · cycle-prep: `design/cycle-prep-recon-T2-decode-display.md`.
**Base:** fork `main` `68e6ead`. Fork-side only (no upstream PR).

---

## 1. Goal & scope

Let the operator **decode a hand-typed ms1 secret on-device and see what it holds** — the recovered BIP-39 seed words (English), or for a non-English seed the entropy + the wordlist language name — plus the "this is the unshared secret vs share k-of-N" inspect line, for verification before engraving. Today the ms1 branch validates the checksum and engraves the string verbatim without ever showing the operator the seed it encodes.

### In scope (T2a)
- Decode the ms1 **m-format payload**: `codex32.String.Seed()` (raw data) → strip the m-format tag byte (`entr`/`mnem`) → for `mnem` read the wordlist-language byte → the BIP-39 entropy.
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
3. **The m-format payload layout is an EXTERNAL PROTOCOL FACT — verify against authoritative source, not this spec.** The assumed layout (tag `0x00`=entr / `0x02`=mnem at `data[0]`; for mnem a language index 0–9 at `data[1]`; entropy = the remaining 16/20/24/28/32 bytes) is from the cycle-prep recon of `ms-codec/src/{payload,consts}.rs`. R0 MUST re-verify it against the ms-codec source AND require a **Rust-sourced parity vector**: a real constellation ms1 string (e.g. ms-codec's canonical `ms10entrsqqqq…34v7f` 12-word vector, or a vectors.json entry) whose known entropy/tag/language the test asserts `codex32.New(ms1).Seed()` (after strip) reproduces byte-for-byte. This guards the "plausible-but-wrong layout" / false-consensus class. **No Go-self-generated vectors.**
4. **Deterministic, reuse-not-port.** Decode is a pure byte-slice operation; no CSPRNG. It REUSES `codex32.New`/`.Seed()`/`.Split()`, `bip39.New`/`LabelFor`/`.String()`, and the `SeedScreen` word widget — it does NOT reimplement codex32, BCH, or bip39 (all in-tree).
5. **Entropy-length validation + unknown-tag refusal.** Accept only BIP-39 entropy lengths {16,20,24,28,32} after strip; an unknown tag byte or a bad length → a clear "can't decode this secret" message, never a panic, never a wrong-length `bip39.New`.
6. **No regression / no new alloc gate.** The ms1 engrave path stays intact; the decode-display is a new (non-`TestAllocs`-benchmarked) screen, but follows the fixed-slice nav discipline. Existing codex32/bip39/gui tests stay green.

---

## 3. Source facts (verified against fork `68e6ead`; the m-format layout pending R0 re-verification per §2.3)

- `codex32.String.Seed() []byte` = `parts().data()` (`codex32/codex32.go:386-388`) — the raw codex32 data payload (for ms1 = the m-format `[tag][lang?][entropy]`). `codex32.String.Split() (id string, threshold int, idx rune)` (`:394`; threshold 0→1 for the unshared secret). `codex32.New` validates the BCH checksum (the existing ms1 entry gate).
- `bip39.New(entropy []byte) Mnemonic` (`bip39/bip39.go:228`); `Mnemonic.Entropy()` (`:158`), `.String()` (`:166`), `.Valid()` (`:107`); `LabelFor(Word) string` (`:79`). **English-only** wordlist (single generated `bip39/wordlist.txt`, 2048 words — no other language files in-tree).
- `SeedScreen.Draw` (`gui/gui.go:2221`) already renders BIP-39 words on-screen ("1: ABANDON" …) — the secret-word display widget to reuse.
- `confirmCodex32Flow` (`gui/codex32_polish.go:83-141`) already shows the inspect line (Unshared-secret vs Share-X-of-k, id, char count) and is the ms1 pre-engrave confirm. ms1 reaches engrave via the `codex32.String` branch of `inputCodex32Flow`→`engraveCodex32` (`gui/gui.go:1874`).
- ms-codec authoritative source (for §2.3 verification): `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/{payload,consts,decode}.rs` (tag bytes, the 10-name language table, entropy lengths) + its test vectors.

---

## 4. Design

### 4.1 The payload decoder (deterministic, reuse-not-port)
A small decoder — `codex32` package (new file, m-format-specific, sibling to `mdmk.go`) OR gui-side; the plan pins — exposing roughly:
```
// MStarSecret holds the decoded m-format ms1 payload (the plan pins exact shape).
//   Tag (entr/mnem); Language (0..9, 0=English); Entropy []byte (16..32, BIP-39 length).
func DecodeMS1(s String) (tag, language int, entropy []byte, err error)
```
- Take `s.Seed()`; require `len ≥ 2`; `data[0]` = tag (`entr`=0x00 / `mnem`=0x02 per §2.3 — verify); for `mnem`, `data[1]` = language, entropy = `data[2:]`; for `entr`, entropy = `data[1:]`. Validate `len(entropy) ∈ {16,20,24,28,32}`. Unknown tag / bad length → error (§2.5). **Exact byte offsets pinned by the plan after R0 re-verifies §2.3.**

### 4.2 The display screen `ms1DecodeFlow` (or fold into the ms1 confirm)
On the ms1 branch, after `confirmCodex32Flow` (or as a "Show secret" affordance on it), a display-only screen:
- **English (entr, or mnem lang 0):** `bip39.New(entropy)` → render the words with the `SeedScreen` word treatment (numbered list); show the inspect line (id, unshared/share-k-of-N).
- **Non-English (mnem lang ≥1):** show "**Language: <name>**" (the 10-name table), the **entropy hex**, and "Words not shown on this device — restore with a <name> BIP-39 wallet." (Surfaces the language so the operator can't mistake it for English — §2.2.)
- **Controls:** Button1 = Back (to the confirm/engrave screen); display-only — no engrave, no NFC, no mutation. Scrub buffers on return.
- Decode is computed once on entry (off any hot path).

### 4.3 Wiring
- `inputCodex32Flow`'s `codex32.String` (ms) path / `engraveCodex32` / `confirmCodex32Flow` — add the "Show secret"/decode affordance + the display screen. The plan pins whether it's a new Button on `confirmCodex32Flow` or a step before it. The engrave path is unchanged.

---

## 5. File manifest (indicative; plan pins)

| File | Change |
|---|---|
| `codex32/mspayload.go` (or gui) | **new** — `DecodeMS1` (strip tag/lang, validate entropy length). |
| `codex32/mspayload_test.go` | **new** — the §2.3 Rust-sourced parity vector + length/tag-refusal tests. |
| `gui/*ms1 display* .go` | **new/modify** — the decode-display screen (English words / non-English name+hex), reusing `SeedScreen` word rendering; the affordance on the ms1 confirm. |
| `gui/*_test.go` | **modify** — display tests (English words shown; non-English name+hex+warning shown, words NOT shown; inspect line; Back). |

Unchanged/reused: `codex32` BCH/string layer, `bip39`, `SeedScreen`, the ms1 engrave path.

## 6. TDD
- **Parity (§2.3, the load-bearing test):** a real constellation ms1 vector → `DecodeMS1` yields the known tag/language/entropy (Rust-sourced; assert byte-for-byte). At least one `entr` and one `mnem` vector (English + a non-English language index) if the corpus has them; else construct the mnem case from the documented layout and flag it for R0.
- **English display:** decode an English ms1 → the rendered frame shows the expected `bip39.New(entropy).String()` words.
- **Non-English display:** a `mnem` lang≥1 → frame shows the language name + entropy hex + the "words not shown" note, and does NOT show English words.
- **Validation:** unknown tag / non-BIP-39 length → clean error message, no panic, no `bip39.New` on bad length.
- **Inspect:** unshared-secret vs share-k-of-N line correct (reuse/verify `confirmCodex32Flow`/`Split`).
- **No regression:** ms1 engrave path unchanged; codex32/bip39/gui suites green. `go test ./codex32/ ./gui/ ./bip39/`.

## 7. Process
cycle-prep (done) → R0 loop → plan → R0 loop → single-implementer TDD in worktree `seedhammer-wt-t2a-ms1` (branch `feat/ms1-decode-display` off `68e6ead`) → whole-diff execution review → merge no-ff signed+DCO → push `bg002h`. **One review agent per gate, agentId tracked on ledger task #2, reconciled before advancing.** Reviews → `design/agent-reports/seedhammer-T2a-ms1-*`. Signed+DCO, Brian Goss.
