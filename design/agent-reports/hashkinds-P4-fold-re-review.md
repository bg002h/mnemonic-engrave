# Fold re-review — hashkinds phase 4, 10 commits (`285f7a9..8225081`, branch `hashkinds-p4`)

**VERDICT: NOT GREEN — 0 Critical / 2 Important / 5 Minor.** One original Important
(adversarial I-4) is only PARTIALLY fixed. The fold also introduces one new Important
(a false file-count claim in commit `cf251e5`'s own message) and one new Minor (a stale
doc comment in `engrave/h6_qr_test.go` contradicting the import it sits above). Every
other Important and Minor from the four reports is FIXED, DELIBERATELY DECLINED (via
honest F-540/F-541 filings), or was already out of scope (harness-methodology notes).

Repo: `/scratch/code/shibboleth/seedhammer`, branch `hashkinds-p4`, tip `8225081` (clean
tree). Toolchain: `/scratch/code/shibboleth/.toolchain/go/bin`, go1.26.7. All test runs
below used `-count=1` and `-v`, and every named test's RUN lines were confirmed present
(no silent zero-match `-run`). `go build ./...`, `gofmt -l .` (five-file baseline) and
`node --check` on both touched `.js` files were independently re-confirmed clean, matching
what was already machine-verified at the tip (not reported as findings, per brief).

## (a) Original findings — fixed / declined / not fixed

| Report | ID | Claim | Status | Evidence |
|---|---|---|---|---|
| journey-walk | I-1 | Reconcile screen names `--kind` but not `--method`; host defaults method to hardened | **FIXED** | `composer_copy.go:743-772` now emits `--kind <k> --method <m>`; `TestReconcileScreenInstructionReproducesTheDigestItShows` (8 subtests: 2 methods × 4 kinds) all PASS. Verified clap `Method` enum renders `sha256`/`hardened` (kebab-case default), matching `hashlockMethod.String()`. |
| journey-walk | I-2 | Hex pad names no kind | **FIXED** | `composerHexEntry` title is now `kind.Token()+" hash"` (`composer_hash.go`). |
| journey-walk | I-3 | Pad silently truncates past kind width, checkmark lit | **FIXED** | `over` flag added; count line appends `" - extra ignored"` while over-length; clears correctly once short again (verified by reading the clamp/reset logic: `> want` sets it, `< want` clears it, `== want` holds — no stuck-true path). |
| journey-walk | I-4 | §7.5 `Which hash?` row carries no kind; ripemd160 payload record drawn identically to sha256 | **FIXED** | `composerHashRow`/`composerHashPreimageRow`/`composerHashPhraseRow` now lead with `digest.Kind().Token()`. `TestWhichHashRowsDrawOnOneLine` measures all four kinds with a new 20px margin gate — PASS. |
| journey-walk | I-5 (= adversarial I-2) | §13.3 census row prints bare `sha256` (method) with no kind | **FIXED** | `composerCopyPreimagePlateRow` takes a `kind md.HashKind` param, prints `path %d  %s %s  %s` (kind first). `TestComposerCensusReportsEveryDecisionAndCutsNothingItself` PASS. (The two "not-cut" forms, `composerCopyPreimageNotOnAnyPath`/`Declined`, remain kind-less, but they never printed a method token either — no ambiguous bare-token collision exists there, so this is not a gap in the same class.) |
| journey-walk | I-6 (= adversarial I-1) | §8's 20-byte warning implemented nowhere | **FIXED** | New `composerWarnTwentyByteUnseen`, gated on `DigestLen()==20 && !held`, inserted at exactly the two non-derived sites (band-1 payload digest, typed-hex arm) and nowhere else — confirmed only 2 call sites via grep, and confirmed the three device-derived routes (phrase, payload-phrase, preimage-record) never call it because material is held before assignment. `TestTwentyByteWarningFiresExactlyWhereSpecEightSaysIt` (6 subtests, matches claimed "6 rows") PASS. |
| journey-walk | M-1 | Method pick shows bare `SHA-256`, two screens after kind screen's `sha256` | **DELIBERATELY DECLINED** | Filed as **F-540** in `mnemonic-engrave/design/FOLLOWUPS.md:17465`, owning phase "post-release UX", honestly recaps the walk's own "documentation only" classification and reasoning (title/lead say "method"; brainwallet confirm catches a mis-pick with phrase intact). |
| journey-walk | M-2 (= adversarial I-5) | Back from pad reopens kind screen scrolled/selected wrong; one page press silently moves selection to sha256 | **FIXED** | `composerPickScreenFrom` now starts at `start:=0` with a post-layout re-home guarded by `homed` (fires at most once, confirmed no spin: the `continue` occurs before `ctx.Frame` is ever called, and `homed=true` is set before the continue so it cannot re-fire). Verified the other call site (`composerPickScreen`, always `initial=0`) never triggers the re-home path (0 ≥ 0+shown is always false), so unrelated pickers (Spend paths, Seat keys, etc.) are behaviorally unchanged. `TestComposerHashKindScreenDrawsAllFourRowsFromEveryStart` (opens at all 4 indices) PASS. |
| journey-walk | M-3 | Back from pad silently discards typed digest | **DELIBERATELY DECLINED** | Filed as **F-541** (`FOLLOWUPS.md:17482`), owning phase "post-release UX", honestly notes its own severity dropped now that the pad names the kind (fixed above), leaving only the plain "mistype near the end, lose it" residual case. |
| journey-walk | M-4 | Kind rows use `%-10s` padding on a proportional face (ragged) | **FIXED** | `composerHashKindRow` now uses `"%s  %d hex"` (two literal spaces), commit `8225081`. |
| journey-walk | N-2 | §12 item 7 never executed | **FIXED** | New `TestReconcileScreenInstructionReproducesTheDigestItShows`, parses flags out of the drawn string (not the variables), 8/8 PASS. |
| adversarial | I-3 (= claim-check #5) | `shots_composer.js:667` still asserts retired `"Type 64 hex"` | **FIXED** | Now `must(hashRows, "Type a digest", ...)`; confirmed no remaining occurrence of the retired string in the file; `node --check` clean. |
| adversarial | I-4 | §12 item 2: seam can't distinguish kinds (sha256/hash256 both 64 hex); no walk composes non-sha256 | **PARTIALLY FIXED** | `shComposerPathHashes` now returns `{kind, digest}` (commit `44b1085`); every consumer updated (grepped both `.js` and `.go` repo-wide — only `composer_js.go` and `walk_hashlock_phrase.js` reference it, both consistently). **But** the walk itself still only calls `pickKind(0)` (confirmed: single occurrence in `walk_hashlock_phrase.js`) — it composes sha256 only. The fold's own message is candid about this ("this journey composes sha256, so the assertion is not exciting today"). §12 item 2's full acceptance ("an emulator walk composes a **non-sha256** hashlock... and asserts... that kind") remains unmet; only the *mechanism* that would let such a walk fail correctly is now in place. |
| adversarial | I-6 | `h6_qr_test.go`'s QR text copy falsified; module-budget gate samples wrong (194 vs 210 byte) content | **FIXED** | `h6HardenedMethodLine`/`h6SHA256MethodLine` now derive from `hashlock.MethodLine`; budget paths (`h6QRText`) route through `h6QRTextKind(..., md.KindRipemd160, ...)` — confirmed ripemd160 is genuinely the longest token (9 chars vs hash160's 8, hash256's 7, sha256's 6), so it is the correct worst case. Goldens split off into `h6QRTextFrozen` with a frozen literal, explicitly NOT wired to production (correct, avoids golden churn). `TestH6ConstantQRGoldens` and `TestConstantTimeQRBudgetBoundsEveryPayload` both PASS. **See part (b): this fix leaves behind a new stale-comment defect.** |
| adversarial | M-1 (= conformance I-1) | Compose-vector pin stale, three new-kind vectors never vendored | **FIXED** | Re-vendored to 176 files / 36 vectors; arithmetic independently recomputed (32 keyed×5 + 4 unkeyed×4 = 176; 36 names in `composeVectorNames`) — matches exactly. `TestKeyedConformanceAgreesWithRust` now runs 46 subtests (grepped count), including all three new `keyed_compose_preset_hashlock_gated_{hash256,ripemd160,hash160}` — all PASS. **See part (b): the commit's own "before" count (156) is wrong.** |
| adversarial | M-2 | `composerHexEntry` doc comment says "returns a sha256 lock… offers no kind" | **FIXED** | Rewritten in `44b1085` to "IT RETURNS A LOCK OF THE KIND IT WAS GIVEN", explains what changed and why. |
| adversarial | M-3 | `hashlockLockOf` doc comment says "every call site passes md.KindSha256" | **FIXED** | Rewritten in `44b1085` to "THE CALL SITES NOW PASS A CHOSEN KIND". |
| adversarial | M-4 | `ConstantQR`'s capacity comment cites a stale 194-byte worst case / 36-byte headroom (actual: 210 / 20) | **NOT FIXED** | `engrave/engrave.go:504-511` still reads "worst case is 194 … 36 bytes of headroom". Not touched by any of the 10 commits; no follow-up filed (`grep` of FOLLOWUPS.md for "194"/"headroom" empty). Bound still holds numerically (210<231), so non-blocking, but the finding stands unaddressed. |
| adversarial | M-5 (= claim-check #4) | `sysw.HashRecord` has zero callers anywhere | **FIXED** | `gui/composer_fixtures_test.go` now builds `composerTestHashRecord = sysw.HashRecord(composerTestLockOf(md.KindSha256, 0xab))` — confirmed via repo-wide grep, only production def + this one call site. |
| adversarial | M-6 | `TestHashLockIdentityIgnoresPadding` cannot fail on padding (no constructor produces non-zero padding) | **NOT FIXED** | Test body unchanged; still only exercises kind-as-identity and width-refusal, not an actual falsifiable padding case. No follow-up filed. |
| conformance | I-1 | Stale corpus provides zero regression coverage for 3 new kinds | **FIXED** | Same as adversarial M-1 above. |
| conformance | I-2 | Go's `Compose()` builder never exercised with new kinds | **FIXED** | New `md/compose_hashkinds_test.go`: `TestComposeRoundTripsEveryHashKind` (4 subtests) and `TestEveryHashKindTakesItsOwnWireTag`, both PASS — Go twin of Rust's `compose_hashkinds.rs`, as the finding recommended. |
| conformance | M-1 | `ErrHashRecord` sentinel doesn't carry which kind failed | **NOT FIXED** | Sentinel unchanged (`sysw/composer_records.go:50`). Report itself said "no user-visible effect" — non-blocking, not addressed, no follow-up filed. |
| conformance | M-2 | `SPEC_hashlock_kinds.md` §1 gap table says `md compose` CLI has "only a sha256= option" (already false — CLI has all 4) | **NOT FIXED** | `design/SPEC_hashlock_kinds.md:25` still reads "only a `sha256=` option". Report called this "expected drift... worth a closing pass when the cycle wraps" — not addressed by this fold, no follow-up filed. |
| conformance | M-3 | `notahash:` harness row proves nothing (methodology note) | **N/A** | Not a code defect — a caveat about the review's own scratch harness. Nothing to fix. |
| claim-check | #1 | "22 row/kind pairs" — actual is 44 | **FIXED** (corrective disclosure) | Commit `8df2496`'s message states the correction verbatim, matches independently-confirmed structure (11 corpus rows × 4 kind columns = 44). Commit history left intact rather than amended, consistent with this project's established practice (`fa38880` correcting `5fae990` the same way — explicitly not re-flagged per brief). |
| claim-check | #2 | "SIX + THREE = eleven" — arithmetic is 9, not 11 | **FIXED** (corrective disclosure) | Same commit; states the sentence contradicted itself and that the true historical count is not recoverable. |
| claim-check | #3 | "vet ... byte-identical to HEAD" — false, one line number differs | **FIXED** (corrective disclosure) | Same commit; states substance (10 warnings, same files/category) is unaffected, the literal "byte-identical" claim was wrong. |
| claim-check | #4 (= adversarial M-5) | `sysw.HashRecord` zero callers | **FIXED** | See above. |
| claim-check | #5 (= adversarial I-3) | `shots_composer.js` retired label | **FIXED** | See above. |
| claim-check | Minor #6 | `mnemonic-engrave/design/SPEC_wallet_policy_composer.md` §8i blockquote stale | **FIXED** | Companion commit `26049c8c` (mnemonic-engrave repo, same session) splits §8i into entry/consent bodies; `SPEC_wallet_policy_composer.md:780` now reads "The preimage must be a 32-byte value" — confirmed by direct read. |
| claim-check | Minor #7 | `TestComposerCanBuildARipemd160HashlockOnTheDevice`'s mutation comment quotes a message the test never prints | **FIXED** | Code diff in `8df2496`: comment now says the harness times out ("...never returned after 256 frames"), correcting the prior "path holds no hash at all" claim. |
| claim-check | Nit #8 | "go build ./... does not reach cmd/emu" is imprecise | **FIXED** (corrective disclosure) | Corrected in `8df2496`'s message: `cmd/emu` itself builds via its `!js` stub; only the `js`-tagged files inside it are excluded. |

## (b) New defects introduced by the fold

### Important — commit `cf251e5`'s own message states a false "before" file count

`cf251e5`'s message claims: *"Re-vendored at dm 745e0fd0 (clean): 156 -> 176 files, 33 ->
36 vectors."* Verified directly against git:

```
git show cf251e5^:md/testdata/compose_vectors.provenance.json  -> 161 files (not 156)
git show 285f7a9:md/testdata/compose_vectors.provenance.json   -> 161 files (not 156)
git show cf251e5^:md/compose_vectors_pin_test.go               -> asserts len(p.Files) != 161
```

The actual pre-commit count — at both `cf251e5`'s direct parent and at the fold's base
`285f7a9` — was **161**, not 156; the diff's own removed line reads `// 29 keyed vectors
carry five files, 4 unkeyed carry four: 161.` The "vectors: 33 -> 36" half of the claim is
correct (independently confirmed); only the "files: 156" half is wrong. The false "156"
number originates in the go-rust-conformance report itself (which cited "156 files" for
the same commit, also without machine-checking it against the actual local file) — so the
fold did not fabricate the number, but it repeated an already-wrong figure into its own
commit message as a fact about its own diff, without verifying it against the file it was
editing. This is the identical defect class the claim-check lens exists to catch (a
machine-checkable numeric claim in a commit message, false), landing fresh in the very
commit that answers that lens's *other* findings, and echoing this same fold's own
`5fae990`→`fa38880` correction earlier in the same session. The actual code and test
assertion (176 files, 36 vectors) are correct and verified; only the commit message's
"before" number is wrong.

### Minor — `engrave/h6_qr_test.go` gains a stale doc comment contradicting its own imports

Commit `cf251e5` adds `"seedhammer.com/hashlock"` to this file's imports and rewrites
`h6HardenedMethodLine`/`h6SHA256MethodLine` to call `hashlock.MethodLine(...)`, but leaves
the *original* comment block sitting directly above them unedited:

```go
// h6HardenedMethodLine and h6SHA256MethodLine are §8.6's two method lines.
// They are literals HERE because this package cannot import hashlock (which
// imports nothing of engrave, but the plate builder in backup does the
// composition); ...
// DERIVED FROM PRODUCTION, NOT COPIED. These were three literals transcribed
// ...
var (
	h6HardenedMethodLine = hashlock.MethodLine(true)
	h6SHA256MethodLine   = hashlock.MethodLine(false)
)
```

"They are literals HERE because this package cannot import hashlock" is now false on both
counts — they are not literals, and the file's own import block two lines above proves it
can import hashlock. A new paragraph was appended explaining the real derivation, but the
old paragraph was never removed, so the two now contradict each other in the same doc
comment. This is exactly the "comments outlive their conditions" class that a *later*
commit in this same fold (`44b1085`, "three comments stop lying") explicitly hunted down
and fixed three other instances of — but `44b1085` never touched this file, so this fourth
instance (introduced by an earlier commit in the same fold, `cf251e5`) survived. Confirmed
via repo-wide grep: this is the only remaining occurrence of "cannot import hashlock" /
"literals HERE" in the tree. No behavioral impact — comment-only.

## What was not re-derived

Per the brief, `go test ./...` (56/0 FAIL), `gofmt -l .` (five-file baseline), `go vet
./...` (10 pre-existing lines), the wasm build, and `node --check` on both walk files were
treated as already machine-verified and not re-run for confirmation; targeted re-runs
above (`go build ./...`, `gofmt -l .`) incidentally reconfirmed the same clean state on the
current tip as a side effect of other checks, not as a fresh finding. Full runtime
execution of the browser-based emulator walk (as opposed to static/`node --check`
verification of its JS) was not attempted — this was also outside the scope of the four
underlying reports, which used static analysis and scratch Go/Rust harnesses rather than a
live browser run.
