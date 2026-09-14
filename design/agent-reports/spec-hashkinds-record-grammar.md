# RECON: the `hash:` payload record's grammar, end to end, and the hash-KIND touch surface

Facts only, file:line cited. No design opinions below except where explicitly
flagged as "friction with a stated design decision" per the brief.

## 1. The grammar as written, byte for byte

**Prefix**: `hash:` (literal, case-sensitive). **Separator**: none — the prefix
IS the separator (`str::strip_prefix` / `strings.CutPrefix`). **Body**: exactly
64 characters, every one of them lowercase `0-9a-f` (strict — uppercase is a
different, invalid string, not folded). **Decoded meaning**: the body IS the
32-byte digest itself in hex, not hex-of-hex like `key:`/`now:`/`phrase:`.
**No checksum, no length field, no kind field, no version marker** — the
length check (`== 64`) is the only structural rule.

Rust — `crates/me-cli/src/sysw/composer_records.rs`:
```rust
30: pub const HASH_PREFIX: &str = "hash:";
...
200: pub fn hash_record(digest: &[u8; 32]) -> String {
201:     format!("{HASH_PREFIX}{}", hex_lower(digest))
202: }
...
302: fn parse_hash(body: &str) -> Result<ComposerRecord, ComposerRecordError> {
303:     if body.len() != 64 {
304:         return Err(ComposerRecordError::Hash);
305:     }
306:     let bytes = unhex_lower(body).ok_or(ComposerRecordError::Hash)?;
307:     let mut h = [0u8; 32];
308:     h.copy_from_slice(&bytes);
309:     Ok(ComposerRecord::Hash(h))
310: }
```
`unhex_lower` (lines 178-192) enforces even length + `is_ascii_digit() || 'a'..='f'` only — uppercase is rejected, not lowercased.

Go — `seedhammer/sysw/composer_records.go`:
```go
27:  HashPrefix = "hash:"
...
191: // ParseHashRecord: exactly 64 lowercase hex characters.
192: func ParseHashRecord(record string) ([32]byte, error) {
193:     body, ok := strings.CutPrefix(record, HashPrefix)
194:     if !ok || len(body) != 64 {
195:         return [32]byte{}, ErrHashRecord
196:     }
197:     b, ok := unhexLower(body)
198:     if !ok {
199:         return [32]byte{}, ErrHashRecord
200:     }
201:     var h [32]byte
202:     copy(h[:], b)
203:     return h, nil
204: }
```
`unhexLower` (lines 178-190) is the byte-for-byte port of the Rust rule.

The value carried, `ComposerRecord::Hash([u8; 32])` (composer_records.rs:108),
becomes `Class::Hash` (record.rs:71 / Go `ClassHash`, record.go:53) — one
32-byte digest, one class, one length, unconditionally.

## 2. The full path from payload bytes to a digest reaching a screen

1. **Classification** (per record, at payload load): `sysw::classify_with` (Rust, `crates/me-cli/src/sysw/mod.rs:305`) / `sysw.Classify` (Go, `seedhammer/sysw/record.go:100`) tries `composer_records::parse`/`IsComposerRecord`+`classifyComposer` BEFORE the free-text/mnemonic/md1/mk1/mt1/descriptor sniffers (Rust: mod.rs:338-345; Go: record.go:131-133 dispatching to `classifyComposer`, composer_records.go:104-122).
2. **Device session load**: `syswSession.load` (`seedhammer/gui/sysw_session.go:83`) calls `sysw.Classify(r)` per record (line 114) and stores the class alongside the raw body string.
3. **Admission gate**: `progWalletPolicy`'s row in `gui/sysw_admit.go:64-81` admits `sysw.ClassHash: true` (line 71) — the only program row that does; every other program's table omits it.
4. **Extraction**: `composerPayloadDigests` (`gui/composer_hash.go:50-71`) filters the session's records for `r.class == sysw.ClassHash` (line 63) and calls `sysw.ParseHashRecord(r.body)` (line 66) to get the `[32]byte`.
5. **Screen rows**: `composerHashRows` (`gui/composer_hash.go:305-336`) builds "Which hash?" band 1 from these digests (`composerHashRow`/`composerHashInPayloadRow`, lines 46-49, 158-163).
6. **Selection → storage**: `composerHashEdit` (`gui/composer_hash.go:385-...`) writes the chosen digest into `st.list.Paths[idx].Hash = &d` (line 414/448).
7. **Lowering to script**: `md/compose.go`'s `pathBody` (lines 391-414) reads `p.path.Hash` (a `*[32]byte`, declared at `md/compose.go:167` in `type SpendPath struct`) and unconditionally emits `node{tag: tagSha256, body: hash256Body(*h)}` (line 403) — no kind is read anywhere in this hop because none exists on the type.

Host-side (no screen, but the same class reaches `me sysw pack`): `pack_deterministic_with` → `split` (`crates/me-cli/src/sysw/mod.rs:549`) → `admit_check` (`mod.rs:531`, calls `classify_with` per record at line 533) → `now_indices`/partition into secret vs public (`Class::Hash.is_secret()` is `false`, record.rs confirms `Hash` is absent from the `is_secret` match arm at record.rs:90-100, so a `hash:` record is always packed in the PUBLIC section, sealed or not).

## 3. What rejects a malformed one, and device inertness

**Rejection is length-first, then charset**: body length != 64 → `ComposerRecordError::Hash` / `ErrHashRecord`, one error for every failure mode (wrong length, uppercase, empty, non-hex char) — the parser never distinguishes *why* beyond "not exactly 64 lowercase hex characters" (composer_records.rs:144; Go's single `ErrHashRecord` string, composer_records.go:41).

**Host line** (`me sysw pack` stderr), `composer_records.rs:144`:
```
record N: hash: must be exactly 64 hex characters
```
Confirmed in the spec at `design/SPEC_wallet_policy_composer.md` §8n (line ~809-816):
```
> record N: hash: must be exactly 64 hex characters
```

**Device: confirmed inert.** `SPEC_wallet_policy_composer.md` §6a (line ~280-286) states it and the code matches: a malformed `hash:` classifies `Class::Unknown`/`ClassUnknown` (mod.rs:344 `Err(_) => Class::Unknown`; composer_records.go's `classifyComposer` default-falls-through to `ClassUnknown`, composer_records.go:121). `gui/composer_hash.go:53-56` states the contract explicitly in a doc comment: *"A malformed one is `ClassUnknown` and INERT under the shipped contract (`sysw/descriptor.go:46-48`): it reaches no screen, and its only device-side signal is the door's not-understood count (§6a)."* I did not independently re-verify `sysw/descriptor.go:46-48`'s exact current text (out of this recon's declared scope), but the citing comment is contemporaneous (Sep 6) with the rest of this file.

The device-visible signal for a malformed `hash:` is per §6a: it changes no admitted-record count (unlike `key:`, which decrements "Keys loaded: N"); it only adds to the door's aggregate "N payload records were not understood" (session inert count, §8r).

## 4. Back-compat surface

**No version/era field exists on individual records at all.** The only version byte in this container is the container-header's `VERSION: u8 = 0x01` (`crates/me-cli/src/sysw/wire.rs:32`, checked at wire.rs:150) — that governs the 52-byte header layout, not per-record grammar, and bumping it would require every reader to gate on it, breaking nothing about `hash:` specifically but not helping it either.

**No generic forward-compat rule for an unrecognized prefix token.** The design is: the SIX reserved prefixes (`text:`, `pass:`, `tx:` — record.rs:27-45 / record.go:14-22; `key:`, `hash:`, `now:`, `phrase:` — composer_records.rs:24-38 / composer_records.go:25-33) are matched by exact prefix string; anything not matching one of them falls through to the constellation sniffers (mnemonic, ms1, md1/mk1, mt1, descriptor) and, failing all of those, becomes `Class::Unrecognised`/`ClassUnknown`. There is no "any unknown `foo:` prefix is forward-compatible reserved space" rule — each prefix is individually reserved by being individually coded for. `SPEC_systemwide_payloads.md` §5.3.1 (lines ~571-660) states the reserved-prefix rule as: *"a record beginning `text:`, `pass:`, `key:`, `hash:` or `now:` that is not valid lowercase hex is `ClassUnknown` and refused, never silently treated as free text"* — i.e. the reservation is per-named-prefix, not a wildcard scheme.

**Consequence for a hash-KIND addition:** a payload already packed with a bare `hash:<64hex>` today decodes, under the current grammar, to exactly one thing — a 32-byte digest, implicitly sha256 (the only kind the composer or the device currently understands; §6c states the sha256 compilation explicitly, `SPEC_wallet_policy_composer.md` line ~383-386). Adding a kind field has two structurally different routes visible in the existing grammar, neither of which is free:
  - **Extend the `hash:` body itself.** The body is currently defined as *exactly* the raw digest hex, zero header bytes (unlike `now:`/`phrase:`, which are hex-of-UTF-8-TEXT with a comma-cut convention — see `now_record`/`phrase_record`, composer_records.rs:224/298-315). Any in-body kind marker (a prepended 2-hex-char kind byte, a comma-separated `<kind>,<hex>` text form mirroring `now:`/`phrase:`) necessarily changes what "exactly 64 lowercase hex characters" accepts — a 64-char bare digest and a 64-char body of the NEW shape are indistinguishable unless the new grammar reserves length 64 for the legacy form and requires a different length (or a distinguishing marker) for kind-qualified bodies. This is exactly the room design decision 2 ("a bare legacy `hash:` keeps meaning sha256") needs, and the current grammar has no marker byte or length band held in reserve for it — the "exactly 64" check (composer_records.rs:303) is the ENTIRE grammar today, so any widened form must special-case 64-char bodies as legacy-sha256 to stay compatible.
  - **New prefixes per kind** (`hash256:`, `ripemd160:`, `hash160:`) — cheap and trivially back-compatible (bare `hash:` is untouched), at the cost of going from one reserved prefix to four, and of not being what design decision 2 (*"the record carries the kind"*, singular record shape) asks for.
  - Digest **length alone cannot carry the kind**: sha256 and hash256 are both 32 bytes (64 hex chars); ripemd160 and hash160 are both 20 bytes (40 hex chars). A 64-char or 40-char body is ambiguous between two kinds without an explicit kind marker — this is the concrete reason design decision 4 ("digest length derived from kind") is a one-way derivation (kind → length) and not invertible (length ↛ kind).

**Precedent already in the sibling grammars**: `now:` and `phrase:` already carry a comma-cut, hex-of-UTF-8-text convention (composer_records.rs:224-232, 298-315) that a kind-qualified `hash:` could mirror (e.g. hex of `"<kind>,<hexdigest>"`) — noted as an observed existing pattern in this codebase, not a recommendation.

## 5. Who else writes `hash:` records

**`me sysw pack` is the only production producer**, and it takes the record **verbatim on argv/`--in`/stdin** — there is no `--hash <digest>` convenience flag. `crates/me-cli/src/main.rs:225-232` (`Pack { records: Vec<String>, ... }`) documents the format in the CLI help but the operator (or a script) types `hash:<64hex>` directly; nothing in `me`'s CLI surface calls `hash_record()`.

`hash_record()` itself (`composer_records.rs:200-202`) has **no production call site** — grep confirms its only callers are itself (doc example) and `crates/me-cli/tests/sysw_composer_records.rs`. It exists purely as a test/fixture builder today, the same shape noted in the source's own comment about `phrase_record`'s history (composer_records.rs:206-221, "eight test uses and nothing else" — that comment is about `phrase_record`, not `hash_record`, but the situation is identical for `hash_record`).

**Fixtures/vectors carrying `hash:`, counted directly (not estimated):**
- Primary cross-language vector file `crates/me-cli/testdata/record_class_vectors.json`: **68 total rows, 7 of which are `hash:` rows** (2 valid → `Hash`, 5 malformed → `Unknown`: 63-char, 66-char, uppercase, empty, and a 31-byte/62-char body) — counted with `python3 -c` over the JSON, not eyeballed.
- Vendored Go copy, pinned by `seedhammer/sysw/testdata/record_class_vectors.provenance.json`: same 68 rows, `sha256: 3575ccb0e1...` pin recorded 2026-09-06 against mnemonic-engrave commit `8cf2a7f98e5f5bfa535200b260a514cdae33f087`.
- `crates/me-cli/tests/sysw_composer_cli.rs`: 5 occurrences of `"hash:`.
- `crates/me-cli/tests/sysw_composer_records.rs`: 7 occurrences (the CASES table source for the vector file).
- `crates/me-cli/tests/sysw_pack_preimage.rs`: 3 occurrences.
- Go test files with `"hash:` literals: `composer_hashlock_test.go` (4), `composer_hash_test.go` (4), `composer_fixtures_test.go` (2, including the **shared fixture constant** `composerTestHashRecord = "hash:" + strings.Repeat("ab", 32)` at line 37, reused by other composer Go tests), `composer_provenance_test.go` (2), `composer_door_test.go` (1), `composer_hashlock_plates_test.go` (1), `composer_gates_test.go` (1), `sysw/composer_records_test.go` (1).

None of these fixtures encode anything but a bare 32-byte digest; a kind-carrying grammar change touches every one of the 68 vector rows (at minimum needs new rows) plus the shared Go fixture constant.

## 6. Spec ownership and the cross-language pin

**Owning section**: `design/SPEC_wallet_policy_composer.md` §6a (lines 259-259+, table at ~270-273, validation prose at ~274-292) is the NORMATIVE owner of the `hash:` class's body rule ("MUST decode to exactly 32 bytes") and its refusal line. §6c (lines 380-395) owns the on-device entry/consent UX (the 32-byte / sha256-compilation rule, the "Which hash?" picker). §8n (lines ~809-816) owns the exact host refusal line text.

`design/SPEC_systemwide_payloads.md` §5.3 / §5.3.1 (lines 548-660) is the CONTAINER-level owner: it states the general reserved-prefix + hex-encoding rule that all six prefixes share, explicitly defers `hash:`'s own per-class body rule to `SPEC_wallet_policy_composer.md` §6a ("their per-class body rules ... are normative in mnemonic-engrave `SPEC_wallet_policy_composer.md` §6a and §8n" — §5.3.1, ~line 620), and states the Rust-primary rule for these three composer classes.

**Cross-language pin mechanism**: `crates/me-cli/testdata/record_class_vectors.json` (Rust-primary, generated — never hand-edited — by the `regenerate` test in `crates/me-cli/tests/sysw_composer_records.rs`, from its `CASES` table, e.g. composer_records.rs:452-458) is vendored byte-for-byte into `seedhammer/sysw/testdata/record_class_vectors.json`, pinned by `seedhammer/sysw/testdata/record_class_vectors.provenance.json` (sha256 + `commit`/`file_commit` + row count). `seedhammer/sysw/vendored_vectors_test.go` fails if the vendored file's sha256/row-count disagree with the pin; `vendored_vectors_live_test.go` (tag-gated) additionally fetches the live upstream file and compares. `seedhammer/sysw/composer_records_test.go` runs every vector row through `classifyComposer`/`Classify` and asserts agreement. This is the mechanism that keeps Rust and Go from silently disagreeing on `hash:` classification, and it is exactly what a hash-kind change must regenerate + re-vendor + re-pin.

## Design-decision friction — lead items

**Already filed, matches this recon's own finding independently derived from md/compose.go and md.go**: `design/FOLLOWUPS.md` **F-507** (`composer-hashlocks-are-sha256-only-while-md1-carries-four-hash-fragments`, filed 2026-09-12, line 16510) states precisely the gap this recon also found by reading the code: the `md` WIRE FORMAT already carries all four hash kinds —
```
md-codec tag.rs   0x1D sha256 (32B) | 0x1E hash160 (20B) | 0x1F hash256 (32B) | 0x20 ripemd160 (20B)
```
(confirmed directly: `seedhammer/md/md.go:69-73`, `tagSha256 = 0x1D`, `tagHash160 = 0x1E`, `tagHash256 = 0x1F`, `tagRipemd160 = 0x20`; body types `hash256Body [32]byte` / `hash160Body [20]byte` at md.go:120-121) — but the COMPOSER layer one level up is hardcoded to one kind end-to-end:
- `type SpendPath struct { ... Hash *[32]byte ... }` (`seedhammer/md/compose.go:165-168`, doc comment says "optional **sha256** preimage" verbatim) — this is the type design decision 4 ("One `HashLock { kind, digest }` type") would replace.
- `pathBody` (`md/compose.go:402-404`) unconditionally lowers to `node{tag: tagSha256, ...}` — no kind is read because none is carried.
- The `hash:` record itself (§1 above) carries only a digest, no kind — this is the wire surface design decision 2 needs to widen.
- The device's `composerHexEntry` fallback pad (`gui/composer_hash.go:79-146`) accepts **exactly 64 hex characters** (`len(kbd.Fragment) > 64` truncation at line 87, `valid := len(frag) == 64` at line 90) — there is no path today for a 20-byte (40-hex) manual entry at all, which design decision 3 ("20-byte kinds warn only when the device did not derive the preimage") requires to exist as an entry mode before it can warn about anything.
- `RECON.md` also separately confirms (`design/agent-reports/spec-hashkinds-core-verdicts.md`) that Bitcoin Core 25.0.0 itself enforces digest length strictly per fragment kind (32B for sha256/hash256, 20B for ripemd160/hash160) with one generic, non-diagnostic error string for every malformed case — independent confirmation of design decision 4's "digest length derived from kind" from the consensus-adjacent side, not the wire/composer side this report covers.

**Genuinely new friction found in this recon** (not already in F-507): the length-ambiguity point in §4 above — sha256/hash256 tie at 32 bytes and ripemd160/hash160 tie at 20 bytes, so **length cannot stand in for a kind field**; the record MUST carry an explicit kind token once more than one kind per length is authorable, and the current `hash:` grammar's "exactly 64 lowercase hex" rule leaves no reserved space (no marker byte, no alternate length band) for that token without redefining what a 64-character body means — which is precisely why back-compat (design decision 2's "a bare legacy `hash:` keeps meaning sha256") is a real constraint on the wire grammar and not just a UX promise.

**Naming note**: no existing type or constant named `HashLock` exists in `crates/me-cli`, `seedhammer/gui`, `seedhammer/sysw`, or `seedhammer/md` (grepped, zero hits) — `HashLock { kind, digest }` (design decision 4) has no name collision to resolve. Separately, `mnemonic-secret/crates/ms-codec` already uses the word "kind" for an unrelated concept (`PayloadKind`, the ms1 payload-type byte distinguishing a seed from a `0x03` preimage plate — see `ms-codec/tests/hashlock_kind.rs`); a new "hash kind" field should pick vocabulary that doesn't collide with that existing "kind" in prose shared between the two repos.

## Things I could not fully verify within this recon's scope

- I did not re-read `seedhammer/sysw/descriptor.go:46-48` directly to confirm its exact current text; I report it via a contemporaneous doc-comment citation in `gui/composer_hash.go:53-56` rather than a first-hand read, since the brief's file list did not name `descriptor.go` and it appeared only as a citation target.
- The spec's own citations at `SPEC_wallet_policy_composer.md` §6a text point to `crates/me-cli/src/sysw/mod.rs:288` (`pack_with`) and `:416` (`admit_check`) — these line numbers have drifted; the functions are at `mod.rs:403` (`pack_with`) and `mod.rs:531` (`admit_check`) as of this recon. Flagging as citation drift, not a grammar-correctness issue.
