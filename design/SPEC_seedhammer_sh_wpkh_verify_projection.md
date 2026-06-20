# SPEC — `sh(wpkh)` on-device-verify projection (P2SH-P2WPKH / BIP-49 `3…`)

FOLLOWUPS: `seedhammer-10b-walletpolicy-nits` **M-3** (✅ BUILD APPROVED 2026-06-19, gated cycle).
Fork: `/scratch/code/shibboleth/seedhammer`, HEAD `8eb51d7`.
Target: SeedHammer II firmware (`md/`, `gui/`). Mainnet-only.

---

## Status

DRAFT — awaiting opus R0 gate (0C/0I) before any implementation. Spec only; no code, no plan.

---

## Why

`sh(wpkh)` is P2SH-P2WPKH single-sig: a `wpkh` key wrapped in P2SH, yielding `3…` (BIP-49)
addresses. Today the device DECODES an `sh(wpkh)` md1 wallet-policy but does **not** project it
to a `*bip380.Descriptor`, so the on-device "Show / Verify addresses" flow is **display-only** for
it. This is SAFE (a shape that is never projected is never address-verified, so it can never be
mis-verified against a wrong address — verified below), but it is a missing capability: BIP-49 is
one of the four standard single-sig templates the device already derives + engraves (the restore-doc
path supports it via a direct descriptor build), and Rust (the toolkit) can express it. The fix adds
the missing `ScriptSh + PolicySingle + InnerWpkh → P2SH_P2WPKH` projection branch, analogous to the
existing `InnerWsh` (nested-multisig) discriminant, so verify lights up for `sh(wpkh)` automatically.

---

## Scope

### IN
1. **Decoder (`md/md.go`):** make a decoded single-string `sh(wpkh)` md1 wire **renderable** as a
   single-sig shape, and carry a new `InnerWpkh` discriminant on `md.Template`, set during
   `summarize`. (Today `classifyPolicy(sh(wpkh))` returns `PolicyComplex` → `Renderable=false` —
   verified empirically below — so an `sh(wpkh)` md1 routes to `expandUnsupported`/"display only".)
2. **Projection (`gui/md1_expand.go`):** add the `ScriptSh + PolicySingle + InnerWpkh → P2SH_P2WPKH,
   Singlesig` arm to `scriptForTemplate`, mirroring the existing `ScriptWpkh → P2WPKH` single-sig
   arm and the `InnerWsh` nesting discriminant pattern.
3. **TDD acceptance** (below): byte-exact BIP-49 `3…` golden; no collision with P2SH-P2WSH; existing
   projections unperturbed; fuzz harness updated to cover the new shape; grep/build clean.

### OUT
- **No new address-derivation code.** `address.addressAt` ALREADY derives P2SH-P2WPKH end-to-end
  (verified below) — this is the load-bearing "net-zero derivation" finding. The spec does NOT touch
  `address/address.go`.
- **No `bip380` change.** `bip380.P2SH_P2WPKH` already exists, is `Singlesig()`-classified, encodes,
  and derives (verified below).
- **No restore-doc / encode-side change.** The encode path (`md.EncodeSingleSig`, `ScriptShWpkh`)
  and the direct restore descriptor (`gui/singlesig_restore.go`) already handle BIP-49 and are out
  of scope (this spec only adds the *decode → display-only-md1 → verify* projection).
- **No new script shapes.** Only `sh(wpkh)` single-sig. Legacy bare `sh(pk)`, taproot script-paths,
  `multi_a`, etc. stay refused.
- **No testnet.** Mainnet-only, consistent with the rest of on-device verify (D1).
- The decode of an `sh(wpkh)` md1 via the *chunked* path is unaffected by this spec beyond the
  classifier/discriminant change (a single-key `sh(wpkh)` canonical payload chunks — see Risks).

---

## Verified facts (file:line, HEAD `8eb51d7`)

**F1 — `address` ALREADY derives P2SH-P2WPKH (net-zero derivation).**
`address/address.go:144-146` maps `bip380.P2WPKH, P2SH_P2WPKH` (singlesig) to a witness-pubkey-hash
inner script; `address/address.go:160-170` then wraps `P2SH_P2WPKH` (and `P2SH_P2WSH`) by
`txscript.PayToAddrScript(addr)` → `address.NewAddressScriptHash(script, network)` — i.e. the
P2WPKH program is hashed into a P2SH `3…` address. `bip380.P2SH_P2WPKH` is single-sig-classified at
`bip380/bip380.go:116-117` (`Singlesig()` contains `P2SH_P2WPKH`). So once a descriptor with
`Script==P2SH_P2WPKH, Type==Singlesig` reaches `address.Receive`/`Change`, it derives correctly with
**no** address-side change.

**F2 — the projection arm is deliberately ABSENT today.**
`gui/md1_expand.go:86-97`: `scriptForTemplate`'s `md.PolicySingle` switch has arms for `ScriptWpkh →
P2WPKH`, `ScriptPkh → P2PKH`, `ScriptTr → P2TR`, and an explicit NOTE (lines 96-97): "there is
deliberately NO ScriptSh+singlesig (P2SH_P2WPKH) arm — classifyPolicy never renders sh-wpkh on the
Go side (R0-Minor)." This is the exact line the new arm replaces.

**F3 — the decoder does NOT render `sh(wpkh)` today (this is the deeper half of the fix).**
`md/md.go:1285-1300`: `classifyPolicy`'s `tagSh` case handles only `sh(wsh(multi/sortedmulti))` and
bare `sh(multi/sortedmulti)`; there is **no** `sh(wpkh)` (inner `tagWpkh`) arm, so it falls through
to `PolicyComplex` (`md.go:1302`) → `Renderable=false` (`summarize`, `md.go:1336`). Confirmed by
running `classifyPolicy` on the `sh(wpkh)` tree node directly:
`classifyPolicy(node{tagSh,[node{tagWpkh,keyArgBody{0}}]}) ⇒ policy=PolicyComplex(5), k=0, m=0`.
Therefore `gatheredDescriptorFlow` (`gui/md1_gather.go:200-209`) routes a decoded `sh(wpkh)` md1 to
`expandUnsupported` → "Complex policy — display only." **This is the display-only state the recon
describes — and it lives in the decoder, not only in `scriptForTemplate`.**

**F4 — the wire shape of `sh(wpkh)`.** `md/encode_singlesig.go:90-101` (`singleSigTree`): the
canonical `sh(wpkh)` tree is `node{tagSh, childrenBody{[node{tagWpkh, keyArgBody{index:0}}]}}` — an
`sh` with a single `wpkh` child that references placeholder `@0`. `rootScriptKind(tagSh) ⇒ ScriptSh`
(`md/md.go:1246-1247`). So a decoded `sh(wpkh)` has `Root==ScriptSh` (NOT `ScriptShWpkh` — that
enum value, `md/md.go:1178`, is an **encode-input** discriminant only; the decoder summarizes to the
on-wire root tag `Sh`, per its own doc comment `md/md.go:1173-1178`).

**F5 — the `InnerWsh` discriminant precedent (the pattern to mirror).**
`md/md.go:1211-1218` (`Template.InnerWsh` field + doc), `md/md.go:1317-1331` (`innerWshNesting`: an
`sh` whose single child is `tagWsh`), `md/md.go:1355` (set in `summarize`). Consumed at
`gui/md1_expand.go:104-109`: `ScriptSh + PolicySortedMulti` picks `P2SH_P2WSH` iff `InnerWsh`, else
`P2SH` (bare legacy). Verified empirically: `innerWshNesting(sh(wpkh))=false`,
`innerWshNesting(sh(wsh(sortedmulti)))=true` — so an `sh(wpkh)` is NOT confusable with the existing
`InnerWsh` branch (different `Policy`, and `InnerWsh` is false for it anyway).

**F6 — the verify flow lights up automatically once projected.**
`gui/md1_gather.go:200-209`: `expandedToDescriptor` → `expandOK` routes to `descriptorFlow`, which
(`gui/gui.go:2405-2421`) offers "Show addresses" / "Verify an address" gated on
`address.Supported(desc)` (`gui/gui.go:2408`); "Verify" calls `verifyAddressFlow`
(`gui/verify_address.go:22`) → `address.Find(desc, candidate, 20)` (`verify_address.go:179`).
`address.Supported`/`Find`/`Receive`/`Change` all already accept a `P2SH_P2WPKH` singlesig
descriptor (F1). **No GUI/flow code changes — projecting `expandOK` is sufficient.**

**F7 — `expandedToDescriptor` is otherwise shape-agnostic.** `gui/md1_expand.go:32-78`: it gates on
`scriptForTemplate`, then expands keys (`useSiteToChildren`), and builds the `*bip380.Descriptor`
with `Threshold: tpl.K` (0 for singlesig, unused). A `PolicySingle` shape with one key flows through
unchanged once `scriptForTemplate` returns `(P2SH_P2WPKH, Singlesig, true)`. The `Network` is pinned
to `&chaincfg.MainNetParams` (`md1_expand.go:61`, D1).

**F8 — golden infrastructure already exists.** The abandon-test seed
(`gui/derive_test.go:13` `abandonAboutMnemonic`), its BIP-84 account xpub
(`gui/derive_test.go:26` `knownAccountXpub84`), and master fp `0x73c5da0a`
(`gui/derive_test.go:27`). A direct-build BIP-49 descriptor + address path is already exercised by
`gui/singlesig_restore_test.go:30-71` (`{49, md.ScriptShWpkh, bip380.P2SH_P2WPKH}`), and the
proven BIP-84 byte-exact golden pattern is `gui/singlesig_restore_test.go:75-91`.

---

## The golden (source + pinned values)

**Source:** the canonical BIP-39 test seed `abandon abandon abandon abandon abandon abandon abandon
abandon abandon abandon abandon about` (`gui/derive_test.go:13`), derived at `m/49'/0'/0'`, the
BIP-49 account path. This is the same well-known vector used across the BIP-49 ecosystem.

**Derived empirically through the real `address.addressAt` (F1) at HEAD `8eb51d7`** (via the
existing `deriveSingleSigBundle` + `singleSigRestoreDescriptor` + `address.Receive/Change`):

| Field | Value |
|---|---|
| Account xpub (`m/49'/0'/0'`) | `xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7` |
| **Receive #0** (`…/0/0`) | **`37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf`** |
| **Change #0** (`…/1/0`) | **`34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7`** |

`37Vuc…` matches the widely-published BIP-49 abandon-seed receive #0 reference value. Both addresses
begin with `3` (mainnet P2SH), confirming the P2SH-P2WPKH wrap. The implementation MUST re-derive
these and assert byte-equality (the golden is load-bearing — Invariant I1).

---

## Faithfulness spine (decoder → projection → derivation → verify)

```
md1 sh(wpkh) wire                      node{tagSh,[node{tagWpkh,keyArgBody{0}}]}   (F4)
   │  Decode → summarize
   ▼
classifyPolicy  ── NEW arm ──►         PolicySingle  (was PolicyComplex, F3)
summarize       ── NEW field ─►        InnerWpkh = innerWpkhNesting(tree) = true   (F5 pattern)
   │
   ▼
md.Template{ Root: ScriptSh, Policy: PolicySingle, InnerWpkh: true, Renderable: true, N:1 }
   │  expandedToDescriptor (F7, unchanged)
   ▼
scriptForTemplate ── NEW arm ─►        (bip380.P2SH_P2WPKH, bip380.Singlesig, true)   (F2 line replaced)
   │
   ▼
*bip380.Descriptor{ Script: P2SH_P2WPKH, Type: Singlesig, Keys:[1] }   (mainnet, D1)
   │  address.Receive/Change  (F1, UNCHANGED — already wraps P2WPKH→P2SH)
   ▼
"3…" receive/change addresses   →  Supported/Find/verifyAddressFlow lights up (F6)
```

**Discriminant design (the C2-class invariant):** the new branch is keyed on
`Policy == PolicySingle`. The existing `sh` branches are keyed on `Policy == PolicySortedMulti`
(bare `P2SH` vs nested `P2SH_P2WSH`, decided by `InnerWsh`). Because `PolicySingle ≠
PolicySortedMulti`, the new single-sig arm and the multisig nesting arm are in **disjoint switch
cases** — there is no path on which an `sh(wpkh)` could be misclassified as `sh(wsh(sortedmulti))`
or vice-versa. The new `InnerWpkh` field is meaningful only when `Root==ScriptSh && Policy==
PolicySingle`; it exists to keep the projection explicit and symmetric with `InnerWsh` (so a future
bare-`sh(pk)`/legacy single-sig shape, were it ever rendered, would NOT collide with P2SH-P2WPKH).
`P2SH_P2WPKH` (witness-pubkey-hash wrapped) and `P2SH_P2WSH` (witness-script-hash wrapped) hash
different inner programs (F1: pubkey-hash vs sha256(script)) → genuinely different `3…` addresses.

---

## Acceptance gate (TDD — tests before impl; reviewer-loop to 0C/0I after every fold)

**A1 (golden — byte-exact, the load-bearing test).** A decoded BIP-49 `sh(wpkh)` md1 (the abandon
seed at `m/49'/0'/0'`, projected via `expandedToDescriptor`) yields
`address.Receive(desc,0)=="37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf"` and
`address.Change(desc,0)=="34K56kSjgUCUSD8GTtuF7c9Zzwokbs6uZ7"`. (Drive it through the real
projection path, NOT the direct restore-descriptor build, so the new classifier + projection arm are
under test. A unit-level `md.Template{Root:ScriptSh, Policy:PolicySingle, InnerWpkh:true, …}` fed to
`expandedToDescriptor` is acceptable if a full md1 round-trip is impractical — see Risk R1.)

**A2 (no discriminant collision — C2).** For the SAME key material, the P2SH-P2WPKH `sh(wpkh)`
receive[0] MUST differ from the P2SH-P2WSH `sh(wsh(sortedmulti))` receive[0] and from the bare-P2SH
`sh(sortedmulti)` receive[0] (mirror `gui/md1_expand_test.go:142-148` `a1 != a2`). Assert all three
are pairwise distinct.

**A3 (Script + status correctness).** `scriptForTemplate(Template{Root:ScriptSh,
Policy:PolicySingle, InnerWpkh:true, Renderable:true})` returns `(P2SH_P2WPKH, Singlesig, true)`;
`expandedToDescriptor` of it with one xpub-present key returns `(non-nil, expandOK)`, and
`address.Supported(desc)==true` (verify lights up, F6).

**A4 (decode renders it).** `md.Decode`/`summarize` of an `sh(wpkh)` wire (or `classifyPolicy` on
the `sh(wpkh)` tree) now returns `Policy==PolicySingle, Renderable==true`, and the resulting
`Template.InnerWpkh==true`; `InnerWsh==false` for the same template (the two discriminants are
independent and `sh(wpkh)` is not `InnerWsh`).

**A5 (no-regression on existing projections).**
- `sh(wsh(sortedmulti))` still → `P2SH_P2WSH` (`gui/md1_expand_test.go:130,137-138` unchanged).
- bare `sh(sortedmulti)` still → `P2SH` (`gui/md1_expand_test.go:129,132-134`).
- `wpkh` still → `P2WPKH`, derives `bc1q…` (`scriptForTemplate` `ScriptWpkh` arm; assert a `bc1q`
  receive[0]).
- `pkh → P2PKH`, `tr → P2TR` unchanged.
- The existing BIP-84 abandon golden `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`
  (`gui/singlesig_restore_test.go:88`) still holds.
- Unsupported shapes (`multi`/`multi_a`/`sortedmulti_a`/`complex`, taproot script-path) still →
  `expandUnsupported`, nil descriptor (`gui/md1_expand_test.go` unsupported cases).

**A6 (fuzz clean).** Update `FuzzExpandedToDescriptor` (`gui/md1_expand_fuzz_test.go:43,48,52-55`)
and `isBip380ExpressibleShape` (`gui/md1_expand_fuzz_test.go:13-23`) so the new `ScriptSh +
PolicySingle + InnerWpkh` shape is exercised and counted as expressible (today `root` is drawn
`%5` and `isBip380ExpressibleShape`'s `PolicySingle` arm omits `ScriptSh`; without the update the
fuzzer would flag the new `expandOK` as "expandOK for non-bip380 shape"). The invariants
(`md1_expand_fuzz_test.go:26-29`: never panic; never `expandOK` for a non-expressible shape;
`expandOK ⇒ non-nil & known script; else nil`) MUST still hold. Run the existing `md/` and `gui/`
fuzz corpora to no new crash.

**A7 (build + grep clean).** `go vet ./...` and `tinygo build` (the firmware target / 0-alloc gate)
pass; `go test ./md/... ./gui/... ./address/... ./bip380/...` green. Grep shows the
`md1_expand.go:96-97` NOTE removed/updated (no stale "deliberately NO ScriptSh+singlesig arm").

---

## Invariants (numbered, load-bearing)

- **I1 — correct-P2SH-P2WPKH-derivation.** A projected BIP-49 `sh(wpkh)` descriptor derives the
  byte-exact `3…` receive[0]/change[0] of the pinned abandon-seed golden
  (`37Vuc…` / `34K56…`). Re-derived, not hard-trusted.
- **I2 — no-discriminant-collision-with-P2SH-P2WSH.** `sh(wpkh)` → `P2SH_P2WPKH` and
  `sh(wsh(sortedmulti))` → `P2SH_P2WSH` (and bare `sh(sortedmulti)` → `P2SH`) derive **pairwise
  distinct** addresses for identical key material; the projection arms are in disjoint switch cases
  (`PolicySingle` vs `PolicySortedMulti`) so no input maps to both.
- **I3 — display-only-fallback-preserved.** Any shape still not projected (everything outside the
  five renderable single-sig/sortedmulti arms) returns `expandUnsupported`/`expandTemplateOnly` with
  a **nil** descriptor and is NEVER address-verified (the safe fallback at `md1_gather.go:204-208`
  is unchanged).
- **I4 — mainnet-only.** The projected descriptor pins `&chaincfg.MainNetParams`
  (`md1_expand.go:61`); no testnet path is introduced.
- **I5 — no-regression.** Existing projections (`P2WPKH`, `P2PKH`, `P2TR`, `P2WSH`, `P2SH`,
  `P2SH_P2WSH`) and their goldens/tests are byte-for-byte unchanged; the decode of every
  currently-renderable shape is unchanged (the new `classifyPolicy` arm only adds a previously
  `PolicyComplex` shape; it must not alter any existing classification).

---

## Risks

- **R1 — md1 round-trip for the test fixture.** `md.EncodeSingleSig(ScriptShWpkh)` produces a
  **chunked** md1 (~3 strings; verified: `chunks=3`), and `md.Decode` refuses chunked
  (`ErrChunkedUnsupported`, `md.go:1229-1230`). So a *single-string* `Decode` round-trip of
  `sh(wpkh)` may not be directly available; A1/A4 can instead test (a) `classifyPolicy`/`summarize`
  on the constructed `sh(wpkh)` tree node, and/or (b) `ExpandWalletPolicyChunks` over the chunked
  strings (`md/expand.go:102`), and/or (c) a hand-built `md.Template` into `expandedToDescriptor`.
  The implementation plan must pick the path that drives the **new code** (the classifier arm + the
  projection arm), not bypass it. NOT a correctness risk to ship — a test-vector-construction
  choice. (Mitigation belongs to the plan, not this spec.)
- **R2 — fuzz harness omission.** If A6's harness update is skipped, the existing fuzz invariant
  would either never reach the new shape (false-green) or flag it as a non-expressible `expandOK`
  (false-fail). Both are caught by requiring the A6 update explicitly.
- **R3 — comment-only drift.** The `md1_expand.go:96-97` NOTE and the `md.go:1173-1178` /
  `singlesig_restore.go:25-29` comments describe the *current* "decoder never renders sh-wpkh" /
  "classifier drops single-key sh(wpkh)" behavior. These become stale once the branch lands and MUST
  be updated in the same change (A7 grep gate), or a future reader will trust a false statement.
- **R4 — accidental classification change.** The new `classifyPolicy` `tagSh`→inner-`wpkh` arm must
  be additive and ordered so it does NOT shadow the existing `sh(wsh(...))` / `sh(multi)` arms
  (I5). Distinct inner tag (`tagWpkh` vs `tagWsh`/`tagMulti`/`tagSortedMulti`) makes this clean, but
  the plan must place it explicitly and a regression test (A5) must pin every existing arm.

---

## Gate

This SPEC must pass an opus architect R0 review to **0 Critical / 0 Important** before any
implementation. Fold findings → persist the review verbatim to `design/agent-reports/` →
re-dispatch after every fold until GREEN. No code before GREEN.
