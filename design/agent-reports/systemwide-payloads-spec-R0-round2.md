# R0 round 2 — re-review of the second fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `fc5da46` (the fold).
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round1.md` at `8ddbe2e`,
  and the isolated fold diff `8ddbe2e..fc5da46` (171 insertions, 30 deletions, one file, one commit).
- **Questions answered:** (1) did the fold fix each of round 1's 11 findings; (2) did the fold
  introduce a new defect. **This is not a fresh audit.** No section the fold did not touch was
  re-opened except where the fold made it wrong.
- **Not re-derived** (stated as machine-verified in the brief): internal `§` cross-reference
  resolution, the mode count across decision 8 / §6's opening / §5.6, the absence of fenced
  executable blocks. One brief-supplied fact did **not** hold and is filed below as **[MINOR] #8**:
  text from the reversed decision *does* survive, in §9's O2 row.
- **Operator rulings treated as decisions, not proposals:** §8 (user-supplied restored), §8a
  (keyboard choice at unlock), §8b (per-invocation checksum gate), §8c (`done` key + count
  confirmation). They are not argued against. They **are** flagged where they contradict the spec's
  other text, a sibling spec, or the code — which the brief explicitly asks for, and which three of
  the four do.
- **Method:** read the fold diff, then the whole post-fold spec, then traced every code and
  measurement claim the fold makes against the fork and the Rust crate:
  `gui/gui.go` (`inputWordsFlow`, `refreshCands`, `onLastWord`, `updateKeys`,
  `updateValidCandidateKeys`, `completeCandidateWord`, `isMnemonicComplete`),
  `bip39/bip39.go` (`Valid`, `LastWordCandidates`, `LabelFor`, `Mnemonic.String`, `splitMnemonic`),
  `bip39/wordlist.txt`, `gui/unlock_kdf.go` (`unlockPassphraseFlow`, `passphraseBytes`,
  `unlockAttemptOnce`), `gui/unlock_kdf_test.go`, `gui/passphrase_flow.go`,
  `gui/passphrase_keyboard.go`, `gui/plate_hook_test.go`, `passphrase/passphrase.go`,
  `seal/open.go`, `seal/record.go` (`cardKey`), `crates/me-cli/src/seal/passphrase.rs`,
  `crates/me-cli/src/main.rs`, and `SPEC_encrypted_payload_delivery.md` §2.2/§8.
  Every count below was measured, not read off a doc comment.

---

## Part 1 — did the fold fix each round-1 finding?

| round-1 finding | status | reason |
| --- | --- | --- |
| **R1-C1** — normative `me` surface still builds the dropped user-supplied mode | **FIXED** | Fixed by reversal, which is the direction the operator chose: decision 8, §6's opening (l.680) and §5.6's flag table (l.658) now all describe three modes, and §6.2.1's table gained the `user-supplied ASCII` row. The three sites agree. *(The restoration's own consequences are new findings **#3**, **#4** and **#5**, not a reopening of C1.)* |
| **R1-C2** — generated N-word passphrase not enterable when `N % 3 == 0` | **PARTIAL** | The root ruling landed (§8b drops the checksum gate for passphrase entry at every length; §6.2.2's `checksum` row and test 19 pin it), and the mask itself is correctly identified. But round 1's other two asks were not done: §2.2's "genuinely new" list still contains no device-side entry work, and the **two further enforcement points** — `gui/unlock_kdf.go:168` and `:359`, both `!m.Valid()` — are never named. See **[IMPORTANT] #2**. |
| **R1-C3** — §3.2.1's store still defined `identity` as the EPD§6.6 digest | **FIXED** | §3.2.1 l.199–201 now reads "the §5.4.1 identity digest. NOT the EPD§6.6 public-data digest, which does not exist when `pub_len == 0`", and §5.4.1 l.576 was changed from "recomputes the digest" to "recomputes the identity". Both sites round 1 named, both fixed. |
| **R1-C4** — secrets-only sealed payload unusable; tests 9 and 15 unsatisfiable | **PARTIAL** | The *consumption* horn is fixed and fixed well (§5.4.1's two-route `compared` table). The *display* horn is untouched: §5.4 l.499 still says "both container variants show the digest" four lines above the table that says NO, and test 9 still says the same, still contradicting test 15 and now also the fold's own test 20. This is the exact "corrected here, stale there" pattern. See **[CRITICAL] #1**. |
| **R1-I1** — §7.1 calls bypass a menu option; §7.2's menu had no bypass row | **FIXED** | §7.2 gains a `skip` row mapped to §7.1.1's `not verified`, with the reason recorded. §7.1.1's fourth provenance is now reachable and test 17 satisfiable. |
| **R1-I2** — pass 3 over the `ClassMDMK` subset is index-coupled to the full list | **FIXED** | §5.3.2 l.469–474 now says the subset must carry its original indices, cites `cardKey` returning `uniq: i + 1`, and names the wrong transcription ("compacting the subset into a fresh slice"). Verified: `seal/record.go:454` and `:464` both return `groupKey{… uniq: i + 1}`. |
| **R1-I3** — 24 words overrun `passphraseBytes`' 128-byte capacity | **FIXED** | §6.2.2 states the no-regrow requirement normatively with the orphaned-copy reason, and test 21 asserts on buffer identity. Verified: `gui/unlock_kdf.go:195` is `make([]byte, 0, 128)`, and the 215-byte figure is right (measured below). *(The cap being stated as an inequality, and the free-text path defeating the same requirement, are new findings **#4** and **#3**.)* |
| **R1-M1** — §3.3.2a's "source is not an admission input" vs §5.4.1's `compared` gate | **FIXED** | §3.3.2a l.298–303 separates the two checks explicitly: the class table is source-blind, the `compared` gate is a separate payload-level check. The word "admitted" no longer does double duty without warning. *(One residual ordering inconsistency — **[MINOR] #7**.)* |
| **R1-M2** — test 18 passes for the wrong reason on a corrupted record | **FIXED** | Test 18 now requires the alteration to survive the structural checks and names the substitution: "Alter a record to another *valid* record of the same class and length." It can now only pass on the AAD binding. |
| **R1-N1** — test 16's substring match fails a correct implementation | **FIXED** | Test 16 now requires an AST identifier comparison and cites `gui/plate_hook_test.go`. Verified: that file imports `go/ast` + `go/parser` and does `ast.Inspect` → `n.(*ast.Ident)` name comparison at l.197–203. The citation is real and does what the spec says. |
| **R1-N2** — O6 still listed as open though §5.5 and §5.6 both decide it | **FIXED** | O6 struck the way O5 was, citing §5.5 and §5.6. |

**Score: 9 FIXED, 2 PARTIAL, 0 NOT FIXED.**

Both partials are the pattern round 1 named: the section the finding pointed at was corrected and the
same rule left standing elsewhere. R1-C4's is in prose and in the test list; R1-C2's is in the scope
list and in two code sites the spec never mentions.

---

## Part 2 — new defects

### [CRITICAL] §5.4's "both container variants show the digest" and test 9 survive the R1-C4 fold verbatim, so the spec still mandates displaying a constant digest for a secrets-only sealed payload — and tests 9, 15 and 20 cannot all pass

**Where:** §5.4 l.499–500 and §8.3 test 9 (l.959–961), against §5.4's own table l.504–508, test 15
(l.976–978) and the fold's new test 20 (l.997–999).

**Consequence:** test 9 is normative and reads *"The digest is displayed for **both** container
variants and for every program that consumes from the region (§5.4)."* An implementer transcribing it
displays a digest when `pub_len == 0` — and EPD§6.6, quoted in this spec's own §5.4 at l.511–513,
says that value is *"the digest of an empty record set … a constant"*. Every fully-encrypted payload
on earth shows the same number, the operator's comparison matches every time against anything, and
R0-C2's silent authentication bypass is back, mandated by the test list rather than merely permitted.
That is the sealed variant's **main case** — a seed and nothing public.

Separately the three tests are mutually unsatisfiable, so the named suite is red by construction:

- test 9: digest displayed for **both** variants;
- test 15: a `pub_len == 0` sealed payload displays **NO** digest;
- test 20 (new this fold): a `pub_len == 0` sealed payload is consumable *"with no digest shown"*.

Test 9's second sentence is stale in a second way: *"A program that consumes payload-sourced input
without a compared digest fails"* — after this fold `compared` need not come from a digest at all.

**Why it is real:** the fold's diff touches §5.4.1 and tests 15–22 and nothing else in this area
(`git diff 8ddbe2e..fc5da46` has no hunk at §5.4's prose or at test 9). Round 1's suggested
resolution was two edits — *"delete l.443–444's 'both container variants' clause; amend test 9 to 'a
payload **with a public section** (`pub_len > 0`)'"* — and neither was made. The surviving sentence
sits four lines above a block headed **"CORRECTED after R0-C2 and R0-I2"** which exists to say that
sentence was wrong. The spec now asserts the rule and its correction side by side, twice.

**Suggested resolution:** delete the "both container variants show the digest" clause at l.499 (the
sentence's real subject is "every *program*", which the same line already says), and amend test 9 to
scope to `pub_len > 0` and to say "without an authenticated payload (§5.4.1)" rather than "without a
compared digest".

---

### [IMPORTANT] §2.2's "genuinely new" list still contains no device-side passphrase-entry work, and the spec never says whether the systemwide unlock is a new flow — under the reuse reading, two `!m.Valid()` guards reinstate R1-C2's permanent unopenability

**Where:** §2.2 l.124–134, against §8a, §8b, §8c and §6.2.2; `gui/unlock_kdf.go:168`, `:359`;
`bip39/bip39.go:119`.

**Consequence:** §2.2 is the section that scopes implementation — its own opening says "Building the
wrong thing here would mean re-inventing controls that exist". Its seven items are the flash region,
widened admission, routing, the session, the verification menu, **"`me` passphrase modes"** (host
only) and the emulator NFC source. The fold added, in decisions 8a/8b/8c and §6.2.2, a keyboard
picker, a per-invocation checksum gate on a *shared* function, a `done` terminator, a word-count
confirmation screen and a fixed-capacity KDF buffer — none of which appear in the list an implementer
sizes the work from. Round 1 asked for exactly this ("§2.2's 'genuinely new' list has to say so") and
it was not folded.

The concrete failure is worse than a sizing error, because the spec never says the systemwide unlock
is a *new* flow. Decision 1 freezes Sealed Payload, §3.2 says the two features share no state, but
§5.1 says the sealed variant is "as `MNEMBLOB`" and §8a says only "the operator picks the keyboard at
unlock". An implementer who reuses `unlockPassphraseFlow` meets two checksum gates the spec never
names:

- `gui/unlock_kdf.go:168` — `if !m.Valid() { … showError("Not a valid passphrase, check the words."); continue }`, an unbounded re-prompt loop;
- `gui/unlock_kdf.go:359` — `if !isMnemonicComplete(m) || !m.Valid() { return errUnlockChecksum }`, before the KDF.

And `Valid()` is stricter than the mask the spec does name. `bip39/bip39.go:119` returns **false for
every `len(m)%3 != 0`**, so with the mask removed and these left in place, N ∈ {2,4,5,7,8,…} is
rejected outright and N ∈ {3,6,…,24} is rejected for all but `1/2^(N/3)` of draws. That is R1-C2's
permanent-unopenability outcome, reached by an implementer who did exactly what §8b and §8c say.
§8c's own explanatory paragraph points at `refreshCands` alone and says *"Without this the feature is
broken at its default"*, which reads as a complete account of the breakage and is not one.

**Why it is real:** the spec applies the opposite discipline everywhere else it has a coupled second
site. §3.1 names both entry points and rejects a boolean because "a boolean can be passed wrongly and
the wrong value still compiles". §5.3.2 says "**pass 3 must be restructured, not merely permitted
through** … a fold that changed one would have shipped a container that admits nothing". §8b is the
same shape and gets neither treatment: it states a per-invocation gate without saying what carries the
per-invocation signal through `inputWordsFlow` (which Sealed Payload also calls), and without naming
the sites that enforce the gate outside `inputWordsFlow`.

**Suggested resolution:** add an item to §2.2 — "a systemwide unlock entry surface: keyboard picker,
per-invocation checksum gate, `done` terminator, count confirmation, fixed-capacity buffer"; state in
§8b that the systemwide unlock is a NEW flow (decision 1 freezes `unlockPassphraseFlow` and
`unlockAttemptOnce`, so their `!m.Valid()` guards are Sealed Payload's and are not reused); and say
which shape carries the gate through the shared `inputWordsFlow`, given §3.1 already rejected a
boolean for the analogous seam.

---

### [IMPORTANT] §8a routes a seed-equivalent passphrase through Go strings, which defeats §6.2.2's no-orphan requirement added in the same fold — and its stated mechanism ("both feed `seal.NormalisePassphrase`") is the very call `passphraseBytes` exists to avoid

**Where:** §8a l.55–62 against §6.2.2 l.770–779; `gui/unlock_kdf.go:186–189`, `:195`;
`gui/passphrase_flow.go:63–64`, `:76`, `:102`; `seal/open.go:76`.

**Consequence:** §6.2.2 is this fold's answer to R1-I3 and it is unconditional: *"The buffer is
allocated once at its maximum (≥ 215 bytes) and never regrows"*, because a regrow *"orphans an
unwipeable copy of a seed-equivalent secret"*. §8a's free-text mode makes that unachievable on its own
path. The ASCII keyboard accumulates into `kbd.Fragment`, a **Go string** — `gui/passphrase_flow.go:76`
is `kbd.Fragment = string(dst[:n])` and l.63 documents the consequence for the existing program:
*"RESIDUAL COPY (spec 5.3): seeding the keyboard converts dst[:n] to a string, which cannot be
wiped."* A string grown one keystroke at a time orphans a copy of **every prefix** of the passphrase,
which is strictly worse than the single regrow R1-I3 was raised on, on a secret this project has
already ruled seed-equivalent.

§8a then names the wrong mechanism: *"Both feed the same `seal.NormalisePassphrase`."* They do not,
and the reason is written in the code the spec is describing. `passphraseBytes`
(`gui/unlock_kdf.go:186–189`): *"`Mnemonic.String()` produces byte-identical output …, but it produces
a Go STRING, which cannot be zeroed. That is the whole reason this exists."* `seal.NormalisePassphrase`
(`seal/open.go:76`) is `strings.ToLower(strings.Join(strings.Fields(s), " "))` — two further
unwipeable allocations. An implementer transcribing §8a literally deletes `passphraseBytes` and
reintroduces, on the word path too, the defect §6.2.2 forbids two sections later.

Test 21 does not catch this: it is scoped to "entering 24 words", i.e. the word path, so it passes
green while the free-text path leaks. That is a false-PASS against §6.2.2's unqualified claim.

**Why it is real:** the *convergence* §8a rests on does hold — verified, not assumed:
`gui/unlock_kdf_test.go:464–471` (`TestPassphraseBytesIsSection81sNormalisedForm`) asserts
`string(passphraseBytes(m)) == seal.NormalisePassphrase(m.String())`, and `Mnemonic.String()`
(`bip39/bip39.go:196`) lowercases via `bytes.ToLower` and single-space joins, so `LabelFor`'s
uppercase wordlist does converge. So the ruling is sound and the header-field argument survives; what
is wrong is the mechanism sentence and the unscoped hygiene guarantee.

**Suggested resolution:** change §8a to "both produce EPD§8.1's normalised form — the word path via a
caller-owned `[]byte` (`passphraseBytes`), the free-text path via an equivalent that never
materialises a `string`"; and either scope §6.2.2's guarantee to the word path with the free-text
residue stated as an accepted, documented exposure, or add "a free-text entry path that writes into a
caller-owned buffer" to §2.2's new-work list.

---

### [IMPORTANT] §6.2.2 states the length cap as an inequality while requiring "`me` enforces the identical range and cap", and the restored user-supplied mode has no length bound at all

**Where:** §6.2.2 l.762–768 (the `length cap` row and the sentence after the table), l.776–777;
against decision 8 and §6.2.1's `user-supplied ASCII` row (l.728).

**Consequence:** the row reads *"length cap | **≥ 215 bytes**, NOT `MaxLen`"* and the buffer
requirement repeats *"(≥ 215 bytes)"*. `≥` is not a value. §6.2.2's own next sentence — *"`me`
enforces the identical range and cap at creation"* — cannot be implemented against an inequality: a
host that picks 256 and a device that picks 215 disagree, and the disagreement is silent until an
operator has sealed a payload. 215 is derived only from the *generated* worst case (24 × 8 + 23); the
user-supplied ASCII mode this fold restored has no derived bound whatsoever, and nothing stops `me`
from accepting a 300-character passphrase. The device then truncates (wrong KDF input, unopenable),
regrows (the orphan defect §6.2.2 just forbade), or refuses (unopenable). That is precisely the
R0-C4 shape — *"the host seals what the device cannot accept"* — inside the section written to close
it, and named after it.

**Why it is real:** §6.2.2 is entirely new in this fold, and so is the mode that has no bound. The
section's own table caption says these are *"Three host/device mismatches of the R0-C4 shape"*; a
fourth is created by the row that fixes the third. The measurement itself checks out — measured over
`bip39/wordlist.txt`: 2048 words, longest is **8** characters (`abstract`; 88 words are 8 long), so
24 × 8 + 23 = **215**, and `passphrase.MaxLen` is indeed **100** with the quoted comment
(`passphrase/passphrase.go:12–13`). Only the *form* of the bound is wrong.

**Suggested resolution:** replace the inequality with a single normative constant — e.g.
`MaxPassphraseLen = 256` bytes of normalised output — state that it binds **both** modes, and say
that `me` refuses at creation anything the device cannot enter.

---

### [IMPORTANT] Restoring user-supplied passphrases overrules a normative MUST NOT in EPD§2.2/§8 and the Rust-primary crate's own module doc, and decision 8 no longer marks the overrule

**Where:** decision 8 l.44–53 and §5.6 l.658 (`--passphrase-ask`), against §1's preamble (l.23–24),
`SPEC_encrypted_payload_delivery.md` l.53 and l.1148–1149,
`crates/me-cli/src/seal/passphrase.rs:4`, `crates/me-cli/src/main.rs:63–66`.

**Consequence:** EPD§2.2 item 1 is normative and unqualified: *"§7 therefore mandates a generated
passphrase, and **the CLI MUST NOT accept a user-supplied one**."* EPD§8 repeats it: *"The CLI
generates it from the OS CSPRNG. It **MUST NOT** accept a user-supplied passphrase — see §2.2 item
1."* `passphrase.rs`'s module doc — the Rust-primary normative record — opens *"GENERATED, never
user-supplied"*, and `me`'s own `--help` (`main.rs:63–66`) tells the operator the passphrase is
generated because a chosen one *"does not survive an offline attack"*. §5.6 adds `--passphrase-ask` to
the same binary. An implementer therefore has a direct conflict with a MUST NOT, and under this
project's Rust-primary rule the edit to `passphrase.rs`'s doc has to land first — and nothing in §2.2,
§9 or §10 schedules it.

**Why it is real:** §1's preamble commits the spec to marking this: *"several overrule a documented
prior decision and are marked where they do."* The **pre-fold** decision 8 did mark it, from the other
side — *"which restores `crates/me-cli/src/seal/passphrase.rs`'s 'GENERATED, never user-supplied'
rather than overruling it"* — and the fold deleted that sentence without replacing it. The reversal
turned an acknowledged non-conflict into an unacknowledged conflict, and removed the only text that
pointed at it. This is flagging a decision's collision, not arguing against the decision: the mode
stands, but the two documents and one crate that forbid it have to be named and re-decided.

**Suggested resolution:** add one clause to decision 8 marking the overrule, scoping it (`me sysw`
only; `me seal` and the `MNEMBLOB` container keep EPD§2.2 item 1 unchanged, since Sealed Payload is
frozen by decision 1), and file a follow-up owned by the Rust phase to amend `passphrase.rs`'s module
doc and EPD§2.2/§8 to say the prohibition is container-scoped.

---

### [IMPORTANT] "A successful AEAD open sets `compared`" is unscoped, so for a sealed payload that *has* a public section the operator comparison stops being required — and the spec permits passphrases it measures at 22 bits and "0 bits"

**Where:** §5.4.1 l.583–594 (the two-route table and the "strictly stronger" paragraph), against §6.1's
table l.694, §6.2.1 l.728, F2 (l.323), `--allow-weak` (l.660) and §5.4 l.530–533.

**Consequence:** the table's second row is `a successful AEAD open (ct_len > 0) → yes`, with no bound
on `pub_len` and none on passphrase strength. So for a sealed payload **with** public records, the
open alone satisfies `compared` and the operator's digest comparison — which §5.4 still displays and
still calls the control — no longer gates anything. That matters exactly where the spec says it
matters: §5.4 l.530–533 describes the funds-loss path as *"an attacker swaps an `mk1` for one encoding
**their** xpub, the tag still verifies, and the operator engraves a steel backup of a wallet they do
not control"*. "The tag still verifies" is the case where the attacker holds the key — i.e. a weak
passphrase — and this spec permits 2 words (§6.1: 22 bits, **42 seconds** on one GPU) and
user-supplied ("treated as 0 bits") behind `--allow-weak`. Under the pre-fold rule the operator had to
compare a digest they recorded out of band, and an altered public section changes that digest.
Under the new rule they are not asked, because the payload authenticated itself with a key the spec
itself says is worth 42 seconds.

**Why it is real:** the justification is stated as unconditional — *"a **cryptographic** guarantee and
strictly stronger than a human reading sixteen hex digits off a screen"* — and the spec's own §6.1
table refutes it for the low end of the range it allows. F2 flags `weak` but flags do not gate, by
§3.3.3's own rule ("Evaluated after admission, never as part of it"). The fix R1-C4 needed was for
`pub_len == 0`, where no comparison is possible; the fold granted it for every sealed payload, which
is wider than the finding.

**Suggested resolution:** scope the AEAD row to `pub_len == 0` — where there is genuinely nothing to
compare — and keep the operator comparison as the `compared` route whenever a digest exists; or, if
the wider rule is wanted, exclude payloads flagged `weak` (§6.2.1) and qualify "strictly stronger" to
"strictly stronger **at or above the §6.2 cliff**".

---

### [MINOR] §3.3.2a places the `compared` gate before classification, §5.4.1 places it at consumption, and §3.2.1's store shape implies the second

**Where:** §3.3.2a l.298–303, against §5.4.1 l.574–575 and §3.2.1 l.198–206.

**Consequence:** the fold's new paragraph says the gate is *"a separate, **earlier** check on the
payload — it asks whether this payload was authenticated at all, **before any record of it is
classified**"*. §5.4.1 says *"A record is **admitted for consumption** only when `compared` is true"*,
and §3.2.1's store holds `records` beside `compared`, which only makes sense if records exist before
the flag is set. An implementer taking §3.3.2a literally refuses to classify an uncompared payload and
so can never present the records the load screen needs; one taking it as licence to check only at load
loses the consumption-time gate §5.4.1 specifies. Both readings are available.

**Why it is real:** the paragraph is entirely new fold text written to resolve R1-M1, and it resolves
it by relocating the gate rather than by scoping it. The relocation is not carried anywhere else.

**Suggested resolution:** change "before any record of it is classified" to "before any record of it
is **consumed**", which is what §5.4.1 and §3.2.1 both already say.

---

### [MINOR] §9's O2 row still records the reversed decision — "it removed a passphrase mode", and the unlock screen "uses the WORD keyboard"

**Where:** §9 O2, l.1011, against decision 8 l.44–53 and 8a l.55–62.

**Consequence:** O2 is the row that asks *"Which keyboard the unlock screen uses"*, and it is struck
as **RESOLVED** with the answer "the WORD keyboard" plus the claim that the finding "removed a
passphrase mode". Decision 8a answers the same question differently — *the operator picks*, word or
free-text ASCII — and decision 8 restored the mode. §9 is the reconciliation surface; a reader
checking whether the keyboard question is settled is told it is settled the wrong way, by a row that
looks authoritative because it is struck through.

**Why it is real:** the brief lists "no text from the reversed decision survives" as machine-verified.
It does survive, here — this is the one place a grep for the reversal's vocabulary would miss, because
the row says "removed a passphrase mode" rather than "user-supplied" or "DROPPED". The fold struck O6
and left O2 untouched.

**Suggested resolution:** rewrite the O2 row: *"RESOLVED 2026-08-11 by decision 8a: the operator picks
the keyboard at unlock — BIP-39 word (default landing) or free-text ASCII. R0-C4's finding about
`unlockPassphraseFlow` was true of Sealed Payload's flow and is not a constraint on the systemwide
one."*

---

### [NIT] §6.2.2 justifies its character range by the same function whose length cap the next row forbids

**Where:** §6.2.2's table rows 1 and 2 (l.764–765); `passphrase/passphrase.go:23–38`.

**Consequence:** row 1's reason is *"`passphrase.ValidatePassphrase` rejects anything else as
`ErrNonASCII`"* and row 2 says `passphrase.MaxLen` must **not** apply. But `ValidatePassphrase` is one
function that enforces both — verified: it rejects `r < 0x20 || r > 0x7E` and then `n > MaxLen`
(=100). Calling it is the natural transcription of row 1 and silently imports exactly the cap row 2
forbids, on a 12-word passphrase that already reaches 107 bytes.

**Suggested resolution:** say in row 1 that the systemwide path enforces the *range* only, and must
not call `passphrase.ValidatePassphrase`, which belongs to the engraving program and carries its
plate-capacity cap.

---

## VERDICT: 1 Critical, 5 Important, 2 Minor, 1 Nit

---

### What the fold got right, recorded so a later round does not re-derive it

- **R1-C4's consumption fix is the right construction.** The two-route `compared` table covers the
  space with no gap: plaintext (`ct_len == 0`) has only the operator route, sealed has the open, and
  there is no payload consumable without one or the other. The reasoning — that an AEAD tag over
  `AAD = header ‖ public section` authenticates more than an operator can — is correct; only its
  *scope* is wrong (finding #6), not its shape.
- **R1-C2's ruling is achievable at the seam it names.** Traced: the mask is confined to
  `inputWordsFlow`'s three closures (`gui/gui.go:740` `onLastWord`, `:742–745` `updateKeys`,
  `:748–749` `completeWord`) plus `refreshCands` at `:758–768`, all driven by one `cands` local. A
  per-invocation signal reaching those four points turns it off without touching
  `updateValidBIP39Keys`, so **seed entry loses nothing** — the per-word wordlist mask, the match
  counter and `completeBIP39Word` are all on the non-last-word branch and are untouched by the gate.
  The mask does **not** reach further than §8b assumes. What reaches further is `Valid()` itself, at
  the two sites in finding #2.
- **§8a's byte-identity claim is true**, by measurement rather than assertion:
  `gui/unlock_kdf_test.go:471` asserts `string(passphraseBytes(m)) == seal.NormalisePassphrase(m.String())`,
  and `Mnemonic.String()` (`bip39/bip39.go:196–205`) lowercases with `bytes.ToLower`, so `LabelFor`'s
  uppercase wordlist converges. The "no header field declares the type" argument stands.
- **§6.2.2's 215 is correct.** Measured over `bip39/wordlist.txt`: 2048 words, max length **8**
  (`abstract`, 88 words at that length), so 24 × 8 + 23 = 215 and 12 words reach 107. `MaxLen` is 100
  with the quoted comment. No other cap binds this path — `codex32`'s length constants are record
  lengths, and `MaxSectionLen` (8191) is nowhere near.
- **R1-I2's fix is right about the code.** `cardKey` returns `groupKey{hrp:'d', uniq: i + 1}` at
  `seal/record.go:454` and `{hrp:'k', uniq: i + 1}` at `:464` — the index really is baked into the
  group key, so "filter the iteration, never the indices" is the correct instruction.
- **Tests 19–22 and the amended 16 and 18 can each fail.** 19 fails whenever any checksum gate
  survives on the passphrase path (and "draw many" is the right instruction — one fixed passphrase
  passes 1 time in 16); 20 fails on a consumption gate that requires a digest; 21 fails against
  today's `make([]byte, 0, 128)`; 22 fails on a confirmation screen that reports the intended count;
  16 now fails only on a real `seedEntryFlow` reference, and `gui/plate_hook_test.go:197–203` is a
  working precedent; 18 can now only pass on the AAD binding. The one caveat is test 21's scope,
  noted inside finding #3.
- **R1-C3, R1-I1, R1-M2, R1-N1 and R1-N2 are clean folds** — each replaced the wrong claim rather
  than annotating it, and R1-C3 fixed both of the two sites round 1 named rather than the headline
  one.
