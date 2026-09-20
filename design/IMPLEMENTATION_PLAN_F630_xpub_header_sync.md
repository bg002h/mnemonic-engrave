# Implementation plan — F-630, the xpub-header corpus sync and the gate that can see it

**Baseline revisions.** seedhammer (fork) main `95716e97`; descriptor-mnemonic
main `b2c5d693` (md-codec 0.45.0 / md-cli 0.17.0); mnemonic-engrave master
`ed4b98fc`. Every count below was measured against those trees on 2026-09-20,
not read from a report.

## What recon falsified

F-630 was filed as "port md-codec 0.44.0's xpub-header rewrite into the fork".
Three of its four prescribed steps do not survive measurement.

| F-630 / continuity step | measured status |
| --- | --- |
| 1. Assert `.chains[].descriptor` in `TestKeyedConformanceAgreesWithRust` | **NOT DONE — this is the whole remaining job** |
| 2. Re-vendor `keyed_compose_*` from dm `b2c5d693` | **PARTIAL** — the script's pattern reaches 32 of the 41 drifted records |
| 3. Port 0.44.0's header rule into `md/` | **NOT NEEDED — the fork was never wrong** (measured, below) |
| 4. Bring 0.45.0's four `precedence_*` cases across | **ALREADY DONE** — 8 cases run, 4 precedence kinds asserted |

### Step 3 is unnecessary, measured rather than argued

`md-codec` 0.44.0 fixed `assemble_origin_and_xkey`, which built a rendered
xpub's header from nothing while building the origin from `e.origin_path`, so
every emitted key serialised at depth 0 / child 0 under a four-component
origin. The fork has no analogue of that function. Its render path is
`gui/md1_expand.go:expandedKeysToBip380` → `bip380.Key.ExtendedKey()`
(`bip380/bip380.go:97-109`), which has always taken

    depth    = uint8(len(k.DerivationPath))
    childNum = k.DerivationPath[len(k.DerivationPath)-1]

from the one origin it was given. A throwaway probe in package `gui` rendered
`keyed_compose_preset_plain_multisig` from its vendored card:

    wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fz…

which is **byte-identical, key for key, to dm `b2c5d693`'s corrected
`chains["0"].descriptor`**. The fork already satisfies the invariant. What is
stale is the vendored corpus, and what is missing is a gate that can see it.

`ParentFingerprint: 0` is set explicitly at `gui/md1_expand.go:143`, matching
0.44.0's "parent fingerprint stays zero, necessarily" — the parent point is not
on the md1 wire.

### Step 4 is already complete

`go test ./md/ -run TestComposeKeylessCap -v` reports
*7 refused, 1 admitted, kinds map[KeylessUnderTr:1 LegacyWrapperShape:1
NoKeyedPath:1 TooManyKeylessPaths:3 TooManySlots:1]*. The vector is pinned at
`b2c5d693` and the test drives every case out of the file, so all four
`precedence_*` cases already execute. Nothing to port.

## The measured drift

46 keyed conformance records, compared file by file:

| class | count | what moved |
| --- | --- | --- |
| identical | 2 | `keyed_tr_keyonly`, `keyed_wpkh` — bare-fingerprint origins, so depth 0 was already right |
| descriptor-only, script-covered | 32 | `keyed_compose_*` |
| descriptor-only, **not** script-covered | 9 | `keyed_tr_depth2`, `…_rightspine`, `keyed_tr_pathological`, `keyed_tr_with_leaf`, `keyed_wsh_multi_2of3`, `keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`, `keyed_wsh_sortedmulti_2of3`, `keyed_wsh_thresh` |
| semantic — **F-529** | 3 | `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`, `keyed_wsh_timelock_hashlock` |

88 chain-descriptor entries move across the 41. Zero address lines and zero id
lines move in those 41 — the three records whose `keys`, `fingerprints`,
`wallet_policy_id`, `md1_encoding_id`, `wallet_descriptor_template_id` and
addresses all move are exactly F-529's three, and none of them is a
`keyed_compose_*`.

**The corpus has two tiers and only one is pinned.** `scripts/vendor-compose-vectors.sh`
selects `^(keyed_)?compose_` (177 files, `compose_vectors.provenance.json`).
The other 14 keyed vectors have no provenance pin at all, and that unpinned
tier is where F-529's divergence lives. A re-vendor run as the script stands
today would fix 32 records and leave 9 stale — shipping the new gate red.

## Decisions

**D1 — the assertion is over the header, not the descriptor string.** The fork
renders multipath (`<0;1>/*`) while the Rust record splits per chain (`/0/*`),
so the two spellings are equivalent and not comparable as strings. For every
`[origin]xpub` in every `chains[].descriptor` the gate asserts: the xpub
base58-decodes with a valid checksum; `depth == len(origin components)`;
`child number == terminal origin component` (hardened-encoded, 0 for an empty
path); `parent fingerprint == 0`.

**D2 — bind the record's two spellings, and bind them to the Go port.** The
0.44.0 defect survived because "a uniformly wrong corpus agrees with itself":
`keys[]` held the correct depth-4 key and `descriptor` held the depth-0
rendering. So the gate also asserts that each descriptor xpub's
(chain code ‖ compressed pubkey) equals some `keys[]` entry's, **and** that it
equals some slot of the Go port's own `ExpandWalletPolicyChunks` expansion of
the same card, with every Go slot carrying an xpub appearing at least once.
That last clause is what makes it cross-language rather than a JSON
self-consistency check.

Note `keys[]` xpubs carry the **real** parent fingerprint (measured: `1cf29716`,
`3edf3f57`, `64f9d328` for `keyed_compose_preset_plain_multisig`) while the
descriptor's carry zero — so the comparison is over the 65 bytes, never the
base58 string.

**D3 — the elided-origin divergence is pinned, not skipped.** For
`keyed_wpkh` and `keyed_tr_keyonly` the Rust record emits a bare-fingerprint
origin (`[73c5da0a]`, depth 0) while the Go port canonical-fills it from
`canonicalOrigin(tree)` and would render `[73c5da0a/84h/0h/0h]` at depth 3 —
the deliberate R0-I1 divergence `md/expand.go:76-81` documents. Both satisfy
D1 independently. The gate therefore does **not** assert bracket-path == Go
origin path globally; it carries an explicit allowlist naming those two
vectors with their measured Go paths, and fails if an allowlisted vector stops
diverging or a non-allowlisted one starts. A pinned gap with an exact shape,
per the repo convention; not a skip.

**D4 — widen the vendor script to the whole keyed tier.** Leaving 9 records
stale ships a red gate, which is worse than no gate. `vendor-compose-vectors.sh`
moves from `^(keyed_)?compose_` to the whole `keyed_*` ∪ `compose_*` set, and
its provenance pin grows to cover them.

**D5 — F-529's three keep a fork-side witness before they are re-vendored.**
`keyed_tr_multi_a` and `keyed_tr_sortedmulti_a` are already pinned in
`md/testdata/forkbuilt/`. `keyed_wsh_timelock_hashlock` is **not**, and it is
the only fork witness for `md.DuplicateRefusedByCore` — depended on by
`md/duplicate_keys_test.go:84`, `gui/composer_flow_test.go:599,684` and
`gui/policy_address_test.go:149`. Re-vendoring it without a pin first would
swap a 3-key duplicate policy for a 5-key reuse-free one and leave those
assertions green against a shape that no longer carries the defect. It gets a
pin of its own first, with its own shape test (the existing
`TestPinnedKeyReuseVectorsAreTheShapeTheyClaim` asserts a *taproot* internal-key
reuse and cannot cover a wsh duplicate).

## Tasks

Each task ends green before the next begins; the gate command is stated per task.

**T1 — the gate, RED first.** Extend `keyedConformanceRecord` in
`md/conformance_keyed_test.go` with `Keys`, and add
`TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins` implementing D1,
D2 and D3. Run it against the **stale** corpus and record the output: it must
fail on the 41 drifted records and pass on the 2 identical ones. That RED is
the evidence the gate has teeth, and it is captured in the commit message.
Gate: `go test ./md/ -run TestKeyedConformance -v`.

**T2 — the `keyed_wsh_timelock_hashlock` pin (D5).** Copy the vendored phrase
to `md/testdata/forkbuilt/keyed_wsh_timelock_hashlock.md1.txt`; add a
`pinnedDuplicateVectors` list and a shape test asserting the pin carries a slot
appearing at two use sites under one wsh miniscript (the shape
`DuplicateRefusedByCore` is about), plus the anti-drift check against the
vendored file while it is still present. Point the three dependent test sites
at the pin-preferring loader. Gate: `go test ./md/ ./gui/ -run 'Duplicate|Pinned' -v`.

**T3 — widen the vendor script (D4).** `scripts/vendor-compose-vectors.sh`
selects the whole keyed tier; the provenance pin records the new file set and
`b2c5d693`. Assert the union is exhaustive — no file in `testdata/vectors`
matching the widened pattern may be outside the pin, and the existing
directory-scan test already enforces the converse for refusal vectors.
Gate: run the script, then `go test ./md/ -run 'Provenance|Pin' -v`.

**T4 — re-vendor.** Run the widened script against dm `b2c5d693`. Expect
exactly 41 conformance records to change descriptor lines, 3 to change
wholesale, 0 address lines and 0 id lines outside those 3. Verify that
expectation with a diff count before committing. T1's gate must now be GREEN.
Gate: the full `./md/` and `./gui/` suites.

**T5 — whole-surface gate.** `go vet` (ArtifactDir baseline only), `gofmt -l .`
against the five-file baseline, `./md/ ./sysw/ ./mk/`, the whole `gui` via
`scripts/gui-shard-test.sh ./gui/ 24`, and the firmware size measurement.
Then push via `scripts/push-via-staging.sh` with main frozen for the window.

## What this plan does NOT cover

- No normative Go behavior changes. T1–T4 touch tests, fixtures, vendored data
  and one shell script. If T4 turns any `./gui/` or `./md/` test red for a
  reason other than a stale expected-value, that is a finding, not a fixup —
  it means the fork and the primary disagree somewhere this plan assumed they
  agreed, and it stops the plan.
- F-529 is **narrowed, not closed**, by T2 + T4: its three vectors get
  fork-side witnesses and the corpus stops diverging silently, but the
  question of whether the device should carry reuse-free or reuse-bearing
  fixtures for F-514's warning is a separate ruling.
- The 17 dm-only `.template` files and the 36 fork-only fixtures are
  out of scope; neither tier feeds the conformance gate.
