# T6b IMPLEMENTATION PLAN — R0 review (round 0) — VERBATIM agent report

**Agent:** `a592176d3eee7b54b` (adversarial opus architect; RAN real Go probes at HEAD 072461a in a throwaway worktree). **Fork HEAD:** `072461a`. **Plan commit:** `be7cadf`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). 3 Minors non-blocking. Persisted per the R0 gate discipline; cleared for single-implementer TDD.

---

# T6b IMPLEMENTATION PLAN — R0 review (round 0)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 072461a  **Verdict:** GREEN (0C/0I)

## Fixture validation (MANDATE #1) — RAN the probe
I created a throwaway detached worktree at `seedhammer-r0probe` (HEAD 072461a), wrote
`gui/zz_r0probe_test.go` loading the 6 chunk strings VERBATIM from the plan, ran
`go test ./gui/ -run TestZzR0Probe -v`, and got a clean PASS. ACTUAL outputs:

- decode: `Root=ScriptWsh(3)`, `Policy=PolicySortedMulti(2)`, `K=2 N=3 Renderable=true`,
  `len(keys)=3`, every key `XpubPresent=true`, every `OriginPath.String()="m/48h/0h/0h/2h"`.
- abandon-about @ m/48'/0'/0'/2': `cc=bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763`,
  `pk=021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29`, `masterFP=73c5da0a`
  — EXACTLY the plan's claimed bytes.
- slot match: `@0 false`, `@1 TRUE (cc&pk both equal via bytes.Equal over 32/33B)`, `@2 false`
  — the abandon seed is slot @1 and ONLY @1, as claimed.
- `WalletPolicyIDStubChunks` = `0x7b716421` — matches.
- `expandedToDescriptor` = `(non-nil desc, expandOK)`; `address.Receive(desc,0)=bc1qg2lsdla23zewexuhn5jcx49mqzs8wqss0lxguarfpnt7ysg7k52slz4dxd`,
  `address.Change(desc,0)=bc1qz76qjcmpwhh6ffenfwg44hpq3cwwfuqcr54vl4485yttpjtxy9qq3yufkt` — both match.
- `mk.Encode` (Network:"mainnet", Path:m/48'/0'/0'/2', FP:73c5da0a, Stubs:[7b716421], Xpub:abandon@1)
  → `mk.Decode` Path=`m/48'/0'/0'/2'`, Fingerprint=`73c5da0a`, Stubs=`[[123 113 100 33]]` (=0x7b716421);
  bundle `{ms1=EncodeMS1(entropy), mk1, md1=chunks}` → `bundle.Verify(b,b)=nil` (PASS).

A second probe confirmed Task-2's TestFindUserSlot premises: deriving the abandon seed at the
three DISTINCT origins (o0/o1/o2) yields three DISTINCT keys (no collision), and the
`foreignXpub()` 0x40+i pattern never collides with any abandon derivation — so the
"match @1 / refuse non-cosigner / ambiguous @0+@2 / skip XpubPresent=false" subtests are all
genuinely exercised. Baseline `go test ./gui/... ./md/... ./bundle/... ./mk/... ./codex32/...`
is all `ok` (regression base is clean). I then removed both probe files and the worktree.

EVERY pinned golden in the plan is verified-correct against real code. The vendoring approach
is sound. On the DESIGN trade-off: vendoring an opaque string + the `TestSuppliedMultisigFixtureIsFullPolicy`
structural guard is ACCEPTABLE here. There is no exported multisig encoder in-tree (T6c deferred),
so programmatic construction at test time is not available without writing throwaway encoder
scaffolding — which would itself be unverified. The guard test re-decodes the fixture on every
run and asserts the full-policy shape + @1 match, so a corrupted fixture string fails loudly at
Task 1.5 rather than silently corrupting downstream asserts. The goldens are NOT merely
self-consistent (stub-to-stub) regression detectors: the addresses come from
`expandedToDescriptor`→`address` (an independent derivation path from the stub/match path), and
the mk1 stub is independently re-computed by `WalletPolicyIDStubChunks` and cross-checked against
the bundle.Verify stub-binding — adequate for the §5 gates.

## Critical
None.

## Important
None.

## Minor
- **m-1 (Task 1.5 fixture provenance — non-blocking).** The plan vendors the md1 as an opaque
  string "generated during authoring" with no recorded recipe. The guard test proves it decodes
  correctly, and my probe proves the goldens, so it is not a correctness risk. But if the fixture
  ever needs regeneration (e.g. md1 format churn), there is no documented procedure — Task 1.5's
  own comment says "do NOT regenerate it ad hoc; re-derive it via the documented descriptor (see
  the plan's Test Vectors)," yet the Test Vectors section only describes the *result*, not the
  encoding inputs (the Pubkeys-TLV descriptor + the two foreign pubkeys). Suggest recording the
  foreign @0/@2 pubkeys (or the source descriptor) in a comment in the testdata file so a future
  maintainer can reproduce it. Does not block — the fixture is proven-correct as vendored.
- **m-2 (Task 7 import-trim judgment call — non-blocking).** Task 7 ships `gui/multisig_restore.go`
  with a placeholder `var ( _ = image.Pt; _ = op.Layer; ... )` block and a paragraph instructing
  the implementer to delete it + the matching imports if `go build` flags them unused. This is
  the one spot in an otherwise fully-spelled-out plan that leaves a build-shaped decision to the
  implementer. The plan gates it with `go build ./gui/` (Task 7 Step 4) and names the exact minimal
  import set (`address`, `md`), so it is recoverable and not a true placeholder — but it is the
  least-clean task. Recommend the implementer just write the minimal 2-import file directly.
- **m-3 (residual, carried from spec n-1).** `extractSuppliedMd1`'s `cardMS1` refusal clause
  (Task 1) is defensive dead code — the gather path never produces a `cardMS1`. The plan's own
  test comment (lines 168-169, 202) and the code comment (lines 233-236) already flag it as n-1
  defensive, so this is fully disclosed. No action needed.

## Verified-correct
- **Fixture & all goldens** — ran real code, see MANDATE #1. Exact match on decode shape, abandon
  cc/pk, slot @1-only match, stub 0x7b716421, both addresses, mk1 fields, bundle.Verify=nil.
- **D14 cross-match code (MANDATE #2)** — Task 2 `findUserSlot` (plan lines 583-609) is correct on
  every point: (a) compare is `bytes.Equal(cc[:], k.Xpub[0:32]) && bytes.Equal(pk[:], k.Xpub[32:65])`
  over the full 32+33 bytes, never `==`, never base58 (line 597) — verified `Xpub` is `[65]byte`
  at `md/expand.go:62`, `decodeXpubBytes` returns `[32]byte/[33]byte` at `gui/singlesig_derive.go:99`,
  so both slices compile and compare all 33 pubkey bytes incl. index 64; (b) derives at each slot's
  OWN `k.OriginPath` (line 589); (c) refuses with `ok=false` on zero matches (lines 601-602);
  (d) ambiguous rule is deterministic — `matches[0]` first-by-index + the full `matches` slice as
  `reused` (lines 604-607); (e) `XpubPresent=false` slots are `continue`-skipped (line 588);
  (f) test data construction is sound — `abandonSlotXpub` derives the real abandon key at the
  chosen origin and packs it cc‖pk; `foreignXpub` is a fixed non-colliding pattern (I proved the
  no-collision premise by running the derivations). All four outcomes (match@1 / refuse-zero /
  ambiguous@0+@2-first+notice / skip-not-present) are exercised by the four subtests (lines 475-537).
- **I-7 scrub spine (MANDATE #3)** — Task 8 orchestrator scrubs the typed mnemonic via a top-level
  `defer` placed immediately after `seedEntryFlow` (plan lines 1509-1513), firing on EVERY exit
  incl. no-match refuse (line 1527), abort, and the early gate refuses (gather/extract/decode/
  full-policy refusals all occur BEFORE the seed is typed, so no secret exists yet). Seed is
  typed-only via `seedEntryFlow` (line 1500) — never `gui/scan.go`. The verify flow types a SECOND
  seed `reMnemonic` with its OWN defer scrub (lines 1143-1147). No path serializes an xprv:
  `deriveMultisigLeg` only ever calls `deriveAccountXpub` (which neuters/serializes xpub internally
  and scrubs seed/master) and `EncodeMS1`; the restore path builds the descriptor from public
  cc‖compressed-pubkey only (`expandedToDescriptor` `gui/md1_expand.go:62-66`), and `Descriptor.Encode`
  emits only xpub/script (`bip380/bip380.go:171`) — the xprv-grep gate (Task 10 Step 3) will be clean.
  NOTE: the orchestrator holds `b.MS1` (a plaintext ms1 Go string) un-scrubbed until GC and passes
  it to `multisigVerifyFlow` — this is byte-identical to the SHIPPED, R0-blessed T6a single-sig
  flagship (`gui/singlesig.go` passes `b` to `singleSigVerifyFlow`; Go strings are immutable and
  cannot be zeroed in place), so it introduces no NEW leak relative to the established baseline.
- **I-9 lockstep (MANDATE #3)** — grep of non-test gui confirms EXACTLY the 8 `engraveSingleSig`
  sites the plan targets: enum `:151`, dispatch `:1501-1502`, left-wrap `:1638`, right-wrap `:1645`,
  title `:1670`, npage `:1846`, layoutMainPlates `:1856`, npages `:1865` — plus the flow def
  (`gui/singlesig.go:30`, untouched). Task 9 Steps 3a-3h edit all 8, INCLUDING the mandatory
  `layoutMainPlates` case (Step 3g, the `panic("invalid page")` default at `:1856-1861`). The 3
  nav-tests: I confirmed `singlesig_program_test.go` is the ONLY one with live wrap-to-backupWallet
  assertions (right-wrap line 52-60 + left-wrap line 74-81) — Task 9 Step 4 retargets both;
  `bundle_program_test.go`/`derive_xpub_program_test.go` are comment-only at the bound and stop at
  Single-Sig/Bundle (their assertions still pass), which the plan correctly states (line 1795).
  qaProgram stays non-navigable (3 name-based refs `:152/:1492/:1606`, no insert change). TestAllocs
  unaffected (enum cases don't allocate).
- **I-2 verbatim engrave** — Task 5 `multisigEngraveCards` puts the supplied md1 into the cardMD1
  `strings` via `append([]string(nil), md1...)` (clone, no re-encode); Task 4 `deriveMultisigLeg`
  sets `b.MD1 = append([]string(nil), suppliedMd1...)` verbatim. `bundleEngrave`→`bundlePlatePlan`
  iterates `c.strings` per-plate unchanged (`gui/bundle_flow.go:303-318`). No re-encode anywhere.
- **Spec coverage I-1..I-11 / §5.1-§5.7** — all map to tasks with real failing-test-first TDD:
  I-1/§5.1→Task 2 (+extract in Task 1); I-2→Tasks 4/5; I-3→Tasks 1+3; I-4/§5.2→Task 4; I-5/§5.4→Task 6;
  I-6/§5.5→Task 7; I-7/§4→Task 8 (+ scrub seam Task 10); I-8→Tasks 4/8 (`MainNetParams`, `Network:"mainnet"`);
  I-9/§5.6→Task 9; I-10/§5.7→Task 10; I-11/§5.1→Task 1. No gap.
- **API/type consistency** — every cited signature matches 072461a source: `ExpandWalletPolicyChunks`
  (`md/expand.go:102`), `ExpandedKey{Xpub [65]byte}` (`:56-64`), `decodeXpubBytes` (`gui/singlesig_derive.go:99`),
  `deriveAccountXpub` (`gui/derive.go:19`), `WalletPolicyIDStubChunks` (`md/walletpolicyid.go:126`),
  `mk.Encode`/`Card`/`Decode`, `EncodeMS1`/`DecodeMS1(String)`/`New`, `expandedToDescriptor`+`expandOK`,
  `address.Receive/Change`, `bundle.Verify`/`Bundle{MS1,MK1,MD1}`, `bundleGatherFlow`/`bundleEngrave`/
  `bundleShowMs1Reminder`, `seedEntryFlow`/`passphraseFlow`/`ChoiceScreen.Choose`/`showError`/`wipeBytes`,
  `inputCodex32Flow (any,bool)` + `obj.(codex32.String)` (mirrors `gui/singlesig_verify.go:110`),
  helpers `chunkString`/`scriptName(md.ScriptKind)`/`policyLine`/`restoreDocScreen`. `findUserSlot`,
  `deriveMultisigLeg`, `multisigEngraveCards`, `extractSuppliedMd1`, `allSlotsHaveXpub`,
  `multisigRestoreLines` signatures are identical across all referencing tasks (verified in the
  plan's own §self-review §3, which I cross-checked).
- **TDD discipline** — every code task writes a failing test first (run + expected FAIL shown),
  then minimal impl, then PASS, then explicit commit. Task 3 and Task 8 are the two non-failing-first
  tasks and both are correctly justified (Task 3 locks already-written `allSlotsHaveXpub` against
  real goldens; Task 8 is UI composition of already-tested units, covered structurally by build +
  Task 9 nav-test + Task 10 regression). No "TODO"/"similar to above"/un-shown code elsewhere.

## Bottom line
GREEN — 0 Critical / 0 Important. I independently RAN real Go code at HEAD 072461a and every
pinned golden the entire downstream suite asserts against is verified-correct: the vendored
fixture decodes to the claimed full-policy 2-of-3 wsh(sortedmulti), the abandon-about seed matches
slot @1 and ONLY @1, the stub is 0x7b716421, both receive/change addresses match, the mk1 round-trips
to the claimed fields, and bundle.Verify(b,b)=nil. The vendoring + structural-guard approach is
sound. The D14 cross-match code is correct on the compare (bytes.Equal over the full 32+33 bytes),
per-slot origin, refuse-on-zero, deterministic ambiguous rule, and XpubPresent skip. The scrub
spine, typed-only seed, verbatim engrave, faithful-or-refuse restore, and the 8-site+3-nav-test
lockstep are all complete and source-accurate. The three Minors (fixture-regeneration recipe,
Task-7 import-trim judgment, disclosed cardMS1 dead branch) are cosmetic and non-blocking. This
plan is cleared to begin implementation.
