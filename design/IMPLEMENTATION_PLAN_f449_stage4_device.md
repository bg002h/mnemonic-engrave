# F-449 stage 4 — the device: the Liana key on the SeedHammer II (choice screen, derivation, print sites)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** an SH2 operator composing a `tr` wallet whose internal key is NUMS today, and which fits Liana's model, is offered Liana's unspendable key on the device; the device derives that wallet's real addresses, names the kind on every screen that names a key path, and refuses (never falls back) for a kind it cannot derive.

**Architecture:** the fork's Go composer gains md-codec 0.47.0's unspendable-key REQUEST (`md.ComposeWithUnspendable`, a port of `compose_with`'s third argument) and SPEC §6's mint refusals on the composer's mint path (F-654). `md.PolicyShape` and `md.Template` gain the fourth key-path kind (`KeyPathLianaUnspendable`, Rust's name). The address deriver gets a third internal-key arm that recomputes SPEC §2 over the card's own leaf keys and derives at `0/i`, `1/i`. The composer gets §0b's choice screen between `composerShapeFlow` and `composerTemplateChunksFor`, whose predicate is re-evaluated on every forward pass (that IS the reset). One seam, `composerCompose`, carries the choice to every artifact.

**Tech stack:** Go 1.26.7 (`/scratch/code/shibboleth/.toolchain/go`), TinyGo via the fork's nix flake (size only), the emulator (`cmd/emu`, wasm + Playwright), md-cli 0.19.0 (host oracle), Liana v15.0 harness (`scripts/liana-live-gate.sh`), Bitcoin Core (evidence only).

**Spec:** `design/SPEC_liana_unspendable_internal_key.md`. §9's stage-4 row is binding: §7's `KeyPathKind`, the class-2 + unlocked-path ruling, the print-site arms including `md1Summary`, F-633 copy, §7a.2's third address branch and §7a.3's refusal, and §0b's choice screen (predicate, placement, reset, default row, copy). Gate: §8.3's device leg, §7's constructed shape, an address test for a kind the device cannot derive, and §0b's predicate on all six `tr` presets firing on exactly `kofn-recovery` and `tiered-recovery`; plus §8.9's four stage-4 rows (PLACEMENT, RESET, DEFAULT ROW, COPY).
**Structural model:** `design/IMPLEMENTATION_PLAN_f449_stage3_go_port.md` (read its "R0 folds" section: its interfaces are this plan's inputs).

**Baseline revisions (record them; a staleness check needs all of them):**
- descriptor-mnemonic `main` **`cf35d61a`** (md-codec 0.47.0, md-cli 0.19.0).
- seedhammer fork `main` **`d2350cb`** — the stage-3 merge ("Merge f449-stage3", landed 2026-09-23 while this plan was being written). The plan was first measured against the R0 reviewer's scratch copy of the stage-3 end state (`/scratch/code/shibboleth/.tmp/r0-s3/fork`, `2a19fa6`), then **rebased onto `d2350cb` and re-measured there**; every number below is from `d2350cb`. The rebase needed a three-way merge in four files, because the real merge carries stage 3's R0-fold comments the scratch copy lacked: `md/md.go` merged cleanly; `md/compose.go`, `gui/policy_address.go` and `gui/taproot_script_path_test.go` each conflicted on ONE stage-3 comment that named F-449 stage 4 as the owner of exactly the code this plan replaces, and those comments are rewritten, not kept.
- descriptor-mnemonic `main` **`d269c556`** (stage 3's vectors merged; md-codec 0.47.0, md-cli 0.19.0 unchanged since `cf35d61a`).
- mnemonic-engrave `master` **`fe73cca6`** (stage 4a merged; F-654 filed by stage 3's records at `6eb56905`).

**This plan's GREEN expires when fork `main` moves past `d2350cb`** (CLAUDE.md). Task 1 Step 2 is the re-validation: re-run the apply gate (below) against the new `main`. Every code block is a patch with context lines or a whole new file, so a hunk that no longer applies IS the drift, located. Scope it to "what did the move falsify here?", not a fresh audit.

**Worktrees:**
- fork: `/scratch/code/shibboleth/sh-worktrees/f449-stage4`, branch `f449-stage4`, off fork `main` (`d2350cb`).
- mnemonic-engrave: Task 7's two journey scripts and Task 9's records, on `master` directly (records) or a branch `f449-stage4` (scripts), stage paths explicitly.

## How this plan was checked before review

Every code block below was applied, **task by task**, to a fresh clone of fork `main` at `d2350cb`, by extracting it from THIS FILE (the apply gate), and each boundary was built and tested before the next was applied (a gate that wires all tasks at once cannot see order). Nothing was committed to a real repo.

| boundary | what ran | result |
| --- | --- | --- |
| baseline `d2350cb` | `go test ./md/ -list`; `scripts/gui-shard-test.sh ./gui/ 24`; gofmt | md **179**; gui **1380**; the five-file set |
| Task 2 | `go build ./...`, `go vet ./md/`, `go test ./md/`, gui `TestCompose*` | green |
| Task 3 | same, gui `TestPolicy|TestMd1|TestTemplate` | green |
| Task 4 | same, gui address + taproot tests | green |
| Task 5 | same, gui copy / key-path / inspect / modal / walk-anchor tests | green |
| Task 6 | same, gui `TestComposer*` + Liana + modal + walk-anchor | green |
| Task 7 | same, gui `TestEmulatorWalks*`; `node --check` | green |
| end state | `go test ./md/`; `scripts/gui-shard-test.sh ./gui/ 24`; every non-gui package | md **188**; gui **1398 (all ran, 24 shards)**; non-gui 55 `ok`, 0 fail |
| end state | gofmt | exactly the five-file baseline |
| end state | emulator: `capture_composer.py --arm both` (keyed-A, keyed-B, keyless, liana) | **all legs matched the host**: keyed-A (7 strings), keyed-B (9), keyless (1), liana (1 string, 28 s, 11 shots); `--prove-it-can-fail` PASSED; the liana arm FAILS under the placement mutation (L1) |
| end state | TinyGo size (CLAUDE.md recipe) | base **1,658,284 B flash / 63,352 B RAM**, end **1,665,828 / 63,376** (**+7,544 / +24**; +608 B from the R0 fold) |
| end state | the mutation table in the Self-Review | every row applied, compiled, and was caught, except the one named survivor |

**R0 fold (review `design/agent-reports/f449-plan-stage4-r0.md`, 0C/1I/5M/2N).** A fold is authorship, so everything in the table above was re-run on the folded plan, from this file, against `d2350cb`: the apply gate (all seven boundaries green, gui **1398**), the mutation table (**40 caught, S8 survives**), the emulator walk (Task 7 did not change, but Task 6 did, and the liana arm drives Task 6's screens: `--arm both` all four legs matched the host, the negative control PASSED), and the TinyGo size (+608 B flash for the fold). One defect the fold itself introduced was caught only by the whole-package shard run, not by the fold's own targeted tests: the unmet-request copy was first called only from a package-level `var`, and `TestComposerEveryScreenFunctionHasAProductionCaller` failed (a `composer*` function with no caller inside a function body). The text is now built inside `composerCompose`.

**The apply gate.** `/scratch/code/shibboleth/mnemonic-engrave/scripts/plan-apply-gate-go.sh <fork-base-dir>` extracts this file's `Create`/`Apply to` blocks per task, applies them in order to a fresh clone, and runs the per-boundary gate above. It is how a fold re-earns the build gate. **The shipped `scripts/plan-build-gate-go.sh` cannot check this plan**: it assembles only whole NEW files matching `md/compose*.go`/`gui/composer_*.go` (here, 5 of the 8 created files) and never applies a patch, while most of this plan is patches to existing files. What no gate covers is listed at the end.

## Measured facts this plan rests on

Each was measured, not read from a doc comment. A reviewer can re-run the command.

| # | fact | how |
| --- | --- | --- |
| F1 | **Rust named the fourth kind `LianaUnspendable`, not the spec's `NumsXpub`** (`crates/md-codec/src/policy_shape.rs:145-162`, `skeleton.rs:275`). The Go sibling mirrors Rust: `md.KeyPathLianaUnspendable`. Task 9 reconciles the spec's §7 sentence. | `grep -n LianaUnspendable crates/md-codec/src/policy_shape.rs` at `cf35d61a` |
| F2 | **§7's class-2 + unlocked-path ruling needs no code**: class 2 is `shape.KeyPath == md.KeyPathNUMS` (`gui/composer_consent.go:397`) and the unlocked count is `shape.KeyPath == md.KeyPathSpendable` (`:407`), so a fourth value is skipped by the first and not counted by the second. Both are pinned by mutation (Self-Review C1, C2). | Task 5's `TestComposerLianaClassRulings` |
| F3 | The device's kind-1 derivation equals **Rust md** on both keyed kind-1 vectors (6 addresses each), on the same wallet engraved as **template + mk1 key cards** (after the R0 I1 fold: the recipe now takes the keys the deriver is given), and **Liana v15.0's own recorded addresses** on all **5** accepted evidence cases (receive and change 0..2). | Task 4's three tests |
| F4 | The Go composer's `kofn-recovery` at kind 1, unseated, emits **`md13ls8aqqxq6tvyyykjmpprj6tvyy495kcgfwtsqrq0zjqgsexd9dcqqqv65q3cm0m0nz2h7w9`**, byte-identical to `md compose --wrapper tr --preset kofn-recovery,2of3,older=26280 --unspendable liana \| md encode --force-chunked --group-size 0` (md-cli 0.19.0). Template-ID `f99cc42e1ff68bae546c4d1070e5963f`; the NUMS twin's is `8107216456de60d05e57f7fe268824d8`. | Task 2's `TestComposeLianaTemplateEqualsTheHost`; Task 7's transcript gates |
| F5 | §0b's predicate over the six `tr` presets fires on exactly `kofn-recovery` and `tiered-recovery`. Without conjunct 1 it also fires on `simple-timelocked-inheritance`; without conjunct 2 on `plain-multisig`, `hashlock-gated`, `decaying-multisig`; with class 2 NOT skipped, on nothing. | Task 6 + Self-Review P1-P3 |
| F6 | **Liana v15.0 refuses** the four F-644 shapes and §7's constructed shape (one timelocked leaf), each with its internal key recomputed over its own leaves by the harness's `unspendable` subcommand, so none is a key refusal wearing a policy refusal's words. It accepts a nested two-recovery tree and a `pk` primary leaf + recovery. The device classifier names a class for all five refused shapes, so the choice screen never offers the Liana key for any of them. | Task 9 Step 1's commands; `scripts/liana-live-gate.sh` PASS (11 verdicts) first |
| F7 | **Bitcoin Core imports the kind-1 `kofn-recovery` descriptor**, and its `getnewaddress` 0..2 equal Liana's recorded receive addresses. Measured on a v30.99 development build (`/scratch/code/bitcoin/build/bin`), not a release; the kind-0 claim in §8f was measured on 25.0/31.1. | Task 9 Step 1 |
| F8 | **The composer emulator walk is RED at the stage-3 end state**, three ways, none caused by F-449: (a) the keyless arm waits for `Keyless template - no addresses.` while the device draws `Template has no keys - no addresses.`; (b) every arm stalls on the `Restore Doc` screen F-544 added after the bundle (fork `5971ad3`, 2026-09-16, after the walk's last edit `b77cb51`); (c) keyed-B's census pages, and `composerReadScreen` withholds the checkmark until the last page is seen. (a) was reproduced on an unmodified clone of the scratch end state; `cmd/emu/shots_composer.js` and every string it waits for are byte-identical at `d2350cb`. | `capture_composer.py --arm keyless` against the base |
| F9 | The `md` on `PATH` (`~/.cargo/bin/md`) and `descriptor-mnemonic/target/release/md` (the transcript's default `$MD`) are both **md 0.18.0**, which has no `--unspendable`. | `md --version` |
| F10 | `composerCopyTable`'s comment says every row gets "the raster floor and the modal-fits assertion (composer_copy_gate_test.go)". **That file does not exist.** So the one new modal body (the RESET signal) gets an explicit `assertModalBodyFits`. | `ls gui/composer_copy_gate_test.go` |
| F11 | The SH2's only input is the touch panel. `ChoiceScreen` carries one Lead and bare row labels; `composerPickScreenFrom` (`gui/composer_paged.go:295`) draws a per-page lead, makes every row a tap target, and reads `initial` once. The choice screen uses the latter; the first-page gate counts **tap targets** (`plateHitPoints`), not text, because `composerPageLines` draws an overflowing row it does not count. | Task 6's first-page test and mutation F1 |
| F12 | Adding the screen broke **no** existing gui test except the two copy-table counters (`TestComposerCopyIsVerbatimFromTheSpec`, `TestComposerCopyTableCoversEveryBody`), measured by a full shard run before the table was updated. | full shard run after Task 6 |
| F13 | Stage-4-owned follow-ups at `fe73cca6`: **F-644** and **F-654** (`grep -n "F-449 stage 4\*\*" design/FOLLOWUPS.md`). F-633 is owned by the coordinator-compat cycle, but SPEC §7 makes its copy gating here. The highest ID is F-658, so new entries start at F-659. | `grep` |
| F14 | `composerRestoreDoc` (`gui/composer_flow.go:528`) tells an operator who cut a template and NO cards that "This backup is a key-less TEMPLATE plus its key cards … restore needs the mk1 key cards listed below" — none are listed. Pre-existing (F-544); filed, not fixed (Task 9). | the liana arm's `l06-` frames |

## Global Constraints

- **Rust leads, Go follows.** Every normative behaviour here exists in md-codec 0.47.0 / md-cli 0.19.0 (the compose request, `validate_unspendable_shape`, `KeyPathKind::LianaUnspendable`). Port names and semantics as Rust has them. If a port step meets Rust behaviour that looks wrong, file a Rust follow-up and port Rust as it is.
- **Never map a non-Slot internal key to the NUMS point.** `taprootInternalKey` has no `default:` that derives; an unknown kind is `errUnderivableInternalKey` (SPEC §7a row 1 / §7a.3).
- **The Liana key derives at `<0;1>/*` explicitly**, never from the wallet's use-site (SPEC §2 "derivation, not just rendering"; §6 row 2 refuses any other use-site at mint).
- **§6's refusals stay OUT of `md`'s `encodePayload`**: `Reassemble` re-encodes decoded cards (stage 3's Global Constraints). They run in `composerCompose`, on the mint path.
- **One compose site.** Every production lowering of the operator's path list goes through `composerCompose`, enforced by `TestComposerComposesOnlyThroughOneSite` **per enclosing function**, not per file (R0 m4). The two reviewed exemptions: `composerUnspendableFires` (composes kind 1 for itself to evaluate §0b's predicate; builds no artifact) and `composerShapeSignature`'s `md.Compose` (reads only the slot map, which the kind does not move, SPEC §5).
- **One key source for the Liana internal key.** It is computed from the keys the address deriver is GIVEN (`md.LianaUnspendableKeyFor(collected, keys)`), the same keys every leaf script is built from, never from the md1's Pubkeys TLV. `lianaInternalKey` is its only production caller (R0 I1).
- **The kind's zero value is NUMS**, and the choice screen seeds its row from the current kind once per entry, never from a constant.
- **Touch only.** No new test or walk step moves a cursor with `Up`/`Down` or presses a nav button with a synthetic `ButtonEvent`: rows are TAPPED (`sessionHarness.choose`/`tapRow`, `chooseRow` over `shTargets()`) and nav buttons are tapped on their slot (`tapNav`/`tapNavSlot`). The first draft broke this in two tests (R0 m2); `grep -n 'click(&ctx.Router' gui/composer_unspendable*_test.go gui/keypath_print_sites_test.go gui/policy_address_liana_test.go` finds nothing. The legacy composer flow tests that click `Down` are not extended by this plan.
- **Copy:** ASCII only (`TestComposerCopyIsDrawable`); every new body is a `composerCopy*` function with a `composerCopyTable` row, verbatim, and SPEC_wallet_policy_composer §8 gets the same text (Task 9). Present-tense claims about third-party software name the version measured ("Liana (as of v15.0)", F-633).
- **md1PolicyFlow hard-chunks lines over 20 bytes mid-word**, so every new inspect-screen line is ≤ 20 bytes.
- **Toolchain:** `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`. nix: `export PATH=/nix/var/nix/profiles/default/bin:$PATH`. Shell blocks are bash (the interactive shell is fish); commit messages go through `git commit -F <file>` with the attribution lines your session's system reminder gives.
- **Baseline noise to diff against, not assert empty:** `gofmt -l .` lists the five files `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`; `go vet ./gui/` reports the two `testing.ArtifactDir requires go1.26` lines.
- **Build under `/scratch/code/shibboleth/.tmp/`, never `/tmp`** (a 32 GB tmpfs). The gui shard script uses `mktemp -d`: set `TMPDIR=/scratch/code/shibboleth/.tmp/f449s4-tmp`.
- **No flash in this plan.** The signed image may be pre-built (`~/bin/sh/sh2-flash -b`); the flash waits for the operator at the board, never unattended.

## Review Focus

Inputs the spec implies that no task's unit tests fully exercise, most likely first:

1. **An operator who minted cosigner cards from the NUMS stub, then chooses Liana on a later pass.** The Template-ID moves; `composerStubDelta`'s changed-id banner is what tells them (existing, unchanged). Pinned only indirectly: Task 6's placement test proves the stub shows the new id; no test asserts the banner text for this cause. Expected behaviour: the banner fires. Owner if it does not: Task 6.
2. **A kind-1 wallet engraved as template + mk1 key cards, proven later on Wallet Policy.** At the COMPOSER's consent, before any card is seated, it shows `Template has no keys - no addresses.` and the Liana key line (the emulator's liana arm). Brought back to Wallet Policy WITH the cosigners' mk1 cards, the device derives the wallet's addresses from the seated cards: `TestTemplatePlusKeyCardsDerivesTheLianaWallet` (Task 4) shows Rust md 0.19.0's receive 0 for `keyed_tr_liana_kofn_recovery` on exactly this route. The R0 review found this route FAILED in the first draft (the recipe read the template's absent TLV; I1). What remains untested is only the EMULATOR walk of it: the mk1 cards arrive over NFC, and the liana arm stops at the engrave.
3. **A board running stage-3-or-earlier firmware reading these plates.** Nothing new can reach it; the stage 5 runbook must say "flash before reading kind-1 plates" (stage 3 already recorded this).
4. **An operator who expects Nunchuk.** The copy says "Nunchuk only by chance" (SPEC §7, fable M-5: 1/24 for four keys). No device test can see Nunchuk; documentation only.
5. **A keyed kind-1 card from ANOTHER producer** (a Liana wallet decomposed by `md decompose`) inspected on the device. It derives through the same `LianaUnspendableKeyFor` path; covered for the two Rust vectors, not for foreign bytes. Expected: the same addresses Liana shows.

## The operator's journey through the new screens

Walked on the emulator (Task 7's liana arm) and in the Go harness. At each step: what the operator has, what the device does, what else they might reasonably do. Classes: **refusal / warning / default / not our concern / documentation only**. A divergence earns a change only if the wrong outcome is worse than telling the operator nothing.

| # | in hand / action | device | what else they might do → outcome | class |
| --- | --- | --- | --- | --- |
| 1 | Wallet Policy → Build → Taproot → `kofn-recovery` → Done | the **Key path** screen, before any id is shown | pick Segwit instead → no screen (wsh has no key path) | not our concern |
| 2 | Key path screen, press ✓ without reading | NUMS (row 0, the wallet every earlier firmware built) | — | **default** (zero-value trap closed, Task 6 DEFAULT ROW test) |
| 3 | tap `Liana key`, ✓ | stub screen shows Template-ID `f99cc42e…` | tap Back → the path list, kind unchanged | default |
| 4 | Back to the path list, make Path 1 a single key, Done | `LIANA KEY DROPPED` modal: "Path 1 is one key with no lock, so it became the key path…"; then NUMS/real key | — | **warning** (RESET, Task 6) |
| 5 | Back, give the recovery path `after(…)`, Done | `LIANA KEY DROPPED`: "Liana would not import this policy (an absolute lock)." | — | **warning** |
| 6 | Change the script to Segwit | `LIANA KEY DROPPED`: "Only a Taproot policy has a key path to choose." | — | **warning** |
| 7 | Back-edit `kofn` → `tiered` (both conjuncts still true) | screen reopens ON the Liana row; ✓ keeps it | — | default (RESET converse, Task 6) |
| 8 | already minted cards from the NUMS stub, then choose Liana | stub screen: the existing changed-id banner (`composerStubDelta`) | — | warning (existing) — Review Focus 1 |
| 9 | seat all keys, reach consent | `KEY PATH: NONE (LIANA KEY)` + Liana's own receive/change addresses | expect Nunchuk → copy says "only by chance" | documentation only |
| 10 | `hashlock-gated` / `decaying-multisig` / `plain-multisig` under tr | no Key path screen; consent keeps §8x "OUTSIDE LIANA'S MODEL … Liana (as of v15.0) takes…" | ask why no Liana option → §8x names the class | documentation only (existing notice) |
| 11 | a hand-built `[2-of-3], [2-of-2]` or `[1 key + older]` alone | no screen (classes 3, 7; Liana v15.0 refuses both, F6) | — | not our concern |
| 12 | engrave template only (no keys seated) | consent: `Template has no keys - no addresses.`; census 1 plate; **Restore Doc says "plus its key cards … listed below", none listed** | — | not our concern here; **filed** (F14 → Task 9) |
| 13 | a year later, the TEMPLATE plate plus the cosigners' mk1 cards, on Wallet Policy | consent: `Key path: Liana key` and the wallet's receive/change addresses, derived from the seated cards (Task 4, R0 I1) | inspect the template ALONE → `Key path: Liana key`, no `Policy id:` and no addresses (a template has no keys); a full KEYED md1 alone → `Policy id:`, `Key path: Liana key`, addresses on Button2 (Task 5); read on a stage-2 board → "Not an md1 descriptor chunk." | default; the stage-2 board: documentation only (stage 5 runbook) |
| 13a | seat two keys from ONE seed in one path of the Liana wallet | the mapping review's existing §8g notice: "SAME SEED, SAME PATH … Liana will refuse it" (Liana v15.0 refuses two keys from one master in one spending path, measured by R0) | — | warning (existing); the choice screen's "Liana (v15.0) … import it" is about the POLICY, and §8g covers the seating (R0 n2) |
| 14 | a policy whose key-path kind this firmware cannot name (a future fourth kind) | Wallet Policy consent REFUSES before steel; inspect says `Key path: unknown`; no address | — | **refusal** (§7a.3, Task 5; unreachable today, see "What no gate covers") |

---

## File Structure

**fork (`/scratch/code/shibboleth/sh-worktrees/f449-stage4`):**
- `md/compose.go` — `lowerPathList`/`lowerTr` take the request; `ComposeWith` delegates; `Composed.requested`. (Task 2)
- `md/compose_unspendable.go` (new) — `UnspendableKind`, `ComposeWithUnspendable`, `UnspendableRequestUnmet`, `ValidateUnspendableShape` + three errors. (Task 2)
- `md/policy_shape.go` — `KeyPathLianaUnspendable`, `keyPathOf`. `md/md.go` — `Template.KeyPath`, `rootKeyPath`. `md/liana.go` — `LianaUnspendableKeyFor`. (Task 3)
- `gui/policy_address.go` — `taprootInternalKey`, `lianaInternalKey`, the third arm. (Task 4)
- `gui/composer_consent.go`, `gui/template_engrave.go`, `gui/md1_inspect.go`, `gui/wallet_policy.go`, `gui/composer_copy.go` — the print-site arms, the classifier's doc, `md1KeyPathLine`, `md1KeyPathUnknown`, `composerCopyLianaKeyPath`, the F-633 edits. (Task 5)
- `gui/composer_unspendable.go` (new), `gui/composer_state.go`, `gui/composer_flow.go`, `gui/composer_selfcheck.go`, `gui/composer_copy.go` — the choice screen, `composerCompose`, the self-check. (Task 6)
- `cmd/emu/shots_composer.js`, `cmd/emu/expect_composer.json`, `cmd/emu/expect_fixture_test.go`, `gui/walk_copy_anchors_test.go` — the walk re-greened and the liana arm. (Task 7)
- Tests, new: `md/compose_unspendable_test.go`, `md/keypath_liana_test.go`, `gui/policy_address_liana_test.go`, `gui/keypath_print_sites_test.go`, `gui/composer_unspendable_test.go`, `gui/composer_unspendable_sites_test.go`. Modified: `gui/policy_address_test.go`, `gui/taproot_script_path_test.go`, `gui/composer_copy_test.go`.

**mnemonic-engrave:** `design/journeys/transcript_composer.sh`, `design/journeys/capture_composer.py` (Task 7); `design/evidence/f449-stage4/` (new), `design/FOLLOWUPS.md`, `design/SPEC_liana_unspendable_internal_key.md`, `design/SPEC_wallet_policy_composer.md` (Task 9).

**Not touched:** `address/`, `sysw/`, md's wire codec (`encode.go`, `chunk.go`, identity), every Rust file.

---

## Task 1 (fork): worktree, measured baseline, and the re-validation

- [ ] **Step 1: Create the worktree off the stage-3 merge.**
```bash
git -C /scratch/code/shibboleth/seedhammer log --oneline -1   # d2350cb Merge f449-stage3 (or later: then Step 2 matters)
git -C /scratch/code/shibboleth/seedhammer worktree add /scratch/code/shibboleth/sh-worktrees/f449-stage4 -b f449-stage4 main
cd /scratch/code/shibboleth/sh-worktrees/f449-stage4
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp/f449s4-tmp
mkdir -p $TMPDIR
```

- [ ] **Step 2: Re-validate the plan if fork `main` has moved past `d2350cb`.**
```bash
/scratch/code/shibboleth/mnemonic-engrave/scripts/plan-apply-gate-go.sh /scratch/code/shibboleth/sh-worktrees/f449-stage4
# the prose line citations, re-resolved by SYMBOL (measured at d2350cb):
grep -n "composerSizeAssignments(st)$\|template, err := composerTemplateChunksFor(st)\|md.ComposeWith(" gui/composer_flow.go   # 96, 98, 268/283/308
grep -n "switch ik {" gui/policy_address.go                                                    # 157
grep -n "switch shape.KeyPath\|if shape.KeyPath == md.KeyPathNUMS\|if shape.KeyPath == md.KeyPathSpendable" gui/composer_consent.go  # 205, 397, 407
grep -n "switch shape.KeyPath" gui/template_engrave.go; grep -n "^func md1Summary" gui/md1_inspect.go   # 159; 84
grep -n "stillUnsupported := map" gui/policy_address_test.go; grep -n "if declared != 86" gui/composer_copy_test.go   # 128; 452
```
Expected: the apply gate applies every block and prints `ALL BOUNDARIES GREEN` (it clones the worktree's HEAD, so run it before Task 2 commits anything). A block that no longer applies is exactly what stage 3's landing changed: re-derive that hunk against the tree, record it here, and re-run. Do not start Task 2 on a red apply gate. The script prints what it does NOT cover in its header.

- [ ] **Step 3: Measure the baseline once, to files.**
```bash
go test ./md/ -list '.*' | grep -cE '^(Test|Fuzz|Example)'            # 179 at d2350cb
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > $TMPDIR/gui-base.txt 2>&1
tail -1 $TMPDIR/gui-base.txt                                             # "all 1380 tests ran" at d2350cb
gofmt -l . | sort                                                        # the five-file set
```
If a count differs, `main` has moved: record the new numbers and carry them through Task 8.

---

## Task 2 (fork, `md/`): the composer's unspendable-key request and SPEC §6's mint refusals (F-654)

**Why here, and why not in `encodePayload`.** md-codec 0.47.0's `compose_with` takes the request as a third argument (`compose/mod.rs:643-667`) and decides the kind at ONE site (`compose/tr.rs:47-55`); md-cli then runs `validate_unspendable_shape` before emitting (`cmd/compose.rs:639`). The Go port keeps both: the request lives in `md.ComposeWithUnspendable`, the refusals in `(md.Composed).ValidateUnspendableShape`, which Task 6's `composerCompose` calls. `encodePayload` is untouched, because `Reassemble` re-encodes every decoded card (stage 3's Global Constraints). `validate_minimal_wire_version` is NOT ported: the Go encoder has no version parameter (it derives the version from the tree, stage 3), so the state it refuses cannot be constructed in Go.

**Files:** Create `md/compose_unspendable.go`, `md/compose_unspendable_test.go`; Modify `md/compose.go` (`Composed` at `:447-452`, `Compose` `:643-649`, `ComposeWith` `:653-662`, `lowerPathList` `:908-910`, `lowerTr` `:933`, the kind at `:961-968`, at `d2350cb`). The stage-3 comment at `:961-963` ("that is the port of md-codec's `--unspendable`, owned by F-449 stage 4") is replaced, because this is that port.

**Interfaces:**
- Consumes (stage 3): `trBody.ik`, `InternalKeyKind` (`InternalKeySlot`/`InternalKeyNUMS`/`InternalKeyLianaUnspendable`), `tagSortedMultiA`, `useSitePath`, `alternative`, `idxUseSite`.
- Produces: `type UnspendableKind uint8` with `UnspendableNums` (zero value) and `UnspendableLiana`; `func ComposeWithUnspendable(list PathList, declared []*SlotOrigin, unspendable UnspendableKind) (Composed, error)`; `func (c Composed) UnspendableRequestUnmet() bool`; `func (c Composed) ValidateUnspendableShape() error`; `ErrUnspendableSortedMultiA`, `ErrUnspendableUseSite`, `ErrUnspendableNotRootTr`. `ComposeWith(list, declared)` is now `ComposeWithUnspendable(list, declared, UnspendableNums)`, so its callers (16 call sites in test files, 4 in production at `d2350cb`; Task 6 moves the 3 composer ones) are unchanged.

- [ ] **Step 1: Write the failing tests.** They bind the Go composer to the Rust primary's two keyed kind-1 vectors (stage 3 vendored them) and to md-cli 0.19.0's printed template (F4).

Create `md/compose_unspendable_test.go`:

```go
package md

import (
	"bytes"
	"encoding/hex"
	"errors"
	"reflect"
	"testing"
)

// F-449 stage 4: the Go composer's Liana request, bound to the Rust primary's
// kind-1 vectors (stage 3 vendored them), and SPEC §6's mint refusals.

// lianaComposeRows are the two keyed kind-1 vectors the Rust primary minted
// (descriptor-mnemonic test_vectors.rs, F-449 stage 3 Task 1) as COMPOSER path
// lists. Both use the tr default origins (48'/0'/i'/3') with every slot
// unseated, which is what those MANIFEST entries declare.
func lianaComposeRows() []struct {
	name string
	list PathList
} {
	return []struct {
		name string
		list PathList
	}{
		{"keyed_tr_liana_kofn_recovery", cpl(ComposeTr, cu(2, 3), clk(ck(1, 1), olderBlocks(26280)))},
		{"keyed_tr_liana_nested_two_recoveries", cpl(ComposeTr, cu(2, 2), clk(ck(1, 1), olderBlocks(26280)), clk(ck(1, 1), olderBlocks(52560)))},
	}
}

// TestComposeLianaReproducesTheRustVectors is the Go composer's Rust-primary
// binding: composing the preset with the Liana request, then binding the
// vector's keys, yields the primary's payload bytes and chunk strings.
//
// Mutation: lowerTr's `case UnspendableLiana:` arm yielding InternalKeyNUMS
// fails here (the bytes are the kind-0 twin's).
func TestComposeLianaReproducesTheRustVectors(t *testing.T) {
	for _, row := range lianaComposeRows() {
		t.Run(row.name, func(t *testing.T) {
			want := loadDescriptor(t, row.name)
			c, err := ComposeWithUnspendable(row.list, make([]*SlotOrigin, want.n), UnspendableLiana)
			if err != nil {
				t.Fatalf("ComposeWithUnspendable: %v", err)
			}
			if !reflect.DeepEqual(c.d.tree, want.tree) {
				t.Fatalf("tree differs from the vendored descriptor.json:\n got %+v\nwant %+v", c.d.tree, want.tree)
			}
			if err := c.Bind(composeTlvPubkeys(want), composeTlvFingerprints(want)); err != nil {
				t.Fatalf("Bind: %v", err)
			}
			gotBytes, _, err := encodePayload(c.d)
			if err != nil {
				t.Fatalf("encodePayload: %v", err)
			}
			if wantBytes := loadBytesHex(t, row.name); !bytes.Equal(gotBytes, wantBytes) {
				t.Fatalf("payload bytes differ:\n got %x\nwant %x", gotBytes, wantBytes)
			}
			gotChunks, err := c.Chunks()
			if err != nil {
				t.Fatalf("Chunks: %v", err)
			}
			if wantChunks := loadPhraseChunks(t, row.name); !reflect.DeepEqual(gotChunks, wantChunks) {
				t.Fatalf("chunks differ:\n got %v\nwant %v", gotChunks, wantChunks)
			}
			if c.UnspendableRequestUnmet() {
				t.Fatal("a met Liana request reports itself unmet")
			}
		})
	}
}

// TestComposeWithIsTheNumsRequest pins that every pre-F-449 caller composes
// what it always composed: ComposeWith == ComposeWithUnspendable(…, Nums).
func TestComposeWithIsTheNumsRequest(t *testing.T) {
	list := lianaComposeRows()[0].list
	a, err := ComposeWith(list, make([]*SlotOrigin, 4))
	if err != nil {
		t.Fatal(err)
	}
	b, err := ComposeWithUnspendable(list, make([]*SlotOrigin, 4), UnspendableNums)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(a.d.tree, b.d.tree) || a.internalKey() != InternalKeyNUMS {
		t.Fatalf("ComposeWith is not the NUMS request: %+v vs %+v", a.d.tree, b.d.tree)
	}
}

// TestALianaRequestOverARealKeyIsUnmet is SPEC §6 row 3 in Go: the first bare
// single key becomes the internal key, the request has nothing to select, and
// the composition says so instead of silently ignoring it.
//
// Mutation: UnspendableRequestUnmet returning false fails the first row;
// returning `c.requested == UnspendableLiana` fails the kofn row.
func TestALianaRequestOverARealKeyIsUnmet(t *testing.T) {
	for _, tc := range []struct {
		name  string
		list  PathList
		kind  UnspendableKind
		unmet bool
		ik    InternalKeyKind
	}{
		{"real key, liana", cpl(ComposeTr, ck(1, 1), clk(ck(1, 1), olderBlocks(26280))), UnspendableLiana, true, InternalKeySlot},
		{"real key, nums", cpl(ComposeTr, ck(1, 1), clk(ck(1, 1), olderBlocks(26280))), UnspendableNums, false, InternalKeySlot},
		{"kofn, liana", cpl(ComposeTr, cu(2, 3), clk(ck(1, 1), olderBlocks(26280))), UnspendableLiana, false, InternalKeyLianaUnspendable},
		{"wsh, liana", cpl(ComposeWsh, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))), UnspendableLiana, true, InternalKeySlot},
	} {
		t.Run(tc.name, func(t *testing.T) {
			n, err := ValidatePathList(tc.list)
			if err != nil {
				t.Fatal(err)
			}
			c, err := ComposeWithUnspendable(tc.list, make([]*SlotOrigin, n), tc.kind)
			if err != nil {
				t.Fatal(err)
			}
			if got := c.UnspendableRequestUnmet(); got != tc.unmet {
				t.Fatalf("UnspendableRequestUnmet = %v, want %v", got, tc.unmet)
			}
			if got := c.internalKey(); got != tc.ik {
				t.Fatalf("built internal key = %d, want %d", got, tc.ik)
			}
		})
	}
}

// TestValidateUnspendableShapeRefusesSpecSixRows is the port of md-codec's
// validate_unspendable_shape, row by row. Row 1 is reachable through the
// composer's API (a sole k-of-n under tr lowers to sortedmulti_a); rows 2 and
// 4 are reached here by editing the built descriptor, because the composer
// cannot produce them -- see TestTheComposerCannotReachSpecSixRowsTwoAndFour.
//
// Mutations: deleting each `return Err…` in validateUnspendableShape fails the
// matching row; isStandardMultipath returning true fails both use-site rows.
func TestValidateUnspendableShapeRefusesSpecSixRows(t *testing.T) {
	compose := func(t *testing.T, list PathList, kind UnspendableKind) Composed {
		t.Helper()
		n, err := ValidatePathList(list)
		if err != nil {
			t.Fatal(err)
		}
		c, err := ComposeWithUnspendable(list, make([]*SlotOrigin, n), kind)
		if err != nil {
			t.Fatal(err)
		}
		return c
	}
	kofn := cpl(ComposeTr, cu(2, 3), clk(ck(1, 1), olderBlocks(26280)))
	if err := compose(t, kofn, UnspendableLiana).ValidateUnspendableShape(); err != nil {
		t.Fatalf("kofn-recovery at kind 1 refused: %v", err)
	}
	if err := compose(t, cpl(ComposeTr, ck(2, 3)), UnspendableNums).ValidateUnspendableShape(); err != nil {
		t.Fatalf("a kind-0 sortedmulti_a refused: %v (the rows bind kind 1 only)", err)
	}
	if err := compose(t, cpl(ComposeTr, ck(2, 3)), UnspendableLiana).ValidateUnspendableShape(); !errors.Is(err, ErrUnspendableSortedMultiA) {
		t.Fatalf("row 1: got %v, want ErrUnspendableSortedMultiA", err)
	}

	c := compose(t, kofn, UnspendableLiana)
	c.d.useSite.multipath = []alternative{{value: 2}, {value: 3}}
	if err := c.ValidateUnspendableShape(); !errors.Is(err, ErrUnspendableUseSite) {
		t.Fatalf("row 2 (shared <2;3>): got %v", err)
	}
	c = compose(t, kofn, UnspendableLiana)
	c.d.tlv.useSiteOverrides = []idxUseSite{{idx: 1, path: useSitePath{hasMultipath: true, multipath: []alternative{{value: 0}, {value: 1}}, wildcardHardened: true}}}
	if err := c.ValidateUnspendableShape(); !errors.Is(err, ErrUnspendableUseSite) {
		t.Fatalf("row 2 (override <0;1>/*h): got %v", err)
	}

	c = compose(t, kofn, UnspendableLiana)
	c.d.tree = node{tag: tagWsh, body: childrenBody{children: []node{c.d.tree}}}
	if err := c.ValidateUnspendableShape(); !errors.Is(err, ErrUnspendableNotRootTr) {
		t.Fatalf("row 4 (wsh(tr(liana))): got %v", err)
	}
}

// TestTheComposerCannotReachSpecSixRowsTwoAndFour pins WHY rows 2 and 4 are
// refusals the composer never meets rather than dead guards: finishComposed
// writes the <0;1>/* use-site and no override, and every lowering puts tr at
// the root. If either ever changes, this fails and the rows become reachable.
//
// Mutation: finishComposed writing {0,1} with wildcardHardened: true fails.
func TestTheComposerCannotReachSpecSixRowsTwoAndFour(t *testing.T) {
	for _, list := range []PathList{
		cpl(ComposeTr, cu(2, 3), clk(ck(1, 1), olderBlocks(26280))),
		cpl(ComposeTr, cu(2, 2), clk(cu(1, 2), olderBlocks(26280))),
		cpl(ComposeTr, ck(2, 3)),
	} {
		n, err := ValidatePathList(list)
		if err != nil {
			t.Fatal(err)
		}
		c, err := ComposeWithUnspendable(list, make([]*SlotOrigin, n), UnspendableLiana)
		if err != nil {
			t.Fatal(err)
		}
		if !isStandardMultipath(c.d.useSite) || c.d.tlv.useSiteOverrides != nil {
			t.Fatalf("the composer wrote a use-site other than <0;1>/*: %+v", c.d.useSite)
		}
		if c.d.tree.tag != tagTr || rejectNestedUnspendable(c.d.tree, true) != nil {
			t.Fatalf("the composer nested a Liana key: %+v", c.d.tree)
		}
	}
}

// TestComposeLianaTemplateEqualsTheHost pins the UNSEATED kofn-recovery Liana
// template -- the chunk the device's stub screen is built from and the plate a
// key-less composition cuts -- to what md-cli 0.19.0 prints for
//
//	md compose --wrapper tr --preset kofn-recovery,2of3,older=26280 --unspendable liana
//	  | md encode --force-chunked --group-size 0
//
// at descriptor-mnemonic cf35d61a (the noCorpusChunks convention: a literal the
// primary printed, dated). Its Template-ID is f99cc42e1ff68bae546c4d1070e5963f,
// the keyed vector's, as SPEC §3e requires (keys do not enter the template id).
//
// Mutation: lowerTr choosing NUMS for the Liana request fails (md1frk8k...).
func TestComposeLianaTemplateEqualsTheHost(t *testing.T) {
	c, err := ComposeWithUnspendable(cpl(ComposeTr, ck(2, 3), clk(ck(1, 1), olderBlocks(26280))), make([]*SlotOrigin, 4), UnspendableLiana)
	if err != nil {
		t.Fatal(err)
	}
	got, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	want := []string{"md13ls8aqqxq6tvyyykjmpprj6tvyy495kcgfwtsqrq0zjqgsexd9dcqqqv65q3cm0m0nz2h7w9"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("chunks differ from md-cli 0.19.0's:\n got %v\nwant %v", got, want)
	}
	id, err := c.TemplateID()
	if err != nil {
		t.Fatal(err)
	}
	if hex.EncodeToString(id[:]) != "f99cc42e1ff68bae546c4d1070e5963f" {
		t.Fatalf("Template-ID %x", id)
	}
}
```

- [ ] **Step 2: Run them and see them fail.**
```bash
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH
go test -count=1 -run 'Liana|Unspendable|ComposeWithIsTheNums|CannotReach' ./md/
```
Expected: FAIL to compile, `undefined: ComposeWithUnspendable`.

- [ ] **Step 3: Implement.**

Create `md/compose_unspendable.go`:

```go
package md

// The composer's unspendable-key REQUEST and SPEC §6's kind-1 mint refusals
// (F-449 stage 4, F-654). A port of md-codec 0.47.0: compose::UnspendableKind
// and compose_with's third argument (compose/mod.rs, compose/tr.rs), and
// validate::validate_unspendable_shape (validate.rs:585-640), which md-cli's
// `md compose --unspendable liana` calls before it emits anything
// (cmd/compose.rs liana_refuse_or_warn).
//
// A REQUEST, NOT A WIRE CONCEPT. InternalKeyKind is what lands on the wire;
// UnspendableKind is what the operator asked for. The Rust primary keeps the
// two apart on purpose (stage 2 plan, Type consistency) and so does this port:
// under a tr whose first bare single-key path becomes a REAL internal key, a
// Liana request has nothing to select, and the built tree says InternalKeySlot.
//
// THE REFUSALS ARE NOT IN encodePayload, AND MUST NEVER MOVE THERE. Reassemble
// re-encodes every decoded card to check its chunk-set id (Reassemble ->
// computeEncodingID -> encodePayload), so a refusal there would make this
// device reject cards the Rust primary decodes; Rust keeps them mint-only for
// the same reason (encode.rs:242-253). They live here, on the composer's
// mint path, and the composer calls them before it builds a chunk.

import (
	"errors"
	"fmt"
)

// UnspendableKind selects the taproot internal key a tr composition uses when
// no path supplies a real one. UnspendableNums is the zero value and the only
// kind the composer built before F-449 stage 4.
type UnspendableKind uint8

const (
	// UnspendableNums is BIP-341's raw H point (wire kind 0).
	UnspendableNums UnspendableKind = iota
	// UnspendableLiana is Liana's unspendable xpub over the composed leaf set
	// (SPEC §2, wire kind 1).
	UnspendableLiana
)

// ComposeWithUnspendable is ComposeWith with the internal-key request md-codec's
// compose_with takes as its third argument. ComposeWith is this with
// UnspendableNums, so every caller that predates F-449 composes what it always
// composed.
func ComposeWithUnspendable(list PathList, declared []*SlotOrigin, unspendable UnspendableKind) (Composed, error) {
	slots, err := ValidatePathList(list)
	if err != nil {
		return Composed{}, err
	}
	if len(declared) != slots {
		return Composed{}, fmt.Errorf("%w: %d given, policy has %d", ErrComposeWrongSlotCount, len(declared), slots)
	}
	c, err := lowerPathList(list, declared, unspendable)
	if err != nil {
		return Composed{}, err
	}
	c.requested = unspendable
	return c, nil
}

// UnspendableRequestUnmet is md-codec's Composed::unspendable_request_unmet: a
// Liana key was requested and the built tree does not carry one. Under tr that
// means a real internal key was extracted (SPEC §6 row 3); under wsh/sh there
// is no internal key at all. Never true for UnspendableNums.
func (c Composed) UnspendableRequestUnmet() bool {
	return c.requested == UnspendableLiana && c.internalKey() != InternalKeyLianaUnspendable
}

// internalKey is the built tree's root taproot internal key, or InternalKeySlot
// for a non-tr root (there is no unspendable key to report).
func (c Composed) internalKey() InternalKeyKind {
	if c.d == nil || c.d.tree.tag != tagTr {
		return InternalKeySlot
	}
	b, ok := c.d.tree.body.(trBody)
	if !ok {
		return InternalKeySlot
	}
	return b.ik
}

var (
	// ErrUnspendableSortedMultiA is SPEC §6 row 1: a belt against a port that
	// feeds MultiALeafScript's derived-key-sorted list into the recipe.
	ErrUnspendableSortedMultiA = errors.New("md: a Liana unspendable key cannot sit over a sortedmulti_a leaf")
	// ErrUnspendableUseSite is SPEC §6 row 2: Liana derives the internal key at
	// 0/i and 1/i, so any other use-site would give the device and Liana two
	// different wallets.
	ErrUnspendableUseSite = errors.New("md: a Liana unspendable key needs the <0;1>/* use-site on every key")
	// ErrUnspendableNotRootTr is SPEC §6 row 4.
	ErrUnspendableNotRootTr = errors.New("md: a Liana unspendable key is allowed only on the root tr()")
)

// ValidateUnspendableShape is md-codec's validate_unspendable_shape over this
// composition: SPEC §6 rows 1, 2 and 4. nil when the tree holds no Liana key.
func (c Composed) ValidateUnspendableShape() error {
	if c.d == nil {
		return nil
	}
	return validateUnspendableShape(c.d)
}

func validateUnspendableShape(d *descriptor) error {
	if err := rejectNestedUnspendable(d.tree, true); err != nil {
		return err
	}
	b, ok := d.tree.body.(trBody)
	if d.tree.tag != tagTr || !ok || b.ik != InternalKeyLianaUnspendable {
		return nil
	}
	if b.tree != nil && containsSortedMultiA(*b.tree) {
		return ErrUnspendableSortedMultiA
	}
	if !isStandardMultipath(d.useSite) {
		return ErrUnspendableUseSite
	}
	for _, o := range d.tlv.useSiteOverrides {
		if !isStandardMultipath(o.path) {
			return ErrUnspendableUseSite
		}
	}
	return nil
}

// isStandardMultipath is md-codec's UseSitePath::standard_multipath(): <0;1>/*
// with no hardening anywhere.
func isStandardMultipath(u useSitePath) bool {
	return u.hasMultipath && !u.wildcardHardened && len(u.multipath) == 2 &&
		u.multipath[0] == (alternative{hardened: false, value: 0}) &&
		u.multipath[1] == (alternative{hardened: false, value: 1})
}

// rejectNestedUnspendable refuses a Liana key on any tr() other than the root,
// recursing through every body that holds children -- the Rust walk's shape.
func rejectNestedUnspendable(n node, isRoot bool) error {
	switch b := n.body.(type) {
	case trBody:
		if b.ik == InternalKeyLianaUnspendable && !isRoot {
			return ErrUnspendableNotRootTr
		}
		if b.tree != nil {
			return rejectNestedUnspendable(*b.tree, false)
		}
	case childrenBody:
		for _, c := range b.children {
			if err := rejectNestedUnspendable(c, false); err != nil {
				return err
			}
		}
	case variableBody:
		for _, c := range b.children {
			if err := rejectNestedUnspendable(c, false); err != nil {
				return err
			}
		}
	}
	return nil
}

// containsSortedMultiA reports a sortedmulti_a node anywhere in n, recursing
// into nested tr() trees as validate.rs does (its fix round 2, M1).
func containsSortedMultiA(n node) bool {
	if n.tag == tagSortedMultiA {
		return true
	}
	switch b := n.body.(type) {
	case trBody:
		return b.tree != nil && containsSortedMultiA(*b.tree)
	case childrenBody:
		for _, c := range b.children {
			if containsSortedMultiA(c) {
				return true
			}
		}
	case variableBody:
		for _, c := range b.children {
			if containsSortedMultiA(c) {
				return true
			}
		}
	}
	return false
}
```

Apply to `md/compose.go`:

```diff
diff --git a/md/compose.go b/md/compose.go
index 2d2443b..9c32ab1 100644
--- a/md/compose.go
+++ b/md/compose.go
@@ -448,6 +448,10 @@ type Composed struct {
 	slots           []ComposeSlot
 	internalKeyPath int // -1 when the internal key is NUMS
 	experimental    []ComposeExperimental
+	// requested is the unspendable kind the caller ASKED for (F-449 stage 4).
+	// What was BUILT is the tree's trBody.ik; the two differ exactly when a
+	// Liana request met a real internal key (UnspendableRequestUnmet).
+	requested UnspendableKind
 }
 
 // Slots is the emitted slot map, index-ascending.
@@ -645,20 +649,16 @@ func Compose(list PathList) (Composed, error) {
 	if err != nil {
 		return Composed{}, err
 	}
-	return lowerPathList(list, make([]*SlotOrigin, slots))
+	return lowerPathList(list, make([]*SlotOrigin, slots), UnspendableNums)
 }
 
 // ComposeWith lowers a list whose slots may carry declared origins (one entry
 // per emitted slot, index-ascending; nil = unseated).
+//
+// It composes the NUMS internal key under tr; ComposeWithUnspendable takes the
+// request (F-449 stage 4).
 func ComposeWith(list PathList, declared []*SlotOrigin) (Composed, error) {
-	slots, err := ValidatePathList(list)
-	if err != nil {
-		return Composed{}, err
-	}
-	if len(declared) != slots {
-		return Composed{}, fmt.Errorf("%w: %d given, policy has %d", ErrComposeWrongSlotCount, len(declared), slots)
-	}
-	return lowerPathList(list, declared)
+	return ComposeWithUnspendable(list, declared, UnspendableNums)
 }
 
 // ─── lowering (the primary's lowering.rs) ─────────────────────────────────────
@@ -905,9 +905,9 @@ func finishComposed(list PathList, declared []*SlotOrigin, tree node, slots []Co
 	return Composed{d: d, slots: slots, internalKeyPath: ik, experimental: exp}, nil
 }
 
-func lowerPathList(list PathList, declared []*SlotOrigin) (Composed, error) {
+func lowerPathList(list PathList, declared []*SlotOrigin, unspendable UnspendableKind) (Composed, error) {
 	if list.Wrapper == ComposeTr {
-		return lowerTr(list, declared)
+		return lowerTr(list, declared, unspendable)
 	}
 	numbered, slots := numberSlots(list, -1)
 	sole := len(list.Paths) == 1
@@ -930,7 +930,7 @@ func lowerPathList(list PathList, declared []*SlotOrigin) (Composed, error) {
 // lowerTr extracts the FIRST-LISTED unlocked, unhashed single key as the
 // internal key (else NUMS); the remaining paths become leaves on a
 // right-leaning spine (depth of leaf j is min(j, m-1)).
-func lowerTr(list PathList, declared []*SlotOrigin) (Composed, error) {
+func lowerTr(list PathList, declared []*SlotOrigin, unspendable UnspendableKind) (Composed, error) {
 	ik := -1
 	for i, p := range list.Paths {
 		if p.isBareSingle() {
@@ -958,12 +958,18 @@ func lowerTr(list PathList, declared []*SlotOrigin) (Composed, error) {
 		}
 		spine = &acc
 	}
-	// ik < 0: no path became the internal key, so it is the NUMS point.
-	// (Stage 3 ports no Liana selection into the composer; that is the port
-	// of md-codec's `--unspendable`, owned by F-449 stage 4.)
-	ikKind := InternalKeyNUMS
-	if ik >= 0 {
-		ikKind = InternalKeySlot
+	// THE ONE DECISION SITE (md-codec compose/tr.rs, F-449 stage 2): no path
+	// supplies a real key, so the request selects which unspendable key. A
+	// real key always wins -- the request then has nothing to select, and
+	// Composed.UnspendableRequestUnmet says so.
+	ikKind := InternalKeySlot
+	if ik < 0 {
+		switch unspendable {
+		case UnspendableLiana:
+			ikKind = InternalKeyLianaUnspendable
+		default:
+			ikKind = InternalKeyNUMS
+		}
 	}
 	tree := node{tag: tagTr, body: trBody{ik: ikKind, keyIndex: 0, tree: spine}}
 	exp := experimentalMarks(list, func(i int) bool { return m == 1 && i != ik && list.Paths[i].isBareMulti() })
```

- [ ] **Step 4: Run them and the package.**
```bash
go vet ./md/ && go test -count=1 ./md/
go test -count=1 -run '^TestCompose' ./gui/
```
Expected: `ok`; md **185** tests (179 + 6). The gui composer tests are unchanged (every gui caller still reaches `ComposeWith`).

- [ ] **Step 5: Commit** `md: the composer's unspendable-key request and SPEC §6's mint refusals (F-449 stage 4, F-654)`.

---

## Task 3 (fork, `md/`): the fourth key-path kind, on the shape and on the Template, and the exported Liana key

**Why.** SPEC §7: "`md.KeyPathNUMS` gains a sibling", and the inspect screen (`md1Summary`, which reads a `md.Template`) must name the kind. Named `KeyPathLianaUnspendable` because the Rust primary named it `KeyPathKind::LianaUnspendable` (F1); the spec's `NumsXpub` is reconciled in Task 9. **Appended**, so the three existing values keep their numbers. One mapping, `keyPathOf`, feeds both `policyShape` and `summarize`, so the consent screen and the inspect screen cannot disagree. A kind with no name makes the shape INCOMPLETE (the package's honesty contract), never a guessed neighbour. `LianaUnspendableKeyFor` exports stage 3's test-only recipe (`md/liana.go`) for Task 4's deriver, over keys the CALLER supplies, walked in the tree's key-occurrence order (`collectKeyOccurrences`, the walk `lianaLeafPubkeys` uses). It does not read the card's Pubkeys TLV: on the Wallet Policy route the md1 is a key-less template and the leaf keys are the seated mk1 cards (R0 I1). From this task on the recipe is live firmware code (it was dead-code-eliminated at stage 3; Task 8's size delta includes it).

**Files:** Create `md/keypath_liana_test.go`; Modify `md/policy_shape.go` (`:30-40`, `:121-130`), `md/md.go` (`Template` `:1278-1299`, `summarize` `:1438-1463`), `md/liana.go` (append).

**Interfaces:**
- Consumes: Task 2's `ComposeWithUnspendable` (tests only); stage 3's `lianaLeafPubkeys`, `lianaUnspendableKey`.
- Produces: `md.KeyPathLianaUnspendable`; `func keyPathOf(InternalKeyKind) (KeyPathKind, bool)`; `func rootKeyPath(node) KeyPathKind`; field `md.Template.KeyPath KeyPathKind`; `func LianaUnspendableKeyFor(strs []string, xpubs map[uint8][65]byte) ([65]byte, error)` (chain code ‖ compressed H; errors if not kind 1 or a leaf's key is missing).

- [ ] **Step 1: Write the failing tests.**

Create `md/keypath_liana_test.go`:

```go
package md

import "testing"

// F-449 stage 4: SPEC §7's sibling key-path kind on the Go side, and the
// exported Liana key the device's address deriver reads.

// TestKeyPathNamesTheLianaKind is SPEC §7's Go sibling kind, on both the
// structural walk and the Template the inspect screen reads.
//
// Mutations: keyPathOf mapping InternalKeyLianaUnspendable to KeyPathNUMS fails
// the kind-1 rows; rootKeyPath returning KeyPathNone fails the Template rows.
func TestKeyPathNamesTheLianaKind(t *testing.T) {
	for _, tc := range []struct {
		name string
		want KeyPathKind
	}{
		{"keyed_tr_liana_kofn_recovery", KeyPathLianaUnspendable},
		{"keyed_compose_preset_kofn_recovery", KeyPathNUMS},
	} {
		chunks := vectorChunksFor(t, tc.name)
		shape, err := PolicyShapeChunks(chunks)
		if err != nil {
			t.Fatal(err)
		}
		if !shape.Complete || shape.KeyPath != tc.want {
			t.Fatalf("%s: PolicyShape KeyPath = %d (complete %v), want %d", tc.name, shape.KeyPath, shape.Complete, tc.want)
		}
		tpl, _, err := ExpandWalletPolicyChunks(chunks)
		if err != nil {
			t.Fatal(err)
		}
		if tpl.KeyPath != tc.want {
			t.Fatalf("%s: Template.KeyPath = %d, want %d", tc.name, tpl.KeyPath, tc.want)
		}
	}
}

// TestAnUnknownInternalKeyKindIsNotDescribed: a kind keyPathOf has no name for
// makes the whole summary incomplete, never a neighbouring kind (SPEC §7a row
// 1's failure, one layer up). No decoder yields such a kind today -- the
// version set {4, 8} admits three -- so the tree is built by hand.
//
// Mutation: keyPathOf's fall-through returning (KeyPathNUMS, true) fails.
func TestAnUnknownInternalKeyKindIsNotDescribed(t *testing.T) {
	leaf := node{tag: tagPkK, body: keyArgBody{index: 0}}
	tree := node{tag: tagTr, body: trBody{ik: InternalKeyKind(3), tree: &leaf}}
	if s := policyShape(tree); s.Complete {
		t.Fatalf("an unknown internal-key kind was described: %+v", s)
	}
	if kp := rootKeyPath(tree); kp != KeyPathNone {
		t.Fatalf("rootKeyPath named an unknown kind %d", kp)
	}
}

// TestLianaUnspendableKeyForIsTheRecipeOverTheSuppliedLeaves binds the
// exported entry the device's address deriver calls to the recipe the Go leg
// of SPEC §8.1 already proved against Liana's goldens -- over the keyed card
// AND over its key-less template with the same keys supplied (the Wallet
// Policy template + mk1 cards route, R0 I1).
//
// Mutation: returning lianaUnspendableKey(nil) fails the first comparison.
// (Visiting the keys in index order instead of occurrence order is inert by
// construction: canonical numbering is first-occurrence -- stage 3's M6 note.)
func TestLianaUnspendableKeyForIsTheRecipeOverTheSuppliedLeaves(t *testing.T) {
	chunks := vectorChunksFor(t, "keyed_tr_liana_kofn_recovery")
	_, keys, err := ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatal(err)
	}
	xpubs := map[uint8][65]byte{}
	for _, k := range keys {
		xpubs[k.Index] = k.Xpub
	}
	got, err := LianaUnspendableKeyFor(chunks, xpubs)
	if err != nil {
		t.Fatal(err)
	}
	d, err := Reassemble(chunks)
	if err != nil {
		t.Fatal(err)
	}
	pks, err := lianaLeafPubkeys(d)
	if err != nil {
		t.Fatal(err)
	}
	if want := lianaUnspendableKey(pks); got != want || len(pks) != 4 {
		t.Fatalf("got %x, want %x over %d leaves", got, want, len(pks))
	}
	tmpl, err := StripToTemplate(chunks)
	if err != nil {
		t.Fatal(err)
	}
	if fromTemplate, err := LianaUnspendableKeyFor(tmpl, xpubs); err != nil || fromTemplate != got {
		t.Fatalf("over the key-less template with the same keys: %x, %v; want %x", fromTemplate, err, got)
	}
	delete(xpubs, 3)
	if _, err := LianaUnspendableKeyFor(tmpl, xpubs); err == nil {
		t.Fatal("a missing leaf key yielded a Liana key")
	}
	if _, err := LianaUnspendableKeyFor(vectorChunksFor(t, "keyed_compose_preset_kofn_recovery"), nil); err == nil {
		t.Fatal("a kind-0 set yielded a Liana key")
	}
}
```

- [ ] **Step 2: Run and see them fail.** `go test -count=1 -run 'KeyPath|UnknownInternal|LianaUnspendableKeyFor' ./md/` — Expected: FAIL to compile, `undefined: KeyPathLianaUnspendable`.

- [ ] **Step 3: Implement.**

Apply to `md/policy_shape.go`:

```diff
diff --git a/md/policy_shape.go b/md/policy_shape.go
index 4c0172d..4e65e38 100644
--- a/md/policy_shape.go
+++ b/md/policy_shape.go
@@ -37,8 +37,32 @@ const (
 	// KeyPathSpendable — a real key can spend directly, WITHOUT satisfying any
 	// leaf. This is the condition a leaf-only summary would have hidden.
 	KeyPathSpendable
+	// KeyPathLianaUnspendable — wire kind 1 (F-449): Liana's unspendable xpub,
+	// derived from the leaf keys (SPEC §2). Script paths only, exactly as
+	// KeyPathNUMS, and a DIFFERENT WALLET from the same tree at KeyPathNUMS:
+	// different addresses, different ids. Named as md-codec 0.47.0 names it
+	// (policy_shape.rs KeyPathKind::LianaUnspendable), not the spec's earlier
+	// "NumsXpub". APPENDED, so the three older values keep their numbers.
+	KeyPathLianaUnspendable
 )
 
+// keyPathOf is the ONE mapping from a taproot internal key to its KeyPathKind,
+// shared by policyShape and summarize so the inspect screen and the consent
+// screen cannot disagree about which kind a card carries. ok=false for a kind
+// this port has no name for: the caller must treat the policy as one it cannot
+// describe, never guess a neighbouring kind (SPEC §7a row 1).
+func keyPathOf(ik InternalKeyKind) (KeyPathKind, bool) {
+	switch ik {
+	case InternalKeySlot:
+		return KeyPathSpendable, true
+	case InternalKeyNUMS:
+		return KeyPathNUMS, true
+	case InternalKeyLianaUnspendable:
+		return KeyPathLianaUnspendable, true
+	}
+	return KeyPathNone, false
+}
+
 // Branch is one independently satisfiable spend path: a tapscript leaf, or the
 // whole script for wsh/sh.
 type Branch struct {
@@ -123,11 +147,13 @@ func policyShape(tree node) PolicyShape {
 		if !ok {
 			return PolicyShape{}
 		}
-		if b.isNums() {
-			s.KeyPath = KeyPathNUMS
-		} else {
-			s.KeyPath = KeyPathSpendable
+		kp, known := keyPathOf(b.ik)
+		if !known {
+			// THE HONESTY CONTRACT, applied to the key path: a summary that
+			// named a kind it does not know would describe a different wallet.
+			return PolicyShape{}
 		}
+		s.KeyPath = kp
 		if b.tree != nil {
 			walkTapTree(*b.tree, 1, &s)
 		}
```

Apply to `md/md.go`:

```diff
diff --git a/md/md.go b/md/md.go
index 75fff70..25864f4 100644
--- a/md/md.go
+++ b/md/md.go
@@ -1296,6 +1296,9 @@ type Template struct {
 	// single-sig sh root, symmetric with InnerWsh for the sorted-multi sh root.
 	// Meaningful only when Root==ScriptSh && Policy==PolicySingle.
 	InnerWpkh bool
+	// KeyPath is the root taproot internal key's kind (F-449 stage 4, SPEC §7:
+	// the inspect screen must name it), or KeyPathNone for a non-tr root.
+	KeyPath KeyPathKind
 }
 
 // Decode decodes a single-string md1 descriptor into a Template. It refuses
@@ -1459,9 +1462,22 @@ func summarize(d *descriptor) Template {
 		Renderable: renderable,
 		InnerWsh:   innerWshNesting(d.tree),
 		InnerWpkh:  innerWpkhNesting(d.tree),
+		KeyPath:    rootKeyPath(d.tree),
 	}
 }
 
+// rootKeyPath is keyPathOf for a root tr(), KeyPathNone otherwise -- including
+// for a kind keyPathOf has no name for, which summarize then reports as no key
+// path rather than as a guessed one.
+func rootKeyPath(tree node) KeyPathKind {
+	b, ok := tree.body.(trBody)
+	if tree.tag != tagTr || !ok {
+		return KeyPathNone
+	}
+	kp, _ := keyPathOf(b.ik)
+	return kp
+}
+
 // fingerprintFor returns the 8-hex lowercase fingerprint for @idx, or "".
 func fingerprintFor(d *descriptor, idx uint8) string {
 	if !d.tlv.fpPresent {
```

Apply to `md/liana.go`:

```diff
diff --git a/md/liana.go b/md/liana.go
index 4b17724..9dcbe19 100644
--- a/md/liana.go
+++ b/md/liana.go
@@ -94,3 +94,47 @@ func collectKeyOccurrences(n node, out *[]uint8) {
 		}
 	}
 }
+
+// LianaUnspendableKeyFor is the 65-byte key material (chain code ‖ the
+// compressed H point) of a kind-1 chunk set's internal key: SPEC §2 over the
+// leaf keys the CALLER supplies, keyed by placeholder index, taken in the
+// tree's key-occurrence order (collectKeyOccurrences, the walk
+// lianaLeafPubkeys uses). The device derives the key path at 0/i and 1/i from
+// it (SPEC §2, "derivation, not just rendering"), whatever the wallet's
+// use-site -- §6 row 2 refuses any other use-site at mint.
+//
+// THE KEYS COME FROM THE CALLER, NOT FROM THE CARD'S Pubkeys TLV (F-449 stage
+// 4 R0 I1). SPEC §7a.2 says the recipe runs "over the collected leaf keys", and
+// on the Wallet Policy route those are the seated mk1 key cards while the md1
+// is a key-less TEMPLATE with no TLV at all. Reading the TLV made every
+// template-plus-cards kind-1 wallet underivable. A keyed card's own keys reach
+// here the same way: ExpandWalletPolicyChunks reads them out of its TLV first.
+//
+// An error for a set that is not kind 1, or when a leaf's key was not supplied:
+// the recipe needs every real leaf key, and there is no fallback.
+func LianaUnspendableKeyFor(strs []string, xpubs map[uint8][65]byte) ([65]byte, error) {
+	d, err := Reassemble(strs)
+	if err != nil {
+		return [65]byte{}, err
+	}
+	b, ok := d.tree.body.(trBody)
+	if d.tree.tag != tagTr || !ok || b.ik != InternalKeyLianaUnspendable {
+		return [65]byte{}, errLianaNotKind1
+	}
+	if b.tree == nil {
+		return [65]byte{}, errLianaNoLeaves
+	}
+	var idx []uint8
+	collectKeyOccurrences(*b.tree, &idx)
+	pks := make([][33]byte, 0, len(idx))
+	for _, i := range idx {
+		x, ok := xpubs[i]
+		if !ok {
+			return [65]byte{}, errLianaMissingKey
+		}
+		var pk [33]byte
+		copy(pk[:], x[32:65])
+		pks = append(pks, pk)
+	}
+	return lianaUnspendableKey(pks), nil
+}
```

- [ ] **Step 4: Run.** `go vet ./md/ && go test -count=1 ./md/` → `ok`, **188** tests. Then `go test -count=1 -run 'TestPolicy|TestMd1|TestTemplate' ./gui/` → `ok` (no existing caller switches on a `KeyPath` value the old code could not produce).

- [ ] **Step 5: Commit** `md: KeyPathLianaUnspendable, Template.KeyPath, LianaUnspendableKeyFor (F-449 stage 4, SPEC §7)`.

---

## Task 4 (fork, `gui/`): the device derives the Liana key (§7a.2) and refuses a kind it cannot (§7a.3, address half)

**Why.** Stage 3 left `complexAddressDeriver`'s switch (`gui/policy_address.go:157-172` at `d2350cb`) with a `default:` that refuses kind 1, so kind-1 cards showed no address (the two `stillUnsupported` entries). This task adds the third arm. The switch moves into `taprootInternalKey` so the refusal of an UNKNOWN kind can be tested directly (no decoder yields one). The Liana key is computed ONCE per card, before any index, from the `keys` the deriver is given (never the md1's TLV, R0 I1), and derived with an EXPLICIT `<0;1>/*` (SPEC §2), never from the wallet's use-site.

**§8.3's device leg, three-way.** md vs device: `TestEveryKeyedVectorReachesAnAddress` over the two keyed kind-1 vectors (Rust md's addresses, 6 each), once their `stillUnsupported` entries are deleted. Liana vs device: `TestDeviceDerivesLianasOwnAddressesForKind1`, which builds each of the five ACCEPTED evidence wallets through the composer (`md.ComposeWithUnspendable` → `Bind` → `Chunks`) and compares receive and change 0..2 with `liana_receive`/`liana_change` in `md/testdata/liana_cases.json` (Liana v15.0's own output, pinned by stage 3). md vs Liana is Rust's (stage 1b). It also asserts the kind-0 twin derives a DIFFERENT receive 0 (§8.3's last sentence).

**The template + key cards route (R0 I1).** The composer's multi-party output is a key-less TEMPLATE plate plus one mk1 card per cosigner, and the operator proves it on Wallet Policy: `walletPolicyConsentLines(template, cards)` seats the cards into `keys` and hands the TEMPLATE as `collected`. The first draft computed the Liana key from the md1's Pubkeys TLV, which a template does not have, so this route showed "This device can't derive addresses for this policy." while the kind-0 twin derived. `TestTemplatePlusKeyCardsDerivesTheLianaWallet` pins the fix: over `keyed_tr_liana_kofn_recovery` stripped to its template plus one mk1 card per slot, the consent shows Rust md 0.19.0's receive 0 (`bc1py0prh5dupvw3egy8acnvrh0kdnma9ywvrlza0547a4lzvw5sgmjqw56q3f`), as it does for the kind-0 twin and for `keyed_tr_with_leaf`. **Other key sources, checked:** `lianaInternalKey` is the only production caller of the recipe (`grep -rn "LianaUnspendableKey\|lianaLeafPubkeys\|lianaInternalKey(" gui md --include='*.go' | grep -v _test.go`); both routes into it (the composer's consent over its keyed chunks, and Wallet Policy over template + cards) pass the same `keys` the leaf scripts are built from.

**Files:** Create `gui/policy_address_liana_test.go`; Modify `gui/policy_address.go`, `gui/key_card_seating_test.go` (`seatFixture` generalised to `seatFixtureFor(t, vector)`), `gui/policy_address_test.go` (`stillUnsupported`, `:128-137`), `gui/taproot_script_path_test.go` (the arm at `:56-62`: stage 3's comment said the arm is unreachable and pointed at stage 4; it now says where kind 1 IS gated).

**Interfaces:**
- Consumes: Task 3's `md.LianaUnspendableKeyFor`; stage 3's three-state `md.EmitTapLeavesChunks`.
- Produces: `func taprootInternalKey(ik md.InternalKeyKind, ikIndex uint8, byIndex map[uint8]bip380.Key, liana *bip380.Key, index uint32, change bool) (*secp256k1.PublicKey, error)`; `func lianaInternalKey(collected []string, keys []md.ExpandedKey, network *chaincfg.Params) (bip380.Key, error)`; test helper `seatFixtureFor(t, vector) (template []string, cards []mk.Card, receive0 string)`.

- [ ] **Step 1: Write the failing tests** (and empty `stillUnsupported`, which is itself a failing test until the arm exists).

Create `gui/policy_address_liana_test.go`:

```go
package gui

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"testing"

	"seedhammer.com/md"
)

// SPEC §8.3's DEVICE LEG (F-449 stage 4): receive and change 0..2 derived by
// THIS firmware, for a wallet THIS firmware's composer built, equal the
// addresses LIANA ITSELF recorded for the same wallet -- Liana v15.0's own
// `LianaDescriptor` receive/change output, vendored from the Rust primary as
// md/testdata/liana_cases.json (stage 3, pinned by md/liana_test.go). The
// md leg of the same three-way equality is Rust's (stage 1b); the device-to-md
// leg is TestEveryKeyedVectorReachesAnAddress over the two keyed kind-1 vectors.
//
// THROUGH THE PRODUCTION PATH, end to end: md.ComposeWithUnspendable (the call
// composerArtifactsFor makes), Bind, Chunks, then complexAddressSource -- the
// gated entry every screen uses. No step is a test double.

type lianaDeviceCase struct {
	Name         string   `json:"name"`
	Accepted     bool     `json:"accepted"`
	LeafTLVHex   []string `json:"leaf_tlv_hex"`
	Descriptor   string   `json:"descriptor_with_checksum"`
	LianaReceive []string `json:"liana_receive"`
	LianaChange  []string `json:"liana_change"`
}

// lianaCaseLists is each ACCEPTED case as the composer path list that lowers to
// its tree. Every accepted case is composable -- a flat or right-spined tree of
// multi_a / pk leaves -- which is itself a fact this test pins: a sixth accepted
// case with no entry here fails rather than being skipped.
var lianaCaseLists = map[string]md.PathList{
	"preset-kofn-recovery-tr": {Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3}}, lianaLocked(1, 1, 26280)}},
	"preset-tiered-recovery-tr": {Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 2}}, lianaLocked(1, 2, 26280)}},
	"same-seed-two-paths-tr": {Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3}}, lianaLocked(1, 1, 26280)}},
	"X19-tr-kofn-nums-older5": {Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3}}, lianaLocked(1, 1, 5)}},
	"nested-2of2-two-recoveries-tr": {Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 2}}, lianaLocked(1, 1, 26280), lianaLocked(1, 1, 52560)}},
}

func lianaLocked(k, n uint8, blocks uint32) md.SpendPath {
	return md.SpendPath{Keys: &md.KeySet{K: k, N: n}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: blocks}}
}

var lianaOriginRE = regexp.MustCompile(`\[([0-9a-f]{8})((?:/[0-9]+'?)+)\]`)

// lianaCaseChunks builds the case's KEYED md1 through the composer, at `kind`.
func lianaCaseChunks(t *testing.T, c lianaDeviceCase, kind md.UnspendableKind) []string {
	t.Helper()
	list, ok := lianaCaseLists[c.Name]
	if !ok {
		t.Fatalf("accepted case %s has no composer path list -- add one, never skip it", c.Name)
	}
	origins := lianaOriginRE.FindAllStringSubmatch(c.Descriptor, -1)
	if len(origins) != len(c.LeafTLVHex) {
		t.Fatalf("%s: %d origins for %d leaf keys", c.Name, len(origins), len(c.LeafTLVHex))
	}
	declared := make([]*md.SlotOrigin, len(origins))
	pubs := map[uint8][65]byte{}
	fps := map[uint8][4]byte{}
	for i, o := range origins {
		var fp [4]byte
		if _, err := hex.Decode(fp[:], []byte(o[1])); err != nil {
			t.Fatal(err)
		}
		var comps []md.PathComponent
		for _, p := range strings.Split(strings.TrimPrefix(o[2], "/"), "/") {
			hard := strings.HasSuffix(p, "'")
			v, err := strconv.ParseUint(strings.TrimSuffix(p, "'"), 10, 32)
			if err != nil {
				t.Fatal(err)
			}
			comps = append(comps, md.PathComponent{Hardened: hard, Value: uint32(v)})
		}
		declared[i] = &md.SlotOrigin{Origin: comps, Fingerprint: fp, FpPresent: true}
		raw, err := hex.DecodeString(c.LeafTLVHex[i])
		if err != nil || len(raw) != 65 {
			t.Fatalf("%s: leaf %d TLV: %v (len %d)", c.Name, i, err, len(raw))
		}
		var b [65]byte
		copy(b[:], raw)
		pubs[uint8(i)] = b
		fps[uint8(i)] = fp
	}
	composed, err := md.ComposeWithUnspendable(list, declared, kind)
	if err != nil {
		t.Fatalf("%s: compose: %v", c.Name, err)
	}
	if err := composed.Bind(pubs, fps); err != nil {
		t.Fatalf("%s: Bind: %v", c.Name, err)
	}
	chunks, err := composed.Chunks()
	if err != nil {
		t.Fatalf("%s: Chunks: %v", c.Name, err)
	}
	return chunks
}

// TestDeviceDerivesLianasOwnAddressesForKind1 is SPEC §8.3's device leg.
//
// Mutations, each measured: taprootInternalKey's Liana arm returning
// address.NUMSInternalKey() (§7a row 1) fails every case at receive 0;
// lianaInternalKey deriving under the wallet's use-site instead of <0;1>
// is inert here (every wallet is <0;1>, which is §6 row 2's point); the
// Range End set to 2 fails at change 0.
func TestDeviceDerivesLianasOwnAddressesForKind1(t *testing.T) {
	raw, err := os.ReadFile(filepath.Join("..", "md", "testdata", "liana_cases.json"))
	if err != nil {
		t.Fatalf("INCONCLUSIVE: %v", err)
	}
	var cases []lianaDeviceCase
	if err := json.Unmarshal(raw, &cases); err != nil {
		t.Fatal(err)
	}
	accepted := 0
	for _, c := range cases {
		if !c.Accepted {
			continue
		}
		accepted++
		t.Run(c.Name, func(t *testing.T) {
			if len(c.LianaReceive) != 3 || len(c.LianaChange) != 3 {
				t.Fatalf("the case records %d receive / %d change addresses, want 3 and 3",
					len(c.LianaReceive), len(c.LianaChange))
			}
			chunks := lianaCaseChunks(t, c, md.UnspendableLiana)
			_, keys, err := md.ExpandWalletPolicyChunks(chunks)
			if err != nil {
				t.Fatal(err)
			}
			at, ok := complexAddressSource(chunks, keys)
			if !ok {
				t.Fatal("the device derives no address for a kind-1 wallet Liana imports")
			}
			for i := 0; i < 3; i++ {
				for _, leg := range []struct {
					change bool
					want   string
				}{{false, c.LianaReceive[i]}, {true, c.LianaChange[i]}} {
					got, err := at(uint32(i), leg.change)
					if err != nil {
						t.Fatalf("index %d change=%v: %v", i, leg.change, err)
					}
					if got != leg.want {
						t.Fatalf("index %d change=%v:\n device %s\n liana  %s", i, leg.change, got, leg.want)
					}
				}
			}
			// THE SAME TREE AT KIND 0 IS A DIFFERENT WALLET (§8.3's last
			// sentence): the device must not show Liana's addresses for it.
			nums := lianaCaseChunks(t, c, md.UnspendableNums)
			_, nkeys, err := md.ExpandWalletPolicyChunks(nums)
			if err != nil {
				t.Fatal(err)
			}
			nat, ok := complexAddressSource(nums, nkeys)
			if !ok {
				t.Fatal("the kind-0 twin derives no address")
			}
			if a, _ := nat(0, false); a == c.LianaReceive[0] {
				t.Fatalf("kind 0 and kind 1 derive the same receive 0 %s", a)
			}
		})
	}
	// Five at descriptor-mnemonic cf35d61a. A count, not ">0".
	if accepted != 5 {
		t.Fatalf("%d accepted Liana cases, want 5", accepted)
	}
}

// TestAnUnderivableInternalKeyKindIsRefusedNotFallenBack is SPEC §7a.3 at the
// device: a kind this firmware has no arm for yields an error, never the NUMS
// point. No decoder yields such a kind today, so the kind is constructed.
//
// Mutation: adding `default: return address.NUMSInternalKey()` fails; so does
// grouping the Liana arm with NUMS (`case md.InternalKeyNUMS,
// md.InternalKeyLianaUnspendable:`), via the nil-liana row.
func TestAnUnderivableInternalKeyKindIsRefusedNotFallenBack(t *testing.T) {
	if _, err := taprootInternalKey(md.InternalKeyKind(3), 0, nil, nil, 0, false); err != errUnderivableInternalKey {
		t.Fatalf("an unknown kind derived (err %v)", err)
	}
	if _, err := taprootInternalKey(md.InternalKeyLianaUnspendable, 0, nil, nil, 0, false); err != errUnderivableInternalKey {
		t.Fatalf("a kind-1 key with no computed Liana key derived (err %v)", err)
	}
}

// TestTemplatePlusKeyCardsDerivesTheLianaWallet is R0 I1's route: the
// composer's usual multi-party output is a key-less TEMPLATE plate plus one mk1
// key card per cosigner, and an operator proves that wallet by bringing all of
// them to Wallet Policy. The consent there must show Rust md 0.19.0's receive 0
// for the kind-1 wallet, exactly as it does for the kind-0 twin and for a
// tr wallet with a real key path.
//
// Mutation: lianaInternalKey building its map from the md1's OWN keys
// (md.ExpandWalletPolicyChunks(collected), which on a template carry no xpub)
// instead of `keys` -- the TLV read the first draft did -- fails the kind-1 row.
func TestTemplatePlusKeyCardsDerivesTheLianaWallet(t *testing.T) {
	for _, vec := range []string{"keyed_tr_liana_kofn_recovery", "keyed_compose_preset_kofn_recovery", "keyed_tr_with_leaf"} {
		t.Run(vec, func(t *testing.T) {
			tmpl, cards, want := seatFixtureFor(t, vec)
			if _, keys, err := md.ExpandWalletPolicyChunks(tmpl); err != nil || allSlotsHaveXpub(keys) {
				t.Fatalf("the fixture's md1 is not a key-less template (err %v)", err)
			}
			lines, err := walletPolicyConsentLines(tmpl, cards)
			if err != nil {
				t.Fatalf("consent refused: %v", err)
			}
			if joined := strings.Join(lines, "\n"); !strings.Contains(joined, want) {
				t.Fatalf("the consent does not show Rust's receive 0 %s:\n%s", want, joined)
			}
		})
	}
}
```

Apply to `gui/key_card_seating_test.go`:

```diff
diff --git a/gui/key_card_seating_test.go b/gui/key_card_seating_test.go
index 7b6eed9..79a8253 100644
--- a/gui/key_card_seating_test.go
+++ b/gui/key_card_seating_test.go
@@ -21,6 +21,13 @@ const seatVector = "keyed_tr_with_leaf"
 
 // seatFixture returns (templateCards, keyCards, expectedReceive0).
 func seatFixture(t *testing.T) ([]string, []mk.Card, string) {
+	t.Helper()
+	return seatFixtureFor(t, seatVector)
+}
+
+// seatFixtureFor is seatFixture over any keyed vector (F-449 stage 4 seats a
+// kind-1 wallet this way, R0 I1).
+func seatFixtureFor(t *testing.T, seatVector string) ([]string, []mk.Card, string) {
 	t.Helper()
 	keyed := loadVectorChunks(t, seatVector)
 	tmpl, err := md.StripToTemplate(keyed)
```

Apply to `gui/policy_address_test.go`:

```diff
diff --git a/gui/policy_address_test.go b/gui/policy_address_test.go
index 802189e..3b48ffd 100644
--- a/gui/policy_address_test.go
+++ b/gui/policy_address_test.go
@@ -126,14 +126,10 @@ func TestEveryKeyedVectorReachesAnAddress(t *testing.T) {
 	// test that lets an undeliverable shape pass quietly is how "display only"
 	// outlives the reason for it. Adding a name here must be a deliberate act.
 	stillUnsupported := map[string]string{
-		// F-449 stage 3 ships the codec half (md decodes kind 1 and reports it
-		// three-state); the device's kind-1 DERIVATION is stage 4 (SPEC §7a.2).
-		// Until then SPEC §7a.3 requires NO address rather than the NUMS
-		// branch's addresses of a different wallet -- and the F-613 mirror
-		// below proves no deriver produces one. Stage 4 deletes both entries:
-		// this test FAILS, naming them, the day the device derives kind 1.
-		"keyed_tr_liana_kofn_recovery":         "Liana-unspendable internal key: device derivation is F-449 stage 4",
-		"keyed_tr_liana_nested_two_recoveries": "Liana-unspendable internal key: device derivation is F-449 stage 4",
+		// EMPTY SINCE F-449 STAGE 4: the two kind-1 vectors derive through
+		// taprootInternalKey's Liana arm and are checked against Rust below
+		// like every other vector. The map stays, because a future
+		// capability gap must be a deliberate entry here, not a silent pass.
 	}
 
 	// Shapes this device CAN derive and DECLINES to (F-531). A separate map
```

Apply to `gui/taproot_script_path_test.go`:

```diff
diff --git a/gui/taproot_script_path_test.go b/gui/taproot_script_path_test.go
index 366891a..ecdc488 100644
--- a/gui/taproot_script_path_test.go
+++ b/gui/taproot_script_path_test.go
@@ -54,13 +54,14 @@ func TestTaprootScriptPathMatchesRust(t *testing.T) {
 			case md.InternalKeyNUMS:
 				t.Skip("NUMS internal key: no key-path spend, not this gate's subject")
 			case md.InternalKeyLianaUnspendable:
-				// F-449: the device's kind-1 derivation is stage 4 (SPEC §7a.2).
-				// FUTURE-PROOFING ONLY at stage 3 (R0 n1): both kind-1 vectors
-				// never reach this arm, because md.TapLeavesChunks refuses their
-				// leaf shapes first and they skip above. The real gate is
-				// TestEveryKeyedVectorReachesAnAddress's stillUnsupported entries,
-				// which FAIL the day stage 4 derives kind 1.
-				t.Skip("Liana-unspendable internal key: device derivation is F-449 stage 4")
+				// UNREACHABLE for both kind-1 vectors (stage 3 R0 n1):
+				// md.TapLeavesChunks refuses their leaf shapes first and they
+				// skip above. Kind 1 has no key-path SLOT for
+				// address.TaprootScriptPath anyway; its internal key is SPEC §2's
+				// recipe, checked against Rust by
+				// TestEveryKeyedVectorReachesAnAddress and against Liana's own
+				// addresses by TestDeviceDerivesLianasOwnAddressesForKind1.
+				t.Skip("Liana-unspendable internal key: not this gate's subject (see the two tests named above)")
 			default:
 				t.Fatalf("%s: internal-key kind %d is unknown to this gate", name, ik)
 			}
```

- [ ] **Step 2: Run and see them fail.**
```bash
go test -count=1 -run 'TestDeviceDerives|TestTemplatePlusKeyCards|TestEveryKeyedVector|TestAnUnderivable' ./gui/
```
Expected: FAIL to compile (`undefined: taprootInternalKey`). With only the test half of `TestEveryKeyedVectorReachesAnAddress` applied it fails naming `keyed_tr_liana_kofn_recovery ... reaches NO address route`.

- [ ] **Step 3: Implement.**

Apply to `gui/policy_address.go`:

```diff
diff --git a/gui/policy_address.go b/gui/policy_address.go
index 37ae125..2ead6b0 100644
--- a/gui/policy_address.go
+++ b/gui/policy_address.go
@@ -134,6 +134,19 @@ func complexAddressDeriver(collected []string, keys []md.ExpandedKey) (func(uint
 		probe[i] = make([]byte, 32)
 	}
 	if ikIndex, ik, _, err := md.EmitTapLeavesChunks(collected, probe); err == nil {
+		// THE LIANA KEY IS COMPUTED ONCE, before any index is asked for: it is
+		// SPEC §2's recipe over the leaf keys this deriver was GIVEN -- `keys`,
+		// the same keys every leaf script below is built from -- which do not
+		// change with the address index. A kind-1 set whose recipe cannot be
+		// computed has no address, never the NUMS branch's (SPEC §7a row 1).
+		var liana *bip380.Key
+		if ik == md.InternalKeyLianaUnspendable {
+			lk, lerr := lianaInternalKey(collected, keys, network)
+			if lerr != nil {
+				return nil, false
+			}
+			liana = &lk
+		}
 		src = func(index uint32, change bool) (string, error) {
 			xonly := make(map[uint8][]byte, len(byIndex))
 			for i, k := range byIndex {
@@ -153,23 +166,7 @@ func complexAddressDeriver(collected []string, keys []md.ExpandedKey) (func(uint
 			for _, l := range leaves {
 				scripts = append(scripts, address.LeafScript{Depth: l.Depth, Script: l.Script})
 			}
-			var ikey *secp256k1.PublicKey
-			switch ik {
-			case md.InternalKeyNUMS:
-				ikey, err = address.NUMSInternalKey()
-			case md.InternalKeySlot:
-				internal, iok := byIndex[ikIndex]
-				if !iok {
-					return "", errors.New("gui: taproot internal key has no @N entry")
-				}
-				ikey, err = address.DeriveChild(internal, index, change)
-			default:
-				// SPEC §7a.3: a kind this firmware cannot derive gets NO address
-				// -- never the NUMS branch, which would show a DIFFERENT
-				// wallet's addresses. The probe below turns this error into
-				// "no address source". Liana-unspendable derivation is stage 4.
-				return "", errUnderivableInternalKey
-			}
+			ikey, err := taprootInternalKey(ik, ikIndex, byIndex, liana, index, change)
 			if err != nil {
 				return "", err
 			}
@@ -202,6 +199,69 @@ func complexAddressDeriver(collected []string, keys []md.ExpandedKey) (func(uint
 	return src, true
 }
 
+// taprootInternalKey is the ONE switch over the internal-key kinds this
+// firmware derives. SPEC §7a has two rules and this function is both of them:
+//
+//   - kind 1 derives Liana's unspendable xpub at 0/index (receive) or 1/index
+//     (change), from `liana` -- never the raw H point, which is a different
+//     wallet's key (§7a row 1, the silent-wrong-address failure);
+//   - a kind with no arm here is REFUSED (§7a.3). No default falls back to
+//     NUMS, so a fourth kind is an error the day it exists rather than a
+//     well-formed address for somebody else's wallet.
+func taprootInternalKey(ik md.InternalKeyKind, ikIndex uint8, byIndex map[uint8]bip380.Key, liana *bip380.Key, index uint32, change bool) (*secp256k1.PublicKey, error) {
+	switch ik {
+	case md.InternalKeyNUMS:
+		return address.NUMSInternalKey()
+	case md.InternalKeySlot:
+		internal, ok := byIndex[ikIndex]
+		if !ok {
+			return nil, errors.New("gui: taproot internal key has no @N entry")
+		}
+		return address.DeriveChild(internal, index, change)
+	case md.InternalKeyLianaUnspendable:
+		if liana == nil {
+			return nil, errUnderivableInternalKey
+		}
+		return address.DeriveChild(*liana, index, change)
+	}
+	return nil, errUnderivableInternalKey
+}
+
+// lianaInternalKey is the kind-1 internal key as a bip380.Key the shipped
+// deriver walks: SPEC §2 steps 3-4 (chain code = the recipe, public key = H,
+// depth 0, no origin) and step 6's `<0;1>/*` -- EXPLICITLY, not the wallet's
+// use-site. Liana derives this key at 0/i and 1/i "in every port, independent
+// of the wallet's use-site path" (§2), and §6 row 2 refuses any other use-site
+// at mint, so the two can never disagree on a card this device composed.
+//
+// OVER `keys`, NOT OVER THE CARD'S TLV (R0 I1). On the Wallet Policy route the
+// md1 is a key-less template and the keys are the seated mk1 cards; a recipe
+// that read the md1's Pubkeys TLV found nothing there, and every template +
+// cards kind-1 wallet showed "This device can't derive addresses" while its
+// NUMS twin derived. One key source for the leaves AND the internal key also
+// means the two can never come from different cards.
+func lianaInternalKey(collected []string, keys []md.ExpandedKey, network *chaincfg.Params) (bip380.Key, error) {
+	xpubs := make(map[uint8][65]byte, len(keys))
+	for _, k := range keys {
+		if k.XpubPresent {
+			xpubs[k.Index] = k.Xpub
+		}
+	}
+	lk, err := md.LianaUnspendableKeyFor(collected, xpubs)
+	if err != nil {
+		return bip380.Key{}, err
+	}
+	return bip380.Key{
+		Network: network,
+		Children: []bip380.Derivation{
+			{Type: bip380.RangeDerivation, Index: 0, End: 1},
+			{Type: bip380.WildcardDerivation},
+		},
+		KeyData:   append([]byte(nil), lk[32:65]...),
+		ChainCode: append([]byte(nil), lk[0:32]...),
+	}, nil
+}
+
 // tapLeafSpecs translates md's leaf descriptions into the address package's,
 // resolving each @N to its key.
 //
```

- [ ] **Step 4: Run.** `go vet ./gui/` (only the two baseline `ArtifactDir` lines) and `go test -count=1 -run 'Address|Taproot|Underivable|DeviceDerives' ./gui/` → `ok`; with `-v`, `keyed_tr_liana_kofn_recovery: 6 addresses via the complex route` five PASSing subtests of `TestDeviceDerivesLianasOwnAddressesForKind1`, and three of `TestTemplatePlusKeyCardsDerivesTheLianaWallet`.

- [ ] **Step 5: Commit** `gui: derive the Liana unspendable internal key; refuse a kind this firmware cannot derive (F-449 stage 4, SPEC §7a)`.

---

## Task 5 (fork, `gui/`): the print-site arms, the class rulings, F-633's copy, the inspect screen, and §7a.3's engrave half

**Why.** Three surfaces switch on the key path with NO default (fable M-3) and printed nothing for a fourth value: the consent screen (`gui/composer_consent.go:205-214`), the template-engrave summary whose own comment says the key-path line "COMES FIRST AND IS NEVER OMITTED" (`gui/template_engrave.go:159-164`), and the inspect screen, which named no key path at all (`md1Summary`, `gui/md1_inspect.go:84-99`). Each gets an arm. `md1Summary` names the kind for ALL taproot policies (a line that appears for one kind only does not "name the kind"), in ≤ 20 bytes because `md1PolicyFlow` hard-chunks longer lines mid-word.

**The class-2 + unlocked-path ruling needs no code (F2).** It is documented at the classifier and pinned by `TestComposerLianaClassRulings`, which also carries §7's constructed shape and the four F-644 shapes (each names a class; Liana v15.0 refuses each, F6).

**F-633's copy, applied to the claims this stage touches.** §8x ("OUTSIDE LIANA'S MODEL") now says "Liana (as of v15.0) takes…": the version the harness re-measured, 289/289 verdicts identical to v8.0 (F-633's own record). §8f's last sentence pointed at "(see F-449)"; it now says the device offers Liana's key where Liana can import the policy. The kind-1 key-path line is a NEW body, not §8f, because §8f's "Nunchuk cannot import a NUMS policy at all" is false for kind 1 some of the time (SPEC §7, fable M-5). The KNOWN_RELEASES gate (F-633's other half) stays with the coordinator-compat cycle.

**§7a.3's engrave half.** `walletPolicyConsentLines` refuses a KEYED taproot policy whose key-path kind md could not name (`md1KeyPathUnknown`), before consent. Scoped to keyed cards, so the D3/D4 template paths that engrave without an address on purpose are untouched (SPEC §7a.3 "Scope of the engrave refusal"). The composer's own consent already refuses such a card, because `policyShape` now reports it incomplete (Task 3) and `composerConsentLinesFor` refuses an incomplete shape (`gui/composer_consent.go:162-170`). Unreachable today; see "What no gate covers".

**Files:** Create `gui/keypath_print_sites_test.go`; Modify `gui/composer_consent.go`, `gui/template_engrave.go`, `gui/md1_inspect.go`, `gui/wallet_policy.go`, `gui/composer_copy.go`, `gui/composer_copy_test.go` (rows + count 86 → **87**).

**Interfaces:**
- Consumes: Tasks 2-4.
- Produces: `func composerCopyLianaKeyPath() string`; `func md1KeyPathLine(md.Template) (string, bool)`; `func md1KeyPathUnknown(md.Template) bool`; test helpers `composerTrPreset(t, name) md.PathList` and `lianaKofnChunks(t, kind) []string` (Task 6's tests use the first).

- [ ] **Step 1: Write the failing tests.**

Create `gui/keypath_print_sites_test.go`:

```go
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/md"
)

// F-449 stage 4: SPEC §7 on the device -- the key-path print sites, the
// class-2 + unlocked-path rulings (and F-644's device half), the inspect
// screen, and §7a.3's engrave half.

func composerTrPreset(t *testing.T, name string) md.PathList {
	t.Helper()
	for _, p := range composerPresets(md.ComposeTr) {
		if p.name == name {
			return p.list
		}
	}
	t.Fatalf("no tr preset %q", name)
	return md.PathList{}
}

func lianaKofnChunks(t *testing.T, kind md.UnspendableKind) []string {
	t.Helper()
	c, err := md.ComposeWithUnspendable(composerTrPreset(t, "kofn-recovery"), make([]*md.SlotOrigin, 4), kind)
	if err != nil {
		t.Fatal(err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	return chunks
}

// TestEveryKeyPathPrintSiteNamesTheLianaKind: the three surfaces that switch on
// the key path with no default (fable M-3) each print a kind-1 line, and the
// consent screen does not show the outside-Liana notice for a shape Liana
// imports.
//
// Mutations: deleting the KeyPathLianaUnspendable arm in composerConsentLinesFor,
// in policySummaryLines, or in md1KeyPathLine each fails its own row.
func TestEveryKeyPathPrintSiteNamesTheLianaKind(t *testing.T) {
	chunks := lianaKofnChunks(t, md.UnspendableLiana)

	consent, err := composerConsentLinesFor(chunks, nil, 0)
	if err != nil {
		t.Fatal(err)
	}
	drawn := normalizeDrawn(strings.Join(consent, "\n"))
	if !strings.Contains(drawn, normalizeDrawn(composerCopyLianaKeyPath())) {
		t.Errorf("consent: no kind-1 key-path line:\n%s", strings.Join(consent, "\n"))
	}
	if strings.Contains(drawn, normalizeDrawn("OUTSIDE LIANA'S MODEL")) {
		t.Errorf("consent: the outside-Liana notice fired on a shape Liana v15.0 imports")
	}

	shape, err := md.PolicyShapeChunks(chunks)
	if err != nil {
		t.Fatal(err)
	}
	if lines := policySummaryLines(shape); len(lines) == 0 || lines[0] != "Key-path: none (Liana key)" {
		t.Errorf("template summary: the key-path line is not first, or not kind 1: %q", lines)
	}

	tpl, _, err := md.ExpandWalletPolicyChunks(chunks)
	if err != nil {
		t.Fatal(err)
	}
	found := false
	for _, l := range md1Summary(tpl) {
		found = found || l == "Key path: Liana key"
	}
	// md1PolicyFlow hard-chunks every line longer than 20 bytes, mid-word, so
	// each key-path line this stage adds must fit one chunk.
	for _, kp := range []md.KeyPathKind{md.KeyPathNone, md.KeyPathNUMS, md.KeyPathSpendable, md.KeyPathLianaUnspendable} {
		if l, _ := md1KeyPathLine(md.Template{Root: md.ScriptTr, KeyPath: kp}); len(l) > 20 {
			t.Errorf("key-path line %q is %d bytes; md1PolicyFlow hard-chunks at 20", l, len(l))
		}
	}
	if !found {
		t.Errorf("md1Summary names no Liana key path: %q", md1Summary(tpl))
	}
	for kind, want := range map[md.UnspendableKind]string{md.UnspendableNums: "Key path: NUMS"} {
		tpl0, _, err := md.ExpandWalletPolicyChunks(lianaKofnChunks(t, kind))
		if err != nil {
			t.Fatal(err)
		}
		if !strings.Contains(strings.Join(md1Summary(tpl0), "|"), want) {
			t.Errorf("md1Summary at kind 0 does not say %q: %q", want, md1Summary(tpl0))
		}
	}
}

// TestComposerLianaClassRulings is SPEC §7's two rulings, stated separately
// because they are separate decisions -- and F-644's device half.
//
// Class 2 is skipped for the Liana kind (it is what the kind is for), and the
// Liana kind is NOT an unlocked path (an unspendable key spends nothing).
// §7's constructed shape -- one timelocked leaf -- is class 7 at kind 1;
// Liana v15.0 refuses it (design/evidence/f449-stage4/liana-probes-out.jsonl,
// `s7-single-timelocked-leaf`).
//
// The four F-644 shapes (md-cli composes them with no warning) each name a
// class here, and Liana v15.0 refuses each with its own recomputed key (same
// evidence file), so the device's choice screen never offers the Liana key
// for any of them.
//
// Mutations: `shape.KeyPath == md.KeyPathSpendable ||
// shape.KeyPath == md.KeyPathLianaUnspendable` in the unlocked count turns the
// §7 row "" (the composer would call the wallet Liana-compatible before
// steel); class 2 as `shape.KeyPath != md.KeyPathSpendable` turns kofn's kind-1
// row into "NUMS key path".
func TestComposerLianaClassRulings(t *testing.T) {
	one := func() md.SpendPath { return md.SpendPath{Keys: &md.KeySet{K: 1, N: 1}} }
	multi := func(k, n uint8) md.SpendPath { return md.SpendPath{Keys: &md.KeySet{K: k, N: n}} }
	locked := func(p md.SpendPath, kind md.LockKind, v uint32) md.SpendPath {
		p.Lock = &md.Lock{Kind: kind, Value: v}
		return p
	}
	for _, tc := range []struct {
		what string
		list md.PathList
		kind md.UnspendableKind
		want string
	}{
		{"kofn at kind 0: class 2 applies", composerTrPreset(t, "kofn-recovery"), md.UnspendableNums, "NUMS key path"},
		{"kofn at kind 1: class 2 skipped", composerTrPreset(t, "kofn-recovery"), md.UnspendableLiana, ""},
		{"SPEC §7 constructed: one timelocked leaf", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			locked(one(), md.LockOlderBlocks, 26280)}}, md.UnspendableLiana, "no unlocked path"},
		{"F-644 1: two unlocked primaries", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			multi(2, 3), multi(2, 2)}}, md.UnspendableLiana, "no locked path"},
		{"F-644 1b: two unlocked primaries plus a recovery", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			multi(2, 3), multi(2, 2), locked(one(), md.LockOlderBlocks, 26280)}}, md.UnspendableLiana, "a second unlocked path"},
		{"F-644 2: absolute-timelock recovery", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			multi(2, 2), locked(one(), md.LockAfterHeight, 800000)}}, md.UnspendableLiana, "an absolute lock"},
		{"F-644 3: duplicate recovery timelock", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			multi(2, 2), locked(one(), md.LockOlderBlocks, 100), locked(one(), md.LockOlderBlocks, 100)}}, md.UnspendableLiana, "two paths with one lock"},
		{"F-644 4: no recovery path", md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
			multi(2, 3)}}, md.UnspendableLiana, "no locked path"},
	} {
		n, err := md.ValidatePathList(tc.list)
		if err != nil {
			t.Fatalf("%s: %v", tc.what, err)
		}
		c, err := md.ComposeWithUnspendable(tc.list, make([]*md.SlotOrigin, n), tc.kind)
		if err != nil {
			t.Fatalf("%s: compose: %v", tc.what, err)
		}
		chunks, err := c.Chunks()
		if err != nil {
			t.Fatalf("%s: %v", tc.what, err)
		}
		shape, err := md.PolicyShapeChunks(chunks)
		if err != nil {
			t.Fatalf("%s: %v", tc.what, err)
		}
		if got := composerLianaOutsideModelClass(md.ScriptTr, shape); got != tc.want {
			t.Errorf("%s: class %q, want %q", tc.what, got, tc.want)
		}
	}
}

// TestAnUnnamedKeyPathIsRefusedBeforeConsent is SPEC §7a.3's engrave half: a
// keyed tr policy whose key-path kind md could not name is refused, never
// engraved without its proof. No decoder yields one today (md's
// TestAnUnknownInternalKeyKindIsNotDescribed pins that rootKeyPath reports it
// as KeyPathNone), so the Template is constructed.
//
// Mutation: md1KeyPathUnknown returning false fails.
func TestAnUnnamedKeyPathIsRefusedBeforeConsent(t *testing.T) {
	if !md1KeyPathUnknown(md.Template{Root: md.ScriptTr, KeyPath: md.KeyPathNone}) {
		t.Fatal("a tr policy with no named key path is not refused")
	}
	for _, kp := range []md.KeyPathKind{md.KeyPathNUMS, md.KeyPathSpendable, md.KeyPathLianaUnspendable} {
		if md1KeyPathUnknown(md.Template{Root: md.ScriptTr, KeyPath: kp}) {
			t.Errorf("a named key path %d is refused", kp)
		}
	}
	if md1KeyPathUnknown(md.Template{Root: md.ScriptWsh}) {
		t.Error("a wsh policy is refused for having no key path")
	}
}

// TestInspectNamesTheLianaKindAndItsPolicyId is SPEC §7's md1Summary ruling on
// the screen an operator uses a year later: a KEYED kind-1 card read through the
// gather path (gatheredDescriptorFlow -> complexAddressSource -> md1PolicyFlow)
// shows its Policy id AND names the Liana key path. At stage 3 it showed
// neither (stage 3 R0 m2), because the device could not derive kind 1 and so
// fell to "Complex policy - display only".
//
// Mutation: taprootInternalKey's Liana arm returning errUnderivableInternalKey
// sends the card back to "display only" and fails on the Policy id.
func TestInspectNamesTheLianaKindAndItsPolicyId(t *testing.T) {
	chunks := loadVectorChunks(t, "keyed_tr_liana_kofn_recovery")
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	frame, drawer, quit := runUITouch(ctx, func() { gatheredDescriptorFlow(ctx, &descriptorTheme, chunks) })
	defer quit()
	var seen strings.Builder
	for page := 0; page < 8; page++ {
		c, ok := frame()
		if !ok {
			break
		}
		seen.WriteString(c)
		if uiContains(seen.String(), "Key path: Liana key") {
			break
		}
		tapNavSlot(t, ctx, drawer(), Button3) // page, BY TOUCH (R0 m2)
	}
	all := seen.String()
	if uiContains(all, "display only") {
		t.Fatalf("a kind-1 keyed card fell to display-only:\n%q", all)
	}
	for _, want := range []string{"Policy id:", "Key path: Liana key"} {
		if !uiContains(all, want) {
			t.Errorf("the inspect screen does not show %q:\n%q", want, all)
		}
	}
}
```

Apply to `gui/composer_copy_test.go`:

```diff
diff --git a/gui/composer_copy_test.go b/gui/composer_copy_test.go
index bb4a7c1..21c5f7e 100644
--- a/gui/composer_copy_test.go
+++ b/gui/composer_copy_test.go
@@ -58,7 +58,13 @@ func composerCopyTable() []composerCopyRow {
 		// claim was measured FALSE, 0 of 7 shapes, by running libnunchuk
 		// 2.1.1. Filed as a §8 amendment so this table stays the diff target.
 		{"composerCopyNUMS", "8f", composerCopyNUMS(),
-			"KEY PATH: NONE (NUMS) Spends use the script paths only. Bitcoin Core imports this form. Nunchuk cannot import a NUMS policy at all: for Nunchuk, use wsh, or a tr policy whose first path is a single key. Liana and BIP-388 signers need an unspendable xpub instead (see F-449), which is a different wallet with different addresses."},
+			"KEY PATH: NONE (NUMS) Spends use the script paths only. Bitcoin Core imports this form. Nunchuk cannot import a NUMS policy at all: for Nunchuk, use wsh, or a tr policy whose first path is a single key. Liana needs its own unspendable key instead, which this device offers where Liana can import the policy; that is a different wallet with different addresses."},
+		// §8y is F-449 stage 4 (SPEC_liana_unspendable_internal_key §0b and
+		// §7): the kind-1 key-path line, the choice screen's lead and two
+		// rows, and the RESET signal. Filed as a §8 amendment so this table
+		// stays the diff target.
+		{"composerCopyLianaKeyPath", "8y", composerCopyLianaKeyPath(),
+			"KEY PATH: NONE (LIANA KEY) Spends use the script paths only. The key path is Liana's unspendable key, computed from this wallet's own keys. Liana (as of v15.0) and Bitcoin Core import this form. Nunchuk imports it only when the keys happen to be in sorted order. The same paths with the NUMS key are a different wallet with different addresses."},
 		{"composerCopyMixedLockBases", "8g", composerCopyMixedLockBases(),
 			"MIXED LOCK BASES Some paths lock by block height and others by time. Nunchuk will refuse this wallet; Bitcoin Core imports it. Taproot accepts both, because it checks each path on its own."},
 		// §8x is the fable review r0 lens-5 I-1/I-2 addition: one consent
@@ -68,7 +74,7 @@ func composerCopyTable() []composerCopyRow {
 		// tell them only after. The example below is the demo payload's own
 		// class -- a plain multisig has no recovery path at all.
 		{"composerCopyOutsideLianaModel", "8x", composerCopyOutsideLianaModel("no locked path"),
-			"OUTSIDE LIANA'S MODEL Liana takes one unlocked path, at least one path locked by older in blocks, and no hash. This policy: no locked path. Bitcoin Core imports it."},
+			"OUTSIDE LIANA'S MODEL Liana (as of v15.0) takes one unlocked path, at least one path locked by older in blocks, and no hash. This policy: no locked path. Bitcoin Core imports it."},
 		{"composerCopySameSeedThreshold", "8g", composerCopySameSeedThreshold([]uint8{1, 2}, 2, 3),
 			"SAME SEED, SAME PATH Slots @1 and @2 are the same seed. This path's 2-of-3 can be satisfied by one person. Liana will refuse it."},
 		{"composerCopySameSeedBelow", "8g", composerCopySameSeedBelow([]uint8{1, 2}, 3),
@@ -449,8 +455,10 @@ func TestComposerCopyTableCoversEveryBody(t *testing.T) {
 	// at §8g) before this row. No existing body could carry it -- §8f and §8g
 	// each name ONE class unconditionally, and this one names whichever of
 	// nine applies, in Liana's own order of refusal.
-	if declared != 86 {
-		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 86 -- "+
+	// 87 SINCE F-449 STAGE 4 TASK 5 added §8y's kind-1 key-path line: the
+	// §8f body is false for kind 1 about Nunchuk, so it could not be reused.
+	if declared != 87 {
+		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 87 -- "+
 			"if that is deliberate, update both", declared)
 	}
 }
```

- [ ] **Step 2: Run and see them fail.** `go test -count=1 -run 'KeyPath|ClassRulings|Unnamed|InspectNames|TestComposerCopy' ./gui/` → FAIL to compile (`undefined: composerCopyLianaKeyPath`).

- [ ] **Step 3: Implement.**

Apply to `gui/composer_consent.go`:

```diff
diff --git a/gui/composer_consent.go b/gui/composer_consent.go
index ed5a719..cb61bf2 100644
--- a/gui/composer_consent.go
+++ b/gui/composer_consent.go
@@ -211,6 +211,11 @@ func composerConsentLinesFor(chunks []string, listed []int, keyPathNo int) ([]st
 		lines = append(lines, line)
 	case md.KeyPathNUMS:
 		lines = append(lines, composerCopyNUMS())
+	case md.KeyPathLianaUnspendable:
+		// SPEC §7's kind-1 arm. Without it this switch printed NOTHING for a
+		// Liana-key policy -- the key-path fact silently absent from the one
+		// screen that consents to steel (fable M-3).
+		lines = append(lines, composerCopyLianaKeyPath())
 	}
 
 	id, kind, err := md.FormAwareIdChunks(chunks)
@@ -378,6 +383,16 @@ func composerMixesLockBases(branches []md.Branch) bool {
 // timelocked leaf -- Liana's OWN accepted shape -- would read as "no
 // unlocked path" (its leaf list holds one entry and that entry is locked).
 // KeyPathNUMS does not count: a NUMS key spends no path at all.
+//
+// THE LIANA KEY (F-449, SPEC §7) IS NEITHER CLASS 2 NOR AN UNLOCKED PATH, and
+// both halves are decided by the two equality tests below rather than by an
+// arm of their own: class 2 asks `== KeyPathNUMS`, and the unlocked count asks
+// `== KeyPathSpendable`, so KeyPathLianaUnspendable passes the first (it is
+// what Liana's own recipe produces, the reason the kind exists) and is not
+// counted by the second (an unspendable key spends nothing). Grouping it with
+// KeyPathSpendable would make `tr(<Liana key>, and_v(v:pk(@0),older(26280)))`
+// read as Liana-compatible when Liana refuses it -- class 7, measured at
+// v15.0 (design/evidence/f449-stage4/). TestComposerLianaClassRulings pins it.
 func composerLianaOutsideModelClass(root md.ScriptKind, shape md.PolicyShape) string {
 	// 1. Liana takes wsh or tr only (analysis.rs:586-587). ScriptSh covers
 	// BOTH bare sh and sh(wsh) (composerScriptLine's grouping): the composer
```

Apply to `gui/template_engrave.go`:

```diff
diff --git a/gui/template_engrave.go b/gui/template_engrave.go
index d53de3f..8baf290 100644
--- a/gui/template_engrave.go
+++ b/gui/template_engrave.go
@@ -161,6 +161,10 @@ func policySummaryLines(shape md.PolicyShape) []string {
 		out = append(out, "Key-path: A KEY CAN SPEND ALONE")
 	case md.KeyPathNUMS:
 		out = append(out, "Key-path: none (script paths only)")
+	case md.KeyPathLianaUnspendable:
+		// SPEC §7: "THE KEY-PATH LINE COMES FIRST AND IS NEVER OMITTED" was
+		// false for kind 1 until this arm -- the switch has no default.
+		out = append(out, "Key-path: none (Liana key)")
 	}
 	if n := len(shape.Branches); n > 0 {
 		word := "paths"
```

Apply to `gui/md1_inspect.go`:

```diff
diff --git a/gui/md1_inspect.go b/gui/md1_inspect.go
index fdd922c..fd2a340 100644
--- a/gui/md1_inspect.go
+++ b/gui/md1_inspect.go
@@ -88,6 +88,13 @@ func md1Summary(tpl md.Template) []string {
 	} else {
 		lines = append(lines, "Complex policy - cannot display safely.", fmt.Sprintf("Keys: %d", tpl.N))
 	}
+	// THE KEY PATH, NAMED (SPEC §7). A year later this is the screen an
+	// operator asks "which kind are these plates?", and before F-449 stage 4
+	// it named no internal key at all. Every string is at most 20 bytes: the
+	// caller hard-chunks longer lines mid-word.
+	if line, ok := md1KeyPathLine(tpl); ok {
+		lines = append(lines, line)
+	}
 	for _, k := range tpl.Keys {
 		fp := k.Fingerprint
 		if fp == "" {
@@ -188,3 +195,21 @@ func md1PolicyFlow(ctx *Context, th *Colors, tpl md.Template, header []string, a
 		ctx.Frame(op.Layer(frameOps...))
 	}
 }
+
+// md1KeyPathLine names a taproot policy's key path, or !ok for a non-taproot
+// root. A tr root whose kind md could not name (KeyPathNone) says so rather
+// than guessing.
+func md1KeyPathLine(tpl md.Template) (string, bool) {
+	if tpl.Root != md.ScriptTr {
+		return "", false
+	}
+	switch tpl.KeyPath {
+	case md.KeyPathSpendable:
+		return "Key path: a key", true
+	case md.KeyPathNUMS:
+		return "Key path: NUMS", true
+	case md.KeyPathLianaUnspendable:
+		return "Key path: Liana key", true
+	}
+	return "Key path: unknown", true
+}
```

Apply to `gui/wallet_policy.go`:

```diff
diff --git a/gui/wallet_policy.go b/gui/wallet_policy.go
index d1cd499..5966b22 100644
--- a/gui/wallet_policy.go
+++ b/gui/wallet_policy.go
@@ -257,6 +257,13 @@ func walletPolicyConsentLines(md1 []string, keyCards []mk.Card) ([]string, error
 				"but one of them holds two seats.", a, b, tpl.N)
 	}
 
+	// SPEC §7a.3's ENGRAVE HALF: a keyed taproot policy whose key-path kind
+	// this firmware cannot name is refused before consent, not engraved
+	// without its proof. Scoped to KEYED cards: a key-less template (D3/D4)
+	// engraves without an address on purpose, and this is not that.
+	if len(keys) > 0 && md1KeyPathUnknown(tpl) {
+		return nil, errors.New("This firmware cannot derive this policy's taproot key path, so it cannot prove the addresses.")
+	}
 	lines = append(lines, md1Summary(tpl)...)
 	lines = append(lines, walletPolicyAddressLines(md1, tpl, keys)...)
 	return lines, nil
@@ -453,3 +460,11 @@ func seatRefusalMessage(err error) error {
 	}
 	return errors.New("Couldn't match the key cards to this policy.")
 }
+
+// md1KeyPathUnknown reports a taproot root whose internal-key kind md could not
+// name (md.rootKeyPath returns KeyPathNone for one). No decoder yields such a
+// kind today -- versions {4, 8} admit three -- so this is the refusal for the
+// day a fourth kind exists (SPEC §7a.3), pinned by a constructed Template.
+func md1KeyPathUnknown(tpl md.Template) bool {
+	return tpl.Root == md.ScriptTr && tpl.KeyPath == md.KeyPathNone
+}
```

Apply to `gui/composer_copy.go`:

```diff
diff --git a/gui/composer_copy.go b/gui/composer_copy.go
index 8bdc5e8..d121583 100644
--- a/gui/composer_copy.go
+++ b/gui/composer_copy.go
@@ -200,9 +200,30 @@ func composerCopyNUMS() string {
 	return "KEY PATH: NONE (NUMS)\n" +
 		"Spends use the script paths only. Bitcoin Core imports this form. " +
 		"Nunchuk cannot import a NUMS policy at all: for Nunchuk, use wsh, or a " +
-		"tr policy whose first path is a single key. Liana and BIP-388 signers " +
-		"need an unspendable xpub instead (see F-449), which is a different " +
-		"wallet with different addresses."
+		"tr policy whose first path is a single key. Liana needs its own " +
+		"unspendable key instead, which this device offers where Liana can " +
+		"import the policy; that is a different wallet with different addresses."
+}
+
+// composerCopyLianaKeyPath is §8f's sibling for wire kind 1 (F-449 stage 4,
+// SPEC §7's print-site arm): the key-path line a Liana-key policy shows at
+// consent. It must NOT reuse §8f, whose Nunchuk sentence is FALSE for kind 1
+// some of the time (SPEC §7, fable M-5): libnunchuk re-renders the key path
+// with the PR-1746 recipe and compares, so it accepts a kind-1 descriptor
+// exactly when the leaf keys already sit in sorted, unique order -- 1 in 2 for
+// two keys, 1 in 24 for four -- and then imports the same wallet.
+//
+// "as of Liana v15.0" is F-633's remedy applied to a NEW claim: a present-tense
+// statement about third-party software names the release it was measured on.
+// Bitcoin Core: measured 2026-09-23, receive 0..2 equal to Liana's own
+// (design/evidence/f449-stage4/).
+func composerCopyLianaKeyPath() string {
+	return "KEY PATH: NONE (LIANA KEY)\n" +
+		"Spends use the script paths only. The key path is Liana's unspendable " +
+		"key, computed from this wallet's own keys. Liana (as of v15.0) and " +
+		"Bitcoin Core import this form. Nunchuk imports it only when the keys " +
+		"happen to be in sorted order. The same paths with the NUMS key are a " +
+		"different wallet with different addresses."
 }
 
 // composerCopyMixedLockBases is the fable review r0 lens-2 I-2 notice, in the
@@ -252,7 +273,7 @@ func composerCopyMixedLockBases() string {
 // only place that order is written down.
 func composerCopyOutsideLianaModel(class string) string {
 	return "OUTSIDE LIANA'S MODEL\n" +
-		"Liana takes one unlocked path, at least one path locked by older in " +
+		"Liana (as of v15.0) takes one unlocked path, at least one path locked by older in " +
 		"blocks, and no hash. This policy: " + class + ". Bitcoin Core imports it."
 }
 
```

- [ ] **Step 4: Run.** `go test -count=1 -run 'TestComposerCopy|KeyPath|Liana|Inspect|Unnamed|Fable|TestEmulatorWalks|Modal' ./gui/` → `ok`. `TestFableNUMSNoteDoesNotPromiseNunchuk` and the outside-Liana table (`composer_fable_r0_funds_test.go`) stay green: they build their expectations from the copy functions.

- [ ] **Step 5: Commit** `gui: name the Liana key on every key-path surface; F-633's version on the Liana claims (F-449 stage 4, SPEC §7)`.

---

## Task 6 (fork, `gui/`): §0b's key-path choice screen — predicate, placement, reset, default row, copy

**Placement (measured at `d2350cb`).** `composerShapeFlow` returns at `gui/composer_flow.go:86-95`, `composerSizeAssignments(st)` is `:96`, `composerTemplateChunksFor(st)` is `:98`, the first `composerStubFlow` `:129`. The step goes between `:96` and `:98`. Back from it `continue`s to the path list, exactly as Back from the stub screen does, so a Back out of the stub screen and forward again re-enters the choice (DEFAULT ROW's re-entry case).

**Predicate.** `composerUnspendableFires` evaluates both conjuncts on a composition it builds for itself (composing is pure; SPEC §0b "How the screen gets a shape"): conjunct 1 is `InternalKeyPath()` reporting no real key; conjunct 2 is `composerLianaOutsideModelClass` on the **kind-1** composition, which is how "class 2 skipped for the new kind" is expressed without a flag (F2).

**Reset.** The same predicate, every forward pass. A kind-1 choice the predicate no longer admits drops to NUMS and says why in a modal (`LIANA KEY DROPPED` + which fact moved). Conjunct 2's drop is a usefulness judgement, not a representability one, so it is SIGNALLED (§0b).

**Default row.** `composerPickScreenFrom(…, initial)` seeded from `st.unspendable`, whose zero value is NUMS (`md.UnspendableNums == 0`). Never a constant.

**Copy (§0b COPY).** The lead says the two are DIFFERENT WALLETS with different addresses and cannot be changed after engraving; each row names its coordinators. The lead is short on purpose: `composerPickScreen` draws it on every page, and the first-page gate counts two TAP TARGETS (F11).

**The one compose site.** `composerCompose` replaces the three `md.ComposeWith(st.list, …)` calls (`:268`, `:283`, `:308`) and runs §6's refusals on the mint path (F-654's second half). `TestComposerComposesOnlyThroughOneSite` parses the package and fails on any other `md.ComposeWith*` call.

**Self-check.** `composerSelfCheck` (§8q, run at consent) gains the biconditional "chose Liana ⟺ the decoded card is kind 1".

**The unmet request (R0 m1).** `composerCompose` also refuses a Liana choice that composed a REAL key path (`(md.Composed).UnspendableRequestUnmet`, SPEC §6 row 3, the signal md-cli warns on). The reset keeps the flow out of that state, so it cannot fire today; it runs before the stub screen shows an id, where the self-check only runs at consent.

**The drop cause for a build failure (R0 n1)** says the device could not build the policy, not "Liana would not import this policy", because a compose error is not Liana's verdict.

**Files:** Create `gui/composer_unspendable.go`, `gui/composer_unspendable_test.go`, `gui/composer_unspendable_sites_test.go`; Modify `gui/composer_state.go` (`:29`), `gui/composer_flow.go`, `gui/composer_selfcheck.go` (`:73`), `gui/composer_copy.go` (five bodies), `gui/composer_copy_test.go` (rows + count 87 → **92**).

**Interfaces:**
- Consumes: Task 2's `md.ComposeWithUnspendable`, `ValidateUnspendableShape`, `ErrUnspendableSortedMultiA`; Task 3's `md.KeyPathLianaUnspendable`; Task 5's `composerTrPreset`.
- Produces: `composerState.unspendable md.UnspendableKind`; `composerUnspendableFires(*composerState) (bool, composerUnspendableDrop)`; `composerUnspendableStep(ctx, th, st) bool`; `composerCompose(st, declared) (md.Composed, error)`; `composerCopyUnspendableLead`, `composerCopyUnspendableRowNUMS`, `composerCopyUnspendableRowLiana`, `composerCopyLianaKeyDropped(cause string)`, `composerCopyLianaUnmet` (its text is the refusal composerCompose returns).

- [ ] **Step 1: Write the failing tests.** Every screen test drives the screen BY TOUCH (`sessionHarness.tapRow` / `tapNav`); DEFAULT ROW is proven by consequence, because a highlight is a colour inversion `ExtractText` cannot see.

Create `gui/composer_unspendable_test.go`:

```go
package gui

import (
	"encoding/hex"
	"strings"
	"testing"

	"seedhammer.com/md"
)

// F-449 stage 4, SPEC_liana_unspendable_internal_key §0b and §7 on the device.

func composerStateFor(list md.PathList, kind md.UnspendableKind) *composerState {
	st := &composerState{reg: &seedRegistry{}, list: list, unspendable: kind}
	composerSizeAssignments(st)
	return st
}

// TestComposerUnspendablePredicateOnEveryTrPreset is §9 row 4's gate: §0b's
// predicate over ALL SIX tr presets, firing on exactly kofn-recovery and
// tiered-recovery. It is asserted as a SET, so a seventh preset added later
// fails until someone decides where it belongs.
//
// Mutations, each measured:
//   - drop conjunct 1 (the InternalKeyPath check): fires on
//     simple-timelocked-inheritance, whose real key path counts as the one
//     unlocked path -- r1's inverted rule, SPEC §0b's table row 1;
//   - drop conjunct 2 (the class check): fires on plain-multisig,
//     hashlock-gated and decaying-multisig;
//   - evaluate the classifier on the NUMS composition instead of kind 1
//     (class 2 NOT skipped): fires on nothing -- r1's rule, rows 2 and 3.
func TestComposerUnspendablePredicateOnEveryTrPreset(t *testing.T) {
	want := map[string]bool{
		"plain-multisig":                false,
		"simple-timelocked-inheritance": false,
		"kofn-recovery":                 true,
		"tiered-recovery":               true,
		"hashlock-gated":                false,
		"decaying-multisig":             false,
	}
	presets := composerPresets(md.ComposeTr)
	if len(presets) != len(want) {
		t.Fatalf("%d tr presets, the gate names %d", len(presets), len(want))
	}
	for _, p := range presets {
		w, named := want[p.name]
		if !named {
			t.Fatalf("tr preset %q is not in the gate's table", p.name)
		}
		fire, drop := composerUnspendableFires(composerStateFor(p.list, md.UnspendableNums))
		if fire != w {
			t.Errorf("%s: fires = %v, want %v (drop %+v)", p.name, fire, w, drop)
		}
	}
	for _, w := range []md.ComposeWrapper{md.ComposeWsh, md.ComposeShWsh, md.ComposeSh} {
		for _, p := range composerPresets(w) {
			if fire, _ := composerUnspendableFires(composerStateFor(p.list, md.UnspendableNums)); fire {
				t.Errorf("%s under wrapper %v: fires, and only a tr policy has a key path", p.name, w)
			}
		}
	}
}

// runUnspendableStep drives composerUnspendableStep alone, by touch.
func runUnspendableStep(t *testing.T, st *composerState, ret *bool) *sessionHarness {
	t.Helper()
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	h := &sessionHarness{t: t, ctx: ctx, done: new(bool)}
	frame, drawer, quit := runUITouch(ctx, func() {
		*ret = composerUnspendableStep(ctx, &descriptorTheme, st)
		*h.done = true
	})
	h.frame, h.drawer = frame, drawer
	t.Cleanup(quit)
	return h
}

// TestComposerUnspendableScreenFirstPage is §0b COPY and the first-page gate:
// both rows are TAPPABLE on the first page (a paginated screen hides its
// overflow, so the count is taken from the hit areas, not the text), and the
// copy names each row's coordinators and says the two are different wallets.
//
// Mutation: lengthening the lead by ~250 characters pushes the Liana row to
// page 2: 1 tappable row, and this fails.
func TestComposerUnspendableScreenFirstPage(t *testing.T) {
	var ret bool
	h := runUnspendableStep(t, composerStateFor(composerTrPreset(t, "kofn-recovery"), md.UnspendableNums), &ret)
	body := h.mustReach("Which key path?")
	if pts := plateHitPoints(h.ctx, h.drawer()); len(pts) != 2 {
		t.Fatalf("the key-path screen drew %d tappable rows on its first page, want 2:\n%q", len(pts), body)
	}
	for _, want := range []string{"DIFFERENT WALLETS", "different addresses",
		"NUMS point", "Liana key", "Bitcoin Core", "Nunchuk", "Liana (v15.0)"} {
		if !uiContains(body, want) {
			t.Errorf("the first page does not say %q:\n%q", want, body)
		}
	}
}

// TestComposerUnspendableDefaultRow is §0b DEFAULT ROW, BOTH assertions -- one
// alone cannot fail. Proven by CONSEQUENCE (a highlight is a colour inversion
// ExtractText cannot see): Button3 with no tap takes the seeded row.
//
// THE FIRST ENTRY STARTS FROM A ZERO-VALUE composerState, as composerFlow
// builds it (a struct literal that never names the field), so the zero-value
// trap is inside the test: if UnspendableKind's zero value were Liana, a
// press-through would build the Liana wallet.
//
// Mutations: seeding `initial := 1` (a constant) fails "first entry";
// `initial := 0` (a constant) fails "re-entry"; swapping md's UnspendableNums
// and UnspendableLiana constants fails "first entry".
func TestComposerUnspendableDefaultRow(t *testing.T) {
	for _, tc := range []struct {
		what string
		st   func() *composerState
		want md.UnspendableKind
	}{
		{"first entry opens on NUMS", func() *composerState {
			st := &composerState{reg: &seedRegistry{}, list: composerTrPreset(t, "kofn-recovery")}
			composerSizeAssignments(st)
			return st
		}, md.UnspendableNums},
		{"re-entry after choosing Liana opens on Liana", func() *composerState {
			return composerStateFor(composerTrPreset(t, "kofn-recovery"), md.UnspendableLiana)
		}, md.UnspendableLiana},
	} {
		t.Run(tc.what, func(t *testing.T) {
			st := tc.st()
			var ret bool
			h := runUnspendableStep(t, st, &ret)
			h.mustReach("Which key path?")
			h.tapNav(Button3) // press straight through, no row tapped
			h.pump(8, "")
			if !*h.done || !ret {
				t.Fatalf("the step did not return forward (done %v, ret %v)", *h.done, ret)
			}
			if st.unspendable != tc.want {
				t.Fatalf("pressing through left kind %v, want %v", st.unspendable, tc.want)
			}
		})
	}
}

// TestComposerUnspendableRowTapChoosesLiana: the Liana row is reachable by
// TOUCH, the only input the SH2 has (no directional buttons), and Back leaves
// the kind as it was.
func TestComposerUnspendableRowTapChoosesLiana(t *testing.T) {
	st := composerStateFor(composerTrPreset(t, "tiered-recovery"), md.UnspendableNums)
	var ret bool
	h := runUnspendableStep(t, st, &ret)
	h.mustReach("Which key path?")
	h.tapRow(1, 2)
	h.pump(8, "")
	if !ret || st.unspendable != md.UnspendableLiana {
		t.Fatalf("tapping the Liana row and taking it left kind %v (ret %v)", st.unspendable, ret)
	}

	st = composerStateFor(composerTrPreset(t, "tiered-recovery"), md.UnspendableLiana)
	h = runUnspendableStep(t, st, &ret)
	h.mustReach("Which key path?")
	h.tapNav(Button1)
	h.pump(8, "")
	if ret || st.unspendable != md.UnspendableLiana {
		t.Fatalf("Back returned %v and left kind %v, want false and Liana", ret, st.unspendable)
	}
}

// TestComposerUnspendableResetIsThePredicate is §0b RESET, both halves:
//   - a Back-edit that makes a conjunct false drops the kind, and SAYS so
//     naming the fact that moved (here conjunct 1: path 1 became a bare
//     single key, so it is the real key path);
//   - the converse, which catches an over-eager reset: a Back-edit that keeps
//     both conjuncts true (kofn -> tiered) keeps the kind.
//
// Mutations: removing the reset (`st.unspendable = md.UnspendableNums`) fails
// the first half at the kind; resetting on every entry fails the second;
// dropping the showError fails the first at the signal.
func TestComposerUnspendableResetIsThePredicate(t *testing.T) {
	realKey := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 26280}},
	}}
	st := composerStateFor(realKey, md.UnspendableLiana)
	var ret bool
	h := runUnspendableStep(t, st, &ret)
	body := h.mustReach("LIANA KEY DROPPED")
	if !uiContains(body, "Path 1 is one key with no lock") {
		t.Errorf("the drop does not name its cause:\n%q", body)
	}
	h.tapNav(Button3)
	h.pump(8, "")
	if !ret || st.unspendable != md.UnspendableNums {
		t.Fatalf("after the drop: ret %v kind %v, want true and NUMS", ret, st.unspendable)
	}

	st = composerStateFor(composerTrPreset(t, "tiered-recovery"), md.UnspendableLiana)
	ret = false
	h = runUnspendableStep(t, st, &ret)
	h.mustReach("Which key path?")
	h.tapNav(Button3)
	h.pump(8, "")
	// THE STEP MUST HAVE RETURNED FORWARD (R0 m3): without this, a press that
	// never registered leaves the kind at Liana too, and the half passes on a
	// screen still waiting for input.
	if !*h.done || !ret {
		t.Fatalf("the re-entered step did not return forward (done %v, ret %v)", *h.done, ret)
	}
	if st.unspendable != md.UnspendableLiana {
		t.Fatalf("an edit that keeps both conjuncts true reset the kind to %v", st.unspendable)
	}
}

// TestComposerLianaKeyDroppedFits: the RESET signal is a showError modal, so it
// gets F-185's class check at the longest cause it can carry.
func TestComposerLianaKeyDroppedFits(t *testing.T) {
	for _, d := range []composerUnspendableDrop{
		{notTr: true}, {keyPath: 8}, {class: "two paths with one lock"}, {unbuilt: true},
	} {
		assertModalBodyFits(t, "the §0b reset signal", errorScreenBody,
			composerCopyLianaKeyDropped(composerUnspendableDropCause(d)))
	}
}

// TestComposerKeyPathChoiceIsPlacedBeforeTheChunks is §0b PLACEMENT, on the
// real flow: after the path list's Done the NEXT screen is the key-path choice,
// not the stub screen, and the stub screen then shows the Template-ID OF THE
// CHOSEN KIND -- which is only possible if the chunks were built after the
// choice. Back from the stub and forward again re-enters the choice seeded on
// Liana, and pressing through keeps the Liana id (no changed-id banner can
// fire, because nothing changed).
//
// Mutation: moving composerUnspendableStep below composerTemplateChunksFor
// makes the first stub screen show the NUMS id.
func TestComposerKeyPathChoiceIsPlacedBeforeTheChunks(t *testing.T) {
	list := composerTrPreset(t, "kofn-recovery")
	idOf := func(kind md.UnspendableKind) string {
		c, err := md.ComposeWithUnspendable(list, make([]*md.SlotOrigin, 4), kind)
		if err != nil {
			t.Fatal(err)
		}
		id, err := c.TemplateID()
		if err != nil {
			t.Fatal(err)
		}
		return hex.EncodeToString(id[:])
	}
	liana, nums := idOf(md.UnspendableLiana), idOf(md.UnspendableNums)
	if liana == nums {
		t.Fatal("the two kinds share a Template-ID; SPEC §3e is broken below this test")
	}

	p := newEngravedAwarePlatform()
	p.engraver = newEngraver()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	frame, drawer, quit := runUITouch(ctx, func() { walletPolicyFlow(ctx, &descriptorTheme) })
	defer quit()
	h := &sessionHarness{t: t, ctx: ctx, frame: frame, drawer: drawer, done: new(bool)}

	// BY TOUCH THROUGHOUT (R0 m2): the SH2 has no directional buttons, so
	// every row is tapped and every nav press is a tap on the nav slot.
	h.mustReach("Build a new policy")
	h.choose(1) // Scan cards, [Build a new policy]
	h.mustReach("Which script?")
	h.choose(0) // [Taproot (tr)]
	h.mustReach("Start from?")
	h.choose(3) // Build my own paths, plain-multisig, simple-timelocked-inheritance, [kofn-recovery]
	h.mustReach("Done")
	composerPickDone(t, h)

	body := h.mustReach("Which key path?")
	if uiContains(body, "mk1 stub") {
		t.Fatalf("the stub screen drew before the key-path choice:\n%q", body)
	}
	h.tapRow(1, 2) // Liana key
	body = h.mustReach("mk1 stub (template)")
	if !strings.Contains(strings.ToLower(body), liana) {
		t.Fatalf("the stub screen does not show the Liana Template-ID %s:\n%q", liana, body)
	}

	h.tapNav(Button1) // Back from the stub -> the path list
	h.mustReach("Done")
	composerPickDone(t, h)
	h.mustReach("Which key path?")
	h.tapNav(Button3) // re-entry, straight through
	body = h.mustReach("mk1 stub (template)")
	if !strings.Contains(strings.ToLower(body), liana) {
		t.Fatalf("re-entry pressed through did not keep the Liana id %s:\n%q", liana, body)
	}
}

// composerPickDone takes the path list's `Done` row, which is the LAST row.
func composerPickDone(t *testing.T, h *sessionHarness) {
	t.Helper()
	pts := plateHitPoints(h.ctx, h.drawer())
	if len(pts) == 0 {
		t.Fatalf("the path list drew no rows:\n%q", h.content)
	}
	tap(&h.ctx.Router, h.drawer(), pts[len(pts)-1])
	h.next("after selecting Done")
	h.tapNav(Button3)
}
```

Create `gui/composer_unspendable_sites_test.go`:

```go
package gui

import (
	"errors"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"seedhammer.com/md"
)

// F-449 stage 4: SPEC §7's print sites, the self-check, the one compose site,
// §6's refusal on the composer's mint path, and §7a.3's engrave half.

// TestComposerSelfCheckSeesTheKeyPathChoice: the self-check compares the
// operator's choice with the decoded card in both directions.
//
// Mutation: deleting the biconditional in composerSelfCheck fails both rows.
func TestComposerSelfCheckSeesTheKeyPathChoice(t *testing.T) {
	for _, tc := range []struct {
		chose, built md.UnspendableKind
	}{{md.UnspendableLiana, md.UnspendableNums}, {md.UnspendableNums, md.UnspendableLiana}} {
		st := composerStateFor(composerTrPreset(t, "kofn-recovery"), tc.chose)
		if err := composerSelfCheck(st, lianaKofnChunks(t, tc.built)); err == nil ||
			!strings.Contains(err.Error(), "key path") {
			t.Errorf("chose %v, built %v: self-check said %v", tc.chose, tc.built, err)
		}
		st.unspendable = tc.built
		if err := composerSelfCheck(st, lianaKofnChunks(t, tc.built)); err != nil {
			t.Errorf("an honest %v composition failed the self-check: %v", tc.built, err)
		}
	}
}

// TestComposerComposesOnlyThroughOneSite: every production lowering of the
// operator's path list goes through composerCompose, so the key-path choice
// reaches every artifact. A second call site that went straight to
// md.ComposeWith would build the NUMS wallet under a Liana choice, and the
// cards minted from it would carry the other wallet's stub.
//
// EXEMPT BY ENCLOSING FUNCTION, NOT BY FILE (R0 m4): a file exemption let a
// second compose inside composer_unspendable.go through unseen. Exactly three
// functions may compose: composerCompose (the site), composerUnspendableFires
// (composes kind 1 for itself, to evaluate §0b's predicate; it builds no
// artifact), and composerShapeSignature (md.Compose; reads only the slot map,
// which the kind does not move, SPEC §5).
//
// Mutations: composerArtifactsFor calling md.ComposeWith(st.list, declared)
// again fails; so does a second md.ComposeWithUnspendable added to
// composerUnspendableStep.
func TestComposerComposesOnlyThroughOneSite(t *testing.T) {
	allowed := map[string]string{
		"ComposeWith":            "composerCompose",
		"ComposeWithUnspendable": "composerCompose|composerUnspendableFires",
		"Compose":                "composerShapeSignature",
	}
	files, err := filepath.Glob("*.go")
	if err != nil {
		t.Fatal(err)
	}
	seen := map[string]int{}
	for _, f := range files {
		if strings.HasSuffix(f, "_test.go") {
			continue
		}
		src, err := os.ReadFile(f)
		if err != nil {
			t.Fatal(err)
		}
		fset := token.NewFileSet()
		af, err := parser.ParseFile(fset, f, src, 0)
		if err != nil {
			t.Fatal(err)
		}
		for _, decl := range af.Decls {
			encl := "<package scope>"
			if fd, ok := decl.(*ast.FuncDecl); ok {
				encl = fd.Name.Name
			}
			ast.Inspect(decl, func(n ast.Node) bool {
				sel, ok := n.(*ast.SelectorExpr)
				if !ok {
					return true
				}
				if x, ok := sel.X.(*ast.Ident); !ok || x.Name != "md" {
					return true
				}
				want, watched := allowed[sel.Sel.Name]
				if !watched {
					return true
				}
				seen[encl]++
				ok = false
				for _, a := range strings.Split(want, "|") {
					ok = ok || a == encl
				}
				if !ok {
					t.Errorf("%s: md.%s in %s; only %s may compose", fset.Position(sel.Pos()), sel.Sel.Name, encl, want)
				}
				return true
			})
		}
	}
	// The allowed sites must still exist, or the rule is guarding a name.
	for _, fn := range []string{"composerCompose", "composerUnspendableFires", "composerShapeSignature"} {
		if seen[fn] != 1 {
			t.Errorf("%s composes %d times, want exactly 1", fn, seen[fn])
		}
	}
}

// TestComposerComposeRefusesAnUnmetLianaRequest is SPEC §6 row 3 on the
// composer's mint path (R0 m1): a Liana choice over a shape whose first bare
// single key became the internal key is refused before any chunk exists. The
// reset keeps the flow out of this state, so it is constructed.
//
// Mutation: deleting the UnspendableRequestUnmet check in composerCompose fails.
func TestComposerComposeRefusesAnUnmetLianaRequest(t *testing.T) {
	realKey := md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 1, N: 1}},
		{Keys: &md.KeySet{K: 1, N: 1}, Lock: &md.Lock{Kind: md.LockOlderBlocks, Value: 26280}},
	}}
	st := composerStateFor(realKey, md.UnspendableLiana)
	if _, err := composerTemplateChunksFor(st); err == nil || err.Error() != composerCopyLianaUnmet() {
		t.Fatalf("an unmet Liana request composed: err %v", err)
	}
	st.unspendable = md.UnspendableNums
	if _, err := composerTemplateChunksFor(st); err != nil {
		t.Fatalf("the same shape at NUMS refused: %v", err)
	}
}

// TestComposerComposeRefusesSpecSix: SPEC §6 on the composer's mint path
// (F-654). The predicate keeps a Liana choice off plain-multisig, so the state
// is set directly: the refusal is the belt for the day the predicate changes.
//
// Mutation: deleting the ValidateUnspendableShape call in composerCompose fails.
func TestComposerComposeRefusesSpecSix(t *testing.T) {
	st := composerStateFor(composerTrPreset(t, "plain-multisig"), md.UnspendableLiana)
	if _, err := composerTemplateChunksFor(st); !errors.Is(err, md.ErrUnspendableSortedMultiA) {
		t.Fatalf("a Liana key over sortedmulti_a composed: err %v", err)
	}
	st.unspendable = md.UnspendableNums
	if _, err := composerTemplateChunksFor(st); err != nil {
		t.Fatalf("the NUMS plain-multisig refused: %v", err)
	}
}
```

Apply to `gui/composer_copy_test.go`:

```diff
diff --git a/gui/composer_copy_test.go b/gui/composer_copy_test.go
index 21c5f7e..fecada5 100644
--- a/gui/composer_copy_test.go
+++ b/gui/composer_copy_test.go
@@ -65,6 +65,16 @@ func composerCopyTable() []composerCopyRow {
 		// stays the diff target.
 		{"composerCopyLianaKeyPath", "8y", composerCopyLianaKeyPath(),
 			"KEY PATH: NONE (LIANA KEY) Spends use the script paths only. The key path is Liana's unspendable key, computed from this wallet's own keys. Liana (as of v15.0) and Bitcoin Core import this form. Nunchuk imports it only when the keys happen to be in sorted order. The same paths with the NUMS key are a different wallet with different addresses."},
+		{"composerCopyUnspendableLead", "8y", composerCopyUnspendableLead(),
+			"Which key path? The two are DIFFERENT WALLETS, with different addresses. It cannot be changed after engraving."},
+		{"composerCopyUnspendableRowNUMS", "8y", composerCopyUnspendableRowNUMS(),
+			"NUMS point: Bitcoin Core imports it. Liana and Nunchuk do not."},
+		{"composerCopyUnspendableRowLiana", "8y", composerCopyUnspendableRowLiana(),
+			"Liana key: Liana (v15.0) and Bitcoin Core import it. Nunchuk only by chance."},
+		{"composerCopyLianaUnmet", "8y", composerCopyLianaUnmet(),
+			"The Liana key was chosen, but this policy has a real key path, so there is no unspendable key to choose. Go back to the key path screen."},
+		{"composerCopyLianaKeyDropped", "8y", composerCopyLianaKeyDropped("Path 1 is one key with no lock, so it became the key path, and there is no unspendable key to choose."),
+			"LIANA KEY DROPPED Path 1 is one key with no lock, so it became the key path, and there is no unspendable key to choose. This policy is back on the NUMS key path: its Template-ID and addresses are not the ones the Liana key gave."},
 		{"composerCopyMixedLockBases", "8g", composerCopyMixedLockBases(),
 			"MIXED LOCK BASES Some paths lock by block height and others by time. Nunchuk will refuse this wallet; Bitcoin Core imports it. Taproot accepts both, because it checks each path on its own."},
 		// §8x is the fable review r0 lens-5 I-1/I-2 addition: one consent
@@ -457,8 +467,11 @@ func TestComposerCopyTableCoversEveryBody(t *testing.T) {
 	// nine applies, in Liana's own order of refusal.
 	// 87 SINCE F-449 STAGE 4 TASK 5 added §8y's kind-1 key-path line: the
 	// §8f body is false for kind 1 about Nunchuk, so it could not be reused.
-	if declared != 87 {
-		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 87 -- "+
+	// 92 SINCE TASK 6 added the key-path choice screen's lead and two rows,
+	// the RESET signal that names which fact dropped a Liana choice, and the
+	// unmet-request refusal composerCompose raises (R0 m1).
+	if declared != 92 {
+		t.Errorf("composer_copy.go declares %d bodies, the plan and the table know 92 -- "+
 			"if that is deliberate, update both", declared)
 	}
 }
```

- [ ] **Step 2: Run and see them fail.** `go test -count=1 -run 'Unspendable|KeyPathChoice|SelfCheckSees|ComposesOnly|ComposeRefuses|LianaKeyDropped' ./gui/` → FAIL to compile (`undefined: composerUnspendableFires`).

- [ ] **Step 3: Implement.**

Create `gui/composer_unspendable.go`:

```go
package gui

import (
	"errors"
	"fmt"

	"seedhammer.com/md"
)

// SPEC_liana_unspendable_internal_key §0b: the KEY PATH choice screen.
//
// WHERE IT SITS, and why there. Between composerShapeFlow and
// composerTemplateChunksFor, in composerFlow's forward leg: the kind changes
// the tree, hence the template chunks, hence the Template-ID and the mk1 stub
// the stub screen tells the operator to COPY ONTO STEEL and mint cosigner
// cards from. Placed any later, those would already have been computed for the
// other wallet.
//
// HOW IT GETS A SHAPE BEFORE THE CHUNKS EXIST. The predicate needs a decoded
// PolicyShape, and the only producer of one is md.PolicyShapeChunks, which
// needs chunks. That is not circular: composing is PURE
// (md.ComposeWithUnspendable(...).Chunks()), so this file composes once for
// itself to evaluate the predicate, and the flow composes again afterwards for
// the stub screen. Nothing is shared between the two calls.
//
// THE RESET IS THE PREDICATE, NOT AN EDIT DETECTOR. composerShapeSignature is
// blind to exactly the edits that flip it (a lock, a hash, a key count that
// makes a path bare), so this does not try to notice edits. It re-evaluates
// the predicate on every forward pass, and a kind the predicate no longer
// admits is dropped -- LOUDLY, because conjunct 2 is a usefulness judgement,
// not a representability one, and a silent change of wallet is the defect
// this screen exists to prevent.

// composerUnspendableDrop names why §0b's predicate does not fire, so a
// dropped kind-1 choice can say which fact moved.
type composerUnspendableDrop struct {
	notTr   bool   // the wrapper has no taproot key path at all
	keyPath int    // > 0: the operator's path number that became the real internal key
	class   string // != "": Liana's first refusal class for the kind-1 composition
	unbuilt bool   // the kind-1 composition itself failed; not a Liana verdict (R0 n1)
}

// composerUnspendableFires is §0b's FIRING PREDICATE, both conjuncts:
//
//	fire <=> the tr internal key is NUMS today (no path became a real key)
//	     AND composerLianaOutsideModelClass returns "" with class 2 skipped
//
// CONJUNCT 2 IS EVALUATED ON THE KIND-1 COMPOSITION, which is what "class 2
// skipped for the new kind" means in code: class 2 fires on KeyPathNUMS, and a
// kind-1 shape reports KeyPathLianaUnspendable, so the classifier skips it by
// construction -- and, by the same construction, does NOT count it as an
// unlocked path (only KeyPathSpendable is counted). SPEC §7 states those as two
// separate rulings; TestComposerLianaClassRulings pins each.
//
// Any composition error is "does not fire": the flow's own compose, a few
// lines later, reports the refusal properly.
func composerUnspendableFires(st *composerState) (bool, composerUnspendableDrop) {
	if st.list.Wrapper != md.ComposeTr {
		return false, composerUnspendableDrop{notTr: true}
	}
	c, err := md.ComposeWithUnspendable(st.list, composerDeclaredOrigins(st), md.UnspendableLiana)
	if err != nil {
		return false, composerUnspendableDrop{unbuilt: true}
	}
	if p, real := c.InternalKeyPath(); real {
		return false, composerUnspendableDrop{keyPath: p + 1}
	}
	chunks, err := c.Chunks()
	if err != nil {
		return false, composerUnspendableDrop{unbuilt: true}
	}
	shape, err := md.PolicyShapeChunks(chunks)
	if err != nil || !shape.Complete {
		return false, composerUnspendableDrop{unbuilt: true}
	}
	if class := composerLianaOutsideModelClass(md.ScriptTr, shape); class != "" {
		return false, composerUnspendableDrop{class: class}
	}
	return true, composerUnspendableDrop{}
}

// composerUnspendableRows is the choice screen's rows, index-aligned with
// composerUnspendableKinds. Row 0 is NUMS: the zero-value trap (§0b DEFAULT
// ROW) is that a default of "Liana" would silently change the wallet for an
// operator who pressed through, so the first-entry default is the wallet
// every earlier firmware built.
func composerUnspendableRows() []string {
	return []string{composerCopyUnspendableRowNUMS(), composerCopyUnspendableRowLiana()}
}

var composerUnspendableKinds = []md.UnspendableKind{md.UnspendableNums, md.UnspendableLiana}

// composerUnspendableStep is §0b's screen and its reset. true = go on to the
// stub screen; false = Back, which returns to the path list (composerFlow's
// `continue`), the same place Back from the stub screen goes.
//
// THE DEFAULT ROW IS SEEDED ONCE PER ENTRY, FROM st.unspendable -- never from
// a constant. composerPickScreenFrom reads `initial` once, so a re-entry after
// choosing Liana opens on the Liana row, and pressing through keeps it (§0b:
// "a picker that opens on row zero proposes a setting").
func composerUnspendableStep(ctx *Context, th *Colors, st *composerState) bool {
	fire, drop := composerUnspendableFires(st)
	if !fire {
		if st.unspendable != md.UnspendableNums {
			st.unspendable = md.UnspendableNums
			showError(ctx, th, "Key path", composerCopyLianaKeyDropped(composerUnspendableDropCause(drop)))
		}
		return true
	}
	initial := 0
	for i, k := range composerUnspendableKinds {
		if k == st.unspendable {
			initial = i
		}
	}
	sel, ok := composerPickScreenFrom(ctx, th, "Key path", composerCopyUnspendableLead(), composerUnspendableRows(), initial)
	if !ok {
		return false
	}
	st.unspendable = composerUnspendableKinds[sel]
	return true
}

// composerUnspendableDropCause is the one sentence naming which fact moved.
func composerUnspendableDropCause(d composerUnspendableDrop) string {
	switch {
	case d.notTr:
		return "Only a Taproot policy has a key path to choose."
	case d.keyPath > 0:
		return fmt.Sprintf("Path %d is one key with no lock, so it became the key path, and there is no unspendable key to choose.", d.keyPath)
	case d.unbuilt:
		// A compose or describe failure is not something Liana said. Today it
		// is unreachable (composerTemplateChunksFor refuses first, a few lines
		// later, with the real reason), and it must not read as a verdict.
		return "This device could not build the policy with the Liana key."
	default:
		return "Liana would not import this policy (" + d.class + ")."
	}
}

// composerCompose is the ONE place the composer lowers its path list, so the
// operator's key-path choice reaches every artifact: the template the stub
// screen shows, the keyed policy, and the cards minted from them. A second
// call site that went straight to md.ComposeWith would silently build the NUMS
// wallet (TestComposerComposesOnlyThroughOneSite).
func composerCompose(st *composerState, declared []*md.SlotOrigin) (md.Composed, error) {
	c, err := md.ComposeWithUnspendable(st.list, declared, st.unspendable)
	if err != nil {
		return md.Composed{}, err
	}
	// A LIANA CHOICE THAT BUILT SOMETHING ELSE IS REFUSED, not carried on (SPEC
	// §6 row 3; md-cli warns on the same signal, Composed::unspendable_request_
	// unmet). composerUnspendableStep resets the kind whenever conjunct 1 fails,
	// so this cannot fire through the flow today -- and it runs HERE, before
	// the stub screen shows an id, rather than only at consent, where the
	// self-check's biconditional is the later belt (R0 m1).
	// The error's text IS the §8y body: composerShowRefusal draws err.Error()
	// for an error composerRefusalBody does not map.
	if c.UnspendableRequestUnmet() {
		return md.Composed{}, errors.New(composerCopyLianaUnmet())
	}
	// SPEC §6's mint refusals (F-654), on the mint path and NOT in md's
	// encodePayload, which Reassemble also runs over cards this device reads.
	// The predicate keeps a kind-1 choice off every shape these refuse today;
	// this is the belt for the day it does not.
	if err := c.ValidateUnspendableShape(); err != nil {
		return md.Composed{}, err
	}
	return c, nil
}
```

Apply to `gui/composer_state.go`:

```diff
diff --git a/gui/composer_state.go b/gui/composer_state.go
index 59dfa88..a6e0973 100644
--- a/gui/composer_state.go
+++ b/gui/composer_state.go
@@ -28,6 +28,13 @@ type composerState struct {
 	// value md.Compose lowers. The GUI never builds a descriptor itself.
 	list md.PathList
 
+	// unspendable is the operator's §0b key-path choice (F-449 stage 4). The
+	// zero value is md.UnspendableNums, the wallet every earlier firmware
+	// built. It is set ONLY by composerUnspendableStep, which also resets it
+	// whenever §0b's predicate stops admitting it, so it is never carried into
+	// a shape that cannot use it.
+	unspendable md.UnspendableKind
+
 	// bound is the payload's now: record (§6a, C24): a LOWER bound on the
 	// present, affecting echoes and refusals only, never an encoded operand.
 	bound composerBound
```

Apply to `gui/composer_flow.go`:

```diff
diff --git a/gui/composer_flow.go b/gui/composer_flow.go
index 6ce42e0..c5ed681 100644
--- a/gui/composer_flow.go
+++ b/gui/composer_flow.go
@@ -95,6 +95,12 @@ func composerFlow(ctx *Context, th *Colors) {
 		}
 		composerSizeAssignments(st)
 
+		// SPEC §0b: the key-path choice, BEFORE the chunks exist, because the
+		// kind is part of the Template-ID the stub screen shows. Back returns
+		// to the path list, as Back from the stub screen does.
+		if !composerUnspendableStep(ctx, th, st) {
+			continue
+		}
 		template, err := composerTemplateChunksFor(st)
 		if err != nil {
 			composerShowRefusal(ctx, th, "Template", err)
@@ -265,7 +271,7 @@ func composerDeclaredOrigins(st *composerState) []*md.SlotOrigin {
 // composerTemplateChunksFor emits the keyless template for the current shape
 // and seating.
 func composerTemplateChunksFor(st *composerState) ([]string, error) {
-	c, err := md.ComposeWith(st.list, composerDeclaredOrigins(st))
+	c, err := composerCompose(st, composerDeclaredOrigins(st))
 	if err != nil {
 		return nil, err
 	}
@@ -280,7 +286,7 @@ func composerTemplateChunksFor(st *composerState) ([]string, error) {
 // that had been keyed by its own policy is not a template.
 func composerArtifactsFor(st *composerState) (template, keyed []string, err error) {
 	declared := composerDeclaredOrigins(st)
-	ct, err := md.ComposeWith(st.list, declared)
+	ct, err := composerCompose(st, declared)
 	if err != nil {
 		return nil, nil, err
 	}
@@ -305,7 +311,7 @@ func composerArtifactsFor(st *composerState) (template, keyed []string, err erro
 			fps[uint8(i)] = a.fingerprint
 		}
 	}
-	ck, err := md.ComposeWith(st.list, declared)
+	ck, err := composerCompose(st, declared)
 	if err != nil {
 		return nil, nil, err
 	}
```

Apply to `gui/composer_selfcheck.go`:

```diff
diff --git a/gui/composer_selfcheck.go b/gui/composer_selfcheck.go
index 4b72ee5..49b72d9 100644
--- a/gui/composer_selfcheck.go
+++ b/gui/composer_selfcheck.go
@@ -70,6 +70,14 @@ func composerSelfCheck(st *composerState, chunks []string) error {
 	if !shape.Complete {
 		return errors.New("self-check: the decoded policy cannot be described")
 	}
+	// THE KEY-PATH CHOICE LANDED (F-449 stage 4). The operator chose a kind on
+	// §0b's screen; the decoded card must carry exactly that one. Compared as
+	// a biconditional so it catches both directions: a Liana choice that
+	// composed NUMS, and a NUMS wallet that came out kind 1.
+	if (st.unspendable == md.UnspendableLiana) != (shape.KeyPath == md.KeyPathLianaUnspendable) {
+		return fmt.Errorf("self-check: the key path is %v in the composition and %v decoded",
+			st.unspendable, shape.KeyPath)
+	}
 	leaves := composerLeafPaths(st.list)
 	if len(shape.Branches) != len(leaves) {
 		return fmt.Errorf("self-check: the decoded policy has %d spend paths, the shape has %d",
```

Apply to `gui/composer_copy.go`:

```diff
diff --git a/gui/composer_copy.go b/gui/composer_copy.go
index d121583..835bee4 100644
--- a/gui/composer_copy.go
+++ b/gui/composer_copy.go
@@ -226,6 +226,46 @@ func composerCopyLianaKeyPath() string {
 		"different wallet with different addresses."
 }
 
+// composerCopyUnspendableLead is §0b's COPY, the half common to both rows: the
+// two rows are DIFFERENT WALLETS, and the choice cannot be undone once cut.
+// SHORT, because composerPickScreen draws the lead as a header on every page,
+// and a long one pushes a row off the first page (the hash-kind screen's
+// lesson, TestComposerHashKindScreenDrawsAllFourRows).
+func composerCopyUnspendableLead() string {
+	return "Which key path? The two are DIFFERENT WALLETS, with different " +
+		"addresses. It cannot be changed after engraving."
+}
+
+// composerCopyUnspendableRowNUMS is §0b's first row, and the first-entry
+// default: the wallet every earlier firmware built.
+func composerCopyUnspendableRowNUMS() string {
+	return "NUMS point: Bitcoin Core imports it. Liana and Nunchuk do not."
+}
+
+// composerCopyUnspendableRowLiana is §0b's second row. The coordinators are
+// the ones composerCopyLianaKeyPath names, measured the same way.
+func composerCopyUnspendableRowLiana() string {
+	return "Liana key: Liana (v15.0) and Bitcoin Core import it. Nunchuk only by chance."
+}
+
+// composerCopyLianaKeyDropped is §0b RESET's signal: a kind-1 choice the
+// predicate no longer admits is dropped, and the operator is told which fact
+// moved. composerStubDelta's cause-free banner is not enough alone (§0b).
+func composerCopyLianaKeyDropped(cause string) string {
+	return "LIANA KEY DROPPED\n" + cause + " This policy is back on the NUMS " +
+		"key path: its Template-ID and addresses are not the ones the Liana " +
+		"key gave."
+}
+
+// composerCopyLianaUnmet is the refusal composerCompose raises when a Liana
+// choice composed a real key path instead (SPEC §6 row 3). Unreachable through
+// the flow -- the reset drops such a choice first -- and worded for the day it
+// is not.
+func composerCopyLianaUnmet() string {
+	return "The Liana key was chosen, but this policy has a real key path, " +
+		"so there is no unspendable key to choose. Go back to the key path screen."
+}
+
 // composerCopyMixedLockBases is the fable review r0 lens-2 I-2 notice, in the
 // register of §8g's Liana line: one sentence naming the wallet that refuses
 // and one naming the way round it.
```

- [ ] **Step 4: Run the new tests, then the whole package once.**
```bash
go test -count=1 -run 'TestComposer|Liana|Unspendable|TestEmulatorWalks|Modal' ./gui/
TMPDIR=/scratch/code/shibboleth/.tmp/f449s4-tmp /scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > $TMPDIR/gui-t6.txt 2>&1; tail -1 $TMPDIR/gui-t6.txt
```
Expected: `ok`; then `RESULT: ok -- all 1398 tests ran across 24 shards` (1380 + 18). F12: nothing else moved.

- [ ] **Step 5: Commit** `gui: the §0b key-path choice -- Liana's unspendable key on the device (F-449 stage 4)`.

---

## Task 7 (fork + mnemonic-engrave): the emulator walk — re-greened, and a Liana arm

**This is the acceptance gate for the screens, and it was red before this stage touched anything (F8).** Three repairs first, each measured at the stage-3 end state, then the new arm.

**Which walk, and how it is run.** `design/journeys/capture_composer.py` (mnemonic-engrave) builds `cmd/emu/emu.wasm` from the fork worktree, serves it, and drives `cmd/emu/shots_composer.js` in headless Chromium (Playwright), comparing every screen against host artifacts written by `design/journeys/transcript_composer.sh`. The new arm `liana`: no payload; Wallet Policy → Build → Taproot → `kofn-recovery` → Done; asserts the NEXT screen is `Which key path?` with both rows as the only two tap targets; taps `Liana key`; asserts the stub screen shows `Template-ID: f99cc42e…` and NOT the NUMS twin's `81072164…`; Back, Done again, presses ✓ straight through and asserts the id did not move (re-entry seeds Liana); consent shows `KEY PATH: NONE (LIANA KEY)` and no §8x notice; engraves; compares the engraved string byte for byte with `md encode` of `md compose … --unspendable liana`. The device-vs-Liana ADDRESS leg is Task 4's (a keyed composition needs four seats; this payload has two keys and one seed).

**Files:** fork: `cmd/emu/shots_composer.js`, `cmd/emu/expect_composer.json` (regenerated), `cmd/emu/expect_fixture_test.go`, `gui/walk_copy_anchors_test.go`. engrave: `design/journeys/transcript_composer.sh`, `design/journeys/capture_composer.py`.

- [ ] **Step 1: A current `md` (F9).** Standing permission (keep local binaries current):
```bash
cd /scratch/code/shibboleth/descriptor-mnemonic && git rev-parse --short HEAD    # cf35d61a or later
export PATH=$HOME/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin:$PATH
cargo build --locked --release -p md-cli && ./target/release/md --version       # md 0.19.0
```

- [ ] **Step 2: The host half.** Section 7 of the transcript gates md's version, the Liana template, its one chunk, its Template-ID and the NUMS twin's.

Apply to `design/journeys/transcript_composer.sh` (mnemonic-engrave):

```diff
--- a/design/journeys/transcript_composer.sh
+++ b/design/journeys/transcript_composer.sh
@@ -382,6 +382,35 @@
 echo
 run "$MD" decode "$KEYLESS_MD1"
 
+echo "########## 7. THE LIANA ARM (F-449 stage 4) -- the same kofn-recovery tree, two wallets"
+echo
+echo "The device's key-path choice builds the LEFT one; the right one is what"
+echo "pressing through the NUMS row builds. Different Template-IDs, because the"
+echo "kind bit is part of the tree the id hashes (SPEC §3e)."
+echo
+gate "md is 0.19.0 (the first with --unspendable)" "$("$MD" --version)" "md 0.19.0"
+LIANA_TEMPLATE="$("$MD" compose --wrapper tr --preset kofn-recovery,2of3,older=26280 --unspendable liana 2>/dev/null)"
+NUMS_TWIN="$("$MD" compose --wrapper tr --preset kofn-recovery,2of3,older=26280 2>/dev/null)"
+printf '%s\n' "$LIANA_TEMPLATE" > "$OUT/liana-kofn.template"
+run cat "$OUT/liana-kofn.template"
+gate "liana template" "$LIANA_TEMPLATE" \
+  "tr(UNSPENDABLE(liana),{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})"
+runcap "$OUT/liana-kofn.md1.txt" '^md1' \
+  "$MD" encode "$LIANA_TEMPLATE" --force-chunked --group-size 0
+gate "liana md1 string" "$(cat "$OUT/liana-kofn.md1.txt")" \
+  "md13ls8aqqxq6tvyyykjmpprj6tvyy495kcgfwtsqrq0zjqgsexd9dcqqqv65q3cm0m0nz2h7w9"
+runcap "$OUT/liana-kofn.id.txt" '^(wallet-descriptor-template-id|md1-encoding-id):|^  @' \
+  "$MD" inspect "$(cat "$OUT/liana-kofn.md1.txt")"
+gate "liana Template-ID" \
+  "$(awk -F': ' '/^wallet-descriptor-template-id:/{print $2}' "$OUT/liana-kofn.id.txt")" \
+  "f99cc42e1ff68bae546c4d1070e5963f"
+runcap "$OUT/liana-kofn-nums-twin.id.txt" '^(wallet-descriptor-template-id|md1-encoding-id):' \
+  "$MD" inspect "$("$MD" encode "$NUMS_TWIN" --force-chunked --group-size 0 2>/dev/null | grep '^md1')"
+gate "NUMS twin Template-ID" \
+  "$(awk -F': ' '/^wallet-descriptor-template-id:/{print $2}' "$OUT/liana-kofn-nums-twin.id.txt")" \
+  "8107216456de60d05e57f7fe268824d8"
+run "$MD" decode "$(cat "$OUT/liana-kofn.md1.txt")"
+
 echo "########## Artifacts for the device half"
 run ls -la "$OUT" "$OUT/cards"
 
```

Apply to `design/journeys/capture_composer.py` (mnemonic-engrave):

```diff
--- a/design/journeys/capture_composer.py
+++ b/design/journeys/capture_composer.py
@@ -52,6 +52,8 @@
               "c03-start-from.png", "c04-lock-echo-p0.png", "c05-hash-rule.png",
               "c07-seat-slot0.png", "c08-seat-slot2-seed.png"],
     "keyless": ["c00a-boot-offer.png", "k01-door.png", "k04-census.png"],
+    # F-449 stage 4: the key-path choice screen and the Liana-key census.
+    "liana": ["c00a-boot-offer.png", "l01-key-path.png", "l05-census.png"],
 }
 
 
@@ -232,9 +234,24 @@
     return results
 
 
+def read_liana():
+    """F-449 stage 4's oracle: `md compose --unspendable liana` over kofn-recovery,
+    and the NUMS twin's id, which the walk asserts the device does NOT show."""
+    ids = need("liana-kofn.id.txt")
+    nums = need("liana-kofn-nums-twin.id.txt")
+    template_id = field(ids, "wallet-descriptor-template-id")
+    return {
+        "templateId": template_id,
+        "templateStub": template_id[:8],
+        "numsTemplateId": field(nums, "wallet-descriptor-template-id"),
+        "entries": 1,
+        "strings": need("liana-kofn.md1.txt"),
+    }
+
+
 def main():
     ap = argparse.ArgumentParser()
-    ap.add_argument("--arm", choices=["keyed", "keyless", "both"], default="both")
+    ap.add_argument("--arm", choices=["keyed", "keyless", "liana", "both"], default="both")
     # THE NEGATIVE CONTROL, AS A COMMAND. A comparison nobody has made fail is
     # not evidence it can. This corrupts ONE character of ONE expected address
     # and requires the walk to notice -- exit 0 only if the capture FAILED.
@@ -271,7 +288,7 @@
     # wasm, starts a browser, or needs a device. That is the point -- the
     # fixture has to be cheap to regenerate or it will not be regenerated.
     if a.emit_expect or a.check_expect:
-        built = {"keyed": read_keyed(), "keyless": read_keyless()}
+        built = {"keyed": read_keyed(), "keyless": read_keyless(), "liana": read_liana()}
         blob = json.dumps(built, indent=2, sort_keys=True) + "\n"
         target = a.emit_expect or a.check_expect
         if a.emit_expect:
@@ -324,6 +341,11 @@
         print(f"host: key-less template id {kl['templateId']}, "
               f"{len(kl['strings'])} md1 string(s), {len(kl['strings'][0])} chars")
 
+    if a.arm in ("liana", "both"):
+        li = read_liana()
+        legs.append(("liana", "liana", None, li))
+        print(f"host: Liana-key template id {li['templateId']} (NUMS twin {li['numsTemplateId']})")
+
     print(f"emulator: {EMU}")
     if not a.no_build:
         build_wasm()
```

```bash
cd /scratch/code/shibboleth/mnemonic-engrave/design/journeys
FORK=/scratch/code/shibboleth/sh-worktrees/f449-stage4 ./transcript_composer.sh > transcript_composer.txt 2>&1; echo $?
```
Expected: `0`, every `GATE PASS`, including `liana md1 string = md13ls8aqqxq6tvyyykjmpprj6tvyy495kcgfwtsqrq0zjqgsexd9dcqqqv65q3cm0m0nz2h7w9`, `liana Template-ID = f99cc42e1ff68bae546c4d1070e5963f`, `NUMS twin Template-ID = 8107216456de60d05e57f7fe268824d8`.

- [ ] **Step 3: The device half.** The three repairs are commented in place (F-544's `Restore Doc` handler; the keyless no-address needle; the paged form-B census), then the `liana` arm. The fixture test learns the one liana-only field; the walk-anchor table learns the one non-composer string the keyless arm now waits for.

Apply to `cmd/emu/shots_composer.js`:

```diff
diff --git a/cmd/emu/shots_composer.js b/cmd/emu/shots_composer.js
index 65a405c..ccaff8e 100644
--- a/cmd/emu/shots_composer.js
+++ b/cmd/emu/shots_composer.js
@@ -273,6 +273,11 @@ const ENGRAVE_HANDLERS = [
   { name: "engrave-done", match: "Engravingcompletedsuccessfully", act: "confirm" },
   { name: "choose-variant", match: "Chooseengraving", act: "confirm" },
   { name: "bundle-engraved", match: "Bundleengraved", act: "confirm" },
+  // F-544 (fork 5971ad3, 2026-09-16) ends every composer run with the restore
+  // document, AFTER the bundle modal and BEFORE the door. This list predates it,
+  // so from that commit on every arm STALLED here -- measured at the F-449
+  // stage-3 base, where nothing ran this walk. Its checkmark leaves it.
+  { name: "restore-doc", match: "RestoreDoc", act: "confirm" },
 ];
 
 const DOOR_ROW = "Buildanewpolicy";
@@ -500,7 +505,7 @@ export async function run({ shotURL = "http://127.0.0.1:8732", arm = "keyed",
         "expect_composer.json with capture_composer.py --emit-expect");
     }
     const all = await res.json();
-    expect = arm === "keyless" ? all.keyless : all.keyed[form];
+    expect = arm === "keyless" ? all.keyless : arm === "liana" ? all.liana : all.keyed[form];
     if (!expect) {
       throw new Error(`expect_composer.json has no entry for arm=${arm} form=${form}`);
     }
@@ -563,7 +568,11 @@ export async function run({ shotURL = "http://127.0.0.1:8732", arm = "keyed",
     must(consent.joined, "KEY PATH: NONE (NUMS)", "the consent's key-path line");
     must(consent.joined, `Template-ID: ${expect.templateId}`, "the consent's Template-ID");
     must(consent.joined, `mk1 stub (template): ${expect.templateStub}`, "the consent's stub");
-    must(consent.joined, "Keyless template - no addresses.", "the no-addresses line");
+    // "Template has no keys", not "Keyless template": the composed template
+    // carries a Keys entry per slot (origins, no xpub), so noAddressLines takes
+    // its !allSlotsHaveXpub arm. Measured at the F-449 stage-3 base, where the
+    // old needle failed.
+    must(consent.joined, "Template has no keys - no addresses.", "the no-addresses line");
     must(consent.joined, "Verify off-device.", "the verify-off-device line");
 
     await tap(CONFIRM, 500);
@@ -608,6 +617,96 @@ export async function run({ shotURL = "http://127.0.0.1:8732", arm = "keyed",
     };
   }
 
+  if (arm === "liana") {
+    // F-449 stage 4, SPEC_liana_unspendable_internal_key §0b on the emulator:
+    // the key-path CHOICE, reached BY TOUCH, placed before the stub screen,
+    // seeded from the current value on re-entry, and the Liana-key template
+    // engraved byte for byte equal to `md compose --unspendable liana`. No
+    // payload, as the keyless arm: the template alone is the artifact, and
+    // the device-vs-Liana ADDRESS leg is TestDeviceDerivesLianasOwnAddresses-
+    // ForKind1 (a keyed composition needs four seats this payload cannot fill).
+    await bootAndChoosePayload(shotURL, taken, "none");
+    await goTo("Wallet Policy");
+    await tap(CONFIRM, 450);
+    await waitFor("Build a new policy");
+    await chooseRow(1, "Which script?", "Build a new policy");
+    await chooseRow(0, "Start from?", "Taproot (tr)");
+    // Start from?: row 0 Build my own paths, 1 plain-multisig,
+    // 2 simple-timelocked-inheritance, 3 kofn-recovery.
+    await chooseRow(3, "Add a spend path", "kofn-recovery");
+    const pathList = window.shScreen();
+    must(pathList, "Path 1: 2-of-3", "the kofn-recovery primary");
+
+    // PLACEMENT: after Done comes the key-path choice, not the stub screen.
+    // Two paths, so no key-order question sits between them.
+    const nRows = window.shTargets().length;
+    await chooseRow(nRows - 1, "Which key path?", "Done");
+    const choice = window.shScreen();
+    mustNot(choice, "Template-ID", "the stub screen drew before the key-path choice");
+    must(choice, "DIFFERENT WALLETS", "the choice's different-wallets sentence");
+    must(choice, "NUMS point", "the NUMS row");
+    must(choice, "Liana key", "the Liana row");
+    // THE FIRST-PAGE GATE, on the frame the operator sees: both rows tappable.
+    if (window.shTargets().length !== 2) {
+      throw new Error(`the key-path screen offers ${window.shTargets().length} tappable ` +
+        `row(s) on its first page, want 2.\nScreen: ${JSON.stringify(choice)}`);
+    }
+    taken.push(await screenShot(shotURL, "l01-key-path.png"));
+    await chooseRow(1, "Template-ID", "Liana key");
+    let stub = await readAllPages(shotURL, "l02-stub-p");
+    taken.push(...stub.names);
+    must(stub.joined, `Template-ID: ${expect.templateId}`, "the Liana Template-ID");
+    mustNot(stub.joined, `Template-ID: ${expect.numsTemplateId}`, "the NUMS twin's id");
+    proven.push("the stub screen shows the kind chosen BEFORE it (placement)");
+
+    // RE-ENTRY: Back to the path list, Done again, and press straight through.
+    // The screen must reopen on the Liana row, so the id must not move.
+    await tap(BACK, 450);
+    await waitFor("Add a spend path");
+    const nRows2 = window.shTargets().length;
+    await chooseRow(nRows2 - 1, "Which key path?", "Done (again)");
+    await tap(CONFIRM, 450);
+    await waitFor("Template-ID");
+    stub = await readAllPages(shotURL, "l03-stub-again-p");
+    taken.push(...stub.names);
+    must(stub.joined, `Template-ID: ${expect.templateId}`,
+      "pressing through the re-entered choice changed the wallet");
+    proven.push("re-entry opens on the current kind");
+
+    await tap(CONFIRM, 450);
+    await waitFor("Seat keys into this template?");
+    await chooseRow(0, "Review", "Engrave a key-less template");
+    const consent = await readAllPages(shotURL, "l04-consent-p");
+    taken.push(...consent.names);
+    must(consent.joined, "KEY PATH: NONE (LIANA KEY)", "the kind-1 key-path line");
+    mustNot(consent.joined, "KEY PATH: NONE (NUMS)", "the kind-0 line on a kind-1 wallet");
+    mustNot(consent.joined, "OUTSIDE LIANA'S MODEL", "the Liana notice on a shape Liana imports");
+    must(consent.joined, `Template-ID: ${expect.templateId}`, "the consent's Template-ID");
+
+    await tap(CONFIRM, 500);
+    await waitFor("Nothing outside this device");
+    window.shPress(...CONFIRM);
+    await sleep(HOLD_MS);
+    window.shRelease(...CONFIRM);
+    await waitFor("No slot is seated");
+    await tap(CONFIRM, 450);
+    const censusScreen = await waitFor("Plates To Cut");
+    must(censusScreen, "This engraves 1 plate.", "the census claim");
+    taken.push(await screenShot(shotURL, "l05-census.png"));
+    const claim = censusClaimOf(censusScreen);
+    await tap(CONFIRM, 500);
+    const tail = await runEngraveTail({ shotURL, prefix: "l06-",
+      variant: { rows: ["TEXT + QR", "TEXT ONLY", "QR ONLY"], take: 0 } });
+    taken.push(...tail.shots);
+    const flat = compareEngraved(tail.census, claim, expect, "the Liana-key template plate");
+    return {
+      arm, shots: taken, elapsedSec: Math.round((performance.now() - t0) / 1000),
+      consentPages: consent.pages.length, censusClaim: claim, engraved: flat,
+      needlesProven: proven,
+      matched: { templateId: expect.templateId, templateStub: expect.templateStub, strings: flat },
+    };
+  }
+
   // ─── The KEYED arm ─────────────────────────────────────────────────────────
   await bootAndChoosePayload(shotURL, taken, "composer");
   await loadComposerPayload(shotURL, taken, expect);
@@ -850,11 +949,18 @@ export async function run({ shotURL = "http://127.0.0.1:8732", arm = "keyed",
   must(modePick, "Full", "the full-mode row (asked because a seed-seated slot exists)");
   await chooseRow(1, "Plates To Cut", "Watch-only (keys)");
 
-  const censusScreen = window.shScreen();
+  // THE CENSUS IS PAGED, and composerReadScreen WITHHOLDS its checkmark until
+  // the last page has been laid out once. Form B's five lines plus F-497's scope
+  // line run to a second page, so a bare CONFIRM on page 0 did nothing and the
+  // engrave tail stalled on "Plates To Cut" -- measured at the F-449 stage-3
+  // base. readAllPages pages to the end and wraps, which arms the checkmark.
+  taken.push(await screenShot(shotURL, `c12-census-${form}.png`));
+  const census = await readAllPages(shotURL, `c12-census-${form}-p`);
+  taken.push(...census.names);
+  const censusScreen = census.joined;
   for (const line of (expect.censusLines || [])) {
     must(censusScreen, line, "the census screen");
   }
-  taken.push(await screenShot(shotURL, `c12-census-${form}.png`));
   const claim = censusClaimOf(censusScreen);
 
   // (20) The engrave loop, and the byte comparison.
```

Apply to `cmd/emu/expect_fixture_test.go`:

```diff
diff --git a/cmd/emu/expect_fixture_test.go b/cmd/emu/expect_fixture_test.go
index 5c40d2e..5820483 100644
--- a/cmd/emu/expect_fixture_test.go
+++ b/cmd/emu/expect_fixture_test.go
@@ -59,12 +59,24 @@ func TestShotsComposerExpectFixtureCoversEveryFieldTheWalkReads(t *testing.T) {
 	var all struct {
 		Keyed   map[string]map[string]json.RawMessage `json:"keyed"`
 		Keyless map[string]json.RawMessage            `json:"keyless"`
+		Liana   map[string]json.RawMessage            `json:"liana"`
 	}
 	if err := json.Unmarshal(blob, &all); err != nil {
 		t.Fatalf("expect_composer.json does not parse: %v", err)
 	}
 
-	// The keyed arm drives both forms and reads EVERY field.
+	// F-449 stage 4's liana arm reads ONE field no other arm has: the NUMS
+	// twin's Template-ID, which it asserts the stub screen does NOT show. It is
+	// named here, so "the keyed arm reads every field" stays a checked claim
+	// about every OTHER field rather than a rule the new arm quietly broke.
+	lianaOnly := map[string]bool{"numsTemplateId": true}
+	for f := range lianaOnly {
+		if !seen[f] {
+			t.Errorf("%q is exempted as liana-only but the walk no longer reads it", f)
+		}
+	}
+
+	// The keyed arm drives both forms and reads every field but those.
 	for _, form := range []string{"A", "B"} {
 		got, ok := all.Keyed[form]
 		if !ok {
@@ -72,12 +84,22 @@ func TestShotsComposerExpectFixtureCoversEveryFieldTheWalkReads(t *testing.T) {
 			continue
 		}
 		for _, f := range fields {
+			if lianaOnly[f] {
+				continue
+			}
 			if _, ok := got[f]; !ok {
 				t.Errorf("keyed form %s is missing %q, which shots_composer.js reads", form, f)
 			}
 		}
 	}
 
+	// The liana arm, like the key-less one, composes no keys: its five fields.
+	for _, f := range []string{"templateId", "templateStub", "numsTemplateId", "entries", "strings"} {
+		if _, ok := all.Liana[f]; !ok {
+			t.Errorf("the liana arm is missing %q", f)
+		}
+	}
+
 	// The key-less arm runs before any policy is composed, so it legitimately
 	// carries only the four fields its own leg compares. Naming them here is
 	// what keeps "fewer fields" from silently becoming "no fields".
```

Apply to `gui/walk_copy_anchors_test.go`:

```diff
diff --git a/gui/walk_copy_anchors_test.go b/gui/walk_copy_anchors_test.go
index 30b9e63..8277c30 100644
--- a/gui/walk_copy_anchors_test.go
+++ b/gui/walk_copy_anchors_test.go
@@ -209,6 +209,7 @@ var notComposerCopy = map[string]string{
 	"FROM PAYLOAD":                             "screen title, menu row, computed summary or another program's copy",
 	"Keep this payload loaded?":                "screen title, menu row, computed summary or another program's copy",
 	"Keyless template - no addresses.":         "screen title, menu row, computed summary or another program's copy",
+	"Template has no keys - no addresses.":     "noAddressLines (wallet_policy.go), shared with the Wallet Policy program; not composer copy",
 	"Keys loaded: 2, plus 1 seed.":             "screen title, menu row, computed summary or another program's copy",
 	"Leave unseated":                           "screen title, menu row, computed summary or another program's copy",
 	"Load Payload":                             "screen title, menu row, computed summary or another program's copy",
```

Regenerate the fixture from the host, then check it:
```bash
cd /scratch/code/shibboleth/mnemonic-engrave/design/journeys
python3 capture_composer.py --emit-expect /scratch/code/shibboleth/sh-worktrees/f449-stage4/cmd/emu/expect_composer.json --emu /scratch/code/shibboleth/sh-worktrees/f449-stage4/cmd/emu
python3 capture_composer.py --check-expect /scratch/code/shibboleth/sh-worktrees/f449-stage4/cmd/emu/expect_composer.json --emu /scratch/code/shibboleth/sh-worktrees/f449-stage4/cmd/emu
```
Expected: `ok: … matches what the host derives`. The regenerated file differs from the committed one by exactly this diff, which is what the apply gate applies (`keyed` and `keyless` byte-identical; only `liana` is new):

Apply to `cmd/emu/expect_composer.json`:

```diff
diff --git a/cmd/emu/expect_composer.json b/cmd/emu/expect_composer.json
index d71b5e4..13d77dd 100644
--- a/cmd/emu/expect_composer.json
+++ b/cmd/emu/expect_composer.json
@@ -67,5 +67,14 @@
     ],
     "templateId": "e0863d3ccac31a64d3b5e14b85ccd6c0",
     "templateStub": "e0863d3c"
+  },
+  "liana": {
+    "entries": 1,
+    "numsTemplateId": "8107216456de60d05e57f7fe268824d8",
+    "strings": [
+      "md13ls8aqqxq6tvyyykjmpprj6tvyy495kcgfwtsqrq0zjqgsexd9dcqqqv65q3cm0m0nz2h7w9"
+    ],
+    "templateId": "f99cc42e1ff68bae546c4d1070e5963f",
+    "templateStub": "f99cc42e"
   }
 }
```

```bash
cd /scratch/code/shibboleth/sh-worktrees/f449-stage4 && go test -count=1 ./cmd/emu/ && node --check cmd/emu/shots_composer.js
go test -count=1 -run TestEmulatorWalks ./gui/
```

- [ ] **Step 4: Run the walk, all arms.**
```bash
cd /scratch/code/shibboleth/mnemonic-engrave/design/journeys
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH
python3 capture_composer.py --arm both --emu /scratch/code/shibboleth/sh-worktrees/f449-stage4/cmd/emu --port 8931 --shot-port 8932; echo $?
```
Expected, measured: `0` and `all legs matched the host.` — keyed-A (7 strings), keyed-B (9), keyless (`md1fkzyy…`), **liana (`md13ls8a…`, 28 s, 11 shots)**. Then the negative control, which must still pass: `python3 capture_composer.py --arm keyed --prove-it-can-fail …` → `NEGATIVE CONTROL PASSED`.

**The liana arm can fail, measured:** with `composerUnspendableStep` moved below `composerTemplateChunksFor` (the placement mutation), the arm fails with `the Liana Template-ID: the screen does not carry "Template-ID: f99cc42e1ff68bae546c4d1070e5963f"`.

- [ ] **Step 5: Commit** fork: `emu: the composer walk runs again, and walks the Liana key (F-449 stage 4)`; engrave (paths explicitly, not the regenerated `out/` or `shots/`): `journeys: the composer transcript and capture learn the Liana arm (F-449 stage 4)`.

---

## Task 8 (fork): the whole gate, the size, the mutation pass, and landing

- [ ] **Step 1: The whole gate, once, captured.**
```bash
cd /scratch/code/shibboleth/sh-worktrees/f449-stage4
export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp/f449s4-tmp
gofmt -l . | sort > $TMPDIR/gofmt.txt; diff <(printf '%s\n' gui/transaction.go gui/transaction_golden_test.go gui/transaction_txrecord_test.go mt/mt.go mt/mt_test.go) $TMPDIR/gofmt.txt
go vet ./md/ && go vet ./gui/ 2>&1 | grep -v ArtifactDir | grep -v '^#'
go test -count=1 $(go list ./... | grep -v '/gui$') > $TMPDIR/nongui.txt 2>&1; grep -vE '^ok|no test files' $TMPDIR/nongui.txt; grep -c '^ok' $TMPDIR/nongui.txt
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 > $TMPDIR/gui.txt 2>&1; tail -1 $TMPDIR/gui.txt
```
Expected, measured at the end state: empty gofmt diff; vet clean but the two baseline lines; **55** `ok` and nothing else; `all 1398 tests ran`; md **188**.

- [ ] **Step 2: Firmware size.**
```bash
export PATH=/nix/var/nix/profiles/default/bin:$PATH
nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```
Measured: base **1,658,284 B flash / 63,352 B RAM**, end **1,665,828 / 63,376** (**+7,544 / +24**; the Liana recipe is live code from Task 3 on). If `nix develop` refuses the worktree's flake, `nix develop /scratch/code/shibboleth/seedhammer -c …` is how these were taken. Record both lines in the merge commit.

- [ ] **Step 3: The mutation pass.** Re-run the Self-Review's table against the real tree. For each row: apply, CHECK it applied (the text existed exactly once), check it COMPILED (a build failure reads as a pass in a naive grep), run the named test, confirm it fails, revert with `git checkout -- <file>` on a clean tree only. S8 must survive (it is the named gap); anything else surviving is a finding.

- [ ] **Step 4: Land.** Merge `f449-stage4` into fork `main` (`--no-ff`), then `./scripts/push-via-staging.sh` from the fork checkout. The push is the last action on the fork in the turn. **No flash.** A signed image may be pre-built for the operator (`~/bin/sh/sh2-flash -b`), and the live walk with the operator on the board (rows 1-9 and 13 of the journey table) is the on-device acceptance.

---

## Task 9 (mnemonic-engrave): evidence, follow-ups, and the spec reconciled

- [ ] **Step 1: Commit the evidence behind F6 and F7** to `design/evidence/f449-stage4/`, reproducibly. The Liana leg (after `scripts/liana-live-gate.sh` PASSes, which builds the harness):
```bash
H=harnesses/liana/target/gate-build/debug/liana-harness      # or $LIANA_GATE_TARGET/debug/liana-harness
A="[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*"
B="[3f635a63/48'/0'/0'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*"
C="[66d455ea/48'/0'/0'/3']xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL/<0;1>/*"
D="[73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9igdwdnmCSoaPVd4ZNZnvfgUsGvQ8AbNAhEmfBEMCMHctwZBuxWK8HkjqUW5F72MCSJCFfisVwRY62Kb1FuDZ66nNQe1/<0;1>/*"
E="[73c5da0a/48'/0'/2'/3']xpub6DXuQW1Q2JpZzZHXLadWbvXrMTD8ysfE7L4YZHsEvpWQ3KQ7CVporF7mSQKcphivSAdwGdLuLLHvrgaQXUeNMwpz5c1HAJHXgxvesUgj4Mb/<0;1>/*"
K="xpub661MyMwAqRbcFn1aHFGgZ359mzBqgj6rjSy73VLCdi92kFK4uHH71MRMGuWX5LqsouhUowavHR6PpPyxjctLm96D6JSZyPAr9RufepmuyEz/<0;1>/*"   # any xpub: `unspendable` replaces it
ev=design/evidence/f449-stage4; mkdir -p $ev
cat > $ev/liana-probes-in.txt <<EOF
tr($K,{multi_a(2,$A,$B,$C),multi_a(2,$D,$E)})
tr($K,{multi_a(2,$A,$B),and_v(v:pk($C),after(800000))})
tr($K,{multi_a(2,$A,$B),{and_v(v:pk($C),older(100)),and_v(v:pk($D),older(100))}})
tr($K,multi_a(2,$A,$B,$C))
tr($K,and_v(v:pk($A),older(26280)))
tr($K,{multi_a(2,$A,$B),{and_v(v:pk($C),older(26280)),and_v(v:pk($D),older(52560))}})
tr($K,{pk($A),and_v(v:pk($B),older(26280))})
EOF
"$H" unspendable < $ev/liana-probes-in.txt > $ev/liana-probes-recomputed.txt
python3 - $ev <<'PY'
import json, sys
ev = sys.argv[1]
names = ["f644-two-unlocked-primaries", "f644-after-recovery", "f644-duplicate-older", "f644-no-recovery",
         "s7-single-timelocked-leaf", "nested-two-recoveries", "pk-primary-leaf-plus-recovery"]
with open(f"{ev}/liana-probes-parse-in.jsonl", "w") as f:
    for n, d in zip(names, [l.strip() for l in open(f"{ev}/liana-probes-recomputed.txt") if l.strip()]):
        f.write(json.dumps({"name": n, "variant": "liana-unspendable-xpub", "desc": d}) + "\n")
PY
"$H" parse < $ev/liana-probes-parse-in.jsonl > $ev/liana-probes-out.jsonl
python3 -c "import json;[print(r['name'],r['ok']) for r in map(json.loads,open('$ev/liana-probes-out.jsonl'))]"
```
Expected, measured with Liana v15.0 at `4684d5cb`: the first five `False` ("Descriptor is not compatible with a Liana spending policy."), the last two `True`. Every recomputed internal key differs from `K` except probe 1's (whose leaves happen to be the stale-key control's own). The Core leg (F7), on an offline mainnet node with a throwaway datadir (no peers, no chain: an import and three `getnewaddress` need neither):
```bash
B=/scratch/code/bitcoin/build/bin; DD=/scratch/code/shibboleth/.tmp/f449s4-core-dd; mkdir -p $DD
cli() { $B/bitcoin-cli -datadir=$DD -rpcport=18977 -rpcuser=u -rpcpassword=p "$@"; }
$B/bitcoind -datadir=$DD -connect=0 -listen=0 -rpcport=18977 -rpcuser=u -rpcpassword=p -daemon -server; until cli getblockcount >/dev/null 2>&1; do sleep 1; done
cli -named createwallet wallet_name=k1 disable_private_keys=true blank=true
D=$(sed -n 2p design/evidence/f449-stage2/liana-live-gate-expected.jsonl | python3 -c 'import json,sys; print(json.load(sys.stdin)["liana_desc"])')
DC=$(cli getdescriptorinfo "$D" | python3 -c 'import json,sys; print(json.load(sys.stdin)["descriptor"])')
{ $B/bitcoind -version | head -1; cli -rpcwallet=k1 importdescriptors '[{"desc":"'"$DC"'","timestamp":"now","active":true,"range":[0,5]}]'
  for i in 0 1 2; do cli -rpcwallet=k1 getnewaddress "" bech32m; done; } | tee $ev/core-kofn-import.txt
sed -n 2p design/evidence/f449-stage2/liana-live-gate-expected.jsonl | python3 -c 'import json,sys; print(*json.load(sys.stdin)["receive"], sep=chr(10))'
cli stop
```
Expected, measured (`Bitcoin Core daemon version v30.99.0-64a7c7cbb975`): `[{"success":true}]`, then `bc1pj6davmee…`, `bc1ptg5kylzs…`, `bc1p5xnuzf59…`, equal to the record's `receive`. Commit: `evidence: Liana v15.0 refuses the F-644 shapes and SPEC §7's constructed shape; Core imports kind 1 (F-449 stage 4)`.

- [ ] **Step 2: Follow-ups.** Re-grep for the next free ID first (F-659 at `fe73cca6`).
  - **F-654 → CLOSED** in the fork merge SHA: `md.ComposeWithUnspendable` (Task 2), `ValidateUnspendableShape` called from `composerCompose` (Task 6), not from `encodePayload`; `validate_minimal_wire_version`, which F-654's second bullet also names, is unported BY DESIGN: the Go encoder takes no version parameter (it derives the version from the tree, stage 3), so the state that function refuses cannot be constructed in Go. Say so in the closing line, so a later reader does not port it as missed.
  - **F-644 → ruling.** Device half CLOSED: §0b's conjunct 2 names a class for all four shapes, so the device never offers the Liana key for them, and Liana v15.0 refuses each with its own recomputed key (`design/evidence/f449-stage4/`). The md-cli half (no warning from `md compose --unspendable liana`) is **re-owned to the next descriptor-mnemonic release**, with the evidence now gathered as F-644's rule requires; its fix keys each warning on the composed shape.
  - **F-633 → Status line:** copy half done at stage 4 (§8x "as of v15.0"; the kind-1 key-path body; §8f's Liana sentence); OPEN for the KNOWN_RELEASES gate, coordinator-compat cycle.
  - **New (next free ID), owning phase none — ownerless UX residue:** `composerRestoreDoc` says "plus its key cards … listed below" on a run that cut a template and no cards (F14), measured on the emulator's liana and keyless arms.
  - **New (next free ID), owning phase: the composer cycle:** the composer emulator walk is not run by anything on push, and was red on three counts for a week (F8). Direction: run the keyless and liana arms (no payload, ~30 s each) in a scheduled or pre-push job, or name the walk in the flash checklist.
  - `scripts/followups-status.sh` → `OK`.

- [ ] **Step 3: The reconciliation sweep** (the standing rule): grep for the superseded claims and rewrite each against the tree, citing SHAs.
  - **SPEC §7:** "Name it `NumsXpub`" → Rust shipped `KeyPathKind::LianaUnspendable` (`cf35d61a`, `policy_shape.rs:145-162`) and Go mirrors it (`md.KeyPathLianaUnspendable`). Keep the reason (opus M3) as history.
  - **SPEC §0b PLACEMENT/RESET:** the line citations `:95`/`:96-128`/`:98`/`:129` → re-resolve at the fork merge SHA; add that the step sits between `composerSizeAssignments` and `composerTemplateChunksFor`.
  - **SPEC §7a:** add a **Status:** line: the third branch is `taprootInternalKey`'s Liana arm; the engrave half is `md1KeyPathUnknown` in `walletPolicyConsentLines`, unreachable until a fourth kind exists.
  - **SPEC §8.9's four stage-4 rows:** name the tests (`TestComposerKeyPathChoiceIsPlacedBeforeTheChunks`, `TestComposerUnspendableResetIsThePredicate`, `TestComposerUnspendableDefaultRow`, `TestComposerUnspendableScreenFirstPage`) and the emulator arm.
  - **SPEC §9's stage-4 row:** a **Status:** line with the fork merge SHA; what landed beyond the row (the self-check biconditional; the walk re-greened; F-644's device half closed).
  - **SPEC_wallet_policy_composer.md §8:** §8f and §8x amended, §8y added, VERBATIM from `composerCopyTable` (the table is the diff target and says so).
  - Commit the sweep separately from the follow-ups.


---

## What no gate covers

- **The drawn pixels of the new screens.** Go tests read extracted text; the first-page gate counts tap targets, which is geometry, but clipping inside a row is not seen. The emulator walk's screenshots (`l01-key-path.png` and the `l02-`…`l06-` frames) are where it is seen, by a person.
- **Hardware.** Nothing is flashed. The live journey walk with the operator on the board (placement, touch accuracy on the two long rows, the drop modal) is the on-device acceptance and waits for the operator.
- **§7a.3's engrave half is unreachable today.** No decoder yields a fourth internal-key kind (the version set {4, 8} admits three). `md1KeyPathUnknown` is tested on a constructed `Template`, and md's `rootKeyPath` is tested on a constructed tree, but **deleting the call in `walletPolicyConsentLines` is invisible to every test** (Self-Review S8, the one survivor). It stays because SPEC §7a.3 requires it; it becomes testable end to end the day a fourth kind exists.
- **§6 rows 2 and 4 in the composer** (use-site, nesting) are unreachable through `composerCompose`: `TestTheComposerCannotReachSpecSixRowsTwoAndFour` pins WHY, so the day they become reachable, that test fails first.
- **Nunchuk and Bitcoin Core release behaviour for kind 1.** Core was measured on a development build (F7); Nunchuk is from SPEC §7's reading of libnunchuk, never run on kind 1.
- **The changed-id banner for a kind change** (Review Focus 1): existing mechanism, not re-tested for this cause.
- **The walk is not CI.** `capture_composer.py` runs a browser; nothing runs it on push, which is how F8 accumulated. Task 9 files that.

---

## Self-Review

**Spec coverage (§9's stage-4 row):**
- §7 `KeyPathKind` sibling: **Task 3** (`KeyPathLianaUnspendable`, Rust's name, F1).
- Class-2 + unlocked-path ruling: **Task 5** (`TestComposerLianaClassRulings`, both rulings separately; F2: zero code).
- Print-site arms including `md1Summary`: **Task 5** (consent, template-engrave summary, inspect) + the inspect route test.
- F-633 copy: **Task 5** ("as of v15.0" on §8x, a new kind-1 key-path body that does not repeat §8f's Nunchuk claim, §8f's Liana sentence pointed at the device's own choice). The KNOWN_RELEASES gate half stays with the coordinator-compat cycle (Task 9 records it).
- §7a.2's third address branch: **Task 4**. §7a.3's refusal: address half **Task 4** (`taprootInternalKey`), engrave half **Task 5** (`md1KeyPathUnknown`).
- §0b predicate, placement, reset, default row, copy: **Task 6**, and on the emulator **Task 7**.
- F-654 (the Go composer's Liana choice + §6 refusals): **Task 2** (code), **Task 6** (the call), closed in **Task 9**.
- F-644: device half closed by §0b's conjunct 2 with Liana evidence (F6); md-cli half re-owned (Task 9).
- Gates: §8.3 device leg (Task 4, two tests), §7's constructed shape (Task 5), a kind the device cannot derive (Task 4), the predicate on all six presets (Task 6), §8.9's four stage-4 rows (Task 6, plus the emulator arm).

**Mutation table.** Measured against the end state; every mutation applied (the replaced text existed exactly once), compiled, and was caught by the named test, except S8.

| # | file: mutation | caught by |
| --- | --- | --- |
| M1 | `md/compose.go`: the `UnspendableLiana` arm yields `InternalKeyNUMS` | `TestComposeLianaReproducesTheRustVectors`, `TestALianaRequestOverARealKeyIsUnmet`, `TestValidateUnspendableShapeRefusesSpecSixRows` (and `TestComposeLianaTemplateEqualsTheHost`) |
| M2 | `UnspendableRequestUnmet` returns `false` | `TestALianaRequestOverARealKeyIsUnmet` (real-key and wsh rows) |
| M3 | `UnspendableRequestUnmet` ignores what was built (`requested == Liana`) | `TestALianaRequestOverARealKeyIsUnmet` (kofn row), `TestComposeLianaReproducesTheRustVectors` |
| M4 | `validateUnspendableShape`: the sortedmulti_a row returns nil | `TestValidateUnspendableShapeRefusesSpecSixRows` |
| M5 | `isStandardMultipath` returns true | `TestValidateUnspendableShapeRefusesSpecSixRows` |
| M6 | `rejectNestedUnspendable` never refuses | `TestValidateUnspendableShapeRefusesSpecSixRows` |
| M7 | `keyPathOf`: Liana → `KeyPathNUMS` | `TestKeyPathNamesTheLianaKind` |
| M8 | `summarize` drops `KeyPath: rootKeyPath(d.tree)` | `TestKeyPathNamesTheLianaKind` |
| M9 | `keyPathOf`'s fall-through returns `(KeyPathNUMS, true)` | `TestAnUnknownInternalKeyKindIsNotDescribed` |
| M10 | `LianaUnspendableKeyFor` hashes no leaves | `TestLianaUnspendableKeyForIsTheRecipeOverTheSuppliedLeaves` |
| M11 | `finishComposed` writes a hardened wildcard | `TestTheComposerCannotReachSpecSixRowsTwoAndFour` |
| A1 | `gui/policy_address.go`: the Liana arm returns `address.NUMSInternalKey()` (SPEC §7a row 1) | `TestDeviceDerivesLianasOwnAddressesForKind1` (all 5), `TestEveryKeyedVectorReachesAnAddress` (both), `TestAnUnderivableInternalKeyKindIsRefusedNotFallenBack` |
| A2 | `taprootInternalKey` falls back to NUMS for an unknown kind | `TestAnUnderivableInternalKeyKindIsRefusedNotFallenBack` |
| A3 | the Liana key's range `<1;2>` instead of `<0;1>` | `TestDeviceDerivesLianasOwnAddressesForKind1`, `TestEveryKeyedVectorReachesAnAddress` |
| C1 | `gui/composer_consent.go`: the Liana kind counted as an unlocked path | `TestComposerLianaClassRulings` (§7's constructed shape reads ""), `TestComposerUnspendablePredicateOnEveryTrPreset` |
| C2 | class 2 as `shape.KeyPath != md.KeyPathSpendable` (Liana grouped with NUMS) | `TestComposerLianaClassRulings`, `TestComposerUnspendablePredicateOnEveryTrPreset` |
| S1 | the consent's Liana arm deleted | `TestEveryKeyPathPrintSiteNamesTheLianaKind` |
| S2 | the template-engrave summary's Liana arm deleted | `TestEveryKeyPathPrintSiteNamesTheLianaKind` |
| S3 | `md1KeyPathLine` names Liana as NUMS | `TestEveryKeyPathPrintSiteNamesTheLianaKind` |
| S7 | `md1KeyPathUnknown` returns false | `TestAnUnnamedKeyPathIsRefusedBeforeConsent` |
| **S8** | **the `md1KeyPathUnknown` call in `walletPolicyConsentLines` deleted** | **SURVIVES** (the whole gui package runs; no test fails) — the named gap: no decoder yields a fourth kind, see "What no gate covers" |
| P1 | `gui/composer_unspendable.go`: conjunct 1 deleted | `TestComposerUnspendablePredicateOnEveryTrPreset` (fires on simple-timelocked-inheritance) |
| P2 | conjunct 2 deleted | `TestComposerUnspendablePredicateOnEveryTrPreset` |
| P3 | the classifier run on the NUMS composition (class 2 not skipped) | `TestComposerUnspendablePredicateOnEveryTrPreset` |
| D1 | seed `initial := 0`, a constant | `TestComposerUnspendableDefaultRow` (re-entry), `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` |
| D2 | seed `initial := 1`, a constant | `TestComposerUnspendableDefaultRow` (first entry) |
| D3 | md's `UnspendableNums`/`UnspendableLiana` constants swapped (the zero-value trap) | `TestComposerUnspendableDefaultRow` (first entry) |
| R1 | the reset assignment deleted | `TestComposerUnspendableResetIsThePredicate` |
| R2 | reset on every entry | `TestComposerUnspendableResetIsThePredicate` (converse half) |
| R3 | the drop signal deleted (silent reset) | `TestComposerUnspendableResetIsThePredicate` |
| F1 | the lead lengthened ~6x | `TestComposerUnspendableScreenFirstPage` (1 tap target on page 1) |
| L1 | `composer_flow.go`: the step moved below `composerTemplateChunksFor` (placement) | `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks`; and the emulator's liana arm |
| S4 | the self-check biconditional deleted | `TestComposerSelfCheckSeesTheKeyPathChoice` |
| S5 | `composerArtifactsFor` calls `md.ComposeWith` again | `TestComposerComposesOnlyThroughOneSite` |
| S6 | `composerCompose` skips `ValidateUnspendableShape` | `TestComposerComposeRefusesSpecSix` |
| X1 | `expect_composer.json`: the liana arm's `numsTemplateId` deleted | `TestShotsComposerExpectFixtureCoversEveryFieldTheWalkReads` |
| I1 | the Liana arm returns `errUnderivableInternalKey` (stage 3's behaviour) | `TestInspectNamesTheLianaKindAndItsPolicyId` (the card falls to display-only) |
| **K1** | `lianaInternalKey` builds its key map from the md1's OWN keys (`md.ExpandWalletPolicyChunks(collected)`, the TLV read of the first draft) instead of the deriver's `keys` (R0 I1) | `TestTemplatePlusKeyCardsDerivesTheLianaWallet` (the kind-1 row) |
| U1 | `composerCompose` drops the `UnspendableRequestUnmet` refusal (R0 m1) | `TestComposerComposeRefusesAnUnmetLianaRequest` |
| E1 | a second `md.ComposeWithUnspendable` added inside `composerUnspendableStep`, in the file the first draft exempted whole (R0 m4) | `TestComposerComposesOnlyThroughOneSite` |
| R4 | the converse half's press-through deleted, so the step never returns (R0 m3) | `TestComposerUnspendableResetIsThePredicate` |

The table is `/scratch/code/shibboleth/mnemonic-engrave/scripts/f449-stage4-mutations.sh` (with `scripts/f449-stage4-mut.py`, which refuses a mutation that did not apply exactly once or did not compile); re-run at `d2350cb` + this plan after the R0 fold: **40 caught, S8 survives** (41 rows).

Two mutations first **survived** while this plan was being written, and the tests were changed because of them:
- `initial := 1` in front of the seeding loop was **inert** (the loop always overwrites it). The DEFAULT ROW test was rebuilt so its first-entry case starts from a ZERO-VALUE `composerState`, as `composerFlow` builds it; the real mutations (a constant seed, or md's two constants swapped) are D1-D3.
- A 20-byte assertion over every `md1Summary` line failed on shipped lines (`@0 - m/48h/…`). The bound now covers the key-path lines this plan adds, which is the claim.

**Type consistency.** `md.UnspendableKind` / `md.UnspendableNums` / `md.UnspendableLiana` / `md.ComposeWithUnspendable` / `(md.Composed).UnspendableRequestUnmet` / `(md.Composed).ValidateUnspendableShape` / `md.ErrUnspendableSortedMultiA` / `md.ErrUnspendableUseSite` / `md.ErrUnspendableNotRootTr` (Task 2); `md.KeyPathLianaUnspendable` / `keyPathOf` / `rootKeyPath` / `md.Template.KeyPath` / `md.LianaUnspendableKeyFor` (Task 3); `taprootInternalKey` / `lianaInternalKey` / `seatFixtureFor` (Task 4); `md1KeyPathLine` / `md1KeyPathUnknown` / `composerCopyLianaKeyPath` (Task 5); `composerState.unspendable` / `composerUnspendableFires` / `composerUnspendableDrop` / `composerUnspendableStep` / `composerUnspendableRows` / `composerUnspendableKinds` / `composerUnspendableDropCause` / `composerCompose` / the four `composerCopyUnspendable*`/`composerCopyLianaKeyDropped` bodies / `composerCopyLianaUnmet` (Task 6). Each name was compiled at its task boundary by the apply gate.

**Placeholder scan.** None. Follow-up IDs in Task 9 are "the next free ID at filing time" by design (stage 3 files F-654/F-655 first).

**Scope calls a reviewer may reject, each stated where it is made:**
- the `md1Summary` key-path line for ALL three kinds, not only kind 1 (Task 5: a line that appears only for one kind is not "naming the kind");
- re-greening the composer walk's three pre-existing failures (Task 7: the walk is this stage's acceptance gate, and a gate that cannot run is a hypothesis);
- the self-check biconditional (Task 6: the kind is new composer state, and §8q compares state with the decoded card).

**Deliberately absent, owned elsewhere:** `me` (stage 4a); the demo site and the emulator reaching §0b on quantoshi.xyz (stage 5); the toolkit (F-642); md-cli's F-644 warnings (next descriptor-mnemonic release); the coordinator-compat KNOWN_RELEASES gate (F-633's other half); the flash (operator).
