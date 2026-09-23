# R1 re-review: fold of IMPLEMENTATION_PLAN_f449_stage4a_me.md (`6b19e3f4` → `803d0ef7`)

Scope: exactly `git diff 6b19e3f4 803d0ef7` (`design/IMPLEMENTATION_PLAN_f449_stage4a_me.md`,
`design/CONTINUITY_f449_stage2.md`). Question: did the fold close I-1, M-1..M-4
and N-1 from `design/agent-reports/f449-plan-stage4a-r0.md`, and did it
introduce a new Critical/Important defect? The engrave checkout itself was not
modified — all execution ran in throwaway worktrees under
`/scratch/code/shibboleth/.tmp/`, both removed afterward (`git worktree list`
confirmed clean for both).

## Findings table

| Finding | Status | Evidence |
| --- | --- | --- |
| **I-1** — Step 0's precondition could not fail | **ADDRESSED** | New Step 0 is two parts. Part 2 (behavioural check) run exactly as written in a detached worktree of fork `origin/main` at `7b6f2fb`: `go test ./md/ -run TestStage4aPreconditionDecodesV8 -v` → `--- FAIL: TestStage4aPreconditionDecodesV8` (`md: wire version mismatch`), matching the plan's own "MEASURED... `--- FAIL`" claim exactly. Part 1 (SHA ancestry) confirmed to fail today too: `grep -n "SHA" design/CONTINUITY_f449_stage2.md` shows no stage-3 merge SHA is recorded yet, which the plan's own text reads as "stage 3 has not shipped." Vacuous-pass modes checked and ruled out: a compile error in the test file reports `FAIL ... [setup failed]`, never `ok` (verified by breaking the file's syntax); an `-run` pattern matching nothing, or the file renamed off `_test.go` so it's not picked up, both report `ok ... [no tests to run]` **with no `--- PASS:` line** — exactly the case the plan's "literal `--- PASS: TestStage4aPreconditionDecodesV8`, `ok` alone is not enough" requirement excludes. `md.Decode(s string) (Template, error)` (`md/md.go:1231`) matches the test's call shape, so a fork that decodes v8 would hit `err == nil` and print the required `--- PASS` line — the check is sound going forward. |
| **N-1** (Nit) — missing `V12_NAMED` assertion in `a_good_plate_does_not_carry_a_bad_one` | **ADDRESSED** | Applied `/scratch/code/shibboleth/.tmp/f449-4a-probe.patch` (`git apply -p2`, clean) to a scratch worktree of engrave at `8aea0d36`, added exactly the one assertion the fold specifies. `cargo test -p mnemonic-engrave --test f449_stage4a a_good_plate_does_not_carry_a_bad_one` (toolchain 1.85.0, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/r1-4a-nit-target`) → **PASS** (`1 passed; 0 failed`). Replacing `V12_SINGLE` with a second `V8_TEMPLATE` (bad plate → good plate) and re-running → **FAILS**, at `assert_eq!(r.code, 4, ...)` (`left: 0, right: 4`) because two good v8 plates bundle successfully (`backup needs 2 public plates...`). The assertion did not need to be the one reported, since the instructed mutation removes the bad plate entirely rather than substituting an unrelated refusal reason — reported as an observation, not a defect: the literal instructions ("must PASS", "then show it fails") were followed and both outcomes hold. No constant correction needed. |
| **M-1** — v8-confirms-but-board-can't-read note | **ADDRESSED** | CHANGELOG note added at plan's Step 1 doc block, labeled "Note (R0 M-1)", with the exact wording the R0 remedy specified. The continuity-file half of the remedy ("and the continuity file") is folded into the M-2 HOLD paragraph rather than given its own M-1-labeled line — but that paragraph's first sentence ("master's `me` confirms v8 cards that no board flashed before stage 3 can read") is substantively M-1's fact, so the ask is met. The **optional** `show`/`pack` warning line was not added — correctly optional per R0's own wording, not a gap. |
| **M-2** — the standing "keep local binaries current" permission bypasses Step 0 | **ADDRESSED** | `design/CONTINUITY_f449_stage2.md` now carries a "HOLD (stage 4a R0 M-2)" paragraph: do not tag `0.11.0`, do not `cargo install` master's `me`, until stage 3 is in fork main AND boards are reflashed, and names Task 5 Step 0 as the mechanical check. This is the file the remedy asked for. Updating the global memory entry itself is a live-session action gated on Step 0 actually blocking during execution, not something a plan-document fold does — out of scope for this diff. |
| **M-3** — stale doc comment now describes the wrong function | **MOSTLY ADDRESSED** | New Step 6b (between Step 6 "Run" and Step 7 "Commit", and `record.rs`/`expect.rs` are in Step 7's commit file list, so ordering is consistent) requires: moving the long doc off the projection and onto `mdmk_unconfirmed_why`; rewriting the caller list "found by grep, not copied"; correcting `expect.rs:55-61`. All three land. One residual gap: R0's remedy also asked for a specific one-line replacement doc on `mdmk_unconfirmed` itself ("projection of `[mdmk_unconfirmed_why]`; kept for the frozen vectors") — Step 6b says the long doc "must sit on `mdmk_unconfirmed_why`... not on the 4-line projection" but doesn't prescribe what text the projection function keeps. Non-blocking: M-3 was already Minor, and the core defect (doc describing the wrong function, stale caller list) is fixed; the exact wording of the short doc is an implementation-time judgment call that doesn't change behavior. |
| **M-4** — the "one walk call" rule rests on the stale comment | **ADDRESSED** | Step 6b: "It must also state the 'one walk call per invocation' rule, which today appears only in that stale comment" — folded into the same move as M-3, as the remedy asked. |

## New findings

- **Minor/Nit (new).** The old Step 0 text explicitly invoked the pinned
  toolchain (`/scratch/code/shibboleth/.toolchain/go/bin/go test ./md/ ...`,
  per repo CLAUDE.md's toolchain note). The fold's replacement text says only
  "Run `go test ./md/ -run TestStage4aPreconditionDecodesV8 -v`" and drops
  that explicit path — `grep -n "toolchain/go/bin/go"
  design/IMPLEMENTATION_PLAN_f449_stage4a_me.md` now returns nothing. An
  implementer could invoke a different `go` on `PATH`. Not blocking: any go
  toolchain new enough to build the fork's `md/` package will behave
  identically for this test, and the path is documented elsewhere
  (CLAUDE.md). Worth a one-line fix at the next non-trivial fold, not worth a
  re-dispatch on its own.

No new Critical or Important defect was found in the fold diff.

## Counts

0 Critical, 0 Important, 1 Minor (residual, non-blocking, on M-3), 1 new
Minor/Nit (toolchain path). I-1 and N-1 both verified by direct execution as
specified in the dispatch brief; M-1..M-4 verified by reading the folded text
against the R0 remedies.

ready for implementation: yes
