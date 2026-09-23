# R3 scoped re-review — the fold of `f449-plan-stage2-r2` into `IMPLEMENTATION_PLAN_f449_stage2_compose.md`

**Verdict: NOT ready for implementation. 0 Critical / 2 Important / 5 Minor / 2 Nit.**

Scope: *did the fold `2855adea..387d4cc4` close R2's five Importants, and did the
fold itself introduce a new Critical or Important?* Not a fresh audit. Reviewed
against `descriptor-mnemonic` main `25acb33c` (`target/debug/md` reports
`md 0.18.0`) and, where Task 2c reaches it, `mnemonic-toolkit` as checked out
(installed `mnemonic 0.103.1`). One throwaway worktree of descriptor-mnemonic
was created in the scratchpad, used to run the vendoring script, and removed;
`git status --porcelain` in descriptor-mnemonic is empty afterwards.

Everything marked **MEASURED** was run on this box. I did not re-derive what
the brief settled (the build gate, plan-api-check, R2 §2, the live Liana gate).

**Headline.** Three of the five are closed: NEW-I-1, NEW-I-4 (at the located
site) and NEW-I-5. Both Importants below are in **Task 2c and Task 5 Step 2,
where the fold wrote new remedies**. That is the same pattern R1 and R2 found:
the new defects are in the fold's own fixes.
- NEW-I-2's ruling (exit 5, "no other repo is touched") rests on a false
  premise. Every way to carry `details` through a failed version check changes
  md-codec's public error or return type. `mnemonic-toolkit` matches that error
  exhaustively and destructures it, and its `mnemonic repair` is one of the
  D26 parity partners.
- NEW-I-3's new mechanical proof fails at baseline with zero edits.

---

## 1. Per-finding verdicts

| R2 finding | verdict | evidence |
| --- | --- | --- |
| **NEW-I-1** `default_value` vs a flag-level refusal | **ADDRESSED** | The ruling is the same in all three places: Step 7 (`Option<String>`, no default, `None => Nums` at the call site), the Step 1 test comment, and Task 1b Step 1 (the refusal is on the flag, plus a flagless exit-0 control under each non-`tr` wrapper). MEASURED: the golden's premise reproduces exactly. Of 24 preset × wrapper cells, **14 exit 0**: 6 under `tr`, 6 under `wsh`, `plain-multisig` under `sh-wsh` and `sh`. The other 10 exit 1. So the new `golden.len() == 14`, 4-wrapper and 6-preset asserts hold. Mutation (c) (`Some("nums") → Liana`) is caught by the inner comparison on the five NUMS `tr` rows. The `main.rs:279` precedent and the `main.rs:287` site both resolve. Wording residue: N-1. |
| **NEW-I-2** `md repair` exit contract undecided | **PARTIAL** | The question R2 asked is ruled (exit 5, no new code). But the ruling's premise, "no other repo is touched", is measurably false (**R3-I-1**). Its assertion set also omits the clean-unsupported-version control (**R3-I-2**). The citations the fold added resolve: SPEC:724 (a v8 row, and v8 IS accepted, `header.rs:36-38`), `chunk.rs:658`, `:666`, and `repair.rs:14-18`. |
| **NEW-I-3** regenerator remedy not executable | **PARTIAL** | The four edits are enumerated, and every script line cited is correct. MEASURED: `NAMES :94-103`, `ACCEPTED_NAMES :104-109`, ok-membership exit `:186-190`, missing-evidence exit `:158-163`, inputs `:65-66`. The fold's new closing proof fails on first execution, and "same commit" spans two repos (**R3-I-2**). The `variant` field is unstated (M-2). |
| **NEW-I-4** Self-Review certifies §4a's JSON bump as shipped | **ADDRESSED** (at the located site) | The ledger at `:725-730` now says "HALF shipped", with compose's side at Task 1b Step 2. The same false fact survives at `:11` and `:822`. Neither site drives an action (Task 7 Step 2 reconciles only stages 3/4/4a/5), so this is Minor M-1, not a reopened Important. |
| **NEW-I-5** no release task | **ADDRESSED** | Task 8 plus a gating Global Constraint. Verified: the `md-cli/Cargo.toml:28` exact pin is `=0.46.0`; the current versions are 0.46.0 / 0.18.0 (`Cargo.lock:489-490`, `:512-513`); stage 1b's `:52` G-3 and `:1067` Task 10 resolve; `compose`/`compose_with` are `pub` (`compose/mod.rs:605`, `:613`); SPEC:861-863 is the correct re-cite in an 863-line file. Residue: M-3 (Cargo.lock), and Task 2c's API change is missing from Step 1/3 (part of R3-I-1). |

---

## 2. NEW Important findings

### R3-I-1 — Task 2c's exit-5 ruling rests on "no other repo is touched", but every implementation of Step 2 changes md-codec's public API, which the toolkit matches exhaustively, and the toolkit's `mnemonic repair` (a D26 partner) diverges on the very card

Task 2c Step 3 justifies exit 5 by saying *"no code is minted and no other repo
is touched"*. It then prescribes *"one line to `repair.rs`'s D26 block
recording that `5` covers this case, so the next reader of the parity contract
does not read it as a divergence."*

**What Step 2 forces.** `decode_with_correction` is `pub` (re-exported at
`md-codec/src/lib.rs:51`) with signature
`Result<(Descriptor, Vec<CorrectionDetail>), Error>` (`chunk.rs:532-534`). On a
failed version check it returns `Err` from `:658` or `:666`, and `md repair`'s
only `Err` arm is `eprintln!; return Ok(2)` with no stdout
(`repair.rs:88-95`). To get the corrected string and the `details` to
`repair.rs`, Step 2 must do one of four things:

1. add an `Error` variant carrying them;
2. add fields to `WireVersionMismatch`;
3. change the `Ok`/return type;
4. add a new function.

The plan prescribes none of them. (Its "fix site … not in `repair.rs`" is also
now false, since Step 3's stdout and exit-5 assertions require a new
`repair.rs` branch.)

**The toolkit consumes this API.** MEASURED in `mnemonic-toolkit`:

- `crates/mnemonic-toolkit/src/error.rs:509` — *"md_codec::Error is NOT
  `#[non_exhaustive]`; match is exhaustive."* `fn md_codec_exit_code`
  (`:520-616`) has no wildcard arm, so shape 1 is an `E0004` there.
- `error.rs:1081` destructures `WireVersionMismatch { got }` without `..`, and
  `:1272` constructs `WireVersionMismatch { got: 99 }`, so shape 2 breaks
  both.
- `repair.rs:1667-1668` destructures `Ok((_descriptor, corrections))` from
  `decode_with_correction`, so shape 3 breaks it.

Only shape 4 is non-breaking, and nothing steers the implementer to it. The
toolkit's pin bump to this stage's tag is scheduled for this cycle (SPEC:861-863).
Task 8 Step 5 files it as "pin bump and a golden refresh" only, and Task 8
Steps 1 and 3 list no md-codec API change from Task 2c.

**The parity note would be false.** MEASURED on the same card:

```
$ mnemonic repair md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg     # v12, 1 correctable error
error: repair: post-correction decode failed: wire-format version mismatch: got 12, expected 4
exit=2
```

After Task 2c, `md repair` on that card exits 5 and `mnemonic repair` exits 2.
The toolkit routes this error through `Err(other) => PostCorrectionDecodeFailed`
(`repair.rs:1736-1739`). The plan says to record this in the D26 block as
"not a divergence". It is one, for identical md1 input, between the two CLIs
the D26 block names as parity partners.

**Counterexample.** The implementer does Step 2 the way the error path
suggests: `Error::UnsupportedVersionCorrected { corrected, details }`, raised
at `:658`/`:666`, and matched in `repair.rs`. All of Task 2c's assertions pass
and descriptor-mnemonic is green. The first toolkit pin bump to md-codec
0.47.0 then fails to compile at `error.rs:520`. The "golden refresh" follow-up
has now become a codec-error-routing decision for `mnemonic repair`. Meanwhile
`repair.rs` carries a comment asserting a parity that the toolkit measurably
does not have. This is R2 NEW-I-2's defect class (a cross-repo reach the plan
denies), moved one layer down from the exit code into the API that produces
it.

**Remedy.**
- (a) Prescribe the non-breaking shape: a new `pub fn` alongside
  `decode_with_correction`, leaving that function and `Error` unchanged.
  Otherwise, name the toolkit break in Task 8 Step 5's follow-up and in both
  CHANGELOG entries.
- (b) Make the D26 line record the divergence with `mnemonic repair`, naming
  the toolkit follow-up that owns convergence, rather than denying it.
- (c) Correct "not in `repair.rs`": both files change.

### R3-I-2 — Task 5 Step 2's new mechanical proof cannot pass: the regenerator stamps the engrave HEAD into every case, and the four edits span two repos that "the same commit" cannot join

The fold closes NEW-I-3 with: *"re-run `./scripts/vendor-liana-evidence.sh
<path-to-mnemonic-engrave>` and assert `git diff --exit-code
crates/md-codec/tests/fixtures/liana/cases.json` comes back clean … the only
evidence that the vector survives."* It also says *"All four edits land **in
the same commit as the vector**."*

MEASURED: the script sets `source_commit=$(git -C "$engrave_repo" rev-parse HEAD)`
(`:75`) and writes it into **every** case (`:204`). The committed corpus
carries `b1eaaee9…` on all 8 cases, while engrave HEAD is `387d4cc4`. I ran
the prescribed proof in a throwaway worktree at `25acb33c` **with no edits at
all**:

```
vendor-liana-evidence: wrote 8 cases … (source 387d4cc4…)
 crates/md-codec/tests/fixtures/liana/cases.json | 16 ++++++++--------
diff-exit=1          # all 16 lines: 8x source_commit b1eaaee9 -> 387d4cc4
```

So the fold's proof fails before the implementer touches anything, for a
reason unrelated to whether the vector survives. It can pass only after
committing a regenerated file, and then it proves nothing beyond the script
being idempotent at one engrave HEAD.

"The same commit" is also impossible. Edits 1–2 (`NAMES`, `ACCEPTED_NAMES`)
live in **descriptor-mnemonic**. Edits 3–4 (the two JSONLs) live in
**mnemonic-engrave** (`:65-66`). That breaks the plan's own "every step must
say which repo" constraint. The ordering this hides is load-bearing:

- If the script runs while the JSONL edits are **uncommitted**, the fifth case
  and the eight re-stamped ones record an engrave `source_commit` that does
  not contain the fifth case's evidence.
- A re-run at the same HEAD then diffs clean, so the proof passes over false
  provenance.
- Nothing checks `source_commit`: md-cli's loader comment says no md-cli test
  touches it (`crates/md-cli/tests/liana_cases.rs:25`).

**Counterexample.** The implementer follows Step 2 literally:
1. Edit the JSONLs in engrave.
2. Edit `NAMES`/`ACCEPTED_NAMES` in descriptor-mnemonic.
3. Hand-add the vector (Step 1).
4. Run the script, then `git diff --exit-code`, which returns **exit 1** on
   nine `source_commit` lines.

The two likely recoveries are both wrong. One commits the regenerated file
with a `source_commit` naming an engrave commit that lacks the evidence. The
other waives the diff as "only provenance", so the gate the fold added is
never actually passed.

**Remedy.** State the order, which spans two repos:
1. Commit the two JSONL records in **mnemonic-engrave** first.
2. Then, in descriptor-mnemonic, edit `NAMES`/`ACCEPTED_NAMES`, run the script
   against that engrave commit, and commit the regenerated `cases.json`. Do
   not hand-add.
3. The proof is: the regenerated file contains the new case, and **every
   non-`source_commit` line** of the eight existing cases is unchanged. For
   example, `git diff -U0 … | grep '^[-+] ' | grep -v source_commit` is empty.

Also say that re-stamping the eight existing cases' `source_commit` is
expected. Step 1's "record its two inferred recovery paths" has no field in
the script's case schema (`:195-205`), so either route that data elsewhere or
state that the regenerator drops it.

---

## 3. Minor

- **M-1 (NEW-I-4 residue).** Two places still carry the false fact the ledger
  fix corrected. `:11` sends the implementer to the recon "first", describing
  it as measuring "that three of SPEC §9's four stage-2 items already shipped
  in 1b". `:822`, in Task 7's motivation, says "three had already shipped in
  1b". Neither drives an action, but together with the recon they are the
  fourth and fifth copies. Two sentences fix it: say "two of four; §4a's JSON
  bump is half shipped".
- **M-2.** Task 5 Step 2 edits 3–4 omit the fields the script actually keys
  on. `load_variant` keeps a record only when `variant == "liana-unspendable-xpub"`
  and indexes it by `name` (`:150-151`). The in-record's `desc` must hold the
  internal key as its **first** xpub plus at least one leaf xpub (`:168-174`),
  and `expected_xpub` is taken from it verbatim, never recomputed. A missing
  `variant` fails loudly (missing evidence), which is why this is Minor.
- **M-3.** Task 8's Files list omits `Cargo.lock`. The workspace members are
  locked at `Cargo.lock:489-490` / `:512-513`, so Step 4's `cargo build
  --locked` fails with "lock file needs to be updated" until it is
  regenerated. That failure is loud. Add `Cargo.lock` to the file list and the
  commit.
- **M-4.** Task 2c's new `repair.rs` branch has no `Descriptor`, but the
  success path's output-class advisory needs `descriptor.is_wallet_policy()`
  (`repair.rs:127-132`), and the L4 fix exists because an unconditional label
  mislabelled watch-only cards. `--json` output for this case is also
  unspecified. The plan should say which advisory to print (or none) and what
  the JSON carries.
- **M-5.** Task 2 Step 3's new "Step 3 requires `ik.is_none()`" suppresses the
  hashlock half of the Liana-shape warning whenever a real key path exists.
  Example: `--path 1of1 --path 1of1,sha256=…,older=100 --unspendable liana`
  prints only Step 4's warning, not that Liana will decline the hashlock leaf.
  Both are advisory, so this is not blocking. Mutual exclusion is only needed
  for the timelock half of the predicate.

## 4. Nit

- **N-1.** Step 9 says mutations (a) and (b) redden the outer comparison "on
  every `tr` row". MEASURED: `simple-timelocked-inheritance` under `tr` has
  `"internal_key_path": 0`, so both mutations are inert on it. They redden 5
  of the 6 `tr` rows. The same holds for (c) on the inner comparison.
- **N-2.** Task 1b Step 1's flagless control, "exits 0 under each of the three
  non-`tr` wrappers", must use `plain-multisig` for `sh` and `sh-wsh`. The
  other five presets exit 1 there (measured above).

---

ready for implementation: no
