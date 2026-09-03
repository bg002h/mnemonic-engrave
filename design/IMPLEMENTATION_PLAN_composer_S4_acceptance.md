# IMPLEMENTATION PLAN — composer S4: the journey EXECUTED, and the device acceptance

**STATUS: DRAFT 2026-09-03. Task 0 EXECUTED; every host-oracle value in §2 is
measured, not transcribed; R0 = ONE journey lens (opus), pending.**

Baseline, measured 2026-09-03: seedhammer fork `main` `60bee002` (= the flashed
`bg60bee00`; boot judgement on machine power is the operator's, pending);
mnemonic-engrave `master` `7a008a6`; descriptor-mnemonic `main` `1dc8d409`, whose
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
device; a Full-mode secret plate anywhere but at the operator's word.

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

Packed with `target/debug/me sysw pack --no-passphrase --in <records> --out
<bin>` (an operator-supplied `now:` wins, so nothing is auto-appended):
`me sysw show` prints `sealed: false`, `pub_len: 730`, records 0-1 "cosigner
key (key:)", 2 "sha256 hashlock (hash:)", 3 "pack time (now:) — 1788220800
(seconds), height 905000", record 4 the mnemonic (named in the sealing
warning), and **digest `dbe9 e774 e9a4 9231 0b62 626c 2b41 cf4b`**.

**The keyed arm's shape (§12 item 2).** wsh; Path 1 = 2-of-2 sorted (A@0,
A@1); Path 2 = 1-of-1 + a wait of 12960 blocks + the payload's hashlock. Host:

```
md compose --wrapper wsh --path 2of2 --path '1of1,older=12960,sha256=abab...abab' --json
```

gives `template` `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pkh(@2/<0;1>/*),and_v(v:sha256(ab...ab),older(12960)))))`
with `template_with_origins` placing the UNSEATED @2 at `48'/0'/2'/2'` (md's
lowest-free rule). **The device seats @2 from seed B, at B's own account 0'**
(§4f: "each at its own hardened account by ordinal among the slots that master
fills"), so the keyed policy's @2 origin is `[b8688df1/48'/0'/0'/2']` and the
host oracle is minted with THAT origin:

```
md encode "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:pkh(@2/48'/0'/0'/2'/<0;1>/*),and_v(v:sha256(abab...abab),older(12960)))))" \
  --key @0=xpub6DkFA...KFrf --key @1=xpub6Dzhy...d6Vk --key @2=xpub6FQya...F8mX \
  --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a --fingerprint @2=b8688df1
```

Measured: **7 md1 chunks** (chunk 1
`md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv`);
`md inspect --in` of them: `wallet-descriptor-template-id
531ab9e1777f018ae53694387dd0d128`, `wallet-policy-id
4dd749a8372af515a61d7104faf944ef` (`wallet-policy-id-fingerprint 0x4dd749a8`),
`md1-encoding-id fb28698ee8bdbc18c6ee36598f2124fe`; `md address --template
<the same> <the same keys> --count 2` → receive
`bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l`,
`bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a`; `--change
--count 2` → `bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru`,
`bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9`. If the device
prints a different Policy-ID the capture FAILS, and the finding is either the
device's seed-account rule or this plan's reading of §4f -- both worth having.

**The keyless arm's shape (§12 item 3) -- also the plate the operator cuts.**
tr; ONE path 2-of-3; no lock, no hash. `md compose --wrapper tr --path 2of3`
→ `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))`;
`md encode` of it with no keys → ONE chunk
**`md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc`**; `md inspect`:
`wallet-descriptor-template-id e0863d3ccac31a64d3b5e14b85ccd6c0`, origins
`@0: m/48'/0'/0'/3'`, `@1: m/48'/0'/1'/3'`, `@2: m/48'/0'/2'/3'`; `md decode`
prints the template and the three origins. Host, emulator and device meet on
this one 46-character string.

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
  target/debug/me sysw pack --no-passphrase --in - --out
  cmd/emu/sysw_composer_payload.bin`; `cmd/emu/sysw_composer_payload.go`
  (`//go:build js`, `//go:embed`, `const syswComposerDigest`, the record
  inventory stated in the header as the cards blob's is).
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
`keyed.md1.txt`, `keyed.id.txt` (`md inspect`), `keyed.receive.txt`,
`keyed.change.txt`, `keyless-tr.template`, `keyless-tr.md1.txt`,
`keyless-tr.id.txt`, and the three mk1 cards the device will mint in form B
(`mk encode --xpub <x> --origin-fingerprint <fp> --origin-path <p>
--policy-id-stub <template stub> --policy-id-stub <policy stub>`; the two
stubs are what the device's stub screen prints -- the transcript takes them
from the emulator's first run and the driver then asserts the device's cards
equal them, so a stub the host cannot derive is still cross-checked).

Run: `./transcript_composer.sh > transcript_composer.txt; echo $?`
Expected: exit 0; the values of §2, verbatim.

### Task 3 — fork + engrave: the driver, `cmd/emu/shots_composer.js` + `design/journeys/capture_composer.py`

On `shots_tr_pathological.js` / `capture_tr_pathological.py`'s pattern: the
driver ASSERTS (it is handed `expect` and throws on disagreement), the capture
exits non-zero unless every shot arrived AND the comparison passed, and
`--prove-it-can-fail` corrupts one character of one expected address and
exits 0 only if the walk caught it. Two arms, `--arm keyed|keyless|both`,
each reloading the page (no arm inherits the other's device state). The
engrave loop is `walk_s3_nested.js`'s: hold via `shPress`/`shRelease` (longer
than `gui.confirmDelay`), toolpath-stall detection, an unrecognised screen
STOPS the walk, and the census is read from `JSON.parse(window.shToolpath.strings())`.

The itineraries below are the plan's claim about the shipped screens; the
needles marked ★ are taken from `gui/composer_join_test.go`
(`TestComposerWalkFromAKeyedPayloadReachesTheEngraveScreen`) and the string
tables in `gui/composer_*.go`; the cells marked ? are what the R0 journey lens
pins or corrects before the implementer runs.

**Keyed arm.**

| # | wait for | do | assert / shot |
| --- | --- | --- | --- |
| 1 | `systemwide payload is present` | BACK (the boot offer's reader is resolved before a script can speak: `walk_s4_gate.js`) | `SeedHammer` |
| 2 | -- | `shSysw("composer")`; carousel to `Load Payload`; CONFIRM | `Payload Digest` -- the screen's digest equals `payload.digest.txt` (shot) |
| 3 | `Payload Digest` | CONFIRM | `Payload Warnings` (F1: "A SECRET is stored unencrypted in flash.") |
| 4 | `Payload Warnings` | CONFIRM | `Keep this payload loaded?` → KEEP (CONFIRM) → `Load Payload` |
| 5 | carousel to `Wallet Policy`; CONFIRM | -- | the door ★: Lead `Keys loaded: 2, plus 1 seed.`; rows `Scan cards`, `Build a new policy` and NO `From payload` (the payload holds no descriptor/md1/mk1) (shot) |
| 6 | select `Build a new policy` | CONFIRM | `Which script?` ★ |
| 7 | `Segwit (wsh)` | CONFIRM | `Start from?` ★ with row 0 `Build my own paths` (W-1) (shot) |
| 8 | row 0 | CONFIRM | `Spend paths` list ★ `Add a spend path`; live line `keys available: 2` ★ plus the seed's `any slots` line ? |
| 9 | `Add a spend path` → `What can spend on this path?` ★ → `Keys` → `how many keys?` ★ 2 → `how many must sign?` ★ 2 | -- | `Path 1: 2-of-2` ★ |
| 10 | `Add a spend path` → `Keys` → 1 → 1 | -- | `Path 2: 1-of-1` |
| 11 | open Path 2 → `Time lock` → `What kind of time lock?` → `After a wait` → `Measured how?` → `Blocks` → digit pad `How many blocks?` 12960 → the §8c relative-blocks echo with the `now:` bound line | -- | the path line carries the wait ? (exact wording pinned by the lens) (shot of the echo) |
| 12 | open Path 2 → `Hash lock` → `Which hash?` → the payload's record row (`abababab...`) | -- | the path line carries the hash mark ? |
| 13 | `Done` → `Sorted keys, or your order?` ★ → `Sorted (usual)` | -- | `Template` screen ★, paged to the end: `Template-ID: 531ab9e1777f018ae53694387dd0d128` (origin-invariant, §7c), `mk1 stub (template): <8 hex>` (recorded for Task 2's cards), three `Slot @i expects a key at m/48'/0'/i'/2'` lines (shots per page) |
| 14 | CONFIRM → `Seat keys into this template?` ★ → yes | -- | `Slot @0, Path 1 key 1 of 2: choose a key` ★ over rows `73c5da0a m/48'/0'/0'/2'`, `73c5da0a m/48'/0'/1'/2'`, the seed row ?, `Type a seed`, `Leave unseated` |
| 15 | @0 ← A@0; @1 ← A@1; @2 ← the seed (no passphrase) | -- | `Key mapping` ★: `@0 73c5da0a`, `@1 73c5da0a` (one master, two accounts: C5), `@2 b8688df1 m/48'/0'/0'/2'`, the `cannot confirm` line ★ (shot) |
| 16 | CONFIRM | -- | `Template` again ★ with `Policy-ID: 4dd749a8372af515a61d7104faf944ef`, `mk1 stub (policy): <8 hex>` (= `0x4dd749a8` unless the lens says otherwise ?), `Stamp BOTH stubs` |
| 17 | CONFIRM | -- | consent, paged ★: `Path 1: 2-of-2`, `Path 2: 1-of-1` + the wait echo + `abababab`/`abababab`, `Policy-ID`, Receive `bc1q8cf5g5f...`, `bc1qkd729k2...`, Change `bc1q9ms8tdk...`, `bc1q3cs923r...` -- **THE COMPARISON**, all four addresses and both ids (shots per page) |
| 18 | CONFIRM → `Nothing outside this device` ★ | HOLD | `What to engrave` / `Which form?` ★: `The policy itself`, `Template plus key cards` |
| 19a | `The policy itself` | CONFIRM | `Engrave Mode` / `What to engrave?` (a seed-derived slot exists) ? → `Watch-only (keys)` → the census ★ `This engraves` 7 plates |
| 20a | CONFIRM → the engrave loop | holds per plate | `shToolpath.strings()` == `keyed.md1.txt` (spaces stripped), 7 strings, byte for byte |
| 19b | (second run) `Template plus key cards` → `Watch-only (keys)` | -- | census: 1 template plate + `mk1 key @0`, `@1`, `@2` ★ |
| 20b | the engrave loop | -- | `shToolpath.strings()` == the keyless template chunk(s) + three mk1 strings that `mk decode` to the slot's xpub, fingerprint, origin and BOTH stubs |

**Keyless arm** (SKIP the boot offer; no payload loaded is the state §12
item 3 names):

| # | wait for | do | assert / shot |
| --- | --- | --- | --- |
| 1 | `systemwide payload is present` | BACK | `SeedHammer` |
| 2 | carousel to `Wallet Policy`; CONFIRM | -- | the door: Lead `No keys loaded. This builds a key-less template.`; rows `Scan cards`, `Build a new policy` (shot) |
| 3 | `Build a new policy` → `Taproot (tr)` → `Build my own paths` → `Add a spend path` → `Keys` → 3 → 2 → `Done` → `Sorted (usual)` | -- | `Template`: `Template-ID: e0863d3ccac31a64d3b5e14b85ccd6c0`; `Slot @0 expects a key at m/48'/0'/0'/3'`, `@1 ... 1'/3'`, `@2 ... 2'/3'` (shots per page) |
| 4 | CONFIRM (no seating: no sources) | -- | consent: `Path 1: 2-of-3`; the §8f NUMS note; `Keyless template - no addresses.`; `Verify off-device.` (shot) |
| 5 | CONFIRM → `Nothing outside this device` | HOLD | `What to engrave`: `No slot is seated, so there is a template and nothing else.` → the census: 1 plate |
| 6 | the engrave loop | one hold | `shToolpath.strings()` == `["md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc"]`; then on the host `md verify --template "<keyless-tr.template>" <that string>` exit 0 and `md decode` prints the three distinct-account origins |

Run: `cd design/journeys && python3 capture_composer.py --arm both; echo $?; python3 capture_composer.py --arm keyed --prove-it-can-fail; echo $?`
Expected: `0` and `0`; the second prints the caught message ("the device's
proof does not match the host's"). Then the three shipped drivers again
(Task 0's script): all exit 0.

### Task 4 — the live walk on the DEVICE, with the operator (no code)

Resumes `design/S4_journey_walk_2026-09-02.md` at step 3 on the keyless arm's
shape (the device holds no payload today): at every step, what is in hand
exactly, what the device does, what ELSE the operator might do; each
divergence refusal / warning / default / not-our-concern / documentation only,
and a change ONLY when the wrong outcome is worse than saying nothing. Fixes
batch into one fold on a fork branch (`composer-s4b`), each with a regression
test that fails under its named mutation, sonnet-verified, merged `--no-ff`,
flashed via `~/bin/sh/sh2-flash` at the operator's word.

Then the plate, keyless tr 2-of-3, ONE plate:

1. The census must read 1 plate and the stub screen's Template-ID must read
   `e0863d3c...` -- photographed before the hold.
2. A single-character test cut first if the machine has been moved or idle
   (the Y-axis-play precedent: two software hypotheses failed before a loose
   screw was found).
3. ABORT on: any refusal or the §8q self-check; a census other than 1; the
   toolpath stalling; the first glyphs showing the plateau artefact (stop,
   check the axis screws, do not re-cut on the same plate).
4. Decode back: the operator reads the plate aloud; host `md verify --template
   "<keyless-tr.template>" <string>` exit 0 and `md inspect` template id
   `e0863d3c...` with origins at accounts 0', 1', 2'. A plate that verifies
   closes Part A of the walk record.

### Task 5 — Part B on the device (the operator's call)

Writes the payload region (reversible: `me sysw wipe`). `target/debug/me sysw
pack --no-passphrase --region --in out/composer/records.txt --out
out/composer/payload-region.bin`; the machine in BOOTSEL; from the fork
checkout `nix develop --command picotool load --verify -t bin -o 0x10D00000
out/composer/payload-region.bin` (the Load Payload journey's step 11); boot →
LOAD → the digest `dbe9 e774 ...` compared against `payload.digest.txt`; the
keyed itinerary of Task 3 walked by hand with the consent's ids and addresses
read against `out/composer/keyed.*`; engrave form B (1 template plate + 3
cards) or A (7 plates) -- how many plates to cut is the operator's; abort
criteria as Task 4. Not required for S4's exit: Task 3 is §12 item 2's gate;
this is acceptance beyond the spec.

### Task 6 — the records

Spec §12 items 2, 3 and 9 each gain `EXECUTED 2026-09-xx` with the run (the
capture's exit codes and the shot count); §13 items 1 and 5 discharged or their
residue filed with an owning phase; `STAGED_PLAN_wallet_policy_composer.md` §S4
STATUS; `design/journeys/README.md` row + `build_pdf_composer.py` (the PDF the
staged plan promises, regenerable by its own README); `design/FOLLOWUPS.md`:
F-460 checked against the code (the Multisig Build comment is present or the
item is closed), F-461's "revisit at S4's journey run" answered (the use-site
arm stays unreachable: say so); the walk record and the continuity file.

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
  assertions -- a comparison that cannot fail is Critical --, what the diff
  made false elsewhere, CI gates as CI runs them); fold; sonnet verification;
  fork merge `--no-ff` + push + watch `test.yml`; engrave push via
  `scripts/push-via-staging.sh` with master FROZEN.
- **R0 for THIS plan**: ONE journey lens (opus; brief
  `design/agent-briefs/composer-S4-plan-R0-journey-brief.md`; report
  `design/agent-reports/composer-S4-plan-R0-r0-journey.md`): walk the two
  itineraries against the SHIPPED fork main `60bee002` (the Go harness or the
  emulator), pin every ? cell, and at each step ask what else the operator
  might do. A clean round closes the lens; the implementer is dispatched
  immediately after the fold.

**Coverage line for the reviewer.** Machine-verified already: every §2 value
(`md`, `me`, `ms` runs on 2026-09-03), Task 0's run, the emulator JS surface
(`shTap`, `shPress`, `shRelease`, `shSysw`, `shScreen`, `shNFC`, `shToolpath`,
`shPace` -- `cmd/emu/*_js.go`), the payload-load sequence (`walk_s4_gate.js`).
NOT verified, and the lens's first job: the path line's wording after a lock
and a hash edit (the Time lock and Hash lock steps), the seed row's label in the seating pick list
(the Seat keys pick list), whether form A of a seed-seated policy asks Full/Watch-only
(the policy-itself form), the census wording, the policy stub's relation to the policy id
(the keyed Template screen), and whether the door's Lead is exactly `Keys loaded: 2, plus 1 seed.`
with this payload.

## 5. Order and ownership

Tasks 1 → 2 → 3 are ONE implementer (opus, UC off) in two worktrees
(`/scratch/code/shibboleth/wt-composer-s4-emu` off fork main, branch
`composer-s4-emu`; `/scratch/code/shibboleth/wt-engrave-s4-emu` off master,
branch `composer-s4-emu`), reporting to
`design/agent-reports/composer-S4-implementation-report.md`. Task 4 proceeds
live with the operator meanwhile (different artifacts, no dependency). Task 5
at the operator's word. Task 6 last, by the controller.
