# R0 final round — `SPEC_constellation_cli_uniformity.md` @ `595e335`

**Lens (operator, verbatim):** *"Focusing on items which will render a plan
inaccurate or misleading to the implementer down the line."*

**Object:** the whole document, read as an instruction. Folds under review:
`720961d` (round 3's Importants) and `595e335` (round 3's Minors/Nits plus
round 2's two Nits).

**Reviewer:** independent context, worktree
`/scratch/code/shibboleth/_work/r0f/mnemonic-engrave` on `review/r0-final`.

**Scope, as briefed:** not elegance, not the design. D1–D5, C-1's resolution,
D6/`--expect`'s shape, the argv ruling and P5 M-7 are inputs, not proposals.
Every finding below is *"an implementer builds the wrong thing / builds nothing
/ burns a cycle finding out"*.

**Verdict up front: NOT GREEN — 0 Critical / 6 Important / 5 Minor / 1 Nit.**

All six Importants are in **§7's phase table**, which round 3 explicitly did not
examine (*"the phasing and `--expect` are OUT OF SCOPE and were not examined"*,
`R0-cli-uniformity-spec-round3.md:5`). None is a design defect. Four are missing
or mis-assigned scope; two are false capability claims that three rounds
confirmed **by grepping help text instead of running the binary**.

---

## What I re-verified and found TRUE (so the fold's budget is not re-spent)

Executed, not read. `md` 0.13.0, `mk` 0.13.0, `ms` 0.16.0, `mt` 0.1.0,
`mnemonic` 0.97.0, `me` 0.7.0.

- **The `mnemonic-transaction` transient-tree disease is genuinely cured.**
  `main` is `cf17591`; `#[test]` = **237**, `Refusal::new(` = **56**, `--qr`
  present, `mt encode -` works through a pipe. All reproducible from the
  ordinary checkout.
- **I swept for the same disease elsewhere and found none.** Every
  `mnemonic-gui`, `mnemonic-toolkit`, `descriptor-mnemonic`, `mnemonic-key`,
  `mnemonic-secret` and `mnemonic-engrave` citation resolves in the ordinary
  checkout on the default branch. No spec fact depends on a worktree.
- **§2a's GUI measurements, all twelve:** `SEPARATORS` at `md.rs:24`,
  `mk.rs:15`, `ms.rs:33`, `mnemonic.rs:47`; `default_value: Some("5")` at
  `md.rs:77`, `mk.rs:71`, `ms.rs:78`, `ms.rs:414`, `mnemonic.rs:332/1281/1960/2050`
  — 8 sites exactly. Drift gate docstring scopes itself to `mnemonic` only.
  `mt --help` has 0 `gui-schema`; the other four have 1 each.
- **§6f's D26 reading is correct in full**, and this is the finding round 3 got
  right. `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md:364-373`
  states D26 semantically (exit-5 = verified now or verifiable-by-reassembly
  later; exit-4 `VERIFY-ME` = bounded-distance substitution with no
  self-oracle), the manual itself calls standalone `md repair`'s unconditional 5
  the outlier, and `md-cli-non-chunked-single-string-repair-demote` is filed
  with companion entries in both `mnemonic-toolkit` and `descriptor-mnemonic`.
  The spec neither owns nor re-opens it. **Correct as written.**
- **§6g's `Class` enum:** `crates/me-cli/src/sysw/record.rs` — one `MdMk`
  variant plus exactly the nine others named. `mdmk_unconfirmed` and
  `seal::record::chunk_key` exist as described. `me sysw pack` has no `--expect`
  today (correct — it is new surface).
- **§6d's clap-echo leak is still live:** `mt encode --qr deadbeefcafe` →
  `error: invalid value 'deadbeefcafe' for '[-]'`, **exit 2**. The spec's
  reproduction is now exact (round 3 could only get the older wording).
- **§1:** `mt encode --qr <210-char hex>` → `REFUSED — §8.2f … (210 characters)`,
  **exit 1**, no echo. `ms encode --phrase "<12 words>"` → **exit 0**, 4 stderr
  lines, last one recommending `> file.txt`.
- **§2/§6a/§6b tables:** `mk` emits **0** `chunk-set-id:` lines even on a
  2-chunk card; `md`'s single emission site is `md-cli/src/cmd/encode.rs:172`
  inside the chunking arm; `mk decode` is a 5-field labelled table; `ms decode`
  is 3 labelled lines; JSON keys are `ms1`/`mk1_strings`/`phrase` with
  `schema_version` string vs number vs `schema` `"md-cli/1"`, `md` alone
  pretty-printed. Stderr on success: `md` 1 line, `mk` 1 line, `ms` 4 lines.
- **§3's mechanism, reproduced end to end.** `md encode … --force-chunked`
  straight into `me sysw pack` → **exit 4 on record 0** (the header). Strip the
  header and pass `--group-size 0` → **exit 0**. No `--no-passphrase` and no
  `--out` are needed for an `md1`-only payload, so **P3's "no flags" gate is
  satisfiable** and genuinely fails today. Good gate.
- **§7's journey counts, exact:** **18** argv call sites across **7** scripts;
  **7** tracked files under `design/journeys/` carry `chunk-set-id:` (5 `.txt`
  transcripts + 2 `.sh` drivers); `git ls-files design/journeys/out` returns
  nothing.
- **§8's `ms` counts, exact:** 76 test files, 31 referencing `--phrase`/`--hex`,
  276 test functions. **§8a:** `SPEC_engrave_transaction.md` trips the
  duplicate-section class **21** times.
- **§8's record-hygiene correction is right.** F-250's entry
  (`design/FOLLOWUPS.md:10732`) carries `DONE 2026-08-25` with a SHA, not
  `CLOSED`. The two-vocabulary warning is real and correctly stated.
- **Both structure gates clean:** 21 sections / 16 cross-refs / STRUCTURE OK;
  58 rows / 0 malformed.
- **§7's `path =` hazard is real:** `mnemonic-transaction` still has five
  worktrees (`git worktree list`), so `path =` is genuinely ambiguous.
- **A hypothesis I formed and DISPROVED, recorded so nobody re-runs it.** I
  expected P3 to refuse `mnemonic`'s five argv channels without delivering
  replacements — the exact hazard §6d raises for `ms combine`. It is not so: all
  five already ship a private channel (`bundle/convert/derive-child/restore
  --passphrase-stdin`, `electrum-decrypt --decrypt-password-stdin` and
  `--decrypt-password-file`), and `--decrypt-password` **already emits an
  argv-leakage advisory**. P3's ordering is safe. See M-5.

---

# IMPORTANT

## I-1 — `ms`'s grouping and separator work, which is §3's ENTIRE decisive measurement, belongs to NO PHASE

**The harm.** An implementer plans P0→P4 from §7, ships all five phases, every
gate green — and `ms encode`'s default stdout **still cannot be packed**. The
document's §3 is titled *"THE DEFAULT OUTPUT OF THREE TOOLS CANNOT BE PACKED"*
and its decisive table is three rows of `ms encode`. The fix for the third of
those three tools is in no phase's content and asserted by no phase's gate. This
is the single most implementer-harmful item in the document: it is not a wrong
instruction, it is the flagship instruction going missing.

**Location.** §7's table, `design/SPEC_constellation_cli_uniformity.md:844-848`.

- **P2** (line 846) — *"`ms` FIRST `-` on `combine` and `--in` on all eight
  verbs, THEN the argv refusal, THEN the 0600 `--out`. Plus this repo's journey
  drivers."* Channels, argv, `--out`. No grouping, no separator, no card.
- **P3** (line 847) — *"**`md`, `mk`** header off stdout, grouping to stderr,
  `--in`/`--out`. Plus **`mnemonic`**'s grouping surface AND its argv
  refusal …"*. `md`, `mk`, `mnemonic`. **`ms` is not named.**

**Proof that `ms` is in scope for the change.** §2a:112 —
*"`mnemonic bundle` carries `--group-size` defaulting to 5, `--separator`
accepting space, hyphen, comma, and `--no-engraving-card` — the same surface §6c
removes from **the other three**."* The other three are `md`, `mk`, `ms`. §6c's
own stderr table lists `ms` as the one already card-shaped, and §6c's
*"consequence operators must be told about"* paragraph is written **about `ms`**:
*"After D4, `ms encode --no-engraving-card`, and any pipeline using
`2>/dev/null`, yield no grouped form anywhere."*

**Proof that it is unowned.** Exhaustive — every phase reference in the file:

```
$ grep -nE '\bP[0-5]\b' design/SPEC_constellation_cli_uniformity.md
```

43 matching lines (`grep -cE`). Not one assigns `ms` grouping or `ms --separator` to a phase. Line 864
comes closest and gives the intent away while the table drops it: *"P2 before P3
is deliberate: the seed-phrase-on-argv hole is the finding with funds behind it;
the grouped default is a usability defect."* — i.e. the grouped default is P3's,
and P3's row names only `md`, `mk` and `mnemonic`.

**Proof the defect is live and unfixed by any other phase's work.**

```
$ ms encode --phrase "<all-abandon vector>" | me sysw pack --no-passphrase
me: record 0 (records count from 0) is not a form this container can place: …
$ echo $pipestatus
0 4 0

$ ms encode --phrase "<all-abandon vector>"
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f

$ ms encode --phrase "<all-abandon vector>" --group-size 0 | me sysw pack --no-passphrase
$ echo $pipestatus
0 0

$ ms encode --help | grep -A2 -- '--separator'
      --separator <SEPARATOR>
          Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5
          [default: space]
```

Nothing in P0's crate (`--in`/`--out`/`-`, argv guard, write gate, exit codes,
remedy text, `--expect` vocabulary) touches grouping — D5's crate is the *IO +
safety* layer, and §6c is presentation.

**What would make it green.** Add `ms` to P3's content cell (or to P2's, after
the safety work), and add a gate clause of the same shape P3 already uses for
`md`: *"`ms encode` into `me sysw pack` runs with no flags."* That clause fails
today at exit 4 and passes only when the work is done.

---

## I-2 — `ms combine` ALREADY reads stdin via `-`. The claim that it does not is the stated reason P2's ordering is "non-negotiable", and it makes a P2 gate clause unfailable

**The harm.** Three things at once. (a) P2's first listed item is to build a
channel that exists — a wasted implementation slice at the front of the phase
the spec calls *"highest safety value"*. (b) A P2 gate clause — *"`ms combine`
and `ms repair` each driven through the private channel"* — **passes today,
before any P2 work**, so it can never report that P2 was skipped. (c) The
recovery-path alarm that justifies the whole ordering is fictional, so an
implementer who reorders for a good reason will be overruled by a false one.

**Location.**
- §2:61, table cell — `` `-` for stdin | … | `ms` | 7 of 8 verbs — **not `combine`** ``
- §6b:307 — *"**So the real `-` gap is `md`'s four other verbs and `ms combine`**
  — two targeted additions, not a constellation rollout."*
- §6d:467, table cell — `` `combine` | **positionals ONLY — no `--in`, no `-`** ``
- §6d:470-476 — *"**`ms combine` is the ordering constraint that makes P2
  non-negotiable** … Refusing argv there before `-` and `--in` exist removes the
  **only** way to recombine split shares — the recovery path, the one that
  matters when everything else has failed. §7 P2 is ordered accordingly."*
- §7:846, P2 content — *"`ms` **FIRST** `-` on `combine` …"*

**Proof.**

```
$ ms split --phrase "<all-abandon vector>" -k 2 -n 3 > shares.txt      # exit 0
$ head -2 shares.txt | ms combine -
entropy: 00000000000000000000000000000000
phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
language: english (12 words)
kind: entr
$ echo $status
0
```

`ms`'s own refusal names the channel:

```
$ ms combine - < /dev/null
error: expected at least one share (positional or via stdin with '-')
```

And both P2 gate clauses pass on today's binary:

```
$ head -2 shares.txt | ms combine - >/dev/null ; echo $status        # 0
$ echo "<an ms1 string>" | ms repair --ms1 - >/dev/null ; echo $status  # 0
```

**How three rounds confirmed a false claim — this is lens item #1 exactly.**
Round 1 closed round 0's I-3 with *"`combine` alone has neither (**0 stdin
mentions in `ms combine --help`**)"* (`round1.md:38`), and round 3 re-confirmed
it as *"`combine` the sole exception (**0 hits**)"* (`round3.md:447`). The grep
is true — `ms combine --help` has 0 stdin/`-` mentions, I re-ran it — and the
inference from it is backwards. The capability ships; only the **help text**
omits it. A conclusion drawn from a literal string, for the second time in this
document's history.

**What would make it green.** §2:61's cell becomes `8 of 8 (combine's is
undocumented)`; §6d:467's cell becomes *"positionals, plus an UNDOCUMENTED `-`;
add `--in` and document `-`"*; the §6d:470-476 paragraph is retracted the way
this document retracts things elsewhere; P2's content item becomes *document*
`-` on `combine` and add `--in`; and the P2 gate clause is re-pointed at `--in`,
which does not exist today and can therefore fail.

---

## I-3 — `mt decode`, `mt verify` and `mt inspect` all REJECT bare `-`. The spec records `mt` as complete and tells P1 its diff is confined to two rulings

**The harm.** P1 is planned as the smallest phase on the strength of a false
"already done". The implementer ships P1 green and `mt decode -` — the ordinary
shell idiom, on the tool this spec generalises *from*, at the very verb an
operator uses to read a backup back — is still a clap usage error. The cycle's
stated goal is *"the user should not care if they are dealing with mk or md or
mt or ms"*; three of `mt`'s four verbs will still care. Worse, F-250 exists
*because this exact idiom failing was found worth filing* — and it was fixed on
`encode` only.

**Location.**
- §2:61, table cell — `` `-` for stdin | … | `mt` | default, plus bare `-` ``.
  Every other cell in that row is verb-qualified (*"`repair` only"*, *"all 5
  artifact verbs"*, *"7 of 8 verbs"*); `mt`'s is not, and reads as complete.
- §6b:301-308 — *"`-` as a positional — read stdin. Accepted and ignored where
  stdin is already the default (**F-250's fix in `mt` is the reference
  implementation**)."* then *"**So the real `-` gap is `md`'s four other verbs
  and `ms combine`**"* — `mt` excluded by name from the gap.
- §7:845 P1 content, and §7:930-932 — *"`mt` needs **zero** test changes for D3,
  D4 and §6c — it already implements all three. **Its diff is confined to the
  two rulings above.**"*

**Proof.**

```
$ echo x | mt encode -
mt encode: REFUSED — §8.2e, input is not a PSBT or a raw transaction (13 bytes)   # exit 1: the '-' was ACCEPTED

$ echo x | mt decode -
error: unexpected argument '-' found
Usage: mt decode [OPTIONS]
                                                                                  # exit 2
$ echo x | mt verify -      →  error: unexpected argument '-' found   exit 2
$ echo x | mt inspect -     →  error: unexpected argument '-' found   exit 2
```

`mt decode|verify|inspect --help` each show `Usage: mt <verb> [OPTIONS]` with no
positional at all. And F-250's own title scopes the fix:
`design/FOLLOWUPS.md:10732` — *"### F-250 — **`mt encode -`** is rejected as an
unexpected argument"*, `DONE 2026-08-25` — *"A hidden positional whose
`value_parser` admits only the literal `-`."* One verb.

**The real `-` gap, measured verb by verb.** Not *"two targeted additions"* —
**seven verbs across two tools**, and not the two the spec names:

| tool | bare `-` honoured | bare `-` REJECTED |
| --- | --- | --- |
| `md` | `repair` | `encode`, `decode`, `verify`, `inspect` — 4, as the spec says |
| `mk` | `decode`, `verify`, `inspect`, `repair` + `--keys -` | `encode` (see M-3) |
| `ms` | `decode`, `verify`, `inspect`, `derive`, **`combine`** | (flag-valued `-` on `encode`/`repair`/`split`) |
| `mt` | `encode` | **`decode`, `verify`, `inspect` — 3, owned by nobody** |

**What would make it green.** §2:61's `mt` cell becomes *"default; bare `-` on
`encode` only"*; §6b:307's sentence becomes *"the real `-` gap is `md`'s four
verbs and `mt`'s three"*; P1's content gains `-` on `decode`/`verify`/`inspect`
and §7:930-932's *"confined to the two rulings"* becomes three.

---

## I-4 — `mk`'s invalid-artifact `2 → 1` — by §6f's own words "the only code this cycle changes" — appears in no phase row, and P3's gate contains no `mk` clause at all

**The harm.** The one normative exit-code change in the entire spec is scheduled
nowhere, and the phase that owns `mk` cannot detect its absence. An implementer
closes P3 green having done **zero** `mk` work — no exit code, no grouping flip,
no separator restriction, no `--in`/`--out`, no card — because every clause of
P3's gate is about something else.

**Location.**
- §6f:674 — *"**`mk`'s invalid-artifact 2 becomes 1**, converging on
  `md`/`ms`/`mt` and removing its collision with `md repair`'s atomic-fail 2.
  **This is the only code this cycle changes.**"* No phase named.
- §7:844-848 — the string `mk` appears in exactly one content cell (P3, line
  847: *"`md`, `mk` header off stdout, grouping to stderr, `--in`/`--out`"*) and
  in **no gate cell**.
- P3's gate, verbatim: *"`md encode` into `me sysw pack` runs with no flags and
  no grep, on a CHUNKING policy; `mnemonic-gui`'s schema mirror regenerated; the
  7 goldens regenerated; `mnemonic`'s five secret-material argv channels each
  refused, named one by one."* — `md`, the GUI, goldens, `mnemonic`. No `mk`.

**Proof that `mk` has real, currently-undone work.**

```
$ mk encode --xpub <xpub> --origin-fingerprint 11223344 \
            --origin-path m/48h/0h/0h --policy-id-stub 11223344
mk1qp 4rtqp qqsq3 zg3ng sgjyv 6ylcp mpqyq sqygp qyqsq ygpqy qsqyq fz9jr …
mk1qp 4rtqp pua7x 7yyer ygk7f fagdz 9yxgr waxfz r70ql afdzr w5720 p4w2a …
        (grouped by 5 on stdout — exactly what D4 moves to stderr)
  stderr: note: stdout is watch-only — public keys only, cannot spend   (1 line, not a card)

$ echo notanartifact | mk decode - ; echo $status
2                                       (the code §6f rules must become 1)
```

Note this compounds with §6c:396-402, which already says D4 *"requires inventing
an engraving card for `md` and `mk`"* and that *"**P3 owns the card's contents,
and the plan must specify them**, or two implementers render them two ways."*
That instruction, too, has no gate clause behind it.

**What would make it green.** Name `mk`'s `2 → 1` in P3's content cell, and give
P3's gate one `mk` clause that fails today — e.g. *"`mk decode` on an invalid
string exits 1; `mk encode` stdout is ungrouped and its grouped card is on
stderr."*

---

## I-5 — P2's gate never asserts that `ms encode --phrase <seed>` refuses. The cycle's one funds-safety fix is ungated by the phase that owns it

**The harm.** P2 can close green with all four gate clauses satisfied and the
argv refusal **not implemented**. Migrating this repo's 18 call sites to stdin
is something an implementer does anyway (it is how the drivers keep working),
and it passes whether or not `ms` refuses. So the phase that carries §1's entire
motivation — *"The tool holding the most dangerous material in the constellation
has the weakest handling of it"* — has no clause that can fail if that is still
true at the end of it.

**Location.** §7:846, P2's gate cell, verbatim: *"round-trip vectors; `ms
combine` and `ms repair` each driven through the private channel; the 18 argv
call sites migrated; `me`'s remedy text still naming only channels that exist."*

Walk it: clause 1 is unchanged behaviour; clause 2 **passes today** (I-2);
clause 3 is a change to *this* repo's scripts, not to `ms`; clause 4 is a
property of `me`'s text, satisfied today. **P2's content names the refusal
(*"THEN the argv refusal"*) and P2's gate does not measure it.**

**Proof the thing that must change is still unchanged, and that a gate clause
for it would fail today:**

```
$ ms encode --phrase "abandon abandon … about" >/dev/null ; echo $status
0
```

**The spec already knows how to write this clause and wrote it for the other
tool.** P3's gate ends *"**`mnemonic`'s five secret-material argv channels each
refused, named one by one**"* — precise, enumerated, unfailable-only-if-done.
`ms` has eight verbs enumerated in §6d's table and gets no equivalent line. The
asymmetry is the defect.

**What would make it green.** Add to P2's gate: *"`ms encode --phrase <phrase>`,
`ms split --phrase`, `ms decode <ms1>`, `ms combine <share>` and `ms repair
--ms1 <ms1>` each refuse on argv, naming class and length without echoing;
`--allow-argv-secret` proceeds."*

---

## I-6 — `me sysw pack --expect` is given two incompatible owners, and its second normative refusal is gated nowhere

**The harm.** `--expect` is the whole of D6 and the whole of C-1's closure, and
it is the largest single piece of *new functionality* in the spec. A plan author
reading §7 gets contradictory answers about who builds it, and the likeliest
resolution — "P0 defines the vocabulary, someone else builds the flag" — leaves
P4's gate unsatisfiable when it is reached. That is precisely the failure mode
this spec caught for itself at §7:934-942 (the P2/P4 journey-driver blocker) and
did not re-check for `--expect`.

**Location and the contradiction.**
- **P0's content** (§7:844) gives four words: *"the `--expect` **kind
  vocabulary** per §6g"*, inside a cell whose subject is *"the shared crate …
  Extracted FROM `mt`/`me`"*. Per D5 that crate is *"depended on by all five"* —
  the five CLIs. **`me` is not one of the five**, so the crate cannot be where
  `me`'s flag lands.
- **P0's gate** (§7:844) — *"its own tests + an R0 round closing 0C/0I + the two
  `mnemonic` exit cells … + the in-memory-history question … measured"*.
  *"Its own tests"* = the crate's. Nothing about `pack`'s behaviour.
- **§6g:756** says *"The discriminant exists one level down and **P0 uses it**"*
  — implying P0 does build the flag.
- **P4's gate** (§7:848) is the only place the behaviour is exercised: *"a
  captured journey that regenerates, and that **FAILS when one producer is made
  to refuse**"* — which requires a finished `--expect` two phases after P0.
- **P1/P2/P3 contain no `me` work at all.** §2a's scope statement compounds it:
  it names `mnemonic-engrave` as affected *"**through its committed journey
  drivers**"* and gives no other reason, so the repo's own binary gaining a flag
  is absent from the scope section too.

**Proof it is new surface, not existing:**

```
$ me sysw pack --help | grep -c expect
0
```

**And §6g's second normative refusal has no gate anywhere.** §6g bullet 3:
*"**When `--expect` names a kind, an INCOMPLETE chunk set of that kind must
REFUSE rather than warn.** Without this, C-1's smaller sibling survives."* P4's
gate tests *one producer refusing*, which is a different case (a missing record,
not a partial set). Search the phase table for any clause about an incomplete
chunk set: there is none. The sibling defect the spec names ships open.

**What would make it green.** Put `me sysw pack --expect` — flag, kind
vocabulary, absent-kind refusal, incomplete-set refusal — in **one** phase's
content cell by name (P0 reads best, since §6g already says P0 uses
`mdmk_unconfirmed`), say plainly that `me` is a sixth affected CLI in §2a, and
give that phase's gate two clauses: *"`--expect` naming an absent kind refuses
non-zero and writes no payload"* and *"1 of a 2-chunk `mk1` set under `--expect
cosigner` refuses"* — the second being the reproduction §6g already recorded as
exiting 0 today.

---

# MINOR

## M-1 — `--from-md1-set FILE` is assigned to P3 in §10 and is absent from P3's row and gate

§10:1119 — *"**`--from-md1-set FILE` is the one piece of new surface this
criterion introduces, and P3 owns it.**"* P3's content cell (§7:847) does not
mention it; P3's gate does not test it. Verified absent from the binary:
`mk encode --help` shows `--keys <FILE>` and `--from-md1 <FROM_MD1>`, no
`--from-md1-set`. A plan built from the phase table alone omits it and §10's
acceptance — which *"does not close until this has been RUN"* — cannot run.
Weaker than the Importants only because §10 does state the owner in words.
**Fix:** name it in P3's content cell.

## M-2 — §2:61 and §6b:304 credit `mk` with bare `-` on all five artifact verbs; `mk encode` rejects it

```
$ echo ZZZ | mk encode -
error: unexpected argument '-' found        (exit 64)
```

`mk encode` has no positional; its stdin channel is `--keys -`
(`mk encode --help`: *"Mint ONE card per key record in FILE (`-` for stdin)"*).
The cell reads *"all 5 artifact verbs, **and** `--keys -`"*, which double-counts
`encode`'s only channel. No work is missed (the channel exists), so this is
Minor rather than Important — but it is the same help-text-vs-behaviour error as
I-2 and I-3, in the third row of the same table. **Fix:** *"`decode`/`verify`/
`inspect`/`repair`, and `--keys -` for `encode`"*.

## M-3 — §8a:1057's verification command does not reproduce its own number

The spec gives the sweep as the thing that makes the claim checkable, then
states a figure the command does not print:

```
$ grep -oE '§[0-9][0-9a-z.]*' design/SPEC_constellation_cli_uniformity.md | sort | uniq -c | sort -rn | wc -l
19
$ grep -oE '§[0-9][0-9a-z.]*' design/SPEC_constellation_cli_uniformity.md | wc -l
50
```

**50 occurrences is right.** The distinct count is 19, not the stated 16,
because the character class `[0-9a-z.]` keeps a trailing sentence period —
`§6h.`, `§6g.` and `§2.` are counted separately from `§6h`, `§6g`, `§2`. Collapse
the trailing dot and it is 16, so **the claim is true and the command as written
contradicts it**. This is the third time this document has paired a number with
a command that prints something else (§8a's own text records the previous two).
**Fix:** `[0-9][0-9a-z]*[0-9a-z]` or a trailing `tr -d .`.

## M-4 — §7:910 gives the `mt` fast-forward as 8 commits; it is 9

```
$ git -C /scratch/code/shibboleth/mnemonic-transaction rev-list --count 95ef842..cf17591
9
```

The endpoints and every measurement drawn from them are correct — this is only
the commit count in the provenance note. It matters slightly more than a typo
because that paragraph is the spec's account of *how* the transient-tree defect
was cured, and a future reader checking that account gets a different number.
**Fix:** 9.

## M-5 — §2a understates `mnemonic`'s existing private-channel precedent, inflating how much P3 looks like new work

§2a cites `mnemonic bundle --passphrase-stdin` as *"an existing
in-constellation precedent"*, singular. Measured, all five of the channels P3
must refuse already have one, and one already warns:

| channel | private alternative today |
| --- | --- |
| `bundle --passphrase` | `--passphrase-stdin` |
| `convert --passphrase` | `--passphrase-stdin`, plus `--bip38-passphrase-stdin` |
| `derive-child --passphrase` | `--passphrase-stdin` |
| `restore --passphrase` | `--passphrase-stdin`, and `@env:VAR` |
| `electrum-decrypt --decrypt-password` | `--decrypt-password-stdin`, `--decrypt-password-file`, **and it already emits an argv-leakage advisory** |

Recorded because it is load-bearing in the *safe* direction: it is why P3 may
refuse all five without the `ms combine` hazard §6d raises, and it means P3's
`mnemonic` work is "add the refusal", not "add refusal + channel". Stating it
stops an implementer inventing `--in` flags that already have equivalents.
**Fix:** one sentence in §2a and a pointer from P3's row.

---

# NIT

## N-1 — §2's *"one emission site"* for `chunk-set-id:` is true of `md encode` and not of `md-cli`

`grep -rn 'chunk-set-id' descriptor-mnemonic/crates/md-cli/src/` returns three
hits: `cmd/encode.rs:172` (the `println!` the spec cites — correct, and the only
one on `encode`'s stdout), plus `cmd/vectors.rs:76`
(`format!("chunk-set-id: 0x{csid:05x}\n")`, the maintainer vector corpus) and
`format/text.rs:354` (a different `chunk-set-id=` rendering). Since §6a scopes
the stdout rule to `encode` only, nothing is wrong — but an implementer grepping
for the header during P3 finds three sites and has to re-derive which are in
scope. One clause — *"one emission site on `encode`'s stdout; `vectors.rs` and
`format/text.rs` render the header elsewhere and are out of scope per §6a"* —
saves that.

---

# Counts

| severity | count |
| --- | --- |
| Critical | **0** |
| Important | **6** |
| Minor | **5** |
| Nit | **1** |

**Verdict: NOT GREEN.**

**The single most implementer-harmful finding is I-1** — the grouping and
separator work for `ms`, which is the tool §3's decisive measurement is entirely
about, is in no phase's content and no phase's gate. A plan written from §7 ships
five green phases and leaves the defect the document exists to fix.

**What would make it green.** Six edits, all in §7 and all mechanical — no design
question is open:

1. **I-1** — add `ms` to a phase's content cell for grouping/separator/card, and
   add the gate clause *"`ms encode` into `me sysw pack` runs with no flags"*.
2. **I-2** — retract §6d:470-476, fix the §2:61 and §6d:467 cells, re-point P2's
   content and gate at `--in` (which does not exist) instead of `-` (which does).
3. **I-3** — qualify §2:61's `mt` cell to `encode`, correct §6b:307's gap
   sentence, add `-` on `decode`/`verify`/`inspect` to P1 and correct §7:932.
4. **I-4** — name `mk`'s `2 → 1` in P3's content and give P3's gate one `mk`
   clause.
5. **I-5** — give P2's gate an enumerated argv-refusal clause of the shape P3
   already uses for `mnemonic`.
6. **I-6** — assign `me sysw pack --expect` to exactly one phase by name, add
   `me` to §2a's scope statement, and gate both of §6g's refusals.

None of the six requires re-measuring anything: every fact needed is in this
report with the command that produced it. **The design is not in question and
was not examined** — D1–D6, C-1's resolution, the argv ruling and the D26 reading
all hold, and the mechanical layer this document has been hammered on for three
rounds is now genuinely clean. What is left is that the phase table was never
reviewed, and it is the part a plan is written from.
