# H6 implementer D — Task 7 (the two classes, the `phrase:` port, admission)

**Repo:** `bg002h/seedhammer` (the fork). **Branch:** `h6-d`, branched from the
controller-merged integration branch `hashlock-h6` at `872ba06c`.
**Tip:** `aeb1a077` — one commit.
**Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` at engrave
master `3890ad64` (STATUS R0 GREEN), implementing
`design/SPEC_hashlock_H6_preimage_plates.md` (`95418b2c`).
**Go:** 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`.
**Every number below is quoted from a captured run under
`/scratch/code/shibboleth/.tmp/h6-d-logs/`.**

---

## 1. What landed

One commit, `aeb1a077`, *"sysw: ClassPreimage, ClassPhrase, the phrase: record
and admission (H6 Task 7)"*, seven files, +339 / −11:

| file | plan step | what |
| --- | --- | --- |
| `sysw/record.go` | 1 | `ClassPreimage`, `ClassPhrase`, both added to `IsSecret` |
| `sysw/composer_records.go` | 2 | `PhrasePrefix`, `ErrPhraseRecord`, `HashlockMethod` (+`String`), `PhraseRecord`, `ParsePhraseRecord`, `PhraseRecordString`, the `IsComposerRecord` and `classifyComposer` arms |
| `sysw/classify.go` | 3 | `isPreimagePlateRecord`, answered BEFORE `isStrictMs1`; `isStrictMs1` byte-unchanged |
| `gui/sysw_admit.go` | 4 | both classes on `progWalletPolicy` and no other row |
| `sysw/testdata/record_class_vectors.json` | 5 | re-vendored, 47 → 68 rows |
| `sysw/testdata/record_class_vectors.provenance.json` | 5 | re-pinned (see deviation 1) |
| `sysw/composer_records_test.go` | 5 | the `Phrase` class, the 68-row assertion, `Phrase` in the non-vacuity sweep, **plus one added test** (deviation 2) |

`isStrictMs1` is **unchanged** — verified in the diff: its final line is still
`return err == nil && !codex32.IsPreimage(c)`, H0's inertness, and the new class
is answered before it so the two rules never overlap.

The three code files are **identical to the gate agent's wired tree**
(`/scratch/code/shibboleth/.tmp/h6-gate`) except for three deliberate
differences, all in `sysw/composer_records.go`:

- two em dashes in the gate tree's comments are ASCII `--` here, matching the
  plan's own fragment text;
- `HashlockHardened`'s doc comment. The gate tree says *"PBKDF2-HMAC-SHA256 over
  the phrase."*; this says the same and adds the parameters, **verified against
  the code rather than the comment**: `hashlock.PreimageHardened`
  (`hashlock/hashlock.go:46-58`) calls `seal.NewDeriver(phrase, Salt,
  Iterations)`, `seal/pbkdf2.go:9` is *"§7's PBKDF2-HMAC-SHA256, run in SLICES"*,
  `hashlock/hashlock.go:22` is `Salt = []byte("ms-hashlock-v1")` and `:25` is
  `Iterations = 100000`.

`gui/sysw_admit.go`, `sysw/record.go` and `sysw/classify.go` are byte-identical
to the gate tree.

---

## 2. TDD — RED quoted, then GREEN

Order: the fixture and the test first (Step 5), then Steps 1 → 2 → 3 → 4.

**RED 1** (`01-RED-step5.txt`) — the corpus and the test edits applied, no code:

```
# seedhammer.com/sysw [seedhammer.com/sysw.test]
sysw/composer_records_test.go:28:12: undefined: ClassPhrase
FAIL	seedhammer.com/sysw [build failed]
```

**RED 2** (`02-RED-step1.txt`) — Step 1's classes only; the corpus's seven valid
`phrase:` rows classify `Unknown` (0) where the host says `Phrase` (14). All
seven, first three quoted:

```
--- FAIL: TestComposerRecordsClassifyExactlyAsTheHost (0.00s)
    composer_records_test.go:78: phrase-hardened: Classify("phrase:68617264656e65642c636f727265637420686f727365206261747") = 0, want 14 (host's answer)
    composer_records_test.go:78: phrase-sha256: Classify("phrase:7368613235362c636f727265637420686f7273652062617474657") = 0, want 14 (host's answer)
    composer_records_test.go:78: phrase-containing-a-comma: Classify("phrase:68617264656e65642c6f6e652c2074776f2c207468726565") = 0, want 14 (host's answer)
```

**GREEN** after Step 2 (`03-GREEN-step2.txt`): `ok seedhammer.com/sysw 0.059s`.

**RED 3** (`04-RED-preimage.txt`) — the added preimage test written before
Step 3's predicate:

```
--- FAIL: TestPreimagePlateIsItsOwnClassAndOnlyUnderTheHashID (0.00s)
    composer_records_test.go:238: Classify(preimage plate under `hash`) = 0, want ClassPreimage
```

**GREEN** after Step 3 (`05-GREEN-step3.txt`): `ok seedhammer.com/sysw 0.042s`.

*Process note, not a plan deviation:* the first mutation batch was run **before**
committing and reverted with `git checkout --`, which discarded the uncommitted
Steps 1–3. They were re-applied from the same scripted edits and re-verified
GREEN (`07-GREEN-reapplied.txt`) before any commit; the mutation results below
are all from runs made **after** `aeb1a077`, so nothing in them depends on that.

---

## 3. Gates, at the tip `aeb1a077`

| gate | result | log |
| --- | --- | --- |
| **`go test ./sysw/`** — the plan's Task 7 boundary gate | **ok**, **42 top-level tests** (66 including subtests), **0 failures**, 0.039 s | `12-sysw-tip.txt` |
| `scripts/gui-shard-test.sh ./gui/ 24` | **ok — all 1239 tests ran across 24 shards**; partition verified exhaustive `1239 == 1239`; wall 31 s | `08-gui-shards.txt` |
| every other package (`go list ./...` minus `./gui/`) | **54 packages ok**, 0 failures | `09-nongui.txt` |
| `gofmt -l sysw/ gui/sysw_admit.go` | empty | — |
| `go vet ./sysw/` | clean | — |
| `go vet ./gui/` | only the two settled baseline reds (`testing.ArtifactDir requires go1.26 or later`, in `freetext_sizeproof_golden_test.go:111` and `transaction_golden_test.go:104`) — present at `fb0dd04`, not mine | — |
| `git status` at the tip | clean | — |

`gofmt -l` over the whole `gui/` package reports `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` — **checked
and pre-existing**: `git show fb0dd04:gui/transaction.go | gofmt -l /dev/stdin`
also reports it. None of my files is in that list.

**The corpus's own claim, measured:** all **68 rows** classify identically on the
host and on the device, including all **21** `phrase:` rows — **7 valid**
(`hardened`, `sha256`, containing a comma, space after the comma, containing a
colon, one character, 100 characters) and **14 refusals** (101 characters, empty,
64 hex, ms1-shaped, ms1-shaped-grouped, unknown method, uppercase method, no
comma, body not hex, body uppercase hex, body odd length, body not UTF-8, body
empty, non-printable tab). Row counts by class: 50 Unknown / 7 Phrase / 5 Now /
4 Key / 2 Hash.

**Firmware** (`nix develop -c tinygo build -size short -target pico-plus2
-stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`), both
measured on this tree:

| tree | flash | RAM |
| --- | --- | --- |
| branch point `872ba06` | 1,599,680 | 63,232 |
| tip `aeb1a077` | **1,600,272** | **63,248** |
| **delta** | **+592 B** | **+16 B** |

---

## 4. Mutations — eight, each applied, run once, reverted

The plan carries no `MUTATION:` lines for Task 7, so these are the implementer's,
one per new arm. Six red; **two survive and both are reported rather than
patched over.**

| # | mutation | result | the line |
| --- | --- | --- | --- |
| M1 | `IsSecret` drops `ClassPreimage \|\| ClassPhrase` | **RED** | `composer_records_test.go:255: IsSecret: preimage=false phrase=false, want both true -- whoever holds one can spend any key-less hashlock path it unlocks` |
| M2 | cut on the **LAST** comma instead of the first | **RED** | `composer_records_test.go:78: phrase-containing-a-comma: Classify("phrase:68617264656e65642c6f6e652c2074776f2c207468726565") = 0, want 14 (host's answer)` |
| M3 | `codex32.IsPreimage` instead of `IsPreimagePlate` (drop the id) | **RED**, both rows | `composer_records_test.go:242: Classify("ms10entrsqv0qqqqqqqq"...) = 13, want ClassUnknown -- only the id `hash` reaches the flow that engraves (H6 §4.3)` and the same for `ms10testsqvrsu9guyv4...`, the 33-byte BIP-93 collision |
| M4 | drop the `hashlock.ValidatePhrase` call | **RED**, 6 rows | `composer_records_test.go:78: phrase-101-characters: … = 14, want 0 (host's answer)`; also `phrase-empty`, `phrase-64-hex`, `phrase-ms1-shaped`, `phrase-ms1-shaped-grouped`, `phrase-non-printable-tab` |
| M5 | method selector compared with `strings.ToLower` | **RED** | `composer_records_test.go:78: phrase-uppercase-method: Classify("phrase:48415244454e45442c636f727265637420686f727365206261747") = 14, want 0 (host's answer)` |
| M6 | `IsComposerRecord` forgets `PhrasePrefix` | **RED**, all 7 valid rows | `composer_records_test.go:78: phrase-hardened: … = 0, want 14 (host's answer)` |
| M7 | `isPreimagePlateRecord` answered **AFTER** `isStrictMs1` | **SURVIVES** — `ok seedhammer.com/sysw 0.038s` | see below |
| M8 | the `progWalletPolicy` admission row drops both classes | **SURVIVES** — `ok seedhammer.com/gui 0.228s` over `-run 'Admit\|Admiss\|Sysw\|Cells\|Oracle'` | see below |

**M7 — the arm ORDER is not observable on this tree, and that is worth knowing.**
The plan and spec both state the new arm is answered *before* `isStrictMs1* "so
the two rules never overlap". Measured: swapping them changes nothing, because
`isStrictMs1`'s final conjunct `!codex32.IsPreimage(c)` already refuses every
preimage. The ordering is therefore a **robustness** property, not a behavioural
one: it is what keeps the two rules disjoint **if H0's inertness clause is ever
removed from `isStrictMs1`**. No test can distinguish the two orders while that
clause stands, and adding one would be a test that cannot fail. Recorded, not
tested.

**M8 — the admission row has no gate until Group E, by construction, and Group
E's implementer must do one thing.** `gui/sysw_admit_oracle_test.go` reconciles
production `syswOffer(...)`/`.take(...)` **sites** against the `admitted` table;
Task 7 adds no such site (Tasks 8b and 10 do), so the table entry is inert here
and removing it is invisible. Two consequences for the controller:

1. This is expected, not a defect in Task 7 — but it means **nothing in this
   commit proves the admission row is right**; Group E's flows are its gate.
2. **Group E must register its new consumption sites** in `syswConsumers` *and*
   add `"ClassPreimage"`/`"ClassPhrase"` to `classNames`
   (`gui/sysw_admit_oracle_test.go:88-112`). A site naming an unmapped class is
   reported as *"names no sysw.Class constant"* — the file's own comment records
   that this is *"a true failure with a false cause -- the worst kind to debug"*,
   and it is exactly what happened to `ClassMt`/`ClassTx` at composer S3.
   Neither name is in `classNames` today; the gate agent's tree did not add
   them either (`gui/sysw_admit_oracle_test.go` is byte-identical there).

---

## 5. Deviations — two, both deliberate

### D1. The provenance pin's `commit`/`file_commit` are PROVISIONAL

**The brief's stated source for the corpus did not exist.** The brief named
`/scratch/code/shibboleth/me-worktrees/h6-b/crates/me-cli/testdata/record_class_vectors.json`;
there is no `me-worktrees` directory and no `h6-b` worktree — implementer B's
Task 2 has **not landed on engrave master**, whose copy is still the 47-row
`5b3960ca…` at `17ba56c1`.

What I used instead: the plan author / gate agent's engrave tree,
`/scratch/code/shibboleth/.tmp/h6-me/crates/me-cli/testdata/record_class_vectors.json`,
which the brief explicitly permits reading. **Verified before use, not assumed:**
its sha256 is
`3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf` — character
for character the literal the plan (Task 7 Step 5) and the spec (§4.2's corpus
table) both pin — and it holds exactly 68 rows. It is also byte-identical to the
copy in the gate agent's fork tree.

The consequence is the pin's commit fields. No commit anywhere carries the
68-row file yet, so I recorded engrave master at re-vendor time
(`17ba56c1e843ca2a16a9dec4dfdf7ddc05aca6b0`) in both `commit` and `file_commit`,
and **said so inside the file** rather than letting a false provenance claim
stand silently — one line appended to the pin's own `_comment` array:

> PROVISIONAL COMMIT PIN (H6 Task 7): `commit`/`file_commit` name engrave master
> at the time this corpus was re-vendored, NOT the commit that carries the
> 68-row file -- H6 Task 2 had not landed on master yet. Re-record both once
> Task 2 is merged; sha256 and vectors are already the final values.

**ACTION FOR THE CONTROLLER:** after B's Task 2 lands, re-record `commit` and
`file_commit` from `git -C mnemonic-engrave rev-parse HEAD` and `git log -1
--format=%H -- crates/me-cli/testdata/record_class_vectors.json`, and drop that
`_comment` line. `sha256` and `vectors` need no change. Note that the gate
agent's own tree has the same problem with no note — it pins
`75f00b5685c151ad0b3f9c5ba93a4bcc98f5742c`, a real engrave commit whose copy of
the corpus is the **47-row** one, so that pin is simply wrong; do not copy it.

Nothing else about the re-vendor deviates: the file is byte-identical to the
primary's output and `TestComposerRecordsClassifyExactlyAsTheHost` verifies the
sha against the pin on every run.

### D2. One test ADDED — `TestPreimagePlateIsItsOwnClassAndOnlyUnderTheHashID`

**The plan left Task 7's own headline product with no gate on the fork side.**
Step 5's comment says ClassPreimage *"has no CASES row in this corpus and is
pinned by the seam corpus instead"*. Measured, that is not what the seam corpus
does: `sysw/codex32_seam_test.go:65` asserts `Classify(v.String) ==
ClassCodex32Secret` against `device_admits`, and `preimage-plate-0x03` carries
`device_admits: false` — true **before and after** this task. So the seam pins
only the negative. With Task 7's other steps in place and nothing else added:

- `isPreimagePlateRecord` returning `false` unconditionally → `go test ./sysw/`
  **green**;
- `IsSecret` omitting both new classes → **green**.

Both are the class the workflow calls blocking — *a gate that cannot fail*. The
fix is one test **inside a file already on my list** (`sysw/composer_records_test.go`),
so it cannot conflict with another group. It uses the seam corpus's own three
strings, which is what makes §4.3's id partition testable at all:

| string | asserted |
| --- | --- |
| `ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c` (kind 0x03, 33 bytes, unshared, id `hash`) | `ClassPreimage` |
| `ms10entrsqv0qqq…5gz69g08wwtz9` (the same shape under `entr`) | `ClassUnknown` |
| `ms10testsqvrsu9…h3pm4xrfdlvvp` (the 33-byte BIP-93 collision, id `test`) | `ClassUnknown` |
| all three | never `ClassCodex32Secret` — H0's inertness, unchanged |
| `ClassPreimage`, `ClassPhrase` | `IsSecret()` true |

M1 and M3 above are that test firing. Its doc comment records that it was added
by the implementer and why.

---

## 6. Interfaces produced, as the plan's Task 7 lists them

All present and exported (`sysw/`): `ClassPreimage`, `ClassPhrase`,
`PhrasePrefix`, `HashlockMethod`, `PhraseRecord`, `ParsePhraseRecord`,
`PhraseRecordString`; unexported: `isPreimagePlateRecord`. Consumed:
`hashlock.ValidatePhrase` (`hashlock/hashlock.go:93`), `codex32.IsPreimagePlate`
(`codex32/mspayload.go:141` — implementer C's Task 5, present on the branch
point).

`PhraseRecordString` has no caller inside `sysw` and none in the fork yet; it is
consumed by Task 8b's tests (`gui/composer_hash_test.go` in the gate tree calls
it). Flagged so it is not mistaken for dead code before Group E lands.

**Not touched, deliberately:** package `hashlock` (its re-vendor is Task 5b,
already on the branch point at `5f7ed50`), `sysw/codex32_seam_vectors.json` and
its two sha literals (spec §4.2: the row's meaning is unchanged and editing it
would red both suites for no reason — confirmed, the file is untouched and the
seam test passes), and `crates/me-cli/` (Task 2/3, group B).

---

## 7. Summary

Task 7 landed in one commit at fork `h6-d` tip `aeb1a077`. `go test ./sysw/` is
ok at 42 tests; the whole `gui` package is ok at 1239 tests over 24 shards; every
other package is ok at 54 packages; firmware +592 B flash / +16 B RAM over the
branch point. Six mutations red, two survive with reasons given. Two deviations:
the provenance commit pin is provisional and **needs re-recording once B's Task 2
lands**, and one test was added because the plan gave `ClassPreimage` and
`IsSecret` no gate on this side.
