# F-531 fold verification

Scope: `git diff cea4a63b..9db77cf5` in mnemonic-engrave (records-only, I-3) and
`git diff a4760e1..9b36ed7` in seedhammer (`fold: the F-531 review -- I-1, I-2,
I-3 and every Minor`). Two questions only: does the fold close what it claims,
and did it introduce a new defect. The original F-531/`repeatsASeat` analysis
is taken as settled and not re-derived.

All commands run with `/scratch/code/shibboleth/.toolchain/go/bin` on PATH.
Both trees left clean (`git status --short` empty in both) after every
mutation was reverted.

---

## I-1 — the F-514 surface gate could not fail — CLOSED

The three unreachable blocks are gone:

- `gui/wallet_policy.go:328-330` (was in `walletPolicyAddressLines`) — deleted,
  replaced by a comment explaining why it was dead (`ok` there implies
  `DuplicateNone`) and naming the live producer (`noAddressLines`).
- `gui/composer_consent.go:219-221` — deleted, same reasoning, same pointer.
- `gui/md1_gather.go`'s `policyIDHeader` block — deleted, comment explains the
  header is only built after the modal above has already stopped the card.

The mutation note in `gui/composer_flow_test.go`
(`TestEveryAddressSurfaceCarriesTheDuplicateWarning`) was rewritten to name the
two live producers, `noAddressLines` and `duplicateRefusalBody`.

**Verified by mutation (run, not read):** deleted the duplicate-key block
inside `noAddressLines` (the `if slot, kind, err := ...; dup = true` block) and
replaced `duplicateRefusalBody` with `return "", false`. Result:

```
--- FAIL: TestConsentWarnsOnDuplicateKeys
--- FAIL: TestEveryAddressSurfaceCarriesTheDuplicateWarning
--- FAIL: TestRepeatedSeatRefusalStillWarns
--- FAIL: TestKeylessRepeatedSeatTemplateIsNotSilent
```

All four go red exactly as the corrected comment claims. Reverted from
`/tmp/wallet_policy.go.bak` and `/tmp/md1_gather.go.bak`; `go build ./...` and
`git status --short` confirmed clean afterward.

## I-2 — silent keyless repeated-seat template — CLOSED

`noAddressLines` (`gui/wallet_policy.go:286-329`) now builds the duplicate
sentence as a **prefix** (`lines = append(lines, "", composerCopyDuplicateKeys(...))`,
`dup := true`) ahead of a `switch` over the three "no address" reasons
(keyless / has-no-keys / generic), so the warning applies regardless of which
arm fires. `gatheredDescriptorFlow` (`gui/md1_gather.go:174-190`) now calls the
new `duplicateRefusalBody` **before** the `expandedToDescriptor` switch, so
`expandTemplateOnly` (previously routed straight to `md1DisplayFlow` with no
modal at all) is covered too.

Walked all four `noAddressLines` arms against both keyed and keyless duplicate
inputs (code reading, cross-checked against the passing test suite): `dup`
prefix is appended before the `switch`, so it precedes whichever of
keyless / no-xpub / generic-refusal / (mutually exclusive) plain-duplicate arm
returns — no combination drops the "Verify off-device." tail, since every
`return` statement still ends with it.

**Verified by test (author's fixture, run):** the exact card from the original
finding (`md1fgefcqqpq2tvyyy4qqxppcq3p0px74klwllmkl4ns5uz3v66nj2ecs`, now
committed as `md/testdata/forkbuilt/dup_seat_wsh_sortedmulti_k1_keyless.md1.txt`)
drives `TestKeylessRepeatedSeatTemplateIsNotSilent` — PASS, including its
`inspect_descriptor` subtest (a real `gatheredDescriptorFlow` render via
`synctest`).

**`duplicateRefusalBody` vs `noAddressLines`' keyed test.** Both use the same
predicate for "keys are fully present": `duplicateRefusalBody` tests
`len(keys) > 0 && allSlotsHaveXpub(keys)` directly; `noAddressLines`' `case
!allSlotsHaveXpub(keys)` combined with its preceding `case len(keys) == 0`
covers the complementary condition (since `allSlotsHaveXpub` already returns
`false` for an empty slice, the two conditions partition the same way). No
disagreement found between what the modal (Inspect) and the consent screen
say about the same card — confirmed by running the full
`TestRepeatedSeatRefusalStillWarns` (keyed) and
`TestKeylessRepeatedSeatTemplateIsNotSilent` (keyless) together, both green.

**Reachability check — no regression to `expandOK`/`descriptorFlow`.** The
hoisted `duplicateRefusalBody` call in `gatheredDescriptorFlow` runs before
`status` is computed, for every status. Given the review's own (unrevisited)
proof that `expandOK ⟹ DuplicateNone`, the hoisted check can never fire on a
card that would otherwise reach `descriptorFlow`, so no previously-reachable
screen was made unreachable. For `expandUnsupported` cards where
`complexAddressSource` used to gate the same duplicate check inline, the
outcome (same message, same `md1DisplayFlow` call) is unchanged; only
`expandTemplateOnly` gained coverage it did not have.

## I-3 — records still described the old sentence — CLOSED

`git diff cea4a63b..9db77cf5` (mnemonic-engrave) touches exactly the three
cited sites:

- `design/SPEC_wallet_policy_composer.md:552-560` (was :555) — replaced the
  single "Keyless template - no addresses" (D4) sentence with all four
  `noAddressLines` arms, correctly naming which one the composer's own
  keyless-with-declared-slots template hits, and citing "F-531, review I-2"
  for why the duplicate warning leads.
- `design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md:312` — expected text
  updated to `"Template has no keys - no addresses."`, with an inline note
  explaining why the sentence changed (composer builds a slot-declaring
  template, hitting `noAddressLines`' second arm).
- `design/SPEC_wallet_policy_composer.md:1014` (now ~1019) — the dated
  "EXECUTED 2026-09-03" capture kept its original quoted string (correct, it
  is a record) and gained a parenthetical noting F-531 later changed the
  branch and current behavior differs.

`grep -rn "Keyless template - no addresses" design/` still finds it in the
historical S3 plan and several `design/agent-reports/*.md` review transcripts
— correct, those are dated records of past states, not the three sites I-3
named, and I-3 never claimed those needed changing.

Cross-checked the new SPEC prose against the actual `noAddressLines` switch
order (keyless → no-xpub → dup-only → generic, dup as prefix): matches.

---

## Minors and the Nit

- **M-1 CLOSED.** `md/duplicate_keys.go`'s header (lines 21-40) no longer says
  "It is a WARNING's predicate, never a refusal's" or that the device "says
  nothing" — rewritten to state it is now also a refusal's predicate, why the
  card is not stranded, and points at F-533 for the narrower-than-BIP-388 gap.
- **M-2 CLOSED.** All three sites the review named now carry the corrected
  claim (`max(1, k-m+1)`, with an explicit retraction note citing review I-5):
  `md/duplicate_keys.go:158-162`, `md/duplicate_keys_test.go:176-180`,
  `gui/composer_flow_test.go:785-788`.
- **M-3 CLOSED**, verified by mutation (run): deleted the `repeatsASeat`
  block in `gui/multisig_restore.go` (fell through to the generic "Addresses
  unavailable for this policy shape." sentence). Result:
  `--- FAIL: TestTheRestoreDocNamesTheReuse` (both subtests). Reverted from
  `/tmp/multisig_restore.go.bak`; rebuilt clean.
- **M-4 CLOSED.** `gui/modal_fits_test.go` gained three rows in
  `TestModalsThisBlockTouchesAreDrawnInFull` — keyed `DuplicateFewerKeys`,
  keyed `DuplicateRefusedByCore`, and keyless (warning alone) — built from the
  same `composerCopyDuplicateKeys(...) [+ " " + composerCopyNoAddressesDuplicateKeys()]`
  construction `duplicateRefusalBody` actually uses. Ran the targeted test:
  159, 145 and 95 characters drawn in full with 397/418/476 headroom against
  an 80-character margin — matches the review's own measurement exactly. The
  fit-gate mechanism (`assertModalBodyFits`/`bodyDrawnFully`) is the same one
  ~20 other rows in the same test already rely on; not re-mutated separately.
- **M-5 PARTIAL.** The code comment is fixed:
  `gui/policy_address.go:45-54` now says "REPEATS A KEY SLOT INSIDE ONE SCRIPT
  EXPRESSION" (not "on either route" as the wide, false claim), explains the
  gap is narrower than BIP 388, cites the two corpus vectors, and names F-533.
  **But `design/SPEC_wallet_policy_composer.md:857-860` (§8s) — the review's
  other cited site for the same overclaim — is untouched**: it still reads
  "The device declines to derive an address for a policy that seats one key
  slot more than once -- BIP 388 forbids the shape ..." with no qualifier and
  no pointer to F-533. Confirmed via `git log --oneline -- design/SPEC_wallet_policy_composer.md`
  (the fold commit `9db77cf5` is present; `git diff cea4a63b..9db77cf5` does
  not touch line 857) and by reading the current file. This is Minor per the
  original review (the gap itself is already filed as F-533) and does not
  gate, but it is not fully closed as the fold's own framing ("every Minor")
  implies.
- **N-1 CLOSED.** `assertShowsNoAddress` (`gui/composer_flow_test.go:760`) now
  checks `"bc1q"`/`"bc1p"` instead of the bare `"bc1"` substring, with a
  comment explaining why (hex wallet ids could contain "bc1"). Confirmed the
  updated helper is exercised by the (passing) `TestKeylessRepeatedSeatTemplateIsNotSilent`
  and `TestEveryAddressSurfaceCarriesTheDuplicateWarning`.
- **N-2** — per the dispatch brief, accepted without change; confirmed
  `gui/md1_gather.go`'s modal still names the duplicate as the cause
  unconditionally. Not reported as a defect.

---

## New defects introduced by the fold

**None found.** Specifically checked and clear:

- No previously-reachable screen became unreachable (see I-2 analysis above).
- No previously-derivable address became silently blocked: `expandOK ⟹
  DuplicateNone` is untouched by this fold (no lines in `repeatsASeat`,
  `multiPolicy`, `classifyPolicy`, or `validatePlaceholderUsage` changed —
  confirmed by `git diff a4760e1..9b36ed7 -- md/md.go md/expand.go` returning
  nothing).
- `dupSeatDescriptor`'s new `keyless bool` parameter has exactly one caller
  (`md/dup_seat_fixture_test.go:116`), correctly updated.
- `gofmt -l` on all ten changed/added Go files in the seedhammer diff: clean
  (no new entries beyond the known five-file baseline, which none of these
  files are in).
- `go build ./...` and `go test ./md/...` both clean at `9b36ed7`.
- Every new file:line / function-name citation added by the fold
  (`noAddressLines`, `duplicateRefusalBody`, `allSlotsHaveXpub`,
  `gui/wallet_policy.go`, `gui/md1_gather.go`) resolves to real code at the
  stated shape; re-grepped rather than trusted.

---

## Counts and verdict

Per finding: I-1 CLOSED, I-2 CLOSED, I-3 CLOSED, M-1 CLOSED, M-2 CLOSED,
M-3 CLOSED, M-4 CLOSED, M-5 PARTIAL, N-1 CLOSED, N-2 correctly left as-is.

New defects from the fold itself: 0 Critical / 0 Important / 0 Minor / 0 Nit.

Outstanding from the fold's own incompleteness: **0 Critical / 0 Important /
1 Minor (M-5's SPEC-side overclaim, §8s, left unfixed) / 0 Nit.**

`0 Critical / 0 Important / 1 Minor / 0 Nit` — **GREEN** (0C/0I; the one open
item is Minor and does not gate, but is recorded so it is not lost: someone
should still add the F-533 pointer / qualifier to `design/SPEC_wallet_policy_composer.md:857-860`).
