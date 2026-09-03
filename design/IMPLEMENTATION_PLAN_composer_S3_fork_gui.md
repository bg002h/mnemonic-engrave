# Wallet Policy Composer — Stage 3 (fork GUI) Implementation Plan

**STATUS: IMPLEMENTED AND MERGED 2026-09-02 -- fork main `b77449db` (merge --no-ff of `composer-s3` at `27afa9f`, 29 commits; Test run 33709139231 green). Not flashed.** Implementation record: `composer-S3-implementation-report.md` (one implementer, 24 tasks; Part A proven alone; A10 unblocked by S0b at descriptor-mnemonic `1dc8d409`); controller fold `a63fd1e` (the census title is the supply paths' "Plates To Cut", not the Build walk's anchor); whole-diff review `composer-S3-exec-review-r0.md` 1C/2I/5M/3N; fold `composer-S3-fold-r0-report.md` (C-1 seating resumes and releases sources, I-1 Move up discards unconditionally, I-2 six fault rows, plus a second Critical the fold's own control found: the self-check read K/N outside their domain and refused 4 of 12 preset pairs); verification `composer-S3-exec-review-r1-fold-verification.md` 0C/0I. Gates on `27afa9f`: `go test ./...` 54 packages ok; gui 1185/1185 across 24 shards; 32-bit; oraclelive; js vet. Task C2's firmware step: **1,579,924 B flash / 62,800 B RAM, +73,040 / +208 over `321acb56` (1,506,884 / 62,592)**, measured by the implementer, the controller and the verifier. Toolchain note: the machine changed distro mid-stage and lost `/nix`; the fork's tests ran on Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`, and the firmware on the reinstalled flake (whose Go is 1.26.3), so the `/nix/store/...go-1.26.7` path cited below is historical. R0 GREEN was reached the same morning: round 3 (targeted fold verification) closed 0 Critical / 0 Important. Superseded status text follows for the record: DRAFT, build-gated, R0 ROUND 2 FOLDED 2026-09-02. Round 0 (journey 2C/6I/6M/4N, fidelity 1C/12I/11M/3N, tests 15C/1I) agreed on one Critical -- Part B built and never joined to any flow -- and **Task B11 is the join**. Round 1 found a **new Critical of the fold's own**: Task A11's fence had been overwritten with Part B's joined body, so "Part A ships alone" was false of the artifact; it is restored self-contained and **proven by a `GATE_UNTIL='^### Task B1'` run**, which then also caught `gui/composer_discard.go`'s task assignment. Round 2 verified that Critical and fidelity I-2, C-12, 6b and 8d, and found **three round-1 guards that could not fail their own named mutation** (journey I-6/F-458, tests I-1, tests C-9): all three are replaced with tests that drive the real function or screen, and **each one's failing output under its named mutation is pasted into Task C0**, which now carries applying-the-mutation as a step rather than a hope. Extraction: **47 fences read, 1 dropped by Task B11's `Replace`, 46 kept across 43 files**. Reviews: `composer-S3-plan-R0-r0-{fidelity,tests,journey}.md`, `composer-S3-plan-R0-r1-fold-verification.md`, `composer-S3-plan-R0-r2-fold-verification.md`, `composer-S3-plan-R0-r3-fold-verification.md` (0C/0I: each replaced guard fails its named mutation with the pasted output; 8/8 hunks accounted for).


Nothing here may be implemented until this plan is GREEN (0 Critical / 0 Important) under the R0 loop. **S2 HAS MERGED: fork `main` is `321acb56`**, so PRECONDITION 1 below is SATISFIED and every citation resolves against that tree; five citations moved with the merge and are re-resolved (`gui/multisig_build_slots.go:125->128, :172->175, :238->241`; `md/policy_shape.go:60->53, :74->87`). **This plan's GREEN will expire:** re-validate immediately before dispatching the implementer, per the CLAUDE.md 2026-08-27 directive, with `scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md /scratch/code/shibboleth/seedhammer 321acb56 gui/`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put the composer inside the Wallet Policy program: a door that names its routes and states the key state, a shape flow that builds an ordered spend-path list under a wrapper, lock and hashlock entry on a new digit pad, a paged stub-teaching screen, slot-directed seating from the payload with a paged pick list and a mapping review, a consent surface derived from the DECODED md1, and an engrave form choice with card minting and a plate census. Stage 2 supplies every codec call this stage makes; nothing normative is decided in Go here.

**Architecture:** Two independently shippable PARTS in one plan.

- **Part A** ends at a keyless TEMPLATE engraved from a device with no payload, whose md1 decodes on the device (spec §12 item 3). It touches: `gui/composer_copy.go` (every §8 string, one file), `gui/composer_paged.go` (the paged list primitive `ChoiceScreen` does not have), `gui/composer_door.go`, `gui/composer_shape.go`, `gui/composer_digitpad.go`, `gui/composer_lock.go`, `gui/composer_hash.go`, `gui/composer_stub.go`, `gui/composer_flow.go`, plus four fragments of shipped files (`gui/sysw_admit.go`, `gui/gui.go`, `gui/multisig_build.go`, `gui/template_engrave.go`).
- **Part B** adds seating, the mapping review, the consent self-check and the engrave forms: `gui/composer_sources.go`, `gui/composer_seat.go`, `gui/composer_review.go`, `gui/composer_consent.go`, `gui/composer_engrave.go`, `gui/composer_census.go`, plus one fragment of `gui/sysw_admit_oracle_test.go` and one of `gui/multisig_build_census.go`.

Part A alone is shippable and useful: C26's keyless template is the whole no-payload journey. Part B is not shippable alone.

**Tech Stack:** Go 1.26 (`/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin/go`), TinyGo for `cmd/controller` via `nix run .#build-firmware`. No new module dependencies: everything below is `gui`-internal or already imported by `gui` (`seedhammer.com/{md,mk,sysw,bip32,bip39,backup,engrave,codex32}`, `github.com/btcsuite/btcd/...`). `encoding/json` appears in `_test.go` only.

**Spec:** `design/SPEC_wallet_policy_composer.md` (this repo) -- §4 grammar and picker bounds, §4e refusals, §4f origins, §6a admission and record classes, §6b lock entry, §6c hashlock entry, §7a-§7g the flow, §8 all copy, §9 items 3-7 and 9-11, §12 items 3, 4, 5, 6, 8, 9, §13 item 1. Staged plan: `design/STAGED_PLAN_wallet_policy_composer.md` §S3.

**Baselines (for `scripts/plan-staleness-check.sh`):** seedhammer fork **`321acb56`** (S2 merged 2026-09-02) -- every `path:line` below resolves against that tree. mnemonic-engrave: the S1 merge (`crates/me-cli/src/sysw/composer_records.rs` on master). descriptor-mnemonic `66bdf2f4`.

**PRECONDITIONS, both hard:**

1. **S2 has merged to the fork's `main` -- SATISFIED at `321acb56`.** Every task calls `md.Compose`/`md.ComposeWith`/`md.Composed`, `md.PathList`/`SpendPath`/`KeySet`/`Lock`/`LockKind`/`SlotOrigin`/`DefaultOrigin`/`ValidatePathList`, the `md.ErrCompose*` sentinels, `md.PolicyShape.Branches[].{K,N,Keys,Timelock,Hashlock,Locks,Sha256Digests,Sorted,Depth}`, `md.ComposerStubs`, `mk.AppendStubs`, `sysw.ClassKey`/`ClassHash`/`ClassNow` and `sysw.ParseKeyRecord`/`ParseHashRecord`/`ParseNowRecord`. All of them exist at `321acb56`: `md/compose.go:343` `Compose`, `:353` `ComposeWith`, `:299` `ValidatePathList`, `:289` `DefaultOrigin`, `:250` `Bind`, `:226` `Slots`; `md/compose_stubs.go:12` `ComposerStubs`; `mk/compose_stubs.go:9` `AppendStubs`; `md/policy_shape.go:44` `Branch` with `:65` `Sha256Digests` and `:69` `Sorted`.
2. **F-453's Rust half has shipped and been vendored** before the preset task, and ONLY that task. `design/FOLLOWUPS.md` F-453 owns it to this stage: `md compose --preset <name>` plus one exported vector per archetype in descriptor-mnemonic FIRST, then vendored into the fork. The blank-shape route does not depend on it, so Part A ships with or without presets; the preset task is written with an explicit precondition and is the only thing it blocks.

## Global Constraints

- **Nothing normative is decided in Go here** (CLAUDE.md Rust-primary rule). This stage builds screens. Every template, chunk, id, stub, address and refusal it shows comes from `md`/`mk`/`sysw` calls S2 pinned to the Rust primary. Where a screen and a codec disagree, the codec wins and the screen changes. **A composer path list is a `md.PathList` value handed to `md.Compose`; the GUI never constructs a `descriptor` and never emits text.**
- **File and symbol naming is load-bearing for the gate.** Every new file is `gui/composer_*.go` with its test in `gui/composer_*_test.go`, and **every new test function is named `TestComposer...`**. `scripts/plan-build-gate-go.sh` extracts blocks anchored on `gui/composer_*.go` and, when any exist, runs `go vet ./gui/` and `go test -count=1 -run '^TestComposer' ./gui/`. A test named otherwise is extracted, compiled, and never run by the gate.
- **The §8 bodies are transcribed word for word.** The spec's hard wrap is a document convention, not part of the string: bodies are joined into one paragraph, and the ONLY newlines are (a) after an ALL-CAPS heading line, and (b) between a statement and the instruction that follows it where §8 prints them as separate sentences (§8o, §8p, §8r's flash line, §8s's first body). `assertModalBodyFits` normalises whitespace on both sides, so the assertion compares words, not wrapping.
- **ASCII only, and it is enforced twice.** `scripts/plan-glyph-check.sh` scans this plan's blockquotes; `gui/font_coverage_test.go` and the `strings.ContainsAny(body, "...")` guards in `gui/multisig_build_prose_test.go:91,394` guard the source. A non-ASCII rune blanks the WHOLE modal body, not one glyph.
- **Three copy gates per §8 body, plus one condition test** (§12 item 5): the glyph check on this plan; `assertFrameHasBody` (`gui/raster_test.go:80`) on the frame that shows it; `assertModalBodyFits` (`gui/modal_fits_test.go:201`) with its 80-character margin; and a test that drives the flow into the state that makes it fire. The three PAGED screens are asserted by paging capacity instead of by a fits assertion, because a body with no single source string cannot be pinned by one (§12 item 5 says so).
- **Every new `ChoiceScreen` label passes `assertChoiceLabelFits`** (`gui/multisig_build_prose_test.go:508`). `ChoiceScreen` draws rows with `widget.Label`, which does not wrap, so a long label is drawn off the panel and the operator picks a truncated option.
- **TinyGo target.** No new module dependencies. Watch allocations in `Draw` loops: the paged primitive allocates its `[]op.Op` once per frame with a capacity, as `confirmReviewScreen` does (`gui/multisig_build.go:1894`).
- **The three operator questions have DEFAULTS, and this plan is written to them.** (a) **Part A ships alone**, and its exit is §12 item 3 WALKED (`TestComposerNoPayloadWalkEngravesAKeylessTemplate`); with a payload loaded Part A's door states "Keys loaded: N" over a Build that cannot use them, so Part A alone is honest for the no-payload journey and Part B follows immediately. (b) **§7f offers the secret plate forms this device HAS** -- ms1 through `cardMS1`, the only secret a bundle card carries; the words-plus-SeedQR plate is a `backup.Seed` needing its own plate pass and stays F-455. (c) **Task A10 is BLOCKED on F-453**, whose Rust half is being authored as a mini-plan (S0b); until it lands the shape flow is blank-only, and §7b's preset-or-blank wiring point is named in Task B11 so A10 is a fill-in rather than a redesign.
- **Deprecation is comment-only** (C7, operator ruling). Nothing here removes, gates or redirects Multisig Build.
- **Secret-handling defects never gate** (operator ruling 2026-08-27). The composer's seed handling reuses `seedRegistry` and installs `defer reg.scrub()` at flow entry because that is the right design, not because a leak would block a phase. A leak found here is a follow-up entry with its reproduction.
- **The md1 chunks are the artifact.** Everything the operator is shown at consent is derived from the decoded chunk set, never from composer UI state (§7e). This is not a style rule: it is the property §8q's self-check exists to enforce.
- **Stage paths explicitly** (no `git add -A`); commits signed with DCO (`-s`), author Brian Goss, and the two trailer lines each commit step carries.
- **Back preserves everything** (2026-08-19 operator directive, already the rule in `gui/multisig_build.go:291-299`). Back inside the shape returns to the path list with the list intact; Back at the door leaves the program.

## What is already machine-verified (reviewer budget goes elsewhere)

- **Every `path:line` in this plan was opened and read at fork `169073c` while it was written**, not carried from `design/agent-reports/composer-S3-recon-gui.md`. Three of the recon's facts are corrected below rather than inherited.
- **`mk.Card.Xpub` is a `string` (base58), not `[65]byte`** (`mk/mk.go:133-139`, read). The recon's fact table says `[65]byte`; that is `md.ExpandedKey.Xpub`'s type (`md/expand.go:84-95`), which is chaincode in `[0:32]` and the compressed pubkey in `[32:65]` (`gui/key_card_seating.go:107-110`). Card minting takes the base58 string from `deriveAccountXpub` (`gui/derive.go:19`) verbatim, exactly as `gui/derive_xpub.go:445-455` and `gui/multisig_derive.go:47-53` do; `md.Composed.Bind` takes the `[65]byte` form, built with `decodeXpubBytes` (`gui/singlesig_derive.go:110`).
- **`slotMatchesCard` is at `gui/key_card_seating.go:128`**, not `:119` (the doc comment starts at `:118`). The fingerprint-elision behaviour §4f's invariant is built on is at `:151-159`.
- **`gui/sysw_admit_oracle_test.go:90` `TestEverySyswConsumptionSiteNamesAnAdmittedClass` will FAIL the moment the composer takes a record**, and the recon does not mention it. It walks every non-test `syswOffer*(...)` and `.take(...)` call by AST and fails on a site absent from `syswConsumers` (`:39-76`) or naming a class absent from `classNames` (`:78-88`). `classNames` today holds eight entries and does NOT include `ClassMt`, `ClassTx`, or the three new composer classes. `takeAll` and `cardSet` are invisible to it (`:127-133` matches `take` by exact selector name), which is a real gap in the oracle and is why the composer's sources task registers its sites by hand rather than relying on the scan.
- `confirmReviewScreen` is defined at `gui/multisig_build.go:1877-1939`; its pager-icon gate is `:1926-1931`; `gui/multisig_build.go:804` is a CALL SITE inside `templateConsentFlow` (`:780`), not the paging logic.
- `policySummaryLines` is DEFINED at `gui/template_engrave.go:142` and has exactly one call site, `gui/template_engrave.go:86`. `templateConsentLines` is defined at `gui/template_engrave.go:63`, and prints `Template-ID:` at `:70` and `:79` -- both, so the relabelling is two edits, not one.
- **`gui/sim*.go` does not exist.** `find . -iname '*sim*.go'` over the fork, excluding `_experiment/`, returns nothing. The emulator is `cmd/emu/` (a WASM build plus JS walk scripts: `walk_build_policy.js`, `walk_s3_nested.js`, `walk_s4_gate.js`, `walk_trace_a.js`, `walk_trace_b.js`, `walk_verify.js`, and the `shots_*.js` screenshot scripts). Go-level journeys are ordinary `_test.go` files on the `synctest.Test` / `runUI` / `pumpUntil` / `click` / `uiContains` harness (`gui/wallet_policy_descriptor_walk_test.go:128-176`).
- `scripts/plan-build-gate-go.sh` extracts every ```go block anchored by ``Create `path` ``, ``In `path` ``, ``Add to `path` ``, ``Prepend to `path` `` or ``Replace `path` `` where the path matches `gui/composer_*.go` (also `md/compose*.go`, `mk/compose*.go`, `sysw/composer_*.go`). **It does NOT assemble fragments of existing files** -- in this plan those are `gui/sysw_admit.go`, `gui/gui.go`, `gui/multisig_build.go`, `gui/template_engrave.go`, `gui/multisig_build_census.go` and `gui/sysw_admit_oracle_test.go`, each given below as an exact old-to-new replacement for the controller to hand-wire in the gate's scratch copy before review. It does not cover the TinyGo build, the whole `./gui/` suite (sharded separately), the emulator, or the render measurements.
- Baseline before this plan, at `169073c`: `CGO_ENABLED=0 go test -count=1 ./gui/` is `ok`; firmware `tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` reports **1,503,652 B flash / 62,592 B RAM**.
- **THE GATE WAS RUN ON THIS PLAN, AND ITS CODE WAS EXECUTED, NOT JUST COMPILED.** `FORK_REPO=/scratch/code/shibboleth/seedhammer SCRATCH=/scratch/code/shibboleth/.plan-build-gate-go-s3 ./scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` extracted every ```go block into a scratch copy of fork `main`; `gofmt` reports no change on any of them; `go vet ./gui/` is clean; the six fragments were hand-wired and the five shipped walks given their door step; and `scripts/gui-shard-test.sh ./gui/ 24` then reported **`ok -- all 1125 tests ran across 24 shards`**. So every `TestComposer*` in this plan has passed against real code once already.
- **Five defects this found, before a reviewer saw them**, listed because they say what the gate is worth and what it is not: `composerUnitsToDays` rounded UP and echoed "91 days" at an operator who typed 90; two Unix epochs in the lock tests were a year and two days wrong (`1756684800` is 2025-09-01, not 2026-09-01; `1804032000` is 2027-03-03, not 2027-03-01); a paging assertion pumped for a needle a rename had removed; the scrub test asserted `Mnemonic[0] != 0` on the "abandon" vector, whose first eleven words ARE index 0, so it could not distinguish scrubbed from never-written; and `TestComposerAdmitCommentNoLongerClaimsNoSeedClass` scanned the whole file for "NO seed class", which `progTransaction`'s own untouched comment also says -- it would have failed after a perfectly correct fold. **None of the five is a design defect and none would have been caught by reading.**
- **One S2 signal, passed on rather than fixed here:** in the gate's scratch copy `sysw`'s `TestComposerRecordsClassifyExactlyAsTheHost` and `TestComposerRecordParsersReturnTheHostsValues` FAIL, because the record-class fixture on mnemonic-engrave master now hashes `5b3960ca…b312` while S2's branch pins `eed6b177…464e`. That is S2's pin against a moved fixture, not this plan's, and it belongs in S2's own re-validation before its implementer is dispatched.

## The nine things the recon found that the spec assumed otherwise

Each is resolved in this plan, in the named task, rather than left for the implementer to discover.

| # | what the spec assumes | what the code does at `169073c` | resolved by |
| --- | --- | --- | --- |
| 1 | §7f: three secret forms (words, SeedQR, ms1) the operator picks between | TWO paths: `engraveSeed` (`gui/gui.go:839`) bakes words AND a SeedQR onto ONE plate via `backup.Seed`; `backup.SeedString` (`backup/backup.go:26`) is the string-only plate `engraveCodex32` (`gui/codex32_polish.go:218`) cuts for ms1. No words-only and no QR-only plate exists for a mnemonic | Engrave forms: the choice offers the TWO forms the device has, labelled for what they are, and reuses both shipped functions. Inventing a third plate design is out of scope and is filed |
| 2 | §9 item 7: a paged pick list, framed as polish | `ChoiceScreen.Draw` (`gui/gui.go:1966-2029`) stacks children with `h += c.Size.Y`, no clip, no bound against the 232 px content box, and `Choose` (`:1910`) has no scroll offset at all. An over-long list draws off frame and Down still selects invisible rows | The paged list primitive is the FIRST task of Part A and everything paged is built on it |
| 3 | §9 item 4: "flag-screen wiring" as device work | F1/F2 fire ONCE, at payload LOAD (`gui/sysw_load.go:210-231`), from `syswLoadFlow`'s three call sites (`gui/gui.go:2074`, `gui/sysw_unload.go:36,75`) -- none per program. `syswLoadWarnings` (`gui/sysw_load.go:259`) walks records with no admission check at all, so a mnemonic in a payload already raises F1 today | The admission row change plus a walk test proving F1 fires for a composer payload and that NO per-program call was added. A spec §6a fold records that the flags are load-time, not seed-step |
| 4 | `gui/multisig_build.go:804` is "the paged form" | `:804` is a call site; the paging is `confirmReviewScreen` at `:1877-1939` with the icon gate at `:1926-1931` | The consent task cites the definition and the gate separately |
| 5 | `policySummaryLines (:142)` and `templateConsentLines (:86)` conflated | def `:142` / call `:86`; `templateConsentLines` def `:63`; `Template-ID:` printed at BOTH `:70` and `:79` | The relabelling task replaces both occurrences and says so |
| 6 | §9 item 11: scrub-on-exit "through `buildMultisigSeedHook`'s seam" | `buildMultisigSeedHook` (`gui/multisig_build.go:38`) is a test-observation seam that zeroes nothing and is nil in production; the real scrub is `defer reg.scrub()` at `gui/multisig_build.go:291`, installed before any seed exists | The seed-source task installs `defer reg.scrub()` at the composer flow's own entry, and adds its own `composerSeedHook` for the same observation purpose |
| 7 | `gui/sim*.go` and JSON journeys under `gui/testdata/` | Neither exists; the emulator is `cmd/emu/` (WASM + JS walks) and Go journeys are `_test.go` files | Every acceptance in this plan is a Go `_test.go` on the shipped harness; the emulator journey is S4's, not this stage's |
| 8 | §13 item 1: a plate ceiling to be read off | `qrCeilingBytes` (`gui/transaction.go:1369`) MEASURES its ceiling by binary search against `txqr.EncodeSet` module fit, on the refusal path only, and its refusal names module size, symbol count, ECC, the measured ceiling and a form-aware remedy (`:1336-1344`) | The census task copies that shape exactly for the concrete-descriptor text plate, and the measurement task records the number |
| 9 | §13: per-frame capacities of the three paged screens | Unmeasurable from source: `Choice.Size` and `widget.Labelw` sizes exist only at render time | One measurement task, run on the real face at the real display size, writing the numbers into spec §13 as a fold |


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
| I-6 a date past 2038-01-19 refused as "does not exist" | **Task A1** adds `composerCopyDateCeiling` (the copy table moves 39 -> 40) and **Task A7** uses it; filed as a §8 addition, F-456. **Round 1 found the fold's own dispatch defective** and it is fixed in Task A7 with its own guard: `composerDateToUnix` returns 0 on EVERY failure, so the ceiling test `y > 2038` OR-ed with `u == 0` was a tautology and "that date does not exist" was dead code -- 2027-02-31 got the ceiling body, advising a block height for a date no height makes real. `composerDateExists` now tells the three failures apart. Filed **F-458**, distinct from F-456 |
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
| I-7 the pager-gate test cannot fail | **Task C0**: replaced by `TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage`, which asserts the BEHAVIOUR and fails in both directions. Round 1 found the old `longInk <= shortInk` test still sitting in `composer_paged_test.go` while this table claimed it "gone"; **Task A2 now deletes it**, with the reason in its place |
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

## R0 round 2: the three guards that could not fail, and where each is replaced

`design/agent-reports/composer-S3-plan-R0-r2-fold-verification.md` returned **NOT GREEN**. It verified the round-1 Critical (Task A11 standalone) by an independently-run Part-A-only gate, and verified fidelity I-2, C-12, cell 6b and cell 8d under their named mutations. What it found open was one class, three times: **a regression guard whose own named mutation, reproduced exactly, does not fail it.**

| # | round-2 finding | folded into |
| --- | --- | --- |
| §3 | journey I-6 / F-458's guard called the pure date helpers and re-derived the dispatch inline; `composerLockEdit` had zero test callers | **Task C0**: `TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate` drives the real screen through the kind picker, the absolute-kind picker and the digit pad. Failing output pasted |
| §4 | tests I-1's guard used an ODD 63-character input, so `hex.DecodeString` refused it for parity before the bound was consulted -- and read a return value the function never produces on a refusal | **Task C0**: `TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters` asserts on the error screen the accept branch draws, with an even twin at 62 for the opposite mutation. Failing output pasted |
| §4 | tests C-9's claimed closure rested on an unmodified pre-round-1 test that recomputes the comparison standalone and never calls `composerFlow` | **Task C0**: `TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit` walks `composerFlow` twice through the stub screen, with and without a real edit. Failing output pasted |
| §4 | journey I-5's named mutation is a structural no-op | recorded, not re-fixed: `composerKeysEdit`'s decline paths never write `Keys` before its single success-path assignment, so nothing needs restoring in the current call graph. Round 0 and round 1 established the same; the test is real and drives the UI through Back, it is just not effective for that one mutation |
| §6 | **NEW Minor** -- Task C1's own commit template still said "three things" and listed three after the task gained a fourth | **Task C1**: the template names four and its body carries the §7a item |
| §7 | Task B11's "two named exemptions" wording (only one logs) | left as round 1 classified it: the reachability test exempts two by name, the gate's step counts references and reports one. Two numbers, two questions -- the plan says so where both appear |

**The lesson, recorded once.** All three failures were tests that recomputed the rule beside the code instead of running the code, and each was written in response to a finding of exactly that shape. Task C0 now carries **applying each guard's named mutation and seeing it fail** as its own step, with the output in the plan.

---

# PART A -- the door, the shape, the stub screen, the keyless template

Part A's exit is spec §12 item 3: a device with NO payload reaches Wallet Policy, chooses Build, composes a shape, reads the stub screen with its per-slot expected origins, consents to a keyless template, and engraves an md1 that DECODES on the device with distinct-account origins on every slot.

### Task A1: `gui/composer_copy.go` -- every §8 body in one file, with a countable coverage table

**Files:**
- Create: `gui/composer_copy.go`
- Test: `gui/composer_copy_test.go`

**Interfaces:**
- Consumes: nothing but `fmt`. This file has no imports from the rest of the tree on purpose -- a copy file that reaches into the flow is a copy file that cannot be tested without one.
- Produces: 39 `composerCopy*` functions, one per §8 body, plus `composerConfirmBody(body string) string` (appends the hold-to-confirm instruction the `ConfirmWarningScreen` shape requires, so the §8 text itself stays verbatim) and two list helpers `composerSlotList`, `composerSlotWord`.

Why one file: §12 item 5 demands three gates and a condition test for EVERY §8 body, and "every" is only checkable if the bodies are enumerable. The test below AST-scans this file for `composerCopy*` declarations and fails when one is absent from the coverage table -- so a body added later without a gate turns the suite red instead of shipping unasserted.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_copy_test.go`:

```go
package gui

import (
	"go/ast"
	"go/parser"
	"go/token"
	"strings"
	"testing"
)

// composerCopyRow is one operator-facing body, its §8 section, and the exact
// text the spec prints for it. `verbatim` is compared WORD FOR WORD after
// whitespace normalisation, so a reviewer diffing this table against SPEC
// §8 is diffing the shipped strings.
type composerCopyRow struct {
	fn       string // the composerCopy* function this row covers
	section  string // the §8 subsection
	got      string // what the function returns for the spec's own example
	verbatim string // SPEC §8, transcribed
}

// composerCopyTable is the whole of §8 as the device draws it.
//
// EVERY ROW IS A CONTRACT WITH THREE OTHER TESTS: the raster floor and the
// modal-fits assertion run over `got` (composer_copy_gate_test.go), and a
// fires-on-condition test drives the flow into the state that shows it
// (named in the section's own task). This table is what makes the count
// exact.
func composerCopyTable() []composerCopyRow {
	return []composerCopyRow{
		{"composerCopyKeylessPath", "8a", composerCopyKeylessPath(),
			"KEY-LESS PATH (EXPERIMENTAL) This path needs no signature. Whoever knows the preimage of its hash can spend it. If that preimage is ever engraved, the plate is bearer access."},
		{"composerCopyUnsortedKeys", "8b", composerCopyUnsortedKeys(),
			"UNSORTED KEYS (EXPERIMENTAL) You chose unsorted keys where sorted was possible. Key order is part of this wallet. Anyone restoring it must keep the same order. Sorted keys need none."},
		{"composerCopyLockEchoDays", "8c", composerCopyLockEchoDays(90, 15188),
			"90 days = 15188 units of 512 s (90.0 days)"},
		{"composerCopyLockEchoBlocks", "8c", composerCopyLockEchoBlocks(1000),
			"1000 blocks (about 6.9 days)"},
		{"composerCopyLockEchoHeight", "8c", composerCopyLockEchoHeight(905000),
			"Block 905000"},
		{"composerCopyLockEchoDate", "8c", composerCopyLockEchoDate(2027, 3, 1),
			"2027-03-01 00:00 UTC"},
		{"composerCopyPackedDateBound", "8c", composerCopyPackedDateBound("2026-09-01"),
			"This device cannot tell the time. The payload says it was packed on 2026-09-01, which may be long ago. Nothing here has checked that this is in the future."},
		{"composerCopyPackedHeightBound", "8c", composerCopyPackedHeightBound(905000),
			"This device cannot tell the time. The payload says the packed height was 905000, which may be long ago. Nothing here has checked that this is in the future."},
		{"composerCopyNoBound", "8c", composerCopyNoBound(),
			"This device cannot tell the time. Nothing here has checked that this is in the future."},
		{"composerCopyOwnWallet", "8d", composerCopyOwnWallet(),
			"A wallet built here is its own wallet. The same rules written by another tool give a different id and different addresses."},
		{"composerCopyNUMS", "8f", composerCopyNUMS(),
			"KEY PATH: NONE (NUMS) Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449)."},
		{"composerCopySameSeedThreshold", "8g", composerCopySameSeedThreshold([]uint8{1, 2}, 2, 3),
			"SAME SEED, SAME PATH Slots @1 and @2 are the same seed. This path's 2-of-3 can be satisfied by one person. Liana will refuse it."},
		{"composerCopySameSeedBelow", "8g", composerCopySameSeedBelow([]uint8{1, 2}, 3),
			"SAME SEED, SAME PATH Slots @1 and @2 are the same seed. One person holds 2 of the 3 signatures this path needs. Liana will refuse it."},
		{"composerCopyHashEveryPath", "8h", composerCopyHashEveryPath(),
			"HASH ON EVERY PATH Every way to spend this wallet needs the preimage of a hash. It is not on this device and not on these plates. Back the preimage up separately."},
		{"composerCopyHashRule", "8i", composerCopyHashRule(),
			"The hash must be SHA-256 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. A hash of the passphrase itself can never be spent."},
		{"composerCopyEditClearsKeys", "8j", composerCopyEditClearsKeys(),
			"EDITING THE SHAPE CLEARS THE KEYS Slot numbers change with the shape. Every key you seated will be cleared. Continue?"},
		{"composerCopyPersonInTwoPaths", "8k", composerCopyPersonInTwoPaths(),
			"One person in two paths needs two keys: a second account from the same seed, or a second card."},
		{"composerCopyNothingChecked", "8l", composerCopyNothingChecked(),
			"Nothing outside this device has checked this policy. Before you fund it, restore these plates in your coordinator and compare your own first receive address."},
		{"composerCopyRefuseNoKeyedPath", "8m", composerCopyRefuseNoKeyedPath(),
			"Every wallet needs at least one path with a key."},
		{"composerCopyRefuseLockOnly", "8m", composerCopyRefuseLockOnly(),
			"A path with only a time lock means anyone can spend after it. Add a key or a hash."},
		{"composerCopyRefuseKeylessTr", "8m", composerCopyRefuseKeylessTr(),
			"This build will not put a key-less path in taproot. Use wsh, or add a key."},
		{"composerCopyRefuseLegacyShape", "8m", composerCopyRefuseLegacyShape(),
			"Legacy wrappers hold one plain multisig only. Use wsh or tr."},
		{"composerCopyRefuseSlotCap", "8m", composerCopyRefuseSlotCap(),
			"This wallet already has 32 key slots."},
		{"composerCopyBelowBoundDate", "8o", composerCopyBelowBoundDate(),
			"That is before this payload was packed. Choose a later date."},
		{"composerCopyBelowBoundHeight", "8o", composerCopyBelowBoundHeight(),
			"That is before this payload was packed. Choose a later height."},
		{"composerCopyShortfall", "8p", composerCopyShortfall(4, 3, []uint8{3}),
			"4 slots, 3 keys available. Unfilled: slot @3."},
		{"composerCopySelfCheckFailed", "8q", composerCopySelfCheckFailed(),
			"The policy on this device does not match what you built. Go back and check the path list, or start again."},
		{"composerCopyKeysLoaded", "8r", composerCopyKeysLoaded(4),
			"Keys loaded: 4"},
		{"composerCopyKeysAndSeeds", "8r", composerCopyKeysAndSeeds(4, 1),
			"Keys loaded: 4, plus 1 seed."},
		{"composerCopySeedOnly", "8r", composerCopySeedOnly(),
			"A seed is loaded. It can fill any number of slots."},
		{"composerCopyNotUnderstood", "8r", composerCopyNotUnderstood(3),
			"3 payload records were not understood."},
		{"composerCopyNoKeys", "8r", composerCopyNoKeys(),
			"No keys loaded. This builds a key-less template."},
		{"composerCopyPayloadNotLoaded", "8r", composerCopyPayloadNotLoaded(),
			"A payload is in flash but not loaded. Load it from the carousel first."},
		{"composerCopyIdChanged", "8s", composerCopyIdChanged(),
			"The shape changed, so this id changed. Cards minted with the old stub will not seat here."},
		{"composerCopySeatPrompt", "8s", composerCopySeatPrompt(2, 1, 2, 3),
			"Slot @2, Path 1 key 2 of 3: choose a key"},
		{"composerCopySeatKeyPathPrompt", "8s", composerCopySeatKeyPathPrompt(0),
			"Slot @0, key path (spends alone): choose a key"},
		{"composerCopyDateFloor", "8t", composerCopyDateFloor(),
			"This build will not write a date before 2009 as a time lock."},
		{"composerCopyDateCeiling", "8t", composerCopyDateCeiling(),
			"This build writes dates up to 2038-01-19. For a later time, use a block height instead."},
		{"composerCopyRelativeCeiling", "8u", composerCopyRelativeCeiling(),
			"Relative locks reach at most 455 days in blocks or 388 days in time. Use an absolute date."},
		{"composerCopySameOriginFewFingerprints", "8v", composerCopySameOriginFewFingerprints(),
			"Two keys declare the same origin and not both carry a fingerprint. This template could not be restored. Use cards or records with fingerprints."},
	}
}

// TestComposerCopyIsVerbatimFromTheSpec compares every shipped string with
// SPEC §8 word for word.
//
// normalizeDrawn is deliberately the comparator: it is the same reduction
// assertModalBodyFits applies to a drawn frame, so a row that passes here
// passes there for the same reason -- and the spec's hard wrap, which is a
// document convention, is not mistaken for a difference in the string.
func TestComposerCopyIsVerbatimFromTheSpec(t *testing.T) {
	for _, r := range composerCopyTable() {
		if normalizeDrawn(r.got) != normalizeDrawn(r.verbatim) {
			t.Errorf("%s (SPEC §%s) does not match the spec.\n got:  %q\n want: %q",
				r.fn, r.section, r.got, r.verbatim)
		}
	}
}

// TestComposerCopyIsDrawable is the shipped prose guard, applied to all 39.
//
// A rune the body face lacks does not degrade one glyph: it blanks the whole
// modal body (gui/font_coverage_test.go). The banned set here is the one
// gui/multisig_build_prose_test.go:91 refuses, verbatim.
func TestComposerCopyIsDrawable(t *testing.T) {
	for _, r := range composerCopyTable() {
		if strings.ContainsAny(r.got, "—–·‘’“”…") {
			t.Errorf("%s carries a glyph the body face lacks, so its line does not draw:\n%q", r.fn, r.got)
		}
		for _, ch := range r.got {
			if ch > 126 || (ch < 32 && ch != '\n') {
				t.Errorf("%s carries the non-ASCII or control rune %q; device strings are ASCII only", r.fn, ch)
			}
		}
	}
}

// TestComposerCopyTableCoversEveryBody is the reason this file exists.
//
// It parses composer_copy.go and requires every composerCopy* declaration to
// appear in the table. A body added later without a row would otherwise ship
// with none of §12 item 5's four gates on it, and nothing would say so.
func TestComposerCopyTableCoversEveryBody(t *testing.T) {
	fset := token.NewFileSet()
	f, err := parser.ParseFile(fset, "composer_copy.go", nil, 0)
	if err != nil {
		t.Fatalf("parsing composer_copy.go: %v", err)
	}
	covered := map[string]bool{}
	for _, r := range composerCopyTable() {
		covered[r.fn] = true
	}
	var declared int
	for _, decl := range f.Decls {
		fn, ok := decl.(*ast.FuncDecl)
		if !ok || fn.Recv != nil || !strings.HasPrefix(fn.Name.Name, "composerCopy") {
			continue
		}
		declared++
		if !covered[fn.Name.Name] {
			t.Errorf("%s is declared in composer_copy.go but is in no row of "+
				"composerCopyTable -- so SPEC §12 item 5's glyph, raster, "+
				"modal-fits and fires-on-condition gates do not reach it",
				fn.Name.Name)
		}
		delete(covered, fn.Name.Name)
	}
	for stray := range covered {
		t.Errorf("composerCopyTable names %s, which composer_copy.go does not declare", stray)
	}
	// 40 SINCE THE DATE CEILING. §8t covered the floor and §8 had no body for
	// the top, so a date past 2038-01-19 was refused as "that date does not
	// exist" -- false of 2045-06-01, on the archetype §4d lists first. The new
	// body is filed as a §8 addition (F-456) so the spec stays the source this
	// table is diffed against.
	if declared != 40 {
		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 40 -- "+
			"if that is deliberate, update both", declared)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerCopy' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerCopyKeylessPath` and 38 more.

- [ ] **Step 3: Write the copy**

Create `gui/composer_copy.go`:

```go
package gui

import "fmt"

// Every operator-facing string the wallet-policy COMPOSER draws, in one file
// (SPEC_wallet_policy_composer.md §8).
//
// ONE FILE, AND THE REASON IS THE GATE, NOT TIDINESS. §12 item 5 requires the
// glyph check, the raster floor, the modal-fits assertion and a
// fires-on-condition test for EVERY §8 body. "Every" is only checkable if the
// bodies are enumerable, so composer_copy_test.go AST-scans this file and
// fails when a composerCopy* function is missing from its table. A body
// written inline at its screen is a body nobody counted.
//
// ASCII ONLY. A non-ASCII rune does not degrade one glyph, it blanks the
// WHOLE modal body (gui/font_coverage_test.go), and an em dash measured 2652
// raster pixels against 7419 for the same line with a hyphen.
//
// THE SPEC'S HARD WRAP IS NOT PART OF THE STRING. §8 wraps its blockquotes at
// about 48 columns because that is a readable document; the panel wraps at
// the real face and the real width. So each body is ONE paragraph, and the
// only newlines are after an all-caps heading line and between a statement
// and the instruction that follows it.

// composerConfirmBody appends the hold-to-confirm instruction to a body shown
// on a ConfirmWarningScreen.
//
// It is separate so the §8 text stays verbatim: the instruction describes the
// CONTROL, not the policy, and gui/multisig_build.go:879 carries the same
// sentence for the same reason. The shipped prose test requires it (
// gui/multisig_build_prose_test.go:84).
func composerConfirmBody(body string) string {
	return body + "\n\nHold button to confirm."
}

// composerSlotWord renders "slot @3" or "slots @3 and @4", so a refusal never
// reads "slots @3".
func composerSlotWord(slots []uint8) string {
	if len(slots) == 1 {
		return fmt.Sprintf("slot @%d", slots[0])
	}
	return "slots " + composerSlotList(slots)
}

// composerSlotList joins slot labels the way a person reads them: "@1 and @2"
// for two, "@1, @2 and @3" beyond.
func composerSlotList(slots []uint8) string {
	switch len(slots) {
	case 0:
		return ""
	case 1:
		return fmt.Sprintf("@%d", slots[0])
	}
	out := ""
	for i, s := range slots {
		switch {
		case i == 0:
			out = fmt.Sprintf("@%d", s)
		case i == len(slots)-1:
			out += fmt.Sprintf(" and @%d", s)
		default:
			out += fmt.Sprintf(", @%d", s)
		}
	}
	return out
}

// ─── §8a, §8b: the two EXPERIMENTAL confirm-to-proceed bodies ────────────────

func composerCopyKeylessPath() string {
	return "KEY-LESS PATH (EXPERIMENTAL)\n" +
		"This path needs no signature. Whoever knows the preimage of its hash can " +
		"spend it. If that preimage is ever engraved, the plate is bearer access."
}

func composerCopyUnsortedKeys() string {
	return "UNSORTED KEYS (EXPERIMENTAL)\n" +
		"You chose unsorted keys where sorted was possible. Key order is part of " +
		"this wallet. Anyone restoring it must keep the same order. Sorted keys " +
		"need none."
}

// ─── §8c: the five lock echoes plus the two bound lines ──────────────────────

// composerCopyLockEchoDays echoes a relative TIME lock. Both the operator's
// days and the encoded units are printed, with the units converted BACK to
// days, because ceil() to 512-second units does not round-trip: the operator
// is entitled to see what the wallet will actually enforce.
func composerCopyLockEchoDays(days, units uint32) string {
	back := float64(units) * 512 / 86400
	return fmt.Sprintf("%d days = %d units of 512 s (%.1f days)", days, units, back)
}

// composerCopyLockEchoBlocks echoes a relative BLOCK lock (§6b's table).
// 600 seconds a block is the same figure §4c's "455.1 days" ceiling comes
// from: 65535 * 600 / 86400.
func composerCopyLockEchoBlocks(blocks uint32) string {
	return fmt.Sprintf("%d blocks (about %.1f days)", blocks, float64(blocks)*600/86400)
}

func composerCopyLockEchoHeight(height uint32) string {
	return fmt.Sprintf("Block %d", height)
}

func composerCopyLockEchoDate(year, month, day int) string {
	return fmt.Sprintf("%04d-%02d-%02d 00:00 UTC", year, month, day)
}

// composerCopyPackedDateBound is §8c's fourth body: the disclaimer WITH the
// payload's pack date. It never withdraws the disclaimer and never says
// "now" (§6b), because a stale now: record can only weaken the below-bound
// refusal, never invent one.
func composerCopyPackedDateBound(packDate string) string {
	return "This device cannot tell the time. The payload says it was packed on " +
		packDate + ", which may be long ago. Nothing here has checked that this " +
		"is in the future."
}

// composerCopyPackedHeightBound is the same body with §6b's height clause:
// "heights read `the packed height was H`". §8c prints the date form
// verbatim and §6b rules the height wording; this is the two joined, and it
// is the one string in this file assembled from two spec sentences rather
// than quoted from one.
func composerCopyPackedHeightBound(height uint32) string {
	return fmt.Sprintf("This device cannot tell the time. The payload says the packed "+
		"height was %d, which may be long ago. Nothing here has checked that this "+
		"is in the future.", height)
}

func composerCopyNoBound() string {
	return "This device cannot tell the time. Nothing here has checked that this " +
		"is in the future."
}

// ─── §8d, §8f ────────────────────────────────────────────────────────────────

func composerCopyOwnWallet() string {
	return "A wallet built here is its own wallet. The same rules written by " +
		"another tool give a different id and different addresses."
}

func composerCopyNUMS() string {
	return "KEY PATH: NONE (NUMS)\n" +
		"Spends use the script paths only. Bitcoin Core and Nunchuk import this " +
		"form. Liana and BIP-388 signers need an unspendable xpub instead (see " +
		"F-449)."
}

// ─── §8g: C29, one seed at two slots INSIDE one path ─────────────────────────

// composerCopySameSeedThreshold is §8g's FIRST body: the shared seed's slots
// in this path REACH the threshold, so one person can satisfy the path alone.
func composerCopySameSeedThreshold(slots []uint8, k, n int) string {
	return fmt.Sprintf("SAME SEED, SAME PATH\nSlots %s are the same seed. This path's "+
		"%d-of-%d can be satisfied by one person. Liana will refuse it.",
		composerSlotList(slots), k, n)
}

// composerCopySameSeedBelow is §8g's SECOND body: shared, but short of the
// threshold, so it says how much of it one person holds.
func composerCopySameSeedBelow(slots []uint8, k int) string {
	return fmt.Sprintf("SAME SEED, SAME PATH\nSlots %s are the same seed. One person "+
		"holds %d of the %d signatures this path needs. Liana will refuse it.",
		composerSlotList(slots), len(slots), k)
}

// ─── §8h, §8i, §8j, §8k, §8l ─────────────────────────────────────────────────

func composerCopyHashEveryPath() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs the preimage of a hash. It is not " +
		"on this device and not on these plates. Back the preimage up separately."
}

func composerCopyHashRule() string {
	return "The hash must be SHA-256 of a 32-byte value. A passphrase must be " +
		"hashed to 32 bytes first, then hashed again. A hash of the passphrase " +
		"itself can never be spent."
}

func composerCopyEditClearsKeys() string {
	return "EDITING THE SHAPE CLEARS THE KEYS\n" +
		"Slot numbers change with the shape. Every key you seated will be " +
		"cleared. Continue?"
}

func composerCopyPersonInTwoPaths() string {
	return "One person in two paths needs two keys: a second account from the " +
		"same seed, or a second card."
}

// composerCopyNothingChecked is §8l.
//
// §8l names it "Multisig Build's warning, reused", and the SURFACE is reused
// -- an unskippable ConfirmWarningScreen -- but the STRING is not: the
// shipped body (gui/multisig_build.go:872-879) is a different, longer text,
// and §8 is the normative copy for this program. The shipped body is NOT
// edited by this cycle; changing a shipped screen's warning is not this
// stage's work.
func composerCopyNothingChecked() string {
	return "Nothing outside this device has checked this policy. Before you fund " +
		"it, restore these plates in your coordinator and compare your own first " +
		"receive address."
}

// ─── §8m: the five structural refusals (§4e) ─────────────────────────────────

func composerCopyRefuseNoKeyedPath() string {
	return "Every wallet needs at least one path with a key."
}

func composerCopyRefuseLockOnly() string {
	return "A path with only a time lock means anyone can spend after it. Add a " +
		"key or a hash."
}

func composerCopyRefuseKeylessTr() string {
	return "This build will not put a key-less path in taproot. Use wsh, or add a key."
}

func composerCopyRefuseLegacyShape() string {
	return "Legacy wrappers hold one plain multisig only. Use wsh or tr."
}

func composerCopyRefuseSlotCap() string {
	return "This wallet already has 32 key slots."
}

// ─── §8o, §8p, §8q ───────────────────────────────────────────────────────────

func composerCopyBelowBoundDate() string {
	return "That is before this payload was packed.\nChoose a later date."
}

func composerCopyBelowBoundHeight() string {
	return "That is before this payload was packed.\nChoose a later height."
}

// composerCopyShortfall is §8p. It names the counts and the unfilled slots
// and GUESSES NO CAUSE: the C5 lesson (a person in two paths needs two keys)
// is taught at the shape step by §8k, and a guess here would be a second,
// possibly wrong, explanation on the screen that refuses.
func composerCopyShortfall(slots, available int, unfilled []uint8) string {
	return fmt.Sprintf("%d slots, %d keys available.\nUnfilled: %s.",
		slots, available, composerSlotWord(unfilled))
}

func composerCopySelfCheckFailed() string {
	return "The policy on this device does not match what you built. Go back and " +
		"check the path list, or start again."
}

// ─── §8r: the door's key-state lines ─────────────────────────────────────────

func composerCopyKeysLoaded(n int) string {
	return fmt.Sprintf("Keys loaded: %d", n)
}

// composerCopyKeysAndSeeds pluralises the SEED noun with its own count (§7a),
// exactly as the not-understood line pluralises its record noun.
func composerCopyKeysAndSeeds(keys, seeds int) string {
	noun := "seeds"
	if seeds == 1 {
		noun = "seed"
	}
	return fmt.Sprintf("Keys loaded: %d, plus %d %s.", keys, seeds, noun)
}

// composerCopySeedOnly prints NO COUNT for the seeds, and that is §7a's rule
// rather than an omission: a seed fills any number of slots, so a count of
// seeds would answer a question the operator is not asking.
func composerCopySeedOnly() string {
	return "A seed is loaded. It can fill any number of slots."
}

func composerCopyNotUnderstood(n int) string {
	if n == 1 {
		return "1 payload record was not understood."
	}
	return fmt.Sprintf("%d payload records were not understood.", n)
}

func composerCopyNoKeys() string {
	return "No keys loaded. This builds a key-less template."
}

func composerCopyPayloadNotLoaded() string {
	return "A payload is in flash but not loaded.\nLoad it from the carousel first."
}

// ─── §8s: the stub screen's changed-id line and the two seating prompts ──────

func composerCopyIdChanged() string {
	return "The shape changed, so this id changed. Cards minted with the old stub " +
		"will not seat here."
}

// composerCopySeatPrompt names the OPERATOR's listed path index, never an
// emitted leaf index (§7d), beside the EMITTED slot index the labels use.
func composerCopySeatPrompt(slot uint8, path, keyIdx, keyCount int) string {
	return fmt.Sprintf("Slot @%d, Path %d key %d of %d: choose a key",
		slot, path, keyIdx, keyCount)
}

func composerCopySeatKeyPathPrompt(slot uint8) string {
	return fmt.Sprintf("Slot @%d, key path (spends alone): choose a key", slot)
}

// ─── §8t, §8u, §8v ───────────────────────────────────────────────────────────

func composerCopyDateFloor() string {
	return "This build will not write a date before 2009 as a time lock."
}

// composerCopyDateCeiling is the ceiling line §8 does not yet carry.
//
// §8t covers the FLOOR ("before 2009") and nothing covered the top, so a date
// past 2038-01-19 was refused as "that date does not exist" -- which is false
// of 2045-06-01 and leaves the operator retyping. The archetype §4d lists
// first is simple-timelocked-inheritance, where a twenty-year date is the
// ordinary case, so this is the sentence that use meets. Filed as a §8
// addition (F-456) so the spec's copy stays the enumerable source.
func composerCopyDateCeiling() string {
	return "This build writes dates up to 2038-01-19. For a later time, use a " +
		"block height instead."
}

func composerCopyRelativeCeiling() string {
	return "Relative locks reach at most 455 days in blocks or 388 days in time. " +
		"Use an absolute date."
}

func composerCopySameOriginFewFingerprints() string {
	return "Two keys declare the same origin and not both carry a fingerprint. " +
		"This template could not be restored. Use cards or records with " +
		"fingerprints."
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerCopy' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: `--- PASS` for `TestComposerCopyIsVerbatimFromTheSpec`, `TestComposerCopyIsDrawable`, `TestComposerCopyTableCoversEveryBody`; `ok seedhammer.com/gui`. If a verbatim row fails, the SPEC is right and the Go string changes -- never the other way round.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposerCopy' ./gui/ 2>&1 | tail -2
git add gui/composer_copy.go gui/composer_copy_test.go
git commit -s -F - <<'MSG'
gui: the composer's 39 operator strings, verbatim from SPEC section 8 (composer S3 task A1)

One file, one table, and an AST scan that fails when a body is declared
without a row -- so section 12 item 5's four gates reach every string by
construction rather than by an author remembering.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task A2: `gui/composer_paged.go` -- the paged list `ChoiceScreen` is not

**Files:**
- Create: `gui/composer_paged.go`
- Test: `gui/composer_paged_test.go`

**Interfaces:**
- Consumes: `Clickable`, `InputTracker`, `ButtonFilter`, `Button1`/`Button2`/`Button3`/`Center`/`Up`/`Down` (`gui/gui.go:132-138`, `gui/event.go:71`); `layoutTitle` (`gui/gui.go:2342`), `layoutNavigation`/`NavButton` (`gui/gui.go:2364,2371`); `widget.Labelw`; `leadingSize` (`gui/theme.go:43`), `cornerRadius`/`buttonPadX`/`buttonPadY` (`gui/gui.go:55-57`); `assets.IconBack`/`IconRight`/`IconCheckmark`.
- Produces: `func composerPageLines(ctx *Context, th *Colors, dims image.Point, lines []string, start, sel int) ([]op.Op, int)` -- the ONE measure site, returning the ops and how many lines were actually laid out; `func composerReadScreen(ctx *Context, th *Colors, title string, lines []string) bool` -- a paged read-only screen with the pager drawn only when a second page exists; `func composerPickScreen(ctx *Context, th *Colors, title, lead string, rows []string) (int, bool)` -- a paged SELECTABLE list.

Why a new primitive and not `ChoiceScreen`: `ChoiceScreen.Draw` (`gui/gui.go:1993-2026`) stacks children with `h += c.Size.Y` and applies no clip and no bound against the content box (`320 - 2*44 = 232` px, from the two `CutTop`/`CutBottom` calls at `:1967-1970`), and `Choose` (`:1910-1964`) holds no scroll offset. Nothing exercises that today because no shipped caller has an unbounded list; a payload's key list is unbounded by construction, and the grammar admits 32 slots. `confirmReviewScreen` (`gui/multisig_build.go:1877-1939`) already solves the READ half correctly, so this file copies its measuring loop rather than inventing one, and adds selection.

Paging is FORWARD-ONLY with wrap, as `confirmReviewScreen` is, and selection never needs backward page arithmetic: Up sets `start = sel` when the cursor leaves the top of the page, Down sets `start = sel` when it leaves the bottom. Each makes the cursor the first row of the newly laid-out page, which is exact rather than a guess at the previous page's size.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_paged_test.go`:

```go
package gui

import (
	"fmt"
	"image"
	"testing"
	"testing/synctest"
)

// composerNumberedLines is a body whose every row is identifiable on a frame,
// so a paging assertion can say WHICH rows were drawn rather than how many
// characters appeared.
func composerNumberedLines(n int) []string {
	out := make([]string, n)
	for i := range out {
		out[i] = fmt.Sprintf("entry %02d marker", i)
	}
	return out
}

// TestComposerPageLinesNeverOverflowsTheContentBox is the defect this
// primitive exists for: ChoiceScreen draws an over-long list past the frame
// with no clip and no cue (gui/gui.go:1993-2026).
//
// It asserts the MEASURE, not the pixels: composerPageLines reports how many
// rows it laid out, and that count must be strictly less than a list far
// longer than any frame can hold -- which is exactly the property
// ChoiceScreen lacks.
func TestComposerPageLinesNeverOverflowsTheContentBox(t *testing.T) {
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	lines := composerNumberedLines(64)
	_, shown := composerPageLines(ctx, &descriptorTheme, sh2DisplaySize, lines, 0, -1)
	if shown <= 0 {
		t.Fatalf("composerPageLines laid out %d rows of 64; a 232 px content box holds several", shown)
	}
	if shown >= len(lines) {
		t.Fatalf("composerPageLines claims all %d rows fit one 232 px frame -- that is "+
			"the ChoiceScreen defect (gui/gui.go:1993-2026) reproduced, not fixed", shown)
	}
	t.Logf("per-frame capacity at %v with body text: %d rows", sh2DisplaySize, shown)
	// And paging from the tail must not report rows past the end.
	_, tail := composerPageLines(ctx, &descriptorTheme, sh2DisplaySize, lines, len(lines)-2, -1)
	if tail != 2 {
		t.Errorf("laying out from the second-to-last row drew %d rows, want 2", tail)
	}
}

// TestComposerPickScreenReachesARowOnASecondPage is the operator-visible half:
// a row that does not fit the first frame is still SELECTABLE, which on
// ChoiceScreen it is not (the row draws off-frame while Down still moves the
// invisible cursor onto it).
func TestComposerPickScreenReachesARowOnASecondPage(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		rows := composerNumberedLines(24)
		var got int
		var ok bool
		frame, quit := runUI(ctx, func() {
			got, ok = composerPickScreen(ctx, &descriptorTheme, "Pick", "Choose one", rows)
		})
		defer quit()
		if _, seen := pumpUntil(frame, "entry 00 marker", 8); !seen {
			t.Fatal("the first page never drew")
		}
		// Page forward once (Button2), then take the first row of that page.
		click(&ctx.Router, Button2)
		content, seen := pumpUntil(frame, "marker", 8)
		if !seen {
			t.Fatalf("the second page never drew.\nLast frame: %q", content)
		}
		if uiContains(content, "entry 00 marker") {
			t.Errorf("Button2 did not advance the page; entry 00 is still drawn.\nFrame: %q", content)
		}
		click(&ctx.Router, Button3)
		for i := 0; i < 8; i++ {
			if _, more := frame(); !more {
				break
			}
		}
		if !ok {
			t.Fatal("the pick screen returned no selection after Button3")
		}
		if got == 0 {
			t.Errorf("selecting the first row of the SECOND page returned index 0, "+
				"so paging did not move the cursor with the page (got %d)", got)
		}
	})
}

// THE INK COMPARISON IS GONE, and its removal is the fix rather than a
// tidy-up.
//
// TestComposerReadScreenDrawsThePagerOnlyWhenASecondPageExists used to live
// here and assert `longInk > shortInk` between a one-line body and a 64-line
// body. ink() counts lit pixels for the WHOLE frame (gui/raster_test.go:24),
// so the difference is dominated by roughly ten drawn body rows: the assertion
// passed if the pager was always drawn, never drawn, or drawn correctly. It
// could not fail for the reason it stated, which is the one thing a gate must
// be able to do.
//
// Its replacement is behavioural and in composer_gates_test.go:
// TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage, which fails in
// both directions -- a one-page screen that withholds its checkmark and a
// multi-page screen that does not.

var _ = image.Pt
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerPage|^TestComposerPick|^TestComposerRead' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerPageLines`, `undefined: composerPickScreen`, `undefined: composerReadScreen`.

- [ ] **Step 3: Write the primitive**

Create `gui/composer_paged.go`:

```go
package gui

import (
	"image"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// The composer's PAGED screens (SPEC §9 items 6 and 7).
//
// WHY THIS IS NOT ChoiceScreen. ChoiceScreen.Draw stacks its children with
// `h += c.Size.Y` (gui/gui.go:2019) and applies NO clip and NO bound against
// the content box, and ChoiceScreen.Choose holds no scroll offset at all
// (gui/gui.go:1910-1964). The content box is 232 px (320 minus the two
// leadingSize bands cut at gui/gui.go:1967-1970), so a list longer than that
// draws past the frame with no visual cue while Down still moves the cursor
// onto rows nobody can see. Nothing exercises that today because no shipped
// caller has an unbounded list. The composer has three: the payload's keys,
// the template's 32 slots, and eight paths plus four addresses at consent.
//
// WHY IT COPIES confirmReviewScreen. That function (gui/multisig_build.go
// :1877-1939) already measures rows against the box correctly, advances by
// the EXACT count it laid out, wraps at the end, and draws its pager only
// when a second page exists. Reimplementing the measurement would be a second
// answer to a question that must have one.
//
// PAGING IS FORWARD-ONLY WITH WRAP, and selection needs no backward page
// arithmetic: when the cursor leaves the top of the page Up sets start = sel,
// and when it leaves the bottom Down sets start = sel. Either way the cursor
// becomes the first row of the page that is then laid out, which is exact
// rather than a guess at how many rows the previous page held.

// composerPageLines lays out lines[start:] into the content box and returns
// the ops plus HOW MANY were drawn.
//
// THE ONE MEASURE SITE. Every capacity number in SPEC §13 comes from this
// function, and every paged screen below calls it, so a screen's capacity and
// the number recorded for it cannot drift apart.
//
// sel is the highlighted row's absolute index, or -1 for a read-only screen.
func composerPageLines(ctx *Context, th *Colors, dims image.Point, lines []string, start, sel int) ([]op.Op, int) {
	lineWidth := dims.X - 2*8
	contentTop := leadingSize + 8
	contentBottom := dims.Y - leadingSize
	body := make([]op.Op, 0, len(lines))
	shown := 0
	y := contentTop
	for i := start; i < len(lines); i++ {
		col := th.Text
		if i == sel {
			col = th.Background
		}
		lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, lineWidth, col, lines[i])
		// The first row is drawn even if it alone overflows: a row too tall for
		// the box is a copy defect, and dropping it would make the screen blank
		// instead of showing what is wrong.
		if i > start && y+sz.Y > contentBottom {
			break
		}
		pos := image.Pt((dims.X-sz.X)/2, y)
		if i == sel {
			bg := image.Rectangle{Max: sz}
			bg.Min.X -= buttonPadX
			bg.Max.X += buttonPadX
			bg.Min.Y -= buttonPadY
			bg.Max.Y += buttonPadY
			lbl = op.Layer(
				lbl,
				op.Compose(
					op.Color(&ctx.B, th.Text),
					op.RoundedRect2(&ctx.B, bg, cornerRadius),
				),
			)
		}
		body = append(body, lbl.Offset(pos))
		y += sz.Y + 6
		// COUNTED ONLY WHEN IT IS INSIDE THE BOX. It used to increment and
		// THEN break, so the last counted row could extend past the content
		// box -- and this count is the number §13 item 1 records as the
		// screen's per-frame capacity, so it would have recorded one more row
		// than the frame fully draws.
		if y > contentBottom {
			if shown > 0 {
				break
			}
			shown++
			break
		}
		shown++
	}
	return body, shown
}

// composerReadScreen is a paged read-only screen: Button3 continues, Button1
// goes back, Button2 pages, and the pager icon is drawn ONLY when a second
// page exists.
//
// The icon gate is confirmReviewScreen's ruling, inherited rather than
// re-argued (gui/multisig_build.go:1919-1931): a control that is present and
// inert teaches the operator that controls here may be inert, on a device
// whose other buttons cut steel.
func composerReadScreen(ctx *Context, th *Colors, title string, lines []string) bool {
	backBtn := &Clickable{Button: Button1}
	contBtn := &Clickable{Button: Button3, AltButton: Center}
	pageBtn := &Clickable{Button: Button2}
	start := 0
	// THE CHECKMARK IS WITHHELD UNTIL THE LAST PAGE HAS BEEN LAID OUT ONCE.
	//
	// §7e's consent carries the per-path lines, the key-path line, both ids
	// and receive AND change addresses 0..1 -- and the addresses are the only
	// thing that proves WHICH wallet this is. The shipped confirmReviewScreen
	// accepts Button3 on the first frame whatever page is showing, so an
	// operator could consent to a wallet whose proof was never drawn. That is
	// inherited behaviour and this is new code, so the composer's own paged
	// screen does not inherit it.
	//
	// ONCE, not every time: paging back to page 1 does not re-arm the gate.
	// The operator has seen it; making them page to the end again would be a
	// control that punishes reading.
	seenEnd := false
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return false
		}
		if seenEnd && contBtn.Clicked(ctx) {
			return true
		}
		dims := ctx.Platform.DisplaySize()
		body, shown := composerPageLines(ctx, th, dims, lines, start, -1)
		if start+shown >= len(lines) {
			seenEnd = true
		}
		if pageBtn.Clicked(ctx) {
			if start+shown < len(lines) {
				start += shown
			} else {
				start = 0
			}
			continue
		}
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)
		navs := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		if start > 0 || shown < len(lines) {
			navs = append(navs, NavButton{Clickable: pageBtn, Style: StyleSecondary, Icon: assets.IconRight})
		}
		if seenEnd {
			navs = append(navs, NavButton{Clickable: contBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		} else {
			// The Button3 handler is still DRAINED above; what is withheld is
			// the icon, so the control is never present-and-inert (the ruling
			// confirmReviewScreen states at gui/multisig_build.go:1919-1925).
			contBtn.Clicked(ctx)
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, navs...)
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return false
}

// composerPickScreen is composerReadScreen with a cursor: Up/Down move the
// selection, Button2 pages, Button3 takes the highlighted row, Button1
// declines. `lead` is drawn as the first body row rather than in the lead
// band, so a long prompt (the §8s seating prompts are long) wraps with the
// rows instead of being cut by the 44 px band.
func composerPickScreen(ctx *Context, th *Colors, title, lead string, rows []string) (int, bool) {
	backBtn := &Clickable{Button: Button1}
	takeBtn := &Clickable{Button: Button3, AltButton: Center}
	pageBtn := &Clickable{Button: Button2}
	inp := new(InputTracker)
	// THE LEAD IS A PER-PAGE HEADER, NOT THE FIRST BODY ROW.
	//
	// It used to be lines[0] with a spacer, which put the §8s prompt ("Slot
	// @2, Path 1 key 2 of 3: choose a key") on the FIRST page only. On the screen
	// this primitive exists for -- a payload holding more keys than a frame --
	// an operator paging for the right key lost which slot they were filling,
	// and paging is forward-only with wrap, so recovering it cost a full
	// cycle. Prepending it to every page costs the same measurement loop.
	lines := rows
	sel := 0
	start := 0
	for !ctx.Done {
		if backBtn.Clicked(ctx) {
			return 0, false
		}
		if takeBtn.Clicked(ctx) {
			return sel, true
		}
		dims := ctx.Platform.DisplaySize()
		page := append([]string{lead, ""}, lines[start:]...)
		const rowBase = 2 // the header and its spacer, redrawn on every page
		pageOps, drawn := composerPageLines(ctx, th, dims, page, 0, sel-start+rowBase)
		shown := drawn - rowBase
		if shown < 1 {
			// A header that fills the frame would leave no room for a row and
			// the list could never advance. One row always draws.
			shown = 1
		}
		body := pageOps
		for {
			e, ok := inp.Next(ctx, ButtonFilter(Up), ButtonFilter(Down))
			if !ok {
				break
			}
			be, ok := e.AsButton()
			if !ok || !be.Pressed {
				continue
			}
			switch be.Button {
			case Up:
				if sel > 0 {
					sel--
				}
			case Down:
				if sel < len(lines)-1 {
					sel++
				}
			}
			if sel < start || sel >= start+shown {
				// The cursor left the page. Making it the FIRST row of the next
				// layout is exact; computing the previous page's size is not.
				start = sel
			}
		}
		if pageBtn.Clicked(ctx) {
			if start+shown < len(lines) {
				start += shown
			} else {
				start = 0
			}
			// THE CURSOR IS CLAMPED INTO THE PAGE IN BOTH DIRECTIONS. Only the
			// upward clamp existed, so after a wrap to start = 0 the cursor
			// stayed on a row belonging to a later page: that frame drew NO
			// highlight and Button3 returned the invisible row -- seating a
			// key the operator never saw selected. Up/Down already clamp the
			// page to the cursor (start = sel); this is the same rule applied
			// the other way.
			if sel < start {
				sel = start
			}
			if sel >= start+shown {
				sel = start
			}
			continue
		}
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)
		navs := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		if start > 0 || shown < len(lines) {
			navs = append(navs, NavButton{Clickable: pageBtn, Style: StyleSecondary, Icon: assets.IconRight})
		}
		navs = append(navs, NavButton{Clickable: takeBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		nav, _ := layoutNavigation(&ctx.B, th, dims, navs...)
		frameOps := append([]op.Op{nav, titleOp}, body...)
		frameOps = append(frameOps, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return 0, false
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerPage|^TestComposerPick|^TestComposerRead' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL|    composer)'`
Expected: three PASS lines, plus the logged capacity (`per-frame capacity at (480,320) with body text: N rows`). **Record N** -- it is the first of the three numbers spec §13 item 1 wants, and the measurement task reads it from this log rather than inventing one.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_paged.go gui/composer_paged_test.go
git commit -s -F - <<'MSG'
gui: a paged list primitive for the composer, with one measure site (composer S3 task A2)

ChoiceScreen stacks its rows with no clip and no bound against the 232 px
content box and holds no scroll offset, so an unbounded list draws off frame
while Down still selects invisible rows. This copies confirmReviewScreen's
measuring loop and adds a cursor; every paged capacity number comes from
composerPageLines so a screen and its recorded capacity cannot drift.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
### Task A3: the admission row, the two comment rewrites, the §8e deprecation comment, and the flag-screen truth

**Files:**
- Modify (FRAGMENT, hand-wired for the gate): `gui/sysw_admit.go:47-52` (the `progWalletPolicy` row and the comment above it)
- Modify (FRAGMENT): `gui/gui.go:191-203` (the `walletPolicy` program comment)
- Modify (FRAGMENT): `gui/multisig_build.go:22` (insert the §8e deprecation comment above the T6c banner)
- Modify (FRAGMENT): `gui/sysw_admit_oracle_test.go:78-88` (`classNames` gains the three composer classes)
- Test: `gui/composer_admit_test.go`
- Modify (mnemonic-engrave, its own commit): `design/FOLLOWUPS.md`

**Interfaces:**
- Consumes: `admitted`, `admits`, `syswProgram`, `progWalletPolicy` (`gui/sysw_admit.go:19-30,32-62,66`); `sysw.ClassKey`/`ClassHash`/`ClassNow` (S2); `syswSession.load` (`gui/sysw_session.go:80`), `syswLoadWarnings` (`gui/sysw_load.go:259`), `syswFlags`/`flagSecretInPlaintext` (`gui/sysw_admit.go:112,95`).
- Produces: `progWalletPolicy` admitting eight classes; two rewritten comments; one deprecation comment; the oracle's class map extended.

**The §9 item 4 correction, stated once so the implementer does not look for work that is not there.** §6a says the F1/F2 flag screens "fire inside the composer's seed step exactly as they do in Multisig Build". They do not fire in a seed step in either program. `syswLoadWarnings` is called from exactly one place -- `syswLoadFlow` (`gui/sysw_load.go:210`) -- and `syswLoadFlow` has exactly three call sites: `gui/gui.go:2074` (boot) and `gui/sysw_unload.go:36,75` (reload). Measured: `grep -n 'syswLoadFlow(' gui/*.go` excluding tests returns those three plus the declaration. Furthermore `syswLoadWarnings` walks `s.records` and calls `syswFlags(r.class, ...)` with NO admission check at all, so a payload holding a mnemonic ALREADY raises F1 at load today, whatever the admission table says. **So there is no per-program wiring to add.** What this task delivers is the admission-row change plus a test that proves F1 fires for a composer payload and that no per-program call was introduced. §6a's sentence is folded in the measurement task at the end of this plan.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_admit_test.go`:

```go
package gui

import (
	"os"
	"strings"
	"testing"

	"seedhammer.com/sysw"
)

// TestComposerWalletPolicyAdmitsTheComposerClasses is C12, as a gate.
//
// The row used to be Descriptor + MDMK and its comment said "NO seed class
// ... least privilege". The composer AUTHORS a wallet and may fill a seat
// from a seed on this device, so the privilege the program needs changed.
// What is still refused is named too, because an admission test that only
// checks the additions cannot catch a row that admitted everything.
func TestComposerWalletPolicyAdmitsTheComposerClasses(t *testing.T) {
	for _, c := range []sysw.Class{
		sysw.ClassDescriptor, sysw.ClassMDMK,
		sysw.ClassMnemonic, sysw.ClassCodex32Secret, sysw.ClassPassphrase,
		sysw.ClassKey, sysw.ClassHash, sysw.ClassNow,
	} {
		if !admits(progWalletPolicy, c) {
			t.Errorf("progWalletPolicy refuses class %v, which SPEC §6a admits", c)
		}
	}
	for _, c := range []sysw.Class{sysw.ClassFreeText, sysw.ClassAddress, sysw.ClassUnknown} {
		if admits(progWalletPolicy, c) {
			t.Errorf("progWalletPolicy admits class %v, which SPEC §6a does not", c)
		}
	}
	// The three composer classes are admitted at Wallet Policy ALONE (§6a).
	for p := progBackupWallet; p <= progTransaction; p++ {
		if p == progWalletPolicy {
			continue
		}
		for _, c := range []sysw.Class{sysw.ClassKey, sysw.ClassHash, sysw.ClassNow} {
			if admits(p, c) {
				t.Errorf("program %d admits composer class %v; §6a admits the three at "+
					"Wallet Policy alone", p, c)
			}
		}
	}
}

// TestComposerSeedInAPayloadStillRaisesF1AtLoad is the §9 item 4 truth.
//
// The spec says the flag screens "fire inside the composer's seed step". They
// fire at LOAD, from syswLoadFlow's three call sites, and syswLoadWarnings
// consults no admission table -- so this behaviour is not created by the row
// change and is not per-program. The test pins the behaviour that IS relied
// on: a plaintext payload holding a seed raises F1, and the operator meets it
// before any program consumes anything.
func TestComposerSeedInAPayloadStillRaisesF1AtLoad(t *testing.T) {
	s := &syswSession{}
	// A payload the composer would use: a seed to seat from, and a key record.
	s.load(&sysw.Payload{
		Public: []string{composerTestKeyRecord},
		Secret: []string{composerTestMnemonicRecord},
	}, [32]byte{}, false, true, true, true)
	if !syswHasFlag(s, flagSecretInPlaintext) {
		t.Fatal("a plaintext payload holding a seed does not raise F1, so the operator " +
			"is never told a secret sits unencrypted in flash")
	}
	lines := syswLoadWarnings(s)
	if len(lines) == 0 {
		t.Fatal("syswLoadWarnings produced no line for an F1 payload")
	}
	found := false
	for _, l := range lines {
		if strings.Contains(l, "SECRET is stored unencrypted in flash") {
			found = true
		}
	}
	if !found {
		t.Errorf("the F1 warning does not name the exposure: %q", lines)
	}
}

// TestComposerAddsNoPerProgramFlagScreenCall is the other half: the row
// change must not have grown a second place where the flags are shown.
//
// A negative inherits the scope of the search that produced it, so the scope
// is named: every non-test .go file in gui/, and the control below proves the
// query finds the calls that DO exist.
func TestComposerAddsNoPerProgramFlagScreenCall(t *testing.T) {
	ents, err := os.ReadDir(".")
	if err != nil {
		t.Fatal(err)
	}
	callers := map[string]int{}
	for _, e := range ents {
		name := e.Name()
		if e.IsDir() || !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		b, err := os.ReadFile(name)
		if err != nil {
			t.Fatal(err)
		}
		if n := strings.Count(string(b), "syswLoadWarnings("); n > 0 {
			callers[name] += n
		}
	}
	// The CONTROL: the declaration and its one caller both live in
	// sysw_load.go. A query returning nothing everywhere would pass a
	// "no new callers" assertion for the wrong reason.
	if callers["sysw_load.go"] < 2 {
		t.Fatalf("INCONCLUSIVE: the query found %d mentions in sysw_load.go, where the "+
			"declaration and its one call both live -- the search is broken, not the tree",
			callers["sysw_load.go"])
	}
	delete(callers, "sysw_load.go")
	if len(callers) != 0 {
		t.Errorf("syswLoadWarnings gained per-program callers %v. The flags are a LOAD-time "+
			"mechanism (gui/sysw_load.go:210); a second site would show them twice and "+
			"put the admission table's classes behind two different rules", callers)
	}
}

// TestComposerMultisigBuildCarriesTheDeprecationComment is C7, whose whole
// deliverable is a comment. A comment-only deliverable with no gate is a
// deliverable nobody can tell was made.
func TestComposerMultisigBuildCarriesTheDeprecationComment(t *testing.T) {
	b, err := os.ReadFile("multisig_build.go")
	if err != nil {
		t.Fatal(err)
	}
	src := string(b)
	for _, want := range []string{
		"Deprecated 2026-09-01 in favour of Wallet Policy",
		"Build a new policy",
		"No enforcement by operator ruling",
	} {
		if !strings.Contains(src, want) {
			t.Errorf("gui/multisig_build.go does not carry %q -- SPEC §8e's whole "+
				"deliverable is this comment", want)
		}
	}
	// And it is a DEPRECATION, not a removal: the flow is still reachable.
	if !strings.Contains(src, "func buildMultisigPolicyFlow(") {
		t.Error("buildMultisigPolicyFlow is gone; C7 is comment-only, with no enforcement")
	}
}

// TestComposerWalletPolicyProgramCommentNoLongerSaysOutsideOnly pins the §6a
// rewrite. The comment argued the program's identity from a premise C12
// retires; leaving it standing is how a stale premise outlives its condition.
func TestComposerWalletPolicyProgramCommentNoLongerSaysOutsideOnly(t *testing.T) {
	b, err := os.ReadFile("gui.go")
	if err != nil {
		t.Fatal(err)
	}
	src := string(b)
	for _, gone := range []string{
		"would drag a seed requirement or a plate census into a flow that needs neither",
	} {
		if strings.Contains(src, gone) {
			t.Errorf("gui.go still says %q. The composer DOES take a seed and DOES cut a "+
				"census inside this program, so the sentence is now false", gone)
		}
	}
	for _, want := range []string{"Build a new policy", "AUTHOR"} {
		if !strings.Contains(src, want) {
			t.Errorf("the walletPolicy program comment does not mention %q", want)
		}
	}
}

// TestComposerAdmitCommentNoLongerClaimsNoSeedClass is the sysw_admit.go half.
func TestComposerAdmitCommentNoLongerClaimsNoSeedClass(t *testing.T) {
	b, err := os.ReadFile("sysw_admit.go")
	if err != nil {
		t.Fatal(err)
	}
	// THE PHRASE ALONE IS NOT THE ASSERTION, and this is the second attempt at
	// it. progTransaction's row legitimately says "NO seed class and no
	// passphrase" (gui/sysw_admit.go:65) and is not touched by this cycle, so
	// scanning the file for "NO seed class" fails whatever the fold does --
	// and the rewritten comment QUOTES its own old wording, so it would fail
	// twice over. The claim retired is a whole sentence, unique to this row.
	if strings.Contains(string(b), "The Wallet Policy program never derives from a secret") {
		t.Error("sysw_admit.go still claims the Wallet Policy program never derives from " +
			"a secret, above a row that now admits Mnemonic, Cdx32 and Passphrase")
	}
	if !strings.Contains(string(b), "C12") {
		t.Error("the rewritten row does not cite C12, the ruling that reverses it")
	}
	// The CONTROL: the phrase that must SURVIVE, so this test cannot pass by
	// the file having been emptied.
	if !strings.Contains(string(b), "progTransaction") {
		t.Fatal("INCONCLUSIVE: sysw_admit.go no longer names progTransaction; the file " +
			"this test reads is not the admission table")
	}
}
```

- [ ] **Step 1a: the shared payload fixtures every composer test reads**

Create `gui/composer_fixtures_test.go`. It is written HERE, in the first task that needs it, so that Part A carries no forward reference into Part B.

```go
package gui

import (
	"encoding/hex"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
	"seedhammer.com/sysw"
)

// The composer's payload fixtures: the smallest records each class has, in
// the wire form `me sysw pack` writes (SPEC §6a: a reserved prefix and a
// lowercase-hex body).
//
// NOTHING SECRET IS COMMITTED. The mnemonic is BIP-39's published "abandon"
// vector, the same one the Rust compose corpus uses; the key records are
// derived from it. The key record's shape is the host's own worked example
// (crates/me-cli/src/sysw/composer_records.rs:284).
//
// THEY ARE BUILT BY THE SAME ENCODING RULE THE HOST APPLIES, not pasted as
// opaque hex, so a reader can see what each record says. The lockstep that
// the DEVICE agrees with the HOST about these bytes is S2's, gated by
// sysw/composer_records_test.go against the vendored 45-row fixture; these
// are for driving screens.
func composerRecord(prefix, text string) string {
	return prefix + hex.EncodeToString([]byte(text))
}

var (
	composerTestKeyRecord  = composerRecord("key:", "[73c5da0a/48'/0'/0'/2']"+composerTestXpubA)
	composerTestKeyRecord2 = composerRecord("key:", "[73c5da0a/48'/0'/1'/2']"+composerTestXpubB)
	composerTestHashRecord = "hash:" + strings.Repeat("ab", 32)
	// 1788220800 is 2026-09-01 00:00:00 UTC, measured, not transcribed.
	composerTestNowRecord = composerRecord("now:", "1788220800,905000")
	// A seed record is the mnemonic itself: ClassMnemonic is SNIFFED, not
	// prefixed (sysw/record.go's classifyConstellation), so no encoding here.
	composerTestMnemonicRecord = "abandon abandon abandon abandon abandon abandon " +
		"abandon abandon abandon abandon abandon about"
	composerTestDescriptorRecord = composerTestDescriptor
)

// The three constants the fixtures above are built from, MEASURED rather than
// transcribed. Each carries the command that produced it, and the commands
// are the plan's, run at plan time -- so a reader can re-run them instead of
// trusting this file.
//
//	ABANDON="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
//	~/.cargo/bin/ms derive --template bip48-p2wsh --account 0 --phrase "$ABANDON"
//	~/.cargo/bin/ms derive --template bip48-p2wsh --account 1 --phrase "$ABANDON"
//
// Both report master_fingerprint 73c5da0a, at m/48'/0'/0'/2' and
// m/48'/0'/1'/2': ONE master at TWO accounts, which is C5's normal case and
// exactly what the door's "two keys share a fingerprint" row and the mapping
// review's label rule need a fixture for. INVOKE ms BY PATH: in this
// operator's shell a bare `md` is aliased to `mkdir -p`, which exits 0 and
// creates a directory, and a fixture step that reports success while
// producing nothing is the failure this rule exists to prevent.
const (
	composerTestXpubA = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"
	composerTestXpubB = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"

	// The record the SHIPPED walk already pins, read out of
	// gui/testdata/s2_descriptor_payload.bin (the container `me sysw pack
	// --as descriptor` wrote; gui/wallet_policy_descriptor_walk_test.go:63-93
	// opens it through the firmware's own sysw.Open). Reused rather than
	// minted a second time, so the composer's door tests and the shipped
	// descriptor walk agree about what a Descriptor record is.
	composerTestDescriptor = "wsh(sortedmulti(2," +
		"[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*," +
		"[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*," +
		"[c5d87297/48h/0h/0h/2h]xpub6DjrnfAyuonMaboEb3ZQZzhQ2ZEgaKV2r64BFmqymZqJqviLTe1JzMr2X2RfQF892RH7MyYUbcy77R7pPu1P71xoj8cDUMNhAMGYzKR4noZ/<0;1>/*))#ud8uyjz3"
)

// composerTestPath is a distinct origin per index, for the pick-list
// measurement: 32 rows that are not all the same width.
func composerTestPath(i int) bip32.Path {
	const h = hdkeychain.HardenedKeyStart
	return bip32.Path{48 | h, 0 | h, uint32(i) | h, 2 | h}
}

// composerTestOrigin is §4f's origin for a wrapper and account, built through
// md.DefaultOrigin so the fixture and the production table cannot disagree.
func composerTestOrigin(scriptType, account uint32) []md.PathComponent {
	w := md.ComposeWsh
	switch scriptType {
	case 1:
		w = md.ComposeShWsh
	case 3:
		w = md.ComposeTr
	}
	return md.DefaultOrigin(w, account)
}

// composerPayloadWith wraps records as the session's load takes them.
func composerPayloadWith(public, secret []string) *sysw.Payload {
	return &sysw.Payload{Public: public, Secret: secret}
}

// composerTwoPathList is the four-slot shape most seating tests use: a 2-of-3
// then a single key, under wsh, so slots @0..@2 are path 1 and @3 is path 2.
func composerTwoPathList() md.PathList {
	return md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 1}},
	}}
}

// composerTestMnemonic is the published "abandon" vector as a bip39.Mnemonic.
func composerTestMnemonic(t *testing.T) bip39.Mnemonic {
	t.Helper()
	m, err := bip39.ParseMnemonic(composerTestMnemonicRecord)
	if err != nil {
		t.Fatalf("the fixture mnemonic does not parse: %v", err)
	}
	return m
}

func composerMainNet() *chaincfg.Params { return &chaincfg.MainNetParams }

// TestComposerFixturesClassifyAsTheClassesTheyClaim is the control every
// composer screen test stands on. A fixture that classifies as ClassUnknown
// would make a "the door shows no keys" assertion pass for the wrong reason.
func TestComposerFixturesClassifyAsTheClassesTheyClaim(t *testing.T) {
	for _, tc := range []struct {
		name   string
		record string
		want   sysw.Class
	}{
		{"key", composerTestKeyRecord, sysw.ClassKey},
		{"key 2", composerTestKeyRecord2, sysw.ClassKey},
		{"hash", composerTestHashRecord, sysw.ClassHash},
		{"now", composerTestNowRecord, sysw.ClassNow},
		{"mnemonic", composerTestMnemonicRecord, sysw.ClassMnemonic},
		{"descriptor", composerTestDescriptorRecord, sysw.ClassDescriptor},
		{"malformed key", "key:zz", sysw.ClassUnknown},
		{"malformed hash", "hash:00", sysw.ClassUnknown},
	} {
		if got := sysw.Classify(tc.record); got != tc.want {
			t.Errorf("%s: Classify = %v, want %v -- every test that reads this fixture "+
				"is measuring something else until this passes", tc.name, got, tc.want)
		}
	}
}
```

**The three constants are already measured**, at plan time, by the commands in their own comment; the implementer re-runs them to confirm rather than filling a blank:

```bash
ABANDON="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
~/.cargo/bin/ms derive --template bip48-p2wsh --account 0 --phrase "$ABANDON"
~/.cargo/bin/ms derive --template bip48-p2wsh --account 1 --phrase "$ABANDON"
strings /scratch/code/shibboleth/seedhammer/gui/testdata/s2_descriptor_payload.bin | grep sortedmulti
```
Expected: `master_fingerprint: 73c5da0a` on both, with `account_xpub` matching `composerTestXpubA` at `m/48'/0'/0'/2'` and `composerTestXpubB` at `m/48'/0'/1'/2'`; the third command prints the descriptor `composerTestDescriptor` carries, ending `#ud8uyjz3`. **If any differs, the constant is wrong and the constant changes** -- these are measurements, not choices.

**Invoke the tools BY PATH.** In this operator's shell `md` is aliased to `mkdir -p`, which exits 0 and creates a directory, so a bare `md` in a fixture step reports success while producing nothing. `ms derive --help` at `ms` 0.16.0 lists `bip44`, `bip49`, `bip84`, `bip86`, `bip48-p2wsh`, `bip48-p2sh-p2wsh`, `bip48`; S1 adds `bip48-p2tr`, and if it is absent the S1 precondition is unmet.

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerWalletPolicyAdmits|^TestComposerMultisigBuildCarries|^TestComposerAdmitComment' ./gui/ 2>&1 | tail -8`
Expected: FAIL. `progWalletPolicy refuses class ...` for the six new classes (or a build failure on `sysw.ClassKey` if S2 has not merged -- in which case STOP, the precondition is unmet); `gui/multisig_build.go does not carry "Deprecated 2026-09-01 ..."`; `sysw_admit.go still says "NO seed class"`.

- [ ] **Step 3: Replace the admission row and its comment**

In `gui/sysw_admit.go`, replace lines 47-52:

```go
	// NO seed class. The Wallet Policy program never derives from a secret: its
	// proof is addresses derived from the policy's OWN public keys plus a named
	// wallet id, so admitting a mnemonic would grant a capability the flow has
	// no use for. Least privilege, and it is enforced here rather than by the
	// flow declining to ask.
	progWalletPolicy: {sysw.ClassDescriptor: true, sysw.ClassMDMK: true},
```

with:

```go
	// SEED CLASSES ADMITTED SINCE THE COMPOSER (C12, SPEC_wallet_policy_composer
	// §6a). This row said "NO seed class ... least privilege", and that was
	// right for as long as the program only PROVED a wallet policy built
	// elsewhere: its proof is addresses derived from the policy's own public
	// keys plus a named wallet id, so a mnemonic granted a capability the flow
	// had no use for.
	//
	// C12 reverses it deliberately. "Build a new policy" AUTHORS a wallet here,
	// and a slot may be filled from a seed this device holds, so Mnemonic,
	// Cdx32 and Passphrase are now capabilities the flow USES. Key, Hash and Now
	// are the composer's own three record classes (§6a) and are admitted at this
	// program ALONE.
	//
	// Least privilege did not stop applying; the privilege the program needs
	// changed. FreeText and Address are still refused, and that is what the
	// admission test checks alongside the additions -- a row that admitted
	// everything would pass an additions-only assertion.
	progWalletPolicy: {
		sysw.ClassDescriptor:    true,
		sysw.ClassMDMK:          true,
		sysw.ClassMnemonic:      true,
		sysw.ClassCodex32Secret: true,
		sysw.ClassPassphrase:    true,
		sysw.ClassKey:           true,
		sysw.ClassHash:          true,
		sysw.ClassNow:           true,
	},
```

- [ ] **Step 4: Rewrite the program comment**

In `gui/gui.go`, replace lines 191-203:

```go
	// walletPolicy is the 10th navigable program (plan D5): a front door for a
	// wallet policy that came from OUTSIDE this device. It is not a rename of
	// Multisig and not an extension of Bundle.
	//
	// WHY IT IS ITS OWN PROGRAM. Engrave Bundle can already gather and engrave a
	// supplied md1 — what it cannot do is PROVE it. Its review screen reads
	// "N cards verified" plus a per-card label, which says the chunks reassembled
	// and nothing about which wallet the operator is about to commit to steel.
	// Engrave Multisig proves more, but demands a seed and cuts COSIGNER plates:
	// its question is "am I in this policy", not "is this the right policy".
	// Neither answers plan D2 — proof is derived addresses plus a NAMED wallet id
	// — and bolting that onto either would drag a seed requirement or a plate
	// census into a flow that needs neither.
```

with:

```go
	// walletPolicy is the 10th navigable program (plan D5), and since the
	// composer it has TWO doors: prove a wallet policy that came from OUTSIDE
	// this device, or "Build a new policy" and AUTHOR one here. It is still not
	// a rename of Multisig and still not an extension of Bundle.
	//
	// WHY IT IS ITS OWN PROGRAM. Engrave Bundle can already gather and engrave a
	// supplied md1; what it cannot do is PROVE it. Its review screen reads
	// "N cards verified" plus a per-card label, which says the chunks reassembled
	// and nothing about which wallet the operator is about to commit to steel.
	// Engrave Multisig proves more, but its question is "am I in this policy",
	// not "is this the right policy", and it can author exactly one shape: a
	// sortedmulti k-of-n under wsh, sh(wsh) or sh, from a seed.
	//
	// THE OLD SENTENCE HERE IS RETIRED, NOT MOVED. It said that bolting proof
	// onto either program "would drag a seed requirement or a plate census into
	// a flow that needs neither". This program now takes a seed and cuts a plate
	// census itself, by C12 and C10, so the sentence became false the day the
	// composer landed. A stale premise left standing under a heading that reads
	// as reasoning is the defect this rewrite exists to avoid, not a wording
	// preference.
	//
	// What it authors is what Multisig Build cannot: arbitrary tr and wsh
	// policies as an ordered list of spend paths, with locks, hashlocks and
	// key-less paths (SPEC_wallet_policy_composer.md). C7 deprecates Multisig
	// Build in its favour BY COMMENT ONLY, with no enforcement.
```

- [ ] **Step 5: Add the §8e deprecation comment**

In `gui/multisig_build.go`, insert immediately above line 22's banner (`// ─── T6c Phase B: the on-device "Build policy" authoring path ───`):

```go
// DEPRECATED.
//
// Deprecated 2026-09-01 in favour of Wallet Policy > Build a new policy.
// No enforcement by operator ruling (C7, SPEC_wallet_policy_composer.md §8e).
//
// THE SENTENCE ABOVE IS UNWRAPPED ON PURPOSE: it is §8e verbatim and
// TestComposerMultisigBuildCarriesTheDeprecationComment matches substrings of
// it, so a comment re-flow that splits "Build a new policy" across two lines
// turns the suite red for a wording change. Reflow the paragraphs below it
// freely; leave those two lines alone.
//
// This flow keeps working, keeps its tests, and is not gated, redirected or
// removed. What the composer does that this cannot is author anything other
// than one sortedmulti k-of-n -- taproot, several spend paths, timelocks,
// hashlocks, key-less paths.
//
// A comment with no enforcement is the whole of the ruling. F-150 item 1's
// dead end stays as filed and is not fixed here.
```

- [ ] **Step 6: Extend the consumption-site oracle's class map**

In `gui/sysw_admit_oracle_test.go`, replace the `classNames` map body (lines 80-88's entries) so the three composer classes resolve. Without this, a composer site naming `sysw.ClassKey` is reported as "consumes from the payload without naming a sysw.Class constant" -- a true-looking failure with a false cause.

```go
var classNames = map[string]sysw.Class{
	"ClassMnemonic":      sysw.ClassMnemonic,
	"ClassCodex32Secret": sysw.ClassCodex32Secret,
	"ClassPassphrase":    sysw.ClassPassphrase,
	"ClassFreeText":      sysw.ClassFreeText,
	"ClassDescriptor":    sysw.ClassDescriptor,
	"ClassMDMK":          sysw.ClassMDMK,
	"ClassAddress":       sysw.ClassAddress,
	"ClassUnknown":       sysw.ClassUnknown,
	// The composer's three (SPEC_wallet_policy_composer §6a), admitted at
	// progWalletPolicy alone. A site naming one of these without an entry here
	// is reported as "names no sysw.Class constant", which is a true failure
	// with a false cause -- the worst kind to debug.
	"ClassKey":  sysw.ClassKey,
	"ClassHash": sysw.ClassHash,
	"ClassNow":  sysw.ClassNow,
}
```

- [ ] **Step 7: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposer|^TestEverySyswConsumptionSite|^TestTheSeamPassphraseOffer' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: every `TestComposer*` PASS; `TestEverySyswConsumptionSiteNamesAnAdmittedClass` still PASS (no consumption site has been added yet, so its site count is unchanged); `ok seedhammer.com/gui`.

- [ ] **Step 8: The FOLLOWUPS entry, in mnemonic-engrave**

§8e's deliverable is "a COMMENT in `gui/multisig_build.go` and a FOLLOWUPS entry only". Append to `design/FOLLOWUPS.md` in `/scratch/code/shibboleth/mnemonic-engrave`:

```text
### F-454 — `multisig-build-deprecated-in-favour-of-the-composer`: Engrave Multisig's "Build policy" path is deprecated by comment, with no enforcement (owning phase: **none; a record, not a task**) `#seedhammer` `#composer` `#c7`

Deprecated 2026-09-01 in favour of Wallet Policy > Build a new policy. No
enforcement by operator ruling (C7). The comment lives at the head of
`gui/multisig_build.go` and is gated by
`TestComposerMultisigBuildCarriesTheDeprecationComment`, which also asserts
`buildMultisigPolicyFlow` still exists -- a deprecation, not a removal.
Removing or redirecting Multisig Build is out of scope
(`SPEC_wallet_policy_composer.md` §14), and F-150 item 1's dead end stays as
filed.
```

- [ ] **Step 9: gofmt, commit (two repos, two commits)**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -2
git add gui/sysw_admit.go gui/gui.go gui/multisig_build.go gui/sysw_admit_oracle_test.go gui/composer_admit_test.go
git commit -s -F - <<'MSG'
gui: admit the composer's classes at Wallet Policy, and retire the two comments C12 falsifies (composer S3 task A3)

progWalletPolicy admits Mnemonic, Cdx32, Passphrase, Key, Hash and Now beside
Descriptor and MDMK; FreeText and Address stay refused and the test checks
both directions. The flag screens needed no wiring: syswLoadWarnings is
load-time, consults no admission table, and already fires for a seed in a
payload -- a test pins that and asserts no per-program caller was added.
Multisig Build carries C7's deprecation comment, with no enforcement.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

Then, in `/scratch/code/shibboleth/mnemonic-engrave`:

```bash
git add design/FOLLOWUPS.md
git commit -s -F - <<'MSG'
followups: F-454 records C7's comment-only deprecation of Multisig Build

Section 8e's second deliverable. The comment itself and its gate ship in the
fork; this is the record the spec asks for.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task A4: `gui/composer_door.go` -- the door ChoiceScreen with its §8r key-state lines

**Files:**
- Create: `gui/composer_door.go`
- Test: `gui/composer_door_test.go`
- Modify (FRAGMENT): `gui/wallet_policy.go:35-97` (the flow opens on the door in every state)

**Interfaces:**
- Consumes: `ChoiceScreen` (`gui/gui.go:1884`), `syswSession.has`/`records` (`gui/sysw_session.go:180,53`), `sysw.ClassKey`/`ClassMnemonic`/`ClassCodex32Secret`/`ClassDescriptor`/`ClassMDMK`/`ClassUnknown`, `ctx.Platform.SyswReader()` (`gui/gui.go:3529`, `sysw/read.go:10` `Probe`), `composerCopy*` (Task A1), `showError`.
- Produces: `type composerRoute int` (`composerRouteScan`, `composerRouteFromPayload`, `composerRouteBuild`); `func composerDoorCounts(s *syswSession) (keys, seeds, inert int)`; `func composerDoorLines(s *syswSession, payloadInFlash bool) []string`; `func composerDoorFlow(ctx *Context, th *Colors) (composerRoute, bool)`.

Today there is NO door. `walletPolicyFlow` (`gui/wallet_policy.go:35`) offers `ClassMDMK` cards through `syswOfferCards` (`:44`), then `ClassDescriptor` through `syswOfferAlt` (`:46`), and when the payload holds neither it falls straight through to the NFC gather at `:97` with **no screen at all**. §7a makes the door a `ChoiceScreen` in EVERY state, and the choices name the route they take (F-437, already the ruling behind `syswAltScan` at `gui/sysw_session.go:233`).

**The key state is computed WITHOUT the `compared` gate, deliberately.** `syswSession.has` (`gui/sysw_session.go:180`) exists exactly for this -- "so a menu can offer `from payload` before the operator has compared anything" -- while `take`/`takeAll` refuse until `compared`. The door COUNTS; it consumes nothing. Seating consumes, and inherits the gate.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_door_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"

	"seedhammer.com/sysw"
)

// composerSessionWith builds a loaded, compared session from raw records.
// compared=true because the door's own key-state lines read through `has`,
// which has no compared gate, while everything that CONSUMES a record
// inherits `take`'s (gui/sysw_session.go:118-124).
func composerSessionWith(public, secret []string) *syswSession {
	s := &syswSession{}
	s.load(&sysw.Payload{Public: public, Secret: secret}, [32]byte{}, false, true, true, true)
	return s
}

func TestComposerDoorLinesCoverEveryKeyState(t *testing.T) {
	for _, tc := range []struct {
		name     string
		session  *syswSession
		inFlash  bool
		want     string
		unwanted string
	}{
		{"keys only", composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil), false,
			"Keys loaded: 2", "plus"},
		{"keys and one seed", composerSessionWith([]string{composerTestKeyRecord}, []string{composerTestMnemonicRecord}), false,
			"Keys loaded: 1, plus 1 seed.", ""},
		{"seed only", composerSessionWith(nil, []string{composerTestMnemonicRecord}), false,
			"A seed is loaded. It can fill any number of slots.", "Keys loaded"},
		{"nothing", composerSessionWith(nil, nil), false,
			"No keys loaded. This builds a key-less template.", "Keys loaded:"},
		{"nothing loaded but flash holds one", nil, true,
			"A payload is in flash but not loaded.", "Keys loaded:"},
		{"inert records", composerSessionWith([]string{composerTestKeyRecord, "hash:zz"}, nil), false,
			"1 payload record was not understood.", ""},
	} {
		t.Run(tc.name, func(t *testing.T) {
			got := strings.Join(composerDoorLines(tc.session, tc.inFlash), " | ")
			if !strings.Contains(got, tc.want) {
				t.Errorf("the door does not say %q.\nLines: %s", tc.want, got)
			}
			if tc.unwanted != "" && strings.Contains(got, tc.unwanted) {
				t.Errorf("the door says %q, which this state must not print.\nLines: %s", tc.unwanted, got)
			}
		})
	}
}

// TestComposerDoorCountsIgnoreClassesThatAreNotKeys guards the count itself:
// a card, a descriptor and a now: record are none of them "keys loaded", and
// a malformed key: reduces the count while raising the inert one (§6a).
func TestComposerDoorCountsIgnoreClassesThatAreNotKeys(t *testing.T) {
	s := composerSessionWith([]string{
		composerTestKeyRecord,        // ClassKey
		composerTestNowRecord,        // ClassNow
		"key:zz",                     // malformed -> ClassUnknown
		composerTestDescriptorRecord, // ClassDescriptor
	}, nil)
	keys, seeds, inert := composerDoorCounts(s)
	if keys != 1 {
		t.Errorf("keys = %d, want 1: only the well-formed key: record is a key", keys)
	}
	if seeds != 0 {
		t.Errorf("seeds = %d, want 0", seeds)
	}
	if inert != 1 {
		t.Errorf("inert = %d, want 1: the malformed key: record goes inert and is counted "+
			"once, in the not-understood line (§6a)", inert)
	}
}

// TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne is §7a's
// conditional choice: "From payload" appears only when the loaded payload
// holds a Descriptor or an md1/mk1 record.
func TestComposerDoorOffersFromPayloadOnlyWhenThePayloadHasOne(t *testing.T) {
	for _, tc := range []struct {
		name    string
		session *syswSession
		want    bool
	}{
		{"key records only", composerSessionWith([]string{composerTestKeyRecord}, nil), false},
		{"a descriptor", composerSessionWith([]string{composerTestDescriptorRecord}, nil), true},
	} {
		t.Run(tc.name, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				ctx.sysw = tc.session
				frame, quit := runUI(ctx, func() { composerDoorFlow(ctx, &descriptorTheme) })
				defer quit()
				content, ok := pumpUntil(frame, "Build a new policy", 16)
				if !ok {
					t.Fatalf("the door never drew.\nLast frame: %q", content)
				}
				if got := uiContains(content, "From payload"); got != tc.want {
					t.Errorf("From payload offered = %v, want %v.\nFrame: %q", got, tc.want, content)
				}
				if !uiContains(content, "Scan cards") {
					t.Errorf("the door does not offer the NFC route.\nFrame: %q", content)
				}
			})
		})
	}
}

// TestComposerDoorDrawsItsKeyStateAndFitsItsLabels is §12 item 5 for this
// screen: it draws (raster floor) and no choice label is cut off the panel.
func TestComposerDoorDrawsItsKeyStateAndFitsItsLabels(t *testing.T) {
	for _, l := range []string{"Scan cards", "From payload", "Build a new policy"} {
		assertChoiceLabelFits(t, l)
	}
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		ctx.sysw = composerSessionWith([]string{composerTestKeyRecord}, []string{composerTestMnemonicRecord})
		frame, _, ink, quit := runUITouchRaster(ctx, func() { composerDoorFlow(ctx, &descriptorTheme) })
		defer quit()
		content, ok := pumpUntil(frame, "Build a new policy", 16)
		if !ok {
			t.Fatalf("the door never drew.\nLast frame: %q", content)
		}
		assertFrameHasBody(t, ink(), "the composer door")
		if !uiContains(content, "Keys loaded: 1, plus 1 seed.") {
			t.Errorf("the door does not draw its key state.\nFrame: %q", content)
		}
	})
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDoor' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerDoorLines`, `undefined: composerDoorCounts`, `undefined: composerDoorFlow`.

- [ ] **Step 3: Write the door**

Create `gui/composer_door.go`:

```go
package gui

import (
	"seedhammer.com/sysw"
)

// The Wallet Policy door (SPEC_wallet_policy_composer.md §7a, C6).
//
// BEFORE THIS THERE WAS NO DOOR. walletPolicyFlow offered the payload's md1
// cards (gui/wallet_policy.go:44), then a Descriptor record (:46), and when
// the payload held neither it fell through to the NFC gather at :97 with no
// screen at all -- so an operator with an empty machine met a wait, not a
// choice. §7a makes it a ChoiceScreen in EVERY state, and each choice NAMES
// the route it takes (F-437, the same ruling behind syswAltScan).
//
// THE KEY STATE IS COUNTED WITHOUT THE `compared` GATE, ON PURPOSE.
// syswSession.has exists for exactly this and says so (gui/sysw_session.go
// :178-180): a menu may offer "from payload" before the operator has compared
// anything. The door COUNTS; it consumes nothing. Seating consumes, through
// take/takeAll, and inherits their refusal.

type composerRoute int

const (
	composerRouteScan composerRoute = iota
	composerRouteFromPayload
	composerRouteBuild
)

// composerDoorCounts reports what the loaded payload holds, for §8r.
//
// `inert` is the not-understood count: records the classifier placed in
// ClassUnknown, which under the shipped contract stay in the session, are
// offered to nobody and reach no screen (sysw/descriptor.go:46-48). It is
// the ONE line that covers all three composer classes' malformations, since
// a bad hash: or now: changes no other count (§6a).
func composerDoorCounts(s *syswSession) (keys, seeds, inert int) {
	if s == nil || !s.loaded {
		return 0, 0, 0
	}
	for _, r := range s.records {
		switch r.class {
		case sysw.ClassKey:
			keys++
		case sysw.ClassMnemonic, sysw.ClassCodex32Secret:
			seeds++
		case sysw.ClassUnknown:
			inert++
		}
	}
	return keys, seeds, inert
}

// composerDoorLines is §8r, in §7a's order.
//
// A SEED PRINTS NO COUNT OF SLOTS and a seeds-only payload prints no key
// count: a seed fills any number of slots (C12, §4f), so a slot number beside
// it would answer a question the operator is not asking.
func composerDoorLines(s *syswSession, payloadInFlash bool) []string {
	if s == nil || !s.loaded {
		if payloadInFlash {
			return []string{composerCopyPayloadNotLoaded()}
		}
		return []string{composerCopyNoKeys()}
	}
	keys, seeds, inert := composerDoorCounts(s)
	var lines []string
	switch {
	case keys > 0 && seeds > 0:
		lines = append(lines, composerCopyKeysAndSeeds(keys, seeds))
	case keys > 0:
		lines = append(lines, composerCopyKeysLoaded(keys))
	case seeds > 0:
		lines = append(lines, composerCopySeedOnly())
	default:
		lines = append(lines, composerCopyNoKeys())
	}
	if inert > 0 {
		lines = append(lines, composerCopyNotUnderstood(inert))
	}
	return lines
}

// composerDoorHasConsumablePolicy reports whether "From payload" has anywhere
// to go: a Descriptor record, or an md1/mk1 chunk set.
func composerDoorHasConsumablePolicy(s *syswSession) bool {
	if s == nil {
		return false
	}
	return s.has(sysw.ClassDescriptor) || s.has(sysw.ClassMDMK)
}

// composerDoorFlow draws the door and reports the chosen route.
//
// The lead carries §8r's key-state lines. ChoiceScreen's Lead is drawn with
// widget.Labelw (gui/gui.go:1969) so it WRAPS, which the choice rows do not
// -- which is why the state is a lead and the routes are rows.
func composerDoorFlow(ctx *Context, th *Colors) (composerRoute, bool) {
	inFlash := false
	if r := ctx.Platform.SyswReader(); r != nil && r.Probe() {
		inFlash = true
	}
	lead := ""
	for i, l := range composerDoorLines(ctx.sysw, inFlash) {
		if i > 0 {
			lead += " "
		}
		lead += l
	}
	choices := []string{"Scan cards"}
	routes := []composerRoute{composerRouteScan}
	if composerDoorHasConsumablePolicy(ctx.sysw) {
		choices = append(choices, "From payload")
		routes = append(routes, composerRouteFromPayload)
	}
	choices = append(choices, "Build a new policy")
	routes = append(routes, composerRouteBuild)

	cs := &ChoiceScreen{Title: "Wallet Policy", Lead: lead, Choices: choices}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return composerRouteScan, false
	}
	return routes[sel], true
}
```

- [ ] **Step 4: Wire the door into `walletPolicyFlow`**

In `gui/wallet_policy.go`, replace the head of the flow at lines 35-46 (from `func walletPolicyFlow` down to and including the `} else if body, ok := syswOfferAlt(...)` line's opening) so the door runs first and dispatches. The offers below are unchanged: `composerRouteFromPayload` runs them, `composerRouteScan` skips them and falls to the gather, and `composerRouteBuild` calls the composer flow and returns.

```go
// walletPolicyFlow is the walletPolicy program front door.
func walletPolicyFlow(ctx *Context, th *Colors) {
	const title = "Wallet Policy"
	// THE DOOR IS A SCREEN IN EVERY STATE SINCE THE COMPOSER (§7a, C6).
	// Before it, a payload holding neither a card nor a descriptor fell
	// straight through to the NFC gather below with no screen at all, so an
	// operator with an empty machine met a wait instead of a choice. Each
	// choice names the route it takes, which is F-437's ruling applied to the
	// door rather than to one offer inside it.
	route, ok := composerDoorFlow(ctx, th)
	if !ok {
		return
	}
	if route == composerRouteBuild {
		composerFlow(ctx, th)
		return
	}
	if route == composerRouteFromPayload {
		// The payload's cards are offered ONCE, before gathering, through the SAME
		// offer() a scanned card enters by -- bundleFlow does it this way and for
		// the reason stated there: a separate insertion path would be a second way
		// for a card to join the set, and only one of them would have the checks.
		//
		// EVERY md1/mk1 RECORD, not the first (F-76): a card is a chunk SET, and one
		// record of it completes nothing.
		if bodies, ok := syswOfferCards(ctx, th, sysw.ClassMDMK, "Cards from where?"); ok {
			ctx.syswBundleSeeds = bodies
		} else if body, ok := syswOfferAlt(ctx, th, sysw.ClassDescriptor, "Input",
			"Wallet policy from where?", syswAltScan); ok {
```

The remainder of the `else if` block (`gui/wallet_policy.go:47-96`, the descriptor comment and `descriptorFlow` call) and everything from `var gathered []bundleCard` (`:97`) down is untouched, except that the `else if` body's closing brace now also closes the `if route == composerRouteFromPayload` block.

`composerFlow` is declared in the flow task at the end of Part A; until then this fragment does not compile, which is why the door's own tests call `composerDoorFlow` directly and this wiring lands with that task. **Order the work so the wiring edit is made in the flow task, not here.**

- [ ] **Step 5: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDoor' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: four PASS (with six sub-tests under the key-state test and two under the from-payload test); `ok seedhammer.com/gui`.

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_door.go gui/composer_door_test.go
git commit -s -F - <<'MSG'
gui: the Wallet Policy door, in every state, with its key-state lines (composer S3 task A4)

A payload holding neither a card nor a descriptor used to fall through to the
NFC gather with no screen at all. The door now names all three routes and
states what the payload holds, counted through has() rather than take() so a
menu can describe a payload the operator has not compared yet.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
### Task A5: `gui/composer_state.go`, `gui/composer_shape.go` and `gui/composer_discard.go` -- the wrapper, the path list, the picker bounds, the §4e refusals and the two EXPERIMENTAL confirms

**Files:**
- Create: `gui/composer_state.go`
- Create: `gui/composer_shape.go`
- Create: `gui/composer_discard.go` -- moved here from Task B3, because `composerShapeFlow` calls `composerApplyShapeEdit` and Part A has to compile with Part B absent. Measured by the `GATE_UNTIL` run: with the file in Part B, Part A failed `go vet` with `undefined: composerApplyShapeEdit`
- Test: `gui/composer_shape_test.go`

**Interfaces:**
- Consumes (all from S2): `md.PathList{Wrapper ComposeWrapper; Paths []SpendPath}`, `md.SpendPath{Keys *KeySet; Hash *[32]byte; Lock *Lock}`, `md.KeySet{K, N uint8; Sorted bool}`, `md.Lock{Kind LockKind; Value uint32}`, `md.ComposeTr`/`ComposeWsh`/`ComposeShWsh`/`ComposeSh`, `md.ValidatePathList(list) (int, error)`, `md.ComposeMaxPaths`/`ComposeMaxKeysPerPath`/`ComposeMaxSlots`, and the sentinels `md.ErrComposeNoPaths`, `ErrComposeNoKeyedPath`, `ErrComposeLockOnlyPath`, `ErrComposeKeylessUnderTr`, `ErrComposeLegacyWrapperShape`, `ErrComposeBadThreshold`, `ErrComposeTooManyPaths`, `ErrComposeTooManySlots`. Plus `ChoiceScreen`, `showError`, `ConfirmWarningScreen`, `composerPickScreen` (Task A2), `composerCopy*` (Task A1), `seedRegistry` (`gui/multisig_build_slots.go:172`).
- Produces: `type composerState`; `type composerBound`; `func composerBoundFrom(s *syswSession) composerBound`; `func composerPathLine(p md.SpendPath, idx int) string`; `func composerSlotsKeysLine(st *composerState) string`; `func composerSlotCount(list md.PathList) int`; `func composerMaxKeysForPath(st *composerState, pathIdx int) int`; `func composerSortedIsLegal(list md.PathList, idx int) bool`; `func composerRefusalBody(err error) (string, bool)`; `func composerConfirmScreen(ctx, th, title, body string) bool`; `func composerWrapperPick(ctx, th) (md.ComposeWrapper, bool)`; `func composerShapeFlow(ctx, th, st *composerState) bool`.

**Picker bounds (§4b, §4e).** The picker does not offer an illegal value, which is why most of §4e can never be reached from the UI: paths are capped at `md.ComposeMaxPaths` (8), a path's `n` at `min(md.ComposeMaxKeysPerPath, md.ComposeMaxSlots - slots already used elsewhere)`, `k` at `1..n`, and under `sh`/`sh(wsh)` the wrapper offers only the plain k-of-n preset with `n >= 2`. The refusals still exist and are still tested, because `md.ValidatePathList` is the authority and the UI is a convenience over it -- a bound written twice is a bound that can disagree with itself, so the flow calls `ValidatePathList` before it leaves the shape and shows §8m for whatever comes back.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_shape_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

func composerKeyedPath(k, n uint8, sorted bool) md.SpendPath {
	return md.SpendPath{Keys: &md.KeySet{K: k, N: n, Sorted: sorted}}
}

func TestComposerPathLineNamesTheShapeAnOperatorSees(t *testing.T) {
	digest := [32]byte{0xab}
	for _, tc := range []struct {
		name string
		p    md.SpendPath
		want string
	}{
		{"plain k of n", composerKeyedPath(2, 3, true), "Path 1: 2-of-3"},
		{"single key", composerKeyedPath(1, 1, false), "Path 1: 1 key"},
		{"with a relative time lock", md.SpendPath{
			Keys: &md.KeySet{K: 2, N: 3}, Lock: &md.Lock{Kind: md.LockOlderUnits, Value: 15188},
		}, "Path 1: 2-of-3 + 90 days"},
		{"with a block height", md.SpendPath{
			Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockAfterHeight, Value: 905000},
		}, "Path 1: 1 key + block 905000"},
		{"key-less hash path", md.SpendPath{Hash: &digest}, "Path 1: hash only"},
		{"keys and a hash", md.SpendPath{Keys: &md.KeySet{K: 2, N: 2}, Hash: &digest},
			"Path 1: 2-of-2 + hash"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if got := composerPathLine(tc.p, 0); got != tc.want {
				t.Errorf("composerPathLine = %q, want %q", got, tc.want)
			}
		})
	}
}

// TestComposerRefusalBodyMapsEverySentinelToItsSection8mLine is §12 item 4 for
// the structural family: every refusal REFUSES, and with the exact §8 line.
//
// It is table-driven off the sentinels rather than off strings, so a renamed
// or added ErrCompose* arm shows up as an unmapped error rather than as a
// screen that says nothing.
func TestComposerRefusalBodyMapsEverySentinelToItsSection8mLine(t *testing.T) {
	for _, tc := range []struct {
		err  error
		want string
	}{
		{md.ErrComposeNoKeyedPath, composerCopyRefuseNoKeyedPath()},
		{md.ErrComposeLockOnlyPath, composerCopyRefuseLockOnly()},
		{md.ErrComposeKeylessUnderTr, composerCopyRefuseKeylessTr()},
		{md.ErrComposeLegacyWrapperShape, composerCopyRefuseLegacyShape()},
		{md.ErrComposeTooManySlots, composerCopyRefuseSlotCap()},
	} {
		got, ok := composerRefusalBody(tc.err)
		if !ok {
			t.Errorf("%v maps to no §8m body, so the operator would be refused with nothing", tc.err)
			continue
		}
		if got != tc.want {
			t.Errorf("%v maps to %q, want %q", tc.err, got, tc.want)
		}
	}
	// An unmapped error must be REPORTED as unmapped, not silently rendered as
	// one of the five.
	if _, ok := composerRefusalBody(md.ErrComposeBadThreshold); ok {
		t.Error("ErrComposeBadThreshold maps to a §8m body; the picker prevents it and " +
			"§8m has no line for it, so it must not borrow another refusal's words")
	}
}

// TestComposerShapeRefusalsActuallyRefuse drives ValidatePathList over the
// four shapes §4e names, so the mapping above is pinned to the codec's real
// answers rather than to this test's idea of them.
func TestComposerShapeRefusalsActuallyRefuse(t *testing.T) {
	digest := [32]byte{0x11}
	for _, tc := range []struct {
		name string
		list md.PathList
		want string
	}{
		{"no path with keys", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{Hash: &digest}}},
			composerCopyRefuseNoKeyedPath()},
		{"a path with neither keys nor hash", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			composerKeyedPath(1, 1, false),
			{Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 100}},
		}}, composerCopyRefuseLockOnly()},
		{"key-less path under tr", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			composerKeyedPath(1, 1, false), {Hash: &digest},
		}}, composerCopyRefuseKeylessTr()},
		{"legacy wrapper, two paths", md.PathList{Wrapper: md.ComposeSh, Paths: []md.SpendPath{
			composerKeyedPath(2, 3, true), composerKeyedPath(1, 2, true),
		}}, composerCopyRefuseLegacyShape()},
	} {
		t.Run(tc.name, func(t *testing.T) {
			_, err := md.ValidatePathList(tc.list)
			if err == nil {
				t.Fatalf("md.ValidatePathList ACCEPTED a shape §4e refuses; the refusal " +
					"screen below would never be reached")
			}
			body, ok := composerRefusalBody(err)
			if !ok {
				t.Fatalf("no §8m body for %v", err)
			}
			if body != tc.want {
				t.Errorf("refused with %q, want %q", body, tc.want)
			}
		})
	}
}

// TestComposerPickerBoundsNeverOfferAnIllegalValue is §4e's "REFUSE at the
// picker (the picker does not offer the value)".
func TestComposerPickerBoundsNeverOfferAnIllegalValue(t *testing.T) {
	st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh}}
	// An empty policy: the whole 32-slot budget is available, capped at 9.
	if got := composerMaxKeysForPath(st, 0); got != md.ComposeMaxKeysPerPath {
		t.Errorf("an empty policy offers up to %d keys, want %d", got, md.ComposeMaxKeysPerPath)
	}
	// Fill 28 slots across four paths; the fifth path may then offer 4, not 9.
	st.list.Paths = []md.SpendPath{
		composerKeyedPath(1, 7, false), composerKeyedPath(1, 7, false),
		composerKeyedPath(1, 7, false), composerKeyedPath(1, 7, false),
		{},
	}
	if got := composerSlotCount(st.list); got != 28 {
		t.Fatalf("composerSlotCount = %d, want 28", got)
	}
	if got := composerMaxKeysForPath(st, 4); got != 4 {
		t.Errorf("with 28 slots taken the picker offers %d more, want 4 (the 32-slot wire cap)", got)
	}
	// And at the cap it offers none, which is what makes §8m line 5 reachable.
	st.list.Paths[4] = composerKeyedPath(1, 4, false)
	st.list.Paths = append(st.list.Paths, md.SpendPath{})
	if got := composerMaxKeysForPath(st, 5); got != 0 {
		t.Errorf("at 32 slots the picker offers %d more, want 0", got)
	}
}

// TestComposerSortedIsLegalOnlyWhereSection5SaysSo is what keeps the §8b
// confirm honest: §5a rules it fires ONLY where sorted was legal and
// declined, never on a lowering-forced multi.
func TestComposerSortedIsLegalOnlyWhereSection5SaysSo(t *testing.T) {
	digest := [32]byte{0x22}
	sole := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{composerKeyedPath(2, 3, true)}}
	if !composerSortedIsLegal(sole, 0) {
		t.Error("a sole unlocked, unhashed 2-of-3 is exactly where sortedmulti is legal")
	}
	locked := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{
		Keys: &md.KeySet{K: 2, N: 3}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 10},
	}}}
	if composerSortedIsLegal(locked, 0) {
		t.Error("a locked path cannot be sortedmulti (nested sortedmulti is refused by md " +
			"and by BIP-383/388), so the §8b confirm must not be offered for it")
	}
	hashed := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{
		Keys: &md.KeySet{K: 2, N: 3}, Hash: &digest,
	}}}
	if composerSortedIsLegal(hashed, 0) {
		t.Error("a hashed path is not a sole sortedmulti child either")
	}
	two := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		composerKeyedPath(2, 3, true), composerKeyedPath(1, 2, true)}}
	if composerSortedIsLegal(two, 0) {
		t.Error("with two paths neither is the sole child, so both are lowering-forced multi")
	}
	single := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{composerKeyedPath(1, 1, false)}}
	if composerSortedIsLegal(single, 0) {
		t.Error("n = 1 lowers to pkh/pk; there is no sorted form to decline")
	}
	legacy := md.PathList{Wrapper: md.ComposeSh, Paths: []md.SpendPath{composerKeyedPath(2, 3, true)}}
	if composerSortedIsLegal(legacy, 0) {
		t.Error("the legacy wrappers are sorted-only (§4e, feasibility M-5), so the §8b " +
			"confirm is never offered under them")
	}
}

// The two EXPERIMENTAL confirm bodies, under all three §12 item 5 gates.
func TestComposerExperimentalConfirmsDrawInFullAndFireOnCondition(t *testing.T) {
	for _, tc := range []struct {
		what string
		body string
	}{
		{"the §8a key-less path confirm", composerConfirmBody(composerCopyKeylessPath())},
		{"the §8b unsorted keys confirm", composerConfirmBody(composerCopyUnsortedKeys())},
	} {
		assertModalBodyFits(t, tc.what, confirmWarningBody, tc.body)
	}
	// FIRES ON CONDITION: adding a key-less path to a wsh list shows §8a, and
	// declining it leaves the path list unchanged.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerConfirmScreen(ctx, &descriptorTheme, "EXPERIMENTAL",
				composerConfirmBody(composerCopyKeylessPath()))
		})
		defer quit()
		content, ok := frame()
		if !ok {
			t.Fatal("the §8a confirm never drew a frame")
		}
		assertFrameHasBody(t, ink(), "the §8a key-less path confirm")
		if !uiContains(content, "bearer access") {
			t.Errorf("the §8a confirm does not name the consequence.\nFrame: %q", content)
		}
		if !strings.Contains(strings.ToLower(content), "hold") {
			t.Errorf("the §8a confirm does not say how to get past it.\nFrame: %q", content)
		}
	})
}

// TestComposerEveryPathHashedWarns is §8h, fired at the transition out of the
// shape, and §12 item 5's condition test for it.
func TestComposerEveryPathHashedWarns(t *testing.T) {
	digest := [32]byte{0x33}
	all := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}, Hash: &digest},
		{Keys: &md.KeySet{K: 2, N: 3}, Hash: &digest},
	}}
	if !composerEveryPathHashed(all) {
		t.Error("a list whose every path carries a hash does not trip §8h")
	}
	some := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}, Hash: &digest},
		composerKeyedPath(2, 3, false),
	}}
	if composerEveryPathHashed(some) {
		t.Error("§8h fired on a list with an un-hashed path; it would then be a warning " +
			"the operator learns to tap past")
	}
	assertModalBodyFits(t, "the §8h every-path-hashed warning", errorScreenBody, composerCopyHashEveryPath())
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerPathLine|^TestComposerRefusal|^TestComposerShape|^TestComposerPicker|^TestComposerSorted|^TestComposerExperimental|^TestComposerEveryPath' ./gui/ 2>&1 | tail -8`
Expected: FAIL to build -- `undefined: composerState`, `composerPathLine`, `composerRefusalBody`, `composerMaxKeysForPath`, `composerSlotCount`, `composerSortedIsLegal`, `composerConfirmScreen`, `composerEveryPathHashed`.

- [ ] **Step 3: Write the state**

Create `gui/composer_state.go`:

```go
package gui

import (
	"fmt"
	"time"

	"seedhammer.com/md"
	"seedhammer.com/mk"
	"seedhammer.com/sysw"
)

// composerState is everything one composition holds, from the door to the
// last plate.
//
// ONE STRUCT FOR BOTH PARTS OF THE FLOW, declared here rather than grown
// field by field, because the seating half reads what the shape half wrote
// and a state split across two types is a state with two answers to "how
// many slots are there". The shape half writes `list` and `bound`; the
// seating half writes `sources`, `assigned` and `reg`.
//
// WHAT IT IS NOT: the source of anything shown at consent. §7e derives the
// consent surface from the DECODED md1 the device is about to engrave, never
// from this struct, and §8q's self-check compares the two. A field here that
// the consent screen read directly would defeat the check that exists to
// catch a builder defect.
type composerState struct {
	// list is the operator's ordered spend-path list under one wrapper: the
	// value md.Compose lowers. The GUI never builds a descriptor itself.
	list md.PathList

	// bound is the payload's now: record (§6a, C24): a LOWER bound on the
	// present, affecting echoes and refusals only, never an encoded operand.
	bound composerBound

	// NO CONFIRM MEMO LIVES HERE, and its absence is the fix rather than an
	// omission. §8a and §8b were memoised by the operator's path INDEX, and an
	// index is not an identity: "Remove path" splices the slice and left the
	// map, so a new key-less path at a reused index skipped an unskippable
	// confirm (C16). §8a now fires where a key-less path is CREATED, which
	// happens exactly once per path by construction, and §8b fires at the
	// transition out of the shape, where the sole-path condition it depends on
	// is final.

	// sources are the seatable keys: key: records, mk1 cards and seeds
	// (§7d). Filled by the seating half.
	sources []composerSource

	// assigned[i] is the source seated at EMITTED slot index i (§5's
	// first-appearance numbering), or the zero value with src < 0 for an
	// unseated slot. Discarded wholesale by any edit that moves slot
	// numbering (§7d, §8j).
	assigned []composerAssignment

	// reg holds every seed entered in this flow. C14: scrub on exit as
	// Multisig Build does -- and as there, the `defer reg.scrub()` is
	// installed at the FLOW's entry, before any seed exists, so every exit is
	// covered by construction (gui/multisig_build.go:290-291).
	reg *seedRegistry
}

// composerBound is the payload's now: record, decoded (§6a, §6b).
//
// A FIELD THAT IS ABSENT BOUNDS NOTHING. `now:` may carry seconds alone or
// seconds and a height; the height bounds heights and the seconds bound
// dates, and the echo for a kind whose field is missing carries the bare
// disclaimer (§6b). The copy never says "now": a stale record can only weaken
// the below-bound refusal, never invent one.
type composerBound struct {
	seconds   uint32
	height    uint32
	hasBound  bool
	hasHeight bool
}

// packDate renders the bound's seconds as the YYYY-MM-DD the §8c body prints.
func (b composerBound) packDate() string {
	return time.Unix(int64(b.seconds), 0).UTC().Format("2006-01-02")
}

// composerBoundFrom reads the loaded payload's single now: record.
//
// AT MOST ONE, enforced at the two sites that see the whole payload: the host
// `pack_with` and syswSession.load (§6a). Two operator-supplied records are a
// host refusal; if two ever reach the device they both go inert, so this
// takes the first ClassNow record and a second changes nothing.
func composerBoundFrom(s *syswSession) composerBound {
	if s == nil || !s.loaded {
		return composerBound{}
	}
	for _, r := range s.records {
		if r.class != sysw.ClassNow {
			continue
		}
		n, err := sysw.ParseNowRecord(r.body)
		if err != nil {
			// Unreachable: a record that does not parse is not ClassNow. The
			// arm exists because consuming a value from a call that returned an
			// error is the defect gui/policy_address.go:63-75 documents.
			return composerBound{}
		}
		return composerBound{
			seconds: n.Seconds, height: n.Height,
			hasBound: true, hasHeight: n.HasHeight,
		}
	}
	return composerBound{}
}

// composerSourceKind names where a seatable key comes from (§7d).
type composerSourceKind uint8

const (
	// composerSourceKey is a key: record: fingerprint, origin and xpub, all
	// DECLARED. The device can check the xpub's depth and last component
	// against the path and nothing else (F-217).
	composerSourceKey composerSourceKind = iota
	// composerSourceCard is an mk1 card from the payload. Its stubs are
	// IGNORED at seating -- the policy does not exist yet -- and both stubs
	// are appended when it is re-minted (§7d).
	composerSourceCard
	// composerSourceSeed is a BIP-39 or ms1 seed. Unlike the other two it is
	// not used up: one seed fills as many slots as the operator assigns, each
	// at its own hardened account by ordinal (§4f, C12).
	composerSourceSeed
)

// composerSource is one seatable key on the pick list.
type composerSource struct {
	kind  composerSourceKind
	label string
	// used is "consumed" for a key: record and an mk1 card (C8's "remaining"),
	// and always false for a seed.
	used bool

	fingerprint [4]byte
	fpPresent   bool
	origin      []md.PathComponent
	// xpub is the base58 account key. mk.Card.Xpub is a string
	// (mk/mk.go:138), and deriveAccountXpub returns one, so this is the form
	// every source has in common; decodeXpubBytes converts for md.Bind.
	xpub string
	// card is the payload mk1 this source came from, for re-minting.
	card mk.Card
	// seedID indexes composerState.reg for a seed source, and is -1 otherwise.
	seedID int
}

// composerAssignment is what fills one emitted slot.
type composerAssignment struct {
	// src indexes composerState.sources, or -1 for an unseated slot.
	src int
	// account is the BIP-48 account component for a SEED-derived slot: the
	// ordinal among the slots that master fills, in ascending emitted slot
	// index (§4f). Zero and meaningless for the other two kinds, which carry
	// the origin their record or card DECLARES, verbatim.
	account uint32
	// origin and fingerprint are the resolved declaration for this slot, as
	// the mapping review prints them and as md.ComposeWith receives them.
	origin      []md.PathComponent
	fingerprint [4]byte
	fpPresent   bool
	xpub        string
}

// composerAnySlotAssigned reports whether any emitted slot holds a source.
//
// It lives with the STATE rather than with the discard rule because both
// halves of the flow ask it: the shape half to decide whether §8j has
// anything to warn about, and the seating half to decide whether the
// shortfall check applies. An unseated slot is src == -1, never a zero value,
// so a freshly-made slice must be initialised rather than left at zero -- a
// zero src would read as "seated from source 0".
func composerAnySlotAssigned(st *composerState) bool {
	for _, a := range st.assigned {
		if a.src >= 0 {
			return true
		}
	}
	return false
}

// composerSlotCount is the policy's TOTAL slot count: the number the wire's
// 5-bit path_decl.n caps at 32 (md/md.go:215-221).
func composerSlotCount(list md.PathList) int {
	n := 0
	for _, p := range list.Paths {
		if p.Keys != nil {
			n += int(p.Keys.N)
		}
	}
	return n
}

// composerMaxKeysForPath is the picker's own bound (§4e: "the picker does not
// offer the value"): a path may hold up to 9 keys, and never more than the
// 32-slot budget the rest of the policy leaves.
func composerMaxKeysForPath(st *composerState, pathIdx int) int {
	used := 0
	for i, p := range st.list.Paths {
		if i == pathIdx || p.Keys == nil {
			continue
		}
		used += int(p.Keys.N)
	}
	free := md.ComposeMaxSlots - used
	if free < 0 {
		free = 0
	}
	if free > md.ComposeMaxKeysPerPath {
		free = md.ComposeMaxKeysPerPath
	}
	return free
}

// composerSortedIsLegal reports whether §5's key-set rule admits a SORTED
// form for this path -- which is the only place the §8b confirm may fire.
//
// §5's rule: SOLE path, unlocked, unhashed, n >= 2 lowers to sortedmulti (or
// sortedmulti_a under tr) by BIP-383/388's sole-child rule; ANY other
// multi-key path is necessarily unsorted, because nested sortedmulti is
// refused by md and by the BIPs. So on those paths the operator declined
// nothing and §8b must stay silent (§5a).
//
// The legacy wrappers are sorted-only (§4a, §4e), so no choice is offered
// under them either.
func composerSortedIsLegal(list md.PathList, idx int) bool {
	if list.Wrapper == md.ComposeSh || list.Wrapper == md.ComposeShWsh {
		return false
	}
	if len(list.Paths) != 1 || idx != 0 {
		return false
	}
	p := list.Paths[0]
	return p.Keys != nil && p.Keys.N >= 2 && p.Lock == nil && p.Hash == nil
}

// composerEveryPathHashed is §8h's condition: every way to spend this wallet
// needs a preimage that is not on this device and not on these plates.
func composerEveryPathHashed(list md.PathList) bool {
	if len(list.Paths) == 0 {
		return false
	}
	for _, p := range list.Paths {
		if p.Hash == nil {
			return false
		}
	}
	return true
}

// composerPathLine is one row of the path-list screen (§7b's "Path 2: 2-of-3
// + 90 days"). idx is the OPERATOR's zero-based index; the label counts from
// one, as every "Path N" prompt in §7d and §8s does.
func composerPathLine(p md.SpendPath, idx int) string {
	body := "hash only"
	if p.Keys == nil && p.Hash == nil {
		// NEITHER element. It used to read "hash only", so a path left empty
		// by a decline was described as something it was not and then refused
		// at Done with a body about a lock nobody set -- with no way to see
		// what was wrong with it.
		body = "empty"
	}
	if p.Keys != nil {
		if p.Keys.N == 1 {
			body = "1 key"
		} else {
			body = fmt.Sprintf("%d-of-%d", p.Keys.K, p.Keys.N)
		}
	}
	if p.Hash != nil && p.Keys != nil {
		body += " + hash"
	}
	if p.Lock != nil {
		body += " + " + composerLockShort(*p.Lock)
	}
	return fmt.Sprintf("Path %d: %s", idx+1, body)
}

// composerLockShort is the lock as a path-list row shows it: short enough for
// one line, in the operator's own units. The full echo (§8c) is what the lock
// entry screen and the consent screen print.
func composerLockShort(l md.Lock) string {
	switch l.Kind {
	case md.LockOlderBlocks:
		return fmt.Sprintf("%d blocks", l.Value)
	case md.LockOlderUnits:
		return fmt.Sprintf("%d days", composerUnitsToDays(l.Value))
	case md.LockAfterHeight:
		return fmt.Sprintf("block %d", l.Value)
	case md.LockAfterTime:
		return time.Unix(int64(l.Value), 0).UTC().Format("2006-01-02")
	}
	return "lock"
}

// composerUnitsToDays converts 512-second units back to whole days.
//
// IT FLOORS, and the direction is not arbitrary. Days-to-units rounds UP (a
// lock must never be shorter than the operator asked for), so the encoded
// value always covers at least the days typed and a bit more: 90 days is
// 15188 units, which is 90.0029 days. Flooring recovers the number the
// operator TYPED; rounding up would print 91 days back at someone who typed
// 90, on the screen whose whole job is to read the value back to them.
func composerUnitsToDays(units uint32) uint32 {
	return uint32(uint64(units) * 512 / 86400)
}

// composerSlotsKeysLine is §7b's live line.
//
// A SEED IS NOT A COUNT. §7d rules that "keys available" counts records plus
// cards plus, for each seed, "any slots", so a payload with two records and a
// seed reads `keys available: 2 + seed` rather than 3 -- which would promise
// a third distinct key that does not exist.
func composerSlotsKeysLine(st *composerState) string {
	if len(st.sources) == 0 {
		// §7b scopes this line to "whenever a payload is loaded". With none it
		// read `keys available: 0` on a build the door has already described
		// as key-less -- a count that answers a question nobody asked.
		return fmt.Sprintf("slots: %d", composerSlotCount(st.list))
	}
	records, seeds := 0, 0
	for _, s := range st.sources {
		if s.kind == composerSourceSeed {
			seeds++
		} else {
			records++
		}
	}
	line := fmt.Sprintf("slots: %d / keys available: %d", composerSlotCount(st.list), records)
	switch {
	case seeds == 1:
		line += " + seed"
	case seeds > 1:
		line += " + seeds"
	}
	return line
}
```

- [ ] **Step 4: Write the discard machinery the shape flow calls**

`composerApplyShapeEdit` is what makes the discard EXACT: `composerShapeGuard` asks §8j before an editor that CAN renumber, and this decides afterwards whether anything actually did. It lives in Part A because `composerShapeFlow` calls it; Task B3 owns the rule's tests and its §8j semantics.

Create `gui/composer_discard.go`:

```go
package gui

import (
	"fmt"
	"strings"

	"seedhammer.com/md"
)

// Discard-on-numbering-change (SPEC §7d, §8j).
//
// WHY ALL ASSIGNMENTS AND NOT THE ONES THAT MOVED. §5 numbers slots by FIRST
// APPEARANCE in the emitted text, and that text is a function of the wrapper
// as well as of the path list -- tr extracts an internal key as @0 and wsh
// does not. A carried assignment would seat keys silently into the wrong
// slots, which is the one failure gui/key_card_seating.go:24-27 refuses to
// allow anywhere on this device: a misassignment does not fail, it derives a
// different wallet's address and shows it to the operator as proof.
//
// A LOCK OR HASH EDIT MOVES NO SLOT. Assignments are kept across it and the
// stub screen is re-shown, because the template ID is not shape-invariant
// even when the numbering is (§7c).

// composerShapeSignature captures exactly what slot numbering depends on: the
// wrapper, the number of paths, and each path's KEY COUNT. Not k, which
// changes no slot; not the lock; not the digest.
func composerShapeSignature(list md.PathList) string {
	var b strings.Builder
	fmt.Fprintf(&b, "w%d/", list.Wrapper)
	for _, p := range list.Paths {
		n := 0
		if p.Keys != nil {
			n = int(p.Keys.N)
		}
		fmt.Fprintf(&b, "%d,", n)
	}
	return b.String()
}

// composerDiscardAssignments clears every seat and releases every source it
// held. Both halves, because a source left marked `used` would never be
// offered again for the slot it no longer fills.
func composerDiscardAssignments(st *composerState) {
	for i := range st.assigned {
		st.assigned[i] = composerAssignment{src: -1}
	}
	for i := range st.sources {
		st.sources[i].used = false
	}
}

// composerApplyShapeEdit runs an edit and discards the seats if, and only if,
// the numbering moved.
func composerApplyShapeEdit(st *composerState, edit func()) bool {
	before := composerShapeSignature(st.list)
	edit()
	if composerShapeSignature(st.list) == before {
		return false
	}
	composerDiscardAssignments(st)
	st.assigned = make([]composerAssignment, composerSlotCount(st.list))
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
	return true
}
```

- [ ] **Step 5: Write the shape flow**

Create `gui/composer_shape.go`:

```go
package gui

import (
	"errors"
	"fmt"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/op"
	"seedhammer.com/md"
)

// The shape half of the composer (SPEC §7b): wrapper, then an ordered list of
// spend paths the operator edits until it validates.
//
// THE UI'S BOUNDS AND THE CODEC'S REFUSALS ARE BOTH KEPT, and that is not
// redundancy. The picker does not offer an illegal value (§4e: "REFUSE at the
// picker"), which is the kinder half; md.ValidatePathList is the AUTHORITY,
// runs before the shape is left, and its answer is what §8m renders. A bound
// written only in the UI is a bound that can drift from the codec's, and the
// codec is the one that decides what gets engraved.
//
// BACK PRESERVES EVERYTHING (2026-08-19 operator directive, the same rule
// gui/multisig_build.go:291-299 states): Back inside a path editor returns to
// the list with the list intact; Back at the list leaves the shape, and the
// caller decides what that means.

// composerRefusalBody maps a compose sentinel to its §8m line.
//
// ONLY THE FIVE §8 NAMES. An error with no §8m line returns ok=false and the
// caller shows the codec's own message instead of borrowing another
// refusal's words -- a refusal that says the wrong true thing is worse than
// one that says an unpolished true thing.
func composerRefusalBody(err error) (string, bool) {
	switch {
	// An EMPTY list and a list with no keyed path are one sentence to an
	// operator: "Every wallet needs at least one path with a key." Tapping
	// Done on an empty list is a plausible first action, and without this arm
	// it printed md.ErrComposeNoPaths' text -- `md: compose: a wallet needs at
	// least one spend path` -- an internal prefix on an operator screen, which
	// §11 forbids.
	case errors.Is(err, md.ErrComposeNoPaths), errors.Is(err, md.ErrComposeNoKeyedPath):
		return composerCopyRefuseNoKeyedPath(), true
	case errors.Is(err, md.ErrComposeLockOnlyPath):
		return composerCopyRefuseLockOnly(), true
	case errors.Is(err, md.ErrComposeKeylessUnderTr):
		return composerCopyRefuseKeylessTr(), true
	case errors.Is(err, md.ErrComposeLegacyWrapperShape):
		return composerCopyRefuseLegacyShape(), true
	case errors.Is(err, md.ErrComposeTooManySlots):
		return composerCopyRefuseSlotCap(), true
	}
	return "", false
}

// composerShowRefusal renders §8m for a compose error, or the codec's own
// message when §8 has no line for it.
func composerShowRefusal(ctx *Context, th *Colors, title string, err error) {
	if body, ok := composerRefusalBody(err); ok {
		showError(ctx, th, title, body)
		return
	}
	showError(ctx, th, title, err.Error())
}

// composerConfirmScreen is the unskippable confirm-to-proceed surface: the
// same ConfirmWarningScreen shape multisigBuildExperimentalWarning uses
// (gui/multisig_build.go:854-871), hold-to-confirm, Back declines.
func composerConfirmScreen(ctx *Context, th *Colors, title, body string) bool {
	warn := &ConfirmWarningScreen{Title: title, Body: body, Icon: assets.IconHammer}
	for !ctx.Done {
		dims := ctx.Platform.DisplaySize()
		d, res := warn.Layout(ctx, th, dims)
		switch res {
		case ConfirmNo:
			return false
		case ConfirmYes:
			return true
		}
		ctx.Frame(op.Layer(d, op.Color(&ctx.B, th.Background)))
	}
	return false
}

// composerShapeGuard asks §8j before an edit that CAN move slot numbering.
//
// IT HAS ITS REAL BODY FROM THE START and is never a stub. In Part A nothing
// has been seated, so it returns true without drawing -- which is §7d's own
// rule ("With no slot yet assigned there is nothing to discard and §8j does
// not fire"), not a placeholder standing in for it.
//
// It runs BEFORE the edit, so it cannot know what the operator will change.
// The confirm is therefore asked on ENTRY to an editor that can renumber, and
// composerApplyShapeEdit's signature comparison afterwards decides whether
// anything is actually discarded -- so answering "continue" and then touching
// only a lock keeps the seats, which is §7d's rule for a lock edit.
func composerShapeGuard(ctx *Context, th *Colors, st *composerState) bool {
	if !composerAnySlotAssigned(st) {
		// Nothing is at stake, so nothing is asked. A warning that fires when
		// nothing is at stake is one the operator learns to tap through.
		return true
	}
	return composerConfirmScreen(ctx, th, "Edit the shape",
		composerConfirmBody(composerCopyEditClearsKeys()))
}

// composerWrapperPick is §4a. The legacy wrappers are offered because C7's
// migration needs them, and §4e then holds them to ONE unlocked, unhashed
// key set with n >= 2.
func composerWrapperPick(ctx *Context, th *Colors) (md.ComposeWrapper, bool) {
	choices := []string{"Taproot (tr)", "Segwit (wsh)", "Nested (sh-wsh)", "Legacy (sh)"}
	wrappers := []md.ComposeWrapper{md.ComposeTr, md.ComposeWsh, md.ComposeShWsh, md.ComposeSh}
	cs := &ChoiceScreen{Title: "New policy", Lead: "Which script?", Choices: choices}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return md.ComposeTr, false
	}
	return wrappers[sel], true
}

// composerCountPick offers 1..max on a paged list, so a 9-row picker cannot
// overflow the panel the way an unpaged ChoiceScreen would.
func composerCountPick(ctx *Context, th *Colors, title, lead string, min, max int) (int, bool) {
	if max < min {
		return 0, false
	}
	rows := make([]string, 0, max-min+1)
	for v := min; v <= max; v++ {
		rows = append(rows, fmt.Sprintf("%d", v))
	}
	sel, ok := composerPickScreen(ctx, th, title, lead, rows)
	if !ok {
		return 0, false
	}
	return min + sel, true
}

// composerKeysEdit asks for n then k, within the picker's bounds, and offers
// the sorted choice ONLY where §5 makes sorted legal.
func composerKeysEdit(ctx *Context, th *Colors, st *composerState, idx int) bool {
	max := composerMaxKeysForPath(st, idx)
	if max == 0 {
		// The 33rd slot, refused where the operator asked for it.
		showError(ctx, th, "Keys", composerCopyRefuseSlotCap())
		return false
	}
	min := 1
	if st.list.Wrapper == md.ComposeSh || st.list.Wrapper == md.ComposeShWsh {
		// §4a: n = 1 is refused at the picker under the legacy wrappers.
		min = 2
		if max < 2 {
			showError(ctx, th, "Keys", composerCopyRefuseLegacyShape())
			return false
		}
	}
	n, ok := composerCountPick(ctx, th, "Keys", fmt.Sprintf("Path %d: how many keys?", idx+1), min, max)
	if !ok {
		return false
	}
	k, ok := composerCountPick(ctx, th, "Threshold", fmt.Sprintf("Path %d: how many must sign?", idx+1), 1, n)
	if !ok {
		return false
	}
	// SORTED IS LEFT TRUE HERE AND THE QUESTION IS ASKED LATER.
	//
	// It used to be asked right here, and that was a decision the device
	// reversed in silence: §5's key-set rule lowers a SOLE unlocked, unhashed
	// path to sortedmulti and ANY other multi-key path to `multi`, so adding a
	// second path turned an answered-sorted path into an order-dependent one
	// with nothing said. The operator was left holding the opposite of §8b's
	// warning from a screen this flow had just shown them.
	//
	// So the question moves to the transition out of the shape
	// (composerKeyOrderStep), where `sole` is FINAL and cannot be reversed
	// afterwards. Restating the outcome on a row instead would describe the
	// reversal after the fact; asking once, at the end, removes it.
	st.list.Paths[idx].Keys = &md.KeySet{K: uint8(k), N: uint8(n), Sorted: true}
	return true
}

// composerKeyOrderStep asks §5's key-order question ONCE, at the transition
// out of the shape, and only where §5 makes a sorted form legal at all.
//
// Called after ValidatePathList has accepted the list, so `sole` is final: no
// later edit can turn the answer into its opposite without coming back through
// here. §8b fires only on the decline, which is §5a's rule -- never on a
// lowering-forced `multi`, where the operator declined nothing.
func composerKeyOrderStep(ctx *Context, th *Colors, st *composerState) bool {
	if !composerSortedIsLegal(st.list, 0) {
		return true
	}
	cs := &ChoiceScreen{
		Title:   "Key order",
		Lead:    "Sorted keys, or your order?",
		Choices: []string{"Sorted (usual)", "Keep my order"},
	}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return false
	}
	if sel == 0 {
		st.list.Paths[0].Keys.Sorted = true
		return true
	}
	if !composerConfirmScreen(ctx, th, "EXPERIMENTAL",
		composerConfirmBody(composerCopyUnsortedKeys())) {
		return false
	}
	st.list.Paths[0].Keys.Sorted = false
	return true
}

// composerAddPath appends a path and runs the §8a confirm when the operator
// makes it key-less.
func composerAddPath(ctx *Context, th *Colors, st *composerState) {
	if len(st.list.Paths) >= md.ComposeMaxPaths {
		// Unreachable from the list, which stops offering the row at the cap.
		// Kept as a bound rather than a screen: a refusal with no §8 home is
		// copy no gate covers (§11).
		return
	}
	idx := len(st.list.Paths)
	st.list.Paths = append(st.list.Paths, md.SpendPath{})
	cs := &ChoiceScreen{
		Title:   fmt.Sprintf("Path %d", idx+1),
		Lead:    "What can spend on this path?",
		Choices: []string{"Keys", "A hash, no keys"},
	}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		st.list.Paths = st.list.Paths[:idx]
		return
	}
	if sel == 0 {
		if !composerKeysEdit(ctx, th, st, idx) {
			st.list.Paths = st.list.Paths[:idx]
		}
		return
	}
	// A key-less path is wsh-only and EXPERIMENTAL (§4b, C16). Under tr it is
	// refused with §8m line 3 rather than confirmed.
	if st.list.Wrapper == md.ComposeTr {
		st.list.Paths = st.list.Paths[:idx]
		showError(ctx, th, fmt.Sprintf("Path %d", idx+1), composerCopyRefuseKeylessTr())
		return
	}
	// §8a FIRES ON EVERY KEY-LESS PATH THAT IS CREATED, with no memo.
	//
	// It used to be memoised as keylessConfirmed[idx], and an index is not an
	// identity: "Remove path" splices the slice and leaves the map, so adding
	// path 1 key-less, removing it and adding another put the new path back at
	// index 0 with the confirm already recorded -- an unskippable
	// confirm-to-proceed that could be skipped (C16). A path is created
	// exactly once, here, so firing on creation IS "once per key-less path"
	// (§8a) and needs no state to be wrong about.
	if !composerConfirmScreen(ctx, th, "EXPERIMENTAL",
		composerConfirmBody(composerCopyKeylessPath())) {
		st.list.Paths = st.list.Paths[:idx]
		return
	}
	if !composerHashEdit(ctx, th, st, idx) {
		st.list.Paths = st.list.Paths[:idx]
		return
	}
	// A path that ends with NEITHER keys nor a hash is a cancel, not a path.
	// The key-less route's last pick-list row is "No hash lock", which clears
	// the digest and returns true, and the empty path it left read
	// "Path N: hash only" on the row and was refused at Done with the
	// lock-only body -- naming a lock nobody set.
	if st.list.Paths[idx].Keys == nil && st.list.Paths[idx].Hash == nil {
		st.list.Paths = st.list.Paths[:idx]
	}
}

// composerPathEdit is one path's own menu.
func composerPathEdit(ctx *Context, th *Colors, st *composerState, idx int) {
	for !ctx.Done {
		choices := []string{"Keys", "Time lock", "Hash lock", "Remove path"}
		if idx > 0 {
			// §5 makes LISTED ORDER decide the or_i/or_d nesting, the leaf
			// depth on the taproot spine and which path becomes the internal
			// key -- so an order mistake is a different wallet, and without a
			// move the only repair was remove-and-re-add, which §8j pays for
			// with every seat.
			choices = append(choices, "Move up")
		}
		cs := &ChoiceScreen{
			Title:   fmt.Sprintf("Path %d", idx+1),
			Lead:    composerPathLine(st.list.Paths[idx], idx),
			Choices: choices,
		}
		sel, ok := cs.Choose(ctx, th)
		if !ok {
			return
		}
		switch sel {
		case 0:
			// THE GUARD IS ON THIS ARM AND ON REMOVE, NOT ON THE EDITOR.
			// §7d: "A lock or hash edit moves no slot, keeps assignments", and
			// §7g classifies it DEFAULT. Asking §8j before the editor told an
			// operator who wanted to change a lock that every key would be
			// cleared -- false for the edit they intended -- and declining it
			// left the lock uneditable at all.
			if !composerShapeGuard(ctx, th, st) {
				continue
			}
			// SNAPSHOT AND RESTORE, not clear. A decline at any screen inside
			// the key editor used to leave Keys == nil on a path that already
			// had a key set, which then read "hash only" on the row and was
			// refused at Done with a body about a lock nobody set.
			before := st.list.Paths[idx].Keys
			composerApplyShapeEdit(st, func() {
				if !composerKeysEdit(ctx, th, st, idx) {
					st.list.Paths[idx].Keys = before
				}
			})
		case 1:
			composerLockEdit(ctx, th, st, idx)
		case 2:
			composerHashEdit(ctx, th, st, idx)
		case 3:
			if !composerShapeGuard(ctx, th, st) {
				continue
			}
			composerApplyShapeEdit(st, func() {
				st.list.Paths = append(st.list.Paths[:idx], st.list.Paths[idx+1:]...)
			})
			return
		case 4:
			if !composerShapeGuard(ctx, th, st) {
				continue
			}
			composerApplyShapeEdit(st, func() {
				st.list.Paths[idx-1], st.list.Paths[idx] = st.list.Paths[idx], st.list.Paths[idx-1]
			})
			return
		}
	}
}

// composerShapeFlow runs the path list until it validates, then returns true.
//
// BACK RETURNS FALSE AND THE CALLER GOES BACK ONE STEP, TO THE WRAPPER, WITH
// THE LIST INTACT. It used to make composerFlow return, dropping the wrapper,
// every path, every lock, every digest and every confirm already given --
// against §7b's own rule ("going back should lose nothing") and the standing
// 2026-08-19 directive. Nothing here clears st.list, so the state the caller
// re-enters with is the state the operator left.
//
// THE DISCARD RULE HAS ONE PLACE TO LIVE, and it is composerPathEdit's Keys,
// Remove and Move arms plus composerAddPath -- the four that can move slot
// NUMBERING. A lock or a hash edit moves none (§7d) and is not guarded.
func composerShapeFlow(ctx *Context, th *Colors, st *composerState) bool {
	for !ctx.Done {
		rows := make([]string, 0, len(st.list.Paths)+3)
		for i, p := range st.list.Paths {
			rows = append(rows, composerPathLine(p, i))
		}
		if len(st.list.Paths) < md.ComposeMaxPaths {
			// §4e rules the path cap "at the picker (the picker does not offer
			// the value)". Refusing it afterwards needed an ad-hoc string that
			// is in no §8 table, so neither the glyph gate nor the modal-fits
			// gate covered it; not offering the row is what §4e asks for and
			// needs no copy at all.
			rows = append(rows, "Add a spend path")
		}
		// §7g's row "shape | edits the wrapper ... after a slot was assigned"
		// needs an affordance to be reachable at all, and §12 item 4 names a
		// wrapper change after seating as one of its vectors. Without this row
		// composerShapeSignature's wrapper term was unreachable from the flow.
		rows = append(rows, "Change the script")
		rows = append(rows, "Done")
		lead := composerSlotsKeysLine(st)
		sel, ok := composerPickScreen(ctx, th, "Spend paths", lead, rows)
		if !ok {
			return false
		}
		// THE ROWS ARE DISPATCHED BY NAME, not by an arithmetic offset: the
		// "Add a spend path" row disappears at the cap (§4e), so an index
		// computed from len(st.list.Paths) would silently address the wrong
		// action on a full list.
		switch {
		case sel < len(st.list.Paths):
			composerPathEdit(ctx, th, st, sel)
		case rows[sel] == "Add a spend path":
			if !composerShapeGuard(ctx, th, st) {
				continue
			}
			composerApplyShapeEdit(st, func() { composerAddPath(ctx, th, st) })
		case rows[sel] == "Change the script":
			if !composerShapeGuard(ctx, th, st) {
				continue
			}
			composerApplyShapeEdit(st, func() {
				w, ok := composerWrapperPick(ctx, th)
				if !ok {
					return
				}
				st.list.Wrapper = w
			})
		default:
			if _, err := md.ValidatePathList(st.list); err != nil {
				composerShowRefusal(ctx, th, "Spend paths", err)
				continue
			}
			if !composerKeyOrderStep(ctx, th, st) {
				continue
			}
			if composerEveryPathHashed(st.list) {
				showError(ctx, th, "Spend paths", composerCopyHashEveryPath())
			}
			return true
		}
	}
	return false
}
```

`composerHashEdit` and `composerLockEdit` are written in the hashlock and lock tasks. Until those land this file does not compile, so **order the work: the digit pad, the lock and the hashlock precede the first run of this file's own tests**.

**`composerShapeGuard` has its real body from the start and no stub anywhere.** It asks §8j only when a slot is assigned, so in Part A -- where nothing has been seated -- it returns true without drawing, which is §7d's own rule ("With no slot yet assigned there is nothing to discard and §8j does not fire") rather than a placeholder standing in for it. A stub that returns the right answer for the wrong reason is a stub that can outlive its task silently.

- [ ] **Step 6: Run the tests (after the lock and hashlock tasks land)**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerPathLine|^TestComposerRefusal|^TestComposerShape|^TestComposerPicker|^TestComposerSorted|^TestComposerExperimental|^TestComposerEveryPath' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: every top-level test PASS, including the six sub-tests of the path-line test and the four of the refusal test; `ok seedhammer.com/gui`. The two `assertModalBodyFits` calls log their headroom -- **record both numbers**, they are §12 item 5 evidence.

- [ ] **Step 7: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_state.go gui/composer_shape.go gui/composer_discard.go gui/composer_shape_test.go
git commit -s -F - <<'MSG'
gui: the composer's shape flow -- wrapper, path list, picker bounds, the five section 4e refusals (composer S3 task A5)

The picker does not offer an illegal value and md.ValidatePathList is still
the authority: its sentinels map to section 8m's five bodies, and an
unmapped error shows the codec's own message rather than borrowing another
refusal's words. Section 8b fires only where section 5 makes sorted legal,
never on a lowering-forced multi.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
### Task A6: `gui/composer_digitpad.go` -- the digit pad §6b needs and the fork does not have

**Files:**
- Create: `gui/composer_digitpad.go`
- Test: `gui/composer_digitpad_test.go`

**Interfaces:**
- Consumes: `NewKeyboard(ctx *Context, alphabet string) *Keyboard` (`gui/gui.go:1463`), `Keyboard.Update(ctx) bool` (`:1658`), `Keyboard.Layout(ctx, th) (op.Op, image.Point)` (`:1813`), `Keyboard.Fragment` (`:1443`); `layout.Rectangle`, `widget.Labelw`, `layoutTitle`, `layoutNavigation`, `assets.IconBack`/`IconCheckmark`.
- Produces: `const composerDigitKeys`; `func composerDigitEntry(ctx *Context, th *Colors, title, lead string, maxDigits int, echo func(string) (string, bool)) (string, bool)`.

**No digit-only widget exists.** Measured: `grep -rln "digit" gui/*.go` excluding tests finds a passphrase hex-length message and a comment, no widget; the passphrase keyboard's digit page is mixed with punctuation (`gui/passphrase_keyboard.go`, cited by spec §3's inventory). The nearest primitive is `NewKeyboard` with a chosen alphabet, which `codex32Keys` (`gui/codex32_polish.go:242`) already uses with a digit-leading row -- so the pad is that primitive with a digits-only alphabet, and the flow shape is `inputCodex32Flow`'s (`gui/gui.go:1262-1352`): update the keyboard, read `Fragment`, validate, draw the fragment plus feedback, enable the confirm only when valid.

`NewKeyboard` appends the backspace key and a row break itself (`gui/gui.go:1464-1465`), so the alphabet carries digits alone.

`echo` is the caller's validator AND its echo line in one: it returns the line to draw under the fragment and whether the fragment is acceptable. One function rather than two because the echo and the acceptance are the same judgement, and splitting them is how a screen comes to show a valid echo above a disabled button.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_digitpad_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"
)

// TestComposerDigitPadTypesOnlyDigits is the widget's whole contract: the pad
// offers digits and a backspace, so an operand can never carry a stray rune
// the parser would then have to reject.
func TestComposerDigitPadTypesOnlyDigits(t *testing.T) {
	for _, r := range composerDigitKeys {
		if r == '\n' {
			continue
		}
		if r < '0' || r > '9' {
			t.Errorf("the digit pad's alphabet carries %q; §6b says the operator never "+
				"types a raw operand and never types a separator", r)
		}
	}
	if !strings.Contains(composerDigitKeys, "0") || !strings.Contains(composerDigitKeys, "9") {
		t.Error("the digit pad is missing a digit")
	}
}

// TestComposerDigitPadDrawsItsEchoAndGatesTheConfirm drives the real screen:
// the echo the caller returns is drawn, and the confirm icon appears only
// once the fragment is acceptable.
func TestComposerDigitPadDrawsItsEchoAndGatesTheConfirm(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerDigitEntry(ctx, &descriptorTheme, "Blocks", "How many blocks?", 5,
				func(frag string) (string, bool) {
					if frag == "" {
						return "type a number", false
					}
					return "echo for " + frag, true
				})
		})
		defer quit()
		content, ok := frame()
		if !ok {
			t.Fatal("the digit pad never drew")
		}
		assertFrameHasBody(t, ink(), "the composer digit pad")
		if !uiContains(content, "type a number") {
			t.Errorf("the pad does not draw the caller's feedback for an empty fragment.\nFrame: %q", content)
		}
		if !uiContains(content, "How many blocks?") {
			t.Errorf("the pad does not draw its lead.\nFrame: %q", content)
		}
	})
}

// TestComposerDigitPadBackLeavesWithNothing: Back is a decline everywhere on
// this device, and an entry screen that returned its partial fragment on Back
// would hand a half-typed operand to a lock.
func TestComposerDigitPadBackLeavesWithNothing(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		var got string
		var ok bool
		frame, quit := runUI(ctx, func() {
			got, ok = composerDigitEntry(ctx, &descriptorTheme, "Blocks", "How many?", 5,
				func(frag string) (string, bool) { return "", true })
		})
		defer quit()
		if _, drew := frame(); !drew {
			t.Fatal("no frame")
		}
		click(&ctx.Router, Button1)
		for i := 0; i < 8; i++ {
			if _, more := frame(); !more {
				break
			}
		}
		if ok || got != "" {
			t.Errorf("Back returned (%q, %v), want (\"\", false)", got, ok)
		}
	})
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDigitPad' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerDigitKeys`, `undefined: composerDigitEntry`.

- [ ] **Step 3: Write the pad**

Create `gui/composer_digitpad.go`:

```go
package gui

import (
	"image"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// The composer's digit pad (SPEC §6b, C25).
//
// NO DIGIT-ONLY WIDGET EXISTS in this tree: the passphrase keyboard's digit
// page is mixed with punctuation, and a grep for a numeric widget finds a hex
// length message and a comment. So this is NewKeyboard with a digits-only
// alphabet -- the same primitive codex32Keys already uses with a digit-leading
// row (gui/codex32_polish.go:242) -- driven the way inputCodex32Flow drives
// it (gui/gui.go:1262-1352).
//
// THE OPERATOR NEVER TYPES A RAW OPERAND (§6b). They type a count of blocks,
// a count of days, a block height or eight date digits, and the encoding is
// computed for them. That is why this widget knows nothing about locks: it
// returns the digits and the caller owns the meaning.
//
// NewKeyboard appends the backspace key and the trailing row break itself
// (gui/gui.go:1464-1465), so the alphabet below is digits alone.
const composerDigitKeys = "123\n456\n789\n0"

// composerDigitEntry collects up to maxDigits digits.
//
// `echo` is the caller's validator AND its echo line in one call: it returns
// the line drawn under the fragment and whether the fragment may be
// accepted. One function rather than two, because the echo and the
// acceptance are the same judgement -- splitting them is how a screen comes
// to draw a valid-looking echo above a confirm that does nothing.
//
// The confirm icon is drawn ONLY when the fragment is acceptable, which is
// confirmReviewScreen's ruling on inert controls applied here
// (gui/multisig_build.go:1919-1925).
func composerDigitEntry(ctx *Context, th *Colors, title, lead string, maxDigits int, echo func(string) (string, bool)) (string, bool) {
	kbd := NewKeyboard(ctx, composerDigitKeys)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		if len(kbd.Fragment) > maxDigits {
			// The pad cannot refuse a keypress, so the cap is applied here. It
			// is a TRUNCATION rather than a refusal screen because the operator
			// can see the field and the next backspace fixes it.
			kbd.Fragment = kbd.Fragment[:maxDigits]
		}
		frag := kbd.Fragment
		line, valid := echo(frag)

		if backBtn.Clicked(ctx) {
			// Back is a decline everywhere on this device. Returning the
			// partial fragment would hand a half-typed operand to a lock.
			return "", false
		}
		// Button3 is always DRAINED, so it cannot block the queue head in a
		// direct-call test, and acted on only when the fragment is acceptable
		// -- the same shape inputCodex32Flow uses (gui/gui.go:1277-1280).
		clicked := okBtn.Clicked(ctx)
		if valid && clicked {
			return frag, true
		}

		dims := ctx.Platform.DisplaySize()
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)

		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))

		shown := frag
		if shown == "" {
			shown = " "
		}
		word, frgSize := widget.Labelw(&ctx.B, ctx.Styles.word, dims.X-50, th.Background, shown)
		frgSize.X = max(frgSize.X, 100)
		r := image.Rectangle{Max: frgSize}
		r.Min.Y -= 3
		r.Max.Y += buttonPadY
		r.Min.X -= buttonPadX
		r.Max.X += buttonPadX
		top, _ := content.CutBottom(kbdsz.Y)
		wordOff := top.Center(frgSize)
		word = op.Layer(
			word,
			op.Compose(
				op.Color(&ctx.B, th.Text),
				op.RoundedRect2(&ctx.B, r, cornerRadius),
			),
		).Offset(wordOff)

		var infoOps []op.Op
		lineY := wordOff.Y + frgSize.Y + 8
		for _, s := range []string{lead, line} {
			if s == "" {
				continue
			}
			lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.body, dims.X-2*8, th.Text, s)
			y := lineY
			if lim := top.Max.Y - sz.Y; y > lim {
				y = lim
			}
			infoOps = append(infoOps, lbl.Offset(image.Pt((dims.X-sz.X)/2, y)))
			lineY = y + sz.Y + 4
		}

		navBtns := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		if valid {
			navBtns = append(navBtns, NavButton{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, navBtns...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, title)

		frameOps := []op.Op{kbdOp, word}
		frameOps = append(frameOps, infoOps...)
		frameOps = append(frameOps, nav, titleOp, op.Color(&ctx.B, th.Background))
		ctx.Frame(op.Layer(frameOps...))
	}
	return "", false
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDigitPad' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: three PASS; `ok seedhammer.com/gui`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_digitpad.go gui/composer_digitpad_test.go
git commit -s -F - <<'MSG'
gui: a digits-only entry pad for the composer's lock operands (composer S3 task A6)

No numeric widget existed; the passphrase keyboard's digit page is mixed with
punctuation. This is NewKeyboard with a digits-only alphabet, driven the way
inputCodex32Flow drives it, with the confirm icon drawn only when the caller
says the fragment is acceptable.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task A7: `gui/composer_lock.go` -- lock entry, the §4c range check, the echoes and the four refusals

**Files:**
- Create: `gui/composer_lock.go`
- Test: `gui/composer_lock_test.go`

**Interfaces:**
- Consumes: `md.Lock{Kind LockKind; Value uint32}` and `Lock.Check() error` (S2 -- §12 item 7 calls it "a unit gate on the emitter's input, not on md's acceptance"), `md.LockOlderBlocks`/`LockOlderUnits`/`LockAfterHeight`/`LockAfterTime`; `composerDigitEntry` (Task A6); `ChoiceScreen`; `showError`; `composerCopy*`; `composerBound` (Task A5); `time`.
- Produces: `func composerDaysToUnits(days uint32) uint32`; `func composerDateToUnix(y, m, d int) (uint32, bool)`; `func composerParseDateDigits(s string) (y, m, d int, ok bool)`; `func composerLockBoundLine(b composerBound, kind md.LockKind) string`; `func composerLockEcho(l md.Lock, b composerBound) []string`; `func composerLockAccept(ctx *Context, th *Colors, l md.Lock, b composerBound) bool`; `func composerLockEdit(ctx *Context, th *Colors, st *composerState, idx int) bool`.

**The device enforces §4c itself.** §4c says so in those words, and §12 item 7 makes it a unit gate on the emitter's INPUT rather than on md's acceptance -- because md today accepts `older(0x400000)`, zero time units, which BIP-68 line 46 defines as no lock at all (filed `md-older-zero-time-units-not-refused`). So every value this screen produces goes through `md.Lock.Check()` before it is stored, and the entry bounds below are the same bands stated a second time where the operator meets them.

**The date band is strictly inside §4c's time row (§6b).** Any date whose 00:00 UTC value is below 500,000,000 encodes as a block HEIGHT, not a time -- 1985-11-05 00:00 UTC is 499,996,800 -- so the entry refuses every date before 2009-01-03 with §8t. The ceiling is 2038-01-19, `math.MaxInt32` as a Unix time.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_lock_test.go`:

```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/md"
)

// TestComposerDaysToUnitsMatchesTheSpecWorkedExample pins §6b's arithmetic to
// the number §8c prints: 90 days is 15188 units of 512 seconds.
func TestComposerDaysToUnitsMatchesTheSpecWorkedExample(t *testing.T) {
	if got := composerDaysToUnits(90); got != 15188 {
		t.Errorf("composerDaysToUnits(90) = %d, want 15188 (ceil(90*86400/512))", got)
	}
	if got := composerDaysToUnits(1); got != 169 {
		t.Errorf("composerDaysToUnits(1) = %d, want 169 (ceil(86400/512))", got)
	}
	// The CEILING never rounds down: a day that rounded down would encode a
	// lock shorter than the operator asked for.
	for d := uint32(1); d <= 388; d++ {
		u := composerDaysToUnits(d)
		if uint64(u)*512 < uint64(d)*86400 {
			t.Fatalf("%d days encodes %d units = %d s, short of %d s", d, u, uint64(u)*512, uint64(d)*86400)
		}
		if u == 0 || u > 65535 {
			t.Fatalf("%d days encodes %d units, outside §4c's 1..=65535", d, u)
		}
	}
}

// TestComposerLockCheckRefusesEverySection4cBoundary is §12 item 7: every
// boundary value in and out, per kind, against the DEVICE's gate.
func TestComposerLockCheckRefusesEverySection4cBoundary(t *testing.T) {
	for _, tc := range []struct {
		name string
		l    md.Lock
		ok   bool
	}{
		{"blocks, one", md.Lock{Kind: md.LockOlderBlocks, Value: 1}, true},
		{"blocks, max", md.Lock{Kind: md.LockOlderBlocks, Value: 65535}, true},
		{"blocks, zero", md.Lock{Kind: md.LockOlderBlocks, Value: 0}, false},
		{"blocks, over", md.Lock{Kind: md.LockOlderBlocks, Value: 65536}, false},
		{"units, one", md.Lock{Kind: md.LockOlderUnits, Value: 1}, true},
		{"units, max", md.Lock{Kind: md.LockOlderUnits, Value: 65535}, true},
		// ZERO UNITS is the one md itself still accepts (older(0x400000), the
		// filed md-older-zero-time-units-not-refused), and BIP-68 line 46 says
		// zero units is NO LOCK. §4c makes the device refuse it independently,
		// which is the whole point of §12 item 7.
		{"units, zero", md.Lock{Kind: md.LockOlderUnits, Value: 0}, false},
		{"units, over", md.Lock{Kind: md.LockOlderUnits, Value: 65536}, false},
		{"height, one", md.Lock{Kind: md.LockAfterHeight, Value: 1}, true},
		{"height, max", md.Lock{Kind: md.LockAfterHeight, Value: 499_999_999}, true},
		{"height, zero", md.Lock{Kind: md.LockAfterHeight, Value: 0}, false},
		{"height, over", md.Lock{Kind: md.LockAfterHeight, Value: 500_000_000}, false},
		{"time, floor", md.Lock{Kind: md.LockAfterTime, Value: 500_000_000}, true},
		{"time, max", md.Lock{Kind: md.LockAfterTime, Value: 2_147_483_647}, true},
		{"time, under", md.Lock{Kind: md.LockAfterTime, Value: 499_999_999}, false},
		{"time, over", md.Lock{Kind: md.LockAfterTime, Value: 2_147_483_648}, false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.l.Check()
			if (err == nil) != tc.ok {
				t.Errorf("Lock{%v, %d}.Check() = %v, want ok=%v", tc.l.Kind, tc.l.Value, err, tc.ok)
			}
		})
	}
}

// TestComposerDateEntryRefusesImpossibleAndPre2009Dates is §6b's entry band
// and §8t.
func TestComposerDateEntryRefusesImpossibleAndPre2009Dates(t *testing.T) {
	for _, tc := range []struct {
		digits string
		ok     bool
		why    string
	}{
		{"20270301", true, "the §8c worked example"},
		{"20090103", true, "the floor itself is admitted"},
		{"20090102", false, "one day below the floor (§8t)"},
		{"20081231", false, "before 2009 (§8t)"},
		{"20270231", false, "2027-02-31 does not exist; time.Date would normalise it to March"},
		{"20271301", false, "month 13"},
		{"20270000", false, "day and month zero"},
		{"20380120", false, "past 2038-01-19, the Unix-time ceiling §4c's time row stops at"},
	} {
		t.Run(tc.digits, func(t *testing.T) {
			y, m, d, ok := composerParseDateDigits(tc.digits)
			if !ok {
				if tc.ok {
					t.Fatalf("%s: the digits did not parse but should have (%s)", tc.digits, tc.why)
				}
				return
			}
			unix, inBand := composerDateToUnix(y, m, d)
			if inBand != tc.ok {
				t.Fatalf("%s -> %04d-%02d-%02d, in band = %v, want %v (%s)",
					tc.digits, y, m, d, inBand, tc.ok, tc.why)
			}
			if !inBand {
				return
			}
			if err := (md.Lock{Kind: md.LockAfterTime, Value: unix}).Check(); err != nil {
				t.Errorf("%s produced %d, which the device's own §4c gate refuses: %v",
					tc.digits, unix, err)
			}
		})
	}
}

// TestComposerLockEchoesAreTheSpecStrings is §8c under the verbatim rule and
// §12 item 5's condition test for the bound and no-bound lines.
func TestComposerLockEchoesAreTheSpecStrings(t *testing.T) {
	// 1788220800 is 2026-09-01 00:00:00 UTC and 1803859200 is 2027-03-01,
	// both MEASURED (`python3 -c "import datetime; ..."`) rather than
	// transcribed: an epoch off by a day reads exactly like a correct one.
	packed := composerBound{seconds: 1788220800, hasBound: true}
	withHeight := composerBound{seconds: 1788220800, height: 905000, hasBound: true, hasHeight: true}
	none := composerBound{}

	got := strings.Join(composerLockEcho(md.Lock{Kind: md.LockOlderUnits, Value: 15188}, none), " ")
	if !strings.Contains(got, "90 days = 15188 units of 512 s (90.0 days)") {
		t.Errorf("the relative-time echo is not §8c's: %q", got)
	}
	// A RELATIVE lock is bounded by nothing: `now:` bounds dates and heights,
	// which are absolute. The bare disclaimer must not appear on it either --
	// there is nothing about the present for it to disclaim.
	if strings.Contains(got, "cannot tell the time") {
		t.Errorf("a relative lock carries a present-tense disclaimer it has no use for: %q", got)
	}

	got = strings.Join(composerLockEcho(md.Lock{Kind: md.LockAfterTime, Value: 1803859200}, packed), " ")
	if !strings.Contains(got, "2027-03-01 00:00 UTC") {
		t.Errorf("the date echo is not §8c's: %q", got)
	}
	if !strings.Contains(got, composerCopyPackedDateBound("2026-09-01")) {
		t.Errorf("the date echo does not carry the packed-date bound line: %q", got)
	}

	got = strings.Join(composerLockEcho(md.Lock{Kind: md.LockAfterHeight, Value: 905001}, withHeight), " ")
	if !strings.Contains(got, "Block 905001") {
		t.Errorf("the height echo is not §8c's: %q", got)
	}
	if !strings.Contains(got, composerCopyPackedHeightBound(905000)) {
		t.Errorf("the height echo does not carry the packed-height bound line: %q", got)
	}

	// NO now: FIELD FOR THIS KIND -> the bare disclaimer, never silence.
	got = strings.Join(composerLockEcho(md.Lock{Kind: md.LockAfterHeight, Value: 905001}, packed), " ")
	if !strings.Contains(got, composerCopyNoBound()) {
		t.Errorf("a height with a seconds-only bound must carry the BARE disclaimer: %q", got)
	}
}

// TestComposerBelowBoundRefusals is §6b's refusal and §8o, both directions,
// plus §12 item 5's fits assertions for the four refusal bodies.
func TestComposerBelowBoundRefusals(t *testing.T) {
	b := composerBound{seconds: 1788220800, height: 905000, hasBound: true, hasHeight: true}
	if composerLockBelowBound(md.Lock{Kind: md.LockAfterTime, Value: 1788220799}, b) == "" {
		t.Error("a date one second before the pack time is not refused")
	}
	if got := composerLockBelowBound(md.Lock{Kind: md.LockAfterTime, Value: 1788220801}, b); got != "" {
		t.Errorf("a date after the pack time is refused with %q", got)
	}
	if composerLockBelowBound(md.Lock{Kind: md.LockAfterHeight, Value: 904999}, b) == "" {
		t.Error("a height below the packed height is not refused")
	}
	// A field that is ABSENT bounds nothing (§6b).
	seconds := composerBound{seconds: 1788220800, hasBound: true}
	if got := composerLockBelowBound(md.Lock{Kind: md.LockAfterHeight, Value: 1}, seconds); got != "" {
		t.Errorf("a height was refused against a bound with no height field: %q", got)
	}
	for _, tc := range []struct {
		what string
		body string
	}{
		{"the §8o below-bound date refusal", composerCopyBelowBoundDate()},
		{"the §8o below-bound height refusal", composerCopyBelowBoundHeight()},
		{"the §8t date floor refusal", composerCopyDateFloor()},
		{"the §8u relative ceiling refusal", composerCopyRelativeCeiling()},
	} {
		assertModalBodyFits(t, tc.what, errorScreenBody, tc.body)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDaysToUnits|^TestComposerLockCheck|^TestComposerDateEntry|^TestComposerLockEchoes|^TestComposerBelowBound' ./gui/ 2>&1 | tail -8`
Expected: FAIL to build -- `undefined: composerDaysToUnits`, `composerParseDateDigits`, `composerDateToUnix`, `composerLockEcho`, `composerLockBelowBound`.

- [ ] **Step 3: Write the lock entry**

Create `gui/composer_lock.go`:

```go
package gui

import (
	"fmt"
	"strconv"
	"time"

	"seedhammer.com/md"
)

// Lock entry (SPEC §6b, C11, C24, C25): kind, then unit, then digits, then an
// echo the operator can check.
//
// THE OPERATOR NEVER TYPES A RAW OPERAND. They type a count of blocks, a
// count of days, a block height, or eight date digits; the encoding is
// computed here. A raw `older(4210836)` on a screen is a number no one can
// check, and §4c's four bands overlap in the operand space -- 26280 is
// equally `older(26280)` and `after(26280)` -- so the KIND has to come from
// the operator's choice rather than from the number.
//
// THE DEVICE ENFORCES §4c ITSELF (§4c, §9 item 3, §12 item 7). md today
// accepts `older(0x400000)` -- zero time units, which BIP-68 line 46 defines
// as no lock at all (filed md-older-zero-time-units-not-refused) -- so every
// value produced here goes through md.Lock.Check before it is stored, and the
// entry bounds are the same bands where the operator meets them.

// composerDaysToUnits is §6b's days-to-units conversion, rounding UP: a day
// that rounded down would encode a lock shorter than the operator asked for.
func composerDaysToUnits(days uint32) uint32 {
	return uint32((uint64(days)*86400 + 511) / 512)
}

// The date-entry band (§6b). The floor is NOT §4c's operand floor: any date
// whose 00:00 UTC value is below 500,000,000 encodes as a block HEIGHT rather
// than a time (1985-11-05 00:00 UTC is 499,996,800), so the entry stops at
// 2009-01-03 and says so with §8t. The ceiling is the Unix-time ceiling
// §4c's time row stops at.
const (
	composerDateFloorUnix   uint32 = 1230940800 // 2009-01-03 00:00:00 UTC
	composerDateCeilingUnix uint32 = 2147472000 // 2038-01-19 00:00:00 UTC
)

// composerParseDateDigits splits a YYYYMMDD field. The pad types no
// separators (§6b), so the field is fixed-width and its shape is checked here
// rather than by a parser that would accept "2027-3-1".
func composerParseDateDigits(s string) (y, m, d int, ok bool) {
	if len(s) != 8 {
		return 0, 0, 0, false
	}
	for _, r := range s {
		if r < '0' || r > '9' {
			return 0, 0, 0, false
		}
	}
	y, _ = strconv.Atoi(s[0:4])
	m, _ = strconv.Atoi(s[4:6])
	d, _ = strconv.Atoi(s[6:8])
	return y, m, d, true
}

// composerDateExists reports whether a calendar date is real.
//
// IT IS SEPARATE FROM THE BAND CHECK, and that separation is the fix for a
// live defect: composerDateToUnix returns (0, false) for EVERY failure --
// impossible date, below the floor, above the ceiling alike -- so a dispatch
// that tried to tell them apart by reading its uint32 result (`y > 2038 ||
// u == 0`) was a tautology, true on every failure, and "that date does not
// exist" was dead code. 2027-02-31 got the CEILING message, advising a block
// height for a date that will never exist at any height (F-458).
//
// The round trip is the test rather than a month-length table, for the reason
// composerDateToUnix states: time.Date NORMALISES, so 2027-02-31 becomes
// 2027-03-03 and would otherwise be silently accepted as a different date.
func composerDateExists(y, m, d int) bool {
	if m < 1 || m > 12 || d < 1 || d > 31 {
		return false
	}
	t := time.Date(y, time.Month(m), d, 0, 0, 0, 0, time.UTC)
	return t.Year() == y && int(t.Month()) == m && t.Day() == d
}

// composerDateToUnix converts a calendar date to its 00:00:00 UTC Unix time
// and reports whether it is inside the entry band.
//
// IMPOSSIBLE DATES ARE CAUGHT BY THE ROUND TRIP, not by a month-length table.
// time.Date NORMALISES: 2027-02-31 becomes 2027-03-03 and would otherwise be
// silently accepted as a different date than the one typed. Comparing the
// components back is exact and needs no leap-year rule of its own.
func composerDateToUnix(y, m, d int) (uint32, bool) {
	if !composerDateExists(y, m, d) {
		return 0, false
	}
	t := time.Date(y, time.Month(m), d, 0, 0, 0, 0, time.UTC)
	u := t.Unix()
	if u < int64(composerDateFloorUnix) || u > int64(composerDateCeilingUnix) {
		return 0, false
	}
	return uint32(u), true
}

// composerLockBoundLine is §6b's bound line: the pack date or height when the
// relevant now: field is present, the bare disclaimer when it is not, and
// NOTHING for a relative lock, which nothing about the present bounds.
//
// The copy never says "now" and never withdraws the disclaimer, because a
// stale now: record can only weaken the below-bound refusal, never invent
// one.
func composerLockBoundLine(b composerBound, kind md.LockKind) string {
	switch kind {
	case md.LockOlderBlocks, md.LockOlderUnits:
		return ""
	case md.LockAfterTime:
		if b.hasBound {
			return composerCopyPackedDateBound(b.packDate())
		}
	case md.LockAfterHeight:
		if b.hasHeight {
			return composerCopyPackedHeightBound(b.height)
		}
	}
	return composerCopyNoBound()
}

// composerLockEcho is what the operator reads back: §8c's echo for the kind,
// then the bound line.
func composerLockEcho(l md.Lock, b composerBound) []string {
	var head string
	switch l.Kind {
	case md.LockOlderBlocks:
		head = composerCopyLockEchoBlocks(l.Value)
	case md.LockOlderUnits:
		head = composerCopyLockEchoDays(composerUnitsToDays(l.Value), l.Value)
	case md.LockAfterHeight:
		head = composerCopyLockEchoHeight(l.Value)
	case md.LockAfterTime:
		t := time.Unix(int64(l.Value), 0).UTC()
		head = composerCopyLockEchoDate(t.Year(), int(t.Month()), t.Day())
	}
	out := []string{head}
	if line := composerLockBoundLine(b, l.Kind); line != "" {
		out = append(out, line)
	}
	return out
}

// composerLockBelowBound returns the §8o body when the lock is below the
// payload's bound, or "" when it is not. A field that is ABSENT bounds
// nothing (§6b), which is why each arm checks its own presence flag.
func composerLockBelowBound(l md.Lock, b composerBound) string {
	switch l.Kind {
	case md.LockAfterTime:
		if b.hasBound && l.Value < b.seconds {
			return composerCopyBelowBoundDate()
		}
	case md.LockAfterHeight:
		if b.hasHeight && l.Value < b.height {
			return composerCopyBelowBoundHeight()
		}
	}
	return ""
}

// composerLockAccept is the ONE gate every entered lock passes: §4c through
// md.Lock.Check, then the payload bound. Both refusals name what to do
// instead and print no encoding (§11).
func composerLockAccept(ctx *Context, th *Colors, l md.Lock, b composerBound) bool {
	if err := l.Check(); err != nil {
		// Reached only by a bound this file and md disagree about, which is a
		// defect rather than an operator error -- so it says so instead of
		// showing a §8 line that would misdescribe it.
		showError(ctx, th, "Time lock", "This device will not write that lock value.")
		return false
	}
	if body := composerLockBelowBound(l, b); body != "" {
		showError(ctx, th, "Time lock", body)
		return false
	}
	return true
}

// composerLockEdit is §6b's kind, unit, digits, echo.
func composerLockEdit(ctx *Context, th *Colors, st *composerState, idx int) bool {
	title := fmt.Sprintf("Path %d lock", idx+1)
	kindCS := &ChoiceScreen{
		Title:   title,
		Lead:    "What kind of time lock?",
		Choices: []string{"None", "After a wait", "After a date or height"},
	}
	kindSel, ok := kindCS.Choose(ctx, th)
	if !ok {
		return false
	}
	if kindSel == 0 {
		st.list.Paths[idx].Lock = nil
		return true
	}

	var lock md.Lock
	if kindSel == 1 {
		unitCS := &ChoiceScreen{Title: title, Lead: "Measured how?", Choices: []string{"Blocks", "Days"}}
		unitSel, ok := unitCS.Choose(ctx, th)
		if !ok {
			return false
		}
		if unitSel == 0 {
			frag, ok := composerDigitEntry(ctx, th, title, "How many blocks?", 5, func(s string) (string, bool) {
				// AN EMPTY FIELD IS NOT A CEILING BREACH. Returning §8u for
				// every unparseable fragment meant the operator read
				// "Relative locks reach at most 455 days..." before typing a
				// digit -- a refusal for a limit they had not approached. The
				// date and height pads already say what to type first.
				if s == "" {
					return "1 to 65535 blocks", false
				}
				n, err := strconv.ParseUint(s, 10, 32)
				if err != nil || n < 1 || n > 65535 {
					return composerCopyRelativeCeiling(), false
				}
				return composerCopyLockEchoBlocks(uint32(n)), true
			})
			if !ok {
				return false
			}
			n, _ := strconv.ParseUint(frag, 10, 32)
			lock = md.Lock{Kind: md.LockOlderBlocks, Value: uint32(n)}
		} else {
			frag, ok := composerDigitEntry(ctx, th, title, "How many days?", 3, func(s string) (string, bool) {
				if s == "" {
					return "1 to 388 days", false
				}
				n, err := strconv.ParseUint(s, 10, 32)
				if err != nil || n < 1 || n > 388 {
					return composerCopyRelativeCeiling(), false
				}
				return composerCopyLockEchoDays(uint32(n), composerDaysToUnits(uint32(n))), true
			})
			if !ok {
				return false
			}
			n, _ := strconv.ParseUint(frag, 10, 32)
			lock = md.Lock{Kind: md.LockOlderUnits, Value: composerDaysToUnits(uint32(n))}
		}
	} else {
		absCS := &ChoiceScreen{Title: title, Lead: "Named how?", Choices: []string{"A date", "A block height"}}
		absSel, ok := absCS.Choose(ctx, th)
		if !ok {
			return false
		}
		if absSel == 0 {
			frag, ok := composerDigitEntry(ctx, th, title, "Date as YYYYMMDD", 8, func(s string) (string, bool) {
				y, m, d, parsed := composerParseDateDigits(s)
				if !parsed {
					return "eight digits, YYYYMMDD", false
				}
				_, inBand := composerDateToUnix(y, m, d)
				if !inBand {
					// THE THREE FAILURES ARE TOLD APART BY WHAT THEY ARE, not
					// by reading the returned operand. `u` is 0 on every
					// failure path, so `y > 2038 || u == 0` was a tautology and
					// "that date does not exist" was dead code -- 2027-02-31
					// got the CEILING message, advising a block height for a
					// date that no height makes real (F-458).
					if !composerDateExists(y, m, d) {
						return "that date does not exist", false
					}
					if y < 2009 {
						return composerCopyDateFloor(), false
					}
					// A DATE PAST THE CEILING EXISTS; the build will not write
					// it as a TIME lock. §4d's first archetype,
					// simple-timelocked-inheritance, is exactly where a
					// twenty-year date is the ordinary case, so the operator is
					// told the limit and the alternative rather than being left
					// to retype.
					return composerCopyDateCeiling(), false
				}
				// THE RAW OPERAND IS NOT SHOWN. §6b's premise is that the
				// operator never types one, and the echo screen after this
				// prints §8c's clean form, so the number added nothing anyone
				// could check.
				return composerCopyLockEchoDate(y, m, d), true
			})
			if !ok {
				return false
			}
			y, m, d, _ := composerParseDateDigits(frag)
			u, _ := composerDateToUnix(y, m, d)
			lock = md.Lock{Kind: md.LockAfterTime, Value: u}
		} else {
			frag, ok := composerDigitEntry(ctx, th, title, "Block height", 9, func(s string) (string, bool) {
				n, err := strconv.ParseUint(s, 10, 64)
				if err != nil || n < 1 || n > 499_999_999 {
					return "1 to 499999999", false
				}
				return composerCopyLockEchoHeight(uint32(n)), true
			})
			if !ok {
				return false
			}
			n, _ := strconv.ParseUint(frag, 10, 64)
			lock = md.Lock{Kind: md.LockAfterHeight, Value: uint32(n)}
		}
	}

	if !composerLockAccept(ctx, th, lock, st.bound) {
		return false
	}
	// THE ECHO IS A CONFIRM, not a notice: §6b's "kind, unit, digits, echo"
	// ends with the operator reading it back, and Back here discards the lock
	// rather than storing an operand nobody agreed to.
	if !composerReadScreen(ctx, th, title, composerLockEcho(lock, st.bound)) {
		return false
	}
	st.list.Paths[idx].Lock = &lock
	return true
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDaysToUnits|^TestComposerLockCheck|^TestComposerDateEntry|^TestComposerLockEchoes|^TestComposerBelowBound' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: five top-level PASS, 16 sub-tests under the §4c boundary test and 8 under the date test; `ok seedhammer.com/gui`. The four `assertModalBodyFits` calls log their headroom -- record the numbers.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_lock.go gui/composer_lock_test.go
git commit -s -F - <<'MSG'
gui: lock entry -- kind, unit, digits, echo, and the device's own section 4c gate (composer S3 task A7)

The operator never types a raw operand: section 4c's four bands overlap in
operand space, so the kind comes from their choice. md.Lock.Check runs on
every value before it is stored, because md itself still accepts
older(0x400000), which BIP-68 line 46 defines as no lock. Impossible dates
are caught by a time.Date round trip rather than a month-length table.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
### Task A8: `gui/composer_hash.go` -- hashlock entry from a `hash:` record or 64 typed hex

**Files:**
- Create: `gui/composer_hash.go`
- Test: `gui/composer_hash_test.go`

**Interfaces:**
- Consumes: `sysw.ParseHashRecord(record string) ([32]byte, error)` and `sysw.ClassHash` (S2); `syswSession.records` (`gui/sysw_session.go:53`); `NewKeyboard` (`gui/gui.go:1463`); `composerPickScreen` (Task A2); `ChoiceScreen`; `showError`; `composerCopyHashRule`.
- Produces: `const composerHexKeys`; `func composerHashRow(i int, digest [32]byte) string`; `func composerPayloadDigests(s *syswSession) [][32]byte`; `func composerHexEntry(ctx *Context, th *Colors) ([32]byte, bool)`; `func composerHashEdit(ctx *Context, th *Colors, st *composerState, idx int) bool`.

§6c makes the payload's `hash:` records the PRIMARY source and typed hex the fallback, and requires the 32-byte rule (§8i) to be stated at entry and again at consent. The row form is §6c's: `hash <i>  <first 8>..<last 8>`, which fits the 436 px label budget where a 64-hex row would be cut rather than wrapped.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_hash_test.go`:

```go
package gui

import (
	"encoding/hex"
	"strings"
	"testing"
)

func TestComposerHashRowIsShortEnoughToDraw(t *testing.T) {
	var d [32]byte
	raw, _ := hex.DecodeString("0123456789abcdeffedcba98765432100123456789abcdeffedcba9876543210")
	copy(d[:], raw)
	got := composerHashRow(1, d)
	if !strings.HasPrefix(got, "hash 1  0123456789abcdef"[:8]) {
		t.Errorf("the row does not lead with the index and the digest head: %q", got)
	}
	if !strings.Contains(got, "..") {
		t.Errorf("the row does not elide the middle, so a 64-hex line would be cut: %q", got)
	}
	if len(got) > 32 {
		t.Errorf("the row is %d characters; §6c budgets about 28 so it draws inside the "+
			"436 px label rather than being cut", len(got))
	}
	assertChoiceLabelFits(t, got)
}

// TestComposerPayloadDigestsTakesOnlyWellFormedHashRecords: a malformed
// hash: record is ClassUnknown and inert (§6a), so it must not appear on the
// pick list -- and it changes no count but the not-understood one.
func TestComposerPayloadDigestsTakesOnlyWellFormedHashRecords(t *testing.T) {
	s := composerSessionWith([]string{
		composerTestHashRecord,
		"hash:00",             // 1 byte, not 32
		composerTestKeyRecord, // a different class entirely
	}, nil)
	got := composerPayloadDigests(s)
	if len(got) != 1 {
		t.Fatalf("composerPayloadDigests returned %d digests, want 1", len(got))
	}
}

// TestComposerHashRuleIsStatedAtEntry is §8i's fires-on-condition test and
// its fits assertion. The reference wallet's own README records months lost
// to hashing a passphrase directly, which is exactly what this line prevents.
func TestComposerHashRuleIsStatedAtEntry(t *testing.T) {
	assertModalBodyFits(t, "the §8i 32-byte preimage rule", errorScreenBody, composerCopyHashRule())
	if !strings.Contains(composerCopyHashRule(), "32-byte") {
		t.Error("the §8i line does not state the size the preimage must be")
	}
	if !strings.Contains(composerCopyHashRule(), "never be spent") {
		t.Error("the §8i line does not state the consequence of getting it wrong")
	}
}

func TestComposerHexKeysAreHexAndNothingElse(t *testing.T) {
	for _, r := range composerHexKeys {
		if r == '\n' {
			continue
		}
		if !strings.ContainsRune("0123456789abcdef", r) {
			t.Errorf("the hex pad offers %q, which is not a hex digit; §6c accepts a "+
				"digest only when exactly 64 valid hex characters are present", r)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerHash|^TestComposerPayloadDigests|^TestComposerHexKeys' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerHashRow`, `composerPayloadDigests`, `composerHexKeys`.

- [ ] **Step 3: Write the hashlock entry**

Create `gui/composer_hash.go`:

```go
package gui

import (
	"encoding/hex"
	"fmt"
	"image"

	"seedhammer.com/gui/assets"
	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
	"seedhammer.com/sysw"
)

// Hashlock entry (SPEC §6c, C25).
//
// THE PAYLOAD IS THE PRIMARY SOURCE and typing is the fallback, because a
// 64-character hex digest typed on a four-button device is a transcription
// with no checksum behind it. A hash: record was checked on the host.
//
// THE 32-BYTE RULE IS STATED AT ENTRY AND AGAIN AT CONSENT (§8i). sha256(H)
// compiles to OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL, so the
// preimage MUST be exactly 32 bytes: a digest of a passphrase directly can
// never be spent, and the reference wallet's own README records months of
// exactly that.
//
// THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this cycle
// (§14). It takes a digest and puts it in a script.

// composerHexKeys is the fallback pad's alphabet: hex digits only, so an
// entry that is 64 characters long is 64 VALID characters by construction.
const composerHexKeys = "0123456789\nabcdef"

// composerHashRow is §6c's row form: `hash <i>  <first 8>..<last 8>`, in the
// host's pack order. A full 64-hex row would be CUT rather than wrapped at
// the 436 px label budget, and a cut digest is worse than an elided one --
// the operator cannot tell which end is missing.
func composerHashRow(i int, digest [32]byte) string {
	h := hex.EncodeToString(digest[:])
	return fmt.Sprintf("hash %d  %s..%s", i, h[:8], h[56:])
}

// composerPayloadDigests returns every well-formed hash: record, in payload
// order. A malformed one is ClassUnknown and INERT under the shipped contract
// (sysw/descriptor.go:46-48): it reaches no screen, and its only device-side
// signal is the door's not-understood count (§6a).
func composerPayloadDigests(s *syswSession) [][32]byte {
	if s == nil || !s.loaded {
		return nil
	}
	var out [][32]byte
	for _, r := range s.records {
		if r.class != sysw.ClassHash {
			continue
		}
		d, err := sysw.ParseHashRecord(r.body)
		if err != nil {
			// Unreachable: a record that does not parse is not ClassHash. The
			// arm exists so no value is consumed from a call that errored.
			continue
		}
		out = append(out, d)
	}
	return out
}

// composerHexEntry is the fallback: 64 hex characters, accepted only when
// exactly 64 are present (§6c).
func composerHexEntry(ctx *Context, th *Colors) ([32]byte, bool) {
	var out [32]byte
	kbd := NewKeyboard(ctx, composerHexKeys)
	backBtn := &Clickable{Button: Button1}
	okBtn := &Clickable{Button: Button3}
	for !ctx.Done {
		for kbd.Update(ctx) {
		}
		if len(kbd.Fragment) > 64 {
			kbd.Fragment = kbd.Fragment[:64]
		}
		frag := kbd.Fragment
		valid := len(frag) == 64
		if backBtn.Clicked(ctx) {
			return out, false
		}
		clicked := okBtn.Clicked(ctx)
		if valid && clicked {
			raw, err := hex.DecodeString(frag)
			if err != nil || len(raw) != 32 {
				// The pad offers hex alone, so this is unreachable; it refuses
				// rather than returning a zero digest, because a silently zero
				// hashlock is spendable by anyone who knows the preimage of
				// zero.
				showError(ctx, th, "Hash lock", "That is not a 32-byte digest.")
				continue
			}
			copy(out[:], raw)
			return out, true
		}

		dims := ctx.Platform.DisplaySize()
		screen := layout.Rectangle{Max: dims}
		_, content := screen.CutTop(leadingSize)
		content, _ = content.CutBottom(8)
		kbdOp, kbdsz := kbd.Layout(ctx, th)
		kbdOp = kbdOp.Offset(content.S(kbdsz))

		shown := frag
		if shown == "" {
			shown = " "
		}
		word, frgSize := widget.Labelw(&ctx.B, ctx.Styles.word, dims.X-50, th.Background, shown)
		r := image.Rectangle{Max: frgSize}
		r.Min.Y -= 3
		r.Max.Y += buttonPadY
		r.Min.X -= buttonPadX
		r.Max.X += buttonPadX
		top, _ := content.CutBottom(kbdsz.Y)
		wordOff := top.Center(frgSize)
		word = op.Layer(word, op.Compose(
			op.Color(&ctx.B, th.Text),
			op.RoundedRect2(&ctx.B, r, cornerRadius),
		)).Offset(wordOff)

		count, csz := widget.Label(&ctx.B, ctx.Styles.body, th.Text,
			fmt.Sprintf("%d of 64 hex", len(frag)))
		countOp := count.Offset(image.Pt((dims.X-csz.X)/2, wordOff.Y+frgSize.Y+8))

		navBtns := []NavButton{{Clickable: backBtn, Style: StyleSecondary, Icon: assets.IconBack}}
		if valid {
			navBtns = append(navBtns, NavButton{Clickable: okBtn, Style: StylePrimary, Icon: assets.IconCheckmark})
		}
		nav, _ := layoutNavigation(&ctx.B, th, dims, navBtns...)
		titleOp, _ := layoutTitle(ctx, dims.X, th.Text, "Hash lock")
		ctx.Frame(op.Layer(kbdOp, word, countOp, nav, titleOp, op.Color(&ctx.B, th.Background)))
	}
	return out, false
}

// composerHashEdit sets or clears one path's hashlock.
func composerHashEdit(ctx *Context, th *Colors, st *composerState, idx int) bool {
	title := fmt.Sprintf("Path %d hash", idx+1)
	digests := composerPayloadDigests(ctx.sysw)
	rows := make([]string, 0, len(digests)+2)
	for i, d := range digests {
		rows = append(rows, composerHashRow(i+1, d))
	}
	rows = append(rows, "Type 64 hex")
	rows = append(rows, "No hash lock")
	sel, ok := composerPickScreen(ctx, th, title, "Which hash?", rows)
	if !ok {
		return false
	}
	// §8i, ONCE THE OPERATOR IS ACTUALLY TAKING A HASH. It used to be shown
	// unconditionally on entry, so an operator whose next choice was "No hash
	// lock" met a modal in front of a clear. It governs how the preimage must
	// have been produced, which is only a question for someone choosing one.
	if sel <= len(digests) {
		showError(ctx, th, title, composerCopyHashRule())
	}
	switch {
	case sel < len(digests):
		d := digests[sel]
		st.list.Paths[idx].Hash = &d
		return true
	case sel == len(digests):
		d, ok := composerHexEntry(ctx, th)
		if !ok {
			return false
		}
		st.list.Paths[idx].Hash = &d
		return true
	default:
		st.list.Paths[idx].Hash = nil
		return true
	}
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerHash|^TestComposerPayloadDigests|^TestComposerHexKeys' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: four PASS; `ok seedhammer.com/gui`. **The shape flow's own tests now compile and run for the first time** -- run the shape task's Run line too and record its two headroom numbers.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_hash.go gui/composer_hash_test.go
git commit -s -F - <<'MSG'
gui: hashlock entry from a payload record or 64 typed hex, with the 32-byte rule at entry (composer S3 task A8)

The payload is primary: a 64-character digest typed on four buttons has no
checksum behind it. The row form elides the middle because a full 64-hex row
is cut rather than wrapped, and a cut digest hides which end is missing.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task A9: `gui/composer_stub.go` -- the paged stub-teaching screen, and the relabelling on the shipped screen

**Files:**
- Create: `gui/composer_stub.go`
- Test: `gui/composer_stub_test.go`
- Modify (FRAGMENT): `gui/template_engrave.go:70` and `:79` (both `Template-ID:` occurrences)

**Interfaces:**
- Consumes: `md.FormAwareIdChunks(strs) ([16]byte, WalletIdKind, error)` (`md/template_id.go:163`), `md.FormAwareStubChunks(strs) ([4]byte, error)` (`md/template_id.go:122`), `md.ExpandWalletPolicyChunks(strs) (Template, []ExpandedKey, error)` (`md/expand.go:102`) and `md.ExpandedKey{Index, OriginPath, Fingerprint, FingerprintPresent, XpubPresent}` (`md/expand.go:84-95`); `composerReadScreen` (Task A2); `composerCopyOwnWallet`, `composerCopyIdChanged`.
- Produces: `func composerStubLines(templateChunks, keyedChunks []string, changed bool) ([]string, error)`; `func composerStubFlow(ctx *Context, th *Colors, templateChunks, keyedChunks []string, changed bool) bool`.

**The per-slot origins come from the DECODED chunks, not from state.** `ExpandWalletPolicyChunks` resolves each slot's origin through the same precedence the consumer path uses (`md/expand.go:115-135`), so the "expects a key at" line the operator writes down is the origin a card will actually be matched against by `slotMatchesCard` (`gui/key_card_seating.go:128`). Reading it off `composerState` instead would print a promise rather than a fact.

**It is a PAGED widget with a stated per-frame budget** (§7c, §9 item 6): the body grows one line per slot and the grammar admits 32, so a fixed header plus 32 rows cannot be one frame. It uses `composerReadScreen`, whose capacity is `composerPageLines`' measured `shown`.

**Re-shown after EVERY shape edit** (§7c). The template id is key-independent and origin-invariant but NOT shape-invariant -- the wrapper, the path list, every lock operand and every hash digest enter it -- so `changed` adds §8s's first body, which tells the operator that cards minted with the old stub will not seat here (`gui/key_card_seating.go:63-73`, layer 1's `errSeatNotThisPolicy`).

- [ ] **Step 1: Write the failing test**

Create `gui/composer_stub_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

// composerTemplateChunks composes a two-path wsh template with no keys, the
// shape Part A's exit engraves.
func composerTemplateChunks(t *testing.T) []string {
	t.Helper()
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 1000}},
	}}
	c, err := md.Compose(list)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatalf("Chunks: %v", err)
	}
	return chunks
}

// TestComposerStubLinesTeachTheStubAndTheOrigins is §7c: the labels are
// LITERAL, the mk encode command is present, §8d is present, and every
// unseated slot names the origin a card must declare to seat there.
func TestComposerStubLinesTeachTheStubAndTheOrigins(t *testing.T) {
	chunks := composerTemplateChunks(t)
	lines, err := composerStubLines(chunks, nil, false)
	if err != nil {
		t.Fatalf("composerStubLines: %v", err)
	}
	joined := strings.Join(lines, "\n")
	for _, want := range []string{
		"Template-ID:",
		"mk1 stub (template):",
		"mk encode --xpub",
		"--policy-id-stub",
		composerCopyOwnWallet(),
	} {
		if !strings.Contains(joined, want) {
			t.Errorf("the stub screen does not say %q:\n%s", want, joined)
		}
	}
	// FOUR SLOTS, FOUR EXPECTED-ORIGIN LINES, each at a DISTINCT account: the
	// §4f invariant is what makes the template seatable at all.
	_, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatal(err)
	}
	seen := map[string]bool{}
	for _, k := range keys {
		line := k.OriginPath.String()
		if !strings.Contains(joined, line) {
			t.Errorf("no line names slot @%d's expected origin %s:\n%s", k.Index, line, joined)
		}
		if seen[line] {
			t.Errorf("two slots declare the same origin %s with no fingerprints, which "+
				"errSeatSlotContested makes unseatable (§4f's invariant)", line)
		}
		seen[line] = true
	}
	if len(keys) != 4 {
		t.Fatalf("the fixture has %d slots, want 4", len(keys))
	}
}

// TestComposerStubScreenSaysTheIdChangedAfterAnEdit is §8s's first body and
// §12 item 5's condition test for it.
func TestComposerStubScreenSaysTheIdChangedAfterAnEdit(t *testing.T) {
	chunks := composerTemplateChunks(t)
	fresh, err := composerStubLines(chunks, nil, false)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(strings.Join(fresh, "\n"), composerCopyIdChanged()) {
		t.Error("a first showing carries the changed-id line, which would be false")
	}
	after, err := composerStubLines(chunks, nil, true)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(strings.Join(after, "\n"), composerCopyIdChanged()) {
		t.Error("a re-show after an edit does not carry §8s's changed-id line")
	}
	assertModalBodyFits(t, "the §8s changed-id line", errorScreenBody, composerCopyIdChanged())
	assertModalBodyFits(t, "the §8d own-wallet line", errorScreenBody, composerCopyOwnWallet())
}

// TestComposerStubScreenIsPagedAtItsMeasuredCapacity is §12 item 5's rule for
// a variable-length screen: assert the PAGING, since a fits assertion cannot
// pin a body with no single source string.
func TestComposerStubScreenIsPagedAtItsMeasuredCapacity(t *testing.T) {
	// A 32-slot template: the grammar's maximum, and the case the screen
	// exists to survive.
	list := md.PathList{Wrapper: md.ComposeWsh}
	for i := 0; i < 4; i++ {
		list.Paths = append(list.Paths, md.SpendPath{Keys: &md.KeySet{K: 1, N: 8}})
	}
	c, err := md.Compose(list)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	lines, err := composerStubLines(chunks, nil, false)
	if err != nil {
		t.Fatal(err)
	}
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	_, shown := composerPageLines(ctx, &descriptorTheme, sh2DisplaySize, lines, 0, -1)
	if shown >= len(lines) {
		t.Fatalf("a 32-slot stub screen claims all %d lines fit one frame; it must page", len(lines))
	}
	t.Logf("stub screen: %d lines for 32 slots, %d per frame, %d pages",
		len(lines), shown, (len(lines)+shown-1)/shown)
	// And the LAST page is reachable: paging forward by the reported count
	// terminates rather than looping short of the end.
	start, pages := 0, 0
	for start < len(lines) && pages < 64 {
		_, n := composerPageLines(ctx, &descriptorTheme, sh2DisplaySize, lines, start, -1)
		if n == 0 {
			t.Fatalf("paging stalled at line %d: composerPageLines drew nothing", start)
		}
		start += n
		pages++
	}
	if start < len(lines) {
		t.Errorf("paging reached line %d of %d before the page cap; the tail is unreachable",
			start, len(lines))
	}
	// It draws.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerStubFlow(ctx, &descriptorTheme, chunks, nil, false)
		})
		defer quit()
		content, ok := frame()
		if !ok {
			t.Fatal("the stub screen never drew")
		}
		assertFrameHasBody(t, ink(), "the composer stub-teaching screen")
		if !uiContains(content, "mk1 stub (template):") {
			t.Errorf("the first frame does not carry the stub label.\nFrame: %q", content)
		}
	})
}

// TestComposerTemplateEngraveScreenUsesTheStubLabel pins the §7c relabelling
// on the SHIPPED screen: its 4-byte value is a STUB, and calling it
// "Template-ID" beside a 16-byte id of the same name is how an operator comes
// to compare the wrong one against a coordinator.
func TestComposerTemplateEngraveScreenUsesTheStubLabel(t *testing.T) {
	lines := templateConsentLines(md.Template{N: 3, Renderable: true, Policy: md.PolicySortedMulti, K: 2},
		[4]byte{0xde, 0xad, 0xbe, 0xef}, 0, md.PolicyShape{})
	joined := strings.Join(lines, "\n")
	if !strings.Contains(joined, "mk1 stub (template): deadbeef") {
		t.Errorf("the template-engrave screen does not label its 4-byte value as a stub:\n%s", joined)
	}
	if strings.Contains(joined, "Template-ID: deadbeef") {
		t.Errorf("the template-engrave screen still calls a 4-byte stub Template-ID, which "+
			"is the label md.WalletIdKind gives the 16-byte id:\n%s", joined)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerStub|^TestComposerTemplateEngraveScreen' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerStubLines`, `composerStubFlow`; then, once built, the relabelling test fails with the shipped `Template-ID: deadbeef`.

- [ ] **Step 3: Write the stub screen**

Create `gui/composer_stub.go`:

```go
package gui

import (
	"fmt"

	"seedhammer.com/md"
)

// The stub-teaching screen (SPEC §7c, C9, §9 item 6).
//
// SHOWN UNCONDITIONALLY once the shape is complete, and RE-SHOWN after any
// shape edit. The template id is key-independent and origin-invariant but NOT
// shape-invariant: the wrapper, the path list, every lock operand and every
// hash digest enter it, so an operator who wrote the stub down and then
// changed a digit is holding a stub that will not seat. §8s says so, and
// gui/key_card_seating.go:63-73 is why it matters -- layer 1 refuses a card whose
// stub set does not include this template's, before any origin is compared.
//
// THE ORIGINS COME FROM THE DECODED CHUNKS. ExpandWalletPolicyChunks resolves
// each slot's origin through the same precedence the consuming path uses
// (md/expand.go:115-135), so the "expects a key at" line the operator writes
// down is the origin slotMatchesCard will actually compare a card against.
// Reading it off composerState would print a promise instead of a fact.
//
// PAGED, because the body grows one line per slot and the grammar admits 32.

// composerStubLines builds the screen. `keyedChunks` is nil until a policy
// has been seated; when present the keyed id and stub are added and the
// screen recommends stamping BOTH (--policy-id-stub is repeatable).
func composerStubLines(templateChunks, keyedChunks []string, changed bool) ([]string, error) {
	tid, tkind, err := md.FormAwareIdChunks(templateChunks)
	if err != nil {
		return nil, err
	}
	tstub, err := md.FormAwareStubChunks(templateChunks)
	if err != nil {
		return nil, err
	}
	_, keys, err := md.ExpandWalletPolicyChunks(templateChunks)
	if err != nil {
		return nil, err
	}

	var lines []string
	if changed {
		lines = append(lines, composerCopyIdChanged(), "")
	}
	// The LABELS ARE LITERAL (§7c): "Template-ID:" and "Policy-ID:" for the
	// 32-hex ids, "mk1 stub (template):" and "mk1 stub (policy):" for the
	// 8-hex stubs. tkind renders the first pair itself, so a template can
	// never be labelled with a policy's word.
	lines = append(lines,
		fmt.Sprintf("%s: %x", tkind, tid),
		fmt.Sprintf("mk1 stub (template): %x", tstub),
	)
	if len(keyedChunks) > 0 {
		kid, kkind, err := md.FormAwareIdChunks(keyedChunks)
		if err != nil {
			return nil, err
		}
		kstub, err := md.FormAwareStubChunks(keyedChunks)
		if err != nil {
			return nil, err
		}
		lines = append(lines,
			fmt.Sprintf("%s: %x", kkind, kid),
			fmt.Sprintf("mk1 stub (policy): %x", kstub),
			"Stamp BOTH stubs on each key card:",
			fmt.Sprintf("--policy-id-stub %x --policy-id-stub %x", tstub, kstub),
		)
	}
	lines = append(lines,
		"",
		"mk encode --xpub <xpub> --origin-fingerprint <fp>",
		fmt.Sprintf("  --origin-path <path> --policy-id-stub %x", tstub),
		"",
		composerCopyOwnWallet(),
		"",
	)
	// One line per slot. A slot that will stay UNSEATED names the origin a
	// card must declare; a seated one names the source's own declaration
	// instead (§7c).
	for _, k := range keys {
		if k.FingerprintPresent {
			lines = append(lines, fmt.Sprintf("Slot @%d: %x %s",
				k.Index, k.Fingerprint, k.OriginPath))
			continue
		}
		lines = append(lines, fmt.Sprintf("Slot @%d expects a key at %s",
			k.Index, k.OriginPath))
	}
	return lines, nil
}

// composerStubFlow shows the screen. Back returns false so the caller can
// send the operator back to the shape.
func composerStubFlow(ctx *Context, th *Colors, templateChunks, keyedChunks []string, changed bool) bool {
	lines, err := composerStubLines(templateChunks, keyedChunks, changed)
	if err != nil {
		showError(ctx, th, "Template", "Couldn't read back the template this device just built.")
		return false
	}
	return composerReadScreen(ctx, th, "Template", lines)
}
```

- [ ] **Step 4: Relabel the shipped template-engrave screen**

§7c requires the shipped screen's 4-byte value to be relabelled in the same change. It appears TWICE -- the classifiable branch and the complex branch -- so this is two edits, not one.

In `gui/template_engrave.go`, at both `:70` and `:79`, replace:

```go
			fmt.Sprintf("Template-ID: %x", templateID),
```

with:

```go
			// mk1 stub, NOT the wallet id. This value is the 4-byte
			// WDT-Id STUB (md.FormAwareStubChunks), and md.WalletIdKind
			// already prints "Template-ID" for the 16-byte id
			// (md/template_id.go:152) -- two different values under one
			// label is how an operator comes to compare the wrong one
			// against a coordinator and read a match as a mismatch.
			// SPEC_wallet_policy_composer.md §7c relabels it.
			fmt.Sprintf("mk1 stub (template): %x", templateID),
```

(The comment is written once above each occurrence; the second may carry a one-line back-reference instead of the whole paragraph.)

- [ ] **Step 5: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerStub|^TestComposerTemplateEngraveScreen|^TestTemplateConsent|^TestWalletPolicy' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: every listed test PASS -- including the SHIPPED `TestTemplateConsentLines` and the wallet-policy tests, which assert on `md.WalletIdKind`'s 16-byte label (`gui/wallet_policy_test.go:47,97`) and are unaffected by relabelling the 4-byte stub. If either shipped test fails, STOP: the relabelling has touched an id it should not have. **Record the logged line count and per-frame capacity for the 32-slot stub screen** -- it is the second of §13 item 1's three numbers.

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -2
git add gui/composer_stub.go gui/composer_stub_test.go gui/template_engrave.go
git commit -s -F - <<'MSG'
gui: the paged stub-teaching screen, and one label for one value (composer S3 task A9)

Every origin on the screen comes from ExpandWalletPolicyChunks over the
device's own output, so what the operator writes down is what slotMatchesCard
will compare a card against. The shipped template-engrave screen called its
4-byte stub "Template-ID", which is md.WalletIdKind's name for the 16-byte
id; both occurrences now say "mk1 stub (template)".

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
### Task A10: the five presets (§4d) -- was BLOCKED on F-453's Rust half (plan S0b, landed 1dc8d409 on 2026-09-02), and the only task that is

**PRECONDITION, hard.** `design/FOLLOWUPS.md` F-453 (`composer-preset-vectors-missing`) owns the Rust half to this stage: `md compose --preset <name>` plus one exported vector per archetype in descriptor-mnemonic, FIRST. That half is authored as its own mini-plan, `design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` (R0 GREEN), implemented on descriptor-mnemonic branch `composer-s0b` and under whole-diff review when this task was written. It is NOT on descriptor-mnemonic `main`: at `66bdf2f4`, `md compose --help` shows `--wrapper`, `--path`, `--experimental`, `--json` and no `--preset`, and the corpus carries no preset entry. **A preset is a normative `PathList` shape; re-authoring one in Go with no pinned oracle is exactly the drift the Rust-primary rule exists to prevent.** Until S0b has merged to descriptor-mnemonic `main`, this task does not start, and Part A ships without presets -- the blank route is unaffected, which is why this blocks one task rather than the stage.

**The one fact this task cannot carry yet is S0b's merge commit, and it is not a placeholder for the implementer to fill.** The controller records it on the line below, on the day S0b merges to descriptor-mnemonic `main`. The implementer READS it and the precondition step below stops if the checkout is not at it; until that line names a SHA, this task is not dispatchable.

> **S0b merge commit on descriptor-mnemonic `main`:** `1dc8d409f6e8daa099226937f5f107f56b64dd97` (landed 2026-09-02 as a fast-forward of `composer-s0b`: 4793619b, 5002ebac, 87bc10ff, 1dc8d409; CI run 33698441737 green; report `composer-S0b-push-report.md`). Recorded by the controller.

**Files:**
- Create: `gui/composer_presets.go`
- Create: `md/testdata/vectors/keyed_compose_preset_<archetype>.{bytes.hex,conformance.json,descriptor.json,phrase.txt,template}` (vendored, never hand-written) -- six archetypes in snake_case, five files each, **30 files**: `plain_multisig`, `simple_timelocked_inheritance`, `kofn_recovery`, `tiered_recovery`, `hashlock_gated`, `decaying_multisig`.
- Modify: `md/testdata/compose_vectors.provenance.json` (regenerated by `scripts/vendor-compose-vectors.sh`, never edited by hand)
- Modify: `md/compose_vectors_pin_test.go` -- **this task OWNS the pin-test edit; an earlier draft of it said no such edit was needed, and that was wrong.** Two hand-maintained constants move together: `composeVectorNames` (`md/compose_vectors_pin_test.go:39`) holds 26 names and gains the six `keyed_compose_preset_*` names, 26 -> 32; the file-count assertion with its arithmetic comment (`md/compose_vectors_pin_test.go:82-84`, "22 keyed vectors carry five files, 4 unkeyed carry four: 126") becomes 28 keyed x 5 + 4 unkeyed x 4 = **156**. `isComposeVectorFile` (`md/compose_vectors_pin_test.go:57`) needs NO edit -- it already accepts anything prefixed `keyed_compose_`.
- Modify: `scripts/vendor-compose-vectors.sh` -- one line only: the generated pin's own `_comment` at `scripts/vendor-compose-vectors.sh:29` names `126` in prose and would otherwise ship a stale instruction into every regenerated pin. **The script's SELECTION needs no change** (next paragraph).
- Test: `gui/composer_presets_test.go`

**The vendoring script's glob is CONFIRMED against the export names, not assumed.** `scripts/vendor-compose-vectors.sh:16` selects `^(keyed_)?compose_` and S0b exports the presets as `keyed_compose_preset_<archetype>`, so they are picked up with no change to the selection and none to `isComposeVectorFile`. An earlier draft left this as a CONFIRM-when-F-453-lands with a fallback for a `preset_*` export; the export names are now known and the fallback is dropped. `TestEveryKeyedComposeVectorHasAConformanceRecord` needs nothing either: S0b exports a `.conformance.json` per preset, which is why each vendored vector is five files and not four.

**Interfaces:**
- Consumes: the vendored preset vectors; `md.PathList`, `md.Compose`, `md.Composed.Chunks`, `md.DefaultOrigin`; `ChoiceScreen`; `composerState`.
- Produces: `func composerPresets(w md.ComposeWrapper) []composerPreset` where `composerPreset{name string; list md.PathList}`; `func composerPresetPick(ctx *Context, th *Colors, w md.ComposeWrapper) (md.PathList, bool)`.

**Which presets are OFFERED depends on the wrapper (§4d), and S0b changed nothing there:** all six under `wsh` and `tr` (the five toolkit archetypes plus plain k-of-n); under `sh`/`sh(wsh)`, plain k-of-n alone, because §4e admits nothing else there.

**Offering is not pinning -- only the PINNING narrows.** S0b exports ONE vector per archetype, five under `wsh` and `kofn-recovery` under `tr`, so six of the twelve offered (wrapper, preset) pairs have an oracle and six do not:

| (wrapper, preset) pairs | how the test checks them |
| --- | --- |
| `wsh` x plain-multisig, simple-timelocked-inheritance, tiered-recovery, hashlock-gated, decaying-multisig | PINNED: chunks compared against the vendored vector |
| `tr` x kofn-recovery | PINNED: chunks compared against the vendored vector |
| `tr` x the other five, and `wsh` x kofn-recovery | STRUCTURAL only, and named UNPINNED in the test itself |

**A `tr` vector's chunks are never compared against a `wsh` preset's, or the reverse.** The wrapper is half of an assertion's identity here: a table iterated at one wrapper against vectors exported at another is a test that passes by comparing the wrong things, and it would pass loudly.

- [ ] **Step 1: Confirm S0b landed, then vendor and move the pin**

Build `md` from descriptor-mnemonic `main` at the merge commit recorded above and run it BY PATH. **A bare `md` on `PATH` may be a stale install**, and a stale binary answering the `--preset` probe is precisely the false GREEN this precondition exists to prevent. Run from the fork root:

```bash
git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse main
cargo build -q --manifest-path /scratch/code/shibboleth/descriptor-mnemonic/Cargo.toml -p md-cli
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md compose --help 2>&1 \
  | grep -q -- --preset && echo PRESET-PRESENT || echo PRESET-ABSENT
ls /scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/tests/vectors/ \
  | grep -c '^keyed_compose_preset_'
```
Expected: the SHA equals the merge commit recorded above; `PRESET-PRESENT`; `30` (six archetypes x five files). It is a presence probe rather than a line count because `--help` prints `--preset` on more than one line -- the `--path` option's own help text names it as mutually exclusive. **If the SHA differs, or the probe prints `PRESET-ABSENT`, or the count is not 30, STOP** and report that S0b has not landed on `main`; this task is blocked and Part A ships without presets.

```bash
scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic
CGO_ENABLED=0 go test -count=1 -run 'TestComposeVectorsMatchTheirProvenancePin' ./md/ 2>&1 | tail -3
```
Expected: the script prints `vendored 156 files, 32 vectors, primary <sha>`, and `md/testdata/vectors` goes from 277 files to 307 (the corpus's 126 -> 156, beside the other vectors that share the directory). The pin test then FAILS with `pin says 32 vectors, this test knows 26` -- the pin is regenerated from the directory, the hand-maintained list is not, which is the whole point of having both. Make the three edits named under **Files** (`composeVectorNames` 26 -> 32, the file count and its comment 126 -> 156, the script's `_comment` 126 -> 156) and re-run:

```bash
CGO_ENABLED=0 go test -count=1 -run 'TestComposeVectors|TestEveryKeyedComposeVector' ./md/ 2>&1 | tail -3
```
Expected: `ok seedhammer.com/md`.

- [ ] **Step 2: Write the failing test**

`gui/composer_presets_test.go` is authored WITH the implementation below, in one commit, and not before -- which is why this plan carries it as a specification rather than as extractable Go. Every other task's code is in a ```go fence because it can be compiled today; this one cannot, because the vectors it reads do not exist in this tree until S0b has shipped and been vendored, and a test fence that cannot compile would make the plan's own build gate report a failure that is really a precondition.

It asserts, in `gui/composer_presets_test.go`:

- `TestComposerPresetsReproduceTheirVendoredVectors`: **six sub-tests, one per PINNED pair, each named `<wrapper>/<preset>`** -- `wsh/plain-multisig`, `wsh/simple-timelocked-inheritance`, `wsh/tiered-recovery`, `wsh/hashlock-gated`, `wsh/decaying-multisig`, `tr/kofn-recovery`. Each looks its preset up in `composerPresets(w)` for ITS OWN wrapper, runs `md.Compose(p.list)`, and compares `Chunks()` chunk for chunk against `../md/testdata/vectors/keyed_compose_preset_<archetype>.phrase.txt` (snake_case archetype; that file's leading `chunk-set-id:` line is a header, not a chunk). The wrapper is in both the sub-test name and the lookup, so no `tr` vector can reach a `wsh` table entry. **If a preset's chunks differ, the VECTOR wins and the Go table changes** (CLAUDE.md's Rust-primary rule); if the vector itself looks wrong, STOP and record it, because the fix lands in Rust first. A missing vector file is `t.Fatalf("INCONCLUSIVE: no vendored vector at %s -- re-run scripts/vendor-compose-vectors.sh (F-453/S0b)", path)`: never a skip, never a pass.
- `TestComposerPresetsWithoutAVectorAreStructurallyValid`: **six sub-tests, one per UNPINNED pair**, named the same way -- `tr/plain-multisig`, `tr/simple-timelocked-inheritance`, `tr/tiered-recovery`, `tr/hashlock-gated`, `tr/decaying-multisig`, `wsh/kofn-recovery`. Each asserts only what there is an oracle for: `md.ValidatePathList(p.list)` accepts, and `md.Compose(p.list)` succeeds with a non-empty `Chunks()`. Its doc comment states that these pairs are UNPINNED and why -- S0b exports one vector per ARCHETYPE, not one per pair -- so a green run is never read as pinning twelve shapes; when a later export adds a pair's vector, that pair moves into the pinned test and out of this one.
- `TestComposerPresetsUnderLegacyWrappersOfferOnlyPlainKofN`: `composerPresets(md.ComposeSh)` and `composerPresets(md.ComposeShWsh)` each return exactly one entry, and `md.ValidatePathList` accepts it -- §4d's last clause, which would otherwise be a refusal the operator meets after choosing.
- Every preset name passes `assertChoiceLabelFits`.

- [ ] **Step 3: Write the preset table and picker**

Create `gui/composer_presets.go` with `composerPreset`, `composerPresets` and `composerPresetPick`. Each entry's `list` is transcribed from the primary's exported vector's `descriptor.json` path list, with a comment naming the vector file it is pinned to. **Do not invent a shape here**; if a vector's path list cannot be read off the export, that is S0b incomplete and this task stops.

**The vectors' parameters ARE the device's offered defaults -- S0b decided that, so this table is not inheriting a test corpus's accident.** The ruling: the smallest legal shape of each archetype is the honest starter on a device whose only editor is a scroll wheel, and a wider tier is one edit away while a shape the operator did not intend is not. So the defaults are the vector parameters verbatim -- `plain-multisig` 2-of-3; `simple-timelocked-inheritance` `older(26280)`; `kofn-recovery` 2-of-3 with `older(26280)`; `tiered-recovery` 2-of-2 then 1-of-2 with `older(26280)`; `hashlock-gated` `older(26280)` and the digest read off `keyed_compose_preset_hashlock_gated.template`, never typed from memory; `decaying-multisig` 2-of-2 at `older(13140)`, then 1-of-1 at `older(26280)`, then one key at `after(1000000)`.

**The two wrappers share one table because the origins are already wrapper-derived.** `md.DefaultOrigin(w, account)` (`md/compose.go:288`) is `m/48'/0'/<account>'/<script-type>'`, and `ComposeWrapper.ScriptType()` (`md/compose.go:42`) supplies 2 for `wsh` and 3 for `tr` -- which is exactly the difference between the five `wsh` templates' `48'/0'/<i>'/2'` and the `tr` template's `48'/0'/<i>'/3'`. So each archetype is constructed once, at the wrapper it is asked for, and the unpinned pairs get the wrapper's own origins rather than a `wsh` shape wearing a `tr` label. Do not hardcode either origin.

**The wiring point is already named**: Task B11's `composerFlow` calls `composerWrapperPick` and then `composerShapeFlow`, and §7b's step is "Wrapper -> preset or blank -> paths". `composerPresetPick` goes between them, and the blank route is `composerShapeFlow` unchanged -- so this task is a fill-in, not a redesign.

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerPresets' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: **three top-level PASS**; under the vector test **six sub-tests**, one per pinned pair, named `wsh/plain-multisig`, `wsh/simple-timelocked-inheritance`, `wsh/tiered-recovery`, `wsh/hashlock-gated`, `wsh/decaying-multisig`, `tr/kofn-recovery`; under the structural test **six more**, one per unpinned pair; `ok seedhammer.com/gui`. A missing vector file prints `INCONCLUSIVE: no vendored vector at ...` and fails, never a skip.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_presets.go gui/composer_presets_test.go md/testdata/vectors \
        md/testdata/compose_vectors.provenance.json md/compose_vectors_pin_test.go \
        scripts/vendor-compose-vectors.sh
git commit -s -F - <<'MSG'
gui: the six archetype presets, pinned to the Rust primary's exported vectors (composer S3 task A10)

A preset is a normative PathList, so the Go table is checked against the
primary's own chunks rather than against this test's idea of the shape. If a
preset's chunks differ, the vector wins and the table changes.

S0b exports one vector per archetype, five under wsh and kofn-recovery under
tr, so six of the twelve offered (wrapper, preset) pairs are pinned and six are
checked structurally and named unpinned -- a green run does not claim twelve
pinned shapes. The vendored corpus moves 126 -> 156 files, 26 -> 32 vectors,
and md/compose_vectors_pin_test.go's two hand-maintained constants move with it.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task A11: `gui/composer_consent.go` and `gui/composer_flow.go` -- the consent lines, the flow, and Part A's C26 exit

**Files:**
- Create: `gui/composer_consent.go`
- Create: `gui/composer_flow.go`
- Modify (FRAGMENT): `gui/wallet_policy.go:35-46` (the door wiring drafted in the door task)
- Test: `gui/composer_flow_test.go`

**Interfaces:**
- Consumes: `md.PolicyShapeChunks(strs) (PolicyShape, error)` (`md/policy_shape.go:74`) with S2's `Branch.Locks`/`Sha256Digests`/`Sorted`; `md.FormAwareIdChunks`, `md.FormAwareStubChunks`; `md.ExpandWalletPolicyChunks`; `md.DecodeChunks` (`md/expand.go:25`); `policyAddressAt(md1, tpl, keys)` (`gui/policy_address.go:87`) and `addrProofPerChain` (`gui/wallet_policy.go:241`); `bundleCard`/`cardMD1`/`bundleEngrave` (`gui/bundle.go:36`, `gui/bundle_flow.go:616`); `composerReadScreen`; `composerCopy*`.
- Produces: `func composerDigestShort(d [32]byte) string`; `func composerBranchLines(b md.Branch, pathNo int, sole bool) []string`; `func composerListedPaths(list md.PathList) (listed []int, keyPathNo int)`; `func composerConsentLinesFor(chunks []string, listed []int, keyPathNo int) ([]string, error)`; `func composerFlow(ctx *Context, th *Colors)`; `func composerEngraveTemplate(ctx *Context, th *Colors, chunks []string) bool`.

**THERE IS NO `composerConsentLines(chunks)` WRAPPER**, and its absence is a fix rather than an omission: a wrapper hardcoding `composerConsentLinesFor(chunks, nil, 0)` was the only form the live UI reached, so the path-numbering fix below was correct in isolation and dead from production's point of view (fidelity I-2). Callers pass the numbering they have; a caller with none passes `nil` explicitly.

**The consent is derived from the DECODED md1, never from `composerState`** (§7e). That is the property §8q's self-check enforces in Part B, and it is what makes the check meaningful: if the lines were read off the builder's input, a builder defect would print itself back as agreement.

**Part B REPLACES `gui/composer_flow.go` wholesale** (Task B11's ``Replace `gui/composer_flow.go` `` anchor) to insert seating between the stub screen and consent. **Part A's version reaches forward into nothing**: it calls `md.Compose` directly and ends at `composerEngraveTemplate`, so it compiles and runs with every Part B file absent. That is what "Part A ships alone" has to mean about the artifact, not only about the prose -- an earlier fold overwrote this fence with Part B's joined body, and the milestone this task promises could not have been reached by an implementer following the plan in its own order.

**THE BUILD GATE CANNOT SEE THIS CLASS, and its blind-spot line should be read as covering it.** `scripts/plan-build-gate-go.sh` extracts the WHOLE document with `Replace` winning, so it can only ever build the FINAL state of a file -- never "what compiles after only Task A11 has landed". Incremental buildability is not something a whole-document extraction can observe, which is why this task carries its own Part-A-only gate step below, using the gate's `GATE_UNTIL` mode.

**What Part A alone is honest for:** the C26 no-payload journey (§12 item 3), walked by `TestComposerNoPayloadWalkEngravesAKeylessTemplate` in this task's own test file. With a payload loaded the door states "Keys loaded: N" over a Build that cannot yet use them, which is why Part B follows immediately; and §7e's self-check is a Part B deliverable (Task B6), so Part A's consent is the decoded md1's own lines plus §8l. The plan says that rather than implying the check is already there.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_flow_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"
	"time"

	"seedhammer.com/md"
)

// TestComposerConsentLinesDescribeEveryPathFromTheDecodedMd1 is §7e.
//
// The input is CHUNKS, not a PathList: the consent must be derivable from
// what the device is about to engrave, or §8q's self-check has nothing to
// compare against.
func TestComposerConsentLinesDescribeEveryPathFromTheDecodedMd1(t *testing.T) {
	digest := [32]byte{0xab, 0xcd}
	digest[31] = 0xef
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 1000}, Hash: &digest},
	}}
	c, err := md.Compose(list)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	lines, err := composerConsentLinesFor(chunks, nil, 0)
	if err != nil {
		t.Fatalf("composerConsentLines: %v", err)
	}
	joined := strings.Join(lines, "\n")
	for _, want := range []string{
		"Path 1: 2-of-3",
		"Path 2:",
		"1000 blocks",  // §6b echo form, in operator units
		"abcd",         // the digest's first bytes
		"Template-ID:", // the id, NAMED by kind (§7c)
		"mk1 stub (template):",
		"Keyless template - no addresses.", // D4
	} {
		if !strings.Contains(joined, want) {
			t.Errorf("the consent surface does not say %q:\n%s", want, joined)
		}
	}
	// NOT the shipped Wallet Policy consent's words: md1Summary prints
	// "Complex policy - cannot display safely." for every shape this composer
	// exists to author (md/md_test.go:337,416), which is exactly why §7e gives
	// the composer its own surface.
	if strings.Contains(joined, "cannot display safely") {
		t.Errorf("the composer's consent fell back to md1Summary's complex-policy line, "+
			"which describes nothing:\n%s", joined)
	}
}

// TestComposerConsentMarksTheExperimentalForms is §7e's "EXPERIMENTAL marks",
// derived from the decoded shape rather than from the operator's answers.
func TestComposerConsentMarksTheExperimentalForms(t *testing.T) {
	digest := [32]byte{0x01}
	keyless := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Hash: &digest},
	}}
	c, err := md.Compose(keyless)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, _ := c.Chunks()
	lines, err := composerConsentLinesFor(chunks, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(strings.Join(lines, "\n"), "KEY-LESS") {
		t.Errorf("a key-less path is not marked on the consent surface:\n%s", strings.Join(lines, "\n"))
	}

	unsorted := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: false}},
	}}
	c2, err := md.Compose(unsorted)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks2, _ := c2.Chunks()
	lines2, err := composerConsentLinesFor(chunks2, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(strings.Join(lines2, "\n"), "UNSORTED") {
		t.Errorf("a sole unsorted key set is not marked EXPERIMENTAL:\n%s", strings.Join(lines2, "\n"))
	}
	// And a LOWERING-FORCED multi carries no such mark: the operator declined
	// nothing, and a mark there would teach them to discount the real one.
	forced := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 2, Sorted: true}},
	}}
	c3, err := md.Compose(forced)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks3, _ := c3.Chunks()
	lines3, err := composerConsentLinesFor(chunks3, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(strings.Join(lines3, "\n"), "UNSORTED") {
		t.Errorf("a lowering-forced multi is marked UNSORTED (EXPERIMENTAL); §5a says the "+
			"mark belongs only where sorted was legal and declined:\n%s", strings.Join(lines3, "\n"))
	}
}

// TestComposerNUMSNoteFiresOnlyForATaprootFallback is §8f's condition test.
func TestComposerNUMSNoteFiresOnlyForATaprootFallback(t *testing.T) {
	digest := [32]byte{0x02}
	// tr with no unlocked single-key path: §5 extracts no internal key, so
	// the policy falls back to NUMS.
	nums := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 1}, Hash: &digest},
	}}
	c, err := md.Compose(nums)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, _ := c.Chunks()
	lines, err := composerConsentLinesFor(chunks, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(strings.Join(lines, "\n"), "KEY PATH: NONE (NUMS)") {
		t.Errorf("a NUMS taproot policy does not carry §8f:\n%s", strings.Join(lines, "\n"))
	}
	// An EXTRACTED internal key says the opposite thing, and it is the line
	// that must never be missing: a spendable key path moves funds without
	// satisfying any leaf.
	extracted := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	c2, err := md.Compose(extracted)
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks2, _ := c2.Chunks()
	lines2, err := composerConsentLinesFor(chunks2, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	joined := strings.Join(lines2, "\n")
	if !strings.Contains(joined, "A KEY CAN SPEND ALONE") {
		t.Errorf("an extracted internal key is not stated:\n%s", joined)
	}
	if strings.Contains(joined, "NUMS") {
		t.Errorf("§8f fired on a policy with a real key path:\n%s", joined)
	}
	assertModalBodyFits(t, "the §8f NUMS note", errorScreenBody, composerCopyNUMS())
}

// ═══ PART A's EXIT: SPEC §12 item 3, the C26 no-payload walk ════════════════
//
// A device with NO payload reaches Wallet Policy, chooses Build, composes a
// shape, reads the stub screen with its per-slot expected origins, consents,
// and engraves a keyless template whose md1 DECODES on the device.
//
// THIS WALK LIVES IN PART A because §12 item 3 is PART A's declared exit, and
// a claim that "Part A ships alone" is only measured if the test that
// discharges it runs with Part B absent. It is written to pass in BOTH states:
// once Part B's seating step lands between the stub screen and consent, the
// walk takes that step's key-less choice and the rest of the journey is
// unchanged. A walk that had to be rewritten when the next task landed would
// stop being the same claim.
//
// It stops at the census: no hardware, no plate.

// TestComposerNoPayloadWalkEngravesAKeylessTemplate is Part A's declared exit,
// §12 item 3, WALKED.
//
// The test that carried this name before asserted only that the door drew. §12
// item 3 asks for "door line present, shape, stub screen with per-slot
// expected origins, consent stating no addresses, form choice collapsed,
// keyless-template engrave whose md1 decodes" -- six clauses, of which one was
// covered, by a separate artifact-level test.
func TestComposerNoPayloadWalkEngravesAKeylessTemplate(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newEngravedAwarePlatform()
		p.engraver = newEngraver()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		// NO payload at all: ctx.sysw stays nil, which is C26's whole case.

		frame, quit := runUI(ctx, func() { walletPolicyFlow(ctx, &descriptorTheme) })
		defer quit()

		got, ok := pumpUntil(frame, "Build a new policy", 24)
		if !ok {
			t.Fatalf("the door never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, composerCopyNoKeys()) {
			t.Errorf("the door does not say the build will be key-less.\nFrame: %q", got)
		}
		if uiContains(got, "From payload") {
			t.Errorf("From payload was offered with no payload loaded.\nFrame: %q", got)
		}
		click(&ctx.Router, Down) // Scan cards -> Build a new policy
		click(&ctx.Router, Button3)

		if got, ok = pumpUntil(frame, "Which script?", 24); !ok {
			t.Fatalf("the wrapper picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // -> Segwit (wsh)
		click(&ctx.Router, Button3)

		// §7b's live line carries NO key count with no payload loaded.
		if got, ok = pumpUntil(frame, "Add a spend path", 24); !ok {
			t.Fatalf("the path list never drew.\nLast frame: %q", got)
		}
		if uiContains(got, "keys available") {
			t.Errorf("the path list counts keys on a machine with no payload; §7b scopes "+
				"that line to \"whenever a payload is loaded\".\nFrame: %q", got)
		}
		click(&ctx.Router, Button3) // Add a spend path

		pumpUntil(frame, "What can spend on this path?", 24)
		click(&ctx.Router, Button3) // Keys
		pumpUntil(frame, "how many keys?", 24)
		click(&ctx.Router, Down, Down) // 1 -> 3
		click(&ctx.Router, Button3)
		pumpUntil(frame, "how many must sign?", 24)
		click(&ctx.Router, Down) // 1 -> 2
		click(&ctx.Router, Button3)

		if got, ok = pumpUntil(frame, "Path 1: 2-of-3", 24); !ok {
			t.Fatalf("the path list does not show the new path.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down, Down, Down)
		click(&ctx.Router, Button3) // Done
		pumpUntil(frame, "Sorted keys, or your order?", 24)
		click(&ctx.Router, Button3) // Sorted

		// THE STUB SCREEN, with a per-slot expected origin for every slot.
		if got, ok = pumpUntil(frame, "mk1 stub (template)", 32); !ok {
			t.Fatalf("the stub screen never drew.\nLast frame: %q", got)
		}
		// The per-slot "expects a key at" lines are pages in: the body grows
		// one line per slot and the grammar admits 32, which is why §7c makes
		// this a paged widget at all.
		if got, ok = composerPageUntil(t, ctx, frame, "expects a key at", 10); !ok {
			t.Fatalf("the stub screen never named an expected origin.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// PART B'S SEATING STEP, IF IT IS PRESENT. With nothing to seat from it
		// asks whether to engrave the key-less template; in Part A alone there
		// is no such screen and consent follows the stub directly. The walk
		// tolerates both, because it is the same journey either way, and a walk
		// rewritten when the next task lands is no longer the same claim.
		if got, ok = pumpUntil(frame, "Seat keys into this template?", 12); ok {
			click(&ctx.Router, Button3) // Engrave a key-less template
		}

		// CONSENT, STATING NO ADDRESSES (D4).
		if got, ok = composerPageUntil(t, ctx, frame, "Keyless template - no addresses", 12); !ok {
			t.Fatalf("the consent never says there are no addresses.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		if got, ok = pumpUntil(frame, "Nothing outside this device", 48); !ok {
			t.Fatalf("§8l never drew.\nLast frame: %q", got)
		}
		press(&ctx.Router, Button3)
		frame()
		time.Sleep(confirmDelay)
		frame()

		// THE FORM CHOICE COLLAPSED, and saying so is §7f's rule.
		if got, ok = pumpUntil(frame, "No slot is seated", 48); !ok {
			t.Fatalf("the collapsed form choice never said why it collapsed.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3)

		if got, ok = pumpUntil(frame, "This engraves", 64); !ok {
			t.Fatalf("the plate census never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "md1 template") {
			t.Errorf("the census does not name the key-less template.\nFrame: %q", got)
		}
		if uiContains(got, "mk1 key") {
			t.Errorf("a key-less composition cut a key card.\nFrame: %q", got)
		}
	})
}

// TestComposerKeylessTemplateDecodesOnTheDevice is the artifact half of
// §12 item 3, at the layer that can assert it: the chunk set this flow
// engraves is read back by the device's own decoder, every slot declares an
// origin, and no two slots share one (§4f's invariant, which is what makes
// the template seatable at all).
func TestComposerKeylessTemplateDecodesOnTheDevice(t *testing.T) {
	for _, w := range []md.ComposeWrapper{md.ComposeTr, md.ComposeWsh, md.ComposeShWsh, md.ComposeSh} {
		list := md.PathList{Wrapper: w, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		}}
		c, err := md.Compose(list)
		if err != nil {
			t.Fatalf("wrapper %v: md.Compose: %v", w, err)
		}
		chunks, err := c.Chunks()
		if err != nil {
			t.Fatalf("wrapper %v: Chunks: %v", w, err)
		}
		tpl, keys, err := md.ExpandWalletPolicyChunks(chunks)
		if err != nil {
			t.Fatalf("wrapper %v: the device cannot decode what it just built: %v", w, err)
		}
		if tpl.N != 3 {
			t.Errorf("wrapper %v: decoded N = %d, want 3", w, tpl.N)
		}
		seen := map[string]bool{}
		for _, k := range keys {
			if k.XpubPresent {
				t.Errorf("wrapper %v: slot @%d carries a key in a KEYLESS template", w, k.Index)
			}
			if len(k.OriginPath) == 0 {
				t.Errorf("wrapper %v: slot @%d declares no origin; the fork's decoder "+
					"refuses a pathless slot (F-166)", w, k.Index)
			}
			o := k.OriginPath.String()
			if seen[o] && !k.FingerprintPresent {
				t.Errorf("wrapper %v: two slots declare %s with no fingerprints, which "+
					"errSeatSlotContested makes unseatable (§4f's invariant)", w, o)
			}
			seen[o] = true
		}
		// And the consent surface reads it.
		if _, err := composerConsentLinesFor(chunks, nil, 0); err != nil {
			t.Errorf("wrapper %v: composerConsentLines: %v", w, err)
		}
	}
}

// composerPageUntil pages forward looking for a string, returning the frame it
// was found on. Paging is forward-only with wrap, so a bounded number of
// pages either finds it or proves it is not on the screen at all.
func composerPageUntil(t *testing.T, ctx *Context, frame func() (string, bool), want string, pages int) (string, bool) {
	t.Helper()
	var last string
	for i := 0; i < pages; i++ {
		c, ok := frame()
		if !ok {
			return last, false
		}
		last = c
		if uiContains(c, want) {
			return c, true
		}
		click(&ctx.Router, Button2)
	}
	return last, false
}

// composerPageToEnd pages a composer read screen to its last page and
// confirms.
//
// The checkmark is withheld until the last page has been laid out once (§7e's
// proof is the addresses, and consenting before they are drawn is consenting
// to a wallet nobody saw), so a walk has to page as an operator would.
func composerPageToEnd(t *testing.T, ctx *Context, frame func() (string, bool)) {
	t.Helper()
	for i := 0; i < 12; i++ {
		click(&ctx.Router, Button2)
		if _, ok := frame(); !ok {
			return
		}
	}
	click(&ctx.Router, Button3)
	frame()
}

// TestComposerBackAtThePathListKeepsTheComposition is §7b's rule, walked --
// and before this the answer to "is any Back asserted by a test that would
// fail if Back lost state" was no, for every one of them.
func TestComposerBackAtThePathListKeepsTheComposition(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
		defer quit()

		pumpUntil(frame, "Which script?", 24)
		click(&ctx.Router, Down)
		click(&ctx.Router, Button3)
		pumpUntil(frame, "Add a spend path", 24)
		click(&ctx.Router, Button3)
		pumpUntil(frame, "What can spend on this path?", 24)
		click(&ctx.Router, Button3)
		pumpUntil(frame, "how many keys?", 24)
		click(&ctx.Router, Down)
		click(&ctx.Router, Button3)
		pumpUntil(frame, "how many must sign?", 24)
		click(&ctx.Router, Button3)
		got, ok := pumpUntil(frame, "Path 1: 1-of-2", 24)
		if !ok {
			t.Fatalf("the path was never added.\nLast frame: %q", got)
		}

		// BACK at the path list goes back ONE STEP, to the wrapper.
		click(&ctx.Router, Button1)
		if got, ok = pumpUntil(frame, "Which script?", 24); !ok {
			t.Fatalf("Back at the path list did not land on the wrapper picker; it used "+
				"to drop the whole composition.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down)
		click(&ctx.Router, Button3)

		// ...and the list is still there.
		if got, ok = pumpUntil(frame, "Path 1: 1-of-2", 24); !ok {
			t.Fatalf("the path list lost its path across a Back; §7b's rule is that going "+
				"back loses nothing.\nLast frame: %q", got)
		}
	})
}

// TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex is C16's
// unskippable-confirm rule, and the reproduction is the one the fidelity lens
// constructed: the confirm was memoised by path INDEX, and "Remove path"
// splices the slice without touching the memo.
func TestComposerKeylessConfirmFiresAgainForANewPathAtAReusedIndex(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh}, reg: &seedRegistry{}}
		frame, quit := runUI(ctx, func() { composerShapeFlow(ctx, &descriptorTheme, st) })
		defer quit()

		add := func(which string) {
			t.Helper()
			pumpUntil(frame, "Add a spend path", 24)
			// An empty list opens on "Add a spend path".
			for i := 0; i < len(st.list.Paths); i++ {
				click(&ctx.Router, Down)
			}
			click(&ctx.Router, Button3)
			pumpUntil(frame, "What can spend on this path?", 24)
			click(&ctx.Router, Down) // Keys -> A hash, no keys
			click(&ctx.Router, Button3)
			got, ok := pumpUntil(frame, "KEY-LESS PATH", 24)
			if !ok {
				t.Fatalf("%s: §8a did not fire for a key-less path -- an unskippable "+
					"confirm that can be skipped is the defect §12 item 4 exists "+
					"for.\nLast frame: %q", which, got)
			}
			// Decline it: the path is truncated and we are back at the list.
			click(&ctx.Router, Button1)
			frame()
		}
		add("first key-less path")
		add("a second key-less path at the same index")
	})
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerConsent|^TestComposerNUMS|^TestComposerNoPayloadWalk|^TestComposerKeylessTemplate' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerConsentLines`, `undefined: composerFlow`.

- [ ] **Step 3: Write the consent lines**

Create `gui/composer_consent.go`:

```go
package gui

import (
	"encoding/hex"
	"fmt"

	"seedhammer.com/md"
)

// The composer's consent surface (SPEC §7e).
//
// IT IS A NEW SURFACE, and neither shipped one would do. walletPolicyConsentLines
// summarises through md1Summary, which prints "Complex policy - cannot display
// safely." for every shape the codec marks non-renderable -- measured for
// every multi-path or taproot shape this composer exists to author
// (md/md_test.go:337,416). And policySummaryLines, the one structural summary
// that exists, counts a multi-path wsh script as ONE branch
// (md/policy_shape.go:41-43). Neither describes what the operator built.
//
// EVERY LINE IS DERIVED FROM THE DECODED md1, never from composerState. That
// is not a preference: §8q's self-check compares the decoded shape against
// what the operator composed, and a surface that read the builder's INPUT
// would print a builder defect back as agreement.

// composerListedPaths maps each LEAF, in the order PolicyShape reports them,
// to the operator's own path number, and names the path §5 extracted as the
// taproot internal key (0 when there is none).
//
// IT EXISTS SO THE FIX REACHES PRODUCTION. composerConsentLinesFor takes this
// numbering and was correct in isolation, but the only call site the live UI
// reaches went through composerConsentLines, which hardcodes (nil, 0) -- so on
// a tr policy with an extracted internal key the consent printed "Path 1" for
// the operator's Path 2 while the seating prompt for the same slot said
// "Path 2". Two screens disagreeing about which path is which, on the surface
// whose whole job is proving which wallet is being consented to.
func composerListedPaths(list md.PathList) (listed []int, keyPathNo int) {
	internal := -1
	if list.Wrapper == md.ComposeTr {
		for i, p := range list.Paths {
			if p.Keys != nil && p.Keys.N == 1 && p.Lock == nil && p.Hash == nil {
				internal = i
				break
			}
		}
	}
	for i := range list.Paths {
		if i == internal {
			continue
		}
		listed = append(listed, i+1)
	}
	if internal >= 0 {
		keyPathNo = internal + 1
	}
	return listed, keyPathNo
}

// composerDigestShort renders a digest as §7e asks: first 8 and last 8 hex.
// A full 64-hex line is CUT rather than wrapped at the label budget, and a
// cut digest hides which end is missing.
func composerDigestShort(d [32]byte) string {
	h := hex.EncodeToString(d[:])
	return h[:8] + ".." + h[56:]
}

// composerBranchLines describes one spend path from its decoded Branch.
//
// `sole` is len(shape.Branches) == 1, which is what makes the UNSORTED mark
// honest: §5's key-set rule admits sortedmulti only for a SOLE unlocked,
// unhashed path, so an unsorted key set anywhere else is lowering-forced and
// the operator declined nothing (§5a). Marking those too would teach the
// operator to discount the mark that matters.
func composerBranchLines(b md.Branch, pathNo int, sole bool) []string {
	head := fmt.Sprintf("Path %d: ", pathNo)
	switch {
	case b.Keys == 0:
		head += "KEY-LESS (EXPERIMENTAL)"
	case b.N > 0:
		head += fmt.Sprintf("%d-of-%d", b.K, b.N)
	case b.Keys == 1:
		head += "1 key"
	default:
		head += fmt.Sprintf("%d key(s), custom", b.Keys)
	}
	out := []string{head}
	for _, l := range b.Locks {
		// §7e asks for "its lock kind and value in operator units (§6b echo
		// form)" -- "1000 blocks (about 6.9 days)", not the path-list ROW form
		// "1000 blocks". The row has one line to spend; the consent does not.
		for _, line := range composerLockEcho(l, composerBound{}) {
			out = append(out, "  "+line)
		}
	}
	for _, d := range b.Sha256Digests {
		out = append(out, "  hash "+composerDigestShort(d))
	}
	if sole && !b.Sorted && b.N >= 2 && len(b.Locks) == 0 && len(b.Sha256Digests) == 0 {
		out = append(out, "  UNSORTED (EXPERIMENTAL)")
	}
	return out
}

// composerConsentLines is the whole surface, in §7e's order: paths, the
// key-path line, the id NAMED by kind with both stubs, then addresses or the
// D4 line saying there are none.
// THERE IS NO PARAMETERLESS WRAPPER, and its absence is the fix.
//
// composerConsentLines(chunks) used to sit here hardcoding
// composerConsentLinesFor(chunks, nil, 0), and it was the ONLY form the live
// UI reached -- so the path-numbering fix below was correct in isolation and
// dead from production's point of view. A wrapper that supplies the very
// argument a fix is about is a wrapper that un-fixes it; callers pass the
// numbering they have, and a caller with none passes nil explicitly.
//
// `listed[i]` is the operator's path number for branch i; `keyPathNo` names
// the path §5 extracted as the taproot internal key, or 0.
func composerConsentLinesFor(chunks []string, listed []int, keyPathNo int) ([]string, error) {
	shape, err := md.PolicyShapeChunks(chunks)
	if err != nil {
		return nil, err
	}
	if !shape.Complete {
		// THE HONESTY CONTRACT (md/policy_shape.go:60-63): an incomplete walk
		// means the summariser met a node it could not classify, and a partial
		// description is worse than none -- the operator would believe they had
		// seen the whole policy. The composer only builds shapes §5 lowers, so
		// this is a builder defect, and it says so rather than showing a
		// half-policy.
		return nil, fmt.Errorf("md: this device cannot describe the policy it just built")
	}
	tpl, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		return nil, err
	}

	var lines []string
	sole := len(shape.Branches) == 1
	// THE OPERATOR'S PATH NUMBER, NOT THE BRANCH ORDINAL. PolicyShape.Branches
	// are LEAVES, and a taproot internal key is reported through KeyPath
	// rather than as a Branch (md/policy_shape.go:42-45) -- so for
	// tr[P1: 1-of-1, P2: 2-of-3] §5 extracts P1 and the branch list holds one
	// entry, which printed as "Path 1: 2-of-3" while the seating prompt for
	// the same slot said "Path 2". Two screens disagreeing about which path is
	// which, on the surface that consents to steel.
	//
	// `listed` maps branch ordinal -> the operator's path number; it is nil
	// when the caller has no path list (a consent rendered from chunks alone),
	// and then the branch ordinal is the honest answer because there is no
	// other numbering to be wrong about.
	for i, b := range shape.Branches {
		pathNo := i + 1
		if i < len(listed) {
			pathNo = listed[i]
		}
		lines = append(lines, composerBranchLines(b, pathNo, sole)...)
	}
	lines = append(lines, "")
	switch shape.KeyPath {
	case md.KeyPathSpendable:
		line := "Key-path: A KEY CAN SPEND ALONE"
		if keyPathNo > 0 {
			line = fmt.Sprintf("Key-path (Path %d): A KEY CAN SPEND ALONE", keyPathNo)
		}
		lines = append(lines, line)
	case md.KeyPathNUMS:
		lines = append(lines, composerCopyNUMS())
	}

	id, kind, err := md.FormAwareIdChunks(chunks)
	if err != nil {
		return nil, err
	}
	stub, err := md.FormAwareStubChunks(chunks)
	if err != nil {
		return nil, err
	}
	label := "mk1 stub (policy): %x"
	if kind == md.WalletIdTemplate {
		label = "mk1 stub (template): %x"
	}
	lines = append(lines, "", fmt.Sprintf("%s: %x", kind, id), fmt.Sprintf(label, stub))

	// ADDRESSES, or a line saying plainly why there are none (D4). Never
	// silence: an absent address block is indistinguishable from a screen that
	// simply has none, and "I did not see any addresses" is exactly the
	// observation that should stop an operator (gui/wallet_policy.go:245-249).
	// §8i, RESTATED AT CONSENT. §6c and §8i's own heading say "at entry AND at
	// consent": the rule whose whole purpose is to prevent an unspendable
	// wallet was stated once, several screens earlier, on a policy that may
	// have gained its hashlock afterwards.
	for _, b := range shape.Branches {
		if len(b.Sha256Digests) > 0 {
			lines = append(lines, "", composerCopyHashRule())
			break
		}
	}
	// §8d, on the surface §7g's divergence table puts it on: the row
	// "consent | compares the shown id with a coordinator's" answers
	// "DOCUMENTATION: §8d line". It was printed only on the stub screen,
	// several screens and possibly several edits earlier.
	lines = append(lines, "", composerCopyOwnWallet())

	at, ok := policyAddressAt(chunks, tpl, keys)
	if !ok {
		return append(lines, "", "Keyless template - no addresses.", "Verify off-device."), nil
	}
	lines = append(lines, "")
	for _, chain := range []struct {
		label  string
		change bool
	}{{"Receive", false}, {"Change", true}} {
		for i := 0; i < addrProofPerChain; i++ {
			a, err := at(uint32(i), chain.change)
			if err != nil {
				return nil, fmt.Errorf("md: address derivation failed for %s %d", chain.label, i)
			}
			lines = append(lines, fmt.Sprintf("%s %d:", chain.label, i), a)
		}
	}
	return lines, nil
}
```

- [ ] **Step 4: Write the flow**

Create `gui/composer_flow.go`:

```go
package gui

import (
	"slices"

	"seedhammer.com/md"
)

// composerFlow is "Build a new policy" (SPEC_wallet_policy_composer.md §7),
// from the door to the plate.
//
// PART A's VERSION, AND IT BUILDS WITH PART B ABSENT. That is the whole point
// of the split: the plan's architecture prose and its Global Constraints both
// say "Part A ships alone", and a Task A11 whose own fence called
// composerSeatFlow, composerMappingReview, composerConsentFlow, composerFormPick
// and composerMintCards would make that false of the artifact meant to deliver
// it -- an implementer following the plan task by task could not compile the
// milestone the plan promises here. Task B11 REPLACES this file wholesale to
// insert seating between the stub screen and consent; nothing here reaches
// forward into it.
//
// WHAT PART A ALONE IS HONEST FOR: the C26 no-payload journey, §12 item 3.
// With a payload loaded the door states "Keys loaded: N" over a Build that
// cannot yet use them, which is why Part B follows immediately. §7e's
// self-check arrives with it (Task B6) -- Part A's consent is the decoded
// md1's own lines plus §8l, and the plan says so rather than implying the
// check is present.
//
// THE SCRUB IS INSTALLED HERE, at the top, before any seed can exist -- the
// same construction gui/multisig_build.go:290-291 uses and for the reason
// stated there: every exit below (a Back, a refusal, a ctx.Done unwind, a
// panic) is covered without an implementer remembering to add one to a new
// return. C14 asks for Multisig Build's treatment and this is it.
func composerFlow(ctx *Context, th *Colors) {
	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
	defer st.reg.scrub()

	w, ok := composerWrapperPick(ctx, th)
	if !ok {
		return
	}
	st.list.Wrapper = w

	var shown []string // the chunk set the stub screen last displayed (§8s)
	for !ctx.Done {
		if !composerShapeFlow(ctx, th, st) {
			// BACK AT THE PATH LIST GOES BACK ONE STEP, to the wrapper, with
			// the list intact -- §7b's "going back should lose nothing".
			w, ok := composerWrapperPick(ctx, th)
			if !ok {
				return
			}
			st.list.Wrapper = w
			continue
		}
		c, err := md.Compose(st.list)
		if err != nil {
			composerShowRefusal(ctx, th, "Spend paths", err)
			continue
		}
		chunks, err := c.Chunks()
		if err != nil {
			showError(ctx, th, "Template", "Couldn't build the template from this shape.")
			continue
		}
		// §8s's changed-id line is decided by COMPARING CHUNK SETS, not by an
		// "edited" flag that any Back would set and nothing would reset. A
		// false "this id changed" on the screen whose job is to be copied onto
		// steel trains the operator to discount the line that will one day be
		// true.
		changed := shown != nil && !slices.Equal(shown, chunks)
		if !composerStubFlow(ctx, th, chunks, nil, changed) {
			shown = chunks
			continue
		}
		shown = chunks

		listed, keyPathNo := composerListedPaths(st.list)
		lines, err := composerConsentLinesFor(chunks, listed, keyPathNo)
		if err != nil {
			showError(ctx, th, "Review", composerCopySelfCheckFailed())
			return
		}
		if !composerReadScreen(ctx, th, "Review", lines) {
			continue
		}
		// §8l, unskippable, immediately before the first thing that cuts.
		if !composerConfirmScreen(ctx, th, "Before you fund it",
			composerConfirmBody(composerCopyNothingChecked())) {
			continue
		}
		composerEngraveTemplate(ctx, th, chunks)
		return
	}
}

// composerEngraveTemplate cuts the keyless template through the SHIPPED
// bundle machinery, so the plate planning, the census and the engrave screen
// are the ones every other md1 goes through -- the composer contributes the
// strings and nothing else (the I-VERBATIM rule gui/multisig_build.go:30-32
// states for its own md1).
//
// §7f's form choice COLLAPSES here and says so: with no slot seated there is
// no concrete policy and there are no cards, so "template only" is the whole
// of the offer. Task B11's composerEngraveStep replaces this with the full
// §7f choice once seating exists.
func composerEngraveTemplate(ctx *Context, th *Colors, chunks []string) bool {
	showError(ctx, th, "What to engrave",
		"No slot is seated, so there is a template and nothing else.")
	cards := []bundleCard{{
		kind:    cardMD1,
		label:   "md1 template",
		strings: chunks,
		summary: "key-less wallet policy",
	}}
	// buildPlateCensusLines, not composerCensusLines: the latter is Task B9's
	// and adds §7f's recovery-error line, which arrives with the cards it is
	// about. Part A cuts one md1 and counts it through the same
	// bundlePlatePlan every other plate goes through.
	// The census carries the SUPPLY paths' title (gui/multisig.go,
	// gui/singlesig.go), not Multisig Build's: the build title is a registered
	// walk anchor (cmd/emu/needle_test.go) whose proof is that exactly one flow
	// draws it, and the composer is a supply-shaped flow, not a build.
	if !confirmReviewScreen(ctx, th, "Plates To Cut",
		buildPlateCensusLines(ctx.Platform.EngraverParams(), cards)) {
		return false
	}
	return bundleEngrave(ctx, th, "Wallet Policy", cards, "", "") == bundleEngraveDone
}
```

- [ ] **Step 5: Apply the door wiring, and give TWO SHIPPED WALKS their new first screen**

Apply the `gui/wallet_policy.go:35-46` replacement drafted in the door task now that `composerFlow` exists.

**The door BREAKS FIVE shipped tests, and this is measured, not anticipated.** All five drive `walletPolicyFlow` and pump to a screen that is no longer first. Only two are in the file a reader would look in; the other three were found by the SHARDED WHOLE-PACKAGE run and a targeted `-run` filter would have shown none of them:

| test | file | its route past the door |
| --- | --- | --- |
| `TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen` | `gui/wallet_policy_descriptor_walk_test.go:128` | Down, Button3 (From payload) |
| `TestWalkWalletPolicyRendersARecordWithLeadingWhitespace` | `gui/wallet_policy_descriptor_walk_test.go` (the second walk) | Down, Button3 |
| `TestF440BundleIncompleteModalDismissesOnBack` | `gui/modal_back_test.go:86` | Down, Button3 |
| `TestF437CardDoorsDoNotPromiseTyping` (its two `wallet policy` rows only) | `gui/payload_door_label_test.go:25` | Down, Button3, guarded on the row's name |
| `TestF76WalletPolicyCountsACompleteMd1CardFromThePayload` | `gui/payload_door_walk_test.go:138` | Down, Button3 |

The failure reads the same in all five:

```
--- FAIL: TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen
    gui/wallet_policy_descriptor_walk_test.go:147: the Descriptor offer never drew.
    Last frame: "Nokeysloaded.Thisbuildsakey-lesstemplate.ScancardsFrompayloadBuildanewpolicyWalletPolicy"
```

`Down` then `Button3` in every case, because the door opens on "Scan cards" at index 0 and "From payload" is index 1 whenever the payload holds a Descriptor or an md1/mk1 record -- which is exactly the condition all five fixtures satisfy.

They are UPDATED, never deleted or skipped: each gains the door step before its existing pump, because the walk it asserts is still exactly right -- one screen earlier than it was. In `gui/wallet_policy_descriptor_walk_test.go`, before each walk's `pumpUntil(frame, "Wallet policy from where?", 16)`:

```go
		// (0) THE COMPOSER'S DOOR, which is now the first screen in every
		// state (SPEC_wallet_policy_composer §7a). "Scan cards" is index 0, so
		// one Down selects "From payload", which is the route this walk takes.
		if _, ok := pumpUntil(frame, "Build a new policy", 16); !ok {
			t.Fatal("the composer door never drew")
		}
		click(&ctx.Router, Down)
		click(&ctx.Router, Button3)
```

The same three lines go before `gui/modal_back_test.go`'s and `gui/payload_door_walk_test.go`'s `pumpUntil(frame, "Cards from where?", ...)`. `gui/payload_door_label_test.go` is table-driven over four programs and only its two `wallet policy` rows take the door, so its step is guarded (`if strings.HasPrefix(tc.name, "wallet policy")`) rather than applied to all four -- the other three programs' doors are unchanged and adding the step to them would make the table lie about what it walks.

**Verified at plan time**, in the build gate's scratch copy of fork `169073c` with the S2 worktree's `md`/`mk`/`sysw`, every extracted block of this plan, and the six fragments hand-wired: all five fail without these steps and pass with them, and `scripts/gui-shard-test.sh ./gui/ 24` then reports **`ok -- all 1125 tests ran across 24 shards`**.

**A shipped walk that starts failing when a door is added in front of it is the door working; a walk deleted or skipped to make it green is the door untested.** Update all five; delete none.

- [ ] **Step 6: PROVE Part A builds and passes with Part B ABSENT**

This is the step that makes "Part A ships alone" a measured claim. The gate's `GATE_UNTIL` mode stops extraction at the first plan line matching its regex, so only Part A's fences reach the scratch copy:

Run: `GATE_UNTIL='^### Task B1' SCRATCH=/scratch/code/shibboleth/.plan-build-gate-go-s3-partA TMPDIR=/scratch/code/shibboleth/.tmp FORK_REPO=/scratch/code/shibboleth/seedhammer scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md 2>&1 | grep -E 'GATE_UNTIL|wrote gui|stopped at'`
Expected: the run reports the line it stopped at, and the extracted files are Part A's only -- **no `gui/composer_sources.go`, `composer_seat.go`, `composer_review.go`, `composer_selfcheck.go`, `composer_engrave.go`, `composer_cards.go`, `composer_census.go` or `composer_join_test.go`** (`composer_discard.go` IS extracted: it moved to Task A5 for exactly this reason).

Then wire Part A's fragments (the door wiring without Task B11's loop) and build:

Run: `python3 scratchpad/handwire_s3.py --part-a /scratch/code/shibboleth/.plan-build-gate-go-s3-partA/seedhammer && cd /scratch/code/shibboleth/.plan-build-gate-go-s3-partA/seedhammer && CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local TMPDIR=/scratch/code/shibboleth/.tmp go vet ./gui/ ; CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local TMPDIR=/scratch/code/shibboleth/.tmp go test -count=1 -run '^TestComposer' ./gui/`
Expected: `go vet ./gui/` clean but for the two pre-existing go1.25 `ArtifactDir` findings, and `ok seedhammer.com/gui` -- **with Part B absent**. Measured at plan time: **23 files extracted, 47 `TestComposer*` top-level PASS**. An `undefined: composerKeySources` (or `composerApplyShapeEdit`, which is how `gui/composer_discard.go` was found to belong in Part A) is this task's own fence reaching forward into Part B, which is exactly what this step exists to catch.

- [ ] **Step 7: Run the whole composer suite**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposer' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)' | tail -40`
Expected: every `TestComposer*` PASS, `ok seedhammer.com/gui`.

Then the whole package, as CI runs it:

Run: `CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -3`
Expected: `ok seedhammer.com/gui`. If a SHIPPED wallet-policy test fails, the door wiring has changed a route it should not have -- read which, and fix the wiring rather than the test.

- [ ] **Step 8: gofmt, commit -- PART A IS SHIPPABLE HERE**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -2
git add gui/composer_consent.go gui/composer_flow.go gui/composer_flow_test.go gui/wallet_policy.go
git commit -s -F - <<'MSG'
gui: the composer's consent surface and a flow that builds with part B absent (composer S3 task A11)

Every consent line is derived from the DECODED md1 rather than from the
builder's input, which is what makes section 8q's self-check meaningful once
part B adds it. This flow reaches forward into nothing: shape, stub screen,
consent, section 8l, key-less template engrave, ending at
composerEngraveTemplate. Part A's exit is section 12 item 3, walked with part
B absent by the GATE_UNTIL run in this task's own steps.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---
---

# PART B -- seating, the mapping review, the self-check, and the engrave forms

Part B is not shippable alone: it needs Part A's shape, stub screen and consent lines. Its exit is spec §12 items 4, 5, 6 and 9 for the seated forms.

### Task B1: `gui/composer_sources.go` -- the payload's keys and cards as a paged pick list, and the oracle that must see them

**Files:**
- Create: `gui/composer_sources.go`
- Test: `gui/composer_sources_test.go`
- Modify (FRAGMENT): `gui/sysw_admit_oracle_test.go:39-76` (`syswConsumers` gains three entries) and `:117-133` (the scanner also matches `takeAll` and `cardSet`)

**Interfaces:**
- Consumes: `syswSession.takeAll(want) ([]string, bool)` (`gui/sysw_session.go:167`), `syswSession.cardSet(want) ([]string, bool)` (`gui/sysw_session.go:214`), `groupRecordsByCard` (via `cardSet`); `sysw.ParseKeyRecord(record) (KeyRecord, error)` with `KeyRecord{Fingerprint [4]byte; Origin bip32.Path; Xpub string; Text string}` (S2); `mk.Decode(in []string) (Card, error)` (`mk/mk.go:148`) and `mk.Card{Network, Path, Fingerprint string; Stubs [][4]byte; Xpub string}` (`mk/mk.go:133-139`); `originComponents(path bip32.Path) []md.PathComponent` (`gui/singlesig_derive.go:139`); `composerPickScreen` (Task A2).
- Produces: `func composerKeySources(ctx *Context) []composerSource`; `func composerCardSources(ctx *Context) []composerSource`; `func composerSourceRow(s composerSource) string`; `func composerSeatPrompt(st *composerState, slot uint8) string`.

**`mk.Card.Xpub` is a base58 STRING** (`mk/mk.go:138`), not `[65]byte` -- the recon's fact table has that wrong, and `md.ExpandedKey.Xpub` is what carries the `[65]byte` form (`md/expand.go:93`). `composerSource.xpub` is therefore the string every source has in common, and `decodeXpubBytes` (`gui/singlesig_derive.go:110`) converts at the one place `md.Composed.Bind` needs bytes.

**Card stubs are IGNORED here** (§7d): the composed policy does not exist yet, so no card can carry its stub, and `seatKeyCards`' layer-1 membership test (`gui/key_card_seating.go:63-73`) would refuse every one of them. The stubs are APPENDED later, when the card is re-minted.

**The consumption-site oracle must be able to see the composer.** `TestEverySyswConsumptionSiteNamesAnAdmittedClass` (`gui/sysw_admit_oracle_test.go:90`) matches `syswOffer*` by prefix and `take` by EXACT selector name (`:127-133`), so `takeAll` and `cardSet` are invisible to it -- three shipped sites are unchecked today (`gui/multisig_build_payload.go:75`, `gui/transaction.go:408,451`; measured with `grep -rn '\.takeAll(\|\.cardSet(' gui/*.go | grep -v _test`, which also finds the three inside `sysw_session.go` that the oracle deliberately skips). The composer's sources would be a fourth. Widening the matcher and registering the three is a small, bounded change that makes §13 D7's mechanism cover what it claims to.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_sources_test.go`:

```go
package gui

import (
	"strings"
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

// TestComposerKeySourcesLabelFingerprintAndOrigin is §7d's label rule: a
// key: record is labelled fingerprint PLUS origin, because two keys sharing
// a fingerprint (one master, two accounts) is the normal C5 case and a
// fingerprint alone would render them identically.
func TestComposerKeySourcesLabelFingerprintAndOrigin(t *testing.T) {
	p := newPlatform()
	ctx := NewContext(p)
	ctx.sysw = composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil)
	got := composerKeySources(ctx)
	if len(got) != 2 {
		t.Fatalf("composerKeySources returned %d sources, want 2", len(got))
	}
	if got[0].label == got[1].label {
		t.Fatalf("two keys from one master render identically as %q; the origin must "+
			"distinguish them (§7g's pack row: labels show fingerprint AND origin)", got[0].label)
	}
	for _, s := range got {
		if !strings.Contains(s.label, "73c5da0a") {
			t.Errorf("the label omits the fingerprint: %q", s.label)
		}
		if !strings.Contains(s.label, "48") {
			t.Errorf("the label omits the origin: %q", s.label)
		}
		if s.xpub == "" {
			t.Errorf("the source carries no xpub, so it can seat nothing: %+v", s)
		}
		if len(s.origin) == 0 {
			t.Errorf("the source carries no origin components; §6a refuses a key: record " +
				"with an empty origin, so this cannot happen from a classified record")
		}
		assertChoiceLabelFits(t, s.label)
	}
}

// TestComposerSourcesRefuseAnUncomparedPayload inherits take's guard: a
// record may not be handed to a program until the payload it came from has
// been authenticated (§12.2). The DOOR counts through has() and is exempt;
// SEATING consumes and is not.
func TestComposerSourcesRefuseAnUncomparedPayload(t *testing.T) {
	p := newPlatform()
	ctx := NewContext(p)
	s := &syswSession{}
	// load(payload, identity, sealed, cliffAbove, compared, digestShown):
	// compared=false is the one that matters here.
	s.load(composerPayloadWith([]string{composerTestKeyRecord}, nil), [32]byte{},
		false, true, false, true)
	ctx.sysw = s
	if got := composerKeySources(ctx); len(got) != 0 {
		t.Errorf("seating took %d keys from an UNCOMPARED payload; take/takeAll refuse "+
			"until one of [compared]'s two routes has run", len(got))
	}
	// CONTROL: the same payload, compared, yields the key -- so the assertion
	// above is measuring the gate and not a broken fixture.
	s2 := &syswSession{}
	s2.load(composerPayloadWith([]string{composerTestKeyRecord}, nil), [32]byte{}, false, true, true, true)
	ctx.sysw = s2
	if got := composerKeySources(ctx); len(got) != 1 {
		t.Fatalf("INCONCLUSIVE: the compared control yielded %d keys, want 1", len(got))
	}
}

// TestComposerSeatPromptIsTheSpecString is §8s's two seating prompts and
// §12 item 5's condition test for them: "Path N" is the OPERATOR's listed
// path index, never an emitted leaf index (§7d).
func TestComposerSeatPromptIsTheSpecString(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	// Slot @0 under wsh is path 1's first key of three.
	if got := composerSeatPrompt(st, 0); got != composerCopySeatPrompt(0, 1, 1, 3) {
		t.Errorf("composerSeatPrompt(@0) = %q, want %q", got, composerCopySeatPrompt(0, 1, 1, 3))
	}
	// Slot @3 is path 2's only key.
	if got := composerSeatPrompt(st, 3); got != composerCopySeatPrompt(3, 2, 1, 1) {
		t.Errorf("composerSeatPrompt(@3) = %q, want %q", got, composerCopySeatPrompt(3, 2, 1, 1))
	}
	assertModalBodyFits(t, "the §8s seating prompt", errorScreenBody, composerCopySeatPrompt(2, 1, 2, 3))
	assertModalBodyFits(t, "the §8s key-path seating prompt", errorScreenBody, composerCopySeatKeyPathPrompt(0))
}

// TestComposerPickListPagesAPayloadLargerThanAFrame is §9 item 7's reason for
// existing, driven end to end rather than at the primitive.
func TestComposerPickListPagesAPayloadLargerThanAFrame(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		rows := make([]string, 24)
		for i := range rows {
			rows[i] = composerNumberedLines(24)[i]
		}
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerPickScreen(ctx, &descriptorTheme, "Seat", composerCopySeatPrompt(2, 1, 2, 3), rows)
		})
		defer quit()
		content, ok := frame()
		if !ok {
			t.Fatal("the pick list never drew")
		}
		assertFrameHasBody(t, ink(), "the composer seating pick list")
		if uiContains(content, "entry 23 marker") {
			t.Error("all 24 rows drew on one frame, so this fixture no longer exercises paging")
		}
	})
}
```

`composerPayloadWith` and `composerTwoPathList` are two more lines in `gui/composer_fixtures_test.go`: the first wraps `&sysw.Payload{Public: ..., Secret: ...}`, the second returns `md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}}, {Keys: &md.KeySet{K: 1, N: 1}}}}`.

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerKeySources|^TestComposerSources|^TestComposerSeatPrompt|^TestComposerPickList' ./gui/ 2>&1 | tail -6`
Expected: FAIL to build -- `undefined: composerKeySources`, `composerSeatPrompt`, `composerPayloadWith`, `composerTwoPathList`.

- [ ] **Step 3: Write the sources**

Create `gui/composer_sources.go`:

```go
package gui

import (
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
	"seedhammer.com/mk"
	"seedhammer.com/sysw"
)

// The composer's seatable keys (SPEC §7d, C8). The seed source and its
// per-slot account rule are appended to this file by the seed task, which is
// why the import block above already carries what that half needs.
//
// THE COMPOSER DOES NOT CALL seatKeyCards, and §7d says why: that function
// seats a template that ALREADY declares its origins, by declaration match,
// for cards that ALREADY carry the template's stub (gui/key_card_seating.go
// :53-73). A composed template has no declarations yet and no card carries
// its stub, so layer 1 would refuse every card before an origin was ever
// compared. Seating here is SLOT-DIRECTED instead: the operator is asked, per
// emitted slot, which key goes in it, and seatKeyCards is what verifies the
// result afterwards (§12 item 6).
//
// CARD STUBS ARE IGNORED AT SEATING for the same reason. They are APPENDED
// when the card is re-minted, so one card seats into either engraved form and
// stays indexed to the wallets it already belonged to.
//
// THIS FILE CONSUMES FROM THE PAYLOAD, so its two functions are registered in
// gui/sysw_admit_oracle_test.go's syswConsumers and each HARD-CODES the one
// class it admits (§13 D7). A site that computed its class could not be
// reconciled against §3.3.2 at all.

// composerKeySources reads every key: record the payload holds.
//
// takeAll, not take: a composition seats a SET, and first-match would hand
// the flow one key for a four-slot policy. It inherits takeAll's refusal on
// an uncompared payload, which the door deliberately does not (the door
// counts through has()).
func composerKeySources(ctx *Context) []composerSource {
	if ctx.sysw == nil {
		return nil
	}
	records, ok := ctx.sysw.takeAll(sysw.ClassKey)
	if !ok {
		return nil
	}
	out := make([]composerSource, 0, len(records))
	for _, r := range records {
		kr, err := sysw.ParseKeyRecord(r)
		if err != nil {
			// Unreachable: a record that does not parse is ClassUnknown and
			// inert. Never consume a value from a call that returned an error.
			continue
		}
		out = append(out, composerSource{
			kind:        composerSourceKey,
			label:       composerKeyLabel(kr.Fingerprint, kr.Origin),
			fingerprint: kr.Fingerprint,
			fpPresent:   true,
			origin:      originComponents(kr.Origin),
			xpub:        kr.Xpub,
			seedID:      -1,
		})
	}
	return out
}

// composerCardSources reads every mk1 card the payload holds.
//
// cardSet, not takeAll: a card is a chunk SET, and one record of it completes
// nothing (F-76). cardSet groups the chunks so each card decodes.
func composerCardSources(ctx *Context) []composerSource {
	if ctx.sysw == nil {
		return nil
	}
	records, ok := ctx.sysw.cardSet(sysw.ClassMDMK)
	if !ok {
		return nil
	}
	var out []composerSource
	// A card's chunks are contiguous in `records` after grouping; mk.Decode
	// takes a complete set in any order and refuses an incomplete one, so a
	// growing window that decodes is exactly one card.
	for start := 0; start < len(records); {
		end := start + 1
		var card mk.Card
		decoded := false
		for ; end <= len(records); end++ {
			c, err := mk.Decode(records[start:end])
			if err == nil {
				card, decoded = c, true
				break
			}
		}
		if !decoded {
			// A record set that never decodes is an md1 card or a partial mk1;
			// neither is a seatable key. Advance by one rather than stopping,
			// so one unusable record cannot hide the cards after it.
			start++
			continue
		}
		path, err := bip32.ParsePath(card.Path)
		if err != nil {
			start = end
			continue
		}
		var fp [4]byte
		fpPresent := false
		if raw, err := hex.DecodeString(card.Fingerprint); err == nil && len(raw) == 4 {
			copy(fp[:], raw)
			fpPresent = true
		}
		out = append(out, composerSource{
			kind:        composerSourceCard,
			label:       composerKeyLabel(fp, path),
			fingerprint: fp,
			fpPresent:   fpPresent,
			origin:      originComponents(path),
			xpub:        card.Xpub,
			card:        card,
			seedID:      -1,
		})
		start = end
	}
	return out
}

// composerKeyLabel is §7d's label: fingerprint AND origin.
//
// BOTH, because two keys sharing a fingerprint is the NORMAL case (C5: one
// person in two paths holds two accounts from one master), and a fingerprint
// alone would render them identically on the one screen whose job is to tell
// them apart.
func composerKeyLabel(fp [4]byte, origin bip32.Path) string {
	return fmt.Sprintf("%x %s", fp, origin)
}

// composerSourceRow is one pick-list row. A used source is not offered again
// (C8's "remaining"); a SEED is never used up (C12), so its row stays.
func composerSourceRow(s composerSource) string {
	if s.kind == composerSourceSeed {
		return s.label + "  (any slots)"
	}
	return s.label
}

// composerSeatPrompt is §8s's prompt for one emitted slot.
//
// "Path N" IS THE OPERATOR'S LISTED PATH INDEX, never an emitted leaf index
// (§7d, stated twice there). Under tr the internal key is extracted as @0 and
// spends alone, which gets its own prompt.
func composerSeatPrompt(st *composerState, slot uint8) string {
	path, keyIdx, keyCount, keyPath := composerSlotPosition(st.list, slot)
	if keyPath {
		return composerCopySeatKeyPathPrompt(slot)
	}
	return composerCopySeatPrompt(slot, path, keyIdx, keyCount)
}

// composerSlotPosition maps an EMITTED slot index back to the operator's
// path, and reports whether it is the extracted taproot internal key.
//
// The emitted numbering is §5's: by first appearance in the emitted text,
// with an extracted internal key at @0. So under tr the FIRST-LISTED
// unlocked, unhashed one-key path becomes @0 and is no longer a leaf, and
// every other slot shifts. This walks the same rule rather than guessing it,
// which is why any edit that could move it discards assignments (§8j).
func composerSlotPosition(list md.PathList, slot uint8) (path, keyIdx, keyCount int, keyPath bool) {
	order := composerSlotOrder(list)
	if int(slot) >= len(order) {
		return 0, 0, 0, false
	}
	p := order[slot]
	return p.path, p.keyIdx, p.keyCount, p.keyPath
}

type composerSlotPos struct {
	path, keyIdx, keyCount int
	keyPath                bool
}

// composerSlotOrder lists, per emitted slot index, which of the operator's
// paths and which key within it that slot is.
//
// IT MUST AGREE WITH md.Compose's numbering. It is checked against
// md.Composed.Slots() by TestComposerSlotOrderAgreesWithTheCodec below, so a
// divergence is a test failure rather than a wrong prompt beside a right
// slot -- which is the shape that seats a key into the wrong seat silently.
func composerSlotOrder(list md.PathList) []composerSlotPos {
	var out []composerSlotPos
	internal := -1
	if list.Wrapper == md.ComposeTr {
		for i, p := range list.Paths {
			if p.Keys != nil && p.Keys.N == 1 && p.Lock == nil && p.Hash == nil {
				internal = i
				break
			}
		}
	}
	if internal >= 0 {
		out = append(out, composerSlotPos{path: internal + 1, keyIdx: 1, keyCount: 1, keyPath: true})
	}
	for i, p := range list.Paths {
		if i == internal || p.Keys == nil {
			continue
		}
		for k := 0; k < int(p.Keys.N); k++ {
			out = append(out, composerSlotPos{path: i + 1, keyIdx: k + 1, keyCount: int(p.Keys.N)})
		}
	}
	return out
}
```

Add to `gui/composer_sources_test.go` (its import block gains `seedhammer.com/md`):

```go
// TestComposerSlotOrderAgreesWithTheCodec is the one assertion that keeps
// every seating prompt honest. composerSlotOrder walks §5's numbering rule in
// Go; md.Compose walks the Rust port of it. If they disagree, the operator is
// told they are filling path 1's second key while the key lands in path 2 --
// a silent mis-seat, which is exactly the failure gui/key_card_seating.go
// :24-27 refuses to allow anywhere else.
func TestComposerSlotOrderAgreesWithTheCodec(t *testing.T) {
	digest := [32]byte{0x44}
	for _, list := range []md.PathList{
		composerTwoPathList(),
		{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}}, {Keys: &md.KeySet{K: 2, N: 3, Sorted: true}}}},
		{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}}, {Keys: &md.KeySet{K: 1, N: 1}}}},
		{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 2}, Hash: &digest},
			{Keys: &md.KeySet{K: 2, N: 2}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 7}}}},
	} {
		c, err := md.Compose(list)
		if err != nil {
			t.Fatalf("md.Compose: %v", err)
		}
		slots := c.Slots()
		order := composerSlotOrder(list)
		if len(order) != len(slots) {
			t.Fatalf("composerSlotOrder gives %d slots, md.Compose gives %d for %+v",
				len(order), len(slots), list)
		}
		for i, s := range slots {
			if int(s.Index) != i {
				t.Fatalf("md.Compose slot %d has Index %d; this walk assumes dense "+
					"ascending indices", i, s.Index)
			}
			if order[i].path != s.Path+1 {
				t.Errorf("slot @%d: the prompt says Path %d, md.Compose says path %d",
					i, order[i].path, s.Path+1)
			}
		}
	}
}
```

- [ ] **Step 4: Widen the consumption-site oracle and register the three sites it then finds**

In `gui/sysw_admit_oracle_test.go`, replace the selector match at `:127-133`:

```go
				var isOffer, isTake bool
				switch fun := call.Fun.(type) {
				case *ast.Ident:
					isOffer = strings.HasPrefix(fun.Name, "syswOffer")
				case *ast.SelectorExpr:
					isTake = fun.Sel.Name == "take"
				}
```

with:

```go
				var isOffer, isTake bool
				switch fun := call.Fun.(type) {
				case *ast.Ident:
					isOffer = strings.HasPrefix(fun.Name, "syswOffer")
				case *ast.SelectorExpr:
					// `take`, AND the two set-shaped consumptions. Matching
					// `take` alone left three shipped sites unchecked --
					// multisig_build_payload.go's cosigner source and
					// transaction.go's two record sweeps -- because a set is
					// not the shape `take` serves and they reach for takeAll
					// and cardSet instead. The composer's own sources would
					// have been a fourth, which is what made the gap worth
					// closing rather than noting.
					switch fun.Sel.Name {
					case "take", "takeAll", "cardSet":
						isTake = true
					}
				}
```

and add to `syswConsumers` (`:39-76`):

```go
	{"multisig_build_payload.go", "buildCosignerSource", []syswProgram{progMultisig},
		"Build Policy's cosigner cards, through cardSet -- unchecked here until the " +
			"matcher grew takeAll/cardSet at composer S3"},
	{"transaction.go", "payloadTransactions", []syswProgram{progTransaction},
		"Engrave Transaction's two record sweeps (ClassMt and ClassTx), through takeAll"},
	{"composer_sources.go", "composerKeySources", []syswProgram{progWalletPolicy},
		"the composer's key: records (§6a), admitted at Wallet Policy alone"},
	{"composer_sources.go", "composerCardSources", []syswProgram{progWalletPolicy},
		"the composer's mk1 card sources; their stubs are ignored at seating because " +
			"the composed policy does not exist yet (§7d)"},
```

- [ ] **Step 5: Run the tests, including the widened oracle**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposer|^TestEverySyswConsumptionSite' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL|.*consumption sites reconciled)'`
Expected: every test PASS, and the oracle logs a site count that is now **at least 3 higher than before the widening** (it logged the old count on the previous run; compare them and record both). If it reports a NEW consumption site that is not in `syswConsumers`, that is the widening working: register it with its programs rather than narrowing the matcher again.

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -2
git add gui/composer_sources.go gui/composer_sources_test.go gui/composer_fixtures_test.go gui/sysw_admit_oracle_test.go
git commit -s -F - <<'MSG'
gui: the composer's seatable keys, and an oracle that can see set-shaped consumption (composer S3 task B1)

Seating is slot-directed and does not call seatKeyCards: a composed template
has no declarations and no card carries its stub, so layer 1 would refuse
every card before an origin was compared. The consumption-site oracle matched
take by exact name, leaving three shipped takeAll/cardSet sites unchecked; it
now matches all three shapes and the sites are registered.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task B2: seeds as a source -- registry reuse, per-slot accounts by ordinal, passphrase, and the scrub seam

**Files:**
- Modify (`Add to`): `gui/composer_sources.go`
- Test: `gui/composer_seed_test.go`

**Interfaces:**
- Consumes: `seedRegistry` with `.add(label, m, passphrase, net) (int, error)`, `.bindPassphrase`, `.at(id) (registeredSeed, bool)`, `.count()`, `.usesPassphrase()`, `.scrub()` (`gui/multisig_build_slots.go:172-335`); `seedEntryFlowTitled` and `syswPassphraseFlowTitled` (the pair `buildSeedForSlot` uses, `gui/multisig_build.go:738-773`); `deriveAccountXpub(m, passphrase, net, path) (xpub string, masterFP uint32, err error)` (`gui/derive.go:19`); `md.DefaultOrigin(w, account) []md.PathComponent` (S2, §4f's table); `chaincfg.MainNetParams`.
- Produces: `func composerSeedSource(ctx *Context, th *Colors, st *composerState) (composerSource, bool)`; `func composerSeedAccountFor(st *composerState, slot uint8, srcIdx int) uint32` (the parameter indexes `st.sources`, NOT `composerSource.seedID` -- two different numbers on the same struct, and this line carried the other one's name for a round after the code was renamed); `var composerSeedHook func(bip39.Mnemonic)`.

**The scrub seam is `defer reg.scrub()`, NOT a hook.** `buildMultisigSeedHook` (`gui/multisig_build.go:38`) is a TEST-OBSERVATION seam: it fires once, right after `seedEntryFlowTitled` returns, purely so a test can capture the words before scrubbing, it is nil in production, and it zeroes nothing. The real C14 mechanism is `defer reg.scrub()` installed at the TOP of the flow before any seed exists (`gui/multisig_build.go:290-291`), so every exit is covered by construction. The composer flow already installs its own (Task A11); this task adds `composerSeedHook` for the same observation purpose and says in its own comment that it scrubs nothing.

**Per-slot accounts are by ORDINAL among the slots that master fills** (§4f, and the same rule `buildSlotSources` states at `gui/multisig_build.go:593-601`): the first slot a master fills gets account 0, the second account 1. **Keyed on the MASTER, not the seed id** -- keying on the id would mint the SAME key twice whenever one master was registered twice, which is what an operator does when they type one seed for two slots, and md's duplicate-key refusal would then reject a legitimate multi-account wallet.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_seed_test.go`:

```go
package gui

import (
	"testing"

	"seedhammer.com/md"
)

// TestComposerSeedAccountsAreOrdinalsPerMaster is §4f's account rule, and the
// C29/C5 case that makes it necessary: one seed at several slots must derive
// several DIFFERENT keys.
func TestComposerSeedAccountsAreOrdinalsPerMaster(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	st.sources = []composerSource{
		{kind: composerSourceSeed, seedID: 0, fingerprint: [4]byte{1, 2, 3, 4}, fpPresent: true},
		{kind: composerSourceSeed, seedID: 1, fingerprint: [4]byte{9, 9, 9, 9}, fpPresent: true},
	}
	st.assigned = make([]composerAssignment, 4)
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
	// Seed 0 at slots @0 and @2; seed 1 at @1.
	st.assigned[0].src, st.assigned[2].src, st.assigned[1].src = 0, 0, 1
	if got := composerSeedAccountFor(st, 0, 0); got != 0 {
		t.Errorf("the FIRST slot a master fills gets account %d, want 0", got)
	}
	if got := composerSeedAccountFor(st, 2, 0); got != 1 {
		t.Errorf("the SECOND slot the same master fills gets account %d, want 1 -- "+
			"account 0 twice would mint one key at two slots, which md refuses at encode", got)
	}
	if got := composerSeedAccountFor(st, 1, 1); got != 0 {
		t.Errorf("a different master's first slot gets account %d, want 0: the ordinal is "+
			"per MASTER, not per flow", got)
	}
}

// TestComposerSeedOriginFollowsSection4fPerWrapper pins the origin table,
// including the taproot 3' arm the shipped multisigScriptTypeComponent does
// not have (gui/multisig_build_slots.go:125-130 returns only 1' or 2').
func TestComposerSeedOriginFollowsSection4fPerWrapper(t *testing.T) {
	for _, tc := range []struct {
		w    md.ComposeWrapper
		want uint32
	}{
		{md.ComposeWsh, 2},
		{md.ComposeSh, 2},
		{md.ComposeShWsh, 1},
		{md.ComposeTr, 3},
	} {
		got := md.DefaultOrigin(tc.w, 0)
		if len(got) != 4 {
			t.Fatalf("wrapper %v: DefaultOrigin has %d components, want 4 (m/48'/0'/a'/T')", tc.w, len(got))
		}
		if got[0].Value != 48 || !got[0].Hardened {
			t.Errorf("wrapper %v: first component %+v, want 48'", tc.w, got[0])
		}
		if got[1].Value != 0 || !got[1].Hardened {
			t.Errorf("wrapper %v: coin %+v, want 0' (mainnet only, §4f and gui/policy_address.go:61)", tc.w, got[1])
		}
		if got[3].Value != tc.want || !got[3].Hardened {
			t.Errorf("wrapper %v: script type %+v, want %d'", tc.w, got[3], tc.want)
		}
	}
}

// TestComposerSeedHookIsObservationOnly is recon risk 6, as a gate: the hook
// must never be mistaken for the scrub. It fires and it zeroes nothing; the
// registry's scrub is what zeroes, and composerFlow installs it with a defer
// before any seed exists.
func TestComposerSeedHookIsObservationOnly(t *testing.T) {
	reg := &seedRegistry{}
	id, err := reg.add("t", composerTestMnemonic(t), "", composerMainNet())
	if err != nil {
		t.Fatal(err)
	}
	got, ok := reg.at(id)
	if !ok || len(got.Mnemonic) != 12 {
		t.Fatalf("the fixture seed did not register (%d words, ok=%v)", len(got.Mnemonic), ok)
	}
	// A NON-ZERO WORD, deliberately chosen: the "abandon" vector's first
	// eleven words are index 0, so asserting Mnemonic[0] != 0 before the
	// scrub would fail on a correctly registered seed, and asserting it is 0
	// afterwards would pass on a seed that was never written. The last word,
	// "about", is index 3.
	if got.Mnemonic[11] == 0 {
		t.Fatalf("the fixture's last word is %v, want a non-zero index -- this test "+
			"cannot distinguish scrubbed from unwritten otherwise", got.Mnemonic[11])
	}
	reg.scrub()
	after, _ := reg.at(id)
	for i, w := range after.Mnemonic {
		if w != 0 {
			t.Fatalf("word %d survived scrub as %v; C14 asks for Multisig Build's "+
				"treatment and this is that mechanism", i, w)
		}
	}
	if composerSeedHook != nil {
		t.Error("composerSeedHook is non-nil in a test that did not set it; it must be " +
			"nil in production, exactly as buildMultisigSeedHook is")
	}
}
```

`composerTestMnemonic` and `composerMainNet` are two more helpers in `gui/composer_fixtures_test.go`.

- [ ] **Step 2: Run to verify it fails, then write the seed source**

Add to `gui/composer_sources.go`:

```go
// composerSeedHook is a TEST-OBSERVATION seam, and nothing else. It fires
// once, right after the seed entry returns, so a test can capture the words
// before they are scrubbed. It is nil in production and IT SCRUBS NOTHING --
// exactly as buildMultisigSeedHook does not (gui/multisig_build.go:36-38).
//
// THE SCRUB IS THE REGISTRY'S, AND IT IS ONE SITE. composerFlow installs
// `defer st.reg.scrub()` at the top, before any seed exists, so every exit --
// a Back, a refusal screen, a ctx.Done unwind, a panic -- is covered by
// construction rather than by an implementer remembering to add a scrub to a
// new return. Copying this hook and not that defer would copy the wrong half.
var composerSeedHook func(bip39.Mnemonic)

// composerSeedSource takes a seed and registers it.
//
// The payload is offered before the keyboard, because §3.3.2 now admits
// ClassMnemonic here (Task A3) and seedEntryFlowTitled is the shared seam
// that does the offering. The passphrase is asked PER SEED, at that seed's
// entry (SPEC 4.1's rule, which buildSeedForSlot states at
// gui/multisig_build.go:725-737): one flow-global passphrase applied to N
// seeds would mint keys the operator can only re-derive with a pairing they
// never chose.
func composerSeedSource(ctx *Context, th *Colors, st *composerState) (composerSource, bool) {
	label := fmt.Sprintf("seed %d", st.reg.count()+1)
	mnemonic, ok := seedEntryFlowTitled(ctx, th, "Seed for the policy", label)
	if !ok {
		return composerSource{}, false
	}
	if composerSeedHook != nil {
		composerSeedHook(mnemonic)
	}
	// Registered IMMEDIATELY, before the passphrase screens can return early:
	// from this line the deferred scrub owns these words.
	seedID, err := st.reg.add(label, mnemonic, "", &chaincfg.MainNetParams)
	if err != nil {
		showError(ctx, th, "Seed", "Couldn't read that seed.")
		return composerSource{}, false
	}
	pp := &ChoiceScreen{
		Title:   "Passphrase " + label,
		Lead:    "Add a BIP-39 passphrase?",
		Choices: []string{"Skip", "Add passphrase"},
	}
	if sel, ok := pp.Choose(ctx, th); ok && sel == 1 {
		if pass, ok := syswPassphraseFlowTitled(ctx, th, "Passphrase "+label); ok {
			if err := st.reg.bindPassphrase(seedID, pass, &chaincfg.MainNetParams); err != nil {
				showError(ctx, th, "Seed", "Couldn't apply that passphrase.")
				return composerSource{}, false
			}
		}
	}
	seed, _ := st.reg.at(seedID)
	var fp [4]byte
	binary.BigEndian.PutUint32(fp[:], seed.MasterFP)
	return composerSource{
		kind: composerSourceSeed, label: label,
		fingerprint: fp, fpPresent: true, seedID: seedID,
	}, true
}

// composerSeedAccountFor is §4f's account rule: the slot's ordinal among the
// slots THIS MASTER fills, in ascending emitted slot index.
//
// KEYED ON THE MASTER, NOT THE SEED ID, and buildSlotSources states the
// reason at gui/multisig_build.go:593-601: keying on the id would mint the
// SAME key twice whenever one master was registered twice -- which is what an
// operator does when they type one seed for two slots -- and md's duplicate
// key refusal would then reject a legitimate multi-account wallet.
// srcIdx indexes st.sources, NOT composerSource.seedID -- two different
// numbers on the same struct, and the parameter used to carry the other one's
// name while every caller passed this one.
func composerSeedAccountFor(st *composerState, slot uint8, srcIdx int) uint32 {
	want := st.sources[srcIdx].fingerprint
	n := uint32(0)
	for i := 0; i < int(slot) && i < len(st.assigned); i++ {
		a := st.assigned[i]
		if a.src < 0 || a.src >= len(st.sources) {
			continue
		}
		s := st.sources[a.src]
		if s.kind == composerSourceSeed && s.fingerprint == want {
			n++
		}
	}
	return n
}

// composerSeedDerive fills one assignment from a seed at its own account.
func composerSeedDerive(st *composerState, slot uint8, srcIdx int) (composerAssignment, error) {
	src := st.sources[srcIdx]
	account := composerSeedAccountFor(st, slot, srcIdx)
	origin := md.DefaultOrigin(st.list.Wrapper, account)
	seed, ok := st.reg.at(src.seedID)
	if !ok {
		return composerAssignment{}, errors.New("composer: no seed for that slot")
	}
	path := make(bip32.Path, 0, len(origin))
	for _, c := range origin {
		v := c.Value
		if c.Hardened {
			v += hdkeychain.HardenedKeyStart
		}
		path = append(path, v)
	}
	xpub, masterFP, err := deriveAccountXpub(seed.Mnemonic, seed.Passphrase, &chaincfg.MainNetParams, path)
	if err != nil {
		return composerAssignment{}, err
	}
	var fp [4]byte
	binary.BigEndian.PutUint32(fp[:], masterFP)
	return composerAssignment{
		src: srcIdx, account: account, origin: origin,
		fingerprint: fp, fpPresent: true, xpub: xpub,
	}, nil
}
```

(The imports this half needs -- `encoding/binary`, `errors`, `hdkeychain`, `chaincfg` and `bip39` -- are already in the file's import block, written there by the sources task above so the two halves of one file are never in a half-imported state.)

- [ ] **Step 4: Run, gofmt, commit**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerSeed' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`4`**, not three: `composer_seed_test.go` declares three functions, and the `^TestComposerSeed` prefix also matches Task A3's `TestComposerSeedInAPayloadStillRaisesF1AtLoad`. Counted rather than named because the filter is what an implementer will actually type.

```bash
git add gui/composer_sources.go gui/composer_seed_test.go gui/composer_fixtures_test.go
git commit -s -F - <<'MSG'
gui: seeds as a composer key source, at one account per slot per master (composer S3 task B2)

The account is the slot's ordinal among the slots THAT MASTER fills, keyed on
the fingerprint rather than the registry id, for the reason buildSlotSources
gives: one seed typed for two slots would otherwise mint the same key twice.
composerSeedHook is observation-only and says so; the scrub is the registry's
deferred one, installed at flow entry.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task B3: discard-on-numbering-change (§7d, §8j) -- the rule's semantics and its tests

**Files:**
- Test: `gui/composer_discard_test.go` (`gui/composer_discard.go` itself is created in **Task A5**, with the shape flow that calls it)

**Interfaces:**
- Produces: `func composerShapeSignature(list md.PathList) string`; `func composerDiscardAssignments(st *composerState)`; `func composerApplyShapeEdit(st *composerState, edit func()) bool`. `composerAnySlotAssigned` and `composerShapeGuard` already exist, in `gui/composer_state.go` and `gui/composer_shape.go`, with their real bodies -- this task adds the SIGNATURE that decides whether an edit actually renumbered anything.

§7d: any change that moves slot NUMBERING -- **the wrapper, the path count, or a path's key count** -- after at least one slot has been assigned discards ALL assignments, and the operator is told before the edit is accepted (§8j). A lock or hash edit moves no slot, keeps assignments, and re-shows the stub screen. With no slot yet assigned there is nothing to discard and §8j does not fire.

The guard runs BEFORE the edit, so it cannot know what the operator is about to change. It therefore takes the conservative reading §7d's own wording forces: entering an editor that CAN move numbering asks first. The signature comparison after the edit is what decides whether assignments are actually dropped, so answering "continue" and then changing only a lock keeps them.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_discard_test.go`:

```go
package gui

import (
	"testing"

	"seedhammer.com/md"
)

// TestComposerShapeSignatureMovesExactlyWithSlotNumbering is §7d's rule,
// stated as an equivalence: the signature changes for the wrapper, the path
// count and a key count, and for NOTHING ELSE.
func TestComposerShapeSignatureMovesExactlyWithSlotNumbering(t *testing.T) {
	digest := [32]byte{0x55}
	base := composerTwoPathList()
	sig := composerShapeSignature(base)

	// MOVES numbering.
	wrapper := base
	wrapper.Wrapper = md.ComposeTr
	if composerShapeSignature(wrapper) == sig {
		t.Error("changing the wrapper does not move the signature, but tr extracts an " +
			"internal key as @0 and renumbers every slot (§5)")
	}
	fewer := md.PathList{Wrapper: base.Wrapper, Paths: base.Paths[:1]}
	if composerShapeSignature(fewer) == sig {
		t.Error("removing a path does not move the signature")
	}
	wider := composerTwoPathList()
	wider.Paths[0].Keys = &md.KeySet{K: 2, N: 4, Sorted: true}
	if composerShapeSignature(wider) == sig {
		t.Error("changing a path's key count does not move the signature")
	}

	// DOES NOT move numbering (§7d: assignments are KEPT).
	locked := composerTwoPathList()
	locked.Paths[1].Lock = &md.Lock{Kind: md.LockOlderBlocks, Value: 42}
	if composerShapeSignature(locked) != sig {
		t.Error("adding a lock moved the signature, so assignments would be discarded " +
			"for an edit that renumbers nothing (§7d rules them KEPT)")
	}
	hashed := composerTwoPathList()
	hashed.Paths[1].Hash = &digest
	if composerShapeSignature(hashed) != sig {
		t.Error("adding a hash moved the signature; §7d keeps assignments across it")
	}
	// The THRESHOLD is not the key count: k moves no slot.
	thresh := composerTwoPathList()
	thresh.Paths[0].Keys = &md.KeySet{K: 3, N: 3, Sorted: true}
	if composerShapeSignature(thresh) != sig {
		t.Error("changing k moved the signature; only n changes how many slots exist")
	}
}

// TestComposerDiscardIsSilentWithNothingSeated is §7d's last clause: with no
// slot yet assigned there is nothing to discard and §8j does not fire. A
// warning that fires when nothing is at stake is a warning the operator
// learns to tap through.
func TestComposerDiscardIsSilentWithNothingSeated(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	if composerAnySlotAssigned(st) {
		t.Fatal("a fresh state reports an assignment")
	}
	st.assigned = make([]composerAssignment, 4)
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
	if composerAnySlotAssigned(st) {
		t.Fatal("an all-unassigned slice reports an assignment; src must be -1 for unseated")
	}
	st.assigned[2].src = 0
	if !composerAnySlotAssigned(st) {
		t.Fatal("a seated slot is not detected, so §8j would never fire")
	}
	assertModalBodyFits(t, "the §8j discard confirm", confirmWarningBody,
		composerConfirmBody(composerCopyEditClearsKeys()))
}

// TestComposerDiscardClearsEverySeat: a partial discard is the state that
// seats keys into the wrong slots silently, which is the whole reason §7d
// discards ALL assignments rather than the ones it can prove moved.
func TestComposerDiscardClearsEverySeat(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	st.assigned = []composerAssignment{{src: 0}, {src: 1}, {src: -1}, {src: 2}}
	st.sources = []composerSource{{used: true}, {used: true}, {used: true}}
	composerDiscardAssignments(st)
	for i, a := range st.assigned {
		if a.src != -1 {
			t.Errorf("slot @%d still holds source %d after a discard", i, a.src)
		}
	}
	for i, s := range st.sources {
		if s.used {
			t.Errorf("source %d is still marked used after a discard, so it would never "+
				"be offered again for the slots it no longer fills", i)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerShapeSignature|^TestComposerDiscard' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `# seedhammer.com/gui [seedhammer.com/gui.test]` then `undefined: composerShapeSignature`, `undefined: composerDiscardAssignments`.

- [ ] **Step 3: Write the signature and the discard**

`gui/composer_discard.go` is created in **Task A5**, with the shape flow that calls it.

**Why it moved.** `composerShapeFlow` (Task A5) wires `composerApplyShapeEdit` around every edit that can renumber, so the callee has to exist when Part A does -- and it must, because Part A has to compile alone. Measured by the `GATE_UNTIL` run: with this file in Part B, Part A failed `go vet` with `undefined: composerApplyShapeEdit`, three times. §8j does not FIRE in Part A -- nothing is seated, so `composerShapeGuard` returns true without drawing -- but the machinery it guards is shape machinery and belongs with the shape.

This task keeps the rule's own tests, and they are what pin the semantics: the signature moves for the wrapper, the path count and a key count, and for nothing else.



Wire `composerApplyShapeEdit` around the two edit calls in `composerShapeFlow` (`gui/composer_shape.go`), so the guard's confirm and the actual discard are decided separately: the guard asks before an editor that CAN renumber, and the signature comparison after it decides whether the seats are dropped.

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerShapeSignature|^TestComposerDiscard' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: three PASS (`TestComposerShapeSignatureMovesExactlyWithSlotNumbering`, `TestComposerDiscardIsSilentWithNothingSeated`, `TestComposerDiscardClearsEverySeat`); `ok seedhammer.com/gui`. The silent-discard test logs the §8j confirm's headroom -- measured: `118 chars drawn in full, headroom 378 chars (margin 80)`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_discard_test.go
git commit -s -F - <<'MSG'
gui: discard every seat when the shape moves slot numbering, and only then (composer S3 task B3)

The signature carries the wrapper, the path count and each path's key count,
and nothing else: a lock or a hash edit renumbers nothing and keeps the seats,
which is what section 7d rules and what the guard on the wrong arms was
telling operators otherwise.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---
### Task B4: `gui/composer_review.go` -- the mapping review, the §4f invariant refusal, the same-xpub refusal, C29 and §8k

**Files:**
- Create: `gui/composer_review.go`
- Test: `gui/composer_review_test.go`

**Interfaces:**
- Consumes: `composerState`, `composerReadScreen`, `showError`, `composerCopy*`; `md.PathComponent`; `bip32.Path`.
- Produces: `func composerOriginKey(o []md.PathComponent) string`; `func composerInvariantViolation(st *composerState) bool`; `func composerDuplicateXpub(st *composerState) (a, b uint8, dup bool)`; `func composerSharedSeedInPath(st *composerState) []composerSharedSeed`; `func composerPersonInTwoPaths(st *composerState) bool`; `func composerMappingLines(st *composerState) []string`; `func composerMappingReview(ctx *Context, th *Colors, st *composerState) bool`.

**§4f's invariant, verbatim, is what §8v enforces:** no two slots of a produced template declare the same origin unless BOTH declare a fingerprint and those fingerprints differ. The reason is measured, not stylistic -- `slotMatchesCard` skips the fingerprint test when the template declares none (`gui/key_card_seating.go:151-159`), so two slots at one origin where only ONE declares a fingerprint let a single card fill both silently: a mis-seated key presented to the operator as reviewed. Two with no fingerprints at all are refused later by `errSeatSlotContested` and are simply unseatable.

**The origins are printed VERBATIM**, with the note that the device cannot confirm the key was derived there. A `key:` record's account and interior components are declarations this device cannot verify (F-217; `mk` cannot either), and the review says so rather than implying a check it did not run.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_review_test.go`:

```go
package gui

import (
	"strings"
	"testing"
)

// TestComposerInvariantRefusesTwoSlotsAtOneOriginWithOneFingerprint is §4f's
// invariant and §8v. The asymmetric case is the dangerous one and is what
// this asserts first: slotMatchesCard skips the fingerprint test when the
// TEMPLATE declares none, so one card fills both slots and the operator is
// shown a mis-seated key as reviewed.
func TestComposerInvariantRefusesTwoSlotsAtOneOriginWithOneFingerprint(t *testing.T) {
	origin := composerTestOrigin(2, 0) // m/48'/0'/0'/2'
	for _, tc := range []struct {
		name string
		a, b composerAssignment
		want bool
	}{
		{"same origin, neither has a fingerprint",
			composerAssignment{src: 0, origin: origin},
			composerAssignment{src: 1, origin: origin}, true},
		{"same origin, only one has a fingerprint",
			composerAssignment{src: 0, origin: origin, fingerprint: [4]byte{1}, fpPresent: true},
			composerAssignment{src: 1, origin: origin}, true},
		{"same origin, two DIFFERENT fingerprints",
			composerAssignment{src: 0, origin: origin, fingerprint: [4]byte{1}, fpPresent: true},
			composerAssignment{src: 1, origin: origin, fingerprint: [4]byte{2}, fpPresent: true}, false},
		{"same origin, the SAME fingerprint twice",
			composerAssignment{src: 0, origin: origin, fingerprint: [4]byte{1}, fpPresent: true},
			composerAssignment{src: 1, origin: origin, fingerprint: [4]byte{1}, fpPresent: true}, true},
		{"different origins",
			composerAssignment{src: 0, origin: composerTestOrigin(2, 0)},
			composerAssignment{src: 1, origin: composerTestOrigin(2, 1)}, false},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st := &composerState{list: composerTwoPathList(),
				assigned: []composerAssignment{tc.a, tc.b}}
			if got := composerInvariantViolation(st); got != tc.want {
				t.Errorf("composerInvariantViolation = %v, want %v", got, tc.want)
			}
		})
	}
	assertModalBodyFits(t, "the §8v same-origin refusal", errorScreenBody,
		composerCopySameOriginFewFingerprints())
}

// TestComposerRefusesTwoSlotsResolvingToTheSameXpub is §7d's refusal, and
// BIP-388 line 193's pairwise-distinct rule. md refuses it only at ENCODE, so
// without this the operator would meet a codec error instead of a review that
// names both slots.
func TestComposerRefusesTwoSlotsResolvingToTheSameXpub(t *testing.T) {
	st := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, xpub: composerTestXpubA}, {src: 1, xpub: composerTestXpubB},
		{src: 2, xpub: composerTestXpubA}, {src: 3, xpub: ""},
	}}
	a, b, dup := composerDuplicateXpub(st)
	if !dup {
		t.Fatal("two slots holding one xpub are not detected")
	}
	if a != 0 || b != 2 {
		t.Errorf("the refusal names slots @%d and @%d, want @0 and @2", a, b)
	}
	// An UNSEATED slot is not a duplicate of another unseated slot: both have
	// no xpub, and refusing on that would refuse every keyless template.
	st.assigned = []composerAssignment{{src: -1}, {src: -1}}
	if _, _, dup := composerDuplicateXpub(st); dup {
		t.Error("two unseated slots were reported as the same key")
	}
}

// TestComposerC29WarningFiresInsideOnePathAndNotAcross is C29, both arms, and
// §12 item 5's condition test for §8g's two bodies.
func TestComposerC29WarningFiresInsideOnePathAndNotAcross(t *testing.T) {
	fp := [4]byte{0xaa}
	// composerTwoPathList is 2-of-3 then 1-of-1: slots @0..@2 are path 1.
	inOne := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, fingerprint: fp, fpPresent: true},
		{src: 0, fingerprint: fp, fpPresent: true},
		{src: 1},
		{src: 2},
	}}
	inOne.sources = []composerSource{
		{kind: composerSourceSeed, fingerprint: fp, fpPresent: true},
		{kind: composerSourceKey}, {kind: composerSourceKey},
	}
	shared := composerSharedSeedInPath(inOne)
	if len(shared) != 1 {
		t.Fatalf("one seed at two slots INSIDE one path gives %d warnings, want 1", len(shared))
	}
	// The FIRST body when the shared slots reach the threshold, the second
	// otherwise (§8g's own heading). Path 1 is 2-of-3 and two slots are
	// shared, so the threshold is reached.
	body := composerSharedSeedBody(shared[0])
	if !strings.Contains(body, "can be satisfied by one person") {
		t.Errorf("two shared slots in a 2-of-3 use the below-threshold body:\n%s", body)
	}

	// ACROSS paths is C5's normal case: an informational line plus §8k, never
	// the warning.
	across := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, fingerprint: fp, fpPresent: true},
		{src: 1}, {src: 2},
		{src: 0, fingerprint: fp, fpPresent: true},
	}}
	across.sources = inOne.sources
	if got := composerSharedSeedInPath(across); len(got) != 0 {
		t.Errorf("one seed across two paths raised %d C29 warnings; C5 makes it normal", len(got))
	}
	if !composerPersonInTwoPaths(across) {
		t.Error("one fingerprint in two paths does not trip the §8k informational line")
	}
	assertModalBodyFits(t, "the §8g at-threshold body", errorScreenBody,
		composerCopySameSeedThreshold([]uint8{1, 2}, 2, 3))
	assertModalBodyFits(t, "the §8g below-threshold body", errorScreenBody,
		composerCopySameSeedBelow([]uint8{1, 2}, 3))
	assertModalBodyFits(t, "the §8k two-paths line", errorScreenBody, composerCopyPersonInTwoPaths())
}

// TestComposerMappingLinesPrintOriginsVerbatimAndSayWhatIsNotChecked is
// §7d's mapping review and F-217: the account and every interior component
// are declarations this device cannot verify, and the screen must say so
// rather than imply a check it did not run.
func TestComposerMappingLinesPrintOriginsVerbatimAndSayWhatIsNotChecked(t *testing.T) {
	st := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, origin: composerTestOrigin(2, 0), fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, fpPresent: true, xpub: composerTestXpubA},
		{src: 1, origin: composerTestOrigin(2, 1), fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, fpPresent: true, xpub: composerTestXpubB},
		{src: -1}, {src: -1},
	}}
	st.sources = []composerSource{{kind: composerSourceKey}, {kind: composerSourceKey}}
	joined := strings.Join(composerMappingLines(st), "\n")
	for _, want := range []string{
		"@0", "@1", "73c5da0a",
		"48'/0'/0'/2'", "48'/0'/1'/2'",
		"cannot confirm",
	} {
		if !strings.Contains(joined, want) {
			t.Errorf("the mapping review does not say %q:\n%s", want, joined)
		}
	}
}
```

`composerTestOrigin(scriptType, account uint32) []md.PathComponent` is one more helper in `gui/composer_fixtures_test.go`, built from `md.DefaultOrigin` so the fixture and the production table cannot disagree.

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerInvariant|^TestComposerRefusesTwoSlots|^TestComposerC29|^TestComposerMappingLines' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `undefined: composerInvariantViolation`, `undefined: composerDuplicateXpub`, `undefined: composerSharedSeedInPath`, `undefined: composerMappingLines`.

- [ ] **Step 3: Write the review**

Create `gui/composer_review.go` with:

```go
package gui

import (
	"fmt"
	"strings"

	"seedhammer.com/md"
)

// The mapping review (SPEC §7d), the last screen before consent and the only
// one that shows slot, fingerprint and origin together.
//
// THE ORIGINS ARE PRINTED VERBATIM, with the note that the device cannot
// confirm the key was derived there. A key: record's origin proves the
// xpub's DEPTH and its LAST COMPONENT against the declared path and nothing
// else; the account and every interior component are declarations neither
// this device nor mk can verify (F-217). Printing them without that sentence
// would imply a check that was never run.

// composerOriginKey renders an origin for comparison. Structural, not
// textual: bip32.Path.String renders `m/48h/...` while an mk1 card carries
// `m/48'/...`, and a string comparison would match neither
// (gui/key_card_seating.go:130-137 states the same lesson).
func composerOriginKey(o []md.PathComponent) string {
	var b strings.Builder
	for _, c := range o {
		fmt.Fprintf(&b, "%d", c.Value)
		if c.Hardened {
			b.WriteByte('h')
		}
		b.WriteByte('/')
	}
	return b.String()
}

// composerInvariantViolation is §4f's pairwise-distinguishability invariant:
// no two slots may declare the same origin unless BOTH declare a fingerprint
// and those fingerprints DIFFER.
//
// The asymmetric case -- one fingerprint present, one absent -- is the
// dangerous one and is why the rule is not simply "two fingerprints".
// slotMatchesCard skips the fingerprint test when the template declares none
// (gui/key_card_seating.go:151-159), so one card fills BOTH slots and the
// operator is shown a mis-seated key as reviewed.
func composerInvariantViolation(st *composerState) bool {
	type seat struct {
		fp        [4]byte
		fpPresent bool
	}
	byOrigin := map[string][]seat{}
	for _, a := range st.assigned {
		if a.src < 0 {
			// AN UNSEATED SLOT IS NOT A COLLISION. Its origin is nil here, so
			// every unseated slot hashed to "" and two of them looked like two
			// keys at one origin with no fingerprints -- refusing §7f's
			// partially seated form and C26's key-less template with §8v, a
			// body about keys that are not there. The origins those slots will
			// DECLARE are §4f's lowest-free accounts, assigned by the codec
			// when the template is emitted, and the real invariant is checked
			// on the decoded md1 by composerSelfCheck.
			continue
		}
		k := composerOriginKey(a.origin)
		byOrigin[k] = append(byOrigin[k], seat{a.fingerprint, a.fpPresent})
	}
	for _, seats := range byOrigin {
		if len(seats) < 2 {
			continue
		}
		for i := 0; i < len(seats); i++ {
			for j := i + 1; j < len(seats); j++ {
				if !seats[i].fpPresent || !seats[j].fpPresent {
					return true
				}
				if seats[i].fp == seats[j].fp {
					return true
				}
			}
		}
	}
	return false
}

// composerDuplicateXpub is §7d's same-xpub refusal (BIP-388 line 193's
// pairwise-distinct rule). md refuses it only at ENCODE, so catching it here
// is the difference between a review that names both slots and a codec error
// the operator cannot act on.
func composerDuplicateXpub(st *composerState) (uint8, uint8, bool) {
	seen := map[string]int{}
	for i, a := range st.assigned {
		if a.xpub == "" {
			continue
		}
		if j, ok := seen[a.xpub]; ok {
			return uint8(j), uint8(i), true
		}
		seen[a.xpub] = i
	}
	return 0, 0, false
}
```

and, in the same file:

```go
// composerSharedSeed is one C29 finding: the slots INSIDE one path that share
// a fingerprint, with that path's threshold.
type composerSharedSeed struct {
	slots []uint8
	k, n  int
}

// composerSharedSeedInPath finds C29's case: one seed (one fingerprint) at
// two or more slots INSIDE ONE path. Across paths is C5's NORMAL case and
// gets an informational line instead (§7g).
func composerSharedSeedInPath(st *composerState) []composerSharedSeed {
	order := composerSlotOrder(st.list)
	byPath := map[int]map[[4]byte][]uint8{}
	for i, a := range st.assigned {
		if a.src < 0 || i >= len(order) || !a.fpPresent {
			continue
		}
		p := order[i].path
		if byPath[p] == nil {
			byPath[p] = map[[4]byte][]uint8{}
		}
		byPath[p][a.fingerprint] = append(byPath[p][a.fingerprint], uint8(i))
	}
	var out []composerSharedSeed
	for i, p := range st.list.Paths {
		if p.Keys == nil {
			continue
		}
		for _, slots := range byPath[i+1] {
			if len(slots) < 2 {
				continue
			}
			out = append(out, composerSharedSeed{slots: slots, k: int(p.Keys.K), n: int(p.Keys.N)})
		}
	}
	return out
}

// composerSharedSeedBody picks between §8g's two bodies: the FIRST when the
// shared slots REACH the threshold (one person can satisfy the path alone),
// the second otherwise (they hold some of what it needs).
func composerSharedSeedBody(c composerSharedSeed) string {
	if len(c.slots) >= c.k {
		return composerCopySameSeedThreshold(c.slots, c.k, c.n)
	}
	return composerCopySameSeedBelow(c.slots, c.k)
}

// composerPersonInTwoPaths is C5's normal case: one fingerprint seated in two
// DIFFERENT paths. It earns §8k's informational line, never a warning.
func composerPersonInTwoPaths(st *composerState) bool {
	order := composerSlotOrder(st.list)
	paths := map[[4]byte]map[int]bool{}
	for i, a := range st.assigned {
		if a.src < 0 || i >= len(order) || !a.fpPresent {
			continue
		}
		if paths[a.fingerprint] == nil {
			paths[a.fingerprint] = map[int]bool{}
		}
		paths[a.fingerprint][order[i].path] = true
		if len(paths[a.fingerprint]) > 1 {
			return true
		}
	}
	return false
}

// composerMappingLines is the review body: slot, fingerprint, origin VERBATIM,
// then what the device did NOT check.
func composerMappingLines(st *composerState) []string {
	var lines []string
	for i, a := range st.assigned {
		if a.src < 0 {
			lines = append(lines, fmt.Sprintf("@%d: unseated", i))
			continue
		}
		fp := "no fingerprint"
		if a.fpPresent {
			fp = fmt.Sprintf("%x", a.fingerprint)
		}
		lines = append(lines, fmt.Sprintf("@%d: %s %s", i, fp, composerOriginText(a.origin)))
	}
	// F-217, said plainly. The xpub's DEPTH and its LAST component are checked
	// against the declared path; the account and every interior component are
	// declarations neither this device nor mk can verify. Printing the origin
	// without this sentence would imply a check that was never run.
	lines = append(lines, "", "This device cannot confirm a key was derived at the origin it declares.")
	for _, c := range composerSharedSeedInPath(st) {
		lines = append(lines, "", composerSharedSeedBody(c))
	}
	if composerPersonInTwoPaths(st) {
		lines = append(lines, "", composerCopyPersonInTwoPaths())
	}
	return lines
}

// composerOriginText renders an origin the way a card writes it, with `'` for
// hardening -- the spelling mk.Card.Path carries (gui/key_card_seating.go
// :130-137 names the two notations and why they are compared structurally).
func composerOriginText(o []md.PathComponent) string {
	var b strings.Builder
	b.WriteByte('m')
	for _, c := range o {
		fmt.Fprintf(&b, "/%d", c.Value)
		if c.Hardened {
			b.WriteByte('\'')
		}
	}
	return b.String()
}

// composerMappingReview refuses first, then shows. Back KEEPS assignments
// (§7d): of everything Back can discard on this path, a seating the operator
// has just worked through is among the most expensive.
func composerMappingReview(ctx *Context, th *Colors, st *composerState) bool {
	if composerInvariantViolation(st) {
		showError(ctx, th, "Key mapping", composerCopySameOriginFewFingerprints())
		return false
	}
	if a, b, dup := composerDuplicateXpub(st); dup {
		showError(ctx, th, "Key mapping", fmt.Sprintf(
			"Slots @%d and @%d hold the same key. Every slot needs a different key.", a, b))
		return false
	}
	return composerReadScreen(ctx, th, "Key mapping", composerMappingLines(st))
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerInvariant|^TestComposerRefusesTwoSlots|^TestComposerC29|^TestComposerMappingLines' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: four top-level PASS with **five sub-tests** under the invariant test (`same origin, neither has a fingerprint`, `same origin, only one has a fingerprint`, `same origin, two DIFFERENT fingerprints`, `same origin, the SAME fingerprint twice`, `different origins`); `ok seedhammer.com/gui`. Four `assertModalBodyFits` headrooms are logged -- measured: §8v 436, §8g at-threshold 436, §8g below-threshold 436, §8k 476, all against a margin of 80.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_review.go gui/composer_review_test.go
git commit -s -F - <<'MSG'
gui: the mapping review -- verbatim origins, the section 4f invariant, C29 and the two-paths line (composer S3 task B4)

The invariant skips unseated slots: their origins are the codec's to assign
when it emits the template, and grouping them by an absent origin refused both
legal key-less forms with a body about keys that are not there. The real
invariant is re-checked on the decoded md1 by the self-check.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task B5: the seating flow and the §8p shortfall

**Files:**
- Create: `gui/composer_seat.go`
- Test: `gui/composer_seat_test.go`

**Interfaces:**
- Produces: `func composerAssignableSlots(st *composerState) int`; `func composerUnfilledSlots(st *composerState) []uint8`; `func composerSeatFlow(ctx *Context, th *Colors, st *composerState) bool`.

**Seating is all-or-nothing** (§7d): fewer assignable slots than slots refuses at the transition with §8p -- the count line and the unfilled slots named -- and offers Back-to-edit or "engrave as a keyless template" (the §7f partially seated form). **No cause is guessed**: the C5 lesson is taught at the shape step by §8k, and a second explanation on the screen that refuses is one more thing that can be wrong.

**"Keys available" counts ASSIGNABLE SLOTS, not sources** (§7d): a `key:` record and an mk1 card are used at most once, and a SEED is a source of as many slots as the operator assigns. So a payload holding one seed can fill a 9-slot policy and `composerAssignableSlots` returns the slot count itself whenever any seed is present.

- [ ] **Step 1: the failing test**

Create `gui/composer_seat_test.go`:

```go
package gui

import (
	"strings"
	"testing"
)

// TestComposerAssignableSlotsCountsSeatsNotSources is §7d's counting rule,
// and getting it wrong in either direction is a refusal the operator cannot
// act on: too low refuses a payload that would have worked, too high walks
// them through seating and refuses at the end.
func TestComposerAssignableSlotsCountsSeatsNotSources(t *testing.T) {
	st := &composerState{list: composerTwoPathList()} // 4 slots
	st.sources = []composerSource{{kind: composerSourceKey}, {kind: composerSourceKey}}
	if got := composerAssignableSlots(st); got != 2 {
		t.Errorf("two key records fill %d slots, want 2", got)
	}
	st.sources = append(st.sources, composerSource{kind: composerSourceSeed, seedID: 0})
	if got := composerAssignableSlots(st); got != composerSlotCount(st.list) {
		t.Errorf("with a seed present %d slots are assignable, want all %d: a seed fills "+
			"any number of slots (C12, §4f)", got, composerSlotCount(st.list))
	}
	st.sources = st.sources[:2]
	st.assigned = []composerAssignment{{src: 0}, {src: 1}, {src: -1}, {src: -1}}
	if got := composerUnfilledSlots(st); len(got) != 2 || got[0] != 2 || got[1] != 3 {
		t.Errorf("unfilled slots = %v, want [2 3]", got)
	}
	assertModalBodyFits(t, "the §8p shortfall refusal", errorScreenBody,
		composerCopyShortfall(4, 3, []uint8{3}))
	// The §8p body NAMES no cause, and that is the rule rather than an
	// omission (§7d): the C5 lesson is taught at the shape step by §8k.
	body := composerCopyShortfall(4, 2, []uint8{2, 3})
	for _, forbidden := range []string{"because", "seed", "card"} {
		if strings.Contains(strings.ToLower(body), forbidden) {
			t.Errorf("the shortfall body guesses a cause (%q):\n%s", forbidden, body)
		}
	}
}

// TestComposerSeatingCompleteTracksEverySlot is the predicate the transition
// out of seating turns on: all-or-nothing (§7d), so "complete" must mean every
// slot and not most of them.
func TestComposerSeatingCompleteTracksEverySlot(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	st.assigned = make([]composerAssignment, composerSlotCount(st.list))
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
	if composerSeatingComplete(st) {
		t.Fatal("an all-unseated template reports complete seating")
	}
	for i := range st.assigned {
		st.assigned[i].src = i
	}
	if !composerSeatingComplete(st) {
		t.Fatal("a fully seated template does not report complete seating")
	}
	// ONE slot short is NOT complete, which is the case the all-or-nothing
	// rule exists for: a partial seating is exactly the state that produces a
	// plausible wrong address.
	st.assigned[len(st.assigned)-1].src = -1
	if composerSeatingComplete(st) {
		t.Errorf("a template one slot short of seated reports complete; §7d refuses at " +
			"the transition instead, with §8p's counts")
	}
	if got := composerUnfilledSlots(st); len(got) != 1 {
		t.Errorf("one slot short leaves %d unfilled, want 1", len(got))
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerAssignableSlots|^TestComposerSeatingComplete' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `undefined: composerAssignableSlots`, `undefined: composerUnfilledSlots`, `undefined: composerSeatingComplete`.

- [ ] **Step 3: Write the seating flow**

Create `gui/composer_seat.go`:

```go
package gui

import (
	"seedhammer.com/md"
)

// Slot-directed seating (SPEC §7d).
//
// IT WALKS SLOTS, NOT SOURCES, and that is C8's model: the operator is asked,
// per emitted slot, which key goes in it, rather than being handed a pile of
// cards to place. The pile version is what seatKeyCards does for a template
// that already declares its origins, and it is why that function refuses to
// guess when two cards claim one slot (gui/key_card_seating.go:24-27): a
// misassignment does not fail, it derives a different wallet's address and
// shows it to the operator as proof.
//
// BACK STEPS BACK ONE SLOT rather than abandoning seating, the same directive
// gatherSlotSeeds follows (gui/multisig_build.go:725-737) and for the same
// measured reason: it previously returned on any Back, so mistyping the
// SECOND slot's seed also discarded the first.

// composerAssignableSlots is §7d's counting rule: records and cards are used
// AT MOST ONCE, and a SEED is a source of as many slots as the operator
// assigns. So the count is of SEATS, not of sources, and a payload holding
// one seed can fill every slot.
func composerAssignableSlots(st *composerState) int {
	slots := composerSlotCount(st.list)
	single := 0
	for _, s := range st.sources {
		if s.kind == composerSourceSeed {
			return slots
		}
		single++
	}
	if single > slots {
		return slots
	}
	return single
}

// composerUnfilledSlots names the slots §8p has to list.
func composerUnfilledSlots(st *composerState) []uint8 {
	var out []uint8
	for i, a := range st.assigned {
		if a.src < 0 {
			out = append(out, uint8(i))
		}
	}
	return out
}

// composerSeatFlow asks for every slot in emitted order.
//
// Returns false when the operator backs out of slot @0, which is the
// directive's rule wherever Back is the way out of an opening screen.
func composerSeatFlow(ctx *Context, th *Colors, st *composerState) bool {
	n := composerSlotCount(st.list)
	if len(st.assigned) != n {
		st.assigned = make([]composerAssignment, n)
		for i := range st.assigned {
			st.assigned[i].src = -1
		}
	}
	for i := 0; i < n && !ctx.Done; {
		slot := uint8(i)
		var rows []string
		var srcIdx []int
		for j, src := range st.sources {
			if src.used {
				continue
			}
			rows = append(rows, composerSourceRow(src))
			srcIdx = append(srcIdx, j)
		}
		rows = append(rows, "Type a seed", "Leave unseated")
		sel, ok := composerPickScreen(ctx, th, "Seat keys", composerSeatPrompt(st, slot), rows)
		if !ok {
			if i == 0 {
				return false
			}
			// Step BACK one slot, releasing what it held.
			i--
			if prev := st.assigned[i].src; prev >= 0 && st.sources[prev].kind != composerSourceSeed {
				st.sources[prev].used = false
			}
			st.assigned[i] = composerAssignment{src: -1}
			continue
		}
		switch {
		case sel < len(srcIdx):
			j := srcIdx[sel]
			if st.sources[j].kind == composerSourceSeed {
				a, err := composerSeedDerive(st, slot, j)
				if err != nil {
					showError(ctx, th, "Seat keys", "Couldn't derive a key from that seed.")
					continue
				}
				st.assigned[i] = a
			} else {
				src := st.sources[j]
				st.sources[j].used = true
				st.assigned[i] = composerAssignment{
					src: j, origin: src.origin, fingerprint: src.fingerprint,
					fpPresent: src.fpPresent, xpub: src.xpub,
				}
			}
			i++
		case sel == len(srcIdx):
			src, ok := composerSeedSource(ctx, th, st)
			if !ok {
				continue
			}
			st.sources = append(st.sources, src)
		default:
			// Left unseated on purpose: the shortfall screen below is what
			// decides whether that is allowed, so this is not a refusal here.
			st.assigned[i] = composerAssignment{src: -1}
			i++
		}
	}
	return true
}

// composerSeatingComplete reports whether every slot holds a key.
func composerSeatingComplete(st *composerState) bool {
	return len(composerUnfilledSlots(st)) == 0
}

// composerShortfall is §7d's all-or-nothing transition: fewer assignable
// slots than slots REFUSES, naming the counts and the unfilled slots.
//
// NO CAUSE IS GUESSED. §8p states two facts and stops; the C5 lesson is
// taught at the shape step by §8k, and a second explanation on the screen
// that refuses is one more thing that can be wrong.
//
// Returns true when the operator chooses to engrave a key-less template
// anyway (§7f's partially seated form).
func composerShortfall(ctx *Context, th *Colors, st *composerState) bool {
	unfilled := composerUnfilledSlots(st)
	showError(ctx, th, "Seat keys", composerCopyShortfall(
		composerSlotCount(st.list), composerAssignableSlots(st), unfilled))
	cs := &ChoiceScreen{
		Title:   "Seat keys",
		Lead:    "What now?",
		Choices: []string{"Back to the paths", "Engrave a key-less template"},
	}
	sel, ok := cs.Choose(ctx, th)
	return ok && sel == 1
}

var _ = md.ComposeMaxSlots
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerAssignableSlots|^TestComposerSeatingComplete' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`2`** (`TestComposerAssignableSlotsCountsSeatsNotSources`, `TestComposerSeatingCompleteTracksEverySlot`). The first logs the §8p body's headroom -- measured: `38 chars drawn in full, headroom 494 chars (margin 80)`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_seat.go gui/composer_seat_test.go
git commit -s -F - <<'MSG'
gui: slot-directed seating, all-or-nothing, with section 8p's counts (composer S3 task B5)

The count is of assignable SEATS, not of sources: a seed fills as many slots
as the operator assigns it, so a payload holding one can fill every slot.
Section 8p names the counts and the unfilled slots and guesses no cause --
the C5 lesson is taught at the shape step by section 8k, and a second
explanation on the screen that refuses is one more thing that can be wrong.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task B6: `gui/composer_selfcheck.go` -- §7e's self-check on the DECODED md1, §8q, and the §8l warning

**Files:**
- Create: `gui/composer_selfcheck.go`
- Test: `gui/composer_selfcheck_test.go`

**Interfaces:**
- Consumes: `md.PolicyShapeChunks`, `md.ExpandWalletPolicyChunks`, `md.ExpandedKey{Index, OriginPath, UseSite, Fingerprint, FingerprintPresent, Xpub, XpubPresent}`; `composerConsentLines` (Task A11); `composerOriginKey`, `composerInvariantViolation` (Task B4); `confirmReviewScreen` (`gui/multisig_build.go:1877`).
- Produces: `var composerSelfCheckFaultHook func(chunks []string) []string`; `func composerSelfCheck(st *composerState, chunks []string) error`; `func composerConsentFlow(ctx *Context, th *Colors, st *composerState, chunks []string) bool`.

**What it checks, on the DECODED md1** (§7e): the decoded shape against the composed path list; the slot assignment; every slot's origin and fingerprint against the mapping review; the fixed `/<0;1>/*` use-site; and §4f's pairwise-distinguishability invariant. **What stays outside it:** the key bytes themselves, which the addresses cover.

**It is provoked by FAULT INJECTION, not by an input** (§12 item 4), because no operator input can make the builder disagree with itself -- which is exactly why a test that only feeds inputs would report this gate as passing while never running it. `composerSelfCheckFaultHook` is nil in production and the test swaps a chunk under it.

**The surface is `confirmReviewScreen`'s paged form** (`gui/multisig_build.go:1877-1939`, pager gate at `:1926-1931`): eight paths plus four addresses do not fit one frame. Then §8l, unskippable.

- [ ] **Step 1: the failing test**

Create `gui/composer_selfcheck_test.go`:

```go
package gui

import (
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

// composerSeatedFixture is a fully seated 2-of-3 wsh policy and the chunks it
// composes to: one composerState whose assignments agree with the artifact,
// which is the state the self-check must ACCEPT before any of its refusals
// mean anything.
func composerSeatedFixture(t *testing.T) (*composerState, []string) {
	t.Helper()
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	st := &composerState{list: list, reg: &seedRegistry{}}
	st.sources = []composerSource{
		{kind: composerSourceKey, seedID: -1}, {kind: composerSourceKey, seedID: -1},
		{kind: composerSourceKey, seedID: -1},
	}
	declared := make([]*md.SlotOrigin, 3)
	st.assigned = make([]composerAssignment, 3)
	for i := range st.assigned {
		fp := [4]byte{0x73, 0xc5, 0xda, byte(i)}
		origin := composerTestOrigin(2, uint32(i))
		st.assigned[i] = composerAssignment{
			src: i, account: uint32(i), origin: origin,
			fingerprint: fp, fpPresent: true,
		}
		declared[i] = &md.SlotOrigin{Origin: origin, Fingerprint: fp, FpPresent: true}
	}
	c, err := md.ComposeWith(list, declared)
	if err != nil {
		t.Fatalf("md.ComposeWith: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	return st, chunks
}

// composerOtherWalletChunks is a DIFFERENT wallet's artifact, for the
// injection that swaps the whole chunk set.
func composerOtherWalletChunks(t *testing.T) []string {
	t.Helper()
	c, err := md.Compose(md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 2, Sorted: true}},
	}})
	if err != nil {
		t.Fatalf("md.Compose: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	return chunks
}

// TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput is §12 item 4's
// last clause, and the one gate here that no input can reach.
//
// A GATE THAT HAS NEVER EXECUTED IS A HYPOTHESIS. The check exists so a
// builder defect in the shape, the seating, the origins, the fingerprints or
// the use-site cannot reach steel as a REVIEWED wallet, and the only way to
// run it is to break the builder's output on purpose.
func TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput(t *testing.T) {
	st, chunks := composerSeatedFixture(t) // a 2-of-3 wsh, every slot seated
	if err := composerSelfCheck(st, chunks); err != nil {
		t.Fatalf("INCONCLUSIVE: the self-check refuses an HONEST build: %v -- every "+
			"assertion below would then pass for the wrong reason", err)
	}
	for _, tc := range []struct {
		name    string
		breakIt func(*composerState, []string) []string
	}{
		{"a slot's origin moves", func(st *composerState, c []string) []string {
			st.assigned[0].origin = composerTestOrigin(2, 31)
			return c
		}},
		{"a slot's fingerprint moves", func(st *composerState, c []string) []string {
			st.assigned[0].fingerprint = [4]byte{0xff, 0xff, 0xff, 0xff}
			return c
		}},
		{"the shape gains a path the chunks do not have", func(st *composerState, c []string) []string {
			st.list.Paths = append(st.list.Paths, md.SpendPath{Keys: &md.KeySet{K: 1, N: 1}})
			return c
		}},
		{"the chunks are another wallet's", func(st *composerState, c []string) []string {
			return composerOtherWalletChunks(t)
		}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st, chunks := composerSeatedFixture(t)
			got := tc.breakIt(st, chunks)
			if err := composerSelfCheck(st, got); err == nil {
				t.Errorf("the self-check ACCEPTED a build where %s; §8q's refusal would "+
					"never fire and a wrong wallet would reach steel as reviewed", tc.name)
			}
		})
	}
	assertModalBodyFits(t, "the §8q self-check refusal", errorScreenBody, composerCopySelfCheckFailed())
	assertModalBodyFits(t, "the §8l unchecked-policy warning", confirmWarningBody,
		composerConfirmBody(composerCopyNothingChecked()))
}

// TestComposerConsentRefusesThroughTheHookAndSaysSection8q drives the SCREEN,
// so the refusal is proven to reach the operator and not merely to be
// returned by a function.
func TestComposerConsentRefusesThroughTheHookAndSaysSection8q(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		st, chunks := composerSeatedFixture(t)
		composerSelfCheckFaultHook = func(c []string) []string { return composerOtherWalletChunks(t) }
		defer func() { composerSelfCheckFaultHook = nil }()
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerConsentFlow(ctx, &descriptorTheme, st, chunks)
		})
		defer quit()
		content, ok := frame()
		if !ok {
			t.Fatal("the consent flow drew nothing")
		}
		assertFrameHasBody(t, ink(), "the §8q self-check refusal")
		if !uiContains(content, "does not match what you built") {
			t.Errorf("the refusal does not say §8q's words.\nFrame: %q", content)
		}
		if !uiContains(content, "start again") {
			t.Errorf("the refusal does not give the operator an exit.\nFrame: %q", content)
		}
	})
}

// TestComposerSelfCheckFaultHookIsNilInProduction: the seam must not be able
// to weaken the gate on a shipped device.
func TestComposerSelfCheckFaultHookIsNilInProduction(t *testing.T) {
	if composerSelfCheckFaultHook != nil {
		t.Error("composerSelfCheckFaultHook is non-nil at rest")
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerSelfCheck|^TestComposerConsentRefuses' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `undefined: composerSelfCheck`, `undefined: composerConsentFlow`, `undefined: composerSelfCheckFaultHook`.

- [ ] **Step 3: Write the check**

Create `gui/composer_selfcheck.go`:

```go
package gui

import (
	"errors"
	"fmt"

	"seedhammer.com/md"
)

// §7e's self-check, on the DECODED md1.
//
// WHAT IT PROVES. Before the consent screen is shown, the device asserts that
// the decoded shape, the slot assignment, every slot's origin and
// fingerprint, the fixed use-site and §4f's pairwise-distinguishability
// invariant ALL hold on the chunk set it is about to engrave. So a builder
// defect in the shape, the seating, the origins, the fingerprints or the
// use-site cannot reach steel as a REVIEWED wallet.
//
// WHAT STAYS OUTSIDE IT: the key bytes themselves, which the addresses on the
// consent screen cover.
//
// IT IS PROVOKED BY FAULT INJECTION, NOT BY AN INPUT (§12 item 4). No
// operator input can make the builder disagree with itself, which is exactly
// why a test that only fed inputs would report this gate as passing while
// never running it. A gate that has never executed is a hypothesis.
var composerSelfCheckFaultHook func(chunks []string) []string

// composerUseSiteIsFixed reports whether a slot carries §5's fixed use-site,
// `/<0;1>/*`. The composer emits nothing else, so anything else is a builder
// defect rather than an exotic wallet.
func composerUseSiteIsFixed(u md.UseSite) bool {
	if !u.HasMultipath || u.WildcardHardened || len(u.Multipath) != 2 {
		return false
	}
	return !u.Multipath[0].Hardened && u.Multipath[0].Value == 0 &&
		!u.Multipath[1].Hardened && u.Multipath[1].Value == 1
}

// composerLeafPaths is the operator's path list with an extracted taproot
// internal-key path removed, in listed order -- which is the order §5 puts
// the leaves on the spine, and therefore the order PolicyShape reports them.
func composerLeafPaths(list md.PathList) []md.SpendPath {
	internal := -1
	if list.Wrapper == md.ComposeTr {
		for i, p := range list.Paths {
			if p.Keys != nil && p.Keys.N == 1 && p.Lock == nil && p.Hash == nil {
				internal = i
				break
			}
		}
	}
	out := make([]md.SpendPath, 0, len(list.Paths))
	for i, p := range list.Paths {
		if i == internal {
			continue
		}
		out = append(out, p)
	}
	return out
}

// composerSelfCheck reports the FIRST mismatch by name. First, not all:
// §8q's body is fixed copy, and the name is for the implementation report and
// the test, where a list of consequences of one root cause is noise.
func composerSelfCheck(st *composerState, chunks []string) error {
	shape, err := md.PolicyShapeChunks(chunks)
	if err != nil {
		return err
	}
	if !shape.Complete {
		return errors.New("self-check: the decoded policy cannot be described")
	}
	leaves := composerLeafPaths(st.list)
	if len(shape.Branches) != len(leaves) {
		return fmt.Errorf("self-check: the decoded policy has %d spend paths, the shape has %d",
			len(shape.Branches), len(leaves))
	}
	for i, p := range leaves {
		b := shape.Branches[i]
		switch {
		case p.Keys == nil:
			if b.Keys != 0 {
				return fmt.Errorf("self-check: path %d is key-less in the shape and has %d keys decoded", i+1, b.Keys)
			}
		case p.Keys.N >= 2:
			if b.K != int(p.Keys.K) || b.N != int(p.Keys.N) {
				return fmt.Errorf("self-check: path %d is %d-of-%d in the shape and %d-of-%d decoded",
					i+1, p.Keys.K, p.Keys.N, b.K, b.N)
			}
		default:
			if b.Keys != 1 {
				return fmt.Errorf("self-check: path %d is one key in the shape and %d decoded", i+1, b.Keys)
			}
		}
		wantLocks := 0
		if p.Lock != nil {
			wantLocks = 1
		}
		if len(b.Locks) != wantLocks {
			return fmt.Errorf("self-check: path %d has %d locks in the shape and %d decoded",
				i+1, wantLocks, len(b.Locks))
		}
		if p.Lock != nil && (b.Locks[0].Kind != p.Lock.Kind || b.Locks[0].Value != p.Lock.Value) {
			return fmt.Errorf("self-check: path %d's lock is %v/%d in the shape and %v/%d decoded",
				i+1, p.Lock.Kind, p.Lock.Value, b.Locks[0].Kind, b.Locks[0].Value)
		}
		wantHash := 0
		if p.Hash != nil {
			wantHash = 1
		}
		if len(b.Sha256Digests) != wantHash {
			return fmt.Errorf("self-check: path %d has %d hash locks in the shape and %d decoded",
				i+1, wantHash, len(b.Sha256Digests))
		}
		if p.Hash != nil && b.Sha256Digests[0] != *p.Hash {
			return fmt.Errorf("self-check: path %d's digest differs from the shape's", i+1)
		}
	}

	_, keys, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		return err
	}
	if len(keys) != len(st.assigned) {
		return fmt.Errorf("self-check: the decoded policy has %d slots, the seating has %d",
			len(keys), len(st.assigned))
	}
	for i, k := range keys {
		a := st.assigned[i]
		if !composerUseSiteIsFixed(k.UseSite) {
			return fmt.Errorf("self-check: slot @%d does not carry the fixed <0;1>/* use-site", i)
		}
		if a.src < 0 {
			// An unseated slot declares §4f's default origin and no
			// fingerprint. Both are checked, because a fingerprint that
			// appeared on an unseated slot would be a key nobody chose.
			if k.FingerprintPresent {
				return fmt.Errorf("self-check: unseated slot @%d declares a fingerprint", i)
			}
			continue
		}
		if composerOriginKey(originComponents(k.OriginPath)) != composerOriginKey(a.origin) {
			return fmt.Errorf("self-check: slot @%d declares %s, the mapping review showed %s",
				i, k.OriginPath, composerOriginText(a.origin))
		}
		if k.FingerprintPresent != a.fpPresent {
			return fmt.Errorf("self-check: slot @%d's fingerprint presence differs from the mapping review", i)
		}
		if a.fpPresent && k.Fingerprint != a.fingerprint {
			return fmt.Errorf("self-check: slot @%d declares %x, the mapping review showed %x",
				i, k.Fingerprint, a.fingerprint)
		}
	}
	// §4f's INVARIANT, ON THE DECODED md1 -- which is where §4f puts it ("no
	// two slots of a PRODUCED template declare the same origin unless BOTH
	// declare a fingerprint and those fingerprints differ") and where §7e says
	// it is re-checked. composerInvariantViolation reads composer UI state,
	// in which an unseated slot has no origin at all; the origins that matter
	// are the lowest-free accounts the codec assigned when it emitted this
	// chunk set.
	type seat struct {
		fp        [4]byte
		fpPresent bool
	}
	byOrigin := map[string][]seat{}
	for _, k := range keys {
		o := composerOriginKey(originComponents(k.OriginPath))
		byOrigin[o] = append(byOrigin[o], seat{k.Fingerprint, k.FingerprintPresent})
	}
	for o, seats := range byOrigin {
		for i := 0; i < len(seats); i++ {
			for j := i + 1; j < len(seats); j++ {
				if !seats[i].fpPresent || !seats[j].fpPresent || seats[i].fp == seats[j].fp {
					return fmt.Errorf("self-check: two slots declare %s without two "+
						"distinct fingerprints", o)
				}
			}
		}
	}
	return nil
}

// composerConsentFlow is §7e end to end: the check, then the paged surface,
// then §8l.
//
// THE SURFACE IS confirmReviewScreen's PAGED FORM (gui/multisig_build.go
// :1877-1939, its pager gated on a second page existing at :1926-1931):
// eight paths plus four addresses do not fit one frame.
func composerConsentFlow(ctx *Context, th *Colors, st *composerState, chunks []string) bool {
	checked := chunks
	if composerSelfCheckFaultHook != nil {
		checked = composerSelfCheckFaultHook(chunks)
	}
	if err := composerSelfCheck(st, checked); err != nil {
		// The BODY is §8q's fixed copy; the error's name reaches the
		// implementation report and the tests, not the operator, who cannot
		// act on "path 2's digest differs".
		showError(ctx, th, "Review", composerCopySelfCheckFailed())
		return false
	}
	listed, keyPathNo := composerListedPaths(st.list)
	lines, err := composerConsentLinesFor(checked, listed, keyPathNo)
	if err != nil {
		showError(ctx, th, "Review", composerCopySelfCheckFailed())
		return false
	}
	// ONE CONSENT SURFACE, and it is composerReadScreen.
	//
	// §7e and §9 item 9 name confirmReviewScreen's PAGED FORM, and
	// composerReadScreen is that form copied (gui/multisig_build.go:1877-1939)
	// plus the one thing §7e needs and it lacks: the checkmark is withheld
	// until the last page has been laid out once, so the addresses -- the only
	// proof of WHICH wallet this is -- cannot be consented to unseen. Part A
	// used composerReadScreen and Part B used confirmReviewScreen, which was
	// one decision described twice with different Back semantics.
	if !composerReadScreen(ctx, th, "Review", lines) {
		return false
	}
	return composerConfirmScreen(ctx, th, "Before you fund it",
		composerConfirmBody(composerCopyNothingChecked()))
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerSelfCheck|^TestComposerConsentRefuses' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`3`** (`TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput` with its four fault-injection sub-tests, `TestComposerConsentRefusesThroughTheHookAndSaysSection8q`, `TestComposerSelfCheckFaultHookIsNilInProduction`); the §8q and §8l bodies log their headroom -- measured: 476 and 339 characters against a margin of 80.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_selfcheck.go gui/composer_selfcheck_test.go
git commit -s -F - <<'MSG'
gui: section 7e's self-check on the DECODED md1, provoked by fault injection (composer S3 task B6)

No operator input can make the builder disagree with itself, which is exactly
why a test that only fed inputs would report this gate as passing while never
running it. The consent surface is composerReadScreen -- confirmReviewScreen's
paged form plus the last-page gate section 7e needs, so the addresses cannot
be consented to unseen.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task B7: `gui/composer_engrave.go` -- the §7f form choice, Full/Watch-only, and the secret's plate form

**Files:**
- Create: `gui/composer_engrave.go`
- Test: `gui/composer_engrave_test.go`

**Interfaces:**
- Consumes: `ChoiceScreen`; `buildFullModeLabel(usedPassphrase bool)` and the shipped mode choice's labels (`gui/multisig_build.go:455`); `seedRegistry.usesPassphrase()` (`gui/multisig_build_slots.go:238`); `engraveSeed(params, m, mfp) (Plate, error)` (`gui/gui.go:839`), `engraveCodex32(ctx, th, scan) bool` (`gui/codex32_polish.go:218`), `backup.Seed`/`backup.SeedString` (`backup/backup.go:16,26`).
- Produces: `type composerForm int` with `composerFormConcrete`, `composerFormTemplateAndCards`, `composerFormTemplateOnly`; `func composerFormsFor(st *composerState) []composerForm`; `func composerFormLabel(f composerForm) string`; `func composerFormPick(ctx, th, st) (composerForm, bool)`; `func composerEngraveModePick(ctx, th, st) (bool, bool)`. **NO secret-form picker**: it returned a value nothing consumed, and the builder for the form it offered lives in Task B11's `composerSecretCards`.

**RECON RISK 1, RESOLVED HERE.** §7f says the secret is cut "as words, as a SeedQR, or as ms1 strings". **Those are not three code paths.** `engraveSeed` (`gui/gui.go:839-861`) builds ONE `backup.Seed` carrying the words AND a SeedQR and cuts them on one plate; `backup.SeedString` (`backup/backup.go:26-31`) is a string-only plate, and `engraveCodex32` (`gui/codex32_polish.go:218-237`) is what cuts an ms1 through it. There is no words-only and no QR-only plate for a mnemonic anywhere in this tree.

**What §7f gets, after the fidelity lens (I-10):** the secret is cut as **ms1**, through `cardMS1` -- the only secret form the plate planner carries as a bundle card (`gui/multisig_engrave.go:36`), and what Multisig Build's own Full mode cuts. The picker this task first carried offered "Words and SeedQR" beside "ms1 string" and **nothing consumed the answer**: neither `engraveSeed` nor `engraveCodex32` was ever called, so §7f's secret plate had no builder at all. A picker whose answer nothing reads is the same defect class as an unreachable screen, so the picker is gone and the one form that HAS a builder is cut. The words-plus-SeedQR plate is a `backup.Seed`, not a bundle card, and needs its own plate pass. It is filed:

```text
### F-455 — `composer-secret-form-words-and-seedqr-are-one-plate`: SPEC §7f offers three secret forms; the device has two plate designs (owning phase: **a later cycle, spec fold at composer S4**) `#seedhammer` `#composer` `#backup`

`engraveSeed` (gui/gui.go:839) bakes BIP-39 words AND a SeedQR onto ONE
`backup.Seed` plate; `backup.SeedString` (backup/backup.go:26) is the
string-only form `engraveCodex32` cuts for ms1. A words-only or a QR-only
plate for a mnemonic does not exist. S3 offers the two real forms with honest
labels; splitting them is a new plate layout with its own sizing and its own
goldens, not a wiring change. Fold §7f's wording at S4 or rule the split in.
```

**Which forms are offered** (§7f): a keyless composition has no form A and no cards, so the choice collapses to template-only and says so; a PARTIALLY seated composition offers no form A either, and its form B is the keyless template plus one card per SEATED slot carrying the TEMPLATE stub only, with the screen saying the policy id does not exist until every slot is seated.

- [ ] **Step 1: Write the form choice**

A table test over the three seating states asserting which forms are offered, and that every label passes `assertChoiceLabelFits`. The secret-form picker is NOT written: see the §7f note above and Task B11's `composerSecretCards`, which is the builder for the one form this device cuts.

Create `gui/composer_engrave.go`:

```go
package gui

// The engrave form choice (SPEC §7f, C10, C13).
//
// THE "THREE SECRET FORMS" ARE TWO CODE PATHS. engraveSeed (gui/gui.go
// :839-861) builds ONE backup.Seed carrying the words AND a SeedQR and cuts
// them on one plate; backup.SeedString (backup/backup.go:26-31) is the
// string-only plate engraveCodex32 cuts for ms1. A words-only or a QR-only
// plate for a mnemonic does not exist anywhere in this tree. So the picker
// offers the two the device HAS, labelled for what is actually on the plate,
// and F-455 owns the split -- which is a new backup layout with its own
// sizing and its own goldens, not a ChoiceScreen over two existing functions.

type composerForm int

const (
	// composerFormConcrete is §7f's form A: the policy itself, as text or QR
	// plates or keyed md1 strings. Offered only when EVERY slot is seated.
	composerFormConcrete composerForm = iota
	// composerFormTemplateAndCards is form B: keyless md1 WITH fingerprints,
	// plus one mk1 card per seated slot.
	composerFormTemplateAndCards
	// composerFormTemplateOnly is the collapsed case: a key-less composition
	// has no form A and no cards.
	composerFormTemplateOnly
)

// THE SECRET FORM PICKER IS GONE, and its absence is the fix.
//
// It offered "Words and SeedQR" and "ms1 string" and NOTHING CONSUMED THE
// ANSWER: neither engraveSeed nor engraveCodex32 was ever called. §7f names
// three secret forms, this device has two plate designs (F-455), and only one
// of them -- ms1 -- is a bundle card the plate planner carries (cardMS1,
// gui/multisig_engrave.go:36). A backup.Seed words-plus-SeedQR plate needs its
// own plate pass, so composerSecretCards cuts ms1 and the words plate is filed
// with F-455. A picker whose answer nothing reads is worse than one choice
// honestly stated.

// composerFormsFor is §7f's offer, per seating state.
func composerFormsFor(st *composerState) []composerForm {
	seated := 0
	for _, a := range st.assigned {
		if a.src >= 0 {
			seated++
		}
	}
	switch {
	case seated == 0:
		// A key-less composition has no form A and no cards: the choice
		// collapses to template only, and the screen says so.
		return []composerForm{composerFormTemplateOnly}
	case seated < len(st.assigned):
		// PARTIALLY seated (§8p's fallback): no form A either. Its form B is
		// the key-less template, whose unseated slots take §4f's lowest-free
		// accounts, plus one card per SEATED slot carrying the TEMPLATE stub
		// only -- the policy id does not exist until every slot is seated.
		return []composerForm{composerFormTemplateAndCards}
	default:
		return []composerForm{composerFormConcrete, composerFormTemplateAndCards}
	}
}

func composerFormLabel(f composerForm) string {
	switch f {
	case composerFormConcrete:
		return "The policy itself"
	case composerFormTemplateAndCards:
		return "Template plus key cards"
	}
	return "Template only (no keys)"
}

// composerFormPick offers what §7f allows for this seating state, and says
// plainly when there is nothing to choose between.
func composerFormPick(ctx *Context, th *Colors, st *composerState) (composerForm, bool) {
	forms := composerFormsFor(st)
	if len(forms) == 1 {
		lead := "No slot is seated, so there is a template and nothing else."
		if forms[0] == composerFormTemplateAndCards {
			lead = "Some slots are unseated, so this policy has no id yet. " +
				"The template and the cards for the seated slots are what this cuts."
		}
		showError(ctx, th, "What to engrave", lead)
		return forms[0], true
	}
	choices := make([]string, len(forms))
	for i, f := range forms {
		choices[i] = composerFormLabel(f)
	}
	cs := &ChoiceScreen{Title: "What to engrave", Lead: "Which form?", Choices: choices}
	sel, ok := cs.Choose(ctx, th)
	if !ok {
		return composerFormTemplateOnly, false
	}
	return forms[sel], true
}

// composerEngraveModePick is §7f's Full versus Watch-only, reusing Multisig
// Build's own labels so the two programs cannot describe one decision in two
// ways. buildFullModeLabel names what "Full" LEAVES OUT when a passphrase was
// used: a passphrase is a required spending factor and is never engraved.
func composerEngraveModePick(ctx *Context, th *Colors, st *composerState) (bool, bool) {
	cs := &ChoiceScreen{
		Title:   "Engrave Mode",
		Lead:    "What to engrave?",
		Choices: []string{buildFullModeLabel(st.reg.usesPassphrase()), "Watch-only (keys)"},
	}
	sel, ok := cs.Choose(ctx, th)
	return sel == 0, ok
}
```

- [ ] **Step 2: Run to verify the tests fail**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerForm|^TestComposerSecretIsCutOnce|^TestComposerEngraveMode' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `# seedhammer.com/gui [seedhammer.com/gui.test]` then `undefined: composerFormsFor`, `undefined: composerFormLabel`, `undefined: composerEngraveModePick`.

- [ ] **Step 3: Write the test file this task promised and never wrote**

The tests lens counted the missing file as a Critical (its C-2): Task B7 named `gui/composer_engrave_test.go` in its own Files header and no ```go block existed, so the engrave-form choice shipped with zero tests of any kind -- below the plan's own stated bar.

Create `gui/composer_engrave_test.go`:

```go
package gui

import (
	"strings"
	"testing"
)

// composerStateSeated builds a state with `slots` slots of which the first
// `seated` hold a source, which is the only input composerFormsFor reads.
func composerStateSeated(slots, seated int) *composerState {
	st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
	st.assigned = make([]composerAssignment, slots)
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
	for i := 0; i < seated; i++ {
		st.assigned[i].src = i
	}
	return st
}

// TestComposerFormsForOfferWhatSection7fAllows is §7f's three seating states,
// and each arm is a refusal to offer something the artifact cannot carry: a
// key-less composition HAS no concrete policy, and a partially seated one has
// no policy id until every slot is seated.
func TestComposerFormsForOfferWhatSection7fAllows(t *testing.T) {
	for _, tc := range []struct {
		name   string
		slots  int
		seated int
		want   []composerForm
	}{
		{"nothing seated collapses to template only", 4, 0,
			[]composerForm{composerFormTemplateOnly}},
		{"partially seated offers no form A", 4, 2,
			[]composerForm{composerFormTemplateAndCards}},
		{"fully seated offers both", 4, 4,
			[]composerForm{composerFormConcrete, composerFormTemplateAndCards}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			got := composerFormsFor(composerStateSeated(tc.slots, tc.seated))
			if len(got) != len(tc.want) {
				t.Fatalf("offered %d form(s), want %d: %v", len(got), len(tc.want), got)
			}
			for i := range got {
				if got[i] != tc.want[i] {
					t.Errorf("form %d is %v, want %v", i, got[i], tc.want[i])
				}
			}
		})
	}
}

// TestComposerFormLabelsFitTheChoicePanel: ChoiceScreen draws its rows with
// widget.Label, which does not wrap, so a long label is drawn off the edge and
// the operator picks a truncated option.
func TestComposerFormLabelsFitTheChoicePanel(t *testing.T) {
	for _, f := range []composerForm{
		composerFormConcrete, composerFormTemplateAndCards, composerFormTemplateOnly,
	} {
		label := composerFormLabel(f)
		if label == "" {
			t.Errorf("form %v has no label", f)
		}
		assertChoiceLabelFits(t, label)
	}
}

// TestComposerSecretIsCutOnceForASeedThatFilledSeveralSlots is §7f verbatim
// ("a seed that filled several slots is cut ONCE") and the reason the picker
// this task once carried is gone: nothing consumed its answer, so §7f's secret
// plate had no builder at all. composerSecretCards is that builder, and ms1 is
// the form the plate planner carries as a bundle card (cardMS1); the
// words-plus-SeedQR plate is a backup.Seed and is filed with F-455.
func TestComposerSecretIsCutOnceForASeedThatFilledSeveralSlots(t *testing.T) {
	st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
	id, err := st.reg.add("seed 1", composerTestMnemonic(t), "", composerMainNet())
	if err != nil {
		t.Fatal(err)
	}
	st.sources = []composerSource{{kind: composerSourceSeed, seedID: id, fpPresent: true}}
	st.assigned = make([]composerAssignment, 3)
	for i := range st.assigned {
		st.assigned[i] = composerAssignment{src: 0}
	}
	cards, err := composerSecretCards(st)
	if err != nil {
		t.Fatalf("composerSecretCards: %v", err)
	}
	if len(cards) != 1 {
		t.Fatalf("one seed at three slots produced %d secret plate(s), want 1 -- §7f "+
			"says a seed that filled several slots is cut ONCE, and three bearer "+
			"plates for one secret is three times the exposure for no recovery value",
			len(cards))
	}
	if cards[0].kind != cardMS1 {
		t.Errorf("the secret card is kind %v, want cardMS1 (the kind bundlePlateMark "+
			"never marks, gui/bundle_flow.go:574)", cards[0].kind)
	}
	if len(cards[0].strings) != 1 || len(cards[0].strings[0]) == 0 {
		t.Errorf("the secret card carries no ms1 string: %+v", cards[0].strings)
	}
	// A WATCH-ONLY build cuts none, which is the whole of the mode choice.
	none, err := composerSecretCards(&composerState{reg: &seedRegistry{}})
	if err != nil {
		t.Fatal(err)
	}
	if len(none) != 0 {
		t.Errorf("a composition with no seed produced %d secret plate(s)", len(none))
	}
}

// TestComposerEngraveModeReusesMultisigBuildsLabels: "Full" must name what it
// LEAVES OUT when a passphrase was used, because a BIP-39 passphrase is a
// required spending factor and is never engraved -- a set labelled
// "Full (seed + keys)" that omits one is F-132's shape.
func TestComposerEngraveModeReusesMultisigBuildsLabels(t *testing.T) {
	bare := buildFullModeLabel(false)
	withPass := buildFullModeLabel(true)
	if bare == withPass {
		t.Fatalf("buildFullModeLabel renders the same label with and without a "+
			"passphrase (%q); the composer inherits the honesty of that label", bare)
	}
	if !strings.Contains(strings.ToLower(withPass), "passphrase") {
		t.Errorf("the passphrased Full label does not name the missing factor: %q", withPass)
	}
	assertChoiceLabelFits(t, bare)
	assertChoiceLabelFits(t, withPass)
	assertChoiceLabelFits(t, "Watch-only (keys)")
}
```

`composerSecretCards`, which the cut-once test drives, is written in Task B11 with the flow that calls it; run this test after that task lands, or run it now with the three form tests alone.

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerForm|^TestComposerSecretIsCutOnce|^TestComposerEngraveMode' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: four top-level PASS with three sub-tests under the forms test (`nothing seated collapses to template only`, `partially seated offers no form A`, `fully seated offers both`); `ok seedhammer.com/gui`. The label test logs each label's width -- measured: `The policy itself` 159 px, `Template plus key cards` 253 px, `Template only (no keys)` 251 px, against a 436 px row.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_engrave.go gui/composer_engrave_test.go
git commit -s -F - <<'MSG'
gui: the section 7f form choice, and one secret form that actually has a builder (composer S3 task B7)

The form choice collapses correctly for a key-less and a partially seated
composition. The secret-form picker is gone: it offered two plate designs and
nothing consumed the answer, so section 7f's secret plate had no builder --
ms1 is the form the plate planner carries, and the words-plus-SeedQR plate is
filed with F-455.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task B8: card minting and re-minting with both stubs

**Files:**
- Create: `gui/composer_cards.go`
- Test: `gui/composer_cards_test.go`

**Interfaces:**
- Consumes: `md.ComposerStubs(templateChunks, keyedChunks []string) ([][4]byte, error)` and `mk.AppendStubs(card Card, stubs ...[4]byte) Card` (both S2); `mk.Encode(card) ([]string, error)` (`mk/encode.go:39`), `mk.Decode` (`mk/mk.go:148`); `deriveAccountXpub`; `md.FormAwareStubChunks`.
- Produces: `func composerMintCard(st *composerState, slot uint8, templateChunks, keyedChunks []string) ([]string, error)`.

Every seated slot yields a card in form B **regardless of source** (§7f): a `key:` record is MINTED as an mk1 (fingerprint, origin, xpub and both stubs), a payload mk1 is RE-MINTED with both stubs APPENDED to the ones it already carries (§7d, so one card seats into either engraved form and stays indexed to the wallets it already belonged to), and a seed-derived slot is minted likewise. `mk.Encode` is deterministic (`mk/encode.go:39`), so re-minting is exact.

- [ ] **Step 1: Write the failing test**

The tests lens counted this file's absence as a Critical (its C-2): Task B8's own prose specified three cases and named `gui/composer_cards_test.go` in its Files header, and no ```go block existed -- so card minting, including the `mk.Decode` round trip, shipped with no test at all.

**The card's declared path is compared STRUCTURALLY.** Measured while writing this: `mk`'s codec round-trips the origin and `Decode` renders `m/48h/0h/0h/2h` where `composerOriginText` wrote `m/48'/0'/0'/2'`. The same path in two notations -- which is the lesson `slotMatchesCard` states in its own comment (`gui/key_card_seating.go:130-137`), and a string comparison here fails on a correct card.

Create `gui/composer_cards_test.go`:

```go
package gui

import (
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"seedhammer.com/bip32"
	"seedhammer.com/md"
	"seedhammer.com/mk"
)

// composerCardFixture is a two-slot wsh policy with REAL xpubs seated: the
// smallest composition that can be minted into cards and seated back.
//
// Both slots carry master fingerprint 73c5da0a at DIFFERENT accounts, which is
// C5's normal case (one person in two paths holds two accounts) and is the
// case §4f's invariant permits -- distinct origins, so no fingerprint
// ambiguity arises.
func composerCardFixture(t *testing.T) (st *composerState, template, keyed []string) {
	t.Helper()
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 2, Sorted: true}},
	}}
	xpubs := []string{composerTestXpubA, composerTestXpubB}
	fp := [4]byte{0x73, 0xc5, 0xda, 0x0a}

	declared := make([]*md.SlotOrigin, len(xpubs))
	st = &composerState{list: list, reg: &seedRegistry{}}
	st.assigned = make([]composerAssignment, len(xpubs))
	st.sources = make([]composerSource, len(xpubs))
	pub := map[uint8][65]byte{}
	fps := map[uint8][4]byte{}
	for i, x := range xpubs {
		origin := composerTestOrigin(2, uint32(i))
		declared[i] = &md.SlotOrigin{Origin: origin, Fingerprint: fp, FpPresent: true}
		st.sources[i] = composerSource{kind: composerSourceKey, seedID: -1, xpub: x}
		st.assigned[i] = composerAssignment{
			src: i, account: uint32(i), origin: origin,
			fingerprint: fp, fpPresent: true, xpub: x,
		}
		cc, pk, _, err := decodeXpubBytes(x)
		if err != nil {
			t.Fatalf("decodeXpubBytes(%s): %v", x[:12], err)
		}
		var b [65]byte
		copy(b[0:32], cc[:])
		copy(b[32:65], pk[:])
		pub[uint8(i)] = b
		fps[uint8(i)] = fp
	}

	// COMPOSED TWICE, NOT COPIED. md.Composed's doc is explicit: a copy shares
	// the underlying descriptor, so Bind on one keys them both -- and a
	// "template" that had been keyed by its own policy would make every
	// assertion below tautological.
	ct, err := md.ComposeWith(list, declared)
	if err != nil {
		t.Fatalf("md.ComposeWith (template): %v", err)
	}
	if template, err = ct.Chunks(); err != nil {
		t.Fatal(err)
	}
	ck, err := md.ComposeWith(list, declared)
	if err != nil {
		t.Fatalf("md.ComposeWith (keyed): %v", err)
	}
	if err := ck.Bind(pub, fps); err != nil {
		t.Fatalf("Bind: %v", err)
	}
	if keyed, err = ck.Chunks(); err != nil {
		t.Fatal(err)
	}
	return st, template, keyed
}

// TestComposerMintCardCarriesBothStubsAndRoundTrips is §7d's minting rule: a
// key: record is MINTED as an mk1 carrying the fingerprint, the origin, the
// xpub and BOTH stubs, and mk.Encode is deterministic so a re-mint is exact.
func TestComposerMintCardCarriesBothStubsAndRoundTrips(t *testing.T) {
	st, template, keyed := composerCardFixture(t)
	strs, err := composerMintCard(st, 0, template, keyed)
	if err != nil {
		t.Fatalf("composerMintCard: %v", err)
	}
	card, err := mk.Decode(strs)
	if err != nil {
		t.Fatalf("the minted card does not decode: %v", err)
	}
	if card.Xpub != composerTestXpubA {
		t.Errorf("the card carries xpub %q, want the seated one", card.Xpub)
	}
	if card.Fingerprint != "73c5da0a" {
		t.Errorf("the card carries fingerprint %q, want 73c5da0a", card.Fingerprint)
	}
	// THE PATH IS COMPARED STRUCTURALLY, NOT AS A STRING, and that is the
	// lesson slotMatchesCard states in its own comment
	// (gui/key_card_seating.go:130-137): mk's codec round-trips the origin and
	// Decode renders it `m/48h/0h/0h/2h` where composerOriginText wrote
	// `m/48'/0'/0'/2'`. The same path in two notations. A string comparison
	// here would fail on a correct card, and a string comparison in the
	// seating would refuse every card while looking like corruption.
	gotPath, err := bip32.ParsePath(card.Path)
	if err != nil {
		t.Fatalf("the card's declared path %q does not parse: %v", card.Path, err)
	}
	wantPath := make(bip32.Path, 0, 4)
	for _, c := range composerTestOrigin(2, 0) {
		v := c.Value
		if c.Hardened {
			v += hdkeychain.HardenedKeyStart
		}
		wantPath = append(wantPath, v)
	}
	if len(gotPath) != len(wantPath) {
		t.Fatalf("the card declares %d components, want %d", len(gotPath), len(wantPath))
	}
	for i := range gotPath {
		if gotPath[i] != wantPath[i] {
			t.Errorf("the card declares %v, want %v (component %d)", gotPath, wantPath, i)
		}
	}
	// BOTH STUBS, in template-then-policy order (§7c "stamping BOTH stubs").
	want, err := md.ComposerStubs(template, keyed)
	if err != nil {
		t.Fatal(err)
	}
	if len(want) != 2 {
		t.Fatalf("a template and a KEYED policy give %d stub(s), want 2 -- if these "+
			"two forms share a stub the whole re-mint story collapses", len(want))
	}
	if len(card.Stubs) != len(want) {
		t.Fatalf("the card carries %d stub(s), want %d", len(card.Stubs), len(want))
	}
	for i := range want {
		if card.Stubs[i] != want[i] {
			t.Errorf("stub %d is %x, want %x", i, card.Stubs[i], want[i])
		}
	}
}

// TestComposerMintCardWithNoKeyedPolicyCarriesTheTemplateStubAlone is §7f's
// PARTIALLY seated rule: the policy id does not exist until every slot is
// seated, so a card cut then must not claim one.
func TestComposerMintCardWithNoKeyedPolicyCarriesTheTemplateStubAlone(t *testing.T) {
	st, template, _ := composerCardFixture(t)
	strs, err := composerMintCard(st, 0, template, nil)
	if err != nil {
		t.Fatalf("composerMintCard: %v", err)
	}
	card, err := mk.Decode(strs)
	if err != nil {
		t.Fatal(err)
	}
	if len(card.Stubs) != 1 {
		t.Fatalf("a card cut before the policy exists carries %d stubs, want 1", len(card.Stubs))
	}
	tmplStub, err := md.FormAwareStubChunks(template)
	if err != nil {
		t.Fatal(err)
	}
	if card.Stubs[0] != tmplStub {
		t.Errorf("the single stub is %x, want the TEMPLATE's %x", card.Stubs[0], tmplStub)
	}
}

// TestComposerReMintPreservesExistingStubsInOrder is §7d's whole reason for
// APPENDING rather than replacing: a payload card stays indexed to the wallets
// it already belonged to. reStubMk1 (gui/template_engrave.go:41) REPLACES,
// which is right for its own flow and would be wrong here.
func TestComposerReMintPreservesExistingStubsInOrder(t *testing.T) {
	st, template, keyed := composerCardFixture(t)
	prior := [][4]byte{{1, 2, 3, 4}, {5, 6, 7, 8}}
	st.sources[0] = composerSource{
		kind: composerSourceCard, seedID: -1, xpub: composerTestXpubA,
		card: mk.Card{
			Network: "mainnet", Path: "m/48'/0'/0'/2'",
			Fingerprint: "73c5da0a", Stubs: prior, Xpub: composerTestXpubA,
		},
	}
	strs, err := composerMintCard(st, 0, template, keyed)
	if err != nil {
		t.Fatalf("composerMintCard: %v", err)
	}
	card, err := mk.Decode(strs)
	if err != nil {
		t.Fatal(err)
	}
	if len(card.Stubs) != 4 {
		t.Fatalf("a re-minted card carries %d stubs, want 4 (2 prior + 2 composed)", len(card.Stubs))
	}
	for i, s := range prior {
		if card.Stubs[i] != s {
			t.Errorf("prior stub %d moved: got %x, want %x", i, card.Stubs[i], s)
		}
	}
	// APPENDED ONCE: re-minting a card that already carries a composed stub
	// must not duplicate it, or the stub list grows on every pass and can
	// push the card into another chunk for nothing.
	again := mk.AppendStubs(card, card.Stubs[2], card.Stubs[3])
	if len(again.Stubs) != 4 {
		t.Errorf("re-appending stubs the card already carries grew it to %d", len(again.Stubs))
	}
}

// TestComposerMintCardsSkipsUnseatedSlotsAndNamesTheRest: an unseated slot has
// no card, and a refusal to mint one is what keeps the census honest about how
// many plates a partially seated composition cuts.
func TestComposerMintCardsSkipsUnseatedSlotsAndNamesTheRest(t *testing.T) {
	st, template, keyed := composerCardFixture(t)
	st.assigned[1].src = -1
	cards, err := composerMintCards(st, template, keyed)
	if err != nil {
		t.Fatalf("composerMintCards: %v", err)
	}
	if len(cards) != 1 {
		t.Fatalf("one seated slot of two produced %d card(s), want 1", len(cards))
	}
	if cards[0].kind != cardMK1 {
		t.Errorf("the minted card is kind %v, want cardMK1", cards[0].kind)
	}
	if cards[0].label != "mk1 key @0" {
		t.Errorf("the card is labelled %q, want \"mk1 key @0\"", cards[0].label)
	}
	if _, err := composerMintCard(st, 1, template, keyed); err == nil {
		t.Error("minting an UNSEATED slot succeeded; a card for a slot holding no key " +
			"is a plate that vouches for a wallet nobody composed")
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerMintCard|^TestComposerReMint' ./gui/ 2>&1 | head -8`
Expected: FAIL to build -- `# seedhammer.com/gui [seedhammer.com/gui.test]` then `undefined: composerMintCard` at five sites in `gui/composer_cards_test.go`, plus one in `gui/composer_seating_vectors_test.go`.

- [ ] **Step 3: Write the minting**

Create `gui/composer_cards.go`:

```go
package gui

import (
	"errors"
	"fmt"

	"seedhammer.com/md"
	"seedhammer.com/mk"
)

// Minting and re-minting the composer's key cards (SPEC §7d, §7f).
//
// EVERY SEATED SLOT YIELDS A CARD IN FORM B, whatever it was seated from: a
// key: record is MINTED as an mk1, a payload mk1 is RE-MINTED with both stubs
// APPENDED to the ones it already carries, and a seed-derived slot is minted
// likewise. Appending rather than replacing is what lets one card seat into
// either engraved form AND stay indexed to the wallets it already belonged to
// (§7d) -- reStubMk1 (gui/template_engrave.go:41-48) REPLACES, which is right
// for its own flow and wrong here.
//
// mk.Encode is deterministic (mk/encode.go:39), so a re-mint is exact.

// composerMintCard builds the mk1 for one seated slot.
//
// `keyedChunks` is nil for a partially seated composition, and md.ComposerStubs
// then returns the TEMPLATE stub alone -- which is §7f's rule, because the
// policy id does not exist until every slot is seated.
func composerMintCard(st *composerState, slot uint8, templateChunks, keyedChunks []string) ([]string, error) {
	if int(slot) >= len(st.assigned) {
		return nil, errors.New("composer: no such slot")
	}
	a := st.assigned[slot]
	if a.src < 0 {
		return nil, fmt.Errorf("composer: slot @%d is unseated and has no card", slot)
	}
	stubs, err := md.ComposerStubs(templateChunks, keyedChunks)
	if err != nil {
		return nil, err
	}
	src := st.sources[a.src]
	card := mk.Card{
		Network:     "mainnet", // LABEL only: mainnet by construction (§4f, gui/policy_address.go:61).
		Path:        composerOriginText(a.origin),
		Fingerprint: fmt.Sprintf("%x", a.fingerprint),
		Xpub:        a.xpub,
	}
	if src.kind == composerSourceCard {
		// RE-MINT: the payload card verbatim, with its own stubs kept in order
		// and the composer's appended.
		card = src.card
		card.Path = composerOriginText(a.origin)
	}
	return mk.Encode(mk.AppendStubs(card, stubs...))
}

// composerMintCards mints every seated slot's card, in emitted slot order --
// which is the order the census lists them and the order the restore document
// reads.
func composerMintCards(st *composerState, templateChunks, keyedChunks []string) ([]bundleCard, error) {
	var out []bundleCard
	for i := range st.assigned {
		if st.assigned[i].src < 0 {
			continue
		}
		strs, err := composerMintCard(st, uint8(i), templateChunks, keyedChunks)
		if err != nil {
			return nil, err
		}
		out = append(out, bundleCard{
			kind:    cardMK1,
			label:   fmt.Sprintf("mk1 key @%d", i),
			strings: strs,
			summary: composerOriginText(st.assigned[i].origin),
		})
	}
	return out, nil
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerMintCard|^TestComposerReMint' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL)'`
Expected: four PASS (`TestComposerMintCardCarriesBothStubsAndRoundTrips`, `TestComposerMintCardWithNoKeyedPolicyCarriesTheTemplateStubAlone`, `TestComposerReMintPreservesExistingStubsInOrder`, `TestComposerMintCardsSkipsUnseatedSlotsAndNamesTheRest`); `ok seedhammer.com/gui`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_cards.go gui/composer_cards_test.go
git commit -s -F - <<'MSG'
gui: mint and re-mint the composer's key cards, with both stubs appended (composer S3 task B8)

Existing stubs are preserved in order and each composed stub is appended once,
so a payload card stays indexed to the wallets it already belonged to --
reStubMk1 REPLACES, which is right for its own flow and wrong here. The card's
declared path is compared structurally: mk's codec renders the origin in the
other notation, and a string comparison fails on a correct card.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task B9: `gui/composer_census.go` -- the census over card chunks, and the measured concrete-descriptor ceiling

**Files:**
- Create: `gui/composer_census.go`
- Test: `gui/composer_census_test.go`

**Interfaces:**
- Consumes: `buildPlateCensusLines(params engrave.Params, cards []bundleCard) []string` (`gui/multisig_build_census.go:63`) and `bundlePlatePlan` (`gui/bundle_flow.go:466`) beneath it; `backup.Text`/`backup.Paragraph`, `backup.EngraveText`, `toPlate` (the pair `planTransactionTextPlates` uses, `gui/transaction.go:1163-1180`); `confirmReviewScreen`.
- Produces: `func composerDescriptorPlateFits(pl Platform, text string) bool`; `func composerDescriptorCeilingChars(pl Platform) int`; `func composerCensusLines(params engrave.Params, cards []bundleCard) []string`. **No `descriptor` parameter and no error return**: the ceiling refusal that would have needed them is F-457's, deferred with the text and QR descriptor plates, and this line said otherwise for a round.

**The census counts CARD chunks** (§7f): appending stubs can push a card into a third chunk (`mk/encode.go:26-29`), so the count must come off the cards after minting, not before. `buildPlateCensusLines` already derives its counts through `bundlePlatePlan` -- the same function `bundleEngrave` loops -- so it cannot drift from what is cut (`gui/multisig_build_census.go:37-46`); the composer reuses it rather than counting again.

**The ceiling is MEASURED BY SEARCH, never a constant**, copying `qrCeilingBytes`' shape exactly (`gui/transaction.go:1359-1391`, and its own comment says why: the answer depends on plate geometry, stroke width and the encoder's choices, so a constant would go stale silently in a refusal message nobody reads until the day it matters). `composerDescriptorCeilingChars` binary-searches `composerDescriptorPlateFits`, which builds a real `backup.Text` and asks `backup.EngraveText` plus `toPlate` -- the same "fit is decided by the real thing" rule `planTransactionTextPlates` states (`gui/transaction.go:1153-1156`).

**The refusal names the measured ceiling and a remedy**, as `gui/transaction.go:1336-1344` does: the character count, the measured ceiling at this plate and font, and the remedy that a template plus key cards holds the same wallet in fewer characters.

**Recovery-time error detection differs by form and the census says so** (§7f): md1 and mk1 carry BCH; a text descriptor carries only its BIP-380 checksum.

- [ ] **Step 1: the failing test**

Create `gui/composer_census_test.go`:

```go
package gui

import (
	"strings"
	"testing"
)

// TestComposerDescriptorCeilingIsMeasuredNotWrittenDown is the §13 item 1
// number, and the reason it is a search: a constant here goes stale the
// first time the plate geometry, the stroke width or the font moves, and it
// goes stale SILENTLY, inside a refusal.
func TestComposerDescriptorCeilingIsMeasuredNotWrittenDown(t *testing.T) {
	p := newPlatform()
	n := composerDescriptorCeilingChars(p)
	if n <= 0 {
		t.Fatalf("the measured descriptor ceiling is %d characters", n)
	}
	t.Logf("concrete descriptor plate ceiling: %d characters at this platform's params", n)
	// The search is EXACT at its own boundary, which is what makes the number
	// quotable in a refusal.
	if !composerDescriptorPlateFits(p, strings.Repeat("a", n)) {
		t.Errorf("the ceiling %d does not itself fit", n)
	}
	if composerDescriptorPlateFits(p, strings.Repeat("a", n+1)) {
		t.Errorf("one character past the ceiling %d still fits, so the search stopped short", n)
	}
	// C10's two-path wallet is 688 characters (brainstorm record). Whether it
	// fits is the MEASUREMENT this test records; it is not asserted either
	// way, because asserting a number nobody has measured is how a plan pins
	// a hope.
	t.Logf("C10's 688-character two-path wallet fits: %v",
		composerDescriptorPlateFits(p, strings.Repeat("a", 688)))
}

// TestComposerCensusLinesSayHowRecoveryDetectsAnError is §7f's last clause:
// recovery-time error detection differs by form and the census says so.
func TestComposerCensusLinesSayHowRecoveryDetectsAnError(t *testing.T) {
	cards := []bundleCard{{
		kind: cardMD1, label: "md1 template",
		strings: []string{"md1abc"}, summary: "key-less wallet policy",
	}}
	lines := composerCensusLines(newPlatform().EngraverParams(), cards)
	joined := strings.Join(lines, "\n")
	if !strings.Contains(joined, "This engraves") {
		t.Errorf("the census does not carry buildPlateCensusLines' plate count:\n%s", joined)
	}
	for _, want := range []string{"error correction", "only its checksum"} {
		if !strings.Contains(joined, want) {
			t.Errorf("the census does not say %q; md1/mk1 carry BCH and a plain "+
				"descriptor plate carries only its BIP-380 checksum:\n%s", want, joined)
		}
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDescriptorCeiling|^TestComposerCensusLines' ./gui/ 2>&1 | head -6`
Expected: FAIL to build -- `undefined: composerDescriptorCeilingChars`, `undefined: composerDescriptorPlateFits`, `undefined: composerCensusLines`.

- [ ] **Step 3: Write the census**

Create `gui/composer_census.go`:

```go
package gui

import (
	"seedhammer.com/backup"
	"seedhammer.com/engrave"
	"seedhammer.com/font/constant"
)

// The composer's plate census (SPEC §7f).
//
// IT REUSES buildPlateCensusLines (gui/multisig_build_census.go:63), whose
// counts are DERIVED through bundlePlatePlan -- the same function
// bundleEngrave loops -- so they cannot drift from what is actually cut. The
// composer contributes the cards and counts nothing itself. That matters more
// here than in Multisig Build: appending both stubs can push a card into a
// THIRD chunk (mk/encode.go:26-29), so a count taken before minting would be
// short by exactly the plates the composer added.
//
// THE DESCRIPTOR CEILING IS A SEARCH, NEVER A CONSTANT, and qrCeilingBytes
// says why on the QR side (gui/transaction.go:1361-1367): the answer depends
// on plate geometry, stroke width and the encoder's own choices, so a
// constant goes stale the first time any of them moves -- silently, inside a
// refusal message nobody reads until the day it matters.

// composerDescriptorPlateFits asks the REAL thing whether a descriptor fits
// one plate: build the plate, plan the engraving, let toPlate reject
// overflow. The same one-source-of-truth rule planTransactionTextPlates
// states for its own packing (gui/transaction.go:1153-1156).
func composerDescriptorPlateFits(pl Platform, text string) bool {
	params := pl.EngraverParams()
	plate := backup.Text{
		Paragraphs: []backup.Paragraph{{Text: text}},
		Font:       constant.Font,
	}
	plan, err := backup.EngraveText(params, plate)
	if err != nil {
		return false
	}
	_, err = toPlate(plan, params)
	return err == nil
}

// composerDescriptorCeilingChars is the largest descriptor that COULD have
// fitted, found by the same doubling-then-bisecting search qrCeilingBytes
// uses (gui/transaction.go:1381-1397). Called only on the refusal path, so
// its cost never lands on a working cut.
func composerDescriptorCeilingChars(pl Platform) int {
	fits := func(n int) bool {
		b := make([]byte, n)
		for i := range b {
			b[i] = 'a'
		}
		return composerDescriptorPlateFits(pl, string(b))
	}
	if !fits(1) {
		return 0
	}
	lo, hi := 1, 2
	for hi < 1<<16 && fits(hi) {
		lo, hi = hi, hi*2
	}
	for lo+1 < hi {
		mid := (lo + hi) / 2
		if fits(mid) {
			lo = mid
		} else {
			hi = mid
		}
	}
	return lo
}

// THE CENSUS REFUSAL LIVES WITH THE PLATE IT REFUSES, and that plate is
// deferred. §7f's "the census REFUSES a concrete descriptor longer than the
// plate holds" belongs to form A's TEXT plates -- and md deliberately emits no
// descriptor text ("a rendering that cannot be re-parsed is the defect this
// package's invariant exists to prevent"), so form A ships as the keyed md1
// this cycle and the text/QR plates are F-457, Rust-first. A refusal function
// with nothing to refuse is the same defect class as a picker whose answer
// nothing reads, so it is not carried here. The two measurement functions
// above stay: §13 item 1 asks for the ceiling as a number, and that is what
// TestComposerMeasureSection13Numbers records.

// composerCensusLines is the census the operator confirms before the first
// cut, plus §7f's read-back-integrity line.
func composerCensusLines(params engrave.Params, cards []bundleCard) []string {
	lines := buildPlateCensusLines(params, cards)
	// RECOVERY-TIME ERROR DETECTION DIFFERS BY FORM AND THE CENSUS SAYS SO
	// (§7f). md1 and mk1 carry BCH; a text or QR descriptor carries only its
	// BIP-380 checksum, which detects a typo and corrects nothing.
	return append(lines, "",
		"md1 and mk1 plates carry error correction. A plain descriptor plate "+
			"carries only its checksum, which finds a mistake but cannot fix one.")
}
```

- [ ] **Step 4: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerDescriptorCeiling|^TestComposerCensusLines' -v ./gui/ 2>&1 | grep -E '^(--- |ok|FAIL|.*ceiling)'`
Expected: **`2`** PASS, and the ceiling test logs the two numbers §13 item 1 records -- measured: `concrete descriptor plate ceiling: 596 characters at this platform's params` and `C10's 688-character two-path wallet fits: false`.

- [ ] **Step 5: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_census.go gui/composer_census_test.go
git commit -s -F - <<'MSG'
gui: the plate census over card chunks, and a ceiling found by search (composer S3 task B9)

The counts are derived through bundlePlatePlan, the same function
bundleEngrave loops, so they cannot drift from what is cut -- which matters
more here than in Multisig Build, because appending both stubs can push a card
into a third chunk. The ceiling is a binary search against the real plate, not
a constant that would go stale silently inside a refusal.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```


---

### Task B10: §12 item 6 -- the seating vector leg

**Files:**
- Test: `gui/composer_seating_vectors_test.go`

**Interfaces:**
- Consumes: the SHIPPED `seatKeyCards(templateMd1 []string, cards []mk.Card) ([]md.ExpandedKey, error)` (`gui/key_card_seating.go:53`); `md.ComposeWith`, `md.Composed.Bind` (`md/compose.go:353,250`); `md.ExpandWalletPolicyChunks` (`md/expand.go:102`); `policyAddressAt` (`gui/policy_address.go:87`) and `addrProofPerChain` (`gui/wallet_policy.go:241`); `composerMintCard` (Task B8); `deriveAccountXpub` (`gui/derive.go:19`); `mk.Decode` (`mk/mk.go:148`).
- Produces: five named vectors and four assertions over them.

**This task was five prose bullets with no code**, and the tests lens counted that as a Critical (its C-3): the one test meant to prove seated cards round-trip through the shipped consumer to a real keyed policy's addresses did not exist. It exists here.

**Why the SHIPPED seater and not a local check.** `seatKeyCards` is what a restoring operator's device runs years later, with none of this flow's state. If the composer's idea of "these cards fit this template" and that function's ever disagree, the plates are a backup only the machine that cut them can read.

**The vectors derive their own xpubs.** `mk.Encode` gates a card on the xpub's depth and last child matching the declared path (`mk: xpub depth/child does not match path`), so a taproot slot at `m/48'/0'/i'/3'` cannot carry a key derived at `.../2'`. Pinning two constants and reusing them across wrappers is exactly what that gate exists to catch, and it caught it while this task was written.

- [ ] **Step 1: Write the failing test**

Create `gui/composer_seating_vectors_test.go`:

```go
package gui

import (
	"encoding/binary"
	"testing"

	"github.com/btcsuite/btcd/btcutil/v2/hdkeychain"
	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip32"
	"seedhammer.com/md"
	"seedhammer.com/mk"
)

// SPEC §12 item 6, as vectors: the cards the composer mints SEAT into the
// template it engraved, through the SHIPPED seatKeyCards, and reproduce the
// keyed policy's addresses.
//
// WHY THE SHIPPED FUNCTION AND NOT A LOCAL CHECK. seatKeyCards
// (gui/key_card_seating.go:53) is what a restoring operator's device runs
// years later, with none of this flow's state. If the composer's own idea of
// "these cards fit this template" and that function's ever disagree, the
// plates are a backup that only the machine that cut them can read. So the
// assertion is: hand the shipped seater the shipped artifacts, and require it
// to reach the same keys and the same addresses.

// composerVector is one named shape, with the xpubs to seat into it.
type composerVector struct {
	name  string
	list  md.PathList
	slots int
	// addrs says this row's addresses must derive on this device. MEASURED
	// true for all five, taproot included: complexAddressSource
	// (gui/policy_address.go:44) reaches the tr script-tree shape the composer
	// emits. The flag stays as a column rather than being dropped, because the
	// S2 implementation report names taproot address derivation as a gap and a
	// row that stops deriving must fail loudly rather than quietly skip.
	addrs bool
}

func composerSeatingVectors() []composerVector {
	return []composerVector{
		{"wsh_sole_sortedmulti_2of2", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 2, N: 2, Sorted: true}},
		}}, 2, true},
		{"wsh_sole_multi_1of2_unsorted", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 2, Sorted: false}},
		}}, 2, true},
		{"wsh_two_paths_single_keys", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}},
			{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 1000}},
		}}, 2, true},
		{"sh_wsh_sole_sortedmulti_2of2", md.PathList{Wrapper: md.ComposeShWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 2, N: 2, Sorted: true}},
		}}, 2, true},
		{"tr_extracted_first_then_leaf", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}},
			{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 7}},
		}}, 2, true},
	}
}

// composerVectorXpub derives slot i's account key AT THE WRAPPER'S OWN §4f
// origin, from the published "abandon" vector, using the DEVICE's own
// deriveAccountXpub.
//
// DERIVED, NOT PINNED, and mk is what forces it: mk.Encode gates the card on
// the xpub's depth and last child matching the declared path
// ("mk: xpub depth/child does not match path"), so a taproot slot at
// m/48'/0'/i'/3' cannot carry a key derived at .../2'. Pinning two constants
// and reusing them across wrappers is exactly the mistake that gate exists to
// catch, and it caught it here.
func composerVectorXpub(t *testing.T, scriptType, account uint32) (string, [4]byte) {
	t.Helper()
	const h = hdkeychain.HardenedKeyStart
	path := bip32.Path{48 | h, 0 | h, account | h, scriptType | h}
	xpub, masterFP, err := deriveAccountXpub(composerTestMnemonic(t), "", &chaincfg.MainNetParams, path)
	if err != nil {
		t.Fatalf("deriveAccountXpub(%s): %v", path, err)
	}
	var fp [4]byte
	binary.BigEndian.PutUint32(fp[:], masterFP)
	return xpub, fp
}

// composerVectorArtifacts composes the keyless template and the keyed policy
// from ONE path list, and returns the state whose assignments describe it.
//
// COMPOSED TWICE, NOT COPIED: md.Composed's doc says a copy shares the
// underlying descriptor, so Bind on one keys them both -- and a "template"
// that had been keyed by its own policy would make every assertion here
// tautological.
func composerVectorArtifacts(t *testing.T, v composerVector) (st *composerState, template, keyed []string) {
	t.Helper()
	n := v.slots
	declared := make([]*md.SlotOrigin, n)
	st = &composerState{list: v.list, reg: &seedRegistry{}}
	st.assigned = make([]composerAssignment, n)
	st.sources = make([]composerSource, n)
	pub := map[uint8][65]byte{}
	fps := map[uint8][4]byte{}
	scriptType := uint32(2)
	if v.list.Wrapper == md.ComposeTr {
		scriptType = 3
	}
	for i := 0; i < n; i++ {
		x, fp := composerVectorXpub(t, scriptType, uint32(i))
		origin := composerTestOrigin(scriptType, uint32(i))
		declared[i] = &md.SlotOrigin{Origin: origin, Fingerprint: fp, FpPresent: true}
		st.sources[i] = composerSource{kind: composerSourceKey, seedID: -1, xpub: x}
		st.assigned[i] = composerAssignment{
			src: i, account: uint32(i), origin: origin,
			fingerprint: fp, fpPresent: true, xpub: x,
		}
		cc, pk, _, err := decodeXpubBytes(x)
		if err != nil {
			t.Fatalf("%s: decodeXpubBytes: %v", v.name, err)
		}
		var b [65]byte
		copy(b[0:32], cc[:])
		copy(b[32:65], pk[:])
		pub[uint8(i)] = b
		fps[uint8(i)] = fp
	}
	ct, err := md.ComposeWith(v.list, declared)
	if err != nil {
		t.Fatalf("%s: md.ComposeWith (template): %v", v.name, err)
	}
	if template, err = ct.Chunks(); err != nil {
		t.Fatalf("%s: template Chunks: %v", v.name, err)
	}
	ck, err := md.ComposeWith(v.list, declared)
	if err != nil {
		t.Fatalf("%s: md.ComposeWith (keyed): %v", v.name, err)
	}
	if err := ck.Bind(pub, fps); err != nil {
		t.Fatalf("%s: Bind: %v", v.name, err)
	}
	if keyed, err = ck.Chunks(); err != nil {
		t.Fatalf("%s: keyed Chunks: %v", v.name, err)
	}
	return st, template, keyed
}

func composerDecodeCards(t *testing.T, st *composerState, template, keyed []string) []mk.Card {
	t.Helper()
	var cards []mk.Card
	for i := range st.assigned {
		if st.assigned[i].src < 0 {
			continue
		}
		strs, err := composerMintCard(st, uint8(i), template, keyed)
		if err != nil {
			t.Fatalf("composerMintCard(@%d): %v", i, err)
		}
		c, err := mk.Decode(strs)
		if err != nil {
			t.Fatalf("the card minted for @%d does not decode: %v", i, err)
		}
		cards = append(cards, c)
	}
	return cards
}

// TestComposerMintedCardsSeatThroughTheShippedSeater is §12 item 6's positive
// leg, over every named vector.
func TestComposerMintedCardsSeatThroughTheShippedSeater(t *testing.T) {
	for _, v := range composerSeatingVectors() {
		t.Run(v.name, func(t *testing.T) {
			st, template, keyed := composerVectorArtifacts(t, v)
			cards := composerDecodeCards(t, st, template, keyed)
			if len(cards) != v.slots {
				t.Fatalf("minted %d card(s) for %d seated slot(s)", len(cards), v.slots)
			}
			seated, err := seatKeyCards(template, cards)
			if err != nil {
				t.Fatalf("the SHIPPED seater refuses the composer's own cards: %v", err)
			}
			_, want, err := md.ExpandWalletPolicyChunks(keyed)
			if err != nil {
				t.Fatal(err)
			}
			if len(seated) != len(want) {
				t.Fatalf("seating gave %d slot(s), the keyed policy has %d", len(seated), len(want))
			}
			for i := range seated {
				if !seated[i].XpubPresent {
					t.Errorf("slot @%d came back from seating with no key", seated[i].Index)
					continue
				}
				if seated[i].Xpub != want[i].Xpub {
					t.Errorf("slot @%d seated a different key than the keyed policy holds",
						seated[i].Index)
				}
			}
		})
	}
}

// TestComposerSeatedTemplateReproducesTheKeyedPolicysAddresses is the half
// that matters to an operator: the plates derive the same wallet.
func TestComposerSeatedTemplateReproducesTheKeyedPolicysAddresses(t *testing.T) {
	derived := 0
	for _, v := range composerSeatingVectors() {
		t.Run(v.name, func(t *testing.T) {
			st, template, keyed := composerVectorArtifacts(t, v)
			cards := composerDecodeCards(t, st, template, keyed)
			seated, err := seatKeyCards(template, cards)
			if err != nil {
				t.Fatalf("seatKeyCards: %v", err)
			}
			tplT, _, err := md.ExpandWalletPolicyChunks(template)
			if err != nil {
				t.Fatal(err)
			}
			tplK, keysK, err := md.ExpandWalletPolicyChunks(keyed)
			if err != nil {
				t.Fatal(err)
			}
			atSeated, okSeated := policyAddressAt(template, tplT, seated)
			atKeyed, okKeyed := policyAddressAt(keyed, tplK, keysK)
			if okSeated != okKeyed {
				t.Fatalf("addresses derive from one form and not the other "+
					"(seated=%v keyed=%v); the two artifacts would then prove "+
					"different things", okSeated, okKeyed)
			}
			if !okKeyed {
				if v.addrs {
					t.Fatalf("this vector is marked address-checkable and no address " +
						"derives; either the row or the deriver is wrong")
				}
				t.Logf("%s: no address derives for this shape on this device "+
					"(the taproot script-tree gap); the ids still bind it", v.name)
				return
			}
			derived++
			for i := 0; i < addrProofPerChain; i++ {
				for _, change := range []bool{false, true} {
					a, err := atSeated(uint32(i), change)
					if err != nil {
						t.Fatalf("seated address %d/%v: %v", i, change, err)
					}
					b, err := atKeyed(uint32(i), change)
					if err != nil {
						t.Fatalf("keyed address %d/%v: %v", i, change, err)
					}
					if a != b {
						t.Errorf("address %d (change=%v) differs: seated %s, keyed %s",
							i, change, a, b)
					}
				}
			}
		})
	}
	if derived == 0 {
		t.Fatal("INCONCLUSIVE: no vector derived an address, so this test compared nothing")
	}
}

// TestComposerPartiallySeatedArtifactIsANamedVector is §7f's partially seated
// form and §12 item 6's last clause: the template plus TEMPLATE-STUB cards for
// the seated slots only, and the shipped seater must REFUSE it as incomplete
// rather than deriving a wallet from half a key set.
func TestComposerPartiallySeatedArtifactIsANamedVector(t *testing.T) {
	v := composerSeatingVectors()[0]
	st, template, _ := composerVectorArtifacts(t, v)
	st.assigned[1].src = -1 // one slot left unseated

	cards := composerDecodeCards(t, st, template, nil)
	if len(cards) != 1 {
		t.Fatalf("a partially seated composition minted %d card(s), want 1", len(cards))
	}
	tmplStub, err := md.FormAwareStubChunks(template)
	if err != nil {
		t.Fatal(err)
	}
	if len(cards[0].Stubs) != 1 || cards[0].Stubs[0] != tmplStub {
		t.Errorf("the card carries %d stub(s) (%x); a partially seated composition has "+
			"no policy id yet, so it must carry the TEMPLATE stub alone (%x)",
			len(cards[0].Stubs), cards[0].Stubs, tmplStub)
	}
	if _, err := seatKeyCards(template, cards); err == nil {
		t.Error("the shipped seater ACCEPTED a template with an unfilled slot; a partial " +
			"seating is exactly the state that produces a plausible wrong address")
	}
}

// TestComposerNeverProducesTheAsymmetricOneCardTemplate is §12 item 6's NAMED
// NEGATIVE: "a named negative vector runs the asymmetric one-card case (one
// slot with a fingerprint, one without, at one origin) and asserts it is never
// produced".
//
// IT IS REFUSED TWICE, AND THE SECOND REFUSAL IS THE STRONGER ONE. §8v catches
// it at the mapping review, where the operator can act on it; md.ComposeWith
// refuses to EMIT it at all, so the artifact cannot leave this device even if
// a future screen forgot to ask. Both are asserted, because a defence that
// exists only in the UI is a defence one refactor from being gone.
//
// WHAT THIS MEANS FOR THE HAZARD §4f NAMES. The plan's own framing was that
// the hand-built asymmetric template would be shown to double-seat through
// seatKeyCards (slotMatchesCard skips the fingerprint test when the template
// declares none, gui/key_card_seating.go:151-159). Measured here: that
// template cannot be built through md.ComposeWith at all, so the double-seat
// is unreachable through this device's own builder and the demonstration
// belongs to the seating package's own tests, not to the composer's. Said
// plainly rather than left as an assertion that quietly tests nothing.
func TestComposerNeverProducesTheAsymmetricOneCardTemplate(t *testing.T) {
	origin := composerTestOrigin(2, 0)
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 2, Sorted: true}},
	}}

	// (a) THE REVIEW REFUSES IT (§8v). One fingerprint present, one absent, at
	//     one origin: the asymmetric case, which is the dangerous one.
	st := &composerState{list: list, assigned: []composerAssignment{
		{src: 0, origin: origin, fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, fpPresent: true},
		{src: 1, origin: origin},
	}}
	if !composerInvariantViolation(st) {
		t.Fatal("(a) the mapping review would ACCEPT two slots at one origin with one " +
			"fingerprint; §8v exists to refuse exactly this")
	}

	// (b) THE BUILDER REFUSES TO EMIT IT. Same shape, handed straight to
	//     md.ComposeWith with the screens bypassed entirely.
	for _, tc := range []struct {
		name     string
		declared []*md.SlotOrigin
	}{
		{"one fingerprint, one without", []*md.SlotOrigin{
			{Origin: origin, Fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, FpPresent: true},
			{Origin: origin, FpPresent: false},
		}},
		{"neither declares one", []*md.SlotOrigin{
			{Origin: origin, FpPresent: false},
			{Origin: origin, FpPresent: false},
		}},
		{"the same fingerprint twice", []*md.SlotOrigin{
			{Origin: origin, Fingerprint: [4]byte{1, 2, 3, 4}, FpPresent: true},
			{Origin: origin, Fingerprint: [4]byte{1, 2, 3, 4}, FpPresent: true},
		}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			if _, err := md.ComposeWith(list, tc.declared); err == nil {
				t.Errorf("md.ComposeWith EMITTED a template violating §4f's invariant; " +
					"one card could then fill both slots and be presented as reviewed")
			}
		})
	}

	// And the CONTROL: two distinct fingerprints at one origin is LEGAL, so
	// the refusals above are the invariant and not a blanket ban on sharing an
	// origin.
	ok := []*md.SlotOrigin{
		{Origin: origin, Fingerprint: [4]byte{1, 2, 3, 4}, FpPresent: true},
		{Origin: origin, Fingerprint: [4]byte{9, 9, 9, 9}, FpPresent: true},
	}
	if _, err := md.ComposeWith(list, ok); err != nil {
		t.Errorf("INCONCLUSIVE: two DISTINCT fingerprints at one origin were refused (%v), "+
			"so the assertions above may be measuring a wider ban than §4f states", err)
	}
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerMintedCards|^TestComposerSeatedTemplate|^TestComposerPartiallySeated|^TestComposerNeverProduces' ./gui/ 2>&1 | head -6`
Expected: FAIL to build before Task B8 lands -- `# seedhammer.com/gui [seedhammer.com/gui.test]` then `undefined: composerMintCard`, in this file.

- [ ] **Step 3: Run the vectors**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerMintedCards|^TestComposerSeatedTemplate|^TestComposerPartiallySeated|^TestComposerNeverProduces' -v ./gui/ 2>&1 | grep -E '^(--- |    --- |ok|FAIL)'`
Expected: four top-level PASS with **five sub-tests each** under the two vector tests (`wsh_sole_sortedmulti_2of2`, `wsh_sole_multi_1of2_unsorted`, `wsh_two_paths_single_keys`, `sh_wsh_sole_sortedmulti_2of2`, `tr_extracted_first_then_leaf`) and three under the asymmetric negative; `ok seedhammer.com/gui`. **Measured while this task was written: all five wrappers derive addresses, taproot included** -- `complexAddressSource` (`gui/policy_address.go:44`) reaches the tr script-tree shape the composer emits, so no row is address-exempt.

- [ ] **Step 4: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_seating_vectors_test.go
git commit -s -F - <<'MSG'
gui: the seating vector leg -- minted cards seat through the SHIPPED seater and reproduce the keyed policy's addresses (composer S3 task B10)

Five wrappers, each composed twice (a copy of a Composed shares its
descriptor, so Bind on one keys them both). The asymmetric one-card template
section 4f names is refused twice over -- by the mapping review and by
md.ComposeWith itself -- so the double-seat hazard is unreachable through this
device's own builder, which is stated rather than asserted as a test that
would then be testing nothing.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task B11: `gui/composer_flow.go` -- THE JOIN, and the walk that can fail on it

**Files:**
- Replace: `gui/composer_flow.go`
- Test: `gui/composer_join_test.go`
- Modify (FRAGMENT): `gui/wallet_policy.go` (the door loop, so Back at the wrapper lands on the door rather than the carousel)

**Interfaces:**
- Consumes: everything Part B built -- `composerKeySources`, `composerCardSources`, `composerSeedSource`, `composerSeatFlow`, `composerShortfall`, `composerSeatingComplete`, `composerMappingReview`, `composerConsentFlow`, `composerFormPick`, `composerEngraveModePick`, `composerMintCards`, `composerCensusLines` -- plus `md.ComposeWith`/`Bind`, `decodeXpubBytes` (`gui/singlesig_derive.go:110`), `codex32.EncodeMS1` (`codex32/msencode.go:17`), `bundleEngrave` (`gui/bundle_flow.go:616`), `confirmReviewScreen` (`gui/multisig_build.go:1877`).
- Produces: `composerFlow` in §7's order; `composerSizeAssignments`; `composerDeclaredOrigins`; `composerTemplateChunksFor`; `composerArtifactsFor`; `composerSeatingStep`; `composerEngraveStep`; `composerSecretCards`.

**THIS IS THE CRITICAL ALL THREE R0 LENSES FOUND.** Part B declared fourteen production functions and nothing called any of them. Measured in the gate's own scratch (declarations excluded):

```
DEAD-IN-PROD: composerApplyShapeEdit, composerCardSources, composerCensusLines,
composerCensusRefusal, composerConsentFlow, composerEngraveModePick,
composerFormPick, composerKeySources, composerMappingReview, composerMintCards,
composerSeatFlow, composerSeatingComplete, composerSecretFormPick, composerShortfall
```

An operator with four `key:` records in flash read the door's "Keys loaded: 4", chose Build, and was never offered a slot; §7e's self-check and §8q never executed; §7f's form choice, card minting and the census's card-chunk count were unreachable; and §7b's live line read `keys available: 0` whatever the payload held. **Go does not error on an unused package-scope function, and every `TestComposer*` called these directly**, so the suite was green over a feature no operator could reach. The plan promised a ``Replace `gui/composer_flow.go` `` and no task supplied it.

**Two gates go in with the code**, because one of them is cheap and the other is the only one that can fail for the right reason: a structural scan that requires every `composer*` production function to have a production caller, and a WALK from a keyed payload to the census.

**Sources are loaded BEFORE the shape** so §7b's live line is right from the first frame; loading them at seating time made the line that helps the operator SIZE the policy read zero for the whole of the decision it exists to inform.

**§7b's preset-or-blank wiring point is named here** (`composerFlow`, immediately after `composerWrapperPick`) so Task A10 is a fill-in when F-453 lands rather than a redesign.

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_join_test.go`:

```go
package gui

import (
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"sort"
	"strings"
	"testing"
	"testing/synctest"
	"time"
)

// ═══ THE JOIN ═══════════════════════════════════════════════════════════════
//
// Two R0 lenses found the same Critical independently: Part B built fourteen
// production functions and nothing called any of them. Go does not error on an
// unused package-scope function, and every other TestComposer* calls these
// directly, so the suite was green over a feature no operator could reach.
//
// The two tests below are the gate against that class. The first is
// STRUCTURAL and cheap; the second is a WALK, and only a walk can fail for the
// right reason.

// TestComposerEveryScreenFunctionHasAProductionCaller is the cheap half.
//
// It parses gui/*.go (non-test), counts call expressions per function name,
// and requires every composer*Flow / composer*Pick / composer*Step / the
// named screen entry points to be CALLED somewhere other than their own
// declaration. "An unreachable feature" is not something the build gate can
// see, and this is what puts it inside a command.
func TestComposerEveryScreenFunctionHasAProductionCaller(t *testing.T) {
	fset := token.NewFileSet()
	declared := map[string]bool{}
	called := map[string]bool{}
	ents, err := os.ReadDir(".")
	if err != nil {
		t.Fatal(err)
	}
	for _, e := range ents {
		name := e.Name()
		if e.IsDir() || !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		f, err := parser.ParseFile(fset, name, nil, 0)
		if err != nil {
			t.Fatalf("parsing %s: %v", name, err)
		}
		for _, d := range f.Decls {
			fn, ok := d.(*ast.FuncDecl)
			if !ok {
				continue
			}
			if fn.Recv == nil && strings.HasPrefix(fn.Name.Name, "composer") {
				declared[fn.Name.Name] = true
			}
			// EVERY body is scanned for calls, not just the composer ones:
			// composerDoorFlow and composerFlow are called from
			// walletPolicyFlow, which is not a composer* name, and a scan
			// restricted to composer* bodies reported the two entry points of
			// the feature as unreachable -- a false positive that would have
			// taught the next reader to distrust this test.
			if fn.Body == nil {
				continue
			}
			ast.Inspect(fn.Body, func(n ast.Node) bool {
				call, ok := n.(*ast.CallExpr)
				if !ok {
					return true
				}
				if id, ok := call.Fun.(*ast.Ident); ok && id.Name != fn.Name.Name {
					called[id.Name] = true
				}
				return true
			})
		}
	}
	if len(declared) < 40 {
		t.Fatalf("INCONCLUSIVE: only %d composer* declarations found; the scan is broken, "+
			"not the tree", len(declared))
	}
	// THE EXEMPTIONS ARE NAMED AND ARGUED, never a silent skip: a test that
	// can be quieted by adding a name is not a gate.
	exempt := map[string]string{
		"composerDescriptorPlateFits":    "§13 item 1's measurement; its consumer, the concrete text/QR descriptor plate, is deferred to F-457 because md deliberately emits no text",
		"composerDescriptorCeilingChars": "the same measurement, called by TestComposerMeasureSection13Numbers",
	}
	var orphans []string
	for name := range declared {
		if called[name] {
			continue
		}
		if why, ok := exempt[name]; ok {
			t.Logf("exempt: %s -- %s", name, why)
			continue
		}
		orphans = append(orphans, name)
	}
	sort.Strings(orphans)
	if len(orphans) != 0 {
		t.Errorf("these composer functions have no production caller, so the screens they "+
			"draw cannot be reached by any operator: %v\n"+
			"This is the defect two R0 lenses found: fourteen of them at once, with a "+
			"green suite, because every test called them directly.", orphans)
	}
}

// TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen is the flow-level
// walk C-1 asks for: door → Build → wrapper → shape → stub → seating →
// mapping review → consent (self-check) → §8l → form choice → census.
//
// IT IS THE ONLY KIND OF TEST THAT CAN FAIL ON THE JOIN. Every screen below
// has its own unit test and all of them passed while the flow reached none of
// them.
func TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newEngravedAwarePlatform()
		p.engraver = newEngraver()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		// TWO key: records, at two accounts of one master -- C5's normal case,
		// and the shape §4f's invariant permits (distinct origins).
		ctx.sysw = composerSessionWith([]string{
			composerTestKeyRecord, composerTestKeyRecord2, composerTestNowRecord,
		}, nil)

		frame, quit := runUI(ctx, func() { walletPolicyFlow(ctx, &descriptorTheme) })
		defer quit()

		// (1) THE DOOR, with §8r's key count for the payload actually loaded.
		got, ok := pumpUntil(frame, "Build a new policy", 24)
		if !ok {
			t.Fatalf("the door never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "Keys loaded: 2") {
			t.Errorf("the door does not count the payload's keys.\nFrame: %q", got)
		}
		click(&ctx.Router, Down, Down) // Scan cards -> From payload -> Build
		click(&ctx.Router, Button3)

		// (2) THE WRAPPER.
		if got, ok = pumpUntil(frame, "Which script?", 24); !ok {
			t.Fatalf("the wrapper picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // Taproot -> Segwit (wsh)
		click(&ctx.Router, Button3)

		// (3) THE PATH LIST, whose live line must read the sources the FLOW
		// loaded -- the half that read `keys available: 0` for every payload.
		if got, ok = pumpUntil(frame, "Add a spend path", 24); !ok {
			t.Fatalf("the path list never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "keys available: 2") {
			t.Errorf("the path list does not count the payload's keys; §7b's live line "+
				"reads st.sources, which only the flow can populate.\nFrame: %q", got)
		}
		click(&ctx.Router, Button3) // an empty list opens on "Add a spend path"

		// (4) KEYS: 2 of 2.
		if got, ok = pumpUntil(frame, "What can spend on this path?", 24); !ok {
			t.Fatalf("the path kind picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // Keys
		if got, ok = pumpUntil(frame, "how many keys?", 24); !ok {
			t.Fatalf("the key-count picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // 1 -> 2
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "how many must sign?", 24); !ok {
			t.Fatalf("the threshold picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down) // 1 -> 2
		click(&ctx.Router, Button3)

		// (5) DONE -> the key-order question, asked where `sole` is FINAL.
		if got, ok = pumpUntil(frame, "Path 1: 2-of-2", 24); !ok {
			t.Fatalf("the path list does not show the new path.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down, Down, Down) // path 1 -> Add -> Change script -> Done
		click(&ctx.Router, Button3)
		if got, ok = pumpUntil(frame, "Sorted keys, or your order?", 24); !ok {
			t.Fatalf("the key-order question is not asked at the transition.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // Sorted (usual)

		// (6) THE STUB SCREEN, paged: the checkmark is withheld until the last
		// page has been laid out once, so this pages to the end.
		if got, ok = pumpUntil(frame, "mk1 stub (template)", 32); !ok {
			t.Fatalf("the stub screen never drew.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// (7) SEATING, slot by slot, from the payload's own records.
		if got, ok = pumpUntil(frame, "choose a key", 32); !ok {
			t.Fatalf("seating never drew -- this is the join: the payload's keys are "+
				"loaded and no slot was ever offered.\nLast frame: %q", got)
		}
		if !uiContains(got, "Slot @0") {
			t.Errorf("the first seating prompt does not name slot @0.\nFrame: %q", got)
		}
		click(&ctx.Router, Button3) // the first remaining source
		if got, ok = pumpUntil(frame, "Slot @1", 32); !ok {
			t.Fatalf("the second slot was never offered.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3)

		// (8) THE MAPPING REVIEW.
		if got, ok = pumpUntil(frame, "cannot confirm", 32); !ok {
			t.Fatalf("the mapping review never drew.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// (9) THE STUB SCREEN AGAIN, now with the keyed policy's id, then
		//     CONSENT -- whose self-check must pass on an honest build.
		if got, ok = pumpUntil(frame, "mk1 stub (policy)", 32); !ok {
			t.Fatalf("the keyed stub screen never drew.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// PAGE LOOKING FOR THE ADDRESSES. They are §7e's proof of WHICH wallet
		// this is, and they are several pages in on a policy with two paths
		// and four address lines -- which is exactly why the checkmark is
		// withheld until the last page has been laid out.
		if got, ok = composerPageUntil(t, ctx, frame, "Receive 0", 12); !ok {
			t.Fatalf("the consent surface never drew its addresses -- §8q would have "+
				"fired instead if the self-check refused.\nLast frame: %q", got)
		}
		composerPageToEnd(t, ctx, frame)

		// (10) §8l, unskippable, hold to confirm.
		if got, ok = pumpUntil(frame, "Nothing outside this device", 48); !ok {
			t.Fatalf("§8l never drew.\nLast frame: %q", got)
		}
		// HOLD, then let the fake clock run out the delay: the shipped walk
		// does exactly this (gui/multisig_build_walk_test.go:317-320), and a
		// press with no sleep leaves the confirm at partial progress forever.
		press(&ctx.Router, Button3)
		frame()
		time.Sleep(confirmDelay)
		frame()

		// (11) THE FORM CHOICE (§7f), offered because every slot is seated.
		if got, ok = pumpUntil(frame, "Which form?", 48); !ok {
			t.Fatalf("the engrave form choice never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "Template plus key cards") {
			t.Errorf("form B is not offered on a fully seated composition.\nFrame: %q", got)
		}
		click(&ctx.Router, Down) // The policy itself -> Template plus key cards
		click(&ctx.Router, Button3)

		// (12) THE CENSUS, counting the md1 AND the minted key cards.
		if got, ok = pumpUntil(frame, "This engraves", 64); !ok {
			t.Fatalf("the plate census never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "mk1 key @0") {
			t.Errorf("the census does not count the minted key cards; §7f says it counts "+
				"CARD chunks too.\nFrame: %q", got)
		}
		// The BCH-versus-checksum line is on a later page of the census, which
		// is the shipped confirmReviewScreen and pages as it always did.
		if got, ok = composerPageUntil(t, ctx, frame, "error correction", 8); !ok {
			t.Errorf("the census never says how recovery detects an error.\nLast frame: %q", got)
		}
	})
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerEveryScreenFunction|^TestComposerWalkFromAKeyedPayload' ./gui/ 2>&1 | head -8`
Expected: FAIL. `TestComposerEveryScreenFunctionHasAProductionCaller` names the dead functions verbatim (`composerSeatFlow`, `composerMappingReview`, `composerConsentFlow`, `composerFormPick`, `composerMintCards`, `composerCensusLines`, `composerShortfall`, `composerKeySources`, `composerCardSources`, `composerEngraveModePick`, `composerSeatingComplete`, `composerApplyShapeEdit`); the walk fails at `seating never drew -- this is the join: the payload's keys are loaded and no slot was ever offered.`

- [ ] **Step 3: Replace `gui/composer_flow.go`**

Replace `gui/composer_flow.go`:

```go
package gui

import (
	"fmt"
	"slices"

	"seedhammer.com/codex32"
	"seedhammer.com/md"
)

// composerFlow is "Build a new policy" (SPEC_wallet_policy_composer.md §7),
// from the door to the plate, in §7's order.
//
// THIS FUNCTION IS THE JOIN, and its absence was the defect two R0 lenses
// found independently. Part B built fourteen production functions --
// composerKeySources, composerCardSources, composerSeatFlow, composerShortfall,
// composerMappingReview, composerConsentFlow, composerFormPick,
// composerMintCards, composerCensusLines and the rest -- and nothing called
// any of them, so an operator with four `key:` records in flash read the
// door's "Keys loaded: 4", chose Build, and was never offered a slot. §7e's
// self-check and §8q never executed on a shipped device; §7f's form choice,
// card minting and the census's card-chunk count were unreachable; and §7b's
// live line read `keys available: 0` whatever the payload held. Go does not
// error on an unused package-scope function, and every TestComposer* called
// these directly, so nothing was red. The lesson is recorded in memory as
// "plans list components and omit the call that joins them"; the gate against
// it is composerFlowWalk's flow-level test and the reachability test beside it.
//
// THE SCRUB IS INSTALLED HERE, at the top, before any seed can exist -- the
// same construction gui/multisig_build.go:290-291 uses, so every exit below (a
// Back, a refusal, a ctx.Done unwind, a panic) is covered without an
// implementer remembering to add one to a new return (C14).
func composerFlow(ctx *Context, th *Colors) {
	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
	defer st.reg.scrub()

	// SOURCES ARE LOADED BEFORE THE SHAPE, so §7b's live line
	// ("slots: N / keys available: M") is right from the first frame. Loading
	// them at seating time would make the line that helps the operator SIZE the
	// policy read zero for the whole of the decision it exists to inform.
	st.sources = append(composerKeySources(ctx), composerCardSources(ctx)...)

	w, ok := composerWrapperPick(ctx, th)
	if !ok {
		return
	}
	st.list.Wrapper = w

	var shown []string // the chunk set the stub screen last displayed (§8s)
	for !ctx.Done {
		if !composerShapeFlow(ctx, th, st) {
			// BACK AT THE PATH LIST GOES BACK ONE STEP, to the wrapper, with
			// the list intact -- §7b's "going back should lose nothing". It
			// used to return, dropping the wrapper, every path, every lock and
			// every digest.
			w, ok := composerWrapperPick(ctx, th)
			if !ok {
				return
			}
			st.list.Wrapper = w
			continue
		}
		composerSizeAssignments(st)

		template, err := composerTemplateChunksFor(st)
		if err != nil {
			composerShowRefusal(ctx, th, "Template", err)
			continue
		}
		// §8s's changed-id line is decided by COMPARING CHUNK SETS, not by an
		// "edited" flag. The flag was set on any Back out of the stub screen,
		// the consent or §8l and never reset, so re-reaching the stub screen
		// without touching the shape asserted that the id had changed -- a
		// false statement on the screen whose job is to be copied onto steel,
		// which trains the operator to discount the line that will one day be
		// true.
		changed := shown != nil && !slices.Equal(shown, template)
		if !composerStubFlow(ctx, th, template, nil, changed) {
			shown = template
			continue
		}
		shown = template

		if !composerSeatingStep(ctx, th, st) {
			continue
		}
		template, keyed, err := composerArtifactsFor(st)
		if err != nil {
			composerShowRefusal(ctx, th, "Template", err)
			continue
		}
		if len(keyed) > 0 && !composerStubFlow(ctx, th, template, keyed, false) {
			continue
		}
		consent := template
		if len(keyed) > 0 {
			consent = keyed
		}
		if !composerConsentFlow(ctx, th, st, consent) {
			continue
		}
		if composerEngraveStep(ctx, th, st, template, keyed) {
			return
		}
	}
}

// composerSizeAssignments sizes st.assigned to the shape's slot count AT FLOW
// LEVEL, not at seating entry.
//
// Sizing it only inside composerSeatFlow left a composition that never entered
// seating -- C26's key-less template, §12 item 3 -- failing composerSelfCheck
// on slot count alone. An unseated slot is src == -1, never a zero value: zero
// would read as "seated from source 0".
func composerSizeAssignments(st *composerState) {
	n := composerSlotCount(st.list)
	if len(st.assigned) == n {
		return
	}
	st.assigned = make([]composerAssignment, n)
	for i := range st.assigned {
		st.assigned[i].src = -1
	}
}

// composerDeclaredOrigins projects the seating onto md.ComposeWith's input.
// An unseated slot declares nothing and the codec assigns §4f's lowest free
// account for it.
func composerDeclaredOrigins(st *composerState) []*md.SlotOrigin {
	out := make([]*md.SlotOrigin, len(st.assigned))
	for i, a := range st.assigned {
		if a.src < 0 {
			continue
		}
		out[i] = &md.SlotOrigin{Origin: a.origin, Fingerprint: a.fingerprint, FpPresent: a.fpPresent}
	}
	return out
}

// composerTemplateChunksFor emits the keyless template for the current shape
// and seating.
func composerTemplateChunksFor(st *composerState) ([]string, error) {
	c, err := md.ComposeWith(st.list, composerDeclaredOrigins(st))
	if err != nil {
		return nil, err
	}
	return c.Chunks()
}

// composerArtifactsFor emits the template AND, when every slot is seated, the
// keyed policy.
//
// COMPOSED TWICE, NOT COPIED: md.Composed's own doc says a copy shares the
// underlying descriptor, so Bind on one keys them both -- and a "template"
// that had been keyed by its own policy is not a template.
func composerArtifactsFor(st *composerState) (template, keyed []string, err error) {
	declared := composerDeclaredOrigins(st)
	ct, err := md.ComposeWith(st.list, declared)
	if err != nil {
		return nil, nil, err
	}
	if template, err = ct.Chunks(); err != nil {
		return nil, nil, err
	}
	if !composerSeatingComplete(st) {
		return template, nil, nil
	}
	pub := map[uint8][65]byte{}
	fps := map[uint8][4]byte{}
	for i, a := range st.assigned {
		cc, pk, _, derr := decodeXpubBytes(a.xpub)
		if derr != nil {
			return nil, nil, derr
		}
		var b [65]byte
		copy(b[0:32], cc[:])
		copy(b[32:65], pk[:])
		pub[uint8(i)] = b
		if a.fpPresent {
			fps[uint8(i)] = a.fingerprint
		}
	}
	ck, err := md.ComposeWith(st.list, declared)
	if err != nil {
		return nil, nil, err
	}
	if err := ck.Bind(pub, fps); err != nil {
		return nil, nil, err
	}
	if keyed, err = ck.Chunks(); err != nil {
		return nil, nil, err
	}
	return template, keyed, nil
}

// composerSeatingStep is §7d: seat every slot, or take §8p's exit.
//
// Offered only when the payload holds keys or seeds, or the operator asks to
// type one -- §7d's own condition. With nothing to seat from, a key-less
// template is the C26 answer and the flow goes straight on.
func composerSeatingStep(ctx *Context, th *Colors, st *composerState) bool {
	if len(st.sources) == 0 {
		cs := &ChoiceScreen{
			Title:   "Keys",
			Lead:    "Seat keys into this template?",
			Choices: []string{"Engrave a key-less template", "Type a seed"},
		}
		sel, ok := cs.Choose(ctx, th)
		if !ok {
			return false
		}
		if sel == 0 {
			return true
		}
		src, ok := composerSeedSource(ctx, th, st)
		if !ok {
			return false
		}
		st.sources = append(st.sources, src)
	}
	if !composerSeatFlow(ctx, th, st) {
		return false
	}
	if !composerSeatingComplete(st) && !composerShortfall(ctx, th, st) {
		return false
	}
	return composerMappingReview(ctx, th, st)
}

// composerEngraveStep is §7f: the form choice, then what it implies.
func composerEngraveStep(ctx *Context, th *Colors, st *composerState, template, keyed []string) bool {
	form, ok := composerFormPick(ctx, th, st)
	if !ok {
		return false
	}
	var cards []bundleCard
	switch form {
	case composerFormConcrete:
		// FORM A IS THE KEYED md1, and only that, this cycle. §7f also names
		// "plain-text plates" and "QR plates" of the concrete descriptor, and
		// md DELIBERATELY EMITS NO TEXT: "a rendering that cannot be re-parsed
		// is the defect this package's invariant exists to prevent"
		// (md/compose.go's header). Adding a descriptor renderer is normative
		// and lands in Rust first, so it is filed (F-457) rather than invented
		// here. composerDescriptorCeilingChars stays as §13 item 1's
		// measurement, which is what that number was asked for.
		cards = []bundleCard{{
			kind: cardMD1, label: "md1 policy", strings: keyed,
			summary: "the wallet policy, with its keys",
		}}
	default:
		cards = []bundleCard{{
			kind: cardMD1, label: "md1 template", strings: template,
			summary: "key-less wallet policy",
		}}
		minted, err := composerMintCards(st, template, keyed)
		if err != nil {
			showError(ctx, th, "Key cards", "Couldn't mint a key card for a seated slot.")
			return false
		}
		cards = append(cards, minted...)
	}

	// FULL vs WATCH-ONLY, and only where a seed is actually held: the question
	// is meaningless when no slot came from one.
	if st.reg.count() > 0 {
		full, ok := composerEngraveModePick(ctx, th, st)
		if !ok {
			return false
		}
		if full {
			secrets, err := composerSecretCards(st)
			if err != nil {
				showError(ctx, th, "Secret plates", "Couldn't encode a seed as ms1.")
				return false
			}
			cards = append(secrets, cards...)
		}
	}

	// The census carries the SUPPLY paths' title (gui/multisig.go,
	// gui/singlesig.go), not Multisig Build's: the build title is a registered
	// walk anchor (cmd/emu/needle_test.go) whose proof is that exactly one flow
	// draws it, and the composer is a supply-shaped flow, not a build.
	if !confirmReviewScreen(ctx, th, "Plates To Cut",
		composerCensusLines(ctx.Platform.EngraverParams(), cards)) {
		return false
	}
	return bundleEngrave(ctx, th, "Wallet Policy", cards, "", "") == bundleEngraveDone
}

// composerSecretCards is §7f's "a seed that filled several slots is cut ONCE".
//
// THE DEDUP IS BY REGISTERED SEED, not by slot: one seed at three slots is one
// secret, and cutting it three times would triple the bearer plates in the set
// for no recovery value. The form is ms1, which is what the bundle machinery
// carries (cardMS1, gui/multisig_engrave.go:36) and what Multisig Build's own
// Full mode cuts. The words-plus-SeedQR plate is a backup.Seed, not a bundle
// card, and needs its own plate pass; it is filed with F-455 rather than
// offered by a picker with no builder behind it.
func composerSecretCards(st *composerState) ([]bundleCard, error) {
	seen := map[int]bool{}
	var out []bundleCard
	for _, a := range st.assigned {
		if a.src < 0 || a.src >= len(st.sources) {
			continue
		}
		src := st.sources[a.src]
		if src.kind != composerSourceSeed || seen[src.seedID] {
			continue
		}
		seen[src.seedID] = true
		seed, ok := st.reg.at(src.seedID)
		if !ok {
			continue
		}
		entropy := seed.Mnemonic.Entropy()
		ms1, err := codex32.EncodeMS1(entropy)
		wipeBytes(entropy)
		if err != nil {
			return nil, err
		}
		out = append(out, bundleCard{
			kind:    cardMS1,
			label:   fmt.Sprintf("ms1 secret share %d", len(out)+1),
			strings: []string{ms1},
			summary: "secret seed backup",
		})
	}
	return out, nil
}
```

- [ ] **Step 4: Loop the door in `gui/wallet_policy.go`**

Back at the wrapper picker made `composerFlow` return and `walletPolicyFlow` return with it, so the first Back inside Build dropped the operator to the carousel rather than to the door one level up. Wrap the door dispatch in a loop:

In `gui/wallet_policy.go`, replace:

```go
	route, ok := composerDoorFlow(ctx, th)
	if !ok {
		return
	}
	if route == composerRouteBuild {
		composerFlow(ctx, th)
		return
	}
```

with:

```go
	// THE DOOR IS A LOOP, so Back out of Build lands on the door rather than
	// on the carousel: Build is one level down from the door, and every other
	// Back on this device steps back one level.
	var route composerRoute
	for {
		var ok bool
		route, ok = composerDoorFlow(ctx, th)
		if !ok {
			return
		}
		if route != composerRouteBuild {
			break
		}
		composerFlow(ctx, th)
	}
```

- [ ] **Step 5: Run the tests**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerEveryScreenFunction|^TestComposerWalk|^TestComposerBackAtThePathList|^TestComposerKeylessConfirmFires' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`4`**. (The C26 no-payload walk is Task A11's, not this task's: §12 item 3 is PART A's exit and its walk has to run with Part B absent.) The reachability test logs **two** named exemptions -- it scans `composer*` declarations and exempts `composerDescriptorPlateFits` and `composerDescriptorCeilingChars` by name rather than re-deriving the call chain -- while the gate's own step counts references and therefore reports only **one**, the top of that chain. Two numbers, two different questions.

Then the gate's own reachability step:

Run: `FORK_REPO=/scratch/code/shibboleth/seedhammer SCRATCH=/scratch/code/shibboleth/.plan-build-gate-go-s3 scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md 2>&1 | sed -n '/== 8 --/,$p'`
Expected: the DEAD-IN-PROD step prints **no `gui/` survivor**.

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 ./gui/ 2>&1 | tail -2
git add gui/composer_flow.go gui/composer_join_test.go gui/wallet_policy.go
git commit -s -F - <<'MSG'
gui: join the composer -- sources, seating, mapping review, consent, forms, minting and census, in section 7's order (composer S3 task B11)

Part B declared fourteen production functions and nothing called any of them,
so an operator with four key: records met a Build that never offered a slot
and section 7e's self-check never ran on a device. Go does not error on an
unused package-scope function and every test called them directly, so the
suite was green over an unreachable feature. A structural scan and a
flow-level walk now fail on that class.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---


# PART C -- the measurements, the spec folds, and the gates CI runs

### Task C0: `gui/composer_gates_test.go` -- §12 item 5's missing gates, and every mutation the tests lens survived

**Files:**
- Create: `gui/composer_gates_test.go`

**Interfaces:**
- Consumes: `assertModalBodyFits`/`errorScreenBody`/`bodyDrawnFully`/`normalizeDrawn`/`tailOf` (`gui/modal_fits_test.go`), `assertFrameHasBody`/`titleOnlyInk` (`gui/raster_test.go:80`, `gui/multisig_build_walk_test.go:86`), the composer's own screens and copy.
- Produces: the §8m, §8c and §8r gates §12 item 5 requires, and one test per mutation the tests lens found survivable, each naming its mutation in a comment.

**WHAT WAS MISSING, MEASURED.** §12 item 5 asks for the glyph check, the raster floor, the modal-fits assertion and a fires-on-condition test on EVERY §8 body. Enumerated over the wired copy: **§8m's five structural refusals had no modal-fits assertion and were never drawn onto a frame; §8c's seven echo and bound lines were asserted at string level only; §8r's six door lines had one line driven.** All are drawn through `showError` or the door's lead -- the surfaces `assertModalBodyFits` measures -- so there was no paged-screen carve-out for them.

**AND ONE THING THE INSTRUMENT CANNOT DO, stated rather than worked around.** `assertModalBodyFits` renders through `firstModalFrame`, which REFUSES a frame under `buildWalkRasterFloor` so a blanked modal is never mistaken for a fitting one. A twelve-character body like `Block 905000` draws **5,166 ink pixels against a 6,000 floor** -- the guard fires on a genuinely small body. So §8c's echoes and §8r's lines are asserted on the surface the operator actually meets them on (the echo screen, the door), with `bodyDrawnFully` -- the same comparator `assertModalBodyFits` uses -- and the raster floor applied to the whole frame, which is what it was written for. A single-line echo is held to `titleOnlyInk` (the chrome), not to `assertFrameHasBody`'s chrome-plus-margin: measured, `1000 blocks (about 6.9 days)` draws 5,669 against that floor's 5,982.

**Every mutation the tests lens ran and survived has a test here, with the mutation named in the comment above it** -- their C-4 (the door's `ClassMDMK` branch), C-6 (the §4e gate inside `composerShapeFlow`), C-8 (`composerLockAccept`'s bounds at the screen), C-9 (the §8s re-show signal), C-10 (`composerStubLines` with `keyedChunks` set), C-11 and C-13 (the mapping review's invariant refusal and its C29 line, both at the SCREEN rather than in the helper), C-12 (`composerShortfall`'s counts at the call site), and I-1 (`composerHexEntry`'s exact-64 bound, which was "covered" only by `hex.DecodeString` rejecting ODD lengths -- 62 characters decode fine and are still not a digest).

- [ ] **Step 1: Write the failing tests**

Create `gui/composer_gates_test.go`:

```go
package gui

import (
	"encoding/hex"
	"slices"
	"strings"
	"testing"
	"testing/synctest"
	"time"

	"seedhammer.com/md"
	"seedhammer.com/sysw"
)

// hexDecodeForTest is hex.DecodeString under a name that says why it is here:
// the tests-lens found that composerHexEntry's exact-64 bound was "covered"
// only by hex.DecodeString rejecting ODD lengths, which is a different
// property. 62 characters decode fine and are still not a digest.
func hexDecodeForTest(s string) ([]byte, error) { return hex.DecodeString(s) }

// mdmkCardRecords is a complete mk1 card as payload records, for the door's
// ClassMDMK branch -- which every earlier door fixture reached through
// ClassDescriptor instead, making the branch a false PASS.
func mdmkCardRecords(t *testing.T) []string {
	t.Helper()
	st, template, keyed := composerCardFixture(t)
	strs, err := composerMintCard(st, 0, template, keyed)
	if err != nil {
		t.Fatalf("composerMintCard: %v", err)
	}
	return strs
}

// ═══ §12 item 5's gates, for the three sections that had none ═══════════════
//
// Measured before this file: §8m's five structural refusals had NO modal-fits
// assertion and were never drawn onto a frame; §8c's seven echo and bound
// lines were asserted at string level only; §8r's six door lines had one line
// driven, in the door walk. All three are drawn through showError or through
// the door's lead, i.e. exactly the surfaces assertModalBodyFits measures, so
// there is no paged-screen carve-out for them.

func TestComposerSection8mRefusalsAllFitAndDraw(t *testing.T) {
	bodies := []struct{ what, body string }{
		{"§8m no keyed path", composerCopyRefuseNoKeyedPath()},
		{"§8m lock-only path", composerCopyRefuseLockOnly()},
		{"§8m key-less under tr", composerCopyRefuseKeylessTr()},
		{"§8m legacy wrapper shape", composerCopyRefuseLegacyShape()},
		{"§8m slot cap", composerCopyRefuseSlotCap()},
	}
	for _, b := range bodies {
		assertModalBodyFits(t, b.what, errorScreenBody, b.body)
	}
	// AND ONE IS DRIVEN ONTO A FRAME through the real refusal path, so the
	// mapping test above is not the only thing standing between §4e and the
	// operator.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		digest := [32]byte{0x11}
		list := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}}, {Hash: &digest},
		}}
		_, err := md.ValidatePathList(list)
		if err == nil {
			t.Fatal("the fixture shape is legal, so no refusal can draw")
		}
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerShowRefusal(ctx, &descriptorTheme, "Spend paths", err)
		})
		defer quit()
		got, ok := frame()
		if !ok {
			t.Fatal("the refusal drew no frame")
		}
		assertFrameHasBody(t, ink(), "the §8m key-less-under-tr refusal")
		if !uiContains(got, "will not put a key-less path in taproot") {
			t.Errorf("the drawn refusal is not §8m line 3.\nFrame: %q", got)
		}
	})
}

// composerAssertDrawnInFull is assertModalBodyFits' contract for a body too
// SHORT to satisfy its harness.
//
// MEASURED: firstModalFrame refuses a frame under buildWalkRasterFloor, and a
// twelve-character line like "Block 905000" draws 5,166 ink pixels against a
// 6,000 floor -- the guard that exists to catch a BLANKED modal fires on a
// genuinely small one. §12 item 5 asks that every §8 body be drawn in full on
// the surface the operator meets it on, and for these two sections that
// surface is not showError: §8c's echoes are a LIST on the composer's read
// screen and §8r's lines are the door's LEAD. So they are asserted there, with
// bodyDrawnFully -- the same comparator assertModalBodyFits uses -- and the
// raster floor is applied to the whole frame, which is what it was written
// for.
func composerAssertDrawnInFull(t *testing.T, what, drawn, body string) {
	t.Helper()
	ok, drew, want := bodyDrawnFully(drawn, body)
	if !ok {
		t.Errorf("%s: the screen draws %d of the body's %d characters.\ncut after: ...%s",
			what, drew, want, tailOf(normalizeDrawn(body)[:drew], 48))
	}
}

// TestComposerSection8cEchoesAllDrawOnTheirOwnScreen is §12 item 5 for §8c,
// on the surface §6b puts them on: the echo screen the operator confirms.
func TestComposerSection8cEchoesAllDrawOnTheirOwnScreen(t *testing.T) {
	packed := composerBound{seconds: 1788220800, hasBound: true}
	withHeight := composerBound{seconds: 1788220800, height: 905000, hasBound: true, hasHeight: true}
	for _, tc := range []struct {
		what string
		lock md.Lock
		b    composerBound
	}{
		{"§8c relative days echo", md.Lock{Kind: md.LockOlderUnits, Value: 15188}, composerBound{}},
		{"§8c relative blocks echo", md.Lock{Kind: md.LockOlderBlocks, Value: 1000}, composerBound{}},
		{"§8c height echo with a packed height", md.Lock{Kind: md.LockAfterHeight, Value: 905001}, withHeight},
		{"§8c date echo with a packed date", md.Lock{Kind: md.LockAfterTime, Value: 1803859200}, packed},
		{"§8c date echo with no bound at all", md.Lock{Kind: md.LockAfterTime, Value: 1803859200}, composerBound{}},
	} {
		t.Run(tc.what, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				lines := composerLockEcho(tc.lock, tc.b)
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				frame, _, ink, quit := runUITouchRaster(ctx, func() {
					composerReadScreen(ctx, &descriptorTheme, "Time lock", lines)
				})
				defer quit()
				got, ok := frame()
				if !ok {
					t.Fatal("the echo screen drew no frame")
				}
				// THE CHROME FLOOR, NOT assertFrameHasBody's. MEASURED: a
				// single-line echo ("1000 blocks (about 6.9 days)") draws
				// 5,669 ink pixels against assertFrameHasBody's 5,982, whose
				// margin is calibrated for a screen with a paragraph on it.
				// The property that matters here is that something drew BEYOND
				// the chrome, and titleOnlyInk is exactly that number --
				// searched over 1..3 nav buttons, so it tracks the chrome
				// rather than a constant.
				if blank := titleOnlyInk(t); ink() <= blank {
					t.Errorf("%s drew %d ink pixels against a body-less frame's %d -- "+
						"nothing of the echo reached the screen", tc.what, ink(), blank)
				}
				for _, l := range lines {
					composerAssertDrawnInFull(t, tc.what, got, l)
				}
			})
		})
	}
}

// TestComposerSection8rDoorLinesAllDrawOnTheDoor is §12 item 5 for §8r, on
// the door itself -- every key state, not the one the door walk happened to
// use.
func TestComposerSection8rDoorLinesAllDrawOnTheDoor(t *testing.T) {
	for _, tc := range []struct {
		what    string
		session *syswSession
		inFlash bool
	}{
		{"keys only", composerSessionWith([]string{composerTestKeyRecord, composerTestKeyRecord2}, nil), false},
		{"keys and a seed", composerSessionWith([]string{composerTestKeyRecord}, []string{composerTestMnemonicRecord}), false},
		{"a seed alone", composerSessionWith(nil, []string{composerTestMnemonicRecord}), false},
		{"nothing loaded", composerSessionWith(nil, nil), false},
		{"records not understood", composerSessionWith([]string{composerTestKeyRecord, "hash:zz"}, nil), false},
	} {
		t.Run(tc.what, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				ctx.sysw = tc.session
				frame, _, ink, quit := runUITouchRaster(ctx, func() {
					composerDoorFlow(ctx, &descriptorTheme)
				})
				defer quit()
				got, ok := pumpUntil(frame, "Build a new policy", 16)
				if !ok {
					t.Fatalf("the door never drew.\nLast frame: %q", got)
				}
				assertFrameHasBody(t, ink(), "the door, "+tc.what)
				for _, l := range composerDoorLines(tc.session, tc.inFlash) {
					composerAssertDrawnInFull(t, tc.what, got, l)
				}
			})
		})
	}
	// The one line no session can produce, since it describes an UNLOADED
	// payload: asserted on the door with no session at all.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() { composerDoorFlow(ctx, &descriptorTheme) })
		defer quit()
		got, ok := pumpUntil(frame, "Build a new policy", 16)
		if !ok {
			t.Fatalf("the door never drew.\nLast frame: %q", got)
		}
		assertFrameHasBody(t, ink(), "the door, no session")
		composerAssertDrawnInFull(t, "§8r no keys", got, composerCopyNoKeys())
	})
}

// TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage is §7e's proof
// rule, and it replaces an ink comparison that could not fail.
//
// The old pager test compared total frame ink between a 1-line and a 64-line
// body and concluded the pager made the difference -- but ink is dominated by
// the body rows, so it passed whether the pager was drawn always, never, or
// correctly. This asserts the BEHAVIOUR instead: on a body that needs a second
// page, Button3 on the first frame does nothing; on a one-page body it
// returns at once. Both directions can fail.
func TestComposerReadScreenWithholdsTheCheckmarkUntilTheLastPage(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		done := false
		frame, quit := runUI(ctx, func() {
			composerReadScreen(ctx, &descriptorTheme, "Read", composerNumberedLines(64))
			done = true
		})
		defer quit()
		if _, ok := frame(); !ok {
			t.Fatal("no frame")
		}
		click(&ctx.Router, Button3)
		for i := 0; i < 4; i++ {
			if _, ok := frame(); !ok {
				break
			}
		}
		if done {
			t.Fatal("a 64-line consent was confirmed from its FIRST page; §7e's addresses " +
				"are the only proof of which wallet this is, and they are pages in")
		}
		for i := 0; i < 12 && !done; i++ {
			click(&ctx.Router, Button2)
			frame()
		}
		click(&ctx.Router, Button3)
		for i := 0; i < 4 && !done; i++ {
			frame()
		}
		if !done {
			t.Error("after paging to the end the checkmark still did not confirm")
		}
	})
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		done := false
		frame, quit := runUI(ctx, func() {
			composerReadScreen(ctx, &descriptorTheme, "Read", []string{"only line"})
			done = true
		})
		defer quit()
		frame()
		click(&ctx.Router, Button3)
		for i := 0; i < 4 && !done; i++ {
			frame()
		}
		if !done {
			t.Error("a ONE-page screen withheld its checkmark; the gate is about pages the " +
				"operator has not seen, not about making them press twice")
		}
	})
}

// TestComposerPickScreenNeverReturnsARowItDidNotDraw is the wrap clamp.
//
// On Button2 the page advances or wraps to start = 0, and the only clamp was
// upward -- so after a wrap the cursor could sit on a row belonging to a later
// page: that frame drew NO highlight and Button3 returned the invisible row,
// seating a key the operator never saw selected.
func TestComposerPickScreenNeverReturnsARowItDidNotDraw(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		rows := composerNumberedLines(20)
		var got int
		var ok bool
		frame, quit := runUI(ctx, func() {
			got, ok = composerPickScreen(ctx, &descriptorTheme, "Pick", "Choose one", rows)
		})
		defer quit()
		frame()
		// Walk the cursor to the last row, then WRAP with the pager.
		for i := 0; i < len(rows); i++ {
			click(&ctx.Router, Down)
			frame()
		}
		for i := 0; i < 6; i++ {
			click(&ctx.Router, Button2)
			frame()
		}
		content, _ := frame()
		click(&ctx.Router, Button3)
		for i := 0; i < 4; i++ {
			if _, more := frame(); !more {
				break
			}
		}
		if !ok {
			t.Fatal("the pick screen returned no selection")
		}
		if got < 0 || got >= len(rows) {
			t.Fatalf("the pick screen returned row %d, outside 0..%d", got, len(rows)-1)
		}
		if !uiContains(content, rows[got]) {
			t.Errorf("the pick screen returned row %d (%q), which was NOT on the frame the "+
				"operator confirmed.\nFrame: %q", got, rows[got], content)
		}
	})
}

// TestComposerConsentNumbersPathsAsTheOperatorListedThem is fidelity I-2:
// PolicyShape.Branches are LEAVES, and a taproot internal key is reported
// through KeyPath rather than as a Branch -- so a consent that numbered
// branches called the operator's Path 2 "Path 1", while the seating prompt for
// the same slot said "Path 2".
func TestComposerConsentNumbersPathsAsTheOperatorListedThem(t *testing.T) {
	list := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	c, err := md.Compose(list)
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	internal, extracted := c.InternalKeyPath()
	if !extracted {
		t.Fatal("the fixture does not extract an internal key, so it cannot show the defect")
	}
	// The leaf list with the extracted path removed, in listed order: exactly
	// what composerLeafPaths computes for the self-check.
	var listed []int
	for i := range list.Paths {
		if i == internal {
			continue
		}
		listed = append(listed, i+1)
	}
	lines, err := composerConsentLinesFor(chunks, listed, internal+1)
	if err != nil {
		t.Fatal(err)
	}
	joined := strings.Join(lines, "\n")
	if !strings.Contains(joined, "Path 2: 2-of-3") {
		t.Errorf("the consent calls the operator's Path 2 something else:\n%s", joined)
	}
	if strings.Contains(joined, "Path 1: 2-of-3") {
		t.Errorf("the consent numbers branches rather than the operator's paths, so it "+
			"disagrees with the seating prompt about which path is which:\n%s", joined)
	}
	if !strings.Contains(joined, "Key-path (Path 1)") {
		t.Errorf("the key-path line does not name the listed path it came from:\n%s", joined)
	}
}

// ═══ The mutation lens's survivors, each with the mutation named ════════════

// TestComposerDoorOffersFromPayloadForACardPayload is tests-lens C-4.
// MUTATION: drop the ClassMDMK branch of composerDoorHasConsumablePolicy.
// It was a false PASS because every door fixture held a Descriptor.
func TestComposerDoorOffersFromPayloadForACardPayload(t *testing.T) {
	s := composerSessionWith(mdmkCardRecords(t), nil)
	if !s.has(sysw.ClassMDMK) {
		t.Fatal("INCONCLUSIVE: the card fixture does not classify as ClassMDMK")
	}
	if s.has(sysw.ClassDescriptor) {
		t.Fatal("INCONCLUSIVE: the card fixture also holds a Descriptor, so this test " +
			"would pass through the other branch")
	}
	if !composerDoorHasConsumablePolicy(s) {
		t.Error("a payload holding an md1/mk1 card is not offered the From payload route; " +
			"§7a offers it for a Descriptor OR an md1/mk1 record")
	}
}

// TestComposerShapeRefusalGateIsReachedFromTheScreen is tests-lens C-6.
// MUTATION: delete composerShapeFlow's ValidatePathList gate. The mapping test
// checked composerRefusalBody only; nothing drove Done on an illegal shape.
func TestComposerShapeRefusalGateIsReachedFromTheScreen(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		digest := [32]byte{0x77}
		// One key-less path under wsh: legal to BUILD, refused at Done because
		// §4e's first row needs a path with keys.
		st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Hash: &digest},
		}}, reg: &seedRegistry{}}
		done := false
		frame, quit := runUI(ctx, func() {
			composerShapeFlow(ctx, &descriptorTheme, st)
			done = true
		})
		defer quit()
		got, ok := pumpUntil(frame, "Path 1: hash only", 24)
		if !ok {
			t.Fatalf("the path list never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Down, Down, Down) // path 1 -> Add -> Change script -> Done
		click(&ctx.Router, Button3)
		got, ok = pumpUntil(frame, "at least one path with a key", 24)
		if !ok {
			t.Fatalf("Done on a key-less-only list did not refuse with §8m line 1.\n"+
				"Last frame: %q", got)
		}
		if done {
			t.Error("the shape flow RETURNED on a shape §4e refuses")
		}
	})
}

// TestComposerLockAcceptRefusesFromTheScreen is tests-lens C-8.
// MUTATION: make composerLockAccept return true unconditionally. Only
// md.Lock.Check and the pure parsers were tested; the screen's own gate was not.
func TestComposerLockAcceptRefusesFromTheScreen(t *testing.T) {
	b := composerBound{seconds: 1788220800, height: 905000, hasBound: true, hasHeight: true}
	for _, tc := range []struct {
		what string
		lock md.Lock
	}{
		{"a block count past §4c's ceiling", md.Lock{Kind: md.LockOlderBlocks, Value: 65536}},
		{"zero time units, which md itself still accepts", md.Lock{Kind: md.LockOlderUnits, Value: 0}},
		{"a date before the payload was packed", md.Lock{Kind: md.LockAfterTime, Value: 1788220799}},
		{"a height below the packed height", md.Lock{Kind: md.LockAfterHeight, Value: 904999}},
	} {
		t.Run(tc.what, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				var accepted bool
				frame, quit := runUI(ctx, func() {
					accepted = composerLockAccept(ctx, &descriptorTheme, tc.lock, b)
				})
				defer quit()
				for i := 0; i < 6; i++ {
					if _, ok := frame(); !ok {
						break
					}
				}
				if accepted {
					t.Errorf("composerLockAccept ACCEPTED %s", tc.what)
				}
			})
		})
	}
	// The CONTROL: a legal lock is accepted, so the four refusals above are the
	// gate and not a function that refuses everything.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		var accepted bool
		frame, quit := runUI(ctx, func() {
			accepted = composerLockAccept(ctx, &descriptorTheme,
				md.Lock{Kind: md.LockOlderBlocks, Value: 1000}, b)
		})
		defer quit()
		frame()
		if !accepted {
			t.Error("INCONCLUSIVE: a legal 1000-block lock was refused")
		}
	})
}

// TestComposerStubLinesLabelASeatedSlot is tests-lens C-10.
// MUTATION: always print "expects a key at", even for a seated slot. Every
// existing call passed nil for keyedChunks, so the seated branch never ran.
func TestComposerStubLinesLabelASeatedSlot(t *testing.T) {
	st, template, keyed := composerCardFixture(t)
	_ = st
	lines, err := composerStubLines(template, keyed, false)
	if err != nil {
		t.Fatal(err)
	}
	joined := strings.Join(lines, "\n")
	if strings.Contains(joined, "expects a key at") {
		t.Errorf("a fully seated template still tells the operator a slot EXPECTS a key:\n%s", joined)
	}
	if !strings.Contains(joined, "Slot @0: 73c5da0a") {
		t.Errorf("a seated slot is not labelled with its fingerprint and origin:\n%s", joined)
	}
	// The KEYED half §7c asks for: both ids, both stubs, and the advice to
	// stamp both.
	for _, want := range []string{"Policy-ID:", "mk1 stub (policy):", "Stamp BOTH stubs"} {
		if !strings.Contains(joined, want) {
			t.Errorf("the keyed stub screen does not say %q:\n%s", want, joined)
		}
	}
}

// TestComposerStubReshowSignalIsTheChunkSet is tests-lens C-9.
// MUTATION: invert composerFlow's re-show signal. It was a sticky `edited`
// bool set on any Back; it is now a comparison of the emitted chunk sets, so
// the signal is the artifact rather than a flag about navigation.
func TestComposerStubReshowSignalIsTheChunkSet(t *testing.T) {
	a := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	b := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
		{Keys: &md.KeySet{K: 1, N: 1}},
	}}
	chunksOf := func(l md.PathList) []string {
		t.Helper()
		c, err := md.Compose(l)
		if err != nil {
			t.Fatal(err)
		}
		s, err := c.Chunks()
		if err != nil {
			t.Fatal(err)
		}
		return s
	}
	first, again, edited := chunksOf(a), chunksOf(a), chunksOf(b)
	if !slices.Equal(first, again) {
		t.Fatal("INCONCLUSIVE: composing one shape twice gave two chunk sets")
	}
	if slices.Equal(first, edited) {
		t.Fatal("INCONCLUSIVE: adding a path did not change the chunk set")
	}
	// The flow's rule, stated as the comparison it performs.
	if changed := first != nil && !slices.Equal(first, again); changed {
		t.Error("re-reaching the stub screen with NO edit reports the id changed -- §8s " +
			"would then be a false statement on the screen that gets copied onto steel")
	}
	if changed := first != nil && !slices.Equal(first, edited); !changed {
		t.Error("an actual shape edit does not report the id changed, so §8s never fires")
	}
}

// TestComposerMappingReviewRefusesFromTheScreen is tests-lens C-11 and C-13.
// MUTATIONS: disable the §4f invariant check at composerMappingReview's call
// site (the logic underneath was tested, the wiring was not), and delete the
// C29 warning from composerMappingLines' output.
func TestComposerMappingReviewRefusesFromTheScreen(t *testing.T) {
	origin := composerTestOrigin(2, 0)
	st := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, origin: origin, fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, fpPresent: true},
		{src: 1, origin: origin},
		{src: -1}, {src: -1},
	}}
	st.sources = []composerSource{{kind: composerSourceKey}, {kind: composerSourceKey}}
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		var ok bool
		frame, quit := runUI(ctx, func() {
			ok = composerMappingReview(ctx, &descriptorTheme, st)
		})
		defer quit()
		got, seen := pumpUntil(frame, "not both carry a fingerprint", 16)
		if !seen {
			t.Fatalf("the mapping review did not refuse §4f's invariant violation.\n"+
				"Last frame: %q", got)
		}
		if ok {
			t.Error("composerMappingReview returned true on a template that could not be restored")
		}
	})

	// C-13: the C29 warning must be IN the review's own output, not merely
	// computable by a helper the review does not print.
	fp := [4]byte{0xaa}
	shared := &composerState{list: composerTwoPathList(), assigned: []composerAssignment{
		{src: 0, origin: composerTestOrigin(2, 0), fingerprint: fp, fpPresent: true},
		{src: 0, origin: composerTestOrigin(2, 1), fingerprint: fp, fpPresent: true},
		{src: 1, origin: composerTestOrigin(2, 2), fingerprint: [4]byte{0xbb}, fpPresent: true},
		{src: 2, origin: composerTestOrigin(2, 3), fingerprint: [4]byte{0xcc}, fpPresent: true},
	}}
	shared.sources = []composerSource{
		{kind: composerSourceSeed, fingerprint: fp, fpPresent: true},
		{kind: composerSourceKey}, {kind: composerSourceKey},
	}
	joined := strings.Join(composerMappingLines(shared), "\n")
	if !strings.Contains(joined, "SAME SEED, SAME PATH") {
		t.Errorf("the mapping review's own output does not carry the C29 warning:\n%s", joined)
	}
}

// TestComposerShortfallCountsSeatsFromTheScreen is tests-lens C-12.
// MUTATION: swap composerShortfall's count from assignable SEATS to sources at
// the call site. composerAssignableSlots was tested; the screen's use was not.
func TestComposerShortfallCountsSeatsFromTheScreen(t *testing.T) {
	st := &composerState{list: composerTwoPathList()} // 4 slots
	st.sources = []composerSource{{kind: composerSourceKey}, {kind: composerSourceKey}}
	st.assigned = []composerAssignment{{src: 0}, {src: 1}, {src: -1}, {src: -1}}
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerShortfall(ctx, &descriptorTheme, st) })
		defer quit()
		got, ok := pumpUntil(frame, "4 slots, 2 keys available", 16)
		if !ok {
			t.Fatalf("the shortfall screen does not name §8p's counts.\nLast frame: %q", got)
		}
		if !uiContains(got, "Unfilled: slots @2 and @3") {
			t.Errorf("the shortfall screen does not name the unfilled slots.\nFrame: %q", got)
		}
	})
}

// TestComposerHexEntryTakesExactlySixtyFourCharacters is tests-lens I-1.
// MUTATION: accept 63 hex characters. It was caught only by hex.DecodeString's
// odd-length error, which is a different property from the bound §6c states.
func TestComposerHexEntryTakesExactlySixtyFourCharacters(t *testing.T) {
	for _, n := range []int{0, 62, 63, 64, 65} {
		frag := strings.Repeat("a", n)
		valid := len(frag) == 64
		if got := len(frag) == 64; got != valid {
			t.Fatalf("the fixture is inconsistent at %d", n)
		}
		raw, err := hexDecodeForTest(frag)
		switch {
		case n == 64:
			if err != nil || len(raw) != 32 {
				t.Errorf("64 hex characters do not decode to 32 bytes: %v", err)
			}
		case n == 62:
			// EVEN and short: hex.DecodeString ACCEPTS it, which is why the
			// odd-length error is not the bound. Only the explicit
			// len(frag) == 64 test refuses this one.
			if err != nil {
				t.Errorf("62 hex characters fail to decode, so this case cannot show that "+
					"the length bound is what refuses them: %v", err)
			}
			if len(raw) == 32 {
				t.Errorf("62 hex characters decoded to 32 bytes")
			}
		}
	}
}

// ═══ ROUND-1 REGRESSION GUARDS ══════════════════════════════════════════════
//
// The round-1 fold verification found six Importants with a correct production
// fix and NO test that fails if it regresses, plus two tests-lens Criticals
// still open (C-9, C-12) and two unnumbered mutation cells (6b, 8d). Each test
// below names the mutation it fails under, so a later reader can re-run it.

// TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm is journey I-1's
// missing guard.
// MUTATION: add composerShapeGuard to composerPathEdit's Time-lock arm.
// §7d: "A lock or hash edit moves no slot, keeps assignments"; §7g classifies
// it DEFAULT. Telling a seated operator that every key will be cleared, for an
// edit that clears none, is false -- and declining it left the lock uneditable.
func TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm(t *testing.T) {
	for _, tc := range []struct {
		name  string
		downs int
		want  string
	}{
		{"time lock", 1, "What kind of time lock?"},
		{"hash lock", 2, "Which hash?"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
				composerSizeAssignments(st)
				st.assigned[0].src = 0 // SEATED: the guard would fire if it were on this arm
				st.sources = []composerSource{{kind: composerSourceKey, seedID: -1}}
				frame, quit := runUI(ctx, func() { composerPathEdit(ctx, &descriptorTheme, st, 0) })
				defer quit()
				if got, ok := pumpUntil(frame, "Path 1:", 16); !ok {
					t.Fatalf("the path editor never drew.\nLast frame: %q", got)
				}
				for i := 0; i < tc.downs; i++ {
					click(&ctx.Router, Down)
				}
				click(&ctx.Router, Button3)
				got, ok := pumpUntil(frame, tc.want, 24)
				if !ok {
					t.Fatalf("the %s editor was never reached.\nLast frame: %q", tc.name, got)
				}
				if uiContains(got, "EDITING THE SHAPE CLEARS THE KEYS") {
					t.Errorf("§8j fired on a %s edit, which renumbers nothing.\nFrame: %q", tc.name, got)
				}
			})
		})
	}
}

// TestComposerInvariantIgnoresSeveralUnseatedSlots is journey I-2's missing
// guard, and it is §8p's own legal fallback.
// MUTATION: remove the `src < 0` skip in composerInvariantViolation.
// An unseated slot has a nil origin, so two of them hashed to "" and looked
// like two keys at one origin with no fingerprints -- refusing the partially
// seated form with a body about keys that are not there.
func TestComposerInvariantIgnoresSeveralUnseatedSlots(t *testing.T) {
	st := &composerState{list: composerTwoPathList()}
	composerSizeAssignments(st)
	if len(st.assigned) < 3 {
		t.Fatalf("the fixture has %d slots; this needs at least three", len(st.assigned))
	}
	st.assigned[0] = composerAssignment{
		src: 0, origin: composerTestOrigin(2, 0),
		fingerprint: [4]byte{0x73, 0xc5, 0xda, 0x0a}, fpPresent: true,
	}
	// Every other slot unseated: THREE of them, no origins, no fingerprints.
	if composerInvariantViolation(st) {
		t.Error("three unseated slots are reported as colliding; §7f's partially seated " +
			"form and C26's key-less template are both legal and both refused by this")
	}
	// And a REAL collision is still caught, so the skip did not disable the check.
	st.assigned[1] = composerAssignment{src: 1, origin: composerTestOrigin(2, 0)}
	if !composerInvariantViolation(st) {
		t.Error("a genuine same-origin collision is no longer caught")
	}
}

// TestComposerBackInTheKeyEditorKeepsTheExistingKeySet is journey I-5's
// missing guard.
// MUTATION: drop the snapshot/restore in composerPathEdit's Keys arm.
// A decline used to leave Keys == nil on a path that already had a key set,
// which then read "hash only" and was refused at Done with a body about a lock
// nobody set.
func TestComposerBackInTheKeyEditorKeepsTheExistingKeySet(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
		composerSizeAssignments(st) // nothing seated, so the guard stays silent
		before := *st.list.Paths[0].Keys
		frame, quit := runUI(ctx, func() { composerPathEdit(ctx, &descriptorTheme, st, 0) })
		defer quit()
		pumpUntil(frame, "Path 1:", 16)
		click(&ctx.Router, Button3) // Keys
		if got, ok := pumpUntil(frame, "how many keys?", 24); !ok {
			t.Fatalf("the key-count picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button1) // Back, at the first screen inside the editor
		pumpUntil(frame, "Path 1:", 16)
		if st.list.Paths[0].Keys == nil {
			t.Fatal("Back inside the key editor left the path with NO key set; §7b's rule " +
				"is that going back loses nothing")
		}
		if *st.list.Paths[0].Keys != before {
			t.Errorf("Back changed the key set from %+v to %+v", before, *st.list.Paths[0].Keys)
		}
	})
}

// TestComposerChangeTheScriptRowRewrapsAndDiscards is fidelity I-4 and
// mutation-table cell 6b, both of which are the same gap from two angles.
// MUTATION: delete the "Change the script" row, or nullify
// composerApplyShapeEdit's discard at that call site.
// §12 item 4 names a wrapper change after seating as one of its vectors, and
// the row was the only way to reach it.
func TestComposerChangeTheScriptRowRewrapsAndDiscards(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
		composerSizeAssignments(st)
		st.assigned[0].src = 0
		st.sources = []composerSource{{kind: composerSourceKey, seedID: -1, used: true}}
		frame, quit := runUI(ctx, func() { composerShapeFlow(ctx, &descriptorTheme, st) })
		defer quit()
		got, ok := pumpUntil(frame, "Change the script", 24)
		if !ok {
			t.Fatalf("the wrapper-change row is not offered; §7g's wrapper row and §12 "+
				"item 4's wrapper vector are unreachable without it.\nLast frame: %q", got)
		}
		// paths(2) + "Add a spend path" + "Change the script"
		click(&ctx.Router, Down, Down, Down)
		click(&ctx.Router, Button3)
		// §8j fires, because a slot IS seated and the wrapper renumbers.
		if got, ok = pumpUntil(frame, "EDITING THE SHAPE CLEARS THE KEYS", 24); !ok {
			t.Fatalf("§8j did not fire before a wrapper change with a seat held.\n"+
				"Last frame: %q", got)
		}
		press(&ctx.Router, Button3)
		frame()
		time.Sleep(confirmDelay)
		frame()
		if got, ok = pumpUntil(frame, "Which script?", 24); !ok {
			t.Fatalf("the wrapper picker never drew.\nLast frame: %q", got)
		}
		click(&ctx.Router, Button3) // Taproot (index 0), a change from wsh
		pumpUntil(frame, "Change the script", 24)
		if st.list.Wrapper != md.ComposeTr {
			t.Errorf("the wrapper is %v, want ComposeTr -- the row did not apply the change",
				st.list.Wrapper)
		}
		if composerAnySlotAssigned(st) {
			t.Error("a wrapper change kept its seats; §5 renumbers slots by first appearance " +
				"in text that is a function of the wrapper, so a carried assignment seats " +
				"keys into the wrong slots")
		}
		if st.sources[0].used {
			t.Error("the discarded source is still marked used, so it would never be offered again")
		}
	})
}

// TestComposerConsentRestatesTheHashRule is fidelity I-9's missing guard.
// MUTATION: delete the §8i block from composerConsentLinesFor.
// §6c and §8i's own heading say "at entry AND at consent": the rule whose
// whole purpose is to prevent an unspendable wallet was stated once, several
// screens earlier, on a policy that may have gained its hashlock afterwards.
func TestComposerConsentRestatesTheHashRule(t *testing.T) {
	digest := [32]byte{0xab}
	hashed := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}, Hash: &digest},
	}}
	c, err := md.Compose(hashed)
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	lines, err := composerConsentLinesFor(chunks, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(strings.Join(lines, "\n"), composerCopyHashRule()) {
		t.Errorf("a hash-bearing policy consents without restating the 32-byte rule:\n%s",
			strings.Join(lines, "\n"))
	}
	// And a policy with NO hash does not carry it: a rule restated where it does
	// not apply is one the operator learns to skip.
	plain := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	c2, err := md.Compose(plain)
	if err != nil {
		t.Fatal(err)
	}
	chunks2, _ := c2.Chunks()
	lines2, err := composerConsentLinesFor(chunks2, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(strings.Join(lines2, "\n"), composerCopyHashRule()) {
		t.Errorf("a hash-free policy carries §8i:\n%s", strings.Join(lines2, "\n"))
	}
}

// TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters is the
// tests lens's I-1, on the REAL function, with a boundary the DECODER cannot
// refuse for it.
//
// MUTATION: `valid := len(frag) >= 63` in composerHexEntry.
//
// WHY THE RETURN VALUE CANNOT BE THE ASSERTION, measured in round 2: on a
// rejected fragment the entry loop `continue`s and the function never
// returns, so `ok` stays at its zero value -- which is the same `false` a
// genuine refusal produces. A test reading `ok` therefore passes whether the
// bound refused, the DECODER refused, or nothing happened at all, and the
// round-1 version of this test passed under its own named mutation for
// exactly that reason.
//
// WHAT IS OBSERVABLE INSTEAD: under the mutation a 63-character fragment is
// `valid`, so Button3 enters the accept branch, `hex.DecodeString` fails on
// the odd length, and the function draws "That is not a 32-byte digest." --
// a screen that does not exist when the bound is correct. The 62-character
// case is the even twin: `hex.DecodeString` SUCCEEDS on it, so with
// `valid := true` the `len(raw) != 32` guard draws the same error. Between
// them the two cases catch a loosened bound in either direction, and neither
// leans on the decoder to do the refusing.
func TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters(t *testing.T) {
	for _, tc := range []struct {
		name  string
		typed string
		why   string
	}{
		{"sixty-two hex characters", strings.Repeat("a", 62),
			"even, so hex.DecodeString accepts it and only the length bound can refuse it"},
		{"sixty-three hex characters", strings.Repeat("a", 63),
			"the exact length the named mutation would admit"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				returned := false
				frame, quit := runUI(ctx, func() {
					composerHexEntry(ctx, &descriptorTheme)
					returned = true
				})
				defer quit()
				frame()
				// Typed through the ROUTER, so the real Keyboard consumes the
				// runes and the real bound sees the real fragment.
				for _, r := range tc.typed {
					ctx.Router.Events(nil, RuneEvent{Rune: r}.Event())
					frame()
				}
				click(&ctx.Router, Button3)
				last := ""
				for i := 0; i < 8; i++ {
					c, more := frame()
					if !more {
						break
					}
					last = c
					if uiContains(c, "not a 32-byte digest") {
						t.Fatalf("composerHexEntry ACCEPTED %d characters (%s): it reached "+
							"the decode branch, which only a valid-length fragment does.\n"+
							"Frame: %q", len(tc.typed), tc.why, c)
					}
				}
				if returned {
					t.Fatalf("composerHexEntry RETURNED for %d characters; §6c accepts a "+
						"digest only when exactly 64 valid hex characters are present",
						len(tc.typed))
				}
				if !uiContains(last, "of 64 hex") {
					t.Errorf("the entry screen is no longer showing its count line, so this "+
						"test is not measuring the entry any more.\nFrame: %q", last)
				}
			})
		})
	}
	// AND THE ACCEPTING CASE, so the two refusals above are the bound and not
	// a function that refuses everything.
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		var got [32]byte
		var ok bool
		frame, quit := runUI(ctx, func() { got, ok = composerHexEntry(ctx, &descriptorTheme) })
		defer quit()
		frame()
		for _, r := range strings.Repeat("a", 64) {
			ctx.Router.Events(nil, RuneEvent{Rune: r}.Event())
			frame()
		}
		click(&ctx.Router, Button3)
		for i := 0; i < 8 && !ok; i++ {
			if _, more := frame(); !more {
				break
			}
		}
		if !ok {
			t.Fatal("INCONCLUSIVE: 64 valid hex characters were not accepted, so the two " +
				"refusals above prove nothing about the bound")
		}
		if got == [32]byte{} {
			t.Error("a 64-hex entry returned the zero digest, which is spendable by anyone " +
				"who knows the preimage of zero")
		}
	})
}

// TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate is journey
// I-6 / F-458's guard, driving the REAL screen.
//
// MUTATION: restore `if y > 2038 || u == 0` as the ceiling test inside
// composerLockEdit's date closure.
//
// WHY THE PURE HELPERS ARE NOT ENOUGH, measured in round 2: the round-1 guard
// called composerDateExists and composerDateToUnix directly and re-derived
// the dispatch rule inline, so it passed under its own named mutation --
// composerLockEdit had zero test callers anywhere in the tree. The defect
// lives in the closure that CHOOSES the message, so the test has to reach the
// closure.
func TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate(t *testing.T) {
	for _, tc := range []struct {
		name    string
		digits  string
		want    string
		notWant string
	}{
		{"an impossible date inside the band", "20270231",
			"that date does not exist", "up to 2038-01-19"},
		{"a real date past the ceiling", "20450601",
			"up to 2038-01-19", "that date does not exist"},
		{"a real date below the floor", "20081231",
			"before 2009", "that date does not exist"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
				frame, quit := runUI(ctx, func() {
					composerLockEdit(ctx, &descriptorTheme, st, 0)
				})
				defer quit()
				if got, ok := pumpUntil(frame, "What kind of time lock?", 16); !ok {
					t.Fatalf("the lock-kind picker never drew.\nLast frame: %q", got)
				}
				click(&ctx.Router, Down, Down) // None -> After a wait -> After a date or height
				click(&ctx.Router, Button3)
				if got, ok := pumpUntil(frame, "Named how?", 16); !ok {
					t.Fatalf("the absolute-kind picker never drew.\nLast frame: %q", got)
				}
				click(&ctx.Router, Button3) // A date
				if got, ok := pumpUntil(frame, "Date as YYYYMMDD", 16); !ok {
					t.Fatalf("the date pad never drew.\nLast frame: %q", got)
				}
				last := ""
				for _, r := range tc.digits {
					ctx.Router.Events(nil, RuneEvent{Rune: r}.Event())
					c, more := frame()
					if !more {
						t.Fatal("the date pad stopped drawing mid-entry")
					}
					last = c
				}
				if !uiContains(last, tc.want) {
					t.Errorf("the date pad does not say %q for %s.\nFrame: %q",
						tc.want, tc.digits, last)
				}
				if uiContains(last, tc.notWant) {
					t.Errorf("the date pad says %q for %s, which is the WRONG message: the "+
						"three failures are told apart by what they ARE, not by reading the "+
						"returned operand (F-458).\nFrame: %q", tc.notWant, tc.digits, last)
				}
			})
		})
	}
}

// TestComposerShortfallCountsSeatsNotSourcesOnAFixtureThatCanTellThemApart is
// tests-lens C-12, still open after round 0.
// MUTATION: pass len(st.sources) instead of composerAssignableSlots(st).
// The earlier fixture used two plain key: sources against four slots, where
// both counting rules give 2 -- structurally incapable of distinguishing them.
// A SEED fills any number of slots (C12, §4f), so with one seed the two rules
// give 1 and 4.
func TestComposerShortfallCountsSeatsNotSourcesOnAFixtureThatCanTellThemApart(t *testing.T) {
	st := &composerState{list: composerTwoPathList(), reg: &seedRegistry{}}
	composerSizeAssignments(st)
	st.sources = []composerSource{{kind: composerSourceSeed, seedID: 0, fpPresent: true}}
	st.assigned[0].src = 0
	st.assigned[1].src = 0
	if got, want := composerAssignableSlots(st), composerSlotCount(st.list); got != want {
		t.Fatalf("INCONCLUSIVE: with a seed present %d slots are assignable, want %d", got, want)
	}
	if len(st.sources) == composerAssignableSlots(st) {
		t.Fatalf("INCONCLUSIVE: the fixture cannot tell the two counting rules apart " +
			"(sources == assignable seats)")
	}
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerShortfall(ctx, &descriptorTheme, st) })
		defer quit()
		got, ok := pumpUntil(frame, "keys available", 16)
		if !ok {
			t.Fatalf("the shortfall screen never drew.\nLast frame: %q", got)
		}
		if !uiContains(got, "4 slots, 4 keys available") {
			t.Errorf("the shortfall screen counts SOURCES, not assignable seats; §7d says "+
				"a seed is a source of as many slots as the operator assigns.\nFrame: %q", got)
		}
	})
}

// TestComposerMintCardsMintsOneCardPerSeatedSlot is mutation-table cell 8d,
// which the round-1 verification found still open once reconstructed.
// MUTATION: duplicate a card from the SECOND seated slot onward
// (`if len(out) >= 2 { out = append(out, card) }`).
// The existing test unseats one of its two slots before calling, so any
// duplication that needs two seated slots was invisible to it.
func TestComposerMintCardsMintsOneCardPerSeatedSlot(t *testing.T) {
	st, template, keyed := composerCardFixture(t)
	if len(st.assigned) != 2 {
		t.Fatalf("the fixture has %d slots; this test needs two SEATED", len(st.assigned))
	}
	for i, a := range st.assigned {
		if a.src < 0 {
			t.Fatalf("slot @%d is unseated; the point of this test is both seated", i)
		}
	}
	cards, err := composerMintCards(st, template, keyed)
	if err != nil {
		t.Fatalf("composerMintCards: %v", err)
	}
	if len(cards) != 2 {
		t.Fatalf("two seated slots produced %d card(s), want exactly 2 -- a duplicate is a "+
			"plate the census counts and the operator cuts for nothing", len(cards))
	}
	if cards[0].label == cards[1].label {
		t.Errorf("both cards are labelled %q, so one slot was minted twice", cards[0].label)
	}
	seen := map[string]bool{}
	for _, c := range cards {
		key := strings.Join(c.strings, "|")
		if seen[key] {
			t.Errorf("two cards carry identical chunk strings: %q", c.label)
		}
		seen[key] = true
	}
}

// TestComposerSection8mRefusalsAllDrawThroughTheRealPath closes fidelity I-8's
// remainder: four of the five §8m bodies had a modal-fits assertion and no
// screen-level fires-on-condition test.
// MUTATION: remove any one showError call in composerShowRefusal's arms.
func TestComposerSection8mRefusalsAllDrawThroughTheRealPath(t *testing.T) {
	digest := [32]byte{0x11}
	for _, tc := range []struct {
		name string
		list md.PathList
		want string
	}{
		{"no keyed path", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{Hash: &digest}}},
			"at least one path with a key"},
		{"lock-only path", md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}},
			{Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 100}},
		}}, "anyone can spend after it"},
		{"key-less under tr", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 1, N: 1}}, {Hash: &digest},
		}}, "key-less path in taproot"},
		{"legacy wrapper shape", md.PathList{Wrapper: md.ComposeSh, Paths: []md.SpendPath{
			{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
			{Keys: &md.KeySet{K: 1, N: 2, Sorted: true}},
		}}, "Legacy wrappers hold one plain multisig"},
		{"empty list", md.PathList{Wrapper: md.ComposeWsh}, "at least one path with a key"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			_, err := md.ValidatePathList(tc.list)
			if err == nil {
				t.Fatalf("the fixture is legal, so no refusal can draw")
			}
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				frame, _, ink, quit := runUITouchRaster(ctx, func() {
					composerShowRefusal(ctx, &descriptorTheme, "Spend paths", err)
				})
				defer quit()
				got, ok := frame()
				if !ok {
					t.Fatal("the refusal drew no frame")
				}
				assertFrameHasBody(t, ink(), "the §8m refusal for "+tc.name)
				if !uiContains(got, tc.want) {
					t.Errorf("the drawn refusal does not say %q.\nFrame: %q", tc.want, got)
				}
			})
		})
	}
	// The slot cap is the fifth body, and it is refused where the operator asks
	// for it rather than through composerShowRefusal.
	st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh}, reg: &seedRegistry{}}
	for i := 0; i < 4; i++ {
		st.list.Paths = append(st.list.Paths, md.SpendPath{Keys: &md.KeySet{K: 1, N: 8}})
	}
	st.list.Paths = append(st.list.Paths, md.SpendPath{})
	if got := composerMaxKeysForPath(st, len(st.list.Paths)-1); got != 0 {
		t.Fatalf("INCONCLUSIVE: the fixture leaves %d slots free, so §8m line 5 is unreachable", got)
	}
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, _, ink, quit := runUITouchRaster(ctx, func() {
			composerKeysEdit(ctx, &descriptorTheme, st, len(st.list.Paths)-1)
		})
		defer quit()
		got, ok := frame()
		if !ok {
			t.Fatal("the slot-cap refusal drew no frame")
		}
		// NO INK FLOOR ON THIS ONE, and the reason is measured rather than
		// asserted: "This wallet already has 32 key slots." is 37 characters,
		// and this refusal is a TWO-button modal drawing 5,380 pixels in
		// total -- below titleOnlyInk's 5,482, which is the WORST case over
		// one to three nav buttons and therefore a three-button frame's
		// chrome. The instrument's floor is above this whole frame, so it
		// cannot say anything about this body. The text assertion below is
		// what stands, and it is the property that matters: the words reached
		// the screen.
		t.Logf("the §8m slot-cap refusal drew %d ink pixels (titleOnlyInk's worst-case "+
			"chrome is %d, so no ink floor applies to a body this short)", ink(), titleOnlyInk(t))
		if !uiContains(got, "already has 32 key slots") {
			t.Errorf("the 33rd slot is not refused with §8m line 5.\nFrame: %q", got)
		}
	})
}

// TestComposerDoorSaysAPayloadIsInFlashButNotLoaded closes fidelity I-8's §8r
// remainder: the one door line no loaded session can produce.
func TestComposerDoorSaysAPayloadIsInFlashButNotLoaded(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		p.sysw = &countingSyswReader{probe: true}
		ctx := NewContext(p)
		// No session: the payload is in flash and was skipped at boot.
		frame, _, ink, quit := runUITouchRaster(ctx, func() { composerDoorFlow(ctx, &descriptorTheme) })
		defer quit()
		got, ok := pumpUntil(frame, "Build a new policy", 16)
		if !ok {
			t.Fatalf("the door never drew.\nLast frame: %q", got)
		}
		assertFrameHasBody(t, ink(), "the door with a payload in flash")
		composerAssertDrawnInFull(t, "§8r payload in flash", got, composerCopyPayloadNotLoaded())
	})
}

// TestComposerConsentFlowNumbersPathsFromTheOperatorsList is fidelity I-2's
// PRODUCTION guard: the numbering fix was correct in isolation and dead from
// the live flow's point of view, because the only reachable call site
// hardcoded (nil, 0).
// MUTATION: pass nil, 0 to composerConsentLinesFor inside composerConsentFlow.
func TestComposerConsentFlowNumbersPathsFromTheOperatorsList(t *testing.T) {
	list := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	listed, keyPathNo := composerListedPaths(list)
	if keyPathNo != 1 || len(listed) != 1 || listed[0] != 2 {
		t.Fatalf("INCONCLUSIVE: composerListedPaths gave listed=%v keyPathNo=%d for a tr "+
			"list whose first path is the extracted internal key", listed, keyPathNo)
	}
	c, err := md.Compose(list)
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	st := &composerState{list: list, reg: &seedRegistry{}}
	composerSizeAssignments(st)
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		frame, quit := runUI(ctx, func() { composerConsentFlow(ctx, &descriptorTheme, st, chunks) })
		defer quit()
		got, ok := composerPageUntil(t, ctx, frame, "Path 2: 2-of-3", 10)
		if !ok {
			t.Fatalf("the consent surface never numbered the leaf as the operator's Path 2 "+
				"-- with an extracted internal key the branch list holds one entry, and "+
				"numbering branches calls the operator's Path 2 \"Path 1\".\nLast frame: %q", got)
		}
		if uiContains(got, "Path 1: 2-of-3") {
			t.Errorf("the consent numbers branches rather than the operator's paths.\nFrame: %q", got)
		}
	})
}

// TestComposerDateCeilingAndImpossibleDateAreToldApart is journey I-6's
// missing guard, and the defect it guards is F-458.
// MUTATION: restore `if y > 2038 || u == 0` as the ceiling test.
// composerDateToUnix returns 0 on EVERY failure, so that condition is a
// tautology and "that date does not exist" was dead code -- 2027-02-31 got
// the ceiling message, advising a block height for a date no height makes real.
func TestComposerDateCeilingAndImpossibleDateAreToldApart(t *testing.T) {
	for _, tc := range []struct {
		digits string
		exists bool
		why    string
	}{
		{"20270231", false, "2027-02-31 is inside the band and does not exist"},
		{"20271301", false, "month 13"},
		{"20450601", true, "2045-06-01 exists and is past the ceiling"},
		{"20081231", true, "2008-12-31 exists and is below the floor"},
		{"20270301", true, "the worked example, in band"},
	} {
		t.Run(tc.digits, func(t *testing.T) {
			y, m, d, parsed := composerParseDateDigits(tc.digits)
			if !parsed {
				t.Fatalf("%s did not parse", tc.digits)
			}
			if got := composerDateExists(y, m, d); got != tc.exists {
				t.Fatalf("composerDateExists(%s) = %v, want %v (%s)", tc.digits, got, tc.exists, tc.why)
			}
			_, inBand := composerDateToUnix(y, m, d)
			if inBand && !tc.exists {
				t.Fatalf("%s is in band but does not exist", tc.digits)
			}
		})
	}
	// The dispatch itself: an impossible date must NOT get the ceiling body,
	// and a past-ceiling date must.
	y, m, d, _ := composerParseDateDigits("20270231")
	if composerDateExists(y, m, d) {
		t.Fatal("INCONCLUSIVE: the impossible-date fixture is a real date")
	}
	y2, m2, d2, _ := composerParseDateDigits("20450601")
	if !composerDateExists(y2, m2, d2) {
		t.Fatal("INCONCLUSIVE: the past-ceiling fixture is not a real date")
	}
	if _, in := composerDateToUnix(y2, m2, d2); in {
		t.Fatal("INCONCLUSIVE: 2045-06-01 is inside the entry band")
	}
}

// TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit is tests-lens C-9,
// driven through `composerFlow` itself.
//
// MUTATION: `changed := false && shown != nil && !slices.Equal(shown, template)`
// in composerFlow.
//
// WHY THE PINNING TEST WAS NOT THE GUARD, measured in round 2:
// TestComposerStubReshowSignalIsTheChunkSet declares its own local `changed`
// and recomputes the comparison standalone -- it never calls composerFlow, so
// the line carrying the signal is untouched by it. The only other direct call
// to composerStubFlow in any test passes a HARDCODED false. So nothing
// exercised the real decision, and forcing it false left the whole suite ok.
//
// This walks the flow twice through the stub screen: once with no edit in
// between (§8s must be silent -- a false "this id changed" on the screen whose
// job is to be copied onto steel trains the operator to discount the line that
// will one day be true), and once after a real key-count change (§8s must
// fire, because a card minted with the old stub will not seat here).
func TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit(t *testing.T) {
	for _, tc := range []struct {
		name string
		edit bool
	}{
		{"back out and Done again with no edit", false},
		{"back out, change a key count, Done again", true},
	} {
		t.Run(tc.name, func(t *testing.T) {
			synctest.Test(t, func(t *testing.T) {
				p := newPlatform()
				p.display = sh2DisplaySize
				ctx := NewContext(p)
				frame, quit := runUI(ctx, func() { composerFlow(ctx, &descriptorTheme) })
				defer quit()

				// Wrapper -> wsh.
				if got, ok := pumpUntil(frame, "Which script?", 24); !ok {
					t.Fatalf("the wrapper picker never drew.\nLast frame: %q", got)
				}
				click(&ctx.Router, Down)
				click(&ctx.Router, Button3)

				// One 1-of-2 path.
				if got, ok := pumpUntil(frame, "Add a spend path", 24); !ok {
					t.Fatalf("the path list never drew.\nLast frame: %q", got)
				}
				click(&ctx.Router, Button3)
				pumpUntil(frame, "What can spend on this path?", 24)
				click(&ctx.Router, Button3) // Keys
				pumpUntil(frame, "how many keys?", 24)
				click(&ctx.Router, Down) // 1 -> 2
				click(&ctx.Router, Button3)
				pumpUntil(frame, "how many must sign?", 24)
				click(&ctx.Router, Button3) // k = 1

				composerFlowDone(t, ctx, frame)

				// THE FIRST SHOWING carries no changed-id line: there is
				// nothing it could be a change from.
				got, ok := pumpUntil(frame, "mk1 stub (template)", 32)
				if !ok {
					t.Fatalf("the stub screen never drew.\nLast frame: %q", got)
				}
				if uiContains(got, "The shape changed") {
					t.Errorf("the FIRST stub screen claims the id changed.\nFrame: %q", got)
				}
				click(&ctx.Router, Button1) // Back, to the path list

				if got, ok = pumpUntil(frame, "Add a spend path", 32); !ok {
					t.Fatalf("Back at the stub screen did not return to the path list.\n"+
						"Last frame: %q", got)
				}
				if tc.edit {
					// Path 1 -> Keys -> 3 keys, which moves a slot and so moves
					// the emitted chunk set.
					click(&ctx.Router, Button3)
					pumpUntil(frame, "Path 1:", 24)
					click(&ctx.Router, Button3) // Keys
					pumpUntil(frame, "how many keys?", 24)
					click(&ctx.Router, Down, Down) // 1 -> 3
					click(&ctx.Router, Button3)
					pumpUntil(frame, "how many must sign?", 24)
					click(&ctx.Router, Button3) // k = 1
					pumpUntil(frame, "Path 1:", 24)
					click(&ctx.Router, Button1) // leave the path editor
					pumpUntil(frame, "Add a spend path", 24)
				}
				composerFlowDone(t, ctx, frame)

				got, ok = pumpUntil(frame, "mk1 stub (template)", 32)
				if !ok {
					t.Fatalf("the stub screen never re-drew.\nLast frame: %q", got)
				}
				if tc.edit && !uiContains(got, "The shape changed") {
					t.Errorf("the stub screen was re-shown after a real key-count change and "+
						"does NOT carry §8s: a card minted with the old stub will not seat "+
						"here, and the operator is not told.\nFrame: %q", got)
				}
				if !tc.edit && uiContains(got, "The shape changed") {
					t.Errorf("the stub screen claims the id changed after a Back with NO "+
						"edit; §8s is then a false statement on the screen that gets copied "+
						"onto steel.\nFrame: %q", got)
				}
			})
		})
	}
}

// composerFlowDone takes the path list's "Done" row and answers the key-order
// question that follows it.
//
// The list's rows are the paths, then "Add a spend path", then "Change the
// script", then "Done" -- so with one path that is three Downs. Dispatch is by
// row NAME in the flow itself; the count here is the walk's own arithmetic and
// is asserted by the screen it lands on.
func composerFlowDone(t *testing.T, ctx *Context, frame func() (string, bool)) {
	t.Helper()
	click(&ctx.Router, Down, Down, Down)
	click(&ctx.Router, Button3)
	if got, ok := pumpUntil(frame, "Sorted keys, or your order?", 24); !ok {
		t.Fatalf("Done did not reach the key-order question.\nLast frame: %q", got)
	}
	click(&ctx.Router, Button3) // Sorted (usual)
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerSection8|^TestComposerReadScreenWithholds|^TestComposerPickScreenNever|^TestComposerConsentNumbers|^TestComposerDoorOffersFromPayloadForACard|^TestComposerShapeRefusalGate|^TestComposerLockAcceptRefuses|^TestComposerStubLinesLabelASeated|^TestComposerStubReshow|^TestComposerMappingReviewRefuses|^TestComposerShortfallCounts|^TestComposerHexEntryTakes' ./gui/ 2>&1 | head -8`
Expected: FAIL to build before the folds land -- `undefined: composerConsentLinesFor`, `undefined: composerCardFixture`, `undefined: hexDecodeForTest`.

- [ ] **Step 3: Run them**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerSection8|^TestComposerReadScreenWithholds|^TestComposerPickScreenNever|^TestComposerConsentNumbers|^TestComposerDoorOffersFromPayloadForACard|^TestComposerShapeRefusalGate|^TestComposerLockAcceptRefuses|^TestComposerStubLinesLabelASeated|^TestComposerStubReshow|^TestComposerMappingReviewRefuses|^TestComposerShortfallCounts|^TestComposerHexEntryTakes' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`16`**. Twelve was the count when this task was written and it was already wrong: `^TestComposerSection8` alone matches three distinct top-level functions, not one. Sixteen is the measured count, and the round-1 guards below add to it -- run the count, do not trust this line if it disagrees with a fresh run.

- [ ] **Step 4: Run the ROUND-1 guards, which are in the same file**

Round 1 of the R0 loop found six Importants with a correct production fix and NO test that fails if it regresses, two tests-lens Criticals still open (**C-9**, **C-12**), and two unnumbered mutation cells (**6b**, **8d**). Twelve tests at the end of `gui/composer_gates_test.go` close them, each naming the mutation it fails under in the comment above it:

| guard | closes | mutation it fails under |
| --- | --- | --- |
| `TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` | journey I-1 | add `composerShapeGuard` to the Time-lock arm |
| `TestComposerInvariantIgnoresSeveralUnseatedSlots` | journey I-2 / fidelity I-1 | remove the `src < 0` skip |
| `TestComposerBackInTheKeyEditorKeepsTheExistingKeySet` | journey I-5 | drop the snapshot/restore |
| `TestComposerChangeTheScriptRowRewrapsAndDiscards` | fidelity I-4, cell 6b | delete the row, or nullify the discard at that call site |
| `TestComposerConsentRestatesTheHashRule` | fidelity I-9 | delete the §8i block |
| `TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters` | tests I-1 | `valid := len(frag) >= 63` in the REAL function |
| `TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate` | journey I-6 / F-458, at the REAL screen | restore the `u == 0` disjunct inside `composerLockEdit`'s date closure |
| `TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit` | tests C-9, through `composerFlow` itself | `changed := false && ...` in `composerFlow` |
| `TestComposerShortfallCountsSeatsNotSourcesOnAFixtureThatCanTellThemApart` | tests C-12 | pass `len(st.sources)` instead of the assignable seats |
| `TestComposerMintCardsMintsOneCardPerSeatedSlot` | cell 8d | duplicate from the SECOND seated slot onward |
| `TestComposerSection8mRefusalsAllDrawThroughTheRealPath` | fidelity I-8's §8m remainder | remove any one `showError` arm |
| `TestComposerDoorSaysAPayloadIsInFlashButNotLoaded` | fidelity I-8's §8r remainder | drop the not-loaded branch |
| `TestComposerConsentFlowNumbersPathsFromTheOperatorsList` | fidelity I-2, at the PRODUCTION call site | pass `nil, 0` inside `composerConsentFlow` |
| `TestComposerDateCeilingAndImpossibleDateAreToldApart` | journey I-6 / F-458 | restore the `u == 0` disjunct as the ceiling test |

**Two of them are fixture fixes, not new assertions, and that is the point.** C-12's earlier fixture used two plain `key:` sources against four slots, where `len(st.sources)` and `composerAssignableSlots(st)` both give 2 -- structurally incapable of telling the two counting rules apart; a SEED fills any number of slots (C12, §4f), so one seed makes them 1 and 4. Cell 8d's earlier test unseated one of its own two slots before calling `composerMintCards`, so any duplication needing two seated slots was invisible to it.

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerLockAndHash|^TestComposerInvariantIgnores|^TestComposerBackInTheKey|^TestComposerChangeTheScript|^TestComposerConsentRestates|^TestComposerHexEntryItself|^TestComposerShortfallCountsSeatsNot|^TestComposerMintCardsMintsOne|^TestComposerSection8mRefusalsAllDraw|^TestComposerDoorSaysAPayload|^TestComposerConsentFlowNumbers|^TestComposerDateCeilingAnd|^TestComposerLockEditTells|^TestComposerFlowReShows' -v ./gui/ 2>&1 | grep -cE '^--- PASS'`
Expected: **`14`**, the count at plan time -- twelve round-1 guards plus the two round-2 replacements that drive `composerLockEdit` and `composerFlow`.

**THREE OF THE ROUND-1 GUARDS COULD NOT FAIL THEIR OWN NAMED MUTATION, and round 2 found all three.** They are replaced above, and the failing output each replacement now produces is pasted below -- produced in the gate's wired scratch by applying the plan's own named mutation, then reverted, `gofmt` clean.

**(a) journey I-6 / F-458.** The round-1 guard called `composerDateExists` and `composerDateToUnix` directly and re-derived the dispatch inline; `composerLockEdit` had zero test callers anywhere in the tree. The defect lives in the closure that CHOOSES the message, so the guard has to reach the closure -- through the kind picker, the absolute-kind picker and the digit pad. Under the named mutation:

```text
--- FAIL: TestComposerLockEditTellsAnImpossibleDateFromThePastCeilingDate
    --- FAIL: .../an_impossible_date_inside_the_band
        the date pad does not say "that date does not exist" for 20270231.
        Frame: "...20270231DateasYYYYMMDDThisbuildwritesdatesupto2038-01-19.
                Foralatertime,useablockheightinstead.Path1lock"
        the date pad says "up to 2038-01-19" for 20270231, which is the WRONG message
    --- FAIL: .../a_real_date_below_the_floor
        the date pad does not say "before 2009" for 20081231.
```

The older `TestComposerDateCeilingAndImpossibleDateAreToldApart` stays: it pins `composerDateExists`, which is worth pinning. It is not the guard, and this table no longer says it is.

**(b) tests I-1.** The round-1 guard used 63 as its "should fail" boundary, and 63 is ODD -- so `hex.DecodeString` refuses it for parity before the length bound is ever consulted, and the mutation was masked. Worse, the assertion read the function's RETURN value, and on a rejected fragment the entry loop `continue`s and never returns, leaving `ok` at its zero `false` -- the same answer a genuine refusal gives. The replacement asserts on what is OBSERVABLE: under the mutation a 63-character fragment is `valid`, so Button3 enters the decode branch and draws an error screen that cannot exist when the bound is correct. Under the named mutation:

```text
--- FAIL: TestComposerHexEntryItselfRefusesAnythingButSixtyFourCharacters
    --- FAIL: .../sixty-three_hex_characters
        composerHexEntry ACCEPTED 63 characters (the exact length the named mutation
        would admit): it reached the decode branch, which only a valid-length fragment
        does.
        Frame: "Thatisnota32-bytedigest.Hashlock"
```

The 62-character case is its even twin, and it is there for a different mutation: `hex.DecodeString` SUCCEEDS on an even length, so with `valid := true` the `len(raw) != 32` guard draws the same error. Between them the two cases catch a bound loosened in either direction, and neither leans on the decoder to do the refusing.

**(c) tests C-9.** The claimed closure rested on `TestComposerStubReshowSignalIsTheChunkSet`, which declares its own local `changed` and never calls `composerFlow`; the only other direct call to `composerStubFlow` in any test passes a HARDCODED `false`. So nothing exercised the real decision. The replacement walks `composerFlow` twice through the stub screen -- once with no edit (§8s must stay silent) and once after a real key-count change (§8s must fire). Under the named mutation:

```text
--- FAIL: TestComposerFlowReShowsTheStubScreenOnlyAfterARealEdit
    --- FAIL: .../back_out,_change_a_key_count,_Done_again
        the stub screen was re-shown after a real key-count change and does NOT carry
        section 8s: a card minted with the old stub will not seat here, and the
        operator is not told.
```

**The pattern, named so a later round does not re-learn it.** All three were tests that recomputed the rule beside the code instead of running the code -- and each was written IN RESPONSE to a finding of exactly that shape. A guard is only a guard once the mutation its own comment names has been applied and seen to fail it. That is now a step in this task, not a hope.

**One measurement the ink instrument cannot make, recorded rather than worked around.** The §8m slot-cap body ("This wallet already has 32 key slots.", 37 characters) draws **5,380** ink pixels on a two-button modal, and `titleOnlyInk`'s floor is **5,482** -- the WORST case over one to three nav buttons, i.e. a three-button frame's chrome. The floor is above this whole frame, so no ink assertion can say anything about this body; the test logs both numbers and asserts the words reached the screen, which is the property that matters.

- [ ] **Step 5: Apply each guard's own named mutation and see it FAIL**

**A guard is only a guard once its named mutation has been applied and seen to fail it.** Round 1 shipped three that could not, and every one of them was written in response to a finding of that exact shape. For each row of the table above: apply the mutation named in the row (and in the test's own comment), run only that test, confirm it FAILS, revert, and confirm the tree is byte-clean.

Run, per row: `git stash list >/dev/null; <apply the mutation>; CGO_ENABLED=0 go test -count=1 -run '<that test>' ./gui/ 2>&1 | head -8; git checkout -- <the mutated file>; git diff --quiet && echo REVERTED-CLEAN`
Expected: `--- FAIL` naming that test, then `REVERTED-CLEAN`. A mutation that leaves the suite green is a finding, not a formality: put the guard back on the workbench rather than the row in the table.

- [ ] **Step 6: gofmt, commit**

```bash
gofmt -l gui/ && CGO_ENABLED=0 go test -count=1 -run '^TestComposer' ./gui/ 2>&1 | tail -2
git add gui/composer_gates_test.go gui/composer_paged_test.go
git commit -s -F - <<'MSG'
gui: the section 12 item 5 gates section 8m, 8c and 8r never had, and a test per surviving mutation (composer S3 task C0)

Nineteen of twenty-seven mutations survived the previous suite. Each one that
did now has a test naming it. The short bodies are asserted on the surface the
operator meets them on, because assertModalBodyFits' own raster guard fires on
a twelve-character body -- measured, 5,166 ink against a 6,000 floor.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task C1: measure §13 item 1's numbers and write them INTO the spec

**Files:**
- Create: `gui/composer_measure_test.go`
- Modify (mnemonic-engrave, its own commit): `design/SPEC_wallet_policy_composer.md` §13 item 1, §6a, §7f

§13 item 1 lists four numbers as NOT VERIFIED: the per-frame capacities of the three paged screens (§7c's stub screen, §7d's pick list, §7e's consent) and the concrete-descriptor plate ceiling. They are plan-time RENDER measurements: `Choice.Size` and `widget.Labelw`'s sizes exist only at render time, so no static read can produce them. This task runs them and folds the results into the spec.

**The numbers are printed by a test, not typed.** A measurement transcribed by hand is a measurement nobody can re-run; this test prints all four in one block, and the fold pastes them.

- [ ] **Step 1: Write the measurement test**

Create `gui/composer_measure_test.go`:

```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/md"
)

// TestComposerMeasureSection13Numbers prints, in ONE block, every number
// SPEC §13 item 1 lists as unverified.
//
// IT ASSERTS ALMOST NOTHING, deliberately. Its job is to MEASURE, and a
// measurement test that also pins a threshold becomes a test nobody dares
// re-run when the font moves. The one assertion it does make is that each
// paged screen actually pages -- because a capacity equal to the whole body
// would mean the screen never pages and the number is meaningless.
func TestComposerMeasureSection13Numbers(t *testing.T) {
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)

	measure := func(name string, lines []string) {
		t.Helper()
		_, shown := composerPageLines(ctx, &descriptorTheme, sh2DisplaySize, lines, 0, -1)
		pages := 0
		if shown > 0 {
			pages = (len(lines) + shown - 1) / shown
		}
		t.Logf("SPEC13 %-14s lines=%3d per_frame=%2d pages=%d", name, len(lines), shown, pages)
		if shown >= len(lines) && len(lines) > 12 {
			t.Errorf("%s: %d lines all fit one frame, so the paging number is meaningless",
				name, len(lines))
		}
	}

	// (a) THE STUB SCREEN at the grammar's maximum: 32 slots.
	maxList := md.PathList{Wrapper: md.ComposeWsh}
	for i := 0; i < 4; i++ {
		maxList.Paths = append(maxList.Paths, md.SpendPath{Keys: &md.KeySet{K: 1, N: 8}})
	}
	c, err := md.Compose(maxList)
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	stub, err := composerStubLines(chunks, nil, true)
	if err != nil {
		t.Fatal(err)
	}
	measure("stub_screen", stub)

	// (b) THE PICK LIST at a payload the composer plausibly meets: 32 keys.
	rows := make([]string, 0, 34)
	for i := 0; i < 32; i++ {
		rows = append(rows, composerKeyLabel([4]byte{0x73, 0xc5, 0xda, byte(i)}, composerTestPath(i)))
	}
	rows = append(rows, "Type a seed", "Leave unseated")
	measure("pick_list", append([]string{composerCopySeatPrompt(2, 1, 2, 3), ""}, rows...))

	// (c) THE CONSENT at §7e's own worst case: eight paths plus four addresses.
	eight := md.PathList{Wrapper: md.ComposeWsh}
	for i := 0; i < 8; i++ {
		eight.Paths = append(eight.Paths, md.SpendPath{Keys: &md.KeySet{K: 1, N: 4}})
	}
	c2, err := md.Compose(eight)
	if err != nil {
		t.Fatal(err)
	}
	chunks2, err := c2.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	consent, err := composerConsentLinesFor(chunks2, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	measure("consent", consent)

	// (d) THE CONCRETE DESCRIPTOR PLATE CEILING, by the same search
	// qrCeilingBytes uses on the QR side (gui/transaction.go:1359-1391).
	n := composerDescriptorCeilingChars(p)
	t.Logf("SPEC13 %-14s ceiling_chars=%d  c10_688_fits=%v", "descriptor_plate", n,
		composerDescriptorPlateFits(p, strings.Repeat("a", 688)))
}
```

- [ ] **Step 2: Run it and capture the block**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposerMeasureSection13Numbers' -v ./gui/ 2>&1 | grep SPEC13`
Expected: four `SPEC13` lines. **Paste them verbatim into the fold below and into the fold commit's message.** Do not round them, do not re-derive them, and do not write a number this command did not print.

- [ ] **Step 3: Fold the four spec changes**

In `/scratch/code/shibboleth/mnemonic-engrave`, edit `design/SPEC_wallet_policy_composer.md`. **FOUR items, not three** -- the fourth is a round-1 addition, the destination a round-0 decline promised and did not reach:

**(a) §13 item 1** -- replace "The per-frame capacities of the three paged screens (§7c stub screen, §7d pick list, §7e consent) are the same kind of plan-time render measurement." with the four measured numbers, each naming the command that produced it and the fork revision it was measured at. Keep the sentence that they are render measurements; what changes is that they now exist.

**(b) §6a** -- the flag-screen sentence. It reads: "section 3.3.3's flag screens (F1 unencrypted-in-flash with its erase offer; F2 weak seal) fire inside the composer's seed step exactly as they do in Multisig Build". Measured at fork `169073c`: they fire at payload LOAD, from `syswLoadFlow`'s three call sites (`gui/gui.go:2074`, `gui/sysw_unload.go:36,75`), and `syswLoadWarnings` (`gui/sysw_load.go:259`) consults no admission table, so a mnemonic in a payload already raises F1 whatever the row says. Replace with a statement of the load-time mechanism, keeping §7g's DEFAULT classification unchanged -- the operator still meets the flags before the composer consumes a seed, which is what the classification asserts.

**(c1) §7a** -- where the door states the key state. §7a says "Beneath Build the door states the key state"; the implementation puts §8r's lines in the `ChoiceScreen.Lead`, i.e. ABOVE the choices, because the Lead wraps with `widget.Labelw` (`gui/gui.go:1969`) and the choice rows do not -- a line as long as "A payload is in flash but not loaded. Load it from the carousel first." would be cut off the panel as a row. Fold §7a's wording to say the state is stated WITH Build rather than beneath it, and name the wrapping as the reason. This was declined as a CODE change in round 0 and promised to this task; round 1 found the promise unfulfilled, which is why it is a numbered item here rather than a sentence in a table.

**(c) §7f** -- the secret's plate form. It reads "the secret is cut as words, as a SeedQR, or as ms1 strings". Measured: `engraveSeed` (`gui/gui.go:839-861`) puts words AND a SeedQR on ONE `backup.Seed` plate and there is no words-only or QR-only plate for a mnemonic; `backup.SeedString` (`backup/backup.go:26`) is the string-only form for ms1. Replace with the two forms the device has, and cite F-455 for the split.

Each edit carries the measurement or the `file:line` that forced it, in the sentence.

- [ ] **Step 4: The gates on the folded spec, then commit**

```bash
cd /scratch/code/shibboleth/mnemonic-engrave
CITE_FORK_ROOT=/scratch/code/shibboleth/seedhammer ./scripts/plan-cite-check.sh design/SPEC_wallet_policy_composer.md 2>&1 | tail -5
./scripts/plan-glyph-check.sh design/SPEC_wallet_policy_composer.md 2>&1 | tail -3
./scripts/plan-table-check.sh design/SPEC_wallet_policy_composer.md 2>&1 | tail -3
./scripts/spec-structure-check.sh design/SPEC_wallet_policy_composer.md 2>&1 | tail -3
```
Expected: citations all resolved with 0 dangling; 0 undrawable strings; every table row matching its header; the structure check clean. **Put this output in the fold commit's message**, which is what the standing rule asks of a fold.

```bash
git add design/SPEC_wallet_policy_composer.md design/FOLLOWUPS.md
git commit -s -F - <<'MSG'
spec: fold the four things S3 measured -- the paged capacities, the door's key state, the flag screens, the secret's plate form

Section 13 item 1's four numbers are now measured rather than deferred (the
command and the fork revision are in the section). Section 7a said the door
states the key state BENEATH Build; it is stated in the Lead, because the Lead
wraps and the choice rows do not. Section 6a said the flag screens fire in the
composer's seed step; they fire at payload load, from syswLoadFlow's three call
sites, and syswLoadWarnings consults no admission table. Section 7f offered
three secret forms; the device has two plate designs, and F-455 owns the split.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA
MSG
```

---

### Task C2: the gates, as CI runs them, plus the firmware size delta

**Files:** none changed unless a gate fails.

- [ ] **Step 1: The composer's own tests, then the whole package**

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposer' -v ./gui/ 2>&1 | tee /tmp/composer-tests.txt | grep -cE '^--- PASS'; grep -cE '^--- FAIL' /tmp/composer-tests.txt`
Expected: **`112`** top-level PASS and **`0`** FAIL. **That is the count at plan time**, measured in the gate's wired scratch on the day this was written, not a threshold: a task that adds a test moves it and must move this line with it, and a count SMALLER than this one is a test that stopped running. **Capture once and grep twice**: re-running the suite for the second number doubles the cost of every measurement.

Run: `CGO_ENABLED=0 go test -count=1 -run '^TestComposer' -v ./gui/ 2>&1 | grep -cE '^\s+--- PASS'`
Expected: **`100`** sub-tests, again the count at plan time.

- [ ] **Step 2: The whole `gui` package, sharded, and then the way CI runs it**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 2>&1 | tail -2`
Expected: `RESULT: ok -- all 1170 tests ran across 24 shards`, the count at plan time (1,125 on fork `main` before this stage, plus this plan's additions; the runner asserts its partition exhaustive before it runs anything, so a dropped test is a refusal rather than a green).

Run: `CGO_ENABLED=0 go test -timeout 20m ./... 2>&1 | grep -vE '^ok|no test files'; echo "exit=$?"`
Expected: no FAIL line. **Both**, not either: the shard runner is the fast local equivalent, and CI runs the plain command -- a suite that passes under process-per-shard isolation and fails under CI's runner is a shared-state defect, and this tree has had one.

- [ ] **Step 3: 32-bit, the oraclelive build, the emulator vet, and gofmt**

Run: `scripts/test-32bit.sh 2>&1 | tail -3; CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/ 2>&1 | tail -3; GOOS=js GOARCH=wasm go vet ./cmd/emu/ 2>&1 | tail -3; gofmt -l gui/ md/ mk/ sysw/ scripts/ 2>/dev/null | head`
Expected: all `ok`; gofmt names only the three files fork `main` already lists (`gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` -- pre-existing, measured at `321acb56`, and none of them this stage's). The composer's arithmetic is `uint32`/`uint64` throughout, so 32-bit is a build check rather than a behaviour question -- but it is run, not assumed.

- [ ] **Step 4: The firmware, and the size delta this stage costs**

Run: `nix run .#build-firmware 2>&1 | tail -5; tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller 2>&1 | tail -4`
Expected: the build succeeds and prints a `code`/`data`/`bss`/`flash`/`RAM` line. Record flash and RAM beside the baseline **1,503,652 B flash / 62,592 B RAM** (fork `169073c`), with the delta. **Unlike Stage 2, a non-zero delta is EXPECTED here**: this stage is reached from `cmd/controller` through `walletPolicyFlow`, so the linker keeps all of it. If the delta is zero, the door wiring did not land and nothing composed is reachable -- which is the same class of defect as a gate that never ran.

- [ ] **Step 5: The reachability gate, and the three emulator walks the door hangs**

Run: `FORK_REPO=/scratch/code/shibboleth/seedhammer SCRATCH=/scratch/code/shibboleth/.plan-build-gate-go-s3 /scratch/code/shibboleth/mnemonic-engrave/scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md 2>&1 | sed -n '/== 8 --/,$p'`
Expected: the `gui` line reads `1 function(s) with no production caller` and names `gui/composer_census.go: composerDescriptorCeilingChars` -- §13 item 1's measurement, whose production consumer is F-457. Any OTHER `gui/` survivor is a screen no operator can reach, which is what all three R0 lenses found, and it blocks. The `md`/`mk` names the same step prints are S2's package APIs called from `gui`; the scan compares within a package.

**THE EMULATOR WALKS ARE PART OF THE BLAST RADIUS AND THIS STAGE CANNOT RUN THEM.** Measured at `321acb56`: `grep -rln -i "wallet *policy" cmd/emu/*.js` finds `shots_seating.js`, `shots_tr_pathological.js` and `shots_walletpolicy.js`, and all three share the entry at `cmd/emu/shots_seating.js:145-146`:

```js
  // (3) Enter it. The gather screen is what it opens on.
  await tap(CONFIRM);
  await waitFor("md1descriptors:0");
```

With the door in front of `walletPolicyFlow`, that `CONFIRM` opens the DOOR and `md1descriptors:0` never draws; all three time out, and `design/journeys/capture_walletpolicy.py` in mnemonic-engrave drives the third, so the journey capture breaks with them. **The fix is one line in each**, because the door opens on "Scan cards" at index 0 and all three take the NFC gather route:

```js
  // (3) Enter it. The composer's door is now the first screen in every state
  //     (SPEC_wallet_policy_composer §7a) and opens on "Scan cards".
  await tap(CONFIRM);
  await tap(CONFIRM);
  await waitFor("md1descriptors:0");
```

Run: `GOOS=js GOARCH=wasm go build -o /dev/null ./cmd/emu/ && grep -c 'await tap(CONFIRM);' cmd/emu/shots_seating.js cmd/emu/shots_walletpolicy.js cmd/emu/shots_tr_pathological.js`
Expected: the WASM build succeeds and each file's `tap(CONFIRM)` count is one higher than at `321acb56` (measured there: `shots_seating.js` 2, `shots_walletpolicy.js` 1, `shots_tr_pathological.js` 2). **This is a build-and-count check, not a run**: the walks need a browser and playwright, which no gate in this stage has. Proving them is S4's journey run (§12 item 2), and it is named here so the edit is not discovered by S4 as a surprise.

- [ ] **Step 6: The report**

Write the implementation report to `design/agent-reports/composer-S3-implementation-report.md` in mnemonic-engrave (the fork is kept clean of design records): per task the fail-then-pass evidence, every `assertModalBodyFits` headroom number logged, the four `SPEC13` measurement lines, the consumption-site count before and after the oracle widening, the firmware sizes and delta, and **every deviation from an Expected line, verbatim**.

---

## Self-review against the spec and the staged plan

**Spec coverage (STAGED_PLAN §S3 "Delivers"), item by item:**

| §S3 deliverable | where |
| --- | --- |
| "the door ChoiceScreen in every state with its key-state lines" | the door task (§8r's six lines, counted through `has()` so an uncompared payload is still described) |
| "the shape flow with presets, the path-list screen, the digit-pad widget and lock/hashlock entry with echoes and refusals" | the shape task (path list, picker bounds, §4e/§8m), the digit-pad task, the lock task (§6b, §8c, §8o, §8t, §8u), the hashlock task (§6c, §8i); presets in their own task, BLOCKED on F-453 |
| "the paged stub-teaching screen with the conditional per-slot origin line and re-show" | the stub task, plus the two `Template-ID:` relabels on `gui/template_engrave.go` |
| "slot-directed seating from the payload with the paged pick list" | the paged-primitive task, the sources task, and **Task B11, which is what calls them** -- all three R0 lenses found that this row previously mapped to two tasks that nothing reached |
| "discard-on-numbering-change with the §8j confirm" | the discard task (`composerShapeSignature` moves for the wrapper, the path count and n, and for nothing else) |
| "the mapping review with verbatim origins, the unverifiable-account note, the C29 warning and the §8k line" | the mapping-review task |
| "the same-xpub refusal, the §4f invariant refusal (§8v)" | the mapping-review task |
| "the composer's consent on the paged review screen with the extended self-check and the §8l warning" | the consent-lines task (Part A) and the self-check task (Part B); the check runs on the DECODED md1 and is provoked by fault injection |
| "the engrave form choice including the partially seated form, Full/Watch-only" | the engrave-forms task; the three secret forms are TWO on this device and F-455 owns the split |
| "card minting and the census counting card chunks" | the minting task and the census task; the concrete-descriptor ceiling is a search, not a constant |
| "the deprecation comment on Multisig Build; the two comment rewrites; the admission-row change" | the admission task, with a test for each -- a comment-only deliverable with no gate is one nobody can tell was made |
| "Every §8 body under the glyph, raster and modal-fits gates and a fires-on-condition test" | the copy task's table plus an AST scan that fails when a body is declared without a row |
| Exit: "all §12 items except 2 and 9's ceiling number green in `go test ./gui/` (sharded)" | the gates task |
| Exit: "the per-frame capacities of the three paged screens and the plate ceilings measured and written into spec §13" | the measurement task, which prints the numbers and folds them |

**What this stage does NOT do, named so a reviewer does not look for it:** no emulator journey (§12 item 2 is S4's, and it is the gate a plan may not close while it has never run -- this plan does not claim it); no hardware and no plate (every walk here stops at the engrave screen); no NFC seating (§14); no on-device preimage derivation (§14); no presets unless F-453's Rust half has shipped; no third secret plate design (F-455); no change to Multisig Build's own EXPERIMENTAL warning text, which stays as shipped even though §8l reuses its SURFACE.

**Three recon facts corrected rather than inherited:** `mk.Card.Xpub` is a `string`, not `[65]byte`; `slotMatchesCard` is at `gui/key_card_seating.go:128`, not `:119`; and the recon does not mention `TestEverySyswConsumptionSiteNamesAnAdmittedClass`, which fails the moment the composer takes a record and which this plan widens and registers against.

**Type consistency across tasks:** `composerState`/`composerSource`/`composerAssignment`/`composerBound` are declared once, in `gui/composer_state.go`, and every later task reads the same fields; `composerSource.xpub` is a base58 `string` everywhere, converted with `decodeXpubBytes` only where `md.Composed.Bind` needs `[65]byte`; `composerSlotOrder`'s numbering is checked against `md.Composed.Slots()` rather than assumed; `md.Lock{Kind, Value}` carries OPERATOR units (units of 512 s for `LockOlderUnits`, not the `0x400000 + u` wire operand) in the state, the echoes and the tests alike.

**Placeholder scan:** three constants are filled by the implementer from named commands and pinned with those commands in comments (`composerTestXpubA`, `composerTestXpubB`, `composerTestDescriptor`); the preset table's path lists are transcribed from vendored vectors and the task refuses to proceed without them. No TBD, no TODO, and no step that says "as above".

**Gate coverage line for the review brief.** `scripts/plan-build-gate-go.sh` extracts and compiles every ```go block anchored on `gui/composer_*.go` -- which is every new file and every new test file in this plan, because the naming rule in Global Constraints makes them all match -- then runs `go vet ./gui/`, `go test -count=1 -run '^TestComposer' ./gui/`, and finally **a DEAD-IN-PROD scan that lists every function the plan declares in a production file that no production file calls**. That last step is the gate against the Critical all three R0 lenses found, and it did not exist when they read the plan: Go does not error on an unused package-scope function, so "an unreachable feature" was invisible to every check. It must print no `gui/` survivor, or the plan must name and justify each one. **Measured on the wired tree after this fold: exactly one**, `gui/composer_census.go: composerDescriptorCeilingChars` -- §13 item 1's measurement, called by `TestComposerMeasureSection13Numbers`, whose PRODUCTION consumer (form A's text and QR descriptor plates) is deferred to F-457 because `md` deliberately emits no descriptor text. `composerDescriptorPlateFits` is not a survivor: `composerDescriptorCeilingChars` calls it, which is the chain being exactly one function deep. The `md`/`mk` survivors the same step prints (`Compose`, `ComposeWith`, `ComposerStubs`, `AppendStubs`) are S2's package APIs consumed from `gui`, and the scan compares within a package. It does NOT assemble the fragments of existing files: `gui/sysw_admit.go`, `gui/gui.go`, `gui/multisig_build.go`, `gui/template_engrave.go`, `gui/multisig_build_census.go` and `gui/sysw_admit_oracle_test.go`, all six given above as exact old-to-new replacements for the controller to hand-wire in the gate's scratch copy before review. It does not cover the TinyGo firmware build, the whole `./gui/` suite (sharded separately), the emulator, the render measurements, or the two spec folds. **Reviewer budget belongs on what tools cannot reach here:** whether `composerSlotOrder` really tracks §5's numbering for a taproot list whose first single-key path is not first-listed; whether the self-check's comparison set is the one §7e names; whether the §8p shortfall's "no cause is guessed" rule survives its own implementation; and whether Part A is genuinely shippable without Part B.
