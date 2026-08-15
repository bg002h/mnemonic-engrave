# Continuity — 2026-08-14f, build S0b

Supersedes `CONTINUITY_2026-08-14e.md`, which closed S0 and recorded the
walk-gate audit. Both decisions 14e left open are **RULED and folded**; nothing
in this cycle is waiting on an answer. Read this one.

## Where to start

**Implement S0b. The plan section is `IMPLEMENTATION_PLAN_multisig_build_repair.md`
§3 "S0b — the walk scaffolding every later gate leans on" (line 524).** Read it
there rather than from a summary here — duplicating a spec into a continuity doc
is how the two drift, which is this cycle's own most-repeated lesson.

Three mechanisms, all exercised against S0's committed record, plus a gate that
requires each to have been **seen to go red**. In one line each:

1. A build-flow driver reaching the Build-policy gather via `Engrave Multisig`,
   asserting a screen with **exactly one production site**, and asserting
   `shNFC.present` was called **zero** times (F-169, F-174).
2. The census **derived from the recorded input tuple**, replacing `plates = 6`
   (F-170).
3. The **oracle comparison** — invoke the pinned `md`/`mk`/`ms` and compare
   engraved strings byte for byte (F-171). Nothing invokes them today.

**Not S0b:** the five per-stage walk scripts. The build flow's tail cannot be
walked before the code that makes it walkable exists.

**This is implementation against a plan, so work solo and verify inline** —
project convention, and the phase table in the global preferences. Orchestration
is for the design phases, which are done.

## The two rulings, binding

**`0..n`** (operator, 2026-08-14). The payload may carry **zero to `n`** cosigner
cards; **no stage may assume `n-1`**. The exact-count check belongs to the
*assembled* set, never the feed. Folded into §1, S1's implementation note, and
S1's tests 6/7/8. The upper bound is `n` and not a typo: a card for every slot
includes one that may be the operator's own — S4's `both` case, already present
in the delivered payload as card `A@0` + the lone `ClassMnemonic`, both master A.

**S0b exists** (operator, 2026-08-14), answering
`design/agent-reports/s1-walk-gate-judgement-review.md`. Option 3 — weaken the
clause to "by test" — was never the plan's to take: SPEC §4.5 is REQUIRED, quotes
the operator verbatim, and says *"A green unit suite is explicitly NOT
sufficient"* (`SPEC_multisig_build_repair.md:447`, R-4 at `:746`).

## State

Both repos **clean and pushed**. `master` went through the `ci/staging` ritual
every time (last: run `31860977550`, green, no bypass message).

| | |
| --- | --- |
| fork `main` | `88d43c7` |
| `mnemonic-engrave` `master` | `4fa0e12` |

## Why S0b exists, in one paragraph

`cmd/emu/walk_trace_a.js` selects `LoadPayload` and `EngraveBundle`;
`engraveBundle` dispatches to `bundleFlow` (`gui/gui.go:1816`). Every stage from
S1 on edits `buildMultisigPolicyFlow` (`gui/multisig_build.go:39`), which sits
behind `Engrave Multisig → Build policy` and is never entered. So five gates
reading "by test and by emulator walk" could not execute the flow they name. The
walk also asserts a plate **count**, not a byte comparison, and nothing anywhere
invokes the pinned oracles. Full report: `design/RECON_S1_S6_walk_gates.md`;
independent review: `design/agent-reports/s1-walk-gate-judgement-review.md`.

**One thing the audit did NOT invalidate:** S2 edits
`layoutTitle(…, "Engrave Bundle")` at `gui/bundle_flow.go:155`, inside the
*shared* gatherer that the existing walk renders. That walk is therefore a real
regression check for D-4's five call sites — keep it, do not replace it.

## Open follow-ups

- **F-169, F-170, F-171, F-174** → **S0b**, all four, **gating**. This is the
  stage's work list.
- **F-170** additionally → **S3**: the plan's preamble had exempted S3 from the
  derived census on a false premise (`bundleEngrave` is
  `gui/multisig_build.go:168`, the restore doc `:191` — after it).
- **F-172** → S3. The walk must pick "Full policy md1" or the restore-doc gate
  has nothing to read (`gui/multisig_build.go:185` skips it on template).
- **F-173** → RULED (`0..n`); **S1** owns building to it.
- **F-168** → S0, folded. Its "one walk per page load" half is a standing note.
- **F-166** → its own cycle. The fork's md decoder refuses a pathless origin.
- F-158 premise STALE; F-160 census gap. Neither blocks.

## Traps — all still live, all paid for

- **A walk outliving ~1800 s dies in the MCP call, not the emulator.** Launch
  fire-and-forget — `run(...).then(r => { window.__walk = r })` — and poll
  `window.__walk`. At the default pace 2048 a six-plate walk is ~174 s.
- **One walk per page load.** The census is cumulative for the session with no
  reset (`cmd/emu/engraved.go`). It fails closed today only because two checks
  are strict equalities; **do not relax either to `>=`.**
- **Only one browser session exists.** Two agents driving it collide.
- **The browser caches `emu.wasm`** and a cache-buster on `index.html` does not
  help. Serve on a fresh port and check a symbol only the new build has.
- **Playwright's `browser_evaluate` `filename` writes the JSON-ESCAPED string**,
  not the value. It looks right and is double-encoded. Hash both ends.
- **The cite gate SILENTLY SKIPS comma-list citations.** `` `f.go:359,386` ``
  never matches; the pattern needs the backtick right after the number. Write
  them separately.
- **Do not describe a flow from its doc comment.** `gui/multisig_build.go:67`
  says "(3) TYPED-ONLY self seed"; the call beneath it is `seedEntryFlow`, which
  puts `FROM PAYLOAD` first. It is one of the nine stale `TYPED-ONLY` comments S3
  deletes.
- **A build failure is not a mutation proof.** Make the mutant compile
  (`if clean == !clean {`), and prove the mutated line RAN.
- **A floor-only bound asserts "enough", never "usable".** That is how F-173 got
  past a gate that had itself been mutation-proved —
  `TestSyswCardsPayloadCoversEveryStagesWalk` has `len(mdmk) < 8` and no ceiling.
- **Never truncate a digest in a mismatch message.** Print both in full.
- **`shScreen()` inks no spaces** — match `"Engravingplate"`.
- **`git rev-parse HEAD` does not identify a binary built elsewhere.**
  `oracle.ByCheckout` refuses a binary outside its checkout; installed binaries
  pin by SHA-256.

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...      # 51 ok, 0 fail
    nix develop --command go vet ./...       # 6 ArtifactDir = the baseline
    nix develop --command gofmt -l ./

Device build — **1,342,468** flash. Nothing in S0b reaches `./cmd/controller`,
so this must not move:

    nix develop --command tinygo build -size short -o /dev/null \
      -target pico-plus2 -stack-size 16kb -gc precise -opt 2 \
      -scheduler tasks ./cmd/controller

Emulator, then the gate record:

    nix develop --command ./cmd/emu/build.sh
    # serve cmd/emu on a FRESH port, open index.html
    import("./walk_trace_a.js").then(w => w.run()).then(r => { window.__walk = r });

    go run ./cmd/gaterecord -stage <S> -walk <saved>.json \
      -inputs oracle/gaterecords/<S>.inputs.json -base <S>-<trace>

Artifact gates, run before any fold: `scripts/plan-cite-gate.sh <artifact>` and
`scripts/fold-propagation-check.sh <artifact> <superseded-pattern>...`.
`FOLLOWUPS.md` reports **17** unresolvable citations — all pre-existing `.rs`
cross-repo references; that count is the baseline, not a regression.

Push `master` via `ci/staging` (see the repo's `CLAUDE.md`); the fork's `main`
pushes directly.
