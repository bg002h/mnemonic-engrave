# Review r2: bg002h/seedhammer PR #29, the fold re-review

Reviewer: second (fold) reviewer (Opus 5.5), 2026-11-25.
Scope: `git diff dde21fe..a27b542` (the r1 head to the current head). It has two commits and five files:
- `ea1c3af`: the I-1 fold, touching `gui/wallet_policy.go`, `gui/composer_consent.go` and `gui/no_change_chain_test.go`.
- `a27b542`: the seam-vectors re-vendor, touching `nonstandard/testdata/descriptor_seam_vectors.json` and the SHA pin in `nonstandard/descriptor_seam_test.go`.

I did not edit the clone. All mutation and instrumentation work ran on `git archive` copies in the scratchpad.

**Verdict: 0 Critical, 0 Important, 0 Minor, 2 Nit. I-1 is fixed. The fold loop is closed.**

## 1. Is I-1 fixed?

**Yes, on both consent screens.** Both screens now go through one helper, `addressProofLines` (`gui/wallet_policy.go:384-403`). The Wallet Policy screen uses it at `gui/wallet_policy.go:361`, and the composer at `gui/composer_consent.go:341`.

The helper handles errors like this:
- **No change chain.** `errors.Is(err, address.ErrNoChangeChain)` is checked only on Change index 0. In that case the helper keeps the receive lines, appends `Change: none (no change chain)` and stops.
- **Any other error, on either chain.** It is still a failure. The Wallet Policy screen maps it to "Address derivation failed. / Do not engrave this card." The composer returns it as an error, and `composer_selfcheck.go:266` turns that into the fixed self-check-failed copy, so no developer string reaches the operator.
- **Both sources of the error are covered.** `errors.Is` matches the bare sentinel from the flat route (`address/address.go:25`) and from the complex route (`gui/policy_address.go:205`).

**Can a valid card show a failure, or an invalid card show success?** I found no case of either.
- The sentinel is returned only when `HasChangeChain` is false, which matches Rust's `check_chain_in_range`.
- A receive-chain error, or a change error other than the sentinel, still fails.
- A card for which `policyAddressAt` returns `!ok` still goes to `noAddressLines`, which is unchanged.

**Evidence.** I ran the PR's three no-multipath cards through both screens, using a temporary test in the scratch copy:
- **`wsh(sortedmulti(1,@0/*,@1/*))`, Wallet Policy screen.** It shows Receive 0 and 1 (`bc1qmj9hd2t9…`, `bc1qzhc2al5w…`, which are Rust's addresses as verified in r1), then `Change: none (no change chain)`.
- **`wsh(or_b(...))`, Wallet Policy screen.** The same layout.
- **`tr(@0/*,pk(@1/*))`, both screens.** This card is not in the new test. Both screens show Receive 0 and 1 (`bc1pw599…`, `bc1pp9fw…`, equal to the Rust-pinned values in `TestNoMultipathCardHasNoChangeChain`), then the no-change-chain line. That includes the composer screen, which shows the line instead of ending the consent.

**Did the refactor change any composer output?** No. I wrapped `composerConsentLinesFor` and `walletPolicyConsentLines` in both trees (dde21fe and HEAD) so that every call's input, lines and error were written to a dump. Then I ran the same 140 tests in both trees: every `Test*` in the 19 gui test files that reach those functions or the composer review/self-check. Both runs passed.
- dde21fe made 89 calls, 70 of them composer calls, and all were normal `<0;1>/*` cards with 15 `Change 0` proofs.
- Under HEAD the same 89 calls gave **byte-identical** lines and errors.
- The only extra entries are the two calls from the new test.
- The composer's error text did change: it now adds `: <cause>`, using `%w`. That text is internal and only feeds `showError`'s fixed copy.

**Do the tests fail without the fold?** Yes. I checked with two mutations in the scratch copy:
- **Mutation A: disable the `ErrNoChangeChain` branch.** `TestWalletPolicyConsentNamesAMissingChangeChain` fails on both subtests, and the output is exactly r1's reproduction: "Address derivation failed. / Do not engrave this card."
- **Mutation B: widen the branch to swallow any change-chain error** (`err != nil` in place of `errors.Is`). The test's tail assertion fails: `addressProofLines on a real change error = ([... "Change: none (no change chain)"], <nil>), want a failure naming Change 0`. So the guard against an invalid card showing success is real.

## 2. The vendored seam vectors

- **Byte-identical.** `sha256sum nonstandard/testdata/descriptor_seam_vectors.json` gives `683ccd3d…20e5cd`. `git -C mnemonic-engrave show 30f8e69:crates/me-cli/testdata/descriptor_seam_vectors.json | sha256sum` gives the same value. The pin in `descriptor_seam_test.go:42` matches.
- **Provenance.** The old pin `71bba6f2…` equals engrave at `094979e^`, and `094979e` (E3) is the commit that produces `683ccd3d`. So the re-vendor picks up exactly E3 and nothing else.
- **What changed.** The whole diff is one new entry in the `refusal_rows` label map (`"md1-origin-mismatch": "S6 row 36 …"`). No vector row changed, so `wantRows = 74` and the population counts are unaffected.
- **Is a Go change needed?** No. No Go code decodes `refusal_rows`:
  - `seamRow`/`loadSeamVectors` unmarshal only `vectors`;
  - the only `RefusalRows` symbol in the tree is `hashlock_test.go:143`, which reads a different file;
  - no vector's `covers` names the new row;
  - E3 is a refusal on the host's `--as md1` route, and the device's `sysw` ClassDescriptor port does not apply the md1-path widening (`sysw/descriptor.go:20-22`).
- **Tests.** `go test ./nonstandard/ ./sysw/ ./address/` passes.

## 3. Drafted follow-ups

`scratchpad/followups/seedhammer.md` logs all six r1 items at the r1 grades, in the engrave FOLLOWUPS slug format:

| r1 finding | follow-up slug | grade |
|---|---|---|
| M-1 | `sh-bh1125-address-list-no-change-chain-toggle` | Minor |
| M-2 | `sh-bh1125-check-check-pkk-type-refusal` | Minor |
| N-1 | `sh-bh1125-mk-encode-stale-comment` | Nit |
| N-2 | `sh-bh1125-derived-csid-vs-me-cli-mk-codec-060` | Nit, plus a cross-repo me-cli item |
| N-3 | `sh-bh1125-pkh-tap-leaf-described-as-pk` | Nit |
| N-4 | `sh-bh1125-legacy-sh-pk-no-address` | Nit |

- **Substance.** Every entry carries r1's location, failure scenario and fix direction.
- **N-3.** Its entry correctly cross-references the existing `sh-tapleaf-pkh-as-pk` entry (engrave `design/FOLLOWUPS.md:20892`) and the existing skipped test.
- **M-2.** r1 recommended folding it. Logging it as a Minor follows Brian's rule (only Critical and Important are folded), and the grade is unchanged.

## 4. Anything else changed?

- **No other changes.** The diff holds only the five files above.
- **Format and vet.** `gofmt -l gui nonstandard` lists only the known baseline files (`gui/transaction*.go`). `go vet` shows only the existing `ArtifactDir` go1.26 notices.
- **Tests.** The scoped gui run (`-run 'NoMultipath|NoChangeChain|WalletPolicy|MultisigRestore|AddressList|Composer'`) passes, as do the 140 tests used for the composer comparison above. I did not re-run the full 7-minute gui package. The fold touches only the two consent functions, and every test that reaches them was run.
- **Commits.** Both new commits are signed (`gpgsig` present). Like the r1 commits, they carry no `Signed-off-by:`, which is consistent with the rest of this fork PR.

## Findings

### N-1 (Nit): the composer's no-change-chain branch has no test in the tree

`gui/composer_consent.go:337-346`. The new test drives only `walletPolicyConsentLines`, plus `addressProofLines` directly. The composer path is unreachable today, because the composer always mints `<0;1>/*`; the comment in the code says so. It shares the helper, and I confirmed by hand with the tr card that it renders correctly.

**Failure scenario.** A later change gives the composer its own loop again, or makes it map the helper's error before the sentinel check. The composer would then end consent on a valid no-change-chain card, and no test would go red.

**Suggestion:** add the tr card as a `composerConsentLinesFor` subtest, which is a few lines.

### N-2 (Nit): the M-1 follow-up cites a stale line range, carried over from r1

The follow-up cites `gui/address_polish.go:72-75`. In fact `recompute` starts at line 69 and the `showError` call is at line 77. **Suggestion:** cite `gui/address_polish.go:69-77` (`recompute` / `showError`) when the follow-up is committed to engrave. Nothing in the code is affected.

## Disposition

- I-1 is closed, and both new commits are clean.
- N-1 and N-2 can be logged, or N-2 corrected in place, when the follow-ups file is committed to mnemonic-engrave.
- No further review round is needed: 0 Critical and 0 Important closes the loop.
