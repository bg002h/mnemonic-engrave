# F-449 stage 4 — implementation report (single implementer)

Plan: `design/IMPLEMENTATION_PLAN_f449_stage4_device.md` (GREEN at R1).
Fork worktree `/scratch/code/shibboleth/sh-worktrees/f449-stage4`, branch `f449-stage4` off `d2350cb`.
Engrave worktree `/scratch/code/shibboleth/me-worktrees/f449-stage4`, branch `f449-stage4-records` off `d4b730fc`.

Mutation rows are run per task on a fresh clone of the committed branch
(`scripts/f449-stage4-mut.py` reverts with `git checkout`, so never in the worktree),
and the whole table again at Task 8.

## Task 1 — baseline and re-validation

- Fork `main` = `d2350cb` (not moved), so the plan's GREEN has not expired.
- Apply gate (`scripts/plan-apply-gate-go.sh <fork worktree> <worktree plan>`): 33 blocks,
  every boundary green, `RESULT: ok -- all 1398 tests ran across 24 shards`, `ALL BOUNDARIES GREEN`.
- Baseline: md **179**, gui **1380** (all ran, 24 shards), gofmt = exactly the five-file set.

## Task 2 — md: unspendable request + SPEC §6 refusals

- Test-first: `md/compose_unspendable_test.go` failed to compile (`undefined: ComposeWithUnspendable`).
- Commit `8d21b07`. Gate: `go vet ./md/` clean; md **185** ok; gui `^TestCompose` ok.
- Mutations M1 M2 M3 M4 M5 M6 M11: all CAUGHT.

## Task 3 — md: KeyPathLianaUnspendable, Template.KeyPath, LianaUnspendableKeyFor

- Test-first: failed to compile (`undefined: KeyPathLianaUnspendable`).
- Commit `d7e1999`. Gate: `go build ./... && go vet ./md/` clean; md **188** ok; gui `TestPolicy|TestMd1|TestTemplate` ok (5 tests).
- **R1 N3 applied:** `lianaLeafPubkeys` deleted from `md/liana.go`. Its users moved onto
  `LianaUnspendableKeyFor`: `reduceLianaInternalKey` now takes the chunks; `TestLianaKeyDependsOnTheLeafKeysNotTheTree`
  counts occurrences with `collectKeyOccurrences` and derives through the helper `lianaKeyOverOwnKeys`;
  the near-miss (`TestLianaReductionRefusesANearMiss`) seats the same four keys at mirrored placeholders.
  Checked by hand-mutation: disabling the reduction's byte-equality clause fails the near-miss test.
- **Ruling:** the plan's `TestLianaUnspendableKeyForIsTheRecipeOverTheSuppliedLeaves` compared against
  `lianaLeafPubkeys`, which N3 deletes. Its reference is now the Rust primary's own rendered internal
  xpub (the vendored record's chain-0 descriptor, `parseExtendedKey`) — why: N3 forbids a second walk,
  and Rust is the stronger oracle — cost if wrong: none found; M10 is still CAUGHT by this test.
- Mutations M7 M8 M9 M10: all CAUGHT.

## Task 4 — gui: the Liana internal-key arm (§7a.2) and the refusal (§7a.3)

- Test-first, both failure modes seen: build failed (`undefined: taprootInternalKey`); with only the
  `stillUnsupported` edit, `TestEveryKeyedVectorReachesAnAddress` failed naming both kind-1 vectors "reaches NO address route".
- Commit `63da55a`. Gate: build; `go vet ./gui/` only the two ArtifactDir lines; `Address|Taproot|Underivable|DeviceDerives` ok —
  both kind-1 vectors "6 addresses via the complex route", 5/5 Liana v15.0 cases PASS;
  `TestTemplatePlusKeyCardsDerivesTheLianaWallet` 3/3 PASS (the plan's Step-4 regex does not match it; run explicitly).
- Mutations A1 A2 A3 K1: all CAUGHT.

## Task 5 — gui: print-site arms, class rulings, F-633 copy, inspect, §7a.3 engrave half

- Test-first: failed to compile (`undefined: composerCopyLianaKeyPath`, `md1KeyPathLine`).
- Commit `a054ead`. Gate: build; gui vet baseline only; `TestComposerCopy|KeyPath|Liana|Inspect|Unnamed|Fable|TestEmulatorWalks|Modal` ok
  (53 top-level PASS; each of the four new tests PASS by name); gofmt adds nothing.
- Mutations C1 C2 S1 S2 S3 S7 I1: CAUGHT. S8: SURVIVED — the plan's one named survivor (re-checked at Task 8).

## Task 6 — gui: the §0b key-path choice screen

- Test-first: failed to compile (`undefined: composerCopyUnspendableLead`, …). New tests use no
  `click(&ctx.Router`, `ButtonEvent`, `Up` or `Down` (grep: none) — touch only.
- Commit `d64695c`. Gate: build; gui vet baseline only; gofmt adds nothing;
  `TestComposer|Liana|Unspendable|TestEmulatorWalks|Modal` ok; **whole gui package: all 1398 tests ran across 24 shards, RESULT ok** (1380 + 18, as the plan says).
- **R1 N2 applied:** the `unbuilt` comment now puts the modal first and `composerTemplateChunksFor`'s
  refusal second — measured: `composerUnspendableStep` at `gui/composer_flow.go:101`, `composerTemplateChunksFor` at `:104`.
- **R1 N1:** the "4 in production" phrase exists only in the plan's prose (grep of all 33 blocks: no code comment carries it), so no code changed.
- Mutations C1 C2 (now also caught by the predicate test) P1 P2 P3 D1 D2 D3 R1 R2 R3 R4 F1 L1 S4 S5 S6 U1 E1: all CAUGHT.

## Task 7 — the emulator walk (acceptance gate)

- md-cli 0.19.0 built from descriptor-mnemonic `main` `d269c556` into a scratch target
  (`/scratch/code/shibboleth/.tmp/f449s4-dm-target/release/md`, `md 0.19.0`), passed as `$MD` — the main checkout's `target/` was not touched.
- Test-first: `go test ./cmd/emu/` failed ("the liana arm is missing …") before the fixture was regenerated;
  `TestEmulatorWalksQuoteCopyThatStillExists` failed on the new JS until the `notComposerCopy` entry was added.
- Host transcript (`FORK=<worktree> MD=<0.19.0>`): exit 0, every `GATE PASS`, including `liana md1 string`,
  `liana Template-ID`, `NUMS twin Template-ID`, `md is 0.19.0`. No `GATE FAIL`.
- `expect_composer.json` regenerated by `--emit-expect`; `--check-expect` ok; its +/- lines equal the plan's block exactly.
- **Walk, `capture_composer.py --arm both`: exit 0, "all legs matched the host."** keyed-A 7 strings (60 s, 23 shots),
  keyed-B 9 (73 s, 24 shots), keyless 1 `md1fkzyy…` (28 s, 10 shots), **liana 1 `md13ls8a…` (28 s, 11 shots), Template-ID f99cc42e…**.
- Negative control `--arm keyed --prove-it-can-fail`: **NEGATIVE CONTROL PASSED** (refused the corrupted address).
- Placement mutation L1 applied to a clone, `--arm liana`: **FAILED** as required — "the screen does not carry
  Template-ID: f99cc42e…"; the screen showed the NUMS twin's `81072164…`.
- Commits: fork `a66491f`; engrave `62ad06ba` (journey scripts + re-run `transcript_composer.txt`).

## Task 8 — whole gate, size, mutation pass

- Whole gate (fork HEAD `a66491f`), captured once: gofmt = exactly the five-file baseline; `go vet ./md/` clean,
  gui vet only the two ArtifactDir lines; non-gui packages **55 ok**, nothing else; gui **all 1398 tests ran across 24 shards, RESULT ok**; md **188**.
- TinyGo size (CLAUDE.md recipe): base `d2350cb` **1,658,284 B flash / 63,352 B RAM**; end **1,665,828 / 63,376**
  (**+7,544 / +24**) — identical to the plan's measurement.
- Mutation pass, whole table (`scripts/f449-stage4-mutations.sh`, MUTDIR = a fresh clone of the branch): **40 CAUGHT, S8 SURVIVED** (41 rows),
  exactly the plan's result; S8 is the named gap (§7a.3's engrave half is unreachable: no decoder yields a fourth kind).
  Output: `/scratch/code/shibboleth/.tmp/f449s4-impl/mut-full.txt`.
- Pixels (the plan's "no gate covers" row): re-ran `--arm liana` on the real tree (all legs matched) and looked at
  `l01-key-path.png` (lead + two rows, NUMS highlighted by default, no clipping) and `l04-consent-p1/p2`
  (`KEY PATH: NONE (LIANA KEY)` body, Template-ID f99cc42e…, `Template has no keys - no addresses.`). Nothing clipped.
- **Step 4 (merge to fork main + push) NOT done** — excluded by the implementer brief (no merge into main, no push, no flash).

## Task 9 — evidence, follow-ups, spec reconciled (engrave branch `f449-stage4-records`)

- `scripts/liana-live-gate.sh`: **PASS — 11 verdicts match; Liana v15.0 at 4684d5cb**; probe rule holds for 10 probes.
- Evidence `1ca29d16` (`design/evidence/f449-stage4/`, each leg a script that regenerates its files):
  - `liana-probes.sh`: the four F-644 shapes and §7's single-timelocked-leaf **refused** ("Descriptor is not compatible with a
    Liana spending policy."), nested-two-recoveries and pk-primary-leaf + recovery **accepted**; internal keys recomputed per probe
    (only probe 1 equals the input key, as the plan predicted). Re-run: byte-identical.
  - `core-kofn-import.sh`: Bitcoin Core **v30.99.0-64a7c7cbb975** (a dev build) imports the kind-1 kofn descriptor
    (`[{"success":true}]`); getnewaddress 0..2 == Liana's recorded receive (`CORE == LIANA`). Daemon stopped after.
- Follow-ups `1aabd8a1` (`scripts/followups-status.sh`: OK): F-654 CLOSED (cites the fork branch commits `8d21b07`, `d64695c`;
  `validate_minimal_wire_version` unported by design); F-644 device half closed, md-cli half re-owned to the next dm release;
  F-633 copy half done; new **F-659** (restore doc "listed below" with no cards, ownerless) and **F-660** (composer walk runs on nothing, composer cycle).
- Spec sweep `deebbd0a`: SPEC_liana §7 (NumsXpub → LianaUnspendable, reason kept as history), §0b PLACEMENT re-resolved
  (`:96`/`:101`/`:104`/`:135` at `a66491f`), §7a Status, §8.9's four §0b rows name their tests, §9 stage-4 Status.
  SPEC_wallet_policy_composer §8f/§8x amended and §8y added; **machine-checked: all 8 bodies re-normalised from the spec's quote
  blocks equal `composerCopyTable`'s strings.**

### Rulings (Task 9)

- **Ruling: F-654 closed against the BRANCH commits, not a merge SHA** — why: the plan says "closed in the fork merge SHA", but my
  brief forbids the merge; the follow-up gate needs a cited commit — cost if wrong: one Status line to re-point at landing.
  Likewise the spec's Status lines say "fork branch `f449-stage4` at `a66491f`, not yet merged".
- **Ruling: F14's evidence is the code, not the `l06-` frame** — why: `l06-bundle-engraved.png` is the bundle modal; the walk
  confirms through the Restore Doc without a shot. The defect is plain in `composerRestoreDoc`'s `len(keyed) == 0` branch
  (`gui/composer_flow.go:534-543`), and the walk's census says 1 plate — cost if wrong: none (the entry says where its fact comes from).
- **Ruling: §8f's "272 chars, headroom 277" replaced by measured lengths** — why: it was already false before this stage
  (the body stage 4 found is 328 bytes; the amended one 360); "headroom" was not re-derived, and the note says so — cost if wrong: none.
- Not new: the watch-only "hand-engrave your ms1 share(s)" modal seen in `l06-` is F-463 (already filed; the walk's own comment says so).

## Final state

- Fork `f449-stage4`: `8d21b07` `d7e1999` `63da55a` `a054ead` `d64695c` `a66491f` (HEAD), all ssh-signed, off `d2350cb`. Not merged, not pushed.
- Engrave `f449-stage4-records`: `62ad06ba` `1ca29d16` `1aabd8a1` `deebbd0a` (HEAD), off `d4b730fc`. Not merged, not pushed.
- Not done, by the brief: Task 8 Step 4 (merge to fork main, push-via-staging), any flash or signed-image pre-build.
- This report itself lives at the main-checkout path the brief named and is **uncommitted** there.

### Concerns for the controller

1. At landing, re-point the "branch … `a66491f`, not yet merged" wording (SPEC_liana §0b, §7a, §9) and F-654's Status at the fork merge SHA.
2. Evidence F7 is from a Bitcoin Core **development** build, as the plan already records; no release measured.
3. S8 (the §7a.3 engrave-half call) remains untestable until a fourth kind exists — the plan's named gap, unchanged.
4. The walk is still not run by anything automatically (F-660); it was run by hand here, green.
