# PROBE — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` §4 step 6, EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe6/mnemonic-engrave`, branch
`probe/p0-step6`, off `8e4532f`. Nothing here is meant to be merged. The
question was **"can `channel.rs` + `exit.rs` be built as the plan specifies, and
do 'codes match §6f' and 'the crate publishes no integer' both hold at once".**

---

## VERDICT

**BUILDABLE WITH DEVIATIONS.**

The hard part **holds cleanly**: `exit.rs` can carry refusal decisions, their
wording and the write-gate ordering with **zero integers**, and `me` reproduces
**every one of its current exit codes byte-for-byte**. No `From<Decision> for
i32` crept back — the mapping moved to the binary as a 12-line
`fn exit_code(&Refusal) -> i32`. §6f and "no shared constant" do **not**
contradict, because §6f is a *per-binary* table that asks `me` to change nothing.

The deviations are in the **gate**, not the design:

- **`-` does not read stdin anywhere in `me` today** — five surfaces, five
  different failures. The premise that it "already works" is false, and building
  it to §6b's literal wording produces **silent record loss at exit 0**.
- **`--out` overwrites** is real and survives separation, but the function that
  implements it (`write_private`) is one §3's own table keeps in `me`, so under
  the plan as tabled `channel.rs` is **9 lines of code** and the clause tests
  something in another crate.
- **"codes match §6f" cannot fail.** All six §6f cells for `me` reproduce today,
  unchanged.
- **§6f's `me` row omits exit 3 entirely** — the policy-refusal code guarding
  seed-and-transaction-on-argv. An implementer conforming to §6f alone would map
  that refusal onto 2.

```
     Summary [  11.799s] 400 tests run: 400 passed, 1 skipped
```
(388 pre-existing, all passing; +12 new unit tests in the two new modules.)

```
 crates/me-cli/src/channel.rs | 161 ++++++++++++++++++++
 crates/me-cli/src/exit.rs    | 335 +++++++++++++++++++++++++++++++++++++++++
 crates/me-cli/src/lib.rs     |   2 +
 crates/me-cli/src/main.rs    | 348 ++++++++++++-------------------------------
 4 files changed, 592 insertions(+), 254 deletions(-)
```

**What was actually built.** `crates/me-cli/src/channel.rs` and
`crates/me-cli/src/exit.rs`, in `me`'s **lib half**. That placement is the
experiment: a lib module **cannot see `main.rs`'s items**, so `EXIT_OK` /
`EXIT_USAGE` / `EXIT_REFUSED` / `EXIT_INVALID` are invisible from both files and
the compiler enforces the crate boundary for real rather than by discipline.
`destination` / `Destination` / `write_private` moved to `channel.rs`;
`WriteBlock` / `write_block` moved to `exit.rs` and gained a `Refusal` decision
type carrying the wording; `main.rs` gained `exit_code`, `refuse` and
`region_bounds`; `read_records` and `no_records_guard` returned
`Result<_, (String, i32)>` and now return `Result<_, Refusal>`;
`stdout_world_readable_mode` was left in `main.rs` because it is **step 2's**
`fd.rs`, not step 6's.

---

## Q1 — `channel.rs`: does `--out` overwrite, and does `-` read stdin?

### `--out` OVERWRITES: yes, and it survives separation trivially

Measured end-to-end, twice over the same path, against a **pre-existing 40-byte
0644 file**:

```
out-overwrite-1                              rc=0 size=61 mode=600 sha=4f27e4b12aa12fc1
out-overwrite-2-different-payload            rc=0 size=79 mode=600 sha=c1056ac39b2053ff
```

Clobber and mode-tightening both hold. `write_private` reaches for nothing in
the binary half — no `Cli`, no `EXIT_*`, no `clap` — so lifting it out is a
copy. A unit test in `channel.rs` pins both halves at once
(`out_overwrites_an_existing_file_and_tightens_its_mode`).

### `-` READS STDIN: **NO. It does not work today, on any `me` surface.**

This is the finding the step's gate exists to produce, and it contradicts the
premise the probe was dispatched with. Measured on the **unmodified** binary:

```
$ printf 'text:6869\n' | me sysw pack --no-passphrase -
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. …
rc=4
```

The `-` is **classified as a record**, found unplaceable, and refused — while the
records the operator piped in are never read. And the other four surfaces each
fail differently:

| invocation | rc | what happens |
| --- | --- | --- |
| `me - --hex` | 2 | `error: unrecognized subcommand '-'` — the converter has **no positional at all** |
| `me sysw pack -` | 4 | `-` classified as an unplaceable record |
| `me sysw show -` | 2 | `me: -: No such file or directory` — treated as a path |
| `me bundle -` | 2 | `error: unexpected argument '-' found` |
| `me hash --unsealed -` | 4 | `non-canonical record: separator '-' at byte 0` |

**Consistent with §6b, which never scopes `-` to `me`.** §6b enumerates the gap
as *"`md`'s four other verbs PLUS `mt`'s `decode`, `verify` and `inspect`"* —
seven verbs, none of them `me`'s — and §7 P1 owns `mt`'s three. So step 6's RED
gate asks the **donor** to grow a channel no spec section assigns to it, at a
step whose only available consumer is the donor.

### Built anyway, for `sysw pack`, and the spec's wording is a trap

`channel::strip_stdin_markers` implements §6b's *"accepted and ignored where
stdin is already the default"* literally: strip every `-`, and if nothing is
left on argv fall through to stdin. It works —

```
- pack-dash-with-stdin                         rc=4
+ pack-dash-with-stdin                         rc=0
+ digest:   185a 0c2d 8441 fbbd 9ba5 1b2d 02f3 28a0
```

— and it is **silently lossy** when `-` accompanies a real record:

```
$ printf 'text:6869\ntext:6a6b\n' | me sysw pack --no-passphrase --out a.bin -
$ me sysw show a.bin        →  pub_len: 19   digest: 185a 0c2d …      (2 records)

$ printf 'text:6869\ntext:6a6b\n' | me sysw pack --no-passphrase --out b.bin - text:6869
$ me sysw show b.bin        →  pub_len:  9   digest: c679 6b68 …      (1 record)
```

Exit **0**, a flashable container, and the piped-in records gone without a word.
Same for `me sysw pack --in recs.txt -`, where `--in` wins and the `-` evaporates
(rc=0). See **C-2**.

The argv gate is unaffected by the strip — `me sysw pack - <ms1>` still refuses
at rc=3, message unchanged.

---

## Q2 — can `exit.rs` hold decisions-and-wording with NO integer?

**YES, and `me`'s codes are unchanged. This is a clean result.**

The machine check, run against the two files that would become the crate:

```
$ grep -nE 'EXIT_|-> *i32|: *i32|as i32|std::process::exit' \
      crates/me-cli/src/exit.rs crates/me-cli/src/channel.rs
crates/me-cli/src/exit.rs:5 :  //! rather than by discipline: `main.rs`'s `EXIT_OK` / `EXIT_USAGE` /
crates/me-cli/src/exit.rs:6 :  //! `EXIT_REFUSED` / `EXIT_INVALID` are invisible from here, …
crates/me-cli/src/exit.rs:14:  //! grep -nE 'EXIT_|…' …   # must be empty
crates/me-cli/src/channel.rs:6: //! `mnemonic-io-lib` would be under: no `EXIT_*`, …
```

**Four hits, all inside `//!` prose. Zero in code.** `cargo build` and
`cargo clippy --all-targets` are both clean with no warnings.

`me`'s mapping, and the whole of it:

```rust
// crates/me-cli/src/main.rs
fn exit_code(r: &Refusal) -> i32 {
    match r {
        Refusal::TerminalDestination { .. } => EXIT_USAGE,
        Refusal::WorldReadableStdout { .. } => EXIT_USAGE,
        Refusal::NoRecords { .. }           => EXIT_USAGE,
        Refusal::ReadFailed { .. }          => EXIT_USAGE,
        Refusal::WriteFailed { .. }         => EXIT_USAGE,
        Refusal::ArgvSecret { .. }          => EXIT_REFUSED,
    }
}
```

**Nothing crept back under another name.** There is no `From<Refusal> for i32`
and no `Refusal::code()`; the crate returns a decision, the binary numbers it,
and the binary also does the printing (`refuse` = `eprintln!("me: {}", …)` +
`exit_code`). §3's "decision from announcement" split executes exactly as
written.

**Proof that the codes are unchanged: a 30-case differential matrix, before vs
after.** Both binaries built from the same tree (`me.before` sha
`5533bab1…`, `me.after` sha `18d7e903…`, confirmed distinct). Every invocation's
exit code **and full stderr** captured, never through a pipe.

```
$ diff -u before.txt after.txt
```

The diff is **three hunks and nothing else**: `Usage: me.before` → `me.after`
(argv[0]), and the two deliberate `-` rows above. Every other case — clap usage,
converter no-mode / empty-input / unreadable-`--in` / unwritable-`--out` /
ms1-refusal / bad-HRP, pack no-records / unplaceable / `tx:`-on-argv /
`ms1`-on-argv / `mt1`-on-argv / `--in`-missing / `--in`-empty / bad-hex / ok,
`sysw show`, `sysw wipe`, `hash`, `bundle`, the 0644 redirect, the 0600 control,
and the **pty terminal refusal** — is byte-identical in code and in message.

> The first run of that matrix returned `rc=127` on every row because the script
> `cd`s to a tmpdir and the binaries were named relatively. It was caught by the
> control rows, not by reading. Absolute paths throughout in the run reported
> here.

`me` also maps its own `Class` onto the crate's `ArgvKind` at the refusal site —
§3's *"split by REPRESENTATION"* executes, and no crate file names a `Class`
variant.

### But three things the plan does not address, which the build surfaced

**(a) The `Refusal` enum's variant set is itself a shared taxonomy, and the plan
does not say whether it is `#[non_exhaustive]`.** Both answers cost something,
and one of them is the shared mapping under another name. Reproduced in a
two-crate scratch project:

```
error[E0004]: non-exhaustive patterns: `&_` not covered
  = note: `Refusal` is marked as non-exhaustive, so a wildcard `_` is necessary
          to match exhaustively
```

- **`#[non_exhaustive]`** → every consumer must write `_ => <some number>`, i.e.
  **a default exit code for a refusal it has never heard of**, chosen blind. That
  default *is* a shared mapping decision, made six times, badly.
- **not `#[non_exhaustive]`** → adding a seventh refusal is a **semver-breaking
  change to all six consumers**, so the crate can never grow a refusal without a
  major bump and six coordinated edits.

**(b) "The crate holds the wording of each refusal" is false for 2 of the 6.**
The moved wording contains the literal string `me sysw pack` **six times**
(`grep -c 'me sysw pack' crates/me-cli/src/exit.rs` → `6`), plus `picotool`,
BOOTSEL, and a SeedHammer II flash region the variant now has to carry as
`region_addr` / `region_end` because the crate cannot know it. A crate shared by
`md`, `mk`, `ms`, `mt` and `mnemonic` cannot hold a refusal whose remedy is
*"run `me sysw pack --region --out payload.bin`"* or whose history-purge recipe
is `sed -i '/me sysw pack/d'`. This sharpens the step-1 probe's M-3: it is not
only the flash region, it is the donor's own command line inside the shared
half.

**(c) "The ordering rule" is one of at least four, and the crate can hold one.**
`write_block` owns *terminal outranks mode*, and that genuinely moves. The other
three are `run_sysw` control flow and no library can hold them:

```
crates/me-cli/src/main.rs:984   read_records(...)        argv gate, first
crates/me-cli/src/main.rs:1011  sysw::admit_check(...)   admission before the ceremony (F-246)
crates/me-cli/src/main.rs:1053  refuse_write_block(...)  write gate before anything describes a container
crates/me-cli/src/main.rs:1102  decide_sealing(...)      the passphrase ceremony
```

---

## Q3 — does "codes match §6f" survive? Do the two rulings contradict?

**No contradiction. §6f is a per-binary table, and it requires `me` to change
nothing.** §6f's single ruling is *"`mk`'s invalid-artifact 2 becomes 1 … the
only code this cycle changes"*, and §7 assigns that to **P3**, not P0. So a
decisions-only crate satisfies §6f trivially: each binary keeps its own numbers,
which is what §6f's table already records.

**Every §6f cell re-verified today, by absolute path, stdin at `/dev/null`:**

| CLI | clap usage | §6f | invalid artifact | §6f |
| --- | --- | --- | --- | --- |
| `md` | 2 | 2 ✓ | `md decode notanartifact` → 1 | 1 ✓ |
| `mk` | 64 | 64 ✓ | `mk decode notanartifact` → 2 | 2 ✓ |
| `ms` | 64 | 64 ✓ | `ms decode notanartifact` → 1 | 1 ✓ |
| `mt` | 2 | 2 ✓ | `printf … \| mt decode` → 1 | 1 ✓ |
| `mnemonic` | 64 | 64 ✓ | `inspect notanartifact` → 2; `inspect md1nonsense` → 1 | "1 or 2 by input shape" ✓ |
| `me` | 2 | 2 ✓ | `me sysw pack notanartifact` → 4 | 4 ✓ |

**`mt`'s cell needs stdin, and that is worth recording** — `mt decode` has no
positional, so `mt decode notanartifact` is clap's **2** and
`mt decode <a valid mt1>` is the §8.2f argv guard's **1**. Neither is the
invalid-artifact code. Fed on stdin it is 1, as §6f says. §6f's table is sound;
reproducing it is not obvious.

**So the clause survives — and it cannot fail.** All six `me`-relevant cells
already match, before any of step 6's work. It is a **regression** gate, not a
RED-first one. See **I-2**.

---

## Q4 — `me`'s exit surface, MEASURED

**`me` produces exactly four codes. `1` is never produced; `101` was not
observed.**

| code | constant | causes measured |
| --- | --- | --- |
| **0** | `EXIT_OK` | every success path; also clap's `--help` / `--version` |
| **2** | `EXIT_USAGE` | **nine distinct causes** — clap usage errors; no output mode selected; empty converter input; unreadable `--in` (converter, `pack`, `bundle`, `sysw show` — four different messages); unwritable `--out`; **no records / R7** (`pack` stdin, `pack --in`, `bundle`); `hash` without exactly one of `--sealed`/`--unsealed`; **world-readable-stdout refusal (F-252)**; **terminal-destination refusal (F-253)** |
| **3** | `EXIT_REFUSED` | **`ms1` over NFC** (converter); **secret or bearer material on argv** (`pack`) |
| **4** | `EXIT_INVALID` | unrecognised HRP (converter); unplaceable record; a reserved prefix with a non-hex body; "not a systemwide container" / header-length mismatch (`sysw show`); *(from source, not run: preview `Render` / `EmptyOutput` failures)* |

`std::process::exit` appears **once** (`main.rs:301`, `std::process::exit(run())`),
and there is **no `unwrap` / `expect` / `panic!` anywhere in `main.rs` outside
its `#[cfg(test)]` module**. 40 single-byte header mutations plus 7 truncations
of a real container through `me sysw show` produced only 0 and 4 — no 101.

**Against §6f's `me` row** — `| me | 2 | 4 = unplaceable record; 2 = terminal refusal | n/a | n/a |`:

- `2` for clap usage — **true**.
- `4` for an unplaceable record — **true**, and `me` is the constellation's only
  binary using 4 for invalid-artifact (`md`/`ms`/`mt` use 1, `mk` 2).
- `2 = terminal refusal` — **true, but it is in the wrong column.** Every other
  row's "invalid artifact" cell answers *what code do you give for an artifact
  that fails to decode*. A destination refusal is not that.
- **`3` is absent from the row entirely.** See **C-1**.
- The row also hides that `me`'s `2` covers **nine** distinct causes. §6f
  carefully records cross-binary collisions (`md encode`'s 2 vs clap's 2; `ms
  repair`'s 4 vs `me sysw pack`'s 4) and records no intra-`me` one.

---

## WHERE THE PLAN IS WRONG

### CRITICAL

**C-1. §6f's `me` row omits `EXIT_REFUSED = 3`, and step 6's gate cites §6f as
the authority.** `3` is `me`'s **policy** refusal — *the record is understood,
well-formed, and this tool will never accept it here* — and it is what
`me sysw pack <ms1>` and `me sysw pack tx:…` return, plus the ms1-over-NFC
refusal. It is the code that separates "you leaked a seed onto argv" from "you
typed the flag wrong".

An implementer wiring `exit_code()` at step 6 has §6f as the stated reference.
§6f lists `2` and `4` for `me`. Mapping `Refusal::ArgvSecret => EXIT_USAGE`
would **conform to §6f**, collapse the policy refusal into the usage code, and
pass the gate. The distinction survives today only because it is in code nobody
is being asked to preserve.

Evidence:
```
$ me sysw pack --no-passphrase <ms1>   → rc=3   "…is SECRET key material on ARGV"
$ me sysw pack --no-passphrase tx:0100 → rc=3   "…is a `tx:` record on ARGV"
$ me --in ms1.txt --hex                → rc=3   "refusing to emit ms1 over NFC…"
```
Fix: add `3` to §6f's `me` row with its meaning, and move "terminal refusal" out
of the invalid-artifact cell.

**C-2. The `-` gate is a genuine RED that no spec section scopes to `me`, and
§6b's own wording makes the compliant implementation silently lossy.** §6b says
`-` is *"accepted and ignored where stdin is already the default"*. For
`me sysw pack`, argv and stdin are **both** record sources, so "ignored" and
"reads stdin" are two different behaviours and the spec picks the one that
discards the operator's pipe:

```
… | me sysw pack --out a.bin -             → 19 bytes, 2 records, digest 185a 0c2d …
… | me sysw pack --out b.bin - text:6869   →  9 bytes, 1 record,  digest c679 6b68 …   rc=0
… | me sysw pack --out c.bin --in f.txt -  → stdin dropped, --in wins,              rc=0
```

Exit 0, a flashable container, no message. On the artifact that gets cut into
metal. §6b needs a sentence ruling what `-` means when another record source is
also present — refusal is the only outcome not worse than telling the operator
nothing — and §7 needs to say whether `me` is in scope for `-` at all, since
§6b's enumeration of the gap does not include it.

### IMPORTANT

**I-1. Step 6's `--out` clause has nothing to test in the files step 6 creates.**
§3's file list says `channel.rs` holds *"`--in` / `--out` / `-`, destination
classification"*, but §3's own M6 table — added so *"a reader can tell where a
function lands"* — assigns `channel.rs` exactly one function, `destination`,
which is **9 lines of code**. The `--out` overwrite lives entirely in
`write_private`, which the same table sends to *"caller-side … stays in `me`"*,
and the `--in`/stdin channel lives in `read_records`, also stays. As tabled,
`channel.rs` is a 9-line file and *"`--out` overwrites"* is a gate on another
crate's code. My build had to pull `write_private` in to have anything to test.

**I-2. "Codes match §6f" cannot fail, so it is a regression gate wearing a RED
gate's clothes.** Measured: all six §6f cells for `me` reproduce on the
untouched binary. §4's own **M5** already made this correction for steps 1 and 7
(*"the column header should not claim they do"*) and lists step 6 among the
"RED-first" steps. One of step 6's three clauses is not. Either state which
clause is RED-first (the `-` one is, genuinely — C-2) or move this clause to the
regression column.

**I-3. `exit.rs` must precede `records.rs`, and the plan orders them the other
way.** §3's table puts `no_records_guard` in `records.rs`, which is **step 5**;
`exit.rs` is **step 6**. `no_records_guard` returns `Result<Vec<String>, (String,
i32)>` and `read_records` returns the same — the step-1 probe measured those two
as **4 of the closure's 8 `EXIT_*` references**. Whichever step first moves a
function carrying an `i32` into the shared half must either publish an integer
there or already have the decision type. **This is C-1 from the step-1 probe, one
file over, and the plan's fix for that one (swap 1 and 2) does not cover it.**
My build removed both `i32`s by introducing `Refusal` first; the plan schedules
`Refusal` three steps later. Order: `exit.rs` → `records.rs`.

**I-4. "The crate holds the wording of each refusal" is false for 2 of 6.** Six
occurrences of `me sysw pack` inside the shared half, plus `picotool`, BOOTSEL
and a flash region carried into the variant as data. Detail in Q2(b). §3 should
say which refusals' wording is shareable and which are the donor's, or
`remedy.rs`'s boundary is decided by whoever implements it.

**I-5. The plan does not say whether the shared refusal enum is
`#[non_exhaustive]`, and both answers cost something.** Detail and the E0004
reproduction in Q2(a). `#[non_exhaustive]` puts a *default exit code for an
unknown refusal* in every consumer — a shared mapping decision made six times,
blind. Not having it makes every new refusal a breaking change for all six. This
is I2's own problem ("a constant is a mapping") arriving through the type system
instead of through a constant, and §3 spends a page on the constant and none on
this.

**I-6. "The ordering rule — which gate outranks which" is one of at least four
orderings, and only one is crate-shaped.** The other three are `run_sysw`
control flow at `main.rs:984 / 1011 / 1053 / 1102` (argv gate → admission →
write gate → sealing ceremony), and F-246 — the ruling that *no line describing a
container may print until every gate that can abort the write has run* — is
entirely caller sequencing. A crate that holds "the ordering rule" holds the
smallest of them.

### MINOR

**M-1. The premise that both step-6 behaviours "already work in `me`" is false
for one of them.** `-` fails on all five `me` surfaces, with four different exit
codes and four different messages (table in Q1). Worth recording because the
plan's step-6 row reads as a refactor gate and one clause is real new work.

**M-2. `me sysw pack --in f.txt -` is undefined by §6b.** Measured: the `-` is
silently ignored, `--in` wins, rc=0. Whatever the ruling, the spec has no
sentence for it.

**M-3. `me`'s exit 2 covers nine distinct causes and §6f's row shows one.**
§6f is careful about collisions *between* binaries and silent about the one
inside the donor. Not blocking — the causes are all "the operator can fix this
and re-run" — but a reader taking §6f's `me` row as `me`'s exit surface will be
wrong about eight of the nine.

**M-4. `cargo fmt` remains red on the untouched baseline** (unchanged from the
step-1 probe; no CI workflow runs it). Running `cargo fmt -- <two files>`
reformats the whole module tree it can reach — it touched `sysw/mod.rs`,
`sysw/mt.rs`, `sysw/tx.rs` and three files under `tests/`, all reverted here.
Anyone formatting during P0 will produce a diff far wider than their change.

---

## WHAT DID NOT GO WRONG

The plan is right about the thing this probe was dispatched to doubt.

- **The crate really can publish no integer.** Zero `EXIT_*`, zero `i32`, zero
  `process::exit` in the code of either new module, enforced by the compiler.
- **`me`'s codes really are unchanged** — 30-case differential, byte-identical
  in code and message apart from the two deliberate `-` rows.
- **`refuse_write_block` returning a decision really is cheap**, as the step-1
  probe measured; it composed with the wording move without further ripple.
- **§3's "split by REPRESENTATION" works.** `me` maps `Class` → `ArgvKind` at
  one site; no crate file names a `Class` variant; the message still picks the
  right private-channel example for the right kind.
- **§6f's table is sound.** All six cells reproduce, across five repositories.
- **`--out` overwrites, and tightens a 0644 target to 0600 doing it.**
- **The pty terminal refusal survived the move** with byte-identical wording,
  which the 388-test suite still cannot see.

## COMMANDS

```
cargo build                                     # exit 0, no warnings
cargo clippy --all-targets                      # exit 0, no warnings
cargo nextest run --locked                      # 400 tests run: 400 passed, 1 skipped
grep -nE 'EXIT_|-> *i32|: *i32|as i32' crates/me-cli/src/{exit,channel}.rs   # 4 hits, all prose
/usr/bin/script -qec "$ME sysw pack --no-passphrase text:6869" /dev/null     # rc 2
diff -u before.txt after.txt                    # 3 hunks: argv[0] + the two `-` rows
```

All binaries — `me`, `md`, `mk`, `ms`, `mt`, `mnemonic` — were invoked by
absolute path. No exit code was read through a pipe.
