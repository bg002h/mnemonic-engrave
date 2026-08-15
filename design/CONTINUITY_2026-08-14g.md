# Continuity — 2026-08-14g, S0b is GREEN; next is S1

Supersedes `CONTINUITY_2026-08-14f.md`, which said "implement S0b". S0b is
built, all three mechanisms are exercised and every one has been **seen to go
red**. Closing it cost two releases in the primary repos, for a reason 14f could
not have known. Read this one.

## Where to start

**S1 — "the payload supplies the whole cosigner set"**,
`IMPLEMENTATION_PLAN_multisig_build_repair.md` §3 (line 581). Read it there. Its
eight tests are written out in the plan, and the `0..n` ruling is already folded
into them.

Before writing code, **read F-175** (below): S1 is the first stage that cannot
emit a gate record, and it needs an answer, not a workaround.

**This is implementation against a plan, so work solo and verify inline.**

## State

All four repos **clean and pushed**.

| | | |
| --- | --- | --- |
| fork `seedhammer` | `main` | `c94c135` |
| `mnemonic-key` | `main` | `a38a908` |
| `mnemonic-secret` | `master` | `ddfa497` |
| `mnemonic-toolkit` | `followup/p2wsh-binding-oracle` | `6bd944bf` |
| `mnemonic-engrave` | `master` | this commit |

Fork gate: `go test ./...` **51 ok / 0 fail**, `go vet` 6 `ArtifactDir`
(baseline), `gofmt` clean, tinygo flash **1,342,468** (unchanged — nothing in
S0b reaches `./cmd/controller`, proven with `go list -deps`).

**Push note:** `mnemonic-key` and `mnemonic-secret` pushes reported *bypassing*
required status checks — both repos have `enforce_admins: false`, so admin
pushes skip them automatically. That is information, not something that was
invoked. The fork's `main` has no protection at all. Only this repo's `master`
needs the `ci/staging` ritual.

## What S0b actually is, now that it exists

Three mechanisms, in `cmd/emu` and `oracle`:

1. **`cmd/emu/walk_build_policy.js`** — reaches the Build-policy cosigner gather
   via `Engrave Multisig`, proving it with needles that
   `cmd/emu/needle_test.go` machine-checks as single-site. Asserts
   `shNFC.presented() === 0` at entry and at the gather.
2. **`oracle.DeriveExpected`** — computes the artifact set from the recorded
   input tuple. `plates = 6` is gone; six now falls out of three seeds × two
   chunks, computed by `mk`.
3. **`oracle.CompareCensus`** — byte-for-byte, in order, against S0's record.

Run it:

    # fresh port; the browser caches emu.wasm
    nix develop --command ./cmd/emu/build.sh
    import("./walk_build_policy.js").then(w => w.run()).then(r => { window.__w = r });
    # ~7 s, engraves nothing

    nix develop --command go test ./oracle/ -run 'S0Census|Compare|Derive'

## The thing 14f could not have known

**`mk encode` was non-deterministic.** It drew `chunk_set_id` from the OS CSPRNG
on every call, so three runs on identical inputs emitted three different cards.
Byte-identity against a chunked mk1 — which S2 and S5 both gate on — was
**permanently unsatisfiable**, and no amount of harness work in the fork could
have fixed it.

Per the Rust-primary rule it was fixed upstream first, and it turned out to be a
**conformance fix, not a wire-format change**: SPEC §2.5 already required
encoders to "reuse the same value for all subsequent re-encodings of the same
card", and a stateless encoder cannot do that from entropy. mk-codec 0.5.0
derives it from the payload, matching md-codec's existing rule and the fork's Go
port, which had always been deterministic. **The corpus did not move** — all 41
vectors already pinned their ids explicitly.

**Then a second gap, and this one was a real product defect.** `ms derive
--template` offered single-sig templates only and accepted no literal path, so
**seed → multisig account xpub had no oracle at all**: the one tool that turns a
seed into an account xpub could not serve the format the constellation exists to
back up. ms-cli 0.15.0 adds BIP-48 `bip48-p2wsh` / `bip48-p2sh-p2wsh`, so an
operator names their **script type** rather than knowing that native segwit
multisig lives at `m/48'/0'/0'/2'`. A bare `bip48` is refused (two script types
are registered; guessing would put a cosigner key at a path nobody chose), and
there is no `bip48-p2tr` because BIP-48 registers none.

The chain is now entirely primary-toolchain, with nothing re-implemented:

    seed words   --ms derive--> master fingerprint + account xpub
    account xpub --mk encode--> the mk1 chunk(s)

## Open follow-ups

- **F-169, F-170, F-171, F-174** → **RESOLVED** (S0b). Each entry carries its own
  resolution note and its mutation evidence.
- **F-175** → **S1, gating, NEW.** An artifact-free stage cannot produce a gate
  record: `ParseWalk` refuses an empty census, and the plan says S1 "ends at a
  screen, not an engrave". Measured, not reasoned about. Three options are
  written out in the entry; none is chosen. Nothing is red today because
  `TestS0GateHasARecord` demands a record for **S0 only** — which is exactly why
  it would go unnoticed until S1's gate.
- **F-172** → S3. The walk must pick "Full policy md1" or the restore-doc gate
  has nothing to read.
- **F-173** → RULED (`0..n`); **S1** owns building to it.
- **`toolkit-mk-codec-0-5-determinism-note`** (in `mnemonic-key/design/FOLLOWUPS.md`)
  → blocked on a toolkit dependency bump. Its "mk1 re-encode is NOT
  string-deterministic" note goes false when it moves off the published
  mk-codec 0.4.1; it is TRUE for the code it compiles today, so it was
  deliberately left alone.
- F-158 premise STALE; F-160 census gap; F-166 own cycle. None block.

## Traps — the 14f list still applies, plus these

- **`md` is a shell alias for `mkdir -p`** on this machine. Harmless from Go
  (`exec.Command` uses no shell, and the oracle resolves absolute paths), but a
  gate command run **by hand** with a bare `md` silently runs coreutils.
- **`cargo fmt --check` through a pipe reports the PIPE's exit code.** `… | head`
  said 0 while the true status was 1. Redirect to a file and read `$status`.
- **A fmt/lint failure may not be yours.** `mnemonic-secret` HEAD is fmt-dirty
  under the pinned 1.95.0 toolchain in `mlock.rs` — pre-existing, deliberately
  not bundled into an unrelated commit. Attribute before fixing.
- **Re-pinning an oracle invalidates every gate record** ("re-walk, do not edit
  it"). But a re-walk is only needed when the DEVICE's output could have moved;
  re-anchoring a saved walk to new pins via `gaterecord -force` is the sanctioned
  path when only the comparison source changed.
- **ChoiceScreen rows are 24px apart, centred on y=160**:
  `rowY(i, n) = 160 - (n-1)*12 + i*24`. Measured on the 2-row front door and
  confirmed on the 4-row n-picker. A wrong row silently picks a different
  parameter, so `choose()` re-asserts with a needle every time.
- **The Build-policy and Engrave Bundle gathers are character-for-character
  identical.** The title says "Engrave Bundle" in both. Only a single-site needle
  tells them apart.
- **A version string in a pin can be a lie about its own commit.** The old `ms`
  pin named `bf77f89` + "ms 0.14.0" while that commit's source declared 0.14.1 —
  so the binary was never built from the commit it named.

## Toolchain

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...          # 51 ok, 0 fail
    nix develop --command go vet ./...           # 6 ArtifactDir = baseline
    nix develop --command gofmt -l ./

`FOLLOWUPS.md` reports **17** unresolvable citations — all pre-existing `.rs`
cross-repo references; that count is the baseline, not a regression.

Push `master` via `ci/staging` (see this repo's `CLAUDE.md`); the fork's `main`
and the primaries push directly.
