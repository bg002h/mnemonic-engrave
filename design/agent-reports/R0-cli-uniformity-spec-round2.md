# R0 — architect review, round 2 (fold verification)

**Artifact:** `design/SPEC_constellation_cli_uniformity.md` as folded at commit `5d37577`.
**Report under review:** `design/agent-reports/R0-cli-uniformity-spec-round1.md` (B1..B9,
0C/5I/2M/2N).
**Reviewer:** independent context, worktree
`/scratch/code/shibboleth/_work/r0r2/mnemonic-engrave` on `review/r0-round2`.
**Scope, as briefed:** exactly two questions — did the fold close B1..B9, and did the fold
introduce a new defect. No fresh audit. D1–D5, C-1's resolution, the 2026-08-26 argv
ruling and P5 M-7 are inputs, not proposals.

**Verdict up front: NOT GREEN. 0 Critical / 3 Important / 4 Minor / 2 Nit.**

Binaries executed (nothing below is read off help text alone):

```
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md          (md 0.13.0)
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic       (mnemonic 0.97.0)
/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/target/debug/mt
```

Note on invoking `md`: this shell aliases `md` to `mkdir -p`, and `/home/bcg/.cargo/bin/md`
is on `PATH` ahead of the build tree. Every `md` invocation below is the absolute path to
`descriptor-mnemonic/target/debug/md`; a reviewer who types `md` gets `mkdir`'s help.

---

# Part A — disposition of B1..B9

| ID | Result | Evidence |
| --- | --- | --- |
| **B1** | **CLOSED** | §6h bullet 5 is restated forward and now records the fix rather than the defect; §7 P2's gate reads *"`me`'s remedy text still naming only channels that exist"*. Tree agrees: `grep -n 'ms encode' crates/me-cli/src/main.rs` → `1993: "    ms encode --phrase - < seed.txt \| me sysw pack --out p.bin"`. The old advice is gone from the spec (`grep -cF 'ms encode --in seed.txt'` → 0) and is now a swept term. |
| **B2** | **CLOSED** | Both counts carry their invocation in a fenced block, the `--from-policy` shape is named, and the bullet states outright that `md encode 'pk(@0)'` is REFUSED. The `mt` figure re-measured here on the named fixture: **6**, not the placeholder (see B6). The `md` figure is the controller's pre-verified one and was not re-derived per brief. |
| **B3** | **PARTIAL** | The false cell is gone and the header no longer over-claims — that half is closed. But the measurement the fold deferred to P0 is one command, and **round 1 was right**: on a single-string `md1` that actually repairs, `md repair` exits **5** and `mnemonic repair` exits **4**. The spec now defers a settled fact while §6f still FREEZEs the codes over the table it sits in. See **N3**. |
| **B4** | **CLOSED** | P3's content cell enumerates all five channels by name (`bundle`, `convert`, `derive-child`, `restore --passphrase`, `electrum-decrypt --decrypt-password`) and §7 gains the prose block explaining why. The exposure now has an owning phase. Residue in **M4**, which does not re-open B4. |
| **B5** | **CLOSED — independently verified** | The `§8` at old line 714 is de-sigilled (`12 distinct section-8 subsections of \`SPEC_mt_v0_1\``), and §8a's document-wide claim is corrected. I listed and read every sigil myself: **47 occurrences on 41 lines, 15 distinct targets** (`§1 §2 §3 §4 §6 §6a §6b §6c §6d §6f §6g §6h §7 §8a §10`), and **every one resolves to a section of this document and is used to mean that section**. The gate cannot prove this; the read does. (The fold's own count is wrong — see N2b in Part B.) |
| **B6** | **PARTIAL** | The string count is fixed and correct for the named fixture: `mt encode --quiet --in <even>` on a pty → **6** strings, exit **0**. But B6's second half — the unattributed *"stderr is 82 lines"* four paragraphs down, which round 1 measured as 47/78 — was not touched and still carries no input. Measured on the fixture the fold names: **39 / 51 / 67**, never 82. See **M3**. |
| **B7** | **PARTIAL** | The line number is fixed and re-measured (`grep -n 'fn write_private' crates/me-cli/src/main.rs` → **856**), and replacing it with a grep-for-the-name instruction is the right call. B7's **second bullet** — §5's branch-relative warning — was not touched and is false on this branch. See **M1**. |
| **B8** | **CLOSED, with a defect in the substitute** | The de-pinning reasoning is **sound, not a dodge**: the count is provably self-referential (4 of today's 30 hits are this spec and its own three review reports), and the load-bearing figure is kept and re-verified — `git ls-files design/journeys \| xargs grep -l 'chunk-set-id:' \| wc -l` → **7**. What replaced the number is broken: see **M2**. |
| **B9** | **CLOSED — verified effective** | Four terms added; all four have **0 live occurrences** in the spec, so the sweep is clean on merit rather than by the retraction exemption. Positive control fires (below), and the four new terms turn **no other `design/` doc red** — the only reds sweeping `design/*.md` come from the pre-existing `base45` term, which is out of scope here. |

**Positive control for B9** — the gate is not green because it stopped checking:

```
$ printf '## 1. Probe\n\nThe cell reads 5 per D26 and the remedy is ms encode --in seed.txt today.\n' \
    > design/_probe_r0r2.md
$ ./scripts/spec-structure-check.sh design/_probe_r0r2.md
  FAIL  line 1: SUPERSEDED term 'ms encode --in seed.txt' appears as LIVE text
  FAIL  line 1: SUPERSEDED term '5 per D26' appears as LIVE text
  2 STRUCTURAL DEFECT(S)
$ rm design/_probe_r0r2.md
```

**Both gates re-run on the folded spec, and both are green for the right reason on
everything except N1:** `spec-structure-check.sh` → `STRUCTURE OK` (21 sections, 15
cross-refs); `plan-table-check.sh` → 57 rows, 0 malformed.

---

# Part B — defects introduced by the fold

## N1 — IMPORTANT — §6f: the fold inserted 21 lines of prose INSIDE the exit-code table, and `me`'s row fell out of it

The B3 response was placed between the `mnemonic` row and the `me` row. Those two rows were
adjacent before the fold and are not now:

```
$ git show 5d37577^:design/SPEC_constellation_cli_uniformity.md | sed -n '573,574p'
| `mnemonic` | 64 | not measured | 5 per D26 | not measured |
| `me` | 2 | 4 = unplaceable record; 2 = terminal refusal | n/a | n/a |

$ sed -n '578p;597p' design/SPEC_constellation_cli_uniformity.md
| `mnemonic` | 64 | not measured | **not measured — see below** | not measured |
| `me` | 2 | 4 = unplaceable record; 2 = terminal refusal | n/a | n/a |
```

Nineteen lines of prose, and a blank line, now sit between them. Under GFM a table ends at
the first non-`|` line, so **line 597 is a table row belonging to no table** — it renders as
a literal line of pipes, not as a row:

```
$ python3 -c "
lines=open('design/SPEC_constellation_cli_uniformity.md').read().split('\n')
blocks=[];cur=[]
for i,l in enumerate(lines,1):
    if l.startswith('|'): cur.append((i,l))
    else:
        if cur: blocks.append(cur); cur=[]
for b in blocks:
    if not any(set(l.strip().strip('|').replace('|','').strip())<=set('-: ') for _,l in b):
        print('ORPHAN at lines', b[0][0], '-', b[-1][0]); [print('  ',n,l[:70]) for n,l in b]
"
ORPHAN at lines 597 - 597
   597 | `me` | 2 | 4 = unplaceable record; 2 = terminal refusal | n/a | n/a |
```

**Why it gates.** §6f is the section that rules *"The repair codes are FROZEN"* over this
table, and the bullet immediately below it says *"Two collisions **this table makes
visible**"* — the second of which is `ms repair`'s 4 colliding with **`me sysw pack`'s
unplaceable-record 4**. That collision is now argued from a row the table no longer
contains. A plan author transcribing §6f into P0's exit-code conformance vectors reads a
five-row table with `me` missing.

**Neither gate can see it, and both were run on this fold.** `spec-structure-check.sh`
resets its column count on the first non-`|` line, so an orphan row is never compared to a
header; `plan-table-check.sh` names it as a declared blind spot in its own output —
*"NOT covered: … tables with no separator row"*. This is the failure mode the brief names:
a gate green because the defect is outside what it checks.

**Fix:** move the B3 prose block below the `me` row. One-line edit; no content changes.

## N2 — IMPORTANT — the fold made a third cell unmeasured and left three sites saying "two", including P0's gate

The fold changed `mnemonic`'s repair-applied cell from `5 per D26` to `not measured`. That
row now reads **three** "not measured" cells:

```
$ sed -n '578p' design/SPEC_constellation_cli_uniformity.md
| `mnemonic` | 64 | not measured | **not measured — see below** | not measured |

$ grep -n 'two unmeasured\|two `mnemonic`' design/SPEC_constellation_cli_uniformity.md
618:- The plan **must fill the two unmeasured `mnemonic` cells** before P0 closes;
775:| **P0** | … | its own tests + an R0 round closing 0C/0I + the two unmeasured `mnemonic` exit cells filled + …
903:- **STILL OPEN — two `mnemonic` exit-code cells** in §6f.
```

All three were correct before the fold and are wrong after it, and the fold's own new header
sentence contradicts them from eleven lines away (*"three cells in the last row said
otherwise"*).

**Why it gates.** Line 775 is not prose, it is **P0's gate**. As written, P0 closes when
*two* cells are filled — and the cell that is over-counted out of the requirement is exactly
the one the same fold declared P0 owes (*"P0 owes the measurement, on a single-chunk `md1`
that actually repairs, before anything is frozen"*). The gate is satisfiable while the work
it exists to force stays undone. That is the same shape as round 0's I-10 and round 1's B4:
a requirement that a later reader satisfies by doing less.

**Fix:** three occurrences of "two" → "three", or drop the count and name the cells.

## N3 — IMPORTANT — settled: `mnemonic repair` exits 4 where `md repair` exits 5, so D26's five-CLI repair parity does not hold — and §6f freezes it anyway

The brief asked for a single-chunk `md1` that actually repairs. `md encode`'s own help
example is one. Corrupting one character in the data part and feeding the **same string** to
both tools:

```
$ MD=/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md
$ MN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic
$ GOOD=$($MD encode 'wpkh(@0/<0;1>/*)' --group-size 0)      # md1yqpqqxqq8xtwhw4xwn4qh
$ BAD=md1yqppqxqq8xtwhw4xwn4qh                              # one substitution at position 3

$ $MD repair "$BAD" ; echo "md=$?"
# Repair report
#   md1 chunk 0: 1 correction at position 3: 'p' -> 'q'
md1yqpqqxqq8xtwhw4xwn4qh
md=5

$ $MN repair "$BAD" ; echo "mnemonic=$?"
# Repair report
#   md1 chunk 0: 1 correction at position 3: 'p' -> 'q'
md1yqpqqxqq8xtwhw4xwn4qh
repair: correction UNVERIFIED — a non-chunked single-string md1 has no cross-chunk/content-id
oracle (the v0.35.0 single-string decode path skips it); a >4-error correction can alias to a
DIFFERENT valid descriptor undetectably — re-derive the wallet/address to confirm
mnemonic=4
```

Both applied the identical correction; the exit codes differ. `mnemonic`'s 4 is deliberate
and it says so on stderr — the same reasoning `ms` uses, which §6f already records as
*"reasoned and load-bearing"*.

**The condition the fold itself set is therefore met.** §6f, lines 593–595:

> If `mnemonic repair` really exits 4 where `md repair` exits 5, the parity that ruling
> asserts does not hold across all five CLIs, and D26 needs restating rather than citing.

It does. Three consequences stand in the folded text:

1. §6f quotes `md repair --help` verbatim as *"the existing ruling"*, and that quote names
   **`mnemonic repair`** as a parity participant. Measured, it is not one for single-string
   `md1`.
2. §6f rules *"**The repair codes are FROZEN.** D26's 0/5/2 and `ms`'s reasoned 4 are not
   touched by this cycle"* — freezing a scheme that records `ms`'s divergence and omits
   `mnemonic`'s.
3. The same section says the measurement is owed *"before anything is frozen"* (line 596)
   while ruling the codes frozen at line 608. The spec contradicts itself on whether the
   freeze has happened.

**Why it gates.** §6f's own justification for the table is round 0's I-4: *"otherwise two
implementers build two different tables, and one of them silently changes what `mnemonic
repair`'s callers read."* A P0 conformance vector asserting `mnemonic repair → 5` on a
single-string `md1` fails today. The fix is the shape §6f already uses for `ms`: record
`mnemonic`'s reasoned 4/5 split with its condition (4 when there is no cross-chunk oracle,
5 when there is), and either measure the remaining cells or drop the FREEZE's claim to
five-CLI coverage.

**Note on the fold's process reasoning, which is otherwise right.** Declining to transcribe
a reviewer's number is correct discipline. What does not follow is deferring a one-command
measurement into P0 and then building a FREEZE on the empty cell in the same section. The
sentence *"the controller could not reproduce it"* now reads as doubt cast on a number that
reproduces on the first try with the tool's own documented example as input.

## M1 — MINOR — §5's branch-relative warning was B7's second bullet and was not folded

```
$ sed -n '246,249p' design/SPEC_constellation_cli_uniformity.md
… Note that this work is newer than the branch
this spec is being folded on, so the shipped `me` binary is ahead of the fold
branch's source — a plan reading only this branch would conclude, wrongly, that
the gate is still bearer-only.

$ grep -n 'pub fn is_argv_forbidden' crates/me-cli/src/sysw/record.rs
105:    pub fn is_argv_forbidden(self) -> bool {
```

The code is present on this branch, so the sentence warns a reader about a hazard that no
longer exists. B7 listed this bullet explicitly; the fold's commit message addresses only
the line number.

## M2 — MINOR — §7's de-pinning replaced a wrong number with a command that silently prints a different wrong number

The B8 response ends: *"whose size is `git ls-files design | xargs grep -l` at the time
anyone does it."* That command has **no pattern**, so `grep` consumes the first filename as
the pattern. It does not error — it exits 0 and prints a plausible list:

```
$ git ls-files design | xargs grep -l | head -3
design/CONTINUITY_tx_engraving_2026-08-25.md
design/FORWARD_PLAN_post_experiment.md
design/IMPLEMENTATION_PLAN_P1_me_container.md
$ echo $?
0

$ git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l
30
```

A reader who runs the spec's own recompute instruction gets 3 where the answer is 30. The
de-pinning is sound (see Part A, B8); the replacement invocation is not one. Add the
pattern.

## M3 — MINOR — §6e's "stderr is 82 lines" is the other half of B6 and still has no input

Same paragraph class the fold corrected, four paragraphs down and untouched. Measured on the
fixture the fold names, all three channels, `--quiet`:

```
$ MT=/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/target/debug/mt
$ python3 -c "import json;d=json.load(open('/scratch/code/shibboleth/mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json'));open('even.hex','w').write(d['vectors'][0]['raw_hex'])"

$ script -qec "$MT encode --quiet --in $PWD/even.hex 2>$PWD/e3.err" /dev/null >/dev/null
$ wc -l < e3.err                                    # stdout on a pty
39
$ (umask 077; $MT encode --quiet --in even.hex > e2.out 2> e2.err)
$ wc -l < e2.err ; grep -c '^mt1' e2.out            # stdout a 0600 file
51
6
$ $MT encode --quiet --in even.hex > even.out 2> even.err ; echo "exit=$?"
exit=1                                              # stdout a 0644 file: REFUSED, §8.2h
$ wc -l < even.err
67
```

39, 51 or 67 depending on the destination; never 82. The **substantive** claim in that
sentence does reproduce — `grep -cE '^(TX|CUT|PREFIX)'` on the `--quiet` stderr is **0** in
every configuration — so only the count is loose, exactly as round 1 reported it.

## M4 — MINOR — P3 gained the five argv channels in its content cell but not in its gate, and the paragraph three lines below still calls P3 the usability phase

```
$ sed -n '778p' design/SPEC_constellation_cli_uniformity.md | cut -c1-120
| **P3** | `md`, `mk` header off stdout, grouping to stderr, `--in`/`--out`. Plus `mnemonic`'s grouping surface AND its

$ sed -n '794,795p' design/SPEC_constellation_cli_uniformity.md
**P2 before P3 is deliberate**: the seed-phrase-on-argv hole is the finding with
funds behind it; the grouped default is a usability defect.
```

P3's gate column verifies the chunking pipeline, the GUI mirror and the goldens — nothing
about the argv refusal that was just added to it. And the ordering rationale contrasts
P2 (argv, funds) with P3 (grouping, usability) while P3 now carries five seed-phrase and
password argv channels of its own. An implementer who ships P0–P2 and defers P3 as the
cosmetic phase leaves `mnemonic`'s largest exposure open, and the gate would not notice.

Filed Minor rather than Important because the prose block immediately above the sentence
does name those five channels as secret material, so the misdirection is locally corrected
for a reader who reads both. Fix is one gate clause plus one clause in the ordering
sentence.

## N-1 — NIT — §6e line 508: nested backticks break the code span, in a 142-character line

```
$ awk 'NR==508 {printf "%d chars: %s\n", length($0), $0}' design/SPEC_constellation_cli_uniformity.md
142 chars:   <the corpus `even` vector>` prints all **six** of that vector's strings and exits **0**. Its bearer-exposure warning fires on the *opposite*
```

The span opens at `` `mt encode --quiet --in `` on line 507 and is closed by the backtick
before `even`, so CommonMark renders `even` outside the code and reopens a span at
`` ` vector>` ``. The file is hard-wrapped at ~78 columns everywhere else. Use a different
delimiter for the placeholder (e.g. `<the corpus "even" vector>`) and rewrap.

## N-2 — NIT — the fold commit's "all 19 resolve internally" reproduces no measurable count

The conclusion is correct — I verified it independently and every sigil resolves internally
(Part A, B5). The number does not correspond to anything in the file:

```
$ grep -o '§' design/SPEC_constellation_cli_uniformity.md | wc -l          # occurrences
47
$ grep -c '§' design/SPEC_constellation_cli_uniformity.md                  # lines
41
$ grep -oE '§[0-9]+[a-z]?(\.[0-9]+)?' design/SPEC_constellation_cli_uniformity.md | sort -u | wc -l
15
```

47 before the fold as well, so it is not a pre-fold figure either. Recorded because the fold
that spent three findings on numbers-without-invocations put one in its own evidence of
having done the sweep. It lives in the commit message, not in the spec, and nothing depends
on it.

---

# Counts

**0 Critical / 3 Important / 4 Minor / 2 Nit**

Round-1 dispositions: **6 CLOSED, 3 PARTIAL (B3, B6, B7), 0 NOT CLOSED, 0 WRONGLY CLOSED.**

Of the three judgement calls the brief flagged: **B8's de-pinning is sound** (the count is
provably self-referential; the actionable 7 is pinned and re-verified) — only its
substitute command is broken. **B5's verification claim is true** and I re-derived it
sigil by sigil. **B3's decision not to adopt the number was defensible as discipline and
wrong as an outcome** — the measurement takes one command and confirms round 1.

# Verdict

**NOT GREEN — do not proceed to implementation.**

The fold is good work: B1, B2, B4, B5, B8 and B9 are genuinely closed, the retraction
machinery now has teeth (positive control fires, four new terms, no self-hits, no collateral
reds), and the two counts I could re-run on named fixtures reproduced exactly. Every one of
the three blocking findings is again in text this fold wrote.

**The single most important finding is N3.** The one measurement the fold deferred is the
one its own section names as the trigger for restating D26 — and it settles against the
spec: `mnemonic repair` exits 4 where `md repair` exits 5 on identical input. §6f freezes
the repair codes over a table that omits that divergence, and says in the same section that
nothing should be frozen until the measurement exists. N1 and N2 are both one-line
mechanical fixes in the same section.
