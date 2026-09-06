# H6 plan author report — `IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`

**Author:** opus, plan-author role, brief
`design/agent-briefs/hashlock-H6-plan-author-brief.md`.
**Spec:** `design/SPEC_hashlock_H6_preimage_plates.md`, R0 GREEN at engrave
`a67a3924`.
**Baselines:** fork `fb0dd04`, engrave `75f00b56`, ms `504ff46`.
**Plan:** 3,255 lines, 13 tasks. **Checker:
`scripts/h6-plan-blocks-vs-tree.sh` — 70 blocks checked, 0 FAIL.**
**Nothing committed.** Only two files are added to the working tree: the plan
and the checker.

---

## 1. What was WIRED AND RUN, and what was SPECIFIED ONLY

This is stated first because it is the report's most important content.

**WIRED, BUILT, TESTED and MUTATED** — Tasks **1, 2, 3, 4, 5, 6, 7, 8a**. Every
`file=`-headed block in them is byte-for-byte the text that compiled and ran in
a scratch tree, and every `MUTATION:` written in them was executed and its
failure quoted.

**SPECIFIED, NOT WIRED** — Tasks **8b, 9, 10, 11, 12** (and Task 13, which is
records). These are the fork's `gui` surface: two new screens, a new flow, a
changed `composerCensusLines` signature, five new copy bodies and a walk arm.
Their blocks deliberately carry **no `file=` header**, so the checker cannot
report a coverage it does not have, and the plan's `## Build gate` says so
before it reports any result.

**Every measured number quoted inside those five tasks is the R0-GREEN spec's
own measurement, carried across — not mine.** I measured nothing on those
screens: no `assertModalBodyFits` run, no pixel width, no picker row. An
implementer must treat them as a specification to build and gate, and the first
thing each owes is its own RED.

**Why the split.** H6 spans three repositories and roughly ten deliverables per
repo; wiring the whole `gui` half would have been implementing the stage rather
than planning it. I spent the budget on the parts where a wrong plan is
expensive and a machine can settle it: the QR raise (four 32-minute fuzzing
campaigns), the plate geometry, the encoder, the classes, the host admission and
its warnings. That is where all nine findings in §4 came from.

---

## 2. Per-task RED / GREEN / mutation tails

### Task 1 — mnemonic-secret: the phrase rule and `qr_text` in `ms-codec`

GREEN: `cargo nextest run --locked` → **562 tests run: 562 passed, 11 skipped.**
New: `crates/ms-codec/tests/hashlock_qr_text.rs`, 3 tests, all pass.
Seven corpus rows added, byte counts MEASURED:

```
anchor-hardened 122   anchor-sha256 63   max-phrase-hardened 194   max-phrase-sha256 135
phrase-with-colon 119   phrase-trailing-space 102   phrase-with-comma 109
```

194 bytes at the 100-character cap is §7.1's worst case, and it encodes at
ECC-L to **53 modules, v9** — the whole reason §7 exists.

### Task 2 — engrave Rust: the `phrase:` record and the two classes

Corpus regenerated from `CASES`:

```
wrote 68 rows to .../crates/me-cli/testdata/record_class_vectors.json
sha256 3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf
```

47 → 68 rows (21 `phrase:` rows). GREEN:
`cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` →
**621 run, 618 passed, 3 failed** (baseline `history_purge`), 2 skipped.

### Task 3 — `--pack-preimage`, the refusals and the four warnings

New: `crates/me-cli/tests/sysw_pack_preimage.rs`, **9 tests, 9 passed.**
Full suite after: **630 run, 627 passed, 3 failed** (the same three), 2 skipped.

**Five mutations, executed, tails verbatim:**

| mutation | failure |
| --- | --- |
| drop `Preimage`/`Phrase` from `Class::is_secret` | `the_warnings_print_in_the_f246_order` … `the passphrase ceremony did not run` — **NOT SEALED: the payload ships bearer material in cleartext. This is the funds-relevant row and it is why §3.2 gates admission, not classification.** |
| drop `admit_check`'s second rule | `no_flag_refuses_both_carriers_by_index` … `carrier 0 packed without the flag` |
| drop the phrase arm from `preimage_digest_of` | `a_space_after_the_comma_derives_a_different_preimage_and_warns` … `the space-after-the-comma row did not warn` |
| make the no-`hash:` case a `WARNING` | `no_hash_record_at_all_is_a_note_not_a_warning` … `the incomplete case did not draw the NOTE` |
| drop the id test from `preimage_plate_admissible` | `the_three_ids_each_get_their_own_refusal` … `not §8.1.2: me: record 0 … is a hashlock PREIMAGE plate (kind 0x03) … Re-run with --pack-preimage` |

### Task 4 — the QR raise to v9 and the scale-2 arm

**§7.1 and §7.2 reproduce EXACTLY.** The alignment centres, derived from the
encoder's own bitmap by testing the 5×5 ring shape at every candidate centre:

```
v2 (18,18)  v3 (22,22)  v4 (26,26)  v5 (30,30)  v6 (34,34)
v7 (22,6) (6,22) (22,22) (38,22) (22,38) (38,38)
v8 (24,6) (6,24) (24,24) (42,24) (24,42) (42,42)
v9 (26,6) (6,26) (26,26) (46,26) (26,46) (46,46)
```

ECC-L thresholds: ≥79→37, ≥107→41, ≥135→45, ≥155→49, **≥193→53**, ≥231→57.

**THE FUZZING CAMPAIGN — four runs, 32 min each on 24 cores, §8.6-shaped
payloads only, 128 minutes of wall clock:**

| dim | v | samples | observed max | max/dim² | last improvement | quiet | `findPath` errors | buffer | ENTRY |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 41 | 6 | 14,494,291 | **823** | 0.4896 | sample 7,013,580 | 16.4 min | 0 | +20 | **843** |
| 45 | 7 | 11,065,492 | **960** | 0.4741 | sample 10,124,362 | **2.8 min** | 0 | **+53** | **1013** |
| 49 | 8 | 10,138,994 | **1179** | 0.4910 | sample 8,820,790 | 4.1 min | 0 | +20 | **1199** |
| 53 | 9 | 7,759,282 | **1379** | 0.4909 | sample 2,213,470 | 22.6 min | 0 | +20 | **1399** |

**43,458,059 payloads in total, ZERO `findPath` failures at any raised
version.** Every dimension CLEARS the spec's stated floor (813 / 945 / 1161 /
1369) by +10 / +15 / +18 / +10, which is the checkable gate §7.2 item 4 asks for.

**The spec's prediction that "v7 is the suspect entry" was CORRECT and is
visible in two independent ways**: its ratio is 0.4741 against a 0.4905 mean of
the other three, and its last improvement landed at sample 10.1M of 11.1M —
**2.8 minutes of quiet against 16.4, 4.1 and 22.6.** v7 is the one entry that
had not converged when its 32 minutes ran out. Its buffer is widened to 53, to
the trend-implied ~993 plus 20, and the code comment says so.

**Scale 2, MEASURED at the production stroke (1920), with scale 3 as control:**

```
scale=2 p={0 0} cmds=5 ink x=[0,3840] want [0,3840] | y=[0,3840] want [0,3840]
scale=2 p={1 0} cmds=5 ink x=[3840,7680] want [3840,7680]
scale=2 p={5 7} cmds=5 ink x=[19200,23040] | y=[26880,30720]
scale=3 p={5 7} cmds=5 ink x=[28800,34560] | y=[40320,46080]
```

3,840 units = **0.6 mm**, the cell exactly, five commands per module — the same
constant as `case 3`. This reproduces the spec's §7.3 measurement byte for byte.

**Mutations, executed:**

| mutation | failure |
| --- | --- |
| change a v9 centre (46,26)→(46,28) | `dim 53 marker 3: centre {46 28}, the encoder draws {46 26}` |
| drop 41 from the single-marker case | `panic: unsupported qr code version` |
| remove the `case 2` arm | `panic: unsupported module scale` |
| symmetric scale-2 arm | `scale 2 p={0 0} axis x: ink [960,4800], want [0,3840]` |
| content-dependent move list | `dim 41: two payloads emitted 13151 and 13241 commands; the toolpath is content-dependent` |
| revert the bound to 37 | `ConstantQR(dim 41) = engrave: constant QR size too large: 41, want accepted` |
| `return 1379 + 20` → `1378` | `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer)…` |

Goldens: `h6-qr-v{6,7,8,9}-scale2.bin`, 2034 / 2649 / 2963 / 3295 bytes,
generated AFTER the budgets (the plan says why: `Engrave` loops `for range
nmod`, so the goldens depend on the entry).

### Task 5 — `EncodeMS1Preimage` and `IsPreimagePlate`

GREEN: `go test ./codex32/` — ok. Four tests.
**The copy-paste hazard, reproduced:**

```
EncodeMS1Preimage = ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q,
             want the corpus ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
```

and `IsPreimagePlate(ms10entrsq…) = true: a kind-0x03 payload under id "entr"
was admitted` when the id test is dropped. Dropping the `0x03` prefix byte gives
a 73-character string.

### Task 6 — the plate

**§6.5 REPRODUCES EXACTLY**, every cell of both tables:

```
rung | chars/line(79mm) | lines/plate | advance mm
6.0 | 19 | 13 | 4.0000     5.0 | 23 | 15 | 3.3333     4.4 | 26 | 17 | 2.9333
3.8 | 31 | 20 | 2.5333     3.4 | 34 | 23 | 2.2666     3.0 | 39 | 26 | 2.0000
budget = 416000 units = 65.00 mm ; QR env scale2 = 31.80 mm ; scale3 = 47.70 mm
method line = 73 chars
phrase+QR(scale2) 3.4mm total=71.20mm fits=false | 3.0mm total=63.80mm fits=true
phrase, no QR     5.0mm total=80.00mm fits=false | 4.4mm total=57.20mm fits=true
string, no QR     6.0mm total=60.00mm fits=true
```

including the corrected 5.0 mm advance (3.3333, not the withdrawn 3.435) and the
1.20 mm of spare at the single fitting rung. Nine tests; goldens
`hashlock-{string-6mm,phrase-noqr,phrase-qr-v9,phrase-space-legend}.bin`
(4348 / 4834 / 7678 / 3825 bytes).

**Mutations:** a locator row in a band → `the top band inks 34133 units tall,
over one 3.0mm line`; QR scale 3 → `3.0mm: fits = false (10 rows, 510080 units
against 416000)`; word wrap → `3.4mm: fits = true (9 rows…), want false`;
method line at 79 characters → `11 rows at 3.0mm need 427520 units against a
budget of 416000`.

### Task 7 — the classes, the port and admission

GREEN: `go test ./sysw/ ./hashlock/` — ok.
**The cross-language result this task exists for: all 68 corpus rows, including
the 21 new `phrase:` rows, classify identically on the host and on the device.**
The fork's vendored ms corpus re-pinned to
`4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`.

### Task 8a — retention

GREEN on its three tests. **Mutations:** assign without the nil check →
`panic: assignment to entry in nil map`; remove the scrub from
`composerFlowExit` → `the phrase survived the flow-exit defer: "correct horse
battery staple"` and `the preimage survived the flow-exit defer: abcdef00…`;
wipe without writing the value back → the preimage assertion fails while the
phrase one passes, which is why both exist.

---

## 3. Whole gates

| gate | result |
| --- | --- |
| `gofmt -l` over every touched package | clean apart from the pristine trio (`gui/transaction*.go`) |
| `go build ./...` (fork) | ok |
| `go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/` | **all ok** |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **exit 0** |
| `./cmd/emu/build.sh` | exit 0, `built emu.wasm (10882452 bytes)` |
| `scripts/gui-shard-test.sh ./gui/ 24` | 1242 tests, partition exhaustive, **23/24 ok, shard 18 FAIL** (see below) — run twice, same single failure |
| ms `cargo nextest run --locked` | 562 run, 562 passed, 11 skipped |
| me `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` | 630 run, 627 passed, **3 failed** (baseline), 2 skipped |
| `scripts/h6-plan-blocks-vs-tree.sh` | **70 blocks, 0 FAIL**; 19 unheadered blocks named |
| firmware, `fb0dd04` | 1,599,208 B flash / 62,856 B ram |
| firmware, wired tree | **1,600,944 B / 63,248 B — +1,736 B flash, +392 B ram** |

**The three baseline reds, each measured on the pristine checkouts:**
1. `gofmt -l` names `gui/transaction.go`, `gui/transaction_golden_test.go`,
   `gui/transaction_txrecord_test.go` at `fb0dd04`. H6 adds no fourth.
2. `go vet ./engrave/` **exits 1** at `fb0dd04` with two
   `testing.ArtifactDir requires go1.26 or later (file is go1.25)` diagnostics
   (`go.mod` says 1.25.10; the toolchain is 1.26.7; the project floor is 1.26).
   **H6 grows the count 2 → 6**: my two golden tests call `t.ArtifactDir()`
   exactly as their four shipped siblings do. Same diagnostic, no new class; the
   `go.mod` bump is filed as a follow-up rather than slipped into this stage.
3. The three `history_purge` failures are `/usr/bin/zsh` missing on this box, and
   the test says so itself: *"there is no way to run it without zsh. This is
   deliberately a FAILURE and not a skip."*

**The one non-baseline red, and it is a FINDING rather than a defect.** Shard 18:

```
--- FAIL: TestComposerEveryScreenFunctionHasAProductionCaller (0.03s)
    composer_join_test.go:104: these composer functions have no production caller, so the screens they
        draw cannot be reached by any operator: [composerHoldHashlockMaterial]
```

That guard is the one the composer cycle added after two R0 lenses found
fourteen production functions at once with a green suite. It is correct here:
Task 8b calls `composerHoldHashlockMaterial` and Task 8b is not wired. **The
plan's consequence — Tasks 8a and 8b land in ONE commit, and the exemption table
is not the instrument — is recorded on Task 8a**, because quieting that gate is
exactly the failure it exists to catch.

---

## 4. What I could not implement as the spec wrote it — nine findings, each measured

None is a Critical against the spec's design. Six would have been got wrong by an
implementer following it literally.

1. **`me` cannot call the phrase rule §3.1 names.** §3.1 requires a `phrase:`
   body to pass *"`ms-cli`'s `validate_phrase` … byte for byte"*. Measured:
   `validate_phrase` and `looks_like_ms1` are `ms-cli`'s, and
   `crates/me-cli/Cargo.toml:53` depends on `ms-codec = "0.8"` and on nothing of
   `ms-cli`. A literal reading produces a **third** copy of a rule whose whole
   point is that no two readers of a phrase disagree. **Task 1 moves it into
   `ms-codec` and makes the stage's first deliverable a published release**, and
   Task 2 is blocked on it. I stood a `[patch.crates-io]` in the me workspace to
   build the rest; the plan flags it as a scratch device that must not be
   committed.
2. **§11.4's method-line mutation cannot fail.** *"Add one character to the
   method line → the worst case no longer fits."* Measured at 74, 75, 76, 77 and
   78 characters: **all five still fit**, because 39 characters per line wraps a
   73-character line to 2 rows anywhere below 79. §6.5 consequence 3's own
   number is the threshold, and at 79 it reds with `11 rows at 3.0mm need 427520
   units against a budget of 416000`. The plan carries the mutation that fires.
3. **§11.3's budget mutation cannot fire from any in-suite sample.** With the
   arms at the campaign maximum minus one (822 / 959 / 1178 / 1378), the fuzzed
   row **still passes** — its 400 payloads per dimension observe 792 / 910 /
   1143 / 1322, and the true maxima took 7 to 14 million payloads. A suite cannot
   spend 32 minutes a dimension. Task 4 adds
   `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes`, a PIN of the four entries
   with their campaign provenance, which is the row that actually carries
   "derived rather than guessed".
4. **`preimage_plate_admissible` needs TWO more conjuncts than §4.3 gives it**,
   and the SHIPPED test `a_preimage_plate_is_named_not_misdiagnosed` found both:
   a kind-`0x03` single under the id `hash` whose X is 16 bytes (the codec's
   `PreimageLengthMismatch`, 50 characters), and the UPPERCASE spelling of a
   plate. The device's `codex32.IsPreimage` already required `len(d) == 33 &&
   d[0] == 0x03`, and `Split()`'s id comparison is case-sensitive — **so Go was
   right and the Rust needed narrowing**, which is the Rust-primary rule's
   "a defect in a Go port triggers a Rust check" running in reverse.
5. **Making the classes BEARER moves the argv surface**, and the spec says
   nothing about it. `me sysw pack --pack-preimage --no-passphrase <ms1>` — §12
   item 3's own acceptance invocation — is refused by `argv_secret_guard` before
   the parser runs. The refusal is CORRECT; its wording was not, because it said
   *"BEARER material -- a signed transaction, or the mt1 set carrying one"* for a
   hashlock preimage. Task 3 gives the guard an arm; Task 13 fixes the acceptance
   path to `--in`.
6. **The raise falsifies two shipped tests §14 does not list.**
   `TestConstantQRLargeVersionsFailClosed` (`engrave/engrave_test.go:544-568`)
   asserts dim 41 is REFUSED; `TestPassphraseQRTooLong`
   (`backup/passphrase_test.go`) uses a 200-character passphrase — dim 53 — as
   its out-of-reach case. Both properties survive; both move to v10. A third
   record, `TestPassphraseQRFitsSupportedVersion`'s *"dim 41 — which is
   deliberately unsupported"*, is rewritten with them.
7. **`composerHoldHashlockMaterial` trips the fork's own join guard**, so §2.2's
   retention cannot land as a commit of its own (§3 above).
8. **§6.2/§6.3 and §6.5 disagree about body-row ORDER** — method line first
   versus locator first. The normative sections win. The row count is identical
   either way (3 + 1 + 2 + 1 + 3 = 10 at 3.0 mm), so **no measurement in §6.5
   moves**; recorded so a reader does not re-open it.
9. **The `preimage-plate-0x03` seam row moves class**, `Unknown` → `Preimage`, in
   a corpus whose test is named *"not one of these records may change class"*.
   Its `entr`-id sibling stays `Unknown`, and that PAIR is §4.3's id narrowing
   stated as data. Recorded in the test's own doc comment rather than absorbed.

**Two spec items I did not attempt, and neither is in scope for a plan author:**
§12 item 8's physical QR scan (a test plate and a phone; the SH2 has no camera),
and §11.7's emulator walk, which the brief puts out of scope.

---

## 5. Scratch trees, left in place

| tree | repo | baseline | state |
| --- | --- | --- | --- |
| `/scratch/code/shibboleth/.tmp/h6-gate` | seedhammer fork | `fb0dd04` | Tasks 4-8a wired, gated |
| `/scratch/code/shibboleth/.tmp/h6-ms` | mnemonic-secret | `504ff46` | Task 1 wired, gated |
| `/scratch/code/shibboleth/.tmp/h6-me` | mnemonic-engrave | `75f00b56` | Tasks 2-3 wired, gated |

Cargo target dirs: `.tmp/h6-ms-target`, `.tmp/h6-me-target`. Campaign log:
`.tmp/h6-campaign.log`. The campaign harness is
`h6-gate/engrave/h6_campaign_test.go`, behind the `h6campaign` build tag, so it
does not run in the ordinary suite; re-run it with
`H6_SECONDS=1920 go test -v -tags h6campaign -run TestH6BudgetCampaign -timeout 24h ./engrave/`.

**No phrase or preimage bytes were written to any log.** The campaign records
counts and maxima only; the fuzzed phrases are generated in-process and never
printed. No `.jsonl` was read. Nothing was committed.
