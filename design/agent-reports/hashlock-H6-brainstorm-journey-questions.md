# H6 preimage plates — journey walk, and the questions the fixed decisions leave open

Journey walker, brainstorm of hashlock stage H6. Brief:
`design/agent-briefs/hashlock-H6-brainstorm-journey-brief.md`. Decisions 1-9 of
`design/agent-briefs/hashlock-H6-brainstorm-draft-brief.md` are FIXED and are
applied, never reopened, below.

Revisions walked (all read-only, nothing committed):

| repo | revision | what was read |
| --- | --- | --- |
| mnemonic-engrave | `07a92e9b` (master) | `design/SPEC_hashlock_H2_device.md`, `SPEC_hashlock_H5_device_polish.md`, `SPEC_wallet_policy_composer.md`, `FOLLOWUPS.md`, `crates/me-cli/src/sysw/`, `crates/me-cli/src/seal/record.rs`, `crates/me-cli/src/main.rs` |
| seedhammer fork | `fb0dd04` (main) | `gui/composer_*.go`, `gui/bundle_flow.go`, `gui/gui.go`, `gui/freetext_flow.go`, `gui/passphrase_flow.go`, `gui/singlesig.go`, `gui/sysw_*.go`, `sysw/`, `backup/`, `hashlock/`, `md/`, `mk/` |
| mnemonic-secret | `504ff46` | `design/SPEC_ms_hashlock.md`, `crates/ms-cli/src/hashlock_phrase.rs` |
| mnemonic-toolkit | `46b40bb`-era working tree | `docs/manual/src/40-cli-reference/43-ms.md` |

Classification vocabulary is the constellation's: **refusal / warning / default /
not our concern / documentation only**. A divergence is kept **only** where the
wrong outcome is worse than telling the operator nothing. Divergences that did
not clear that bar are listed under each journey as "walked, not kept", so the
author can see they were considered rather than missed.

Nothing here is answered. Every kept divergence becomes a question in §6.

---

## J1 — device-derived: build, type a phrase, HOLD, Done, review, cut, then a year later

### The walk

| # | step | what the operator has in hand, exactly | what the device does at `fb0dd04` + decisions 1-9 | what ELSE they might do → what happens | class |
| --- | --- | --- | --- | --- | --- |
| 1 | Wallet Policy door | a payload (or none) | `composerDoorFlow` lists "Build a new policy" (`gui/composer_door.go:98-119`) plus, under D4, a new "Hashlock plates" route | pick Hashlock plates with a composition in mind → lands in the payload-only flow, no policy | see J2/Q6 |
| 2 | shape, then a path's `Which hash?` | nothing yet | rows = payload `hash:` digests, `Type a hashlock phrase`, `Type 64 hex`, `No hash lock` (H2 §4.1; label-keyed switch, H2 §5). D4 adds payload preimage/phrase records as pickable digests | pick a payload phrase row → the digest is not known until ~10 s of PBKDF2 has run | **kept → Q7** |
| 3 | phrase screen | the phrase in their head, on paper, or off a generator | `hashlockPhraseFlow`, 4-page printable-ASCII keyboard, `n/100` counter from `hashlock.PhraseMaxChars` (`hashlock/hashlock.go:31`) | paste an ms1 plate string → `ValidatePhrase` refuses via `IsMS1Shaped` (`hashlock/hashlock.go:101,122`) and names the host route | shipped |
| 4 | method pick + its modal | phrase in RAM | `hardened` (≈10 s) or `sha256`, each behind a confirm-to-proceed modal (H2 §4.3) | decline → back to the method pick, phrase intact | shipped |
| 5 | derivation | phrase in RAM | `hashlockDeriveFlow`; **X lives on the stack and is dropped when the route returns** (`gui/composer_hashlock.go:19-20`) | — | **D9 changes this: X, the phrase and the method must now survive to Done → Q9** |
| 6 | HOLD | digest on screen as `first8..last8` | `st.list.Paths[idx].Hash = &d`, `composerNotePhraseDigest`, then the F-487 reconcile screen (`gui/composer_hashlock.go:66-83`) | — | shipped |
| 7 | later: re-open `Which hash?` on the same path and pick a payload row, or Remove path | a retained phrase whose digest no path carries | `phraseDigests` **never deletes** (`gui/composer_state.go:46-50`) — deliberately, for §8h. D6's "not on any path, will not be cut" rule is written for *payload-delivered* preimages only | the review offers, or silently drops, a plate for a phrase the policy no longer uses | **kept → Q4** |
| 8 | Done → §8h | every path hashed, at least one by phrase | `composerCopyHashEveryPath` (`gui/composer_copy.go:169-173`): "not on this device and not on these plates … Back up every preimage separately" | with H6 the preimage IS on a plate in this same bundle, so the banner is now false | **kept → Q10** |
| 9 | form pick + census | the composition | `composerFormPick` → `composerCensusLines` → `confirmReviewScreen("Plates To Cut")` (`gui/composer_flow.go:389-392`) | D6 inserts a per-plate form/QR review here | **kept → Q2 (where the preimage plate sits in the census)** |
| 10 | the cut | blanks on the bench | `bundleEngrave(ctx, th, "Wallet Policy", cards, "", "")` (`gui/composer_flow.go:393`). Order today: secrets **first**, then md1/mk1 (`composerEngraveStep`, `cards = append(secrets, cards...)`) | power dies, or Back, after the md1 plates and before the preimage plate | **kept → Q1** |
| 11 | per plate | one blank clamped | `bundleEngrave` draws a `ChoiceScreen` "Choose engraving" from `validateMdmkStrings`, which for a **single-string card offers TEXT + QR / TEXT ONLY / QR ONLY** and QR-encodes **that string** (`gui/gui.go:2634-2643`) | the preimage plate string is one string → this machinery would offer a QR **of the ms1 string**, which D1 forbids | **kept → Q3** |
| 12 | a year later, plate-string form, plate alone | one steel plate: title band, header, a 75-char `ms10hashsq…` string | host: `ms hashlock --in plate.txt` → `hash:<64 hex>` + the digest (manual `43-ms.md:320,372`) | they hand it to `ms decode` → prints kind, preimage hex, digest, never words (`43-ms.md:278`) | shipped |
| 13 | a year later, phrase form + QR | one plate: labelled lines, QR on the right | **nobody on the SH2 can scan it — the SH2 has no camera.** The reader is a phone or a host camera; `ms hashlock` does not parse the QR text (D2 defers that) | they scan with a phone → the phrase lands in a phone's scan history/clipboard, then must be **retyped** into `ms hashlock` | **kept → Q5** |
| 14 | a year later, the md1 policy plates alone | the descriptor plates, no preimage plate found | nothing on those plates says a preimage exists. The composer prints **no restore document** (`composerEngraveStep` ends at `bundleEngrave`; only `multisigRestoreDocFlow`/`singlesig_restore.go` produce one) | they conclude the wallet is spendable and fund it | **kept → Q8** |

### Walked, not kept

- **Back at step 3 drops the phrase** (H2 §4.6) — shipped, tested, and the loss is one retype. *Not our concern.*
- **`show` on the phrase keyboard reveals the phrase** — H2 §4.2 inherits it; secret-handling, non-gating by the 2026-08-27 ruling (F-481/F-483 territory). *Documentation only.*
- **`first8..last8` vs the host's full 64 hex** — closed by F-486 in the manual. *Nothing owed.*
- **Two paths, one phrase** — the confirm modal's other-path line already covers the *different*-digest case (H2 §4.5); identical digests on two paths is one backup burden and correctly silent. *Default.*

---

## J2 — host-derived: `ms hashlock`, `me sysw pack --pack-preimage`, tap, Hashlock plates, cut

### The walk

| # | step | in hand | what happens | what ELSE → what happens | class |
| --- | --- | --- | --- | --- | --- |
| 1 | `ms hashlock --hashlock-phrase-stdin --out X.txt < phrase.txt` | a phrase file | stdout `hash:<64 hex>`; `--out` writes the **ms1 preimage string**, 0600; the stderr card carries the preimage, the method line and `phrase_chars` (`SPEC_ms_hashlock` §4.4) | `--random --out X.txt` → the file is the only copy, `create_new` refuses to overwrite (§4.1) | shipped |
| 2 | build the record file | `X.txt` (ms1) **or** the phrase + method | the ms1 line is a record; a **phrase-and-method record is NEW** (D3: Rust first with vectors, then Go) | they put the raw phrase in as `text:` → it packs as a free-text engraving record, not a preimage | **kept → Q11** |
| 3 | `me sysw pack` without the flag | records file | the preimage plate is refused by kind: `UnknownReason::PreimagePlate` (`crates/me-cli/src/sysw/mod.rs:209-210`, predicate `seal::record::preimage_plate` at `seal/record.rs:287`) | D7 keeps this refusal and adds "re-run with `--pack-preimage` if that is what you intend" | shipped + D7 |
| 4 | `me sysw pack --pack-preimage --out payload.bin` | same | packs; D7 requires a **transit warning** (an unsealed payload is readable by anyone holding the tag; for a key-less path the preimage alone spends) and a warning when the flag is given with no preimage/phrase record, or when a preimage's digest matches no `hash:` record | they also pass `--allow-world-readable` / redirect stdout → a bearer secret at 0644 | secret-handling, non-gating; *documentation only* |
| 5 | tap the tag | an unsealed payload holding a bearer secret | **today the device makes it inert**: `sysw/classify.go:126` is `err == nil && !codex32.IsPreimage(c)`, so a preimage string is `ClassUnknown`. A new class is required for D3 to work at all | that class's `IsSecret()` decides whether `syswLoadWarnings` prints **"A SECRET is stored unencrypted in flash."** at LOAD (`gui/sysw_load.go:261-285`, keyed on `Class.IsSecret()` at `sysw/record.go:64` via `syswFlags`) | **kept → Q1 (secrecy) and its blast radius** |
| 6 | door | payload loaded | the door counts keys/seeds/inert (`gui/composer_door.go:37`); D4 adds a "Hashlock plates" route | a preimage that stayed `ClassUnknown` shows only in "N payload records were not understood" | **kept → Q6** |
| 7 | Hashlock plates flow | preimage/phrase records, **no composition** | cut without a composition; header = digest, the payload's `hash:` position **when matched**, template id **when the payload carries one** (D4) | the payload carries a preimage whose digest matches no `hash:` record → nothing on the device can locate it | **kept → Q6** |
| 8 | cut | blanks | a bundle of preimage plates only | how many preimage records may one payload carry, and does the flow cut them as one bundle or one at a time | **kept → Q12** |
| 9 | later: use that preimage in a policy | the same payload | D4 lets `Which hash?` list payload preimage/phrase records as pickable digests, so the same preimage seats into a path | the operator then reaches Done and the composer offers to cut the preimage **again** | **kept → Q2** |

### Walked, not kept

- **argv leakage of a plate string** (`me <plate>` echoed by clap) — already F-476, secret-handling, non-gating. *Documentation only.*
- **`--pack-preimage` given with `--expect`** — `--expect`'s vocabulary is descriptor/cosigner/transaction/mnemonic/secret (`main.rs`, Pack `expect`); a preimage satisfies none, and adding a kind that cannot be satisfied is the exact trap that doc comment already refuses. *Not our concern.*
- **`now:` auto-append** — the rule keys on `key:`/`hash:` (`SPEC_wallet_policy_composer` §6a); a preimage-only payload is not a composer payload in that sense and correctly gets no bound. *Default.*

---

## J3 — a payload with a phrase record: the 10 s derivation, and where the phrase becomes visible

### The walk

| # | step | in hand | what happens | what ELSE → what happens | class |
| --- | --- | --- | --- | --- | --- |
| 1 | tap | payload with one or more phrase-and-method records | the record is admitted at `progWalletPolicy` only (the pattern `ClassKey`/`ClassHash`/`ClassNow` already set, `gui/sysw_admit.go:26` map) | admitted at Password or Text as well → a hashlock phrase reaches the passphrase flow | **kept → Q11** |
| 2 | door | N records | the door counts. A phrase record's **digest is not known** until derivation runs | the door shows a count but the composer cannot show digests | **kept → Q7** |
| 3 | `Which hash?` | the row list | D4 says payload preimage/phrase records are **pickable digests**. A `hardened` phrase costs ≈10 s each; three records = ≈30 s before a list can be drawn | derive eagerly at load (a 30 s stall with no context) vs. lazily on pick (a row that cannot show a digest) vs. row shows the method and derives after the pick | **kept → Q7** |
| 4 | derivation | a phrase the operator never typed and may not have chosen | the `Deriving` countdown (H2 §4.4). Back abandons and assigns nothing | Back mid-derivation with a payload record → returns to a row that still shows no digest | folded into **Q7** |
| 5 | the plate | phrase + method to be engraved | D1's phrase form requires the device to **display** the phrase — at the D6 review, on the plate preview, or both | the phrase is on screen in a room with other people; the device cannot know whose phrase it is | **kept → Q13** |
| 6 | reconcile | digest on screen | H5 §1's reconcile screen says "run `ms hashlock` with **this phrase** and method on the host" — but for a payload phrase the host already has it | the instruction is a no-op for this provenance and reads as a task | **kept → Q10 (same family: provenance-blind copy)** |

### Walked, not kept

- **A phrase record whose bytes fail `ValidatePhrase`** (non-printable, >100 chars, ms1-shaped, 64-hex) — the host refuses it at pack by the same rule (`hashlock_phrase.rs`), and the device leaves an unclassifiable record inert. Symmetric, already the pattern. *Refusal, already specified by D3's lockstep.*
- **A phrase record whose method spelling is unknown to this firmware** — a records-class validation question the author's §2 owns; on the device it is `ClassUnknown` and inert by the shipped contract. *Refusal, already the pattern.*

---

## J4 — mistakes

### The walk

| # | mistake | what the operator has | what happens under D1-D9 | class |
| --- | --- | --- | --- | --- |
| 1 | **wrong method chosen for a payload phrase** | a payload phrase record that says `sha256`; the operator picks `hardened` on the device (or the reverse) | the digest differs from every `hash:` record; H2 §4.5's relation line says "no `hash:` record in the payload has this digest" — a warning, not a refusal | shipped warning; **but see Q7**: if the row derives from the record's own method there is no pick to get wrong, and if it does not, the method line on the plate can disagree with the record |
| 2 | **a preimage record whose digest matches no path** | payload + composition | D6 says the composer's review lists it as "not on any path, will not be cut". The **mirror case** — a device-derived phrase whose path was re-hashed or removed — is not covered (J1 step 7) | **kept → Q4** |
| 3 | **two hashlocks in one wallet, plates mixed up** | two preimage plates, two paths | D4 puts the **path number** in the composer-native header and the **payload `hash:` position** in the payload-flow header. Two plates from the two routes carry different locators; `first8..last8` is the only common token | **kept → Q6** |
| 4 | **a declined plate at review** | the D6 review screen | undefined by the decisions: does declining drop that plate from the bundle, abort the bundle, or cut the rest? The composer then cuts an md1 whose path needs a preimage that reached no plate | **kept → Q2** |
| 5 | **power loss / Back mid-bundle** | some plates cut | `bundleAbortWarning` says "a partial bundle can't be used"; `composerState` is RAM, so the phrase and preimage are gone with it. If the md1 plates were cut first, the wallet is now **unrecoverable, not merely partial** — the abort copy does not distinguish | **kept → Q1** |
| 6 | **the plate string typed into Free Text** | the ms1 string on a keyboard | D8: WARN naming what it looks like, then cut as typed. The free-text plate has **no title band, no `NOT A SEED` footer** and no dedicated layout — a future reader sees a bare ms1 string. `ftWarnQR` already warns that a QR is a camera-readable copy (`gui/freetext_flow.go:1207-1217`) | **kept → Q14** |
| 7 | **the phrase typed into Free Text** | the phrase, not ms1-shaped | nothing fires: `IsMS1Shaped` cannot see a phrase, and no predicate can | *not our concern* — the device cannot tell |
| 8 | **the preimage plate cut, then the policy rebuilt with a different phrase** | two plates, one live wallet | the digest on the stale plate matches no live path; nothing on the device or the plate expires | *documentation only* — the header's digest is the discriminator and it already differs |

---

## J5 — the template id: where it comes from, what the operator does with it

### The walk

| # | question | measured answer at `fb0dd04` / `07a92e9b` |
| --- | --- | --- |
| 1 | what ids exist? | **Template-ID** and **Policy-ID**, 32 hex each; **mk1 stub (template)** and **mk1 stub (policy)**, 8 hex each (`SPEC_wallet_policy_composer` §7c, labels literal); **csid**, a 20-bit chunk-set id derived from the 16-byte encoding id (`md/identity.go:27-31`, rendered `SET %05X`, `gui/transaction.go:388`) |
| 2 | what does D4 put on each header? | composer-native: path number, digest, **policy csid**. Payload flow: digest, the payload's `hash:` position when matched, **template id when the payload carries one** — two *different* identifier families for the same locating job |
| 3 | can the payload carry a template id today? | **No.** The reserved prefixes are `text:`, `pass:`, `tx:` (`sysw/record.go:14-21`) plus `key:`/`hash:`/`now:` (`:52-54`). None carries an id. The only route is an md1 record in the same payload, from which the device could compute one |
| 4 | is csid strong enough? | 20 bits. It identifies a chunk set, not a policy, and it is **not** what §7c teaches the operator to write down |
| 5 | is the template id stable? | §7c: key-independent and origin-invariant but **NOT shape-invariant** — the wrapper, the path list, every lock operand and **every hash digest** enter it. Re-hashing a path changes the id on the stub screen after the plate was cut |
| 6 | what does the operator DO with it a year later? | match this preimage plate to the right policy plates. The md1 plates carry the id the operator was taught (§7c); a preimage plate carrying a *csid* cannot be matched to them by eye |

**Kept → Q6.** Also feeds Q8 (the md1 plates carry no reciprocal mark).

---

## §6. The questions

Most consequential first. Each states the journey step that produced it, the
options, and a one-line recommendation. **None is answered here.**

---

### Q1 — Where does the preimage plate sit in the cut order, and what does an abort between the md1 and it say?

*From J1 steps 10-11, J4 mistake 5.* Today `composerEngraveStep` puts secrets
**first** (`cards = append(secrets, cards...)`) and `bundleAbortWarning`'s text
distinguishes only "carries a secret" from "does not"
(`bundleSetCarriesASecret`, `gui/bundle_flow.go:690-694`). Under D9 the phrase
and preimage live only in RAM for the composition's lifetime, so an abort or a
power loss **after** the md1 plates and **before** the preimage plate leaves a
funded-shaped wallet whose spending secret no longer exists anywhere. That is a
different failure from "partial backup", and the shipped copy calls it the same
thing.

- **(a)** Cut the preimage plate **first**, like the ms1 secret cards, and leave the abort copy alone.
- **(b)** Keep it last but give the abort a third arm: "the phrase for path N is not on this device and is now gone; do not fund this wallet."
- **(c)** Both: cut first *and* give the abort the third arm.

*Recommendation:* (c) — ordering removes the window and the copy covers the window that ordering cannot (a blank runs out mid-preimage-plate).

---

### Q2 — What is the review's contract when a preimage plate is declined, and does the census count it as part of the backup?

*From J1 step 9, J4 mistake 4, J2 step 9.* Two shipped facts collide. The census
line is "This engraves N plates … a set is only a backup when all of it exists"
(`gui/multisig_build_census.go:66-72`), and S6b **deliberately forbids** the
passphrase plate from entering `plan` or the inventory "because either would tell
a reader it travels WITH the set" (`gui/multisig_build_census.go:88-93`). A
preimage plate is the same shape of artifact — it must not travel with the
descriptor plates — yet D4/D6 cut it *inside* the composer's bundle.

- **(a)** A preimage plate is a `bundleCard` like any other: it enters `plan`, the census, and the "all of it exists" claim.
- **(b)** It follows the passphrase-plate precedent: cut in the same run, counted in its own line, excluded from the completeness claim, with the census saying it must be stored apart.
- **(c)** It is cut in a separate run entirely (the composer hands off to the Hashlock plates flow at Done).

*Recommendation:* (b) — it is the one shape the constellation has already ruled on for a secret that must not travel with the set, and it keeps D6's "cut at Done" intact.

---

### Q3 — Does the preimage plate go through `validateMdmkStrings`, which would offer a QR of the ms1 string?

*From J1 step 11.* `bundleEngrave` builds every plate through
`validateMdmkStrings`, and for a **single-string card** it offers `TEXT + QR /
TEXT ONLY / QR ONLY` with the QR encoding **that string**
(`gui/gui.go:2634-2643`). The preimage plate string is a single string. So the
shipped machinery would put a QR of the ms1 preimage on the plate — which **D1
forbids** ("a QR … ALWAYS encodes the phrase and method as plain text, never the
ms1 string"), and which D6 has already asked about once at the review screen.
D5's dedicated layout is not a `backup.Text` at all, so this is not merely a copy
question.

- **(a)** The preimage plate is a new card kind that **bypasses** `validateMdmkStrings` entirely, rendered by its own layout function (the `backup.Passphrase` pattern, `backup/passphrase.go:23-48`, which already carries `QR bool` opt-in, a 100-character cap and title/footer bands).
- **(b)** It stays a `bundleCard` and `validateMdmkStrings` grows a kind switch that suppresses the QR variants for it.
- **(c)** It stays a `bundleCard` and the QR variants are left in place, with D1 reinterpreted as being about the *phrase* form only.

*Recommendation:* (a) — `backup.Passphrase` is the exact precedent (verbatim text, opt-in QR, 100-char cap, marked bands) and (c) contradicts a fixed decision.

---

### Q4 — What happens to a device-derived preimage whose path no longer carries its digest?

*From J1 step 7, J4 mistake 2.* D6 gives the review a rule for a **payload**
preimage that matches no path ("not on any path, will not be cut"). The mirror
case is unhandled: the operator types a phrase, HOLDs, then re-opens `Which
hash?` and picks a payload row, or removes the path. `phraseDigests` **never
deletes** — deliberately, so §8h stays correct (`gui/composer_state.go:46-50`) —
so a naive review would offer a plate for a phrase the policy no longer uses.

- **(a)** Symmetry: apply D6's rule to both provenances — any retained preimage whose digest matches no current path is listed as "not on any path, will not be cut".
- **(b)** Device-derived preimages are offered whenever they were derived here, on the grounds the operator may still want the plate.
- **(c)** Retained material is dropped the moment no path carries its digest (a state deletion, which is what `phraseDigests`'s comment refuses for §8h's sake).

*Recommendation:* (a) — it reuses one rule for both, and (c) reintroduces exactly the staleness the `phraseDigests` comment was written to avoid.

---

### Q5 — Does the QR toggle carry a warning, and does the plate say the QR is the secret?

*From J1 step 13.* The **SH2 has no camera**, so nothing on the device can ever
read this QR back — the reader is always a phone or a host. `ms hashlock` does
not parse the D2 text (D2 defers that as a follow-on), so the scan produces text
the operator must **retype**. Meanwhile the free-text plate already warns "The QR
makes the text readable by any camera. A photograph of the plate is a copy of the
text." (`gui/freetext_flow.go:1207-1217`) — and here the text is the value that
spends the path.

- **(a)** The QR toggle carries an `ftWarnQR`-shaped confirm naming what a photograph of this plate is.
- **(b)** No warning at the toggle; the plate's `NOT A SEED` band and the D6 review are held to be enough.
- **(c)** Warning at the toggle **and** a plate footer stating the QR is the secret.

*Recommendation:* (a) — the sibling warning already exists for strictly less dangerous content, and (c) spends band budget the header fields need.

---

### Q6 — Which identifier locates a preimage plate, and where does the payload flow get one?

*From J5, J2 steps 6-7, J4 mistake 3.* D4 names **policy csid** for the
composer-native header and **template id** for the payload flow. They are
different families: csid is 20 bits derived from the encoding id
(`md/identity.go:27-31`), while §7c teaches the operator a 32-hex `Template-ID`
and an 8-hex `mk1 stub`. **No payload record carries an id today** (`sysw/record.go:14-21,52-54`),
so "template id when the payload carries one" has no source unless the payload
also holds an md1 record. And the template id is **not shape-invariant** — every
hash digest enters it (§7c) — so it moves after the plate is cut if the shape is
edited.

- **(a)** One field for both routes: the 8-hex `mk1 stub (policy)` the operator was already taught to write down, present when the device can compute it and omitted otherwise.
- **(b)** Keep two fields as D4 states, and define the payload flow's source as "an md1 record in the same payload, else omitted".
- **(c)** No id at all on the plate; the digest is the only locator, and the operator matches by `first8..last8` against the md1.

*Recommendation:* (a) — one label that already exists in the operator's notebook beats two that cannot be compared to each other, and the digest stays the fallback.

---

### Q7 — When is a payload phrase record derived, and what does its row say before it is?

*From J1 step 2, J3 steps 2-4, J4 mistake 1.* D4 makes payload preimage/phrase
records "pickable digests" on `Which hash?`. A preimage record's digest is one
SHA-256 away; a **phrase** record's costs ≈10 s of PBKDF2 each (`hashlock.Iterations = 100000`),
and the row cannot show `hash <i> <first8>..<last8>` until it has run. Three such
records is ≈30 s before a list can be drawn.

- **(a)** Derive eagerly at load, behind the existing `Deriving` countdown, once per phrase record.
- **(b)** Derive lazily on pick; the row shows the method and a marker instead of a digest, and the digest appears at the confirm modal.
- **(c)** Phrase records are **not** offered on `Which hash?` at all — they reach only the Hashlock plates flow, and a digest for a policy comes from a `hash:` record as it does today.

*Recommendation:* (b) — it keeps the ≈10 s cost on an action the operator chose, and the confirm modal is already where the digest is reconciled.

---

### Q8 — Do the md1 policy plates say a preimage is required?

*From J1 step 14.* A year later, the reader has the descriptor plates and no
preimage plate. Nothing on those plates says one exists; the composer prints
**no restore document** (unlike `multisigRestoreDocFlow`); §8h is a build-time
screen. This is F-132 ("the hashlock preimage is required to spend, absent from
the backup, and unmentioned by it", `design/FOLLOWUPS.md:4298`) in its original
form, and H6 is the cycle that could close it. The mechanism already exists:
`bundleEngrave` takes `markTitle, markFooter` and marks every non-ms1 plate,
which single-sig uses for `"PASSWORD REQUIRED"` (`gui/singlesig.go:371`,
`bundlePlateMark` at `gui/bundle_flow.go:573`).

- **(a)** Mark the composer's md1 plates `PREIMAGE REQUIRED` when any path carries a hash, reusing the single-sig marking parameters unchanged.
- **(b)** No plate marking; close F-132 with a census line and the §8h screen only.
- **(c)** Marking plus a footer carrying the count of hash-gated paths.

*Recommendation:* (a) — the parameter, the predicate shape and the precedent all exist, and it is the one artifact the year-later reader actually holds.

---

### Q9 — What exactly is retained, for how long, and what clears it?

*From J1 step 5.* D9 accepts retention "for the composition's lifetime", but
`gui/composer_hashlock.go:19-20` currently states the opposite as a normative
property ("the preimage lives on the stack here and is dropped when this function
returns (L7, L15)"), and `composerState` holds no phrase field
(`gui/composer_state.go:26-79`). H2 §4.4's "a power loss ends the composition"
and C14's `defer reg.scrub()` at flow entry (`gui/multisig_build.go:290-291`) are
the shipped shapes for this. The question is not *whether* to retain — that is
fixed — but *what the state model is*, because Q1 and Q4 both read it.

- **(a)** A `map[[32]byte]{phrase, method, preimage}` beside `phraseDigests`, scrubbed by the same flow-entry `defer` the seed registry uses.
- **(b)** A per-path field on the composition, discarded with the path (which makes Q4 answer itself, and loses a phrase the operator may want to re-cut).
- **(c)** Retain the **phrase and method only**, re-deriving the preimage at cut time (≈10 s per plate, and a second derivation is a second chance to disagree).

*Recommendation:* (a) — it matches `phraseDigests`' keying, inherits C14's scrub-by-construction, and leaves Q4 free to be decided on its own merits.

---

### Q10 — Does the copy that says the preimage is "not on these plates" change when it now is?

*From J1 step 8, J3 step 6.* Two shipped strings become false or hollow under H6.
`composerCopyHashEveryPath` (`gui/composer_copy.go:169-173`) says the preimage
"is not on this device and not on these plates … Back up every preimage
separately" — but with H6 it is on a plate in this very bundle. And H5 §1's
reconcile screen tells the operator to "run `ms hashlock` with **this phrase**
and method on the host and check the digest matches", which is a no-op when the
phrase came *from* the host in the payload. Both are pinned by
`TestComposerCopyIsVerbatimFromTheSpec`, so changing either is a spec fold.

- **(a)** Make both provenance-aware: §8h gains a "cut in this set" form when a preimage plate is in the bundle; the reconcile screen is suppressed for payload-delivered phrases.
- **(b)** Change §8h only; leave the reconcile screen unconditional (a redundant check is cheap).
- **(c)** Change neither; add one census line stating the preimage plate is in this set.

*Recommendation:* (a) — a banner that is false on the screen where the backup burden is defined is the exact defect class F-480 and F-487 were filed for.

---

### Q11 — Where may a phrase-and-method record be admitted, and what stops it reaching the passphrase flow?

*From J2 step 2, J3 step 1.* `key:`/`hash:`/`now:` are admitted at
`progWalletPolicy` **alone** and the comment says why (`gui/sysw_admit.go:26-51`).
A hashlock phrase is a bearer secret of a different kind from a BIP-39
passphrase, and the two must never be interchanged — the whole terminology ruling
exists for that reason. An operator who packs a phrase as `pass:` gets it offered
to Password; as `text:` gets it engraved as free text.

- **(a)** One new class admitted at `progWalletPolicy` only, exactly as the three composer classes are, with an explicit row saying it is not a passphrase.
- **(b)** Also admit it at Password, so an operator who wants a phrase plate can reach the shipped passphrase plate layout.
- **(c)** Admit at `progWalletPolicy` and, additionally, refuse the record at pack time when it duplicates a `pass:` record's bytes.

*Recommendation:* (a) — (b) is exactly the confusion the terminology ruling forbids, and (c) is a host check with no device counterpart.

---

### Q12 — How many preimage plates may one run cut, and does the flow bundle them?

*From J2 step 8, J1 step 9.* A payload may carry several preimage/phrase records;
a composition may hash several paths. `bundlePlatePlan` numbers plates "Card X of
Y | Plate P of Q" across the whole set (`gui/bundle_flow.go:466-483`), so several
preimage plates in one run read as one set — which, for artifacts that must be
stored **apart from each other** (different paths, possibly different custodians),
says the wrong thing.

- **(a)** No cap; they are cards in the bundle and the census names each by path/record index.
- **(b)** No cap, but each preimage plate is its own single-plate card with a line saying these are stored separately from each other and from the policy plates.
- **(c)** One preimage plate per run; a second requires re-entering the flow.

*Recommendation:* (b) — a cap punishes the two-hashlock wallet the composer already supports, and the separation claim is what the census is for.

---

### Q13 — Is a payload-delivered phrase displayed on screen before it is cut?

*From J3 step 5.* D1's phrase form must engrave the phrase, so the D6 review has
to show it (or show a plate preview containing it). For a **device-typed** phrase
the operator has just typed it and the keyboard's `show` key already reveals it.
For a **payload-delivered** phrase the device is displaying a secret the operator
never typed, possibly does not own, and did not ask to see, on a screen that in
J2's own journey is being read in whatever room the machine lives in.

- **(a)** Show it, unmasked, as the review must — the operator is about to put it on steel in the same room.
- **(b)** Mask by default with a `show` toggle, matching the phrase keyboard's affordance.
- **(c)** Show a character count and the method only; the phrase appears only in the plate preview immediately before the cut.

*Recommendation:* (b) — it reuses an affordance the operator has already met on the phrase screen, and it is secret-handling (non-gating), so it should not cost a round.

---

### Q14 — Does the free-text warning offer the route that produces a proper plate?

*From J4 mistake 6.* D8 fixes the warning's existence and its "never refuse"
posture. What it does not fix is the *destination*: after the warning, the
operator cuts an ms1 preimage on a **free-text plate** — no `HASHLOCK PREIMAGE`
title, no `NOT A SEED` footer, no dedicated layout, and (if they enabled the QR)
a QR of the ms1 string, which is what D1 forbids on the real plate. Two plates of
the same secret with different legibility guarantees is the wrong outcome, and it
is worse than silence because the operator believes they have done the thing.

- **(a)** The warning names the Hashlock plates flow as the route that produces a marked plate, and still cuts as typed if they proceed.
- **(b)** The warning describes what the string is and stops there (D8's minimum).
- **(c)** The warning offers a choice: continue in Free Text, or switch to the Hashlock plates flow with this string pre-loaded.

*Recommendation:* (a) — it is documentation-only, costs one sentence, and (c) is a cross-program hand-off with its own state contract.

---

## §7. Two notes for the author, not questions

1. **Q3 and Q2 are the two places where a "dedicated layout" (D5) collides with
   shipped bundle machinery.** Neither is a copy question. `backup.Passphrase`
   (`backup/passphrase.go:23-48`) already solves the same problem — verbatim text
   up to 100 characters, opt-in QR, title/footer bands, a visible space mark for
   a secret whose spaces matter — and a `SpaceMark`-equivalent is worth
   considering for a phrase plate, since "hunter2 " and "hunter2" derive different
   preimages exactly as they derive different wallets.

2. **`MaxTitleLen` is 18** (`backup/backup.go:71`), and `HASHLOCK PREIMAGE` is 17
   characters, so D5's title band fits the shipped cap; `NOT A SEED` is 10. The
   binding geometry constraint is therefore the **header rows plus the QR
   envelope**, not the title or the 100-character phrase. `backup.FontSizes`
   descends to 3.0 mm on an 85 mm plate with 3 mm outer margins
   (`backup/backup.go:71-78`), which is where D5's "fits at the smallest rung"
   gate should be measured — with a real render, not arithmetic.
