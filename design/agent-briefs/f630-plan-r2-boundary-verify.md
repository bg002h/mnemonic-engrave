# Brief — does D5a's boundary close the class, or does the class move again? (sonnet)

This is round 2 of a pattern this repo has seen before: a review finds a
defect, the fold fixes it, and the next round finds the same class one level
inside the remedy. Round 1 found C-1 (a pinned card paired with a vendored
record). The fold wired one call site and claimed a global property. Round 2
(NEW-1) found two more sites. The fold under review now replaces call-site
edits with a **boundary**: one `loadVectorRecord` loader plus a structural test
that fails on any direct record read in `gui/`.

**The ONE question: does the boundary actually close the class?**

Answer these five, nothing else:

1. **Can the structural gate be defeated?** It scans `gui/*_test.go` for a
   direct `filepath.Join(.., "md", "testdata", "vectors", …".conformance.json")`
   read outside `loadVectorRecord`. Name a spelling a future test could use
   that reads the vendored record and the scan misses (a variable path, a
   helper in another file, `os.DirFS`, a glob whose result is read later, a
   `const`, string concatenation instead of `filepath.Join`). One concrete
   spelling is enough.
2. **Can it report a FALSE PASS?** A source-scanning test that matches zero
   files passes vacuously — the same trap as a `-run` filter that matches
   nothing. Does the plan require the gate to assert it actually examined the
   files it claims to? State what it must assert to be non-vacuous.
3. **Does routing EVERY reader through the loader break a currently-green
   test?** The plan deliberately admits no exception list, so the four
   non-intersecting readers get routed too:
   `gui/policy_address_test.go:261`, `gui/key_card_seating_test.go:40,243`,
   `gui/composer_policy_address_test.go:48`. For each, does a pin-preferring
   record change its behaviour today (is there a `forkbuilt/` record for the
   vector it names)? A loader that silently prefers a pin for a vector whose
   pin exists but whose record does not is a trap worth naming.
4. **Is `md/` in or out?** The boundary is stated for `gui/`.
   `md/conformance_keyed_test.go:44` globs records and `md/policy_shape_test.go:19`
   uses the pin-preferring `vectorChunksFor`. Is there a split pairing inside
   `md/` that the gui-scoped boundary leaves open — now, or after T2 adds
   three `forkbuilt/` records?
5. **Does T2's ordering still work?** T2 both introduces the loader and adds
   the gate. Can the gate pass at T2 while the corpus is still stale, or does
   it depend on T3?

## Already established — do not re-derive

- The three split sites and their line numbers are verified:
  `gui/policy_address_test.go:125/:174`, `gui/taproot_script_path_test.go:30/:55`,
  `gui/wsh_script_emit_test.go:31/:72`.
- The four non-intersecting readers and the vectors they name are verified:
  `vectorAddress` → `keyed_tr_with_leaf`, `keyed_wsh_thresh`;
  `key_card_seating_test.go` → `keyed_tr_with_leaf`,
  `seat_same_origin_two_masters`; `composer_policy_address_test.go` →
  `keyed_compose_wsh_timelock_hashlock` (the compose variant, NOT the pinned
  `keyed_wsh_timelock_hashlock`).
- `md/testdata/forkbuilt/` today holds five `.md1.txt` files and no records:
  `dup_seat_wsh_sortedmulti_k1{,_keyless}`, `dup_seat_wsh_sortedmulti_k2`,
  `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`.
- All 14 r0 findings are FIXED; that is settled and not reopened.
- The fork already emits the 0.44.0-correct header; F-630 steps 3 and 4 are
  unnecessary/done. Not open for re-litigation.

## Out of scope

Re-verifying the r0 findings. Style and wording. Proposing the plan do more
than close this class. Reviewing an implementation — none exists.

## Rules of evidence

A finding must name a concrete failure: the state, and the wrong outcome. For
Q1 give the actual spelling that slips past. Run commands in the fork rather
than reasoning from the plan's prose. Do NOT modify tracked files; delete any
scratch file and verify both trees clean.

## Output

Severity per project standard. **FINAL ACTION: write the report to
`design/agent-reports/f630-plan-r2-boundary-verify.md`** and return ONLY a
one-paragraph summary, that path, and C/I/M/N counts.
