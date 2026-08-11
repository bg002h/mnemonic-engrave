# R0 round 1 — re-review of the fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `6c2d0c7` (the fold).
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round0.md` at `6d89299`,
  and the isolated fold diff `6d89299..6c2d0c7` (237 insertions, 62 deletions, one file).
- **Questions answered:** (1) did the fold fix each of the 14 round-0 findings; (2) did the fold
  introduce a new defect. **This is not a fresh audit** — no section the fold did not touch was
  re-opened except where the fold made it wrong.
- **Not re-derived** (stated as already machine-verified in the brief): the 23 internal `§`
  cross-references, the digest-label consistency, the 7-sites-across-4-programs count, the two verify
  flows' first statements, EPD§6.6's `pub_len == 0` rule, `unlockPassphraseFlow`'s signature, the
  zero-code-block build gate.
- **Operator rulings treated as decisions, not proposals:** C4 (user-supplied dropped, generated-only,
  any N in 2..24 on the word keyboard) and C3 (verification never forced). They are not argued
  against below. They *are* flagged where they contradict the spec's other text or the code — which
  the brief explicitly asks for, and which both of them do.
- **Method:** read the fold diff, then the whole post-fold spec, then read the fold's new code claims
  against the fork: `gui/unlock_kdf.go`, `gui/gui.go` (`inputWordsFlow`, `updateValidCandidateKeys`,
  `completeCandidateWord`), `bip39/bip39.go` (`LastWordCandidates`, `Valid`, `splitMnemonic`),
  `seal/record.go` (`AdmitSection`, `permitted`, `groupRecords`, `cardKey`, `labelCards`),
  `gui/derive_xpub.go`, and `crates/me-cli/src/seal/passphrase.rs`. Every count and probability below
  was computed from the code, not from a doc comment.

---

## Part 1 — did the fold fix each round-0 finding?

| finding | status | reason |
| --- | --- | --- |
| **C1** — single seam routes the verify re-entries | **FIXED** | Two entry points (`seedEntryFlow` / `seedEntryFlowTypedOnly`), verify sites take the second, and test 16 asserts it structurally rather than in prose; traced — the only two verify re-entries in `gui/` are `singlesig_verify.go:50`… `multisig_verify.go:50` and `singlesig_verify.go:67`, and `newInputFlow`/`xpubVerifyFlow` re-enter no seed, so the seam is sufficient. |
| **C2** — `pub_len == 0` has no digest | **PARTIAL** | The §5.4 table and §5.4.1's identity are corrected, but the *blocking* horn survives: §5.4's own prose (l.443) still says "both container variants show the digest", test 9 (l.837) still says so and still fails any program consuming without a compared digest, and §5.4.1 (l.518) still admits records only when `compared` is true — so a secrets-only sealed payload is unusable and tests 9 and 15 cannot both pass. See new **[CRITICAL] #4**. |
| **C3** — §7.1 gates on a fact the device cannot observe | **FIXED** | The entry condition is gone; §7.1 offers the menu unconditionally and §7.1.1 gives outcomes a provenance the device can honestly record. (The ruling created a separate gap in §7.2's menu — new **[IMPORTANT] #5** — but the round-0 finding itself is closed.) |
| **C4** — passphrase modes have no device-side entry path | **PARTIAL** | User-supplied is dropped in decision 8 and §6.2.1, but not in §6's opening ("Three modes in `me`", l.603) or in the NORMATIVE `me` surface (`--passphrase-ask`, l.572/581); and the surviving generated-N mode is **not enterable on the existing keyboard** for any N divisible by 3, including the default 12. See **[CRITICAL] #1** and **[CRITICAL] #2**. |
| **I1** — `md1`-shaped smuggling defeats the §5.3 flag | **FIXED** | §5.3.2 carries EPD§6.3's decode requirement over normatively, names the threat, and adds test 14 asserted against the real `md.Reassemble`. The pass-3 statement is the right normative sentence; its *implementation* coupling is a new finding — **[IMPORTANT] #6**. |
| **I2** — AEAD tag coverage / AAD unspecified | **FIXED** | The table row is corrected, "neither is redundant" is deleted, the sealed digest is re-stated as downgrade detection, the AAD is written out as `header ‖ public section`, bytes `[0, HeaderLen + pub_len)`, and test 18 exists. |
| **I3** — "once per payload" needs an identity the spec never defines | **PARTIAL** | §5.4.1 now defines a content-derived identity (and the construction is sound — see the note after Part 2), but §3.2.1's NORMATIVE store still defines the same field as "the full EPD§6.6 digest" (l.158), i.e. the value §5.4.1 was revised to reject. See **[CRITICAL] #3**. |
| **I4** — "one seam, not eight" is off by a factor of four | **FIXED** | Replaced by the measured 7-sites/4-programs table, the four unshared entry points named, and the slogan explicitly withdrawn rather than quietly dropped. |
| **I5** — overwrite artefact described as a container | **FIXED** | It is now a raw region image, exactly `RegionLen` bytes, no magic/header/records, and both tests 11 and 12 become satisfiable. (O6 was left open despite §5.5 and §5.6 both deciding `random` — **[NIT] #11**.) |
| **I6** — admission tuple has no value for an NFC record | **FIXED** | §3.3.2a makes source a flag input rather than an admission axis, and F4 gives the NFC path a screen. (One residual ambiguity against §5.4.1 — **[MINOR] #8**.) |
| **M1** — `MNEMBLOB/pub/v1` reused | **FIXED** | `MNEMSYSW/pub/v1`, with the reasoning stated; consistent across all sites per the controller's machine check. |
| **M2** — test 8 cannot fail | **FIXED** | Test 8 now names `gui` as the assertion target and says why a `seal`-scoped test would be a false pass. |
| **M3** — §5.4's forward reference to §11 unfulfilled | **FIXED** | §11 gains the bullet: NFC-delivered secrets carry no integrity check, nothing to compare, F4 says so on screen. |
| **N1** — lost pre-KDF typo screen | **FIXED** | §6.3 now states the loss (~31 s per typo, indistinguishable from tamper) and specifies a per-word wordlist check at the keystroke, and is honest that a valid word in the wrong place is unrecoverable. `bip39.ClosestWord` exists (`bip39/bip39.go:95`) — the citation is real. *Not equivalent to what was lost, and the spec says so; but its description of the keyboard is incomplete in a way that matters — see* **[CRITICAL] #2**. |

**Score: 10 FIXED, 4 PARTIAL, 0 NOT FIXED.**

---

## Part 2 — new defects

### [CRITICAL] The NORMATIVE `me` surface still builds the user-supplied passphrase mode that C4 dropped, and §6's opening still says it exists

**Where:** §5.6 (l.572 synopsis, l.581 flag table), §6 opening (l.603), §6.2.1 lead-in (l.641–646);
against §1 decision 8 (l.44–53) and §6.2.1's table (l.652–655).

**Consequence:** an implementer transcribing §5.6 — the section headed "The `me` command surface —
NORMATIVE" — builds `--passphrase-ask`, "prompt for a user-supplied passphrase". The operator uses
it, writes the payload to `0x10D00000`, and the machine cannot open it: `unlockPassphraseFlow`
(`gui/unlock_kdf.go:109`) returns a `bip39.Mnemonic` typed on the word keyboard, with no free-text
path. On a *backup* device that is the permanent-data-loss class — the exact consequence C4 was
raised on, reachable from the spec's own normative CLI table.

**Why it is real:** the fold changed decision 8 and §6.2.1's strength table, and changed nothing else.
Three sites still describe the dropped mode as live, two of them normatively:

- l.603, §6's first line: *"Three modes in `me`: **none**, **user-supplied**, **generated N words**"*.
- l.581: *"`--passphrase-ask` | prompt for a user-supplied passphrase; **never** taken from argv…"* —
  written as an available flag, with operational advice attached.
- l.641–646, §6.2.1's own lead-in, in the present tense: *"meaningless for a user-supplied one"* and
  *"units that only apply to one of the three modes"* — immediately above the paragraph that says the
  mode was dropped.

So the majority of the passphrase text says the mode exists, and the CLI section that an implementer
actually transcribes is one of them. This is not a stale narrative aside; §5.6 is normative and is
the only place the flag surface is specified.

**Suggested resolution:** delete `--passphrase-ask` from the §5.6 synopsis and flag table, change
§6's opening to two modes, and rewrite §6.2.1's lead-in in the past tense so no present-tense sentence
in the document describes user-supplied as available.

---

### [CRITICAL] A generated N-word passphrase is not enterable on the existing word keyboard whenever `N % 3 == 0` — including the DEFAULT N = 12, where 15 draws in 16 are permanently unopenable

**Where:** §1 decision 8 (l.51–53), §6.3's replacement paragraph (l.690–695), §5.6 (`--passphrase-words N`,
default 12); `gui/gui.go:740`, `:758–768`, `:1204`, `:1224`; `bip39/bip39.go:117`, `:165`;
`gui/unlock_kdf.go:156`, `:168`; `crates/me-cli/src/seal/passphrase.rs:19`.

**Consequence:** `me sysw pack` with the default `--passphrase-words 12` emits twelve words drawn
uniformly from the wordlist (§6.3 mandates exactly this: *"drawing words directly from the wordlist
rather than via `Mnemonic::from_entropy_in`"*). At the machine, `inputWordsFlow` masks the keyboard
on the **last** word to the checksum-valid candidate set. A uniformly drawn last word is in that set
with probability **1/16**. The other **93.75%** of the time the operator physically cannot press the
keys for the twelfth word — and if they could, `unlockPassphraseFlow:168`'s `if !m.Valid()` rejects
the entry and re-prompts forever. The payload is permanently unopenable, on a backup device, using
the spec's default setting. This is C4's data-loss class, reinstated by the fold's own ruling.

**Why it is real:** decision 8's rationale makes a claim about the code that the code contradicts:

> *"Arbitrary N stays: the word keyboard can enter any number of words — it is the mnemonic
> *checksum* parse that forces 12/15/18/21/24, not the keyboard."*

The keyboard is checksum-gated, measured:

- `gui/gui.go:758–768` — `refreshCands()` sets `cands = bip39.LastWordCandidates(mnemonic)` when the
  operator reaches the final slot.
- `gui/gui.go:740` — `onLastWord()` is `selected == len(mnemonic)-1 && cands != nil`.
- `gui/gui.go:745` / `:1224` — on the last word the key mask is `updateValidCandidateKeys(cands, …)`,
  which enables only letters that extend the fragment toward a **candidate**.
- `gui/gui.go:1204` — `completeCandidateWord` returns `false` for any word not in `cands`, so the OK
  button does nothing.
- `bip39/bip39.go:165` — `LastWordCandidates` returns `nil` when `len(prefix)%3 != 0` (so N not
  divisible by 3 *is* unrestricted — the defect is confined to, and certain for, N ∈ {3,6,9,12,15,18,21,24}),
  and otherwise keeps every word for which `m.Valid()` holds. With `checkBits = N/3`
  (`bip39/bip39.go:217`), that is `2048 / 2^(N/3)` candidates:

  | N | 3 | 6 | 9 | **12** | 15 | 18 | 21 | 24 |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | candidates | 1024 | 512 | 256 | **128** | 64 | 32 | 16 | 8 |
  | P(uniform draw is enterable) | 1/2 | 1/4 | 1/8 | **1/16** | 1/32 | 1/64 | 1/128 | 1/256 |

  The 128 figure is the same one the spec already quotes from the other side — §6.3: *"a random
  12-word draw passes about one time in sixteen"*. The spec computed the number and then reasoned as
  if only the host were subject to it.

Today's shipped system is consistent precisely because the host does **not** draw uniformly:
`passphrase.rs:19` uses `Mnemonic::from_entropy_in`, so `me` emits a checksum-valid twelve, and the
device's last-word mask accepts it. §6.3 removes that agreement on the host side and leaves the
device's enforcement in place. EPD§8.1's "byte-identical KDF input" is not the binding constraint
here — the device cannot produce the string at all.

Test 6 still cannot catch this, for the reason round 0 gave: *"Host and device produce byte-identical
KDF input for an arbitrary-N passphrase"* is a pure-function test over a supplied string.

**Suggested resolution:** the ruling stands; what has to change is the device side, and §2.2's
"genuinely new" list has to say so. Specify that the systemwide unlock screen (a) takes an N picker
or length from the header, and (b) **does not apply `LastWordCandidates`** — the passphrase is not a
mnemonic and has no checksum, so the last slot must be the full 2048-word keyboard — and drop
`unlockPassphraseFlow`'s `m.Valid()` gate for this path. Then restate §6.3's "replacement" honestly:
the per-word check is `updateValidBIP39Keys`, on every slot including the last. Add a test that
drives the UI and enters a uniformly drawn N-word passphrase for N ∈ {2, 12, 24}.

---

### [CRITICAL] §3.2.1's NORMATIVE store still defines `identity` as the EPD§6.6 digest — the exact value §5.4.1 was revised to reject

**Where:** §3.2.1 l.158, against §5.4.1 l.497–506.

**Consequence:** §3.2.1 is the block an implementer transcribes for the session store, and it reads:

```
identity   [32]byte      the full EPD§6.6 digest (§5.4.1)
```

Built from that line, a secrets-only sealed payload's identity is the digest of an empty record set —
a **constant**, identical for every such payload — so a swapped payload inherits the previous one's
`compared` flag and is consumed without a prompt. That is the silent authentication bypass C2 was
raised on, verbatim, still reachable from a NORMATIVE block. The cross-reference `(§5.4.1)` makes it
worse: it reads as corroborated rather than stale.

**Why it is real:** the fold rewrote §5.4.1 to *"`SHA-256("MNEMSYSW/id/v1" ‖ 0x00 ‖ the region bytes
as read, bounded by the header's declared total)`"* and explains at length why the digest cannot be
the identity. §3.2.1 was not touched by the fold (confirmed against the diff) and still names the
rejected construction. §5.4.1's closing paragraph compounds it — *"Re-reading the region recomputes
**the digest**; if it differs, the entry is a different payload"* (l.520) — using "digest" where the
new text means "identity".

**Suggested resolution:** change §3.2.1's field to `the §5.4.1 payload identity` and reword l.520 to
"recomputes the identity". Two lines, and the only two places the old construction still appears.

---

### [CRITICAL] A secrets-only sealed payload shows no digest yet may not be consumed without one, so the sealed variant's main case is unusable — and tests 9 and 15 cannot both pass

**Where:** §5.4 l.443–444 and the table at l.448–452, §5.4.1 l.518–519, §8.3 tests 9 and 15.

**Consequence:** the fold fixed the display rule and left the consumption rule and the test that pins
it. As written:

- §5.4 table: `sealed, secrets only (pub_len == 0)` → digest shown? **NO**.
- §5.4.1 l.518: *"A record is admitted for consumption only when `compared` is true for the identity
  it came from."*
- Test 9: *"A program that consumes payload-sourced input without a compared digest **fails**."*

There is no digest to compare, therefore `compared` can never become true, therefore **no program may
ever consume a record from a secrets-only sealed payload** — which is the ordinary reason to seal one
(a seed, nothing public). This is round 0's first horn, unaddressed; round 0's suggested resolution
("state what stands in the digest's place when there is no public section — for a sealed payload that
is the AEAD tag, and a successful unlock is the evidence") was not folded.

Separately, the two named tests are mutually unsatisfiable: test 9 requires the digest displayed for
**both** container variants; test 15 requires a `pub_len == 0` sealed payload to display **NO**
digest. No implementation passes both, so the suite is red by construction.

**Why it is real:** §5.4's pre-fold prose survives four lines above the table that corrects it —
l.443: *"both container variants show the digest, and so does every program that consumes from the
region"* — with the "CORRECTED after R0-C2" block immediately below saying that sentence was wrong.
The fold added test 15 rather than amending test 9, so the spec now asserts the corrected rule and
the superseded rule side by side, in the same section and in the same test list.

**Suggested resolution:** delete l.443–444's "both container variants" clause; amend test 9 to *"a
payload **with a public section** (`pub_len > 0`)"*; and state in §5.4.1 what satisfies the
consumption precondition when there is no digest — a successful AEAD open is the machine-checkable
evidence, and it is strictly stronger than an operator's button press.

---

### [IMPORTANT] §7.1 makes bypass "a menu option, not a hidden escape", but §7.2's normative menu has no bypass row

**Where:** §7.1 l.723, §7.1.1 l.738, §7.2 l.750–756.

**Consequence:** §7.2 is the menu an implementer builds, and its five rows are `every word`,
`even words / odd words`, `6 words`, `3 words`, `read only`. There is no "skip" / "don't verify" row.
So bypass becomes the Back button — precisely the "hidden escape" §7.1 forbids — and §7.1.1's fourth
provenance, `not verified`, has no menu action that produces it, so it is never recorded. Test 17
requires the four provenances to be distinguishable "in whatever the flow records and displays"; one
of the four is unreachable.

**Why it is real:** the fold rewrote §7.1 and added §7.1.1, and did not touch §7.2's table. Decision 9
(l.54–55, new) says *"Verification is never forced. The operator may bypass it"*, and §7.1 says
bypass is a menu option — three new normative statements about an option the one normative menu does
not contain.

**Suggested resolution:** add a `bypass — do not verify` row to §7.2's table, mapped to §7.1.1's
`not verified`, and say whether it is the default cursor position (it should not be).

---

### [IMPORTANT] "Pass 3 runs over the `ClassMDMK` subset only" is buildable, but `groupRecords`/`labelCards` are index-coupled to the FULL record list, so a literal transcription backfills plate identity onto the wrong records

**Where:** §5.3.2 l.415–421; `seal/record.go:295–327`, `:339` (`labelCards`), `:409` (`groupRecords`),
`:444` (`cardKey`).

**Consequence:** pass 3 does three things, not one: `groupRecords(strs)` → `decodePublicSet(g)` →
`labelCards(out, g)`. The spec's sentence is right about the middle one and silent about the third.
An implementer who narrows `strs` to the `ClassMDMK` subset gets a `grouping` whose `perRecord` is
indexed **over the subset**, and `labelCards` then writes `out[i].HRP / CardIndex / CardTotal /
PlateIndex / PlateTotal` at *full-list* index `i` (`record.go:349–355`). With the §5.3 widening in
force, a public section of `[mnemonic, md1-a, md1-b]` labels the **mnemonic** with md1-a's card
identity and leaves md1-b unlabelled. Plate identity is what the operator reads to tell "card 1 of 2,
plate 2 of 3" apart on steel; getting it wrong is a restore hazard on the artefact the machine exists
to produce.

**Why it is real:** the coupling is explicit in the code, and it is not incidental —
`cardKey(record.go:454, :464)` builds the non-chunked group key as `groupKey{hrp:'d', uniq: i + 1}`,
i.e. it **bakes the record index into the key**. Re-indexing the input therefore changes group
identity as well as label placement. Today this is invisible because `permitted(record.go:233)`
admits only `ClassMDMK` into the public section, so subset == full list; the widening is exactly what
makes them diverge. §3.3 promises *"an implementer transcribes it; nothing here is left to be
derived"*, and this is left to be derived.

**Suggested resolution:** one more sentence in §5.3.2: pass 3 groups and decodes the `ClassMDMK`
subset, but the subset **carries its original full-section indices**, so `cardKey`'s uniqueness
discriminator and `labelCards`' backfill remain aligned with `out`. Add a test with a mixed public
section (one secret + two md1 cards) asserting the labels land on the cards.

---

### [IMPORTANT] Arbitrary N up to 24 overruns `passphraseBytes`' fixed 128-byte capacity, and the regrow leaves an unwipeable copy of the passphrase

**Where:** §1 decision 8 (N max 24), §5.6 (`--passphrase-words N`, `2 ≤ N ≤ 24`), §5.1 ("as
`MNEMBLOB`"); `gui/unlock_kdf.go:194–195`.

**Consequence:** `passphraseBytes` builds EPD§8.1's normalised KDF input into
`make([]byte, 0, 128)`. Twenty-four words average ~145 bytes and reach 215 at the longest words, so
`append` regrows — and the function's own comment says what that costs: *"a regrow would leave a
stale copy of the first half of the passphrase in an orphaned array that nothing can reach to wipe."*
The passphrase is seed-equivalent by this project's own ruling (`unlock_kdf.go`'s §10.2.4 note: *"an
in-flight passphrase is seed-equivalent… it derives the key that opens everything"*), so the fold
silently creates an unwipeable secret residue on the device for large N.

**Why it is real:** the capacity is not a round number, it is a computed one — the comment reads
*"Twelve words of at most eight letters plus eleven separators is 107 bytes."* It was sized for the
mode decision 8 has just widened, and nothing in §2.2, §5.1 or §6 mentions the reuse or its bound.
This is the class the memory note *"comments outlive their conditions"* was filed against: a
correctness argument that was true only under a constraint the spec has now removed.

**Suggested resolution:** state in §5.1 or §2.2 that the systemwide unlock's KDF-input buffer is
sized for `MaxPassphraseWords` (24 × 8 + 23 = 215 → 256), and add it to the "genuinely new"
device-side work alongside the entry-surface item from **[CRITICAL] #2**.

---

### [MINOR] §3.3.2a's "source is not an admission input" contradicts §5.4.1's `compared` precondition

**Where:** §3.3.2a l.259–263, against §5.4.1 l.518–519 and test 9.

**Consequence:** §3.3.2a is normative and flat — *"Source is not an admission input; it is a **flag**
input… One function, every path, no exceptions."* §5.4.1 is equally normative and says a record *"is
**admitted** for consumption only when `compared` is true for the identity it came from"* — a
precondition that exists only for flash-sourced records and can never hold for a typed or NFC one.
An implementer building the single admission function has nowhere to put the compared-gate without
breaking §3.3.2a's "no exceptions", or must read §5.4's "'Flash' scopes this to payloads read from
the region" as an unstated exception to it.

**Why it is real:** both sentences are new-or-load-bearing fold text (§3.3.2a is entirely new), and
they use the same word — "admitted" — for two different gates. §5.4's scoping sentence rescues a
careful reader, which is why this is Minor rather than Important.

**Suggested resolution:** one clause in §3.3.2a: admission by class is source-blind; the §5.4.1
compared-gate is a **separate, flash-only** precondition evaluated after it, alongside the flags.

---

### [MINOR] Test 18 can pass for the wrong reason — an "altered public section" is refused before the AEAD open

**Where:** §8.3 test 18, against §5.3.2 and `seal/record.go:254` (pass 1) / `:322` (`decodePublicSet`).

**Consequence:** *"a payload whose public section is altered after sealing fails to open"* is
satisfied by altering an `mk1` into garbage — which is refused by pass 1 (uppercase), pass 2
(`ErrRecordNotPermitted`) or pass 3 (`ErrUndecodableCardSet`) *before* any AEAD open happens. The
test then passes green with `AAD = header` only, which is the exact binding the test exists to pin
and the funds-loss path EPD§6.1a closes.

**Why it is real:** the threat model round 0 stated is a **valid substitution** — *"an attacker swaps
an `mk1` for one encoding their xpub"* — not corruption. Only a substitution that survives admission
distinguishes `AAD = header` from `AAD = header ‖ public section`.

**Suggested resolution:** restate test 18 as: replace one `mk1` with a **different, well-formed,
admissible** `mk1` of the same length, leaving `pub_len` unchanged; the open must fail on the tag.

---

### [NIT] Test 16's structural assertion matches `seedEntryFlowTypedOnly` as a substring, so it fails on a correct implementation

**Where:** §8.3 test 16, against §3.1 l.117–119.

**Consequence:** *"asserted structurally, by no verify flow naming `seedEntryFlow`"* — but the
function the verify flows are *required* to call is `seedEntryFlowTypedOnly`, which contains
`seedEntryFlow` as a prefix. A grep-based test fails on the correct implementation, and the natural
fix (loosen the pattern) is the one that reintroduces the false pass.

**Suggested resolution:** either name the assertion precisely (`seedEntryFlow(` as a call token, or
an AST/callgraph check), or rename the typed-only entry point to something that is not a superstring
— e.g. `typedSeedEntryFlow`.

---

### [NIT] O6 is still listed as an open item that "R0 should challenge", though §5.5 and §5.6 both decide it and round 0 endorsed the decision

**Where:** §9 row O6, against §5.5 ("Random is the DEFAULT — decided here rather than left open") and
§5.6 (`--fill` … **default `random`**).

**Consequence:** the open-items table is the reconciliation surface; leaving a decided item open
costs a later reader a round-trip. O1, O2 and O5 were struck by this fold; O6 was not, and round 0's
I5 finding explicitly closed it (*"random is the right default under a raw image"*).

**Suggested resolution:** strike O6 the way O5 was struck, citing §5.5.

---

## VERDICT: 4 Critical, 3 Important, 2 Minor, 2 Nit

---

### What the fold got right, recorded so a later round does not re-derive it

- **C1's structural fix is sufficient.** Two entry points beat a boolean, and the reasoning given
  (a wrong boolean still compiles) is correct. Traced: the only seed re-entries in a verify path are
  the two `seedEntryFlow` sites the spec names; `newInputFlow` has one caller (`gui.go:1704`, the
  `backupWallet` arm) and no verify re-entry, and `xpubVerifyFlow` takes an `mk.Card`, not a seed.
  The four unshared wirings do not reopen C1.
- **C2's identity construction is sound on the attack it was asked about.** The header-declared bound
  is attacker-controlled but *committed to inside the hashed prefix*: the header sits at offset 0 and
  is therefore inside `bytes[0, T)`, so two regions agreeing on `bytes[0, T)` agree on `T` and on
  every byte any parser consumes (`HeaderLen + pub_len + ct_len + tag`), and two different declared
  totals give two different hashed prefixes. **No two distinct payloads share an identity, and no one
  payload yields two.** The label is fixed-width, `0x00`-separated, and is not a prefix of
  `MNEMSYSW/pub/v1`, so the two hash inputs cannot alias. The defect is in where the identity is
  *transcribed* (§3.2.1), not in what it is.
- **I1's pass-3 claim is buildable** — `AdmitSection`'s pass 3 is exactly the three steps the spec
  describes, `permitted` is exactly the widening point, and `cardKey`'s `default` branch is exactly
  the fail-closed the spec cites. Only the label re-alignment is missing.
- **Tests 14–18 can each fail** as written, which was the assigned question. 14, 15, 16 and 17 fail on
  a real defect; 18 fails on a real defect but can also pass on a wrong one (Minor above); 16 also
  fails on a *correct* implementation for a naming reason (Nit above); 15 contradicts test 9
  (Critical #4).
- **I4, I5, M1, M2 and M3 are clean folds** — each replaced the wrong claim rather than annotating it,
  and I4 in particular withdrew its own slogan instead of defending it.
