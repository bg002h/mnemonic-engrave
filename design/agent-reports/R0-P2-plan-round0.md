# R0 — `IMPLEMENTATION_PLAN_P2_ms_adopts.md`, round 0

**Reviewer:** independent opus agent. **Author:** a different agent.
**Date:** 2026-08-27. **Artifact:** `design/IMPLEMENTATION_PLAN_P2_ms_adopts.md`
at `review/p2` (`6c24e62`). **Subject code:** `/scratch/code/shibboleth/mnemonic-secret`
at `7c12f66`, clean tree, read-only.

## VERDICT

**2 Critical / 8 Important / 8 Minor / 0 Nit.** Not GREEN.

The plan's measurement discipline is genuinely good and most of it survived
re-measurement — see "What I re-measured and found TRUE" at the end, which is
long on purpose. What it did not survive is the question the brief asked:
*construct a way each row produces a wrong or unsafe outcome, or shows a gate
that cannot fail.* Two rows do, and both are in the argv/remedy work that is the
phase's reason for existing.

## HOW I MEASURED

- `cargo build --locked` in `mnemonic-secret` first (exit 0, no-op — the tree's
  build was already current), then every probe against
  `/scratch/code/shibboleth/mnemonic-secret/target/debug/ms` **by absolute
  path**. Never `/home/bcg/.cargo/bin/ms`.
- `cargo build --locked -p mnemonic-engrave` in the main checkout (exit 0,
  no-op) — `crates/` is byte-identical between `master` and `review/p2`
  (`git diff --stat master..HEAD -- crates/` is empty), so
  `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me` is the reviewed
  `me`.
- `cargo nextest run --locked` in `mnemonic-secret`: **414 tests run, 414
  passed, 5 skipped**, exit 0.
- Exit codes captured to variables or files and read directly, never through a
  pipe.
- **One correction to my own method, recorded because it nearly produced a false
  finding.** The tool shell here is **zsh**, which does not word-split unquoted
  parameter expansions. A first sweep run as `$MS $a < "$f"` reported exit 64 for
  every `-` form and would have contradicted §1.3. Re-run with `"$MS" "$@"`, all
  twelve exit 0 and §1.3 reproduces. Every number below is from the corrected
  form.

---

# CRITICAL

## C-1 — The argv guard's gate omits the `--flag=value` spelling, so the leak survives its own 56-row cross-product

**Where.** Row 5 (*the guard*), and §5 closure condition 5.

**What the plan prescribes.** Two layers: **flag-keyed**, "a static list of the
14 secret-bearing channels of §1.4, **matched as strings, no parse**"; and
**value-shape**, "(`ms1` by HRP, a BIP-39 phrase by wordlist, hex by charset and
length) **for material arriving positionally**". The gate is "a generated
cross-product, not a hand list: **14 channels × 4 spellings (canonical,
leading-space, trailing-space, UPPERCASE) = 56 rows**". Condition 5 restates it:
"No seed material reaches stderr for any argv carrying it, **in any of the four
spellings**, on any of the 14 channels — 56 rows, generated."

**The counterexample.** `--phrase=<seed>` is a **single argv token**. It is not
the literal string `--phrase`, so layer 1 does not match it. It is not a
positional, so layer 2 is not even scoped to look at it. Neither layer sees it,
and it is not one of the four spellings, so no row of the gate exercises it.

Measured today, against the tree's build:

```
ms encode --phrase="abandon abandon … abandon about"   -> rc 0, stdout: ms10e ntrsq qqqqq … 34v7f
ms encode --hex=00000000000000000000000000000000       -> rc 0, stdout: ms10e ntrsq qqqqq … 34v7f
ms derive <ms1> --passphrase=hunter2                   -> rc 0, 2 argv warnings
```

An implementation that generates exactly the 56 rows the plan prescribes passes
**all 56** and still puts a seed phrase on argv at exit 0.

**Why the second layer does not rescue it.** For `--passphrase` there is no shape
to fall back on at all — that is the reason §6d makes layer 1 "the primary
layer" in the first place: *"This is how a passphrase — arbitrary text,
indistinguishable from a filename — is caught at all."* For `--phrase=<seed>`,
even an implementation that ran the wordlist check over every token (not just
positionals) fails: the first whitespace-delimited word of the token is
`--phrase=abandon`, which is not in the wordlist.

**This is not a novel case; the donor names it explicitly.** From the very
implementation P0 extracted the crate from —
`crates/me-cli/src/main.rs:347-349`:

> `=`-joined tokens are split, because `--in=<ms1>` is one argv token and the
> secret is the right-hand half of it. Splitting on every `=` rather than the
> first costs nothing and cannot miss a shape.

and `argv_candidates` (`crates/me-cli/src/main.rs:350-358`) normalises
`trim` + `to_ascii_lowercase` **and** splits on `=`. The plan's other three
spellings — leading-space, trailing-space, UPPERCASE — are exactly `me`'s trim
and case-fold. The plan appears to have taken three of the donor's four
normalisations and dropped the one that is a **bypass** rather than a variant.

`grep -n 'phrase=\|=<\|equals\|joined' design/IMPLEMENTATION_PLAN_P2_ms_adopts.md`
→ **exit 1, zero hits.** The plan never mentions the form.

**Severity.** Seed material on argv at exit 0, on the one tool the cycle calls
the most sensitive, through the phase built to stop it, past the phase's own
generated gate. `ms` handles seed material; this is the class the brief said
outranks everything.

---

## C-2 — `me`'s shipped private-channel remedy is broken today, and row 8 records it as correct

**Where.** Row 8 (*the sibling remedy*), §5 closure condition 9, and SPEC §7's
P2 gate *"`me`'s remedy text still naming only channels that exist"*.

**What the plan claims.** Row 8: *"**Fails today only in the direction that
matters**: the current line is correct and must stay correct until `--in`
ships"*, and its gate is *"the advised line, extracted from `me`'s own stderr
and **RUN**, exits 0 and produces a payload — **the assertion `me`'s suite
already makes, retargeted**."*

**The counterexample — the emitted line, run verbatim.** `me` emits, to a
secret-class operator:

```
    ms encode --phrase - < seed.txt | me sysw pack --out p.bin
```

Run exactly as printed:

```
$ ms encode --phrase - < seed.txt | me sysw pack --out p.bin
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record.
rc = 4
ls: cannot access 'p.bin': No such file or directory
```

**It fails today.** `ms encode`'s default stdout is grouped
(`ms10e ntrsq qqqqq …`) and `me sysw pack` cannot classify a grouped `ms1` —
which is §1.7's own measurement, correctly recorded there and then contradicted
in row 8. The `--group-size 0` form exits 0 and writes 102 bytes
(reproduced exactly, mode `-rw-------`).

**Three consequences, in ascending order.**

1. **SPEC §7's P2 gate is violated on `master` right now**, not after P2. `me`'s
   refusal advises a *pipeline* that does not run. §6h's standing rule — remedy
   text must be executable, "reproduced by running the emitted line, never by
   reading it" — is being broken by the very repo that wrote the rule.

2. **Row 8's gate is unsatisfiable at row 8's position in the order.** Row 10
   (*the ungrouped stdout*) is what makes `ms encode`'s stdout packable, and it
   is **two rows later**. So the *new* advice — `ms encode --in seed.txt | me
   sysw pack --out p.bin` — also exits 4 when row 8 is gated. The plan's own
   rule is "No entry begins until the previous is green."

3. **The gate's stated basis does not exist.** "the assertion `me`'s suite
   already makes" — searched all 13 `.rs` files under `crates/me-cli/tests/`
   plus `crates/me-cli/src/` for (a) an invocation of an `ms` binary, (b)
   `seed.txt`, (c) `ms encode --phrase`, (d) `Use a private channel`. The only
   hits are `prop.rs:58` and `prop.rs:81`, where `"ms"` is an HRP string in a
   proptest generator, and `main.rs:2164`, the emitted literal itself. **No test
   runs the emitted line.** The scope of that negative is: those 13 test files
   and `crates/me-cli/src/`.

**The operator path this produces.** Someone pastes a seed on argv → `me`
refuses → they follow the printed remedy verbatim → exit 4, no payload, and the
seed is now *also* sitting in `seed.txt` on disk. The remaining affordance the
refusal itself names is `--allow-argv-secret`. A refusal whose remedy does not
run pushes the operator back onto the channel it just refused.

**Related, and it is the reason this survived:** the source comment at
`crates/me-cli/src/main.rs:2160-2163` asserts the opposite — *"`--phrase -` is
ms's shipped stdin idiom and **is verified to pipe into pack**."* Nothing
verifies it.

---

# IMPORTANT

## I-1 — §3's blocker is already discharged, and row 4's gate cannot fail

**Where.** §3 (*THE DEPENDENCY*), row 4 (*the pin*).

Three statements, all presented as measured:

| §3 says | measured at the reviewed commit `6c24e62` |
| --- | --- |
| "**`mnemonic-io-lib` is not on `origin/master`.** `git ls-tree -d origin/master crates/` lists `crates/me-cli` alone." | it lists **both**: `crates/me-cli` **and** `crates/mnemonic-io-lib` |
| "the fish purge recipe is being built into `remedy.rs` (today that module *describes* fish rather than prescribing it)" | `crates/mnemonic-io-lib/src/remedy.rs:127` — "**fish is now PRESCRIBED**, and its cost is printed with it — F-273"; `history_purge_recipes` returns `("fish", "history clear-session")` |
| "`write_private` moving into the crate from `me-cli` (**where it still sits** — `fn write_private` in `crates/me-cli/src/main.rs`)" | `git grep -n 'fn write_private' HEAD -- crates/` → one hit, `crates/mnemonic-io-lib/src/write.rs:45` |

`crates/mnemonic-io-lib` entered `master` at `1db1e81` (*"P0 IMPLEMENTED: rows
1-10 by an opus agent"*), **11:32 on the plan's own day**, and is an ancestor of
`origin/master`. `write_private` moved at `54b7943` (*"P1 row 6"*). Both were
true at the plan's authoring commit `37fa40b` (14:45) — I checked
`git ls-tree -d 37fa40b crates/` and `git grep 37fa40b` and the crate was already
in-tree while `write_private` was still at `me-cli/src/main.rs:1079` — and the
merge that published the plan, `6c24e62` (14:55), is the commit that falsified
them.

**The defect is not the staleness; it is what the plan built on it.**

- **Row 4's gate cannot fail.** It reads: *"`git ls-tree -d origin/master
  crates/` names `crates/mnemonic-io-lib` — it does **not** today, measured."*
  It does. Green before a line is written. That is the failure mode the brief
  named as this cycle's most costly.
- **The ordering rationale is void.** §3 says *"The consequence for ordering is
  the load-bearing part"* and derives the whole "crate-free work first" sequence
  from the pin being blocked. It is not blocked.
- The plan forbids a workaround it no longer needs: *"This plan does not design a
  workaround and must not be given one."* An implementer reading §3 literally
  waits for a push that has happened.

## I-2 — Row 6's prescribed override mechanism cannot satisfy its own gate on `encode` and `split`

**Where.** Row 6 (*the override*), §5 closure condition 7.

**What it prescribes.** *"When present, the layer removes **both** the override
and the admitted token from the argv handed to clap, and carries the material in
through the same internal path as `--in` content."* Its gate: *"`ms encode
--allow-argv-secret --phrase <seed>` exits **0** and emits stdout byte-equal to
`ms encode --in <the same phrase in a file>`."*

**The counterexample.** `encode` and `split` each declare a **required**
`ArgGroup` over `["phrase","hex"]` — `crates/ms-cli/src/cmd/encode.rs:27` and
`crates/ms-cli/src/cmd/split.rs:28`, both cited by the plan itself. Remove the
admitted token from argv and clap sees no group member:

```
$ ms encode
error: the following required arguments were not provided:
  <--phrase <PHRASE>|--hex <HEX>>
rc = 64

$ ms split -k 2 -n 3
error: the following required arguments were not provided:
  <--phrase <PHRASE>|--hex <HEX>>
rc = 64
```

And the narrower reading — remove only the *value* — fails too:

```
$ ms encode --phrase
error: a value is required for '--phrase <PHRASE>' but none was supplied
rc = 64
```

Either way, exit 64, not the exit 0 the gate demands. On the other six verbs the
prescription is fine (their positionals are optional and default to stdin); it
fails on precisely the two verbs that take a **raw seed**, which is where the
override matters most.

**The plan is stricter than the spec here, and that is what creates the
contradiction.** §6d rules only that admitted material is *"never re-presented to
clap **as a positional**"* — which leaves room for the layer to hand clap a
group-satisfying token while routing the material internally. The plan tightened
"never as a positional" into "removed entirely", and boxed itself in. I am not
prescribing the fix; I am reporting that the instruction as written cannot pass
its own gate.

## I-3 — P2 removes the only private route for `derive` from a phrase with a passphrase, and supplies none

**Where.** Row 1, row 3, §2.5, and the absence of any entry in §6 or §8.

**What `--in` binds to on `derive`.** SPEC §6d's per-verb table gives `derive`'s
channel as *"positional, or `-`, or omitted"* → *"add `--in`"*. So `--in` reads
the **ms1**. Confirmed against the binary — the positional is ms1-only:

```
$ ms derive - < a-file-holding-a-BIP-39-phrase
error: string length 82 not in v0.1 set [50, 56, 62, 69, 75]
rc = 1
$ ms derive --help
Usage: ms derive [OPTIONS] [MS1]
Arguments:
  [MS1]   ms1 string. Use `-` or omit to read from stdin
```

**The counterexample — an operation that works today and cannot be done
privately after P2.**

```
today:   ms derive --phrase "<seed>" --passphrase "<pass>"     -> rc 0 (2 argv warnings)
after P2 (row 5): both --phrase and --hex and --passphrase are argv-refused
private attempt 1: ms derive --phrase - --passphrase-stdin
                   -> rc 1  "cannot read both the entropy source and --passphrase
                             from stdin (one stdin per invocation)"
private attempt 2: ms derive --in <phrase-file> --passphrase-stdin
                   -> --in reads an ms1; a phrase is a length error
```

There is **no private form**. The only remaining route is
`--allow-argv-secret` — the exposure this phase exists to close. The same holds
for `--hex` + `--passphrase`.

**§2.5 asserts the opposite in as many words:** *"**`--in` is not only a
hardening measure here; it is the first private way to do two things at once**,
and that is the argument for doing it before the refusal rather than after."*
That is true for `verify --in card.txt --phrase -` and for
`derive --in card.txt --passphrase-stdin` — both of which row 3 gates, correctly,
and both of which fail at rc 1 today as the plan says. It is **not** true for the
phrase-plus-passphrase case, which is the shape an operator recovering from a
paper seed actually has. The residue is named nowhere: not in §6 OUT OF SCOPE,
not in §8's F-281…F-286, not in a closure condition.

## I-4 — Row 10's gate and closure condition 11 are unsatisfiable as written, and the spec's own wording is right

**Where.** Row 10 (*the ungrouped stdout*), §5 closure condition 11.

Row 10's gate: *"`ms encode --phrase <the all-abandon vector>` with **no flags**
piped into `me sysw pack` exits **0** and writes a payload."*

Row 5 (**four rows earlier**) makes a real BIP-39 phrase on argv a refusal. The
all-abandon vector is a real BIP-39 phrase. **The gate's own invocation is
refused by the work that precedes it.**

Condition 11 is worse: *"`ms encode` with no flags, piped into `me sysw pack`
with no flags, exits 0 and writes a payload."* Measured:

```
$ ms encode
error: the following required arguments were not provided: <--phrase <PHRASE>|--hex <HEX>>
rc = 64
```

`ms encode` with no flags cannot exit 0 under any implementation, because it
names no input channel.

**The spec had it right and the plan diverged from it.** SPEC §7's P2 gate reads
*"**`ms encode --in <file>` piped into `me sysw pack` runs with NO flags and
exits 0 (I-1)**"* — `--in <file>`, and "no flags" scoped to `me sysw pack`. The
plan restated it twice and lost the channel both times.

This is the same defect class the plan diagnoses in §1.9 about the spec's *"18
argv call sites"*: a gate that, read literally, no implementation can satisfy.

## I-5 — Row 9 asserts a false premise about `ms repair`'s stdout, and its gate cannot tell the two readings apart

**Where.** Row 9 (*the private write*).

**What it claims.** *"`--out FILE` on `encode`, `split` and `repair`. **The three
verbs whose stdout IS a canonical artifact**"*, explicitly contrasted with
*"`combine`/`derive` emit labelled reports rather than artifacts"*.

**The counterexample.** `ms repair` emits **both**:

```
$ ms repair --ms1 <an ms1 with one induced error>
# Repair report
#   ms1 chunk 0: 1 correction at position 1: 'f' -> 'e'
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
rc = 4
```

Two comment lines and then the artifact. So `--out` on `repair` has an
unspecified meaning, and **row 9's gate cannot distinguish the choices**: it
asserts only that the file ends at mode `0600` "holding the new artifact", and
never pins the bytes. Both implementations pass:

- write the whole stdout → the operator's `--out` file begins `# Repair report`,
  and an engraving path that reads the file expects an `ms1`;
- write only the corrected `ms1` → the correction report, which is the only
  record of *what changed on a card the operator is about to trust*, is silently
  dropped from the artifact they keep.

`repair` also **exits 4** on an applied correction, so `--out` on `repair` is the
one case where a non-zero exit accompanies a written file. Row 9 exercises only
`encode`.

`write_private` itself is sound and I verified it — mode on create *and*
`set_permissions` on the **open handle** before `write_all`, so there is no
window where the file holds bytes at 0644 (`crates/mnemonic-io-lib/src/write.rs:45-66`).
The defect is entirely in what row 9 hands it.

## I-6 — Row 7 requires a verb-qualified purge command built pre-parser, without the allowlist the donor documents as load-bearing

**Where.** Row 7 (*the purge text*), §5 closure condition 8.

**What it prescribes.** *"**The `command` argument is the VERB-QUALIFIED
invocation — `ms encode`, not `ms`** — because the recipe is a word-bounded
`sed` pattern and a two-character command name is a collision generator."* The
reasoning is right and the crate's doc backs it exactly (`remedy.rs:65-70`:
`\bme\b` "left four [of six], removing only the invocation and `cd /home/me`").

**What it omits.** The recipes interpolate `command` straight into a shell
command the operator is told to run:

```rust
format!("fc -W; sed -i '/\\b{command}\\b/d' \"$HISTFILE\"; h=$HISTSIZE; HISTSIZE=0; HISTSIZE=$h; fc -R")
```

and the guard runs **before clap has resolved the subcommand**, so the verb is
whatever token happens to sit at `argv[1]`. `me` solved this and says why, at
`crates/me-cli/src/main.rs:400-404`:

> **The words come from an ALLOWLIST, and that is the whole safety argument.**
> Deriving them instead — "leading tokens that classify as `Unknown`" — would
> admit a TRUNCATED or otherwise unparseable secret into the pattern […] An
> allowlist of `me`'s own eight subcommand words cannot carry material at all.

`grep -in 'allowlist\|allow-list'` over the P2 plan → **exit 1, zero hits.**

**The counterexample.** A mistyped verb — `ms encoed --phrase "<seed>"` — is
still argv carrying a seed, so the guard must refuse and print a purge block. Its
`command` is `ms encoed`, so the emitted recipe is
`sed -i '/\bms encoed\b/d' "$HISTFILE"`. The operator runs it, `sed` exits 0, and
**the history line holding the seed is untouched.** A remedy that reports success
and purges nothing after a seed leak is the exact shape this repo's own doc calls
the trap (`remedy.rs:145-160`, on `history -d`).

Condition 8 does not catch it: *"Every purge recipe `ms` emits has been RUN […]
and observed to purge"* is satisfied by the canonical verbs.

## I-7 — The two `me-cli` citations are stale by 24 lines and land on unrelated text

**Where.** Row 8.

The plan cites `crates/me-cli/src/main.rs:2188` as the line emitting
`ms encode --phrase - < seed.txt`, and `:2184` as "the comment four lines above"
stating `ms encode --in` DOES NOT EXIST. Resolved on `HEAD`, `origin/master` and
`master` alike (`git grep -n 'ms encode --phrase - < seed.txt' <ref>`):

| plan | actual | what is really on the cited line |
| --- | --- | --- |
| `main.rs:2188` — the emitted line | **`main.rs:2164`** | `If argv is safe where you are -- a single-user \` |
| `main.rs:2184` — the DOES-NOT-EXIST comment | **`main.rs:2160`** | `Use a private channel instead:\n      \` |

The relative offset of 4 is correct, so the author measured a consistent tree —
just not this one. `plan-cite-check.sh` reports both `ok`, because it checks only
that a line exists: **F-279's exact shape**, in the one row that edits this repo.

The plan's own guarantee does not cover them. §4 says *"**EVERY
`crates/ms-cli/` LINE NUMBER ABOVE IS ANCHORED AT `7c12f66` AND NAMES ITS
SYMBOL**"* — scoped to `ms`, and that scope held: I resolved all 22
`crates/ms-cli/` citations and every one landed on exactly what the plan says
(see the TRUE list below). The two anchors into *this* repo got neither the
symbol treatment nor the anchor.

## I-8 — P2 ships a 0600 `--out` while `ms`'s own unconditional advisory keeps recommending `> file.txt`

**Where.** Row 9 adds `--out`; nothing in the plan touches `ms`'s advisory, and
no follow-up covers it.

`ms encode` prints this on **every** invocation (measured, stderr):

```
warning: stdout carries private key material (can spend) — redirect or encrypt
         (e.g. '> file.txt' or '| age -e ...')
```

and, measured under the default umask 022:

```
$ ms encode --phrase - < seed.txt > w.txt
$ ls -la w.txt
-rw-r--r--  … w.txt          # 0644, holding an ms1 that decodes to the seed
```

After row 9, `ms encode --out w.txt` gives **0600** for the same artifact. So the
tool's own shipped advice points the operator at the strictly weaker of two
in-tool channels, and P2 is the phase that creates the better one.

This is the identical obligation the plan applies to `me` in row 8 — §6h's rule
that remedy text must name the channels that exist — applied to `ms`'s own text
instead of the sibling's. It is **not** F-281 (whether to *gate* a world-readable
stdout) and **not** F-285 (verbs that get no `--out`).

I am not prescribing that the wording change in P2. There is a real constraint
against it and the plan states it: `crates/ms-cli/tests/cli_output_class.rs:56`
(`fn byte_parity_advisory_lines`) pins these lines byte-for-byte against
`mnemonic-toolkit`, so the change is joint work. But the gap needs an owning
phase, and today it has neither a row, a condition, an out-of-scope bullet, nor
a follow-up.

---

# MINOR

**M-1 — "THE SIX SINGLE-CHANNEL VERBS" contradicts §1.4's own measurement.**
Row 1's title calls `decode`, `verify`, `inspect`, `repair`, `derive`, `combine`
single-channel. §1.4, four pages earlier, measures `verify` at **2** channels
(positional, `--phrase`) and `derive` at **4** (positional, `--hex`,
`--phrase`, `--passphrase`) — and makes a point of it: *"The argv surface is 14
channels across the eight verbs, not 8."* The plan corrects the spec's
understatement and then re-adopts it in the row that acts on it. No ambiguity
about what `--in` binds to remains — §6d settles it as the positional — but I-3
above is what that understatement hides.

**M-2 — §1.3 says "eleven invocations" and lists twelve.** `encode --phrase -`,
`encode --hex -`, `decode -`, `decode`, `verify -`, `inspect -`,
`repair --ms1 -`, `split --phrase -`, `split --hex -`, `combine -`, `derive -`,
`derive` = 12. All twelve do exit 0 (re-measured).

**M-3 — The journey-driver table's column is mislabelled, and the residue figure
is wrong.** The column reads "`"$MS"` occurrences" and the method is given as
`grep -n`, which counts **lines**. Measured both ways:

| script | lines (`grep -c`) | occurrences (`grep -o \| wc -l`) | plan says |
| --- | --- | --- | --- |
| `derive-rcw-keys.sh` | 3 | **4** | 3 |
| `derive-hashvault-keys.sh` | 2 | **3** | 2 |
| **total, 7 scripts** | **18** | **20** | 18 |

The plan notices the anomaly — *"Two lines each carry two nested invocations,
which is why one script's material count exceeds its occurrence count"* — and
explains it instead of re-measuring. Consequence: *"**5** of the 18 are not
invocations of material at all"* is wrong; the plan's own enumerated residue
(2 `[ -x ]` + 3 `--version` + 2 `--phrase -`) sums to **7**, which is 20 − 13.
Row 12's *"The 5 non-material occurrences are left alone and named"* is the
sentence a later reader audits against. **The actionable number is right:** I
walked all 20 invocations and 13 carry material.

**M-4 — Condition 15's enumeration scope excludes a test row 11 must change.**
It reads *"The diff to `ms`'s **276 integration tests** is enumerated"*. Row 11
removes the `hyphen` and `comma` arms from `parse_separator`, and
`crates/ms-cli/src/format.rs:197` (`fn parse_separator_keyword_and_literal`)
asserts `parse_separator("hyphen").unwrap() == '-'` and
`parse_separator("comma").unwrap() == ','`. That is one of the **146** `#[test]`s
in `src/`, outside the 276 the condition enumerates.

**M-5 — The `channel::destination` divergence: right verdict, wrong reason.**
The brief asked me to decide this one. **P2's DECLINE is correct for `ms`, and
`mt`'s ADOPT is correct for `mt`** — the crate's boundary is not at fault here
and F-276 needs no third site on this item. But P2's stated discriminator is
falsified by the sibling. P2 says: *"the other two arms exist to feed
`write_block`, which is declined."* P1's own verdict table
(`IMPLEMENTATION_PLAN_P1_mt_adopts.md:133-140`) shows `mt` **declining**
`exit::write_block` **and** `exit::WriteBlock` while **adopting** `destination`
— so "write_block is declined" cannot be what separates them. The real
discriminator is stated elsewhere in P2 and should be the one carried: `mt` has a
world-readable-stdout gate (§8.2h) and a terminal policy, so it has something to
map `Stream` and `Terminal` onto — P1 row 10 says exactly that, *"with `mt`
mapping `Terminal` onto its own permissive policy rather than `me`'s refusal"* —
whereas P2 builds **no** stdout gate (§6, first bullet) and row 14 pins its
absence, so both non-`File` arms would be dead in `ms`.

**M-6 — Row 5's leak assertion is unsatisfiable as literally written.** *"that
the material's own **characters** never appear in stderr."* A 12-word BIP-39
phrase shares characters with every English sentence `ms` can print; the
canonical refusal itself would fail. It presumably means tokens or substrings,
and a generated 56-row harness is exactly where a literal reading gets
implemented.

**M-7 — Exit code 4 is asserted in §5 but absent from §1.1's measurement.**
§1.1 gives *"Measured codes: clap usage **64**, invalid artifact **1**,
repair-uncorrectable **2**."* Condition 16 then lists *"clap 64, invalid artifact
1, repair-uncorrectable 2, **repair-applied 4**"* as if §1.1 had measured it. It
is 4 (verified: `ms repair --ms1 <one induced error>` → rc 4), so the claim is
true and the provenance is not.

**M-8 — The 56 rows were extrapolated from 12, and four of them are already
fully green.** *"**All 56 pass material today at exit 0** — 52 of them in total
silence, 4 with a warning only — measured per §1.4's table."* §1.4's table has
**12** rows, all canonical spelling. Measured, the UPPERCASE spelling on all four
`--phrase` channels:

```
ms encode --phrase "<UPPERCASE 12-word phrase>"          -> rc 1, stderr: "error: unknown BIP-39 word at position 0 …"
ms split  --phrase "<UPPERCASE>" -k 2 -n 3               -> rc 1, same
ms derive --phrase "<UPPERCASE>"                         -> rc 1, same
ms verify <ms1> --phrase "<UPPERCASE>"                   -> rc 1, same
grep -c 'ABANDON' stderr                                 -> 0, in every case
```

Non-zero exit **and** no material in stderr — so those four rows are green on
**both** halves today and can never fail. (UPPERCASE `--hex` and UPPERCASE `ms1`
do still exit 0, so most of the cross-product is a real gate.) The claim "all 56
… measured" is the fifth failure mode from the brief: an extrapolation written up
as a measurement.

---

# WHAT I RE-MEASURED AND FOUND TRUE

Recorded so a later round does not re-derive it, and because the ratio matters:
this plan's `ms` measurements are unusually reliable and the findings above are
about *reasoning and ordering*, not sloppiness.

- **All 22 `crates/ms-cli/` citations resolve to exactly what the plan says.**
  `parse.rs:21/36/54/95` → `read_input` / `read_phrase_input` /
  `read_stdin_passphrase` / `is_stdin_arg`; `main.rs:125/169/170/176` →
  the `gen-man --out` examples / `set_non_dumpable()` / `Cli::try_parse()` / the
  exit-64 carve-out comment; `advisory.rs:38/53` → `secret_in_argv_warning` /
  `enum OutputClass`; `format.rs:12/18/41/265` → `is_display_separator` /
  `render_grouped` / `parse_separator` / the vectors path;
  `encode.rs:27/77` → the `input` ArgGroup / `resolve_secret_payload`;
  `split.rs:28` → the `split_input` ArgGroup; `combine.rs:54` → `read_shares`;
  `derive.rs:322/327/332/336` → the four `secret_in_argv_warning` sites;
  `process_hardening.rs:26` → `set_non_dumpable`; `cli_output_class.rs:56` →
  `byte_parity_advisory_lines`; `cli_derive.rs:347` → `inline_secret_argv_advisory`.
- **The two central negatives hold.** `git grep -n 'env::args' -- crates/` →
  exit 1, zero hits. `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode' -- crates/`
  → exit 1, zero hits. (Scope note: `ms` *does* create files, via
  `std::fs::create_dir_all` + `clap_mangen::generate_to` at
  `cmd/gen_man.rs:30-35` — so the greps have a blind spot, but the conclusion
  they support, "`ms` has never created a file with a mode", is correct.)
- **§1.4's argv table reproduces**, including `ms encode --phrase "<a real
  seed>"` at **exit 0 in silence**, and the 4 warning sites on `derive` alone.
- **§1.3 reproduces** — all twelve `-`/omitted forms exit 0 — and the
  documentation gap is as measured: `grep -ci stdin` over each `--help` gives
  `combine` **0**, everything else ≥ 1.
- **§1.6's `--out` collision is entirely real and every citation checks out.**
  `ms gen-man --out <DIR>`; `.github/workflows/man-release.yml` **line 46** is
  `./target/release/ms gen-man --out man`; `mnemonic-secret` has **no**
  `scripts/` directory; `ms gen-man --help` does say *"`scripts/install.sh`
  invokes this"*; the real one is
  `mnemonic-toolkit/scripts/install.sh:305`, inside the man-page post-install
  hook.
- **§1.7 reproduces exactly**, including the grouped string
  `ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f`, the exit-4
  refusal, and the **102-byte** payload at mode `0600` from the
  `--group-size 0` + `--no-passphrase` run.
- **The conformance pin survives the separator work, for the reason given.**
  `sha256sum -c display-grouping-vectors.tsv.sha256` → exit 0, `OK`. The test at
  `format.rs:265` calls `render_grouped` through a **test-local** keyword mapper
  (`"comma" => ','` at `format.rs:255`), never `parse_separator`. And
  `parse_separator` is the `value_parser` on `encode.rs:51` and `split.rs:56`
  and nowhere else, so "one parser serves both, so it cannot bind to one of
  them" is exact.
- **§1.8's counts are exact.** 76 integration-test files, **276** `#[test]`,
  **31** files matching `--phrase|--hex` holding **147** tests, **146**
  `#[test]` in `src/`. `cargo nextest run --locked` → **414 run, 414 passed, 5
  skipped**, exit 0.
- **The 147-test migration IS achievable** — the brief flagged this as a
  possible Critical and it is not one. The one shape I expected to be unreachable
  is reachable: `encode_arg_group_violations.rs::encode_rejects_both_phrase_and_hex`
  needs two material flags at once, and `ms encode --phrase - --hex -` exits
  **64** with clap's group error, before either stream is read. Only three test
  files pair a material flag with a `code(64)` assertion, and all three migrate.
  (Row 2's phrasing *"extended to three members rather than rewritten"* is
  imprecise — the argv **is** rewritten — but nothing is lost.)
- **§2.4's `records` decline is measured correctly.** `ms split` emits shares
  grouped in fives by default, and feeding two of them to `ms combine -`
  recovers the secret at exit 0 — `read_shares` strips the display separators,
  which `split_record_stream` does not.
- **F-284 is right.** `me sysw pack` refuses a codex32 share at exit 4 **grouped
  and ungrouped alike**, so grouping is not what blocks it.
- **F-286 reproduces, with a proper control.** A probe citing
  `.github/workflows/release.yml:1` — a file that **exists** in this repo — is
  reported `DANGLING  github/workflows/release.yml:1  (no such file under any
  root)`, exit 1. The leading dot is stripped by the `path="${path#./}"` line.
- **Row 12's committed expectations exist**, so its byte-identical gate is
  satisfiable once a release build exists: `design/journeys/inputs-rcw/` is 36
  tracked files, `inputs-hashvault/` is 16. And the unrunnability claim holds —
  `mnemonic-secret/target/release/ms` does not exist.
- **Row 13's gui-schema baseline reproduces to the flag.** `ms gui-schema` →
  10 subcommands, **36** flags: derive 9, encode 7, decode 2, inspect 1, verify
  3, vectors 1, gen-man 1, repair 2, split 8, combine 2. The arithmetic to 55
  (+8 `--in`, +8 `--allow-argv-secret`, +3 `--out`) is correct.
- **Row 14's regression controls are live gates.** `ms encode > <a 0644 file>`
  exits 0 today (so a smuggled stdout gate would go RED), and `--no-engraving-card`
  does **not** ungroup stdout (so row 10 is a real change, not a no-op).
- **`write_private` is sound** — see I-5.
- **Row 7's crate claims are accurate**: the `\bme\b` / `cd /home/me` six-line
  sample is in `remedy.rs:65-70`; `history -d` appears in the prose and in **no**
  recipe; fish is `history clear-session` with no `command` interpolation.

---

# CLOSING NOTE ON LENSES

This was a **counterexample-construction** round, per the brief. The questions it
did **not** ask, and which a clean re-round should not be mistaken for having
closed:

1. **A journey walk with the operator.** C-2 was found by running one printed
   line; §6h says the rule was earned by shipping the opposite once, and it has
   now been shipped a second time. Everything an operator is *told* to type in
   this cycle should be executed, not read.
2. **A comprehension lens on `--in`'s per-verb meaning.** `--in` will mean *a
   phrase* on `encode`/`split`, *an ms1* on `decode`/`verify`/`inspect`/`repair`/
   `derive`, and *a file of shares* on `combine`. That may well be right, but no
   round has asked whether an operator can predict it.
3. **The cross-repo ordering of row 8.** It edits `mnemonic-engrave` to advise a
   flag that only exists in an unreleased `mnemonic-secret`. How `me`'s gate
   obtains an `ms` that has `--in` is unaddressed, and `me`'s suite invokes no
   `ms` binary today.
