# composer S3 — implementation report

**Agent:** the single S3 implementer, executing
`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` at mnemonic-engrave
`722edbdd340e04a1be3db129129600ba4c2574e5` under
`design/agent-briefs/composer-S3-implementer-brief.md`.

**Worktree:** `/scratch/code/shibboleth/wt-composer-s3`, branch `composer-s3`,
created from fork `main` `321acb56` exactly as the brief says. Nothing was
pushed and nothing was flashed. The main fork checkout was not touched.

**Verdict: all 24 tasks executed (A1–A11, B1–B11, C0–C2). Part A's milestone is
PROVEN. One BLOCKING finding is open — a shipped `cmd/emu` test the plan does
not name, broken by the plan's own Task B11 fence — and one gate could not be
run at all because its toolchain is absent from this machine.**

---

## 0. Two environment facts that shaped the whole run

### 0.1 The pinned Go toolchain did not exist; it was restored from source

The brief names `/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`.
**`/nix` does not exist on this machine at all** — not unmounted, absent:

```
$ ls -d /nix
ls: cannot access '/nix': No such file or directory
$ pacman -Q go tinygo
error: package 'go' was not found
error: package 'tinygo' was not found
```

Remnants survive (`/etc/nix/nix.conf` dated today, `/etc/profile.d/nix.sh`), and
`/home/bcg/go/pkg/mod` still holds the module cache, so a Go toolchain existed
here recently. There is no Go anywhere on the box, and the module cache holds no
`golang.org/toolchain` entry to bootstrap from.

**Restored, at the pinned version, checksum-verified against go.dev:**

```
$ sha256sum go1.26.7.tar.gz
ffb5f8de10c62550dfddab66b36b57030721e0a44a3218e9e1181d7b59f121ca  go1.26.7.tar.gz
$ curl -sS 'https://go.dev/dl/?mode=json&include=all'   # go.dev's published digest
go1.26.7.linux-amd64.tar.gz ffb5f8de10c62550dfddab66b36b57030721e0a44a3218e9e1181d7b59f121ca
$ /scratch/code/shibboleth/.toolchain/go/bin/go version
go version go1.26.7 linux/amd64
```

It lives at `/scratch/code/shibboleth/.toolchain/go`, outside every repo. The
baseline confirms it is the right toolchain: at untouched `321acb56`,
`go build ./...` is clean and `go vet ./gui/` reports exactly the two
pre-existing `testing.ArtifactDir requires go1.26 or later` findings the brief
excepts. Every command below ran with `CGO_ENABLED=0 GOPROXY=off
GOTOOLCHAIN=local TMPDIR=/scratch/code/shibboleth/.tmp` and default
`-mod=readonly`; `go.mod` was not modified.

**This is an environment repair, not a plan change**, and it is recorded because
a future run on this machine will hit it too.

### 0.2 TinyGo and nix are gone, so Task C2 Step 4 could not run — see §6.4

---

## 1. Commits

25 on the fork branch (`git log --oneline 321acb56..HEAD`), newest first:

```
b300a84 emu: give the three wallet-policy screenshot walks the composer's door step (composer S3 task C2)
7e409e6 gui: print SPEC section 13 item 1's four numbers from one measurement test (composer S3 task C1)
cf47e8b gui: the section 12 item 5 gates section 8m, 8c and 8r never had, and a test per surviving mutation (composer S3 task C0)
621743d gui: join the composer -- sources, seating, mapping review, consent, forms, minting and census, in section 7's order (composer S3 task B11)
90071f0 gui: the section 7f form choice, and one secret form that actually has a builder (composer S3 task B7)
e67e1c3 gui: the seating vector leg -- minted cards seat through the SHIPPED seater and reproduce the keyed policy's addresses (composer S3 task B10)
8479262 gui: the plate census over card chunks, and a ceiling found by search (composer S3 task B9)
687432d gui: mint and re-mint the composer's key cards, with both stubs appended (composer S3 task B8)
d3d7f70 gui: section 7e's self-check on the DECODED md1, provoked by fault injection (composer S3 task B6)
2a837d4 gui: slot-directed seating, all-or-nothing, with section 8p's counts (composer S3 task B5)
7f24a23 gui: the mapping review -- verbatim origins, the section 4f invariant, C29 and the two-paths line (composer S3 task B4)
caa1273 gui: discard every seat when the shape moves slot numbering, and only then (composer S3 task B3)
7683662 gui: seeds as a composer key source, at one account per slot per master (composer S3 task B2)
e2df9b1 gui: the composer's seatable keys, and an oracle that can see set-shaped consumption (composer S3 task B1)
dbc40a9 gui: the composer's consent surface and a flow that builds with part B absent (composer S3 task A11)
2b80c7d gui: the six archetype presets, pinned to the Rust primary's exported vectors (composer S3 task A10)
0799f93 gui: the paged stub-teaching screen, and one label for one value (composer S3 task A9)
0e07bd6 gui: the composer's shape flow -- wrapper, path list, picker bounds, the five section 4e refusals (composer S3 task A5)
993577f gui: hashlock entry from a payload record or 64 typed hex, with the 32-byte rule at entry (composer S3 task A8)
e680545 gui: lock entry -- kind, unit, digits, echo, and the device's own section 4c gate (composer S3 task A7)
c5cdd44 gui: a digits-only entry pad for the composer's lock operands (composer S3 task A6)
e555f27 gui: the Wallet Policy door, in every state, with its key-state lines (composer S3 task A4)
2213d0d gui: admit the composer's classes at Wallet Policy, and retire the two comments C12 falsifies (composer S3 task A3)
511f8b1 gui: a paged list primitive for the composer, with one measure site (composer S3 task A2)
0e8e61d gui: the composer's 39 operator strings, verbatim from SPEC section 8 (composer S3 task A1)
```

Two in mnemonic-engrave, on `master` (the plan assigns both):

```
93f009f spec: fold the four things S3 measured -- the paged capacities, the door's key state, the flag screens, the secret's plate form
1e96e9a followups: F-460 records C7's comment-only deprecation of Multisig Build
```

Every commit is `-s` signed-off and carries the two trailer lines. Paths were
staged explicitly; no `git add -A`.

**Fences were not hand-transcribed.** A per-task extractor
(`/scratch/code/shibboleth/.tmp/s3/apply.py`) reproduces
`scripts/plan-build-gate-go.sh`'s semantics exactly — `cur` persists past a
consumed fence, only ` ```go ` fences are taken, blocks for one path
concatenate, `Replace` drops earlier ones. Validated against the plan's own
published numbers before use: **47 fences read, 46 kept across 43 files.** The
first draft (resetting `cur`) counted 46/43 and would have silently dropped
Task B4's SECOND `composer_review.go` fence; matching the gate caught it.

---

## 2. Per-task fail-then-pass evidence

Every task ran its Step-2 failing test first and its Step-N passing test after.
Only deviations from an **Expected** line are quoted below; where a task is
listed with no deviation, its Expected line was met verbatim.

| task | fail-first | pass | deviation from Expected |
| --- | --- | --- | --- |
| A1 copy | `undefined: composerCopyPackedHeightBound` +38 | 3/3 PASS | none |
| A2 paged | `undefined: composerPageLines/PickScreen/ReadScreen` | 2 PASS | **YES — see 3.1** |
| A3 admission | `multisig_build.go does not carry "Build a new policy"`; `sysw_admit.go still claims …` | all PASS incl. shipped `TestEverySyswConsumptionSite…`, `TestTheSeamPassphraseOffer…` | **YES — see 3.2 (F-454 collision), 3.3 (gofmt), 3.9 (`ms` version)** |
| A4 door | `undefined: composerDoorLines/Counts/Flow` | 4 PASS, 6+2 sub-tests | none |
| A5 shape | `undefined: composerMaxKeysForPath` … | 7 PASS, 6+4 sub-tests | **YES — see 3.4 (commit order)** |
| A6 digitpad | `undefined: composerDigitKeys, composerDigitEntry` | 3 PASS | none |
| A7 lock | `undefined: composerLockEcho, composerLockBelowBound` | 5 PASS, 16+8 = 24 sub-tests | none |
| A8 hash | `undefined: composerHashRow, composerPayloadDigests, composerHexKeys` | 4 PASS | none |
| A9 stub | `undefined: composerStubLines/Flow`, then `Template-ID: deadbeef` | 9 PASS incl. shipped `TestTemplateConsentLines`, `TestWalletPolicy*` | none |
| A10 presets | n/a (authored with impl, as the plan directs) | 4 PASS, 6 pinned + 6 unpinned sub-tests | **YES — see 3.5** |
| A11 consent+flow | `undefined: composerConsentLinesFor, composerFlow` | Part-A milestone, see §4 | none |
| B1 sources | `undefined: composerKeySources, composerSeatPrompt, composerSlotOrder` | 5 PASS + oracle | **YES — see 3.6 (imports), 3.7 (ClassMt/ClassTx)** |
| B2 seeds | `undefined: composerSeedAccountFor, composerSeedHook` | **4** PASS (the plan predicts 4 and says why) | none |
| B3 discard | — (see 3.8) | 3 PASS | **YES — see 3.8** |
| B4 review | `undefined: composerInvariantViolation, composerDuplicateXpub, …` | 4 PASS, 5 sub-tests | none |
| B5 seating | `undefined: composerAssignableSlots/UnfilledSlots/SeatingComplete` | **2** PASS | none |
| B6 self-check | `undefined: composerSelfCheck, composerConsentFlow, composerSelfCheckFaultHook` | **3** PASS, 4 fault sub-tests | none |
| B7 forms | `undefined: composerForm, composerFormTemplateOnly` | 4 PASS, 3 sub-tests | **YES — see 3.4 (commit order)** |
| B8 cards | `undefined: composerMintCard` ×5 | 4 PASS | none |
| B9 census | `undefined: composerDescriptorCeilingChars/PlateFits` | 2 PASS | none |
| B10 vectors | n/a (B8 had landed) | 4 PASS, 5+5+3 sub-tests | none |
| B11 the join | 12 dead functions named; `seating never drew — this is the join` | **4** PASS | **YES — see 3.5** |
| C0 gates | `undefined: …` | **16** then **14** PASS; 14 mutations run, see §5 | **YES — see 3.5** |
| C1 measure | n/a | 4 `SPEC13` lines, see §6.1 | none |
| C2 gates | n/a | see §6 | **YES — see 3.10 (BLOCKING), 3.11, 3.12** |

### All `assertModalBodyFits` headroom numbers logged (§12 item 5 evidence)

| body | drawn | headroom | margin |
| --- | --- | --- | --- |
| §8a key-less path confirm | 166 | 339 | 80 |
| §8b unsorted keys confirm | 173 | 339 | 80 |
| §8h every-path-hashed warning | 131 | 397 | 80 |
| §8o below-bound **date** refusal | 50 | 494 | 80 |
| §8o below-bound **height** refusal | 52 | 494 | 80 |
| §8t date floor refusal | 48 | 513 | 80 |
| §8u relative ceiling refusal | 73 | 476 | 80 |
| §8j discard confirm | 118 | 378 | 80 |
| §8v same-origin refusal | 120 | 436 | 80 |
| §8g at-threshold | 104 | 436 | 80 |
| §8g below-threshold | 107 | 436 | 80 |
| §8k two-paths line | 76 | 476 | 80 |
| §8p shortfall refusal | 38 | 494 | 80 |
| §8q self-check refusal | 85 | 476 | 80 |
| §8l unchecked-policy warning | 153 | 339 | 80 |

**Every one of these that the plan pre-recorded matched to the digit** — §8j
(118/378), §8p (38/494), §8v/§8g/§8g/§8k (436/436/436/476), §8q/§8l (476/339) —
as did B7's three choice-label widths (`The policy itself` 159 px, `Template
plus key cards` 253 px, `Template only (no keys)` 251 px against a 436 px row)
and B9's two ceiling numbers.

---

## 3. Deviations, findings, and everything I had to decide

### 3.1 (Minor, plan text) Task A2 Step 4 says "three PASS lines"; the task creates two

`gui/composer_paged_test.go` declares exactly two test functions. The plan's own
fence comment explains why, in the space where the third used to be:

> THE INK COMPARISON IS GONE, and its removal is the fix rather than a tidy-up.
> … Its replacement is behavioural and in composer_gates_test.go:
> TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage

So the third test the `-run '…|^TestComposerRead'` filter matches arrives with
Task C0. Two PASS at A2 is correct; the Expected line was not updated when
fidelity I-7's fold moved the test. **No code change made.**

### 3.2 (Records defect, fixed by renumbering) F-454 was already taken

Task A3 Step 8 asks to file `F-454 — multisig-build-deprecated-in-favour-of-the-composer`.
`design/FOLLOWUPS.md` already holds an **F-454**, filed after the plan was
written:

```
### F-454 — `me-0.8.1-owed-plus-sign-path-tightening-unreleased` … Filed 2026-09-02 from `composer-S2-exec-review-r0` I-1's Rust-first fold.
```

F-453 and F-455–F-459 are also taken, and each matches what the plan expects it
to be (F-453 the preset vectors, F-455 the secret-form split, F-456 the date
ceiling, F-457 the descriptor renderer, F-458 the ceiling-dispatch tautology).
Only F-454 collided, and the composer's deprecation record was filed nowhere.

**Decision:** filed at the next free number, **F-460**, with the plan's slug,
body, owning phase and tags **verbatim** — only the number differs. Following
the plan literally would have put two `### F-454` headings in one file. No code
cites the number. The renumber and its reason are in commit `1e96e9a`'s message.

### 3.3 (Minor, mine, fixed) A3's admission-row fence left `gui/sysw_admit.go` unformatted

Turning `progWalletPolicy` from a one-line composite literal into a multi-line
one changes gofmt's alignment group, so the neighbouring `progBip85:` needed one
space fewer. `gofmt -w` fixed it and the A3 commit was amended. I record it
because I ran `git commit` inside a `gofmt -l … && …` chain, and `gofmt -l`
exits 0 while naming files — **the chain does not gate the commit.** Every later
task checked `gofmt -l` on the touched files as its own step.

### 3.4 (Ordering, forced) Three tasks could not be committed in the plan's order

The plan's task order does not equal a buildable commit order in three places.
In each, the plan's own text acknowledges the dependency; what it does not do is
say which commit the shared file belongs to.

- **A5 after A6–A8.** `composerShapeFlow` calls `composerLockEdit` and
  `composerHashEdit`, declared by A7 and A8. The plan's A5 Step 6 is titled
  *"Run the tests (after the lock and hashlock tasks land)"*, and A8's Step 4
  says *"The shape flow's own tests now compile and run for the first time"*.
  A6's Step-2 Expected is a **clean** two-symbol failure, which is only
  reproducible before A5's shape file exists — so A5's two files were parked
  while A6, A7 and A8 ran their own fail-then-pass cycles, then restored.
  Committed **A6, A7, A8, A5**.
- **B7 after B11.** `composer_engrave_test.go`'s cut-once test drives
  `composerSecretCards`, which Task B11 declares; the plan says so
  (*"run this test after that task lands"*). B7's production file went in its
  own commit (`90071f0`, message says why), its test file with B11 (`621743d`).
- **B3.** `gui/composer_discard.go` is created in Task **A5**, by the plan's own
  round-1 fix. See 3.8.

Each commit's tree builds and vets clean. The reason is in each commit message.

### 3.5 (Plan gap, closed as the plan directs) A10 was written as blocked, so four places needed its wiring

The brief unblocked A10 and it ran in full. But every fence downstream of it was
authored while it was blocked, so the plan's own artifacts do not account for
the preset screen:

- **The wiring itself.** B11's `Replace gui/composer_flow.go` fence contains no
  `composerPresetPick` call, and `TestComposerEveryScreenFunctionHasAProductionCaller`
  duly listed it as dead. The plan names the exact fix in A10 Step 3 — *"Task
  B11's `composerFlow` calls `composerWrapperPick` and then `composerShapeFlow`
  … `composerPresetPick` goes between them, and the blank route is
  `composerShapeFlow` unchanged — so this task is a fill-in, not a redesign"* —
  and that is what I implemented, declining the picker being §7b's blank route.
- **Four `composerFlow` walks** then stopped at the new screen with
  `the path list never drew. Last frame: "Startfrom?plain-multisig…"`. Each
  gained a Back (blank route) step, with a comment: the C26 walk and
  `TestComposerBackAtThePathListKeepsTheComposition`
  (`gui/composer_flow_test.go`), `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen`
  (`gui/composer_join_test.go`), and `TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit`
  (`gui/composer_gates_test.go`). This is the same shape as the five shipped
  walks gaining a door step at A11 — and, as there, **no walk was deleted or
  skipped**.
- **B11 Step 2's dead list** named 12 functions; the observed list was also 12
  but differed by two: `composerPresetPick` present (A10's), `composerApplyShapeEdit`
  absent (already called by A5's shape flow, per the plan's own round-1 move).
- **C2's counts.** Plan-time 112 top-level / 100 sub-tests; measured **116 /
  112**. The difference is exactly A10's 4 tests and 12 sub-tests. Reconciles.

**A10's own result is the strongest single signal in this run.** All six pinned
(wrapper, preset) pairs reproduced the primary's chunk sets **byte for byte on
the first execution**, so the Go table's six shapes are correct against
descriptor-mnemonic `1dc8d409`, not merely plausible. The six preset parameter
sets were resolved by running `md compose --preset … --json` and checking
`template_with_origins` against each vector's template in
`crates/md-codec/src/test_vectors.rs`, rather than guessed:

```
plain-multisig,2of3 | simple-timelocked-inheritance,older=26280
kofn-recovery,2of3,older=26280 (tr) | tiered-recovery,2of2,1of2,older=26280
hashlock-gated,older=26280,sha256=a8a8…a8
decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=1000000
```

**One addition to A10's Step-2 specification, and it is load-bearing.** The plan
says to compare `md.Compose(p.list).Chunks()` against
`keyed_compose_preset_<a>.phrase.txt`. That phrase file is the **keyed** chunk
set, so an unbound template's chunks can never equal it. The test therefore
takes the same two steps `md/compose_test.go`'s own parity test takes — Compose,
then `Bind` the vector's pubkeys and fingerprints read from `descriptor.json` —
which is what the primary's MANIFEST binding did when the vector was made. The
`md` package's helpers are unexported, so the gui test parses the TLV itself.
Without this the six pinned assertions would have failed for a reason that is
not the shape.

### 3.6 (Plan defect, fixed) Task B1's fence leaves its own tree unbuildable

The plan writes B2's five imports into B1's `composer_sources.go` fence and says
why: *"already in the file's import block, written there by the sources task
above so the two halves of one file are never in a half-imported state."* Go
rejects an unused import, so **after B1 the tree does not compile** —
`"encoding/binary" imported and not used`, and four more.

**Decision:** B1 carries only the imports its own half uses; B2 restores
`encoding/binary`, `errors`, `hdkeychain`, `chaincfg` and `bip39` alongside the
code that uses them. The final file is byte-identical to the plan's assembled
version, both commits build and vet clean, and each task's code sits under its
own message. Recorded in both commit messages.

### 3.7 (Real defect the plan's own widening exposed, fixed) `classNames` lacked `ClassMt`/`ClassTx`

Task B1 Step 4 widens the consumption-site oracle to match `takeAll`/`cardSet`.
That made `transaction.go:payloadTransactions` visible for the first time, and
the oracle then reported:

```
transaction.go:payloadTransactions consumes from the payload without naming a
sysw.Class constant — §13 D7's enforcement is that each site HARD-CODES its one
admitted class …
```

The site **does** hard-code `sysw.ClassMt` (`gui/transaction.go:408`) and
`sysw.ClassTx`. They are simply absent from the test's `classNames` map — which
is precisely the failure mode Task A3's own fence comment describes for the
composer's three: *"a site naming one of these without an entry here is reported
as 'names no sysw.Class constant', which is a true failure with a false
cause — the worst kind to debug."* The plan added the composer's three and not
these two. Both added, with that reason in a comment.

**Consumption sites reconciled: 10 before the widening, 15 after** (the plan
asks for "at least 3 higher").

### 3.8 (Minor, plan self-inconsistency) B3 Step 2 expects a build failure that cannot occur

Step 2 expects `undefined: composerShapeSignature, composerDiscardAssignments`.
Step 3 of the same task says *"`gui/composer_discard.go` is created in **Task
A5**"* — the plan's own round-1 move, so the symbols already exist and the test
passes immediately. Step 4's three PASS were then met, and the §8j headroom
matched the plan's pre-recorded `118 chars drawn in full, headroom 378 chars
(margin 80)` exactly. **No code change made.**

### 3.9 (Observation) The installed `ms` is 0.16.0, older than S1's 0.17.1

A3 Step 1a's parenthetical says `bip48-p2tr` should be listed or "the S1
precondition is unmet". `~/.cargo/bin/ms --version` reports **0.16.0** and does
not list it. **This is not load-bearing for A3:** none of the three verification
commands uses that template, and all three matched exactly — `master_fingerprint
73c5da0a` on both accounts, `composerTestXpubA`/`XpubB` byte-identical, and the
payload blob's descriptor identical to `composerTestDescriptor` including
`#ud8uyjz3`. Recorded so nobody reads the absent template as a failed
precondition.

### 3.10 🔴 **BLOCKING — a shipped `cmd/emu` test the plan does not name, broken by the plan's own B11 fence**

`CGO_ENABLED=0 go test -timeout 20m ./...` (C2 Step 2, **CI's own command**)
exits 1 with exactly two failures, both in `seedhammer.com/cmd/emu`, both the
same cause:

```
--- FAIL: TestBuildFlowNeedlesAreDrawnByExactlyOneFlow
    needle_flow_test.go:318: needle "Plate Count" reaches the screen from 2 flows:
          buildMultisigPolicyFlow (gui/multisig_build.go)
          composerFlow (gui/composer_flow.go)
        A string one shared helper emits appears on every caller's screen, so a
        walk anchoring on it cannot prove which flow it is in (F-190).
--- FAIL: TestBuildFlowNeedlesHaveExactlyOneProductionSite
    needle_test.go:177: needle "Plate Count" has 2 production site(s), want exactly 1:
          gui/composer_flow.go
          gui/multisig_build.go
        a walk anchoring on this cannot prove which flow it is in
```

`gui/composer_flow.go:291` (the plan's B11 `Replace` fence, verbatim) calls
`confirmReviewScreen(ctx, th, "Plate Count", …)`; `gui/multisig_build.go:495`
already does. `"Plate Count"` is a **registered walk anchor** —
`cmd/emu/needle_test.go:108` holds `{"Plate Count", "gui/multisig_build.go"}` —
and the composer's census screen takes its uniqueness away.

**Why no earlier gate caught it.** These tests live in `cmd/emu`, not `gui`.
`scripts/gui-shard-test.sh ./gui/ 24` reported `ok — all 1174 tests` on this
exact tree, and every per-task `-run '^TestComposer'` run was green. Only the
plan's own `go test ./...` reaches them. **The plan's Step 2 note is right that
running both matters — but for a wider reason than it gives: this is not a
shared-state defect, it is a package the shard runner never enters.**

**I did not fix it, and the brief's rule is why:** *"NO pre-existing test may be
edited except the ones the plan names as exact old→new replacements; if another
fails, stop and record it."* The plan names neither test. It also is not a
mechanical call — `cmd/emu/needle_flow_test.go:8-21` records this exact
situation as prior art and rules against the obvious remedy:

> F-190 … F-188 hit it directly: reusing the build path's plate-census title on
> the supply path made the build walk's anchor two-site, and the implementer had
> to differ an OPERATOR-FACING TITLE ("Plates To Cut" vs "Plate Count") purely
> to keep a test honest. **That is the tail wagging the dog.**

So the three candidate resolutions are all somebody else's call:
1. **Give the composer's census screen a different title.** Cheapest, and
   exactly the move the fork's own comment calls the tail wagging the dog. It is
   also operator-facing copy, which SPEC §8 governs — a spec change, not an
   implementation one.
2. **Retire or re-anchor the `"Plate Count"` needle**, choosing a Build-flow
   anchor that is still single-site. Edits a shipped test the plan does not
   name.
3. **Promote it to a decoy** (`{"Engrave Bundle", 0}`'s shape — "at least one,
   count not pinned"), with the same caveat.

**Nothing else in the tree fails**: 53 `ok` package lines, and these two are the
only `--- FAIL` in the whole `./...` run.

### 3.11 (Plan measurement error, benign) `shots_walletpolicy.js`'s baseline count is 2, not 1

C2 Step 5 says the `tap(CONFIRM)` counts at `321acb56` are *"`shots_seating.js`
2, `shots_walletpolicy.js` 1, `shots_tr_pathological.js` 2"*. Measured from the
commit itself, all three are **2**:

```
$ for f in shots_seating.js shots_walletpolicy.js shots_tr_pathological.js; do
    git show 321acb56:cmd/emu/$f | grep -c 'await tap(CONFIRM);'; done
2
2
2
```

The Expected is a **rule** — "one higher than at `321acb56`" — and all three are
now **3**, satisfying it. Only the parenthetical baseline was wrong.

### 3.12 (Gate not run) The firmware size delta — see §6.4

---

## 4. The Part-A milestone (Task A11 Steps 6–7) — PROVEN, twice

**(a) The plan's own `GATE_UNTIL` extraction.** Ran against untouched fork
`main`:

```
$ GATE_UNTIL='^### Task B1' … scripts/plan-build-gate-go.sh …
   GATE_UNTIL=^### Task B1
   extraction stops at line 5861: ### Task B1: `gui/composer_sources.go` …
   [23 files]
```

**23 files extracted — the plan's own measured count** — and none of
`composer_sources.go`, `composer_seat.go`, `composer_review.go`,
`composer_selfcheck.go`, `composer_engrave.go`, `composer_cards.go`,
`composer_census.go` or `composer_join_test.go` is among them.
`composer_discard.go` **is**, as the plan says it must be.

*The plan's second command could not run as written:* it invokes
`python3 scratchpad/handwire_s3.py --part-a …`, and no such file exists in
mnemonic-engrave — it lived in a session scratchpad, which is wiped on process
exit. So I proved the same claim on better evidence:

**(b) The real tree at commit `dbc40a9`**, which *is* Part A complete with Part
B absent:

```
Part B files present in the worktree?   absent × 8   (all eight checked by name)
go vet ./gui/                            only the two pre-existing ArtifactDir findings
go test -run '^TestComposer' ./gui/      ok — 51 top-level PASS
whole gui package, 24 shards             ok — all 1109 tests ran
```

The plan measured 47 `TestComposer*` at plan time; 51 here, the difference being
A10's four, which the plan wrote as blocked. **Part A ships alone.**

---

## 5. Task C0 Step 5 — every guard's named mutation applied

The plan's own step, run in full: 14 rows, each mutation applied to the real
source, that test alone run, then `git checkout --` and `git diff --quiet`
verified. C0 was committed **before** this step so a checkout could never
discard unstaged work.

**13 of 14 guards failed their own named mutation. Every revert was clean.**

| # | guard | mutation | result |
| --- | --- | --- | --- |
| 1 | `…LockAndHashEditsAreNotGuardedByTheDiscardConfirm` | guard the Time-lock arm | FAIL ✓ (sub-test `time_lock`) |
| 2 | `…InvariantIgnoresSeveralUnseatedSlots` | remove the `src < 0` skip | FAIL ✓ |
| 3 | `…BackInTheKeyEditorKeepsTheExistingKeySet` | drop the restore | **SURVIVED — expected, see below** |
| 4 | `…ChangeTheScriptRowRewrapsAndDiscards` | delete the row | FAIL ✓ |
| 5 | `…ConsentRestatesTheHashRule` | delete the §8i block | FAIL ✓ |
| 6 | `…HexEntryItselfRefusesAnythingButSixtyFourCharacters` | `valid := len(frag) >= 63` | FAIL ✓ (sub-test `sixty-three_hex_characters`) |
| 7 | `…LockEditTellsAnImpossibleDateFromThePastCeilingDate` | restore the `u == 0` disjunct | FAIL ✓ (`an_impossible_date_inside_the_band`, `a_real_date_below_the_floor`) |
| 8 | `…FlowReShowsTheStubScreenOnlyAfterARealEdit` | `changed := false && …` | FAIL ✓ (`back_out,_change_a_key_count,_Done_again`) |
| 9 | `…ShortfallCountsSeatsNotSourcesOnAFixtureThatCanTellThemApart` | pass `len(st.sources)` | FAIL ✓ |
| 10 | `…MintCardsMintsOneCardPerSeatedSlot` | duplicate from the 2nd seated slot | FAIL ✓ |
| 11 | `…Section8mRefusalsAllDrawThroughTheRealPath` | remove the slot-cap `showError` arm | FAIL ✓ |
| 12 | `…DoorSaysAPayloadIsInFlashButNotLoaded` | drop the not-loaded branch | FAIL ✓ |
| 13 | `…ConsentFlowNumbersPathsFromTheOperatorsList` | pass `nil, 0` | FAIL ✓ |
| 14 | `…DateCeilingAndImpossibleDateAreToldApart` | `composerDateExists` stops distinguishing | FAIL ✓ (`20270231`, `20271301`) |

For rows 6, 7 and 8 the failing **sub-test names match the plan's pasted output
exactly** — the three round-2 replacements do fail the mutations they were
written for.

**Row 3's survival is not a new finding.** The plan's R0 round 2 table already
records it: *"journey I-5's named mutation is a structural no-op … recorded, not
re-fixed: `composerKeysEdit`'s decline paths never write `Keys` before its
single success-path assignment, so nothing needs restoring in the current call
graph … the test is real and drives the UI through Back, it is just not
effective for that one mutation."* Confirmed, unchanged.

**Row 14 needed a substitution, stated.** The literal mutation ("restore the
`u == 0` disjunct") inside `composerDateExists` makes it call
`composerDateToUnix`, which calls it back — `fatal error: stack overflow`, a
compile-shaped failure rather than an assertion. The equivalent that removes the
same distinction without recursion (`composerDateExists` returns `true`) fails
the guard on the two impossible-date rows, which is the property the guard pins.

---

## 6. Task C2 — the gates as CI runs them

### 6.1 Composer tests, and §13 item 1's four numbers

```
top-level PASS: 116     (plan time 112; +4 = A10's, which the plan wrote as blocked)
top-level FAIL: 0
sub-test PASS:  112     (plan time 100; +12 = A10's)
ok  	seedhammer.com/gui	6.762s
```

Captured once to a file and grepped, per the standing rule.

```
SPEC13 stub_screen    lines= 42 per_frame= 7 pages=6
SPEC13 pick_list      lines= 36 per_frame= 7 pages=6
SPEC13 consent        lines= 17 per_frame= 7 pages=3
SPEC13 descriptor_plate ceiling_chars=596  c10_688_fits=false
```

Folded into SPEC §13 item 1 verbatim by commit `93f009f`, together with §6a
(flag screens are load-time), §7a (the key state is stated WITH Build, because
the Lead wraps and the rows do not) and §7f (two plate forms, F-455 owns the
split) — the four items the plan lists. Gates on the folded spec, with
`CITE_FORK_ROOT` pointed at the worktree under review:

```
plan-cite-check.sh      76 citations, all ok, 0 dangling (exit 0)
plan-glyph-check.sh     operator strings scanned: 106 ; undrawable: 0
plan-table-check.sh     table rows checked: 137 ; malformed: 0
spec-structure-check.sh sections: 56 ; cross-refs checked: 49 ; STRUCTURE OK
```

### 6.2 The whole package, both ways

```
scripts/gui-shard-test.sh ./gui/ 24   RESULT: ok -- all 1174 tests ran across 24 shards   (55s)
CGO_ENABLED=0 go test -timeout 20m ./...    exit=1 -- 53 ok, cmd/emu FAILS (see 3.10)
```

The plan expects `1170` sharded at plan time; **1174** here, +4 for A10.

### 6.3 32-bit, oraclelive, js vet, gofmt — all clean

```
GOARCH=386 test:  exit 0
GOARCH=arm build: exit 0
oraclelive:       ok oracle / gui / sysw
GOOS=js GOARCH=wasm go vet ./cmd/emu/    (no output)
gofmt -l gui/ md/ mk/ sysw/ scripts/     gui/transaction.go
                                         gui/transaction_golden_test.go
                                         gui/transaction_txrecord_test.go
```

**Exactly the three files the plan says fork `main` already lists**, and I
verified them at untouched `321acb56` before touching anything — a stray-blank-line
collapse in each, none of them this stage's.

### 6.4 ⚠️ **Task C2 Step 4 (the firmware and its size delta) COULD NOT BE RUN**

`nix run .#build-firmware` and the `tinygo build -size short …` line both need a
toolchain that is **not on this machine**: `/nix` does not exist, and neither
`nix` nor `tinygo` is on `PATH` or in any package (see §0.1). There is no
workaround — TinyGo is not a Go-module dependency.

**So the flash/RAM delta against the plan's baseline (1,503,652 B flash /
62,592 B RAM at fork `169073c`) is UNMEASURED, and the plan's own check on it is
undischarged.** That check is not cosmetic: the plan states *"If the delta is
zero, the door wiring did not land and nothing composed is reachable — which is
the same class of defect as a gate that never ran."*

**Partial substitute, offered as such and not as a pass:** the door wiring is
independently proven reachable from `walletPolicyFlow` by
`TestComposerEveryScreenFunctionHasAProductionCaller`, by the DEAD-IN-PROD scan
in §6.5, and by the five shipped walks that failed *because* the door now
intercepts them. That establishes reachability; it does not establish the size
number, and **this step must be re-run on a machine with the nix toolchain
before anything is flashed.**

### 6.5 The reachability gate

The plan's Step 5 command aborts before its own step 8: the gate copies fork
`main`, whose `md/compose_vectors_pin_test.go` predates A10's vendoring, so the
copy fails `pin says 32 vectors, this test knows 26` — the gate testing an old
tree with new vectors, not a defect here. The same scan run against the shipped
tree:

```
   gui: 20 new production file(s), 1 function(s) with no production caller
      DEAD-IN-PROD gui/composer_census.go: composerDescriptorCeilingChars
```

**Exactly the one survivor the plan names and justifies** (§13 item 1's
measurement, production consumer deferred to F-457). No other `gui` survivor.
The `md`/`mk` names the same scan prints (`Compose`, `ComposeWith`,
`ComposerStubs`, `AppendStubs`) are S2's package APIs consumed from `gui`; the
scan compares within a package.

### 6.6 The emulator walks

All three edited exactly as the plan specifies (one extra `tap(CONFIRM)` plus
the §7a comment), `GOOS=js GOARCH=wasm go build ./cmd/emu/` succeeds, and each
file's count is 3 — one higher than the 2 each carried at `321acb56` (see 3.11).
**They were not run**: they need a browser and playwright, which no gate in this
stage has. Proving them is S4's journey run, and
`design/journeys/capture_walletpolicy.py` in mnemonic-engrave rides on the third.

---

## 7. What a reviewer should look at first

1. **§3.10, the `"Plate Count"` needle collision.** The only blocking item, it
   fails CI's own command, and its resolution is a spec/design call the fork's
   own `needle_flow_test.go` has an opinion about.
2. **§6.4, the unmeasured firmware delta.** A named gate that has never
   executed — the class the project's own rule says a plan may not close over.
3. **§3.5, A10's four downstream wiring points.** The preset screen is now
   between the wrapper and the path list in production; four walks decline it to
   reach the blank route. Whether "Back = blank" is the right operator gesture
   for that screen (versus an explicit "Blank" row) is a design question the
   plan settles only by implication.
4. **§3.6, B1/B2's import split**, and §3.4's three forced commit reorderings —
   the final tree matches the plan; the commit boundaries do not.
5. The four questions the plan's own self-review nominates for reviewer budget:
   whether `composerSlotOrder` tracks §5's numbering for a taproot list whose
   first single-key path is not first-listed; whether the self-check's
   comparison set is the one §7e names; whether §8p's "no cause is guessed" rule
   survived its implementation; and whether Part A is genuinely shippable
   without Part B (§4 says yes, measured twice).
