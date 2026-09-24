# F-671 independent review

**Scope.** Fork `f3586db..bc7cbe7` (`git -C sh-worktrees/f671 diff`), engrave `80735588`
(spec + FOLLOWUPS only), the F-671 FOLLOWUPS entry, and
`design/agent-reports/f671-impl.md`. Question: does every screen now tell the
truth about whether Liana imports the wallet AS SEATED, and does anything else
break. Reviewer model: sonnet (mechanical/verification pass per project tiering).

## 1. One rule, three sites

`composerLianaRefusesSeating(st) bool { return len(composerSharedSeedInPath(st)) > 0 }`
(`gui/composer_review.go:199-201`) is the only function with that name; grepped
for it and for every other "Liana ... refus" / "... import ... Liana" string in
non-test `.go` files.

Production call graph, verified by grep (each has exactly one non-test caller):
- **Mapping review** (§8g): `composerSharedSeedInPath(st)` directly, at
  `composer_review.go:253` (the per-path body loop) — the same finder the
  predicate wraps, not a second copy of the rule.
- **Consent**: `composer_selfcheck.go:245`, the only non-test call site of
  `composerConsentLinesFor`, passes `composerLianaRefusesSeating(st)`.
- **Key-path chooser**: `composer_unspendable.go:94`, the only non-test call
  site of `composerUnspendableRows(st)`.

Every other Liana string in non-test code is a different, pre-existing fact
unaffected by this diff: the structural "outside Liana's model" refusal
(`composerLianaOutsideModelClass`/drop-cause "Liana would not import this
policy (…)"), the hashlock refusal, the NUMS body, and the kind-naming-only
print sites (`md1_inspect.go`, `template_engrave.go`). None of them decide the
same-seed-in-path question a second way. **Verdict: exactly one rule, three
consistent call sites.**

## 2. Does the rule match Liana v15.0

Read `liana/src/descriptors/analysis.rs` (checkout at
`.tmp/fable-liana-src-v15/liana`, the harness's own dependency). `LianaPolicy::_new`
builds ONE `DescKeyChecker` shared across the whole spending-path iterator (global
exact-xpub dedup, `DuplicateKey`) and, **separately, a fresh `origin_fingerprints`
HashSet created inside the loop for each `PathInfo::Multi` path only**
(`DuplicateOriginSamePath`) — never for `PathInfo::Single`, and never across
paths. `composerSharedSeedInPath` mirrors this exactly: it groups by
`(path, fingerprint)`, requires `p.Keys != nil` (skips keyless/internal
branches), and only flags a group with `>= 2` slots — matching "same origin
fingerprint twice inside one Multi path." Cross-path fingerprint reuse is a
different, pre-existing function (`composerPersonInTwoPaths`, §8k,
informational-only), which is the correct classification: I traced Liana's own
code path for a seed reused as one 2-of-3 signer AND the separate 1-key
recovery leaf (the walk's own scenario) and confirmed the recovery leg is
`PathInfo::Single`, so Liana raises nothing for it — cross-path reuse is
genuinely unrefused, matching the composer's §8k treatment.

This is not just a source read: `design/agent-reports/e2e-live-site-wallets.md`
D-1 measured the **actual** wallet this fix targets against real Liana v15.0
(`LianaDescriptor::from_str`, the GUI's own entry point) and got the exact
refusal text the copy now echoes: `derived from the same origin as another key
present in the same spending path`.

**Counterexample search:** none found. I looked for (a) a false positive from
BIP-32 fingerprint collision — theoretical, applies equally to Liana's own
check, not composer-specific; (b) a false negative from `fpPresent=false`
(e.g. a malformed/fingerprint-less mk1 card) — real, but it is a property of
`composerSharedSeedInPath` itself, which predates F-671 and already gated
§8g; F-671 only wires the *existing* finder into two more screens, so it
inherits rather than introduces this edge case, and net effect is strictly
better (two screens went from an affirmatively wrong claim to whatever the
existing finder already gets right on the mapping review). **Not a new
defect. Logged as a pre-existing limitation, not blocking.**

## 3. The carried-over Nunchuk sentence

**Not established, and I'm saying so rather than guessing.** The same-seed
body keeps "Nunchuk imports it only when the keys happen to be in sorted
order," which was measured (R-1, `design/DESIGN_coordinator_compatibility.md`,
`IMPORTABILITY_composer_shapes.md`) for wallets with **distinct** signers, not
for a same-seed-in-path seating. I did not find an existing evidence row that
runs the Nunchuk harness (`.tmp/fable-nunchuk-*`) against a same-seed
descriptor, and did not build/run one myself (out of scope for the time
budget of a mechanical review; would need a fresh same-seed test vector run
through the already-built `fable-nunchuk-policyprobe`/`libnunchuk.a`).

Source-level check, as far as it goes: `libnunchuk`'s `descriptor.cpp` (checkout
at `.tmp/fable-nunchuk-lib`) has no fingerprint/origin duplicate-detection —
only `get_master_fingerprint()` calls for rendering, none for comparison. Since
`composerSeedDerive` assigns a **different account per slot** even for a
shared seed, the leaf **public keys** at each slot stay distinct (only the
origin fingerprint repeats), so nothing in the code I could read would make
Nunchuk's sorted-order behavior differ for this case. That is consistent with
the implementer's stated reasoning, but it is inference from reading code, not
a measurement — the same standard this repo's own evidence practice holds
Liana's claim to. **Minor, not blocking:** the implementer disclosed this
themselves ("carried over unmeasured") rather than asserting it as fact; file
a follow-up to run the Nunchuk harness against a same-seed vector.

## 4. Mutation re-applied

Forced `composerLianaRefusesSeating` to `return false && len(composerSharedSeedInPath(st)) > 0`
in the f671 worktree (grep-confirmed applied, 1 occurrence). Ran
`go test ./gui/ -run 'TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet|TestComposerKeyPathChoiceDoesNotClaimLianaImportsASameSeedSeating|TestComposerNoConsentClaimsLianaImportsANUMSWallet'`:

- `TestComposerConsentDoesNotClaimLianaImportsASameSeedWallet` — **FAIL**, exactly
  the claimed message ("the consent still says \"Liana (as of v15.0) and Bitcoin
  Core import this form\"...").
- `TestComposerKeyPathChoiceDoesNotClaimLianaImportsASameSeedSeating` — **FAIL**,
  exactly the claimed message.
- `TestComposerNoConsentClaimsLianaImportsANUMSWallet` — PASS (correctly
  unaffected; it drives the `refuses` bool as a literal, not through the
  predicate).

Restored with `git checkout -- gui/composer_review.go`; `git status --short`
confirmed clean; re-ran all four F-671 tests plus
`TestComposerKeyPathChoiceSameSeedRowFitsTheFirstPage` — all **PASS**.

## 5. Spec vs. copy, verbatim

Programmatically normalized (whitespace-collapsed) every blockquote in
`design/SPEC_wallet_policy_composer.md` and diffed against the two new Go
string literals (`composerCopyLianaKeyPathSameSeed`,
`composerCopyUnspendableRowLianaSameSeed`) from `gui/composer_copy.go`
verbatim (not the implementer's paraphrase). **Both are exact matches.** No
persisted Go test cross-checks the *whole* `composerCopyTable` against the
spec file mechanically (only comments reference it); the implementer's "checked
mechanically" claim was a one-off check, which I independently reproduced and
confirmed true for the F-671 bodies. Out of scope: verifying all 94 declared
bodies against spec text, which predates this fix.

## Does anything else break

- `go build ./...` (whole fork module): clean.
- `gui-shard-test.sh ./gui/ 24`: **1403/1403**, partition verified exhaustive
  (`1403 == 1403`), wall 22s. Independently re-run, not taken on trust.
- `gofmt -l .`: exactly the five-file baseline
  (`gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) — no new
  violations.
- `go vet ./gui/...`: only the three pre-existing `testing.ArtifactDir`
  go1.25 notes, in files this diff does not touch.
- Both worktrees (`sh-worktrees/f671` at `bc7cbe7`, `me-worktrees/f673` at
  `80735588`) are clean (`git status --short` empty) after this review.
- Did not reproduce the `cmd/emu` wasm/browser walk (`liana-same-seed` arm) —
  its assertions duplicate what the Go suite above already covers, and
  building/serving the wasm harness was judged out of proportion for a
  mechanical review; the implementer's own report already discloses it is not
  wired into `capture_composer.py --arm` (tracked, not blocking).

## Counts and verdict

- **Critical: 0**
- **Important: 0**
- **Minor: 2** — (a) the Nunchuk same-seed sentence is unmeasured (§3, already
  disclosed by the implementer, follow-up: run the Nunchuk harness against a
  same-seed vector); (b) the `liana-same-seed` emulator arm is not wired into
  `capture_composer.py --arm` (already disclosed in the FOLLOWUPS closure
  text).

Every claim in the implementer's report that I checked mechanically
(grep counts, spec-vs-copy verbatim, gofmt/vet output, the 1403 test count, the
mutation's exact failure messages) reproduced exactly as stated. The one rule
is genuinely one rule, it matches Liana v15.0's real per-path
same-origin-fingerprint check (both by source and by the D-1 live measurement),
the mutation reddens and restores cleanly, and nothing else in the package
regressed.

ready to ship: yes
