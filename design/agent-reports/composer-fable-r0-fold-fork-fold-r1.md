# Composer fable review r0 — fold B (fork), round 1 fold: implementation account

Branch `fable-r0-fold` in `/scratch/code/shibboleth/seedhammer`, worktree
`/scratch/code/shibboleth/.tmp/fold-fork`. Range `0f435048..aabef11`, six
commits. Answers `composer-fable-r0-fold-fork-review.md` (0C/2I/5M/4N).

**Authorship.** The fold implementer (opus) landed the first four commits and
was terminated by the API's weekly rate limit while starting M-4 ("Now M-4 —
the doc-comment theft"); its evidence for those four is in their commit
messages, quoted below. The controller (fable, this session) finished M-4,
M-5, N-1, N-4 inline in the same worktree, declined N-3, and ran the gate.

## Per finding

| finding | commit | evidence |
| --- | --- | --- |
| M-3 vector gate cannot express precedence kinds; re-vendor | `1eb4d4d` | Gate now dispatches on `kind` with per-kind fields; a closing check requires every kind in the file's `precedence` list to have been exercised. Re-vendored from dm `b2c5d693` (md-codec 0.45.0). Two typed errors added (`KeylessUnderTrError{Path}`, `TooManySlotsError{Got,Max}`), `Error()` strings unchanged. |
| I-1 Go orders the cap before the structural refusals; M-2 empty path counted as key-less | `a3d16cd` | RED before (from the gate `1eb4d4d` built): `precedence_too_many_slots` refused with the cap ("paths 5 and 6", want TooManySlots); `precedence_legacy_wrapper_shape` refused with the cap ("paths 2 and 3", want LegacyWrapperShape). GREEN after: 8 cases, 7 refused / 1 admitted, kinds {KeylessUnderTr 1, LegacyWrapperShape 1, NoKeyedPath 1, TooManyKeylessPaths 3, TooManySlots 1}. `TestFableValidatePathListOrdersTheCapLast` pins the four orderings directly; mutation (block moved back above the structural checks) fails 2 vector cases + 2 rows. GUI creation-time guard counts `Keys == nil && Hash != nil` only, and under `sh`/`sh-wsh` the legacy body wins. |
| I-2 `soleMulti` widened; guard vacuous | `f4c484e` | Measured at base/tip: `wsh(and_v(v:older(10), or_d(multi(2,@0,@1,@2), c:pk_k(@3))))` base K=0 N=0 Keys=4, tip K=2 N=3 Keys=4. Fix: the multi's threshold is accepted only when it accounts for EVERY key the branch references (`nkeys == br.Keys`); `TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee` gains the mixed fixture and its clean twin. |
| M-1 testnet mk1 card offered as a source | `c891c29` | RED before: "the composer offers a testnet mk1 card as a source (1): label 73c5da0a m/48h/1h/0h/2h xpub tpubDFH9dgzv…"; "a payload with one testnet and one mainnet card yields 2 sources, want 1". GREEN after. Predicate reads `mk.Card.Network` (the decoded version bytes); empty counts as mainnet (pre-field cards). |
| M-4 two doc-comment thefts; N-4 stale self-check comment | `f222eed` | `TestComposerHelpersDidNotStealADocComment` with `composerSecretCards` and `composerCopySameSeedThreshold` added to `composerDocOwners`: RED at the prior tip on BOTH names ("has NO doc comment"), GREEN after moving `composerCopyMixedLockBases` above the §8g banner, re-binding the §8g doc (a blank line had crept between doc and func), giving `composerSecretCards` its doc, and splitting the consent's §8i comment from the MIXED block. N-4: the self-check comment now states the current K/N contract. |
| M-5 bare + passphrase → two identical plates; N-1 declined passphrase keeps the registration | `aabef11` | New `TestFableSpecOneSeedBareAndWithPassphraseIsCutOnce`: RED under the fingerprint key — "planned 2 ms1 plates (byte-identical: true)" — GREEN under `sha256(entropy)`; mutation re-run through the same digest call on the fingerprint: RED again, restored. `TestFableSpecOneSeedTypedTwiceIsCutOnce` still GREEN. N-1: `reg.discardLast(seedID)` on the `bindPassphrase` failure leg. |
| N-2 device prints no per-record reason for a testnet key | — | Doc-only; accepted as stated by the reviewer (the door's one-line contract). |
| N-3 the two-keyless body names no path numbers | DECLINED | §8m bodies are FIXED, one per structural refusal (spec §8m; every FIXED body passes the modal-fits assertion); the primary names the paths in its CLI message because it has no modal. Recorded, not changed. |

## Gate at `aabef11` (controller-run)

`go vet ./...` — 0 lines beyond the `ArtifactDir` baseline. `gofmt -l .` —
exactly the five-file baseline. `go test ./md/ ./sysw/ ./mk/` — ok. gui shard
set — **ok, all 1373 tests across 24 shards** (r0 tip 1369; +4). Firmware
(`nix develop -c tinygo build -size short … ./cmd/controller`): **1,653,724 B
flash / 63,336 B RAM** (r0 tip 1,652,268 / 63,336; base `f5b068fa` 1,644,840 /
63,320).

## What was not done

- The addendum the implementer would have written does not exist; this file
  stands in for it, quoting the four commit messages it left.
- N-3, above.
