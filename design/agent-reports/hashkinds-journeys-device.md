# Device operator journeys — all four hash kinds

**Verdict: the four kinds compose correctly on every route I walked, but three
screens are silent or misleading at the moment an operator needs them — the hex
pad truncates a re-entered draft without saying so, the whole phrase arm names
no kind between the kind screen and the confirm, and the 20-byte warning tells
the operator nothing has seen a preimage while the preimage sits two rows down
in the same payload.**

Repo: `/scratch/code/shibboleth/seedhammer` @ `af3b937` (branch `main`, tree
clean at start and at finish). Driven with a throwaway `gui/zz_journey_test.go`
(deleted; `git status` clean, `gofmt -l .` = the pristine five).

**How the screen text is quoted.** Frames captured through
`op.Drawer.ExtractText`, which **strips all whitespace** — so a frame reads
`Path1hashWhichhash?hash16015741af53..774b6515…`. Where a frame is quoted I
give the extracted form verbatim and, where it matters, the source string from
`gui/composer_copy.go` beside it. Both are reproduced from a run, not retyped
from the source.

Findings already filed (F-534..F-546) are not re-filed. Two of them
(F-534, F-535) are **confirmed with new device-side evidence** and said so
inline.

---

## A. Compose a hashlock wallet on the device, once per kind (typed digest)

Driven four times: `Path 1 hash` → `Type a digest` → §8i rule → kind screen →
pad → (§8 warning) → assigned. `composerHashEdit`, real touch harness,
`sh2DisplaySize`.

### A1 — `Which hash?`, empty payload

> `Path1hashNohashrecordinthepayload.Typeaphrasebelow,ormakeonewithmshashlockonthehost.TypeahashlockphraseTypeadigestNohashlock`

1. **In hand:** nothing but a digest on paper.
2. **Device:** three rows; the lead names the host tool.
3. **What else:** identical for all four kinds — correct, no kind exists yet.

### A2 — the §8i rule modal

> `Thepreimagemustbea32-bytevalue.Apassphrasemustbehashedto32bytesfirst,thenhashedagain.Ahashofthepassphraseitselfcanneverbespent.Path1hash`

Holds for all four kinds (every miniscript hash fragment takes `OP_SIZE <32>`).
No divergence.

### A3 — the kind screen, identical on all four runs

> `HashfunctionAllfourtakea32-bytepreimage.sha256istheusualchoice.sha25664hexhash25664hexripemd16040hexhash16040hex`

1. **In hand:** a digest and (they hope) the name of the function that made it.
2. **Device:** four rows, each `<token>  <n> hex`; opens on the caller's seed.
3. **What else:** a 40-hex digest whose kind the operator does **not** know is a
   50/50 guess between rows 3 and 4, and nothing on the device can break the
   tie. **Classification: documentation only** — the host's `ms hashlock` with
   no `--kind` lists all four, and the device cannot know better.

### A4/A5 — the pad, per kind (the kind reaches the title, the logic and the bound)

| kind | empty pad | full pad |
| --- | --- | --- |
| sha256 | `0123456789ABCDEF0of64hexsha256hash` | `…ABABAB…64of64hexsha256hash` |
| hash256 | `0123456789ABCDEF0of64hexhash256hash` | `…CDCDCD…64of64hexhash256hash` |
| ripemd160 | `0123456789ABCDEF0of40hexripemd160hash` | `…EFEFEF…40of40hexripemd160hash` |
| hash160 | `0123456789ABCDEF0of40hexhash160hash` | `…121212…40of40hexhash160hash` |

The title (`ripemd160 hash`) and the bound (`of 40 hex`) both follow the kind.
This is the one screen in the flow that handles the four kinds best, and it is
where the earlier journey walk put its fixes.

**Observation, not a finding:** the pad readout extracts as **uppercase**
(`B867DB87…`) while the confirm, reconcile, census and plate locator all draw
**lowercase** (`b867db87..edbc96cb`). That is `ctx.Styles.word`
(`comfortaa.Bold17`, `gui/theme.go:104`), shared with BIP-39 word entry
(`gui/gui.go:1313`) — device-wide and older than this cycle. **Not our concern.**

### A6 — the §8 warning, ripemd160 and hash160 only

> `20-BYTEHASH,NOTDERIVEDHEREThisisaripemd160digestandnothingonthisdevicehasseenapreimageforit.Whoeversupplieditcanspendthispath.A20-bytehashalsoleavesfarlesscollisionmarginthansha256.Checkyouholdthepreimagebeforeyoufundthiswallet.Holdbuttontoconfirm.Hashlock`

Fires for both 20-byte kinds, names the kind, is a hold-to-confirm. Correct here
(see D for where the same copy is false).

**It carries no digest.** After the pad, the operator does not see the digest
they typed again until the Done consent screen. That is the reason F-570 below
matters as much as it does.

### A7 — the divergence that is not on any screen

`hashlockHeld` is empty after **all four** typed-digest runs (`A8 held material
entries=0`), so no preimage plate is offered at Done on this arm, for any kind.
Correct and by design.

---

## B. The typed-phrase arm, per kind

Driven four times: kind screen → phrase → method → derivation → confirm →
reconcile.

### B2/B3 — the phrase screen, **identical on all four runs**

> `qwertyuiopasdfghjklzxcvbnmABCspaceshowThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.0/100Hashlockphrase`

1. **In hand:** a phrase they are about to commit a path to.
2. **Device:** title `Hashlock phrase`, lead
   `"This screen does that hashing for you. Use a phrase you have never used anywhere else."`,
   an `n/100` counter, a masked readout.
3. **What else — and this is the finding:** the operator wants to check *which
   kind they tapped* (sha256 is the default and ripemd160/hash160 are adjacent
   rows). The screen does not say. The only way to look is **Back — which drops
   every character typed** (measured: 28 characters typed, Back, forward again,
   `0/100`). The pad got both remedies for exactly this (kind in the title,
   draft survives Back, F-541); the phrase arm got neither. → **F-571.**

### B4 — the method pick, identical on all four runs

> `HashlockmethodWhichmethod?hardened-slowKDF,about10ssha256-onehashofthephrase`

Names no kind either. Note the row token `sha256` here is the **method** axis
while a row reading `sha256` two screens earlier was the **kind** axis — that is
F-540, filed, left deliberately. Journey-wise the collision is worse than F-540
assumed, because with no kind named on this screen the operator has nothing to
disambiguate against: at hash160 they are looking at a screen offering `sha256`
and nothing anywhere saying `hash160`.

### B5 — the deriving screen

> `0%Deriving.Thistakesabout10seconds.Deriving`

Names no kind. Ten seconds with no way back except abandoning.

### B6 — the confirm modal, per kind (the first screen since the kind pick that names the kind)

| kind | modal head |
| --- | --- |
| sha256 | `hashsha2563cf5d421..b70a4c12method:hardenedchars:28` |
| hash256 | `hashhash25698a20fc2..641cd488method:hardenedchars:28` |
| ripemd160 | `hashripemd16009e7bb50..bcc2946bmethod:hardenedchars:28` |
| hash160 | `hashhash160b5b72c0e..214cd0bdmethod:hardenedchars:28` |

followed by
`Writedownthephrase,method,hashkindanddigestnow.Thiscompositionholdsthemuntilitends.Withoutthem,thispathcanneverbespent.Onephraseperpolicy.Neveruseitasapassphraseorpasswordanywhereelse.`

Good: the write-down instruction names all four things, and the kind rides with
the digest.

### B7 — the reconcile screen, per kind. **Copyable and correct on all four.**

> sha256: `Beforeyoucutplates,runmshashlock--kindsha256--methodhardenedwiththisphraseonthehostandcheckthedigestmatches.Iftheydiffer,donotfundthiswallet:builditagain.`
> hash256: `…runmshashlock--kindhash256--methodhardened…`
> ripemd160: `…runmshashlock--kindripemd160--methodhardened…`
> hash160: `…runmshashlock--kindhash160--methodhardened…`

Both flags present, both spelled as the host takes them, and the digest the
command reproduces is drawn on the same screen above the instruction. This is
the best screen in the cycle. **Nothing to file.**

The one thing it is not is *copyable* in the literal sense — there is no QR and
no way to get the string off the device; the operator retypes ~55 characters.
That is inherent to a four-button device with no camera. **Not our concern.**

---

## C. Back, at every screen

Measured, phrase arm, kind = hash160:

| Back pressed at | lands on | kept | dropped |
| --- | --- | --- | --- |
| kind screen | `Which hash?` | the path's existing hash | nothing |
| **phrase screen** | **the kind screen** (not `Which hash?`) | the chosen kind | **the whole phrase** |
| method pick | the phrase screen | **the phrase** | nothing |
| deriving (discard icon) | the method pick | the phrase | the derivation |
| confirm modal | the method pick | the phrase | nothing |
| reconcile | — (dismiss only, Button3) | — | — |
| hex pad | the kind screen | kind **and draft** (F-541 fix) | nothing |
| preimage-plate pick | = `do not cut this preimage` | the composition | that plate |

Two things are surprising:

1. **The phrase screen is the only screen on the route whose Back is
   destructive**, and it is the one an operator has the strongest reason to
   press (B2/B3 above). Its sibling, the pad, was fixed. → **F-571.**
2. **Back from the phrase screen no longer lands where the copy says.** The
   digest-shaped-phrase advisory (F-539, shipped today) reads
   *"If you meant a digest you already hold, go back and use the `Type a digest`
   row."* Measured at hash160 with a 40-hex phrase: decline → phrase screen with
   the 40 characters intact (good) → Back → **kind screen** → Back → `Which
   hash?` where the row is. Two Backs, and the first one destroys the 40
   characters. → **F-577** (Nit; the operator was just told not to use that
   text, so the loss is cheap).

---

## D. A payload arrives

Row sets measured from `composerHashRows` against real `syswSession` loads.

### D1 — the record forms, as built

```
hash160 record  = "hash:hash160:5741af53b9f8d706ccb09b5ed96fc6f5774b6515"
bare sha256 rec = "hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb"
```

### D2 — what each band tells the operator about the hash function

| payload | rows |
| --- | --- |
| bare `hash:` record | `sha256 1  b867db87..edbc96cb` |
| tagged `hash:hash160:` | `hash160 1  5741af53..774b6515` |
| `hash:hash160:` + the preimage plate **for the same X** | `hash160 1  5741af53..774b6515` / `preimage 1  sha256 b867db87..edbc96cb` |
| bare `hash:` + the preimage plate for the same X | `sha256 1  b867db87..edbc96cb  (in payload)` / `preimage 1  sha256 b867db87..edbc96cb` |
| `hash:hash160:` + a `phrase:` record | `hash160 1  5741af53..774b6515` / `phrase record 1 (derive to see the digest)` |

Band 1 answers the kind question exactly — including for a **bare** record,
which it labels `sha256`. That is the right default (§6) and reads as an answer
rather than a guess.

Bands 2 and 3 answer it **only for sha256**, because both are hard-wired:
`gui/composer_hash.go:412` builds every preimage-record row as
`hashlockLockOf(md.KindSha256, &x)` and `gui/composer_hashlock.go:147` assigns
`hashlockLockOf(md.KindSha256, &x)` for every `phrase:` record.

### D3 — the divergence, walked on screen

Payload: `hash:hash160:<hash160(X)>` **and** the preimage plate record carrying
**X itself**. Rows drawn:

> `Path1hashWhichhash?hash16015741af53..774b6515preimage1sha256b867db87..edbc96cbTypeahashlockphraseTypeadigestNohashlock`

1. **In hand:** a payload the host built for their hash160 wallet, containing
   both the digest and the preimage.
2. **Device, if they pick the correct row (`hash160 1`):**
   > `20-BYTEHASH,NOTDERIVEDHEREThisisahash160digestand**nothingonthisdevicehasseenapreimageforit**.Whoeversupplieditcanspendthispath.…`

   and afterwards `held material entries=0` — **no preimage plate at Done.**
3. **What else — if they pick `preimage 1` instead:** the confirm reads
   > `hashsha256b867db87..edbc96cbfromapreimagerecordinthispayloadnohash:recordinthepayloadhasthisdigestSpendingthispathneedsthatpreimage.…`

   and the path is assigned a **sha256** lock — a different wallet from the one
   the payload was built for. The relation line
   (`no hash: record in the payload has this digest`) is the guard, and it does
   fire, which is why this half is not the finding.

The finding is the **first** half. The device decoded X at row-build time; it is
in RAM, two rows below, and the sentence *"nothing on this device has seen a
preimage for it"* is one `hash160()` call away from being false. The copy's own
comment says the hazard it is guarding against is teaching operators to skip the
warning that matters — and this is a payload in which the warning is wrong.
→ **F-572.** The lost plate affordance → **F-573.**

Also note: with the tagged record the two rows draw **completely different
digests** (`5741af53..` vs `b867db87..`) and **no `(in payload)` annotation** —
`composerHashInPayload` compares locks, so it can never match across kinds. An
operator holding both cannot see that they are the same secret. That is part of
F-573.

**F-535, confirmed on the device:** the `phrase:` record carries the method axis
and not the kind axis, and D3 is where that is felt — the row is the operator's
only device-side use of such a record and it can produce nothing but sha256.

---

## E. Done, and the plates

All measured from the production builders (`composerPreimagePlatePick`'s lead,
`composerPreimagePlateRows`, `composerBuildHashlockPlate`,
`composerHashlockLocator`, `composerPreimageCensusLines`).

### E1 — the plate itself carries the kind, on every form and every kind

```
[hash160] PLATE form=HashlockString  ms1="ms10hashsq0zthjclhmye6edlt8v9er9k9m3dh93lplssdayrmxh6ww75uwdg52jkm0p78s9w7h"
          locator=["path 1" "hash  hash160 5741af53..774b6515"]
[hash160] PLATE form=HashlockPhrase  phrase="correct horse battery staple"
          method="method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"  qr=true
          locator=["path 1" "hash  hash160 5741af53..774b6515"]
          qrtext="hashlock v1\nmethod: pbkdf2-hmac-sha256 …\nhash: hash160\nphrase: correct horse battery staple"
```

Both halves of §13.1 hold for all four kinds: the locator row names the kind and
the QR text carries `hash: <kind>`. **Answer to the year-later question: yes**,
for the phrase forms. The operator has the phrase, the method, the kind and 16
hex characters of the digest on one plate.

**Except for the `preimage string` form, where it is a near miss.** The ms1
string is **byte-identical across all four kinds** (it encodes X, which does not
change), so the only thing on that plate distinguishing a hash160 wallet from a
sha256 one is the elided locator row. Feed that string back through
`ms decode` and you get the sha256 answer with no indication a choice was made —
which is **F-534, now with a producer**: before this cycle nothing cut a
non-sha256 preimage plate, and now the device does. That raises F-534 from a
label wart to the read-back path for a plate this firmware cuts. **Confirmation,
not a new finding.**

### E2 — the pick lead, per kind

```
[hash160] "hash  hash160 5741af53..774b6515   path 1
           phrase: 28 characters   method: hardened"
[hash160] "hash  hash160 5741af53..774b6515   path 1
           preimage held, phrase not: only the string form can be cut"
```

Names the kind, the path, and the phrase length without a character of the
phrase. No divergence across kinds.

### E3 — the census block. The accepted row names the kind; its siblings do not.

Measured for a composition with a hash160 plate accepted, a sha256 plate
declined and a ripemd160 preimage on no path:

```
Plus 1 preimage plate(s), cut first and NOT part of this backup:
path 1  hash160 5741af53..774b6515  phrase, hardened, QR
This is what this composition will cut. Plates cut in earlier runs are not known to this device and are not listed.
preimage b867db87..edbc96cb: declined, will not be cut
preimage 23caccda..e67afe11: not on any path, will not be cut
Keep each preimage plate apart from the policy plates and from the others.
```

Row 2 carries `hash160` because §13.3 put it there. Rows 4 and 5 — the two rows
about preimages the operator is **not** getting a plate for, i.e. the ones they
most need to identify later — carry 16 hex characters and no kind. §6's
both-or-neither rule, unapplied in the sibling rows of the row it was applied
to. → **F-574.**

### E4 — the path list, between assignment and Done

`composerPathLine` (`gui/composer_state.go:413`), measured at all four kinds:

```
sha256    -> "Path 1: hash only"   "Path 2: 2-of-3 + hash"
hash256   -> "Path 1: hash only"   "Path 2: 2-of-3 + hash"
ripemd160 -> "Path 1: hash only"   "Path 2: 2-of-3 + hash"
hash160   -> "Path 1: hash only"   "Path 2: 2-of-3 + hash"
```

and re-opening `Path N hash` to check draws a **fresh** `Which hash?` list that
does not say what the path currently carries. So between the kind screen and the
Done consent screen there is **no screen that answers "which kind did I pick?"**
— and the operator who wants to look has to enter a screen where a mis-tap
reassigns the hash. → **F-575.**

The Done consent screen does answer it, correctly, for all four:

```
"  hash sha256 b867db87..edbc96cb"      "  hash hash160 5741af53..774b6515"
"  hash hash256 6915f813..1a00bc26"     "  hash ripemd160 cbfb7f6b..8f193cb1"
```

and its rule body reads well at one kind and at two:

```
one : "The hash must be hash160 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. …"
two : "This wallet's hashes are sha256 and hash160. Each must be of a 32-byte value. …"
```

---

## FOLLOW-UP LIST

### F-570 — re-entering the hex pad at a narrower kind truncates the draft SILENTLY, and the pad then says the entry is complete

**Important.** Classification: **warning** (one line).

`gui/composer_hash.go:214-217` clamps a restored draft to the new kind's width
before the loop; `over := false` at `:232` is only ever set by the **live**
clamp inside the loop, so the re-entry truncation prints nothing. The live
clamp does speak (`:304`, `" - extra ignored"`).

**Reproduction (measured, real touch harness):**

1. `Path 1 hash` → `Type a digest` → rule → kind screen → `sha256`.
2. Type the 64-hex digest
   `b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb`.
   Pad reads `64of64hexsha256hash`.
3. Type one more character. Pad now reads
   `64of64hex-extraignoredsha256hash` — **the live clamp speaks.**
4. Back → kind screen. Tap `ripemd160` (one row from `hash256`, a plausible
   mis-tap on a four-row screen).
5. Pad reads
   `0123456789ABCDEFB867DB875479BCC0287352CDAA4A1755689B833840of40hexripemd160hash`
   — **`40 of 40 hex`, checkmark lit, no notice.** 24 characters were dropped.
6. Take it. The only screen between here and the policy is the §8 warning, which
   **shows no digest at all**.
7. Result: `ripemd160 b867db875479bcc0287352cdaa4a1755689b8338` — the first 40
   characters of a sha256 digest. A path nobody can open.

**Worse than saying nothing?** Yes. The pad's own shipped guarantee is
*"THE CLAMP IS SILENT NO LONGER (journey walk I-3)"* — and on this path it is
silent, while presenting the two strongest "you are done" signals the screen
has (`N of N` and a lit checkmark). The existing gate
`TestBackFromThePadClampsADraftToANarrowerKind` pins `40 of 40 hex` and asserts
nothing about the notice, so nothing fails today.

Not Critical: the Done consent screen draws `hash ripemd160 b867db87..689b8338`,
whose last eight will not match the operator's paper, so the error is catchable
before the first cut.

**Shape of the fix:** `over` is initialised from the re-entry clamp rather than
from `false` — the condition is already computed two lines above.
Owning phase: this cycle's follow-up sweep (the arm it lands on shipped today).

---

### F-571 — the phrase arm names the kind on no screen, and the only way to look costs the whole phrase

**Important.** Classification: **default** (name the kind in the title) plus
**warning/default** (carry the phrase across Back the way the pad carries the
draft).

Three consecutive screens sit between §7.1's kind screen and the first screen
that names the kind:

```
gui/composer_hashlock.go:358  layoutTitle(..., "Hashlock phrase")
gui/composer_hashlock.go:404  composerPickScreen(..., "Hashlock method", "Which method?", …)
gui/composer_hashlock.go:463  layoutTitle(..., "Deriving")
```

The pad, the sibling arm, does the opposite: `gui/composer_hash.go:322` draws
`kind.Token()+" hash"` precisely because *"an off-by-one tap on the kind screen
lands on a same-width sibling and nothing here would differ."* That reasoning is
identical on the phrase arm and stronger — there is no width to read at all.

**Reproduction (measured):**

1. `Type a hashlock phrase` → rule → kind screen → tap `hash160` (row 4).
2. Phrase screen: `…Thisscreendoesthathashingforyou.…0/100Hashlockphrase`.
   Nothing says `hash160`.
3. Type 28 characters → `…****…28/100Hashlockphrase`.
4. Press Back to check the kind. Lands on the **kind screen**
   (`HashfunctionAllfourtake…`), where the answer is a highlight.
5. Tap the kind again → phrase screen at **`0/100`**. All 28 characters gone.

`hashlockPhraseRoute` (`gui/composer_hashlock.go:58`) holds `phrase` in its own
frame and returns `hashlockBackToWhichHash` at `:61-63`, so the variable dies
with the call; `composerHashEdit`'s kind loop then re-enters with `phrase = nil`.
The pad arm avoids exactly this by hoisting `draft` **outside** the kind loop
(`gui/composer_hash.go`, the `case sel == rows.hexRow` arm).

**Worse than saying nothing?** Yes, on both halves. On a 100-character keyboard
a six-word diceware phrase is a minute of tapping, and the operator is penalised
for *checking the one axis this cycle exists to make unambiguous* — which makes
them check less, which is the failure the kind screen was built to prevent. The
title fix alone removes most of the reason to press Back and is a one-string
change; carrying the phrase makes the two arms agree about what Back means.

Secret-handling note (non-blocking by project rule): holding the phrase across
one more screen extends RAM retention, but it already survives Back from the
method pick, so this is the same accepted F-483 residue, not a new class.

Owning phase: this cycle's follow-up sweep.

---

### F-572 — the 20-byte warning says nothing here has seen a preimage, in a payload that contains the preimage

**Important.** Classification: **warning** (the sentence is false in that state).

`composerCopyTwentyByteUnseen` (`gui/composer_copy.go:951`) asserts
*"nothing on this device has seen a preimage for it"*. Its condition is
`hashlockHeld`, which only the deriving routes populate — so the sentence is a
statement about the composer's bookkeeping being presented as a statement about
the payload.

**Reproduction (measured, real touch harness):**

1. Load a payload holding `hash:hash160:5741af53b9f8d706ccb09b5ed96fc6f5774b6515`
   **and** the preimage plate record for the X that hashes to it
   (`ms10hashsq0zthjclhmye6edlt8v9er9k9m3dh93lplssdayrmxh6ww75uwdg52jkm0p78s9w7h`).
2. `Path 1 hash` draws both:
   `hash16015741af53..774b6515` and `preimage1sha256b867db87..edbc96cb`.
3. Take the **correct** row, `hash160 1`.
4. The device draws
   `20-BYTEHASH,NOTDERIVEDHEREThisisahash160digestandnothingonthisdevicehasseenapreimageforit.Whoeversupplieditcanspendthispath.…`

`composerPayloadPreimages` (`gui/composer_hash.go:412`) decoded X while building
the very screen behind this modal. The device is holding the preimage while
saying it has never seen one.

**Worse than saying nothing?** Yes — because the copy's own comment names the
hazard: *"It does NOT claim the wallet is unsafe … saying otherwise would teach
operators to skip the warning that matters."* A warning that is demonstrably
false in a payload the host built correctly does exactly that teaching.

**Shape of the fix (not prescriptive):** the check that suppresses the warning is
`hashlockHeld` membership; the same question could be asked of the payload's
decoded preimages at each kind before drawing. Note §8's own accepted residual
already allows `hashlockHeld` to suppress the warning across a composition, so
widening the suppressor is in keeping with it.

Owning phase: this cycle's follow-up sweep.

---

### F-573 — the payload's own material is a dead end for three of the four kinds

**Minor.** Classification: **documentation only** (or a row annotation).

Both material-bearing bands are hard-wired to sha256:

```
gui/composer_hash.go:412      digest: hashlockLockOf(md.KindSha256, &x)     (preimage plate records)
gui/composer_hashlock.go:147  h := hashlockLockOf(md.KindSha256, &x)        (phrase: records)
```

so on a hash256/ripemd160/hash160 payload:

* the `preimage N` row assigns the **wrong kind** (caught by the relation line
  `no hash: record in the payload has this digest` — measured, it does fire);
* the correct `hash:` row assigns the right kind but holds **no material**
  (`held material entries=0`), so **no preimage plate is offered at Done** even
  though the preimage is in flash;
* the two rows draw **unrelated digests** and the `(in payload)` annotation
  cannot appear, because `composerHashInPayload` compares locks across kinds.

The only device route to a non-sha256 lock *with* material is to retype the
phrase by hand on the phrase arm. Nothing on any screen says so.

**Worse than saying nothing?** Marginally. The operator is warned at every wrong
turn (relation line, §8 warning), so no wrong wallet is built silently — what is
missing is the sentence that would save them the search. A row annotation on the
preimage band (`preimage 1  sha256 … (sha256 only)`) or one line in the §8
warning would close it.

Directly adjacent to **F-535** (the `phrase:` record carries method, not kind) —
close them together; this is the device-side consequence F-535 predicted.
Owning phase: post-cycle CLI/UX work, with F-535.

---

### F-574 — the census names the kind on the rows it cuts and not on the rows it does not

**Minor.** Classification: **default**.

```
gui/composer_copy.go:962  "preimage " + first8last8 + ": not on any path, will not be cut"
gui/composer_copy.go:967  "preimage " + first8last8 + ": declined, will not be cut"
```

against the accepted row, `composerCopyPreimagePlateRow`, which was given the
kind precisely so that *"two plates for one preimage under two kinds"* are not
described identically.

**Reproduction (measured):** a composition with a hash160 plate accepted, a
sha256 plate declined and a ripemd160 preimage on no path draws

```
path 1  hash160 5741af53..774b6515  phrase, hardened, QR
preimage b867db87..edbc96cb: declined, will not be cut
preimage 23caccda..e67afe11: not on any path, will not be cut
```

**Worse than saying nothing?** Weakly yes. These two rows are about the
preimages the operator is walking away *without*, and the census is their only
inventory. Both take the kind as a parameter change of the same shape §13.3
already made next door.

Owning phase: this cycle's follow-up sweep.

---

### F-575 — no screen between assignment and the Done consent answers "which kind did I pick?"

**Minor.** Classification: **default**.

`composerPathLine` (`gui/composer_state.go:413`) renders `Path 1: hash only` and
`Path 2: 2-of-3 + hash` for **all four kinds** (measured). Re-opening
`Path N hash` draws a fresh `Which hash?` whose rows describe the *payload*, not
the path's current assignment — so "let me check" means entering a screen where
selecting a row reassigns the hash, and the only non-destructive exit is Back.

**Worse than saying nothing?** Weakly. The Done consent screen does name the kind
before anything is cut, so nothing is lost — what is lost is the chance to catch
a mis-tap before seating every key, and the cost of the late catch is retyping
the digest or the phrase. `Path 2: 2-of-3 + hash160` is the obvious shape;
it needs a row-width measurement at `sh2DisplaySize` (`ripemd160` is the long
token) before it is written.

Owning phase: post-release UX.

---

### F-576 — (withdrawn) the pad draws hex uppercase

Not filed. Measured (`B867DB87…` on the pad vs `b867db87..edbc96cb` everywhere
else) and traced to `ctx.Styles.word` = `comfortaa.Bold17`
(`gui/theme.go:104`), shared with BIP-39 word entry (`gui/gui.go:1313`).
Device-wide, older than this cycle, identical for all four kinds.
**Not our concern.** Recorded so the next walker does not re-measure it.

---

### F-577 — "go back and use the `Type a digest` row" is one screen short, and the Back it asks for is the destructive one

**Nit.** Classification: **documentation only**.

`composerCopyPhraseLooksLikeDigest` (F-539, shipped today) tells the operator to
go back to a row that is now **two** Backs away, and the first Back drops the
typed phrase.

**Reproduction (measured, kind = hash160):** type
`5741af53b9f8d706ccb09b5ed96fc6f5774b6515` on the phrase screen → OK →

> `THISLOOKSLIKEADIGESTYoutyped40hexcharacters,thewidthofadigest.Thisdevicewillhashthosecharacters,sothewalletcommitstotheTEXTyoutypedandnottothedigestitspells.Ifyoumeantadigestyoualreadyhold,gobackandusetheTypeadigestrow.Continue?`

Decline → phrase screen, **40 characters intact** (correct, the decline arm keeps
them). Back → **kind screen**. Back → `Which hash?`, where the row is.

**Worse than saying nothing?** No, and that is why it is a Nit: the text being
dropped is text the operator has just been advised not to use. Recorded because
it is the same mechanism as F-571 and should be re-read when F-571 is folded —
if the phrase starts surviving Back, this copy becomes accurate about the cost
and only the screen count is left.

Owning phase: post-release UX, with F-571.

---

## Confirmations of already-filed entries (no new numbers)

* **F-534** (`ms decode`'s preimage route answers only for sha256) — the device
  now **cuts** the plates that route reads. The `preimage string` form is
  byte-identical across all four kinds (E1), so the ms1 string alone cannot
  answer the question and `ms decode` answers it wrongly-by-default. F-534's
  severity should be re-read now that a producer exists.
* **F-535** (`phrase:` carries the method axis and not the kind axis) — the
  device-side consequence is F-573 above: the `phrase:` band can only ever build
  a sha256 lock.
* **F-540** (two spellings of the method) — the journey makes the collision
  worse than the original walk assumed, because the method pick names no kind
  (B4), so at hash160 the operator is looking at a screen whose only token is
  `sha256`. Noted on F-540 rather than re-filed; F-571's title fix would resolve
  it without touching either spelling.
* **F-541** (Back from the pad discarded the digest) — **fixed and verified**:
  the draft survives, the kind is re-seeded, and pressing Button3 straight
  through after a Back reopens the same kind's pad. F-570 is the defect the fix
  introduced, not a regression of F-541.
* **F-539** (digest-shaped phrase warns) — **fixed and verified at 40 hex**, the
  width this cycle created. Advisory fires at hash160, declining keeps the text.
