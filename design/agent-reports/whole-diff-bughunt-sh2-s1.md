# Whole-diff review sh2 bughunt, slice 1 (design and threat, executed probes)

Reviewer: Opus, effort medium. First review of this build. None of these findings comes from an earlier fix commit.
Targets: fork `bughunt-sh2-fix` `319ed85..91fb004`; me `bughunt-sh2` `393d6564..6c3331db`.
FIX `design/FIX_bughunt_sh2.md` is normative. Scratch: `/scratch/cache/bughunt/sh2/whole-diff-s1/`.

## Verdict

No fix changes a correct card, seat binding, address or key. Every new refusal
fails closed, with a clean error and no panic and no partial output. There are 0 Critical and 0 Important findings.

## Per-fix verdicts (probes executed)

- **bip380 xpubMeta, depth and child fallback (fix-01):**
  - Correct. Probes round-trip keys whose depth and child disagree with their origin.
  - The fork now keeps the input key's own depth and child.
  - me's promotion canonical still rebuilds them from the origin. This is filed as `sh2-me-canonical-depth` (architect S3, binding), and the seam row `accepted/sh-wsh-sortedmulti-16-keys` pins me's form through `canonical_from`/`origin_serialised` in rows.py.
  - Field-level diff of the vectors: that row's `canonical` is unchanged, so no address moves.
- **06a threshold check:**
  - `k=0`, `k=-1` and `k>n` are refused (probed).
  - Valid thresholds still encode.
- **06c type/script mismatch:** `tr`/`wpkh`/`pkh`/`sh(wpkh)` over `sortedmulti`, plus `wsh(KEY)` and `sh(KEY)`, are refused with an error. Singlesig and multisig descriptors still parse.
- **06b/50, the BlueWallet parser** (`nonstandard/parse.go:159-175`):
  - The order is: key-count check, then missing Policy, missing Format, no keys, threshold.
  - Defects 1 and 2 are refused cleanly (no `Encode()` panic).
  - The Derivation path is applied to every key after the loop, so a path declared after the keys now binds all keys instead of none.
  - The `ParseKey` change keeps network, chain code and key data.
- **codex32 guard** (`codex32.go:133`): correct.
- **picobin offset** (`picobin.go:328`): correct.
- **OTP white-label index** (`otp.go:185/253`): correct.
- **gui fixes:**
  - 02, 03, 04, 31, 32 and 22: correct.
  - fix-21 resolveOrigins self-check: it fails closed. See M-1 for one stale test comment.
- **Vectorfont relative-path fixes** (`cmd/vectorfont/main.go:449-452, 600-695`):
  - The h/v, l/m, c and s relative pen tracking is correct, and so is the optimizer's `p0 = p1`.
  - Font regen gives no diff (settled).
- **fix-39 build-tag seam** (`lockboot_values.go`, `lockboot_default.go`, `main_host.go`, `bughunt_sh2_39_test.go`):
  - `main_host.go` matches FIX §2.14 verbatim.
  - Retry safety was checked against the RP2350 bootrom source (`varm_otp.c:287-294, 403-404`, saved under the scratch `rom/` directory). Rewriting the same ECC value does not raise UNSUPPORTED_MODIFICATION, so a retried lock is idempotent.
  - A conflicting addr row refuses the lock, so it fails closed.
- **Seam vectors (me and fork, byte-identical):**
  - The vectors keep 74 rows: 29 have `device_admits` true and 45 false.
  - The field-level diff `393d6564 -> 6c3331db` shows exactly 11 rows flipped to `device_admits` false: the 9 `narrowed/*` refusals listed in the §4.7 amendment, plus `bluewallet/no-format-header` and `bluewallet/zero-cosigner-lines`. Those two also drop `device_probe`.
  - No row is added or removed, and `refusal_rows` and `invariant` are unchanged.
  - The two `gate/duplicate-key-*` rows that rows.py gains were already in the JSON. This is generator catch-up, not new data.
- **me SPEC_descriptor_input amendments (§4.2, §4.3, §4.7, §7, §9):**
  - Verbatim against the FIX.
  - They agree with the measured flips: 9 of the 14 §4.7 shapes are refused and the 5 that are still accepted match the `narrowed/*` rows with `device_admits` true.
- **me `descriptor_seam.rs`, `comment.json` and FOLLOWUPS:** consistent (sha, 29/45, `device_probe` count 0).
- **Test-file edits** (`scan_test`, `confinement_test` comment, `sysw/descriptor_test` titledZeroKey, `seam_test` counts): fine.

## Findings

### Minor

- **M-1** `gui/composer_selfcheck_test.go:147-150` and the fixture comment at `:256-266`:
  - The fix-21 unseated-default arm now fires before the §4f "two distinct fingerprints" arm, so the case's expected string became the default-origin message.
  - The fixture comment still says "§4f's is what fires", which is stale.
  - The §4f arm is now reached only when both colliding unseated slots sit at the default origin, and no test drives that case, so this row no longer exercises §4f. That is s2's lens to confirm.
  - It fails closed: no wrong card.
  - Fix: reword the comment to "the unseated-default arm fires first", and add a case whose shipped md1 has both slots at the default origin, so §4f's message is asserted.

### Nit

- **N-1** `bip380/bip380.go:352`: the 06c error formats `MultisigType` with `%v` and prints an integer ("Segwit (P2WSH) does not take a 0 expression"). It matches the FIX text, so it is cosmetic only.
- **N-2** `design/SPEC_descriptor_input.md` §4.2 AMENDED paragraph: "refuses, in this order" omits the pre-existing `expected %d keys, but got %d` check, which runs first (`parse.go:159`). The paragraph also says the "narrowed" covers note is dropped, but the rows keep `covers=["narrowed-4.2"]`; what was dropped is `device_probe`. The holder-facing message is unchanged (generic), so this is cosmetic only.

C / I / M / N: 0 / 0 / 1 / 2
