# IMPL-S1S3 — fold 2, against the fold-1 re-review

**Folder:** the P2 implementer, 2026-08-29.
**Review folded:** `design/agent-reports/IMPL-S1S3-fold1-rereview.md` (commit
`171ec42`) — RED, **0C / 1I / 1M / 3N** over `32b94c4..83703b4`.
**Heads:** engrave `impl/descriptor-s1s3` at **`e2002c3`** (this report lands on
top); fork `seam/descriptor-vectors` at **`a5e29b4`**. Nothing pushed, no tags,
nothing outside the two worktrees.

**Disposition: 4 fixed, 1 skipped by instruction (N-c).** The Important was in
the records, not the code, and no refusal behaviour changed in this fold — the
one behavioural edit is `quote_operator`'s escaping.

**New vector-file sha256, both repositories:**
`542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974`
(was `0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584`).

---

## Summary

| # | severity | disposition | measurement |
| --- | --- | --- | --- |
| I-A | Important | **FIXED**, 3 sites + a 4th the sweep found | §2 |
| M-A | Minor | **FIXED**, both constructions | §3 |
| N-a | Nit | **FIXED**, and my own report's claim corrected | §4 |
| N-b | Nit | **FIXED** | §5 |
| N-c | Nit | **SKIPPED** by instruction | §6 |

### Commits

| repo | sha | subject |
| --- | --- | --- |
| engrave | `0271f89` | spec: §7 clause 8 and §5.4's parenthetical state the SHIPPED conjunct order (SPEC ONLY) |
| engrave | `5adc0a2` | vectors: correct two `source` annotations; re-pin the sha256 in both repos |
| **fork** | **`a5e29b4`** | nonstandard: re-pin the descriptor seam vectors after two `source` corrections |
| engrave | `6156a24` | vectors: correct the GENERATOR's two `source` strings, so a re-run reproduces them |
| engrave | `e2002c3` | fold M-A/N-a/N-b: escape the delimiters and the invisible classes too |

---

## 1. The gate

```
$ cargo nextest run --locked
     Summary [  32.168s] 562 tests run: 562 passed, 1 skipped
$ cargo clippy --all-targets --locked -- -D warnings     clean
$ cargo fmt --check                                      clean
$ grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs   -> 0
$ grep -rn '^#\[ignore' crates/ | wc -l                        -> 0
$ grep -c '^fn row_' crates/me-cli/tests/descriptor_refusals.rs -> 36
$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json
  542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974
$ sha256sum .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
  542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974
$ (fork) go test ./nonstandard/ -count=1  -> ok  seedhammer.com/nonstandard  0.020s
         go vet ./nonstandard/  -> clean  ·  gofmt -l nonstandard/  -> clean
         git status --porcelain -> (empty)
```

### 1.1 An environment sensitivity I hit, and did NOT fix — for P3

Running the suite with a Go toolchain on `PATH` (which I had, for the fork
harness) turns one test RED:

```
FAIL mnemonic-engrave::cross_lang rust_ndef_parses_in_seedhammer_go_reader
  go harness failed: seedhammer.com@v0.0.0 (replaced by ../../third_party/seedhammer):
  reading .../third_party/seedhammer/go.mod: no such file or directory
```

`third_party/seedhammer` is a submodule this git WORKTREE never populated
(`ls -A third_party/seedhammer | wc -l` → 0), and `git diff 83703b4..HEAD --
third_party/ .gitmodules` is empty, so this is not a fold-2 regression — it is a
property of the worktree, present for every gate this cycle has run.

**What is worth recording is the direction of the failure.** The test's own doc
says *"locally-unset behavior is unchanged (skip note + pass)"* — with no `go`
it PASSES, and the differential oracle silently does not run. Every `560/560`
and `562/562` in this cycle was measured in that state. `ME_REQUIRE_GO=1` is the
guard, and CI sets it; this worktree cannot satisfy it either way. That is the
"a skipped gate prints ok" class, and it belongs to P3's records rather than to
this fold — I have not widened scope to it.

---

## 2. I-A — IMPORTANT, FIXED (three sites named, a fourth found)

**Nature.** Fold-1's C1 fix reordered `admit()`, and three sites still stated
the OLD order — the order that produced C1. No code change was asked for or
made; the build is right and the records were wrong.

### 2.1 Site 1 — `SPEC_descriptor_input.md` §7 clause 8, NORMATIVE (`0271f89`)

Was: *"whose conjunct-8 refusal binds the `--as md1` path ONLY: under
`--as descriptor` a `multi` gets conjunct 1's permanent refusal first"*.

Now states the shipped order — conjunct 1's `--as`-independent shape half, then
conjuncts 2–8, then conjunct 1's `multi`-under-`--as descriptor` arm — and
carries an `AMENDED 2026-08-29` block giving the reason, the anyone-can-spend
measurement, and §5.4's carriage rule as the authority. The row set it governs
is now *"all three rows on both `--as` paths"*.

### 2.2 Site 2 — §5.4's tier parenthetical (`0271f89`)

Applied conjunct 1's permanent refusal to BOTH the conjunct-8-PASSING and the
conjunct-8-FAILING `multi`. Only the first is true. They are now split
explicitly, with the reason (the flag-dependent arm runs last).

**What this makes true that was not.** §6's key-identity row already required
*"**`EXIT_REFUSED` (3)**, both `--as` paths"*, and the old ordering rule
contradicted it. The spec was internally inconsistent before fold 1 and is
consistent after fold 2. Marked in the status header as amendment **(b)**, in
the style of the two before it.

### 2.3 Site 3 — the vector file's `source`, plus the batched F-2 (`5adc0a2`, fork `a5e29b4`)

Both corrections in ONE byte-change event, per the brief.

**`source` is annotation — confirmed before relying on it, not assumed.** The
Rust harness names `"source"` only in `KNOWN_ROW_KEYS` (`descriptor_seam.rs:81`)
and in the required-key presence loop (`:232`); the fork's Go harness never
reads the field (`grep -i source` over `nonstandard/descriptor_seam_test.go` →
no hits). No assertion moved — only the two sha256 literals.

* `gate/colliding-origin-multi` — now states conjunct 8 binds BOTH paths, and
  names the C1 finding.
* `gate/deadbeef-fronts-an-xpub` — the parked IMPL-P1-report F-2. It claimed the
  pinned row followed *"the device's own precedence: parseBlueWalletDescriptor
  succeeds and OutputDescriptor's `Title != ""` gate is what refuses"*. Measured
  at fork `1f09537`, it does not succeed: `bluewallet: expected 0 keys, but got
  1` fires at `parse.go:151`, before the Title gate at `:37`, because the file
  carries no `Policy:` header. The pinned `refusal_row: bluewallet-no-name` is
  unchanged and still right — by §6's own standard rather than the device's,
  since the key-count row's text names a `Policy:` line this file does not have
  and would be FALSE about the operator's file.

Both harnesses re-run against the new bytes: Rust `15 tests run: 15 passed`,
fork `ok 0.064s`.

### 2.4 Site 4 — the GENERATOR, which the widened sweep found (`6156a24`)

`scripts/descriptor-seam-vectors/rows.py` — where the 71 rows are AUTHORED —
still carried BOTH superseded `source` strings (`:398`, `:458`). Re-running the
generator would have silently reverted both corrections and moved the sha256
back, **in two repositories**. The artifact was fixed; without this the fix had
an expiry date.

This is the constellation's own measured class — *a reproduction path nobody
re-runs rots while its artifact keeps vouching for it* — and it is the direct
dividend of widening the sweep, since `scripts/` was outside fold-1's search
space exactly as `design/` and `testdata/` were.

Checked rather than asserted, every row:

```
71 rows compared, 0 source strings differ
```

---

## 3. M-A — MINOR, FIXED

`quote_operator` now escapes four classes, not one: C0/C1 controls, **both
delimiters it is interpolated into** (`"` and `` ` ``), **the escape character
itself**, and **Unicode `Cf` plus the private use areas**.

**(a) the delimiter**, the re-review's own label:

```
me: warning: the label "ok\" -- nothing is wrong with this wallet. \"" is not carried
    by any record format and will not appear on the device. Nothing else is lost.
```

**(b) bidi**, raw bytes read off stderr:

```
b'me: warning: the label "a\\u{202e}KCATTA\\u{202c}b\\u{200b}\\u{2066}x\\u{2069}" is not carried \xe2\x80\xa6'
```

All five of U+202E, U+202C, U+200B, U+2066, U+2069 escaped.

**The control that bounds the fix.** A legitimate label survives unmangled —

```
me: warning: the label "Grüße — Konto Nº1 ✓ 日本語" is not carried by any record format …
```

— which is why the fix escapes by CLASS rather than by an ASCII allowlist: the
operator's job at this line is to recognise their own label. `\` is escaped too,
which the review did not raise but which follows from introducing an escape
syntax at all: once the function emits `\x1b`, a literal backslash in a label is
indistinguishable from an escape it produced.

`is_invisible_or_directional` hard-codes the `Cf` ranges plus PUA, one commented
line per range so a reader can check it rather than trust it. No dependency:
`std` exposes no general-category table and this does not warrant one. Erring
wide is safe here — a false positive escapes a character instead of hiding it.

```
MUTANT Cf passes through + `"` unescaped -> 562 run: 561 passed, 1 FAILED
```

---

## 4. N-a — NIT, FIXED; and the correction to my own fold-1 report

**The correction, recorded here rather than by editing the old report.**
`IMPL-S1S3-fold1.md` §9 said of the N2 test: *"The test asserts every emitted
line has an **even** backtick count, which pins the general property rather than
this one case."* **That sentence is wrong twice** and the re-review was right to
call it:

1. the loop runs over the stderr of ONE input, so it never pinned a general
   property; and
2. the property is not true in general — a backtick inside an
   operator-supplied fragment falsified it, which is N-a.

`IMPL-S1S3-fold1.md` is left byte-untouched; a persisted report is a record of
what was claimed.

**The fix.** `quote_operator` escapes `` ` `` as well as `"`, because one helper
serves two call sites with different delimiters:

```
before:  the use-site path is not a path: `a`b`.
after:   the use-site path is not a path: `a\`b`.
```

**And the test I wrote to pin it was wrong in the same way as the report.** Its
first version asserted an even count of ALL backticks and **failed on its own
new fixture** — `` `a\`b` `` carries three, correctly rendered: two delimiters
plus one escaped literal. The property is narrowed to what is true:
*delimiting* backticks pair up, counted by `delimiting_backticks`, which
discounts a backslash-escaped one. The test's doc now states its scope and
explicitly does **not** claim the general property — other refusal texts carry
backticks in fixed prose, and the cosigner-line row still quotes an operator
line through `elide_line` rather than `quote_operator`.

```
MUTANT `` ` `` unescaped -> 562 run: 561 passed, 1 FAILED
```

---

## 5. N-b — NIT, FIXED

*"bounded at 48 columns"* → **48 characters**, and the constant is named
`QUOTE_MAX` with the distinction spelled out at its definition: `width`
accumulates `chars().count()`, so 48 CJK glyphs occupy roughly 96 terminal
columns. `grep -rn "48 columns" crates/ design/` outside
`design/agent-reports/` → **0**.

---

## 6. N-c — SKIPPED by instruction

The controller files N1's declination in `design/FOLLOWUPS.md` at P3.
`FOLLOWUPS.md` is **untouched on this branch** — verified:
`git diff 44e121a..HEAD -- design/FOLLOWUPS.md` is empty — so the branch still
merges cleanly against a `master` that has moved.

---

## 7. Propagation sweep — WHOLE-REPO scope

**Scope searched, stated because the scope IS the finding:** the entire
`impl-s1s3` worktree (all of `crates/`, `design/`, `scripts/`, `firmware/`,
`preview/`, every top-level file) **and** the entire `seam-fork` worktree,
excluding only `.git/` and `target/`. Fold-1's sweep covered `crates/` alone,
which is why I-A's three sites — in `design/` and `testdata/` — and the fourth,
in `scripts/`, were all invisible to it.

Persisted reports under `design/agent-reports/` are excluded from the counts
below: they quote the old text deliberately, and rewriting them would destroy
what a record is for.

| old form | non-history hits |
| --- | ---: |
| `conjunct 1's permanent refusal first` | **0** |
| `binds the \`--as md1\` path ONLY` | **0** |
| `conjunct 8 binds the --as md1 path` | **0** |
| `Title != "" gate is what refuses` | **0** |
| `the device's own precedence` | **0** |
| `different depths` | **0** |
| `no single first address to compare` | **0** |
| `conjunct_1_shape(d, path)` | **0** |
| `48 columns` | **0** |
| `0393592f…` (the superseded digest) | **0** |
| `conjunct 1 refuses first` | **2 — disclosed below** |

**The two hits, disclosed rather than excluded.** Both are the two copies of the
vector file, and both are inside the sentence that CORRECTS the rule:

> `The previous note said conjunct 1 refuses first, which was the ordering that
> produced the C1 Critical (IMPL-S1S3-adversarial-review)`

A future implementer grepping that phrase lands on a line telling them, in the
same sentence, that it is the superseded reading and why — which is the outcome
the sweep exists to produce, not a stale statement. **Removing it is available
but would cost a second two-repo byte-change and re-pin**, which the brief
explicitly asked me to avoid ("one byte-change event, not two"); the trade is
recorded here so the controller can reverse it cheaply if they would rather the
grep be literally clean.

The superseded digest is gone from both repositories: the only two pin sites
were `crates/me-cli/tests/descriptor_seam.rs:45` and
`.../seam-fork/nonstandard/descriptor_seam_test.go:40`, and both now carry
`542cd492…`.

---

## 8. What a re-reviewer should look at first

1. **`is_invisible_or_directional`'s range list is hand-copied Unicode data.**
   It is commented range by range and errs wide, but it is the one thing in this
   fold that a test cannot prove complete — the tests pin the five characters
   the re-review constructed, not the category.
2. **The generator now agrees with the file on `source` for all 71 rows, but I
   did not re-run `gen.py` end to end** — that needs the goprobe/rsprobe
   toolchain wiring and would re-measure every device column, which is a P3-scale
   action, not a fold. The check I ran is the authored half only, which is where
   both defects were.
3. **§1.1's `cross_lang` observation** is the only thing I found and did not
   fix, deliberately.
