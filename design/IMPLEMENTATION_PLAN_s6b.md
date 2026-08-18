# IMPLEMENTATION PLAN — S6b

**Status: DRAFT, ungated.** No code until this passes review at 0C/0I.

**This is a SCHEDULING document, not a design document.** Every design decision
is already settled and reviewed in `SPEC_s6b_pre_flash_cycle.md` (GREEN across
four lenses: correctness ×3, comprehension, unfounded assumptions, rewrite
fidelity) and in `REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis (R-A … R-M). **This
document decides only ORDER, PHASING and PER-PHASE EXIT.** It does not restate,
reinterpret or extend the spec — where the two ever disagree, **the spec wins**.

---

## 0. HOW THE CYCLE RUNS

- **One implementer at a time**, in a git worktree off the fork. Never parallel
  implementers: this is execution against a frozen design, and parallel attempts
  produce reconciliation work rather than coverage.
- **TDD per phase** — the phase's gates are written as failing tests *before* its
  implementation.
- **The controller folds small post-review fixes inline** rather than dispatching
  a fresh implementer for a one-line change.
- **Ultracode is OFF for implementation** and ON again for the whole-diff review
  at the end.

### The test suite is near a hard ceiling, and it binds every phase

| invocation | wall |
| --- | --- |
| `go test ./gui/ -count=1` scoped by a `-run` filter to 2 tests | **~6-7 s** |
| `go test ./gui/ -count=1` | ~430-450 s |
| `go test ./... -count=1` | ~436 s |

`gui` runs **429–507 s against Go's 600 s per-package default** (~71–85%).
A previous cycle's draft blew straight through it and the package died at 600 s
mid-engrave **with every assertion passing** — a timeout is not a test failure,
and the fix is never to delete assertions or raise the limit reflexively.

**Scope with `-run` while iterating; run the full suite once per phase gate,
with stdout and stderr captured to separate files.** Narrowing `./...` to
`./gui/` buys nothing — `gui` is essentially the whole suite (**440.6 s of
473.5 s at P3 — 93%**).

### THE GATE MUST PASS `-timeout`, from P5 onward. Measured, not precautionary.

| phase | `gui` wall |
| --- | --- |
| P3 | 440.6 s |
| P4 | **496.8 s** (+56.2 s in one phase, **83% of the 600 s default**) |
| P5 projected | ~553 s (92%) |
| **P6 projected** | **~609 s — OVER** |

**Every gate run from P5 on passes `-timeout 20m` explicitly.**

**This is not the reflexive limit-raising the runbook forbids.** That warning is
about raising a limit to turn a red run green, hiding a real failure. This is the
opposite: the default is about to convert a **passing** suite into a fake failure.
A package that dies at 600 s reports **every assertion passing** — S6a lost a
whole cycle-step to exactly that. Raising it preemptively, with the growth
measured and stated, keeps a timeout meaning *"something hung"* instead of
*"the suite got longer"*.

### Use the cores: 24 available, `gui` uses ONE

`gui` has **zero** `t.Parallel()` call sites and **zero** `testing.Short()`
guards, so its ~500 s runs on a single core while 23 sit idle.

**Shard the gate across parallel `-run` invocations** — each shard is its own
process with its own timeout budget and its own binary. No code change, and it
removes the ceiling cliff as a side effect rather than postponing it.

**Committed as `scripts/gui-shard-test.sh`, and MEASURED:**

```
814 top-level tests   partition verified exhaustive: 814 == 814
6 shards, all ok, stderr empty      WALL: 106s   (longest shard 105.5s)
```

**versus ~497 s at P4 — a 4.2× reduction**, with no shard within a factor of ten
of its timeout.

**Use it for P6's gate and for the whole-diff review:**

```
./scripts/gui-shard-test.sh ./gui/ 6 20m
```

**Its safety property is why it is a script and not a shell one-liner.** The
obvious way to shard is a hand-written "everything else" regex, which **silently
drops every test added afterwards** — and a shard set that skips tests still
reports `ok`. This project's own rule is that a skipped gate is the default
failure mode. So the script enumerates from `go test -list`, partitions
programmatically, and **asserts the union equals the full set before running
anything**, refusing to start if one test would be missed. It also refuses to
report success on zero tests enumerated, and fails if any shard's stderr is
non-empty rather than only on exit status.

It prints what it does **not** cover: subtests are not partitioned, one package
only, and it cannot detect tests coupled through side effects — though sharding
*exposes* such coupling as a failure rather than hiding it. **If a shard fails
where a full run passes, suspect coupling before suspecting the shard.**

**Do NOT add `t.Parallel()` to `gui` as a speed fix.** The package carries global
state (`Context`, `Platform`, the emulator harness), and cross-test interference
produces flakes. A flake on a funds-critical gate is worse than a slow gate: it
teaches people to re-run until green.

**Toolchain:** go1.26.3 at
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin` (CI pins 1.26 via
`actions/setup-go@v6`). The older runbook line
`export PATH="/nix/var/nix/profiles/default/bin:$PATH"` is **wrong** — that
profile contains no `go`, and yields `command not found`, which proves nothing.

**Two `go vet` go1.26 `t.ArtifactDir()` failures are PRE-EXISTING** and not
yours: `gui/freetext_sizeproof_golden_test.go:111` and `gui/op/draw_test.go:176`.
Two files, not one.

---

## 1. PHASE ORDER, AND WHY

Six phases. The order is forced by three dependencies and one convenience.

| # | phase | spec | why here |
| --- | --- | --- | --- |
| **P1** | verify tail | §3 | self-contained copy + control flow, no new mechanism, lowest risk — and it lands R-M's new modal body early so P6 can sweep it |
| **P2** | plate marking | §1 | introduces the `Text` fields and the `bundleEngrave` parameters; touches four callers and three source-text tests |
| **P3** | passphrase plate | §2 | needs the largest new mechanism (preloaded entry, new `syswSource`, provenance field, policy id) |
| **P4** | restore document | §6 | **depends on P3** — its condition reads the result P3's flow starts returning |
| **P5** | scroll arrows | §5 | independent; R-I's layout costs no body width, so it does not disturb P6 |
| **P6** | modal fit sweep, S6b's own modals | §4 | scheduled last so it covers every modal this cycle added or changed, including P1's R-M body |
| **P7** | modal fit sweep, **the rest of the firmware** | §4 | **added 2026-08-18** — P6's scope was narrower than §4's, see below |

### P7 exists because THIS PLAN SILENTLY NARROWED THE SPEC

**Spec §4:** *"**Every** long modal body is gated by the F-185 class check. Only
S5.C's screens are measured today."* — firmware-wide, matching **F-192**'s filed
text: *"every other long modal in the firmware carries the same unmeasured
exposure."*

**This plan's P6 row said:** *"every modal **this cycle added or changed**."*

That is a narrower requirement wearing the same name. P6 followed the plan,
**noticed, and documented both readings instead of picking one** — which is the
only reason it surfaced before S6b closed with F-192 marked done.

**The plan review did not catch it.** That review was briefed to find
requirements with **no owner**; it never asked whether a requirement had been
**shrunk**. A narrowed requirement still has an owner, still has a gate, and
still goes green — it is invisible to an ownership lens by construction.
**Scope-narrowing is its own review question and is not covered by "is anything
unowned".**

### The scope was BOUNDED before P7 was dispatched, not after

A `go/ast` scan over every `showError`/`showNotice` body argument in `gui`:

| population | at risk |
| --- | --- |
| **92 literal bodies** | **no** — longest 152 chars against F-185's ~500-char cut |
| **~36 composed bodies** (functions, `Sprintf`) | **yes — this is P7** |

An earlier regex attempt at the same count returned noise (it swallowed comment
blocks and reported an 8210-character "modal body"). It was discarded rather
than reported. **A measurement that cannot distinguish a comment from a string
literal cannot bound a sweep**, and the parser-based one can.

**The three real dependencies:**

1. **§1.1 before §1.2/§1.3** — the optional `Title`/`Footer` fields must exist
   before anything conditions them. Internal to P2.
2. **P3 before P4** — §6 conditions on *"this run cut a passphrase plate"*, and
   the flow returning that result is P3's work (`engravePassphraseFlowFrom`
   returns nothing today).
3. **P6 last** — the sweep must run over every modal this cycle added or
   changed, R-M's replacement body among them. *(This is a property of P6 being
   last, not a constraint distinguishing P1's position: P1 could sit anywhere
   before P6 and the requirement would still hold. Stated this way so the plan
   does not claim more forcing than it has — only dependencies 1 and 2 actually
   pin a phase's place.)*

**P5 is order-free.** R-I chose a layout costing no body width precisely so the
arrows and the fit sweep stopped blocking each other; nothing here re-couples
them.

---

## 2. PER-PHASE EXIT

A phase is done when **its spec gates pass as tests** and the **full suite is
green**. Gates are named by their spec numbers — the spec is the authority on
what each one means.

| phase | gates that must be green | goldens expected to move |
| --- | --- | --- |
| **P1** | 3.1, 3.2, 3.2a, 3.3 | none |
| **P2** | 1.1, 1.2, 1.2a, 1.2b, 1.3 | **none** — GATE 1.1 says `text-{0,1,2}-shards-1.bin` must **not** move |
| **P3** | 2.1, 2.2, 2.3, 2.3b, 2.3c, 2.3d, 2.4a, 2.4b, 2.4c, 2.5, 2.6 | `passphrase-*.bin` (4), re-recorded **in the same commit** |
| **P4** | 6, 6a, 6.1 | none |
| **P5** | 5.1, 5.1b, 5.3 | `sizeproof-{front,back}.bin` may move — they are designed to |
| **P6** | 4 | none |

### Two golden rules that are easy to get wrong

- **P2's expected churn is ZERO.** If `backup/testdata/text-{0,1,2}-shards-1.bin`
  move, that is **a defect in the optionality**, not a golden to refresh — R-F's
  empty-title path must be byte-identical to today. Treat a moved byte as a
  finding.
- **Never run a bare `go test ./... -update`.** It rewrites the frozen sixteen in
  `backup/testdata`. Scope every regeneration with `-run`.

### GATE 5.1b is expected to FAIL and does not gate

It is R-E's divergence probe on `maxScroll`. Its failures are **findings to
file**, not a gate to weaken. GATE 5.1 — the new predicate against actual
visibility — is the one that must be green. Recording this here because a red
result on a probe reads as "the gate failed, loosen it", and that is how a
false-PASS gate is born.

> **REVISED 2026-08-18 at the S6b merge, by operator ruling — the probe now
> PINS the gap instead of failing on it.** The paragraph above is right about
> the danger and wrong about the remedy, and the two halves came apart at the
> merge: `.github/workflows/test.yml` in the fork runs `go test` on every push
> and skips nothing (operator directive 2026-08-15), so a permanently failing
> test **does** gate — it makes `main` red forever and hides the next real
> failure inside an already-red run. "Does not gate" was never implemented.
>
> `TestGate51bMaxScrollAgreesWithVisibility` now asserts the divergence is
> **exactly 22 values spanning `[239,260]`** — the measured gap. It is not
> loosened; it is **strictly more sensitive**, and both directions were
> mutation-proved before the merge:
>
> - predicates made to agree (fadeClip restored) → **FAILS**, naming that
>   outcome and saying to delete the gate. The old shape went green and told
>   nobody.
> - geometry shifted by one → **FAILS** with `23 values spanning [238,260]`.
>   The old shape could not tell 22 divergences from 200 — both were just red,
>   which is the state it already sat in.
>
> The danger this section names is real and unchanged: **do not re-pin the
> constants to whatever a red run prints.** The test says so itself. A moved
> gap is either the deferred honest-geometry work landing, or a defect.

---

## 3. WHAT EACH PHASE MUST NOT DO

Stated per phase because these are the boundaries a reasonable implementer would
otherwise cross.

- **P1** — must not widen `verifyRefused`; only `:753` re-offers, and the gate
  asserts the other three do not loop (`:854` by source assertion, since it is
  unreachable in-process).
- **P2** — must not mark `"Engrave Multisig"`, `"Build Policy"`,
  `"Engrave Bundle"`, `mdmkFlow`, `deriveXpubFlow` or any `cardMS1` plate. Must
  not use a variadic tail or shared state on `Context` to avoid editing callers:
  **Go has no default parameters**, every caller passes `""`, `""`, and three
  tests assert the call text as a source string.
- **P3** — must not leave the fingerprint-entry steps present-but-skipped; they
  are **elided from the sequence**, because the Back transition is `step -= 2`.
  Must not use `md.WalletPolicyIDStub` (the keyed branch) — the form-aware
  function, computed from the **post-`templateizeBundle`** `b.MD1`. Must not key
  the footer on policy-id presence; it reads a recorded provenance. **Must not
  carry the preloaded values on a package-level variable or a field on
  `Context`** (spec §2.1.3) — either would make a secret-adjacent value outlive
  one flow. They are **parameters**. *(P2 carries the same prohibition against
  shared state for a different reason — there it would cross R-B; here it is
  secret lifetime. Both are stated because the cheap shortcut is identical.)*
- **P4** — must not condition on the offer being *shown*; the condition is
  **cut**. Must not add the passphrase plate to `bundlePlatePlan`'s count.
- **P5** — must not change body width from 417, and must not route `Up`/`Down`
  through `layoutNavigation` (it indexes `Button - Button1` into a `[3]int`).
- **P6** — must not silently skip a modal; the sweep states its own coverage.

---

## 4. COMMIT AND REVIEW SHAPE

- One phase per commit series in the fork worktree, with the phase's gate output
  in the commit message.
- **The `me` CLI is untouched and `md1`/`mk1`/`ms1` stay byte-identical** — this
  is what keeps S6b fork-native and out of the Rust-primary rule. Re-verify at
  the end, not by assertion.
- **After the last phase: a mandatory, non-deferrable independent adversarial
  review over the WHOLE DIFF.** The spec's R0 covered plan correctness; that
  review catches implementation-introduced regressions TDD misses. It is not
  optional and does not fold into a phase review.
- Reports persist to `design/agent-reports/`; the controller commits each in its
  own commit before folding.

---

## 5. WHAT THIS PLAN DOES NOT DECIDE

- Anything in the spec's §8 — the no-passphrase arm's wording, §2.4's
  both-forms label, §3.2's optional third arm, and R-H's band placement.
- Whether `gui/singlesig_derive.go:28`'s stale doc comment is fixed in P3's
  commit or separately — the spec requires it land with the §2.4 change either
  way.
