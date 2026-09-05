You are the INDEPENDENT fold-verification reviewer (sonnet tier, narrowly scoped) for round 2 of the R0 gate on `design/SPEC_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 1's report is `design/agent-reports/hashlock-H2-spec-R0-r1-fold-verification.md` (`040e85f`): 5/5 C and 10/11 I fixed, fidelity I-5 partial, and two new Importants the round-0 fold introduced — NF-A (a false mutation claim: `correct-horse,battery staple` is a fixed point of `seal.NormalisePassphrase`) and NF-B (a wrong fit-gate citation: "588 / composer_copy_test.go" is not the gate). The r2 fold is ONE commit, `c06a760`, over `60a86f6`; its message and the spec's "R0 round 1 folded here" paragraph map each item.

ONE QUESTION: does the r2 fold fix NF-A, NF-B and the I-5 partial — with every new claim about the fork true — and introduce no contradiction of its own?

Read-only on every repo (`/scratch/code/shibboleth/mnemonic-engrave`; `/scratch/code/shibboleth/seedhammer` at main `c4a64fc`; `/scratch/code/shibboleth/mnemonic-secret` at `cd0a60f`); commit nothing; no sub-agents; no scratch copies; read no `.jsonl`.

## Already settled — do not re-derive
- Rounds 0 and 1 closed every other C/I; do not re-review them. Rulings L5, L7, L12, L15, L16, L22, L24 stand.
- Scope is §2 (the normaliser sentences), §4's preamble, §4.4's one clause, §4.5 (the confirm surface, HOLD, the drop order), §7.1's mutation bullet, §7.2's geometry sentence, §10's three new rows, the opening paragraph and §1 item 5 (I-5), and the "R0 round 1 folded here" paragraph.

## Verify (execute; quote)
1. **NF-A.** Apply `seal.NormalisePassphrase` (`seal/open.go:76-78`) by hand or with a throwaway `go run` in `/scratch/code/shibboleth/.tmp` (never inside the fork tree) to the three corpus phrases `Correct Horse Battery Staple`, `  a  b `, `correct-horse,battery staple` (from ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` — confirm the exact strings there). Does §2 now credit exactly the rows the normaliser changes, and does §7.1's mutation bullet name a mutation the separators row CAN catch (stripping display separators — which characters does ms-cli's `format::strip_display_separators` strip, and does the row contain them)?
2. **NF-B.** Read `gui/modal_fits_test.go`: confirm `assertModalBodyFits` (:201), `modalHeadroom` (:182), `firstModalFrame` (:140), `normalizeDrawn` (:60), `modalBodyMargin = 80` (:51), and that no capacity constant exists; confirm §4, §4.5, §7.2 and §10 now describe that mechanism correctly and require every new body to be in the table. Is the §4.5 drop order well-formed (each step names a concrete edit; the never-dropped lines are stated)?
3. **I-5.** Confirm `gui/composer_hash.go:27-28`'s current text and that §1 item 5 gives the replacement; confirm `SPEC_wallet_policy_composer.md:386` still carries the sentence and that the spec now says H3 folds it (no present-tense "are folded" claim remains anywhere — grep the spec).
4. **M-4 / M-2.** `composerConfirmScreen` (`gui/composer_shape.go:77`) and `composerConfirmBody` ("Hold button to confirm.", `gui/composer_copy.go:32-33`): is §4.5 consistent (HOLD to confirm, Back to decline) with §4.6 and §7.2 wherever the confirm step is mentioned (grep CONTINUE — any leftover?). §4.4's forbid clause present.
5. **New contradictions** within the scoped sections, read as a hostile implementer.

## Severity
NF-A/NF-B/I-5 not fixed = Important. A new false claim about the fork or a contradiction = Important. Wording = Minor/Nit. A clean round closes R0 for this spec.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-spec-R0-r2-fold-verification.md` (create; must not exist): the executed checks with output, a verdict per item, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
