# FIX: the canonical descriptor keeps an extended key's own depth and child number (sh2-me-canonical-depth)

- Source: security sweep unit **canonical-keeps-xpub-depth**, graded **Important**
  (coordinator `reports/security-sweep/grade/routing.md` row `canonical-keeps-xpub-depth`;
  `agent-reports/grade-security-sweep-engrave-1.md` r32 and `grade-security-sweep-engrave-2.md`
  row `sh2-me-canonical-depth`). Already tracked here: FOLLOWUPS `sh2-me-canonical-depth`
  (`design/FOLLOWUPS.md:21205`, status open, owning phase "next me FIX").
- Routing's "what changes" (binding): carry the parsed xpub's own depth and child number in `Key`
  and use them in `canonical_string`, as the fork's `xpubMeta` does; re-pin seam row
  `accepted/sh-wsh-sortedmulti-16-keys` and the promotion announcement text.
- Base: master `0bd8dfba`. Fork measured at main `65717c5`. Author: Claude Opus 5.5
  (claude-opus-5-5), 2026-11-23.
- Track: small fix. No new flag, command, refusal or exit code. One changed holder-visible
  sentence (§4, **W-1**, PROPOSED). Two new fields on `descriptor::cascade::Key` (a crate-internal
  library type; nothing outside this workspace builds one). One test-data re-pin that must land in
  **both** this repo and the seedhammer fork (§6.3).
- Build gate: `scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh` (committed with this document).
  Its NOT-covered line is in its header.

## 1. The defect (measured on master 0bd8dfba)

`Key::canonical_string` (`crates/me-cli/src/descriptor/cascade.rs:262-277`) writes the depth byte as
`origin.len()` and the child number as `origin.last()`. The fork's `Key.ExtendedKey()`
(`bip380/bip380.go:102-118` at `65717c5`) uses the depth and child number of the xpub it parsed
(`xpubMeta`, set at `bip380.go:438` by `ParseKey`, which both the BIP-380 route and
`nonstandard/parse.go:150`'s Coldcard route call). For a key whose own depth or child number differs
from its origin path (explicit, implied by the script, or the SLIP-132 fallback), `me` and the device
produce different canonical strings for the same input.

Measured, the input `[4bbaa801/44h/0h/0h]xpub6DiY…hpan` (the `dc567276` fixture cosigner: depth 4,
child 2h), promoted:

| side | canonical |
| --- | --- |
| `me sysw pack --as descriptor` (packed record and `descriptor:` line) | `pkh([4bbaa801/44h/0h/0h]xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCjDeYDRVzcr561WmrjVL5mSty4se5Jsgx3dGsJPkitFALrDJ7P8M44f5)#fufq4cvp` |
| fork `nonstandard.OutputDescriptor` then `Encode()` | `pkh([4bbaa801/44h/0h/0h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan)#57c9a380` |

Consequence (engrave-2 asked for it to be measured): it does **not** fail closed. The device parses
`me`'s string and re-encodes it to itself (fixed point), and the key data, chain code, addresses and
md1 wallet-id are unchanged (`wallet-id: f371341c52c0a9929a281e6e57252427`, `address 0:
1M88vKcJFc4KPAe5RHXsuJqWcg3muStyK4` either way). What goes wrong is the string: the plate carries a
base58 xpub the holder has never seen, and a holder comparing it with their wallet software's xpub
sees a mismatch on a correct result. The same divergence was measured for three more shapes (§5 rows
2 to 4), including `pkh(<depth-4 xpub>)` with **no** origin block, where `me` rebuilds from the
script's implied path and prints no origin to explain the change. The grade stays with the grader
(I as filed; engrave-2's "only fails closed, regrade to M" condition is not met as stated, because
nothing is refused).

## 2. Fix shape

1. `Key` (`cascade.rs:229-248`) gains two fields, after `parent_fingerprint`:
   `depth: u8` and `child_number: u32`, documented as "the depth byte and child number of the
   extended key as parsed (BIP-32 serialisation bytes 4 and 9..13); the fork's `xpubDepth` and
   `xpubChildNumber`".
2. `ExtendedKey` (`cascade.rs:1072-1077`) gains the same two fields; `parse_extended_key` fills them
   from `raw[4]` and `u32::from_be_bytes(raw[9..13])`.
3. Both `Key` constructors copy them from the parsed `ExtendedKey`: `parse_key` (`cascade.rs:1187`)
   and the Coldcard branch (`cascade.rs:681`). The `admit.rs` test helper `key()` (`admit.rs:478`)
   sets `depth: 0, child_number: 0` (its tests never encode).
4. `canonical_string` writes `self.depth` and `self.child_number` and no longer reads `origin`. Its
   doc comment says the version is normalised and depth and child number are the key's own, as
   `Key.ExtendedKey()` does with `xpubMeta`.
5. `promotion_announcement`'s last line changes to W-1 (§4); its doc comment (`gate.rs:332-339`)
   drops the "rebuilds an xpub's depth and child-number bytes" sentence and says the two forms differ
   only by version bytes (`zpub` → `xpub`), `'` → `h` and the checksum.
6. Not changed: `Key::identity` (already key data and chain code only); the md1 route (md1 carries
   key material, not the xpub envelope); every refusal text. `conjunct_4_versions`'s
   `canonical_string()` argument is unreachable from a cascade-built descriptor (the cascade refuses
   those versions first) and now keeps the key's own depth, like `parse_extended_key`'s existing
   `converted` remedy.

No warning is added when a key's depth or child number contradicts its origin: the device admits
such a key silently, the key material and addresses are right, and the routing row does not ask for
one.

## 3. Spec delta (`design/SPEC_descriptor_input.md`)

§4.2 defect 3 (`:402-412`) is left as written: it describes the device at the pre-sh2 baseline, and
§4.2 already records that the sh2 bug hunt fixed all three defects (`:391-393`).

**§4.5, `:678-688`.** Replace the paragraph that begins "**The announcement echoes the operator's key
AS SUPPLIED, alongside the inferred descriptor — R0's I5.**" with:

> **The announcement echoes the operator's key AS SUPPLIED, alongside the inferred descriptor — R0's
> I5.** The canonical re-encoding keeps the key's own depth and child-number bytes, as the device
> does since the sh2 bug hunt (`Key.ExtendedKey()` uses the parsed key's `xpubDepth` and
> `xpubChildNumber`, `bip380/bip380.go:102-118` at fork `65717c5`), but it rewrites a `zpub`'s
> version bytes as `xpub` and `'` as `h`, so the inferred descriptor can still differ from what the
> operator typed. *(AMENDED 2026-11-23, FIX_sec_canonical_keeps_xpub_depth: until then `me` rebuilt
> depth and child number from the origin path, so a depth-4 cosigner xpub promoted to `pkh(…)` was
> packed as a different `xpub6Bq…`.)* The one check the announcement exists for is "is that my
> key?", so the announcement prints both — `key as supplied: <verbatim>` and `inferred wallet:
> <canonical>` — with one line stating that only the version bytes may be normalised and that the
> depth, child number and key material are unchanged.

**§5.2, `:1074-1078`.** Replace "measured four ways (R0's C5): SLIP-132 versions normalise (`zpub` →
`xpub`), `'` → `h` (`bip32.Path.Encode`), the checksum is recomputed, and an xpub's
depth/child-number bytes are rebuilt from the origin path — so the base58 string itself can change."
with:

> measured three ways (R0's C5): SLIP-132 versions normalise (`zpub` → `xpub`), `'` → `h`
> (`bip32.Path.Encode`), and the checksum is recomputed — so the base58 string itself can change, in
> its version bytes only. An xpub's own depth and child-number bytes are kept, as the device keeps
> them. *(AMENDED 2026-11-23, FIX_sec_canonical_keeps_xpub_depth: R0's C5 measured a fourth
> transformation, depth and child number rebuilt from the origin path; the device dropped it in the
> sh2 bug hunt and `me` drops it here.)*

No other spec sentence states the rebuild (`git grep -n "rebuil" design/SPEC_descriptor_input.md`
at `0bd8dfba` finds only `:406`, `:679` and `:1078`).

## 4. Printed texts and exit codes

Exit codes: unchanged everywhere.

The packed `Descriptor` record, the `descriptor:` line of the identification block (`me sysw pack`
and `me sysw show`) and the announcement's `inferred wallet:` line all carry the new canonical: for
the §1 input, `pkh([4bbaa801/44h/0h/0h]xpub6DiY…hpan)#57c9a380` (full string in §1). For every key
whose own depth and child number already match its origin, nothing printed changes (§6.3 measures
this over all 19 admitted seam rows).

**W-1 (changed, PROPOSED; WORDING_REVIEW row proposal).** The announcement's last line
(`gate.rs:349-350`), printed after `inferred wallet: …` with the same six-space indent:

- Was: `The key's serialisation was normalised (version and depth bytes rebuilt from the origin path); the key material itself is unchanged.`
- Now: `The key's version bytes may be normalised (`zpub` becomes `xpub`); its depth, child number and key material are unchanged.`

WORDING_REVIEW row proposal (this repo has no `design/WORDING_REVIEW.md`; the coordinator files it):

> The key's version bytes may be normalised (`zpub` becomes `xpub`); its depth, child number and key material are unchanged.

- Prints: `me sysw pack --as descriptor` and `--as md1`, stderr, last line of the promotion
  announcement (a bare key promoted to a wallet).
- Source: FIX_sec_canonical_keeps_xpub_depth §4 W-1. "may be": an `xpub` input is not rewritten; only
  `zpub` reaches promotion with other version bytes (`tpub` promotion is refused, §4.5, and `Ypub`/`Zpub`
  are never promoted, `PROMOTABLE` at `cascade.rs:1017`).
- Status: PROPOSED.

CHANGELOG (`crates/me-cli/CHANGELOG.md`, `[Unreleased]` → `### Fixed`), exact entry:

> - **The packed descriptor keeps an extended key's own depth and child number** (security sweep
>   canonical-keeps-xpub-depth, Important; FOLLOWUPS sh2-me-canonical-depth). `me` rebuilt them from
>   the origin path, so a key whose depth or child number differed from its path (a depth-4 key
>   promoted to `pkh(…)`, or `pkh(<depth-4 xpub>)` with no origin) was packed as an xpub string the
>   holder had never seen, and the device encoded the same input differently. The key material and
>   addresses were never affected. The promotion announcement's last line now reads "The key's version
>   bytes may be normalised (`zpub` becomes `xpub`); its depth, child number and key material are
>   unchanged." Exit codes are unchanged.

Toolkit manual: not affected (no `mr` or `md-cli` text or flag changes; `me` is not in
`mnemonic-toolkit` `docs/manual/src/40-cli-reference/`).

## 5. RED rows (run on master 0bd8dfba; temporary tests, reverted; mainnet xpubs are public fixture keys, no secret involved)

Fixture keys: `D4` = `xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan`
(depth 4, child 2h); `XPUB` = `cascade_tests.rs`'s `XPUB` (depth 3, child 0h). Expected outputs are the
fork's `Encode()` of the same input at `65717c5`, measured with
`scripts/descriptor-seam-vectors/goprobe` (`replace` pointed at a `65717c5` worktree): every one
`device_admits=true`, `fixed_point=true`.

Unit rows go in `crates/me-cli/src/descriptor/cascade_tests.rs` (new section
`// ── FIX_sec_canonical_keeps_xpub_depth ──`, a `const D4` beside `XPUB`); each asserts
`ok(<input>).encode() == <expected>`.

| # | test | input | expected after the fix | measured on master (FAIL, `left`) | mutant it catches |
| - | --- | --- | --- | --- | --- |
| 1 | `canonical_keeps_a_promoted_keys_own_depth_and_child` | `[4bbaa801/44h/0h/0h]{D4}` | `pkh([4bbaa801/44h/0h/0h]{D4})#57c9a380` | `pkh([4bbaa801/44h/0h/0h]xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCjDeYDRVzcr561WmrjVL5mSty4se5Jsgx3dGsJPkitFALrDJ7P8M44f5)#fufq4cvp` | today's code; also "own child, depth from origin" |
| 2 | `canonical_keeps_a_coldcard_keys_own_depth_and_child` | `Name: x\nPolicy: 1 of 1\nDerivation: m/48h/0h/0h\nFormat: P2WSH\ndc567276: {D4}\n` | `wsh(sortedmulti(1,[dc567276/48h/0h/0h]{D4}))#8nt5gkve` | `wsh(sortedmulti(1,[dc567276/48h/0h/0h]xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCjDeYDRVzcr561WmrjVL5mSty4se5Jsgx3dGsJPkitFALrDJ7P8M44f5))#lcwxj396` | fixing `parse_key` but not the Coldcard constructor (`cascade.rs:681`) |
| 3 | `canonical_keeps_the_keys_own_child_number_when_the_origin_ends_elsewhere` | `pkh([4bbaa801/44h/0h/5h]{XPUB})` | `pkh([4bbaa801/44h/0h/5h]{XPUB})#vxseckva` | `pkh([4bbaa801/44h/0h/5h]xpub6C9j4wAxxkWNHMxzvsATpRpv6c4vQFXrtxv8avt2SnKiac3AChU3QKuhcknG6cNBCcsKDhneperABQWkwvuHTtJuBzdPgG8U9exEDK57NMY)#rp099zkd` | own depth kept, child still `origin.last()` (depths agree here, so only the child can differ) |
| 4 | `canonical_keeps_the_keys_own_depth_with_no_origin_block` | `pkh({D4})` | `pkh({D4})#0dksumg9` | `pkh(xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCjDeYDRVzcr561WmrjVL5mSty4se5Jsgx3dGsJPkitFALrDJ7P8M44f5)#d58fnqc4` | keeping the key's own bytes only when `origin_explicit` (rebuilding from an implied path) |

Row 5, CLI, in `crates/me-cli/tests/descriptor_as.rs`:
`promotion_of_a_depth_4_key_packs_and_announces_the_key_as_supplied`. Writes
`[4bbaa801/44h/0h/0h]{D4}\n` to `document.txt`, runs
`me sysw pack --no-passphrase --as descriptor --in document.txt --out container.bin`, and asserts:
exit 0; stderr contains the whole lines `      inferred wallet: pkh([4bbaa801/44h/0h/0h]{D4})#57c9a380`
and `      The key's version bytes may be normalised (`zpub` becomes `xpub`); its depth, child number and key material are unchanged.`;
`sysw::open` of the container gives exactly one public record, equal to
`pkh([4bbaa801/44h/0h/0h]{D4})#57c9a380`. Measured on master: FAIL at the `inferred wallet` assertion;
stderr carried

    inferred wallet: pkh([4bbaa801/44h/0h/0h]xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCjDeYDRVzcr561WmrjVL5mSty4se5Jsgx3dGsJPkitFALrDJ7P8M44f5)#fufq4cvp
    The key's serialisation was normalised (version and depth bytes rebuilt from the origin path); the key material itself is unchanged.

Mutant it catches: fixing the encoder but leaving the old announcement line, or announcing one
string and packing another.

Row 6, the seam row (§6.3): `the_encoder_produces_every_canonical_the_file_carries`
(`tests/descriptor_seam.rs:1092`), unchanged code. With the regenerated file and pin in place and the
encoder unchanged, measured on master: `FAIL` at `descriptor_seam.rs:1109`,
`accepted/sh-wsh-sortedmulti-16-keys: canonical`, `left: "sh(wsh(sortedmulti(2,[00000001/48h/0h/0h/2h]xpub6EEHPqvm…`
`right: "sh(wsh(sortedmulti(2,[00000001/48h/0h/0h/2h]xpub6G7RFdSe…`; 18 of 19 `descriptor_seam` tests
passed. Mutant it catches: any encoder that still rebuilds depth for a depth-5 key under a 4-step
origin.

## 6. Tests

### 6.1 New

Rows 1 to 5 above.

### 6.2 Changed

- `promotion_announces_both_the_supplied_key_and_the_inferred_wallet` (`cascade_tests.rs:385-416`).
  It PASSES on master because it asserts the rebuild (`!canonical.contains(COSIGNER)`, "a depth-4 key
  promoted to a depth-3 origin must RE-SERIALISE"). Rewrite in the RED commit: the `deep` leg asserts
  `canonical == format!("pkh([4bbaa801/44h/0h/0h]{COSIGNER})#57c9a380")` and that the announcement
  contains both `[4bbaa801/44h/0h/0h]{COSIGNER}` (as supplied) and that canonical; the doc comment
  says the two forms now differ by `'` → `h`, `zpub` → `xpub` and the checksum. Add a third leg that
  keeps the test's purpose: `[4bbaa801/84'/0'/0']<zpub>` where `<zpub>` is the `zpub` spelling of
  `XPUB` (the seam file's `promotion/07-origin-84h-zpub` input), asserting the announcement contains
  the input verbatim and `ok(input).encode()`, and that they differ.
- No other test pins a rebuilt string: `git grep -n "xpub6BqQ\|fufq4cvp\|d58fnqc4" -- crates` at
  `0bd8dfba` finds nothing.

### 6.3 Seam file re-pin (two repos, lockstep)

- `scripts/descriptor-seam-vectors/rows.py`: delete the 16-key row's `canonical_from=` argument
  (`:299-300`) and the now unused `origin_serialised` helper (`:45-67`).
- Regenerate per `scripts/descriptor-seam-vectors/README.md` with goprobe's `replace` at fork main
  `65717c5` or later. Measured by the author (fork `65717c5`, debug `md` at descriptor-mnemonic
  `e11b4ce1`, `gen.py` exit 0): exactly **one** line of the file changes, the 16-key row's
  `canonical`, which becomes that row's `input` followed by `#thg3nw30`; `address_0`/`address_1` are
  unchanged. New sha256
  `71bba6f24a4b3613d9d7ac7fafa7fc43ff35d1b557ec5b9189b3a3fc6ad5b227` (was `34093bfd…674b`). If a
  regeneration changes any other line, stop and report: the md or fork side has moved and that is not
  this FIX.
- Re-pin `SEAM_VECTORS_SHA256` (`crates/me-cli/tests/descriptor_seam.rs:44`). Update the README's
  baseline line to name the fork rev and md rev used.
- Fork (seedhammer): copy the regenerated file byte-identically to
  `nonstandard/testdata/descriptor_seam_vectors.json` and re-pin `seamVectorsSHA256`
  (`nonstandard/descriptor_seam_test.go:42`). Measured: with exactly those two edits on `65717c5`, all
  six `TestDescriptorSeam*` tests pass (`go test ./nonstandard -run DescriptorSeam -count=1`). No
  fork code changes; the fork already keeps the key's own bytes. This is a test-data change, not a
  normative one, so the Rust-primary rule is not engaged.
- Pre-check of the other 18 admitted rows: the fork's `Encode()` of each normalised input at
  `65717c5` equals the file's `canonical` (measured), so the fixed encoder lands on all 19.

### 6.4 Order

RED commit: rows 1 to 5, the §6.2 rewrite, and the §6.3 data (rows.py, regenerated JSON, pin);
rows 1 to 6 fail. GREEN commit: §2 and §3 and the CHANGELOG entry; the gate passes. The fork PR
(§6.3) is separate and can land in either order: each repo's test reads its own copy.

## 7. Points this document decides that the routing row and rulings do not literally state (PROPOSED)

1. W-1's wording (§4). The routing row says only "re-pin … the promotion announcement text".
2. Keep and say nothing, rather than also warn on a depth/child contradiction (§2, last paragraph).
   Brian's warn-and-proceed ruling covers choosing a warning over a refusal; it does not require a
   warning where the device itself admits silently and nothing is at risk.
3. §4.2 defect 3 is left as history (§3).
4. The seam regeneration's "any other changed line is a STOP" rule (§6.3).

## 8. FOLLOWUPS

No line is added by the PR. After it merges, the closing line below is direct-pushed with the GREEN
sha filled in:

    - [sh2-me-canonical-depth] status=closed@<GREEN sha> grade=I kind=input owner=next-me-fix filed=2026-11-02 src=design/FIX_sec_canonical_keeps_xpub_depth.md repo=mnemonic-engrave :: me's canonical now keeps an xpub's own depth and child number, as the fork's xpubMeta does; seam row accepted/sh-wsh-sortedmulti-16-keys re-pinned in me and the fork (2026-11-23: security sweep unit canonical-keeps-xpub-depth).

## 9. The gate, shown RED on master 0bd8dfba

`CARGO_TARGET_DIR=<private> GATE_LOG_DIR=<private> bash scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh`
exited 1 (`gate: FAIL`): A, 13 FAIL lines (seam sha and pin, 16-key canonical, rows.py, gate.rs,
spec, CHANGELOG); B, 4 FAIL lines (the `descriptor:`, `inferred wallet:` and W-1 lines, and
`show`'s `descriptor:` line; `pack exit 0` and `key as supplied:` were ok); C, nextest
`828 tests run: 828 passed, 2 skipped`, and the 5 new tests passed 0 times; D passed (fmt, clippy,
doc-tests clean on master). With only the §6.3 data applied (regenerated file, pin, rows.py edits),
the A data checks pass and the code, spec and CHANGELOG checks still fail, as they should.
