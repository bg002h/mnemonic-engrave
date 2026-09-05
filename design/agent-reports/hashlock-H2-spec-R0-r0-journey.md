# R0 round 0 — journey + adversarial lens on `SPEC_hashlock_H2_device.md`

**Artifact:** `design/SPEC_hashlock_H2_device.md` at engrave `bfd042e`.
**Sources read:** `design/BRAINSTORM_hashlock_phrase.md` §3.7, §4.4, §4.5, §4.6, §5, §7/§7.1;
`design/SPEC_wallet_policy_composer.md` §6b, §6c, §7a, §8g–§8l; `mnemonic-secret/design/SPEC_ms_hashlock.md`
§4.3, §4.4, §7 (at ms `fb98d73`); the fork at `/scratch/code/shibboleth/seedhammer` main `c4a64fc`
(`gui/composer_hash.go`, `gui/composer_shape.go`, `gui/composer_consent.go`, `gui/composer_state.go`,
`gui/composer_copy.go`, `gui/composer_paged.go`, `gui/unlock_kdf.go`, `gui/codex32_polish.go`,
`gui/unlock_session.go`, `gui/modal_fits_test.go`, `cmd/emu/walk_h0_preimage.js`).

**Scope honoured:** read-only everywhere; nothing committed; no scratch copies; no sub-agents; no
`.jsonl` read. Citations, the port's constants and the §2 phrase-rule text were NOT re-reviewed —
the fidelity lens owns them. Where I cite a line of fork code it is to establish what the operator
meets, not to audit the spec's citation table.

**Machine-checked while walking** (so no finding below rests on a description):

- `composerHashEdit` (`gui/composer_hash.go:140-172`) returns `false` on Back at the pick screen and
  on Back in `composerHexEntry`; `composer_shape.go:269` turns that `false` into
  `st.list.Paths = st.list.Paths[:idx]` — the path is **deleted**. `composer_shape.go:346` ignores the
  return value.
- `composerPathLine` (`gui/composer_state.go:254`) renders a hashed path as `hash only` / `… + hash`.
  It never prints a digest or a method.
- `composerPickScreen` (`gui/composer_paged.go:259`) opens with `sel := 0` and receives `rows` only —
  it has no notion of a current value, so nothing marks the hash already on the path.
- `composerEveryPathHashed` (`gui/composer_state.go:239`) walks `list.Paths` — i.e. the paths that
  exist *so far*. §8h fires today at Done (`composer_shape.go:443`), not per path.
- `composerDigestShort` (`gui/composer_consent.go:61`) is `first8..last8`; consent prints
  `hash <short>` per branch (`:94-96`) and repeats §8i when any branch has a digest (`:193`).
- The modal gate (`gui/modal_fits_test.go`) measures capacity at **588 normalized characters** and
  requires **80** of headroom, so the effective budget is **~508**. I measured §4.5's stack
  (whitespace-stripped, lowercased): digest line 22 + method line 15 + §8i 132 + §4.5's reuse copy 121
  = **290**; with §8h added = **421**; with the brainstorm's *full* reuse line (194) instead = **494**.
- `unlockDerive` (`gui/unlock_kdf.go:242`) polls `backBtn` between KDF slices, so Back **does** work
  during the countdown. `unlockKDFLead` (`:212`) returns the fixed string
  `"Unlocking. This takes about 30 seconds."` while `done <= 0` — the first frame.
- H0's preimage refusals read `"This record is a hashlock preimage, not a seed. It is not engraved as
  one."` (`gui/codex32_polish.go:233`, `gui/unlock_session.go:198`) and offer no onward route.
- `composerState` (`gui/composer_state.go:26`) is in memory only; nothing in `composer_flow.go` or
  `composer_state.go` persists it.

---

## Journey 1 — The happy path, twice

**Step 1. `Which hash?`.** *In hand:* a policy under construction; possibly a payload with `hash:`
records. *Device:* rows = payload digests, `Type a hashlock phrase`, `Type 64 hex`, `No hash lock`;
with no records, the lead adds "No hash record in the payload. ms hashlock on the host makes one."
*What else:* the operator reads a lead that sends them to the host while the row beneath it does the
job on the device. → **divergence: documentation** (M-1).

**Step 2. Tap `Type a hashlock phrase`.** *Device:* per §5 the operator is "taking a hash", so the
§8i modal fires: *"…A hash of the passphrase itself can never be spent."* *What else:* an operator
who has just chosen the phrase route reads a sentence about phrase-hashing being unspendable and may
conclude they chose the wrong row. Nothing says the device performs the two-step for them.
→ **divergence: documentation** (N-1).

**Step 3. Type six diceware words.** *Device:* `Hashlock phrase`, `n/100`. *What else:* nothing here
warns about reuse or strength — every such warning is downstream (I-2).

**Step 4. Method pick → Hardened.** *Device:* the row says "about 10 s"; no modal at ≥ 20 characters.

**Step 5. `Deriving`.** *Device:* countdown; Back abandons. *What else:* the first frame inherits
`unlockKDFLead`'s zero-state string, "about 30 seconds", against a row that promised 10.
→ **divergence: default** (M-3).

**Step 6. The confirm modal.** *In hand:* `hash 3cf5d421..b70a4c12`, `method: hardened`, §8i again,
the reuse lines, and — because path 1 is at this moment the only path and it is hashed — §8h.
*What else, and this is the journey's whole point:* the operator presses CONTINUE and the device
drops the preimage and the phrase. **Nothing on this screen tells them to write the phrase and the
method down.** The one line that speaks of backup is §8h — "It is not on this device and not on these
plates. Back the preimage up separately." — and on this route **there is no preimage to back up**:
the device never showed it, and §9 forbids storing, showing or engraving it. The instruction cannot
be followed. → **divergence: refusal/copy — worse than silence** (C-1).

**Step 7. Later, on the host: `ms hashlock`.** *In hand:* stdout `hash:<64 hex>`, `--out` preimage
ms1, and the stderr card carrying the method line, the character count, the §8i/F-132 lines and the
reuse lines. **The card is the artifact the device leg does not have.** The operator who used the
device route holds plates carrying `sha256(H)` and their memory. If the phrase or the method is lost,
that path is unspendable; nothing on the device ever said so.

*Not a finding, but the journey's honest ending:* the preimage plate produced by `ms hashlock --out`
cannot be cut on this machine — H0 refuses it at the engrave choke point. §9 says engraving is out of
scope, so this is **ruled, not a defect**; it is however exactly why C-1 matters: with the device
route, **the phrase is the only backup that exists.**

---

## Journey 2 — The wrong order (device first, host weeks later)

**In hand weeks later:** a phrase in the operator's memory or notebook, and plates carrying a digest.

**Divergence a — different method.** `ms hashlock` defaults to `--method hardened`. If the device run
used SHA-256, the host prints a different digest and a valid-looking record. *Could the device have
prevented it?* Only by making the method durable. It is shown once, in the confirm modal, and stored
nowhere: `st.list.Paths[idx].Hash` is a `[32]byte` and `composerPathLine` never prints a method.
→ **refusal impossible; copy is the only defence** (C-1).

**Divergence b — capitalisation.** §2 pins bytes verbatim, and the corpus row
`Correct Horse Battery Staple` is the lockstep proof. Neither side can detect it; the digests simply
differ. → **not our concern** (the rule is right; detection is impossible by construction).

**Divergence c — a trailing space in the host's file.** ms §4.3's byte-verbatim reader strips exactly
one `\r?\n` and nothing else, so the space survives and X differs. The brainstorm named the character
count as "the one signal that shows a stray space" and put it on the host card — but the device's
**confirm modal carries no count**, only `n/100` on a screen that is gone by then. An operator
reconciling "the card says 30 characters" against the device has nothing to compare.
→ **divergence: default — cheap to fix** (M-5).

**What actually catches all three:** comparing the host's `hash:` line against the digest on the
plate. The spec never tells the operator to do that before funding (C-2's second half).

---

## Journey 3 — The mismatch they never see

**Step 1.** A payload with a `hash:` record is loaded; the operator nonetheless takes
`Type a hashlock phrase` (it is the first typing row, and friendlier than 64 hex).

**Step 2.** They type the phrase and pick **Hardened** — the first method row, the safer-sounding
one — while the host record was made with `--method sha256` (the brainstorm's own M-3 records sha256
as "the only documented recipe" before this tool existed).

**Step 3. The confirm modal** shows `hash <first8>..<last8>` and `method: hardened`. *What else:*
nothing on this screen mentions the payload's digest. The two digests differ, both are well-formed,
and the modal is the same modal in either case. **The device knows both values and compares neither.**

**Step 4. `Which hash?` re-entered.** `composerPickScreen` starts at row 0 and receives only labels —
no marker, no "current". The path row reads `Path 1: hash only`. So between the confirm modal and the
consent screen there is **no surface on which the set digest is visible at all**.

**Step 5. Consent** does print `hash <first8>..<last8>` per path — the one place a diligent operator
could compare against the host card. Nothing points them at that comparison, and §8l's remedy
(restore in a coordinator, compare the first receive address) **cannot** catch it: the address is
derived from the policy the device built, digest included, so it matches the wrong digest perfectly.

→ **divergence: warning — worse than silence.** The operator ends holding a preimage plate that does
not open their wallet, and every check the device offers agrees with itself. (C-2)

*The same shape without a payload record:* device first, host later, wrong method or a mistyped
phrase — discovered at spend time.

---

## Journey 4 — Back, power, and the countdown

**Back during derivation.** Works: `unlockDerive` polls Button1 between slices. Nothing was assigned.
→ **default, correct.**

**Power loss during derivation.** §4.4 says "power loss likewise". Read plainly that says the
composer state is untouched. It is not: `composerState` lives only in RAM, so a power loss ends the
**whole composition** — shape, seats, locks, everything. The sentence reassures about the one thing
that was never at risk and is silent about the thing that is. → **documentation** (M-4).

**Back on the confirm modal.** §4.5: "Back discards (nothing was assigned before CONTINUE)". §1.2:
"Back at any step returns to `Which hash?` with the path unchanged." These are two different
statements, and the shipped sibling resolves them the dangerous way: `composerHexEntry`'s Back makes
`composerHashEdit` return `false`, and at **path creation** `composer_shape.go:269` deletes the path.
An operator who typed a phrase, waited 10 s, read a digest and pressed Back could lose the whole path
they had just confirmed as EXPERIMENTAL — with no modal. §7.2's test ("Back at each step leaves
`Hash` unchanged") is satisfied in the *edit* context while the creation context loses the path: a
test that passes over the defect. → **refusal/state** (I-4).

**Back on the method modal (declining a warning).** Unspecified. If a decline unwinds to
`Which hash?`, the operator loses a phrase they touch-typed on a four-button device — so heeding the
warning costs more than ignoring it, which is how warnings get trained away. The warning's whole
purpose (steer sha256 → hardened) needs the phrase to survive. → **refusal/default, spec silent**
(I-5).

**Re-entering `Which hash?`.** Shows nothing about the current state (Journey 3, step 4).

---

## Journey 5 — Reuse

**In hand:** a person who already has a phrase they like — a password-manager master phrase, a BIP-39
passphrase.

**Device:** the phrase screen asks for it with no warning. The reuse lines appear in the **confirm
modal**, after the phrase has been typed, after the method has been chosen, after a 10-second wait.

**What else:** by the time the copy says "Never use this phrase as a passphrase or a password
anywhere else", the operator has already typed one that they do. The copy states a rule and names no
action. The action exists — Back, choose another phrase — and is not named, and its destination is
unspecified (I-5). The same placement problem afflicts both method modals: "use six diceware words"
arrives after the words were typed.

**Is §3.7 strong enough in it?** §4.5's version stops at "anywhere else." and drops the brainstorm's
and ms §7's tail: "— a spend publishes the preimage, and anyone can then test guesses at the phrase
itself." That tail is the *mechanism*, i.e. the only part that explains why the rule is not
superstition. Measured, restoring it costs 73 normalized characters and the all-hashed modal still
lands at 494 against an effective budget of ~508 — it fits, but by ~14 characters, wrap permitting.
→ **copy/geometry** (I-2, M-2).

---

## Journey 6 — The short phrase

**`hunter2`, 7 characters, Hardened.** The modal reads: *"A 20-character phrase falls in about 72
days on one GPU. Choose it from a generator. Continue?"*

*What the operator has in hand:* a 7-character phrase from a wordlist. *What the copy tells them:*
the cost of breaking a **20-character** phrase. A tired reader takes "72 days" as the answer to "how
strong is mine?" — and for `hunter2` the true answer is "immediately, it is in every list". The
sentence is a *bound at the threshold* and reads as an *estimate for the phrase in hand*, understating
by many orders of magnitude at the exact moment of decision. → **warning that misleads** (I-3).

**`hunter2`, SHA-256.** The brainwallet modal is honest and names the remedy — but arrives after the
typing (Journey 5).

**Should there be a length at which the device refuses?** No: L12 is an operator ruling ("both
warnings warn, never refuse") and this lens does not reopen it. The fix is to make the consequence
concrete, not to refuse: one word ("Even a 20-character phrase falls in about 72 days on one GPU.
Shorter ones fall sooner.") does the whole job.

**Noted, not filed:** the hardened warning is length-gated at 20, so a 20-character human-chosen
phrase gets no strength warning at all. That is brainstorm §5's ruled default ("Neither floor can see
a dictionary phrase, so the copy is the defence"), and "Even …" partially covers it.
→ **default, ruled.**

---

## Journey 7 — The typed-plate temptation

**In hand:** a preimage plate from `ms hashlock --out` — 75 characters of codex32, `ms10hash…`.

**Route A — `Type a hashlock phrase`.** §2 rule 3 refuses: *"That is an ms1 string, not a phrase.
Load it from the payload instead."* **The remedy named is impossible.** L22/H0 made a kind-`0x03`
single inert on both classifiers, so a preimage ms1 sitting in a payload reaches no screen; the
spec's own parenthetical in the same clause says "there is no device route for a preimage plate this
cycle". The operator who obeys the copy repacks a payload, loads it, finds nothing on `Which hash?`
(its only signal is the door's not-understood count), and is back where they started having lost a
host round trip. → **refusal whose remedy does not exist — worse than silence** (I-1).

**Route B — `Type 64 hex`.** They cannot: the plate is codex32, not hex, and the pad offers hex only.
The device is silent, correctly. → **not our concern.**

**Route C — `Backup Wallet` → `M*1 STRING`.** H0 refuses: "This record is a hashlock preimage, not a
seed. It is not engraved as one." Correct and terminal, with no onward route named. → **default**
(H0's, shipped).

**Is there any route that gets the plate's digest onto the device?** Yes, but only through the host:
`ms hashlock <ms1>` prints `hash:<digest>`, `me sysw pack` puts it in a payload, and it becomes a
payload row. **That is the sentence route A should be printing** and nowhere in the spec is it
written down. The spec is honest in its own prose (§1.4, §6, §9) and dishonest in the one string the
operator actually reads.

---

## Journey 8 — The all-hashed policy

**Step 1.** Every path carries a hash, all set by phrase.

**Step 2. Each confirm modal** carries §8h, because at path creation `composerEveryPathHashed` sees
only the paths built so far. On the **first** path of any multi-path build, one hashed path means
"every path is hashed" and §8h fires — then the operator adds a keyed path and it is false. A
funds-critical banner that is wrong most of the times it appears is a banner that gets dismissed.
→ **warning at the wrong moment** (I-6).

**Step 3. At Done**, §8h fires again, this time truly. Its text: "Every way to spend this wallet
needs the preimage of a hash. It is not on this device and not on these plates. Back the preimage up
separately." *What does the operator have to back up?* On the phrase route: nothing. There is no
preimage artifact. What they must keep is **the phrase and the method**, and §8h names neither.
→ **copy that misdirects at the highest-stakes moment** (C-1).

**Step 4. Consent** repeats §8i and prints each `hash <short>`; nothing names the method, and nothing
survives the session that ties a digest to the phrase that made it.

---

# Findings

### C-1 — The device asks for the only secret it will ever forget, and never says to write it down; §8h's remedy is unfollowable on this route

**Where:** §4.5 (the confirm modal), §3 ("the preimage … is dropped after CONTINUE or Back"), §9;
composer §8h reused unchanged.

Every other secret this device handles ends on metal: seeds, shares, BIP-39 passphrases. The phrase
route is the first flow that takes a secret, uses it, and deliberately forgets it — and the confirm
modal, the last screen before the digest becomes part of a policy, says nothing about that. The only
line in the whole composer that speaks of backing a hashlock up is §8h, which (a) fires only when
*every* path is hashed, so the common mixed policy (2-of-3 keys + a hashed recovery path) gets
nothing at all, and (b) instructs the operator to back up **the preimage**, an artifact this route
never produces and §9 forbids the device from producing. An operator who follows it literally goes
looking for something they cannot obtain (Journey 7), and one who does not follow it holds plates
that carry `sha256(H)` and a memory. Journeys 1, 2 and 8 all end here, and the ending is unspendable
funds.

**SUGGESTION.** Make the confirm modal always carry a backup line, and pay for it from copy that is
already redundant on this route — §8i is shown at the pick (§5), again in this modal, and again at
consent, and on the phrase route the device *performs* the two-step §8i describes. Drop §8i from this
modal and spend the 132 characters on, e.g.:

> Write down this phrase and the method now. They are
> not on this device and not on your plates. Without
> both, this path can never be spent.

and add a phrase-route form of §8h that names the phrase and the method rather than "the preimage".

---

### C-2 — Nothing ever compares the derived digest with the preimage the operator actually holds

**Where:** §4.5 (the confirm modal), §4.1 (`Which hash?`), §8 (acceptance).

The device holds the payload's `hash:` digests and the digest it has just derived, in the same
function, and compares them at no point. Journey 3's operator sets `hardened` on a policy whose
host-made record was `sha256`; both digests are well-formed, the confirm modal reads identically
either way, `Which hash?` shows no current value, the path row says `hash only`, and §8l's coordinator
check *agrees* with the wrong digest because the address is derived from it. The same shape reaches
the same end with no payload at all: device first, host weeks later, wrong method (the host defaults
to `hardened`) or a stray character — discovered at spend time. §8 makes the operator reconcile the
device against the host **for the anchor phrase during acceptance**, which is exactly the wrong
phrase to reconcile: their real one is never checked.

**SUGGESTION.** Two lines, both cheap:

1. In the confirm modal, when the payload holds `hash:` records, state the relation:
   `matches hash 1 in the payload` or `no hash: record in the payload has this digest`. Omit the line
   entirely when there are no records, so it never fires as noise.
2. Add to the confirm modal (or to §8h's phrase form): *"Before you fund this wallet, run ms hashlock
   with this phrase and method and check the digest matches."* That converts an unspendable path into
   a five-minute check the operator can actually perform.

---

### I-1 — The ms1 refusal sends the operator to a route that does not exist

**Where:** §2 rule 3.

The refusal reads *"That is an ms1 string, not a phrase. Load it from the payload instead."* A
kind-`0x03` single in a payload is inert by L22/H0 — it reaches no screen — and the spec says so two
clauses later. The operator who obeys the string loses a host round trip and learns nothing. The real
route (`ms hashlock <ms1>` → `hash:` record → `me sysw pack` → the payload row) is nowhere in the
spec's operator-facing copy.

**SUGGESTION.** *"That is a preimage plate, not a phrase. Run ms hashlock on the host with it and
load the hash: record it prints."* And state in §6 that this string is the only device-side signpost
to the host route, so it does not drift when H3 writes the manual.

---

### I-2 — Every warning that could change the phrase fires after the phrase is typed, and none names the remedy

**Where:** §4.2 (the phrase screen), §4.3 (both method modals), §4.5 (the reuse lines).

The ruled order (method after phrase) is not in question. What is missing is that the reuse rule —
which brainstorm §3.7 calls "the whole defence", the tool being unable to detect reuse — reaches the
operator only *after* they have typed a phrase they may already use as a password, and it names no
action. Same for "use six diceware words", which arrives after the words are typed. A rule with no
verb, delivered after the fact, is a rule the operator can only regret.

**SUGGESTION.** (a) Put one line on the phrase screen's lead, before the keyboard: *"Use a phrase you
have never used anywhere else."* (b) Give the reuse lines and both method modals a verb: *"If you
have used this phrase anywhere else, press Back and choose another."* (c) Fix I-5 so that pressing
Back actually preserves the work.

---

### I-3 — The hardened warning quotes the cost of a 20-character phrase to someone who typed seven

**Where:** §4.3 row 1.

*"A 20-character phrase falls in about 72 days on one GPU"* is a statement about the threshold, shown
only to operators **below** it. For `hunter2` the real cost is instant. As written, the sentence
reads as an estimate for the phrase in hand and reassures at the exact decision point — the opposite
of what brainstorm §5 ruled the copy is for.

**SUGGESTION.** One word carries it: *"Even a 20-character phrase falls in about 72 days on one GPU,
and shorter ones fall sooner. Choose it from a generator. Continue?"* — 39 characters, well inside the
budget, and it also partly covers the phrases just above the 20-character floor that get no warning
at all.

---

### I-4 — Back's contract is stated two ways, and the shipped resolution deletes the path

**Where:** §1.2 ("Back at any step returns to `Which hash?` with the path unchanged") vs §4.5 ("Back
discards"); §7.2's Back test.

`composerHashEdit` signals "the operator left" by returning `false`, and at path creation
(`composer_shape.go:269`) `false` **removes the path**. That is what `composerHexEntry`'s Back does
today. If the phrase route copies its sibling, an operator who typed a phrase, waited ten seconds,
read a digest they did not expect and pressed Back loses the whole path — including the EXPERIMENTAL
key-less confirm they already gave — with no modal. §1.2's wording (return to `Which hash?`) is the
safe behaviour but requires a loop inside `composerHashEdit` that no section describes, and §7.2's
assertion ("Back at each step leaves `Hash` unchanged") is satisfied in the *edit* entry point while
the creation entry point loses the path: a test that passes over the defect.

**SUGGESTION.** State the contract once, normatively: Back from the phrase screen, the method pick,
the derivation and the confirm modal returns to `Which hash?` and `composerHashEdit` does **not**
return `false` for any of them; only Back at `Which hash?` itself returns `false`. Then require
§7.2's Back tests to run through the **creation** entry point (`composerAddPath`) and assert the path
still exists, not only that `Hash` is unchanged.

---

### I-5 — A declined method warning has no specified destination

**Where:** §4.3.

Both modals are confirm-to-proceed; the spec never says where a decline goes. If it unwinds past the
method pick, the phrase — typed a character at a time on a four-button device — is gone, so the
operator who *heeds* the brainwallet warning is punished and the one who dismisses it is not. The
warning's purpose is to move them from SHA-256 to Hardened, which requires the phrase to survive.

**SUGGESTION.** Normative: declining either method modal returns to the **method pick** with the
phrase intact; Back at the method pick returns to the phrase screen with the phrase intact; Back
there returns to `Which hash?` and drops it. Add a §7.2 row: decline SHA-256, choose Hardened, and the
resulting `Hash` equals the corpus's `hardened_h` for the phrase typed **once**.

---

### I-6 — §8h inside the per-path confirm modal is evaluated on a partial policy

**Where:** §4.5 ("then the F-132 line the composer already prints when every path is hashed (§8h)").

`composerEveryPathHashed` walks the paths that exist at the moment it is called. Inside the confirm
modal at path creation, that is the path being created — so on the **first** path of every multi-path
build a hashed path means "HASH ON EVERY PATH", and it is false as soon as a keyed path is added.
§8h already fires at Done (`composer_shape.go:443`) with the whole policy in view, which is the only
place the predicate is true. Adding a whole-policy banner to a per-path modal produces a warning that
is wrong most times it appears — and it is the same string the operator must take seriously at Done.

**SUGGESTION.** Drop §8h from the confirm modal and keep it at Done, where it is correct; or gate it
explicitly on the shape being complete. If it is kept, it also frees 131 characters for C-1's line.

---

### M-1 — `Which hash?`'s no-payload lead still routes the operator to the host

**Where:** §4.1.

"No hash record in the payload. ms hashlock on the host makes one." was written when the host was the
only maker. The row immediately beneath it now makes one on the device.

**SUGGESTION.** *"No hash record in the payload. Type a phrase below, or make one with ms hashlock on
the host."*

---

### M-2 — §4.5 becomes the composer's largest modal body, and the spec asserts it fits without a number

**Where:** §4.5, §7.2 ("Geometry: the confirm modal … fit").

Measured against the gate's own constants (588 capacity, 80 margin → ~508 effective, normalized): the
stack is 290 without §8h, **421** with it, and **494** if the brainstorm's full reuse line is restored
in place of §4.5's truncated one. The last is inside the budget by ~14 characters — and capacity
depends on wrap, which the digest's 18-character unbreakable token does not help. The gate's own error
text says a funds-critical modal must not require scrolling. The spec should not leave the trim order
to whoever first sees the test go red.

**SUGGESTION.** State the drop order in §4.5 (§8i first — see C-1 — then §8h — see I-6), and require
§7.2 to assert the **all-hashed, longest** variant, not the plain one.

---

### M-3 — `Deriving`'s first frame inherits "about 30 seconds" against a row that promised 10

**Where:** §3, §4.4.

`unlockKDFLead` returns the fixed string "Unlocking. This takes about 30 seconds." until the first
slice completes. Retitling the screen does not change that body, and 30 seconds is the sealed
payload's number, not this one's.

**SUGGESTION.** §4.4 names the zero-state lead too: *"Deriving. This takes about 10 seconds."*

---

### M-4 — "power loss likewise" reassures about the wrong thing

**Where:** §4.4.

`composerState` is in RAM and nothing persists it, so a power loss during the derivation ends the
whole composition, not just the un-assigned hash. The sentence is true about the digest and reads as
true about the session.

**SUGGESTION.** *"Back during the derivation abandons it and nothing is assigned. A power loss ends
the composition, as it does at any other point in this flow."*

---

### M-5 — The character count, which the brainstorm calls the one signal that shows a stray space, is on the host card and not on the device's confirm modal

**Where:** §4.5.

`n/100` exists only on the entry screen. The confirm modal is the device's durable surface — the one
the operator is (per C-1) meant to copy down — and it carries no count, so a later reconciliation
against the host card's `phrase_chars` has nothing to compare.

**SUGGESTION.** Make the modal's second line `method: hardened   chars: 29` — about 12 characters, the
brainstorm's own M-2 rationale applied to the side that keeps no card.

---

### N-1 — §8i is shown three times on the phrase route and never says the device does the two-step

**Where:** §5 (fires at the pick), §4.5 (in the confirm modal), composer §7e (again at consent).

"A hash of the passphrase itself can never be spent", read immediately after tapping
`Type a hashlock phrase`, describes the mistake this row exists to prevent — and nothing says the row
performs the safe construction. See C-1 for the proposal to spend those characters better.

---

## Counts

| severity | count | ids |
| --- | --- | --- |
| Critical | 2 | C-1, C-2 |
| Important | 6 | I-1, I-2, I-3, I-4, I-5, I-6 |
| Minor | 5 | M-1, M-2, M-3, M-4, M-5 |
| Nit | 1 | N-1 |

**Headline.** The spec builds a careful, lockstep-correct way to derive a digest from a phrase and
then never tells the operator that the phrase is now the only key to that path: the confirm modal
carries no backup instruction, and the one line that mentions backup (§8h) fires only on all-hashed
policies and names an artifact this route cannot produce.

**Explicitly not reviewed:** citations, the port's constants, the §2 phrase-rule text (fidelity
lens); L12's warn-never-refuse ruling and the method-after-phrase order (operator rulings, not
reopened); the engraving of preimage plates (§9, ruled out of scope).
