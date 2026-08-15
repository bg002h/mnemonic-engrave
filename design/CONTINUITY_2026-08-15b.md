# Continuity — 2026-08-15b: S2 landed, its review is in flight

Supersedes `CONTINUITY_2026-08-15.md`, which said "S2 is in flight". S2 has
landed — **five commits, unpushed and UNREVIEWED**. Read this one.

## THE WORK QUEUE, in order

1. **S2's mandatory independent review** — dispatched and running when this was
   written. Non-deferrable; the implementer was forbidden to self-review.
   - **If its report exists** at `design/agent-reports/s2-execution-review-2026-08-15.md`:
     read it, persist it in its OWN commit, fold findings, re-review scoped to
     "did the fold fix each finding and introduce no new defect". 0C/0I closes
     the loop — do not loop for reassurance.
   - **If it does not exist:** re-dispatch it. The brief is reproduced in
     "What the S2 review must probe" below.
2. **Then, in parallel — they are genuinely disjoint:**
   - **S3** (`IMPLEMENTATION_PLAN_multisig_build_repair.md` §3, S3). Not before
     S2 closes: both stages edit `gui/multisig_build.go`, and there is a
     MEASURED S2↔S3 interaction — making origins template-aware before S5 would
     strand S3's walk on S2's interim foreign-origin refusal.
   - **The S0b adversarial execution review (opus, read-only).** S0b never got a
     dedicated review — it was implemented by the controller and had only
     incidental scrutiny. It is the foundation every later gate leans on, so a
     defect there makes later stages' green results meaningless. Scope: *does the
     derived census (`oracle/expect.go`, `inputsfile.go`, `CompareCensus`) or the
     needle/NFC gate have a defect that would let a later stage pass wrongly?*
3. **Push** — dispatch a **sonnet** agent as a matter of course; the operator
   ruled 2026-08-15 not to ask first. But **do NOT push S2 until its review
   closes** — the directive is about not asking, not about shipping unreviewed
   work.

**Reserve fable for the pre-hardware S6 gate** — the first irreversible action.
Design-level adversarial review of a landed diff is opus's job.

## State

| repo | branch | head | unpushed |
| --- | --- | --- | --- |
| fork `seedhammer` | `main` | `3ea3ede` | **5 (all of S2)** |
| `mnemonic-engrave` | `master` | `f4e0920` | 2 |
| `mnemonic-key` | `main` | `3462157` | 0 |
| `mnemonic-secret` | `master` | `de593ca` | 0 |
| `mnemonic-toolkit` | `followup/p2wsh-binding-oracle` | `aa5e1ae5` | 0 |

Fork gate at `3ea3ede`: `go test ./...` **51 ok / 0 FAIL / exit 0**, `go vet` 6
`ArtifactDir` (baseline), `gofmt` clean, tinygo flash **1,354,568** (S1 moved it
to 1,349,428; S2 added 5,140). `gui/` is reachable from the firmware, so a move
is expected — a build failure is not.

## Stages

S0 ✅ · **S0b** ✅ (review queued) · **S1** ✅ 0C/0I after two folds · **S2** ✅
landed, review in flight · S3 S4 S5 S6 — not started.

**An operator still cannot build a wallet policy on the device.** S5 is where the
wallet gets engraved; S3–S4 are its runway.

## What S2 landed

- `dcd90a5` — **the duplicate-key check, committed ALONE as S2's first landing.**
  In `assembleBuildPolicy` after the slot set completes, 65-byte cc‖pk equality,
  named modal. RED first through the production path. The ruled basis is now
  MEASURED: A@0 and A@1 share a master fingerprint, so "fingerprint would refuse
  the legitimate multi-account wallet" is a fact, not a prediction.
- `101c8eb` — D-4: `bundleGatherFlow` takes a title; Build passes
  `"Cosigner Keys"`; walk needles 6→7, each still single-site.
- `f712a81` — M-E foreign-origin refusal, comparing **parsed path components**
  (permissive on spelling, strict on value — §0.1 applied).
- `189b173` — §0.1a announcements, three distinct sentences; legacy `sh` states
  that no BIP assigns it a path.
- `3ea3ede` — the typed-seed test (**9 plates from a keyboard-typed seed**, count
  derived not observed), the calibrated raster floor, and the **md1
  byte-identity gate** (6 chunks byte-identical to the pinned primary, oracle
  resolved by binary hash).

## The find worth carrying forward

**An em-dash does not merely fail to draw — it BLANKS ITS WHOLE LINE.** The
raster floor caught it on its first run, on the EXPERIMENTAL warning: 4,973 ink
px against a 5,482 px *blank* frame, 18,563 after the fix. Cross-checked at
2,652 px — the exact figure `raster_test.go` records for **F-151**, so F-151 and
this are the same defect.

That test exists because fable ruled it a standing D-1-class guard that must not
wait for a reproduction. It found one immediately. **~30 sites remain: F-179.**

## What the S2 review must probe

Recorded so it survives a lost dispatch. Rank Critical/Important/Minor; both
block.

1. **The duplicate check is the funds-safety deliverable.** Is it reachable from
   EVERY route into `assembleBuildPolicy`, or only the one the walk drives? Find
   a path that assembles a policy without passing it.
2. **Check ORDER was the implementer's own judgment call** — duplicate before
   foreign-origin, justified by §0.1 clause 2 (invisible harm outranks printed
   harm). Rule on it; construct the input that distinguishes the two orders.
3. **F-179's ~30 unfixed sites** — can any blank a screen carrying a refusal, an
   origin, or a fingerprint?
4. **The raster floor must be able to FAIL** — verify it sits between the blank
   frame and the real one, not below both.
5. **The md1 gate** must resolve the oracle by binary hash, not bare name
   (`md` is a shell alias for `mkdir -p` here; invoke by ABSOLUTE path).
6. **Re-apply mutations yourself** — 8 claimed, all compiled. A mutation that
   does not compile is not a proof.
7. **F-181**: the emulator typed-seed leg is BLOCKED and was deliberately not
   shipped half-built. Verify no half-driver landed; judge whether stopping was
   right.

## Rulings that decide things (plan §0 — read it first)

- **§0.1 THE GOAL and the named tiebreaker.** Permissive on input, expressive on
  output, speak loudly when a common assumption must be made. Rule: **defaults
  for spelling, never for stakes, every default printed** — permissiveness stops
  where a wrong assumption would be **invisible in every artifact**.
- **§0.1a** BIP-48 script-type origins must be spoken; template-aware defaults at
  S5, not earlier.
- **§0.1b** The SH2 **HAS** NFC hardware (soldered, knife-defeatable). Payload +
  keyboard are this phase's primary data entry; NFC is a future pass.
- **§0.2** A claim falsifiable by reading someone else's spec or running their
  binary may not be RULED — only cited and checked.
- **§0.3** The plan is FROZEN from S1 on.
- **F-173** `0..n` · **F-175** S1 recordless on the D-1 arm · **cc‖pk** is the
  ruled duplicate basis · the duplicate check is S2's first landing and **no
  hardware engrave of the Build path until it is in**.
- **F-178's screens are screens of the DEFECT** — never pin them as a good state.
  D-1 itself moved to S6.

## Open follow-ups

**F-179** (em-dash, ~30 sites) · **F-180** (the Go cosigner roster is in a
different order from the emulator payload) · **F-181** (typed-seed emulator leg,
with the cheap fix named) · **F-177** (the `ms` pin lags settled 0.16.0 — batch
the re-pin with S2's oracle extension so the record re-anchors once) · **F-178**
→ S6 · **F-172** → S3 · F-166, F-158, F-160 — none block. **F-176 is WITHDRAWN;
do not re-file it.** In other repos: `parity-smoke-toolkit-version-drift` (red
fixed, guard DORMANT) and `toolkit-mk-codec-0-5-determinism-note`.

## Traps this cycle paid for

- **A negative finding needs the same rigour as a positive one** (F-176 was filed
  on three failed syntaxes; the mechanism existed).
- **A fold is authorship and re-earns the gate** — the fold that fixed
  "tests verify the parts, not the wiring" reproduced that defect inside itself.
- **Never trust a summary counting `test result:` lines** — it reported a red
  suite as "0 failed" twice. Check TRUE exit codes; never judge through a pipe.
- **A comment can outlive its condition and take a gate's justification with it.**
- **`md` is a shell alias for `mkdir -p`.**
- **The push agent's refusal to push a dirty tree caught two uncommitted edits.**
  Keep that strict now that pushing is automatic.
- Both primaries have `enforce_admins: false`, so their pushes BYPASS required
  checks. `ci/staging` is the fix; neither documents it. **Operator decision, not
  yet made.**

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...
    nix develop --command go vet ./...
    nix develop --command gofmt -l ./
    nix develop --command tinygo build -size short -o /dev/null -target pico-plus2 \
      -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller

Emulator: `nix develop --command ./cmd/emu/build.sh`, serve `cmd/emu` on a
**fresh port** (the browser caches `emu.wasm`), then

    import("./walk_build_policy.js").then(w => w.run()).then(r => { window.__w = r });

Artifact gates before any fold: `scripts/plan-cite-gate.sh <artifact>` and
`scripts/fold-propagation-check.sh <artifact> <superseded-pattern>...`.
`FOLLOWUPS.md`'s unresolvable-citation count is a BASELINE, not a target — and
it is whatever this prints, never a remembered number:

    ./scripts/plan-cite-gate.sh design/FOLLOWUPS.md 2>&1 | grep unresolvable

**20 as of 2026-08-15**, all pre-existing cross-repo `.rs` references plus one
ambiguous `checksum.go` (two files match; it wants a repo-relative path). It had
been quoted as "17" in three continuity docs while the real figure drifted —
the same stale-hand-count disease F-179 carried. Compare before/after by running
the gate on `git show HEAD~1:design/FOLLOWUPS.md`, not by trusting the prose.

Push `master` here via `ci/staging` (this repo's `CLAUDE.md`); the fork's `main`
and the primaries push directly.
