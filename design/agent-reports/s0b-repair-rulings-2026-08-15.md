# S0b repair rulings — 2026-08-15

**This document is an agent's advisory ruling, standing in for the operator. It
is not a human decision.** Scope per the dispatch brief: engineering design
only. Nothing below proposes or relies on any change to repository security
settings, branch protection, `enforce_admins`, or required-check configuration
on any repo; `operator-rulings-2026-08-15.md` §B remains NOT ADOPTED and is not
built on. Editing workflow *steps* is in scope; changing what any protection
rule *requires* is not.

Basis: `s0b-execution-review-2026-08-15.md` (C-1, C-2, I-1, I-2),
`s0b-review-controller-addendum-2026-08-15.md` (C-3), the coordinator's mid-task
addendum (the sysw sites), and my own verification below. Fork
`/scratch/code/shibboleth/seedhammer` at `4b8488e`, tree clean before and after;
nothing was edited.

**One new measured fact that sharpens everything below.** The Test workflow DID
run on `4b8488e` — run `31898063163`, event `push`, created
2026-08-15T17:21:16Z, conclusion **success**. Its log for the
`CGO_ENABLED=0 go test ./...` step reads:

```
ok  	seedhammer.com/gui	89.913s
ok  	seedhammer.com/oracle	0.135s
ok  	seedhammer.com/sysw	0.010s
```

No skip is visible anywhere in it. So this is no longer a prediction about what
CI *would* do: the deciding machine has already reported success on the exact
HEAD under review, with the derived-census gate, the md1 byte-identity gate, the
pin-resolution test, and the whole sysw conformance suite silently skipped.
(Procedural note: `gh run list` initially returned a stale listing that omitted
this run entirely; the workflow-runs API by path was authoritative. A future
session concluding "CI never ran" from `gh run list` would be wrong the same way
I nearly was.)

**The class is FIVE sites, not two or four.** The tree-wide `t.Skip` grep,
re-run by me, puts these in the class "a Go-firmware-vs-Rust-primary gate that
cannot execute on the deciding machine":

1. `oracle/expect_test.go:21,30` (`resolveBins`) — C-1.
2. `gui/multisig_build_oracle_test.go:37,46` (`s2OracleMD`) — C-3.
3. `oracle/oracle_test.go:297,307` (`TestRealPinsResolveTheInstalledOracles`) —
   found by me: skips when the binaries are absent, so it too has never run in
   CI. Its own comment says *"It is the test that would have caught a stale pin
   file"* — the M-1 backstop is itself behind the same door.
4. `sysw/conformance_test.go:60` — `defaultVectors` (`:18`) points into the
   sibling repo `../../mnemonic-engrave/...`, which the fork's CI never checks
   out; `SYSW_REQUIRE_VECTORS` appears nowhere under `.github/` (grep: no hits).
5. `gui/sysw_load_test.go:42` — same vectors, same skip, same never-set
   escalation.

Not in the class, and deliberately not moved by this fold: the fixture-shape
skips (`gui/bundle_test.go:264`, `md/expand_test.go:65`,
`md/chunk_test.go:201,220`, `engrave/residency_test.go:140`) depend on committed
data, so they return the same verdict on every machine — they are silent-vacuity
hazards, not deciding-machine divergence; filed below as Minors. The `-short`
skip at `oracle/oracle_test.go:285` stays (CI does not pass `-short`; verified
in `test.yml`). `gui/idle_realclock_diag_test.go:112` is an explicitly
env-gated wall-clock diagnostic, and `oracle/oracle_test.go:267` skips only
where symlinks are unavailable, which ubuntu-latest is not.

---

## QUESTION A — how the byte-identity comparison is made to actually run

### RULING

Adopt the hybrid, shape (c): **vendor provenance-pinned expectation vectors
into the fork as the always-on, skip-proof comparison that CI executes on every
push, keep live-oracle derivation wherever the pinned binaries exist with
absence now FAILING by default behind an explicit, declared opt-out — and move
all five skip sites of the class together in one fold.**

### WHY

- **Pure (a) — install the oracles in CI — fails on its own pin design before
  cost is even weighed.** The pins bind each oracle to the SHA-256 of a binary
  built on the maintainer's machine (`oracle/pins.json`: `sha256` per pin,
  resolved in binary-hash mode). A CI `cargo install` produces a different
  binary, a different hash, and a failed resolution — so (a) requires building a
  parallel checkout-mode resolution path anyway, plus a Rust toolchain and three
  external repo fetches on every push of a Go repo, each a new way for the
  deciding verdict to go red for reasons that are not defects. All three primary
  repos are PUBLIC (verified), so (a) is *feasible* — it is just the wrong
  default layer. It returns below as a filed follow-up, not as the gate.
- **Pure (b)'s objection — "a vector minted wrong once is wrong forever and
  green, the skip's disease relocated" — misidentifies the disease.** The skip's
  disease is that the check *does not run* and nothing records that. A vendored
  vector *runs everywhere and can fail*; its residual risks are wrong-at-mint
  and staleness, and both are closed mechanically: (i) the minting tool derives
  live and refuses to write a vector it did not just derive from oracles that
  resolve against `pins.json` — a vector cannot exist except as the output of a
  live derivation (this is C-2's own minimal fix, generalized); (ii) a plain Go
  test, needing no toolchain, asserts every vendored vector's recorded oracle
  identity equals `pins.json` — bump the pins without regenerating and CI goes
  red; (iii) the live layer still runs on every machine that can mint, which is
  exactly the population that could mint fraudulently. What pins the vectors to
  the primary is therefore the same thing that pins the Go ports to the primary
  today: a provenance pin updated on every sync — the repo's own standing
  convention, not a new invention. The sysw pair even shipped the right
  mechanism already (`SYSW_REQUIRE_VECTORS=1` → `t.Fatalf`, plus a
  `len(vs) == 0` INCONCLUSIVE floor); what failed is that nothing ever set it.
  An escalation nobody sets is a skip with paperwork — so this ruling makes the
  enforced layer one that *has no skip path at all*, and reserves env vars for
  the opt-*out* direction only.
- **The contributor-experience concern is legitimate and currently mis-served.**
  A contributor without Rust deserves a suite that checks what it can and says
  what it cannot — not a suite that silently checks less. Under this ruling
  they get the real byte-identity comparison with zero Rust (the vendored
  layer), and the live layer's failure message names its own two remedies:
  install the pinned oracles, or type `SH_ORACLES_OPTIONAL=1`. **What makes the
  opt-out non-silent:** failure is the default; skipping requires either a
  human-typed environment variable or a reviewed line in the workflow file that
  states what still enforces the property there; and no test that *can* be
  opted out of is load-bearing — enforcement rests entirely on the layer that
  cannot skip.

### CONSEQUENCES

The coupled fold — all of it lands together or none of it:

1. **`oracle/expect_test.go`** — `resolveBins` absence becomes `t.Fatalf` by
   default; `SH_ORACLES_OPTIONAL=1` (name at implementer's discretion, one
   variable for the whole class) restores the skip, with the message naming
   both remedies and the vendored layer. Delete the false justification at
   `:13-16` ("The gate that makes absence fail is `TestS0GateHasARecord`") —
   the review proved it false, and comments outlive their conditions.
2. **`oracle/` vendored expectations** — commit
   `oracle/gaterecords/S0-trace-a.expect.json`: the derived artifact strings
   plus provenance (per-oracle name, commit, binary SHA-256, derivation args).
   Rewrite the census gate to: loop over **every** record in
   `oracle/gaterecords/` (C-2's fix — the hardcoded filename dies); FAIL on any
   record without a committed expectation; compare census vs expectation
   unconditionally (no oracle needed — this arm cannot skip; a missing or empty
   expectation is `t.Fatalf("INCONCLUSIVE...")`); where oracles are present,
   re-derive live and require equality with the committed expectation.
3. **`cmd/gaterecord`** — refuses to mint a record whose census does not equal
   the live-derived expectation, and writes the `.expect.json` beside the
   record as part of minting. Minting is where live derivation is mandatory and
   unskippable — which is also why the dev-machine opt-out is safe.
4. **`gui/multisig_build_oracle_test.go`** — same default-flip in `s2OracleMD`;
   the test splits into an unconditional comparison of the device's assembled
   chunks against a committed golden (`gui/testdata/`-something, carrying the
   same provenance block, derived from `md` @ `5a0a4f41`), plus the existing
   live-oracle comparison where `md` exists, which must equal the golden
   (freshness).
5. **`oracle/oracle_test.go`** (`TestRealPinsResolveTheInstalledOracles`) —
   the absence skips at `:297,:307` follow the same default-fail/opt-out rule.
   The `-short` skip stays.
6. **Provenance-equality test** (new, no toolchain needed) — every committed
   expectation/golden's recorded oracle identity must equal `oracle/pins.json`.
   This is the always-on binding that makes stale vectors red in CI.
7. **sysw** — vendor `sysw_vectors.json` into the fork (e.g.
   `sysw/testdata/`), with a provenance pin recording the `mnemonic-engrave`
   commit and file SHA-256, per the standing Go-port provenance-pin convention.
   `defaultVectors` in `sysw/conformance_test.go` and the path in
   `gui/sysw_load_test.go` point at the in-repo copy; missing or empty is
   `t.Fatalf("INCONCLUSIVE...")` always — the skip dies. `SYSW_VECTORS` stays
   as a dev override. Add a sync-audit test comparing the vendored copy to the
   sibling checkout when it is present; it may skip when the sibling is absent
   because it is an audit of freshness, not the gate — say so in its comment.
8. **`.github/workflows/test.yml`** — set `SH_ORACLES_OPTIONAL=1` on the
   go-test step, with a comment stating exactly why (no Rust toolchain here)
   and exactly what still enforces byte-identity on this machine (the vendored
   layer, which cannot skip). This is a step edit, within scope. For the fold's
   acceptance only, run the gate tests once with `-v` on CI (a temporary step
   or a manual re-run) so the fold commit can quote the deciding machine's own
   `--- PASS` lines by test name; the *permanent* proof-of-run is structural —
   the vendored tests have no skip path, so their package `ok` implies
   execution.

**What must be able to fail, and how that is proven** (each is one command; the
outputs go in the fold commit's message per the two-commit rule):

- Flip one byte in a vendored expectation → suite RED **in a no-oracle
  environment** (fake `HOME`, real `GOPATH` — the review's own harness).
- Flip a pin's commit in `pins.json` without regenerating vectors →
  provenance-equality test RED, same environment.
- Delete a vendored vector file → `INCONCLUSIVE` fatal, not skip.
- Run the live tests in a no-oracle environment **without** the opt-out →
  FAIL naming the remedies; **with** `SH_ORACLES_OPTIONAL=1` → skip.
- Mint a record with a doctored census via `cmd/gaterecord` → refused.
- One byte flipped in the vendored sysw copy → conformance RED.
- The review's three existing mutation proofs re-run green→red→green after the
  rewrite, since the fold restructures the tests that carry them.

**Filed, not folded** (kept out of the coupled set deliberately):

- The four fixture-shape skips and `engrave/residency_test.go:140` → convert to
  `t.Fatalf("INCONCLUSIVE...")`; Minor, no owning phase, batches to the end —
  or a separate third commit if the implementer wants them now; never mixed
  into the fold commit.
- M-1 (pin-distance-to-primary-HEAD report) — unchanged, still Minor.
- A live-oracle CI job (checkout-mode builds of the three pinned oracles from
  their public repos, in a second workflow job) — follow-up owned by **S6**,
  the pre-hardware gate, where its marginal value (anti-mint-fraud on the
  maintainer's machine) is worth its flake surface. It is explicitly NOT
  needed for this fold's guarantee.
- I-1 and I-2 ride in the same S0b fold as the review prescribed (drop
  `census.strings.length === plates` from both walks' `ok` and report the count
  as data; add the Go test that extracts `NEEDLE_*` literals from
  `walk_build_policy.js` and requires each to appear in `buildFlowNeedles`) —
  they are S0b findings, just not part of Question A's coupled pair.

### WHAT I VERIFIED

- `grep -rn "t\.Skip" --include="*.go"` tree-wide (excl. `third_party`): 17
  hits; the five class members and every non-member listed above were read in
  context, not counted from the reports.
- `oracle/expect_test.go:17-45` read: `t.Skipf` at `:21,:30`; the false comment
  at `:13-16`. `gui/multisig_build_oracle_test.go` read in full: `t.Skipf` at
  `:37,:46`; doc comment carries the contributor rationale verbatim.
  `oracle/oracle_test.go:283-329` read: absence skips at `:297,:307`, `-short`
  at `:285`, and the "would have caught a stale pin file" comment.
- `sysw/conformance_test.go:18` — `defaultVectors =
  "../../mnemonic-engrave/crates/me-cli/testdata/sysw_vectors.json"`; the file
  exists locally and is tracked in `mnemonic-engrave` (8,549 bytes);
  `grep -rn SYSW_REQUIRE_VECTORS .github/` → no hits; workflows present:
  `test.yml`, `image.yml` only.
- `.github/workflows/test.yml` read in full: `actions/checkout` +
  `actions/setup-go`, `CGO_ENABLED=0 go test ./...`, no Rust step, no `-short`,
  no SYSW env; the "gate whose instrument does not compile" comment is at the
  `GOOS=js` vet step.
- CI: `gh api .../workflows/test.yml/runs?branch=main` → run `31898063163`,
  head `4b8488e`, event push, success, 2026-08-15T17:21:16Z; its log's three
  package lines quoted above; no skip text appears in the non-verbose log.
- `gh repo view` × 5: `descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`,
  `mnemonic-engrave`, `seedhammer` all PUBLIC.
- `oracle/pins.json` read: three pins, binary-SHA-256 attestation, commits
  `5a0a4f41` / `a38a908e` / `ddfa497`.
- `grep -rn "CompareCensus\|DeriveExpected"` excluding tests: definitions only
  (`oracle/expect.go:125,179`) — C-2's no-callers claim re-measured.
- `oracle/gaterecords/` holds exactly the three `S0-trace-a.*` files.
- Local full suite at `4b8488e` via `nix develop`: 51 `ok`, zero `FAIL`/`SKIP`
  lines (oracles present here, so the gates ran); `git status --porcelain`
  empty before and after; HEAD unchanged. An earlier background attempt printed
  only `zsh: command not found: nix` with exit 0 from the pipe's tail — caught
  by reading the output, re-run with the absolute nix path; "empty output is
  not absence" fired again and is worth recording.

---

## QUESTION B — stage S2's formal status

### RULING

**S2 stays CLOSED at 0C/0I — no reopening and no new review round — and its
status line is annotated to "GREEN (0C/0I; gate verified locally; CI-enforced
execution pending the S0b fold, see C-3)", with the S0b fold's acceptance
required to include S2's gate executing and passing on CI, which completes the
annotation — or, if that execution fails, reopens S2 then, on evidence.**

### WHY

The two project rules do not actually collide here, because they bind different
artifacts. Lens-closure says a clean round exhausts the *question asked*, not
the artifact — and C-3 is a genuinely new question ("where does enforcement
execute?") asked of a *different artifact*: S0b's scaffolding and the CI
wiring, where the skip construct lives and where the repair belongs. S2's
review answered its own question — is the assembly correct, and is its gate
real where run — and that answer stands: the byte-identity test executed and
passed on the machine where it was run, measured in both directions in the
addendum. The "no gate may never-have-run" clause does not fire either: S2's
gate *has* run; what has never run is its CI enforcement, which was never S2's
deliverable. Reopening S2 would buy a round of re-deriving settled facts —
exactly what the proportional re-review rule forbids — while leaving the status
unannotated would let a future session read "GREEN" as "enforced", which is the
one false fact in circulation. The annotation records precisely what was
measured and what was not, and hands the repair to the fold that owns the
defective mechanism. The contingency clause exists because the first CI-executed
run of S2's gate is also its first run on a fresh environment: if it fails
there, that is new evidence about S2, and evidence — not reassurance — is what
reopens a closed loop.

### CONSEQUENCES

- **Next continuity doc** (the successor to `CONTINUITY_2026-08-15b.md`): the
  stage line changes from `S2 ✅ 0C/0I after one round and one fold` to
  `S2 ✅ 0C/0I — gate verified locally; CI-enforced execution pending the S0b
  fold (C-3)`. This is the one place a future session reads first, so it is the
  load-bearing copy of the annotation.
- **The closed S2 reports are not edited.** `s2-execution-review-2026-08-15.md`,
  `s2-fold-2026-08-15.md`, `s2-fold-review-2026-08-15.md` are verbatim records
  and stay byte-identical.
- **The S0b fold report** records the first CI-executed PASS of
  `TestAssembledMd1MatchesThePrimaryByteForByte` — run id, date, and the quoted
  `--- PASS` line from the deciding machine's log. The continuity doc then
  drops the annotation in its next update.
- **If that run fails**, S2 reopens at that moment as a new Critical with the
  CI log as its evidence; nothing is pre-emptively reopened today.
- No review agent is dispatched against S2. C-3 is charged to S0b, and S0b's
  loop is already open.

### WHAT I VERIFIED

- `CONTINUITY_2026-08-15b.md` read: the S2-closed statement, the stage line's
  exact wording and location, and the standing instruction not to re-run a
  closed loop for reassurance.
- The addendum's both-directions measurement of the S2 gate (PASS with oracle
  present, SKIP without, both `ok`/exit 0) re-read against the actual test
  code, which I read in full — the test body genuinely compares chunk-by-chunk
  with full strings on mismatch, so the local PASS was a real comparison.
- The deciding machine's verdict on S2's merge commit exists and is `success`
  with the gate skipped (run `31898063163`, quoted in the header) — so
  "enforcement pending" is the measured status, not a hypothesis.
- `push-2026-08-15-s2-green.md` read: fork pushed `ca2e14b..4b8488e` plain to
  `main` ("No branch protection observed" — recorded here as a fact about how
  the deciding machine is reached; no settings change is proposed, per scope).
- `git ls-remote origin main` → `4b8488e`; local `main` and `origin/main`
  identical.

---

## QUESTION C — sequencing against S3 in flight

### RULING

**S3 proceeds now and does not pause, but may not close: the S0b fold gets its
own single implementer in the main checkout immediately, running concurrently
with S3 under an explicit file fence; the fold lands on `main` first; S3
rebases onto the folded `main` before its acceptance walk runs; and until that
rebased walk passes, S3 reports "IMPLEMENTED — GATE PENDING the S0b fold",
never GREEN.**

### WHY

Pausing S3 buys nothing: its authored surface (`scriptName` and its three
callers, the `TYPED-ONLY` burndown in `gui/`, the restore doc, a brand-new walk
file) is disjoint from the fold's surface, and the parallel-writer rule is
satisfied — separate worktrees, separate branches, disjoint files. But letting
S3 *close* before the fold lands would mint one more green on scaffolding with
known Criticals: its walk's `ok` still carries I-1's driver-supplied `plates`
term, its gate record would be minted by a `cmd/gaterecord` that does not yet
enforce expectations, and C-2 means no test would ever compare that record's
census — the exact false-summit shape this cycle has now paid for twice. The
closure rule's second clause decides it: S3's gate *as specified after this
fold* — a walk whose record is covered by the every-record derived comparison —
has never run, and a gate that has never executed is a hypothesis. On "one
implementer": that directive is per work item, not per repository. The fold is
its own work item with its own persisted findings to answer, and the
established isolation rule exists precisely to let two writers proceed in
separate worktrees on disjoint files. Serializing the fold behind S3 would
invert priority — the fold repairs the instrument that every later stage's
green depends on, so every day it waits is another day stage work can close
against a broken instrument.

### CONSEQUENCES

- **S3 (worktree `/scratch/code/shibboleth/seedhammer-s3`, branch
  `s3-nested-segwit`)**: continues as briefed. Until it has rebased onto the
  folded `main`, it must not touch: `oracle/` (anything), `cmd/emu/needle_test.go`,
  `cmd/emu/walk_trace_a.js`, `cmd/emu/walk_build_policy.js`,
  `gui/multisig_build_oracle_test.go`, `gui/sysw_load_test.go`, `sysw/`,
  `.github/`. Its new walk script stays a new file; if its walk needs new
  needles in `buildFlowNeedles`, that edit happens after the rebase (its gate
  cannot run before then anyway). The existing warning not to reproduce I-1/I-2
  in its walk stands — concretely: no driver-supplied count in `ok`, and its
  `NEEDLE_*` list must be covered by the extractor test the fold adds.
- **The S0b fold**: one dedicated implementer, main checkout
  `/scratch/code/shibboleth/seedhammer`, starting now, owning exactly the
  fenced file set above plus the new vendored files and `cmd/gaterecord`. The
  standing loop applies: the reviews are already persisted (two commits in this
  repo); fold → run the fail-proofs and full suite → fold commit carrying the
  gate output → scoped re-review (the fold is non-trivial — new logic — so
  re-review is mandatory; sonnet for does-the-fold-match-the-findings
  mechanics, opus if the vendored-layer design itself needs design-level eyes,
  fable not warranted — nothing here is the last review before an irreversible
  action).
- **Landing order**: the fold is the next landing on `main`; no stage work
  merges to `main` before the fold and its re-review close.
- **S3's closure path**: after the fold lands — rebase `s3-nested-segwit` onto
  the new `main`, resolve the (expectedly small) overlaps, re-run unit tests,
  then run its acceptance walk on the repaired scaffolding, minting its record
  through the expectation-enforcing `cmd/gaterecord`. Only then can S3 close,
  through its normal review loop.
- **What S3 may claim if it reports before then**: implementation complete;
  unit tests green locally on its branch (based on `4b8488e`); walk-rehearsal
  results as *data*, explicitly caveated that the walk's `ok` still contains
  I-1's `plates` term. It may not claim GREEN, closed, 0C/0I, or "gate
  passed". Status string: **"IMPLEMENTED — GATE PENDING the S0b fold and
  rebase."**
- **S3b** is unchanged: same worktree, same implementer, after S3's rebase.

### WHAT I VERIFIED

- The S3 plan section read
  (`IMPLEMENTATION_PLAN_multisig_build_repair.md:959-1015`): S3's gate is an
  emulator walk of an `sh(wsh)` build plus a `gui/`-scoped `TYPED-ONLY` grep —
  i.e. it terminates in exactly the walk/record scaffolding under repair.
- S3's file surface from the plan (`gui/md1_inspect.go`,
  `gui/multisig_restore.go`, `gui/bundle.go`, `TYPED-ONLY` comment sites in
  `gui/multisig.go`, `gui/bip85.go`, `gui/singlesig.go`,
  `gui/multisig_build.go`, one `cmd/emu` comment citation) is disjoint from the
  fold's file set except for the `cmd/emu` comment retirement, which the plan
  already demotes to stage work S3 "cannot control" — deferred to post-rebase
  along with any needle additions.
- `walk_trace_a.js:151/274` and `walk_build_policy.js:299/532` read: the
  `plates` default and the `census.strings.length === plates` term in each
  `ok`, verbatim as the review states.
- `grep -rn walk_build_policy --include="*.go"`: zero hits — no Go file reads
  the driver (I-2 re-measured, not inherited).
- The parallel-isolation and one-implementer directives re-read from this
  repo's `CLAUDE.md` and memory notes; the fence above is what makes both hold
  simultaneously.
