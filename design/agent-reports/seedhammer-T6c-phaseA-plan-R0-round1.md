# T6c Phase A plan — R0 review (round 1) — VERBATIM agent report

**Agent:** `ad86c1568c38893fa` (adversarial opus architect; RAN the per-vector template-parity probe). **Fork HEAD:** `65a1018`. **Plan commit:** `05a81f0`. **Date:** 2026-06-20.
**Verdict:** GREEN (0C/0I). All 4 round-0 findings resolved, no design drift. 1 cosmetic non-blocking Minor. Cleared for single-implementer TDD.

---

# T6c Phase A plan — R0 review (round 1)
**Reviewer:** opus architect (adversarial)  **Fork HEAD:** 65a1018  **Plan commit:** 05a81f0  **Verdict:** GREEN (0C/0I)

## Round-0 resolution check
- **I1: RESOLVED.** Task-3a (plan lines 556–607) now uses a per-vector table with a per-vector index list AND a `wantHex` column. `mkTree` takes `(rootTag, innerWsh, multiTag, k, indices)` — each vector built at its OWN n: `wsh_sortedmulti` = (tagWsh, tagSortedMulti, k=2, `[0,1,2]`), `sh_wsh_multi` = (tagSh+innerWsh, tagMulti, k=2, `[0,1]`). Pinned hex matches the vendored `.bytes.hex` byte-for-byte: `wsh_sortedmulti.descriptor.json` n=3/k=2/indices `[0,1,2]`, `bytes.hex=2082001821c22180`; `sh_wsh_multi.descriptor.json` n=2/k=2/indices `[0,1]`, tag `Multi`, `bytes.hex=2042001830860850`. Misleading Step-2 prose GONE (line 613: "PASSES immediately for both vectors … built at each vector's OWN n (n=3 / n=2)" + "there is no 're-verify VF2' failure mode here"). **OPTIONAL PROBE RAN** (verbatim plan `mkTree`+table, throwaway test at `65a1018`, deleted):
  ```
  --- PASS: TestZZR0ProbeTemplateParity/wsh_sortedmulti  OK got=2082001821c22180
  --- PASS: TestZZR0ProbeTemplateParity/sh_wsh_multi      OK got=2042001830860850
  ```
  n=2 leg no longer trips `errPlaceholderRange`; both legs byte-match the pinned `wantHex`.
- **I2: RESOLVED.** Imports accrete per task. Task 1 imports ONLY `testing` (line 124; full block deleted). Task 2 widens to `encoding/hex`+`seedhammer.com/codex32` (290–296). Task 3 adds `encoding/json`/`os`/`path/filepath`/`strings` (872–883). Task 5 adds `errors` (1168–1180). Each per-task body uses exactly its imports → every per-task commit compiles green. Production file keeping `import "errors"` correct.
- **M1: RESOLVED.** VF6 cite now `md/encode_singlesig.go:20` (notes `:16`–`:19` doc-comment). `grep -n "type PathComponent struct"` → line 20. Self-Review symbol list also `:20`.
- **M2: RESOLVED.** Framing note present (lines 62–63): A2 = drift-guard/frozen-output (self-referential alone); A1 (template-parity vs Rust `.bytes.hex`) + A3 (T6b byte-exact, `e1c4240`) are the non-circular byte-correctness anchors.

## No-drift confirmation
Via `git show 05a81f0`: the fold changed ONLY (a) VF6 cite `:18→:20`; (b) the M2 note (+2 lines); (c) Task-3a `mkTree` sig + per-vector `wantHex` table + corrected Step-2 prose + the pin; (d) per-task import-accretion notes (Tasks 1/2/3/5); (e) Self-Review symbol-list `:18→:20`. **No design-bearing block changed** — the Task-1 struct defs (`EncodeMultisigRequest{…OriginMode…}`, `MultisigCosigner`, `SlotInfo`, the three typed errors), Task-2 `EncodeMultisig` body, the `(out, stub, slots, err)` handle, `FpPresent` gating, `multiSigTree`, the ordering contract, round-trip/identity (A4), refuse paths (A6) do NOT appear in the diff → byte-unchanged from round-0 (whose byte-exact correctness was proven by running the verbatim assembler: T6b 6/6 chunks, `WalletPolicyId 7b716421…`).

## New findings (this round)
None. Fresh checks pass:
- `mkTree`+table compilable, correct Go (probe). `wsh_sortedmulti`=Wsh⊃SortedMulti; `sh_wsh_multi`=Sh⊃Wsh⊃Multi (`tagMulti`, not `tagSortedMulti`) — table carries the right `multiTag` per vector (line 580).
- Cited helpers exist: `loadDescriptor(t,name) *descriptor` (`testdata_test.go:143`), `loadBytesHex` (`:47`), `encodePayload(d) ([]byte,int,error)` (`encode.go:374`), `parsePathComponents` (`encode_singlesig_test.go:99`), `mustHexFuzz` (`:67`).
- Tags/types confirmed: `tagWsh=0x02 tagSh=0x03 tagMulti=0x06 tagSortedMulti=0x07` (`md.go:42-47`); `node`/`childrenBody`/`multiKeysBody`/`tag` (`md.go:37,105,110,135`).
- A6 `errors.Is` targets shipped vars: `errThresholdRange` (`encode.go:18`), `errKeyCountRange` (`encode.go:25`), `errKGreaterThanN` (`md.go:24`).
- Spec coverage I1–I8 still mapped; no new placeholder; type/signature consistency intact.

## Critical / Important
None / None.

## Minor
- (carry-over, non-blocking, NOT drift) VF facts + Self-Review still say "verified @8eb51d7" while HEAD is `65a1018`; pre-existing (round 0 reviewed at `65a1018` and found symbols correct). Re-confirmed every symbol exists at `65a1018`. Optional: refresh `@8eb51d7`→`@65a1018` at implementation time. Does not gate.

## Bottom line
**GREEN — 0 Critical, 0 Important.** All four round-0 findings resolved with no drift: I1's per-vector index table + `wantHex` byte-matches both vendored `.bytes.hex` (probe PASS both legs, n=2 bug gone); I2's per-task import accretion makes every commit compile green; M1 `:20`; M2 note present. Fold diff surgical — assembler design, struct constructor, `(out, stub, slots, err)` handle, `FpPresent` gating, ordering contract, round-trip/identity, refuse paths byte-unchanged from the round-0-verified plan. The one Minor (`@8eb51d7` tag) is cosmetic. **Cleared for implementation.**
