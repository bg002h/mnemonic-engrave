# IMPL — P0 of `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`

**Executed by:** the single P0 implementer, 2026-08-28/29.
**Plan:** GREEN at `c3fefe4`, executed as written.
**Branches (nothing pushed, no tags, no publishes):**

| repo | branch | head |
| --- | --- | --- |
| `mnemonic-engrave` | `impl/descriptor-s1s3` (cut from master `c3fefe4`) | see the commit table |
| `seedhammer` (worktree `/scratch/code/shibboleth/_work/seam-fork`) | `seam/descriptor-vectors` (cut from `main` `d402f18`) | `1f09537` |

**Verdict: P0 is COMPLETE and every clause of the P0 gate passes.** One
authoring ambiguity was found and resolved with the reading written into the
file; one row could not carry a `wallet_id` and the reason is a measured
toolchain limit, not a spec defect. Both are in *Findings* below. No
F-212-class divergence exists in this corpus.

---

## 1. What was built

### P0.1 — `crates/me-cli/testdata/descriptor_seam_vectors.json`

80,457 bytes · **71 physical rows** · 9 tags · **88 tag-slots** ·
sha256 `0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584`.

Per-tag population, machine-counted, every minimum met **exactly**:

| tag | min | got |
| --- | :-: | :-: |
| `formats-happy` | 4 | 4 |
| `promotion-near-miss` | 15 | 15 |
| `narrowed-4.7` | 14 | 14 |
| `accepted-extreme` | 1 | 1 |
| `narrowed-4.2` | 5 | 5 |
| `neither` | 3 | 3 |
| `whitespace` | 3 | 3 |
| `md1-splits` | 6 | 6 |
| `gate` | 37 | 37 |

Overlap distributed exactly as §7 states: **15** rows carry a second tag (all
fifteen §4.5 rows, each second-tagged `gate`), **2** of those carry a third
(the named pair — `promotion/14-bare-xpub-trailing-newline` adds `whitespace`,
`promotion/01-bare-xpub` adds `formats-happy`), no row carries a fourth.
88 − 17 = **71**.

Gate clause tally, 15+6+2+4+1+3+3+3 = 37, is the same arithmetic PLAN-r4
verified, re-derived here from the rows themselves rather than from the table.

**Every device-side and value column is a measurement taken at authoring
time.** Nothing was transcribed from a report. The measuring apparatus is
committed at `scripts/descriptor-seam-vectors/` (see §4).

### P0.2 — `crates/me-cli/tests/descriptor_seam.rs`

6 tests running today, 6 ignore-tagged. Each ignore reason names the phase that
removes it: **2 name P1** (the cascade + admission predicate; the gate rows'
real invocation) and **4 name P2** (the `md1_admits` column, md1-route
addresses, md1-route `wallet_id`, the read-back pin).

### P0.3 — the fork half

`nonstandard/testdata/descriptor_seam_vectors.json` (byte-identical, `cmp`
verified) and `nonstandard/descriptor_seam_test.go`, `package nonstandard_test`
as §7 requirement 1 requires.

---

## 2. The P0 gate — actual output, pasted

### (a) Go seam test green on the fork branch

```
$ cd /scratch/code/shibboleth/_work/seam-fork
$ go test ./nonstandard/ -v
=== RUN   TestDescriptors
--- PASS: TestDescriptors (0.00s)
=== RUN   TestDecoder
--- PASS: TestDecoder (0.00s)
=== RUN   TestElectrumSeed
--- PASS: TestElectrumSeed (0.00s)
=== RUN   TestDescriptorSeamDeviceColumn
--- PASS: TestDescriptorSeamDeviceColumn (0.00s)
=== RUN   TestDescriptorSeamInvariant
--- PASS: TestDescriptorSeamInvariant (0.00s)
=== RUN   TestDescriptorSeamAddresses
--- PASS: TestDescriptorSeamAddresses (0.01s)
=== RUN   TestDescriptorSeamWalletID
--- PASS: TestDescriptorSeamWalletID (0.00s)
=== RUN   TestDescriptorSeamSyswClass
    descriptor_seam_test.go:388: S2 (F-418): sysw.Classify has no descriptor arm yet, so the 4 sysw_class rows cannot be asserted. Un-skip when §5.2's arm lands -- importing sysw here is why this file is package nonstandard_test.
--- SKIP: TestDescriptorSeamSyswClass (0.00s)
PASS
ok  	seedhammer.com/nonstandard	0.019s

$ go vet ./nonstandard/          -> clean (no output)
$ gofmt -l nonstandard/          -> clean (no output)
```

The `nonstandard` package's own three pre-existing tests are in that run and
pass, so the fork's baseline is untouched.

### (b) Rust harness green on its non-parser assertions

```
$ cargo nextest run --locked -p mnemonic-engrave --test descriptor_seam
        PASS [   0.003s] (1/6) mnemonic-engrave::descriptor_seam the_row_set_is_not_vacuous
        PASS [   0.003s] (2/6) mnemonic-engrave::descriptor_seam every_column_has_the_expected_population
        PASS [   0.004s] (3/6) mnemonic-engrave::descriptor_seam the_file_is_the_one_the_fork_pins
        PASS [   0.003s] (4/6) mnemonic-engrave::descriptor_seam the_row_schema_holds_on_every_row
        PASS [   0.004s] (5/6) mnemonic-engrave::descriptor_seam the_coverage_manifest_is_met_by_count_not_by_reading
        PASS [   0.004s] (6/6) mnemonic-engrave::descriptor_seam every_row_pins_the_digest_of_its_own_input
     Summary [   0.004s] 6 tests run: 6 passed, 6 skipped
```

### (c) The two vendored files byte-identical

```
$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json \
            /scratch/code/shibboleth/_work/seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  crates/me-cli/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
$ cmp <the two>   -> byte-identical

# and the same literal pinned in BOTH tests:
crates/me-cli/tests/descriptor_seam.rs        "0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584"
nonstandard/descriptor_seam_test.go           "0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584"
```

### (d) Per-column count manifests agree across the two suites

```
  rows                   rust= 71  go(wantRows)= 71  AGREE
  device_admits_true     rust= 37  go(wantDeviceTrue)= 37  AGREE
  device_admits_false    rust= 33  go(wantDeviceFalse)= 33  AGREE
  device_admits_absent   rust=  1  go(wantDeviceAbsent)=  1  AGREE
  canonical              rust= 19  go(wantCanonical)= 19  AGREE
  address_0              rust= 20  go(wantAddress0)= 20  AGREE
  address_1              rust=  5  go(wantAddress1)=  5  AGREE
  wallet_id              rust=  4  go(wantWalletID)=  4  AGREE
  sysw_class             rust=  4  go(wantSyswClass)=  4  AGREE
shared-column manifests agree: True
```

The two suites additionally pin *route-scoped* counts that differ by
construction and are reconciled arithmetically: Rust pins
`both_routes_address_0 = 11`; Go pins `wantDeviceAddr0 = 16` and
`wantDeviceAddr1 = 4`. 20 `address_0` rows − 4 the device cannot derive from
the INPUT (the three §4.6 whitespace rows, whose raw bytes the device refuses,
and `neither/wsh-multi`, whose `multi` the device refuses outright) = 16; of
those 16, 11 are also md1-carried and are therefore asserted to ONE value by
both routes; the remaining 5 are `md1_admits=false` rows the Rust half asserts
only as refusals.

### (e) Baseline me-cli suite untouched-green

```
$ cargo nextest run --locked
     Summary [  32.144s] 446 tests run: 446 passed, 7 skipped
$ cargo clippy --all-targets -- -D warnings   -> clean
$ cargo fmt --check                           -> clean
```

7 skipped = this file's 6 ignore-tagged tests + the one pre-existing
`sysw/vectors.rs` fixture regenerator. No pre-existing test changed.

### (f) Staleness re-validation

```
$ ./scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md . c3fefe4
═══ design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
─── against . at c3fefe4 .. 2990917
─── unchanged: 4 ; DRIFTED: 0 ; not in this repo: 4
─── NOT covered: whether the citation was ever RIGHT, drift onto an
───              identical line, and stale prose that cites nothing.
───              Cite the SYMBOL beside the number; this checks bytes.

$ ./scripts/plan-staleness-check.sh <plan> /scratch/code/shibboleth/_work/seam-fork d402f18
─── against /scratch/code/shibboleth/_work/seam-fork at d402f18 .. 1f09537
─── unchanged: 3 ; DRIFTED: 0 ; not in this repo: 5

$ ./scripts/plan-staleness-check.sh <plan> /scratch/code/shibboleth/descriptor-mnemonic 6864f377
─── against /scratch/code/shibboleth/descriptor-mnemonic at 6864f377 .. 6864f377
─── unchanged: 1 ; DRIFTED: 0 ; not in this repo: 7
```

**0 DRIFTED in all three repos.** P0 moved no line any plan anchor cites, in
either repo it touched.

The cite gate reports one failure, and it is a **gate-scope artifact that
predates P0**:

```
$ ./scripts/plan-cite-gate.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
  FAIL  crates/md-codec/src/encode.rs:118          no such file
  ...
   1 unresolvable citation(s) -- fix before review
```

The gate resolves file paths against the CURRENT repo. That path is a
`descriptor-mnemonic` anchor, and it is **correct there** — verified directly
at `6864f377`, lines 118 and 120 are literally
`crate::validate::validate_origin_key_consistency(d)?;` and
`crate::validate::validate_no_duplicate_key_slots(d)?;`, the two validator
calls the published 0.42 crate lacks. The same citation is present at the GREEN
baseline `c3fefe4` (`git show c3fefe4:<plan> | grep -c` → 1), so P0 introduced
nothing. Filing a note rather than editing the plan: the fix is the gate's
cross-repo resolution, not the citation.

---

## 3. Findings

### F-1 (Minor, RESOLVED in-file) — the `format` column is under-determined for rows `me` refuses at the cascade

§7 defines `format` as *"the branch of §4's cascade that `me` MATCHED …  or
`none` where no branch matched"*, while §6 separately ranks *"the branch the
input most RESEMBLES"*. On the five `narrowed-4.2` BlueWallet rows and the
eight refused §4.5 promotion rows the two readings give different answers:
under "matched", `me`'s branch FAILS (it refuses the file) so the value is
`none`; under "resembles", §6 selects branch 1 / branch 4 and the value would
be `bluewallet` / `promoted-key`.

**Resolved as MATCHED = the branch that SUCCEEDED**, per §4.1's *"first branch
that succeeds wins"*, and the reading is now written into the file's own
`_comment` so P1 is not guessing. The consequence is the column's actual
information content: it says whether a refusal came from the **parser**
(`none`) or from the **profile** (`bip380` etc., cascade succeeded, §4.7 or
§5.3 refused). Resulting population: `none` 34 · `bip380` 28 ·
`promoted-key` 7 · `bluewallet` 1 · `json` 1.

**P1 must confirm or overturn this deliberately.** If P1's `admit.rs` reports
these differently, the file changes and both sha256 literals are re-pinned —
which is the mechanism working, not a defect.

### F-2 (Minor) — `accepted/sh-wsh-sortedmulti-16-keys` carries no `wallet_id`, and the reason is a CLI-only guard

The plan scopes `wallet_id` to multisig rows at the device-default use-site;
this row qualifies (`<0;1>/*`, `sh(wsh(sortedmulti(2, …)))`, 16 keys). The Go
side computes it (`bbd2dc3af0bd1c6e301ca2e00bb5197f`), but the **Rust side
cannot be measured in P0**: the row's keys are r6's recorded construction — 16
unhardened children of the `dc567276` fixture key, hence **depth 5** — and
`md-cli`'s key intake refuses them:

```
md: --key @0: expected an account-level xpub at depth 3 or 4
    (this script context conventionally uses 4), got 5
```

That guard lives at `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs:132`
and is the **CLI's alone**. It does not bind `me`, which builds the
`md_codec::encode::Descriptor` in process (§2.6(b)), and the guard's own
comment at `keys.rs:118` records why: *"Widening admission cannot move a wallet
id or change an encoded md1 string: the payload below is `bytes[13..78]`, so
depth never reaches it."*

So the row keeps `md1_admits: true` (correct — its use-site is representable)
and carries device-route `address_0`/`address_1` only. Carrying a one-sided
`wallet_id` was rejected deliberately: the column's whole purpose is that both
languages compute it independently, and a value only one side has verified
would make that guarantee false for one row while looking identical to the
other four. The Rust harness enforces the choice structurally — it asserts that
no row carries a `wallet_id` only one side can compute.

**P2 note:** when the in-process builder exists, this row becomes measurable on
both sides. Adding `wallet_id` to it then is a good, cheap widening of the
F-212 gate.

### F-3 (Nit, FIXED) — P2's ignore-gate could never have reached zero

The plan makes *"ZERO `#[ignore]`"* in `descriptor_seam.rs` P2's gate. As first
written the module doc spelled the attribute while explaining the gate, so the
obvious `grep` counted 6 attributes **plus 3 prose mentions** — and would still
have counted the prose after P2 removed every attribute. A gate that cannot
reach zero gets "fixed" by weakening the grep. The gate is now anchored at
column 0 and the doc says why:

```
grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs   -> must be 0
```

Measured now: **6**. (Commit `2990917`.)

### F-4 (informational) — no F-212-class divergence in this corpus

The plan instructs: if the Rust and Go `wallet_id` disagree, STOP. They do not.
All four `wallet_id` rows agree across **three** independent computations,
checked at generation time, and the generator refuses to write the file
otherwise:

| row | wallet_id | md-cli | published md-codec 0.42 | fork Go `md` |
| --- | --- | :-: | :-: | :-: |
| `formats-happy/bluewallet-sh-fixture` | `a67e07d16b2500fde6c557a76c7390f6` | ✓ | ✓ | ✓ |
| `formats-happy/bip380-sortedmulti-multipath` | `9e95257e60aacbb260129dac7b36d9f4` | ✓ | ✓ | ✓ |
| `md1-split/childless` | `47ecf2de11530f266e9b08640734447a` | ✓ | ✓ | ✓ |
| `md1-split/mixed-childless-and-multipath` | `47ecf2de11530f266e9b08640734447a` | ✓ | ✓ | ✓ |

The **published md-codec 0.42** column is not decoration: it is the exact crate
`me` links, and it is a different artefact from the repo-tree `md` binary the
plan's baseline names. Agreement across the two Rust versions AND the Go port
is stronger evidence than the plan asked for, and it means P2's in-process
builder has a value it can be held to.

The last two rows sharing one id is correct and load-bearing: they are the same
wallet, one written childless and one written childless-plus-`<0;1>/*`, and
§5.3(a′)'s materialisation is exactly the claim that those are one policy.

### F-5 (informational) — two of the spec's own recorded values reproduced fresh

Both were re-derived from the recorded construction, not copied:

- **`39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q`** — §4.7 conjunct 3's r6 address for
  the 16-key `sh(wsh(sortedmulti(…)))`. Reproduced exactly from "16 unhardened
  children of the `dc567276` fixture key", which retires r2's unreproducible
  `3HBBPgNtm…` for good.
- **`bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a`** —
  §5.3(a)'s device-route address, now `md1-split/childless`'s `address_0`.

---

## 4. `scripts/descriptor-seam-vectors/` — why a generator was committed

The plan does not ask for one. It is committed anyway, for one reason: the
file's `_comment` claims every value in it is measured, and a provenance claim
whose reproduction path is a scratch directory is a claim nobody can check.
(Constellation lesson: a generator nobody re-runs rots while its artifact keeps
vouching for it.)

`rows.py` holds the 71 row DEFINITIONS — host-side columns only, authored from
the spec, no measurements. `gen.py` runs the probes and fills every measured
column. It **exits non-zero and writes nothing** on any of:

- a `host_admits=true` row whose `canonical` is missing or is not a device
  fixed point;
- an unexpected parse panic on a row not marked `device_probe`;
- an `address_N` the device route and the md1 route disagree on;
- a `wallet_id` the three implementations do not all agree on;
- an `md_descriptor_contains` pin absent from the real `md descriptor`
  read-back;
- a `want_wid` row with no md1 route to compute the Rust side from.

The `rsprobe/` crate carries its own `[workspace]` table so it is excluded from
the me-cli workspace; `cargo metadata` on the repo is unaffected (verified).
`rsprobe/target/` is gitignored.

**Baselines this corpus was measured against:** fork `main` `d402f18` ·
`descriptor-mnemonic` `6864f377` (debug `md` — the installed
`~/.cargo/bin/md` is stale and lacks `descriptor`, spec §2's trap) · published
`md-codec 0.42.0` · Go `1.26.3`.

---

## 5. Mutation testing — the assertions can actually fail

A green harness over a file it also defines proves little, so both halves were
mutated. **15 mutations, 15 caught**, each by the intended assertion. Every
mutation re-pinned the sha256 first, so the pin was never what caught it.

**Rust (9/9):** drop a required row · drop a row and count around it by
duplicating a tag (*the exact defect §7's manifest exists for* — caught twice,
by the schema's distinctness rule and by the row floor) · rename a field ·
give a gate row an outcome its exit code contradicts · drop a `canonical` from
a host-admitted row · use a `refusal_row` slug the vocabulary does not define ·
rewrite `multi(` to `sortedmulti(` inside an input · make a row host-wider with
no canonical · put a `wallet_id` on a row only one side can compute.

**Go (6/6):** flip a `device_admits` · break a canonical's checksum so it stops
being a fixed point · point a host-admitted row's canonical at a descriptor the
device refuses (**the dangerous direction** — reports *"THE HOST ADMITS WHAT
THE DEVICE REFUSES"*) · corrupt an address · corrupt a `wallet_id` · **remove
the `panic:parse` marker**.

That last one is the most informative result in this section. Without the
marker the suite does not fail — it **crashes**:

```
--- FAIL: TestDescriptorSeamDeviceColumn (0.00s)
panic: runtime error: index out of range [3] with length 1 [recovered, repanicked]
```

which proves two things at once: the marker names a **real** defect at
`nonstandard/parse.go:136-149` (not a guess copied from the spec), and the Go
test's refusal to feed that row to the parser is **load-bearing** — the
false-signal shape §7 warns about is one line away.

---

## 6. What P1's implementer must know

1. **The two P1-owned ignore-tagged tests are stubs, not skeletons.** They call
   `unimplemented!()`. Un-ignore them FIRST, watch them go red, then build —
   that is the plan's TDD instruction and the file was authored to be the
   failing test.
2. **F-1 is yours to confirm.** The `format` column's "MATCHED = succeeded"
   reading is written into the file. If `admit.rs` disagrees, change the file
   and re-pin BOTH sha256 literals — do not weaken the assertion.
3. **`refusal_rows` is a new top-level map** (36 slugs, one per §6 data row,
   machine-counted). It closes PLAN-r4's NEW-M6: the harness rejects any
   `refusal_row` the map does not define, so P2.4's per-row text tests have a
   vocabulary on disk instead of one to reverse-engineer. **P2.4's 36 named
   tests should be keyed to these slugs.** 10 distinct slugs are used by gate
   rows today; the other 26 are reached by P2.4's own fixtures.
4. **One authoring decision to review:** `gate/deadbeef-fronts-an-xpub` carries
   `refusal_row: bluewallet-no-name`. §7 clause 3 names the outcome
   (`descriptor-refusal`) but not the §6 row. The choice follows the DEVICE's
   own precedence — `parseBlueWalletDescriptor` succeeds on that one-line file
   and `OutputDescriptor`'s `bw.Title != ""` gate (`parse.go:37`) is what
   refuses — so the no-`Name:` row is the first cause in cascade order. The
   reasoning is in the row's `source` field. If P1's cascade orders the
   BlueWallet checks differently, this row is the one to revisit.
5. **The gate rows are all four fields, and the Rust half asserts them against
   the real `--as`-omitted invocation** — 37 rows: 7 `as-decides` at exit 2,
   17 `descriptor-refusal` at 3, 12 `record-refusal` at 4, 1 `multi-record` at
   4. The harness already enforces `gate_open == (outcome != "record-refusal")`
   and the outcome→exit_code mapping, so P1 only has to make `me` agree.
6. **The 12 `record-refusal` rows are the SHIPPED surface** (exit 4, record
   vocabulary) and could in principle be asserted before the gate exists. They
   are inside the P1-ignored test deliberately, so the gate's two invariants
   are proven by one mechanism rather than two. If P1 wants earlier signal,
   splitting that test is safe.
7. **Whitespace rows are the only `(host=true, device=false)` rows** and the
   Go half pins that count at exactly 3. If P1's normalisation makes a fourth
   row host-wider, both suites go red on the count before anything subtler
   breaks — that is intended.
8. **P0 changed no existing file** in either repo, verified with
   `git diff --diff-filter=MDR --name-status` against both baselines: **empty**
   in each. `mnemonic-engrave` adds 16 files / 3,530 lines (the vector file,
   the harness, and the generator directory); the fork adds 2 files / 1,685
   lines.

---

## 7. Commits

**`mnemonic-engrave`, branch `impl/descriptor-s1s3`** (from master `c3fefe4`):

| sha | subject |
| --- | --- |
| `4165532` | P0.1: the 71-row descriptor seam vector corpus, measured not transcribed |
| `dbe075c` | P0.2: the Rust half of the seam — red where the parser is, green where the file is |
| `2990917` | P0.2: anchor P2's ignore-gate so it cannot match its own documentation |

**`seedhammer`, branch `seam/descriptor-vectors`** (from `main` `d402f18`,
worktree `/scratch/code/shibboleth/_work/seam-fork`):

| sha | subject |
| --- | --- |
| `1f09537` | nonstandard: the device half of the descriptor seam gate |

This report lands in its own commit on top of `2990917`, so the three P0 work
commits above are exactly the diff a reviewer reads.

Nothing pushed in either repo. No tags, no releases, no publishes, no
on-device actions — the overnight boundaries hold.
