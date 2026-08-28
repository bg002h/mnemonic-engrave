# RECON — the payload walk matrix, and one complete chain

**Date** 2026-08-27
**Scope** every record class of a systemwide payload, from `me` at the command line
to an engraved plate, via the firmware's GUI.
**Fork branch** `walk/payload-chain` (worktree `/scratch/code/shibboleth/_work/walk/seedhammer`),
one commit `9c5c066` off `main` `0b656d7`. Nothing pushed. `upstream/` untouched.
**Tools** `me 0.7.0` (`/home/bcg/.cargo/bin/me`), `mt 0.1.0` (`/home/bcg/.cargo/bin/mt`),
Go 1.26.3 (`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go` — `go` is
**not on PATH** on this box; it comes from the Nix store).

---

## 0. Three corrections to the brief, up front

The brief's premises were mostly right. Three were not, and each changes the estimate
at the end.

1. **Link 4 already exists, for nearly every class.** `golden.CompareBSpline` is used
   by five test files, and the committed goldens cover the whole plate vocabulary:
   `gui/testdata/tx-{qr,text,unconfirmed,unsigned-qr}.bin`,
   `gui/testdata/sizeproof-{front,back}.bin`, and **17 blobs in `backup/testdata/`**
   (`seed-*`, `codex32-*`, `passphrase-*`, `freetext-*`, `slip39-*`, `text-*`).
   Each renders the real b-spline to SVG at the production stroke. `cmd/plateview`
   is *a* renderer, not *the* renderer — and it is the weaker one, because it has no
   transaction plate at all (see §3).

2. **A CLI-built payload already reaches the device — twice — but never reaches a cut.**
   - `gui/transaction_crosslang_test.go` → `TestHostPackedMtPayloadLoadsAndConfirms`
     loads `gui/testdata/sysw_mt_payload.bin`, packed by `me sysw pack` over the
     "even" vector's 6 `mt1` strings plus its `tx:` record. It stops at the session:
     parse, open, classify, confirm. No flow, no plate.
   - `cmd/emu/sysw_test_payload.bin` and `cmd/emu/sysw_cards_payload.bin` are both
     `me sysw pack` output embedded in the browser build, and `cmd/emu/walk_trace_a.js`
     drives one of them **to a completed engrave with a decoded toolpath**. That is a
     genuine four-link chain for `ClassMDMK` — it just is not a `go test`.

3. **`cmd/emu` does not discard the step stream.** `cmd/emu/engraver.go`'s own comment:
   *"It does, however, DECODE the stream on the way past (toolpath.go). Throwing it
   away left one defect class reachable only by cutting metal."* `emuEngraver.Plate`
   hands the spline to `beginPlate` and the page draws the plate as it cuts. What the
   emulator does not model is the machine — no stalls, no load, no failure.

**Verified, not assumed:** `me 0.7.0` re-packing the three records named in
`cmd/emu/sysw_test_payload.go`'s provenance comment produces a file **byte-identical**
to the committed blob (`sha256 b575baad2c4d9fd8d36e34f9341ca77d6242bad1dfbc9bdcefffa93a64277882`,
265 B) and prints the pinned digest `55ad b800 6ec6 a066 94f3 6a0e 900a c8d5`. That
fixture was recorded against `me 0.6.0` on 2026-08-13 and has not drifted.

---

## 1. PART A — the matrix

One row per class. **Every cell was measured**; the command or file is named.

Column meanings:
- **pack?** — does `me sysw pack` emit it? (exit code from a real invocation)
- **device?** — does `sysw.Classify` place it, and does any program admit it?
- **walk?** — is there a gui test that drives a *program flow* holding that class from
  a payload session?
- **cut?** — does such a walk reach a completed engrave (`testEngraver` closed, words > 0)?
- **source** — is the payload in that walk CLI-built or Go-built?

| class | pack? | device? | walk? | cut? | payload source |
|---|---|---|---|---|---|
| `Mnemonic` | **yes** rc=0, 93 B pub | classify **yes**; admitted by 6 programs | offer only | **no** | Go (`sessionHolding`) / **CLI in the emulator** |
| `Codex32Secret` | **yes** rc=0, 51 B pub | classify **yes**; BackupWallet, Xpub, SingleSig, Multisig, Bip85 | offer only | **no** | Go (`sysw_cells_test.go`) |
| `Passphrase` | **yes** rc=0, 61 B pub | classify **yes**; 6 programs | offer + consumed | **no** (as a plate) | Go / **CLI in the emulator** |
| `FreeText` | **yes** rc=0, 31 B pub | classify **yes**; Text only | offer only | **no** | Go / **CLI in the emulator** |
| `Descriptor` | **NO** rc=4 | **not classifiable** by `sysw`; *is* by `seal` | n/a | n/a | — |
| `MdMk` | **yes** rc=0, 67 B pub | classify **yes**; Bundle, SingleSig, Multisig, WalletPolicy | **yes** | **yes** | Go (`sessionHolding`) / **CLI in the emulator** |
| `Mt` | **yes** rc=0, 527 B pub (6 chunks) | classify **yes**; Transaction | **yes** | **yes** | **CLI** (new) + Go |
| `Tx` | **yes** rc=0, 447 B pub | classify **yes**; Transaction | **yes** | **yes** | **CLI** (new) + Go |
| `Address` | **NO** rc=4 | **not classifiable** by `sysw`; *is* by `seal`; **admitted by no program** | n/a | n/a | — |
| `Unknown` | **refused** rc=4 | fail-closed | n/a | n/a | — |

### 1.1 Evidence per cell

**pack? — measured by running the binary.** Records file per class, then
`me sysw pack --in rec.txt --no-passphrase --out pack.bin`, exit code captured to a
file and read from the file (never through a pipe):

```
mnemonic rc=0   text rc=0   pass rc=0   mdmk rc=0   mt rc=0   tx rc=0   ms1 rc=0
addr rc=4       desc rc=4   garbage rc=4
```

All three refusals print the same sentence, which is the honest one:

> `me: record 0 (records count from 0) is not a form this container can place: not a
> BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
> record. Descriptors and addresses are not yet classifiable here — see sysw::classify`

`me sysw show` on each container names the classes back:
`public record 0: mt1 chunk — confirmed`, `raw signed transaction — txid 2dcf2b97…, 222 bytes`,
`md1/mk1 — unconfirmed — engraveable, but the device REPLACES the legend`.

**A `tx:` record on argv is refused, but a *packed* one is not.** The refusal is an
argv-channel rule, not a container rule — `Class::is_bearer()` covers `Mt` and `Tx`
and `is_argv_forbidden() = is_secret() || is_bearer()`
(`crates/me-cli/src/sysw/record.rs`). Through `--in` or stdin it packs at rc=0. The
2026-08-26 ruling made this uniform: before it, argv refused a *transaction* and
accepted a *seed phrase*.

**One extra refusal inside `Tx`, and it matters.** A `tx:` record whose transaction
parses but whose inputs carry neither scriptSig nor witness is refused (rc=4) with a
four-line explanation naming the txid, unless `--allow-unsigned-inputs` is passed —
in which case it is admitted with a warning naming the input indices. The device's
`sysw.Classify` requires only a **structural parse**, so it accepts what the host
refuses; the signature predicate on device lives in `payloadTransactions`
(`gui/transaction.go:471`) and produces the `UNSIGNED TRANSACTION` legend substitution
rather than a refusal. **The CLI and the device are deliberately asymmetric here**,
and that asymmetry is what §4's defect rides on.

**device? — `Descriptor` and `Address` are absent by design, in both languages.**
`crates/me-cli/src/sysw/mod.rs:172`: *"Descriptor and Address are deliberately absent,
and this is a known limitation rather than an oversight: classifying them needs a
descriptor parser and an address decoder, neither of which is a dependency of this
crate."* `sysw/record.go:88` says the same, citing the primary. **They are not a walk
gap; they are a classifier gap, and it is upstream.**

`ClassAddress` additionally appears in **no** entry of `gui/sysw_admit.go`'s `admitted`
map — measured, `grep -c ClassAddress gui/sysw_admit.go` = 0 — so even if the sysw
classifier grew an address decoder tomorrow, no program would consume one.
`ClassDescriptor` *is* admitted (Bundle, Multisig, WalletPolicy), so a descriptor
classifier would be immediately useful and an address one would not.

**The other container does classify both.** `seal/record.go:205-222` reaches
`nonstandard.OutputDescriptor` and `btcaddr.DecodeAddress` for MainNet and TestNet3.
So the *device* can read a Descriptor or an Address out of a **Sealed Payload** — but
`me` still cannot write one:

```
me seal --plaintext <address>  → rc=4  "unrecognised record: unrecognized HRP 'bc'
                                        (expected md, mk, ms, or mt)"
me seal --in <descriptor file> → rc=4  "record has an uppercase character at byte 32
                                        — records must be lowercase (§6.4)"
```

The descriptor refusal is **structural, not incidental**: an `xpub` contains uppercase
by construction, so `me seal`'s §6.4 lowercase rule means **no descriptor can ever pass
through `me`**, by either container. This is worth a ruling. It is not obviously wrong
— §6.4 exists so one wallet cannot have two public-data hashes — but the consequence
is that a live `seal.Scan` branch has no producer in this constellation.

**walk? / cut? — measured by parsing the test source, not by recall.** A Python pass
over `gui/*_test.go` splitting on top-level `func Test` and requiring a payload session
(`sessionHolding` / `sessionWith` / `ctx.sysw =`) *and* an engraver in the **same
function body** yields exactly three, all `ClassMDMK`:

```
gui/multisig_build_walk_test.go        TestBuildWalkTypedSeed
gui/multisig_engrave_tail_walk_test.go TestBothEngraveFlowsDriveTheRetryLoop
gui/multisig_engrave_tail_walk_test.go TestBuildAbortIsTheLastScreenOfTheProgram
```

plus the six in `gui/transaction_walk_test.go` and the four new ones in
`gui/chain_walk_test.go`, whose `newEngraver()` sits inside the `newTxWalk` /
`newChainWalk` helpers and so is invisible to that filter. **Scope of this negative:**
`gui/*_test.go` only, function-body granularity. I did not search other packages; a
payload-fed cut living outside `gui` would not appear.

For `Mnemonic`, `Codex32Secret`, `Passphrase` and `FreeText` the payload-fed tests
that exist (`gui/sysw_cells_test.go`, `gui/sysw_picker_test.go`,
`gui/s6b_passphrase_plate_test.go`) prove the record **arrives** — the offer is drawn,
`FROM PAYLOAD` is chosen, the right typed object comes back — and stop there. A
`pass:` record *is* consumed by flows that engrave (`s5PassphraseRecord` in
`singlesig_truth_test.go` and `multisig_supply_passphrase_test.go`), but as key-derivation
input, not as the engraved artifact; the dedicated BIP-39 Password plate has no
payload→cut walk.

---

## 2. PART B — the one complete chain, made real

**`gui/chain_walk_test.go`, four tests, all passing.** The kind is `Tx`+`Mt`.

```
me sysw pack --in <records> --no-passphrase --out chain-tx.bin      (link 1)
   → gui/testdata/chain/chain_payloads.json  {blob, bytes, sha256, digest, command}
   → padded to 64 KiB, 0xFF tail                                     (link 2)
   → sysw.FileReader  — the host stand-in for the XIP read
   → syswLoadFlow(ctx, th, reader, atBoot=true)
        "A systemwide payload is present. Load it?" → LOAD
        "Payload Digest" → asserted equal to `me sysw show`'s number → CONTINUE
   → engraveTransactionFlow                                          (link 3)
        review (txid + BEARER) → plate kind → plan → engrave loop → post-cut
   → golden.CompareBSpline vs gui/testdata/tx-qr.bin                 (link 4)
```

### 2.1 Exact commands to re-run it

```sh
cd /scratch/code/shibboleth/_work/walk/seedhammer
GO=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go

# the chain itself (no toolchain but Go needed; the fixture is committed)
$GO test ./gui/ -run TestChain -count=1 -v -vet=off -timeout 10m

# write the plate SVG somewhere you can look at it
CHAIN_PLATE_OUT=/tmp/chain-tx-qr.svg \
  $GO test ./gui/ -run TestChainFromAMePackedPayloadToACutQRPlate -count=1 -v -vet=off

# regenerate the fixture from the real CLI
ME=/home/bcg/.cargo/bin/me MT=/home/bcg/.cargo/bin/mt ./scripts/gen-chain-fixtures.sh

# audit the committed fixture against the CLI (needs `me`; absence is FATAL, not a skip)
ME=/home/bcg/.cargo/bin/me \
  $GO test -tags oraclelive ./gui/ -run TestChainFixtures -count=1 -v -vet=off
```

### 2.2 What it measured

```
TestChainFromAMePackedPayloadToACutQRPlate   PASS  7.64s
    56,686,496 stepper words, digest 0x53608cc84e9996c1
    1 plate(s), 1 QR, ECC H, 0.6mm modules
    plate artifact tx-qr.bin.svg  1,050,640 B  — matches golden testdata/tx-qr.bin
TestChainFromAMePackedPayloadToACutTextPlate PASS  3.19s
    6 string(s), 1 plate
    plate artifact tx-text.bin.svg  547,685 B — matches golden testdata/tx-text.bin
TestChainATxOnlyPayloadOffersNoTextPlates    PASS
TestChainGP310SilentlyDropsAByteDifferentTwin PASS
```

Fixtures: `chain-tx` 1027 B (7 records), `chain-gp310` 809 B (7 records),
`chain-txonly` 499 B (1 record). `me sysw pack` is **deterministic for the unsealed
variant** — measured, three runs, identical sha256 — because salt and IV are only
consumed on the sealed path. That is what makes the fixture pinnable; a sealed one
could not be.

### 2.3 The two joins that did not exist before

**The digest.** `me sysw show chain-tx.bin` prints
`e7e5 152f 6fe7 c022 d3fe 837b f643 f033`. The device's Payload Digest screen
recomputes it from the container and asks the operator to compare it against exactly
that command. Nothing asserted the two were equal. Now `chainWalk.ingest` does.

**The plate.** `gui/testdata/tx-qr.bin` was recorded from a **Go-built** session
(`sessionWith(txEven...)`, a literal). The chain reaches the same plate from **CLI
bytes** and compares knot-for-knot. A producer that emitted a byte-different record,
or a device that classified it into a different shape, moves the spline and fails.
`chainGoldenPlate` passes `update=false` unconditionally, whatever `-update` was given
— letting the CLI route re-record the golden would let it silently redefine the thing
it is measured against.

### 2.4 Mutation-tested, because a chain that cannot fail is decoration

| mutation | result |
|---|---|
| swap `chain-tx`'s `blob` for `chain-txonly`'s, keep its `digest` | **FAIL** at ingest — device recomputed `c2826ca8…`, pinned `e7e5152f…` |
| point the QR walk's golden at `tx-unsigned-qr` | **FAIL** — spline lengths 46952 vs 38461, 38460/38461 knots differ |

Both fired at the intended assertion with the intended message. Both mutations were
reverted and the tree verified clean.

### 2.5 Full validation surface

```
gui        998/998 ok, 64 s wall, 24 shards, partition verified exhaustive
           (scripts/gui-shard-test.sh from mnemonic-engrave; PATH must carry the Nix go)
rest       52 packages ok
go build ./...  clean
oraclelive TestChainFixturesStillMatchWhatMeEmits  3/3 subtests ok against me 0.7.0
```

**Pre-existing, not caused by this work:** `go vet ./gui/` reports
`testing.ArtifactDir requires go1.26 or later (file is go1.25)` at
`gui/freetext_sizeproof_golden_test.go:111` and `gui/transaction_golden_test.go:104`.
`go.mod` says `go 1.25.10`; CI's `setup-go` says `1.26`. `go test` runs a vet subset
that excludes `stdversion`, so tests are unaffected — but the `go vet -tags oraclelive
./...` step the vendored-vectors comment relies on would trip on it. **Worth a
separate look; I did not touch it.**

### 2.6 What this chain does NOT prove

Stated in the test file's own header, so it travels with the code:

- **No hardware.** `testEngraver` accepts the stepper stream and digests it. Nothing
  here says a motor turned or steel was cut.
- **The screens are text, not pixels.** `op.Drawer.ExtractText` walks the op tree, so
  the frames show *what the device says*, not *how it looks*. A legend rendered off
  the panel passes. The extractor also **concatenates ops with no separator**, so the
  screen's `e7e5 152f …` arrives as `e7e5152f…`: the digest comparison is on the 32
  nibbles and **cannot see the grouping**. (This cost me one failing round to discover
  and is now written down at the assertion.)
- **The plate is a render of the PLAN, not a capture of the cut.** The `Plate` the walk
  engraved is a local inside `transactionReviewAndEngrave` and never escapes. The
  render re-plans from the same session's candidate — deterministic — and is bound to
  the walk by the plate **title** and **count** the walk itself asserted, and by nothing
  stronger.
- **The fixture is pinned, not live.** The committed bytes are `me` output; the default
  suite does not run `me`. `gui/chain_fixture_live_test.go` (build tag `oraclelive`)
  does, and is the only thing that catches drift.
- **One kind.** `Tx` and `Mt`. Six other packable classes reach no payload→cut walk.
- **Unsealed only.** The sealed variant's KDF path (`inputWordsFlow` + PBKDF2) is
  walked by `gui/sysw_load_test.go` against the vendored vectors, but not from a
  CLI-built fixture, and cannot be byte-pinned.

---

## 3. `cmd/plateview` cannot render a transaction plate

`gui/preview.go`'s `previewBuilders` map has exactly **eight** entries:
`textproof`, `constproof`, `bothproof`, `sizeproof-front`, `sizeproof-back`,
`freetext`, `seed`, `passphrase`. There is no transaction builder, so
`plateview -plate tx-qr` is an unknown plate. The chain therefore reaches
`golden.Vectorize` through `golden.CompareBSpline` instead — the same renderer, the
same production stroke width — rather than inventing a ninth builder whose output
nothing would compare. **If a transaction preview is wanted for its own sake, that is
a small, separate piece of work** (one builder that takes a candidate; the plan
functions already return `[]Plate`).

---

## 4. G-P3.10, as it manifests along the whole chain

`gui/transaction.go:467` merges transaction candidates on `c.tx.TxidDisplay` — the
**derived txid**, never the bytes. A `tx:` record byte-different from an existing
candidate but sharing its txid is **dropped**: not merged, not flagged, not a second
picker row. A transaction and its signature-stripped form share a txid by construction.

`TestTheMergeIsKeyedOnTheTxidNotOnTheBytes` already measures this at the
`payloadTransactions` level from a Go-built session. `TestChainGP310SilentlyDropsAByteDifferentTwin`
measures it **along the chain**, which is where the cost shows:

1. `me sysw pack` **refused** the 113-byte stripped record outright and printed four
   lines naming the txid, the reason, and the escape hatch.
2. Under `--allow-unsigned-inputs` it packed, printing a warning that names the input
   index and says *"the plate you are about to cut can never be broadcast"*.
3. `me sysw show` lists **seven** records and describes the seventh at length:
   *"raw transaction with UNSIGNED input(s) — txid 2dcf2b97…, 113 bytes; … It was
   packed with --allow-unsigned-inputs."*
4. The device loads all seven and classifies them **6 `Mt` + 1 `Tx`** — asserted, so
   the loss below is the merge and not a classification failure.
5. `payloadTransactions` returns **one** candidate, the 222-byte signed form.
6. No picker appears. The review screen names one transaction and says nothing about
   the other. The test asserts the screen mentions none of `113`, `discard`, `dropped`,
   `ignored`, `second record` — so if the drop ever stops being silent, the test says so.

**The operator had every reason to believe the machine held both.** They fought a
refusal to get the second record in, and the host tool listed it back to them twice.

**Not fixed here** — the operator has ruled *"engrave both"*, and that is separate work.
Recorded so the ruling starts from what the code does rather than from what the
acceptance sheet says it does. (The sheet's R10 row is wrong in the other direction:
it records the merge as being on bytes and files the residual as "two identical picker
rows". There is one row.)

---

## 5. What remains, per kind, ordered by how much work each needs

Ordered smallest first. "CLI-fed" below means *replace the Go literal with a committed
`me sysw pack` fixture and run the existing walk against it*.

| # | kind | what is missing | size |
|---|---|---|---|
| 1 | **`Mt` / `Tx`** | nothing — done | — |
| 2 | **`MdMk`** | a CLI-built fixture for `sessionHolding(records...)`. `cmd/buildpayloadcards` already pipes into `me sysw pack` (see `cmd/emu/sysw_cards_payload.go`'s provenance line), and `TestBuildWalkTypedSeed` already walks to a cut with goldens behind it. This is **one fixture entry plus a `newChainWalk` variant**. | ~1 h |
| 3 | **`Mnemonic`** | a payload→cut walk at all. `backup/testdata/seed-{0,1}*.bin` goldens exist; `TestSeedEntryOffersThePickerForAPayloadAlone` gets the record out of the payload. The missing piece is driving `backupWalletFlow` past the offer to an engrave, then binding to the seed golden. | ~half a day |
| 4 | **`FreeText`** | same shape. `backup/testdata/freetext-{0-plain,1-qr}.bin` goldens exist; `gui/freetext_flow.go:1496` has the offer. Drive `engraveTextFlow` from a payload to a cut. | ~half a day |
| 5 | **`Passphrase`** | same shape, plus a decision: the BIP-39 Password *plate* (`progPassword`) is the artifact, and `backup/testdata/passphrase-*.bin` has **five** goldens including `passphrase-4-preloaded`. The `pass:` body must be hex-decoded on the way (`syswPassphraseFlowTitled` — three sites do this and the trap is the same at all three). | ~half a day |
| 6 | **`Codex32Secret`** | same shape. `backup/testdata/codex32-{0,1}.bin` goldens exist; `TestBackupWalletTakesACodex32SecretFromThePayload` proves arrival and returns the `codex32.String` `engraveObjectFlow` routes. | ~half a day |
| 7 | **`Descriptor`** | **blocked upstream, and not by the walk.** `sysw::classify` cannot place one (no descriptor parser in the crate), and `me seal`'s §6.4 lowercase rule refuses every descriptor by construction because an `xpub` has uppercase. Three programs admit the class and nothing can deliver it. **Needs a ruling before any code.** | ruling first |
| 8 | **`Address`** | **not a gap.** No `sysw` classifier, and admitted by **zero** programs. `seal` classifies it for the other container. Unless a program is going to engrave an address, this row should be marked "cannot be engraved by nature of the admission table" and closed. | close it |
| 9 | **`Unknown`** | not a kind. It is the fail-closed arm and is already tested on both sides. | — |

**Two cross-cutting items** that are not per-kind:

- **A generic `newChainWalk` seam.** Right now it is transaction-shaped: it calls
  `engraveTransactionFlow` directly. Rows 3–6 want it parameterised by the flow to run.
  That is a small refactor and should happen once, before row 3, not five times.
- **The `go vet` / `go.mod` language-version mismatch** in §2.5. It disables the
  `oraclelive` type-check the vendored-vectors comment claims runs on every push.

---

## 6. Is "every kind" a day, a week, or a structural problem?

**Not a structural problem. Call it three to four focused days, and the first day
buys most of the value.**

The reason it is not structural is that every part already exists and is proven
somewhere; what is missing is the *join*:

- **Link 1** works for all eight packable classes — measured, eight `rc=0` invocations.
- **Link 2** works for anything link 1 emits — `sysw.FileReader` + `syswLoadFlow`, and
  the 8 vendored vectors already exercise the load path including the sealed KDF. (The
  vendored copy is byte-identical to the primary's, sha `7e58779d…`, pin
  `file_commit 2b570fc…` — verified current against `mnemonic-engrave` HEAD `59dd1e4`.)
- **Link 3** exists per program; the walks are written, they just start from literals.
- **Link 4** exists for the whole plate vocabulary — **23** committed goldens across
  `gui/testdata` and `backup/testdata`.

So each remaining kind is: one fixture entry in `chain_payloads.json`, one generator
line in `gen-chain-fixtures.sh`, one walk that drives its flow past the payload offer,
one `CompareBSpline` against a golden that is already committed. The `Tx` row took an
afternoon *including* discovering the harness, the button semantics, the ExtractText
space-stripping, and the load-flow ordering trap. Rows 2–6 pay none of that again.

**The two things that could turn it into a week** are both decisions, not code:

1. **The `Descriptor` ruling** (§5 row 7). If a descriptor must be deliverable, §6.4's
   lowercase rule has to be revisited *and* `sysw::classify` needs a descriptor parser
   in the Rust primary first (the Rust-primary rule binds: normative behaviour lands in
   Rust with vectors, then ports). That alone is more work than rows 2–6 combined.
2. **Whether a CLI-built fixture is the right shape for the seed classes.** A fixture
   carrying a mnemonic is a payload with a secret in plaintext; `syswLoadFlow` will show
   the F1 warning and the KEEP/UNLOAD offer, which every such walk must then drive
   through. That is *good* — it is a screen nothing currently walks from a real payload
   — but it means rows 3–6 each grow two extra steps, and the F1 path deserves its own
   assertion rather than being tapped past.

**One thing I would do before anything else**, and it is cheap: the emulator already
walks `ClassMDMK` from a CLI-built payload to a completed engrave with a real
framebuffer (`cmd/emu/walk_trace_a.js` + `sysw_cards_payload.bin`). Nothing in `go test`
knows that. Row 2 should be written so the `go test` chain and the emulator walk are
looking at **the same fixture bytes** — otherwise the constellation has two
CLI-built MdMk payloads that can drift apart, and only one of them fails a CI run.

---

## 7. Files touched

**In the fork**, branch `walk/payload-chain`, commit `9c5c066` (nothing pushed):

```
gui/chain_walk_test.go                     the four chain walks + harness
gui/chain_fixture_live_test.go             //go:build oraclelive — re-runs `me`
gui/testdata/chain/chain_payloads.json     3 CLI-built containers, pinned
scripts/gen-chain-fixtures.sh              the reproduction path, committed
```

**In `mnemonic-engrave`**: this report only. No source touched — the crate was read-only
throughout, as briefed.
