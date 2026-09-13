# Policy differential run

- **Seed**: `1`  (`--seed 1 --count 200 --indices 3` reproduces this run exactly)
- Generated: **200**   (minted a card: 184)
- Agreed — every available leg identical on every index: **166**
- Disagreed: **0** cases, in **0** distinct divergences
- Documented divergences only: **18** cases, in **1** class
- Refused before a card was minted: **16** (compose 0, encode 16)
- Wall clock: 12.8s; `md` invocations: 1544; Core RPCs: 716

## Legs
- **Rust** `/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md` — mainnet and regtest
- **Bitcoin Core** `/scratch/code/shibboleth/.tmp/bitcoin-31.1/bin` — regtest, datadir `/scratch/code/shibboleth/.tmp/policy-diff-regtest`, RPC port 18988
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

Wrapper distribution this run: `sh` 26, `sh-wsh` 21, `tr` 74, `wsh` 79

## Findings

Occurrences are collapsed by **signature** — the same divergence hit by several generated policies is one entry with a count, not N copies. Each entry carries the smallest policy that still reproduces it.

**No unexpected divergence.** Every case that minted a card produced identical scripts on every available leg, for all requested receive and change indices, except for the documented classes listed below.

## Documented divergences (not defects)

These are differences the implementations are *meant* to have. They are counted and named rather than silently dropped, so that widening the suppression list shows up in a diff.

### ACCEPTANCE_MISMATCH — receive: md derived addresses, Core refused the same descriptor

- Occurrences: **36** across 18 policies
- Smallest reproduction: `--wrapper wsh --path keyless,sha256=9d1a1ab55daae4a604e9121cce37d832b2c4dc344a99c472c2e7c2f3d3543f1a --path 1of1`
- Template: `wsh(or_i(sha256(9d1a1ab55daae4a604e9121cce37d832b2c4dc344a99c472c2e7c2f3d3543f1a),pkh(@0/48'/0'/0'/2'/<0;1>/*)))`

Why this is expected: `md encode --experimental` exists precisely to relax rust-miniscript's *"all spend paths must require a signature"* rule, which its own `--help` calls "a safety policy, not a language rule". Bitcoin Core enforces that same policy in `IsSane` and offers no flag to relax it, so a policy md will only encode under `--experimental` is one Core is expected to refuse. Only refusals carrying `witnesses without signature exist` **and** on a case that needed `--experimental` are classed here; every other Core refusal stays a finding.


## Refusals before a card was minted
These are policies the generator produced inside the composer's stated limits that some stage of the Rust leg declined. A compose refusal means the generator stepped outside a rule `validate()` enforces; an encode refusal usually means rust-miniscript's own resource or timelock rules, which the composer's bounds say nothing about.

| count | stage and message |
| --- | --- |
| 9 | `encode: md: codec error: encoding requires 65 chunks; max is 64 per spec §9.8` |
| 7 | `encode: md: template parse error: miniscript parse failed even with --experimental: Miniscript is malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits, repeated keys and timelock mixing still apply)` |

## Reproducing
```sh
/scratch/code/shibboleth/me-worktrees/policy-harness/scripts/policy-differential.py \
  --seed 1 --count 200 --indices 3 \
  --report <path>
```

Bitcoin Core is started on demand against the scratch datadir and stopped when the run ends. `--no-core` drops that leg for a fast iteration loop; `--keep-core` leaves the node up between runs.

