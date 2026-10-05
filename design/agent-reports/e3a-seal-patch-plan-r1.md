# R0 round 2 (mechanical fold check): IMPLEMENTATION_PLAN_e3a_picotool_seal_patch.md (draft 2)

*2026-10-05. Repo `mnemonic-engrave` at `6ed76db`. Scope: did the fold fix SP-I1, SP-I2, SP-M1..M6,
N1..N4 as asked, and did it add a contradiction or unsatisfiable gate. Settled facts (patch correct,
nix override works, `info -a` on a file verifies) were not re-derived. Only this report was written.*

## 1. Finding-by-finding

| id | verdict | note |
|---|---|---|
| SP-I1 | FIXED | Job id `picotool-probe` verified (`release.yml:270`, display name `picotool argv probe`). It has no `if:`, so adding it to `assemble.needs` (`:442`) cannot skip assemble, and it runs on docs-only pushes, so making it a required check cannot leave that context pending. Fallback (green run linked in the PR) is stated. |
| SP-I2 | FIXED, see SP2-M1 | Hold wording is specified in PICOTOOL_PIN, RUNBOOK:85-88 and F-701. |
| SP-M1 | FIXED, see SP2-M2 | Check added to `sign-firmware.sh`; bench R0 names the Clear entry. |
| SP-M2 | FIXED | Explicit `-o pt-patched` / `-o pt-stock`, store paths asserted different. |
| SP-M3 | FIXED | Tamper control added; plan honestly says hunk B stays on the mutation run (matches R0). |
| SP-M4 | FIXED | `develop@ba3df40` lines and affected paths listed. |
| SP-M5 | FIXED | Patched store path recorded in PICOTOOL_PIN (input-addressed, so CI and Brian's box agree). |
| SP-M6 | FIXED | `design/patches/` to `nix/patches/` for FOLLOWUPS and PICOTOOL_PIN; INV left verbatim. `git grep` confirms exactly those files plus the plan and the R0 report still name the old path. |
| N1 | FIXED | Anchored, `verified` only, notes the double print. |
| N2 | FIXED | R4 scope stated precisely. |
| N3 | FIXED | Provenance list in fixture README. |
| N4 | FIXED | shellcheck list entry added in section 3. |

## 2. Checks the brief asked for (executed)

**Clear-entry check vs R's old e2e and the real signing flow.**
- R calls `sign-firmware.sh` through `sign_image` (`pico2-bootkey-rehearsal.sh:430`) on the TinyGo
  blinky (lines 902, 1095), which `build_blinky` produces unsealed. `sign-firmware.sh` step 1 therefore
  seals it with `--clear`, so the load map has the Clear entry on 2.2.0-a4 and patched 2.3.1 alike
  (INV section 3 and 5). R's old e2e still passes. Phase 3b's unsigned image never goes through it.
- Real firmware (`sh2-flash:182`, R phase 5b, line 1039): the fork's build seals with
  `picotool seal --clear` (INV: real firmware shows `Clear 0x20000000->0x20082000` as entry 0 and
  `Load` as entry 1), so step 1 sees a SIGNATURE section and skips sealing. `picosign sign -clear`
  (`cmd/picosign/main.go:30`, "zero public key and signature") and the later `sign` calls touch only
  pubkey and signature, never the load map. The Clear entry therefore survives to step 7. INV's
  picosign table confirms the load map is untouched (re-hash verifies after picosign).
- Real `info -a` output format (run on the stock-compatible binary in the scratchpad):
  ` load map entry 0:    Load 0x10000000->0x10005c00`, i.e. one leading space, key, then 4 spaces
  before the type. The plan's literal `load map entry 0:    Clear 0x20000000->0x20082000` matches that
  spacing, but see SP2-M2.

**Tamper control on a UF2.** Executed on a sealed 93-block blinky UF2 (47,616 bytes): block 0 has
target address `0x10000000` at offset 12 and payload at offset 32. Flipping byte 32 of block 0, a byte
of block 16 (`0x10001000`), or a byte in the `0x10005c00` metadata block each turned both `signature:`
lines from `verified` to `incorrect`. So the control can work as written (flash payload blocks are in
the hashed range; the first block is the flash base, not an absolute/E10 block). The recipe is not
spelled out in the plan, see SP2-N1.

## 3. Findings

### SP2-M1 (Minor): the hold wording and the RUNBOOK's "either shell" sentence will disagree
The plan's hold says real SeedHammer firmware is signed "only from the fork's shell". RUNBOOK:83-84
(unchanged context around the replaced note) and R both say the SH2 steps work from either shell, and
R phase 5b / `sh2-flash` sign already-sealed fork firmware from `.#otp`. For already-sealed firmware
the new picotool never seals (step 1 skips; only `info -a` runs), so the practical risk is small, but
the operator now gets two instructions. Fold: state in the same edit that the hold concerns images
`sign-firmware.sh` would *seal* (no SIGNATURE section yet), and amend the "either shell" sentence to
match. The SP-M1 check in `sign-firmware.sh` also backstops this, which the hold text could say.

### SP2-M2 (Minor): the new Clear check is brittle and its refusal path is never executed by any gate
1. The plan fixes the literal text with four spaces. Make the check a whitespace-tolerant regex
   against the captured `INFO` (here-string per F-695), e.g. `load map entry [0-9]+: +Clear
   0x20000000->0x20082000`, and say whether position 0 is required. The 2.2.0-a4 print path is the
   same code but was measured only by INV's collapsed quoting; the bench R0 run on the fork shell is
   the first real execution there.
2. Per the repo rule, "a gate that has never executed is a hypothesis". The positive branch runs in R,
   but nothing runs the *refusal* branch (an image sealed without `--clear`, or already carrying a
   load map). Cheapest fold: have `seal-clear-test.sh` (it already seals without `--clear`) feed that
   output to a small check function shared with `sign-firmware.sh`, or add one case that runs the
   step-7 assertion on the no-`--clear` image and requires refusal. Without this the plan's own SP-M1
   fix is untested.

### SP2-N1 (Nit): tamper recipe and `--expect-fail` interaction unstated
Specify the flip (for example block 0, file offset 32, XOR 0x01, i.e. the first payload byte at the
`0x10000000` target address) so the implementer does not pick an unhashed trailing byte or a
non-flash block, and say that the tamper step is skipped under `--expect-fail` (that run exits at 248
before there is a sealed file).

## 4. Contradictions / unsatisfiable gates
None found. `picotool-probe` is the right id, adding it to `needs` is satisfiable, the required-check
fallback is explicit, and the Clear check passes for every flow in the repo that completes today.

VERDICT: 0C / 0I / 2M / 1N
