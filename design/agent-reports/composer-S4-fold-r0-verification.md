# composer S4 — INDEPENDENT fold-verification (targeted), round 0

Verifying the fold of `design/agent-reports/composer-S4-exec-review-r0.md`
(opus, 0C/1I/4M/4N) against the controller's decisions in
`design/agent-briefs/composer-S4-fold-r0-brief.md`. The implementer's own
`design/agent-reports/composer-S4-fold-r0-report.md` was read but NOT trusted —
every claim below was re-derived independently.

Folded tips verified in place, both worktrees clean at exactly the cited SHAs:

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu status --porcelain   (empty)
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu rev-parse HEAD
a6eb44e794c3ee7bd6484d0125fc51a256401706
$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu status --porcelain    (empty)
$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu rev-parse HEAD
651fa0ea019ec25fe1a878e89623903cf1bb4e6c
```

Read-only throughout: every mutation ran in `cp -r` copies under
`/scratch/code/shibboleth/.tmp/s4verify/` (`fork`, `engrave`, `.git` removed
from both so no git command could reach a shared gitdir). No sub-agents, no
`.jsonl` read, nothing committed. Both source worktrees confirmed still clean
and at the cited tips after every check (shown above, re-checked at the end).
`Go 1.26.7` by path, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local -mod=readonly`,
`TMPDIR=/scratch/code/shibboleth/.tmp`.

---

## 1. Diff scope — VERIFIED both ways

```
$ git diff 86cec95..a6eb44e794c3ee7bd6484d0125fc51a256401706 --stat   (fork)
 cmd/emu/needle_test.go    | 130 ++++++++++++++
 cmd/emu/shots_composer.js |  79 ++++++++++--------
 2 files changed, 193 insertions(+), 16 deletions(-)

$ git diff c6adac2..651fa0ea019ec25fe1a878e89623903cf1bb4e6c --stat   (engrave)
 design/journeys/capture_composer.py     | 77 ++++++++++++++++++++++++++++-----
 design/journeys/transcript_composer.txt | 36 +++++++--------
 2 files changed, 83 insertions(+), 30 deletions(-)
```

Read both diffs in full (not just `--stat`). Fork: `shots_composer.js` changes
are exactly N-1 (guard hoisted above the `shTargets()` call), M-1 (`"seed 1"` →
`"(any slots)"`), M-2 (`variantRows` string → a `variant{rows,take,forbid}`
object, `mustNot` calls, `variantRowsTaken` added to the returned record); `needle_test.go`
adds exactly N-4's new list, counter and two tests. Engrave:
`capture_composer.py` changes are exactly I-1 (the `Failure` class,
`COMPARISON_FIRED`, the attributed control branch), M-3 (the 4-address count
guard in `read_keyed()`), M-4 (port defaults + docstring); `transcript_composer.txt`
changes are exactly N-3 (see §5). **No oracle value and no itinerary row changed
beyond these eight decisions.**

---

## 2. I-1 — all three arms VERIFIED

### (a) The report's reproduction — corrupted `payload.digest.txt`

```
$ sed -i 's/dbe9/dbe0/' out/composer/payload.digest.txt
$ python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build --port 8803 --shot-port 8744
NEGATIVE CONTROL INCONCLUSIVE: the walk failed before the comparison --
  Page.evaluate: Error: the device's payload digest does not equal the host's
  `me sysw show`: the screen does not carry "dbe0 e774 e9a4 9231 0b62 626c 2b41 cf4b".
real 0m8.159s   EXIT=1
```
Digest file restored and diffed byte-identical to the pre-mutation copy immediately after.

### (b) The honest run

```
$ python3 capture_composer.py --arm keyed --prove-it-can-fail --no-build --port 8803 --shot-port 8744
NEGATIVE CONTROL PASSED: the walk refused the corrupted address.
  it failed at the address comparison, naming bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4q
real 0m38.920s   EXIT=0
```
Walked the whole itinerary (17 numbered shots through `c11-consent-p3.png`) before failing at consent, not at itinerary row 2 — the discriminator the finding said was missing.

### (c) The extra arm the report did not run — `keyed.receive.txt` corrupted directly, plain `--arm keyed`

The brief's third arm: corrupt the address in the host artifact itself (not via
`--prove-it-can-fail`), then run the plain leg.

```
$ cp out/composer/keyed.receive.txt out/composer/keyed.receive.txt.orig
$ sed -i '1s/l$/q/' out/composer/keyed.receive.txt   # bc1q...tp4l -> bc1q...tp4q
$ python3 capture_composer.py --arm keyed --no-build --port 8803 --shot-port 8744
DRIVER FAILED on leg keyed-A: Page.evaluate: Error: the device's proof does not
  match the host's:
  address bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4q
...
capture failed: the driver did not return
EXIT=1
```
The consent screen's own text (captured in the same run) shows the real device
address ends `...tp4l`; the driver names the corrupted host expectation
`...tp4q` and fails, exactly as required. File restored and diffed identical.

**I-1: VERIFIED (a), (b), and the extra arm (c).**

---

## 3. M-1, M-2, M-3, M-4, N-1 — all VERIFIED

**M-3** — emptied `keyed.receive.txt`, called `read_keyed()` directly (no browser, no walk):
```
$ : > out/composer/keyed.receive.txt
$ python3 -c "import capture_composer as c; c.read_keyed()"
the host wrote 2 address(es) into out/composer/keyed.{receive,change}.txt, want 4 ...
EXIT=1
```
File restored, diffed identical.

**M-2, keyed arm ("QR ONLY" absence assertion can fail)** — mutated the fork
copy's `forbid` list (`cmd/emu/shots_composer.js:825`) to also forbid `"TEXT ONLY"`,
which IS on the real screen, and re-ran `--arm keyed`:
```
DRIVER FAILED on leg keyed-A: Page.evaluate: Error: Choose engraving offers a
  variant the plan says it must not: the screen carries "TEXT ONLY" and must not.
EXIT=1
```
`shots_composer.js` restored, diffed identical to the pre-mutation copy.

**M-2, keyless arm (taken row is "TEXT + QR" by label)** — ran the unmutated
`--arm keyless` and read the driver's own `composer-result.json` (not stdout,
which doesn't surface it):
```
keyless -> variantRowsTaken: ['TEXT + QR']
```

**M-1** — `cmd/emu/shots_composer.js:738`: `await chooseRow(0, "(any slots)", "Skip the passphrase");`.
Confirmed the post-condition text is drawn at exactly one non-comment
production site:
```
$ grep -rn "any slots" gui/*.go | grep -v _test.go
gui/composer_sources.go:149:   return s.label + "  (any slots)"
(all five other hits are comments)
```
so the post-condition can only be satisfied by the re-drawn pick list the tap
produces, never by the pre-tap passphrase screen. Also confirmed live: this
step passed cleanly in every full-itinerary run above (2b, 2c).

**M-4** — `design/journeys/capture_composer.py`: usage line `:24` reads
`[--port 8803] [--shot-port 8744]`; `ap.add_argument` defaults at `:247-248` are
`8803` / `8744`. Cross-checked against every other shipped driver's defaults —
no new collision; the only shared pair left is `capture_operator.py` /
`capture_pathological.py` at `8791/8732`, pre-existing and outside this diff
(also recorded, un-actioned, in the fold report).

**N-1** — `cmd/emu/shots_composer.js:174-177` (the `typeof window.shTargets !== "function"` throw) precedes `:178` (`const targets = window.shTargets();`).

**M-1, M-2, M-3, M-4, N-1: VERIFIED.**

---

## 4. N-4 — VERIFIED, on the tip and in a mutated copy

```
$ go test -count=1 -run 'Needle' -mod=readonly ./cmd/emu/        (tip, fork copy)
ok  	seedhammer.com/cmd/emu	0.445s
EXIT=0
```

Added `var verificationMutationSecondDoorRow = "Build a new policy"` to a copy
of `gui/composer_census.go` (a different `gui/*.go` file than the pinned
`gui/composer_door.go`):

```
$ go test -count=1 -run 'Needle' -mod=readonly ./cmd/emu/        (mutated copy)
--- FAIL: TestComposerFlowNeedlesHaveExactlyOneLiteralSite (0.17s)
    needle_test.go:200: composer needle "Build a new policy" is spelt in 2
      production file(s) in CODE, want exactly 1:
        gui/composer_census.go
        gui/composer_door.go
FAIL
EXIT=1
```
File restored, diffed identical.

**N-4: VERIFIED.**

---

## 5. N-3 — VERIFIED

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu rev-parse --short a6eb44e794c3ee7bd6484d0125fc51a256401706
a6eb44e
$ grep -A2 "rev-parse --short HEAD" design/journeys/transcript_composer.txt
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu rev-parse --short HEAD
a6eb44e
[exit 0]
```
Rev line matches the folded tip's short SHA exactly.

Pulled the pre-fold (`c6adac2`) and post-fold (`651fa0e`) copies of
`transcript_composer.txt` via `git show`, stripped the rev-parse output line
and every `ls -la` mtime column with `sed`, and diffed:
```
$ diff pre_stripped.txt post_stripped.txt
SUBSTANTIVE DIFF EXIT=0
```
Byte-identical once the rev line and mtimes are excluded — the ids, addresses
and md1/mk1 strings are unchanged.

**N-3: VERIFIED.**

---

## 6. Gates — all VERIFIED, recomputed independently

Fork (`.tmp/s4verify/fork`):
```
$ gofmt -l cmd/                                       (no output)   EXIT=0
$ go test -count=1 -mod=readonly ./cmd/emu/
ok  	seedhammer.com/cmd/emu	2.085s                                EXIT=0
$ GOOS=js GOARCH=wasm go vet -mod=readonly ./cmd/emu/                EXIT=0
```

Engrave (`.tmp/s4verify/engrave/design/journeys`, `EMU` = the fork copy):

| gate | exit | detail (recomputed, not read from the fold report) |
| --- | --- | --- |
| `capture_composer.py --arm both --no-build` | **0** | keyed-A 60s/21 shots, keyed-B 74s/21 shots, keyless 29s/8 shots = **50 shots**, `all legs matched the host.` |
| unchunked-string mutation (`md15zfdsssj6...`, 47 chars, into `keyless-tr.md1.txt`) | **1** | `device: "...49cqps8ys3psqcsmzu90h5wvl3" (56 chars)` / `host: "...q9dp5v3xc" (47 chars)` |
| `capture_walletpolicy.py --port 8793 --shot-port 8734` | **0** | wallet id `4e67c6fd8220c32e51c9ad9947e24141` |
| `capture_seating.py --port 8797 --shot-port 8738` | **0** | wallet id `c8fe87cd5fb7351db12479a2bab8f8ad` |
| `capture_tr_pathological.py --port 8795 --shot-port 8736` | **0** | wallet id `590f3abcaad2aca5a3f526917f5bb57a` |

(The three shipped drivers hard-code `EMU` as `../../../seedhammer/cmd/emu`
relative to `design/journeys`; ran them by symlinking
`.tmp/s4verify/seedhammer -> .tmp/s4verify/fork`, matching what the review
report itself did — neither real checkout was touched. `capture_walletpolicy.py`
and `capture_tr_pathological.py` needed their upstream fixtures regenerated in
my copy first, `transcript_walletpolicy.sh` and `make_seating_fixture.py`
(clean, exit 0); `transcript_tr_pathological.sh` hit an unrelated pre-existing
FATAL in its own `--prove-layer2-can-fail hashvault` control, but it had already
written the files `capture_tr_pathological.py` needs before that FATAL,
so the driver itself ran and matched — this FATAL is outside the S4 diff and
outside this brief's scope, not investigated further.)

All wallet ids and shot counts above match the fold report's claims exactly —
independently recomputed, not transcribed.

Both source worktrees re-confirmed clean and at the cited tips after every
check (identical to the block at the top of this report).

---

# Closing counts

**8 of 8 assigned findings VERIFIED as folded, correctly, with no scope creep:
I-1, M-1, M-2, M-3, M-4, N-1, N-3, N-4.**

- 0 findings not folded as decided.
- 0 guards that cannot fail — every guard was independently driven to fail via
  its own mutation (I-1 both attribution failure modes plus the extra
  third arm; M-1's dead-post-condition text confirmed single-sited; M-2's
  `mustNot` fired on a forced violation; M-3's count guard fired on an emptied
  file; N-4's literal-site test fired on a planted second literal).
- 0 hunks outside the eight decisions (§1).
- All whole-repo/whole-tree gates green, independently re-run, exit codes
  recorded directly.

**0 Important / 0 Minor / 0 Nit.** Nothing to fold back.
