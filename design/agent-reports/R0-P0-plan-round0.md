# R0 — P0 implementation plan, ROUND 0

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` (244 lines, DRAFT v1)
**Worktree:** `/scratch/code/shibboleth/_work/p0r0/mnemonic-engrave` @ `786b709` (branch `review/p0-r0`)
**Date:** 2026-08-26. **Reviewer:** independent R0 agent, round 0. Never reviewed before.

**Counts: 4 Critical / 5 Important / 7 Minor / 2 Nit. Verdict: NOT GREEN.**

Method note: every binary invoked by absolute path with `< /dev/null`. Exit codes captured
into a variable **before** any command substitution — `echo "$(basename $p) -> $?"` reports
`basename`'s status, not the program's, and my first pass at the exit-code sweep read 0 for
all five binaries because of it. Re-measured correctly below.

---

## THE NAMED ATTACK — §2.2's refusal to settle `0o044` vs `0o077`

### Verdict: the SCOPING is right. The plan's STATEMENT of it is not, and C1 below is how it fails.

**The concrete case, same machine and same intent.** An operator produces a bearer artifact
into a file another local account can write. `>` truncates but keeps an existing file's mode —
`mt`'s own remedy text says so in as many words (`chmod 600 <file>  then re-run -- \`>\`
truncates but keeps the mode`, `validate.rs:683`). So with the target already at 0620:

| command | destination 0620 | why |
| --- | --- | --- |
| `mt encode --qr --in tx.hex > out.txt` | **exit 1, REFUSED** | `0620 & 0o077 != 0` |
| `me sysw pack --in records.txt > payload.bin` | **exit 0, written** | `0620 & 0o044 == 0` |

Both artifacts are substitutable by anyone with write access before they are used. `mt`'s
strings get cut into a plate and later broadcast; `me`'s payload gets flashed to the machine
that does the cutting. **The hazard is identical in kind, and the weaker of the two rules is
the one on the physically-destructive path.** The operator learns from `mt` that the
constellation checks this, and gets no check from `me`.

**Why the divergence is nevertheless survivable, and hoisting is not.** The cost of the
divergence is an inconsistent lesson. The cost of settling it inside a shared dependency is a
behaviour change to two shipped funds-adjacent binaries made by a *refactor* rather than by a
*ruling*: tightening `me` to `0o077` changes its refusal surface with no operator ruling behind
it, and loosening `mt` deletes a live check. Neither belongs in P0, and nothing about sharing
the mechanism forces the choice — **provided the crate carries no mask at all.** That proviso
is exactly what the plan does not state, and C1 is what happens without it.

**But "a deliberate disagreement" is not established, and the plan is less honest than its
own spec.** `me`'s function is documented *"Is this process's stdout a REGULAR FILE that others
can **read**?"* and its refusal says *"grant read to group or others"* — scoped to readability
throughout, with nothing on the record weighing group-*write*. F-260 establishes only that `me`
is internally consistent: *"`me` is NOT affected on this axis — checked, not assumed. Its mask
is `0o044` … so its 'world-readable' wording matches what it tests."* The spec says the honest
thing — §5a: *"`mt` is arguably right that a group-writable destination is a hazard **`me`
currently ignores**."* The plan converts "currently ignores" into "a deliberate disagreement"
(§2.2) and then files it under **Out of Scope** (§7). After that, §7 is the record that the
question was considered and closed — which is the opposite of what an unfiled gap needs.

**What the plan must state so a later reader does not silently "fix" it:**

1. **The crate ships no mask, ever, and adoption never changes a consumer's mask.** This is the
   only sentence that blocks the silent convergence in *both* directions.
2. **`me`'s `0o044` is an open gap, not a ruling** — filed with an owning phase, so it has a
   home other than §7's out-of-scope list.
3. **Name the two hazards as different in kind** — read is confidentiality, write is
   substitution — so the next reader does not have to re-derive why two masks differ and
   conclude one is a typo.

---

## CRITICAL

### C1 — `stdout_world_readable_mode` fuses `me`'s POLICY into the mechanism, and §4 steps 1 and 2 give contradictory orders about it. Either literal reading publishes one binary's policy as the crate's mechanism.

**Site:** §1's 11-function table (row `stdout_world_readable_mode`, 25 lines); §3
(`src/fd.rs — MECHANISM: fstat, extract mode. No policy.`); §4 steps 1 and 2.

**What is wrong.** `me`'s function is not mechanism. At `crates/me-cli/src/main.rs:896-918` it
fstats fd 1, exempts char devices, computes `mode & 0o777` — and then, at **line 912**, applies

```
(mode & 0o044 != 0).then_some(mode)
```

returning `Option<u32>`, i.e. `None` when *`me`'s own mask* says clean. **The mask is inside the
function, not at the call site.** §4 step 1 requires it move with "no behaviour change"; §4 step
2 requires `fd.rs` to carry "**no** policy assertion". Both cannot hold for this function
without a split the plan never names.

**The concrete failure.** The cheap way to satisfy step 1 and the 388 tests is to move it
verbatim — after which `mnemonic-io-lib::fd` exports `0o044`. Step 8 publishes that,
irreversibly. At P1, `mt` adopts the crate. `mt`'s mask is `0o077`
(`mnemonic-transaction/crates/mt-cli/src/validate.rs:653`, inside `world_readable_stdout_guard`
at `:627`) and it currently REFUSES a group-writable stdout. `mt` then either keeps its own copy
— defeating D5, the crate's entire reason to exist — or adopts the crate's mask and **silently
loses a shipped refusal on the transaction-engraving path.** P1's gate ("the diff to [`mt`'s 237
tests] enumerated and each edit justified by a named §6 ruling") *accepts* that deletion,
because "§5a: the crate holds the mechanism" is a real ruling to cite.

Net: a funds-safety check on the artifact an operator cuts into metal, deleted by a refactor, by
an implementer with no authority to rule it — **the author's named worry arriving from the
opposite side.** Not the disagreement persisting; the disagreement resolved silently in favour
of the weaker rule.

**What closes it.**
- §1 records that this is **the one function of the 11 whose move is a SPLIT, not a move.**
- §3 fixes `fd.rs`'s contract: return the raw `mode & 0o777` for regular files, `None` for char
  devices and for a failed `fstat` (fail-open). **No mask anywhere in the crate.** And say that
  the char-device exemption and the fail-open ARE shared mechanism — both binaries implement
  both identically (`me` main.rs:906-917 vs `mt` validate.rs:645-655, comments included) —
  **while the mask is not.** Without that sentence, an implementer reading "No policy" literally
  pushes the char-device exemption out to callers, where it is load-bearing for `/dev/null`
  (mode 0666) and where a caller can forget it.
- §4: step 1 moves it intact; step 2 splits it, and `me`'s call site regains `& 0o044` so
  behaviour is *still* unchanged.
- §7's out-of-scope line states that adoption never changes a consumer's mask.

### C2 — §6d's normative pre-parser ordering is assigned to P0 by the spec, appears nowhere in the plan, and §1 cites its absence as evidence the work is easy.

**Site:** §1 ("**0** hits for … `clap`, or `std::env::args`. Nothing reaches for the binary's
argument parser … so the move is **mechanical rather than a rewrite**"); §4 step 5; §6's seven
conditions.

**What the spec requires of P0.**
- §6d: *"The guard runs on raw `std::env::args()` BEFORE any argument parser sees the material.
  **This is NORMATIVE, not an implementation note (C-4).**"*
- §6d's detector table marks `me` `read_records` as **post-clap ✗**, and rules: *"**P0 cannot
  'extract' a thing that does not exist in either source; it must build the union.**"* Two
  layers; layer 1 (flag-keyed, modelled on `NodeType::is_argv_secret_bearing`) is *"the one both
  references lack"*.
- §5a: *"§6d rules the argv override's own parse must run on raw argv, and `me` currently ships
  it as an ordinary clap flag (`me-cli/src/main.rs:252`) — so `me` is not already compliant, and
  no phase owned that fix until now. **P0 owns it.**"*
- §7 P0's content column: *"argv guard **with pre-parser ordering**"*.

**Verified in the plan.** `grep -in 'argv|6d|override|raw'` returns 7 hits — none about
ordering, `std::env::args`, layer 1, or the override's own parse. The 431-line closure's 0 hits
for `clap`/`std::env::args` are the **symptom**, not the achievement: they are what
"post-parser" looks like from inside.

**The concrete failure.** All eight steps execute, §6's seven conditions pass, 0.1.0 publishes.
The published crate's argv guard is the post-clap, class-keyed one. §6d's measured leak stays
live — a token the guard does not classify reaches clap and clap echoes it (`mt encode --qr
deadbeefcafe` → `error: invalid value 'deadbeefcafe'`, exit 2) — and `mnemonic bundle
--passphrase <arbitrary text>` is caught by nothing. Retrofitting layer 1 afterwards changes the
guard's entry point (raw `&[String]` + a flag-name table vs. a parsed record list): **a semver
break on the crate P0 exists to create, across six manifests.**

**What closes it.** §1 reframes the 0-hit measurement as the gap it is. §3's `records.rs`
specifies a pre-parser entry point taking raw argv plus the flag-name table. §4 gains a step for
layer 1 with a test that fails today (`--passphrase <arbitrary text>` admitted). §6 gains a
condition that the guard **and the override's own parse** are both decided before
`Cli::parse()`, asserted at least in the donor.

### C3 — `me sysw pack --expect` is in P0's content and in the plan's own closure condition 4, but no step builds it, and the kind vocabulary the spec ORDERS THE PLAN to enumerate is not enumerated.

**Site:** §3 (`records.rs — … kind vocabulary`, four words); §4 (eight steps, none builds
`--expect`); §6 condition 4.

**What the spec requires.**
- §7 P0: *"**and `me sysw pack --expect` in full — the kind vocabulary, the flag, and §6g's
  refusal on an incomplete chunk set of a named kind (I-6)**"*. §7 P4: *"`--expect` is BUILT in
  P0; P4 exercises it."*
- §6g: *"The kind vocabulary must be **fixed and enumerated in the plan**, not invented per call
  site, and it must map onto exactly one of those two discriminants per kind."*
- §6g: *"the kind `transaction` is satisfied by `Class::Mt` OR `Class::Tx`, **and the plan must
  implement it that way**"* … *"**Left unstated, this is first detectable at P4** — after the
  vocabulary has shipped inside a released crate consumed by five CLIs."*

**Measured.** `grep -n 'expect' crates/me-cli/src/main.rs` (excluding `.expect(`) returns 5
hits, all prose or an unrelated version check — **`--expect` does not exist in `me` today.** So
this is build work, not extraction, and none of the 11 functions is it. `me`'s `Class` has a
single `MdMk` variant (`sysw/record.rs:44`), so `descriptor` vs `cosigner` must resolve through
the HRP, exactly as §6g says.

**The concrete failure.** An implementer walking §4 reaches step 8 — **publish, irreversible** —
having never built `--expect`, then meets §6.4 and must build a spec-normative vocabulary,
including the `Mt ∪ Tx` union and the incomplete-chunk-set escalation, *after* the crate is
published. §6g names both wrong single-class bindings and their costs: `Tx` alone false-refuses
the hand-engraving path §1/§3/§4/§6a are all about; `Mt` alone makes §10's acceptance criterion
unsatisfiable.

**What closes it.** Enumerate the vocabulary as a table in the plan (kind → discriminant → HRP
or `Class`), with `transaction = Mt ∪ Tx` stated. Add TDD steps for the flag and for §6g's
incomplete-set refusal, **before** step 8.

### C4 — §4 step 7's gate ("all 388 tests still pass, unchanged") is unsatisfiable together with §6 condition 6 ("F-259 … cannot recur by construction").

**Site:** §4 steps 3 and 7; §6 condition 6.

**Measured.** `cargo nextest run --locked` in this worktree → `Summary [12.348s] 388 tests run:
388 passed, 1 skipped`. F-259's fix "by construction" — a payload kind as a **type** rather than
a bool — changes `write_block`'s signature. `write_block(out_given, allow_world_readable,
stdout_is_tty, world_readable_mode)` is called **positionally at 8 assertions** in
`main.rs:2206-2224` (`write_block_decides_both_gates_once`) plus 2 production sites (1200,
2070); `emit(bytes, out, allow_world_readable)` at 3 sites, one passing
`WIPE_IMAGE_CARRIES_NO_SECRET` in the `allow_world_readable` slot (main.rs:1385-1389).

This repo's own FOLLOWUPS says it outright, of that same unit test: *"It never contemplates the
second meaning, so **it locks the defect in while reading as deliberate**."*

**The concrete failure.** The implementer meets a contradiction at step 7 and resolves it the
cheap way: leave `emit`/`write_block` on the bool, satisfy "unchanged", and declare §6.6 met
because a payload-kind type exists somewhere in `observation.rs` that nothing calls. **F-259
still reproduces on the shipped binary while P0 closes green** — condition 6 satisfied by the
construction of an unused type.

**What closes it.** Step 7's gate becomes *"all 388 tests pass, with the diff to them enumerated
and each edit justified by a named finding"* — the shape §7 P1 already uses for `mt`. §6.6 says
explicitly that `emit` and `write_block` change signature and that
`write_block_decides_both_gates_once` is expected to change.

---

## IMPORTANT

### I1 — Step 1's gate cannot fail for the terminal arm, which is one of the 11 being moved and the one with the funds-bearing behaviour.

FOLLOWUPS F-259: *"all 12 tests in `world_readable_output.rs` redirect to files, so **none
reaches the terminal arm at all**."* Verified: `grep -c '#[test]'
crates/me-cli/tests/world_readable_output.rs` → **12**; `main.rs` holds only **3** `#[test]`
total. F-259 was reproducible only under `script -qec`. So `refuse_terminal_destination`
(main.rs:1026-1055) and `WriteBlock::Terminal`'s end-to-end path have **no test in the 388**.
Step 1's entire proof of "no behaviour change" is "the 388 still pass" — green whether or not
the terminal refusal survives the move. **Closes it:** pin the terminal arm with a pty assertion
(the repo already has the technique) as a step-1 precondition, or state in §4 that the terminal
path is uncovered and that step 1 proves nothing about it.

### I2 — `src/exit.rs — the exit-code table` has C1's exact mechanism/policy problem in a second file, and unlike the mask the plan never flags it.

Measured (`<binary> --definitely-not-a-flag < /dev/null`, rc captured before substitution):
**md 2, mk 64, ms 64, mt 2, me 2** — matching §6f's table. §9b rules *"**The clap-usage
exit-code split, 2 versus 64**"* explicitly OUT OF SCOPE, and §6f rules the `mnemonic`/`mk`
invalid-artifact `2` collision **stands**. So there is no single table. §3 names one; §4 step 6
tests "codes match §6f" without saying which rows. **Concrete failure:** `exit.rs` ships
`EXIT_USAGE = 2` (the donor's), and P2's `ms` adoption either flips a published CLI's usage code
from 64 to 2 — a change ruled out of scope — or ignores the constant, making the "table"
decorative. Discovered at P2, after publish, where changing the export is breaking. **Closes
it:** §3 says `exit.rs` holds only the codes the five binaries agree on, enumerates them, and
names 2-vs-64 and the invalid-artifact collision as deliberately absent.

### I3 — §6 condition 5 weakens a spec gate from "measured" to "measured, or explicitly recorded as unanswerable", and no step measures it.

Spec §6h: *"No command for it is stated here because none has been verified; **P0 owes the
measurement** before it writes the sentence."* §7 P0's gate: *"+ the in-memory-history question
of §6h **measured**"*. The plan adds an opt-out the spec does not offer, and §4 step 4's only
remedy test is negative-content (`does not contain history -d`), which cannot reach the
question. **Concrete failure:** the question is answerable in minutes (interactive shell, `sed`
the histfile, exit, re-read), so "unanswerable" would be a false record — and `me`'s shipped
remedy `sed -i '/me sysw pack/d' "$HISTFILE"` is advice an interactive shell may undo on exit,
so the operator who follows it believes bearer material is purged when it is not. **That is the
exact defect class (`history -d` reports success and purges nothing) that `remedy.rs` exists to
prevent, recreated inside the fix for it.** Closes it: restore the spec's wording; give step 4 a
positive test that runs the recipe under an interactive shell.

### I4 — F-260's owning phase is P0, and the plan neither burns it down nor reassigns it.

FOLLOWUPS headers: F-259 *"(owning phase: **P0**, or sooner)"*; F-260 *"(repo:
**mnemonic-transaction**; owning phase: **P0**)"*, with a subsection titled *"Why this belongs
to P0 rather than a `mt` patch."* The plan touches `mt` in no step — §7 P1 is where `mt` adopts
— and §6.6 asks only that the two "cannot recur **by construction**", a statement about the
crate rather than about the two shipped wrong messages. So F-260's live defect (`mt encode`
telling an operator that mode 0620 *"grants read to group or others"*) survives P0's green with
its owning phase already passed. Per the constellation rule, an item whose owning phase has
passed is **overdue, not deferred**. Closes it: either §6 gains a condition that `mt`'s message
is derived from the observed mode — which means P0 touches `mt`, contradicting §7 — or the plan
explicitly re-assigns F-260 to P1 and FOLLOWUPS is updated in the same fold.

### I5 — the closure is enumerated as FUNCTIONS only; the types and constants it depends on are not listed, so §1's own argument is left incomplete by the section that makes it.

Of the 11: `destination` / `write_block` / `refuse_write_block` / `emit` depend on the
`main.rs`-private enums **`Destination`** and **`WriteBlock`**; `refuse_write_block` /
`no_records_guard` / `read_records` on **`EXIT_USAGE` / `EXIT_REFUSED` / `EXIT_OK`**. None
appears in either table, yet §1's argument is *"Each must move or the extraction does not
compile"*. The three `#[test]`s in `main.rs`'s `mod tests` (line 2164) exercise `destination`
and `write_block` through private paths and must move too — so step 1's one measurable gate,
*"`main.rs` shrinks by ~431 lines"*, is understated by the test module plus the type and const
definitions. The consequence is a compile error rather than a wrong result, hence Important not
Critical — but it is precisely the failure mode §1's closing line exists to prevent
(*"discovers the other eight one compile error at a time"*), committed one level down.

---

## MINOR

**M1 — §3: "The 431 lines contain 4 `eprintln!` and 4 `println!`." There are ZERO `println!`.**
Measured over the author's own 431-line regions (reconstructed exactly: the per-row counts are
next-`fn` deltas and sum to 431). `eprintln!` → **4** (main.rs:1035, 1058, 2039, 2091) ✓.
`println!` → **0** ✗. The closure's only stdout write is `emit`'s
`std::io::stdout().write_all(bytes)` (main.rs:2081-2083), which is the payload itself and must
**stay** a write, not become a returned string. §3's ruling stands on the 4 `eprintln!` alone —
but as written it points an implementer at four stdout writes that do not exist, and the one
that *does* exist is the one the ruling must not be applied to. §3 should say so. *(Anyone
re-checking this needs a real word boundary: `grep '[^r]println!'` matches `eprintln!`, because
"eprintln" is `e` + "println". That grep reports 4 and is wrong.)*

**M2 — §1: "`me-cli` is the ONLY crate in the constellation with both `lib.rs` and `main.rs`" is
false.** Measured across all first-party crates: `mnemonic-engrave/crates/me-cli` LIB+MAIN
**and** `mnemonic-toolkit/crates/mnemonic-toolkit` LIB+MAIN. The plan's own source spec says so
at §5a-i: *"`mnemonic-engrave` and `mnemonic-toolkit` ship a lib target as well."* The two
sub-claims in the same sentence are TRUE (5 of 5 `-codec` lib-only: md/mk/ms/mt/wc; 4 of 5
`-cli` main-only). Fix: "the only `-cli` crate with both", which is what §5a-i establishes.

**M3 — §2.1's cross-repo note cites the wrong mask line.** It gives `validate.rs` *"line 585
(the `if mode & 0o077 == 0` mask)"* as one of the two sites belonging to
`world_readable_stdout_guard`. Line 585 carries that exact text but sits inside
**`file_mode_warning`** — the input *warning*, which never refuses. The mask producing the
measured 0620 **refusal** is line **653**, inside the guard at 627. FOLLOWUPS F-260 cites both
(`validate.rs:585,653`); the plan kept the one that does not refuse. The note instructs a future
folder to *"re-verify by hand"*, and hand-verification at 585 finds a textual match in the wrong
function — the failure mode the note exists to prevent. Fix: cite 653, or both as F-260 does.

**M4 — §2.4's "100 pure lines and 32 of mechanism" is not reproducible and is off under any
reading.** No definition of the split is given. `read_records` spans main.rs:1921-2051 (131
lines; 132 under the author's region measure). IO-bearing lines are **1927** (`read_to_string`
for `--in`) and **2037-2039 / 2046-2047** (the TTY hint and the stdin read) — a mechanism half
of roughly 22 lines counting whole branches, not 32. **The claim it supports is TRUE and is the
load-bearing part:** lines 1933-2035 (the argv gate) contain no `std::fs`, `stdin`,
`read_to_string`, `eprintln!` or `println!` — the only regex hits at 1936 and 1990 are comments.
Fix: state the line ranges instead of the totals.

**M5 — §4's column header is "the test that must fail first", and rows 1 and 7 both hold tests
that must PASS.** "Each step is RED first" cannot be executed for a pure move (step 1) or for a
rewire that must not change behaviour (step 7). 2 of 8 rows contradict the column's contract,
and an implementer told to write a failing test first has nothing to write. Fix: split into
"gate" and "the test that must fail first", or mark 1 and 7 as regression gates.

**M6 — the 11 functions are never mapped onto the 7 files.** §3 lists 7 files, §4 sequences 6,
and nothing says where `emit`, `write_private`, `refuse_write_block`,
`refuse_terminal_destination` or `no_records_guard` land. `remedy.rs` is scoped to "purge/remedy
text" and `observation.rs` to "what was measured, as types" — neither obviously owns a refusal
message that is both. With C1 open this is more than bookkeeping: **the file a function lands in
is what decides whether its mask travels with it.**

**M7 — the plan never reconciles §8, "What is NOT verified, and must be before the plan
closes".** Two of §8's bullets address P0 directly: the §6h in-memory question (see I3), and
*"**P0 must not read an absent `CLOSED` as open work**: grep both, or the plan will schedule
work that is already finished"* — this repo closes follow-ups in two vocabularies (`CLOSED` and
`DONE`). A third (*"which existing invocations break … must not be re-deferred"*) is partly
carried by P2's gate. The plan cites §5a/§5b/§6b/§6d/§6f/§6h/§7 as sources and never mentions §8.

---

## NIT

**N1 — "388 tests" is 388 *run* / 1 skipped, out of 389 `#[test]` attributes.** `cargo nextest
run --locked` → `388 tests run: 388 passed, 1 skipped`; `grep -rc '#[test]' crates
--include='*.rs'` → **389**. The figure is right for the run count; a gate written as an
equality should say which count it means.

**N2 — the `#[cfg(not(unix))] fn stdout_world_readable_mode` stub (main.rs:921-923) is a 12th
definition in neither table**, while §4 step 1's line *"proves the closure is really 11 and not
12"* reads as though that were the open question. It moves with its twin; naming it costs one
clause.

---

## FACTUAL CLAIMS TESTED — command, output, verdict

| # | claim (site) | command | printed | verdict |
| --- | --- | --- | --- | --- |
| 1 | 0 hits for `std::process::exit`, `Cli`, `clap`, `std::env::args` across the 431 (§1) | python over the author's exact 11 regions (next-`fn` deltas, total **431**) | `0 / 0 / 0 / 0` | **TRUE** |
| 2 | 4 `eprintln!` in the 431 (§3) | same regions | `4` — lines 1035, 1058, 2039, 2091 | **TRUE** |
| 3 | 4 `println!` in the 431 (§3) | same regions, word-boundary regex | `0` | **FALSE** → M1 |
| 4 | `main.rs` is 2,226 lines; the closure is 19% (§1) | `wc -l crates/me-cli/src/main.rs` | `2226`; 431/2226 = 19.4% | **TRUE** |
| 5 | `read_records` = 100 pure / 32 mechanism (§2.4) | span 1921-2051, IO-line scan | IO at 1927, 2037-2039, 2046-2047 ≈ 22 | **NOT REPRODUCIBLE** → M4 |
| 6 | its argv gate performs no IO (§2.4) | scan of 1933-2035 | only 2 hits, both comments | **TRUE** |
| 7 | `read_records` has 13 spawn tests, 0 unit tests (§2.4) | `grep -rn 'fn .*argv' crates/me-cli/tests/` | **13** fns; `main.rs` has 3 `#[test]`, none on it | **TRUE** |
| 8 | 12 tests in `world_readable_output.rs` (§2.4) | `grep -c '#[test]'` | `12` | **TRUE** |
| 9 | `mt`'s purge text advises zsh `history -d` (§3) | `grep -n 'fn purge_command' -A 40 …/validate.rs` | `:543 "history -d $HISTCMD && fc -W  # zsh"` | **TRUE** |
| 10 | `mt`'s fish branch matches on the bearer material (§3) | same | `:544 "history delete --contains <tx>  # fish"` | **TRUE** |
| 11 | `me`'s mask at `main.rs:912` (§2.2) | `grep -n '0o044' main.rs` | `912: (mode & 0o044 != 0).then_some(mode)` | **TRUE** |
| 12 | `is_secret`/`is_bearer`/`is_argv_forbidden` at record.rs:73/89/105 (§1) | `grep -n` | 73 / 89 / 105 | **TRUE** |
| 13 | `world_readable_stdout_guard` at validate.rs:627 taking `(allow: bool, form: Form)` (§2.1) | `grep -n` + read | exact match | **TRUE** |
| 14 | validate.rs:585 is "the `0o077` mask" of that guard (§2.1) | read 555-700 | 585 is in `file_mode_warning`; the guard's mask is **653** | **MIS-CITED** → M3 |
| 15 | 388 tests (§4 steps 1, 7) | `cargo nextest run --locked` | `388 run: 388 passed, 1 skipped`; 389 `#[test]` | **TRUE (run count)** → N1 |
| 16 | §5b's invariant, 16 checks passing (§6.2) | `<bin> <verb> --help < /dev/null`, rc before substitution | 16/16 → 0 | **TRUE** |
| 17 | `mnemonic inspect` → 2 bad HRP, 1 md1 decode failure (§6.3) | `mnemonic inspect notanartifact` / `md1nonsense` | `2` / `1` | **TRUE** |
| 18 | `me-cli` is the only crate with lib+main (§1) | lib/main sweep over all first-party crates | `me-cli` **and** `mnemonic-toolkit` | **FALSE** → M2 |
| 19 | 5 of 5 `-codec` lib-only; 4 of 5 `-cli` main-only (§1) | same sweep | md/mk/ms/mt/wc-codec lib-only; md/mk/ms/mt-cli main-only | **TRUE** |
| 20 | F-259 traces to a bool two callers read differently (§2.3) | `grep -n 'WIPE_IMAGE_CARRIES_NO_SECRET'` | main.rs:1385-1389, passed in the `allow_world_readable` slot | **TRUE** |
| 21 | `--expect` exists to be extracted (§6.4 presupposes) | `grep -n 'expect' main.rs` minus `.expect(` | no such flag | **DOES NOT EXIST** → C3 |
| 22 | one shared exit-code table is possible (§3, §4.6) | `<bin> --definitely-not-a-flag` | md 2, mk 64, ms 64, mt 2, me 2 | **CONTRADICTED** → I2 |

**Not re-derived** (machine-checked before dispatch, per the brief): `plan-cite-check.sh` 4/4,
`plan-table-check.sh` 25 rows, the 431-line closure total, the 0620/0600 divergence with its
0600 control, and the crates.io availability of both name spellings.

---

## SPEC FIDELITY — where the plan is FAITHFUL

Recorded so a fold does not re-litigate settled ground.

- **§5c "decided, not scheduled"** — §7's first bullet reproduces D7 correctly: nothing
  relocates, every verb keeps its home through P0–P4. ✓
- **§6h remedy provenance** — §3's *"`mt`'s purge text is NOT a source"* matches §6d's ruling
  that §7 P0's "from `mt` and `me` jointly" was the wrong site; both cited defects in `mt`'s
  text are verified live (claims 9, 10). ✓
- **§5a's boundary lines** — §3 states two of the spec's four bullets verbatim, and the other
  two (mechanism-not-policy; DO hold the measured vocabulary) are carried in §2.2 and §2.3.
  Incomplete in §3 but **handled elsewhere in the plan**, so not filed.
- **§5b's criterion and the toolkit as sixth consumer** — §7's third bullet is right that
  toolkit adoption is not P0's (§7 P3 owns `mnemonic`). ✓
- **§5 the irreversible step** — the `-A` requirement, the `serde` control as the gate, both
  spellings, and "a 404 is availability at a moment" all reproduce §5a-i faithfully, including
  the operator gate on publication. ✓

## SPEC FIDELITY — where it DIVERGES

C2 (§6d pre-parser, §5a "P0 owns it"), C3 (§7 P0's `--expect`, §6g's enumerate-in-the-plan
order), I2 (§9b's out-of-scope 2-vs-64), I3 (§6h "measured" → "or unanswerable"), M7 (§8 never
reconciled).

---

## VERDICT

**4 Critical / 5 Important / 7 Minor / 2 Nit — NOT GREEN.**

No code may be written. The single most important finding is **C1**: the plan's central ruling —
the crate holds mechanism, each binary keeps its policy — is *correct*, and the plan does not
carry it into the one function where `me` fused the two. Steps 1 and 2 order opposite things
about `stdout_world_readable_mode`, and the reading that satisfies step 1 and the 388 tests
publishes `0o044` as `mnemonic-io-lib`'s mechanism — after which P1's adoption can delete `mt`'s
group-writable refusal under cover of a §6 ruling, with a gate written to accept the diff.
