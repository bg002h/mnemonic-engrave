# Hashlock H6 — implementer A, independent adversarial execution review (pre-publish)

**Verdict: GREEN. 0 Critical, 0 Important, 2 Minor, 2 Nit.**

Reviewed: `git -C /scratch/code/shibboleth/ms-worktrees/h6-a diff 504ff46..ca715165`
— 9 files, 635 insertions, 90 deletions. Every hunk in every file was read; the
hunk headers are enumerated in §7 so the coverage claim is checkable.

Scope: the ONE question of the brief — can a typed phrase, an ms1 string, or a
`(phrase, method)` pair be constructed for which ms-codec 0.9.0's
`validate_phrase` / `looks_like_ms1` / `qr_text` differ from what ms-cli 0.18.0
did at `504ff46` for the same bytes; can any of the seven `qr_text` corpus rows
be shown not to follow from the three constants and §8.6's normative text; and
does every test the diff adds fail on the defect it names.

**I could not construct one.** Two independent instruments, both controlled:

- A **differential fuzz** (§1) that re-implements the DELETED `ms-cli`
  predicates verbatim and diffs them against the ms-codec replacements over
  **1,200,045 inputs** with every clause of the rule densely reached —
  **0 disagreements**.
- An **end-to-end A/B** (§2) of the two built binaries over **108 cases** —
  **108/108 byte-identical** on exit code, stdout AND stderr.

Both instruments were then shown to DETECT a divergence when one is introduced
(§1.2, §2.1), so the negative result is a measurement and not a silent pass.

Work was done in my own detached worktrees (`h6-a-review` at `ca715165`,
`h6-a-base` at `504ff46`) with their own target dirs; both are removed, and the
implementer's branch worktree is untouched and clean at
`ca71516511db0b608a14c7dc3f82376424b107a8`. Nothing was committed.

---

## 0. What the controller settled, and what I re-derived anyway

Not re-derived, per the brief: the tip's `cargo nextest run --locked`,
`fmt --check`, `clippy -D warnings`, `publish --dry-run`, the corpus sha256, the
`Cargo.lock` scope, and the `ms-cli/Cargo.toml` one-line change.

I did re-run two of them at the end as a **pristine-tree proof**, because this
review mutated files in its own worktree and had to show it left none behind:

```
HEAD=ca71516511db0b608a14c7dc3f82376424b107a8
status: ''  (blank = pristine)
diff vs ca715165: ''
     Summary [   0.248s] 562 tests run: 562 passed, 11 skipped
fmt exit=0
```

---

## 1. Behavioural equivalence of the move — the differential

### 1.1 Clause-by-clause, then measured

The deleted `ms-cli` predicate and the added `ms-codec` one differ in exactly
two places, and both are provably nil:

| clause | 0.18.0 (`ms-cli`) | 0.9.0 (`ms-codec`) | verdict |
| --- | --- | --- | --- |
| normalisation | `raw.trim().to_ascii_lowercase()` | same | identical |
| separator strip | `crate::format::strip_display_separators(s)` | `.chars().filter(\|c\| !c.is_whitespace() && *c != '-' && *c != ',')` | **identical** — `format::is_display_separator` is *literally* `c.is_whitespace() \|\| c == '-' \|\| c == ','` (`crates/ms-cli/src/format.rs:12-14`) |
| length / prefix / charset | `t.len() >= 48 && t.starts_with("ms1") && t[3..]…` | same, same `MIN_MS1_LEN`, same `BECH32_CHARSET` | identical |
| rule order | empty → printable → ms1 → cap → 64-hex | same | identical |
| cap | `HASHLOCK_PHRASE_MAX_CHARS = 100`, `s.len()` (bytes) | same constant value, same `s.len()` | identical |
| 64-hex | `s.len() == 64 && hex::decode(s).is_ok()` | `s.len() == 64 && s.bytes().all(\|b\| b.is_ascii_hexdigit())` | **the one deliberate change.** Over an already-64-byte window the `hex` crate accepts exactly `[0-9a-fA-F]` pairs, so the predicates are equal; measured below |

I did not stop at reading. I added a temporary `#[cfg(test)] mod rev_differential`
inside `crates/ms-cli/src/argv_guard.rs` (the only place that can reach both, as
`ms-cli` is a binary-only crate with no lib target) which re-implements
`old_is_ms1_shaped`, `old_looks_like_ms1` and `old_validate_phrase` **verbatim as
they stood at `504ff46`**, including `strip_display_separators` and
`hex::decode`, and diffs them against the codec's.

```
REV looks_like_ms1: checked=400018 true_cases=207749 disagreements=0
REV validate_phrase: checked=800027 outcomes={"Empty": 3493, "Hex64": 22362,
  "Ms1Shaped": 138098, "NotPrintableAscii": 369704, "Ok": 240648,
  "TooLong": 25722} disagreements=0
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 77 filtered out
```

The generators are structured rather than uniformly random, because a uniform
generator does not reach the interesting branch: my first run reached only **12**
TRUE cases for `looks_like_ms1` and the harness's own non-vacuity guard failed it.
The final generators mutate a VALID `ms1` string (case folds, separator
insertions at every position, truncations across the 48-character boundary,
NBSP/EM-SPACE/leading-trailing padding, non-bech32 substitutions) and build
phrases straddling the 100-character cap and 63/64/65-character hex windows. All
six rule outcomes are reached in the thousands, and 207,749 of the 400,018
`looks_like_ms1` inputs are TRUE.

### 1.2 The instrument is controlled

Three mutations of the codec, each run once and reverted; each was caught:

| control mutation | disagreements | first counterexample |
| --- | --- | --- |
| drop `to_ascii_lowercase` | **11** | `FIXED "MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F": old=true new=false` |
| `is_ascii_hexdigit() \|\| b == b'g'` | **11** | a 64-char window ending `…101g`: `old=Ok new=Hex64` |
| cap check BEFORE the ms1-shape check | **11** | a 108-char grouped ms1: `old=Ms1Shaped new=TooLong { chars: 108 }` |

A harness that returns 0 on the real code and 11 on each of three seeded defects
is measuring; the negative result stands.

---

## 2. End-to-end A/B of the two binaries

Both binaries built from source: `0.18.0` at master `504ff46`, and the branch at
`ca715165`. Both report `ms 0.18.0` (`ms-cli`'s own version is correctly
untouched). For each case the harness compares **exit code, stdout AND stderr in
full**, not just the first line.

```
A/B cases: 108   FULL-OUTPUT identical: 108   DIFFERING: 0
```

### 2.1 The A/B harness is controlled

Rebuilding the branch with `HASHLOCK_PHRASE_MAX_CHARS = 99` and re-running the
same 108 cases: `FULL-OUTPUT identical: 101   DIFFERING: 7`. The harness detects
a one-constant change, so 108/108 is a measurement.

### 2.2 The table

The cases cover every clause the brief names: a phrase of exactly 100 / 101 / 102
characters; 100 multibyte characters; leading, interior and trailing TAB; CR;
NUL in two positions; DEL; `0x1f`; `0x80`; `0xff`; the `0x20`/`0x7E` boundary;
`ms1` under every separator (space, hyphen, comma, tab) and in lower, UPPER and
mixed case, with and without surrounding whitespace, and grouped-by-2 at 112
characters (the order clause); 46/47/48/49-character ms1-shaped strings;
64-hex in lower, UPPER and mixed case, 63- and 65-hex, and 64-character windows
carrying `g`, `-` or a space; all 15 corpus `refusals` rows; the argv guard on
raw tokens through four different verbs; and both methods with and without the
engraving card and `--json`.

| input | 0.18.0 exit | branch exit | full stdout+stderr identical | first stderr line (both) |
| --- | --- | --- | --- | --- |
| `corpus[0] empty ''` | 1 | 1 | True | error: the hashlock phrase is empty |
| `corpus[1] printable-ascii 'café'` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `corpus[2] hex=ff rule=printable-ascii` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `corpus[3] printable-ascii 'a\tb'` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `corpus[4] hex=617f rule=printable-ascii` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `corpus[5] None ' ~'` | 0 | 0 | True |  |
| `corpus[6] 64-hex 'c3e97525442520da4cffd5f57aae3f6273` | 1 | 1 | True | error: that is 64 hex characters -- a preimage, 32 bytes (64 |
| `corpus[7] 64-hex 'C3E97525442520DA4CFFD5F57AAE3F6273` | 1 | 1 | True | error: that is 64 hex characters -- a preimage, 32 bytes (64 |
| `corpus[8] None 'beef'` | 0 | 0 | True |  |
| `corpus[9] ms1-shaped 'ms10hashsqw46h2at4w46h2at4w46h` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `corpus[10] ms1-shaped 'MS10HASHSQW46H2AT4W46H2AT4W46` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `corpus[11] ms1-shaped 'ms10h ashsq w46h2 at4w4 6h2at` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `corpus[12] ms1-shaped '  ms10hashsqw46h2at4w46h2at4w` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `corpus[13] ms1-shaped 'ms 10 ha sh sq w4 6h 2a t4 w4` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `corpus[14] too-long 'hashlock phrase row: one hundre` | 1 | 1 | True | error: the hashlock phrase is 101 characters; at most 100 ar |
| `empty` | 1 | 1 | True | error: the hashlock phrase is empty |
| `space only` | 0 | 0 | True |  |
| `tilde` | 0 | 0 | True |  |
| `0x20+0x7e` | 0 | 0 | True |  |
| `100 chars` | 0 | 0 | True |  |
| `101 chars` | 1 | 1 | True | error: the hashlock phrase is 101 characters; at most 100 ar |
| `102 chars` | 1 | 1 | True | error: the hashlock phrase is 102 characters; at most 100 ar |
| `100 multibyte chars` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `leading TAB` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `interior TAB` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `trailing TAB` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `CR interior` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `NUL interior` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `NUL leading` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `DEL` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `0x1f` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `0x80` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `0xff` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `ms1 lower` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 UPPER` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `MS1 mixed case` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 grouped space` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 grouped hyphen` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 grouped comma` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 grouped tab` | 1 | 1 | True | error: the hashlock phrase must be printable ASCII (bytes 0x |
| `ms1 leading/trailing ws` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1 grouped2 (112ch)` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1-like len 47` | 0 | 0 | True |  |
| `ms1-like len 48` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1-like len 49` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `MS1-like len 48 upper` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `ms1-like 48 w/ non-bech32 b` | 0 | 0 | True |  |
| `ms1-like 48 w/ dot` | 0 | 0 | True |  |
| `ms1 filename` | 0 | 0 | True |  |
| `64 hex lower` | 1 | 1 | True | error: that is 64 hex characters -- a preimage, 32 bytes (64 |
| `64 hex UPPER` | 1 | 1 | True | error: that is 64 hex characters -- a preimage, 32 bytes (64 |
| `64 hex mixed` | 1 | 1 | True | error: that is 64 hex characters -- a preimage, 32 bytes (64 |
| `63 hex` | 0 | 0 | True |  |
| `65 hex` | 0 | 0 | True |  |
| `64 with g` | 0 | 0 | True |  |
| `64 with hyphen` | 0 | 0 | True |  |
| `64 with space` | 0 | 0 | True |  |
| `short hex beef` | 0 | 0 | True |  |
| `phrase with colon` | 0 | 0 | True |  |
| `phrase trailing space` | 0 | 0 | True |  |
| `phrase with comma` | 0 | 0 | True |  |
| `anchor` | 0 | 0 | True |  |
| `argv --in ms1 lower` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv decode ms1 lower` | 1 | 1 | True | ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase ms1 lower` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1 lower` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `argv --in ms1 UPPER` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv decode ms1 UPPER` | 1 | 1 | True | ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase ms1 UPPER` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1 UPPER` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `argv --in ms1 grouped` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv decode ms1 grouped` | 1 | 1 | True | ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase ms1 grouped` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1 grouped` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `argv --in ms1 comma` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv decode ms1 comma` | 1 | 1 | True | ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase ms1 comma` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1 comma` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `argv --in ms1 filename` | 1 | 1 | True | error: failed to read --in ms1-2026-08-23-backup.txt: No suc |
| `argv decode ms1 filename` | 1 | 1 | True | error: string length 21 not in v0.1 set [50, 56, 62, 69, 75] |
| `argv hashlock-phrase ms1 filename` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1 filename` | 0 | 0 | True |  |
| `argv --in ms1-like 48` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv decode ms1-like 48` | 1 | 1 | True | ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase ms1-like 48` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1-like 48` | 1 | 1 | True | error: that is an ms1 string, not a hashlock phrase; pass it |
| `argv --in ms1-like 47` | 1 | 1 | True | error: failed to read --in ms1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqq |
| `argv decode ms1-like 47` | 1 | 1 | True | error: string length 47 not in v0.1 set [50, 56, 62, 69, 75] |
| `argv hashlock-phrase ms1-like 47` | 1 | 1 | True | ms: argument 3 on ARGV (arguments count from 0, and 0 is `ms |
| `argv hashlock-phrase+allow ms1-like 47` | 0 | 0 | True |  |
| `--method hardened 'correct horse batter'` | 0 | 0 | True |  |
| `--method hardened CARD 'correct horse batter'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method hardened JSON 'correct horse batter'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |
| `--method hardened 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True |  |
| `--method hardened CARD 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method hardened JSON 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |
| `--method hardened 'one, two, three'` | 0 | 0 | True |  |
| `--method hardened CARD 'one, two, three'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method hardened JSON 'one, two, three'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |
| `--method sha256 'correct horse batter'` | 0 | 0 | True |  |
| `--method sha256 CARD 'correct horse batter'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method sha256 JSON 'correct horse batter'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |
| `--method sha256 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True |  |
| `--method sha256 CARD 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method sha256 JSON 'aaaaaaaaaaaaaaaaaaaa'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |
| `--method sha256 'one, two, three'` | 0 | 0 | True |  |
| `--method sha256 CARD 'one, two, three'` | 0 | 0 | True | THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries |
| `--method sha256 JSON 'one, two, three'` | 0 | 0 | True | warning: stdout carries private key material (can spend) — r |

Five distinct refusal sentences, both exit codes, the engraving card and the
JSON warning all appear in the table, so the identity is not the identity of 108
identical clap errors.

---

## 3. `qr_text` — the seven rows recomputed from the constants

Recomputed **independently in Python** from `HASHLOCK_SALT = "ms-hashlock-v1"`,
`HASHLOCK_ITERATIONS = 100000`, `HASHLOCK_DKLEN = 32` and §8.6's template, with
no reference to the Rust implementation:

```
row                      method    recomputed==row  len   row.bytes match
anchor-hardened          hardened  True             122   122       True
anchor-sha256            sha256    True             63    63        True
max-phrase-hardened      hardened  True             194   194       True
max-phrase-sha256        sha256    True             135   135       True
phrase-with-colon        hardened  True             119   119       True
phrase-trailing-space    hardened  True             102   102       True
phrase-with-comma        hardened  True             109   109       True
ALL: True
hardened method line len: 73
worst case hardened(100 chars): 194
worst case sha256(100 chars): 135
```

Byte counts **122/63/194/135/119/102/109** are the plan's Step 4 table exactly,
and the row names and methods match it row for row. The 73-character method line
and the 194/135 worst cases are §6.5's and §7.1's own pins.

Against the spec's normative text (§8.6, `SPEC_hashlock_H6_preimage_plates.md:1821`):
three labelled lines, LF-separated (rule: three lines — asserted), no trailing
newline (asserted), the phrase LAST (asserted), `hashlock v1` as line 1
(asserted), parameters read from the three constants and never a literal
(asserted), the sha256 middle line exactly `method: sha256` (asserted). Every
rule of §8.6 has an assertion behind it.

**Additional checks the plan does not make, run here.** I wrote a temporary
`crates/ms-codec/tests/rev_api_probe.rs` and confirmed for all seven rows that
`lines[1]` IS the method line — which is what Task 5b's planned
`MethodLine(hardened) == lines[1]` test depends on, and which nothing in Task 1
pins:

```
REV corpus: 7 rows, 5 hardened, 2 sha256, all method lines at index 1
```

Both method arms are present (5 hardened / 2 sha256), so Task 5b's non-vacuity
sweep will find both. I also asserted that **every corpus row's phrase itself
passes `validate_phrase`** — otherwise the corpus would pin a plate text no
`me sysw pack` `phrase:` record could ever produce. All seven pass.

### 3.1 The corpus delta is exactly what is claimed

```
base keys: ['format','kind','derivation','refusals','lengths_by_door','downgrade','lockstep']
tip  keys: [... , 'qr_text']
added keys: ['qr_text']      removed keys: []
  format            unchanged=False
  kind              unchanged=True
  derivation        unchanged=True
  refusals          unchanged=True
  lengths_by_door   unchanged=True
  downgrade         unchanged=True
  lockstep          unchanged=True
```

`format` moves to `"ms hashlock corpus v0.9 (SPEC_ms_hashlock §8; qr_text rows
added by SPEC_hashlock_H6 §8.6/§11.2)"`. The report's §3 claim that the whole
file was re-serialised pretty-printed with content untouched is **confirmed**:
every other top-level key is semantically identical.

Corpus sha256, measured from git objects rather than the working tree:

```
ca715165   4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4
ffdb77d    4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4
504ff46    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30
```

The corpus commit `ffdb77d` and the tip agree (so implementer C may pin either),
the new sha is the one the plan pins and the CHANGELOG records, and the old sha
is the one the CHANGELOG's "was" cites. All four literals check out.

---

## 4. The published artifact itself

The brief's question is about a publish, so I unpacked the `.crate` the release
will upload and ran it standalone, outside the workspace:

```
sha256  4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4
        crateck/ms-codec-0.9.0/tests/vectors/hashlock-v0.8.json
name = "ms-codec"
version = "0.9.0"

running 3 tests
test parameters_come_from_the_constants ... ok
test the_worst_case_is_194_bytes ... ok
test qr_text_matches_every_corpus_row ... ok
test result: ok. 3 passed; 0 failed
```

Both `ms-codec-0.9.0/tests/hashlock_qr_text.rs` and
`ms-codec-0.9.0/tests/vectors/hashlock-v0.8.json` are in the archive, so the
published crate's `include_str!` resolves and its corpus is byte-identical to the
repo's. The implementer's §5 claim is confirmed, and confirmed one step further
than the report took it: the packaged crate is not merely present but **green**.

Toolchain, in the worktree:

```
1.85.0-x86_64-unknown-linux-gnu (overridden by '…/h6-a-review/rust-toolchain.toml')
clippy 0.1.85 (4d91de4e48 2025-02-17)
```

`ci/repro/vendor-freshness.sh` → `vendor-freshness: OK — vendor/ satisfies Cargo.lock.`

---

## 5. The public API surface

`crates/ms-codec/tests/rev_api_probe.rs` (temporary, removed) resolved every name
Tasks 1, 2 and 5b consume **from outside the crate**, bound to the plan's exact
signatures as `fn` pointers, so a changed arity or type is a compile error:

| item | signature asserted | consumer per the plan | resolves |
| --- | --- | --- | --- |
| `validate_phrase` | `fn(&[u8]) -> Result<(), PhraseRefusal>` | Task 1, Task 2 | yes |
| `looks_like_ms1` | `fn(&str) -> bool` | Task 1 | yes |
| `qr_text` | `fn(bool, &str) -> String` | Task 1, Task 5b (Go twin) | yes |
| `HASHLOCK_PHRASE_MAX_CHARS` | `usize`, `== 100` | Task 1 | yes |
| `PhraseRefusal` + all 5 variants | constructible, matchable, `Copy`+`Clone`+`Debug`+`Eq` | Task 1 | yes |
| `preimage_hardened` / `preimage_sha256` | `fn(&[u8]) -> Zeroizing<[u8; 32]>` | Task 2 | yes |
| `digest` | `fn(&[u8; 32]) -> [u8; 32]` | Task 2 | yes |
| `HASHLOCK_SALT` / `HASHLOCK_ITERATIONS` / `HASHLOCK_DKLEN` | `&[u8]` / `u32` / `usize` | Task 5b | yes |

All five items SPEC §3.1 clause 1 requires to move are present and public. No
name a later implementer will call is missing. `HASHLOCK_PHRASE_MAX_CHARS` is the
one item no shipped test referenced from outside the crate, which is why I probed
it explicitly — it is reachable.

---

## 6. Tests that cannot fail — every mutation run

Each mutation applied to the branch tree in my own worktree, run once, reverted,
and the tree confirmed clean afterwards.

| # | mutation | result | verdict |
| --- | --- | --- | --- |
| 1 | `qr_text` emits a trailing newline | 2 of 3 red. `hashlock_qr_text.rs:51` row `anchor-hardened`; `:121` `left: 195 right: 194` | reproduces |
| 2 | `phrase:` line before `method:` line | 2 of 3 red. `:51` `left: "hashlock v1\nphrase: correct horse battery staple\nmethod: …"` vs `right: "hashlock v1\nmethod: …\nphrase: …"` | reproduces |
| 3 | `HASHLOCK_ITERATIONS` `100_000`→`100_001`, corpus untouched | `qr_text_matches_every_corpus_row` red, `left: …iterations=100001…` vs `right: …iterations=100000…` | reproduces |
| 4 | drop `dklen={HASHLOCK_DKLEN}` from the method line | **3 of 3 red**, and `parameters_come_from_the_constants` panics at **`hashlock_qr_text.rs:100`** (the `dklen=` `contains` row), NOT at `:104` | reproduces — see below |
| 4b | shorten the ALGORITHM NAME, all three parameters kept | 3 of 3 red; `:104` `assertion left == right failed: the hardened method line is 73 characters …  left: 61  right: 73` | reproduces |
| 5 | drop `to_ascii_lowercase` from `ms_codec::hashlock::looks_like_ms1` | `argv_guard.rs:526:9: assertion failed: looks_like_ms1("MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F")` **and** `hashlock_phrase_rule.rs:131:9: assertion left == right failed: stdin UPPERCASE:  left: Some(0)  right: Some(1)` | reproduces **byte-exact** to the report |

**The implementer's correction of the plan is CORRECT, and I reproduced both
halves of it.** The plan's Step 5 says *"drop a parameter from the method line →
the 73-character assertion fails"*. Measured, mutation 4 panics at `:100`, the
`dklen=` `contains` assertion, which precedes the 73-character assertion at
`:104`; all three parameters have their own `contains` row, so no dropped
parameter can reach the length row. Mutation 4b then shows the length row **can**
fail on its own. The plan's conclusion (the test reds) holds; the assertion it
names is not the one that fires. This is a defect in the PLAN's prose, already
found and recorded by the implementer, not a defect in the code or the tests.

My 4b reports `left: 61` where the report reports `left: 68`, because we shortened
the algorithm name by different amounts (`pbkdf2-hmac-sha256` → `pbkdf2` is 12
characters; the report's was 5). Same assertion, same line, same conclusion — not
a discrepancy in the report's claim.

Every test the diff adds therefore fails on the defect it names, and the two
tests whose mutation the plan mis-attributed both still fail.

---

## 7. Diff coverage, records and the lockfile

Every hunk of all 9 files was read:

```
CHANGELOG.md                                     |  42 +++
Cargo.lock                                       |   2 +-      (ms-codec 0.8.0 -> 0.9.0, one line)
crates/ms-cli/Cargo.toml                         |   2 +-      (the pin, one line)
crates/ms-cli/src/argv_guard.rs                  |  67 +++--   @@ -92,16 +92,6 @@ / @@ -146,21 +136,30 @@ / @@ -501,22 +500,32 @@
crates/ms-cli/src/hashlock_phrase.rs             |  41 ++-     @@ -115,31 +115,24 @@   (one hunk; no test changes)
crates/ms-codec/Cargo.toml                       |   2 +-
crates/ms-codec/src/hashlock.rs                  | 126 +++++++++
crates/ms-codec/tests/hashlock_qr_text.rs        | 123 +++++++++
crates/ms-codec/tests/vectors/hashlock-v0.8.json | 320 ++++++++++++++++++++---
```

`scripts/h6-plan-blocks-vs-tree.sh` run with the branch as the `ms` tree:
**97 blocks checked, 0 FAIL**, and **6 of 6 `ms/` blocks PASS** (plan lines 283,
380, 412, 426, 492, 516). The branch's code is byte-for-byte the plan's blocks.

`MIGRATION.md` and the `FOLLOWUPS.md` cross-repo entry are known-OPEN and
release-agent owned; per the brief they are not reported here. Deviation D4
correctly names both.

---

## 8. The report's counts, independently measured

Per the brief, a false count in the implementer's report is Critical. Every
measurable claim in it was re-measured:

| report claim | measured | verdict |
| --- | --- | --- |
| baseline `559 tests run: 559 passed, 11 skipped` | `Summary [0.266s] 559 tests run: 559 passed, 11 skipped` at `504ff46` | **true** |
| tip `562 tests run: 562 passed, 11 skipped` | `Summary [0.248s] 562 tests run: 562 passed, 11 skipped` | **true** |
| "the three new tests are the whole delta" | 562 − 559 = 3; `hashlock_qr_text.rs` contains exactly 3 `#[test]` | **true** |
| corpus sha `4f1819cd…21d4` at `ffdb77d` and at the tip | both `4f1819cd…21d4` | **true** |
| previous sha `a46c197a…1d30` | `504ff46` hashes to `a46c197a…1d30` | **true** |
| byte counts `[122, 63, 194, 135, 119, 102, 109]` | recomputed from the constants: identical | **true** |
| three commits off `504ff46`, tree clean, nothing pushed | `ffdb77d`, `68c6602`, `ca71516`; worktree clean; branch not on origin | **true** |
| `97 blocks checked, 0 FAIL`; `6 of 6 ms/ blocks PASS` | re-run: identical | **true** |
| `rustup show active-toolchain` → 1.85.0 via `rust-toolchain.toml`; `clippy 0.1.85` | identical | **true** |
| dry-run packages the test and the corpus | both in the `.crate`, and it passes standalone | **true**, and stronger |
| `vendor-freshness.sh` → OK | `OK — vendor/ satisfies Cargo.lock` | **true** |
| mutation 5's two failures, quoted | reproduced byte-exact | **true** |
| O1: the test-support `is_ms1_shaped` case-folds and is behaviourally equal | `tests/support/mod.rs:30-39` does `to_ascii_lowercase` then the same filter; it omits `.trim()`, which the filter subsumes since it removes ALL whitespace | **true** |
| O2: `hex::decode` ≡ `is_ascii_hexdigit` over a 64-byte window | 22,362 `Hex64` outcomes and 0 disagreements in the differential | **true** |

**No false count.** Every number in the report is what the tree produces.

### Deviations, each with a verdict

| # | deviation | verdict |
| --- | --- | --- |
| **D1** | `crates/ms-cli/Cargo.toml` edited (`=0.8.0` → `=0.9.0`) although not in the file list | **CORRECT and forced.** Reverted the pin and ran `cargo metadata --locked`: exit 101, `failed to select a version for the requirement 'ms-codec = "=0.8.0"' … candidate versions found which didn't match: 0.9.0`. One line, minimal, `ms-cli`'s own version untouched at 0.18.0 |
| **D2** | no vendor-refresh commit | **CORRECT.** `vendor-freshness.sh` is green at the tip; workspace members are not vendored, and the lock moved by exactly the one member line |
| **D3** | corpus row `name`/`phrase`/`note` taken from the gated tree | **CORRECT and harmless.** Everything mechanical (the `qr_text` bytes and the counts) recomputes from the constants — I regenerated all seven independently and they match byte for byte |
| **D4** | Step 6 items 3 and 9 not done, and not in either brief | **CORRECT.** Out of scope for this review by the brief; the flag itself is the right call and should reach the release agent |

---

## 9. Findings

### M-1 — the CHANGELOG entry is dated `unreleased`, and every precedent carries a date

`CHANGELOG.md:7` is `## ms-codec [0.9.0] — unreleased`. Measured, every other
`ms-codec` entry carries a real date:

```
7:## ms-codec [0.9.0] — unreleased
72:## ms-codec [0.8.0] — 2026-09-05
223:## ms-codec [0.6.0] — 2026-06-21
284:## ms-codec [0.5.0] — 2026-06-21
321:## ms-codec [0.4.4] — 2026-06-12
```

`RELEASE_PROCESS.md` item 2 pins the header's per-crate prefix and the three
subsections but does not mention a date, so this violates no written rule — it is
accurate *right now*, and wrong the moment the crate is published. The word ships
inside the `.crate`. **Owner: the release agent**, as a one-word edit at the
release commit, before item 8's tag. Minor, non-blocking.

### M-2 — `qr_text` returns the phrase in an unprotected `String`

`qr_text(hardened, phrase: &str) -> String` builds a heap `String` containing the
hashlock phrase verbatim, with no `Zeroizing`, while every other phrase path in
the workspace is `Zeroizing<Vec<u8>>` (`read_phrase_from`, `read_phrase_stdin`,
`preimage_hardened`, `preimage_sha256`). The returned `String` is a secret that
outlives the call and is dropped without being wiped.

Per the operator ruling of 2026-08-27, a failure to handle secret material
secretly is **never Critical and never Important** — logged here for future
optimization, and it does not hold the gate. Recording it because the ruling is a
severity rule, not a discovery rule. A follow-up would return
`Zeroizing<String>`; note that would be a **breaking** signature change, so it is
cheaper to decide before the publish than after.

### N-1 — `PhraseRefusal` is not `#[non_exhaustive]`; the crate's convention is mixed

Measured across `ms-codec`'s public enums and structs:

| item | `#[non_exhaustive]` |
| --- | --- |
| `error::Error` | YES |
| `payload::PayloadKind` | YES |
| `payload::Payload` | YES |
| `inspect::InspectReport` (struct) | YES |
| `inspect::InspectKind` | no |
| `codex32::Error` | no |
| `codex32::field::Error` | no |
| `codex32::Case` | no |
| **`hashlock::PhraseRefusal`** (new) | **no** |

So this is not a departure from a universal convention — 4 of 9 carry it. The
decision is also defensible on the merits and is recorded: `ms-cli`'s delegation
matches all five variants with **no wildcard arm**, which means a future sixth
variant is a compile error in `ms-cli` rather than a silent fall-through to a
wrong refusal sentence — `#[non_exhaustive]` would force the wildcard and lose
that. The CHANGELOG states it outright (*"nothing was removed, renamed or made
non-exhaustive"*), and under the pre-1.0 convention the second component is the
breaking axis, so a new variant is expressible as 0.10.0. Recorded as checked,
not as a defect.

### N-2 — two `HASHLOCK_PHRASE_MAX_CHARS` constants now exist, and the guard is indirect but real

The diff leaves `ms-cli`'s constant in place (`hashlock_phrase.rs:24`, now used
only to render the `TooLong` sentence and in one test) while adding the codec's
(`hashlock.rs:94`), which is the one the rule actually applies. There is no direct
equality assertion between them. Divergence produces a self-refuting message —
built with the codec at 99 and `ms-cli` still at 100:

```
$ python3 -c "print('a'*100,end='')" | ms hashlock --hashlock-phrase-stdin
error: the hashlock phrase is 100 characters; at most 100 are allowed
```

**But it cannot ship.** The same mutation reds the suite immediately, and the test
that catches it is named for exactly this:

```
FAIL [0.004s] ms-cli::hashlock_phrase_rule lockstep_100_and_101
  assertion `left == right` failed: 100 characters must be accepted:
  error: the hashlock phrase is 100 characters; at most 100 are allowed
```

A cap change in either direction is caught (the corpus pins 100 accepted and 101
refused). So the duplication is guarded by a named test, and the finding is only
that the guard is a test failure rather than a compile error. A one-line
`const _: () = assert!(ms_codec::hashlock::HASHLOCK_PHRASE_MAX_CHARS == HASHLOCK_PHRASE_MAX_CHARS);`
in `ms-cli` would make it the latter. Nit, non-blocking.

---

## 10. Closing counts

| severity | count |
| --- | --- |
| Critical | **0** |
| Important | **0** |
| Minor | 2 (M-1 CHANGELOG date, M-2 unzeroized `qr_text` — secret-handling, non-gating by ruling) |
| Nit | 2 (N-1 `#[non_exhaustive]`, N-2 duplicated cap constant) |

**GREEN.** Nothing blocks the publish of `ms-codec` 0.9.0.

The CHANGELOG's load-bearing claim — *"no behaviour change to any existing
verb"* — is not merely unrefuted; it is measured, twice, by two independent
controlled instruments over 1,200,153 inputs, with zero divergences. The seven
`qr_text` rows follow from the three constants and §8.6's normative text by
independent recomputation. Every name Tasks 2 and 5b will call exists with the
planned signature. Every test the diff adds fails on the defect it names, and the
one place the plan mis-attributed a mutation was already found and corrected by
the implementer — I reproduced both halves of that correction.

M-1 is worth handing to the release agent before the tag, since it is a one-word
edit and the word ships inside the crate. M-2 is worth a decision before the
publish rather than after, because the fix is a breaking signature change.

*Written by the reviewer as its final action. Read-only on the branch and on
master; nothing committed; both review worktrees and their target dirs removed;
the implementer's branch worktree left clean at `ca71516511db0b608a14c7dc3f82376424b107a8`.*

---

## Delta review — fold `1a4f4aa8`

**Verdict: GREEN. Both findings fixed as claimed; no new defect. 0 Critical, 0
Important, 1 Nit (N-3, new).**

Scope: `git diff ca715165..1a4f4aa8` only — 3 files, 43 insertions, 10 deletions.
Nothing else re-audited. Read-only on the branch; built and mutated in a fresh
detached worktree at `1a4f4aa8`, since removed.

```
CHANGELOG.md                              | 18 +++++++++++-------
crates/ms-codec/src/hashlock.rs           | 29 +++++++++++++++++++++++++++--
crates/ms-codec/tests/hashlock_qr_text.rs |  6 +++++-
```

The corpus is **not in the delta at all**, and its sha is unmoved:

```
$ git show 1a4f4aa8:crates/ms-codec/tests/vectors/hashlock-v0.8.json | sha256sum
4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4
```

### Q1 — does the fold fix M-1 and M-2 as claimed?

**M-1 — YES.** `CHANGELOG.md:7` is now `## ms-codec [0.9.0] — 2026-09-05`,
matching the H1 precedent's shape exactly (`## ms-codec [0.8.0] — 2026-09-05`).
The word `unreleased` is gone from the file. One line, nothing else in the header
touched.

**M-2 — YES, on all four sub-claims, each checked separately.**

| claim | evidence | verdict |
| --- | --- | --- |
| `qr_text` → `Zeroizing<String>` | probe bound it as `let _: fn(bool, &str) -> Zeroizing<String> = qr_text;` — a signature mismatch would be a compile error. Compiles and passes | **holds** |
| buffer `Zeroizing` **from the first byte** | `Zeroizing::new(String::with_capacity(…))` is constructed EMPTY and wrapped *before* any `push_str`. The phrase is pushed into an already-protected buffer; it never exists in an unprotected `String`. This is the substantive difference from wrapping a finished `format!`, which would have protected only the copy | **holds** |
| no unwiped intermediate copy | the reserved capacity is `HEAD.len() + method.len() + LABEL.len() + phrase.len()` and the final string is exactly `HEAD + method + LABEL + phrase`, so no `push_str` can reallocate and abandon a buffer holding a phrase prefix. Computed for the anchor row: reserved 122, final 122. Measured across 9 shapes (both methods, empty phrase, the 100-character cap, `:`-bearing, trailing-space, comma-bearing, multibyte): `capacity() == len()` on every one | **holds** |
| the sole caller adjusted | `grep -rn qr_text crates/ --include=*.rs` outside the codec module and its test returns **nothing** — the test is the only caller, and it was adjusted | **holds** |

Compilation is itself the proof of the load-bearing bound: `Zeroizing<Z>`'s
`Drop` requires `Z: Zeroize`, so `Zeroizing<String>` only builds because
`zeroize` implements `Zeroize for String` (which wipes the `Vec<u8>`'s capacity
region, not merely the length). The build is green, so the wipe is real and not a
type that silently does nothing.

The doc comment's one exclusion is correct on inspection: `method` is left an
unprotected `String` because it is rendered from three compile-time constants and
carries no phrase bytes.

### Q2 — does the delta introduce any new defect?

**No.** The three named risks were each tested rather than reasoned about.

**(a) A changed assertion that no longer asserts.** The delta changes exactly one
assertion, `assert_eq!(got, row.qr_text)` → `assert_eq!(*got, row.qr_text)`
(needed because `Zeroizing<String>: PartialEq<String>` does not hold; the deref
compares the `String` inside). **Control run:** change one byte of the output
(`hashlock v1` → `hashlock v2`) and the row assertion still fires —

```
test qr_text_matches_every_corpus_row ... FAILED
thread 'qr_text_matches_every_corpus_row' panicked at crates/ms-codec/tests/hashlock_qr_text.rs:55:9:
  left: "hashlock v2\nmethod: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32\nphrase: correct horse battery staple"
 right: "hashlock v1\nmethod: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32\nphrase: correct horse battery staple"
```

so it is a full byte-for-byte comparison, not weakened to a length or prefix
test. The file's other five assertions are untouched and reach `str`'s inherent
methods through the same `Deref` — the implementer's comment says so and the
diff confirms only the one line changed.

**(b) A `format!` of the phrase.** The `format!("hashlock v1\n{method}\nphrase:
{phrase}")` is **removed**; the surviving `format!` renders the method line from
the three constants only and never sees the phrase. No `format!`, `to_string`,
`println!` or other unprotected materialisation of the phrase remains in the
function.

**(c) A behaviour change to the seven corpus rows.** Recomputed all seven
independently at the new tip from `HASHLOCK_SALT` / `HASHLOCK_ITERATIONS` /
`HASHLOCK_DKLEN` and §8.6's template, then compared the corpus, the
recomputation and the implementation three ways:

```
REV delta: 7 rows byte-identical; capacity==len on every shape
test the_seven_rows_are_byte_identical_to_the_independent_recomputation ... ok
test one_allocation_sized_exactly_so_no_push_can_realloc ... ok
test the_return_type_is_zeroizing_string ... ok
```

The output string is `HEAD + method + LABEL + phrase` = `"hashlock v1\n"` +
method + `"\nphrase: "` + phrase, which is character-for-character the string the
removed `format!` produced. Byte counts 122/63/194/135/119/102/109 unchanged.

**Gates at `1a4f4aa8`**, all re-run in the review worktree under the repo's
`rust-toolchain.toml` pin (`1.85.0-x86_64-unknown-linux-gnu`, honoured by rustup
in the worktree):

| gate | result |
| --- | --- |
| `cargo build -p ms-codec --locked` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, zero warnings |
| `cargo nextest run --locked` | **562 tests run: 562 passed, 11 skipped** — unchanged from `ca715165` |

The test count is unchanged, which is the right outcome: the fold changed a
signature and an assertion, and added no test.

### N-3 (new, Nit) — the no-realloc property has no regression guard

The fold's most substantive claim — the buffer is sized so no `push_str` can
reallocate and abandon an unwiped copy of a phrase prefix — is **true today and
guarded by nothing that ships**. Measured: shortening `with_capacity` by
`phrase.len()` makes the buffer reallocate, and

```
one_allocation_sized_exactly_so_no_push_can_realloc ... FAILED
  capacity 188 != len 122 -- the buffer was resized, so an unwiped copy of a
  phrase prefix was abandoned
```

fires only in **my** probe, which is removed. The shipped suite does not notice:

```
--- and does the SHIPPED corpus test notice? (it should NOT: content is unchanged) ---
test result: ok. 3 passed; 0 failed
```

The content is identical either way, so `hashlock_qr_text.rs` cannot see it. A
later edit to the method line or the labels that forgets the capacity arithmetic
would reintroduce the exact defect M-2 was raised about, silently and green.

A four-line `assert_eq!(got.capacity(), got.len())` in
`the_worst_case_is_194_bytes` would close it. **Not blocking**: this is a
secret-handling property, which by the operator ruling of 2026-08-27 is never
Critical and never Important — logged for future optimization, exactly as that
ruling directs. Recording it because the ruling is a severity rule, not a
discovery rule, and because the fold's own doc comment asserts the property in
prose that no test defends.

### Delta counts

| severity | count |
| --- | --- |
| Critical | **0** |
| Important | **0** |
| Minor | 0 |
| Nit | 1 (N-3, no regression guard on the no-realloc property) |

**M-1 and M-2 are CLOSED.** The fold does what it claims, changes nothing about
the seven rows or the corpus sha, weakens no assertion, and leaves every gate
green. Combined with the base review, the branch stands at **0 Critical / 0
Important** and nothing blocks the publish of `ms-codec` 0.9.0 at
`1a4f4aa8a3b4b29e5e6f7a479b4c57bc0e073fea`.

*Delta review written as the reviewer's final action. Read-only on the branch;
nothing committed; the review worktree `h6-a-review2` and its target dir removed;
the implementer's branch worktree left clean at `1a4f4aa8`.*
