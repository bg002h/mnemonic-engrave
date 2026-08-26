# R0 round 3 — `SPEC_constellation_cli_uniformity.md` @ `1baccfa`

**Lens (operator, verbatim):** *"focusing on mechanical damage and assumptions
without proof."* Two questions only. Design, the five operator decisions, the
phasing and `--expect` are OUT OF SCOPE and were not examined.

**Object:** `git show 1baccfa` (the round-2 fold), read against the whole
document.

**Verdict: NOT GREEN — 0 Critical / 3 Important / 4 Minor / 3 Nit.**

---

## A — MECHANICAL

**Summary: the document is structurally sound. The round-2 N1 repair holds.**

I did not take the gates' word for it. Independently re-derived, outside both
gates:

- **11 table blocks**, every one contiguous, uniform in column count, and
  carrying a valid separator row. Data rows total **58** — matching
  `plan-table-check.sh` by an independent route.
- The §6f exit-code table (lines 577–584) is **8 lines: header + separator + 6
  CLI rows**, with `me`'s row back **inside** it. Round-2 N1 is genuinely closed,
  not merely gate-clean.
- **Backticks balance in every paragraph**; no unclosed or nested fence
  (`fence balance ok`).
- **Headings run 1, 2, 2a, 3, 4, 5, 6, 6a…6i, 7, 8, 8a, 9, 10** — 21 sections,
  no duplicate, no level skip, no gap. `§6a..§6i` is complete and in order.
  `D1..D6` complete. `P0..P4` complete. `C-1..C-4` all defined.
- **15 distinct sigils**, enumerated by hand: §1 §2 §3 §4 §6 §6a §6b §6c §6d §6f
  §6g §6h §7 §8a §10. **Every one resolves to a section that exists.** No
  external sigil survives.
- **`design/SUPERSEDED_TERMS.txt`: 0 self-hits.** All 18 terms grepped against
  the spec individually; every count is 0.

Three cosmetic defects, all editing artifacts, none blocking.

### N-1 (Nit) — a mis-paired code span at line 512, introduced by the round-1 fold

`design/SPEC_constellation_cli_uniformity.md:511-512`

```
  not a measurement (round-1 B6, same defect as B2): `mt encode --quiet --in
  <the corpus `even` vector>` prints all **six** of that vector's strings and exits **0**. Its bearer-exposure warning fires on the *opposite*
```

Four backticks, so the paragraph *balances* and every gate stays green — but
they pair wrongly. The rendered result is the code span
`` `mt encode --quiet --in <the corpus ` ``, then the word **even** as plain
prose, then a second code span `` ` vector>` ``. The command reads as two
fragments with a bare word wedged between them.

Introduced by `5d37577` (the round-1 fold), confirmed by
`git log -L 505,520:design/SPEC_constellation_cli_uniformity.md`. Line 512 is
also the only non-table prose line over 92 characters (142), which is the visible
symptom of the same un-rewrapped edit.

Neither gate can see this: both count backticks or pipes, and this is a
*pairing* error inside a balanced count.

### N-2 (Nit) — §6f refers to itself in the third person

Line 610: *"§6f's freeze is retracted below"* — written inside §6f, twelve lines
above the bullet doing the retracting. "Below" is right; the sigil is redundant
and reads as a cross-reference to another section.

### N-3 (Nit) — the round-2 fold's rewording dropped a verb's object

The fold changed line 544 from *"the report appears only **without** `--quiet`"*
to *"the report **shows** only **without** `--quiet`"*. `shows` is transitive
here and has nothing to show. `appears` was correct.

### M-4 (Minor) — "three cells in the last row" now points at the wrong row

Lines 573–575:

> **Measured where the cell says a number; cells that say "not measured" were
> not run.** The header previously claimed every cell was run while **three
> cells in the last row** said otherwise …

The fold changed both halves of that sentence's referent and left the sentence
alone. Before the fold, `mnemonic` **was** the last row (that was the N1 defect —
`me`'s row sat outside the table) and it carried **three** unmeasured cells. At
`1baccfa` the last row is **`me`** — whose cells read `n/a`, not "not measured" —
and the `mnemonic` row carries **two**. A reader who follows the sentence to
"the last row" finds nothing that matches it.

This is the *"a diff falsifies text it never touches"* class: correct as history,
wrong as a pointer.

---

## B — ASSUMPTIONS WITHOUT PROOF

### I-1 (IMPORTANT) — every `mnemonic-transaction` fact in this spec is sourced from an unmerged branch in a worktree the spec itself calls transient

This is the round's headline finding, and it is the *third and fourth* instance
of the class the spec has already retracted twice.

The spec names its subject repo as `/scratch/code/shibboleth/mnemonic-transaction`.
That checkout is on branch **`main`** at **`95ef842`**. Its `mt` facts, however,
resolve only against **`/scratch/code/shibboleth/_work/p3b/mnemonic-transaction`**
— branch **`p3b/mt-record`** at **`cf17591`**, **9 commits ahead of `main`** and
merged nowhere.

The spec knows that worktree exists. §7 (lines 846–853) cites it by name as an
argument against `path =` deps, and calls it, verbatim, *"transient"*. It is also
the spec's measurement source, and nothing says so.

Nothing in the document lets a reader tell the trees apart — **both binaries
report `mt 0.1.0`**, and §7's version list pins `mt-cli` at `0.1.0`.

**Site 1 — §7, line 865-866, and the P1 gate at line 798.**

> The count is right — `grep -rc "#\[test\]"` over `mnemonic-transaction/crates`
> totals **236**, re-run during the fold

> | **P1** | … | `mt`'s **236** tests pass, **with the diff to them enumerated** … |

Run:

```
$ cd /scratch/code/shibboleth/mnemonic-transaction        # main, 95ef842
$ grep -ro '#\[test\]' crates | wc -l
212
$ cd /scratch/code/shibboleth/_work/p3b/mnemonic-transaction   # p3b/mt-record, cf17591
$ grep -ro '#\[test\]' crates | wc -l
237
```

`cargo nextest list --locked` on `main` enumerates **212** test cases;
`cargo test --doc` runs **0**. So the suite on `main` is 212. **236 is
reproducible in neither tree** — it is `p3b/mt-record` at a moment during the
fold, since overtaken. P1's gate is pinned to a number that exists nowhere.

**Site 2 — §6i, line 778.**

> Measured in `mt`: `Refusal::new(` is constructed at **56** sites, naming **12**
> distinct section-8 subsections of `SPEC_mt_v0_1`

```
main:  grep -ro 'Refusal::new(' crates | wc -l   ->  53
p3b:   grep -ro 'Refusal::new(' crates | wc -l   ->  56
p3b:   grep -rhoE '"§8\.[0-9]+[a-z]?"' crates/*/src | sort -u | wc -l  ->  12
```

**56 and 12 are both exactly p3b.** On `main` it is 53.

**Site 3 — §1, line 42.** *"`mt` returns only `ExitCode::SUCCESS` and
`ExitCode::FAILURE` (`mt-cli/src/main.rs:237,253,256`)"*

- p3b `main.rs:237,253,256` → `return ExitCode::FAILURE;` / `Ok(()) =>
  ExitCode::SUCCESS,` / `ExitCode::FAILURE`. Exact.
- `main` `main.rs:237,253,256` → `Refusal::new(` / `validate::secret_guard(…)` /
  `if let Some(w) = validate::file_mode_warning(…)`. Unrelated code.

**Site 4 — §6d, line 420.** *"Its own source says so at
`mt-cli/src/main.rs:219-238`: the guard sits on `std::env::args()` and runs
before `Cli::parse()`"*

- p3b `219-238` → `fn main()`, the §8.2f comment, `command_line_guard(&argv)`,
  with `Cli::parse()` at 240. Exact.
- `main` `219-238` → a `std::fs::read` error arm inside `encode`. The guard on
  `main` is at `main.rs:190`. *The substance holds on both branches; only the
  citation is branch-local.*

**Site 5 — §6d, line 428 — a command in the spec that does not run.**

> **That echo is still live and the fold reproduced it.** … `mt encode --qr
> deadbeefcafe` prints `error: invalid value 'deadbeefcafe' for '[-]'` and
> exits **2**.

```
$ /scratch/…/_work/p3b/…/mt encode --qr deadbeefcafe
error: invalid value 'deadbeefcafe' for '[-]'          exit 2      <- as written
$ /scratch/code/shibboleth/mnemonic-transaction/target/debug/mt encode --qr deadbeefcafe
error: unexpected argument '--qr' found                exit 2      <- on main
$ /scratch/…/mnemonic-transaction/target/debug/mt encode --help | grep -c -- --qr
0
```

**`mt encode` has no `--qr` on `main` at all.** (§9 also defers "F-247, the NFC-fit
line on `mt encode --qr`" — a flag that does not exist in the named checkout.)
The clap-echo *point* survives on `main` via `mt encode deadbeefcafe`
→ `error: unexpected argument 'deadbeefcafe' found`; the quoted reproduction does
not.

**Site 6 — §8, lines 935-937 — two of three "verified during the fold" behaviours
are absent from `main`.**

> the behaviour is present in the binaries — verified during the fold: **`mt
> encode -` works through a pipe**, **`mt`'s world-readable refusal carries the
> F-252 wording about only the file's own mode**, and `me`'s terminal refusal
> fires at exit 2.

```
$ echo "" | /scratch/…/mnemonic-transaction/target/debug/mt encode -
error: unexpected argument '-' found
$ diff <(main-build stderr) <(p3b-build stderr)          # 0644 input
- WARNING: … is mode 0644 — readable by other users on this machine.
+ WARNING: … is mode 0644 — its permissions grant read to group and others.
+   WHAT THIS DID NOT CHECK (F-252): only the file's own mode was measured. …
```

Both fixes are on the unmerged branch:

```
$ git merge-base --is-ancestor 5c7d827 HEAD ; echo $?   # F-250 -> 1 (NO)
$ git merge-base --is-ancestor 54c6d54 HEAD ; echo $?   # F-252 -> 1 (NO)
$ git branch -a --contains 54c6d54
  p3b/mt-record
  ship/tx-engraving
```

**Site 7 — §6e, lines 543-544 — the fold's own M3 re-measurement does not
reproduce.**

> (on the corpus `even` vector, `--quiet` gives **70** stderr lines against
> **108** without it, and none of `TX`/`CUT`/`PREFIX` appear)

Measured on the `even` vector (`crates/mt-codec/src/test_vectors/mt1_v1.json`,
`label: "even"`, `chunk_count: 6` — the **six strings** claim is TRUE), streams
separated to files, never through a pipe:

| stdout / input mode | `main` | `p3b/mt-record` | spec |
| --- | --- | --- | --- |
| file, input 0600 | 51 / 89 | 51 / 89 | — |
| file, input 0644 | 61 / 99 | 66 / 104 | — |
| pty | 49 / 87 | 54 / 92 | — |
| **as written** | — | — | **70 / 108** |

Width-independent (61 at `COLUMNS` 60/80/120/200). **Every** measured pair has
Δ = 38, so the shape is right and the absolute value comes from a third state.
The `--quiet` suppression itself is TRUE — `grep -cE '^(TX|CUT|PREFIX)'` on the
`--quiet` stderr is **0** in both builds.

This is the defect the fold was *fixing*. Round-1 B6 said a count without a named
input is not a measurement; the fold named the input and left the **build** and
the **input file's mode** unnamed — and each of those moves the number by 5–18
lines.

**What would fix I-1:** pin the `mnemonic-transaction` commit every `mt`
measurement was taken against, say so once at the top of the spec, and re-take
anything that will not reproduce there. P1's gate must cite a count that a reader
can run.

### I-2 (IMPORTANT) — §6f's retraction misdescribes D26, and the divergence it "discovers" is already recorded with an owning follow-up

The fold's headline change. Lines 606–611 and the P0 gate at line 797.

> **So D26's repair parity does NOT hold across five CLIs, and this spec must
> stop citing it as though it does.** The ruling was written when the
> constellation was four tools; `mnemonic` diverges, for a stated and defensible
> reason.

> | **P0** | … | … + **D26 restated against the measured 5-vs-4 repair
> divergence** + … |

**The measurement is TRUE. I reproduced it exactly:**

```
$ md repair       md1yqpqqzqq8xtwhw4xwn4qh   -> md1yqpqqxqq8xtwhw4xwn4qh, exit 5
$ mnemonic repair md1yqpqqzqq8xtwhw4xwn4qh   -> md1yqpqqxqq8xtwhw4xwn4qh, exit 4
      + "repair: correction UNVERIFIED — a non-chunked single-string md1 has no
         cross-chunk/content-id oracle …"
```

Identical correction, different code, UNVERIFIED banner present. And the input's
provenance checks out: `md encode --help`'s EXAMPLES block contains
`md1yqpqqxqq8xtwhw4xwn4qh` verbatim.

**What is unproven is everything the spec concludes from it.**

**(a) D26 is not a numeric-parity rule.** The spec never went and read it. Its
normative statement, in `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md:364`:

> The principled rule across all four CLIs (D26 of the v0.22.x follow-ups cycle,
> refined by Cycle E + Cycle F): exit-5 `REPAIR_APPLIED` means a correction is
> **verified now** … **or verifiable-by-reassembly later** … Exit-4 `VERIFY-ME`
> means a bounded-distance substitution correction that spent the checksum's
> error-detection budget and has **no self-oracle**.

D26 rules *semantics*, not a shared integer. Under it, `mnemonic`'s 4 on a
non-chunked single-string `md1` — no content-id oracle — **conforms**. So does
`ms`'s 4, which the spec itself calls "reasoned and load-bearing" at line 569-571
without noticing it is the same rule. **`md repair`'s unconditional 5 is the
non-conformant one.** The spec has the divergence pointed at the wrong tool.

**(b) "written when the constellation was four tools" is contradicted by the
spec's own block quote, forty lines above it.** Line 557-558 quotes
`md repair --help` verbatim, and I confirmed it byte-for-byte:

> Exit codes (**D26 cross-CLI parity** with `ms repair` / `mk repair` /
> **`mnemonic repair`**)

D26 **names `mnemonic repair` explicitly and in bold**. `mt` has no `repair`
verb (the table says `n/a`), so D26 never claimed five CLIs — "does NOT hold
across five CLIs" is a strawman, and the divergence is *inside* D26's own four.

**(c) It is not a discovery. It is a filed, OPEN, cross-repo follow-up with a
prescribed direction.** Slug `md-cli-non-chunked-single-string-repair-demote`,
mirrored in both repos:

- `descriptor-mnemonic/design/FOLLOWUPS.md:2095` (entry heading `:2093`) — *"this repo's `md-cli`'s own
  `repair` command was **deliberately left UNCHANGED** … The two CLIs now
  disagree on the exit code for the identical non-chunked md1 correction."*
- `mnemonic-toolkit/design/FOLLOWUPS.md:4974` (entry heading `:4971`) — *"**Fix (if pursued):** port the
  same demote predicate … into `md-cli`'s `repair` command, **flipping its
  exit-5-always claim to match the toolkit**. Needs its own R0-gated cycle in
  `descriptor-mnemonic`."*
- `42-md.md:373` already carries a **"Sibling-CLI divergence (v0.86.0)"**
  qualification on the D26 prose.

So P0's gate — *"a restated D26 that either admits `mnemonic`'s divergence as
deliberate or changes it"* — sends the plan to restate a ruling that already
accommodates `mnemonic`, in the wrong repo, without the R0-gated cycle the
existing record says the real fix needs. This is precisely the *"do not re-open
closed work"* hazard §8's own N-2 bullet raises.

**The retraction of the freeze survives** — the codes should not be frozen by
this spec — and *"this cycle renumbers no repair code"* is true. Only the stated
grounds and the gate need rewriting.

### I-3 (IMPORTANT) — §3's "102-byte payload" cannot be produced by the pipeline that row describes

Line 152, in the section headed **"THE DECISIVE MEASUREMENT"**, under a
document-level guarantee (lines 16-18) that *"Every measurement in this document
was re-run against the built binaries during the fold."*

> | `ms encode …` | piped into `me sysw pack` |
> | `--group-size 0` | **exit 0, 102-byte payload** |

The two exit codes in that table are TRUE — I reproduced all three rows:
default → **4**, `--separator hyphen` → **4**, `--group-size 0` → **0**. The byte
count is not.

```
$ ms encode --phrase "<BIP-39 test vector, 12 words>" --group-size 0 | me sysw pack | wc -c
118
$ ms encode --hex 000…0 (16 B) --group-size 0 | me sysw pack | wc -c   -> 118
$ ms encode --hex 000…0 (32 B) --group-size 0 | me sysw pack | wc -c   -> 143
```

**118 B is the floor.** An `ms1` is 50 chars at 128-bit entropy and only grows;
`me sysw pack` seals a secret record (`me sysw show` → `sealed: true, ct_len: 50`),
so the container cannot shrink below it. **No valid input to that row produces
102.**

Where 102 comes from is measurable:

```
$ 1 md1 record  | me sysw pack | wc -c  ->  76
$ 2 md1 records | me sysw pack | wc -c  -> 101
$ 3 md1 records | me sysw pack | wc -c  -> 126
```

**102 B is a two-record *public* payload** — which is exactly what §6g's C-1
reproduction describes (line 649: *"payloads of **1794 B and 102 B**"*, the
short one being the group with `mt` dropped out). The figure was transcribed from
§6g into §3, where it belongs to a different pipeline and a different class of
record. A reader checking either site is misled about which was run.

### M-1 (Minor) — §8's record-hygiene conclusion is an inference from a literal string, and it is backwards

Lines 933–941:

> **None of those five carries a `CLOSED` marker in `design/FOLLOWUPS.md`**,
> while F-244 does. The citations are correct about the code and **stale about
> the record**.

The literal claim is TRUE — grepping the token `CLOSED` finds it on F-244 only.
The conclusion drawn from it is not. Every one of the five carries a dated
resolution marker with commit SHAs:

| | marker |
| --- | --- |
| F-244 `:10153` | `CLOSED 2026-08-24` |
| F-246 `:10344` | `DONE 2026-08-25` — `08c9c80`, `9952c7f` |
| F-250 `:10732` | `DONE 2026-08-25` — `5c7d827` |
| F-251 `:10775` | `DONE 2026-08-25` — `6c3289b` |
| F-252 `:10842` | `DONE 2026-08-25` — `54c6d54`, `86854c6` |
| F-253 `:11015` | `DONE 2026-08-25` — `9ef69ee` |

The record is not stale; the marker *word* differs. And for **F-250 and F-252 the
polarity is exactly inverted** (see I-1 site 6): the record is accurate down to
the SHA, and it is the **code** that is missing from `mnemonic-transaction`
`main`. The spec schedules a sweep on a premise that is false in one direction
and reversed in another.

### M-2 (Minor) — the paragraph that argues against pinning a self-referential count pins one

Lines 891–904. Two sentences apart:

> **A self-referential count is a fact with a shelf life measured in commits.**
> … a documentation sweep whose size is whatever this prints at the time anyone
> asks — **31 as of this commit** …

The command is now written out in full and it **runs and returns 31** —
`git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l` → `31`, in both the
worktree and the main checkout. M2 is properly closed.

But the pinned figure is the exact construct the paragraph rejects, and it is a
commit-relative claim in a document that has retracted two of those. It falsifies
on the next commit that discusses the header — **including the commit that lands
this report**. Measured, not predicted: immediately after this report was
persisted, the same command returns **32**. The spec's pinned number was stale
before the round that produced it had finished. Either drop the number and keep
the command, or state the count with the SHA it was taken at.

### M-3 (Minor) — §6g's C-1 payload figures still carry no invocation

Line 649: *"identical pipelines differing only in whether `mt`'s input is valid
produced payloads of **1794 B and 102 B**, and **both exited 0**."*

The 102 B end is corroborated by my measurement (a 2-record public payload is
101 B), so this is not falsified — but neither producer, neither input, nor the
`me` build is named, and this is the evidence for **C-1**, the finding that
motivates D6 and `--expect`. The document holds itself to a higher standard four
sections earlier (line 667-668: *"each with the invocation that produced it — a
bare number is not a measurement"*) and at line 511 (*"naming the input because a
count without one is not a measurement"*). This site has not caught up.

---

## VERIFIED TRUE — checked, and not findings

Recorded so the next round does not re-derive them. Every one was executed.

- **The repair reproduction** (§6f): both tools return `md1yqpqqxqq8xtwhw4xwn4qh`,
  exits 5 and 4, UNVERIFIED banner present. Input is `md encode --help`'s own
  EXAMPLES string, corrupted at position 5.
- **The D26 block quote** (§6f lines 557-560) is verbatim from `md repair --help`.
- **The §4 provenance repair — the finding round 1 raised is properly closed.**
  *"stdout carries the artifact, stderr carries everything the human must see."*
  is at `design/SPEC_mt_v0_1.md:1660-1661`, character-for-character, and 1661
  falls inside `## 3b` (1490–1672). The citation is correct. (Note the
  near-identical line 119 reads *"a human"* — the spec quotes the right one.)
- **§6b's `--out` ruling:** `mt`'s refusal string *"mt has no --out: stdout IS the
  {}, by design (§3b)"* is at `validate.rs:680` (p3b) / `:657` (main), and §3b
  contains no `--out` or file-channel ruling. The spec's reading is right.
- **§6d's `mt` quote** *"md and mk DO take their strings as arguments; md1/mk1 are
  watch-only, so a leak there costs privacy rather than the money"* —
  `validate.rs:482`, on **both** branches.
- **§6b/§6d channel enumeration, verb by verb:** `ms` documents `-` on **7 of 8**,
  `combine` the sole exception (0 hits); `md` on **`repair` alone**; `mk` on **all
  five** artifact verbs plus `--keys`. Exactly as written.
- **§2's header claims:** `md encode 'wpkh(@0/<0;1>/*)'` emits **0**
  `chunk-set-id:` lines; the single emission site is
  `md-cli/src/cmd/encode.rs:172`, inside the chunking arm — the cited line is
  exact.
- **§2a's GUI measurements, all eight:** `SEPARATORS` at `md.rs:24`, `mk.rs:15`,
  `ms.rs:33`, `mnemonic.rs:47`; `default_value: Some("5")` at `md.rs:77`,
  `mk.rs:71`, `ms.rs:78`, `ms.rs:414` + four in `mnemonic.rs`. The drift gate's
  docstring does scope itself to `mnemonic` only. `mt --help` has **0**
  `gui-schema`.
- **§6g's `Class` enum:** `crates/me-cli/src/sysw/record.rs:44` is `pub enum
  Class {`, with a single `MdMk` and exactly the nine other variants listed.
- **§6g's two-record figure:** `md encode --group-size 0 --from-policy 'pk(@0)'
  --context segwitv0 --key '@0=<xpub>'` → **2** `md1` strings. And
  `md encode 'pk(@0)'` **is** refused: `template parse error: unsupported
  descriptor wrapper`, exit 1. The SHAPE note is correct.
- **§6g's empty-input guard:** `: | me sysw pack` → exit **2**.
- **§6h:** commit `956eea3` exists, dated 2026-08-26, titled *"fix: the argv
  remedy advised `ms encode --in`, which does not exist"*; `me`'s current advice
  is `ms encode --phrase - < seed.txt` (`main.rs:1993`). fish is **4.8.1**, zsh is
  **5.9.2**, `fish_history` is at `~/.local/share/fish/fish_history` in
  `- cmd:` / `when:` pairs. All as written.
- **§7's distribution facts:** `md-codec = "0.42"`, `mk-codec = "0.4"`,
  `ms-codec = "0.7"`, `mt-codec` on a git rev. Versions `md-cli` 0.13.0, `mk-cli`
  0.13.0, `ms-cli` 0.16.0, `mt-cli` 0.1.0, `me` 0.7.0. `write_private` at
  `crates/me-cli/src/main.rs:856`, in the binary crate.
- **§7's journey counts:** **18** `--phrase`/`--hex` call sites across exactly the
  **7** named scripts; **7** tracked files under `design/journeys/` carry
  `chunk-set-id:` (5 `.txt` transcripts + 2 `.sh` drivers);
  `git ls-files design/journeys/out` returns nothing.
- **§8's `ms` suite counts, all three exact:** **76** test files under
  `mnemonic-secret/crates/*/tests/`, **31** referencing `--phrase`/`--hex`,
  **276** `#[test]` functions.
- **§1's argv row:** `ms encode --phrase "<12 words>"` exits **0**;
  `mt encode --qr <tx hex>` prints the §8.2f refusal and exits **1** on *both*
  branches (the guard precedes clap on both).
- **§6f's collision:** `me sysw pack` has no `--expect` today (correct — it is
  proposed), and does carry `--allow-argv-secret`, `--allow-world-readable`,
  `--in`, `--out`.

---

## COUNTS

| severity | count | ids |
| --- | --- | --- |
| Critical | **0** | — |
| Important | **3** | I-1, I-2, I-3 |
| Minor | **4** | M-1, M-2, M-3, M-4 |
| Nit | **3** | N-1, N-2, N-3 |

**Verdict: NOT GREEN (0C / 3I).**

The trend holds — 4C → 0C → 0C, and the mechanical half is genuinely clean for
the first time: the tables, headings, fences, sigils and swept terms all survive
independent re-derivation, not just the gates. All three blockers are in the
second half of the lens, and all three are the same species: **a declarative
claim about how a tool behaves, that nobody ran in the place the spec says it
ran.** I-1 is systemic (eight sites, one branch), I-2 is a citation the fold
never opened, I-3 is a number transcribed between two pipelines.

None of the three touches the design. Every one is fixable by measurement.
