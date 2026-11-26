# seedhammer thresh "any k of n" port, review r1 (2026-11-25)

Repo `/home/user/seedhammer`, branch `claude/project-thread-utm1aa`, commit `a7b805f3` on `c4529d4`.
Primary: descriptor-mnemonic `d05655ac`, `crates/md-codec/src/policy_shape.rs`.
Design: `scratchpad/DESIGN_thresh_summary.md` (as approved).
This review was read-only. The probes ran in a `git archive` copy at `scratchpad/shcopy`, and the repo was not touched.

**Counts: 0 Critical, 1 Important, 3 Minor, 3 Nit.**

## What I ran

- Go 1.26.7 from the module cache: `GOTOOLCHAIN=local PATH=/root/go/pkg/mod/golang.org/toolchain@v0.0.1-go1.26.7.linux-amd64/bin:$PATH`. The `/usr/local/go` binary is 1.24.7.
- `go test -count=1 ./md/`: ok.
- `go test ./gui/ -run 'AnyOf|PolicySummary|Thresh|Unspendable|SelfCheck|Consent'`: ok.
- The whole gui package through `mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 16`: "RESULT: ok -- all 1473 tests ran across 16 shards" in 91 s wall time.
- The fit test logged "11408 entries x 999 entry numbers, 15 prefix-advance groups".
- `gofmt -l .` lists exactly the five baseline files. Nothing new.
- `go vet ./md/ ./gui/`: only the two baseline `testing.ArtifactDir` go1.25-file notices.
- Vendored file: its sha256 `2ea640f2…8dd8` equals `git show d05655ac:crates/md-codec/tests/fixtures/policy_shape/cases.json`. `6620a7dd..d05655ac` touches only `design/FOLLOWUPS.md`, as the commit says.
- I fed a probe test (`scratchpad/shcopy/md/zz_probe_test.go`) the trees `policyShape` is asked about below.

## Important

### I-1. The ported walk inherits the primary's open md-r2 I-1: an untyped `or_b(pk, thresh(...))` gets a complete summary of a wallet anyone can spend

- Code: `md/policy_shape.go:262-301` (`splitBranches`) and `:596-606` (`anyOfRoot`).
- `anyOfRoot` accepts a bare `thresh` in any position, including `or_b`'s second child without `a:`. Nothing in the walk checks the context's types.
- Probe result: `wsh(or_b(pk(@0), thresh(2,pk(@1),s:pk(@2),s:pk(@3))))` gives `Complete=true` with the entries `{Keys:1}` and `{AnyK:2 AnyN:3 Keys:3}`.
- `policySummaryLines` would print "1: 1 key(s), custom / 2: any 2 of these 3 keys".
- As md review r2 I-1 (`scratchpad/reviews/md-r2.md:36-82`) shows, that script spends with no signature: witness `[1,<>,<>,<>]`, where the thresh's first CHECKSIG consumes the 0 left by `pk(@0)`, then BOOLOR.
- So the new any-of text, and the "at least m keys" floor, overstate the signatures for a typed-looking thresh in an untyped context.
- The pre-existing `or_b(pk,pk)` probe is the same class. It is also `Complete=true`, with two 1-key entries.

**Why Important and not Critical:**

- Today only two flows read these shapes:
  - the Multisig Build template consent (`templateConsentFlow`, `gui/multisig_build.go:450,788`), whose input is `md.EncodeMultisig` output (sortedmulti only);
  - the composer readers, whose input is the composer's own output (it has no thresh).
- No device flow renders an outside card's shape yet, so the hole is latent.
- It is still the exact I-1 class ("complete, overstates signatures"). The port also introduces the `orBSecond` context rule without its converse, as Rust does.

**The vectors cannot see it:**

- The 67 cases at `d05655ac` are all typed.
- `TestPolicyShapeAgreesWithRustCases` compares Go with Rust, and both are wrong in the same way.

**What has moved:**

- The primary's working tree (uncommitted, under separate review) adds `policy_type_checks`. Its doc comment calls it "THE TYPE RULE of this summary, normative for every port".
- With it, an untyped tree has no summary.
- `cases.json` will gain a `typed` answer per case.

**Required:**

- Do not record this commit as the finished convergence for seedhammer#29 I-1.
- File a companion FOLLOWUP that cites md-r2 I-1 and the Rust fix.
- Add the context hole to the "Known lags" paragraph in `md/bits.go:27-30`.
- After Rust lands, port the B/V/K/W plus z/o/n/d/u rule for md's fragment set (Rust-first: Go must not lead). Then re-vendor, and gate on the `typed` field.

The re-vendor itself is safe:

- The pin test fails on any sha change, so a stale copy cannot pass silently.
- The Go reader ignores unknown JSON fields. A new `typed` field would therefore not be checked unless the reader is extended. Add it to `shapeCase` and assert `Complete == false` whenever `typed == false`.

## Minor

### m-1. The parity test is vector-only, and the vectors are Rust's own output; there is no Go-side property test of the allowlist

- Code: `md/policy_shape_cases_test.go`.
- Rust has `policy_shape_thresh_props.rs`, a proptest that every accepted thresh `to_miniscript`-type-checks, and it caught `s:pkh` under mutation.
- Go has only the 67 fixed cases, and their `shape` is Rust's output (md-r2 n-2).
- I read `threshConditions`, `threshKey`, `threshLock`, `threshTimelock`, `zeroArmOther`, `anyOfRoot`, `branchOf` and the `collect` thresh arm against Rust line by line, and found no divergence. Specifically:
  - pkh is gated on `s:`;
  - `c:` is unwrapped once only;
  - exactly one `0` arm is required;
  - `1 <= v < 2^31` holds on a `uint32` body (no truncation: `timelockBody` is `uint32`);
  - `k == 0`, `k > n` and duplicate slots are refused;
  - `k == n` uses the same list;
  - `AnyN - Keys == len(Locks)`.
- But a later Go-only edit that loosens the list would be caught only if one of the 67 trees exercises it.
- Several Go branches have no case of their own:
  - a descriptor-level `tagPkh`/`tagWpkh`/`tagRawPkH` as a thresh child;
  - `after(>= 2^31)` (only `older` bit 31 is covered);
  - `a:thresh(k == n)` as `or_b`'s second child;
  - thresh as `or_i`'s left child.
- Suggestion: a Go fuzz test that builds random wrapper stacks over KEY/LOCK and asserts the accepted set equals a small, independent list of spellings. Or add these cases on the Rust side, so the shared file carries them.

### m-2. The new any-of root brings the primary's open m-2 into the device text: an `or_i(0|1, thresh)` root shows a phantom or anyone-can-spend entry next to the any-of

- Code: `md/policy_shape.go:262` (`splitBranches` splits `or_i` at a root) and `:509` (`collect` accepts `tagTrue`/`tagFalse`).
- Probe: `wsh(or_i(0, thresh(2,...)))` (that is, `l:thresh`) gives `Complete=true` with `{Keys:0}` and the any-of.
- The device prints "1: 0 key(s), custom" for an arm that can never be satisfied. `or_i(1, thresh)` prints the same text for an arm anyone can spend.
- This predates the change and is identical in Rust (md-r2 m-2).
- It is listed because the design widened what reaches the summary: before, these trees were summarised the same way, but as conjunctions.
- Fix belongs in Rust first.

### m-3. The provenance field `repo_clean_when_recorded` over-claims, and no test reads it

- Code: `scripts/vendor-policy-shape-cases.sh:21` checks `git status --porcelain -- "$REL"`, which covers the cases file only.
- At review time the primary's tree had uncommitted changes to `policy_shape.rs`, `skeleton.rs` and `to_miniscript.rs`. The pin would still say `true`.
- This time the content is right (sha equals the committed blob). But the field name claims a clean repo, and `policyShapeCases` never asserts it.
- Two possible fixes:
  - check the whole repo, or rename the field to `file_clean_when_recorded`;
  - better, have the script vendor `git show "$commit:$REL"` rather than `cp` from the working tree. The pinned commit and the bytes then cannot disagree.

## Nit

- **n-1.** `gui/policy_summary_anyof_test.go` golden `thresh_mixed_after_kinds` reads "NO KEY NEEDED", but a key is needed, because a height lock and a time lock cannot both be met. The test comment says so. The wording errs in the safe direction and is owner-approved; listed only so it is a known, named imprecision.
- **n-2.** The fit test's "10 to 999" bound is not tied to a proven maximum branch count. An entry number of 1000 or more would get a 6-character prefix that no measured group covers. This is implausible within md1 size limits, but the test could assert the bound it relies on, or measure up to the wire's real maximum.
- **n-3.** Go `collect`'s `tagAfter/tagOlder` arm (`md/policy_shape.go:476`) sets `Timelock` without a `Lock` when the body is not `timelockBody`, where Rust refuses. This is already filed. The decoder always builds `timelockBody`, and the any-of path uses `threshTimelock`, which does refuse. So the gap affects only hand-built conjunction trees.

## Checked and found sound

**The walk matches Rust.**

- `splitBranches` passes `orBSecond = (tag == or_b)` to the second child only, and `false` everywhere else (`andor` X/Y/Z included).
- `andor` X/Y go through the synthetic `and_v` and `collect`, so a `k < n` thresh there is refused.
- `v:thresh` under `or_c` is refused.
- A tap leaf `a:thresh` and an `or_d` second-child `a:thresh` are refused (probe `or_d(pk, a:thresh)` gives `Complete=false`).

**Allowlist hunt targets, all refused:**

- a timelock without `n:` (`sl:`, `su:`, `a:older`, `an:older`);
- `s:pkh`;
- `s:` on X_1;
- a bare later child;
- `av:`, `sv:` and `aj:`/`an:`/`al:` over pk;
- `adv:older`;
- `ss:pk` and `sc:c:pk`;
- `older(0)` and `older(2^31)`;
- an `or_i(0,0)` child;
- multi, hash, nested-thresh and compound children;
- duplicate slots;
- `k = 0` and `k > n` (the Go decoder also makes `k >= 1`, and refuses `k > count` itself).

**Device text cannot overstate a key's necessity for a typed tree.** Every accepted child requires its own key's signature (pk/pkh with CHECKSIG; wire `PkK`/`PkH` carry the implicit `c:`, `md/script_emit.go:159-200`). `AnyOfMinKeys` counts every lock as meetable, so it can only understate.

**The composer guards are sufficient.**

- Every reader of `PolicyShape.Branches` in the repo is one of:
  - `policySummaryLines` (the any-of arm comes first);
  - `composerConsentLinesFor` (guard before the first `Branches` use, including `composerMixesLockBases` and `composerLianaOutsideModelClass`);
  - `composerSelfCheck`;
  - `composerUnspendableJudge`.
- There are no readers outside `gui`.
- Each of the three guard tests asserts the guard's exact refusal on a shape that is otherwise complete. The unspendable test first proves the conjunction reading would fire, so it is not vacuous.
- A false conjunction or a false "no locked path" cannot be shown from these flows.

**The fit-test sampling assumption holds.**

- `text.Layout.init` (`gui/text/text.go:380-410`) breaks on the cumulative fixed-point width and the last space before overflow.
- Kerning is per adjacent pair (`state.next`). The prefix always ends in `": "` before `"any"`, so the kerning pair across the boundary is constant.
- `body` style is start-aligned. The prefix has its break candidates before the entry.
- Two prefixes with equal advance therefore give identical breaks. The test also cross-checks one other member per group.
- `confirmReviewScreen` lays lines out with `ctx.Styles.body` and `dims.X - 16`, which is exactly what the test measures.
- Thresh `n <= 32` is the wire bound (5-bit count), so the 11,408 enumeration is complete.

**Cards without a thresh are unchanged.** `anyOfRoot` acts only on `tagThresh` or `a:thresh`, and only the thresh arm of `collect` changed. The full gui suite and the md suite pass, with no existing golden edited.

**Vendoring.** The file is byte-equal to the pinned commit. The case count (67) and the from-card count (64) are asserted, and the class-coverage test guards against a regeneration that drops a class. `md/bits.go` records the 0.49.0 convergence.
