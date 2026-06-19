# #10b (Phase B) Implementation Plan — md1 multi-chunk gather + wallet-policy xpub-expansion → descriptor display + verify

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. `- [ ]` checkboxes; strict TDD (fail → run-fail → impl → run-pass → commit).

**Goal:** On-device, gather a multi-chunk md1 descriptor, reassemble+integrity-check it (via #10a's `md.Reassemble`), expand its wallet-policy xpubs, project the bip380-expressible subset onto a `*bip380.Descriptor`, and route it into the existing descriptor display + receive-address verification — replacing the current "Multi-part descriptor — not yet supported" refusal.

**Architecture:** Builds strictly on #10a (the md codec, shipped `3a55ae5`). New exported `md.DecodeChunks` + a per-@N wallet-policy expansion accessor; a real secp256k1 on-curve check in `md.validateXpubBytes`; a `gui` projection from expanded keys → `*bip380.Descriptor`; an `md1Gatherer`/`md1GatherFlow` clone of the mk1 gather UX; and a one-line route change in `mdmkFlow`. Reuses `verifyAddressFlow`/`DescriptorScreen`/`address.Find` unchanged.

**Tech stack:** Go (host tests via `/home/bcg/.local/go/bin/go`; TinyGo-safe production paths). Module `seedhammer.com`.

**Spec:** `design/SPEC_seedhammer_10_md_encoder.md` (Phase B / §2 IN-B; GREEN @ R0 `a8d697f`). **Recon (touchpoints + citations):** `design/agent-reports/seedhammer-10b-recon-phaseB-gui.md`. **Blueprint §3-4:** `design/agent-reports/seedhammer-10-md-encoder-architect-blueprint.md`.
**Rust reference** (`descriptor-mnemonic` @ `c85cd49`): `src/{canonicalize,derive,to_miniscript,validate}.rs`. **Go to extend/invert:** `md/md.go`, `md/chunk.go`; **clone source:** `gui/mk1_inspect.go`; **reuse:** `gui/verify_address.go`, `gui/gui.go` (`mdmkFlow`/`descriptorFlow`/`DescriptorScreen`), `bip380/bip380.go`, `address/address.go`.

## Locked decisions (R0 may challenge)
- **D1 — Network: MAINNET-ONLY.** md1 carries no network on the wire; Rust hardcodes `Main` (`derive.rs:57`). Faithful to the constellation + the user's "mirror m\* behavior" directive + simplest. `bip380.Key.Network = &chaincfg.MainNetParams`. A testnet picker is a trivial follow-up if later wanted (logged to FOLLOWUPS if R0 agrees).
- **D2 — Faithful-or-refuse projection (I-6).** Project ONLY the bip380-expressible subset: singlesig (wpkh/pkh/tr-keyonly/sh-wpkh) + `wsh(sortedmulti)` + `sh(wsh(sortedmulti))`. **Unsorted `multi`/`multi_a`/tr-with-taptree → display the template read-only (existing `md1DisplayFlow`), NEVER build a descriptor / address-verify** (else verify against a wrong sorted-key address).
- **D3 — Template-only fallback.** An md1 with no Pubkeys TLV (`tlv.pubPresent==false`) has no xpubs to expand → template-only display, no descriptor build / no verify.
- **D4 — secp256k1 check lives in `md.validateXpubBytes`** (matches Rust placement `validate.rs:215-227`; `md` gains a `secp256k1` dep). Chain-code prefix intentionally unvalidated.
- **D5 — Hardened wildcard / hardened multipath alt → explicit early reject** (Rust `HardenedPublicDerivation`), with a clear on-screen message (don't let `derivePubKey` fail late).
- **D6 — Reassemble→validate→build-descriptor happens in the gather-completion handler, BEFORE entering `DescriptorScreen`** (the alloc-gated screen; never per-frame).

---

## Task 0: Worktree + baseline
- [ ] **Step 1:** From `/scratch/code/shibboleth/seedhammer`, `git worktree add ../seedhammer-wt-10b -b feat/10b-md-walletpolicy 3a55ae5` (sibling-dir convention). Work there. (Sandbox-fallback: `git checkout -b` in place, and say so.)
- [ ] **Step 2:** Baseline `/home/bcg/.local/go/bin/go test ./md/... ./codex32/... ./bip380/... ./address/... ./gui/...` → all pass. Else BLOCKED.

---

## Task 1: Real secp256k1 on-curve check in `md.validateXpubBytes` (D4)

**Files:** Modify `md/md.go`; Test `md/md_test.go` (or `md/validate_test.go`).

- [ ] **Step 1: Failing test.** A decoded md1 whose Pubkeys TLV carries a 33-byte pubkey that is NOT a valid secp256k1 point → `md.Decode`/validation returns `errInvalidXpubBytes`; a valid point → no error. Build the bad-point case by white-box-constructing the payload (reuse the `testBitWriter` / `testdata_test.go` shim), or by mutating a valid golden's pubkey byte.
- [ ] **Step 2: Run → FAIL** (the no-op currently returns nil).
- [ ] **Step 3: Implement** `validateXpubBytes` (`md/md.go:1071-1077`): for each `idxPub`, `if _, err := secp256k1.ParsePubKey(p.xpub[32:65]); err != nil { return errInvalidXpubBytes }` (import `github.com/decred/dcrd/dcrec/secp256k1/v4` — already an in-tree dep). Leave `xpub[0:32]` (chain code) unvalidated (Rust `validate.rs:203-206`).
- [ ] **Step 4: Run → PASS.** Confirm all existing golden decodes still pass (their pubkeys are valid points).
- [ ] **Step 5: Commit** — `md: real secp256k1 on-curve check in validateXpubBytes (#10b, D4)`.

---

## Task 2: Exported `md.DecodeChunks` + wallet-policy expansion accessor (NET-NEW)

**Files:** Modify `md/chunk.go` + `md/md.go` (or new `md/expand.go`); Test `md/expand_test.go`.

Expose the chunked path + the per-@N data the GUI needs (the Go analog of Rust `expand_per_at_n`, `canonicalize.rs:420-474`).

- [ ] **Step 1: Failing tests.** (a) `md.DecodeChunks(split(d)) ` returns a `Template` equal to `md.Decode` of the equivalent single-string (for `wsh_multi_chunked` + the hand-built `chunked_md1_vector`); a tampered set → the `Reassemble` error (csid-mismatch/incomplete) surfaces. (b) `md.ExpandWalletPolicy(tpl-or-strs)` returns one `ExpandedKey{Index uint8; OriginPath bip32.Path; OriginHardened []bool; UseSite UseSite; Fingerprint [4]byte; FingerprintPresent bool; Xpub [65]byte; XpubPresent bool}` per `@N` in `0..n`, with origin precedence override>divergent[idx]>shared and use-site override>baseline (mirror `canonicalize.rs:437-460`); for a no-pubkeys md1, `XpubPresent==false` for all.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** in `md`: `DecodeChunks(strs []string) (Template, error)` = `Reassemble(strs)` → `summarize` (the same path `Decode` uses post-decode) — exported because `Reassemble` returns the unexported `*descriptor`. Add an exported `ExpandedKey` struct + `ExpandWalletPolicy(...)` that walks `0..d.n`, resolving structured origin (from `d.tlv.originOverrides` / `d.pathDecl`) + use-site (from `d.tlv.useSiteOverrides` / `d.useSite`) + `fingerprints[idx]` + `pubkeys[idx]`. Provide both a `[]string`-input form (decode+reassemble internally) and a form taking the already-decoded result, so the gather flow doesn't decode twice. Surface `OriginPath` as a structured `bip32.Path` (+ a hardened-flags slice if `bip32.Path` doesn't encode hardening) so Task 3 can build `bip380.Key.DerivationPath` and serialize depth/childnum correctly.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `md: exported DecodeChunks + ExpandWalletPolicy per-@N accessor (#10b)`.

---

## Task 3: Project expanded keys → `*bip380.Descriptor` (NET-NEW, D1/D2/D3/D5)

**Files:** Create `gui/md1_expand.go`; Test `gui/md1_expand_test.go`.

`expandedToDescriptor(tpl md.Template, keys []md.ExpandedKey) (*bip380.Descriptor, expandStatus)` where `expandStatus ∈ {expandOK, expandTemplateOnly, expandUnsupported}`.

- [ ] **Step 1: Failing tests** (table-driven over the vendored wallet-policy goldens). (a) `wsh_sortedmulti`/`wsh_multi_2of3` → `expandOK` + a `*bip380.Descriptor{Script:P2WSH, Type:SortedMulti, Threshold:k, Keys:[…]}` whose `address.Find` matches a known receive address (derive it via `address.Receive(desc,0)` and assert round-trip). (b) a singlesig golden (`wpkh_basic`/`pkh_basic`/`tr_keyonly`) → `expandOK` + `Type:Singlesig`, correct `Script`. (c) an **unsorted-multi** template → `expandUnsupported` (no descriptor). (d) a **no-pubkeys** template (`XpubPresent==false`) → `expandTemplateOnly`. (e) a hardened-wildcard use-site → `expandUnsupported` (D5).
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `gui/md1_expand.go`:
  - Map `tpl.Root`(`ScriptKind`)+`tpl.Policy`(`PolicyKind`) → `bip380.Script` + `MultisigType`: `ScriptWpkh→P2WPKH/Singlesig`, `ScriptPkh→P2PKH/Singlesig`, `ScriptTr`+`PolicySingle→P2TR/Singlesig`, `ScriptSh`+singlesig→`P2SH_P2WPKH`, `ScriptWsh`+`PolicySortedMulti→P2WSH/SortedMulti`, `ScriptSh`+wsh-sortedmulti→`P2SH_P2WSH/SortedMulti`. **`PolicyMulti`(unsorted)/`PolicyMultiA`/`PolicySortedMultiA`/tr-with-tree → `expandUnsupported`** (D2). `!tpl.Renderable` → `expandUnsupported`.
  - If any `!XpubPresent` → `expandTemplateOnly` (D3).
  - Per expanded key build `bip380.Key{Network:&chaincfg.MainNetParams (D1), MasterFingerprint:fpFrom(Fingerprint), DerivationPath:OriginPath, Children:useSiteToChildren(UseSite), KeyData:Xpub[32:65], ChainCode:Xpub[0:32], ParentFingerprint:0}`. `useSiteToChildren`: `<a;b>/*` → `[RangeDerivation{Index:a,End:b}, WildcardDerivation]`; bare `*` → `[WildcardDerivation]` (mirror `to_miniscript.rs:116-131`; note `address.derivePubKey` requires `End==Index+1`). **Hardened wildcard or hardened multipath alt → `expandUnsupported`** (D5).
  - `Threshold` = `tpl.K` for multisig; `Keys` ordered by `@N`.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: project expanded md1 wallet-policy → *bip380.Descriptor (#10b, D1-D5)`.

---

## Task 4: `md1Gatherer` + `md1GatherFlow` (clone of mk1, D6)

**Files:** Modify `gui/md1_inspect.go` (or new `gui/md1_gather.go`); Test `gui/md1_gather_test.go`.

- [ ] **Step 1: Failing tests** (drive via `runUI`+`click`/`runes`; `NFCReader()==nil` so no goroutine — exercise the gatherer + completion handler directly). `md1Gatherer.offer` primes/foreign/dup/added exactly like mk1 (use `md.ParseChunkHeader`); `complete()` at `len==total`; on completion the flow calls `md.DecodeChunks(collected)` and, per `expandedToDescriptor`, routes to descriptor display (expandOK) / template-only (expandTemplateOnly) / a clear "complex policy — display only" (expandUnsupported); a csid-mismatch set → a DISTINCT error message (not the generic decode failure). Unit-test `md1Gatherer` directly like `mk1Gatherer`.
- [ ] **Step 2: Run → FAIL.**
- [ ] **Step 3: Implement** `md1Gatherer` (clone `mk1Gatherer` `gui/mk1_inspect.go:48-83`, swap `mk.ParseHeader`→`md.ParseChunkHeader`, keep `!h.Chunked`→foreign guard) + `md1GatherFlow(ctx, th, first string) bool` (clone `mk1GatherFlow:156-256`: same scanner-goroutine shell, "Captured N of M / Scan the next chunk." copy adapted to "descriptor", title "Inspect descriptor"). Completion handler (D6 — all before any `DescriptorScreen`): `tpl, err := md.DecodeChunks(g.collected())`; on `errChunkSetIDMismatch`→`showError(...,"Chunks don't match — mixed or tampered set.")`; on other err→`"Can't decode this descriptor set."`; on ok → `keys := md.ExpandWalletPolicy(...)`; `desc, status := expandedToDescriptor(tpl, keys)`; switch status → `descriptorFlow(ctx,th,desc)` / `md1DisplayFlow(ctx,th,tpl)` / `showError(...,"Complex policy — display only.")` then `md1DisplayFlow`.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: md1Gatherer + md1GatherFlow (multi-chunk descriptor gather) (#10b)`.

---

## Task 5: Route `mdmkFlow` chunked-md1 → gather (the wiring)

**Files:** Modify `gui/gui.go` (`mdmkFlow` `:1984-1985`); Test `gui/gui_test.go`.

- [ ] **Step 1: Failing test.** Scanning/inspecting a chunked md1 (first chunk) no longer shows "not yet supported" — it enters the gather flow; a complete `wsh_sortedmulti` chunk set reaches the descriptor display + address-verify; the single-md1 path is unchanged.
- [ ] **Step 2: Run → FAIL** (current arm shows the refusal).
- [ ] **Step 3: Implement** — replace the `errors.Is(err, md.ErrChunkedUnsupported)` arm (`gui/gui.go:1984-1985`) `showError(...)` with `md1GatherFlow(ctx, th, str)`. Leave the `err==nil`→`md1DisplayFlow` and default arms unchanged.
- [ ] **Step 4: Run → PASS.**
- [ ] **Step 5: Commit** — `gui: route chunked md1 from refusal into md1GatherFlow (#10b)`.

---

## Task 6: No-regression + fuzz

**Files:** Test only.

- [ ] **Step 1:** `/home/bcg/.local/go/bin/go test -count=1 ./...` + `TestAllocs` green. **Confirm `DescriptorScreen` alloc gate intact** — the expanded `*bip380.Descriptor` is built in the gather-completion handler (Task 4), never per-frame (D6). `go vet ./md/... ./gui/...` clean (vs baseline); `gofmt -l` empty. Single-md1 decode/display, mk1, ms1 unchanged.
- [ ] **Step 2: Fuzz** `FuzzExpandWalletPolicy` (arbitrary decoded descriptors → expand → no panic) and `FuzzExpandedToDescriptor` (no panic; never returns `expandOK` for a non-bip380 shape). ≥1M execs each.
- [ ] **Step 3: Run → 0 panics.**
- [ ] **Step 4: Commit** — `md/gui: no-regression + fuzz for md1 wallet-policy expansion (#10b)`.

---

## Acceptance (GREEN bar for the exec review)
- A complete chunked `wsh(sortedmulti)` md1 set gathers → reassembles (integrity-checked) → expands → builds a `*bip380.Descriptor` → the existing display + `address.Find` verify a known receive address (Task 3.a, Task 5).
- Singlesig md1 → singlesig descriptor + verify (Task 3.b). Unsorted `multi`/`multi_a`/taptree → display-only, NO descriptor/verify (Task 3.c, D2). No-pubkeys md1 → template-only (Task 3.d, D3). Hardened wildcard → rejected (Task 3.e, D5).
- secp256k1 on-curve check rejects an off-curve pubkey (Task 1, D4).
- csid-mismatch set → distinct error UX (Task 4).
- Full suite + `TestAllocs` green; `DescriptorScreen` alloc gate intact (D6); fuzz 0 panics; single-md1/mk1/ms1 unchanged.

## Self-review (author, pre-R0)
- Spec coverage: B1 (gather)→T4; B2 (xpub-expansion + secp256k1)→T1+T2+T3; B3 (wire to verify/display + route)→T3+T5. I-6 (faithful-or-refuse subset)→T3/D2. ✓
- Decisions D1-D6 all locked + traced to source. Network=mainnet flagged for R0. ✓
- No placeholders; types consistent (`md.ExpandedKey`, `expandStatus`, `*bip380.Descriptor`); each code step cites the Rust to mirror + the Go to clone/extend. ✓
- Risk: the `bip32.Path` hardening representation (T2 Step 3) — confirm whether `bip32.Path` encodes hardened bits or needs a parallel `[]bool`; the implementer must check `bip32` before T2 and adapt. Flagged for R0.

## Gate
This plan MUST pass opus R0 to 0C/0I before code; fold → persist verbatim → re-dispatch after every fold until GREEN. Then single-implementer TDD in the worktree → mandatory whole-diff adversarial exec review → merge no-ff (signed+DCO) → push bg002h.
