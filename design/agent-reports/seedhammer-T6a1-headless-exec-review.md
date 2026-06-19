# T6a-1 HEADLESS wire-format core — whole-diff adversarial EXECUTION REVIEW

**Reviewer:** opus architect (independent post-implementation execution review)
**Date:** 2026-06-19
**Scope:** cumulative diff `e4013a8..feat/t6a1-headless` (6 commits) in worktree
`/scratch/code/shibboleth/seedhammer-wt-t6a1`.
**Mandate:** catch IMPLEMENTATION-INTRODUCED defects; independently re-verify
byte-fidelity vs the authoritative Rust source (`md-codec` v0.36.0 @ `c85cd49`),
NOT the implementer's self-consistency. Reviewer-only: no tracked source modified.

---

## VERDICT: GREEN (0 Critical / 0 Important)

1 Minor (observability/error-path divergence, unreachable through the public API
on the T6a-1 surface). No blocking findings. The cycle is byte-exact vs Rust on
every checked vector, the goldens are authentic and non-circular, the new package
layering is clean, and the diff is headless-only with zero regression.

---

## Required explicit statements

**(a) `WalletPolicyId` byte-fidelity — INDEPENDENTLY re-derived, byte-exact vs Rust.**
I re-derived the cell-7 golden from the authoritative Rust preimage MYSELF (not
via the Go code): taking the byte-exact preimage that `md-codec-0.36.0`
`identity.rs:540-544` documents
(`00035d1dea420b080060deadbeef ‖ [11;32] ‖ 02 ‖ [22;32]`, 79 bytes) and computing
`SHA-256(preimage)[:16]` in a throwaway Python script yielded
`6650b980 3b3c6621 0140540d a8d765a0` — identical to the Rust golden
(`identity.rs:547-550`). The Go `TestWalletPolicyIdGolden` (walletpolicyid_test.go:81)
pins that same value and PASSES. Because SHA-256 is collision-resistant, a passing
Go test on a 79-byte preimage proves the Go preimage is byte-identical to Rust's —
the implementer's corrections are confirmed correct against source:

- **writeNode-only leading segment** (NOT `encode_payload`): Go `WalletPolicyId`
  (walletpolicyid.go:41-45) writes ONLY `writeNode(tree)` byte-padded, matching
  Rust `identity.rs:179-182` (`tree::write_node` → `into_bytes`). The cell-7 tree
  segment is `0x00` (5-bit Wpkh tag, kiw=0, zero-padded to one byte) — Rust
  `identity.rs:526` asserts `tree_bytes == [0x00]`. Confirmed.
- **6-bit node tag** (the "stale 5-bit docstring" non-issue): authoritative
  `tag.rs:142` writes `w.write_bits(u64::from(primary), 6)`; the module doc
  `tag.rs:3` says "primary 6-bit space". Go `writeTag` (encode.go:45-47) writes 6
  bits. At n=1 the cell-7 tree byte is `0x00` either way, and the byte output is
  correct per the actual `tag.rs`/`write_node`. 6 is right. Confirmed.
- **per-@N `record_bytes` = SEPARATE byte-padded bitstream**
  `varint(path_bit_len)‖path_bits‖varint(us_bit_len)‖us_bits`: Go (walletpolicyid.go:69-82)
  matches Rust `identity.rs:206-211` line-for-line. The cell-7 record is the
  8-byte `5d 1d ea 42 0b 08 00 60` (Rust `identity.rs:516-519`); my independent
  SHA-256 over the preimage containing exactly these bytes reproduced the golden.
  Confirmed (60 record bits → 8 bytes; fully-present record = 1+8+4+65 = 78 bytes).
- **presence byte `&0x03`**: Go `(b2u(fp)|(b2u(xpub)<<1))&0x03` (walletpolicyid.go:86)
  matches Rust `identity.rs:219`. Confirmed.
- **fp/xpub OMITTED (not zero-filled) when absent**: Go appends `fp[:]`/`xpub[:]`
  only inside `if fpPresent`/`if xpubPresent` (walletpolicyid.go:90-95), matching
  Rust `identity.rs:223-228` (`if let Some(..)`). The
  presence-significance test (`TestWalletPolicyIdPresenceSignificant`,
  walletpolicyid_test.go:122) PASSES — nulling pubkeys+fp yields a DIFFERENT id.
- **origin resolution = override-TLV > path_decl AS-IS, NO canonicalOrigin
  fallback at hash time**: I read `canonicalize.rs:420-474` (`expand_per_at_n`)
  in full. Rust resolves `origin_path` as `OriginPathOverrides[idx]` else
  `path_decl.paths` cloned AS-IS (lines 437-444) and pushes it verbatim
  (line 467). `canonical_origin(&d.tree)` is consulted ONLY at line 452 to decide
  whether to RAISE `MissingExplicitOrigin` — it is NEVER substituted into the
  hashed path. Go `resolveOriginRaw` (walletpolicyid.go:141-159) mirrors this
  exactly (override → divergent[idx] → *shared → empty), with NO canonicalOrigin
  substitution. `resolveUseSiteRaw` (164-173) matches `canonicalize.rs:458-460`.
  The R0-I2 correction is verified correct. The encoding-stability test
  (`TestWalletPolicyIdEncodingStable`, walletpolicyid_test.go:147) PASSES,
  exercising the override>baseline branch (mirrors Rust
  `walletpolicyid_stable_across_origin_elision`, identity.rs:572-588).
- **SHA-256→[:16]**: Go `sha256First16(preimage)` matches Rust `identity.rs:236-238`.
- **fresh raw resolver, NOT the display path**: grep confirms neither
  walletpolicyid.go nor encode_singlesig.go CALLS `ExpandWalletPolicy`/
  `ExpandedKey`/`canonicalOrigin` (only doc-comment mentions). The display
  accessor `ExpandWalletPolicy` (expand.go:83) and `canonicalOrigin` (md.go:1095)
  are never invoked from the hash path. Confirmed.

The bit machinery underneath was cross-checked against Rust `bitstream.rs`:
`bitWriter.write` (bits.go:102) == `write_bits` (bitstream.rs:29-69);
`bitLen` (bits.go:136) == `bit_len` (bitstream.rs:72-78);
`intoBytes` (bits.go:147) == `into_bytes` (bitstream.rs:81-83);
`reEmitBits` (bits.go:156) == `re_emit_bits` (bitstream.rs:220-230);
`writeVarint` (encode.go:51) == `write_varint` (varint.rs:15-42);
`writeOriginPath` (encode.go:89) == `OriginPath::write` (origin_path.rs:54-66);
`writeUseSitePath` (encode.go:133) == `UseSitePath::write` (use_site_path.rs:80-95).
All algorithmically identical.

The 4 vendored goldens additionally pin the FULL 16-byte id + `[0:4]` stub
(`TestWalletPolicyIdToolkitDifferential`, walletpolicyid_test.go:94) — all PASS,
with stub == WPID[:8] asserted.

**(b) Goldens are authentic / non-circular.** Verified via an independent fork
(throwaway test, since deleted; tree left clean) that decoded each vendored
`.md1.txt` DIRECTLY through the shipped `md.ExpandWalletPolicyChunks` /
`md.WalletPolicyIDStubChunks` (the decode path — never `EncodeSingleSig`) and
compared to `.meta.json`. ALL 4 sets decode to exactly their claimed wallet
policy:

| set | root | chaincode/pubkey | fp | origin | decode-derived stub == meta == WPID[:8] |
|---|---|---|---|---|---|
| pkh | ScriptPkh | ✓ | 73c5da0a | m/44h/0h/0h | fc90e097 ✓ |
| sh_wpkh | ScriptSh | ✓ | 73c5da0a | m/49h/0h/0h | 8807be80 ✓ |
| wpkh | ScriptWpkh | ✓ | 73c5da0a | m/84h/0h/0h | 1c0170fe ✓ |
| tr | ScriptTr | ✓ | 73c5da0a | m/86h/0h/0h | 6ae6d59e ✓ |

The mk1 panel value `1c0170fe`/`1c017` is the `singlesig_wpkh` stub itself
(repo-wide grep finds it only in singlesig_wpkh.meta.json), and the decode-derived
stub reproduces it independently of any re-encode. Because the goldens decode
through the independently-reviewed decoder to the claimed policy, the
`EncodeSingleSig` string-equality gate (`TestEncodeSingleSigStringEquality`) is
NOT circular. Provenance documented (README_singlesig.md): md-codec 0.36.0 ==
git c85cd49, toolkit v0.58.1 @ 4e21d94, abandon seed, fp 73c5da0a.

**(c) `EncodeSingleSig` 4 shapes + `EncodeMS1` byte-exact.**
- `EncodeSingleSig` (encode_singlesig.go:36) builds n=1 wallet-policy descriptor:
  pubkeys+fingerprints TLV, explicit `pathDecl.Shared` origin, use-site `<0;1>/*`.
  Trees per `singleSigTree` (encode_singlesig.go:92): pkh/wpkh `keyArgBody{0}`,
  tr `trBody{isNums:false, tree:nil}` (NOT keyArgBody — asserted by
  `TestEncodeSingleSigTrBody`), sh-wpkh `sh{children{wpkh}}`. Routes through
  `split` (CHUNKED, 3 strings each; >320-bit payload). `TestEncodeSingleSigPayloadParity`
  (form-independent byte parity), `TestEncodeSingleSigStringEquality` (exact wire
  + ValidMD), and `TestEncodeSingleSigRoundTrip` (DecodeChunks/ExpandWalletPolicyChunks)
  all PASS. Empty origin rejected (`TestEncodeSingleSigEmptyOriginRejected`).
  `ScriptShWpkh` APPENDED after `ScriptTr` (md.go:1176; iota order preserved:
  Wpkh=0,Pkh=1,Sh=2,Wsh=3,Tr=4,ShWpkh=5) — no renumber, `rootScriptKind`/#10b
  consumers unaffected (confirmed via whole-repo test pass).
- `EncodeMS1` (msencode.go:17) = `NewSeed("ms",0,"entr",'s',[0x00‖entropy])`.
  Round-trips `DecodeMS1` for 16/20/24/28/32-byte entropy
  (`TestEncodeMS1RoundTrip`); `ms10entrs` prefix asserted (`TestEncodeMS1Prefix`);
  the pinned zero-16 vector `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`
  reproduced (`TestEncodeMS1ZeroVector`); bad lengths rejected. The `[0x00][entropy]`
  payload matches `DecodeMS1`'s `msPrefixEntr` branch (mspayload.go:9,40-41).

**(d) New `seedhammer.com/bundle` package layering is clean (no cycle, no GUI leak).**
`go list -deps seedhammer.com/bundle` = {codex32, bip32, md, mk, bundle} — composes
the three headless pieces, pulls NO gui/program/flow. `go list -deps
seedhammer.com/md` = {bip32, codex32, md} — md does NOT import mk or bundle
(verified: grep for `seedhammer.com/mk|bundle|gui` in md/*.go,codex32/*.go is
empty). No import cycle (`go list -deps` succeeded). The comparator (verify.go)
checks stub-binding FIRST on both bundles, then fp/xpub/path (via mk.Decode),
md1 exact-string, ms1 on RECOVERED ENTROPY (not string); returns the first
diverging field; scrubs copied entropy buffers. Tests cover match, mutated
xpub/path, mutated descriptor, reordered md1, mutated entropy, entropy-not-string,
and stub mismatch — all PASS.

**(e) Headless-only, no regression.** The diff touches ONLY `md/`, `codex32/`,
`bundle/` + testdata. The ONLY modified existing file is `md/md.go` (the 6-line
`ScriptShWpkh` append); everything else is net-new files. NO GUI/program/flow,
NO shipped behavior changed. Whole-repo `go test -count=1 ./...` = 38 packages
`ok`, 0 FAIL (existing md/mk/codex32/ms1 decode + #10a/#10b + T5 + gui all pass
verbatim).

---

## Observed test / vet / fuzz output

```
$ go test -count=1 ./...      → 38 packages ok, 0 FAIL (gui/mk/md/codex32/bundle/slip39/... all ok)
$ go vet ./md/... ./codex32/... ./bundle/...   → RC=0 (clean)
$ gofmt -l .                  → (empty — clean)
```

Pre-existing baseline warning (NOT introduced by this diff):
`gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file
is go1.25)` — `gui/op/` is untouched by the diff (`git diff --stat e4013a8..HEAD
-- gui/op/` is empty), so this is identical at baseline `e4013a8`. `go vet` still
returns RC=0 (it is a note, not a failure).

Fuzzers (each ≥1M execs, 0 panics):
```
FuzzEncodeMS1        1,000,000 execs  PASS  (codex32)
FuzzEncodeSingleSig  1,000,000 execs  PASS  (md)
FuzzWalletPolicyId   1,000,000 execs  PASS  (md)
FuzzVerify           1,000,000 execs  PASS  (bundle comparator)
```
(Note: the first FuzzEncodeSingleSig run aborted with a transient build error —
`open md/zz_authenticity_throwaway_test.go: no such file` — caused by my parallel
authenticity fork deleting its throwaway file mid-compile. This is a reviewer
test-harness race, NOT a feature defect. The clean re-run after the fork finished
passed 1M execs. Tree confirmed clean afterward.)

---

## Findings

### Minor

**M1 — `WalletPolicyId` does not replicate Rust's `MissingExplicitOrigin` error
gate (unreachable through the public API on the T6a-1 surface).**
*File:* `md/walletpolicyid.go:141-159` (`resolveOriginRaw`), vs Rust
`canonicalize.rs:450-454`.
*Defect:* Rust `expand_per_at_n` returns `Err(MissingExplicitOrigin{idx})` when,
for some `@N`, the resolved origin path is empty AND no `OriginPathOverrides[idx]`
is present AND `canonical_origin(&d.tree)` is `None` (e.g. `sh(wpkh)`,
`sh(sortedmulti)`, `tr(.., TapTree)` — see `canonical_origin.rs:12-19`). Go's
`resolveOriginRaw` returns a depth-0 `originPath{}` silently, so `WalletPolicyId`
would hash an empty-origin record and return a (Rust-inconsistent) id rather than
an error.
*Evidence/repro:* a hand-built `*descriptor{tree: sh(wpkh@0), pathDecl.shared:
empty, no origin override}` passed directly to `WalletPolicyId` hashes
successfully in Go but errors in Rust. NOT reproducible via any public API:
`WalletPolicyId` takes an *unexported* `*descriptor`; the only public entry points
(`WalletPolicyIdChunks`/`WalletPolicyIDStubChunks` and the comparator) reach it
exclusively through a *decoded* descriptor, and the decoder's
`validateExplicitOriginRequired` (md.go:1033, invoked from `decodePayloadValidated`
at md.go:1151) already rejects exactly this case at decode time — the identical
`canonical_origin`-gated rule (port of `validate.rs:182-207`). `FuzzWalletPolicyId`
also decodes via `decodePayloadValidated`, so it is gated too. For every shape
`EncodeSingleSig` produces (always with a non-empty explicit origin, rejected
otherwise at encode_singlesig.go:37), the gate is never triggered and Go/Rust
outputs are byte-identical.
*Impact:* none on correctness for T6a-1 (the byte-exact surface is fully
verified); purely an error-path/observability divergence for a non-public,
decoder-already-rejected input class.
*Exact fix (optional hardening, defer-acceptable):* have `WalletPolicyId` return
`errMissingExplicitOrigin` when `resolveOriginRaw` yields an empty path AND no
override AND `canonicalOrigin(dc.tree)` is `None`, so an in-package caller that
ever bypasses the decoder gets the same typed error as Rust. Low priority — does
not affect any shipped or reachable path.

---

## Bottom line

The byte-lock-riskiest cycle in T6 is byte-exact vs the authoritative Rust
`md-codec` 0.36.0 source. I independently re-derived the `WalletPolicyId` golden
from the Rust preimage (not the Go code) and confirmed the Go output is
byte-identical, including every one of the implementer's corrections
(writeNode-only leading segment, 6-bit tag, no canonicalOrigin substitution at
hash time, presence `&0x03`, omit-not-zero fp/xpub). The goldens are authentic
and non-circular. `EncodeSingleSig` (4 shapes) and `EncodeMS1` are byte-exact.
The new `bundle` package layering is clean (no cycle, no GUI leak; md does not
import mk). The diff is headless-only with zero regression across 38 packages.
The single Minor finding is an unreachable error-path divergence, not a defect.

**VERDICT: GREEN (0 Critical / 0 Important / 1 Minor).**
