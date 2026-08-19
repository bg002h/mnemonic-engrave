# Pin-regime differential — does md-codec behave differently across rust-miniscript revisions?

**Run 2026-08-18 by the controller (not an agent), during the Wallet Policy
brainstorm.** Every number below is pasted command output, not description.

## Why this was run

The constellation builds `md-codec` against **two different rust-miniscript
revisions**, depending on which binary you build:

| binary | source repo | rust-miniscript |
| --- | --- | --- |
| `md` | `descriptor-mnemonic` | registry **13.0.0** — no `[patch.crates-io]` |
| `mnemonic` | `mnemonic-toolkit` | git **`95fdd1c5`**, mid-master (`Cargo.toml:29-30`) |

Address derivation is *delegated* to rust-miniscript (`derive.rs:1-10`), so a
behavioural difference between the revisions would land on funds-relevant
output — two constellation binaries answering differently for the same wallet.
Nobody had run the check.

`git diff --stat 13.0.0 95fdd1c5 -- src/` in the local rust-miniscript clone is
**34 files, +2557 / −1453**, including a new `descriptor/wallet_policy/mod.rs`
(405 lines) and churn across `miniscript/`, `policy/` and `plan.rs` — so "the
revisions are too close to differ" was not available as a shortcut.

## Method

No file in any repo was edited. Cargo's `--config` injects the patch, and a
separate `--target-dir` keeps each build apart:

```sh
cargo test -p md-codec \
  --target-dir <scratch>/<rev> \
  --config 'patch.crates-io.miniscript.git="https://github.com/rust-bitcoin/rust-miniscript"' \
  --config 'patch.crates-io.miniscript.rev="<rev>"'
```

## Result 1 — behaviour: NO divergence found

```
BASELINE 13.0.0        : passed=461 failed=0 ignored=2 (suites=25)
PATCHED  95fdd1c5      : passed=461 failed=0 ignored=2 (suites=25)
PATCHED  ff4732e (#953): passed=461 failed=0 ignored=2 (suites=25)
```

Identical across all three, including the ~29 s `bitcoind_differential` suite.

**The bound on this result, which matters as much as the result.** This proves
no *covered* behaviour diverges. It does not prove addresses are identical for
shapes the corpus does not reach — and the corpus is weakest exactly where the
risk is highest: **13 of 15 `test_vectors::MANIFEST` entries carry
`keys: &[]`** (`test_vectors.rs:68-117`), so most vectors derive no address at
all. Keyed derivation across revisions is therefore barely exercised.

So this is a **weak green**. It becomes a strong one only after R3 puts keys
and addresses into the corpus — which is the same reason R3 exists.

## Result 2 — build: `md-cli` does NOT compile against either newer revision

This was not the question asked, and it is the more actionable finding.

| target | `95fdd1c5` | `ff4732e` (#953) |
| --- | --- | --- |
| `md-codec` (library) | **clean, 0 errors** | **clean, 461 tests pass** |
| `md-cli` (the `md` binary) | 2 errors | **2 errors** |

```
error[E0432]: unresolved import `miniscript::descriptor::WshInner`
   --> crates/md-cli/src/parse/template.rs:945:9
error[E0599]: no variant or associated item named `SortedMulti` found for enum `ShInner`
   --> crates/md-cli/src/parse/template.rs:931:18
```

Both are fallout from PR #915's `Terminal::SortedMulti` refactor.

**This corrects a claim carried in `mnemonic-toolkit/Cargo.toml:20-22`**, which
records that a 2026-06-13 spike found bumping to `ff4732e` "build-clean +
regression-free". That is true **for the toolkit**, which consumes only the
`md-codec` library. It is **false for `md-cli`**: the `md` binary does not
compile against `ff4732e` at all.

Consequence for the Wallet Policy cycle: advancing the pin to lift the
depth-≥2 taproot gate is **not** a free batching decision. It requires porting
`md-cli/src/parse/template.rs` off two removed miniscript APIs first. That cost
had not been priced anywhere.

## What this does NOT settle

- Whether `md` and `mnemonic` produce identical addresses for a *keyed*
  miniscript policy. Blocked two ways: the corpus has no keyed vectors for
  those shapes, and `md` cannot be built against the toolkit's revision to
  compare in the first place. R3 + the `md-cli` port unblock it.
- Whether the two revisions differ for shapes outside md-codec's admission set.
  Out of scope — md-codec cannot express them.
- Whether upstream plans a release carrying #953. Recon found no milestone.
