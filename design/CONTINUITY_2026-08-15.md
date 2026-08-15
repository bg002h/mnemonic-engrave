# Continuity — 2026-08-15: S1 is GREEN, S2 is in flight

Supersedes `CONTINUITY_2026-08-14g.md`. That one said "S0b is green, next is S1".
S1 is now green after two folds, and **the plan itself changed shape** — a goal
sentence and a decision rule now sit in §0 and decide every refuse-vs-permit
question. Read §0 before anything else.

## Where to start

1. **Check whether S2's implementer landed.** It was dispatched and running when
   this was written. `git -C /scratch/code/shibboleth/seedhammer log --oneline -5`
   and look for `design/agent-reports/s2-implementation-2026-08-15.md`.
   - **If it landed:** dispatch the MANDATORY independent adversarial review over
     the whole S2 diff (opus). Non-deferrable; the implementer was forbidden to
     self-review. Probe the duplicate check hardest — see "the one that matters"
     below.
   - **If it did not:** re-dispatch S2 from `IMPLEMENTATION_PLAN_multisig_build_repair.md`
     §3's S2 section, whose test list was rewritten 2026-08-15 and is authoritative.
2. **Nothing else is blocked.** No open Critical/Important findings anywhere.

**This is implementation against a plan, so work solo and verify inline.**
Orchestration is for design phases. Pushes: dispatch a **sonnet** agent as a
matter of course — the operator ruled 2026-08-15 not to ask first.

## State

All five repos clean and pushed at the time of writing (verify, do not assume).

| repo | branch | head |
| --- | --- | --- |
| fork `seedhammer` | `main` | `ca2e14b` + whatever S2 landed |
| `mnemonic-key` | `main` | `3462157` |
| `mnemonic-secret` | `master` | `de593ca` |
| `mnemonic-toolkit` | `followup/p2wsh-binding-oracle` | `aa5e1ae5` |
| `mnemonic-engrave` | `master` | `7b1707b` + this doc |

Fork gate: `go test ./...` **51 ok / 0 fail**, `go vet` 6 `ArtifactDir` (baseline),
`gofmt` clean, tinygo flash **1,349,428** (S1 moved it from 1,342,468; S2 will
move it again — `gui/` is reachable from the firmware, so a move is expected and
a build failure is not).

## Stages

| stage | state |
| --- | --- |
| S0 | done (before this cycle) |
| **S0b** | done 2026-08-15 — walk driver, single-site needle gate, derived census, oracle byte comparison |
| **S1** | done — payload supplies the whole cosigner set; closed 0C/0I after two folds |
| **S2** | IN FLIGHT |
| S3, S4, S5, S6 | not started |

**Against the actual goal, an operator still cannot build a wallet policy on the
device.** S5 is where the wallet gets engraved; S2–S4 are its runway.

## The rulings that now decide things (plan §0)

- **§0.1 THE GOAL, verbatim, and the named tiebreaker.** "Permissive on input,
  expressive on output, speaking loudly when common assumptions must be made."
  It appeared NOWHERE in the plan or spec until 2026-08-15, which is why choices
  kept resolving to refusal by house habit. The rule: **defaults for spelling,
  never for stakes, and every default is printed** — permissiveness stops exactly
  where a wrong assumption would be **invisible in every artifact**.
- **§0.1a** BIP-48 script-type origins must be SPOKEN. `multisigSharedOrigin()`
  stamps `…/2'` template-blind while BIP-48 assigns `1'` to nested segwit.
  Announce from S2; template-aware defaults at S5, not earlier.
- **§0.1b** The SH2 **HAS** NFC hardware (soldered ST25R3916, knife-defeatable).
  **Payload + keyboard are this phase's primary data entry**; NFC is a future
  pass. This corrected a false "phase-1 hardware has no reader" premise that had
  propagated into six sites — and it STRENGTHENS the zero-NFC gate, because a
  working reader means a walk really can pass by scanning.
- **§0.2** A claim falsifiable by reading someone else's spec or running someone
  else's binary may not be RULED — only cited and checked.
- **§0.3** The plan is FROZEN from S1 on. Discoveries go to FOLLOWUPS and
  continuity; edit the plan only when a gate or ruling changes.
- **F-173** `0..n`; **F-175** S1 recordless on the D-1 arm with the substitute
  named.

## The one that matters for S2

**F-178's "no dead end" screens are screens of the DEFECT.** Machine-checked: the
S1 walk takes payload cards A@0 and A@1 by default, the hand-drive took the self
seed from the payload (masterA), and `deriveAccountXpub(masterA,
multisigSharedOrigin())` is byte-identical to card A@0. `assembleBuildPolicy`
accepted it and returned stub `4c3c96f1` — the stub on F-178's own Policy Review
screen. That wallet is a "2-of-3" masterA can spend alone, every slot `(no fp)`.

So:

- **Do not pin those screens as a good-state guard.**
- The duplicate check is **S2's FIRST landing**, before any S2 work that
  completes an engrave, and **no hardware engrave of the Build path until it is
  in**.
- **Comparison basis is RULED: 65-byte chain code ‖ pubkey.** It fires on the
  delivered collision and PASSES Trace B, which deliberately holds `A·acct0` and
  `A·acct1`. Master fingerprint would refuse that legitimate wallet; base58
  compares metadata the encoder drops. Do not "improve" this.
- The check lives in `assembleBuildPolicy` — the sole md1 producer — after the
  slot set completes. NOT at selection, NOT at review.
- D-1 itself moved to **S6** (hardware; unfalsifiable in the emulator).

## Open follow-ups

- **F-177** — the `ms` pin lags settled ms-cli 0.16.0. Honest and gate-neutral;
  batch the re-pin with S2's oracle extension so the record re-anchors once.
- **F-178** → S6, annotated (see above).
- **F-172** → S3; **F-166**, F-158, F-160 — none block.
- `parity-smoke-toolkit-version-drift` (in `mnemonic-secret`) — the red is fixed,
  the guard is DORMANT. Residual: the BCH port has no live independent
  cross-check, because the only other implementation became this one.
- `toolkit-mk-codec-0-5-determinism-note` (in `mnemonic-key`) — blocked on a
  toolkit dependency bump.
- **F-176 is WITHDRAWN** — `md encode` authors per-key origins today via inline
  placeholder origins. Do not re-file it.

## Traps this cycle paid for

- **A negative finding needs the same rigour as a positive one.** F-176 was filed
  because a reviewer tried three syntaxes, all failed, and generalised. The
  mechanism existed. My own verification nearly repeated the error — `path_decl`
  is nested under `descriptor`, so a top-level probe returns `null`.
- **A fold is authorship and re-earns the gate.** The fold that fixed "tests
  verify the parts, not the wiring" reproduced that exact defect inside itself:
  mutating a function's body died, deleting its only CALL survived the suite.
- **Never trust a summary that counts `test result:` lines** — a `FAILED` line
  can be mis-summed to zero. It reported a red suite as "0 failed" twice.
  Check TRUE exit codes; never judge a command through a pipe.
- **A comment can outlive its condition and take a gate's justification with it**
  (the "no reader" premise).
- **`md` is a shell alias for `mkdir -p`** on this machine. Harmless from Go;
  fatal to a hand-run gate command.
- **The push agent's refusal to push a dirty tree caught two of my own
  uncommitted edits.** Keep that rule strict now that pushing is automatic.
- Both primaries have `enforce_admins: false`, so their pushes BYPASS required
  checks. `mnemonic-engrave`'s `ci/staging` ritual is the pattern that fixes it;
  neither primary documents one. **Operator decision, not yet made.**

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
`FOLLOWUPS.md`'s **17** unresolvable citations are the baseline, not a regression.

Push `master` here via `ci/staging` (see this repo's `CLAUDE.md`); the fork's
`main` and the primaries push directly.
