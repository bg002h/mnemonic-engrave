# Hashlock H6 — implementer F report (Tasks 12 and 13)

**Written 2026-09-06 as this agent's final action.** Verbatim; nothing here has
been read or filtered by the controller.

| repo | branch | tip | base |
| --- | --- | --- | --- |
| fork (`seedhammer`) | `h6-f` | **`e089a53917f92c44624fa72617e0d4a67a7f7050`** | `hashlock-h6` `bdb66963` |
| engrave (`mnemonic-engrave`) | `h6-records` | **`32a4686064748db629edb441cc2482097228cb90`** | `master` `5e7211d0` |
| toolkit (`mnemonic-toolkit`) | `h6-manual` | **`4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a`** | `master` `6cb55bb8` |

All three worktrees are CLEAN at those tips (`git status --short` empty).
Nothing pushed. No commit on any `main`/`master`.

| task | commit | what |
| --- | --- | --- |
| 12 | fork `72b5a757c9084f323d2e32770ccd277255eb24ff` | the walk's H6 arm |
| 13 (fork half) | fork `e089a53917f92c44624fa72617e0d4a67a7f7050` | three falsified statements + the sysw provenance re-pin |
| 13 (engrave half) | engrave `32a4686064748db629edb441cc2482097228cb90` | two falsified spec statements, F-132's plate half, eight follow-ups, the acceptance doc |
| 13 (toolkit half) | toolkit `4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a` | the manual |

**The engrave base moved between dispatch and execution.** The addendum names
engrave master `bd5276b5`; `git worktree add … master` took `5e7211d0` (two
commits later: E's report, then the continuity note). Recorded, not worked
around — `h6-records` branches from the current master, which is what the
controller will merge into.

---

## Task 12 — the walk (spec §11.7)

### What it does

`cmd/emu/walk_hashlock_phrase.js`, 481 → 675 lines. The H2 arm is untouched; the
H6 arm begins where the old walk ended, at the path list after the reconcile
screen, and runs:

1. **a second, KEYED path**, 2-of-3, left UNSEATED. Normative, not convenience:
   `md.Compose` refuses a wholly key-less composition, so the ONE hashed
   key-less path the four trials build cannot reach Done — it stops at a
   refusal, not at a census. Unseated is §12 item 3's shape and is what
   collapses the form choice to `No slot is seated, so there is a template and
   nothing else.`
2. Done → Template (paged) → `Engrave a key-less template` → Review (paged) →
   hold → the collapsed form notice.
3. **step (A): ACCEPT a preimage plate** at §5.3's pick screen, row 0,
   `preimage string`.
4. **step (B): the census**, paged, asserting §8.3's row.

### The assertion the task asks for, and what makes it falsifiable

> the census row carries the same `first8..last8` the confirm modal carried

`displayed` is parsed out of the **confirm modal's own frame** by `drawnToken`
(shipped, H5); `censusToken` is parsed out of the **census's own frames** by a
new `censusPlateToken`. Neither is a constant, so this is one screen against
another. `ANCHOR_HARD_H` still pins the modal to the corpus one line earlier, so
the corpus stays the oracle for what the value should have been — the same
ordering H5 §4.1 established.

`censusPlateToken` distinguishes its two failures deliberately, because the two
mutations produce one each: the digest-group is **optional** in the regex, so a
row with no digest matches the row and fails on the group ("carries NO digest"),
while a perturbed digest matches everything and fails at the caller's compare.
Collapsing them would report a dropped digest as though §8.3's block had never
drawn.

Three further screens are asserted on the way, each of which can fail on its
own: the pick screen's lead carries the same token and is **masked**
(`mustNot(pick, "battery staple")`), the Review screen carries the same token
(the first surface outside the hashlock flow to hold it), and the census does not
print the phrase.

### `readPages` and the page count

`composerReadScreen` withholds the continue affordance until the last page has
been laid out once, so the arm pages to the screen's own wrap rather than
reading one frame. **MEASURED: the census is 2 pages on this walk's fixture, not
the 3 §11.7 records.** Not a discrepancy — §11.7's figure is for the gate's
three-plate fixture (one accepted phrase+QR plate, one declined, one unused);
this walk accepts one plate and declines nothing. §8.3's block is on page 2 in
both, so a walk asserting only the first frame would still assert the wrong
screen. The stub screen is 2 pages and the Review 1; both recorded in `out`.

### Step 2: FOUR RUNS, in a browser, against the real emulator

Each run on its **own fresh port** (the browser caches `emu.wasm`), driven
through playwright, fire-and-forget with `window.__done`/`window.__walk` polled.
The walk file is byte-identical across all four —
`sha256 dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581`,
re-measured before each build.

| run | port | tree | result |
| --- | --- | --- | --- |
| (a) | 8841 | unmutated | **`ok: true`**, 61.6 s |
| (b) | 8842 | MUTATION (a) | **REJECTED** |
| (c) | 8844 | MUTATION (b) | **REJECTED** |
| (d) | 8846 | unmutated, after the revert | **`ok: true`**, 62.7 s |

Run (d) is what proves the revert, which is why the protocol is four runs and
not three. Between (c) and (d), `git status --short` reported
`M cmd/emu/walk_hashlock_phrase.js` and nothing else, and the rebuilt
`emu.wasm` returned to run (a)'s exact size, 11,011,084 B.

**MUTATION (a) — "drop the census row's digest".**
`gui/composer_copy.go`, `composerCopyPreimagePlateRow`:

```go
-	return fmt.Sprintf("path %d  %s  %s", path, first8last8, form)
+	_ = first8last8
+	return fmt.Sprintf("path %d  %s", path, form)
```

Failing line, verbatim from `window.__err`:

```
the Plates To Cut census: the census row for path 1 carries NO digest, so the operator is handed a plate list they cannot read against the plates on the bench, and there is nothing to compare the confirm modal's token against.
Census: "PlatesToCutThisengraves1plate.md1template:1plate(key-lesswalletpolicy)Eachplatetakesminutestocut.Havethatmanyblanksreadybeforeyoustart:asetisonlyabackupwhenallofitexists.md1andmk1platescarryerrorcorrection.Aplaindescriptorplatecarriesonlyitschecksum,whichfindsamistakebutcannotfixone.PlatesToCutPlus1preimageplate(s),cutfirstandNOTpartofthisbackup:path1preimagestringKeepeachpreimageplateapartfromthepolicyplatesandfromtheothers."
```

**MUTATION (b) — "perturb the digest the census reports".**
`gui/composer_preimage_plate.go`, `composerPreimageCensusLines`:

```go
+			d := p.digest
+			d[0] ^= 0x01
 			out = append(out, composerCopyPreimagePlateRow(p.path,
-				hashlockFirst8Last8(p.digest), hashlockPlateFormWords(p)))
+				hashlockFirst8Last8(d), hashlockPlateFormWords(p)))
```

Failing line, verbatim:

```
§8.3's census row carries a DIFFERENT first8..last8 than the confirm modal drew: the operator is handed a plate list they cannot read against the digest they wrote down, and the plate about to be cut first is identified by a value nothing else showed them.
  confirm modal: 3cf5d421..b70a4c12
  census row:    3df5d421..b70a4c12
```

Both mutations were reverted with `git checkout --` and the revert proven by
`git diff --quiet` on the file plus run (d).

### Run (a)/(d) `ok: true`, the H6 fields

```
displayed:     3cf5d421..b70a4c12      (parsed from the confirm modal's frame)
consentToken:  3cf5d421..b70a4c12      (parsed from the Review screen's frames)
censusToken:   3cf5d421..b70a4c12      (parsed from §8.3's census row)
stored:        3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
storedBeforeHold: null
paths:         Spendpathsslots:3Path1:hashonlyPath2:2-of-3AddaspendpathChangethescriptDone
pick:          Preimageplatehash3cf5d421..b70a4c12path1phrase:28charactersmethod:hardened
               preimagestringphrase+methodphrase+method+QRdonotcutthispreimage
censusPages:   2      stubPages: 2
census:        ...PlatesToCutPlus1preimageplate(s),cutfirstandNOTpartofthisbackup:
               path13cf5d421..b70a4c12preimagestringKeepeachpreimageplateapartfromthepo
```

Raw captures kept at `/scratch/code/shibboleth/.tmp/h6f-runs/` (run-a, run-d as
JSON; runs b and c as text with their mutations and verbatim failures).

### Boundary gate, at the committed tip

```
GOOS=js GOARCH=wasm go vet ./cmd/emu/   -> exit 0
./cmd/emu/build.sh                      -> exit 0, emu.wasm 11,011,084 B
go test ./cmd/emu/                      -> ok  seedhammer.com/cmd/emu  1.104s
gofmt -l .                              -> the five baseline files, no sixth
```

`needle_test.go` needed no change, exactly as the plan predicts: `out.ok = true`
is still SET after the last assertion, which
`TestWalkOkContainsNoDriverSuppliedPlateCount` accepts as the strongest shape.

**There is no RED-then-GREEN evidence for Task 12 and there cannot be**, and
this is stated rather than glossed: the task adds no test to a suite. Its
falsifiability evidence IS runs (b) and (c) — a walk arm that could not fail is
the defect, and both mutations were constructed to produce exactly the two
failures the assertion is for.

---

## Task 13 — records

Records only. No production statement in any repo changes; the fork's gui suite
is the same 1287 tests before and after.

### Step 1 — the five falsified records

| # | where | done? |
| --- | --- | --- |
| 1 | fork `gui/composer_hash.go` | YES |
| 2 | fork `gui/composer_hashlock.go` | YES |
| 3 | engrave `SPEC_wallet_policy_composer.md` §6c + §14 row | YES |
| 4 | ms `crates/ms-cli/src/cmd/hashlock.rs:352` | **NO — DEVIATION, filed as F-501** |
| 5 | engrave `SPEC_hashlock_H2_device.md` §4.5 blockquote | YES |
| + | fork `codex32/mspayload.go` `IsPreimage` header | YES |

Records 1, 2 and the `IsPreimage` header were confirmed **not** already rewritten
by implementers C and E before editing — read at the merged tip, not assumed.
Each is rewritten rather than deleted, quotes the sentence it replaces, and cites
sites that were re-grepped **after** the edit (a comment edit moves the lines it
cites: `IsPreimagePlate` moved from `:141` to `:145` and the citation was
corrected).

**Record 5 also closes F-491, deliberately.** The blockquote's last two lines
were stale on a second axis: the shipped body dropped *"Spending any path of a
wsh wallet publishes this digest"* and its trailing clause **before H6 began** —
verified by reading `composerCopyHashlockConfirm` at fork `main` `fb0dd04` as
well as at `hashlock-h6`, so H6's falsification and F-491's pre-existing drift
are told apart rather than merged. Both are folded because record 5's own mandate
is that the blockquote and the shipped string stay in step, and half a fold
leaves the next reader diffing a stale sentence against a corrected one inside
one fenced block. F-491's heading now says CLOSED, per that file's convention.

### Step 2 — follow-ups

- **F-132's plate half CLOSED** (heading, per the file's convention), with the
  restore-doc half explicitly left open and still owned by operator journeys.
- **F-491 CLOSED** (above).
- **F-483 RE-STATED**, not closed — and the re-statement makes the boundary
  explicit: the phrase reaches `kbd.Fragment` before H6 stores anything, so
  `hashlockHeld` is downstream of the leak and cannot fix it; what H6 adds is a
  scrub for its own copy.
- **F-495** a CLI producer for `phrase:` records — a wire form with no writer.
- **F-496** a reconcile-style screen for a PAYLOAD phrase (§10.2).
- **F-497** cross-run awareness of preimage plates already cut (§5.3 item 6, §8.4b).
- **F-498** the stale `gui/composer_flow.go:34` citation, at three sites.
- **F-499** baseline red 1 — gofmt is FIVE files, not three.
- **F-500** baseline red 3 — the `history_purge` trio, with its measured cause.
- **F-501** the deviation above.
- **F-502** an observation: §8h's blockquote in two older specs shows one arm of four.

F-493 and F-494 were left untouched and not renumbered, as instructed.

### The three baseline reds, RE-MEASURED rather than transcribed

Two of the three plan claims did not survive measurement.

**Red 1 — gofmt.** The plan names three files. Measured on a detached worktree
of pristine fork `main` `fb0dd04`:

```
$ gofmt -l .
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
```

**FIVE**, confirming the controller's addendum. The same command at my tip prints
the same five and no sixth. Filed F-499.

**Red 2 — go vet.** Already F-494; not re-filed. Two numbers measured while
confirming it: `go vet ./engrave/` is **2** pristine and **3** at the H6 tip
(`engrave/h6_qr_test.go:516`), **not the 6 the plan predicts**; across `./...`
the `ArtifactDir` count goes **8 → 10** (H6's two new golden tests, in
`engrave/h6_qr_test.go` and `backup/hashlock_test.go`). Underneath both sits a
class the plan's baseline never mentions: **33 pre-existing `bspline` unkeyed-field
findings**, identical pristine and at the tip.

**Red 3 — history_purge.** Reproduced at engrave master `5e7211d0`:

```
$ cargo nextest run --locked --no-fail-fast -p mnemonic-engrave
     Summary [0.349s] 621 tests run: 618 passed, 3 failed, 2 skipped
```

618 of 621 confirmed. **`--no-fail-fast` is required to see it** — without it
nextest stops at 465 of 621. And the cause is not "box-local": each test prints
its own reason, *"/usr/bin/zsh is required … This is deliberately a FAILURE and
not a skip — a skipped gate prints ok and exit 0."* `which zsh` is empty on this
box. Not flaky, not a code defect, and not to be weakened: `pacman -S zsh`.
Filed F-500 so the next implementer spends no round on it.

### Step 3 — the toolkit manual

`docs/manual/src/40-cli-reference/43-ms.md`, +100 lines, one shipped quote
folded. Covers the two plate forms and why one is conditional; that the plate is
a **bearer instrument**, said early; §8.6's QR text verbatim with its three
deliberate choices; that **`ms hashlock` does not parse it yet** as its own
bolded paragraph; `me sysw pack --pack-preimage` with both wire forms, that it is
not `--seal-secret`, and that `--in` is required rather than stylistic; and that
no verb emits a `phrase:` record yet, with the `xxd` line.

The confirm-screen blockquote carried *"The phrase and method are not on this
device."* — the same string record 5 covers — and is folded to the shipped H6
text with one parenthesis explaining the change, so a reader with an H2 machine
is not left thinking the manual is wrong.

```
$ make lint
[lint] OK
```

(markdownlint; cspell 0 issues over 40 files; lychee 274 OK / 0 errors;
flag-coverage; glossary-coverage; index bidirectional.) Two markdownlint
findings were fixed **before** the commit, not after: MD040 (the QR block needed
a `text` language) and MD038 (`` `phrase: ` `` carried a trailing space inside a
code span; rewritten as prose).

### Step 4 — §12's acceptance

New: `design/ACCEPTANCE_hashlock_H6_preimage_plates.md` (96 lines). No hashlock
acceptance document existed — `ACCEPTANCE_engrave_transaction.md` belongs to a
different cycle — so this creates one rather than reporting a missing file.

It transcribes §12's twelve items into a table with a column the spec's list does
not carry: **what a machine has already proved, and by what**, so operator budget
goes to the rows nothing can prove without hardware. Item 8 gets its own section
and is flagged as a **GATE**, with the three durations quoted from a real run
rather than from the plan:

```
$ go test ./backup/ -run TestHashlockPlateCutDurationsAreBounded -v
    worst-case phrase plate WITH the v9 QR   43m31s
    the same plate with the QR removed       14m39s
    the string-form plate                    12m14s
    the v9 QR alone                          32m12s (dim 53, scale 2)
    one 6 mm character                       7s
PASS
```

`ConstantQRCmd.Engrave`'s `for range nmod` was grepped (`engrave/engrave.go:753`)
before writing that the QR is indivisible by construction.

### The provenance re-pin (addendum item 1)

`sysw/testdata/record_class_vectors.provenance.json`:
`commit` → `8cf2a7f98e5f5bfa535200b260a514cdae33f087` (the `h6-b` tip),
`file_commit` → `4d00fbbf2b4012a533d060161bef72e3c7b547c1` (the commit touching
`crates/me-cli/testdata/record_class_vectors.json`), both measured with the pin's
own RE-SYNC recipe against a **clean** `h6-b` worktree. The PROVISIONAL note is
replaced by one that is true and was not before: the commits are on a branch, not
on engrave master.

**sha256 verified on both sides of the edit** — the vendored fork copy and B's
engrave file both hash to
`3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf`, before and
after. `grep -rl PROVISIONAL` over the fork tree now returns nothing. A
`commit_branch` key was added; `loadRecordClassRows` and `loadVectorPin` both
unmarshal into subset structs, so the extra key is ignored, and `sysw` tests pass.

---

## Gates run

**Fork, at `e089a53`:**

```
go build ./...                                  -> exit 0
go test $(go list ./... | grep -v /gui$)        -> every package ok
scripts/gui-shard-test.sh ./gui/ 24             -> 1287 tests, 24 shards, partition
                                                   verified exhaustive, ok, wall 25s
GOOS=js GOARCH=wasm go vet ./cmd/emu/           -> exit 0
./cmd/emu/build.sh                              -> exit 0
go test ./cmd/emu/                              -> ok 1.104s
gofmt -l .                                      -> the five baseline files, no sixth
```

**Engrave:** `cargo nextest run --locked --no-fail-fast -p mnemonic-engrave` →
618/621 (the three `history_purge` reds above). Records-only commit; no Rust
changed.

**Toolkit:** `make lint` → `[lint] OK`.

---

## Deviations, all of them

1. **Record 4 of Step 1 was NOT folded.** `crates/ms-cli/src/cmd/hashlock.rs:352`
   is in mnemonic-secret, which this brief names no branch in, and whose
   `ms-codec` 0.9.0 is already published. Editing a fourth repo's shipped copy on
   an unannounced branch is the merge hazard the brief's "touch only the files
   your tasks list" rule exists to prevent. Filed as **F-501** with its
   reproduction and one-line fix. `grep -rn "it is on no plate" --include=*.rs`
   finds only the production site, so no test asserts it. **The controller should
   route this to the next `ms` cycle before H6 reaches an operator.**

2. **Task 13 touched fork files the plan's File Structure table does not list
   for Task 13.** That table assigns Task 13 only `engrave design/…` and
   `toolkit docs/manual/…`, while Step 1's own text names three fork files. They
   were edited, because the table is the PARALLEL-GROUP partition and group F is
   last: `h6-f` branches from the merged `hashlock-h6` carrying A–E, so no other
   implementer can conflict. Had F run in parallel this would have been a STOP.

3. **The plan's `go vet ./engrave/` prediction is wrong** — 6 predicted, **3**
   measured (2 pristine). Recorded under F-500's closing note, not folded into
   the plan (the plan is not mine to edit).

4. **The plan's gofmt baseline of three is wrong** — FIVE, as the addendum said.
   Measured on pristine `fb0dd04`, filed F-499.

5. **The plan's `gui/composer_flow.go:48` for the `composerState` literal is
   itself stale** — measured `:51` at the H6 tip. F-498 records the measured
   number and argues for citing the symbol instead, since a line number in a
   comment is a citation with no gate behind it.

6. **§11.7's "11 lines over 3 pages" does not reproduce on this walk's fixture**
   — 2 pages. Not a defect: different fixture (one accepted plate, none declined,
   none unused, against §11.7's three). Documented in the walk's own comment so
   the next reader does not treat the difference as drift.

7. **The engrave base moved** from the addendum's `bd5276b5` to `5e7211d0`
   between dispatch and `worktree add`.

8. **F-502 was filed rather than folded.** §8h's blockquote in
   `SPEC_wallet_policy_composer.md` and `SPEC_hashlock_H2_device.md` each show
   one arm of a chooser that has had four since H6. Neither is FALSE, it is
   outside the five records the plan enumerates, and deciding which spec owns
   that copy is a re-decision rather than a record fix.

## What I did not do

- No push, no tag, no release, no flash.
- No sub-agent.
- No `.jsonl` read.
- No phrase or preimage bytes in any retained log: the walk's `out` fields carry
  digests and screen text only, and the anchor phrase is the published
  brainwallet phrase already in the corpus and in the shipped walk.
- No edit to `SPEC_hashlock_H6_preimage_plates.md` or to
  `IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`.
