# S5 (work block A+B) — the multi-slot model, divergent origins, and the engrave tail

Implementer report. Worktree `/scratch/code/shibboleth/wt-s5`, branch `s5-multislot`,
based on fork `main` @ `84a4f4a`. **Not committed; the worktree is left dirty for
review.**

Plan: `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` §0 (lines 1–163) and
the S5 section (lines 1137–1298). The plan is frozen from S1 on; nothing here is
a redesign. Two places where I made a call the plan does not spell out are named
in **§7 Assumptions and things I could not do**.

---

## 1. What changed, and where

All paths are in `/scratch/code/shibboleth/wt-s5`.

### New files

| file | what |
| --- | --- |
| `gui/multisig_build_tail.go` | `buildEngraveTail` (`:45`) — one leg per HELD slot at that slot's own origin, one ms1 per DISTINCT master; `errBuildNoHeldSlot` (`:25`) |
| `gui/multisig_build_s5_test.go` | plan tests 1–7 plus §0.1a and the no-collision argument |
| `gui/multisig_build_s5_flow_test.go` | plan test 8, driven through the REAL flow |

### The model

- `gui/multisig_build.go:687-700` — `buildPolicyParams.SelfSlot int` →
  **`SelfSlots []int`**, a set of held slots. Doc records that each held slot gets
  its own BIP-48 account, which is what keeps two slots held from one master off
  §4.1's duplicate-key refusal.
- `gui/multisig_build.go:771-775` — the `@S` picker now writes
  `p.SelfSlots = []int{sIdx}`. **The picker is still single-select**; only the
  model and tail are multi-slot (see §7).
- `gui/multisig_build.go:395-440` — `buildSlotSources(p, seedIDs []int, chosen []int, reg *seedRegistry)`.
  The account is assigned **here**, keyed on the seed's **master fingerprint**,
  not on the seedID: the ordinal of a held slot among the held slots sharing a
  master. Keying on seedID would mint the *same key twice* whenever the operator
  typed one seed for two held slots (the flow registers one entry per held slot),
  and §4.1 would then refuse a legitimate multi-account wallet.
- `gui/multisig_build.go:448-455` — new `heldSlotKey{Slot, Xpub, MasterFP, Origin}`.
  The origin travels **with** the key rather than being recomputed at assembly, so
  the path the device derived at and the path it declares on steel cannot come
  apart.
- `gui/multisig_build.go:462-484` — new `buildSelfKeys(sources, script, reg, net)`:
  step (4b) generalised to derive one key per DERIVED held slot at
  `derivedSlotOrigin(script, account)`. Still **skipped on a `both` slot** (M-B).
- `gui/multisig_build_slots.go:103-122` — `derivedSlotOrigin(script, account)` is
  now **template-aware** (§0.1a) via the new single-site
  `multisigScriptTypeComponent`: `wsh`→`2'`, `sh(wsh)`→`1'`, legacy `sh`→`2'`
  (device convention, announced loudly).
- `gui/multisig_build_slots.go:347` — `buildSlotGate` gained a
  `script md.MultisigScript` parameter, so the gate's distinctness key uses the
  same single account→path site rather than a script-blind copy.
- `gui/multisig_build_slots.go:494-550` — `buildSlotSourceLines` /
  `buildSlotSourceReviewFlow` lost their `selfSlot int` parameter; a derived slot
  now names its account when the account is **not 0** (behaviour-identical for
  every shape reachable before S5).
- `gui/multisig_build_payload.go:364` — `buildCosignerOrigins(p, chosen)`
  (was `(n, selfSlot, selfFromCard, chosen)`); the skip test is now set
  membership over `p.SelfSlots`.
- `gui/multisig_build.go:912` and `:944` — `buildSlotProvenance(slot, p, origins)`
  and `buildDuplicateKeyMessage(dup, p, origins)` take the whole `p` for the same
  reason.

### Origins

- `gui/multisig_build.go:985` — **`cosignerFromCard` stops discarding
  `card.Path`.** The origin is carried as PARSED COMPONENTS (permissive on
  spelling, strict on value); a path that does not parse is refused rather than
  stamped over.
- `gui/multisig_build.go:1117-1127` — `assembleBuildPolicy` now picks the origin
  mode: **`OriginShared` when every slot's origin agrees, `OriginDivergent` when
  they do not** (`commonOrigin`, `:1159`). This is what keeps S2's committed
  byte-identity golden meaningful — the two are different wire forms of the same
  paths, and switching unconditionally would re-mint every id already matched
  against a coordinator.
- `gui/multisig_build.go:1025` — `assembleBuildPolicy(p, self []heldSlotKey, cosigners []mk.Card)`;
  `want` is now `p.N - len(self)`, with bounds/duplicate-slot guards.
- **S2's interim foreign-origin refusal is REMOVED**: `errBuildForeignOrigin`,
  `buildForeignOriginMessage` and `originIsShared` are deleted, the loop that
  stood at `:962-985` is gone, and the dispatch arm at `:245-250` is replaced.
  **§4.1's duplicate-key check survives and still runs FIRST** — it is still the
  block immediately above the encode call (`gui/multisig_build.go:1097`),
  and the comment that rules the order is preserved and updated in place.
- `gui/multisig_build.go:864` + `:880` — new **named** refusal for spec M-1:
  `errBuildEmptyOrigin` / `buildEmptyOriginMessage`, dispatched to a "Key origin
  missing" screen at `:253-260`. `emptyOriginSlot` is at `:1182`. **md refuses first and the gui
  attributes**: the request goes to `md.EncodeMultisig`, and only when it errors
  does `emptyOriginSlot` name the slot. That keeps `md` the authority (the plan
  forbids changing `md/`, and `errMultisigEmptyDivergent` is unexported) instead
  of pre-empting a refusal that might not exist.
- `gui/multisig_build.go:1265` — `buildOriginAnnouncement` rewritten to
  §0.1a's **from-S5** text: `sh(wsh)` now says "Key origins follow BIP-48 for
  nested segwit (script type 1h)" and names `m/48h/0h/0h/1h`; legacy `sh` keeps
  its "no BIP assigns" sentence; every arm says "Your key origins", because a
  payload card's own origin is an answer the operator gave and announces nothing.

### The tail

- `gui/multisig_engrave.go` rewritten: `multisigEngraveCardsMulti(ms1s, mk1s, md1)`
  emits **all ms1s, then all mk1s, then the md1** — the order
  `oracle.ArtifactKindsFor(KindBuiltPolicyFull)` declares and
  `oracle.CheckArtifactShape` enforces as consecutive runs.
  `multisigEngraveCards(ms1, mk1, md1, full)` is kept as a one-of-each adapter,
  so **`gui/multisig.go:172`'s behaviour is byte-identical** (a lone card keeps
  its shipped label verbatim; `numberedLabel` only indexes when there are
  several).
- `gui/multisig_build_tail.go:45-95` — `buildEngraveTail`: origin is
  `derivedSlotOrigin(script, account)` for a derived slot and the **card's own
  declared path** for a `both` slot; ms1 is requested only for a seed registry
  entry not already engraved.
- `gui/multisig_build.go:324` — the flow's step (9) now calls the tail.
- `gui/multisig_build.go:44-56` — defensive guard: `len(p.SelfSlots) == 0` is a
  named refusal rather than a panic mid-flow.

### Tests updated for the new signatures (mechanical)

`gui/multisig_build_test.go`, `gui/multisig_build_dupkey_test.go`,
`gui/multisig_build_gate_test.go`, `gui/multisig_build_oracle_test.go`,
`gui/multisig_build_payload_test.go`, `gui/multisig_build_scrub_test.go`,
`gui/multisig_nested_name_test.go`, `gui/template_engrave_test.go`,
`gui/multisig_testhelpers_test.go` (new `selfKeyAt` helper).
**The full list was resolved with `grep -rn "SelfSlot" --include='*.go'`, not
from the brief's list** — the brief's list omitted `multisig_nested_name_test.go`
and `multisig_build_scrub_test.go`.

### Tests whose SUBJECT changed

- `gui/multisig_build_origin_test.go` — S2's `TestBuildRefusesForeignOriginCardBeforeS5`
  and `TestBuildFlowRefusesForeignOriginCard` are **replaced** by
  `TestBuildRecordsTheCardsOwnOrigin` and `TestBuildFlowAcceptsDivergentOriginCard`.
  The property both stages exist for is preserved verbatim: *the plate never
  carries a path the card disagrees with*. The unreadable-origin refusal and the
  duplicate-outranks-origin ordering survive as subtests.
- `gui/multisig_build_announce_test.go` — the "every template states the origin in
  force" subtest now reads the expected path off `derivedSlotOrigin` rather than
  asserting `multisigSharedOrigin()` for all three, and the nested-segwit subtest
  asserts the S5 sentence and that the review **no longer** names the 2' path.
- `gui/multisig_build_gate_test.go:446` —
  `TestDerivedSlotAccountIsTheBip48AccountComponent` now pins the ACCOUNT binding
  at `md.MultisigWsh`; the SCRIPT-TYPE binding gets its own test.
- `cmd/emu/needle_test.go:76` and `cmd/emu/walk_s3_nested.js:65` — S3's
  nested-segwit walk needle moved from `"BIP-48 assigns m/48h/0h/0h/1h to nested
  segwit"` to `"BIP-48 for nested segwit (script type 1h)"`, because the
  announcement it anchors on was rewritten. Caught by
  `TestBuildFlowNeedlesHaveExactlyOneProductionSite` going red, not by me.

---

## 2. Test-first: what I saw fail, and how

**Sequencing, stated plainly.** I first landed a *behaviour-preserving* widening
of the signatures (`SelfSlot`→`SelfSlots`, `assembleBuildPolicy(p, self, cosigners)`,
`buildSlotGate(+script)`, `derivedSlotOrigin(script, account)` with the parameter
**ignored**, and the tail extracted into `buildEngraveTail` unchanged), and gated
it on the existing suite: `go test ./... -count=1` → exit **0**, **51 ok / 0 FAIL**.
That refactor is not a behaviour change and had no test of its own. **Every
behaviour change after it was written test-first**, against an API that already
compiled, so the reds below are behavioural rather than "undefined symbol" —
except test 6, which introduces a new error type and could only fail to compile.

Verbatim red baseline (`go test ./gui/ -run '<the S5 tests>' -v`, exit **1**):

```
=== RUN   TestGateStillFiresAfterOriginsDiverge
=== RUN   TestGateStillFiresAfterOriginsDiverge/PROCEED_when_the_key_is_genuinely_derived_at_the_card's_own_origin
    multisig_build_s5_flow_test.go:134: a `both` slot whose card declares m/48h/0h/1h/2h, carrying the key that seed derives AT m/48h/0h/1h/2h, did not reach the policy review; got "Thekeycardforslot@0(payloadcard1)saysitskeywasderivedatm/48h/0h/1h/2h,butthisbuilddeclaresonesharedorigin,m/48h/0h/0h/2h,foreveryslot.Stampingthesharedoriginonitwouldputaderivationpathonyoursteelthatthecarditselfdisagreeswith.Nothingwasengraved.Useacardderivedatm/48h/0h/0h/2h,orwaitforper-slotorigins.Keyoriginmismatch".
        A gate deriving at m/48h/0h/0h/2h instead would fail exactly here
=== RUN   TestGateStillFiresAfterOriginsDiverge/FAIL_naming_the_slot_when_the_key_was_derived_somewhere_else
    multisig_build_s5_flow_test.go:160: S4's slot-source review was not reached; got "Yousaidslot@0(payloadcard1)holdsYOURkey,butthatcard'skeyisnotwhatyourseedderivesatm/48h/0h/1h/2h.Nothingwasengraved.Likelycauses:thewrongcardforthisslot,amistypedorskippedpassphrase,oranotherwallet'scard.ReassigningthisslotSUPPRESSESthecheckratherthanfixingit.Gobackandpickadifferentcard,checkthepassphrase,orrewritethepayloadwith`mesyswpack`.Keydoesnotmatchseed"
--- FAIL: TestGateStillFiresAfterOriginsDiverge (0.33s)
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/PROCEED_when_the_key_is_genuinely_derived_at_the_card's_own_origin (0.05s)
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/FAIL_naming_the_slot_when_the_key_was_derived_somewhere_else (0.27s)
=== RUN   TestMultiSlotSelfAssembles
    multisig_build_s5_test.go:127: @1 origin = m/48h/0h/0h/2h, want m/48h/0h/1h/2h. A policy that declares the wrong derivation path is a policy a BIP-48-aware coordinator restores at the wrong place
    multisig_build_s5_test.go:135: @0 and @1 declare the SAME origin (m/48h/0h/0h/2h); Trace B holds two accounts of ONE master, so the policy MUST be divergent-origin
--- FAIL: TestMultiSlotSelfAssembles (0.01s)
=== RUN   TestCosignerCardOriginIsHonoured
    multisig_build_s5_test.go:183: cosignerFromCard produced a 0-component origin for a card declaring "m/48h/0h/1h/2h" (4 components); the card's origin is being discarded
--- FAIL: TestCosignerCardOriginIsHonoured (0.00s)
=== RUN   TestLegDerivedAtHeldSlotOrigin
--- PASS: TestLegDerivedAtHeldSlotOrigin (0.01s)
=== RUN   TestOneMk1PerHeldSlot
--- PASS: TestOneMk1PerHeldSlot (0.01s)
=== RUN   TestFullModeEngravesMs1ForEveryMaster
    multisig_build_s5_test.go:357: a FULL build across masters A and B engraved 3 ms1 plate(s), want 2. A backup labelled "Full (seed + keys)" that is missing a master leaves two legs against k=3: unspendable
--- FAIL: TestFullModeEngravesMs1ForEveryMaster (0.01s)
=== RUN   TestReRunMintsByteIdenticalPlates
--- PASS: TestReRunMintsByteIdenticalPlates (0.02s)
=== RUN   TestAssembleBuildPolicyStaysSharedWhenOriginsAgree
--- PASS: TestAssembleBuildPolicyStaysSharedWhenOriginsAgree (0.00s)
=== RUN   TestDerivedSlotOriginIsTemplateAware
    multisig_build_s5_test.go:580: derivedSlotOrigin(1, 0) = m/48h/0h/0h/2h, want m/48h/0h/0h/1h (BIP-48's assignment for nested segwit)
    multisig_build_s5_test.go:586: derivedSlotOrigin(sh(wsh), 3) = m/48h/0h/3h/2h, want m/48h/0h/3h/1h
    multisig_build_s5_test.go:591: nested segwit and native segwit derive at the same path, so the template-awareness is not there
--- FAIL: TestDerivedSlotOriginIsTemplateAware (0.00s)
=== RUN   TestTwoHeldSlotsFromOneMasterDoNotCollide
--- PASS: TestTwoHeldSlotsFromOneMasterDoNotCollide (0.01s)
FAIL
FAIL	seedhammer.com/gui	0.420s
```

Test 6's red is a compile failure, because the named error it demands did not
exist (`go vet ./gui/`, exit **1**):

```
# seedhammer.com/gui
# [seedhammer.com/gui]
vet: gui/multisig_build_s5_test.go:682:9: undefined: errBuildEmptyOrigin
```

### Per test

| # | test | red before implementation? |
| --- | --- | --- |
| 1 | `TestMultiSlotSelfAssembles` | **YES**, behavioural: `@1` reported the shared origin |
| 2 | `TestCosignerCardOriginIsHonoured` | **YES**, behavioural: `cosignerFromCard` produced a 0-component origin |
| 3 | `TestLegDerivedAtHeldSlotOrigin` | **NO — it passed on first run.** See below |
| 4 | `TestOneMk1PerHeldSlot` | **NO — it passed on first run.** See below |
| 5 | `TestFullModeEngravesMs1ForEveryMaster` | **YES**, behavioural: 3 ms1 plates for 2 masters |
| 6 | `TestDepthZeroCosignerCardIsNamedRefusal` | **YES**, compile: `undefined: errBuildEmptyOrigin` |
| 7 | `TestReRunMintsByteIdenticalPlates` | **NO — it passed on first run.** It pins a property that was already true and unasserted, which is what the plan says it is for |
| 8 | `TestGateStillFiresAfterOriginsDiverge` | **YES**, behavioural, both arms |
| — | `TestDerivedSlotOriginIsTemplateAware` (§0.1a) | **YES**, behavioural |
| — | `TestAssembleBuildPolicyStaysSharedWhenOriginsAgree` | **NO** — a regression guard on Trace A's committed bytes; it *should* be green before and after |
| — | `TestTwoHeldSlotsFromOneMasterDoNotCollide` | **NO** — green because I chose master-fingerprint account keying before writing it (see §7) |

**Tests 3 and 4 not going red is a real weakness in this report's TDD claim and I
am not dressing it up.** The cause: my signature-widening step extracted today's
tail into `buildEngraveTail` as a loop over the slot sources, and that loop reads
each slot's origin off the source, so the per-held-slot behaviour those two tests
assert arrived with the *refactor* rather than with the feature. What stands in
for the missing red is an explicit mutation of each — §3 items B and C — which is
the same evidence a red would have been.

---

## 3. Mutation checks

Each mutation prints a marker **from the mutated line** to stderr, so the
evidence is that the line RAN and not merely that the edit landed. All four were
reverted (`grep -rn "MUTATION-RAN" gui/ cmd/` → **0** hits at the end).

### A. Plan test 5 — capture ONE mnemonic in the engrave loop

Mutation: in `buildEngraveTail`, hoist the first held slot's registry entry into
`capturedSeed` and use it for every leg.

```
=== RUN   TestFullModeEngravesMs1ForEveryMaster
MUTATION-RAN buildEngraveTail slot=0 using captured seed fp=73c5da0a (real fp=73c5da0a)
MUTATION-RAN buildEngraveTail slot=1 using captured seed fp=73c5da0a (real fp=73c5da0a)
MUTATION-RAN buildEngraveTail slot=2 using captured seed fp=73c5da0a (real fp=b8688df1)
    multisig_build_s5_test.go:401: master A's seed was engraved TWICE and the other master not at all: a "Full" backup with a master missing
--- FAIL: TestFullModeEngravesMs1ForEveryMaster (0.07s)
FAIL
FAIL	seedhammer.com/gui	0.080s
```

Proof the line ran: three markers, one per held slot, and **slot 2 shows the
captured fingerprint `73c5da0a` against the real `b8688df1`** — master A's seed
substituted for master B's at the moment the leg was derived.

### B. Plan test 3 — derive every leg at the shared origin

Mutation: `origin = multisigSharedOrigin()` in `buildEngraveTail`'s
`slotFromSeed` arm.

```
=== RUN   TestLegDerivedAtHeldSlotOrigin
MUTATION-RAN buildEngraveTail slot=0 origin=m/48h/0h/0h/2h (real=m/48h/0h/0h/2h)
MUTATION-RAN buildEngraveTail slot=1 origin=m/48h/0h/0h/2h (real=m/48h/0h/1h/2h)
MUTATION-RAN buildEngraveTail slot=2 origin=m/48h/0h/0h/2h (real=m/48h/0h/0h/2h)
    multisig_build_s5_test.go:253: the @1 leg's mk1 declares origin m/48h/0h/0h/2h, want m/48h/0h/1h/2h. Deriving every leg at the shared origin is exactly what this test exists to catch
    multisig_build_s5_test.go:265: the @1 leg's key is not the key the descriptor holds at @1
    multisig_build_s5_test.go:270: the @1 leg carries @0's key, so it was derived at the shared origin and merely landed on a slot the descriptor also contains
--- FAIL: TestLegDerivedAtHeldSlotOrigin (0.06s)
FAIL
FAIL	seedhammer.com/gui	0.069s
```

Proof the line ran: the slot-1 marker shows the substituted origin next to the
real one.

### C. Plan test 4 — stop after the first held slot's leg

```
=== RUN   TestOneMk1PerHeldSlot
MUTATION-RAN buildEngraveTail stopping after slot=0 with 1 leg(s)
    multisig_build_s5_test.go:301: 1 leg(s) for 3 held slot(s)
    multisig_build_s5_test.go:310: the engrave set carries 1 mk1 card(s) for 3 held slot(s); a held slot with no key card is a slot the operator cannot prove
--- FAIL: TestOneMk1PerHeldSlot (0.02s)
FAIL
FAIL	seedhammer.com/gui	0.027s
```

### D. Plan test 8 — the gate derives at the shared origin, not the card's own

Mutation: `origin = multisigSharedOrigin()` at the end of `bothSlotKey`
(`gui/multisig_build_slots.go`), which is the exact wrapper S4's
`TestGateDerivesAtTheCardsOwnOrigin` was written against.

```
=== RUN   TestGateStillFiresAfterOriginsDiverge
=== RUN   TestGateStillFiresAfterOriginsDiverge/PROCEED_when_the_key_is_genuinely_derived_at_the_card's_own_origin
MUTATION-RAN bothSlotKey card=m/48h/0h/1h/2h -> gate derives at m/48h/0h/0h/2h
    multisig_build_s5_flow_test.go:130: the gate did not PROCEED on a card declaring m/48h/0h/1h/2h and carrying the key that seed derives there; screen reads "Yousaidslot@0(payloadcard1)holdsYOURkey,butthatcard'skeyisnotwhatyourseedderivesatm/48h/0h/1h/2h...Keydoesnotmatchseed"
=== RUN   TestGateStillFiresAfterOriginsDiverge/FAIL_naming_the_slot_when_the_key_was_derived_somewhere_else
MUTATION-RAN bothSlotKey card=m/48h/0h/1h/2h -> gate derives at m/48h/0h/0h/2h
    multisig_build_s5_flow_test.go:170: a card declaring m/48h/0h/1h/2h while carrying the key from m/48h/0h/0h/2h was ACCEPTED. Its mk1 would assert membership in a wallet at a path its own key is not at. Screen reads "KeysourcesWhereeachkeycomesfrom:@0yours:payloadcard1,checkedagainstyourseedfor@0@1acosigner:payloadcard2,takenassupplied..."
--- FAIL: TestGateStillFiresAfterOriginsDiverge (0.91s)
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/PROCEED_when_the_key_is_genuinely_derived_at_the_card's_own_origin (0.14s)
    --- FAIL: TestGateStillFiresAfterOriginsDiverge/FAIL_naming_the_slot_when_the_key_was_derived_somewhere_else (0.72s)
FAIL
FAIL	seedhammer.com/gui	0.916s
```

**Both arms moved**, which is the pair's whole point: the honest card was refused
AND the liar card was accepted (it reached "Key sources"). A smoke test asserting
only "no error" would have stayed green on the second arm.

---

## 4. The gate (final, after every mutation was reverted)

```
$ nix develop --command go test ./... -count=1
GO_TEST_EXIT=0
ok lines: 51
FAIL lines: 0
ok  	seedhammer.com/gui	98.485s
```

```
$ nix develop --command gofmt -l ./
GOFMT_EXIT=0
--- gofmt -l output (excluding the nix warning) ---
--- line count: 0 ---
```

```
$ rm -rf $GOCACHE; nix develop --command env GOCACHE=$GOCACHE go vet ./...
GO_VET_EXIT=1
total=40 non_test=0
```

Exit codes were read from `$status` **immediately after the command**, with
output redirected to a file and grepped afterwards — never through a pipe. The
vet run used a freshly deleted `GOCACHE`, so the 40 findings were genuinely
recomputed; **0** of them are outside `_test.go`, which is the documented clean
baseline.

---

## 5. Trace B, as built

`TestMultiSlotSelfAssembles` logs it:

```
Trace B assembled: 8 md1 chunk(s), stub ..., origins
  m/48h/0h/0h/2h / m/48h/0h/1h/2h / m/48h/0h/0h/2h / m/48h/0h/0h/2h
```

and `TestFullModeEngravesMs1ForEveryMaster` pins the full engrave set as
**2 ms1 + 3 mk1 + 1 md1 = 6 cards**, in that kind order, with each ms1 decoded
through `codex32.DecodeMS1` and its entropy compared to the master it claims.

**The brief's warning about held slots colliding did not materialise, and the
reason is a decision I had to make** — see §7 item 3.

---

## 6. Scope kept out (as instructed)

Not touched: the review screen's per-slot keys, the EXPERIMENTAL warning
rewrite, DESTROY-not-discard, the passphrase-absence text, F-182, F-185, the
emulator walk, and the gate-record mint. Plan test 7's byte-identical-re-run
**assertion** is in (`TestReRunMintsByteIdenticalPlates`); its abort-screen
**wording** is not.

---

## 7. Assumptions, and things I could NOT do

1. **The `@S` picker is still single-select, so the FLOW cannot build Trace B.**
   The plan's S5 implementation list names the model, `cosignerFromCard`, the
   tail and the refusal removal, and the brief scopes screens out. So
   `buildPolicyParams.SelfSlots` is a set, `buildSlotSources`, `buildSelfKeys`,
   `assembleBuildPolicy`, `buildSlotGate` and `buildEngraveTail` all accept N
   held slots, and the multi-slot tests drive them directly — the same posture
   S4 took for the gate, and stated in the header comment of
   `gui/multisig_build_slots.go`. The seed-entry loop at
   `gui/multisig_build.go:194-201` is already per-held-slot, so the block that
   lands the multi-select screen should not need to touch it.
   **Consequence for the S5 gate: "Trace B completes ... by emulator walk" is NOT
   satisfied by this block.** It is satisfiable by test, and it is not
   satisfiable by walk until the picker lands.

2. **`multisigVerifyFlow` verifies `legs[0]` only.** It takes a single
   `bundle.Bundle` and recovers the origin through `findUserSlot`, which returns
   the FIRST slot a seed matches — so it structurally cannot verify the second
   and third legs of a multi-held-slot build. Every shape the flow can currently
   produce has exactly one leg, so this is today's behaviour unchanged, and I
   left it that way rather than looping (a loop would verify legs 2..n against
   the wrong origin and report confusing FAILs). **This needs a follow-up owned
   by the multi-select-picker block.** It is commented in place at
   `gui/multisig_build.go:345-352`.

3. **I chose to key the BIP-48 account ordinal on the seed's MASTER FINGERPRINT,
   not on the seedID.** The plan says two held slots from one master derive at
   different accounts but does not say how the account is assigned. The flow
   registers one registry entry per held slot, so an operator typing master A for
   both held slots produces two entries of one master; seedID keying would give
   both account 0, mint the same key twice and trip §4.1 — refusing a legitimate
   wallet. Master-fingerprint keying is what makes
   `TestTwoHeldSlotsFromOneMasterDoNotCollide` pass. **If a reviewer disagrees
   with this rule, it is one function** (`buildSlotSources`,
   `gui/multisig_build.go:395-440`). A theoretical 32-bit fingerprint collision
   between two genuinely different masters would give the second one account 1
   instead of 0 — harmless, since the keys still differ.
   Note the interaction I did **not** fix: the gate's multi-account NOTICE keys
   on `SeedID`, so one master registered twice will not produce the "different
   key origins, that is allowed" notice. Unreachable through today's flow; worth
   a follow-up alongside item 1.

4. **`errMultisigEmptyDivergent` is unexported, so test 6 asserts md's refusal by
   its message text** (`"non-empty Origin"`), with the file:line cited. The plan
   forbids changing `md/`. The production path genuinely reaches md's refusal —
   `assembleBuildPolicy` encodes first and only then attributes — so the two
   cannot drift silently.

5. **`buildEmptyOriginMessage`'s wording is mine.** The plan requires a named
   refusal and says nothing about the text; I followed §0.1b (name the payload
   and the keyboard, never "scan a card") and the no-em-dash rule, and the test
   asserts all three. It has **not** been raster-checked against
   `buildWalkRasterFloor` through a real flow drive — there is no fixture on the
   delivered payload carrying a depth-0 card, so a full-flow drive would need a
   new payload. The message is 300+ characters and is in the same length class as
   the refusals that were measured, but **I did not measure it**; a reviewer
   should treat that as an open item.

6. **`sh(wsh)` builds now derive at `m/48h/0h/0h/1h`, which changes the bytes of
   any `sh(wsh)` policy this device would previously have minted.** That is
   §0.1a's ruling and its whole point, but it means any `sh(wsh)` artifact cut
   before this change no longer round-trips against a rebuild from the same
   inputs. Nothing in the tree pinned such an artifact (the suite is green), and
   `sh(wsh)` is hardware-unvalidated, but it is a normative behaviour change and I
   am naming it.

7. **The engrave-card labels for multiples are `"<base> N of M"`** (e.g. "mk1 key
   2 of 3"), chosen so a lone card keeps its shipped label byte-for-byte. Naming
   them by SLOT would be more useful on the plate census and is screen work I
   left to the later block.

8. **Not committed.** The worktree is dirty: 17 modified files and 3 new ones
   (`gui/multisig_build_tail.go`, `gui/multisig_build_s5_test.go`,
   `gui/multisig_build_s5_flow_test.go`).
