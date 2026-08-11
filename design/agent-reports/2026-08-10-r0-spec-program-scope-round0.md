# R0 round 0 — SPEC amendment: §10.2.4's guarantee scoped to the Sealed Payload PROGRAM

**Artifact:** `design/SPEC_encrypted_payload_delivery.md` at `53b1ab5` (two edits: §2.2 item 12,
§10.2.4 SCOPE note, plus a §2.3 paragraph).
**Implementation:** `/scratch/code/shibboleth/seedhammer-b2b` @ `75233b8` (branch `b2b`).
**Scope:** the two edits and their consistency with the rest of the spec. Not a fresh audit.
**Reviewer:** independent architect, author ≠ reviewer.

## VERDICT — NOT GREEN: 0 Critical, 4 Important, 1 Minor, 1 Nit

The ruling is implemented in the right direction and the central factual claim about the code
holds (see "what I verified sound", below). What does not hold is the *precision* of the two
edits. Three of the four Importants are the same shape: the amendment describes a boundary in
three different places — item 12's enumeration, item 12's carve-out list, and §10.2.4's SCOPE
note — and the three do not agree with each other or with the code. The fourth is that the new
item is not physically located in the section every citation to it names.

None of them changes what the machine does today. All four change what the *next* person does.
That is exactly the failure mode item 12 was written to prevent (F-83 → F-85 + F-108), so it is
worth one more fold rather than a shrug.

---

## IMPORTANT 1 — §10.2.4's SCOPE note describes ONE bracket; the timer it introduces has TWO, and the second is row 4

**Exact text at fault** (§10.2.4, the new note, lines 1368-1372):

> **SCOPE — read this before the table.** "Resident" means resident **within the Sealed Payload
> program's session**, not anywhere on the machine. This timer is implemented as the lifetime of
> that session's guard (`gui/wipe_guard.go`, whose bracket is `unlockSecretSession`'s own first
> and last act) […]

**The fact.** There are exactly **two** production `wipeGuard` install sites, measured:

```
$ grep -rn "ctx.wipe = " --include="*.go" . | grep -v _test.go
gui/unlock_kdf.go:136:      ctx.wipe = &wipeGuard{subject: wipeWarningSubjectPassphrase}
gui/unlock_kdf.go:138:              ctx.wipe = prev
gui/unlock_session.go:89:   ctx.wipe = g
gui/unlock_session.go:91:           ctx.wipe = prev
```

`gui/unlock_kdf.go:135-144` is `unlockPassphraseFlow`'s own bracket. It is **row 4** of the very
table this note is placed in front of, and it is not merely a different bracket — it is one that
by construction can never be inside `unlockSecretSession`'s, because the code comment at
`unlock_kdf.go:131-134` says so: *"this function always returns before `unlockSecretSession`
runs"*. §10.2.4 itself states this eight paragraphs below the new note: *"Row 5 is implemented as
the **passphrase bracket closing before the derivation is called**, not as a second flag on the
guard."*

So the SCOPE note — introduced as the binding definition of "resident" for the whole table,
labelled *"read this before the table"* — defines a scope that excludes rows 4 and 5 of that
table. The section now contradicts itself.

**Concrete scenario.** F-112's burndown (owning phase: post-B2b, before the release tag) requires
someone to reconcile "which brackets does the spec actually require" against six unbracketed
legacy flows. That person reads the SCOPE note, finds it names exactly one required bracket
(`unlockSecretSession`'s), and treats `gui/unlock_kdf.go:136`'s guard as unaccounted-for machinery
— either removing it, or, far more likely, not flagging its removal when a later refactor of
`unlockSealedFlow`'s retry loop drops it. The operator then starts a Sealed Payload unlock, is on
the twelve-word passphrase keyboard at word 7, is interrupted, and walks away. Seven-plus words of
the payload passphrase sit in the `[]Word` from `emptyBIP39Mnemonic(12)` and in `ctx.B`'s glyph
args, on a powered machine, with the sealed blob they open in flash beside them, with no timer and
no scrub, indefinitely. Row 4 exists to prevent precisely that state, and the operator — told by
§2.3 that Sealed Payload is *the program that wipes* — expects the wipe.

**Smallest fix.** Replace the parenthetical:

> This timer is implemented as the lifetime of that program's wipe guards (`gui/wipe_guard.go`) —
> installed by `unlockSecretSession` for rows 1–3 and by `unlockPassphraseFlow` for row 4, each as
> that function's own first and last act — and it makes **no claim** about […]

---

## IMPORTANT 2 — item 12's carve-out 2 promises a discipline on the plate path that §10.2.4 row 3 forbids

**Exact text at fault** (§2.2 item 12, "What this does NOT license", 2nd bullet):

> - It does not cover the Sealed Payload program's **own** inspect and plate paths. If the
>   operator is in that program, the discipline applies to **every screen it reaches.**

**The fact.** It does not, and must not. `unlockSecretSession` restores `ctx.wipe = prev` on exit
(`gui/unlock_session.go:91`), and `prev` is `nil` — pinned by
`gui/wipe_guard_test.go:129-130` (*"ctx.wipe is non-nil after unlockSecretSession returned — the
bracket did not uninstall"*). So on the plate list (`unlockPlateListFlow`) and every screen it
reaches, §10.2.4's timer is **not running**. That is correct and deliberate: table row 3 says

> | **no** secret record resident **and no passphrase in flight** | **none** | Public data only.
> Nothing to protect. |

and §10.2.4's opening paragraph gives the reason a timer there is actively harmful. The carve-out
over-promises — the safe direction, but still a contradiction, and it is the sentence that will
adjudicate F-76.

**Concrete scenario.** F-76 ("inspecting a payload-sourced card", owning phase: *after B2b* — i.e.
next) lands. Its implementer needs to know whether the new Inspect entry on the Sealed Payload
plate list must carry §10.2.4's discipline. They read carve-out 2, conclude "every screen it
reaches", and install a `wipeGuard` for the plate list and the primed gatherer. The timer now arms
with **no secret resident**: mid-bundle, at plate 4 of 15 of a 2-of-3, the operator spends four
minutes swapping and squaring steel — a legitimate, ordinary pause — and returns to a wiped
session, needing twelve words and a ~31 s KDF to get back to a plate list holding nothing but
xpubs. §10.2.4 names this outcome in its own first paragraph: *"a timer there would guard an xpub
while still firing during the legitimate multi-minute pauses of a plate swap — the fastest way to
teach an operator to disable a control."* The amendment would have caused the thing the section it
amends exists to prevent.

**Smallest fix.** Make the carve-out follow the secret rather than the screen — which also answers
F-76 unambiguously, and is the reviewer's priority-2 question:

> - It does not cover the Sealed Payload program's **own** inspect and plate paths. Inside that
>   program the discipline follows the **secret**, not the screen: any screen it reaches while a
>   secret record or an in-flight passphrase is resident is inside the bracket, however that screen
>   is shared with a legacy flow. Once §10.2.2 has wiped the last secret, §10.2.4 row 3 applies and
>   there is nothing left to time — that is not this item's licence, it is the table's own row.

---

## IMPORTANT 3 — item 12's enumeration lists "the backup/inspect screens" among the OTHER programs; the hardened program's only seed screen IS one

**Exact text at fault** (§2.2 item 12, opening paragraph):

> Every other program on this machine — NFC scan, manual word entry, BIP-85 derivation, account
> xpub, SeedXOR, SLIP-39, free text, **the backup/inspect screens** — may leave seed material
> resident in SRAM and in the frame buffer with **no wipe at all**, indefinitely, and that is
> **accepted rather than a defect**.

**The fact.** Every other entry in that list names a *program* (a menu destination). "The
backup/inspect screens" names a *screen class*, and those screens are **shared with Sealed
Payload**, not alternatives to it. Measured — `SeedScreen` has exactly two non-test construction
sites:

```
$ grep -rn "SeedScreen{\|new(SeedScreen)" --include="*.go" . | grep -v _test.go
gui/gui.go:2195:           ss := new(SeedScreen)        # backupWalletFlow (legacy)
gui/unlock_session.go:291: ss := &SeedScreen{NoEdit: true}   # unlockEngraveMnemonic (Sealed Payload)
```

The second is the screen on which the **decrypted payload seed** is confirmed, and per F-112 it is
`SeedScreen`'s *only bracketed* construction site. So the sentence that accepts "the
backup/inspect screens" leaking seed material with no wipe, indefinitely, names as accepted the
one screen the amendment's own carve-outs 1 and 4 say is fully protected. The item states both
positions about the same object, and the enumeration is the one a reader hits first — before the
carve-out list two hundred words later.

**Concrete scenario.** B2c ("secret-residency cleanup: F-88, F-90 items 1 and 3, F-94") opens.
F-88's three copies all live on this path: `bip39.MnemonicSeed`'s `sentence []byte` — *the
plaintext mnemonic* — plus `seedqr.QR(m)`'s backing bytes and `qr.Code.Bitmap`, all produced by
`unlockEngraveMnemonic` → `engraveSeed` while the payload seed is on `SeedScreen`. The B2c owner
reads item 12's enumeration, sees the backup screens' residency accepted by operator ruling, and
closes F-88 as *accepted, not a defect*. The decrypted payload seed's plaintext copy then remains
resident with no wipe — inside the hardened program, which is what carve-out 1 forbids in the same
breath. A more aggressive reading reaches carve-out 4 as well: `unlockSecretSession`'s
`ctx.B.Scrub()` (F-107, `gui/unlock_session.go:104`) is a scrub of a *rendered backup screen*, and
its own comment records what its absence costs — *"the twelve words come back verbatim and in
order from the backing array on a NORMAL exit — read your words, press back."*

This is one carve-out away from Critical; it is held at Important only because carve-outs 1 and 4
both fence the bad reading. But the F-83 precedent named three lines above the carve-out list is
proof that a fence two hundred words downstream of the licence is not enough.

**Smallest fix.** Delete five words — strike `, the backup/inspect screens` from the enumeration.
It is the only entry that is not a program, §2.3's list and the §10.2.4 SCOPE note's list both
already omit it, and carve-out 2 (as repaired by Important 2) covers the shared-screen case
properly.

---

## IMPORTANT 4 — item 12 is not in §2.2; it is in §2.2a, and §2.2 is the operator-doc extraction source

**The fact.** Measured section boundaries:

| line | content |
| --- | --- |
| 46 | `### 2.2 What this does NOT defend against` |
| 48 | *"This list is normative and belongs in operator documentation, not only here."* |
| 50-132 | items 1 … 11 |
| **134** | `### 2.2a What admitting ms1 changed (operator sign-off, 2026-08-07)` |
| **162** | **`12. **Secrets handled by any program OTHER than Sealed Payload.**`** |
| 209 | `### 2.3 The operating rule that follows` |

Item 12 sits **inside §2.2a**, after §2.2a's closing paragraph. Both citations to it —
`§2.2 item 12` at line 219 (§2.3) and line 1374 (the SCOPE note), plus the commit subject — name a
section it is not in. Item 12's own first bullet (*"As this section says above…"*) resolves only
because of the misplacement, so the two halves are incoherent whichever way you read them: if it
belongs in §2.2a it should not be numbered into §2.2's sequence nor cited as `§2.2 item 12`; if it
belongs in §2.2 it must sit before line 134 and its internal back-reference must name §2.2a
explicitly.

**Concrete scenario.** §2.3's new paragraph says the two-classes distinction *"is **operator-facing**
and belongs in the documentation, not only here."* The person writing that documentation follows
§2.2's own instruction at line 48 — *this list is normative and belongs in operator documentation*
— and extracts §2.2's normative list. They get items 1 through 11. Item 12 is not in §2.2 to
extract, and §2.2a is titled as a 2026-08-07 `ms1` sign-off, which is not where anyone looks for
"what this does not defend against". The shipped manual never tells the operator that the legacy
programs do not wipe. The operator, having watched Sealed Payload wipe itself after three minutes,
scans a seed by NFC the following week, walks away from the machine mid-plate, and leaves the seed
resident in SRAM and on a blanked screen believing the timer they have seen work is behind it —
the precise misapprehension §2.3's new paragraph was written to prevent, defeated by the location
of the item it cites.

**Smallest fix.** Move the item-12 block (lines 162-207) to immediately after item 11 (i.e. before
the `### 2.2a` heading at line 134), and change its first bullet's *"As this section says above"*
to *"As §2.2a says"*. No other text change; every existing citation then resolves.

---

## MINOR — three divergent enumerations of "the legacy programs"

| location | entries |
| --- | --- |
| §2.2 item 12 | NFC scan, manual word entry, BIP-85 derivation, account xpub, SeedXOR, SLIP-39, free text, **the backup/inspect screens** (8) |
| §10.2.4 SCOPE note | NFC scan, manual word entry, BIP-85, account xpub, SeedXOR, SLIP-39, free text (7) |
| §2.3 | NFC scan, manual entry, BIP-85, xpub, SeedXOR, SLIP-39, free text (7) |

The single divergent entry is the one at fault in Important 3, so the fix there removes today's
drift. The structural risk remains: three hand-maintained copies of a normative list, one of which
(§2.2's) is the documented operator-doc source. Whoever adds the eighth legacy program updates one
or two of the three. **Fix:** keep the enumeration once, in item 12, and have the SCOPE note and
§2.3 say "the legacy programs listed in §2.2 item 12" rather than restating them.

## NIT — `gui/wipe_guard.go` is cited as the thing that installs the bracket; it does not

Item 12: *"it is what `gui/wipe_guard.go` already does, since the guard's lifetime *is*
`unlockSecretSession`'s own first and last act"*; SCOPE note: *"(`gui/wipe_guard.go`, whose bracket
is …)"*. `gui/wipe_guard.go` is 71 lines defining the `wipeGuard` type, `armed()` and
`warningSubject()`. It installs nothing and has no bracket. The brackets are
`gui/unlock_session.go:87-105` and `gui/unlock_kdf.go:135-144`. Costs nothing to cite correctly,
and a future reader chasing the bracket opens the wrong file.

---

## What I verified and found SOUND — do not re-derive

1. **The bracket edge on the secret session is genuinely tight.** There is no operator-visible
   frame between `p.Secret` being populated and the guard being installed. `unlockAttemptOnce`
   ends in `return o.UnlockWithKey(blob, p, key)` with no frame after it
   (`gui/unlock_kdf.go:374`); `unlockSealedFlow` returns `true` immediately on `err == nil`
   (`:418-419`); `unlockPayloadFlow` then executes `clear(blob); blob = nil; unlockSecretSession(…)`
   — straight-line, frame-free (`gui/unlock_flow.go:110-114`). The amendment's "first and last act"
   claim holds for rows 1-3.
2. **No secret escapes the session loop.** `seal.IsSecret` is exactly
   `ClassCodex32Secret || ClassMnemonic` (`seal/session.go:16-18`). Records in `p.Secret` that fail
   it are the encrypted section's md1/mk1 cards, which §6.3 makes public data wherever they
   travelled; the backstop is `defer p.Wipe()` at `gui/unlock_flow.go:85`.
3. **F-76's screens are not reachable from the Sealed Payload program today.** `mdmkFlow`
   (`gui/gui.go:2137`) is the sole caller of `mk1GatherFlow` / `md1GatherFlow`, reached only from
   the legacy scan path at `gui/gui.go:2084`; `unlockEngraveFlow` deliberately does not reuse it
   (`gui/unlock_platelist.go:192-201`). F-76 is a **future** seam, not a present gap — which is why
   Important 2's fix is worth landing *before* F-76 rather than as part of it.
4. **The frame buffer does not carry seed glyphs past the bracket.** Both brackets call
   `ctx.B.Scrub()` on exit (`gui/unlock_session.go:104`, `gui/unlock_kdf.go:143`), so the plate list
   does not inherit them.
5. **No contradiction with §2.1, §2.2 item 9, §2.2a or §10.2.2.** §2.1's claim is about persistent
   state (flash), item 12 is about SRAM and the frame buffer; §2.2 item 9 and §10.2.2 are both
   already scoped to the bundle session, which is what item 12 says. Row 1 of §10.2.4's table is
   unaffected. The clash is with row 3 (Important 2) and row 4 (Important 1).
6. **"five legacy flows" is right.** F-112 names six; one (`gui/unlock_session.go:276`) is inside
   the payload path. (Given as settled in the brief; confirmed against F-112's table, not
   re-derived.)

## Re-review scope if these are folded

All four Importants are wording or placement; none requires new logic. A re-review should ask only
*"did the fold fix each of the four, and did it introduce a new claim about the code that is not
true"* — items 1-6 above are settled and should not be re-derived.
