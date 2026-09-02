# R0 round 1 — SPEC_wallet_policy_composer.md — LENS: THE OPERATOR'S JOURNEY, WALKED AGAIN

**Artifact:** `design/SPEC_wallet_policy_composer.md`, 735 lines, last touched by
`bc1c07c` ("fold: composer spec R0 round 0"). Working tree at `aa022ae`, spec
unchanged since the fold.
**Lens:** the same three questions per step, over the REGENERATED §4-§14, on a
480x320 panel with no clock, no NFC and a digit pad that does not exist yet.
**Reviewer:** independent agent, read-only. No repo file modified except this report.
**Heads read:** mnemonic-engrave `aa022ae`; fork `bg002h/seedhammer` `169073c` at
`/scratch/code/shibboleth/seedhammer`.

**Counts: 1 Critical / 9 Important / 6 Minor / 2 Nit.**

**Settled and NOT re-raised.** Round 0's C-1..C-6 and I-1..I-10, its
NOT-OUR-CONCERN / DOCUMENTATION-ONLY / VERIFIED-CLEAR lists, operator rulings
C1..C29 and the brainstorm §3.12 controller defaults. Where the fold's answer
holds I say so in the "verified closed" list at the end rather than restating it.

**Method note — what was MEASURED this round, not read.** No Go toolchain on this
shell (`go` not found), so nothing was rendered; every pixel claim below is
arithmetic over measured constants and is marked as a plan-time render check.
Measured by command: `plan-glyph-check.sh` on the spec (55 strings, 0
undrawable); the non-whitespace size of every §8 body and of the §7c screen at
1..72 slots; `modalBodyMargin = 80` and the 588/~500 character figures in
`gui/modal_fits_test.go:26-46`; `assertChoiceLabelFits`'s budget
(`gui/multisig_build_prose_test.go:508-519`, `buttonPadX = 6`, `gui/gui.go:56`);
every shipped `ChoiceScreen{Lead:}` string length; `slotMatchesCard`'s predicate
(`gui/key_card_seating.go:117-140`); the ClassUnknown contract
(`sysw/descriptor.go:46-48`); §8c's four arithmetic claims.

---

## J1 — the two-path taproot wallet (brainstorm §3.4 worked example)

`tr(NUMS,{multi_a(2,@0,@1,@2),and_v(v:pk(@3),older(26280))})`. Payload: four
`key:` records (A acct1 `48'/0'/1'/2'`, B `48'/0'/0'/2'`, C `48'/0'/3'/2'`,
A acct2 `48'/0'/2'/2'` — A's two share fingerprint `73c5da0a`), one `now:`.
Four slots, two paths, no extractable internal key (path 2 is locked), so NUMS.

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | payload LOADED at boot | Wallet Policy door: "Scan cards" / "From payload" / "Build a new policy", with "Keys loaded: 4" beneath Build | tap **From payload** — the payload holds four `key:` records and NO md1 | **GAP** — the route is offered whenever *a payload* is loaded, not when it holds a policy; it dead-ends | §7a / **M-1** |
| 2 | "Build" | wrapper picker → `tr` | pick a preset | DEFAULT | §7b, §4d |
| 3 | wrapper set | path list, empty; live line "slots: 0 / keys available: 4" | — | DEFAULT (r0 I-9 closed) | §7b |
| 4 | Path 1 | keys n=3, k=2 | — | DEFAULT | §7b |
| 5 | Path 2 | keys n=1, k=1; lock: relative → blocks → `26280` → "26280 blocks (about 182 days)" | type a date instead | DEFAULT — the pad now types `YYYYMMDD` and echoes `YYYY-MM-DD` (r0 I-6 closed) | §6b |
| 6 | relative lock, `now:` present | echo carries no bound line (relative locks need no time) | — | DEFAULT | §6b, §8c |
| 7 | shape complete, "slots: 4 / keys available: 4" | **stub screen**: Template-ID 32 hex, mk1 template stub 8 hex, `mk encode ... --policy-id-stub`, **"Slot @0 expects a key at m/48'/0'/0'/3'"** ×4, §8d's three lines | believe the screen and go derive four keys at `48'/0'/0..3'/3'` on the host — or mint mk1 cards there against those origins | **GAP** — the expected-origin lines are §4f's *unseated* rule; this template is about to be seated from records at `48'/0'/{0,1,2,3}'/2'`, which §4f says are carried VERBATIM. The screen states an origin the next step overwrites | §7c/§4f / **I-1** |
| 8 | the same screen | 4 slot lines + id + stub + command + §8d | — | **GAP** — 338 non-ws chars pre-seating, 404 post-seating, against a body measured to draw ~588 in full and cut at ~500; unbounded above (72 slots = 2648) | §7c / **I-2** |
| 9 | 4 sources, 4 slots | "Slot @0, Path 1 key 1 of 3: choose a key" over the remaining sources; labels fingerprint + origin | — | DEFAULT; label is 23 chars, well inside the 436 px row | §7d |
| 10 | 3 seated | last slot | — | NOT OUR CONCERN (r0 item 2) | recorded |
| 11 | mapping review | slot → fingerprint + origin; A's two accounts get the informational line + §8k | Back to the path list, then **Back again to the wrapper picker** and change `tr`→`wsh` | **GAP** — §8j's discard is scoped to "the path list"; a wrapper change is not one, yet it renumbers every slot | §7d/§8j / **C-1** |
| 12 | consent | per-path k-of-n, lock in operator units, digests, §8f NUMS note, id by kind + both stubs, receive+change 0..1, self-check, then §8l | — | **GAP** — no widget stated for the longest content in the flow; §9 item 7 required paging for the pick list and item 9 required nothing here | §7e/§9.9 / **I-3** |
| 13 | consent, self-check fires | "The policy on this device does not match what you built." | ask what to do | **GAP** — the one refusal in the flow that names no action, and §12.4 cannot construct it from an input | §7e / **I-8** |
| 14 | engrave form | A concrete policy / B template + keys; all four slots came from `key:` records and are MINTED as mk1 with both stubs | — | DEFAULT (r0 I-1 closed) | §7f |
| 15 | census, then cut | census; read-back integrity by form | — | DEFAULT | §7f |

---

## J2 — "our reasonably complex wallet" (`design/fixtures/reasonably-complex-wallet/`)

**Premise, re-confirmed:** tier 4 has been KEYED since 2026-08-22 (*"keyless path
is not reasonable"*); the fixture has three `sha256` hashlocks and no keyless
tier. Per the brief I walked BOTH: the operator who *wants* the pre-2026-08-22
keyless tier 4 under `tr`, and the wsh route the refusal sends them to.

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | 7 `key:` + 3 `hash:` + `now:` | door: "Keys loaded: 7"; Build → `tr` | — | DEFAULT | §7a |
| 2 | tier 1: 3-of-3 + hash | keys (3,3); then hash: **pick from the payload's three `hash:` records** | pick H2 for tier 1 | **GAP** — no row content specified; three 64-hex digests are indistinguishable, and a 64-char label does not fit a 436 px row and does not wrap | §6c/§7d / **I-7** |
| 3 | hash chosen | §8i fires at entry: SHA-256 of a 32-byte value, hash a passphrase twice | supply `sha256(phrase)` | REFUSAL is impossible (the device cannot tell), but the copy now says so at entry AND consent — r0 C-4 closed | §6c, §8i |
| 4 | tier 2: 2-of-2 + hash + `older(32768)` | relative → blocks → 32768 → echo | — | DEFAULT | §6b |
| 5 | tier 3: 1 key + `after(1173520)` | absolute → height → digits → "block 1173520" + bound line, or the "cannot tell the time" line when `now:` carries no height | pack a `now:` with no height | DEFAULT — the absence is now SPOKEN (r0 I-2 closed) | §6b, §8c |
| 6 | tier 4, keyless: hash + `after(1383520)` | **REFUSE: "Taproot cannot hold a key-less path. Use wsh, or add a key."** | — | REFUSAL, and it names both exits | §4e |
| 7 | the refusal | operator goes Back to the wrapper picker and chooses `wsh` | this is the wrapper change of J1 step 11, and here the spec's own refusal ROUTES them into it | **GAP** — if any seating had begun, assignments cross a renumber silently | §4e→§7d / **C-1** |
| 8 | wsh, 4 paths, 6 slots | §8a EXPERIMENTAL keyless-path confirm, once, at the moment tier 4 is added | — | DEFAULT, correct and unskippable | §8a |
| 9 | tiers 1/2 multi-key, non-sole | lowering forces `multi`, not `sortedmulti`; §8b does NOT fire (it fires only where sorted was legal and declined) | — | **GAP** — the operator gets exactly the property §8b warns about and is not told; and §8b's stated cost is itself removed by §5's declarations | §5/§8b / **M-6** |
| 10 | every path hashed? | tier 3 has none, so §8h does not fire | — | DEFAULT, correct | §8h |
| 11 | shape complete | stub screen, 6 slot lines | — | **GAP** — 404 non-ws pre-seating, 470 post-seating, against a ~500 cut | §7c / **I-2** |
| 12 | 7 sources, 6 slots | paged pick list (§9 item 7); all seven share origin `270028'/0'/0'/0'` but have distinct fingerprints | — | DEFAULT — r0 C-3 closed: §5 declares the fingerprint per seated slot and `slotMatchesCard` compares origin AND fingerprint | §7d, §5 |
| 13 | consent | 4 paths × (k-of-n, lock, digest first-8/last-8, EXPERIMENTAL mark) + id + both stubs + 4 addresses | — | **GAP** — the same unstated widget; 4 bech32 addresses alone are 248 non-ws chars | §7e / **I-3** |
| 14 | census | measured ceiling, 616-char concrete descriptor refused for form A | — | REFUSAL, recorded r0 | §7f, §13.1 |

---

## J3 — Build with NO payload (C26)

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | flashed SH2, boot LOAD offer SKIPped | door in every state; beneath Build: "A payload is in flash but not loaded. Load it from the carousel first." | proceed anyway | DEFAULT, and the line is 69 chars — inside the shipped `Lead` precedent (longest measured 100). r0 I-3 closed | §7a |
| 2 | no payload at all | "No keys loaded. This builds a key-less template." | — | DEFAULT | §7a |
| 3 | Build | wrapper → blank → paths; no "slots / keys available" line (no payload) | — | DEFAULT | §7b |
| 4 | a 3-of-5 wsh shape | lowering; unseated slots declare `m/48'/0'/i'/2'`, no fingerprint | — | DEFAULT — r0 C-6 closed by §4f; the F-166 pathless trap is avoided and `errSeatSlotContested` cannot fire | §4f, §5 |
| 5 | shape complete | stub screen with FIVE expected-origin lines + id + stub + command + §8d | write it all down | **GAP** — 371 non-ws chars, and this is the journey where the lines are load-bearing; nothing bounds the body | §7c / **I-2** |
| 6 | seating | not offered; the door already said why | — | DEFAULT (r0 I-3 closed) | §7d |
| 7 | consent | "Keyless template - no addresses"; per-path shape from the decoded md1; self-check; §8l | — | DEFAULT | §7e |
| 8 | engrave form | collapses to "template only" and says so | — | DEFAULT (r0 I-1(a) closed) | §7f |
| 9 | plates | keyless md1 only | get it to a host | DOCUMENTATION ONLY (r0 item 3) | §14 |

---

## J4 (NEW) — preset → edit → seat from a short payload → refuse → Back → discard → re-seat

Preset `tiered-recovery` (`or_i(sortedmulti(k1,…), and_v(v:older(N), thresh(k2,…)))`
in the toolkit) → 2 paths. The operator raises path 1's `n` from 3 to 4 and adds
a hashlock to path 2. Payload holds 5 keys; the edited shape has 7 slots.

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | wrapper picker | pick `sh(wsh)` (the operator is migrating a legacy multisig), then pick the `tiered-recovery` preset | — | **GAP** — preset availability per wrapper is unspecified; the preset populates a 2-path shape §4a admits only for `wsh`/`tr`, and §4e's refusal arrives at the end although §4e elsewhere suppresses at the picker | §4d/§4a / **M-3** |
| 2 | `wsh`, preset applied | path list populated; "slots: 6 / keys available: 5" | — | DEFAULT; the count line is what makes step 5 predictable | §7b |
| 3 | edit path 1 n=3→4 | slot count 6→7; the line updates | — | DEFAULT | §7b |
| 4 | add hashlock to path 2 | §6c entry, §8i fires | — | DEFAULT | §6c |
| 5 | shape complete → stub screen → seating | **REFUSE at the transition**, naming both counts and the cause: *"Path 2 needs a second key from the same person: a second account, a second card"* | — | **GAP** — the cause is FALSE here: the operator is short a key because they enlarged `n`, not because of C5. A refusal that misdiagnoses sends them to derive an account they do not need | §7d / **I-5** |
| 6 | the refusal's exits | "Back-to-edit" or "engrave as a keyless template" | take Back-to-edit | **GAP** — §8j says "Every key you seated will be cleared" and nothing was seated; "after seating began" is undefined for a refusal that fires before any assignment | §7d/§8j / **M-2** |
| 7 | payload holds one BIP-39 seed instead of 5 keys | seating: a seed "may fill several slots" — but "Each source is used at most once" and "fewer sources than slots → REFUSE" | — | **GAP** — 1 source, 7 slots. Two sentences of §7d give opposite answers, and §12 item 10's Multisig-Build parity gate needs the reading §7d forbids | §7d/§12.10 / **I-6** |
| 8 | the operator packed 6 keys, one as a bare xpub | §6a: that record is `ClassUnknown` and "refused with its own line (§11)" | look for the line | **GAP** — measured: a ClassUnknown record "stays in the session, is offered to nobody, and **reaches no screen**". The door says "Keys loaded: 5" and the promised fix-naming line has no surface | §6a / **I-4** |
| 9 | edit accepted | §8j confirm, all assignments discarded, stub screen re-shown, re-seat | — | DEFAULT (r0 C-5 closed for path-list edits) | §7d, §7c |

---

## J5 (NEW) — the shape edit AFTER the stub was written down

| # | in hand, EXACTLY | device does (spec) | what ELSE might they do | class | § / GAP |
|---|---|---|---|---|---|
| 1 | stub screen, first time | Template-ID + 8-hex template stub + `mk encode ... --policy-id-stub` | copy the 8 hex onto paper | DEFAULT — this is C9's whole point | §7c |
| 2 | 8 hex on paper | — | go to the host and mint mk1 cards with that stub **and the origins the same screen just stated** | **GAP** — see I-1: those origins are the unseated rule and will be overwritten by seating; `slotMatchesCard` refuses on an origin mismatch at restore | §7c/§4f / **I-1** |
| 3 | back on the device | operator goes Back, adds Path 3 | — | DEFAULT | §7b |
| 4 | shape re-completed | stub screen RE-SHOWN, "This id changed with the shape." | ask what happens to the cards already minted | **GAP** — the line names the change and not the consequence; `seatKeyCards` layer 1 refuses any card lacking THIS template's stub, so those cards are dead to the device path | §7c / **M-5** |
| 5 | the re-shown screen | id, stub, command, N origin lines, §8d | — | **GAP** — I-2 again, and this is the screen the operator is being asked to re-read carefully | §7c / **I-2** |
| 6 | — | glyph gate covers §8's blockquotes | — | **GAP** — "This id changed with the shape." is prose-quoted, not blockquoted; the gate does not scan it | §11/§12.5 / **I-9** |

---

# FINDINGS

## CRITICAL

### C-1 — the discard-on-edit rule is scoped to "the path list", but §5's slot numbering also depends on the WRAPPER; J2's own refusal routes the operator into the uncovered edit

§7c says the stub screen is re-shown "after any **shape** edit". §7d and §8j say
assignments are discarded on "any change to the **path list**". §7b defines the
shape as *"Wrapper → preset or blank → paths"* and promises *"Back preserves
everything"*. So a wrapper change is a shape edit that is not a path-list change,
and the narrower word is the one that decides whether seated keys are cleared.

It renumbers. §5's tr rule extracts *"the FIRST-LISTED unlocked, unhashed one-key
path (then not a leaf)"* as `@0`; wsh extracts nothing. Take a 2-of-3 plus a
bare single-key path: under `tr` the single key is `@0` and the 2-of-3 is
`@1,@2,@3`; under `wsh` the 2-of-3 is `@0,@1,@2` and the single key is `@3`.
Every index moves. Numbering is *"by FIRST APPEARANCE in the emitted text"* and
the emitted text is a function of the wrapper.

Nothing downstream catches it. The mapping review shows slot → fingerprint +
origin, i.e. the same (fingerprint, origin) pairs the operator already approved,
against different slot numbers. §7e's self-check *"asserts that the decoded shape
equals the composed path list"* — **shape, not assignment** — so a permuted
seating passes it and is presented as reviewed. §3.5(b) already concedes *"the
residual hazard is a mistap, which no derivation can detect"*; this is that
hazard produced by the device rather than by the operator.

And J2 is not a contrived route to it: §4e's own refusal, *"Taproot cannot hold a
key-less path. **Use wsh**, or add a key."*, tells the operator to make exactly
this edit. Same class and same outcome as round 0's C-5, through the one route
the fold left uncovered.

**Fix:** one word. §7d/§8j discard on any change to the **shape**, wrapper
included; and §12 item 4's negative-vector family gains "a wrapper change after
seating began".

## IMPORTANT

### I-1 — §7c states a per-slot expected origin with no stated condition, and the very next step overwrites it

§4f introduces the line inside a sentence scoped to C26: *"**Unseated slots (a
keyless template composed with no keys, C26)** declare the §4f origin with
`account' = the slot's emitted index` … the template screen (§7c) states the
expected origin per slot."* But §7c's own heading says the screen is shown
**UNCONDITIONALLY**, and its blockquote — the only rendering of the screen
anywhere in the spec — carries `Slot @0 expects a key at m/48'/0'/0'/3'` with no
condition on it. §12.3 asks for the line in the no-payload walk and §12.2 does
not ask for it in the payload journey, so the intent is inferable from the
acceptance gates and absent from the screen.

In J1, J2 and J4 the line is false by the next screen: §4f says a slot seated
from a `key:` record or card *"carries the origin the record or card DECLARES,
verbatim"*. J1's four keys are at `48'/0'/{0,1,2,3}'/2'`; the screen says
`48'/0'/{0..3}'/3'`. Two screens of the same flow contradict each other, and the
operator has no way to know which is binding.

The escalation is not hypothetical, because C9's stated purpose for this screen
is that the operator *"might later choose to use mk1 encoding"* on the host. An
operator who mints cards against the stated stub **and** the stated origins gets
cards that `slotMatchesCard` refuses at restore — it compares the card's parsed
path against the slot's declared origin component by component
(`gui/key_card_seating.go:117-140`), so a card at `48'/0'/0'/3'` matches no slot
in a template seated at `48'/0'/1'/2'`. That is round 0's C-1 outcome (cards
refused at restore, on metal) reached from the screen the fold added.

**Fix:** state the condition on the screen. Unseated slots → "Slot @i expects a
key at …"; a payload with keys → either omit the line or mark it "if you seat
nothing here".

### I-2 — the §7c screen's body grows with the slot count and nothing bounds it; §12.5's fits assertion cannot pin a variable body

§8's header claims *"every body passes the modal-fits assertion"* and §12 item 5
runs `assertModalBodyFits` on *"every §8 body and every new screen"*. The stub
screen is a new screen whose body is not a §8 body and is not a fixed string: it
is id + template stub + command + §8d's three lines + **one line per slot** (+
two more lines after seating), and §4b admits 8 paths × n ≤ 9 = **72 slots**.

Measured (non-whitespace characters, the unit `normalizeDrawn` uses):

| slots | pre-seating | post-seating (both stubs) |
|---|---|---|
| 1 | 239 | 305 |
| 4 (J1) | 338 | 404 |
| 5 (J3) | 371 | 437 |
| 7 (J2/J4) | 437 | **503** |
| 9 | **503** | 569 |
| 72 | 2582 | 2648 |

Against `gui/modal_fits_test.go:26-46`: both modal shapes drew **588** normalized
characters in full, F-185's real refusal was cut at **~500**, and every screen
must clear its own body by `modalBodyMargin = 80`. So a seven-slot wallet — the
reference wallet — sits on the cut line, and the grammar's own ceiling is five
times over it. A body past the fold is text the operator is never told exists
and, on this machine, cannot scroll to.

This is not the same finding as round 0's I-4 (which asked for the gate to be
listed at all, and the fold listed it). The gate is now listed and **cannot
constrain this screen**, because the assertion compares a drawn frame against a
source string and there is no single source string.

**Fix:** the spec owes this screen a widget with paging, exactly as §9 item 7
gives the pick list, or a stated per-frame slot budget with a "more" affordance.
The exact per-frame capacity is a plan-time render measurement.

### I-3 — the composer's consent surface, the longest content in the flow and the last screen before steel, has no stated widget

§7e defines a NEW surface and enumerates a great deal for it to say: per path in
listed order its k-of-n or single key, its lock kind and value in operator units,
its digest as first-8/last-8, and its EXPERIMENTAL marks; then the key-path line;
then the id NAMED by kind **with both stubs**; then receive and change addresses
0..1; then §8l. §9 item 9 gives it no widget.

Round 0 verified `confirmReviewScreen` pages and draws its pager only when a
second page exists (`gui/multisig_build.go:1908-1931`) — but that clearance
belongs to the *shipped* Wallet Policy consent, and §7e says explicitly that
neither shipped surface *"may be the composer's consent"*. The new one inherits
nothing by default.

Scale: eight paths at one to two lines each, plus a 32-hex id, plus two 8-hex
stubs, plus four bech32m addresses (248 non-whitespace characters by themselves).
The fold recognised this exact exposure for the pick list and wrote *"as a PAGED
widget with stated capacity"* into §9 item 7; the same sentence is missing for the
screen where the consequence is worse. If the composer's consent does not page,
the later spend paths of a four-tier vault are invisible on the one screen that
can still stop the cut.

**Fix:** state paging and capacity in §9 item 9 as §9 item 7 already does, or
name `confirmReviewScreen` as the base.

### I-4 — a malformed `key:` / `hash:` / `now:` record reaches NO screen; §6a's promised per-failure refusal line has no surface, and §12.8 passes either way

§6a: *"Body validation after hex-decoding, each failure `ClassUnknown` and
**refused with its own line (§11)**"*, and *"a bare xpub is refused **naming the
fix**"*.

Measured, the shipped contract for that class (`sysw/descriptor.go:46-48`,
verbatim): *"A record failing any of it is ClassUnknown and goes INERT — the
existing contract for an unclassifiable record (it **stays in the session, is
offered to nobody, and reaches no screen**)."*

So there is no moment at which the promised line can be shown. The operator packs
six keys, spells one as a bare xpub — the single most likely `key:` mistake, and
the one §6a singles out — and the device says nothing at all. The only signal is
that the door's count reads "Keys loaded: 5", which the operator can only read as
wrong if they remember packing six. This is the "empty output is not absence"
trap on the input side of a funds-relevant flow.

Worse, the acceptance gate cannot see it. §12 item 8 asserts each malformation
*"classifies identically on the host and on the device"* — both say Unknown, the
gate is green, and the operator-facing half of §6a was never tested. It then
compounds into I-5: the operator meets a seating refusal short by one key, and
the refusal blames a cause that has nothing to do with what happened.

**Fix:** §6a must name the surface. Either the payload-load path grows a
"records this device could not read: N" line (a change with a §11 refusal line
attached), or §6a drops the promise and says plainly that a malformed record is
inert and invisible, and the door's count is the only signal — which is at least
true, and lets §12.8 gain the assertion that matches.

### I-5 — the seating-shortfall refusal names ONE cause for a condition that has several

§7d: *"fewer sources than slots → REFUSE at the transition, naming both counts
AND the cause (\"Path 2 needs a second key from the same person: a second
account, a second card\")"*. §11 requires every refusal in this family to name
what to do instead, and §12 item 4 requires *"the exact line shown"* — so this
string is the specified line, not an illustration.

It is right for exactly one cause: C5's person-in-two-paths (round 0's I-9). It
is wrong for at least three others reachable on the walks above — the operator
raised `n` (J4 step 3), the operator authored more slots than they ever packed
keys for, and a record went inert unseen (I-4). In all three the refusal sends
them to obtain a second hardened account from a cosigner they do not need one
from, which costs a round trip to a third party and a re-flash.

A refusal that misdiagnoses is worse than a refusal that only counts: *"4 slots,
3 keys"* alone would leave the operator to look, whereas a confident wrong cause
stops them looking.

**Fix:** the count line is unconditional; the cause line is conditional. Emit the
C5 sentence only when a fingerprint the payload already holds appears in two
paths of the composed shape, and otherwise say which slots are unfilled.

### I-6 — "a seed may fill several slots" and "each source is used at most once" both decide the shortfall refusal, and §12.10 needs the reading §7d forbids

Two sentences of §7d, adjacent:

> seeds — … **a seed may fill several slots**, each at its own hardened account by
> ordinal among the slots that master fills …
>
> **Each source is used at most once in the composer** (C8 "remaining").

And the refusal that both feed: *"Seating is all-or-nothing: **fewer sources than
slots** → REFUSE."* One BIP-39 seed against a 2-of-3 is either one source (refused
before a single slot is offered) or three (seated at three accounts). C12 and §4f
say the latter; the sentence that governs the arithmetic says the former.

This is not academic. §12 item 10 is *"the `sortedmulti` preset with seed-derived
slots reproduces `gui/testdata/t6b_multisig_full.md1.txt`"* — the Multisig Build
parity gate, which is C7's whole migration story and is unsatisfiable under the
once-per-source reading. It also makes the counts the operator is shown
undefined: §7a's "Keys loaded: N" and §7b's "keys available: M" have no stated
value for a seed that can fill any number of slots.

**Fix:** state that "at most once" governs `key:` records and mk1 cards, that a
seed is a source of as many slots as the operator assigns it, and define the
shortfall test over *assignable slots* rather than *sources* — then say what
"keys available" counts when a seed is present.

### I-7 — the hashlock pick list has no specified row content, and a 64-hex digest cannot be drawn in a row

§6c: *"Primary: pick from the payload's `hash:` records."* Nothing about what a
row says. The reference wallet packs **three** digests, one per hashed tier, and
they differ only in 64 hex characters.

Measured: `ChoiceScreen.Draw` lays each choice out with `widget.Label`, which
**does not wrap** (`assertChoiceLabelFits`'s own error text says so), against a
budget of `480 − 2*16 − 2*buttonPadX` = **436 px**
(`gui/multisig_build_prose_test.go:508-519`, `buttonPadX = 6`). The longest
choice label anywhere in the shipped firmware is 17 characters
("Watch-only (keys)"); the seating pick list's own key rows are fine
("73c5da0a m/48'/0'/1'/2'", 23 characters). A 64-character hex string is roughly
three times the longest shipped label and is cut off the panel, not wrapped.

So at the moment the operator chooses which secret gates which tier, the rows are
either identically truncated prefixes or unreadable. Picking H2 for tier 1 makes
that tier unspendable by the preimage the operator holds — discovered at
recovery, which is the same failure mode the fixture README records months of.

The backstop is real and worth stating: §7e's consent shows each path's digest as
first-8 and last-8 hex, so a prepared operator can catch it before cutting. That
is what keeps this Important rather than Critical.

**Fix:** specify the row as index + first-8…last-8 (e.g. `hash 2  a7ef0ba4…725367f1`,
28 characters, inside the budget) in payload order, and say the order is the
host's pack order.

### I-8 — §7e's self-check refusal names no action, and §12 item 4 cannot construct it from an input

The fold added a good mechanism: *"the device asserts that the decoded shape
equals the composed path list and REFUSES to continue on mismatch (\"The policy on
this device does not match what you built.\")"*. §7g's table carries it as a row,
so it belongs to the refusal family §11 and §12 item 4 govern.

Two problems at that moment.

1. **§11's own rule is not met.** *"Every refusal in §4e, §6a, §6b, §6c, §7d and
   §7g **names what to do instead**"* — this line names nothing. It fires on the
   last screen before engraving, after the shape, the stub screen, seating and
   the mapping review, and the operator's next move is undefined. Every other
   refusal in the spec offers an exit; this one is a wall.
2. **The gate cannot be built as specified.** §12 item 4 asks, for each refusal,
   *"an input that must be refused and the exact line shown"*. This refusal
   cannot be provoked by an input: it fires only when the builder and the decoder
   disagree, which requires injecting a defect into the builder. Under the
   standing rule that a plan may not close while one of its own gates has never
   been run, this gate has no runnable construction.

**Fix:** give the line an exit ("Go back and check the path list, or start
again"), and move its acceptance out of the input-driven family into a
fault-injection or mutation test that flips one builder output and asserts the
refusal fires.

### I-9 — the copy the operator meets at the refusal moments is outside the gate §11 says covers it

§11: *"the copy of each refusal is a blockquote in §8 **or a quoted string in its
table**, so the glyph and modal-fits gates cover it."* The second half is false.

Measured. `scripts/plan-glyph-check.sh` recognises exactly two things — markdown
blockquotes not starting `> **`, and backticked spans of **40+** characters — and
prints its own blind spot: *"prose-embedded strings, line-fit, the Go source
itself"*. Run on this spec: **55 strings scanned, 0 undrawable**. A scan for
double-quoted strings of 18+ characters outside blockquotes and long backticked
spans finds **26** more, and the operator-facing ones include every §4e refusal
("Every wallet needs at least one path with a key.", "A path with only a time lock
means anyone can spend after it. Add a key or a hash.", "Taproot cannot hold a
key-less path. Use wsh, or add a key.", "Legacy wrappers hold one plain multisig
only. Use wsh or tr."), §7a's three door lines, §6b's echo forms, §7c's "This id
changed with the shape.", §7d's slot prompt and its refusal cause, and §7e's
self-check line.

These are the strings on the screens where a journey stops. They are also the
ones most likely to be authored with an em dash, because they were written in
prose. The glyph gate exists because *"a plan reviewed with an em dash in it
becomes code with an em dash in it"*, and F-185's raster measurement showed the
cost: the body did not draw at all.

Same for the fits gate — §12 item 5 enumerates §8 bodies, so a refusal that is
not a §8 blockquote is enumerated nowhere.

**Fix:** promote every operator-facing refusal string in §4e, §6a, §6b, §6c, §7a,
§7c, §7d and §7e into §8 blockquotes (the §4e/§7g tables then cite them by
letter), and re-run the gate. Cheap, mechanical, and it makes §11's sentence true.

## MINOR

- **M-1 — the door offers "From payload" for a payload that holds no policy.**
  §7a conditions the choice on *"only when a payload is loaded"*. J1's payload
  holds four `key:` records, three `hash:` records and a `now:` — no descriptor,
  no md1. The operator who reads "Keys loaded: 4" and picks "From payload"
  reaches a gather with nothing in it. Condition the row on the payload holding a
  Descriptor/MDMK record, or state what the empty case says.
- **M-2 — §8j's copy is false in the case the spec routes the operator through,
  and "after seating began" is undefined.** The shortfall refusal fires *at the
  transition*, before any slot is offered, and offers "Back-to-edit". If §8j then
  says *"Every key you seated will be cleared"* with zero keys seated, it is a
  confirm that misdescribes the state; if it does not fire, the spec does not say
  so. Define "seating began" as "at least one slot assigned" and suppress the
  confirm otherwise.
- **M-3 — preset availability per wrapper is unspecified.** §4a admits `sh`/`sh(wsh)`
  for *one* unlocked, unhashed n ≥ 2 path only; four of the five presets are
  multi-path. §7b's order is wrapper → preset, so a legacy-wrapper operator can
  populate a shape that §4e refuses only at the end, although §4e elsewhere
  suppresses illegal values *at the picker* ("the picker does not offer the
  value"). Say which presets each wrapper offers.
- **M-4 — the seating prompt has no defined form for the extracted internal-key
  slot.** §7d's prompt is *"Slot @2, Path 1 key 2 of 3: choose a key"*. Under `tr`
  the internal key is `@0` and is *"then not a leaf"* (§5), so it has no "key i of
  n" within a path, while the operator still thinks of it as the path they typed.
  One sentence fixes it ("Slot @0, key-path (from Path 2): choose a key") and it
  is also the natural place to foreshadow §7e's "A KEY CAN SPEND ALONE".
- **M-5 — "This id changed with the shape." names the change, not the
  consequence.** §7c's whole purpose (C9) is that the operator mints cards on the
  host from that stub. `seatKeyCards` layer 1 refuses any card whose stubs do not
  include *this* template's (`gui/key_card_seating.go:66-73`), so cards already
  minted against the old stub will not seat. One clause — "Cards minted with the
  old stub will not seat into this template" — closes the loop the fold opened.
- **M-6 — §8b warns about a cost §5's declarations have removed, and stays silent
  in the case where key order really is part of the wallet.** Its body is *"Key
  order is part of this wallet. Restoring a key-less template of it needs the key
  order or a permutation search."* But §5's declarations row now has **every**
  slot declare an origin and, when seated, a fingerprint, and §4f gives unseated
  slots distinct accounts — so restore matches by declaration
  (`slotMatchesCard`) and no permutation search arises for anything this composer
  engraves. Meanwhile J2's tiers 1 and 2 are forced to `multi` because they are
  non-sole, and §5a says the confirm *"fires ONLY where sorted was legal and
  declined"* — so the operator gets the property and not the warning. Recalibrate
  the copy to the cost that actually remains (a third-party restore from the
  descriptor text, not from these plates).

## NIT

- **N-1 — three new confirm-to-proceed modals with no stated confirm gesture.**
  §8a, §8b and §8j are unskippable confirms; §8j ends "Continue?". The shipped
  precedent ends its body with a literal `"\n\nHold button to confirm."`
  (`multisigBuildExperimentalWarningBody`). On a machine with this button set the
  gesture is worth spelling, and it is 24 characters against the budgets measured
  in I-2.
- **N-2 — §8c is five different echoes in one blockquote, so "every §8 body
  passes the modal-fits assertion" measures a string no screen shows.** The
  relative echo, the height echo, the date echo, the bound lines and the no-bound
  line never appear together. Split them, or say §8c is a catalogue rather than a
  body.

---

# RECORDED AND NOT ACTIONED

**NOT OUR CONCERN**
1. `hash:` and `key:` records carry no human label, so the pick lists identify by
   digest and by fingerprint + origin. Round 0 settled this for keys (no label
   field exists in the constellation's wire); it extends to digests. I-7 asks
   only for the row's *rendering*, not for a label field.
2. The composed artifact leaves the device only on metal — no on-screen QR (§14),
   no NFC (C8), no camera (by design). Round 0 item 3, unchanged.
3. A QR plate this device cuts can never be read back by it. §7f states the
   read-back-integrity difference by form; the absence of a device-side read-back
   is the machine, not the spec.
4. The one-row pick list for the last remaining source. Round 0 item 2.

**DOCUMENTATION ONLY**
5. Picking a preset under a wrapper the toolkit golden was never spelled for
   (`tiered-recovery` under `tr`). §4d's generalisation and §8d's line already say
   a wallet built here is its own wallet with its own id and addresses; the
   preset case adds nothing new. (M-3 is a different point — a preset the wrapper
   structurally cannot hold.)
6. Origin/wrapper mismatch on a seated card (a `.../2'` key under `tr`). §4f, C28,
   round 0 item 5 — unchanged.
7. The brief's premise that the reference wallet has a keyless tier 4 is stale;
   it was keyed on 2026-08-22 and the fixture now carries three hashlocks and
   seven keys. §4e's `tr` refusal and the wsh fallback were walked anyway, per
   the brief, and the refusal reads correctly.
8. §7d's "the consume path's rules coexist" — `seatKeyCards`'s *"ONE CARD MAY
   FILL SEVERAL SLOTS"* (`gui/key_card_seating.go:28-30`) against the composer's
   "remaining" rule. The fold's separation is explicit and correct; the residual
   ambiguity is only about seeds, which is I-6.

**VERIFIED CLOSED THIS ROUND — measured, do not re-derive**
9. **Every §8 body fits with large headroom.** Non-whitespace sizes: 8a 146,
   8b 137, 8c 184, 8d 101, 8f 143, 8g 104, 8h 131, 8i 132, 8j 98, 8k 76, 8l 133 —
   against 588 drawn in full, ~500 at F-185's real cut, and an 80-character
   margin. Round 0's I-4 is answered for §8; the residual exposure is the *§7c
   screen* (I-2) and the *consent surface* (I-3), neither of which is a §8 body.
10. **The door's key-state line fits the shipped `Lead` precedent.** §7a's longest
    is 69 characters ("A payload is in flash but not loaded. Load it from the
    carousel first."); two shipped `ChoiceScreen{Lead:}` strings are 100
    characters, one is 71. `Lead` uses `widget.Labelw`, which wraps.
11. **Round 0's C-3 is closed.** `slotMatchesCard` compares the card's parsed path
    against the slot's declared origin component by component **and** the
    fingerprint when the template declares one; §5's declarations row now has
    seated slots declare the fingerprint and §4f gives unseated slots distinct
    accounts. J1's two `73c5da0a` keys at different accounts seat cleanly; J2's
    seven same-origin keys seat on distinct fingerprints.
12. **§8c's arithmetic is correct**, all four claims: 90 days → ceil(7,776,000/512)
    = 15,188 units → 90.003 days; 2026-09-01 → 2027-03-01 is 181 days;
    1985-11-05 00:00:00 UTC is 499,996,800; 500,000,000 is 1985-11-05 00:53:20 UTC.
13. **`plan-glyph-check.sh` on the spec: 55 strings scanned, 0 undrawable.** What
    it did not scan is I-9.
14. Round 0's I-1 (engrave-form collapse and `key:`-sourced cards), I-2 (the
    no-bound line), I-3 (the door's key state), I-5 (a paged pick list), I-6 (the
    eight-digit date field), I-7 (§8l), C-2 (a new consent surface), C-4 (§8i),
    C-5 for path-list edits (§8j), and C-6 (unseated origins) are all answered by
    the fold as written. Only C-5's wrapper leg (C-1 above) and C-1's stub-screen
    leg (I-1 above) survive.

**NOT MEASURED — plan-time render checks, not review questions**
15. The exact per-frame capacity of the §7c screen and of the composer's consent
    surface at 480x320. No Go toolchain on this shell; the character arithmetic
    in I-2 is an estimate against `modal_fits_test.go`'s measured numbers, and
    wrapping differs between hex-heavy short lines and prose. The *conclusion* —
    an unbounded body with no stated widget — does not depend on the estimate.
16. Whether `widget.Label` renders 64 hex characters at 436 px as a hard cut or a
    clip. I-7's point is that it does not wrap; the exact truncation is a render
    check.
