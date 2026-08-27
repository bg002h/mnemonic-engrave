# R0 ROUND 8 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `3e18d2e`
(worktree `review/p0-r8`).
**Round 7's report:** `design/agent-reports/R0-P0-plan-round7.md` (1C/2I/8M/2N).
**The fold under review:** `3e18d2e` — 28 insertions, 19 deletions, 6 hunks.
**Object:** (1) did the fold close round 7's thirteen findings, verified against
the DIFF and not the commit message; (2) can an implementer execute all twelve
rows and can each gate fail; (3) do rows 6 and 7 as rewritten actually work —
both halves of the flag claim; (4) prose ↔ table contradictions.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **1** |
| **Important** | **2** |
| Minor | 7 |
| Nit | 2 |

**First, the thing the brief asked about most: the pattern is broken.** Every
item `3e18d2e`'s message claims as folded **is in the diff**. I checked all nine
claims. Nothing was named-and-not-done, and nothing is `diff`-identical to
`72868b9` while being reported as fixed. **Round 7's I-1 — five folds running —
is genuinely CLOSED**: row 7 is a real new row naming all five F-265 sites, with
a mutation gate that fails today, and `grep -c "the table carries a step that
edits all five"` → **0**. That is the first time in six folds that this finding
was performed rather than described.

**The failure moved.** It is no longer *claims of work that was not done*; it is
**measured facts that are false**, in the fold's own new prose, and they are the
facts the fold's central decision rests on.

**The fold's argument was:** `me` declares no secret-bearing flag → a real `ms1`
on a flag never reaches stderr → so the flag-name layer has no observable here →
so the guard is ONE layer, value-shape, and the gate is *"no `ms1` in stderr
across ALL surfaces — bare `me`, `bundle`, `sysw wipe`, `sysw show`, `sysw
pack`"*.

**Measured on today's binary, absolute path, `$?` never read through a pipe, the
fixture `me`'s own classifier calls a codex32 SECRET:** a real `ms1` on
`--in` — **a flag `me` declares** — reaches stderr on **four** invocations, and
**three more clap-shaped surfaces leak** that row 6's enumeration does not
contain. The gate covers **4 of the 11 leaking argv shapes I measured**, and
calls that set *ALL surfaces*.

The three flags the plan measured — `--mnemonic`, `--seed`, `--passphrase` — are
**not declared by `me`**. The negative was taken entirely over flags that do not
exist. **That is round-6 C-1's error verbatim** — *"a negative inheriting a
scope of one, and the one chosen was the exception"*, which is the sentence this
very plan uses to describe the defect it was fixing — one layer over, committed
by the fold that fixed it.

---

## QUESTION 1 — DISPOSITION OF ROUND 7's 1C/2I/8M/2N, AGAINST THE DIFF

`git show 3e18d2e` has six hunks: `@@ -9`, `@@ -266`, `@@ -269`, `@@ -540`,
`@@ -767`, `@@ -775`, `@@ -808`. **"claimed?"** = named in `3e18d2e`'s commit
message as folded.

| # | claimed? | disposition | line in the diff that closes it (or why not) |
| --- | --- | --- | --- |
| **C-1** row 6 unfolded, gate unfailable | yes | **CLOSED** | hunk `@@ -540,2 +549,2` replaces rows 6 **and** 7. Row 6 is now the value-shape guard; its gate *"which **FAILS on today's tree** (F-266)"* is RED — I reproduced 4 leaks among the five surfaces it lists. The corroborating *"the only gate here whose whole content is an ordering claim"* is gone. **Superseded by C-1 below**, which is about the enumeration, not the cell |
| **I-1** condition 10's false self-claim | yes | **CLOSED — genuinely** | hunk `@@ -775,4 +784,5`. `grep -c 'the table carries a step that edits all five'` → **0**. `awk` over §4 rows 543–556 now finds `refuse_write_block`, `read_records`, `emit` **and `WorldReadable` in row 7**, outside any STAY clause. Row 7's gate is a 2→3 mutation at each of the five, which I verified are exactly the five `EXIT_USAGE` refs in the functions that stay (`main.rs:997`, `:1001`, `:1928`, `:2048`, `:2092`) |
| **I-2** no digit on the end-to-end clause; error-path satisfiable | yes | **NOT CLOSED** | neither of round 7's two remedies was applied. The digit is still absent from the ALL-surfaces clause. The substitute is `grep -c 'env::args'` → non-zero, a **presence** test where an **ordering** test was asked for. **The commit message claims the property outright** — *"so a guard that has not moved off clap cannot pass"* — and it is false → **I-1 below** |
| **M-1** condition 9's dangling *"either"* | yes | **PARTIAL** | hunk `@@ -767 +776`: *"P0 **either** fixes"* → *"P0 fixes"* ✔. The **second** half round 7 named — *"two sentences still spliced with no punctuation between `)` and `**The`"* — survives verbatim at **776–777**, and the fold left the recipe parenthetical duplicated in the same sentence |
| **M-2** §7 *"updated in this same fold"* | yes | **CLOSED** | hunk `@@ -808,2 +818`: line **818** now *"`FOLLOWUPS.md` **records that reassignment**"* |
| **M-3** *"really 11 and not more"* | no | **NOT CLOSED** (carried) | line **668** byte-identical; not claimed |
| **M-4** `crates/me-cli/src/io.rs` | no | **NOT CLOSED** (carried) | line **545** byte-identical; `grep -n 'io\.rs'` → **1** hit, row 2 only; not claimed |
| **M-5** M5 paragraph made more wrong | no | **CLOSED as a side effect** | lines **583–593** untouched, but row 6 **is** now the value-shape guard, so *"Everything else is RED-first, including the value-shape argv guard"* resolves to a row whose gate does fail today. Round 6 predicted this: *"fixing C-1 should fix this one at the same time."* It did |
| **M-6** vacuous `fold-propagation-check.sh` in the header | yes | **CLOSED — and round 7 had the exit code wrong** | hunk `@@ -9 +9,5`. I ran it as its own command, streams separated: `./scripts/fold-propagation-check.sh <plan>` → **exit 2**, `no patterns given -- nothing to check` on **stderr**. The fold's header says 2; round 7's report said 0. **The fold is right and round 7 was wrong** — recorded so round 9 does not "fix" it back |
| **M-7** by-name refs not 1:1 | no | **NOT CLOSED, and one added** | *the signature change* ×3, *the crate adoption* ×1 unchanged. The fold introduced *"the digit-pinning work"* (line **784**); **four** rows pin a digit — 2, 4, 7, 8 → **M-5 below** |
| **M-8** enumeration attached to the move | no | **NOT CLOSED** (carried) | lines **496–513** byte-identical; not claimed |
| **N-1** doubled word *"while a / a"* | no | **CLOSED** | removed by the condition-10 rewrite; `grep -c 'while a$'` → **0** |
| **N-2** *"the moving set reference"* | no | **NOT CLOSED** (carried) | line **513** byte-identical; not claimed |

**Score: 1 of 1 Critical closed, 1 of 2 Important closed, 3 of 8 Minor closed
(+1 partial), 1 of 2 Nit closed.** **Zero findings were claimed as folded and
not folded.** The six carried Minors/Nits are all ones the commit message is
silent about — which is the correct behaviour, not the failure mode.

---

## CRITICAL

### C-1. Row 6's gate says "across ALL surfaces" and enumerates five. Measured, `me` echoes a codex32 secret to stderr on **eleven** argv shapes — four of them on `--in`, a flag `me` declares. The fold's evidence that no flag leaks was taken over three flags `me` does not declare.

**Site:** §3 lines **275–284** (the fold's new text) and **293–301**; §4 row **6**
(line **549**); §6 condition **8** (lines **770–772**).

**The claims under test**, verbatim from the fold:

> line 278: *"Measured: a real `ms1` on `--mnemonic`, `--seed` or `--passphrase`
> never reaches stderr, because clap names the flag."*
>
> line 283: *"**Every one of F-266's four leaking surfaces is an unexpected
> POSITIONAL**"*
>
> row 6's gate: *"**`no ms1 in stderr` across ALL surfaces** — bare `me`,
> `bundle`, `sysw wipe`, `sysw show`, `sysw pack`"*

**The measurement.** Binary by absolute path
(`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`), stdout and stderr
to separate files, `rc` from `$?` immediately after the call, never through a
pipe, stdin from `/dev/null`. Fixture is
`ms10entrsqqg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9pzs3gg5y2z9q5f042qmrw90mw`
— the one `me`'s **own classifier** calls SECRET (`me sysw pack <it>` → rc 3,
*"is SECRET key material on ARGV"*). `leak` is `grep -c` of a 53-char interior
substring of the body. `who` is whether the first stderr line starts with
`error:` (clap) or not (`me`'s own message).

```
### the five surfaces row 6 calls ALL
rc=2   leak=1   CLAP     me <ms1>
rc=2   leak=1   CLAP     me bundle <ms1>
rc=2   leak=1   CLAP     me sysw wipe <ms1>
rc=2   leak=1   me-msg   me sysw show <ms1>
rc=3   leak=0   -        me sysw pack <ms1>

### NOT in row 6's list -- clap-shaped
rc=2   leak=1   CLAP     me sysw <ms1>
rc=2   leak=1   CLAP     me help <ms1>
rc=2   leak=1   CLAP     me sysw help <ms1>
rc=2   leak=0   -        me seal <ms1>
rc=2   leak=0   -        me hash <ms1>

### NOT in row 6's list -- on a flag me ACTUALLY DECLARES
rc=2   leak=1   me-msg   me --in <ms1>
rc=2   leak=1   me-msg   me --in=<ms1>
rc=2   leak=1   me-msg   me bundle --in <ms1>
rc=2   leak=1   me-msg   me sysw pack --in <ms1>
rc=2   leak=0   -        me sysw show --in <ms1>
rc=0   leak=0   -        me sysw wipe --out <ms1>      <- creates a FILE named with the secret

### the plan's own flag measurement -- flags me does NOT declare
rc=2   leak=0   -        me --mnemonic <ms1>
rc=2   leak=0   -        me bundle --seed <ms1>
rc=2   leak=0   -        me sysw pack --passphrase <ms1>
```

**Eleven leaking shapes. Row 6's gate covers four of them.**

**The sharpest line in that table:** `me sysw pack <ms1>` is **clean at rc 3** —
it is the plan's own worked example of the post-parse guard working. **`me sysw
pack --in <ms1>` leaks.** Same subcommand, same secret, same binary; the only
difference is that the secret rides `--in` instead of a positional. `me`'s
message is verbatim:

```
me: cannot read ms10entrsqq…q5f042qmrw90mw: No such file or directory (os error 2)
```

**Three separate falsehoods, each load-bearing.**

1. **Line 278's measurement has an empty scope.** `--mnemonic`, `--seed` and
   `--passphrase` are **not declared anywhere in `me`**. I enumerated every
   `#[arg]` in `crates/me-cli/src/main.rs`: the value-taking flags are `--in`,
   `--out`, `--manifest`, `--preview`, `--plaintext`, `--iterations`,
   `--passphrase-words`, `--fill`. Those three are absent, so all three
   measurements are of clap's *unknown-flag* path, which names the flag by
   construction. **The general negative "a real `ms1` on a flag never reaches
   stderr" is refuted by `--in`, four ways.** This is round-6 C-1's exact shape —
   the plan's own words for it are *"A negative inheriting a scope of one, and
   the one chosen was the exception"* — repeated by the fold that closed it.

2. **Line 283's mechanism is wrong for one of its own four rows.** `me sysw
   show` **declares** a positional: `Show { file: std::path::PathBuf }`
   (`crates/me-cli/src/main.rs:275`), and `me sysw show --help` prints
   `Usage: me sysw show <FILE>`. Clap **accepts** the token; the leak is `me`'s
   own `No such file or directory` message, **after** the parse. So *"every one
   is an unexpected POSITIONAL"* is false, and with it the inference *"so
   value-shape is the layer that closes them."* Two of the four are a
   **post-parse path-echo**, not a clap echo — see **I-1**.

3. **Row 6's enumeration is a proper subset presented as a total.** `me sysw
   <x>`, `me help <x>` and `me sysw help <x>` all leak through clap and are not
   in it. Nor is any flag-borne form.

**The concrete failure.** An implementer builds the guard, closes the five
enumerated surfaces, and the gate goes green. §6 condition 8 closes. F-266 —
filed **Critical**, `#security`, owning phase P0, gating, and recorded as being
fixed by P0 *as a side effect* — is marked closed. **`me --in <ms1>` still
prints the operator's codex32 secret to the terminal**, as do `me sysw <x>`,
`me help <x>` and `me sysw help <x>`. Nothing in the plan can notice, because
the observable is a hand-written list of five.

**And a correctly built guard still misses one of them.** Row 6 specifies the
recogniser as *"`ms1`/`mt1` by HRP, `tx:` by prefix, a BIP-39 mnemonic by
wordlist"*, applied to raw `std::env::args()`. The argv token for
`me --in=<ms1>` is the single string `--in=ms10entrsqq…` — it does **not** begin
with an HRP. **The plan never says the recogniser splits on `=`,** so a guard
built exactly as written leaves that shape live, and the gate cannot see it
either. `me --in <ms1>` (space form) *is* caught, so the two spellings of the
same invocation diverge.

**What closes it.** Any of these; the requirement is that the surface list stop
being hand-written.

- **Derive the surfaces mechanically.** `me`'s clap tree is enumerable — walk
  every subcommand path and assert the property at each, so a subcommand added
  later is covered without editing the gate. This is the only form that cannot
  go stale.
- If it stays enumerated, it must at minimum add `me sysw <x>`, `me help <x>`,
  `me sysw help <x>`, and the **flag-value** forms `me --in <ms1>` /
  `me --in=<ms1>` / `me bundle --in <ms1>` / `me sysw pack --in <ms1>` — and
  then it must not be called *ALL*.
- **State that the recogniser inspects the value after `=`** in a
  `--flag=value` token.
- **Retract line 278.** Re-measure on flags `me` declares, and let the
  conclusion follow the measurement rather than the other way round.

---

## IMPORTANT

### I-1. `grep -c 'env::args'` → non-zero does not discriminate pre-parser from clap's error path. §6 condition 8 still closes with the parse having run first — round 7's I-2, round 6's C-1 part 3, third round — and the commit message asserts the property outright.

**Site:** §4 row **6** (line **549**); §6 condition **8** (lines **770–772**).

Row 6's second clause, verbatim:

> Plus `grep -c 'env::args'` over `main.rs` goes **0 → non-zero**: a guard that
> has not moved off clap cannot pass.

**The counterexample, constructed rather than asserted.** `std::env::args()` may
be called anywhere, including *after* the parser has run:

```rust
fn main() {
    let raw: Vec<String> = std::env::args().collect();   // grep -c 'env::args' == 1
    let cli = match Cli::try_parse() {                   // THE PARSE HAS RUN
        Ok(c) => c,
        Err(e) => {
            if let Some(k) = argv_secret_shape(&raw) {   // decided on the ERROR PATH
                eprintln!("me: a {k} was passed on the command line. Refused.");
                std::process::exit(3);
            }
            e.exit()
        }
    };
    …
}
```

Score this against row 6's gate, surface by surface:

| surface | outcome | gate |
| --- | --- | --- |
| `me <ms1>` | clap `Err` → guard fires, exit 3, clean | ✔ |
| `me bundle <ms1>` | clap `Err` → clean | ✔ |
| `me sysw wipe <ms1>` | clap `Err` → clean | ✔ |
| `me sysw pack <ms1>` | already clean at rc 3 | ✔ |
| `me sysw show <ms1>` | clap **`Ok`** — the positional is declared — guard never fires; still leaks | ✘ |

**One post-parse edit closes the last one**: change `sysw show`'s file-open
message so it does not echo the path. That is a three-line change in `me`'s own
error handling, entirely downstream of `Cli::parse()`. With it: **all five
surfaces green, `grep -c 'env::args'` == 1, `--allow-argv-secret` still parses,
row 6 green, condition 8 closed** — and the guard has reached its decision by
parsing first, which condition 8 states in terms:

> *"A guard that reaches its decision by parsing first has reintroduced the leak
> §6d exists to stop."*

**Why the `sysw show` row does not rescue the gate.** It looks like it
discriminates — it is the one surface where clap succeeds — but it discriminates
against the *error-path guard alone*, not against *post-parse implementations*.
The plan itself supplies the proof that a post-parse fix satisfies it: §3
records `me sysw pack <ms1>` as clean today, achieved entirely post-parse, with
`grep -c 'env::args'` → **0**.

**The plan still has no structural observable for ordering.** `grep -n
'env::args'` over the plan returns hits in §1, §3 and now row 6 — and **not one
of them asserts that the guard PRECEDES `Cli::parse()`**. The second half of
condition 8 — *"the override's own parse"* — is worse off: row 6 says *"The
override's own parse is decided there too"* with **no observable attached at
all**, and its only companion clause, *"`--allow-argv-secret` must still parse
afterwards"*, is a regression check that is green on the untouched tree.

**What closes it.** An observable that can distinguish the two, not a stronger
adjective. The forms available:

- a **donor test that calls the guard as a free function over `&[String]` with
  `Cli` never constructed** — structurally impossible to satisfy from clap's
  error path; or
- a **source-level assertion of order**, e.g. that the byte offset of the guard
  call in `main.rs` precedes that of `Cli::parse()` / `try_parse`; or
- an argv shape where clap **succeeds** and the guard must still refuse
  *pre-parse* — `me sysw pack --in <ms1>` is exactly that shape, and it is live
  today (**C-1**).

Whichever is chosen, **the sentence *"a guard that has not moved off clap cannot
pass"* must go**, because it is false as written.

### I-2. Row 6 hands the flag-name layer to `mnemonic-toolkit` and names the wrong predicate. The parity it says "is asserted there" does not exist, the toolkit has no pre-parser guard at all, and nothing schedules the layer §6d makes normative.

**Site:** §3 lines **280–281**; §4 row **6** (line **549**); §1 line **101**
(where §6d's *"Both layers run pre-parser (C-4)"* is quoted as inherited); §7
(line **810**).

The claims:

> §3: *"`mnemonic-toolkit` **does** declare such flags and proves the shape with
> `NodeType::is_argv_secret_bearing`; **the parity test is asserted there.**"*
>
> row 6: *"Flag-name parity against `mnemonic-toolkit`'s
> `NodeType::is_argv_secret_bearing` is asserted **there**, where such flags
> exist"*

**Half of it is true.** The toolkit does declare secret-bearing flag names — I
enumerated them: `--passphrase`, `--bip38-passphrase`, `--decrypt-password`,
`--ms1`, `--share`, `--secret-stdin` and more.

**The rest is wrong, and the toolkit's own source says so in the same words.**

- **`NodeType::is_argv_secret_bearing`
  (`crates/mnemonic-toolkit/src/cmd/convert.rs:117`) is not a flag-name
  predicate.** It is a predicate over **node types** in the `--from
  <node>=<value>` grammar: `self.is_secret_bearing() || matches!(self,
  Self::MiniKey)`. Whether `--from` carries a secret depends on the *value*, not
  the flag.
- **The toolkit's flag-name authority is `flag_is_secret`
  (`crates/mnemonic-toolkit/src/secrets.rs:60`)**, whose module doc is explicit:
  *"this `flag_is_secret` predicate covers FLAT flag-name-form secrets only"* —
  and, of `--from`/`--to`: *"secrecy is value-dependent … the GUI applies
  node-type-level secret classification via `secret_taxonomy::SECRET_NODE_TYPES`,
  **not flag-name-level**."* The plan cites the node-type predicate for the
  flag-name job the toolkit itself rules it out of.
- **The parity test named does not assert flag-name parity.**
  `secret_taxonomy_argv_parity_with_is_argv_secret_bearing`
  (`convert.rs:1998`) asserts `SECRET_NODE_TYPES_ARGV` ≡
  `is_argv_secret_bearing` across `ALL_NODE_TYPE_VARIANTS` — intra-toolkit
  taxonomy drift. It says nothing about flag names, nothing about
  `mnemonic-io-lib`, and nothing about pre-parser ordering.
- **The toolkit has no pre-parser guard to be parity with.** `grep -rn
  'env::args' crates/` over `mnemonic-toolkit` → **0**; `args_os` → **0**. Its
  argv handling is a **post-parse advisory** —
  `secret_advisory.rs::secret_in_argv_warning(stderr, flag, alternative)`,
  called from `convert.rs:1861`, `bundle.rs:2714`, `derive_child.rs:372`,
  `verify_bundle.rs:1885`. It warns after parsing; it does not refuse before it.

**The concrete failure.** §1 quotes §6d as normative and inherited: *"Both
layers run pre-parser (C-4)."* This fold drops one of the two from the crate
P0 exists to build, and discharges it by pointing at a test in another repo that
does not test it. Then:

- **§4 schedules no flag-name work** — row 6 builds value-shape only, and §3's
  file table gives `records.rs` *"the string-level recognisers (prefix / HRP /
  wordlist)"*, no flag-name set;
- **§6 has no condition for it**;
- **§7 puts `mnemonic-toolkit`'s adoption out of scope** — *"It is the sixth
  consumer, not P0's work"*;
- **no follow-up is filed.**

So the layer is owned by nothing, and the plan reads as though it were done. The
consumer that actually needs it is the one P0 declares out of scope.

**The gap is real but the fix is small,** which is why this is Important and not
Critical: a flag-name recogniser is a `&[&str]` membership test, and **it does
not need `me` to declare such a flag to be testable** — a crate-level unit test
over a fixture list can fail. The reason offered for dropping it ("no observable
HERE") only holds for an *end-to-end* observable in the donor.

**What closes it.** Any one of:

- name **`secrets.rs::flag_is_secret`** if that is the shape P0 adopts, and drop
  the `is_argv_secret_bearing` citation; **or**
- keep the layer in the crate with a **crate-level** gate that can fail —
  membership over a fixture list — and say plainly that the donor supplies no
  end-to-end case; **or**
- **file it**, with an owning phase, and change §3 and row 6 to say P0 ships one
  of §6d's two layers and why. What must not survive is a sentence asserting
  that a parity test exists in another repo when it does not.

---

## MINOR

**M-1. §3's prose says six surfaces; its table shows five; measured, there are
seven command surfaces and eleven leaking shapes.** Line **293** — *"`me` DOES
leak this way, on **four of six surfaces**"* — heads a code block (**296–301**)
with **five** rows. The "six" is inherited from `FOLLOWUPS.md`'s F-266 table,
which carries a sixth row (`me sysw pack --nosuchflag <ms1>`) that the plan
moved into prose. `me --help` lists four subcommands plus `help`, and
`me sysw --help` lists three plus `help`, so the surfaces that accept an argv
token number **eight** — bare `me`, `bundle`, `seal`, `hash`, `sysw`,
`sysw pack`, `sysw wipe`, `sysw show` — before counting `help` and
`sysw help`. Not six. Prose ↔ table, the brief's question 3. Pick one number and let the block
show it.

**M-2. Condition 9's sentence splice survived the fold that fixed its *"either"*,
and the fold added a duplication inside it.** Lines **776–777**:

> *"so P0 fixes the recipe (`fc -W`, edit, `fc -R`) **The remedy must make the
> recipe WORK** — flush, edit, reload (`fc -W`, `sed -i`, `fc -R`)."*

Two sentences with no punctuation between `)` and `**The`, and the recipe now
appears twice in one sentence. Round-4 M-8, round-5, round-6 M-1, round-7 M-1 —
half-fixed at the fifth attempt.

**M-3. Round 7's M-3, carried.** Line **668**: *"It is the step that proves the
closure is really 11 and not more."* The move relocates **five** functions plus
the `cfg(not(unix))` stub — §3's own row-2 cell lists them — so it proves nothing
about the other six. Not claimed in the commit message.

**M-4. Round 7's M-4, carried.** Line **545** still reads ``grep -c 'EXIT_'
`crates/me-cli/src/io.rs` == 0`` — nested backticks that will not render, and
`grep -n 'io\.rs'` over the plan returns **1** hit, this one. §3's tree names
seven modules (`channel.rs`, `fd.rs`, `observation.rs`, `records.rs`, `exit.rs`,
`remedy.rs`, `lib.rs`) and none is `io.rs`, so an implementer following §3 runs
`grep` against a file that never exists — exit 2, no count, and the clause reads
as passing. Not claimed.

**M-5. Round 7's M-7 carried, and the fold added a fourth ambiguous by-name
reference.** *the signature change* ×3, *the crate adoption* ×1 unchanged. New:
**"the digit-pinning work"** (line **784**). `awk` over §4's rows finds **four**
rows that pin a digit — **2** (*"asserting the exit DIGIT"*), **4** (*"pinning
the **exit digit**"*), **7** (*"pin the exit digit at ALL FIVE sites"*) and **8**
(*"every refusal below pins its exit DIGIT"*). Condition 10's context resolves it
to row 7, which is why it stays Minor, but the plan's own rule requires a name
that resolves 1:1 (*"rationale refers to work by NAME … so a renumbering cannot
falsify it"*). *"the five-site digit pin"* would.

**M-6. Round 7's M-8, carried.** The *"enumerate every type and constant"*
requirement (lines **496–513**) is still attached to the move, where everything
lands in `me`'s own lib half alongside `Class` and `E0116` cannot occur; it has
teeth only at 9b. Backstopped by 9b's *"no `Class` in it"*, so it is a
mis-assignment rather than a hole. Not claimed.

**M-7. Round 7's M-6 is closed, but round 7 measured the exit code wrong and the
plan now records the right one — noting it so it is not "corrected" back.** I
ran `./scripts/fold-propagation-check.sh design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`
as its own command with the streams separated: **exit 2**, and `no patterns given
-- nothing to check` on **stderr**, not stdout. The header (lines **9–13**) says
2. Round 7's evidence table says *"exit **0**"*, which is what reading the code
through a pipe returns. No change needed to the plan; this entry exists so
round 9 does not fold a correct line into an incorrect one.

---

## NIT

**N-1. Round 7's N-2, carried.** Line **513**: *"The move must enumerate every
type and constant **the moving set reference**"* — the residue of the *"the 11"*
→ *"the moving set"* replace, subject-verb. Not claimed in the commit message.

**N-2. There is a twelfth leak shape and it is not stderr.** Measured:
`me sysw wipe --out <ms1>` exits **0** having created a file in the working
directory **whose NAME is the secret** — a copy on disk, surviving the shell
session, at exit 0 with no message. It is outside F-266 (which is about stderr)
and outside row 6's observable (likewise), and the pre-parser guard row 6
specifies would refuse it, so nothing needs scheduling. Recorded because the
plan describes its observable as covering *"the leak"* without ever saying the
observable is **stderr-only**.

---

## QUESTION 2 — CAN AN IMPLEMENTER EXECUTE ALL TWELVE ROWS, AND CAN EACH GATE FAIL?

§4 carries **12** step rows — `1, 2, 3, 4, 5, 6, 7, 8, 9, 9b, 10, 11` (counted
from the table, not from prose). The diff touches only rows **6** and **7**, so
rows 1–5 and 8–11 are round 7's findings unchanged and are not re-derived here.

| # | executable? | can its gate fail? | evidence |
| --- | --- | --- | --- |
| 1 signature change | yes | yes | `EXIT_*` count inside `no_records_guard` is 1 today (`main.rs:1896`), must reach 0 |
| 2 the move | yes | yes | the pty assertion's AFTER half, digit pinned. One grep clause names a path the plan contradicts (**M-4**); the row's other content still fails |
| 3 mask split | yes | yes | `0o620 & 0o044 == 0`, so `Some(0o620)` is RED against a masked implementation |
| 4 `observation.rs` + pty | yes | yes | F-259 live; the probe re-wrote the bug under 391/391 green and the assertion caught it |
| 5 `remedy.rs` | yes | yes | F-264 live; *"RUN under a real interactive zsh"* is mechanical and RED today |
| **6 the argv guard** | **yes** | **yes, but green while the leak is live** | RED today — I reproduced 4 leaks among its five surfaces. **But it covers 4 of 11 leaking shapes (C-1)** and is satisfiable post-parse (**I-1**) |
| **7 F-265 five-site digit pin** | **yes** | **yes** | the five sites resolve to real code: `main.rs:997`, `:1001` (`refuse_write_block` Terminal / WorldReadable), `:1928`, `:2048` (`read_records` `--in` / stdin), `:2092` (`emit` write failure) — every one an `EXIT_USAGE`, so 2→3 is a real mutation. The gate is *"turns the suite RED"* and today it does not. **Correctly ordered**: it precedes row 9, which is what changes `refuse_write_block`'s signature |
| 8 `--expect` | yes | yes | the flag does not exist; digit pinned |
| 9 `exit.rs` + `channel.rs` | yes | yes | *"`-` is IMPLEMENTED"* is RED — §3 measures `-` reading stdin nowhere in `me` today |
| 9b create the crate | yes | yes | the crate does not exist; *"no `EXIT_*` and no `Class` in it"* is checkable at the one moment it can fail |
| 10 consume | yes | yes, as regression | count is not stale |
| 11 publish | n/a | n/a | operator-gated |

**Row 7 is the model of what was asked for** and deserves saying plainly: named
sites, each resolvable to a line, a mutation that is proven not to be noticed
today, and *"with the line executing"* so a mutation that never ran cannot be
mistaken for a passing gate.

**Row 6 is executable and its gate does fail today.** What it cannot do is
distinguish a compliant implementation from two non-compliant ones — a
narrower one (**C-1**) and a later one (**I-1**).

---

## QUESTION 3 — PROSE ↔ TABLE. Three contradictions, two of them new.

| site | prose says | the table / the code says |
| --- | --- | --- |
| §3 **293** vs **296–301** | *"four of **six** surfaces"* | the block has **five** rows; `me --help` + `me sysw --help` give **eight** token-taking surfaces (**M-1**) |
| §3 **283** vs `main.rs:275` | *"every one … is an unexpected POSITIONAL"* | `Show { file: std::path::PathBuf }` — declared, expected, consumed; the leak is post-parse (**C-1**) |
| §3 **278** vs §4 row **6** | *"a real `ms1` on a flag never reaches stderr"* | `me --in <ms1>` → rc 2, secret in stderr (**C-1**) |

**Resolved this round:** row 6's old *"it is the only gate here whose whole
content is an ordering claim"* — the contradiction round 7 raised — is gone with
the cell. And condition 10's *"the table carries a step that edits all five"* is
now true of the table: `grep` finds all five names in row 7.

---

## WHAT I VERIFIED HERE

Absolute paths throughout. `$?` read immediately, never through a pipe. stdout
and stderr to separate files. Nothing re-derived that the brief listed as
machine-checked.

| check | result |
| --- | --- |
| every claim in `3e18d2e`'s commit message vs the diff | **9 of 9 present** — no named-and-not-done |
| `grep -c 'the table carries a step that edits all five'` | **0** — I-1 closed |
| `awk` over §4 rows 543–556 for F-265's five sites | `refuse_write_block`, `read_records`, `emit`, `WorldReadable` → **row 7**, outside any STAY clause |
| the five F-265 sites resolve to code | `main.rs:997`, `:1001`, `:1928`, `:2048`, `:2092` — all `EXIT_USAGE`, all in functions §3 keeps in `me` |
| §4 step-row count | **12** (`1 2 3 4 5 6 7 8 9 9b 10 11`) |
| `me`'s value-taking flags, enumerated from every `#[arg]` | `--in --out --manifest --preview --plaintext --iterations --passphrase-words --fill`. **`--mnemonic`, `--seed`, `--passphrase` are not declared** |
| `me --in <ms1>` / `--in=<ms1>` / `bundle --in` / `sysw pack --in` | rc=2, **leak=1**, `me`'s own message |
| `me sysw <ms1>` / `me help <ms1>` / `me sysw help <ms1>` | rc=2, **leak=1**, clap |
| `me seal <ms1>` / `me hash <ms1>` | rc=2, leak=0 |
| `me sysw wipe --out <ms1>` | **rc=0**, file created whose name is the secret |
| `me sysw pack <ms1>` (me-classified secret fixture) | rc=**3**, clean — matches §3 |
| `me sysw show --help` | `Usage: me sysw show <FILE>` — the positional is declared |
| `NodeType::is_argv_secret_bearing` | `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/convert.rs:117` — a **node-type** predicate |
| `flag_is_secret` | `mnemonic-toolkit/crates/mnemonic-toolkit/src/secrets.rs:60` — the flag-name predicate, doc says node-type is *"not flag-name-level"* |
| `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` | `convert.rs:1998` — asserts `SECRET_NODE_TYPES_ARGV` ≡ the predicate; not flag names, not `mnemonic-io-lib` |
| `grep -rn 'env::args' crates/` in `mnemonic-toolkit`; `args_os` | **0**; **0** — no pre-parser guard there either |
| `fold-propagation-check.sh <plan>` with no patterns, streams separated | **exit 2**, message on **stderr** — the fold's header is right, round 7's report was wrong |
| lines 545, 668, 496–513, 583–593 vs `72868b9` | `diff` → **IDENTICAL** |
| `grep -c 'while a$'` | **0** — N-1 closed |
| `grep -n 'io\.rs'` | **1** hit, row 2 only — M-4 carried |

---

## WHAT THE FOLD GOT RIGHT

Recorded so round 9 does not re-open it.

- **The named-and-not-done pattern is broken.** Five folds running, this report
  had to open with a finding of that shape. This one does not have one. Every
  claim in the message is in the diff, and the six carried Minors are ones the
  message is silent about — which is the correct behaviour, not a lapse.
- **Row 7 is the finding actually performed.** Not a rewritten sentence: a new
  row, five sites that each resolve to a line of shipped code, a mutation gate
  proven not to be noticed today, and *"with the line executing"* so a mutation
  that never ran cannot pass for a gate. This is what round 4, 5, 6 and 7 asked
  for.
- **Dropping the flag-name layer was the right instinct even though the
  supporting facts are wrong.** `me` genuinely declares no secret-bearing flag —
  I enumerated every `#[arg]`. One layer in this donor is simpler and more
  honest than two. The defect is the *evidence* offered (**C-1**) and *where the
  remainder was said to land* (**I-2**), not the decision.
- **The commit message leads with the author's own worst failure**, names it as
  the fifth occurrence, and does the work. That is the behaviour that ends this
  class of finding.
- **M-6 was folded correctly against a report that had the fact wrong** — the
  header says exit 2 and exit 2 is what the script returns. Not deferring to the
  reviewer's number is exactly right.

---

**VERDICT: NOT GREEN — 1 Critical, 2 Important.** No code may be written against
this plan.

**The one sentence for round 9.** The fold stopped describing findings instead
of doing them — that pattern is closed — and the failure moved one level down,
into **measured facts with a scope too small to support the sentence built on
them**: three flags that do not exist standing in for "no flag leaks", four
surfaces standing in for "ALL surfaces", and a parity test in another repo
standing in for a layer nobody schedules. **The remedy is the rule this repo
already wrote down — *a negative is only as wide as what you searched* — applied
to the fold's own evidence before it is committed: enumerate the flags `me`
declares, walk the subcommand tree, and open the file in the other repo.** All
three are one command.
