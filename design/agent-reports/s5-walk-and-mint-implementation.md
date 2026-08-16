# S5.D — the Trace B emulator walk, and the first gate-record MINT for a built policy

Implementer report, 2026-08-16. Worktree `/scratch/code/shibboleth/wt-s5`,
branch `s5-multislot`, parent `023505c`.

**Outcome: GREEN.** Trace B walks end to end in the emulator, the built-policy
gate record has been minted for the first time, and all 17 engraved strings are
byte-identical to the pinned primary. Two commits landed:

    42a0b99  S5.D: four screens the three older walks had drifted past, and a
             needle counter that measures screens
    7da66bd  S5.D: Trace B walks, and the first gate record for a BUILT policy
             is MINTED

Four findings are recorded below, all against the three pre-existing walks.

---

## 1. THE MINT — the gate that had never executed

    $ go run ./cmd/gaterecord -stage S5 \
        -walk <the saved run() return value> \
        -inputs oracle/gaterecords/S5-trace-b.inputs.json \
        -base S5-trace-b
    EXIT=0

    17 artifact(s) derived live by [ms md mk] and matched the walk's census
    wrote oracle/gaterecords/S5-trace-b.record.json
          oracle/gaterecords/S5-trace-b.walk.json
          oracle/gaterecords/S5-trace-b.expect.json

`oracle/gaterecords/` now holds two records: `S0-trace-a.*` and `S5-trace-b.*`.

### The minted record's digests

    S5-trace-b.record.json  06888d28403c3fdd91259beb9126c51ecb4801a00ff423b6ebc3fbe63a7cf524
    S5-trace-b.walk.json    83522bfcc456b029fc737bf415059291096c4284526a9cfa79a71d2f03ee53fe
    S5-trace-b.expect.json  adf13d2f22a7b26052bfbb72de801b040c66615766105aed87eb3fa3a4c99090
    S5-trace-b.inputs.json  568e1c669bf74ed910710101a9aa18ad4a9aeb23e09bb8294123c8fad6e0f2e0

Resolved oracle SOURCE COMMITS in the record (method `binary-sha256`, all three
`version_matches_pin: true`):

    md  5a0a4f41017d71d47f70684c145702d4ca0c3aa9   reports "md 0.13.0"
    mk  a38a908e143c2c4bd6405997d62385b3df01615f   reports "mk 0.13.0"
    ms  d49d5c099bab89a1738f0d0c3df9306b354d62c3   reports "ms 0.16.0"

Walk binding: `pace 2048`, `elapsed_sec 446`, census `announced 51`,
`unattributed 0`, 17 plate digests for 17 strings.

The inputs file states `policy_id_stub: 70fb9d6e`. `oracle.DeriveExpected`
treats a stated stub on a built policy as a **checkable claim, never an
override** — md computes the id from the policy — and it agreed.

**The device's own policy stub, read off its Policy Review screen, is
`70fb9d6e`.** That is the first time the device's `md.EncodeMultisig` output has
been shown to agree with `md encode --policy-id-fingerprint` on a
divergent-origin policy.

---

## 2. THE BYTE COMPARISON, at full strength (§4.5 / F-171)

`cmd/gaterecord` refuses to mint a record whose census is not what the primary
just derived, byte for byte and **in order**, so the mint above *is* the
comparison. It was then re-run **independently** — re-deriving from the inputs
file rather than re-reading the committed expectation, invoking the pinned
binaries by **absolute path** (`~/.cargo/bin/{md,mk,ms}`, because `md` is a shell
alias for `mkdir -p` on this machine).

    policy template : wsh(sortedmulti(3,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/0'/2'/<0;1>/*,@3/48'/0'/0'/2'/<0;1>/*))
    md policy id    : 0x70fb9d6e

    expected 17 artifact(s); the walk engraved 17

    plate  0  ms1  EQUAL     self:masterA (slot @0, BIP-48 account 0)
              primary  ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
              engraved ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
    plate  1  ms1  EQUAL     self:masterB (slot @2, typed on the keyboard)
              primary  ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6
              engraved ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6
    plate  2  mk1  EQUAL     slot @0 chunk 1/2
              primary  mk1qpfxwdpqqsqhp7uadeeutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5fx368qrg28f8te2
              engraved mk1qpfxwdpqqsqhp7uadeeutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5fx368qrg28f8te2
    plate  3  mk1  EQUAL     slot @0 chunk 2/2
              primary  mk1qpfxwdpp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl99yee42sln0dmy2mnm7x6
              engraved mk1qpfxwdpp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl99yee42sln0dmy2mnm7x6
    plate  4  mk1  EQUAL     slot @1 chunk 1/3
              primary  mk1qpnxu9zqqsqhp7uadeeutks2lcztpqyqsqygpqyqsqygrqyqsqyg9qyqsqyqfz9jrcld706hn9svfgll7zvw5qnkxgea7gvezr79ykwqh7c2
              engraved mk1qpnxu9zqqsqhp7uadeeutks2lcztpqyqsqygpqyqsqygrqyqsqyg9qyqsqyqfz9jrcld706hn9svfgll7zvw5qnkxgea7gvezr79ykwqh7c2
    plate  5  mk1  EQUAL     slot @1 chunk 2/3
              primary  mk1qpnxu9zp68w6hzragnj3g5qrl85zeape8wq0vdczfyy55tqsd5576trsa3p40nfpd7hsyjyf7vlx6hk2j6ckr4wf0m3ej5qda3epmem8t59h
              engraved mk1qpnxu9zp68w6hzragnj3g5qrl85zeape8wq0vdczfyy55tqsd5576trsa3p40nfpd7hsyjyf7vlx6hk2j6ckr4wf0m3ej5qda3epmem8t59h
    plate  6  mk1  EQUAL     slot @1 chunk 3/3
              primary  mk1qpnxu9zzhpwhqz4qndj2dmjn8p
              engraved mk1qpnxu9zzhpwhqz4qndj2dmjn8p
    plate  7  mk1  EQUAL     slot @2 chunk 1/2
              primary  mk1qpz4mspqqsqhp7uad6ux3r03q5zg3vs7llvu2xd8x2rk7av9gmew82jq5zap9302ynhp37ggd6z5u4emag0zr8gh9upnj25xq0fg0fqy8dga
              engraved mk1qpz4mspqqsqhp7uad6ux3r03q5zg3vs7llvu2xd8x2rk7av9gmew82jq5zap9302ynhp37ggd6z5u4emag0zr8gh9upnj25xq0fg0fqy8dga
    plate  8  mk1  EQUAL     slot @2 chunk 2/2
              primary  mk1qpz4msppwyp4dfykwfkgg6fxyxetdcmythf4hsqzd3v879jprztejzs7ru2hwrrl5lpej444jxwfj
              engraved mk1qpz4msppwyp4dfykwfkgg6fxyxetdcmythf4hsqzd3v879jprztejzs7ru2hwrrl5lpej444jxwfj
    plate  9  md1  EQUAL     policy chunk 1/8
              primary  md1fc9ncrs9q6tvyyy5jmpprjjtvyyy5jmppp9gqpsgwyxxckg9qhwsv0jskp2rsal4elqtpkym09m9nl
              engraved md1fc9ncrs9q6tvyyy5jmpprjjtvyyy5jmppp9gqpsgwyxxckg9qhwsv0jskp2rsal4elqtpkym09m9nl
    plate 10  md1  EQUAL     policy chunk 2/8
              primary  md1fc9ncrs2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5wl4l0mn058ndxfl63ku4znzdd67w9fcd
              engraved md1fc9ncrs2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5wl4l0mn058ndxfl63ku4znzdd67w9fcd
    plate 11  md1  EQUAL     policy chunk 3/8
              primary  md1fc9ncrsjfzvynh66n94j5lcxlmx9ayav9mj0jjejcxy50llpx82qfmryv7l68w6hzjneq4gl32fjk5
              engraved md1fc9ncrsjfzvynh66n94j5lcxlmx9ayav9mj0jjejcxy50llpx82qfmryv7l68w6hzjneq4gl32fjk5
    plate 12  md1  EQUAL     policy chunk 4/8
              primary  md1fc9ncrscl2yu529qqleaqk0gwfmsrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3zutrxr6vmsmjw7
              engraved md1fc9ncrscl2yu529qqleaqk0gwfmsrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3zutrxr6vmsmjw7
    plate 13  md1  EQUAL     policy chunk 5/8
              primary  md1fc9ncr3rue7d40v4943v82ujlhr48x2rk7av9gmew82jq5zap9302ynhp37ggd6z5lp5zm3yqferql
              engraved md1fc9ncr3rue7d40v4943v82ujlhr48x2rk7av9gmew82jq5zap9302ynhp37ggd6z5lp5zm3yqferql
    plate 14  md1  EQUAL     policy chunk 6/8
              primary  md1fc9ncr309wwl2rcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq3y0cppephw3cjv
              engraved md1fc9ncr309wwl2rcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq3y0cppephw3cjv
    plate 15  md1  EQUAL     policy chunk 7/8
              primary  md1fc9ncr35f0xg2rc0ugjshlaruhytwx696ej6trer6486ajnxplvwf3eqy7pewnjcldss7d0scuqak8
              engraved md1fc9ncr35f0xg2rc0ugjshlaruhytwx696ej6trer6486ajnxplvwf3eqy7pewnjcldss7d0scuqak8
    plate 16  md1  EQUAL     policy chunk 8/8
              primary  md1fc9ncr37cmsx5tedmgpg3jz4xc5x2ue9a7z5h3rycu0nx9leekrarz2svuc4wj2qxd2smdkyn2d20
              engraved md1fc9ncr37cmsx5tedmgpg3jz4xc5x2ue9a7z5h3rycu0nx9leekrarz2svuc4wj2qxd2smdkyn2d20

    RESULT: 17/17 byte-identical, 0 mismatch(es)

Every **mk1** (7 chunks across 3 held slots) and every **ms1** (2, one per
distinct held master) — full string equality, plus all 8 md1 chunks. The
weakened two-part mk1 relation is not used anywhere.

---

## 3. THE WALK — `cmd/emu/walk_trace_b.js`

n=4, k=3, wsh. The operator holds **@0 (master A account 0), @1 (master A
account 1), @2 (master B account 0)**; payload cosigner card 4 (master C) fills
**@3**. Full mode. Completed in **446 s**, **17 plates**, `ok: true`,
`presented: 0`, `unattributed: 0`, `announced: 51`, 17 plate digests, and the
restore doc reads `Type: P2WSH 3-of-4 multisig (sorted)`.

### The needle, and why it is single-site TODAY

The walk's decisive anchors are **S5's multi-select @S picker**, which no earlier
walk reaches at all because they all tap "NO, THAT IS ALL" on its first screen:

    "Do you hold another slot?"    gui/multisig_build.go
    "Which other slot is yours?"   gui/multisig_build.go

plus the **plural** arm of S4's slot-source question, which only a multi-slot
build can draw:

    "keys on cards?"               gui/multisig_build_slots.go
    (the singular arm is "key on a card?"; buildSelfSourceLead deliberately
     keeps the two substrings disjoint so each is pinned separately)

Re-verified today, not inherited, by two independent measurements:

1. Source-site count, `git grep -lF … -- 'gui/*.go' | grep -v _test`:

       Choose policy type          1 site   gui/multisig_build.go
       How many keys (n)?          1 site   gui/multisig_build.go
       Which slot is your key?     1 site   gui/multisig_build.go
       Do you hold another slot?   1 site   gui/multisig_build.go
       Which other slot is yours?  1 site   gui/multisig_build.go
       Cosigner Keys               1 site   gui/multisig_build.go
       Where each key comes from:  1 site   gui/multisig_build_slots.go
       keys on cards?              1 site   gui/multisig_build_slots.go
       Plate Count                 1 site   gui/multisig_build.go

2. **Flow-owner count** — the new check (F-190, §6). Every one of the 15 pinned
   flow needles is drawn by exactly ONE flow, and each one's file matches its
   pin. `"Choose policy type"` and `"How many keys (n)?"` both resolve to
   `buildParamPickFlow` in `gui/multisig_build.go`.

Needles observed by the run (13 pushes, `Which other slot is yours?` twice):

    Supply or build a policy?, Choose policy type, How many keys (n)?,
    Which slot is your key?, Do you hold another slot?,
    Which other slot is yours? (x2), keys on cards?, Cosigner Keys,
    Payload cards, Use payload card, Where each key comes from:, Plate Count

### The census derivation — FROM THE INPUT TUPLE, never a literal (F-170)

The walk contains no expected plate count anywhere, and `ok` contains no
caller-supplied term (`TestWalkOkContainsNoDriverSuppliedPlateCount` covers all
five walk scripts). The census is derived twice, from the tuple, on the Go side:

* **`oracle.DeriveExpected`** computes the artifact set from
  `(template, n, k, per-slot origins, seeds, held_slots)` by invoking the
  primary: `ms derive` per slot → `md encode` for the policy and its id →
  `ms encode` per distinct held master → `mk encode` per held slot. 17 falls out
  of `len(...)`; nothing states it.
* **`oracle.CompareCensus`** compares that set against the engraved census byte
  for byte and IN ORDER, and refuses a comparison of nothing.

The only count the walk touches is the **device's own census screen**, read and
compared against the recorder's count — both terms the emulator's:

    Plate Count. This engraves 17 plates.
      ms1 secret share 1 of 2: 1 plate (secret seed backup)
      ms1 secret share 2 of 2: 1 plate (secret seed backup)
      mk1 key 1 of 3: 2 plates (account key card)
      mk1 key 2 of 3: 3 plates (account key card)
      mk1 key 3 of 3: 2 plates (account key card)
      md1 descriptor: 8 plates (wallet policy descriptor)

`censusClaim 17 === census.strings.length 17` → `censusHeld: true`.

### `shNFC.presented() === 0` (F-174)

Asserted at entry, at the cosigner gather, and after the restore doc; the final
value in the walk result is **0**. Four cards in the gather tally with zero
records across a *working* reader can only have come from the payload.

### What the walk asserts in the device's own words

The Key-sources review is read across all pages and required to contain:

    @1  yours: derived from your seed for @1, account 1
    @3  a cosigner: payload card 4, taken as supplied

The first is the multi-account assignment; a run that had collapsed @0 and @1
onto one account cannot draw it (and would hit §4.1's duplicate-key refusal).
Recorded screen, verbatim (squashed):

    Key sources  Where each key comes from:
    @0  yours: derived from your seed for @0
    @1  yours: derived from your seed for @1, account 1
    @2  yours: derived from your seed for @2
    @3  a cosigner: payload card 4, taken as supplied
    No slot claims to be both a seed and a card here, so nothing was
    cross-checked. The cosigner keys are taken as supplied.

### The keyboard — why the walk needed one, and how it is pinned

`cmd/emu`'s cards payload carries exactly **one** `ClassMnemonic` and
`syswSession.take` is **first-match and non-consuming**, so every "FROM PAYLOAD"
seed entry in one flow hands back the SAME master. Trace B holds two. F-181
recorded a keyboard driver as "genuinely optional"; it stopped being optional
here.

It types by **tapping at device coordinates** — no new emulator primitive, no
rune injection, nothing added to `gui`. `gui/keyboard_geometry_test.go` reads the
walk's own `KEY_PITCH`/`KEY_ROWS` out of `walk_trace_b.js` and types the whole
12-word phrase with them through `runUITouch`'s pointer events, asserting the
drawn word line after **every letter** and the assembled mnemonic at the end:

    typed all 12 words of the walk's phrase at the walk's own coordinates

F-181's two empirical data points, measured before the formula existed, agree
with it: `shTap(80,180)` typed Q (Q's rect is `(74,180)-(100,216)`) and
`shTap(300,200)` typed U (`(278,180)-(304,216)`).

---

## 4. RE-RUNNING THE THREE PRE-EXISTING WALKS — four findings

Each was run **individually**, in the browser, from a fresh page load.

### `walk_s4_gate.js` — was NOT broken, both arms green, unchanged

    pass arm: ok true, 7 plates, censusClaim 7 === cutCount 7, presented 0, 203 s
    fail arm: ok true, 0 plates cut, outcome "Key does not match seed",
              refusal {namesSlot: true, saysNothingEngraved: true,
                       saysSuppresses: true, namesHostRoute: true}, 17 s

### `walk_build_policy.js` and `walk_s3_nested.js` — BOTH BROKEN

Both were edited by a later block (they gained taps for S5's multi-select
picker) and **neither had been run since before S4 landed**. Four drifted
anchors, found one at a time because each fix got the walk one screen further.
Verbatim:

    FINDING 1 (both files). S4's buildSelfSourceFlow, drawn whenever the payload
    can supply n cards — the delivered blob carries FOUR against a default n=3.

      choosing omit fingerprints (row 0 of 2) did not land on "Scan a card, or
      Done": waitFor("Scan a card, or Done") timed out after 10000ms; screen
      reads "Isyour@0keyonacard?NO,JUSTMYSEEDYES,CHECKTHECARDYourkey"

    FINDING 2 (both files). The build path titles seed entry PER HELD SLOT
    ("Seed for @0"); seedEntryTitle ("Input Seed") is every OTHER program's.

      waitFor("Input Seed") timed out after 10000ms; screen reads
      "Wherefrom?FROMPAYLOADTYPEITSCANSeedfor@0"

    FINDING 3 (both files). S4's UNCONDITIONAL slot-source review, between the
    gate and assembly.

      none of ["Duplicate key","Policy stub"] appeared within 15000ms; screen
      reads "KeysourcesWhereeachkeycomesfrom:@0yours:derivedfromyourseedfor@0
      @1acosigner:payloadcard3,takenassupplied..."

    FINDING 4 (both files). S4's plate census, between the mode choice and the
    first plate.

      waitFor("Chooseengraving") timed out after 10000ms; screen reads
      "PlateCountThisengraves9plates.ms1secretshare:1plate(secretseedbackup)
      mk1key:2plates(accountkeycard)md1descriptor:6plates(walletpolicydescriptor)..."

**All four are DRIVER drift against screens that were reviewed and are correct.**
Nothing was relaxed in the product to make a walk pass; each fix answers a
screen the operator answers. Every one is commented at its call site with the
message it produced, so the cause is on record and not just the patch. Both
walks then re-ran green, individually:

    walk_build_policy.js  ok true, 9 plates, unattributed 0, censusClaim 9 held,
                          presented 0, 250 s, 9 needles proven
    walk_s3_nested.js     ok true, 9 plates, unattributed 0, censusClaim 9 held,
                          presented 0, 258 s, 11 needles proven,
                          namedOnRestoreDoc true, claimsLegacyToo false,
                          restore doc "Type: P2SH-P2WSH 2-of-3 multisig (sorted)"

Two consequences folded in rather than filed:

* **`ok` names its needles instead of counting them.** `proven.length === 7`
  stopped working the moment a walk had legs of different lengths (S4's
  slot-source question is drawn only on over-supply). Naming keeps the
  protection and says *which* needle went missing.
* **Both walks now read the plate-census screen and check its promise** against
  the recorder's own count.

**The transferable half.** CI runs `GOOS=js go vet` and the static needle checks
and **no walk at all**, so a walk can be edited, pass every Go test, and be
broken — which is the blind spot the plan's §5 names. Three of the four defects
were introduced by *S4*, a stage that closed green; its own walk was fine and
the two it did not own were not.

---

## 5. THE EMULATOR BINARY UNDER TEST

Served on a **fresh port (8791)** because the browser caches `emu.wasm`, and
every walk refuses to start if `shToolpath`/`shPace`/`shNFC.presented` is
missing (checked live before the first run: all present, `shPace()` = 2048).

**There is no size delta, and that is the honest answer rather than a missing
one.** Every Go change in this block is in a `_test.go` file, which `go build`
excludes, so the wasm cannot have moved. Proved rather than asserted: a build
from a clean checkout of the parent commit `023505c` (via `git worktree add`)
and the build from this working tree are byte-identical.

    built emu.wasm (9947743 bytes)
    09d3802a68284ef1ccb3cb7075285ba1cb7914fe1592e540d2388be3099adc85  pristine 023505c
    09d3802a68284ef1ccb3cb7075285ba1cb7914fe1592e540d2388be3099adc85  working tree

Before the build, `cmd/emu/emu.wasm` did not exist in the worktree, so no stale
binary from an earlier session could have been served.

---

## 6. F-190 — the needle counter now measures SCREENS, not source bytes

`cmd/emu/needle_flow_test.go` parses `gui`, finds the functions whose **string
literals** carry a needle, and walks **up the call graph** to the flows that can
draw it.

* **F-184 closes for free.** `go/parser` without `ParseComments` yields an AST
  with no comment text, and only `*ast.BasicLit` of kind STRING is inspected. "Do
  not quote a needle literal in `gui/` comments" stops being a rule to remember.
* **F-190 closes by attribution.** The discriminator is the **caller count**, not
  flow-ness. The first version stopped at the first `...Flow` ancestor and
  reported the shared gatherer's body as unique — the very screen F-169 measured
  as character-for-character identical across two programs. **The control test
  caught it**, which is what a control is for.

Measured — all 15 flow needles, one drawing flow each, file matching the pin:

    "Choose policy type"          -> buildParamPickFlow (gui/multisig_build.go)
    "How many keys (n)?"          -> buildParamPickFlow (gui/multisig_build.go)
    "Which slot is your key?"     -> multisigSelfSlotPickFlow (gui/multisig_build.go)
    "Do you hold another slot?"   -> multisigSelfSlotPickFlow (gui/multisig_build.go)
    "Which other slot is yours?"  -> multisigSelfSlotPickFlow (gui/multisig_build.go)
    "Cosigner Keys"               -> buildMultisigPolicyFlow (gui/multisig_build.go)
    "Supply or build a policy?"   -> engraveMultisigFlow (gui/multisig.go)
    "Payload cards"               -> buildPayloadReviewFlow (gui/multisig_build_payload.go)
    "Use payload card"            -> buildCosignerPickFlow (gui/multisig_build_payload.go)
    "BIP-48 for nested segwit…"   -> buildReviewFlow (gui/multisig_build.go)
    "key on a card?"              -> buildSelfSourceFlow (gui/multisig_build_slots.go)
    "keys on cards?"              -> buildSelfSourceFlow (gui/multisig_build_slots.go)
    "Where each key comes from:"  -> buildSlotSourceReviewFlow (gui/multisig_build_slots.go)
    "Key does not match seed"     -> buildMultisigPolicyFlow (gui/multisig_build.go)
    "Plate Count"                 -> buildMultisigPolicyFlow (gui/multisig_build.go)

Controls: the shared gather body resolves to **5** flows
(`buildMultisigPolicyFlow`, `bundleFlow`, `multisigVerifyFlow`,
`singleSigVerifyFlow`, `supplyMultisigPolicyFlow`); an impossible string to none;
a literal the caller passes in (`"Cosigner Keys"`) to exactly one.

**FINDING — one needle was reclassified by the measurement.** `"P2SH-P2WSH"` is
spelt in ONE production file, so the old counter called it unique. The flow
counter says **11** flows can draw it, because it reaches the restore doc and the
restore doc is shown by the build path *and* the supply path. It proves a
nested-segwit policy was **built** and proves nothing about which flow a walk is
in. It moves to a new `contentNeedles` list, which may only ever be asserted
**alongside** a flow needle — which is what `walk_s3_nested.js` already does. The
old substring counter is kept beside the new one because it is strictly wider (it
sees strings built by concatenation), and
`TestTheTwoCountersDisagreeOnlyWhereRecorded` pins the one place they differ so a
future edit cannot erase the finding.

**F-190's ask about the operator-facing titles is now unblocked but NOT done.**
The entry says that after fixing the counter, `"Plates To Cut"` vs
`"Plate Count"` "should be reconciled on UX grounds alone". That is a UX change
to two shipped screens; it is left for whoever owns the prose, and it no longer
has a test forcing it either way.

---

## 7. F-180 — the two cosigner rosters deliberately differ, and both files say so

    cmd/buildpayloadcards/main.go                A@0, A@1, B@0, C@0
    gui/multisig_build_payload_testdata_test.go  A@0, B@0, C@0, A@1, B@1

**Documented rather than re-ordered**, which is the option the entry allows,
because the order is load-bearing in both directions: the Go roster is
prefix-addressed (`cosignerCardRecords(t, n)` takes the first n and S1's tests
assert which card fills which slot), while the emulator blob is walked by tap
sequence and needs the two same-master accounts adjacent at the front so Trace B
can take them as @0 and @1. Both files now carry the twin note, and both state
the cost: reaching Trace A's B@0 + C@0 is **SKIP, SKIP** over the payload and
**SKIP, USE, USE** over the roster.

---

## 8. One change not in the brief, and why it was mandatory

`cmd/emu/gaterecord_anchor_test.go`'s
`TestGateRecordStringsAreRecordsOfTheCardsPayload` asserts that **every engraved
mk1 in a record is a verbatim record of the cards payload**. That is true of a
**gathered** bundle (Trace A) and **false of a built policy**: Trace B's key
cards are MINTED by the device under this policy's id `70fb9d6e`, while the
payload's cards carry `cmd/buildpayloadcards`' placeholder stub `5b48af35`. The
S5 record would have failed seven honest plates.

Rather than exempting the new kind — which would leave it anchored to nothing —
the test is now **kind-aware** (reading the kind from the inputs file beside the
record, the one place it is stated) and the built-policy arm asserts the
**converse**: no engraved mk1 may be a verbatim payload record, because a build
that shipped a gathered card as the operator's own key plate would be engraving
somebody else's card under this wallet's name. Both arms have their own
"checked nothing" fatal.

---

## 9. THE GATE — verbatim, unpiped, with true exit codes

    $ nix develop --command go test ./... -count=1
    EXIT=0
    ok  = 51,  FAIL = 0

    $ nix develop --command gofmt -l ./
    gofmt exit=0
    files listed: []

    $ rm -rf $GOCACHE && nix develop --command env GOCACHE=$GOCACHE go vet ./...
    EXIT=1
    findings: 40
    outside _test.go: 0            <- the stated clean baseline

    $ nix develop --command ./scripts/oracle-live.sh
    EXIT=0
       discovered 7 tagged test(s) from source
       --- PASS: TestLiveDerivationReproducesEveryCommittedExpectation
       --- PASS: TestRealPinsResolveTheInstalledOracles
       --- PASS: TestPinsAreCurrentWithTheirPrimaries
       --- PASS: TestBuiltPolicyDerivationMatchesTheS2Golden
       --- PASS: TestBuiltPolicyDerivesDivergentOrigins
       --- PASS: TestAssembledMd1MatchesThePrimaryByteForByte
       --- PASS: TestVendoredVectorsAreInSyncWithThePrimary
       live checks: PASS (exit 0)

`TestLiveDerivationReproducesEveryCommittedExpectation` now re-derives **this
record's** expectation live, so the mint is re-checked against the pinned
binaries rather than trusted.

Baseline at `023505c` was 51 ok / 0 FAIL, vet exit 1 / 40 / 0 — unchanged. The
suite grew four tests and lost none.

---

## 10. WHAT I COULD NOT DO / WHAT IS LEFT

1. **A `both`-slot Trace B was not walked.** Trace B as specified holds three
   `derived` slots; the walk answers "NO, JUST MY SEEDS" at S4's plural
   slot-source question. `walk_s4_gate.js` drives the `both` path (single-slot)
   and its plural multi-slot form is unexercised. The plan does not require it;
   naming it because a reader may assume otherwise.
2. **`TestGateStillFiresAfterOriginsDiverge` and `TestReRunMintsByteIdenticalPlates`
   (plan S5 tests 7 and 8) were not part of this block** and I did not check
   whether they exist. This block owned the walk, the mint and the follow-ups
   listed in the brief.
3. **F-190's UX half is open** (§6): the `"Plates To Cut"` / `"Plate Count"`
   titles are now free to be reconciled and were left alone.
4. **`walk_trace_a.js` was not re-run.** It is not one of the three the brief
   names and it drives the `Engrave Bundle` flow, not the build flow; S0's record
   remains its evidence.
5. **The `sh` (legacy) template and n=5 are unwalked.** Trace A and Trace B are
   two shapes, as the plan's §5 already says.
6. **No hardware.** S6 owns that, and Trace B's plan text requires it to be
   rehearsed there before S6 can close.

### For the controller to file, if wanted

* The four walk-drift findings above are worth one FOLLOWUPS entry as a class:
  **a walk that is edited without being run is a gate that has silently stopped
  existing**, and CI cannot see it. A cheap mitigation would be a per-block rule
  that any edited `walk_*.js` is re-run before the block closes — three of the
  four defects were introduced by S4, a stage that closed green.
