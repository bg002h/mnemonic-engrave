# Adversarial verification — D3-1 (verifier #0)

- **Finding:** D3-1 (moderate) — `--preview` never cleans the output directory; stale higher-index
  `plate-N` images from a prior run persist and can be mistaken for the current wallet's plates.
- **Location:** `crates/me-cli/src/main.rs:262` (`wire_previews`) + `crates/me-cli/src/preview.rs:119-172`
  (`render_plate`).
- **Verdict:** CONFIRMED (not refuted) on the facts, **severity adjusted moderate → low**.
- **Confidence:** high.

## What I checked

### 1. Does the cited code behave as claimed? — YES.

- `wire_previews` (`main.rs:227-296`): the *only* thing done to the output directory before rendering is
  the existence/type guard at **line 262** `if !dir.is_dir() { … EXIT_USAGE }`. There is no directory
  emptying, no non-empty warning, no `--force`, and no per-wallet/`chunk_set_id` filename namespacing.
- The render loop (`main.rs:271-294`) iterates **only** `manifest.plates` — i.e. the *current* run's
  plates. It writes each via `preview::render_plate(&sidecar, string, dir, plate.plate, png)`.
- `render_plate` (`preview.rs:119-172`): output path is `dir.join(format!("plate-{idx}.{ext}"))` where
  `idx = plate.plate` (1-based global plate sequence number). No pre-existing file is removed; the file is
  written by the Go sidecar via `os.WriteFile(path, payload, 0o644)` (`preview/main.go:128` `writeOut`),
  which writes only that single `--out` file and removes nothing else.
- Corroborating grep: the **only** `remove_*`/dir-cleanup calls in `crates/me-cli/src/` are
  `fs::remove_dir_all` inside `#[cfg(test)]` teardown (`preview.rs:290,300,311,329,341,360`). Non-test code
  contains no `remove_file` / `read_dir` / `is_empty(dir)` / `force` / overwrite-guard logic. The Go sidecar
  has no `os.Remove`/`RemoveAll`/`ReadDir` on the render path either.

Consequence: rendering wallet A (5 public plates → `plate-1..5.svg`) then a *smaller* wallet B (3 public
plates → `plate-1..3.svg`) into the **same** dir overwrites `plate-1..3` but leaves `plate-4.svg`/
`plate-5.svg` (wallet A's mk1 key-card plates, a different `chunk_set_id`) on disk, unreferenced by B's
manifest. The code-behavior claim is **accurate and reproducible** (the finder's probe reproduced it; the
source logic is unambiguous, so a re-probe is not load-bearing). I do not refute the facts.

### 2. Is the failure scenario reachable with valid inputs? — YES.

Two independently valid, different-sized bundles rendered into one reused directory. No exotic/invalid input
required. Reachable.

### 3. Does another layer already prevent it? — PARTIALLY (mitigates, does not eliminate).

The **manifest is the documented source of truth** and is 100% correct: `wire_previews` sets
`plate.preview = Some(path)` only for plates it actually rendered *this run*, so the emitted manifest lists
exactly the current wallet's N plates with their exact `preview` paths (D3's own "Manifest ↔ plates
cross-consistency — SOUND" negative result confirms this). The stale files are **not referenced** by the new
manifest. A user who follows the manifest (the intended workflow, spec §5) is fully safe. The gap is purely
at the raw-filesystem level for a user who ignores the manifest.

### 4. Is the severity honest? — Overstated for a funds-safety scale; **low is more honest**.

Reasons the funds impact is contingent rather than direct:

1. **The tool's authoritative output is correct.** `me` neither produces nor "accepts" the stale plate as
   part of this run — there is no *wrong-but-accepted* plate in the tool's actual deliverable (the manifest +
   NDEF). This is not "engraved output ≠ validated input"; every byte the tool emitted this run is right.
2. **Previews are advisory visual aids, not the engraving data path.** Per SPEC §9/§5 the preview exists
   "so the user can eyeball a plate before engraving"; the engraved data is the NDEF/NFC string the user
   drives from the manifest, not the SVG/PNG files. A stale SVG is not itself engravable device input.
3. **The stale files are the user's own prior explicit output.** Standard CLI semantics: writing named
   outputs into a user-chosen directory does not sweep unrelated pre-existing files (cc/gcc/ffmpeg/image
   converters all behave this way). The tool did not "silently mix" anything — the filesystem retained the
   user's earlier artifacts.
4. **The hazard requires a compound user error against the documented flow:** (a) reuse a *dirty* directory
   across two *different* wallets, (b) entirely ignore the manifest (the documented deliverable that lists
   exact per-plate `preview` paths), (c) treat orphan `plate-*.svg` as an authoritative plate set, and (d)
   physically engrave them.

The finder's own rationale for "moderate" (identical `plate-N` filenames with no wallet discriminator, and
physical/one-shot engraving) is a fair footgun argument, but for a *funds-safety* severity scale "moderate"
should be reserved for cases where the tool produces or accepts wrong data under a realistic,
manifest-following workflow. Here a user on the documented path is safe; the residual risk is a
directory-hygiene / defense-in-depth footgun. That is a genuine **low** finding, not a moderate one.

## Bottom line

The defect is **real and correctly located** — I confirm the code does not clean/warn/namespace the
`--preview` output directory, and stale higher-index plate images from a larger prior run persist. But the
funds-safety impact is contingent (correct manifest is the source of truth; previews are advisory; standard
output-dir semantics; requires dirty-dir reuse + manifest bypass to reach harm), so the honest severity is
**low**, not moderate. The finder's fix menu (refuse/`--force` a non-empty dir, sweep pre-existing
`plate-*`, or namespace by `chunk_set_id`) plus the proposed double-render regression test remain sound and
worth doing as hardening.

- refuted: **false**
- adjustedSeverity: **low**
- confidence: **high**
