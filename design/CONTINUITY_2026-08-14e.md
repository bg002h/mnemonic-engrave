# Continuity — 2026-08-14e, S0 is CLOSED and the walk gates are not what they said

Supersedes `CONTINUITY_2026-08-14d.md`. Both of 14d's "Where to start" items are
**done**: the fail-closed gate record is built, exercised and committed, and the
S1–S6 walk-gate audit is written, folded and pushed. Read this one.

## Where to start

**One ruling is BLOCKED ON THE OPERATOR and everything else waits on it.**

### F-173 — the ruling to make first

The review found a Critical that is independent of every walk finding and
survives all three options below: **Trace A cannot complete on the payload S0
delivered.** The payload holds nine `ClassMDMK` records forming **four** cards;
S1's unconditional feed puts all four into `buildCosignerCards(cards, p.N-1)`,
whose last check is `if len(out) != want` (`gui/multisig_build.go:268`); and
`n` ranges over 2..5, so `want` ranges over 1..4. **The build refuses for every
n except 5**, and n=5 is neither trace. S2's "Trace A completes end to end" is
unsatisfiable with or without a walk.

Two ways out, and it is the operator's call:

- **(a) per-card accept/skip on the payload feed** — S1 scope, one screen,
  restores every n, keeps §2's Trace A as written. This looks right.
- **(b) run the walks at n=5** and restate §2's Trace A shape.

S0's gate missed it because `TestSyswCardsPayloadCoversEveryStagesWalk` has a
`len(mdmk) < 8` floor and **no ceiling** — it gates "enough cards", never "a
usable number of cards".

### The decision that follows it

Five stage gates say *"by test and by emulator walk"*, and
`design/RECON_S1_S6_walk_gates.md` measured that the only walk that exists
drives a **different program** than the one all five stages edit. So S1 no
longer starts with `gui/sysw_session.go`; it starts with the walk it will be
gated by, or it starts with a gate that cannot execute.

Three ways to take it, and this is a ruling to make rather than a thing to
discover in S1's third round:

1. **S1 builds the walk scaffolding first** — a Build-policy walk with a
   flow-identifying needle (F-169) and an input-tuple-derived census (F-170) —
   then its own feature. Honest, and it makes S1 bigger than the plan says.
2. **A pre-S1 stage S0b** that owns the scaffolding for all five, since S1–S5
   share it and the concurrency ceiling is already 1.
3. **Weaken the clause** to "by test", and say plainly that the walk is a smoke
   check rather than a gate.

### REVIEWED, and the answer moved

An independent opus review ran on 2026-08-14 —
`design/agent-reports/s1-walk-gate-judgement-review.md`, persisted at `b8c8cb5`,
folded at the commit after. Every measurable claim in it was re-measured by the
controller before folding.

**C1 CONFIRMED for all five.** S3 is the only partial (its `grep` half needs no
walk). **Option 3 is off the table and was never the plan's to take:** SPEC
§4.5 is REQUIRED, quotes the operator verbatim — *"The emulator must be used to
walk journeys…"* — and states *"A green unit suite is explicitly NOT
sufficient"*, with R-4 (`SPEC:746`) restating it. Weakening it is an operator
decision.

**Recommendation is still 2, and its reason of record has CHANGED.** The
rationale written here first — "splitting them across stages means the byte
comparison arrives at S2 having never been exercised at S1" — is wrong: S1's
walk produces no artifact, so the comparison cannot be exercised at S1 under any
option. The real reason is stronger. **F-170 and F-171 have exactly one target
that exists today with known-correct expected outputs: S0's committed record**
(`oracle/gaterecords/S0-trace-a.record.json`, six mk1 plates reproducible from
`go run ./cmd/buildpayloadcards`). Build the census derivation and the oracle
comparison there and mutate an expected string until it goes red. Do it at S2
instead and the harness's first execution is also its first verdict — the
never-run gate this cycle has now been burned by twice.

**S0b's scope is narrower than "the scaffolding for all five".** Three shared
mechanisms, each exercised against the S0 record: the build-flow driver + needle
(F-169, F-174), the census derivation replacing `plates = 6` (F-170), and the
oracle comparison (F-171) — plus its own three-way seen-to-fail proof. **Not**
the five per-stage walk scripts: the build flow's tail cannot be walked before
the code that makes it walkable exists.

## State

Both repos **clean and pushed**. `master` went through the `ci/staging` ritual
(run `31858455895`, all seven jobs green, no bypass message).

| | |
| --- | --- |
| fork `main` | `88d43c7` |
| `mnemonic-engrave` `master` | `1d728e8` |

## S0 — CLOSED

Eleven properties, re-derived from the tree at close rather than edited in
place (that instruction is what caught the mislabelled walk). The table is in
the plan; do not duplicate it here.

The last clause — the gate record — landed at fork `88d43c7`:

- `oracle/record.go` — `ParseWalk` refuses anything that is not a completed
  green walk; `NewRecord` embeds the walk's census and per-plate digests plus
  the full SHA-256 of the raw `run()` value; `VerifyRecord` re-checks the pair
  and every oracle commit against `pins.json`.
- `cmd/gaterecord` — the emitter. `-walk` is required and there is no path that
  writes anything without one.
- `oracle/gaterecords/S0-trace-a.{record,walk,inputs}.json` — a real run: six
  plates, 174 s at pace 2048, `unattributed 0`, walk sha256 `fb294b52…41e7`.
- `TestS0GateHasARecord` **never skips**; `cmd/emu`'s anchor test proves each
  engraved mk1 is a record of the payload the record names, so a self-consistent
  pair cannot vouch for itself.

**Seen to fail, three ways**, then green on restore — remove the record, change
one character of the walk file, change one embedded plate digest. The commit
message carries the exact failure text.

## What the audit found, in one paragraph each

Full report: `design/RECON_S1_S6_walk_gates.md`. Follow-ups F-168–F-172.

- **The walk is a bundle engrave, not Trace A.** `goTo("EngraveBundle")` →
  `bundleFlow`. `buildMultisigPolicyFlow` sits behind `Engrave Multisig → Build
  policy` and is never entered. S0's evidence line called it a Trace A run;
  corrected in place. **This is not S0 D3 failing** — the shapes drive the build
  flow fine; the label was wrong, and a label is what a later stage reads when
  it decides its own gate is already met.
- **Every needle is ambiguous.** `"First card from where?"` is in three
  production flows; `"Choose engraving"` in six; the gather title says "Engrave
  Bundle" from inside Build policy because it is set in the shared gatherer. A
  walk written by editing this script's `goTo` target asserts identically and
  nothing notices.
- **`ok` is a plate count.** `CARDS` holds the six expected strings and is never
  compared to the census. The plan's own §3 preamble requires the census be
  derived from the input tuple; nothing derives it, and `plates` is a literal.
- **Nothing invokes the pinned `md`/`mk`/`ms`.** S2's and S5's byte comparisons
  are unimplemented, not merely unrun.
- **S3's restore doc is skipped on the template branch**, so its gate has
  nothing to read unless that walk picks "Full policy md1".

## Traps, added to 14d's (which all still hold)

- **`walk_trace_a.js` runs at pace 2048 by default, ~174 s for six plates.**
  Launch fire-and-forget and poll; 14d's 1800 s MCP timeout trap is real.
- **One walk per page load.** The census is cumulative for the session with no
  reset. It fails closed today only because two checks are strict equalities;
  do not relax either to `>=`.
- **The cite gate SILENTLY SKIPS comma-list citations.** `` `f.go:359,386` ``
  never matches — the pattern needs the backtick right after the number. Same
  blind spot its own header records for ranges. Write them separately.
- **Playwright's `browser_evaluate` `filename` writes the JSON-ESCAPED string**,
  not the value. It looks right and is double-encoded. Hash both ends: the
  committed walk file was proved byte-identical to the browser's by SHA-256 on
  each side before it was recorded.
- **Do not describe a flow from its doc comment.** `multisig_build.go:67` says
  "(3) TYPED-ONLY self seed"; the call it guards is `seedEntryFlow`, which puts
  `FROM PAYLOAD` first. Believing the comment would have bought a seed-typing
  harness nobody needs. It is one of the nine stale `TYPED-ONLY` comments S3
  deletes.

## Open follow-ups

- **F-173** → **RULING FIRST**, before S0b or S1 is scheduled. See above.
- **F-168** → S0 (folded; the "one walk per page load" half is a standing note).
- **F-169, F-170** → S0b if created, else S1, **gating**. The driver + needle and
  the derived census. F-170 now also owned by **S3** — the preamble exempted it
  on a false premise, and S3 engraves before its restore doc.
- **F-171** → S0b if created, else S2, **gating**. The oracle-invoking harness.
- **F-172** → S3. Pick "Full policy md1" or the gate reads nothing.
- **F-174** → S0b/S1, **gating**. Zero `shNFC.present` in a stage-gate build
  walk, or the gate passes without S1's feature.
- **F-166** → its own cycle. The fork's md decoder refuses a pathless origin.
- **F-167** → CLOSED, folded into D5's text at `1d728e8`.
- F-158 premise STALE; F-160 census gap. Neither blocks.

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...      # 51 ok, 0 fail
    nix develop --command go vet ./...       # 6 ArtifactDir = the baseline
    nix develop --command gofmt -l ./

Device build — **1,342,468** flash, unchanged (nothing added reaches
`./cmd/controller`).

Emulator: `nix develop --command ./cmd/emu/build.sh`, serve `cmd/emu` on a
**fresh port**, open `index.html`, then

    import("./walk_trace_a.js").then(w => w.run()).then(r => { window.__walk = r });

and afterwards

    go run ./cmd/gaterecord -stage <S> -walk <saved>.json \
      -inputs oracle/gaterecords/<S>.inputs.json -base <S>-<trace>

Gate scripts, all run before the fold at `1d728e8`:
`scripts/plan-cite-gate.sh`, `scripts/fold-propagation-check.sh`.
