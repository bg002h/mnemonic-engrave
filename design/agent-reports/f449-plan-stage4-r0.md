# F-449 stage 4 (device) plan: R0 architect review

**Verdict: NOT GREEN. 0 Critical, 1 Important, 5 Minor, 2 Nit.**

- Artifact: `design/IMPLEMENTATION_PLAN_f449_stage4_device.md` at engrave `master` (7ed3f9a2).
- Tree reviewed: a scratch clone of `/scratch/code/shibboleth/.tmp/s4-plan/verify-real` (d2350cb plus the plan's T2..T7 commits; `gen.py` generates the plan's code blocks from this tree), at `/scratch/code/shibboleth/.tmp/r0-s4/fork`.
- Host oracle: md 0.19.0 (`.tmp/s4-plan/dm-target/release/md`). Liana oracle: the v15.0 harness (`.tmp/s4-plan/liana-gate-target/debug/liana-harness`).
- I did not re-derive the controller's apply gate, the author's walk, the mutation run or the size numbers.

---

## Important

### I1: a kind-1 **template + mk1 key cards** never derives an address. §7a.2 is unmet on the D3 route, which is the verify route the plan's own journey leads to.

**Mechanism.** `lianaInternalKey(collected, network)` (Task 4) computes the recipe with `md.LianaUnspendableKeyChunks(collected)`, which reads each leaf key from the md1's **Pubkeys TLV** (`lianaLeafPubkeys` → `xpubForId`).

On the Wallet Policy D3 route, `walletPolicyConsentLines` seats the mk1 cards into `keys` (`gui/wallet_policy.go:208`) but passes the **keyless template** as `md1`. The resulting chain is:

1. `policyAddressAt(md1, tpl, keys)` calls `complexAddressSource(template, seatedKeys)`.
2. `lianaInternalKey` finds no TLV and returns `errLianaMissingKey`.
3. `complexAddressDeriver` returns `nil, false`.

The NUMS twin derives on the same route, because NUMS needs no leaf xpubs.

SPEC §7a.2 says the third branch "recomputes §2 over the **collected leaf keys**". On this route, the leaf keys are the seated cards.

**Counterexample (run, `gui/r0_liana_seat_test.go` in the scratch clone).** It uses the seatFixture construction from `gui/key_card_seating_test.go` (`StripToTemplate` + `FormAwareStubChunks` + `mk.Card` per slot from the vector's xpubs), fed to `walletPolicyConsentLines(template, cards)`:

| vector | consent shows | Rust receive 0 |
| --- | --- | --- |
| `keyed_tr_with_leaf` (kind "a key") | Receive 0 `bc1p2ps82seq…` ✓ | same |
| `keyed_compose_preset_kofn_recovery` (kind 0) | Receive 0 `bc1p0w9889dw…` ✓ | same |
| **`keyed_tr_liana_kofn_recovery` (kind 1)** | **"This device can't derive / addresses for this policy."** | `bc1py0prh5dupvw3egy8acnvrh0kdnma9ywvrlza0547a4lzvw5sgmjqw56q3f` |

The full kind-1 keyed card derives correctly on the same function (`walletPolicyConsentLines(keyed, nil)` shows the address). Only the template-plus-cards form fails.

**Why it matters (journey lens 5).** The plan's own `liana` walk arm engraves a **key-less template** ("No slot is seated", census 1 plate). The operator later proves the wallet by bringing the cosigners' mk1 plates to Wallet Policy with that template, and gets "This device can't derive addresses for this policy." That is F-214's unsupported-shape sentence, and it tells them their Liana wallet is a shape the device doesn't support. The kind-0 twin of the same plates shows addresses.

This is not a wrong address. The device fails safe, so the finding is not Critical. It is Important for three reasons:
- the stage's stated goal ("the device derives that wallet's real addresses") and §7a.2 are unmet for the multi-party form the composer exists to produce;
- the plan's journey table row 13 claims the opposite ("addresses on Button2"), which is true only for a full keyed card;
- nothing in the plan tests this route for kind 1.

**Remedy (prototyped and measured).**
- Compute the recipe from the ExpandedKeys the deriver already receives, in the tree's key-occurrence order, instead of from the TLV. For example, add `md.LianaUnspendableKeyFor(strs []string, xpubs map[uint8][65]byte) ([65]byte, error)`, which walks `collectKeyOccurrences` over the root tr's tree and takes `xpubs[i][32:65]`. `lianaInternalKey` would then take `keys` (`k.Index → k.Xpub`).
- I applied this in the scratch clone. The kind-1 template + cards consent then shows `bc1py0prh5…`, and `TestDeviceDerivesLianasOwnAddressesForKind1`, `TestEveryKeyedVectorReachesAnAddress`, `TestInspectNamesTheLianaKindAndItsPolicyId` and all my differentials below still pass.
- Add a Task 4 test: the seatFixture over `keyed_tr_liana_kofn_recovery` through `walletPolicyConsentLines` must show the Rust receive 0. Add a mutation row for it: reading the recipe from the TLV reproduces the defect.
- Correct journey row 13 and Review Focus 2 to name the template+cards route.

---

## Minor

- **m1: `UnspendableRequestUnmet` has no production caller.** A grep of `gui/` and `md/` non-test code finds only its definition. md-cli's `liana_refuse_or_warn` acts on it. The device relies on the reset predicate, with the self-check biconditional as a belt at **consent**, which comes after the stub screen has shown an id. Remedy: have `composerCompose` refuse a Liana request that went unmet (one line, plus a test that constructs the state). Otherwise say in the Global Constraints why it is deliberately unused.

- **m2: two new tests use inputs the SH2 lacks.** This contradicts the plan's own "Touch only" constraint (Global Constraints: no new test moves a cursor with Up/Down).
  - `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` uses `click(&ctx.Router, Down)` and `Down, Down, Down` to reach "Build a new policy" and `kofn-recovery`.
  - `TestInspectNamesTheLianaKindAndItsPolicyId` pages with `click(Button3)`.

  The emulator arm reaches the same screens by touch, so this is a harness honesty issue, not a gap. Remedy: use `chooseRow`/`tapRow` over `plateHitPoints`, or drop the "no new test" wording.

- **m3: the RESET converse half can pass without the step returning.** In `TestComposerUnspendableResetIsThePredicate`, the second half checks only `st.unspendable == Liana` after `tapNav(Button3)`. If the press-through never registers, the kind stays Liana and the test passes. The first half and `TestComposerUnspendableDefaultRow` do check `*h.done && ret`. Remedy: assert both here too.

- **m4: the one-compose-site gate has a blind spot.** `TestComposerComposesOnlyThroughOneSite` exempts the whole of `composer_unspendable.go`, because `composerUnspendableFires` legitimately composes kind 1 for the predicate. A second production compose added in that file bypasses the gate. Remedy: exempt by enclosing function (`composerCompose`, `composerUnspendableFires`), not by file.

- **m5: citation drift in prose.**
  - F14 cites `composerRestoreDoc` at `gui/composer_flow.go:534`. At d2350cb it is `:528`.
  - Task 2's Interfaces says `ComposeWith` has "69 test callers". At d2350cb there are 16 `ComposeWith(` call sites in test files and 4 in non-test files.

  Neither is load-bearing. Re-resolve both in Task 9's sweep.

## Nit

- n1: the drop modal's cause for a failed compose reads "Liana would not import this policy (a policy this device cannot build)." That is a compose error, not a Liana verdict. It is unreachable while `composerTemplateChunksFor` refuses first.
- n2: the Liana row and §8y copy say "Liana (v15.0) … import it". Liana v15.0 also refuses two keys from one master in one spending path (measured below). The existing SAME SEED consent notice covers that case at seat time, so this is not worse than silence.

---

## What I verified sound (measured, not read)

### Lens 1: funds

The device agrees with Rust md 0.19.0 on shapes beyond the plan's corpus. The test is `gui/r0_liana_diff_test.go`: keyed md1 minted by `md encode`, then the device's `complexAddressSource`, then `md address --index i --change`.

Coverage: 9 kind-1 shapes, at indices 0, 1, 7, 1000 and 2^31-1, on both receive and change. The shapes:
- balanced nested `{{multi_a,pk},{…older,…multi_a older}}`;
- a left-deep 3-level tree;
- multi-key leaves in non-sorted order;
- `thresh`, `or_d` and `pkh` leaves;
- an 11-key tree;
- one leaf `pk`;
- four keys from **one master** (shared fingerprint, accounts 0..3).

Cards ranged from 3 to 21 chunks. Every address matched. **Negative control:** mutating the Liana arm to ignore `change` produced 45 mismatches, so the differential can fail.

The device also agrees with **Liana v15.0 itself** on two accepted shapes outside the plan's five cases. Both came from harness `unspendable` then `parse`, then device derivation from `md encode` of the same wallet:
- **recovery-first leaf order** (recipe order D,A,B,C, not the composer's): receive and change 0..2 equal;
- **1-of-2 primary**: equal.

There is no path by which the device shows a kind-1 address that differs from Rust's or Liana's:
- `taprootInternalKey` has no default that derives;
- the Liana key is built with an explicit `<0;1>/*`;
- Rust's `to_miniscript.rs:505-512` also always uses `<0;1>`, so the two agree even on a hand-crafted non-`<0;1>` card that neither can mint.

### Lens 2: the choice screen

- **Predicate on the six tr presets:** plan test green. Hand-built equivalents behave as follows (`gui/r0_pred_test.go`):

  | hand-built shape | fires? | Liana v15.0 verdict |
  | --- | --- | --- |
  | kofn built by hand | yes | — |
  | recovery path listed first | yes | ACCEPTS |
  | 1-of-2 primary | yes | ACCEPTS |
  | two recoveries | yes | — |
  | older in 512-s units | no ("a lock in time units") | refuses |
  | wsh kofn | no (notTr) | — |

- **Placement, reset, Back and re-entry.** Every forward pass reaches `composerTemplateChunksFor` and `composerArtifactsFor` only through the step: it sits inside `composerFlow`'s loop, and every Back (stub, seating, consent) `continue`s to the top. The composition-state hook carries hashes only, so no `unspendable` field goes unexported.
- **Default row seeding:** from `st.unspendable`. `md.UnspendableNums` is the zero value.
- **First page:** `l01-key-path.png` shows the lead and both rows fully drawn, with NUMS highlighted, and two tap targets.

### Lens 3: the survivor and the walk

- `capture-real.txt`: all four legs matched the host, the liana leg's engraved string is byte-equal to `md13ls8a…`, and the NEGATIVE CONTROL passed. The walk is a real gate for placement, re-entry seeding, the consent copy and the engraved bytes. It cannot see addresses, and the plan says so.
- **S8 (the `md1KeyPathUnknown` call) is acceptable as named.** No decoder yields a fourth kind, and over all 59 decodable vendored vectors (25 tr), `md1KeyPathUnknown` is false for every one (`TestR0NoShippedTrVectorIsKeyPathUnknown`), so the guard cannot misfire today.

### Lens 4: false passes

No false pass found in the Task 2-6 tests beyond m3 and m4.
- `TestDeviceDerivesLianasOwnAddressesForKind1` pins its case count at 5 and fails on an unmapped accepted case.
- The DEFAULT ROW test starts from a zero-value state.

### Cited line numbers

Task 1 Step 2's greps at d2350cb all resolve as cited: `composer_flow.go` 96/98/268/283/308, `policy_address.go` 157, `composer_consent.go` 205/397/407, `template_engrave.go` 159, `md1_inspect.go` 84, `policy_address_test.go` 128, `composer_copy_test.go` 452, `composer_paged.go` 295. F10 holds: `gui/composer_copy_gate_test.go` does not exist.

---

Scratch artifacts: `/scratch/code/shibboleth/.tmp/r0-s4/` (the clone with `gui/r0_*_test.go`, the I1 remedy prototype in `md/liana.go` and `gui/policy_address.go`, and the Liana probe files).

ready for implementation: no
