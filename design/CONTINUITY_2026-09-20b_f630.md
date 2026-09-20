# Continuity — F-630, the descriptor gate (2026-09-20)

Resumed from `CONTINUITY_2026-09-20_composer_fable_r0.md` at its resume point 1.
**F-630 is CLOSED and SHIPPED** at fork main `a4246a2`, CI green, no bypass.

## What F-630 turned out to be

It was filed as "port md-codec 0.44.0's xpub-header rewrite into the fork".
**Three of its four prescribed steps did not survive measurement.**

- **There was no rule to port.** `bip380.Key.ExtendedKey()` (`bip380.go:97-109`)
  always took depth and child number from the origin it was given; a probe
  reproduced dm `b2c5d693`'s corrected descriptor byte for byte, all 3 xpubs
  across both chains. md-codec 0.44.0 fixed `assemble_origin_and_xkey`, which
  the fork has no analogue of.
- **0.45.0's four `precedence_*` cases were already in** — the keyless-cap test
  drives 8 cases and asserts all four kinds.
- What remained: a gate (the conformance test parsed `.chains[].descriptor`
  into a field it never asserted) and a re-vendor the script could only reach
  32 of 41 records with.

## What shipped

A two-tier descriptor gate over all 46 keyed records: **D1** (header agrees
with origin), **D1′** (the descriptor reduces to its own `template`), **D1″**
(BIP-380 checksum), **D2a/b/c** (key material and bracket fingerprint, bound to
the Go port's expansion), **D3** (the elided-origin divergence, both sides),
**D5g** (a pinned record is a fork-maintained fixture; only its header arm
relaxes). Corpus re-vendored to `b2c5d693`; the vendor script widened to the
whole keyed tier (246 files, 50 vectors) with `isComposeVectorFile`'s directory
scan widened alongside it. Fixture loading became a boundary: one fixtures file
per package owns the record loader, the card loader and the enumeration, and
`md/testdata/forkbuilt/` is a closed, hashed set.

Measured at the merged tip: gate **46/46 (43+3), 0 fail**; `go vet` 10
ArtifactDir and nothing else; `gofmt` the five-file baseline; `./md/ ./sysw/
./mk/` ok; **1374/1374** gui across 24 shards; re-vendor byte-identical to the
primary's selection.

## The finding worth carrying forward

**D1 alone would not have closed the class.** It walks `[fp/path]xkey` and
stops at the key, so six descriptor-only mutations passed the whole 1374-test
suite green — including a corpus-wide suffix regression across 92 of 92 chain
descriptors with addresses and ids untouched, and two funds-relevant shapes
(the quorum `multi(2→3)` and the script type `wsh→sh(wsh(`). An enumerated
field check cannot be finished; its residue is whatever you did not think of.
D1′ reduces the whole string instead and needs no clause per shape. See
`memory/assert-by-reduction-not-by-enumeration.md`.

Two lessons recurred *within* the cycle: "reports is not asserts" was folded
into the plan at r3 and came back in the implementation as a `t.Logf` count;
and the pin/record split produced the same Critical three times, at one call
site, then at three, then at the enumeration.

## Artifacts

Plan `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` (carries the three
inaccuracies the implementation exposed, at the top). Reports and briefs at
`design/agent-reports/f630-*` and `design/agent-briefs/f630-*`. Controller
measurements at `design/F630_CONTROLLER_MEASUREMENTS.md`.

Rounds: r0 adversarial 1C/7I/4M/2N → r1 fold-verify 1C → r2 boundary 0C/3I →
r3 implementability 2C/5I/5M/2N → r4 end-state 2C/1I/3M/1N → r5 closing 0C/2I
→ r5 fold-verify 0C/0I → implementation → whole-diff 0C/1I/3M/4N → fold →
pre-push verify 0C/0I/1M.

## Repo tips

| repo | tip |
| --- | --- |
| seedhammer (fork) | `a4246a2` main — F-630 shipped, CI green, no bypass |
| mnemonic-engrave | see `git log` — records for this cycle |
| descriptor-mnemonic | `b2c5d693` (unchanged) |
| mnemonic-toolkit | `53457147` (unchanged) |

## RESUME POINT

1. **The flash is still the operator's** and now supersedes the previous image:
   fork main moved `95716e97` → `a4246a2`, so the signed
   `bg95716e9` image no longer matches main. F-630 touched only tests, fixtures
   and one exported identifier — the firmware is byte-identical — so re-signing
   is optional, not required, for a device walk.
2. **F-629 and the docs batch** (F-624/626/627/628, F-631's fixture): move both
   operator runbooks into `demo/sh2/WALKS.md`.
3. **F-632** (the xprv forgery) is residue, secret-handling, non-gating — close
   it opportunistically with a version-byte assertion.
4. Toolkit doc-build drift remains its own cycle; do NOT make it green by
   re-pinning older tools.

**Standing:** the operator's directive for this cycle was "proceed
autonomously"; pushes and releases without asking, the flash never unattended.
