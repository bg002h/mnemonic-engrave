# FIX-F411 — the keyed origin-depth clause, and one CHANGELOG line

**Agent report, written by the implementing agent as its final action.**
Work landed as `b6d8b515` on `fix/f411-keyed-note` in the worktree
`/scratch/code/shibboleth/_work/f411/descriptor-mnemonic` (base `6f75ecfb`).
Not pushed. `md-codec` untouched. `--path`, `--help` and the README untouched.

---

## 1. The keyed clause, as implemented

`emit_unhardened_origin_note` (`crates/md-cli/src/cmd/encode.rs`) now takes
`keys: &[ParsedKey]` and runs **two tiers**. Both call sites in `run` pass
`&parsed_keys` (text branch and `--json` branch).

Per placeholder **declaration** (not per occurrence), with a non-empty origin:

```rust
// TIER 2 (F-411) — evaluated first; needs the key seated in THIS slot.
if let Some(key) = keys.iter().find(|k| k.i == occ.i) {
    let d = usize::from(key.depth);
    if key.depth >= 1                                   // (1) master excluded
        && components.len() > d                         // (2) there is an excess
        && !components[d..]                             // (3) the excess is unhardened
            .iter()
            .any(|c| matches!(c, ChildNumber::Hardened { .. }))
    { /* note, then `continue` */ }
}
// TIER 1 (F-410) — the narrow, key-blind predicate. UNCHANGED.
if components.iter().any(|c| matches!(c, ChildNumber::Hardened { .. })) { continue; }
affected.entry(occ.i).or_insert_with(|| format!("/{path}"));
```

**Two decisions the ruling left to the implementation, stated plainly:**

- **A slot lands in at most one tier and is said once.** Tier 2 is tested first,
  so where both match (an all-unhardened origin *longer* than the seated key,
  e.g. `@0/0/1/2/3/*` with a depth-3 key) tier 2's wording wins. It is the
  better-informed one: it knows the key is not master, so it can say the two
  readings **do** diverge rather than that they might. No firing was removed —
  every input that noted before still notes.
- **One line per firing slot in tier 2**, versus tier 1's joined list. Each
  tier-2 line carries its own depth, level count and excess, which do not
  collapse into a shared sentence.

**`ParsedKey` gained `depth: u8`** — byte 4 of the 78-byte serialization, the
field `bitcoin::bip32::Xpub::depth` exposes, read from the buffer `parse_key`
had already decoded rather than parsing the key twice. It is **not** in
`payload` (bytes `13..78`), so it can move no wire byte, no address and no
wallet id.

**The ruling's premise about scope held.** `parsed_keys` is in scope at both
call sites, `ParsedKey.i` associates a key with its placeholder index, and
`lex_placeholders` yields `occ.i` — no STOP condition was hit. The one thing
the ruling assumed that was *not* already true: `ParsedKey` did not carry
depth. Adding the field was the minimal way to get it, and `Xpub::depth` is the
same byte.

### The emitted line

```
note: @0's declared origin runs DEEPER than the xpub seated there — `/84'/0'/0'/0`
is 4 levels, but the key at @0 is depth 3, so the trailing `/0` hangs BELOW it. In
an md template the WHOLE path after `@0` is that key's origin declaration and md
derives nothing through it: this card backs the seated key's own `/i`, NOT `/0/i`
as a descriptor-style reading expects. Every step past depth 3 is unhardened, which
is exactly the shape that xpub COULD have derived, so nothing on the card tells the
two readings apart. Confirm the xpub seated at @0 is the key `/84'/0'/0'/0` names; a
step meant as DERIVATION belongs in the use-site tail (`/<0;1>/*`), not in the origin.
```

Every factual claim in it was measured on the binary, not reasoned:

```
wpkh(@0/84'/0'/0'/0/*)      -> bc1qr932kkqd95r3chv9sh36wkjez4jvsmlf46xuc9
wpkh(@0/84'/0'/0'/*)        -> bc1qr932kkqd95r3chv9sh36wkjez4jvsmlf46xuc9
wpkh(@0/84'/0'/0'/<0;1>/*)  -> bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m
```

with the depth-3 xpub seated at `@0`. The first two agree, so the excess `/0` is
inert — "md derives nothing through it" is measurement, not assertion. The third
is what the descriptor-style reading meant. **That gap is the whole finding.**

---

## 2. Every test case, and why it fires or does not

`crates/md-cli/tests/cli_keyed_excess_origin_note.rs` (10 tests) plus three unit
tests in `cmd::encode::tests`. Fixtures: `K3` = depth-3 account xpub, `K4` =
depth-4 (`m/48'/0'/0'/2'`), `MASTER` = depth 0.

| # | case | seated | result | why |
| --- | --- | --- | --- | --- |
| 1 | `wpkh(@0/84'/0'/0'/0/*)` | K3 (d=3) | **FIRES** | 4 levels > 3; excess `/0` unhardened |
| 2 | `wsh(multi(1,@0/48'/0'/0'/2'/0/<0;1>/*))` | K4 (d=4) | **FIRES** | 5 > 4; names "depth 4" — proves no hardcoded 3 |
| 3 | `wsh(multi(2,@0/84'/0'/0'/0/…,@1/48'/0'/0'/2'/…))` | K3, K4 | **FIRES @0 only** | @1's 4 levels == its depth 4, no excess |
| 4 | same declaration twice in one template | K3 | **FIRES once** | keyed per declaration, not per occurrence |
| 5 | `--json --force-chunked`, case-1 template | K3 | **FIRES** | advisory parity with the text branch |
| 6 | `wpkh(@0/84'/0'/0'/<0;1>/*)` | K3 | silent | 3 levels == depth 3 — **the ordinary workflow** |
| 7 | `wpkh(@0/84'/0'/0'/0'/*)` | K3 | silent | 4 > 3 but the excess `0'` is HARDENED |
| 8 | `wpkh(@0/84'/0'/0'/0/*)` | *none* | silent | tier 1: undecidable without a key. **Same template as #1** |
| 9 | `wpkh(@0/0/*)` | K3 | tier 1 fires, tier 2 silent | 1 level is not deeper than depth 3 — F-410 regression guard |
| 10 | `wpkh(@0/0/*)` | MASTER | **exit 1, refused** | see below |
| U1 | `wpkh(@0/0/1/*)`, synthetic depth-0 key | d=0 | tier 1 only | condition 1 in isolation |
| U2 | `wpkh(@0/0/1/2/3/*)`, synthetic d=3 | d=3 | tier 2 only, one line | precedence where both tiers match |
| U3 | `@0` unkeyed, `@1` keyed | d=4 on @1 | tier 1 on @0 | a key elsewhere must not lend its depth |

**On the master case (#10), which the brief asked me to state rather than
assume.** A master xpub cannot be seated at all: `parse_key` admits depth 3 or
4 only, so `md encode … --key @0=<master>` exits **1** with
`expected an account-level xpub at depth 3 or 4 …, got 0` before any advisory
runs. So condition 1 (`depth >= 1`) is **unreachable through the CLI**. It is
still implemented, because the guard has to be right — the two spellings
provably agree on a master key, so a note there would claim a divergence that
cannot happen — and it is exercised by unit test U1, which is the only way to
reach depth 0. Test #10 pins *why* the CLI cannot reach it, so a later widening
of `parse_key`'s depth admission does not silently orphan the guard.

**Mutation-tested — a green test that survives its own mutation proves
nothing.** Each condition was removed in turn, the suite re-run, and the
mutation reverted:

| mutation | tests that went RED |
| --- | --- |
| drop `key.depth >= 1` | `a_depth_zero_key_takes_the_keyless_tier_not_the_keyed_one` |
| `components.len() > d` → `>=` | `matching_depth_origin_is_silent`, `a_key_seated_elsewhere_does_not_reach_this_slot` |
| drop the hardened-excess check | `excess_hardened_suffix_is_silent` |

All three conditions are load-bearing and individually caught.

---

## 3. Proof that stdout and the exit code are unchanged

Two independent proofs.

**(a) Pre/post binary matrix.** 18 invocations — `encode` text and `--json`,
`--path`, `address`, `decode`; firing and silent; keyed and keyless — run
against the binary built at `6f75ecfb`, then against the binary built after
this change. `diff -rq` over the whole capture (stdout, stderr and exit code
per invocation) reports **only six `.stderr` files differing**, and those six
are exactly the invocations where the keyed clause fires. The delta in each is
one added `note:` line and nothing else.

```
concatenated *.stdout + *.exit, pre   md5 31e77e63c4d38ef186708149b8216fa1  (2130 bytes stdout)
concatenated *.stdout + *.exit, post  md5 31e77e63c4d38ef186708149b8216fa1  (2130 bytes stdout)
```

Re-run after the clippy and `cargo fmt` fixes: the post capture is byte-identical
to itself, so those edits changed nothing observable either.

**(b) Goldens in the tests.** Every stdout assertion in the new test file was
captured from the **pre-change** binary before a line of implementation was
written, so a note leaking onto stdout fails there. The `--json` test does not
restate its own golden: it asserts the two chunk strings the *text* branch
printed pre-change, making it a cross-branch check.

Exit codes: `assert_eq!(code, 0, …)` on all nine exit-0 cases, and
`assert_eq!(code, 1, …)` on the refused master.

---

## 4. The CHANGELOG line, verbatim

Added as a new `[Unreleased]` section at the top of `CHANGELOG.md`, titled
"BIP-388 `/**` accepted on encode", under `### Added`:

```
- **`md encode` accepts `@i/**` as BIP-388 sugar for `@i/<0;1>/*`** — byte-identical to the desugared spelling, where it previously exited non-zero — while `md decode` renders the canonical `<0;1>/*`, so the round trip is not spelling-preserving.
```

Machine-checked before writing it: `md encode "wpkh(@0/84'/0'/0'/**)"` emits
`md1yq802gggqpsqwgtua24e7ssf3`, byte-identical to the pre-existing golden for
the desugared `wpkh(@0/84'/0'/0'/<0;1>/*)`; decoding that card prints
`wpkh(@0/<0;1>/*)` on stdout. The prior refusal is quoted verbatim in the module
docs of `cli_bip388_double_wildcard.rs`. Commit `5465253b` touched no CHANGELOG,
so this was genuinely undocumented.

`--help` and the README were **not** touched, per the ruling.

---

## 5. Gates

| gate | result |
| --- | --- |
| `cargo nextest run --locked` | **863 tests run: 863 passed, 2 skipped** (baseline 850/2, +13) |
| `cargo clippy --all-targets --locked -- -D warnings` | **exit 0** |
| `cargo fmt --check` | **exit 0** |

Clippy caught one thing on the first run — `format_collect` on the excess-suffix
builder — fixed by appending to a `String` directly; the matrix was then re-run
and produced byte-identical output.

---

## 6. Filed, not fixed

**F-412** (`design/FOLLOWUPS.md`, commit `5d9caeb`) — after this change,
`--path` can produce a note about a declaration the card no longer carries:

```
md encode "wpkh(@0/84'/0'/0'/0/*)" --key @0=<depth-3> --path bip84
  -> note: @0's declared origin runs DEEPER than the xpub seated there …
```

while the card carries `m/84'/0'/0'` against a depth-3 key — consistent, no
misreading available. The ruling put `--path` out of scope in the *widening*
direction; this is the narrowing direction, which it also did not authorise, and
tier 1 has always behaved this way. A tier-2-only patch would split the tiers on
a rule nobody wrote, so what is owed is one decision covering both.

**Left for the controller:** `F-411`'s own entry in `design/FOLLOWUPS.md` is
still open and still says *"What is owed: a decision, not code."* The decision
has now been made and half of it implemented (keyed tier shipped; keyless tier
ruled FINAL; `--path` deferred to F-412). Editing that entry's ruling text is
the controller's call, not the implementer's, so it was left untouched.
