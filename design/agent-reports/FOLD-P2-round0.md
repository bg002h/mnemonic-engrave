# FOLD — `IMPLEMENTATION_PLAN_P2_ms_adopts.md`, R0 round 0

**Responds to:** `design/agent-reports/R0-P2-plan-round0.md` (2C / 8I / 8M / 0Nit),
persisted verbatim at `802cd78` on `fold/p2`.
**Folded by:** an independent agent, not the plan's author.
**Date:** 2026-08-27. **Subject code:** `/scratch/code/shibboleth/mnemonic-secret`
at `7c12f66`, clean tree, read-only, built with `cargo build --locked` before any
behavioural measurement. `me` built from this worktree.

---

## THE OPERATOR RULING, AND WHAT IT MOVED

Received mid-fold, verbatim:

> *"While we prefer secret handling to work well, failure to handle secrets
> secretly will no longer be critical or important. Such issues will be logged
> when discovered for future optimization."*

Applied as a re-grade of a class of **harm**, not a class of wording. A defect
whose harm is material becoming visible is logged and scheduled. A defect that is
a **gate which cannot fail**, a **refusal that does not refuse**, or a **tool
advising a path that does not run** still blocks, whatever sits nearby. Nothing
was relabelled in either direction to move it across that line; where a finding
has halves in both classes it is split and both halves are stated.

---

## VERDICT TABLE

| # | verdict | what changed, or why not |
| --- | --- | --- |
| **C-1** `--flag=value` bypasses the guard's 56-row gate | **FIXED** — re-graded **Critical → Minor** by the ruling, then closed anyway | Reproduced independently: three `=`-joined spellings exit 0 with material on argv. The guard entry now requires the donor's fourth normalisation — split on **every** `=` and normalise each half (`crates/me-cli/src/main.rs:354`, documented at `:347`) — and the generated gate was rebuilt to **92 rows** and RUN. Closing it cost one normalisation in a list the plan already specified, so it was not left open. Logged as **F-302** regardless, per the ruling. The gate's cell now states what the 92 does **not** cover |
| **C-2** `me`'s shipped remedy is broken and the plan records it as correct | **FIXED, SPLIT INTO TWO CLASSES** | **Blocking half** (a tool advising a path that does not run, plus a gate whose stated basis does not exist): the entry is rewritten as a REPAIR, not a retarget; it is reclassified RED-first; it now BUILDS the test rather than retargeting one that does not exist; and it MOVED in the order. **Logged half** (the exposure the failed remedy leaves behind): recorded in **F-301**, not gate-holding. **F-301 also carries the live `master` defect**, which is not P2's plan to fix |
| **I-1** §3's blocker is discharged; the pin's gate cannot fail | **FIXED** | §3 rewritten around re-measurement: `origin/master` (`6c24e62823e6c1ac02aa3862cd6020674bf58544`) carries `crates/mnemonic-io-lib`; `write_private` has **one** hit there and **zero** in `me-cli`; fish is prescribed. The retracted ordering rationale is named as retracted, and the order now rests on §7's ruling plus the one real dependency. The pin's gate is replaced with four checks that can fail, including a compile of a `use` line naming all four adopted items |
| **I-2** the override's mechanism cannot pass its own gate | **FIXED** | Reproduced: removing the token gives exit 64 on `encode` and `split`; removing only the value gives 64 too. The entry now **substitutes** rather than removes — the admitted token becomes `-`, the stdin sentinel, on flag and positional channels alike, and the material is seeded into the internal path ahead of stdin. §6d's actual wording was re-read at source and it permits this. Two new assertions: the same run with **stdin closed** must still exit 0, and the override alone must still fail the required group |
| **I-3** no private route for `derive` from phrase + passphrase | **FIXED, with the reasoning DECLINED** | The residue is real and was named nowhere; §2.5 now states it, the freed-stdin entry gates it, and **F-303** carries the one-command form. **The report's claim that NO private form remains is refused**: a two-command route exists and was measured — `ms encode --in seed.txt --out card.ms1` then `ms derive --in card.ms1 --passphrase-stdin` — reproduced today in the closest available form at rc 0 / rc 0, `master_fingerprint: ca2c62d2`. The gate asserts it derives the **same fingerprint** as today's argv form, so it is proved equivalent, not merely runnable |
| **I-4** the ungrouped-stdout gate is unsatisfiable, twice | **FIXED** | Both sites now read `ms encode --in <file>`, matching §7's own wording, where *no flags* scopes to `me sysw pack`. Measured: `ms encode` with no flags exits **64**, so the closure condition could never have passed |
| **I-5** `repair`'s stdout is a report AND an artifact | **FIXED** | Reproduced: `ms repair --ms1 <one induced error>` exits **4** and prints two `#` lines then the corrected `ms1`. `--out` on `repair` is now RULED — the artifact line alone, report stays on stdout — with a byte-pinned assertion that a whole-stdout implementation fails, and the non-zero-exit-with-a-written-file case stated. **F-285 in `design/FOLLOWUPS.md` was corrected too**: this diff falsified a sentence it never touched |
| **I-6** the verb-qualified purge command has no allowlist | **FIXED** | The donor's argument at `crates/me-cli/src/main.rs:400` is carried over. `ms`'s allowlist is its **12** command words, enumerated from `ms --help`; `ms` nests no subcommands so exactly one word is appended; a non-allowlisted token falls back to bare `ms` **and the emitted text says the match is broad**, because over-matching costs history lines while under-matching leaves a seed behind a `sed` that exited 0. A mistyped-verb gate row was added and it is RUN, not read |
| **I-7** two `me-cli` citations stale by 24 lines | **FIXED** | Both re-located by symbol/string: the emitted literal is `crates/me-cli/src/main.rs:2164`, and the comment is four lines above it. **The stale numbers are described, not written**, so two known-wrong citations are not put back into a document the gate would pass. A third stale citation the report did not find was caught by re-measuring: the `ROOTS` entry naming `mnemonic-secret` moved from line 78 to **`scripts/plan-cite-check.sh:101`** under the script's own repair |
| **I-8** `ms`'s advisory recommends a 0644 redirect while P2 ships a 0600 `--out` | **FIXED** | Reproduced: the redirect lands at **644**. Its harm class is the logged one under the ruling, but the finding's actual ask was an **owning phase**, and that is delivered — an out-of-scope bullet plus **F-304**, owned by P3, with the cross-repo byte-parity constraint stated as the reason P2 does not act |
| **M-1** "six single-channel verbs" contradicts §1.4 | **FIXED** | Retitled; the entry now says what is single (the channel `--in` binds to) and points at the cost §2.5 records |
| **M-2** "eleven invocations" over a list of twelve | **FIXED** | Recounted and **re-run**: 12 forms, all exit 0 |
| **M-3** driver column mislabelled; residue wrong | **FIXED** | Re-measured both ways per script: **18 lines, 20 invocations, 13 material, residue 7**. The table gained a column; the draft's own enumeration already summed to 7, so the document had disagreed with itself |
| **M-4** condition 15's scope misses a `src/` test | **FIXED** | Condition 15 now covers the 146 `#[test]`s in `src/` too, and the separator entry names `parse_separator_keyword_and_literal` (`crates/ms-cli/src/format.rs:197`) as a test it must rewrite |
| **M-5** `channel::destination`: right verdict, wrong reason | **FIXED** | Verified against P1's verdict table: `mt` declines `write_block` **and** `WriteBlock` and adopts `destination`, so the draft's discriminator is falsified. The reason is replaced with the real one — what a consumer has to map the non-`File` arms onto — the decline stands, and it is stated that **F-276 gains nothing** from this item |
| **M-6** "the material's own characters" is unsatisfiable | **FIXED** | The leak assertion is now per whole value and per constituent word of 4+ characters, case-insensitively |
| **M-7** exit 4 asserted in §5, absent from §1.1 | **FIXED** | Measured and added to §1.1; the condition now points at it |
| **M-8** the 56 rows were extrapolated from 12 | **FIXED, and it produced the round's most useful number** | The cross-product was **generated and run**: 92 rows, **84 exit 0** (58 silent, 26 with `derive`'s advisory), **0 leak into stderr**. The 8 rows the report predicted were green are green — UPPERCASE `--phrase` on four verbs, in both join forms — so they now assert the **guard's own refusal text**, not a bare non-zero exit, or clap's wordlist error satisfies them forever |

---

## DECLINED, WITH EVIDENCE

Two claims in the report were refused. Both are refusals of **reasoning**; the
substance of each finding was folded.

1. **I-3's "There is no private form."** Too strong. Measured counter, in the
   closest form today's binary supports: `ms encode --phrase - --group-size 0 <
   seed.txt` → rc 0, then `ms derive "$CARD" --passphrase-stdin < pass.txt` → rc
   0, `master_fingerprint: ca2c62d2`. After P2 the same route is
   `ms encode --in seed.txt --out card.ms1` then
   `ms derive --in card.ms1 --passphrase-stdin`. The finding's substance — that
   the residue is named nowhere and §2.5 implies the opposite — is folded in
   full, and the route is now gated on producing the **same fingerprint** rather
   than merely exiting 0.

2. **C-2's "13 `.rs` files under `crates/me-cli/tests/`."** Recounted: **14**.
   The negative it supports is unaffected and was re-derived rather than
   inherited — 33 `Command::new` sites across those 14 files, **0** naming an
   `ms` binary; `ms encode` appears twice in `crates/me-cli/src/`, in the comment
   and the emitted literal; `seed.txt` appears once in the crate. The plan quotes
   **14/33/0**.

---

## NUMBERS RECOMPUTED RATHER THAN CARRIED

| figure | draft | recomputed | how |
| --- | --- | --- | --- |
| argv cross-product | 56 rows, extrapolated | **92 rows, generated and run** | 9 flag channels × 4 spellings × 2 join forms + 5 positional × 4 |
| its baseline | "all 56 at exit 0; 52 silent, 4 warned" | **84 exit 0 — 58 silent, 26 warned; 8 exit 1; 0 of 92 leak** | one harness run, results to a file, counted from the file |
| journey drivers | 18 occurrences, residue 5 | **18 lines, 20 invocations, 13 material, residue 7** | `grep -c` against `grep -o` piped to `wc -l`, per script |
| §1.3 stdin forms | eleven | **twelve, all re-run at exit 0** | one invocation per form, exit code read directly |
| `me-cli` emitted line | a line 24 further down | **`crates/me-cli/src/main.rs:2164`** | `git grep` on the emitted string |
| its comment | **line 2160 of the same file**, four above the literal | same |
| `ROOTS` entry | line 78 of the script, now a comment | **`scripts/plan-cite-check.sh:101`, array opens at line 95** | `grep -n mnemonic-secret` on the script |
| regression-gated entries | three | **two** | the sibling remedy's text exits 4, so it is RED-first |
| `me` test surface | "the assertion the suite already makes" | **14 files, 33 `Command::new`, 0 naming `ms`** | enumerated, then grepped |
| `ms` allowlist words | not stated | **12** (11 verbs + `help`) | `ms --help` |
| closure-vocabulary counts | 138 `CLOSED` / 45 `DONE` | **unchanged, re-measured** | `grep -c` on `design/FOLLOWUPS.md` |

Two negatives in §0 were re-run rather than trusted: `git grep -n 'env::args' --
crates/` and the mode-constant grep both exit 1 with **zero** lines.

---

## THE ONE ORDER CHANGE

The sibling remedy moved to follow the ungrouped stdout. At its old position
**neither** the old advice nor the new advice runs, because grouping is what
`me sysw pack` refuses — and the plan's own rule is that no entry begins until
the previous is green. This is inside §7's ruling rather than a departure from
it: §7's P2 row enumerates `ms`-side content and lists the remedy under P2's
**gate**, not its contents, so it is sequenced by its own dependencies. The
name list in §4 was reordered to match, and the move is stated in the entry's
own cell.

---

## GATES

Each run as its own command from the worktree root, exit code read directly,
never chained with the commit:

```
./scripts/plan-table-check.sh    design/IMPLEMENTATION_PLAN_P2_ms_adopts.md   -> 0
./scripts/plan-cite-check.sh     design/IMPLEMENTATION_PLAN_P2_ms_adopts.md   -> 0
./scripts/plan-stepref-check.sh  design/IMPLEMENTATION_PLAN_P2_ms_adopts.md   -> 0
```

- table: **56 rows checked, 0 malformed**
- cite: **38 / 38 resolved, 0 dangling, 0 ambiguous**
- stepref: **0 step numbers in prose**

`plan-build-gate.sh` does not apply: the plan carries **0** fenced `rust` blocks.

**The cite gate cannot see what is on a line (F-279), so every one of the 38 was
read against the claim beside it.** Two false-negative traps were hit and fixed
during the fold rather than left for a reviewer: an escaped pipe inside a table
header, which the table gate does not check because it only examines rows
**after** the separator; and two journey-driver paths written without their
`design/journeys/` prefix, which the cite gate did catch.

---

## FOLLOW-UPS FILED

All four in `design/FOLLOWUPS.md`, each with a repo, an owning phase and its
measurement.

- **F-301** — `me`'s shipped private-channel remedy advises a pipeline that exits
  4 and writes nothing, and a source comment asserts it is verified. **The live
  half of C-2, on `master` today, and not P2's plan to fix.** Owning phase:
  before P2's sibling-remedy entry. Explicitly **not** the secret-handling class.
- **F-302** — the `=`-joined argv bypass, with the full reproduction. The logged
  class by the ruling; closed inside P2 anyway; states what the 92-row gate does
  not cover, and records two shapes that do **not** exist on `ms` (abbreviated
  long flags exit 64; no material channel has a short alias).
- **F-303** — no one-command private form for `derive` from a phrase plus a
  passphrase; carries the measured two-command route so the deferral is a
  deferral of convenience, not of capability.
- **F-304** — `ms encode`'s advisory recommends a redirect landing at 0644 while
  P2 adds a 0600 `--out`; owning phase P3, with the cross-repo byte-parity pin
  named as the reason.

**Also corrected, though nothing asked for it:** F-285 in `design/FOLLOWUPS.md`
said `encode`, `split` and `repair` are "the three verbs whose stdout IS a
canonical `ms1` or share string". I-5 falsified that for `repair`, and the entry
sits in a file this fold's diff would otherwise not have touched.

---

## WHAT THIS FOLD DID NOT ANSWER

The report's own closing note lists three lenses it did not run, and this fold
closes none of them: a journey walk with the operator; a comprehension lens on
whether an operator can predict what `--in` means per verb (it will mean a
phrase on `encode`/`split`, an `ms1` on five verbs, and a file of shares on
`combine`); and the cross-repo mechanics of the sibling-remedy entry. The third
is now **partly** addressed — the entry states that its test locates `ms` by an
environment variable and skips explicitly, naming the reason, when it is unset —
but how the two repos' CI obtain each other's builds is unresolved and is a
question, not a finding.
