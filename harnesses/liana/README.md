# Liana harness — drives Liana's own importer over our descriptors

**Why this is committed.** The coordinator-compatibility design
(`design/DESIGN_coordinator_compatibility.md` §3, mechanism 3) requires every
coordinator harness to be committed and runnable. Before 2026-09-20 this
program lived only in `/scratch/code/shibboleth/.tmp`, outside every repo —
which meant the Liana evidence could never be regenerated, only trusted. That
is the defect the fable architect review raised as C-4, and it is why the
registry could have grown refusals forever and never a measured positive.

It calls `LianaDescriptor::from_str` — the same entry point Liana's GUI uses
behind **"Import the wallet"** — so an OK here is Liana's own answer, not an
inference from reading its source.

## Run it

The `liana` crate is not vendored; point `Cargo.toml` at a checkout of
<https://github.com/wizardsardine/liana> at the tag you are measuring. The
layout moved between the versions we have measured:

| tag | date | path in Cargo.toml |
| --- | --- | --- |
| `v8.0` | 2024-11-08 | `<checkout>` |
| `v15.0` | 2026-07-31 | `<checkout>/liana` |

```sh
git clone https://github.com/wizardsardine/liana <checkout>
git -C <checkout> checkout v15.0
# point Cargo.toml's liana path at <checkout>/liana, then:
CARGO_TARGET_DIR=/some/scratch/target cargo build
./target/debug/liana-harness parse \
    < ../../design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl \
    > out.jsonl
```

`parse` reads JSONL (`{"name","variant","desc"}`) and prints one JSON line per
input: the verbatim error, or the inferred primary/recovery paths and the
first three receive and change addresses.

The harness source compiled against **both v8.0 and v15.0 unmodified**, which
is itself a measured fact about `from_str`'s API stability.

## What has been measured with it

| tag | evidence file |
| --- | --- |
| v8.0 | `design/evidence/composer-fable-r0/fable-liana-parse-out.jsonl` |
| v15.0 | `design/evidence/composer-fable-r0/fable-liana-parse-out-v15.jsonl` |

Inputs for both: `fable-liana-parse-in.jsonl`, 289 descriptors.

**Result of the v8.0 -> v15.0 re-measurement (F-633), seven majors apart:**

- 289 of 289 verdicts identical
- 73 of 73 accepted policies: identical inferred policy AND identical addresses
- 206 of 216 refusals: identical message
- **10 refusals changed message and none changed verdict** — exactly the two
  key-less shapes (`keyless-hash-path-wsh`, `keyless-hash-older-path-wsh`) at
  5 variants each, all moving from the generic *"Descriptor is not compatible
  with a Liana spending policy."* to the specific *"Miniscript error: 'All
  spend paths must require a signature'."*
