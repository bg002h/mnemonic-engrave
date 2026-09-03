# composer S4 — fold of the whole-diff execution review, round 0

**Report folded:** `design/agent-reports/composer-S4-exec-review-r0.md` (opus,
**0C / 1I / 4M / 4N**, persisted at mnemonic-engrave `93988cd`).
**Brief:** `design/agent-briefs/composer-S4-fold-r0-brief.md`.
**Implementer:** the same single opus agent, resumed a third time.

**All eight assigned findings folded** (I-1, M-1, M-2, M-3, M-4, N-1, N-3, N-4).
N-2 is the controller's own record and needed no change. Every gate is green and
every proof the brief asked for is reproduced below, exit codes recorded
directly and never through a pipe.

## New tips

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu log --oneline main..HEAD
a6eb44e emu: fold the S4 whole-diff review -- M-1, M-2, N-1, N-4 (composer-S4-exec-review-r0)
86cec95 emu: the composer journey's walk -- shots_composer.js, and shTargets to tap rows by (S4 Task 3)
a79a454 Merge main into composer-s4-emu: W-2, the pick screen's rows are touch targets (S4 Task 3 needs it)
05d903b emu: a THIRD test payload carrying the composer's own record classes (S4 Task 1)

$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu log --oneline master..HEAD
651fa0e journeys: regenerate transcript_composer.txt at the fork tip (S4 review N-3)
05a066a journeys: fold the S4 whole-diff review -- I-1, M-3, M-4 (composer-S4-exec-review-r0)
c6adac2 journeys: the composer's device half -- capture_composer.py (S4 Task 3)
5040bb2 journeys: the composer's host half -- transcript_composer.sh (S4 Task 2)
```

Both trees end clean (`git status --porcelain` empty, exit 0). Nothing pushed,
nothing flashed, no sub-agent, no `.jsonl` read. Every mutation ran in a `cp -r`
copy under `/scratch/code/shibboleth/.tmp/` with no `.git` reachable
(`n4copy`, `m2mut`, `s4fold`); no committed artifact was mutated in place.

---

## I-1 — the negative control now attributes its own failure

**`design/journeys/capture_composer.py`** — new `Failure` class (`:146-155`),
`drive()` returns `Failure(name, str(e))` instead of `None` (`:196-206`), the
`COMPARISON_FIRED` constant (`:45`), `corrupted_address` bound where it is
produced (`:271-274`), and the control branch rewritten (`:317-329`).

`drive()` returned a bare `None` on any `page.evaluate` exception, and
`--prove-it-can-fail` read `res is None` as proof the **address comparison** had
fired. It is the same value for "the walk broke at step 2". The plan's words are
*"exits 0 only if the walk **caught** it"*; the code exited 0 if the walk merely
**stopped**.

`Failure` is a class rather than a tuple deliberately, so `res is None` cannot
survive anywhere as a stand-in for "the comparison fired". The control now
requires the failure text to contain **both** the comparison's own message and
the corrupted address; anything else is `NEGATIVE CONTROL INCONCLUSIVE`.

### (a) The review's reproduction — INCONCLUSIVE, non-zero, in seconds

In a copy of `out/` (`.tmp/s4fold/…`), `payload.digest.txt` `dbe9` → `dbe0`, so
the walk dies at itinerary row 2 and never reaches row 17 where an address is
compared:

```
$ sed -i 's/dbe9/dbe0/' out/composer/payload.digest.txt
$ python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build --port 8803 --shot-port 8744
I-1(a) EXIT=1  elapsed=7s

NEGATIVE CONTROL INCONCLUSIVE: the walk failed before the comparison --
Page.evaluate: Error: the device's payload digest does not equal the host's
`me sysw show`: the screen does not carry "dbe0 e774 e9a4 9231 0b62 626c 2b41 cf4b".
Screen: "PayloadDigestComparethisagainst`mesyswshow<file>`onthehost:dbe9e774e9a492310b62626c2b41cf4b"
```

Before the fold this same command printed `NEGATIVE CONTROL PASSED` and exited
**0** in 8 s. The digest file was restored immediately after (verified).

### (b) The honest run — PASSED, exit 0, whole itinerary walked

```
$ python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build --port 8803 --shot-port 8744
I-1(b) EXIT=0  elapsed=39s

DRIVER FAILED on leg keyed-A: the device's proof does not match the host's:
  address bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4q
NEGATIVE CONTROL PASSED: the walk refused the corrupted address.
  it failed at the address comparison, naming bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4q
```

39 s against 7 s is itself the discriminator the control was missing.

**The needle is specific.** `COMPARISON_FIRED` is the id/address message; the
byte comparison throws a different one (`string N of M does not match the host's
BYTE FOR BYTE`) which does not contain it, so the control cannot be satisfied by
the wrong comparison either. It is spelt once here and once in
`shots_composer.js:788`; a rename that broke the pairing turns the control
INCONCLUSIVE — loudly — rather than leaving it passing for the wrong reason.

---

## M-1 — the dead post-condition

**`cmd/emu/shots_composer.js:701`** — `chooseRow(0, "seed 1", …)` →
`chooseRow(0, "(any slots)", …)`.

`"seed 1"` was already on the pre-tap frame: it is the slot prefix in the
passphrase screen's **own title** (`gui/composer_sources.go:259`,
`Title: "Passphrase " + label`), so the frame reads
`AddaBIP-39passphrase?SkipAddpassphrasePassphraseseed1`. Waiting for it
certified nothing about the tap.

`"(any slots)"` is drawn in **exactly one production site** — measured, not
assumed:

```
$ grep -rn "any slots" gui/*.go | grep -v _test
gui/composer_sources.go:149:  return s.label + "  (any slots)"     <- composerSourceRow
(the other five hits are comments in composer_seat.go, composer_state.go,
 multisig.go and multisig_supply_tail.go)
```

so it can only appear on the re-drawn pick list the tap produces. The
passphrase `ChoiceScreen`'s own copy is `Skip` / `Add passphrase` and carries no
source row at all. Confirmed live by the full run below.

---

## M-2 — `Choose engraving` asserted for exclusivity, and taken by row

**`cmd/emu/shots_composer.js`** — the handler (`:294-330`), the tail signature
(`:262`), and both call sites (`:530-534` keyless, `:790-793` keyed).

The plan claims a packed plate offers *"TEXT ONLY alone"* and that *"TEXT + QR is
row 0"*; a substring test proves neither, and the row was taken with a bare
`shTap(CONFIRM)` on whatever was selected by default. The handler now asserts the
row **count**, the rows **joined** (which pins their order, and so makes `take` an
index into a known list), a **forbid** list, and **takes the row through
`shTargets()`** as `chooseRow` does.

| arm | rows | take | forbid |
| --- | --- | --- | --- |
| keyed | `["TEXT ONLY"]` | 0 | `["QR ONLY", "TEXT + QR"]` |
| key-less | `["TEXT + QR", "TEXT ONLY", "QR ONLY"]` | 0 | — |

Both forbid literals rather than only `QR ONLY`: "alone" is what the plan
claims, and forbidding `QR ONLY` alone would not catch a `TEXT + QR` that
appeared. That is the decision's intent, executed; it changes no other
behaviour.

**Proven to bite** — expecting two rows where the device offers three, in a
copied `cmd/emu`:

```
M-2 MUTATION EXIT=1
DRIVER FAILED on leg keyless: Choose engraving offers 3 row(s), the walk expects 2
  (TEXT + QR, TEXT ONLY).
  Screen: "ChooseengravingTEXT+QRTEXTONLYQRONLYCard1of1|Plate1of1"
```

**And the taken row is now recorded**, so the run's own artifact carries the
evidence rather than only the assertion having fired:

```
keyed-A -> variantRowsTaken: ['TEXT ONLY', 'TEXT ONLY']
keyed-B -> variantRowsTaken: ['TEXT ONLY', 'TEXT ONLY', 'TEXT ONLY', 'TEXT ONLY']
keyless -> variantRowsTaken: ['TEXT + QR']
```

That addition came from a mistake worth recording: my first check of the taken
row read `tail.acts` through the leg result, where `acts` **is not surfaced** —
an empty list that looked exactly like a negative. Empty output is not absence,
so the value is now returned explicitly.

---

## M-3 — the address comparison cannot degrade to zero comparisons

**`design/journeys/capture_composer.py:88-101`** — `read_keyed()` exits non-zero
unless exactly four addresses were read.

```
$ : > out/composer/keyed.receive.txt
$ python3 -c "import capture_composer as c; c.read_keyed()"
M-3 emptied-file EXIT=1
the host wrote 2 address(es) into out/composer/keyed.{receive,change}.txt, want 4
(receive 0-1 and change 0-1).
An empty or short file would make the consent comparison compare NOTHING while the
run still reported a match. Re-run ./transcript_composer.sh.

$ (file restored) python3 -c "… print(len(...))"
M-3 restored EXIT=0
4 addresses
```

It also pins what the consent screen must carry: two receive and two change.

---

## M-4 — the port collision

**`design/journeys/capture_composer.py:24` (usage) and `:243-248` (defaults)** —
`8797/8738` → **`8803/8744`**, and the docstring now names the pair it actually
uses (it had advertised `capture_walletpolicy.py`'s `8793/8734`).

Measured across every shipped driver after the change:

```
capture_composer.py              8803 / 8744     <- free pair
capture_csid_warning.py          8798 / 8739
capture_hashvault.py             8799 / 8740
capture_operator.py              8791 / 8732
capture_pathological.py          8791 / 8732
capture_rcw.py                   8801 / 8742
capture_seating.py               8797 / 8738
capture_tr_pathological.py       8795 / 8736
capture_walletpolicy.py          8793 / 8734
```

**Observation, not a change:** `capture_operator.py` and
`capture_pathological.py` share `8791/8732`. That collision is pre-existing,
outside this diff, and untouched — recorded here because the sweep found it.

---

## N-1 — the unreachable stale-wasm guard

**`cmd/emu/shots_composer.js:167-179`** — the guard is **hoisted** above the
`window.shTargets()` call it protects. It sat one line below, so a stale wasm
threw `TypeError: window.shTargets is not a function` first and the friendly
message could never print.

**Hoisted rather than deleted, and the reason changed during the fold:** `run()`'s
copy fires once at entry, but after M-2 `chooseRow` is *also* reached from the
engrave loop's variant handler, long after that check has passed. Keeping it is
now load-bearing rather than redundant.

---

## N-3 — the transcript regenerated at the fork tip (done last)

**`design/journeys/transcript_composer.txt`**, regenerated at fork `a6eb44e`
with `FORK=/scratch/code/shibboleth/wt-composer-s4-emu`, after every other
commit.

```
$ FORK=… ./transcript_composer.sh > transcript_composer.txt
TRANSCRIPT EXIT=0
$ grep -c '^GATE PASS'   27
$ grep -n  '^GATE FAIL'  (none, EXIT=1)
```

**The entire diff against the previous file:**

```
24c24
< 05d903b
---
> a6eb44e
462-475, 481-483:  the `ls -la` mtimes,  Sep  3 01:07 -> Sep  3 03:53
```

and nothing else. No path line moved — the previous run already used this
worktree. With the rev line and the `ls -la` rows excluded the two files are
**byte-identical**:

```
$ diff <(strip before) <(strip after)
SUBSTANTIVE DIFF EXIT=0
```

Every artifact compared with `cmp` rather than by eye — `keyed.id.txt`,
`keyed.receive.txt`, `keyed.change.txt`, `keyed.md1.txt`,
`keyed-template.md1.txt`, `keyless-tr.md1.txt`, `payload.digest.txt` all
identical to the pre-fold copies, and `payload.bin` still byte-identical to
`cmd/emu/sysw_composer_payload.bin` (exit 0).

---

## N-4 — the composer walk's terminal anchor, machine-checked

**`cmd/emu/needle_test.go`** — new `composerFlowNeedles` (`:110-123`),
`literalSites()` (`:126-166`),
`TestComposerFlowNeedlesHaveExactlyOneLiteralSite` (`:193-213`) and
`TestLiteralSiteCounterIgnoresComments` (`:222-243`); `go/ast`, `go/parser`,
`go/token`, `strconv` added to the imports.

### A second list, and the measurement that forced it

The brief says to add the two pins "on `buildFlowNeedles`' pattern", with the
acceptance criteria **pass on the tip** and **fail when a second literal is
added**. Putting them in `buildFlowNeedles` itself fails the first criterion,
and the reason is measured rather than argued: `productionSites` is a **raw text
scan** — deliberately, since that is what lets `decoyNeedles` pin counts for
strings built by concatenation — and `"Build a new policy"` occurs in **five**
gui files, of which **one** is a rendered site:

```
$ go test -run Needle ./cmd/emu/          (with the entries in buildFlowNeedles)
needle "Build a new policy" has 5 production site(s), want exactly 1:
  gui/composer_door.go      <- the only rendered site, :116
  gui/composer_flow.go      :11    comment
  gui/gui.go                :193   comment
  gui/multisig_build.go     :24, :29  comments
  gui/sysw_admit.go         :54    comment
EXIT=1
```

Widening `productionSites` would move every existing pin and decoy count with
it. So the needles are pinned by the site that matters — **a string literal in
code** — using the AST walk `embed_confinement_test.go` already uses for exactly
the same "a mention is not a reference" distinction. This is the pattern applied,
not the decision re-litigated; both acceptance criteria are met below.

### Passes on the tip

```
$ go test -count=1 -v -run 'ComposerFlowNeedles|LiteralSiteCounter' ./cmd/emu/
--- PASS: TestComposerFlowNeedlesHaveExactlyOneLiteralSite
    "Build a new policy"     -> gui/composer_door.go
    "Which script?"          -> gui/composer_shape.go
--- PASS: TestLiteralSiteCounterIgnoresComments
    raw counter 5 file(s) […]; literal counter 1 [gui/composer_door.go]
new needle tests EXIT=0
```

### Fails in a copy with a second literal

`var mutationSecondDoorRow = "Build a new policy"` added to
`gui/composer_census.go` in `.tmp/n4copy` (no `.git`):

```
$ go test -count=1 -run 'ComposerFlowNeedles' ./cmd/emu/
--- FAIL: TestComposerFlowNeedlesHaveExactlyOneLiteralSite
    composer needle "Build a new policy" is spelt in 2 production file(s) in CODE,
    want exactly 1:
      gui/composer_census.go
      gui/composer_door.go
    the composer walk anchors on this; a second site makes it name the wrong screen,
    and for the door's row that ends the engrave tail early against a PARTIAL census
MUTATED COPY EXIT=1
```

`TestLiteralSiteCounterIgnoresComments` is the counter's own mutation proof, on
`TestNeedleSiteCounterCanCount`'s pattern: it fails if the two counters ever
agree on `"Build a new policy"` (i.e. the comment sites it exists to discount
are gone), if the literal count is not 1, or if a string no gui file spells
comes back non-empty. Without it a `literalSites` that returned one file for
everything would make every composer needle look unique — the false-PASS shape
that file exists to remove.

---

## N-2 — no action, as decided

Recorded for completeness: `gofmt -l gui/` lists `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`. All three
are unformatted at `60bee002` already and outside this diff; **not touched**. The
plan's actual gate is `gofmt -l cmd/`, which is clean.

---

## Gates

Fork, in `/scratch/code/shibboleth/wt-composer-s4-emu`:

```
$ gofmt -l cmd/                               (no output)   EXIT=0
$ CGO_ENABLED=0 go test -count=1 ./cmd/emu/   ok 2.041s      EXIT=0
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/                      EXIT=0
$ node --check (shots_composer.js as a module)               EXIT=0
```

Engrave, in `/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys`, against
the fork worktree's `cmd/emu`:

| gate | exit | detail |
| --- | --- | --- |
| `capture_composer.py --arm both --no-build` | **0** | keyed-A 62 s / keyed-B 74 s / keyless 29 s, **50 shots**, `all legs matched the host.` |
| `--prove-it-can-fail` (honest) | **0** | PASSED, 39 s, naming the corrupted address |
| `--prove-it-can-fail` (digest corrupted) | **1** | INCONCLUSIVE, 7 s |
| the plan's unchunked-string mutation | **1** | `device: … (56 chars)` / `host: … (47 chars)` |
| M-2 variant mutation (3 rows vs 2) | **1** | `Choose engraving offers 3 row(s), the walk expects 2` |
| M-3 emptied address file | **1** | `the host wrote 2 address(es) … want 4` |
| N-4 second-literal copy | **1** | `spelt in 2 production file(s) in CODE` |
| `capture_walletpolicy.py --port 8793 --shot-port 8734` | **0** | wallet id `4e67c6fd…` |
| `capture_seating.py --port 8793 --shot-port 8734` | **0** | wallet id `c8fe87cd…` |
| `capture_tr_pathological.py --port 8793 --shot-port 8734` | **0** | wallet id `590f3abc…` |
| `transcript_composer.sh` (N-3) | **0** | 27 GATE PASS, 0 GATE FAIL |

The final `--arm both` was run **after** both code commits, so the recorded gate
is against the committed tree. Every oracle still matches: payload digest
`dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b`, Template-ID `531ab9e1…`, Policy-ID
`4dd749a8…`, the four addresses, form A's 7 chunks, form B's 9 strings and the
key-less 56-character chunked string.

---

## What I decided, and what I could not do

1. **N-4 needed a second list and a literal-aware counter** rather than two
   entries in `buildFlowNeedles`. The brief's two acceptance criteria cannot both
   hold under a raw-text counter that sees four comments; the measurement is
   above. Same pattern, same file, new counter with its own mutation proof.
2. **M-2's keyed `forbid` carries two literals**, not just `QR ONLY`, because
   "TEXT ONLY alone" is what the plan claims and one literal does not assert it.
3. **`variantRowsTaken` was added to the result** so the taken row is evidence in
   the artifact, not only an assertion that fired — prompted by my own vacuous
   first check against the unsurfaced `acts`.
4. **`capture_operator.py` and `capture_pathological.py` still share
   `8791/8732`.** Pre-existing, outside this diff, left alone — reported because
   the M-4 sweep surfaced it and someone may want it filed.
5. **Not done, and not mine:** merging or pushing either branch, flashing, Task 4
   (the live device walk, which the plan gates on the W-2 fix being flashed),
   Task 5 and Task 6.
