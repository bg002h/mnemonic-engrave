# Failure / interruption / partial-state review — IMPLEMENTATION_PLAN_multisig_build_repair.md

Lens: what state can this flow be left in, and can the operator get out of it.
Reviewed 2026-08-13 against the plan (post-round-3), SPEC_multisig_build_repair.md
(GREEN), and source at `/scratch/code/shibboleth/seedhammer` @ `a10d007`. Every
mechanism cited below was read or grepped in the source, not inferred from
comments; prior reports (R0 rounds 0–3, fold-check, inherited-facts,
propagation-check) contain **zero** hits for scrub/abort/interrupt/power — this
lens is genuinely unexamined after nine rounds.

**Verdict: no Critical, six Important.** The plan is explicit about every way a
gate *refuses* and silent about every way the flow *stops*. All six gaps have
bounded, mostly plan-text fixes; two (F1, F2) border Critical because following
the device's own abort instruction can put a master seed in the trash, and a
stage gate can close green on a walk that produced a subset of the artifacts it
claims to compare.

## Ranked failure-state table

| # | sev | state | likelihood | badness |
| --- | --- | --- | --- | --- |
| F1 | Important | Interruption of the multi-plate engrave tail (power loss, walk-away, abort): no record of what was cut, no resume, no ruled plate order, determinism unstated and untested | HIGH — mundane causes; Trace B full mode is ~6–9 plates over hours | part-sets in drawers; half-cut plates; recovery possible but nowhere described |
| F2 | Important | Secret (ms1) plates are cut FIRST and the abort warning says "discard the engraved plate(s)" without distinguishing them | MED-HIGH — any full-mode abort | a possibly-funded master's seed on steel in the trash, by instruction |
| F3 | Important | The plan drops spec §4.2 entirely: no stage owns seed-material lifetime, and the mandated mutation-checked scrub test appears in no stage's test list | certain (gap exists by construction) | multi-seed working copies with unproven scrub on a multiplied set of exit paths |
| F4 | Important | A partially failed emulator walk can vacuously satisfy "every mk1 and EVERY ms1"; S1's two-armed gate has no arm for "the walk itself stalled" | MED — any tail bug or walk hang | a stage closes green on exactly the missing-artifact defect the gate exists to catch |
| F5 | Important | S2→S4 window: engraves complete (S2) two stages before the duplicate-key check exists (S4); a duplicated key engraves silently in between | LOW-MED — needs operator provisioning error, unreleased + EXPERIMENTAL | k-of-n that is (k−1)-of-(n−1) on steel, invisible |
| F6 | Important | Under-supplied payload / incomplete chunk set: every refusal message instructs scanning, which phase 1 removed; the only real route (host rewrite) is never named on-device | MED — wrong/stale/partial payloads are ordinary | dead end the operator cannot diagnose from the screen |
| F7 | Important | Walk-away with several masters' seeds in RAM between plates: idle brings the screensaver only; the §10.2.4 wipe is armed solely inside the unlock-session bracket | HIGH — hours-long sets | unattended machine holding multiple seeds indefinitely; the exact state the unlock flow refuses to be in |
| M1 | Minor | Back semantics of the new S4/S5 slot/seed screens unspecified (seed registry retention on reassignment) | — | sloppy-but-compliant retention within the constructor |
| M2 | Minor | ms1 travels as a Go `string` (`bundle.Bundle.MS1`); §4.2's "working copies MUST NOT outlive the flow" is unsatisfiable for string values | — | unscrubbable GC-managed copies; a scrub test written naively cannot fail |

---

## F1 — Interruption of the multi-plate engrave tail

**How reached.** Trace B full mode cuts one md1 card (multiple chunks/plates),
one mk1 per held slot (3), and one ms1 per distinct master (2) — sequentially,
through `bundleEngrave` (`gui/bundle_flow.go:295`), over hours. Power loss,
BOOTSEL reset, mains unplugged, or operator Back at plate k of N.

**What exists afterwards.** Plates 1..k−1 complete, plate k possibly half-cut in
the fixture. Everything else is RAM: `engraveJob` state, the hold-to-resume
`safePoint` history, the set position. `bundleEngrave` "records no completed
state (I-5)" by design; there is no flash record, no plate list, no cut marks
for the build path (the `cut` marks in `unlockPlateListFlow` are a different
flow and RAM-only anyway). On operator Back the abort warning names "card %d of
%d"; on power loss or a §10.2.4-style unwind nothing renders at all (`showError`
exits immediately once `ctx.Done`).

**What the operator sees.** After a reboot: the start screen. A pile of steel
whose membership in the set is recoverable only by reading each plate's chunk
strings by eye — mk1 plates for two accounts of the same master differ only in
their encoded origin. Nothing on the device or in the plan tells them which
cards completed.

**Route out.** It exists, and the plan never says so: the payload region is
flash and survives reboot; Load Payload → digest compare re-authenticates; and
the fork's encoders are **deterministic** — verified: `grep rand` over `md/`,
`mk/`, `codex32/` is empty outside tests, and `mk/encode.go:33,235,329` derives
`chunk_set_id` from the bytecode. Same inputs (payload record order, template,
n, k, held slots, fp choice, seeds, passphrases) mint **byte-identical**
strings, hence identical plates. So a full re-run completes the set, and run-1
survivors are exact duplicates. If any input differs (fp choice, slot order),
the WalletPolicyId/stub differs, so a mixed set fails stub comparison at a
host-side restore rather than restoring wrong — fail-closed, but only checkable
off-device in phase 1. A half-cut plate has no resume after power loss (the
`safePoint` history is RAM; `releaseResumeState` is terminal-only); re-cutting
identical geometry over it depends on fixture registration and is tested
nowhere, including S6.

**Bounded fix.** (a) One paragraph in S5: state the determinism property, and
pin it with a test — encode the same tuple twice, assert byte equality — so
"re-run to complete an interrupted set" is a property with a proof rather than
folklore. (b) One sentence in the S6 gate: at least one hardware run interrupts
a set (power off between plates) and completes it by re-run, confirming the
duplicate-plate story on real steel. (c) The emulator already has the exact
instrument for the in-session half: `cmd/emu`'s `shToolpath.summary().digest`
is documented as "equal digests mean the resumed plate follows the same path"
(`cmd/emu/main.go:50`, `toolpath_js.go:37`) — add one interrupted-and-resumed
plate to a §4.5 walk and assert digest equality. None of this is new machinery.

## F2 — Secret plates first, and "discard the engraved plate(s)"

**How reached.** Full-mode card order is `[ms1, mk1, md1]`
(`gui/multisig_engrave.go:11-35`): the seed plate(s) are cut **first**. Any
abort after plate 1 — operator Back, or `validateMdmk` failing on a later card
— triggers `bundleAbortWarning` (`gui/bundle_flow.go:352`): *"A partial bundle
can't be used — discard the engraved plate(s) and start the bundle over."*

**What exists afterwards.** A completed ms1 plate: a master's full entropy on
steel. Under S5, up to one per distinct master — and in the multi-account shape
these masters plausibly already back **funded** wallets elsewhere; the plate is
not scoped to the new, unfunded policy.

**What the operator sees.** An instruction whose word for a public plate
("discard") is seed-leakage guidance for a secret one. Nothing distinguishes
them.

**Route out.** Operator judgement, against the device's own text.

**Bounded fix, two lines of plan text in S5.** (1) Rule the plate order:
public first, secret last — then most aborts leave only public steel, and the
window in which a completed seed plate exists at all shrinks to the set's tail.
(The current ms1-first order is inherited convention, not a ruling; S5 already
owns `multisigEngraveCards`' multi-ms1 generalisation.) (2) The abort warning
for a set containing `cardMS1` must say **destroy, don't discard** for any
secret plate already cut. Both are S5-territory; neither touches other flows'
call sites (the gate is cards-derived, per the existing R0-I2 pattern).

## F3 — Spec §4.2 has no owner in the plan

**How reached.** By construction. Spec P3 is "§4.1, §4.1a **and §4.2**"; plan
S5 absorbs P3 but its deliverables and both its test lists never mention §4.2,
and `grep -in "scrub"` over the plan returns **zero** lines. §4.2 is normative
REQUIRED and ends with: "A test MUST prove the scrub, and that test MUST be
mutation-checked." S4's `TestGateNeverPrintsSeedOrPassphrase` is a display
assertion, not a lifetime one.

**What exists afterwards.** Today's single-seed scrub is one `defer` over one
`bip39.Mnemonic` (`gui/multisig_build.go:75-79`), reachable on every exit
including `ctx.Done` unwind — sound. S4/S5 replace that with a seed **registry**
(`seedID`-keyed, per-seed passphrases, retained across the whole constructor by
§4.2's own allowance) and multiply the exits: per-slot seed entry Back, slot
review Back, gate FAIL screens, engrave-tail abort, wipe unwind. A registry
whose scrub misses one exit path leaves N masters' seeds in RAM, and no planned
test can notice — the exact false-PASS class F-151 records.

**Operator visibility / route out.** None; this is invisible by nature. That is
what the mutation-checked test is for.

**Bounded fix.** Add to S4's test list (S4 is where the registry and per-seed
passphrase land): `TestBuildFlowScrubsEverySeedOnEveryExit` — every entered
seed observed via the existing `buildMultisigSeedHook` seam, asserted zeroed on
each exit class (Back at each new screen, gate failure, tail abort, `ctx.Done`),
mutation-checked by deleting one scrub site per §4.6. Precedent:
`TestBip85DeriveFlow_ScrubsBothMnemonics` already does this for two mnemonics.
One test name in the plan closes the gap; its absence is a dropped spec MUST.

## F4 — A partial walk can pass a total gate

**How reached.** The §4.5 walk is a script. It hangs, or the tail under test
emits fewer artifacts than it should — the precise defect class S5 exists to
prevent (a missing master's ms1, a missing held-slot mk1).

**What exists afterwards.** A gate record containing an md1 that compares equal
(produced early in the flow) and a comparison loop over "every mk1 … EVERY ms1"
that iterated over **what was produced**. Zero ms1s compared, zero mismatches,
green. `TestFullModeEngravesMs1ForEveryMaster` covers the unit level; the walk
— the thing stages actually close on — has no stated cardinality rule.
Similarly S1's gate is a disjunction ("either … a completed engrave, or D-1
reproduces and is captured") with no arm for "the walk itself stalled partway";
a hung walk satisfies neither and the plan doesn't say that this fails the
stage. This project has already shipped a release on exactly this shape
(empty-output-is-not-absence).

**Operator visibility.** A future reader of the gate record sees green.

**Bounded fix, one sentence in §3 (S5 gate) + the walk-script requirements.**
The walk already MUST record the full input tuple (S0 deliverable 1). Derive
the **expected** artifact census from it — 1 md1, |held slots| mk1s, |distinct
masters| ms1s in full mode — and fail the walk on any count mismatch **and** on
any exit before the flow's terminal screen. A partial walk is a failed gate,
never a partial pass.

## F5 — The duplicate-key window between S2 and S4

**How reached.** S2 makes engraves complete. The duplicate-key check over the
final slot set arrives at S4 (`TestGateRefusesDuplicateKeyAcrossFinalSlots`).
Verified: **no duplicate check exists today anywhere on the path** — neither
`assembleBuildPolicy` (`gui/multisig_build.go:464-506`) nor `md.EncodeMultisig`
(`md/encode_multisig.go:89-164`) compares keys across slots. So from the end of
S2 to the end of S4: operator writes a payload containing their own mk1 among
the cosigner cards (the spec itself names "the 11-card constellation set of
which this wallet needs a subset" as the ordinary provisioning shape), count
happens to be n−1, then enters the same seed → self slot and a card slot carry
the identical 65-byte key.

**What exists afterwards.** `sortedmulti(2, K, K, X)` on steel, labelled
2-of-3, spendable by K alone — §4.1's quorum-degradation hazard, engraved,
invisible on every surface the operator sees.

**Operator visibility / route out.** None on-device during the window; the
review screen shows fingerprints only when included, and the default omits
them.

**Why not Critical.** Unreleased firmware, EXPERIMENTAL-gated, requires an
operator provisioning error — and stages could land in one sitting. But this
project's cycles demonstrably pause for days between green stages, and the plan
already accepted the identical argument for the S2 interim **origin** refusal:
"an interim silence becomes a designed refusal for the cost of an `if`."

**Bounded fix.** Extend S2's interim-guard bullet: alongside the foreign-origin
refusal, refuse when any two final slots (self included) carry an identical
chain code ‖ pubkey — a 65-byte compare in `assembleBuildPolicy`, superseded by
S4's gate exactly as the origin refusal is superseded by S5. One test row.

## F6 — Under-supply and incomplete chunk sets speak a retired language

**How reached.** Payload holds 2 complete cards for n=4 (3 needed), or a card
with 1 of its 2 chunks — a stale payload, a host mistake, a truncated pack.
Mundane.

**What exists afterwards / what the operator sees.** The pending-card drop says
"Dropped an incomplete card — **scan all its chunks** to include it"
(`gui/bundle_flow.go:128`); the count refusal says "**Gather** exactly N−1
cosigner key cards" (`gui/multisig_build.go:63`). Phase 1 removed scanning.
Both messages instruct an action that does not exist on this device; S1's own
deliverable retitles the gather as "a review of what the payload supplied" but
rules no refusal text. Plan S1 test 6 names the over-supply refusal;
under-supply falls through to the stale message. Note the over-supply refusal
also refuses the spec's own 11-card provisioning shape — permitted (spec P0
item 4 allows refusal over selection), but then the message is the only route
map the operator gets.

**Route out.** Real but off-device and unnamed: rewrite the payload on the host
(USB/BOOTSEL) with the right cards, reload, compare digests. An operator who
doesn't know that is stranded in front of a screen telling them to scan.

**Bounded fix.** S1 gains one test and one deliverable line: a named
under-supply refusal stating what the payload supplied vs. what n needs and
that the remedy is rewriting the payload on the host; the incomplete-chunk-set
drop message loses the word "scan" on this path. Message text is cheap; a dead
instruction on a device whose other buttons cut steel is not.

## F7 — Walk-away: seeds in RAM with no bound

**How reached.** Between plates, the tail waits at "hold button to start" /
plate-done indefinitely. Verified in `gui/run_flow.go:361-401`: idle after 3
minutes brings the **screensaver**; the wipe branch requires `armed`, and
`wipeGuard.armed()` is bracketed exclusively by the unlock secret session
(`gui/wipe_guard.go`). The build flow holds — post-S5, legitimately, per §4.2 —
several masters' working seeds for the duration of a multi-hour set, and no
timer covers it.

**What exists afterwards.** An unattended, powered machine holding multiple
seeds in RAM, mid-set. The unlock flow's §10.2.4 names this exact window
("walk-away states with secrets still held are ARMED") and engineered the
warn-then-wipe for it, including the disarm-while-cutting rule so the needle is
never down during a wipe.

**Operator visibility / route out.** The screensaver, indistinguishable from an
idle machine holding nothing.

**Bounded fix.** A ruling in S5, either way: bracket the constructor + tail
with the existing `wipeGuard` (the mechanism already solves needle-down and
mid-derivation edges; a wipe between plates aborts the set, which F1's
determinism property makes completable by re-run), **or** record explicitly
that the build flow's walk-away window is accepted for phase 1 and why. The
gap is the silence, not necessarily the answer.

## M1 — Back semantics of the new S4/S5 screens

`buildParamPickFlow` documents Back stage-by-stage; the new surfaces (per-slot
source picker, per-slot seed entry, slot review) have no ruled Back behavior.
The gate cannot go stale (it runs at construction, after all edits — see
"already handled"), so what remains is bookkeeping: whether a seed entered for
a slot that is then reassigned stays in the registry. §4.2 permits retention
for the constructor's duration, so any answer is compliant; an unruled one just
invites divergent implementation and a scrub test written against the wrong
inventory. One sentence in S4.

## M2 — The ms1 is a Go string and cannot be scrubbed

`deriveMultisigLeg` wipes the entropy buffer but returns `b.MS1` as a `string`
(`gui/multisig_derive.go:64-72`), which then rides `bundleCard.strings` through
the whole tail. Strings are immutable and GC-copied; no scrub can reach them.
§4.2's "working copies MUST NOT outlive the flow" is therefore unsatisfiable
as written for the ms1 representation — the same unsatisfiable-MUST class §4.2
was already corrected for once. F3's scrub test must scope itself to the
scrubbable buffers (`bip39.Mnemonic`, entropy slices) and the plan should name
the string residue as a known blind spot rather than let the test claim more
than it proves. (Existing behavior; the plan multiplies ms1 count, not the
class.)

---

## Already handled — probed and found sound

- **Single-plate pause/resume/retry within a powered session.**
  Back-while-running → `engraveStopped` → hold-to-resume with catch-up motion
  planned at the plate's own config (`engraveJob.catchup`, mutation-tested per
  its own comment); driver failure → "Hold button to retry" on the same
  resume state; resume-state release is terminal-only (`releaseResumeState`),
  with the wipe-path residue already filed as F-110. The emulator can already
  verify the resumed path is the same path (`shToolpath` digest) — F1 asks only
  that a walk actually use it.
- **Set-level abort on operator action.** Every operator-driven exit of
  `bundleEngrave` (variant-picker Back, `validateMdmk` failure) reaches
  `bundleAbortWarning` and records no completed state (I-5) — the warning's
  *text* is F2; its *reachability* is sound.
- **Gate staleness under back-navigation.** Spec §4.3 pins the seed↔key gate
  "at construction time", which follows every edit; `buildReviewFlow` Back
  aborts rather than re-edits; S5 test 7 re-proves the gate through the real
  post-rewire flow with the origin-binding mutation. There is no path that
  reaches assembly around the gate.
- **The S2 interim origin refusal, both directions.** Added the moment engraves
  complete (closing D-2's silent stamping), removed in the same unsplittable
  stage (S5) that rewires origins, with S5 test 7 covering the seam. Both edges
  of the window are designed.
- **Reboot and the payload.** The payload region is flash and survives power
  loss; re-entry is Load Payload → digest compare, and S1 test 2's
  mutation-checked `!loaded || !compared` refusal on `takeAll` means a reboot
  cannot skip re-authentication. A payload-sourced seed does not need retyping
  to recover a set — flash-resident residue is §5.3's stated, accepted threat.
- **Pending chunk sets at Done.** `bundleDoneDecision` warns and drops a
  partial card rather than engraving it (message text aside — F6).
- **Determinism as the recovery foundation.** No randomness in `md/`, `mk/`,
  `codex32/` (measured); mk `chunk_set_id` is bytecode-derived; so restart
  recovery is *possible* and mixed-run sets are either byte-identical or
  stub-mismatched (fail-closed at a host restore). The plan needs only to say
  and pin it (F1).

## Severity summary

0 Critical / 7 Important (F1–F7) / 3 Minor (M1, M2, and F6's message-text
subset if the refusal itself is judged spec-compliant as-is). F2 and F5 are the
two that would graduate to Critical if phase 1 shipped beyond the operator's
own bench: one puts a funded master's seed in the trash by instruction, the
other engraves a quorum that lies.
