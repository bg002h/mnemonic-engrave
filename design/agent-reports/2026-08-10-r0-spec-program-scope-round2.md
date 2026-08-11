# R0 round 2 — fold review: §2.2 item 12 / §2.3 (responds to round 1)

**Artifact reviewed:** fold commit `438368f` on `design/SPEC_encrypted_payload_delivery.md`.
**Round 1 report reviewed against:**
`design/agent-reports/2026-08-10-r0-spec-program-scope-round1.md`.
**Scope:** did the fold fix round 1's one Important, and did it introduce a new defect. Not a
fresh audit — rounds 0 and 1's settled facts and the operator ruling are taken as given.

## VERDICT — GREEN: 0 Critical, 0 Important, 1 Nit

Round 1's Important is fixed, correctly, in the one location it needed to land. The §2.3 rewrap
is clean: prose is byte-identical to before the rewrap, just re-flowed, no duplication, no
truncation. One Nit found on the new "each installed as its installing function's own first and
last act" wording — true for `unlockSecretSession`, imprecise in the strictest literal sense for
`unlockPassphraseFlow` (a no-op test hook check precedes the guard install), but with no
operational consequence and no concrete harm scenario, so it does not block.

---

## 1. Round 1's Important — FIXED, and consistent across all three locations

Quoting all three, current text:

**Item 12** (`design/SPEC_encrypted_payload_delivery.md:157-164`):

> **The boundary is the PROGRAM, not the data's provenance.** A legacy program that reads an
> encrypted payload does **not** inherit this discipline; conversely, anything inside the Sealed
> Payload session carries it regardless of how the bytes arrived. This is not a new constraint on
> the implementation — it is what the program's two `wipeGuard` brackets already do, each
> installed as its installing function's own first and last act: `unlockSecretSession` for the
> secret session, and `unlockPassphraseFlow` for the in-flight passphrase.

**Carve-out 2** (`design/SPEC_encrypted_payload_delivery.md:173-179`, "What this does NOT
license"):

> It does not cover the Sealed Payload program's **own** inspect and plate paths. Inside that
> program the discipline follows the **secret**, not the screen: any screen reached while a
> secret record or an in-flight passphrase is resident is inside the bracket, however that screen
> is shared with a legacy flow. Once §10.2.2 has wiped the last secret, §10.2.4 row 3 applies and
> there is nothing left to time — that is the table's own row, not a licence granted here.

**§10.2.4 SCOPE note** (`design/SPEC_encrypted_payload_delivery.md:1375-1386`):

> **SCOPE — read this before the table.** "Resident" means resident **within the Sealed Payload
> program's session**, not anywhere on the machine. This timer is implemented as the lifetime of
> that program's wipe guards (`gui/wipe_guard.go` defines the type) — installed by
> `unlockSecretSession` for rows 1–3 and by `unlockPassphraseFlow` for row 4, each as that
> function's own first and last act — and it makes **no claim** about seed material held by any
> other program — NFC scan, manual word entry, BIP-85, account xpub, SeedXOR, SLIP-39, free text.
> Those may hold plaintext indefinitely with no timer behind them; see **§2.2 item 12**, which
> accepts that explicitly and states what it does not license.

All three now agree on the same two-bracket model: `unlockSecretSession` brackets the secret
session (rows 1–3), `unlockPassphraseFlow` brackets the in-flight passphrase (row 4). No
contradiction remains. `grep -n "single-bracket\|one bracket\|guard's lifetime \*is\*"` returns
zero hits anywhere in the file — the old one-bracket phrasing is gone, not just superseded.

## 2. The two-bracket claim — TRUE for both functions, with one imprecise phrase (see Nit)

Checked against `/scratch/code/shibboleth/seedhammer-b2b` (branch `b2b`, HEAD `75233b8`):

- `gui/unlock_session.go:81-89` — `unlockSecretSession`'s first executable statement (after its
  doc comment) is `prev := ctx.wipe` (line 87), immediately followed by `g := &wipeGuard{}` /
  `ctx.wipe = g` (lines 88-89). Nothing precedes the guard install. The install genuinely is the
  function's first act.
- `gui/unlock_kdf.go:109-136` — `unlockPassphraseFlow`'s first *executable* statement is
  `if unlockPassphraseHook != nil { unlockPassphraseHook() }` (lines 110-112), which runs *before*
  `prev := ctx.wipe` / `ctx.wipe = &wipeGuard{...}` (lines 135-136). `unlockPassphraseHook` is a
  package-level `var` that is nil in production and set only in `gui/unlock_kdf_test.go:571,610`
  (with `t.Cleanup` resetting it to nil) — so in production this is a no-op nil-check, but it is
  a statement that runs ahead of the guard install, which is literally what the phrase "own first
  ... act" says does not happen. See the Nit below.
- Both functions' defers restore `ctx.wipe = prev` as the first line of the deferred closure,
  before the subsequent `ctx.B.Scrub()` call in each — symmetric, and the "last act" half of the
  claim holds structurally for both (the defer always fires at every return path).
- `ctx.wipe =` production sites: confirmed at exactly `gui/unlock_session.go:89` and
  `gui/unlock_kdf.go:136`, matching the brief's settled fact.

## 3. §2.3 rewrap — clean, no duplication, no truncation

Diffed the paragraph's flattened text between parent (`438368f^`) and the fold, ignoring line
breaks: byte-identical prose, just re-wrapped. Current line lengths in §2.3
(lines 215-233) are all ≤80 characters, consistent with the file's established wrap width — no
172-character line remains. `grep -n "SLIP-39, free text"` finds the enumeration at lines 138,
228, and 1381, each reading correctly in context, no repeated clause.

## 4. No remaining internal contradiction

Checked all three locations (item 12, §2.3, §10.2.4 SCOPE note) together: consistent two-bracket
model, consistent enumeration of the seven legacy programs (word-for-word identical at all three
sites, per round 1's Minor), no stray single-bracket sentence anywhere.

---

## NIT — "own first and last act" is literally imprecise for `unlockPassphraseFlow`

**Exact text at fault** (item 12, line 162; same claim also at SCOPE note line 1379, pre-existing
and not touched by this fold): "each installed as its installing function's own first and last
act."

**The fact.** For `unlockPassphraseFlow`, the guard install is not the literal first statement
executed — `if unlockPassphraseHook != nil { unlockPassphraseHook() }` runs first (see §2 above).
For `unlockSecretSession` the claim is exactly true.

**Why this does not block.** `unlockPassphraseHook` is always nil in production (only ever
assigned inside `_test.go` files, which are not compiled into production builds), so the
preceding statement is a genuine no-op there. No sensitive material exists yet at that point
either: the passphrase mnemonic buffer (`m := emptyBIP39Mnemonic(12)`, `gui/unlock_kdf.go:156`)
isn't allocated until inside the loop, well after the guard is armed at line 136. So the
operationally load-bearing guarantee — the guard covers the entire window in which anything
sensitive exists — holds for both functions. The imprecision is in the literal parsing of "first
... act" as "first statement of any kind," not in the security property the sentence is used to
argue for. This exact phrasing (for `unlockPassphraseFlow`) already existed in the SCOPE note
before this fold and was reviewed favorably in round 1 as part of I1 without this being raised;
this fold's only change is to duplicate the same (pre-existing, already-accepted) claim into item
12. No concrete scenario of harm — hence Nit, not Minor.

**Smallest fix (optional, not blocking):** replace "own first and last act" with "own first and
last act with the guard" or similar, or move the `unlockPassphraseHook` nil-check to after the
guard install so the literal claim becomes true without changing behavior (the hook only
observes; it doesn't need to run before the guard is armed).

---

## Independently re-verified for this round

- `./scripts/plan-cite-gate.sh design/SPEC_encrypted_payload_delivery.md`: 2 unresolvable
  citations, matching the commit message and the brief's already-settled note (pre-existing
  F-115 basename-resolution false positives, not this file's citations).
- `grep -n "first and last act"` : exactly two occurrences (item 12 line 162, SCOPE note line
  1379), both now describing the same two-bracket model.
- `grep -n "single-bracket\|one bracket\|guard's lifetime \*is\*"`: zero hits — no orphaned
  single-bracket phrasing survives anywhere in the file.
- Guard-install/defer code read directly from `seedhammer-b2b` branch `b2b` at
  `gui/unlock_session.go:81-105` and `gui/unlock_kdf.go:109-144`.

## Loop status

This closes the loop: 0 Critical, 0 Important. The one Nit is optional and does not warrant a
round 3.
