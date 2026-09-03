# IMPLEMENTATION PLAN — composer S4: the journey EXECUTED, and the device acceptance

**STATUS: DRAFT 2026-09-03, R0 ROUND 0 FOLDED. Task 0 EXECUTED; every host-oracle
value in §2 is measured, not transcribed. R0 round 0 = one journey lens (opus,
`composer-S4-plan-R0-r0-journey.md`, 1C/12I/5M/2N -- every finding was in this
plan's expected values, none in the shipped code; the keyed arm's oracle was
confirmed on the Go harness byte for byte). Folded here; the Critical was the
keyless arm's md1 pinned in `md encode`'s UNCHUNKED form while the device is
chunk-form-always. Round 1 = sonnet fold verification, pending.**

Baseline, measured 2026-09-03: seedhammer fork `main` `60bee002` (= the flashed
`bg60bee00`; boot judgement on machine power is the operator's, pending);
mnemonic-engrave `master` `ac2014e`; descriptor-mnemonic `main` `1dc8d409`, whose
`target/release/md` is the `md` used below (the installed `~/.cargo/bin/md` is
0.14.0 and has no `compose`); `me` 0.8.0 = this repo's `target/debug/me` (the
installed `me` is 0.7.0 and has no `key:`/`hash:`/`now:` classes); `ms` 0.16.0
and `mk` 0.13.0 at `~/.cargo/bin` (invoke every tool BY PATH: a bare `md` has
been `mkdir -p` in this shell). Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go/bin`; Python Playwright with
chromium-1208 is installed; `/scratch/code/shibboleth/.tmp` is the durable
scratch (the session scratchpad is wiped on exit).

## 0. What S4 is, and is not

S4 is the last stage of `STAGED_PLAN_wallet_policy_composer.md` (§S4): spec
§12 item 2 (the C8 journey EXECUTED on the emulator with a payload of `key:`,
`hash:`, `now:` records and a seed; the consent's ids and addresses compared
against `md`; the capture refusing to finish on a mismatch; the negative
control run), item 3 (the no-payload walk ending in a keyless-template
engrave whose md1 decodes with distinct-account origins), item 9 (the engrave
surface per journey), and §13 items 1 and 5. Every S3 plan closed with "the
emulator journey is S4's, and a plan may not close while this gate has never
run". Plus what no emulator can do: the live walk WITH the operator on the
device (`design/S4_journey_walk_2026-09-02.md`, paused at step 3) and ONE real
plate, decoded back on the host.

NOT in S4: NFC seating and on-device preimage derivation (spec §14); text/QR
descriptor plates and their census refusal (F-457); the 32-slot shape on the
device; a Full-mode secret plate anywhere but at the operator's word; changes
to shipped copy found on the way (F-462, F-463 -- filed, not fixed here).

## 1. Task 0 — DONE 2026-09-03: the three shipped drivers, run

The S3 plan's Task C2 Step 5 put a second `await tap(CONFIRM)` (the composer's
door) into `cmd/emu/shots_walletpolicy.js`, `shots_seating.js` and
`shots_tr_pathological.js` and could only build-and-count the edit. Run from
`/scratch/code/shibboleth/.tmp/s4-emu-regression.sh` against a fresh
`emu.wasm` (10,788,612 B) of fork main `60bee002`: `capture_walletpolicy.py`
exit 0 / 8 shots, `capture_seating.py` exit 0 / 8 shots,
`capture_tr_pathological.py` exit 0 / 9 shots. Each is a host-vs-device
comparison, so the door moved nothing those journeys prove. Recorded in the
walk record (`7a008a6`).

## 2. The fixture — one wallet, three implementations meet on it

**The composer payload (five records; nothing secret beyond BIP-39's published
vectors).** `key:` bodies are the hex of the bracketed text; `hash:` is the
digest itself; `now:` is the hex of `1788220800,905000` (2026-09-01 00:00 UTC,
height 905000, the S3 fixture's values); the seed is master B's words, sniffed
as ClassMnemonic (no prefix).

```
key:<hex of [73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf>
key:<hex of [73c5da0a/48'/0'/1'/2']xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk>
hash:abababababababababababababababababababababababababababababababab
now:<hex of 1788220800,905000>
legal winner thank year wave sausage worth useful legal winner thank yellow
```

Master A = BIP-39's "abandon ... about" vector, fingerprint `73c5da0a`; the
two xpubs are `ms derive --template bip48-p2wsh --account 0|1` and equal the S3
fixture's (`gui/composer_fixtures_test.go`). Master B = BIP-39's "legal winner
... yellow" vector, fingerprint `b8688df1`; at `m/48'/0'/0'/2'` `ms derive`
gives `xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX`.
**Two accounts of ONE master in one 2-of-2 path is deliberate** (r0 M-4): it is
C5's "one master, two accounts" case AND it fires §8g's first body at the
mapping review ("Slots @0 and @1 are the same seed. This path's 2-of-2 can be
satisfied by one person. Liana will refuse it."), so the capture photographs a
§8 warning no other fixture reaches. Nobody reading the record later should
take that warning for a defect.

Packed with `target/debug/me sysw pack --no-passphrase --in <records> --out
<bin>` (an operator-supplied `now:` wins, so nothing is auto-appended; with
neither `--in` nor argv the same lines are read from STDIN, and the stdin form
packs a byte-identical blob -- measured):
`me sysw show` prints `sealed: false`, `pub_len: 730`, records 0-1 "cosigner
key (key:)", 2 "sha256 hashlock (hash:)", 3 "pack time (now:) — 1788220800
(seconds), height 905000", record 4 the mnemonic (named in the sealing
warning), and **digest `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b`**.

**The keyed arm's shape (§12 item 2).** wsh; Path 1 = 2-of-2 (A@0, A@1);
Path 2 = 1 key (seed B) + a wait of 12960 blocks + the payload's hashlock.
Host:

```
md compose --wrapper wsh --path 2of2 --path '1of1,older=12960,sha256=abab...abab' --json
```

gives `template` `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pkh(@2/<0;1>/*),and_v(v:sha256(ab...ab),older(12960)))))`
with `template_with_origins` placing the UNSEATED @2 at `48'/0'/2'/2'` (md's
lowest-free rule -- and that is what the FIRST stub screen shows before
seating). **The device seats @2 from seed B at B's own account 0'** (§4f:
"each at its own hardened account by ordinal among the slots that master
fills"; measured on the harness by the r0 lens: `composerSeedAccountFor`
counts the earlier slots THIS master fills, 0 for @2), so the keyed policy's
@2 origin is `[b8688df1/48'/0'/0'/2']` and the host oracle is minted with THAT
origin:

```
md encode "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:pkh(@2/48'/0'/0'/2'/<0;1>/*),and_v(v:sha256(abab...abab),older(12960)))))" \
  --key @0=xpub6DkFA...KFrf --key @1=xpub6Dzhy...d6Vk --key @2=xpub6FQya...F8mX \
  --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a --fingerprint @2=b8688df1
```

Measured, and CONFIRMED on the device side by the r0 lens (a throwaway
harness walk to the consent, plus a direct recomputation): **7 md1 chunks**
(chunk 1
`md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv`);
`md inspect --in` of them: `wallet-descriptor-template-id
531ab9e1777f018ae53694387dd0d128`, `wallet-policy-id
4dd749a8372af515a61d7104faf944ef` (`wallet-policy-id-fingerprint 0x4dd749a8`),
`md1-encoding-id fb28698ee8bdbc18c6ee36598f2124fe`; the device's stubs are the
first four bytes of each id: `mk1 stub (template): 531ab9e1`, `mk1 stub
(policy): 4dd749a8`; `md address --template <the same> <the same keys> --count
2` → receive
`bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l`,
`bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a`; `--change
--count 2` → `bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru`,
`bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9`.

**Form B's artifacts (§7f), also host-minted.** The template WITH fingerprints
(what the second stub screen and form B's first card carry; NOT the first stub
screen's unseated chunk set, which shares the Template-ID and puts @2 at
`2'`):

```
md encode "<the same origin template>" --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a --fingerprint @2=b8688df1 --force-chunked --group-size 0
```

→ 2 chunks, `chunk-set-id: 0x34c51`,
`md1fxnz3qs9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2shlte30qvuhvrq`
and `md1fxnz3qsw46h2at4w46h2at4w46h2at4w46h2msqqqv4qp0npeutks2tnchdq4ts6yd7yq5swf47peq533w`
(the r0 lens read the same two off the device). The three cards, both stubs
appended: `mk encode --xpub <x> --origin-fingerprint <fp> --origin-path <p>
--policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8` → @0 (A, `m/48'/0'/0'/2'`)
2 chunks, @1 (A, `m/48'/0'/1'/2'`) 3 chunks, @2 (B, `m/48'/0'/0'/2'`) 2 chunks
-- the device's census counts one plate per card and the same chunk counts.

**The keyless arm's shape (§12 item 3) -- also the plate the operator cuts.**
tr; ONE path 2-of-3; no lock, no hash. `md compose --wrapper tr --path 2of3`
→ `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))`.
**THE DEVICE IS CHUNK-FORM-ALWAYS** (`md.Composed.Chunks` is `split(...)`,
"as the primary's force_chunked vectors are"), and a template this short
encodes UNCHUNKED on the host by default -- r0 C-1: the first draft of this
plan pinned the 47-character unchunked string, and `md verify` and `md
inspect` accept BOTH forms identically (same template, same
`wallet-descriptor-template-id`, same `md1-encoding-id b0884601...`), so no
verify step can tell them apart; only the byte comparison can. The oracle is
therefore minted with `--force-chunked --group-size 0`:

```
md encode "<the tr origin template>" --force-chunked --group-size 0
```

→ ONE chunk, `chunk-set-id: 0xb0884`, **56 characters**:
**`md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3`** (the r0 lens
read exactly this off the device's census). `md inspect`:
`wallet-descriptor-template-id e0863d3ccac31a64d3b5e14b85ccd6c0` (the device's
`mk1 stub (template): e0863d3c`), origins `@0: m/48'/0'/0'/3'`, `@1:
m/48'/0'/1'/3'`, `@2: m/48'/0'/2'/3'`; `md decode` prints the template and the
three origins. Host, emulator and device meet on this one 56-character string.
Every later plan that mints a short-policy oracle passes `--force-chunked`.

## 3. Tasks

### Task 1 — fork: the THIRD emulator payload (`cmd/emu`)

The records blob has no `key:`/`hash:`/`now:` record and the cards blob's
digest is pinned and photographed, so neither may change (the reasoning in
`cmd/emu/sysw_cards_payload.go`). Add, on the cards blob's pattern (fork commit
`3ea08f9`):

- `cmd/buildpayloadcomposer/main.go`: emits §2's five records to stdout. It
  derives A@0 and A@1 through the device's own path (`bip39.MnemonicSeed` →
  `hdkeychain.NewMaster` → `bip32.Derive` → `Neuter`, as `cmd/buildpayloadcards`
  does) and REFUSES to emit if either differs from the `ms`-derived xpub pinned
  in the file -- the cross-implementation check is inside the generator, not
  in a comment.
- `cmd/emu/sysw_composer_payload.bin` = `go run ./cmd/buildpayloadcomposer |
  target/debug/me sysw pack --no-passphrase --out
  cmd/emu/sysw_composer_payload.bin` (`me sysw pack` reads the records from
  STDIN when neither `--in` nor argv is given; `--in -` is NOT a stdin
  spelling, r0 M-2); `cmd/emu/sysw_composer_payload.go` (`//go:build js`,
  `//go:embed`, `const syswComposerDigest`, the record inventory stated in the
  header as the cards blob's is).
- `cmd/emu/sysw_composer_payload_host_test.go`: the digest recomputed by
  `sysw.Open` + `sysw.PublicDataHash` equals the pinned constant; the inventory
  is exactly 2 ClassKey, 1 ClassHash, 1 ClassNow, 1 ClassMnemonic, in that
  order; `me sysw show`'s digest line equals it.
- `cmd/emu/platform.go` `SyswReader`: `case "composer"`; `cmd/emu/walk_js.go`
  `shSysw`: accept `"composer"` (and say so in its usage string).

Run: `cd /scratch/code/shibboleth/wt-composer-s4-emu && CGO_ENABLED=0 go test -count=1 ./cmd/emu/ && GOOS=js GOARCH=wasm go vet ./cmd/emu/ && gofmt -l cmd/`
Expected: `ok`; the confinement test (`embed_confinement_test.go`) passes
UNCHANGED, because it discovers the new `//go:embed` itself; vet exit 0; gofmt
prints nothing. Digest = `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b` (a
different digest means the generator's bytes differ from §2's records file --
diff them, do not re-pin).

### Task 2 — engrave: the host half, `design/journeys/transcript_composer.sh`

On `transcript_walletpolicy.sh`'s pattern (every block is real stdout+stderr
with its exit code; `mapfile`, never `$(cat)`), writing `out/composer/`:
`records.txt` (byte-identical to the generator's stdout -- **gate:** `diff
<(go run ./cmd/buildpayloadcomposer) out/composer/records.txt` is empty),
`payload.bin` and `payload.digest.txt` (`me sysw show`), `compose.json`,
`keyed.template` (the origin-notated template with @2 at B's account 0'),
`keyed.md1.txt` (7 chunks), `keyed.id.txt` (`md inspect`), `keyed.receive.txt`,
`keyed.change.txt`, `keyed-template.md1.txt` (the fingerprinted template, 2
chunks, `--force-chunked --group-size 0`), `cards/slot0.mk1.txt`,
`cards/slot1.mk1.txt`, `cards/slot2.mk1.txt` (both stubs `531ab9e1` and
`4dd749a8`, which are the ids' first four bytes -- confirmed on the device by
the r0 lens, so the host mints them without waiting for an emulator run),
`keyless-tr.template`, `keyless-tr.md1.txt` (ONE chunk, `--force-chunked
--group-size 0`), `keyless-tr.id.txt`.

Run: `./transcript_composer.sh > transcript_composer.txt; echo $?`
Expected: exit 0; the values of §2, verbatim -- including the 56-character
keyless string and the two-chunk fingerprinted template.

### Task 3 — fork + engrave: the driver, `cmd/emu/shots_composer.js` + `design/journeys/capture_composer.py`

On `shots_tr_pathological.js` / `capture_tr_pathological.py`'s pattern: the
driver ASSERTS (it is handed `expect` and throws on disagreement), the capture
exits non-zero unless every shot arrived AND the comparison passed, and
`--prove-it-can-fail` corrupts one character of one expected address and
exits 0 only if the walk caught it. Two arms, `--arm keyed|keyless|both`,
each reloading the page (no arm inherits the other's device state).

**The engrave loop** is `walk_s3_nested.js`'s (hold via `shPress`/`shRelease`
longer than `gui.confirmDelay`; toolpath-stall detection; an unrecognised
screen STOPS the walk) with two changes the r0 lens measured (I-9): the
composer offers NO verify step, and after the last plate `bundleEngrave` shows
a modal `Bundle engraved. Also hand-engrave your ms1 share(s) - they are never
sent over NFC.` (shipped copy, shown even on a watch-only run: F-463) and the
flow returns to the DOOR. So the handler list gains a fourth entry (`match:
"Bundleengraved"`, `act: "confirm"`) and the loop terminates on the door's
`Build a new policy`, never on a verify offer. The census is
`JSON.parse(window.shToolpath.strings())`, which is **one entry per PLATE,
newline-joined** (`notifyPlateText`, r0 I-2): the driver splits each entry on
`"\n"`, concatenates in order, and compares that flat list against the host
file byte for byte -- AND asserts the entry count equals the census screen's
plate count (the half that catches a packing change).

**Paged screens wrap** (`composerReadScreen`, `composerPickScreen`: after the
last page comes page 0 again; r0 M-3): the driver records the first page's
text and stops paging when it recurs, as `readAllPages` in
`shots_walletpolicy.js` already does; the number of shots per screen is an
assertion, never a loop bound.

**Notation**: the stub screen prints origins as `m/48h/0h/0h/2h` (a
`bip32.Path`); the mapping review, the census and the card summaries print
`m/48'/0'/0'/2'` (r0 M-1; F-462). Frames are compared with spaces stripped
(`ExtractText`), as the shipped walks do.

The itineraries below are the plan's claim about the shipped screens, every
row either verified or corrected by the r0 lens on the Go harness against
`60bee002` (the emulator arm could not run before Task 1 exists); "shot" marks
a screenshot the capture keeps.

**Keyed arm.**

| # | wait for | do | assert / shot |
| --- | --- | --- | --- |
| 1 | `systemwide payload is present` | BACK (the boot offer's reader is resolved before a script can speak: `walk_s4_gate.js`) | `SeedHammer` |
| 2 | -- | `shSysw("composer")`; carousel to `Load Payload`; CONFIRM | `Payload Digest` -- the screen's digest equals `payload.digest.txt` (shot) |
| 3 | `Payload Digest` | CONFIRM | `Payload Warnings` (F1: "A SECRET is stored unencrypted in flash.") |
| 4 | `Payload Warnings` | CONFIRM | `Keep this payload loaded?` → KEEP (CONFIRM) → `Load Payload` |
| 5 | carousel to `Wallet Policy`; CONFIRM | -- | the door: Lead `Keys loaded: 2, plus 1 seed.`; rows `Scan cards`, `Build a new policy` and NO `From payload` (the payload holds no descriptor/md1/mk1) (shot) |
| 6 | select `Build a new policy` | CONFIRM | `Which script?`: `Taproot (tr)`, `Segwit (wsh)`, `Nested (sh-wsh)`, `Legacy (sh)` |
| 7 | `Segwit (wsh)` | CONFIRM | `Start from?` with row 0 `Build my own paths` (W-1), then the six presets (shot) |
| 8 | row 0 | CONFIRM | `Spend paths`: live line `slots: 0 / keys available: 2` (no seed line: the payload's seed is not a composer SOURCE until typed in at seating, r0 I-5); rows `Add a spend path`, `Change the script`, `Done` |
| 9 | `Add a spend path` → `What can spend on this path?` (`Keys`, `A hash, no keys`) → `Keys` → `Path 1: how many keys?` 2 → `Path 1: how many must sign?` 2 | -- | `Path 1: 2-of-2`; live line `slots: 2 / keys available: 2` |
| 10 | `Add a spend path` → `Keys` → 1 → 1 | -- | `Path 2: 1 key` (a 1-key path is rendered `1 key`, never `1-of-1`, r0 I-6) |
| 11 | open Path 2 (`Keys`, `Time lock`, `Hash lock`, `Remove path`, `Move up`) → `Time lock` → `Path 2 lock` / `What kind of time lock?` → `After a wait` → `Measured how?` → `Blocks` → `How many blocks?` (`1 to 65535 blocks`) → digits 12960 | -- | the echo screen is ONE line, `12960 blocks (about 90.0 days)` -- a relative lock carries no `now:` bound line (r0 I-7) (shot); the path row reads `Path 2: 1 key + 12960 blocks` |
| 12 | open Path 2 → `Hash lock` → `Path 2 hash` / `Which hash?` rows `hash 1  abababab..abababab`, `Type 64 hex`, `No hash lock` → row 0 | -- | the §8i modal draws FIRST: `The hash must be SHA-256 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. A hash of the passphrase itself can never be spent.` (shot; r0 I-8) → CONFIRM → the path row reads `Path 2: 1 key + hash + 12960 blocks` |
| 13 | `Done` | -- | straight to the `Template` screen: the key-order question is asked ONLY when the list has one path (`composerSortedIsLegal`; r0 I-3). Page 0: `Template-ID: 531ab9e1777f018ae53694387dd0d128`, `mk1 stub (template): 531ab9e1`, the `mk encode --xpub <xpub> ...` lines; page 1: `Slot @0 expects a key at m/48h/0h/0h/2h`, `@1 ... m/48h/0h/1h/2h`, `@2 ... m/48h/0h/2h/2h` (unseated: md's lowest-free account); then page 0 again (shots: 2) |
| 14 | CONFIRM | -- | the pick list at once (no `Seat keys into this template?` -- that ChoiceScreen belongs to a composition with NO sources, r0 I-4): `Seat keys` / `Slot @0, Path 1 key 1 of 2: choose a key` rows `73c5da0a m/48h/0h/0h/2h`, `73c5da0a m/48h/0h/1h/2h`, `Type a seed`, `Leave unseated` (shot) → row 0 → `Slot @1, Path 1 key 2 of 2: choose a key` (three rows, A@0 gone) → the `73c5da0a m/48h/0h/1h/2h` row |
| 14a | `Slot @2, Path 2 key 1 of 1: choose a key` rows `Type a seed`, `Leave unseated` | `Type a seed` → `Where from?` (`FROM PAYLOAD`, `TYPE IT`, and `SCAN` on the emulator, which reports NFC) → `FROM PAYLOAD` → `Seed for the policy` / `Source: the systemwide payload` → CONFIRM → `Add a BIP-39 passphrase?` (`Skip`, `Add passphrase`) → `Skip` | the pick list re-drawn with `seed 1  (any slots)` (shot) → pick it |
| 15 | `Key mapping` page 0: `@0: 73c5da0a m/48'/0'/0'/2'`, `@1: 73c5da0a m/48'/0'/1'/2'`, `@2: b8688df1 m/48'/0'/0'/2'`, `This device cannot confirm a key was derived at the origin it declares.` (shot); page 1: `SAME SEED, SAME PATH` / `Slots @0 and @1 are the same seed. This path's 2-of-2 can be satisfied by one person. Liana will refuse it.` (§8g, deliberate -- §2; shot; r0 I-11) | CONFIRM after the last page | -- |
| 16 | `Template` again | -- | `Template-ID: 531ab9e1...`, `mk1 stub (template): 531ab9e1`, `Policy-ID: 4dd749a8372af515a61d7104faf944ef`, `mk1 stub (policy): 4dd749a8`, `Stamp BOTH stubs on each key card:` / `--policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8`; later pages `Slot @0: 73c5da0a m/48h/0h/0h/2h` ... `Slot @2: b8688df1 m/48h/0h/0h/2h` (shots per page, stop on wrap) |
| 17 | CONFIRM | -- | consent `Review`, paged: `Path 1: 2-of-2`; `Path 2: 1 key`, `12960 blocks (about 90.0 days)`, `hash abababab..abababab`; `Policy-ID: 4dd749a8...`; pages 2-3: `Receive 0: bc1q8cf5g5f...`, `Receive 1: bc1qkd729k2...`, `Change 0: bc1q9ms8tdk...`, `Change 1: bc1q3cs923r...` -- **THE COMPARISON**: both ids and all four addresses, whole (shots per page) |
| 18 | CONFIRM after the last page → `Nothing outside this device has checked this policy.` ... `Hold button to confirm.` | HOLD | `What to engrave` / `Which form?`: `The policy itself`, `Template plus key cards` |
| 19a | `The policy itself` | CONFIRM | `Engrave Mode` / `What to engrave?`: `Full (seed + keys)`, `Watch-only (keys)` (asked because a seed-seated slot exists, r0 I-1) → `Watch-only (keys)` → `Plates To Cut`: `This engraves 2 plates.`, `md1 policy: 2 plates (the wallet policy, with its keys)` (7 chunks pack 5 + 2, F-423) (shot) |
| 20a | CONFIRM → the engrave loop (on every packed plate `Choose engraving` offers `TEXT ONLY` alone) | holds per plate; `Bundle engraved` → confirm; stop at the door | `shToolpath.strings()` has 2 entries; split on `"\n"` and flattened they equal `keyed.md1.txt`'s 7 chunks byte for byte; the door reads `Keys loaded: 2, plus 1 seed.` again |
| 19b | (second run of rows 1-18) `Template plus key cards` → `Watch-only (keys)` | -- | `This engraves 4 plates.`; `md1 template: 1 plate (key-less wallet policy)`; `mk1 key @0: 1 plate (m/48'/0'/0'/2')`; `mk1 key @1: 1 plate (m/48'/0'/1'/2')`; `mk1 key @2: 1 plate (m/48'/0'/0'/2')` -- rows @0 and @2 differ only by the `@i` (A's account 0' and B's account 0'; r0 M-5) (shot) |
| 20b | the engrave loop | -- | 4 entries: the template (2 chunks = `keyed-template.md1.txt`), @0 (2), @1 (3), @2 (2) = `cards/slot{0,1,2}.mk1.txt`, byte for byte after the split; each card `mk decode`s on the host to its xpub, fingerprint, origin and BOTH stubs |

**Keyless arm** (the state §12 item 3 names is NO payload loaded AND no region
present at the door: `composerDoorFlow` probes `SyswReader()`, and with the
emulator's default records blob unloaded the Lead is `A payload is in flash
but not loaded. Load it from the carousel first.`, r0 I-10 -- so the arm
switches the reader off after the boot offer, as `shots_operator.js` does):

| # | wait for | do | assert / shot |
| --- | --- | --- | --- |
| 1 | `systemwide payload is present` | BACK; then `shSysw("none")` | `SeedHammer` |
| 2 | carousel to `Wallet Policy`; CONFIRM | -- | the door: Lead `No keys loaded. This builds a key-less template.`; rows `Scan cards`, `Build a new policy` (shot) |
| 3 | `Build a new policy` → `Taproot (tr)` → `Build my own paths` → `Add a spend path` → `Keys` → 3 → 2 → `Done` → `Sorted keys, or your order?` (asked: ONE path) → `Sorted (usual)` | -- | `Template`: `Template-ID: e0863d3ccac31a64d3b5e14b85ccd6c0`, `mk1 stub (template): e0863d3c`; page 1: `Slot @0 expects a key at m/48h/0h/0h/3h`, `@1 ... m/48h/0h/1h/3h`, `@2 ... m/48h/0h/2h/3h` (shots: 2) |
| 4 | CONFIRM → `Seat keys into this template?` rows `Engrave a key-less template`, `Type a seed` (drawn because there are NO sources, r0 I-4) | row 0 | consent `Review`: `Path 1: 2-of-3`; `KEY PATH: NONE (NUMS)` / `Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449).`; `Template-ID: e0863d3c...`; `mk1 stub (template): e0863d3c`; page 1: `Keyless template - no addresses.` / `Verify off-device.` (shots per page) |
| 5 | CONFIRM after the last page → `Nothing outside this device ...` | HOLD | the modal `No slot is seated, so there is a template and nothing else.` (a modal, not a picker) → CONFIRM → `Plates To Cut`: `This engraves 1 plate.`, `md1 template: 1 plate (key-less wallet policy)` (shot) |
| 6 | CONFIRM → `Choose engraving` (`TEXT + QR` is row 0; `QR ONLY` would give the operator a plate the SH2 can never read back) | row 0; the engrave loop, one hold; `Bundle engraved` → confirm; stop at the door | `shToolpath.strings()` == `["md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3"]` byte for byte -- the ONLY check that catches the unchunked substitution (§2); then on the host `md verify --template "<keyless-tr.template>" <that string>` exit 0 and `md decode` prints the three distinct-account origins |

Run: `cd design/journeys && python3 capture_composer.py --arm both; echo $?; python3 capture_composer.py --arm keyed --prove-it-can-fail; echo $?`
Expected: `0` and `0`; the second prints the caught message ("the device's
proof does not match the host's"). Then the three shipped drivers again
(Task 0's script): all exit 0.

### Task 4 — the live walk on the DEVICE, with the operator (no code)

Resumes `design/S4_journey_walk_2026-09-02.md` at step 3 on the keyless arm's
shape: at every step, what is in hand exactly, what the device does, what ELSE
the operator might do; each divergence refusal / warning / default /
not-our-concern / documentation only, and a change ONLY when the wrong outcome
is worse than saying nothing. Fixes batch into one fold on a fork branch
(`composer-s4b`), each with a regression test that fails under its named
mutation, sonnet-verified, merged `--no-ff`, flashed via `~/bin/sh/sh2-flash`
at the operator's word.

Read the door's Lead FIRST and write it down: the Load Payload journey left a
region on this machine, so it may read `A payload is in flash but not loaded.
Load it from the carousel first.` rather than the key-less line (r0 I-10). A
keyless template needs no payload, so the walk continues either way; which
line it was is part of the record.

Then the plate, keyless tr 2-of-3, ONE plate:

1. The census must read `This engraves 1 plate.` and the stub screen's
   Template-ID `e0863d3ccac31a64d3b5e14b85ccd6c0` with `mk1 stub (template):
   e0863d3c` -- photographed before the hold. At `Choose engraving` take
   `TEXT + QR` (row 0), never `QR ONLY`: the device has no camera and the
   read-back below is by eye.
2. A single-character test cut first if the machine has been moved or idle
   (the Y-axis-play precedent: two software hypotheses failed before a loose
   screw was found).
3. ABORT on: any refusal or the §8q self-check; a census other than 1; the
   toolpath stalling; the first glyphs showing the plateau artefact (stop,
   check the axis screws, do not re-cut on the same plate).
4. Decode back: the operator reads the plate aloud and the string is typed
   into the host; it must equal
   `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` BYTE FOR BYTE
   (56 characters); `md verify --template "<keyless-tr.template>" <string>`
   exit 0 and `md inspect` template id `e0863d3c...` with origins at accounts
   0', 1', 2' are the round trip, but `md verify` also accepts the unchunked
   form, so the byte equality is the check that closes Part A of the walk
   record.

### Task 5 — Part B on the device (the operator's call)

Writes the payload region (reversible: `me sysw wipe`). `target/debug/me sysw
pack --no-passphrase --region --in out/composer/records.txt --out
out/composer/payload-region.bin`; the machine in BOOTSEL; from the fork
checkout `nix develop --command picotool load --verify -t bin -o 0x10D00000
out/composer/payload-region.bin` (the Load Payload journey's step 11); boot →
LOAD → the digest `dbe9 e774 ...` compared against `payload.digest.txt`; the
keyed itinerary of Task 3 walked by hand with the consent's ids and addresses
read against `out/composer/keyed.*`; engrave form B (4 plates: template +
three cards) or A (2 plates) in Watch-only -- `Full (seed + keys)` adds a
BEARER plate of master B's seed and is not taken here; how many plates to cut
is the operator's; abort criteria as Task 4. Not required for S4's exit: Task
3 is §12 item 2's gate; this is acceptance beyond the spec.

### Task 6 — the records

Spec §12 items 2, 3 and 9 each gain `EXECUTED 2026-09-xx` with the run (the
capture's exit codes and the shot count); §13 items 1 and 5 discharged or their
residue filed with an owning phase; `STAGED_PLAN_wallet_policy_composer.md` §S4
STATUS; `design/journeys/README.md` row + `build_pdf_composer.py` (the PDF the
staged plan promises, regenerable by its own README); `design/FOLLOWUPS.md`:
F-460 checked against the code (the Multisig Build comment is present or the
item is closed), F-461's "revisit at S4's journey run" answered (the use-site
arm stays unreachable: say so), F-462 (the `h`/`'` notation split) and F-463
(the ms1 reminder after a watch-only composer engrave) left filed with their
owning phase; the walk record and the continuity file.

## 4. Gates, and who runs them

- **Fork diff** (Tasks 1, 3): `CGO_ENABLED=0 go test -count=1 ./cmd/emu/`;
  `GOOS=js GOARCH=wasm go vet ./cmd/emu/`; `gofmt -l cmd/`; at merge the full
  `go test -timeout 20m ./...` and the sharded gui run. The firmware is NOT
  touched (`cmd/emu` and `cmd/buildpayloadcomposer` are outside `cmd/controller`):
  the size recipe must still read `1,579,940 B flash / 62,800 B RAM` at the
  merge, and a different number is a finding.
- **Engrave diff** (Tasks 2, 3): `capture_composer.py --arm both` exit 0;
  `--prove-it-can-fail` exit 0; the three shipped drivers exit 0;
  `transcript_composer.txt` regenerated by its script.
- **Review**: one opus whole-diff execution review over BOTH diffs (brief in
  `design/agent-briefs/`, the S3 shape: counterexamples, mutate the driver's
  assertions -- a comparison that cannot fail is Critical, and the specific
  mutation to run is substituting the unchunked keyless string, which every
  `md` verb accepts --, what the diff made false elsewhere, CI gates as CI
  runs them); fold; sonnet verification; fork merge `--no-ff` + push + watch
  `test.yml`; engrave push via `scripts/push-via-staging.sh` with master
  FROZEN.
- **R0 for THIS plan**: round 0 = one journey lens (opus; brief
  `design/agent-briefs/composer-S4-plan-R0-journey-brief.md`; report
  `design/agent-reports/composer-S4-plan-R0-r0-journey.md`, 1C/12I/5M/2N,
  folded above); round 1 = sonnet fold verification (did each finding land;
  did the fold introduce nothing). A clean round 1 closes R0 (lenses: journey
  on the shipped code, fold verification); the implementer is dispatched
  immediately after.

**Coverage line for the reviewer.** Machine-verified already: every §2 value
(`md`, `me`, `ms`, `mk` runs on 2026-09-03, incl. the chunked keyless string,
the fingerprinted template, the card chunk counts and the stdin pack); Task 0's
run; the emulator JS surface (`shTap`, `shPress`, `shRelease`, `shSysw`,
`shScreen`, `shNFC`, `shToolpath`, `shPace` -- `cmd/emu/*_js.go`); the
payload-load sequence (`walk_s4_gate.js`); every itinerary row's needle, order
and Expected on the Go harness against `60bee002` (the r0 lens, with quoted
frames). NOT verified, because no harness reaches it before Task 1 exists:
the emulator-only rows (the boot offer, `Load Payload`, `shSysw`), the `SCAN`
row at `Where from?` under the emulator's NFC feature, the exact rows of
`Choose engraving` on a packed plate, the consent's page count, and that
`shToolpath.strings()` splits exactly as `notifyPlateText` joins.

## 5. Order and ownership

Tasks 1 → 2 → 3 are ONE implementer (opus, UC off) in two worktrees
(`/scratch/code/shibboleth/wt-composer-s4-emu` off fork main, branch
`composer-s4-emu`; `/scratch/code/shibboleth/wt-engrave-s4-emu` off master,
branch `composer-s4-emu`), reporting to
`design/agent-reports/composer-S4-implementation-report.md`. Task 4 proceeds
live with the operator meanwhile (different artifacts, no dependency). Task 5
at the operator's word. Task 6 last, by the controller.
