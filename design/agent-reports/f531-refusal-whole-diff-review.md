# F-531 refusal — whole-diff adversarial review

Scope: `git diff 3bb9f91..a4760e1` in `/scratch/code/shibboleth/seedhammer`, plus
`git show a832433b` in `/scratch/code/shibboleth/mnemonic-engrave`.
One question: does it close the defect it claims to close, and does it introduce a new one?

Reviewer did not author the change. Everything below was run, not reasoned about, except
where explicitly marked as a structural argument.

---

## The headline answer

**Yes, the funds defect is closed for the md1 route, on both address paths.** I could not
construct an input that reaches an address for a repeated-seat multisig through any gui
entry point that starts from an md1 card. The gate is exact and I could not defeat it.

**But the diff leaves three Important problems**: the F-514 warning gate it strengthened
is now unable to fail, a consent screen that precedes steel is silent about the reuse for
the keyless form of exactly this shape, and the records commit moved one spec section and
left two others describing text the device no longer shows.

---

## What I could NOT break (negative results, with their scope)

These are stated so the next reviewer does not re-spend the round on them.

### `repeatsASeat` is exact. No counterexample exists.

`gui/md1_expand.go:106`. `tpl.M != len(keys)` for `PolicySortedMulti`/`PolicyMulti`.

Proof, checked against source rather than the comment:

* `tpl.M = len(multiKeysBody.indices)` — `md/md.go:1318` (`multiPolicy`), reached only from
  `classifyPolicy` (`md/md.go:1266`), whose three multi-bearing arms are `wsh(multi)`,
  `sh(wsh(multi))` and `sh(multi)`. In all three the multi is the ONLY node in the tree
  that can hold a placeholder, so every `@i` reference is a seat.
* `len(keys) == d.n` unconditionally: `ExpandWalletPolicy` (`md/expand.go:84`) loops
  `idx := 0; idx < d.n`, and `seatKeyCards` (`gui/key_card_seating.go:104-107`) returns
  `make([]md.ExpandedKey, len(keys))` — the only two producers reaching either caller.
* `validatePlaceholderUsage` (`md/md.go:904`) refuses an unreferenced `@i` AND
  `errPlaceholderRange` for `i >= n`, so the set of distinct placeholders is exactly
  `{0..n-1}`. Therefore `M >= N` always, and `M == N` iff every seat is distinct.

So `M != N` ⟺ a repeat, with no third case. I found no false positive and no false
negative. I also checked the `sh` and `sh(wsh)` arms explicitly: both route through
`multiPolicy`, both counted the same way.

**Also checked**: `scriptForTemplate` (`gui/md1_expand.go:176`) admits `PolicySingle`
(wpkh/pkh/tr/sh-wpkh) and `PolicySortedMulti` (wsh/sh) and nothing else, so the gate's
"scoped to the multisig arm" note holds — a single slot cannot repeat.

### There is no fourth path to an address.

I enumerated every address-producing call site in `gui/` (`grep` for `address.Receive`,
`address.Change`, `address.Supported`, `address.WitnessScriptAddress`,
`address.TaprootScriptPathAddress`, `addressListFlow`, `descriptorAddressFlow`,
`verifyAddressFlow`, and every `&bip380.Descriptor{}` construction in the repo):

| producer | reached from | gated? |
| --- | --- | --- |
| `gui/wallet_policy.go:356` flat deriver | `policyAddressAt` | yes (`repeatsASeat`) |
| `gui/policy_address.go:76` emitter | `complexAddressSource` | yes (`DuplicateKeySlotChunks`) |
| `gui/multisig_restore.go:44,48` | `expandedToDescriptor` | yes |
| `gui/singlesig_restore.go:75,98` | single-sig only | n/a |
| `gui/md1_inspect.go:166` `addressListFlow` | `complexAddressSource` | yes |
| `gui/gui.go:3249,3251` `DescriptorScreen` | `descriptorFlow(desc)` | see below |
| `gui/policy_address_export.go:32` `PolicyAddressAt` | the probe | yes (router) |

`descriptorFlow` has three sources: `gui/md1_gather.go:177` (gated), `gui/gui.go:2599`
(scanned descriptor) and `gui/wallet_policy.go:122` (`nonstandard.OutputDescriptor` over a
payload record). The last two carry no Template, so neither gate can see them — that is
F-530, which the diff's rewritten comment at `gui/md1_expand.go:166-171` names correctly
and does not make worse.

### The flat/emitter split cannot leak.

Structural: `expandOK` requires `PolicySingle` or `PolicySortedMulti`; the sortedmulti case
with a repeat is refused by `repeatsASeat`; a single slot cannot produce a
`DuplicateKeySlot` hit. So `expandOK ⟹ DuplicateNone`.

Measured over the vendored corpus: **54 wallet-policy vectors, 44 reach a deriver, 0 of
those carry a non-`DuplicateNone` kind.** (Scratch test over
`md/testdata/vectors/*.phrase.txt` calling `policyAddressAt` and
`md.DuplicateKeySlotChunks`; deleted after the run.)

That is also the seed of finding I-1.

---

# Findings

## I-1 — Important — the F-514 surface gate cannot fail; the diff's own mutation claims are false

**Where**: `gui/composer_flow_test.go:652` (the claim), and the three producers it names:
`gui/wallet_policy.go:328`, `gui/composer_consent.go:219`, `gui/md1_gather.go:236`.
Also `gui/composer_flow_test.go:582`.

**The claim the diff added**:

> `// MUTATION: delete the block in any of the three producers and its row fails;`
> `// remove either gate (gui/md1_expand.go's repeatsASeat arm, or`
> `// complexAddressSource's) and the address half fails.`

and, on `TestConsentWarnsOnDuplicateKeys`:

> `// MUTATION: remove the duplicate-key block from composerConsentLinesFor and the`
> `// wsh case fails; ...`

**Both halves of the first sentence's first clause are false, and so is the second note.**

F-531's own gates made all three warning blocks unreachable. Each sits on the branch that
HAS an address, and as shown above `expandOK ⟹ DuplicateNone`, while
`complexAddressSource` now returns `!ok` for every non-`DuplicateNone` kind
(`gui/policy_address.go:66`). `policyIDHeader`'s block
(`gui/md1_gather.go:236`) is only ever called from inside
`if at, ok := complexAddressSource(...); ok {` at `gui/md1_gather.go:189-191`, so it is
dead for the same reason.

The warning that the three rows of `TestEveryAddressSurfaceCarriesTheDuplicateWarning`
now observe comes from two NEW producers instead: `noAddressLines`
(`gui/wallet_policy.go:300`) for the two consent rows, and the `showError` body at
`gui/md1_gather.go:204-206` for the inspect row. The test's "inspect descriptor" comment
still explains at length why it drives `gatheredDescriptorFlow` rather than
`policyIDHeader` — and it is now finding the string on a different screen entirely
(a modal), which the comment does not say.

**Reproduction (run)**: delete all three blocks —

```
gui/composer_consent.go:219-221   if slot, kind, err := md.DuplicateKeySlotChunks(chunks) ... }
gui/wallet_policy.go:328-330      if slot, kind, err := md.DuplicateKeySlotChunks(md1) ... }
gui/md1_gather.go:236-238         if slot, kind, err := md.DuplicateKeySlotChunks(collected) ... }
```

then:

```
$ go build ./...                                        # BUILD_OK
$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1312 tests ran across 24 shards
```

**1312/1312 green with every F-514 warning producer deleted.** The targeted set
(`TestConsentWarnsOnDuplicateKeys`, `TestEveryAddressSurfaceCarriesTheDuplicateWarning`,
`TestRepeatedSeatRefusalStillWarns`, and the four new F-531 tests) is green too.
Tree reverted with `git checkout --` afterwards; `git status` clean.

**Why it matters beyond tidiness.** F-533 — filed by the author in the same session —
prescribes moving the refusal onto a BIP-388 predicate while leaving the F-514 warning on
`md.DuplicateKeySlot`. The day that lands, these three blocks become live again, and they
are the code with no coverage. More immediately: this is the second time this gate has
gone vacuous (the author records finding the first), and the note asserting it has not is
the thing that will stop the third from being looked for.

**What would fix it**: either delete the three blocks and say in the comment that the
refusal replaced them (and correct the mutation notes), or keep them and add a test that
drives a policy which derives AND duplicates — which today requires constructing one, since
no such input exists.

---

## I-2 — Important — the consent screen before steel is silent about the reuse for a KEYLESS repeated-seat template

**Where**: `gui/wallet_policy.go:286-310` (`noAddressLines`, new in this diff), arms 1 and 2
returning before arm 3.

The function's own comment states the ordering principle:

> `// FOUR CASES, IN THE ORDER THEY BECOME TRUE. Keylessness first because it is`
> `// the loudest fact about the card; the duplicate-key refusal before the generic`
> `// one because it is the only one of the four with a cause the operator can act on.`

The duplicate arm is placed before the *generic* one but after *both* keyless ones — so
the only arm "with a cause the operator can act on" is pre-empted whenever any slot lacks
an xpub.

**Failure case (constructed and run).** A keyless `wsh(sortedmulti(1,@0,@0,@1))` template —
the exact F-531 shape, with the xpub TLV dropped. Built by taking `dupSeatDescriptor`
(`md/dup_seat_fixture_test.go:56`), clearing `d.tlv.pubPresent`/`d.tlv.pubkeys`, and
`split`ing:

```
md1fgefcqqpq2tvyyy4qqxppcq3p0px74klwllmkl4ns5uz3v66nj2ecs
```

Measured facts on that card: `K=1 M=3 N=2`, `PolicySortedMulti`, renderable;
`md.DuplicateKeySlotChunks` → `DuplicateFewerKeys @0`;
`md.TemplateEngraveShapeGuardChunks` → `nil` (admitted for template engrave, D4).

`walletPolicyConsentLines(chunks, nil)` — the screen at `gui/wallet_policy.go:148`, one
`confirmReviewScreen` and one `bundleReviewFlow` away from `bundleEngrave` — returns:

```
Template-ID: 1e8de2eae261f982a15b1e23077801a1
Type: P2WSH 1-of-3 multisig (sorted)
@0 deadbeef m/48h/0h/0h/2h <0;1>/*
@1 feedface m/48h/0h/0h/2h <0;1>/*

Template has no keys - no addresses.
Verify off-device.
```

No warning. No mention of the reuse. The operator reads "1-of-3", counts two slots, and
cuts steel. The composer's consent surface (`composerConsentLinesFor`) gives the same
answer, and the Inspect screen is worse: a keyless card returns `expandTemplateOnly`, which
`gui/md1_gather.go:178-179` routes straight to `md1DisplayFlow` — no modal, no header, no
sentence at all.

Compare the keyed form of the same card, which now says everything:

```
Check before funding: slot @0 fills more than one seat, so fewer separate keys
can spend this than its k-of-n says.

No addresses: this device does not derive them for a wallet that reuses a key.
Verify off-device.
```

**Why this is the diff's problem and not purely pre-existing.** The arm order is inherited,
but this diff (a) promoted it to a shared, documented decision, (b) extended it to a second
consent surface that previously had its own copy, and (c) changed the downstream
consequence: before, a seated repeated-seat template still produced an address; now it
never will. The plate is cut under silence for a wallet the device has decided not to
serve. That is precisely the "strand a card that may already be engraved" outcome F-514
weighed — arriving by the route F-531 opened.

**Reachability** is the same as F-531's own: no shipped encoder emits a repeated seat
(`md/dup_seat_fixture_test.go:18-21` says so), so the card comes from another producer —
and the device admits it on the supply path, which is the entire premise of the defect
being fixed.

**Fix shape**: the duplicate sentence is independent of keyed-ness. Prepend it to whichever
arm fires rather than making it a fourth alternative, and give `expandTemplateOnly` in
`gatheredDescriptorFlow` the same treatment the `expandUnsupported` arm just got.

---

## I-3 — Important — the records commit moved §8s and left two sections describing text the device no longer shows

**Where**: `design/SPEC_wallet_policy_composer.md:555`,
`design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md:312`
(and, as a dated observation, `SPEC_wallet_policy_composer.md:1014`).

The diff changed the composer consent's D4 sentence. `gui/composer_consent.go:212-216`
now routes through `noAddressLines`, and the diff's own test edits record the consequence:
`gui/composer_flow_test.go:44` and `:281` were changed from
`"Keyless template - no addresses."` to `"Template has no keys - no addresses."`, because
a composer-built keyless template DECLARES slots and so hits arm 2, not arm 1.

`a832433b` added §8s's new body and moved one corpus row. It did not touch:

* **SPEC §555** — normative, and now false about the surface it describes:
  > "then receive and change addresses 0..1 when seated, or `"Keyless template - no
  > addresses"` (D4)."
  The composer never prints that string for a slot-declaring template any more, and §555
  does not mention the F-531 refusal sentence at all.
* **`IMPLEMENTATION_PLAN_composer_S4_acceptance.md:312`** — step 4 of the S4 acceptance
  walk, whose expected consent text is
  `page 1: "Keyless template - no addresses." / "Verify off-device."`. That walk is a
  **pending gate** (the same file's Task 4 physical plate is "pending"). Run as written it
  reports a mismatch that is not a defect — and the likeliest repair is someone relaxing
  the row, which is how an acceptance gate stops gating.

`SPEC:1014` is a dated "EXECUTED 2026-09-03" observation, so it is a record of what was
true then rather than a false claim; it still deserves a note that the sentence has since
changed, or a future reader will read it as current.

**Verified by**: reading the diff's own test changes (which are the measurement — the
strings were updated because the behaviour changed), and `grep -rn "Keyless template - no
addresses" design/`.

---

## M-1 — Minor — `md/duplicate_keys.go`'s file header now states the opposite of what the code does

`md/duplicate_keys.go:24-31`, untouched by the diff:

> "The device derives a fundable address for that wsh policy today and says nothing
> (F-514) ... It is a WARNING's predicate, never a refusal's. Refusing on-device would
> strand a card that may already be engraved, which is worse than telling the operator
> nothing at all."

Every clause is now false. `complexAddressSource` (`gui/policy_address.go:66`) uses this
exact predicate as a refusal's predicate, the device no longer derives for
`keyed_wsh_timelock_hashlock` (the corpus row moved to `{device: source}` in `a832433b`),
and it no longer says nothing. The author corrected the equivalent paragraph in
`gui/composer_copy.go:336-346` and did not propagate it here — the codec's header is the
document a `md` reader meets first.

**Verified by**: reading both files; confirmed by the corpus baseline move in `a832433b`.

---

## M-2 — Minor — the claim retracted at review I-5 survives in three untouched places

`gui/composer_copy.go:330-334` records the retraction:

> "(An earlier draft of this comment said such a key 'can meet the threshold alone', which
> is false for every k > m and was retracted at review I-5.)"

The retracted sentence is still asserted, unqualified, at:

* `md/duplicate_keys.go:149` — "one key filling two seats of a threshold can meet it alone."
* `md/duplicate_keys_test.go:177` — "since one key filling two seats can meet the threshold alone"
* `gui/composer_flow_test.go:774` — same wording, in a file this diff edited elsewhere

The classic incomplete-propagation shape: the fold corrected the site under review and not
the phrase. `grep -n "meet the threshold alone\|meet it alone"` finds all three in one
command.

---

## M-3 — Minor — the restore document's new sentence is untested

`gui/multisig_restore.go:36-40`, new in this diff:

```go
if repeatsASeat(tpl, keys) {
    lines = append(lines, "Addresses unavailable: this policy seats one key",
        "slot more than once.")
    return lines, false, nil
}
```

**Reproduction (run)**: delete the block so the branch falls through to
`"Addresses unavailable for this policy shape."`, then
`scripts/gui-shard-test.sh ./gui/ 24` → `RESULT: ok -- all 1312 tests ran across 24
shards`. Reverted afterwards.

`TestNoAddressSurfaceDerivesForARepeatedSeatPolicy`
(`gui/duplicate_seat_address_test.go:157-165`) asserts only `hasAddr == false`, which the
generic sentence satisfies. The comment above the block argues the sentence matters
precisely because "this is the document a reader holds in five years" — and nothing holds
it there.

Related, same screen: for the OTHER duplicate class (`DuplicateRefusedByCore`, e.g.
`keyed_wsh_timelock_hashlock` reaching this doc through Engrave Multisig) the restore
document still says only "Addresses unavailable for this policy shape." — it never names
the reuse. The diff gave one of the two classes a cause and not the other.

---

## M-4 — Minor — the new modal body has no row in the class fit gate

`gui/md1_gather.go:204-206` composes a new `showError` body by concatenating two sentences:

```
Check before funding: slot @0 fills more than one seat, so fewer separate keys can
spend this than its k-of-n says. No addresses: this device does not derive them for
a wallet that reuses a key.
```

It is not in `TestModalsThisBlockTouchesAreDrawnInFull`
(`gui/modal_fits_test.go:302`), whose table's own comment says it is "SCOPED TO WHAT THIS
BLOCK TOUCHED" — and this block touched it.

**Measured** (scratch test through the file's own `assertModalBodyFits`/`errorScreenBody`,
deleted after the run):

```
F-531 inspect refusal, FewerKeys:      159 chars drawn in full, headroom 397 (margin 80)
F-531 inspect refusal, RefusedByCore:  145 chars drawn in full, headroom 418 (margin 80)
```

So **no clipping defect today** — but the gate has no row, and the only test that reads
this screen (`gui/duplicate_seat_address_test.go:215`) asserts `want[:40]`, a 40-character
prefix of the FIRST sentence. A future edit that pushes the refusal sentence past the fold
passes both. One table row closes it.

---

## M-5 — Minor — the gate's headline comments and §8s overclaim, and neither points at F-533

`gui/policy_address.go:45`:

> `// F-531: NO ADDRESS FOR A POLICY THAT REUSES A KEY SLOT, on either route.`

and `design/SPEC_wallet_policy_composer.md:857-859`:

> "The device declines to derive an address for a policy that seats one key slot more than
> once -- BIP 388 forbids the shape ..."

Both are false as written. The gate's predicate is `md.DuplicateKeySlot`, which the codec's
own header says answers **Core's** question, not BIP 388's. `keyed_tr_multi_a` and
`keyed_tr_sortedmulti_a` are `tr(@0/…, multi_a(2,@0/…,@1/…))` — the same slot at the
internal key AND inside the leaf, same use-site — and the device still derives addresses
for both. Confirmed in `design/policy-corpus-baseline.json`: two rows remain
`{device: "ok", rust: "refused"}` after this diff.

The author filed this correctly as **F-533** (`design/FOLLOWUPS.md`), which is why this is
Minor and not Important — the gap is known and scoped. What is missing is a pointer: the
code comment and §8s both state the wide claim with nothing next to them saying it is
narrower in practice, so the next reader inherits the overclaim rather than the follow-up.

---

## N-1 — Nit — `assertShowsNoAddress` can fail for an unrelated reason

`gui/composer_flow_test.go:745-755` greps the joined line set for the literal `"bc1"`. The
same line sets carry 32-hex wallet ids (`Policy-ID: a6bece8ae1…`). A future fixture whose
id happens to contain `bc1` fails the assertion with an address-shaped accusation about a
hex string. Anchoring on `"bc1q"`/`"bc1p"` or on the label rows alone removes it.

## N-2 — Nit — the inspect refusal names key reuse as THE cause even when it is not the only one

`gui/md1_gather.go:204`. `complexAddressSource`'s gate short-circuits above the deriver, so
a card that both duplicates a slot AND carries a shape the emitter could not derive anyway
(a hardened wildcard, an exotic range — `useSiteToChildren` `!ok`) still gets the
duplicate sentence as its explanation. Harmless, but the sentence is more specific than the
evidence.

---

# Counts

**3 Critical → 0. 3 Important. 5 Minor. 2 Nit.**

`0 Critical / 3 Important / 5 Minor / 2 Nit`

# Verdict

**NOT GREEN.**

The funds defect itself is closed and closed well: `repeatsASeat` is exact, both address
routes refuse, I could not find a fourth path, and the Core measurement is kept alive
beneath the gate rather than deleted by it. What blocks is the surrounding work —
a gate that can no longer fail while asserting in writing that it can (I-1), a consent
screen that goes silent for the keyless form of the very shape being refused (I-2), and
two records that still describe the old sentence, one of them a pending acceptance gate
(I-3).

# Reproduction index

Every measurement above, in the order I ran it. Tree was clean before and after; all
scratch files deleted, all mutations reverted with `git checkout --`; `gofmt -l .` returns
the known five-file baseline.

1. `expandOK ⟹ DuplicateNone` over the corpus: 54 vectors, 44 derive, 0 duplicate.
2. Triple mutation (delete all three F-514 producers) → `go build` ok →
   `scripts/gui-shard-test.sh ./gui/ 24` → `all 1312 tests ran`, 0 failures.
3. Keyless repeated-seat template generated from `dupSeatDescriptor` with the xpub TLV
   cleared → `md1fgefcqqpq2tvyyy4qqxppcq3p0px74klwllmkl4ns5uz3v66nj2ecs` → run through
   `noAddressLines`, `walletPolicyConsentLines`, `composerConsentLinesFor`,
   `policyAddressAt`, `multisigRestoreLines`.
4. Restore-doc mutation (delete the `repeatsASeat` arm) → `all 1312 tests ran`, 0 failures.
5. Modal fit of the new `showError` body through `assertModalBodyFits`: headroom 397/418.
6. `TestTheTwoAddressRoutesNeverDisagree` compares 3 of 69 sources (66 skip); the
   `compared == 0` guard is real, so this is a scope note rather than a finding.
