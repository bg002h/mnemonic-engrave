# Composer Stage 2 (fork codec) — independent adversarial execution review, r0

**Reviewer:** independent agent; did not write the implementation, the plan, the spec or the fold.
**Under review:** `composer-s2` at `489d52eba1ca94475e5fa51f134be0b07484ea76`, 8 commits over
seedhammer fork `169073c` (7 implementer commits + the controller's fold `489d52e`).
152 files, 8,435 insertions, 65 deletions.
**Oracle:** descriptor-mnemonic `66bdf2f47e7fc703d5fb09120122b3e98cab5528` (tree clean).
**Toolchain:** go1.26.7, `CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off`, default `-mod=readonly`.
**Read-only:** every mutation and every probe file reverted/deleted; `git status --porcelain`
empty at the end in `wt-composer-s2`, `descriptor-mnemonic`, `mnemonic-engrave` and the main
`seedhammer` checkout; the temporary `169073c` worktree was removed.

## Counts

**1 Critical / 1 Important / 4 Minor / 1 Nit.**

Both blocking findings are in `sysw/composer_records.go` (Task 6, commit `7ac35dc`) — the
`key:` record's origin-path grammar diverges from the host in **both** directions, against a
NORMATIVE acceptance item (spec §12 item 8, "classifies identically on the host and on the
device"). The 45-row vendored fixture structurally cannot see either: it has no row that varies a
path component's numeric range or sign (all 19 `key:` rows vary length, depth, termination or
hex). Everything else the brief asked me to attack came back clean, including 224 Go-vs-Rust
compose cross-checks and six mutations.

---

## Lens 1 — counterexamples against the builder that the corpus cannot see

**Method.** I built two probes driven from one text grammar so both sides run the same input:
a Go test calling `md.Compose` / `md.ComposeWith` and printing `Composed.Chunks()`, and a Rust
integration test calling `md_codec::compose::{compose,compose_with}` and printing
`md_codec::chunk::split(&c.descriptor)`. **Control first:** on the corpus row
`wsh|2of3|1of1,older=26280` both printed the identical single chunk
`md1ftqr5qq9q6tvyyy5jmpprjjtvyy49ykcgfw2sqrqnqvzyxfnf0wqqqrx4qqlfju9m03sxgg2`, which is also
what `md encode --force-chunked` prints for the vendored `.template`. Harness validated before
use.

### The five the brief named — all identical

| # | case | result |
| --- | --- | --- |
| 1 | `tr` 8 paths of mixed kinds, internal key listed 4th: `tr\|1of1,older=10\|2of3,after=1000000\|3of4,older=100u\|1of1\|2of2,sha256=a8…\|1of1,after=1893456000t\|4of5\|1of1,older=65535` | 18 slots, `ik=3` on both, **5 chunks byte-identical** |
| 2 | declared origin equal to a default at account 0: `wsh\|1of1\|1of1,older=100` with slot 1 declared `48'/0'/0'/2'` | identical chunk; `md inspect` confirms the unseated slot 0 got account **1** — the lowest-free rule skips the taken default identically |
| 3 | `sh-wsh\|1of9` | 9 slots, **2 chunks identical** |
| 4 | hash+lock keyless path under wsh with a bare-multi head, with a lock kind the corpus does not carry: `wsh\|2of3\|keyless,sha256=ffee…,older=300u` | identical; both mark `KeylessPath(1)` / `{Kind:0 Path:1}` |
| 5 | two declared slots, same origin, distinct fingerprints, plus unseated slots: `wsh\|2of3\|1of1,older=100` with `0=48'/0'/0'/2':73c5da0a;1=48'/0'/0'/2':deadbeef` | identical 2-chunk set; the pathDecl collapses the same way. With the second fingerprint removed, **both** refuse: Go `ErrComposeIndistinguishableSlots: slots @0 and @1`, Rust `IndistinguishableSlots { a: 0, b: 1 }` |

Also hand-run and identical (chunks or refusal kind): `wsh|keyless,…|2of3` (keyless *first*),
`wsh|9of9|9of9|9of9|6of6` (33 slots → `TooManySlots{got:33}`), `sh|2of2|1of1,older=100` and
`sh-wsh|2of3,unsorted` and `sh|1of1` (→ `LegacyWrapperShape`), `after=499999999` /
`after=500000000t` (the §4c band boundary, both sides), `older=65535u`,
`tr|keyless,…` (→ `KeylessUnderTr`), `wsh|3of2` (→ `BadThreshold`), 9 paths (→ `TooManyPaths`),
`tr|1of1,sha256=…|1of1` (a hashed single is not extractable: `ik=1` on both),
`tr|2of2|1of1|1of1`, `tr|4of4`×8 (32 slots), `tr|2of2,unsorted` and `wsh|2of3,unsorted`
(both mark `UnsortedKeys(0)`), `tr|1of1|2of3,unsorted|1of1` (both mark **nothing** — the primary
prints "`unsorted` has no effect here" for that position and Go agrees byte-for-byte),
`tr|1of1|1of1,older=100|2of2` with two declared origins including a short unhardened one.

### Randomized battery

199 further path lists (seeded, wrappers × 1–8 paths × all five lock encodings × unsorted ×
sha256 × keyless × 0–4 declared origins with and without fingerprints), run through both
implementations and diffed:

```
go cases: 199   rs cases: 199   same order+keys: True
composed-and-identical: 136   refused-by-both: 63   MISMATCHES: 0
```

The 63 refusals correspond 1:1 by kind (42 `LegacyWrapperShape`, 21 `IndistinguishableSlots`).

**Total: ~224 Go-vs-Rust compose comparisons, zero byte, chunk, slot-map, internal-key,
experimental-mark or refusal divergence. This lens found nothing.**

One type-level (not behavioural) note, no finding: Rust's `Lock::OlderBlocks(u16)` makes an
out-of-range block count unrepresentable, while Go's `Lock{Value uint32}` admits it and refuses
at `Lock.operand()`. Both refuse; only the moment differs. (`ValidatePathList`'s `anyKeyed`
guard also makes `resolveOrigins`'s `origins[0]` unreachable at n = 0, so the empty-slot panic I
went looking for does not exist.)

---

## Lens 2 — mutation-testing the tests

`mut.sh` copies the file, applies a `sed`, **asserts the file actually changed**, runs the named
package, then restores from the copy and asserts `git diff --quiet` on that path. Every
mutation below is of the `if false &&` / opcode-substitution kind, so a *dead* line could not
change behaviour — a killed mutation is itself the proof the line ran.

| # | target | mutation | killed by |
| --- | --- | --- | --- |
| M1 | `lockFromWire` bit-22 branch (`md/compose.go:144`) | `if false && operand&sequenceTypeFlag != 0` | `TestPolicyShapeSplitsAlternativesIntoBranches/keyed_compose_wsh_single_head_or_i` **and** `TestLockCheckIsTheDeviceSideRangeGate` |
| M2a | `resolveOrigins` pairwise check (`md/compose.go:534`) | `for b := a + 2` (skip adjacent pairs) | `TestComposeRefusesWhatThePrimaryRefuses` |
| M2b | same | `if false && sameOrigin(...)` | `TestComposeRefusesWhatThePrimaryRefuses` |
| M3 | the `pk_h` arm's `opHASH160` (`md/script_emit.go:201`) | `opHASH160` → `opSHA256` | `TestPkhWitnessScriptsReproduceRustsAddresses` (all 5 sub-tests) |
| M3b | same | same | `gui` `TestThePkhTapLeafGapIsCLOSED` (the fold's new positive test) |
| M4 | `ParseKeyRecord`'s depth check (`sysw/composer_records.go:234`) | `if depth != 3 && depth != 4` → `if false` | `TestComposerRecordsClassifyExactlyAsTheHost` |
| M5 | **the fold's own arm** (`md/script_emit.go:241`) | `case opNUMEQUAL:` → `case 0xfe:` | `TestVerifyWrappedMultiAFoldsIntoNumEqualVerify` |
| M5b | same | same | `gui` `TestEveryKeyedVectorReachesAnAddress/keyed_compose_tr_nums_three_leaves` |

All six behaviours are genuinely covered, and both of the fold's claims about its own pins
("pinned at byte level by `TestVerifyWrappedMultiAFoldsIntoNumEqualVerify` and at address level
by the existing gui gate") are true. **This lens found nothing.**

---

## Lens 3 — what did the diff make false elsewhere

### The consent screen text for the shipped `or_*` cards — measured, not reasoned

`policySummaryLines` output, run at `169073c` and at `489d52e` over every non-compose vector.
Three shipped cards changed, and only those three:

| vector | before (`169073c`) | after (`489d52e`) |
| --- | --- | --- |
| `keyed_wsh_or_d_degrading` | `Spend path: 1` / `1: 3 key(s), custom +timelock` | `Spend paths: 2` / `1: 2-of-2` / `2: 1 key(s), custom +timelock` |
| `keyed_wsh_or_b` | `Spend path: 1` / `1: 2 key(s), custom` | `Spend paths: 2` / `1: 1 key(s), custom` / `2: 1 key(s), custom` |
| `keyed_wsh_timelock_hashlock` | `Spend path: 1` / `1: 3 key(s), custom +timelock +hashlock` | `Spend paths: 2` / `1: 3 key(s), custom +timelock +hashlock` / `2: 2 key(s), custom +timelock` |

Checked against the templates: `or_d(multi(2,@0,@1), and_v(v:older(65535),pk(@2)))` really is
2-of-2 **or** one key after a timelock; `or_b(pk,s:pk)` really is either key alone — the old
"1 path, 2 keys" read as *both* keys, which was the misleading one. The text reads **better**,
not worse, and `TestPolicyShapeSplitsTheShippedOrCards` pins it. The conservative
"`N key(s), custom`" rendering for a threshold nested under `and_v` is unchanged and still
guarded by the pre-existing `TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee`. **No finding.**

### Documentation and comments

- I grepped the fork for the specific stale-claim shapes the brief named — Branch "one per wsh
  script", "no pk_h", "three classes", "PolicyShape counts a multi-path wsh as one". The only
  live hits are the *new* doc comments in `md/policy_shape.go:145,169-172`, which say the right
  thing. `sysw/record.go`'s new `ClassKey/ClassHash/ClassNow` block is accurate (I verified
  `IsComposerRecord` is prefix-matched before the sniffers, at `sysw/record.go:135`).
- `gui/multisig_build_slots.go`'s new tie comment is true (`ComposeWrapper.ScriptType()` returns
  1/2/3 exactly as `multisigScriptTypeComponent` does, plus tr = 3), and
  `gui/composer_origin_test.go` enforces it.
- `mk/compose_stubs.go`'s claim "Encode's stub_count bound (<= 255) is enforced by Encode" is
  true: `mk/encode.go:79` `if len(card.Stubs) > 0xff`.
- Findings M-1..M-4 and N-1 below.

---

## Lens 4 — whole-repository gates as CI runs them

All re-run by me, not taken from the report or the fold message.

```
$ go test -count=1 -timeout 20m <all 72 packages except ./gui/>
72/72 accounted for: 45 ok, 27 "no test files", 0 FAIL, EXIT=0

$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1059 tests ran across 24 shards      (wall 35s, EXIT=0)

$ go test -count=1 -timeout 20m ./gui/          # CI's ACTUAL runner, unsharded
ok  	seedhammer.com/gui	138.600s               EXIT=0

$ scripts/test-32bit.sh
GOARCH=386 test: exit 0 ; GOARCH=arm build: exit 0    32bit EXIT=0

$ go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/     EXIT=0
$ GOOS=js GOARCH=wasm go vet ./cmd/emu/                            EXIT=0
$ git diff --name-only 169073c..HEAD -- '*.go' | xargs gofmt -l    (empty)
```

I ran the gui package **unsharded** deliberately: sharding is process-per-shard, which can mask
shared-state bleed the way nextest's isolation once did in this constellation. It is green
either way.

`gofmt -l md/ mk/ sysw/ gui/ scripts/` still prints `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` — **identical at
`169073c`**, untouched by this branch, and no CI gofmt job exists. The fold's phrasing "gofmt
clean on the plan's files" is precise and true.

`go vet ./gui/` prints exactly two findings
(`testing.ArtifactDir requires go1.26 or later (file is go1.25)` at
`gui/freetext_sizeproof_golden_test.go:111` and `gui/transaction_golden_test.go:104`) — byte-identical
at `169073c`. The fold's claim holds.

### Firmware (nix available; tinygo only inside the dev shell, as the implementer said)

`nix develop --command tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`, built by me at both revisions:

| revision | flash | RAM |
| --- | --- | --- |
| `169073c` baseline | **1,503,652 B** | 62,592 B |
| `fa52bb3` (implementer's tip) | 1,506,820 B *(their measurement)* | 62,592 B |
| `489d52e` (with the fold) | **1,506,900 B** | **62,592 B** |

The baseline reproduces the plan's number exactly, so the implementer's `+3,168 B` is confirmed
for its revision. **The tip's delta is `+3,248 B` (+0.216 %), RAM unmoved** — the fold's
`opNUMEQUALVERIFY` arm costs 80 B. Their explanation of the non-zero delta (the `pk_h` arm pulls
`btcsuite/btcd/address/v2` into `emitFragment`, which the shipped GUI already calls; and
`sysw.Classify` now reaches `hdkeychain` through `classifyComposer`) is consistent with what I
measured. `controller.elf` was deleted afterwards; the worktree is clean.

**This lens found nothing.**

---

## Lens 5 — provenance honesty

- `md/testdata/compose_vectors.provenance.json`: 126 pinned files. All 126 present, all 126
  sha256-match the pin, and — the check that matters — all 126 are byte-identical to
  `git -C descriptor-mnemonic show 66bdf2f4:crates/md-codec/tests/vectors/<name>`. Zero absent,
  zero differing. `commit` is descriptor-mnemonic's real clean HEAD.
- `sysw/testdata/record_class_vectors.provenance.json`: `sha256 eed6b177…464e` matches the
  vendored file **and** mnemonic-engrave's `crates/me-cli/testdata/record_class_vectors.json`
  (`diff` identical, 45 rows). `file_commit 5720e3c0747f72e7c6a6225b2993db9d0d40d24e` is exactly
  what `git log -1 --format=%H -- crates/me-cli/testdata/record_class_vectors.json` returns
  today, and `commit 38e3ed13…` is an ancestor of engrave HEAD (`b07eda0d`).
- The **fold's own new fixture** `gap_wsh_andor` carries no pin, so I re-ran the exact command
  `md/testdata/README.md` documents (`md encode --force-chunked "<template>" --key @0=… --key @1=…
  --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a`, journey cosigners 0 and 1) at
  descriptor-mnemonic `66bdf2f4`: the five md1 lines are **byte-identical** to
  `gap_wsh_andor.phrase.txt`. The README's regeneration recipe is honest and reproducible.

**This lens found nothing.**

---

## Extra lens the brief did not name — differential fuzzing of the new `sysw` record classifier

Spec §12 item 8 makes host/device classification lockstep a NORMATIVE acceptance item, and
`sysw.Classify` is reachable from the shipped scan door today. The 45-row fixture proves lockstep
on 45 inputs; I asked what happens on inputs it does not contain. I built a probe on each side
(`mnemonic_engrave::sysw::classify` in Rust, `sysw.Classify` in Go), fed both the same 49
hand-built records, and diffed. **47 agreed; 2 did not.**

### C-1 (Critical) — a `key:` record with an unhardened path component ≥ 2³¹ is accepted by the device, refused by the host, and its origin is silently re-read as a *hardened* path

**File:** `sysw/composer_records.go:167-190` (`parseOriginPath`), via `bip32/bip32.go:69`
(`ParsePathElement`). New code on this branch (commit `7ac35dc`).

**Reproduction.** Record body = hex of `[73c5da0a/2147483648/0'/0']xpub6CKZtUaK1YHpQbg6CLaGRmsMKLQB1iKzsvmxtyHD6X7gzLqCB2VNZYd1XCxrccQnE8hhDxtYbR1Sakkvisy2J4CcTxWeeGjmkasCoNS9vZm` (the fixture's own depth-3 xpub):

```
host  mnemonic_engrave::sysw::classify(...)  ->  Unknown     (DerivationPath::from_str rejects
                                                              an unhardened index >= 2^31)
device sysw.Classify(...)                    ->  ClassKey
device ParseKeyRecord(...).Origin.String()   ->  m/0h/0h/0h      <-- the record says 2147483648/0'/0'
```

**Why.** `bip32.ParsePathElement` uses the fork's in-band `+HardenedKeyStart` convention: with no
`'`/`h` suffix `offset` is 0, so the only range guard (`iu32+offset < iu32`) never fires, and the
literal `2147483648` lands on the same `uint32` that spells hardened index 0. The
component-count and last-component checks then pass, so the record is admitted under an origin
that is not the one written on it. The host delegates to `bitcoin::bip32::DerivationPath::from_str`,
whose `ChildNumber::from_normal_idx` refuses the value outright.

**Why nothing caught it.** All 19 `key:` rows in `record_class_vectors.json` vary origin *length*,
xpub *depth*, bracket termination or hex case; **no row varies a component's numeric range**. The
gate `TestComposerRecordsClassifyExactlyAsTheHost` cannot fail on this class, and `parseOriginPath`'s
own doc comment ("the host's `DerivationPath::from_str` … each ASCII digits with an optional `'`
or `h` hardening marker") asserts a grammar it does not implement.

**Severity.** Two things are wrong, and either alone would block: (a) §12 item 8's lockstep
guarantee is unmet, and (b) a shipped parser returns an origin that differs from the record's
text — and the origin is precisely what makes a composer cosigner restorable, so a Stage-3
consumer of `KeyRecord.Origin` would seat and display the wrong path. Reachability is **bounded**
and I want that stated plainly: `me sysw pack` refuses an unclassifiable record by index
(§6a; `pack_with`/`admit_check`), so such a record cannot ride a host-produced payload — it needs
a payload built by other tooling.

**Rust-primary check (mandatory, and done):** the primary **refuses** this input, so this is a
Go-only convergence defect. Fix it in Go; no Rust change and no test vector are owed first.

**Hypothesis (not authoritative — reproduce before adopting).** Range-check the component in
`parseOriginPath` before calling `bip32.ParsePathElement`: reject a component whose numeric part
is ≥ 2³¹, hardened or not. The same laxity lives in the shared helper and is reached by
`bip380/bip380.go:482,486,500` (pre-existing, outside this diff) — worth running the class
tree-wide rather than patching one call site.

### I-1 (Important) — a `key:` record with a `+`-signed path component is accepted by the host and refused by the device

**File:** same, `sysw/composer_records.go:176-183`.

**Reproduction.** Three shapes, all `Key` on the host and `Unknown` on the device:
`[73c5da0a/+48'/0'/0']xpub…`, `[73c5da0a/48'/+0'/0']xpub…`, `[73c5da0a/48'/0'/+0']xpub…`.
Rust's `ChildNumber::from_str` parses through `u32::from_str`, which accepts a leading `+`; the
Go port's explicit ASCII-digit loop rejects it.

**Why it matters.** Same unmet §12 item 8 guarantee, opposite direction: a record the host packs
successfully goes **inert** on the device, and the operator's only signal is "Keys loaded"
silently one lower plus the generic "not understood" count. Nothing tells them which record, or why.

**Rust-primary check:** here the **primary is the laxer side**, and `+0'` is not BIP-380
key-origin notation either (§6a requires the record to parse *as* BIP-380 notation). So unlike
C-1 this one is not a free Go fix: it is a choice between tightening the host first (Rust-primary
rule: land it in Rust with a vector, then converge Go) and loosening Go. **I am not making that
call** — it is a spec question about which grammar §6a means, and it needs the plan or the
operator.

**Hypothesis.** Whichever direction is chosen, add fixture rows for both `+` and the ≥ 2³¹ range so
the corpus can see the class at all. A grammar row set that varies only *shape* and never *value
range or sign* will keep passing through this defect class.

*(For the record, these two are the whole delta. `-1`, leading/trailing slash, leading/trailing
space, empty element, `H` marker, U+2019, `0''`, `4294967295'`, leading zeros, `h`-spelling,
uppercase fingerprint, uppercase body hex, odd-length body, non-UTF-8 body, and every `hash:` and
`now:` edge I tried — `0`, `2147483648`, 11-digit seconds, 10-digit height, `1,`, `1,2,3`, `+1`,
`1.0`, ` 1`, full-width `１`, `1,500000000`, `2147483647,499999999` — agree exactly.)*

---

## Review of the controller's fold `489d52e` specifically

Held to the same standard as the implementer's seven, as instructed.

**F-1 (`v:multi_a` → `OP_NUMEQUALVERIFY`) is correct, and correct in general.** The new
`case opNUMEQUAL` in `md/script_emit.go:241` makes the fork's VERIFY fold match
`bitcoin::blockdata::script::Builder::push_verify`, whose table is exactly
`{EQUAL, NUMEQUAL, CHECKSIG, CHECKMULTISIG}` — Go now covers all four. I checked the one way a
byte-level fold can go wrong (the switch reads the last *byte*, not the last *opcode*): every
B-type miniscript fragment this emitter builds terminates in an opcode, never in a data push, so
the byte read is the opcode. M5/M5b prove the arm is load-bearing at both byte and address level,
and the wrong address it fixes is real — `keyed_compose_tr_nums_three_leaves` is the repo's only
verify-wrapped `multi_a`. The change cannot have moved a previously-shipped address: the whole gui
suite was green at `169073c` against Rust-vendored addresses, so no shipped vector emitted
`NUMEQUAL VERIFY`. The commit message's Rust-primary reasoning ("the primary emits through
rust-miniscript, whose verify fold covers NUMEQUAL; Go-only convergence fix") is right.

**F-3's two test edits are sound in substance.** `TestThePkhTapLeafGapIsCLOSED` is a
structure-for-structure copy of the shipped `TestTheTimelockedTapLeafGapIsCLOSED` (same direct
`complexAddressSource` call, same both-chains × indices 0..1 loop against the vendored
conformance addresses), so it follows the repo's own precedent rather than inventing one. And I
verified by construction that the re-aimed consent test really does exercise the derive **probe**:
for `gap_wsh_andor`, `ExpandWalletPolicyChunks` yields 2 keys with `XpubPresent=true`,
`expandedKeysToBip380` succeeds, `expandedToDescriptor` returns `expandUnsupported`, and
`EmitWitnessScriptChunks` is what fails, with `md.ErrScriptUnsupported`. The rendered consent is:

```
Policy-ID: 211b8e652c2bf9fe7c055e0998f18723
Complex policy - cannot display safely.
Keys: 2
@0 73c5da0a m/48h/0h/0h/2h <0;1>/*
@1 73c5da0a m/48h/0h/1h/2h <0;1>/*

This device can't derive
addresses for this policy.
```

I also confirmed the gap-fixture states moved exactly as claimed and no further:
`gap_tr_leaf_pkh` NOT derivable at `169073c` → derivable at the tip; `gap_wsh_andor` not
derivable; `keyless_tr_with_leaf` not derivable; `gap_tr_leaf_and_v` derivable at **both**
revisions (see N-1). Four Minors follow.

### M-1 (Minor) — the fold deleted the "it must be the PROBE that refuses" guards and did not carry them to the fixture that inherited the job

`gui/policy_address_test.go` (before `489d52e`) asserted three preconditions on `gap_tr_leaf_pkh`
— real xpubs, derivable use-sites, and `expandedToDescriptor == expandUnsupported` — with the
comment *"It has to be the PROBE that refuses, not an earlier guard."* The fold removed all three
and moved the refusal-pinning job to `gap_wsh_andor`, which inherits none of them.
`TestWalletPolicyConsentNeverHidesTheAbsenceOfAddresses` does rule out two of the three earlier
guards indirectly (it asserts the text contains neither `Keyless` nor `no keys`), but **nothing**
rules out `expandedKeysToBip380` returning `!ok` — that path produces the identical
"This device can't derive" wording. I verified today's behaviour is the emitter (above), so this
is a coverage gap, not a live defect: if the use-site guard ever started rejecting this fixture,
the test would keep passing while testing a different layer.
**Hypothesis:** re-add the three `t.Fatal` preconditions inside `consentText`'s caller for
`gap_wsh_andor`, or assert `errors.Is(err, md.ErrScriptUnsupported)` from
`md.EmitWitnessScriptChunks` directly in that test.

### M-2 (Minor) — the fold's new comment describes the wrong fixture

`gui/policy_address_test.go:380-382` (new in `489d52e`): *"`TestWalletPolicyConsentNeverHidesThe
AbsenceOfAddresses` covers the 'cannot derive' consent wording **with a hand-built shape**
instead."* It does not: the same commit created the vendored fixture `gap_wsh_andor` and pointed
the test at it. A reader who greps for a hand-built shape will not find one.
**Hypothesis:** say "with the `gap_wsh_andor` fixture instead".

### M-3 (Minor) — `gap_wsh_andor` cannot become "the next positive one, exactly as its two predecessors did"

`md/testdata/README.md`'s new section promises that, and in the same breath records
*"No `.conformance.json`: nothing asserts its addresses."* Both predecessors could be converted
to positive tests **only** because Rust's addresses were vendored beside them — that is the
stated reason `gap_tr_leaf_and_v`'s section gives for carrying one. When an `andor` arm lands,
this fixture will correctly fail the consent test, and whoever fixes it will have no ground truth
to be right against and will have to re-vendor. The promise overstates the parallel.
**Hypothesis:** either vendor `gap_wsh_andor.conformance.json` from the primary now (it is one
`md address` run at `66bdf2f4`), or reword the last sentence to say the fixture will need
conformance addresses generated at that point.

### M-4 (Minor) — `md.ComposerStubs` distinguishes `nil` from empty

`md/compose_stubs.go:19` gates on `keyedChunks != nil`, so a caller passing a non-nil empty
`[]string{}` reaches `FormAwareStubChunks` with no chunks and gets that function's error rather
than the "no keyed policy yet" behaviour the doc comment describes. There are no callers until
Stage 3, so this is a latent trap, not a live defect.
**Hypothesis:** `if len(keyedChunks) > 0`.

### N-1 (Nit, PRE-EXISTING) — the README still describes `gap_tr_leaf_and_v` as an open gap

`md/testdata/README.md:126-148` says *"this port's tap-leaf emitter describes pk / multi_a /
sortedmulti_a only, so it refuses … when the emitter grows `and_v`/`older` leaves the test FAILS
saying the gap is closed."* I measured it at `169073c` **and** at the tip: the fixture derives
`bc1pkt2t64zvw4pzfvm5qn4yydkkt6teexhjp7huqfyjth8wh40z3l7qp7aqla` at both, and the live test is
already named `TestTheTimelockedTapLeafGapIsCLOSED`. The prose went stale when F-214 landed, not
on this branch — but the fold's own new section sends the reader to those two predecessors as the
model, so it is worth one line while the file is open.

---

## What I did NOT find, stated so the next reviewer does not redo it

- No divergence from the Rust primary anywhere in `md.Compose`/`ComposeWith` across 224
  constructed cases, including all five the brief named, every §4c band edge, every refusal arm,
  the 32-slot ceiling, and declared-origin collapse with and without fingerprints.
- No false-PASS in any of the six behaviours I mutated, including both of the fold's own new pins.
- No stale claim created by the diff in `md/policy_shape.go`, `sysw/record.go`,
  `gui/multisig_build_slots.go`, `mk/compose_stubs.go`, or the emitter comments.
- No provenance dishonesty: 126/126 compose files and the 45-row fixture verified against the
  real upstream objects, and the fold's unpinned new fixture reproduced from its own recipe.
- No gate red, on any runner, including the gui package run the way CI actually runs it.
- The three `gofmt -l` hits and the two `go vet ./gui/` findings are byte-identical at `169073c`.

## Commands, for reproduction

Probes were temporary and are deleted; recreating them is the only step that is not a
copy-paste. Everything else:

```sh
W=/scratch/code/shibboleth/wt-composer-s2
export PATH=/nix/store/i77g9dmcd399rmxk8688qfr4g2wzgk37-go-1.26.7/bin:$PATH
export CGO_ENABLED=0 GOTOOLCHAIN=local GOPROXY=off
cd $W && go test -count=1 -timeout 20m $(go list ./... | grep -v '/gui$')
/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24
go test -count=1 -timeout 20m ./gui/
scripts/test-32bit.sh
go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/
GOOS=js GOARCH=wasm go vet ./cmd/emu/
git diff --name-only 169073c..HEAD -- '*.go' | xargs gofmt -l
nix develop --command tinygo build -size short -target pico-plus2 \
  -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```

C-1 needs no probe at all — the record is a literal:

```sh
python3 -c 'print("key:"+("[73c5da0a/2147483648/0\x27/0\x27]xpub6CKZtUaK1YHpQbg6CLaGRmsMKLQB1iKzsvmxtyHD6X7gzLqCB2VNZYd1XCxrccQnE8hhDxtYbR1Sakkvisy2J4CcTxWeeGjmkasCoNS9vZm").encode().hex())'
# host:   mnemonic_engrave::sysw::classify(...) == Class::Unknown
# device: sysw.Classify(...)                    == ClassKey
#         sysw.ParseKeyRecord(...).Origin.String() == "m/0h/0h/0h"
```
