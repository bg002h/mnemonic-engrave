# R0 round 2 — adversarial review, LENS: FUNDS SAFETY BY COUNTEREXAMPLE, scoped to the ROUND-0/ROUND-1 FOLDS

**Artifact:** `design/SPEC_wallet_policy_composer.md` at commit `99463ac`
(verified unchanged from `99463ac` to the working tree: `git diff --stat
99463ac..HEAD -- design/SPEC_wallet_policy_composer.md` is empty).
**Reviewer:** independent adversarial agent, read-only. No repo file modified but this one.
**Scope:** ONLY the mechanisms the round-0 and round-1 folds ADDED. The round-0
adversarial report's findings and its "attacks that failed" list are treated as
settled; nothing below re-raises one. Where a finding sits downstream of a folded
finding, the fold's own replacement text is what is attacked, and the relationship
is named.
**Measured against:** `~/.cargo/bin/md` 0.14.0, `~/.cargo/bin/mk` 0.13.0,
`target/release/me` 0.7.0, fork worktree `/scratch/code/shibboleth/seedhammer`
(no Go toolchain on this box — device traces are step-by-step through the shipped
source, with host-side command output where a tool can reach the same object).

## VERDICT: 1 Critical / 7 Important / 5 Minor / 1 Nit — NOT GREEN

---

# CRITICAL

## C-1. §4f's unseated-slot origins collide with SEATED slots' declared origins, and one card then silently fills BOTH slots — the mixed template is §8p's own escape hatch, and §12 item 6's acceptance is worded so it cannot catch it

**Spec defeated:** §4f ("**Unseated slots … declare the §4f origin for the wrapper
with `account' = the slot's emitted index`, and no fingerprint.** … identical
origins with no fingerprints are unseatable at restore (`errSeatSlotContested`),
so distinct accounts by slot index are the one form that both decodes and
seats"); §5 declarations row ("a keyless template is engraved WITH fingerprints
for seated slots and with distinct-account origins for unseated ones"); §7d/§8p
("then Back-to-edit or 'engrave as a keyless template'"); §12 item 6 ("a template
with two same-origin slots and no fingerprints is never produced").

This is NOT r0-adversarial C-3 or r0-journey C-3 (both: *the keyless template
never declares fingerprints, so seating refuses it*). Those are folded. The
defect is in the FOLD: the mixed form the fold introduced — fingerprints for
seated slots, index-derived accounts for unseated ones — is silently
MIS-SEATED, not refused.

### Constructed input

Host payload, three records (bodies are the hex of these exact strings; `md
decompose --emit keys` prints this form verbatim — verified):

```
key:  [73c5da0a/48'/0'/1'/2']xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk
key:  [73c5da0a/48'/0'/2'/2']xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR
key:  [73c5da0a/48'/0'/3'/2']xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq
```

These are `design/journeys/inputs-walletpolicy/key1..3.xpub` at the origins that
directory's own README pins. Nothing exotic is required: this is the repo's
conformance fixture, accounts 1', 2', 3' of one master.

Composed shape: wrapper `wsh`, the §4d **plain k-of-n preset**, one path,
**2-of-4** → 4 slots.

### Demonstration, step by step through the spec's own rules

1. §5 key-set row: SOLE path, unlocked, unhashed, n = 4 ≥ 2 → `sortedmulti`.
   Slots `@0..@3`.
2. §7d seating: `@0` ← record 1, `@1` ← record 2, `@2` ← record 3. Each carries
   "the origin the record … DECLARES, verbatim" (§4f), so `@2` declares
   `48'/0'/3'/2'` with fingerprint `73c5da0a`.
3. §7d shortfall: 4 slots, 3 assignable → REFUSE at the transition with §8p
   ("4 slots, 3 keys available. / Unfilled: slot @3."), offering **"engrave as a
   keyless template"**. The operator takes it — this is the spec's own offered
   route, not a mistake.
4. §4f unseated rule: slot `@3`, wrapper `wsh`, `account' = emitted index = 3`
   → `m/48'/0'/3'/2'`, **no fingerprint**.
5. `@2` and `@3` now declare the SAME origin. The engraved artifact:

```
wsh(sortedmulti(2,@0/48'/0'/1'/2'/<0;1>/*,@1/48'/0'/2'/2'/<0;1>/*,
                  @2/48'/0'/3'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*))
```

Measured — `md` encodes it, and warns exactly about this:

```
$ md encode --in tpl_collide.txt --fingerprint @0=73c5da0a \
      --fingerprint @1=73c5da0a --fingerprint @2=73c5da0a
md15rfdss3ef9kzz2jjtvyyh9ykcgfw2sqrqsuyvdshesu79mg99eutks2nnchdq5nlxxta0qpvsgl
warning: this keyless template's slots cannot be told apart — @2, @3 all declare
m/48'/0'/3'/2'. … a card here matches several slots and a device that will not
guess must refuse the whole set. …
```

```
$ md inspect md15rfdss…
origins:
  @0: [73c5da0a/48'/0'/1'/2']
  @1: [73c5da0a/48'/0'/2'/2']
  @2: [73c5da0a/48'/0'/3'/2']
  @3: m/48'/0'/3'/2'
wallet-descriptor-template-id: 6193a296a304a02d35fdaea20c5ed18a
```

The mixed declaration form is real: per-slot fingerprints for `@0..@2`, a bare
origin for `@3`.

6. **The device does NOT do what md's warning predicts.** Restore, through the
   shipped `seatKeyCards` (`gui/key_card_seating.go:53-118`). The operator mints
   the three cards on the host with the §7c-taught stub. `FormAwareStub`
   (`md/template_id.go:112-118`) selects `WalletDescriptorTemplateIdStub` because
   `isWalletPolicy` is `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0`
   (`md/template_id.go:17-19`) — false for a keyless template — so all three
   cards carry the template stub and LAYER 1 admits all three.

   Card `C2` = `{Path: "m/48'/0'/3'/2'", Fingerprint: "73c5da0a", Xpub: key3}`.

   - `slotMatchesCard(slot @2, C2)` (`:117-153`): path components equal;
     `slot.FingerprintPresent == true`; `equalFingerprint` → **match**.
     `filledBy[2] = C2`.
   - `slotMatchesCard(slot @3, C2)`: path components equal;
     `slot.FingerprintPresent == false`, so the fingerprint block at `:141-148`
     is **skipped entirely** — the function returns `true`. **match**.
     `filledBy[3] = C2`.
   - The contest guard is `if prev := filledBy[si]; prev >= 0 && !sameCardKey(…)`
     (`:88-92`). `prev` is `-1` on `@3`'s first (and only) claim, so
     `errSeatSlotContested` **never fires**. One card, two slots.
   - Every slot is filled, so `errSeatSlotUnfilled` never fires either.

   `seatKeyCards` returns a complete `[]md.ExpandedKey` in which
   `out[2].Xpub == out[3].Xpub`, and the device derives and DISPLAYS addresses
   for it. That is precisely the state the file's own header comment says the
   design exists to prevent: *"A misassignment does not fail — it derives a
   different wallet's address and shows it to the operator as PROOF, which is
   worse than showing none."*

7. **And the intended completion is then impossible.** When the real 4th
   cosigner's card `C4` (same origin `m/48'/0'/3'/2'`, a DIFFERENT fingerprint)
   is added, it matches `@3` too (no declared fingerprint to reject it),
   `prev = C2`, `sameCardKey` is false → `errSeatSlotContested`, and the WHOLE
   card set is refused. The template is either silently the wrong wallet (3
   cards) or permanently unseatable (4 cards).

### Why the existing gates all pass

- The shipped pinned test `TestAStrippedTemplateCannotSeatTwoMastersAtOnePath`
  (`gui/key_card_seating_test.go:349-393`) asserts its premise that **both**
  slots' fingerprints are stripped and uses **two** cards. The asymmetric,
  one-card case has no test.
- §12 item 6's acceptance reads "a template with two same-origin slots **and no
  fingerprints** is never produced". The artifact above has two same-origin
  slots and one of them DOES carry a fingerprint, so the acceptance is satisfied
  by a template that mis-seats.
- §7e's self-check compares "the decoded shape AND the slot assignment" — origins
  are not in either, so it is silent (see I-7).
- md's host warning is a CLI `warning:` on stderr. The device does not run `md`.

### Consequence

A silently mis-seated key, and a wallet on the restore screen that is not the
wallet that was composed, reached through the route §8p offers by name. Funds
sent to the displayed address land in a 2-of-4 whose third and fourth slots are
one key — a policy BIP-388 forbids (l.193, pairwise distinct), which no
coordinator will register, and whose threshold is satisfiable by fewer parties
than the operator believes.

### Minimal fix

1. §4f: the unseated-slot account must be **the lowest `account'` not already
   declared by any slot of this template**, not the slot's index. Add the
   invariant explicitly: *no two slots of a produced template may declare the
   same origin unless BOTH declare a fingerprint, and those fingerprints
   differ.*
2. §12 item 6: strike "and no fingerprints"; assert the invariant as stated
   above, and add the asymmetric one-card case (one slot with a fingerprint, one
   without, at one origin) as a named negative vector against the shipped
   `seatKeyCards`.
3. §7e: fold the pairwise-distinguishability test into the self-check on the
   DECODED md1 and refuse before consent — `md` already computes exactly this
   answer on the host, so it is a port, not a new rule (§9 item 1).

---

# IMPORTANT

## I-1. §7c's re-show trigger enumerates "a path or wrapper change", but the template id — and therefore the taught stub — measurably changes with a LOCK OPERAND and with a HASH DIGEST

**Spec defeated:** §7c ("The template id is key-independent and origin-invariant
but NOT shape-invariant: **a path or wrapper change alters it**, so the screen
re-appears after every shape edit"); §7d ("**Any change to the SHAPE, wrapper
included**, after at least one slot has been assigned discards ALL assignments …
because §5 renumbers slots by first appearance"); §8j.

Downstream of r0-journey I-8 (*§7c teaches an id the operator can invalidate by
taking the Back that §7d offers*) and r1-journey2 C-1 (*discard scoped to "the
path list", not the wrapper*). Both are folded. The defect is that the fold's
replacement enumerates the wrong set.

### Demonstration (measured)

Two templates differing in NOTHING but the `older` operand — the exact edit
§6b's day picker produces when the operator changes a relative lock:

```
$ md encode --in t_26280.txt   # wsh(or_d(multi(2,@0,@1,@2),and_v(v:pkh(@3),older(26280))))
wallet-descriptor-template-id: 7a426a7ec63f9c1305282efa16267a26
$ md encode --in t_26281.txt   # ... older(26281)
wallet-descriptor-template-id: b054e89b0b481309aad81d80b83671cd
```

And the hash digest, the exact edit §6c's pick list produces:

```
sha256(aa1122…) → wallet-descriptor-template-id: b7758367ea54eb7429fdb693568006e7
sha256(bb1122…) → wallet-descriptor-template-id: 35058b33003bc5db125a9b8b171d5320
```

Mechanism, confirmed in the Go port: `WalletDescriptorTemplateId` hashes
`useSitePath ‖ writeNode(tree)` (`md/template_id.go:21-28`) — the lock operand
and the digest are nodes of that tree. §7c's other two claims ARE true and were
checked: key-independence and **origin-invariance** hold (three variants of one
shape at origins `0'/1'`, `7'/9'`, and none, all give
`aad0e0e0718cbe91da67cc2bd72c68c9`).

### Both readings of "shape edit" produce a defect

- **Narrow** ("a path or wrapper change", as §7c literally enumerates): the
  operator writes the stub down at §7c, mints cards on the host with
  `mk encode --policy-id-stub`, then changes "90 days" to "180 days". No
  re-show, no §8s. Every card now carries a stub the engraved template does not
  have, and LAYER 1 refuses all of them at restore
  (`errSeatNotThisPolicy`, `gui/key_card_seating.go:66-73`). The plates and the
  cards are individually valid and mutually unusable.
- **Broad** (any §7b edit, since §7b puts lock and hash under "Shape"): §7d
  discards every seating on a lock-value edit, which cannot be justified by its
  own stated reason, and §8j's body — *"Slot numbers change with the shape."* —
  is FALSE in that state: numbering is by first appearance in the emitted text
  (§5) and a lock operand changes no placeholder.

### Minimal fix

Replace the enumeration in §7c with the measured rule: *the template id changes
with the wrapper, the path list, any lock operand and any hash digest — that is,
with everything §7b lets the operator edit — so the screen re-appears after every
edit made on the shape screen.* Then split §7d's discard rule from it: discard
assignments only when **slot numbering** changes (wrapper, path count, or a
path's key count), and re-show the stub screen on every edit. §8j's body then
stays true, because it fires only where slot numbers really move.

## I-2. §7f and §8p give contradictory rules for the partially-seated template, and neither §7d's "BOTH stubs" nor §7f's form-collapse has a rule for that state

**Spec defeated:** §7f ("**Every seated slot yields a card in form B regardless
of source**" / "A keyless composition (**no seated slots**) has no form A and no
cards"); §8p / §7d ("'engrave as a keyless template' (**§7f form B with no
cards**)"); §7d ("every seated card is later cut as a RE-MINTED mk1 carrying
**BOTH** the composed template's stub and the composed policy's stub"); §12
item 6 ("**both stubs present**").

The partially-seated state is created by the fold — it is the §8p escape hatch —
and three normative sentences disagree about it:

1. §7f: every seated slot yields a card. §8p: form B **with no cards**. In the
   C-1 construction three slots are seated; the operator either gets three cards
   or none, and which one decides whether the wallet is restorable at all.
2. §7d and §12 item 6 require BOTH stubs on every card. In this state the
   composed **policy** id does not exist — §7c is explicit that it is added only
   "After seating (§7d)", and seating never completed. "Both stubs" is
   unsatisfiable, and §12 item 6 as written cannot pass for the one artifact
   route §8p advertises.
3. §7f collapses the form choice to "template only" only when there are **no**
   seated slots. With some slots seated, form A (concrete policy) is still
   offered by the letter of the rule and is impossible to produce — an unseated
   slot has no xpub. The operator meets that dead end AFTER consent.

### Consequence

The single most likely real-world outcome — more slots than keys in hand — has
no defined artifact. Depending on which sentence an implementer follows, the
operator engraves a template whose cards were never cut, or cards that cannot
carry the stub the spec requires, or picks a form that cannot be built.

### Minimal fix

Give the partially-seated case its own paragraph in §7f: form A is not offered;
form B yields the keyless template plus one card per **seated** slot, carrying
the **template stub only**; the screen says the policy id does not exist yet and
will not until every slot is seated. Amend §12 item 6 to "both stubs when a keyed
policy exists, the template stub otherwise", and add the partially-seated
artifact as a named vector.

## I-3. §6a's "the door's 'Keys loaded: N' is the device's only signal" and §12 item 8's "the door's count reduced by one" are FALSE for a malformed `hash:` or `now:` record — there is no signal at all, and the acceptance cannot pass

**Spec defeated:** §6a ("On the DEVICE a record that fails classification goes
INERT … so the **door's 'Keys loaded: N' (§7a) is the device's only signal** and
the spec says so rather than promising a screen"); §12 item 8 ("for each
malformation the host emits its §8n line and **the device leaves the record inert
with the door's count reduced by one**"); §7g row "pack | a malformed
`key:`/`hash:`/`now:` record | REFUSAL on the host (§8n); INERT on the device,
**visible only in the door's count**".

This is the FOLD's answer to r1-journey2 I-4 (*a malformed record reaches NO
screen; §12.8 passes either way*). The answer is wrong for two of the three
classes.

### Constructed input

A payload packed by any tool other than `me sysw pack` (or by a future `me` whose
validation drifts) carrying:

```
hash:aa11223344556677889900aabbccddeeff001122334455667788   # 31 bytes, not 32
now:<hex of "1756684800,999999999">                          # height above §4c's band
```

### Demonstration

- The door's line is **"Keys loaded: N"** (§7a, §8r). A `hash:` record is not a
  key and a `now:` record is not a key, so N is unchanged by either
  malformation. §12 item 8's assertion "the door's count reduced by one" is
  **unsatisfiable** for these two classes: the count it names never moves, so
  the acceptance either fails on a correct implementation or is quietly dropped.
  A gate that cannot pass is not a gate.
- §6a's "the door's count is the device's only signal" is therefore false: for
  `hash:` and `now:` there is **no** signal. The operator reaches §6c, finds the
  hashlock pick list one row short of the digests they packed, and has nothing
  on the device to tell them why. The plausible recovery — typing the 64 hex by
  hand from memory of what they packed — is exactly the path §6c's fallback
  makes available, and it re-enters a digest the device never verified against
  anything.
- For the `now:` case the device silently falls back to the "cannot tell the
  time" line (§6b), which is indistinguishable from a payload that carried no
  `now:` at all.

### Minimal fix

Say what is actually true, per class: a malformed `key:` reduces "Keys loaded";
a malformed `hash:` or `now:` produces **no device-visible signal at all**, and
that is the accepted cost of the inert contract. Then either add a door line
that can carry it (e.g. "3 payload records were not understood") or state
explicitly in §7g that these two classes are host-refusal-only. Rewrite §12
item 8's per-class assertion to the signal each class actually has.

## I-4. §8n's second-`now:` refusal fires on the host's OWN auto-appended record, names an index the operator's file does not contain, and names no remedy — violating §11

**Spec defeated:** §8n line 4 ("record N: a second now: record; only one is
allowed"); §6a / §10 item 2 ("`me sysw pack` appends `now:` as the LAST record by
default; `--no-now` omits it"); §11 ("**Every refusal … names what to do
instead**").

### Constructed input

```
$ cat records.txt
key:<hex of "[73c5da0a/48'/0'/1'/2']xpub6Dzhy…">
hash:aa11…ee
now:<hex of "1893456000">          # the operator pins their own pack time
$ me sysw pack --in records.txt
```

### Demonstration

Three records go in. §10 item 2 appends `now:` **last, by default**, making four.
The payload-wide rule then fires. Measured constraint on the index it can print
(`me sysw pack --help`, verified): *"Blank lines are skipped, so a record's index
is its position among the NON-blank lines, not its line number."* — the indices
are positions in the record vector, and the offending fourth record is one the
host manufactured. So the refusal is one of:

- `record 4: a second now: record; only one is allowed` — pointing at a record
  the operator's file does not have. There is no line 4 to fix.
- `record 3: …` — pointing at the operator's record and telling them it is the
  *second*, which it is not.

Either way the operator cannot act on it: nothing names `--no-now`, which is the
only way to supply their own `now:`. §11 says every refusal names what to do
instead; this one names nothing, and the flag that resolves it appears only in
§10, a host **work item**, not in any operator-facing copy.

### Consequence

The one deliberate, expert use of the `now:` class — pinning a bound instead of
accepting wall-clock — is unreachable by a route the operator can discover, and
the refusal blames the wrong record.

### Minimal fix

Auto-append `now:` only when the record vector contains none, and make an
explicit operator-supplied `now:` win silently. Keep the two-`now:` refusal for
two OPERATOR records, and give it a remedy line: *"…; remove one, or pass
`--no-now` to keep your own."*

## I-5. §6b's bound line replaces the "nothing has checked this" disclaimer with a reassurance whose truth depends on the payload's age, and nothing bounds that age

**Spec defeated:** §6b ("Above it → echo '**at least N days after this payload was
packed on `<pack date>`**' … When the relevant field is ABSENT the echo carries
instead: 'This device cannot tell the time. Nothing here has checked that this is
in the future.'"); §8c; §7g row "lock | no `now:` field for this lock kind |
DEFAULT: the 'cannot tell the time' line".

This is the fold's answer to r0-journey I-2 (*the device is silent when it has no
pack-time bound*). The fold added the honest line for the ABSENT case and, in the
same move, made the PRESENT case the only state where the disclaimer is
withdrawn.

### Constructed input

A payload packed on 2026-09-01 (`now:` seconds = 1788220800-ish), left in flash.
§7a's own door line proves payloads persist across boots and are re-loadable
later: *"A payload is in flash but not loaded. Load it from the carousel
first."* Nothing in §6a, §6b or §7a bounds how old a loaded `now:` may be.

Two years later the operator composes an inheritance path and enters
`20280101`.

### Demonstration

1. §6b: the date's Unix value exceeds the `now:` seconds field, so it is not
   refused.
2. §6b's above-bound branch fires: **"at least 487 days after this payload was
   packed on 2026-09-01"**.
3. The date entered is, in real time, a year in the PAST. The `after()` lock is
   already satisfied; the recovery path is spendable the moment the wallet is
   funded, by whoever holds that one key.
4. The operator never sees "This device cannot tell the time. Nothing here has
   checked that this is in the future." — the one line that would have told them
   the truth. The fold's rule withdraws it in exactly the state where a stale
   input makes it most needed.

The line is literally true and its inference is exactly what it invites; the
pack date is on screen, but reading it as *"and that was recent"* is what the
sentence is built to encourage.

### Consequence

An inheritance or recovery path that the operator believes is years away is live
immediately, on the first device-authored timelock this constellation ships.

### Minimal fix

Never withdraw the disclaimer. Make the bound line additive, not a replacement:

```
This device cannot tell the time. The payload says
it was packed on 2026-09-01, which may be long ago.
Nothing here has checked that this is in the future.
```

Keep the below-bound REFUSAL exactly as specified — that direction is sound,
because a stale `now:` only weakens a refusal, never invents one.

## I-6. §6a's `key:` body rule pins the component COUNT and (downstream) the LAST component, and leaves the ACCOUNT unverifiable — which is F-217, recorded in this repo, unmitigated and unclassified

**Spec defeated:** §6a ("`key:` MUST parse as BIP-380 key-origin notation with a
NON-EMPTY origin …, an xpub at depth 3 or 4 (md's own `--key` rule), and **an
origin whose component count equals the xpub's depth**"); §4f ("A slot seated
from a `key:` record or an mk1 card carries the origin the record or card
DECLARES, **verbatim**"); §7g (no row for it).

This is the fold's answer to r0-adversarial I-5 (*§6a specifies only
hex-validity*). The body rule the fold wrote reads as an integrity check. It is
not one.

### Constructed input

```
key:<hex of "[73c5da0a/48'/0'/0'/2']xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk">
```

That xpub is `key1.xpub`, which the fixture README pins at `48'/0'/1'/2'`. The
record declares account **0'**.

### Demonstration (measured — what each layer actually checks)

```
$ mk encode --xpub xpub6Dzhy… --origin-fingerprint 73c5da0a \
      --origin-path "m/48'/0'/0'/2'" --policy-id-stub aad0e0e0
mk1qp 74yrp qqsq6 458qu peutk s2q5z g3vs7 …          # ACCEPTED
```

`md decompose` accepts the same declaration in a descriptor. The check that DOES
exist bites only on the last component:

```
$ mk encode … --origin-path "m/48'/0'/1'/1'"
error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 4 / last Some(Hardened { index: 1 })
```

Confirmed at the source: `compactFromXpub` (`mk/encode.go:151-161`) compares
`key.Depth()` against `len(comps)` and `key.ChildIndex()` against
`comps[len(comps)-1]` — and nothing else. All four fixture xpubs have child
`0x80000002` (verified by base58-decoding their headers), because all four end
at `2'`. The **account** component is structurally unverifiable from an xpub
alone, and the master fingerprint is unverifiable at any depth above 1.

So a `key:` record satisfying every clause of §6a can name any account it likes.

### Consequence

The declared origin is engraved verbatim — into the template, into the minted
mk1 card, and into any form-A concrete descriptor. Restore from the plates works
(the xpub is on them) and the addresses are right, so consent shows a green
wallet. What fails is **signing**: a signer handed this descriptor derives at
`m/48'/0'/0'/2'`, gets a different key, does not find itself in the policy and
refuses. The wallet is funded and unspendable by that cosigner until someone
diagnoses the path by hand. This repo has already had this exact defect once —
F-217, recorded in
`design/journeys/inputs-walletpolicy/README.md`: *"that card described a wallet
that cannot exist"*.

### Minimal fix

State the residual explicitly in §6a — *the last component and the depth are
verified; the account and every interior component are declarations this device
cannot check* — add a §7g row for it (class: **documentation**, at the mapping
review), and make the §7d mapping review print each slot's origin verbatim beside
its fingerprint with a one-line note that the device cannot confirm the key was
derived there. That costs one line and turns an invisible failure into a
readable one.

## I-7. §7e claims "a builder defect cannot reach steel as a reviewed wallet", but the self-check compares only shape and assignment — origins, fingerprints, use-site and id kind are outside it, and no other surface reads them off the decoded md1

**Spec defeated:** §7e ("Before the screen is shown the device asserts that the
decoded shape AND the slot assignment equal the composed path list and seating,
and REFUSES to continue on mismatch with §8q, **so a builder defect cannot reach
steel as a reviewed wallet**").

Downstream of r1-journey2 I-8 (*the self-check refusal names no action*), which
the fold closed by adding §8q's exit and the fault-injection acceptance. The
scope claim is what is untouched.

### Constructed defect and demonstration

Inject a defect in the §4f origin function — the code §9 item 8 explicitly allows
to be a **second copy** of the wrapper→account-type table (*"a taproot member on
`md.MultisigScript`, **or the composer's own origin function with the same
table**"*). The defect: the `tr` arm emits `2'` (the wsh account type) instead of
`3'`.

Now walk the spec's own surfaces:

| surface | sees the defect? |
| --- | --- |
| §7d mapping review (slot → fingerprint + origin) | No — it renders **UI state**, not the decoded md1; the UI state carries the same wrong origin |
| §7e self-check (decoded shape, slot assignment) | No — origins are in neither operand |
| §7e consent lines (k-of-n, lock, digest, EXPERIMENTAL marks, key-path line, ids, addresses) | No — the enumerated list contains no origin |
| addresses 0..1 | No — addresses derive from the **xpubs**, which are correct |
| §8l warning | No — it asks the operator to compare an address, which matches |

Every surface is green. The engraved template declares
`m/48'/0'/N'/2'` for a taproot wallet whose keys were derived at
`m/48'/0'/N'/3'`. The cards the device mints carry the same wrong origin, so
restore-and-seat also succeeds. The error surfaces the first time a cosigner is
asked to sign — see I-6's consequence, identical shape.

The same argument holds for a use-site defect (`/<0;1>/*` emitted as bare `*`):
shape and assignment are unchanged, and consent's receive/change pair would
merely print two identical addresses, which nothing asserts against.

### Consequence

The sentence is a guarantee the mechanism does not deliver, on the screen that
exists to be the last gate before steel. C-1 above is a live instance: it is
exactly an origin defect, and the self-check is silent on it.

### Minimal fix

Extend the self-check to the two fields it can compare for free — it is already
holding the decoded md1 — namely **per-slot origin and fingerprint against the
mapping review**, plus **pairwise slot distinguishability** (see C-1 fix 3) and
the fixed use-site. Then narrow the claim to what is checked: *"so a builder
defect in the shape, the seating, the origins, the fingerprints or the use-site
cannot reach steel as a reviewed wallet"* — and name what remains outside it.

---

# MINOR

## M-1. §8t's body states a technical impossibility that is false

**Spec defeated:** §8t ("**Dates before 2009 cannot be written as a time lock.**");
§6b ("the entry refuses every date before 2009-01-03 with §8t").

Dates from 1985-11-06 onward have Unix values above 500,000,000 and encode
perfectly well as `after()` time locks — 1995-12-25 00:00 UTC is 819,849,600,
comfortably inside §4c's time row. §6b's own text says so two sentences earlier
("1985-11-05 00:00 UTC is 499,996,800"). The 2009-01-03 floor is a policy choice
of this build, not a limit of the encoding, and §4e's sibling copy (§8m line 3)
gets this exactly right for the analogous case: *"This build will not put a
key-less path in taproot"*. Reachable by typing `19951225`. Fix: *"This build
will not write a date before 2009 as a time lock."*

## M-2. §8g's satisfiability sentence is false whenever the shared seed fills fewer slots than the path's threshold

**Spec defeated:** §8g ("Slots @1 and @2 are the same seed. **This path's 2-of-3
can be satisfied by one person.**"); C29; §7d.

C29's trigger is "one seed at two slots INSIDE one path" — it does not require
the seed to reach the threshold. Constructed: a 3-of-5 path with slots @0..@4,
one seed assigned to @0 and @1. The warning fires; the body says the 3-of-5 can
be satisfied by one person; it cannot — that person holds two of the three
signatures required. The C29 grounds are otherwise well sourced and one of them
stays true in this state (Liana's `DuplicateOriginSamePath` refuses on shape, not
on threshold, per brainstorm §3.11), so only the middle sentence is wrong. Fix:
make the middle line conditional — state single-person spendability only when the
shared seed's slot count in that path is ≥ k, and otherwise say *"one person
holds 2 of the 3 signatures this path needs."*

## M-3. §7a/§8r's "Keys loaded: N" counts a seed as one key, while §7b and §7d count it as "any slots"

**Spec defeated:** §7a ("'**Keys loaded: N**' when a payload holds keys **or
seeds**"); §8r ("Keys loaded: 4"); §7d ("'keys available' … counts records plus
cards plus, **for each seed, 'seed: any slots'**").

A payload holding one BIP-39 mnemonic and no `key:` records shows **"Keys loaded:
1"** at the door, while the path-list screen two steps later shows "keys
available: seed: any slots". The door's number is the operator's first and only
input to "can I build a 4-slot policy from what I loaded?", and it understates the
answer by a factor of 32. The most likely divergence is the operator building a
smaller policy than they wanted, or taking the keyless-template route
unnecessarily — and §7g has no row for it. Fix: the door line should say
"Keys loaded: N (plus 1 seed)" or, with seeds only, "A seed is loaded. It can
fill any number of slots."

## M-4. §8p's C5 cause line fires on this repo's own fixture set and misdiagnoses a plain shortfall

**Spec defeated:** §7d ("the C5 cause line **ONLY when a fingerprint the payload
already holds appears in two paths** of the composed shape"); §8p line 2.

This is the fold's answer to r1-journey2 I-5 (*the shortfall refusal names ONE
cause for a condition that has several*), and the gate it added admits the wrong
cases. Constructed: the `key0..3` fixtures all share fingerprint `73c5da0a`
(README, verified). Load three of them, compose a two-path wsh policy with 4
slots, seat three. The condition holds — a fingerprint the payload holds appears
in two paths — so §8p prints *"One person is in two paths and needs two keys: a
second account from the same seed, or a second card."* The operator has ALREADY
done exactly that (two accounts of one seed, in two paths). The real cause is
"you have three keys and four slots", which line 1 already says. Note also that a
`key:` record or card is "used at most once" (§7d), so this line can never fire
for its intended cause via records at all — only via a seed, and a seed cannot
produce a shortfall. Fix: gate the line on **unfilled slots whose path already
contains a fingerprint the operator has assigned elsewhere**, or drop it and let
line 1 stand.

## M-5. §8s's "Path N key i of n" prompt is undefined once taproot extracts the internal key

**Spec defeated:** §8s ("Slot @2, **Path 1** key 2 of 3: choose a key"); §5
(placeholder numbering by first appearance; the extracted internal key is `@0`);
§7d.

Constructed: under `tr`, path list `[P1: 2-of-3] [P2: single key, unlocked,
unhashed]`. §5 extracts P2's key as the internal key `@0`; L = `[P1]`, m = 1, so
the emitted text is `tr(@0/…,sortedmulti_a(2,@1/…,@2/…,@3/…))` — verified to
encode:

```
$ md encode "tr(@0/48'/0'/0'/3'/<0;1>/*,pk(@1/48'/0'/1'/3'/<0;1>/*))"   → OK
$ md encode "tr(@0/…,{pk(@1/…)})"  → md: template parse error: taptree branch must have 2 children
```

(§5's "the leaf written bare" rule is correct — that attack failed.) The operator
listed the 2-of-3 as **path 1**; it is the only leaf, so an implementation
numbering by emitted leaf calls it path 1 too. Add a lock to P1 and reorder, or
use ≥ 2 leaves with a not-first-listed internal key, and the two numberings
diverge. §5 says slot labels are the emitted indices; it says nothing about
"Path N". The operator seats by their own mental model of their own list. Fix:
one sentence in §7d — *"'Path N' in the seating and mapping prompts is always the
operator's listed path index, never the emitted leaf index."*

---

# NIT

## N-1. §7d's re-mint can raise a card's chunk count, and §7f's census does not cover cards

`mk` bytecode is `header(1) | stub_count(1) | stubs(4*N) | fp(4) | path |
compact73`, and the cross-chunk fragment is 53 bytes (`mk/encode.go:26-29`; a
1-stub card is ~84 B → 2 chunks). Appending two stubs adds 8 bytes; a card that
already carries several stubs can cross into a third chunk, i.e. an extra plate.
§7f's census refuses "a concrete descriptor longer than the plate holds" and says
nothing about card chunk counts. Not a correctness defect (the cap is 255 stubs,
`mk/encode.go:78-80`), but the plate count is a number the operator plans around.

---

# ATTACKS TRIED THAT FAILED — do not re-run these

1. **`key:` body form vs `md decompose`.** §6a claims the `key:` body is "the key
   form `md decompose` prints". Verified: `md decompose --emit keys` prints
   `[fingerprint/path]xpub` with **no** use-site suffix, so the origin-vs-use-site
   ambiguity does not exist and the mk1 `Path` field (origin only,
   `mk/mk.go:135`) is the right target for `slotMatchesCard`
   (`gui/key_card_seating.go:117-140`, which parses `c.Path` and compares to
   `slot.OriginPath`). §7f's mint description is consistent.
2. **Template-id origin-invariance (§7c).** Three variants of one shape at
   origins `0'/1'`, `7'/9'` and none all yield
   `wallet-descriptor-template-id: aad0e0e0718cbe91da67cc2bd72c68c9`. §7c's
   claim holds; confirmed at the source (`md/template_id.go:21-28`: the preimage
   is `useSitePath ‖ writeNode(tree)`, no origins, no fingerprints, no keys).
   Note the **wallet-policy-id** is origin-dependent even with no keys
   (`cc13…`/`bea0…`/`465a…`), but §7c makes no invariance claim about it.
3. **Stub dispatch for a fingerprinted keyless template.** `isWalletPolicy` is
   `d.tlv.pubPresent && len(d.tlv.pubkeys) > 0` (`md/template_id.go:17-19`) —
   fingerprints do NOT flip it. Verified: the C-1 artifact, carrying three
   per-slot fingerprints, reports `wallet-policy-mode: false`, so
   `FormAwareStubChunks` returns the TEMPLATE stub and layer 1 of `seatKeyCards`
   looks for the same stub §7c teaches. The r0 C-4 fix holds.
4. **§7c's taught `mk encode` command line.** All four flags exist on the
   installed mk 0.13.0: `--xpub`, `--origin-fingerprint`, `--origin-path`,
   `--policy-id-stub` (and it is repeatable). The command as printed is runnable.
5. **§4b's 32-slot cap and §8m line 5's boundary.** Measured on md 0.14.0: a
   32-slot `wsh(or_i(multi(2,@0..@15),multi(2,@16..@31)))` **encodes**; the
   33-slot sibling gives `md: codec error: key count 33 out of range; require
   1 ≤ n ≤ 32`. The cap is policy-wide and exactly 32, as §4b states and r1
   feasibility C-1 established.
6. **§5's taproot spine forms.** `m = 1` bare leaf `tr(IK,P)` encodes; the braced
   `tr(IK,{P})` is refused (`taptree branch must have 2 children, but found 1`),
   exactly as §5 says; the `m = 3` spine `tr(IK,{P1,{P2,P3}})` at depths 1,2,2
   matches `min(j, m−1)` and encodes.
7. **`me sysw pack`'s host refusal machinery.** §6a's cite is sound in substance:
   `admit_check` (`crates/me-cli/src/sysw/mod.rs`) loops the record vector and
   returns `SyswError::Unclassifiable(i, …)` for any `Class::Unknown`, and
   `split` calls it first so the partition is total. A malformed prefixed record
   really is refused by index on the host. (The defect is I-4's *index*, not the
   mechanism.)
8. **`older`/`after` band arithmetic (§4c, §6b).** 388 days → `ceil(388×86400/512)
   = 65475 ≤ 65535`; 389 days → 65643, correctly outside. 2009-01-03 = 1,230,940,800
   and 2038-01-19 = 2,147,472,000, both strictly inside §4c's time row. The
   `now:` regex's 10-digit seconds and 9-digit height are both caught by the
   stated range checks. No off-by-one found.
9. **Duplicate-xpub seating (§7d).** "Two slots resolving to the same xpub →
   REFUSE at the mapping review" is genuinely upstream of md's encode-time
   refusal, and the same-fingerprint-same-path case is refused by md
   independently (F-217's guard). No gap.
10. **§8g's "Liana will refuse it".** Sourced and measured in the brainstorm
    record §3.11 (`DuplicateOriginSamePath`, W1/W3 refuse). Only the
    satisfiability sentence is wrong (M-2).
11. **Script-limit exhaustion at the grammar's bounds.** 8 paths × 9 keys is
    capped at 32 slots by §4b, giving a wsh witnessScript far under
    `MAX_STANDARD_P2WSH_SCRIPT_SIZE` (3600) and an op count under 201; §5b's
    `sanity_check` leg covers it in any case. No unspendable-by-limit
    construction found.
12. **Two `now:` records reaching the device.** §6a's device rule (both inert,
    door shows no bound) fails safe — the operator gets the "cannot tell the
    time" line, never a bound chosen from one of two. Correct as written.

---

# WHAT I RAN

- `git diff --stat 99463ac..HEAD -- design/SPEC_wallet_policy_composer.md` (empty
  — the spec under review is the working-tree text).
- Read: the full spec at `99463ac`; `design/agent-reports/composer-spec-R0-r0-adversarial.md`
  (head + finding headings) and the finding headings of all six r0/r1/r2 reports,
  to avoid re-raising settled findings.
- Fork source read: `gui/key_card_seating.go` (whole), `gui/key_card_seating_test.go`
  (lines 100-393), `md/template_id.go` (17-174), `mk/encode.go` (1-182),
  `mk/mk.go` (133-158), `sysw/descriptor.go` (35-60), `gui/sysw_session.go` (60-110).
- Host source read: `crates/me-cli/src/sysw/mod.rs` (`pack_with`, `admit_check`,
  `split`); `me sysw pack --help`, `md --help`, `md encode --help`,
  `md decompose --help`, `mk encode --help`.
- `md decompose --in desc4.txt` on a 4-key wsh descriptor built from
  `design/journeys/inputs-walletpolicy/key0..3.xpub`.
- `md encode --in tpl_collide.txt --fingerprint @0/@1/@2=73c5da0a` +
  `md inspect` — the C-1 artifact, its per-slot origins and md's own
  indistinguishable-slots warning.
- `md encode` + `md inspect` on `older(26280)` vs `older(26281)`, and on
  `sha256(aa11…)` vs `sha256(bb11…)` — I-1's template-id measurements.
- `md encode` + `md inspect` on three origin variants of one shape — the
  origin-invariance control (failed attack 2).
- `mk encode` with a wrong account (accepted) and with a wrong last component
  (refused) — I-6.
- `md decompose` with a wrong last component (refused, with md's full guidance
  string) — I-6.
- `python3` base58 header parse of `key0..3.xpub` (depth, parent fingerprint,
  child index) — I-6's proof that the account component is unverifiable.
- `md encode` on 32-slot and 33-slot templates — failed attack 5.
- `md encode` on the three taproot spine forms — failed attack 6.

Scratch files under
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/702b37c9-e041-404f-8220-2456ff9c6bf3/scratchpad/`.
No repo file was modified except this report.
