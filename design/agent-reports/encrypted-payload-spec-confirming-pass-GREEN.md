# Confirming pass — §6.3 accessor table + mk1 csids. **Spec R0 CLOSED.**

Reviewer: sonnet, mechanical/verification tier, tight scope. Dispatched 2026-08-07
to confirm the two remaining items from `encrypted-payload-spec-6.3-rereview.md`
landed correctly:

1. the §6.3 accessor citation fix (`derive_chunk_set_id` retracted, replaced with
   the verified four-accessor table), and
2. the mk1 `chunk_set_id` correction (`852310/852311/852308` → `153720/153721/153723`).

Brief stated the already-settled facts so they would not be re-derived, and
required execution over reading — citation accuracy is the thing this document has
repeatedly got wrong, and a host/device disagreement on the grouping key would be
Critical.

Verdict: **0 Critical / 0 Important / 0 Minor / 0 Nit — GATE: PASS.**

This closes the spec's R0 loop after **nine rounds** across two versions.

Note for the record: the reviewer's own first device-side attempt hand-transcribed
the record strings and produced spurious `codex32: not a valid md1 string` errors.
It caught this by byte-diffing against the source JSON before concluding anything,
then re-ran with programmatically-embedded strings. That is the same
transcription-defect class the build gate exists to catch, appearing inside the
review itself.

---

## Verification Summary

I regenerated vector G's public section with the exact command given (`mnemonic bundle --network mainnet --template wsh-sortedmulti --threshold 2 --group-size 0 --slot @0.phrase=bacon×24 --slot @1.phrase=abandon×23+art --slot @2.phrase=zoo×23+vote --json`), which produced 6 md1 chunks (one 6-chunk card) and 3 mk1 cards × 2 chunks (6 records) — 12 public records in 4 cards, matching §11.4's vector G shape.

I then ran all four accessors from the §6.3 table for real, against these raw records:

- **Host md1** (`codex32::unwrap_string` → `bitstream::BitReader::new` → `ChunkHeader::read` → `.chunk_set_id`): compiled against the exact locked crate version (`md-codec 0.40.0`, per `Cargo.lock`) in a scratch cargo project. All 6 chunks → `chunk_set_id = 841149`.
- **Host mk1** (`string_layer::decode_string` → `StringLayerHeader::from_5bit_symbols` → `Chunked{chunk_set_id,..}`): compiled against `mk-codec 0.4.1` (locked version). Three csids: `153721`, `153720`, `153723`.
- **Device md1** (`md.ParseChunkHeader(s)`) and **device mk1** (`mk.ParseHeader(s)`): ran in the real `/scratch/code/shibboleth/seedhammer` checkout via `nix develop -- go run` on a temporary file (deleted afterward; repo confirmed clean via `git status`). Results: md1 `ChunkSetID=841149` on all 6; mk1 `{153721, 153720, 153723}`.

Host and device agree exactly, and both match the spec's stated values (md1 `841149`; mk1 `153720/153721/153723`). Note: my first device-side attempt hand-transcribed the record strings and got corrupted middle sections, producing spurious `codex32: not a valid md1 string` errors — caught by byte-diffing against the source JSON before concluding anything, then re-ran with programmatically-embedded strings to get the clean result above. This is exactly the failure mode the task asked me to guard against by running rather than reading.

I also verified `derive_chunk_set_id`'s signature directly in the crate source: `pub fn derive_chunk_set_id(id: &Md1EncodingId) -> u32`, and `Md1EncodingId` is only constructible via `compute_md1_encoding_id(&Descriptor)` (requiring an already-decoded descriptor) or a raw `::new([u8;16])` wrapper — confirming the retraction is correct, not over-cautious. `StringLayerHeader` is confirmed `#[non_exhaustive]` in `mk-codec 0.4.1` source (and my test code needed a wildcard arm to compile, corroborating this in practice).

Cross-reference check across §6.4, §9, §10.2, §10.2.1, §11.4: all consistently state the `(HRP, chunk_set_id)` grouping + reassemble/decode requirement; every remaining mention of `md.Decode` or `derive_chunk_set_id` is explicitly in "not this" warning form, never as a live citation. No stale `852310/852311/852308` values remain anywhere in the file.

```
| # | Fixed? YES/PARTIAL/NO | Note |
| --- | --- | --- |
| 1 (Important — §6.3 accessor table) | YES | Table replaced with the 4 correct accessors; verified by execution on both host (md-codec 0.40.0 / mk-codec 0.4.1, the locked versions) and device (real seedhammer checkout), on freshly generated vector-G raw records. `derive_chunk_set_id`'s retraction verified correct via its actual signature. `#[non_exhaustive]` claim verified in source. |
| 2 (observation — mk1 csids) | YES | Corrected to 153720/153721/153723; measured independently on host and device, both agree, both match the spec's stated values. md1 csid 841149 also independently confirmed on both sides. |
```

No new defects found.

## VERDICT
Critical: 0   Important: 0   Minor: 0   Nit: 0
GATE: PASS

CONFIDENCE: execution — all four accessors were compiled and run against real crates/packages and real freshly-generated vector-G records, not read from the spec's prose; results cross-checked host vs. device and against the spec's stated csid values.

**Files referenced:**
- `/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_encrypted_payload_delivery.md` (§6.3 lines 444–521, §6.4 lines 639–644, §9 line 952, §10.2 lines 1049–1054, §10.2.1 line 1122)
- `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/md-codec-0.40.0/src/{codex32.rs,bitstream.rs,chunk.rs,identity.rs}`
- `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/mk-codec-0.4.1/src/string_layer/{bch.rs,header.rs,mod.rs}`
- `/scratch/code/shibboleth/seedhammer/md/chunk.go` (`ParseChunkHeader`), `/scratch/code/shibboleth/seedhammer/mk/mk.go` (`ParseHeader`) — read-only, temp test file created and deleted, repo left clean
- Scratch test project: `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/b4bd97f9-770f-407e-9105-3c9dcd1dc62b/scratchpad/csid-check/`
