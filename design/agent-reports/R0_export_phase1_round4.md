# R0 re-review, round 4 — PLAN_wallet_file_export.md Phase 1 fold (09b1035..3aa6f33)

- **Reviewed:** 2026-08-22. Scope strictly: (1) did the round-3 fold answer
  R3-1, R3-2, R3-3; (2) did the fold introduce a new defect. Not a fresh audit;
  settled rulings (`--allow` name, Phase 2 deleted, hot export out, keying tier
  4, the measured refusals, C1's import criterion, the (b) sigless-branch-only
  ruling, note-not-refusal) were not reopened, and none moves below.
- **Verdict: 0 Critical / 2 Important / 1 Minor / 0 Nit — the gate stays RED.**
- Both Importants are defects *of this fold's answers*, on exactly the two
  lenses the round-4 brief pushed: the round-3 rulings do not compose with each
  other (R4-1), and topology (B)'s stated locus is not a real single place in
  the code (R4-2). The trend 1C/4I → 0C/4I → 0C/2I → 0C/2I has NOT converged;
  the two remaining fixes are, once again, one topology-placement sentence and
  one note-wording table.
- Machine-checked this round (not recalled), all at `mnemonic-toolkit` tip
  `5f88071c`, paths `crates/mnemonic-toolkit/src/`:
  - The three export arms build `EmitInputs` at **two** sites in **two**
    functions: `cmd/export_wallet.rs:697-698` (`run`: `--descriptor` and
    `--template`/`--slot` converge on the local `canonical` there) and
    `:926-927` (`run_from_import_json`, its own local `canonical_descriptor`).
    The only pre-`EmitInputs` chokepoint they share is the
    `CheckedDescriptor::new` boundary (called in both struct literals); the
    only place all three arms meet as one value is `emit_payload(&inputs, …)`
    — **after** `EmitInputs`, not before.
  - Two **more** production `EmitInputs` constructors exist outside
    export-wallet: `cmd/restore.rs:2496` (`build_import_payload`,
    builder-typed, cannot be sigless) and `cmd/restore.rs:2801`
    (`build_multisig_import_payload`, whose general arm is by its own doc
    comment "descriptor-mode `EmitInputs` mirroring `export-wallet
    --descriptor`", dispatching through the same shared `emit_payload`).
  - **Run, not inferred:** `mnemonic restore --md1 <RCW keyed wsh chunks>
    --format bitcoin-core` (md1 from
    `design/journeys/out/rcw/wsh/md1-keyed.txt`; reconstructed descriptor
    carries the sigless tier 4 `and_v(v:after(1383520),sha256(…))`) → **exit
    0, 2694 bytes of bitcoin-core JSON on stdout, no flag, no admission note**
    — the same byte count as the plan's §1 measured export-wallet row.
  - `restore` has **no** descriptor-rule `--allow`; its only waiver flag is
    `--allow-mismatch` (cosigner-fingerprint cross-check, unrelated).
  - `wallet_export/bitcoin_core.rs` contains no taproot refusal and uses
    `bitcoin_core_version` only for output shaping (`:29`, `:46`) — bears on
    R3-3's soundness below.

## Ledger — the three round-3 findings

| ID | answered? |
| --- | --- |
| R3-1 | **Structurally yes** — topology (B) chosen and written down; `--from-import-json` corrected out of the exempt list, into acceptance as gated, with its own refuse/waive test. But the answer carries both new Importants: the locus phrase names a convergence point that does not exist as one place (R4-2), and the `--template`/`--slot` half of the split contradicts the R3-2 wording the same fold adopts (R4-1). |
| R3-2 | **Half.** The rule — "the 'passes that rule' parenthetical may only ever be printed by a rule that actually ran" — is adopted and sound; the unenforced-rule wording is truthful on every arm and composes with the five-value vocabulary and drift test. The **ungated-path wording is false under the fold's own topology ruling** (R4-1). |
| R3-3 | **Yes, soundly.** `--format bitcoin-core` named. Verified the choice is satisfiable per wrapper on the emission side: the bitcoin-core emitter carries no taproot refusal, so the tr export-with-flag baseline can pass; refusal baselines are intake-side and format-independent. |

## Direct answers to the three pushed questions

1. **Pairwise composition of the whole ruling set: one contradiction.** R3-1's
   ruling text (uniform gate, "honouring the `AllowSet` uniformly", template/
   slot "stated as a consequence rather than an exemption") contradicts R3-2's
   ungated-path note and its acceptance bullet (an exemption, in as many
   words: "no descriptor admission gate runs on `--template`/`--slot`") —
   R4-1. All other pairs compose: (B)×(b) sigless-only; (B)×per-wrapper
   fired-detection (the gate sees the canonical descriptor on every arm);
   (B)×never-silent (every `--allow` still yields exactly one output per rule
   once R4-1 is fixed); unenforced wording×shared vocabulary; the corrected
   `--from-import-json` classification×C1's import criterion; the status
   banner's round counts are accurate.
2. **Topology (B)'s locus is not a real place — as one place.** The three arms
   converge on a single canonical descriptor **nowhere before `EmitInputs`**:
   two construction sites, two functions (measured above). The ruling is
   implementable — one gate helper invoked at both sites, and the fold's own
   `--from-import-json` acceptance test pins the second site — but the words
   "a single admission gate … before `EmitInputs`" equally describe gating the
   shared `CheckedDescriptor`/`EmitInputs` boundary itself, and that boundary
   also serves `restore`'s two production constructors. The two readings are
   observably different shipped products (R4-2).
3. **The note wordings: no case falls through to the OLD lying note — the
   defect is a spurious case instead.** Under (B) the reachable matrix is
   three cells with **no arm dimension**: (sigless-branch, fired) → WARNING;
   (sigless-branch, ran-didn't-fire) → did-not-fire note, parenthetical TRUE
   on every arm including `--template`/`--slot`; (any other rule, any arm) →
   unenforced-rule note, TRUE on every arm. The old note survives only in its
   truthful cell. The fold's scheme instead mandates a **fourth** wording
   whose trigger condition — an ungated path — does not exist under the
   fold's own topology, and assigns it the two template/slot cells, where it
   displaces a true sentence with a false one (R4-1).

---

## R4-1 (Important, NEW — re: the R3-1 and R3-2 answers) — the fold's two rulings disagree about whether `--template`/`--slot` are gated, and the note it mandates there is false under its own topology

**Fold text attacked:** R3-1 ruling — *"**the admission gate runs on the
canonical descriptor where all three arms converge, before `EmitInputs`,
honouring the `AllowSet` uniformly.**"* and *"`--template` / `--slot` — a
builder-produced descriptor cannot carry a sigless branch, so those paths only
ever emit the note. True, and now stated as a consequence rather than an
exemption."* — against R3-2 ruling — *"**ungated path** — 'note: `--allow`
does not apply to this path — no descriptor admission gate runs on
`--template`/`--slot`'"* — and acceptance — *"`--allow` on
`--template`/`--slot` emits the ungated-path note (NOT `--from-import-json`,
which is gated — see above)."*

**Why it is wrong.** Under topology (B) there is no ungated path. The gate
runs on the canonical descriptor of **every** invocation — that is the entire
content of "where all three intakes converge", "honouring the `AllowSet`
uniformly", and the acceptance's "no arm routes around it". A
`--template`/`--slot` descriptor is admitted by that same gate; the gate
evaluates `sigless-branch` on it and never fires, because builder output
cannot be sigless — which is precisely the fold's own "consequence rather
than an exemption" sentence. Reproduced on the fold's own terms:
`export-wallet --template wsh-multi --slot … --allow sigless-branch --format
bitcoin-core` — under (B) the rule **ran** and did not fire, so the truthful
output is the existing did-not-fire note, whose parenthetical *"(the policy
passes that rule without it)"* is TRUE here by the plan's own new test ("may
only ever be printed by a rule that actually ran" — it ran). The fold instead
mandates printing *"no descriptor admission gate runs on
`--template`/`--slot`"* — a false statement about the mechanism, on the
surface whose last two rounds were spent on exactly the ran-vs-fired
distinction. R3-2's defect class, inverted: round 3's note asserted a check
that never ran; this note denies a check that runs.

How it entered: round 3's two prescribed remedies were themselves
inconsistent — R3-2's ungated wording listed `--template`/`--slot`/
`--from-import-json`, written against the pre-(B) exempt-list framing, while
R3-1's remedy made (B) gate everything. The fold caught the
`--from-import-json` half of that collision and removed it from the wording;
it kept the `--template`/`--slot` half. In round 3's text, template/slot
"only ever emit **the note**" bound to the **did-not-fire** note (the F-4
ruling it was a prediction of); the fold silently re-bound "the note" to the
new ungated-path note while keeping the consequence-not-exemption sentence
that is only true of the original binding. Prescribed fixes are not
authoritative; this one needed checking against the topology it was adopted
beside.

The acceptance set is now unsatisfiable by a truthful implementation: to make
the ungated note true, exempt template/slot from the gate — violating "no arm
routes around it" and "consequence rather than an exemption"; to honour the
topology, print a falsehood. Two implementers resolve that differently — the
exact defect class R3-1 was filed under, produced by the fold that closed it.

**What would fix it:** delete the ungated-path note — under (B) it has no
referent. The note matrix loses its arm dimension entirely: `--allow
sigless-branch` on any arm → fired WARNING or the did-not-fire note (true
everywhere, including template/slot, as a consequence of the uniform gate);
`--allow <other>` on any arm → the unenforced-rule wording (already
arm-independent and true everywhere). Rewrite the acceptance bullet to:
"`--allow sigless-branch` on `--template`/`--slot` emits the did-not-fire
note — a consequence of the uniform gate, asserted as such; `--allow <other>`
emits the unenforced-rule note; same wordings as `--descriptor`, because no
arm is ungated."

## R4-2 (Important, NEW — re: the R3-1 answer) — "where all three arms converge, before `EmitInputs`" names a place that does not exist as one place, and the boundary it does name contains a fourth shipped surface

**Fold text attacked:** *"**the admission gate runs on the canonical
descriptor where all three arms converge, before `EmitInputs`, honouring the
`AllowSet` uniformly.**"* and acceptance — *"**A single admission gate on the
canonical descriptor, where all three intakes converge (before
`EmitInputs`)** — topology (B) — with an assertion that it is the ONLY
admission point and that no arm routes around it."*

**Why it is wrong.** Measured this round: the three arms never converge on a
single canonical descriptor before `EmitInputs`. Arms 1-2 converge inside
`run` and build `EmitInputs` at `cmd/export_wallet.rs:697-698`; arm 3 is a
separate function building its own `EmitInputs` at `:926-927`. The one thing
the sites share pre-`EmitInputs` is the `CheckedDescriptor::new` boundary —
and that boundary is **wider than export-wallet**: `cmd/restore.rs:2496` and
`:2801` are production `EmitInputs` constructors too, the second being a
general-policy arm that mirrors `--descriptor` and reaches the same emitters.
So the implementer must invent the gate's location, and the two natural
inventions are different products:

- **Gate helper at export-wallet's two construction sites** (the intended
  reading): passes every acceptance bullet; `restore` untouched.
- **Gate at the shared boundary** (`CheckedDescriptor::new` / `EmitInputs`
  construction — the reading the words "a single admission gate … before
  `EmitInputs`" most literally describe): ALSO passes every acceptance bullet
  (all of them test export-wallet only), and silently extends enforcement to
  `restore --md1 --format` — a shipped surface with **no `--allow` flag**
  (only `--allow-mismatch`, a fingerprint cross-check). Run this round:
  `restore --md1 <RCW keyed wsh> --format bitcoin-core` reconstructs the
  flagship wallet's sigless-tier wsh and **emits flagless at exit 0 today**
  (2694 bytes). Under boundary-gating that command starts refusing **with no
  waiver possible** — "new refusals nobody asked for on a shipped tool", the
  precise blast radius the (b) ruling's own rationale rejects, decided in the
  implementer's PR, which is what F-2 was filed to prevent. Round 1 called
  gate-location divergence "the single largest 'two implementers build
  different things' risk"; the fold's locus sentence reintroduces it one
  level up.

Note what this does NOT reopen: "the wsh hole closed rather than pinned"
remains true as a claim about `export-wallet`'s surface. But the plan now
needs one honest sentence about the adjacent door it measured into existence:
under the intended call-site reading, the same emitters remain reachable
flagless via `restore --md1 --format` — out of Phase-1 scope by choice, not
by silence.

**What would fix it:** replace the locus phrase with the mechanism, stated
against the real code: "one gate helper, invoked at export-wallet's two
`EmitInputs` construction sites (`run` and `run_from_import_json`), on each
arm's canonical descriptor, honouring the `AllowSet`; other `EmitInputs`
builders (`cmd/restore.rs:2496`, `:2801`) are explicitly out of scope — no
behaviour change to `restore` in Phase 1 (if the operator wants that door
ruled on, it is its own decision with its own release note)." Add the intake
consequence the topology implies but never states: the `--descriptor` intake
parse (`:524`, per the plan's own cite) becomes lenient so a tr form can
reach the gate at all — an implementer who keeps it strict fails the
export-with-flag baseline, so this is machine-caught, but say it. And take
the zero-cost half of round 3's remedy the fold left behind: extend the
parse-site enumeration bullet from "the `--descriptor` path" to all three
arms, annotated with which arm each site serves — that annotation is what
makes "ONLY admission point" assertable rather than asserted.

## R4-3 (Minor, re: the R3-1 answer's acceptance bullet) — "a sigless envelope refuses without the flag and exports with it" is unsatisfiable for the tr wrapper

**Fold text attacked:** acceptance — *"**`--from-import-json` gated like
`--descriptor`** — a sigless envelope refuses without the flag and exports
with it. This is the hole round 3 found."*

For a **wsh** envelope both halves hold and the bullet is exactly right — it
is the hole round 3 found. For a **tr** envelope the "exports with it" half
is unreachable: a sigless-tr body dies at the arm's strict script-type parse,
and any parse-surviving taproot envelope is categorically refused by the
v0.28.7 Fix-α gate regardless of flags. Round 3 flagged this ordering as a
side symptom of the old ruling; the fold dissolved the old ruling but the new
bullet, wrapper-unnamed, re-collides with it. An implementer who picks a tr
envelope for this test gets a permanently red gate — or worse, "fixes" it by
relaxing Fix-α, which nobody ruled on. Fail-closed in every case, hence
Minor. Fix: one word and one clause — "a sigless **wsh** envelope refuses
without the flag and exports with it; taproot envelopes remain categorically
refused by Fix-α regardless of `--allow`."

---

**Gate: RED — 0 Critical / 2 Important / 1 Minor / 0 Nit.** The fold answered
all three round-3 IDs and correctly took the one non-obvious half-step (pulling
`--from-import-json` out of the ungated wording). What it did not do is the
thing round 3 itself failed to do one round earlier: check the adopted
remedies against each other and against the measured surface. Both Importants
are that same omission — R4-1 is round 3's two remedies colliding, R4-2 is
round 3's prescribed locus phrase adopted without verifying the place exists.
The fixes are one placement sentence, one note-matrix simplification (delete a
wording, don't add one), and one word in an acceptance bullet; no settled
ruling moves, and (b), (B), note-not-refusal and the import criterion all
stand.
