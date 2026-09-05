You are the INDEPENDENT fidelity + design reviewer (opus tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan committed at `e672194`). The spec is `mnemonic-secret/design/SPEC_ms_hashlock.md` §1 rule 2, §3, §9 (in `/scratch/code/shibboleth/mnemonic-secret/design/`); the follow-up is F-473 in `design/FOLLOWUPS.md`; the codec it bumps to is ms-codec 0.8.0 (source: `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec` at `cd0a60f`, the same bytes as the crates.io release).

ONE QUESTION: if an implementer follows this plan literally, does `me` at ms-codec 0.8 refuse a kind-0x03 PREIMAGE plate by name on every path that packs or seals a record — and does the plan claim nothing false about the code and API it cites?

Read-only; commit nothing; no sub-agents; read no `.jsonl`. Read code freely; do not create scratch copies (a tests reviewer is running the plan concurrently in its own copies).

## Already settled — do not re-derive
- The build gate ran (controller hand-wire; output in `git show e672194`): the bump builds; the RED step is five failures at the bare bump; Tasks 2 and 3 turn them green; three mutations each fail the named tests; whole crate 619/616 (three box-local `history_purge`); fmt clean. Do not report compile/test findings on gated blocks.
- Operator rulings L24 (TagKindMismatch refused) and L26 (released regardless of the device) stand. Secret-handling never gates.
- H0 (merged, `024dd08`) already added `RecordError::PreimagePlate`, `UnknownReason::PreimagePlate`, `preimage_plate`, the seam corpus rows and the tripwire tests; this plan only re-points them for the bump.

## Verify (construct counterexamples; an assessment without one is Minor at most)
1. **Every path that reaches `ms_codec::decode` in `me`** (`grep -rn "ms_codec::decode\|ms_codec::" crates/me-cli/src`): does each one either go through `validate_record` (now guarded) or handle `Payload::Preimage` itself? The seam corpus test, `bip93_outside_the_profile`, `preimage_plate`, `sysw::classify`, `seal`'s admission, the descriptor gate — trace each. A path that could treat a decoded preimage as a seed or a placeable record is Critical.
2. **The `#[non_exhaustive]` matches.** Task 2's `match ms_codec::decode(s)` has `Ok(_)`; is any OTHER match on codec types in `me` exhaustive and therefore broken by the bump (the compiler would say so — check whether the gate's build proves it: it built)? Is there any place that RELIES on `ms_codec::decode` failing for 0x03 (e.g. a test or doc that asserts `ReservedPrefixViolation`)?
3. **The seam corpus at 0.8.** Rows `preimage-plate-0x03`, `preimage-shape-entr-id`, `bip93-plain-33-byte-payload-0x03`: what does the host now answer for each (`sysw::classify`), and does `host_admits: false` still hold for all three? `preimage-shape-entr-id` decodes to `TagKindMismatch` at 0.8 — confirm Task 3's arm names it and that `me seal` on it gives a sane refusal (the plan leaves `me seal`'s mismatch wording to the codec's Display — is that acceptable, or does §1 rule 2 owe the operator more?).
4. **Records.** Does the plan's Task 4 (me 0.8.1, CHANGELOG, F-473 closed) leave anything stale — the `RELEASE`/`release.yml` expectations, the H0 CHANGELOG entry's "unreleased", the composer spec's "me 0.8.1" mention in the hashlock records? Point at the sentence.
5. **Anchors.** Every `Modify` names anchor text; check each exists exactly once at `0f5ce23`.

## Severity
Critical: a path that treats a decoded preimage as a seed/placeable record; a false claim about the API. Important: a missing case, an unsound assumption, an unappliable fragment. Minor/Nit: wording, records.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H1b-plan-R0-r0-fidelity.md` (create; must not exist): findings `### C-n / I-n / M-n / N-n — title` each with the plan section, the counterexample or trace, and a SUGGESTION; the path table from item 1; closing counts. Return a two-line summary plus the path.
