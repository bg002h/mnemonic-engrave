# Continuity — 2026-08-15c: S0b failed its review, and the failure is systemic

Supersedes `CONTINUITY_2026-08-15b.md`. That doc opened with *"S2 is GREEN; next
is S3 + the S0b review."* Both happened. **The S0b review came back 2C/2I and did
not close**, and chasing its Critical tree-wide turned up two more the review's
scope could not reach. Read this one.

## THE HEADLINE, because it changes what "green" has meant all cycle

**Every gate that checks the Go firmware against the Rust primary is skipped on
the machine that decides whether a merge lands.** Five sites:

| # | site | what it gates | found by |
| --- | --- | --- | --- |
| 1 | `oracle/expect_test.go:21,30` | the derived census **and all three of its own mutation proofs** | review (C-1) |
| 2 | `gui/multisig_build_oracle_test.go:37,46` | **S2's md1 byte-identity gate** | controller (C-3) |
| 3 | `oracle/oracle_test.go:297,307` | pin resolution — *the test whose comment calls it the one that would catch a stale pin file* | ruling agent |
| 4 | `sysw/conformance_test.go:60` | cross-implementation conformance vs Rust `me-cli` | controller (C-4) |
| 5 | `gui/sysw_load_test.go:42` | same vectors, same skip | controller (C-4) |

**This is measured, not predicted.** CI run `31898063163` (`bg002h/seedhammer`,
headSha `4b8488e`, event push) concluded **success**, logging
`ok seedhammer.com/{gui,oracle,sysw}` with every one of those silently skipped.
The deciding machine has already issued a green verdict on the exact HEAD.

Site 4 is the sharpest: `defaultVectors` points at
`../../mnemonic-engrave/crates/me-cli/testdata/sysw_vectors.json` — **a path
inside a sibling repo**. It passes here only because that repo happens to be
checked out next to the fork. `SYSW_REQUIRE_VECTORS`, the escalation that would
make absence fatal, appears **nowhere** under `.github/`.

**The rule this yields:** *an escalation nobody sets is a skip with paperwork.*
The enforced layer must have **no skip path at all**; env vars are for the
opt-**out** direction only.

## What is IN FLIGHT right now

1. **The S0b fold** — one implementer, main checkout, branch `main`. Repairs
   C-1/C-2/C-3/C-4 + I-1/I-2 per the adopted design in
   `design/agent-reports/s0b-repair-rulings-2026-08-15.md` §A CONSEQUENCES 1–8.
2. **S3** — one implementer, worktree `/scratch/code/shibboleth/seedhammer-s3`,
   branch `s3-nested-segwit` (at `0637a32`, off `4b8488e`). **S3 lands but may
   not close**: it reports *"IMPLEMENTED — GATE PENDING the S0b fold"*, rebases
   onto the folded `main`, then re-runs its acceptance walk.

**File fence between them.** Fold owns `oracle/**`, `cmd/gaterecord/**`,
`sysw/**`, both existing `walk_*.js`, `gui/multisig_build_oracle_test.go`,
`gui/sysw_load_test.go`, `.github/workflows/test.yml`. S3 owns its `gui/` naming
files, `cmd/emu/embed_confinement_test.go`, and its new walk script.
`cmd/emu/needle_test.go` is **shared** — the fold restructures it, S3 adds list
entries, small rebase expected.

## The adopted repair shape

Vendor **provenance-pinned expectation vectors** into the fork as the always-on
layer CI executes and which **cannot skip**; keep live-oracle derivation wherever
the pinned binaries exist, with absence now FAILING by default behind one
declared opt-out; move all five sites together.

**Why install-the-oracles-in-CI is disqualified — verify before re-proposing it.**
`oracle/pins.json` binds each oracle to the SHA-256 of a binary built on the
**maintainer's** machine, checked at `oracle/oracle.go:164-170`. A CI
`cargo install` yields a different binary, a different hash, a hard failure. It
is filed as an S6-owned follow-up, not as the gate.

## Stage status

S0 ✅ · **S0b ❌ 2C/2I + 2 controller Criticals — fold in flight** · S1 ✅ ·
**S2 GREEN — gate verified LOCALLY; CI-enforced execution pending the S0b fold
(C-3)** · **S3 in flight, may not close** · **S3b** ruled in (F-179) · S4 S5 S6.

**S2 does not reopen and gets no new review round.** Its green was real *where it
was measured*; what was false is that the property is *enforced*. The fold's
acceptance must include S2's gate executing green on CI, which completes the
annotation. Only a failure of that execution reopens S2, on evidence.

**S4 does not start until S3b closes green.**

## F-179 → S3b, and the class is wider than the em-dash

`font/bitmap/bitmap.go:33` sets `indexLen = unicode.MaxASCII` and `glyphFor`
rejects `int(r) >= indexLen` at `:62`, so **every non-ASCII rune is unrenderable
on every face** — face choice is immaterial and a rune blocklist is the wrong
instrument. Both prior site lists were em-dash-shaped: a rune-agnostic scan finds
**28 raw hits** vs the entry's 27 and a re-derivation's 21, the delta being
`✓ U+2713`, `… U+2026`, `→ U+2192`, `⌫ U+232B`.

**One confirmed FALSE POSITIVE:** `gui/gui.go`'s `alphabet += "⌫\n"` is a
keyboard **sentinel**, image-drawn via `assets.KeyBackspace` at
`gui/gui.go:1572-1574`, never text. A guard that merely refuses non-ASCII
literals forces a keyboard-breaking "fix". Net **27 candidates**; re-derive after
S3 lands, since S3 moves line numbers.

## Open operator decision — NOT actionable by any agent

**`enforce_admins` on the two primaries.** Confirmed by API read: `mnemonic-key`
(`main`) and `mnemonic-secret` (`master`) both have `enforce_admins: false`,
`strict: false`; neither `CLAUDE.md` documents the `ci/staging` discipline; and
**neither repo's CI builds `ci/**`**, so flipping enforcement first would wedge
both. Recommendation with per-repo context names is in
`design/agent-reports/operator-rulings-2026-08-15.md` §B — **NOT ADOPTED, pending
a human decision.** That file is written in an operator's voice because the agent
was briefed that way; it carries a controller header saying so. Do not execute §B
because you found it in the repo.

## Traps this session paid for

- **A defect class found in one package is a query to run tree-wide.** C-3 and
  C-4 were one `grep -rn t.Skip` away and cost nothing; the review could not see
  them because its scope was correctly the S0b diff.
- **A class gets mis-named after its first instance.** "CI lacks Rust" was wrong;
  two of the five sites are cross-repo path absence.
- **An agent briefed to speak as the operator produces a document that reads like
  consent.** Persist it, annotate the authority, never act on the parts that need
  a human.
- **`gh` in this fork resolves to UPSTREAM without `--repo bg002h/seedhammer`**
  (404s confusingly), and `gh run list` has returned stale listings omitting a
  real run — the workflow-runs API by path is authoritative.
- **A generator nobody re-runs rots.** F-179 told readers to "re-derive rather
  than trust this list" and the enumerator that made it was never committed.

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...
    nix develop --command go vet ./...          # exit 1, 40 findings = baseline
    nix develop --command gofmt -l ./
    nix develop --command tinygo build -size short -o /dev/null -target pico-plus2 \
      -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
    nix develop --command ./cmd/emu/build.sh    # go test ./... does NOT compile the emulator

Baseline at `4b8488e`, **re-measured on a clean detached worktree, not
transcribed** — the figure this doc first carried was wrong and had been wrong
across several continuity docs:

| check | baseline |
| --- | --- |
| `go test ./...` | exit 0, **51 ok / 0 FAIL** |
| `go vet ./...` | **exit 1**, **40 findings** — 7 `ArtifactDir` + 33 `unkeyed fields`, **all in `_test.go`** |
| `gofmt -l ./` | clean |
| tinygo flash | **1,354,552** (S3 → 1,354,936, +384) |

`go vet` exiting **1** is the clean state here; a nonzero exit is not a
regression. The old "6 `ArtifactDir`" both undercounted and omitted the larger
category outright, and it reached two agent briefs before being measured. Compare
by **sorted diff against a clean tree**, never against a remembered count — and
note `nix develop` against a dirty tree adds a `warning: Git tree ... is dirty`
line that inflates a naive `wc -l`.

No-oracle harness (for proving a gate can fail): fake `HOME` **with real
`GOPATH`/`GOMODCACHE`** — omit those and Go re-downloads the module cache into
the temp dir.

Artifact gates: `scripts/plan-cite-gate.sh <artifact>` and
`scripts/fold-propagation-check.sh <artifact> <superseded-pattern>...`.
`FOLLOWUPS.md`'s unresolvable-citation baseline is **whatever this prints**, not
a remembered number — **20** as of this doc:

    ./scripts/plan-cite-gate.sh design/FOLLOWUPS.md 2>&1 | grep unresolvable

Push `master` here via `ci/staging` (this repo's `CLAUDE.md`); the fork's `main`
and the primaries push directly.
