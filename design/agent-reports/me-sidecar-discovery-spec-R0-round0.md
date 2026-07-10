# R0 architect review — SPEC_me_sidecar_discovery (Cycle D / F11), round 0

Reviewer: opus R0 architect (independent)
Date: 2026-07-09
Spec under review: `design/SPEC_me_sidecar_discovery.md` (branch `me-sidecar-discovery`)
Branch base: `4908dbb` (origin/master; merged Cycles A/PR#1, B/PR#2, C/PR#3)
Standard: GREEN = 0 Critical / 0 Important before implementation.
Verdict: **NOT GREEN — 0 Critical, 1 Important, 1 Low, 3 Nits.**

The design choice is sound and it genuinely closes the finding. The one blocker is a
verification/test-migration gap: the existing preview test suite is built on the very $PATH
mechanism this spec removes, so `cargo test` cannot be "all green" as the spec claims until
those tests are migrated — and the spec does not mention them at all.

---

## 1. Is the design choice sound? (adjudication)

**Yes.** Co-located-only auto-discovery + an explicit `ME_PREVIEW_BIN` opt-in is the correct
resolution, and it is a real improvement over both rejected alternatives.

- **Does it close the finding?** Yes. The finding (D5-4) is: with `me` installed WITHOUT a
  co-located sidecar, `locate_sidecar` walks `$PATH` and pipes the public md1/mk1 payload to
  the first `me-preview` found, which can also write attacker-controlled files into `--preview
  DIR`. Deleting the `$PATH` branch (`preview.rs:81-89`) means a `$PATH`-planted binary is
  **never reached by automatic discovery**. Verified against the current tree: `locate_sidecar`
  (`preview.rs:68-92`) has exactly two arms — exe-adjacent (72-79) and `$PATH` (81-89) — and
  the whole exposure is the second arm. Removing it is a complete closure of the automatic path.

- **Does `ME_PREVIEW_BIN` reintroduce ambient authority?** No. `$PATH` is *ambient* — any
  directory an attacker can write to that happens to precede the real sidecar on the victim's
  `$PATH` wins, with no user action. `ME_PREVIEW_BIN` is a *single explicit path the user
  themselves sets*. The reasoning in the spec holds: an attacker who can set the victim's
  `ME_PREVIEW_BIN` already controls the victim's process environment and can do strictly worse
  (override `$PATH` itself, `LD_PRELOAD`, shell-alias `me`, replace `me`). So the opt-in does
  not lower the trust floor; it raises it from "any writable `$PATH` dir" to "the user's
  explicit choice."

- **Does it break the documented happy path?** No. Release archives ship `me` + `me-preview`
  side by side (`.github/workflows/release.yml:347-355` copies both into one stage dir), which
  is discovery arm #1 (exe-adjacent), **untouched**. The only workflow that regresses is "install
  `me` bare, put `me-preview` somewhere on `$PATH`" — and `me-preview` is a Go binary not
  published to crates.io, so the canonical bare-install case (`cargo install mnemonic-engrave`)
  has no sidecar at all. The residual inconvenience is narrow, the graceful "preview skipped"
  note guides the user, and `ME_PREVIEW_BIN` is the supported replacement.

- **Alternatives correctly rejected.** Hash-pinning is brittle (per-platform sidecar hash → breaks
  every legitimate rebuild) for a LOW public-only finding; warn-only is not fail-closed (the
  discovery has already happened). Co-located-only + explicit opt-in is fail-closed by default and
  matches the tool's posture.

- **Graceful-degrade-on-None preserved?** Yes. `wire_previews` (`main.rs:236-242`) maps a `None`
  from `locate_sidecar` to the "preview skipped" note + `return None` (exit 0). D1 keeps that
  contract; only the *set of inputs* that yield `None` changes (a `$PATH`-only sidecar now → `None`
  instead of `Some`). Confirmed the spec's claim that "no caller change is needed for the graceful
  case."

**Version gate left UNCHANGED — correct.** The finding is about *discovery* (who gets reached),
not about the gate's integrity. Once discovery is co-located-only + explicit-opt-in, the only
binaries that ever run are ones placed next to your installed `me` (attacker needs write access
to your install dir → already game over) or ones you explicitly vouch for. So the spoofable
string-match gate (`preview.rs:100-114`, called at `main.rs:245-259`) is **moot** for integrity
and correctly stays as version-skew protection only. Leaving it spoofable is acceptable.

---

## 2. Executability

- **`locate_in(exe_dir, explicit)` pure-function refactor — correct and sufficient.** Extracting
  a pure precedence function `locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>) ->
  Option<PathBuf>` and having the thin `locate_sidecar()` wrapper supply
  `current_exe().parent()` and the `ME_PREVIEW_BIN` value is the right shape. It makes the
  discovery order unit-testable without depending on the real `current_exe()` (which in a test
  returns the test binary's path and cannot be controlled).

- **Can the D1 test assert "$PATH no longer used" deterministically? Yes** — because after D1
  `locate_in` has **no `$PATH` parameter and reads no env**, so `locate_in(Some(empty_dir),
  None) == None` is deterministic and hermetic regardless of any planted `$PATH`. The "$PATH is
  not consulted" property is then true *by construction* (the code path is gone), which is
  stronger than a dynamic assertion. See Nit N3 on the "red today" phrasing and a recommended
  behavioral companion test.

- **D2 precedence + set-but-missing interaction — clean, with one caveat (Low L1).** Precedence
  `explicit` → exe-adjacent → `None` composes fine with the version gate and the None-degrade:
  an explicitly-chosen binary still flows through `sidecar_version` (must match) and the
  dirty-dir scan unchanged. The caveat is the set-but-missing return type — see L1.

---

## 3. Interaction with merged Cycle A — clean, no conflict

Cycle A (PR #1, in `4908dbb`) added, in `wire_previews`: the dirty-dir refusal
(`main.rs:270-297`), the `EmptyOutput` variant + `validate_render_output` signature gate
(`preview.rs:22-24, 181, 190-227`), and `write_private` 0o600 writes. **D's surface is disjoint
from all of it:**

- D edits `locate_sidecar`/`locate_in` (`preview.rs`) + the env read; it does **not** touch
  `render_plate`, `validate_render_output`, `EmptyOutput`, or the dirty-dir scan.
- In `main.rs`, D touches only the *locate* step (`236-242`) and (per L1) may add a
  set-but-missing error branch *before* the version gate. A's dirty-dir scan (270-297) and the
  version gate (245-259) run strictly after locate and are undisturbed. The None-degrade contract
  A relies on is preserved.

No duplication of A's validation, no textual conflict. The spec's "merge adjacency" note is now
moot (see Nit N2): PR #1 already merged and this branch is already rebased onto `4908dbb`.

---

## FINDINGS

### I1 (Important) — the entire existing preview test suite depends on `$PATH` discovery; D1 breaks 8 tests, and the spec neither migrates them nor acknowledges it (its "all green" / "no behavior change beyond discovery" claims are false as written)

`crates/me-cli/tests/cli.rs` `mod preview` stands up a fake `me-preview` shell script in a temp
dir and injects it **exclusively via `.env("PATH", &bindir)`** (module doc `cli.rs:312-318`:
"put that dir FIRST on PATH … PATH-only discovery is deterministic"). After D1 removes the
`$PATH` arm, `locate_sidecar` returns `None` for a `$PATH`-only sidecar, so every test that
expects the fake to be **discovered** now degrades to "preview skipped"/exit 0 and fails its
assertion:

1. `empty_sidecar_output_exit_4` (`cli.rs:444`) — expects exit 4; will get exit 0.
2. `render_failure_exit_4` (`:464`) — expects exit 4; gets 0.
3. `matched_version_renders_and_sets_preview_exit_0` (`:481`) — expects renders/previews; gets none.
4. `png_flag_renders_png` (`:527`) — expects a `.png` preview; gets none.
5. `dirty_preview_dir_refused_exit_2` (`:558`) — expects exit 2 (dirty-dir scan); degrades before reaching the scan → exit 0.
6. `mismatched_version_exit_2` (`:592`) — expects exit 2; a `$PATH` binary is no longer found → exit 0.
7. `unwritable_preview_dir_exit_2` (`:675`) — expects exit 2; gets 0.

Plus the cross-lang integration test:

8. `preview_cross_lang.rs` (`:104-118`) — explicitly prepends `bindir` to `$PATH` "and let
   discovery find the real sidecar on `$PATH`"; expects `.success()` with populated previews →
   after D1, no previews, assertion fails (gated on `ME_REQUIRE_GO=1`, which the spec's own
   verification command sets).

(Two tests survive: `absent_sidecar_degrades_exit_0…` (`:613`, empty `$PATH` → `None` either
way) and `no_preview_flag_is_byte_for_byte_phase_a` (`:648`, never calls locate).)

The spec's §Ordering & verification asserts `ME_REQUIRE_GO=1 cargo test --locked` is "all green
(incl. new discovery tests)" and the framing is "no behavior change beyond discovery" — but the
suite will be **red in 8 places** until these tests are migrated off `$PATH`. This is a required,
non-trivial implementation step (8 call sites) that the spec omits entirely; an implementer
following it verbatim produces a broken PR and cannot pass the stated gate.

**Fix (fold into the spec, then re-dispatch):**
- Add an explicit task (call it D1b or fold into D2): migrate each `$PATH`-injecting preview test
  to the new opt-in — replace `.env("PATH", &bindir)` with
  `.env("ME_PREVIEW_BIN", bindir.join("me-preview"))` (the D2 mechanism). This conveniently makes
  the existing suite double as `ME_PREVIEW_BIN` happy-path coverage.
- For `preview_cross_lang.rs`, set `ME_PREVIEW_BIN` to the built real sidecar path instead of
  prepending `$PATH`.
- Correct the two claims: verification is green only *after* the migration; "no behavior change
  beyond discovery" is true for production behavior but the **test harness's discovery vehicle
  changes**, which must be stated.
- Keep `absent_sidecar_degrades…` as-is (it already asserts the None/degrade path and still holds);
  consider renaming its comment (currently "empty bin dir on PATH") to reflect that absence now
  means "no `ME_PREVIEW_BIN` and no co-located sidecar."

### L1 (Low) — set-but-missing `ME_PREVIEW_BIN`: the draft's fail-loud choice is under-specified against the `locate_in -> Option<PathBuf>` signature

The draft (D2) wants a set-but-missing `ME_PREVIEW_BIN` to be an **error exit** (fail-loud), but
the D1 pure-function signature `locate_in(..) -> Option<PathBuf>` can only return `None`, which at
the `wire_previews` call site (`main.rs:236-242`) is the **graceful degrade** path (exit 0). So
"return `None` on missing" (D2 as literally written) silently contradicts "treat set-but-missing
as an error exit" (the draft's stated preference). This is not a defect the spec overlooked — it
explicitly flags it as open question #2 — but the mechanism for the fail-loud branch is not
specified, and a naive implementer will implement `None` → silent skip. Note also that even a
*graceful* choice needs the wrapper to know `ME_PREVIEW_BIN` was set, because the current skip
note is "preview skipped (install me-preview)" — misleading when the user *did* point at a (wrong)
path.

**Fix:** adopt the recommendation in §Open-question 2 below. Keep `locate_in` a pure precedence
`-> Option`. Read `ME_PREVIEW_BIN` in the thin wrapper (or `wire_previews`); if it is set and the
path is not a regular file, emit a distinct message (e.g. `me: ME_PREVIEW_BIN=<x> does not exist`)
and return `Some(EXIT_USAGE)` **before** the version gate. Spell this out in the spec so the
signature and the fail-loud intent are consistent. (Classified Low, not Important, because the
spec correctly deferred the decision and a graceful-degrade fallback keeps `-> Option` fully
executable with zero new plumbing — so it does not, by itself, block a buildable implementation.)

### N1 (Nit) — stale line-number citations

The recon cites `preview.rs:62-86` / `:76-83` / `:94-108` and `main.rs:238-241` / `:246-259`
(recon SHA `9fafb6b`), but the branch base is `4908dbb`. Current locations: `locate_sidecar`
68-92, `$PATH` arm 81-89, `sidecar_version` 100-114; `wire_previews` locate/None 236-242, version
gate 245-259. Structure is accurate; refresh the numbers so the plan/implementer cite the current
tree.

### N2 (Nit) — "merge adjacency" note is stale

The spec's merge-adjacency paragraph speaks in future tense ("whichever of PR #1 / this PR merges
second may need a trivial rebase"). PR #1 (Cycle A) has already merged into `4908dbb` and this
branch is already rebased onto it. Update the note to reflect that A/B/C are merged and the
adjacency risk is resolved (the `render_plate`/dirty-dir regions are disjoint from
`locate_sidecar`, confirmed in §3).

### N3 (Nit) — "genuine red today" phrasing + recommend a behavioral companion test

The spec says the D1 test "is a genuine red today (current code returns the `$PATH` binary)." The
*primary* D1 test targets the **new** pure `locate_in`, so its "red" is really "the function does
not exist yet" (a valid TDD red, but a compile-red, not a behavioral red against current code).
That's fine, but to lock the finding closure at the **real entry point**, also add one behavioral
integration red-test, e.g. `planted_path_sidecar_is_ignored`: set `.env("PATH", &bindir_with_fake)`
with **no** `ME_PREVIEW_BIN` and no co-located sidecar, assert exit 0 + "preview skipped" + **no**
preview keys. That test is behaviorally red today (the `$PATH` fake renders → previews present) and
green after D1 (fake ignored → previews absent) — a direct, non-vacuous regression lock on the
closure.

---

## Firm recommendations on the 4 open questions

1. **Escape hatch — `ME_PREVIEW_BIN` env var only (no `--preview-bin` flag this cycle).** Confirm
   the draft. The env var is sufficient, is the natural vehicle for migrating the existing tests
   (I1), and avoids defining flag-vs-env precedence prematurely. A `--preview-bin` flag is purely
   additive and can be added later if a real workflow demands it. (A path on argv is not a secret,
   so there's no leak argument either way — this is purely a surface-minimization call.)

2. **Set-but-missing `ME_PREVIEW_BIN` — error exit (fail-loud), `EXIT_USAGE` (2), with a distinct
   message.** Confirm the draft's preference, but nail the mechanism (L1): read the env var in the
   wrapper/`wire_previews`, and if set-and-not-a-regular-file, print `me: ME_PREVIEW_BIN=<x> does
   not exist` and return `Some(EXIT_USAGE)` *before* the version gate; keep `locate_in` a pure
   `-> Option`. Rationale: an explicit opt-in deserves an explicit failure (matches the version
   gate's own fail-loud posture at `main.rs:245-259`); silently skipping a user's explicit request
   — and printing "install me-preview" when they *did* point at a path — is a footgun. You need to
   distinguish "set-but-missing" from "unset" for a correct message regardless, so fail-loud costs
   nothing extra.

3. **Dropping the `$PATH` fallback — acceptable; NOT over-reach for a LOW finding.** Confirm the
   draft. Fail-closed-by-default is the right posture even for a LOW/public-only finding when the
   fix is this small and preserves the documented happy path (co-located release archive,
   unchanged). The only regressed workflow ("install `me` bare + `me-preview` on `$PATH`") is
   uncommon (`me-preview` isn't crates.io-installable) and has a clear, documented replacement
   (`ME_PREVIEW_BIN`) plus a graceful note. The behavior change is real but proportionate and
   well-mitigated.

4. **Symlink/relative-path canonicalization of the exe-adjacent path — out of scope.** Confirm the
   draft. It is gold-plating a LOW finding: the exe-adjacent directory is already the trust root,
   and an attacker who can plant a symlink next to your installed `me` can equally plant the real
   binary — canonicalization buys nothing against that threat and adds failure modes
   (`canonicalize` errors on odd mounts). Leave it out.

**Version gate unchanged — confirmed correct** (see §1): the finding is about discovery, not the
gate; co-located-only + explicit-opt-in makes the spoofable gate moot, so leaving it as
version-skew protection is the right call.

---

## Scope / TDD / no-bleed

- **No bleed into other findings.** D5-1 (stderr echo), D5-2 (0o600 perms), D5-3 (stale previews)
  are all closed by Cycle A; D touches none of them. D stays in `locate_sidecar`/`locate_in` +
  the env read + docs + test migration.
- **TDD integrity:** the pure-`locate_in` unit test is a sound deterministic lock; add the
  behavioral companion (N3). The D1 red is a new-function red, not a behavioral-red against the
  current `locate_sidecar` (N3).
- **Behavior change is confined to discovery** for production, but the **test harness's discovery
  vehicle changes** (I1) — the spec must say so.

---

## Verdict

**NOT GREEN.** 0 Critical, **1 Important (I1)**, 1 Low (L1), 3 Nits (N1–N3).

Fold I1 (add the test-migration step for the 8 `$PATH`-dependent tests → `ME_PREVIEW_BIN`; correct
the "all green"/"no behavior change" claims), resolve L1 per open-question-2 (fail-loud mechanism
spelled out), and sweep N1–N3. Re-dispatch after the fold (folds can drift). The design choice
itself is sound and closes the finding; the blocker is purely spec completeness around the tests
it silently invalidates.
