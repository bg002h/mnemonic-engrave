# RULING — F-216: keyless-template slot mapping for gathered mk1 cards

Operator-stand-in ruling, 2026-08-21. Facts taken as given from the dispatch
brief; no code audited.

## 1. RULING

Build it. A gathered mk1 card is seated by **declaration match under stub
membership**: the card's `policy_id_stub` must verify against the template's
policy identity, and the card then fills exactly the slot(s) whose declared
origin path equals the card's origin — with the declared master fingerprint,
when the template carries one, required to equal the card's fingerprint. Gather
order is never an input; the operator is never asked to assign a card to a
slot; no address is ever derived from a partial or contradicted gather. This is
not a heuristic: under three invariants the device already enforces (stub =
membership in *this* policy; one origin binds one key within a policy;
slot origins pairwise distinct at encode since F-217), the assignment is
fully determined or the gather is refused — there is no state in which the
device guesses and shows the guess as proof. F-216 stays open and ships D3's
first half under this rule; it is not closed as won't-do.

## 2. THE RULE

Layered, both layers mandatory:

1. **Membership (stub).** On scan, verify the card's `policy_id_stub` against
   the loaded keyless template's policy identity. Fail → the card is refused at
   the scanner and never considered for any slot. The stub cannot map a card to
   a slot — the brief is right about that — but it is what makes layer 2 exact:
   it eliminates the wrong-master card whose origin coincidentally matches
   (standard paths like `48'/0'/0'/2'` collide across masters constantly).
2. **Declaration match.** Seat the card at every slot whose declared origin
   equals the card's origin. Where the template declares a fingerprint for that
   slot, the card's fingerprint must equal it; a mismatch is a contradiction,
   not a non-match (see 3). Where the template elides the fingerprint, origin
   alone suffices — *because of layer 1 plus the one-origin-one-key invariant*:
   a stub-verified card at origin P can only be the one key this policy binds
   at P. Elision costs nothing.

Consequences the rule handles without special cases:

- **One master, several slots (different accounts):** two different cards (two
  xpubs, two origins), each matches its own slot. No "one card, one slot"
  assumption anywhere.
- **Same key at two use-sites (if encodable as two slots):** identical
  declarations → the one card seats both slots; the resulting descriptor is
  unique, so this is deterministic, not ambiguous. If F-217's encode refusal
  makes this shape unencodable, the case simply never arises — pin whichever
  way it falls with a test (A6).
- **Completion is counted in SLOTS, not cards.** The gather completes when
  every slot is seated, however many physical cards that took.

Rejected outright: **gather order** (the operator's hands as a mapping input is
precisely the silent-wrong-address channel F-216 exists to avoid) and
**operator assignment UI** (same channel with extra steps). The stub alone is
also rejected as a mapping rule — it is a membership gate and nothing more.

**One precondition (A0):** this ruling assumes the stub is *checkable at gather
time* from bytes present in the keyless md1 and the mk1 card. That is asserted
by the brief's framing but must be machine-verified first. If A0 fails, the
elided-fingerprint clause is void: the admissible set narrows to templates
declaring fingerprints for **every** slot (exact (fingerprint, origin) match,
no stub needed), and an elided-fingerprint template keeps today's behavior with
the summary screen stating why: "Template declares no fingerprints — addresses
cannot be proven. Skip to consent, or re-encode with fingerprints."

## 3. WHEN IT CANNOT DECIDE

It refuses. It never asks, never guesses, never shows a partial address —
a multisig address is underivable with a missing xpub anyway, so partial
display is not even a temptation to resist. Specifically:

- **Stub fails on scan** → scanner screen: "Card is not part of this policy",
  showing the card's fingerprint + origin. Card not seated; gathering
  continues; other seated cards are unaffected.
- **Stub verifies, no slot matches** (card's origin declared by no slot) →
  hard stop screen naming the card's origin: this contradicts the invariants
  and indicates an encoding/integrity defect, not operator error. Gather
  aborts, no addresses this session; the existing skip-to-consent path remains
  reachable and says, as it does today, that no address proof was shown.
- **Contradiction** (declared fingerprint ≠ card fingerprint at a matching
  origin; or two stub-verified cards with different xpubs claiming one slot —
  possible only via a corrupted or mis-stamped card) → hard stop naming both
  sides. No operator override: a card the device cannot reconcile is a card
  whose address proof would be meaningless.
- **Incomplete gather** → the gather screen itself: the slot list, each row
  showing declared origin + fingerprint (or "fp —") and seated/empty state, so
  the operator can see WHICH key is missing. No address screen until all slots
  seat. Skip-to-consent-without-proof remains available at all times, exactly
  as shipped.

## 4. ACCEPTANCE — each machine-checkable, all required before ship

- **A0 — stub checkability.** A test that verifies an mk1 card's
  `policy_id_stub` against its keyless template using only bytes present in the
  two artifacts. Run FIRST; if it cannot pass, apply the narrowing in §2 and
  re-scope A1–A6 to declared-fingerprint templates.
- **A1 — order invariance.** For each vector policy, gathering the cards in
  every permutation of scan order yields byte-identical final descriptors.
- **A2 — Rust cross-check.** Encode a keyed policy → derive addresses in Rust →
  strip to keyless + mint mk1 cards → gather on the emulator → device addresses
  byte-equal the Rust addresses. Vectors must cover: N distinct masters;
  one master filling two slots via two accounts; elided-fingerprint template;
  declared-fingerprint template.
- **A3 — mutation.** A test build with the mapping deliberately rotated by one
  slot must FAIL A2, with evidence the mutated line executed — proving the
  cross-check can actually detect a wrong mapping, not merely that it passes.
- **A4 — refusals.** Journeys asserting: (i) wrong-policy card → refused at
  scan, never seated, no address rendered; (ii) stub-valid card matching no
  slot (harness-constructed) → hard stop, no address; (iii) fingerprint
  contradiction (harness-constructed) → hard stop, no address. Each asserts the
  specific screen AND the absence of any address render in the session.
- **A5 — no partial.** With any strict subset of slots seated, the address
  screen is unreachable (journey assertion).
- **A6 — same-key-two-slots pinned.** If the shape is encodable: a vector
  proving one physical card completes an N-slot gather and A2 holds. If encode
  refuses it: a test asserting that exact refusal, so the gap has a pinned
  shape.

## 5. WHAT THIS COSTS AND BUYS

**Costs:** a small matching function (two equality checks under a stub
verification that already exists as a concept), one gather screen extension
(slot list with seated state), and the A0–A6 battery. No new card formats, no
new wire fields, no migration — the operator has ruled no engraved cards exist,
so there is nothing to be compatible with.

**Buys:** the only field-verifiable integrity check the keyless format can
ever have. A keyless plate is the privacy-preserving distribution form, and
without the gather gate its only path is consent-without-proof — an engraved
backup that cannot be rehearsed against reality until an actual recovery.
The address proof closes that: operator gathers plate + cards, compares the
derived address to their wallet, and knows the backup reconstructs the wallet.

**Why not won't-do:** F-216 was held back because the mapping rule "is not
obvious" and a wrong mapping shows a wrong address as proof. The rule above has
zero design freedom left — every clause is an invariant the device already
enforces, and every undecidable state is a refusal, not a guess. Once the rule
cannot guess, the original reason to withhold D3's first half is gone, and the
operator's "we can establish seeds and derive / replace keys for any journey"
makes the full A0–A6 battery cheap to stand up. Closing as won't-do would
record "the rule is not obvious" as the reason precisely when the rule has
become forced.
