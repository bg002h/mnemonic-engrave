# R0 round 0 — adversarial architect review of `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `d107c5c` (dispatched against `18ec759`; the
  §5.4 / §8.3-9,10 / O5 revision landed mid-review and this report is written against the **new**
  text — the NFC-digest findings from the old §5.4 are not carried).
- **Reviewer:** independent architect context, opus tier.
- **Question answered:** does this specification contain a defect that would produce a wrong, unsafe,
  or unbuildable result if implemented as written?
- **Not re-derived** (machine-verified before dispatch): the 11 `file:line` citations, the 14
  internal and 15 `EPD§` cross-references, the flash constants, the §6.1 and §7.3 tables, F-123/F-124
  numbering, the zero-code-block build gate.
- **Method:** read the spec whole, then read against the real sources — `SPEC_encrypted_payload_delivery.md`
  §§2.2, 5, 6, 6.1a, 6.2, 6.3, 6.6, 8, 8.1, 10.2.1, 10.2.1a, 10.2.3; and the fork's
  `seal/wire.go`, `seal/record.go`, `seal/session.go`, `seal/pubhash.go`, `seal/open.go`,
  `seal/read_tinygo.go`, `gui/scan.go`, `gui/derive_xpub.go`, `gui/singlesig_verify.go`,
  `gui/multisig_verify.go`, `gui/unlock_flow.go`, `gui/unlock_kdf.go`, `gui/gui.go`. Every count in
  this report was produced by `grep`, not by hand.

---

### [CRITICAL] §3.1's single seam routes the two verify re-entries, so a "from payload" source at the verify prompt lets the session cache answer the verification §7.4 says it must never answer

**Where:** §3.1 vs §7.4; `gui/singlesig_verify.go:67`, `gui/multisig_verify.go:50`,
`gui/derive_xpub.go:82`.

**Consequence:** the operator engraves a seed sourced from the payload, chooses Verify, is offered
the same three sources at the re-entry prompt, and selects **Payload**. The flow receives the
identical cached mnemonic it engraved from, re-derives the comparator baseline from it, and compares
it against itself. A mis-cut plate — the exact failure §7 exists for, on a machine whose output end
demonstrably fails — is certified PASS, silently, with no operator input at all. This is
reachable by pressing the obvious button on the screen §3.1 mandates.

**Why it is real:** §3.1 does not merely permit this, it *requires* it. It names the verify
re-entries in the list of call sites that get the feature: *"`seedEntryFlow` … is called from six
sites … and the two verify re-entries at `singlesig_verify.go:67` and `multisig_verify.go:50`.
Teaching that one function to offer three sources — Typed / Scanned / Payload — gives five programs
the feature at one stroke."* Verified in the code: both verify flows call `seedEntryFlow(ctx, th)`
as their first statement. The existing code was written specifically to prevent this, and says so —
`singleSigVerifyFlow`'s doc comment: *"It re-types + re-derives the seed (the comparator baseline is
re-derived internally, **NOT passed in**)"*; `multisigVerifyFlow`'s: *"comparing the READ-BACK mk1
against the re-derived mk1 (**H1: never the re-derived value against itself**)"*. §7.4 then forbids
what §3.1 built: *"The session cache must never answer a verification prompt … Otherwise verify
compares the engrave source against itself and passes unconditionally."* Two normative sections, one
seam, opposite requirements.

Test 1 does not close this. *"A cached secret cannot reach a verify comparison"* is satisfiable by a
unit test on the session store (e.g. the store refuses a read tagged `verify`), which passes green
while `seedEntryFlow` still renders the Payload choice at `singlesig_verify.go:67` — the offer is in
the UI, the assertion is in the store. The test as named cannot distinguish the two.

**Suggested resolution:** make the source menu a *parameter* of `seedEntryFlow`, not a property of
it, and state normatively that the two verify re-entry sites pass "Typed only". Restate §3.1's count
as "five sites gain three sources; two verify sites keep one", and rewrite test 1 to drive the verify
flow through the UI and assert the Payload choice is absent from the rendered menu.

---

### [CRITICAL] A sealed container with `pub_len == 0` has no digest to show, so §5.4 and test 9 either block the sealed variant's main use or display a constant that every fully-encrypted payload shares

**Where:** §5.4 ("both container variants show the digest"), §8.3 test 9; EPD§6.6;
`seal/open.go:37`, `gui/unlock_flow.go`.

**Consequence:** a systemwide sealed payload carrying only secrets — one seed, nothing public, which
is the ordinary reason to seal — has an empty public section. §5.4 says both variants show the
digest and test 9 says *"A program that consumes payload-sourced input without a compared digest
fails."* Two outcomes, both defective:

- Show nothing → test 9 fails by construction and no program may consume a secrets-only sealed
  payload. The sealed variant's primary case is unusable.
- Show the digest anyway → it is `SHA-256("MNEMBLOB/pub/v1"‖0x00‖0x01‖0x00)[:16]`, a **fixed
  constant**, identical for every secrets-only sealed payload ever produced. The operator compares
  it, it matches, they tick "compared" and are reassured by a number that carries zero bits about
  their payload. It matches an attacker's substituted payload too.

The second outcome is worse under the revision, and is the answer to the controller's payload-identity
question. §5.4 says *"the session records that this payload's digest was compared"* and *"the flag is
on the payload"*. The only content-derived identity the spec makes available is the digest itself —
and for this class of payload the digest is a constant, so **every secrets-only sealed payload shares
one identity**. Any implementation that keys the compared-flag on the digest lets a swapped payload
inherit a previous payload's "already compared" state and never prompt.

**Why it is real:** EPD§6.6 is normative and explicit: *"**Displayed whenever `pub_len > 0`** … When
`pub_len == 0` **nothing is displayed**: the digest of an empty record set is a constant, and showing
the same number on every fully-encrypted payload would teach the operator it is furniture."* The
shipped code enforces it — `seal/open.go:37`: *"HasHash is false exactly when pub_len == 0"*, and
`gui/unlock_flow.go` guards the notice with `if p.HasHash`. EPD§2.2 item 10 states the same boundary
from the other side: *"§9 prints it for every payload **with a public section**."* §5.4's "both
variants show the digest" contradicts a NORMATIVE EPD rule, and test 9 pins the contradiction into
the test suite.

**Suggested resolution:** carry EPD§6.6's `pub_len > 0` condition into §5.4 verbatim, and state what
stands in the digest's place when there is no public section — for a sealed payload that is the AEAD
tag, and a successful unlock is the evidence. Amend test 9 to *"a payload with a public section"*,
and state normatively that the compared-flag's identity is **never** derived from the digest.

---

### [CRITICAL] §7.1 gates a verification *weakening* on a fact the device cannot observe, coupling the two defences §7 opens by declaring independent

**Where:** §7 opening paragraph vs §7.1; §5.2, §5.4; EPD§6.6, EPD§10.2.3.

**Consequence:** an attacker replaces the plaintext systemwide payload — which §5.2 concedes has *no
authentication whatsoever*. At load the device shows a digest the operator does not actually compare
and dismisses, exactly the habit §5.4 says a re-prompt would train. §7.1 now classifies the source as
*"independently verified"* and offers the reduced menu, including **read only**. The operator engraves
the attacker's seed, performs no readback, and the machine reports the plate verified. One dismissed
button defeats both controls at once, and the second control was *unlocked by* the failure of the
first.

**Why it is real:** §7 opens by asserting the two are independent — *"The hash and the plate verify
defend opposite ends of the pipeline … **No amount of input integrity says anything about the
second**"* — and §7.1 then makes the second conditional on the first: *"The menu appears when the
secret's source is independently verified: **a plaintext container whose hash the operator
compared**."* The device cannot establish that predicate. §5.2 states this in the same document:
*"**The device never detects a hash mismatch.** It has no idea what the operator wrote down."*
EPD§6.6 agrees: *"an out-of-band check that works **only if the operator actually compares it**."*
What the device observes is a button press — EPD§10.2.3's `[ Matches — continue ]`. §5.4 then
supplies the argument that this button *does* get dismissed reflexively, which is why it forbids
re-prompting. The spec therefore knows the gate is unreliable and uses it as a gate anyway.

Note this is not an argument against the operator's decision that verification depth is their choice
(§1 item 7) — it is an argument that §7.1's *entry condition* is not implementable as written.

**Suggested resolution:** decouple them. Offer the depth menu on a basis the device can actually
establish — a **sealed** container the operator unlocked (the AEAD tag is a machine-checkable fact) —
and keep full re-entry for the plaintext container, whose digest the device cannot confirm was
compared. Alternatively make the menu unconditional (§7.1 already concedes the read-back does the
same job either way), which removes the false coupling rather than mis-implementing it; that is an
operator ruling either way, but the current middle position is the one option that cannot be built
honestly.

---

### [CRITICAL] Decision 8's passphrase modes have no device-side entry path — a user-supplied or N≠12 passphrase produces a payload the machine cannot open

**Where:** §1 item 8, §6, §6.3, §2.2's "genuinely new" list, O2;
`gui/unlock_kdf.go:109` `unlockPassphraseFlow`.

**Consequence:** `me` seals a systemwide payload with a user-supplied passphrase, or with 7 generated
words. The operator writes it to `0x10D00000`, and — this being a *backup* device — may have no other
copy. At the machine the unlock screen can accept neither: it is a fixed 12-slot BIP-39 word entry.
The payload is permanently unopenable. That is the data-loss class, arrived at by using a mode the
spec mandates.

**Why it is real:** traced, and it closes O2 as a defect rather than an open question.
`unlockPassphraseFlow` (`gui/unlock_kdf.go:109`) does **not** use `PassphraseKeyboard`. Its own
comment: *"It does NOT reuse `seedEntryFlow`, which opens with a 12/24 word-count picker. §8 says the
passphrase is twelve words; there is no choice to offer. What it does reuse unmodified is
`inputWordsFlow`."* The body is `m := emptyBIP39Mnemonic(12)` then `inputWordsFlow(ctx, th, m, 0, "")`
— a hard-coded twelve, on the BIP-39 word keyboard gated by `updateValidBIP39Keys`. There is no free
text and no variable length. §5.1 says the sealed variant is *"Passphrase + PBKDF2-SHA256 +
AES-256-GCM, **as `MNEMBLOB`**"*, which points an implementer straight at this flow. §2.2's list of
what is genuinely new contains no device-side entry surface; item 6 is *"`me` passphrase modes"*,
host-side only. §6.3 says *"both sides move together"* but scopes that to `is_valid` and the KDF
input, not to how a human types the thing.

Test 6 cannot catch it. *"Host and device produce byte-identical KDF input for an arbitrary-N
passphrase"* is a pure function test over a supplied string; it passes green while the device UI is
incapable of producing that string.

**Suggested resolution:** add a device-side passphrase entry surface to §2.2's "genuinely new" list
and specify it: a word-count picker over 2–24 for generated mode, and `PassphraseKeyboard`
(free text) for user-supplied. State what replaces the BIP-39 checksum as the pre-KDF typo screen for
non-mnemonic passphrases — EPD§8 relies on that checksum to avoid burning ~31 s on a typo. Convert O2
from "not verified" to the traced fact above, and add a test that the device can *enter* every
passphrase `me` can *emit*.

---

### [IMPORTANT] §5.3's "flagged on screen" control is defeated by the smuggling EPD§6.3 exists to close, and test 4 passes anyway

**Where:** §5.3, §8.3 test 4; EPD§6.3, EPD§10.2.1; `seal/record.go` (`Classify`, `permitted`,
`AdmitSection` pass 3), `seal/session.go:17`.

**Consequence:** a defective or third-party sealer wraps 32 bytes of seed entropy in a BCH-valid
`md1`-shaped record and puts it in a plaintext systemwide container. `Classify` returns `ClassMDMK`;
`IsSecret` is false; **no flag is raised**; the operator is never offered the paired "erase this
region"; the secret sits in cleartext flash where `picotool save` reaches it with no passphrase, on a
device whose BOOTSEL is enabled by design. The one mitigation the spec pairs with decision 6 is
silent in exactly the case that matters.

**Why it is real:** the flag is defined over classification — §5.3: *"A plaintext container carrying a
**secret class** is flagged on screen at load"* — and §2.1 names the secrecy predicate as
`seal/session.go:17`, which is `ClassCodex32Secret || ClassMnemonic`. EPD§6.3 measured the bypass:
*"`ValidMD`/`ValidMK` … are **pure BCH verifiers** … So arbitrary bytes wrap into a record that
classifies as `mdmkText`,"* with the worked example
`32 bytes of entropy → md1qqqsyqcyq… ValidMD = true → scanner.Scan(...) = mdmkText`. EPD's answer is
the per-card-set **DECODE** requirement (`decodePublicSet`, `seal/record.go`), whose comment says
precisely why: *"without this a defective or third-party sealer can put seed entropy in the cleartext
section, where `picotool save` reaches them with no passphrase at all."* §5.3 says the systemwide
container has *"its own admission rules"* and never says whether the decode requirement survives.
§8.3 has no decode test.

Test 4 is a false-pass: *"A plaintext container carrying a secret class raises the flag"* is satisfied
by a real `ms1` or mnemonic, which classifies correctly. The defect lives entirely in the records that
*do not* classify as secret, which the test never constructs.

Second-order note, same root: §5.3 permits `ClassCodex32Secret` and `ClassMnemonic` in the public
section, but `AdmitSection`'s pass 3 sends every public record through `cardKey`, whose `default`
branch fails closed with *"record %d is not an md1 or mk1 card"*. Widening `permitted` without
restructuring pass 3 rejects every payload the widening was meant to allow — the two passes are
coupled and §5.3 changes only one.

**Suggested resolution:** state normatively whether EPD§6.3's card-set decode applies to the
systemwide container's `md1`/`mk1` records. It should — the widening is about admitting *declared*
secrets, not about admitting undeclared ones. Add test 4b: a BCH-valid `md1` carrying non-decodable
entropy must be refused (or flagged), and assert it against the real `md.Reassemble`. Say explicitly
that pass 3 runs only over the `ClassMDMK` subset of the public section.

---

### [IMPORTANT] §5.4's table says the AEAD tag covers "the ciphertext"; EPD§6.1a's AAD is the header **and** the public section, and the systemwide container's AAD is never specified

**Where:** §5.4 table and the paragraph below it; §5.1; EPD§6.1a; `seal/open.go:222`.

**Consequence:** an implementer building the new container's AEAD from §5.4's table sets AAD to the
ciphertext's own framing, or to the header alone. The public section then travels unbound. An attacker
with the brief physical access EPD§2.2 item 4 already concedes swaps one `mk1` for one encoding *their*
xpub; the tag still verifies; the operator engraves a steel backup of a wallet they do not control.
That is the funds-loss path EPD§6.1a was written to close, reopened by a table.

**Why it is real:** §5.4 asserts *"the AEAD tag covers the ciphertext"* and then *"the digest and the
tag cover different halves … **Neither is redundant**."* EPD§6.1a is normative and says the opposite:
*"**AAD = the header AND the public section**, i.e. bytes `[0, 52 + pub_len)` … Without the AAD
binding, an attacker with brief physical access could swap a `mk1` for one encoding *their* xpub, and
the operator would engrave a steel backup of a wallet they do not control."* The code agrees
(`seal/open.go:222`: *"open over AAD = header ‖ public section (§6.1a)"*). The public section is
therefore covered **twice**, deliberately — once cryptographically by the tag, once out-of-band by the
digest — and the redundancy is the design, not an oversight. §5.4's "neither is redundant" is the
premise for its coverage-is-complete claim, and it is false.

§5.1's *"as `MNEMBLOB`"* gives a careful implementer the right answer, which is why this is Important
rather than Critical — but §5.4's table is the only place the spec says what the tag covers, and it
says the wrong thing about a funds-path binding.

**Suggested resolution:** correct the table row to *"the AEAD tag covers the ciphertext **and binds
the header and public section as AAD**"*, delete "Neither is redundant", and re-state the sealed
digest's actual job in one line: it is not coverage the tag lacks, it is **downgrade** detection —
visible before any key exists, which is the one thing an AEAD structurally cannot do. State the
systemwide container's AAD explicitly in §5.1 rather than by reference.

---

### [IMPORTANT] "Once per payload" requires a payload identity the spec does not define, and §3.2 gives the plaintext variant no session presence at all

**Where:** §5.4 ("the flag is on the payload"), §3.2, §8.3 test 10.

**Consequence:** the two clauses give opposite answers to the same question and an implementer must
pick. §5.4: *"Re-reading the region produces a new payload identity and therefore a new comparison."*
Test 10: *"a second program consuming from the same loaded payload does NOT re-prompt."* For the
**plaintext** variant there is nothing to decrypt and therefore no reason to cache — the natural
implementation reads XIP per consumption, which is a re-read, which by §5.4 is a new identity, which
re-prompts, which fails test 10 *and* produces exactly the disarming repetition §5.4 was added to
prevent. Pick the other branch and identity becomes "a session slot", which is not derived from
content at all and cannot detect that the slot's backing bytes are not the ones that were compared.

**Why it is real:** §3.2 describes the session as *"Unlocking the sealed variant runs the KDF once;
**the decrypted records** live in a session store until power-off"* — it defines residency only for
the sealed variant's *plaintext output*. Nothing in §3.2 or §5.4 says a plaintext container is loaded
into a session at all, yet §5.4's flag is stateful and must live somewhere for both. And "identity"
is used four times without ever being defined by *what*: not by content, not by region bytes, not by
digest (see the Critical above, which forecloses digest-as-identity), not by a load counter.

**Suggested resolution:** define identity by construction: a payload is loaded into the session **once**
by copying the region's declared extent out of XIP (`seal/read_tinygo.go` already copies out for this
exact reason — *"a slice aliasing XIP would silently change underneath it"*), and identity is a
monotonic session-local load counter, incremented on every read of the region and never derived from
payload content. State that the plaintext variant occupies the session store too. Then test 10's two
halves are distinguishable by construction rather than by convention.

---

### [IMPORTANT] §3.1's "one seam, not eight" is contradicted by the code: 7 call sites across 4 programs, and the other four programs share no helper at all

**Where:** §3.1; measured across `gui/`.

**Consequence:** the architecture's central claim, and the sizing that follows from it, are wrong by a
factor of four on the second half. An implementation built to §3.1 delivers the payload/NFC sources to
Account Xpub, Engrave Single-Sig, Engrave Multisig and BIP-85 Child Seed — and leaves **Backup
Wallet, BIP-39 Password, Engrave Text and Engrave Bundle** without one, while §1 item 3 puts all eight
in scope and test 9 requires a compared digest for *every program that consumes from the region*. The
work either silently shrinks to half the scope or overruns, and §3.3's "admission is one function" has
no single seam to hang off.

**Why it is real:** measured by `grep`, not read.

- `seedEntryFlow` non-definition call sites: **7**, not "six" — `derive_xpub.go:107`,
  `multisig.go:91`, `singlesig.go:33`, `multisig_verify.go:50`, `bip85.go:271`,
  `singlesig_verify.go:67`, `multisig_build.go:58`.
- Distinct menu programs behind those 7 sites: **4**, not "five" — `engraveXpub`,
  `engraveSingleSig` (singlesig + its verify), `engraveMultisig` (multisig + build + verify),
  `bip85Derive`. Confirmed by tracing `singleSigVerifyFlow` ← `singlesig.go:120` and
  `multisigVerifyFlow` ← `multisig.go:153`, `multisig_build.go:164`.
- §3.1 asserts *"`backupWallet`'s `newInputFlow`, plus BIP-39 Password, Engrave Text and Engrave
  Bundle, **call the same helper** so there is one admission path, not eight."* `newInputFlow` has
  exactly **one** non-test caller, `gui/gui.go:1704`, on the `backupWallet` arm.
  `engraveTextFlow` (`gui/freetext_flow.go:1470`), `engravePassphraseFlow`
  (`gui/passphrase_flow.go:601`) and `bundleFlow` are separate dispatch arms that call **neither**
  `newInputFlow` nor `seedEntryFlow`.

So the real shape is at least **four** seams — `seedEntryFlow`, `newInputFlow`, `engraveTextFlow`,
`engravePassphraseFlow`, plus `bundleFlow` — not one.

**Suggested resolution:** replace §3.1's paragraph with the measured table above and state the real
work: one shared *source-selection* helper, adopted at four or five entry points. The "one admission
path" goal (§3.3) survives and is still right; it is the "at one stroke" sizing that does not.

---

### [IMPORTANT] §5.5's overwrite artefact is described as a container, which makes test 12 unsatisfiable and "fills the region" false

**Where:** §5.5, §8.3 tests 11 and 12, O6; EPD§6.2.

**Consequence:** two of the thirteen named tests cannot both pass against any single implementation.

- Test 12 requires *"all-ones is byte-identical to an erased region"* — every byte `0xFF`. But §5.5
  frames the artefact as a **payload** and argues against the zero-length alternative on the grounds
  that it *"rewrites the header"*, which says the artefact has a header. A header cannot be all-`0xFF`:
  it carries the magic, `version = 0x01` and `reserved = 0x00`. Test 12 fails by construction.
- Test 11 requires *"after writing it, no byte of the previous payload remains"* and §5.5 says it
  *"fills the region"*. If the systemwide container inherits EPD§6.2's bounds — and §5 says it reuses
  EPD's construction — the largest legal blob is `52 + 8191 + 8191 + 16 = 16450` bytes against a
  65,536-byte region. It fills **25%**. If it does *not* inherit them, the spec states no length bound
  at all for a container parsed on a device EPD§6.2 establishes has **no active watchdog**, where an
  unbounded declared length is a hang rather than an error.

**Why it is real:** the artefact is doing two incompatible jobs. As a *region image* — no magic, no
header, 65,536 bytes of fill — every one of §5.5's claims and both tests are satisfiable, and each
fill lands where §5.5 says it does (all-`0xFF` is exactly the erased state, so test 12 becomes
trivially true). As a *payload* neither is. §5.5 never chooses, and the sentence that argues against
zero-length implies the wrong one.

**Suggested resolution:** state that the overwrite artefact is a **raw region image, not a container**
— no magic, no header, exactly `RegionLen` bytes — so the device's next probe reads "no payload"
(EPD§6.1's rule) rather than a structural failure it must render forever. Then §5.5's whole argument
holds unchanged and both tests become satisfiable. On O6: **random** is the right default under a raw
image; all-`0xFF` is genuinely indistinguishable from erased, and all-zeros is the only fill that
proves deliberate action, so the doc line §5.5 already asks for is the correct control.

---

### [IMPORTANT] §3.3's admission tuple has no value for an NFC record, so the one path §5.4 just removed all integrity checking from escapes the one admission function

**Where:** §3.3, §3.1, §5.4 ("NFC is NOT covered"), §5.3 item 1; EPD§10.2.1.

**Consequence:** §3.3 defines admission over `(record class, container variant, requesting program)`.
A scanned record has no container and therefore no variant — there is no value to pass. An
implementer either invents one or, far more likely, bypasses the function for the NFC path and
type-switches locally, which is precisely the *"spread across `engraveObjectFlow`'s type switch and
each flow's private assumptions"* state §3.3 says it exists to end. Eight programs then each decide
independently what a tag may deliver, on the one delivery path that — after the revision — carries no
digest, no header and no authentication of any kind.

**Why it is real:** EPD§10.2.1 is titled *"The classifier allow-list — NORMATIVE, and load-bearing"*
and its argument is that `gui/scan.go`'s acceptance surface is wider than the format's: it returns
`debugCommand`, `addressText` and output descriptors as well. §2.1 of this spec correctly observes the
scanner has always accepted secrets and that *"the rule was enforced by which flows consume
`act.scan`"* — i.e. by the very per-flow assumptions §3.3 is removing. The spec adds a **new free-text
record class** (§5.3 item 1) and a new consumer for it, and §8.3 contains no allow-list test for the
NFC path at all. EPD's own rule is that this *"MUST be an allow-list, not a deny-list: a deny-list
silently admits whatever branch the classifier grows next."*

**Suggested resolution:** make the admission tuple `(record class, **source**, requesting program)`
where source ∈ {typed, scanned, plaintext container, sealed container} — this subsumes the container
variant and gives NFC a first-class value. Add a test: for each of the eight programs, every
`gui/scan.go` classification the program does not admit is refused with a named reason, including
`debugCommand`.

---

### [MINOR] The digest label `"MNEMBLOB/pub/v1"` is reused for a container §4 insists must be distinguishable

**Where:** §5 ("Reuses EPD§6.6's hash construction"), §4; EPD§6.6 point 3.

**Consequence:** a systemwide container and a `MNEMBLOB` container holding the same records with the
same sealed byte produce the **same** digest. An operator keeping one list of recorded values cannot
tell which region a value belongs to, and a match therefore certifies nothing about which security
regime — wiping or non-wiping, `0x10E00000` or `0x10D00000` — the payload will be handled under. No
wrong bits are engraved, which is why this is Minor.

**Why it is real:** §4 spends a paragraph justifying a distinct magic — *"a blob written to the wrong
address is **rejected** rather than half-understood"* — and §5 then reuses a label whose stated purpose
(EPD§6.6 point 3) is *"so this digest can never collide with any other SHA-256 use in the system."*
The revised §5.4 makes the same argument about NFC — *"`\"MNEMBLOB/pub/v1\"` cannot be reused over a
differently-shaped input, which is what that label exists to prevent"* — and does not apply it to the
container whose magic §4 just changed.

**Suggested resolution:** give the systemwide container its own label version alongside its own magic;
the change is one constant and one Rust vector, and it makes §4's and §5.4's reasoning consistent.

---

### [MINOR] Test 8 as named cannot fail — the phrase it forbids lives in `gui`, not in `seal`

**Where:** §8.3 test 8, §5.2; `gui/unlock_flow.go`, `seal/wire.go:66-79`.

**Consequence:** *"Structural failures never emit the words 'payload unreadable'"* passes vacuously if
written against the reader's error sentinels, which are `"seal: blob too short"`, `"seal: not a sealed
payload (bad magic)"` and so on — none contains the phrase. Meanwhile the screen can still say it: in
the existing flow every `Inspect` error but one renders as `showError(ctx, th, unlockTitle, "Payload
unreadable.")`. The requirement is satisfied in the layer that never violated it and unasserted in the
layer that does.

**Why it is real:** the phrase appears in `gui/unlock_flow.go` and in `gui/` tests, and nowhere in
`seal/`'s sentinels. The precedent for getting this right is already in the tree —
`TestUnlockNamesAnUnengraveableSecretInsteadOfCallingItUnreadable` (`gui/unlock_engraveable_test.go`)
drives the UI and greps the rendered frame.

**Suggested resolution:** specify that test 8 asserts on **rendered screen content**, and name the
existing UI-level test as the pattern.

---

### [MINOR] §5.4's forward reference to §11 is unfulfilled — §11 does not say NFC-delivered secrets are un-verified

**Where:** §5.4 ("See §11 for what that means the operator is *not* getting"), §11.

**Consequence:** the revision scopes digest verification to flash and points at §11 for the disclosure.
§11 was not updated: it discusses NFC only as *transient* versus flash's persistence, which is an
argument that NFC is *less* dangerous. Nothing in the document tells the operator that a secret
arriving over NFC — now a first-class path for eight programs — carries no integrity check at all and
that a substituted tag is indistinguishable from their own. The spec's §11 is explicitly the list of
things it does not claim, and this one is missing from it.

**Suggested resolution:** add a bullet to §11: NFC-delivered records are not digest-verified and not
authenticated; the operator's protection on that path is physical custody of the tag. One sentence,
and it makes the cross-reference true.

---

### [NIT] The `is_valid` change in §6.3 loses the pre-KDF typo screen and the spec does not say what replaces it

**Where:** §6.3; EPD§8.

**Consequence:** EPD§8 justifies the 12-word BIP-39 passphrase partly on the checksum: *"The BIP-39
checksum lets the device reject most typos in about a second, before committing to a ~30-second KDF.
Without it, a typo costs the full KDF and then an indistinguishable tag failure."* An arbitrary-N word
sequence has no checksum. §6.3 correctly identifies that `is_valid` must change and that both sides
move together, but does not mention that the operator now pays ~31 s per typo and receives an error
that cannot distinguish "you mistyped" from "this payload was tampered with".

**Suggested resolution:** one line in §6.3 acknowledging the cost, or a retype-to-confirm on the entry
screen. Related to the device-entry Critical above, and best fixed in the same place.

---

## VERDICT: 4 Critical, 6 Important, 3 Minor, 1 Nit

### Cross-cutting note on §8.3, since it was a named target

Of the thirteen named tests, **four can pass while the defect they name is present**, each for a
different reason, and each is called out in the finding it belongs to:

| test | why it can pass anyway |
| --- | --- |
| 1 — cached secret cannot reach verify | assertable on the store while the UI still offers Payload at the verify seam |
| 4 — plaintext + secret class raises the flag | passes on a real `ms1`; the smuggled `md1`-shaped secret never classifies as one |
| 6 — byte-identical KDF input for arbitrary N | a function test over a string the device UI cannot produce |
| 8 — never says "payload unreadable" | the phrase is not in the layer the natural test targets |

Tests 11 and 12 are the opposite problem: as written they cannot both pass at all (see the §5.5
finding). Tests 2, 3, 5, 7, 9, 10 and 13 can fail for the right reasons once the findings above are
folded — 9 and 10 only after §5.4's `pub_len == 0` case and the identity definition are settled.

### What is sound and should not be re-litigated

Stated so a later round does not spend budget re-deriving it: §4's address and clearance argument
holds against EPD§3/§5; §5.2's refusal to reuse "payload unreadable" for benign structural failures is
correct in intent and matches the reasoning already won for `ErrTooManyRecords` and
`ErrCodex32TooLong`; §7's opening claim that input integrity and plate verification are different
defences is right (§7.1 is what breaks it, not the premise); §7.3's honesty about "read only" having
no statable rate is the correct treatment; and §11's refusal to count the overwrite reminder as
protection is exactly the discipline F-123 was filed to enforce.
