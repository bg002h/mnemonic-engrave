# R0 round 9 — fold verification: the response to round 8

**Scope: two questions only.** Did the fold close each round-8 finding, and did
the fold introduce a new defect? Not a fresh audit. The spec, phases 1/3/4, and
everything closed in R0 rounds 1–7 were not re-reviewed. Round 8's settled facts
were taken as given: the four opcode names, the `hash256` hazard chain, card
volume at a 15-line maximum, and every prior gate number.

**Artifact.** BRANCH `hashkinds-p2` @ `5fe8cd0` ("fold: R0 round 8 -- the second
clause of the rule I had just written"), in
`/scratch/code/shibboleth/ms-worktrees/hashkinds-p2`. Parent `2ff6211`. Diff:
`crates/ms-cli/src/cmd/hashlock.rs` (+50/−19 net across hunks),
`crates/ms-cli/tests/hashlock_kind.rs`, `crates/ms-cli/tests/hashlock_outputs.rs`.
F-536's rescoping is a separate commit, `0a984c9f`, in `mnemonic-engrave`
(`design/FOLLOWUPS.md`).

**Toolchain.** `export PATH=/home/bcg/.cargo/bin:$PATH` inside the worktree ⇒
`cargo 1.85.0 (d73d2caf9 2024-12-31)`, matching `rust-toolchain.toml`.
**Landmine hit and cleared:** `me` on `$PATH` (`/home/bcg/.cargo/bin/me`) is a
stale build (`me 0.7.0`, built Aug 31) that does not even recognize `hash:`
records — it does not match the `me 0.9.0` round 7/8 tested against. Re-pointed
at `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me` (built fresh this
round, confirmed `me 0.9.0`) for the M-1 spot-check; `md` at the path the brief
named is `md 0.14.0`, unaffected. This is an environment fact about my own
verification run, not a fold defect — flagging it so the next round doesn't
repeat the confusion.

**Tree state.** `git status --porcelain` empty before and after every mutation.
`crates/ms-cli/src/cmd/hashlock.rs` = `d787bcc5…9329`,
`crates/ms-cli/tests/hashlock_kind.rs` = `3d93710a…fa3f3`,
`crates/ms-cli/tests/hashlock_outputs.rs` = `1616011e…e2500` — byte-identical to
`5fe8cd0` before, during (re-hashed after each revert), and after this pass.
`/scratch/code/shibboleth/ms-worktrees/p2-shaped` was not touched.

---

## 0. The green claims — independently re-measured

| gate | claimed | measured | verdict |
| --- | --- | --- | --- |
| `cargo test --workspace --locked` | 101 suites / 0 failures | exit 0; `grep -c '^test result:'` → **101**; no result line without ` 0 failed`; `FAILED` → **0** | **TRUE** |
| `cargo nextest run --locked --all-targets` | 584 tests | `584 tests run: 584 passed, 11 skipped`, exit 0 | **TRUE** |
| `clippy -p ms-codec --all-targets --locked -- -D warnings` | 0 | exit 0; no line outside `Checking`/`Compiling`/`Finished` | **TRUE** |
| `clippy -p ms-cli --all-targets --locked -- -D warnings` | 0 | exit 0, same | **TRUE** |
| `cargo +1.95.0 fmt --all -- --check` | clean | exit 0, **0 bytes** of output | **TRUE** |
| `ci/repro/vendor-freshness.sh` | OK | `vendor-freshness: OK — vendor/ satisfies Cargo.lock.` | **TRUE** |

All six gate numbers in the fold's commit message are **TRUE**, independently
re-measured.

---

## 1. Findings CLOSED

### Round-8 **I-1** (hash256 notice unguarded; `--json --no-engraving-card --kind hash256` broke the stream pin) — **CLOSED**

`hashlock.rs:579` now reads `if kind == HashKind::Hash256 && !(args.json &&
args.no_engraving_card) {`. Ran the pinned pair across **every kind × every
source the brief named**, not just the one cell round 8 constructed:

```
$ printf 'correct horse battery staple' | ./target/debug/ms hashlock \
      --hashlock-phrase-stdin --kind hash256 --json --no-engraving-card
--- STDERR (1 line) --- warning: stdout carries private key material...
```

Full matrix, `--json --no-engraving-card`, stderr line count:

| kind | `--hashlock-phrase-stdin` | `--hex -` | `--random --out f` | `--in <ms1 plate>` |
| --- | --- | --- | --- | --- |
| (omitted) | 1 | 1 | 1 | 1 |
| sha256 | 1 | 1 | 1 | 1 |
| hash256 | 1 | 1 | 1 | 1 |
| ripemd160 | 1 | 1 | 1 | 1 |
| hash160 | 1 | 1 | 1 | 1 |

20/20 cells, every one exactly the advisory line. `--emit-record` added to the
phrase-stdin row changes nothing (still 1).

**Checked the mirror failure the brief asked for — over-suppression.** Ran
`--no-engraving-card` **alone** (no `--json`) across all five kind values and
confirmed the notices still fire there (they must, since the pinned pair is the
*only* thing that silences them):

```
--kind hash256 --no-engraving-card (no --json):
the record on stdout needs `hash256` support in `me sysw pack`. ...
WARNING: this record's `hash256:` tag is the ONLY thing distinguishing it ...
```

Both lines present. No cell found where a notice vanished under
`--no-engraving-card` alone or under `--json` alone. Over-suppression not found.

**Mutation-proven, two ways.** (A) Stripped `&& !(args.json &&
args.no_engraving_card)` from `hashlock.rs:579` (the exact I-1 site) — mutation
confirmed applied by `diff` against the saved original, built, ran:
`the_json_purity_contract_holds_for_every_kind` reds, nothing else does,
`11 passed; 1 failed`. Reverted, `sha256sum` back to `d787bcc5…9329`. (B) Same
mutation against the *new* M-3 notice's guard at `:566` — same single test reds.
(C) Inverted the guard to `args.json && args.no_engraving_card` (the guard fires
under the wrong condition instead of the right one) — this correctly breaks
*two* tests (`a_non_sha256_kind_warns_that_md_may_not_accept_the_operand` too,
since it also exercises `--no-engraving-card` alone), confirming the test suite
distinguishes "wrong direction" from "no guard" rather than happening to pass
both by accident.

### Round-8 **I-2** (`hex_source_gets_the_unconditional_warning_and_no_write_it_down_line` asserted a string that existed nowhere — a dead gate) — **CLOSED**

`hashlock_outputs.rs:135-138` now asserts `!se.lines().any(|l|
l.starts_with("phrase:"))` — structural, not textual. Applied round 8's own
mutation D (`d.phrase_chars` → `d.phrase_chars.or(Some(0))` at
`hashlock.rs:501`), confirmed applied by diff, built, ran:

```
$ ./target/debug/ms hashlock --hex - <<< c3e975...
phrase:          0 characters -- write the method line AND the hash line (sha256) next to ...
$ cargo test -p ms-cli --test hashlock_kind --test hashlock_outputs
test hex_source_gets_the_unconditional_warning_and_no_write_it_down_line ... FAILED
test result: FAILED. 10 passed; 1 failed
```

Exactly the test named in I-2 reds; nothing else does. Reverted, hash confirmed
identical to baseline.

### Round-8 **M-1** (caveat quoted `"unknown option"` without the backticks `md` actually prints) — **CLOSED**

`hashlock.rs:482` now reads `\"unknown option \`{0}\`\"`. Ran `md 0.14.0`
directly (note: the correct `me`/`md` binaries, see toolchain note above):

```
$ md compose --wrapper wsh --path 2of3,hash256=98a2...d488
md: path `2of3,hash256=98a2…d488`: unknown option `hash256`
```

Backticks confirmed real. Mutation-proven: stripped the backticks from the
source string at `:482`, confirmed applied by diff, built, ran —
`a_non_sha256_kind_warns_that_md_may_not_accept_the_operand` reds alone
(`11 passed; 1 failed`). Reverted, hash confirmed identical.

Also re-ran the `me sysw pack` half against the **correct** `me 0.9.0` (not the
stale `$PATH` one — see toolchain note): confirmed it really does say `hash:
must be exactly 64 hex characters`, for both `hash256` and `ripemd160` bodies.
The M-3 notice's claim is true.

### Round-8 **M-2** (kind-axis recovery clause unpinned while method-axis was) — **CLOSED**

`hashlock_kind.rs:404-411` now asserts both `line.contains("if the method line
is lost")` and `line.contains("if the hash line is lost")`. Mutation-proven:
deleted `, and if the hash line is lost, re-run with no --kind and match the
digest against your descriptor` from `hashlock.rs:502` (confirmed applied via a
Python replace that asserts exactly one occurrence existed before the edit),
built, ran — `the_write_down_line_names_both_axes` reds alone
(`11 passed; 1 failed`, where round 8's own mutation F on the *unfixed* branch
left this at `11 + 11 passed, 0 failed`). Reverted, hash confirmed identical.

### Round-8 **M-3** (the `me sysw pack` half of the caveat is about stdout but stayed card-scoped) — **CLOSED**

`hashlock.rs:566-575`. The caveat is split: `md compose` stays on the card at
`:479-484` (shortened, no longer mentions `me sysw pack`), and a new notice
below the boundary covers `me sysw pack`. Confirmed by execution: under
`--no-engraving-card` alone, the `me sysw pack` notice still prints (see the
I-1 over-suppression check above) — exactly the piping-path fix M-3 asked for.
Mutation-proven as part of I-1's mutation (A)/(B) above, since M-3's new line
and I-1's guard are the same `if` statement.

---

## 2. New defect

### I-3 — REPO — **Important** — the fold's rescoped F-536 answers I-3 with the same shape of undercount I-3 itself found, and its own arithmetic does not add up

**Location.** `design/FOLLOWUPS.md:17445-17486` in `mnemonic-engrave` (commit
`0a984c9f`), mirroring `crates/ms-cli/src/cmd/hashlock.rs:531-561`.

Round 8's I-3 said the original F-536 (three `WARNING:`-prefixed lines) was
false because it enumerated by grepping a string prefix instead of applying the
boundary's own rule, and named **three specific lines the grep missed**:
`hashlock.rs:527` (the `--random` "only copy" notice), `:495` ("THIS RECORD
CARRIES THE PHRASE … treat the file … like the phrase itself"), and `:511`
("Never use this phrase as a passphrase…").

The fold's rescoped F-536 (`0a984c9f`) picked up **two of the three**: the
`--random` notice (its item 5) and the "never use this phrase as a passphrase"
line (its item 1). It did not add the "THIS RECORD CARRIES THE PHRASE" line —
the one round 8 itself quoted by name and location.

**Independently re-classified every `writeln!(stderr, …)` above the current
boundary** (the boundary comment is now at `hashlock.rs:530`), the same method
the fold's commit message claims to have used:

```
$ grep -n 'writeln!' crates/ms-cli/src/cmd/hashlock.rs | awk -F: '$1<530' | wc -l
17
```

17 statements confirmed (matches round 8's own count). Classified against the
rule as written ("a NOTICE about a hazard in what was just emitted"):

| line(s) | content | class |
| --- | --- | --- |
| 456-459 | "THIS CARD CARRIES THE PREIMAGE…" | card |
| 461 | `digest:` | card |
| 466-472 | `for md compose: --path…` | card |
| 479-487 | conditional `requires md compose…` | card |
| 489 | `preimage (ms1):` | card |
| 490 | `preimage (hex):` | card |
| 491 | `method:` | card |
| 493 | `record (phrase): {r}` | card |
| **494-497** | **"THIS RECORD CARRIES THE PHRASE… treat the file you put it in like the phrase itself"** | **notice — omitted from F-536** |
| 502 | `phrase: {n} characters…` | card |
| 509 | OP_SIZE line | card |
| 510 | "One phrase per policy… Never use this phrase…" | notice — in F-536 (item 1) |
| 513 | WARNING brainwallet | notice — in F-536 (item 2) |
| 517 | WARNING short phrase | notice — in F-536 (item 3) |
| 523 | WARNING `--hex` forever | notice — in F-536 (item 4) |
| 526 | "No phrase exists… only copy" | notice — in F-536 (item 5) |
| 528 | `source:` | card |

11 card lines, **6** notices — not the 5 F-536 claims. F-536's own text says:

> classifying **every** `writeln!(stderr, …)` above the boundary — seventeen of
> them — **eight are card content and five are notices**

`8 + 5 = 13 ≠ 17`. The stated total does not reconcile with the stated
per-category count under either reading (my independent count is 11 + 6 = 17;
even granting the fold's own "eight card", `17 − 8 = 9` notices, not five). This
is not a close call resolved by a differing judgment on one borderline line —
the arithmetic itself is inconsistent, and the specific line round 8 named by
location (`:495`, now `:494-497`) is absent from the list.

**Why Important, not Minor.** This is I-3's own defect class — a follow-up entry
that states a complete enumeration and is not one — recurring **inside the fix
for I-3**, in the same commit that quotes the standing rule *"A follow-up that
certifies a class handled while it is not is worse than one that admits it has
not looked"* as its own justification. The omitted line is exactly the kind of
line the rule exists to catch: a notice that a record just printed on the card
carries the phrase and must be handled like the phrase itself, currently
suppressed under `--no-engraving-card` with nothing tracking that fact. (Whether
*moving* it is warranted is a separate, undecided question — same as the other
five — and this finding does not take a position on that. The defect is that
F-536 claims to have settled the *count* and has not.)

**Verified by:** the `grep -n 'writeln!'` count above; direct read of
`hashlock.rs:450-530` cross-checked against round 8's own 17-row table (same
17 lines, same line numbers before the fold's edits shifted them by a
consistent small offset); `sed -n '17445,17486p' design/FOLLOWUPS.md` in
`mnemonic-engrave` for the current F-536 text.

**Fix shape.** Either add the sixth item (`:494-497`, "THIS RECORD CARRIES THE
PHRASE…") to F-536's list and correct "eight … five" to the true split (11 and
6), or state explicitly why that line is excluded from the rule (e.g. if the
intent is "only lines with no correspondence to a still-suppressible spec
guarantee" or similar) so a reader does not have to re-derive the count to
notice it does not add up.

---

## 3. Minor / not re-litigated

Round 8's M-1/M-2/M-3 are closed (§1). No new Minor found this round beyond the
arithmetic note folded into I-3 above.

---

## 4. Counts and verdict

| severity | round 8 | **round 9** |
| --- | --- | --- |
| Critical | 0 | **0** |
| Important | 3 (I-1, I-2, I-3) | **1** (I-3, recurring — F-536's enumeration still wrong) |
| Minor | 3 (M-1, M-2, M-3) | **0** |

**Round-8 disposition.** I-1 **closed** — mutation-proven both at the exact site
and at the new M-3 notice, over-suppression checked across 20 matrix cells and
not found. I-2 **closed** — the dead gate now asserts structure and reds on the
reviewer's own mutation. M-1, M-2, M-3 **closed**, each mutation-proven.

I-3 is **not closed**: the fold rescoped F-536 in direct response to it, and the
rescoping still omits one of the three lines round 8 named, with a stated total
that does not arithmetically match its own stated per-category breakdown. Code
behavior is unaffected — this is a documentation-accuracy defect in a follow-up
entry, not a runtime defect — but it is the same finding under the same name,
answered incompletely.

All six gate numbers (101 suites / 584 tests / clippy 0×2 / fmt clean / vendor
OK) are independently re-measured **TRUE**. Tree left byte-identical
(`d787bcc5…`, `3d93710a…`, `1616011e…`) and `git status --porcelain` clean.

## **NOT GREEN** — 0 Critical / 1 Important.

The code-side fix (I-1, I-2, M-1, M-2, M-3) is solid: every claim reproduces on
execution, every touched test mutation-proves to exactly the test it names, and
the broadened matrix (4 sources × 5 kind-values × the pinned pair, 20 cells)
found no over-suppression regression — the mirror-image failure this round was
specifically asked to hunt for. What remains open is narrower than round 8: one
follow-up entry, rescoped as I-3's direct fix, that still doesn't count right.
