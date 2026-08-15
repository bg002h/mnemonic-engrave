# Fable rulings — the two problems S2 inherits from S1 (2026-08-15)

Dispatched to rule, not analyse: (1) what S2's test 1 becomes now that D-1 did
not reproduce; (2) the duplicate-key window, all five sub-questions. Grounded in
plan §0/S1/S2, SPEC §2.2/§4.1/§4.3/P0/P1, F-178, the S1 execution review, and
the code as it stands at seedhammer `ca2e14b`. Everything a tool could check was
run; the one finding below that changes the picture is machine-confirmed.

## The finding that binds both rulings

**F-178's "no dead end" evidence and the duplicate-key defect are THE SAME
WALK.** Machine-checked, not inferred:

- The S1 walk's selection loop takes the first `use=2` payload cards by default
  (`cmd/emu/walk_build_policy.js:262-269` — "USE THIS CARD is row 0 and the
  default"), i.e. cards **A@0 and A@1**. F-178's hand-drive continued from that
  session and took the self seed FROM PAYLOAD — **masterA**.
- I reproduced that exact assembly through the real code paths (temp test in
  `gui/`, run via `go test ./gui/ -run TestTempDupCollisionScratch`, then
  deleted; tree left clean):
  - `deriveAccountXpub(masterA, "", MainNet, multisigSharedOrigin())` yields the
    **byte-identical 65-byte cc‖pk** as card A@0 (`COLLISION CONFIRMED`).
  - A@0 vs A@1 **differ** on both chain code and pubkey (`TRACE B SAFE`).
  - `assembleBuildPolicy({wsh, N:3, K:2, SelfSlot:0, fp omit}, selfXpub, fp,
    [A@0, A@1])` returns **no error, 6 md1 chunks, stub `4c3c96f1`**.
- **Stub `4c3c96f1` is the stub on F-178's own Policy Review screen**
  (FOLLOWUPS.md:6098). So the session recorded as "the flow ran to the engrave
  screen with no dead end" was walking a "2-of-3" holding masterA's acct-0 key
  at BOTH @0 (self) and @1 (card A@0), plus masterA acct-1 at @2 — one master
  holds the whole wallet, and the duplicated key alone satisfies k=2. Every
  slot rendered `(no fp)`.

Consequence: S2 must not pin F-178's screens as a good-state regression guard —
they are screens of the defect — and the duplicate refusal is not merely
"S2-owned", it is the first thing S2's walk hits on default taps.

---

## RULING 1 — S2's test 1

**Test 1 as written is void — there is no reproduction to promote — and it is
replaced by three items, none of which is "look harder at the payload path":**

1. **The regression guard for "the flow continues" is S2's own completed-engrave
   gate, not a screen-list pin.** Pinning F-178's exact screens would enshrine
   a walk whose assembled policy was the duplicate (stub match above), and the
   intermediate screens are scheduled to change at S4/S5 by design — a screen
   pin is a default for spelling where the stake is "an operator can finish".
   What is worth pinning is the stake: **Trace A completes an engrave with the
   labelled Trace A cosigners (B@0, C@0 — `cmd/buildpayloadcards/main.go:56-57`)
   and the md1 matches the oracle by production** — which is S2's gate already.
   The walk taps SKIP on cards 1 and 2; the remaining-equals-needed
   short-circuit (`gui/multisig_build_payload.go:317-321`) then takes B@0 and
   C@0 without further taps.
2. **Test 5 is re-scoped into the standing D-1-class guard.** Without a
   reproduced defect, "calibrate the floor against the real defect" is
   unsatisfiable. Instead: a whole-walk raster floor — every screen from the
   template picker to the engrave-style picker must draw a **body**, with the
   floor calibrated both ways by measuring each real screen's ink against a
   title-only frame (F-151's defect shape is exactly "title draws, body does
   not", which is what a field "blank screen after configuration" would look
   like). This is the guard that catches a D-1 recurrence whatever its cause.
3. **The one emulator shape S2 drives is the typed-seed source (F-178 item 4).**
   It is one of the two shapes F-178 names cheapest, and it is the only
   §0.1b-primary data entry (payload AND keyboard) that has never been walked —
   independent of D-1, that walk is owed. The `typeWord` driver it needs (over
   `shTap`; `walk_js.go` exposes button-level control only) is not throwaway:
   S4's per-slot multi-seed entry needs it regardless. If a dead end appears,
   capture it as the failing test — the original arm. If not, extend F-178's
   record. **The other unexercised shapes are not S2's**: the engrave itself is
   S2's gate; `sh(wsh)` is S3's walk; n/fp/self-slot variants are covered by
   S4/S5's walks; hardware is S6.
4. **D-1 itself is reassigned to S6 (hardware), and S2 may not close it.** The
   field observation was on a physical SH2; the emulator's display, engraver
   and NFC are stand-ins, and F-178 already names hardware "the single most
   likely home". FOLLOWUPS F-178 gets the owning-phase move (S2 → S6 for the
   D-1 question itself; S2 keeps the record-or-reproduce duty for the emulator
   shapes above), and S6's checklist gets an explicit reproduce-or-record line.

**Scope consequence:** S2 does not shrink materially. D-4 (title), M-E
(foreign-origin refusal), the duplicate check, and the completed-engrave gate
each stand on justifications independent of D-1. One file-table row corrects:
"the D-1 fix … in `assembleBuildPolicy`" (plan line 892) is void — there is no
emulator D-1 to fix; the row's other occupants (origin refusal, duplicate
check) stay.

## RULING 2 — duplicate keys reach the assembled set

**2.1 Where.** One seam: **`assembleBuildPolicy`**
(`gui/multisig_build.go:538`), after the `all []md.MultisigCosigner` slice is
complete (:557-570) and before `md.EncodeMultisig` (:572-579) — a pairwise
comparison over `all`, returning a sentinel error type carrying both slot
indices (e.g. `errBuildDuplicateKey{SlotA, SlotB int}`). The flow at :174-178
branches on `errors.As`, maps slots to provenance it already holds
(`p.SelfSlot` → "your key"; `origins` from `buildCosignerOrigins` → "payload
card N") and shows the named refusal instead of the generic "Couldn't assemble
the wallet policy." Why exactly here and nowhere else:

- **Not at card selection**: the self key does not exist until step (4)
  `deriveAccountXpub` — the delivered hazard is self-vs-card, undetectable at
  the picker. A selection-time card-vs-card copy would be the second copy of a
  decision that `classifyCosignerSupply`'s own comment warns grows back.
- **Not at the review screen**: a duplicate must never reach review. The check
  at :174 runs before `buildReviewFlow` at :182, so it doesn't.
- `assembleBuildPolicy` is the SOLE md1 producer for the Build path
  (I-VERBATIM), so every present and future route passes through it. **This is
  §4.1's permanent final-slot-set check landed early, not an interim one** —
  when S4/S5 add `derived`/`both` slots those still land in `all`, and the
  comparison is source-independent by construction. S2's test keeps its
  scheduled name; the check itself never gets replaced, only fed more sources.

**2.2 Refuse, named, with the way forward in the text.** §0.1 clause 2 decides
refuse: quorum degradation invisible in every artifact when fingerprints are
omitted, which is the default — and the operator's own review screen would show
three `(no fp)` slots (measured, S1 execution review N5). The refusal is a
dismissible named modal (`showError`) after which the flow returns — consistent
with every other refusal in this flow. The "deselect and continue" surface is
NOT built at S2: S4's slot-review screen is that surface by design, and
building a bespoke retry loop two stages early is scaffolding S4 deletes.
Operator-facing text (slot numbers and card numbers substituted from the
sentinel + provenance):

> **Duplicate key**
> Slot @0 (your key, from your seed) and slot @1 (payload card 1) hold the
> SAME key. A policy that repeats a key can be spent by fewer different keys
> than its k-of-n says. Nothing was engraved. Build again and choose different
> cards, or use a different seed; if the payload has no other cards, rewrite
> it on the host with `me sysw pack`.

(For a card-vs-card duplicate both lines name payload cards. Every sentence is
load-bearing: which slots, why it is harm, that nothing was cut, and both
routes that exist on this hardware.)

**2.3 The comparison is the identical 65-byte chain code ‖ compressed pubkey
over the assembled slots** — SPEC §4.1's rule verbatim, already normative;
S2 implements it, it does not choose it. Machine-confirmed both ways: the rule
FIRES on the delivered collision (self at `multisigSharedOrigin()` == A@0,
byte-equal) and PASSES Trace B's shape (A@0 vs A@1 differ on both components —
same master, different accounts, different keys). Rejected alternatives:
**master fingerprint** identifies a master, not a key — it would refuse Trace
B's legitimate `A·acct0`+`A·acct1` and is absent by default anyway;
**base58 xpub** carries parent-fp/depth metadata that differs across sources
for the same key (§4.3 reason 1) and the encoder drops it (`md/expand.go:62`).
Identical xpubs derive identical child keys at every address index, and
differing chain codes derive differing children even under an equal parent
pubkey, so cc‖pk is exact: no missed real duplicate, no refused legitimate
setup.

**2.4 The delivered payload stays exactly as it is.** Changing it would break
three committed anchors: the digest pin (`cmd/emu/sysw_cards_payload.go`,
asserted by `walk_trace_a.js` `CARDS_DIGEST`), S0's gate-record
reproducibility (`go run ./cmd/buildpayloadcards`), and S4's honest-`both`
fixture — masterA's mnemonic and card A@0 MUST coexist on the payload
(`main.go:54` says so in the card's own label), which is precisely the pair
that creates the collision. The collision is not an accident to remove; after
S2's check it is the refusal's standing walk fixture: **S2's walk gains a
refusal leg for free** (default taps + payload seed → the Duplicate key
screen) alongside the clean Trace A leg (SKIP, SKIP → B@0+C@0 → engrave).
A test payload that can drive both arms of a funds-safety refusal without a
new fixture is worth more than one that cannot.

**2.5 Should it have blocked S1? No — the review's scheduling call stands, but
its stated reason was already false, and that buys two conditions.** The call
("S2-owned, does not block S1", execution review N5) is per-phase burndown
working as designed: the check was scheduled to S2 in the frozen plan before S1
began, nothing ships between stages, and the engrave sits behind the mandatory
EXPERIMENTAL hold. But N5's justification — the check lands "before any
engrave completes" — was contradicted by F-178 in the same round: the flow
demonstrably runs to the engrave-style picker, so the window is live in the
tree now, not from S2. The remedy is sequencing, not re-opening a closed
stage: **(i) the duplicate check and its test are S2's FIRST landing, before
any other S2 work — in particular before any walk that completes an engrave;
(ii) no hardware engrave of the Build path until the check is in.** If S2 were
ever resequenced or deferred, the check becomes immediately gating wherever it
lands.

---

## PLAN-READY TEXT

### For S2's test list, replacing test 1 and test 5

> 1. **D-1 did not reproduce on the payload path (F-178), and its screens may
>    not be pinned: the session that proved "no dead end" was assembling the
>    duplicate policy (its Policy Review stub `4c3c96f1` is byte-reproducible
>    from self=masterA + cards A@0,A@1 at n=3,k=2,@0,fp-omit — machine-checked
>    2026-08-15).** What replaces the promotion:
>    (a) the completed-engrave gate below IS the regression guard for "the flow
>    continues past the gather" — Trace A's walk selects the labelled Trace A
>    cosigners by tapping SKIP on payload cards 1 and 2, letting the
>    remaining-equals-needed short-circuit take B@0 and C@0;
>    (b) `TestBuildWalkTypedSeed` — the typed-seed source (F-178 item 4, the
>    §0.1b-primary entry no walk has driven): the same walk with the self seed
>    entered on the keyboard, via a `typeWord` driver over `shTap` that S4's
>    multi-seed entry will reuse. If a dead end appears, capture it as the
>    failing test; if not, extend F-178's record;
>    (c) D-1 itself moves to S6: field-observed on hardware, unfalsifiable in
>    the emulator. S6 reproduces it or records its non-reproduction on the
>    machine. S2 may not close D-1. (F-178's owning-phase entry updates
>    accordingly.)
> 5. **A raster floor over the whole Trace A walk** — every screen from the
>    template picker to the engrave-style picker draws a body. Calibrated both
>    ways against a measurable pair: each real screen's ink vs a title-only
>    frame (F-151's shape — title draws, body does not — is what a field
>    "blank screen after configuration" looks like). This is the standing
>    D-1-class guard; it does not wait for a reproduction to exist.

### For S2's test 4, replacing the "interim" framing

> 4. **`TestS2RefusesDuplicateKeysBeforeS4` — and this is §4.1's PERMANENT
>    final-slot-set check landed early, not an interim one.** The check lives
>    in `assembleBuildPolicy` (the SOLE md1 producer — every present and
>    future route passes through it), after the `all` slice is complete and
>    before `md.EncodeMultisig`: refuse iff any two final slots carry an
>    identical 65-byte chain code ‖ pubkey, via a sentinel error naming both
>    slots. The flow maps the slots to provenance it already holds
>    (`p.SelfSlot`, `buildCosignerOrigins`) and shows a named modal:
>    *"Duplicate key — Slot @A (your key, from your seed) and slot @B (payload
>    card N) hold the SAME key. A policy that repeats a key can be spent by
>    fewer different keys than its k-of-n says. Nothing was engraved. Build
>    again and choose different cards, or use a different seed; if the payload
>    has no other cards, rewrite it on the host with `me sysw pack`."*
>    NOT at card selection (the self key does not exist until step 4, and a
>    second copy of the count/identity decision is how `n-1` grew back last
>    time); NOT at review (a duplicate never reaches review). S4 does not
>    replace this check — it feeds it more sources.
>    **Comparison basis is ruled:** cc‖pk equality fires on the delivered
>    collision (self at `multisigSharedOrigin()` == card A@0, byte-equal,
>    machine-checked) and passes Trace B (A@0 vs A@1 differ on both
>    components). Master fingerprint would refuse the legitimate multi-account
>    wallet; base58 xpub compares metadata the encoder drops.
>    **Sequencing, RULED:** this test and its check are S2's FIRST landing —
>    before any S2 work that completes an engrave — and no hardware engrave of
>    the Build path happens until it is in. F-178 proved the window is live
>    NOW (the flow reaches the engrave screens), so the plan's "from S2, which
>    makes engraves complete, until S4" understated it; the S1 closure stands,
>    the sequencing is the price.
>    **The delivered payload does not change.** masterA's mnemonic + A@0 must
>    coexist for S4's honest-`both` fixture, A@0+A@1 are Trace B's
>    multi-account shape, and the digest is pinned three ways (blob pin, walk
>    `CARDS_DIGEST`, S0's gate record). The collision it creates is the
>    refusal's standing walk fixture: S2's walk drives BOTH arms — default
>    taps + payload seed → the Duplicate key screen; SKIP, SKIP → B@0+C@0 →
>    completed engrave.

### File-table correction (S2)

> `gui/multisig_build.go` — ~~the D-1 fix,~~ the interim foreign-origin
> refusal and the §4.1 duplicate-key check (permanent, see test 4), in
> `assembleBuildPolicy`. There is no emulator D-1 to fix (F-178); D-1's
> remaining home is S6's hardware gate.

---

## WHAT I COULD NOT CHECK

- **Whether the engrave actually completes past the style picker in the
  emulator.** F-178's drive stopped there; I did not run a browser walk. S2's
  gate owns it. (The duplicate window's liveness does not depend on it — the
  policy bytes exist at `assembleBuildPolicy`, machine-confirmed.)
- **Whether D-1 reproduces on the typed-seed shape or on hardware** — open by
  definition; owned by S2(b) and S6 respectively under Ruling 1.
- **`bundleGatherer.offer`'s dedup of byte-identical duplicate cards** — read
  (`gui/bundle.go:112,150-219`), not exercised. Irrelevant to the seam choice:
  the assembly check catches every duplicate that survives any upstream path.
- **The modal-return UX** (`showError` → flow returns to the menu) — read
  (`gui/slip39_polish.go:36-38`), not walked.
- The temp machine-check test was created in `gui/`, run once (`go test ./gui/
  -run TestTempDupCollisionScratch -count=1` — PASS, three confirmations
  logged), and deleted; `git status` clean at `ca2e14b` after.
