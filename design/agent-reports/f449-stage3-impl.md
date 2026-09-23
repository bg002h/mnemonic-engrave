# F-449 stage 3 — implementer report

Plan: `design/IMPLEMENTATION_PLAN_f449_stage3_go_port.md` (GREEN, R0 0C/0I), R0 folds applied.
Implementer: single agent (opus). Date: 2026-09-23. Nothing pushed, tagged, merged into main/master or flashed (controller brief).

## Standing rulings

- **Ruling: no merge / no push of any branch — the controller brief forbids it; the plan's Task 1 Step 10 and Task 7 Step 6 merges/pushes are NOT done. The Go side pins the dm BRANCH commit (Task 1's commit on `f449-stage3-vectors`) instead of a `main` merge SHA — cost if wrong: when the controller merges dm with `--no-ff`, the fork's pin names the branch commit, which is the merge's second parent and still reachable from main; only the `md/bits.go` doc text would want the merge SHA instead, a one-line edit.**
- Vendor scripts are pointed at the dm WORKTREE (`/scratch/code/shibboleth/dm-worktrees/f449-stage3`), not the dm main checkout.

## Task 1 — descriptor-mnemonic (`f449-stage3-vectors`)

Commit `430ea478` "vectors: three wire-kind-1 vectors for the Go port (F-449 stage 3)".

- `md vectors`: exactly 14 untracked files, no `M`. Measured values equal the plan's: `liana_taproot` = `md1gzfdsssj5qqcreqygvtam2lm5fenw4v` / `4092d84212a00181e4044300`; kofn ids `116659077988624006ef630ddf96c982` / `f99cc42e1ff68bae546c4d1070e5963f` / `cb913ed2f6ca08903f67a183aae088d5`.
- RED run before the harness fixes: `1536 tests run: 1532 passed, 4 failed, 4 skipped` — exactly the four predicted (two json_snapshots, skeleton_key_conformance, wire_version_8).
- R0 m1 folded: polarity test reads `[06 00]@8`→NumsPoint, `[07 00]@8`→Liana, `[06]@4`→NumsPoint, and a Slot golden `[04 80]` (kiw 2, Slot(1)) @8, read AND write.
- Gate: `scripts/phase-gate.sh` **exit 0**; nextest `1536 tests run: 1536 passed, 4 skipped`.
- Mutations (applied → red, reverted + touched → green):
  - X1 Rust twin (`if r.read_bits(1)? != 0 || true`): 636/637, red = `the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped` ("nums at v8: golden [06, 00] read back wrong"). The ONLY test that catches it.
  - X2 Rust twin (kind bit read for Slot at v8): same test red ("slot 1 at v8 (no kind bit): golden [04, 80] read back wrong" — read Slot(2)).
  - Step 9 (`want == xkey` → `true` in the recogniser): **survives**, 2/2 green, as the plan predicts → F-655.

## Task 2 — fork baseline (`f449-stage3` at 7b6f2fb)

All measured equal to the plan: md 167 tests, `go test ./md/` ok, gui `RESULT: ok -- all 1376 tests ran across 24 shards`, gofmt = the five-file set, `go vet ./md/` clean, `go vet ./gui/` 2 ArtifactDir. Fork commits sign automatically (ssh, `commit.gpgsign=true`), author Brian Goss; recent history carries no DCO trailer, so none added.

## Task 3 — fork `1026d6d` (three-state internal key, zero wire change)

Mechanical sites as the plan lists. Gate: gofmt baseline only; vet md clean; build ok; md ok 167; gui `-run 'TestEveryKeyedVectorReachesAnAddress|TestTaprootScriptPathMatchesRust|Emit|Pkh'` ok (59 RUN, 0 FAIL). R0 n1 folded: the Liana skip arm in `gui/taproot_script_path_test.go` is commented as future-proofing.

## Task 4 — fork `8aa4371` (SPEC §2 recipe)

`vendor-liana-cases.sh` run against the dm worktree (pin commit `430ea478`, clean). RED first (`undefined: lianaUnspendableKey`), then md 169, both tests PASS (9/9 Liana goldens).

## Task 5 — fork `03c4082` (wire version 8)

- Vendoring: `vendored 260 files, 53 vectors, primary 430ea478d89c`; 14 new + provenance modified, nothing else.
- Step 2 RED (old decoder on the new plates): exactly the five predicted tests. Also measured: the OLD encoder silently wrote kind 1 as the NUMS twin (`2092d84212a00181c80886`). Lines pasted in the commit.
- R0 m1 folded: `TestKindBitPolarityIsPinnedOnTheWire` reads each golden back via `readNode`, plus a Slot-at-v8 row `[04 80]` (kiw 2, Slot 1), identical to the Rust rows.
- R0 m5 confirmed: 10 recursive `readNodeDepth` sites.
- Gate: gofmt baseline; vet md clean; build ok; md **179** (179 PASS); gui address/taproot gates ok, kind-1 address "none", taproot gate skips kind 1 at `TapLeavesChunks` (n1 confirmed).
- Mutations: X1 (`if kind || true`) red → `TestKindBitPolarityIsPinnedOnTheWire` only ("nums at v8: golden 0600 read back as ik:2"); X2 (kind bit read for Slot at v8) red → same test only ("slot 1 … read back as keyIndex:2"). Both reverted + touched → md ok.

## Task 6 — fork `e2b4c6f` (SPEC §6a on three surfaces)

RED first (`undefined: gatherUnsupportedVersion`). R0 m3 folded: the single-card half asserts `uiContains(all, "Keys: 3")`. The four tests RUN and PASS (-v). gui sharded: `RESULT: ok -- all 1380 tests ran across 24 shards`.

## Task 7 — fork `43294c6` (provenance pin, whole gate, size, mutations)

- `md/bits.go` pins md-codec 0.47.0 (cf35d61a) and the kind-1 vectors at dm `430ea478`; no `<TASK-1` placeholder remains. README section added.
- Whole gate: gofmt diff vs the five-file baseline empty; vet md clean, gui only the 2 ArtifactDir; non-gui 55 ok / 0 fail; gui 1380 all ran ok; md 179.
- Size (from the worktree's flake): baseline 7b6f2fb **1,655,388 / 63,336**; branch **1,658,284 / 63,352** (+2,896 / +16) — identical to the plan's measurement.
- Mutation pass: all 23 rows caught (M1, M1b, M2a, M2b, M3, M4, M5a, M5b, M6, M7, M8, M9, M10, X1, X2, G1–G7), plus an extra **m3x** (the single-card v8 flow draws nothing) caught ONLY by R0 m3's new positive. Per-row reds are in the commit message; raw output `/tmp/claude-1000/f449s3-mut/{md,gui}-mut.txt`. M8 reds 13 tests (plan said 12); every other count equals the plan's.
- Not merged / not pushed (brief).

## Task 8 — mnemonic-engrave (`f449-stage3-records`)

- `699286b9` followups: F-643 ruling appended, re-owned to the next dm release after F-449 stage 2, still OPEN; F-654 (stage 4) and F-655 (next dm release) filed. IDs re-grepped (R0 n2): free on this branch and on engrave master `e2b21905`. `scripts/followups-status.sh`: OK.
- `3e79f538` spec sweep: §3c, §3e, §6a, §7a.1, §7a.3, §8.5, §8.9's stage-3 row and §9's stage-3 status. Every new Go citation was resolved and printed at fork `43294c6` (the output is in the commit message). R0 m4 recorded. R0 m2 recorded **after measuring it** with a throwaway `gatheredDescriptorFlow` probe (since deleted): kind 1 shows keys only and no `Policy id:` line, while its kind-0 twin shows `Policy id: c5788d70…`.
- `607a9429` plan: Review Focus 2 and no-gate bullet 6 corrected per R0 m2 (text only).

## End state

| repo | branch | HEAD | merges clean onto |
| --- | --- | --- | --- |
| descriptor-mnemonic | `f449-stage3-vectors` | `430ea478` | main `cf35d61a` |
| seedhammer fork | `f449-stage3` | `43294c6` | main `7b6f2fb` |
| mnemonic-engrave | `f449-stage3-records` | `607a9429` | master `e2b21905` (checked with merge-tree) |

Tests: dm nextest 1536/1536 (4 skipped), phase-gate exit 0. Fork: md 179, gui 1380 all ran ok, non-gui 55 ok, gofmt = baseline, vet = baseline. 23 of 23 mutations caught, plus m3x. The one expected survivor is the Rust recogniser, filed as F-655.

## Concerns for the controller

1. **Landing is yours, and order matters.** Merge and push dm first. The fork's provenance pin (`md/testdata/*.provenance.json`, `md/bits.go`) names `430ea478`, which a `--no-ff` merge keeps reachable from dm main. A squash or rebase would orphan it and force a re-vendor.
2. The pin names the dm **branch** commit rather than a main merge SHA (see the ruling at the top). If you want the merge SHA in `md/bits.go`, it is a one-line edit. The vendored bytes are identical either way.
3. The records branch base (`6d4c2a6d`) is behind engrave master (`e2b21905`, F-642 closed). merge-tree reports no conflict.
4. The spec's §9 status says the fork is "not yet merged at the time of writing". Update it to the fork merge SHA when you land.
5. Minor: M8 reddens 13 tests where the plan said 12. Every other count equals the plan's.
