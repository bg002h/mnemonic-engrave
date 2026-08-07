# R0 round 2 — `SPEC_encrypted_payload_delivery.md` (fold verification)

Reviewer: sonnet, scoped fold-verification pass (not a fresh audit).
Dispatched 2026-08-07, after folding all 8 round-1 findings.
Verdict: **0 Critical / 0 Important / 0 Minor / 0 Nit — GATE PASS.**

Model tiering note: round 1 used fable (the single highest-stakes crypto gate).
Round 2 is a verification pass — "did the fold match the findings, did it
introduce a defect" — which is sonnet's tier per project standard. Fable is not
spent per round.

Persisted verbatim.

---

Confirms the `Platform.LockBoot()` citation used by §10.2.1 and the §11.2 fake-platform assertion is real and matches the round-1 citation.

## Fold table

| # | Fixed? | Note |
|---|--------|------|
| I1 | YES | §6.2 now bounds `payload_kind ∈ {0x01, 0x02}` with `0x03` gated on §12.6; no residual `== 0x01`-only text anywhere in the file (grepped all `payload_kind` occurrences). |
| I2 | YES | `--addr` deleted from the §9 synopsis; only remaining mention is the explicit "there is deliberately no `--addr` flag" prohibition with the wrap-to-`0x10000000` rationale. |
| I3 | YES | §10.2 step 6 rewritten to route through the new §10.2.1 allow-list, fail-closed on anything else. Verified against `gui/scan.go` source: the allow-list + "everything else including `debugCommand`, `addressText`, output descriptors" clause covers all 8 real branches of `Scan()` (debugCommand, bip39, OutputDescriptor, codex32.New secret, mdmkText, addressText×2 nets, unknown-format error). No branch is left unnamed. |
| I4 | YES | §11.1 freshness test text is unambiguous (two seals, same plaintext, must differ in salt/IV/mnemonic/ciphertext). Vector B independently recomputed from stated inputs (PBKDF2-HMAC-SHA256 + AES-256-GCM, Python `hashlib`/`cryptography`): derived key, tag, and blob sha256 all match byte-for-byte, including the full hex dump. Also independently confirmed the load-bearing claim: a key derived with iterations hardcoded to 100000 fails `InvalidTag` against vector B's ciphertext/AAD, while the header-driven (100001) key succeeds. |
| M1 | YES | Bound changed to `ct_len <= 8191` with the exact overflow rationale (`s.overflow` trips when `s.n == len(s.buf)`, an exactly-full 8192-byte buffer). Closes the described failure scenario. |
| M2 | YES | New §10.2.1 sentence: "classification MUST also be cross-checked against `payload_kind`... enforce the policy on the classified content." Combined with the kind→permitted-classification table (`0x01`→`mdmkText`, not `codex32.String`), an `ms1` string mislabeled `0x01` classifies to the wrong type and is rejected. |
| N1 | YES | §9.1 and §11.4 now both state "payload bytes beyond the blob are `0x00`," matching the pinned UF2 sha256. |
| N2 | YES | §6.2 adds the unsigned/wider-than-32-bit arithmetic requirement; §11.2 explicitly adds the `ct_len = 0xFFFF_FFF0` bound-violation case. |

No new defects or contradictions found. Checked specifically: no stray `payload_kind == 0x01` or `--addr` text anywhere in the file; `8191`/`8192` usage is consistent throughout (no bound ever reads 8192); the §11.3 mutant→killer table's seven rows all cite tests that actually exist verbatim in §11.1/§11.2/§11.4; the §11.2 `command: lock-boot` assertion ("fake/instrumented platform that fails the test if `LockBoot` is called, not by return value alone") is a genuine kill condition — it exercises the real allow-list-then-dispatch path rather than checking an isolated return value, so a weakened or omitted allow-list would actually trip it.

## VERDICT
Critical: 0   Important: 0   Minor: 0   Nit: 0
GATE: **PASS**
