# Continuity — 2026-08-15d: S5.0 is CLOSED. Next is S5 proper, the stage the plan exists for

Supersedes `CONTINUITY_2026-08-15c.md`. Read this one.

## STATE — everything is pushed and green

| repo | branch | head | unpushed |
| --- | --- | --- | --- |
| fork `seedhammer` | `main` | `84a4f4a` | 0 |
| `mnemonic-engrave` | `master` | `4cef1d0` | see below |
| `descriptor-mnemonic` | `main` | `89ab0f62` | 0 |
| `mnemonic-secret` | `master` | `6fdfd36` | 0 |
| `mnemonic-toolkit` | `master` | `27a68e9f` | 0 |
| `mnemonic-key` | `main` | `8dc5dcb` | 0 |

**Fork baseline at `84a4f4a`** — all judged **unpiped, on true exit codes**:

    go test ./... -count=1     exit 0 · 51 ok / 0 FAIL
    gofmt -l ./                exit 0 · 0 files
    go vet ./...               exit 1 · 40 findings · 0 outside _test.go
    ./scripts/oracle-live.sh   exit 0 · 7 discovered, 7 ran

**Two of those read wrong and are not regressions.** `go vet` exiting **1** with
40 findings **is** the clean baseline. And **`GOCACHE` must be cold** on both
sides of any vet comparison — a warm cache reports exit 0 with no output, which
is where the long-lived bogus "6 ArtifactDir" figure came from.

## STAGES — eight of nine accounted for

S0 ✅ · S0b ✅ · S1 ✅ · S2 ✅ · S3 ✅ · S3b ✅ · S4 ✅ · **S5.0 ✅** ·
**S5 ← NEXT** · S6 (hardware).

**An operator still cannot build a wallet policy on the device.** That is S5.

## S5 — what it is and what gates it

Plan §3, `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` line **1137**.
Read §0 first (the rulings) and the whole S5 section. **The plan is FROZEN from
S1 on (§0.3)** — implement it, do not redesign it.

**Gate, and it is the strongest in the plan:** Trace B completes with a correct
descriptor **by test AND by emulator walk**, and the §4.5 comparison extends to
**every mk1 and EVERY ms1, byte for byte, against the current primary** — each
ms1 equal to `ms encode --hex <that master's entropy>`, each mk1 byte-identical.

**The mint has never run.** S5.0 proved the built-policy `ExpectKind` *refuses*
correctly; nothing has yet proven it *mints* correctly from a real build walk.
Per lens-closure, **S5 cannot close before that mint has executed.**

**Inherited, non-optional:**

- **F-185** — a modal's body can scroll off the first frame with no affordance,
  so a required instruction is present in the string and absent from the screen.
  S5 owns the engrave tail's screens. Every content assertion in `gui` checks the
  string *submitted*, not the pixels *drawn*.
- **The four prose constraints** in the plan's S4 section are normative for S5's
  screens too: no spec language on screen; every comparison the device asks for
  must be one the operator can *perform*; a FAIL screen must not make silencing
  the gate the obvious next move; state the plate count before the tail and the
  inventory after.

## WHAT S5.0 BUILT — do not rebuild or re-review it

- `ms` pin at **`ms-cli-v0.16.0`** (tag target `d49d5c0`), S0 re-anchored via
  `gaterecord -force` over the **saved** walk; census and digests byte-unchanged.
- **Built-policy `ExpectKind` as TWO kinds** — `built-policy-full` /
  `built-policy-watch` — not a mode field: a missing kind refuses, a missing
  field defaults.
- `ArtifactKindsFor` returns kinds in the device's **engrave order**
  (`ms1, mk1, md1` full; `mk1, md1` watch); `CompareCensus` is order-sensitive by
  design, and `CheckArtifactShape` binds each `Artifact.Kind` to its string's
  prefix.
- **Inline per-`@N` origins** — `@i/48'/0'/x'/2'/<0;1>/*`. `--path` is gone.
- `scripts/oracle-live.sh` **cannot pass vacuously**: the tagged-test list is
  derived from the tree (`grep -rlE '^//go:build\b.*\boraclelive\b'`), the run is
  checked by **set difference on names**, and the `-update` mint is anchored.

## md's TEMPLATE SYNTAX — the trap that cost the most today

md takes a key's origin **AFTER** the placeholder:

    wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))

The **descriptor** form `[fingerprint/path]@0/…` is NOT md template syntax. Two
further facts, both measured: md's template parser **rejects `h` notation**
(while `--path` accepts it) and misattributes the error to the multipath; and
`pathRe` makes the BIP-48 script_type level optional, so `m/48h/0h/0h` parses.

## THE LESSON OF THE DAY, in one line

**A check that cannot fail is worse than no check, and a mechanism written to
prevent that can itself have it.** Three times today a guard passed while the
thing it guarded did not run — twice inside my own fixes. Whenever you add a
gate, prove it RED before believing it green, and prove the *right* red: a
round-trip assertion catches a DROP and a FLATTEN and silently misses a SWAP.

## OPEN, NOT BLOCKING

**F-186** (half withdrawn — md *can* encode divergent origins; the `internal:`
error is fixed in `descriptor-mnemonic` `11b01a9e`) · **F-187** (document md's
template origin syntax in the toolkit manual) · **F-179/F-183** resolved ·
**F-184** (needle uniqueness counts comments) · **F-180** · `md compile` emits
`[fp/48h]@0` templates `md encode` now refuses.

**Environment, verified twice:** the fork requires **Go 1.26**
(`testing.T.ArtifactDir`) while `go.mod` declares `1.25.10`; under 1.25 the `gui`
package fails to build. Worth a follow-up, blocks nothing today.

## TOOLCHAIN

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    nix develop --command go test ./...
    nix develop --command ./cmd/emu/build.sh   # go test does NOT compile the emulator
    ./scripts/oracle-live.sh                   # live oracle checks, by name

Emulator: serve `cmd/emu` on a **FRESH port** (the browser caches `emu.wasm`) and
**prove the rebuild by its byte size** — a size delta is the evidence the binary
under test is the new one.

`md` is a shell alias for `mkdir -p`; invoke pinned binaries by **absolute path**
(`~/.cargo/bin/{md,mk,ms}`). `gh` needs `--repo bg002h/<name>` in the fork, and
`head_sha` queries need the **full 40-char SHA** or they silently return nothing.

## PUSHING

**All five repos build `ci/**`.** Push protected branches by letting the SHA earn
its check first — a status check binds to a COMMIT SHA, not a branch:

    git push origin <branch>:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/<name>
    git push origin <branch>          # no bypass message = satisfied
    git push origin --delete ci/staging

The fork's `main` is **unprotected** — plain push, no staging. Required contexts:
`mnemonic-engrave` → `test (rust + go)`; `mnemonic-key` → `build (stable on
ubuntu-latest)`; `mnemonic-secret` → four; `mnemonic-toolkit` → three;
`descriptor-mnemonic` → two. **They differ per repo; copying a sibling's block
waits forever on a check that never reports.**

**`enforce_admins: false` is DELIBERATE** — the operator's own escape hatch,
ruled 2026-08-15. **Never propose flipping it.** The no-bypass rule binds agents,
not the human.
