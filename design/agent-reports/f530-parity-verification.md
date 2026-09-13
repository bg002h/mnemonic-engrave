# F-530 fold parity verification — NEW-1 / NEW-2, round 3

Scope: `mnemonic-engrave` `git diff e85bb2d6..3fbc4a02`, `seedhammer`
`git diff fcf8ac7..0562e81`. Per brief, C-1/C-2/I-1/I-2/M-1/M-2/N-1/N-2 were
NOT re-examined (verified closed in `design/agent-reports/f530-fold-verification.md`,
committed `e85bb2d6`).

## NEW-1 — CLOSED

Claim: Rust's `derive::derives_same_key` (`crates/me-cli/src/descriptor/derive.rs`)
now asks the identical either-chain question as Go's `address.DerivesSameKey`
(`seedhammer/address/address.go`), which the fork's diff confirms was
*unchanged* by this fold (`git diff fcf8ac7..0562e81 -- address/` is empty) —
Go already had the either-chain rule; the fold widened Rust to match it.

**Property-style cross-language check.** Built a temporary (reverted) harness
in each language: 18 named children shapes (`empty`, `/*`, `/0/*`, `/0h/*`,
`/<0;1>/*`, `/<2;3>/*`, `/3/*`, `/<0;5>/*`, `/<0;1>`, `/1/*`, `/<1;2>/*`,
`/0/<0;1>/*`, `/<0;1>/<2;3>/*`, `/*/*`, `/5`, `/3h/*`, `/<2;3>`, `/*h`), all
18×18 = 324 ordered pairs, same key identity on both sides.

- Rust `derives_same_key` vs. an independent ground-truth deriver (actual
  CKDpub via `bitcoin::bip32::Xpub`, walking each `Derivation` element,
  applying the same `DEFAULT_CHILDREN` normalisation for empty children,
  checked over indices 0..6 on both chains): **0 mismatches / 324 pairs**.
  (First harness draft omitted the empty→`<0;1>/*` normalisation in the
  ground-truth deriver itself — an 8-pair self-inflicted false mismatch,
  fixed before drawing any conclusion; not a production defect.)
- Go `DerivesSameKey` vs. the same style of ground truth (`derivePubKey` over
  indices 0..6, both chains): **0 mismatches / 324 pairs**.
- Rust's 324 outputs vs. Go's 324 outputs, same shape-pair ordering: **byte-
  identical, 0 divergences** (`diff` on sorted `name_a\tname_b\tresult`
  matrices was empty).
- The exact NEW-1 shape (`/3/*` vs `/<2;3>/*`) returns `true` (collides) in
  both languages and matches ground truth (collides on the change chain: id 3
  vs. `end=3` when `change`).

**End-to-end, not just unit-level.** `cargo test -p mnemonic-engrave --test
descriptor_seam the_host_column_matches_the_admission_predicate` (all 74 rows,
including the new one) and `the_gate_rows_pin_the_real_invocation` (spawns the
real `me` binary) both pass — re-run live, not read off the "already run"
list. Go: `go test ./sysw/ -run TestAdmissionRefusesEverySpellingOfADuplicateKey`
passes all 6 subtests including `collides_on_the_change_chain_only`, through
the real `nonstandard.OutputDescriptor` → `keyIdentityOK` →
`address.DerivesSameKey` path.

**False-positive check (widening didn't wrongly refuse anything legal).**
`the_host_column_matches_the_admission_predicate` asserts every row's
`host_admits` against the actual admission predicate for all 74 rows — reran
it live, green. No row's `host_admits` needed to change and silently didn't.

**`device_admits: true` on the new vector is correct, not a discrepancy.**
Confirmed by reading `nonstandard/descriptor_seam_test.go`:
`TestDescriptorSeamDeviceColumn` asserts `device_admits` against
`nonstandard.OutputDescriptor` only — the syntax/scan-door layer, not the
funds-safety identity gate (`sysw.keyIdentityOK`), which is checked
separately. The new row's `(host_admits=false, device_admits=true)` pattern
is byte-identical to the two prior `key-identity-duplicate` rows
(`duplicate-key-same-use-site`, `duplicate-key-implicit-use-site`) — checked
directly from the JSON.

**Comments.** Re-read the rewritten blocks in `admit.rs` and `derive.rs`;
both match verified behavior. `sysw/descriptor.go`'s disjoint-multipath
comment (`<0;1>/*` vs `<2;3>/*` being a legal two-chain wallet) was not
touched, describes an unrelated shape (both ranges, no fixed child), and is
still accurate — not falsified by this fold.

**Verdict: NEW-1 CLOSED.** No divergence found in 324 pairs, ground truth
agrees with both implementations independently, and both agree with each
other byte-for-byte.

## NEW-2 — CLOSED

Re-ran the two named disablings against `gui/descriptor_duplicate_keys.go` /
`gui/gui.go`, both reverted after:

1. **Delete the `truncateForDisplay` call site** (replace with bare
   `desc.Title` at `gui/gui.go:3351`): `go test ./gui/ -run
   TestTheDisplayedTitleIsBounded` → **FAIL** (both new assertions fire: the
   120-char title is drawn whole, and no `"..."` marker is present).
2. **Raise `maxTitleDrawn` from 48 to 480**
   (`gui/descriptor_duplicate_keys.go:65`): → **FAIL** (the two `t.Errorf`
   checks fire first, then the test panics on `[]rune(desc.Title)[:480]` with
   `desc.Title` only 120 runes — a hard crash, still a red result, not a
   silent pass).

Baseline (unmutated) test passes; tree restored and diffed clean after each
mutation. **Verdict: NEW-2 CLOSED** — the test cannot pass under either
disabling that broke it before.

## New defects introduced by this fold

**None found.** Checked:
- Six bumped POP/manifest counts, recomputed directly from
  `crates/me-cli/testdata/descriptor_seam_vectors.json` by field presence per
  the test's own `count`/`truthy` semantics (not guessed): `rows=74`,
  `device_admits_true=40`, `gate_fields=39` (from `gate_open` presence),
  `refusal_row=20`, `TAG_SLOTS=91` (sum of all `covers` tag counts, including
  `gate=39`), `SINGLE_LINE_ROWS=61`. All six match the committed constants
  exactly; `SINGLE_LINE_ADMITTED=15` (unchanged) also recomputed and matches.
- Both copies of the vector file are byte-identical
  (`diff` empty) and both hash to `ed3706e2…60f4`, matching both languages'
  pinned `SEAM_VECTORS_SHA256`/`seamVectorsSHA256` constants exactly (not
  just "looks similar").
- `NEW-3` (the `TAG_SLOTS` stale-comment Nit from the prior round's report)
  was incidentally fixed by this same fold — `MANIFEST`'s `("gate", 37)` is
  now `("gate", 39)` and the comment states "every entry states the ACTUAL
  count." Recomputation above confirms `gate=39` is in fact the real count.
  This was a Nit before and remains non-gating; noted only because its
  resolution is visible in this diff and the brief's severity table doesn't
  need it re-litigated.

## Build gate (already run, reconfirmed selectively)

Re-ran (not merely trusted) two targeted, high-value checks live during this
review: `cargo test -p mnemonic-engrave --test descriptor_seam` → 19/19 pass;
`go test ./sysw/ -run TestAdmissionRefusesEverySpellingOfADuplicateKey` → 6/6
pass. Did not re-run the full 646/646 nextest / 1324/1324 gui-shard suite —
per brief, already run and not to be re-run to confirm.

## GREEN — 0 Critical / 0 Important

NEW-1: CLOSED. NEW-2: CLOSED. No new defect found. Both worktrees left clean
(scratch test files and experimental mutations all reverted; `git status
--porcelain` empty in both repos at end of review).
