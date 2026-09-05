# Refute pass (skeptic 2/2) — host-device-e2e I-1: "two phrases to back up" is a hard-coded count

Branch under test: seedhammer fork `hashlock-h2` @ 17b3979, worktree
`/scratch/code/shibboleth/.tmp/h2-wf-refute-host-device-e2e-I-1-1` (detached,
read-only intent; a single scratch test file was added locally to reproduce the
claim and is not committed).

Claim under test (host-device-e2e, Important): `hashlockOtherPathLine`
(`gui/composer_hashlock.go:119-129`) returns the literal string
`composerCopyHashlockOtherPath()` — "another path has a different hash: two
phrases to back up" — as soon as it finds ONE other path with a differing
hash, so the count "two" is wrong whenever a policy has 3+ distinct hashlock
digests; the operator's own "reasonably complex wallet" (RCW) fixture is such
a policy.

**Verdict: CONFIRMED at the severity claimed (Important). Not refuted; not
downgraded.**

## Reproduction 1 — the pure function, count-invariant

```
$ go test ./gui/ -run TestZZRefuteI11OtherPathLineIsCountInvariant -v
=== RUN   TestZZRefuteI11OtherPathLineIsCountInvariant
n=2 other-differing-paths=1 -> "another path has a different hash: two phrases to back up"
n=3 other-differing-paths=2 -> "another path has a different hash: two phrases to back up"
n=8 other-differing-paths=7 -> "another path has a different hash: two phrases to back up"
--- PASS: TestZZRefuteI11OtherPathLineIsCountInvariant (0.00s)
PASS
ok  	seedhammer.com/gui	0.003s
```

Scratch test (`gui/zz_refute_i11_test.go`, not committed) calls
`hashlockOtherPathLine(st, 0, h)` directly with 1, 2 and 7 OTHER paths each
carrying a distinct non-matching hash. The returned string is byte-identical
across all three counts — the code has no counting logic at all; the loop at
`gui/composer_hashlock.go:119-129` returns on the *first* `*p.Hash != h` it
finds and never counts how many such paths exist:

```go
func hashlockOtherPathLine(st *composerState, idx int, h [32]byte) string {
	for i, p := range st.list.Paths {
		if i == idx || p.Hash == nil {
			continue
		}
		if *p.Hash != h {
			return composerCopyHashlockOtherPath()   // fixed string, no count
		}
	}
	return ""
}
```
and `gui/composer_copy.go:451-453`:
```go
func composerCopyHashlockOtherPath() string {
	return "another path has a different hash: two phrases to back up"
}
```

## Reproduction 2 — the operator's own reference fixture is the n=3 case

`design/fixtures/reasonably-complex-wallet/tr.policy` (and `wsh.policy`, same
digests) carries three DISTINCT `sha256(...)` hashlocks, one per tier 1, 2, 4:

```
sha256(a7ef0ba42dada5629bbb95e386c572006d4bea43d483e5c44f4c3858725367f1)   tier 1
sha256(e9955f6f5b49ff288c3f8360e6a7dde1d54aa590eb6a20f28b23db361d4f09b4)   tier 2
sha256(7950085dca9f90b67bbcfeb8141499a98df93e32709807420c86f2ff071d6af7)  tier 4
```
— exactly the three digests the lens report quoted. So the RCW is not a
hypothetical n>2 case; it is the constellation's own standing reference
wallet. At the moment the operator types the phrase for tier 4 (the third
hashlock), paths for tiers 1 and 2 already carry hashes that both differ from
tier 4's — `hashlockOtherPathLine` returns on the first of those two it scans
and the confirm modal reads "...two phrases to back up," though three
distinct phrases (one per tier) are now in play for this policy.

## Is the line specified, and is there a corrective count elsewhere?

- `design/SPEC_hashlock_H2_device.md` §4.5 enumerates the confirm-modal body
  verbatim and contains no "other path" / "two phrases" text at all — grepped,
  zero hits in the file outside an unrelated citation on line 87. The
  implementation plan itself records this as **deferred, not landed**:
  `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md:3137` — "See the H3 record
  item below — the spec's §4.5 step 2 is not edited here" — and the plan's own
  design intent for this line (`:3173`, item 10) is literally titled **"Two
  paths, two phrases, no cross-check"**: a binary present/absent check for a
  second phrase, not a counted design. So the copy is free to change, exactly
  as the lens report states, and this is a pre-existing scope gap in the
  design, not a regression against a stated guarantee.
- No other screen in the phrase route supplies a corrected count. §8h's
  "HASH ON EVERY PATH" banner (`composerCopyHashEveryPathFor`,
  `gui/composer_shape.go:442-443`) is gated by `composerEveryPathHashed`,
  which requires **every** spend path to carry a hash — false on the RCW,
  since tier 3 (`@5` alone) has none. That banner's own text is count-free
  ("the phrase and its method", singular) regardless. The reconciliation line
  (`composerCopyHashlockReconcile`) that does fire is only about matching one
  digest against the host's `ms hashlock`, not about how many phrases exist.
  So on the RCW's own route, "two phrases to back up" is the *only* place a
  count is ever stated, and it is wrong.

## Test-coverage claim, checked directly

Both existing tests that exercise `hashlockOtherPathLine` build their state
with `composerStateWithPaths(t, 2)` — confirmed by reading
`gui/composer_hashlock_test.go:844` (`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`)
and `:914` (`TestHashlockOtherPathLineIsSilentOnAnEqualHash`). Neither
constructs a third path with a third distinct hash; n>2 has zero committed
coverage, exactly as claimed.

## Severity check against this pass's rubric

Not a digest divergence, not a hash assigned before HOLD, not a false-PASS on
a stated normative guarantee (none exists for this line's wording) — so not
Critical under this pass's bar. It is a real, reproduced defect in
operator-facing advisory text with a plausible harm path (an operator who
trusts the literal count and backs up only two phrases loses the ability to
satisfy the third tier's spend condition later) and a missing test case for
the policy shape the constellation itself uses as its reference wallet. That
is squarely **Important**, matching the lens report's own rating — no
downgrade to Minor/Nit is warranted (this is not a secret-handling defect;
the count is public advisory text).

## Closing counts

- 0 Critical
- 1 Important — CONFIRMED as reported (host-device-e2e I-1): `hashlockOtherPathLine` /
  `composerCopyHashlockOtherPath()` prints a fixed "two phrases to back up" regardless
  of how many other paths' hashes differ, reproduced at n=1/2/7 other differing paths
  and on the RCW's own 3-hashlock fixture; uncorrected elsewhere in the route; SPEC
  §4.5 does not constrain this line's wording (plan records it as an H3-deferred item);
  existing tests cover only n=2.
- 0 Minor / 0 Nit raised by this pass.

Verdict returned to controller: **CONFIRMED**.
