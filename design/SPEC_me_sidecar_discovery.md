# SPEC — me sidecar discovery integrity (Cycle D: F11)

Status: **DRAFT, pre-R0** (2026-07-09). Closes `me-sidecar-discovery-integrity` (F11)
from `design/FOLLOWUPS.md`. Evidence: `design/agent-reports/funds-audit-D5-hygiene-round0.md`
(D5-4). Executed locally (cloud CCR env failed to start). Process: R0 gate to 0C/0I →
single implementer, TDD → post-impl adversarial review.

**Merge adjacency note:** this cycle edits `preview.rs` (`locate_sidecar`) and reads near
`main.rs::wire_previews`, regions ALSO touched by Cycle A / PR #1 (render_plate validation,
dirty-dir refusal). The edited functions differ (`locate_sidecar` vs `render_plate`;
discovery vs dirty-dir scan), so a textual conflict is unlikely, but whichever of PR #1 /
this PR merges second may need a trivial rebase. The controller rebases after the first
lands.

Recon (verified against current master `9fafb6b`, 2026-07-09):
- `locate_sidecar` (preview.rs:62-86): discovery order = (1) the dir of the current exe
  (release archives ship `me` + `me-preview` side by side — the trusted path), then
  (2) **each entry on `$PATH`**. First existing file wins.
- Version gate (main.rs:246-259): runs `<sidecar> --version`, string-matches
  `me-preview <ver>`, requires `found == CARGO_PKG_VERSION`. A hostile stand-in need only
  PRINT `me-preview 0.3.0` to pass — the gate provides no integrity, only version-skew
  protection.
- Payload sent to the sidecar is PUBLIC (md1/mk1 only; ms1 is refused upstream and never
  reaches preview). So the exposure is: a planted `me-preview` earlier on `$PATH` (when
  `me` is installed WITHOUT a co-located sidecar, e.g. `cargo install me`) receives the
  public descriptor/xpub payload AND can write arbitrary files into the user's preview
  `--preview DIR`. **Not a seed/secret leak** — hence the finding is low — but a real
  integrity gap (arbitrary-write + public-key exposure to an attacker-controlled binary).

## The fix — co-located-only discovery, explicit opt-in escape hatch

**Design decision (for R0 to confirm):** drop the implicit `$PATH` fallback. The
sidecar is auto-discovered ONLY next to the current executable (the trusted release-archive
layout). If it is not there, `me` degrades gracefully exactly as today (prints the "preview
skipped" note, exits 0) — it NO LONGER silently reaches for a `$PATH` binary. For the
legitimate non-standard install (sidecar in a different dir), add an **explicit opt-in**:
an env var `ME_PREVIEW_BIN=/abs/path/to/me-preview` (and/or a `--preview-bin PATH` flag),
by which the user vouches for a specific binary. Explicit path = the user's choice,
not an ambient-authority `$PATH` search an attacker can seed.

Rationale vs the alternatives (D5-4 named "hash-pinned or co-located-only"):
- **Hash-pinning** (embed the expected sidecar hash at build, verify before exec) is
  rejected: the sidecar is a separately-built artifact whose hash varies per platform/build,
  so a pinned hash is brittle (breaks on every legitimate rebuild) and the maintenance cost
  is disproportionate to a LOW, public-only finding.
- **Warn-only** (keep `$PATH`, print a warning) is rejected: a warning on an
  already-executed discovery is not fail-closed and users ignore warnings.
- **Co-located-only + explicit opt-in** is fail-closed by default, preserves the escape
  hatch for real multi-dir installs, and matches the project's fail-closed posture.

## D1 — remove the implicit `$PATH` fallback from `locate_sidecar`

`locate_sidecar` returns the exe-adjacent sidecar if present, else `None`. Delete the
`$PATH` scan (preview.rs:76-83). The `None` path already degrades gracefully at the call
site (main.rs:238-241) — no caller change needed for the graceful case.

Acceptance:
- With a `me-preview` next to the current exe → discovered (unchanged happy path).
- With NO co-located sidecar but a `me-preview` earlier on `$PATH` → `locate_sidecar`
  returns `None` (previously returned the `$PATH` one). Test: set `$PATH` to a dir holding
  a fake `me-preview`, ensure the exe dir has none, assert `None`. (Fails today: returns
  the $PATH binary.) NOTE: the test must control "the exe dir" — use a helper that takes
  the search dir(s) as parameters (refactor `locate_sidecar` to delegate to a pure
  `locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>) -> Option<PathBuf>` that the
  test can drive without depending on the real `current_exe()`).

## D2 — explicit opt-in (`ME_PREVIEW_BIN` env var, and/or `--preview-bin` flag)

When `ME_PREVIEW_BIN` is set to an existing file, `locate_sidecar` returns that path
(highest precedence, before the exe-adjacent check) — the user has explicitly vouched for
it. If set but the file does not exist → return `None` (graceful degrade) OR a clear error
(R0 to choose; draft: treat a set-but-missing `ME_PREVIEW_BIN` as an error exit, since the
user asked for a specific binary that isn't there — fail loud, don't silently fall back to
exe-adjacent which would be surprising).

R0 open question: env var only, or ALSO a `--preview-bin` clap flag on `bundle`? Draft:
**env var only** for this cycle (smaller surface; the flag can be a later addition). The
version gate is UNCHANGED — an explicitly-chosen binary still must pass `--version` match
(the user vouches for identity; the gate still catches a stale/mismatched version).

Acceptance:
- `ME_PREVIEW_BIN=/path/to/valid/me-preview` (version-matched) → used, preview renders.
- `ME_PREVIEW_BIN` unset → co-located-only behavior from D1.
- `ME_PREVIEW_BIN=/nonexistent` → error/graceful per R0 decision; test pins whichever.

## D3 — documentation

Update the `locate_sidecar` doc comment (the current "Each entry on `$PATH`" bullet is now
false) and the `--preview` help/README to state: the sidecar must be installed alongside
`me`, or pointed at explicitly via `ME_PREVIEW_BIN`. Note the security rationale (no
ambient `$PATH` search) in the doc comment so a future editor does not "helpfully" restore
the fallback.

## Ordering & verification
D1 (remove fallback, with the `locate_in` refactor enabling tests) → D2 (opt-in) → D3
(docs). TDD: the D1 `$PATH`-no-longer-used test is a genuine red today (current code
returns the $PATH binary). D2 tests are new behavior. Full verification:
`ME_REQUIRE_GO=1 cargo test --locked` at root (all green incl. new discovery tests) +
`go test ./...` in preview/ still green + clippy clean + a manual e2e: with a co-located
sidecar `me bundle --preview` works; after removing it and planting one on `$PATH`,
`me bundle --preview` now prints "preview skipped" (does NOT use the $PATH binary);
`ME_PREVIEW_BIN=<that>` makes it render again.

## Open questions for R0
1. **Escape hatch:** `ME_PREVIEW_BIN` env var only (draft) vs also a `--preview-bin` flag.
2. **Set-but-missing `ME_PREVIEW_BIN`:** error-exit (draft, fail-loud) vs graceful degrade.
3. **Is dropping the `$PATH` fallback an acceptable behavior change?** It could break a
   user who today relies on a `$PATH` sidecar with `me` installed bare. Mitigations: the
   graceful "preview skipped" note tells them what to do, and `ME_PREVIEW_BIN` is the
   supported replacement. Draft says yes (fail-closed default is worth it for a
   trust/integrity fix). R0 to confirm this is not an over-reach for a LOW finding.
4. **Symlink/relative-path hardening of the exe-adjacent path:** should we canonicalize
   `current_exe().parent()` and reject a symlinked sidecar, or is that gold-plating a LOW
   finding? Draft: no extra hardening — out of scope; the exe-adjacent dir is already the
   trust root.
