# Funds-safety audit — Dimension D3: bundle pipeline & sidecar invocation (Rust side)

Auditor: D3 finder (multi-agent USER-FUNDS-SAFETY audit)
Repo: `/scratch/code/shibboleth/mnemonic-engrave`
Scope files: `crates/me-cli/src/bundle.rs` (read in full), `crates/me-cli/src/preview.rs` (read in full),
plus the wiring in `crates/me-cli/src/main.rs` (`run_bundle_cli` / `wire_previews`), `manifest.rs`,
`validate.rs`, `classify.rs`, the integration tests (`tests/cli.rs`, `tests/preview_cross_lang.rs`),
the pinned codec sources (md-codec 0.36.0 `chunk.rs::reassemble`, mk-codec 0.4.0 `decode`), and the
Go sidecar (`preview/main.go`, `layout.go`, `render_svg.go`).

Pins verified from `Cargo.lock`: `md-codec 0.36.0`, `mk-codec 0.4.0`.

/ TL;DR: the parse→emit chain is sound (plates are emitted verbatim from the same validated parse; codec
reassembly is a robust pure integrity gate that catches drop/dup/extra/foreign/reorder; ms1 is refused
early and never reaches the sidecar; the version handshake is exact and fail-closed). Two real gaps found,
both on the `--preview` output side: (D3-1, moderate) the preview output directory is never cleaned, so
stale higher-index `plate-N` images from a prior run persist and can be mistaken for the current wallet's
plates; (D3-2, low) `me` does zero validation of the sidecar's rendered output — a sidecar that exits 0 but
writes a blank/empty/wrong file is silently accepted and recorded in the manifest as a valid preview.

---

## Findings

### D3-1 (moderate) — `--preview` never cleans the output directory: stale plate images from a prior run silently persist and can be mistaken for the current wallet's plates

**Where:** `crates/me-cli/src/main.rs` `wire_previews` (lines 227-296) + `crates/me-cli/src/preview.rs`
`render_plate` (lines 119-172). The output path is `dir.join(format!("plate-{idx}.{ext}"))` where
`idx = plate.plate` (the 1-based global plate sequence number), written via the Go sidecar's
`os.WriteFile(path, payload, 0o644)` (`preview/main.go:128`). Before rendering, `wire_previews` only
checks `dir.is_dir()` (`main.rs:262`) — it does **not** empty the directory, warn on a non-empty
directory, or namespace files per wallet / `chunk_set_id`.

**Failure scenario (concrete, proven):** a user renders wallet A into `out/`, then renders a *different*
(smaller) wallet B into the *same* `out/`. Wallet A produced `plate-1..5.svg`; wallet B overwrites
`plate-1..3.svg` but leaves `plate-4.svg` and `plate-5.svg` — which are wallet A's **key-card (mk1) plates
from a different `chunk_set_id`** — untouched on disk. Wallet B's manifest correctly references only
`plate-1..3`, but the directory now holds 5 images named identically to a single wallet's plate set, with
no wallet identifier in the filenames. A user who engraves "everything in the folder" (rather than
cross-referencing the JSON manifest's `preview` paths) engraves two plates belonging to a *different
wallet's key card* into wallet B's backup stack — a silently mixed backup (wallet B rendered
unrecoverable, and/or wallet A's key card physically filed under wallet B).

**Proof (probe, this session):**
```
RUN1: md1 + MK1_A/MK1_B (set 0x12345) + MK1_C/MK1_D (a DISTINCT key card) -> out/plate-1..5.svg
RUN2 (same dir): md1 + MK1_A/MK1_B only                                    -> out/ STILL has plate-1..5.svg
RUN2 manifest "preview" fields reference ONLY plate-1, plate-2, plate-3.
=> plate-4.svg and plate-5.svg (wallet-A key card MK1_C/MK1_D) linger, unreferenced by RUN2's manifest.
```
Deterministic; reproduced across repeated runs. The overwrite of same-index files is correct (no
stale content *at a given index*), so the hazard is strictly the *trailing higher-index* leftovers.

**Why moderate (not important):** the emitted manifest is the intended source of truth and lists exactly
the correct N plates with correct `preview` paths, so a user who follows the manifest is safe. The hazard
requires the user to bypass the manifest and treat the raw image directory as the plate set — but the
files are named `plate-1.svg … plate-N.svg` with **no wallet/chunk_set_id discriminator**, which actively
invites exactly that. Engraving is physical, one-shot, and costly, so "render images, then engrave what's
in the folder" is a realistic workflow. This is squarely the dimension's "can a bundle silently mix files
from two different runs?" concern.

**Fix options:** (a) refuse a non-empty `--preview` directory (or require `--force`); (b) delete any
pre-existing `plate-*.{svg,png}` in the dir before rendering; (c) namespace filenames with the
`chunk_set_id` (and a wallet/run tag) so cross-run/cross-wallet files can never collide or masquerade;
(d) at minimum, emit a loud stderr warning listing pre-existing `plate-*` files it did not write.

**Regression test that would have caught it:** a hermetic-fake integration test that runs
`me bundle --preview DIR` twice into the same DIR — first a 5-public-plate bundle, then a 3-public-plate
bundle — and asserts that after the second run DIR contains no `plate-*.{svg,png}` file that is not
referenced by the second run's manifest `preview` paths (currently `plate-4/5` survive → test fails).

---

### D3-2 (low) — `me` performs no validation of the sidecar's rendered output; a sidecar that exits 0 but writes an empty/wrong file is silently accepted and recorded as a valid preview

**Where:** `crates/me-cli/src/main.rs` `wire_previews` (lines 278-293) and `preview.rs` `render_plate`
(lines 163-171). `render_plate` treats `out.status.success()` (exit 0) as the *sole* success criterion:
it never re-reads the written file, never checks it is non-empty, and never confirms the rendered content
corresponds to the input string. `wire_previews` then unconditionally sets `plate.preview = Some(path)`
and prints `me: rendered plate N`. The sidecar's stdout (`mode <m>`) is explicitly discarded
(`preview.rs:113-115` doc-comment: "which we do not need here"), so there is not even a mode/handshake
cross-check on the render call. There is likewise no verification that the string the sidecar rendered is
the string in the manifest — the trust is entirely "sidecar exited 0".

**Failure scenario:** any sidecar that exits 0 while producing a blank, truncated, placeholder, or
wrong-content image (a build/version skew that slips the exact-version gate, a partial write on a full
disk that still returns success from the sidecar's perspective, a future sidecar refactor bug) yields a
manifest that *looks complete* — every public plate carries a `preview` path — while the images are
useless or misleading. The user's pre-engrave visual verification (the entire purpose of `--preview`) is
silently defeated: a blank preview reads as "nothing to check", a wrong preview could be approved.

**Observed concretely (this session):** driving `me bundle --preview` against a fake sidecar that exits 0
after writing its `--out` file, `me` reported `me: rendered plate 1 → …/plate-1.svg` and exited 0 with
`plate.preview` set in the manifest, even though the written `plate-1.svg` was **0 bytes**. `me` accepted a
0-byte SVG as a valid preview without complaint. (Root cause of the 0 bytes in *this sandbox* is an
execution artifact, not a repo defect — see Negative/Uncertain results — but the acceptance-without-
validation is real regardless of what made the file empty.)

**Why low (not higher):** the preview is an advisory, host-side visual aid; the data engraved on the plate
is the NDEF/NFC string the user pushes from the manifest, **not** the preview image, so a bad preview does
not by itself corrupt the engraved data. And the *real* Go sidecar is defensive on the specific empty-input
case (`preview/main.go:70-73` rejects empty stdin with exit 1, which `me` maps to exit 4). The gap is that
`me` relies on that downstream defensiveness rather than validating the output itself, so it is only as safe
as the sidecar's internal error discipline — it degrades a funds-safety verification aid rather than
directly losing funds.

**Fix:** after a successful sidecar exit, stat/read the `--out` file and fail (or at least warn loudly and
leave `preview` unset) if it is missing or empty; optionally parse the sidecar's `mode` line and require it,
as a minimal render-actually-happened handshake.

**Regression test that would have caught it:** a hermetic-fake integration test where the fake exits 0 but
writes a 0-byte (or fixed-wrong) `--out` file; assert `me bundle --preview` either exits non-zero or does
**not** record a `preview` path pointing at an empty/invalid image (currently it records it and exits 0).

---

## Areas checked and found SOUND (negative results)

**Single-parse consistency (no re-encode divergence) — SOUND.** Every emitted plate `string` is
`s.clone()` of the *same* trimmed input that `parse_line` classified, ms1-refused, and pristine-validated
(`bundle.rs:205,255-263,282-292`). Set membership is proven by `md_codec::chunk::reassemble(&refs)` /
`mk_codec::decode(&refs)` whose **return value is discarded** — reassembly is used purely as an integrity
gate, never as a re-encode whose bytes could diverge from the emitted strings (`bundle.rs:246-247,273-274`).
So the bytes written to the manifest are byte-identical to the validated input; there is no parse→emit
divergence surface.

**Ordering / skip / duplicate / foreign chunk — SOUND (codec-enforced).** md-codec 0.36.0 `reassemble`
(`chunk.rs:305-389`) enforces: all headers agree on `count`/`chunk_set_id`/`version` (else
`ChunkSetInconsistent`); `parsed.len() == expected_count` (else `ChunkSetIncomplete` — catches both dropped
and *extra* chunks); after sort-by-index, `h.index == i` for all i (else `ChunkIndexGap` — catches
duplicates, e.g. `[0,0]` for count 2 fails at i=1); and a cross-chunk integrity check that the descriptor's
derived chunk-set-id equals every header's csid (else `ChunkSetIdMismatch` — catches a foreign chunk
re-stamped with a matching csid). mk-codec `decode` is analogously strict; the bundle tests exercise
dropped (`dropped_mk1_chunk_fails`), duplicate (`duplicate_mk1_chunk_index_fails`), reordered
(`reordered_mk1_chunks_still_verify`), mismatched-set-id (`foreign_mismatched_set_ids_fail`), and
same-set-id-foreign (`cross_chunk_hash_mismatch_fails`) cases. The bundle-side `total = chunks.len()` is
computed only *after* reassembly succeeds, so it always equals the true header count. Emitted `chunk_index`
comes from the same header read used for grouping (`h.index`), and reassembly independently re-reads each
string's own header and re-sorts, so a mis-grouping cannot slip a wrongly-labeled chunk into a set.

**ms1 refusal — SOUND.** Two independent gates: a classify-only pre-scan over all lines *before* any BCH
validation (`bundle.rs:188-192`), and a per-line refusal in `parse_line` (`bundle.rs:97-99`). ms1 is refused
with exit 3 whether it appears alone, first, middle, or last (`ms1_anywhere_refuses_early`,
`bundle_ms1_refused_exit_3`). ms1 never becomes a `Parsed`/`PlateEntry.string`; the ms1 reminder plate
carries `string: None` (`bundle.rs:297-306`). Classification is HRP-based and case-insensitive
(`classify.rs`), so `MS1…`/mixed-case cannot bypass the refusal.

**No secret reaches the sidecar; no secret in argv/env — SOUND.** The preview loop skips `PlateKind::Ms1`
(`main.rs:272-274`) and only renders plates with `Some(string)` (public md1/mk1). The string is delivered
to the sidecar on **stdin** (`preview.rs:155`), never as an argv/env value, so it cannot leak via a process
listing. The sidecar itself has no network and no secret handling. (ms1 could only reach the sidecar via a
`Some(string)` on an ms1 plate, which the pipeline never constructs.)

**Version handshake — SOUND / fail-closed.** `wire_previews` requires an exact string match between
`me-preview --version` output and `CARGO_PKG_VERSION` (`main.rs:245-259`). Mismatch → exit 2, **no render**
(`mismatched_version_exit_2`). Unparseable/absent version → exit 2. A sidecar built without `-ldflags`
reports an empty version → mismatch → exit 2 (never a silent stale render; covered by
`preview.rs::sidecar_version_empty_when_unset`). An *absent* sidecar degrades gracefully: a stderr note and
exit 0 with a manifest that carries no `preview` keys (`absent_sidecar_degrades_exit_0_with_note_and_manifest`)
— detectable, not a false "previewed" claim. The same located path is used for both the version probe and
the render (no re-resolution / TOCTOU swap).

**Exit codes & no success-on-failure — SOUND.** `run_bundle` errors propagate `e.exit_code()` (2 usage /
3 ms1 / 4 invalid-or-integrity) and the manifest is **not** emitted on failure
(`bundle_dropped_chunk_exit_4_no_stdout` asserts empty stdout on failure). A preview version/spawn failure
→ exit 2 and a render failure → exit 4, both returning *before* the manifest is serialized, so a failed run
never emits a complete-looking manifest. Serialize/write failures → exit 2 with a message.

**Manifest ↔ plates cross-consistency — SOUND.** `wallet_plates = plates.len()` and every plate is
renumbered `plate=i+1, of=total` over the fully-assembled ordered vector (`bundle.rs:308-313`); there is no
path that lists N in the manifest header while writing a different number of plate entries. Plate ordering
is deterministic (md1 singles, then md1 groups and mk1 groups in `BTreeMap` chunk_set_id order, then the
ms1 reminder), and every parsed input becomes exactly one plate — no dedup that could silently drop a plate,
and reassembly failure aborts the whole bundle rather than emitting a partial one. `plate-N` preview
filenames are unique (global sequence numbers) and the manifest's `preview` field records the exact path
written.

**Preview ↔ bundle byte identity — SOUND (for what `me` controls).** The bytes piped to the sidecar are the
same `plate.string` recorded in the manifest (`main.rs:275-278`); the sidecar trims only a single trailing
newline (`preview/main.go:69`) which `me` does not add, so the sidecar renders exactly the manifest string.
Auto mode selection (`text+qr` > `text` > `qr`, `layout.go:47-62`) can pick a text-less (QR-only) or
QR-less (text-only) layout, but every mode encodes the **same full string** (QR payload == text ==
`qr.Encode(s)`), so a mode difference between preview and device is at worst cosmetic, never a data-content
divergence. (Preview-vs-device *geometric* fidelity of the replicated `validateMdmk`/`engrave.Params`
layout is a separate preview-fidelity concern, out of D3's parse/emit scope.)

**Converter `--out` / `--manifest` file writes — SOUND (minor non-atomicity, low).** Both use
`std::fs::write` (truncate+write, non-atomic). A crash/disk-full mid-write can leave a truncated file, but
the error is surfaced and the process exits 2, and a truncated JSON manifest is invalid JSON (detectable
downstream). The NDEF `--out` payload is derived from a single `convert(&input)` parse (no re-encode). Not a
silent-partial-success path. Recorded as low/hygiene, not a funds finding.

---

## Negative / uncertain results worth flagging to other auditors

**The real cross-language sidecar round-trip was NOT exercised in this session.** `go` is absent on this
host, so `tests/preview_cross_lang.rs::real_sidecar_renders_public_plates_only` early-returns (skips) and
reports "ok" without building or driving the real Go sidecar. The authoritative end-to-end
`me → me-preview` byte round-trip therefore ran only in CI historically, not here. A follow-up auditor with
`go` installed should run `cargo test -p mnemonic-engrave` to actually exercise it.

**Sandbox execution artifact — investigated and dismissed as a repo defect.** While probing preview I/O I
observed that the *actual* `me` binary, launched from the repo tree in this sandbox, delivered **0 bytes**
of stdin to the sidecar (deterministically, 5/5), causing a 0-byte SVG that `me` nonetheless accepted (this
is the concrete evidence behind D3-2). I ruled this out as a source-code defect: a standalone Rust binary
replicating `render_plate` *exactly* — including reading its own stdin first and performing the prior
`me-preview --version` `.output()` call — delivered all 24 bytes reliably (3/3 and 5/5) against the same
fake sidecar, and the in-repo `preview.rs::render_plate_writes_file_and_returns_path` unit test (in-process)
passes with the string reaching the fake. The source `render_plate` (spawn stdin-piped, `write_all`, drop to
EOF, `wait_with_output`) is textbook-correct. The anomaly tracks the *binary's execution context* (repo path
vs `/tmp`), not its logic, and is best explained by the harness's filesystem-sandbox process wrapper
mishandling a nested child's stdin pipe. It should be treated as an environment artifact and re-verified in
a plain environment; only D3-2 (the missing output validation, which the artifact happened to expose) is
reported as a repo issue.

**Duplicate unchunked md1 singles are not de-duplicated** (`bundle.rs:205`) — supplying the same md1 single
twice yields two identical md1 plates. Not corruption (identical, correct content; nothing dropped or
altered), just redundancy; not a funds issue. Noted for completeness.

**Concurrent `me bundle --preview` into one directory** could interleave `plate-N` writes and mix files.
Unusual usage; compounds D3-1's fix rationale (namespacing/cleaning the dir). Low.

---

## Reviewed against `design/FOLLOWUPS.md`
No open FOLLOWUP covers the `--preview` output-directory hygiene (D3-1) or the sidecar output-validation gap
(D3-2); both are new. The Phase-B item (`me-bundle-preview-sidecar`, Resolved) documents the version-gate
and ms1-exclusion design, which I confirmed sound. Neither D3 finding is a re-report of a known deferred
item.
