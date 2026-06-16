# Spec — `me bundle` Phase A (bundle orchestration + chunk-set integrity)

- **Status:** Draft for review (promoted from the GREEN design `DESIGN_me_bundle_preview.md`). Pending spec self-review → user review → spec R0 architect gate.
- **Date:** 2026-06-16
- **Provenance:** FOLLOWUP `me-bundle-preview-layer`; recon `cycle-prep-recon-me-bundle-preview-layer.md`; design review `agent-reports/me-bundle-preview-design-review-R{0,1}.md` (R1 GREEN). Verified vs me-repo `3068f64`, `md-codec 0.36.0`, `mk-codec 0.4.0`.
- **SemVer:** new top-level subcommand `me bundle` ⇒ **MINOR**, `me` v0.1.x → **v0.2.0**.
- **Scope boundary:** This spec is **Phase A only** — the pure-Rust orchestration core. The faithful host-side **preview sidecar** (`me-preview`, Go) + bundled signed release archive are **Phase B** (deferred, `DESIGN_me_bundle_preview.md` §B), their own brainstorm→spec→plan→R0 cycle (→ v0.3.0).

## 1. Goal
Given the **public** constellation strings of one or more wallet backups, `me bundle`:
1. **proves** each multi-string chunk **set** (md1 descriptor and/or mk1 key card) is complete and internally consistent — catching a dropped/reordered/duplicate/foreign chunk that per-string BCH validation cannot detect;
2. emits a machine-readable **manifest** (the ordered set of plates a backup needs); and
3. emits a guided per-plate **workflow checklist**.

It never ingests or emits the secret `ms1` (represented only as a "type on device" reminder).

## 2. Non-goals (Phase A)
- No image/plate **preview** (Phase B). No Go interop, no release archive, no network.
- No string **generation** or **reassembly-to-wallet** (the tool proves set integrity but does not output the reconstructed descriptor/key — that is recovery-time tooling).
- No on-device interaction; no NFC writing. No change to the existing `me` (single-string→NDEF) subcommand.

## 3. CLI surface
- New subcommand: **`me bundle`** (an **optional** clap subcommand). When NO subcommand is given, the existing single-string→NDEF behavior runs exactly as today (`me --hex`, `me --in f --out g`, etc. unchanged — backward-compatibility is a hard requirement). When the `bundle` subcommand is given, the Phase-A path runs. (clap: optional `Option<Command>` subcommand with the existing flags retained on the top-level parser as the no-subcommand fallback.)
- **Input:** newline-separated PUBLIC strings from **stdin** (default) or **`--in <file>`**. Blank lines and surrounding whitespace are ignored/trimmed. Strings are public ⇒ argv is still never used to pass string content. Read the whole input into a `Zeroizing<String>` (defense-in-depth, consistent with the converter), even though only `ms1` is secret.
- **Output:**
  - Manifest JSON → **stdout** by default, or to a file with **`--manifest <file>`**. stdout carries ONLY the manifest JSON. (No bundle-specific `--stdout` flag — that name belongs to the converter path; reusing it here would collide. Bundle is stdout-by-default + `--manifest` for a file.)
  - Guided checklist + all human guidance/warnings → **stderr** (never stdout), so JSON and human text never collide (same discipline as the converter).
- **Exit codes** (consistent with the converter): `0` success; `2` usage error (bad flags, empty input); `3` `ms1` refused; `4` invalid string or set-integrity failure.

## 4. Pipeline (per-string PRISTINE first, then set-integrity)
Order is load-bearing (resolves design I-2): pristine-ness is enforced per-string **before** any reassembly, so a silently BCH-correctable string can never yield a "verified" set.

1. **Read & split** input into trimmed, non-empty lines. **Empty input (zero strings) → exit 2** (usage).
2. **Classify each line** by HRP via `classify::classify` (reused). On the **FIRST** `Format::Ms` line → print the converter's `ConvertError::RefusedSecret` RF-risk message to stderr and **exit 3 immediately — before validating any line** (early-exit; resolves m-1). On `ClassifyError` (unknown HRP / no separator) → exit 4 naming the offending line.
3. **Per-string pristine validation** via `validate::validate(fmt, s)` (reused verbatim):
   - md1 → `md_codec::codex32::unwrap_string` (pure verify; any corruption → `ValidateError::Md` → exit 4).
   - mk1 → `mk_codec::string_layer::decode_string`; reject `corrections_applied != 0` (`ValidateError::MkCorrected` → exit 4).
   This guarantees every surviving string is pristine before step 4.
4. **Group by `chunk_set_id` and prove set-integrity** (per HRP):
   - **chunk_set_id extraction (public API, no upstream change):**
     - **mk1:** from the `DecodedString` produced in step 3, call `mk_codec::string_layer::header::StringLayerHeader::from_5bit_symbols(decoded.data())`; the chunked variant carries `chunk_set_id` (real mk1 strings are always chunked, even `total_chunks == 1`). If `is_chunked()` is **false** (a `SingleString` header — possible only for a synthetic mk1 with ≤56-byte bytecode, never a real-world card) → **exit 4**, message `"mk1 SingleString header: unsupported for bundle (no chunk_set_id)"` (resolves spec-R0 I-1).
     - **md1:** parse the 37-bit `md_codec::chunk::ChunkHeader::read(&mut r)` where `r = md_codec::bitstream::BitReader::with_bit_limit(&payload, bit_count)` from `unwrap_string` (the chunked-flag = bit 0 of the first 5-bit symbol). Dispatch on the result (resolves spec-R0 I-2 — `WireVersionMismatch` is NOT "unchunked"):
       - `Ok(header)` → chunked; `header.chunk_set_id` is the group key.
       - `Err(ChunkHeaderChunkedFlagMissing)` → **unchunked single-payload md1**, integrity = `bch-only` (do NOT call `reassemble`).
       - `Err(WireVersionMismatch{..})` → **unsupported md1 wire version → exit 4** (`"unsupported md1 wire version"`); never silently treated as unchunked.
       - any other `Err` → **exit 4**.
   - **Per-group set-integrity** (the dispatch above identifies each group; this is the integrity call per group; resolves I-1 + m-6):
     - **chunked md1 group** (chunked-of-1 OR multiple, grouped by `chunk_set_id`) → `md_codec::chunk::reassemble(&refs)` proves completeness/consistency.
     - **mk1 group** (by `chunk_set_id`) → `mk_codec::decode(&refs)` proves it (re-verifies + checks the cross-chunk hash).
     - **unchunked single md1** has no group → integrity `bch-only`, **never** `reassemble` (per the dispatch above).
     - Both `mk_codec::decode` and `md_codec::chunk::reassemble` take **`&[&str]`** (resolves m-1): strings are owned as `Vec<String>`, so per group build `let refs: Vec<&str> = group.iter().map(String::as_str).collect();` before the call.
   - Two strings of the same HRP with **mismatched** `chunk_set_id` belong to **separate groups** (resolves m-2); each is validated independently and a partial group fails (step below).
5. **Integrity verdict:** any set that fails reassembly → **exit 4** naming the set's `chunk_set_id` and the specific codec error (mapped from the variants in §5). The **manifest is emitted only if every set is complete and consistent** (resolves m-4); otherwise nothing is written to stdout and the failure is reported on stderr.
6. **On full success:** build & emit the manifest (§6) + checklist (§7); **exit 0**.

## 5. Set-integrity error mapping (codec → user message, all → exit 4)
- **mk1** (`mk_codec::Error`): `ChunkSetIdMismatch`, `ChunkedHeaderMalformed(..)` (covers dropped chunk / `total_chunks` mismatch / duplicate `chunk_index`), `MixedHeaderTypes`, `CrossChunkHashMismatch`.
- **md1** (`md_codec::Error`): the analogous chunk-set errors from `md_codec::chunk::reassemble` (missing/duplicate index, `chunk_set_id`/`count` mismatch, re-derived-id mismatch).
Each is surfaced verbatim alongside the human framing "set `<chunk_set_id>` is incomplete/inconsistent: …".

## 6. Manifest schema (JSON, stdout/`--manifest`)
```json
{
  "tool": "me",
  "version": "0.2.0",
  "wallet_plates": 4,
  "ms1_required": true,
  "sets": [
    { "kind": "md1", "chunk_set_id": "0x1a2b3", "total": 1, "integrity": "set-verified" },
    { "kind": "mk1", "chunk_set_id": "0x4c5d6", "total": 2, "integrity": "set-verified" }
  ],
  "plates": [
    { "plate": 1, "of": 4, "kind": "md1",       "string": "md1…", "chunk_set_id": "0x1a2b3", "chunk_index": 0, "integrity": "set-verified" },
    { "plate": 2, "of": 4, "kind": "mk1-chunk", "string": "mk1…", "chunk_set_id": "0x4c5d6", "chunk_index": 0, "integrity": "set-verified" },
    { "plate": 3, "of": 4, "kind": "mk1-chunk", "string": "mk1…", "chunk_set_id": "0x4c5d6", "chunk_index": 1, "integrity": "set-verified" },
    { "plate": 4, "of": 4, "kind": "ms1",                                                                       "integrity": "n/a" }
  ]
}
```
- **`integrity`** ∈ { `"set-verified"` (chunked set proven complete), `"bch-only"` (single unchunked md1 — per-string BCH passed, no cross-chunk hash exists; resolves m-3), `"n/a"` (the `ms1` reminder) }.
- **Plate ordering:** md1 policy plate(s) (chunk_index order) → each mk1 set's chunks (chunk_index order) → a single trailing **`ms1` reminder** plate (no `string`; the tool never knows it). `wallet_plates` counts the public plates + the ms1 reminder.
- **`sets[]`** lists **only multi-string chunked sets** (chunked md1 and mk1) — each `set-verified`. An **unchunked single md1** is NOT a chunk set: it appears **only in `plates[]`** with `integrity: "bch-only"` and **no `chunk_set_id`/`chunk_index`**, and is **omitted from `sets[]`** (resolves m-2). The example's md1 set is a *chunked-of-1* (`total: 1`, hence it has a `chunk_set_id` and is `set-verified`). `sets[]` models ≥1 independent sets (e.g. one chunked md1 + two mk1 key cards → three entries), confirmed sound in design R1 (multi-set Q5). `string` is omitted for the `ms1` plate; `chunk_set_id`/`chunk_index` omitted for unchunked md1 and ms1.
- **`version`** is the crate version at build time (`env!("CARGO_PKG_VERSION")`) — the `"0.2.0"` literal above is illustrative, NOT hardcoded. The §10 manifest golden test (#12) must therefore either pin/normalize the `version` field or assert on a version-independent projection, so a routine version bump does not break the golden.
- Hex `chunk_set_id` rendered `0x%05x` (20-bit). The schema is forward-compatible for a Phase-B consumer (it selects which plate to preview via `plates[]`).

## 7. Guided workflow checklist (stderr)
One line per public plate plus the ms1 reminder, e.g.:
```
me: backup needs 4 plates (3 public + ms1 on device):
  plate 1/4  md1 policy        → push via NFC & engrave
  plate 2/4  mk1 chunk 1/2     → push via NFC & engrave
  plate 3/4  mk1 chunk 2/2     → push via NFC & engrave
  plate 4/4  ms1 secret        → TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool
```

## 8. Edge cases (resolves m-4)
- Empty input → exit 2. All lines `ms1` → exit 3 at the first `ms1` line. Exact-duplicate chunk (same `chunk_index` twice in a set) → codec dup-detection → exit 4. Incomplete set (missing chunk) → exit 4. Corrupted (BCH-correctable or failing) string → exit 4 at step 3. Mixed md1 + mk1 with any incomplete set → exit 4 (no manifest). A lone unchunked md1 with no mk1 → valid 1-public-plate manifest (+ ms1 reminder). mk1 chunks only, no md1 → valid manifest (+ ms1 reminder).

## 9. File structure
- **New** `crates/me-cli/src/bundle.rs` — orchestration: read/split, per-line classify+pristine-validate (reusing `classify`/`validate`), group by `chunk_set_id`, set-integrity, assemble the manifest model. Public entry `pub fn run_bundle(input: &str) -> Result<Manifest, BundleError>` (pure; no I/O) + a thin `BundleError` enum mapping to exit codes.
- **New** `crates/me-cli/src/manifest.rs` — the `Manifest`/`PlateEntry`/`SetEntry` types + JSON serialization (use `serde` + `serde_json`; add as deps) + the checklist renderer (`fn checklist(&self) -> String`).
- **Modify** `crates/me-cli/src/main.rs` — add the optional `bundle` clap subcommand (existing converter flags retained on the top-level parser as the no-subcommand fallback, §3); wire stdin/`--in` input + manifest output (stdout default, `--manifest <file>` for a file — no bundle `--stdout`); map `BundleError` → exit codes; print checklist to stderr.
- **Modify** `crates/me-cli/Cargo.toml` — add `serde` (derive) + `serde_json`; bump version to `0.2.0`.
- Reuse (no change): `classify.rs`, `validate.rs`, `lib.rs::exceeds_plate_budget`.

## 10. Testing
Pure-Rust (no `go`). Library tests in `bundle.rs`/`manifest.rs` + CLI tests in `tests/cli.rs`:
1. **Happy path:** one unchunked md1 + one 2-chunk mk1 set → manifest has 3 public plates + ms1 reminder; all `set-verified`/`bch-only`; exit 0; checklist lines correct.
2. **Multi-set:** one md1 + two distinct mk1 sets (different `chunk_set_id`) → `sets[]` has 3 entries, plates grouped & ordered correctly.
3. **Dropped chunk:** mk1 set missing one of N → exit 4, message names the set + a "missing/total" reason.
4. **Reordered chunks:** chunks supplied out of index order → still `set-verified` (order-independent) — asserts reorder does NOT falsely fail.
5. **Duplicate chunk:** same `chunk_index` twice → exit 4.
6. **Foreign chunk:** two mk1 with mismatched `chunk_set_id` presented as one intended set → treated as two sets, each incomplete → exit 4.
7. **Cross-chunk hash mismatch:** construct the vector via the public path (resolves m-3) — `mk_codec::encode_with_chunk_set_id` to get a valid set, then swap one chunk for a *different* set's same-index chunk (or re-encode a different KeyCard at the same `chunk_set_id`) so each chunk is individually pristine but the cross-chunk hash disagrees → `CrossChunkHashMismatch` → exit 4. (No codec-internals access needed; mirrors the `pipeline.rs` perturbation tests.)
8. **md1 chunked set:** build the vector with `md_codec::chunk::split` on a large descriptor (or the `wsh_multi_chunked` test vector) → all chunks `set-verified`; dropping one → exit 4. Also assert a single **unchunked** md1 → `bch-only` and is absent from `sets[]`.
9. **Pristine policy:** a single BCH-correctable mk1 chunk → exit 4 (`MkCorrected`), never "verified".
10. **ms1 refusal:** any `ms1` line → exit 3 early (before other-line validation); message contains "CODEX32".
11. **Edge:** empty input → exit 2; manifest-not-emitted-on-failure (stdout empty when exit 4).
12. **Manifest golden:** byte-stable JSON for a fixed input vector (deterministic field order via serde struct order). **Normalize the `version` field before comparison** (resolves m-4) — e.g. overwrite it with a fixed `"x.y.z"` token, or compare a `serde_json::Value` with the `version` key removed — so a routine crate version bump does not break the golden.

## 11. Lockstep / release
- No firmware lockstep (no GUI `schema_mirror`, no SeedHammer change). No toolkit-manual mirror yet (`me` not documented in a manual; if a manual is added later, the new `bundle` subcommand mirrors there).
- Bump `me` → **0.2.0**; CHANGELOG entry. Publishing a new crates.io version needs a `publish-update`-scoped token (per [[mnemonic-engrave-project]]).
- After Phase A ships, update `design/FOLLOWUPS.md`: mark `me-bundle-preview-layer` as Phase-A-done and open a `me-bundle-preview-sidecar` (Phase B) FOLLOWUP carrying `DESIGN_me_bundle_preview.md` §B (incl. R0 findings I-3/I-4/m-5 + the upstream-v1.4.2 pin).
