# Policy differential run

- **Seed**: `31337`  (`--seed 31337 --count 600 --indices 3` reproduces this run exactly)
- Generated: **600**   (minted a card: 548)
- Agreed — every available leg identical on every index: **548**
- Disagreed: **0** cases, in **0** distinct divergences
- Documented divergences only: **0** cases, in **0** classes
- Refused before a card was minted: **52** (compose 0, encode 52)
- Wall clock: 19.7s; `md` invocations: 3392; Core RPCs: 0

## Legs
- **Rust** `/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md` — mainnet and regtest
- **Bitcoin Core** — disabled (`--no-core`)
- **Device** `/scratch/code/shibboleth/.tmp/policyprobe-fixed` — mainnet only

### Which comparisons are on which network
The device is mainnet-only; Core here is on a throwaway regtest datadir and will not parse a mainnet `xpub`. So:

| comparison | Rust asked for | other leg answers in | compared as |
| --- | --- | --- | --- |
| Rust vs device | mainnet (`bc1…`) | mainnet (`bc1…`) | normalised scriptPubKey |
| Rust vs Core | regtest (`bcrt1…`) | regtest (`bcrt1…`) | normalised scriptPubKey |
| Rust vs itself | mainnet and regtest | — | normalised scriptPubKey |

`normalize_address` (scripts/policy_keys.py) discards the network and keeps the witness program or script hash, so `bc1q<prog>` and `bcrt1q<prog>` compare equal. The third row is a check in its own right: a network flag must never change the script, and `RUST_NETWORK_VARIANCE` fires if it does.

## Generator bounds
Read at runtime from `/scratch/code/shibboleth/seedhammer/md/compose.go`:

- `ComposeMaxPaths` = 8
- `ComposeMaxKeysPerPath` = 9
- `ComposeMaxSlots` = 32

Varied per case: wrapper (`wsh`, `tr`, `sh-wsh`, `sh`), path count, k-of-n per path, lock kind (none / `older=N` blocks / `older=Nu` units / `after=H` height / `after=Tt` time), `sha256` hashlock presence, `unsorted`, and keyless paths.

Wrapper distribution this run: `sh` 66, `sh-wsh` 73, `tr` 195, `wsh` 266

## Findings

Occurrences are collapsed by **signature** — the same divergence hit by several generated policies is one entry with a count, not N copies. Each entry carries the smallest policy that still reproduces it.

**No unexpected divergence.** Every case that minted a card produced identical scripts on every available leg, for all requested receive and change indices, except for the documented classes listed below.

## Documented divergences (not defects)

None encountered this run.

## Refusals before a card was minted
These are policies the generator produced inside the composer's stated limits that some stage of the Rust leg declined. A compose refusal means the generator stepped outside a rule `validate()` enforces; an encode refusal usually means rust-miniscript's own resource or timelock rules, which the composer's bounds say nothing about.

| count | stage and message |
| --- | --- |
| 30 | `encode: md: template parse error: miniscript parse failed even with --experimental: Miniscript is malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits, repeated keys and timelock mixing still apply)` |
| 12 | `encode: md: codec error: encoding requires 65 chunks; max is 64 per spec §9.8` |
| 6 | `encode: md: codec error: encoding requires 66 chunks; max is 64 per spec §9.8` |
| 4 | `encode: md: codec error: encoding requires 67 chunks; max is 64 per spec §9.8` |

## Reproducing
```sh
/scratch/code/shibboleth/mnemonic-engrave/scripts/policy-differential.py \
  --seed 31337 --count 600 --indices 3 \
  --report <path>
```

Bitcoin Core is started on demand against the scratch datadir and stopped when the run ends. `--no-core` drops that leg for a fast iteration loop; `--keep-core` leaves the node up between runs.

