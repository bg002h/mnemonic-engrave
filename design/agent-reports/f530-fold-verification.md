# F-530 fold verification

**Scope.** Two questions only: does the fold close each finding in
`design/agent-reports/f530-descriptor-rule-review.md` (committed at `40159165`),
and did the fold introduce a new defect. Fold under test: fork `2a40f5d..fcf8ac7`
(commits `d17a21f`, `fcf8ac7`), primary `40159165..fc0a3c28` (commit `fc0a3c28`).
Not a re-audit of F-530 or F-531. Every experiment below was constructed and
run against the real trees, then reverted; both trees are clean
(`git status --porcelain` empty in each) as of this report.

---

## Per-finding verdicts

### C-1 — CLOSED

`sameKeyExpression` (struct comparison) is gone; `descriptorRepeatsAKey` now
calls `address.DerivesSameKey(a, b)`, which normalises `Children` through the
same `effectiveChildren`/`defaultChildren` derivePubKey uses, ignores
`Hardened` (matching derivePubKey), resolves `RangeDerivation` per-chain via a
new `resolveChild`, and reports a duplicate on **either** chain.

Verified beyond the fold's own tests:

- Wrote a property test (`address/zz_probe_test.go`, removed after) that
  cross-checks `DerivesSameKey` against **actual `derivePubKey` output** over
  16 derivation shapes × 16 shapes × 8 indices × 2 chains (256 pairs,
  including wildcard-not-trailing, non-trailing range, and an unsupported
  `End != Index+1` range). **0 mismatches.**
- Confirmed all four C-1 spellings (`A` vs `A/<0;1>/*`, `A/0/*` vs
  `A/<0;1>/*`, `A/0h/*` vs `A/0/*`, `A/*h` vs `A/*`) are caught by the shipped
  code (`TestEverySpellingOfTheSameKeyIsCaught` passes; also re-derived by the
  probe).
- Checked the `derivePubKey` refactor (inline default → `effectiveChildren`
  returning a package-level `defaultChildren` slice) for aliasing: the loop in
  `derivePubKey` only ranges over `children`, never mutates or appends to it,
  so no address can move and `defaultChildren` cannot be corrupted through an
  aliased return. Confirmed by reading the full function body.
- `TestDisjointChildrenStillDerive`-class false positives (`A/0/*` vs `A/1/*`,
  disjoint multipath `<0;1>/*` vs `<2;3>/*`) still correctly return "not
  duplicate," both in the shipped tests and the probe.

No false negative or false positive found.

### C-2 — CLOSED, in both languages, correctly ordered

Rust first: `crates/me-cli/src/descriptor/admit.rs` conjunct 8(b) now compares
`derive::receive_path(a) == receive_path(b)` (made `pub(crate)`) instead of
`a.children == b.children`. `crates/me-cli/tests/descriptor_refusals.rs`'s
`row_key_identity_duplicate` now runs both the exact-duplicate and the
implicit-vs-explicit spelling over the same §6 row (vector
`gate/duplicate-key-implicit-use-site`). Go's `sysw/descriptor.go`
`keyIdentityOK` now calls the shared `address.DerivesSameKey(*a, *b)`,
explicitly commented as a **convergence port**, matching the Rust-primary
rule. `TestAdmissionRefusesEverySpellingOfADuplicateKey` (Go) passes for all
5 rows including the disjoint-multipath control (correctly admitted).

Checked the `Some`/`None` handling the brief flagged: `receive_path` returns
`None` only for a use-site outside conjunct 7's closed set, and
`admit()` runs `conjunct_7_use_site(d)?` (which loops over **every** key and
returns early via `?` on the first failure) strictly before
`conjunct_8_key_identity(d)?`. So by the time conjunct 8 runs, every key's
children are already in the closed set `receive_path` supports — the
`if let (Some,Some)` is provably always-`Some` at that point, not a live gap.
Read both functions to confirm this ordering and the closed-set match
(`use_site_ok`'s five arms line up 1:1 with `receive_path`'s five arms).

Ran the full Rust suite: `cargo nextest run --locked` → **646/646 passed, 2
skipped**, `cargo fmt --check` clean.

**However — see "Rust and Go now disagree" below**, a new cross-language
finding from hunting exactly this question.

### I-1 — CLOSED

The warning block in `gui/gui.go`'s `Draw` now renders **before** Title/Type/
Script, unconditionally on `reused`. Mutation-tested: moved the block back to
its old (post-Script) position — `TestTheDescriptorWarningIsDrawnInFull`
(2 of 4 subtests) and `TestTheTitleCannotPushTheWarningOffTheScreen` both went
red with the exact "text the operator cannot reach" failure the original I-1
described. Reverted; suite green again.

### I-2 — CLOSED for its stated claim, but see a new finding below

The fit-gate now pads with `wideWords` (space-separated 8-char tokens, word-
wrapping) instead of an unbroken run of `x`. Confirmed the two tests that
depend on the warning-first ordering (`TestTheDescriptorWarningIsDrawnInFull`,
`TestTheTitleCannotPushTheWarningOffTheScreen`) pass and correctly go red
under the I-1 regression above — that part of I-2's intent (a realistic-glyph
gate around the funds warning) is subsumed by C-1/I-1's structural fix and is
sound.

The **cap itself** (`maxTitleDrawn = 48`, applied via `truncateForDisplay` in
`Draw`) is a separate, narrower claim ("Type and Script are worth protecting
too") pinned by `TestTheDisplayedTitleIsBounded`. That test does **not**
actually verify the cap — see the new finding below.

### M-1 — CLOSED

`address/derives_same_key_test.go`'s `TestDerivesSameKeyIgnoresOriginMetadata`
pins the exclusion. Mutation-tested: reinstated
`a.MasterFingerprint != b.MasterFingerprint || !slices.Equal(a.DerivationPath, b.DerivationPath)`
inside `DerivesSameKey` — the test failed with exactly the message the fold
wrote ("two keys differing only in origin metadata are reported as
different"). Reverted.

### M-2 — CLOSED

`BenchmarkAllocs` now builds a `dupDesc` (5 keys, `Keys[1] = Keys[0]`) with a
self-check (`b.Fatal` if `descriptorRepeatsAKey` doesn't call it a duplicate)
and adds `dupDS.Confirm` to the benchmarked screens. Mutation-tested: added a
heap-escaping allocation (`package-level slice assignment`, not a
compiler-eliminated dead store — a first attempt using `_ = make(...)` was
silently optimised away and produced a false green, which is itself a caution
about how to mutation-test Go alloc counters) inside `DerivesSameKey` —
`TestAllocs` correctly failed ("got 11 allocs, expected 0"). Reverted;
`TestAllocs` green again at 0 allocs/op.

### N-1 — CLOSED

`composerCopyNoAddressesDuplicateKeys`'s doc comment is back above its own
function, separated by a blank line from `composerCopyDescriptorRepeatsAKey`'s
comment. Read the current file: both functions carry their own, distinct doc
blocks.

### N-2 — CLOSED

`gui/address_polish.go`'s precondition comment now reads "The caller opens
this only when `address.Supported(desc)` AND the descriptor does not reuse a
key (`gui/gui.go`'s `supported && !reusesAKey`, F-530)" — matches the actual
gate at `gui/gui.go`'s `Confirm`.

---

## New defects found

### NEW-1 (Important) — Rust and Go disagree on a change-chain-only key collision

Reproduced directly against both implementations (not merely traced):

- **Rust**: added a temporary unit test inside
  `crates/me-cli/src/descriptor/admit.rs`'s existing
  `conjunct_reachability` test module, building a 2-of-3 `Parsed` with keys
  `A/3/*`, `A/<2;3>/*` (same xpub/chaincode as `A/3/*`), and a distinct third
  key. `admit(&d, Path::Descriptor)` → **`Ok(())`** (admitted). Removed after.
- **Go**: added a temporary test in `sysw/` calling `admitDescriptor` on
  `wsh(sortedmulti(2, A/3/*, A/<2;3>/*, B))` with the equivalent xpubs.
  `admitDescriptor(...)` → **`false`** (refused). Removed after.

Why they diverge: `A/3/*` is a fixed (non-multipath) child — `derivePubKey`
never consults `change` for a plain `Child`, so it derives the **same** key on
both chains. `A/<2;3>/*` is multipath: receive uses `2`, change uses `3`. The
two keys collide **only on the change chain** (both resolve to child `3`
there). Rust's conjunct 8(b) now compares `receive_path(a) == receive_path(b)`
— receive-chain only, by design ("the first address anyone funds") — so it
does not see this collision. Go's `address.DerivesSameKey` deliberately
checks **either** chain ("a collision there is the whole harm even when
change differs") — so it does.

This is a real behavioral difference the fold introduced: before the fold,
both languages used the same (broken) spelling comparison and so agreed
(wrongly, in the same direction) on every input. After the fold, they encode
two different, both individually well-reasoned, but **mutually inconsistent**
strictness rules for conjunct 8(b).

**Does it matter?** Traced the call graph: `sysw.Classify` (which calls
`admitDescriptor`) is what the **device** runs when it opens any payload
record (`gui/sysw_session.go:114`), independent of whatever the host `me`
tool decided at pack time. So a record the host admits but the device would
refuse simply fails `Classify`'s `ClassDescriptor` arm on the device — the
device fails **closed** (refuses to recognize the record), not open. No path
was found where the device *trusts* the host's admission decision without
re-deriving it. So this is not a funds-safety hole: no wrong address is ever
computed or shown. It **is** a real defect against the codebase's own stated
guarantee — `sysw/descriptor.go`'s own comment calls Rust/Go parity here
"MANDATORY rather than desirable," enforced by asserting both languages'
admission against the same shared vector file
(`nonstandard/testdata/descriptor_seam_vectors.json`,
`crates/me-cli/testdata/descriptor_seam_vectors.json`, byte-identical,
confirmed by SHA256). That file has **no row** shaped like this (a fixed
child colliding with a multipath key only on change), so this divergence is
invisible to the seam-vector parity gate in either direction: a future change
that widens the gap further (or narrows it inconsistently) would ship green.
Recommend a follow-up vector row exercising a change-chain-only collision, or
an explicit decision (recorded, not just implied by two independent doc
comments) on whether conjunct 8(b) should be receive-only or either-chain,
applied identically on both sides.

### NEW-2 (Important) — `TestTheDisplayedTitleIsBounded`'s main assertion cannot fail; the display cap has no working regression test

Measured, not inferred: with the `truncateForDisplay` call site in
`gui/gui.go`'s `Draw` **removed entirely** (Title drawn unbounded) — a direct
regression of I-1's "belt" — `TestTheDisplayedTitleIsBounded` still **passed**.
Also tried raising `maxTitleDrawn` from 48 to 500 (i.e., effectively
disabling the cap for any realistic or even the test's own 400-character
probe title): every test in the package, including
`TestTheDisplayedTitleIsBounded`, still passed. Reverted both.

Root cause, measured directly: the test's only Draw-integration assertion
checks whether the **full** 400-character probe title appears verbatim in the
extracted frame text, using `descDistinctKeys` (no duplicate-key warning, so
Title is drawn first). But the screen's own natural viewport clipping already
hides any title past roughly 200–300 characters, with or without the cap —
measured by sweeping title length from 20 to 400 with the cap removed
entirely: full title still visible at n=20..200, first goes invisible at
n=300 (`drawn_len` plateaus at 230 runes from n=300 onward). Since the test
only probes at n=400, the assertion is true regardless of what `maxTitleDrawn`
is set to (0, 48, 500, or removed), because natural clipping alone already
guarantees "not the full 400 chars" at that length. The two direct unit calls
to `truncateForDisplay(...)` in the same test do correctly pin that function
in isolation, but nothing in the suite pins the **call site** — that `Draw`
actually invokes `truncateForDisplay(desc.Title, maxTitleDrawn)` rather than
using `desc.Title` raw, or invokes it with the right constant.

Impact: this is the "belt" for Type/Script visibility, not the funds warning
itself (which I-1's ordering fix protects unconditionally, independent of any
title length — confirmed above). So no funds-relevant regression follows
today. But it is a test that reads as verifying the cap and does not, which
is exactly the "new test that cannot fail" class the review brief asked to
hunt for. A regression test that actually catches call-site removal would
need a title length inside the natural-clip window (e.g. 100–150 chars) rather
than 400.

### NEW-3 (Nit) — stale comment on `TAG_SLOTS` in the Rust seam-vector gate

`crates/me-cli/tests/descriptor_seam.rs`: the doc comment
`/// The minima sum to 89 tag-slots.` sits directly above
`const TAG_SLOTS: usize = 90;` (bumped from 89 by this fold). `MANIFEST`'s
`("gate", 37)` entry was **not** bumped to 38 even though the actual file now
carries 38 rows tagged `gate` in `covers` (recomputed directly from the JSON:
`covers` tag counts are `gate=38, promotion-near-miss=15, narrowed-4.7=14,
md1-splits=6, narrowed-4.2=5, formats-happy=4, whitespace=3, neither=3,
accepted-extreme=1, version-gap=1`, summing to 90 — matching the new
`TAG_SLOTS`). `MANIFEST`'s stated minima literally still sum to 89 (unchanged
values), so the comment is factually true about `MANIFEST` as written, but no
longer describes what `TAG_SLOTS` equals, and breaks the pattern every other
tag in the table follows (stated minimum == actual count). Not gating: the
`n >= *min` check still holds (38 ≥ 37) and `assert_eq!(slots, TAG_SLOTS)`
still holds (both computed from the file, 90 == 90). Purely a documentation/
bookkeeping drift; recommend bumping `("gate", 37)` to `("gate", 38)` and the
comment to "90" the next time this file is touched.

---

## Machine-checks run (this pass, beyond the brief's "already verified" list)

- `go build ./...`, `go vet ./...` (fork): clean, only the known
  `testing.ArtifactDir requires go1.26` TinyGo-vs-toolchain gap (10 lines,
  pre-existing).
- `gofmt -l .` (fork): exactly the known 5-file baseline
  (`gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) — none of
  the fold's files.
- `cargo nextest run --locked` (primary): 646/646 passed, 2 skipped.
  `cargo fmt --check`: clean.
- `go test ./sysw/... ./address/...`: pass.
- Recomputed every bumped seam-vector population count directly from
  `crates/me-cli/testdata/descriptor_seam_vectors.json` (byte-identical to
  the fork's copy, SHA256 `6352aa45...` matching the pinned constant in both
  languages): `rows=73`, `device_admits_true=39`, `device_admits_false=34`
  (unchanged), `host_admits_true=19` (unchanged), rows with `gate_open`
  present `=38`, rows with `refusal_row` present `=19`, single-line rows
  `=60` — all match the fold's new constants in both
  `crates/me-cli/tests/descriptor_seam.rs` and
  `nonstandard/descriptor_seam_test.go`.
- Both trees confirmed clean (`git status --porcelain` empty) after every
  experiment.

---

## Counts

**0 Critical / 2 Important / 0 Minor / 1 Nit**

(NEW-1: Rust/Go admission divergence on a change-chain-only collision — fails
closed, no funds impact, but breaks a stated MANDATORY parity guarantee with
no vector coverage. NEW-2: a new test that cannot fail, leaving the Title
display cap without a working regression test. NEW-3: a stale comment/
off-by-one MANIFEST entry with no test impact.)

## Verdict

**NOT GREEN** (2 Important open). Every finding in the F-530 review (C-1, C-2,
I-1, I-2, M-1, M-2, N-1, N-2) is genuinely closed — reproduced independently
via property testing, mutation testing, and direct execution rather than
reading the diff. The fold is correct and well-tested on the question it set
out to answer. But hunting the specific questions the brief posed (Rust/Go
agreement; new tests that cannot fail) found two Important-severity gaps the
fold introduced: a real cross-language admission disagreement on
change-chain-only collisions (fails safe, but unpinned), and a display-cap
regression test that passes regardless of whether the cap exists.
