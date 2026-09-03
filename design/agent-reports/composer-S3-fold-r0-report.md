# composer S3 — fold of whole-diff review round 0

**Agent:** the S3 implementer, folding `design/agent-reports/composer-S3-exec-review-r0.md`
(1C / 2I / 5M / 3N) under `design/agent-briefs/composer-S3-fold-r0-brief.md`.

**Branch:** `composer-s3` in `/scratch/code/shibboleth/wt-composer-s3`.
**Base of this fold:** `a63fd1e` (the controller's census-title fold, kept).
**New tip: `27afa9fadd9e2c5ad6c5c53143d711c1fcfaa84a`.**

Nothing was pushed and nothing was flashed. No sub-agents were spawned. Both
trees are clean.

**Every finding is closed: 1C + 2I + 5M + 3N folded or filed, and a SECOND
CRITICAL that the I-2 fold uncovered is fixed with it.** The firmware size
step, unrunnable for the whole cycle, ran: Nix came back mid-fold and the
delta is measured below.

---

## 1. Commits

Three on `composer-s3` (`git log --oneline a63fd1e..HEAD`):

```
27afa9f gui: the five bounded Minors and Nits from review r0 (M-1, M-4, M-5, N-1, N-2)
83e932a gui: six more self-check fault rows, and the K/N read that made four presets unbuildable (review r0 I-2, and a Critical it found)
7edc863 gui: seating RESUMES, a Back at the mapping review lands on the last seated slot, and Move up discards (review r0 C-1, I-1)
```

Two in mnemonic-engrave on `master`:

```
b1a1985 followups: F-461 records the one self-check arm review r0's I-2 fold could not gate
db53513 spec: fold review r0's two documentation Minors -- the census refusal that does not exist, and three copy bodies section 8 never carried (M-2, M-3)
```

C-1 and I-1 share a commit because the review says they must: *"the natural fix
for C-1 (resume rather than re-ask) makes this one directly reachable, so they
should be fixed together or not at all."*

---

## 2. C-1 — seating re-entry offered zero sources · `7edc863`

**Both halves of the controller's decision, as given.**

**(a) `composerSeatFlow` RESUMES.** A slot that already holds a source is
skipped rather than re-asked, so the pick list is never filtered against
assignments that are about to be overwritten.

**(b) A Back at the mapping review lands on the last seated slot.**
`composerSeatingStep` no longer returns `false` up to `composerFlow`'s
`continue` (which restarted at the path list, conflating "back one screen" with
"start the composition again"). It releases the last seated slot and loops into
seating — the same back-step `composerSeatFlow` already does one level down.
With nothing seated there is no slot to land on and Back leaves the step, which
is the opening-screen rule one level up.

**Discarded assignments release their sources.** `composerReleaseSeat(st, i)` is
now the one place that drops an assignment and frees the source's `used`, used
by both the in-flow back-step and the new `composerReleaseLastSeat`. A seed
source is never marked `used` (one seed fills any number of slots, C12/§4f), so
there is nothing to release for it.

**Failing first — the review's own walk, through the real screens:**

```
PASS 2 slot @0 frame: "SeatkeysSlot@0,Path1key1of2:chooseakey Typeaseed Leaveunseated"
```

**Then passing**, with the frame naming `73c5da0a` again.

**The mutation the brief names — restore the re-ask from 0:**

```
--- FAIL: TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys
    composer_join_test.go:377: Back at the mapping review did not land on the last
    seated slot: seating re-asked from slot @0 instead of resuming at @1, so every
    earlier assignment is overwritten by a pass that cannot offer the sources those
    assignments still hold (SPEC §7d).
    Frame: "SeatkeysSlot@0,Path1key1of2:chooseakey73c5da0am/48h/0h/1h/2hTypeaseedLeaveunseated"
```

**One thing I had to strengthen, and it matters.** My first version of the walk
asserted only *"the second pass still names a fingerprint"*, exactly as the
brief specifies — and **the mutation survived it**. With half (b) in place the
Back releases slot @1, so even a re-ask from @0 finds one free source and the
frame still shows a fingerprint. The assertion that separates resuming from
re-asking is *which slot the Back lands on*: `Slot @1`, not `Slot @0`. That is
also the literal statement of the controller's decision, so the test now
encodes the decision rather than a side effect of it.

**Unit half:** `TestComposerSeatingReleasesASourceWhenItsAssignmentIsDropped`
asserts the released slot reads `src == -1`, its source is no longer `used`, a
neighbouring seat is untouched, and a second release of the same slot reports
nothing released.

---

## 3. I-1 — "Move up" reordered paths and discarded nothing · `7edc863`

The controller's third option, one line of behaviour: `composerMoveUp` swaps the
two paths and **discards unconditionally**, which is what §8j had already
promised the operator ("Every key you seated will be cleared"). It does not go
through `composerApplyShapeEdit`, and the comment says why: the signature
carries the wrapper, the path count and each path's key count — §7d's own list,
right for §7d's own enumerated edits, and blind to a reorder of two paths with
equal key counts. Move up is the one edit whose numbering effect the signature
cannot see, because it changes the *order* of paths and not their shape.

**Mutation — remove the unconditional discard:**

```
--- FAIL: TestComposerMoveUpDiscardsUnconditionally
    Move up discarded nothing on an equal-key-count swap, after §8j had already
    promised the operator every seated key would be cleared
    slot @0 still holds source 0 after a Move up      [... @1 @2 @3 @4 @5 ...]
    source 0 is still marked used after a Move up discarded every seat
```

The test opens with an INCONCLUSIVE guard that the fixture's swap really does
leave the signature identical, so it cannot pass by picking a fixture that
moves it.

---

## 4. I-2 — six more fault-injection rows · `83e932a`

The table gains a `want` substring per row, so **a row cannot pass by tripping a
different assertion earlier in the check** — which is how a coverage table comes
to report ten arms covered while four do the work. The four pre-r0 rows keep
their "refused at all" assertion; the six new ones name their arm.

Two arms needed a fixture the seated 2-of-3 cannot provide, and that is stated
in the code rather than worked around:

- **`composerLockedDigestFixture`** — the lock-VALUE and digest arms need a
  shape that *has* a lock and a digest; perturbing a nil one trips the count arm
  above them.
- **`composerCollidingOriginFixture`** — §4f's arm on the decoded md1 needs a
  chunk set the composer's own builder **refuses to emit**. It is driven by the
  shipped `md/testdata/template/wsh_sortedmulti.tmpl.md1.txt`, whose two slots
  both declare `m/48'/0'/0'/2'` with no fingerprint — exactly the template §4f
  says cannot be restored. Both slots are unseated, so the per-slot arms are
  skipped and §4f's is what fires.

**Each arm's `if false {` mutation, applied to the real source, now fails its
own named row:**

```
lock VALUE arm       -> FAIL .../a_path's_lock_VALUE_moves
sha256 digest arm    -> FAIL .../a_path's_sha256_digest_moves
unseated fp arm      -> FAIL .../an_UNSEATED_slot_declares_a_fingerprint
fp PRESENCE arm      -> FAIL .../a_seated_slot's_fingerprint_PRESENCE_differs
§4f invariant arm    -> FAIL .../the_decoded_md1_puts_two_slots_at_ONE_origin_with_no_fingerprints
use-site arm         -> *** SURVIVED ***   (filed, F-461 — see §6)
```

Every mutation was reverted and the tree verified clean after each.

### The sixth arm is filed, not faked — F-461

The use-site arm's **dispatch has no reachable input**. Measured while folding:
`md.ComposeWith` always emits the fixed `<0;1>/*`; all 61 vendored compose
vectors carry it; both readable `md/testdata/template/*.tmpl.md1.txt` fixtures
carry it; and `md`'s per-slot `useSiteOverrides` are unexported with no exported
constructor, so `composerSelfCheckFaultHook` — which rewrites *chunk strings* —
has nothing to rewrite them into.

What landed instead is
`TestComposerUseSiteGuardRefusesEveryShapeButTheFixedOne`, driving the predicate
`composerUseSiteIsFixed` in both directions over seven shapes (no multipath,
hardened wildcard, one alternative, three alternatives, wrong receive chain,
wrong change chain, a hardened alternative). **The rule is tested and the
dispatch is not**, and that distinction is in the test's own comment rather than
left for a later reviewer to rediscover. Filed as **F-461** with its
reproduction (`b1a1985`).

---

## 5. 🔴 A SECOND CRITICAL, found by writing I-2's missing control · `83e932a`

Adding the table's **honest control** — *does the self-check accept an honest
build?* — showed it does not.

**4 of the 12 offered (wrapper, preset) pairs were refused:**

```
--- FAIL: .../wsh/tiered-recovery      self-check: path 2 is 1-of-2 in the shape and 0-of-0 decoded
--- FAIL: .../wsh/decaying-multisig    self-check: path 1 is 2-of-2 in the shape and 0-of-0 decoded
--- FAIL: .../tr/tiered-recovery       self-check: path 2 is 1-of-2 in the shape and 0-of-0 decoded
--- FAIL: .../tr/decaying-multisig     self-check: path 1 is 2-of-2 in the shape and 0-of-0 decoded
```

An operator picking `tiered-recovery` or `decaying-multisig` — two of the six
§4d archetypes this stage ships — reached consent on a **correct** composition
and met §8q: *"The policy on this device does not match what you built. Go back
and check the path list, or start again."* The wallet could not be built on this
device at all, and the device was calling its own output wrong.

**Cause.** `md.Branch` documents K and N as set **only** for a branch that is
exactly a threshold over keys (`md/policy_shape.go:45-48`, *"Zero means 'not a
plain k-of-N' — NOT '1-of-1'"*), and §5 lowers a multi behind a timelock to
`and_v(v:multi(k,…),older(n))`, which is not one. `composerSelfCheck` read K/N
outside their own domain.

**Fix.** Compare the threshold where the codec reports one, and the key COUNT —
which `Branch.Keys` always carries — where it does not. This is not a weakening:
`Keys` is the strongest fact the decoded tree offers for that branch, and the
lock and digest that make the branch not-a-plain-threshold are themselves
compared by value immediately below.

**Rust-primary check, per CLAUDE.md.** `PolicyShape` is **fork-native** — there
is no `policy_shape.rs` in `md-codec` — so nothing is owed upstream: the
contract being misread is the Go type's own, and the misreader is fork GUI code.

**Guards, both directions.**
`TestComposerSelfCheckAcceptsEveryOfferedPresetsHonestBuild` runs **every**
offered pair (12 rows, all four wrappers), and
`TestComposerSelfCheckStillComparesKeyCountsUnderALock` proves the fix did not
turn a real disagreement into a pass — a shape claiming 4 keys against chunks
carrying 3, on a locked path, is still refused. Restoring the unconditional K/N
read fails the control on exactly those four pairs.

**Why nothing caught it.** Task A10 pinned the presets' **chunks** against the
Rust primary — byte-exact, first run — and never ran one through the **device's
own consent gate**. A preset's two facts, *right bytes* and *acceptable to this
device*, were proved separately and only the first was proved at all.

---

## 6. Minors and Nits

| # | disposition | where |
| --- | --- | --- |
| M-1 three lock bands untestable | **folded** | `27afa9f` |
| M-2 census ceiling refusal described but absent | **folded (spec)** | `db53513` |
| M-3 three copy bodies not §8 blockquotes | **folded (spec)** | `db53513` |
| M-4 same-xpub body outside the copy table | **folded** | `27afa9f` |
| M-5 `ErrComposeIndistinguishableSlots` unmapped | **folded** | `27afa9f` |
| N-1 re-mint identity fields unasserted | **folded** | `27afa9f` |
| N-2 C29 groups in map order | **folded** | `27afa9f` |
| N-3 `gofmt -l` lists five, not three | **no change needed — see below** |
| I-2's use-site arm | **filed, F-461** | `b1a1985` |

**M-1.** The three entry validators were closures inside `composerLockEdit` with
no caller a test could reach. Extracted as `composerBlocksBandEcho`,
`composerDaysBandEcho`, `composerHeightBandEcho`, and asserted at each ceiling,
one past it, each floor, one below it, plus the two empty-field hints that say
what to type rather than what is too much (§8u, journey M-2).

**M-2.** F-457 already owns the concrete-descriptor plate *and* the refusal that
guards it (its own text: *"the census's descriptor-ceiling refusal went with
it"*), so this stage does not own the implementation. §7f, §7g's divergence row
and §12 item 9 now say the refusal is **NOT IMPLEMENTED** and point at F-457,
noting the ceiling itself is measured (596) and ready when the plate is.

**M-3.** §8c now blockquotes **seven** bodies (gaining the blocks echo and the
packed-height bound) and §8t gains the date **ceiling** (F-456's), each with the
reason recorded in the section. Re-running the review's own spec-versus-table
diff after the fold:

```
spec blockquotes: 44   table rows: 41
NOT-IN-SPEC-BLOCKQUOTE  composerCopySameXpub  §7d  'Slots @0 and @1 hold the same key...'
```

The one remaining row is the body **M-4 moved into the table in the same round**,
and §11 explicitly admits that form: *"a blockquote in §8 **or a quoted string in
its table**"*. So all 41 rows are now legitimate spec diffs.

**M-4.** `composerCopySameXpub` is a real copy function with its own table row;
the declared-body count moves 40 → 41 and the AST scan's message moves with it.
Measured: *57 chars drawn in full, headroom 494*.

**N-3 — no change, and the reason.** All five entries are pre-existing on
**untouched fork main `321acb56`**, verified directly there, and `mt/` is absent
from `git diff --stat 321acb56..HEAD`. Neither of the two extra files is this
diff's, so there is nothing to gofmt. The review is right that the settled list
should say five.

**Every Minor/Nit guard fails its own mutation:**

```
blocks 65535 -> 65536        FAIL .../blocks_one_past
days   388   -> 389          FAIL .../days_one_past
height 499999999 -> 5e8      FAIL .../height_one_past
same-xpub table row deleted  FAIL "composerCopySameXpub is not in composerCopyTable"
M-5 sentinel arm deleted     FAIL "has no §8m/§8v arm ... draws the codec's own `md:`-prefixed text"
re-mint fingerprint zeroed   FAIL "carries fingerprint \"00000000\", the source card carries \"73c5da0a\""
N-2 group sort deleted       FAIL "render in a different order on run 0 (group 0 starts at slot @1, was @0)"
```

N-2's test runs the build 200 times, because a nondeterminism test that runs
once passes half the time by construction.

---

## 7. The gate set, as CI runs it

```
$ go test -count=1 -run '^TestComposer' -v ./gui/
  top-level PASS: 127   top-level FAIL: 0   sub-test PASS: 150
  ok  seedhammer.com/gui  1.597s

$ scripts/gui-shard-test.sh ./gui/ 24
  === wall: 26s ===
  RESULT: ok -- all 1185 tests ran across 24 shards

$ CGO_ENABLED=0 go test -count=1 -timeout 20m ./...
  exit=0 — 54 ok, 0 FAIL          (the cmd/emu needle suite included)

$ scripts/test-32bit.sh
  GOARCH=386 test:  exit 0 ; GOARCH=arm build: exit 0

$ go test -count=1 -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
  ok oracle | ok gui | ok sysw   [no tests to run]

$ GOOS=js GOARCH=wasm go vet ./cmd/emu/          (clean)
$ go test -count=1 -run Needle ./cmd/emu/        ok  seedhammer.com/cmd/emu  0.068s
$ go vet ./gui/    only the two pre-existing go1.25 ArtifactDir findings
$ gofmt -l .
  gui/transaction.go  gui/transaction_golden_test.go  gui/transaction_txrecord_test.go
  mt/mt.go  mt/mt_test.go        — all five pre-existing on 321acb56 (N-3)
```

Counts moved 116 → **127** top-level and 112 → **150** sub-tests; the sharded
package 1174 → **1185**. `go test ./...` was `-count=1`, so nothing is a cached
pass.

### The firmware size step RAN — it no longer cannot

The brief said to state that it cannot run. **It can now: `/nix` came back
mid-fold** (7,670 store paths; the controller's `f9d4f0b` records the
reinstall), so I ran it rather than reporting an unrun gate.

```
$ nix run .#build-firmware
  Built seedhammerii-v0.0.0-bg27afa9f.uf2                       exit=0

$ tinygo build -size short -o /dev/null -target pico-plus2 \
    -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
     code    data     bss |   flash     ram
  1548128   31796   31004 | 1579924   62800
```

| | flash | RAM |
| --- | --- | --- |
| baseline, fork `321acb56` (controller, `f9d4f0b`) | 1,506,884 B | 62,592 B |
| `composer-s3` @ `27afa9f` | **1,579,924 B** | **62,800 B** |
| delta | **+73,040 B** | **+208 B** |

**Non-zero, which is what the plan requires**: *"If the delta is zero, the door
wiring did not land and nothing composed is reachable — which is the same class
of defect as a gate that never ran."* The composer is reached from
`cmd/controller` through `walletPolicyFlow`, so the linker keeps all of it.
**This closes the one gate that had never executed in the whole S3 cycle.**

---

## 8. For the next reviewer

1. **§5, the new Critical.** It was invisible to a byte-exact pin against the
   Rust primary and to twelve green preset tests, and it made a third of the
   shipped archetypes unbuildable. The lens that found it was a two-line
   control: *does the honest build pass?* Worth asking of every gate this cycle
   added, not just this one.
2. **C-1's strengthened assertion (§2).** The brief's own regression assertion
   did not kill the brief's own mutation. Where a fix has two halves, an
   assertion that either half satisfies proves neither.
3. **F-461** is the only finding not closed by code, and its filing states the
   measurement that makes it unclosable today rather than deferring the
   question.
4. Nothing in this fold edited a pre-existing test except as the plan's named
   old→new replacements; the four rows the review's table already had are
   unchanged in behaviour, gaining only the `want` field.
