# R0 round 1 — PLAN_F533_bip388_reuse_predicate.md

**Verdict: 2 Critical / 3 Important / 3 Minor / 1 Nit — NOT GREEN.**

Reviewed read-only at fork `e4ab97d` (tip `e4ab97d`, tree clean), primary
`descriptor-mnemonic` at its current checkout, plan+followups in
`mnemonic-engrave`. One scratch Go module was built under `/tmp/f533probe`
(outside every repo) to construct the counterexample in C1; nothing in any repo
was edited.

---

## 1. The load-bearing question — answered NO, with evidence

> Can a policy be **Core-duplicate but BIP-388-legal** — one key slot at several
> use sites with DISJOINT multipath sets (`@0/<0;1>` and `@0/<2;3>`) — expressed
> in the md1 wire format at all?

**No. That shape is not expressible in md1, and the set is empty rather than
merely small.** Six independent pieces of evidence, four of them from the wire
format itself:

1. **A use-site is a property of the SLOT, not of the occurrence.** `descriptor`
   (`/scratch/code/shibboleth/seedhammer/md/md.go:817-823`) carries exactly one
   `useSite useSitePath` — the shared baseline — plus per-index overrides in the
   TLV section (`tlvSection.useSiteOverrides []idxUseSite`, `md/md.go:523-525`).
   `idxUseSite` is `{idx uint8; path useSitePath}` (`md/md.go:261-264`): one path
   per index.

2. **A tree occurrence carries an index and nothing else.** The only bodies that
   reference a key are `keyArgBody{index uint8}` (`md/md.go:119`),
   `multiKeysBody{k uint8; indices []uint8}` (`md/md.go:110-113`) and
   `trBody{isNums bool; keyIndex uint8; tree *node}` (`md/md.go:114-118`). There
   is no per-occurrence suffix, no per-occurrence multipath, and no selector that
   could pick between two use-sites for one index.

3. **The wire forbids two override entries for one slot.** `readSparseTLVIdx`
   (`md/md.go:657-671`) range-checks `idx < n` and then rejects `idx <= last`
   with `errOverrideOrder`, so the use-site override list is strictly ascending
   by index: **at most one override per slot, enforced at decode.**

4. **Resolution is a function of the index alone.** `resolveUseSite(d, idx)`
   (`md/expand.go:166-186`) returns the first override matching `idx` and
   `break`s, else the shared baseline. `ExpandWalletPolicy` calls it once per
   `idx` in `0..n` (`md/expand.go:89-101`), producing exactly one `UseSite` per
   `@N`. The encoder mirrors this — `writeTLV` emits one `useSitePath` per
   override entry (`md/encode.go:250-268`) and `encode_multisig.go:166` /
   `encode_singlesig.go:62` set a single shared `<0;1>/*`.

5. **The Rust primary has already recorded the same conclusion, twice, in
   shipped code.** `descriptor-mnemonic/crates/md-cli/src/parse/reuse.rs:93-95`:
   `Finding::MultipathDisjoint` — *"The wallet is BIP-388-LEGAL; md1 cannot
   express it (F-417)"*. And `reuse.rs:307-313` (the `MultipathOverlap`
   message): *"md1 could not carry two use sites for one key slot in any case
   (one path per key slot, F-417)"*.

6. **It is a standing operator ruling, not an accident** — F-417, "md1 narrow
   paths are deliberate: never widen the wire format".

### What follows, and it is stronger than "step 3 is unnecessary"

Because one slot has exactly one use-site, **two occurrences of one slot always
carry the SAME multipath set.** A set is never disjoint from itself, so BIP
388's disjointness rule — quoted verbatim from `bip-0388.mediawiki` and
re-fetched for this review:

> "If two KEY are KP/<M;N>/* and KP/<P;Q>/* for the same key placeholder KP,
> then the sets {M, N} and {P, Q} must be disjoint."

is violated by **every** md1 payload in which a slot occurs twice. Therefore:

```
{ payloads where md.DuplicateKeySlot fires }  ⊊  { payloads where a slot repeats }
                                              =  { payloads BIP 388 forbids by disjointness }
```

`DuplicateKeySlot` fires only on a repeat *inside one expression*; a whole-tree
count is a strict superset of that in every arm (`tagTr` → `duplicateInTapTree`
scopes per leaf and skips the internal key; `tagWsh`/`tagSh` → `duplicateInExpression`
on the unwrapped inner; default → the same). **In md1 the two rules ARE nested,
in the direction the plan says they are not.** The plan's §"THE TWO RULES ARE
NOT NESTED" is true about BIP 388 vs Core in general and **false for the wire
format this device reads**.

Consequences the plan must fold:

* **Step 3 is dead weight.** The address branch of `walletPolicyAddressLines`
  cannot be reached with a duplicate of any kind once the refusal sits on the
  wider predicate, so the deleted block stays deleted. The plan should say so
  with this evidence, and the comment at `gui/wallet_policy.go:337-348` should be
  updated to record that the case is empty in md1 rather than merely unreachable
  today.
* **The plan's core design — one predicate for the warning, another for the
  refusal — loses its justification.** Nothing is warned-about-but-not-refused,
  so keeping two predicates buys nothing and costs the silence in I1 below.
  One predicate with a widened `DuplicateKind` is the shape the nesting argues
  for.

**Caveat, and it is I2 below:** the *other* BIP 388 reuse rule — pairwise
distinctness of the key information vector — reaches a shape md1 CAN express
(one xpub at two different slots with different use-sites). That is a different
question from the one asked, and the plan's proposed predicate cannot see it.

---

## 2. Findings

### C1 — the predicate as specified falsely refuses every NUMS-internal-key taproot wallet

**What is wrong.** Step 1 specifies `BIP388ReusedSlot` as counting key-slot
occurrences "over the WHOLE expression, **including `trBody.keyIndex`**". It
does not say *conditionally*. But `keyIndex` is only meaningful when
`isNums == false`: the decoder reads it only in that case and otherwise leaves
the zero value —

```go
// md/md.go:432-457
case tagTr:
        isNums, err := r.readBool()
        ...
        var keyIndex uint8
        if !isNums {
                idx, err := r.read(int(kiw))
                ...
                keyIndex = uint8(idx)
        }
        ...
        b = trBody{isNums: isNums, keyIndex: keyIndex, tree: sub}
```

— and the composer writes it as a literal zero for a NUMS tree
(`md/compose.go:811`: `trBody{isNums: ik < 0, keyIndex: 0, tree: spine}`). So an
unguarded count charges slot **@0** an extra occurrence for every NUMS taproot,
and `validatePlaceholderUsage` guarantees `@0` is also referenced somewhere in
the tree. The predicate reports `(0, true)` for an ordinary, BIP-388-legal,
Core-accepted taproot script wallet, and the refusal denies it an address on
both routes.

**Reproduction (constructed, not reasoned).** A probe built against the fork's
exported `md.TapLeavesChunks` (which returns `internalKeyIndex, isNUMS, leaves`)
simulates the plan's predicate over every vendored vector, once as worded and
once with a `!isNums` guard:

```
vector                                isNUMS  ik   core            plan-as-worded   with !isNums
compose_tr_thirty_two_slots           true    @0   DuplicateNone   dup @0           none
keyed_compose_tr_sole_sortedmulti_a   true    @0   DuplicateNone   dup @0           none
keyed_compose_tr_unsorted_sole_leaf   true    @0   DuplicateNone   dup @0           none
keyed_tr_multi_a                      false   @0   DuplicateNone   dup @0           dup @0
keyed_tr_sortedmulti_a                false   @0   DuplicateNone   dup @0           dup @0
keyed_tr_depth2 / _rightspine         false   @0   DuplicateNone   none             none
keyed_tr_with_leaf / keyless_…        false   @0   DuplicateNone   none             none
gap_tr_leaf_pkh                       false   @0   DuplicateNone   none             none
```

`keyed_compose_tr_sole_sortedmulti_a` and `keyed_compose_tr_unsorted_sole_leaf`
are `{"device": "ok", "rust": "agrees"}` in
`design/policy-corpus-baseline.json` today. The plan as written moves both to
`{"device": "source"}` — the device refusing to derive an address for a wallet
the primary and Core both accept.

**Why it matters.** This is address denial on a funds-adjacent surface for legal
wallets, and taproot-with-a-NUMS-internal-key is the flagship shape the composer
mints (`md/compose.go:811`). Note the guard is an established pattern the plan
simply omitted: `md/md.go:957`, `md/md.go:1282`, `md/canonicalize.go:111,147,221`
and `md/policy_shape.go:116` all gate on `isNums` before touching `keyIndex`.

**Remedy.** Step 1 must read "including `trBody.keyIndex` **when
`!trBody.isNums`**", and the acceptance must carry a second mutation row:
*drop the `!isNums` guard → the three NUMS vectors above stop deriving and the
corpus check goes red.*

---

### C2 — step 2 names a seam that is not the one both address routes pass through; the corpus gate cannot see the difference

**What is wrong.** Step 2 says *"`policyAddressAt` refuses on
`BIP388ReusedSlot`"*. The F-531 refusal is not in `policyAddressAt`. It is in
`complexAddressSource` (`gui/policy_address.go:76`), and it is there
deliberately, because **two different call sites route between the flat and the
complex address routes**:

* `policyAddressAt` (`gui/wallet_policy.go:373-383`) — the consent/inspect-lines
  router, also the export `gui.PolicyAddressAt` used by `cmd/policyprobe`.
* `gatheredDescriptorFlow` (`gui/md1_gather.go:163-211`) — the gather-completion
  handler, which does the same routing *as control flow* and calls
  `complexAddressSource(collected, keys)` **directly at `gui/md1_gather.go:205`**,
  never through `policyAddressAt`.

**Reproduction.** Implement step 2 literally — gate in `policyAddressAt`, gate
removed from `complexAddressSource` — and feed the device `keyed_tr_multi_a`:

1. `gatheredDescriptorFlow` calls `duplicateRefusalBody` (`gui/md1_gather.go:184,
   215-232`), which asks `md.DuplicateKeySlotChunks`. The vector's kind is
   `DuplicateNone` — that is the entire defect F-533 exists for — so no modal,
   no return.
2. `expandedToDescriptor` returns `expandUnsupported` (`scriptForTemplate`
   admits only `PolicySingle` and `PolicySortedMulti`, `gui/md1_expand.go:158-186`).
3. The default arm calls `complexAddressSource(collected, keys)` — ungated now —
   and `md1PolicyFlow` renders Receive/Change addresses.

The Inspect-descriptor route derives an address for a wallet the consent route
refuses. **That is verbatim the F-531 title: "two address routes in one binary
disagree for a repeated-slot multisig" — reopened by its own remedy.**

**And the plan's own gate would report GREEN on that build.** `cmd/policyprobe`
wraps `gui.PolicyAddressAt` → `policyAddressAt` and nothing else
(`gui/policy_address_export.go:31-33`), by an explicit decision recorded in that
file's comment. So `scripts/policy-generate.py --corpus` would print
`ok/refused 0`, acceptance bullet 1 would pass, and the divergent route would be
invisible to the only automated check the plan names.

**Remedy.** Say explicitly that the predicate is swapped **at the existing
gate inside `complexAddressSource`**, which both routers pass through, and
enumerate all three sites currently bound to `md.DuplicateKeySlot` so the
implementer changes them deliberately rather than by inference:

| site | today | after |
| --- | --- | --- |
| `gui/policy_address.go:76` (the F-531 refusal) | `md.DuplicateKeySlotChunks` | the BIP-388 predicate |
| `gui/md1_gather.go:222` (`duplicateRefusalBody`, the inspect modal) | `md.DuplicateKeySlotChunks` | see I1 |
| `gui/wallet_policy.go:306` (`noAddressLines`, the F-514 sentence) | `md.DuplicateKeySlotChunks` | see I1 |

If the refusal genuinely should move up to `policyAddressAt`, then
`gui/md1_gather.go:205` must be changed to route through it too — but that is a
larger change than the plan describes and it is not what the plan says.

---

### I1 — every card the new refusal stops gets a message that names a device limitation instead of the wallet

**What is wrong.** Step 2 pins the sentence to the narrower predicate
(*"`md.DuplicateKeySlot` keeps the F-514 WARNING"*). The two vectors this plan
exists to refuse are `DuplicateNone` under that predicate, so after the change
they are refused **in silence**:

* Consent route: `policyAddressAt` returns `!ok` → `noAddressLines`
  (`gui/wallet_policy.go:300-320`) asks `md.DuplicateKeySlotChunks`, gets
  `DuplicateNone`, sets `dup = false`, and falls through to the final arm —
  **"This device can't derive addresses for this policy."**
* Inspect route: `duplicateRefusalBody` returns `!ok`, `complexAddressSource`
  returns `!ok`, and the default arm prints **"Complex policy - display only."**
  (`gui/md1_gather.go:209`).

Both sentences are already recorded in this repo as the defect they reintroduce:

* `composerCopyNoAddressesDuplicateKeys`'s doc comment
  (`gui/composer_copy.go:468-476`): *"'This device can't derive addresses for
  this policy' is true of the shape and useless about it: it reads as a device
  limitation, and the operator goes looking for a better tool instead of
  learning that their wallet reuses a key."*
* `gatheredDescriptorFlow`'s comment (`gui/md1_gather.go:181-183`): *"AND IT
  REPLACES 'Complex policy - display only' for these cards, which was the wrong
  sentence twice over … the operator would read a device limitation where there
  is a fact about their wallet."*

Step 4 ("the refusal sentence must say BIP 388") is in direct tension with step
2 ("the warning stays on `md.DuplicateKeySlot`"): step 4 rewrites a sentence
that step 2 guarantees will not fire for the newly refused cards.

**Why it matters.** The consent screen is one confirm from steel. An operator
told "this device can't derive addresses for this policy" concludes the device
is limited and engraves a BIP-388-forbidden wallet anyway — `walletPolicyAddressLines`
returns lines, it does not abort, and the `md.DuplicateKeySlots` abort at
`gui/wallet_policy.go:254` does not fire for a single-slot repeat.

**Remedy.** Since §1 establishes the two predicate sets are nested in md1, drive
**both** the refusal and the sentence from the BIP-388 predicate, and give
`DuplicateKind` a third arm for the delta (BIP-388-forbidden, Core-accepted:
"one key at the taproot internal key and in a leaf"). `md/duplicate_keys.go`'s
kind taxonomy already documents that the kinds are named for the sentence they
produce — a third sentence is the intended extension point, not a new mechanism.

---

### I2 — step 1 claims to port `KeyAtDisjointUseSites`; the proposed signature structurally cannot compute it, and that shape IS expressible in md1

**What is wrong.** Step 1 says the predicate is *"Ported from the primary's
taxonomy (`dm crates/md-cli/src/parse/reuse.rs`, `Finding::SamePathExpression`
and `KeyAtDisjointUseSites`)"*. `BIP388ReusedSlot(tree node)` takes the tree and
nothing else. `KeyAtDisjointUseSites` is Family 2 in the primary and is computed
from **key material**: `classify(occs: &[PlaceholderOccurrence], keys: &[ParsedKey])`
compares `a.payload != b.payload` across *different* placeholder indices
(`reuse.rs:220-256`). A tree-walking slot counter cannot see xpubs. The plan
names a finding its own signature cannot implement.

**And the shape is reachable.** One xpub at two slots with different use-sites:

* it decodes — `decodePayloadValidated` (`md/md.go:1133-1160`) runs exactly five
  validators (`validatePlaceholderUsage`, `validateMultipathConsistency`,
  `validateTapScriptTree`, `validateExplicitOriginRequired`, `validateXpubBytes`)
  and none compares xpubs across slots; `validateMultipathConsistency` only
  requires equal alt-*counts*, which `<0;1>` and `<2;3>` satisfy;
* `md.DuplicateKeySlots` deliberately passes it — `sameUseSite` is false
  (`md/expand.go:257-290`), and its doc argues the disjoint form is "two wallets,
  not a duplicate";
* `repeatsASeat` does not fire (`tpl.M == len(keys)`, `gui/md1_expand.go:92-108`);
* `descriptorRepeatsAKey` does not fire either — `address.DerivesSameKey`
  (`address/address.go:264-287`) resolves each child and returns false when the
  indices differ;
* so it reaches `expandOK` and **derives an address today**.

BIP 388's *other* reuse rule forbids it, verbatim: *"The public keys obtained by
deserializing elements of the key information vector must be pairwise
distinct"*, with the security note *"Reusing pubkeys could be insecure in the
context of wallet policies containing miniscript."* The primary refuses it —
`crates/md-cli/tests/duplicate_key_slots.rs:339`
(`t_row_one_key_at_two_disjoint_use_sites_refuses_as_the_r_n1d_delta`) asserts
exit 1 and the message *"@0 and @1 were given the SAME extended public key at
DIFFERENT use sites"*.

**Why it matters.** The plan would ship claiming convergence with the primary's
taxonomy while leaving an expressible, BIP-388-forbidden, address-deriving shape
unrefused — and the claim is the kind that stops the next reviewer from looking.

**Remedy.** Either (a) widen the entry point to see both halves —
`BIP388Reused(tree node, keys []ExpandedKey)` / a `…Chunks` form, mirroring the
primary's "two entrances, one classifier" — or (b) drop the
`KeyAtDisjointUseSites` citation from step 1, state that only the disjointness
rule is being ported, and file the pairwise-distinctness delta as its own
follow-up with the reproduction above. Do not leave the claim as written.

---

### I3 — the acceptance cannot fail in the over-refusal direction, which is how C1 would have shipped

**What is wrong.** Acceptance bullet 1 is *"The corpus check reports
`ok/refused 0`, with the baseline rewritten in the same commit and each verdict
hand-checked (the script demands this)."* Three problems, compounding:

1. **`ok/refused 0` measures one direction only.** When the device refuses, the
   probe reports stage `source` and `run_corpus` records `{"device": "source"}`
   (`scripts/policy-generate.py:296-328`). A *wrongly* refused vector therefore
   lands in the `source` bucket alongside the 22 legitimate ones, and
   `ok/refused` still reads 0. C1's two false refusals pass this bullet.
2. **"the baseline rewritten in the same commit" erases the only signal that
   would have caught it.** The `MOVED` diff is the check; `--write-baseline`
   overwrites `observed` wholesale and returns 0 before any comparison runs
   (`scripts/policy-generate.py:513-517`).
3. **"the script demands this" is false.** Nothing in the script enforces a
   hand-check. `--write-baseline`'s help text says *"Deliberate, never
   automatic"* and the missing-baseline error says *"once you have checked the
   verdicts by hand"* — both are instructions to a human, not enforcement. The
   parenthetical asserts a gate that does not exist.

**Remedy.** State the whole expected distribution, derived rather than
described. Current baseline, machine-counted from
`design/policy-corpus-baseline.json`: 70 vectors = `ok/agrees 45` +
`ok/refused 2` + `source 22` + `expand 1`. After a correct fix the only movement
is the two target vectors, so the acceptance should read:

```
corpus: 70 vectors
  expand           1
  ok/agrees       45      (unchanged)
  ok/refused       0      (was 2)
  source          24      (was 22)
MOVED exactly: keyed_tr_multi_a, keyed_tr_sortedmulti_a  — and no others
```

and the baseline rewrite must be the *second* run, after a diffing run has
printed exactly those two `MOVED` lines and nothing else.

---

### M1 — `BIP388ReusedSlot(tree node)` is not callable from package `gui`

`node` is unexported (`md/md.go:131-134`), so an exported function taking one
cannot be called from `gui`, where step 2 places the refusal. The shipped
sibling solves this with `DuplicateKeySlotChunks(strs []string)`
(`md/duplicate_keys.go:172-180`), and every `gui` call site uses that form. The
plan needs the `…Chunks` wrapper named in step 1 alongside the core predicate.

### M2 — the mutation step covers one term of the predicate, not the change

*"disable the internal-key term → the two vectors derive again"* exercises only
the `trBody.keyIndex` term. The other half of the widening is dropping
`duplicateInTapTree`'s **per-leaf scoping**, which today makes one slot in two
different leaves invisible (`md/duplicate_keys.go:183-197`). Restoring per-leaf
scoping would leave the two target vectors still refused (their repeat spans the
internal key, not two leaves), so that mutation would report a false PASS
against the corpus — there is no cross-leaf reuse vector in the vendored set.
Add a hand-built `md/` fixture for `tr(NUMS, {pk(@0), pk(@0)})` (decodable: all
five validators pass) and a mutation row against it, plus the `!isNums` mutation
from C1.

### M3 — the F-529 mitigation is stated as a choice and never made

*"Sequence F-529 against this before implementing, or pin the two vectors
locally"* leaves the decision open in a section titled "Noted, not resolved". The
risk is real and specific: F-529 records that a re-vendor replaces
`keyed_tr_multi_a` (fork: 2 keys, tid `f15f2969`) and `keyed_tr_sortedmulti_a`
(fork: 2 keys, tid `87212fb6`) with reuse-free upstream policies of the same
name, after which `ok/refused 0` is vacuously true and the plan's entire
acceptance evidence is gone. The plan should decide it: **the new predicate's
durable evidence is hand-built `md/` unit fixtures** (the precedent is
`md/duplicate_keys_test.go`, which builds trees directly), with the corpus check
as a cross-implementation confirmation rather than the sole gate. F-529's
observation that F-514's tests assert the reuse property before relying on it —
so a re-vendor fails loudly — is worth citing in the plan as the existing guard.

### N1 — citation drift

`md/expand.go:33-42` is cited for "`UseSite.HasMultipath` + `[]UseSiteAlt`". That
range covers `UseSiteAlt` (36-39) only; `type UseSite` is at 45-49. Cite
`md/expand.go:33-49`.

---

## 3. Verified SOUND — do not re-check these next round

* **The located cause is correct.** `trBody` is at `md/md.go:114-118` exactly as
  cited, with `keyIndex` a sibling of `tree`. `DuplicateKeySlot`
  (`md/duplicate_keys.go:92`) dispatches `tagTr` to `duplicateInTapTree(*b.tree)`
  — it is handed the taptree, never the `trBody` — and `countKeySlots`
  (`md/duplicate_keys.go:210-228`) has cases for `keyArgBody`, `multiKeysBody`,
  `childrenBody` and `variableBody` and **no `trBody` case**. No path reaches
  `keyIndex`. Measured through the probe: `keyed_tr_multi_a` and
  `keyed_tr_sortedmulti_a` both report `DuplicateNone` with `isNUMS=false,
  ik=@0`. **Verified.**
* **The predicate does not OVER-refuse against BIP 388** (once C1's guard is
  added). Per §1, two occurrences of one md1 slot always share one use-site, so
  their multipath sets are identical and never disjoint — every such payload is
  forbidden by the disjointness rule as quoted. No false refusal from the
  disjointness direction is possible in this wire format.
* **BIP 388's text is as the primary quotes it.** Re-fetched from
  `bitcoin/bips` master `bip-0388.mediawiki` for this review: the
  pairwise-distinctness rule, the disjointness rule, the miniscript-reuse
  security note, and the invalid-example rows `sh(multi(1,@0/**,@0/**))`
  ("Repeated keys with the same path expression") and
  `sh(multi(1,@0/<0;1>/*,@0/<1;2>/*))` ("Non-disjoint multipath expressions").
  `crates/md-cli/src/bip388.rs`'s two constants match. No drift.
* **Step 4's premise is right.** `composerCopyDuplicateKeys`
  (`gui/composer_copy.go:427-447`) has two arms, one of which says "Bitcoin Core
  refuses such a descriptor" — a Core verdict, in Core's voice, as the plan says.
* **The settled facts hold.** `md.DuplicateKeySlot` answering Core's question is
  deliberate and correct for the warning; the two divergent vectors are the ones
  named; the Rust primary already refuses them; this is convergence. Nothing in
  this review proposes a Rust change. Re-confirmed, not re-derived.
* **The baseline counts are machine-read, not described:** 70 vectors,
  `ok/agrees 45`, `ok/refused 2` (`keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`),
  `source 22`, `expand 1`.

---

## 4. Verdict

**NOT GREEN — 2 Critical / 3 Important / 3 Minor / 1 Nit.**

C1 and C2 are both "the plan as written produces a wrong outcome that its own
acceptance reports as GREEN", which is why I3 is graded Important rather than
Minor: the gate's blind spot is what makes the two Criticals shippable.
