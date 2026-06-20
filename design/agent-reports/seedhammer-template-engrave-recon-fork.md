# RECON — SeedHammer fork delta to engrave a wallet-policy TEMPLATE md1

**Phase:** cycle-prep recon (research/recon). **Mode:** read-only on the fork source. No source modified, no build run.
**Fork:** `/scratch/code/shibboleth/seedhammer` @ `main` HEAD **`39cb5cf`** (`39cb5cf6351f6c87ab779e5688a60186f65412e0`).
**Toolkit (off-device sibling, primary source for the ports):** `/scratch/code/shibboleth/mnemonic-toolkit`.
**Scope:** single-sig AND non-taproot sortedmulti; taproot (`*_a`) refused (inherited).
**Date:** 2026-06-20.

---

## SYNC BLOCK

- The fork ALREADY decodes/expands/displays a keyless (template) md1 end-to-end: `validateXpubBytes` is a no-op when `pubPresent` is absent (`md/md.go:1074`); `ExpandedKey.XpubPresent` (`md/expand.go:63`) drives a real `expandTemplateOnly` GUI status (`gui/md1_expand.go:18,42-49`) routed to a read-only display (`gui/md1_gather.go:211-212`).
- The fork ALREADY engraves ANY scanned md1 string VERBATIM, template or full, via the generic post-scan `mdmkFlow` (`gui/gui.go:1972-2027`, engrave at :2023) — NO xpub gate on that path. This is the cheapest template-engrave surface and it is form-agnostic today.
- The xpub gate (`allSlotsHaveXpub`, `gui/multisig_supply.go:72`) is ONLY on the multisig **supply-and-derive-leg** path (`gui/multisig.go:83`) — it exists *because* that flow cross-matches a typed seed and derives the operator's mk1/ms1 leg. It is NOT a generic engrave gate.
- GAPS confirmed: (a) no `WalletDescriptorTemplateId` anywhere in the fork (grep empty); (b) both md1 encoders force `pubPresent:true` (`md/encode_singlesig.go:73`, `md/encode_multisig.go:147`); (c) `bundle/verify.go` stub binding is unconditionally `WalletPolicyId`-derived (`verify.go:116` via `WalletPolicyIDStubChunks`) — key-dependent, wrong for a keyless md1.
- `engraveSingleSig` and `engraveMultisig` are already distinct navigable programs (`gui/gui.go:151-152`); a template/full toggle is an INNER `ChoiceScreen` branch (precedent: `engraveMultisigFlow`'s "Supply policy"/"Build policy", `gui/multisig.go:40-55`) → adds **no** new program, does **not** trip the program-count guard `var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}` (`gui/gui.go:164`).

---

## FINDING 1 — Current fork state (gaps confirmed)

**1a. No `WalletDescriptorTemplateId`; only presence-significant `WalletPolicyId`.**
`grep -rn "WalletDescriptorTemplateId|TemplateId|template_id|is_wallet_policy" --include=*.go` over the whole fork returns **empty**. The only policy-identity primitive is `WalletPolicyId` / `WalletPolicyIDStub` (`md/walletpolicyid.go:30,104,119,127`). The package doc itself flags presence-significance: nulling pubkeys+fp yields a DIFFERENT id (`md/walletpolicyid_test.go:120-141`, `TestWalletPolicyIdPresenceSignificant`). So `WalletPolicyId` of a template md1 ≠ `WalletPolicyId` of the keyed md1 — the wrong binding source for a template.

**1b. Both md1 encoders force `pubPresent:true`.**
- Single-sig: `md/encode_singlesig.go:72-77` — `tlv: tlvSection{pubPresent: true, pubkeys: []idxPub{{idx:0, xpub:xpub}}, fpPresent:true, fingerprints:...}`. Always one pubkey, always fp.
- Multisig: `md/encode_multisig.go:146-151` — `tlv: tlvSection{pubPresent:true, pubkeys: pubkeys, fpPresent: len(fps)>0, fingerprints: fps}`. `n` pubkeys always; fp optional. There is **no keyless emit path** on-device.

**1c. The DECODER accepts a keyless md1 (pubPresent optional).**
`readTLVEntry` sets `section.pubPresent = true` only when a Pubkeys TLV (`tlvPubkeys`) is seen (`md/md.go:597-603`); absent → `pubPresent` stays false, no error. `validateXpubBytes` short-circuits when `!d.tlv.pubPresent` with the explicit comment "When the Pubkeys TLV is absent (template-only mode) this is a no-op" (`md/md.go:1073-1076`). `ExpandWalletPolicy` sets `XpubPresent: hasXpub(d, idx)` per slot (`md/expand.go:93`). So template md1 decode/expand is already first-class.

**1d. The supply path REFUSES a template-only md1 (by design, for the leg-derive flow).**
`allSlotsHaveXpub` returns false if `len(keys)==0` or any `!k.XpubPresent` (`gui/multisig_supply.go:72-82`). Sole production caller: `gui/multisig.go:83-86` in `supplyMultisigPolicyFlow`, which then types a seed, `findUserSlot` cross-matches it to a slot xpub (`gui/multisig.go:116`), and `deriveMultisigLeg` mints the operator's mk1/ms1 (`gui/multisig.go:140`). A keyless md1 has nothing to cross-match → the gate refuses BEFORE any seed is typed. The single-sig flow has **no supply-md1 path at all** — `engraveSingleSigFlow` always derives on-device via `md.EncodeSingleSig` (`gui/singlesig_derive.go:61`, sole non-test caller); supply-existing-md1 is multisig-only.
**Nuance (load-bearing for design-Q a):** `allSlotsHaveXpub` is NOT a global engrave gate. The generic scan→engrave `mdmkFlow` (`gui/gui.go:1972`) has no such gate and already engraves a scanned template md1 verbatim.

**1e. On-device verify stub binding is KEY-DEPENDENT.**
`bundle/verify.go` `checkStubBinding` compares `card.Stubs` against `md.WalletPolicyIDStubChunks(b.MD1)` (`verify.go:116`) — unconditionally the `WalletPolicyId`-derived stub. For a template md1 (no pubkeys) this computes the wrong stub: the toolkit binds a template's mk1 to the **`WalletDescriptorTemplateId`**-derived stub, not `WalletPolicyId` (toolkit `synthesize.rs:484,1146-1203` "re-root on the key-stable `WalletDescriptorTemplateId`"). `bundle.Verify` is the single-sig (T6a) composer; the multisig analog `verifyMultisig`/`deriveMultisigLeg` likewise pull the stub via `md.WalletPolicyIDStubChunks(suppliedMd1)` (`gui/multisig_derive.go`, reported by the GUI recon). All stub sites are key-dependent today.

---

## FINDING 2 — Where the template-engrave CHOICE slots in

**Programs already exist; no new program needed.** Enum: `engraveSingleSig`, `engraveMultisig` (`gui/gui.go:151-152`); dispatch `gui/gui.go:1512-1516`; main-menu plates `gui/gui.go:1877`; guard `gui/gui.go:164`. Adding a program between `bip85Derive` and `qaProgram` would make the guard array length ≠ 1 and fail the build — so do NOT add one.

**Multisig front-door is already a choose-or-supply `ChoiceScreen`** (`gui/multisig.go:40-55`): `{"Supply policy (md1)", "Build policy"}` → `supplyMultisigPolicyFlow` / `buildMultisigPolicyFlow`. A template path is a natural third sibling OR an inner branch — both stay inside the `engraveMultisig` program (the file's own doc says it "adds NO program … only branches inside the existing program's flow function", `gui/multisig.go:35-39`).

**Single-sig front-door** (`gui/singlesig.go`, `engraveSingleSigFlow`) is a straight derive pipeline (seed → pick → passphrase → full/watch-only → derive → engrave; reported by the single-sig recon). It has no supply branch; a "supply/engrave a template md1" option would be a NEW inner branch there OR — preferred — routed through the existing generic surfaces below.

**The cheapest, already-working surface: `mdmkFlow`** (`gui/gui.go:1972-2027`). Reached from a top-level scan: `scan.go:72` recognizes a valid md1 via `codex32.ValidMD` → `mdmkText` → `engraveObjectFlow` (`gui/gui.go:1918-1919`) → `mdmkFlow`. It offers `"Inspect descriptor"` + engrave-variant labels and engraves the chosen variant verbatim (`gui/gui.go:2023`, `NewEngraveScreen(ctx, engravings[idx])`). `engravings` comes from `validateMdmk` on the raw string (`gui/gui.go:1930`, format-agnostic). **A template md1 scanned here already engraves verbatim today.** Caveat: a multi-chunk template md1 only fully resolves on the **Inspect** path (`md1GatherFlow`, `gui/gui.go:2012` on `ErrChunkedUnsupported`); the engrave path holds only the single scanned chunk — so a verbatim multi-plate template engrave still needs a gather step wired to engrave (mirror `gui/md1_gather.go:79` + `gui/bundle_flow.go:303` plate plan).

**Quoted anchors:**
- `gui/multisig.go:41-54` front-door ChoiceScreen + dispatch.
- `gui/gui.go:1985-1991` — `mdmkFlow` builds `"Inspect descriptor"` + labels for an md1.
- `gui/gui.go:2019-2025` — non-inspect choice → `NewEngraveScreen(...engravings[idx]).Engrave`.
- `gui/gui.go:164` — program-count guard.

---

## FINDING 3 — The verify/stub port (exact sites)

**Port target A — `md.WalletDescriptorTemplateId` (Go), net-new.** Port the toolkit's template-id (the key-stable id over the keyless canonical tree; toolkit `synthesize.rs:50,484,1146-1203`). Add alongside `md/walletpolicyid.go` (e.g. `md/walletdescriptortemplateid.go`): `WalletDescriptorTemplateId(d *descriptor) [16]byte`, `WalletDescriptorTemplateIDStub`, and the `…Chunks` reassembly forms mirroring `walletpolicyid.go:104-130`. Needs an `is_wallet_policy()` analog in Go: derive it from `d.tlv.pubPresent` (true ⇒ wallet policy; false ⇒ template) — the decoder field already exists (`md/md.go:529,603`).

**Port target B — a FORM-AWARE stub in the binding check.** The toolkit rule (verify against authoritative toolkit source, not the draft): stub source = `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId` (`synthesize.rs:1154` "`WalletDescriptorTemplateId` (NOT `WalletPolicyId`, which a keyless md1 …)"). Sites to make form-aware:
- `bundle/verify.go:116` (`checkStubBinding`) — replace the unconditional `md.WalletPolicyIDStubChunks(b.MD1)` with a form-dispatched helper (new `md.BindingStubChunks(strs)` that picks template vs policy id by `pubPresent`).
- `gui/multisig_derive.go` `deriveMultisigLeg` — the `md.WalletPolicyIDStubChunks(suppliedMd1)` stub source (so a template-supplied multisig binds its mk1 correctly).
- Any other `WalletPolicyIDStub*` stub-source call: `gui/singlesig_derive.go:66-70` (single-sig leg stub), `gui/multisig_build.go` (`assembleBuildPolicy` stub) — these are FULL-policy producers so they stay `WalletPolicyId`, but the binding/verify CHECK must accept either id-form.

**Relax target — `allSlotsHaveXpub` on the template path.** Do NOT widen `allSlotsHaveXpub` itself (it correctly gates the cross-match/derive flow). Instead, on a dedicated template path, simply do not call it: engrave the supplied/scanned template md1 verbatim with no seed cross-match and no leg derivation. The decode/expand already yields `expandTemplateOnly` (`gui/md1_expand.go:43-48`) — branch on that to a verbatim engrave instead of the read-only display (`gui/md1_gather.go:211`).

---

## FINDING 4 — Key design questions (recommendations)

**(a) Template md1 SOURCE — ingest vs on-device emit.**
**Recommend: INGEST a user-supplied template md1 (produced off-device by `toolkit bundle --md1-form=template`) and engrave VERBATIM, for BOTH single-sig and multisig.** Primary-source support: the toolkit owns this — `Md1Form::{Policy, Template}` (toolkit `synthesize.rs:54-65`, `is_template()` at :63-65), keyless template synthesis (`synthesize.rs:218-284`), the `bundle --md1-form=template` CLI surface (`error.rs:311-317`, `restore.rs:122`). The fork's verbatim-engrave invariant already exists ("the device never re-encodes a multisig descriptor", `gui/multisig_engrave.go` I-2; verbatim plate plan `gui/bundle_flow.go:301-318`). On-device EMIT would require a net-new keyless Go encoder (both encoders force `pubPresent:true`, F1b) plus a keyless `WalletDescriptorTemplateId` encode-time fill — more attack surface on a device that should stay a dumb engraver. Verbatim ingest reuses `gui/gui.go:1972`/`gui/md1_gather.go` wholesale. (If on-device emit is ever wanted, it is strictly larger and gated behind the same template-id port.)

**(b) On-device search vs display-only.**
**Recommend: the fork must NOT run any key-permutation search on-device.** It ENGRAVES the template verbatim and, at most, DISPLAYS the off-device recovery-time estimate (the `seedhammer-template-engrave-key-search-time-estimate` UI feature) sourced from a number the toolkit computes/embeds. The toolkit owns the recompose/search: `permutation_search.rs` exists there, not in the fork (grep of the fork's Go for permutation/factorial search → none). The RP2350 is slow and air-gapped; running `n!`-class search on-device is a non-goal. The estimate is display-only text — no crypto, no new identity math — so it composes with verbatim-engrave at near-zero risk.

**(c) Scope — non-taproot only.**
**Confirmed.** The decoder's `classifyPolicy`/`multiPolicy` return renderable only for `tagMulti`/`tagSortedMulti`; `tagMultiA`/`tagSortedMultiA` (taproot) fall through to `PolicyComplex` → non-renderable (`md/md.go:1318-1328`, enum `md/md.go:1189-1190`). `scriptForTemplate` explicitly lists `sortedmulti_a` among "not bip380-expressible" shapes → `expandUnsupported`/display-only, never a verified policy (`gui/md1_expand.go:104-120`). A single-key `tr(@N)` (taproot **single-sig**, no script tree) IS renderable (`md/md.go:1282-1284`; `bip380.P2TR` at `gui/md1_expand.go:93`) — so taproot SINGLE-sig is in scope, taproot MULTISIG (`*_a`) is refused. The fork inherits the constellation refusal at the classify/encode layer; no extra refusal code needed.

---

## FINDING 5 — Plate-cost angle (located, not derived)

Chunk→plate mapping: **one plate per chunk string.** `gui/bundle_flow.go:291-318` — `bundlePlate{plateIdx, plateTotal}`, `bundlePlatePlan` flattens each card's `c.strings` into one plate per string (`plateTotal: len(c.strings)`, :311; "A standalone md1 card yields exactly 1 plate", :302). Same one-string-one-plate model in the standalone scan/engrave path (`validateMdmk` yields per-string plate variants, `gui/gui.go:1930`) and `multiPlateEngrave` (`gui/derive_xpub.go:263-301`, "Plate i of total"). A template md1 (pubkeys-null) has a much smaller canonical payload than the keyed form (the single-sig keyed payload is "~81 bytes / >320 bits, so it always chunks — ~3 strings", `md/encode_singlesig.go:8-11`); stripping the 65-byte-per-key Pubkeys TLV typically collapses a template to ~1 chunk/plate vs ~2-3 for a full policy. Exact chunk count is the chunker's call (`md.split`); not derived here.

---

## FORK DELTA VERDICT

**What to PORT (from toolkit primary source):**
1. `md.WalletDescriptorTemplateId` + `…Stub` + `…Chunks` (Go), net-new file mirroring `md/walletpolicyid.go`, sourced from toolkit `synthesize.rs:484,1146-1203`.
2. A Go `is_wallet_policy()` analog = `d.tlv.pubPresent` (field already decoded, `md/md.go:529,603`).
3. A form-aware binding-stub helper `md.BindingStubChunks(strs)` = `pubPresent ? WalletPolicyIDStub : WalletDescriptorTemplateIDStub`.

**What to RELAX / re-wire (NOT widen the existing gate):**
- `bundle/verify.go:116` `checkStubBinding` → call the form-aware stub helper.
- `gui/multisig_derive.go` `deriveMultisigLeg` stub source → form-aware (only matters if a template multisig leg is ever derived on-device — see below; for pure verbatim engrave, leg derivation is skipped).
- Keep `allSlotsHaveXpub` (`gui/multisig_supply.go:72`) UNCHANGED; route templates around it (branch on `expandTemplateOnly`, `gui/md1_expand.go:43-48`), do not relax it.

**Where the CHOICE goes:**
- **Preferred (smallest):** reuse the generic scan→`mdmkFlow` (`gui/gui.go:1972`) + extend it (or `md1GatherFlow`, `gui/md1_gather.go:79`) so the `expandTemplateOnly` case offers "Engrave template" (verbatim, gathered multi-chunk) in addition to the read-only display. One inner branch, no new program, form-agnostic for single-sig AND multisig.
- **Alternative:** a third front-door choice in `engraveMultisigFlow` (`gui/multisig.go:44`) — "Supply template (md1)" — and a symmetric inner branch in `engraveSingleSigFlow`. Still no new program.

**Sizing:**
- **md/template-id port + form-aware stub: S–M** (one new file + one helper + the `verify.go` rewire; pattern fully precedented by `walletpolicyid.go`).
- **Single-sig template engrave: S** if routed through `mdmkFlow`/gather (verbatim, no derive); **M** if a dedicated `engraveSingleSigFlow` branch is wanted.
- **Multisig template engrave: S–M** — verbatim engrave is S (route around `allSlotsHaveXpub`); a full "supply template" front-door branch with multi-chunk gather + recovery-time display is M.
- **Recovery-time estimate UI: S** (display-only text from a toolkit-supplied number).

**Refused shapes (inherited, no new code):** taproot multisig `sortedmulti_a` / `multi_a` / taptree / unsorted `multi` → `PolicyComplex`/`expandUnsupported`, display-only (`md/md.go:1318-1328`, `gui/md1_expand.go:104-120`). ms1 secret cards refused on the supply/gather path (`gui/multisig_supply.go:26-28,58-59`; classify-level refusal noted at `gui/multisig_supply.go:16-17`).

**Open items to flag for the brainstorm/R0:**
- For a TEMPLATE multisig: should the device derive an operator leg at all (it can't cross-match without xpubs), or is template engrave strictly "engrave the shared template + the operator hand-supplies/derives keys off-device via the toolkit"? Recommend: template engrave is VERBATIM-ONLY (no on-device leg derive); the toolkit recomposes the keyed md1 later. This keeps `deriveMultisigLeg`'s key-dependent stub on the FULL path only and sidesteps porting template-id into the leg-derive path.
- Confirm the chunked template fits ≤1-2 plates so the verbatim multi-chunk gather→engrave is bounded (locate via `md.split`; not derived here).
