# H6 implementer C — Tasks 4, 5, 5b and 6 (the fork)

**Branch `h6-c`** in the fork, off `main` `fb0dd04`, worktree
`/scratch/code/shibboleth/.tmp/seedhammer-h6-c`. **Tip `872ba06`.** Nothing
pushed; no commit on `main`.

| task | commit | what |
| --- | --- | --- |
| 4 | `2474558` | `engrave`: ConstantQR to v9, the scale-2 arm, the four fuzzed budgets, four goldens, two falsified records |
| 5 | `2da8cbe` | `codex32`: `EncodeMS1Preimage`, `IsPreimagePlate` |
| 5b | `5f7ed50` | `hashlock`: Task 1's corpus re-vendored, `MethodLine`, `QRText` |
| 6 | `872ba06` | `backup`: the dedicated plate layout, ten tests, four goldens, the second falsified record |

Every number below comes from a run at this tip, captured to a file under
`/scratch/code/shibboleth/.tmp/h6c-logs/` and quoted from there.

---

## Gates, as run

| gate | result |
| --- | --- |
| `go test ./engrave/` | **ok**, 36 tests pass (`t4-final.txt`) |
| `go test ./codex32/` | **ok**, 62 tests pass (`t5-boundary.txt`) |
| `go test ./hashlock/` | **ok**, 10 tests pass (`t5b-green.txt`) |
| `go test ./backup/` | **ok**, 141 tests pass, 0 fail (`t6-final.txt`) |
| `gui-shard-test.sh ./gui/ 24` | **RESULT: ok — all 1239 tests ran across 24 shards**, wall 41s (`gui-shards.txt`) |
| every other package (74 packages, gui excluded) | **exit 0**, 54 `ok`, no failure (`repo-nongui.txt`) |
| `gofmt -l` | no file of mine; the pristine baseline is unchanged (see Deviation 7) |
| plan blocks vs tree | **29 of 29** fenced blocks headed `file=fork/…` for my files appear VERBATIM in the tree (script below) |

The block check was run rather than asserted:

```
blocks for my files: 29, verbatim in tree: 29
```

(29 blocks across `engrave/engrave.go`, `engrave/h6_qr_test.go`,
`engrave/engrave_test.go`, `codex32/msencode.go`, `codex32/mspayload.go`,
`codex32/msencode_preimage_test.go`, `hashlock/hashlock.go`,
`hashlock/methodline_h6_test.go`, `backup/hashlock.go`,
`backup/hashlock_test.go`, `backup/passphrase_test.go`.)

### Firmware size

Measured with nix on PATH, the CLAUDE.md recipe, both trees, same session:

```
=== TIP h6-c ===          code 1567596  data 32084  bss 31148 | flash 1599680  ram 63232
=== BASELINE fb0dd04 ===  code 1567356  data 31852  bss 31004 | flash 1599208  ram 62856
```

**Delta: +472 B flash, +376 B RAM.** Small because most of Task 6 is not yet
reachable: no `gui` code calls `backup.EngraveHashlock` or
`codex32.EncodeMS1Preimage` until group E lands, so the linker drops it. What
does survive is Task 4's `qrAlignCentres` table and the four budget arms, which
`ConstantQR` reaches from the shipped passphrase plate.

---

## Task 4 — the constant-time QR encoder to v9, and its scale-2 arm

### RED, quoted (`t4-red.txt`, `t4-red-each.txt`)

The test file was written first and run against `fb0dd04`'s encoder:

- `TestQRAlignmentTableMatchesTheEncoder` — `panic: unsupported qr code version`
  at `engrave/engrave.go:411`, from `bitmapForQRStatic(41)`.
- `TestConstantQRAcceptsThroughV9AndRefusesV10` —
  `ConstantQR(dim 41) = engrave: constant QR size too large: 41, want accepted`,
  and the same at 45, 49, 53.
- `TestConstantTimeQRBudgetBoundsEveryPayload` —
  `constantTimeQRModules(41) = 0: the version is admitted by ConstantQR's bound
  and refused by its budget, which is the C-1 failure`.
- `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes` —
  `constantTimeQRModules(41) = 0, want 843 (823 observed + 20 buffer)…`, and the
  same for 45/49/53.
- `TestConstantQRMoveCountIsAFunctionOfDimAlone` —
  `dim 41: engrave: constant QR size too large: 41`.
- `TestEngraveModuleScale2` — `panic: unsupported module scale`.
- `TestConstantQREngraveAtScale2Completes` —
  `ConstantQR(dim 53): engrave: constant QR size too large: 53`.
- `TestH6ConstantQRGoldens` — all four sub-tests,
  `engrave: constant QR size too large: 41/45/49/53`.
- `TestECCLThresholdsAreWhatTheBudgetAssumes` — **ok at RED**, as expected: it
  pins the encoder's own thresholds, which H6 does not change. It is a pin, not
  a behaviour change, and it is what stops `h6DimForBytes` drifting and quietly
  ceasing to sample a version.

### GREEN, and the plan's own numbers reproduced

The fuzzed row's headroom, over all seven dims §8.6 content reaches, reproduces
the plan's table **exactly**:

```
dim 29 v3: 400 payloads, observed max 347, budget 391, headroom 44
dim 33 v4: 400 payloads, observed max 478, budget 547, headroom 69
dim 37 v5: 400 payloads, observed max 617, budget 684, headroom 67
dim 41 v6: 400 payloads, observed max 787, budget 843, headroom 56
dim 45 v7: 400 payloads, observed max 918, budget 1013, headroom 95
dim 49 v8: 400 payloads, observed max 1134, budget 1199, headroom 65
dim 53 v9: 400 payloads, observed max 1348, budget 1399, headroom 51
```

`v9 at scale 2: 12401 commands over a budget of 1399 modules`. Four new goldens
(`h6-qr-v{6,7,8,9}-scale2.bin`); `git status` confirms **no existing golden
moved**.

The two falsified shipped records were rewritten, not deleted:
`TestConstantQRLargeVersionsFailClosed` now asserts the refusal at v10 (dim 57),
and `TestPassphraseQRFitsSupportedVersion`'s boundary comment no longer calls
dim 41 "deliberately unsupported" — it pins that a 100-character passphrase
still never reaches it.

### Mutations — ten, each run once and reverted

| # | mutation | failing line |
| --- | --- | --- |
| 1 | `qrAlignCentres[53]` (46,26) → (46,28) | `dim 53 marker 3: centre {46 28}, the encoder draws {46 26}` |
| 2 | drop 41 from the single-marker case | `panic: unsupported qr code version` |
| 3 | bound raised to 57 with no v10 row | `panic: unsupported qr code version` (inside `ConstantQR`, which is what the refusal stands in front of) |
| 4 | `case 53: return 1379 + 20` → `1378` | `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer). The original was derived by fuzzing: 7,759,282 in 32 min, converged with 22.6 min of quiet. A different number needs its own campaign.` |
| 4b | the SAME mutation against the fuzzed row | **PASS** — `dim 53 v9: 400 payloads, observed max 1348, budget 1378, headroom 30`. This is the plan's finding, re-measured: no in-suite sample can carry §11.3's stated budget mutation, which is why the PIN exists. |
| 5 | SHIPPED dim-29 entry `386 + extra` → `340` | `dim 29, payload 9: ConstantQR: too many dims 29 QR modules for constant time engraving n: 342 waste: 8` **and** `constantTimeQRModules(29) = 340, want the SHIPPED 391` |
| 6 | ECC-L threshold 193 → 195 | `n=193: table says 49, the encoder says 53` |
| 7 | remove the `case 2` arm | `panic: unsupported module scale` (both `TestEngraveModuleScale2` and the end-to-end row) |
| 8 | symmetric `case 2` arm | `scale 2 p={0 0} axis x: ink [960,4800], want [0,3840]` (see Deviation 3) |
| 9 | revert the bound to 37 | `ConstantQR(dim 53): engrave: constant QR size too large: 53` |
| 10 | per-module command count made content-dependent (skip `engraveModule` on a padding move) | `dim 41: two payloads emitted 6837 and 6927 commands; the toolpath is content-dependent` |

After the last revert `go test ./engrave/` reported **`(cached)`** — the tree is
byte-identical to the green state, which is the revert's own proof.

---

## Task 5 — `codex32.EncodeMS1Preimage` and `IsPreimagePlate`

### RED (`t5-red.txt`)

```
codex32/msencode_preimage_test.go:29:14: undefined: EncodeMS1Preimage
codex32/msencode_preimage_test.go:49:6:  undefined: IsPreimagePlate
… 8 undefined references, FAIL seedhammer.com/codex32 [build failed]
```

### GREEN

All four rows pass; `go test ./codex32/` **ok**, 62 tests. Every value comes
from the vendored corpus, never a literal this file typed.

### Mutations — five, each run once and reverted

| # | mutation | failing line |
| --- | --- | --- |
| 1 | `NewSeed("ms", 0, "entr", …)` | `EncodeMS1Preimage = ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q, want the corpus ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c` — the plan's quoted string, character for character. It also reds the id row: `EncodeMS1Preimage emitted id "entr" for x=525ed2ee…` |
| 2 | drop the `msPrefixPreimage` byte | `EncodeMS1Preimage = ms10hashs4w46…smgvnn7kngecsg` (73 characters), comparison fails; the round trip fails with `codex32: not an m-format secret payload` |
| 3 | drop the id test from `IsPreimagePlate` | `IsPreimagePlate(ms10entrsq…) = true: a kind-0x03 payload under id "entr" was admitted`, and the uppercase row too |
| 4 | `strings.EqualFold` on the id | `IsPreimagePlate admitted the UPPERCASE spelling of a plate` — and **only** that row, which is the point: the case-sensitivity is what keeps this side and the host's `preimage_plate_admissible` agreeing |
| 5 | `append(payload, x[:31]...)` | `DecodeMS1Preimage: codex32: invalid entropy length` |

`IsPreimage` is untouched.

---

## Task 5b — the corpus re-vendor, `MethodLine` and `QRText`

### RED, in two stages — the second is this task's own gate

1. With the test written and neither function present:
   `undefined: QRText`, `undefined: MethodLine`, `FAIL … [build failed]`.
2. With both functions present and the corpus still at `fb0dd04`:
   **`the corpus carries 0 qr_text rows; H6 §11.2 pins seven`** — the plan's own
   stated mutation, reproduced as a live RED rather than injected afterwards.

### The pin, verified before use

`/scratch/code/shibboleth/ms-worktrees/h6-a` carries A's corpus commit
`ffdb77d428d9e5cb55aafe11da163bb2f6bc2de4` ("H6 Task 1 (ms): the phrase rule and
the QR text move into ms-codec"). Verified independently with `sha256sum`
**before** pinning:

```
4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4  hashlock/testdata/hashlock-v0.8.json
```

— the value the plan names. Row counts read from the file, not asserted:
11 derivation, 15 refusals, 1 kind, **7 qr_text**, 4 lockstep. The branch tip
`ca71516` (the 0.9.0 version bump) leaves the blob byte-identical, checked with
`git show <rev>:<path> | sha256sum` at both revisions; the provenance names both.

### GREEN

`go test ./hashlock/` **ok**, 10 tests; `./codex32/` still **ok** against the
re-vendored file.

### Mutations — three, each run once and reverted

| # | mutation | failing line |
| --- | --- | --- |
| 1 | reorder `salt=` and `iterations=` in `MethodLine` | `row anchor-hardened: MethodLine(true) = "method: pbkdf2-hmac-sha256 salt=ms-hashlock-v1 iterations=100000 dklen=32", want the corpus line "method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"` |
| 2 | swap `MethodLine`'s two arms | both classes red: `row anchor-hardened: MethodLine(true) = "method: sha256"…` and `row anchor-sha256: QRText = …pbkdf2…` |
| 3 | corpus back at `fb0dd04` (Step 1 skipped) | `the corpus carries 0 qr_text rows; H6 §11.2 pins seven` |

---

## Task 6 — the dedicated plate layout

### RED (`t6-first.txt`)

Test file first: `undefined: Hashlock`, `undefined: HashlockPhrase`,
`undefined: HashlockQRCode`, `undefined: hashlockLayoutFor`,
`undefined: EngraveHashlock`, `undefined: hashlockFit`, `[build failed]`. With
the implementation in and before `-update`, the golden row red on four missing
files; every other row passed on the first run.

`TestPassphraseQRTooLong` was RED at this tip **before** its fold, which is the
falsification Step 8 exists for:
`passphrase_test.go:784: want an error for a QR beyond ConstantQR's reach, got nil`.

### §6.5's geometry, REPRODUCED (`t6-geometry.txt`)

```
rung | chars/line(79mm) | lines/plate | advance mm
6.0 | 19 | 13 | 4.0000
5.0 | 23 | 15 | 3.3333
4.4 | 26 | 17 | 2.9333
3.8 | 31 | 20 | 2.5333
3.4 | 34 | 23 | 2.2666
3.0 | 39 | 26 | 2.0000
budget = 416000 units = 65.00 mm
QR env scale2 = 203520 units = 31.80 mm ; scale3 = 47.70 mm
method line chars = 73

phrase+QR(scale2)    3.4mm rows=11 text=37.40mm total= 71.20mm fits=false
phrase+QR(scale2)    3.0mm rows=10 text=30.00mm total= 63.80mm fits=true
phrase, no QR        5.0mm rows=16 text=80.00mm total= 80.00mm fits=false
phrase, no QR        4.4mm rows=13 text=57.20mm total= 57.20mm fits=true
string, no QR        6.0mm rows=10 text=60.00mm total= 60.00mm fits=true
```

Every cell of §6.5's two tables reproduces exactly, the corrected 5.0 mm advance
included. The measurement was taken with a scratch test file that was **deleted
immediately after the run** (`ls` confirms its absence); nothing outside my file
list was committed.

Cut durations, also reproducing the plan's numbers exactly:

```
worst-case phrase plate WITH the v9 QR   43m31s
the same plate with the QR removed       14m39s
the string-form plate                    12m14s
the v9 QR alone                          32m12s (dim 53, scale 2)
one 6 mm character                       7s
```

### GREEN

`go test ./backup/` **ok**, 141 tests pass, 0 fail. `grep -c '^func Test'
backup/hashlock_test.go` = **10**, the count the plan pins. Four new goldens; no
existing golden moved.

### Mutations — thirteen, each run once and reverted

| # | mutation | failing line |
| --- | --- | --- |
| 1 | method line grown to 79 characters | `EngraveHashlock(the worst case): backup: the hashlock plate does not fit at any font size: 11 rows at 3.0mm need 427520 units against a budget of 416000` — the plan's string exactly |
| 1b | the spec's own retired mutation, "add one character" | **74, 75, 76, 77 and 78 characters ALL PASS** (`ok seedhammer.com/backup` at each). Re-measured, not taken on faith; 79 is the threshold that fires |
| 2 | `hashlockQRScale` 2 → 3 | `10 rows at 3.0mm need 510080 units against a budget of 416000` |
| 3 | group the MS1 in tens | `the string form laid out at 5.0mm, want 6.0mm`, and the rejoin fails |
| 4 | `strings.ToUpper` the MS1 | `the engraved string is "MS10HASHSQ…" want … verbatim`; `row "MS10HASHSQW46H2AT4W" is not lowercase` |
| 5 | wrap on word boundaries | `100 characters took 1 rows at 39 per line, want 3` — **after** the row was strengthened; see Deviation 2 |
| 6 | draw a locator row in the bottom band | `the bottom band inks 32000 units tall, over 1 3.0mm line(s): a body row is being drawn in it` |
| 7 | `HashlockQRCode` encodes `plate.MS1` | `the QR is not §8.6's text: dim 33 against 41` |
| 8 | `HashlockQRCode` encodes the SpaceMark glyphs | `the QR is not §8.6's text: dim 41 against 41` |
| 9 | drop the QR-form guard | `a QR was accepted on the string form` |
| 10 | one character of `h6HardenedMethodLine` (`v1` → `v2`) | the corpus lockstep reds on every hardened row: `row anchor-hardened:` / `row max-phrase-hardened:` / `row phrase-with-colon:` … |
| 11 | build the text as phrase-then-method | all 7 rows red (14 assertion lines) |
| 12 | return the bottom rung's layout instead of an error | `EngraveHashlock accepted a plate one row over the budget` |
| 13 | production engraving speed cut to a third | `worst-case phrase plate WITH the v9 QR takes 1h14m43s to cut, over the 1h0m0s bound…` — the plan's 1h14m43s, to the second |

---

## Deviations, findings and one incident

1. **Task 5b's provenance pins the CORPUS commit, not a released one.** The plan
   says "record the ms release commit"; at the time of the re-vendor A had not
   published 0.9.0. I pinned `ffdb77d` (the commit that produced the file) and,
   after the controller named the branch tip, amended the 5b commit so the
   provenance also names `ca71516` (the 0.9.0 version bump) with the note that
   the blob is byte-identical there — verified, not assumed. **If ms-codec
   0.9.0 is published from a different tree, this pin needs re-checking** — the
   sha is the thing that must still match.

2. **`TestHashlockBodyWrapsOnCharacters` was strengthened, because its own
   documented mutation did not fire.** As specified (a four-word 28-character
   sample at a 39-character line) a word wrapper puts everything on one row, the
   "every row but the last is full" loop walks nothing, and the row **PASSES**
   on the word-wrap mutation its header describes. Measured. The suite as a
   whole did catch it — the fit gate (`3.4mm: fits = true …, want false`), the
   string form (`the 75-character string took 1 rows at 6.0mm, want 4`) and
   three goldens all red — so the mutation was never invisible, but a row whose
   stated mutation cannot fire proves less than it claims. Added: a
   worst-case single-token sub-case plus a row-count assertion. Re-run under the
   mutation, the row now reds on **both** sub-cases.

3. **Task 4 mutation 8 used a different symmetric arm than the plan's
   illustration.** The plan quotes `ink [-1067,3200], want [0,3840]`; my
   "case 3 shape scaled down" (±sw/2) gives `ink [960,4800], want [0,3840]`.
   Same class, same assertion, different arithmetic — the plan's figure looks
   like it was taken at this package's own `strokeWidth = mm/3`, while the test
   runs at the production 1920.

4. **Task 5 mutation 2's second half.** The plan says dropping the prefix byte
   leaves a string `New()` still parses; confirmed (73 characters), and the
   round trip additionally fails with `errMSBadPrefix` rather than
   `errMSBadLength` — the length error is what the `x[:31]` mutation produces.

5. **Mutation 13 temporarily edited a file outside my list.** Cutting the
   production engraving speed means editing `internal/sh2/params.go`; it was
   reverted immediately and `git status internal/` is clean. Recorded because
   the brief scopes me to a file list.

6. **INCIDENT: one mutation was reverted with `git stash`, which took my
   re-vendored corpus with it.** Mutating "the corpus without its qr_text rows"
   I ran `git stash push hashlock/testdata/hashlock-v0.8.json`, which restored
   HEAD's copy before I had saved mine aside, so my backup captured the OLD
   file. Detected immediately by `sha256sum` after the restore
   (`a46c197a…`, not `4f1819cd…`). Recovered by re-copying from the ms worktree
   and re-verifying the sha; the stash was dropped; `go test ./hashlock/
   ./codex32/` green afterwards. No work was lost, and the constellation's
   standing rule ("never `git checkout` a file with unstaged agent work")
   applies to `git stash` just as literally.

7. **RECORDS CORRECTION — the plan's `gofmt` baseline is understated.** Global
   Constraint baseline red 1 says `gofmt -l` reports three files on the pristine
   fork. Measured at `fb0dd04` with this box's `gofmt` (go1.26.7), it reports
   **five**: `gui/transaction.go`, `gui/transaction_golden_test.go`,
   `gui/transaction_txrecord_test.go`, **`mt/mt.go`** and **`mt/mt_test.go`**
   (the last two committed at `d5fa9fd`). My tip leaves exactly the same five
   and adds no sixth, so the constraint is satisfied — but a future gate that
   checks "three" will read false.

8. **RECORDS CORRECTION — the ArtifactDir vet prediction.** Baseline red 2
   predicts H6 grows the count "from 2 to 6". Measured: `go vet ./engrave/` goes
   **2 → 3**, and repo-wide `go vet ./...` goes **8 → 10** — one new site per new
   golden test (`engrave/h6_qr_test.go:516`, `backup/hashlock_test.go:498`).
   The baseline is 8 repo-wide, not 2; the "+4" would need four new call sites
   and only two exist. Still a pre-existing red, still a `go.mod` directive bump,
   still not H6's to make.

9. **Not mine, flagged for the controller.** Task 6 builds the plate but nothing
   calls it yet: `backup.EngraveHashlock` and `codex32.EncodeMS1Preimage` have no
   caller until groups D and E land, which is why the firmware delta is 472 B.
   The "can a user do the thing" question is answerable only after Task 10.

---

## What this touched

Committed, and nothing else: `engrave/engrave.go`, `engrave/engrave_test.go`,
`engrave/h6_qr_test.go`, `engrave/testdata/h6-qr-v{6,7,8,9}-scale2.bin`,
`codex32/msencode.go`, `codex32/mspayload.go`,
`codex32/msencode_preimage_test.go`, `hashlock/hashlock.go`,
`hashlock/hashlock_test.go`, `hashlock/methodline_h6_test.go`,
`hashlock/testdata/hashlock-v0.8.json`,
`hashlock/testdata/hashlock-v0.8.provenance.json`, `backup/hashlock.go`,
`backup/hashlock_test.go`, `backup/passphrase_test.go`,
`backup/testdata/hashlock-{string-6mm,phrase-noqr,phrase-qr-v9,phrase-space-legend}.bin`.

`git status` at the tip: clean.
