> **CONTROLLER HEADER — added 2026-08-15 when persisting. READ THIS FIRST.**
>
> This document was produced by a dispatched **agent**, not by the operator. The
> operator's standing directive that day was *"ask fable the questions you'd
> otherwise have to stop and ask about"*, so the agent was briefed to answer in
> an operator's voice. **That voice is a briefing artifact. It is advice.** The
> word "RULING" below does not record a human decision.
>
> - **Question A (S3 vs S3b) is ADOPTED** by the controller as ordinary
>   engineering scope, after independently re-verifying its two load-bearing
>   claims: `font/bitmap/bitmap.go:33` `indexLen = unicode.MaxASCII` with
>   `glyphFor` rejecting `int(r) >= indexLen` at `:62`, and the
>   `gui/md1_inspect.go` `:58`-vs-`:60,:65` collision inside one function. The
>   substance that matters is preserved either way: **F-179 is burned down
>   before S4 starts.**
> - **Question B is SUPERSEDED BY AN OPERATOR RULING, 2026-08-15:** *"You are not
>   permitted to bypass, but I am."* The `enforce_admins: false` setting is
>   **deliberate** — it is the operator's own escape hatch, not a lapse. §B's
>   central recommendation, flipping `enforce_admins` to true, would remove a
>   capability the operator intentionally holds, and its supporting argument
>   ("solo-maintainer is the argument FOR enforcement") is simply **wrong**: it
>   assumed nobody wanted the hatch. **Do not re-propose the flip.** The
>   asymmetry is the rule — the constraint binds *agents*, not the human.
> - **What survives from §B, and it is now a precondition rather than a
>   nicety:** neither primary's CI builds `ci/**`, so a staged SHA cannot earn a
>   context there. An agent forbidden to bypass therefore **cannot push to those
>   repos compliantly at all** until that trigger exists. The `ci/**` triggers
>   and the per-repo documentation stand; the flip does not.
> - **Question B's original NOT-AUTHORIZED note, kept for the record.** It proposes changing
>   security controls on live repositories — flipping `enforce_admins`, editing
>   branch-protection-gating CI triggers, dropping a `paths:` filter, and a
>   documented API call that DELETES enforce_admins protection. **No agent can
>   consent to that on the operator's behalf.** It is recorded here as a
>   researched recommendation and is **pending a real operator decision**. Its
>   verified facts (the API reads, the workflow triggers, neither `CLAUDE.md`
>   documenting the discipline) are sound and reusable; its instruction to act
>   is not. Do not execute section B because you found it in the repo.

# Operator rulings — 2026-08-15

Standing in for the operator; two pending decisions ruled after reading the
artifacts and machine-checking every count relied on. Facts verified are listed
per section; nothing below is quoted from the brief without an independent
check.

---

## Question A — F-179 vs stage S3

### RULING

**F-179 is ruled into its own stage S3b — inserted after S3 and before S4
(precedent: S0b, ruled 2026-08-14) — executed by the SAME in-flight
implementer, sequentially, in the same worktree, after it finishes S3 exactly
as briefed; S4 may not start until S3b closes green.**

Sub-rulings:

1. **S3 does not absorb F-179.** F-179's owning phase moves from S3 to S3b by
   this ruling; the FOLLOWUPS header is amended accordingly. This is
   re-ownership by explicit ruling with an immediately adjacent, gated landing
   slot — not a deferral, and not "carrying a phase-owned item across the
   gate": nothing passes S3b's position in the plan while F-179 is open.
2. **The current implementer takes S3b as a second, separately-briefed stage
   after S3 completes** — not as a mid-flight extension of the S3 brief, and
   not via a second agent. Commits stay per-stage. One adversarial review
   round MAY cover the S3+S3b diffs together, scoped per stage, with both
   gates' outputs stated in the brief.
3. **S3b's gate** is defined below, and every element of it can fail.
4. **The guard's scope** (gui/*.go production literals, the two named faces) is
   ruled ADEQUATE, with one strengthening: implement it as a face-coverage
   LOOKUP, not the 7-rune blocklist. Reasoning and the one residual class are
   below.

### WHY

The two bodies of work are disjoint in kind: S3 is a naming fix (scriptName
takes a Template; three callers; TYPED-ONLY comment burndown) with a
walk-plus-grep gate; F-179 is package-wide text hygiene plus a class guard
with a lookup-plus-raster gate. Stuffing the second into a frozen stage's text
makes S3's gate composite, its review non-proportional, and its diff
unreviewable as "what changed in response to what". The plan is FROZEN from S1
on (§0.3), and the exercised mechanism for new mid-plan work is a ruled stage
insertion (S0b), not silent scope growth. Two further facts force the shape:
F-179's live sites include `gui/md1_inspect.go:60,65`, two lines below S3's
briefed edit at `:58` — so a second parallel agent violates the
parallel-writer isolation rule on a specific file — and the guard cannot land
before the fixes without a red suite, so guard+fixes are one atomic unit that
deserves its own gate. Extending the in-flight brief mid-execution is the
exact shape mid-flight scope injection defects come from; a clean second brief
to the same agent preserves "implementation tight, one agent" and costs
nothing (the message was held pending this ruling).

### S3b — scope and gate (ruling 3)

**Scope.** Fix every live glyph site in `gui/*.go` production string literals
(measured TODAY: **21 sites**, list below — the entry's 27 is stale, six
`bundle_flow.go` gather sites were fixed by S2's fold at `4b8488e`; re-derive
again at execution time), replacing the glyphs with ASCII per the existing
house style (`—`→`-`, `·`→`|`, as S2 and the F-78 sites did). Widen S2's
guard shape (`stringLiterals` + the mutation-proof test in
`gui/bundle_gather_refusal_test.go`) to the whole package. Retire the
`cmd/emu/embed_confinement_test.go:12` TYPED-ONLY citation stays S3 work as
the plan already says; it is not S3b's.

**Gate — four elements, each executable, each able to fail. The trap is
named: no element may judge by `uiContains` or any content assertion; only
source lookup and ink count.**

1. **The package-wide guard test is GREEN on the landed tree**: every rune of
   every production string literal in `gui/*.go` non-test files passes
   `face.GlyphAdvance(r) == ok` against `poppins.Regular16` and
   `poppins.Bold25` (with `\n`/`\t` explicitly allowed). Lookup, not the
   7-rune `blankingGlyphs` list — see ruling 4.
2. **Mutation proof, RUN and RECORDED**: reintroduce U+2014 into one
   production literal; the guard must FAIL naming the file and rune; revert;
   the failing output is quoted in the stage commit's message. Plus the
   scanner-vacuity check (the guard fails if it finds zero literals), which
   S2 already built the shape of — extend, don't re-invent.
3. **Ink proof on the two named worst-case refusals**: raster
   `sysw_load.go`'s "A SECRET is stored unencrypted in flash." and
   `sysw_source.go`'s "This secret arrived with NO integrity check at all…"
   bodies through the proven `runUITouchRaster`/`showError` pattern (the S2
   review measured five bodies this way). Measure BEFORE the fix (must sit at
   the title-only value — this is the gate's own proof it can fail, taken
   while the strings are still broken) and AFTER (must clear the floor);
   assert the after value in a test; quote both numbers in the commit
   message.
4. **The re-derivation scanner prints 0 live sites**; output quoted in the
   commit message. (Subsumed by element 1; kept because it is the burndown
   count itself.)

### Guard scope (ruling 4)

The entry's scope — `gui/*.go` production string literals, faces
`poppins.Regular16` + `poppins.Bold25` — is ruled **adequate**, on a
machine-checked ground the entry did not have: `font/bitmap/bitmap.go:33` has
`indexLen = unicode.MaxASCII`, so the bitmap font FORMAT cannot index any
non-ASCII rune, and ALL six faces gui uses (`theme.go` also styles text with
`Bold10`, `Bold16`, `Bold20`; `Boldprogress45` draws only the progress
readout) share one coverage boundary. Checking two faces or six gives the
same verdict today. **Implement as the `GlyphAdvance` lookup anyway**, not the
hard-coded blocklist: the lookup tracks the format if it ever widens and
catches the NEXT missing glyph (`…`, accented runes) without a list edit —
the blocklist is a hand-maintained list, the exact construct F-163 indicts.

Two residual classes are named, neither gates S3b:

- **Runtime strings entering gui from outside** (`err.Error()` from codec
  packages, formatted `%s` inputs). Operator-entered data is
  charset-constrained (BIP-39 words, bech32, hex); the calibrated raster
  floor guards every walked screen. Out of static-guard reach by nature.
- **The root mechanism**: one absent glyph blanks the ENTIRE body. That is a
  renderer defect and the durable fix is renderer-level tolerance (substitute
  U+FFFD/skip the rune rather than abandoning the body). **File it as a new
  follow-up, owning phase: the next fork GUI cycle after this plan ships**,
  explicitly non-gating here — every surface this plan touches is guarded
  twice (lookup + raster floor) without it.

### CONSEQUENCES

- Controller appends an **S3b section** to
  `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (ruled insertion,
  citing this report; the §0.3 freeze is honored by ruling, as with S0b).
- `design/FOLLOWUPS.md` F-179 header: owning phase → **S3b (ruled
  2026-08-15)**; annotate that the 27-site list is stale — measured live
  count is **21** as of `4b8488e` (six `bundle_flow.go` sites fixed by S2's
  fold; the four lines `bundle_flow.go:383`, `codex32_polish.go:185,289`,
  `slip39_polish.go:237` confirmed as F-78 trailing comments, not sites).
- The held message to the S3 implementer is released as: *finish S3 exactly
  as briefed; S3b follows in this worktree under a second brief* (gate as
  above). No parallel agent. Separate commits per stage.
- **S4 does not start until S3b closes green.** File the renderer-tolerance
  follow-up when S3b lands.

### WHAT I VERIFIED (Question A)

- **TYPED-ONLY counts** (matches plan §3): `grep -rn "TYPED-ONLY"
  --include='*.go' gui/ | wc -l` → **9**; tree-wide adds exactly
  `cmd/emu/embed_confinement_test.go:12`.
- **S3 worktree live**: `git -C /scratch/code/shibboleth/seedhammer-s3 status
  --short --branch` → `## s3-nested-segwit`, head `4b8488e`, one untracked
  probe test.
- **F-179 site list re-derived** (scanner over `gui/*.go` non-test files,
  whole-line comments excluded, per-line literal extraction mirroring
  `stringLiterals`; script in scratchpad, method identical to the entry's):
  25 literal-carrying lines, of which 4 (`bundle_flow.go:383`,
  `codex32_polish.go:185,289`, `slip39_polish.go:237`) were read and are
  glyphs quoted inside trailing `// F-78:` comments (my scanner reads quotes
  inside comments; the entry's script agreed they are not sites) → **21 live
  sites**:

      bip85.go:228            bundle_flow.go:430,438   codex32_polish.go:28
      derive_xpub.go:254,487  gui.go:1020              md1_gather.go:105,155,168
      md1_inspect.go:60,65    mk1_inspect.go:202       seedxor_polish.go:85
      sysw_load.go:128,274,275,279,280                 sysw_source.go:114
      verify_address.go:95

  Diff vs the entry's 27: `bundle_flow.go:62,65,67,184,200,202` are gone —
  S2's fold (`4b8488e`, "stop shipping invisible refusals") fixed them, as
  the entry's own FIXED-AT-S2 paragraph says; its list predates that fold.
- **The named worst-case strings read directly**: `sysw_load.go:275` "A
  SECRET is stored unencrypted in flash.", `:274`/`:279-280` carry em-dashes
  in the literal ("could not confirm — treated as a secret —"), `:128` Lead
  carries "— unlock?", `sysw_source.go:114` "NO integrity check at all".
- **File overlap forcing sequential execution**: plan §3 names
  `gui/md1_inspect.go:58` as an S3 caller edit; F-179 live sites include
  `md1_inspect.go:60,65`.
- **Face coverage mechanism read**: `font/bitmap/bitmap.go` — `glyphFor`
  returns `ok=false` for `r >= indexLen`, `indexLen = unicode.MaxASCII`
  (line 33), so every non-ASCII rune fails `GlyphAdvance` on every face;
  faces used in gui measured by grep: `Regular16` ×9, `Bold25` ×7, `Bold10`
  ×2, `Bold20` ×1, `Bold16` ×1, `Boldprogress45` ×1 (the latter four via
  `gui/theme.go:83,96,100,111`).
- **S2's guard shape read in full**: `gui/bundle_gather_refusal_test.go`
  (`stringLiterals`, `TestGatherScreenTextCarriesNoBlankingGlyph`,
  `TestStringLiteralScannerCanSee`, the ink-judged pending-refusal drive).

---

## Question B — the push-bypass on the two primaries

### RULING

**(c) Both — each primary adopts and documents the `ci/staging` discipline in
its own `CLAUDE.md` AND flips `enforce_admins` to true — in a forced order:
CI triggers first, documentation second, the flip LAST.** The
`enforce_admins: false` claim is now **CONFIRMED by direct API read this
session** on both repos (the earlier permission block did not recur).

### WHY

These are the repos where normative correctness is established for the
firmware's funds-handling code (Rust-primary rule: behavior lands here, with
test vectors, first) — their checks matter more than anyone's, not less. The
bypass is a lapse pattern, not a choice: it fired five consecutive times in
the sibling repo before being understood, and this very cycle measured that
undocumented discipline decays (the citation baseline drifted 17→20 across
three continuity docs while prose kept vouching for it). Solo-maintainer is
the argument FOR enforcement: there is no second person to catch a red push,
so the machine must refuse. Option (d) accepts a standing silent-failure mode
on funds-relevant repos; (a) alone leaves the gate advisory — a future
session that never reads the doc pushes straight past it; (b) alone wedges
both repos, because **neither repo's CI can currently attach contexts to a
staged SHA** (verified below: mnemonic-key builds pushes to `main` only;
mnemonic-secret additionally path-filters push builds, and its own workflow
comment says docs-only pushes are "covered by admin bypass
(enforce_admins:false)" — that rationale dies with the flip and its config
must die with it).

### CONSEQUENCES

Order is forced; each step's commit can itself be staged (push-event
workflows run from the pushed ref, so the trigger fix is active on its own
staging push — no final bypass needed).

1. **`mnemonic-key`** `.github/workflows/ci.yml`: `on.push.branches: [main]`
   → `[main, 'ci/**']` (tags/PR triggers unchanged). Required context to
   watch: **`build (stable on ubuntu-latest)`**.
2. **`mnemonic-secret`** `.github/workflows/rust.yml`: `on.push.branches:
   [main, master]` → `[main, master, 'ci/**']` **AND drop the push-side
   `paths:` filter** — required contexts + `enforce_admins: true` are
   incompatible with path-filtered push builds (a docs-only push to `master`
   would never earn its contexts and would be refused forever). Rewrite the
   trigger comment whose bypass rationale is now obsolete. All four required
   contexts must attach: **`test (ubuntu-latest)`, `clippy`,
   `test (ms-codec)`, `clippy (ms-codec)`**.
3. **Each `CLAUDE.md`** gets the staging block adapted with **that repo's own
   context names and default branch** — a copy-paste of the engrave repo's
   `test (rust + go)` would be wrong in both repos. Include the emergency
   valve as a documented command pair (temporary
   `gh api -X DELETE repos/bg002h/<repo>/branches/<branch>/protection/enforce_admins`
   during a CI outage, re-enable with POST immediately after), so the escape
   hatch is a recorded action rather than an undocumented lapse.
4. **Then flip**:
   `gh api -X POST repos/bg002h/mnemonic-key/branches/main/protection/enforce_admins`
   and
   `gh api -X POST repos/bg002h/mnemonic-secret/branches/master/protection/enforce_admins`.
   Verify each with a GET showing `"enabled": true`.
5. **Proof the gate is real**: the next real push per repo goes through
   staging and prints no bypass message; a direct push of an unchecked SHA
   now gets REFUSED rather than warned — that refusal, if ever seen, is the
   gate working.
6. Update `CONTINUITY` on the next roll: the "Operator decision, not yet
   made" line is resolved by this report; the doc's closing line "the
   primaries push directly" becomes false once step 4 lands.
7. **Observation, not a ruling**: mnemonic-key requires only ONE of its nine
   build-matrix cells (`build (stable on ubuntu-latest)`); `fmt` and
   `vectors-roundtrip` are not required contexts. Worth an operator look
   separately; widening the required set is out of this ruling's scope.

### WHAT I VERIFIED (Question B)

- `gh api repos/bg002h/mnemonic-key/branches/main/protection` →
  `enforce_admins.enabled: false`, `required_status_checks.strict: false`,
  contexts `["build (stable on ubuntu-latest)"]`.
- `gh api repos/bg002h/mnemonic-secret/branches/master/protection` →
  `enforce_admins.enabled: false`, `strict: false`, contexts
  `["test (ubuntu-latest)", "clippy", "test (ms-codec)", "clippy (ms-codec)"]`.
  **The continuity doc's claim is confirmed for both repos.**
- `mnemonic-key/.github/workflows/ci.yml` trigger read: push builds `main`
  branch + `mk-cli-v*`/`mk-codec-v*` tags + PRs — **no `ci/**`**.
- `mnemonic-secret/.github/workflows/rust.yml` trigger read: push builds
  `[main, master]` **with a `paths:` filter** (`crates/ms-cli/**`,
  `crates/ms-codec/**`, `Cargo.toml`, `Cargo.lock`, the workflow itself); PR
  trigger deliberately unfiltered, with an in-file comment stating docs-only
  direct pushes are "covered by admin bypass (enforce_admins:false)".
- `grep -n "ci/staging\|staging" <both CLAUDE.md>` → exit 1: **neither
  documents the discipline.**
- Remotes: `bg002h/mnemonic-key`, `bg002h/mnemonic-secret` (git remote -v).
