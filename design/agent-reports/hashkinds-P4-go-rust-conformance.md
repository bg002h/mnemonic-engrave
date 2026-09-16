# Hashlock kinds — cross-language conformance (Go port vs Rust primaries), P4

**VERDICT: GREEN on live conformance — 0 Critical, 0 divergence found in digests, wire
tags, script lowering, QR text, or the §6 record grammar across all four kinds
(sha256/hash256/ripemd160/hash160), verified by running real code on both sides against
an independent third-party oracle and against each other. 2 Important findings, both
about MISSING regression coverage (not a live bug) in the Go repo's own test suite.**

Repos and revisions at the time of this review:

| repo | HEAD | notes |
| --- | --- | --- |
| `seedhammer` (Go, downstream) | `285f7a99a924d6b9fc4c056d37a82b804ebfb2f7` | branch `hashkinds-p4` |
| `descriptor-mnemonic` (Rust, codec/lowering primary) | `745e0fd0b8b714564e762fc4fbfd7cb2e5d3cdef` | |
| `mnemonic-secret` (Rust, hashlock digest primary) | `92e781e95c2be4d1df6d19b1a2091d30798c08d0` | |
| `mnemonic-engrave` (Rust, §6 record-grammar primary) | `935cb2038104d19e98c97a8668f3227ae4ddd7b3` | |

Toolchain: Go 1.26.7 (`/scratch/code/shibboleth/.toolchain/go/bin`), Rust
`cargo 1.97.0-nightly`, Python 3.14.7 (`hashlib` — `ripemd160` **is** available via
OpenSSL's legacy provider on this machine; not disabled, so item 1's independent oracle
did not need a workaround).

Method: for every item below, real code was **run** on both sides (or against a scratch
crate/module that imports the actual package under review) rather than compared by
reading. All scratch code lives under `/tmp/hashkinds-check/` and is not part of either
repo; nothing in either repo was modified.

---

## 1. Digest functions vs an independent oracle (python3 `hashlib`)

Three preimages, all four digest functions, computed three ways: Go
(`hashlock.DigestSHA256/DigestHash256/DigestRIPEMD160/DigestHash160` **and** the named
dispatch `hashlock.DigestOf`), Rust (`ms_codec::hashlock::digest_sha256/digest_hash256/
digest_ripemd160/digest_hash160`), and Python (`hashlib.sha256` / double `sha256` /
`hashlib.new('ripemd160')` / `ripemd160(sha256(x))`).

Python oracle sanity-checked first against the official RIPEMD-160 test vectors:
`RIPEMD160('')=9c1185a5c5e9fc54612808977ee8f548b2258d31`,
`RIPEMD160('a')=0bdc9d2d256b3ee9daae347be6f4dc835a467ffe`,
`RIPEMD160('abc')=8eb208f7e05d987a9b044a8e98c6b087f15a0bfc` — all three matched, so the
oracle itself is trustworthy.

Commands:
```
python3 <inline script computing hashlib sha256/ripemd160 for 3 preimages> \
  > /tmp/hashkinds-check/python_oracle.json
cd /tmp/hashkinds-check/go-digest   && go run .   # imports seedhammer.com/hashlock via replace
cd /tmp/hashkinds-check/rust-digest && cargo run  # path-deps ms-codec directly
```

| preimage (hex, first 8 bytes shown) | fn | Go | Rust | python3 | match |
| --- | --- | --- | --- | --- | --- |
| `abababab…` (32×0xab) | sha256 | `9a2db2e2…af885` | `9a2db2e2…af885` | `9a2db2e2…af885` | Y |
| `abababab…` | hash256 | `88b8f02c…a2691` | `88b8f02c…a2691` | `88b8f02c…a2691` | Y |
| `abababab…` | ripemd160 | `5786aabc…f43f4` | `5786aabc…f43f4` | `5786aabc…f43f4` | Y |
| `abababab…` | hash160 | `e81bfa71…988e95` | `e81bfa71…988e95` | `e81bfa71…988e95` | Y |
| `00000000…` (32×0x00) | sha256 | `66687aad…f2925` | `66687aad…f2925` | `66687aad…f2925` | Y |
| `00000000…` | hash256 | `2b32db6c…71 51e` | `2b32db6c…7151e` | `2b32db6c…7151e` | Y |
| `00000000…` | ripemd160 | `d1a70126…40ff842` | `d1a70126…40ff842` | `d1a70126…40ff842` | Y |
| `00000000…` | hash160 | `b8bcb07f…fdbbc6` | `b8bcb07f…fdbbc6` | `b8bcb07f…fdbbc6` | Y |
| `d669b92e…` (sha256 of an arbitrary string) | sha256 | `96d88c6a…410a7bc` | `96d88c6a…410a7bc` | `96d88c6a…410a7bc` | Y |
| `d669b92e…` | hash256 | `7ffef7a8…d1f190d` | `7ffef7a8…d1f190d` | `7ffef7a8…d1f190d` | Y |
| `d669b92e…` | ripemd160 | `41820b3c…c559f5a9` | `41820b3c…c559f5a9` | `41820b3c…c559f5a9` | Y |
| `d669b92e…` | hash160 | `f3ae6681…08a5743` | `f3ae6681…08a5743` | `f3ae6681…08a5743` | Y |

Full 64/40-char hex values are in `/tmp/hashkinds-check/python_oracle.json` and the two
program outputs; the Go run additionally re-checked all 12 rows through the named
dispatch `DigestOf(kind, x)` (12 more rows, all `[OK]`) so the dispatch table itself,
not only the four leaf functions, is exercised — **24 Go comparisons + 12 Rust
comparisons, 36/36 match.**

**Also verified:** the QR text builder (`hashlock.QRText` / `ms_codec::hashlock::qr_text`)
— named in scope as part of the `hashlock` package — for 4 (hardened, kind, phrase)
combinations covering both methods and all four kinds. Byte-for-byte identical output
on both sides, including the `method:` line derived from `HASHLOCK_ITERATIONS`/`SALT`/
`DKLEN` and the per-kind `hash: <token>` line. 4/4 match.

## 2. Wire tags

Static comparison, then an end-to-end interop test (below) that would fail immediately
if either side's constant were wrong:

| kind | Rust `md-codec` (`tag.rs`) | Go `md` (`md.go`) | match |
| --- | --- | --- | --- |
| sha256 | `Tag::Sha256` = `0x1D` | `tagSha256` = `0x1D` | Y |
| hash160 | `Tag::Hash160` = `0x1E` | `tagHash160` = `0x1E` | Y |
| hash256 | `Tag::Hash256` = `0x1F` | `tagHash256` = `0x1F` | Y |
| ripemd160 | `Tag::Ripemd160` = `0x20` | `tagRipemd160` = `0x20` | Y |

Commands: `grep -n "0x1D\|0x1E\|0x1F\|0x20" crates/md-codec/src/tag.rs` (Rust) vs
`grep -n "tagSha256\|tagHash160\|tagHash256\|tagRipemd160" md/md.go` (Go).

The stronger check is item 3 below: Go **decoded** md1 chunk strings that Rust
**encoded** for hash256/ripemd160/hash160 policies and got byte-identical P2WSH
addresses. If Go's tag constant for any of the three new kinds were wrong, decoding
would either fail outright or silently read the wrong fragment — it did neither, for
all three kinds, across 2 chains × 2 indices each.

## 3. The lowering (script emission)

This is the strongest check in the review: real Rust-generated compose vectors,
**not yet vendored into the Go repo** (see Important-1), copied to a scratch directory
and fed through Go's actual script-emission function
(`md.EmitWitnessScriptChunks`, the same one `TestPkhWitnessScriptsReproduceRustsAddresses`
already uses for the pre-existing sha256 vector), then compared against the P2WSH
addresses `descriptor-mnemonic`'s test corpus recorded for the same descriptor.

Source vectors: `descriptor-mnemonic/crates/md-codec/tests/vectors/
keyed_compose_preset_hashlock_gated{,_hash256,_ripemd160,_hash160}.{phrase.txt,
conformance.json}` (commit `af3ba0b2`, "one hashlock-gated vector per hash kind").

Command:
```
cd /tmp/hashkinds-check/go-crosscheck && go run .
```

| vector (kind) | chain | idx | Go address | Rust address | script hex | match |
| --- | --- | --- | --- | --- | --- | --- |
| sha256 | 0 | 0 | `bc1qal27e93…u2kfj` | `bc1qal27e93…u2kfj` | `…88a82012088a820a8a8…876776…` | Y |
| sha256 | 0 | 1 | `bc1qujrt5ps…faazh6` | `bc1qujrt5ps…faazh6` | (analogous, distinct key hash) | Y |
| sha256 | 1 | 0 | `bc1qzgx03pm…99fjdf` | `bc1qzgx03pm…99fjdf` | — | Y |
| sha256 | 1 | 1 | `bc1qtzuhgzh…5dahs72vv3w` | `bc1qtzuhgzh…5dahs72vv3w` | — | Y |
| hash256 | 0 | 0 | `bc1qu9vp7sz…4q7zcrdl` | `bc1qu9vp7sz…4q7zcrdl` | `…88a82012088aa2098a20fc25…876776…` (`OP_HASH256`=`0xaa`) | Y |
| hash256 | 0 | 1 | `bc1q20atv5v…hzjws6s6j4c` | `bc1q20atv5v…hzjws6s6j4c` | — | Y |
| hash256 | 1 | 0 | `bc1qugtfdkh…jsm2g5qzprgp9` | `bc1qugtfdkh…jsm2g5qzprgp9` | — | Y |
| hash256 | 1 | 1 | `bc1qx7jwwkm…6m47qpyle2n` | `bc1qx7jwwkm…6m47qpyle2n` | — | Y |
| ripemd160 | 0 | 0 | `bc1q27yrquz…0tmhqf6zw6u` | `bc1q27yrquz…0tmhqf6zw6u` | `…88a82012088a61409e7bb5051…876776…` (`OP_RIPEMD160`=`0xa6`, 20-byte push) | Y |
| ripemd160 | 0 | 1 | `bc1q055s9vj…pf4t8c` | `bc1q055s9vj…pf4t8c` | — | Y |
| ripemd160 | 1 | 0 | `bc1qf3sdwxj…q7x34nt` | `bc1qf3sdwxj…q7x34nt` | — | Y |
| ripemd160 | 1 | 1 | `bc1qav0hndh…qwc3r5p0s8h3et0` | `bc1qav0hndh…qwc3r5p0s8h3et0` | — | Y |
| hash160 | 0 | 0 | `bc1qzme9m8p…sle67fr` | `bc1qzme9m8p…sle67fr` | `…88a82012088a914b5b72c0e6896…876776…` (`OP_HASH160`=`0xa9`, 20-byte push) | Y |
| hash160 | 0 | 1 | `bc1q9x4j0k2…scqnj0k` | `bc1q9x4j0k2…scqnj0k` | — | Y |
| hash160 | 1 | 0 | `bc1qt2jecas…rqnluu3v` | `bc1qt2jecas…rqnluu3v` | — | Y |
| hash160 | 1 | 1 | `bc1qnp8vuhz…yw6psgemyg6` | `bc1qnp8vuhz…yw6psgemyg6` | — | Y |

**16/16 addresses match** — since a P2WSH address is `sha256(witness_script)`, this is
proof (to cryptographic-collision odds) that Go's emitted witness scripts are
byte-identical to what Rust's descriptor compiles to, for all four kinds, both key
positions, both chains. Full script hex is in the tool output
(`/tmp/hashkinds-check/go-crosscheck`); confirmed by eye that the fixed opcode positions
move exactly as F1 requires (`OP_SIZE OP_2 OP_EQUALVERIFY <hashop> <push> OP_EQUAL`,
`0x82 …88 <hashop> … 87`) and only `<hashop>` (`0xa8`/`0xaa`/`0xa6`/`0xa9`) and the push
width (32 vs 20 bytes) move between kinds.

## 4. The §6 record grammar

Ran 21 hand-built boundary strings through the **real parsers on both sides** —
`mnemonic_engrave::sysw::composer_records::parse` (Rust, via a scratch crate that
path-deps the `mnemonic-engrave` lib crate) and `seedhammer.com/sysw.ParseHashRecord`
(Go, via a scratch module with a `replace` to the local checkout) — plus a producer
round-trip (`hash_record`/`HashRecord`) for all four kinds.

Commands:
```
cd /tmp/hashkinds-check/rust-record && cargo run --quiet
cd /tmp/hashkinds-check/go-record   && go run .
```

| input | Rust `parse()` | Go `ParseHashRecord` | match | what it tests |
| --- | --- | --- | --- | --- |
| `hash:0000…00` (64×`0`) | `OK sha256` | `OK sha256` | Y | bare defaults to sha256 |
| `hash:a8a8…a8` (64 hex) | `OK sha256` | `OK sha256` | Y | bare, non-trivial digest |
| `hash:sha256:a8a8…a8` | `OK sha256` | `OK sha256` | Y | explicit `sha256:` accepted on **input** |
| `hash:hash256:a8a8…a8` (64 hex) | `OK hash256` | `OK hash256` | Y | tagged, correct width |
| `hash:ripemd160:09e7…46b` (40 hex) | `OK ripemd160` | `OK ripemd160` | Y | tagged, correct width |
| `hash:hash160:b5b7…0bd` (40 hex) | `OK hash160` | `OK hash160` | Y | tagged, correct width |
| `hash:ripemd160:09e7…46` (39 hex, one short) | `ERR Hash(Some(Ripemd160))` | `ERR Hash(Some(?))` | Y (same class) | wrong width, known kind |
| `hash:HASH256:a8a8…a8` | `ERR Hash(None)` | `ERR Hash(None)` (`ErrHashKind`) | Y | **wrong case rejected, never folded** |
| `hash:Hash160:b5b7…0bd` | `ERR Hash(None)` | `ERR Hash(None)` | Y | mixed-case kind rejected |
| `hash:ripemd160:B5B7…0BD` (uppercase hex) | `ERR Hash(Some(Ripemd160))` | `ERR Hash(Some(?))` | Y | uppercase hex rejected under a known kind |
| `hash:sha512:a8a8…a8` | `ERR Hash(None)` | `ERR Hash(None)` | Y | **unknown kind refused, NEVER read as sha256 (fail-closed)** |
| `hash:hash256:a8a8…a8` (63 hex, one short) | `ERR Hash(Some(Hash256))` | `ERR Hash(Some(?))` | Y | wrong width |
| `hash:hash256:a8a8…a8a8` (66 hex, two long) | `ERR Hash(Some(Hash256))` | `ERR Hash(Some(?))` | Y | wrong width, too long |
| `hash:ripemd160:…0bd00` (42 hex) | `ERR Hash(Some(Ripemd160))` | `ERR Hash(Some(?))` | Y | wrong width, too long |
| `hash:sha256:` (empty digest) | `ERR Hash(Some(Sha256))` | `ERR Hash(Some(?))` | Y | empty tagged digest |
| `hash:` (empty, no colon) | `ERR Hash(Some(Sha256))` | `ERR Hash(Some(?))` | Y | empty bare digest, defaults to sha256 then fails width |
| `hash:sha256:sha256:a8a8…a8` (double tag) | `ERR Hash(None)` | `ERR Hash(None)` | Y | last-colon split behaves identically under a pathological body |
| `hash:hash160:zz5b…0bd` (invalid hex char) | `ERR Hash(Some(Hash160))` | `ERR Hash(Some(?))` | Y | non-hex char rejected |
| `hash:ripemd160: 09e7…46b` (leading space) | `ERR Hash(Some(Ripemd160))` | `ERR Hash(Some(?))` | Y | whitespace rejected |
| `hash:sha256:a8a8…a8` (63 hex) | `ERR Hash(Some(Sha256))` | `ERR Hash(Some(?))` | Y | odd-length hex rejected |
| `notahash:a8a8…a8` | `NONE` (not one of the 4 prefixes) | `NONE` (harness only, see note) | Y | sanity check of the harness, not a real class boundary |

**21/21 rows agree** on accept/reject and, when accepted, on kind and digest; on
rejection, on whether the token was unknown (`Hash(None)`/`ErrHashKind`) or the width
was wrong under a known kind (`Hash(Some(kind))`/`ErrHashRecord`) — Go's sentinel
doesn't carry the kind value inside the error (Minor, see below), but the two-way
classification matches every time.

**Fail-closed property directly exercised and holding**: `hash:sha512:…` (an unknown
kind token) is refused by both sides with the "unknown kind" error class, never
silently accepted as sha256 — this is the exact property SPEC_hashlock_kinds §6
requires ("an old parser must never read a non-sha256 digest AS sha256").

Producer round-trip (`hash_record`/`HashRecord`), all four kinds, output compared
character-for-character:
```
sha256    -> hash:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8
hash256   -> hash:hash256:a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8
ripemd160 -> hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
hash160   -> hash:hash160:b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```
Identical on both sides for all four — confirms "bare for sha256, tagged for the other
three, and `hash:sha256:` is never emitted" on both sides.

**Corroboration, not just self-consistency**: the vendored 81-row
`sysw/testdata/record_class_vectors.json` independently contains the same boundary
shapes (`hash-unknown-token`, `hash-kind-uppercase`, `hash-kind-mixedcase`,
`hash-*-wrong-width`, `hash-sha256-explicit`, one valid row per kind) with `host_line`
refusal text baked in from the Rust side, and its recorded outcomes agree with what the
scratch harness found independently. 20 of the 81 rows are hash-related; 6 accept
(`class: Hash`), 14 (of the hash-prefixed subset) refuse.

## 5. Vendored fixture pins

| file | recorded sha256 | actual sha256 (vendored copy) | sha256 of source file in Rust repo at pinned commit | sha256 of source file at Rust repo's current HEAD | verdict |
| --- | --- | --- | --- | --- | --- |
| `seedhammer/sysw/testdata/record_class_vectors.json` (81 rows) | `d6766fdd…9b928dd` | `d6766fdd…9b928dd` | `d6766fdd…9b928dd` (at `73f3e7d8` in `mnemonic-engrave`) | `d6766fdd…9b928dd` (at `935cb203`) | **fresh — no drift** |
| `seedhammer/hashlock/testdata/hashlock-v0.8.json` | `0a911f78…9bc3d8ce` | `0a911f78…9bc3d8ce` | `0a911f78…9bc3d8ce` (at `7cdcd711` in `mnemonic-secret`) | `0a911f78…9bc3d8ce` (at `92e781e9`) | **fresh — no drift** |

Commands:
```
sha256sum seedhammer/sysw/testdata/record_class_vectors.json
git -C mnemonic-engrave show 73f3e7d8:crates/me-cli/testdata/record_class_vectors.json | sha256sum
git -C mnemonic-engrave merge-base --is-ancestor 73f3e7d8 HEAD && echo YES
sha256sum seedhammer/hashlock/testdata/hashlock-v0.8.json
git -C mnemonic-secret show 7cdcd711:crates/ms-codec/tests/vectors/hashlock-v0.8.json | sha256sum
git -C mnemonic-secret merge-base --is-ancestor 7cdcd711 HEAD && echo YES
```
Both pinned commits are ancestors of their repo's current HEAD, and the file has not
changed since the pin (same hash at the pinned commit as at HEAD). **Neither of the two
fixtures explicitly named in the task is stale.**

The fixture that **is** stale is a different one, not named in the task's item 5 — see
Important-1.

## Test-suite gates run (all currently green; command + result)

```
go test -count=1 ./md/... ./sysw/... ./hashlock/...                  # seedhammer: ok, ok, ok
cargo nextest run --locked -p md-codec                                # descriptor-mnemonic: 524 passed, 2 skipped
cargo nextest run --locked -p ms-codec                                 # mnemonic-secret: 200 passed, 0 skipped
cargo nextest run --locked -p mnemonic-engrave --test sysw_composer_records  # mnemonic-engrave: 25 passed, 1 skipped
cargo nextest run --locked -p mnemonic-engrave sysw                    # mnemonic-engrave: 113 passed, 539 skipped
cargo nextest run --locked -p md-codec --test compose_hashkinds        # descriptor-mnemonic: 5 passed (Rust's OWN compose-kind self-tests)
go test -count=1 -run 'TestComposeVectorsMatchTheirProvenancePin|TestEveryKeyedComposeVectorHasAConformanceRecord' -v ./md/...   # PASS, PASS
```

---

## Findings

### Critical
None.

### Important

**I-1 — The Go repo's vendored compose-vector corpus predates the three new hash-kind
preset vectors, so its currently-green pin test provides ZERO regression coverage of
hash256/ripemd160/hash160 lowering.**

`seedhammer/md/testdata/compose_vectors.provenance.json` pins the vendored corpus to
`descriptor-mnemonic` commit `9f637cf1142e7fcbe7f0a656429a78ad93f99496` (156 files).
The three new per-kind vectors
(`keyed_compose_preset_hashlock_gated_{hash256,ripemd160,hash160}.*`) were added in
`descriptor-mnemonic` commit `af3ba0b2` ("vectors: one hashlock-gated vector per hash
kind, and the help text"), which sits strictly **after** the pinned commit in that
repo's history (`git log 9f637cf1..HEAD -- crates/md-codec/tests/vectors/` shows
`af3ba0b2` as the only vectors-touching commit in that range). `grep -rl
"hash256\|ripemd160\|hash160" seedhammer/md/testdata/vectors/*.descriptor.json` returns
nothing — none of the three new vectors have been vendored to Go yet.

`go test -run TestComposeVectorsMatchTheirProvenancePin` and
`TestEveryKeyedComposeVectorHasAConformanceRecord` both currently **PASS** — correctly,
since the pin matches its own (stale) corpus — but that green result says nothing about
whether Go's lowering is correct for the three new kinds, because those vectors simply
aren't in the corpus it checks.

This review closed the resulting gap out-of-band (§3 above: 16/16 addresses matched
using the un-vendored Rust vectors copied to a scratch directory), so **there is no live
divergence today**. But that check exists only in this review's scratch files
(`/tmp/hashkinds-check/`, not committed anywhere) — the repo's own gate would not catch
a future regression in the three new kinds' lowering until the corpus is re-vendored.

*Recommendation*: re-run `scripts/vendor-compose-vectors.sh` against current
`descriptor-mnemonic` (or at least `af3ba0b2` or later) so the 3 new-kind presets enter
the pinned 156(+)-file corpus, the way `keyed_compose_wsh_hash_and_time` already covers
sha256.

**I-2 — Go's own `Compose()` builder (as opposed to decode+emit) has never been
exercised with the three new kinds in any committed Go test.**

`seedhammer/md/compose_test.go`'s `composeFamily()` — the table
`TestComposeReproducesEveryVectorByteForByte` and its siblings run over — has exactly
one hash-bearing row (`keyed_compose_wsh_hash_and_time`), and its digest is
`composeH`, hardcoded: `NewHashLock(KindSha256, ...)` at the top of the file. A
repo-wide `grep -rl "KindHash256\|KindRipemd160\|KindHash160" seedhammer/md/*_test.go`
returns **no files** — no Go test anywhere calls `Compose()` with a `HashLock` of the
three new kinds. (`testdata_test.go` references `tagHash256`/`tagRipemd160`/
`tagHash160`, but only in a decode-side tag-name lookup table, not a compose test.)

By contrast, `descriptor-mnemonic/crates/md-codec/tests/compose_hashkinds.rs` is a
dedicated 5-test file on the Rust side (`every_kind_maps_to_its_own_wire_tag`,
`lowering_emits_the_kinds_tag_and_its_own_width`,
`every_kind_round_trips_compose_to_md1_to_template`, etc.) — Rust tests its own
compose-path for all four kinds; Go does not.

This review's §3 cross-check went through Go's **decode + script-emission** path
(`EmitWitnessScriptChunks` on Rust-produced md1 chunks), which does exercise the same
`emitFragment` opcode logic `Compose()`'s output would eventually reach, and reading
`compose.go`'s `pathBody` (the kind-to-`Body`-type switch, ~10 lines) shows nothing
Compose()-specific beyond what decode already proves. So the residual risk is low, but
the gap is real: nothing in the committed suite proves Go's `PathList → Compose() →
Node tree` step is correct for a freshly-authored (not decoded-from-wire)
hash256/ripemd160/hash160 policy, which is exactly the path the composer GUI/CLI takes
when an operator types a new hashlock.

*Recommendation*: add one `composeFamily()`-style row per new kind (or a small
dedicated test alongside the existing `chs`/`composeH` helper) exercising `Compose()`
directly, mirroring what `compose_hashkinds.rs` already does in Rust.

### Minor

**M-1 — Go's `ErrHashRecord` sentinel doesn't carry which kind the width check failed
under; Rust's `ComposerRecordError::Hash(Some(kind))` does.** Functionally both refuse
identically (verified, §4), and Go's caller-facing `sysw`-level doc comment
(`ErrHashRecord`) says the §8n line "names that kind's own width" — that line is
rendered elsewhere in the device UI using the kind recovered before the error is
raised, so this is not a behavior gap, just a structural difference in the Go error
type versus Rust's enum. No user-visible effect found.

**M-2 — Spec text is now stale relative to implementation.**
`design/SPEC_hashlock_kinds.md` §1's gap table says (as of "today," 2026-09-14) the `md
compose` CLI has "only a sha256= option." `descriptor-mnemonic/crates/md-cli/src/
main.rs:289` already documents a `sha256`/`hash256`/`ripemd160`/`hash160` argument for
compose as of the reviewed HEAD (`745e0fd0`). This is expected drift for an in-flight
spec during an implementation cycle, not a defect, but worth a closing pass when the
cycle wraps.

**M-3 — `notahash:` harness row (§4, last table row) is a harness sanity check, not a
real cross-language comparison.** Go's `ParseHashRecord` is hash-specific and Rust's
`parse()` is a 4-way dispatcher, so feeding a non-`hash:`-prefixed string exercises
different code on each side by construction; both correctly report "not a record of
this class," but the row proves nothing beyond "the two harnesses were wired up
correctly." Included for completeness/transparency, not as evidence.

---

## Scratch artifacts (not part of any repo, for reproduction)

```
/tmp/hashkinds-check/python_oracle.json          -- independent digest oracle (§1)
/tmp/hashkinds-check/go-digest/                  -- Go digest + QR-text check (§1)
/tmp/hashkinds-check/rust-digest/                -- Rust digest + QR-text check (§1)
/tmp/hashkinds-check/vectors/                    -- copies of un-vendored Rust compose vectors (§3)
/tmp/hashkinds-check/go-crosscheck/              -- Go EmitWitnessScriptChunks vs Rust addresses (§3)
/tmp/hashkinds-check/boundary_cases.json         -- 21 §6 record-grammar boundary strings (§4)
/tmp/hashkinds-check/rust-record/, go-record/    -- record-grammar parse + producer checks (§4)
```
