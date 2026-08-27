# R0 ROUND 10 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `ce5a206`
(worktree `review/p0-r10`).
**Round 9's report:** `design/agent-reports/R0-P0-plan-round9.md` (1C/4I/10M/1N).
**The fold under review:** `ce5a206` — 22 insertions, 21 deletions in the plan,
**5 hunks**, all inside §3 and §4 row 6; plus a `FOLLOWUPS.md` edit.
**Object:** (1) does the *reuse-the-classifier* design hold — does `classify()`
cover every argv-forbidden class, is it callable pre-parse, does it produce false
refusals, do the positive controls bite; (2) did the fold close round 9's
1C/4I/10M/1N, verified against the DIFF; (3) can an implementer execute all twelve
rows.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **2** |
| **Important** | **4** |
| Minor | 6 |
| Nit | 1 |

**The design is right and the fold applied it to one section out of two.**
Reusing `me`'s own classifier is correct, and every claim the fold makes *about
the classifier* measures TRUE — five classes covered, callable before
`Cli::parse()`, no false refusal on `mt1-…-transfer.txt`, `me bundle` and
`me help` survive, and the three new positive controls do exclude a
refuse-everything guard. That question is answered and it should not be re-opened.

**What did not happen is propagation.** The fold rewrote §4 row 6 and left §3 —
the section that *defines* `records.rs` — describing the previous design at both
of the other two sites round 9 named by line number. The plan now carries two
incompatible guards twelve lines apart, and the one in the design section is the
four-shape hand list that omits `pass:`, i.e. round 9's Critical verbatim. Worse,
§3 also says *where* the guard lives — inside the new crate — and the adopted
design has it calling into `me`, which is a **cyclic package dependency** cargo
refuses, and which turns row 9b's own gate RED.

---

## QUESTION 1 — DOES THE REUSE-THE-CLASSIFIER DESIGN HOLD?

### Method

`classify()` measured **directly**, not inferred from binary behaviour: a
throwaway integration test in the worktree
(`crates/me-cli/tests/r10_classify_probe.rs`, removed before this commit) calling
`mnemonic_engrave::sysw::classify` on a token list supplied through an env var,
printing `{Class, is_secret, is_bearer, is_argv_forbidden}` per token. Source, so
it can be re-run:

```rust
use mnemonic_engrave::sysw::classify;
#[test]
fn r10_probe() {
    for t in std::env::var("R10_TOKENS").unwrap_or_default().split('\u{1}').filter(|s| !s.is_empty()) {
        let c = classify(t);
        println!("R10\t{:?}\tforbidden={}\t{}", c, c.is_argv_forbidden(), t);
    }
}
```

Binary measurements: `me` by absolute path
(`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`), stdout and stderr
to separate files, `rc` from `$?` immediately, never through a pipe, stdin
`/dev/null`, **`umask 077`** (a 0644 `out.txt` otherwise trips `me`'s own
world-readable-stdout refusal and masks the result — round 9's `--in` rows show
that artefact). `leak` is `grep -qF` for a fixture-body substring in **stderr
only**.

### 1a. Does `classify()` cover every argv-forbidden class? — **YES**

All five classes in `is_argv_forbidden()` are produced by `classify()`, measured:

| token | `Class` | `forbidden` |
| --- | --- | --- |
| `ms10entrs…34v7f` | `Codex32Secret` | **true** |
| `mt1p9h8jqq9…29sax` | `Mt` | **true** |
| `tx:0200…` (447 chars, signed) | `Tx` | **true** |
| `pass:68756e74657232` | `Passphrase` | **true** |
| `abandon ×11 about` (one token) | `Mnemonic` | **true** |

**Round 9's C-1 is genuinely answered at the level of the class set.** The
classifier has no missing class. The gap is elsewhere — see 1c and C-2.

### 1b. Is `classify()` callable before `Cli::parse()`? — **YES**

It is `pub fn classify(record: &str) -> record::Class`
(`crates/me-cli/src/sysw/mod.rs:178`) in the **lib** half, taking a `&str` and
nothing else. The probe above calls it from a test binary that never constructs
`Cli` and never links clap's parse. Its call graph — `record::decode_body`,
`tx::parse`, `bip39::Mnemonic::parse_normalized`, `sysw::mt::valid_mt`,
`seal::record::validate_record` — contains **zero** `std::env` references
(`grep -rn 'std::env' crates/me-cli/src/sysw/ crates/me-cli/src/seal/` returns one
hit, `seal/mod.rs:705`, a `ME_EMIT_VECTORS` vector dumper, not in the path). No
ordering hazard. **This half of the design is sound.**

### 1c. Does the classifier have its own gap? — **YES, and it is I-1**

`classify()` is **not case- or whitespace-normalising**. Every near-miss spelling
of a forbidden class returns `Class::Unknown`, `forbidden=false`:

| token | `Class` | `forbidden` |
| --- | --- | --- |
| `tx:0200…` with a **leading space** | `Unknown` | false |
| `tx:0200…` with a **trailing space** | `Unknown` | false |
| `TX:0200…` (uppercase prefix) | `Unknown` | false |
| ` pass:68756e74657232` | `Unknown` | false |
| `PASS:68756E74657232` | `Unknown` | false |
| `MS10ENTRS…34V7F` (uppercase) | `Unknown` | false |
| `ABANDON ×11 ABOUT` | `Unknown` | false |
| `mt1…` uppercase / leading space | `Mt` | **true** — the one class that folds |

**The donor's own gate compensates for exactly one of these, and its source says
why**, three lines above the `classify` call row 6 adopts:

> *"Matched on the TRIMMED, case-folded prefix rather than through `classify`,
> deliberately: a near-miss like ` TX:<hex>` is then refused here for the BEARER
> reason rather than three screens later for a formatting one. Neither message may
> name the body."* … *"The `tx:` PREFIX check stays, and stays FIRST … `classify`
> would call that shape `Unknown`."*
> — `crates/me-cli/src/main.rs:1946-1958`

`read_records` therefore refuses on `by_prefix || class.is_argv_forbidden()`
(`main.rs:1979`). **Row 6 specifies only the second half**, and says
*"It does not invent a recogniser."* Measured consequence, today:

```
sysw pack TX:<447-char signed tx>   rc=3   -          me: record 0 … is a `tx:` record on ARGV
bundle    TX:<447-char signed tx>   rc=2   LEAK-ERR   error: unexpected argument 'TX:0200…'
bare      TX:<447-char signed tx>   rc=2   LEAK-ERR   error: unrecognized subcommand 'TX:0200…'
sysw      TX:<447-char signed tx>   rc=2   LEAK-ERR   error: unrecognized subcommand 'TX:0200…'
help      TX:<447-char signed tx>   rc=2   LEAK-ERR   error: unrecognized subcommand 'TX:0200…'
bare      <MS1 uppercase>           rc=2   LEAK-ERR   error: unrecognized subcommand 'MS10ENTRS…'
bundle    <MS1 uppercase>           rc=2   LEAK-ERR   error: unexpected argument 'MS10ENTRS…'
```

A whole signed transaction — a bearer instrument — and a codex32 secret, printed
verbatim, **after** the guard row 6 specifies has run and passed them.

### 1d. Does it produce false refusals? — **NO. All three of the plan's claims measure TRUE**

| token the plan says must NOT be refused | `Class` | `forbidden` |
| --- | --- | --- |
| `mt1-2026-08-23-transfer.txt` | `Unknown` | false ✔ |
| `mt1-2026-08-23-cold-storage-transfer.txt` | `Unknown` | false ✔ |
| `ms1-recovery-notes.txt` | `Unknown` | false ✔ |
| `bundle` | `Unknown` | false ✔ |
| `help` | `Unknown` | false ✔ |
| `text:68656c6c6f` | `FreeText` | false ✔ |

Round 9's I-1 (the sibling repo's shipped false refusal) and M-2 (`bundle` and
`help` are BIP-39 words — re-verified: both present in `bip39-2.2.2`'s 2048-word
English list) are **closed, and closed by the right mechanism**: the classifier's
granularity is whole-token decode, so a single word is not a mnemonic and a
filename beginning with an HRP is not a constellation string. The plan needed no
charset rule after all, and saying so was correct.

### 1e. Do the positive controls exclude a refuse-everything guard? — **YES**

`fn guard(_) -> ! { exit(3) }` fails row 6's new controls: `me help` exits **0**
today and would exit 3; `me sysw pack --out c.bin text:68656c6c6f` exits **0**
today and would exit 3; `me sysw pack --out d.bin --in mt1-2026-08-23-transfer.txt`
packs. Round 9's M-4 is **closed**. (One wrinkle → M-4 below: `me bundle` with no
arguments exits **2**, not 0, so *"still work"* needs an observable.)

### 1f. The ordering observable — **constructible, and it fails today**

`me --nosuchflag <ms1>` → **rc 2**, clap naming the flag, no leak. Row 6 requires
rc 3 with the guard's wording. A post-parse guard can never produce that, because
clap errors first. It needs no feature flag, no env hook and no edit to non-test
code. **Round 9's I-4 is closed.**

---

## CRITICAL

### C-1. The adopted design cannot live where §3 puts it. The guard is assigned to the crate's `records.rs`; row 6 has it calling `me`'s `classify()` and `Class::is_argv_forbidden()`; `me` depends on the crate. That is a cyclic package dependency, and row 9b's own gate goes RED.

**Sites:** §3 file tree **line 208** (*"`src/records.rs` — record stream
splitting, **the argv gate**, kind vocabulary"*); §3 file table **line 474**
(*"`records.rs` | … and **the pre-parser argv machinery**"*); §3 **line 518**
(*"**Nothing in the crate ever names a `Class` variant.**"*); §4 row **6**,
line **568**; §4 row **9b**, line **572** (*"the crate builds standalone; `me`
depends on it by path; **no `EXIT_*` and no `Class` in it**"*).

**The contradiction, in the plan's own words.** §3 line 515-518 rules the split:

> *"**So the split is by REPRESENTATION.** The crate's recognisers work on
> strings — a `tx:` prefix, an HRP character, a BIP-39 word — and return the
> crate's **own** kind type. `me` maps that kind onto its `Class` and keeps the
> three predicates. **Nothing in the crate ever names a `Class` variant.**"*

Row 6 now says the opposite:

> *"…and asks `me`'s **OWN** classifier — `classify(token)` then
> `Class::is_argv_forbidden()` … **It does not invent a recogniser.**"*

Both cannot hold for one function. The fold changed **what** the guard does and
touched neither the sentence saying **where** it lives nor the ruling saying what
it may name.

**The concrete failure is a build error, not a style objection.** Row 9b makes
`me` depend on `mnemonic-io-lib` by path. A `mnemonic-io-lib::records` that calls
`mnemonic_engrave::sysw::classify` needs `mnemonic-io-lib` to depend on `me-cli`.
Reproduced in a two-crate scratch project, exactly as the plan reproduced `E0116`
for N-C2:

```
error: cyclic package dependency: package `a-bin v0.1.0` depends on itself. Cycle:
package `a-bin v0.1.0`
    ... which satisfies path dependency `a-bin` of package `b-lib v0.1.0`
    ... which satisfies path dependency `b-lib` of package `a-bin v0.1.0`
```

So an implementer builds row 6 in `me`'s lib half (fine — `classify` is right
there), reaches row 9b, moves `records.rs` into the crate because §3's table says
that is where it lands, and the workspace stops resolving. The alternative reading
— leave the guard in `me` — contradicts §3 twice and quietly removes P0's headline
deliverable from the crate P0 exists to create. **This is the same shape as N-C2
and as the move's `EXIT_*` problem: a language/build rule that no ordering avoids,
discovered by executing rather than by reading.**

**What closes it.** Pick one and say it in §3's tree, §3's table and row 6
together:

- **The guard stays in `me`.** Then §3 line 208 and line 474 must stop assigning
  the argv machinery to `records.rs`, and §3 must say what P0's crate contributes
  to the argv work instead (plausibly: nothing, and that is an honest answer).
- **Or the crate holds the traversal and `me` supplies the predicate** — e.g.
  `mnemonic_io_lib::records::scan_argv(args: impl Iterator<Item=String>, forbidden: &dyn Fn(&str) -> bool) -> Result<(), Refusal>`,
  with the `=`-split, the `--` handling and the override scoping in the crate and
  the classifier call in `me`. That satisfies row 9b's *"no `Class` in it"*
  literally, keeps §3's representation ruling intact, and is testable in the crate
  with a fixture predicate.

Either way row 9b's gate must be re-stated so it can pass.

### C-2. Round 9's C-1 was closed at ONE of the three sites it named. §3 still specifies the four-shape hand-written recogniser that omits `pass:` — the defect itself, present tense, in the section that defines the crate. The plan's own propagation gate returns **1**.

**Sites:** §3 **lines 270-273**, and §3's file table **line 474**. Round 9's C-1
listed its sites as *"§3 line **272** … §4 row **6**, line **567** … §3's file
table row for `records.rs`"*. `git diff -U0 5cf8ac9..ce5a206` shows the fold's
hunks at new-file lines **275-289, 310-313, 320, 532, 568**. Lines 272-273 and 474
are **not in any hunk** — they appear in the diff only as context.

What §3 says today, present tense and normative:

> line 270-273: *"**`records.rs` — the PRE-PARSER argv guard, ONE layer in this
> donor (C2, and round-7 C-1).** It runs on raw `std::env::args()` **before
> `Cli::parse()`** and recognises material by **value shape**: `tx:` by prefix,
> `mt1`/`ms1` by HRP, a BIP-39 mnemonic by wordlist."*
>
> line 474: *"| `records.rs` | `split_record_stream`, `no_records_guard`, **the
> string-level recognisers (prefix / HRP / wordlist)**, and the pre-parser argv
> machinery |"*

That is the four-shape list. `pass:` is not in it. It is the exact enumeration
round 9 called Critical, and it now sits **twelve lines above** the paragraph that
retracts it and **94 lines above** the file table that repeats it.

**Machine-checked, with the plan's own gate**, run with the phrasings this fold
retracted:

```
$ ./scripts/fold-propagation-check.sh design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md \
      'prefix / HRP / wordlist' 'by HRP' 'by wordlist' 'string-level recognisers'
  LEFT   prefix / HRP / wordlist       474
  LEFT   by HRP                        272
  LEFT   by wordlist                   273
  LEFT   string-level recognisers      474
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.
rc=1
```

The commit message reports *"fold-propagation 0"*. Both are true: the gate was
run, with patterns that did not include what this fold retracted. That is the
blind spot the script documents in its own header (*"It cannot invent the patterns
for you"*), and this is the first time in the cycle it has produced a green that
was false for the fold that ran it.

**The concrete failure.** §3 is *"WHAT THE CRATE CONTAINS"* — it is where an
implementer goes to learn what `records.rs` is. An implementer who reads §3 builds
a prefix/HRP/wordlist recogniser, ships a pre-parser guard blind to
`Class::Passphrase`, and `me pass:<hex of the operator's wallet passphrase>` still
prints it. Row 6's cross-product then carries *"every argv-forbidden class,
`pass:` included"* — a gate written against a design §3 does not describe.

**What closes it.** Replace both sites with the adopted design and re-run the
propagation gate **with these four patterns**, quoting the exit code in the fold
commit. If the sentence is kept as a historical note, mark it as one — the script
prints hits precisely so that call is made per hit.

---

## IMPORTANT

### I-1. `classify()` is not case- or whitespace-normalising, so a near-miss spelling of a bearer or secret class is `Unknown` and the specified guard passes it. The donor's post-parse gate has a trim-and-case-fold prefix check for exactly this and says so in a comment. Row 6 excludes it, and its carrier axis cannot express it.

**Site:** §4 row **6**, line **568** (*"It does not invent a recogniser."*).

Evidence and the seven measured leaks are in **§1c** above; the donor's rule is
`by_prefix || class.is_argv_forbidden()` at `crates/me-cli/src/main.rs:1979`, with
`by_prefix` computed on `r.trim_start().to_ascii_lowercase()` at `:1958`.

**Why the gate cannot catch it.** Row 6's third axis is *"every argv-forbidden
class"* — a set defined by what `classify()` **returns**. A token `classify()`
calls `Unknown` is by construction not a member, so no generated row can carry
` TX:<hex>` or `MS1…`. The axis that was supposed to stop the shape list being
hand-written also makes the shape list exactly as wide as the classifier, and the
classifier is narrower than `me`'s own shipped gate.

**This is round 9's C-1 one level down again** — a pre-parser guard weaker than
the post-parse guard it fronts, with the guarantee in §6 condition 8 closing
anyway because it asks only about ordering.

**What closes it.** Say in row 6 that the token is **normalised before
classification the way the donor normalises it** — `trim()` and, for the reserved
prefixes, ASCII-lowercase — and keep the donor's `tx:`-prefix arm rather than
dropping it, naming `main.rs:1946-1958` as the reason. Then add **one carrier** to
the cross-product that is a near-miss (` TX:<hex>` or `MS1…` uppercase), which
fails today on six surfaces. Note that this is not "inventing a recogniser": it is
copying two lines the donor already has and documents.

### I-2. Row 6 claims reusing the classifier *"settles M-2 and M-3"*. Measured, it does not settle M-3, and the gate's headline observable — *"no secret material in stderr for ANY argv containing some"* — is an assertion the specified guard cannot satisfy.

**Site:** §4 row **6**, line **568**, both the step column's closing clause and
the gate column's opening clause.

Measured today, `umask 077`, secret embedded in a **path**:

```
bare   --in /tmp/nope/<ms1>.txt        rc=2  LEAK-ERR  me: cannot read /tmp/nope/ms10entrs….txt: No such file…
sysw pack --in /tmp/nope/<ms1>.txt     rc=2  LEAK-ERR  me: /tmp/nope/ms10entrs….txt: No such file…
sysw show /tmp/nope/<ms1>.txt          rc=2  LEAK-ERR  me: /tmp/nope/ms10entrs….txt: No such file…
bundle --in /tmp/nope/<ms1>.txt        rc=2  LEAK-ERR  me: cannot read /tmp/nope/ms10entrs….txt: No such file…
sysw pack --out /tmp/nope/<ms1>.bin text:6869   rc=2  LEAK-ERR
```

And `classify("/tmp/ms10entrs…34v7f.txt")` → **`Unknown`, `forbidden=false`**.
These are argvs containing secret material; the guard does not refuse them and —
per I-1 of round 9, which the fold correctly honoured — **must not**, or it
re-opens the sibling repo's false-refusal bug. So the two sentences cannot both
stand.

Round 9's M-3 asked the plan to *"take a side rather than leave both sentences
standing"*. The fold left both standing and added a third asserting the question
is settled.

**What closes it.** Narrow the gate's observable to what the design delivers —
*"no argv **token that classifies as argv-forbidden** reaches stderr"*, or
*"…for any argv whose tokens include one"* — and state plainly, once, that a
secret embedded in a filename is **out of the guard's reach by design**, with the
false-refusal trade as the reason. Then drop *"and M-3"* from the settles claim,
or say what it settles it to.

### I-3. The flag-name layer went from *"P0 does not build it"* to *"P0 builds both"* with no step, no closing condition, no follow-up entry — and its only stated home is contradicted by §7.

**Sites:** §3 lines **282-289**; §7 line **829**. Spec §6d lines 824-841.

Verified, this round:

| where the layer could be owned | state |
| --- | --- |
| §4's twelve step rows | **no row builds it** (`grep -n 'flag-name\|flag_is_secret'` over the plan → **3 hits: 283, 285, 340**, none in §4) |
| §6's eleven closing conditions | **no condition mentions it** |
| §7 out of scope | **not listed** |
| `design/FOLLOWUPS.md` | `grep -n 'flag-name\|flag_is_secret'` → **0 hits** |

And the one home the fold does name contradicts §7 forty pages later:

> §3 line **287**: *"…it is why the layer's own conformance test is **asserted
> against `mnemonic-toolkit`**, which declares them."*
> §7 line **829**: *"**`mnemonic-toolkit`'s own adoption.** It is the sixth
> consumer, **not P0's work**."*

The factual half of the fold's claim is TRUE and is not at issue —
`mnemonic-toolkit` does declare `--passphrase` / `--passphrase-stdin` /
`--bip38-passphrase` and does carry `flag_is_secret`
(`crates/mnemonic-toolkit/src/secrets.rs:59-88`). What is missing is ownership.
§3 line 270 still calls the guard *"ONE layer in this donor"* while line 282 says
*"P0 builds both"*.

**Round 9's I-2 is not closed.** Its stated core was *"the layer is owned by
nothing, and the plan reads as though it were done"*, and the fold changed only
the sentence that describes it — from a false justification for skipping it to a
true statement that it will be built, with the same nothing behind both.

**What closes it** — unchanged from round 9, minus the option the fold used up:
either a §4 row that builds the membership layer in the crate with a gate over a
**fixture** flag list (which can fail without `me` declaring such a flag, and
which does not drag the toolkit into P0's scope), **or** a `FOLLOWUPS.md` entry
with an owning phase plus a sentence in §7 saying P0 ships one of §6d's two
layers. Whichever is chosen, §3 line 270's *"ONE layer"* and §7 line 829 must
agree with it.

### I-4. `--allow-argv-secret` regained a step clause and still has no observable. Row 6 gained three positive controls and not the one round 9 named, and the clause as written does not say the override is scoped to where the flag is declared.

**Sites:** §4 row **6**, line **568** (*"`--allow-argv-secret` still overrides,
and its own parse runs here"*); §6 condition **8**, line **789**.

The override is a shipped escape hatch on the funds path
(`crates/me-cli/src/main.rs:252`, honoured at `:1959`, named in `me`'s own refusal
text at `:2022`). Measured today:

```
sysw pack --allow-argv-secret --out o.bin <ms1>   rc=0   SEALED, 143 bytes written
```

Row 6's gate asserts (a) absence of secret material across 8×3×N rows, (b) the
ordering rc, (c) three positive controls — *filename with an HRP packs*, *`me
bundle`/`me help` still work*, *a `text:` record packs*. **None of them is an
override row.** A guard that refuses `--allow-argv-secret <ms1>` passes row 6
completely and is caught only at row 10, by the 15 pre-existing test references
(`tests/sysw_cli.rs` ×13, `tests/cli.rs` ×2) — a regression catch arriving four
rows late, which is precisely what M-4 was filed to prevent one round ago.

**And the scoping is still unstated.** `--allow-argv-secret` is declared on
`me sysw pack` **alone**. *"its own parse runs here"* read literally is
`if argv.iter().any(|a| a == "--allow-argv-secret")` — a bypass token that works
on every surface. (Measured: on `me`, clap happens to name the unknown *flag*
rather than the value on `bundle`, `sysw wipe` and bare `me`, so the bypass does
not leak **on this donor today** — which is the round-6 exception again, and not
something to build a rule on.)

**What closes it** — round 9's two one-line assertions, verbatim, added to row 6's
gate: `me sysw pack --allow-argv-secret <ms1>` still exits **0**, and
`me bundle --allow-argv-secret <ms1>` is still refused by the guard. Plus five
words in the step column saying the override is honoured **only where the flag is
declared**.

---

## MINOR

**M-1. Five of round 9's Minors are untouched, and none is claimed.** Verified
present verbatim (`grep -c` = 1 each):

| round 9 | phrase | state |
| --- | --- | --- |
| M-6 | condition 9's sentence splice + duplicated recipe (`so P0 fixes the recipe`) | carried, **seventh round** |
| M-7 | `really 11 and not more` (the move relocates five + the stub) | carried |
| M-8 | nested backticks around `crates/me-cli/src/io.rs`, a module §3's tree never lists | carried |
| M-9 | `the digit-pinning work` resolving to four rows that pin a digit | carried |
| M-10 | `enumerate every type and constant` attached to the move, where `E0116` cannot occur | carried |

This is an improvement on round 9 (which found 0 of 7 touched): **M-1, M-4, M-5
and N-1 are closed** and the commit message claims exactly those. But the fold's
43 changed lines are again entirely inside §3 and row 6, and five Minors have now
survived every fold since round 7.

**M-2. The retracted "all positional" claim survives at a third site, 40 lines
below the fold's own retraction.** Line **299** (the fold's): *"**F-266's leaks
are not four and not all positional** — `--in` carries them too"*. Line **339**
(untouched): *"**And the leaking cases are UNEXPECTED POSITIONALS, not declared
flags** — so they are the **value-shape** layer's business"*. That sentence is
also the last remaining prose argument for demoting the flag-name layer, which
§3 line 282 has now reversed → interacts with **I-3**.

**M-3. The `FOLLOWUPS.md` F-266 fix is itself a splice.** The fold deleted the
subject (*"`me sysw pack` is the only surface taking positional records"*) and
left the predicate: *"**Every other surface** takes no positional, so clap rejects
the argument and names it"* (`design/FOLLOWUPS.md:11797-11799`). Other than what?
And the surviving clause restates the retracted claim — `me sysw show` takes
`<FILE>` (`crates/me-cli/src/main.rs:275`). Same shape as M-6's condition-9 splice.

**M-4. Row 6's positive control *"`me bundle` … still work[s]"* has no observable,
and the obvious one is false today.** Measured: `me bundle` with no arguments
exits **2** (*"me: no input strings (expected newline-separated md1/mk1)"*).
`me help` exits 0. An implementer writing `.success()` for both gets a control
that fails for an unrelated reason. State it as *"not refused by the guard —
rc unchanged from today's 2, and stderr does not carry the guard's wording"*.

**M-5. The other half of round 9's M-2 is still unstated.** The fold answers the
granularity question in the direction that produces false refusals (good) and
says nothing about the direction it gives up: an **unquoted** twelve-word mnemonic
is twelve tokens, each `Unknown` (measured), so `me abandon abandon … about` is
not refused by the guard. Round 9 asked the plan to *"say what happens to the
shape the other one catches"*. One sentence, and it belongs next to the
*"a single word is not a mnemonic"* claim that trades it away.

**M-6. A donor observation, for `FOLLOWUPS.md` rather than for this plan.** The
same normalisation gap in I-1 is present in `me`'s **shipped post-parse** gate for
the non-`tx:` classes: `me sysw pack " pass:68756e74657232"` classifies `Unknown`
and is **not** refused as SECRET (the `by_prefix` arm covers `tx:` only). P0 is
the phase that rewrites this code, so it is the cheapest moment to file it.

---

## NIT

**N-1.** Row 6's control names two BIP-39-word surfaces (*"`me bundle` and
`me help`"*). The cross-product's eight subcommand shapes contain **three**:
`bundle`, `help`, and `sysw help`. `pack`, `show`, `wipe`, `sysw` and `seal` are
not wordlist members (re-verified against `bip39-2.2.2`).

---

## QUESTION 2 — DISPOSITION OF ROUND 9's 1C/4I/10M/1N, AGAINST THE DIFF

`git diff -U0 5cf8ac9..ce5a206` on the plan: **5 hunks**, new-file lines
**275-289, 310-313, 320, 532, 568**. **"claimed?"** = named in `ce5a206`'s commit
message.

| # | claimed? | disposition | the diff line that closes it, or why not |
| --- | --- | --- | --- |
| **C-1** — recogniser omits `pass:` | yes | **PARTIAL — closed at 1 of the 3 named sites** | row 6 (line 568) now derives the class set from `is_argv_forbidden()` ✔ and the carrier axis is *"every argv-forbidden class, `pass:` included"* ✔. §3 lines **272-273** and the file table **474** are untouched and still specify the four-shape list → **C-2 above** |
| **I-1** — bare prefix rule / charset | yes | **CLOSED, and by the better mechanism** | measured: `mt1-…-transfer.txt`, `ms1-recovery-notes.txt` → `Unknown`. No charset rule needed. Correct call |
| **I-2** — flag-name layer owned by nothing | yes | **NOT CLOSED** | lines 282-289 replace a false justification with a true intention; still no row, no condition, no follow-up, and §7:829 contradicts §3:287 → **I-3 above** |
| **I-3** — override has no step/observable | yes | **PARTIAL** | row 6's step column regained the clause ✔; the gate column still has no override row, and the scoping is unstated → **I-4 above** |
| **I-4** — ordering observable not constructible | yes | **CLOSED** | `me --nosuchflag <ms1>` measured rc **2** today, needs rc 3 + guard wording; no feature, no env hook, no non-test edit |
| **M-1** — retracted counts survive | yes | **PARTIAL** | lines 310-313 and 320 fixed ✔; line **339** untouched → **M-2 above** |
| **M-2** — wordlist granularity | yes | **PARTIAL** | the false-refusal half is settled by the classifier ✔; the unquoted-phrase half unstated → **M-5 above** |
| **M-3** — "ANY argv containing one" vs anchored recogniser | yes (as *settled*) | **NOT CLOSED — and now asserted closed** | measured: path tokens still `Unknown`, four leaks → **I-2 above** |
| **M-4** — no positive control | yes | **CLOSED** | three controls added; each excludes `fn guard(_) -> ! { exit(3) }`. Minor wrinkle → **M-4 above** |
| **M-5** — `FOLLOWUPS.md` F-266 counts | yes | **CLOSED**, with a splice → **M-3 above** | heading and table caption rewritten |
| **M-6 … M-10** — five carried Minors | no | **NOT CLOSED** | all five verbatim, `grep -c` = 1 each → **M-1 above** |
| **N-1** — `the moving set reference` | yes | **CLOSED** | line 532 |

**Score: 0 of 1 Critical fully closed (1 of its 3 sites), 2 of 4 Important closed,
3 of 10 Minor, 1 of 1 Nit.**

**No named-and-not-done, for a third round.** Every claim in `ce5a206`'s message
is in its diff. What the message does claim and should not is the gate line —
*"fold-propagation 0"* — which is true of the command that was run and false of
the fold it was run on (**C-2**).

---

## QUESTION 3 — CAN AN IMPLEMENTER EXECUTE ALL TWELVE ROWS?

§4 carries **12** step rows: `1, 2, 3, 4, 5, 6, 7, 8, 9, 9b, 10, 11`. The diff
changed **1** table line (row 6). Rows 1-5 and 7-11 are unchanged from text rounds
8 and 9 verified row-by-row and are not re-derived.

| # | executable? | can its gate fail? | this round's note |
| --- | --- | --- | --- |
| 1 signature change | yes | yes | unchanged |
| 2 the move | yes | yes | still names `io.rs` (**M-1**) |
| 3 mask split | yes | yes | unchanged |
| 4 `observation.rs` + pty | yes | yes | unchanged |
| 5 `remedy.rs` | yes | yes | unchanged |
| **6 the argv guard** | **partly** | **yes, and better than round 9's** | the class set, the false-refusal behaviour, the ordering test and the positive controls all measure sound. Blind to near-miss spellings (**I-1**), asserts an observable it cannot meet (**I-2**), no override row (**I-4**), and §3 describes a different guard (**C-2**) |
| 7 F-265 five-site digit pin | yes | yes | unchanged — still the model row |
| 8 `--expect` | yes | yes | unchanged |
| 9 `exit.rs` + `channel.rs` | yes | yes | unchanged |
| **9b create the crate** | **NO** | **no — its gate cannot pass** | *"no `Class` in it"* is unsatisfiable while §3 assigns the argv machinery to `records.rs` and row 6 has it calling `Class::is_argv_forbidden()`; the move itself is a cyclic package dependency (**C-1**) |
| 10 consume | yes | yes, as regression | unchanged. Still the only thing between **I-4** and a broken escape hatch |
| 11 publish | n/a | n/a | operator-gated |

**Ten of twelve rows are executable with a gate that can fail.** Row 6 is
materially better than round 9's and still short; row 9b is the one that cannot be
executed at all as written.

---

## WHAT I VERIFIED HERE

Absolute paths throughout. `$?` read immediately, never through a pipe. stdout
and stderr to separate files. `umask 077` on every binary run. Nothing re-derived
that the brief listed as machine-checked.

| check | result |
| --- | --- |
| `classify()` over the five argv-forbidden carriers | `Codex32Secret`, `Mt`, `Tx`, `Passphrase`, `Mnemonic` — **all `forbidden=true`** |
| `classify()` callable pre-parse | yes — `sysw/mod.rs:178`, `&str` in, called from a test binary with no `Cli`; **0** `std::env` in its call graph |
| `classify(" tx:<hex>")`, `("TX:<hex>")`, `(" pass:…")`, `("MS1…")`, `("ABANDON…")` | **`Unknown`, `forbidden=false`** — all five |
| `classify("MT1…")`, `classify(" mt1…")` | **`Mt`, forbidden** — the one class that folds case/space |
| `me bundle TX:<447-char signed tx>` | rc 2, **transaction in stderr** |
| `me <MS1 uppercase>`, `me bundle <MS1 uppercase>` | rc 2, **secret in stderr**, both |
| donor gate rule | `by_prefix \|\| class.is_argv_forbidden()`, `main.rs:1979`; `by_prefix` on `trim_start().to_ascii_lowercase()`, `:1958`; the comment naming ` TX:<hex>` at `:1946-1949` |
| `classify("mt1-2026-08-23-transfer.txt")` / `("ms1-recovery-notes.txt")` / `("bundle")` / `("help")` / `("text:68656c6c6f")` | `Unknown` / `Unknown` / `Unknown` / `Unknown` / `FreeText` — **no false refusal** |
| `bundle`, `help`, `text`, `pass` in the BIP-39 English list; `pack`, `sysw`, `wipe`, `show` | present / absent, as the plan says |
| `me help` | rc **0** — usable as a positive control |
| `me bundle` (no args) | rc **2** — *"no input strings"* → **M-4** |
| `me sysw pack --out c.bin text:68656c6c6f` | rc **0** |
| `me --nosuchflag <ms1>` | rc **2**, clap names the flag, **no leak** — the ordering observable fails today |
| `me sysw pack --allow-argv-secret --out o.bin <ms1>` | rc **0**, 143-byte sealed payload |
| `me bundle/--/sysw wipe --allow-argv-secret <ms1>` | rc 2, clap names the **flag**, no leak |
| path-embedded secret, 4 surfaces + `--out` | rc 2, **secret in stderr**, 5 of 5; `classify(path)` = `Unknown` |
| `fold-propagation-check.sh` with this fold's retracted phrasings | **rc 1**, 4 hits: 272, 273, 474 ×2 |
| `git diff -U0 5cf8ac9..ce5a206` hunks (new-file lines) | **275-289, 310-313, 320, 532, 568** — lines 272-273 and 474 are context only |
| five carried round-9 Minors, `grep -c` | **1 each**, verbatim |
| `grep -n 'flag-name\|flag_is_secret'` over the plan / over `FOLLOWUPS.md` | **3 hits (283, 285, 340), none in §4** / **0 hits** |
| `mnemonic-toolkit` secret-bearing flags | `--passphrase`, `--passphrase-stdin`, `--bip38-passphrase`, `flag_is_secret` — `crates/mnemonic-toolkit/src/secrets.rs:59-88`. Fold's claim TRUE |
| cyclic package dependency | reproduced: `error: cyclic package dependency: package 'a-bin' depends on itself` |
| §4 step-row count | **12** (`1 2 3 4 5 6 7 8 9 9b 10 11`) |

---

## WHAT THE FOLD GOT RIGHT

Recorded so round 11 does not re-open it.

- **Reusing the classifier is the right answer, and it is the last one needed.**
  Three designs, and this is the first whose class set is defined by something
  that ships. Every claim the fold makes about the classifier's *behaviour*
  measures true — including the two that looked most like wishful thinking, that
  a filename beginning with an HRP is not refused and that `me bundle` survives
  despite `bundle` being a BIP-39 word. **C-2 is an argument for propagating this
  design into §3, never for going back to a list.**
- **The false-refusal problem is genuinely gone**, and gone without the charset
  rule round 9 prescribed. That is the better fix, arrived at by finding a
  mechanism rather than by adopting a remedy — the report's prescription was not
  treated as authoritative, correctly.
- **The ordering observable is now real, cheap and RED today.** It needs no cargo
  feature, no env hook and no edit to shipped code. Round 8 and round 9 both
  failed to produce one; this one works.
- **The positive controls bite.** `fn guard(_) -> ! { exit(3) }` fails three of
  them. The fold identified M-4 as the finding it would have shipped, and it was
  right about that.
- **Named-and-not-done stayed closed for a third round**, and the message again
  leads with the author's own error.

---

**VERDICT: NOT GREEN — 2 Critical, 4 Important.** No code may be written against
this plan.

**The one sentence for round 11.** The design question is answered — `classify()`
covers all five classes, runs pre-parse, refuses nothing legitimate, and the new
controls bite — so what is left is not a fourth guard but a **fold that reaches
§3**: the section that defines `records.rs` still describes the retracted
four-shape recogniser at two sites and still puts the guard inside a crate that
cannot call `me` without cargo refusing the workspace, which is why row 9b's gate
can no longer pass. **Propagate the adopted design into §3's tree, §3's table and
§3's representation ruling, decide whether the guard lives in `me` or behind a
predicate seam, add the donor's own trim-and-lowercase normalisation and one
near-miss carrier, and give the override and the flag-name layer each an owner.**
