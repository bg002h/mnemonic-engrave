# F-674 items 1 and 2: implementation report

**Status: DONE.** Engrave branch `f674-records` (worktree `me-worktrees/f674`):
`86d63288` (item 1) and `f243bce9` (item 2), on top of the parallel agent's
`b729bdb1`. The fork is **unchanged** (`f674-nunchuk` is still at `0287e3a`),
because the copy did not need to change. Nothing has been pushed or merged.

## Item 1: the Nunchuk sentence is TRUE for same-seed seatings. It stays.

**Method.**

- **Wallets.** They were built through the fork's own composer code at `0287e3a`, not by hand. `zz_f674_measure_test.go` is a scratch test dropped into a *copy* of the fork. It uses:
  - `composerPresets(tr)` for the shapes;
  - `composerSeedDerive` for the seating, so each repeated seed gets its own account;
  - `composerArtifactsFor` to compose and bind with `UnspendableLiana`.
- **Premise checks.** The test asserts that `composerSharedSeedInPath` and `composerLianaRefusesSeating` fire for every wallet.
- **Rendering.** `md descriptor` 0.20.1 renders each wallet. `measure.py` asserts that the internal key equals Liana's recipe over the leaves in slot order.
- **Harness.** `fableharness` is the coordinator-compat recon's binary: libnunchuk `a7cfb49`, Nunchuk 2.1.1's pin, run with `HOME` sandboxed. It calls `ParseWalletDescriptor`, the app's import entry point.
- **Wallets measured.** Four are the natural seatings, with the demo seed `b8688df1` in two or more slots of one path: kofn all-B, kofn B,B,A,zoo, tiered B,B,A,zoo, and tiered all-B. Four more are composer seatings chosen so that slot order puts the leaves in ascending order. They cover kofn and tiered, with the shared seed in path 1 and in path 2.
- **Controls.**
  - For every accepted wallet, Nunchuk's receive and change 0..2 are compared with `md address`.
  - Every wallet gets a PR-1746 twin: the same leaves with Nunchuk's own sorted-unique internal key. If the twin is accepted, a refusal came from key order and not from the repeated seed.

**Result** (`verdicts.txt`, mismatches: 0):

| wallet | leaves | Nunchuk | addresses vs md | PR-1746 twin |
| --- | --- | --- | --- | --- |
| kofn-allB, kofn-2B, tiered-2B, tiered-allB | unsorted | REFUSE ("Failed to verify wallet descriptor", -1017) | n/a | ACCEPT |
| kofn-3B-sorted, kofn-2B-sorted, tiered-3B-sorted, tiered-2Bp2-sorted | sorted | ACCEPT, `DISABLE_KEY_PATH`, all 4 signers kept (3 share xfp `b8688df1`), `check_valid=ok`, `sanitize_signers=4` | EQUAL | ACCEPT |

The repeated seed changes nothing, and key order alone decides the outcome. "Nunchuk imports it only when the keys happen to be in sorted order" is therefore accurate as written, so the fork copy and the spec blockquote are unchanged. The spec gets a provenance paragraph under the same-seed body in §8y, citing the evidence.

**Evidence.** `design/evidence/f674-nunchuk-same-seed/` contains `run.sh <fork-checkout>`, the generator test, `measure.py`, a copy of `harness.cpp` with its two CMake lines, and the outputs. Running `run.sh` against `0287e3a` reproduced every output byte for byte.

## Item 2: the `liana-same-seed` arm is wired in

`design/journeys/capture_composer.py` changes:

- a new `--arm liana-same-seed` choice;
- an `EXPECTED` entry with its six fixed shots: c00a, c01, s01, s03-mapping-p0, s05-consent-p0 and s06;
- a leg whose expect is `read_keyed()["A"]`, since the arm compares only the payload digest;
- a branch in the summary printer, because the arm has no census and no strings;
- `--arm both` now includes it. It is a 22-second copy walk and the only walk that reaches the same-seed consent body.

`--emit-expect` and `--check-expect` are unaffected, because the arm reuses keyed form A.

Walks, all run against the fork `0287e3a` emulator:

| run | result |
| --- | --- |
| `--arm both` | rc=0, 5 legs: keyed-A, keyed-B, keyless, liana, liana-same-seed. "all legs matched the host." |
| `--arm liana-same-seed` | rc=0 |
| scratch fork copy with `composerLianaRefusesSeating` forced to `return false && …` (applied once, verified by grep) | rc=1: "F-671: the consent claims Liana imports a wallet the mapping review says it refuses" |
| the same copy, restored and touched, `cmp`-identical to the worktree | rc=0 |

## Concerns

- **`transcript_composer.sh` has a stale gate.** "md is 0.19.0 (the first with --unspendable)" fails with md 0.20.1, and the script exits 1. Every other gate passes and all the host artifacts are written. I did not touch the file because it is outside my scope. It should read "at least 0.19.0", or pin 0.20.1.
- **The harness is still uncommitted scratch** at `.tmp/fable-nunchuk-lib/build/fableharness`. The evidence directory now carries `harness.cpp` and its build lines, but not the libnunchuk build itself.
- **The same-seed key-path *row*** ("Liana key: Bitcoin Core imports it. SAME SEED, SAME PATH: Liana will refuse it.") says nothing about Nunchuk, while the unseated row says "Nunchuk only by chance". Nothing in it is false, but the two rows no longer match. I left it alone because it is not in F-674.
- **Scope of the measurement.** The harness measures the library's import path, not a Nunchuk app UI walk. That is the same scope as the R-1 and recon measurements it extends.
