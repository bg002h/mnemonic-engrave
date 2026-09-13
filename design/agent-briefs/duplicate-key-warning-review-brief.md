You are the INDEPENDENT adversarial execution reviewer (opus tier) for F-514 in
the SeedHammer fork: the device now WARNS when a policy's miniscript repeats a
key, the condition Bitcoin Core refuses as "contains duplicate public keys".

Repo `/scratch/code/shibboleth/seedhammer`, branch
`composer-duplicate-key-warning`, tip `ad64629`. Your range is
`git diff main..ad64629` — one commit, seven files. The finding is F-514 in
`/scratch/code/shibboleth/mnemonic-engrave/design/FOLLOWUPS.md`.

ONE QUESTION: can you construct a policy the device shows an address for where
this warning is WRONG in either direction — silent on a descriptor Core refuses,
or firing on one Core accepts?

Read-only on the fork; commit nothing; no sub-agents; never read any `.jsonl`.
Work in your OWN scratch copy:
`rm -rf /scratch/code/shibboleth/.tmp/dupkey-review && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/dupkey-review ad64629`.
Go is `/scratch/code/shibboleth/.toolchain/go/bin/go`, first on PATH;
`TMPDIR=/scratch/code/shibboleth/.tmp`. Revert every mutation; remove the
worktree when done.

Bitcoin Core is NOT installed and you do not need it: the three corpus verdicts
below were measured on a throwaway regtest datadir and are given as settled.
If you want Core for a shape outside that set, say so in the report rather than
downloading it.

## Already settled — do not re-derive
- Core 31.1 `getdescriptorinfo`, keys version-swapped to tpub, stale BIP-380
  checksum stripped: `keyed_tr_multi_a` ACCEPTED, `keyed_tr_sortedmulti_a`
  ACCEPTED, `keyed_wsh_timelock_hashlock` refused with
  "... is not sane: contains duplicate public keys".
- gui 1302/1302 across 24 shards partition-verified; all 75 non-gui packages
  pass; go vet clean beyond the TinyGo gap; tinygo 1,649,564 flash / 63,304 ram.
- Three mutations already run and RED: whole taptree counted at once; multi keys
  not counted; the consent warning removed.
- A refusal was considered and rejected: refusing on-device strands a card that
  may already be engraved. Do not re-argue that; it is the operator's standing
  position that a wrong outcome must be worse than saying nothing.
- Secret-handling never gates.

## Construct counterexamples — an assessment without one is Minor at most
1. **The scoping rule.** `DuplicateKeySlot` treats each taptree LEAF as its own
   expression and the tr internal key as outside all of them. Find a shape where
   that is wrong. Specifically: a key in the internal key AND in two different
   leaves; nested taptrees deeper than the corpus has; `sh(wsh(...))` and bare
   `sh(...)`, which take different arms; a `thresh` (variableBody) whose children
   repeat a slot; a wrapper chain that hides a key from `countKeySlots`.
   **`countKeySlots` does not walk `trBody`** — prove that is safe, or find the
   shape that makes it unsafe.
2. **Reachability.** The warning is added in `composerConsentLinesFor` and
   `walletPolicyAddressLines`. Is there a THIRD surface that shows an address
   for a decoded policy and does not carry it? Trace every caller of
   `policyAddressAt`.
3. **The silent-error path.** Both call sites use `err == nil && dup`, so an
   unreadable chunk set warns about nothing. Is that the right side to fail on
   here, given both callers have already decoded those same chunks successfully
   by the time they run?
4. **The copy.** It names Core, the slot, and "duplicate public keys". Is any
   part of it false, and does it fit the screen the way the file's own
   `assertModalBodyFits`-style gates expect? Is naming `@N` useful to an
   operator, or is the slot number meaningless outside the codec?
5. **The tests.** For each test the diff adds, mutate the guarantee it names and
   quote the failure. Note especially that the tap-leaf scoping test builds a
   `node` tree by hand because no vendored vector separates the two scopings —
   check that claim yourself with your own probe, and say if you find a vector
   that does.

## Severity
Critical: silence on a shape Core refuses that the device shows an address for;
a test that cannot fail on what it names. Important: a false warning on a shape
Core accepts, an unreachable surface, an unsound assumption. Minor/Nit: copy,
wording, records.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/duplicate-key-warning-review.md`
(create; must not exist): findings as `### C-n / I-n / M-n / N-n — title` with the
command, verbatim output and the line violated; a table of every shape you
exercised with the verdict the predicate gave; the mutation table; closing counts
and GREEN / NOT GREEN. Return a two-line summary plus the path.
