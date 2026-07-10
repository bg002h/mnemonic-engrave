# R0 architect review — SPEC_me_sidecar_discovery (Cycle D / F11), ROUND 1

Reviewer: opus R0 architect (independent), round 1
Date: 2026-07-09
Spec under review: `design/SPEC_me_sidecar_discovery.md` (branch `me-sidecar-discovery`,
HEAD `f2d7213`, parent/base `4908dbb` = merged master, Cycles A/B/C landed)
Round-0 review: `design/agent-reports/me-sidecar-discovery-spec-R0-round0.md`
(0C / 1I / 1L / 3N — ALL folded 2026-07-09)
Standard: GREEN = 0 Critical / 0 Important before implementation.

Verdict: **GREEN — 0 Critical, 0 Important, 0 Low, 6 Nits (all advisory).**

Round-0's blocker (I1) and the Low (L1) are both substantively CLOSED. I verified the
migration numerically and per-test against the actual tree, traced the folded wrapper
control-flow against merged Cycle A, and did a fresh adversarial pass on the
precedence/version-gate interaction and post-migration coverage. The residual items are
documentation-precision and test-hardening nits that do not gate implementation.

---

## 1. I1 closure — the `$PATH`-dependent test migration (round-0 Important)

**CLOSED.** The spec now (§D1, lines 71–78) adds the MANDATORY migration of the seven
`mod preview` tests + the cross-lang test off `$PATH` and onto `ME_PREVIEW_BIN`, and
corrects the "all green"/"no behavior change beyond discovery" framing (line 77–78 spells
out that the discovery contract *and* the test-injection vector change).

**Count verified against the current tree (adversarial).** `grep '\.env("PATH"'` in
`cli.rs` returns 9 hits. Classifying each by whether it *injects a sidecar and expects
discovery*:

| line | test | injects+expects discovery? |
|------|------|----|
| 444 | `empty_sidecar_output_exit_4` | YES (fake_empty_output) |
| 464 | `render_failure_exit_4` | YES (fake_render_fail) |
| 481 | `matched_version_renders_and_sets_preview_exit_0` | YES (fake) |
| 527 | `png_flag_renders_png` | YES (fake) |
| 558 | `dirty_preview_dir_refused_exit_2` | YES (fake) |
| 592 | `mismatched_version_exit_2` | YES (fake, wrong ver) |
| 675 | `unwritable_preview_dir_exit_2` | YES (fake) |
| 613 | `absent_sidecar_degrades_exit_0…` | NO — **empty** bindir, relies on absence → `None` |
| 648 | `no_preview_flag_is_byte_for_byte_phase_a` | NO — fake present but no `--preview`, so `locate` is never called |

Exactly **7** inject-and-expect. Plus `preview_cross_lang.rs:118` (prepends `bindir` to
`$PATH` for the *real* sidecar) = the "+1". **The spec's "seven … and one" is correct.**
The two survivors are correctly excluded (they do not inject via `$PATH` for discovery).

**Migration validity, per test (does each still exercise its intent under `ME_PREVIEW_BIN`?):**
- `mismatched_version_exit_2` — the load-bearing case: an *explicitly-vouched* binary must
  still hit the version gate. Spec line 98–99 states the gate is UNCHANGED and an explicit
  binary must still pass `--version` match; flow `explicit → version gate → found≠expected →
  EXIT_USAGE(2)`. Still exit 2. ✔
- `dirty_preview_dir_refused_exit_2` — version MATCHED via `ME_PREVIEW_BIN`, so control
  reaches the Cycle-A dirty-dir scan (past locate+version) and exits 2 on `plate-9.svg`. ✔
- `empty_sidecar_output_exit_4` / `render_failure_exit_4` — explicit → version match →
  render → `EmptyOutput`/`Render` → exit 4 (F9 / §6). ✔
- `matched_version…` / `png_flag…` — explicit → version match → renders → exit 0; this pair
  doubles as D2's `ME_PREVIEW_BIN` happy-path coverage, as the spec intends (line 76). ✔
- `unwritable_preview_dir_exit_2` — adversarial check: `ME_PREVIEW_BIN` points at the
  *existing* fake (bindir/me-preview); the MISSING path is the separate `--preview` target
  dir. So set-but-missing does **not** misfire; flow `explicit(exists) → version match →
  !dir.is_dir() → EXIT_USAGE(2)`. Still exit 2, and correctly not confused with a missing
  `ME_PREVIEW_BIN`. ✔

**No test relies on `$PATH` for a non-injection reason that would be wrongly migrated** —
the two `$PATH` uses that are *not* injection (`absent_sidecar_degrades`, `no_preview_flag`)
are outside the migration set. (See R1-N2 for a hardening nit against a blind find-replace.)

`locate_sidecar` has a single non-comment caller (`main.rs:236`) and **no existing direct
unit test**, so the `locate_in` refactor is purely additive — it cannot break a hidden
caller, and the preview unit tests (`sidecar_version`, `render_plate`, …) call their targets
directly and are untouched by the discovery change.

## 2. L1 closure — set-but-missing fail-loud vs pure `locate_in` (round-0 Low)

**CLOSED.** The fold (§D2 lines 85–94 + open-question-2 lines 128–131) makes the design
coherent: `locate_in` stays a pure `-> Option`; the `ME_PREVIEW_BIN` env read + existence
check live in the WRAPPER, BEFORE the version gate, with three enumerated cases (unset →
`locate_in(exe_dir, None)`; set+exists → pass as `explicit`; set+missing → **`EXIT_USAGE(2)`**
distinct message).

**Traced against the actual merged `wire_previews`** (`main.rs:227–329`): the set-but-missing
early return sits at the very top (at/replacing the locate step), returning `Some(EXIT_USAGE)`
— strictly *before* the version gate (245–259), the `dir.is_dir()` check (262–268), the
Cycle-A dirty-dir scan (275–297), and the render loop (300–327). It does **not** collide with
any Cycle-A structure: the `None` graceful-degrade (unset+not-found → exit 0) is preserved,
and `EmptyOutput`/dirty-dir behavior is untouched. `wire_previews` already returns
`Option<i32>`, so `Some(EXIT_USAGE)` is the natural carrier. (One wording nit — R1-N5.)

## 3. N1–N3 closure (round-0 Nits)

- **N1 (stale line numbers) — PARTIALLY closed.** Recon line 20 now correctly cites the
  `$PATH` arm at `~:81-89` and the doc adds "line numbers approximate, locate by symbol."
  BUT §D1 line 56 still says *"Delete the `$PATH` scan (preview.rs:76-83)"* — stale and
  internally inconsistent with line 20: the actual scan (comment + block) is lines **81–89**;
  76 is inside the exe-adjacent arm's `return Some(cand);` and 83 is mid-`$PATH`-loop. See
  R1-N1.
- **N2 (stale merge-adjacency) — CLOSED.** The future-tense "whichever merges second may
  rebase" paragraph is gone; lines 11–14 now state past-tense that A/B/C are landed on
  `4908dbb` and D's surface is disjoint (confirmed).
- **N3 (behavioral companion + "red today" phrasing) — CLOSED.** §D1 lines 66–69 add
  `planted_path_sidecar_ignored` (fake ONLY on `$PATH`, no co-located, no `ME_PREVIEW_BIN` →
  exit 0 + "preview skipped" + no preview keys); lines 117–119 correctly distinguish the
  `locate_in` compile-red from the integration behavioral-red. I confirmed the behavioral
  test is non-vacuously red today (current code scans `$PATH`, finds the fake, renders → previews
  present, no skip note) and green after D1.

## 4. Fresh adversarial pass

- **`ME_PREVIEW_BIN` precedence vs version gate — correct and stated.** An explicit binary
  takes highest precedence for *discovery* but does NOT bypass the version gate (spec line
  98–99). Verified: `mismatched_version_exit_2`, once migrated to `ME_PREVIEW_BIN`, still
  exits 2 — it is exactly the regression lock proving explicit-selected ≠ version-exempt. No
  bad interaction.
- **Post-migration coverage of the co-located (exe-adjacent) branch — NOT a regression.**
  Before this cycle the exe-adjacent arm had **zero** integration coverage (every preview
  test always went through the `$PATH` arm). After migration they go through the *explicit*
  arm. So no integration coverage is *removed*; and the spec-mandated pure-`locate_in` unit
  test is a net *improvement* — acceptance bullet "with a `me-preview` next to the current
  exe → discovered" (line 60) maps to `locate_in(Some(dir_hit), None) == Some(...)`. The one
  branch left genuinely untested is **explicit-wins-over-a-present-exe-adjacent** precedence
  (no test sets both) — correct-by-construction in the pure function, minor. Recommend
  enumerating the unit matrix (R1-N3). This does not gate — it is a Nit.
- **Graceful note is no longer misleading.** Round-0 flagged that "preview skipped (install
  me-preview)" is wrong when the user pointed at a bad path; the set-but-missing fail-loud
  branch (distinct message, exit 2) removes that footgun.

## 5. Scope / executability / TDD integrity

Single-implementer executable: D1 (`locate_in` refactor + `$PATH` deletion + migrate 7+1 +
add `planted_path_sidecar_ignored`) → D2 (env read + precedence + set-but-missing EXIT_USAGE)
→ D3 (docs). TDD ordering is sound (compile-red unit + behavioral-red integration + new D2
behavior). No bleed into D5-1/-2/-3 (Cycle A). Verification command (`ME_REQUIRE_GO=1 cargo
test --locked` + `go test ./...` + clippy + manual e2e) is appropriate and now truthful
post-migration.

---

## FINDINGS (all Nits — advisory, none gates GREEN)

**R1-N1 (Nit) — residual stale line ref.** §D1 line 56 cites the `$PATH` scan as
`preview.rs:76-83`; the actual block is `~81-89` (as recon line 20 correctly states).
Fix: change to `~:81-89` (or reference "the `// 2) On $PATH` block"). Minor: recon line 18's
`locate_sidecar (preview.rs, ~:62)` is really `:68` — flagged "~/approximate" so acceptable,
but could refresh.

**R1-N2 (Nit) — guard the blind find-replace.** The spec says migrate "seven … that inject
via `$PATH`" but never *names* the two survivors nor warns that migrating
`absent_sidecar_degrades` (cli.rs:613) to `ME_PREVIEW_BIN=<empty-bindir>/me-preview` would
flip it from exit 0 (graceful `None`) to exit 2 (set-but-missing). A careless
find-replace of all 9 `.env("PATH")` would break it. Self-correcting via the suite-green gate
(the test would fail), hence Nit — but naming the two to LEAVE ALONE (613, 648) hardens it.

**R1-N3 (Nit) — enumerate the `locate_in` unit matrix.** Spell out the cases so precedence
and the exe-adjacent happy path each get a unit test: (a) exe_dir hit → `Some`; (b) exe_dir
miss + no explicit → `None`; (c) explicit set + exe_dir *also* hit → **explicit wins**
(the only otherwise-untested branch after migration); (d) explicit passed only when the
wrapper already confirmed existence. Closes the "co-located happy path / precedence untested"
concern crisply.

**R1-N4 (Nit) — test hermeticity + stale comments.** The two UNSET-reliant tests
(`planted_path_sidecar_ignored`, `absent_sidecar_degrades`) should `.env_remove("ME_PREVIEW_BIN")`
so an ambient `ME_PREVIEW_BIN` in the runner env cannot perturb them. Also refresh the now-
stale comments: the `mod preview` module doc (cli.rs:312–318, "PATH-only discovery is
deterministic") and `absent_sidecar_degrades`'s "empty bin dir on PATH → None" (cli.rs:607) —
absence now means "no `ME_PREVIEW_BIN` and no co-located sidecar."

**R1-N5 (Nit) — pin the fail-loud return site.** §D2 line 87 offers "the WRAPPER
(`locate_sidecar` or a small step in `wire_previews`)" — but the set-but-missing
`EXIT_USAGE(2)` cannot originate from an `Option`-returning `locate_sidecar`. Pin the env-read
+ missing-check (the branch that returns a code) to `wire_previews` (or a helper returning
`Option<i32>`/`Result`); `locate_sidecar` may still read the env for the success/`None` cases.
A competent implementer converges here anyway (Option can't carry the exit), so Nit.

**R1-N6 (Nit) — tighten D2 acceptance to the adjudicated decision.** §D2 acceptance line 104
still reads "`ME_PREVIEW_BIN=/nonexistent` → error/graceful per R0 decision; test pins
whichever," but R0 already adjudicated **`EXIT_USAGE(2)`** (lines 91, 130). State it
definitively and mandate the set-but-missing test (exit 2 + distinct message naming the path)
so the acceptance criterion is unambiguous.

---

## Verdict

**GREEN — 0 Critical, 0 Important, 0 Low.** Round-0 I1 and L1 are closed and verified
against the tree; N2/N3 closed; N1 has a small residual (R1-N1). The 6 nits are advisory
hardening/precision items — none blocks implementation. Proceed to the single-implementer TDD
build. Recommend the implementer fold R1-N1..N6 opportunistically (especially R1-N2 and
R1-N5, which prevent a footgun and a dead-end during coding).
