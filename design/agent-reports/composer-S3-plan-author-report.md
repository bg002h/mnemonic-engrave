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
