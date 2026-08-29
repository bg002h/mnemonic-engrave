# IMPL-P0-review — sonnet fold-vs-manifest pass over P0 of `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`

**Reviewer:** sonnet, per the plan's P0-review clause ("a sonnet fold-vs-manifest
pass, counts by machine, NOT a fresh audit").
**Reviewed:** `mnemonic-engrave` branch `impl/descriptor-s1s3` @ `e0d3d65`;
`seedhammer` fork worktree `/scratch/code/shibboleth/_work/seam-fork` branch
`seam/descriptor-vectors` @ `1f09537`. Implementer's own report:
`design/agent-reports/IMPL-P0-report.md` (on the branch, at `e0d3d65`).
**Method:** read-only against both branches; all verification run in a scratch
worktree (`/scratch/code/shibboleth/_work/review-p0-engrave`, removed after use)
and the pre-existing fork worktree. Neither branch was modified. One brief,
intentional mutation was made and reverted inside the scratch worktree only
(see "Mutation sanity check" below); `git status --short` confirmed clean
before removal.

**VERDICT: P0 CLOSES. Proceed to P1.** Every claim checked reproduced exactly;
no discrepancy found between the report and the machine-checked artifacts.

---

## Environment note (out of scope, not a finding against this work)

The main `mnemonic-engrave` checkout was found mid-session on branch
`impl/descriptor-s1s3` (clean, exactly at `e0d3d65`) rather than `master`,
and carried an untracked, gitignored 252 MB leftover
(`scripts/descriptor-seam-vectors/{rsprobe/target,__pycache__}`) from a prior
checkout. Restored the checkout to `master` before writing this report, as
instructed. Also present: two unrelated experiment branches/worktrees
(`exp/tx-plan-driven`, `exp/tx-brief-driven`) and a worktree/branch
`review/p0-prepublish` (head `6ea4e66`) carrying a divergent, self-declared
"P0 IMPLEMENTED... SAFE TO PUBLISH" history that deletes ~31k lines including
`design/agent-reports/*` and `scripts/plan-staleness-check.sh`. This branch is
**not part of this review's scope**, was not read as authoritative for
anything, and its "SAFE TO PUBLISH" claim was not relied upon or acted on.
Flagging its existence only; it appears unrelated to the P0 work under review.

---

## (a) Manifest arithmetic vs spec §7 — RAN, all exact

Computed independently in Python directly against
`crates/me-cli/testdata/descriptor_seam_vectors.json` (not read from the
report):

| check | spec §7 requirement | measured |
| --- | --- | --- |
| physical rows | ≥ 71 | **71** |
| tags | 9 | **9**, no unknown tags |
| per-tag minima | see table below | all met **exactly** |
| tag-slots | 88 | **88** |
| overlap distribution | 15 rows ×2 tags, 2 of those ×3 tags | histogram `{1:56, 2:13, 3:2}` → 13+2=**15** rows ≥2 tags, **2** rows =3 tags — exact |
| arithmetic | 88 − 17 = 71 | confirmed |
| `covers` distinct within row | required | **0** violations |
| `md1_admits` present, boolean, on every row | required | **0** missing/non-bool |
| `covers` present on every row | required | **0** missing |
| whole-file sha256 | pinned literal | **matches** both pinned literals (see (d)) |
| per-row sha256 of `input` | required | **0** mismatches (all 71 recomputed) |
| gate fields (`gate_open`/`outcome`/`refusal_row`/`exit_code`) present iff `gate`-tagged | required | **0** violations |
| `gate_open == (outcome != "record-refusal")` | invariant | **0** violations |
| `refusal_row` present iff `outcome ∈ {descriptor-refusal, multi-record}` | required | **0** violations |
| `refusal_rows` map size / all used slugs defined | — | **36** entries, **0** undefined references |
| requirement 5 non-vacuity | ≥1 both, ≥1 device-only, ≥1 neither | both=16, device-only=21, neither=30, host-only(whitespace)=3 |

Per-tag table (all exact matches, no over/under):

| tag | min | got |
|---|---|---|
| formats-happy | 4 | 4 |
| promotion-near-miss | 15 | 15 |
| narrowed-4.7 | 14 | 14 |
| accepted-extreme | 1 | 1 |
| narrowed-4.2 | 5 | 5 |
| neither | 3 | 3 |
| whitespace | 3 | 3 |
| md1-splits | 6 | 6 |
| gate | 37 | 37 |

Gate sub-distribution (outcome × exit_code, 37 gate rows), computed
independently: `as-decides/2 → 7`, `descriptor-refusal/3 → 17`,
`multi-record/4 → 1`, `record-refusal/4 → 12`. Matches the report exactly.

## (b) `cargo nextest run --locked` on the branch — RAN

Full workspace, in the scratch worktree at `e0d3d65`:

```
Summary [ 32.134s] 446 tests run: 446 passed, 7 skipped
```

Matches the report exactly. `cargo clippy --all-targets -- -D warnings` and
`cargo fmt --check` both exit clean (verified, not read).

**Identity of the 7 skips, verified by grep, not by claim:**
`grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs` → **6**, split
2×`P1:` / 4×`P2:` (verified by reading each `#[ignore = "..."]` string — 2 name
P1, 4 name P2, none vague). The 7th is
`crates/me-cli/src/sysw/vectors.rs:132` (`"regenerates the fixture; run
deliberately"`, pre-existing). `grep -rn "#\[ignore" --include="*.rs" crates/`
across the whole workspace returns exactly these 7 — no ignore exists outside
this accounted-for set.

## (c) Fork `go test ./nonstandard/` — RAN, green

At `1f09537`, `go1.26.4`:

```
--- PASS: TestDescriptors / TestDecoder / TestElectrumSeed (pre-existing baseline)
--- PASS: TestDescriptorSeamDeviceColumn
--- PASS: TestDescriptorSeamInvariant
--- PASS: TestDescriptorSeamAddresses
--- PASS: TestDescriptorSeamWalletID
--- SKIP: TestDescriptorSeamSyswClass (named reason: S2/F-418)
ok  	seedhammer.com/nonstandard	0.023s
```

`go vet ./nonstandard/` and `gofmt -l nonstandard/` both silent/clean.
Matches the report exactly.

## (d) Two vendored files byte-identical — RAN

```
sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json
  = 0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584
sha256sum .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
  = 0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584
cmp <both>  → byte-identical (no output, exit 0)
```

Same 64-hex-char literal confirmed present in both
`crates/me-cli/tests/descriptor_seam.rs:45` and
`nonstandard/descriptor_seam_test.go:40`.

## (e) Spot re-measurement with the debug `md` binary — RAN, 5/5 reproduced

Built `md-cli` at the pinned baseline `descriptor-mnemonic` `6864f377`
(already the checked-out HEAD there — verified via `git rev-parse HEAD`
before building). Reconstructed each row's template from its `input`/spec
description and re-derived independently (not by copying the pinned value
into the command):

| row | field(s) | pinned | re-measured | match |
|---|---|---|---|---|
| `formats-happy/bluewallet-sh-fixture` (wallet_id row) | wallet_id | `a67e07d16b2500fde6c557a76c7390f6` | `a67e07d16b2500fde6c557a76c7390f6` | ✓ |
| same | address_0 | `bc1qtahtpjkgtljxl20jgevs2tjhgzvd87jepcrsd92kcyvtzkj34mnsq0j928` | same | ✓ |
| same | address_1 | `bc1qnww8rjenwn24psu5h6exrhqpgkc6t5y27cusa6wylr4khuscl86qevfpx2` | same | ✓ |
| `md1-split/childless` (address row) | wallet_id | `47ecf2de11530f266e9b08640734447a` | same, **computed over the (a′)-materialised `<0;1>/*` policy**, not the literal childless template (confirms plan's stated method — literal-childless template gives a different, wrong id) | ✓ |
| same | address_0 | `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a` | same | ✓ |
| `gate/duplicate-key-same-use-site` (gate row) | `host_admits=false`, refusal cause | claimed cause: same `(xpub, use-site)` in two slots | Reproduced independently against the **primary Rust validator** (`md encode` with the identical key in both `@0`/`@1` slots): refused with *"@0 and @1 carry the same key at the same use-site: this policy names 2 cosigners but one of them holds two of the seats"* | ✓ (grounds the claim; see note below) |
| `gate/colliding-origin-sortedmulti` (clause-8 row) | `host_admits=false`, refusal cause | claimed cause: one `(fingerprint, origin)` naming two different xpubs | Reproduced independently (same fingerprint on two different xpubs): refused with *"@0 and @1 declare the same key origin (...) but different xpubs; ... this card describes a wallet that cannot exist"*; a clean control with distinct fingerprints for the same two keys encodes without error | ✓ |
| `whitespace/crlf-bip380` (whitespace row) | address_0 | `bc1q4taqq6q6l8fvguva6ftvrz3qgdjy6p3w2s0ds0nl6qrjw7t0hfhqgrqcwd` | same | ✓ |
| same | wallet_id (bonus — not a required field on this row, present only via the childless-shared derivation used for the check) | — | `9e95257e60aacbb260129dac7b36d9f4`, self-consistent with the row's canonical | — |

**Note on the two gate/clause-8 rows:** `me`'s own cascade + conjunct-8
admission predicate is P1-scope and does not exist yet in P0 (correctly
ignore-tagged), so there is no `me` binary to invoke against these rows
today, and the gate fields on non-clause-8 gate rows (malformed bech32
strings, hostile record payloads, buried keys, edge tokens) have no
`md`-testable content at all — they are `me`-only surface. What **is**
independently checkable today is whether the *real-world claim* the two
clause-8-family gate rows rest on is true, using `descriptor-mnemonic`'s
already-shipped validators (`validate_origin_key_consistency` /
`validate_no_duplicate_key_slots`, the two calls the plan's Load-bearing
Anchors section cites at `encode.rs:118`/`:120`). Both refused exactly as
described, on freshly-constructed inputs (not the pinned descriptor strings
copy-pasted verbatim, but reconstructed from the same fingerprints/xpubs).
This grounds F-2/clause-8's "cannot exist" framing in a running validator
rather than prose.

## (f) Per-column count manifests agree across the two suites — RAN

Computed all 16 of the Rust `POP` struct's fields and all 12 of the Go
`want*` constants **independently from the JSON**, not read out of either
test file's literals, then diffed against both files' pinned constants:

**16/16 Rust `POP` fields exact.** **12/12 Go `want*` constants exact**,
including the derived `wantDeviceAddr0=16` / `wantDeviceAddr1=4` (re-derived
by filtering `address_0`/`address_1` rows to `device_admits=True`, landing on
exactly 16 and 4, and the excluded 4 rows for `address_0` are exactly the 3
`whitespace`-tagged rows plus `neither/wsh-multi`, and the excluded 1 for
`address_1` is `neither/wsh-multi` alone — matches the report's stated
reasoning row-for-row, not just the totals).

## Mutation sanity check (not the full 15/15 — one, to confirm the gate is live)

Dropped one required row (`narrowed/tr-sortedmulti`) from a scratch-worktree
copy of the vector file. With the sha256 pin left untouched: all 6 tests fail
immediately on the pin mismatch (expected — pin-first is the design). Re-ran
after re-pinning the sha256 to the mutated file's own digest (isolating the
manifest-arithmetic assertions from the pin check, as the report's mutation
methodology describes): `every_column_has_the_expected_population` failed on
`rows: left=70, right=71`; `the_coverage_manifest_is_met_by_count_not_by_reading`
and `the_file_is_the_one_the_fork_pins` also failed (the latter correctly,
since only one copy was mutated). Reverted both files
(`git checkout -- <files>`); re-ran clean: `6 passed, 6 skipped`. Confirms the
manifest gate is live, not vacuous, independent of the sha256 pin.

---

## Findings check — item 3

All three named items, plus the fourth (cross-language `wallet_id`
agreement), verified **on disk**, not taken from the report's prose:

- **F-1 (`_comment` strict-reading decision):** present verbatim in the
  file's `_comment` array — `format` reads STRICTLY as "the branch that
  SUCCEEDED", with the full rationale and resulting population breakdown.
  Confirmed present.
- **F-2 (dropped 16-key `wallet_id`):** `accepted/sh-wsh-sortedmulti-16-keys`
  carries `md1_admits: true` and **no** `wallet_id` key — confirmed absent.
  The Rust harness structurally enforces the class of defect this guards
  against: `every_column_has_the_expected_population` (line ~545) asserts,
  for every row carrying `wallet_id`, that `md1_admits == true AND
  device_admits == true` — i.e. no row can carry a `wallet_id` only one side
  can compute. This assertion runs today (not ignore-tagged) and passed.
- **F-3 (ignore-gate anchor):** the module doc states the anchored command
  `grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs` (note the
  `.` wildcard instead of a literal `[`, which keeps the doc's own `//! grep
  -c '^#.ignore' ...` line — which does not start with `#` — from
  self-matching). Ran it: **6**, matching "measured now: 6" in the report.
- **F-4 (no F-212-class divergence):** exactly 4 rows carry `wallet_id`
  (independently counted, matches). Re-measured 2 of the 4 with the debug
  `md` binary in (e) above and both reproduced exactly, corroborating the
  report's 3-way (md-cli / published md-codec 0.42 / fork Go) agreement
  table without relying on it.

**Commit discipline:** `git log --format='%H %s' master..e0d3d65` shows
exactly 3 P0 work commits (`4165532` P0.1, `dbe075c` P0.2, `2990917` P0.2
ignore-gate fix) followed by 1 report-only commit (`e0d3d65`, touches only
`design/agent-reports/IMPL-P0-report.md`, 454 insertions, 0 other files —
verified via `git show --stat`). `git diff --diff-filter=MDR --name-status`
against both baselines (`master` for `mnemonic-engrave`, `d402f18` for the
fork) is **empty** in each — P0 modified no pre-existing file in either repo.
`mnemonic-engrave` adds exactly 16 files / 3,530 lines through `2990917`
(recount, not reread); the fork adds exactly 2 files / 1,685 lines through
`1f09537`. Both exact matches to the report.

`rsprobe`'s own `[workspace]` table was confirmed to exclude it from the main
workspace: `cargo metadata --no-deps` lists only `mnemonic-engrave` and
`mnemonic-io-lib` as members.

---

## Verdict

**P0 CLOSES. Proceed to P1.**

No findings. Every machine-checkable claim in the implementer's report — the
manifest arithmetic, both test suites' pass/skip counts and identities, the
byte-identity of the vendored file, five independently reconstructed
row-level values (wallet_id ×2, addresses ×4, two refusal causes grounded
against the primary Rust validator), all 28 pinned per-column counts across
both languages, the commit-message discipline, and the location of all four
named findings — reproduced exactly under independent recomputation, with
zero discrepancies. One additional mutation check (beyond what was asked)
confirmed the manifest gate fails on a real defect rather than passing
vacuously.
