# R0 round 0 — SPEC_wallet_policy_composer.md — LENS: THE OPERATOR'S JOURNEY, WALKED

**Artifact:** `design/SPEC_wallet_policy_composer.md` (DRAFT 2026-09-01, 497 lines, R0 round 0).
**Lens:** the C8 workflow walked as the operator, three journeys, three questions per step.
**Reviewer:** independent agent, read-only. No repo file modified except this report.
**Heads read:** mnemonic-engrave `b452a79`; fork `bg002h/seedhammer` working tree at
`/scratch/code/shibboleth/seedhammer`.

**Counts: 6 Critical / 10 Important / 8 Minor / 2 Nit.**

**Method note.** Every claim below that cites a `file:line` was read this round.
Two claims are explicitly NOT measured and are marked as plan-time machine checks:
there is no Go toolchain on this shell (`go` not found on `PATH`, nix profile empty),
so I could not render a screen. I did not re-run the structure, glyph or citation
gates (stated as already run).

---

## J1 — the two-path taproot wallet (brainstorm §3.4)

**The wallet.** `tr(NUMS,{multi_a(2,@0,@1,@2), and_v(v:pk(@3),older(26280))})` — 2-of-3
now; one of those three people alone after 26280 blocks, from a *second* hardened
account. Four slots, one wrapper, two paths.

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | 4 `key:` records packed by `me sysw pack`; SH2 flashed; payload LOADED at boot | boot offer LOAD/SKIP (`gui/sysw_load.go:43-53`) | tap SKIP, then Build | **GAP** — spec is silent; the journey silently becomes J3 | §7a / **I-3** |
| 2 | main menu | Wallet Policy → ChoiceScreen: Scan cards / From payload / Build a new policy | — | DEFAULT (F-437 naming) | §7a |
| 3 | "Build" | wrapper picker → preset or blank → path list | pick a preset then edit | DEFAULT | §7b, §4d |
| 4 | path list, empty | add Path 1: keys n=3, k=2 | — | DEFAULT | §7b |
| 5 | Path 1 done | add Path 2: keys n=1, k=1; lock | **operator does not know Path 2 needs a SECOND key from a person already in Path 1** (C5) | **GAP** — nothing in §7b/§8 says so; discovered only at the seating refusal | §7b / **I-9** |
| 6 | lock entry | kind (relative) → unit (blocks) → digit pad → echo "26280 blocks (about 182 days)" | type a *date* instead | DEFAULT, but the digit pad cannot type `YYYY-MM-DD` | §6b / **I-6** |
| 7 | shape complete | stub screen: Template-ID 32 hex, mk1 stub 8 hex, `mk encode … --policy-id-stub` | write the 8 hex down, then go Back and add a path | **GAP** — the id changes and nothing says so, and §7d itself routes them Back | §7c / **I-8** |
| 8 | — | compare the shown Template-ID against `md inspect` of the same policy on the host | ids and addresses differ (fixed conjunct order) | **GAP** — §4d says this of presets only | §7c/§7e / **I-10** |
| 9 | 4 sources, 4 slots | "Slot @0, Path 1 key 1 of 3: choose a key", pick list of remaining | @0 and @3 share fingerprint `73c5da0a`; labels show fp AND origin | DEFAULT (C29 informational) | §7d |
| 10 | 3 seated | last slot: exactly one source remains | device still shows a one-row pick list | DEFAULT, harmless | recorded |
| 11 | mapping review | slot → fingerprint + origin; Back keeps assignments | Back, then EDIT the shape → slot indices renumber | **GAP** — assignments carried across a renumber = silent misassignment | §7b/§7d / **C-5** |
| 12 | consent | "the structural summary, the id NAMED by kind, receive+change" | — | **GAP** — measured: this shape is `Renderable=false`, so the shipped surface prints "Complex policy - cannot display safely." | §7e / **C-2** |
| 13 | engrave form | concrete policy / template + keys | choose template + keys; all 4 slots came from `key:` records, not mk1 cards | **GAP** — §7d/§9.5 re-mint *cards*; nothing says a `key:` record becomes a card | §7f / **I-1** |
| 14 | — | re-minted cards carry "the new policy's stub" | engrave the *keyless template* alongside them | **GAP** — layer 1 of `seatKeyCards` needs the TEMPLATE stub; the restore refuses | §7c vs §7d / **C-1** |
| 15 | census, then cut | plate census, read-back integrity by form | — | DEFAULT | §7f |

---

## J2 — "our reasonably complex wallet" (`design/fixtures/reasonably-complex-wallet/`)

**Premise correction first.** The brief describes J2 as "four tiers incl. a keyless
hashlock tier". That is **stale**: the fixture README records the operator ruling of
2026-08-22 — *"keyless path is not reasonable"* — which keyed tier 4 as
`after(1383520) AND sha256(H3) AND pk(@6)` and added a seventh seed. RCW has **no**
keyless tier today, and a keyless tier under `tr` would be refused twice over (spec
§4e, and `rust-miniscript`'s `requires_sig`). I walked the current, keyed shape.

**The wallet.** `tr.policy`: 4 leaves, thresholds 3/2/1/1, 7 slots, two `sha256`
hashlocks + a third, `older(32768)` relative and `after(1173520)` / `after(1383520)`
absolute, no key path (NUMS). All seven keys at ONE shared origin
`m/270028'/0'/0'/0'`.

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | 7 `key:` records + 3 `hash:` records + `now:` | Build → wrapper `tr` | — | DEFAULT | §7a |
| 2 | tier 1: 3-of-3 + hash | keys (3,3); hash: pick from the payload's `hash:` records | supply `sha256(phrase)` instead of `sha256(sha256(phrase))` | **GAP** — an unspendable tier, composed silently; nothing on-device or in §8 states the 32-byte-preimage rule | §6c / **C-4** |
| 3 | tier 2: 2-of-2 + hash + `older(32768)` | relative → blocks → 32768 → echo | — | DEFAULT | §6b |
| 4 | tier 3: 1 key + `after(1173520)` | absolute → height → digits → "block 1173520" + lower-bound line from `now:` | pack a payload whose `now:` carries **no height** (the field is optional) | **GAP** — the height check silently does not run; the echo's shape is the only signal | §6b / **I-2** |
| 5 | tier 4: 1 key + hash + `after(1383520)` | same | — | DEFAULT | §6b |
| 6 | 4 paths listed | internal key = first-listed unlocked, unhashed one-key path; none exists → NUMS | — | DEFAULT; matches the fixture | §5, §8f |
| 7 | shape complete | stub screen | compare Template-ID against the fixture's `a00772ed…` | **GAP** — MISMATCH: the composer's fixed conjunct order (`and_v(v:KEYS, …)`) is not the fixture's (`and_v(v:sha256(H), multi_a(…))`). Different leaf script → different tapleaf → **different address**, not only a different id | §5/§7c / **I-10**, **M-3** |
| 8 | 7 sources, 7 slots | pick list of remaining sources, per slot | — | **GAP** — 7 rows; `ChoiceScreen` lays every choice out unscrolled and the largest shipped list is 5 | §7d / **I-5** |
| 9 | seated | mapping review: slot → fingerprint + origin | all seven share ONE origin, so the review's discriminator is the fingerprint alone | DEFAULT (fixture keys have distinct fingerprints) | §7d |
| 10 | consent | structural summary | 4 leaves, hashlocks and locks → `Renderable=false` | **GAP** — "Complex policy - cannot display safely." on a policy the operator authored tap by tap | §7e / **C-2** |
| 11 | engrave form | "template + keys" (keyless md1 + 7 mk1 cards) | — | **GAP** — the composed keyless template's per-slot FINGERPRINT declaration is unspecified; without it every card matches every slot at restore (`errSeatSlotContested`), the F-227 trap, on metal | §7f / **C-3** |
| 12 | census | plate census, then cut | pick "concrete policy" instead (616 policy chars, 16 keyed md1 chunks) | REFUSAL by census with the measured ceiling — but only at the end | §7f, §13.1 |

---

## J3 — Build with NO payload, ending in a keyless template (C26)

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | a flashed SH2, no payload loaded (or SKIPped at boot) | Wallet Policy door: "From payload" is absent | choose Build believing the payload is loaded | **GAP** — nothing says the key list is empty, nor that a payload exists but was skipped | §7a / **I-3** |
| 2 | Build | wrapper → preset/blank → paths | — | DEFAULT | §7b |
| 3 | a shape with N slots | lowering → keyless template | — | **GAP** — the lowering defines no ORIGIN for an unseated slot; the natural answer (pathless `m`) is refused by the fork's own decoder | §5 / **C-6** |
| 4 | shape complete | stub screen (template id + stub) | — | DEFAULT — this is C9's whole point and J3 is where it pays | §7c |
| 5 | — | seating "offered only when the payload holds keys or seeds" → not offered | wonder why; there is no screen | **GAP** — same silence as step 1 | §7d / **I-3** |
| 6 | consent | "Keyless template - no addresses. Verify off-device." | — | DEFAULT (measured at `gui/wallet_policy.go:243`) | §7e |
| 7 | engrave form | "concrete policy / template + keys" | there are no keys and no concrete policy | **GAP** — the form choice is not conditioned on seating state; §12.3 pins the outcome but §7f does not | §7f / **I-1** |
| 8 | plates | keyless md1 only | get the template onto a host to register the wallet | DOCUMENTATION ONLY — no QR display (§14), no NFC, no camera; the journey ends on metal by design | recorded |

---

# FINDINGS

## CRITICAL

### C-1 — §7d re-mints seated cards with the POLICY stub only; the engraved template then refuses them at restore
§7c says the screen "recommends stamping BOTH stubs on each key card". §7d says the
device mints one: *"a seated card is RE-MINTED for engraving with the new policy's
stub appended to its existing stubs"*. C9 (brainstorm §2) is unambiguous that the
intent is both, *"so one card matches either form of the wallet"*.

This is not a wording slip. Measured in `gui/key_card_seating.go:66-73`, layer 1 of
`seatKeyCards` is:

    stub, err := md.FormAwareStubChunks(templateMd1)   // the TEMPLATE's stub
    if !hasStub(c.Stubs, stub) { return errSeatNotThisPolicy }

So if the operator picks §7f's **template + keys** form — which J1 step 14 and J2
step 11 both do — the engraved keyless template's stub is the *WalletDescriptorTemplateId*
stub, and the re-minted cards carry only the *WalletPolicyId* stub. Every card is
refused at restore with "card does not belong to this policy", and the backup is
unrecoverable through the device path. **Data loss, discovered at recovery.**
Fix: §7d must say BOTH stubs, matching C9 and §7c.

### C-2 — §7e's consent surface cannot state the shape of anything the composer authors
§7e: *"The existing Wallet Policy consent surface (`walletPolicyConsentLines`): the
structural summary, the id NAMED by kind, receive and change addresses 0..1"*.

Measured: `walletPolicyConsentLines` (`gui/wallet_policy.go:163-228`) builds its
summary from `md1Summary(tpl)` (`gui/md1_inspect.go:84-99`), which is

    if tpl.Renderable { "Type: " + scriptName + policyLine }
    else              { "Complex policy - cannot display safely.", "Keys: N" }

and `Renderable` is **false** for exactly the shapes the composer exists to author —
pinned in the fork's own tests: `wsh(and_v(...))` → `Renderable=false`
(`md/md_test.go:337-344`), `tr(NUMS, sortedmulti_a)` → `Renderable=false`
(`md/md_test.go:416-423`). So on the operator's last screen before cutting steel, a
four-tier vault they authored by tapping reads "Complex policy - cannot display
safely." plus a key list.

Two consequences, both journey-shaped:
- A mis-tapped threshold, a lock on the wrong path, or paths in the wrong order is
  invisible at the one moment it can still be fixed. The composer *holds the path
  list*; the consent is derived from the md1 instead.
- A taproot policy whose first path was extracted as the **internal key** — a key
  that can spend alone, silently, for the lowest fee — is never announced. The
  shipped string for this exists (`"Key-path: A KEY CAN SPEND ALONE"`,
  `gui/template_engrave.go:152`) but lives in `policySummaryLines`, which
  `walletPolicyConsentLines` does not call (`policySummaryLines` has exactly one
  call site, `gui/template_engrave.go:86`). §7g has no row for it and §8 has no copy
  for it — only §8f's NUMS note for the *opposite* case.

The machinery to fix it ships: `md.PolicyShapeChunks` (`md/policy_shape.go:73`) is
already used by `gui/multisig_build.go:800`. **This is a false proof at consent.**
§7e as written is measurably wrong about the surface it cites, and §9 has no work
item for it.

### C-3 — the composed keyless template's per-slot fingerprint declaration is unspecified; J2's shape is unseatable without it
The reference wallet's own README states the trap and names it F-227: *"All seven
share one origin, so a keyless template that declares no fingerprints is unseatable
— every card matches every slot and the device refuses (`errSeatSlotContested`).
Pass one `--fingerprint @i=HEX` per slot."* Confirmed in the seating code
(`gui/key_card_seating.go:88-92`): two different cards claiming one slot is refused
as undecidable, and `slotMatchesCard` compares fingerprint *"when the template
declares one"*.

The spec never says whether a composed template records per-slot fingerprints. §5's
lowering table specifies the use-site (`/<0;1>/*`) and nothing about origin or
fingerprint declaration; §4f covers only seed-derived origins. The brainstorm's own
§3.4 worked example passes `--fingerprint` for all four slots — so the fact is known
and did not reach the spec.

Outcome if it stays silent: an operator composing any wallet whose cosigners share an
origin (RCW is the constellation's named reference for exactly this) engraves a
template + cards that this device can never re-seat. **Unrecoverable backup, on metal.**

### C-4 — hashlock entry never states the 32-byte-preimage rule; a phrase-derived digest composes a permanently unspendable path
§6c is three sentences: pick a `hash:` record, or type 64 hex; the consent screen
shows the digest; on-device preimage derivation is deferred. §8 has no copy for it.

`sha256(H)` compiles to `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL`, so the
witness preimage must be **exactly 32 bytes** and that is consensus-enforced. The
fixture README records what happens when this is not understood: *"Until then the
policies committed to `sha256(phrase)` directly, so tiers 1, 2 and 4 could never
satisfy `OP_SIZE` and were unspendable by anyone — three of four tiers, silently."*
That was in this constellation's own reference wallet, for months.

An operator whose secret is a passphrase will reach for `sha256(phrase)` — it is the
obvious thing, and the device accepts 64 hex without comment. Because §14 defers
on-device preimage derivation, the device can never detect it, and because the tier
is a *recovery* tier it is not exercised until it is needed. **Funds unrecoverable.**
The wrong outcome is far worse than telling the operator nothing, and the fix is
copy at the moment of entry (the digest must be of a 32-byte value; a phrase must be
hashed twice), plus a line on the consent screen.

### C-5 — "Back keeps assignments" has no rule for what a SHAPE EDIT does to slot indices
§7b: *"Back preserves everything ('going back should lose nothing')"*. §7d: *"Back
keeps assignments."* §7d's own all-or-nothing refusal **offers Back-to-edit** as one
of its two exits.

But slot indices are `@i` **by first appearance in the emitted text, computed after
lowering** (§5). Adding a path, changing an `n`, reordering paths, or introducing an
extractable internal key **renumbers every slot**. The spec never says what happens
to assignments made against the old numbering. Carrying them silently seats keys in
the wrong slots; the mapping review then shows the same fingerprint+origin hex the
operator already approved, so it cannot catch it — and §3.5(b) already concedes
*"the residual hazard is a mistap, which no derivation can detect"*.

**Wrong result, presented as reviewed.** A stated rule closes it: assignments survive
an edit only for slots whose (path, key-index-within-path) identity is unchanged;
everything else is cleared with a named line.

### C-6 — J3 has no defined slot ORIGIN, and pathless is refused by the fork's own decoder; §12.3's gate may be unsatisfiable
C26 blesses Build with no payload, §12.3 makes it a normative acceptance gate
("**Emulator walk with NO payload** ends in a keyless-template engrave"), and §5's
lowering table defines the use-site but **no origin for a slot**. Seated slots inherit
an origin from the key or from C28; an *unseated* slot has none, and J3 has nothing
but unseated slots.

The one obvious answer is refused by this device today. F-166 (`design/FOLLOWUPS.md:5801`,
still open, owning phase "its own cycle"):

    decodePayloadValidated("md1yqpqqxpsq258xsks3kh0ye")
      -> md: missing explicit origin        (md/md.go:893)

for a descriptor declaring `"path_decl":{"tag":"Shared","data":"m"}`. So a pathless
composed template would fail to decode on the very device that wrote it — at the
consent screen, which decodes the md1 to build its lines.

This is the "a plan may not close while one of its own gates has never been run"
shape: §12.3 names a walk that, as specified, has no defined artifact to produce.
Either §5 gains an origin rule for unseated slots (asked for, or defaulted per
wrapper and echoed), or J3's output form is specified some other way, or F-166 is
pulled into this cycle's scope. Silence is the one option that fails at the plate.

## IMPORTANT

### I-1 — §7f's engrave-form choice is undefined for two of the three journeys
Two distinct holes at the same screen.
(a) **J3 / any keyless template:** the choice is "concrete policy (text / QR / keyed
md1)" vs "template + keys (keyless md1 + mk1 cards)". With no keys, neither branch is
well-formed: there is no keyed md1 and there are no cards. §12.3 pins the *outcome*
(a keyless-template engrave) but §7f never says the choice collapses.
(b) **J1/J2 with `key:`-sourced slots:** §7f's form B is "keyless md1 + **mk1 cards**",
and §7d/§9.5 provide cards only by *re-minting an mk1 card that was already there*.
A slot seated from a `key:` record has no card. The spec never says the device mints
one (it can — fp + origin + xpub is exactly an mk1's payload, `mk/encode.go:39`), so
J1 — where all four slots come from `key:` records — has nothing defined to cut for
its keys.

### I-2 — the device is silent when it has NO pack-time bound, and the absence of a line is the only signal
§6b refuses a date or height below `now:`, and *"Without `now:` the echo shows the
typed value alone; the copy never says 'now'."* Nothing warns that the check did not
run. The two regimes differ only by the presence of §8c's extra two lines — the
"empty output is not absence" trap, on a funds-relevant screen.

Two ways in, both plausible: a payload packed by an `me` that predates the record, and
J3 (no payload at all, explicitly legal). A third is inside the record: the block
height is **optional** (`now:<unix-seconds>[,<block-height>]`), so a payload can check
a date lock and silently not check a height lock — J2 uses height locks exclusively.

Wrong outcome: an absolute recovery tier that is *already live* is composed and
engraved as a future tier. A one-line warning at the echo ("This device cannot tell
the time. Nothing here has checked that this is in the future.") costs nothing and is
strictly better than the current asymmetry.

### I-3 — Build after SKIPping the boot payload silently becomes the keyless-template journey
The boot offer is a real fork (`gui/sysw_load.go:43-53`, LOAD default / SKIP one tap
away / Back also skips), so "payload in flash, not in session" is a normal state. The
brainstorm's own C8 walk had a row for it — *"boot: operator SKIPs the boot load
offer, then chooses Build → refusal naming the route: 'No payload loaded. Load it
from the carousel'"* — and the spec **dropped that row when C26 legalised no-payload
Build**. §7g's door row now reads only "choose Build with no payload → DEFAULT:
compose a keyless template".

So the operator who meant to seat keys authors an entire shape, is never offered
seating (§7d: "Offered only when the payload holds keys or seeds"), is never told
why, and arrives at a keyless-template engrave. C26 correctly removed the *refusal*;
it did not licence the *silence*. The fix is a stated line at the Build door naming
the state ("No keys loaded — this will build a key-less template") and, when a
payload is present but unloaded, the route to load it.

### I-4 — §12.5's copy gates omit the modal-fits check, and every §8 body is a new modal
§12.5 lists `scripts/plan-glyph-check.sh` and the raster floor (`gui/raster_test.go`).
It does not list `assertModalBodyFits` / `gui/modal_fits_test.go`, which exists for
exactly this class: *"A modal's body scrolls and nothing on the frame says so, and
this machine has no button that scrolls it, so a sentence past the fold is a sentence
the operator is never told exists"* (`gui/multisig_build.go:869-874`). The fits test's
own comment records that F-185's second open item is that *"every other long modal in
the firmware carries the same unmeasured exposure"* (`gui/modal_fits_test.go:295-300`).

§8a, §8b, §8f and §8g are four new modal bodies, and §8g is the longest and most
load-bearing (it is the only thing standing between the operator and a nominal
2-of-3 one person can satisfy). A glyph gate proves each character *draws*; it does
not prove the last line is *on the screen*. Add the fits assertion to §12.5.

### I-5 — the seating pick list has no specified widget, and the shipped one does not scroll
§7d specifies a pick list "of the REMAINING sources" with no bound and no widget.
Measured in `ChoiceScreen.Draw` (`gui/gui.go:1993-2026`): every choice is laid out in
one vertical stack, `h` accumulates all of them, and the stack is centred with
`content.Center(image.Pt(maxW, h))`. There is no scroll, no paging, and no
scroll-to-selection — Up/Down move `s.choice` (`gui/gui.go:1928-1949`) but the draw
does not follow it, so a selected row past the fold is selected and invisible. The
content box is `dims.Y (320) − 2 × leadingSize (44)` = 232 px
(`gui/gui.go:1967-1973`, `gui/theme.go:43`), and the largest shipped list is five
rows ("12 WORDS / 24 WORDS / M*1 STRING / SLIP-39 / SEED XOR", `gui/gui.go:2865`).
J2 needs seven; a payload may hold many more, and §4b permits up to 72 slots.

I could not render a row to get the exact capacity (no Go toolchain on this shell) —
that is a plan-time machine check, not a review question. What the spec owes is the
widget and its stated capacity, plus the refusal or paging behaviour past it.
Contrast `confirmReviewScreen`, which *does* page and draws its pager only when there
is a second page (`gui/multisig_build.go:1908-1931`) — the consent screen is safe;
the pick list is the exposed one.

### I-6 — the digit pad as specified cannot type the date format the same table requires
§6b/C25 define the widget as "a NEW digit-pad widget (digits, backspace, done)" and
the absolute/date row's entry as `YYYY-MM-DD`. The hyphens cannot be typed. Cheap
fix (eight digits with a formatted echo), but as written the one new widget in the
cycle cannot express one of the four entry rows. Related and unstated: no rule
refuses an impossible date (2027-02-31).

### I-7 — no "nothing outside this device has checked this policy" warning, on the first device-authored arbitrary policy
Multisig Build shows one before cutting, and its body was rewritten precisely so it
asks for a comparison that *can* fail (`gui/multisig_build.go:872-880`): *"Nothing
outside this device has checked this policy. Before you fund it, compare the keys you
just reviewed, or the descriptor on the restore doc, against the same wallet in your
coordinator… What settles it is restoring these plates in your coordinator and seeing
your own first receive address."*

§7e reuses **Wallet Policy's** consent instead — a surface built for a policy that
came from OUTSIDE, where an external check already exists by construction
(`gui/gui.go:191-193`). The composer inverts that premise: no coordinator has this
wallet, and the addresses on the consent screen were derived by the same device that
authored the policy. §8 carries no equivalent copy and §7g has no row for it. The
composer needs Multisig Build's warning, or its own.

### I-8 — §7c teaches an id the operator can invalidate by taking the Back that §7d offers
C9's rationale for teaching the stub before seating is that the template id is
*"key-independent and origin-invariant, so it is final before any seating"* — and
that half holds (no origin/path-decl bits enter the preimage,
`md/template_id_test.go:124`). But it is **not shape-invariant**, and §7d's own
refusal offers "Back-to-edit" as an exit. An operator who wrote the 8 hex down, went
back and added a path now holds a stub for a wallet that no longer exists, and may
mint cards against it on the host.

Not Critical because a stub is *"a human-indexing aid, not a cryptographic
primitive"* (mk SPEC §3.3, quoted at brainstorm §3.5(a)) and the recovery check is
recomputation — but a card stamped on steel with the wrong stub is not free. The spec
should say the stub screen is re-shown after any shape edit, and that its value
changes if the shape does.

### I-9 — C5's most surprising consequence never reaches the operator during authoring
C5 means a person who appears in two paths must hand over **two keys from two
hardened accounts**, and the brainstorm states the cost plainly: *"a FOREIGN cosigner
in two paths hands over two cards (hardened accounts cannot be derived from an
xpub)"*. J1 is precisely that wallet, and it is the archetype the whole cycle exists
to support.

Nothing at the shape step says so. The operator authors "Path 2: 1 key + 182 days",
and the first the device mentions it is the all-or-nothing refusal at the seating
transition, which names *counts* ("4 slots, 3 keys") and not the cause. The operator
then has to go back to the host, and to a third party, with the whole shape already
authored. Cheap fix: a live "slots: 4 / keys available: 3" on the path-list screen
whenever a payload is loaded, and one line of §8 copy on the second-account rule.

### I-10 — composing a wallet the host already has yields a different id AND a different address
§4d says the presets are *"NOT byte-identical"* to the toolkit goldens. That statement
is true of far more than the presets: the lowering is a single fixed spelling, so
**any** policy an operator also holds on the host in another spelling composes to a
different template. J2 is the demonstration: the composer emits
`and_v(v:KEYS, …)` (§5, "inside a path"), while `tr.policy` spells tier 1
`and_v(v:sha256(H1), multi_a(3,…))`. Same spend conditions, different leaf script,
therefore a different tapleaf hash, therefore **a different merkle root and a
different address** — not merely a different id.

The stub-teaching screen (§7c) and the named id at consent (§7e) invite exactly the
comparison that will now fail, and the shipped code already names the consequence of
a spurious id mismatch: *"an operator comparing the wrong one against a coordinator
reads the mismatch as a corrupted backup"* (`gui/wallet_policy.go:189-192`). One
sentence of copy at the stub screen and a generalisation of §4d close it.

## MINOR

- **M-1 — `hash:` does not follow the §5.3 pattern §6a says all three follow.** §6a:
  *"All three follow `SPEC_systemwide_payloads.md` section 5.3"*. §5.3's normative
  decode is hex → **UTF-8** (*"a `ClassFreeText` record's body is hex-decoded back to
  UTF-8"*). `key:` and `now:` are hex of UTF-8 text; `hash:` is hex of the raw
  32 bytes. Unambiguous in practice, but an implementer applying one rule to all three
  gets garbage for one of them. Say so in the row.
- **M-2 — §6a names the wrong comment site.** §6a says *"The enum comment at
  `gui/gui.go:191` … is rewritten (C12)"*. The comment that actually states the
  rationale C12 reverses is in `gui/sysw_admit.go` at `progWalletPolicy`: *"NO seed
  class. The Wallet Policy program never derives from a secret… Least privilege, and
  it is enforced here rather than by the flow declining to ask."* Both need rewriting;
  the spec names only the weaker one. (Repo memory: comments outlive their conditions.)
- **M-3 — "byte-identical" is used for two different things.** §5a: *"Conjunct order is
  byte-identical while LOCK is last"* is a claim about byte **count** in a witness-cost
  analysis. §5 and §12.1 use "byte-identical" for actual bytes. Reordering conjuncts
  changes the script and the address (see I-10); a reader who takes §5a literally would
  conclude order is free.
- **M-4 — the replacement label in §7c's id/stub fix is unspecified.** §7c says the
  ambiguity between `gui/wallet_policy.go:194` (16-byte id) and
  `gui/template_engrave.go:70,79` (4-byte stub) *"is fixed in the same change"* but not
  what either becomes — and its own blockquote reuses `Template-ID:` for the 32-hex id.
  Four assertions pin the current wallet-policy strings
  (`gui/wallet_policy_test.go:44,47,97,100`).
- **M-5 — §8g's example is unreachable in `tr`.** *"Slots @0 and @2 are the same seed.
  This path's 2-of-3…"*: under §5 an extracted internal key is `@0` and is *"then not a
  leaf"*, so `@0` and `@2` cannot share a path in a taproot policy. Harmless as an
  illustration; pick indices that can co-occur.
- **M-6 — the typed-hex hashlock fallback has no stated validation.** §6c allows typing
  64 hex on the keyboard with no stated length/case check and no confirm-before-accept
  step. One wrong character is an unspendable path (see C-4), and the consent screen's
  digest is the only chance to catch it.
- **M-7 (secret-handling — follow-up only, never blocking per the 2026-08-27 operator
  ruling).** C14/§9.9 wires scrub-on-exit "through `buildMultisigSeedHook`'s seam", but
  the composer holds seeds across strictly more screens than Multisig Build does (shape
  → seating → mapping review → consent → census → engrave-form → cut). The spec does not
  say when a seed or per-seed passphrase is dropped if the operator abandons at the
  census or at the form choice. Log for future optimisation; it does not gate.
- **M-8 — no total-slot bound.** §4b bounds paths (1..8) and per-path n (1..9); the
  product is 72 slots with nothing stated about the total, the pick-list scale, or the
  md1 chunk count. The only backstop is the plate census at the very end (§7g last row).

## NIT

- **N-1** — §4b's "at most one of `older` or `after`" per path is *why* C11's
  timelock-mixing rule is satisfied structurally. The spec never connects them, so a
  reviewer looking for the mixing refusal will not find one.
- **N-2** — §7c's taught command is `mk encode ... --policy-id-stub <8 hex>`. The
  ellipsis hides the required `--keys`/`--from-md1` argument, and `mk encode` requires
  one of `--policy-id-stub` or `--from-md1` (brainstorm §3.5(a)). Naming the flag is the
  teachable part, so this is a nit — but the screen is the operator's only record of it.

---

# RECORDED AND NOT ACTIONED — do not re-raise

**NOT OUR CONCERN**
1. A `key:` record cannot carry a human label, so the pick list and the mapping review
   identify people by fingerprint + origin hex. No label field exists anywhere in the
   constellation's wire; adding one is not this cycle.
2. When exactly one source remains for the last slot, the device still shows a one-row
   pick list. Harmless; making it auto-seat would remove the operator's last look.
3. The composed template can leave the device only on metal — no on-screen QR (§14,
   staged plan 6b), no NFC hardware (C8), no camera (the SH2 has none, by design). The
   journey ending at the plate is a deliberate deferral, not a gap.
4. `md`'s acceptance of `older(0x400000)` — already filed and already handled: §4c/§6b
   have the device enforce its own table rather than rely on md's guard, and §10.4
   carries the patch.

**DOCUMENTATION ONLY**
5. A card whose origin script type disagrees with the wrapper (a `.../2'` key under
   `tr`). §7g already classes it DOCUMENTATION and C28 settled it: nothing measured
   refuses or warns on any origin, the origin is provenance. Note that the brainstorm's
   own §3.4 worked example is an instance of it (`48'/…/2'` slots under `tr`).
6. The brief's premise that RCW has "a keyless hashlock tier" is stale — tier 4 was
   keyed on 2026-08-22 by operator ruling (*"keyless path is not reasonable"*), adding a
   seventh seed. A keyless tier under `tr` would be refused by §4e and by
   rust-miniscript's `requires_sig`. Worth a line in §4d/§13 so the next reader of the
   fixture is not misled, but the spec is not wrong.

**VERIFIED CLEAR THIS ROUND — measured, do not re-derive**
7. `md.TemplateEngraveShapeGuardChunks` does **not** dead-end the composer at its own
   consent screen. `sortedmulti_a` has been admitted since F-215 and `sortedmulti` is
   refused only under a combinator (`md/template_guard.go:57-90`); §5 emits
   `sortedmulti` only for the unlocked single-path wsh case, which the `tagWsh` arm
   walks with `inCombinator=false`. J2's four-leaf taptree passes.
8. F-218's duplicate-slot refusal (`md.DuplicateKeySlots`, `gui/wallet_policy.go:222-229`)
   does **not** conflict with C29. It refuses the same *key* at two slots; C29 concerns
   one *master* at two hardened accounts, which are different xpubs.
9. The template id taught at §7c survives seating: no origin or path-decl bits enter the
   preimage (`md/template_id_test.go:124`). Only a shape edit can stale it — that is I-8,
   and it is the only route.
10. `confirmReviewScreen` pages, and draws its pager only when a second page exists
    (`gui/multisig_build.go:1908-1931`), so a long consent line list is fully reachable.
    The unscrollable surfaces are the modals (I-4) and `ChoiceScreen` (I-5).

**NOT MEASURED — plan-time machine checks, not review questions**
11. `ChoiceScreen` row capacity at 480x320 with `poppins.Bold20 / LineHeightScale 0.70`
    (`gui/theme.go:99-103`). No Go toolchain on this shell; the arithmetic bound is a
    232 px content box, and the count belongs in the plan with the rendered number.
12. Whether each §8 body fits its modal frame (`assertModalBodyFits`). Same reason;
    see I-4, which is that this gate is not in §12.5 at all.
