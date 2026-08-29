# IMPL-S1S3 — mechanical verification of fold 2

**Scope:** `git diff 171ec42..e56ae1b` (4 commits) on `impl/descriptor-s1s3`,
worktree `/scratch/code/shibboleth/_work/impl-s1s3`, binary
`target/debug/me` built from `e56ae1b`. Fork worktree
`/scratch/code/shibboleth/_work/seam-fork` at `a5e29b4`. One question: **did
fold 2 fix each finding of `IMPL-S1S3-fold1-rereview.md` (I-A, M-A, N-a, N-b;
N-c skipped by controller instruction) as claimed in
`IMPL-S1S3-fold2.md`, without breaking anything?** Dispositions were
re-derived against the artifacts, not inherited from either report.

**Counts: 0 Critical / 0 Important / 1 Minor / 0 Nit.**

**Verdict: GREEN — 0C/0I. All four claimed fixes verified true by execution;
one Minor citation defect found and not previously flagged.**

---

## 1. I-A — verified FIXED, with one Minor residual

Both amended spec sites read as claimed:

* `design/SPEC_descriptor_input.md:1663-1690` (§7 clause 8) now states the
  shipped order — conjunct 1's `--as`-independent shape half first, then
  conjuncts 2–8, then conjunct 1's `multi`-under-`--as descriptor` arm last —
  and carries the `AMENDED 2026-08-29 (IMPL-S1S3 adversarial review, C1)`
  block.
* `design/SPEC_descriptor_input.md:1205-1211` (§5.4's parenthetical) now
  splits the conjunct-8-PASSING and conjunct-8-FAILING `multi` explicitly,
  each meeting a different follower.
* The status header (`design/SPEC_descriptor_input.md:29-33`) carries
  `**Post-GREEN amendment 2026-08-29 (b):**` naming both sites.

**Executed the rule.** Built `me` at `e56ae1b` and ran the
`gate/colliding-origin-multi` row's own input:

```
$ me sysw pack --no-passphrase --in collide-multi.txt --as descriptor   # rc=3
me: this wallet description contradicts itself: keys 0 and 1 both claim
    origin `dc567276/48h/0h/0h/2h` but name different keys ...
$ me sysw pack --no-passphrase --in collide-multi.txt --as md1          # rc=3
me: (identical text)
```

Both paths give conjunct 8's key-identity refusal — matches the amended
spec sites, and matches the vector row's `refusal_row: key-identity`.

**Control re-verified.** A sound `multi` (different origins) still gets
conjunct 1's permanent refusal under `--as descriptor` (rc=3, after the full
FULL-tier block, `wallet-id: 0501609a…`) and packs at rc=0 under `--as md1`
— the referral is still true by construction. An anyone-can-spend
`multi(0,…)` still gets conjunct 2's own refusal ("treat them as at risk
now") under `--as descriptor`, not conjunct 1's referral — the ordering fix
from fold 1 was not disturbed by fold 2.

**Vector `source` fields.** Both corrected rows read as fold-2 claims.
`gate/deadbeef-fronts-an-xpub`'s claim that the fork fails at
`parseBlueWalletDescriptor` with `bluewallet: expected 0 keys, but got 1`
(not the `Title != ""` gate) was reproduced by running
`nonstandard.OutputDescriptor` and `parseBlueWalletDescriptor` directly
against the fork at `a5e29b4` (Go 1.26.3): confirmed, `err = "bluewallet:
expected 0 keys, but got 1"`.

**Minor — the line-number citation `parse.go:151` is wrong.** Both the
vector's `source` field and `IMPL-S1S3-fold2.md` §2.3 state the count check
"fires at `parse.go:151`". `git blame -L 155,160 nonstandard/parse.go`
(both at fork `1f09537` and `a5e29b4`) shows the `fmt.Errorf("bluewallet:
expected %d keys, but got %d", …)` line is **158**, not 151 — 7 lines off,
same function. The claim is stated as "measured at fork 1f09537", but the
number was carried forward unverified from `IMPL-P1-review.md:269` and
`IMPL-P1-report.md:307` (`grep` shows the same wrong `:151` in both earlier
reports), not re-derived in this fold. The substance is correct (which
error fires, that it is not the `Title` gate) and `source` is confirmed
annotation-only — no test in either harness reads it (`grep -n '"source"'
crates/me-cli/tests/descriptor_seam.rs` → lines 81, 232 only, both list/
presence checks; `grep -i source
.../seam-fork/nonstandard/descriptor_seam_test.go` → 0 hits) — so nothing
gates on it and no behavior is affected. Recorded as Minor because it is a
citation asserted as measured that was not, now baked into a two-repository
pinned artifact.

## 2. Generator fidelity — CONFIRMED

`rows.py`'s `ROWS` list is a pure module-level import (no probe toolchain
needed to read `source`). Compared all 71 rows' `source` field against the
committed `descriptor_seam_vectors.json`:

```
rows compared: 71   source strings differ: 0   missing: 0
```

Matches the implementer's claim exactly. (Full `gen.py` end-to-end was not
re-run — the implementer disclosed this in fold2.md §8 item 2, and it is out
of this verification's scope; the authored half, where both defects were, is
what was checked.)

## 3. Lockstep — CONFIRMED

```
542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974  crates/me-cli/testdata/descriptor_seam_vectors.json
542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974  .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
```

`diff` of the two files: identical. The vector-file diff between `171ec42`
and `e56ae1b` touches exactly the two `source` strings and nothing else.
`grep -rn 0393592f` (the superseded sha) over both worktrees (excluding
`.git`/`target`): hits only inside `design/agent-reports/` in the engrave
repo; 0 in any active file, 0 anywhere in the fork. Both pin sites
(`crates/me-cli/tests/descriptor_seam.rs:45`,
`.../seam-fork/nonstandard/descriptor_seam_test.go:40`) carry the new sha.

## 4. M-A — CONFIRMED FIXED

Reproduced the re-review's own constructions plus additional ones, against
`formats-happy/bluewallet-sh-fixture` with a substituted `Name:` label:

* **Quote-closing label** (`ok" -- nothing is wrong with this wallet. "`):
  renders as `the label "ok\" -- nothing is wrong with this wallet. \""` —
  both quotes escaped, delimiter unclosable.
* **Bidi** (`a‮KCATTA‬b​⁦x⁩`): all five codepoints
  render as `\u{202e}`/`\u{202c}`/`\u{200b}`/`\u{2066}`/`\u{2069}` — `xxd`
  confirms no raw UTF-8 bytes for any of them reach stderr.
  ("The device parser" for these constructions was not applicable —
  checked via `me`'s own stderr directly.)
* **C0/C1 bytes** (own construction, not in the fold's test set: ESC, BEL,
  BS, and C1 NEL U+0085): all escaped as `\x1b`, `\x07`, `\x08`,
  `\u{0085}`; `xxd` confirms no raw control bytes present.
  **Long label** (300× `A`): truncates cleanly with a trailing `…`.
* **Legitimate non-ASCII** (`Grüße — Konto Nº1 ✓ 日本語`): passes through
  completely unmangled, matching the fold's own control construction.

## 5. N-a / N-b — CONFIRMED FIXED

`IMPL-S1S3-fold2.md`'s correction of its own §9 claim reads accurately
against the diff: `a_quoted_fragment_never_spans_a_newline`'s doc now states
its scope is the stderr of the inputs the test itself drives, not a general
property, and names the one surface (`bluewallet_bad_fingerprint`, via
`elide_line`) that still isn't backtick-safe — confirmed by reading
`refusal.rs:461-476`: that function is the only caller of `elide_line`
rather than `quote_operator`, and `elide_line` does not escape backticks.
The new `delimiting_backticks` helper counts only non-escaped backticks, as
claimed.

`grep -rn "48 columns"` over `crates/` and `design/` outside
`design/agent-reports/`: **0** hits. `QUOTE_MAX` is named and documented as
counting `chars()`. Constructed a 120-character CJK label and measured the
truncation point directly: **48 glyphs** before the `…`, matching the
corrected doc's claim that 48 CJK characters occupy ~96 columns.

## 6. Suite + sweep — CONFIRMED

```
$ cargo nextest run --locked
     Summary [  32.170s] 562 tests run: 562 passed, 1 skipped
$ cargo clippy --all-targets --locked -- -D warnings     (clean, no output)
$ cargo fmt --check                                       (clean)
$ grep -c '^#\[ignore' crates/me-cli/tests/descriptor_seam.rs      -> 0
$ grep -rn '^#\[ignore' crates/ | wc -l                            -> 0
$ grep -c '^fn row_' crates/me-cli/tests/descriptor_refusals.rs    -> 36
```

Fork, from `/scratch/code/shibboleth/_work/seam-fork` (Go 1.26.3):

```
$ go test ./nonstandard/ -count=1 -v   -> PASS (7 run, 1 pre-existing SKIP unrelated to this fold, F-418)
$ go vet ./nonstandard/                -> clean
$ gofmt -l nonstandard/                -> clean
$ git status --porcelain               -> (empty)
```

**Mutation tests reproduced independently** (mutated `refusal.rs` in place,
ran nextest, reverted with `git checkout --`, confirmed clean tree after
each):

* Un-escaping `"` and removing the `is_invisible_or_directional` check:
  `562 run: 561 passed, 1 failed` — the sole failure is
  `a_quoted_label_can_neither_close_the_quote_nor_reorder_the_line`.
  Matches the report's claimed mutant exactly.
* Un-escaping `` ` ``: `562 run: 561 passed, 1 failed` — the sole failure is
  `a_backtick_inside_a_quoted_fragment_is_escaped`. Matches exactly.

**Propagation sweep, reproduced independently** over both worktrees
(excluding `.git`/`target`), for the same 10 patterns fold2.md's §7 table
names: **0 hits outside `design/agent-reports/`** in every case except
`conjunct 1 refuses first`, which returns exactly **2** hits — both inside
the corrected vector file's own correcting sentence ("The previous note said
conjunct 1 refuses first, which was the ordering that produced the C1
Critical…"), one copy in each repository, as disclosed in fold2.md §7.

**The single behavioral edit.** `git diff 171ec42..e56ae1b --
crates/me-cli/src/descriptor/refusal.rs` shows the only code change in
`src/` is `quote_operator` and its two new helpers (`QUOTE_MAX`,
`is_invisible_or_directional`) — no other `.rs` source file under `src/`
changed. Matches fold2.md's claim that "the one behavioural edit is
`quote_operator`'s escaping."

---

## Items not independently re-derived

Per the brief's scope: the spec's and plan's own GREEN, P0/P1/P2, the
adversarial review's clean list, and `gen.py`'s full end-to-end
re-measurement (the implementer disclosed not running it; reproducing it
needs the goprobe/rsprobe toolchain wiring, a P3-scale action, not a fold
verification).

## Verdict

**GREEN — 0 Critical / 0 Important / 1 Minor / 0 Nit.** All four claimed
fixes (I-A, M-A, N-a, N-b) are real, verified by independent execution
against the built binary, the fork's Go code, and by reproducing both
mutation results and the propagation sweep from scratch rather than trusting
either report's numbers. N-c's skip-by-instruction is confirmed accurate
(`design/FOLLOWUPS.md` untouched on the branch). The one finding — a
line-number citation (`parse.go:151` vs. the actual `:158`) asserted as
"measured" but carried forward unverified from an earlier report, now
present in the vector file's `source` annotation in both repositories — is
Minor: `source` is confirmed to bind no assertion in either test harness,
and the substantive claim it supports (which error fires, that it precedes
the `Title` gate) is independently confirmed true. This closes the loop.
