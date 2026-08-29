# IMPL-S2-P3 — the Go port: classify, consume, display (fork)

**Phase:** P3 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md` (GREEN, R0 closed r7).
**Implementer:** single agent, per the tight-implementation rule.
**Fork worktree:** `/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm`,
branch `s2/descriptor-arm`, base `0abbf81`, tip **`fe9475c`**.
**Engrave worktree:** `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2`,
branch `impl/descriptor-s2` @ `b9b7f42` — read only, except this report's commit.
**Nothing pushed anywhere. No main checkout touched.**

## What landed, per task

| task | fork sha | subject |
| --- | --- | --- |
| P3.1 | `252027f` | `sysw: the descriptor classifier arm, and it is the PREDICATE` |
| P3.2 | `cde7545` | `gui: Wallet Policy takes a Descriptor record, and the cell finally fires` |
| P3.3 | `29cb930` | `nonstandard: the seam sync, and the sampled column becomes a derived rule` |
| P3.4 | `fe9475c` | `bip380: F-426's ypubVer arm gets the tests it landed without` |

Diffstat `0abbf81..fe9475c`: 14 files, +1121 / −126, one new binary fixture
(509 bytes).

### P3.1 — the classifier arm

`sysw/descriptor.go` (new, 375 lines incl. comments) + one arm at the END of
`classifyConstellation`. The arm is a port of §5.2's predicate, composed as the
Rust primary composes `descriptor::host_admits`:

1. **parse** via `nonstandard.OutputDescriptor` — and the parse runs FIRST, which
   is a cost decision only (the predicate is a conjunction, so order is free);
2. the cascade's two **single-line-reachable narrowings**:
   - §4.3's five admitted versions as a **string-level** check over the record's
     own bytes, on both the descriptor-embedded and bare-key paths. It cannot be
     a conjunct: `bip380.Key` has no version field and `ParseExtendedKey`
     normalises the version away, and `0abbf81`'s `ypubVer` arm makes the parser
     itself accept `ypub`. Implemented as a scan for maximal base58 runs of
     ≥ 100 characters (base58check of 82 bytes is always 111–112 characters when
     the leading version byte is `0x02`/`0x04`), each decoded with
     `hdkeychain.NewKeyFromString` and its `Version()` checked.
   - §4.5's promotion ruling — `me` refuses `tpub` promotion entirely. Branch 4
     is detected **exactly**: `bip380.ParseKey(nil, record)` succeeding on the
     WHOLE record is that branch's condition, because a key expression carries
     no `": "` for branch 1, no `(` for branch 2 and is not JSON for branch 3.
3. §4.7's **conjuncts 1, 2, 3, 5, 6, 7, 8** over the parsed descriptor.
   Conjunct 4 lives in step 2 (the only place the answer still exists).
   Conjunct 1's `multi` arms have no counterpart and cannot: this parser cases
   `sortedmulti` alone, so a `multi` policy is a parse refusal.

`recover()` fails the arm closed to `ClassUnknown`. `Classify` runs over every
record of every loaded payload, so a panic there is a payload that will not
LOAD. It compiles under TinyGo `-gc precise -scheduler tasks` (see the gate).

**Named unit tests** (`sysw/descriptor_test.go`), all passing:

| test | input | verdict |
| --- | --- | --- |
| `TestShortFingerprintHeaderClassifiesUnknownWithoutPanicking` | `ab: xpub…` | `ClassUnknown`, no panic |
| `TestTitledZeroKeyDescriptorClassifiesUnknown` | `Name: my wallet` | `ClassUnknown` |
| `TestBareTpubClassifiesUnknown` | bare `tpub…` (+ bare-`xpub` control) | `ClassUnknown` / `ClassDescriptor` |
| `TestBareYpubClassifiesUnknown` | bare `ypub…` (rows.py's `SKYL`) | `ClassUnknown` |
| `TestFullOriginYpubDescriptorClassifiesUnknown` | `sh(wpkh([4bbaa801/49h/0h/0h]ypub…))` (+ xpub twin control) | `ClassUnknown` / `ClassDescriptor` |

Measured detail on the titled-zero-key case: in Go the **shape** conjunct
refuses it first (`Script` is the zero value, neither a single-key script nor a
multisig slot). The threshold conjunct also refuses it (0 keys, threshold 0) —
both are asserted, so the plan's "caught by conjunct 2" is true but is not the
first refusal. `Descriptor.Encode` panics on that shape, so admission refusing
it is what keeps it away from `DescriptorScreen`.

### P3.2 — the walletPolicy consumer, and the first execution of the cell

`walletPolicyFlow` gains a SECOND offer at the same door
(`else if body, ok := syswOffer(ctx, th, sysw.ClassDescriptor, …)`), following
`newInputFlow`'s shipped precedent and its stated rule (`syswOffer` takes one
class). An accepted record re-parses → `*bip380.Descriptor` → `descriptorFlow`
→ the existing `DescriptorScreen`, and the call RETURNS. The md1-card path is
byte-unchanged.

`gui/wallet_policy_descriptor_walk_test.go` is the **sim walk**, and it drives a
REAL packed S2 container:

```
me sysw pack --no-passphrase --as descriptor \
    --in <the seam file's formats-happy/bip380-sortedmulti-multipath input> \
    --out gui/testdata/s2_descriptor_payload.bin
```

built from `impl/descriptor-s2`'s own `cargo build --locked -p mnemonic-engrave`.
Fixture: **509 bytes**, sha256
`672d8d2c49b6c2004c38849c7b68b6dffa8629eb6bf9ac61f6ebc1e1657c58bb`, one public
record and no secret section; `me` reported wallet-id
`9e95257e60aacbb260129dac7b36d9f4` and digest `9c16 bfa9 bb3b ecd4 6c3c f20f
e48c 12a9`. The walk asserts the fixture's length and hash before using it,
opens it with the firmware's own `sysw.Open`, classifies with the real
`sysw.Classify`, takes the real `syswOffer`, and reaches
`DescriptorScreen.Draw` showing `Engrave Descriptor` / `2-of-3 multisig` /
`Segwit (P2WSH)` **without panicking**. Both tests pass; both are enumerated by
`go test -list` and therefore run inside the shard.

Oracle: `syswConsumers`' `walletPolicyFlow` `why` string updated. The entry is
keyed `file:fn`, so no registration was needed — **measured both ways**: the
reconciled site count is **9 before the change and 10 after**
(`git stash` on `gui/wallet_policy.go`, run, pop, run).

**Records this diff falsified, corrected in the same commit** (the
"a diff falsifies text it never touches" lens, run deliberately):

- `gui/chain_walk_test.go` and `gui/chain_class_walk_test.go` both asserted in
  prose that `ClassDescriptor` can never enter a payload, citing `rc=4`.
  Re-measured against the S2 `me`: a descriptor with no `--as` is now **rc=2**
  (§5.1's window) and `--as descriptor` is **rc=0**; an address is still
  **rc=4** with reworded text. Both headers rewritten with the measured values.
- `sysw/record.go`'s `Classify` doc said "DESCRIPTOR AND ADDRESS ARE
  DELIBERATELY ABSENT".
- `gui/sysw_admit.go` now names `progBundle`'s and `progMultisig`'s Descriptor
  cells as **declared and INERT** (no consumer, records unoffered) — P5.1 files
  the follow-up.

### P3.3 — the seam sync

Vector file copied **byte-identically** from the engrave worktree (`cmp`: no
difference) and re-pinned `542cd492…` → **`e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758`**,
the same literal the Rust half pins at `crates/me-cli/tests/descriptor_seam.rs:46`.

`TestDescriptorSeamSyswClass` un-skips and becomes the exhaustive derived rule,
split into two tests, one per basis, so the retired column's input-vs-canonical
ambiguity cannot return:

- `TestDescriptorSeamSyswClass` — every SINGLE-LINE row:
  `sysw.Classify(input) == ClassDescriptor` iff `host_admits`, else `ClassUnknown`;
- `TestDescriptorSeamSyswClassCanonical` — every `host_admits` row:
  `sysw.Classify(canonical) == ClassDescriptor`.

**Count guards — every value recounted from the file, none transcribed:**

| guard | before | after | note |
| --- | --- | --- | --- |
| `wantRows` | 71 | **72** | |
| `wantDeviceTrue` | 37 | **38** | the `ypub` row's flip |
| `wantDeviceFalse` | 33 | **34** | the new `wsh(multi(…/0/*))` witness |
| `wantDeviceAbsent` | 1 | **DELETED** | declared, never read; population now 0 |
| `wantSyswClass` | 4 | **RETIRED** | with its column |
| `wantPanicParse` | 1 | **RETIRED** | with its marker |
| `wantPanicEncode` | 2 | 2 | unchanged — S2 never touches `Encode` |
| `wantHostWider` | 3 | 3 | recounted: the three §4.6 whitespace rows, and only those |
| `wantCanonical` | 19 | 19 | recounted |
| `wantAddress0` / `wantAddress1` | 20 / 5 | 20 / 5 | recounted, **and now actually read** — see deviations |
| `wantWalletID` | 4 | 4 | recounted |
| `wantDeviceAddr0` / `wantDeviceAddr1` | 16 / 4 | 16 / 4 | recounted |
| `wantSingleLine` | — | **59** NEW | matches the Rust half's `SINGLE_LINE_ROWS` |
| `wantSingleLineAdmitted` | — | **15** NEW | matches `SINGLE_LINE_ADMITTED` |

The `panic:parse` switch arm was DELETED rather than left dead: a reintroduced
row now falls to `default` and reds, which is right — the harness rule that made
such a row safe (never feed it to `OutputDescriptor`) was retired with it.

### P3.4 — F-426's tests

`bip380/ypub_test.go`, three tests: `ParseExtendedKey` classifies `ypub` as
`P2SH_P2WPKH` (BIP-49, **not** `Ypub`'s `P2SH_P2WSH`, asserted adjacently);
normalises it to `xpub` (version `0488b21e`, and the exact xpub twin string);
and `ParseKey` accepts a bare `ypub` and takes the SLIP-132 fallback to
`49'/0'/0'`. A `upub` — the same 78 bytes under `044a5262`, constructed and
re-checksummed — must still refuse, so the widening is proven to be one case and
not a door.

The header states, and this was **measured against the S2 `me`**, that the
host's five-version admission is UNCHANGED in S2: both the bare `ypub` and
`sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))` refuse at **rc 3** with
*"the device admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is
`ypub`, whose equivalent is `xpub`: …"*. The convergence widening is F-426's own
later cycle.

The sysw-level bare-`ypub` negative lives in **one** place, not two:
`TestBareYpubClassifiesUnknown` in `sysw/descriptor_test.go`, landed with the arm
at P3.1 and cross-referenced from `bip380/ypub_test.go`'s header.

## Measured numbers

### Predicate parity — first try, no divergence

The derived rule holds over the regenerated file: **59/59 single-line rows in
both directions (15 `Descriptor`, 44 `Unknown`) and 19/19 canonicals**. This was
measured against the engrave copy in a scratch package *before* P3.1 was
committed, and is re-asserted by the committed test at P3.3.

### Classify cost delta

Go 1.26.3, i7-13700K, `-benchtime 500x -count 6`, medians, arm on vs. arm
disabled with an `if false &&`:

| corpus | without the arm | with the arm | per record |
| --- | --- | --- | --- |
| 76 records (63 single-line seam inputs + 13 `sysw/testdata/sysw_vectors.json` records) | 142 µs | 5.32 ms | 1.9 µs → **70 µs** |
| the 13 constellation records alone | 4.2 µs | 10.9 µs | 323 ns → **838 ns** |

So an ordinary non-descriptor record costs about **+0.5 µs**, and a
descriptor-shaped one about **+68 µs** — dominated by base58check and secp point
decompression inside the parser, not by the narrowings. Moving the parse ahead
of the string scan cut the short-record case from ~5.1 µs/record to ~0.8
µs/record and is why the order is what it is.

### TinyGo image cost

Same command both sides, `nix develop`:

| | `0abbf81` (baseline) | `fe9475c` | delta |
| --- | --- | --- | --- |
| total flash | 1 492 132 | 1 494 748 | **+2 616 B** |
| total ram | 62 568 | 62 568 | **0** |
| `seedhammer.com/sysw` flash | 4 154 | 5 278 | +1 124 B |
| `seedhammer.com/gui` flash | 237 247 | 237 477 | +230 B |
| `seedhammer.com/nonstandard` flash | 1 542 | 1 542 | 0 |

### Mutation checks — the new gates can fail, and on the right rows

| mutation | result |
| --- | --- |
| `wantSingleLineAdmitted` 15 → 14 | RED: `admitted single-line rows: 15, want 14` |
| drop §4.3's string-level version check | RED on **2** rows: `promotion/15-bare-tpub-host-refused`, `version-gap/full-origin-ypub` |
| drop ONLY §4.5's `tpub` clause | RED on **1** row: `promotion/15-bare-tpub-host-refused` |
| drop §4.7's conjuncts (arm = the scan door — the r1 C3 divergence) | RED on **16** rows — the remainder of the 18 after the two narrowings catch their own |
| delete `case ypubVer` in `bip380.ParseExtendedKey` | RED on all three P3.4 tests |

## Deviations, with reasons

1. **`wantAddress0` / `wantAddress1` were made to be READ.** Both were declared
   and never referenced — count guards that cannot fail, in the file P3.3 owns.
   `wantDeviceAddr0/1` cannot cover them: those count only the rows this route
   derives, so they are blind to an address column shrinking on a row the test
   skips. Two extra lines in `TestDescriptorSeamAddresses`. Populations
   unchanged (20 / 5).
2. **The §4.3 version check is a base58-run scan, not a re-parse of the
   grammar.** The plan says "STRING-LEVEL check" without prescribing a
   mechanism. The floor of 100 characters is a fact about the encoding (82 bytes
   base58check with a leading `0x02`/`0x04` is always 111 or 112 characters), and
   the direction that could be wrong is a false REFUSAL — which would require the
   parser to accept a ≥ 100-character base58 run that is not a key, which the
   grammar makes impossible. A missed key would need one shorter than 111
   characters.
3. **`recover()` was included.** The plan left it to the implementer. The two
   measured panics are closed and `Encode` is not called from classify, but the
   parsers underneath are handed bytes from a container this device did not
   write, and a panic in `Classify` is a payload that will not load. It costs 0
   bytes of RAM and the TinyGo build is green. This is the fork's first
   production `recover()`.
4. **The sim walk starts one link in, at an OPENED payload**, not at
   `syswLoadFlow` like `gui/chain_walk_test.go`'s four-link chain. The plan named
   `gui/transaction_walk_test.go:28-50`'s `runUI`/`pumpUntil`/`sessionWith`
   pattern as the template, which is the same shape; what changed is that the
   session is built from container bytes `me` wrote rather than from Go literals.
   Stated in the test's own header under "what it does not prove", and named in
   `chain_walk_test.go`'s header so the chain's own claims stay true.
5. **`scripts/descriptor-seam-vectors/goprobe/go.mod` left untouched**, per the
   brief. Note for the merge (review M-3): it already carries
   `replace seedhammer.com => /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm`,
   committed by P2 at `70f566e` — that is how P2.6 measured the two device
   booleans. **No Rust test invokes goprobe** (grep over `crates/`: one doc-comment
   mention at `crates/me-cli/tests/descriptor_as.rs:781`, and `scripts/…/gen.py`,
   which CI never runs), so the engrave suite does not consume the fork worktree
   through it.
6. **P3.5 was not in this brief** and was not done. It is the plan's own
   spec-amendment task, folded inline by the controller.

## P3 gate — output tails

**`go vet ./...`** — 41 findings at `fe9475c` and **41 at the `0abbf81`
baseline** (measured in a detached worktree, since removed). None names a file
P3 touched; all are pre-existing `unkeyed fields` / `testing.ArtifactDir
requires go1.26` notes, which is why `.github/workflows/test.yml:94` runs
`go test` and not `go vet`. `go vet ./sysw/ ./nonstandard/ ./bip380/` is clean.

**`gofmt -l`** — empty for **every file P3 touched**. The whole-tree list is
five files, identical at baseline and tip and untouched by P3
(`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`) — a Go 1.26.3
gofmt alignment difference against the module's pinned 1.25.10. CI runs no
gofmt check.

**`go test` on every package except `./gui/`** — exit 0, 52 packages `ok`.

**gui shard, `scripts/gui-shard-test.sh ./gui/ 24`** — exit 0:

```
=== enumerating tests in ./gui/ ===
    1008 top-level tests
    partition verified exhaustive: 1008 == 1008
=== running 24 shards in parallel (timeout 20m each) ===
…
=== wall: 72s ===
RESULT: ok -- all 1008 tests ran across 24 shards
```

**TinyGo device build** — the exact CI command, via
`nix develop --command`:

```
tinygo build -size full -print-stacks -o /dev/null -target pico-plus2 \
    -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```

```
   code  rodata    data     bss |   flash     ram | package
   1182     360       0       0 |    1542       0 | seedhammer.com/nonstandard
   4774     424      80       0 |    5278      80 | seedhammer.com/sysw
1194311  268825   31612   30956 | 1494748   62568 | total
function                         stack usage (in bytes)
Reset_Handler                    recursive, runtime.runtimePanicAt may call itself
…
EXIT=0
```

(The six `recursive, runtime.runtimePanicAt may call itself` lines are present at
the `0abbf81` baseline too — unchanged by the new `recover()`.)

**The sim walk renders** —
`TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen` PASS,
`TestS2ContainerRecordClassifiesAsADescriptor` PASS.

**Vector-copy byte-equality and both pins:**

```
cmp <engrave>/crates/me-cli/testdata/descriptor_seam_vectors.json \
    <fork>/nonstandard/testdata/descriptor_seam_vectors.json     → identical
e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758  (both)
crates/me-cli/tests/descriptor_seam.rs:46         e7a4160c…
nonstandard/descriptor_seam_test.go:42            e7a4160c…
```

**The engrave suite against the updated fork worktree** —
`PATH=<go 1.26.3> ME_REQUIRE_GO=1 cargo nextest run --locked`, exit 0:

```
     Summary [  32.234s] 579 tests run: 579 passed, 1 skipped
```

(The one skip is pre-existing and deliberate: `crates/me-cli/src/sysw/vectors.rs:132`,
`#[ignore = "regenerates the fixture; run deliberately"]`.)

**Not run, and deliberately:** the proportional opus review of the port named in
the plan's P3 gate — that is the controller's dispatch, not the implementer's.
Everything else in the gate is above.
