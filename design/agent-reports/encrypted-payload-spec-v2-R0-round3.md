# R0 v2 round 3 — close-out of the round-2 fold

Artifact @ db7880c. Verdict: **0 Critical / 0 Important / 1 Minor / 0 Nit — GATE PASS.**

This closes the v2 R0 loop. Per project standard a re-review returning 0C/0I
closes the gate; no further rounds.

All eight fixes traced through the actual diff (`git diff 381265d..db7880c`),
not by re-reading final prose. Cross-reference consistency grepped across the
whole document. Two empirical claims reproduced byte-exactly:

- **Vector F verified from scratch.** The 2-of-3 `wsh-sortedmulti` bundle was
  regenerated and matches on every field: 15 records, `ms1`×3 at indices 0,1,2,
  lengths `75,75,75,111,93,111,93,111,93,85,85,85,85,85,77`, derived key
  `d9bdc867…`, tag `660202c7…`, ct_len 1353, blob 1421 bytes, sha256
  `97e059ac…`, and the exact header hex.
- **The decode negative is real, not vacuous.** `md.Decode` on §6.3's own
  example (`md1qqqsyqcyq5rq…`) returns `md: wire version mismatch` **despite
  `ValidMD == true`** — confirming the new §11.2 test discriminates. Likewise the
  uppercase variant still returns `ValidMD == true`, so the lowercase refusal
  test is not vacuous either.

Vectors A–E confirmed untouched by the fold.

## The one Minor — introduced by the fold's own cleanup

Removing the duplicated mode-`0600` bullet deleted the bullet but left its
continuation line, orphaning `` `main.rs:375`. `` onto the "prints the 12 words"
bullet, which it does not support.

**Folded**, and it surfaced a latent inconsistency worth recording: that bullet
also said the passphrase goes to **stdout**, while the implementation plan's
Task 8 mandates **stderr** — because `me seal md1… > payload.uf2` would
otherwise write the twelve words into the very file they decrypt (§2.3). The
spec now says stderr and agrees with the plan.
