# Journey walk — the Wallet Policy composer

Fork `main` at **812ff06**, walked in `cmd/emu` (built from my own worktree
`/scratch/code/shibboleth/.tmp/seedhammer-journey`, served on a fresh port,
driven with Playwright through `shTap`/`shPress`/`shRelease`/`shScreen`/
`shTargets`/`shSysw`/`shComposerPathHashes`). Nothing committed.

**The journey.** *"I want a wallet my heir can spend after a year, that I can
spend any time, and that a hashlock can open."* Three spend paths under one
wrapper: a plain key path, a key path with a relative timelock, and a key path
with a hashlock and an absolute timelock. Walked once under `Segwit (wsh)` and
once under `Taproot (tr)`, then a third short run with two slots so the policy
would fully seat — because both journey walks end key-less, and the key-less
ending hides several screens.

Method: at every step, what does the operator have, what does the machine do,
and **what else might they reasonably do**. Every divergence is classified
(refusal / warning / default / not our concern / documentation only) and earns
a change only where the wrong outcome is worse than saying nothing.

---

## Counts

**1 Critical · 10 Important · 10 Minor · 6 Nit.**

The Critical is C-1: the script picker defaults to Taproot rather than to the
script in force, so opening it to check which wrapper is set and leaving by the
forward button silently rebuilds the wallet as Taproot. I-7 and M-10 are its
detection gap — the path list never names the wrapper and the Review never
names it either, so in the key-less flow nothing downstream can catch it.

---

## Walk 1 — Segwit (wsh)

### Getting in

| step | what happened |
| --- | --- |
| Boot | `A systemwide payload is present. Load it? LOAD / SKIP` |
| Skipped it, opened `Wallet Policy` | `A payload is in flash but not loaded. Load it from the carousel first.` + `Scan cards` / `Build a new policy` |
| Loaded from the carousel with `shSysw("composer")` | `Payload Digest … dbe9e774e9a492310b62626c2b41cf4b` → `Payload Warnings: A SECRET is stored unencrypted in flash.` → `Keep this payload loaded? KEEP / UNLOAD` → KEEP |
| `Wallet Policy` again | `Keys loaded: 2, plus 1 seed.` + `Scan cards` / `Build a new policy` |

The skipped-payload case is handled well: the program names the condition and
the remedy in one line. Worth recording as a thing that works — it is exactly
the moment a journey usually finds a silent dead end.

Screen order thereafter matched the brief: `Which script?` → `Start from?` →
`Build my own paths` → `Add a spend path` → `What can spend on this path?`.

### Probing `Start from?` before building

`Start from?` offers `Build my own paths` and six templates
(`plain-multisig`, `simple-timelocked-inheritance`, `kofn-recovery`,
`tiered-recovery`, `hashlock-gated`, `decaying-multisig`). My journey is
literally a hashlock-gated inheritance wallet, so I took `hashlock-gated`
first — what any operator with this goal would do.

It dropped straight into a prefilled path list:
`Path 1: 1 key + hash` / `Path 2: 1 key + 26280 blocks`. No description screen,
no confirmation. That is defensible (the result is immediately visible and
editable) — recorded as **N-5** only.

Backing out and choosing `Build my own paths` returned **the same two paths**.
That is **I-2**, measured twice (`hashlock-gated` then `plain-multisig`; each
template replaced the list, `Build my own paths` never cleared it).

Leaving the program entirely and re-entering did give an empty list
(`slots: 0 / keys available: 2`), so the defect is specifically that
`Build my own paths` is a no-op rather than a reset.

### Building the three paths

`Add a spend path` → `What can spend on this path? Keys / A hash, no keys` →
`Path N: how many keys? 1 2 3 4 5` → `Threshold — Path N: how many must sign?`
(one row at n=1, **N-3**) → back to the path list.

**Path 2's relative lock.** `Timelock` → `What kind of timelock? None /
After a wait / After a date or height` → `Measured how? Blocks / Days` →
`How many days? 1 to 388 days`. The live readout is genuinely good:
`365 days = 61594 units of 512s (365.0 days)` — it shows the encoded value *and*
the true duration after rounding. I verified the rounding independently
(365 d = 31 536 000 s; /512 = 61 593.75 → 61 594).

Divergences tried here:

- `389` → `Relative locks reach at most 455 days in blocks or 388 days in time.
  Use an absolute date.` Refused **in place**, digits kept, and the route out is
  named. Very good.
- `0` → the **same** maximum-range message (**M-6**).
- `0365` → the field shows `036` and reads *36 days*. The `5` was silently
  dropped (**I-1**).

**Path 3's hashlock.** `Hashlock` → `Which hash? hash1 abababab..abababab /
Type a hashlock phrase / Type 64 hex / No hashlock`. (Note: the composer
payload at 812ff06 *does* carry a `hash:` record, contrary to the brief's
hand-over note.) Then §8i's rule screen, then the keyboard
(`Hashlock phrase`, `0/100`, masked, `show` available), then
`Hashlock method: Hardened (about 10s) / SHA-256`, then the brainwallet warning
(hold-gated; a plain tap correctly does nothing and the screen says
`Hold button to confirm`), then the digest screen, then a host cross-check
screen.

`correct horse battery staple` under `sha256` produced
`b867db87..edbc96cb`. `shComposerPathHashes()` read `[null,null,null]` on the
digest screen and
`[null,null,"b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb"]`
only after the hold — the F-485 ordering holds. I recomputed it off-device:
`sha256(sha256("correct horse battery staple"))` is byte-identical. **The
composer's hashlock arithmetic is correct.**

**Path 3's absolute lock.** `After a date or height` → `Named how? A date /
A block height` → `Date as YYYYMMDD`. Four validations, three of them inline
and excellent:

| input | behaviour |
| --- | --- |
| `20270230` | `that date does not exist` — inline, digits kept |
| `99991231` | `This build writes dates up to 2038-01-19. For a later time, use a block height instead.` — inline, digits kept, route out named |
| `20250101` | `That is before this payload was packed. Choose a later date.` — **full-screen modal that ejects to the path menu** (**I-3**) |
| `20270913` | accepted, with an honest caveat screen naming the pack date |

### The tail

`Done` is on page 2 of the path list once there are three paths (**M-10**).

`Template` (2 pages) → `Seat keys` → census → `What now?` → `Key mapping`
(2 pages) → `Review` (4 pages) → `Before you fund it` (hold) →
`What to engrave` → `Preimage plate` → `Plates To Cut` (3 pages) →
`Engrave Plate`.

At seat time I deliberately put the `m/48h/0h/1h/2h` key into slot @0, which
the Template screen had just said *expects* `m/48h/0h/0h/2h`. Accepted with no
comment. The `Key mapping` screen then showed `@0: 73c5da0a m/48'/0'/1'/2'`
without flagging it against the stated expectation — and in the opposite
notation (**N-1**). This is the wsh instance of what becomes systematic under
tr (**I-4**).

`Leave unseated` for @2 gave a clean census — `3 slots, 2 keys available.
Unfilled: slot @2.` — and `What now? Back to the paths / Engrave a key-less
template`. No option to engrave a partial wallet: correct and safe.

---

## Walk 2 — Taproot (tr)

Same three paths, built from a fresh composition. The path list is
**byte-identical** to walk 1's at every step — nothing on it names the wrapper.

Differences that matter, and the composer gets the important ones right:

- Template slot paths become `m/48h/0h/Nh/**3h**` (taproot) rather than `2h`.
- Slot @0 is presented as **`Slot @0, key path (spends alone): choose a key`** —
  the composer promotes the single-key Path 1 to the taproot internal key and
  says so, with the consequence spelled out. That is the right answer to
  "which key becomes the internal key", delivered at the right moment.
- `Review` page 2 carries **`Key-path (Path 1): A KEY CAN SPEND ALONE`** in
  caps.

And the ones it gets wrong:

- Every key the payload offers for those `3h` slots is derived at `2h`. The
  machine states the expectation and then offers only keys that violate it,
  silently (**I-4**).
- `Review` page 1 under tr begins at `Path 2` with no note that Path 1 became
  the key path (**M-9**).
- `Key mapping` lists @0 and @1 identically; nothing marks which one spends
  alone (**M-5**).

I took the `Hardened` method this time; digest `3cf5d421..b70a4c12`, no
brainwallet warning (correct), and the digest screen still insists on recording
the method.

The `phrase + method + QR` preimage option is hold-gated behind a warning I
thought was very well judged: *"The QR makes the phrase readable by any camera.
A photograph of the plate is a copy of the phrase, and the phrase spends this
path."*

---

## Walk 3 — the fully-seated ending (two slots, tr)

Both journey walks end key-less, so I built a two-slot tr policy that seats
completely. The ending is materially different and much better:

- `Template` gains a **Policy-ID** alongside the Template-ID, and says
  `Stamp BOTH stubs on each key card: --policy-id-stub 4f5306f9
  --policy-id-stub 11a2c2ae` — while the `mk encode` command on the very next
  page carries only one of them (**I-9**).
- `Review` carries `Receive 0/1` and `Change 0/1` addresses
  (`bc1p908x6pty…`, taproot). This is the off-device check the final gate asks
  for, and it exists only here.
- `What to engrave: Which form? The policy itself / Template plus key cards`.
  `The policy itself` gives a one-plate census on a single page.

This is what makes **I-10** a real finding rather than a quibble: the final
gate's instruction is exactly right in this flow and impossible in the one both
journey walks reached.

I also used this run to walk `Change the script`, which produced the Critical.

---

## Findings

### C-1 — `Change the script` defaults to Taproot, so looking at the setting changes it

**Step.** Path list → `Change the script`.
**Operator has.** A composition they set to `Segwit (wsh)`, and a path list that
never says so.
**They do.** Open `Change the script` to check which wrapper is in force, then
leave by the forward button (✓) — the same control that has advanced every
other screen in this flow.
**What happens.** The picker's selection is row 0, `Taproot (tr)`, not the
script currently in force. ✓ commits row 0. The wallet becomes Taproot.
**Expected.** The picker shows which script is set, and leaving without
choosing changes nothing.

Measured, at 812ff06:

| action | Template-ID after `Done` |
| --- | --- |
| set `Segwit (wsh)` explicitly | `730513330db452b8e831426938b4f3d2` |
| open picker, tap ✓ **without selecting a row** | `4f5306f9c6b23da7569b31f1e6039801` |
| set `wsh` again, open picker, leave via **←** | `730513330db452b8e831426938b4f3d2` |

`4f5306f9…` is the id this same shape produced under Taproot earlier in the
session, so ✓ did not merely re-hash — it switched the wrapper. ← is safe.

Nothing downstream reports the change in the key-less flow: the path list is
byte-identical under every wrapper (I checked wsh, tr and sh), and the Review
never names the script (**I-7**). In the seated flow the address prefix
(`bc1p` vs `bc1q`) is the only signal, and only to an operator who reads
prefixes.

**Classification: default.** **Severity: Critical** — a reasonable action
produces a different wallet than the one specified, with no notice at the time
and no signal afterwards on the path both journey walks took.

**Suggestion.** Preselect the script currently in force. Then ✓ is a no-op and
the screen also becomes the place to *read* the setting, which is what the
operator opened it for.

---

### I-1 — A leading zero silently costs a digit: `0365` becomes 36 days

**Step.** `How many days?` (relative wait).
**They do.** Type `0`, see the red range message, then type `365` on top
without clearing — a very ordinary recovery sequence.
**What happens.** The field holds three characters and silently ignores
everything after. `0365` shows `036`, and the readout reads
`36 days = 6075 units of 512s (36.0 days)`. Confirmed with `12345` → `123`:
the 4th and 5th keypresses produce **no feedback of any kind**.

The resulting value *is* displayed in plain language before confirming, which
is why this is not Critical. But a keypress that does nothing and says nothing
is how a wrong number lands, and here the wrong number is an heir's timelock —
36 days instead of 365.

**Classification: warning (or better, a refusal of the leading zero).**
**Severity: Important.**

**Suggestion.** Don't append `0` to an empty buffer, and give some feedback
when a keypress is discarded.

---

### I-2 — `Build my own paths` does not clear a template's paths

**Step.** `Start from?`.
**They do.** Open a template to see what it builds, press ← because it is not
what they wanted, then choose `Build my own paths`.
**What happens.** The template's spend paths are still there.

Measured: `hashlock-gated` → ← → `Build my own paths` →
`Path 1: 1 key + hash / Path 2: 1 key + 26280 blocks`. Then `plain-multisig` →
← → `Build my own paths` → `Path 1: 2-of-3`. Templates replace the list;
`Build my own paths` never clears it. On a first, unvisited entry it does show
`slots: 0`, which is what makes the behaviour look correct until you back out
of a template once.

The same defect has a second motive: an operator who has made a mess and wants
to start over will reach for `Build my own paths` and get their mess back.

If they then add their own three paths and press `Done`, the policy carries
extra spend paths they never asked for — under `hashlock-gated`, a hashlock
path whose preimage nobody holds. It is visible on the path list, so it is
missable rather than invisible.

**Classification: default.** **Severity: Important.**

**Suggestion.** Make `Build my own paths` clear the composition, or rename it
to something that does not promise a blank start (`Keep what I have`).

---

### I-3 — The past-date refusal ejects the operator out of the editor

**Step.** `Date as YYYYMMDD`.
**They do.** Type a date they believe is in the future — or make a one-digit
typo, `20260913` for `20270913`.
**What happens.** `That is before this payload was packed. Choose a later
date.` appears as a full-screen modal titled `Timelock`. The only forward
control dismisses it to **Path N's own menu**, with no timelock set. The eight
digits are gone and the operator must re-navigate
`Timelock` → `After a date or height` → `A date` and retype.

Reproduced twice (`20200101`, `20250101`). The first time, my next taps landed
on the path menu and re-opened the hashlock flow — the ejection is
disorienting enough that blind continuation lands somewhere unrelated.

This is a defect rather than a design choice because the *same field* handles
its other two failures correctly and in place: `20270230` gives
`that date does not exist` inline, and `99991231` gives
`This build writes dates up to 2038-01-19…` inline. Only the most likely
operator error gets the destructive treatment.

**Classification: refusal, wrong shape.** **Severity: Important.**

**Suggestion.** Make it an inline message like the other two, keeping the
digits.

---

### I-4 — Under `tr`, every offered key violates the slot's stated origin, silently

**Step.** `Template` then `Seat keys`, under Taproot.
**Operator has.** A Template screen that just told them
`Slot @0 expects a key at m/48h/0h/0h/3h`.
**They do.** Seat a key — there is nothing else to do.
**What happens.** The only candidates are `73c5da0a m/48h/0h/0h/2h` and
`73c5da0a m/48h/0h/1h/2h`. Both are P2WSH-script-type keys. Seating one is
accepted without a word, and `Key mapping` then prints
`@0: 73c5da0a m/48'/0'/0'/2'` with no reference to the `3h` it asked for.

This is not the operator picking the wrong candidate — *every* candidate
mismatches, systematically, and the machine stated the requirement two screens
earlier. The consequence is a coordination hazard: a cosigner who re-derives
from the stated `m/48h/0h/0h/3h` gets a different key, and the descriptor will
not match.

Under `wsh` the expectation was `2h` and the payload keys were `2h`, so the
mismatch is specific to the taproot wrapper.

The wsh walk showed the same class from the other direction: I seated the `1h`
key into the slot that expected `0h` and nothing objected, on the seat screen
or on the review-grade `Key mapping`.

**Classification: warning.** Refusing would make taproot unusable with this
payload, so a warning is the right remedy. **Severity: Important.**

**Suggestion.** Mark the row at seat time and on `Key mapping`, e.g.
`@0: 73c5da0a m/48'/0'/0'/2' — expected …/3'`.

---

### I-5 — "The shape changed, so this id changed" fires when nothing changed

**Step.** `Template`, on any revisit.
**They do.** Reach the plate census, press ← to re-read something, then
`Done` again — changing nothing.
**What happens.** The Template screen now reads
`The shape changed, so this id changed. Cards minted with the old stub will not
seat here.` — and prints `Template-ID: cabb49b6c4d24d552e5013231bb5f673`,
`mk1 stub (template): cabb49b6`. Byte-identical to the first pass. Nothing
changed.

I confirmed the banner is *sometimes* true: after genuinely editing Path 1 from
1 key to 1-of-2, the id really did move to
`43e0b444e55e5e25e94e269722030864`. So the banner is not decorative — it is
shape-sensitive in one direction and fires spuriously in the other.

An operator who has already minted cosigner cards with `cabb49b6` — which is
precisely what this screen instructs them to do — reads that their cards are
now useless. Re-minting means real work and, in a multisig, other people.

**Classification: warning, false positive.** **Severity: Important** — a false
claim about card validity on the screen that governs card minting. Not Critical
only because the unchanged id is printed two lines below the claim.

**Suggestion.** Compare the ids and show the banner only when they differ;
better, show both (`was cabb49b6, now 43e0b444`).

---

### I-6 — Leaving the composer destroys the composition, and the hashlock phrase, with no warning

**Step.** Any composer screen.
**Operator has.** Three paths, a 365-day lock, an absolute date, and a hashlock
phrase that exists **only in this composition** — the machine said so itself:
*"Write down this phrase, the method and this digest now. This composition holds
them until it ends."*
**They do.** Press ← once too many.
**What happens.** Four ← taps from the path list reach the carousel. No
confirmation at any point. Re-entering gives `slots: 0` — everything is gone,
including a phrase the machine cannot show again.

The asymmetry is the tell: a mere *shape edit* is hold-gated with
`EDITING THE SHAPE CLEARS THE KEYS / Slot numbers change with the shape. Every
key you seated will be cleared. Continue? Hold button to confirm.` — but ending
the composition, which destroys strictly more, is ungated.

**Classification: warning.** **Severity: Important.**

**Suggestion.** Gate the last ← out of a non-empty composition with the same
hold, naming what is lost (`N paths, and a hashlock phrase this device cannot
show again`).

---

### I-7 — The Review never names the script wrapper

**Step.** `Review`, before the hold gate.
**Operator has.** A wallet they are about to cut into metal.
**What happens.** All four Review pages under `wsh` — paths, Template-ID,
the hash rule, `Keyless template - no addresses. Verify off-device.` — never
say `wsh`, `Segwit`, `tr` or `Taproot`. Under `tr` the closest thing is
`Key-path (Path 1): A KEY CAN SPEND ALONE` on page 2, which implies taproot to
someone who already knows.

The wrapper was chosen fifteen-plus screens earlier, can be changed mid-flow
via `Change the script`, and decides which paths are legal and where the money
goes. It is the one parameter the Review omits.

For a **seated** policy the address prefix is an indirect signal. For the
**key-less template** — the ending both journey walks reached — there are no
addresses, so there is no signal at all.

**Classification: documentation (one line on the Review).**
**Severity: Important** — this is what makes C-1 undetectable.

**Suggestion.** Put the wrapper on Review page 1, in the words the operator
chose it with.

---

### I-8 — An illegal wrapper is accepted; the refusal arrives after the damage

**Step.** `Change the script` → `Legacy (sh)`.
**They do.** Switch to Legacy on a composition with two paths, one of them
timelocked.
**What happens.** Accepted silently. The hold gate that precedes the change
(`EDITING THE SHAPE CLEARS THE KEYS`) has already cleared the seated keys. The
path list returns looking exactly as before. Only on `Done` does the machine
say `Legacy wrappers hold one plain multisig only. Use wsh or tr.`

The refusal itself is good — it names the constraint and the route out. It is
the *timing*: the machine knows the current paths when the operator picks the
wrapper, and could refuse at `Which script?` before the seating is destroyed.

The operator is then left on a normal-looking path list that refuses to finish,
with nothing on it indicating the wrapper is the blocker.

**Classification: refusal, too late.** **Severity: Important.**

**Suggestion.** Refuse (or annotate) the incompatible rows on `Which script?`.

---

### I-9 — "Stamp BOTH stubs" and the command that stamps one

**Step.** `Template`, fully seated.
**What happens.** Page 1:

```
Policy-ID: 11a2c2ae2d2d6fbd26f6c9568e7c2358
mk1 stub (policy): 11a2c2ae
Stamp BOTH stubs on each key card:
  --policy-id-stub 4f5306f9  --policy-id-stub 11a2c2ae
```

Page 2, the command the operator will actually copy:

```
mk encode --xpub <xpub> --origin-fingerprint <fp> --origin-path <path>
          --policy-id-stub 4f5306f9
```

One stub. The instruction and the executable form of the instruction disagree,
on adjacent pages of the same screen, and the operator will use the one that
looks like a command line. This machine has already warned them that cards
minted with the wrong stub will not seat.

**Classification: documentation.** **Severity: Important.**

**Suggestion.** Make the page-2 command carry both stubs once a Policy-ID
exists.

---

### I-10 — The final gate asks for a check the artefact cannot support

**Step.** `Before you fund it`, the last hold before cutting.
**Operator has.** A key-less template — Review page 4 has just said
`Keyless template - no addresses. Verify off-device.`
**What happens.** The gate reads: *"Nothing outside this device has checked
this policy. Before you fund it, restore these plates in your coordinator and
compare your own first receive address."*

A key-less template has no keys, so no coordinator can derive a receive address
from it. In the fully-seated flow this instruction is exactly right — the
Review even prints the addresses to compare. In the key-less flow it is
impossible, on the screen whose entire job is to be the last safety gate.

An operator who cannot perform the instructed check either stops and
investigates, or — the outcome that matters — decides the check does not apply
and stops reading this gate.

**Classification: warning, missing case.** **Severity: Important.**

**Suggestion.** Give the key-less arm its own sentence — what to compare is the
Template-ID against `mk encode`'s, not an address.

---

### M-1 — Nothing says what the relative wait is measured from

**Step.** The whole relative-timelock leg, and the Review.
**Operator has.** The intent *"my heir can spend after a year"*, which they
almost certainly mean as a year from now.
**What happens.** `After a wait` → `Measured how?` → `How many days?` →
`Path 2 lock: 365 days = 61594 units of 512s (365.0 days)` → Review:
`Path 2: 1 key / 365 days = 61594 units of 512 s (365.0 days)`. At no point
does any screen say the wait is counted **from when each coin is received** —
so topping the wallet up restarts the heir's clock for those coins.

The contrast is what makes this an omission rather than a deliberate silence:
the *absolute* branch gets a full honest paragraph — *"This device cannot tell
the time. The payload says it was packed on 2026-09-01, which may be long ago.
Nothing here has checked that this is in the future."* — while the relative
branch, whose semantics people actually get wrong, gets nothing. On the Review
the two sit side by side: an explicit UTC date for Path 3, a bare duration for
Path 2.

**Classification: documentation.** **Severity: Minor** — the machine faithfully
encodes what it offered (`After a wait`), so there is no wrong result; it is a
missing sentence at the moment of commitment. I considered Important and
stopped short only because nothing the machine says is untrue.

**Suggestion.** One clause on the confirm and Review lines:
`365 days after each coin is received`.

---

### M-2 — The blanks instruction sits with the count that excludes the preimage plate

**Step.** `Plates To Cut`.
**What happens.** Page 1: `This engraves 3 plates. … Each plate takes minutes
to cut. Have that many blanks ready before you start: a set is only a backup
when all of it exists.` Page 2: `Plus 1 preimage plate(s), cut first and NOT
part of this backup: path 3 3cf5d421..b70a4c12 phrase, hardened, QR`.

I first read page 1 as the whole census and reported an undercount; it is not
one, and the separation is deliberate and correct — the preimage plate is not
part of the backup set. But the *actionable* sentence, go and fetch N blanks,
is on the page carrying the number that excludes it. Four plates will be cut
and page 1 says three.

Measured control: choosing `do not cut this preimage` gives an identical page 1
(`3 plates`) and page 2 reads `preimage b867db87..edbc96cb: declined, will not
be cut`. So page 1 cannot distinguish the two cases at all.

**Classification: documentation.** **Severity: Minor.**

**Suggestion.** Put the blank count where the total is known, or say
`3 backup plates plus 1 preimage plate — have 4 blanks ready`.

**Not established:** whether a first-time operator can confirm from page 1
without paging. ✓ *was* available on page 1 in my run, but I had already paged
to the end of that census; `gui/composer_preimage_plate_test.go` says the
checkmark is withheld until the last page has been laid out once. Worth one
mechanical check — if the forced read holds, this drops to a Nit.

---

### M-3 — §8i's rule screen says "passphrase", and never says the flow does the hashing

**Step.** Between `Type a hashlock phrase` and the keyboard.
**What happens.** A full-screen information page: *"The hash must be SHA-256 of
a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed
again. A hash of the passphrase itself can never be spent."* Only ← and ✓.

Two things, and the confusion was mine before it was anyone else's — I could
not tell from this screen whether the machine was about to do the two-step for
me or warning me not to try.

1. It calls the thing a **passphrase**, twice. Ruling L2
   (`design/BRAINSTORM_hashlock_phrase.md:38`, agreed 2026-09-03) is explicit:
   *"the memorable text is the **hashlock phrase** (two words; never
   'passphrase')"*. The menu row the operator just tapped says
   `Type a hashlock phrase`; the next screen is titled `Hashlock phrase`. This
   screen is the odd one out, and it is the one explaining the rule.
2. The reassurance exists — *"This screen does that hashing for you"* — but on
   the **next** screen. An operator who reads the rule screen as a warning and
   presses ← never sees it. Worse, a plausible misreading is *"I must pre-hash
   my phrase myself"*, and acting on that via `Type 64 hex` with
   `sha256(phrase)` produces exactly the unspendable case the screen warns
   about.

**Classification: documentation.** **Severity: Minor.**

**Suggestion.** Say "hashlock phrase", and move
*"This screen does that hashing for you"* onto the rule screen.

---

### M-4 — The past-date refusal does not name the date it is refusing against

`That is before this payload was packed. Choose a later date.` gives the
operator no way to pick a valid date except by guessing. The machine knows the
answer and prints it on the *success* screen: *"The payload says it was packed
on 2026-09-01."*

**Classification: warning.** **Severity: Minor.** **Suggestion.** Name the
date in the refusal.

---

### M-5 — `Key mapping` does not mark the taproot internal key

Under `tr`, `Key mapping` lists `@0` and `@1` identically. But `@0` is the key
path and **spends alone** — the seat screen says so
(`Slot @0, key path (spends alone)`) and the Review says so in caps. The
review-grade mapping screen, the one an operator scans to check who holds what,
does not.

**Classification: documentation.** **Severity: Minor.**

---

### M-6 — Entering `0` in the wait field shows the maximum-range message

`0` produces `Relative locks reach at most 455 days in blocks or 388 days in
time. Use an absolute date.` — a message about ceilings, for a floor violation.
CONFIRM is correctly refused. **Severity: Minor.**

---

### M-7 — "Write down this phrase" on a screen that does not show the phrase

The digest screen says *"Write down this phrase, the method and this digest
now."* It shows the digest, `method: sha256` and `chars: 28` — not the phrase.
The phrase was masked on the keyboard (revealable with `show`, two screens
back) and there is no way to see it again from here.

The instruction is right and the moment is right; the screen is one where the
operator cannot comply without already having complied.

**Classification: documentation.** **Severity: Minor.**
**Suggestion.** Say *"the phrase you just typed"*, or offer `show` here too.

---

### M-8 — The engrave prompt never says which plate is being cut

`Engrave Plate / Insert a blank plate and close the lock. / Hold button to
start the engraving process. The process is loud, use hearing protection.`
Single page, verified against the pixels — ← and the hold button only.

With four plates queued and the census insisting *"Keep each preimage plate
apart from the policy plates and from the others"*, the screen that hands the
operator each plate does not name it. They must track the order themselves,
and page 1 of the census does not list the preimage plate in that order.

Mitigated by the fact that the finished plate is readable, and that the census
says the preimage plate is cut first.

**Classification: documentation.** **Severity: Minor.**

**Not verified by me:** that the preimage plate really is cut first. I did not
run an engrave.

---

### M-9 — Under `tr`, Review page 1 starts at "Path 2"

`Review` page 1 lists `Path 2` and `Path 3` and no `Path 1` — because Path 1
became the key path, which is reported on page 2. An operator counting their
paths on the first page sees two of the three they built.

**Classification: documentation.** **Severity: Minor.**
**Suggestion.** A line on page 1: `Path 1 is the key path — see next page`.

---

### M-10 — The path list clips `Done` with no "more" indicator

At three paths the list fills with `Path 1 / Path 2 / Path 3 / Add a spend
path / Change the script` and `Done` moves to page 2, reachable only via the
right-edge → . The screen looks complete: no partial row, no `1/2`, no
ellipsis. The → is present and is the natural next control, so this is
discoverable rather than a dead end — but the operator has to guess that → means
"more rows" rather than "next screen".

**Classification: documentation.** **Severity: Minor.**

---

### N-1 — Two hardened-path notations on the two screens that must be compared

`Template` writes `m/48h/0h/0h/2h`. `Key mapping` writes `m/48'/0'/0'/2'`.
Those are the two screens an operator must read against each other to notice
I-4, and they are spelled differently.

### N-2 — "Nothing here has checked that this is in the future" understates

A floor check demonstrably exists — it refused `20250101` as *"before this
payload was packed"*. The sentence is literally true (the check is against the
pack date, not against now) and reads as though no check exists at all.

### N-3 — The threshold screen is asked at n = 1, with one row

`Threshold — Path 1: how many must sign?` with the single option `1`. Harmless,
but it trains the operator to tap ✓ without reading, and the next screen in
this flow is a key-seating decision.

### N-4 — `no hash: record in the payload has this digest` parses badly

Meaningful to someone who knows the payload record format; to anyone else
`no hash` reads as a noun phrase.

### N-5 — `Start from?` describes none of its six templates

`kofn-recovery`, `tiered-recovery`, `decaying-multisig` are identifiers, not
English, and choosing one commits immediately. Mitigated: the result is
visible and editable on the next screen (modulo I-2).

### N-6 — `Hashlock method` gives no basis to choose

`Hardened (about 10s)` / `SHA-256`. The only annotation is a time cost, which
makes the weaker option look cheaper. The consequence of `SHA-256` is disclosed
only after choosing it, in the brainwallet warning — which is also where the
advice *"use six diceware words"* first appears, i.e. after the phrase has
already been typed. Reordering the method question before the phrase would put
that advice where it can still change what the operator invents. Kept at Nit
because the warning is clear, hold-gated, and offers Back.

---

## What works, and is worth not breaking

Not a list of defects only — several of these are the reason the walk was as
short as it was:

- The **no-payload hint** on entry names both the condition and the remedy.
- **Relative-wait feedback** — `365 days = 61594 units of 512s (365.0 days)` —
  shows the encoding *and* the post-rounding truth.
- **Three of the four date validations** are inline, keep the digits, name the
  limit and name the route out (`use a block height instead`).
- The **absolute-lock caveat** is unusually honest about what the device cannot
  know, and names the pack date.
- The **brainwallet warning** is specific (`10^10 phrases per second`),
  actionable (`use six diceware words`) and hold-gated; a tap does nothing and
  the screen says so.
- The **QR warning** — *"A photograph of the plate is a copy of the phrase"* —
  is the clearest sentence in the flow.
- The hash is **assigned only after the hold** (`shComposerPathHashes()` read
  all-null on the digest screen), and the stored digest is byte-identical to an
  off-device recomputation.
- **`EDITING THE SHAPE CLEARS THE KEYS`** is hold-gated and correctly fires
  only when keys are actually seated.
- Under `tr`, **`key path (spends alone)`** at seat time and
  **`A KEY CAN SPEND ALONE`** on the Review are exactly the right words at
  exactly the right moments.
- With unfilled slots the machine offers only `Back to the paths` or
  `Engrave a key-less template` — it never offers to engrave a partial wallet.

---

## What I did not establish

- Whether the census ✓ is withheld until the last page on a **first** visit
  (bears on M-2).
- Whether the preimage plate is genuinely cut first (bears on M-8). No engrave
  was run.
- The hardened KDF's real-hardware timing; in the emulator it returned in under
  half a second, so `about 10s` could not be checked.
- Whether `Type a seed` at a slot behaves differently from the payload
  candidates with respect to I-4.
