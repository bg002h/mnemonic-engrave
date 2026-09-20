# Fold verification — Liana-model consent notice, composer fable review r0 lens 5 (sonnet, mechanical)

**Verdict: GREEN. 55/56 shapes AGREE, 1 documented carve-out (§8g), 0 disagreements. 4/4 independently-run
mutations RED (class 1, class 3, class 7, and class 9's two halves in one mutation). Doc-comment gate and
modal-fits assertion independently re-run and confirmed. One new Minor finding (test-coverage gap, not a
functional defect). No deviations from the brief found beyond the implementer's own declared none.**

Repo `/scratch/code/shibboleth/seedhammer`, range `781dc7cd..fa070df0` (one commit `fa070df0`), reviewed in a
detached worktree at `/scratch/code/shibboleth/.tmp/verify-liana` (removed at the end, tree left clean — `git
status --porcelain` empty, `diff -q` against the committed `composer_consent.go` matched after every mutation
was reverted). Read-only; nothing committed; no sub-agents; no `.jsonl` read except the two evidence files the
brief named. Go `/scratch/code/shibboleth/.toolchain/go/bin/go` (`go version` → `go1.26.7`) first on PATH,
`TMPDIR=/scratch/code/shibboleth/.tmp` throughout.

## The ONE QUESTION

**Does the predicate name Liana's FIRST applicable class for every path list, in the brief's order, and stay
silent exactly when Liana 8.0 accepts?** Yes, for all 56 shapes but one, and that one exception is a
pre-existing, documented design carve-out, not a defect.

Method: built `md.PathList` structs reproducing all 56 shapes from `fable-liana-shapes.json`'s `desc_md`
descriptors (wrapper + per-path keys/lock/hash, cross-checked against the report's matrix table), ran each
through `md.Compose` → `.Chunks()` → `composerListedPaths` → `composerConsentLinesFor` — the exact pipeline
the fold's own test uses — in a throwaway package-`gui` test file (`gui/zz_verify_liana_agreement_test.go`,
never committed, deleted before the worktree was removed), and compared "does `OUTSIDE LIANA'S MODEL` fire"
against the `ok` field of `fable-liana-parse-out.jsonl`'s `variant: "md"` records (56/56 present, extracted
independently with a one-off Python script over the JSONL, not read as a whole transcript). 14 of the 56 rows
used `composerPresets()` directly (the same presets the fold and the device ship) rather than hand-built
`SpendPath`s, to remove transcription risk for K/N/sorted/lock values on the six named archetypes.

Command:
```
CGO_ENABLED=0 go test ./gui/ -run '^TestZZLianaAgreementMatrix$' -count=1 -v
```
Result: `TOTALS: agree=55 disagree=0 carve-out=1 of 56`, test **PASS**.

### Shape-by-shape agreement table

`fires` = predicate produced `OUTSIDE LIANA'S MODEL`; `class` = the extracted `<class>` phrase; `Liana ok` =
measured truth (`fable-liana-parse-out.jsonl`, variant `md`); `want fire` = `!ok`, except X24 (named exception,
want fire = true regardless).

| # | shape | fires | class | Liana ok | want fire | agree? |
|---|---|---|---|---|---|---|
| 1 | single-tr | true | no locked path | false | true | AGREE |
| 2 | single-wsh | true | no locked path | false | true | AGREE |
| 3 | plain-2of3-shwsh | true | legacy wrapper | false | true | AGREE |
| 4 | plain-2of3-sh | true | legacy wrapper | false | true | AGREE |
| 5 | plain-2of3-wsh-DEMO | true | no locked path | false | true | AGREE |
| 6 | plain-2of3-tr | true | NUMS key path | false | true | AGREE |
| 7 | plain-2of3-wsh-UNSORTED | true | no locked path | false | true | AGREE |
| 8 | preset-simple-timelocked-inheritance-wsh | false | — | true | false | AGREE |
| 9 | preset-kofn-recovery-wsh | false | — | true | false | AGREE |
| 10 | preset-tiered-recovery-wsh | false | — | true | false | AGREE |
| 11 | preset-hashlock-gated-wsh | true | a hash lock | false | true | AGREE |
| 12 | preset-decaying-multisig-wsh | true | an absolute lock | false | true | AGREE |
| 13 | preset-simple-timelocked-inheritance-tr | false | — | true | false | AGREE |
| 14 | preset-kofn-recovery-tr | true | NUMS key path | false | true | AGREE |
| 15 | preset-tiered-recovery-tr | true | NUMS key path | false | true | AGREE |
| 16 | preset-hashlock-gated-tr | true | NUMS key path | false | true | AGREE (order-priority: NUMS+hash both apply, NUMS wins) |
| 17 | preset-decaying-multisig-tr | true | NUMS key path | false | true | AGREE |
| 18 | keyless-hash-path-wsh | true | no locked path | false | true | AGREE |
| 19 | keyless-hash-older-path-wsh | true | a hash lock | false | true | AGREE |
| 20 | hash160-gated-wsh | true | a hash lock | false | true | AGREE |
| 21 | ripemd160-gated-wsh | true | a hash lock | false | true | AGREE |
| 22 | hash256-gated-wsh | true | a hash lock | false | true | AGREE |
| 23 | hashlock-gated-tr-hash160 | true | NUMS key path | false | true | AGREE |
| 24 | older-units-wsh | true | a lock in time units | false | true | AGREE |
| 25 | after-time-wsh | true | an absolute lock | false | true | AGREE |
| 26 | mixed-lock-bases-wsh | true | an absolute lock | false | true | AGREE |
| 27 | mixed-lock-bases-tr | true | an absolute lock | false | true | AGREE |
| 28 | same-seed-two-accounts-wsh | true | no locked path | false | true | AGREE |
| 29 | same-seed-two-paths-tr | true | NUMS key path | false | true | AGREE |
| 30 | nine-of-nine-wsh | true | no locked path | false | true | AGREE |
| 31 | X01-wsh-1of1-2of3older | false | — | true | false | AGREE |
| 32 | X02-wsh-2of3-tworec | false | — | true | false | AGREE |
| 33 | X03-wsh-2of3-tworec-samelock | true | two paths with one lock | false | true | AGREE |
| 34 | X04-wsh-2of3-plus-1of1-unlocked | true | no locked path | false | true | AGREE |
| 35 | X05-wsh-1of1-plus-2of3-unlocked | true | no locked path | false | true | AGREE |
| 36 | X06-wsh-1of1-older-units | true | a lock in time units | false | true | AGREE |
| 37 | X07-wsh-1of1-after-height | true | an absolute lock | false | true | AGREE |
| 38 | X08-wsh-1of1-older-65535 | false | — | true | false | AGREE |
| 39 | X09-tr-1of1-2of3older | false | — | true | false | AGREE |
| 40 | X10-tr-1of1-tworec | false | — | true | false | AGREE |
| 41 | X11-wsh-samefp-in-path | false | — | false | true | **CARVE-OUT** |
| 42 | X12-wsh-samefp-across-paths | false | — | true | false | AGREE |
| 43 | X13-wsh-2of2-2of2older | false | — | true | false | AGREE |
| 44 | X14-wsh-1of1-1of2older | false | — | true | false | AGREE |
| 45 | X15-wsh-hashlock-known | true | a hash lock | false | true | AGREE |
| 46 | X16-wsh-kofn-older5 | false | — | true | false | AGREE |
| 47 | X17-wsh-decaying-small | true | no unlocked path | false | true | AGREE |
| 48 | X18-tr-inherit-older5 | false | — | true | false | AGREE |
| 49 | X19-tr-kofn-nums-older5 | true | NUMS key path | false | true | AGREE |
| 50 | X20-tr-hashlock-known | true | NUMS key path | false | true | AGREE |
| 51 | X21-wsh-preset-decaying-default | true | an absolute lock | false | true | AGREE |
| 52 | X22-wsh-1of1-2of3older-tr-origins | false | — | true | false | AGREE |
| 53 | X23-tr-1of1-1of1older-h-spelling | false | — | true | false | AGREE |
| 54 | X24-wsh-2of3-1of1-unlocked-plus-rec | true | a second unlocked path | **true** | true (NAMED EXCEPTION) | AGREE |
| 55 | X25-wsh-1of1-2of3-unlocked-plus-rec | true | a second unlocked path | false | true | AGREE |
| 56 | X26-tr-2of3-1of1-unlocked-plus-rec | true | a second unlocked path | false | true | AGREE |

**X24 (I-2's declared exception): CONFIRMED.** Fires with class `a second unlocked path` even though Liana's
`LianaDescriptor::from_str` accepts it (silently folding the second single-key unlocked path into the
2-of-3's threshold set, per the finding). Independently reproduced, matching the implementer's own report.

**X11 (one CARVE-OUT, not a disagreement).** `X11-wsh-samefp-in-path` is structurally identical, at the
`md.PathList`/`PolicyShape` level, to `preset-kofn-recovery-wsh` (2-of-3 unlocked + one `older(26280)`
recovery) — same-seed duplication is a property of *bound* keys (fingerprints), which `md.Compose` does not
even require at this stage (`Composed` is "not-yet-keyed"), so `composerLianaOutsideModelClass` structurally
cannot see it and correctly stays silent. Liana refuses X11 for `DuplicateOriginSamePath`, a *different*
reason the brief explicitly assigns to §8g ("Same-seed inside one path is already §8g's 'Liana will refuse
it' — do not duplicate" — verbatim in both the original brief and the fold's own doc comment). This is a
designed, pre-authorized exclusion, not a fold defect: the consent screen shows §8g's own same-seed notice
for this shape independently of this new notice. No other shape in the 56 needed this carve-out — `X12`
(same fingerprint *across* paths, which Liana accepts) and `same-seed-two-accounts-wsh` /
`same-seed-two-paths-tr` (whose *other* structural properties — sole bare-multi / NUMS — already make Liana
refuse them, coincidentally landing on the same fire/silent answer) all AGREE cleanly with no carve-out
needed.

## Mutations — 4 of 9 classes re-run independently, all RED

Re-ran mutations independently (own test file, own path lists — not reusing the implementer's test rows),
covering class 3 (required), class 9's two halves (required), plus classes 1 and 7 as the other two of "at
least four." Each mutation was applied with `sed`, tested, then reverted; `diff -q` against a saved original
confirmed byte-exact restoration after every mutation.

| class | mutation | result |
|---|---|---|
| 1 (legacy wrapper) | `root == md.ScriptSh` → `root == md.ScriptWsh` | RED: `class1-sh` (a `sh` plain-multisig) now reports `no locked path` instead of `legacy wrapper`; every `wsh` row in the targeted set now *wrongly* reports `legacy wrapper` (5 assertion failures) |
| 3 (no locked path) | `case !anyLock:` → `case anyLock && false:` | RED: `class3-demo` (plain-multisig/wsh) goes from firing `no locked path` to silent (`fires=false`) |
| 7 (no unlocked path) | `case unlocked == 0:` → `case unlocked == -1:` | RED: `class7-no-unlocked` (decaying, two locked paths, zero unlocked) goes silent |
| 9, BOTH halves (second unlocked path) | `if unlocked >= 2 {` → `if unlocked >= 3 {` | RED on **both** constructed rows in the SAME mutation: the X24-style single-key-second-unlocked shape AND an independently-built two-MULTI-key-unlocked shape (`[2of3 unlocked, 2of2 unlocked, older(100) recovery]` — not present in the 56-shape corpus at all) both go silent |

The class-9 double-row result is the mechanical answer to "class 9's two halves": the code's own doc comment
claims the class deliberately unifies two different real Liana behaviors (outright refusal of a second
unlocked MULTI path vs. silent folding of a second unlocked SINGLE-key path) under one label. Constructing a
genuine second-unlocked-MULTI shape (which does **not** exist anywhere in the 56-shape corpus — every X24-family
shape in the corpus uses a single-key second path) and confirming it hits the exact same `unlocked >= 2` line
as the X24-style shape shows this is one code path, not two — the "two halves" are a semantic distinction in
Liana's behavior, not two branches in the Go predicate, so one mutation legitimately catches both.

## Modal-fits assertion and doc-comment gate — independently re-run

- `TestFableOutsideLianaModelNamesTheFirstClass` (the fold's own test): re-run standalone, **PASS**, printed
  `the outside-Liana-model notice, longest class line: 143 chars drawn in full, headroom 397 chars (margin
  80)` — matches the implementer's report exactly. Verified independently that `two paths with one lock`
  (23 chars) is in fact the longest of the nine class phrases (`a second unlocked path` is next at 22; the
  other seven are all ≤ 20), so the assertion target is correctly the longest line.
- `TestComposerHelpersDidNotStealADocComment`: re-run standalone, **PASS**.
- `composerDocOwners` completeness: `git diff 781dc7cd..fa070df0 -- gui/composer_consent.go
  gui/composer_copy.go | grep -E '^\+func '` lists exactly two new top-level functions
  (`composerLianaOutsideModelClass`, `composerCopyOutsideLianaModel`) — both are present in the updated
  `composerDocOwners` map. No new helper was left unnamed.
- `TestComposerCopyTableCoversEveryBody`: re-run standalone, **PASS** (86 declared bodies).

## New finding

**Minor — test-coverage gap: the `after`-vs-`older-units` compound-order priority (class 5 before class 6) has
no dedicated test row, unlike the `NUMS`-vs-`hash` priority (class 2 before class 4, which IS pinned by the
fold's own row 10).** `mixed-lock-bases-{wsh,tr}` are the only shapes in the 56-shape corpus that exercise
both `after(...)` and `older` in 512-second units simultaneously, and the predicate correctly reports `an
absolute lock` (class 5) for both, matching Liana's model membership (fires, since Liana refuses both) — so
there is no functional defect, confirmed by direct execution. But real Liana's *own* observed error message
for these two shapes is `InsaneTimelock` (the class-6 mechanism), not `IncompatibleDesc` (class 5's), per the
review report's own matrix (`design/agent-reports/composer-fable-r0-liana-core.md` lines 238-239) — the
report itself already notes this ("`IncompatibleDesc` (or `InsaneTimelock` when the same policy also carries
units)"). The fold's ordering is a deliberate, brief-authorized simplification (same principle as the
decaying-multisig-wsh case the implementer's mutation 5 already demonstrates for a different pair), not an
error — but no test row pins the after/older-units order the way row 10 pins the NUMS/hash order, so a future
edit that silently reordered classes 5 and 6 would not be caught by the existing suite. Not blocking (Minor):
the fire/silent answer is unaffected either way, and the ordering choice is exactly the kind of thing the
predicate's own doc comment already flags as a named simplification, not a bug.

## Deviations from the brief

None found, beyond the implementer's own declared "None." The implementer's report did not name the exact
file for the new test (`gui/composer_fable_r0_funds_test.go`, appended to, not created) — confirmed by `git
diff` that this file pre-existed at `781dc7cd` and only gained the one new test function; not a deviation,
just an omission of a filename in the report.

## What was not re-verified (out of this lens's scope, per the brief)

`go vet`, `gofmt -l .` baseline, the full `gui` shard set (1374/1374), and the firmware size delta were
already settled by the controller at `fa070df0` per the brief and were not re-run here.

## Files

- Verification harness (throwaway, not committed): `gui/zz_verify_liana_agreement_test.go` — deleted before
  the worktree was removed.
- Ground truth: `/scratch/code/shibboleth/mnemonic-engrave/design/evidence/composer-fable-r0/fable-liana-parse-out.jsonl`
  (variant `md`, 56 records) and `fable-liana-shapes.json` (56 shapes, `desc_md` descriptors).
- Code under review: `gui/composer_consent.go` (`composerLianaOutsideModelClass`), `gui/composer_copy.go`
  (`composerCopyOutsideLianaModel`), `gui/composer_copy_test.go`, `gui/composer_doc_comment_test.go`,
  `gui/composer_fable_r0_funds_test.go` (`TestFableOutsideLianaModelNamesTheFirstClass`), all in
  `/scratch/code/shibboleth/seedhammer` at commit `fa070df011b382ae9baa1c54c93bbf353d902c59`.
