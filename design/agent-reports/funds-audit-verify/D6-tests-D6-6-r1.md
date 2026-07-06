# Adversarial verification — D6-6 (md1 chunked/single discriminator + md-codec-0.36 drift guard)

Verifier: adversarial verifier #1. Date: 2026-07-06.
Finding under test: D6-6 (severity **important**), finder report
`design/agent-reports/funds-audit-D6-tests-round0.md`.
Location cited: `crates/me-cli/src/bundle.rs:144`.

## Verdict: REFUTED at the stated (important / funds-loss) severity.
Residual test-hardening value is genuine but **low** (not funds-losing).
Confidence: **high** (empirically probed, pin verified).

---

## What the finding claims

The chunked-flag probe (`bundle.rs:144-167`) uses `read_bits(5).map(|sym| sym & 0x01 != 0)`
as the chunked/single discriminator, then only calls `ChunkHeader::read` when that flag is
set (a documented workaround for an md-codec-0.36 deviation). The finder claims:

1. Coverage is "exactly two hand-built fixtures"; the `ChunkHeaderChunkedFlagMissing` and
   `WireVersionMismatch -> Md1WireVersion` arms are untested; no test pins the 0.36 behavior.
2. **A masked-bit mutation (`& 0x03`) can escape** the suite, and a patch within md-codec 0.36.x
   could silently flip classification.
3. **Failure scenario (funds loss):** one chunk of a multi-chunk md1 is misclassified as
   `Md1Single`, admitted as a standalone bch-only "complete" plate, the set-completeness check
   is bypassed, and the remaining chunks are silently dropped → user engraves an incomplete,
   unrecoverable backup that looks complete.

The narrow test-adequacy facts (1) are **true**. The funds-loss failure scenario (3), and the
"mutation escapes → funds loss" bridge (2), are **not substantiated** and are contradicted by
the code mechanics and by an existing test.

---

## Evidence

### A. The discriminator direction the failure scenario needs is CHUNK→single; `& 0x03` cannot produce it.

`chunked_flag = (sym & mask) != 0`. By md-codec's own definition (cited in the DEVIATION
comment, `chunk.rs decode_with_corrections`, `symbols.first() & 0x01`): a genuine **chunk** has
bit0 = 1; an **unchunked single** has bit0 = 0.

- `& 0x01 → & 0x03` (`= bit0|bit1`): a chunk (bit0=1) still yields non-zero → `chunked_flag = true`
  → **still classified as a chunk**. `& 0x03` can *only* change classification for an *unchunked
  single* whose first symbol has bit1=1 (0→"chunked"). It **never turns a chunk into a single**,
  so it **cannot** produce "a set member admitted as a complete plate."
- What `& 0x03` actually does to such a single: `chunked_flag=true` → `ChunkHeader::read` →
  per the documented 0.36 deviation returns `WireVersionMismatch{got:2}` → `Md1WireVersion`
  error → **exit 4 (false-REJECT)**. That is an availability regression, not a silent-acceptance
  funds loss.

### B. The mutation that *could* cause chunk→single (`& 0x02`) IS CAUGHT by an existing test.

Copied the crate to `/scratch` (offline, deps cached) and ran the real suite under two mutations:

| Mutation | Direction | Result |
|---|---|---|
| baseline `& 0x01` | — | 16/16 pass |
| `& 0x02` (drop bit0, keep bit1) | can turn chunk→single | **CAUGHT** — `md1_chunked_set_verifies_and_drop_fails` FAILS (panic at `bundle.rs:508`, the `assert!(m.sets.iter().any(kind==Md1 && SetVerified))`) |
| `& 0x03` (finder's cited escape) | single→"chunked" only | escapes: 16/16 pass — **but that same chunked-set test still passes**, proving chunks are still classified as chunks under `& 0x03` (no set member becomes a lone plate) |

So: the *funds-losing* mutation is caught; the *escaping* mutation is not funds-losing. The
finder conflated "a mutation escapes the tests" with "that mutation loses funds"; they are two
different mutations. (The finder's own §4 M4 even concedes `& 0x02` is "probably caught" — the
probe upgrades that to *definitely* caught.)

### C. Partial misclassification does not silently pass; only total does, and total is caught.

If a mask mutation misclassified *some* chunks of an N-chunk set as singles, the remaining
correctly-classified chunks form a group that fails `md_codec::chunk::reassemble` (missing
members) → `SetIncompleteMd` → **exit 4**. Only *all-N* misclassified is silent (no group at
all), and that case is exactly what `md1_chunked_set_verifies_and_drop_fails` asserts against
(it demands a `Kind::Md1` `SetVerified` set be present). Verified empirically in (B).

### D. The "silent md-codec 0.36.x patch" drift prong is blunted by the lockfile.

`Cargo.lock` pins md-codec to **`0.36.0`** with checksum
`75b1bfb71335d439e10bcf5c1e6dacdd25da5eddd3c0051b4c6c6abf628804d6`
(and mk-codec `0.4.0`). A different 0.36.x cannot be pulled in "silently"; it requires a
deliberate `cargo update` that rewrites the lock + checksum — a reviewed diff. Builds run
`--locked`. Additionally, the *primary* discriminator (bit0) is computed **by bundle.rs itself**
via `BitReader::with_bit_limit(...).read_bits(5)`, not delegated to `ChunkHeader::read` for the
single case; a md-codec regression affecting the *chunked* path (`ChunkHeader::read` on a genuine
chunk) is itself exercised by `md1_chunked_set_verifies_and_drop_fails`, which decodes a real
md-codec `split()` multi-chunk set — so its funds-relevant manifestation is already guarded.

### E. The untested arms are safe outcomes, not funds-loss.

- `Err(ChunkHeaderChunkedFlagMissing) => Md1Single` — reachable only if bit0=1 but
  `ChunkHeader::read` reports flag-missing (an md-codec self-contradiction). If it ever fired on
  real chunks, the chunked-set test would catch it (chunks→singles→no set→assert fails).
- `Err(WireVersionMismatch) => Md1WireVersion` — an **error/rejection** (exit 4), not acceptance.

Neither untested arm is a silent-acceptance funds-loss path.

---

## Assessment of severity

- The narrow observations (two `parse_line` md1 error arms lack a direct unit test; no explicit
  assertion pins the 0.36 `WireVersionMismatch{got:2}` deviation; no fixture varies the first
  symbol's bit1) are **factually true** and worth a cheap regression test — but they are
  **test-hardening niceties at LOW severity**.
- The finding's headline — **important**, "misclassification admits a set member as a complete
  plate → silently incomplete unrecoverable backup" — is **not concretely reachable**:
  the funds-losing mask mutation is caught by an existing test; the cited escaping mutation
  (`& 0x03`) produces a false-reject, not silent acceptance; the dependency is version+checksum
  pinned; and even a hypothetical md-codec chunked-path regression is covered by the real
  chunked-set test. No wrong-but-accepted plate is demonstrable.

**Refuted at important/funds severity. Residual is a low-severity test-hardening item.**
