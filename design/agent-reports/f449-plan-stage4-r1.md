# F-449 stage 4 (device) plan: R1 scoped re-review of the R0 fold

**Verdict: GREEN. 0 Critical, 0 Important, 0 Minor, 3 Nit.**

- **Fold reviewed:** engrave `48c30cc7..HEAD` (HEAD `5ca41000`), covering `design/IMPLEMENTATION_PLAN_f449_stage4_device.md` and `scripts/f449-stage4-mutations.sh`.
- **Findings folded:** `design/agent-reports/f449-plan-stage4-r0.md`: I1, m1..m5, n1, n2.
- **Tree:** my own run of the apply gate. It is a copy of `scripts/plan-apply-gate-go.sh` with only `W` changed to `/scratch/code/shibboleth/.tmp/r1-s4/gate`, so it did not touch the controller's `.tmp/f449s4-apply-gate`. It ran against fork `d2350cb` and produced **ALL BOUNDARIES GREEN**:
  - Tasks 2..7 green;
  - md 185/188/188…;
  - `RESULT: ok -- all 1398 tests ran across 24 shards`.
- **Working clone for probes:** `/scratch/code/shibboleth/.tmp/r1-s4/fork`, committed as `ae59ea2` on top of `d2350cb`.
- **Oracle:** md-cli built fresh from descriptor-mnemonic `main` `d269c556` (`md 0.19.0`, md-codec 0.47.0), at `/scratch/code/shibboleth/.tmp/r1-s4/dm-target/release/md`.

## Per-finding table

| R0 finding | Status | Evidence (measured) |
| --- | --- | --- |
| **I1**: template + mk1 cards never derives kind 1 | **ADDRESSED** | See §1. `lianaInternalKey(collected, keys, …)` builds its map from the deriver's `keys`, and `md.LianaUnspendableKeyFor` walks `collectKeyOccurrences` over the root tr tree. Across 13 kind-1 shapes, with cards in order, reversed and rotated, the template+cards route equals Rust md 0.19.0 at indices 0, 1, 7, 1000 and 2^31-1 on both chains. A wrong card gives the wrong-key wallet's Rust address, never the original's. A duplicate xpub is refused. The keyed route also still equals Rust (R0's 9 shapes). A negative control (reversed recipe order) produced 470 mismatches. Mutation K1 reddens `TestTemplatePlusKeyCardsDerivesTheLianaWallet` on the kind-1 row. |
| **m1**: `UnspendableRequestUnmet` had no production caller | **ADDRESSED** | See §2. `composerCompose` now refuses it. It cannot refuse a met request. Mutation U1 is in the table. `TestComposerComposeRefusesAnUnmetLianaRequest` constructs the unmet state, and the gate ran it green. |
| **m2**: two tests used inputs the SH2 lacks | **ADDRESSED** | `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` uses `h.choose`/`h.tapNav`, and `TestInspectNamesTheLianaKindAndItsPolicyId` uses `tapNavSlot`. `h.choose` (`gui/unlock_session_test.go:184`) taps `plateHitPoints` and then `tapNav(Button3)`. `tapNavSlot` (`gui/unlock_platelist_test.go:179`) hit-tests the nav slot. The plan's grep returns nothing. So does `git diff d2350cb -- 'gui/*_test.go' \| grep -E '^\+.*(click\(\|Down\|Up\b\|ButtonEvent)'` over all 9 changed test files. |
| **m3**: the RESET converse half could pass without returning | **ADDRESSED** | `ret = false` plus `if !*h.done \|\| !ret` were added. **R4 re-applied by me:** it fails with `the re-entered step did not return forward (done false, ret false)`. |
| **m4**: the one-compose-site gate exempted a whole file | **ADDRESSED** | The exemption is now per enclosing `FuncDecl`, and the test checks that each allowed site composes exactly once. **E1 re-applied by me:** it fails with `composer_unspendable.go:102:9: md.ComposeWithUnspendable in composerUnspendableStep; only composerCompose\|composerUnspendableFires may compose`. |
| **m5**: citation drift | **ADDRESSED** (one Nit, N1) | At `d2350cb`, `git grep -n "func composerRestoreDoc"` gives `gui/composer_flow.go:528` ✓. There are 16 `ComposeWith(` hits in `*_test.go` files, all call sites ✓. "4 in production" counts the `func ComposeWith(` definition (`md/compose.go:653`) as a call site; there are 3 call sites (`composer_flow.go:268/283/308`). See N1. |
| **n1**: the drop cause for a compose failure read as a Liana verdict | **ADDRESSED** (one Nit, N2) | The `unbuilt` field and the cause "This device could not build the policy with the Liana key." are true statements wherever they can show. `TestComposerLianaKeyDroppedFits` covers `{unbuilt: true}` and a real class string (`"two paths with one lock"`, `gui/composer_consent.go:473`). |
| **n2**: "import it" ignores Liana's same-master refusal | **ADDRESSED** | Journey row 13a quotes the §8g copy verbatim. `composerCopySameSeed*` returns "SAME SEED, SAME PATH … Liana will refuse it." (`gui/composer_copy_test.go:89,91`), and it is shown by the mapping review (`gui/composer_review.go:196-198`), as the row says. |

## §1. I1: can "the keys the deriver is given" and "the keys the recipe uses" diverge?

**Structure.** One `keys` slice feeds three things in `complexAddressDeriver`:
- `byIndex`, which builds every leaf script;
- `taprootInternalKey`;
- `lianaInternalKey`, which builds `xpubs[k.Index] = k.Xpub` and calls `md.LianaUnspendableKeyFor`.

`LianaUnspendableKeyFor` indexes that map by the placeholder indices of the same decoded tree (`collected`) that `EmitTapLeavesChunks` emits leaves from. On the Wallet Policy route, `seatKeyCards` (`gui/key_card_seating.go:53`) fills each template slot by origin and fingerprint, requires every slot filled, and refuses a contested slot. So the leaf keys and the recipe keys are the same bytes by construction.

`grep -rn "LianaUnspendableKey\|lianaLeafPubkeys\|lianaInternalKey(" gui md --include='*.go' | grep -v _test.go` finds exactly one production call of the recipe (`gui/policy_address.go:250`), which confirms the plan's "one key source" constraint.

**Execution.** The harness is `gui/r1_liana_seat_test.go` in the scratch fork. For each case:
1. The keyed md1 comes from `md encode … --key … --fingerprint …`.
2. The template comes from `md encode` of the same text with fingerprints only, so same-path slots can seat. `StripToTemplate` drops the fingerprints, and shared-path templates then refuse at seat with "two different cards claim one slot", which is the pre-existing F-414-class limit.
3. There is one `mk.Card` per slot, carrying the template's stub, path and fingerprint. The slot→xpub map is resolved **by matching the keyed md1's TLV bytes**, because Rust renumbers placeholders to first occurrence. A first draft of my harness that assumed text numbering produced false mismatches.
4. The cards go through `walletPolicyConsentLines`, `seatKeyCards` and `complexAddressSource`, and each address is compared with `md address --index i [--change]` on the keyed md1.

| shape (kind 1) | in order | reversed | rotated | wrong card in last slot | same xpub in @0,@1 |
| --- | --- | --- | --- | --- | --- |
| nested-balanced (6 keys) | match | match | match | match (wrong-key wallet) | refused (F-218) |
| left-deep (6) | match | match | match | match | refused |
| right-deep (5) | match | match | match | match | refused |
| multi-key-leaf-order (7) | match | match | match | match | refused |
| same-seed-fps (4, one master, divergent paths) | match | match | match | match | refused |
| divergent-paths-no-fp (4) | match | match | match | match | refused |
| single-leaf-pk (1) | match | match | — | match | — |
| thresh-leaf (5) | match | match | match | match | refused |
| or_d-leaf (3) | match | match | match | match | refused |
| pkh-leaf (2) | match | match | — | match | refused |
| many-keys (11) | match | match | match | match | refused |
| **nonmono-leaf-order** `{pk(@2)+older, multi_a(2,@1,@0)}` | match | match | match | match | refused |
| **nonmono-in-leaf** `multi_a(2,@2,@0,@1)` | match | match | match | match | refused |
| slot in two leaves | Rust refuses to encode (BIP 388 disjointness) | | | | |
| multipath `<2;3>` / mixed `<0;1>`+`<2;3>` | Rust refuses to encode (kind 1 requires `<0;1>`) | | | | |
| `sortedmulti_a` leaf | Rust refuses to encode | | | | |

- **"match"** means every one of the 10 addresses (5 indices × 2 chains) equals Rust's, and the consent lines show the deriver's receive 0.
- **Wrong card:** the device shows the Rust address of the wallet built with the wrong key, and my test checks that this is **not** the original wallet's receive 0. Leaves and internal key move together, so no mixed wallet can appear.
- **Duplicate xpub:** Rust refuses to encode, and the device refuses at consent with "Slots @0 and @1 hold the same key…".
- **Keyed route**, after the fold changed its key source from `xpubForId` to `ExpandedKey.Xpub`: R0's `TestR0LianaDeviceVsRust` re-run on the folded tree passes 9/9 shapes at 5 indices × 2 chains.
- **Negative control:** mutating `LianaUnspendableKeyFor` to hash the occurrences in reverse gave 470 MISMATCH lines, failing 12 of the 13 encodable shapes. single-leaf-pk is order-insensitive.

Row 13's new claim was also run (`gui/r1_inspect_tpl_test.go`). Inspecting the kind-1 template alone shows "Complex policy - display only … Key path: Liana key", with no `Policy id:` and no address, as the row says.

## §2. m1: can the new refusal block a valid wallet?

No.
- `UnspendableRequestUnmet` is `requested == Liana && internalKey() != InternalKeyLianaUnspendable` (`md/compose_unspendable.go:65`).
- In `lowerTr` (`md/compose.go:933-975`), `ikKind` depends **only on `list`**: Liana is chosen exactly when no path `isBareSingle()`. A non-tr root is always "unmet". The declared origins do not enter the decision.
- `composerUnspendableFires` resets `st.unspendable` to NUMS on both `notTr` and `InternalKeyPath() real`, which are exactly those conditions, over the same `st.list`.
- Every write to `st.list` in production (`grep`: `composer_flow.go:233`, the start step; `composer_shape.go:471-724`, the shape flow) happens before `composerUnspendableStep` in `composerFlow`'s loop, and every Back `continue`s back through the step.
- Seating (`composerSeatingStep`) does not touch `st.list`.

So at each of the three `composerCompose` call sites (`composer_flow.go:274/289/314`), a surviving Liana request is always met. A NUMS request can never trip the refusal.

## §3. Mutations re-applied by me (exact-once apply, build, targeted test, revert)

| row | result |
| --- | --- |
| K1 (`lianaInternalKey` reads the md1's own keys) | FAIL `TestTemplatePlusKeyCardsDerivesTheLianaWallet`: "the consent does not show Rust's receive 0 bc1py0prh5dupvw3egy8acnvrh0kdnma9ywvrlza0547a4lzvw5sgmjqw56q3f" |
| R4 (the converse half's press-through deleted) | FAIL `TestComposerUnspendableResetIsThePredicate` (message quoted above) |
| E1 (a second compose in `composerUnspendableStep`) | FAIL `TestComposerComposesOnlyThroughOneSite` (message quoted above) |

All three use only SH2 inputs: touch taps and nav-slot taps, with no directional buttons.

## New findings

### Critical
None.

### Important
None.

### Minor
None.

### Nit
- **N1 (m5 remnant):** Task 2's Interfaces says "4 in production at `d2350cb`". There are 3 production call sites (`gui/composer_flow.go:268/283/308`). The fourth hit is the definition `md/compose.go:653`. The same sentence's "Task 6 moves the 3 composer ones" is right. The fix is wording only.
- **N2 (n1 comment):** the new `unbuilt` comment in `composerUnspendableDropCause` says "unreachable (composerTemplateChunksFor refuses first, a few lines later …)".
  - The order is the reverse: if it were ever reached, the step's modal draws first, and `composerTemplateChunksFor`'s refusal draws after it.
  - It is also only "unreachable" in the sense that `md.ValidatePathList` gates Done (`composer_shape.go:685`) and the kind moves only `ikKind`.
  - The shown text is true either way.
- **N3:** `md.lianaLeafPubkeys` (the TLV-reading walk the fold retired from production) remains in `md/liana.go` with its own copy of the occurrence walk. It now has test-only callers, including as the `want` in `TestLianaUnspendableKeyForIsTheRecipeOverTheSuppliedLeaves`. It is harmless: TinyGo drops it, and the Liana-golden test anchors the recipe externally. It is a second copy of the rule the fold's Global Constraint names, so Task 9 could fold it into `LianaUnspendableKeyFor` or say why it stays.

## Scratch

`/scratch/code/shibboleth/.tmp/r1-s4/` holds:
- `gate/` (the apply-gate run) and `gate.log`;
- `fork/` (probes: `gui/r1_liana_seat_test.go`, `gui/r1_inspect_tpl_test.go`, and R0's `gui/r0_liana_diff_test.go` repointed at my md);
- `dm-target/` (md 0.19.0);
- `mut.py`, `mut_*.txt`, `r1seat.txt`, `ctl.txt`.

No real checkout was modified.

ready for implementation: yes
