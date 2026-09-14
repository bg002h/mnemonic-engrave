# RECON: blast radius of `SpendPath.hash: [u8;32]` -> `HashLock { kind, digest }`

Recon only. No design, no refactor, no code-quality opinions. Every count below
states the exact command that produced it; commands were re-run against the
working trees at:

- descriptor-mnemonic `40c400de` (2026-09-13 11:49:24 -0700)
- mnemonic-engrave `40c400de` (2026-09-13 11:49:24 -0700, same commit — shared checkout state at recon time)
- seedhammer `0562e811` (2026-09-13 15:52:21 -0700)

All greps exclude `/target/` and `/vendor/` unless noted. `crates/` scoping was
used instead of repo-root scoping in descriptor-mnemonic and mnemonic-engrave
specifically to keep vendored dependency source out of the count without a
blanket exclude flag (verified this drops zero real hits: the repo-root run
was inspected first and every excluded hit was `vendor/*`).

## 0. The one architectural fact that changes the shape of this estimate

**The wire-format machinery for all four hash kinds already exists and is
already generic, in both languages.** This is not new plumbing:

- Rust: `Tag::Sha256` (0x1D), `Tag::Hash256` (0x1F, inferred from context),
  `Tag::Hash160` (0x1E), `Tag::Ripemd160` (0x20) all exist in
  `crates/md-codec/src/tag.rs:75-84`. `Body::Hash256Body([u8;32])` and
  `Body::Hash160Body([u8;20])` (`crates/md-codec/src/tree.rs:65-68`) already
  carry both widths. `to_miniscript.rs:664-682` already lowers all four tags to
  their `rust-miniscript` `Terminal` variants (`Sha256`, `Hash256`,
  `Ripemd160`, `Hash160`), each via its own `_from_bytes` helper
  (`to_miniscript.rs:748-761`).
- Go: `tagSha256`/`tagHash160`/`tagHash256`/`tagRipemd160`
  (`md/md.go:69-72`), `hash256Body [32]byte` / `hash160Body [20]byte`
  (`md/md.go:120-121`), and `script_emit.go:458-482` already emits script for
  all four.
- Both languages already have property-test / fuzz coverage of all four kinds
  **at the `Node`/`Tag`/`Body` layer**: `crates/md-codec/tests/common/mod.rs:490-493,798-801`
  generates arbitrary `Sha256`/`Hash256`/`Ripemd160`/`Hash160` nodes, and
  `crates/md-codec/tests/proptest_to_miniscript.rs:189,286,306,1068-1107`
  exercises all four through `to_miniscript`.

**The gap is entirely one layer up: the composer (`SpendPath`), everything
that authors or renders a `SpendPath`, and one independent third
implementation of a `hash:` wire record in mnemonic-engrave that has nothing
to do with `md_codec::compose` at all (section 1c).** Nothing found below
requires new codec-level machinery. Everything below is "teach the composer,
the CLI, the sysw record, and six GUI screens about a `kind` field that the
tree layer underneath them already understands."

---

## 1. Every construction and read of the hash field

### 1a. `descriptor-mnemonic` — `SpendPath.hash: Option<[u8; 32]>`

Definition: `crates/md-codec/src/compose/mod.rs:151`.

Commands:
```
grep -rn '\.hash\b' --include=*.rs crates/
grep -rn 'hash:\s*[A-Za-z0-9\[]' --include=*.rs crates/
```

| File | Kind | Lines | Count |
|---|---|---|---|
| `crates/md-codec/src/compose/mod.rs` | production, read | 159, 164, 373 | 3 |
| `crates/md-codec/src/compose/presets.rs` | production, construction | 11, 94 | 2 |
| `crates/md-codec/src/compose/lowering.rs` | production, read (the ONE emission site: `Tag::Sha256` hardcoded at line 78) | 76 | 1 |
| `crates/md-cli/src/cmd/compose.rs` | production, construction (57) + read (114) + construction (120) | 57, 114, 120 | 3 |
| `crates/md-codec/tests/compose_support.rs` | test, construction (166,177,186,192) + read (135) | 135, 166, 177, 186, 192 | 5 |
| `crates/md-codec/tests/compose_lowering.rs` | test, construction (21,33,44,51,97) + read (722) | 21, 33, 44, 51, 97, 722 | 6 |
| `crates/md-codec/tests/compose_crosscheck.rs` | test, construction | 25, 33, 142 | 3 |

**Totals: 23 code-line references (9 production, 14 test), across 7 files.**
Production splits 4 constructions / 5 reads; test splits 12 constructions / 2
reads. (`lowering.rs:76-79` is the single site that turns the field into wire
`Tag::Sha256` — everything else is a predicate or a builder.)

One more production site is a fixed-32-byte **function signature**, not a
`SpendPath` read/write, and belongs in §4 too:
`crates/md-codec/src/compose/presets.rs:90` — `pub fn hashlock_gated(hash: [u8; 32], ...)`.

### 1b. `mnemonic-engrave` — does **not** touch `md_codec::compose::SpendPath` at all

Verified: `grep -rln "SpendPath\|md_codec::compose\|compose::" --include=*.rs crates/`
returns **zero** files. `me-cli` depends on `md-codec = "0.42"` (published
crate, not a path dep — `crates/me-cli/Cargo.toml`) only for
`Descriptor::derive_address` (address derivation), never the composer.

**This means item 2 of the brief ("mnemonic-engrave, which consumes the
composer") does not describe how the hashlock field actually reaches this
repo.** What mnemonic-engrave has instead is a **third, independent
implementation of a `hash:` wire record**, unrelated to `SpendPath` by any
shared type, living in `crates/me-cli/src/sysw/composer_records.rs`. This is
squarely in scope for the kind-tag change (design decision #2 names this exact
record), so it is inventoried in full:

- `HASH_PREFIX: &str = "hash:"` — `composer_records.rs:30`
- `ComposerRecord::Hash([u8; 32])` — `composer_records.rs:108`
- `hash_record(digest: &[u8; 32]) -> String` — `composer_records.rs:200-202`
- `parse_hash(body: &str) -> ...` — `composer_records.rs:302-309`, hardcodes
  `body.len() != 64` (line 303)
- `ComposerRecordError::Hash` — two format sites, `composer_records.rs:144,156`

Production consumers, by file:line (command: `grep -rn "ComposerRecord::Hash\|HASH_PREFIX\|hash_record\|parse_hash\b" --include=*.rs crates/`):

| Site | What it does |
|---|---|
| `sysw/mod.rs:341` | `Ok(ComposerRecord::Hash(_)) => Class::Hash` — classifies the record for the door/consent screens |
| `main.rs:1795` | `Class::Hash` matched in an admission-eligibility list |
| `main.rs:2272-2278` | `print_composer_confirmation`: renders `hash %d..%d` — **truncation site, see §4** |
| `main.rs:2569` | `preimage_digest_of`'s caller collects all `hash:` digests into `Vec<[u8;32]>` for §8.2.3's orphan check |
| `main.rs:2685` | `C::Hash => "sha256 hashlock (hash:)"` — the class's human label, hardcodes "sha256" |

Test consumers: `crates/me-cli/tests/sysw_composer_records.rs` (6 sites: lines
7, 44, 45, 46, 309, 381) and 5 `Case{...}` fixture rows at
`composer_records.rs:452-458` (`hash-valid`, `hash-valid-zeros`,
`hash-63-chars`, `hash-66-chars`, `hash-uppercase`, `hash-empty`,
`hash-31-bytes` — 7 rows, all asserting the 64-char rule one way or another).

**Boundary note, to prevent scope confusion in the spec:** `me-cli` also has a
large, unrelated "hashlock" surface — `ms_codec::hashlock` preimage PLATES and
`phrase:` records (`sysw/record.rs`, `sysw/composer_records.rs:67-97`,
most of `main.rs`'s `hashlock_*` functions). That machinery derives a SHA-256
preimage/digest for the *ms1* secret-material container (a different repo,
mnemonic-secret's `ms-codec`) and is orthogonal to the `SpendPath.hash` /
`hash:` composer-record kind question — it was not touched by this recon
except where it collides with the `hash:` record's own 32-byte/64-hex
assumptions (§4).

### 1c. `seedhammer` — `SpendPath.Hash *[32]byte` (Go)

Definition: `md/compose.go:167`.

Commands (see script logic; false-positive families were identified and
excluded by hand, listed below so the count is checkable):
```
grep -rn '\.Hash\b' --include=*.go .
grep -rn 'Hash:' --include=*.go .
```
Excluded as unrelated to `SpendPath.Hash` (different field, same name):
`chainhash.Hash` (address/taproot_script_path.go), `seal.Payload.Hash`
(`[16]byte`, in `seal/open.go`, `seal/open_test.go`, `seal/unlock_key_test.go`,
`gui/unlock_flow.go`, `gui/unlock_kdf.go`, `gui/unlock_kdf_test.go`,
`gui/seal_fixture_test.go`), `hash.Hash` interface (`seal/pbkdf2.go`,
`picobin/picobin.go`), `sysw.ClassHash` / `HasHash` (unrelated identifiers that
substring-match), and Go `case` labels ending in `Hash:`
(`hashlockBackToWhichHash`, `composerFieldHash`).

| | Production | Test | Total |
|---|---:|---:|---:|
| Constructions (`.Hash = ...` or `Hash: ...` literal) | 9 | 51 | 60 |
| Reads (`== nil`, `!= nil`, `*p.Hash`, map index, etc.) | 30 | 25 | 55 |
| **Total** | **39** | **76** | **115** |

Production files (12): `md/compose.go` (4, all reads — the 2 predicate helpers
at 172/176, the validate check at 315, and the lowering read at 402),
`gui/composer_hash.go` (3 writes), `gui/composer_hashlock.go` (3 writes, 2
reads), `gui/composer_discard.go` (2 writes), `gui/composer_presets.go` (1
construction), `gui/composer_preimage_plate.go` (10 reads),
`gui/composer_state.go` (6 reads), `gui/composer_state_hook.go` (2 reads),
`gui/composer_selfcheck.go` (3 reads), `gui/composer_sources.go` (1 read),
`gui/composer_shape.go` (1 read), `gui/composer_consent.go` (1 read).

Test files (13): `composer_shape_test.go`, `composer_sources_test.go`,
`composer_selfcheck_test.go`, `composer_backleg_test.go`,
`composer_state_hook_test.go`, `composer_preimage_plate_test.go`,
`composer_copy_test.go`, `composer_gates_test.go`, `composer_flow_test.go`,
`composer_hashlock_test.go` (largest, 20 real code lines: 8 writes / 12
reads), `composer_provenance_test.go` (11: 8 writes / 3 reads),
`composer_discard_test.go`, `md/compose_test.go` (the `chs`/`ckl` helpers).

Internal consistency check performed: re-deriving the totals two independent
ways (raw grep minus classified exclusions/comments vs. per-file manual
tally) landed on the same 115/60/55 split both times.

---

## 2. `PolicyShape.Sha256Digests` and its consumers

Definition: `seedhammer/md/policy_shape.go:65` — `Sha256Digests [][32]byte`.
Populated only at `policy_shape.go:284-290`:

```go
case tagSha256:
    br.Hashlock = true
    if h, ok := n.body.(hash256Body); ok {
        br.Sha256Digests = append(br.Sha256Digests, [32]byte(h))
    }
    return true
case tagHash256, tagRipemd160, tagHash160:
    br.Hashlock = true
    return true
```

**Pre-existing gap, independent of this spec's change:** `Branch.Hashlock` is
already set `true` for all four tags, but `Sha256Digests` is appended to for
`tagSha256` alone. Any md1 payload built by hand (or via the general
`md compile` search path, not the fixed composer) containing a `hash256(...)`,
`ripemd160(...)`, or `hash160(...)` literal already produces a `PolicyShape`
where `Hashlock == true` and `len(Sha256Digests) == 0`. This is reachable
today because the underlying `to_miniscript.rs`/`script_emit.go` already
accept those three tags (§0) — only the *composer* is currently sha256-only.

Consumers (command: `grep -rn "Sha256Digests" --include=*.go . | grep -v /vendor/`):

| Site | What it assumes |
|---|---|
| `gui/composer_selfcheck.go:130-139` | Self-check re-decode: `wantHash := 0; if p.Hash != nil { wantHash = 1 }`, then `len(b.Sha256Digests) != wantHash` is an error, and (if 1) `b.Sha256Digests[0] != *p.Hash` byte-compares. Assumes exactly one kind exists and that kind is sha256; would need the kind carried through both sides of the comparison. |
| `gui/composer_consent.go:90-100` (`composerBranchLines`) | `for _, d := range b.Sha256Digests { ... "  hash " + composerDigestShort(d) }` renders each digest as a consent-screen row; separately, `len(b.Sha256Digests) == 0` is one of four conjuncts in the "is this path a bare sorted-multi" legality check — the Go mirror of Rust's `is_bare_multi()`/`is_bare_single()` (`compose/mod.rs:158-165`), which likewise gate on `hash.is_none()`. Because non-sha256 kinds never populate `Sha256Digests` today, a hand-built hash160/ripemd160-locked bare-multi path currently passes this `== 0` check even though `Hashlock` is `true` — i.e. this predicate is ALREADY wrong for the general decompose path, and the spec must decide explicitly whether the fixed field's replacement keys off `Hashlock` or off "does this path have a hash of any kind" rather than reproducing this len-based idiom. |
| `gui/composer_consent.go:190-203` | Gates a whole extra explainer paragraph (`composerCopyHashRule()`) on "does ANY branch have `len(Sha256Digests) > 0`" — payload-wide, printed once. |

All four consumers live in `gui/` (device consent/self-check screens); none
have a Rust-side counterpart (the Rust `PolicyShape`/decompose equivalent, if
one exists, was not searched — out of the three named codebases' composer
path, this is Go-only machinery per the "decompose" direction).

---

## 3. Test and vector coverage of the composer hash path

### Rust (descriptor-mnemonic)
- `crates/md-codec/tests/compose_lowering.rs`: 41 `#[test]` functions total: 6
  reference a hash helper (`with_hash`/`keyless`/`H1`) —
  `compose_refuses_a_policy_with_no_keyed_path`,
  `compose_refuses_a_keyless_path_under_tr`, `conjunct_order_is_keys_hash_lock`,
  `a_keyless_wsh_path_is_admitted_and_marked_experimental`,
  `presets_compose_and_carry_the_documented_shapes`,
  `presets_lower_to_their_pinned_templates`.
- `crates/md-codec/tests/compose_crosscheck.rs`: 5 tests total, 2 reference
  hash (`a_keyless_wsh_path_is_admitted_with_top_unsafe_and_refused_by_the_default_sanity`,
  `every_preset_passes_the_5b_cross_check`).
- `crates/md-codec/tests/vectors/`: **63 distinct vector stems**
  (`ls crates/md-codec/tests/vectors | sed -E 's/\.(bytes\.hex|descriptor\.json|phrase\.txt|template|conformance\.json)$//' | sort -u | wc -l`).
  Of these, **6 `.template` files contain `sha256(`**
  (`grep -l "sha256(" crates/md-codec/tests/vectors/*.template`):
  `keyed_wsh_timelock_hashlock`, `keyed_tr_pathological`,
  `keyed_compose_wsh_hash_and_time`, `keyed_compose_tr_hash_leaf`,
  `keyed_compose_preset_hashlock_gated`, `keyed_compose_wsh_timelock_hashlock`.
  **Zero contain `hash256(`, `ripemd160(`, or `hash160(`.**

### Go (seedhammer)
- `md/compose_test.go`: 9 `Test*` functions. The hash-bearing `SpendPath`
  builders (`chs`/`ckl`, `compose_test.go:30-31`) are used in a shared vector
  table consumed by `TestComposeReproducesEveryVectorByteForByte`, and again
  directly inside `TestComposeRefusesWhatThePrimaryRefuses` (lines 356-357)
  and `TestComposeExperimentalMarks` (line 440).
- `md/testdata/vectors/`: **91 distinct vector stems**
  (same stem-dedup command as above), of which **6 `.template` files contain
  `sha256(`** — the same six names as the Rust list, byte-identical content
  (this directory is vendored FROM the Rust one; see §5). **Zero contain
  `hash256(`, `ripemd160(`, or `hash160(`.**
  (Note the 91-vs-63 stem-count difference between the two repos' vector
  directories is real and measured, not a typo; §5 explains why — the Go
  directory holds vectors from more than one pinning mechanism, only some of
  which are sourced from the Rust corpus.)
- `gui/composer_hashlock_test.go` is the single largest hash-focused test
  file in the whole tree: 24 raw `.Hash`-pattern hits, 20 real code lines (8
  writes / 12 reads) after excluding 4 comment lines — this is where the
  device-side hashlock composer screens (derive/manual-entry/back-navigation)
  are exercised.

### Cross-language record-classifier vectors
`seedhammer/sysw/testdata/record_class_vectors.json` (vendored from
`crates/me-cli/src/sysw/composer_records.rs`'s `CASES` table, §5): **68 total
case rows**, of which **14 mention "hash" in some field** and **2 are
classified `"class": "Hash"`** (`grep -c '"hash'`, and
`grep -o '"class":\s*"Hash"' | wc -l`).

### Honest gap statement
No test or vector, in either language, at either the composer layer or the
sysw-record layer, exercises a `hash256`/`ripemd160`/`hash160` hashlock
**composed through `SpendPath`/`PathList`/`Compose()`**. The only coverage of
those three kinds anywhere in the three repos is the generic `Node`/`Tag`/`Body`
property tests named in §0 (`proptest_to_miniscript.rs`,
`compose_lowering.rs`'s sibling `common/mod.rs` generators) — i.e., the
underlying miniscript-lowering machinery is fuzzed for all four kinds, but
nothing that goes through a `SpendPath`, a CLI `--path`, a `hash:` record, or
a GUI composer screen has ever constructed a non-sha256 hashlock. This gap is
total, not partial.

---

## 4. Everything found that assumes 32 bytes / 64 hex chars, beyond the field's own type

This is the section the brief called most valuable; every entry below is a
place where a 20-byte digest (hash160/ripemd160) would either **panic** or
**silently misbehave** if the type is widened without also touching this site.
Each is marked which.

### 4a. Truncated hex display: `first-8..last-8`, hardcoded on a 64-char string

Five independent implementations of the identical `<first 8 hex>..<last 8
hex>` rendering rule, every one of them written against a hex string that is
**always exactly 64 characters** because its input parameter is `[32]byte`:

| Site | Signature | Slice |
|---|---|---|
| `seedhammer/gui/composer_hash.go:48-51` (`composerHashRow`) | `digest [32]byte` | `h[:8]`, `h[56:]` |
| `seedhammer/gui/composer_hash.go:171-175` (`composerHashPreimageRow`) | `digest [32]byte` | `h[:8]`, `h[56:]` |
| `seedhammer/gui/composer_hash.go:183-190` (`composerHashPhraseRow`) | `d *[32]byte` | `h[:8]`, `h[56:]` |
| `seedhammer/gui/composer_consent.go:61-64` (`composerDigestShort`) | `d [32]byte` | `h[:8]`, `h[56:]` |
| `seedhammer/gui/composer_hashlock.go:247-250` (`hashlockFirst8Last8`) | `h [32]byte` | `s[:8]`, `s[len(s)-8:]` |
| `mnemonic-engrave/crates/me-cli/src/main.rs:2270-2278` (`print_composer_confirmation`) | `hx = hex(&h)` where `h: [u8;32]` | `&hx[..8]`, `&hx[56..]` |
| `mnemonic-engrave/crates/me-cli/src/main.rs:2598-2602` (§8.2.3 orphan-check warning) | same | `&hx[..8]`, `&hx[56..]` |

**Failure mode: loud, not quiet.** A 20-byte digest hex-encodes to 40
characters. `h[56:]` on a 40-character Go string, or `&hx[56..]` on a 40-char
Rust `&str`, is a **slice-bounds panic** in both languages — not silent
corruption. This is worth stating precisely because the brief's framing
("breaks quietly rather than loudly") does not hold for these seven sites:
they crash. (The team has hit this exact bug shape before, for an unrelated
field: `gui/transaction.go:365-395` and `gui/transaction_picker_test.go:10-13`
document a prior incident where `c.tx.TxidDisplay[:8]` panicked on a
zero-length unconfirmed-tx display string — same "fixed slice into
variable/shorter content" class, different field.) All seven sites need a
kind-aware truncation (e.g. slice to `min(8, len/2)` / `len-min(8,len/2)`, or
an explicit per-kind rule) before a 20-byte kind can reach any of them.

### 4b. Hex-length / hex-body checks hardcoded to 64

| Site | Rule |
|---|---|
| `descriptor-mnemonic/crates/md-cli/src/cmd/compose.rs:134-147` (`parse_sha256_hex`) | `value.len() != 64` — backs `--path ...,sha256=HEX` and `--preset hashlock-gated,sha256=HEX`. The option name itself (`sha256=`) is also kind-specific; adding kinds means new option names or a generalized `hash=<kind>:<hex>` grammar — a decision, not a rename. |
| `mnemonic-engrave/crates/me-cli/src/sysw/composer_records.rs:302-309` (`parse_hash`) | `body.len() != 64` |
| `seedhammer/sysw/composer_records.go:191-198` (`ParseHashRecord`) — **independent third implementation of the same rule, on-device** | `len(body) != 64` |
| `seedhammer/gui/composer_hash.go:79-99` (`composerHexEntry`) — the on-device manual-entry keyboard screen | `valid := len(frag) == 64`, fixed `[32]byte` return type, fixed keyboard alphabet `composerHexKeys` (hex digits only, no kind selector) |
| `seedhammer/gui/composer_gates_test.go:630-631,643` | test mirrors the same `len(frag) == 64` rule |

`composer_hexEntry` is the concrete site design decision #2's "device asks on
manual entry" lands on: today it is a single 64-char hex pad with no kind
concept at all. Adding kind support here is a real UI decision (where does
the kind picker go — before or after hex entry; does the keyboard's expected
length change live as the operator types 40 vs 64 chars) not a mechanical
find-and-replace.

Three independent copies of "hash: needs exactly 64 lowercase hex" exist
(Rust host, Go on-device parser, Go on-device keyboard) plus the CLI's fourth
copy for `--path ...,sha256=`. All four must move together; per this
project's Rust-primary-for-Go-ports rule, the Go copies are downstream and any
normative change lands in `descriptor-mnemonic`/`mnemonic-engrave` (Rust)
first, with vectors, before the `seedhammer` Go ports change.

### 4c. Fixed-width types beyond the field declaration itself

- `descriptor-mnemonic/crates/md-codec/src/compose/presets.rs:90` — public
  function parameter `hashlock_gated(hash: [u8; 32], ...)`.
- `mnemonic-engrave/crates/me-cli/src/main.rs:2566` — `let hashes: Vec<[u8; 32]>
  = ...` (the §8.2.3 orphan-check accumulator; membership tested by
  `hashes.contains(&digest)` at line 2570, a **value-only equality** with no
  kind — see §4d).
- `seedhammer/md/policy_shape.go:65` — `Sha256Digests [][32]byte` (§2).
- `seedhammer/gui/composer_hash.go` — six function signatures over bare
  `[32]byte`/`*[32]byte` (`composerHashRow`, `composerPayloadDigests`,
  `composerHexEntry`, `composerHashInPayloadRow`, `composerHashPreimageRow`,
  `composerHashPhraseRow`) plus the `hashlockRow` struct
  (`preimage [32]byte; digest [32]byte`, line 196-197).
- `seedhammer/gui/composer_preimage_plate.go:56,78,90,145,232,499` — a struct
  field, a `map[[32]byte]bool` dedup set, a `[][32]byte` slice, and two more
  function signatures.
- `seedhammer/gui/composer_hashlock.go:53,136,174,192,212,235,247,385` — eight
  function signatures over `[32]byte`/`[][32]byte`.
- `seedhammer/gui/composer_state.go:53,81,308,337,346,367` — `phraseDigests
  map[[32]byte]struct{}` and `hashlockHeld map[[32]byte]hashlockMaterial`
  (§4d), plus a `hashlockMaterial.preimage [32]byte` field.
- `seedhammer/gui/composer_state_hook.go:48,56-60,81` — the JS-bridge hook
  `ComposerPathHashes() []*[32]byte`, exported to the web preview surface (a
  fourth boundary, beyond CLI/device/host, that encodes "a hash is 32 bytes").
- `seedhammer/gui/composer_presets.go:54-55`, `composer_discard.go:110` — two
  more local fixed-size locals.

`grep -c "\[u8; 32\]\|\[u8;32\]" --include=*.rs crates/` and
`grep -c "\[32\]byte" --include=*.go md/ gui/` were used to find these; raw
counts (**253** Rust hits, **147** Go hits repo-wide) are dominated by
unrelated 32-byte types (chain codes, secp256k1 scalars, codex32 alphabets,
RNG seeds) — the table above lists only the ones actually reachable from the
composer/sysw/GUI hashlock surface, confirmed by reading each site, not by
the raw grep count.

### 4d. Equality and map-keying on the bare digest, with no kind

This is the one place a **fixed 32-byte array plus a kind-derived length
genuinely does not fail loudly on its own** — it is a silent-correctness
question, not a panic, so it is called out on its own rather than folded into
4c:

- `seedhammer/gui/composer_state.go:53` — `phraseDigests map[[32]byte]struct{}`
- `seedhammer/gui/composer_state.go:81` — `hashlockHeld map[[32]byte]hashlockMaterial`
- `seedhammer/gui/composer_preimage_plate.go:78` — `seen := map[[32]byte]bool{}`
- `seedhammer/gui/composer_hashlock.go:240` — `if *p.Hash != h` (direct byte compare)
- `mnemonic-engrave/crates/me-cli/src/main.rs:2570` — `hashes.contains(&digest)` over `Vec<[u8;32]>`

If `HashLock{kind, digest}` keeps `digest` as a full, kind-derived-length-only
region of a fixed `[32]byte` (design decision #4), every one of these five
sites must be migrated to key/compare on the **whole** `HashLock` value (kind
+ digest), not the bare digest array, or a hash160 digest and a sha256 digest
that happen to share their first 20 bytes would collide in these maps/sets
and in the orphan-check membership test. This does not mean the fixed-array
choice "does not work" (a `HashLock{kind byte; digest [32]byte}` is still a
valid, comparable, hashable Go map key and Rust `Vec` element) — it means
these five specific call sites are exactly where an implementer would, by
habit, key on `digest` alone and reintroduce a cross-kind collision. Flagging
this was requested to "lead with" any such site; this is the one found.

---

## 5. Cross-language pinning

Two independent, hand-maintained pinning mechanisms touch this change:

### 5a. The compose-vector corpus pin
- Source of truth: `descriptor-mnemonic/crates/md-codec/tests/vectors/`
  (Rust). No file in descriptor-mnemonic itself pins a `seedhammer` commit —
  the relationship is one-directional (verified:
  `grep -rln "seedhammer\|bg002h" --include=*.rs --include=*.json --include=*.md crates/ design/`
  finds no such pin, only unrelated design-doc prose hits).
- Vendored by `seedhammer/scripts/vendor-compose-vectors.sh` into
  `seedhammer/md/testdata/vectors/`, alongside a generated pin file
  `seedhammer/md/testdata/compose_vectors.provenance.json` that records the
  exact descriptor-mnemonic commit (currently `9f637cf1142e7fcbe7f0a656429a78ad93f99496`),
  whether that source tree was clean, and a per-file SHA-256.
- Enforced by `seedhammer/md/compose_vectors_pin_test.go`:
  `TestComposeVectorsMatchTheirProvenancePin` asserts (a) the pin's declared
  vector count equals `len(composeVectorNames)` — a **33-entry hand-maintained
  Go slice literal**, `compose_vectors_pin_test.go:38-59` — (b) the pin lists
  **exactly 161 files** (hardcoded literal at line 90, with an inline comment
  explaining the arithmetic: 29 keyed vectors × 5 files + 4 unkeyed × 4 files),
  (c) every pinned file's SHA-256 matches disk, and (d) no stray
  `compose_*`/`keyed_compose_*` file exists in the directory unpinned.
  `TestEveryKeyedComposeVectorHasAConformanceRecord` additionally requires a
  `.conformance.json` for every `keyed_*` name in that same 33-entry list.
- **Of the 33 pinned composer-corpus names, 4 carry a hashlock**:
  `keyed_compose_tr_hash_leaf`, `keyed_compose_wsh_hash_and_time`,
  `keyed_compose_wsh_timelock_hashlock`, `keyed_compose_preset_hashlock_gated`.
  (The other 2 hashlock vectors found in §3 — `keyed_wsh_timelock_hashlock`
  and `keyed_tr_pathological` — are **not** in `composeVectorNames`;
  `isComposeVectorFile` at line 63 gates strictly on a `compose_`/
  `keyed_compose_` name prefix, so those two live in the vectors directory
  under a separate, older pinning scheme this recon did not chase further —
  flagged as unexplored, not asserted-absent.)
- **Found staleness in the generator's own output, not in the enforced test:**
  `scripts/vendor-compose-vectors.sh`'s embedded Python heredoc hardcodes the
  string `"if the file count is not 156"` into the JSON's `_comment` array
  regardless of the actual file count it just computed. The real Go test
  enforces **161**, and the JSON on disk today lists **161** `files` entries
  (`python3 -c "import json; print(len(json.load(open('md/testdata/compose_vectors.provenance.json'))['files']))"`
  -> 161). The comment string is simply wrong and has been since at least the
  last re-pin; do not trust that comment as a live invariant when re-pinning
  for this change — only the Go test's literal `161` (which itself needs
  hand-editing, at `compose_vectors_pin_test.go` line 90) governs.

**Cost of re-pinning for this change**, concretely: (1) author new composer
vectors for at least the three uncovered kinds in
`descriptor-mnemonic/crates/md-codec/tests/vectors/` and add matching entries
to whatever Rust-side manifest generates them; (2) add their stems to the
33-entry `composeVectorNames` Go slice by hand; (3) re-run
`vendor-compose-vectors.sh`, which recomputes every SHA-256 and regenerates
`compose_vectors.provenance.json` (all 161+N files re-hashed, not just the new
ones, since the script does not diff); (4) hand-edit the `161` literal in
`compose_vectors_pin_test.go` to the new total; (5) the `_comment` string will
again silently disagree with reality unless someone also fixes the generator
script — worth doing while in there, but not enforced by any test today.

### 5b. The record-classifier vector pin
- Source of truth: the `CASES: [Case; N]` table hardcoded in
  `mnemonic-engrave/crates/me-cli/src/sysw/composer_records.rs:432-515`,
  exported as `mnemonic-engrave/crates/me-cli/tests/testdata/record_class_vectors.json`
  (`FIXTURE_PATH`, `sysw_composer_records.rs:400`).
- Vendored into `seedhammer/sysw/testdata/record_class_vectors.json`, pinned
  by a sibling `record_class_vectors.provenance.json`
  (`seedhammer/sysw/composer_records_test.go:12-13`) — same
  commit/clean/SHA-256 pattern as §5a, a structurally separate pin.
- Coverage measured in §3: 68 total rows, 2 classified `Hash`. This is the
  vector set that would need new `hash-*` rows (and, per decision #2, new
  `phrase:`-adjacent rows if the manual-entry kind selector produces new
  record shapes) for the other three kinds, mirrored through this second
  independent vendoring/pin pipeline — distinct machinery from §5a, so it is
  a second re-pinning cost, not the same one paid twice.

---

## Design-decision cross-checks

Checked each of the four stated decisions against what was found; noting
confirmations as well as the one place worth a second look, since the brief
asked to flag contradictions:

1. **"All four kinds authorable from composer, Rust and device."** No
   contradiction found. §0 shows the wire/lowering machinery for all four
   kinds already exists in both languages; the composer layer is the only gap,
   consistently in both languages (§1a, §1c).
2. **"The `hash:` record carries the kind; device asks on manual entry; bare
   `hash:` means sha256."** Confirmed as the exact record and the exact
   on-device screen to change: `composer_records.rs`'s `HASH_PREFIX`/`parse_hash`
   (Rust host), `sysw/composer_records.go`'s `ParseHashRecord` (Go device
   parser — a **third independent implementation** of the same rule, §4b),
   and `gui/composer_hash.go`'s `composerHexEntry` (the on-device keyboard,
   §4b) are three separate call sites that must all gain a kind concept
   together. No contradiction found, but note the count: this is not one
   record parser to change, it is (at least) three.
3. **"20-byte kinds warn only when the device did not derive the preimage."**
   Consistent with the code as found: `hashlockDeriveFlow`
   (`seedhammer/gui/composer_hashlock.go:385-390`) only ever derives via
   `hashlock.PreimageSHA256` — there is no derivation path today that could
   produce anything but a 32-byte sha256 preimage/digest. So "device did not
   derive" and "kind is not sha256" are currently the same condition by
   construction; a 20-byte kind can only arise from manual entry, which
   matches the decision's premise exactly. No contradiction found.
4. **"`HashLock{kind, digest}` with a FIXED 32-byte array... because the
   device has 0-alloc paths under TinyGo."** No site was found where a fixed
   array plus kind-derived length structurally cannot work. The one place
   that will silently misbehave if not handled deliberately is §4d
   (map/set/equality keys on the bare `[32]byte` region, ignoring kind) —
   flagged there as requested, but it is a migration-discipline issue at five
   named call sites, not a reason the fixed-array approach is unworkable.

---

## What this recon did not chase further (stated rather than silently dropped)

- The two non-`composeVectorNames` hashlock vectors (`keyed_wsh_timelock_hashlock`,
  `keyed_tr_pathological`) are pinned by *some* mechanism (they exist and are
  exercised, per §3), but which pin governs them was not identified — noted
  in §5a rather than assumed.
- Whether `descriptor-mnemonic` has a Rust-side `PolicyShape`-equivalent
  decompose/consent path (a counterpart to §2's Go-only findings) was not
  searched; the three named codebases' "consumer of the composer" framing for
  item 2 pointed at Go (`policy_shape.go` is Go-only in this tree), and no
  Rust file named `policy_shape.rs` or similar was encountered during any of
  the searches run.
- `ms_codec::hashlock` itself (the mnemonic-secret crate `me-cli` depends on
  for preimage-plate/phrase derivation) was not opened — it lives in a fourth
  repo outside this recon's three-codebase scope, and its `preimage_sha256`/
  `preimage_hardened` functions were read only insofar as their call sites in
  `me-cli` appear above.
