<!--
Persisted verbatim. Architect panel (crypto-security lens), agentId aff35c5f5d49ca258. Verdict: proceed with 3 enforced/tested changes — hold-to-confirm of the BIP-39-entropy interpretation, best-effort secret scrubbing, always-on fingerprint display + safe-default labeled passphrase prompts. Skip constant-time theater; file verbatim-hex followup.
The text below is the agent's report exactly as returned; do not edit.
-->

# Design-input brief — Cycle D: on-device SLIP-0039 secret recovery

**Reviewer lens:** crypto-security + threat model. **Verdict scope:** high-level design input, read-only, pre-implementation.

Threat model I'm assuming (from the docs): a hand-held, single-user, air-gapped engraver; shares hand-typed; the adversary of concern is (a) someone who later gets physical hold of the device, (b) a supply-chain/firmware-compromise that turns the device malicious, and (c) the user's own foot-cannons (wrong passphrase, wrong backup convention). No network, no multi-tenant OS, no swap.

---

## Q1 — Is the engraver the right place to reconstruct the master SECRET?

**Reasoning.** This is the one genuine threat-model *escalation* in the cycle, and the spec underplays it. Today (`engraveSLIP39`, `slip39_polish.go:79`) the device parses one share and engraves it verbatim — it never holds material that can spend funds; a single SLIP-39 share is useless to a thief below threshold. After Cycle D, a successful recovery means the **full BIP-39 seed transits RAM** (`recoverSLIP39Flow` → `bip39.New(secret)` → `backupWalletFlow`). A compromised or malicious device that previously saw only one sub-threshold share now sees spendable key material. That is a real increase in blast radius, and it's the crux of the "should this be on-device at all" question.

Counter-weight: the *only* alternative that avoids it is CLI-only recovery — which the toolkit already does on a general-purpose computer. But the whole reason this device exists is that a general-purpose computer is a *worse* place to expose a seed (browser, network, swap, malware). An air-gapped, single-purpose, hand-typed device with NFC disabled (`scan.go:62`, SLIP-39 scan path commented out) is a *better* trust environment than the laptop running `me`/the toolkit. And the user is *already* trusting this device with the full seed in the existing 12/24-word and codex32-recover flows (`backupWalletFlow`, `recoverCodex32Flow` reconstructs the unshared codex32 secret today). So on-device recovery is **consistent with the device's existing posture**, not a new precedent — codex32 multi-share recovery (Cycle B) already reconstructs a full secret on-device.

**Recommendation: proceed on-device, but make the mitigations in invariant §2 *mechanically enforced and tested*, not just asserted.** Specifically, three are mandatory and one is currently missing:

1. **NFC off during recovery — already structurally true** (the scanner never yields a `slip39.Share`; shares come only from `inputSLIP39Flow`). Keep it that way; do not add a SLIP-39 NFC branch in this cycle. Good.
2. **No-persist** — the recovered seed must never be written to flash/SD/anywhere. The existing engrave path doesn't persist, so this holds *as long as* recovery reuses `backupWalletFlow` unchanged. Add a test/assertion that no new persistence is introduced.
3. **Memory scrubbing — the gap.** The Rust oracle scrubs aggressively (`Zeroizing` on every share value, EMS, digest payload, round keys; `mlock` page-pinning in `mod.rs:161/319`). **The Go firmware has *zero* zeroize discipline today** (grep found none in `gui/`, `slip39/`, `bip39/`, `codex32/`). The spec's invariant 2 says secrets "never leave the device" and §5.2 step 6 says "do not retain after return" — but Go's GC means a dropped `[]byte` lingers in the heap until reuse, and TinyGo's GC is conservative. **This invariant is currently aspirational.** See Risk #2 below for the concrete, proportionate ask. (`mlock` is irrelevant on RP2350 — no OS, no swap — so don't port that part; it's the *byte-wiping* that matters.)

Net: the *place* is defensible. The *posture* is only sound if scrubbing graduates from prose to code.

---

## Q2 — The engrave model (SPEC §3): silently engraving BIP-39 words from a possibly-Trezor-native backup

**Reasoning.** This is the highest-severity *correctness-as-security* issue in the cycle, and "document the assumption" is **not** enough. The recon (§4) and spec (§3) both verify the hard fact: in the *standard*, the recovered master secret **is the BIP-32 seed** (`from_seed(MS)` reproduces vector #1's `bip32_xprv`). The constellation convention treats it as BIP-39 *entropy* instead. These two interpretations of the same 16–32 bytes derive **completely different wallets** with **no detectable error** — the bytes are valid under both readings.

The failure mode is brutal and silent: a user with a *Trezor-native* SLIP-39 backup (the overwhelmingly common case in the wild — Trezor Model T, Keystone, etc.) recovers on this device, sees a clean 12/24-word mnemonic and a confident fingerprint, engraves a beautiful steel plate, destroys their paper shares, and has just permanently memorialized **the wrong wallet**. They will not discover this until they try to restore and find an empty wallet. This is exactly the "1 valid last word" class of plausible-but-wrong the CLAUDE.md recon rule exists to catch — except here the wrongness is *unobservable on-device*.

`backupWalletFlow`'s fingerprint display does **not** save them: it shows the fingerprint of the *BIP-39 interpretation* (`masterFingerprintFor` runs the words through BIP-39's PBKDF2), so it's internally consistent and looks legitimate — it cannot reveal that the BIP-32-seed interpretation would have been different. There is no on-device signal of the foot-cannon.

**Recommendation (concrete): mandatory on-device acknowledgement gate before the engrave, not just a doc note.** After a successful `Combine` and before `backupWalletFlow`, show a blocking screen along the lines of:

> "Recovered as BIP-39 seed. This is correct ONLY for backups created by this toolkit / from a BIP-39 phrase. If your shares came from a Trezor or other SLIP-39 wallet, this will engrave the WRONG seed. Hold to continue."

Use the existing `ConfirmWarningScreen` hold-to-confirm pattern (`gui.go:2079`) — it already exists for the "Discard Seed?" gate, so this is cheap and idiomatic. The spec's "must appear in the spec + an on-device or doc note" (§3) should be tightened to **"must appear as an on-device hold-to-confirm acknowledgement"** — a doc note is invisible at the moment of irreversible action.

I would **not** block the cycle on a verbatim-hex fallback (engraving raw MS bytes for the Trezor-native case), but I'd **file it as a FOLLOWUP**: it's the clean way to actually *support* Trezor-native backups later, and it sidesteps the interpretation ambiguity entirely (you engrave exactly what you recovered). For this cycle, the acknowledgement gate + scoping Trezor-native explicitly out (§3 already does) is the right line.

---

## Q3 — The digest gate & the wrong-passphrase property

**Reasoning.** The spec's *crypto* handling is correct in spirit and matches the oracle:

- **Digest gate is mandatory and correctly placed** (§4.3 `recoverSecret`, mirroring `mod.rs:430-458`): for any threshold ≥ 2, `HMAC-SHA256(R, S)[:4] == digest` must hold or the layer returns `errDigestVerificationFailed`. The spec correctly carries this at *both* levels (member and group) and correctly notes the **T==1 no-digest exception** (a single share IS the secret — no integrity check exists at that layer, by design). Good. This catches forged/typo'd/mismatched share *sets*.
- **The critical subtlety the spec gets right** (invariant §2.3): the digest gate verifies the **shares interpolate consistently** — it does **NOT** verify the **passphrase**. A wrong SLIP-39 passphrase produces a *different but fully valid* EMS→MS (the Feistel decrypt in `feistel.rs` always succeeds; there's no MAC over the passphrase — this is SLIP-39's deliberate plausible-deniability / hidden-wallet property). So digest-pass + wrong-passphrase = a clean, valid-looking, **wrong** seed, silently.

So there are **two distinct silent-wrong-seed channels**: wrong *interpretation* (Q2) and wrong *passphrase* (this question). Both terminate in a confident engrave.

The spec is right that "the UI must not claim to verify it" (§2.3), but it doesn't yet say what the UI *should* do, and "don't claim verification" is a weak negative. The danger is a user who used `""` (skip) when their backup had a passphrase, or fat-fingered the passphrase — same outcome as Q2: wrong plate, paper destroyed.

**Recommendation:**
1. **The passphrase prompt must default to "Skip" with an explicit warning that a passphrase changes the result silently** — the existing `ChoiceScreen` defaults to index 0 (`backupWalletFlow:1941` precedent), so make Skip index 0 and label it honestly: *"SLIP-39 passphrase? Most backups have none. A wrong passphrase silently recovers a different seed."*
2. **A post-recovery fingerprint display is warranted — but framed as a check-against-your-records, not a verification.** Show the BIP-39 master fingerprint (`masterFingerprintFor`, already computed in `backupWalletFlow`) *before* engraving, labeled: *"Fingerprint ABCD1234 — confirm this matches your wallet records before engraving."* This is the only on-device handle a user has to catch *both* the wrong-passphrase and (partially) the wrong-interpretation case, **if** they happen to know their real fingerprint. It cannot manufacture safety on its own (a first-time recovery has nothing to compare to), but for anyone with records it's the difference between catching the error and engraving it. `backupWalletFlow` *already* surfaces the fingerprint in the passphrase branch — extend it to **always** surface it on the recovery path. Cheap, high-value.

Net: crypto spec is correct; the **UX honesty** around "valid ≠ verified" needs to be explicit, defaults-safe, and fingerprint-surfacing.

---

## Q4 — The double-passphrase design (SLIP-39 EMS passphrase, then optional BIP-39 25th-word passphrase)

**Reasoning.** These are genuinely two different secrets feeding two different algorithms at two different stages:
- **SLIP-39 passphrase** → enters *only* the Feistel round function (`feistel.rs:188`, `password = [round_idx] || passphrase`); decrypts EMS → MS. Determines *which seed you recover*.
- **BIP-39 25th-word passphrase** → enters BIP-39's PBKDF2 in `masterFingerprintFor`/derivation; determines *which wallet a given seed derives*. Offered by `backupWalletFlow:1941` and **only affects the engraved fingerprint**, not the engraved words.

There is **no cryptographic cross-contamination** — they touch disjoint code paths and disjoint key schedules. The risk is purely **human confusion**: two "passphrase?" prompts in one flow, both optional, both defaulting to skip, semantically unrelated. A user who entered their passphrase at the *first* prompt may assume the second is a duplicate and skip it (or vice-versa), or may enter the same string in both believing it's "the passphrase."

**Recommendation: allow both in the same flow (they're legitimately independent), but enforce hard labeling discipline + sequencing.** The spec already flags this (§5.4 "Two-passphrase UX note", §2.3) and mandates explicit labels ("SLIP-39 passphrase (not a BIP-39 passphrase)"). Strengthen to:
1. **Label by *function*, not just name:** first prompt "SLIP-39 share passphrase — unlocks the shares"; second "BIP-39 wallet passphrase (25th word) — optional extra wallet." Distinct titles, distinct lead text.
2. **Never reuse the same `passphraseFlow`/keyboard instance** across the two (avoid any state bleed; they're separate `NewPassphraseKeyboard` instances anyway, but make it explicit and tested).
3. **Consider gating the second prompt behind the recovery acknowledgement** so the user has already mentally committed to "this is my recovered seed" before being asked about a 25th word — reduces conflation.

This is a UX-safety item, not a crypto-safety item. No need to forbid the combination. **Do add a test** that exercises both-passphrases-set and asserts the two strings reach their respective algorithms unmixed (the spec's §7 "passphrase-path test" should be extended to cover the double case).

---

## Q5 — Side-channel / memory hygiene on RP2350

**Reasoning.** Threat model: hand-held, single-user, no co-resident attacker process, no shared cache, no network, no swap. That eliminates the entire class of *remote/co-tenant timing and cache side-channels* that motivate constant-time GF(256) on servers. The realistic physical-access adversary against an RP2350 has *far* cheaper attacks than timing the table-driven `gfMul` — they have SWD/debug-port access, glitching, and cold-boot RAM reads, against which constant-time arithmetic is irrelevant.

Assessing each candidate mitigation:

| Mitigation | Verdict | Why |
|---|---|---|
| **Byte-table GF(256) `expTbl`/`logTbl` (data-dependent table indices)** | **Theater to "fix"** | The Rust uses tables; the spec mandates tables (§4.1). Cache-timing on these tables requires a co-resident attacker measuring cache state — impossible on a single-task MCU with no adversary process. Leave as-is. Constant-time GF here would be pure ritual. |
| **PBKDF2 timing** | **Theater** | The secret-dependent input to PBKDF2 is the passphrase; iteration count (2500·2^e) is public. There's no remote observer. Not worth a thought. |
| **Zeroizing recovered seed / EMS / share values / round keys after use** | **WORTH IT — the one real item** | This is *not* a side-channel concern; it's a **secret-lifetime / cold-boot / GC-residue** concern, and it's the genuine gap (Q1). After engrave, the seed bytes should not sit indefinitely in heap pages where a later compromise, a crash dump, or a physical RAM read can recover them. The Rust oracle does this everywhere; the Go firmware does it nowhere. |
| **`mlock`/page-pinning (Rust `mod.rs:161`)** | **Don't port** | No OS, no swap on RP2350. Meaningless here. |
| **`crypto/subtle.ConstantTimeCompare` for the digest check** | **Cheap, do it** | The digest comparison (`D[:4] == computed[:4]`) is a 4-byte equality. Using `subtle.ConstantTimeCompare` costs nothing and avoids a (theoretical) early-exit; it's idiomatic Go crypto hygiene and signals care to reviewers. Marginal real value but free. |

**Recommendation:** Skip all constant-time-arithmetic theater (tables are fine). **Do** add explicit byte-wiping of the recovered-secret / EMS / per-share-value / round-key buffers on the Go side — best-effort `for i := range b { b[i] = 0 }` after use in `Combine`, `feistelDecrypt`, and `recoverSLIP39Flow`. Acknowledge openly in the spec that **TinyGo's GC can copy/retain** these buffers so wiping is *best-effort defense-in-depth, not a guarantee* — but best-effort here is materially better than the current nothing, and it makes invariant §2 honest. Use `subtle.ConstantTimeCompare` for the digest gate as free hygiene.

---

## Top 3 security risks (ranked)

**1. Silent wrong-seed engrave from the entropy-vs-BIP-32-seed interpretation mismatch (Q2).**
The single most likely real-world loss-of-funds path: a Trezor-native SLIP-39 backup recovers to a clean, valid-looking, *wrong* BIP-39 seed with a confident fingerprint and no on-device error; user engraves it and destroys the paper. *Mitigation:* **mandatory on-device hold-to-confirm acknowledgement** (reuse `ConfirmWarningScreen`) before `backupWalletFlow`, stating the BIP-39-entropy assumption and that Trezor-native backups will engrave the wrong seed; keep Trezor-native explicitly out of scope; **file a verbatim-hex-fallback FOLLOWUP**. Do not ship with only a doc note.

**2. Invariant §2 ("secrets never persist/leak") is unenforced — no memory scrubbing exists in the firmware (Q1/Q5).**
The spec asserts the property in prose; the codebase has zero zeroize discipline, and recovery is the first flow to hold a *full* seed reconstructed from sub-threshold material. GC residue / crash-dump / physical-RAM exposure of the recovered seed is real. *Mitigation:* add best-effort byte-wiping of recovered-secret/EMS/share-value/round-key buffers in `Combine`, `feistelDecrypt`, `recoverSLIP39Flow`; document the TinyGo-GC best-effort caveat; **add a test asserting the scrub calls happen** (mirror the Rust's `pin_pages_for`-attempt-counter test pattern, adapted to a wipe hook). Don't overclaim — call it defense-in-depth.

**3. Wrong-passphrase / "valid ≠ verified" silent failure (Q3/Q4).**
A wrong (or wrongly-skipped) SLIP-39 passphrase yields a valid-but-different seed with no error — a second silent wrong-seed channel, compounded by the two-passphrase UX confusion. *Mitigation:* passphrase prompt defaults to Skip with an explicit "a passphrase silently changes the result" warning; **always** surface the recovered seed's master fingerprint before engrave as a "confirm against your records" check (not a verification claim); enforce distinct function-labeled prompts for the two passphrases; add a both-passphrases test asserting no cross-mixing.

*(Honorable mention, not top-3 but must-do: §4.4 panic-safety — all attacker/typo-controlled preconditions converted to returned errors before any interpolation/`gfDiv` runs, with a malformed-set-never-panics test. The spec already mandates this and flags it R0-critical; it's correctly scoped. The risk if missed is DoS/crash, not key compromise, hence below the silent-wrong-seed trio.)*

---

## Overall verdict

**Proceed with these changes.** The cryptographic design is sound and faithful to an already-audited oracle, the digest gate and panic-safety are correctly specified, and on-device recovery is a defensible posture for this air-gapped single-user device. But three prose-level safety claims must become *mechanically enforced, tested* behavior before implementation starts: (1) a **mandatory on-device hold-to-confirm acknowledgement** of the BIP-39-entropy interpretation, (2) **best-effort secret scrubbing** with an honest TinyGo-GC caveat, and (3) **always-on post-recovery fingerprint display** + safe-default, honestly-labeled passphrase prompts. Fold these into the spec and re-run the R0 gate. Skip all constant-time-arithmetic work — it's theater for this threat model.

*(Cited: SPEC §§2.1–2.6, 3, 4.1–4.5, 5.2–5.4, 7; recon §§1, 4, 5; `mod.rs:206-331/430-458`, `feistel.rs:100-166/188`, firmware `gui/slip39_polish.go:79`, `gui/gui.go:1929-1974/2062-2139`, `gui/scan.go:62`, `gui/codex32_polish.go:161-217`. Confirmed: firmware has no existing zeroize/mlock code; SLIP-39 NFC scan path is commented out at `scan.go:62`.)*
