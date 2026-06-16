# cycle-prep recon — 2026-06-16 — me-bundle-preview-layer

**Origin/master SHA at recon time:** `5e69e70`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `(none)`

Slug verified: `me-bundle-preview-layer`. Expectation: this is a forward-looking/greenfield feature FOLLOWUP (deferred v1 non-goal), so citations are architectural references, not precise code line numbers — verified at the level the FOLLOWUP makes them. All clean.

---

## Per-slug verification
### me-bundle-preview-layer
- **WHAT (from FOLLOWUPS.md):** Host-side **bundle orchestration** — a wallet backup is a *set* of plates (`md1` policy + `mk1` xpub chunk(s) + `ms1` secret typed on-device). Build a manifest + guided per-plate workflow ("plate N of M") and *optionally* a faithful host-side plate preview (possibly reusing SeedHammer's Go `engrave`/`backup` libs). Its own spec→plan→R0 cycle. Honors the per-string model (multi-chunk `mk1` = multiple plates).
- **Citations:**
  - `design/SPEC_seedhammer_engrave.md` §2 "v1 non-goals" — **ACCURATE.** Lines 32–34 list "Bundle manifest / guided multi-plate (\"plate N of M\") workflow" and "Plate-preview rendering on the host" as explicitly-deferred v1 non-goals. (Also line 161: "(Later) bundle manifest + guided multi-plate workflow; host-side plate preview.")
  - SeedHammer Go `engrave`/`backup` libs are reusable host-side — **ACCURATE.** In `/scratch/code/shibboleth/seedhammer` @ `6ab12c0`: `engrave/engrave.go`, `engrave/engrave_test.go`, `backup/backup.go`, `backup/backup_test.go` all carry **no `//go:build` tag** ⇒ host-portable (not TinyGo/driver-gated), and both packages ship host `_test.go`. So a faithful host-side plate preview built on them is viable.
  - "per-string model (multi-chunk `mk1` = multiple plates)" — **ACCURATE.** `design/SPEC_seedhammer_engrave.md` §3 line 68: "The engraver does not reassemble chunks … A multi-chunk card is simply multiple plates (a 'bundle'). … The deferred bundle layer is what sequences 'card = N chunks = N plates.'"
- **Bonus (set-integrity, load-bearing for this cycle):** SPEC §3 line 70 states per-chunk BCH validation **cannot detect a dropped/reordered chunk**; the `cross_chunk_hash` set-integrity guard is checked only at **reassembly**, so **missing-chunk detection is deferred to THIS bundle layer.** Verified against the pinned codec: `mk-codec 0.4.0` (`~/.cargo/registry/src/.../mk-codec-0.4.0/`) exposes the chunk-set metadata + reassembly — `chunk_set_id` (20-bit, CSPRNG), `chunk_index`, `total_chunks` (`0 < n <= 32`), `cross_chunk_hash = SHA-256(canonical_bytecode)[0..4]`, public `decode()` (enforces set integrity: `ChunkSetIdMismatch`, malformed-header, gaps/dupes, hash mismatch — see `error.rs:72-96`), and `encode_with_chunk_set_id()` for byte-deterministic re-encode. The bundle layer can call `mk_codec::decode` over the collected chunk strings to prove a set is complete & consistent before declaring the bundle engrave-ready.

**Action for brainstorm spec:** Citations are all accurate — no corrections needed. Cite source SHA `5e69e70` (me-repo) and seedhammer `6ab12c0`. The spec should resolve the open scope questions below (manifest-only vs. + preview; what "guided workflow" means for a non-interactive CLI; whether the bundle layer performs set-integrity reassembly-checking via `mk-codec::decode`).

---

## Cross-cutting observations
1. **No citation drift / no structural errors.** This is a greenfield feature; the FOLLOWUP's references are architectural and all hold at `5e69e70` / seedhammer `6ab12c0`.
2. **Scope is genuinely open** — the FOLLOWUP bundles three sub-capabilities that can ship independently: (a) bundle **manifest** (machine-readable list of the N plates a backup needs), (b) **guided workflow** UX (sequencing/checklist output), (c) **plate preview** rendering. (c) is the heaviest (Go-interop or a Rust re-implementation of SeedHammer's layout) and is the most cuttable. Brainstorm must pin v1 scope before sizing.
3. **Set-integrity belongs here** (per SPEC §3 line 70). A real value-add over the v1 per-string converter: the bundle layer can *prove* a collected chunk set is complete/consistent (via `mk-codec::decode`) — catching the dropped/reordered-chunk gap v1 cannot. Worth weighing as a v1 anchor feature vs. deferring.
4. **Preview = Go-interop decision.** Reusing `engrave`/`backup` (Go) host-side means either (i) a Go sidecar binary the Rust CLI shells out to (mirrors the existing `firmware/ndef-roundtrip` cross-lang harness pattern), or (ii) re-implementing the plate layout in Rust (fidelity risk). This is a brainstorm approach-tradeoff, not a recon finding.
5. **`ms1` stays off-tool.** Any manifest/workflow must represent the `ms1` plate as "type on device" — the tool never ingests/emits the secret (consistent with the security spine, [[mnemonic-engrave-project]]).

---

## Recommended brainstorm-session scope
- **One cycle**, but the brainstorm's first job is a **scope cut**: decide v1 = {manifest, guided-workflow text, set-integrity check} and **defer preview to a v2** (recommended), vs. including preview now (much larger, Go-interop). Rough sizing: manifest + workflow + set-integrity ≈ a few hundred LOC of Rust + tests (uses `mk-codec::decode` already a dep); preview adds a Go sidecar or a Rust layout port (multiplies the cycle).
- **SemVer:** a new top-level subcommand (e.g. `me bundle`) = **MINOR** (`me` is 0.1.x, pre-1.0; new subcommand is additive but more than a flag). Additive flags on it = PATCH.
- **Locksteps:** none of the firmware lockstep rules apply (no GUI `schema_mirror`, no firmware change). If/when `me` lands in a toolkit manual, mirror the new subcommand there (manual-mirror invariant) — currently `me` is not documented in a manual, so no lockstep yet.
- **Dependencies/ordering:** independent of `seedhammer-upstream-prs-tracking`. Does NOT block on the firmware PRs. The set-integrity feature leans on `mk-codec 0.4` (already pinned) — no new cross-pin.
- **Mandatory next gate:** the brainstorm spec (and later the plan) MUST pass an opus-architect R0 review to 0C/0I before any code (project standard, [[iterative-architect-review-standard]]).
