<!--
Persisted verbatim. Architect design-input consult on how to handle the followup
`seedhammer-slip39-recovery-verbatim-hex`. Agent agentId a89706fc9955e8479. Bottom line:
DON'T build verbatim-hex (no wallet restores from raw seed/MS hex — verified vs Trezor docs);
the Trezor-native user is already served by the shipped verbatim-SHARE engrave; keep on-device
recovery constellation-only + document; the real (optional, S-sized) gap is to turn the
post-recovery one-way acknowledgement into a two-way fork routing 'not mine' to
engraveSLIP39Verbatim (no BIP-39 fingerprint on that arm). The text below is the agent's
report exactly as returned; do not edit.
-->

# Design-input brief — `seedhammer-slip39-recovery-verbatim-hex`

**Reviewer:** senior architect, read-only design input. **Base:** fork `main` `9db3fd2` (Cycle D shipped). **Question:** how to handle the filed follow-up to add a verbatim-hex engrave so Trezor-native SLIP-39 backups recover correctly instead of being scoped out.

The follow-up's premise is right that a real gap exists — but I think it names the wrong artifact, and the proportionate fix is mostly *subtractive* (re-frame the existing acknowledgement), not a new engrave path. Reasoning per question, then a bottom line.

---

## Q1 — Is hex even a useful backup artifact? (No — it names the wrong target)

**The premise behind "verbatim hex" is that the Trezor-native master secret IS the BIP-32 seed, so engraving it raw preserves the wallet.** That byte-fact is verified (SPEC §3: `from_seed(MS)` reproduces vector #1's `bip32_xprv`). But "the recovered bytes are the seed" does not imply "raw hex is a restorable backup." It is not.

- **No mainstream wallet restores from a raw BIP-32 seed / SLIP-39 master-secret hex.** The entire SLIP-39 ecosystem — Trezor (fw ≥ 2.7.2), Keystone, etc. — restores by **re-entering the share words on the device**; the master secret is an *internal* value the firmware reconstructs and never exposes for re-import (Trezor SLIP-39 docs, below). There is no "paste seed hex" recovery entry point on a Trezor. So a steel plate of 32 hex bytes is a backup of something **no consumer wallet will take back in.** It is strictly worse than what the user already holds.
- **What the user already holds IS the useful artifact.** A Trezor-native user arrives at this device holding **SLIP-39 share words on paper**. The genuinely useful durable backup of that is *the same share words on steel* — which the firmware **already engraves verbatim today** (`engraveSLIP39Verbatim`, `slip39_polish.go:453`, via `backup.Seed`; all lengths since D2). That path is convention-agnostic: it preserves the shares byte-for-byte regardless of whether they're Trezor-native or constellation, and it round-trips into any SLIP-39 wallet. The Trezor-native user is **not excluded** — they are served by the verbatim *share* path, just not by the *recover-then-engrave* path.
- **If you wanted a single-plate restorable artifact for the raw bytes**, the closest fit would be a **SeedQR/string of the bytes via `backup.SeedString`** (the codex32 precedent, `codex32_polish.go:212`) — but a QR/string of raw seed bytes has the *same* problem: nothing scans it back as a wallet. It would be a novel, non-standard artifact format we'd be inventing, with no reader. That's a foot-gun dressed as a feature.

**Recommendation:** "verbatim hex" is the wrong target. There is no consumer restore path for raw seed/MS hex, so engraving it produces an artifact the user cannot use — arguably *more* dangerous than the status quo (it *looks* like a backup). The artifact that actually serves the Trezor-native case is the **verbatim share words**, and that already ships. The follow-up's real content is therefore not "add hex" — it's "make sure the Trezor-native user is routed to the verbatim-share path instead of the wrong-seed BIP-39 path," which is a *framing* problem (Q2), not a *new-artifact* problem.

---

## Q2 — The convention-detection problem (the real issue — and it's already mostly handled)

The device cannot tell a Trezor-native share set from a constellation one; the bytes are valid under both readings (SPEC §3, crypto lens Risk #1). Cycle D's answer was a **post-recovery hold-to-confirm** (`engraveRecoveredSLIP39`, `slip39_polish.go:387`) acknowledging "BIP-39 seed; a Trezor/other backup would engrave the WRONG seed," plus an always-on fingerprint check (`confirmSLIP39Fingerprint:413`). That gate is sound. The weakness the follow-up implicitly targets is that the gate is a **dead end for the Trezor user**: it tells them "this is wrong for you" and then offers no correct action — so a determined Trezor user might hold-through it anyway and engrave the wrong seed.

Weighing the options the prompt lists:

- **(b) Ask up-front "where was this made?"** — *Rejected.* This is exactly the silent-wrong-seed surface the crypto lens warned about, just moved earlier. A non-expert frequently does *not know* their wallet's convention ("I have a Trezor and some word cards" — they don't know Trezor ≠ this toolkit at the byte-interpretation level). An up-front mis-answer is *more* dangerous than a post-recovery one because it sets a frame the user then trusts. It also can't be verified by anything downstream.
- **(c) Always engrave BOTH.** *Rejected.* Doubles plate cost/time, and engraving a useless hex/QR plate next to a words plate manufactures the false impression that the hex is a usable backup. Worst of both.
- **(a) Post-recovery artifact choice with strong framing** — *This is the right shape, but the choice should be "BIP-39 words" vs. "verbatim shares (Trezor/other)," NOT "BIP-39 words vs. hex."* The fix to the dead-end is: when the user declines the BIP-39 acknowledgement, **route them back to the verbatim-share engrave** (which already exists and is correct for their case) rather than just aborting. They keep their shares on steel — the correct, restorable, convention-agnostic artifact.

So the minimal-risk design is not a new engrave artifact at all. It's: **turn the existing one-way acknowledgement into a two-way fork — "Engrave as BIP-39 seed (this toolkit)" vs. "Not mine — engrave the shares verbatim instead."** That removes the dead end, gives the Trezor user a correct action, and adds zero new artifact format. The fingerprint check stays as the records-cross-check for the BIP-39 branch.

**Recommendation:** Keep the post-recovery model (option a), but its two arms are *BIP-39 words* and *verbatim shares* — not hex. The convention problem is solved by routing, not by inventing a hex plate.

---

## Q3 — On-device, or "use the toolkit CLI for non-constellation backups"?

The strategic question. My read:

- **The device's air-gapped value is real and the recovery path already exists for the constellation case.** Cycle D already reconstructs a full seed on-device (crypto lens Q1: it's consistent with the codex32-recover precedent). So we're not debating whether on-device recovery is acceptable — that ship sailed, gated and reviewed.
- **But the Trezor-native user does not benefit from on-device *recovery* at all**, because (Q1) there's no useful artifact to produce from their recovered seed on this device. Recovery only helps when the output is a restorable plate; for the Trezor case the restorable plate is the *shares*, which need no recovery. So building a Trezor-native *recovery* path is solving a problem the user doesn't have.
- **The toolkit CLI is the correct home for any genuine "convert my Trezor SLIP-39 to a different format" need** — it's already where SLIP-39 combine lives, it's not constrained to one plate, and a hex/xprv output there is at least pipeable into other tooling. On an air-gapped engraver, raw hex is a dead artifact (Q1).

**Recommendation:** Keep the engraver's recovery path **constellation-only** (BIP-39 words out), exactly as shipped. Do **not** build a Trezor-native recovery/hex path into the firmware. For the Trezor-native user the device's correct service is the **verbatim-share** engrave (already shipped); for anything beyond that, **document "Trezor/other SLIP-39 backups → recover with the toolkit, or engrave your shares verbatim here."** This maximizes air-gapped value where it helps and refuses to manufacture foot-cannon surface where it doesn't.

---

## Q4 — If we build it anyway: the cleanest shape (and why I'd still avoid hex)

If a future cycle decides the dead-end routing (Q2) isn't enough and wants an explicit non-words artifact, the cleanest design is **not hex** — it's routing the decliner to the **existing `engraveSLIP39Verbatim` share path**:

- **Artifact:** reuse `backup.Seed` (words+SeedQR) via the existing `engraveSLIP39Verbatim` — *not* a new `SeedString`-of-hex. The verbatim-share plate is already implemented, already fits-checked, already round-trips into Trezor. If a raw-bytes artifact were ever truly wanted, `backup.SeedString` is the mechanism (codex32 precedent), but per Q1 it has no reader — don't.
- **Confirm/fingerprint gates for the non-words path:** this is the subtle part the prompt flags. **The BIP-39 master fingerprint is meaningless for the verbatim-share path** (you're not deriving a BIP-39 wallet; you're re-engraving shares). So `confirmSLIP39Fingerprint` (`:413`) must **not** run on that arm — surfacing a BIP-39 fingerprint there would be an actively misleading "verification" of a number that has nothing to do with the user's wallet. The verbatim-share arm already has its *own* correct verification: the per-share `id #m/t` confirm in `confirmSLIP39Flow` (`:83`), which lets the user check identifier and index against their cards. That's the right check; reuse it, add nothing.
- **Composition:** trivial — the two arms already exist. `engraveRecoveredSLIP39` (`:384`) becomes a fork: BIP-39 acknowledgement → `confirmSLIP39Fingerprint` → `backupWalletFlow` (today's path), *or* "not mine" → fall back to `engraveSLIP39Verbatim(scan)` on the original first share. The loop in `engraveSLIP39` (`:359`) already supports `continue`-back, so wiring is a few lines.

**Recommendation:** if built, the "other" arm is **verbatim shares (reuse `engraveSLIP39Verbatim`), with NO BIP-39 fingerprint shown** — the fingerprint is convention-specific and would mislead on that arm. Avoid `SeedString`-of-hex entirely.

---

## Q5 — Scope & priority

This is **not a cycle.** The verified facts collapse it:

- The "verbatim hex" artifact (the follow-up's literal ask) is **won't-build** — no consumer wallet restores from it; it's a more-dangerous artifact than the status quo (Q1).
- The real gap (Trezor user hits a dead-end acknowledgement) is a **small routing change** on top of code that already exists: turn the one-way ack into a two-way fork that routes "not mine" back to `engraveSLIP39Verbatim`. ~30–60 LOC in `slip39_polish.go`, one new `ChoiceScreen`, a couple of GUI tests, and a doc line. It still runs the full gated pipeline (it touches a safety-critical seed flow), so the *process* cost dominates the *code* cost — but it's a small-addition cycle at most, not a crypto cycle.
- Honestly, even that is **optional polish.** Cycle D already serves the Trezor user (verbatim shares) and already warns them off the wrong path (the ack). The only thing missing is *graceful routing* instead of an abort. That's a real UX improvement but not a loss-of-funds gap — the loss-of-funds gap (silently engraving the wrong seed) is *already* gated.

**Sizing if built:** S (small). One file, one new choice screen + re-route, no crypto, no new artifact format. Most of the budget is the mandatory R0 + whole-diff review on a seed-bearing flow, not implementation.

---

## Bottom line

**Don't build "verbatim hex." Reframe and downgrade the follow-up.**

1. **Won't-build the named artifact (hex/raw-bytes plate).** It's verified-correct as *bytes* but useless as a *backup*: no wallet restores from a raw BIP-32 seed / SLIP-39 master-secret hex — the whole ecosystem (Trezor et al.) restores by re-entering **share words**. A hex plate is a more-dangerous-than-nothing artifact dressed as a backup. (Q1)

2. **The real, already-shipped answer for the Trezor-native user is the verbatim *share* engrave** (`engraveSLIP39Verbatim`, all lengths since D2) — convention-agnostic, restorable, already implemented. The Trezor user is *not* excluded by Cycle D; they're served by a different (correct) path. (Q1/Q3)

3. **Keep on-device recovery constellation-only** (BIP-39 words out, as shipped) and **document** "Trezor/other SLIP-39 → engrave your shares verbatim here, or recover with the `mnemonic-toolkit` CLI." On-device *recovery* gives the Trezor user nothing useful; the air-gapped value is in the verbatim-share and constellation-words paths that already exist. (Q3)

4. **Optional small follow-up (rescope, S):** convert the post-recovery one-way acknowledgement (`engraveRecoveredSLIP39`, `slip39_polish.go:387`) into a **two-way fork** — "Engrave as BIP-39 seed (this toolkit)" vs. "Not mine → engrave my shares verbatim instead" — so the Trezor user who declines the ack lands on the correct action instead of an abort. Reuse `engraveSLIP39Verbatim`; **do not** show the BIP-39 fingerprint on that arm (it's convention-specific and would mislead). This is UX polish over an already-closed loss-of-funds gate, not a new safety requirement. (Q2/Q4/Q5)

**Suggested disposition for `FOLLOWUPS.md`:** rewrite the `seedhammer-slip39-recovery-verbatim-hex` entry from "engrave hex via `backup.SeedString`" to **"won't-build hex (no restore path); instead, optional S-sized routing fix: post-recovery fork to the existing verbatim-share engrave for non-constellation backups, plus a doc line pointing Trezor users at verbatim-shares / the toolkit CLI."** That preserves the genuine insight the crypto lens had (don't strand the Trezor user) while dropping the artifact that turned out to be the wrong target.

**Key files cited:** `/scratch/code/shibboleth/seedhammer/gui/slip39_polish.go` (`engraveSLIP39:359`, `engraveRecoveredSLIP39:384`, `confirmSLIP39Fingerprint:413`, `engraveSLIP39Verbatim:453`); `/scratch/code/shibboleth/seedhammer/backup/backup.go` (`Seed`, `SeedString:25`, `EngraveSeedString:75`); `/scratch/code/shibboleth/seedhammer/gui/codex32_polish.go:212` (the `SeedString` verbatim precedent); `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_seedhammer_slip39_recovery.md` §3; `/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md` (the entry to rescope).

Sources for the load-bearing external fact (Trezor SLIP-39 restores via share words, not master-secret hex):
- [SLIP39 FAQs | Trezor](https://trezor.io/guides/backups-recovery/general-standards/slip39-faqs)
- [Recovering a wallet with Multi-share Backup | Trezor](https://trezor.io/guides/backups-recovery/advanced-wallets/recover-a-wallet-with-multi-share-backup)
- [SLIP-0039 | Trezor Firmware docs](https://docs.trezor.io/trezor-firmware/core/misc/slip0039.html)
