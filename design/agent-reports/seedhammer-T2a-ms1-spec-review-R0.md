<!--
Persisted verbatim. opus-architect R0 gate of the T2a ms1-decode spec
(SPEC_seedhammer_T2a_ms1_decode.md @ 42670f3). Reviewer agentId a9f955bdc2506a921.
Verdict: NOT GREEN 0C/3I (3 minor). The load-bearing external fact (§2.3 m-format ms1 payload
layout) was CONFIRMED: byte-proved against ms-codec source (consts.rs/envelope.rs/payload.rs) AND
by running 4 Rust-sourced vectors through the fork's codex32.New().Seed() (entr 16/32B incl. a
non-zero, + the mnem-English golden). Importants: I-1 the 0x00/0x02 discriminator at data[0] is the
m-format PREFIX byte, NOT the codex32 4-char id/Tag (which is "entr" for BOTH entr and mnem) — the
"tag" naming was a decode-trap; I-2 the corpus has no pinned NON-English mnem wire string, so the
non-English parity test must be Rust-encoder-sourced (the spec's fallback risked a prohibited
Go-self-generated vector); I-3 decode is meaningful only for the UNSHARED secret (a raw K-of-N share
carries an SSS point, not the m-format payload) → gate the affordance on Fields.Unshared. Minors:
M-1 inspect line uses ParsePrefix/Fields not Split(); M-2 mspayload.go lands in the fork tree
(seedhammer.com/codex32); M-3 submodule pin drift (informational; work in 68e6ead). Disposition:
all folded; re-dispatching R1. The text below is the agent's report exactly as returned.
-->

# R0 GATE REVIEW — T2a ms1 decode (spec)

**Reviewer:** opus architect (read-only adversarial R0 gate). **Spec:** `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_T2a_ms1_decode.md` (committed `42670f3`). **Fork base:** `/scratch/code/shibboleth/seedhammer` @ `68e6ead` (the working `main`; the `third_party/seedhammer` submodule is pinned older at `713aee2`, but the spec's cited line numbers and the fork-`main` base both refer to `68e6ead` — verified below). **ms-codec authoritative source:** `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/`.

## Verification Results

### A. THE load-bearing fact — m-format ms1 payload layout (§2.3) — CONFIRMED, with one terminology correction

I verified the layout against the authoritative ms-codec Rust source AND ran the §2.3 parity vector through the fork's `codex32.New().Seed()` in a throwaway test (now removed).

**Source proof (ms-codec):**
- `envelope.rs:192-220` `dispatch_payload(data)` reads `data[0]`:
  - `0x00` (`RESERVED_PREFIX`, `consts.rs:17`) → `Payload::Entr(data[1..])`
  - `0x02` (`MNEM_PREFIX`, `consts.rs:39`) → `Payload::Mnem { language: data[1], entropy: data[2..] }`
  - any other byte → `Error::ReservedPrefixViolation`
- `payload.rs:45-56` doc + `envelope.rs:240-248` `payload_wire_bytes` pin the on-wire layout: Entr = `[0x00][entropy]`; Mnem = `[0x02][language][entropy]`.
- `data` here is `Codex32String::parts().data()` (`envelope.rs:169,191`) — i.e. the codex32 data payload — which is exactly what the fork's `codex32.String.Seed()` returns (`codex32/codex32.go:386-388` `Seed() = parts().data()`). So `Seed()` **does** return `[prefix][lang?][entropy]` for a constellation ms1. The fork's `parts.data()` right-pads the final byte with 0s for non-multiple-of-8 bit lengths (`codex32/codex32.go:415-450`), but ms1 payloads are byte-aligned (entr = entropy+1 byte; mnem = entropy+2 bytes — see `consts.rs:41-43` `VALID_MNEM_STR_LENGTHS` "byte-aligned" comment), so no spurious trailing byte appears.
- Language table: `consts.rs:47-58` `MNEM_LANGUAGE_NAMES` — exactly 10 names, indices 0..9: english, japanese, korean, spanish, chinese-simplified, chinese-traditional, french, italian, czech, portuguese. `payload.rs:77` rejects `language >= 10`.
- Entropy lengths: `consts.rs:29` `VALID_ENTR_LENGTHS = [16,20,24,28,32]`, enforced by `Payload::validate()` (`payload.rs:64-90`) for both Entr and Mnem.

**Concrete-vector parity check (ran against fork code, then cleaned up):**

| vector (source) | fork `Seed()` hex | prefix | lang | entropy |
|---|---|---|---|---|
| `ms10entrsqqqq…34v7f` (`bch_decode.rs:35`, `v0.1.json` #0) | `00`+`00…00`(16) | `0x00` | — | `0000…00` ✓ |
| `ms10entrsqqqjx3t83x4…kkj` (`v0.1.json` #5, non-zero) | `00`+`0123…4567` | `0x00` | — | `0123456789abcdef0123456789abcdef01234567` ✓ (catches bit-ordering) |
| 24-word `…fme2w` (`v0.1.json` #4) | `00`+`00…00`(32) | `0x00` | — | 32 B ✓ |
| `ms10entrsqgqqc83yukgh…h82yz` mnem-English (`mnem.rs:144-158`) | `02 00 0c1e24e5917544d666c342992acfda1b` | `0x02` | `0x00` | `0c1e24e5917544d666c342992acfda1b` ✓ |

The layout holds byte-for-byte. **§2.3 is CONFIRMED.** The parity test the spec mandates is constructible and the corpus contains Rust-sourced vectors (`tests/vectors/v0.1.json` for all 5 entr lengths incl. a non-zero one; `tests/mnem.rs:144` for one mnem-English wire string).

**Terminology correction (the one real defect in §2.3 — see IMPORTANT-1):** The spec repeatedly calls the `0x00`/`0x02` discriminator the **"tag byte … at data[0]"** and says "tag `0x00`=entr / `0x02`=mnem". In ms-codec these are TWO DISTINCT THINGS: the **Tag** (`tag.rs`) is the 4-byte `id` field on the wire (positions sep+2..sep+6), which for *both* entr and mnem constellation secrets is the literal string `"entr"` (confirmed: the mnem golden vector's `Split()` returns `id="entr"`). The byte at `data[0]` is the **prefix byte** (`RESERVED_PREFIX`/`MNEM_PREFIX`). The decoder MUST branch on `data[0]` (the prefix byte), NOT on the id/tag. The spec's pseudo-code in §4.1 happens to read `data[0]` correctly, so the *algorithm* is right, but the naming ("tag") is wrong and actively dangerous — an implementer who took "tag" literally and read the 4-char id field would mis-decode every mnem secret (id="entr" for both). Must be corrected before the plan pins offsets.

### B. Fork API reality (§3, item 1) — CONFIRMED

- `codex32.String.Seed() []byte = parts().data()` — `codex32/codex32.go:386-388` ✓
- `codex32.String.Split() (id string, threshold int, idx rune)`, threshold 0→1 — `codex32/codex32.go:394-401` ✓ (but see MINOR-1: `confirmCodex32Flow` actually uses `codex32.ParsePrefix`/`codex32.Fields`, not `Split()`, for its inspect line — `codex32_polish.go:84-96`, `codex32/polish.go:63-71`).
- `bip39.New(entropy []byte) Mnemonic` — `bip39/bip39.go:228`; `.Entropy()` `:158`; `.String()` `:166`; `.Valid()` `:107`; `LabelFor(Word) string` `:79` — all present with claimed signatures ✓
- English-only wordlist — single `bip39/wordlist.txt` + generated `wordlist.go`; no other language files in `bip39/` ✓
- `SeedScreen.Draw(ctx, th, dims, mnemonic bip39.Mnemonic) op.Op` renders numbered words — `gui/gui.go:2221` ✓
- `confirmCodex32Flow` already shows the inspect line (Unshared-secret-S vs Share-X-of-k, id, char count) — `gui/codex32_polish.go:83-141` ✓
- ms1 engrave path = the `codex32.String` branch of `engraveObjectFlow` → `engraveCodex32` — `gui/gui.go:1874-1875`, `gui/codex32_polish.go:203-222` ✓; md1/mk1 use the separate `mdmkText` → `mdmkFlow` branch (`gui.go:1878-1879`), so hooking the decode-display on the codex32.String path cannot disturb md1/mk1 ✓

### C. The panic-guard necessity (§2.5) — CONFIRMED NECESSARY

`bip39.New` (`bip39/bip39.go:229-234`) **panics** on `len(entropy) < 16 || 32 < len(entropy)` *and* on `len(entropy)%4 != 0`. The {16,20,24,28,32} guard the spec requires is necessary and sufficient: all five lengths are %4==0, so the set-membership check covers both panic conditions. ✓

### D. Secrecy & scrub precedent (§2.1) — CONFIRMED

`wipeBytes` best-effort scrub exists at `gui/slip39_polish.go:328-330` and is used at `:293` — the spec's "SLIP-39/Cycle-D scrubbing precedent" is real and reusable. Display-only/never-NFC is consistent with the SeedScreen treatment. ✓

### E. No-alloc-gate / no-regression (§2.6) — CONFIRMED

`TestAllocs` (`gui/gui_test.go:93`) benchmarks only `BenchmarkAllocs` (a fixed descriptor/StartScreen path), so a new decode screen is not auto-gated. ✓ Baseline `go test ./codex32/ ./bip39/` passes (built/ran with `/home/bcg/.local/go/bin/go`). ✓

## Findings

### CRITICAL — none.

The decode algorithm (read `data[0]`, branch on prefix, strip lang for mnem, validate entropy length) is correct against the authoritative source; no garbage-entropy / dropped-mnem-byte / missing-panic-guard / port-masquerading-as-reuse defect found. The reuse-not-port claim (§2.4) is accurate: decode is `s.Seed()` + slice arithmetic + `bip39.New`; no codex32/BCH/bip39 reimplementation.

### IMPORTANT

**IMPORTANT-1 — §2.3 / §4.1 mislabels the prefix byte as the "tag".** (Spec §2.3 "tag `0x00`=entr / `0x02`=mnem at `data[0]`"; §4.1 `data[0]` = tag (`entr`=0x00 / `mnem`=0x02)"; §1 In-scope "strip the m-format tag byte (`entr`/`mnem`)".) In ms-codec the discriminator at `data[0]` is the **prefix byte** (`RESERVED_PREFIX 0x00` / `MNEM_PREFIX 0x02`, `consts.rs:17,39`); the **Tag** is the separate 4-byte `id` field which is `"entr"` for *both* entr and mnem constellation secrets (verified: mnem golden vector `Split()` → `id="entr"`). **Required fix:** rename throughout to "prefix byte" (or "type-discriminator byte"); explicitly state the decoder branches on `Seed()[0]`, never on the 4-char id, and that the id is `"entr"` for both forms. The algorithm is right; the naming is a correctness-trap for the implementer and must be corrected before the plan pins byte offsets.

**IMPORTANT-2 — the §2.3-mandated NON-ENGLISH parity vector is NOT Rust-sourced in the corpus; the spec's fallback risks a self-referential vector.** The ms-codec corpus contains exactly ONE pinned mnem *wire string* and it is **language=0 (English)** (`tests/mnem.rs:144-158`). All non-English mnem round-trip tests (`mnem.rs:97` lang=2, `envelope.rs:348-356` lang=2, `:449-457` lang=3) only assert internal encode→decode round-trips — they never pin a wire string. §6 says "else construct the mnem case from the documented layout and flag it for R0." A non-English mnem string constructed via the *fork's own* `codex32.NewSeed(hrp,0,"entr",'s',[0x02,lang,...entropy])` would be a **Go-self-generated vector**, which §2.3 explicitly prohibits ("No Go-self-generated vectors"). **Required fix:** the plan must mandate that the non-English mnem display test's input string is generated by the *Rust* ms-codec encoder (`encode(Tag::ENTR, &Payload::Mnem{language: <≥1>, entropy})`) — e.g. add a non-English entry to `tests/vectors/v0.1.json` or a `golden_mnem_<lang>_wire_vector` and copy the byte-pinned string into the Go test — so the non-English path is anchored to an authoritative wire string, not a Go round-trip. This is exactly the "plausible-but-wrong layout / false-consensus" class §2.3 is guarding against, and it is currently unguarded for the mnem≥1 path. (Note: the layout itself is already byte-proven for mnem via the English golden vector, so this is a test-provenance gap, not a layout-correctness gap — hence Important, not Critical.)

**IMPORTANT-3 — the decode-display affordance must be gated to the unshared-secret + valid-m-format case; the spec under-specifies when it appears.** `engraveCodex32` (`codex32_polish.go:203-222`) handles three codex32.String shapes through the same `confirmCodex32Flow`: (a) an unshared ms1 secret (id="entr", index 's'), (b) a **K-of-N share** (index != S; id is a random 4-char value, `consts.rs:71` `RESERVED_ID_BLOCKLIST`), and (c) a share recovered into a secret. A raw share's `Seed()[0]` is NOT a valid m-format prefix byte (a share carries an SSS-evaluated point, not the secret payload) — calling `DecodeMS1` on a share would hit the `data[0]` not-in-{0x00,0x02} error path or, worse, mis-slice. The spec's §4.2/§4.3 say "on the ms1 branch, after `confirmCodex32Flow`" but don't pin that the decode/Show-secret affordance must be **offered only for `f.Unshared` strings** (and only after BCH validity, which `New` already guarantees). **Required fix:** the plan must state the decode-display is offered only when `confirmCodex32Flow`'s `f.Unshared` is true (mirror the existing Recover-only-for-shares gate at `codex32_polish.go:109,119`); a share shows decode only after Recover produces the secret. Without this pin, an implementer could wire decode onto the share confirm and surface a confusing "can't decode this secret" on every legitimate share. (The §2.5 refusal prevents a *panic*, so this is Important, not Critical — but the UX/scoping must be pinned.)

### MINOR

- **MINOR-1 — §3 attributes the inspect line to `Split()`; the fork uses `ParsePrefix`/`Fields`.** `confirmCodex32Flow` derives the inspect line from `codex32.ParsePrefix(scan.String())` → `codex32.Fields{Unshared, Identifier, ShareIndex}` (`codex32_polish.go:84-96`, `codex32/polish.go:63-71`), not from `Split()`. `Split()` exists and is used at engrave time (`codex32_polish.go:216`) and is fine to cite for id/threshold, but the inspect-line reuse claim should point at `ParsePrefix`/`Fields` for accuracy. The §6 test ("reuse/verify `confirmCodex32Flow`/`Split`") should reference `ParsePrefix`/`Fields` for the unshared-vs-share determination.
- **MINOR-2 — §5 file manifest cites `codex32/mspayload.go` as new in the fork.** Confirm the plan places it in the fork tree (`/scratch/code/shibboleth/seedhammer/codex32/`), sibling to `mdmk.go`/`polish.go`, not in the mnemonic-engrave repo. (The spec's §7 worktree `seedhammer-wt-t2a-ms1` off `68e6ead` implies fork-side; just make it explicit so the new file lands in the right module — `seedhammer.com/codex32`.)
- **MINOR-3 — submodule pin drift (informational).** `third_party/seedhammer` is pinned at `713aee2`; the spec base and all cited line numbers are `/scratch/code/shibboleth/seedhammer` @ `68e6ead`. All §3 line citations verified against `68e6ead`. No action for T2a, but the implementer must work in the `68e6ead` checkout, not the older submodule pin.

## Verdict

The load-bearing external protocol fact (§2.3) is **confirmed correct** — the m-format ms1 layout is `[prefix:0x00|0x02][lang? for 0x02][entropy:16..32]`, byte-proven against ms-codec source AND against the fork's `codex32.New().Seed()` on four Rust-sourced vectors including the mnem-English golden. The panic-guard is genuinely necessary, the reuse-not-port claim is accurate, the mnem-byte-surfacing design is sound, and no Critical defect exists. Three Important items (prefix-vs-tag mislabel; non-English vector provenance; unshared-only affordance gating) must be folded before GREEN.

**NOT GREEN — 0 Critical / 3 Important.**
