# Continuity — 2026-08-14, on-device wallet-policy authoring

Written at the end of a session that ran spec → plan → fifteen review rounds →
first code. Read this instead of the fifteen reports.

## Where to start

**S0 deliverable 3, second half: `window.shScreen()`.** Everything else is
either done or blocked behind it.

The route is settled and the reasoning is not worth re-deriving:
`Context.FrameCallback` is owned by `gui/run_flow.go`'s run loop, so `cmd/emu`
must not take it. Add an **optional interface the platform implements**, exactly
as `gui.PlateAware` already does for the plate overlay — that pattern exists in
this codebase precisely because the emulator needs to see something the firmware
does not expose. `shToolpath.strings()` (the engraved md1/mk1/ms1 out of a
completed walk) reaches the artifacts the same way.

This is the **first `gui` package change** of the cycle. Everything before it was
emulator-only.

## State

| | |
| --- | --- |
| Spec | `design/SPEC_multisig_build_repair.md` — R0 GREEN |
| Plan | `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` — R0 GREEN, then four lens reviews + a fact audit reopened it; all folded |
| Code | fork `main`, three commits: `3009f22`, `3ea08f9`, `10286e4` |
| Stage | S0, 3 of 8 deliverables |
| Repos | both clean, both pushed as of the last commit |

**Do not re-review the plan.** Fifteen rounds have run and the last four found
what reading could not. The prescription from
`design/agent-reports/multisig-build-repair-process-diagnosis.md` is: stop
reviewing text, build S0's harness, walk Trace A until it breaks.

## What the three commits established

1. **`3009f22`** — confinement is now STRUCTURAL. It discovers every
   `//go:embed` under `cmd/emu` rather than matching a names list, checks the
   **AST** not raw text (so a doc comment mentioning a blob is not a leak), and
   is mutation-checked three ways. The two name-keyed guards it subsumes are
   deleted; `TestNothingImportsTheEmulator` is untouched and pins a different
   property.
2. **`3ea08f9`** — a SECOND payload, `sysw_cards_payload.bin`, 978 bytes, four
   cosigner mk1 cards + master A. The first blob has no `ClassMDMK` at all,
   which is why no walk could reach Build policy's gather. Its record inventory
   is in the source **and** pinned by a test, asserted per-ORIGIN: a count tells
   you a card vanished, an origin tells you which stage lost its walk.
   Regenerate with `go run ./cmd/buildpayloadcards`.
3. **`10286e4`** — `shTap`, `shPress`/`shRelease`, `shSysw("records"|"cards"|
   "none")`. Verified driving the real GUI in a browser (ink 7442 → 9009 →
   14091 across two taps).

## The traps, so nobody pays for them twice

- **A walk without `shScreen()` is a hope, not a walk.** Proven the hard way: the
  primitives work, and a full walk still failed because one mis-timed step made
  every later step a guess. Four consecutive "steps" sat on the same screen
  unnoticed.
- **`mk.Decode` returns `m/48h/0h/0h/2h`**, not apostrophe notation. An
  expectation asserting apostrophes failed first.
- **Never edit `sysw_test_payload.bin`.** Its digest is pinned in
  `sysw_test_payload.go` *and photographed* in the published Load Payload
  journey PDF. That is why there are two blobs.
- **`sh(wsh)` is a template-PICKER choice, not a payload property.** S3 needs no
  extra record. The plan briefly implied otherwise.
- The `go vet` finding in `gui/freetext_sizeproof_golden_test.go`
  (`testing.ArtifactDir requires go1.26`) is **pre-existing and unrelated**.

## Process rules this cycle changed

Both are now in `CLAUDE.md`; this is the short form.

- **Closure is LENS-closure, not finding-closure.** Both gates closed GREEN under
  a correctness lens and the six rounds after found seven more Criticals — every
  one from a *first-time question*, none from looking harder. Enumerate the
  lenses up front.
- **A plan may not close while one of its own gates has never been run.** The
  acceptance mechanism here was a walk against a payload with no cosigner cards:
  invisible to thirteen readings, an hour to find by trying it.
- **Folds fail by incomplete propagation**, four times this cycle. Run
  `scripts/fold-propagation-check.sh` after every fold, and **derive the patterns
  from the report's own quoted strings**, not from a paraphrase — a gate fed
  paraphrased patterns passed while the defect survived.
- **Gate coverage, not gate quality, is the weak point.** 16/16 *gated* code
  citations were true; **5/22 ungated facts were false**.

## Still owed, in order

S0: `shScreen()` + `shToolpath.strings()` (blocked, above) · frame-receiver
security · oracle pinning by **source commit** (a `--version` string is
self-reported and spoofable) · published-BIP vectors — **BIP-383** for
`wsh(multi(…))` compared at *scriptPubKey*, BIP-67 ordering, BIP-141 for
P2SH-P2WSH; **not** BIP-382, which publishes no addresses · `address_test.go`
provenance · md vendored-vector re-pin (0.36→current; measured **zero** byte
drift, so coverage catch-up not correctness repair).

Then S1–S5 per the plan. **S6 is hardware and needs the operator**: flashing via
`~/bin/sh/sh2-flash`, cutting real plates, restoring at an external coordinator,
including one divergent-origin multi-slot multi-master build and an ms1 plate
read back.
