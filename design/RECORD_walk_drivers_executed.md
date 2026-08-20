# RECORD — the three unexecuted walk drivers were run, 2026-08-19

Closes P1 item 6 of `RECON_constellation_journeys_synthesis.md`: *"Establish
whether the 3 `seedhammer` walk drivers with no execution evidence can run at
all — a gate that has never run is a hypothesis."*

**Answer: all three run, and the one with a negative control refuses correctly.**

## What the recon could and could not say

The `seedhammer` inventory found five `cmd/emu/walk_*.js` drivers, of which only
two (`walk_trace_a`, `walk_trace_b`) had ever produced a persisted gate record.
For the other three it recorded **UNKNOWN, not "never run"** — no gate record, no
`walk.json`, no doc reference, but a human could have run one in a browser and
simply not minted a record. That was the honest limit of a read-only sweep.

It is no longer unknown. They were driven, in a real browser, against a freshly
built `emu.wasm`.

## Method

`sh cmd/emu/build.sh` (Go 1.26.3 → `emu.wasm`, 10,017,804 bytes), served from
`cmd/emu/` on `127.0.0.1:8791`, driven through Playwright by importing each
module and calling its own documented entry point — the same
`const w = await import("./walk_x.js"); await w.run()` each file's header
prescribes. The emulator page was reloaded between walks so no walk inherited
another's device state.

**The wasm was rebuilt first, deliberately.** The committed artifact was a day
old and untracked; a walk against a stale binary tests the binary, not the tree.

## Results

| driver | wall | outcome |
| --- | --- | --- |
| `walk_build_policy.js` | 14s | `ok: true`, 8 needles proven, 4 cards gathered, 2 open slots |
| `walk_s3_nested.js` | 233s | `ok: true`, 11 needles, **`plateCount` 9 == `censusClaim` 9** |
| `walk_s4_gate.js` (pass arm) | 193s | `ok: true`, 9 needles, `censusClaim` 7 |
| `walk_s4_gate.js` (**fail arm**) | 28s | **refused, correctly — see below** |

All three module loads succeeded and all export `run`.

## The negative control is the real result

A green happy path proves less than its own negative control, and `walk_s4_gate`
ships one: `w.run({ arm: "fail" })` assigns a card whose key does **not** derive
from the operator's seed, and SPEC 4.3 requires the device to *"verify key can be
derived from seed and if not, fail loudly."*

It failed loudly:

```
needle proven:  "Key does not match seed"
refusal: { namesSlot: true, saysNothingEngraved: true,
           saysSuppresses: true, namesHostRoute: true }
censusClaim: -1          ← nothing engraved, which is the correct census for a refusal
```

So the seed↔key gate **bites on real firmware, driven through the real GUI**, and
the refusal names the offending slot, states that nothing was engraved, states
that it suppresses, and names the host route. That is a funds-safety gate
demonstrated rather than assumed.

## What this does NOT establish

- **No gate record was minted.** `cmd/gaterecord` shells out to installed
  `ms`/`mk` Rust binaries to mint `oracle/gaterecords/*.{record,walk,expect}.json`,
  and that step was not run. These runs prove the drivers execute and what they
  return; they are not a substitute for a recorded, independently-checked gate.
- **Still not in CI.** Nothing here changes finding 6 of the recon: no
  `walk_*.js` driver is executed by CI, which runs `GOOS=js GOARCH=wasm go vet
  ./cmd/emu/` — a compile check only. Running a walk still requires a human (or
  a browser automation harness like the one used here).
- **`walk_build_policy` stops short of the engrave census** — it returned
  `plateCount: null` and `censusClaim: -1` with `params.engrave: false` while
  `params.expect` is `"engrave"`. Not investigated; recorded because it is the
  one result of the four that looks internally inconsistent.

## Why it matters that this took a browser

The recon's finding 6 is that these drivers are human-gated, and that is exactly
why three of five had no execution evidence: **the cost of running them is what
kept them unrun.** The barrier turned out to be about ten minutes of automation,
and behind it sat a working demonstration that the device refuses a mismatched
key. A gate nobody can afford to run is a gate nobody runs.
