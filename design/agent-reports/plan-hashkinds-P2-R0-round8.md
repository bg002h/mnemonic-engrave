# R0 round 8 — fold verification: the response to round 7

**Scope: two questions only.** Did the fold close each round-7 finding, and did
the fold introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and
everything closed in R0 rounds 1–6 and the journey walk were not re-reviewed.
Round 7's settled facts were taken as given and not re-derived: the four opcode
names against rust-miniscript and rust-bitcoin, the `hash256` hazard chain on
`me` 0.9.0, the `md compose` caveat's conditional phrasing surviving phase 1,
card volume, and every gate number.

**Artifact.** BRANCH `hashkinds-p2` @ `2ff6211` ("fold: R0 round 7 -- the
card/notice boundary, and a test that could not fail"), single commit, in
`/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`. Parent `0f70a8c`.
Diff: `crates/ms-cli/src/cmd/hashlock.rs` (+65/−?), `crates/ms-cli/tests/hashlock_kind.rs`.

**Toolchain.** `export PATH=/home/bcg/.cargo/bin:$PATH`, every measurement taken
**from inside the worktree**, where `cargo --version` → `cargo 1.85.0
(d73d2caf9 2024-12-31)`, matching `rust-toolchain.toml`. (From the
`mnemonic-engrave` cwd the same shim reports `1.97.0-nightly` — the pin is
cwd-resolved.) The system 1.98.0 phantom-clippy trap was avoided.

**Tree state.** `git status --porcelain` empty before and after. Every mutation
below was applied by script, reverted, and re-hashed:
`crates/ms-cli/src/cmd/hashlock.rs` = `ed474846…9329`,
`crates/ms-cli/tests/hashlock_kind.rs` = `06d8443b…f21e`,
`crates/ms-cli/tests/hashlock_outputs.rs` = `7d33a07c…b412` — byte-identical to
`2ff6211`. `/scratch/code/shibboleth/ms-worktrees/p2-shaped` was not touched.

---

## 0. The green claims — independently re-measured

One capture per suite, greped afterwards.

| gate | claimed | measured | verdict |
| --- | --- | --- | --- |
| `cargo test --workspace --locked` | 101 suites / 0 failures | exit 0; `grep -c '^test result:'` → **101**; result lines not ` 0 failed` → **0**; `FAILED` → **0** | **TRUE** |
| `cargo nextest run --locked --all-targets` | 583 tests | `583 tests run: 583 passed, 11 skipped`, exit 0 | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0; output has no line that is not `Checking`/`Compiling`/`Finished` | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, same | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, **0 bytes** of output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |

**stdout and exit codes are untouched by the fold.** `git show 2ff6211 --
crates/ms-cli/src/cmd/hashlock.rs | grep '^[+-]' | grep "stdout\|Ok(\|return "`
returns exactly **one** line, and it is prose inside the new caveat string
(`` `me sysw pack` for the record on stdout. If either answers ``). Every
matrix cell below exits 0 with `stdout_lines=1`.

---

## 1. Findings CLOSED

### Round-7 **I-1** (the `hash256` notice nested inside `if !args.no_engraving_card`) — **CLOSED for `--no-engraving-card`; a new cell is broken — see I-1 below**

`hashlock.rs:551-568`, now at function level, below the new boundary comment at
`:531`. Observed on the binary built from `2ff6211`:

```
$ printf 'correct horse battery staple' | ./target/debug/ms hashlock \
      --hashlock-phrase-stdin --kind hash256 --no-engraving-card
hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
WARNING: this record's `hash256:` tag is the ONLY thing distinguishing it from a sha256 record …
```

stderr = 1 line, 365 characters, where round 7 measured **zero bytes**. The test
now iterates both argv shapes (`hashlock_kind.rs:337-347`), and the reviewer's
own mutation reds it — see mutation **B** below.

**Is the boundary REAL or just a comment?** Partly. I classified every
`writeln!(stderr, …)` in `run()`:

| line | content | above/below | suppressible by `--no-engraving-card`? |
| --- | --- | --- | --- |
| 456 | `THIS CARD CARRIES THE PREIMAGE …` | above | yes — card header, correct |
| 461 | `digest:` | above | yes — card |
| 466 | `for md compose: --path … <kind>=<h>` | above | yes — card |
| 479 | the `md compose` / `me sysw pack` caveat | above | yes — **half of it is about stdout; see M-3** |
| 490,491 | `preimage (ms1)` / `(hex)` | above | yes — card, carries the secret |
| 492 | `method:` | above | yes — card |
| 494 | `record (phrase):` | above | yes — card |
| 495 | `THIS RECORD CARRIES THE PHRASE … treat the file … like the phrase itself` | above | yes — **a notice by the fold's rule; not in F-536** |
| 503 | the write-down instruction | above | yes — card |
| 510 | `OP_SIZE 32 before <opcode>` | above | yes — card self-check line |
| 511 | `One phrase per policy … Never use this phrase as a passphrase …` | above | yes — **a notice by the rule; not in F-536** |
| 514 | `WARNING:` brainwallet | above | yes — F-536 |
| 518 | `WARNING:` short phrase | above | yes — F-536 |
| 524 | `WARNING:` `--hex` publishes forever | above | yes — F-536 |
| 527 | `The file you just wrote is the only copy until you cut the plate.` | above | yes — **a DATA-LOSS notice by the rule; not in F-536** |
| 529 | `source:` | above | yes — card |
| **559** | the `hash256` tag-strip WARNING | **below** | **no — unguarded (this is I-1)** |
| **594+** | the §13.4 four-digest listing | **below** | no, but stands down under `--json && --no-engraving-card` |

Everything above the line does belong to the card in the sense that the spec
(`SPEC_ms_hashlock.md:467-481`) lists it as a card component. What the table
shows is that the fold's **new** rule — "below is a NOTICE about a hazard in
what was just emitted" — reclassifies more lines than F-536 names (**I-3**), and
that the two lines now below the boundary obey two different suppression rules
while the comment states one (**I-1**).

### Round-7 **I-2** (`the_write_down_line_names_both_axes` had a false-PASS path) — **CLOSED**

`hashlock_kind.rs:381` now asserts `line.contains("write the method line AND the
hash line")` — the instruction, not the bare noun — and `:387` asserts
`line.contains("(ripemd160)")`. Round 7's exact mutation M4b now reds it
(mutation **A** below); it was green before.

**Is the new assertion over-coupled to wording?** Examined and declined as a
finding. A rephrasing to `"write the method line and the hash line"` (lowercase
`and`) or `"write BOTH the method line and the hash line"` would red it with no
defect — but the failure message names precisely what must be preserved (*"the
write-down INSTRUCTION must name both axes; a later clause mentioning either
word is not the instruction"*), and the realistic regression — deletion — is now
caught. The second assertion is also strictly stronger than what it replaced:
`(ripemd160)` with parentheses appears only in the instruction, while the old
`contains("ripemd160")` could have been satisfied anywhere on the line.

**Is there still an edit that leaves it green?** Yes, one, and I judge it not a
finding: **mutation E**, negating the instruction to `do NOT write the method
line AND the hash line (ripemd160)`, stays at 11 passed. That is the residual
weakness of every `contains` assertion and is not a plausible regression. Two
other green edits are recorded as **M-2** below.

### Round-7 **M-1** (the caveat named only `md compose`) — **CLOSED, and both claims are TRUE**

`hashlock.rs:479-487`. I ran **both doors for all three kinds** against the real
binaries (`md 0.14.0`, `me 0.9.0`) rather than accepting the prose:

```
$ md compose --wrapper wsh --path 2of3,hash256=98a2…d488
md: path `2of3,hash256=98a2…d488`: unknown option `hash256`        [exit 1]
$ md compose --wrapper wsh --path 2of3,ripemd160=09e7…946b
md: path `2of3,ripemd160=09e7…946b`: unknown option `ripemd160`    [exit 1]
$ md compose --wrapper wsh --path 2of3,hash160=b5b7…d0bd
md: path `2of3,hash160=b5b7…d0bd`: unknown option `hash160`        [exit 1]
$ md compose --wrapper wsh --path 2of3,sha256=3cf5…4c12
wsh(and_v(v:multi(2,…),sha256(3cf5…4c12)))                          [exit 0]
```

```
$ printf 'hash:<kind>:<digest>\n' | me sysw pack --no-passphrase --out x.bin
me: record 0 … record 0: hash: must be exactly 64 hex characters    [exit 4]
```
— identical for `hash256`, `ripemd160` and `hash160`. So *"requires `<kind>=`
support in `md compose`, and `<kind>` support in `me sysw pack` for the record on
stdout"* is true on both doors for all three kinds, and *"asks for \"exactly 64
hex characters\""* is a literal substring of `me`'s real output. The `sha256`
negative arm holds: no caveat, and `md compose` succeeds.

**Readability and volume.** The caveat stays **one line** (385 characters at its
longest, vs 383 for the pre-existing OP_SIZE line, so it is not an outlier), and
card volume is byte-for-byte what round 7 measured:

| invocation | stderr lines |
| --- | --- |
| `--kind sha256` | 10 |
| `--kind ripemd160` / `--kind hash160` | 11 |
| `--kind hash256` | 12 |
| `--kind hash256 --method sha256` | 13 |
| `--kind hash256 --json` | 13 (12 + advisory) |
| **no `--kind`** | **15 — the maximum, unchanged** |
| `--kind hash256 --no-engraving-card` | 1 |
| `--json --no-engraving-card` | 1 |
| `--kind hash256 --json --no-engraving-card` | **2 — see I-1** |

Nothing scrolls off. The moved WARNING now trails `source:` as the card's last
line, which reads correctly and is not orphaned.

### Mutation proof of the fold's three claims — all three hold

Applied by script, one at a time, reverted and re-hashed between runs; baseline
is 11 + 11 passed across `--test hashlock_kind --test hashlock_outputs`.

| # | mutation | result | test reddened |
| --- | --- | --- | --- |
| **A** | delete `write the method line AND ` from the instruction (round 7's M4b) | 10 passed, 1 failed | `the_write_down_line_names_both_axes` |
| **B** | re-nest: `if kind == HashKind::Hash256 && !args.no_engraving_card` | 10 passed, 1 failed | `hash256_warns_against_stripping_its_own_tag` |
| **C** | drop the `` `me sysw pack` `` half of the caveat | 10 passed, 1 failed | `a_non_sha256_kind_warns_that_md_may_not_accept_the_operand` |
| **D** | `d.phrase_chars` → `d.phrase_chars.or(Some(0))` — print the write-down line on `--hex`/`--random` | **11 + 11 passed, 0 failed** | **NONE — see I-2** |
| E | negate the instruction (`do NOT write the method line AND …`) | 11 + 11 passed | NONE — declined above |
| F | delete the hash-axis recovery clause | 11 + 11 passed | NONE — see M-2 |

The fold's commit message says *"Both fixes mutation-proven with the reviewer's
exact mutations"* — **A** and **B** are exactly those mutations and both red.
That claim is TRUE.

---

## 2. New defects

### I-1 — BRANCH — **Important** — the notice is unguarded, so `--json --no-engraving-card --kind hash256` breaks the stderr pin; the pin test never passes `--kind`, and a comment 30 lines below still asserts the pin holds

**Location.** `crates/ms-cli/src/cmd/hashlock.rs:551` (`if kind ==
HashKind::Hash256 {` — no flag guard at all), against `:583-584` and
`crates/ms-cli/tests/hashlock_outputs.rs:188-193`.

Round 7's fix shape was *"under the same condition the §13.4 listing already
uses (`!(args.json && args.no_engraving_card)`)"*. The fold used **no**
condition. Prescribed fixes are not authoritative, so the deviation is not the
defect — its consequence is.

**Constructed failure.**

```
$ printf 'correct horse battery staple' | ./target/debug/ms hashlock \
      --hashlock-phrase-stdin --kind hash256 --json --no-engraving-card
--- STDOUT --- {"digest":"98a2…","hash_record":"hash:hash256:98a2…","kind":"hash256",…}
--- STDERR (485 bytes, 2 lines) ---
WARNING: this record's `hash256:` tag is the ONLY thing distinguishing it from a sha256 record …
warning: stdout carries private key material (can spend) — redirect or encrypt …
```

Against the default kind, the same pair gives **1 line, the advisory alone**
(116 bytes) — so the pin holds everywhere the test looks and nowhere else.

**Three things are now simultaneously true and mutually inconsistent.**

1. `hashlock_outputs.rs:192` asserts, as an **equality** on the whole stream:
   `"under --json --no-engraving-card, stderr must be exactly the
   PrivateKeyMaterial advisory (spec §4.4, §11)"`. Unconditional wording.
2. `hashlock.rs:583-584` still states in a comment: *"that pair, and only that
   pair, is what `hashlock_outputs.rs` pins to exactly the PrivateKeyMaterial
   advisory (§4.4, §11). There the notice **changes channel instead**, into
   `kind_specified` and `digests_by_kind` on the object."* That sentence is now
   false about the code sitting 30 lines above it.
3. The code emits 485 bytes in that cell.

**Why the test cannot see it.** `json_both_variants`
(`hashlock_outputs.rs:167-205`) has two invocations — `--hashlock-phrase-stdin`
and `--hex -` — and **neither passes `--kind`**, so `kind` is the `Sha256`
default and `:551` never fires. The `--kind hash256` cell of the flag matrix has
no coverage at all under `--json --no-engraving-card`. Adding `"--kind",
"hash256"` to either invocation turns the equality assertion red immediately —
that is the whole reproduction.

**Why Important.** The fold's stated thesis is that it fixed the *shape* of a
thrice-recurring "notice on the wrong side of a guard" class. It fixed the
`--no-engraving-card` axis and introduced a new inconsistency on the `--json`
axis in the same edit — and the one artifact in the tree that would have caught
it is blind to the only kind that triggers it. Round 7 recorded *"the branch
honours it"* about this pin; that is no longer true.

**Two acceptable remedies, and the tree must pick one.**
(a) Stand the notice down under `json && no_engraving_card`, matching §13.4 —
defensible because the JSON object already states `"kind":"hash256"`
unambiguously and a script that pipes `jq -r .hash_record | me sysw pack` gets
exit 4 and stops rather than hand-deleting a tag; this restores the pin and
makes `:583` true again. (b) Keep it unguarded, narrow the claim at `:583` and
the assertion message at `:192` to say the pin holds *except for hazard
notices*, and add the `--kind hash256` row to `json_both_variants` so the
narrowed contract has a gate. What is not acceptable is the present state, where
the code and two comments disagree and nothing can fail.

**Verified by:** the command above; the matrix table in §1; `grep -n
"no_engraving_card" crates/ms-cli/src/cmd/hashlock.rs` → `:81, :454, :570,
:582`; `sed -n '167,205p' crates/ms-cli/tests/hashlock_outputs.rs`.

---

### I-2 — BRANCH — **Important** — `hex_source_gets_the_unconditional_warning_and_no_write_it_down_line` asserts the absence of a string that exists nowhere in the crate; it is a gate that cannot fail, and it guards a spec guarantee

**Location.** `crates/ms-cli/tests/hashlock_outputs.rs:133`.

```rust
    assert!(
        !se.contains("write the method line next to your phrase"),
        "no phrase, no instruction:\n{se}"
    );
```

```
$ grep -rn "write the method line next to your phrase" crates/
crates/ms-cli/tests/hashlock_outputs.rs:133:        !se.contains("write the method line next to your phrase"),
```

The only occurrence in the crate is the assertion itself. `git log -S` shows
why: commit **`3056360`** (the journey-walk fold, the commit round 7 reviewed)
rewrote the source string from `write the method line next to your phrase` to
`write the method line AND the hash line ({}) next to your phrase`, and never
updated the negative assertion aimed at it.

**Constructed failure (mutation D).** Make the write-down line print on the
sources that have no phrase:

```
-  if let Some(n) = d.phrase_chars {
+  if let Some(n) = d.phrase_chars.or(Some(0)) {
```

```
$ ./target/debug/ms hashlock --hex - <<< c3e97525…2016
…
phrase:          0 characters -- write the method line AND the hash line (sha256) next to
your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both; …

$ cargo test -p ms-cli --test hashlock_kind --test hashlock_outputs
test result: ok. 11 passed; 0 failed …
test result: ok. 11 passed; 0 failed …
```

The `--hex` card now instructs an operator with **no phrase** to write a method
line beside it, announces `0 characters`, and the suite is green — including the
test whose own name is `…_and_no_write_it_down_line`.

**The guarantee it was protecting is a spec guarantee.**
`SPEC_ms_hashlock.md:471-474`: *"for `--hex`, `--random` and `<ms1>` the line
reads `preimage supplied` and carries no write-it-down instruction, there being
no phrase to write it beside (correctness N-2)"*. The behaviour is correct
today; only its gate is dead. Per this project's severity rule — *"defects in
what a tool claims to have done (a gate that cannot fail … a test that reports a
false PASS)"* — that is blocking.

**Scope note, stated plainly.** This assertion was killed by `3056360`, not by
`2ff6211`. I raise it under this round anyway, and count it, for three reasons:
it is round-7 I-2's **exact class** (a write-down-line assertion that cannot
fail); it is one `grep` from the line the fold's I-2 fix edited; and the fold's
commit message frames its I-2 response as recognising *"a literal false PASS in
the test I wrote"* — the recognition did not extend to the sibling assertion on
the same instruction in the same crate. The fold fixed I-1 as a shape and I-2 as
a line.

**Fix shape.** Assert on what the line's presence actually looks like today —
`!se.contains("write the method line")` alone is enough and matches both the old
and new wording — then re-run mutation D to confirm it reds.

**Verified by:** the `grep -rn` above; `git log -S"write the method line next to
your phrase" --oneline -- crates/ms-cli/src/cmd/hashlock.rs` → `3056360`,
`3ec623e`; mutation D applied, built, output pasted, reverted, `sha256sum` back
to `ed474846…9329`.

---

### I-3 — REPO — **Important** — F-536 enumerates the class by grepping `WARNING:`, not by the rule the fold just wrote, and then declares the enumeration complete

**Location.** `design/FOLLOWUPS.md:17445-17478` (F-536), and the code comment it
mirrors at `crates/ms-cli/src/cmd/hashlock.rs:546-549`.

F-536's tone is honest — it volunteers the sharpest instance, states the
behaviour change is real, and gives a reason for deferring that matches the
cycle's own divergence rule. **The deferral is not softening.** Its *scope* is
the defect. F-536 closes with:

> These three are the remaining instances of the class, already located.

That sentence is false. The fold's own rule is *"a NOTICE about a hazard in what
was just emitted"*. Applying the rule rather than the `WARNING:` prefix, at
least three further above-the-line lines qualify — none named by F-536 and none
named by round 7:

1. **`hashlock.rs:527`** — *"No phrase exists, so nothing can be guessed … The
   file you just wrote is the only copy until you cut the plate."* A **data-loss**
   notice, and it disappears on the path the spec itself documents.
   `SPEC_ms_hashlock.md:340` advertises the one-liner `ms hashlock --random
   --json --no-engraving-card | jq -r …`. Run:
   ```
   $ ./target/debug/ms hashlock --random --out $T/p.ms1 --json --no-engraving-card
   [stderr] warning: stdout carries private key material (can spend) — redirect or encrypt …
   ```
   One line. With the card on, the same invocation prints *"The file you just
   wrote is the only copy"* at stderr line 9 (verified). An operator following
   the spec's own scripted recipe is never told the file is the only copy.
2. **`hashlock.rs:495`** — *"THIS RECORD CARRIES THE PHRASE … treat the file you
   put it in like the phrase itself."* Under `--emit-record --json
   --no-engraving-card` the phrase reaches **stdout** inside `phrase_record`
   (verified: the key is present in the object) with that handling notice
   suppressed. Secret-handling, therefore **non-blocking on its own** by the
   2026-08-27 ruling — but it is an instance of the class F-536 says is fully
   enumerated.
3. **`hashlock.rs:511`** — *"Never use this phrase as a passphrase or a password
   anywhere else — a spend publishes the preimage, and anyone can then test
   guesses at the phrase itself."* A hazard about what stdout just published.

**The concrete bad outcome.** A later sweep greps `F-536`, moves three
`WARNING:` lines below the boundary, closes it, and the code comment at `:546`
plus F-536's closing sentence then jointly certify that the thrice-recurring
class is handled — while a data-loss notice stays suppressed on a
spec-documented scripted path, with nothing left to re-find it. Before the fold
there was no claim at all; now there is a false all-clear, which is worse than
saying nothing. This repo has already recorded the class (*"the defect class
moves into the remedy — commit the enumeration"*); a shape fix whose enumeration
under-counts by half re-creates the bug it set out to kill.

**Fix shape.** Either widen F-536's list and `:546-549` to the lines the rule
actually selects, or state the rule's boundary explicitly (e.g. "notices about
the *record on stdout*, not about the preimage") so that `:527`, `:511` and
`:495` are excluded **by the rule** rather than by the grep that produced the
list. No behaviour change is required either way; the deferral itself can stand.

**Verified by:** the classification table in §1, built by reading every
`writeln!(stderr, …)` between `:454` and `:618`; the two `--random` runs above;
the `--emit-record --json --no-engraving-card` run; `sed -n
'17445,17479p' design/FOLLOWUPS.md`.

---

## 3. Minor

### M-1 — BRANCH — Minor — the caveat quotes `"unknown option hash256"`, but `md` prints `` unknown option `hash256` ``

`hashlock.rs:481-482`. The pre-fold text quoted `"unknown option"`, which was a
literal substring of `md`'s output. The fold made it `"unknown option {0}"`,
which is not: the real message wraps the token in backticks (`md: path
`2of3,hash256=…`: unknown option `hash256``, reproduced in §1). An operator
matching the card's quoted string literally — grep, Ctrl-F — misses. The
`me sysw pack` half is unaffected: *"exactly 64 hex characters"* **is** literal.
One-word fix (drop `{0}` from the quote, or add the backticks).

### M-2 — BRANCH — Minor — the kind-axis recovery clause has no pin, while the method-axis one does

Mutation **F** deletes `, and if the hash line is lost, re-run with no --kind and
match the digest against your descriptor` from `hashlock.rs:503` and the suite
stays at 11 + 11 passed. Its sibling — the method-axis recovery clause — *is*
pinned, at `hashlock_outputs.rs:70` (`"each method that shipped with the version
named on this card"`, spec §11 tests N-3). So the write-down line's four halves
are now: instruction/method **pinned** (round 7 I-2's fix), instruction/kind
**pinned**, recovery/method **pinned**, recovery/kind **unpinned**. Graded Minor
rather than Important because §13.4's four-digest listing is the actual recovery
mechanism for a lost kind and is separately tested — the clause is a pointer to
it, not the mechanism.

### M-3 — BRANCH — Minor — the caveat's `me sysw pack` half is a claim about stdout but sits above the boundary

The fold's new rule puts notices about *"a hazard in what was just emitted"*
below the line. `hashlock.rs:479-487` now says `` `me sysw pack` `` will refuse
*"the record on stdout"* — a statement about stdout, which ships regardless of
`--no-engraving-card` — and it stays above the line. So `--kind ripemd160
--no-engraving-card` emits a record `me sysw pack` will refuse, with no hint
why. Round 7 graded exactly this stuck-and-safe (the 40-hex strip is also
refused, so the cost is confusion, not a wrong outcome), so keeping it
card-scoped is defensible — but it is the new boundary's first unacknowledged
exception, decided in the same commit that wrote the rule, and the rule's
comment does not mention it.

---

## 4. Counts and verdict

| severity | round 7 | **round 8** |
| --- | --- | --- |
| Critical | 0 | **0** |
| Important | 2 | **3** (I-1, I-2, I-3) |
| Minor | 1 | **3** (M-1, M-2, M-3) |
| Secret-handling (non-blocking) | 0 | **1** (inside I-3, item 2) |

**Round-7 disposition.** I-1 **closed** for `--no-engraving-card`, with a new
`--json --no-engraving-card` cell broken → I-1. I-2 **closed**, mutation-proven
with the reviewer's own mutation. M-1 **closed**, and both new claims reproduce
on the real `md 0.14.0` and `me 0.9.0` for all three kinds.

**All six gate numbers in the commit message are TRUE**, independently
re-measured. stdout and exit codes are untouched.

## **NOT GREEN** — 0 Critical / 3 Important.

The fold's content is good: everything it *says* is true where I could run it —
both refusal strings on both doors for three kinds, card volume unchanged at a
15-line maximum, the moved WARNING landing as the card's last line rather than
orphaned, and the three claimed mutations all reddening exactly one test.

What it did not do is finish its own thesis. The commit argues, correctly, that
three occurrences of "notice nested one guard too deep" is a wrong *shape* and
not three slips — and then fixes the shape along one axis while opening a fresh
inconsistency along the other (I-1: unguarded where §13.4 stands down, with the
only pin test blind to the one kind that triggers it), scopes the remaining
instances by grepping a string prefix instead of applying the rule it just wrote
(I-3), and answers the false-PASS finding at the one line the reviewer pointed
at while an identical dead assertion on the *same instruction* sits one file over
(I-2). Each of the three is a small edit. None was visible by reading the diff;
all three took a constructed execution — a flag cell, a `grep -rn`, and a
mutation that stayed green.
