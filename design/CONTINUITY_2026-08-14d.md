# Continuity — 2026-08-14d, S0's deliverables are all done and its gate is not

Supersedes `CONTINUITY_2026-08-14c.md`, whose "Where to start" — write S0
deliverable 6's three tests — is **done**, along with D7, D8, D5 and D4. Read
this one. 14c's dangling pace-1 conditional is also resolved; see **Traps**.

## Where to start

**Two things, in this order. The first has an agreed design; build it.**

### 1. The fail-closed gate record — the last clause of S0's gate

S0's gate requires the harness to print resolved oracle commits and the full
input tuple **into every gate record**. The `oracle` package now resolves the
real toolchain (`3dd64fb`), but **nothing emits a record**, so this clause is
unmet. Measured: importers of `seedhammer.com/oracle` outside its own package
= 0.

The obvious build — a command the operator runs beside a walk — was **rejected
on 2026-08-14 after the operator named the criterion: "we don't want to be
surprised later by code that doesn't work as expected."** A plain command fails
that twice: it is *optional*, so a gate can pass with no record and absence is
silent; and it is *unbound*, so a record from run A can sit beside run B's
artifacts and nothing notices.

**Build it with these three properties instead. They are the design; do not
re-litigate them, and do not drop one for convenience.**

1. **It refuses to emit without a walk result.** No run, no record.
2. **The record embeds the walk's `digests` and `census`,** binding it to a
   specific run.
3. **A Go test verifies every record on disk** — oracle commits still resolve,
   embedded digests still match the artifacts beside it — **and S0's gate
   requires a record to exist.** Absence must be a failure, never a silence.

The enabling fact, already checked: `run()` in `cmd/emu/walk_trace_a.js:251`
returns `{pace, paceOverridden, elapsedSec, census, digests, gathered, acts,
screen, ok}`. Everything property 2 needs is already there — you are not adding
instrumentation, only carrying what the walk already returns.

Residual risk, accepted with eyes open: the operator still has to run it.
Property 3 is what converts forgetting from "never noticed" into "fails at gate
time".

Not chosen, and why: a Go test that drives the walk itself would bind harder,
but needs Go↔browser automation that does not exist and there is one browser
session. A walk-emits-partial/host-completes split was rejected outright — two
half-records that can disagree.

### 2. Audit S1–S6's walk gates against what the walk actually proves

Every stage gate says "by test and by emulator walk". **Nobody has checked that
the walk asserts what those gates assume.** This is the higher-value work and
it is larger than the record.

The reason to do it before S1 opens: **three of this plan's gates turned out to
be unable to fail**, all found today, none by reading — each took running the
check (F-163, F-164, F-165). Two rules to apply to every gate:

- **A gate must fail closed.** Missing evidence reports failure, never silence.
- **A gate must have executed at least once, and been seen to fail once.** An
  unexecuted gate is a hypothesis. Same discipline as a mutation check: a green
  assertion whose subject was never broken is not evidence.

## State

Both repos **clean and pushed**. `master` was pushed via the `ci/staging`
ritual each time (runs `31849912362`, `31851122396`, `31853098270`,
`31853578657`, `31855742586` — all green, no bypass message).

| | |
| --- | --- |
| fork `main` | `3dd64fb` |
| `mnemonic-engrave` `master` | `cd31a65` |

## S0 — all eight deliverables done

| | deliverable | evidence |
| --- | --- | --- |
| D1 | structural embed confinement | `TestEveryEmbeddedPayloadIsStructurallyConfined` |
| D2 | second cosigner payload | 978-byte blob, 2 tests |
| D3 | walk harness | `shTap`/`shPress`/`shRelease`/`shPace`/`shSysw`/`shScreen`/`shToolpath.strings()`; ~165 s six-plate walk |
| D4 | frame receiver | **rescoped** (F-165) + `shot_server.py`'s 3 properties verified, mutation-proved each (`e15e3e6`) |
| D5 | oracle pinning by commit | `oracle` package, `pins.json`, 6 mutations (`1333cc4`, `3dd64fb`) |
| D6 | published-BIP vectors | `address/bip_vectors_test.go`, BIP-383/67/143/84/86 (`042ded2`, `8871fa7`) |
| D7 | `address_test.go` provenance | `0ae3756` |
| D8 | md re-pin + coverage catch-up | `0f9e756` — and it found F-166 |

**Do not close S0 by ticking these off.** The gate also needs the record above,
and its own instruction: *re-derive the gate table from the tree rather than
editing it in place* — that is what went stale the first time.

## Parallelism — settled, do not re-derive

**The concurrency ceiling for S1–S6 is 1**, structurally. Measured in
`design/agent-reports/plan-wide-file-touch-matrix.md` (`bb45d7d`): all five
stages edit `buildMultisigPolicyFlow` (`gui/multisig_build.go:39-198`), S5
deletes code S2 writes, and every gate is the same single-session emulator
walk. Named-vs-actual edit targets: S1 4→8, S2 **0**→9, S3 7→11, S4 2→7, S5 3
citations/0 targets→10. No pair is disjoint. Both S2's and S5's file lists are
now folded into the plan (`2b7fc96`).

Read-only fan-out stays free and paid off repeatedly today. The one thing
isolation cannot buy: on 2026-06-20 two isolated tracks ran cleanly and still
shipped a defect, because the broken invariant spanned both and neither owned
it. **Before any split, name the spanning invariants and their owners.**

**Fable was considered and declined** for the parallelism question: it is a
`git grep` question, not a judgement one. Reserve it for the pre-execution gate
before a first irreversible action.

## Traps, so nobody pays for them twice

- **A walk outliving 1800 s dies in the MCP call, not the emulator.**
  `browser_evaluate` has an idle timeout; a pace-1 six-plate walk is ~2 h.
  Launch fire-and-forget — `run(...).then(r => { window.__walk = r })` — and
  poll `window.__walk`. This cost the pace-1 baseline two sessions.
- **A long-running agent must persist INCREMENTALLY**, not "as its final
  action" — that never fires for a timed loop. One agent re-armed a 2-minute
  timer for ~62 min and wrote nothing. A long-quiet agent is parked, not
  progressing: check whether its output file exists, not its status.
- **A build failure is not a mutation proof.** Two of six mutations first came
  back as compile errors (`if false {` → unused variable), which would happen
  whether or not the test asserted anything. Re-do them so they compile
  (`if clean == !clean {`).
- **Never truncate a digest in a mismatch message.** A hash differing at
  character 16 rendered as `have 9ef480ad1f1e, pinned 9ef480ad1f1e` — two
  identical-looking strings. Print in full.
- **A whole-tree `grep` as an acceptance criterion is a shared resource.** It
  makes one stage's gate depend on every other stage's text (F-163).
- **Name gates by property + file, not by test identifier.** Five of S0's eight
  named tests had drifted (F-164).
- **`git rev-parse HEAD` does not identify a binary built elsewhere.** Pairing
  `~/.cargo/bin/md` with a checkout path is an attestation dressed as a
  measurement. `ByCheckout` now refuses a binary outside its checkout.
- **`shScreen()` inks no spaces** — use `"Engravingplate"`.
- **Only one browser session exists.** Two agents driving it collide.

## Open follow-ups

- **F-165** — D4's rescope. Closing D4 means verifying `shot_server.py`, which
  **is done** (`e15e3e6`); the standing constraint half is inherited by whoever
  ever adds a frame receiver.
- **F-166** — the fork's md decoder refuses a **pathless** origin
  (`md/md.go:893`) that the Rust primary handles as of the commit D8 re-pins
  to. Convergence work, its own cycle. Reproduce in one line: add `"sh_wpkh"`
  to `singleStringVectorNames` and run `go test ./md/`.
- **F-167** — D5 records a seed **digest**, not seed words, departing from the
  plan's text. Fold into the plan's D5 wording at S0 close, or overrule.
- F-158 premise is STALE; F-160 census gap. Neither blocks.

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...      # 51 ok, 0 fail
    nix develop --command go vet ./...       # 6 ArtifactDir = the baseline
    nix develop --command gofmt -l ./

Device build — currently **1,342,468** flash, unchanged all day:

    nix develop --command tinygo build -size short -o /dev/null \
      -target pico-plus2 -stack-size 16kb -gc precise -opt 2 \
      -scheduler tasks ./cmd/controller

Emulator: `nix develop --command ./cmd/emu/build.sh`, serve `cmd/emu` on a
**fresh port** (the browser caches `emu.wasm`), open `index.html`, then:

    const w = await import("./walk_trace_a.js");
    await w.run();          // ~165s, 6 plates, unattributed 0
