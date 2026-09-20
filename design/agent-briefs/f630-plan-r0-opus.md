# Brief — R0 adversarial review of the F-630 implementation plan

**Artifact under review:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
at mnemonic-engrave `5cc0a0a2`. Repos: fork `/scratch/code/shibboleth/seedhammer`
main `95716e97`; primary `/scratch/code/shibboleth/descriptor-mnemonic` main
`b2c5d693`.

## The ONE question

**Will the gate this plan designs actually fail on the drift it claims to
catch, and will T2/T4 preserve the duplicate-key evidence they touch?**

Everything else is out of scope. Concretely, answer these and nothing else:

1. **Does D1+D2 have teeth?** Construct a concrete corpus corruption that the
   assertion as specified would let through. The defect it exists to catch is
   "a uniformly wrong corpus agrees with itself" — a record whose `keys[]` and
   `descriptor` are consistently wrong together. Does binding both to the Go
   port's own expansion (D2) actually close that, or is there a mutation of
   the vendored JSON that passes all four D1 clauses plus both D2 clauses?
2. **Is D3's allowlist a pin or a hole?** It exempts `keyed_wpkh` and
   `keyed_tr_keyonly` from the bracket-path comparison. Can a real divergence
   hide behind that exemption? Does the "fails if an allowlisted vector stops
   diverging" clause actually detect the decay it is written for?
3. **Does T2 preserve what it claims?** `keyed_wsh_timelock_hashlock` is the
   only fork witness for `md.DuplicateRefusedByCore`. After T2's pin and T4's
   re-vendor, is every one of the three dependent test sites still exercising
   a policy that carries the duplicate? Name any site that would go green
   against a shape no longer carrying the defect — that is the highest-value
   finding available here.
4. **Is the T4 expectation falsifiable?** T4 says "expect exactly 41 records
   to change descriptor lines, 3 wholesale, 0 address and 0 id lines outside
   those 3". Is that check specified tightly enough to fail if the primary
   changed something else between the pinned commit and `b2c5d693`?
5. **Ordering.** Is T1→T2→T3→T4 the right order, or does any task leave the
   tree in a state where a later task's gate cannot fail?

## ALREADY MACHINE-VERIFIED — do not re-derive, do not re-measure

These were measured by the controller against the two trees above on
2026-09-20. Spend no budget reconfirming them; if you believe one is wrong,
say so in one line with the command that shows it, and move on.

- 46 keyed conformance records: 2 identical, 41 descriptor-only, 3 semantic.
  88 chain-descriptor entries move. The 3 semantic are exactly F-529's
  (`keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`, `keyed_wsh_timelock_hashlock`)
  and none is a `keyed_compose_*`.
- `scripts/vendor-compose-vectors.sh:16` selects `^(keyed_)?compose_`, 177
  files, reaching 32 of the 41.
- The fork's render path already emits the 0.44.0-correct header: a probe on
  `keyed_compose_preset_plain_multisig` produced bytes identical to dm
  `b2c5d693`'s `chains["0"].descriptor`, key for key.
- `go test ./md/ -run TestComposeKeylessCap -v` → 7 refused, 1 admitted,
  kinds `map[KeylessUnderTr:1 LegacyWrapperShape:1 NoKeyedPath:1
  TooManyKeylessPaths:3 TooManySlots:1]`.
- Every file:line citation in the plan was re-grepped and is correct:
  `bip380/bip380.go:97-109`, `gui/md1_expand.go:143`, `md/expand.go:76-81`,
  `md/duplicate_keys_test.go:84`, `gui/composer_flow_test.go:599,684`,
  `gui/policy_address_test.go:149`.
- `keys[]` xpubs carry the real parent fingerprint, descriptor xpubs carry
  zero — comparison is over the 65 bytes, never the base58 string.

## Explicitly OUT of scope

- Re-auditing the fork's codec, the composer, or anything F-630 does not touch.
- Re-litigating that step 3 is unnecessary and step 4 is done; that is measured.
- Style, naming, wording, doc polish, and opportunistic cleanups.
- Proposing that the plan do MORE. If a task is missing, say which failure it
  lets through; do not widen scope for completeness' sake.
- F-529's separate ruling (reuse-free vs reuse-bearing device fixtures).

## Rules of evidence

Reproduce a defect, do not prescribe a remedy. A finding must name a concrete
failure: the input or state, and the wrong outcome. "Consider also asserting
X" without a failure it admits is a Nit at best. You have both repos; run
commands rather than reasoning from the plan's prose, and quote what you ran.

## Output

Severity per the project standard: **Critical** (wrong result / data loss /
funds-safety / an unmet guarantee / a gate that cannot fail), **Important**
(real defect, missing case, unsound assumption), Minor, Nit. Secret-handling
defects are never Critical or Important here — log them as follow-ups.

**As your FINAL action, write the full report to
`design/agent-reports/f630-plan-r0-opus.md`** (in
`/scratch/code/shibboleth/mnemonic-engrave`) and return ONLY a one-paragraph
summary plus that path and the C/I/M/N counts. Do not return the report
inline.
