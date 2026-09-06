# H6 PLAN — R0 round 0, fidelity + design lens (opus)

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` at engrave master
`e6d84d9c` (verified: `git diff e6d84d9c HEAD -- <plan>` is empty).
**Against:** `design/SPEC_hashlock_H6_preimage_plates.md` at `a2a031fa` (verified likewise).
**Trees:** private `cp -a` copies of `/scratch/code/shibboleth/.tmp/h6-gate`, `-ms`, `-me`,
removed after the run; the gated trees themselves were never written to (`diff -rq` clean
before removal).

**The one question:** if implementers follow this plan literally, task by task in the stated
order and file groups, do the three repos end up doing exactly what the spec says?

**Answer: no, in eight places.** The *content* is very close — every copy string, every
geometry number, every corpus sha, every budget entry and the whole constant-time argument
reproduce exactly (see "What I could not break"). What does not hold is the plan's
**scheduling layer**: the file groups are not disjoint, one task's boundary gate cannot pass
in the declared order, and a normative deliverable is scheduled by no task at all. Two
further findings are content: one lockstep the spec required is absent, and one host line is
affirmatively false about where the operator's passphrase is.

**Counts: 0 Critical / 8 Important / 4 Minor / 2 Nit.**

---

## Method and machine-checks I ran myself

Everything below was executed; no assessment is offered without a command and its output.

- `scripts/h6-plan-blocks-vs-tree.sh` against my own copies: **86 blocks checked, 0 FAIL** —
  the plan's claim reproduces independently.
- `go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/` in my copy: all **ok**.
- `constantTimeQRModules` measured: `(41)=843 (45)=1013 (49)=1199 (53)=1399`, shipped
  `171/266/391/547/684` untouched — exactly §7.2 item 4.
- ECC-L thresholds measured against the fork's own encoder: first byte count reaching each
  dim is `1 / 18 / 33 / 54 / 79 / 107 / 135 / 155 / 193 / 231` — §7.1's table, every row.
- §6.5 reproduced on the gated layout: rungs `19/23/26/31/34/39` chars,
  advances `4.0000/3.3333/2.9333/2.5333/2.2666/2.0000` mm, budget `416000 units = 65.00 mm`,
  QR envelope `31.80 mm` at scale 2, method line **73** chars, worst case
  **10 rows / 30.00 mm text / 63.80 mm total / 1.20 mm spare**, phrase-no-QR fits at 4.4 mm
  and not 5.0, string fits at 6.0. Segment order measured:
  `method, "", path 2, hash …, mk1 stub (template): …, "", phrase` = `2+1+3+1+3 = 10`.
- Corpora: class `3575ccb0…a1c4abaf` **68 rows**, byte-identical in
  `crates/me-cli/testdata/` and `sysw/testdata/`, pinned on both sides; ms
  `4f1819cd…aba21d4` identical on both sides; seam `2c2fbb3f…6bd541b` unchanged and pinned
  on both sides.
- Rust "unchanged" claims verified by diff against `75f00b56`: `decide_sealing`
  **byte-identical**, `preimage_plate` **byte-identical**, `unknown_reason` **byte-identical**.
- H0 guards verified by diff against fork `fb0dd04`: `isStrictMs1` **identical**,
  `codex32.IsPreimage` **identical**, and `gui/ms1_decode.go`, `gui/codex32_polish.go`,
  `gui/singlesig_verify.go`, `gui/multisig_verify.go`, `bundle/verify.go`, `gui/scan.go`,
  `gui/unlock_session.go`, `seal/record.go`, `seal/open.go` all **byte-unchanged**.
  Six live `IsPreimage` call sites, as §14 says; `IsPreimagePlate` has exactly one
  production caller (`sysw/classify.go:141`).
- Host copy exercised on a `me` binary built from the gated tree: §8.1.1, §8.1.2, §8.2.1,
  §8.2.2, §8.2.3 (both bodies and the note), §8.2.4 and §3.6's new bearer arm all print
  **byte-for-byte** as the spec blockquotes them.
- Device copy diffed by hand against §8.3, §8.4a, §8.4b, §8.5, §8.8, §9 and §10.1's two
  arms: **byte-for-byte**. §8.9's arithmetic re-counted from the tree: the stage adds
  **20** `composerCopy*` bodies, **13** blockquoted, **7** not — exactly as stated.

---

## Findings

### I-1 — The parallel-group table is wrong on both axes: three `gui` groups write the same three files, and F's stated dependency understates what it needs

**Plan section:** "Parallel groups, by DISJOINT file lists" (`:186-198`); Task 10 header
(`:3133`); Task 11 header (`:3346`).

**Counterexample (files).** The table says

> `| **G** | 11 | gui/freetext_flow.go, gui/passphrase_flow.go, gui/sysw_session.go — disjoint from every other group |`  (`:197`)
> `| **F** | 10 | gui/composer_door.go + a new file; disjoint from E once E has landed hashlockHeld |`  (`:196`)

Measured in the gated tree, groups **E (Task 9)**, **F (Task 10)** and **G (Task 11)** all
write `gui/composer_copy.go`, and all three write `gui/composer_copy_test.go`
(`composerCopyTable` lives there, and `TestComposerCopyTableCoversEveryBody` requires a row
per body). E and F and G all write `gui/modal_fits_test.go`:

```
$ grep -n "HashlockPlatesNotCut\|HashlockPlatesEmpty\|HashlockPhraseNotPassphrase\|AbortNoPreimage" gui/modal_fits_test.go
352:  {"H6 §8.4a, no preimage plate was cut", composerCopyAbortNoPreimage()},        <- Task 9  (E)
358:  {"H6 §5.2, the Hashlock plates flow's own abort", composerCopyHashlockPlatesNotCut()},  <- Task 10 (F)
359:  {"H6 §5.2, the empty-payload refusal", composerCopyHashlockPlatesEmpty()},     <- Task 10 (F)
360:  {"H6 §8.8, the Password-program notice", composerCopyHashlockPhraseNotPassphrase()},    <- Task 11 (G)

$ grep -n "^func composerCopyHashlockPlates\|^func composerCopyPreimagesLoaded" gui/composer_copy.go
689:func composerCopyHashlockPlatesLead(n int) string {     <- Task 10 (F), in a file its Files list omits
699:func composerCopyHashlockPlatesEmpty() string {
708:func composerCopyHashlockPlatesNotCut() string {
716:func composerCopyPreimagesLoaded(n int) string {
```

Task 10's own **Files** list (`:3136-3138`) names neither `gui/composer_copy.go` nor
`gui/composer_copy_test.go` nor `gui/modal_fits_test.go`, yet its four bodies and their six
table rows live in them. The plan's own File Structure table already records half the
collision — `| fork gui/modal_fits_test.go | Modify | 9,11 |` (`:181`) — so the two tables
contradict each other.

**Counterexample (ordering).** F is declared "disjoint from E once E has landed
`hashlockHeld`" (i.e. after Task **8a**). Measured: `gui/composer_hashlock_plates.go` uses
**five** symbols declared in Task **9**'s new file `gui/composer_preimage_plate.go` —
`composerHashlockPlateFor`, `composerPreimagePlateRows`, `hashlockPlate`,
`hashlockPlateChoice`, `hashlockPlateLocator`. F cannot compile until E is complete, not
until 8a is. Symmetrically, neither Task 8b nor Task 9 names Task 6 or Task 7 as a
prerequisite, and both use them (`sysw.PhraseRecord`, `sysw.ClassPreimage`,
`sysw.ParsePhraseRecord`, `backup.Hashlock`, `backup.EngraveHashlock`).

**Why it matters.** The constellation's parallel-agent rule is "writers need their own
worktree AND disjoint files". Dispatching E, F and G from this table produces three
implementers editing `gui/composer_copy.go`, `gui/composer_copy_test.go` and
`gui/modal_fits_test.go` concurrently — and the two test files are *tables*, where a
three-way merge silently drops a row and the suite still reports ok, because a dropped row
is a body with no gate rather than a compile error.

**SUGGESTION.** Rewrite the group table from the measured file set. The honest shape is
either (a) E, F and G are **one sequential group** (they share three files), or (b) the four
shared table files are carved out into a final "copy + gates" task owned by one implementer,
with E/F/G each landing only their own screens. State F's real predecessor as Task 9, and
state E's as Tasks 6 and 7. Add `gui/composer_copy.go`, `gui/composer_copy_test.go` and
`gui/modal_fits_test.go` to Tasks 10 and 11's Files lists.

---

### I-2 — Task 6's declared boundary gate cannot pass in the declared order

**Plan section:** Task 6 header (`:1743`, "**Repo:** the fork. **After Task 4**"), Task 6
Boundary gate (`:2166`, "`go test ./backup/`"); group table `| **C** | 4, 5 in parallel,
then 6 |` and `| **D** | 7 |`.

**Counterexample, RUN.** I built a tree that is the gated fork with `hashlock/` reverted to
`fb0dd04` — i.e. Task 7 Step 6 (which vendors the seven `qr_text` rows) not yet done — and
ran Task 6's own gate:

```
### hashlock/ at fb0dd04 (Task 7 Step 6 NOT done):
ok      seedhammer.com/hashlock 0.229s
### Task 6's own boundary gate, go test ./backup/ :
--- FAIL: TestHashlockQRTextMatchesTheMSCorpus (0.00s)
    hashlock_test.go:462: the corpus carries 0 qr_text rows; H6 §11.2 pins seven
FAIL    seedhammer.com/backup
```

`backup/hashlock_test.go:444` reads `../hashlock/testdata/hashlock-v0.8.json` and requires
≥7 `qr_text` rows. Those rows arrive only at **Task 7 Step 6** (`:2316-2324`), which the
plan schedules in group **D**, *after* group C's Task 6. So Task 6 is scheduled before the
thing its gate depends on, and the plan's Task 7 Step 6 even says so from the other side
("Task 6's `TestHashlockQRTextMatchesTheMSCorpus` is what CONSUMES those rows") without
drawing the ordering conclusion.

**SUGGESTION.** Either move the ms-corpus re-vendor out of Task 7 into its own step that
runs with Task 1's release (it is purely a copy of Task 1's output and has no fork
dependency), or state Task 6 as "**After Tasks 4 and 7 Step 6**" and move it out of group C.
The first is better: the re-vendor is Task 1's product and every fork task that reads the
corpus then has it.

---

### I-3 — `hashlock/hashlock.go` is a normative deliverable of the stage that no task schedules

**Spec:** §8.6 rule 4a — "`hashlock.MethodLine(hardened bool)` and
`hashlock.QRText(hardened bool, phrase string)` **are deliverables of this stage**".
**Plan:** the File Structure table (`:133-184`) has **no row** for `hashlock/hashlock.go`,
and no task's Files list names it.

```
$ grep -n "hashlock/hashlock\.go" design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md
3362:**The predicate is `hashlock.IsMS1Shaped`** (`hashlock/hashlock.go:122-148`),
```

— one citation of a *shipped* function, and nothing else. Measured against `fb0dd04`, the
gated tree modifies `hashlock/hashlock.go` (adds `MethodLine` and `QRText`, 30 lines) and
creates `hashlock/methodline_h6_test.go`; neither file appears in any Files list. Task 9's
Interfaces block does flag the gap in prose ("**AND IT NEEDS `hashlock.MethodLine` AND
`hashlock.QRText`, WHICH NO TASK WIRED**", `:2738`) and then does not schedule it either —
Task 9's Files list is `gui/*` only.

This is precisely the blind spot the plan's own checker prints on every run: its last
"NOT COVERED" line is *"files the plan modifies without carrying a block for them"*.

**SUGGESTION.** Add a File Structure row `fork hashlock/hashlock.go | Modify | 9 |
MethodLine, QRText (§8.6 rule 4a)` and `fork hashlock/methodline_h6_test.go | Create | 9`,
and put both in Task 9's Files list — or, better, give them their own step in **Task 7**
(the task that already owns package `hashlock`), which also removes one more E↔D coupling.

---

### I-4 — §8.6 rule 4a's "pinned against the vendored `qr_text` rows, not against themselves" is not delivered; the production functions are pinned to a literal in their own test

**Spec §8.6 rule 4a (verbatim):** "They are DOWNSTREAM of the Rust primary and are **pinned
against the vendored `qr_text` rows (§11.2), not against themselves**."

**What the gated tree does.** `hashlock/methodline_h6_test.go` is the only test that pins
them, and it never opens the corpus:

```go
func TestH6MethodLineAndQRTextMatchTheRustPrimary(t *testing.T) {
	const wantHard = "method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"
	if got := MethodLine(true); got != wantHard { … }
```

```
$ grep -rn "qr_text" hashlock/          # the package that OWNS the corpus copy
(no output)
$ python3 -c "import json;print(len(json.load(open('hashlock/testdata/hashlock-v0.8.json'))['qr_text']))"
7
```

Seven `qr_text` rows sit in that package's own testdata, unread by it. The corpus lockstep
that does exist — `backup/hashlock_test.go:443` — pins `backup`'s **test-local** constants
`h6HardenedMethodLine`/`h6QRText`, which production never calls. Production calls
`hashlock.MethodLine`/`hashlock.QRText` (`gui/composer_preimage_plate.go:290,292`).

**Counterexample, RUN.** I simulated exactly the event the pin exists for: the Rust primary
changes the *shape* of the method line with no change to the derivation constants (I
reordered `salt=` and `iterations=` — same 73 characters), re-vendored the corpus and
updated `corpusSHA256`, as a legitimate re-vendor would:

```
$ go test ./hashlock/
ok      seedhammer.com/hashlock 0.237s          <-- GREEN with a corpus it now disagrees with
$ go test ./backup/ -run TestHashlockQRTextMatchesTheMSCorpus
--- FAIL: TestHashlockQRTextMatchesTheMSCorpus
    row anchor-hardened:
     got "hashlock v1\nmethod: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32\n…"
    want "hashlock v1\nmethod: pbkdf2-hmac-sha256 salt=ms-hashlock-v1 iterations=100000 dklen=32\n…"
```

The failure names `backup`'s literal. Update that one literal — the only thing the failure
points at — regenerate the goldens, and:

```
$ go test ./hashlock/ ./backup/ ./engrave/
ok  seedhammer.com/hashlock   ok  seedhammer.com/backup   ok  seedhammer.com/engrave
$ # production, unchanged:
production MethodLine(true) = "method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"
```

Green suite, and every phrase plate cut from that firmware carries a method line and a QR
text that disagree with the ms corpus and with `ms hashlock`. The `gui` test cannot catch it
either — `composer_preimage_plate_test.go:632` compares `desc.Method` against
`hashlock.MethodLine(false)`, i.e. production against production.

**SUGGESTION.** Give `hashlock/methodline_h6_test.go` the corpus, not a literal: read
`testdata/hashlock-v0.8.json`'s `qr_text` rows and assert
`QRText(row.method=="hardened", row.phrase) == row.qr_text` and `len(...) == row.bytes` for
all seven, exactly as `backup/hashlock_test.go` does. That is a ~15-line change and it makes
the test's own name true.

---

### I-5 — The constant-time budget argument covers v6–v9, but the plate that motivates it also emits v3–v5 codes whose budgets were fuzzed over a different content class

**Spec §7.2 item 4 / §11.3; plan Task 4 Step 3 and Step 5.** The campaign and the in-suite
row both scope themselves to the *newly admitted* versions:

```go
// engrave/h6_qr_test.go, TestConstantTimeQRBudgetBoundsEveryPayload
for _, dim := range []int{41, 45, 49, 53} {
```

and every arm's comment says the number was "Derived by fuzzing … over **§8.6-SHAPED
PAYLOADS ONLY** … which is the only content this plate ever carries".

**Counterexample, measured.** §8.6-shaped text is `21 + len(method) + len(phrase)` bytes, so
it reaches the *shipped* versions routinely — a `sha256` plate with a 44–71-character phrase
is 79–106 bytes, i.e. **dim 37**, and a `hardened` plate with a 1–12-character phrase is
95–106 bytes, also dim 37. Enumerating every `(method, phraseLen)` pair for phrase lengths
1–100 and fuzzing 40,000 §8.6-shaped payloads per dim in the gated tree:

```
dim 21: NO §8.6-shaped payload reaches it (budget 171)
dim 25: NO §8.6-shaped payload reaches it (budget 266)
dim 29 v3: 18 shapes reach it, 40000 payloads, observed max 359, SHIPPED budget 391, headroom 32
dim 33 v4: 25 shapes reach it, 40000 payloads, observed max 489, SHIPPED budget 547, headroom 58
dim 37 v5: 40 shapes reach it, 40000 payloads, observed max 641, SHIPPED budget 684, headroom 43
```

No violation was found, so this is a gap in the argument rather than a demonstrated failure —
but the argument is the whole point of §7. The v5 entry is `664 + 20` where its own shipped
comment records the trend-implied true maximum as **~682**, i.e. ~2 modules of margin above
the trend; it was derived from 18.5M *passphrase-shaped* executions, and H6 hands it a new
content class that no campaign and no suite row samples. §7.2 item 4's own reasoning —
"a budget between the observed min and max makes the plate cuttable for some phrases and
refused for others AT THE SAME QR VERSION — an operator-visible, content-dependent failure
on the funds path" — applies verbatim to v3/v4/v5 and is applied only to v6–v9. The
`TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes` pin asserts the five shipped entries are
*untouched*, which is a different property from *sufficient*.

**SUGGESTION.** One line in Task 4 Step 5: extend
`TestConstantTimeQRBudgetBoundsEveryPayload`'s dim list to `{29, 33, 37, 41, 45, 49, 53}`
(measured above: it passes today, with 32/58/43 modules of headroom, and it fails loudly if
that ever stops being true). If §7.2 item 4's standard is to be applied consistently, add a
short §8.6-shaped campaign at dim 37 to the plan's Step 3 table and say whether 684 stands;
if it does not, v5 gets an H6 buffer the way v7 got 53. Either way, say in §7.2 item 4 which
dims §8.6 content reaches, because "the only content this plate ever carries" is currently
attached to four arms out of nine.

---

### I-6 — §8.2.4 tells the operator the passphrase is "above"; it is below, and the line printed immediately above it says so

**Spec §8.2.4 / §3.3.** The spec places warning 4 after the sealing line and justifies it
(`:390-392`): "Warning 4 prints **AFTER** the sealing line, because its own wording (*'the
device needs the passphrase above'*) refers to it." The plan implements that placement
faithfully (`main.rs:1738-1741`, `if sealing { report_sealed_preimage(&recs); }`).

**Counterexample, RUN** on a `me` built from the gated tree, the *default* path
(`--pack-preimage`, no `--no-passphrase`):

```
sealing:  SEALED — this payload holds secret material (record 0 (hashlock preimage plate)), so it is encrypted
      and opens only with the passphrase below. Pass --no-passphrase to write it in
      cleartext instead.
me: this payload is SEALED and holds a hashlock preimage, so the device needs the passphrase above before it can reach it. `me seal` — the Sealed Payload container — refuses a preimage plate outright; this one does not.
passphrase — write this down and store it APART from the machine:

    series give fitness ahead monkey hawk impulse sword fragile wealth desk shine
```

Two consecutive lines point in opposite directions at the same passphrase, and the shipped
one is right: the passphrase is generated at `main.rs:1758-1768`, *after*
`report_sealed_preimage`. The `--passphrase-ask` prompt is also after it. So "the passphrase
above" refers to nothing at the moment it is printed, on the ceremony screen whose whole job
is to get the operator to write that passphrase down.

The spec's justification for the ordering is the source of the error: it reads "the
passphrase above" as referring to *the sealing line*, but the sentence's noun is a
passphrase, and the sealing line contains none.

**SUGGESTION.** One word: "…the device needs the passphrase **below** before it can reach
it." That keeps §3.3's ordering rule (warning 4 after the sealing line, which is what makes
"SEALED and holds a hashlock preimage" a follow-on rather than a non-sequitur) and makes the
sentence true. Moving the call after the passphrase block is the alternative, but it puts a
paragraph between the passphrase and the `strength:` line that comments on it.

---

### I-7 — The step-(A) Back contract: the plan's prose and the code the plan gated say opposite things

**Plan prose, Task 9 Step 1 (`:2777-2780`):**

> **Back contract:** Button1 is `do not cut` for the highlighted plate, matching
> `composerPickScreen`'s shipped decline arm; **backing out of the STEP returns to the
> engrave step's entry and thence round `composerFlow`'s loop with the state intact**, which
> is §2.2 item 4 unchanged.

**The gated code, `gui/composer_preimage_plate.go:140-143`:**

```go
// BACK IS `do not cut` FOR THE HIGHLIGHTED PLATE, matching composerPickScreen's
// shipped decline arm. Backing out of the STEP is not offered here: §2.2 item 4
// keeps the composition intact through composerFlow's own loop, and a Back that
// unwound the whole step would leave the operator no way to answer the question
// for the remaining digests.
```

Measured: `composerPickScreen` returns `(0,false)` on Button1, and
`composerPreimagePlatePick` maps `!ok` to `hashlockPlateDecline` — it has no failure return
at all, and `composerPreimagePlateStep` has none either. So there is **no** Back out of step
(A) on any screen, first or otherwise. The spec carries the plan's prose verbatim (§5.3 step
(A), `:878-882`), so both documents assert a control the gated implementation deliberately
does not build, and the deviation is recorded only inside a Go comment — not in the plan's
prose, not in "Build gate folded here", and not in the spec's `## Plan-round fold`.

**Operator consequence.** Reaching the pick step commits the operator to answering for every
held digest; the first escape is Button1 on the census one screen later, which costs every
pick decision already made and is not signposted anywhere. That is a real path, not a
hypothetical: `composerEngraveStep` runs the form pick and the FULL/WATCH-ONLY pick *before*
step (A), so an operator who realises at the pick screen that they chose the wrong engrave
mode has no way back to it.

**SUGGESTION.** Pick one and write it down in both documents. The code's behaviour is
defensible; if it is kept, delete the "backing out of the STEP returns to the engrave step's
entry" clause from plan `:2777-2780` and spec §5.3 step (A), and replace it with the code
comment's reasoning plus the fact that the census's Button1 is the exit. If the spec sentence
is kept, `composerPreimagePlatePick` needs a third return state and `composerPreimagePlateStep`
an abort, which is a larger change and should be a decision rather than a comment.

---

### I-8 — Task 1 Step 6 is the stage's only irreversible action and is the one step with no gate, no checklist and no named commands

**Spec §3.1 item 2 / §12 item 0** make a *published* `ms-codec` 0.9.0 the stage's FIRST
deliverable, "through `mnemonic-secret/design/RELEASE_PROCESS.md`'s checklist … items 3, 7
and 8 (CI green, `cargo publish --dry-run`, the `ms-codec-v0.9.0` tag) bind unchanged."

**The plan, in full (`:457-464`):**

> - [ ] **Step 6: RELEASE `ms-codec` 0.9.0.** Bump `crates/ms-codec/Cargo.toml`, write the
>   CHANGELOG entry (…), re-vendor, publish. **Task 2 is blocked until this lands** …
>
> **Boundary gate (RUN):** `cargo nextest run --locked` in the ms workspace — **562 tests
> run: 562 passed, 11 skipped.**

```
$ grep -n "RELEASE_PROCESS\|cargo publish\|ms-codec-v0.9.0\|MIGRATION" <plan>
(no output)
```

The plan never names the process document, the dry-run, or the tag. Its boundary gate is
`cargo nextest run` alone, which is a strict subset of checklist item 3 (`cargo build`,
`cargo test`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check` across
stable + beta + MSRV 1.85). Checklist item 5 (MIGRATION.md on an API change — this release
adds five public items to `ms-codec`) and item 6 (cross-repo notification — this release is
consumed by *two* siblings) are not mentioned in either document. And "re-vendor" does not
name `ci/repro/vendor-freshness.sh`, which exists in that repo and whose omission is a
recorded trap in this constellation.

A crates.io publish cannot be undone: a wrong 0.9.0 is a burnt version number and a
`me`/fork dependency pinned to it.

**SUGGESTION.** Expand Step 6 into the checklist, with commands: bump; CHANGELOG entry
recording `4f1819cd…aba21d4`; `cargo fmt --check && cargo clippy --all-targets -D warnings
&& cargo nextest run --locked`; `ci/repro/vendor-freshness.sh`; MIGRATION.md section for the
five new public items; `cargo publish --dry-run`; tag `ms-codec-v0.9.0`; push; then publish.
Make the boundary gate that list rather than `cargo nextest run`.

---

### M-1 — `ConstantQR`'s new bound comment names 192 bytes as the admitted version's cap; measured, v9 caps at 230 and 192 is v8's

**Spec §7.2 item 3 (`:1346-1348`)** asks for "the new pair (v9, 192 bytes at the last full
step below 194)", and the plan's Step 2 block writes it into the code:

```go
if dim > 53 {
    // The bound is v9 (dim 53), which is what the H6 hashlock phrase plate
    // needs: ECC-L caps at 192 bytes at the last full step below the
    // 194-byte worst case …
```

Measured against the fork's own encoder: the first byte count reaching dim 53 is **193**, and
the first reaching dim 57 is **231** — so **v9 holds up to 230 bytes**; 192 is v8's cap. The
shipped v5 comment this replaces pairs the *admitted* version with *its own* capacity ("ECC-L
caps at 106 bytes and the passphrase caps at 100"), and §7.1's table uses the same convention
("37 (v5, today's cap, 106 bytes)").

As written the comment reads as "the bound holds 192 bytes and the content needs 194" — i.e.
already over — which would tell the next raiser the bound must move again for exactly the
content it was raised for. The true headroom is 36 bytes.

**SUGGESTION.** "…ECC-L caps at 230 bytes at v9 and the §8.6 worst case is 194 (hashlock v1
+ the 73-character hardened method line + a 100-character phrase). Raise all three together
or not at all." Fix §7.2 item 3's parenthetical to match.

---

### M-2 — §8.2.3's payload-wide note prints once per carrier

`report_preimage_admission`'s no-`hash:`-record branch sits inside the per-carrier loop
(plan Task 3 Step 6, `if hashes.is_empty() { eprintln!(note); continue; }`). Measured on a
payload holding one plate and one `phrase:` record and no `hash:` record:

```
$ me sysw pack --pack-preimage --no-passphrase --in c.txt --out c.bin 2>&1 | grep -c "me: note —"
2
```

Two byte-identical copies of a sentence whose subject is "this payload". §3.3 is explicit
that all four are payload-wide, and §8.2.3's own reason for demoting this case to a note is
that "a WARNING on every single run is how a warning stops being read" — repeating it per
carrier works against that.

**SUGGESTION.** Hoist the `hashes.is_empty()` branch above the loop: print the note once and
skip the per-carrier orphan check entirely (there is nothing to be orphaned from).

---

### M-3 — `ms-cli` keeps a second implementation of the ms1 shape test, and the plan's own comment claims an equality assertion that does not exist

**Spec §3.1 item 1:** "`ms-cli` DELEGATES and keeps its message rendering … and **there is
still exactly one implementation**."

**Plan Task 1 Step 3's block, shipped verbatim into `crates/ms-cli/src/argv_guard.rs:149-152`:**

> // `is_ms1_shaped` below is kept as the crate-local spelling the unit tests drive **and is
> asserted equal to the codec's.**

```
$ grep -rn "is_ms1_shaped" crates/ms-cli/src/
argv_guard.rs:156:fn is_ms1_shaped(s: &str) -> bool {
argv_guard.rs:512,515,518,521:  assert!(… is_ms1_shaped(…))     # four direct assertions
```

There is no such equality assertion. `ms-cli` also keeps its own `MIN_MS1_LEN`,
`BECH32_CHARSET` and `HASHLOCK_PHRASE_MAX_CHARS` beside the codec's.

**Measured, and this is why it is Minor and not Important:** mutating
`ms_codec::hashlock::MIN_MS1_LEN` to 200 leaves all 7 `argv_guard` unit tests green (they
drive the local copy), but the workspace still reds —
`ms-cli::argv_guard_cross_product::every_argv_channel_refuses_with_the_guards_own_text_and_leaks_nothing`
and two others fail. So the delegated path is covered; what is wrong is a false claim in
shipped code and four unit tests aimed at a production-dead function.

**SUGGESTION.** Either add the one assertion the comment promises (a table driven through
both predicates), or delete `is_ms1_shaped` and point its four assertions at
`looks_like_ms1`. Do not leave the comment as it stands.

---

### M-4 — A wrong-id record draws §8.2.2 ("this payload holds no preimage plate") beside §8.1.2 ("is a kind-0x03 preimage payload")

§8.1.1's own rule is that a wrong-id record "gets §8.1.2 straight away … and sees ONE
refusal". Measured with `--pack-preimage` on a kind-`0x03` single under id `test`:

```
me: --pack-preimage was passed and this payload holds no preimage plate and no `phrase:` record. Nothing was admitted that would otherwise have been refused.
me: record 0 (records count from 0) is a kind-0x03 preimage payload whose 4-character id is not `hash`. …
```

Both lines are individually true (§8.2.2 keys on *classified carriers*, §8.1.2 on the shape),
and there is only one refusal — but the pair reads as a contradiction on the one path §4.3
exists for. **SUGGESTION:** suppress §8.2.2 when `admit_check` is about to refuse, or narrow
its wording to "no preimage plate was **admitted**".

---

### N-1 — Task 12 says "three times" twice and records four runs

Header (`:3540`) "**WIRED AND RUN — THREE TIMES**" and Step 2 (`:3562`) "run it three times",
against the boundary-gate row "four runs — (a) ok=true; (b) RED; (c) RED; (d) ok=true" and
the Build-gate section's "unmutated, then twice against a mutation, then unmutated again".
The four-run shape is the right one (the final unmutated run proves the mutation was
reverted). **SUGGESTION:** say four.

---

### N-2 — Task 7's Files list omits two files its own Step 6 changes

Task 7's Files list names only `hashlock/testdata/hashlock-v0.8.provenance.json` under
"Re-pin", while Step 6 also copies `hashlock/testdata/hashlock-v0.8.json` and edits
`hashlock/hashlock_test.go`'s `corpusSHA256`. The File Structure table (`:168`) has all
three. **SUGGESTION:** make the Files list match the table.

---

## What I could not break

Stated so the fold does not re-derive it, and so the residue is visible.

- **Every copy string is byte-exact.** Seven host bodies exercised on a real binary, ten
  device bodies diffed against the spec blockquotes. §8.9's 20/13/7 arithmetic re-counted
  from the tree and correct.
- **Every geometry number in §6.5 reproduces**, including the corrected 3.3333 mm advance,
  the 1.20 mm of spare, and the `2+1+3+1+3 = 10` body-row order in the order §6.2/§6.3
  require.
- **The constant-time deliverables are sound where they are scoped.** The alignment table is
  derived from the encoder on every run and matches §7.2 for v2–v9; the bound is `dim > 53`;
  the scale-2 arm is asymmetric with the geometry §7.3 derives, emits 5 commands like
  `case 3`, and `ConstantQR`+`Engrave` complete at v9/scale 2; the four budget entries are
  843/1013/1199/1399 with the shipped five untouched; §11.3's split into a fuzzed row and a
  pin is the right shape and the retired mutation is correctly retired.
- **The Rust-first order holds where it is stated.** `preimage_plate` and `decide_sealing`
  are byte-unchanged; `preimage_plate_admissible` carries all three conjuncts and the shipped
  `a_preimage_plate_is_named_not_misdiagnosed` rows pin them; classification is unconditional
  and `Class::Preimage`/`Phrase` are secret and bearer; the argv guard's new hashlock arm
  prints and refuses before the parser runs.
- **Every H0 guard is untouched.** `isStrictMs1` and `codex32.IsPreimage` byte-identical to
  `fb0dd04`; all nine refusal-site files byte-unchanged; the six live `IsPreimage` call sites
  intact; `IsPreimagePlate` answered before `isStrictMs1` with exactly one production caller.
- **All three corpora and all six pins agree across repos**, and the two-repo seam corpus is
  correctly left alone.
- **The walk arm does what §11.7 asks and a little more**: it adds the second keyed path,
  pages to reach §8.3's block, asserts the pick screen is masked (`mustNot(pick, ANCHOR)`),
  and compares the census row's own `path <n> <first8>..<last8>` token — parsed by a second,
  deliberately non-interchangeable parser — against the token the confirm modal drew.
- **`scripts/h6-plan-blocks-vs-tree.sh` reproduces 86/0 independently**, and its blind-spot
  tail is honest — its last line is the one that names I-3.

---

## Closing counts

| severity | n | ids |
| --- | --- | --- |
| Critical | **0** | — |
| Important | **8** | I-1 disjoint-group claim false (files + ordering); I-2 Task 6's gate cannot pass in the declared order; I-3 `hashlock/hashlock.go` scheduled by no task; I-4 `MethodLine`/`QRText` not pinned to the corpus; I-5 the budget argument skips the dims §8.6 content also reaches; I-6 §8.2.4 says "above", the passphrase is below; I-7 step-(A) Back contract, prose vs code; I-8 the release step has no gate or checklist |
| Minor | **4** | M-1 the `dim > 53` comment's 192 bytes; M-2 the note repeats per carrier; M-3 `ms-cli`'s surviving shape test + a false comment; M-4 §8.2.2 beside §8.1.2 |
| Nit | **2** | N-1 "three times" vs four runs; N-2 Task 7's Files list |

Six of the eight Importants (I-1, I-2, I-3, I-7, I-8, and the documentation half of I-5) are
**scheduling and record** defects rather than behaviour: the wired trees are green and the
implementation is largely right, but the plan as a *set of instructions to N implementers*
does not describe the tree it produced. I-4 and I-6 are behaviour, and both are small fixes.
