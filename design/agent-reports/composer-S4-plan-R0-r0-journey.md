# composer S4 plan — R0 round 0, the JOURNEY lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md` at mnemonic-engrave `d640875`
(unchanged at `1e57f19`, verified `git diff --stat d640875 HEAD -- <plan>` empty).
**Against:** seedhammer fork `main` `60bee002f24dfb0092a9767b9f20d0b4c5cdf619`, read-only, walked in a
`cp -r` copy at `/scratch/code/shibboleth/.s4-lens/seedhammer` (deleted at the end; `git status
--porcelain` empty in both repos).
**Method:** (a) throwaway Go harness tests in the copy — `gui/zz_s4lens_walk_test.go`,
`gui/zz_s4lens_reencode_test.go` — driving `walletPolicyFlow` with the plan §2 payload through
`synctest`/`runUI`/`pumpUntil`/`click`, dumping every frame; plus direct calls to
`composerArtifactsFor`, `composerStubLines`, `composerConsentLinesFor`, `composerMappingLines`,
`composerCensusLines`, `bundlePlatePlan`. (b) `md` / `me` by path for the host oracle.
The emulator arm was NOT run: `cmd/emu` has no composer payload blob (plan Task 1), so the keyed
itinerary is unwalkable there today, exactly as the brief says.
**Environment:** Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go/bin`,
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local -mod=readonly TMPDIR=/scratch/code/shibboleth/.tmp`.

Frames below are quoted as `ExtractText` returns them: **spaces are stripped**, which is also what
`uiContains` compares against. Where a string is quoted with spaces it comes from a direct call to
the copy-writing function, not from a frame.

---

## 0. THE ORACLE — the brief's single highest-value check

**The keyed arm's Policy-ID and addresses are CORRECT, and the plan's reading of §4f is right.**

Driving the keyed arm to the consent (throwaway `TestS4LensKeyedArm`) and independently
recomputing it with no UI (`TestS4LensOracle`):

```
@2 account=0 origin=m/48'/0'/0'/2' fp=b8688df1
   xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX
SELF-CHECK PASSES
STUB LINE: Template-ID: 531ab9e1777f018ae53694387dd0d128
STUB LINE: mk1 stub (template): 531ab9e1
STUB LINE: Policy-ID: 4dd749a8372af515a61d7104faf944ef
STUB LINE: mk1 stub (policy): 4dd749a8
CONSENT LINE: Receive 0:  bc1q8cf5g5fxfld9t22xguk7e0mg9mkjl2ujcxuux9napkw8cy89n3mqk0tp4l
CONSENT LINE: Receive 1:  bc1qkd729k2r3kvrewzgdtpj0quhrrv9u4jgndt2zsmy6ypnr7rslzwsfhmu9a
CONSENT LINE: Change 0:   bc1q9ms8tdk54dzaelef0rrg82fpm3s9nfgyr30aed96rnyuj02hhgrqy3dyru
CONSENT LINE: Change 1:   bc1q3cs923r9rdcv5s8zmwkd5strrh7svzzpg2yrl4hcue3f3fv4lyfsdp3tz9
KEYED CHUNKS (7): first = md1flv5xrq9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2at4gj8fq03ncnsuxv
```

Every one of those equals plan §2 byte for byte: Policy-ID `4dd749a8372af515a61d7104faf944ef`,
Template-ID `531ab9e1777f018ae53694387dd0d128`, 7 chunks, chunk 1 identical, all four addresses.
`composerSeedAccountFor` counts the earlier slots THIS MASTER fills (0 for @2, since @0/@1 are
master A's key records), so the device seats @2 at `m/48'/0'/0'/2'` — the plan's origin, not md's
lowest-free `48'/0'/2'/2'`. **No oracle mismatch on the keyed arm.**

**The keyless arm's oracle IS wrong — see C-1.** Same policy, different string.

---

## 1. Row fidelity — KEYED arm

| # | verdict | evidence |
| --- | --- | --- |
| 1 | not walkable here (emulator only) | `walk_s4_gate.js` / `walk_build_policy.js:357-381` carry exactly this leg; needles unchanged |
| 2 | VERIFIED (by inheritance) | `walk_s3_nested.js:303-311`: `goTo("Load Payload")` → CONFIRM → `Payload Digest` |
| 3 | VERIFIED (by inheritance) | ibid., `Payload Warnings` follows the digest; the blob carries a ClassMnemonic so F1 fires |
| 4 | VERIFIED (by inheritance) | ibid., `Keep this payload loaded?` → CONFIRM (KEEP) → `Load Payload` |
| 5 | **VERIFIED** | `Keysloaded:2,plus1seed.ScancardsBuildanewpolicyWalletPolicy` — the lead is exactly `Keys loaded: 2, plus 1 seed.`, two rows, no `From payload` |
| 6 | **VERIFIED** | `Whichscript?Taproot(tr)Segwit(wsh)Nested(sh-wsh)Legacy(sh)Newpolicy` |
| 7 | **VERIFIED** | `Startfrom?Buildmyownpathsplain-multisigsimple-timelocked-inheritancekofn-recoverytiered-recoveryhashlock-gateddecaying-multisigNewpolicy` — row 0 is `Build my own paths` (W-1 shipped) |
| 8 | **CORRECTED** (I-5) | `Spendpathsslots:0/keysavailable:2AddaspendpathChangethescriptDone`. `keys available: 2` holds; there is **no** `+ seed` and no "any slots" line, and `slots:` reads **0** on an empty list |
| 9 | **VERIFIED** | `Whatcanspendonthispath?KeysAhash,nokeysPath1` → `KeysPath1:howmanykeys?12345` → `ThresholdPath1:howmanymustsign?12` → `Path1:2-of-2` |
| 10 | **CORRECTED** (I-6) | row reads `Path2:1key`, not `Path 2: 1-of-1` (`composerPathLine`: `N==1` → `"1 key"`) |
| 11 | **CORRECTED** (I-7) | title is `Path 2 lock`; `Whatkindoftimelock?NoneAfterawaitAfteradateorheight` → `Measuredhow?BlocksDays` → `1234567890Howmanyblocks?1to65535blocks` → after 12960: `12960blocks(about90.0days)`. The echo screen is `Path2lock12960blocks(about90.0days)` — **ONE line, no `now:` bound line** (`composerLockBoundLine` returns `""` for `LockOlderBlocks`/`LockOlderUnits`). Path row becomes `Path2:1key+12960blocks` |
| 12 | **CORRECTED** (I-8) | title `Path 2 hash`; `Path2hashWhichhash?hash1abababab..ababababType64hexNohashlock` — the row is `hash 1  abababab..abababab`. Taking it draws an **unnamed modal first**: `ThehashmustbeSHA-256ofa32-bytevalue....` (§8i). Path row becomes `Path2:1key+hash+12960blocks` |
| 13 | **CORRECTED** (I-3, M-1) | `Done` goes straight to the stub screen: `Sorted keys, or your order?` is **never asked** (`composerSortedIsLegal` requires `len(Paths)==1`; measured `sorted legal for path 0: false`). Stub page 0: `TemplateTemplate-ID:531ab9e1777f018ae53694387dd0d128mk1stub(template):531ab9e1mkencode--xpub<xpub>...`; page 1: `TemplateSlot@0expectsakeyatm/48h/0h/0h/2hSlot@1expectsakeyatm/48h/0h/1h/2hSlot@2expectsakeyatm/48h/0h/2h/2h`. Notation is `h`, not `'`. Template-ID and the `531ab9e1` stub are correct |
| 14 | **CORRECTED** (I-4, I-5, M-1) | `Seat keys into this template?` **never draws** (it is `composerSeatingStep`'s `len(st.sources)==0` branch). The first frame is the pick list: `SeatkeysSlot@0,Path1key1of2:chooseakey73c5da0am/48h/0h/0h/2h73c5da0am/48h/0h/1h/2hTypeaseedLeaveunseated` — four rows, `h` notation, **no seed row** |
| 15 | **CORRECTED** (I-5, I-11) | @2's list is `SeatkeysSlot@2,Path2key1of1:chooseakeyTypeaseedLeaveunseated`. `Type a seed` opens three screens the row does not name: `Wherefrom?FROMPAYLOADTYPEITSeedforthepolicy` (plus `SCAN` on the emulator, `Features()==FeatureNFC`), then `SeedforthepolicySource:thesystemwidepayload`, then `AddaBIP-39passphrase?SkipAddpassphrasePassphraseseed1`. Only then does the row appear: `seed1(anyslots)`. Mapping review page 0 is as the plan says — `Keymapping@0:73c5da0am/48'/0'/0'/2'@1:73c5da0am/48'/0'/1'/2'@2:b8688df1m/48'/0'/0'/2'Thisdevicecannotconfirmakeywasderivedattheoriginitdeclares.` — but **page 1 carries a warning the row omits**: `KeymappingSAMESEED,SAMEPATHSlots@0and@1arethesameseed.Thispath's2-of-2canbesatisfiedbyoneperson.Lianawillrefuseit.` |
| 16 | **VERIFIED** | `TemplateTemplate-ID:531ab9e1777f018ae53694387dd0d128mk1stub(template):531ab9e1Policy-ID:4dd749a8372af515a61d7104faf944efmk1stub(policy):4dd749a8StampBOTHstubsoneachkeycard:--policy-id-stub531ab9e1--policy-id-stub4dd749a8`. The `?` resolves: the policy stub **is** `4dd749a8`, the id's first four bytes. Slot lines on page 1-2 now read `Slot@0:73c5da0am/48h/0h/0h/2h` … `Slot@2:b8688df1m/48h/0h/0h/2h` |
| 17 | **CORRECTED** (I-6) | `ReviewPath1:2-of-2Path2:1key12960blocks(about90.0days)hashabababab..ababababPolicy-ID:4dd749a8372af515a61d7104faf944ef` — `Path 2: 1 key`, not `1-of-1`; the hash line is ONE line `hash abababab..abababab`. Both ids and all four addresses present; the addresses are on consent pages 2-3 |
| 18 | **VERIFIED** | `Nothingoutsidethisdevicehascheckedthispolicy....Holdbuttontoconfirm.Beforeyoufundit` then `Whichform?ThepolicyitselfTemplatepluskeycardsWhattoengrave` |
| 19a | **CORRECTED** (I-1) | The `?` resolves YES: form A with a seed-seated slot **does** ask — `Whattoengrave?Full(seed+keys)Watch-only(keys)EngraveMode`. But the census reads **`Thisengraves2plates.md1policy:2plates(thewalletpolicy,withitskeys)`**, not 7 plates |
| 20a | **CORRECTED** (I-2) | `shToolpath.strings()` is one entry **per PLATE**, newline-joined (`notifyPlateText` → `strings.Join(strs, "\n")`, `gui/engraved_hook.go:124`). Measured plan: plate 1 = chunks 1-5, plate 2 = chunks 6-7. So 2 entries, not 7 strings |
| 19b | **VERIFIED** | `This engraves 4 plates. / md1 template: 1 plate (key-less wallet policy) / mk1 key @0: 1 plate (m/48'/0'/0'/2') / mk1 key @1: 1 plate (m/48'/0'/1'/2') / mk1 key @2: 1 plate (m/48'/0'/0'/2')` |
| 20b | **CORRECTED** (I-2, I-12) | 4 entries, each newline-joined: template 2 chunks, @0 2 chunks, @1 **3** chunks, @2 2 chunks. And the file it is compared against does not exist in Task 2 |

## 1b. Row fidelity — KEYLESS arm

| # | verdict | evidence |
| --- | --- | --- |
| 1 | VERIFIED (by inheritance) | as the keyed arm's row 1 |
| 2 | **CORRECTED** (I-10) | In the Go harness (no reader) the lead is exactly `Nokeysloaded.Thisbuildsakey-lesstemplate.ScancardsBuildanewpolicyWalletPolicy`. **On the emulator it will not be**: `platform.SyswReader()` returns the embedded records blob by default and `Probe()` is true, so `composerDoorLines(nil, true)` gives `A payload is in flash but not loaded. Load it from the carousel first.` |
| 3 | **VERIFIED** | `Startfrom?` → row 0 → `Spendpathsslots:0Adda…` → 3 → 2 → `Spendpathsslots:3Path1:2-of-3…` → Done → **`Sortedkeys,oryourorder?Sorted(usual)KeepmyorderKeyorder`** (asked here, and only here) → `TemplateTemplate-ID:e0863d3ccac31a64d3b5e14b85ccd6c0mk1stub(template):e0863d3c…` / `TemplateSlot@0expectsakeyatm/48h/0h/0h/3hSlot@1expectsakeyatm/48h/0h/1h/3hSlot@2expectsakeyatm/48h/0h/2h/3h` (again `h`, not `'`) |
| 4 | **CORRECTED** (I-4) | Not "no seating": `Seatkeysintothistemplate?Engraveakey-lesstemplateTypeaseedKeys` draws first. Then the consent is right: `ReviewPath1:2-of-3KEYPATH:NONE(NUMS)Spendsusethescriptpathsonly.BitcoinCoreandNunchukimportthisform.LianaandBIP-388signersneedanunspendablexpubinstead(seeF-449).Template-ID:e0863d3ccac31a64d3b5e14b85ccd6c0mk1stub(template):e0863d3c` and page 1 `…Keylesstemplate-noaddresses.Verifyoff-device.` |
| 5 | **VERIFIED** | `Nothingoutsidethisdevice…` (HOLD) → `Noslotisseated,sothereisatemplateandnothingelse.Whattoengrave` (a modal, not a picker) → `PlatesToCutThisengraves1plate.md1template:1plate(key-lesswalletpolicy)…` |
| 6 | **CORRECTED — CRITICAL (C-1)** | the device emits `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3`, not the plan's `md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc`. One chunk, one plate, so `strings()` is a one-element array — but of the OTHER string |

---

## 2. The `?` cells, pinned

| plan cell | pinned value |
| --- | --- |
| keyed 8 — "the seed's `any slots` line" | **absent.** Line is `slots: 0 / keys available: 2`; `composerSlotsKeysLine` appends `+ seed` only for a `composerSourceSeed` in `st.sources`, and `composerFlow` loads only `composerKeySources` + `composerCardSources` |
| keyed 11 — "the path line carries the wait" | `Path 2: 1 key + 12960 blocks`; echo screen `Path 2 lock` / `12960 blocks (about 90.0 days)`, **no bound line** |
| keyed 12 — "the path line carries the hash mark" | `Path 2: 1 key + hash + 12960 blocks` |
| keyed 13 — `mk1 stub (template): <8 hex>` | `531ab9e1` |
| keyed 14 — "the seed row" | **not offered on the first pass.** After `Type a seed` it is `seed 1  (any slots)` (`composerSourceRow`) |
| keyed 16 — `mk1 stub (policy): <8 hex>` | `4dd749a8` — the plan's guess is right; both stubs are the first 4 bytes of their ids |
| keyed 19a — "does form A ask Full/Watch-only" | **yes**, whenever `st.reg.count() > 0`. Rows: `Full (seed + keys)` / `Watch-only (keys)` |
| coverage line — "is the door's Lead exactly `Keys loaded: 2, plus 1 seed.`" | **yes**, measured `composerDoorLines(...) = ["Keys loaded: 2, plus 1 seed."]`; `hash:`/`now:` are neither Key nor Mnemonic nor Unknown, so `inert == 0` and no second line |
| coverage line — "the census wording" | form A `This engraves 2 plates.` + `md1 policy: 2 plates (the wallet policy, with its keys)`; form B `This engraves 4 plates.` + four rows; keyless `This engraves 1 plate.`; Full-mode form A `This engraves 3 plates.` with `ms1 secret share 1: 1 plate (secret seed backup)` FIRST |

---

## 3. What else might the operator do

| step | the other action | what the device does | class | earns a change? |
| --- | --- | --- | --- | --- |
| door | `Scan cards` | `composerRouteScan` → the NFC gather; on the emulator that is a wait with no tag | not-our-concern | no |
| door | Back | leaves Wallet Policy (the door is a loop; `walletPolicyFlow` returns) | default | no |
| `Start from?` | pick a preset (e.g. `hashlock-gated`) | seeds a 2-path shape with the pinned `a8..a8` digest — a different wallet, silently | default | no (row 0 is the default and the plan taps it) |
| `Start from?` | Back | returns to `Which script?` — W-1's fix, shipped | default | no |
| path list | Back | `composerShapeFlow` returns false → `composerWrapperPick` again, list intact | default | no |
| `How many blocks?` | type `0` or `65536` | echo becomes `Relative locks reach at most 455 days in blocks or 388 days in time. Use an absolute date.` and the checkmark is withheld; empty field says `1 to 65535 blocks` | refusal | no |
| lock kind | `Days` instead of `Blocks` | `LockOlderUnits`, echo `90 days = 15188 units of 512 s (90.0 days)` — a **different Policy-ID** | not-our-concern | no (the driver must pick Blocks; already does) |
| `Which hash?` | `No hash lock` | clears the digest and skips the §8i modal — a different Policy-ID | not-our-concern | no |
| seat @2 | `Leave unseated` | §8p: `3 slots, 2 keys available.` / `Unfilled: slot @2.` then `Seat keys` / `What now?` / `Back to the paths` \| `Engrave a key-less template`; forms collapse to `Template plus key cards` alone; slot @2 falls back to `m/48h/0h/2h/2h` | refusal | no |
| seat | seed at two slots of one path | §4f gives accounts 0 and 1 (distinct origins, no §8v), and the mapping review adds §8g's SAME SEED body | warning | no |
| mapping review | Back | releases the last seat and lands on `Slot @2` with its source offered again (shipped `TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys`) | default | no |
| consent | Button3 before the last page | withheld — `composerReadScreen` arms the checkmark only after the last page has been laid out once | refusal | no |
| consent / §8l | Back / decline the hold | `composerConsentFlow` returns false → `composerFlow`'s `continue` → **the path list**, seats intact but the whole tail re-walked | default | no (recoverable, and the seats survive) |
| `Choose engraving` | `QR ONLY` on the keyless plate | a plate the SH2 cannot read back (no camera) — Task 4 step 4's read-aloud becomes impossible | default | no — `TEXT + QR` is row 0. Note: on every **packed** plate (both form-A plates, all four form-B plates) only `TEXT ONLY` is offered, so the picker is a single row there |
| `Engrave Mode` | `Full (seed + keys)` | prepends `ms1 secret share 1` — a **bearer** plate of master B — and the census becomes 3 plates | warning | no (the plan takes Watch-only) |
| after the last plate | — | `bundleShowMs1Reminder` is true for every composer form, so a modal titled `Wallet Policy` reads `Bundle engraved. Also hand-engrave your ms1 share(s) - they are never sent over NFC.`, then the flow returns to the **door** | — | **yes — I-9** |
| keyless door, on the emulator | do nothing | the flash region probes true, so the lead is the payload-not-loaded line, not the key-less line | — | **yes — I-10** |

---

## 4. Findings

### C-1 (Critical) — the keyless arm's md1 oracle is the UNCHUNKED encoding; the device engraves the CHUNKED one

Plan §2 and keyless rows 3/6, Task 2's `keyless-tr.md1.txt`/`keyless-tr.id.txt`, and Task 4 steps 1
and 4 all pin

```
md15zfdsssj6tvyywtfdssj5hqqxqujzyxaduyd9dp5v3xc          (47 chars)
```

The device emits

```
md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3  (56 chars)
```

**Reproduction.** In the copy, `TestS4LensKeylessChunksAndShortfall`:

```go
st.list = md.PathList{Wrapper: md.ComposeTr, Paths: []md.SpendPath{
    {Keys: &md.KeySet{K: 2, N: 3, Sorted: true}}}}
composerSizeAssignments(st)
chunks, _ := composerTemplateChunksFor(st)
// KEYLESS TR CHUNKS (1): ["md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3"]
```

and the full keyless walk reaches `This engraves 1 plate.` over exactly that string.

**Cause, measured — this is NOT a host/device codec divergence.** `md.Composed.Chunks()` is
`split(c.d)` and its own doc says "always chunk form, as the primary's force_chunked vectors are"
(`md/compose.go:236-238`). The plan ran `md encode` without `--force-chunked`, and a template this
short encodes unchunked. On the host:

```
$ md encode "tr(50929b74…,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*))" --force-chunked --group-size 0
chunk-set-id: 0xb0884
md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3
```

— byte-identical to the device's. Both strings `md inspect` to the same template, the same
`wallet-descriptor-template-id e0863d3ccac31a64d3b5e14b85ccd6c0`, the same origins and the same
`md1_encoding_id b0884601fa89b3d294c599d8a6bb1602`, and `md verify --template <T>` exits 0 for
**both** — so no gate the plan names would catch the substitution.

**Why Critical.** It is the string on the ONE real plate Task 4 cuts and reads back, and the byte
comparison in keyless row 6 would fail against it. The failure is also seductive in the wrong
direction: `md verify` passing on both invites "the driver is wrong, relax the comparison", which
would remove the only check that the device and the host agree.

**Hypothesis.** In §2, keyless rows 3/6, Task 2 and Task 4, replace the string with
`md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3`, and mint it in
`transcript_composer.sh` with `md encode … --force-chunked --group-size 0`. Add a one-line note that
the device is chunk-form-always, so every short-policy oracle in this and later plans must pass
`--force-chunked`. (Also fix §2's "one 46-character string": it is 56.)

### I-1 (Important) — form A's census is 2 plates, not 7

Keyed row 19a. Measured frame:
`PlatesToCutThisengraves2plates.md1policy:2plates(thewalletpolicy,withitskeys)…`. F-423 packs a
card's chunks onto as many plates as fit (`bundleCardPlates`), so 7 chunks become 5 + 2. Layout
depends only on `engrave.Params.StrokeWidth` and `Millimeter`, which are identical in the test
platform (`gui/gui_test.go:376-377`: `mm = 6400`, `strokeWidth = 0.3*mm`) and `internal/sh2.Params()`
— so this is the emulator's and the device's number too.
**Hypothesis.** Row 19a expects `This engraves 2 plates.` and `md1 policy: 2 plates`.

### I-2 (Important) — `shToolpath.strings()` is one entry per PLATE, newline-joined

Keyed rows 20a/20b. `notifyPlateText` does `ea.PlateText(ids, strings.Join(strs, "\n"))`
(`gui/engraved_hook.go:124`), and `engravedRecorder` records one string per finished plate. So form A
yields **2** entries (chunks 1-5 joined, chunks 6-7 joined) and form B yields **4** (template 2
chunks; `mk1 key @0` 2; `mk1 key @1` **3**; `mk1 key @2` 2). "7 strings, byte for byte" cannot hold.
**Hypothesis.** State the comparison as: split each census entry on `"\n"`, concatenate in order,
and compare that flat list against `keyed.md1.txt`. Assert the entry COUNT equals the census's plate
count, which is the half that catches a packing change.

### I-3 (Important) — `Sorted keys, or your order?` is never asked on the keyed arm

Keyed row 13's "do" cell. `composerSortedIsLegal` returns false unless `len(list.Paths) == 1`
(`gui/composer_state.go:226-235`); the keyed arm has two. Measured `sorted legal for path 0: false`,
and the walk goes from `Done` straight to the stub screen. A driver that `waitFor`s it hangs.
The ★ on that cell is honest about its provenance and wrong about its scope: the join test's shape is
a SINGLE 2-of-2 path.
**Hypothesis.** Delete the key-order step from keyed row 13. It stays correct on the keyless arm,
where it is measured (`Sortedkeys,oryourorder?Sorted(usual)KeepmyorderKeyorder`).

### I-4 (Important) — `Seat keys into this template?` is in the wrong arm

Keyed row 14 names it; it never draws there. Keyless row 4 says "no seating: no sources"; it draws
there. `composerSeatingStep` shows that ChoiceScreen only when `len(st.sources) == 0`
(`gui/composer_flow.go:219-238`). Measured, keyless:
`Seatkeysintothistemplate?Engraveakey-lesstemplateTypeaseedKeys`.
**Hypothesis.** Move the row. Keyed 14 goes straight from the stub screen's CONFIRM to
`Slot @0, Path 1 key 1 of 2: choose a key`; keyless 4 gains a step selecting
`Engrave a key-less template` (row 0) before the consent.

### I-5 (Important) — the payload's seed is not a composer source

`composerFlow` loads `composerKeySources(ctx) + composerCardSources(ctx)` and nothing else
(`gui/composer_flow.go:41`); no site reads `sysw.ClassMnemonic` into `st.sources`. So:

* row 8's `+ seed`/"any slots" line does not exist;
* row 14's seed row is not in the first pick list;
* reaching the seed costs three screens the itinerary does not name —
  `Where from?` (`FROM PAYLOAD` / `TYPE IT` / `SCAN` on the emulator, `FeatureNFC` is reported at
  `cmd/emu/platform.go:343`), the acceptance screen `Seed for the policy` / `Source: the systemwide
  payload`, and `Add a BIP-39 passphrase?` / `Skip` \| `Add passphrase`.

`take(ClassMnemonic)` does not consume, so a Back and a retry still offers the payload.
**Hypothesis.** Split row 14 into 14 (the pick list, four rows, no seed) and 14a (`Type a seed` →
those three screens → the pick list re-drawn with `seed 1  (any slots)`), and drop `+ seed` from
row 8.

### I-6 (Important) — a 1-key path renders `1 key`, not `1-of-1`

Keyed rows 10 and 17. `composerPathLine` (`gui/composer_state.go:264-268`) and
`composerBranchLines` (`gui/composer_consent.go:79-82`) both special-case `N == 1`. Measured:
`Path2:1key+hash+12960blocks` on the list and `Path2:1key` on the consent.
**Hypothesis.** Both cells read `Path 2: 1 key`.

### I-7 (Important) — a relative lock's echo carries no `now:` bound line

Keyed row 11's "wait for" cell asks for "the §8c relative-blocks echo with the `now:` bound line".
`composerLockBoundLine` returns `""` for `LockOlderBlocks` and `LockOlderUnits`
(`gui/composer_lock.go:108-113`) — deliberately: nothing about the present bounds a relative lock.
Measured echo screen, whole frame: `Path2lock12960blocks(about90.0days)`.
**Hypothesis.** Row 11's expected frame is that one line. If the plan wants the bound line
photographed, it needs an ABSOLUTE lock, which changes the Policy-ID and the whole §2 oracle — so
the cheap fix is to delete the clause.

### I-8 (Important) — the §8i hash-rule modal is an unnamed screen in row 12

Selecting a payload digest draws `composerCopyHashRule()` as a `showError` before returning
(`gui/composer_hash.go:496-502`). Measured frame:
`ThehashmustbeSHA-256ofa32-bytevalue.Apassphrasemustbehashedto32bytesfirst,thenhashedagain.Ahashofthepassphraseitselfcanneverbespent.Path2hash`.
A driver that taps once lands its next tap on the path menu.
**Hypothesis.** Row 12 gains the modal as an explicit step (and it is worth a shot — it is a §8 body
the capture would otherwise never record).

### I-9 (Important) — the composer's engrave loop has no verify offer, and ends on a modal the handler list does not know

Rows 18/20 say "the engrave loop is `walk_s3_nested.js`'s". That loop terminates on
`VERIFY_OFFER = "Verify the engraved plates?"` (`cmd/emu/walk_s3_nested.js:94, 231`). The composer
never draws it: `composerEngraveStep` ends at `bundleEngrave(...) == bundleEngraveDone`
(`gui/composer_flow.go:324`) and `composerFlow` returns into `walletPolicyFlow`'s door loop
(`gui/wallet_policy.go:46-52`). The only three flows that offer verify are `gui/singlesig.go:284`,
`gui/multisig.go:341` and `gui/multisig_build.go:554`.

Worse, `bundleEngrave` ends by showing `showError(ctx, th, "Wallet Policy", bundleMs1ReminderText())`
whenever no `cardMS1` is in the set — true for form A watch-only, form B and the keyless arm alike:
`"Bundle engraved. Also hand-engrave your ms1 share(s) - they are never sent over NFC."`. That screen
matches none of `ENGRAVE_HANDLERS`, so the loop's "an unrecognised screen STOPS the walk" arm fires
and the capture ends `{act: "STALLED"}` with `reachedVerifyOffer: false` — after every plate was
correctly cut.

**Hypothesis.** `shots_composer.js` keeps the three handlers verbatim, adds a fourth
(`match: "Bundleengraved"`, `act: "confirm"`), and terminates on the DOOR
(`"Build a new policy"`) rather than on a verify offer. The census is still read from
`shToolpath.strings()` after the loop.

### I-10 (Important) — the keyless door's lead on the emulator is the payload-in-flash line

Keyless row 2 expects `No keys loaded. This builds a key-less template.`. `composerDoorFlow` calls
`ctx.Platform.SyswReader().Probe()`, and the emulator's default reader is the embedded records blob
whose `Probe()` is `magic == sysw.MAGIC` → true (`cmd/emu/platform.go:313-327`,
`cmd/emu/sysw_test_payload.go:83`). With nothing loaded, `composerDoorLines(nil, true)` returns
`"A payload is in flash but not loaded.\nLoad it from the carousel first."`
(`gui/composer_door.go:59-65`). Only the harness (no reader) gives the plan's line.
**Hypothesis.** The keyless arm calls `window.shSysw("none")` before the walk — the same thing
`shots_operator.js:188` and `shots_pathological.js:203` already do — and row 1's boot-offer step
disappears with it. If the plan prefers to keep the flash region present, row 2's expected lead
becomes the payload-not-loaded line. Same question on the DEVICE at Task 4: the plan asserts the
machine "holds no payload today", which is worth confirming at the door before step 1.

### I-11 (Important) — row 15 omits a warning the plan's own fixture fires

The mapping review's page 2 reads
`KeymappingSAMESEED,SAMEPATHSlots@0and@1arethesameseed.Thispath's2-of-2canbesatisfiedbyoneperson.Lianawillrefuseit.`
(§8g's first body, via `composerSharedSeedInPath`). It fires because §2's two `key:` records are two
ACCOUNTS OF ONE MASTER (`73c5da0a`) seated in one 2-of-2 path — the plan says so itself at row 15
("one master, two accounts: C5") without noticing that the same fact makes the path
one-person-satisfiable.
**Hypothesis.** Row 15 gains the §8g body as an expected line and a shot. See also M-4 on whether
this is the fixture §12 item 2 should photograph.

### I-12 (Important) — row 20b compares against a file Task 2 does not produce

Row 20b: "`shToolpath.strings()` == the keyless template chunk(s) + three mk1 strings". Form B's
first card is the wsh template **with fingerprints** (`composerArtifactsFor` composes it from
`composerDeclaredOrigins`, which is why the second stub screen prints `Slot @0: 73c5da0a …` rather
than "expects a key at"). Task 2's file list has `keyless-tr.template`/`keyless-tr.md1.txt` — the
taproot ARM's artifact — and nothing for this one. The phrase "the keyless template" names both.
For the record, the device emits two chunks:
`md1fxnz3qs9qjtvyyy5jmpprjjtvyyy4qqxpxqcyy2v6tjv6a4w46h2at4w46h2at4w46h2shlte30qvuhvrq` and
`md1fxnz3qsw46h2at4w46h2at4w46h2at4w46h2msqqqv4qp0npeutks2tnchdq4ts6yd7yq5swf47peq533w`.
**Hypothesis.** Task 2 mints `keyed-template.md1.txt` (the wsh template WITH fingerprints, i.e.
`md encode` of the origin+fingerprint form, `--force-chunked` not needed at this size but harmless),
and row 20b names that file. Note the first stub screen (row 13) shows a DIFFERENT chunk set — the
same shape with no seating, @2 at `48'/0'/2'/2'`, no fingerprints — which shares the Template-ID and
is not what form B cuts.

### M-1 (Minor) — two origin notations on two screens

The stub screen prints `m/48h/0h/0h/2h` (it reads `ExpandedKey.OriginPath`, a `bip32.Path`); the
mapping review and the card summaries print `m/48'/0'/0'/2'` (`composerOriginText`). Rows 13, 14 and
the keyless row 3 all quote the `'` form. The stub screen is the one whose job is to be copied onto
an `mk encode --origin-path` invocation, so the difference is on the wrong screen — but
`key_card_seating` compares structurally, so nothing mis-seats.
**Hypothesis.** Correct the plan's cells to `h`; file the notation split as a follow-up rather than
changing shipped copy in S4.

### M-2 (Minor) — Task 1's pack command does not run

```
$ printf 'hash:…\n' | me sysw pack --no-passphrase --in - --out /dev/null
me: -: No such file or directory (os error 2)   exit 2
```

`--in` takes a FILE; the help says "With neither this nor argv records, the same newline-separated
form is read from STDIN".
**Hypothesis.** Drop `--in -`: `go run ./cmd/buildpayloadcomposer | target/debug/me sysw pack
--no-passphrase --out cmd/emu/sysw_composer_payload.bin`. (§2's own `--in <records>` form is fine.)

### M-3 (Minor) — the paged screens WRAP, and "shots per page" has no stop condition

`composerReadScreen` and `composerPickScreen` page forward and wrap to `start = 0`
(`gui/composer_paged.go:135-141`). Measured: the consent cycles 0,1,2,3,0,1,2,3…; the stub screen
cycles 0,1,0,1…. A driver that pages a fixed number of times re-screenshots page 1 and reports more
pages than exist.
**Hypothesis.** The driver records the first page's text and stops paging when it recurs; the shot
count per screen becomes an assertion rather than a loop bound.

### M-4 (Minor) — the acceptance fixture is a wallet the device warns about

Because @0 and @1 are one master, §12 item 2's photographed record will carry "This path's 2-of-2
can be satisfied by one person. Liana will refuse it." A second master for @1 would make the
exemplar a plain 2-of-2 and cost only a second `ms derive`.
**Hypothesis.** Either swap `key:` record 2 to a second master (and re-mint every §2 value), or add
one sentence to §2 saying the shared master is deliberate, so nobody reads the warning as a defect
years later. The C5 label ("one master, two accounts") is exactly the case §8g exists to flag when
both land in ONE path.

### M-5 (Minor) — two census rows read identically

Form B's census: `mk1 key @0: 1 plate (m/48'/0'/0'/2')` and `mk1 key @2: 1 plate (m/48'/0'/0'/2')`.
The summary is the origin, and master B's seed lands at ITS OWN account 0', so two rows differ only
by the `@i` in the label. Correct, and easy to mis-read while counting blanks.
**Hypothesis.** Note it in the plan's expected census so the reviewer of the capture is not surprised.

### N-1 (Nit) — §2's character count

"Host, emulator and device meet on this one 46-character string": the host string is 47 characters
and the right one is 56.

### N-2 (Nit) — the end-of-engrave modal on a watch-only run

`Bundle engraved. Also hand-engrave your ms1 share(s) - they are never sent over NFC.` is shown after
a watch-only composer engrave that produced no ms1 share. Shipped copy
(`bundleShowMs1Reminder`/`bundleMs1ReminderText`), out of S4's scope, but the capture will record it
and someone will ask.

---

## 5. What each lens found

* **Lens 1 (row fidelity)** — 8 rows VERIFIED, 12 CORRECTED across both arms.
* **Lens 2 (the `?` cells)** — all 9 pinned; one (the policy stub) confirmed the plan's guess.
* **Lens 3 (what else)** — 15 divergences classified; **none earned a plan change on its own**. Every
  one is a refusal, a warning, a default, or not-our-concern whose wrong outcome is no worse than
  silence. Two divergences surfaced findings that belong to other lenses (I-9's terminal modal,
  I-10's door lead), and they are filed there rather than counted twice here.
* **Lens 4 (the oracle)** — the keyed arm's Policy-ID, Template-ID, both stubs, all four addresses
  and chunk 1 are CORRECT and the device's seed-account rule matches §4f as the plan reads it. The
  keyless arm's md1 is wrong (C-1). The engrave loop differs from `walk_s3_nested.js` (I-9).
* **Lens 5 (Tasks 4 and 5)** — no defect that is not already C-1 (the plate's own string) or I-10
  (the door lead on a machine whose flash region is not empty). Task 4's census check (1 plate),
  Template-ID check (`e0863d3c…`) and `md verify` step all hold as written — with the caveat that
  `md verify` exits 0 for BOTH strings, so it cannot be the check that catches C-1.

## 6. Counts

**1 Critical / 12 Important / 5 Minor / 2 Nit.**

The Critical and every Important is a defect in the PLAN's expected values or steps, not in
`60bee002`: the shipped composer behaved correctly at every step of both itineraries, its self-check
passed on an honest build, and the keyed arm's host-versus-device oracle agrees exactly.
