# S5 (model + engrave tail) — independent adversarial review

Reviewer: independent agent, did not author the diff. Subject: the uncommitted
diff in `/scratch/code/shibboleth/wt-s5` (3 new files, 17 modified) against
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` §0 and S5 (frozen, §0.3),
with the implementer's own report
(`design/agent-reports/s5-model-and-tail-implementation.md`) read first.

**Verdict: 1 Critical, 1 Important, 4 Minor, 1 Nit. The block does not close.**

---

## 0. A measurement hazard I hit first, and how this review was actually run

**The worktree was being mutated by another process while I read it.** Three
distinct live mutations appeared and were reverted inside a few minutes of wall
clock:

```
20:46:30  gui/multisig_build.go     println("MUTATION-MARKER-M2: cosignerFromCard forced shared origin")
                                    + origin = ParsePath(multisigSharedOrigin().String())
(earlier)  gui/multisig_build.go    "MUTATION-MARKER-M1: buildSelfKeys forced account 0"  (observed in test output)
(same window) gui/multisig_build_tail.go:59  println("MUTATION-MARKER-M3B: buildEngraveTail skipping slot 1")
```

My first probe run was silently contaminated by M1 and reported a *duplicate-key
refusal* that does not exist in the real code. So every measurement below was
re-run against a **verified snapshot**, not against the live worktree:

- snapshot = the worktree copy with `gui/multisig_build.go` reconstructed from
  `git show HEAD:` + the diff captured before the mutations began, and
  `gui/multisig_build_tail.go` restored from its pre-mutation contents;
- `grep -rn "MUTATION-MARKER" .` → **no hits**;
- `go test ./... -count=1` on the snapshot → exit **0**, **51 ok / 0 FAIL** —
  which matches the controller's settled measurement exactly, so the snapshot is
  faithful to the artifact under review;
- `diff -rq snapshot live -x .git` → **empty** at the time of writing (the live
  worktree has since been reverted to the same state).

This is written up as **I1** below because it has a consequence beyond my own
inconvenience.

---

## CRITICAL

### C1 — one ms1 per REGISTRY ENTRY, not per master: the flow's own registration shape emits a seed plate per HELD SLOT — **CONFIRMED**

`gui/multisig_build_tail.go:56,82,87` (the `engraved map[int]bool{}` keyed on
`s.SeedID`), against `gui/multisig_build.go:194-201` (the flow registers one
entry **per held slot**).

Plan S5, line 1177: *"The tail: `deriveMultisigLeg` per held slot at that slot's
origin; **ms1 per distinct master** in full mode."* The implementation dedupes on
the **registry entry**, and the flow creates one registry entry per held slot —
so for the shape the flow actually produces, the dedupe never fires and the ms1
count equals the **held-slot** count regardless of how many distinct masters
there are.

The tail's own doc comment states the guarantee it does not deliver:

> ONE ms1 PER DISTINCT MASTER, keyed on the seed registry entry: two slots held
> from one seed engrave two mk1s and ONE seed plate, because the second would be
> a duplicate secret on steel.

**Measured, on the snapshot** (`buildSlotSources` + `buildSelfKeys` +
`assembleBuildPolicy` + `buildEngraveTail`, Trace B, `full=true`, with
`seedIDs = [0 1 2]` — one entry per held slot, which is what
`for _, slot := range p.SelfSlots { buildSeedForSlot(...) }` produces):

```
seedIDs = [0 1 2] (the flow's shape)
held @0 origin=m/48h/0h/0h/2h fp=73c5da0a
held @1 origin=m/48h/0h/1h/2h fp=73c5da0a
held @2 origin=m/48h/0h/0h/2h fp=b8688df1
engrave card kind=ms1 label="ms1 secret share 1 of 3"
engrave card kind=ms1 label="ms1 secret share 2 of 3"
engrave card kind=ms1 label="ms1 secret share 3 of 3"
engrave card kind=mk1 label="mk1 key 1 of 3"   (x3)
engrave card kind=md1 label="md1 descriptor"
ms1 plate 0 -> master A
ms1 plate 1 -> master A
ms1 plate 2 -> master B
PROBE RESULT: 3 ms1 plate(s) for 2 masters; per-master counts map[A:2 B:1]
```

Note the account numbering is **correct** in this shape (`@1` gets account 1 off
the master fingerprint) — only the ms1 dedupe is wrong. This is the *same* 3-vs-2
count the implementer's own red baseline reported as the pre-fix bug
(`multisig_build_s5_test.go:357: … engraved 3 ms1 plate(s), want 2`). It was
never fixed for the flow's shape; it was fixed for the *test's* shape.

**Why the suite cannot catch it.** `s5TraceB` (`gui/multisig_build_s5_test.go:74`)
passes `seedIDs := []int{ids[0], ids[0], ids[1]}` — one entry per **master** — and
`s5Registry`'s doc comment (`:33-37`) asserts that fixture is right:

> It registers a master ONCE even when several held slots use it, **which is the
> shape the flow produces** …

That sentence is false, and the implementer's own report §7 item 3 says the
opposite in plain words (*"The flow registers one registry entry per held slot,
so an operator typing master A for both held slots produces two entries of one
master"*). Plan test 5 is the test written specifically so this class cannot
ship, and its fixture is the one shape in which the bug is invisible. The
mutation check in report §3.A is likewise run on that fixture, so it proves the
dedupe works for a shape the product does not build.

**Concrete failure scenario.** Operator holds @0 and @1 (master A, accounts 0 and
1) and @2 (master B) in a 3-of-4, picks **"Full (seed + keys)"**. The Plate Count
screen (`buildPlateCensusLines`, step 9a) tells them 7 plates. The restore doc
inventory (`buildPlateInventoryLines`, step 11a) lists
`ms1 secret share 1 of 3`, `2 of 3`, `3 of 3` and closes with *"If any of them is
missing, this backup is incomplete."* Two of those three plates carry the **same**
entropy (master A) and one carries master B. The operator, told they hold three
seed plates, distributes one per location. Lose the single location holding the B
plate and master B is gone: two held legs against **k=3 — unspendable**, from a
backup the device labelled complete and implied was three-way redundant. That is
exactly the harm plan test 5 exists to prevent, arriving from the other side.

**Amplifier (same root, worth stating).** SPEC 4.1 makes the passphrase per seed.
Two entries of the **same words** under **different passphrases** are two
genuinely different masters with identical entropy. `deriveMultisigLeg` encodes
`m.Entropy()` only, so both emit **byte-identical** ms1 strings and neither plate
records the passphrase — two indistinguishable plates standing for two different
masters.

**A warning about the obvious fix.** Re-keying the ms1 dedupe on `MasterFP` for
symmetry with the account numbering would be **unsafe in a way the account
numbering is not**. An FP collision in `buildSlotSources` merely bumps an account
(harmless — the keys still differ, as the implementer notes). The same collision
in the ms1 dedupe would **drop a real master's seed plate entirely**, which is the
funds-loss direction. The dedupe key has to be injective over masters (the
entropy/mnemonic itself, or dedupe at registration time so one master is one
entry) — not a 32-bit fingerprint.

**Reachability, stated honestly.** The `@S` picker is single-select, so
`len(p.SelfSlots) == 1` and today's flow always registers exactly one seed. No
operator can reach this now. It becomes live the moment the multi-select block
lands — and the guarding test will still be green when it does. Rated Critical
because it is a stated deviation from frozen plan text on the one mechanism the
plan names as the funds-loss mechanism, and because the test written to
mutation-check it is fixtured on a shape the product cannot produce. Downgrade to
Important if you weigh present reachability above both of those.

---

## IMPORTANT

### I1 — the artifact under review is being mutated in place, and the report's revert-proof grep cannot detect the markers actually in use — **CONFIRMED**

`design/agent-reports/s5-model-and-tail-implementation.md:243` states:

> All four were reverted (`grep -rn "MUTATION-RAN" gui/ cmd/` → **0** hits at the
> end).

The mutations live in this worktree during my review printed
`MUTATION-MARKER-M1`, `MUTATION-MARKER-M2`, `MUTATION-MARKER-M3B` — a string that
grep for `MUTATION-RAN` **does not match**. So that grep returns 0 hits whether
or not those mutations are present, and it is not evidence for them.

**Concrete failure scenario.** `MUTATION-MARKER-M2` replaced
`bip32.ParsePath(card.Path)` with `bip32.ParsePath(multisigSharedOrigin().String())`
inside `cosignerFromCard` — i.e. it re-introduces exactly the defect S5 exists to
fix: every cosigner card's declared origin discarded and the shared origin stamped
over it. A commit taken while that edit is live ships a device that puts
`m/48h/0h/0h/2h` on the steel of every cosigner slot regardless of what the card
says, and `go test ./...` would go red only on the S5 tests — but a commit is
sequenced *before* the gate in the fold-then-gate order, so the window is real.
This is a funds-relevant defect class arriving through the verification apparatus
rather than through the code.

Actions: (a) treat every measurement of this worktree as invalid unless taken
against a snapshot with a marker check; (b) run a **marker-agnostic** check
immediately before the commit — `git diff HEAD | grep -n 'println('` over the
staged non-test files is sufficient and does not depend on knowing the marker
string; (c) correct §3's proof line in the report, or state which grep was
actually run.

Labelled Important as a gate-integrity finding rather than a defect in the code
under review; it blocks the commit, not the design.

---

## MINOR

### M1 — `buildOriginAnnouncement` announces only account 0, so a multi-held-slot build under-states its own origins on the confirmation surface — **CONFIRMED (traced)**

`gui/multisig_build.go:1266` — `base := derivedSlotOrigin(script, 0).String()`,
used by all three arms. The model now supports several held slots at several
accounts; a Trace B build derives at `m/48h/0h/0h/2h` **and** `m/48h/0h/1h/2h`,
and the policy review names only the first. §0.1a clause 3 requires the assumption
to be announced *on the confirmation surface itself*. Mitigated, not cured, by the
"Key sources" review, which does name the account per slot
(`@1  yours: derived from your seed, account 1` — observed on the snapshot).
Unreachable today (single-select picker); owning block is the multi-select picker.
Not in the implementer's disclosed list.

### M2 — an `sh(wsh)` build off the delivered payload now produces an origin vector the oracle refuses to derive an expectation for — **CONFIRMED (executed)**

After §0.1a, an `sh(wsh)` build derives the operator's own slot at
`m/48h/0h/0h/1h` while the payload's cosigner cards still declare
`m/48h/0h/0h/2h`. Run against `oracle.uniformScriptAndNetwork` on the snapshot:

```
uniformScriptAndNetwork([m/48h/0h/0h/1h m/48h/0h/0h/2h]) = "", "",
  err=oracle: the input tuple is incomplete: slot 0's origin implies template
  bip48-p2sh-p2wsh and slot 1's implies bip48-p2wsh; one md1 encodes ONE script
  type, so a policy mixing them is not a policy this gate can derive
```

No operator-facing harm — the descriptor honestly records each key's real origin,
which is the point of §0.1a. The consequence is that the `built-policy-*` oracle
cross-check S5 owns cannot cover the `sh(wsh)` template on the delivered payload
at all, and nothing in the suite notices (the live tier is `oraclelive`-gated).
S5's own gate is Trace B, which is `wsh`, so the gate itself is unaffected.
Also side-effects the encoding **mode**: `commonOrigin` now returns `!ok` for every
`sh(wsh)` build with shared-origin cards, so those builds are `OriginDivergent`
rather than `OriginShared` — a wider byte change than the implementer's §7 item 6
("the self slot's path") states.

### M3 — an unparseable cosigner origin now reaches the operator as the generic "Couldn't assemble the wallet policy." — **CONFIRMED (traced), latent**

`gui/multisig_build.go:1000-1003` — `cosignerFromCard` returns the raw
`bip32.ParsePath` error, which `assembleBuildPolicy` propagates and the flow
renders with the generic text. S2's `errBuildForeignOrigin` gave this shape a
NAMED screen quoting the slot and the card. The plan's own reasoning for M-1's
named refusal (*"the generic text describes a device problem and offers the
operator nothing to do"*) applies verbatim here. **Not reachable from a decoded
payload card**: `mk.Decode` reconstructs `Card.Path` from decoded components via
`pathString(comps)` (`mk/mk.go:286`), so it always parses; the surviving subtest
`TestBuildRecordsTheCardsOwnOrigin/an unreadable declared origin is refused`
asserts only that it is refused, not that it is named. Latent regression only.

### M4 — a comment that S5 was named to re-decide is now describing the picker rather than the model

`gui/multisig_build_census.go:60-63`: *"The registry today holds exactly one seed
… **S5 multiplies the masters in it; the bound is filed to be re-decided there**,
when it would actually change something."* S5 is this block. The claim stays true
only because the picker is single-select — i.e. it now describes a screen that a
scheduled follow-up will remove, and it will go stale silently at that moment.
Either re-decide the bound or re-word it to name the picker as the reason.

---

## NIT

### N1 — `emptyOriginSlot`'s attribution loop can produce an empty `Declared`

`gui/multisig_build.go:1130-1145`. The loop that recovers the card's spelling
`continue`s over held slots and can only ever match a card-filled slot; if
`emptyOriginSlot` ever returned a **held** slot, `declared` stays `""` and
`buildEmptyOriginMessage` renders it as `"m"` — quoting the operator something
they never typed. Unreachable today because `derivedSlotOrigin` always returns 4
components, so a held slot cannot have an empty origin. Worth a one-line guard or
a comment naming the invariant.

---

## The seams, one by one

| # | seam | verdict |
| --- | --- | --- |
| 1 | ms1-per-distinct-master loop | **WRONG — C1.** The reverse direction is safe: two different masters can never collide onto one ms1, since `SeedID` is a registry index and the flow assigns a fresh one per held slot. |
| 2 | fingerprint-vs-seedID keying | The disclosed half is **sound**; a third, undisclosed site is not — see below. |
| 3 | emission order | **CORRECT on every arm — CONFIRMED (executed).** `full=true` → `[ms1 ms1 mk1 mk1 mk1 md1]`; `full=false` → `[mk1 mk1 mk1 md1]`. The three separate append loops make interleaving structurally impossible; watch-only and template both flow through the same site. |
| 4 | removed foreign-origin refusal | **CLEAN — CONFIRMED.** `grep` for `originIsShared`, `errBuildForeignOrigin`, `buildForeignOriginMessage` over the whole tree → no hits. §4.1's duplicate check still runs first: `duplicateSlotPair(all)` at `gui/multisig_build.go:1097`, `md.EncodeMultisig(req)` at `:1128`, and the empty-origin attribution only runs on the encoder's error. `TestBuildRecordsTheCardsOwnOrigin/a duplicate outranks anything the encoder would say` pins it. |
| 5 | `sh(wsh)` normative byte change | **No committed golden or gate record stales — CONFIRMED.** `oracle/gaterecords/S0-trace-a.*` is `bip48-p2wsh` / `m/48h/0h/0h/2h` (Trace A is wsh); the oracle's origin→template map already registers `m/48'/0'/0'/1' → bip48-p2sh-p2wsh` (`oracle/expect_test.go:429`); `cmd/emu/walk_s3_nested.js` carries no `m/48…` assertion, only the NOTE needle. What is **not** clean is M2 above. |
| 6 | `multisigEngraveCards` one-of-each adapter | **Byte-identical — CONFIRMED (executed against `git show HEAD:gui/multisig_engrave.go`).** `full=true` → `ms1 secret share` / `mk1 key` / `md1 descriptor` with the same summaries and the same `append([]string(nil), …)` cloning; `full=false` → `mk1 key` / `md1 descriptor`. `numberedLabel` returns the base string verbatim for `n <= 1`, so `gui/multisig.go:172` is unchanged. |
| 7 | seed handling | **CLEAN — CONFIRMED (traced).** The tail passes `seed.Mnemonic` (a slice header sharing the registry's backing array, which `reg.scrub()` zeroes) into `deriveMultisigLeg`; `deriveAccountXpub` allocates a fresh seed and wipes it, `m.Entropy()` returns a fresh buffer wiped after `EncodeMS1`, and neither mutates the mnemonic. No new buffer outlives the single deferred scrub. One quantitative note, not a finding: `legs` now retains N MS1 **strings** (unscrubbable by construction, as the registry comment already states) for the remainder of the flow where pre-S5 it retained one. |

## The three questions the brief asked

1. **Is the disclosed keying choice (c) sound?** Yes. Keying the BIP-48 account
   ordinal on the master fingerprint is the safer of the two options, and its
   worst case is benign — an FP collision bumps an account and the keys still
   differ. The gate's `SeedID`-keyed multi-account notice diverging from it costs
   only a **missing informational line**, not a wrong artifact. Measured on the
   snapshot: under the flow's shape the notice is absent (`notices = []`); under
   the test's shape it is present (*"Slots @0 and @1 all come from your seed, at
   different key origins. That is a multi-account wallet and is allowed."*). No
   funds consequence either way.
2. **Is the disclosed list complete?** **No.** The same `SeedID`-vs-master
   inconsistency governs a third site the report does not mention — the ms1
   dedupe key in `buildEngraveTail` — and that one *is* funds- and
   secret-relevant (**C1**). A second omission is **M1**
   (`buildOriginAnnouncement` announces account 0 only).
3. **Does the diff otherwise implement S5's model and tail per the frozen plan?**
   Yes, everywhere else I probed. Per-slot origins reach the descriptor and travel
   with the key so the derived path and the declared path cannot come apart; the
   `both` arm honours the card; `OriginShared`/`OriginDivergent` is decided over
   parsed components so notation cannot move the bytes; the duplicate-key ordering
   ruling survives; the engrave order matches the oracle's declared kind sequence;
   and the one-of-each adapter keeps the supply path byte-identical.

---

## How to reproduce C1

On a snapshot of the worktree with no mutation markers present, in `package gui`:

```go
reg, ids := s5Registry(t, fixtureMasterA, fixtureMasterA, fixtureMasterB) // one entry PER HELD SLOT
p := buildPolicyParams{Script: md.MultisigWsh, N: 4, K: 3, SelfSlots: []int{0, 1, 2}}
cards := []mk.Card{dupTestCard(t, 2)}
sources := buildSlotSources(p, ids, []int{2}, reg)
self, _ := buildSelfKeys(sources, p.Script, reg, s5Net)
out, _, _, _ := assembleBuildPolicy(p, self, cards)
_, cardsOut, _ := buildEngraveTail(sources, p.Script, reg, s5Net, cards, out, true)
// count cardMS1 in cardsOut, decode each with codex32.DecodeMS1, compare entropy:
// 3 plates, master A twice.
```

The only difference from `s5TraceB` is `ids` — one registry entry per held slot,
which is what `gui/multisig_build.go:194-201` builds — instead of
`{ids[0], ids[0], ids[1]}`.
