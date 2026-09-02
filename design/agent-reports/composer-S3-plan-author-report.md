# Author report — `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`

Single-author pass on the Wallet Policy composer's Stage 3 (fork GUI) implementation plan, written 2026-09-02 against fork `169073c` with the unmerged S2 worktree (`wt-composer-s2`, `489d52e`) used to run the build gate. Read-only against all code; one new file written (`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`) plus this report. Nothing staged, nothing committed. No `.jsonl` file was read.

## 1. The file structure chosen, and why

**Two PARTS in one plan, 23 tasks, 7,626 lines.**

- **Part A (11 tasks)** ends at spec §12 item 3: a device with no payload composes a shape, reads the stub screen, consents, and engraves a keyless template whose md1 decodes on the device. It is **independently shippable**, which was the structural decision the rest followed from. C26's keyless template is a whole journey on its own, and a stage that only lands when seating also lands has no intermediate state anyone can flash.
- **Part B (10 tasks)** adds seating, the mapping review, the self-check, the engrave forms, minting and the census. It is explicitly **not** shippable alone.
- **Part C (2 tasks)** runs the §13 item 1 measurements and folds them into the spec, then the CI gates and the firmware size delta.

Ordering inside Part A is forced by compilation rather than chosen: the copy file first (39 §8 bodies with an AST-scanned coverage table), then the paged primitive, then the screens that use both.

**Three conventions carry the whole plan:**

1. **Every new file is `gui/composer_*.go` and every new test function is named `TestComposer…`.** `scripts/plan-build-gate-go.sh` extracts blocks anchored on `gui/composer_*.go` and runs `go test -run '^TestComposer' ./gui/`. A test named otherwise is compiled and never run — the gate would report a pass on tests it never executed. This is stated as a Global Constraint, not left implicit.
2. **`gui/composer_copy.go` holds every §8 body**, and `composer_copy_test.go` AST-scans it, failing when a `composerCopy*` function is declared without a row in the coverage table. §12 item 5 demands four gates per body; "every" is only checkable if the bodies are enumerable.
3. **Fragments of shipped files are given as exact old→new replacements**, six of them, and the plan names them in the gate-coverage line as the gate's blind spot.

## 2. The nine recon risks, and where each is resolved

A table in the plan (`## The nine things the recon found that the spec assumed otherwise`) carries all nine with the task that resolves each. Summary:

| # | resolution |
| --- | --- |
| 1. three secret forms vs two plate paths | The engrave-forms task offers the **two** the device has (`engraveSeed`'s words+SeedQR plate, `backup.SeedString`'s ms1 plate) with honest labels; a third plate design is a new layout, not wiring, and is filed as **F-455**. The spec-fold task rewrites §7f. |
| 2. `ChoiceScreen` does not scroll | `gui/composer_paged.go` is the **first task of Part A**, not polish. It copies `confirmReviewScreen`'s measuring loop and adds a cursor; `composerPageLines` is the one measure site every capacity number comes from. |
| 3. flag-screen "wiring" is a no-op | The admission task ships the row change plus three tests: F1 fires at load for a composer payload; `syswLoadWarnings` gains **no** per-program caller (with a control proving the query works); §6a is folded in Part C. |
| 4. `multisig_build.go:804` is a call site | The consent task cites `:1877-1939` for the paging and `:1926-1931` for the icon gate, separately. |
| 5. `policySummaryLines` def vs call conflated | The stub task cites def `:142`, call `:86`, `templateConsentLines` def `:63`, and relabels **both** `Template-ID:` occurrences (`:70` and `:79`). |
| 6. `buildMultisigSeedHook` is not the scrub | `composerFlow` installs `defer st.reg.scrub()` at flow entry (the `multisig_build.go:290-291` construction); `composerSeedHook` exists for observation and its comment says it zeroes nothing. A test pins the scrub through the registry. |
| 7. `gui/sim*.go` does not exist | Every acceptance in this plan is a Go `_test.go` on the shipped `synctest`/`runUI`/`pumpUntil`/`click` harness. The emulator journey is named as S4's and this plan does not claim it. |
| 8. plate ceiling is a search | `composerDescriptorCeilingChars` binary-searches `backup.EngraveText`+`toPlate`, copying `qrCeilingBytes`' shape; the refusal names the measured ceiling and a remedy. |
| 9. paged capacities unmeasurable statically | One measurement task prints four `SPEC13` lines and folds them into §13. |

## 3. What the build gate proved, and the five defects it caught

`FORK_REPO=/scratch/code/shibboleth/wt-composer-s2 ./scripts/plan-build-gate-go.sh <plan>` extracted every ```go block into a scratch fork carrying S2's `md`/`mk`/`sysw`. `gofmt` clean on all of them; `go vet ./gui/` clean. The six fragments were then hand-wired and the five broken shipped walks given their door step, after which:

```
scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1125 tests ran across 24 shards
```

**Every `TestComposer*` in the plan has passed against real code once already.** Doc gates: 222/222 citations resolved, 0 dangling, 0 ambiguous; 174 operator strings scanned, 0 undrawable; 28 table rows, 0 malformed; 0 step numbers in prose.

Five defects the gate caught that reading would not have:

1. `composerUnitsToDays` rounded **up**, so an operator who typed 90 days was echoed "91 days = 15188 units". Now floors.
2. Two Unix epochs were wrong: `1756684800` is 2025-09-01 (wanted 2026-09-01 = `1788220800`); `1804032000` is 2027-03-03 (wanted 2027-03-01 = `1803859200`). Both now measured, with the command in the comment.
3. A paging assertion pumped for a needle a marker rename had removed — the test failed while the code was right.
4. The scrub test asserted `Mnemonic[0] != 0` on the "abandon" vector, whose first eleven words **are** index 0: it could not distinguish scrubbed from never-written. Now asserts on word 11 ("about", index 3).
5. `TestComposerAdmitCommentNoLongerClaimsNoSeedClass` scanned the whole file for `"NO seed class"`, which `progTransaction`'s own untouched comment also says — **it would have failed after a perfectly correct fold**. Now asserts on a sentence unique to the Wallet Policy row, with a control.

**The biggest single finding is not in that list: the door breaks FIVE shipped tests**, not the zero the recon implies and not the one or two a reader would guess. Three of the five (`TestF440BundleIncompleteModalDismissesOnBack`, `TestF437CardDoorsDoNotPromiseTyping`, `TestF76WalletPolicyCountsACompleteMd1CardFromThePayload`) live in files no reviewer of `wallet_policy.go` would open, and a targeted `-run` filter shows none of them — it took the sharded whole-package run. The plan carries all five in a table with each one's route past the door, verified failing-then-passing, and says plainly that they are updated and never deleted or skipped.

## 4. Three recon facts corrected, and one gap the recon missed

- **`mk.Card.Xpub` is a `string`** (base58, `mk/mk.go:138`), not `[65]byte`. `[65]byte` is `md.ExpandedKey.Xpub` (`md/expand.go:93`). The plan makes `composerSource.xpub` a string everywhere and converts with `decodeXpubBytes` only where `md.Composed.Bind` needs bytes.
- **`slotMatchesCard` is at `gui/key_card_seating.go:128`**, not `:119` (the doc comment starts at `:118`).
- **The recon does not mention `TestEverySyswConsumptionSiteNamesAnAdmittedClass`** (`gui/sysw_admit_oracle_test.go:90`), which fails the moment the composer takes a record. Its scanner matches `take` by **exact selector name**, so `takeAll` and `cardSet` are invisible to it and **three shipped sites are unchecked today** (`gui/multisig_build_payload.go:75`, `gui/transaction.go:408,451`). The plan widens the matcher, registers those three plus the composer's two, and adds the five missing classes to `classNames`. Measured after the widening: **15 consumption sites reconciled**, suite green.

## 5. Open questions the OPERATOR must decide

Three, deliberately few. Everything else was resolvable from the spec, the rulings or measurement.

1. **Does Part A ship on its own?** The plan is built so it can (its exit is §12 item 3, a keyless template that decodes), and shipping it would put the door, the shape flow, the digit pad and the stub screen on the machine before seating exists. That is a release decision, not a technical one. If the answer is no, nothing in the plan changes — the parts just merge as one.

2. **§7f's three secret forms are two on this device (F-455).** `engraveSeed` bakes words **and** a SeedQR onto one `backup.Seed` plate; there is no words-only or QR-only plate for a mnemonic anywhere in the tree. The plan offers the two real forms with honest labels and files the split. **The alternative is to rule the split IN**, which means a new backup layout with its own sizing and its own goldens — out of this stage's scope as written, and worth saying so out loud rather than letting §7f's wording stand as though it described the device.

3. **F-453 blocks the presets, and only the presets.** `md compose --preset` does not exist at `66bdf2f4` and no preset vector is exported. The plan refuses to author preset shapes in Go without a Rust oracle (the Rust-primary rule) and blocks that one task. If the operator would rather S3 ship presets, F-453's Rust half has to land in descriptor-mnemonic first; if not, Part A ships blank-shape only and the presets follow. **No third option is offered on purpose** — authoring five normative path lists in Go with nothing to check them against is the drift the rule exists to prevent.

One thing that is **not** an operator question but is worth the reviewer's eye: **§8l's body is not the shipped Multisig Build string.** §8l calls it "Multisig Build's warning, reused", and the plan reuses the *surface* (an unskippable `ConfirmWarningScreen`) while using §8's own text, because §8 is the normative copy and the shipped body (`gui/multisig_build.go:872-879`) is a different, longer sentence. The shipped body is not edited. If a reviewer reads §8l as requiring byte-identity with the shipped string, that is the one place this plan takes a reading rather than following an instruction.

## 6. Size

| measure | value |
| --- | --- |
| lines | 7,626 |
| tasks | 23 (Part A 11, Part B 10, Part C 2) |
| new Go files specified | 20 (`gui/composer_*.go` and their tests) |
| fragments of shipped files | 6, each an exact old-to-new replacement |
| §8 bodies transcribed | 39, all in one file with an AST-scanned coverage table |
| citations | 222, all resolved against fork `169073c` |
| tests green at plan time | 1,125 across 24 shards, with the fragments hand-wired |

## 7. One signal passed on rather than fixed

In the gate's scratch copy, `sysw`'s `TestComposerRecordsClassifyExactlyAsTheHost` and `TestComposerRecordParsersReturnTheHostsValues` **FAIL**: the record-class fixture on mnemonic-engrave master now hashes `5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312` while the S2 branch pins `eed6b177d1a3406a69c4a0102635f5d59c6412fa65e106f85b831c4736ac464e`. That is **S2's pin against a fixture that has moved under it**, not a Stage 3 concern, and it belongs in S2's own "what did the S1 merge falsify here?" re-validation before its implementer is dispatched. Flagging it rather than touching it: this pass is read-only outside its one plan file.

## 8. Hand-wire script

`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/702b37c9-e041-404f-8220-2456ff9c6bf3/scratchpad/handwire_s3.py`

Python 3, one argument: the root of a scratch copy of the fork that already carries the S2 branch's `md`/`mk`/`sysw` and the plan's extracted `gui/composer_*.go`. It applies **the six fragments of shipped files the plan specifies**, plus **the five shipped-walk door steps** the same wiring step names:

| file | what it applies |
| --- | --- |
| `gui/sysw_admit.go` | `progWalletPolicy` admits the composer's eight classes; the "NO seed class" comment retired (C12) |
| `gui/gui.go` | the `walletPolicy` program comment: two doors, and the seed/census sentence retired rather than moved |
| `gui/multisig_build.go` | C7's §8e deprecation comment, comment-only, inserted above the T6c banner |
| `gui/sysw_admit_oracle_test.go` | BOTH edits: `classNames` gains the composer's three classes (and the two transaction classes the widened matcher now reaches), and the AST matcher grows `takeAll`/`cardSet` with the four sites that then need registering |
| `gui/wallet_policy.go` | the door runs first and dispatches (two replacements: the head, and the closing brace) |
| `gui/template_engrave.go` | both `Template-ID:` occurrences relabelled `mk1 stub (template):` |
| `gui/wallet_policy_descriptor_walk_test.go`, `gui/modal_back_test.go`, `gui/payload_door_walk_test.go`, `gui/payload_door_label_test.go` | the five shipped walks gain their door step; the label table's is guarded to its two `wallet policy` rows |

**Properties.** Every edit asserts its anchor occurs exactly once (twice for the one the plan says appears twice) and a mismatch is loud and non-zero; each edited file is printed; `gofmt -w` runs on the edited files (PATH, else the nix-store toolchain) so the result is formatted as well as compilable; and a tree that is **already wired is refused with exit 3 before anything is touched** — necessary because the deprecation edit anchors on a banner it inserts *above*, so an unguarded re-run would duplicate that comment while every other anchor failed.

**Tested** on a fresh `cp -r` of the gate's own extraction (`SCRATCH=/scratch/code/shibboleth/.plan-build-gate-go-s3`), then removed. `go vet ./gui/` reports **only the two pre-existing `testing.ArtifactDir requires go1.26` findings**; `go test -count=1 -run '^TestComposer' ./gui/` is **`ok`**; `scripts/gui-shard-test.sh ./gui/ 24` is **`ok -- all 1125 tests ran across 24 shards`**; a second run exits 3 without touching the tree.

**One plan defect it forced into the open, and the plan was edited to fix it.** The §8e deprecation comment as written wrapped `Wallet Policy > Build a new policy` across two lines, so two of the three substrings `TestComposerMultisigBuildCarriesTheDeprecationComment` requires were not contiguous — **the plan's own copy failed the plan's own test**, twice over, on line breaks alone. My earlier 1,125-green run had hidden it, because I had reflowed the comment in the scratch copy by hand rather than in the plan; making the script apply *exactly what the plan says* is what surfaced it. The fix puts §8e's sentence on its own unwrapped line with a comment saying why it must stay that way, in the plan and in the script identically (one hunk, +13/-4 against `88b4a4a`). That is the only repository edit this task made, and it was unavoidable: a script that applied the plan verbatim and a plan whose text failed its own gate cannot both be left standing.


## 9. R0 round 0 folded, and the fold applied to the plan

Three lenses read the committed plan on 2026-09-02 -- **journey 2C/6I/6M/4N**,
**fidelity 1C/12I/11M/3N**, **tests 15C/1I** -- and all three found the same
Critical independently: **Part B was built and never joined to any flow.**
Fourteen production functions had no caller, so an operator with four `key:`
records read the door's "Keys loaded: 4", chose Build, and was never offered a
slot; §7e's self-check and §8q never executed on a device. Go does not error on
an unused package-scope function and every `TestComposer*` called them directly,
so the suite was green over a feature nobody could reach. That is the class
recorded in memory as *"plans list components and omit the call that joins them;
six green stages shipped an inert feature"*, and I shipped it again.

The whole fold has been APPLIED to
`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` (working tree, uncommitted;
the only repository file touched). The plan grew **7,626 -> 10,893 lines** and
**23 -> 25 tasks**: **Task B11** is the join, with a flow-level walk that is the
only kind of test that can fail on that class, and **Task C0** carries §12 item
5's missing gates plus one test per surviving mutation.

**Every `gui/composer_*.go` fence in the plan was replaced with code that was
compiled and run first** -- 41 fences. Nothing in the plan is code that has not
executed.

### Gate output, all re-run after the fold

| gate | result |
| --- | --- |
| `plan-build-gate-go.sh` (fork `main` `321acb56`) | 41 fences extracted, gofmt clean, `./md/ ./mk/ ./sysw/` ok |
| DEAD-IN-PROD reachability (the gate's last step, wired tree) | **gui: 1 survivor**, `composerDescriptorCeilingChars` -- §13 item 1's measurement, production consumer deferred to F-457; named and justified in the plan |
| `go vet ./gui/` (wired) | clean but for two pre-existing go1.25 `ArtifactDir` findings on fork `main` |
| `go test -count=1 -run '^TestComposer' ./gui/` (wired) | **100 top-level PASS, 81 sub-tests, 0 FAIL** |
| `gui-shard-test.sh ./gui/ 24` (wired) | **ok -- all 1158 tests ran across 24 shards** |
| `go test ./md/ ./mk/ ./sysw/` (wired) | ok x3 |
| `plan-cite-check.sh` | 238/238 resolved, 0 dangling, 0 ambiguous |
| `plan-glyph-check.sh` | 237 operator strings, 0 undrawable |
| `plan-table-check.sh` | 89 rows, 0 malformed |
| `plan-stepref-check.sh` | 0 step numbers in prose |
| `plan-staleness-check.sh <plan> <fork> 321acb56` | 142 unchanged, **0 drifted** |

**One false red worth recording:** the first sharded run failed with `no space
left on device` on six payload-writing tests. `/tmp` is a 32 GB tmpfs at 81%,
held mostly by other agents' Rust build caches in the shared session scratchpad.
Green with `TMPDIR` on `/scratch`; nothing was deleted from another agent's tree.

### Where the fold declined a fix

Three, each with its citation:

1. **Journey C-2's second shape** (restate the reversed key-order outcome on the
   path-list row and consent line) -- declined for the first shape. Asking the
   question at the transition, where `sole` is final, REMOVES the reversal;
   restating it describes the reversal after the fact, and §8b then fires exactly
   where §5a permits.
2. **Fidelity I-10's form A** (concrete descriptor as text and QR plates) --
   declined and filed **F-457**. `md` deliberately emits no descriptor text
   (`md/compose.go`'s header: *"a rendering that cannot be re-parsed is the
   defect this package's invariant exists to prevent"*), so a renderer is
   normative and lands in Rust first. Form A ships as the keyed md1, which §7f
   also names. `composerCensusRefusal` went with it rather than staying as a
   refusal nothing could trigger (tests-lens C-15).
3. **Fidelity N-1** (§8r's lines are "beneath Build" in the spec, in the Lead in
   the code) -- declined as a code change and recorded for the §13 spec fold:
   the Lead wraps with `widget.Labelw` (`gui/gui.go:1969`) and the choice rows do
   not, which the reviewer calls sound.

### The per-finding table, as folded into the plan

## R0 round 0: what the three lenses found, and where each is folded

Three lenses read the committed plan on 2026-09-02 and persisted verbatim to `design/agent-reports/composer-S3-plan-R0-r0-{journey,fidelity,tests}.md`: **journey 2C/6I/6M/4N, fidelity 1C/12I/11M/3N, tests 15C/1I**. All three found the same Critical independently. Every finding below is folded into the task named, or declined in one sentence with its citation. Nothing is carried as "noted".

**The one Critical all three found.** Part B declared fourteen production functions and nothing called any of them; the plan promised a ``Replace `gui/composer_flow.go` `` that no task supplied. **Task B11 is that task**, with a structural reachability scan and a flow-level walk -- the only kind of test that can fail on the class. Recorded in memory as *"plans list components and omit the call that joins them; six green stages shipped an inert feature"*, and it happened here anyway.

### Journey lens (2C / 6I / 6M / 4N)

| finding | folded into |
| --- | --- |
| C-1 Part B never joined to any flow | **Task B11**, with `TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen` and `TestComposerEveryScreenFunctionHasAProductionCaller` |
| C-2 "Sorted (usual)" silently reversed by adding a second path | **Task A5**: the question moves OUT of `composerKeysEdit` into `composerKeyOrderStep`, asked at the transition where `sole` is FINAL. The reviewer's second shape (restate the outcome on a row) is declined: it describes the reversal after the fact where asking once removes it, and §8b then fires exactly where §5a permits |
| I-1 §8j blocks the lock and hash edits §7g rules DEFAULT | **Task A5**: `composerShapeGuard` moves onto `composerPathEdit`'s Keys, Remove and Move arms; a lock or hash edit is unguarded, per §7d |
| I-2 the §4f invariant fires on unseated slots | **Task B4** (`composerInvariantViolation` skips `src < 0`) and **Task B6** (the real invariant runs over the DECODED md1's declared origins, which is where §4f puts it); **Task B11** sizes `st.assigned` at flow entry so a never-seated composition no longer fails the self-check on slot count |
| I-3 the pick list can take a row the operator cannot see | **Task A2**: `composerPickScreen` clamps `sel` into `[start, start+shown)` after a page advance, both directions; gated by `TestComposerPickScreenNeverReturnsARowItDidNotDraw` (Task C0) |
| I-4 consent confirmable before its proof is drawn | **Task A2**: `composerReadScreen` withholds the checkmark until the last page has been laid out ONCE; **Task B6** makes it the single consent surface, closing fidelity M-5 with it |
| I-5 Back at Key order destroys the path's key set | **Task A5**: the key set is snapshotted and RESTORED on any decline, and `composerPathLine` gives a path with neither element its own body ("empty") instead of "hash only" |
| I-6 a date past 2038-01-19 refused as "does not exist" | **Task A1** adds `composerCopyDateCeiling` (the copy table moves 39 -> 40) and **Task A7** uses it; filed as a §8 addition, F-456, so the spec stays the enumerable source |
| M-1 the changed-id line fires after a Back with no edit | **Task B11**: the §8s signal is a comparison of the emitted CHUNK SETS, not a sticky `edited` flag |
| M-2 an empty digit field echoes the §8u ceiling | **Task A7**: the two relative pads say what to type before what is too much |
| M-3 `ErrComposeNoPaths` shows a codec string | **Task A5**: mapped to §8m line 1 beside `ErrComposeNoKeyedPath` |
| M-4 a key-less path with "No hash lock" leaves an empty path | **Task A5**: a path that ends with neither keys nor a hash is treated as a cancel |
| M-5 the own-wallet line absent from the consent surface | **Task A11**: `composerConsentLines` carries it, where §7g's divergence table puts it |
| M-6 the seating prompt is drawn on one page only | **Task A2**: the lead is a PER-PAGE header rather than the first body row |
| N-1 Back at the wrapper exits the program | **Task B11**: the door is a loop, so Back out of Build lands on the door |
| N-2 the slots/keys line drawn with no payload | **Task A5**: `composerSlotsKeysLine` prints the slot count alone when no source is loaded |
| N-3 no way to reorder paths | **Task A5**: a "Move up" arm on paths after the first, through `composerApplyShapeEdit` so the discard stays exact |
| N-4 the date echo appends the raw operand | **Task A7**: removed; §6b's premise is that the operator never types one |

### Fidelity lens (1C / 12I / 11M / 3N)

| finding | folded into |
| --- | --- |
| C-1 Part B has no caller | **Task B11** (same as journey C-1) |
| I-1 the invariant on unseated slots, checked on UI state | **Tasks B4, B6** (see journey I-2) |
| I-2 consent mis-numbers paths for an extracted taproot internal key | **Task A11**: `composerConsentLinesFor` takes the operator's numbering and the key-path line names its listed path; gated by `TestComposerConsentNumbersPathsAsTheOperatorListedThem` |
| I-3 Back at the path list abandons the composition | **Task B11**: Back returns to the wrapper with the list intact, gated by `TestComposerBackAtThePathListKeepsTheComposition` -- the first test in this plan that fails if a Back loses state |
| I-4 the wrapper cannot be changed after the first pick | **Task A5**: a "Change the script" row, so §7g's row and §12 item 4's wrapper vector are reachable |
| I-5 `composerKeysEdit` destroys an existing key set | **Task A5** (see journey I-5) |
| I-6 §8a/§8b memoised by path index, so an unskippable confirm can be skipped | **Task A5**: both memos deleted. §8a fires where a key-less path is CREATED (once per path by construction) and §8b at the transition; gated by `TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex` |
| I-7 the pager-gate test cannot fail | **Task C0**: replaced by `TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage`, which asserts the BEHAVIOUR and fails in both directions; the ink comparison is gone |
| I-8 §12 item 5's gates missing for §8m, §8c, §8r | **Task C0**, with the measured reason the instrument could not be used unchanged for the short bodies |
| I-9 §8i's "and at consent" half absent | **Task A11**: `composerConsentLines` restates it whenever the decoded shape carries a digest |
| I-10 form A and the secret plate have no builder; "cut ONCE" unimplemented | **Task B7** (the secret-form picker is deleted, since nothing consumed it) and **Task B11** (`composerSecretCards` dedups by registered seed and cuts ms1 through `cardMS1`). Form A ships as the KEYED md1; the text and QR descriptor plates are declined here and filed **F-457**, because `md` deliberately emits no descriptor text (`md/compose.go`'s header: "a rendering that cannot be re-parsed is the defect this package's invariant exists to prevent") and a renderer is normative, Rust-first |
| I-11 Part A's exit not discharged; Part A alone breaks §7e/§7f | **Task B11**: `TestComposerNoPayloadWalkEngravesAKeylessTemplate` walks all six of §12 item 3's clauses, and the joined flow gives Part A the collapsed form choice and the self-check §7e makes unconditional. The default stands: Part A ships alone for the no-payload journey, and the plan says where that is honest |
| I-12 the blast radius covers Go tests only | **Task C2**: the three `cmd/emu/*.js` walks and `capture_walletpolicy.py`, with the one-line fix and an explicit note that no gate in this stage can run them |
| M-1 the 9th-path refusal has no §8 home | **Task A5**: the row is not offered at the cap, which is what §4e asks; the ad-hoc string is gone |
| M-2 the consent renders locks in the row form | **Task A11**: `composerBranchLines` uses §6b's echo form |
| M-3 the sticky `edited` flag | **Task B11** (journey M-1) |
| M-4 the raw operand in the date echo | **Task A7** (journey N-4) |
| M-5 two consent surfaces | **Task B6**: one, `composerReadScreen`, which is `confirmReviewScreen`'s paged form plus the last-page gate §7e needs |
| M-6 §8i shown in front of a clear | **Task A8**: shown once the operator is actually taking a hash |
| M-7 `composerCensusLines`' declared vs real signature | **Task B9**: the Interfaces line matches the code, and `composerCensusRefusal` is removed with its deferred consumer |
| M-8 `seedID` parameter is a source index | **Task B2**: renamed `srcIdx`, with the collision named |
| M-9 no preset-or-blank wiring point | **Task A10 and Task B11**: the point is named so A10 is a fill-in |
| M-10 stale STATUS baseline | the header: fork `main` `321acb56`, the five drifted citations re-resolved |
| M-11 `composerPageLines` counts a row that may overflow | **Task A2**: a row is counted only when it is inside the box, because that count IS §13 item 1's recorded capacity |
| N-1 §8r "beneath Build" vs the Lead | DECLINED as a code change and folded to the spec instead: the Lead wraps and the rows do not (`gui/gui.go:1969`), which is the reason the reviewer calls sound; recorded for the §13 fold in **Task C1** |
| N-2 the upward-only clamp | **Task A2** (journey I-3) |
| N-3 §8n's host lines not scoped | **Task A1**: the copy table's own header says §8n is `me sysw pack` stderr and belongs to S1 |

### Tests lens (15C / 1I) -- the mutation survivors

| finding | folded into |
| --- | --- |
| C-1 Part B unreachable | **Task B11** |
| C-2 `composer_engrave_test.go` and `composer_cards_test.go` promised and never written | **Tasks B7 and B8**, both files written with their steps |
| C-3 Task B10 is prose, no code | **Task B10**, rewritten with the five vectors and four assertions |
| C-4 the door's `ClassMDMK` branch is a false PASS | **Task C0**: `TestComposerDoorOffersFromPayloadForACardPayload`, on a real minted card |
| C-5 nothing clicks "Build a new policy" | **Task B11**: both walks do |
| C-6 the §4e gate inside `composerShapeFlow` can be deleted | **Task C0**: `TestComposerShapeRefusalGateIsReachedFromTheScreen` |
| C-7 the §8a confirm can be bypassed | **Task B11**: `TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex` drives the decline |
| C-8 `composerLockAccept`'s bounds can be disabled | **Task C0**: `TestComposerLockAcceptRefusesFromTheScreen`, four refusals and a control |
| C-9 the §8s re-show signal can be inverted | **Task C0**: `TestComposerStubReshowSignalIsTheChunkSet` |
| C-10 `composerStubLines` never called with `keyedChunks` | **Task C0**: `TestComposerStubLinesLabelASeatedSlot` |
| C-11 the mapping review's invariant check at the call site | **Task C0**: `TestComposerMappingReviewRefusesFromTheScreen` |
| C-12 `composerShortfall`'s counts at the call site | **Task C0**: `TestComposerShortfallCountsSeatsFromTheScreen` |
| C-13 the C29 warning deletable from the review's output | **Task C0**, same test's second half |
| C-14 §8l deletable from `composerFlow` | **Task B11**: both walks assert it draws and hold to confirm it |
| C-15 `composerCensusRefusal` unreachable | **Task B9**: removed with its deferred consumer (F-457), rather than left as a refusal nothing can trigger |
| I-1 `composerHexEntry`'s exact-64 bound untested | **Task C0**: `TestComposerHexEntryTakesExactlySixtyFourCharacters`, which shows that 62 characters decode fine and are still not a digest |


## 10. R0 round 1 folded -- the fold's own Critical, and eight open items closed

`design/agent-reports/composer-S3-plan-R0-r1-fold-verification.md` returned **NOT GREEN**. It
verified both round-0 Criticals as correctly fixed and holding under mutation, and then found a
**new Critical of the round-0 fold's own making**: Task A11's code fence had been overwritten with
Task B11's joined body, so "Part A ships alone" -- one of three named operator-question defaults --
was false of the artifact meant to deliver it. An implementer following the plan in its own order
could not have compiled the milestone it promises there, and the build gate could not see it: a
whole-document, `Replace`-wins extraction only ever builds a file's FINAL state.

That is my defect, and it is the same shape as the one round 0 caught me on -- a claim about
structure that no gate was asked to check. The answer this time is a command rather than a promise.

### Per item, with plan lines

| r1 item | fix | plan line |
| --- | --- | --- |
| §2 NEW CRITICAL: A11's fence overwritten | Task A11's `composerFlow` restored self-contained (`md.Compose` direct, ends at `composerEngraveTemplate`); Produces line and commit message corrected; the weak `TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes` DELETED and the real C26 walk moved into Part A's own test file, written to pass with or without Part B's seating step | 4852 |
| §2, same: prove it rather than assert it | a `GATE_UNTIL='^### Task B1'` step, plus `handwire_s3.py --part-a`; the gate's blind spot on incremental buildability is stated in the task | 5755 |
| §2, found BY that step | `gui/composer_discard.go` moved from Task B3 to Task A5 -- `composerShapeFlow` calls `composerApplyShapeEdit`, and Part A failed `go vet` three times without it | 2157 |
| §2, consequence | Task B3 restated as the rule's semantics and tests | 6567 |
| §3 fidelity I-2: fix never reached production | the parameterless `composerConsentLines` wrapper DELETED (it hardcoded the very argument the fix was about); `composerConsentFlow` passes `composerListedPaths(st.list)` | 11110 (guard) |
| §3 journey I-6: `u == 0` tautology | `composerDateExists` separates the three failures; F-458 cited | 11110 (guard) |
| §3 six unguarded Importants + C-9, C-12, 6b, 8d | twelve guards in Task C0, each naming its mutation; two are FIXTURE fixes, because C-12's and 8d's old fixtures were structurally incapable of failing | 11110 |
| §3 fidelity I-7 | the stale `longInk <= shortInk` test deleted from Task A2, with the reason in its place | -- |
| §4 B5/B6/B9 never reached their closing steps | each now has Run/Expected and a `-s -F -` commit step | -- |
| §4 five under-counted Expected lines | corrected to measured values, each saying why its filter nets more | -- |
| §5a extraction count | STATUS: 47 fences read, 1 dropped by Task B11's `Replace`, 46 kept across 43 files | 3 |
| §5c Task C2's stale headline counts | 110/95, and stated as **the count at plan time** rather than a threshold | 11317 |
| §5f two stale Produces lines | Task B9 (`composerCensusLines`' real signature) and Task B2 (`srcIdx`) | -- |
| §5h N-1's missing destination | Task C1 gains it as a fourth spec-fold item | 11274 |
| §6 C-15 | **MOOT, not closed**, and the plan now says so: `composerCensusRefusal` was removed with its deferred consumer (F-457), so there is nothing to wire | -- |

### Gate output, all re-run after the fold

| gate | result |
| --- | --- |
| **Part-A-only** (`GATE_UNTIL='^### Task B1'`, `handwire_s3.py --part-a`) | **23 files extracted, `go vet` clean but for the two pre-existing go1.25 findings, `ok seedhammer.com/gui`, 47 `TestComposer*` PASS -- with Part B absent** |
| `plan-build-gate-go.sh` (fork `main` `321acb56`) | 43 files written, gofmt clean, `./md/ ./mk/ ./sysw/` ok x3 |
| DEAD-IN-PROD (wired tree) | **gui: 1**, `composerDescriptorCeilingChars` (F-457's deferred consumer), named and justified |
| `go vet ./gui/` (wired) | clean but for the two pre-existing go1.25 `ArtifactDir` findings |
| `go test -count=1 -run '^TestComposer' -v ./gui/` (wired) | **110 top-level PASS, 95 sub-tests, 0 FAIL** |
| `gui-shard-test.sh ./gui/ 24` (wired) | **ok -- all 1168 tests ran across 24 shards** |
| `plan-cite-check.sh` | 241/241, 0 dangling, 0 ambiguous |
| `plan-glyph-check.sh` | 289 strings, 0 undrawable |
| `plan-table-check.sh` | 118 rows, 0 malformed |
| `plan-stepref-check.sh` | 0 step numbers in prose |
| `plan-staleness-check.sh <plan> <fork> 321acb56` | 144 unchanged, **0 drifted** |
| per-task structure | all 25 tasks carry >= 3 checkbox steps with at least one `Run:` and one `Expected:` |

### What round 1 changed about how I check my own work

The round-0 fold's Critical and round-1's were the same failure at two levels: a claim the gates were
not asked to check. Round 0's was "these functions are called by something"; round 1's was "this
milestone builds on its own". Both are now commands -- the gate's DEAD-IN-PROD step and its
`GATE_UNTIL` mode -- and the second one paid for itself immediately, catching
`gui/composer_discard.go`'s task assignment, which no reading would have found.

### The plan's own round-1 table, as folded

## R0 round 1: the fold verification, and where each item is folded

`design/agent-reports/composer-S3-plan-R0-r1-fold-verification.md` returned **NOT GREEN**. Both round-0 Criticals were verified fixed and hold under mutation; what follows is everything it found open, and where round 1 closes it.

| # | round-1 finding | folded into |
| --- | --- | --- |
| §2 | **NEW CRITICAL** -- Task A11's fence was overwritten with Task B11's joined body, so "Part A ships alone" was false of the plan's own artifact; the old weak walk test sat unretired beside its replacement | **Task A11**: the fence is restored self-contained (calls `md.Compose` directly, ends at `composerEngraveTemplate`), its Produces line and commit message match what it stages, the weak `TestComposerNoPayloadWalkReachesAKeylessTemplateThatDecodes` is DELETED, the real C26 walk moves into Part A's own test file, and a **`GATE_UNTIL='^### Task B1'` step proves the milestone builds with Part B absent**. The gate's blind spot is stated: a whole-document, Replace-wins extraction can only see a file's FINAL state |
| §3 fidelity I-2 | the path-numbering fix never reached production -- the only reachable call site hardcoded `(nil, 0)` | **Task A11**: the parameterless `composerConsentLines` wrapper is DELETED, and **Task B6**'s `composerConsentFlow` passes `composerListedPaths(st.list)`. Guard: `TestComposerConsentFlowNumbersPathsFromTheOperatorsList` (Task C0) |
| §3 journey I-6 | the ceiling dispatch's `u == 0` was a tautology, so "that date does not exist" was dead code and 2027-02-31 got the ceiling body | **Task A7**: `composerDateExists` tells the three failures apart. Filed **F-458**, distinct from F-456. Guard: `TestComposerDateCeilingAndImpossibleDateAreToldApart` |
| §3 six Importants | journey I-1, I-2, I-5; fidelity I-4, I-9; tests I-1 -- correct production fix, zero regression test | **Task C0**'s round-1 guards, one per finding, each naming its mutation |
| §3 fidelity I-7 | the fold's table claimed the `longInk <= shortInk` test was "gone"; it was still there | **Task A2** deletes it, with the reason in its place |
| §3 fidelity I-8 | §8m 1/5 and §8r 5/6 driven onto a frame | **Task C0**: `TestComposerSection8mRefusalsAllDrawThroughTheRealPath` (all five, the slot cap through `composerKeysEdit`) and `TestComposerDoorSaysAPayloadIsInFlashButNotLoaded` |
| §4 | Tasks B5, B6, B9 never reached their own closing Run/Expected/commit steps | each now has both, with measured counts |
| §4 | five Expected lines under-counted | corrected to measured values, with the reason each filter nets more than it did |
| §5a | the extraction count was 41; it is 47 read / 46 kept / 43 files | the STATUS line |
| §5c | Task C2's headline counts were 92/77 against an actual 100/81 | corrected, and stated as **the count at plan time** rather than a threshold, so the claim stays true by saying what it is |
| §5f | two `Interfaces > Produces` lines the round-0 table claimed fixed and were not | **Task B9** (`composerCensusLines`' real signature) and **Task B2** (`srcIdx`) |
| §5h | fidelity N-1's declined-and-recorded destination did not exist | **Task C1** gains it as a fourth spec-fold item (§7a: the key state is stated WITH Build, and the Lead wraps where rows do not) |
| §6 C-9 | the §8s re-show signal untested through `composerFlow` | **Task C0**: `TestComposerStubReshowSignalIsTheChunkSet` pins the comparison, and both flow walks drive the screen it feeds |
| §6 C-12 | the shortfall fixture could not tell seats from sources | **Task C0**: a SEED fixture, where the two rules give 4 and 1 |
| §6 cell 6b | the "Change the script" row had no behavioural test | **Task C0**: `TestComposerChangeTheScriptRowRewrapsAndDiscards` |
| §6 cell 8d | `composerMintCards` never called with both slots seated | **Task C0**: `TestComposerMintCardsMintsOneCardPerSeatedSlot` |
| §6 C-15 | **MOOT, not closed** -- `composerCensusRefusal` was removed outright with its deferred consumer (F-457), so there is nothing to wire up. It would need re-verifying if F-457's concrete-descriptor plate is ever built | recorded here, no code |
