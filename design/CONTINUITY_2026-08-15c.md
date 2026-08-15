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

## S0b IS FOLDED — and the no-skip directive was satisfiable

Four commits on the fork's `main`: `afca974` (C-4 sysw vendoring), `9f792c3`
(C-1/C-2/C-3), `af00360` (I-1/I-2), `05c5a73` (the no-skip directive).

**Operator directive 2026-08-15: "Don't skip jobs unless I ask."** Applied. The
opt-out env var and `oracle/optout.go` are **deleted**; the live-oracle checks
moved behind an `oraclelive` **build tag**, so they **do not exist** in a normal
build rather than deciding at runtime to skip. Inside the tag, absence is a hard
`Fatalf`. Run them by name: `./scripts/oracle-live.sh`.

**Controller-verified behaviourally**, not by counting `t.Skip` in source (a
weaker metric — 10 occurrences remain, almost none reachable). Full suite in a
no-oracle environment (fake `HOME`, real `GOPATH`/`GOMODCACHE`, `CGO_ENABLED=0`,
`-count=1 -v`):

    SUITE_EXIT=0 · 0 FAIL · exactly ONE --- SKIP tree-wide:
    TestIdleTimerUnderSH2ShapedEventLoop  (opt-in SH2_REALCLOCK diagnostic)

That is **22 → 1**, and the one left is a deliberate opt-in 3.5-minute wall-clock
diagnostic that never gated anything.

**Two corrections to earlier records here:**

- The CI-enforcing byte-identity test is now
  **`TestAssembledMd1MatchesTheCommittedGolden`**.
  `TestAssembledMd1MatchesThePrimaryByteForByte` (named in the C-3 addendum) is
  by design **not present on CI** — it is behind the `oraclelive` tag.
- **`go vet` and a warm build cache: a warm `GOCACHE` makes `go vet ./...` report
  exit 0 with NO output for an offending package.** Measured: 6 findings instead
  of 40 on identical source. That is almost certainly where the long-lived
  "6 `ArtifactDir`" baseline came from — it was never a miscount, it was a cache
  artifact. **Pin `GOCACHE` on both sides of any vet comparison.**

**Rulings on the fold's two open items:**

- **F-a is NOT blocking S3.** S3's gate is *"emulator walk showing `P2SH-P2WSH`
  on the restore doc, and `grep -rn TYPED-ONLY gui/` returns 0"* — no minted gate
  record. S3 executed both arms. The new `ExpectKind` for built policies is owned
  by the first stage that must **mint a record for a built policy**, not by S3.
- **F-i stays.** `oracle/oracle_test.go:267` skips only where symlinks are
  unavailable — hermetic, never fires on CI or locally (the verification run
  above shows it did not fire), and hides no Go-vs-Rust gate. Converting it could
  hard-fail a contributor on an exotic filesystem for no gain.

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

## S4 IS CLOSED TOO — seven down, and S5 is BLOCKED on two prerequisites

**S4 ✅ 0C/0I**, both Minors folded, merged and pushed; `origin/main` = `80d0c5d`.

The review's strongest evidence was not a test: it took the **seven plate strings
the emulator actually cut** and decoded them cold — the engraved mk1 holds exactly
the key the engraved md1 puts at slot @0, does **not** hold @1's key, and the
engraved ms1 seed derives @0's key at the shared origin. Nothing in the suite or
the walk compared those artifacts to each other; that gap is closed by
measurement.

**S4's walk caught what its tests could not** → **F-185**. The gate's FAIL screen
carried all four required elements *in its string* while the rendered first frame
ended mid-word; `ErrorScreen` scrolls with no affordance, so the host route sat
below an invisible fold. Every content assertion in the package checks the string
**submitted**, not the pixels **drawn** — the F-179 seam by another route. S5
inherits the class.

**Both Minors folded** (`80d0c5d`): the mismatch refusal now leads with the cause
its own negative control produces (picking the wrong card for the slot) — the old
text sent the operator to rewrite a payload that was never wrong — and the
post-gate dispatch ladder now has a regression guard pinning **order**, not just
membership. The first fold attempt dropped `me sysw pack` and S4's own first-frame
test caught it; the body is now **414 chars against the 422 it replaced**, so the
fold margin improved.

## S5's TWO PREREQUISITES — do not brief an implementer before these are ruled

S5's gate is the strongest in the plan: Trace B completes with a correct
descriptor by test **and** emulator walk, and the §4.5 byte comparison extends to
**every mk1 and EVERY ms1, byte for byte, against the current primary**.

1. **No `ExpectKind` exists for what S5 engraves (F-a).** `oracle/expect.go`
   defines exactly one — `KindCosignerCards` → `mk1`. `DeriveExpected` refuses
   any other kind *by design*, and `cmd/gaterecord` refuses to mint a record it
   did not derive. S5 engraves **md1 policy chunks + mk1 + ms1** from a build, so
   **no S5 gate record can be minted today**.
   `TestCommittedFingerprintsAreRealAndDistinct` is a second blocker on the same
   seam — it will hard-fail the first md1-chunk record.
2. **The `ms` oracle pin lags the primary (F-177), and 2026-08-15 widened it.**
   `oracle/pins.json` pins `ms` at **0.15.0** (`ddfa497`); `ms-cli-v0.16.0` was
   released the same day. The gate says *"the **current** primary"*. **A stale pin
   passes SILENTLY** — resolution checks only the installed binary's SHA-256, and
   nothing asks whether the primary moved.

**The question that matters, and why this was not just briefed out:** with the pin
as it stands, would S5's byte-identity gate passing actually mean the engraved
artifacts match what today's primary produces? If not, the strongest gate in the
plan would be green about the wrong thing — this session's whole failure class,
arriving at the last stage that could still hide it.

S5 also inherits **F-185** (it owns the engrave tail's screens).

## Stage status — SEVEN CLOSED, two to go

S0 ✅ · **S0b ✅** (2C/2I → fold → 0C/1I → fold `43a07fe`) · S1 ✅ · **S2 ✅**
(its byte-identity gate is now genuinely CI-enforced — that was C-3) ·
**S3 ✅** · **S3b ✅** · **S4 · S5 · S6 — not started.**

`main` is at `6922b43`. S3 and S3b closed 2026-08-15 after a rebase onto the
folded `main` and a **re-executed** gate — not a re-reasoned one:

- suite exit 0, 51 ok / 0 FAIL; `grep -rn TYPED-ONLY gui/` → **0**; the glyph
  guard scans **1790** production literals, **0 undrawable**.
- the `sh(wsh)` walk re-driven: `ok true`, 9/9 needles, `presented 0`, 9 plates,
  `unattributed 0`, restore doc reading `P2SH-P2WSH 2-of-3 multisig (sorted)`.
- **the rebuild is proven, not asserted**: `emu.wasm` deleted and rebuilt,
  9,809,117 bytes vs 9,809,168 pre-rebase. That delta is the positive evidence
  the binary under test is the new one — the check that catches the stale-cache
  false pass. Served on a fresh port.
- the negative control still discriminates: `templateRow: 0` throws before
  engraving, with a different stub (`06215ac0` vs `cd5ae625`).

**Carry this forward:** the nine engrave digests are **byte-identical** to the
pre-rebase run — same values, same order, same stub. So the S0b fold moved
nothing on the toolpath the build path produces, measured as output rather than
inferred from a diff. It is a same-inputs comparison, **not** the byte-identity
oracle check against the primary toolchain, which is still its own gate.

**An operator still cannot build a wallet policy on the device.** S5 is where the
wallet gets engraved; S4–S5 are the remaining substance and S6 is hardware.

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

## The push bypass — RULED 2026-08-15, and the ruling is an ASYMMETRY

> **"You are not permitted to bypass, but I am."** — operator, 2026-08-15

`enforce_admins: false` on the primaries is **deliberate**: it is the operator's
own escape hatch. **The constraint binds agents, not the human.**

- **Do NOT propose flipping `enforce_admins`.** It would remove a capability the
  operator intentionally holds. The argument in
  `design/agent-reports/operator-rulings-2026-08-15.md` §B — *"solo-maintainer is
  the argument FOR enforcement"* — is **wrong**, because it assumed nobody wanted
  the hatch. That file is written in an operator's voice because the agent was
  briefed that way; its controller header says so, and §B is now superseded.
- **An agent pushing anywhere uses `ci/staging`, always**, and treats a
  "Bypassed rule violations" message as a failure to report, never to paper over.
- **The open consequence, and it is now a precondition rather than a nicety:**
  confirmed by API read, `mnemonic-key` (`main`) and `mnemonic-secret`
  (`master`) both have `strict: false`, and **neither repo's CI builds `ci/**`**.
  So a staged SHA cannot earn a context there, which means an agent forbidden to
  bypass **cannot push to those two repos compliantly at all**. Adding the
  `ci/**` trigger — plus documenting each repo's own context names, since a
  copy-paste of `test (rust + go)` is wrong in both — is what unblocks that.
  Required contexts, verified: `mnemonic-key` → `build (stable on
  ubuntu-latest)`; `mnemonic-secret` → `test (ubuntu-latest)`, `clippy`,
  `test (ms-codec)`, `clippy (ms-codec)`.

**DONE 2026-08-15** — `mnemonic-key` `8dc5dcb`, `mnemonic-secret` `d476b77`.
Both now build `ci/**`; both `CLAUDE.md`s document the sequence with **their own**
context names. `mnemonic-secret` also **dropped its push-side `paths:` filter**,
because with it a docs-only staged SHA would never build and never earn its four
contexts — the same wedge that workflow's own PR trigger already documents and
avoids. Its old rationale ("covered by admin bypass") stays true for the operator
and is false for automation, which is exactly the asymmetry above.

**Safety audit run before pushing — no publish path is reachable from `ci/**`.**
Every other workflow in both repos was checked: `musl-binaries.yml` (`mk-cli-v*`)
and `man-release.yml` (`ms-cli-v*`) are **tag-gated**, as is `release-on-tag`
inside `mnemonic-key`'s `ci.yml`; `vendor-freshness.yml` is `[main, master]` +
paths in both. So a `ci/**` push cannot sign, release or publish anything.

**One accepted side effect, recorded so it is not rediscovered as a bug.**
`fuzz-smoke.yml` in **both** repos triggers on push with **no branch filter** —
only `paths:`. So a `ci/**` staging push whose commit touches `fuzz/**` or
`crates/*-codec/src/**` now runs fuzz-smoke **twice**, once on the staging ref and
once on the final branch. Harmless (it is a smoke test, not a required context,
and duplicate fuzz coverage is not a hazard), and it does not apply to
docs/CI-only commits like these two. Fix if it ever becomes noisy by adding a
branch filter that excludes `ci/**`.

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
