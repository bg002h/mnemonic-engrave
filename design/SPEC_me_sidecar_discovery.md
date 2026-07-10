# SPEC — me sidecar discovery integrity (Cycle D: F11)

Status: **GREEN — R0 passed at round 1 (0C/0I, 6 advisory nits folded inline)** (reviews:
`me-sidecar-discovery-spec-R0-round0.md` = 0C/1I/1L/3N all folded [I1 migrate the 7+1
$PATH-injecting tests to ME_PREVIEW_BIN; L1 fail-loud in the wrapper]; `…-round1.md` =
GREEN, migration count verified exact, design endorsed). Cleared for single-implementer
TDD. Closes `me-sidecar-discovery-integrity` (F11)
from `design/FOLLOWUPS.md`. Evidence: `design/agent-reports/funds-audit-D5-hygiene-round0.md`
(D5-4). Executed locally (cloud CCR env failed to start). Process: R0 gate to 0C/0I →
single implementer, TDD → post-impl adversarial review.

**Base:** this branch is rebased onto merged master `4908dbb` (Cycles A/B/C already
landed). Cycle A's `wire_previews` changes (dirty-dir refusal, EmptyOutput) are in the
tree; D stays inside `locate_sidecar` + a new env read in the wrapper and does NOT disturb
A's validation (R0 confirmed no overlap).

Recon (verified against current base `4908dbb`, 2026-07-09; line numbers approximate,
locate by symbol):
- `locate_sidecar` (preview.rs, ~:62): discovery order = (1) the dir of the current exe
  (release archives ship `me` + `me-preview` side by side — the trusted path), then
  (2) **each entry on `$PATH`** (the arm to delete, ~:81-89). First existing file wins.
- Version gate (main.rs, `wire_previews`): runs `<sidecar> --version`, string-matches
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
`$PATH` scan (preview.rs ~:81-89). The `None` path already degrades gracefully at the call
site (main.rs:238-241) — no caller change needed for the graceful case.

Acceptance:
- With a `me-preview` next to the current exe → discovered (unchanged happy path).
- With NO co-located sidecar but a `me-preview` earlier on `$PATH` → `locate_sidecar`
  returns `None` (previously returned the `$PATH` one). Unit test on the pure
  `locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>) -> Option<PathBuf>` helper
  (refactor `locate_sidecar` to delegate to it so the test drives search dirs without
  depending on real `current_exe()`). **`locate_in` unit matrix (R0 N3)**: (a) explicit
  present → returns explicit (precedence over exe-adjacent even when both exist);
  (b) explicit None + exe_dir has the file → exe-adjacent hit (the co-located happy path,
  which otherwise has no integration coverage); (c) explicit None + exe_dir None/no file
  → None. `$PATH` is never consulted by `locate_in` at all.
- **Behavioral integration test (R0 N3): `planted_path_sidecar_ignored`** — via the real
  `me` binary with a fake `me-preview` ONLY on `$PATH` (no co-located one) → `me bundle
  --preview` prints "preview skipped" and exits 0, does NOT use the `$PATH` binary. This
  locks the closure at the real entry point, not just the unit helper.

**MANDATORY existing-test migration (R0 I1 — these tests break otherwise).** Seven tests
in `crates/me-cli/tests/cli.rs mod preview` and one in `preview_cross_lang.rs` currently
inject the fake/real sidecar EXCLUSIVELY via `$PATH` (`.env("PATH", &bindir)`); after D1
they would silently degrade to exit 0 (no sidecar found). Migrate each from
`.env("PATH", &bindir)` to `.env("ME_PREVIEW_BIN", bindir.join("me-preview"))` (D2's own
mechanism — so the migration doubles as D2's happy-path coverage).

**Migrate EXACTLY these 7 (cli.rs `mod preview`) + 1 (preview_cross_lang.rs)** — verified
by R0: the inject-and-expect-discovery tests at cli.rs ~444/464/481/527/558/592/675 and
preview_cross_lang.rs ~118. **Do NOT touch the 2 survivors** (R0 N2 — find-replace
footgun): cli.rs ~613 (empty-dir "preview skipped" test — relies on NO sidecar, must stay
`$PATH`-free) and ~648 (no-`--preview` test). For any test whose fake was reached only
via `$PATH` and that also `env_remove`s nothing, ensure `$PATH` isn't still needed and
refresh now-stale comments (R0 N4). This is why the spec does NOT claim "no behavior
change beyond discovery": the discovery contract changes, and the test-injection vector
changes with it. Verify the full suite green AFTER migration.

## D2 — explicit opt-in (`ME_PREVIEW_BIN` env var, and/or `--preview-bin` flag)

When `ME_PREVIEW_BIN` is set to an existing file, that path is used (highest precedence,
before the exe-adjacent check) — the user has explicitly vouched for it.

**Where the env read lives (R0 L1):** keep `locate_in` a PURE function over already-known
paths (it returns `Option`, so it cannot express a fail-loud error). Read the
`ME_PREVIEW_BIN` env var and check existence in the WRAPPER (`locate_sidecar` or a small
step in `wire_previews`) BEFORE the version gate, where set-vs-unset can be distinguished:
- unset → co-located-only via `locate_in(exe_dir, None)`.
- set + file exists → use it (pass as `explicit` to `locate_in`).
- **set + file missing → fail loud: `EXIT_USAGE` (2)** with a distinct message naming the
  path (the user asked for a specific binary that isn't there; silently degrading or
  falling back to exe-adjacent would be surprising). Distinguishing set-vs-unset is
  required for a correct message anyway, so failing loud costs nothing.

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
D1 (remove fallback, with the `locate_in` refactor + the existing-test migration to
`ME_PREVIEW_BIN` so the suite stays green) → D2 (opt-in precedence + set-but-missing
error) → D3 (docs). TDD: the D1 unit test is a new-function compile-red; the behavioral
`planted_path_sidecar_ignored` integration test is the real red that locks the closure
(current code uses the `$PATH` binary). D2 tests are new behavior. Full verification:
`ME_REQUIRE_GO=1 cargo test --locked` at root (all green incl. new discovery tests) +
`go test ./...` in preview/ still green + clippy clean + a manual e2e: with a co-located
sidecar `me bundle --preview` works; after removing it and planting one on `$PATH`,
`me bundle --preview` now prints "preview skipped" (does NOT use the $PATH binary);
`ME_PREVIEW_BIN=<that>` makes it render again.

## Open questions — adjudicated at R0 round 0 (all 4 drafts endorsed)
1. Escape hatch → **`ME_PREVIEW_BIN` env var only** (smaller surface; doubles as the
   test-migration vehicle; a `--preview-bin` flag is additive later).
2. Set-but-missing → **error-exit `EXIT_USAGE` (2)**, distinct message, env read in the
   wrapper before the version gate (L1).
3. Dropping `$PATH` → **acceptable, not over-reach** — fail-closed default is right even
   for LOW; happy path preserved; only the uncommon bare-install+`$PATH`-sidecar setup
   regresses, with `ME_PREVIEW_BIN` as the clear replacement + graceful note.
4. Symlink canonicalization → **out of scope** (an attacker who can symlink next to
   installed `me` can plant the real binary — buys nothing). Version gate stays unchanged
   (co-located-only makes its spoofability moot).
