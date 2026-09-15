# R0 round 1 — SPEC_hashlock_kinds phase 1, `descriptor-mnemonic` branch `hashkinds-p1`

**Scope:** the branch only. `064e107f`, `af3ba0b2`, `83cd53e4` over base `40c400de`,
worktree `/scratch/code/shibboleth/dm-worktrees/hashkinds-p1`.
**Type:** first adversarial review of this branch, not a fold verification.
**Toolchain:** `cargo 1.85.0 (d73d2caf9 2024-12-31)` via `PATH=/home/bcg/.cargo/bin:$PATH`.

**Verdict: NOT GREEN — 0 Critical, 2 Important, 4 Minor, 2 Nit.**

Neither Important is a wrong result that ships today. I-1 is a documented
guarantee that the type does not keep, and the trap it sets is inherited
structurally by phase 4's Go port. I-2 is a coverage gate that was bypassed by a
mislabel rather than by a decision — and it is the *same* gate the author
correctly reasoned about, in the same commit, for a neighbouring tag.

---

## Re-measurement of the author's claims

Every one holds.

| claim | measured | result |
| --- | --- | --- |
| 1313 tests pass / 3 skipped | `cargo nextest run --locked --all-targets` | **1313 passed, 3 skipped**, exit 0 |
| clippy 0 under `-D warnings` | `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit **0**, zero `warning`/`error` lines |
| fmt clean | `cargo +1.95.0 fmt --all -- --check` | exit **0**, 0 bytes of output |
| +9 tests (1304 → 1313) | 5 new in `cli_compose_hashkinds.rs`, 4 in `compose_hashkinds.rs` | **9** |

Note on method: my first clippy/fmt run piped to `tail`, so `$?` was `tail`'s.
Re-run without the pipe; the exit codes above are the real ones.

### The digests are independently correct — the F4 funds-loss class is clean

§3 F4 names the sharpest failure in this cycle: `hash256` is `sha256d`, and an
implementer one word short produces a 32-byte value with no structural signal.
I recomputed all three new digests from scratch, outside both repos:

```
python3 -c "import hashlib
p=b'correct horse battery staple'
X=hashlib.pbkdf2_hmac('sha256',p,b'ms-hashlock-v1',100000,32)
print('hardened_hash256',hashlib.sha256(hashlib.sha256(X).digest()).hexdigest())
r=hashlib.new('ripemd160'); r.update(X); print('hardened_rmd160 ',r.hexdigest())
r=hashlib.new('ripemd160'); r.update(hashlib.sha256(X).digest()); print('hardened_hash160',r.hexdigest())"

hardened_hash256 98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
hardened_rmd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
hardened_hash160 b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

All three match `test_vectors.rs:489-503`, `compose_support.rs:HK256/HRIPE/H160K`,
and the `hardened_h_*` columns of `ms-codec/tests/vectors/hashlock-v0.8.json`
`derivation[0]`. The right digest function is wired to the right kind. No finding.

### Other claims checked and confirmed

- **No existing vector changed.** `git diff 40c400de..83cd53e4 --diff-filter=MDR
  --name-status` lists no file under `crates/md-codec/tests/vectors/`; all 15
  vector files are `--diff-filter=A`.
- **The wire carries 20 bytes, not 32, for the 20-byte kinds.**
  `keyed_compose_preset_hashlock_gated{,_hash256}.bytes.hex` = 199 bytes;
  `..._ripemd160`/`..._hash160` = **187 bytes**, exactly 12 fewer. The ripemd160
  digest appears byte-aligned and verbatim in the wire; no padding.
- **"No wire-format change"** (CHANGELOG + MIGRATION). `git diff` over
  `encode.rs tag.rs chunk.rs bitstream.rs validate.rs tree.rs to_miniscript.rs
  render.rs` is **empty**. Only `compose/{mod,lowering,presets}.rs` and
  `test_vectors.rs` changed in `md-codec/src/`.
- **`cargo metadata --locked` resolves at each of the three commits** (exit 0 at
  `064e107f`, `af3ba0b2`, `83cd53e4`, each in its own throwaway worktree). The
  version bump and the exact pin move together in `83cd53e4`:
  `md-codec 0.42.0→0.43.0`, `md-cli 0.14.0→0.15.0`, and
  `md-codec = { path = "../md-codec", version = "=0.43.0" }` in the same commit.
  SemVer is right on both (breaking API, breaking `--json` key, pre-1.0 `0.X` axis).
- **Nothing off-topic in the diff.** 31 files, all phase-1 surface.
- **The 15 new corpus files are really consumed, not decoration.** Flipping one
  nibble of the ripemd160 `.bytes.hex` reds
  `md-cli::vector_corpus::vectors_output_matches_committed_corpus`; so does
  changing one character of an address in its `.conformance.json`.

### CLI grammar — enumerated and run

All four kinds on `--path` and on `--preset`, under `wsh` and `tr`; wrong widths
both directions; uppercase option and uppercase hex; two kinds on one path; the
same kind twice; empty value; missing `=`; odd length; non-hex; keyless paths;
`--json` and not. **Every refusal is correct, names the kind's own width, and
exits 1.** `--json` stdout parses (the "keyless template" note goes to stderr).
`--preset` duplicate of one option refuses (`` `ripemd160=` given twice ``);
uppercase on `--preset` refuses through `named_only`
(`preset hashlock-gated admits no RIPEMD160= parameter`). No finding here beyond
N-1/N-2 below.

### Mutation proofs — every new test earns its keep

Each mutation asserted unique-site application **before** running (`assert
s.count(old)==1`), then grepped back out of the file.

| mutation | site | reds |
| --- | --- | --- |
| `digest()` → `&self.digest[..]` | `mod.rs:226` | 3 of 4 (`digest_len_is_the_only_length…` on the accessor; the other two panic in `lowering.rs:95`) |
| `tag: h.kind().tag()` → `Tag::Sha256` | `lowering.rs:100` | `lowering_emits…`, `every_kind_round_trips…` |
| `HashKind::Ripemd160 => Tag::Hash160` | `mod.rs:182` | `every_kind_maps_to_its_own_wire_tag` + 2 |
| `let want = kind.digest_len()*2` → `64` | `compose.rs:159` | `each_kind_refuses_the_other_width`, `every_kind_is_a_path_option…`, `the_preset_takes_every_kind…`, **and** `preset_refuses_a_missing_or_malformed_hash` |
| JSON `kind`+`digest` → `sha256` | `compose.rs:497` | `the_preset_takes_every_kind_and_the_json_names_it` |
| case-fold the option name | `compose.rs:117,140` | `an_unknown_or_uppercase_kind_is_refused` |
| drop the one-hash guard | `compose.rs:118-122` | `two_hashes_on_one_path_are_refused_across_kinds` |

**No new test passes vacuously.** All nine are killed by at least one mutation.

---

## I-1 (Important) — `HashLock`'s derived `Eq`/`Hash`/`Ord` read the alloc-gate padding, contradicting the guarantee stated in three places

**`crates/md-codec/src/compose/mod.rs:206-227`** — `#[derive(… PartialEq, Eq,
Hash, PartialOrd, Ord)]` at `:206` over `{ kind: HashKind, digest: [u8; 32] }`
at `:208-209`, with the public constructor `pub const fn new(kind, [u8; 32])` at
`:215` and `digest()` at `:225-226`.

The branch states the guarantee three times:

- `mod.rs:213-214` — "`digest`'s tail beyond `kind.digest_len()` is padding and
  is **never read back**."
- `MIGRATION.md` — "**`digest()` is the only correct way to read it** and
  returns exactly `kind.digest_len()` bytes."
- `SPEC_hashlock_kinds` §5 — the padding "is **unobservable** except through
  `ComposerPathHashes` (§7.3)".

All three are false. `PartialEq`, `Eq`, `Hash`, `PartialOrd` and `Ord` read the
whole array.

### Constructed failure

Temporary probe in `crates/md-codec/tests/` (since deleted), two `HashLock`s of
kind `Ripemd160` with **identical 20-byte digests**, differing only at byte 31:

```
digest(a) == digest(b): true
la == lb              : false
hash(la) == hash(lb)  : false
la < lb               : true
set.contains(lb)      : false
set len after both    : 2
```

Two values that lower to byte-identical script, byte-identical md1 and
byte-identical JSON compare unequal, hash differently, and both fit in a
`HashSet`.

### Why it matters, concretely

§5 mandates that "**eleven** map/set/equality sites currently keyed on a bare
`[32]byte` — including `composerState.hashlockHeld`, on which §8's warning
depends — re-key on the whole `HashLock`." This type is what they re-key on. A
`HashLock` reaching such a map by any route that does not zero the tail misses
its own lookup, and §8's 20-byte warning — the one that tells an operator the
device did not derive the preimage — silently does not fire.

Phase 4 inherits this structurally, not by choice: Go's `==` on a struct
containing a `[32]byte` compares the whole array and offers no way to hide the
tail, so the Go port cannot be correct here unless the tail is normalised.

### Why Important and not Critical

No wrong result ships in phase 1. Every in-tree construction zero-fills —
`parse_hash_hex` at `compose.rs:173` allocates `[0u8; 32]` and writes exactly
`digest_len()` bytes, and the test fixtures do the same. I checked all seven
readers of `SpendPath.hash` across both crates' `src/`: only `lowering.rs:78`
reads the value, and it reads it through `digest()`. No `PathList`-equality test
in the tree involves a hashlock. So the padding reaches no script, no wire, no
template, no JSON, no vector file and no CLI string — I looked for an escape and
could not construct one. What is defective is the public contract: `new()`
accepts arbitrary padding and the type's own equality then disagrees with the
wire it lowers to. A reviewer could argue Critical on "unmet guarantee"; I rate
Important because the guarantee's *purpose* — never committing padding into a
script — is met.

**Fix:** zero the tail in `new()` (a `const fn` may use a `while` loop), or
hand-write the five impls over `(kind, digest())`. The former is one loop and
makes the doc, MIGRATION and §5 true by construction rather than by convention.

---

## I-2 (Important) — the three new vectors carry the wrong `head:` tag, and that mislabel is exactly what let `head:hashed` stay in `SINGULAR_TAGS`

**`crates/md-codec/tests/compose_support.rs:386, 389, 392`** (the three new tag
vectors), against the shape definition at `:378-383` and `SINGULAR_TAGS` at
`:417-421`.

The pre-existing sha256 vector defines the shape in a comment at `:379-381`:

> the head path is one key PLUS a hash, unlocked -- neither `head:bare-multi`
> (n = 1), `head:single` (`is_bare_single` needs no hash), nor `head:locked` (no
> lock). `head:hashed` names this fourth shape

All four vectors are built by the **same call**,
`presets::hashlock_gated(Wrapper::Wsh, <hash>, 26280)`, so all four have the
identical head path: one key, a hash, no lock. `is_bare_single`
(`compose/mod.rs:250-252`) requires `hash.is_none()`, so `head:single` is
provably wrong for the three new ones. They should be `head:hashed`.

### Constructed failure

Retagging the three to their true shape reds the gate:

```
$ cargo test --locked -p md-codec --test compose_vectors every_tag_appears_in_at_least_two_vectors
assertion `left == right` failed: a singular tag has exactly one vector: head:hashed
  left: Some(4)
 right: Some(1)
test result: FAILED. 0 passed; 1 failed
```

### Why it matters

`SINGULAR_TAGS`'s own doc at `:404-412` states its purpose: "The test pins them
at exactly one so that a second vector forces an explicit decision here instead
of silently widening the exemption." Three vectors of that shape were added and
**the decision was not forced** — the gate can fail, but it was handed a wrong
input label, so it did not.

Three consequences, all live in the tree right now:

1. `head:hashed` stays exempt with a now-false justification at `:419-420` —
   "the **ONLY** family vector whose head path is a single key plus a hash,
   unlocked". There are four.
2. `head:single`'s coverage count is inflated by three vectors that are not
   single-headed, so the family's coverage accounting is wrong in both
   directions.
3. The author reasoned about this exact gate correctly in the same commit —
   `af3ba0b2`'s message says "`preset:hashlock-gated` LEAVES SINGULAR_TAGS. It
   had an exemption … it now has four, so the ordinary rule applies and is a
   stronger check than the exemption was." That reasoning applies verbatim to
   `head:hashed` and was not applied, because the tag said `head:single`.

**Fix:** tag the three `head:hashed`, remove `head:hashed` from `SINGULAR_TAGS`
(it now has four vectors, so the ordinary two-vector rule is the stronger
check), and update the `:419-420` comment — the same three edits the author
already made for `preset:hashlock-gated`.

---

## M-1 (Minor) — the width assertion cannot distinguish 40 hex from 64

**`crates/md-codec/tests/compose_hashkinds.rs:118-125`.** The comment at
`:114-115` claims the check proves "a 20-byte kind must show 40 hex characters,
not 64", but the assertion is `rendered.contains(&hex)` where `hex` is the
40-character digest. A 64-character padded render **contains** that 40-character
string as a prefix, so the assertion would pass on exactly the render it claims
to exclude.

I checked reachability by constructing the padded lowering — `Body::Hash256Body`
with the 20 bytes left-aligned, under the correct `Tag::Ripemd160`. The test
**did** red, but at `:109`, not at `:124`:

```
render: MalformedTree("ripemd160 body must be Hash160Body")
```

So the codec's body-type invariant catches it and the claimed failure mode is
currently unreachable. Minor, not a defect. Tightening to
`contains(&format!("({hex})"))` is one line and makes the test self-sufficient
instead of leaning on an invariant three modules away.

## M-2 (Minor) — uppercase rejection is pinned on `--path` only, not on `--preset`

`cli_compose_hashkinds.rs:94-102` covers `--path 1of1,RIPEMD160=…`. Spec §6
makes "case is rejected, never folded" a grammar-wide rule. The preset side
behaves correctly — I ran
`--preset hashlock-gated,RIPEMD160=<40hex>,older=26280` → exit 1,
`preset hashlock-gated admits no RIPEMD160= parameter` — but nothing pins it,
because the refusal comes from `named_only`'s allow-list rather than from a test.
A future widening of `named_only` would go unnoticed. One extra case in the
existing test closes it.

## M-3 (Minor) — the `sh(wsh)` finding is correct, but §10 still asks for it

**The author's claim is true**, verified two independent ways rather than taken
on the commit message:

- Structurally: `is_bare_multi` (`compose/mod.rs:244-248`) requires
  `hash.is_none()`; `Wrapper::ShWsh` is legacy (`:94-96`); the guard at `:480-488`
  refuses anything else.
- By execution: `--wrapper sh-wsh --preset hashlock-gated,sha256=<64hex>,older=26280`
  → exit 1, `legacy wrappers hold one plain sorted multisig only (n >= 2, no
  lock, no hash); use wsh or tr`. Same for `--path 2of3,ripemd160=…` and for
  bare `sh`. It refuses **for sha256 too**, so it is not a hash-kinds
  regression.
- `git blame` puts the rule at `ebdd898e`, 2026-09-02 ("composer S0 task 1") —
  it predates this cycle, as claimed.

Documenting it at the test is the right call for a `descriptor-mnemonic`-only
branch; the author could not edit the spec from here. But `SPEC_hashlock_kinds`
§10 (in `mnemonic-engrave`) still states the requirement, so the next reader of
§10 sees a gate that no phase can close. **Amend §10 before phase 4** — either
drop `sh(wsh)` citing this refusal, or relax the composer's legacy narrowness
deliberately, since §3 F2 measured Core as accepting `sh(wsh(<hashlock>))`. This
is a spec edit in the other repo, not a change to this branch.

## M-4 (Minor) — §12 acceptance item 1 is not closed, and nothing in-tree closes it

The three new address sets in `*.conformance.json` are produced by `md vectors`
via `crates/md-cli/src/cmd/vectors.rs:199`, `d.derive_address(chain, index,
Network::Bitcoin)` — md-codec's rust-miniscript route. That is genuinely
independent of the composer's own lowering (rust-miniscript compiles
`ripemd160(...)` to `OP_RIPEMD160` itself), and the files are a real pin, not
decoration — I proved both by mutation. But it is a **self-pin, not Core's
measured value**, and §12 item 1 asks for "its address matches Core's measured
value", while §10 says the keyless shape for the three new kinds "is unmeasured
and must be measured before this closes".

I could not measure it: this box's `bitcoin-cli` reaches a running node that
returned `Work queue depth exceeded` on every call across four attempts, and I
did not hammer the operator's node or stand up a competing instance. Flagging it
as an open cycle-level gate rather than a defect in this branch — but it is open,
and phase 1 is the phase that produced the addresses.

## N-1 (Nit) — the preset's two-kind refusal says "per path"

`compose.rs:427` emits `preset hashlock-gated: at most one hash per path, got
sha256 and ripemd160`. A `--preset` invocation has no `--path`; the phrasing is
borrowed from the `--path` arm. "at most one hash" would read correctly in both.

## N-2 (Nit) — the four widths are written as literals in the preset's help refusal

`compose.rs:420-422` hardcodes `sha256=<64 hex>, hash256=<64 hex>,
ripemd160=<40 hex> or hash160=<40 hex>`. §5 makes `digest_len()` "the only place
a digest length is written" a named property, and this is a second place. It is
operator text rather than a rule, and `parse_hash_hex` (the actual rule) derives
correctly from `digest_len()*2` at `:159` — so nothing can drift silently. Worth
building from `KINDS.iter()` for the property's sake.

---

## Counts

| severity | count | items |
| --- | --- | --- |
| Critical | **0** | — |
| Important | **2** | I-1 padding observable through `Eq`/`Hash`/`Ord`; I-2 wrong `head:` tag bypassed the `SINGULAR_TAGS` gate |
| Minor | **4** | M-1 width assertion; M-2 uppercase unpinned on `--preset`; M-3 §10 `sh(wsh)` unsatisfiable; M-4 §12 item 1 not Core-measured |
| Nit | **2** | N-1 "per path" in a preset message; N-2 literal widths in help text |

**NOT GREEN.** I-1 and I-2 block. Both fixes are small and local: normalise the
padding (or hand-write the impls) in `compose/mod.rs`, and correct three tag
strings plus one `SINGULAR_TAGS` entry in `compose_support.rs`. I-2's fix will
red `every_tag_appears_in_at_least_two_vectors` until `head:hashed` is removed
from the exemption list — that red is the gate working, not a regression.

Secret-handling: none found; none would have gated regardless, per the standing
severity rule.

## Worktree state

Left **clean and byte-identical**. Every mutation was reverted from a `cp`
backup and the probe file deleted; verified by
`git status --porcelain` (empty) and `git ls-files -s | sha256sum` =
`aaed1165199b98a33f44ff684997b17f61b05ed1d06060bd2479bf5781a9bcb9`, the value
recorded before the first mutation. Final
`cargo nextest run --locked --all-targets`: **1313 passed, 3 skipped**.
