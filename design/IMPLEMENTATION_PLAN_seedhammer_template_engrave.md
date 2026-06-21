# SeedHammer Template-Engrave — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to execute this plan task-by-task (single implementer per task + two-stage review). Steps use checkbox (`- [ ]`) syntax. **Gate:** this plan must pass an opus-architect R0 (0C/0I) BEFORE any code.

**Goal:** Add opt-in on-device wallet-policy TEMPLATE engraving (keyless md1) to the SeedHammer fork — default stays full-policy; engrave + verify cover any admissible md1; on-device display is honest-minimal for shapes the device can't classify.

**Architecture:** Headless-first. (1) `md` codec gains `WalletDescriptorTemplateId` + `isWalletPolicy` + a form-aware stub selector + a conditional-elision strip; (2) the four stub-mint sites + verify route through the form-aware selector; (3) GUI opt-in on single-sig + multisig-BUILD; (4) refusals + recovery estimate. No md-codec/mk-codec constellation change (golden-lock targets only).

**Tech Stack:** Go / TinyGo (RP2350). Host tests `go test`; device gate `tinygo build ./cmd/controller`. Go via `export PATH=$PATH:/home/bcg/.local/go/bin`.

**Source of truth:** SPEC `design/SPEC_seedhammer_template_engrave.md` (R0 GREEN, `8fc938e`). Fork `/scratch/code/shibboleth/seedhammer` @ `main` `39cb5cf`. Golden-lock targets: `descriptor-mnemonic@54dd765` (`md-codec/src/identity.rs:71-104` WDT-Id, `encode.rs:50-52` is_wallet_policy, `tree.rs` write_node), `mnemonic-key@1279ef9` (`mk-cli/src/cmd/mod.rs:72-82` derive_stub_from_md1), `mnemonic-toolkit@6de53879` (`synthesize.rs:1158-1283` synthesize_template_descriptor, `:1185-1198` conditional origin elision).

**Commits (fork convention):** SSH-signed (`-S`) + DCO (`-s`), author Brian Goss; trailer `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Stage paths explicitly (no `git add -A`).

---

## File structure

| File | Responsibility | New/Modify |
|---|---|---|
| `md/template_id.go` | `WalletDescriptorTemplateId`, `WalletDescriptorTemplateIdStub`, `isWalletPolicy`, form-aware `FormAwareStub`/`FormAwareStubChunks` | **Create** |
| `md/template_strip.go` | `StripToTemplate` (decode→null pubkeys/fp→conditional origin-elide→`encodePayload`) | **Create** |
| `md/walletpolicyid.go` | reference pattern for the WDT-Id preimage (mirror `:30-64` MINUS `canonicalize` at `:32`) | Read-only ref |
| `bundle/verify.go` | `checkStubBinding` (`:116`) → form-aware | Modify |
| `gui/singlesig_derive.go` | mint site (`:67`) → form-aware | Modify |
| `gui/multisig_derive.go` | mint site (`:42`) → form-aware | Modify |
| `md/encode_multisig.go` | mint site (`:158`, `WalletPolicyIDStub(d)`) → form-aware | Modify |
| `gui/singlesig.go` (+ flow) | single-sig template opt-in ChoiceScreen + warning + estimate + complex/depth consent | Modify |
| `gui/multisig.go` (+ build flow) | multisig-BUILD template opt-in; supply path stays full-policy/verbatim | Modify |
| `md/testdata/template/*` | golden vectors from `toolkit bundle --md1-form=template` + the §5 fixture | **Create** |

---

## Task 0 — Worktree + clean baseline

**Files:** none (setup).

- [ ] **Step 1:** Create an isolated worktree off fork `main` (`39cb5cf`):
```bash
cd /scratch/code/shibboleth/seedhammer
git worktree add /tmp/seedhammer-wt-template feat/template-engrave main
cd /tmp/seedhammer-wt-template
export PATH=$PATH:/home/bcg/.local/go/bin
```
- [ ] **Step 2:** Baseline build + tests green:
```bash
go build ./... && go test ./md/... ./bundle/... ./gui/...
```
Expected: PASS (record the count). If red, STOP and report.

---

## Task 1 — Headless: `isWalletPolicy` predicate (S2 / I1)

**Files:** Create `md/template_id.go`; Test `md/template_id_test.go`.

- [ ] **Step 1: Failing test.** A keyless template descriptor → false; a keyed policy → true; a descriptor with `pubPresent==true` but **empty** `pubkeys` → **false** (I1 — Rust `encode.rs:50-52` is Some-AND-non-empty).
```go
func TestIsWalletPolicy(t *testing.T) {
    full := mustDecodeDesc(t, fullWpkhMD1)      // keyed
    tmpl := mustDecodeDesc(t, keylessWpkhMD1)   // pubkeys:null
    if !isWalletPolicy(full) { t.Fatal("keyed must be wallet-policy") }
    if isWalletPolicy(tmpl) { t.Fatal("keyless must NOT be wallet-policy") }
    desync := *full; desync.tlv.pubkeys = nil   // pubPresent stays true, pubkeys empty
    if isWalletPolicy(&desync) { t.Fatal("empty pubkeys must NOT be wallet-policy (I1)") }
}
```
- [ ] **Step 2:** Run → FAIL (undefined `isWalletPolicy`).
- [ ] **Step 3: Implement** (package `md`, so it sees the unexported `descriptor`/`tlv`):
```go
// isWalletPolicy mirrors Rust md-codec encode.rs:50-52: pubkeys present AND non-empty.
func isWalletPolicy(d *descriptor) bool {
    return d.tlv.pubPresent && len(d.tlv.pubkeys) > 0
}
```
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Commit** `md/template_id.go md/template_id_test.go` — "feat(md): isWalletPolicy predicate (Some-AND-non-empty, I1)".

---

## Task 2 — Headless: `WalletDescriptorTemplateId` + stub (S2 — security-load-bearing)

**Files:** Modify `md/template_id.go`; Test `md/template_id_test.go`.

**Reference:** mirror `md/walletpolicyid.go:30-64` STRUCTURE but: (a) **NO `canonicalize(d)`** (R0 pin #1 — Rust `compute_wallet_descriptor_template_id` does not canonicalize; rely on the decode-side canonical invariant `validatePlaceholderUsage`); (b) preimage = `use_site_path ‖ writeNode(tree) ‖ UseSitePathOverrides-TLV` with **NO keys/fingerprints** (this is what makes it key-independent — Rust `identity.rs:71-104`); (c) kiw from `d.n`; (d) carry the `d.pathDecl.n == d.n` guard (`errPathDeclNMismatch`) **inside** this function (it bypasses `encodePayload` where that guard normally lives, `encode.go:401`).

- [ ] **Step 1: Failing golden test.** WDT-Id of a keyless `wsh(sortedmulti(2,@0,@1,@2))` template → `b02b4403…` (full 16 bytes from `md-cli`/`mk-cli`); origin-invariant (default vs `--path bip84` vs `m/48'/0'/0'/2'` → identical); distinct per (script,k,N,use-site). Generate the expected bytes:
```bash
# in descriptor-mnemonic @ 54dd765:
cargo run -q -p md-cli -- identity --template 'wsh(sortedmulti(2,@0,@1,@2))' --path m/48h/0h/0h/2h   # → b02b4403...
```
```go
func TestWalletDescriptorTemplateId_Golden(t *testing.T) {
    d := mustDecodeDesc(t, keylessWshSortedmulti2of3)
    got, err := WalletDescriptorTemplateId(d)
    if err != nil { t.Fatal(err) }
    want := mustHex16(t, "b02b4403...")   // full 16B from md-cli
    if got != want { t.Fatalf("WDT-Id = %x, want %x", got, want) }
}
func TestWalletDescriptorTemplateId_OriginInvariant(t *testing.T) { /* id under 3 origins identical */ }
func TestWalletDescriptorTemplateId_Distinct(t *testing.T) { /* wsh-multi != wsh-sortedmulti; k=1 != k=2; N=2 != N=3; wpkh != pkh */ }
```
- [ ] **Step 2:** Run → FAIL (undefined).
- [ ] **Step 3: Implement** (mirror `WalletPolicyId` writeNode-preimage MINUS canonicalize; consult `identity.rs:71-104` for the exact preimage byte order + the override-TLV inclusion rule):
```go
// WalletDescriptorTemplateId = SHA-256( useSitePath ‖ writeNode(tree) ‖ UseSitePathOverrides-TLV )[0:16].
// Key-independent + origin-invariant (no keys/fp/origin in the preimage). Mirrors Rust identity.rs:71-104.
// NOTE: does NOT canonicalize (R0 pin #1) — relies on the decode-side canonical invariant.
func WalletDescriptorTemplateId(d *descriptor) ([16]byte, error) {
    if d.pathDecl.n != d.n { return [16]byte{}, errPathDeclNMismatch } // R0 pin #2, guard INSIDE
    width := kiw(d.n)
    var w bitWriter
    if err := writeUseSitePath(&w, d.useSite); err != nil { return [16]byte{}, err }
    if err := writeNode(&w, d.tree, width); err != nil { return [16]byte{}, err }
    // append the UseSitePathOverrides TLV iff present, matching identity.rs preimage rule
    // ... (mirror walletpolicyid.go override handling, byte-for-byte vs identity.rs)
    sum := sha256.Sum256(w.intoBytes())
    var id [16]byte; copy(id[:], sum[:16]); return id, nil
}
func WalletDescriptorTemplateIdStub(d *descriptor) ([4]byte, error) {
    id, err := WalletDescriptorTemplateId(d); if err != nil { return [4]byte{}, err }
    var s [4]byte; copy(s[:], id[:4]); return s, nil
}
```
- [ ] **Step 4:** Run → PASS (all three tests). Cross-check the override-TLV branch against `identity.rs` if the golden mismatches.
- [ ] **Step 5: Commit** — "feat(md): WalletDescriptorTemplateId Go port (no-canonicalize, kiw-from-n, guard-inside; golden b02b4403)".

---

## Task 3 — Headless: form-aware stub selector (S2 / S3)

**Files:** Modify `md/template_id.go`; Test `md/template_id_test.go`. The existing mint sites call two shapes — `WalletPolicyIDStubChunks(strs []string)` and `WalletPolicyIDStub(d *descriptor)` — so provide both form-aware variants.

- [ ] **Step 1: Failing test** (byte-exact vs `mk-cli derive_stub_from_md1`): keyless template md1 → stub from WDT-Id; keyed policy md1 → stub from WalletPolicyId.
```go
func TestFormAwareStub(t *testing.T) {
    // keyless wsh-sortedmulti → top4(WDT-Id) == 0xb02b4403
    got, _ := FormAwareStubChunks([]string{keylessWshSortedmultiMD1})
    if got != [4]byte{0xb0,0x2b,0x44,0x03} { t.Fatalf("template stub = %x", got) }
    // keyed → top4(WalletPolicyId) (unchanged from today)
    g2, _ := FormAwareStubChunks([]string{fullWpkhMD1})
    w2, _ := WalletPolicyIDStubChunks([]string{fullWpkhMD1})
    if g2 != w2 { t.Fatal("keyed must select WalletPolicyId") }
}
```
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.** Decode once, branch on `isWalletPolicy`:
```go
func FormAwareStub(d *descriptor) ([4]byte, error) {
    if isWalletPolicy(d) { return WalletPolicyIDStub(d) }
    return WalletDescriptorTemplateIdStub(d)
}
func FormAwareStubChunks(strs []string) ([4]byte, error) {
    d, err := decodeChunksToDescriptor(strs) // reuse the same decode WalletPolicyIDStubChunks uses
    if err != nil { return [4]byte{}, err }
    return FormAwareStub(d)
}
```
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Commit** — "feat(md): form-aware stub selector (is_wallet_policy ? WalletPolicyId : WDT-Id)".

---

## Task 4 — Headless: conditional-elision strip transform (S1 / C1)

**Files:** Create `md/template_strip.go`; Test `md/template_strip_test.go`; golden vectors `md/testdata/template/*`.

**Mutations (match `synthesize.rs:1158-1283`):** `tlv.pubkeys = nil`; `tlv.fingerprints = nil`; **origin: elide ONLY when `canonicalOrigin(d.tree)` returns ok (C1 — `md.go:1097`); KEEP source origins otherwise** (eliding a no-canonical-origin policy → decode-rejected `MissingExplicitOrigin`). Re-emit via `encodePayload` / `encodeMD1String` (shape-general).

- [ ] **Step 1: Generate golden vectors** from the toolkit @ `6de53879`:
```bash
# single-sig, wsh-sortedmulti (canonical origin → elided), and the §5 general wallet (no canonical origin → kept)
toolkit bundle --md1-form=template ... > md/testdata/template/wpkh.tmpl.md1
toolkit bundle --md1-form=template ... > md/testdata/template/wsh_sortedmulti.tmpl.md1
toolkit bundle --md1-form=template ... > md/testdata/template/example5_11key.tmpl.md1   # M3: inline the §5 fixture
```
- [ ] **Step 2: Failing test.** Strip(full md1) byte-identical to the toolkit template golden, for each fixture — INCLUDING the `canonical_origin==None` general-policy vector which must KEEP its source origins (decode the strip output → asserts origins present, no `errMissingExplicitOrigin`).
- [ ] **Step 3:** Run → FAIL (undefined `StripToTemplate`).
- [ ] **Step 4: Implement:**
```go
// StripToTemplate decodes a full md1, nulls pubkeys+fingerprints, conditionally elides
// origin (only when canonicalOrigin(tree) is present — C1), and re-emits the keyless md1.
func StripToTemplate(md1Chunks []string) ([]string, error) {
    d, err := decodeChunksToDescriptor(md1Chunks); if err != nil { return nil, err }
    d.tlv.pubkeys = nil; d.tlv.pubPresent = false
    d.tlv.fingerprints = nil
    if _, ok := canonicalOrigin(d.tree); ok {
        // elide: set the shared/use-site origin to the empty/canonical form (mirror synthesize.rs:1185-1198)
    } // else: KEEP source origins (general policy, e.g. §5)
    return splitToChunks(d) // encodePayload-backed; same chunker the encoders use
}
```
- [ ] **Step 5:** Run → PASS (all fixtures byte-identical; §5 keeps origins).
- [ ] **Step 6: Commit** — "feat(md): StripToTemplate conditional-elision strip (golden-locked, incl. §5 keep-origins)".

---

## Task 5 — Form-aware binding at ALL FOUR mint sites + verify (S3 / C2 / I5)

**Files:** Modify `bundle/verify.go:116`, `gui/singlesig_derive.go:67`, `gui/multisig_derive.go:42`, `md/encode_multisig.go:158`; Tests in `bundle/verify_test.go` + the derive tests.

- [ ] **Step 1: Failing tests.**
  - **Security (template):** an engraved keyless-template bundle's mk1 (rooting on WDT-Id) VERIFIES; a foreign/wrong mk1 FAILS. (Fails today: `verify.go:116` uses `WalletPolicyIDStubChunks` → template mis-binds.)
  - **Device own-readback (C2):** a BUILT template's derive-minted stub matches its verify stub (fails today: derive sites mint `WalletPolicyId`-of-keyless).
  - **Regression (I5):** a full-policy bundle verifies AND the minted stubs at all 4 sites are byte-identical to today (selector picks `WalletPolicyId`).
- [ ] **Step 2:** Run → the template + own-readback tests FAIL; regression PASS.
- [ ] **Step 3: Implement.** Replace at each site:
  - `bundle/verify.go:116`: `md.WalletPolicyIDStubChunks(b.MD1)` → `md.FormAwareStubChunks(b.MD1)`.
  - `gui/singlesig_derive.go:67`: `md.WalletPolicyIDStubChunks(md1)` → `md.FormAwareStubChunks(md1)`.
  - `gui/multisig_derive.go:42`: `md.WalletPolicyIDStubChunks(suppliedMd1)` → `md.FormAwareStubChunks(suppliedMd1)`.
  - `md/encode_multisig.go:158`: `WalletPolicyIDStub(d)` → `FormAwareStub(d)`.
- [ ] **Step 4:** Run → all PASS.
- [ ] **Step 5: Commit** — "fix(md,gui,bundle): form-aware stub at all 4 mint sites + verify (C2)".

---

## Task 6 — GUI: single-sig template opt-in + warning + estimate + complex/depth consent (S4 / S5 / S6)

**Files:** Modify `gui/singlesig.go` (+ its engrave flow); Test the host GUI harness (mirror existing `gui/*_test.go` patterns).

- [ ] **Step 1: Failing test.** Default lands on full-policy (byte-identical engrave to today). Selecting "Template-only" → shows the warning + estimate strings, then engraves `StripToTemplate(builtMD1)` + the form-aware single mk1 stub. For a `classifyPolicy`→`PolicyComplex` shape (taproot depth-2 single-key path is N/A here; use the complex fixture if reachable single-sig) the confirm screen shows `{family, slot-count N, template-id}` (assert strings). Depth-≥2 shows the EXPERIMENTAL warning naming ">13.1.0 / PR #953".
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.** Inner `ChoiceScreen` on `engraveSingleSig` (no new `program` → no `gui/gui.go:164` trip). Full = today's path verbatim. Template = strip + warning (S4 mockup) + estimate (S6: sortedmulti→none / ordered→N! @6.9/7.4µs; the harmonized N→time table) + the complex/depth consent (S5). Surface `Template.N` for the slot-count line.
- [ ] **Step 4:** Run → PASS. Confirm the full-policy default path is byte-identical (golden pin).
- [ ] **Step 5: Commit** — "feat(gui): single-sig template opt-in + warning/estimate/consent".

---

## Task 7 — GUI: multisig-BUILD template opt-in; supply stays verbatim (S4 / N1)

**Files:** Modify `gui/multisig.go` (+ the on-device BUILD flow). **Do NOT touch** `supplyMultisigPolicyFlow` / `allSlotsHaveXpub` (`gui/multisig_supply.go:72`) — it stays full-policy-only (D1).

- [ ] **Step 1: Failing test.** On the multisig on-device **BUILD** path, selecting Template-only → engraves `StripToTemplate(builtMD1)` + N keyless cosigner mk1 stubs, each rooting on the one WDT-Id (form-aware, C2); the bundle passes the device's own readback verify. The **supply** path is unchanged (full-policy; a keyless template supplied there is NOT accepted by the seed-cross-match flow — it has no xpub to match). A SUPPLIED template bundle (md1 + N keyless mk1) engraved via `bundleFlow`/`bundleEngrave` (`gui/bundle_flow.go:24,327`) verbatim verifies via the form-aware binding (N1: engrave-verbatim, bind-at-verify split).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.** Add the opt-in ChoiceScreen on the multisig BUILD path only; reuse `StripToTemplate` + the form-aware mint (Task 5). Leave the supply flow + `allSlotsHaveXpub` untouched.
- [ ] **Step 4:** Run → PASS; supply-flow regression byte-identical.
- [ ] **Step 5: Commit** — "feat(gui): multisig-BUILD template opt-in (supply path unchanged, D1)".

---

## Task 8 — Refusals at the correct layer (S5)

**Files:** Modify the template-parser / derive-address paths as needed; Tests.

- [ ] **Step 1: Failing test.** `tr(sortedmulti_a)` + `sortedmulti`-in-combinator → refused at the **template parser** with a clear message; **hardened use-site** → refused at the **derive/address** path (`HardenedPublicDerivation`-equivalent), NOT at the template parser (a `/*'`/`/N'/` template still encodes/strips fine).
- [ ] **Step 2:** Run → FAIL (or confirm existing refusals already cover; if so, this task is assertion-only).
- [ ] **Step 3: Implement / assert** the refusals at the right layer with clear messages.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Commit** — "feat: template refusals at correct layer (tr(sortedmulti_a)/combinator/hardened-use-site)".

---

## Task 9 — TinyGo device build + full regression gate

- [ ] **Step 1:** Full host regression: `go test ./md/... ./bundle/... ./gui/...` → all PASS.
- [ ] **Step 2:** TinyGo device build (the final integration gate):
```bash
nix develop --command tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```
Expected: success. (Watch for any new generic / stdlib that breaks TinyGo.)
- [ ] **Step 3: Commit** any build-fix; otherwise note green.

---

## Post-implementation (MANDATORY, non-deferrable)
A single independent opus-architect **whole-diff adversarial execution review** (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). Persist verbatim to `design/agent-reports/`. Converge to 0C/0I before merge. Then `git merge --no-ff -S --signoff` to fork `main`, push, confirm CI (Test + `tinygo-device-build` + Build-image) green.

## Plan-time notes folded (from R0 round 2)
- **N1:** the N-cosigner verbatim home is `bundleFlow`/`bundleEngrave` (`gui/bundle_flow.go:24,327`), NOT `mdmkFlow` (single-card); engrave-verbatim vs form-aware-verify split is explicit in Task 5 + Task 7.
- **M1:** estimate provenance — 6.9 µs/perm (policyID), 7.4 µs/perm (first address), 24-core i7-13700 @ 5.3 GHz, Rust toolkit `permutation_search.rs` (full N! enumeration, no pruning).
- **M3:** the §5 11-key general-miniscript wallet is vendored as a concrete golden fixture in Task 4.

## Open for plan-R0 to rule on
- Is the WDT-Id preimage's UseSitePathOverrides-TLV inclusion rule reproduced exactly vs `identity.rs:71-104` (the one byte-order subtlety not fully shown above)?
- Is `decodeChunksToDescriptor` the right existing entry the chunks-based stub helpers already use (confirm the exact decode path so `FormAwareStubChunks` matches `WalletPolicyIDStubChunks`'s decode)?
- Task 6/7 GUI test reachability of a `PolicyComplex` single-sig vs multisig fixture (where to assert the honest-minimal consent screen).
- Does Task 8 find the refusals already present (assertion-only) or needing new code?
