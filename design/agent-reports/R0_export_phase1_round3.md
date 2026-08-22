# R0 re-review, round 3 — PLAN_wallet_file_export.md Phase 1 fold (6087a35..a0a935f)

- **Reviewed:** 2026-08-22. Scope strictly: (1) did the fold answer round 2's
  seven findings (F-1..F-5, N-2, N-3), (2) did the fold introduce a new defect.
  Not a fresh audit; settled rulings (`--allow` name, Phase 2 deleted, hot
  export out, keying tier 4, the measured refusals, C1's import criterion, the
  (b)-over-(a) blast-radius choice, note-not-refusal on non-descriptor paths)
  were not reopened.
- **Verdict: 0 Critical / 2 Important / 1 Minor / 0 Nit — the gate stays RED.**
- Both Importants are concentrated in two fold paragraphs (F-2's decision text
  and F-4's ruling sentence) plus the acceptance bullets they feed. Neither
  reopens a ruling: (b) stands, note-not-refusal stands, gate-at-intake stands
  — what is wrong is where the gate sits and what the note *says*.
- Machine-checked this round (not recalled): the three descriptor intakes and
  five downstream parse sites listed under R3-1; `CheckedDescriptor::new`
  validates ONLY the BIP-380 `#<8-char>` suffix format
  (`wallet_export/mod.rs:449-475`) — it is not a sanity gate; the existing
  not-fired note's full wording (`cmd/build_descriptor.rs:201`); `AllowSet` and
  `to_ext_params` at `descriptor_builder/gate.rs:52,61` (so F-3's "beside
  `gate.rs`, which already owns `AllowSet`" is true); the F-1 blockquote
  matches §1 row 1 (plan line 37: exit 0, measured) and §0's v0.97.0 (line 22);
  N-2's harness claim matches the round-1 evidence method
  (`R0_export_phase1.md:310`: `createwallet r0gate` → `importdescriptors`).
  All code cites are `mnemonic-toolkit/crates/mnemonic-toolkit/src/`.

## Ledger — the seven round-2 findings

| ID | answered? |
| --- | --- |
| F-1 | **Yes, with one defect in the mechanism.** Breaking change stated and verified against the plan's own measured row; rule set chosen ((b), with the (a) blast radius named); Q4 vectors + release-note line in acceptance. The note mechanism that (b) leans on is R3-2, and "the wsh hole closed rather than pinned" is overclaimed under one reading of R3-1. |
| F-2 | **Both decisions made** (single admission gate; sole-gate/`--template` invariant restored as an acceptance bullet). But the topology it decides contradicts the F-4 answer — R3-1 — and the declined site-naming sub-part ("that is discovery, not a decision") is exactly what hid the contradiction. |
| F-3 | **Yes, soundly.** Per-wrapper fired-detection matches round 2's prescribed fix; `to_ext_params` added to the pieces; module named, and the naming claim is true against the code. |
| F-4 | **Baselines restored per wrapper; I4.3 ruled — but the ruling's stated premise is false for `--from-import-json`** (R3-1) and its note text is R3-2. "Format named" residue is R3-3 (Minor). |
| F-5 | **Yes.** Removed; the replacement banner's round counts are accurate (1C/4I, 0C/4I). |
| N-2 | **Yes.** Harness named (regtest `importdescriptors`, per-entry `success`, on this box); verified it is the method that produced the round-1 evidence and the plan's own v25 measurement (plan line 240). |
| N-3 | **Yes.** Decision kept, rationale narrowed to format-specific/silently-dropped — consistent with what round 2 established. |

## Direct answers to the three pushed questions

1. **The (b) ruling is coherent and admits nothing (a) would catch that the
   shipped surface does not already admit** — its blast-radius rationale is
   sound, and I affirm the choice. The dishonesty the question probes is real
   but lives entirely in the *note text* the fold pins ("the existing
   'requested but did not fire' note — no new concept"): the existing note
   asserts a check passed that never ran (R3-2). Fix the wording and (b) is
   fully honest; no state exists where the operator loses a check they have
   today.
2. **"A single admission gate at intake" is implementable — but only in one of
   the two topologies the fold simultaneously asserts.** The surface has three
   descriptor intakes, not one; "downstream re-parses become lenient parses of
   admitted strings" holds for no topology as written, because three of the
   five downstream sites serve all three arms (R3-1). No downstream site needs
   sanity *re-validation*; what is needed is a stated gate position.
3. **The did-not-fire ruling interacts badly with (b) exactly as suspected,
   and worse with the gate decision.** The identical note string ends up
   covering three states — enforced-rule-didn't-trigger (true), rule never
   enforced (false), path never gated (false) — R3-2; and for
   `--from-import-json` the ruling's premise "those paths do not reach the
   descriptor gate" is factually false — R3-1.

---

## R3-1 (Important, NEW — interaction of the F-2 and F-4 answers) — the fold decides two incompatible gate topologies, and its "already-admitted string" claim is false against the real three-intake surface

**Fold text attacked:** F-2 — *"Decided: **a single admission gate at
intake.** Every downstream re-parse then becomes a lenient parse of an
already-admitted string"*; acceptance — *"**A single admission gate at
intake**, and an assertion that it is the ONLY admission point — `--template`
inputs must not route around it"*; F-4 — *"those paths do not reach the
descriptor gate, so there is nothing to waive and a refusal would be a lie
about why"*; acceptance — *"the wsh hole closed rather than pinned."*

**Why it is wrong.** `export-wallet` has **three** descriptor intakes, each
with its own strict parse (machine-checked this round):
`--descriptor` at `cmd/export_wallet.rs:524`; `--template` builds then
strict-parses the built string inside `build_descriptor_string`
(`wallet_export/pipeline.rs:28`); `--from-import-json` strict-parses the
envelope's descriptor body at `cmd/export_wallet.rs:826`. The downstream
emitter parses (`wallet_export/bitcoin_core.rs:48`, `bsms.rs:105`,
`green.rs:52`, `wallet_export/pipeline.rs:175`) all consume
`EmitInputs.canonical_descriptor`, which is fed by **all three arms**
(`EmitInputs` built at `export_wallet.rs:698` and `:927`). Now take the fold's
two statements in turn:

- **Topology (A) — the gate is the `--descriptor` intake** (what F-4's
  rationale asserts). Then "ONLY admission point" is false — `:28` and `:826`
  remain live admission points — and "the wsh hole closed rather than pinned"
  is false too: an envelope carrying the same sigless wsh passes `:826`'s
  `from_str` (round 1's own measured premise — no sane rule runs on wsh at
  `from_str`), and the only other checks on that string are the BIP-380
  checksum (`descriptor_body_no_csum`) and `CheckedDescriptor::new`, which
  validates **only the `#<8-char>` suffix format** (`wallet_export/mod.rs:
  449-475`). So `export-wallet --from-import-json <envelope w/ sigless wsh>
  --format bitcoin-core` exits 0 with no flag, no warning, no note — today and
  after Phase 1. The refusal the F-1 blockquote promises ("will REFUSE without
  `--allow sigless-branch`") is bypassed through the adjacent door of the same
  command, and "enforced uniformly" is enforced on one of two string intakes.
- **Topology (B) — the gate sits after the join, covering all arms** (what
  "ONLY admission point — `--template` inputs must not route around it" says).
  Then F-4's rationale "those paths do not reach the descriptor gate" is false
  by construction, and its ruling inverts: a sigless-wsh envelope *reaches*
  the gate, `--allow sigless-branch` *waives* it, and the correct output is
  the fired WARNING — not the did-not-fire note the plan mandates for that
  path.

Two implementers reading this fold build different products — the exact
defect class I1/I3/F-2 were filed under, now produced by the fold that closed
them. A side symptom under either topology: a sigless-**tr** envelope dies at
`:826`'s strict parse (raw `DescriptorParse` error) before any gate or note
code can run, so "produces the did-not-fire note rather than refusing" is
unimplementable as stated for that input unless note emission is ordered
before intake parsing — which nothing specifies. (Sane taproot envelopes are
refused later anyway by the v0.28.7 Fix-α gate; the sigless ones never get
that far.)

**What would fix it:** pick the topology and write it down. **(B) is the one
consistent with the fold's own acceptance** ("ONLY admission point", "enforced
uniformly", "hole closed"): the admission gate runs on the canonical
descriptor where the three arms converge (before `EmitInputs`), honoring the
`AllowSet` uniformly. Then the F-4 ruling becomes a true *prediction* for
`--template`/`--slot` (a builder-produced descriptor cannot carry a sigless
branch, so those paths do in fact only ever emit the note) and must be
**corrected for `--from-import-json`** (an envelope descriptor CAN be sigless:
it is gated and waivable exactly like `--descriptor`). If (A) is intended
instead, the plan must say "gate on the `--descriptor` arm only; `:28` and
`:826` remain those arms' own strict admission points", scope "ONLY admission
point" to the `--descriptor` path, and downgrade "the wsh hole closed rather
than pinned" to "closed on the `--descriptor` intake; retained on
`--from-import-json`". Either way, carry the parse-site list file+function
**with which arm(s) each serves** — the enumeration the fold declined as
"discovery, not a decision" already exists in round 1's persisted appendix at
zero cost, and its absence is precisely what let "every downstream re-parse
becomes a lenient parse of an already-admitted string" ship unfalsified.

## R3-2 (Important, NEW — the (b) mechanism, re: the F-1 and F-4 answers) — the note the plan specifies asserts a check that never ran

**Fold text attacked:** *"requesting one always produces the existing
*'requested but did not fire'* note — no new concept, and the plan says so
explicitly rather than leaving a flag that silently does nothing"*; F-4 —
*"produces the did-not-fire note."*

**Why it is wrong.** The existing note's full wording, machine-checked at
`cmd/build_descriptor.rs:201`:

> `note: --allow {} was requested but did not fire (the policy passes that
> rule without it)`

On `build-descriptor` all five rules run, so "did not fire" always means
"checked and clean" and the parenthetical is truthful. Under (b) on export,
four of the five rules **never run** — the same sentence then asserts "the
policy passes that rule" about a rule nobody evaluated. Reproduced:
`export-wallet --descriptor <malleable wsh> --format bitcoin-core --allow
malleable` exports (malleability unenforced on this surface) and prints a
mechanically-produced **false funds-safety statement** — on the surface whose
standing constraint is that no text may say `--allow` "enables" anything, and
whose round-1 C1 was precisely an output overclaim ("works" vs
Core-refuses-at-import). The fold's cure for the lying-flag worry — "the plan
says so explicitly" — puts the truth in the plan and the lie in the tool:
users read stderr, not `design/PLAN_wallet_file_export.md`. And the same one
string is also the mandated output for the never-gated paths, completing the
collision the round-3 brief predicted: `--template` + `--allow sigless-branch`
and `--descriptor` + `--allow malleable` print identical sentences for
entirely different reasons. Three states, one wording — (i) rule enforced,
gate ran, didn't trigger (the words are true); (ii) rule never enforced on
this surface (parenthetical false); (iii) path never gated (parenthetical
false). Only (i) matches the words.

**What would fix it:** keep (b) and keep note-not-refusal — both stand.
Specify two export-side wordings and hang them on the existing
"export-specific warning wording" acceptance bullet, with the rule that the
"passes that rule" parenthetical may only ever be printed by a rule that
actually ran: unenforced rule → *"note: `--allow <rule>` has no effect on
`export-wallet` — only `sigless-branch` is enforced here; the descriptor was
NOT checked against `<rule>`"*; ungated path → *"note: `--allow` does not
apply to `--template`/`--slot`/`--from-import-json` — no descriptor admission
gate runs on this path"*. That dissolves the same-note-two-reasons collision
and makes (b) as honest in the tool as it now is in the plan.

## R3-3 (Minor, re: F-4.3 residue) — "format named" names no format

**Fold text attacked:** acceptance — *"**Baseline tests, per wrapper, format
named:** flagless refusal on tr AND wsh; export-with-flag; the fired warning;
the requested-not-fired note."*

Round 2's complaint was that the positive export path "names no `--format`
under test"; the fold's bullet requires that a format be named and still names
none — it transcribes round 2's fix phrase without executing the naming, so
the choice lands with the implementer after all. Confined to the positive
path (the refusal tests are format-independent: intake precedes any emitter),
so Minor. Fix: name `--format bitcoin-core` — the plan's own measured row —
for the export-with-flag test.

---

**Gate: RED — 0 Critical / 2 Important / 1 Minor / 0 Nit.** The loop's shape
held: both Importants are defects *of this fold* — a topology contradiction
between two of its answers and a specified-output falsehood in a third — not
reopened rounds. The fixes are one topology paragraph, two note wordings, and
one named format; no settled ruling moves.
