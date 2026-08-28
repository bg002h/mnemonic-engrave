# IMPL — a payload→plate chain for every packable record class

**Date** 2026-08-27
**Scope** multiply the proven `Tx`/`Mt` chain across every other class a systemwide
payload can hold, as `go test`, with all four links real.
**Where** `/scratch/code/shibboleth/_work/walk/seedhammer`, branch `walk/payload-chain`.
Nothing pushed. `upstream/` untouched. One commit added on top of `9c5c066`.
**Tools** `me 0.7.0` (`/home/bcg/.cargo/bin/me`), `mt 0.1.0`,
Go 1.26.3 (`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go` — `go` is
**not on PATH** on this box).

---

## 0. The answer, in one table

Twelve chain tests, all four links real in every one, all green, all
mutation-tested in both directions with 12/12 mutants killed at the intended
assertion.

| class | fixture | flow | plates cut | golden | mut 1 (wrong fixture) | mut 2 (wrong golden) |
|---|---|---|---|---|---|---|
| `Mt` + `Tx` | `chain-tx` | `engraveTransactionFlow` | 1 | `tx-qr` / `tx-text` | KILLED at ingest | KILLED at compare |
| `Mnemonic` | `chain-seed` | Backup Wallet | 1 | `chain-seed` | KILLED at ingest | KILLED at compare |
| `Codex32Secret` | `chain-codex32` | Backup Wallet | 1 | `chain-codex32` | KILLED at ingest | KILLED at compare |
| `FreeText` | `chain-text` | Engrave Text | 1 | `chain-text` | KILLED at ingest | KILLED at compare |
| `Passphrase` | `chain-pass` | BIP-39 Password | 1 | `chain-pass` | KILLED at ingest | KILLED at compare |
| `MDMK` | `chain-mdmk` | Build Multisig Policy | **9** | `chain-mdmk-md1-1` | KILLED at ingest | KILLED at compare |
| `Descriptor` | — | — | — | — | **not built — see §6** | — |
| `Address` | — | — | — | — | **not built — see §6** | — |

`Unknown` is not a kind; it is the fail-closed arm.

**The one invocation that runs every chain:**

```sh
cd /scratch/code/shibboleth/_work/walk/seedhammer
/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go \
  test ./gui/ -run TestChain -count=1 -vet=off -timeout 10m
# ok  seedhammer.com/gui  43.327s   — 12 top-level tests, 9 subtests
```

---

## 1. The two structural fixes

### 1.1 The harness is parameterised by FLOW, once

`newChainWalkFlow(t, name, flow)` owns links (1) and (2) — read the region,
`sysw.FileReader`, `syswLoadFlow`, the digest comparison, the F1 screens. A class
chain supplies only link (3), as a `chainFlow` with the signature every top-level
flow in the package already has:

```go
type chainFlow func(ctx *Context, th *Colors)
func newChainWalk(t *testing.T, name string) *chainWalk {
	return newChainWalkFlow(t, name, engraveTransactionFlow)
}
```

`newChainWalk` is now that one line, so the four pre-existing transaction chains
are byte-unchanged in behaviour and were re-run to prove it. Link (4) got the
same treatment: `chainCompareGolden` is the single comparison, and
`chainGoldenPlate` (the transaction-shaped re-planner) became a caller of it.

**Evidence there is one harness and not six:** every chain in the tree reaches
`syswLoadFlow` through one function.

```
$ grep -n 'syswLoadFlow' gui/chain_walk_test.go gui/chain_class_walk_test.go
gui/chain_class_walk_test.go:62:  ... makes syswLoadFlow draw a warning          (prose)
gui/chain_walk_test.go:40:        ... and opened by syswLoadFlow,                (prose)
gui/chain_walk_test.go:205:       syswLoadFlow's real screens.                  (prose)
gui/chain_walk_test.go:265:       if !syswLoadFlow(ctx, &descriptorTheme, ...)  <-- THE ONLY CALL
gui/chain_walk_test.go:820:       ctx.sysw is assigned inside syswLoadFlow,     (prose)

$ grep -c 'runUITouch(ctx'   gui/chain_walk_test.go        1
$ grep -c 'newChainWalkFlow' gui/chain_class_walk_test.go  5   (one per class)
```

Five textual hits, **one call site**, five classes reaching it.

The load half is also the half most likely to grow a screen — it grew two during
this work, the F1 warning summary and the KEEP/UNLOAD offer — which is exactly
why it must have one home. Adding them cost one edit, not six.

### 1.2 The `MdMk` chain and `walk_trace_a.js` read the SAME BYTES

The drift risk was real and is now closed structurally rather than by promise.

`gui/testdata/chain/chain_payloads.json` gained a `file` field. `chain-mdmk` sets
it to `../../../cmd/emu/sysw_cards_payload.bin` and carries **no `blob`**:

```
$ python3 -c "import json;d=json.load(open('gui/testdata/chain/chain_payloads.json'));p=[x for x in d['payloads'] if x['name']=='chain-mdmk'][0];print('file:',p['file']);print('blob?',bool(p.get('blob')));print('bytes:',p['bytes'])"
file: ../../../cmd/emu/sysw_cards_payload.bin
blob? False
bytes: 978
```

`chainBytes` resolves it, checks the length and the sha256, and fails with a
message naming the generator. So there is exactly **one copy of those bytes in
the repo**, and the `go test` chain and the browser walk load the same file.

The *digest* was written down in three places and nothing required them to agree.
`TestChainMdMkFixtureIsTheEmulatorsOwnPayload` now does, reading the two source
files rather than the symbols (one is `//go:build js`, the other is not Go):

| where | value |
|---|---|
| `cmd/emu/sysw_cards_payload.go` `syswCardsDigest` | `2527 1e58 3f3e aa03 ae18 f359 c72b 76e3` |
| `cmd/emu/walk_trace_a.js` `CARDS_DIGEST` | `25271e583f3eaa03ae18f359c72b76e3` |
| `chain_payloads.json` `digest` | `2527 1e58 3f3e aa03 ae18 f359 c72b 76e3` |
| recomputed by `sysw.PublicDataHash` over the blob | `2527 1e58 3f3e aa03 ae18 f359 c72b 76e3` |

```
$ go test ./gui/ -run TestChainMdMkFixtureIsTheEmulatorsOwnPayload -v
--- PASS: TestChainMdMkFixtureIsTheEmulatorsOwnPayload (0.00s)
```

The `oraclelive` audit learned the file case too: for a file-backed fixture it
runs `me sysw show <file>` and compares the digest, rather than trying to re-emit
a container that needs `go run ./cmd/buildpayloadcards`. 8/8 fixtures audit clean
against `me 0.7.0`.

---

## 2. The four links, per class, with the command that proves each

Link (1) is the same command for all five new fixtures and is committed in
`scripts/gen-chain-fixtures.sh`; re-running it left the three pre-existing
entries **byte-identical**, which is the generator's own regression check.

```sh
ME=/home/bcg/.cargo/bin/me MT=/home/bcg/.cargo/bin/mt ./scripts/gen-chain-fixtures.sh
# wrote gui/testdata/chain/chain_payloads.json with 8 payloads
```

### 2.1 `ClassMnemonic` — `chain-seed`

| link | what proves it |
|---|---|
| 1 CLI | `me sysw pack --in <93-byte mnemonic> --no-passphrase --out chain-seed.bin` → rc=0, 145 B, digest `f36e 9900 a235 0b1e 1c5f c580 2623 32b9` |
| 2 ingest | `chainRegion` pads to `sysw.RegionLen`, `sysw.FileReader`, `syswLoadFlow(atBoot=true)`; the walk asserts the Payload Digest screen shows that number |
| 3 walk | `newInputFlow` → `Seed from where?` **FROM PAYLOAD** → seed screen carrying `1: ABANDON … 12: ABOUT` → `Add a BIP-39 passphrase?` Skip → cut. **28,005,740 stepper words**, digest `0x6f2a0bd266b1df53` |
| 4 plate | `chainSeedPlate` from the mnemonic the payload produced, vs `gui/testdata/chain-seed.bin` recorded by `TestChainPlateGoldens` from the Go constant |

This is the first chain in the tree that walks `syswLoadFlow`'s **F1 screens**
from a real container: `Payload Warnings — A SECRET is stored unencrypted in
flash.` then `Keep this payload loaded?`. `me` says the same thing at the other
end (`NOT SEALED … this payload HOLDS SECRET MATERIAL (record 0 (BIP-39
mnemonic))`), so both ends of the chain warn about one fact and `ingest()`
asserts the device's half.

### 2.2 `ClassCodex32Secret` — `chain-codex32`

| link | what proves it |
|---|---|
| 1 CLI | `me sysw pack --in <ms1> …` → rc=0, 102 B, digest `313a 85da 2fb6 5406 da36 ead1 c215 a7e7` |
| 2 ingest | as above; F1 fires (an ms1 secret is secret material) |
| 3 walk | `newInputFlow`'s **second** offer — `syswOffer(ClassCodex32Secret)`, gui.go:2763 — → `Confirm Codex32 Secret`, `id ENTR`, `Unshared secret` → cut. **27,374,812 stepper words**, digest `0x736deaf7c5e3892b` |
| 4 plate | `backup.EngraveSeedString` on the `codex32.String` the payload produced, vs `chain-codex32.bin` |

The payload deliberately holds **only** an ms1: `newInputFlow` asks for a
mnemonic first, and a fixture carrying both would never reach the second offer.

### 2.3 `ClassFreeText` — `chain-text`

| link | what proves it |
|---|---|
| 1 CLI | `me sysw pack --in <text: hex> …` → rc=0, 105 B, digest `f802 8b09 a39e bc14 f745 baf2 c55e 918d` |
| 2 ingest | as above; **no F1** — a `text:` record is not secret, asserted with `assertF1(false)` |
| 3 walk | `engraveTextFlow` → `Text from where?` FROM PAYLOAD → the F3 acceptance screen → QR / Font / Size pickers, each chosen by a **tap hit-tested against the drawn frame** → the text field asserted to hold the payload's body → Title, Footer, Confirm → cut. **1,529,636 stepper words**, digest `0x2a9bf4d60a5314f4` |
| 4 plate | **captured, not rebuilt** — `freetextEngraveHook` receives the finished `Plate` at the moment the flow hands it to the engraver |

### 2.4 `ClassPassphrase` — `chain-pass`

| link | what proves it |
|---|---|
| 1 CLI | `me sysw pack --in <pass: hex> …` → rc=0, 113 B, digest `760c 6f11 b4ed 1892 9fff 63a0 ad34 8612` |
| 2 ingest | as above; F1 fires |
| 3 walk | `engravePassphraseFlow` → `Password from where?` FROM PAYLOAD → acceptance → entry step with the field pre-filled (`/100` counter) → `Seed FP` blank → `Expected Comb FP` blank → QR declined → Confirm → cut. **4,416,257 stepper words**, digest `0x9cba458d64e96fbb` |
| 4 plate | built from the exact arguments `ppBuildPlate` was called with, captured through `passphrasePlateHook` — the seam whose own comment says a caller passing the whole buffer instead of `secret[:n]` would put a stale tail on the plate and no unit test could see it. The chain asserts the captured secret is the payload's body and that both fingerprints and the QR flag are empty. |

This is the BIP-39 Password **plate** — the artifact — not the key-derivation
input. `progPassword` is the only program that admits `ClassPassphrase` and cuts
it.

### 2.5 `ClassMDMK` — `chain-mdmk`, and why it is Build and not Bundle

| link | what proves it |
|---|---|
| 1 CLI | `go run ./cmd/buildpayloadcards \| me sysw pack --no-passphrase --in - --out cmd/emu/sysw_cards_payload.bin` (the blob's own provenance line); 978 B, 10 records, digest `2527 1e58 …` |
| 2 ingest | the file itself, sha256-checked, padded, read through `sysw.FileReader`; F1 fires (record 9 is master A's mnemonic) |
| 3 walk | `buildMultisigPolicyFlow` — 13 screens, then the seed **also** from the payload, then Key sources, Policy stub, Which md1?, EXPERIMENTAL (held), What to engrave? Full, `This engraves 9 plates`, then **nine** completed cuts. **202,830,156 stepper words**, digest `0xf4e3b14dd71330ca` |
| 4 plate | the md1's first chunk, re-planned through the same `validateMdmk(pl, s, "", "")` `bundleEngrave` makes, variant 0 (`TEXT + QR`), vs `chain-mdmk-md1-1.bin` |

`bundleFlow` **cannot** serve this blob without a tag reader, and that is a
structural fact rather than a fixture gap: its payload seam is
`ctx.syswBundleSeeds = []string{body}` — **one record** — and every mk1 card here
is two or three chunks, so the gatherer drops it as incomplete. The emulator walk
shows exactly that (`Dropped an incomplete card`) before falling back to NFC.
Build Multisig Policy is the one program that consumes the payload's cards
*whole*, through its own over-supply picker.

The nine-plate census is read back through the production seam
`gui/engraved_hook.go` (the same one `cmd/emu`'s gate uses), so "nine plates" is a
statement about **content**:

```
plate 1: ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
plate 2: mk1qppeytpqqsqsvg26cpeutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5455km6nvcp4e2cj
plate 3: mk1qppeytpp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl9y8y3dgz4682v22fwl09n
plate 4: md1fxrvxzspqjtvyyy4qqxppcgsc27rchwsv0jskp2rsal4egz4ep5859p875x67p5s5tk09nzz08lv4
plate 5: md1fxrvxzsg3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e20urgnf9h0eap9ujpc
plate 6: md1fxrvxzshanz7jwkzae8efd8x2rk7av9gmew82jq5zap9302ynhp37ggd6z5u4emcwp309kt3p7qjm
plate 7: md1fxrvxzsag0zr8gh9upnjugr26jfvunvs35jvgdjkm3kghwnt0qqymzc0utyzxyhs535vcd9tq33dm
plate 8: md1fxrvxz3ry9pu8uyfgtl737tj9hrdzaved93u3a2nawefnqlk8ycusz0quhfev0as4ure80muhz6hz
plate 9: md1fxrvxz3fhqdghjmksz3ry92d3gv4ejtmu9f0zxf3clxvtlnnv86xy4qee32ay5yaf3qfqktd9h9
```

`unattributed` is asserted to be 0, so nothing was cut this census cannot name.
Plate 4's string is pinned as `chainMdMkMd1Chunk1` — measured then pasted, not
re-derived beside the assertion, because deriving it would restate the code under
test.

---

## 3. Mutation results — 12 of 12 killed, both directions

Committed as `scripts/chain-mutation-check.sh` so the claim is a command, not a
paragraph. It swaps a payload's bytes **while keeping its pinned digest** (and
swaps the donor's `sha256` and `bytes` in too, deliberately — leaving them would
make `chainBytes`' hash check fire first, a different assertion proving nothing
about the ingest), then overwrites a golden with another chain's. Every mutation
is reverted and the tree is verified clean.

```
$ GO=…/go ./scripts/chain-mutation-check.sh
chain-tx       MUTATION 1  KILLED at ingest: chain_walk_test.go:474: the device's digest screen does not show what `me sysw show` printed.
chain-tx       MUTATION 2  KILLED at compare: spline lengths 46952, 38461, with 38460/38461 knot mismatches
chain-seed     MUTATION 1  KILLED at ingest: chain_class_walk_test.go:89
chain-seed     MUTATION 2  KILLED at compare: spline lengths 21378, 1366, with 1365/1366 knot mismatches
chain-codex32  MUTATION 1  KILLED at ingest: chain_class_walk_test.go:198
chain-codex32  MUTATION 2  KILLED at compare: spline lengths 23823, 21378, with 21377/21378 knot mismatches
chain-text     MUTATION 1  KILLED at ingest: chain_class_walk_test.go:330
chain-text     MUTATION 2  KILLED at compare: spline lengths 779, 1366, with 778/1366 knot mismatches
chain-pass     MUTATION 1  KILLED at ingest: chain_class_walk_test.go:420
chain-pass     MUTATION 2  KILLED at compare: spline lengths 1366, 779, with 778/779 knot mismatches
chain-mdmk     MUTATION 1  KILLED at ingest: chain_class_walk_test.go:538
chain-mdmk     MUTATION 2  KILLED at compare: spline lengths 16540, 23823, with 16539/23823 knot mismatches

killed at the intended assertion: 12     not killed as intended: 0
tree clean: every mutation was reverted
```

Every mutation-1 kill lands on the **same assertion**: the Payload Digest screen
not showing what `me sysw show` printed — the number an operator compares by hand.
Every mutation-2 kill lands on `golden.CompareBSpline` with the two spline lengths
named.

The donors are chosen so the F1 classification matches. A secret payload swapped
for a non-secret one would stop at `assertF1` and never reach the digest screen
the mutation is aimed at — which would be a mutant killed by the wrong thing, and
the script says so in a comment.

---

## 4. Findings — recorded, not fixed

### 4.1 G-P3.10, as briefed: unchanged, still measured

`gui/transaction.go:467` merges transaction candidates on the derived txid, never
on the bytes. `TestChainGP310SilentlyDropsAByteDifferentTwin` measures it along
the whole chain and is untouched by this work — it still passes, still asserts the
device loads all 7 records, classifies them 6 `Mt` + 1 `Tx`, offers **one**
transaction, and that no screen mentions `113`, `discard`, `dropped`, `ignored`
or `second record`. The operator has ruled "engrave both"; that is separate work.

### 4.2 NEW — `me` and the fork disagree about which codex32 lengths exist

Found because the `Codex32Secret` chain could not be written with the string the
rest of the fork uses.

```
$ me seal --in <ms1 74 chars> --out /tmp/x --seal-secret   ; echo rc=$?
me: invalid record: string length 74 outside v0.1 set [50, 56, 62, 69, 75]
rc=4
$ me seal --in <ms1 48 chars> --out /tmp/x --seal-secret   ; echo rc=$?
me: invalid record: string length 48 outside v0.1 set [50, 56, 62, 69, 75]
rc=4
```

Both of those are **committed fork fixtures**: `gui/sysw_cells_test.go`'s
`cellMs1` (74) and `backup/backup_test.go`'s `ms13cash…` (48). Go accepts all
three candidates as `ClassCodex32Secret`, measured through the firmware's own
code:

```
len=50 class=ClassCodex32Secret codex32err=<nil> id="entr"
len=48 class=ClassCodex32Secret codex32err=<nil> id="cash"
len=74 class=ClassCodex32Secret codex32err=<nil> id="leet"
```

`me sysw pack` refuses both of the fork's at rc=4, and the message it prints is
the *generic* one — "not a form this container can place … Descriptors and
addresses are not yet classifiable here" — which sends the operator looking at
the classifier when the actual reason is a length set. The specific message only
surfaces via `me seal --seal-secret`.

**Scope of this negative:** I tested three ms1 strings. I did not enumerate
`ms_codec`'s accepted set beyond reading it out of that error. Rust-primary
applies if this is to be closed: the length rule is normative and lives in Rust.

### 4.3 NEW — `me sysw show` names no secret record at all

The device's Payload Digest screen tells the operator to run `me sysw show
<file>`. For four of the eight classes that command prints **nothing about the
records**:

```
$ me sysw show chain-seed.bin
sealed:   false
pub_len:  93
ct_len:   0
identity: 07413e737b629b711cb2c00d7629b04340bad2707b004d69d05e0038184dbb36
[stderr] digest:   f36e 9900 a235 0b1e 1c5f c580 2623 32b9
```

Same for `chain-text`, `chain-pass`, `chain-codex32`. And on the cards payload it
lists **9 of 10** records — every mk1, and no line at all for record 9, a
plaintext BIP-39 mnemonic:

```
$ me sysw show cmd/emu/sysw_cards_payload.bin | grep -c "^public record"
9
$ python3 -c "...split pub section..."
n records 10
9 93 abandon abandon abandon …
```

`print_mdmk_confirmation` iterates the records and `continue`s on anything that
is not `Class::MdMk`; `print_mt_confirmation` adds `Mt`/`Tx`. Nothing prints a
line for a secret class. `me sysw pack` *does* warn at pack time, and the device
*does* raise F1 at load time — so the gap is specifically in the command the
device's own screen sends the operator to. Recorded, not fixed.

### 4.4 NEW — the payload-card picker seats the last cards with no question

`buildCosignerPickFlow` asks about a payload card only while a choice remains:

> once the cards that remain are exactly the slots that remain, they are all
> taken without asking — a question with one possible answer is not a choice, and
> asking it is how an operator skips their way into an under-supply that was
> never real.

Measured: with 4 cards and 2 open slots the walk taps SKIP, SKIP and cards 3 and
4 are seated silently. This is deliberate and documented at the function, and it
cost the walk one wrong guess to discover (the first draft expected `Use payload
card 3 of 4?`). The chain now asserts the `Key sources` review names *payload
card 3* and *payload card 4* and does **not** name 1 or 2.

Worth noting for a journey pass, not filed as a defect: with fingerprints omitted
— this walk's own choice, and the screen's default — that review identifies each
cosigner by **ordinal only** (`@1 a cosigner: payload card 3, taken as
supplied`). The ordinal is the whole of what an operator can check there. Which
keys actually landed is settled downstream, by the md1 the chain pins.

### 4.5 A harness defect, found and fixed here

`click()` only **queues** an event; events are consumed on the UI side of
`iter.Pull` and nothing advances that but `frame()`. The accept tap after a
finished cut therefore never landed, `Engrave` reported "not completed", and
`backupSeedStringFlow`'s `for { NewEngraveScreen(…).Engrave(…) }` opened a fresh
engrave job. The codex32 chain then blocked there until the test timed out —
**twenty seconds after every assertion in it had already passed**, which is the
shape that produces a green-looking test suite that hangs CI. Diagnosed from the
timeout's goroutine dump, not guessed:

```
seedhammer.com/gui.newEngraverJob(...)
seedhammer.com/gui.NewEngraveScreen(...)
seedhammer.com/gui.backupSeedStringFlow(...)
seedhammer.com/gui.engraveCodex32(...)
```

Fixed by pumping frames after the accept click, with the measurement written at
the fix. Note in passing, **not changed**: `backupSeedStringFlow`'s loop has no
`ctx.Done` check, unlike `backupWalletFlow`'s, which is why the block landed
there and not in the seed chain.

---

## 5. What these chains do NOT prove

Stated in the test files' own headers so the limits travel with the code.

- **No hardware.** `testEngraver` accepts the stepper stream and digests it.
  Nothing here says a motor turned or that steel was cut.
- **The screens are text, not pixels.** `op.Drawer.ExtractText` walks the op
  tree, so the frames show *what the device says*, not *how it looks*. A legend
  rendered off the panel passes.
- **The extractor concatenates ops with no separator**, so the screen's
  `e7e5 152f …` arrives as `e7e5152f…`: the digest comparison is on the 32
  nibbles and **cannot see the grouping**.
- **The plate is a build of the plan, not a capture of the cut** — except for the
  free-text and password chains, which capture the finished `Plate` (or the exact
  arguments it was built from) through production hooks and say so. The others
  build it from the value the flow received, and are bound to the walk by that
  value plus the screens the walk asserted.
- **A golden match is an equality of INPUTS carried by two routes**, not two
  independent renders: both sides call the same plate builder. It catches a
  record arriving different from the constant — a changed encoder, classifier or
  decode. It cannot catch a plate builder wrong in the same way for both.
- **The fixtures are pinned, not live.** The default suite does not run `me`;
  `gui/chain_fixture_live_test.go` (tag `oraclelive`) does and is the only thing
  that catches drift.
- **The walks press buttons where a finger would do.** Most steps use synthesized
  `Button`/`Down` events. SeedHammer II has no directional buttons, so a screen
  wired to `ButtonFilter` alone is dead on the machine and green here. The
  free-text and password chains use real hit-tested taps for their pickers; every
  step that uses `click()` is a step the chain does **not** prove a finger can
  reach.
- **Unsealed only.** `me sysw pack` is deterministic for the unsealed variant and
  not for the sealed one, so no fixture here can be byte-pinned as sealed.
- **`chain-mdmk` compares ONE plate of nine** against a golden. The other eight
  are *counted* and *named* by the engraved census, not compared knot-for-knot.

---

## 6. The two classes with no chain, and what would unblock them

Both were re-measured here rather than inherited from the recon.

```
$ me sysw pack --in <bech32 address>    --no-passphrase --out /tmp/x ; echo rc=$?
rc=4
$ me sysw pack --in <wsh descriptor>    --no-passphrase --out /tmp/x ; echo rc=$?
rc=4
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

**A class that cannot enter a payload is not an element of one**, so neither has a
chain and neither can have one. This is not a walk gap; it is a classifier gap,
and it is upstream:

- **`Descriptor`** — three programs admit it (Bundle, Multisig, WalletPolicy) and
  nothing can deliver it. Unblocking needs a descriptor parser in the **Rust
  primary** (`sysw::classify`), with vectors, then a port — the Rust-primary rule
  binds. `me seal`'s §6.4 lowercase rule additionally refuses every descriptor by
  construction, since an `xpub` has uppercase, so that ruling is needed too. More
  work than all five chains here combined.
- **`Address`** — `grep -c ClassAddress gui/sysw_admit.go` = 0. No program would
  consume one even if the classifier grew an address decoder. This row should be
  closed as "cannot be engraved by nature of the admission table" rather than
  left open as a gap.

---

## 7. Validation surface

```
go test ./gui/ -run TestChain -count=1 -vet=off          ok 43.3s
                                                          12 top-level, 9 subtests, all PASS
scripts/gui-shard-test.sh ./gui/ 24                       1006 top-level tests
                                                          partition verified exhaustive: 1006 == 1006
go test <every package but ./gui>                          52 ok, 20 with no test files, rc=0
go build ./...                                             clean
ME=… go test -tags oraclelive ./gui -run TestChainFixtures 8/8 fixtures ok vs me 0.7.0
GO=… ./scripts/chain-mutation-check.sh                     12/12 killed, tree clean
```

The `gui` count moved 998 → 1006, which is exactly the eight tests added
(5 class chains + `TestChainFixtureRecordsMatchTheGoldenConstants` +
`TestChainMdMkFixtureIsTheEmulatorsOwnPayload` + `TestChainPlateGoldens`).

**Pre-existing, not caused by this work, not fixed here:** `go vet ./gui/` reports
`testing.ArtifactDir requires go1.26 or later (file is go1.25)` at
`gui/freetext_sizeproof_golden_test.go:111` and `gui/transaction_golden_test.go:104`.
`go.mod` says `go 1.25.10`; CI's `setup-go` says `1.26`. `go test` runs a vet
subset that excludes `stdversion`, so tests are unaffected — but the
`go vet -tags oraclelive ./...` step the vendored-vectors comment relies on would
trip on it. It did not block this work; `-vet=off` was used throughout, as the
recon's own commands do.

---

## 8. Files touched

Branch `walk/payload-chain`, one commit on top of `9c5c066`. Nothing pushed.

```
gui/chain_walk_test.go                       harness parameterised by flow; file-backed
                                             fixtures; F1 screens in ingest(); touch helpers
gui/chain_class_walk_test.go       NEW       the five class chains + two binding tests
gui/chain_plate_goldens_test.go    NEW       the goldens, recorded from Go constants
gui/chain_fixture_live_test.go               the oraclelive audit learned the file case
gui/testdata/chain/chain_payloads.json       5 new entries (4 blobs + 1 file reference)
gui/testdata/chain-seed.bin        NEW       3,344 B
gui/testdata/chain-codex32.bin     NEW       3,707 B
gui/testdata/chain-text.bin        NEW       1,333 B
gui/testdata/chain-pass.bin        NEW       1,728 B
gui/testdata/chain-mdmk-md1-1.bin  NEW       6,291 B
scripts/gen-chain-fixtures.sh                emit_file, and the five new fixtures
scripts/chain-mutation-check.sh    NEW       the both-directions mutation gate
```

**In `mnemonic-engrave`:** this report only. No source touched.
