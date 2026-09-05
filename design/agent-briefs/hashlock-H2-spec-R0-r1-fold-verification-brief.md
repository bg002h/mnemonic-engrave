You are the INDEPENDENT fold-verification reviewer (sonnet tier, targeted) for round 1 of the R0 gate on `design/SPEC_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`). Round 0 = two opus lenses, persisted verbatim: fidelity at `3f88280` — `design/agent-reports/hashlock-H2-spec-R0-r0-fidelity.md`, 3C/5I/6M/2N — and journey at `a70f950` — `design/agent-reports/hashlock-H2-spec-R0-r0-journey.md`, 2C/6I/5M/1N. The fold is ONE commit, `60a86f6`, over the draft `bfd042e`; its message and the spec's closing "R0 round 0 folded here" paragraph map every finding to a change.

ONE QUESTION: did the fold address every Critical and Important from both reports — FIXED / PARTIAL / NOT FIXED / DECLINED-with-reason, one line each — without introducing a contradiction or a false claim of its own?

Read-only on every repo (`/scratch/code/shibboleth/mnemonic-engrave`, `/scratch/code/shibboleth/seedhammer` at main `c4a64fc`, `/scratch/code/shibboleth/mnemonic-secret` at `cd0a60f`); commit nothing; no sub-agents; no scratch copies; read no `.jsonl`.

## Already settled — do not re-derive
- The two reports are the specification for this round; read them first, then `git show 60a86f6`.
- The controller re-grepped the fold's new citations at `c4a64fc` (`composer_shape.go:269/:443`, `composerEveryPathHashed` in `composer_state.go:239`, `composer_copy.go:169-173`, `unlock_kdf.go:26/:219-221`, `sysw/open.go:55`, `seal/open.go:231`, `md/compose.go:167`) and at ms `cd0a60f` (`argv_guard.rs:148-164`, `format.rs:35`). Do not re-report line numbers; DO report a mechanism described wrongly.
- Rulings L5, L7, L12, L15, L16, L22, L24 stand. Secret-handling never gates.

## Verify
1. **The finding table**: 5 Criticals + 11 Importants across both reports — the spec section that now carries each fix, quoted, and a verdict; then the 11 Minors and 3 Nits, one line each.
2. **The three fidelity Criticals, checked against the sources**: (C-1) does §2 now name a forbidden MECHANISM, and do §7.1/§7.2/§7.5 drive rows that are NOT fixed points of `seal.NormalisePassphrase` — confirm by reading `seal.NormalisePassphrase` in the fork and applying it to `Correct Horse Battery Staple`, `  a  b `, `correct-horse,battery staple` (from ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json`'s `derivation` rows — check those phrases exist there); (C-2) is §2 rule 3 now the host's predicate exactly — compare clause by clause with `looks_like_ms1`/`is_ms1_shaped` and `strip_display_separators` (which characters?), and confirm the five ms1-shaped `refusals` rows the spec cites exist in the corpus; (C-3) `b867db87..edbc96cb` — recompute from the corpus's `sha256_h` for the anchor row.
3. **The two journey Criticals**: does §4.5 now carry an unconditional backup line and a reconciliation line, and a relation line only when payload records exist; does §4.7 give §8h a phrase-route form and keep it at Done? Then measure the §4.5 body against the fork's modal-fits gate in `gui/composer_copy_test.go` (find the gate's actual capacity/margin constants — the journey lens cited 588/80; confirm or correct) and say whether the longest variant (relation line present, `chars: 100`) fits, quoting the count.
4. **The Back contract (§4.6)**: is it stated once and consistently everywhere Back is mentioned (§1, §4.2, §4.3, §4.4, §4.5, §4.6, §7.2)? Any sentence still saying "Back discards" or "returns to Which hash?" from a screen §4.6 sends elsewhere is a contradiction.
5. **New contradictions**: the fold rewrote §1-§9 and §10; read as a hostile implementer for any two sentences that disagree, any copy quoted two ways, and any test in §7 that cannot fail on what it names.

## Severity
A C/I marked FIXED but not fixed = Critical. A new contradiction between two normative sentences, or a claim about the fork/host that is false = Important. Wording = Minor/Nit. A clean round closes R0 for this spec (lens-closure: fidelity/design, journey/adversarial, fold-verification).

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-spec-R0-r1-fold-verification.md` (create; must not exist): the finding table (16 C/I rows, then M/N), the executed checks, the modal-fit count, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
