**⚠ 22 Important (0 Critical): sh2 grading. Two disagreements with the triage: 31b goes from M to I, and 30c/30d/49 go from M to I.**

# Bug-hunt grading, seedhammer fork (sh2), slices 1-5

Inputs: hunt report `bughunt-2026-10-31-sh2.md`, triage `bughunt-triage-sh2-s{1..5}.md`,
evidence `/scratch/cache/bughunt/sh2/s{1..5}/`. Source read at fork worktree
`sh-worktrees/bughunt-triage-s1`. I created no scratch worktree: every C/I fix is a
triage scratch patch that I read and that passed its row in the triage run.

Reach fact used throughout (checked in source). The default (`!refugium`) build
is the shipped artifact. `cmd/controller/nfc_default.go` and `lockboot_default.go` are
`!refugium`. `flake.nix` `build-firmware` builds `./cmd/controller` with no
tag, and `.github/workflows/image.yml` uploads that build as `seedhammerii-<version>.uf2`. So any
defect reachable through NFC scan in the default build counts as device-reachable.

## Grades

| # | final grade | triage proposal | agree? | row tests the defect? | likelihood conditions / why |
|---|---|---|---|---|---|
| 2 | I | I | yes | yes (Back on keyboard goes on with passphrase "") | Silently ignored input: Back becomes "no passphrase", and the plate is engraved for a different wallet. Fails condition 1. |
| 3 | I | I | yes | yes | Same as 2 in the BIP-85 flow: the child is derived from the wrong root. |
| 4 | I | I | yes | yes (3 of 12 words returned as confirmed) | A half-typed mnemonic is returned as confirmed. Wrong data, fails condition 1. |
| 9 | M | M | yes | yes | Fails closed: later scans keep reporting "buffer overflow" until reset. Needs (a) a tag larger than the buffer and (b) a second scan in the same session. |
| 31a | I | I | yes | yes | Same harm as 2 in the multisig supply flow. |
| 31b | **I** | M | **no** | yes (Back on the question goes on with "") | The harm is the same as 31a: Back on "BIP-39 passphrase?" goes on with the empty passphrase, so the seat is supplied by a different key (fails condition 1). fix-31 fixes 31a and 31b together, so skipping 31b's row as deferred while the fix makes it pass would be incoherent. |
| 32 | I | I | yes | yes | Back on the SLIP-39 passphrase question recovers with "", which is a wrong seed. |
| 33 | M | M | yes | yes | The verify flow only checks; it writes nothing. A wrong passphrase reports "not checked yet" or a mismatch, so it fails closed. Needs (a) the verify flow and (b) Back on the keyboard. |
| 42 | M | M | yes | yes | Back on "Which md1?" is taken as "Full policy" and leads to the EXPERIMENTAL screen, which the holder can still leave. No engraving happens without further confirmations. Needs (a) the build flow and (b) Back at that one picker. |
| 48 | M | M | yes | yes | Back on the first slot asked releases @1's seat. The composer self-check and the review screen show the unseated slot before any engraving. Needs (a) a resumed composer and (b) Back on the first slot. |
| 5 | M | M | yes | yes | The chunk index past the total is accepted, but expand refuses the set as inconsistent, so it fails closed. Needs (a) a malformed or hostile chunk and (b) a scan order that completes. |
| 21 | I | I | yes | yes | The self-check accepts an unseated slot with a non-default origin. The self-check is the last guard before engraving, and here the guard does not guard (a tool defect in what it claims). |
| 22 | I | I | yes | yes | Button3 picks a row that is not on screen. The holder confirms an item they cannot see, so the wrong row is chosen silently. |
| 23 | N | N | yes | yes | Display only ("1 h 60 min"). |
| 24 | N | N | yes | yes | The doc does not match the contract. The existing test pins n plus a marker. |
| 45 | M | M | yes | yes | A keyless path is refused later, at finish, with a named reason rather than at creation. Fails closed. Needs (a) a legacy wrapper and (b) 'A hash, no keys' on the first path. |
| 51 | M | M | yes | yes | An unsigned and a signed copy of one transaction count as two candidates, and the holder is asked to pick. Nothing is wrong. Needs (a) two payloads of the same txid and (b) different witness data. |
| 1 | I | I | yes | yes (re-encode gives a different xpub) | Depth and child number are rewritten from the origin length, so the displayed and engraved xpub string differs from the input. Wrong plate data. |
| 6a | I | I | yes | yes | The text parser accepts threshold > n or < 1, which gives an unspendable or wrong policy on the plate. |
| 6b | I | I | yes | yes | Same as 6a through the BlueWallet door. |
| 6c | I | I | yes | yes | wpkh(sortedmulti) and similar are accepted, giving a wrong script type and wrong addresses. |
| 10a-d | M | M (latent) | yes | yes | The UR fountain decoder has no caller in the shipped build (ruling P-S3-2). It fails by panic or OOM, not a wrong result. Becomes I when wired. |
| 11 | I | I | yes | yes | codex32 `New` panics on short data reachable from an NFC scan in the shipped default build. A device crash on a scanned input. |
| 12 | M | M | yes | yes | `Valid` panics on 0 or 27 words. Callers bound the word count first. Needs (a) a caller that skips the bound and (b) that length. |
| 34 | M | M (borderline) | yes | yes | A pkh leaf is described as pk, but no caller emits taproot leaves from this path today. Latent, becomes I when wired. |
| 35 | M | M | yes | yes | `ParsePath` wraps an unhardened index of 2^31 or more. The only callers are host tools and fixtures, and the wrap shows as "h" on review. Needs (a) an out-of-range index typed by hand and (b) not noticing the change. |
| 36 | M | M | yes | yes | Same class as 35 for HD key origins in UR. UR is latent (P-S3-2). |
| 37 | M | M | yes | yes | The UR encoder's range shape is refused by its own parser, so it fails closed. Latent. |
| 38 | M | M | yes | yes | Script type mismatch in UR. Latent. 6c's parser fix covers the text door. |
| 47 | M | M (borderline) | yes | yes | Host biptool only. `-seedlen` is restricted to 16-64 and these lengths are deliberate F-116 fixtures. F-116 already prints "NOT VALID codex32 ... Nothing will parse this" and does not refuse, so it fails loudly. Needs (a) biptool and (b) an off-spec length chosen on purpose. |
| 50a | I | I | yes | yes | A Name-only BlueWallet file is accepted with zero keys and then panics in Encode (shipped scan door). |
| 50b | I | I | yes | yes | No Format gives Script Unknown and then an Encode panic (shipped scan door). |
| 50c | I | I | yes | yes | Derivation after the key lines gives an empty origin: wrong origin data on display and engraving. |
| 52 | N | N | yes | yes | Wrong error text for a lone chunk ("not supported" instead of "chunk set incomplete"). It still refuses. |
| 7 | I | I | yes | yes (HashData differs) | The load-map zero-fill reads entry 0's size for every entry, so the image hash is wrong. That is a signed-image or boot-verify defect. |
| 8 | M | M | yes | yes | A Type 2 tag over 848 bytes reads short with a clean EOF, and the NDEF parse then refuses the truncated message. Fails closed. Needs (a) a tag above the size limit and (b) content that long, which md1 and mr1 payloads do not reach. |
| 13 | I | I | yes | yes | The new white-label string overlaps a UTF-16 neighbour in OTP. That is wrong data in one-time-programmable memory and cannot be undone. |
| 14 | I | I | yes | yes | `|` is used where `&` belongs, so an invalid OTP entry is read as valid. The device shows a wrong label or identity. |
| 18 | M | M (narrowed) | yes | yes (subtests `rtd-valid-minimal`, `zero-length`) | An empty Text or URI record is refused or mis-sized only for hand-built minimal records. Fails closed. Needs (a) a zero-length payload and (b) that record type. |
| 19 | M | M | yes | yes (477..479) | A uf2 payload size above the data area panics on the host uf2 tool. Needs (a) a malformed uf2 and (b) the host tool. |
| 20 | M | M | yes | yes | Start address 0 is reported as non-contiguous and refused, so it fails closed. Needs (a) a uf2 at address 0 and (b) the host reader. Real images are not at 0. |
| 25 | N | N | yes | yes | alpha4 panics on very wide bounds in an asset tool. |
| 26 | N | N | yes | yes | `bspline.InterpolatePoints(nil)` panics. No caller passes nil. |
| 27a | M | M | yes | yes | Round-trip in the golden test helper. Test infrastructure only, and it fails loudly. |
| 27b | N | N | yes | yes | Zero-count header in the test helper. |
| 27c | N | N | yes | yes | SVG aspect is 0 in a debug output. |
| 28 | M | M | yes | yes | biptool `derive -path m bip39 -words 12` panics. Host tool, fails loudly. Needs (a) biptool and (b) that argument combination. |
| 29 | M | M | yes | yes | The assets palette accepts 256 colours. The build-time tool then emits an off-by-one palette, which is caught at asset review. Needs (a) a 256-colour source and (b) the build step. |
| 30a | M | M | yes | yes | vectorfont panics on a path that starts with a line. Fails loudly at build time. |
| 30b | N | N | yes | yes | A missing input file reports EOF. Wrong message, still fails. |
| 30c | **I** | M | **no** | yes (second curve's control point) | vectorfont is a live pipeline tool, and its output is the engraved font. A repeated `S` silently reflects the wrong control point, which gives a wrong glyph shape and no error (fails condition 1, and could make a plate unreadable). Unlike code with no caller, this tool runs on every font regeneration. |
| 30d | **I** | M | **no** | yes (implicit repeated relative `l`/`h`) | Same pipeline: implicit repeated relative coordinates are taken as absolute, so the glyph is silently wrong. Relative `c` has the same stale-pen fault, and fix-30d does not cover it (see Fixes). |
| 41 | N | N | yes | yes | A degenerate zero-length curve samples an origin point. The "very short" claim is refuted (see below). |
| 43 | N | N | yes | yes | `Bounds.Empty()` is wrong for an inverted y. Used only in a debug tool. |
| 46 | N | N | yes | yes | glyphtrace ranks a short channel as worst. Advisory diagnostic only. |
| 49 | **I** | M | **no** | yes (backtrack point dropped) | The vectorfont optimizer drops the backtrack to the stale pen point, so the engraved stroke path skips a segment and the glyph is silently wrong. Same reasoning as 30c. |

Open question for the architect, should the 30c/30d/49 raise be contested: is a
build-time tool whose output is reviewed only by eye "fails closed"? I grade
no, because a slightly wrong glyph passes a visual check and the plate is engraved with it.

## False positives and dropped sub-cases

- **40 (FALSE POSITIVE): accepted.** The planted fault fails the row, the unmodified code
  passes, and the code matches the datasheet's Table 7 (0 means "use the negotiated current").
- **18 narrowed: accepted.** Only the minimal and zero-length records fail. The broader
  claim (every empty Text or URI) passes on unmodified code.
- **19, 480 case dropped: accepted.** 480 is within the data area and parses
  correctly. 477..479 are the defect.
- **41 "very short curve" claim refuted: accepted.** Long and short non-degenerate curves sample on
  the curve. Only zero-length, nil and seeded degenerate inputs fail.
- **UNRESOLVED 15, 16, 17, 44: accepted as UNRESOLVED.** These are TinyGo-only drivers (pio,
  st25r3916, clrc663) with no host seam. Filed to FOLLOWUPS as `needs-seam`, owned by the
  fix phase's device-test step. No grade.
- **UNRESOLVED 39: accepted as UNRESOLVED, flagged potentially I.** The lockboot default path
  (`lockboot_default.go`) is in the shipped build. If the report holds, it is a boot or lock
  defect. It needs a host seam or a device test before the fix closes. Owned by the fix phase,
  not deferred.

## Product rulings

- **S1-Q1 (Back on a passphrase keyboard or question):** Back steps back. It never means "no
  passphrase" (2026-08-19 "lose nothing" directive, and `passphrase_off.go`'s own contract: "ok
  false means 'no passphrase' or 'step back', never 'take one'"). No new text.
- **S1-Q2 (where Back lands in supply multisig):** accept fix-31's `return`. Back ends the supply
  flow and returns to the menu, under the refugium notice too. Re-asking the question is a
  later nicety (`followup-supply-reask`, N).
- **S1-Q3 (item 4, Back on a half-typed mnemonic):** do not return the mnemonic as confirmed.
  The existing behaviour of leaving seed entry is unchanged. No new text.
- **S1-Q4 (42):** Back on "Which md1?" aborts the build flow (fix-42, deferred M).
- **S1-Q5 (9):** clear the overflow state on EOF or error (fix-09, deferred M).
- **S2-Q5:** an index past the total is classified as `gatherIgnored` (deferred).
- **S2-Q23:** print "2 h", not "2 h 0 min" (deferred N).
- **S2-Q24:** the contract is n runes plus the marker, as the existing test pins it, so only the doc
  comment changes (deferred N).
- **S2-Q51:** the same txid merges whatever the witness. Doc and code follow that (deferred M).
- **S3-Q1 (47):** keep F-116's warn-and-exit-0 in biptool. The library-level `NewSeed`
  length check is deferred.
- **S3-Q2:** grade code with no caller by its reach today. Each latent item's FOLLOWUPS
  entry says "becomes I when wired".
- **S3-Q3 (delete bc/* vs keep it for a future UR door): open, for the architect.** It is a product
  decision outside this brief. The UR items stay deferred meanwhile.
- **S3-Q4 (cap on seqLen):** decided with fix-10 when it is taken (deferred).
- **S3-Q5 (seam vectors):** me leads. The file lives in mnemonic-engrave
  `crates/me-cli/testdata/descriptor_seam_vectors.json`, and the fork vendors it byte-identically.
  Regenerate through `scripts/descriptor-seam-vectors/` (README recipe) and re-pin the sha in
  both `crates/me-cli/tests/descriptor_seam.rs` and the fork's
  `nonstandard/descriptor_seam_test.go`. Update `SPEC_descriptor_input.md` §4.2 (see Fixes).
- **S3-Q6:** enforce threshold and script-type validity in the parser (scan door), as fix-06a and
  fix-06c do, not in the encoder.
- **S3-Q7:** defer.
- **S3-Q8:** the bip380 DeepEqual test edits that fix-01 needs are confirmed as test changes, not
  regressions.
- **S3-Q9:** the default build is shipped (reach fact above).
- **S4-Q8 (Type 2 size cap), S4-Q18, S4-Q19 (refuse vs clamp), S4-Q14 (retry until finish):** all deferred
  with their items. For 14 itself, only the bit test changes now.
- **S5-Q2..Q4:** defer with their items. For 30c, 30d and 49, see Fixes.

## Fixes (C/I, exact)

All patches are in `/scratch/cache/bughunt/sh2/s<n>/`. Each passed its row in the triage run.
No toolkit manual change is needed: these are device and fork-internal paths, and no `mr` or
`md-cli` flag or printed text changes.

| # | patch | change (exact) | tests that change | docs in lockstep |
|---|---|---|---|---|
| 2 | s1/fix-02 | `gui/singlesig.go`: `pass, pok := syswPassphraseFlow(ctx, th); if !pok { continue }; passphrase = pass` | RED row passes | none |
| 3 | s1/fix-03 | `gui/bip85.go`: same pattern, `continue` | RED row | none |
| 4 | s1/fix-04 | `gui/derive_xpub.go`: `_, confirmed := inputWordsFlow(...); if confirmed && !isEmptyMnemonic(mnemonic) {` | RED row | none |
| 31a, 31b | s1/fix-31 | `gui/multisig.go:145`: `sel, ok := askBIP39Passphrase(ctx, th, ppChoice); if !ok { return }; if sel == 1 { pass, ok := syswPassphraseFlow(ctx, th); if !ok { return }; passphrase = pass }` | both RED rows (31b is NOT skipped) | none |
| 32 | s1/fix-32 | `gui/slip39_polish.go:290`: `psel, pok := ppChoice.Choose(ctx, th); if !pok { return nil, false }` | RED row | none |
| 21 | s2/fix-21 | `gui/composer_selfcheck.go`: the expected origin of an unseated slot is `md.DefaultOrigin(st.list.Wrapper, account)` at the lowest free account. New error: `"self-check: unseated slot @%d declares %s, not §4f's default %s"` | RED row. The want-substring of the "two slots at ONE origin" case in `TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput` changes to the new text | none |
| 22 | s2/fix-22 | `gui/composer_paged.go`: `_, nd, _ := composerPickPageLayout(ctx, th, dims, lead, lines[start:], -1); nshown := max(nd-rowBase, 1); if sel >= start+nshown` | RED row | none |
| 1 | s3/fix-01 | `bip380`: add `xpubMeta`, `xpubDepth` and `xpubChildNumber`, set from `xpub.Depth()` and `xpub.ChildIndex()` at parse. `ExtendedKey` uses them instead of `uint8(len(k.DerivationPath))` | RED row. The bip380 DeepEqual expectations gain the new fields (S3-Q8) | none |
| 6a | s3/fix-06a | `bip380`: `if threshold < 1 \|\| threshold > len(keys) { return nil, fmt.Errorf("bip380: multikey threshold %d out of range for %d keys", threshold, len(keys)) }` | RED row | none |
| 6c | s3/fix-06c | `bip380`: `if r.Type == Singlesig && !r.Script.Singlesig() \|\| r.Type == SortedMulti && r.Script.Singlesig() { return nil, fmt.Errorf("bip380: %v does not take a %v expression", r.Script, r.Type) }` | RED row | none |
| 6b, 50a, 50b, 50c | s3/fix-06b-50 | `nonstandard/parse.go`: new errors `"bluewallet: missing Policy header"`, `"bluewallet: missing Format header"`, `"bluewallet: no keys"`, `"bluewallet: invalid threshold %d of %d"`. After the parse, `desc.Keys[i].DerivationPath = path` for every key (50c) | 4 RED rows. `TestDescriptorSeamDeviceColumn` rows `bluewallet/no-format-header` and `bluewallet/zero-cosigner-lines` now refuse: regenerate the vectors in me (`scripts/descriptor-seam-vectors/`: gen.py, goprobe, rsprobe, refusal_rows.json), copy them byte-identically to the fork's `nonstandard/testdata/`, and re-pin the sha256 (currently `ed3706e2...`) in `crates/me-cli/tests/descriptor_seam.rs` and the fork's `nonstandard/descriptor_seam_test.go` | `SPEC_descriptor_input.md` §4.2: Go parser defects 1-3 are now fixed on the device, so the device refuses these shapes like me does, and the "narrowed" covers note is dropped for those rows. me lands first, and the fork's vendored copy follows in the same fix round |
| 11 | s3/fix-11 | `codex32/codex32.go`: `if len(res) < 6+checkLen { return nil, errInvalidLength }` | RED row | none |
| 7 | s4/fix-07 | `picobin/picobin.go:328`: `eidx+8` becomes `eidx+i*12+8` | RED row | none |
| 13 | s4/fix-13 | `driver/otp/otp.go:253`: `switch idx` becomes `switch uint8(i)` | RED row | none |
| 14 | s4/fix-14 | `otp.go:185`: `flags\|(0b1<<idx) == 0` becomes `flags&(0b1<<idx) == 0` | RED row | none |
| 30c | s5/fix-30c | `cmd/vectorfont`: on a repeated `S`, reflect about the new pen and the previous curve's second control point, as the patch does | RED row | Regenerate the committed font output with vectorfont and diff it. Any glyph that changes is reviewed as a font change in the same PR |
| 30d | s5/fix-30d, extended | `cmd/vectorfont`: implicit repeated relative coordinates (`l`, `h`, `v` as in the patch) are offset from the current pen. **Extend to relative `c`**, which has the same stale-pen fault and is not in fix-30d. Add a subtest `"m0,0 c1,0 1,0 1,0 1,0 1,0 1,0"` that ends at {2 0} | RED row plus the new `c` subtest (RED first) | same font regeneration diff as 30c |
| 49 | s5/fix-49 | `cmd/vectorfont` optimizer: keep the backtrack point, so the stale pen is not used as the start of the next segment | RED row | same font regeneration diff |

Unresolved 39 is owned by this fix round. It needs a host seam or a device test before the round closes.

## Deferred (FOLLOWUPS `deferred-minor`; RED rows get `t.Skip("deferred-minor: <slug>")`)

| # | grade | slug | entry |
|---|---|---|---|
| 9 | M | sh-nfc-scanner-overflow-sticky | The NFC scanner keeps the overflow error after the oversized tag ends; clear on EOF or error (fix-09). |
| 33 | M | sh-multisig-verify-pp-back | Back on the verify-flow passphrase keyboard goes forward to "not checked yet"; step back (fix-33). |
| 42 | M | sh-build-which-md1-back | Back on "Which md1?" is taken as Full policy; abort the flow (fix-42). |
| 48 | M | sh-composer-resume-back-seat | Back on the first resumed slot releases @1's seat; keep it (fix-48). |
| 5 | M | sh-md1-gather-index-past-total | A chunk index past the total is accepted; classify as gatherIgnored (fix-05). |
| 23 | N | sh-jobtime-60-min | Job time prints "60 min" or "2 h 0 min"; carry into hours and print "2 h" (fix-23). |
| 24 | N | sh-truncate-doc | The truncateForDisplay doc says at most n; the contract is n plus the marker; fix the doc (fix-24). |
| 45 | M | sh-legacy-keyless-path-at-creation | A legacy keyless first path is refused at finish, not at creation (fix-45). |
| 51 | M | sh-payload-merge-same-txid | Signed and unsigned copies of one txid count as two candidates; merge on txid (fix-51). |
| 10a-d | M | sh-ur-fountain-bounds | The UR fountain decoder panics or OOMs on seqLen 0, a length mismatch, a huge seqLen or seqNum 0; latent, becomes I when wired (fix-10). |
| 12 | M | sh-bip39-valid-bounds | bip39 `Valid` panics on 0 or 27 words (fix-12). |
| 34 | M | sh-tapleaf-pkh-as-pk | TapLeavesChunks describes a pkh leaf as pk; latent, becomes I when wired (fix-34). |
| 35 | M | sh-parsepath-unhardened-overflow | ParsePath wraps an unhardened index of 2^31 or more (fix-35). |
| 36 | M | sh-ur-hdkey-index-range | A UR HD key index of 2^31 or more wraps; latent, becomes I when wired (fix-36). |
| 37 | M | sh-ur-range-shape | The UR encoder's range shape is refused by its own parser; latent (fix-37). |
| 38 | M | sh-ur-script-type-mismatch | UR accepts wpkh or pkh(sortedmulti); latent, becomes I when wired (fix-38). |
| 47 | M | sh-codex32-newseed-length | NewSeed emits lengths that New refuses; add a library check behind F-116 (fix-47). |
| 52 | N | sh-md1-single-chunk-error | A lone md1 chunk reports "not supported" instead of "chunk set incomplete" (fix-52). |
| 8 | M | sh-type2-large-tag | A Type 2 read stops at 848 bytes with a clean EOF; return an error or read it all (fix-08). |
| 18 | M | sh-ndef-empty-record | Minimal or zero-length Text/URI records are mis-sized (fix-18). |
| 19 | M | sh-uf2-payload-size | A uf2 payload size of 477..479 panics; refuse (fix-19). |
| 20 | M | sh-uf2-start-addr-zero | A uf2 at address 0 is reported as non-contiguous (fix-20). |
| 25 | N | sh-alpha4-wide-span | alpha4.New panics on very wide bounds (fix-25). |
| 26 | N | sh-bspline-nil | InterpolatePoints(nil) panics (fix-26). |
| 27a | M | sh-golden-roundtrip | The golden test encoder does not round-trip (fix-27a). |
| 27b | N | sh-golden-zero-count | The golden decoder accepts a zero-count header (fix-27b). |
| 27c | N | sh-golden-svg-aspect | The golden SVG width is 0 from integer aspect math (fix-27c). |
| 28 | M | sh-biptool-short-seed | biptool derive -path m bip39 -words 12 panics (fix-28). |
| 29 | M | sh-assets-palette-256 | The assets tool accepts 256 colours (fix-29). |
| 30a | M | sh-vectorfont-first-line | vectorfont panics on a path starting with a line (fix-30a). |
| 30b | N | sh-vectorfont-missing-input | vectorfont reports EOF for a missing input file (fix-30b). |
| 41 | N | sh-bezier-degenerate-sample | A degenerate bezier samples the origin (fix-41). |
| 43 | N | sh-bounds-empty-inverted | Bounds.Empty is wrong for an inverted y (fix-43). |
| 46 | N | sh-glyphtrace-short-channel | glyphtrace ranks a short channel worst (fix-46). |
| (S1-Q2) | N | sh-supply-reask | Back in supply multisig could re-ask the passphrase question instead of returning to the menu. |
| 15, 16, 17, 44 | n/a | sh-needs-seam-tinygo-drivers | pio, st25r3916 and clrc663 reports are UNRESOLVED and need a host seam or a device test (owned by the device-test work). |
| S3-Q3 | open | sh-bc-ur-door | Architect: delete bc/* or keep it for a future UR door. |

Counts: C 0 · I 22 (2, 3, 4, 31a, 31b, 32, 21, 22, 1, 6a, 6b, 6c, 11, 50a, 50b, 50c, 7, 13, 14, 30c, 30d, 49) · M 26 (9, 33, 42, 48, 5, 45, 51, 10a, 10b, 10c, 10d, 12, 34, 35, 36, 37, 38, 47, 8, 18, 19, 20, 27a, 28, 29, 30a) · N 11 (23, 24, 52, 25, 26, 27b, 27c, 30b, 41, 43, 46) · FP 1 (40) · UNRESOLVED 5 (15, 16, 17, 39, 44) · open product questions 1 (S3-Q3, architect)
