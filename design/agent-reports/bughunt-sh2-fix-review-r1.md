**⚠ 7 Important findings (0 Critical): this FIX doc is not build-ready.**

# FIX review r1: sh2 bug-hunt Importants (`design/FIX_bughunt_sh2.md` @ a064a199)

Reviewer: Opus, effort medium (independent; author of the doc != reviewer). Scratch: `/scratch/cache/bughunt/sh2/fix-review-r1/`.

## Findings (appended as found)

### I-1 (Important): fix 39's method-value OTP vars turn the fork's `gui` positive control red
- **Location:** FIX doc §2.14 (fix 39, the package-level function vars for `otp.WriteWhiteLabelAddr`, `otp.WriteWhiteLabelString` and `otp.AddBootKey` in `cmd/controller/lockboot_values.go`); §7 line 446 ("No other fork test fails on the fixed copies").
- **Failure (run):** on the combined fixed clone (`/scratch/cache/bughunt/sh2/fix-review-r1/fork`, abdcb18 + triage s1..s5 + fx1-fx5 + s39), with the vars written as method values (`otpAddBootKey = otp.AddBootKey`, as §2.14 says), `go test -count=1 -run TestRefugium ./gui/` FAILS:
  `refugium_build_test.go:181: POSITIVE CONTROL: no default-build file calls AddBootKey, so its absence from the Refugium build proves nothing`. It fails the same way for `WriteWhiteLabelAddr` and `WriteWhiteLabelString`. The test counts only CallExprs that name the function, and a method value is not a call. So the builder's `go test ./...` gate goes red, and §7's claim is false. The s39 probe ran only `./cmd/controller` (vet and build), not `./gui`.
- **Fix text (run; gui test `ok`, row 39 PASS, `go vet ./cmd/controller && go build ./...` OK):** in §2.14, replace the function-var text with: "Declare in `cmd/controller/lockboot_values.go` (untagged in the default build): `var ( otpWriteWhiteLabelAddr = func(row uint16) error { return otp.WriteWhiteLabelAddr(row) }; otpWriteWhiteLabelString = func(idx uint8, s string) error { return otp.WriteWhiteLabelString(idx, s) }; otpAddBootKey = func(hash []byte) (int, error) { return otp.AddBootKey(hash) } )`. They are func literals, not method values: `gui`'s `TestRefugiumFileSetsCarryNoForbiddenLiteralOrCall` positive control needs a call expression naming each OTP writer in the default file set." In §7, after "No other fork test fails", add: "(with §2.14's func-literal vars; method values fail the `gui` Refugium positive control)".

### I-2 (Important): §8 item 1 regenerates the seam corpus before the fork fixes exist, with no command
- **Location:** FIX doc §8 item 1 ("me side of 6b/50 first: apply `FIX_bughunt_sh2-s3-me.diff`, regenerate, sha256 must be `34093bfd…674b`") and item 4 ("Apply the fork fixes in §3").
- **Failure (run):** the generator measures the fork that `scripts/descriptor-seam-vectors/goprobe/go.mod`'s `replace seedhammer.com => …` names. Followed in the printed order (item 1 before item 4), the fork has no 06a/06b/06c/50 fix yet; I ran `gen.py` with the s3-me diff applied against the unfixed fork copy: `sha256 ae067f5a7858fb73c9f99c79dbcb5d165587637cdb7a4db5d1c9871403c253f5` (`/scratch/cache/bughunt/sh2/fix-review-r1/gen-unfixed.log`), not `34093bfd…`. The item also never says which command regenerates, or that the `replace` target must be moved to the fixed fork, so the builder must decide both.
- **Fix text (run; output `wrote ../../crates/me-cli/testdata/descriptor_seam_vectors.json: 84550 bytes, 74 rows, sha256 34093bfd3aa8f303670098838ff6fe76c7cb7485199338088f795d9e15b0674b`):** replace §8 item 1 with: "1. **Fix the fork first (§3: fix-06a, fix-06c in `bip380/bip380.go`; fix-06b/50 in `nonstandard/parse.go`), then the me side of 6b/50:** apply `FIX_bughunt_sh2-s3-me.diff`; set the last line of `scripts/descriptor-seam-vectors/goprobe/go.mod` to `replace seedhammer.com => <absolute path of the fixed fork worktree>`; then `cd scripts/descriptor-seam-vectors && (cd rsprobe && cargo build --offline) && python3 gen.py ../../crates/me-cli/testdata/descriptor_seam_vectors.json`. Its last `wrote` line must end `sha256 34093bfd3aa8f303670098838ff6fe76c7cb7485199338088f795d9e15b0674b` (today `ed3706e2…` in `crates/me-cli/tests/descriptor_seam.rs:46`). Restore the `replace` line before committing." and in item 4 strike "Apply the fork fixes in §3 and" (leaving "Apply the §7 test expectations").

### I-3 (Important): §7 misses two fork tests that fail, gives one row a builder cannot write, and its "no other fork test fails" is false
- **Location:** FIX doc §7 table and its closing sentence ("No other fork test fails on the fixed copies (`go test -count=1 ./...` per copy; s4 all ok).").
- **Failure (run):** with every fix applied to one fork copy (`go test -count=1 ./...`, `/scratch/cache/bughunt/sh2/fix-review-r1/fork-full.log`), besides the §6.2 rows and the §7 rows, two tests fail:
  1. `gui` `TestScan/Descriptor` (`gui/scan_test.go`): after fix-01 the scanned key carries `xpubMeta=true, xpubDepth=4, xpubChildNumber=0x80000002`, which the test's struct literal (package `gui`) cannot set, so `reflect.DeepEqual` fails.
  2. `sysw` `TestTitledZeroKeyDescriptorClassifiesUnknown` (`sysw/descriptor_test.go`, helper `titledZeroKey`): after fix-06b/50 `nonstandard.OutputDescriptor([]byte("Name: my wallet"))` refuses, and the helper `t.Fatal`s.
  Also the row `nonstandard · TestDescriptors … same new fields (fix-01)` cannot be written: `xpubMeta`, `xpubDepth`, `xpubChildNumber` are unexported fields of `bip380.Key`, and `parse_test.go` is package `nonstandard`. (With I-4's fix this test passes unchanged.) The "combined copy" is the case a builder actually reaches; the doc checked per-slice copies.
- **Fix text (run; `gui` TestScan 9/9 subtests ok, `sysw` ok, nonstandard ok):**
  - Delete the §7 row `nonstandard · TestDescriptors`.
  - Add row: "`gui` · `TestScan/Descriptor` (`gui/scan_test.go`) | DeepEqual on the scanned `*bip380.Descriptor` | insert, immediately before the line `if want := test.Content; !reflect.DeepEqual(got, want) {`: `if gd, ok := got.(*bip380.Descriptor); ok { if wd, ok := test.Content.(*bip380.Descriptor); ok { if gd.Title != wd.Title || gd.Encode() != wd.Encode() { t.Errorf(\"scanner decoded\\n%s\\nexpected\\n%s\", gd.Encode(), wd.Encode()) }; break } }` (fix-01: compare the re-encoded descriptor; the literal cannot set the unexported xpub metadata)".
  - Add row: "`sysw` · `TestTitledZeroKeyDescriptorClassifiesUnknown` (helper `titledZeroKey`, `sysw/descriptor_test.go`) | builds the descriptor via `nonstandard.OutputDescriptor([]byte(\"Name: my wallet\"))` | replace the helper body after `t.Helper()` with: `if _, err := nonstandard.OutputDescriptor([]byte(\"Name: my wallet\")); err == nil { t.Fatal(\"fix-06b/50: the BlueWallet branch must refuse a titled zero-key file\") }; return &bip380.Descriptor{Title: \"my wallet\", Type: bip380.SortedMulti}`; change the test's comment \"ACCEPTS it\" to \"refused it before fix-06b/50; the classifier must still fail closed on a hand-built titled zero-key descriptor\"". (`Type: bip380.SortedMulti` is needed: with the zero-value type `thresholdOK` returns true and the test's second conjunct fails.)
  - Replace the closing sentence with: "No other fork test fails with all fixes applied to one copy (`go test -count=1 ./...`)."
  - Planted (tests can fail): removing the four 06b/50 guards makes the new sysw helper fail with `descriptor_test.go:53: fix-06b/50: the BlueWallet branch must refuse a titled zero-key file`. The new TestScan comparison only fixes the false failure: with `ExtendedKey`'s meta branch disabled it still passes (plant `01-depth-scan`), because the scan fixture's path depth equals its xpub depth. Rows 01 and 01bw (I-4) are what catch a fix-01 revert.

### I-4 (Important): fix-01 does not reach the BlueWallet parser, so bug 1 survives on the device's BlueWallet path
- **Location:** FIX doc §2.7 (fix-01: "`ParseKey` sets them") and §3; code `nonstandard/parse.go` `parseBlueWalletDescriptor`, the `desc.Keys = append(desc.Keys, bip380.Key{…})` literal (~:150).
- **Failure (run):** the BlueWallet branch builds `bip380.Key` by hand and never calls `ParseKey`, so `xpubMeta` stays false and `ExtendedKey` still re-derives depth from `len(DerivationPath)`. Scratch test on the doc-state fork: the file `Name: sh / Policy: 1 of 1 / Format: P2WSH / 5A0804E3: xpub6F148LnjUhGrHfEN6Pa8VkwF8L6FJqYALxAkuHfacfVhMLVY4MRuUVMxr9pguAv67DHx1YFxqoKN8s4QfZtD9sR2xRCffTqi9E8FiFLAYk8` (no Derivation header) re-encodes to `xpub67UWhCiWHp3tbCmmz451WxdawVs8q41MKcrVVJgExxVv9dbcnQD5pjMQQ4vGxVRw1xLCuMkv8QKbE95B1uKd4znv4GWkFZbUGZ4abppQaji`; with `Derivation: m/48'/0'/0'` it re-encodes to `xpub6D7vGZGiGPbrLnwwzUbCAZ2ySc4F8KkGPyXaYNBvP2C3kH6vLcxEwCMyY6qgBqUmti6cHFwDkoqiNBeysNsKhs1gevR74nxQeNvLe8neh52`. The device shows and engraves that xpub: a wrong backup string, the unmet guarantee fix-01 states ("a parsed key re-encodes to the same xpub string"). fix-06b/50's new `desc.Keys[i].DerivationPath = path` loop makes a late header change depth too.
- **Fix text (run; with it the scratch round-trip passes for all three cases, `nonstandard` TestDescriptors passes UNCHANGED, rows 06b/50a/50b/50c and TestDescriptorSeam* pass, and the regenerated seam corpus is still `sha256 34093bfd…674b`, `/scratch/cache/bughunt/sh2/fix-review-r1/gen-parsekey.log`):** add to §2.7: "Also in `nonstandard/parse.go` `parseBlueWalletDescriptor`: change `pub, err := xpub.ECPubKey()` to `_, err = xpub.ECPubKey()` and `network, err := bip32.NetworkFor(xpub)` to `_, err = bip32.NetworkFor(xpub)`, and replace the `desc.Keys = append(desc.Keys, bip380.Key{ … })` statement with `k, err := bip380.ParseKey(path, []byte(val)); if err != nil { return nil, fmt.Errorf(\"bluewallet: invalid xpub: %q\", val) }; k.MasterFingerprint = binary.BigEndian.Uint32(fp); desc.Keys = append(desc.Keys, k)`." Add a §6.1 RED row (run both ways): `nonstandard/bughunt_sh2_01bw_test.go` · `TestBughuntSh2_01_BlueWalletRoundTrip`: for each `deriv` in `""`, `"Derivation: m/48'/0'/0'\n"`, `"Derivation: m/48'/0'/0'/2'\n"`, `OutputDescriptor([]byte("Name: sh\nPolicy: 1 of 1\n" + deriv + "Format: P2WSH\n\n5A0804E3: " + x))` (x = the xpub above) must succeed and `d.Keys[0].String()` must contain x, else `t.Errorf("fix-01: BlueWallet key with %q re-encodes as %s, want %s", deriv, got, x)`. RED on the doc-as-written fork: `bughunt_sh2_01bw_test.go:16: fix-01: BlueWallet key with "" re-encodes as xpub67UWhCiWHp3t…` (and the `m/48'/0'/0'` case as `xpub6D7vGZGi…`); GREEN with the swap: `ok seedhammer.com/nonstandard`. Run: `go test -count=1 -run '^TestBughuntSh2_01_BlueWalletRoundTrip$' ./nonstandard/`.

### I-5 (Important): fix-21 does not say an earlier unseated slot's default counts as taken, so a correct reading refuses honest builds with 2+ unseated slots
- **Location:** FIX doc §2.5 lines 134-136 ("at the lowest free account (taken origins keyed by `composerOriginKey`)"); row 21 (§6.1 line 369) covers only one unseated slot.
- **Failure (run):** md `resolveOrigins` (md/compose.go:818-847) gives each unseated slot the lowest account that no other slot holds, "declared or filled earlier". The text reads equally well as "taken = the seated slots' origins". A builder who seeds `takenOrigins` from seated slots only and does not mark each unseated slot's assigned default gives slot @2 the same account as @1. The self-check then refuses EVERY honest build with two or more unseated slots. `composerSelfCheck` runs in every `composerConsentFlow` (gui/composer_selfcheck.go:257), so this hits the common key-less and partially seated templates and blocks a correct card. Row 21 stays GREEN under this plant (`21a-no-mark`, rc=0), so no §6.1 row catches it. Scratch row `gui/bughunt_sh2_21b_test.go` (body below): PASS on the fixed code. With the marking line dropped it fails: `bughunt_sh2_21b_test.go:35: the self-check REFUSED an honest build with two unseated slots: self-check: unseated slot @2 declares m/48h/0h/2h/2h, not the default m/48'/0'/0'/2'`.
- **Fix text:** in §2.5, replace "(taken origins keyed by `composerOriginKey`)" with "(taken origins keyed by `composerOriginKey`: every seated slot's origin, plus each earlier unseated slot's default once it is assigned, as md `resolveOrigins` does)". Add this §6.1 row after row 21: "`gui` · `TestBughuntSh221SelfCheckTwoUnseatedSlots` (new, `gui/bughunt_sh2_21b_test.go`) | PASS | guards fix-21's earlier-slot marking; with it dropped: :35 `the self-check REFUSED an honest build with two unseated slots: self-check: unseated slot @2 declares m/48h/0h/2h/2h, not the default m/48'/0'/0'/2'`". Add the file to §8 with this exact body:

```go
package gui

import (
	"testing"

	"seedhammer.com/md"
)

// Bug 21 (second case): with TWO unseated slots the codec gives them the two
// lowest free accounts (§4f, md resolveOrigins: "no other slot (declared or
// filled earlier) holds"), so the self-check must count an earlier unseated
// slot's origin as taken, or it refuses an honest build.
func TestBughuntSh221SelfCheckTwoUnseatedSlots(t *testing.T) {
	list := md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{Keys: &md.KeySet{K: 2, N: 3, Sorted: true}},
	}}
	st := &composerState{list: list, reg: &seedRegistry{}}
	st.sources = []composerSource{{kind: composerSourceKey, seedID: -1}}
	fp := [4]byte{0x73, 0xc5, 0xda, 0x00}
	origin := composerTestOrigin(2, 1)
	st.assigned = []composerAssignment{
		{src: 0, account: 1, origin: origin, fingerprint: fp, fpPresent: true},
		{src: -1}, {src: -1},
	}
	declared := []*md.SlotOrigin{{Origin: origin, Fingerprint: fp, FpPresent: true}, nil, nil}
	c, err := md.ComposeWith(list, declared)
	if err != nil {
		t.Fatalf("md.ComposeWith: %v", err)
	}
	chunks, err := c.Chunks()
	if err != nil {
		t.Fatal(err)
	}
	if err := composerSelfCheck(st, chunks); err != nil {
		t.Errorf("the self-check REFUSED an honest build with two unseated slots: %v", err)
	}
}
```

### I-6 (Important): the doc does not name the fork build base or the slice merge
- **Location:** FIX doc §8, before item 1 (lines 26-31 name the slice heads only).
- **Failure:** the §6.1 RED rows live on five branches, `origin/bughunt/triage-s1..s5`. The doc never says which branch to build on or that the five must be merged. A builder working from `origin/bughunt/triage` or from one slice sees rows missing, so "turn every §6.1 row GREEN" cannot be met. Which base to use is left to the builder to decide.
- **Fix text (run as printed in a fresh clone of the fork: merges cleanly, head a148b5c, 17 `bughunt_sh2_*_test.go` files):** insert as §8 item 0: "0. In the fork: `git switch -c bughunt/sh2-fix origin/bughunt/triage && for s in 1 2 3 4 5; do git merge --no-edit origin/bughunt/triage-s$s; done` (merges cleanly; 17 `bughunt_sh2_*_test.go` files)."

### I-7 (Important): the row-39 test body is nowhere in the doc
- **Location:** FIX doc §8 item 5 ("add row `bughunt_sh2_39_test.go`") and §6.1 line 385 (its `:22`/`:25` pins).
- **Failure:** the body exists only in `/scratch/cache/bughunt/sh2/fixdoc-probe/s39.diff` lines 5-31. The doc calls that file "not part of the build". A builder working from the doc alone must write the test, and the §6.1 line pins depend on its exact layout. Which body to write is left to the builder to decide. This interacts with I-1: the same file's OTP-var approach is what turns the `gui` control red.
- **Fix text (run: the output's sha256 matches the review fork's copy):** in §8 item 5, replace the bare "add row" with "write the row with `sed -n '5,31p' /scratch/cache/bughunt/sh2/fixdoc-probe/s39.diff | sed 's/^+//' > cmd/controller/bughunt_sh2_39_test.go` (sha256 `61160852f25428c2d7802ad1998cac6bde08ffa0ddd37991582def46a3a55311`)".

## Minors
- M-1: no row isolates a single 50a guard. Planting no-keys alone or policy alone leaves row 50a GREEN; only removing all four guards fails it (`:40 Name-only file accepted`). The device fails closed only if every guard holds. Unusual conditions: (1) a builder drops exactly one guard; (2) a BlueWallet file that reaches only that guard.
- M-2: §8 item 6 says to update a comment but gives no new comment text. This is comment-only and changes no behaviour.

## Nits
- N-1: the fix-21 error string mixes path notations: the declared origin prints as `m/48h/0h/2h/2h` and the default as `m/48'/0'/0'/2'`.

## Planted faults (fork review clone, all fixes applied; `plants/run.py`, `plants/out*.txt`)
| plant | row | result |
|---|---|---|
| 21b-no-check | 21 | FAIL as stated |
| 21a-no-mark (earlier unseated not marked) | 21 | survives; caught by new 21b row (I-5) |
| 02-back, 22-wrap | 02, 22 | FAIL as stated |
| 01-depth | 01 | FAIL |
| 01-depth-scan | TestScan | survives (I-3) |
| 01-depth-bw | 01bw (I-4 scratch) | FAIL |
| 06a, 06b, 06c | 06a/06b/06c | FAIL as stated |
| 50a-nokeys, 50a-policy | 50a | survive (M-1) |
| 50b-format, 50c-path | 50b, 50c | FAIL as stated |
| 11, 07, 14, 13 | 11, 07, 14, 13 | FAIL as stated |
| 30d-l/h/v/c/s | 30d | FAIL as stated |
| 30d-s-reflect | 30d | survives; row 30c catches it (`{0 0}, want {3 3}`) |
| 49 | 49 | FAIL as stated |
| 39-continue | 39 | FAIL as stated |

Device-path check (d): with the fixes applied, the seam corpus (except the stated sha flip), `nonstandard` TestDescriptors and the self-check accept every honest case I ran. Each refusal names its problem and builds no card.

## Design line (ungraded)
Coldcard's `from_simple_text` treats Format (default P2SH) and Policy (default N-of-N) as optional. The fixed device refuses both, so it fails closed but rejects some files that are valid for Coldcard.

r1: 0C/7I/2M/1N
