# The policy harness

Four programs that build wallet policies nobody wrote down, push them through
the whole seam, and ask independent implementations whether they agree.

## Why it exists

The md1 decoder is fuzzed four ways and the codec has property tests on its
primitives. The POLICY space had none of that: until this harness, nothing built
a wallet policy outside the 67-vector corpus, pushed it through compose → encode
to md1 → decode back → seat keys → derive an address → bundle a plate, and
compared what came out.

The corpus is 67 vectors written by hand over months. The harness generates
thousands, and the interesting ones are the shapes nobody thought to write down.

## The pieces

| program | what it is |
| --- | --- |
| `scripts/policy-generate.py` | generator + two-way differential: Rust primary vs the device, plus the round trip and the engrave leg. Needs no Bitcoin Core, so it runs anywhere and fast. |
| `scripts/policy-differential.py` | the three-way driver: Rust, the device, and Bitcoin Core on a throwaway regtest datadir. `--no-core` drops to two. |
| `scripts/policy-differential-selftest.py` | makes each leg wrong in turn and asserts the finding fires. A differential harness whose comparator cannot report a disagreement is a false-PASS machine that prints `agreed=1000` forever. |
| `cmd/policyprobe` (seedhammer fork) | the device leg. JSONL in, JSONL out. Derives through `gui.PolicyAddressAt`, the router the inspect screen calls. |

## What each leg checks

- **Round trip, first.** Does the card decode back to the template it was
  encoded from? An encoder that dropped a lock would still produce an address
  both sides agree on — they would simply agree about the wrong wallet.
  Template equality is the tighter invariant, so it runs before the addresses.
- **Addresses.** A disagreement is a funds-safety finding: two different
  addresses for one engraved card.
- **Asymmetric refusals**, counted separately. Usually a deliberate difference
  in what each side supports. F-513 is one that turned out to be real.
- **Engrave.** `me bundle` — the leg between "the codec accepts it" and "a plate
  exists". A policy can encode cleanly and still be unmintable (F-515).

## Running it

```sh
# two-way, fast, boundary locks, with the engrave leg
python3 scripts/policy-generate.py --count 1200 --seed 4711 --edges

# three-way without Core
python3 scripts/policy-differential.py --no-core --count 600 --seed 31337 \
    --policyprobe /path/to/policyprobe

# prove the comparator can still report a disagreement
python3 scripts/policy-differential-selftest.py
```

Both drivers exit non-zero on a disagreement, a panic or a broken round trip, so
a CI job can gate on them. Both are seeded, and both record the exact arguments
for every case: a finding that cannot be reproduced by hand is a rumour.

Bitcoin Core is **not installed**. `policy-differential.py` prints how to fetch
and verify it, and refuses a datadir outside `/scratch/code/shibboleth/.tmp`.

## What it has found

| | |
| --- | --- |
| F-512 | the device leg derived through one branch of the address router, reporting eight vendored vectors as device refusals that the device derives |
| F-513 | `md address` refuses three key-reuse policies whose own conformance files carry expected addresses; Bitcoin Core accepts two of the three |
| F-514 | the device derives a fundable address, unwarned, for a script Core calls `not sane: contains duplicate public keys` |
| F-515 | the 64-chunk wire cap is not implied by `ComposeMaxSlots = 32`, so a policy inside every documented limit can still be unmintable |

## What it does NOT cover, stated plainly

- Only the **address** is compared, not the script that would spend it. For
  these wrappers the address commits to the script, but that is an argument,
  not a check.
- Generation is over the **composer's** language, so it reaches the policies an
  operator can build on the device and not the wider set `md encode` accepts.
  A shape the composer cannot express cannot be generated here.
- Two of the three implementations descend from the same specification. Core is
  the only genuinely independent opinion, and it is the leg that needs a
  download.
- The device leg is the device's **code**, not the device. Nothing here runs on
  real hardware.
