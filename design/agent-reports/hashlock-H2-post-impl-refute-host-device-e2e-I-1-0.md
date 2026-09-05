# Refute pass (skeptic 1 of 2) -- lens `host-device-e2e`, finding I-1

Target: seedhammer fork worktree `/scratch/code/shibboleth/.tmp/h2-wf-refute-host-device-e2e-I-1-0`,
detached at `17b3979`. Under test: `gui/composer_hashlock.go` (`hashlockOtherPathLine`),
`gui/composer_copy.go` (`composerCopyHashlockOtherPath`). Spec:
`design/SPEC_hashlock_H2_device.md` §4.5; plan addendum:
`design/IMPLEMENTATION_PLAN_hashlock_H2_device.md:3260-3277` (H3 record item).

Claim under test: `"two phrases to back up"` is a hard-coded count that is wrong
whenever more than one *other* path carries a differing hash, and this makes an
uncounted phrase's backup get skipped, so its spend path becomes unrecoverable.

## Verdict: PARTIAL

The code defect is real and reproduces exactly as claimed. The severity claim --
Important, on the theory that an operator "loses" the uncounted phrase and a
spend path becomes permanently unspendable -- does not survive: the same confirm
screen carries an **unconditional** backup instruction for the phrase just typed,
on every single hash-by-phrase path, regardless of this line's wording. Nothing
in the flow gates a phrase's backup on this count. The defect is real but its
consequence is informational imprecision, not lost operator work -- which the
project's own severity rubric for this pass places at Minor ("wording, records"),
not Important ("a real defect, missing case, unsound assumption" in the sense of
an actual broken guarantee).

## 1. The hard-coded-string claim reproduces exactly

`gui/composer_hashlock.go:119-129`:

```go
func hashlockOtherPathLine(st *composerState, idx int, h [32]byte) string {
	for i, p := range st.list.Paths {
		if i == idx || p.Hash == nil {
			continue
		}
		if *p.Hash != h {
			return composerCopyHashlockOtherPath()
		}
	}
	return ""
}
```

`gui/composer_copy.go:451-453`:

```go
func composerCopyHashlockOtherPath() string {
	return "another path has a different hash: two phrases to back up"
}
```

Added a pure-function counterexample (`gui/zzrefute_otherpath_test.go`, not
committed -- scratch worktree) driving 1, 2, 3 and 7 other differing hashes:

```
$ go test ./gui/ -run TestZZRefuteOtherPathLineCounts -v
    zzrefute_otherpath_test.go:16: n=1 other differing paths -> "another path has a different hash: two phrases to back up"
    zzrefute_otherpath_test.go:16: n=2 other differing paths -> "another path has a different hash: two phrases to back up"
    zzrefute_otherpath_test.go:16: n=3 other differing paths -> "another path has a different hash: two phrases to back up"
    zzrefute_otherpath_test.go:16: n=7 other differing paths -> "another path has a different hash: two phrases to back up"
--- PASS: TestZZRefuteOtherPathLineCounts (0.00s)
```

Confirmed too through the real route (`gui/zzrefute_route_test.go`), a 3-path
policy with paths 0 and 1 pre-hashed to two DIFFERENT digests and path 2 taking
the anchor phrase (a third, distinct digest) through `composerHashEdit`:

```
$ go test ./gui/ -run TestZZRefuteThreeDistinctHashesStillSaysTwo -v
    zzrefute_route_test.go:33: confirm body (3 distinct hashes in play) =
    "hash b867db87..edbc96cb method:sha256 chars:28
     another path has a different hash: two phrases to back up
     Write down this phrase and the method now. They are not on this device and
     not on your plates. Without both, this path can never be spent.
     One phrase per policy. Never use this phrase as a passphrase or a password
     anywhere else. Hold button to confirm."
--- PASS
```

(Whitespace added above for readability; the test asserts the un-spaced
concatenated string, matching this repo's `uiContains` convention.) The fork's
only committed test of this line, `gui/composer_hashlock_test.go:912-928`
(`TestHashlockOtherPathLineIsSilentOnAnEqualHash`), and the one route test that
reaches the string, `:857-903`
(`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`), both use exactly one
other path. `n > 2` is genuinely never exercised in the shipped suite -- I-1's
factual claim on this point is correct.

`SPEC_hashlock_H2_device.md` §4.5 itself has no clause for this line (verified:
`grep -n "other path" design/SPEC_hashlock_H2_device.md` finds nothing in that
section). It is an **H3 record item** in the plan
(`IMPLEMENTATION_PLAN_hashlock_H2_device.md:3260-3277`), which states the exact
string to fold into the spec at H3, verbatim identical to what shipped,
including the word "two" -- so the copy is deliberate as currently written, not
an oversight the plan failed to notice; it simply was never generalized past
the two-path case the r0-journey-I-1 addition was built to cover.

## 2. Why the severity claim does not hold

I-1's causal chain is: the operator reads "two phrases to back up" while holding
a third differing phrase in hand → undercounts → a phrase's backup is skipped →
that spend path becomes permanently unspendable.

That chain requires the backup instruction for a given phrase to be *gated by*
or *derived from* this count. It is not. `composerCopyHashlockConfirm`
(`gui/composer_copy.go:407-421`) appends the backup line **unconditionally**,
on every hash-by-phrase confirm screen, independent of `otherPath`:

```go
func composerCopyHashlockConfirm(first8last8, method string, chars int, relation, otherPath string) string {
	b := "hash  " + first8last8 + "\n" + ...
	if relation != "" { b += relation + "\n" }
	if otherPath != "" { b += otherPath + "\n" }
	return b +
		"Write down this phrase and the method now. They are not on this device and " +
		"not on your plates. Without both, this path can never be spent.\n" +
		"One phrase per policy. Never use this phrase as a passphrase or a password " +
		"anywhere else."
}
```

Every path that takes the phrase route gets its own confirm screen, at the exact
moment its own phrase exists in memory, with this same unconditional sentence
("Write down this phrase... this path can never be spent"). That is the only
window in which the phrase exists at all (`hashlockPhraseRoute`'s doc comment,
`composer_hashlock.go:15-19`: "the preimage lives on the stack here and is
dropped when this function returns"), and the instruction to write it down does
not read `otherPath`, does not read a running total, and fires the same way
whether this is the operator's first hashlocked path or their fifth. The
reproduction above shows this directly: the same confirm screen that undercounts
in its second line still carries the correct, unconditional third line telling
the operator to write down *this* phrase.

So there is no mechanism by which reading "two" instead of "three" causes the
operator to skip writing down a phrase -- each phrase got its own unconditional
instruction, independently, when it was typed. What is actually imprecise is a
supplementary framing sentence about the aggregate size of the backup burden
across the whole policy, not the operative instruction. `§8h`'s banner
(`composerCopyHashEveryPathPhrase`) and the reconciliation line
(`composerCopyHashlockReconcile`) -- the two other places that talk about the
policy's overall phrase burden -- both carry no number either; only this one
supplementary line does, and only this one is wrong past n=2.

This is a wording-accuracy defect confined to an informational sentence, not a
defect in what causes a phrase to be backed up. Per this pass's severity
rubric ("Minor/Nit = wording, records"), that places it at Minor, not Important.
I-1's own suggested fix ("count the distinct other digests... or drop the
number") is itself evidence for this reading: dropping the number entirely
would fully resolve the defect with a wording change alone, which is not
possible for an Important-class "missing case" with a real operational
consequence.

## 3. What is NOT disputed

- The string is genuinely hard-coded and genuinely wrong past exactly one other
  differing path -- reproduced above at n=1,2,3,7, both as a pure function and
  through the real confirm-screen route.
- `n > 2` is genuinely unexercised by the shipped test suite.
- The line is genuinely reachable on a wallet with three or more hashlocked
  paths (shown above via the real route, not merely the pure function).

## Machine checks run for this pass

```
$ go test ./gui/ -run 'TestZZRefute' -v          # 2 new tests, both PASS
$ go test ./gui/ -run TestHashlockOtherPathLineIsSilentOnAnEqualHash -v   # PASS, unmodified
```

No file under `gui/` was modified; two scratch test files were added in this
worktree only (`gui/zzrefute_otherpath_test.go`, `gui/zzrefute_route_test.go`)
and are not committed or pushed anywhere.

## Closing counts

- Confirmed as described: 1 (the hard-coded string, reproduced at n up to 7)
- Severity/scope disputed: 1 (I-1's Important / "unrecoverable spend path"
  claim -- actual consequence is wording imprecision in a supplementary line,
  not a lost backup; the operative per-path backup instruction is unconditional
  and unaffected)
- Refuted outright: 0

**Verdict: PARTIAL.**
