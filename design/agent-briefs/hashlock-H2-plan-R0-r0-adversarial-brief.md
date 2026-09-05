You are the INDEPENDENT adversarial / failure-states reviewer (opus tier) for round 0 of the R0 gate on `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` in mnemonic-engrave (`/scratch/code/shibboleth/mnemonic-engrave`, plan at `<PLAN_SHA>`). Spec: `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`). Fork: `/scratch/code/shibboleth/seedhammer` main `c4a64fc` (read-only). Host reference: mnemonic-secret `cd0a60f`; corpus `crates/ms-codec/tests/vectors/hashlock-v0.8.json`.

ONE QUESTION: construct the inputs, timings and states under which the device the PLAN builds fails -- derives a wrong digest, panics, hangs, leaks the phrase, corrupts composer state, or reports success falsely -- and show each with a trace through the plan's code (Task 1-4 blocks) and the fork functions it calls.

Attack surface (construct, do not assess):
1. **Bytes**: every corpus `refusals` row typed on the device; a phrase of 100 spaces; `0x7F`; a multi-byte UTF-8 character if the keyboard can produce one (can it?); a phrase equal to `ms1` + 45 bech32 chars; a 64-hex phrase with one uppercase letter; the empty string; a phrase with leading/trailing/double spaces (the host's rule byte for byte -- NO normalisation; prove the plan's `ValidatePhrase` and the keyboard path never alter a byte, or show the byte they alter).
2. **The KDF**: `seal.NewDeriver(passphrase, salt, iterations)` with the plan's salt slice and 100_000; `Step(kdfStepIterations)` accounting (does the loop perform exactly 100_000 -- read `seal/pbkdf2.go`); `Key()` length; a second `Step` after `Done`; `Wipe` before `Key` read; the goroutine/scheduler model (`-scheduler tasks`): does a 30 s derivation starve the UI or the watchdog; what happens on a button press mid-derivation; does progress false actually abandon (trace `DeriveHardened`).
3. **State**: `composerHashEdit` returning false from the phrase route on a NEW path (`composerAddPath` deletes it) vs an EXISTING path; `Hash` assigned before HOLD; `hashByPhrase` set but the path later removed; the seat-discard guard (W-7) interplay when Back leaves the composer with a phrase-set hash; `st.list` mutation while the confirm modal is open; re-entering the phrase route with a hash already set (edit); the `default` arm of the label switch (panic in production -- reachable?).
4. **Display**: the confirm body at the longest legal phrase and digest lines -- `assertModalBodyFits` covers the rows the plan adds, but does the PRODUCTION body builder produce exactly those rows (or can a runtime body differ from the tested one: dynamic line count, width, the relation line present/absent)?
5. **Firmware**: `-stack-size 16kb` with PBKDF2's HMAC state + the composer's frames; flash budget after Task 1's package (the plan's size step); TinyGo-incompatible constructs in the plan's code (reflection, `fmt` in hot paths, maps in the lockstep table -- read the plan's blocks).
6. **The gate**: could the lockstep corpus test pass while `DeriveHardened` is wrong (e.g. the test recomputing expectations with the code under test, comparing lengths only, or the corpus vendored copy diverging from ms's -- what pins it)?

Read-only; commit nothing; no sub-agents; read no `.jsonl`; no scratch copies (you may READ `/scratch/code/shibboleth/.tmp/h2-gate` for the wired tree; never modify it).

## Already settled -- do not re-derive
The gate proved the tree compiles and the plan's tests pass (`design/agent-reports/hashlock-H2-plan-build-gate.md`). The spec's rulings stand. Secret-handling never gates (report leaks as Minor with the trace).

## Severity
Critical: a constructed input/state where the device's digest differs from the host's, a panic/hang reachable by the operator, composer state loss, or a false-green gate. Important: an unsound assumption in the plan's code with a constructed trigger. Minor/Nit: leaks, wording.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-H2-plan-R0-r0-adversarial.md` (create; must not exist): findings `### C-n / I-n / M-n / N-n -- title` each with the constructed input/state, the trace (file:line in the plan and the fork), the wrong outcome, and a SUGGESTION; an attack table (surface item, constructed case, outcome); closing counts. Return a two-line summary plus the path.
