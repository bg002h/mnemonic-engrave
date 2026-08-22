# R0 re-review, round 2 — PLAN_wallet_file_export.md Phase 1 fold (cc5b2ed..bd6e316)

- **Reviewed:** 2026-08-22. Scope strictly: (1) did the fold fix round 1's eight
  findings, (2) did the fold introduce a new defect. Not a fresh audit.
- **Verdict: 0 Critical / 4 Important / 1 Minor / 2 Nit — the gate stays RED.**
- All four Importants are text fixes to the plan; none reopens a settled ruling.
  Machine-checked this round (not recalled): `to_ext_params` is a private `fn`
  at `descriptor_builder/gate.rs:61` (module distinct from the three pieces the
  fold names); the gate machinery is `Segwitv0`-typed (gate.rs:25,178,414);
  `allow_set`/`emit_allow_notes` are private `fn`s (build_descriptor.rs:158,175);
  §G1's purpose sentence was already softened at the review target (cc5b2ed
  carries "it does not unblock import into any of the three targets"), so C1
  has no un-propagated §G1 residue; "Phase 1 may start now" exists at cc5b2ed
  line 224 and is unchanged by the fold.

## Ledger — the eight findings

| ID | status |
| --- | --- |
| C1 | **Fixed.** Table row corrected, non-goal constraint added, purpose re-scoped to parity/inspection/archival. Import-as-acceptance is workable — see the direct answer below and N-2. |
| I1 | **Not fixed** — F-2. The fold schedules the implementer to make the decision round 1 said the plan must make. |
| I2 | **Partially fixed** — right for round 1's plan, understated under the fold's own I3 ruling. F-3. |
| I3 | **Partially fixed** — the ruling is correct; its breaking-change consequence and rule-set scope are stated nowhere. F-1. |
| I4 | **Partially fixed** — sub-part 3 unaddressed, and the fold deleted the baseline tests without restating them. F-4. |
| M1 | **Fixed** (decision stated with rationale; one rationale sentence contradicts the fold's own I4 text — N-3). |
| M2 | **Fixed** at Minor-appropriate resolution (export-specific wording committed; acceptance line present). |
| N1 | **Fixed in substance** (shared home + before-second-caller ordering, which addresses the decay mechanism better than a name would). Residue — module unnamed, `to_ext_params` omitted — folded into F-3. |

## Direct answers to the three pushed questions

1. **I3: the ruling is correct; the omission is the consequence.** Uniform
   policy is the right design — quirk passthrough gives a flag whose reach is
   an upstream parser accident, evaporates on a pin advance past #734, and pins
   a hole upstream itself calls broken. But yes: **uniform policy means wsh
   descriptors that export today start requiring a flag**, including the plan's
   own §1 row-1 invocation, and the plan says so nowhere (F-1).
2. **I2/N1: the decomposition is right for tr and silently wrong for wsh.**
   Under the uniform-policy ruling, fired-detection is needed on every wrapper
   the policy enforces — the fold and its acceptance pin **tr-leaf only** (F-3).
3. **C1: the import criterion is workable as written.** The constraint makes
   Phase 1 claim no Core compatibility, so Phase 1's suite needs no import; the
   criterion binds only future compat claims, and the six-binary matrix proves
   imports are runnable on this box. One clause naming the harness would stop
   it silently becoming an unrunnable CI criterion later (N-2).

---

## F-1 (Important, re: I3) — the uniform-policy ruling is correct and its breaking-change consequence is stated nowhere; the rule set it enforces is unscoped

**Fold text attacked:** *"The plan picks **uniform policy** … that decision
makes the wsh hole a **defect to close**, not a behaviour to pin"*; acceptance —
*"implemented as uniform policy, with the wsh hole closed rather than pinned."*

**Why it is wrong (three omissions, reproduced from the plan's own text):**

1. **The breaking change is unstated.** "Enforce on every wrapper, then waive"
   means the command in §1's own table row 1 — `export-wallet --descriptor
   <wsh> --format bitcoin-core`, **exit 0 today, measured** — starts refusing
   without `--allow sigless-branch` the day Phase 1 lands. That is a behavior
   change to a shipped surface (`mnemonic export-wallet`, shipped since
   v0.97.0 per §0), it flips the plan's own measured table, and no sentence in
   the plan says so. Round 1's option (a) — which this fold adopts — carried
   the rider verbatim: *"then the wsh keyless form starts requiring the flag
   too (a behavior change to state loudly, Rust-first per Open Q4)"*. The fold
   took the option and dropped the rider.
2. **The enforced rule set is unscoped.** "The rule" / "the wsh hole" is
   singular throughout, but per round 1's I3 evidence the pinned miniscript
   runs **no** sane rule on Wsh/Sh/Bare at `from_str` — not just sigless. So
   uniform policy is a fork the plan does not take: (a) enforce the **full
   five-rule sane set** on every wrapper (true build-descriptor parity — and
   then any currently-exportable wsh/sh descriptor that is malleable, has
   repeated keys, mixes timelocks, or exceeds resource limits also starts
   refusing, widening the breaking change well past "the wsh hole"); or
   (b) enforce **sigless-branch only**, uniformly, leaving the other four
   uniformly unenforced. Different products, different blast radius, different
   fired-detection obligations — two implementers will build different things,
   which is the exact defect class I3 was filed under.
3. **Open Q4's own obligation is not connected.** Q4 says the tr/wsh
   sanity-check asymmetry is *potentially normative* and *"if either is
   touched, it is Rust-first with vectors."* Uniform policy **is** touching it
   — Phase 1 now erases the asymmetry — yet neither the Phase-1 section nor
   its acceptance requires the vectors, and the fold deleted the old
   Rust-primary bullet from Phase 1 in the same diff.

**What would fix it:** one paragraph and two acceptance bullets. State plainly:
"Phase 1 is a breaking change: the §1 row-1 wsh export (exit 0 today) will
refuse without `--allow sigless-branch`; §1's table describes pre-Phase-1
behavior." Pick and name the enforced rule set ((a) or (b) above). Add to
acceptance: vectors pinning the new refusals per Open Q4, and a release-note
line for the behavior change.

## F-2 (Important, re: I1) — the fold defers to the implementer the two decisions round 1 said the plan must make, and drops the one safety constraint that governs the pick

**Fold text attacked:** *"the implementer **enumerates every strict parse site
on the `--descriptor` path first and lists them in the PR**, then states where
the relaxation lives (one chokepoint vs per-site)."*

**Why it is wrong.** Round 1's demand was *"The plan must name the sites and
pick the architecture"*; the fold's response is to schedule the implementer to
do both, past the gate. Three specific defects:

1. **Nothing is bought by deferral.** The enumeration already exists — round
   1's persisted appendix lists all six sites (`export_wallet.rs:524, 640`;
   `bitcoin_core.rs:48`; `pipeline.rs:175`; `bsms.rs:105`; `green.rs:52`). The
   plan can carry them by file+function (line numbers decay; identities do
   not) at zero cost.
2. **The fold's own I3 ruling already constrains the pick, and the fold does
   not connect them.** Uniform policy means export-wallet owns an **explicit
   admission check** — that is the single-admission-gate design. Once a
   sigless string is *admitted* at intake, every downstream strict re-parse
   (:640 script-type derive, the emitter re-parses) dies on it regardless;
   they must become lenient re-parses of an already-admitted string. Leaving
   "one chokepoint vs per-site" open as if it were free presents a closed
   question as open — the residual per-site option (threading ExtParams to
   every site) implements the same policy with a strictly worse failure mode
   (any site that misses the waiver refuses late with a wrong error).
3. **The governing safety caveat was dropped.** Round 1: *"a lenient emitter
   parse is safe only while intake remains the sole gate — and pipeline.rs:28
   also serves the `--template` path."* That is the invariant that makes the
   chokepoint design sound, and it appears nowhere in the fold. An implementer
   who makes `pipeline.rs:28` lenient without preserving intake-as-sole-gate
   silently removes validation from the `--template` path.

**What would fix it:** the plan names the sites (file+function), rules for the
single admission gate at intake (which F-1's ruling already implies), states
that downstream re-parses become lenient parses of admitted strings, and
carries the sole-gate/`--template` invariant as an acceptance bullet.

## F-3 (Important, NEW — interaction of the I3 ruling with the I2/N1 folds) — acceptance pins fired-detection to tr leaves while the ruling extends the never-silent contract to every wrapper

**Fold text attacked:** *"the **fired-detection for tr leaves is new code** and
is Phase 1's real work"*; acceptance — *"tr-leaf fired-detection implemented
and tested (new code, not reuse)."*

**Why it is wrong.** This decomposition was correct against round 1's plan
(quirk-shaped relaxation, tr the only refusing wrapper). The same fold then
ruled for uniform policy — under which wsh/sh admission is **also** gated and
**also** waivable, so the fired/did-not-fire contract (*"an unmissable stderr
warning for every allowed rule that actually FIRED, plus a note for each
requested allowance that did not"*) now binds on wsh too. The existing
machinery cannot satisfy it there as-is: gate.rs's check is private, template-
pipeline-shaped (`Miniscript::<_, Segwitv0>::from_str_ext` over a rendered
`wsh(M)` inner, gate.rs:178), not callable on an arbitrary admitted export
descriptor. So wsh-side fired-detection at export is at minimum new wiring —
and the acceptance list names **only** tr-leaf detection. An implementer
satisfying acceptance literally ships tr detection and leaves the wsh waiver
path either warning-unconditionally or warning-silent — recreating on wsh
exactly the divergence round 1's I2 flagged for tr, on the flagship wallet's
own wsh form (the one export the operator will actually run with `--allow`).

**N1 residue, same locus:** the fold's piece list ("both pieces are currently
private to `cmd/build_descriptor.rs`") omits `AllowSet::to_ext_params` —
private `fn`, `descriptor_builder/gate.rs:61`, a **different module** — which
the new detection code needs. And round 1's literal N1 ask ("name the
destination module") remains unmet ("a shared module").

**What would fix it:** acceptance bullet becomes "fired-detection per enforced
wrapper — per-leaf for tr, top-level for wsh/sh — implemented and tested";
add `to_ext_params` to the pieces getting the shared home; name the module.

## F-4 (Important, re: I4) — sub-part 3 is unaddressed, and the fold deleted the baseline admission tests without restating them

**Fold text attacked:** *"Fold: added — (a) requesting one rule must not admit
any other, asserted per-rule; (b) a keyless-leaf vector through the
**transforming** emitters (`bip388`)"* — presented as closing I4 — plus the
Acceptance list.

**Why it is wrong:**

1. **I4.3 is nowhere.** Round 1: *"What does `--allow` do with
   `--template`/`--slot` or `--from-import-json`? … one line settles it."*
   Grep of the folded plan: `--from-import-json` does not appear; no sentence
   addresses `--allow` on non-`--descriptor` paths. The one line was never
   written, and the divergence it settles is still open.
2. **The fold deleted the round-1 baseline tests and the acceptance list does
   not restore them.** The old bullet — *"tr form exports with `--allow
   sigless-branch` and refuses without it; the warning appears; a rule
   requested but not fired says so"* — was removed in this diff. Acceptance
   now contains over-admission tests, the bip388 vector, "tr-leaf
   fired-detection … tested", and warning wording. None of those is the
   **refuses-without-the-flag** test — the one test that asserts admission is
   actually gated, per wrapper, which under uniform policy is the surface's
   own new check rather than an upstream accident. An implementation
   satisfying the acceptance list verbatim never proves the flagless refusal.
3. **I4.2 is half-done.** The bip388 vector is pinned (good), but the positive
   export path still names no `--format` under test — and now there is no
   positive-path test at all to attach it to (see 2).

**What would fix it:** restore the baseline tests to Acceptance, per wrapper
and with the format named (per F-1's ruling: flagless refusal on tr AND wsh;
export-with-flag; fired warning; requested-not-fired note), and add the one
sentence ruling `--allow`'s behavior on `--template`/`--slot`/
`--from-import-json` (did-not-fire notes, or refusal as meaningless — either,
stated).

---

## F-5 (Minor, NEW) — "Phase 1 may start now" survives the RED gate the fold recorded

**Text attacked (untouched by the diff, falsified by it):** §3 preamble —
*"Phase 1 may start now; Phases 2-4 are blocked on the pending reports being
folded."*

True when written (it meant "not blocked on the other reports"); false after
the fold recorded **gate RED** and "No code until a re-review returns 0C/0I"
in the status line and the Phase-1 header. The plan now both blocks and
authorizes the same act — the diff-falsifies-text-it-never-touches class, and
outside the machine gate's four assertions (which checked the status line
only). Fix: "Phase 1 starts when its R0 gate returns 0C/0I; Phases 2-4 are
additionally blocked on …".

## N-2 (Nit, re: C1) — the import criterion should name its harness

*"The acceptance criterion for anything claiming Core compatibility is an
**import**, not an emit"* is sound and, for Phase 1, deliberately vacuous (the
constraint forbids the claim). But repo CI has no Core binaries; the imports
that established the six-binary matrix ran on this box. One clause — "run on
the operator's Core-binary harness, not CI" — stops the criterion silently
mutating into either an unrunnable CI gate or an emit-only shortcut in a later
phase.

## N-3 (Nit, re: M1) — the no-trace rationale contradicts the fold's own I4 text

M1's fold: *"The emitters are passthrough by design."* Four paragraphs later,
I4's fold: *"the **transforming** emitters (`bip388`)"*. Both cannot be true
as written; round 1 established bip388 transforms (pipeline.rs:175). The
**decision** (no trace in the artifact) survives — no target format has a
comment field either way — but the stated rationale is half-false. Fix the
sentence ("passthrough or format-fixed by design"), keep the decision.

---

**Gate: RED — 0 Critical / 4 Important / 1 Minor / 2 Nit.** All four
Importants are plan-text fixes concentrated in the Phase-1 section and its
Acceptance list; none reopens the uniform-policy ruling itself, which this
review affirms.
