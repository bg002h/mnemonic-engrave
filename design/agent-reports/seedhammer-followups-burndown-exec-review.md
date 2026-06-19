# FOLLOWUPS burndown — whole-diff exec review (round 0) — VERBATIM agent report

**Agent:** `a3a8df7d784690fe9` (adversarial opus architect; RAN base-vs-branch byte-equality probes on WalletPolicyId + md1/mk1 encode goldens + 9M+ fuzz execs). **Branch:** `chore/followups-burndown`. **Base:** `2d3fd7c`. **Final commit:** `17a6bcc`. **Date:** 2026-06-19.
**Verdict:** GREEN (0C/0I). Cleared the cosmetic/defensive nits burndown for merge. Mandatory consolidated post-implementation gate per CLAUDE.md phase (4).

---

# FOLLOWUPS burndown — whole-diff exec review (round 0)
**Reviewer:** opus architect (adversarial)  **Branch:** chore/followups-burndown  **Base:** 2d3fd7c  **Verdict:** GREEN (0C/0I)

## Wire/identity byte-equality probes (MANDATE) — RAN

**Static evidence first:** Across the ENTIRE diff, the only changed wire-string golden is `bundle/verify_test.go` `wpkhMK1` (the intended t6a2-M1 regen). Every other chunk-string golden — all `mk/mk_test.go` `parityVectors`, `wpkhMD1`, `tamperedCSIDChunks` — is byte-identical. `md/walletpolicyid_test.go`, `md/encode_test.go`, `md/encode_singlesig_test.go`, and the fuzz tests are UNCHANGED files (confirmed `diff` byte-identical across trees) and all pass against the new code.

**WalletPolicyId probe (md pkg, base vs branch, byte-for-byte identical):**
```
PROBE cell7_wpid=6650b9803b3c66210140540da8d765a0   ← matches pinned golden
PROBE singlesig_pkh    wpid=fc90e097... stub=fc90e097
PROBE singlesig_sh_wpkh wpid=8807be80... stub=8807be80
PROBE singlesig_tr     wpid=6ae6d59e... stub=6ae6d59e
PROBE singlesig_wpkh   wpid=1c0170fe... stub=1c0170fe
PROBE empty_shared_wpid=ce85bdfa... (no error: hashed AS-IS)
>>> diff base vs branch: IDENTICAL <<<
```
The legitimate elided-shared / empty-origin case returns the empty origin and is HASHED AS-IS with NO error — only the genuinely-unreachable no-decl fallthrough surfaces `errMissingExplicitOrigin`. t6a1-M1 changes ZERO WalletPolicyId bytes → stub binding intact.

**mk1 encode probe (mk pkg, base vs branch):** re-encoded the wpkh Card + all parity-vector Cards on both trees → 11/11 chunk strings byte-for-byte identical. The `& 0x1f` mask is a no-op for total ≤ 32 (total-1 ≤ 31, i ≤ 31); error-threading produces identical output. **md1 encode goldens** asserted via unchanged `TestEncodePayloadGoldens`/`TestEncodeMD1StringGoldens` → pass.

**Goldens that changed + display-only confirmation:** Only `'`→`h` display strings (mk1/md1 inspect-text, `Card.Path` round-trip expectations, parity-vector `path` display field) and the one regenerated `wpkhMK1` (csid-only). The `wpkhMK1` regen decodes to a byte-identical Card (net/path/fp/**stub 1c0170fe**/xpub) as the old chunks and `bundle.Verify` still binds — test NOT weakened. csid is an integrity field, not part of the `mk1.Stubs ⊇ WalletPolicyIDStubChunks(md1)` binding.

## Per-nit verdict (1-13)
1. **t4-M1** display-only ✓ (probes prove encode bytes identical; only `'`→`h` display) — OK
2. **t4-M2** chunk-count guard + error threading — mask no-op for real counts, `TestEncodeChunksGuard` covers boundary/over-limit — OK
3. **t4-M3** `MKChecksumSymbols`→`([]byte,error)` — all callers updated (`go build ./...` clean; `MDChecksumSymbols` correctly left `[]byte`) — OK
4. **#10a-M1** n-mismatch guard — `readPathDecl` always sets `pathDecl.n==descriptor.n`; canonicalize preserves it; guard unreachable on real input (3.9M-exec fuzz clean) — OK
5. **#10b-M1/M2** comment-only — `canonicalOrigin` doc accurate; tampered-csid comment verified (`0xce33a` header vs real `0x2d950`, both probed) — OK
6. **t6a1-M1** `resolveOriginRaw`→error — identity bytes unchanged (probe above) — OK
7. **t6a2-M1** regenerated `wpkhMK1` — same Card, still binds, not weakened — OK
8. **t6a2-M2** dropped dead `derived` param — `singleSigVerifyFlow` re-derives comparator internally (line 87/123); param was genuinely dead — OK
9. **t6a2-M3** `showNotice`/`showError` split — both delegate to `showModal`, identical presentation today — OK
10. **t5-M1** static assertion — compiles now (`qaProgram-bip85Derive==1`); PROVED it fails the build (`gui.go:165: [2]struct{} as [1]struct{}`) when a program is inserted — OK
11. **t6b-M1** `formatSlotList` lists all slots — `TestFormatSlotList` covers 0/1/2/3/4 — OK
12. **t6b-M2** thread `tpl`/`keys` — same deterministic `ExpandWalletPolicyChunks` output; removed second decode truly redundant (unreachable error path) — OK

## Test results
- `go build ./...`: clean (no missed caller / build break).
- Full suite `go test -count=1 ./gui/... ./md/... ./mk/... ./bundle/... ./codex32/... ./bip39/...`: **all ok**, exit 0.
- Fuzz: `FuzzWalletPolicyId` 3.88M execs, `FuzzEncodeSingleSig` 1.59M, `FuzzEncodePayload` 3.91M — all PASS, no new failures.
- `go vet` over the scoped packages: **identical** on base and branch (one pre-existing finding: `gui/op/draw_test.go:176` go1.26 ArtifactDir). Branch == base, **zero NEW findings**.
- Scope: diff touches no `nfc`/`template`/`projection`/`bip85-index` files; no `sort.` additions; "tinygo" hits are pre-existing comment text only. SKIP items (#10a-M3, #10a-M2) NOT done.

## Critical
None.
## Important
None.
## Minor
- `showNotice`/`showError` are presentationally identical today (intentional, documented for future divergence). No action.

## Bottom line
**GREEN (0C/0I).** Every wire/identity byte is proven byte-identical between base and branch (WalletPolicyId incl. the pinned `6650b980…` golden, md1 + mk1 encode output, stub binding). No legitimate input is newly rejected (empty-shared origin hashed as-is; n-mismatch guard structurally unreachable; 9M+ fuzz execs clean). No missed caller / build or test break. The single regenerated golden is csid-cosmetic and still binds. Scope and SKIP-set respected; no new vet findings. **This clears the burndown for merge.**
