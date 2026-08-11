# R0 round 3 — re-review of the third fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `e3ca4db` (the fold).
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round2.md` at `5cec977`,
  and the isolated fold diff `5cec977..e3ca4db` — measured: **86 changed lines in one file**
  (`git diff --stat`: 1 file, 261 insertions / 10 deletions across the two commits; the spec's own
  hunk is 86 lines).
- **Questions answered:** (1) did the fold fix each of round 2's 9 findings; (2) did the fold
  introduce a new defect. **This is not a fresh audit.** No section the fold did not touch was
  reopened except where the fold made it wrong.
- **The AEAD-scope ruling is treated as a DECISION, not a proposal.** It is not argued against. It
  **is** flagged where it contradicts other text in this spec, a sibling spec, or the code — which
  the brief explicitly asks for, and which it does, twice.
- **Attack #1 in the brief is MOOT and its finding is WITHDRAWN.** A second operator ruling arrived
  mid-review: *the cliff is 5 or more BIP-39 words, regardless of any other consideration —
  `abandon` five times is above it.* Strength stops being an entropy estimate that must be recorded
  and trusted, and becomes a count both sides compute locally from the passphrase string. There is
  no header field, so nothing is attacker-controlled, so attack #1 has no target. The finding this
  review had written against it is deleted, not softened. **What replaces it is sharper and is
  finding 2 below:** the ruling redefines the cliff, and *five* places in the spec — including two
  paragraphs this same fold added — still define it as strength. That is a genuine
  host-seals-what-the-device-refuses split, not a labelling question.
- **Machine-checked before any judgement was formed** (values pasted, never described):
  - `scripts/spec-check.py design/SPEC_systemwide_payloads.md` → **exit 0**, 23 citations resolved,
    22 tests numbered 1..22 without gaps, "all invariants hold".
  - `bip39/wordlist.txt`: **2048 words; longest = 8 characters; 88 words at 8; zero words longer.**
    File SHA-256 `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda` — the official
    BIP-39 English list. So 24 × 8 + 23 = **215** and 12 × 8 + 11 = **107**. **The 215-byte cap's
    arithmetic is CORRECT.** (Its *measurement basis* is not stated — finding 5.)
  - `gui/unlock_kdf.go:168` = `if !m.Valid() {`; `:359` = `if !isMnemonicComplete(m) || !m.Valid() {`.
    Both §2.2 item 8 citations are exact.
  - `bip39/bip39.go` `Valid()` opens `if len(m)%3 != 0 { return false }` — §2.2 item 8's claim holds.
  - `seal/wire.go` `Header` = `{Iterations uint32, Salt [16]byte, IV [12]byte, PubLen uint32,
    CtLen uint32}`, `HeaderLen = 52`, reserved byte MUST be 0. **There is no passphrase-mode or
    word-count field, and no spare byte for one.**
  - `passphrase.ValidatePassphrase` enforces `r < 0x20 || r > 0x7E` **and then** `n > MaxLen`
    (`MaxLen = 100`) — one function, both constraints.
  - **`grep -x` against `bip39/wordlist.txt`: `correct` IN-LIST, `horse` IN-LIST, `battery` NOT,
    `staple` NOT.** §6.2.1's own canonical user-supplied example, `correct horse battery staple`,
    is **2 BIP-39 words out of 4 tokens** — which is what makes the new cliff's counting rule
    ambiguous rather than obvious (finding 2).
  - `grep inputWordsFlow design/SPEC_systemwide_payloads.md` → **no match. The spec never mentions
    it.** `gui/gui.go:727` is its definition; `refreshCands`/`LastWordCandidates` at `:758`–`:768`
    are inside it; `unlockPassphraseFlow` merely *calls* it at `unlock_kdf.go:160`, alongside
    `derive_xpub.go:90`, `gui.go:2346`, `gui.go:2445` and `seedxor_polish.go:52`.
  - **Gate mutation-tested** (three mutants, results in finding 7).

---

## Part 1 — did the fold fix each round-2 finding?

| round-2 finding | status | reason |
| --- | --- | --- |
| **R2-C1** — §5.4's "both container variants show the digest" + test 9 | **FIXED** | §5.4 l.522 now reads "**wherever a digest exists**, it is shown"; test 9 is scoped to `pub_len > 0`, says "without a satisfied `compared`", and names the 9/15 pair explicitly. Both sites round 2 identified, both replaced rather than annotated. |
| **R2-I1** — §2.2 lists no device-side entry work; the unlock flow's newness unstated | **PARTIAL** | Items 8 and 9 landed and name both `!m.Valid()` sites correctly. But the fold forbids reusing `unlockPassphraseFlow` — which is **not** where the mask that breaks 15/16 draws lives — and still names no carrier for the per-invocation gate through the shared `inputWordsFlow`, which round 2's consequence paragraph named. See **[IMPORTANT] 3**. |
| **R2-I2** — §8a routes the passphrase through Go strings; its mechanism sentence is wrong | **PARTIAL** | The residue horn is answered well and honestly by the new §6.2.2a. The **mechanism horn is untouched**: §8a l.68 still says "Both feed the same `seal.NormalisePassphrase`", which is the call `passphraseBytes` exists to avoid and which contradicts §6.2.2's buffer rule, §6.2.2a's own scoping, and test 21. See **[IMPORTANT] 4**. |
| **R2-I3** — the length cap stated as an inequality; user-supplied unbounded | **PARTIAL** | The table row is now "**exactly 215 bytes**, host and device" and explicitly binds the user-supplied mode — the substance is fixed. Round 2 named **two** sites; the normative blockquote at l.821 still says "(≥ 215 bytes)", and the row never says whether 215 is measured on raw entry or normalised output, which differ only for the mode it newly claims to bound. See **[MINOR] 5** and **[NIT] 9**. |
| **R2-I4** — restoring user-supplied overrules EPD§2.2/§8's MUST NOT, unmarked | **PARTIAL** | The headline is fixed: decision 8 l.55–63 marks the overrule, quotes `passphrase.rs`, and points at §6.2.1's 0-bit pricing. The finding's second half — nothing in §2.2, §9 or §10 schedules amending EPD§2.2/§8 and `passphrase.rs`'s module doc — is still true. See **[MINOR] 8**. *The new fold text also rests on §6.2.1 pricing user-supplied at 0 bits, which the incoming cliff ruling removes — see **[CRITICAL] 2**.* |
| **R2-I5** — "a successful AEAD open sets `compared`" is unscoped | **PARTIAL** | The `compared` **table** is now correctly three-rowed and the operator's ruling is recorded with its consequence. But §5.4.1's prose four lines below (l.633–639: "**Opening it is the proof**… strictly stronger") and **test 20** (l.1063) both still state the unconditional rule verbatim, and §3.2.1 l.220 still describes `compared` as the operator route only. See **[CRITICAL] 1**. |
| **R2-M1** — §3.3.2a puts the `compared` gate before classification | **FIXED** | The new l.323–327 makes the order normative, rules that §5.4.1 governs, and explicitly **withdraws** the offending clause. Annotation rather than excision, but the withdrawal is unambiguous and consistent with §3.2.1 and §5.4.1. |
| **R2-M2** — §9's O2 row records the reversed decision | **FIXED** | The row is rewritten: the `!m.Valid()` enforcement is added, "it removed a passphrase mode" is replaced with "It did NOT remove a passphrase mode", and it forwards to §2.2 item 8. |
| **R2-N1** — §6.2.2 row 1 cites the function whose cap row 2 forbids | **PARTIAL** | Row 1 gained a good explanation of *why* the two are not in tension ("a fact about steel, not about typing"). It still presents `passphrase.ValidatePassphrase` as "the device's real constraint" and never says **do not call it** — and calling it is the natural transcription, which silently imports `MaxLen = 100` onto a 107-byte 12-word passphrase. See **[NIT] 9**. |

**Score: 3 FIXED, 6 PARTIAL, 0 NOT FIXED.**

Five of the six partials are the same shape rounds 1 and 2 both named, now for the third round: **the
site the finding pointed at was corrected and the same rule was left standing elsewhere.** R2-I5's
residue is the serious one — the fold deleted a rule from a normative table and left it asserted, in
bold, in the prose immediately below and in a named test.

---

## Part 2 — new defects

### [CRITICAL] 1. §5.4.1's prose and test 20 still mandate the unconditional AEAD rule that the fold's own table just deleted — so the spec states the sub-cliff bypass and its correction three inches apart, and a transcribing implementer builds the bypass

**Where:** §5.4.1 l.633–639 and §8.3 test 20 (l.1063–1065), against §5.4.1's own table l.611–615 and
the ruling paragraph l.617–631. Also §3.2.1 l.220.

**Consequence:** the fold's table now has three rows and the third says a sub-cliff or user-supplied
open sets `compared` = **NO**. Four lines later, unedited:

> A secrets-only sealed payload (`pub_len == 0`) displays no digest — EPD§6.6's own rule — so there
> is nothing for the operator to compare. **Opening it is the proof.** The AEAD tag authenticates
> the whole payload under a key only the passphrase derives, which is a *cryptographic* guarantee
> and strictly stronger than a human reading sixteen hex digits off a screen. Requiring a comparison
> that cannot happen would have made the sealed variant's principal use impossible.

That paragraph does not merely restate the deleted rule — it **argues against the fold**, on the
exact ground the fold's own ruling paragraph rejects ("That is true only while the key is strong").
It is the single most emphatic sentence in the section, it is bolded, and it is unqualified.

Test 20 is worse, because it is the artefact an implementer executes: *"A secrets-only sealed payload
is consumable: opening it sets `compared` with no digest shown."* No passphrase-strength
precondition. Written against a 2-word fixture — the length §6.1 prices at **42 seconds on one
GPU** — it passes **only if the implementation contains the bypass**, and it is a *named test in the
suite this gate green-lights*. That inverts the control: the test list mandates the defect rather
than merely permitting it.

The attack §5.4 l.559–561 spells out is then live in its stated form: an attacker who brute-forces
the passphrase produces a payload that opens cleanly, `compared` is set by the attack itself,
"the tag still verifies, and the operator engraves a steel backup of a wallet they do not control."

§3.2.1's store comment is a third stale site — `compared bool  the operator compared the digest for
THIS identity` — which has now been wrong across two consecutive folds that changed what `compared`
means.

**Why it is real:** measured, not inferred — `git diff 5cec977..e3ca4db` has **no hunk** at §5.4.1
l.633–639, at test 20, or at §3.2.1. The fold edited test 9 in the same commit and left test 20, the
one that carries the same rule. `grep -n "Opening it is the proof\|opening it sets"` returns l.634
and l.1063: two live sites, zero of them touched. This is R2-C1's exact failure mode reproduced by
the fold that fixed R2-C1.

**Suggested resolution:** delete l.633–639 outright — the ruling paragraph at l.617–631 already
covers everything true in it, and nothing in it survives the ruling. Amend test 20 to *"A
secrets-only sealed payload sealed with a passphrase **at or above the cliff** is consumable: opening
it sets `compared` with no digest shown. A sibling case with a **2-word** passphrase must NOT be
consumable — a test that omits the second half passes on the bypass."* Change §3.2.1 l.220 to *"the
payload was authenticated for THIS identity, by either §5.4.1 route."*

---

### [CRITICAL] 2. The new cliff ruling makes "above the cliff" a WORD COUNT, but §6.2.1 — the section §5.4.1 defers to for exactly this word — still defines it by MODE and by BITS, so `me` seals as above-cliff what the device refuses as below-cliff. Labelling the §5.4.1 row a speed bump does not close it

**Where:** §6.2.1 (l.761–800, especially the table l.771–774 and the transcription rule l.796–797)
against the incoming ruling; and §5.4.1 l.615 and l.621, decision 8 l.55–63, §6.2 l.751–754,
§6.1 l.734 and l.746, §3.2.1 l.222, §3.3.3 F2 (l.347), §5.6 l.705, §8.3 test 5 (l.1014).

**Consequence.** This is the direct answer to the controller's question 1: **the speed-bump label is
necessary and not sufficient**, because the thing that has to carry the label is not §5.4.1's row —
it is §6.2.1, and §6.2.1 is built the other way round.

§5.4.1's rule does not define the cliff; it *points* at one: *"whose passphrase is at or above the
cliff **(§6.2.1)**"*. §6.2.1 answers with a **mode** table — generated-N → `11 × N` bits;
user-supplied → **"treated as 0 bits. Not estimated."** → flag required **"always"** — and with a
transcription rule an implementer copies verbatim:

> **Secret content + (user-supplied OR no passphrase OR generated with `N < 5`) ⇒ `me` refuses
> without the explicit flag.**

Feed one input to both: a user-supplied passphrase of five wordlist words.

- **`me`, following the ruling:** 5 BIP-39 words ⇒ above the cliff ⇒ seals it **without**
  `--allow-weak`, and prints it as above the cliff.
- **The device, following §6.2.1 as written:** user-supplied ⇒ 0 bits ⇒ sub-cliff ⇒ §5.4.1's third
  row fires ⇒ `compared` is **NO**.

For a secrets-only payload — the sealed variant's main case, the one with no digest to fall back
on — that payload is **created successfully and is permanently unconsumable**, with the machine
reporting only "weakly protected". That is the R0-C4 shape, "the host seals what the device cannot
accept", now reached by two documents disagreeing about one word.

§5.4.1 l.615 states the collision in its own text: the row reads *"a successful AEAD open under a
**sub-cliff or user-supplied** passphrase"* — the `or user-supplied` clause is a mode test welded
into the row the ruling redefines by count, and it was added by this fold.

**Four more sites define the cliff as strength, and two are this fold's own new prose:**

| site | text | why the ruling breaks it |
| --- | --- | --- |
| decision 8, l.61 (**added by this fold**) | "§6.2.1 **prices the mode at 0 bits** so nothing downstream mistakes it for protection" | the R2-I4 fix's entire justification for permitting user-supplied is that §6.2.1 prices it at 0. Under the ruling §6.2.1 will not, and the overrule paragraph stops justifying the overrule |
| §5.4.1, l.621 (**added by this fold**) | "treats user-supplied as **0 bits** … A tag proves someone knew the passphrase; that is worth exactly what the passphrase is worth" | this is the ruling's own principle, and under the ruling it refutes the rule it was written to justify: `abandon` ×5 clears the cliff and is worth nothing. The speed-bump label must land **in this paragraph**, not only on the table row |
| §6.2, l.751 & l.754 | "**Below 5 words (55 bits)**" / "secret material is **protected by** less than the cliff" | the parenthetical *equates* count and entropy; the ruling severs them |
| §6.1, l.734 & l.746 | "the *shape* — a cliff between 4 and 5 words — … is **the only property the rule in §6.2 rests on**" | §6.1 is the entropy argument that licenses the cliff. Under the ruling §6.2 no longer rests on it, so §6.1 justifies a rule the spec no longer has |

And the flag's **name and screen text become false in the reassuring direction**: `weak`
(§3.2.1 l.222) and F2's *"this secret is weakly protected"* (l.347) now go **unset** for `abandon`
×5. An operator reads the absence of the warning as protection. That is precisely the over-claim
F-123 was filed against, which this spec invokes four separate times against other controls.

**The counting rule itself needs one more sentence, and the spec's own example shows why.** Measured:
`correct` and `horse` are in `bip39/wordlist.txt`; `battery` and `staple` are **not**. So §6.2.1's
canonical user-supplied example, `correct horse battery staple`, is 4 tokens of which 2 are BIP-39
words. "5 or more BIP-39 words" does not say whether that means **≥5 tokens, all in-list** or **≥5
in-list tokens among any number**. `correct horse battery staple abandon ability able` is 7 tokens,
5 in-list: above the cliff under the second reading, below under the first. Host and device
implementing different readings reproduce the same split this finding is about.

**Why it is real:** these are not the ruling's fault — the ruling is a clean simplification and it
does dissolve attack #1 completely. They are the *fold surface* the ruling lands on, and every one
of them is a site that says "cliff" or "0 bits" today. Two of the five were written by this same
fold, so folding the ruling without them repeats, for a fourth round, the exact pattern rounds 1, 2
and 3 have each found: the rule changed in one place and left standing elsewhere.

**Suggested resolution:** rewrite §6.2.1 as a property of the **string**, not of the mode — it is a
smaller section afterwards, and it closes §8a's keyboard question for free (a 12-word generated
passphrase counts 12 whichever keyboard typed it, so the keyboards stay interchangeable exactly as
§8a claims):

> **NORMATIVE.** Cliff strength is `wordCount(NormalisePassphrase(p))`, where `wordCount` is the
> number of space-separated tokens **that appear in `bip39/wordlist.txt`**; tokens not in the list
> count zero. At or above the cliff ⇔ `wordCount ≥ 5`. The mode is not an input, and host and device
> compute this identically from the passphrase alone — there is no header field and nothing for an
> attacker to flip.

Then: (1) **label it in §6.2.1 itself**, not only at §5.4.1 — *"this is a SPEED BUMP, not a security
boundary. `abandon` five times clears it and is worth ~0 bits. It exists to stop a one- or two-word
passphrase, not to certify a five-word one"*; (2) delete `or user-supplied` from §5.4.1 l.615 and
rewrite l.621's justification so it no longer asserts a principle the rule violates; (3) fix
decision 8 l.61 to rest on something the ruling preserves; (4) add the same caveat to §6.1 and §6.2
where they equate 5 words with 55 bits; (5) rename `weak` → `subCliff` and F2 → *"this secret's
passphrase is under 5 words"*, so neither claims a strength judgement; (6) restate test 5 in word
counts, and add a test that `abandon` ×5 is above the cliff — a test written from today's §6.2.1
asserts the opposite.

---

### [IMPORTANT] 3. §2.2 item 8 forbids reusing `unlockPassphraseFlow`, but the mask that makes 15 of 16 passphrases unopenable lives in `inputWordsFlow` — a different, shared function the spec never names, with no carrier for the per-invocation gate

**Where:** §2.2 item 8 (l.144–150) against §8b (l.74–78), §8c (l.99–104) and §3.1 (l.174–186);
`gui/gui.go:727`, `:758`–`:768`; `gui/unlock_kdf.go:160`.

**Consequence:** item 8 is precise about what must not be reused and names exactly the two guards it
enumerates. Traced in the fork, those two guards and the mask are in **different functions**:

- `unlockPassphraseFlow` (`gui/unlock_kdf.go:109`) holds `!m.Valid()` at `:168`, and
  `unlockAttemptOnce` holds it at `:359`. Item 8 correctly excludes both.
- **`refreshCands` — the mask — is a closure inside `inputWordsFlow` (`gui/gui.go:758`), which
  `unlockPassphraseFlow` merely calls at `:160`,** alongside `derive_xpub.go:90`, `gui.go:2346`,
  `gui.go:2445` and `seedxor_polish.go:52`.

So an implementer who writes a brand-new `syswUnlockFlow` — complying with item 8 **verbatim** — and
calls `inputWordsFlow` for the word entry (the obvious reuse: it is where the candidate refresh, key
masking and completion machinery live) hits `bip39.LastWordCandidates` on the final slot and gets
back exactly §8c's stated outcome: *"15 of every 16 uniformly generated 12-word passphrases would be
permanently unopenable."* That is R1-C2's Critical, reached by an implementer who did precisely what
the fold told them to do.

The spec also still owes the shape. §8b makes the gate "PER-INVOCATION" without saying what carries
the signal into a function five other call sites share — and §3.1 already **rejected** the obvious
answer for the structurally identical seam: *"a boolean can be passed wrongly and the wrong value
still compiles"*, resolved there with two named entry points. §8b gets neither the entry-point
treatment nor an explicit exemption.

**Why it is real:** `grep -n "inputWordsFlow" design/SPEC_systemwide_payloads.md` returns **nothing**.
The spec names `refreshCands` (§8c l.99) and `gui/gui.go:758` — the defect site — but never the
function that contains it, and item 8's prohibition therefore lands one level too shallow. Round 2's
finding stated both halves ("without saying what carries the per-invocation signal through
`inputWordsFlow` … and without naming the sites that enforce the gate outside `inputWordsFlow`"); the
fold did the second.

**Suggested resolution:** extend §2.2 item 8 to *"…and it may not call `inputWordsFlow`
(`gui/gui.go:727`) unmodified: `refreshCands` at `:758` applies the `LastWordCandidates` mask and is
shared with seed entry and SeedXOR."* Then state the carrier in §8b, following §3.1's precedent
rather than a boolean — e.g. `inputWordsFlow` gains a sibling `inputPassphraseWordsFlow`, or takes an
explicit `cands func(bip39.Mnemonic) []bip39.Word` the passphrase caller passes as `nil` — and add a
test-19 clause asserting that seed entry's mask is unaffected.

---

### [IMPORTANT] 4. §8a's "Both feed the same `seal.NormalisePassphrase`" survives the fold and contradicts §6.2.2's buffer rule, §6.2.2a's own scoping, and test 21 — the sentence deletes the buffer those three are about

**Where:** §8a l.68 against §6.2.2's blockquote (l.821–824), §6.2.2a l.828–831 and test 21 (l.1066);
`gui/unlock_kdf.go:186–194`, `seal/open.go:76`.

**Consequence:** §6.2.2a is a good, honest fold — but it scopes the accepted residue to *"the free-text
path"*, and its first sentence explicitly names `passphraseBytes` as "the exact shape … exists to
avoid", i.e. it assumes the **word path keeps the caller-owned buffer**. §8a says the opposite, in
the mechanism sentence round 2 flagged and the fold did not touch: *"Both feed the same
`seal.NormalisePassphrase`."*

Verified in the code: `NormalisePassphrase` (`seal/open.go:76`) is
`strings.ToLower(strings.Join(strings.Fields(s), " "))` — it **takes and returns a `string`**.
`passphraseBytes`' own doc comment (`gui/unlock_kdf.go:186–189`) says why it exists: *"`Mnemonic.String()`
produces byte-identical output …, but it produces a Go STRING, which cannot be zeroed. That is the
whole reason this exists."* An implementer transcribing §8a literally routes the word path through
`NormalisePassphrase`, deletes `passphraseBytes`, and then:

- §6.2.2's normative *"The buffer is allocated once at its maximum … and never regrows"* has **no
  subject** — there is no buffer;
- **test 21** — *"Assert on the buffer's identity, not on its contents"* — is unwritable, or written
  against a buffer that no longer exists and passes vacuously. A false PASS on the one hygiene
  requirement this spec makes normative;
- §6.2.2a's careful scoping collapses: residue was accepted for the free-text path, and the word path
  quietly joins it without anyone deciding that.

**Why it is real:** `git diff 5cec977..e3ca4db` has no hunk at §8a's mechanism sentence. Round 2's
finding had two named horns and its suggested resolution opened with the mechanism fix
("change §8a to 'both produce EPD§8.1's normalised form — the word path via a caller-owned `[]byte`
(`passphraseBytes`) …'"); the fold answered the second horn and left the first. The convergence §8a
actually rests on is fine and was already verified in round 2 — `gui/unlock_kdf_test.go:471` asserts
`string(passphraseBytes(m)) == seal.NormalisePassphrase(m.String())` — so only the *mechanism* clause
is wrong, and it is a one-sentence fix.

**Suggested resolution:** replace §8a l.68's clause with *"Both produce EPD§8.1's normalised form —
the word path via the caller-owned `[]byte` of `passphraseBytes` (`gui/unlock_kdf.go:194`), never via
`seal.NormalisePassphrase`; the free-text path via `seal.NormalisePassphrase`, whose residue §6.2.2a
accepts."* Then add one clause to §6.2.2a saying which path each rule binds, so the scoping is stated
where both rules can be read together.

---

### [MINOR] 5. "exactly 215 bytes, host and device" does not say whether 215 is measured on raw entry or on the normalised form — and they differ only for the user-supplied mode the row was rewritten to bound

**Where:** §6.2.2's `length cap` row (l.810) and its heading *"what is ENTERABLE"* (l.802), against
l.813 (*"`me` enforces the identical range and cap at creation"*) and the buffer blockquote (l.821),
which sizes a buffer holding the **normalised** form; `seal/open.go:76`.

**Consequence:** `NormalisePassphrase` collapses whitespace runs, so raw ≥ normalised for any input
with a double space. A user-supplied passphrase of 219 raw characters containing five double spaces
normalises to 214. If `me` measures the cap post-normalisation it accepts and seals it; if the device
measures it at entry the operator cannot type past character 215, silently truncates, and gets an
unopenable payload after a ~31 s KDF. That is the R0-C4 shape — "the host seals what the device
cannot accept" — surviving in the row rewritten to close it, for exactly the mode the rewrite added
("it bounds the **user-supplied** mode too, which otherwise had no derived bound at all").

**Why it is real:** the trigger is narrow (a >215-character passphrase with redundant whitespace) but
the row's stated purpose is to remove ambiguity between two independently-implemented sides, and it
leaves one measurement question open on the only mode where the two measurements differ. On the
generated word path the question cannot arise, which is why the 215 derivation reads unambiguously
and the gap is easy to miss.

**Suggested resolution:** add four words to the row — *"exactly 215 bytes **of the normalised form**,
host and device"* — and one sentence: *"The device must therefore normalise before applying the cap,
not at entry; entry itself is bounded by 215 × 2 to keep the field finite."*

---

### [MINOR] 6. The fold states "cannot be consumed at all" in §5.4.1 and never carries it to F2's screen text or to §8.3, so the operator meets a dead-end the machine describes as "weakly protected"

**Where:** §5.4.1 l.627–631 against §3.3.3 F2 (l.347), §3.2.1 l.222 and §8.3.

**Consequence:** the ruling's stated outcome is that a secrets-only sub-cliff payload is unusable. The
only screen the spec gives that operator is F2's *"this secret is weakly protected"* — which is now
false by understatement. Having unlocked successfully after ~31 s and seen their records listed, the
operator watches every program refuse them with no explanation, and the natural inference is a typo:
they retry the passphrase, at 31 s per attempt, forever. No named test covers the dead-end either, so
nothing in the suite proves the refusal is reachable, distinguishable, or explained.

**Why it is real:** the fold's own words are "Consequence, **stated rather than discovered**" — the
consequence is stated in §5.4.1 and propagated to none of the three sections that would show it to a
human or test it. The pattern is the one rounds 1–3 have each found. *Distinct from finding 2's F2
point: that one is about the flag's absence over-reassuring above the cliff; this one is about there
being no screen at all for the unusable state below it.*

**Suggested resolution:** add F5 to §3.3.3 — *"admitted class is secret, container is sealed
secrets-only, and the open did not qualify → **this payload cannot be used; re-seal it with 5 or
more generated words**"* — and a test 23 asserting the refusal is reached and names that reason
rather than a generic failure.

---

### [MINOR] 7. The new gate is green on two defects it names as its own purpose — demonstrated by mutation — and its docstring claims a citation check the code does not implement

**Where:** `scripts/spec-check.py` — `FORBIDDEN[3]` ("unconditional digest display", l.52–58),
`check_citations` (l.100–117), docstring l.28–29, `check_tests` l.122–123.

**Consequence:** three mutants were run against the committed gate. **Measured results:**

| mutant | what it does | gate |
| --- | --- | --- |
| **mut1** | replaces §5.4's "wherever a digest exists, it is shown" with *"**for every variant, sealed or not**, the digest is shown"* — R2-C1's exact claim, different words | **exit 0, "all invariants hold"** |
| **mut2** | rewrites §6's opening to "Two passphrase Modes in `me`" | exit 1 — caught, but by `REQUIRED["three modes"]`, **not** by `FORBIDDEN["mode count"]`, whose `two passphrase modes` alternative is case-sensitive and misses `Two passphrase modes` |
| **mut3** | redirects `gui/unlock_kdf.go:168` → `:12` (a line that exists and says something unrelated) | **exit 0, "23 citations resolved"** |

So the invariant written for round 2's Critical does not guard the claim — it guards one phrasing of
it — and the docstring's *"checked for existence **and for an expected substring where one is
declared**"* describes a mechanism that appears nowhere in the file: `check_citations` tests only
`ln > len(lines)`. A gate that overstates its coverage in the very paragraph headed "WHAT IT DOES NOT
COVER" is the failure that paragraph exists to prevent.

Two smaller points: `check_tests` l.122–123 computes `nums` through an `and` expression that discards
the first `findall`, and then never uses `nums` — dead code. And the fold introduced a **new normative
rule** (the AEAD scoping) in the commit after the gate landed, without adding an invariant for it —
the gate's own third blind spot, and the rule whose second site is finding 1. A single
`FORBIDDEN` line matching `Opening it is the proof|opening it sets \`compared\`` catches finding 1
outright; verified by construction against the live text.

**Why it is real:** measured, not argued — mut1 and mut3 exit 0. The gate is genuinely valuable and
its blind-spot paragraph is the right instinct; these are corrections to it, not an argument against
it.

**Suggested resolution:** (a) restate `FORBIDDEN[3]` as a claim-shaped pattern
(`(both|every|either)[^.\n]{0,30}variant[^.\n]{0,60}(digest|shown|display)`) and add case-insensitive
matching to the mode-count line; (b) add the missing AEAD invariant; (c) either implement the
declared substring check — a third tuple element per citation — or delete the claim from the
docstring; (d) delete the dead `nums`; (e) add a `--self-test` mode running the mutants above, so a
future fold cannot silently weaken an invariant.

---

### [MINOR] 8. Nothing schedules the EPD§2.2/§8 and `passphrase.rs` amendments that decision 8's overrule now requires

**Where:** decision 8 l.55–63 against §2.2 item 6, §9 and §10;
`crates/me-cli/src/seal/passphrase.rs`, `SPEC_encrypted_payload_delivery.md` §2.2 item 1 and §8.

**Consequence:** the overrule is now marked, which was round 2's headline ask. But EPD§2.2 item 1 and
EPD§8 still carry an unqualified MUST NOT, and `passphrase.rs`'s module doc still opens "GENERATED,
never user-supplied" — the Rust-primary normative record. Under this project's Rust-primary rule the
doc edit lands **first**, and §2.2 item 6 ("`me` passphrase modes and the cliff flag") does not name
it. A future reader of EPD meets a MUST NOT with no pointer to the overrule; a future reader of this
spec is not told the prohibition remains in force for `MNEMBLOB`.

**Why it is real:** round 2's consequence paragraph stated "nothing in §2.2, §9 or §10 schedules it",
and the fold changed none of those three sections.

**Suggested resolution:** one clause in decision 8 — *"the prohibition is container-scoped: `me seal`
and `MNEMBLOB` keep EPD§2.2 item 1 unchanged (decision 1)"* — and an F-125 in §10 owned by the Rust
phase: amend `passphrase.rs`'s module doc and EPD§2.2/§8 to say so, before `--passphrase-ask` ships.

---

### [NIT] 9. Two second sites round 2 named are still standing: the buffer blockquote's inequality, and the unrestricted citation of `ValidatePassphrase`

**Where:** §6.2.2 l.821 and §6.2.2's `character range` row (l.809);
`passphrase/passphrase.go:23–38`.

**Consequence:** (a) the fold replaced "≥ 215" in the table row and left `"(≥ 215 bytes)"` in the
normative blockquote three lines below. Harmless in substance — a buffer capacity ≥ the cap is the
correct requirement — but it is the second site of the finding whose thesis was "an inequality is not
a spec", left standing. (b) Row 1 still presents `passphrase.ValidatePassphrase` as "the device's
real constraint" and never says *do not call it*. Verified: that one function enforces `r < 0x20 ||
r > 0x7E` **and then** `n > MaxLen` (=100). Calling it is the natural transcription of row 1 and
silently rejects every passphrase of 12 words or more (107 bytes).

**Suggested resolution:** blockquote → *"allocated once at exactly 215 bytes and never regrows"*; row
1 → append *"— the systemwide path enforces the range only and MUST NOT call
`passphrase.ValidatePassphrase`, which also applies `MaxLen`."*

---

## VERDICT: 2 Critical, 2 Important, 4 Minor, 1 Nit

---

### What the fold got right, recorded so a round 4 does not re-derive it

- **R2-C1's fix is complete and clean.** Both sites were replaced, not annotated, and test 9 now
  carries its own satisfiability argument against test 15. Verified green.
- **The 215-byte cap is arithmetically correct and its basis is the authoritative wordlist.**
  Measured: 2048 words, max length 8, 88 words at 8, **zero longer**; file SHA-256 matches the
  official BIP-39 English list. 24 × 8 + 23 = 215; 12 × 8 + 11 = 107. `passphrase.MaxLen` is 100 with
  the quoted comment. The row's *values* need no further review — only its measurement basis
  (finding 5).
- **§2.2 item 8's code claims are exact.** `gui/unlock_kdf.go:168` and `:359` both hold `!m.Valid()`,
  and `bip39.Valid()` opens `if len(m)%3 != 0 { return false }`. The rejection claim is true as
  written; the finding against item 8 is about the function it *omits*, not the ones it cites.
- **§6.2.2a is the right answer to R2-I2's residue horn.** It refuses to claim a wipe it cannot
  deliver, grounds the acceptance in decision 2 and EPD§2.2 item 12 rather than in convenience, and
  scopes the no-regrow rule down to what it actually buys. It is consistent with both rulings and
  excuses nothing they did not cover — it is narrower than they are, which is the safe direction.
- **§8a's byte-identity claim still holds** (`gui/unlock_kdf_test.go:471`), and under the new cliff
  ruling its "no header field declares the type" argument is not merely sound but *load-bearing*:
  a word count computed from the passphrase string is exactly the discriminator that needs no field
  and gives an attacker nothing to flip. Finding 2's fix preserves this rather than trading it away.
- **The `compared` table's three rows are the right shape.** Both findings against it are about sites
  the fold did not carry the rule to (finding 1) and about the section it defers to for the word
  "cliff" (finding 2) — neither is an argument against either operator ruling.
- **The new cliff ruling genuinely dissolves attack #1**, and it is the better construction: it
  removes an unrecordable predicate from a consumption gate and replaces it with one both sides
  compute locally. Finding 2 is the fold surface it lands on, not a case against it.
- **R2-M1, R2-M2 are clean folds**, and **decision 8's overrule paragraph is a good one** — it quotes
  the prohibition it overrules, gives the reason for the prohibition rather than paraphrasing it, and
  points at the pricing that keeps the mode honest.
- **`scripts/spec-check.py` is a net win** despite finding 7: it caught the mode-count mutant, it
  resolved 23 citations and the test numbering, and its blind-spot paragraph is the right practice.
  Findings 7's items are repairs, not a case against the tool.
