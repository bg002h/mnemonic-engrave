# S6a — R5 DISCLOSURE lens review

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md` (this repo)
**Code inspected:** `/scratch/code/shibboleth/seedhammer`, branch `main`
**Lens:** does anything the plan adds to the restore document disclose something it
should not, and does the T6a spec's "greps clean of any xprv/private material"
guarantee still hold. No other lens (verify-status design, build order, test
plan, prose/wording) reviewed.

## VERDICT: GREEN — 0 Critical, 0 Important, 1 Minor

## EVERY ADDED LINE, AND WHAT IT REVEALS

| line (source) | what it discloses | derived from | sensitive? |
| --- | --- | --- | --- |
| Verify status, one of 5 (§4.7a: `Plates VERIFIED...` / `...on a repeat check...` / `Plates NOT VERIFIED...` / `Plate verification DID NOT COMPLETE...` / `WARNING: a read-back check DISAGREED...`) | which of 5 coarse outcomes a verify attempt sequence reached | `verifyStatus` enum only — no diff, no field name, no attempt count | no — coarse outcome, no secret-derived content |
| Plate census header + per-card lines + `"If any of them is missing, this backup is incomplete."` (§4.2, reusing `buildPlateInventoryLines`, `gui/multisig_build_census.go:75-92`) | plate count; per-card static label + static summary (`"ms1 secret share (secret seed backup)"`, `"mk1 key (account key card)"`, `"md1 descriptor (wallet policy descriptor)"`) | `len(c.strings)`, `c.label`, `c.summary` — all static strings set at `gui/singlesig_engrave.go:20-45`; never the card's actual `.strings` payload | no — confirmed by reading `buildPlateCensusLines`/`buildPlateInventoryLines`/`bundlePlatePlan`: none touches `c.strings` content, only its length |
| Passphrase statement, existing function unchanged (`buildPassphraseInventoryLines`, `gui/multisig_build_census.go:121-173`), now wired to single-sig for the first time | whether a BIP-39 passphrase was used; on a multi-seed build, per-seed label + master fingerprint of which seeds need one | `seedPassphraseFact{Label, MasterFP, Uses}` — `Uses = passphrase != ""`; never the passphrase text itself | no — struct carries no passphrase/mnemonic text (`gui/multisig_build_census.go:219-230`); single-sig/supply use `oneSeedPassphraseFact` which omits the fingerprint entirely (one seed, nothing to tell apart) |
| Seed statement, 3 variants — new (§4.4, `buildSeedInventoryLines`): absence / one ms1 plate / several ms1 plates | whether this specific set physically carries a seed-bearing plate at all, and (multi-seed) how many | count of `cardMS1` entries in `cards`, and `bundleSetCarriesASecret(cards)` (`gui/bundle_flow.go:482-484`, itself just `!bundleShowMs1Reminder(cards)`, a boolean over card kinds) | no secret content — but see **M-1**: a genuine, small, new disclosure to a document-only reader (see below) |
| Seed-handling / walk-away ruling, capacity- and presence-keyed — modifies existing sentence (§4.3, new `seedCapacity` type) | operational instruction only ("stays in device memory until the build ends", "power off when done"); the "plates are the secret" clause now appears only when `bundleSetCarriesASecret(cards)` is true | `seedCapacity` (a compile-time constant per call site, not data) + the same boolean as above | no — pure operational text, no data leak; this fix actually *removes* a false claim (today's shipped sentence claims "the plates are the secret" unconditionally, even on watch-only builds — see "found sound" below) |
| Mode-label change (§4.1, `buildFullModeLabel`) — `"Full (seed + keys, NOT passphrase)"` vs `"Full (seed + keys)"` | whether a passphrase was chosen, on the **engrave-mode picker screen**, not the restore document | `passphrase != ""`, read in the room by the operator before pressing | no, and **out of primary scope** — this is a transient operator-facing screen, not the durable artifact a stranger reads years later; not part of §4.2/§4.3/§4.4/§4.7a/passphrase-lines the brief scoped this review to |
| Pre-engrave census screen (§4.6, F-202, reuses `buildPlateCensusLines`) | plate count + per-card summary, shown **before** cutting | same static fields as the restore-doc census | no, and also transient/out of primary scope (operator is physically present) |

## M-1 — Seed-presence disclosure's adversary side is weighed nowhere in the plan

**What is disclosed:** §4.4's new seed statement tells a reader holding **only**
the restore document (no plates) a definite yes/no on whether a spendable
seed-bearing plate exists somewhere for this wallet, and on a multi-master
build, how many. Before this plan the single-sig restore document said nothing
about full-vs-watch-only mode at all — measured, `singleSigRestoreLines`
(`gui/singlesig_restore.go:97-113`) renders exactly master fp / descriptor /
two addresses regardless of mode.

**To whom, and how they would get it:** anyone who obtains a copy of the
restore document without the plates — a photograph that leaks, a paper copy
stored (against the operator's instructions) apart from the steel, or found by
an heir's untrusted associate before the plates are located. They learn
definitively whether a seed plate is out there to look for, and (multi-seed)
how many.

**Why it matters (or why the trade-off favors disclosure):** The same
document, unchanged by this plan, already discloses the full descriptor and
the first receive/change addresses — enough for anyone to check the chain and
learn the wallet's balance and activity, which is already a far stronger
incentive to go hunting for the plates than a bare "yes, a seed plate exists."
The marginal disclosure is real but small against that baseline, and it directly
serves this cycle's own directive ("expressive on output") and closes F-195 —
the identical fact ("no plate in this set carries a seed") is already spoken
today on the transient abort-warning screen (§1.6, `gui/bundle_flow.go:555`),
so this is applying an already-accepted disclosure policy to the durable
document, not opening new territory.

On balance the disclosure is sound. The gap is procedural, not substantive:
the plan's own justification for §4.4 ("Three things this wording does on
purpose") argues only from the legitimate-recoverer's side — "is this
everything?" — and never states the adversary side, even though the plan
performs exactly this kind of incentive-weighing elsewhere (§4.7's argument
against a hard verify-fail gate is the same shape of reasoning, done well).
Recording the trade-off explicitly — including the observation that the
descriptor+addresses already dominate it — would let a future reader see this
was decided on purpose.

**Disposition:** record as a deliberate trade-off (one paragraph, citing the
existing descriptor/address disclosure as the dominating baseline and the
abort-warning precedent) wherever §4.4 lands; does not block implementation.

## WHAT I CHECKED AND FOUND SOUND

- **The passphrase disclosure trade-off (task item 2) is sound, and it is not
  a decision this plan makes.** `buildPassphraseInventoryLines` and
  `seedPassphraseFact` are pre-existing, S5-shipped code
  (`gui/multisig_build_census.go:96-230`); this plan only wires single-sig
  into a call that was already correct for both multisig paths (§1.2). I
  independently re-derived the trade-off: an adversary with the document but
  not the plates has no seed words to attempt derivation with regardless of
  whether the passphrase fact is stated, so the marginal disclosure to that
  adversary is ~nil; an adversary with both the plates and the document
  already learns "a passphrase is needed" in one extra derivation attempt
  (bare-mnemonic fp will not match the document's masterFP), so the fact
  saves them one step, not a capability. The legitimate-recoverer benefit is
  the entire point of F-198/F-132. Net: disclosure clearly dominates, and the
  codebase's own doc comment (lines 108-120) already reasons through the
  recovery-side half of this correctly.
- **No new function touches secret-derived content.** Traced every function
  named in the brief: `buildPlateInventoryLines`, `buildPassphraseInventoryLines`,
  `seedPassphraseFact` (all pre-existing, `gui/multisig_build_census.go`), and
  the plan's proposed `buildVerifyStatusLines` (input: a 5-valued enum) and
  `buildSeedInventoryLines` (input: card kind + count). None accepts a
  mnemonic, passphrase text, or entropy — only public keys, fingerprints,
  static labels, and enum values. `bundleCard.strings` (the actual ms1 secret
  payload, confirmed at `gui/singlesig_engrave.go:20-45`) is read only via
  `len(...)` by every census/inventory function that touches it
  (`gui/multisig_build_census.go:36-46,75-92`, `gui/bundle_flow.go:348-363`)
  — never its contents.
- **The spec's "greps clean of xprv" constraint (task item 4) holds and is
  unchanged.** `grep -c xprv design/SPEC_seedhammer_T6a_singlesig_flagship.md`
  → **3** today (lines 36, 43, 66), matching the plan's own "expect unchanged"
  gate (§4.9). None of the plan's new operator strings (quoted in full at
  §4.3, §4.4, §4.7a) contains "xprv" or any private-key material; verified by
  reading every quoted string in the plan against this check.
- **The plan's claim "every added line is public" (task item 5, §3.1.7/§4.9)
  is true**, independent of its own three-category shorthand ("a plate
  count, a passphrase fact, a verify outcome") not literally naming the seed
  statement or the seed-handling ruling as separate categories — that is a
  wording/taxonomy completeness gap in the plan's self-description, not a
  false claim about what is disclosed (out of scope: prose review).
- **Master fingerprint disclosure is not new territory.** It was already one
  of the spec's original four fields (`design/SPEC_seedhammer_T6a_singlesig_flagship.md:36`,
  "master fp"). Per-seed fingerprint listing on multi-seed builds is
  unchanged S5-shipped behavior on the BUILD path only; single-sig and the
  SUPPLY path are single-seed capacity and `oneSeedPassphraseFact` omits the
  fingerprint entirely by design (`gui/multisig_build_census.go:190-199`).
- **The verify-status line cannot leak comparison detail.** §4.7d explicitly
  collapses three non-comparison failure causes (foreign plates, undecodable
  read-back, a mistyped seed at verify time) into the same `DID NOT COMPLETE`
  status as an incomplete/refused/abandoned run — no differentiated text that
  could hint at which failure mode occurred, and no retry count is ever
  surfaced.
- **The seed-handling ruling fix reduces a pre-existing over-disclosure, it
  does not add one.** Today's shipped sentence (`gui/multisig_build_census.go:86-90`)
  unconditionally claims "the plates are the secret," even on watch-only
  builds where it is false. §4.3 conditions that clause on
  `bundleSetCarriesASecret(cards)`, which only ever *removes* the false claim
  on watch-only sets — a correctness fix in the same direction as this
  review's concern, not against it.
- **This review did not re-litigate** whether the document should always
  render, cycle scope, or the verify-status/build-order/test-plan designs —
  those are other lenses' territory, per the brief.

GREEN with a single Minor is a real and expected outcome here: this document
is public-by-design, the plan's added content is uniformly non-secret by
construction (enum values, static labels, counts, and pre-existing booleans),
and the one gap found is a missing explicit trade-off statement, not a leak.
