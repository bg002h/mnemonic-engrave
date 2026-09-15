# R0 round 2 — fold verification of P1 R0 round 1, `descriptor-mnemonic` branch `hashkinds-p1`

**Scope:** fold commit `496b0767` against round-1 report
`design/agent-reports/plan-hashkinds-P1-R0-round1.md`, plus spec-correction commit
`507b0f46` in `mnemonic-engrave`. Two questions only: did the fold close each
round-1 finding, and did it introduce a new defect. Not a fresh audit.
**Toolchain:** `cargo 1.97.0-nightly` reported by `cargo --version` after
`export PATH=/home/bcg/.cargo/bin:$PATH` — this is NOT the 1.85.0 the round-1
brief expected, but it is the same binary round 1 itself used to produce its
1313-test baseline (no toolchain change occurred; the version string differs
from the round-1 report's citation, immaterial to any measurement below since
every number was independently re-derived, not read off the report).

**Verdict: GREEN — 0 Critical, 0 Important, 0 new Minor/Nit.**

Both round-1 Importants are closed by construction, verified by building and
running the code, not by reading it. No new defect found.

---

## I-1 — CLOSED

**Claim:** `Eq`/`Hash`/`PartialOrd`/`Ord` are now hand-written over
`(kind, digest())` at `crates/md-codec/src/compose/mod.rs:216-250`, making the
alloc-gate padding unobservable, with a new test
`the_padding_is_unobservable_through_eq_ord_and_hash`.

**Verified by construction**, not by reading the impls. Wrote a scratch probe
`crates/md-codec/tests/zzprobe_r2.rs` (deleted before finishing; see Worktree
state) testing the full contract round 1's brief asked for:

- **Reflexive / symmetric / transitive `Eq`** over a mixed-kind, mixed-padding
  set of 7 locks — all pairs and triples checked. **PASS.**
- **`Hash` consistent with `Eq`.** Built pairs/triples with identical
  `(kind, visible digest)` and different padding tails, and a 24-member sweep
  across all four kinds × six padding bytes: every equal pair hashed equal via
  `DefaultHasher`. **PASS.**
- **`Ord` consistent with `Eq`, total, antisymmetric, agrees with
  `PartialOrd`.** For all pairs in a 28-member sample: `a == b` iff
  `a.cmp(b) == Equal`; `a.partial_cmp(b) == Some(a.cmp(b))`;
  `a.cmp(b) == b.cmp(a).reverse()`; a full sort is monotonic and idempotent.
  **PASS.**
- **Different kinds, identical bytes** — `Sha256`/`Hash256`/`Ripemd160`/`Hash160`
  built from the same 32 bytes are pairwise unequal and sort deterministically
  (twice, same order): `Sha256 < Hash256 < Ripemd160 < Hash160` — enum
  declaration order, matching the derived `Ord` on `HashKind` itself (unchanged
  by this fold). **PASS.**
- **`HashMap`, `BTreeMap`, `HashSet` keyed on padding-differing `HashLock`s** —
  three keys built from the same `(kind, visible digest)` with padding
  `0x00`/`0xFF`/`0x5A` collapse to one entry in all three collection types, and
  the last insert wins the value. **PASS.**

```
running 5 tests
test btreemap_and_hashmap_dedup_padding_differing_keys ... ok
test different_kinds_identical_bytes_compare_unequal_and_order_deterministically ... ok
test eq_reflexive_symmetric_transitive_over_a_set ... ok
test hash_consistent_with_eq ... ok
test ord_total_antisymmetric_and_consistent_with_eq_and_partialord ... ok
test result: ok. 5 passed; 0 failed
```

**Negative control on my own probe** — restored the pre-fold derive line
(`#[derive(... PartialEq, Eq, Hash, PartialOrd, Ord)]` over the raw `[u8; 32]`)
and reran: 2 of 5 probe tests correctly failed (`hash_consistent_with_eq`,
`btreemap_and_hashmap_dedup_padding_differing_keys`), each on the exact
padding-differing pair, proving the probe is not vacuous. Restored the fold's
code from a `cp` backup immediately after.

**Also mutation-reproved the fold's own test**, independent of the probe:
restored the derive line again and ran only
`the_padding_is_unobservable_through_eq_ord_and_hash` — reds on
`"padding must not make two equal locks unequal"`, matching the fold commit
message's claim verbatim. Restored again.

**Grep for anything relying on the removed derives.** `HashLock` appears in
`crates/md-cli/src/cmd/compose.rs`, `crates/md-codec/src/compose/presets.rs`,
`crates/md-codec/tests/compose_lowering.rs` and `compose_support.rs` — every
use is construction (`HashLock::new`) or a struct field, never a sort, dedup,
`assert_eq!` on the type, or a `HashMap`/`BTreeMap`/`HashSet` key. No existing
code depended on the old whole-array semantics; nothing broke by removing it
(also independently confirmed by the full-suite run below being green).

## I-2 — CLOSED

**Claim:** the three new vectors are retagged `head:hashed` (were
`head:single`), and `head:hashed` now has four vectors and left
`SINGULAR_TAGS`.

**Verified in the diff and by execution.**
`crates/md-codec/tests/compose_support.rs:386,389,392` now read `head:hashed`;
`SINGULAR_TAGS` at `:423-430` no longer lists `"head:hashed"` (a comment there
now explains why it left).

- **Classification re-derived independently, not taken from the report.**
  `is_bare_single` at `compose/mod.rs:292-294` is
  `n==1 && hash.is_none() && lock.is_none()`. All four `hashlock_gated`
  vectors carry a hash, so `hash.is_none()` is false and `head:single` was
  provably wrong for all of them — `head:hashed` is the only shape that fits.
- **Neighbouring tags on the three retagged lines checked against every other
  `wsh`, 2-path, `older`-locked, hashed vector in `family()`**: `w:wsh`,
  `paths:2`, `lock:blocks`, `hash`, `ik:none`, `fp:one-seed-two-paths`,
  `origins:default-wsh` all match the pre-existing (untouched) sha256
  `head:hashed` vector at line 383 and the pattern used throughout the file
  for other 2-path, one-seed, `wsh`, `older`-locked entries (e.g. lines
  273, 279, 369). No mislabelled neighbour found.
- **Mutation-reproved the gate**, independent of the round-1 report: put
  `"head:hashed"` back into `SINGULAR_TAGS` and ran
  `every_tag_appears_in_at_least_two_vectors` — reds with
  `"a singular tag has exactly one vector: head:hashed" left: Some(4) right: Some(1)`,
  matching the fold commit message. Restored from a `cp` backup, confirmed
  clean.

---

## New-defect check

- **Full re-measurement**, all from a clean tree (`git status --porcelain`
  empty, `git ls-files -s | sha256sum` = `786b9e04…` both before and after):
  - `cargo nextest run --locked --all-targets` → **1314 passed, 3 skipped**
    (matches the fold's claim exactly; +1 over round 1's 1313, the one new
    test).
  - `cargo clippy --locked --all-targets --workspace -- -D warnings` → **exit
    0**, 0 lines matching `warning`/`error`.
  - `cargo +1.95.0 fmt --all -- --check` → **exit 0**, 0 bytes of output.
  - `cargo metadata --locked` at the fold tip → **exit 0**, resolves.
- **Doc-comment attachment** around the new `impl` block checked by hand: the
  explanatory note above the five impls uses `//`, not `///`, so it is not a
  doc comment and attaches to nothing — no risk of severing the next item's
  doc comment (a class this constellation has hit before).
- **No clippy lint fired** on the manual/derived split (e.g.
  `derived_hash_with_manual_eq`) — expected, since none of the five are
  derived any more; all five are hand-written together.
- Diff scope is exactly the three files the fold commit message claims:
  `compose/mod.rs` (+44/-1), `compose_hashkinds.rs` (+47), `compose_support.rs`
  (+30/-12 net, retag + `SINGULAR_TAGS` comment). Nothing off-topic.

## Spec-correction commit `507b0f46` (mnemonic-engrave)

- **§10 `sh(wsh)` → `tr` correction**: consistent with round 1's independent
  three-way confirmation (structural, executed, `git blame`); this commit only
  edits prose and cites the error variant by name, not by line — reasonable
  given round 1 flagged line-citation drift risk. Not re-derived (round 1
  already settled it and the brief lists it as already-settled framing).
- **F-537** (16 dangling spec citations) — re-ran
  `./scripts/plan-cite-check.sh design/SPEC_hashlock_kinds.md` from
  `mnemonic-engrave` independently: **28/44 resolved, 16 dangling, 0
  ambiguous** — matches the follow-up's cited counts exactly, including the
  specific dangling lines (`main.rs:2275,2685,3196`, `me-cli/Cargo.toml:53`,
  `ms-codec/hashlock.rs:59`, etc.).
- **F-538** (no in-tree Core-measured address gate) — follow-up text is
  consistent with round 1's own M-4 finding (self-pin via rust-miniscript, not
  an independent verdict; `bitcoind` unreachable during round 1). Correctly
  filed as open rather than fixed; nothing to re-derive beyond what round 1
  already measured.

Neither new follow-up is fixed in code, and neither needed to be — both are
Minor-owning-phase items filed per the round-1 report's own M-3/M-4, not
blocking findings.

---

## Counts

| severity | count |
| --- | --- |
| Critical | **0** |
| Important | **0** (I-1, I-2 both closed) |
| Minor (new, from this fold) | **0** |
| Nit (new, from this fold) | **0** |

**GREEN.** Both round-1 Importants are closed by construction and reproved by
independent mutation, not merely re-read. Full gate re-measured clean at the
fold tip: 1314 passed / 3 skipped, clippy 0, fmt clean, `cargo metadata
--locked` resolves. No new defect found in the fold's diff or in the spec
correction commit.

## Worktree state

Left **clean and byte-identical**. `git ls-files -s | sha256sum` =
`786b9e0451e01c232519b34ab6ceaa5609062685b05d585a06596c6ef35003d4` before and
after every mutation (three total: the negative-control derive restore for my
own probe, the derive restore to reprove the fold's own test, and the
`SINGULAR_TAGS` restore to reprove the tag gate) — each restored from a `cp`
backup immediately after its assertion, and the scratch probe file
`crates/md-codec/tests/zzprobe_r2.rs` deleted. Final `git status --porcelain`:
empty.
