# Composer Stage 2 — fold verification, r1

**Reviewer:** independent agent; did not write the implementation, the review r0, or the fold.
**Scope:** mechanical — did the fold `74ec6e9` fix each of r0's findings exactly, and did it
introduce a new defect? Not a fresh audit.
**Under review:** fork `wt-composer-s2` @ `composer-s2` `74ec6e9`, against the review it responds
to (`design/agent-reports/composer-S2-exec-review-r0.md`, 1C/1I/4M/1N), plus the Rust-first host
fold `mnemonic-engrave` @ `c05074f1`.
**Read-only:** every mutation reverted; `git status --porcelain` empty in both trees at the end
(fork fully clean; host carries one pre-existing untracked file,
`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`, present before this review started and
untouched by it).

## Result

**0 findings folded wrongly on the substantive (C/I/M) set. 1 new documentation-placement defect
in the N-1 fold — Important, per this review's severity rule ("a finding folded wrongly ...is
Important unless cosmetic"; this one relocates real prose to the wrong section and leaves the
original stale claim exactly as stale as r0 found it).**

C-1, I-1, M-1, M-2, M-4 VERIFIED clean. M-3 VERIFIED. N-1 NOT VERIFIED as folded — see finding
**V-1** below.

---

## C-1 — VERIFIED

**Claim:** `parseOriginPath` now refuses any path component whose numeric value is >= 2^31,
hardened or not, closing the alias where an unhardened `2147483648` was silently re-read as
hardened `0`.

**Construction, both sides, unmutated code:**

```
record (fixture row key-origin-component-unhardened-2^31):
  key:5b3733...4b467266
  = "[73c5da0a/2147483648/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"
```

Go, throwaway test `sysw/zzverify_test.go` (deleted after use), `go test -mod=readonly -run
TestZZVerifyC1AndI1 -v ./sysw/`:
```
zzverify_test.go:16: C-1 unhardened-2^31 -> Classify=0        (ClassUnknown = iota, value 0)
zzverify_test.go:27: 2^31-1 unhardened -> ClassKey (as expected)
--- PASS
```
So the device now: refuses the 2^31 record (`ClassUnknown`), and still accepts `2147483647`
(2^31-1, the last legal unhardened index) as `ClassKey` — both halves of the claim hold.

**Mutation.** Reverted the guard with `sed -i 's/if idx >= 1<<31 {/if false \&\& idx >= 1<<31 {/'
sysw/composer_records.go` (confirmed the file actually changed via `diff` against a pre-copy):

```
$ go test -mod=readonly -run 'TestComposerRecordsClassifyExactlyAsTheHost|TestKeyRecordPathGrammarMatchesTheHost' -v ./sysw/
--- FAIL: TestComposerRecordsClassifyExactlyAsTheHost
    key-origin-component-unhardened-2^31: Classify(...) = 10, want 0 (host's answer)
--- FAIL: TestKeyRecordPathGrammarMatchesTheHost
    "73c5da0a/2147483648/0'/0'/2'" accepted
    "73c5da0a/48'/2147483648/0'/2'" accepted
```
Both named gates fail exactly as the brief predicted. Restored from the pre-copy;
`git diff --quiet sysw/composer_records.go` confirmed clean afterward.

**Host side:** the fixture's `host_line` for this row is the fixed §8n `Key` refusal line
(`"record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record"`)
— read `ComposerRecordError::line()` in `crates/me-cli/src/sysw/composer_records.rs:80-88`: every
`Key(_)` variant renders through the same fixed sentence regardless of its internal `detail()`, so
an identical `host_line` across many `Unknown` rows is the existing, intentional design, not a
sign the new row was misclassified.

## I-1 — VERIFIED

**Claim:** the host (Rust-first, per the ruling this needed) now refuses a `+`-signed path
component, converging on the device's pre-existing refusal; both sides pinned by a new fixture
row.

**Host, `me sysw pack`, the three shapes named in r0** (built with `--no-passphrase --no-now
--allow-argv-secret`, binary rebuilt from `c05074f1`):

```
+48'        -> exit 4, "path component is not digits with an optional ' or h"
48'/+0'     -> exit 4, "path component is not digits with an optional ' or h"
48'/0'/+0'  -> exit 4, "path component is not digits with an optional ' or h"
```
All three exit 4 with the detail the fold's commit claims.

**Device:** same throwaway test, `Classify` on the fixture row `key-origin-component-plus-sign`
(`[73c5da0a/+48'/0'/0'/2']xpub...`) → `ClassUnknown` (0). Matches the host.

**Rust unit test (`key_origin_rules_are_each_enforced`)** asserts the exact new detail string —
ran it directly: `cargo test --locked -p mnemonic-engrave --test sysw_composer_records
key_origin_rules_are_each_enforced -- --nocapture` → `test key_origin_rules_are_each_enforced ...
ok`.

**Fixture regeneration, sha256, provenance.** Ran the ignored `regenerate` test directly:
```
$ cargo test --locked -p mnemonic-engrave --test sysw_composer_records regenerate -- --ignored --nocapture
wrote 47 rows to .../testdata/record_class_vectors.json
sha256 5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312
test regenerate ... ok
```
`git status --porcelain` was empty afterward — the regenerated file is byte-identical to what's
committed, so the sha256 in the commit message, the CHANGELOG, and the fork's
`record_class_vectors.provenance.json` (`5b3960ca...` again) all agree with the real artifact, not
just with each other.

**Only two rows changed.** Diffed `c05074f1^:crates/me-cli/testdata/record_class_vectors.json`
(45 rows) against `c05074f1:...` (47 rows) in Python: `old == new[:45]` is `True`; the two added
rows are exactly `key-origin-component-unhardened-2^31` and `key-origin-component-plus-sign`. No
other row's class moved.

**CHANGELOG.** `[Unreleased] / Changed` in `crates/me-cli/CHANGELOG.md` states the `+`-sign
refusal, names the fixture and row count (47), and cites C-1/I-1 — matches what shipped.

**Provenance object** (`sysw/testdata/record_class_vectors.provenance.json`, fork side):
`sha256` = `5b3960ca...` (matches the file, matches the regenerated file), `vectors` = 47,
`commit`/`file_commit` = `c05074f1d45970ca416785dfa9d9a812aaa21dbd`; `git -C mnemonic-engrave log
-1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json` returns the same SHA. All
four checks pass.

**Cross-language lockstep, unmutated code:** `go test -mod=readonly -run
TestComposerRecordsClassifyExactlyAsTheHost -v ./sysw/` → `PASS` (47/47 rows). `go test
-mod=readonly -run TestKeyRecordPathGrammarMatchesTheHost -v ./sysw/` → `PASS`.

## Provenance — VERIFIED (folded into I-1 above; restated as its own line item per the brief)

sha256 match, `vectors: 47`, `commit`/`file_commit` both `c05074f1...`, and that commit is the one
that actually touches the fixture (`git log -1 --format=%H -- ...` agrees) — all confirmed above.

## M-1 — VERIFIED

The fold's new block in `gui/wallet_policy_test.go` (before the existing `consentText(t,
"gap_wsh_andor")` call) adds exactly **four** preconditions on the `gap_wsh_andor` fixture:

1. every key has `XpubPresent == true` (rules out the no-keys guard),
2. `expandedKeysToBip380(keys)` returns `ok == true` (rules out the use-site guard — **this is the
   gap r0 named**: nothing previously ruled this one out),
3. `expandedToDescriptor(tpl, keys)` returns `status == expandUnsupported` (must reach the complex
   branch),
4. `complexAddressSource(chunks, keys)` returns `ok == false` (it must be *this* probe that
   refuses).

Read from the code (a mutation was optional per the brief and was not run, given check 2 is a
direct, unconditional `t.Fatal` on the exact failure mode r0 named — any of the four returning the
"wrong" side for the layer it names fails the test immediately with a message identifying which
layer tripped, which is the property M-1 asked for). Ran the test standalone to confirm current
behavior: `go test -mod=readonly -run TestWalletPolicyConsentNeverHidesTheAbsenceOfAddresses -v
./gui/` → `PASS`.

## M-2 — VERIFIED

`gui/policy_address_test.go`'s comment above `TestThePkhTapLeafGapIsCLOSED` was changed from
"a hand-built shape instead" to "the vendored `gap_wsh_andor` fixture instead" — matches: that
fixture is real (loaded via `loadVectorChunks(t, "gap_wsh_andor")` in `gui/wallet_policy_test.go`)
and is what the referenced test actually points at.

## M-3 — VERIFIED

`md/testdata/README.md`'s `gap_wsh_andor` section no longer promises the fixture "becomes the
next positive one, exactly as its two predecessors did." It now says the opposite is true unless
someone acts first: "this fixture carries no vendored Rust addresses, so whoever closes the gap
must first generate `gap_wsh_andor.conformance.json` from the primary ... to have ground truth."
This is the "reword" branch of r0's hypothesis (r0 offered vendor-now or reword; the fold chose
reword) — a legitimate fold of the finding.

## M-4 — VERIFIED

`md/compose_stubs.go:18` now reads `if len(keyedChunks) > 0` (was `if keyedChunks != nil`).
Wrote a throwaway addition to `md/compose_stubs_test.go` (reverted after) exercising exactly the
case r0 named:
```go
emptyKeyed, err := ComposerStubs(template, []string{})
// err == nil; len(emptyKeyed) == 1; emptyKeyed[0] == both[0]
```
`go test -mod=readonly -run TestComposerStubsAreTheTwoIdsFirstFourBytes -v ./md/` → `PASS`. A
non-nil empty `keyedChunks` now behaves identically to `nil` — one stub, no spurious error from
`FormAwareStubChunks`. File restored from a pre-edit copy; `git diff --quiet` confirmed clean.

## N-1 — NOT VERIFIED AS FOLDED (new finding: **V-1**, Important)

**r0's ask:** `md/testdata/README.md` still described `gap_tr_leaf_and_v` as an open gap after
F-214 closed it; say so where the stale claim lives.

**What the fold actually did.** `git show 74ec6e9 -- md/testdata/README.md` inserts a
"**Status 2026-09-02: CLOSED.**" paragraph — but it lands in the **wrong section**. Exact
structure, current file, line numbers via `awk`:

```
126: ### `gap_tr_leaf_and_v` (Stage 4, added 2026-08-20)
...
139-140: ... when the emitter grows `and_v`/`older` leaves the
         test FAILS saying the gap is closed, rather than going quiet.      <- STILL says "refuses" (untouched)
142: ## The compose corpus (wallet-policy composer, Stage 2)                <- unrelated new section
...
156: mirrored as chunk-set literals in `md/compose_test.go`, produced by ...
159: **Status 2026-09-02: CLOSED.** The paragraph above describes the gap as it was
160: filed. F-214's emitter grew `and_v`/`older` leaves, the pinned test failed with
...
164: in this repo.
165: ### `gap_wsh_andor` (composer Stage 2 fold, added 2026-09-02)          <- no blank line before heading
```

Diffed against the parent fold (`git show 489d52e:md/testdata/README.md`, lines 100-161) to
confirm the insertion point precisely: the new paragraph was spliced in immediately **after** the
"## The compose corpus" section's intro paragraph (about the 26 `compose_*`/`keyed_compose_*`
vectors) and immediately **before** "### `gap_wsh_andor`" — three paragraphs and one H2 heading
away from the `gap_tr_leaf_and_v` section it is actually about.

Two consequences, both real:

1. **N-1's actual target is unfixed.** The `gap_tr_leaf_and_v` section itself (lines 126-140) is
   byte-identical to before the fold — it still reads "so it refuses" and "when the emitter grows
   `and_v`/`older` leaves the test FAILS," with no closure note anywhere in it. A reader who reads
   that section will see exactly the stale claim r0 flagged.
2. **The landed text is wrong where it sits.** "The paragraph above describes the gap as it was
   filed" — the paragraph immediately above (lines 144-156) is the compose-corpus vector-provenance
   paragraph, which does not describe a gap at all. The sentence is a non sequitur in its landed
   position.

The fold commit message's claim ("N-1: README marks `gap_tr_leaf_and_v` CLOSED (F-214), as the
live test says") is true of the *prose written* but not of *where it was written* — the section
titled `gap_tr_leaf_and_v` is not where the closure note appears.

**Severity, per this review's rule** ("a finding folded wrongly ... is Important unless
cosmetic"): this is not cosmetic — the original stale claim survives untouched at its own
location, and the fix text is now attached to the wrong paragraph, which will mislead a reader
navigating by section rather than by search. Rated **Important**.
**Hypothesis (not authoritative):** move the "Status 2026-09-02: CLOSED" paragraph from its
current position (after line 156) to directly after line 140, before the `## The compose corpus`
heading.

---

## What else moved

`git show 74ec6e9 --stat`: 8 files, 73 insertions, 13 deletions.

| file | attributable to |
| --- | --- |
| `gui/policy_address_test.go` | M-2 (comment) |
| `gui/wallet_policy_test.go` | M-1 (4 preconditions) |
| `md/compose_stubs.go` | M-4 (`len(...) > 0`) |
| `md/testdata/README.md` | M-3 (reword) + N-1 (misplaced, see V-1) |
| `sysw/composer_records.go` | C-1 (range check + comment) |
| `sysw/composer_records_test.go` | C-1/I-1 (47-row count, two new bad-path assertions) |
| `sysw/testdata/record_class_vectors.json` | I-1 (2 new rows) |
| `sysw/testdata/record_class_vectors.provenance.json` | I-1 (sha256/vectors/commit re-pin) |

Every hunk maps to a named finding; nothing unattributed. Host side (`mnemonic-engrave`
`c05074f1`, 4 files): `crates/me-cli/CHANGELOG.md`, `crates/me-cli/src/sysw/composer_records.rs`,
`crates/me-cli/testdata/record_class_vectors.json`, `crates/me-cli/tests/sysw_composer_records.rs`
— all I-1, all accounted for.

## Gates re-run (not taken from the fold message)

Fork (`wt-composer-s2`, Go 1.26.7, `-mod=readonly`, `CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off`):
`go test -mod=readonly -count=1 ./md/ ./mk/ ./sysw/` → all `ok`; `gofmt -l` on the touched files →
empty; `git status --porcelain` → empty at every checkpoint (see the go.mod note below).

Host (`mnemonic-engrave`, `RUSTUP_TOOLCHAIN=1.85.0`): `cargo fmt --check` → exit 0; `cargo clippy
--locked --all-targets -- -D warnings` → exit 0; `cargo nextest run --locked` → 622 run, 622
passed, 2 skipped; `key_origin_rules_are_each_enforced` and the `regenerate` fixture check run
directly (above).

**Aside, not a fold defect:** a bare `go test ./sysw/` (without `-mod=readonly`) rewrote `go.mod`
(promoted `github.com/btcsuite/btcd/chainhash/v2` from indirect to direct) — a pre-existing
toolchain/module-graph quirk unrelated to this fold, reverted immediately with `git checkout --
go.mod`. All verification after that point used `-mod=readonly` explicitly and left `git status
--porcelain` empty.

## Closing counts

r0's set: **6 of 7 folded correctly** (C-1, I-1, M-1, M-2, M-3, M-4). **1 folded with a new
defect** (N-1 → **V-1**, Important: real text, wrong location, original stale claim still stale).

**0 Critical / 1 Important (V-1) / 0 new Minor / 0 new Nit.**
