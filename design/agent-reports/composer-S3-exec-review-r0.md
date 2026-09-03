# composer S3 — independent adversarial execution review, round 0

**Subject:** fork branch `composer-s3`, worktree `/scratch/code/shibboleth/wt-composer-s3`,
base fork `main` `321acb56`, tip `a63fd1e` (26 commits = the implementer's 25 + the
controller's census-title fold). Plan: `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`
at mnemonic-engrave `722edbd`; spec `design/SPEC_wallet_policy_composer.md` §4–§9, §12, §13.

**Environment.** Go `/scratch/code/shibboleth/.toolchain/go/bin/go` (go1.26.7 linux/amd64),
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local TMPDIR=/scratch/code/shibboleth/.tmp`,
default `-mod=readonly`. `/nix` does not exist on this machine (confirmed:
`ls: cannot access '/nix': No such file or directory`), so the firmware size step is
**NOT RUN** — see lens 5. No sub-agents were spawned. Every mutation and every probe file
was reverted; `git status --porcelain` is empty and `go test -count=1 -run '^TestComposer' ./gui/`
is `ok seedhammer.com/gui 6.084s` at the end of this review.

**Counts: 1 Critical, 2 Important, 5 Minor, 3 Nit.**

---

## Lens 5 — whole-repo gates as CI runs them (`.github/workflows/test.yml`)

The controller's own run was **entirely cached** (`grep -c cached` = 54 of 54 `ok` lines),
so every gate below was re-executed with `-count=1` or through the sharded runner.

```
$ go test -timeout 20m ./...                      # 54 ok, 0 FAIL — ALL CACHED, re-run below
$ go test -count=1 -timeout 20m $(go list ./... | grep -v '/gui$')   EXIT=0
$ scripts/gui-shard-test.sh ./gui/ 24
  === wall: 43s ===
  RESULT: ok -- all 1174 tests ran across 24 shards        EXIT=0
$ go test -count=1 -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
  ok seedhammer.com/oracle | ok seedhammer.com/gui | ok seedhammer.com/sysw  [no tests to run]  EXIT=0
$ scripts/test-32bit.sh
  GOARCH=386 test:  exit 0 ; GOARCH=arm build: exit 0      EXIT=0
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/                    EXIT=0
$ gofmt -l .
  gui/transaction.go  gui/transaction_golden_test.go  gui/transaction_txrecord_test.go
  mt/mt.go  mt/mt_test.go
$ go test -count=1 -run Needle -v ./cmd/emu/               7/7 PASS
```

**Firmware size (`nix run .#build-firmware` + the `tinygo build -size short` line): NOT RUN.**
Neither Nix nor TinyGo exists on this machine. No substitute was used and no size number is
reported. The operator reinstalls Nix before anything is flashed.

The three `gui/transaction*` gofmt entries are settled per the brief. `mt/mt.go` and
`mt/mt_test.go` are **also** unformatted and are **also** untouched by this diff (they appear
nowhere in `git diff --stat 321acb56..a63fd1e`), so they are pre-existing too — recorded as
N-3 only because the brief's settled list named three files and the real count is five.

---

## Lens 4 — what the diff made false elsewhere

**Task C1's four spec numbers re-measured, not read:**

```
$ go test -count=1 -run '^TestComposerMeasureSection13Numbers' -v ./gui/
  SPEC13 stub_screen    lines= 42 per_frame= 7 pages=6
  SPEC13 pick_list      lines= 36 per_frame= 7 pages=6
  SPEC13 consent        lines= 17 per_frame= 7 pages=3
  SPEC13 descriptor_plate ceiling_chars=596  c10_688_fits=false
```

Byte-for-byte what §13 item 1 now carries. **Correct.**

**§8 copy, machine-diffed against the spec's own blockquotes** (script:
extract every `>` paragraph from spec §8, whitespace-normalise, set-diff against the
`verbatim` column of `composerCopyTable()`):

```
spec blockquotes: 41   table rows: 40
NOT-IN-SPEC-BLOCKQUOTE  composerCopyLockEchoBlocks     §8c  '1000 blocks (about 6.9 days)'
NOT-IN-SPEC-BLOCKQUOTE  composerCopyPackedHeightBound  §8c  'This device cannot tell the time. The payload says the packed height was 905000, ...'
NOT-IN-SPEC-BLOCKQUOTE  composerCopyDateCeiling        §8t  'This build writes dates up to 2038-01-19. ...'
SPEC-BODY-UNCLAIMED  ×4  — the four §8n HOST lines (`me sysw pack` stderr, S1's, not this diff)
```

**37 of 40 rows are byte-verbatim from the spec.** The three exceptions are M-3.

**Shipped-file edits.** `gui/gui.go`, `gui/sysw_admit.go`, `gui/multisig_build.go`,
`gui/template_engrave.go`, `gui/wallet_policy.go` each retire a premise C12/C7 falsified
rather than moving it — the `sysw_admit` row keeps FreeText and Address refused, which the
admission test checks alongside the additions. The three `cmd/emu/shots_*.js` gain exactly one
`tap(CONFIRM)` for the door, which is correct: the door's cursor starts on "Scan cards".

**The controller's fold `a63fd1e` verified independently.** Its claim — that "Plate Count" is
a registered single-site walk needle and "Plates To Cut" is pinned by no walk — reproduces:

```
$ grep -rn '"Plates To Cut"\|"Plate Count"' --include=*.go . | grep -v _test.go
  gui/multisig_build.go:495  "Plate Count"      <- cmd/emu/needle_test.go:108 registers it
  gui/multisig.go:282        "Plates To Cut"
  gui/singlesig.go:236       "Plates To Cut"
  gui/composer_flow.go:295   "Plates To Cut"
$ go test -count=1 -run Needle ./cmd/emu/      ok (7/7)
```

**FOLLOWUPS the plan claims to close.** F-453 is marked CLOSED and the vendored corpus pin
resolves to it exactly (`compose_vectors.provenance.json`: commit `1dc8d409…`, clean, 156 files,
32 vectors). F-458's fix is real and tested (see lens 3). F-455 and F-457 are filed as *later
cycles* and the code matches the filing — but F-457 leaves §7g's "concrete descriptor longer
than the plate holds → REFUSAL by census" describing a refusal the device has no path to (M-2).

---

## Lens 2 — the end-to-end walk and byte identity against the Rust primary

**Byte identity, four wrappers, against `md compose` built from descriptor-mnemonic
`1dc8d409` (`cargo build --release -p md-cli`).** The host CLI emits the *single-string*
form for short policies, which is not the chunk form §9 item 1 names, so `--force-chunked`
is the correct comparison; using the default form is what makes a false mismatch:

| shape | Rust `md compose --json` → `md encode --force-chunked` | Go `composerTemplateChunksFor` |
| --- | --- | --- |
| `wsh 2of3` | `md1f6frtqq9qjtvyyy5jmpprjjtvyy49gqpsgwzyxqqzqw55nlnzfdw3` | identical |
| `tr 2of3 + 1of1,older=26280` | `md1frk8kqq9q6tvyyykjmpprj6tvyy495kcgfwtsqrqw9yq3pjv62msqqqe4gqs6s700nzfxf6l` | identical |
| `sh-wsh 2of3` | `md1fhet3qq9qjtvyyyd9kzz8rfdssj5wqqvrppcgscqnfmcvj263pa86` | identical |
| `sh 2of2` | `md1f6ds9qq9q2tvyyy5jmpprj5qqcx8ppgqa3nwjkmv5ljq5` | identical |
| `wsh 1of1 + 1of1,after=1000000` | `md1fvucsqq9q2tvyyy5jmpprj5qqcye9jv6tkcqpapyqarpjfzwj5d7t8` | identical |
| `wsh 1of1 + keyless,sha256=ab…ab` (2 chunks) | `md1f4zfeqspqztvyyy4qqxpxfdm2at4w46h2at4w46h2agyqjatkrfmlfsa md1f4zfeqs02at4w46h2at4w46h2at4w46h2at4w46kq8zgg7twzyg6mn` | identical |

**Six shapes, four wrappers, keyless and locked and hashed — byte-identical. No finding.**

**The taproot slot-numbering question the plan named as reviewer budget** ("whether
`composerSlotOrder` really tracks §5's numbering for a taproot list whose first single-key path
is not first-listed") — cross-checked against the Rust `slots` map, which is an independent
oracle for exactly this:

```
$ md compose --wrapper tr --path 2of3 --path 1of1 --path 1of2 --json
  ik_path 1 ; slot0=path1 slot1..3=path0 slot4..5=path2
$ md compose --wrapper tr --path 2of3 --path 1of1,older=100 --path 1of2 --path 1of1 --json
  ik_path 3 ; slot0=path3 slot1..3=path0 slot4=path1 slot5..6=path2
```

`composerSlotOrder` reproduces both maps exactly, and §8s's prompts read
`Slot @0, key path (spends alone): choose a key` / `Slot @1, Path 1 key 1 of 3: choose a key`
— the OPERATOR's listed index, never the leaf index, in the hard case. `composerSelfCheck`
accepts both honest builds. **No finding.**

**Part A's acceptance (§12 item 3)** is `TestComposerNoPayloadWalkEngravesAKeylessTemplate`,
which drives `walletPolicyFlow` through the door, the wrapper, the preset decline, the shape,
the paged stub screen (paging until `expects a key at` appears), the collapsed form choice, the
census and the engrave. It passes and its assertions bite (see lens 3's census mutation).

**The emulator journey (§12 item 2) does not exist and is not this stage's.** The plan says so
explicitly at line 11763 — "no emulator journey (§12 item 2 is S4's, and it is the gate a plan
may not close while it has never run -- this plan does not claim it)" — and at line 11734 the
`shots_*.js` edit is stated as a build-and-count check, not a run. **Not a finding against this
diff**, recorded so the next reviewer does not look for it.

---

## Lens 1 — counterexamples at every §4/§4e/§6b bound

Every structural bound was driven through `md.ValidatePathList` + `composerRefusalBody` +
`composerTemplateChunksFor` (probe, since reverted). **Nothing was accepted that should have
been refused, and nothing panicked:**

```
wsh k>n (3of2)            refused    wsh n=9  (max)       ACCEPTED   wsh 32 slots (max)      ACCEPTED
wsh k=0                   refused    wsh n=10 (one past)  refused    wsh 33 slots (one past) refused §8m-5
wsh n=0                   refused    wsh 8 paths (max)    ACCEPTED   empty list              refused §8m-1
lock older(0x400000)=0u   refused    wsh 9 paths (past)   refused    no keyed path           refused §8m-1
lock after(0)             refused    sh n=1               refused §8m-4   lock-only path     refused §8m-2
lock after 2147483647     ACCEPTED   sh 2 paths           refused §8m-4   keyless under tr   refused §8m-3
lock after 2147483648     refused    sh locked / sh hashed / sh-wsh n=1   refused §8m-4
```

All five §8m bodies map from their sentinel and all five are reachable from the real screens
(`TestComposerSection8mRefusalsAllDrawThroughTheRealPath`). The unmapped refusals (k>n, n>9,
>8 paths, out-of-band lock) are all unreachable from the flow — `composerCountPick(…, 1, n)`
bounds k, `composerMaxKeysForPath` bounds n, the "Add a spend path" row disappears at the cap,
and `composerLockAccept` catches an out-of-band operand with its own body.

**Date entry (§6b, F-458):** `composerDateExists` is a `time.Date` round trip, so 2027-02-31
(which `time.Date` normalises to 2027-03-03) is told apart from the 2009 floor and the
2038-01-19 ceiling by *what it is*, not by reading a returned operand. Verified by mutation.
`composerDateCeilingUnix = 2147472000` is 2038-01-19 00:00:00 UTC — correct.

**§4f's unseated-account rule under partial seating** — the case where a seated slot's declared
account could collide with a codec-assigned one:

```
seat@acct0: @0=m/48h/0h/0h/2h(fp) @1=m/48h/0h/1h/2h @2=m/48h/0h/2h/2h
seat@acct1: @0=m/48h/0h/1h/2h(fp) @1=m/48h/0h/0h/2h @2=m/48h/0h/2h/2h
seat@acct2: @0=m/48h/0h/2h/2h(fp) @1=m/48h/0h/0h/2h @2=m/48h/0h/1h/2h
seat@acct5: @0=m/48h/0h/5h/2h(fp) @1=m/48h/0h/0h/2h @2=m/48h/0h/1h/2h
```

The codec skips the seated account in every case, `composerSelfCheck` accepts every emitted
template, and forcing the same account twice is refused by both layers
(`md: compose: two slots declare the same origin without two distinct fingerprints … slots @0 and @1`).
**No finding.**

**Screen-capacity counterexamples** (`ChoiceScreen` neither wraps its rows nor scrolls, and its
Lead sits in a fixed 44 px band): measured at `sh2DisplaySize` (480×320, content box 232 px,
one choice row 28 px). The 6-row preset picker is 168 px — fits. Every other composer
`ChoiceScreen` is 2–5 rows. The door's Lead at its worst joined form
(`"Keys loaded: 32, plus 9 seeds. 99 payload records were not understood."`) is 44 px against
44 px — fits, exactly, and the "payload in flash but not loaded" state returns *early* from
`composerDoorLines`, so it can never be joined to the not-understood line (which would be 65 px
and would overflow). **No finding, but the margin is zero.**

**The one finding this lens produced is C-1**, below: it is not a bound, it is a state.

---

## Lens 3 — mutation-testing the tests

Harness: back up the file, `sed` one guard, `go test -count=1 -run '^TestComposer' ./gui/`,
restore. A mutation that fails to apply, or that fails to compile, is not counted as a kill —
two of my first attempts produced `declared and not used` and were redone semantically.

### Killed (the guard has a test that can fail)

| mutation | killed by |
| --- | --- |
| `composerDateExists`: `return t.Year() >= 0` (accept every normalised date — the F-458 regression) | `TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate/an_impossible_date_inside_the_band`, `TestComposerDateCeilingAndImpossibleDateAreToldApart/20270231`, `TestComposerDateEntryRefusesImpossibleAndPre2009Dates/20270231` |
| date floor `1230940800` → `1230854400` (one day) | `TestComposerDateEntryRefusesImpossibleAndPre2009Dates/20090102` |
| `composerShapeSignature`: zero the per-path key count | `TestComposerShapeSignatureMovesExactlyWithSlotNumbering` |
| `composerShapeSignature`: drop the wrapper term | `TestComposerShapeSignatureMovesExactlyWithSlotNumbering`, `TestComposerChangeTheScriptRowRewrapsAndDiscards` |
| `composerSharedSeedBody`: `>=` → `>` (§8g body choice) | `TestComposerC29WarningFiresInsideOnePathAndNotAcross` |
| `composerInvariantViolation`: drop the asymmetric-fingerprint arm | 4 tests, incl. `TestComposerNeverProducesTheAsymmetricOneCardTemplate` |
| `composerSortedIsLegal`: drop the sole-path condition | `TestComposerSortedIsLegalOnlyWhereSection5SaysSo` |
| `composerFormsFor`: offer form A when partially seated | `TestComposerFormsForOfferWhatSection7fAllows/partially_seated_offers_no_form_A` |
| `composerSelfCheck`: drop the fingerprint VALUE compare | `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput/a_slot's_fingerprint_moves` |
| `composerMintCard`: `mk.Encode(card)` (drop both stubs) | the card suite |
| `composerCensusLines`: `buildPlateCensusLines(params, nil)` | `TestComposerNoPayloadWalkEngravesAKeylessTemplate`, `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen` |

### Survivors

Ten mutations survived. Six are in one function (I-2), three in one (M-1), one is N-1.

---

# Findings

## C-1 — Re-entering seating after any Back past the seating step offers **zero** sources, so §7d's "Back keeps assignments" is unmet and the keyed policy becomes unreachable

`gui/composer_seat.go:53-124` (`composerSeatFlow`), with
`gui/composer_flow.go:97-115` (`composerSeatingStep` → `continue`).

`composerSeatFlow` marks each consumed `key:`/mk1 source `used = true` and never releases it
except on its own one-slot back-step. But every Back *after* seating —
at the mapping review (`composerMappingReview` → `composerReadScreen` Back), at consent
(`composerConsentFlow`), at §8l, at the form pick, at the census — returns `false` up to
`composerFlow`, which does `continue` back to the shape. Done then re-enters
`composerSeatingStep`, and `composerSeatFlow` **re-asks every slot from index 0** while the
pick list filters out every `used` source. The rows are `["Type a seed", "Leave unseated"]`.

`composerDiscardAssignments` is the only thing that clears `used`, and it runs only from
`composerApplyShapeEdit` when the shape *signature* moved — i.e. only on an edit the operator
had no reason to make.

**Reproduction (end to end, through the real screens; probe since reverted):**

```go
ctx.sysw = composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil)
runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
// wsh -> decline preset -> add path -> 2 keys -> 2 of 2 -> Done -> Sorted -> stub screen
// seat @0, seat @1 -> mapping review -> Button1 (Back) -> path list -> Done -> stub -> seating
```

```
PASS 1 slot @0 frame: "SeatkeysSlot@0,Path1key1of2:chooseakey
                       73c5da0am/48h/0h/0h/2h  73c5da0am/48h/0h/1h/2h  Typeaseed Leaveunseated"
mapping review:       "Keymapping @0:73c5da0am/48'/0'/0'/2' @1:73c5da0am/48'/0'/1'/2' ..."
PASS 2 slot @0 frame: "SeatkeysSlot@0,Path1key1of2:chooseakey Typeaseed Leaveunseated"
```

**Why Critical.** §7d states the guarantee twice — "Back keeps assignments" at the mapping
review, and "A mapping-review screen … precedes consent; Back keeps assignments." The
assignments survive in `st.assigned` but are unconditionally overwritten by a re-ask that
cannot offer the operator's own keys back. From that state the only exits are a key-less or
partially-seated template (§8p's fallback), a typed seed, or an unrelated shape edit that
happens to move the signature — none of them signposted, and the one the operator wants
(the policy they just reviewed) is gone. That is an unmet guarantee and a state the operator
cannot leave by any route the device names.

**Hypothesis (not prescriptive).** Two independent halves: (a) release the sources an
assignment holds when seating is re-entered — or, better, do not re-ask a slot that already
holds a source, so `composerSeatFlow` resumes at the first `src < 0`; and (b) `composerFlow`'s
`continue` after `composerSeatingStep` returns `false` currently conflates "Back one screen"
with "restart from the shape" — a Back at the mapping review should land on the last slot, as
`composerSeatFlow`'s own back-step already does one level down. Whichever is chosen, the
regression test is the walk above: assert the second pass's frame still names a fingerprint.

## I-1 — "Move up" reorders the spend paths without moving the shape signature, so §8j fires and clears nothing

`gui/composer_shape.go:322-330` (the `Move up` arm) and
`gui/composer_discard.go:27-37` (`composerShapeSignature`).

`composerShapeSignature` captures the wrapper, the path count and each path's key count —
which is exactly §7d's list, and correct for §7d's enumerated edits. But **"Move up" is an
affordance this diff adds that §7b/§7d do not describe**, and reordering two paths with the
*same* key count leaves the signature identical, so `composerApplyShapeEdit` discards nothing —
*after* `composerShapeGuard` has already drawn §8j: "Slot numbers change with the shape. Every
key you seated will be cleared. Continue?"

**Reproduction:**

```go
list := wsh{ {2of3}, {1of3, older(1000)} }          // equal key counts
st.assigned = [seated, seated, seated, seated, seated, seated]
composerApplyShapeEdit(st, func() { swap(paths[0], paths[1]) })
// signature before="w1/3,3," after="w1/3,3."  discarded=false  anyAssigned=true
```

§5 renumbers slots by first appearance in the emitted text, and that text follows *listed
order*, so after the swap slot @0..@2 denote the formerly-second path. The retained assignments
now mean different spend paths — the family's keys behind the timelock, the recovery keys
spending immediately — with no screen saying so and no refusal firing (`composerSelfCheck`
compares the decoded md1 against `st.list`, which moved with it, so it agrees).

**Why Important and not Critical.** Today the wrong state is not directly engravable: seating
is re-entered after the edit and `composerSeatFlow` overwrites every `st.assigned[i]` (which is
C-1). The two findings are coupled — the natural fix for C-1 (resume rather than re-ask) makes
this one directly reachable, so they should be fixed together or not at all.

**Hypothesis.** Either add the path *order* to `composerShapeSignature` (a positional term, not
a multiset — `fmt.Fprintf(&b, "%d:%d,", i, n)` is not enough since the counts are equal; the
term has to be an identity, e.g. an index assigned when the path is created), or gate the
`Move up` arm on `!composerAnySlotAssigned(st)`, or discard unconditionally on that arm. The
third is one line and matches what §8j has already promised the operator.

## I-2 — Six of `composerSelfCheck`'s ten assertions have no test that can fail; §12 item 4's fault injection covers four

`gui/composer_selfcheck.go`. §7e enumerates what the check asserts — "the decoded shape, the
slot assignment, every slot's origin and fingerprint (against the mapping review), the fixed
use-site, and §4f's pairwise-distinguishability invariant" — and §12 item 4 makes fault
injection the way that is proved. `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput`
carries **four** cases: origin moves, fingerprint value moves, the shape gains a path, the
chunks are another wallet's.

Removing each of these arms leaves `go test -run '^TestComposer' ./gui/` **green**:

| line | arm | mutation | result |
| --- | --- | --- | --- |
| `:103` | lock kind + value | `if false {` | **SURVIVED** |
| `:115` | sha256 digest | `if false {` | **SURVIVED** |
| `:130` | fixed `<0;1>/*` use-site | `if false {` | survived every behavioural test; killed only by `TestComposerEveryScreenFunctionHasAProductionCaller` reporting `composerUseSiteIsFixed` as dead — a reachability artefact of the mutation, not a check on the guard |
| `:137` | an unseated slot declares a fingerprint | `if false {` | **SURVIVED** |
| `:146` | fingerprint *presence* differs | `if false {` | **SURVIVED** |
| `:173` | §4f's invariant on the decoded md1 | `if false {` | **SURVIVED** |

The guards themselves are correct — driven directly they all refuse:

```
lock arm:        self-check: path 2's lock is 0/1001 in the shape and 0/1000 decoded
digest arm:      self-check: path 2's digest differs from the shape's
fp-presence arm: self-check: slot @0's fingerprint presence differs from the mapping review
```

So this is a **coverage** finding, not a broken gate. It matters because the lock operand and
the digest are precisely the two values §7c says enter the template id and that a builder
defect would move, and because §4f's invariant *on the decoded md1* is the only place the
invariant is checked for a partially seated or key-less template —
`composerInvariantViolation` deliberately skips unseated slots and says so. A guard nobody can
break silently is a guard the next refactor deletes.

**Hypothesis.** Six more rows in the existing table, each mutating `st.list` or `st.assigned`
away from the honest fixture (a lock value, a digest byte, `fpPresent` on a seated slot,
`fpPresent` on an unseated one) plus, for the use-site and the §4f arm, a `chunks` mutation —
`composerSelfCheckFaultHook` already exists for exactly that and is correctly reset
(`defer func(){ composerSelfCheckFaultHook = nil }()`, with an at-rest assertion at `:142`).

## M-1 — Three lock entry-validator bands have no test that can fail

`gui/composer_lock.go:216` (blocks `> 65535`), `:232` (days `> 388`), `:292`
(height `> 499_999_999`). Widening each by one leaves the whole `^TestComposer` suite green:

```
n > 65535        -> n > 65536         SURVIVED
n > 388          -> n > 389           SURVIVED
n > 499_999_999  -> n > 500_000_000   SURVIVED
```

§12 item 7 is satisfied — the *emitter's* gate is `md.Lock.Check` and it is tested at every
§4c boundary by both `TestComposerLockCheckRefusesEverySection4cBoundary` and
`md`'s `TestLockCheckIsTheDeviceSideRangeGate`, and I confirmed the entry bands are tighter
than or equal to `Lock.operand()`'s in all four kinds, so no value slips through. What a drift
in these three would cost is the *copy*: `composerLockAccept` would catch it and draw
"This device will not write that lock value." — the builder-defect line — instead of §8u or the
band hint. Minor, and cheap to close: three table rows at 65535/65536, 388/389,
499999999/500000000 asserting the validator's own `(string, bool)`.

## M-2 — §7f/§7g/§12 item 9's census descriptor-ceiling refusal does not exist, and §7g still describes it

`gui/composer_census.go:74-84` removes it deliberately, with a stated reason (form A ships as
the keyed md1 because `md` emits no descriptor text; the renderer is Rust-first, F-457), and
`composerDescriptorCeilingChars` survives only as §13 item 1's measurement. The reasoning holds
and the follow-up is filed with an owning phase. But §7g's divergence table still carries
"engrave | concrete descriptor longer than the plate holds | REFUSAL by census with the
measured ceiling (§13 item 1)", and §12 item 9 says that refusal "is asserted once §13 item 1
has a number" — which it now does (596). The spec has not been folded, so a reader of §7g looks
for a refusal the device has no path to. Documentation only; nothing an operator can reach.

## M-3 — Three of the 40 copy bodies are not §8 blockquotes, and two of those cite §8c for text §8c does not carry

`gui/composer_copy_test.go` rows: `composerCopyDateCeiling` (§8t) is a **new 40th body** with no
spec blockquote at all — correctly filed as F-456, correctly reasoned in the code comment, and
the `declared != 40` guard names it. `composerCopyLockEchoBlocks` and
`composerCopyPackedHeightBound` are tagged §8c but §8c blockquotes five bodies and neither is
among them: the blocks echo comes from §6b's table row ("N blocks (about D days)") and the
height-bound body is §8c's date form spliced with §6b's "heights read `the packed height was H`"
— which the code comment states plainly. The effect is that
`TestComposerCopyIsVerbatimFromTheSpec` compares three rows against text the spec does not
contain, so for those three it is a self-consistency check, not a spec check. Fold §8 (F-456
already owns one third of it) and the table becomes a diff against the spec again for all 40.

## M-4 — The same-xpub refusal body is outside `composerCopyTable`, so §12 item 5's four gates do not reach it

`gui/composer_review.go:222-223`:

```go
showError(ctx, th, "Key mapping", fmt.Sprintf(
    "Slots @%d and @%d hold the same key. Every slot needs a different key.", a, b))
```

§7d requires this refusal and §11 says "the copy of each refusal is a blockquote in §8 or a
quoted string in its table, so the glyph and modal-fits gates cover it" — this string is
neither, and `TestComposerCopyTableCoversEveryBody` only scans `composerCopy*` declarations, so
nothing counts it. I measured it: `assertModalBodyFits` reports *57 chars drawn in full,
headroom 494* — it fits today. Every other ad-hoc composer body I measured fits too (the form
pick's two leads, the census error-correction line, the F-217 note, the four digit-pad hints,
the three failure lines). So this is a gate-coverage gap, not a truncation.

## M-5 — `ErrComposeIndistinguishableSlots` has no `composerRefusalBody` arm, so §8v's own sentinel would print raw codec text

`gui/composer_shape.go:34-52`. The five §8m sentinels map; `md.ErrComposeIndistinguishableSlots`
does not, even though §8v is the body for exactly that condition. `composerShowRefusal` would
therefore draw `md: compose: two slots declare the same origin without two distinct
fingerprints; a template like that cannot be restored: slots @0 and @1` — an internal `md:`
prefix on an operator screen, which §11 forbids. Unreachable today, because
`composerInvariantViolation` refuses at the mapping review before `composerArtifactsFor` runs
and the unseated case cannot collide (verified in lens 1). One `case errors.Is(err,
md.ErrComposeIndistinguishableSlots): return composerCopySameOriginFewFingerprints(), true`
closes it.

## N-1 — The re-mint path's carried fingerprint and xpub are not asserted

`gui/composer_cards.go:48-52`. `card = src.card; card.Fingerprint = "00000000"` **survives** the
whole `^TestComposer` suite. `TestComposerReMintPreservesExistingStubsInOrder` asserts stub
count, stub order and the append-once property, and nothing asserts the identity fields the
plate actually carries. The code is correct — `a.fingerprint`/`a.xpub` come from
`src.fingerprint`/`src.xpub` at seating, so they agree with `src.card` by construction — but the
assertion is missing.

## N-2 — Multiple C29 groups in one path render in map-iteration order

`gui/composer_review.go:118-133`: `for _, slots := range byPath[i+1]` iterates a
`map[[4]byte][]uint8`, so with two distinct fingerprints each duplicated inside one path the two
§8g bodies appear in a nondeterministic order on the mapping review. The slots *within* a group
are appended in ascending index and are stable. Cosmetic and rare; sorting the group keys makes
the screen reproducible, which matters for a screenshot walk.

## N-3 — `gofmt -l` lists five files, not the three the brief settled

`mt/mt.go` and `mt/mt_test.go` join `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`. Both are absent from
`git diff --stat 321acb56..a63fd1e`, so they are pre-existing on fork `main` exactly as the
three named ones are, and `test.yml` has no `gofmt` step. Recorded only so the settled list is
right next time.

---

## What I checked and found nothing

- **`composerSlotOrder` vs the Rust slot map**, including the taproot internal-key-not-first
  case the plan flagged as reviewer budget — exact, both shapes.
- **Byte identity of the composed template chunks vs `md compose` + `md encode --force-chunked`**
  at `1dc8d409` — six shapes, four wrappers, keyless and locked and hashed, all identical.
- **Every §4b/§4e/§4c structural bound and one past it** — 22 cases, no accepted-but-wrong
  shape, no panic, all five §8m bodies mapped and reachable.
- **§4f's unseated-account rule under partial seating** at four different seated accounts — the
  codec skips the seated account every time and the self-check accepts the result.
- **The vendored preset corpus** — the provenance pin resolves to `1dc8d409` with 156 files /
  32 vectors, six of the twelve offered (wrapper, preset) pairs are pinned against the primary's
  own export, and the other six are *named* as structurally-checked rather than counted as
  pinned.
- **`ChoiceScreen` capacity** for every composer screen (rows and Lead) at the SH2 display size
   — nothing overflows, though the door's worst-case Lead is 44 px against a 44 px band.
- **Test-hook globals** (`composerSelfCheckFaultHook`, `composerSeedHook`) — both reset by
  `defer` with an at-rest assertion, so the shared-state bleed that CI's threaded runner exposes
  elsewhere does not apply here.
- **The controller's `a63fd1e` fold** — its needle claim reproduces and the needle suite is 7/7.
- **§13 item 1's four numbers** — re-measured, identical to what the spec now carries.
- **The scrub seam** — `defer st.reg.scrub()` is installed at `composerFlow`'s top, before any
  seed exists, so every exit is covered by construction. (Secret-handling is non-gating in any
  case by the 2026-08-27 ruling; nothing here needed a follow-up.)

**Not run, stated as such:** the firmware size recipe (`nix run .#build-firmware`, then
`tinygo build -size short …`). `/nix` does not exist on this machine and TinyGo is not
installed. No substitute measurement was made.
