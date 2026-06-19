# R1 spec review — SeedHammer T4 (hand-typed seed → account xpub → engrave as mk1)

**Doc reviewed:** `design/SPEC_seedhammer_T4_seed_xpub_mk1.md` @ commit `1230d60` (folded)
**Prior round:** `design/agent-reports/seedhammer-T4-seed-xpub-mk1-spec-review-R0.md` (NOT GREEN — 1C/5I)
**Authoritative sources verified:** fork `/scratch/code/shibboleth/seedhammer` @ `a4d669d` (confirmed HEAD = `a4d669de0b78…`, the binding decoder oracle); reference wire spec `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec` (`913febc8…`); btcsuite `hdkeychain@v2.0.0` (module cache `btcutil/v2@v2.0.0`).
**Method (R1 brief):** verify each fold closed correctly against source with accurate citations + no new drift. Did NOT re-derive the R0-cleared architecture facts (round-trip invertibility, compact-73 invariant, stub-0 structural validity, no-NFC-writer) beyond confirming the folds didn't disturb them. Ultracode recon: 4 parallel source-verification subagents (C-1 BCH-init; I-1 ExtendedKey.Zero/Derive scrub-sequence hazards; I-5/I-4 GUI lockstep + picker; I-2/M-1/M-2/M-3 wire/type facts) + the reviewer's own direct reads of `gui/gui.go` (program enum/StartScreen/uiFlow/layoutMainPlates/layoutMainPager/ChoiceScreen.Draw), `bip32/bip32.go`, `bip39/bip39.go`, `gui/mk1_inspect.go`, `gui/theme.go`, `gui/slip39_polish.go`, `cmd/controller/platform_sh2.go`, and the Rust `xpub_compact.rs` / `encode.rs` / `pipeline.rs` / `header.rs` / `chunk.rs` / `consts.rs`.

---

## Fold verification

### C-1 (CRITICAL → CLOSED) — BCH-GENERATE must use the mk1 verifyMDMK-style engine, not codex32.NewSeed's init
**CLOSED.** §4.1 now carries an explicit `**C-1 (CRITICAL — BCH-init trap):**` paragraph mandating the `verifyMDMK`-style engine and warning against cloning `NewSeed`'s checksum step. Verified against source:
- `mdmkPolymodInitLo uint64 = 0x23181b3` at **`codex32/mdmk.go:39`** — value matches the spec's `POLYMOD_INIT = 0x23181b3`. (The Go identifier is `mdmkPolymodInitLo`, not literally `POLYMOD_INIT`; the spec uses `POLYMOD_INIT` as the logical/spec name and cites the right line + value, so this is accurate, not drift.)
- mk targets at **`mdmk.go:58-62`**: `mkRegularTargetHi=0x1 / mkRegularTargetLo=0x62435f91072fa5c`, `mkLongTargetHi=0x418 / mkLongTargetLo=0x90d7e441cbe97273` — the spec's `mkRegularTargetHi/Lo` + `mkLongTargetHi/Lo` names are exact.
- `verifyMDMK` engine setup at **`mdmk.go:103-107`**: `&engine{generator: generator, residue: unpackSyms(0, mdmkPolymodInitLo, n), target: unpackSyms(targetHi, targetLo, n)}` — exactly the "mk1 generator + mk1 targets + POLYMOD_INIT residue" the spec says to mirror. Line cite is correct.
- `codex32.NewSeed` (**`codex32.go:279-383`**) builds its engine via `newShortChecksum()`/`newLongChecksum()` whose `residue` is the hardcoded `feQ…feP` codex32 init (`codex32.go:36-46/57-66`) — confirmed DIFFERENT from `0x23181b3`, so the spec's "don't clone NewSeed's checksum step verbatim — it silently fails `ValidMK`" warning is correct.
- Engine is target/init-agnostic — `engine{generator,residue,target}` struct + `inputFe`/`inputTarget` operate purely on fields (`checksum.go:11-18,123-170`), so a GENERATE wrapper reusing it with the mk1 init is sound.
- Regular/long selection by data-part length confirmed at **`mdmk.go:47-50`** (`mkRegularMinLen=14/MaxLen=93`, `mkLongMinLen=96/MaxLen=108`; 94-95 reserved-invalid) and the `ValidMK` switch at `mdmk.go:139-147`. §4.1 mandates the known-answer test (each emitted chunk passes `codex32.ValidMK`; full set round-trips via `mk.Decode`).
The §4.1 instruction is now **unambiguous and correct**: it names the mk1 init, the mk targets, the `verifyMDMK` constructor to mirror, the "do NOT clone NewSeed" trap, and the KAT. No residual ambiguity.

### I-1 (IMPORTANT → CLOSED) — scrub master + intermediate ExtendedKeys via (*ExtendedKey).Zero(); capture master FP before zeroing
**CLOSED, with the spec'd sequence verified hazard-free.**
- `(*hdkeychain.ExtendedKey).Zero()` EXISTS at **`extendedkey.go:634`**, zeroing `key`, `pubKey`, `chainCode`, `parentFP` (+ nilling version/key, resetting depth/childNum/isPrivate) — the R0 "hook exists" claim and the §2.5 cite are correct.
- `bip32.Derive` (**`bip32.go:43-53`**) walks `key.Derive(p)` and `.Neuter()`s ONLY the final key with NO intermediate `.Zero()` — confirms the §4.1 rationale that master + intermediates are left resident. `NewMaster` returns a PRIVATE key, so each `.Derive` yields a private intermediate — confirmed.
- **Double-zero hazard: NONE.** `Neuter()` (`extendedkey.go:486-504`) constructs and returns a NEW `ExtendedKey` from `pubKeyBytes()`/chainCode/etc.; it does not mutate the receiver. So `acct := k.Neuter(); k.Zero()` zeroes only the private receiver and cannot corrupt `acct`. Each key object in the spec'd path-walk (`k := master; for c { next := k.Derive(c); k.Zero(); k = next }`) is a distinct object, zeroed exactly once.
- **Use-after-zero hazard: NONE.** The xpub is serialized from `acct` (a separate object); `k.Zero()` after `Neuter()` does not touch `acct`. (`String()` even guards a zeroed key, returning "zeroed extended key".) Ordering Neuter-then-Zero is correct.
- **Master FP timing: CORRECT.** `bip32.Fingerprint` (`bip32.go:38-41`) reads the pubkey via `SerializeCompressed()`; §4.1 mandates capturing the master FP BEFORE `master.Zero()` — exactly right, since `Zero()` nils the key.
§4.1 ↔ §2.5(b) are coherent: §2.5(b) names the `.Zero()` hook + the "Derive neuters only the final key" rationale; §4.1 spells out the path-walk that zeroes master + each intermediate + the pre-neuter account key. No drift.

### I-2 (IMPORTANT → CLOSED) — clear the bip39.Mnemonic []Word root secret
**CLOSED.** `Mnemonic` is `[]Word` where `Word int` (**`bip39.go:22-24`**) — NOT `[]byte`. §2.5(c) correctly states the mnemonic must be cleared by zeroing the `[]Word` slice and that `wipeBytes` (`slip39_polish.go:330`, takes `[]byte`) does not apply — sensible and source-faithful. (`MnemonicSeed` returns a 64-byte `[]byte` at `bip39.go:217-226`, so `wipeBytes(seed)` for §2.5(a) is correctly typed.)

### I-3 (IMPORTANT → CLOSED) — multi-plate set-level abort semantics
**CLOSED.** §4.3 now defines the set-level contract: a partial set does NOT reassemble; on mid-sequence abort the flow MUST show an explicit "Incomplete: N of M plates engraved; this set cannot be restored — discard and start over" warning, NOT a silent done; re-entry re-derives+re-encodes deterministically (identical strings) rather than resuming; no completed-backup state is recorded for a partial set. Coherent with the deterministic-encode invariant (2.3) and the multi-plate invariant (2.6). The "deterministic re-derive → identical strings" claim is sound (the csid is deterministic per 2.3; verified below). §6 adds the multi-plate test.

### I-4 (IMPORTANT → CLOSED) — drop the mk1DisplayFlow "picker"; commit to a two-stage picker
**CLOSED.** `mk1DisplayFlow` (**`mk1_inspect.go:105-149`**) is confirmed a PURE display scroller: only `backBtn` (exit) + `pageBtn` (advance `start` by `shown`, wrap to 0); NO cursor, NO selected index, NO per-line hit region, NO chosen-value return (void). Correctly NOT reusable as a picker. §4.2 step 2 now commits to a two-stage `ChoiceScreen` picker (stage 1 = script type, stage 2 = network → one of 14 paths). The architecture is sound. (See M-5 below for one precision nit on the "≤7 within the proven ceiling" phrasing — non-blocking.)

### I-5 (IMPORTANT → CLOSED) — the new-program lockstep set is real + complete
**CLOSED.** All six lockstep sites verified real, and the set is complete (no missing site after a full `program`/`backupWallet`/`qaProgram` sweep of `gui/`):
- `program` enum at **`gui.go:145-150`** (`backupWallet=0`, `qaProgram=1`; `qaProgram` reachable only via the hidden NFC debug string, NOT navigable) — confirmed.
- `StartScreen.Flow` nav clamp at **`gui.go:1624-1635`** (`m.prog--; if <0 → backupWallet`; `m.prog++; if >backupWallet → 0`) — both edges clamp to `backupWallet`; a new navigable program is unreachable unless changed. Confirmed.
- `layoutMainPlates` at **`gui.go:1836-1844`** — `panic("invalid page")` for any page != `backupWallet`; a new program page panics unless a case is added. Confirmed.
- Pager page-count constants — TWO: `npage = int(backupWallet)+1` at **`gui.go:1828`** (gates L/R arrow rendering, `if npage>1`) and `npages = int(backupWallet)+1` at **`gui.go:1847`** (gates pager-dot count). The spec's plural "pager page-count constants" correctly covers both. Confirmed.
- Start-screen title `switch` at **`gui.go:1651-1654`** — only `case backupWallet`; a new program renders a blank title (mis-render) unless a case is added. Confirmed.
- `uiFlow` dispatch at **`gui.go:1476-1503`** (`switch act.prog { case qaProgram…; case backupWallet… }`) — a new program needs a new case or its flow is silently dropped. Confirmed.
§5 file manifest enumerates exactly this set as "all edited together (missing one panics/mis-renders)" — matches §4.2. §2.8 alloc-gate caveat (StartScreen now renders a multi-page pager → re-verify `TestAllocs`) is preserved and is correctly an implementation-phase check.

### M-1 (MINOR → CLOSED) — master FP (on-card origin) vs compact-73 parent FP
**CLOSED.** Two distinct fingerprints confirmed: bytecode `[fp]` = `origin_fingerprint` = MASTER fp (`key_card.rs:32-36`, "Master-key fingerprint identifying the seed"); compact-73 `parent_fingerprint` = the account xpub's PARENT fp (`xpub_compact.rs:36`, populated from `xpub.parent_fingerprint` at `:49`; fork reads `compact[4:8]` at `mk.go:383`). For depth-3/4 accounts they differ. The encode-side invariant validates only depth+child, NOT parent FP (`encode.rs:38-48`). §4.1's `Card` note states this exactly right (`Fingerprint` = master FP, distinct from compact-73's parent FP set by the xpub itself).

### M-2 (MINOR → CLOSED) — exact top-20-bit csid extraction
**CLOSED.** `(h[0]<<12)|(h[1]<<4)|(h[2]>>4)` is the correct big-endian top-20-bit extraction of `SHA-256(bytecode)`: h[0]→bits 19..12, h[1]→bits 11..4, h[2]>>4→bits 3..0, total 20 bits masked to `0xFFFFF`. The csid field is 20 bits with NO reserved value (`header.rs:27` `MAX_CHUNK_SET_ID=(1<<20)-1`; decoder checks only cross-chunk CONSISTENCY, never value — `chunk.rs:149`), so any deterministic value round-trips. §4.1 pins the exact arithmetic. (Note: mk-codec's default `encode()` uses a CSPRNG `fresh_chunk_set_id` at `pipeline.rs:45-49`, but its own docstring directs deterministic callers to `encode_with_chunk_set_id` (`pipeline.rs:53-55,67`); the Go `mk.Encode` mirrors the deterministic variant per invariant 2.3 because the device has no app CSPRNG. This intentional divergence is correctly documented in §2.3/§3/§4.1 — NOT a contradiction.)

### M-3 (MINOR → CLOSED) — N chunks, not hardcoded 2
**CLOSED.** Chunk count = `ceil(stream_len / CHUNKED_FRAGMENT_LONG_BYTES)`, with `CHUNKED_FRAGMENT_LONG_BYTES=53` (`consts.rs:39`), `SINGLE_STRING_LONG_BYTES=56` (`consts.rs:33`), formula `stream.len().div_ceil(frag_size).max(1)` (`chunk.rs:73`). A 1-stub no-fp card (~83-byte stream) → 2 chunks; a 3-stub/long-path card → 3+ chunks. §4.1 says "N chunks, **N is data-driven — do NOT hardcode 2**" and §6 says assert `>= 2`. Correct.

---

## Drift check

Skimmed the whole spec for internal consistency. No new contradiction introduced by the folds:
- **Invariant numbering** 2.1–2.9 is consistent; all §4 references (R0-M1/M2/M3, C-1, R0-I3/I4/I5) point at the matching findings.
- **§4.1 ↔ §2.5 scrub coherence:** §2.5(a) seed buffer ↔ §4.1 `defer wipeBytes(seed)`; §2.5(b) master+intermediates ↔ §4.1 path-walk `.Zero()` + "capture master FP before zeroing"; §2.5(c) mnemonic `[]Word` ↔ §4.2/§6 (best-effort index-zero). Coherent. Minor non-blocking note: §4.2 step 3 says "scrub the seed buffer" but does not restate the §2.5(c) mnemonic-slice scrub inline in the numbered flow steps; §2.5(c) is the binding invariant and §6 asserts it, so coverage is complete — not a contradiction.
- **§4.2 ↔ §5 lockstep:** the six sites listed in §4.2 are exactly the six in §5; both phrase it "missing one panics/mis-renders." Aligned.
- **§4.3 abort ↔ 2.6/2.3:** the deterministic re-derive claim is consistent with the deterministic-csid invariant.
- **Stub-0 (2.4)** unchanged and still consistent with the mandatory warning in §4.2 step 5 + §6.

---

## Findings

### CRITICAL
None.

### IMPORTANT
None.

### MINOR (fold opportunistically into the implementation plan; do NOT block)
- **M-5 (NEW, from the I-4 fold). The "≤7 entries, within the proven ceiling" phrasing in §4.2 step 2 overstates the shipped ceiling.** The largest shipped `ChoiceScreen` list is **5 entries** (`slip39_polish.go:45-51`), not 7; `ChoiceScreen.Draw` (`gui.go:1438-1470`) stacks entries (`h += c.Size.Y`) and centers the block with NO pagination and NO clipping, so an overflowing list renders its tail offscreen/unselectable. Stage 1 of the proposed picker (BIP-44/49/84/86/48/87) is **6 entries**, exceeding the proven 5. Geometric check on the actual SH2 hardware (480×320 — `platform_sh2.go:33-34`; content height = 320 − 2×`leadingSize`(44) = 232px; button style `poppins.Bold20` @ `LineHeightScale 0.70` ⇒ ~20-28px/entry): 6 entries ≈ 140-170px and even 7 ≈ 200px fit within 232px, so there is **no actual overflow on the target display** and the two-stage picker is functionally sound. The nit is purely the word "proven" (5 is proven; 6/7 are plausible-but-unshipped). The plan should (a) reword to "≤5-7 entries; renders within the 480×320 content area" and (b) add a layout/selection check for the 6-entry stage-1 list (TDD `runUI`/`click` won't catch a render overflow, so the plan must verify the last entry is hittable). Non-blocking: bounded, no architecture impact, fits the planned GUI test.
- **M-6 (NEW, residual; defense-in-depth only). `MnemonicSeed` builds an unscrubbed intermediate plaintext `sentence []byte`.** `bip39.MnemonicSeed` (`bip39.go:217-226`) assembles the space-joined word sentence (the literal seed phrase) into a local `sentence []byte` that PBKDF2 consumes and that is never wiped — a pre-existing library-internal buffer the T4 helper cannot reach from outside. This is precisely the class §2.5 directs the spec to "Document any residual the runtime/GC can't guarantee, as defense-in-depth." Consistent with the existing codebase; calling it out in the plan's security note suffices. Non-blocking.
- **R0 M-4 (carried, informational).** Stale family-token text (`mk-codec 0.2` self-id vs 0.4 runtime behavior) — no wire drift affects round-trip; the standard-path picker emits only depth-3/4 cards. Reconcile only if the plan asserts a "0.4" token. Unchanged from R0; informational.

(R0 M-1/M-2/M-3 are folded and verified CLOSED above.)

---

## Verdict

**GREEN — 0 Critical / 0 Important**

All R0 findings (C-1, I-1…I-5) closed correctly against source with accurate citations; all R0 minors (M-1/M-2/M-3) folded and verified. No new Critical/Important and no drift introduced by the folds: the BCH-init engine instruction is now unambiguous and mirrors the only mk1-correct constructor (`verifyMDMK`); the secret-scrub spine (seed `wipeBytes` + master/intermediate/pre-neuter `(*ExtendedKey).Zero()` + `[]Word` mnemonic clear, master-FP-before-zero) is source-faithful and free of double-zero / use-after-zero hazards; the mnemonic type, fingerprint distinction, csid bit-math, deterministic-csid divergence, and chunk-count formula all match source; the dropped `mk1DisplayFlow` picker is confirmed unusable and the two-stage picker is sound; and the six-site new-program lockstep set is real and complete. The two new MINORs (M-5 picker-ceiling wording + layout check; M-6 PBKDF2 intermediate-buffer residual) are spec-precision/defense-in-depth notes for the implementation plan and do not block. Proceed to `IMPLEMENTATION_PLAN_seedhammer_T4_seed_xpub_mk1.md` and its own R0 gate.
