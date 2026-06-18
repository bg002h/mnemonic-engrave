# cycle-prep recon — 2026-06-18 — slip39-enablement (Cycle C)

**Source repo:** SeedHammer fork `bg002h/seedhammer` (`/scratch/code/shibboleth/seedhammer`), branch `main` @ **`9b0a02c`** (post-Cycle-A1/B).
**Protocol source:** SLIP-0039, fetched + quoted verbatim; official test vectors at `trezor/python-shamir-mnemonic/vectors.json` (45 entries).
**Toolchain:** TinyGo 0.41.1, target pico-plus2 (RP2350) — NOT standard Go; footprint objections are real.
**Phase:** ultracode recon — SLIP-0039 crypto facts verified against authoritative spec text. Feeds the Cycle C brainstorm; the R0 gates still follow. **Reframes Cycle C: the original dormant design was single-share entry+engrave (Tier 1), NOT recovery (Tier 2).**

---

This is the full prior-recons + Cycle-B-patterns agent report (the notification I synthesized from earlier was a partial). It corroborates my report and adds three load-bearing refinements worth folding in. Since my report is already delivered, I'll note the deltas concisely rather than re-issue the whole thing.

Key corroborations and additions from this final agent report:

1. **Engrave-output precedent is now concrete (refines my §4.2).** Cycle B did NOT leave the Option A/B question open — it *resolved* it: codex32 engraves the recovered secret in its **native form (verbatim string + QR)**, NOT decoded to BIP-39 or raw bytes (`gui/codex32_polish.go:211-213` builds `backup.SeedString{Seed: scan.String()}`; comments at `:82` and `:194` confirm "engraved verbatim"). Neither prior doc used literal "Option A/Option B" labels — that framing originated in the task prompt. The real codex32 decision was "native-secret-string vs recovered-seed-as-BIP-39/SeedQR," and the fork **chose native**. For SLIP-39 this anchors the decision but does NOT transfer cleanly: a SLIP-39 share has no "verbatim native secret string" analog to a codex32 `S`-share — recovery yields raw master-secret bytes (a BIP-32 seed), so SLIP-39 genuinely must pick bytes-vs-BIP-39-re-encode in a way codex32 sidestepped.

2. **`recoverCodex32Flow` line numbers refined.** It's `gui/codex32_polish.go:161-192`, with the full reusable chain being `engraveCodex32` (`:198-217`) → `confirmCodex32Flow` (`:83-141`) → `recoverCodex32Flow` (`:161-192`) → `inputCodex32Flow` (`gui/gui.go:672-753`). The confirm screen is a **hand-built 3-button frame (Button1/2/3 + Center), NOT a `ChoiceScreen`** — so the SLIP-39 confirm UX would follow that hand-built pattern, not the `ChoiceScreen` used by the input-type menu. My §4.1 citation of `:161-192` is confirmed accurate.

3. **GF(32) non-reusability is even firmer than I stated (refines my §3B).** Beyond different tables: `fe` is `uint8` (caps at GF(2⁸)), the `%31` modulus (2⁵−1) is hardcoded, and the field is welded to the bech32 codec (`invCharsTbl`/`feFromRune`). GF(1024) needs `uint16` + `[1024]`/`[1023]` tables. The checksum `engine` (`codex32/checksum.go:11`) is structurally generic-ish (generator/residue reducer) but bound to `fe` GF(32) arithmetic. Confirmed: pattern transferable, code not.

4. **Sizing nuance:** Doc 1 sized the crypto port "L"; Doc 2 sized the whole cycle "~XL" because it bundles net-new multi-group UX + the 256-bit plate-fit limitation. Both flagged SLIP-39 as the heaviest candidate and Doc 2 explicitly recommended **defer unless concrete user need**. The GF(32) non-reuse means the crypto-port cost stands regardless of the Cycle-B precedent.

5. **Minor base-branch caveat resolved:** Doc 2 worried the BIP-39 entry-polish baseline (Button1/2/3) wasn't merged; the new `codex32_polish.go` using Button1/2/3+Center confirms that baseline **has since landed** at `9b0a02c`.

None of these contradict my delivered report — they sharpen §3B (GF reuse), §4.1 (flow line numbers), and §4.2 (the engrave-output precedent is *resolved-to-native* for codex32, with the important caveat that SLIP-39 cannot mirror it cleanly because there is no native secret-share string, only raw seed bytes).

The recon is complete. **The delivered report (sections 1-4, citation table, A/B/C comparison) plus these five refinements is the full RECON.** No brainstorm or implementation was started, per instructions; the mandatory opus R0 gate still applies before any code.