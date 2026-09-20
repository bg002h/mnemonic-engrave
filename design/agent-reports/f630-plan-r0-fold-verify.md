# Verification of the F-630 plan fold (sonnet, mechanical)

**Verifier:** sonnet, independent of the fold's author.
**Input report:** `design/agent-reports/f630-plan-r0-opus.md` (1C/7I/4M/2N).
**Plan before:** `git show 5cc0a0a2:design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
(confirmed byte-identical to `03f368d1`'s copy, the persist commit — no drift
between what the reviewer read and what the fold started from).
**Plan after:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at `4b7894c9`.
**Fold diff:** `git diff 03f368d1..4b7894c9` — 162 insertions / 66 deletions,
touching only the plan file.
**Trees:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97dd27bae2`;
primary `/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d69384327265`.
**Method for Q1:** read each finding against the quoted before/after plan text.
For the two findings the brief flagged hardest (C-1's D5 remedy, D6's replacement
check), the remedy was **implemented for real** in a throwaway `git worktree` of
the fork at `95716e97` (a `loadVectorRecord` helper added to
`gui/policy_address_test.go` exactly as D5/T2 describe, the three F-529
`.conformance.json` records + the wsh card pinned to `md/testdata/forkbuilt/`
per T2, then a widened re-vendor of 246 files from `b2c5d693` simulating T3/T4)
and the affected tests actually run, not just reasoned about. The worktree was
removed; `git status --short` is empty in the fork, the primary, and
mnemonic-engrave.

**Result: all 14 findings FIXED. One new Critical found** — the fold's own D5
claim ("record and card are always one policy") is disproved by two further,
real test sites sharing C-1's exact mechanism, neither touched by the fold.

---

## Q1 — per-finding verdict (14 of 14 FIXED)

| # | Verdict | Settling text (after-fold plan) |
|---|---|---|
| C-1 | **FIXED*** | D5: *"the pin gains a record: a `loadVectorRecord(name)` helper mirroring `loadVectorChunks`, preferring `md/testdata/forkbuilt/<name>.conformance.json`, so record and card are always one policy."* T2: *"Add `loadVectorRecord` and point `TestEveryKeyedVectorReachesAnAddress` at it."* **Verified by execution**: implementing exactly this and simulating T2+T3/T4 in a worktree, `go test ./gui/ -run TestEveryKeyedVectorReachesAnAddress -v` now passes **46/46**, including all three F-529 subtests (previously reproduced red by the report). The specific reproduction in the report is closed. \*See **NEW-1** below — D5's *general* claim is false at two other sites. |
| I-1 | FIXED | D6: *"The check becomes: the pin matches the vendored file, **or** the vendored file's `wallet_descriptor_template_id` equals a recorded ... value — `8c1c0566` / `09903620` / `71ff3b74`, measured at `b2c5d693`."* Replaces the skip-on-deletion escape hatch (which the report showed never fires) with a check keyed to the actual drift shape. **Verified**: computed `wallet_descriptor_template_id` directly from `b2c5d693`'s three vectors — `8c1c05666abdf6b...`, `09903620dbcf4e0...`, `71ff3b7424b35d4...` — all three 8-hex prefixes match the plan exactly. |
| I-2 | FIXED | T3: *"update `md/compose_vectors_pin_test.go:103` (36 hardcoded names in `composeVectorNames`) and `:110-111` (the literal `176`) to the widened set."* D4 separately names the `compose_refusal_` exclusion that produces the report's "latent failure" (`compose_refusal_keyless_cap.json` selected-but-not-pinned): *"excluding `compose_refusal_`. That exclusion is not cosmetic: the script as it stands already selects 177 files against a pin of 176 ... Pre-existing rot, detonated by T3."* Both the literal-count gap and the latent exclusion bug are named. |
| I-3 | FIXED | T3 and T4 are merged into one task ("T3 — widen the script and re-vendor (one action, D4)"), with the ordering hazard stated explicitly: *"T3 and the re-vendor are the same action — the script copies and then hashes the destination ... so there is no tree state in which the script is widened, the pin regenerated, and the corpus still stale."* An instruction to capture the pre-state and verify the diff count is present. |
| I-4 | FIXED | D3 rewritten to pin **both** sides: *"pinning the Go path (`m/84h/0h/0h`, `m/86h/0h/0h`) **and the record's expected bracket** (bare `[73c5da0a]`, zero components). It fails ... **or if an allowlisted record's bracket changes at all**."* This closes the report's account-`9h` mutation: a record re-pointed to a different account changes the bracket's component count/content away from the pinned "bare, zero components" expectation, which the added clause now catches. |
| I-5 | FIXED | T1: *"Acceptance: 44 of 46 fail, 2 pass ... Not 41: the F-529 three carry stale descriptors too, and an implementer who tunes the gate until exactly 41 fail would exempt precisely the three vectors this cycle is most exposed on."* Matches the report's measured `STALE corpus: 44 vectors FAIL, 2 PASS` exactly. |
| I-6 | FIXED | T1: *"Use the pin-preferring loader (`vectorChunksFor`, `md/duplicate_keys_test.go:16`) for the card and, once D5 lands, the pin-preferring record too — the plain `loadPhraseChunks` would silently pair a pinned card with a re-vendored record, which is C-1 in a second place."* The loader ambiguity is resolved with an explicit choice, and the plan even names the exact failure mode it is avoiding. |
| I-7 | FIXED | New decision D2c: *"bind the origin bracket's FINGERPRINT (R0 I-7) ... The gate asserts the bracket fingerprint equals `ExpandedKey.Fingerprint` (`md/expand.go:56-64`); measured, they agree **284/284** on the re-vendored corpus today."* T1's task list now reads *"implementing D1, D2a, D2b, D2c and D3."* |
| M-1 | FIXED | The new "Sites depending on `keyed_wsh_timelock_hashlock`" table plus closing paragraph: *"D5's rationale in the earlier draft was that these would go green against a shape no longer carrying the defect. Measured, they fail loudly ... The distinction matters: the false version frames T2 as sufficient, and T2 alone is exactly what produces C-1."* The false rationale is retracted and replaced with the correct one (evidence would be *deleted*, not silently *passed*). |
| M-2 | FIXED | Tasks preamble: *"No task pushes a red tree; T1 deliberately commits one. T1's whole purpose is the RED demonstration ... (The earlier draft asserted 'each task ends green before the next begins', which T1 contradicts by construction.)"* |
| M-3 | FIXED | The new dependent-sites table adds the fourth site: `md/compose_shape_test.go:111` — *"stays green — same branch shape either way."* **Verified**: line 111 of that file is exactly the `keyed_wsh_timelock_hashlock` table row. |
| M-4 | FIXED | "The measured drift" now reads: *"82 chain-descriptor entries move across the 41, and 88 across all 44 drifted records (the F-529 three carry stale descriptors too)."* Matches the already-machine-checked count exactly (82/41, 88/44). |
| N-1 | FIXED | D3 no longer contains the contradiction. It now reads: *"So: the gate **does** assert bracket-path == Go origin path for every vector, and carries a two-sided allowlist for those two"* — replacing the old "does **not** ... globally" / "fails if a non-allowlisted one starts" self-contradiction with one consistent statement. |
| N-2 | FIXED | "What this plan does NOT cover" now records the four out-of-scope descriptor-string parts verbatim (BIP-380 checksum, chain-index swap, derivation-suffix swap, `multi()` key-position swap), matching N-2's content and — per N-2's own instruction — recorded rather than scheduled as a task. |

**Tally: 14 FIXED / 0 PARTIAL / 0 NOT-FIXED / 0 DECLINED.**

---

## Q2 — did the fold introduce a new defect

### NEW-1 (Critical) — D5's "record and card are always one policy" is false: two more test sites pair a pinned card with a purely-vendored record, and both fail identically after T2+T3/T4

D5's fix, as verified above, closes the report's **named** reproduction
(`gui/policy_address_test.go:TestEveryKeyedVectorReachesAnAddress`). But the
fold's own justifying sentence is a **global** claim — "record and card are
always one policy" — and the brief's own instruction to check "anything else
globbing `keyed_*.conformance.json`" finds two more sites with the identical
shape: the **card** comes from the pin-preferring `loadVectorChunks`, the
**record** comes from a plain glob/read of `md/testdata/vectors/`, with no
pin-preferring mechanism at all. Neither site is touched by D5 or T2, and
neither is named anywhere in the plan.

**State, reproduced for real** (not simulated on paper): in the same throwaway
worktree used to verify C-1's fix — T2 applied (the three F-529
`.conformance.json` records and the wsh card pinned to
`md/testdata/forkbuilt/` from the **pristine pre-revendor** content, confirmed
distinct from the post-revendor vendored copy by `diff`) — plus T3/T4 applied
(246 files re-vendored from `b2c5d693` under the widened `^(keyed_|compose_)`
pattern, `compose_refusal_` excluded):

```
$ go test ./gui/ -run TestTaprootScriptPathMatchesRust -v
    taproot_script_path_test.go:114: keyed_tr_multi_a chain 0 index 0:
          go:   bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt
          rust: bc1pgrupj0fjv79xtj05mzthds4dqzvdhcptx2gzpgzt90uzc86qzfes2a0yhh
    [... 5 more mismatched indices across both chains ...]
--- FAIL: TestTaprootScriptPathMatchesRust (0.01s)
    --- FAIL: TestTaprootScriptPathMatchesRust/keyed_tr_multi_a (0.00s)
    --- FAIL: TestTaprootScriptPathMatchesRust/keyed_tr_sortedmulti_a (0.00s)

$ go test ./gui/ -run TestWshWitnessScriptHashesToRustsAddress -v
    wsh_script_emit_test.go:41: keyed_wsh_timelock_hashlock chain 0 index 0:
          go:   bc1q6h9y4ngdfacaplw0qk67rugxs3vanv0jayk3n3xnhed7uke76zksfqt7py
          rust: bc1qa9wapjm45uthw7r806zuz9mev2a5cwqmrlwnyuj4pjs0c78d75rqp6j29e
    [... 5 more mismatched indices ...]
--- FAIL: TestWshWitnessScriptHashesToRustsAddress (0.02s)
```

Mechanism, per site:

- `gui/taproot_script_path_test.go:30` (`TestTaprootScriptPathMatchesRust`)
  globs the record from `md/testdata/vectors/keyed_tr_*.conformance.json`
  directly, and gets the card from `loadVectorChunks(t, name)` at line 55 —
  the **same** pin-preferring loader `TestEveryKeyedVectorReachesAnAddress`
  uses. `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are already pinned in
  `md/testdata/forkbuilt/` today (pre-fold, pre-implementation — confirmed by
  `ls`), so this hazard is **live today**, not hypothetical: it only needs
  the record side to move, which T3/T4 does.
- `gui/wsh_script_emit_test.go:31` (`TestWshWitnessScriptHashesToRustsAddress`
  via `checkWshVector`) globs the record from
  `md/testdata/vectors/keyed_wsh_*.conformance.json` at line 31, and gets the
  card from `loadVectorChunks(t, name)` at line 72 — same story, for
  `keyed_wsh_timelock_hashlock`.

Both files are inside T4's stated gate ("the full `./md/` and `./gui/`
suites"), so T4 would go red on these two files for the exact reason C-1 was
rated Critical: an unmet guarantee (D5's own new sentence), a wrong result
(addresses compared across two different policies), and **no task in the plan
that fixes it** — T2 wires `loadVectorRecord` into exactly one call site
(`gui/policy_address_test.go`), and D5's closing sentence claims a scope it
does not have.

I checked every other site in the fork tree that reads a `.conformance.json`
by name or glob and pairs it with card material for one of the F-529 three
(`gui/composer_policy_address_test.go`, `gui/key_card_seating_test.go`,
`gui/f533_taproot_reuse_address_test.go`, `md/policy_shape_test.go`,
`md/conformance_keyed_test.go`'s existing `TestKeyedConformanceAgreesWithRust`,
`md/compose_pkh_emit_test.go`, `md/compose_stubs_test.go`): none of them
intersects the F-529 three with a split card/record source — either they
don't reference these three vectors at all, or (like the existing
`TestKeyedConformanceAgreesWithRust`) both halves already come from the same
vendored source, so they move together and stay consistent. The two named
above are the only additional live sites.

**Why Critical, not Important:** this is the same finding shape the report
rated Critical for the same reasons — an explicit guarantee the plan's own new
text states and then fails to hold, producing a wrong cross-language
comparison inside the plan's own gate, with no task addressing it. The
severity rubric's "unmet guarantee" / "wrong result" clause applies
identically here.

**Remedy shape** (not prescribed, per the brief's out-of-scope note — proposing
plan changes is not this review's job, but the fix is structurally the same one
T2 already applies once, just not everywhere it's needed): either extend
`loadVectorRecord` (or an equivalent) to every site that pairs a
pin-preferring card loader with an F-529 vector, or centralize card+record
sourcing behind one pair of loaders package-wide so a future third site
cannot reintroduce the same split.

### No other new defects found

I looked specifically at whether D6's replacement check (`8c1c0566` /
`09903620` / `71ff3b74`) could be spoofed by a genuine third policy landing
under these names — it cannot: a third policy would carry neither the pinned
byte-identical file nor the recorded id, so both branches of the `or` fail and
the check reds, as D6 intends. I also checked whether T3's "capture the
pre-state first (`git stash` or a copy)" instruction is actually sufficient:
`md/testdata/vectors/` is git-tracked (328 files, none gitignored), so the
pre-state is already captured by git regardless of whether an implementer
runs `git stash` — `git diff --stat` after the script runs gives the exact
diff count against the last commit with or without a stash. The instruction
works; it just isn't the only way to get there. This is wording, not a defect,
and out of scope.

**New-defect tally: 1 Critical / 0 Important / 0 Minor / 0 Nit.**

---

## What was NOT re-derived (per brief)

Citation resolution, the 82/88/44/46 counts, the 284/284 fingerprint
agreement, and the "fork already emits the correct header" facts were taken as
settled per the brief's machine-checked list and not re-measured, except where
a specific finding's verdict required fresh execution (I-1's three id
constants; C-1/NEW-1's actual test runs).
