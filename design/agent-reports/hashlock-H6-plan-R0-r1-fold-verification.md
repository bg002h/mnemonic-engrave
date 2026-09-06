# H6 plan — R0 round 1, independent fold verification (sonnet)

**Reviewer:** sonnet, fold-verification role, brief
`design/agent-briefs/hashlock-H6-plan-R0-r1-fold-verification-brief.md`.
**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` at engrave
`7021ea43` (fold) over `e6d84d9c` (round-0 baseline); companion spec fold
`95418b2c` (`git log e6d84d9c..7021ea43` confirms both commits, in that order).
**Against:** the four round-0 artifacts — `hashlock-H6-spec-plan-round-verification.md`
(GREEN, prior-stage context), `hashlock-H6-plan-R0-r0-fidelity.md` (0C/8I/4M/2N),
`-tests.md` (0C/0I/2M), `-journey.md` (1C/5I/4M/2N) — and the fold author's own
`hashlock-H6-plan-R0-r0-fold-report.md`.

**Method.** Read-only on every repo; no sub-agents; no `.jsonl` read; nothing
committed. Own `cp -a` copies at `/scratch/code/shibboleth/.tmp/h6-r1-{ms,me,gate}`
(never modified after the copy — confirmed with `diff -rq` against the originals
at the end, identical except `h6-r1-me/Cargo.toml`'s deliberate `[patch.crates-io]`
re-point to `../h6-r1-ms` so builds never touch the gated tree, the same
discipline the round-0 tests lens used). Six further disposable mutable copies
(`h6-r1-gate-mut{,2..6}`) for the mutation runs below, each `diff`-verified
identical to the pristine copy after revert, then deleted. Go
`/scratch/code/shibboleth/.toolchain/go/bin/go` (1.26.7); Cargo
`PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp
CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h6-r1-target-{ms,me}` (removed
after use).

**Result: every Critical and Important the fold claims is genuinely fixed or
truly declined. The checker PASSES independently (97/0). All full-suite counts
reproduce exactly. All ten mutations I ran reproduce the fold's quoted output
exactly. `cargo fmt --check` is clean in both Rust trees. The merged group's
files are mechanically confirmed disjoint from every group that runs in
parallel with it. Two new Minor findings below (a citation-mechanism
imprecision and one unfolded half-sentence of a suggestion); neither blocks.**

---

## 1. The Critical — journey C-1, the phrase-form locator

**Diff** (`git diff e6d84d9c..7021ea43`): three changes, exactly as the fold
report states — `composerHashlockPlateBuiltHook` (a nil-in-production seam),
a phrase row added to `TestHashlockPlatesLocatorAlwaysCarriesTheHashRow`, and
new `TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest` driving the flow
end to end.

**RE-RUN, mutation 1 — omit the locator's `hash` row for a phrase record**
(`gui/composer_hashlock_plates.go`, `hashlockPlatesLocator`, my own copy):
```
--- FAIL: TestHashlockPlatesLocatorAlwaysCarriesTheHashRow/a_phrase_record,_derived
    locator = [], want a "hash  e7e68d52..476e1407" row: a phrase-form plate with no
    locator at all is the worst artifact this stage can cut
--- FAIL: TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest
    the plate's locator = [], want a "hash  f22cc3f5..81f6e837" row.
```
Reverted; `diff -rq` against the pristine copy empty; unmutated run `ok`.

**RE-RUN, mutation 2 — build the locator one statement before
`hashlockPlatesDerive`** (moved the `hashlockPlatesLocator` call ahead of the
derive, passed the pre-derive value to the plate builder):
```
--- FAIL: TestHashlockPlatesFlowLocatorCarriesTheDerivedDigest
    the plate's locator = ["hash  00000000..00000000"], want a "hash  f22cc3f5..81f6e837" row
```
(the unit-level row still PASSES here, exactly as expected — it calls the
function directly on an already-derived record, so only the flow-level test
can see this ordering bug, which is precisely why the fold added the second
test.) Reverted; `diff -rq` empty; unmutated run `ok`.

**Verdict: FIXED.** Both mutations the fold names now RED, on two independent
assertions, and both revert clean.

---

## 2. The Importants — table

| # | finding | change | verdict |
| --- | --- | --- | --- |
| fid I-1 | 3 `gui` groups falsely declared disjoint; F's real predecessor understated | group table rewritten: E = 8a→8b→9→10→11 one implementer, after Tasks 6/7; shared files added to every task's Files list | **FIXED, mechanically confirmed** — see §4 |
| fid I-2 | Task 6's gate can't pass in the declared order | ms-corpus re-vendor moved to new Task 5b, group C, before Task 6 | **FIXED, RE-RUN both directions** — see §3 |
| fid I-3 | `hashlock/hashlock.go`'s `MethodLine`/`QRText` scheduled by no task | Task 5b owns them; File Structure rows added | **FIXED** — files exist, Task 5b's Files list names them |
| fid I-4 | pinned to a literal, not the corpus | `methodline_h6_test.go` reads the vendored `qr_text` rows | **FIXED, RE-RUN re-vendor simulation** — see §5 |
| fid I-5 | budget argument skipped dims §8.6 content also reaches (v3-v5) | dim list extended to `{29,33,37,41,45,49,53}` | **FIXED, RE-MEASURED** — see §6 |
| fid I-6 = jrn M-1 | §8.2.4 said "above", passphrase is below | → "this payload's passphrase" | **FIXED, RE-RUN restore-mutation** — see §7 |
| fid I-7 = jrn I-5 | Back contract: plan/spec said one thing, tree does another | both now say Button1 declines; backing out of the STEP not offered; census's Button1 is the exit | **FIXED** — text now matches `gui/composer_preimage_plate.go:140-144`'s own comment (read directly) |
| fid I-8 | release step (Task 1 Step 6) had no gate/checklist | 9-item `RELEASE_PROCESS.md` checklist with commands | **FIXED, item numbers RE-VERIFIED against the live `mnemonic-secret/design/RELEASE_PROCESS.md`** — see §8 |
| jrn I-1 (+M-3) | `phrase:` undocumented, refusal names wrong kinds | Task 3 Step 8b: `Pack` doc comment, `U::Composer`, `U::Unrecognised` all gain the fifth prefix | **FIXED, RE-RUN live** — see §9 (partial residue noted in §11) |
| jrn I-2 | HOLD confirm modal still says phrase "not on this device" | rewritten, §0 gains 5th falsified record | **FIXED, RE-MEASURED 342/107**; declined alternative RE-MEASURED refused — see §10 |
| jrn I-3 = fid M-4 | §8.2.2 fires above a refusal naming the same record | new `hashlock_carrier_shaped` diagnostic silences it | **FIXED, RE-RUN restore-mutation** — see §9 |
| jrn I-4 | QR gate cost model wrong ~900× | replaced by measurement (43m31s / 32m12s / etc.) | **FIXED, RE-MEASURED + RE-RUN 2 mutations** — see §6 |

All 13 Importants named across the two lenses are accounted for in this table
(8 fidelity + 5 journey); the "=" rows are the fold's own correct cross-lens
identification of the same defect, not a miscount.

---

## 3. Task 5b boundary, both directions (fid I-2)

**Declared order (my pristine `h6-r1-gate` copy, unmutated):**
```
$ go test ./hashlock/   -> ok  seedhammer.com/hashlock  0.276s
$ go test ./backup/     -> ok  seedhammer.com/backup    2.997s
```

**Baseline order (mutable copy, `hashlock/` replaced wholesale with fork
`fb0dd04`'s tree via `git archive fb0dd04 hashlock | tar -x`):**
```
$ python3 -c "...json.load(open('hashlock/testdata/hashlock-v0.8.json'))..."
qr_text present: False, rows: 0
$ go test ./hashlock/   -> ok  seedhammer.com/hashlock  0.230s
$ go test ./backup/     -> FAIL
    --- FAIL: TestHashlockQRTextMatchesTheMSCorpus (0.00s)
        hashlock_test.go:464: the corpus carries 0 qr_text rows; H6 §11.2 pins seven
```
Exact match to the fidelity report's and the plan's quoted failure.

---

## 4. Group disjointness — mechanically extracted, not eyeballed

Parsed every task's `**Files:**` block out of the folded plan with a script
(no manual transcription) and unioned per group:

```
Within group C (4, 5, 5b — run in parallel before Task 6 joins):
  4 vs 5   -> disjoint
  4 vs 5b  -> disjoint
  5 vs 5b  -> disjoint

Truly-parallel top-level groups (B={2,3}, C={4,5,5b,6}, D={7}):
  B vs C -> disjoint
  B vs D -> disjoint
  C vs D -> disjoint

Merged group E (8a,8b,9,10,11 — declared ONE sequential implementer)
  vs B -> disjoint
  vs C -> disjoint
  vs D -> disjoint
```

E's own 26-file union (`gui/composer_copy.go`, `gui/composer_copy_test.go`,
`gui/modal_fits_test.go` included) is shared **internally** across 8a/9/10/11,
which is exactly the fix (one implementer, not three) — not a violation.
`gui/sysw_admit.go` (Task 7 / group D) appears in no other task's list,
confirming the plan's own "which no other task touches" claim.

---

## 5. MethodLine/QRText pinned to the vendored corpus, not a literal (fid I-4)

**Simulated re-vendor** on a mutable copy: reordered `salt=`/`iterations=` in
one corpus row's `qr_text` string (same 73 characters), recomputed and
re-pinned `corpusSHA256` to the mutated file's real SHA-256 (a genuine
re-vendor updates the pin too) — production `MethodLine`/`QRText` **untouched**:
```
$ go test -run TestH6MethodLineAndQRTextMatchTheMSCorpus -v ./hashlock/
    row anchor-hardened: QRText = "...iterations=100000 salt=ms-hashlock-v1...",
      want the corpus row "...salt=ms-hashlock-v1 iterations=100000..."
    row anchor-hardened: MethodLine(true) = "...iterations=100000 salt=ms-hashlock-v1...",
      want the corpus line "...salt=ms-hashlock-v1 iterations=100000..."
--- FAIL
```
This is the property a self-referential/literal-pinned test cannot have: the
corpus alone changed, production code did not, and the suite still reddened.
Reverted (corpus file + `corpusSHA256`); `go test ./hashlock/` back to `ok`.

---

## 6. Numbers re-measured

**QR budget headroom** (`go test -run TestConstantTimeQRBudgetBoundsEveryPayload -v ./engrave/`):
```
dim 29 v3: observed max 347, budget 391, headroom 44
dim 33 v4: observed max 478, budget 547, headroom 69
dim 37 v5: observed max 617, budget 684, headroom 67
dim 41 v6: observed max 787, budget 843, headroom 56
dim 45 v7: observed max 918, budget 1013, headroom 95
dim 49 v8: observed max 1134, budget 1199, headroom 65
dim 53 v9: observed max 1348, budget 1399, headroom 51
```
Exact match to the plan/spec's table.

**ECC-L thresholds** (independent probe, not the shipped test — `qr.Encode`
swept 1..240 bytes, dim transitions logged directly): first byte reaching
dim 53 is **193**, first reaching dim 57 is **231** — confirms "v9 holds 230
bytes, 192 is v8's cap" (fid M-1) from first principles, not by re-reading the
shipped test's PASS.

**Cut durations** (`go test -run TestHashlockPlateCutDurationsAreBounded -v ./backup/`):
```
worst-case phrase plate WITH the v9 QR   43m31s
the same plate with the QR removed       14m39s
the string-form plate                    12m14s
the v9 QR alone                          32m12s (dim 53, scale 2)
one 6 mm character                       7s
```
Exact match. **Mutation — production engraving speed ÷3:**
```
worst-case phrase plate WITH the v9 QR takes 1h14m43s to cut, over the 1h0m0s bound
```
Exact match, reverted clean. **Mutation — QR at scale 3** (the "obvious" one):
reds earlier, at the layout gate, across three tests simultaneously:
`the hashlock plate does not fit at any font size: 10 rows at 3.0mm need
510080 units against a budget of 416000` — exact match to the plan's quote;
reverted clean.

**Firmware**: `nix develop -c tinygo build -size short ... ./cmd/controller`
→ **1,643,580 B flash / 63,272 B ram**, byte-identical to the fold's claim.
**`cmd/emu/build.sh`**: `built emu.wasm (11011084 bytes)` — exact match.

---

## 7. §8.2.4 wording (fid I-6 = jrn M-1)

**Mutation — restore "the passphrase above"** in `report_sealed_preimage`
(`crates/me-cli/src/main.rs`), ran the new integration test:
```
thread 'the_sealed_transit_note_does_not_point_the_wrong_way' panicked:
§8.2.4 points ABOVE at a passphrase that is printed BELOW it
```
Exact match. Reverted; full `me` suite back to 633/630/3(history_purge)/2skip.

---

## 8. Task 1 Step 6's release checklist (fid I-8)

Read `mnemonic-secret/design/RELEASE_PROCESS.md` directly (not the plan's
transcription): items 1-8 are Wire-format SHA pin / CHANGELOG / CI gate (build,
test, clippy, fmt, stable+beta+MSRV 1.85) / phase convergence / MIGRATION.md /
cross-repo notification / `cargo publish --dry-run` / tag+push — all eight
match the plan's Task 1 Step 6 citations exactly, item for item.
`ci/repro/vendor-freshness.sh` exists in the `ms` tree (confirmed by `ls`).

---

## 9. §8.2.2 silencing + jrn I-1's operator-facing fixes

**Mutation — restore the unconditional `carriers.is_empty()` warning**
(drop the new `hashlock_carrier_shaped` guard):
```
thread 'the_no_op_warning_is_silent_when_a_carrier_shaped_record_is_present' panicked:
§8.2.2 fired above the refusal for the wrong-id plate, and the two contradict each other:
me: --pack-preimage was passed and this payload holds no preimage plate...
me: record 0 ... is a kind-0x03 preimage payload whose 4-character id is not `hash`...
```
Exact match. Reverted clean.

**Mutation — move the payload-wide note back inside the per-carrier loop:**
```
thread 'the_no_hash_record_note_prints_once_per_payload' panicked:
assertion `left == right` failed: the payload-wide note printed 2 times
```
Exact match. Reverted clean.

**Live reproduction of journey I-1's own journey**, built `me` from my copy:
```
$ me sysw pack --help | grep -A1 phrase:
`phrase:<hex of "<method>,<phrase>">` is a hashlock PHRASE and the method that
derives its preimage...

$ printf 'phrase:%s\n' "$(printf '%s' 'correct horse battery staple' | xxd -p -c256)" > ph.txt
$ me sysw pack --pack-preimage --no-passphrase --in ph.txt --out /tmp/x.bin
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:`/`phrase:` record
    whose body fails its rule...
    Build the record with `me sysw pack`'s helpers: ... a phrase record is
    `phrase:` + the hex of `<method>,<phrase>`...
```
The refusal no longer misnames the record kind and now offers the phrase
recipe — the exact defect journey I-1 reported is gone.

---

## 10. The HOLD confirm modal's fifth falsified record (jrn I-2)

**Re-run**, unmutated: `342 chars drawn in full, headroom 107 chars (margin 80)`
— exact match.

**Independent reconstruction of the finding's own declined suggestion**
("…and can cut a plate for them at Done…"): swapped it in and re-ran the fit
test — **headroom 64**, refused under the 80-char margin, exactly as the fold
claims (`fits today with only 64 characters to spare`). My own re-typed
wording measured 359 drawn rather than the fold's quoted 364 (a small
transcription variance in my reconstruction of the illustrative "e.g." text,
which was never given as one exact byte string to reproduce) — the
**headroom, and the refusal itself, both matched exactly**, which is the
property that matters (the alternative is genuinely too long and genuinely
refused, not merely asserted to be).

---

## 11. Two new Minor findings (non-blocking)

**M-new-1 — "both repos pin `1.85.0` in `rust-toolchain.toml`" is imprecise for `me`.**
`mnemonic-secret` (`ms`) does: `/scratch/code/shibboleth/.tmp/h6-r1-ms/rust-toolchain.toml`
pins `1.85.0`. `mnemonic-engrave` (`me`) has **no `rust-toolchain.toml` at all**
(confirmed absent in both the gated tree and the live repo checkout); its
1.85.0 pin lives in `.github/workflows/release.yml`'s `RUST_TOOLCHAIN` env var
instead (`RUST_TOOLCHAIN: '1.85.0'`, used by two `dtolnay/rust-toolchain@master`
steps). The underlying conclusion — clippy cannot be proven on this box's
`rustc 1.98.0` and Task 1 Step 6 correctly says so rather than claiming a green
it doesn't have — is still true, and this is a citation-mechanism detail, not a
wrong verdict. Wording fix: name the two different mechanisms rather than one
file both repos supposedly share.

**M-new-2 — journey I-1's toolkit-manual clause was not folded.** The finding's
own SUGGESTION had three parts: the `Pack` doc comment, `U::Composer`/
`U::Unrecognised`, and *"Task 13 Step 3's toolkit-manual entry should carry the
wire form too, not only `--pack-preimage` and the plate forms."* The fold's
Task 3 Step 8b fixed the first two (verified live in §9); Task 13 Step 3 is
**untouched by this fold** (`git diff e6d84d9c..7021ea43` has zero hits for
"toolkit manual" or "Step 3" in Task 13) and still reads *"the toolkit manual —
`--pack-preimage`, the two plate forms, §8.6's text..."* with no `phrase:`
mention. The core operator-facing defect (a misleading `--help`/refusal) is
genuinely fixed and verified live; this is a residual completeness gap in a
different repo's reference doc, correctly out of scope for any gate this plan
runs. Not blocking — file it as a follow-up on Task 13.

---

## 12. Superseded phrasing — grepped, both documents

Searched the plan and spec for every phrase named as superseded across all
three round-0 reports and the fold's own list (`"disjoint from every other
group"`, `"Group F/G"`, `"192 bytes"` [unqualified], `"the passphrase above"`
[unqualified], `"run it three times"`, `"WHICH NO TASK WIRED"` [unqualified],
`is_ms1_shaped` [as a live function], `"3 + 1 + 2 + 1 + 3"`, `"~2 s a try"` /
`"~21 min"` [unqualified], `"at least one of those digests came from a phrase
typed here"` [as normative], stale `621`/`618` counts outside Task 2's own
section). Every hit in both documents is inside a deliberate quotation
contrasting old vs. new wording (all confirmed by reading the surrounding
sentence) — none live as normative text. The one `621 tests run: 618 passed`
citation still present (plan line ~870) is Task **2**'s own correct,
unaffected boundary-gate count, not a stale copy of Task 3's (which the fold
correctly changed to 633/630).

---

## 13. Full-suite gates, re-run in my own copies

```
ms    cargo fmt --check                                clean
ms    cargo nextest run --locked                        562 tests run: 562 passed, 11 skipped
me    cargo fmt --check                                 clean
me    cargo nextest run --locked -p mnemonic-engrave --no-fail-fast
                                                        633 tests run: 630 passed, 3 failed
                                                        (history_purge x3, box-local /usr/bin/zsh), 2 skipped
fork  go test ./engrave/ ./backup/ ./codex32/ ./sysw/ ./hashlock/ ./cmd/emu/    all ok
fork  scripts/gui-shard-test.sh ./gui/ 24               1286 top-level tests, partition verified
                                                        exhaustive: 1286 == 1286, 24 of 24 shards ok, wall 26s
fork  gofmt -l gui/ sysw/ backup/ engrave/ codex32/ hashlock/ cmd/emu/
                                                        the three baseline files (gui/transaction*), no fourth
fork  go vet ./gui/                                     the two baseline ArtifactDir diagnostics
fork  GOOS=js GOARCH=wasm go vet ./cmd/emu/             exit 0
fork  ./cmd/emu/build.sh                                exit 0, built emu.wasm (11011084 bytes)
fork  nix develop -c tinygo build -size short ... ./cmd/controller
                                                        1611456 code / 32124 data / 31148 bss
                                                        flash 1643580, ram 63272
engrave scripts/h6-plan-blocks-vs-tree.sh design/IMPLEMENTATION_PLAN... \
        .tmp/h6-r1-gate .tmp/h6-r1-ms .tmp/h6-r1-me     97 blocks checked, 0 FAIL
Task counts, re-grepped: backup/hashlock_test.go = 10 Test funcs;
        sysw_pack_preimage.rs = 12 #[test] rows                match the plan's claims exactly
```

**Not re-run**: the browser walk (no browser available to this agent, same as
the fold author; `cmd/emu/walk_hashlock_phrase.js` sha256 unchanged, confirmed
by the fold, not re-derived here since the file is byte-identical and
untouched by this fold's diff — verified via `git diff e6d84d9c..7021ea43`
touching no file under `cmd/emu/`); the 4×32-minute fuzzing campaigns (cited,
not re-run, same as every prior round — re-running is 128 minutes of wall
clock that does not change the truth value of a number that already reproduces
at 400-sample scale); clippy at the pinned 1.85.0 toolchain (this box resolves
1.98.0; the plan correctly declines to claim a green it cannot have — see
§11's M-new-1 on the citation itself).

---

## Closing counts

**0 Critical, 0 Important open.** All 1 Critical + 13 Important findings named
by round 0 are fixed and independently reproduced by mutation or measurement,
or truly declined on a measurement that itself reproduces (jrn I-2's suggested
wording, jrn I-3's widened predicate). The checker passes independently
(97/0). Every full-suite count matches exactly. Every mutation I ran matches
the fold's quoted output byte-for-byte (module count, headroom, digest
sentinel, error string) except one immaterial re-typing variance in my own
reconstruction of an illustrative alternative wording, where the property that
mattered (refused, headroom 64) matched exactly. The merged group's files are
mechanically confirmed disjoint from every group that runs in parallel with
it. `cargo fmt --check` is clean in both Rust trees. No superseded phrasing
found live in either document.

**2 new Minor findings** (§11), both non-blocking: a citation-mechanism
imprecision (`rust-toolchain.toml` claimed for a repo that pins it in CI
config instead) and one unfolded clause of journey I-1's suggestion (the
toolkit-manual doc, in a different repo, not gated by anything here).

## GREEN

This closes the r1 fold-verification gate for the plan. R0 round 0 is folded
and verified.
