# F-630 — controller measurements, 2026-09-20

Recorded so they survive compaction. Measured against fork `95716e97` and
descriptor-mnemonic `b2c5d693`. These are the controller's own runs, not an
agent's report.

## The fork already emits the 0.44.0-correct header

A throwaway probe in package `gui` (`expandedToDescriptor` → `Encode()`) on
`keyed_compose_preset_plain_multisig`, compared against dm `b2c5d693`'s
`chains["0"]` and `chains["1"]`:

    chain 0: all 3 xpubs identical to the fork render
    chain 0: all 3 origin brackets identical (h/' spelling normalised)
    chain 1: all 3 xpubs identical to the fork render
    chain 1: all 3 origin brackets identical (h/' spelling normalised)

So "byte-identical, key for key" in the plan is all six keys across both
chains, not one spot check.

## Hardening is spelled differently on the two sides — FOLD THIS INTO THE PLAN

The fork renders `[73c5da0a/48h/0h/0h/2h]`; the primary's record carries
`[73c5da0a/48'/0'/0'/2']`. Same path, different spelling of hardening
(`bip32.Path.Encode()` emits `h`; `Derivation.Encode()` likewise, bip380.go).

Consequence for the gate: D1 parses the RECORD's bracket, which is always the
`'` spelling, so counting components is unaffected. But **D3's allowlist
compares a bracket path to the Go port's `OriginPath`**, and that comparison
must normalise `h` ↔ `'` or it will report a divergence that is only a
spelling. Same class as F-627.

Not folded into the plan yet — the R0 reviewer is reading `5cc0a0a2` and the
artifact must not move under it. Fold this with the review findings.

## Corpus drift, for reference

46 keyed conformance records: 2 identical, 41 descriptor-only (88 chain
entries), 3 semantic (F-529's three, none a `keyed_compose_*`). Zero address
lines and zero id lines move outside those 3.

## T4 baselines, re-measured at fork main `95716e97` (2026-09-20)

Both hold, and one needs a correction in the plan's T4 wording.

**`gofmt -l .`** — exactly the five files `CLAUDE.md` records, no drift:
`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`.

**`go vet ./...` EXITS 1 AT BASELINE.** Ten diagnostics, every one of them
`testing.ArtifactDir requires go1.26 or later (file is go1.25)`; nothing else.
So T4's gate is *"vet's diagnostic set equals the ten known ArtifactDir
notices"* — **not** `go vet` exiting 0, which it never does on this tree. A
gate written as an exit-code check is red before it starts and will be
"fixed" by deleting it.

Fold this into T4 with the round-4 findings; the plan is under review and must
not move under the reviewer.

## T1–T3 cannot move the firmware, so T4's size step is ceremony here

Every file the plan edits is a `_test.go`, a `testdata/` fixture, or
`scripts/vendor-compose-vectors.sh`. **Zero non-test Go files.** Test files are
not linked into `./cmd/controller`, so the flash/RAM figures are invariant
across T1–T3 by construction — no build needed to know it, and a measurement
that cannot differ proves nothing.

Keep the step only as a cheap guard against the plan's own scope claim being
violated: if the firmware size DOES move, a non-test file was edited and the
plan's "no normative Go behaviour changes" clause is broken. State it that way
in T4, so the number is read as a scope check rather than a performance one.

## Three plan inaccuracies the implementation exposed (2026-09-20)

Reported by the implementer, all independently re-measured by the controller.
None blocking; all fold into the plan after the whole-diff review returns.

**1. T4's scope check says "one non-test Go file". It is TWO.** Exporting
`validChecksum` for D1″ necessarily edits both `bip380/checksum.go` and its
call site `bip380/bip380.go`. Confirmed:

    $ git diff --name-only 95716e97..HEAD | grep '\.go$' | grep -v '_test\.go$'
    bip380/bip380.go
    bip380/checksum.go

A gate reading that sentence literally **fails on a correct implementation** —
exactly the shape this cycle has been finding all day. The firmware is
byte-identical with `bip380/` reverted, so the export reaches nothing on the
device and the invariance argument still holds; only the count was wrong.

**2. "The six record reads move to the loaders" — applying D5b's rule as
stated reaches TEN.** Measured: 11 loader call sites across 8 files outside
the fixtures files. The four extra are inert today and are precisely the
"fourth site" shape D5a warns about — so the rule found more instances than
the survey that motivated it, which is the argument for stating it as a rule
rather than a list of sites.

**3. A restore hazard the plan omits.** `git checkout -- md/testdata/` reverts
the *uncommitted* re-vendor, so mutation-restore by git during T3 silently
undoes the task. It bit the implementer once and was caught only because the
restored run still printed FAIL; later mutations were restored from a
filesystem snapshot verified with `diff -rq`. Worth a line in T3.
