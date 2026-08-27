# PROBE — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` §4 step 4 + §6 condition 5, EXECUTED

**Throwaway feasibility probe, 2026-08-26.** Worktree
`/scratch/code/shibboleth/_work/probe4/mnemonic-engrave`, branch
`probe/p0-step4`, off `8e4532f`. Nothing here is meant to be merged. The
question was not "is the remedy text good" but **"does step 4's RED gate hold
against the donor, and can §6 condition 5's POSITIVE test be written at all".**

---

## VERDICT

**BUILDABLE WITH DEVIATIONS.**

Condition 5's positive test **can** be written — it is written, it passes, and it
fails under mutation. But **step 4's RED gate as worded is FALSE against the
correct donor text**, and executing the positive test measured that
**`me`'s zsh recipe purges nothing at the moment `me` prints it** — the same
"reports success, purges nothing" failure the plan cites as `history -d`'s
disqualification.

```
     Summary [  13.507s] 396 tests run: 396 passed, 1 skipped
```

(388 pre-existing + 8 new probe tests. Six of the eight assert the CURRENT,
defective behaviour, so they pin defects rather than endorsing them.)

---

## Q1 — DOES `me`'s CURRENT REMEDY TEXT SATISFY THE RED GATE?

**Half 1: NO — and the gate is what is wrong, not the text.**
**Half 2: it passes, but it cannot fail, so it measures nothing.**

The text, printed by the binary rather than read out of the source:

```
$ /…/target/debug/me sysw pack --no-passphrase --out …/n.bin \
    "abandon abandon … about" < /dev/null ; echo EXIT=$?
EXIT=3
me: record 0, as given (records count from 0), is SECRET key material on ARGV. Refused; nothing was read and nothing was written.
      It can spend everything derived from it, forever -- and argv is public: /proc, `ps` and your shell history all keep a copy.
      Use a private channel instead:
          ms encode --phrase - < seed.txt | me sysw pack --out p.bin
          me sysw pack --in records.txt --out p.bin

      TO PURGE WHAT ALREADY LEAKED -- match on the COMMAND, never on the secret, or you type it into history a second time:
          bash/zsh:  sed -i '/me sysw pack/d' "$HISTFILE"
          fish:      history delete --prefix 'me sysw pack'
          and `shred -u` any file you pasted it from.
      (On zsh, `history -d` does NOT delete -- -d prints timestamps. It would report success and purge nothing.)
      If argv is safe where you are -- a single-user air-gapped box, an amnesic Tails session -- --allow-argv-secret proceeds.
```

### C-1 (CRITICAL) — the gate's zsh half is false against the correct text, and satisfying it would RE-CREATE the defect the plan is guarding against

> step 4 | `remedy.rs` | zsh remedy does **not** contain `history -d` …

The shipped text **does** contain the literal string `history -d` — in the
sentence that warns it does not delete. A gate written as
`assert!(!zsh_remedy.contains("history -d"))` is RED against the *correct* text,
and the only way to turn it green is to delete the warning — leaving operators
to reach for `history -d` unwarned, which is precisely §3's stated reason for
rejecting `mt`'s text as a source.

`me` already carries this trap as a comment, and step 4 was written without
reading it (`crates/me-cli/tests/sysw_cli.rs:2080`):

```rust
    // NOT `!err.contains("history -d")` -- the message deliberately NAMES that
    // command in order to warn against it, so the naive negative fails on the
    // warning itself. The requirement is that it is never OFFERED, which is
    // what the explicit disclaimer proves.
```

Pinned by `the_shipped_zsh_remedy_DOES_contain_the_string_history_dash_d`.
**Fix the gate, not the text:** the testable property is *never OFFERED* — e.g.
`the zsh recipe LINE is a sed invocation` plus `the text contains an explicit
"does NOT delete" disclaimer`.

### I-1 (IMPORTANT) — the gate's fish half cannot fail

`me`'s remedy is a **static** string. The `format!` at `main.rs:2005` interpolates
`{i}`, `{what}`, `{why}` and `{example}` — a record *index*, a class *name*, a
reason, and a fixed example. The secret is never in scope. So *"fish remedy does
not contain the secret"* is true of the empty string, of `mt`'s text, and of any
string whatsoever that a `remedy.rs` might return. It is not a RED gate; it is a
tautology. (It would be meaningful only if `remedy()` took the leaked record as
an argument — which is exactly the signature §3 does not describe.)

### I-2 (IMPORTANT) — step 4 has no test that can fail first, and M5 does not cover it

The plan's M5 concedes that steps 1 and 7 have no failing-first test and that the
column header should not claim one. Step 4 is the same shape — a **move** of text
that already exists and already has the intended properties — but is not
exempted. Taken literally the gate goes RED (C-1); taken as intended it is green
before a line is written.

### I-3 (IMPORTANT) — "the zsh remedy" and "the fish remedy" are not separable values in the donor, and one call site gets no remedy at all

`me` emits **one** block naming both shells; unlike `mt` there is no `$SHELL`
dispatch, so `remedy.rs` must invent the per-shell API the gate presumes.
Worse, the recipe is **command-specific** (`/me sysw pack/`), and `me` has a
**second** argv-leak site with no recipe at all — `run_seal_cli`
(`crates/me-cli/src/main.rs:586-592`), which *warns and proceeds*:

```
me: WARNING -- seed material was passed on the COMMAND LINE.
    /proc/<pid>/cmdline is world-readable without hidepid, `ps` shows it, and your
    shell has already written it to history. Treat this seed as EXPOSED.
    For real seed material use --in <file> or stdin instead.
```

The site that **lets the leak through** is the one with no purge advice. A
`remedy.rs` extracted as *"the purge/remedy text"* freezes that asymmetry unless
it takes the invocation as a parameter and both sites call it.

---

## Q2 — CAN THE POSITIVE TEST BE WRITTEN? **YES** — and here it is

`crates/me-cli/tests/probe_history_remedy.rs`, 8 tests, all passing:

```
        PASS [   0.005s] (1/8) the_shipped_zsh_remedy_DOES_contain_the_string_history_dash_d
        PASS [   0.276s] (2/8) zsh_recipe_purges_NOTHING_when_run_in_the_SAME_session
        PASS [   0.548s] (3/8) zsh_recipe_purges_the_entry_when_run_in_a_LATER_session
        PASS [   0.548s] (4/8) zsh_recipe_is_UNDONE_at_exit_under_no_append_history
        PASS [   0.549s] (5/8) control_without_the_recipe_the_entry_survives_both_sessions
        PASS [   2.751s] (6/8) fish_recipe_purges_only_when_the_operator_answers_all
        PASS [   2.756s] (7/8) fish_prefix_does_not_match_a_path_qualified_invocation
        PASS [   2.756s] (8/8) fish_recipe_is_interactive_and_prints_the_secret
     Summary [   2.757s] 8 tests run: 8 passed, 0 skipped
```

The shape: `script -qec "<shell> -i" /dev/null` gives a real pty; session A runs
the **real `me` binary** with the seed on argv (refused, exit 3) so the leak is
genuine; session B runs the recipe; the assertion reads the history **file** with
an outside tool. Two guards make it mean something:

* **A control** — same two sessions with the recipe replaced by `true` — asserts
  the entry *survives*, so the driver is not what removes it.
* **Mutation** — the recipe's pattern replaced by `/zzz_no_such_pattern_zzz/d`:

```
        FAIL [   0.555s] (1/1) zsh_recipe_purges_the_entry_when_run_in_a_LATER_session
     Summary [   0.557s] 1 test run: 0 passed, 1 failed, 396 skipped
```

### FIVE TRAPS, EACH OF WHICH SILENTLY PRODUCES A FALSE PASS

Naming them is most of what this probe is worth — every one reports success.

1. **`zsh -f` disables the write under test.** `-f` is `NO_RCS`, and zsh saves
   the history file at exit only when `RCS` is set. With `-f` the file is never
   written, so *"the entry is gone"* is true because nothing was ever there.
   Measured: `-f` → no `$HISTFILE`; same session without `-f` → 53 bytes.
   Hermeticity must come from `ZDOTDIR` + `--no-globalrcs` instead.
2. **`fish --no-config` disables history persistence** (4.8.1). The obvious
   isolation flag leaves the history file empty or absent — again, vacuously
   green. Isolate with `XDG_*` + `HOME` instead.
3. **`XDG_DATA_HOME` alone does not isolate fish.** With the real `HOME`, a
   fresh session arrives holding the operator's real history and copies it into
   the fixture: measured **422 lines**, timestamps rewritten to now, from a
   session whose only command was `exit`. With `HOME` also redirected: **2
   lines**. A fish test that skips this both tests the wrong history and spills
   the operator's real one into `/tmp`.
4. **Two sessions in the same wall-clock second**: the second ignores the
   first's entries, the recipe finds nothing, and a naive test flakes. A ~1 s
   gap fixes it.
5. **`TERM=xterm` under a pty costs 10 s per fish session** waiting for a
   Primary-DA reply that never arrives (`TERM=dumb` → 0.27 s).

### MINOR — "run the emitted recipe" is not literally what the test does

The recipe is printed with a label and padding (`\x20   bash/zsh:  sed -i …`), so
a test that truly *executes what was emitted* must strip `bash/zsh:` off the
line first. These tests hard-code the recipe string. Closing that gap is a
`split_once("bash/zsh:")` — worth doing, because it is what makes the test bind
to the text rather than to a copy of it.

---

## Q3 — IS ANYTHING UNANSWERABLE? **No.** Condition 5 stands as worded

Both shells are measurable, including fish's interactive path (`all` supplied as
the answer). Condition 5 needs no escape hatch and should keep none.

---

## Q4 — IN-MEMORY vs ON-DISK: **it decides the answer**, and `me`'s recipe loses

### C-2 (CRITICAL, against `me` — reported, not fixed) — the recipe purges NOTHING at the moment it is printed

Under **stock zsh 5.9.2 defaults** the leaked command is held in memory and
written to `$HISTFILE` at exit. An operator who is refused and immediately runs
the recipe `me` just printed edits a file that does not yet contain the entry —
`sed` exits 0, prints nothing — and then zsh writes the secret to disk on the way
out. End-to-end with the real binary:

```
--- did me refuse? ---            1
--- HISTFILE after the session ---
/…/target/debug/me sysw pack --no-passphrase --out /…/n.bin "abandon abandon … about"
sed -i '/me sysw pack/d' "$HISTFILE"
exit
--- seed still in history? ---    1
```

This is the same class as `history -d`: **reports success, purges nothing.** The
operator's own `~/.zshrc` (98 bytes) sets no history options, so this machine is
in the affected configuration.

The distinction is measurable and complete:

| zsh configuration | same-session recipe | later-session recipe |
| --- | --- | --- |
| **stock defaults** (this box) | **FAILS — secret on disk at exit** | works |
| `setopt incappendhistory` | works (file) | works |
| `setopt sharehistory` | works (file) | works |
| `unsetopt appendhistory` | fails | **UNDONE at exit — secret resurrected** |

### I-4 (IMPORTANT) — a purge that works on disk still leaves the secret in memory

Even where the file ends up clean, `fc -l` in the same session still lists the
entry, so `Ctrl-R` and `↑` still surface it, and any later `fc -W` writes it back:

```
MEM:
: me sysw pack SECRETSEEDXYZ
sed -i "/me sysw pack/d" "$HISTFILE"
ENDMEM
```

Bash behaves the same (`history` after the `sed` still shows entry 1). **The
recipe never touches the in-memory copy in any shell.** What would work — and is
one clause longer — is naming the memory step too: on zsh/bash the operator must
either run the recipe from a **new** shell, or drop the in-memory copy in the old
one before it is written back.

### I-5 (IMPORTANT) — the fish recipe cannot run as printed

Stock fish 4.8.1, cross-session, unattended:

```
[1] me sysw pack --no-passphrase --out /…/n.bin "abandon abandon … about"
Enter nothing to cancel the delete, or
Enter one or more of the entry IDs or ranges like '5..12', separated by a space.
Enter 'all' to delete all the matching entries.
Delete which entries?
```

Three separate problems, all measured:

* **It is interactive.** Unattended it deletes nothing; the operator's *next*
  typed line is eaten as the answer (`Ignoring invalid history entry ID "echo"`).
  Answering `all` does purge — the message never says so.
* **It re-displays the secret**, in full, on the terminal it is trying to clean.
* **`--prefix` is anchored**, `sed`'s pattern is not: an invocation typed as
  `/path/to/me sysw pack …`, `sudo me …` or `VAR=x me …` is not matched at all,
  and the recipe still exits 0 having found nothing.

### I-6 (IMPORTANT) — on this operator's own machine the fish recipe is a silent no-op that dumps the whole history

`/home/bcg/.config/fish/config.fish:105` shadows the builtin and **drops every
argument**:

```fish
function history
    builtin history --show-time='%F %T '
end
```

`history delete --prefix 'me sysw pack'` therefore becomes
`builtin history --show-time=…` — it prints the entire history, deletes nothing,
and exits 0. Measured on the real config; the leak survived both in memory and on
disk. That a personal function can silently disarm the recipe is an argument for
the advice being `builtin history …`, and a second instance of the failure mode
`remedy.rs` exists to prevent.

---

## WHAT THIS MEANS FOR THE PLAN

* **C-1** blocks step 4 as written. Rewrite the gate to assert *never offered* +
  *disclaimer present*, and record why the naive negative is wrong.
* **I-1/I-2** mean step 4 currently has no falsifiable RED gate at all. Either
  give it one (the positive test IS one — it fails under mutation) or exempt it
  the way M5 exempts steps 1 and 7.
* **I-3** means `remedy.rs`'s signature must take the invocation, and §3 should
  name both call sites.
* **C-2 and I-4/I-5/I-6 are `me` defects, not plan defects** — but condition 5
  is what surfaces them, which is the strongest possible argument for keeping it.
  A condition 5 that only asserted *a command was printed* would have shipped the
  crate with a remedy that does not work at the moment it is offered.

---

## APPENDIX — the run

```
$ cargo nextest run --locked --no-fail-fast
     Summary [  13.507s] 396 tests run: 396 passed, 1 skipped
```

```
$ git show --stat
 crates/me-cli/tests/probe_history_remedy.rs | 354 ++++++++++++++++++++++++++++
 design/agent-reports/PROBE-P0-step4.md      | 311 ++++++++++++++++++++++++
 2 files changed, 665 insertions(+)
```
