# R0 ROUND 9 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `5cf8ac9`
(worktree `review/p0-r9`).
**Round 8's report:** `design/agent-reports/R0-P0-plan-round8.md` (1C/2I/7M/2N).
**The fold under review:** `5cf8ac9` — 29 insertions, 14 deletions, 2 hunks
(§3 lines 275–302, and §4 row 6). Nothing else in the plan changed.
**Object:** (1) does the token-scan design hold — missed shapes, constructibility
of the ordering observable, false refusals; (2) did the fold close round 8's
1C/2I/7M/2N, verified against the DIFF; (3) can an implementer execute all twelve
rows and can each gate fail.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **1** |
| **Important** | **4** |
| Minor | 10 |
| Nit | 1 |

**The fold did the right thing and did it one class short.** Replacing the
hand-written surface list with a token scan is correct, and it genuinely closes
the surface half of round 8's Critical: the cross-product covers `me sysw <x>`,
`me help <x>`, `--in X` and `--in=X`, every one of which a hand list missed. The
sentence *"a token scan has no surface list to be short"* is true.

**But it traded a surface list for a SHAPE list, and the shape list is short.**
`me`'s own argv-forbidden set is five classes —
`is_argv_forbidden() = is_secret() || is_bearer()` over
`{Mnemonic, Codex32Secret, Passphrase, Mt, Tx}`
(`crates/me-cli/src/sysw/record.rs:105`, `:73`, `:89`). The recogniser the plan
specifies covers **four**. The missing one is `Class::Passphrase` — the `pass:`
record — which `me` itself refuses on argv at rc 3 with the words *"is SECRET key
material on ARGV"*, and which **`pass:` appears zero times in this plan**.
Measured today, a `pass:` record reaches stderr verbatim on **7 of 8** argv
shapes. Row 6's cross-product carries exactly one carrier value — an `ms1` — so
**the gate is structurally incapable of failing on it.**

That is round-8 C-1 one level down: an enumeration that reads as total, a
negative taken over the wrong scope, and a gate whose axes cannot see the gap.

---

## QUESTION 1 — DOES THE TOKEN-SCAN DESIGN HOLD?

### Method

Every binary by absolute path
(`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`). stdout and stderr
to separate files. `rc` from `$?` immediately after the call, never through a
pipe. stdin `/dev/null`. `leak` is a `grep -qF` for a fixture-body substring in
**stderr only**. Fixtures: the `ms1` round 8 used, and
`pass:68756e74657232` — `pass:` + hex of `hunter2`, the exact form
`me sysw pack --help` documents as *"a BIP-39 passphrase"*.

### 1a. A secret split across two tokens — **gap, Minor (M-2)**

`me abandon abandon … about` (twelve tokens): clap names only the first word.
No full-phrase leak today. But the phrase is in `/proc/<pid>/cmdline` and the
shell history, which is the hazard `is_argv_forbidden`'s own doc names and §6h's
purge machinery exists for. A per-token scan cannot see it, and a per-token
*membership* test instead refuses `me bundle` and `me help` — **both are BIP-39
English words** (verified against `bip39-2.2.2/src/language/english.rs`, 2048
words; `bundle`, `help`, `text`, `pass`, `secret`, `expect`, `art`, `index`,
`key`, `card`, `file` all members). **Three of the cross-product's eight
subcommand shapes contain such a token** — `bundle`, `help` and `sysw help`;
`pack`, `show`, `wipe`, `sysw`, `seal` and `hash` are not wordlist members. The plan states no granularity rule and both
readings are available from its text.

### 1b. A secret in an environment variable — **not a finding**

`me` declares no `env = ` clap attribute. Out of the guard's stated scope, and
§6d scopes it to `std::env::args()`. Recorded so it is not re-derived.

### 1c. A secret reaching stderr via a PATH — **gap, Minor (M-3)**

```
rc=2  leak=SECRET  me --in /tmp/<ms1>.txt
rc=2  leak=SECRET  me sysw pack --in /tmp/<ms1>.txt
rc=2  leak=SECRET  me sysw show /tmp/<ms1>.txt
```

Row 6's gate says *"no `ms1` in stderr for **ANY argv containing one**"*. These
are argv containing one. The recogniser as specified is **anchored** — `by HRP`,
`by prefix`, plus one named decomposition (`=`-split) — so the token
`/tmp/ms10entrsqq…txt` matches nothing. The gate's own axes (`{positional,
--in X, --in=X}` with `X` a bare `ms1`) cannot reach it either.

**This pulls directly against I-1 below**, and the plan must say which side of
the trade it takes: substring matching closes 1c and re-opens the false-refusal
incident `mt` already had.

### 1d. `--` end-of-options — **holds**

`me sysw pack -- <ms1>` is already clean at rc 3; `me -- <ms1>` leaks today and
a token scan sees the `<ms1>` token regardless of the `--`. No finding.

### 1e. stdin-vs-argv — **holds for P0's ordering**, noted only

`me sysw pack --help` already advises *"prefer `--in` or stdin"*, and `--in` is
a working private channel today. §3's separate finding that `-` reads stdin
nowhere is row 9's, unchanged by this fold.

### 1f. Can the ordering observable actually be built? — **Important (I-4)**

**No, not as written, and not without editing non-test code.** `Cli` and
`Cli::parse()` are declared in `crates/me-cli/src/main.rs` (`fn run() -> i32 {
let cli = Cli::parse();` at **main.rs:304-305**). A binary crate root is not
importable; no integration test in `crates/me-cli/tests/` can interpose on it,
and every one of those tests reaches the binary by spawning it
(`assert_cmd::cargo::cargo_bin("me")`). Making `Cli::parse()` panic requires a
`panic!` **committed into `main.rs`**, reached by one of two mechanisms — and
`crates/me-cli/Cargo.toml` has **no `[features]` section** at all.

Both mechanisms have a consequence the plan does not state:

- **A cargo feature.** The feature-on and feature-off binaries cannot coexist in
  one cargo invocation — `target/debug/me` is one path. So the ordering test
  cannot run in the same command as the other 388. Under the project's standard
  command, `cargo nextest run --locked` (no features), the test either runs
  against the *normal* binary — where the guard fires, exit 3, **and it passes
  trivially, forever** — or is itself `#[cfg(feature)]`-gated and **never
  executes**. This repo's own rule: *"a plan may not close while any of its own
  gates has never been run."*
- **An env-var hook** (`if env::var_os("ME_ORDERING_PROBE").is_some() { panic!()
  }` before the parse). One build, test can fail, runs under the standard
  command — but it ships a parser-disabling hook inside a security binary, which
  is a ruling this plan should make rather than an implementer discover.

Round 8's I-1 offered two forms that need neither (a free-function guard called
over `&[String]` with `Cli` never constructed; a source-order assertion). The
plan took a third and left the mechanism unnamed.

### 1g. Does refusing any token by shape create false refusals? — **YES, Important (I-1)**

Measured, on today's binary, both at **rc 0**:

```
rc=0  me sysw pack --no-passphrase --out out.bin --in mt1-2026-08-23-cold-storage-transfer.txt
rc=0  me sysw pack --no-passphrase --out out.bin --in ms1-recovery-notes.txt
```

Both filenames begin with the HRP. A guard built to row 6's rule — *"`ms1`/`mt1`
by HRP"* — refuses both.

**The sibling repo already shipped this bug, and its source says so in the
words the plan needs:**

> *"**THE CHARSET TEST IS NOT OPTIONAL, and leaving it out refused a legitimate
> input.** `mt verify --in mt1-2026-08-23-cold-storage-transfer.txt` is a
> recovery-path invocation with a perfectly sensible FILENAME — 40 characters,
> beginning `mt1` — and a length-and-prefix rule called it a bearer leak, with a
> verdict line stating something false about what the operator had done. **An
> over-correction that blocks a valid path is worse than the silence it
> replaced**, because it stops someone who is doing everything right, at the
> moment they are trying to recover money."*
> — `looks_like_a_transaction`, in `mt-cli`'s validate module (sibling repo,
> `mnemonic-transaction`), the comment above the `mt1` arm. Its actual rule is
> **prefix + bech32 charset + `body.len() >= 37`**, and its whole-guard shape is
> `for a in args.iter().skip(1)` — a token scan, i.e. exactly the design this
> fold adopted, with the charset test the plan omits.

`grep -nic 'bech32\|charset\|alphabet'` over the plan → **0**. Neither §3 nor
row 6 mentions a charset or a length bound. And row 6's gate is **green on a
refusal** — it asserts only that no `ms1` is in stderr — so §4 cannot catch the
over-correction at any step.

---

## CRITICAL

### C-1. The recogniser omits `pass:` — a class `me` itself calls SECRET on argv. Measured, it leaks on **7 of 8** shapes, and the cross-product carries only an `ms1`, so the gate cannot fail on it.

**Site:** §3 line **272** (*"`tx:` by prefix, `mt1`/`ms1` by HRP, a BIP-39
mnemonic by wordlist"*); §4 row **6**, line **567** (same three, verbatim);
§3's file table row for `records.rs` (*"the string-level recognisers (prefix /
HRP / wordlist)"*). `grep -n 'pass:'` over the plan → **0 hits**.

**What `me` already believes.** `Class::is_argv_forbidden`
(`crates/me-cli/src/sysw/record.rs:105`) is `is_secret() || is_bearer()` over
`{Mnemonic, Codex32Secret, Passphrase}` ∪ `{Mt, Tx}` — **five** classes.
`classify_with` (`crates/me-cli/src/sysw/mod.rs:184`, the `pass:` arm at `:203`) produces
`Class::Passphrase` from the `pass:` prefix (`record.rs:28`), and
`me sysw pack --help` documents it as a first-class record:
*"`pass:<hex of the UTF-8 bytes>` is a BIP-39 passphrase."*

**The refusal `me` gives today, post-parse:**

```
$ me sysw pack pass:68756e74657232        rc=3
me: record 0, as given (records count from 0), is SECRET key material on ARGV.
    Refused; nothing was read and nothing was written.
```

**The leak, measured on every other shape:**

```
rc=2   leak=pass_hex  CLAP   me <pass:>
rc=2   leak=pass_hex  CLAP   me bundle <pass:>
rc=2   leak=pass_hex  CLAP   me sysw <pass:>
rc=2   leak=pass_hex  CLAP   me help <pass:>
rc=2   leak=pass_hex  CLAP   me sysw wipe <pass:>
rc=2   leak=pass_hex  post   me sysw show <pass:>
rc=3   leak=0         post   me sysw pack <pass:>      <- the only one that refuses
rc=2   leak=pass_hex  post   me sysw pack --in <pass:>
```

Bare `me` prints it verbatim: `error: unrecognized subcommand
'pass:68756e74657232'`.

**Three consequences, each load-bearing.**

1. **The pre-parser guard would be WEAKER than the post-parse guard it is
   replacing.** `me sysw pack` refuses a `pass:` record on argv today. P0's whole
   argument is that a post-parse guard has already lost — and the replacement, as
   specified, does not recognise the class at all. On six clap-echoing surfaces
   the passphrase is printed; on `sysw pack` the new guard is silent and the old
   one still catches it, so nothing regresses *there* and nothing warns anywhere
   else.
2. **No gate in §4 can fail on it.** Row 6's cross-product is
   `{8 subcommand shapes} × {positional, --in X, --in=X}` with a single carrier
   value, an `ms1`. The subcommand axis was generated precisely so a missing
   surface could not hide; **the carrier axis is a hand-written singleton**, and
   that is where the gap now lives.
3. **§6 condition 8 closes anyway.** It asks only that the guard and the override
   be decided pre-parse. A four-shape guard satisfies it, F-266 is marked fixed
   *as a side effect* as the brief describes, and `me pass:<hex>` still prints
   the operator's wallet passphrase to the terminal.

**Why this is the same finding as round 8's, not a new one.** The fold's own
sentence is *"Enumerating surfaces is what produced round-8's Critical; a token
scan has no surface list to be short."* Correct — and it has a **shape** list,
which is short by one class, and a **carrier** list, which is short by four. The
fix for "a negative is only as wide as what you searched" was applied to the
surface axis and not to the other two.

**What closes it.** The requirement is that the shape set stop being
hand-written:

- **Derive the recogniser's classes from `me`'s own argv-forbidden set** — name
  `is_argv_forbidden()`'s five classes in §3 and row 6, and add the `pass:`
  prefix to the recogniser. That set is already the donor's ruling
  (*"RULED 2026-08-26: we want uniform behavior with secret bearing between ms1
  and passwords and mt1 to the extent we can"*, `record.rs:105`), so P0 is
  adopting it, not inventing it.
- **Make the carrier axis a set, not a value** — one fixture per forbidden
  class (`ms1`, `mt1`, `tx:`, `pass:`, a quoted mnemonic) crossed with the eight
  surfaces and the three carriers. That is the form that cannot go stale, and it
  fails today on `pass:` at 7 shapes.
- **State the observable is stderr-only**, or widen it (round 8's N-2 and M-3
  above both live outside it).

---

## IMPORTANT

### I-1. The recogniser is a bare prefix rule. The sibling repo shipped exactly that, refused a funds-recovery invocation with it, and its source says the charset test "is not optional". The plan mentions neither charset nor length, and row 6's gate is green on a false refusal.

**Site:** §3 line **272**; §4 row **6** line **567**.

Evidence, method and the sibling quotation are in **§Question 1g** above.
Measured: `me sysw pack --in mt1-2026-08-23-cold-storage-transfer.txt` → **rc 0**
today; `--in ms1-recovery-notes.txt` → **rc 0** today. Both are refused by the
rule as written.

**The concrete failure.** An operator recovering from steel points `--in` at the
file they named after the transaction. P0's new guard refuses it *before the
parser*, so there is no path around it, and the refusal text — which §6d
mandates says *"it is now in your shell history"* — states something false about
what they did. The plan's own §3 already rates this class: *"A false refusal
carrying a false message, which is C-1's shape reproduced inside the feature P0
is adding."*

**What closes it.** State the recogniser's rule to the precision `mt` already
uses: HRP prefix **plus** bech32 charset over the body **plus** a minimum body
length, with `-`, `.`, `/`, `_` explicitly outside the alphabet. Cite the
sibling's comment as the reason rather than re-deriving it. And add one
cross-product row asserting a **legitimately-named file still packs at rc 0** —
without it no gate here can distinguish a correct guard from one that refuses
too much.

### I-2. §6d makes the flag-name layer the PRIMARY layer and assigns "the union" to P0. The plan calls it a belt, does not build it, files nothing, and justifies the drop with "strictly stronger" — which §6d's own worked example refutes and the plan's very next sentence contradicts.

**Site:** §3 lines **286–290**. Spec: `SPEC_constellation_cli_uniformity.md`
lines **824–841**.

The fold's text:

> line 286: *"**P0 owns it, and the token scan above subsumes it.**"*
> line 288: *"…which is **strictly stronger** than a flag-name list…"*
> lines 289–290: *"The flag-name layer stays specified in §6d as a **belt** for
> material a shape test cannot recognise; **P0 does not build it, and no gate
> here pretends to.**"*

**"Strictly stronger" is false, and line 290 says so.** If there is material a
shape test cannot recognise, the shape test is not stronger than the layer that
catches it. §6d supplies the counterexample in terms:

> line 824: *"So `mnemonic bundle --passphrase <arbitrary text>` is invisible to
> both, and does not exist in either source; **it must build the union**."*
> line 830: *"**FLAG-KEYED, and this is the primary layer.** A flag declares
> whether its value is secret-bearing, and the value needs no recognisable
> shape. **This is how a passphrase — arbitrary text, indistinguishable from a
> filename — is caught at all.**"*
> line 841: *"**Both layers run pre-parser (C-4).**"*

So the plan demotes the layer §6d calls **primary** to a *belt*, and hands the
job of catching arbitrary text to the layer §6d says cannot do it.

**Round 8's I-2 is not closed.** Its false citation is gone — correctly, and the
toolkit facts are now right. But its stated core was *"the layer is owned by
nothing, and the plan reads as though it were done"*, and its remedies were:
name `flag_is_secret`; **or** keep the layer with a crate-level gate that can
fail; **or** **file it with an owning phase**. The fold took a fourth path and
supported it with a false claim. Verified now:

| where the layer could be owned | state |
| --- | --- |
| §4's twelve step rows | no row builds it |
| §6's eleven closing conditions | no condition mentions it |
| §7 out-of-scope | not listed |
| `design/FOLLOWUPS.md` | `grep -n 'flag-name\|flag_is_secret'` → **0 hits** |

**"P0 owns it" and "P0 does not build it" four lines apart, with nothing else
scheduled, is the same hole with an affirmative sentence over it.**

**What closes it.** Either build the membership layer in the crate with a
crate-level gate over a fixture flag list (which can fail without `me`
declaring such a flag), **or** file it in `FOLLOWUPS.md` with an owning phase and
say plainly that P0 ships one of §6d's two layers. What must not survive is
*"strictly stronger"*, and a plan that both owns and declines a normative layer.

### I-3. `--allow-argv-secret` has no step and no observable, and row 6 as written refuses it. Condition 8 still demands it. Round 8's I-1 second half regressed from "no observable" to "not scheduled".

**Site:** §4 row **6** line **567**; §3 lines **307–309**; §6 condition **8**,
line **788**.

The old row 6 carried *"The override's own parse is decided there too, and
`--allow-argv-secret` must still parse afterwards."* Round 8 called that clause
observable-free. **The fold deleted the clause.** `grep -n
'allow-argv-secret\|override'` over the plan now returns **4 hits: 175, 307,
308, 788 — none of them in §4.**

What still demands it:

> §3 **307–309**: *"**The override's own parse must also run pre-parser**, or
> `--allow-argv-secret` cannot be honoured without parsing the very argv the
> guard exists to protect."*
> §6 condition **8**, line **788**: *"**The guard AND the override's own parse
> are both decided before `Cli::parse()`**, asserted at least in the donor."*

**And row 6, as written, breaks it.** *"any token that looks like secret material
by value shape … **refuses, wherever it sits**"* — no carve-out. Measured today:

```
$ me sysw pack --allow-argv-secret --out /tmp/p1.bin <ms1>      rc=0
  SEALED — this payload holds secret material (record 0 (codex32 secret))
  -rw------- 143 bytes written
```

A documented, shipped escape hatch (`main.rs:252`, honoured at `:1959`, named in
`me`'s own refusal text at `:2022`) that a guard built to row 6 refuses.
**The cross-product gate contains no override row and asserts only absence, so a
refusal passes it 24/24.** The only backstop is §6 condition 1 and the fifteen
existing test references to `--allow-argv-secret`
(`tests/sysw_cli.rs` ×13, `tests/cli.rs` ×2) — a regression catch, not a gate,
and it arrives as a surprise mid-step rather than as the RED the column promises.

**And the pre-parse override is not the trivial string test it looks like.**
`--allow-argv-secret` is declared on `me sysw pack` **alone**. A guard that
honours the bare token anywhere in argv hands `me bundle --allow-argv-secret
<secret>` straight to clap. (Measured: today that particular shape does not leak,
because clap names the unknown *flag* rather than the value — which is the
round-6 exception all over again, and not something to build on.)

**What closes it.** A row in §4 that builds the override's own pre-parse
recognition, scoped to where the flag is declared, with an observable: at
minimum `me sysw pack --allow-argv-secret <ms1>` still exits 0, and
`me bundle --allow-argv-secret <ms1>` still refuses. Both are one line each and
both can fail.

### I-4. The ordering observable cannot be built as written without editing non-test code, and under the project's standard command it is either trivially green or never run.

**Site:** §4 row **6** line **567**.

Full evidence in **§Question 1f** above. In summary: `Cli::parse()` is in
`crates/me-cli/src/main.rs:305`, unreachable from any test; `crates/me-cli/Cargo.toml`
has **no `[features]` section**; the tests spawn the binary via
`assert_cmd::cargo::cargo_bin("me")`, one path per profile. So the panicking
build and the normal build cannot coexist in one `cargo nextest run --locked`,
and the gate lands on one of the two failure modes this cycle has already hit
seven times — a test that passes trivially, or a gate that never executes.

**What closes it.** Name the mechanism in the row: either the cargo feature
**plus the separate command that runs it** (and say the standard suite does not
cover it), or the env-var hook **plus the ruling that a parser-disabling hook may
ship in `me`**, or fall back to one of round 8's two mechanism-free forms — a
free-function guard unit-tested with `Cli` never constructed, or a source-order
assertion on the byte offsets of the guard call and `Cli::parse()` in `main.rs`.

---

## MINOR

**M-1. §3's retracted claims survive eleven and nineteen lines below their own
retraction — round-8 C-1 part 2 and round-8 M-1, neither closed.**
Line **300** (the fold's): *"**F-266's leaks are not four and not all
positional**"*. Line **311** (untouched): *"`me` DOES leak this way, on **four of
six surfaces**"*. Line **319** (untouched): *"`me sysw pack <ms1>  rc=3  clean —
**the only surface taking a positional**"*. The last is false and measurably so:
`Show { file: std::path::PathBuf }` (`crates/me-cli/src/main.rs:275`), and
`me sysw show --help` prints `Usage: me sysw show <FILE>`. Nothing downstream
rests on it any more — row 6 is a token scan — which is why it is Minor rather
than a repeat of the Critical. Delete both, or let the block show the measured
numbers.

**M-2. The wordlist recogniser's granularity is unstated, and both available
readings are wrong in a different direction.** See §Question 1a. Per-token
membership refuses `me bundle` and `me help` — both BIP-39 English words, and
present in **three** of the cross-product's eight subcommand shapes (`bundle`,
`help`, `sysw help`). Whole-token
`Mnemonic::parse_normalized` (which is what `classify_with` uses,
`sysw/mod.rs:217`) misses an unquoted twelve-token phrase entirely. Say which,
and say what happens to the shape the other one catches.

**M-3. The gate claims "ANY argv containing one"; the recogniser is anchored.**
See §Question 1c — three measured leaks through a path token. Either narrow the
gate's wording or state that the recogniser scans for embedded material — and
note that the second choice directly re-opens **I-1**, so the plan must take a
side rather than leave both sentences standing.

**M-4. Row 6's gate has no positive control.** It asserts only *absence* of the
secret from stderr, plus an ordering property that also holds for a
refuse-everything guard. `fn guard(_argv) -> ! { exit(3) }` passes all 24 rows
and the ordering observable. The plan applies exactly this test to its own
remedy gate one page earlier — *"a **tautology** … A gate that cannot fail is not
a gate"* — and to condition 6 — *"with a **positive control**, mutation-checked"*.
Row 6 needs one row asserting a clean argv still succeeds.

**M-5. `FOLLOWUPS.md`'s F-266 entry still carries the counts this fold
retracted**, and §6 condition 7 sends the implementer there. Its heading reads
*"on four of six argv surfaces"* and its mechanism paragraph says
*"`me sysw pack` is the only surface taking positional records"* — the same two
statements the fold corrected in the plan. F-266's **fix** is out of scope for
this round; its **count**, which the plan cites by name, is not.

**M-6 — M-10. Round 8's five carried Minors, none touched by this fold and none
claimed in its commit message.** Verified byte-identical to `ba106a1`:

| this round / round 8 | site | state |
| --- | --- | --- |
| **M-6** condition 9's sentence splice + duplicated recipe | lines **794–796** | carried, sixth round |
| **M-7** *"proves the closure is really 11 and not more"* (the move relocates five + the stub) | line **686** | carried |
| **M-8** nested backticks around `crates/me-cli/src/io.rs`, a module §3's tree never lists | line **563** (row 2) | carried |
| **M-9** *"the digit-pinning work"* resolves to four rows that pin a digit | line **802** | carried |
| **M-10** *"enumerate every type and constant"* attached to the move, where `E0116` cannot occur | lines **514–531** | carried |

---

## NIT

**N-1. Round 8's N-1, carried.** Line **531**: *"every type and constant **the
moving set reference**"* — subject-verb residue of the *"the 11"* → *"the moving
set"* replace. Not claimed.

---

## QUESTION 2 — DISPOSITION OF ROUND 8's 1C/2I/7M/2N, AGAINST THE DIFF

`git show 5cf8ac9` has **two** hunks: `@@ -272,19 +272,34` (§3) and
`@@ -549,7 +564,7` (row 6). **"claimed?"** = named in `5cf8ac9`'s commit message.

| # | claimed? | disposition | the diff line that closes it, or why not |
| --- | --- | --- | --- |
| **C-1** part 1 — the three undeclared flags | yes | **CLOSED** | lines **292–298** retract it by name (*"`me` declares none of those flags — each is `0` occurrences"*) and re-measure on `--in`. The old sentence is gone: `grep -c 'never reaches stderr, because clap names the flag'` → **0** |
| **C-1** part 2 — *"every one is an unexpected POSITIONAL"* | yes | **PARTIAL** | line **300** adds the retraction; lines **311** and **319** keep both retracted statements → **M-1** |
| **C-1** part 3 — enumeration presented as a total | yes | **CLOSED for surfaces, RE-OPENED for shapes** | row 6 is a generated cross-product covering `sysw`, `help`, `sysw help`, `--in X`, `--in=X`, and states the `=`-split. The shape and carrier axes stay hand-written → **C-1 above** |
| **I-1** ordering observable / *"has not moved off clap"* | yes | **first half CLOSED, second half REGRESSED** | `grep -c 'has not moved off clap'` → **0** ✔, and the presence test is replaced by an ordering one ✔ — but that one is not constructible as written (**I-4**), and the override half went from *no observable* to *no step* (**I-3**) |
| **I-2** flag-name layer / wrong predicate | yes | **PARTIAL** | the retracted citation is gone ✔ and the toolkit facts are now right ✔; the layer is still scheduled by nothing, now justified by a false claim (**I-2 above**) |
| **M-1** *"four of six surfaces"* | no | **NOT CLOSED** | line **311** unchanged text (shifted +15) |
| **M-2** condition 9 splice | no | **NOT CLOSED** | lines **794–796** unchanged text (shifted +15) |
| **M-3** *"really 11 and not more"* | no | **NOT CLOSED** | line **686** unchanged text (shifted +15) |
| **M-4** `io.rs` | no | **NOT CLOSED** | line **563** unchanged text (shifted +15) |
| **M-5** *"the digit-pinning work"* | no | **NOT CLOSED** | line **802** unchanged text (shifted +15) |
| **M-6** enumeration at the move | no | **NOT CLOSED** | lines **514–531** unchanged text (shifted +15) |
| **M-7** propagation-gate exit code (informational) | yes | **N/A — correctly recorded** | the header says 2; the script returns 2. Not re-litigated here |
| **N-1** *"the moving set reference"* | no | **NOT CLOSED** | line **531** unchanged text (shifted +15) |
| **N-2** `--out <ms1>` creates a file named with the secret (informational) | no | **N/A** | still true; still outside a stderr-only observable → **M-3** |

**Score: 0 of 1 Critical fully closed (2 of its 3 parts), 0 of 2 Important fully
closed, 0 of 7 Minor, 0 of 2 Nit.**

**No named-and-not-done.** Every claim in `5cf8ac9`'s message is in its diff —
the token scan, the `=`-split, the cross-product, the ordering observable, the
I-2 rewrite. That pattern stayed broken for a second round, and the message again
leads with the author's own error. What did not happen is any Minor at all: the
fold's 43 changed lines are entirely inside §3 and row 6.

---

## QUESTION 3 — CAN AN IMPLEMENTER EXECUTE ALL TWELVE ROWS, AND CAN EACH GATE FAIL?

§4 still carries **12** step rows, counted from the table: `1, 2, 3, 4, 5, 6, 7,
8, 9, 9b, 10, 11`. The diff changed **2** table lines, both in row 6. Rows 1–5
and 7–11 are unchanged from the text round 8 verified row-by-row and are not
re-derived here.

| # | executable? | can its gate fail? | this round's note |
| --- | --- | --- | --- |
| 1 signature change | yes | yes | unchanged |
| 2 the move | yes | yes | one grep clause still names `io.rs`, a module §3 never lists (**M-8**); the row's pty assertion is unaffected |
| 3 mask split | yes | yes | unchanged |
| 4 `observation.rs` + pty | yes | yes | unchanged |
| 5 `remedy.rs` | yes | yes | unchanged |
| **6 the argv guard** | **yes** | **yes for `ms1`, NO for `pass:`, and its ordering half is not constructible as written** | RED today on four shapes a hand list missed — a real improvement. But blind to a whole argv-forbidden class (**C-1**), green on a false refusal (**I-1**), green on a refuse-everything guard (**M-4**), silent on the override (**I-3**), and its ordering clause needs a mechanism the plan does not name (**I-4**) |
| 7 F-265 five-site digit pin | yes | yes | unchanged — still the model row in this plan |
| 8 `--expect` | yes | yes | unchanged |
| 9 `exit.rs` + `channel.rs` | yes | yes | unchanged |
| 9b create the crate | yes | yes | unchanged |
| 10 consume | yes | yes, as regression | unchanged. Note it is the only thing standing between **I-3** and a shipped false refusal |
| 11 publish | n/a | n/a | operator-gated |

**Eleven of twelve rows are executable with a gate that can fail.** Row 6 is the
one the fold rewrote, and it is the one that cannot fail on the class it misses.

---

## WHAT I VERIFIED HERE

Absolute paths throughout. `$?` read immediately, never through a pipe. stdout
and stderr to separate files. Nothing re-derived that the brief listed as
machine-checked.

| check | result |
| --- | --- |
| `Class::is_argv_forbidden` = `is_secret() \|\| is_bearer()` | `{Mnemonic, Codex32Secret, Passphrase}` ∪ `{Mt, Tx}` — **five** classes (`sysw/record.rs:73,89,105`) |
| the plan's recogniser | **four** — `tx:`, `mt1`, `ms1`, BIP-39 wordlist (lines 272, 567) |
| `grep -n 'pass:'` over the plan | **0 hits** |
| `me sysw pack pass:68756e74657232` | **rc 3**, *"is SECRET key material on ARGV"* |
| `pass:` on the other 7 shapes | **7 of 7 echo the hex to stderr** (bare, bundle, sysw, help, sysw wipe, sysw show, sysw pack --in) |
| `me --in /tmp/<ms1>.txt`, `sysw pack --in /tmp/<ms1>.txt`, `sysw show /tmp/<ms1>.txt` | rc 2, **secret in stderr**, all three |
| `me sysw pack --in mt1-2026-08-23-cold-storage-transfer.txt` | **rc 0** — a legitimate invocation the HRP rule refuses |
| `me sysw pack --in ms1-recovery-notes.txt` | **rc 0** — likewise |
| `mt`'s `looks_like_a_transaction` | prefix **+ bech32 charset + `len >= 37`**; its comment records the false refusal that forced the charset test |
| `grep -nic 'bech32\|charset\|alphabet'` over the plan | **0** |
| `me sysw pack --allow-argv-secret --out /tmp/p1.bin <ms1>` | **rc 0**, 143-byte sealed payload written |
| `grep -n 'allow-argv-secret\|override'` over the plan | 4 hits — **175, 307, 308, 788**; none in §4 |
| `--allow-argv-secret` in the test suite | **15** references (`sysw_cli.rs` ×13, `cli.rs` ×2) — the only backstop |
| `Cli::parse()` call site | `crates/me-cli/src/main.rs:305`, inside `fn run()`, binary crate root |
| `[features]` in `crates/me-cli/Cargo.toml` | **absent** |
| test invocation mechanism | `assert_cmd::cargo::cargo_bin("me")` — one binary path per profile |
| `bundle` / `help` in the BIP-39 English wordlist | **both present** (2048 words, `bip39-2.2.2/src/language/english.rs`) |
| `me abandon ×11 about` (12 tokens) | rc 2, clap names only the first word; `me sysw pack` same → rc 4 |
| `me sysw pack -- <ms1>` / `me -- <ms1>` | rc 3 clean / rc 2 **leak** — both seen by a token scan |
| `Show { file: std::path::PathBuf }` | `crates/me-cli/src/main.rs:275`; `me sysw show --help` → `Usage: me sysw show <FILE>` |
| §4 step-row count | **12** (`1 2 3 4 5 6 7 8 9 9b 10 11`) |
| `git show 5cf8ac9` hunks / changed table lines | **2 hunks**, **2** changed table lines, both row 6 |
| every carried Minor/Nit's text vs `ba106a1` | present **verbatim** in both, shifted **+15 lines** by the fold's insertion: *"four of six surfaces"* 296→311, *"the only surface taking a positional"* 304→319, `io.rs` 548→563, *"really 11 and not more"* 671→686, *"the digit-pinning work edits all five"* 787→802, *"the moving set reference"* 516→531, condition 9's splice 779→794 |
| `grep -c 'has not moved off clap'` | **0** — round 8's I-1 sentence removal done |
| `grep -n 'flag-name\|flag_is_secret'` over `design/FOLLOWUPS.md` | **0 hits** — the layer is filed nowhere |
| spec §6d layer ruling | lines **824–841**: layer 1 is *"the primary layer"*, *"it must build the union"*, *"Both layers run pre-parser"* |

---

## WHAT THE FOLD GOT RIGHT

Recorded so round 10 does not re-open it.

- **The token scan is the right design.** Three attempts at this guard, and this
  is the first that removes a hand-written list from the load-bearing axis. The
  surface axis is now generated and it is RED today on four shapes a hand list
  missed. C-1 above is an argument for finishing the same move on the other two
  axes, not for going back.
- **The `=`-split is exactly right and was measured, not assumed.** `--in=<ms1>`
  is one token; an HRP match on the whole token fails; the fold says so and says
  why.
- **The ordering observable is the right KIND of observable.** Round 8 asked for
  something that discriminates pre-parse from error-path, and a presence grep was
  replaced with one that does. I-4 is about how to build it, not about whether it
  is the right thing to want.
- **Named-and-not-done stayed closed for a second round.** All the message's
  claims are in the diff, and it is silent about the Minors it did not touch —
  which is correct behaviour, not a lapse.
- **The commit message again leads with the author's own error, names it as a
  repeat, and does the work.** That is the habit that ended the previous class of
  finding.

---

**VERDICT: NOT GREEN — 1 Critical, 4 Important.** No code may be written against
this plan.

**The one sentence for round 10.** The fold correctly stopped enumerating
*surfaces* and started generating them — and left the **shape** list and the
**carrier** list hand-written, so the guard is short by a class `me` itself calls
SECRET on argv (`pass:`), the gate carries a single `ms1` and cannot see it, and
the same axis-by-axis half-measure recurs three more times: a prefix rule with no
charset test that the sibling repo already shipped and had to fix, an override
the table stopped mentioning while condition 8 still demands it, and an ordering
observable whose mechanism does not exist in this crate. **Derive all three axes
from something already in the source — `is_argv_forbidden()`'s five classes,
`looks_like_a_transaction`'s charset rule, and `main.rs`'s own declaration of
`--allow-argv-secret` — and the guard stops being a list at all.**
