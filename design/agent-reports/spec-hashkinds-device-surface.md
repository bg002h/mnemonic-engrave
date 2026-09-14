# RECON: hash-digest surface on the SeedHammer II composer, for a hash-KIND spec

Scope: every place a hash digest is accepted or displayed by the Wallet Policy
composer, and every hand-maintained table/gate that a hash-KIND picker would
need a new row in. Recon only -- no design, no code, no review of quality.

Fork checkout: `/scratch/code/shibboleth/seedhammer`, HEAD `0562e811` (2026-09-13).
All file:line citations below are against that commit.

---

## 0. Ground truth: nothing kind-aware exists yet in the composer's hash path

- The wire record is fixed-width, no kind tag: `sysw.ParseHashRecord`
  (`sysw/composer_records.go:191-204`) accepts `hash:` followed by **exactly
  64 lowercase hex characters** (`len(body) != 64` refuses,
  `ErrHashRecord = "sysw: hash: must be exactly 64 lowercase hex characters"`,
  `sysw/composer_records.go:98`). There is no selector byte, no kind field,
  nothing reserved for one.
- `md.SpendPath.Hash` is `*[32]byte` (`md/compose.go:167`), and `md.Compose`
  emits it as `tagSha256` only (`md/compose.go:403`) -- confirmed by the
  spec's own table row: `SPEC_wallet_policy_composer.md:126`, *"HASH | at
  most one `sha256(H)`, H = 32 bytes... `hash256`/`ripemd160`/`hash160` stay
  decodable, not composable"*.
- `hashlock.Digest` (`hashlock/hashlock.go:85-88`) is `sha256.Sum256(x[:])`
  with no kind parameter at all: `func Digest(x *[32]byte) [32]byte`.
- The **decoder** is already kind-aware at the node level:
  `md/md.go:120-121` declares `hash256Body [32]byte` and `hash160Body
  [20]byte` distinctly, and `md/policy_shape.go:283-290` sets
  `br.Hashlock = true` for `tagHash256`/`tagRipemd160`/`tagHash160` but
  **does not append to `br.Sha256Digests` for them** -- only `tagSha256`
  does (`policy_shape.go:285-287`). So today, if a decoded wsh policy ever
  carried a `hash256`/`ripemd160`/`hash160` fragment, the branch would report
  `Hashlock: true` with **no digest value at all**, and
  `composerBranchLines` (`gui/composer_consent.go:94-96`, `for _, d := range
  b.Sha256Digests`) would print nothing for it. This is currently unreachable
  in production because `md.Compose` never emits those tags and the only
  caller of the consent path (`composerConsentFlow` ->
  `composerConsentLinesFor`, one call site at
  `gui/composer_selfcheck.go:225`) only ever consents to what the composer
  itself just built. It is a live gap the moment `md.Compose` gains the
  other three kinds.

This means: today there is exactly ONE kind (sha256, unlabeled) and every
digest in the composer is `[32]byte` by construction. Everything below is
scoped against that baseline.

---

## 1. Every entry path for a digest

| # | Screen / function | file:line | What it validates |
|---|---|---|---|
| 1 | Payload `hash:` record, picked directly off `Which hash?` band 1 | `gui/composer_hash.go:411-415` (`composerHashEdit`, `case sel < len(rows.digests)`) | Parsed upstream by `sysw.ParseHashRecord` -- exactly 64 lowercase hex (`sysw/composer_records.go:191-204`). No confirm screen fires for this arm and **no material is held** (`composerHoldHashlockMaterial` is never called here). |
| 2 | Typed 64-hex fallback (`composerHexEntry`) | `gui/composer_hash.go:79-147` | The keyboard alphabet is hex-only (`composerHexKeys`, line 42); the fragment is truncated to 64 chars (line 87-89) and `valid := len(frag) == 64` (line 91) gates the OK button; on submit, `hex.DecodeString` + `len(raw) != 32` is a redundant belt-and-braces check (lines 97-105) that is stated unreachable given the pad. **Hardcoded to 64 hex / 32 bytes** in three places (the pad cap, the `== 64` gate, the `!= 32` decode check). No material held here either. |
| 3 | Hashlock-phrase route, typed on the device (`hashlockPhraseRoute`) | `gui/composer_hashlock.go:53-106` | Phrase validated by `hashlock.ValidatePhrase` (empty / non-ASCII / ms1-shaped / >100 chars / hex64-shaped, `hashlock/hashlock.go:93-...`); method chosen (`hashlockHardened`/`hashlockSHA256`); digest = `hashlock.Digest(&x)`, always SHA-256, always 32 bytes out of a 32-byte preimage. **Holds material** via `composerHoldHashlockMaterial(st, d, hashlockMaterial{..., provenance: hashlockFromPhrase})` (line 85-87). |
| 4 | Payload `phrase:` record, derived on the device (`hashlockPayloadRoute`) | `gui/composer_hashlock.go:136-168` | Same derivation, method comes from the record (`hashlockMethodOf`, line 138), no phrase screen, no method pick. **Holds material** with `provenance: hashlockFromPayload` (line 156-158). |
| 5 | Payload preimage-PLATE record, decoded directly (`hashlockPreimageRecordRoute`) | `gui/composer_hashlock.go:192-204` | No KDF: `codex32.DecodeMS1Preimage` already produced X at row-build time (`composerPayloadPreimages`, `gui/composer_hash.go:207-227`). Digest = `hashlock.Digest`. **Holds material** with `provenance: hashlockFromPayload` (line 200-202). |
| 6 | Preset archetype (`composerPresetDigest`) | `gui/composer_presets.go:50-60` | A fixed illustrative digest (32 bytes of `0xa8`) baked into one of the six wsh/tr archetypes (`gui/composer_presets.go:101`, `keyed_compose_preset_hashlock_gated`). No validation, no material held -- the operator is expected to replace it via the path's own "Hash lock" edit before Done. **Not named in the recon brief's enumerated list; it is a real fourth (well, sixth counting sub-cases) entry path for a digest value into `SpendPath.Hash`.** |
| — | `No hash lock` | `gui/composer_hash.go:450-452` | Not an entry path -- clears `Hash` to `nil`. |

**Provenance summary (feeds directly into decision #3, see §6 below):** only
routes 3, 4, 5 call `composerHoldHashlockMaterial`. Routes 1, 2 and 6 assign
`st.list.Paths[idx].Hash` directly with **no** entry in `st.hashlockHeld`.

---

## 2. Every display of a digest

All digest display goes through one of two elision helpers, and they are
**not** written the same way:

| Function | file:line | Elision arithmetic |
|---|---|---|
| `composerHashRow` | `gui/composer_hash.go:48-51` | `h[:8]` .. `h[56:]` -- **fixed offsets**, computed against a hardcoded 64-char hex string. |
| `composerHashInPayloadRow` | `gui/composer_hash.go:165-167` | Delegates to `composerHashRow`; same fixed offsets. |
| `composerHashPreimageRow` | `gui/composer_hash.go:171-174` | `h[:8]` .. `h[56:]` -- **fixed offsets**. |
| `composerHashPhraseRow` | `gui/composer_hash.go:183-189` | `h[:8]` .. `h[56:]` -- **fixed offsets**. |
| `composerDigestShort` (consent) | `gui/composer_consent.go:61-64` | `h[:8]` .. `h[56:]` -- **fixed offsets**. |
| `hashlockFirst8Last8` | `gui/composer_hashlock.go:247-250` | `s[:8]` .. `s[len(s)-8:]` -- **length-relative**, the only one that is not hardwired to 64 characters. |
| `hashlockDigestHex` | `gui/composer_preimage_plate.go:499` | Full, unelided hex. Declared in a production file but every caller found is a `_test.go` file (`gui/composer_hashlock_plates_test.go:217`, `gui/composer_preimage_plate_test.go:74,82`) -- currently dead in production. |

**Consequence for a 20-byte digest:** if any of the five fixed-offset
functions were fed a 40-character hex string (20-byte digest) unmodified,
`h[56:]` would panic with an out-of-range slice index (len 40 < 56). By
contrast, `hashlockFirst8Last8` would compute `s[len(s)-8:]` = `s[32:]`
correctly on a 40-char string -- **its arithmetic is already
length-agnostic; only its parameter type (`h [32]byte`, a fixed array) would
need to widen.** This is a real distinction for future work: some
call sites need only a type change, others need an arithmetic rewrite.

**Where each is drawn:**

- **`Which hash?` picker rows** (composer_hash.go's four row functions
  above) -- one line each, band-limited. §6c's own budget: *"28 characters,
  inside the 436 px label"* (`SPEC_wallet_policy_composer.md:382`); the
  436 px figure is the picker's text band, measured by `composerTextBand`
  (`gui/composer_paged.go:49-54`), not a separate constant.
- **Consent screen** (`composerBranchLines`, `gui/composer_consent.go:94-96`)
  -- `"  hash " + composerDigestShort(d)`, one line per `Sha256Digests`
  entry on the branch.
- **§8i rule restated at consent** -- printed once, unconditionally text
  (`composerCopyHashRule()`), not digest-bearing itself; fires when any
  branch has `Sha256Digests` (`gui/composer_consent.go:198-203`).
- **Confirm/derive modals** -- `hashlockPhraseRoute`/`hashlockPayloadRoute`/
  `hashlockPreimageRecordRoute` all build their body via
  `hashlockFirst8Last8(h)` (the safe one) before handing a pre-formatted
  string into `composerCopyHashlockConfirm` /
  `composerCopyHashlockPreimageConfirm` / `composerCopyHashlockReconcile`
  (`gui/composer_hashlock.go:75-76,100,144-145,164-165,193-194`). These copy
  functions themselves take a `string` parameter, not a digest -- no width
  assumption baked into the copy layer itself.
- **Preimage-plate pick lead and census rows** -- `composerCopyPreimagePlateLead`,
  `composerCopyPreimagePlateRow`, `composerCopyPreimageNotOnAnyPath`,
  `composerCopyPreimageDeclined` (`gui/composer_copy.go:717-...`) all take a
  pre-formatted `first8last8 string` built by the caller via
  `hashlockFirst8Last8` (`gui/composer_preimage_plate.go:158,401,410,412`,
  and `gui/composer_hashlock_plates.go:277`). Same: safe at the copy layer.
- **Plate locator rows** (engraved, not drawn) -- `hashlockPlateLocator`
  (`gui/composer_preimage_plate.go:232-249`) prints `"hash  " +
  hashlockFirst8Last8(digest)` -- the safe helper again.
- **"Policy-id header"** -- this is a *different* value, not a hashlock
  digest: `composerConsentLinesFor` prints `fmt.Sprintf("%s: %x", kind, id)`
  and `fmt.Sprintf(label, stub)` (`gui/composer_consent.go:184-188`), where
  `id` is `[16]byte` (`md.FormAwareIdChunks`, `md/template_id.go:163-174`)
  and `stub` is `[4]byte` (`md.FormAwareStubChunks`, `md/template_id.go:122-128`).
  Both are printed **in full** (`%x`, not elided) and are computed by a
  wallet-id hash entirely independent of the hashlock digest mechanism.
  **No width-budget interaction with a hash-KIND change** -- flagging this
  so the future spec doesn't conflate the two.
- **Restore document** -- **does not exist for the composer.** Explicitly
  named out of scope by H6: `SPEC_hashlock_H6_preimage_plates.md:2644` lists
  *"a restore document for the composer"* among the items §13 defers. The
  closest analogs are `composerCensusLines`/`composerPreimageCensusLines`
  (`gui/composer_census.go:98-113`, `gui/composer_preimage_plate.go:385-419`)
  and the plate's own engraved locator rows (`hashlockPlateLocator` above) --
  both already covered.

**No display site was found that assumes the digest is 32 bytes for LAYOUT
reasons** (row width, wrap point) -- because every drawn row elides to a
fixed `first8..last8` shape regardless of the underlying length. The 32-byte
assumption that *would* break is entirely at the **string-slicing / parameter-type**
level (the five fixed-offset functions above and every function signature
`(..., digest [32]byte)` / `(..., h [32]byte)` / `(..., d [32]byte)` across
`composer_hash.go`, `composer_hashlock.go`, `composer_consent.go`, and
`composer_preimage_plate.go`), not at the pixel-budget level.

---

## 3. Where a kind picker would slot into the composer flow

Path creation and path editing both reach the hash screen the same way:

- **New key-less path**: `Path N` -> `ChoiceScreen{"Keys", "A hash, no keys"}`
  (`gui/composer_shape.go:341-345`) -> if "A hash, no keys" and wrapper is
  `tr`, refused outright (`gui/composer_shape.go:352-356`); else the §8a
  EXPERIMENTAL confirm-to-proceed (`gui/composer_shape.go:367-370`) -> directly
  into `composerHashEdit` (`gui/composer_shape.go:373`). A `false` return
  discards the just-created path (line 374-376). A path ending with neither
  keys nor hash is discarded as a cancel (line 379-382).
- **Editing an existing path**: `Path N` menu ("Keys", "Time lock", "Hash
  lock", "Remove path", conditionally "Move up") -> "Hash lock" (index 2,
  `gui/composer_shape.go:445-451`) -> `composerEditCanRenumber` /
  `composerShapeGuard` (§8j) if applicable -> `composerHashEdit`.
- **Inside `composerHashEdit`** (`gui/composer_hash.go:389-457`): one
  `composerPickScreen` titled `Path N hash`, lead `Which hash?` (or the
  no-payload lead), six bands in this fixed order (`composerHashRows`,
  `gui/composer_hash.go:318-357`): (1) payload `hash:` digests, (2) payload
  preimage-plate records, (3) payload `phrase:` records, (4) `Type a
  hashlock phrase`, (5) `Type 64 hex`, (6) `No hash lock`. Each row dispatch
  is by recorded band-start index (`rows.preimageRow`, `rows.phraseRecRow`,
  `rows.phraseRow`, `rows.hexRow`, `rows.noneRow`), not by arithmetic on the
  selection -- a comment at `gui/composer_hash.go:288-294` explains this was
  a deliberate fix for a prior index-drift bug (r2 review C-4).
- A hash-KIND choice, if added, most naturally inserts as a NEW step **before
  or inside** band 5 (typed hex) and/or band 1 (a payload `hash:` record,
  once that record carries a kind tag) -- i.e., "what kind is this digest"
  needs answering wherever a digest arrives with no self-describing kind
  (typed hex today; a legacy bare `hash:` record under decision #2). It does
  **not** need to ask for bands 2-4 (preimage record, phrase typed or
  payload) since those derive X and can compute the digest in whatever kind
  is chosen, or -- per decision #2 -- the wire record states the kind and
  the device need not ask. No screen sequence is proposed here; this is
  strictly what precedes/follows today.

---

## 4. Every gate that would need a new row

| Gate | file:line | Current state |
|---|---|---|
| **Composer copy table + declared-body count** | `gui/composer_copy_test.go:31-33` (table), `:371` (assertion `if declared != 78`) | AST-scans `composer_copy.go` for every `composerCopy*` func and requires each to have a table row; **78** is a hand-maintained literal bumped with a dated comment (`gui/composer_copy_test.go:317-368`) each time a body is added. Any new hash-kind copy body (a kind-picker lead, a kind-mismatch refusal, a 20-byte-digest warning) must add both the function's row and bump this literal, or the test fails outright (it is a hard `t.Errorf`, not a soft warning). |
| **Modal-fit gate** | `gui/modal_fits_test.go:203-227` (`assertModalBodyFits`), margin constant `gui/modal_fits_test.go:53` (`modalBodyMargin = 80`) | Runs per-body via the copy table's own test coverage (indirectly) and via explicit calls (e.g. `gui/composer_hash_test.go:51`, `TestComposerHashRuleIsStatedAtEntry`). A new §8i-adjacent body (e.g. "this digest is 20 bytes, not derived here") needs its own `assertModalBodyFits` call site the way every other §8 body has one, or it ships uncovered by the raster/fits gate (per `TestComposerCopyTableCoversEveryBody`'s own stated purpose). |
| **`Which hash?` row-count formulas (no preimage/phrase records)** | `gui/composer_hash_test.go:96` (`if got := len(rows.labels); got != n+3`) | Hardcodes "n payload hash-digests + 3 fixed rows (phrase, hex, none)". A new fixed row (a kind picker inserted as its own band) changes this to `n+4`. |
| **`Which hash?` row-count formula (with preimage + phrase records)** | `gui/composer_hash_test.go:178` (`if got, want := len(rows.labels), 3*n+3; got != want`) | Same shape, `TestWhichHashRowsCarryTheTwoNewBands`. Also hand-maintained. |
| **Fixed page-1 touch-target count for `Which hash?`** | `gui/composer_hashlock_test.go:1433-1460` (`TestWhichHashPageHoldsFiveRows`) | Asserts a *specific fixture* (1 hash: record + 1 preimage record + 1 phrase record = 6 labels) draws **exactly 5** tappable rows on page 1 and pushes `No hash lock` to page 2. The test's own comment: *"If this moved, every row count in Task 8b moved with it."* A new band changes this pagination boundary. |
| **One-line-per-row geometry measurement** | `gui/composer_hash_test.go:239-269` (`TestWhichHashRowsDrawOnOneLine`) | Measures all six existing row forms (plus the three literal labels) at `sh2DisplaySize` and asserts each wraps to exactly one line in the picker's text band. A new row form (e.g. `"hash <i>  <kind>  <first8..last8>"` if the kind is echoed on the row) must be added to this table and re-measured; the file's own comment already anticipates width pressure (composer_hash.go:161-164, the reason `(in payload)` was chosen over `(preimage in payload)`). |
| **`composerCopyTable`'s per-row §8/H-section pins** | `gui/composer_copy_test.go:130-215` | Every hashlock-adjacent body already in the table (H2-\*, H6-\*, §8i, §8h) is pinned verbatim against the spec text; §8i's line ("The hash must be SHA-256 of a 32-byte value...") is sha256-specific prose and would itself need editing plus a table-row update the moment a second kind exists. |
| **`composerPickScreenMaxRows` ceiling** | `gui/composer_paged.go:243` (`const composerPickScreenMaxRows = 24`) | A fixed-size `[24]Clickable` array backing every pick screen, `Which hash?` included (`gui/composer_paged.go:286`). Not currently under strain (six bands, small counts), but any new fixed row narrows headroom before payload-driven bands (1-3) hit it. Referenced defensively already at `gui/composer_hash_test.go:113` (`if composerPickScreenMaxRows < 2+3`). |
| **Preimage-plate pick screen's fixed row count** | `gui/composer_preimage_plate_test.go:779-798` (`TestComposerPreimagePlatePickDrawsAllFourRows`, `if len(pts) != 4`) | Not directly kind-related (it counts plate FORM choices: string / phrase / phrase+QR / decline), but sits in the same family of hand-maintained counts and would need re-checking if a kind-aware plate form were ever added. |
| **`TestComposerCopyIsDrawable`'s "applied to all 39" comment** | `gui/composer_copy_test.go:266-270` | Stale comment (says "all 39"; the table is 78-strong per the count above) -- a pre-existing drift, unrelated to hash kinds, worth naming since a reviewer reading that comment as current would undercount. |

No walk/journey test was found asserting a fixed *total step count* through
the hash-entry flow (searched `composer_flow_test.go`, `composer_hashlock_test.go`,
`composer_hashlock_held_test.go`, `composer_hashlock_geometry_test.go` for
`Test.*[Ww]alk` and `needle`-style counters) -- the counting gates that exist
are all **row/label/body counts**, enumerated above, not step-sequence counts.

---

## 5. §8 copy rows that mention hashes (quoted, with section numbers)

From `SPEC_wallet_policy_composer.md`:

**§6c, "Hashlock entry"** (lines 380-399, not itself a §8 blockquote but the
narrative section §8i is carved from):

> Primary: pick from the payload's `hash:` records, each row `hash <i>
> <first 8>..<last 8>` in the host's pack order (28 characters, inside the
> 436 px label budget; a 64-hex row would be cut, not wrapped). Fallback:
> type 64 hex on the keyboard, accepted only when exactly 64 valid hex
> characters are present. At entry and at consent the device states the
> 32-byte rule: `sha256(H)` compiles to `OP_SIZE <32> OP_EQUALVERIFY
> OP_SHA256 <H> OP_EQUAL`, so the preimage MUST be exactly 32 bytes; a digest
> of a passphrase directly can never be spent (§8i...).

**Table row, §4b** (line 126):

> `HASH` | at most one `sha256(H)`, H = 32 bytes, the SHA-256 of a 32-byte
> preimage (§6c) | both reference wallets use sha256 only; `hash256`/
> `ripemd160`/`hash160` stay decodable, not composable

**§8i, "Hashlock entry rule (at entry and at consent)"** (lines 769-774):

> The hash must be SHA-256 of a 32-byte value. A
> passphrase must be hashed to 32 bytes first, then
> hashed again. A hash of the passphrase itself can
> never be spent.

**§8h, "Every path needs a preimage"** (lines 730-767) -- four arms, none
name a kind, all say "the preimage of a hash" / "a hashlock preimage"
generically; quoted in full already in the code excerpt above and not
re-quoted here for length, but every one of the four bodies would need
re-reading once a second kind exists, since "the preimage" (singular,
kind-unaware) is the noun used throughout.

Outside `SPEC_wallet_policy_composer.md`, `SPEC_hashlock_H2_device.md` and
`SPEC_hashlock_H6_preimage_plates.md` are the two other hashlock specs named
in the dispatch; both are sha256-only end to end (their own §2/§3 define
`hashlock.Digest` as SHA-256), and neither declares a kind concept -- a targeted
grep for `ripemd`/`hash160`/`hash256` inside either spec's body returned no
hits beyond the one composer table row quoted above.

---

## 6. Preimage vs. digest length -- where the code conflates them

**Established, and independently confirmed at the codec layer:** `md/md.go:120-121`
already models `hash256Body [32]byte` separately from `hash160Body [20]byte`
-- the DECODER does not conflate preimage and digest width, because it never
sees a preimage at all, only a digest, and the digest's width already varies
correctly by kind at that layer.

**The composer/gui layer conflates them everywhere, because it has only ever
had one kind to support:**

- Every hashlock-adjacent function signature in `composer_hash.go`,
  `composer_hashlock.go`, `composer_consent.go` and `composer_preimage_plate.go`
  types its digest parameter as `[32]byte` (a fixed array, not a slice) --
  e.g. `composerHashRow(i int, digest [32]byte)` (`composer_hash.go:48`),
  `composerDigestShort(d [32]byte)` (`composer_consent.go:61`),
  `hashlockFirst8Last8(h [32]byte)` (`composer_hashlock.go:247`),
  `hashlockPlateLocator(path int, digest [32]byte, ...)`
  (`composer_preimage_plate.go:232`), `hashlockDigestHex(h [32]byte)`
  (`composer_preimage_plate.go:499`), `payloadStatesDigest(payload [][32]byte, h [32]byte)`
  (`composer_hashlock.go:174`), `hashlockRelationLine(payload [][32]byte, h [32]byte)`
  (`composer_hashlock.go:212`), `composerHashInPayload(r composerHashRowSet, st *composerState, d [32]byte)`
  (`composer_hash.go:368`). None of these distinguish "this is the preimage
  width" from "this is the digest width" -- they are all named `digest`/`d`/`h`
  and are all `[32]byte`, which is currently correct only because sha256's
  digest happens to equal the preimage's 32 bytes.
- **The map key is the sharpest instance.** `composerState.hashlockHeld` is
  `map[[32]byte]hashlockMaterial` (`gui/composer_state.go:81`), keyed by
  **digest**. `hashlockMaterial.preimage` is separately `[32]byte`
  (`gui/composer_state.go:337`) -- so the struct itself already has two
  distinct `[32]byte` fields for two distinct 32-byte quantities that, for a
  ripemd160/hash160 kind, would need to become a 32-byte preimage keyed
  under a **20-byte** digest. A `[32]byte` map key cannot hold a 20-byte
  value without a padding convention or a type change (e.g. to `[]byte` via
  `string(h[:])`, or a `HashLock{Kind, Digest}`-shaped key struct).
  `composerPreimagePlates`'s `seen map[[32]byte]bool`
  (`gui/composer_preimage_plate.go:78`) has the identical shape.
- `hashlockPayloadPreimage{preimage [32]byte; digest [32]byte}`
  (`gui/composer_hash.go:195-198`) and `hashlockPlate{digest [32]byte; ...}`
  (`gui/composer_preimage_plate.go:55-60`) both carry preimage and digest as
  two same-typed, same-width fields side by side -- correct today, silently
  wrong the moment digest width diverges from preimage width.
- At the wire-composition layer, `md.SpendPath.Hash *[32]byte`
  (`md/compose.go:167`) is the same conflation one level up: it is a pointer
  to a 32-byte array with no kind tag, feeding only `tagSha256`
  (`md/compose.go:403`).

**What is NOT conflated:** the codex32 preimage plate encoding
(`codex32.EncodeMS1Preimage`/`DecodeMS1Preimage`, referenced at
`gui/composer_hash.go:220`, `gui/composer_preimage_plate.go:280`) is
correctly scoped to the **preimage** (always 32 bytes, per the given fact
that all four miniscript fragments require a 32-byte preimage) and never
touches digest width at all -- so the preimage-plate machinery itself needs
no change for a kind picker; only the digest-side types above do.

---

## 7. Design decisions -- contradiction check

1. **All four kinds authorable from the device composer.** No contradiction
   found; confirmed NOT yet true (§0 above) and the gap is precisely at
   `md.Compose` (`md/compose.go:403`, sha256-only) plus every `[32]byte`
   digest signature enumerated in §6.
2. **`hash:` record carries the kind; device asks when a digest is TYPED;
   bare legacy `hash:` means sha256.** No contradiction found; confirmed the
   wire record currently carries NO kind tag at all (`sysw/composer_records.go:191-204`,
   §0 above), so this is a clean, additive change to the record grammar
   (per the Rust-primary rule, the host's `ms-cli`/`me-cli` sysw record
   parser is presumably the one to change first -- not inspected here, out
   of this recon's file scope).
3. **20-byte kinds warn only when the device did not derive the preimage
   itself; can the code currently tell provenance apart?** **Yes, partially.**
   `st.hashlockHeld` (`gui/composer_state.go:81`) holds a material entry
   **only** for digests assigned via routes 3, 4, 5 in §1's table (typed
   phrase, payload phrase, payload preimage record) -- i.e. every route
   where the device itself computed `hashlock.Digest` from an X it derived
   or decoded. Routes 1, 2, 6 (payload `hash:` row picked directly, typed
   hex, preset placeholder) never call `composerHoldHashlockMaterial`, so
   `_, held := st.hashlockHeld[digest]` is already exactly the boolean
   decision #3 describes: false for "digest handed in", true for "device
   derived it". **Caveat, and it matters:** this is a digest-VALUE-keyed
   set for the whole composition, the same shape as the older
   `phraseDigests` set (`gui/composer_state.go:47`, H5 §2) and inherits its
   documented imprecision: *"the same digest re-typed as 64 hex is still
   by-phrase"* (`gui/composer_provenance_test.go:108`, asserted as
   intentional). So if a composition first derives digest D on one path
   (holding material) and later a DIFFERENT path is assigned the SAME value
   D via the payload-row or typed-hex route, `hashlockHeld[D]` still reports
   "held" for that second, undived assignment -- the existing design choice
   is per-digest-ever-seen, not per-assignment-event. A spec introducing the
   §8i-adjacent 20-byte warning needs to either accept that same imprecision
   (consistent with existing precedent) or add a genuinely per-assignment
   provenance field, which does not exist anywhere today -- `md.SpendPath`
   has no provenance field and none is proposed by any code found.
4. **`HashLock { kind, digest }`, digest length derived from kind.** No such
   type exists anywhere in the fork (`grep -rn "type HashLock" .` returns
   nothing.) The nearest existing shapes are `sysw.HashlockMethod`
   (`sysw/composer_records.go:74-83`, an unrelated KDF-method selector, not
   a digest-kind selector) and the md-codec's internal `hash256Body`/
   `hash160Body` node bodies (§0, §6 above), which are kind-differentiated
   but private to the decoder and not exposed as a reusable public type.
   Introducing `HashLock{Kind, Digest}` is new work at both the `md` and
   `gui` layers, not a rename of something that already exists.

---

## Files read in full or in the relevant part for this recon

- `gui/composer_hash.go`, `gui/composer_hashlock.go`, `gui/composer_consent.go`,
  `gui/composer_preimage_plate.go`, `gui/composer_hashlock_plates.go`,
  `gui/composer_state.go`, `gui/composer_shape.go`, `gui/composer_census.go`,
  `gui/composer_presets.go` (hash-adjacent excerpt), `gui/composer_copy.go`
  (hash-adjacent excerpt), `gui/composer_copy_test.go`, `gui/composer_gates_test.go`,
  `gui/modal_fits_test.go` (gate mechanics), `gui/composer_hash_test.go`,
  `gui/composer_hashlock_test.go` (row-count tests), `gui/composer_provenance_test.go`,
  `gui/composer_preimage_plate_test.go` (excerpt).
- `md/compose.go`, `md/md.go`, `md/policy_shape.go`, `md/template_id.go`
  (excerpts).
- `sysw/composer_records.go` (full).
- `hashlock/hashlock.go` (excerpt).
- `SPEC_wallet_policy_composer.md`, `SPEC_hashlock_H6_preimage_plates.md`
  (targeted sections, in `mnemonic-engrave/design/`).
