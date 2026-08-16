# S5.0 oracle block — independent adversarial review

**Artifact:** worktree `/scratch/code/shibboleth/seedhammer-s5`, branch `s5-oracle-block`,
three commits on `main` @ `80d0c5d`: `3c879e7` (S5.0a pin bump + re-anchor),
`5edb162` (S5.0b built-policy `ExpectKind`), `5ed87c7` (S5.0c inline per-`@N` origins).
**Date:** 2026-08-15. **Reviewer:** independent context, read-only on source.

**THE ONE QUESTION — would this machinery report a byte-identity PASS for an engraving
that does not match what the primary toolchain produces?**

**Answer: not as committed. The derivation is correct and I could not make it produce a
wrong wallet.** But the *only* proofs of that correctness live behind the `oraclelive`
tag, and the mechanism that decides which of those proofs execute is a comment. I
demonstrated the whole repo green — untagged suite, tagged compile, 32-bit check, and
`./scripts/oracle-live.sh` reporting "live checks: PASS (exit 0)" — with the oracle
handing every policy placeholder another slot's account xpub.

## Counts

| severity | n |
| --- | --- |
| Critical | 0 |
| Important | 2 |
| Minor | 3 |
| Nit | 2 |

**2 Important — the block does NOT close green.**

## Environment

`go` is not on PATH in this environment; the repo needs Go **1.26** (`testing.T.ArtifactDir`
is used in 7 `_test.go` files) although `go.mod` says `go 1.25.10`. I used
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go` with a pinned `GOCACHE`
on both sides of every comparison. All three pinned binaries are installed and their
SHA-256 match `oracle/pins.json` exactly.

Restored-tree baseline, re-measured by me at the end (all mutations reverted,
`git status --porcelain` empty, HEAD `5ed87c7`):

```
go test ./... -count=1        exit 0, 51 ok / 0 FAIL
gofmt -l .                    no output
go vet ./...                  exit 1, 40 findings, 0 outside _test.go
./scripts/oracle-live.sh      exit 0, 7 --- PASS
```

---

# IMPORTANT

## I-1 — the `oraclelive` allowlist is enforced by prose, and an unmatched `-run` filter exits 0, so `oracle-live.sh` can print "live checks: PASS" having executed none of S5.0's proofs

**`scripts/oracle-live.sh:59-63` (the rule) and `scripts/oracle-live.sh:99-102` (the filter).**

The script's own comment states the hazard and then does not check it:

> `# EVERY TAGGED TEST MUST BE NAMED IN THE -run FILTER BELOW. That filter is an`
> `# ALLOWLIST: a test added behind the tag and not added there still compiles, and`
> `# still passes ... on CI, and never executes anywhere. A check that exists and`
> `# never runs is the exact defect this deliverable was filed about.`

`go test -run <matches nothing>` exits **0** and prints `[no tests to run]`; the script
captures that `rc` and prints `live checks: PASS (exit 0)`. So the allowlist failing is
indistinguishable from the checks passing.

This matters more for S5.0 than for anything before it, because **every** proof that the
built-policy derivation encodes the intended wallet is behind the tag:
`TestBuiltPolicyDerivationMatchesTheS2Golden` (the byte cross-check against S2's golden),
`TestBuiltPolicyDerivesDivergentOrigins` (path_decl mapping through md's encoder, the
flatten control, and Trace B's shape). The untagged arm proves only string-builder
properties.

**Evidence I ran.**

1. Filter matching nothing:
   `go test -tags oraclelive -count=1 -v -run 'TestThisNameDoesNotExistAnywhere' ./oracle/ ./gui/ ./sysw/`
   → `PASS`, `ok ... [no tests to run]` for all three packages, **exit 0**.
2. Renamed both S5.0 tagged tests to names the filter does not match
   (`TestBuiltPolicyDerivesDivergentOrigins` → `TestDivergentOriginsSurviveTheEncoder`,
   `TestBuiltPolicyDerivationMatchesTheS2Golden` → `TestBuiltPolicyMatchesTheS2Golden`)
   → `./scripts/oracle-live.sh` printed **`live checks: PASS (exit 0)`** with only 5 of the
   7 tests executed and no mention of the two that vanished. CI's tagged-compile step
   (`go test -tags oraclelive -run '^$'`) also stayed exit 0.
3. **The decisive combination.** With that rename in place I also mutated
   `oracle/expect.go` `mdEncode` so the xpub for `@i` comes from `xpubs[len-1-i]` —
   i.e. slot *i*'s declared origin sits beside another slot's account key:

   ```
   go test ./...                                        exit 0   (51 ok / 0 FAIL)
   go test -tags oraclelive -run '^$' ./oracle ./gui ./sysw   exit 0
   ./scripts/test-32bit.sh                              exit 0
   ./scripts/oracle-live.sh                             exit 0
   ```

   Every gate in the repository green while the oracle's md1 declares each cosigner's key
   at another cosigner's derivation path. **A wallet whose descriptor pairs every
   fingerprint/path with the wrong key: a signer restoring from those plates looks for its
   key at a path it does not own, and the cosigner set the gate vouched for is not the one
   the operator built.** (Verified separately that this mutation is caught *only* by
   `TestBuiltPolicyDerivationMatchesTheS2Golden` — with the allowlist intact it dies loudly,
   printing all six chunk mismatches; the untagged suite reports `ok` either way.)

**Minimal fix.** Make the allowlist a command rather than a discipline. In
`scripts/oracle-live.sh`, before the run: extract `^func Test` from the three
`//go:build oraclelive` files, require each name to appear in the `-run` string, and fail
otherwise; and after the run, require the observed `--- PASS` count to equal the number of
allowlisted tests so `[no tests to run]` can never read as success. Both are a few lines of
`grep`/`comm` over output the script already produces.

## I-2 — `Artifact.Kind` is never bound to `Artifact.String`'s bytes, so `CheckArtifactShape`'s stated C2 guarantee ("a Full backup with no seed in it") and watch-only's no-seed-on-steel guarantee are enforced against a free-text label

**`oracle/expect.go:169-202` (`CheckArtifactShape`), `oracle/expect.go:224-256`
(`CheckFingerprintScope`), `oracle/expect.go:826-854` (`CompareCensus`).**

`CheckArtifactShape`'s doc comment claims it catches

> "a MISSING class (a full-mode expectation holding no ms1 at all — the C2 defect ...)
> and a REORDERED one"

but it reads only `arts[i].Kind`, a string field in the committed JSON that nothing
compares against the artifact's own bytes. `CheckFingerprintScope` dispatches on the same
field. `CompareCensus` is Kind-blind by construction. I grepped the tree: the only
`HasPrefix("ms1"/"mk1"/"md1")` checks in `oracle/` are on the **oracle's stdout at
derivation time** (`expect.go:987`, `:1084`, `:1156`) — i.e. behind the toolchain — and
`cmd/emu/gaterecord_anchor_test.go:74` only inspects census strings for records naming the
cosigner payload. Nothing untagged binds a committed expectation's kind to its plate.

Consequence: the two failures this pair exists to name are both defeated by editing one
word.

**Evidence I ran** (temporary `oracle/zz_probe_test.go`, since removed):

- A `built-policy-full` set whose single `"kind":"ms1"` artifact holds an **md1 string**
  (`md1fxrvxzspqjtvyyy4qq`) with a fingerprint attached was **accepted** by
  `CheckArtifactShape`, **accepted** by `CheckFingerprintScope`, and **accepted** by
  `CompareCensus`. That is a "Full" backup carrying no seed plate at all — precisely the
  C2 shape — passing every toolchain-free check.
- A `built-policy-watch` set whose `"kind":"mk1"` artifact holds an **ms1 string**
  (`ms10entrsqqqqqqq`) was **accepted**. Watch-only exists so that no seed reaches steel;
  the guarantee is currently carried by a label.

This is not reachable from an honest `cmd/gaterecord` mint (`DeriveExpected` always emits
the right kinds), which is exactly why it matters: the pair's *entire* purpose is to police
a committed file that a human wrote or edited, and against that adversary it is nearly
vacuous. `CheckProvenance`'s own refusal text — "re-mint the expectation, do not edit it" —
exists because hand-editing is the realistic failure.

**Minimal fix.** One check, in `CheckArtifactShape` (or a sibling called from the same two
untagged tests): require `strings.HasPrefix(a.String, a.Kind)` for every artifact. The
engraved forms are `md1…`/`mk1…`/`ms1…` by construction, so this is exact, needs no
toolchain, and turns "a Full backup with no seed" into an untagged failure.

---

# MINOR

## M-1 — a wholly hand-authored built-policy trio passes the entire untagged suite green; only `oraclelive` refuses it, and unlike Trace A there is no payload anchor even in principle

**`oracle/expect_test.go:156-207`, `cmd/emu/gaterecord_anchor_test.go:68-71`.**

I planted `S5-fake.{record,walk,expect,inputs}.json` in `oracle/gaterecords/`: kind
`built-policy-full`, `held_slots [0,1]`, divergent origins, a fabricated payload
name/digest, provenance copied from `pins.json`, and **five strings nothing derived**
(`ms1invented…`, `mk1invented…`, `md1invented…`). Result:

```
go test ./oracle/ -count=1     exit 0
go test ./... -count=1         exit 0, 51 ok / 0 FAIL
```

with `expect_test.go:123` logging
`S5-fake.record.json: 5 committed artifact(s) matched the engraved census byte for byte`
and `record_test.go:395` logging `verified 2 gate record(s)`. Every structural gate passed:
`CheckProvenance`, `CheckArtifactShape`, `CheckFingerprintScope`, `TestPlateCountIsDerived…`,
`VerifyRecord`. Only `./scripts/oracle-live.sh` refused it (exit 1, at `ms derive`).

Two things keep this at Minor rather than Important. First, the design **states** this
residual and the stated bar is met — `expect_test.go:166-169` says a fabricator must forge
an inputs file "that the live arm will contradict the moment anyone runs it", and mine was
contradicted. Second, the same construction works for `cosigner-cards`, so S5.0 did not
introduce it.

What S5.0 *does* change is the backstop. Trace A has an independent untagged anchor —
`TestGateRecordStringsAreRecordsOfTheCardsPayload` requires every engraved `mk1` to be a
chunk of the committed cosigner blob. A built policy's md1/ms1 are **produced**, not
supplied, so no analogous anchor can exist; and the existing one is skipped outright for any
record naming a different payload (`gaterecord_anchor_test.go:68` `continue`s with a
`t.Logf`), which my fake exploited. Worth recording in FOLLOWUPS as the accepted residual
for the S5 kind, with I-2's prefix check as the cheapest available narrowing.

## M-2 — three tagged files, one of them re-asserted by S5.0b, cite a CI command the workflow deliberately does not run

**`scripts/oracle-live.sh:61` (added by `5edb162`), `scripts/oracle-live.sh:66`,
`oracle/live_test.go:74`, `gui/multisig_build_oracle_live_test.go:25`,
`sysw/vendored_vectors_live_test.go:24`.**

All five say the tagged files are type-checked by **`go vet -tags oraclelive`** in
`.github/workflows/test.yml`. The workflow runs
`CGO_ENABLED=0 go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`
(`.github/workflows/test.yml:68`) and explains in the same file *why* `go vet` is unusable
there: "vet reports 40 pre-existing findings in `_test.go` files across this tree …, so a
vet step here would fail on day one". I confirmed both: `go vet ./...` exits 1 with 40
findings, 0 outside `_test.go`.

The property (tagged files are compiled on every push) genuinely holds — this is a false
citation, not a coverage gap. It is still the comments-outlive-their-conditions class, and
`git log -S` shows `5edb162` added a **new** sentence carrying the wrong command rather than
inheriting it. Fix: change the four citations to name `go test -tags oraclelive -run '^$'`.

## M-3 — `InputTuple.SlotOrder` and `InputTuple.FPChoice` are recorded and read by nothing in the derivation

**`oracle/oracle.go:217-226`, `oracle/expect.go:372-410`.**

`DeriveExpected` never consults `SlotOrder` or `FPChoice`; the artifact order follows the
`Origins`/`seeds` index and `HeldSlots`. So an S5 record can carry a `slot_order` that
contradicts its own origins array and nothing notices. The direction is fail-safe (a device
that honoured `slot_order` would produce a census the oracle does not match, so it FAILS
rather than passing), but the tuple currently has two fields that read as load-bearing and
are inert. Either drive the derivation order from `SlotOrder`, or refuse a `slot_order`
that is not `0..n-1` so the record cannot make a claim the deriver ignores.

---

# NIT

## N-1 — `templateForOrigin` discards `strconv.Atoi`'s error at every level

**`oracle/expect.go:869-871`.** `purpose, _ :=` / `coin, _ :=` / `account, _ =`. For
`purpose` and `coin` an overflow is fail-closed (the clamped value fails the `!= 48` and
coin-type switch). For `account` it is not: `m/48h/0h/2147483648h/2h` passes
`templateForOrigin` and `inlineOrigin` (I measured: `inlineOrigin` returns
`48'/0'/2147483648'/2'`), and is refused only downstream by `ms` (measured: `ms derive
--account 2147483648` exits 1). `templateForOrigin`'s doc comment calls itself "the gate's
single authority on what an origin may be", so a hardened index ≥ 2³¹ belongs here.

## N-2 — `3c879e7`'s "only behavioural change" claim understates the ms 0.16.0 delta

The commit message says 0.15.0→0.16.0's "only behavioural change is a bare `--template
bip48` alias and a notice emitted only when that bare form is used". Reading the diff,
`crates/ms-cli/src/format.rs` also adds `script_type_defaulted` to `DeriveJson` **without**
`skip_serializing_if`, so `ms derive --json` now emits an extra field on every invocation.
Harmless here — `oracle/expect.go:920-924`'s `msDeriveJSON` uses plain `json.Unmarshal`,
which ignores unknown fields, and the six mk1 strings reproduced byte-for-byte — but the
claim as written is incomplete.

---

# CLEAN PROBES — what I ran, and what it showed

Every probe below was executed; none is inferred from reading.

### P-1 The origin→placeholder mapping dies under the reversal mutation ✅

Mutated `oracle/expect.go` `policyTemplate` to build `revd` and map slot *i* to
`@(n-1-i)` — the `(0..n).rev()` defect verbatim.

- **Untagged** `TestPolicyTemplateMapsSlotIOriginToPlaceholderI` → **FAIL**.
  (`TestPolicyTemplateEncodesTheDevicesOwnS2Wallet` stayed PASS, correctly: its fixture is
  uniform.)
- **Live** `TestBuiltPolicyDerivesDivergentOrigins` → **FAIL**, at the assertion that reads
  the origins back out of the encoded bytes:
  `SLOT 0's origin did not land on @0 IN THE ENCODED BYTES` and the same for slot 2, with
  `md decode` reporting `path_decl Divergent [m/48'/0'/2'/2' m/48'/0'/1'/2' m/48'/0'/0'/2']`
  against a tuple of `[0', 1', 2']`.

The distinct-account fixture is real and the palindrome self-guard is real. Restored;
`git status --porcelain` empty and `diff` against the pre-mutation copy identical.

### P-4 The flatten control fires, and the "obvious control" is confirmed insufficient ✅

Mutated `policyTemplate` to splice `origins[0]` into every placeholder.

- **Live** → `THE ORIGINS WERE FLATTENED. Encoding the SAME three xpubs under the divergent
  template and under a template with every slot at "m/48h/0h/0h/2h" produced byte-identical
  md1`, plus `md decode reports path_decl tag "Shared" ... want "Divergent"`.
- **The reported empirical claim is true**: under the same mutation the *obvious* control
  (`sameChunks(divMD, uniMD)` — same masters at uniform origins) **did not fire**, because
  each slot's xpub is derived at its own account and carries it regardless of the template.
  The committed control — same three xpubs, divergent template vs flattened template —
  is the one that isolates the template, and it is the one that dies.
- **Untagged** also FAILs (the mapping assertion), so the flatten specifically has an
  untagged backstop. The xpub-permutation defect does not — see I-1.

### P-6 Both late-found subtleties confirmed against the pinned `md 0.13.0` ✅

Invoked `~/.cargo/bin/md` directly with two real account xpubs from `~/.cargo/bin/ms`:

| invocation | result |
| --- | --- |
| `@0/48h/0h/0h/2h/<0;1>/*` inline | exit 1, `md: template parse error: @0: derivation steps after the multipath group are not representable in md1; the multipath ` `<…>` ` must be the final derivation step before the wildcard` |
| `@0/48'/0'/0'/2'/<0;1>/*` inline | exit 0, `chunk-set-id: 0xe50f6` + chunks |
| `--path m/48h/0h/0h/2h` (flag form) | exit 0 |

So the `h`→`'` normalisation is mandatory, the diagnostic does misattribute the failure to
the multipath, and `--path` does accept both notations. Removing the normalisation is
fail-closed (md refuses), not a silent wrong wallet.

`pathRe`'s optional script_type level is likewise handled correctly: `inlineOrigin` routes
through `templateForOrigin` and returns
`origin "m/48h/0h/0h" is BIP-48 but names no script_type level, so it will not be spliced
into a policy template`. Mixed-notation and whitespace-padded origins normalise correctly
(`m/48h/0'/0h/2'` → `48'/0'/0'/2'`); `M/48h/…` is refused.

### P-3 The tag split is enumerated and the allowlist is currently complete ✅ (mechanism: see I-1)

Three files carry `//go:build oraclelive`: `oracle/live_test.go`,
`gui/multisig_build_oracle_live_test.go`, `sysw/vendored_vectors_live_test.go`. They hold
exactly **7** `func Test`, and **all 7** appear in the `-run` allowlist. Verified by
extracting both lists mechanically.

What lives only behind the tag, and what holds untagged:

| behind the tag | untagged substitute |
| --- | --- |
| `TestLiveDerivationReproducesEveryCommittedExpectation` | none — the untagged arm compares two files the mint wrote together; the primary is reached only at mint time in `cmd/gaterecord` |
| `TestRealPinsResolveTheInstalledOracles` | none |
| `TestPinsAreCurrentWithTheirPrimaries` | none (needs sibling checkouts; correctly tagged) |
| `TestBuiltPolicyDerivationMatchesTheS2Golden` | `TestPolicyTemplateEncodesTheDevicesOwnS2Wallet` — a string *relation*, not bytes |
| `TestBuiltPolicyDerivesDivergentOrigins` | `TestPolicyTemplateMapsSlotIOriginToPlaceholderI` — the string builder only, never md's encoder |
| `TestAssembledMd1MatchesThePrimaryByteForByte` | `TestAssembledMd1MatchesTheCommittedGolden` |
| `TestVendoredVectorsAreInSyncWithThePrimary` | `TestConformance` + provenance-pin test |

CI does build the tagged files on every push (`test.yml:68`), so they cannot rot uncompiled.
I confirmed that step is exit 0 on the restored tree.

### P-5 The pin bump and re-anchor are honest in every particular ✅

| claim | measured |
| --- | --- |
| `d49d5c09` is the target of annotated tag `ms-cli-v0.16.0` | `cat-file -t` → `tag`; `rev-parse ms-cli-v0.16.0^{}` → `d49d5c099bab89a1738f0d0c3df9306b354d62c3` ✅ |
| one CI-only commit behind master `6fdfd36` | `rev-list --count` → `1`; that commit is `ci: retire the mlock.rs fmt exemption` ✅ |
| newest release tag | `git tag -l 'ms-cli-v*' --sort=v:refname \| tail -1` → `ms-cli-v0.16.0`; pin == newest, no drift ✅ |
| binary sha256 `9727689c…02e5` | `sha256sum ~/.cargo/bin/ms` matches; md and mk also match their pins ✅ |
| `checkout_clean_when_recorded: false` is honest | `/scratch/code/shibboleth/mnemonic-secret` → 3 untracked files, on `master`, not at the tag. The flag understates, exactly as the comment says ✅ |
| the delta does not touch `crates/ms-codec/src` | `diff --name-only ddfa4970..d49d5c09 \| grep -c crates/ms-codec/src` → **0** ✅ (see N-2 for the one incompleteness) |
| only provenance moved in S0's artifacts | machine-diffed with `json.load`, not by eye: **record** — only `oracles` and `recorded_at` differ; `walk`, `inputs`, `payload`, `stage` byte-equal. **expect** — only `derivation` and `note` differ; **`artifacts` equal on every field**, `derivation.args` equal, 6 artifacts. And `record.walk.census.strings == [a.string for a in expect.artifacts]`, in order ✅ |
| the walk and inputs files are untouched | sha256 vs `80d0c5d`: `S0-trace-a.walk.json` and `S0-trace-a.inputs.json` **unchanged**; `gui/testdata/s2_md1_golden.expect.json` **unchanged** ✅ |
| nothing still expects ms 0.15.0 | grep over `*.go/*.json/*.md/*.sh/*.yml`: one hit, `oracle/expect.go:38`, a true historical statement ("the templates were added upstream first (ms-cli 0.15.0)"), not a stale expectation ✅ |

### P-7 Dropping `Origin` from md1 artifacts loses nothing a later stage needs ✅

Grepped every consumer of `oracle.Artifact.Origin` in the tree: **all of them are in
`oracle/live_test.go`** — the re-derivation equality check (`:185`) and Trace B's
two-cards-same-master-different-path assertion (`:883`), which reads **mk1** artifacts.
Nothing untagged reads it at all. S2's committed golden already carries neither `origin` nor
`fingerprint` on its md1 artifacts (verified: 6 artifacts, all kind `md1`, no `origin` key,
no `fingerprint` key), so the change converges on the primary's own recorded shape. The
per-slot origins remain available three ways: in the md1 `Label` (the full policy template,
strictly more than the single path it replaced), in `record.inputs.origins`, and in the
encoded bytes themselves via `md decode --json`. **The judgement call is correct.**

### P-8 The device-facing citations resolve ✅

Not asked for, but load-bearing and ungated, so I ran them:

- `ArtifactKindsFor`'s order is cited to `gui/multisig_engrave.go:11-35`. Read it:
  `multisigEngraveCards` appends `cardMS1` first when `full`, then `cardMK1`, then
  `cardMD1`; watch-only omits the ms1. Matches `{"ms1","mk1","md1"}` / `{"mk1","md1"}`
  exactly.
- `--force-chunked` is cited to `gui/multisig_build_oracle_test.go:104-107` ("a 3-key policy
  is 335 data symbols and the regular code caps a single string at 80"). The comment is
  there, at :105-107.
- `s2WantTemplate` mirrored at `oracle/expect_test.go:721` is byte-identical to
  `gui/multisig_build_oracle_test.go:87`.
- The device passes `--path` and no `--policy-id-fingerprint`; the oracle passes
  `--policy-id-fingerprint` and no `--path`. That these produce identical md1 chunks is
  proved live and I re-ran it: `TestBuiltPolicyDerivationMatchesTheS2Golden` PASS,
  `built-policy-full: 1 ms1 + 2 mk1 + 6 md1 = 9 artifact(s); md1 byte-identical to S2's
  committed golden`.

### P-9 `CompareCensus` has no vacuous path ✅

Read and exercised: unequal lengths refuse; a one-character flip refuses naming the plate
and printing both strings in full; a short census refuses; a reorder refuses; `nil,nil`
refuses via the explicit `n == 0` clause; `len(want)==0, len(got)==3` refuses on length.
The three mutation proofs plus `TestCompareCensusCatchesAMultiKindReorder` all execute
untagged and all pass on the restored tree.

---

## Restoration

Every mutation was `edit → run → git checkout --`. Final state verified:

```
$ git -C /scratch/code/shibboleth/seedhammer-s5 status --porcelain
(empty)
$ git log --oneline -1
5ed87c7 S5.0c: derive md1 with INLINE per-@N origins, so a divergent policy is derivable
$ diff <pre-mutation copy of oracle/expect.go> oracle/expect.go
(identical)
```

The planted `S5-fake.*` files were removed and `git status --porcelain` confirmed empty
afterwards. `/scratch/code/shibboleth/seedhammer` was not touched.

## Verdict

**0 Critical / 2 Important — the loop does not close.** The derivation itself survived every
mutation I could aim at it: the origin→placeholder mapping dies under reversal in two
independent places, the flatten control fires and the weaker control it replaced is
confirmed insufficient, the pin bump is honest in every particular I could measure, and the
md1 `Origin` drop is right. What blocks is the *instrumentation*: I-1 (the tag allowlist is
a comment, and an empty filter reports PASS) and I-2 (`Kind` is an unbound label, so the
shape rule's own stated guarantee is defeated by a one-word edit). Both have fixes of a few
lines, and neither needs a re-derivation or a re-walk.
