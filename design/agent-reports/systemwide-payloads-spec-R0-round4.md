# R0 round 4 — re-review of the fourth fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `8383cc6` (the fold, which also carries
  `scripts/spec-check.py` at `de8a04f`).
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round3.md` at `f0ef3bc`,
  and the isolated fold diff `f0ef3bc..8383cc6` — measured, `git diff --stat`: **2 files, 244
  insertions / 61 deletions**; the spec's own hunk is **229 changed lines**.
- **Questions answered:** (1) did the fold fix each of round 3's 9 findings; (2) did the
  restructure introduce a new defect. **This is not a fresh audit.** No section the fold did not
  touch was reopened except where the fold made it wrong.
- **The cliff ruling is treated as a DECISION.** It is not argued against anywhere below. Per the
  brief it **is** flagged wherever a rule still relies on the cliff meaning strength, which it does
  at six sites.
- **Machine-checked before any judgement was formed** (values pasted, never described):
  - `scripts/spec-check.py design/SPEC_systemwide_payloads.md` → **exit 0**; 24 citations resolved,
    10 pinned to their content, 3 rules defined once, 22 tests numbered 1..22 without gaps.
  - **All five `[term]` references resolve to a §12 definition.** `grep` returns 27 bracketed
    references; `[cliff]`→§12.1, `[compared]`→§12.2, `[identity]`→§12.3, `[digest-shown]`→§12.4,
    `[passphrase-bounds]`→§12.5. **No reference points at a rule §12 does not define.**
  - `bip39/wordlist.go:9` → `const LongestWord = 8`. `24 × 8 + 23 = 215`. **§12.5's arithmetic and
    its named constant are both correct**, and `bip39/wordlist.go` really does carry the wordlist
    (the `words` constant), so §12.1's citation of it is valid.
  - `grep -x` against `bip39/wordlist.txt`: `correct` **IN**, `horse` **IN**, `battery` **NOT**,
    `staple` **NOT**; `abandon` **IN**. **§12.1's two worked examples are both true as stated.**
  - `passphrase/passphrase.go:13` → `const MaxLen = 100`, with the comment §12.5 quotes verbatim
    ("a plate-capacity limit chosen for legibility, not a BIP-39 rule"). `ValidatePassphrase`
    enforces `r < 0x20 || r > 0x7E` **and then** `n > MaxLen` — one function, both constraints, so
    §6.2.2's new "do not call it" paragraph is correct.
  - `inputWordsFlow` non-test call sites, measured: `gui/seedxor_polish.go:52`,
    `gui/derive_xpub.go:90`, `gui/unlock_kdf.go:160`, `gui/gui.go:2346`, `gui/gui.go:2445` —
    **exactly five**, all of which keep the mask. §2.2 item 8's "shared with five call sites" is
    **exact**.
  - `gui/gui.go:758` = `refreshCands := func() {`; `gui/unlock_kdf.go:168` = `if !m.Valid() {`;
    `:359` = `if !isMnemonicComplete(m) || !m.Valid() {`. Both rows of §2.2 item 8's table are
    exact.
  - `inputWordsFlow` builds `NewKeyboard(ctx, wordKeys)` at **`gui/gui.go:728`**, type `Keyboard`
    (`gui/gui.go:1048`). `NewPassphraseKeyboard` (`gui/passphrase_keyboard.go:76`) returns
    `*PassphraseKeyboard` — **a different type**, constructed only at `gui/passphrase_flow.go:74`
    and `gui/gui.go:641`. See finding 2.
  - **Gate mutation-tested — eight mutants run, results pasted in finding 4.**

---

## Part 1 — did the fold fix each round-3 finding?

| round-3 finding | status | reason |
| --- | --- | --- |
| **R3-C1** — §5.4.1's prose and test 20 mandate the unconditional AEAD rule | **PARTIAL** | The two sites that carried the Critical are gone: "Opening it is the proof" is deleted outright and test 20 is now two-sided (`[cliff]`-above consumable, below NOT). **The third site the finding named is untouched**: §3.2.1 l.233 still glosses `compared` as "the operator compared the digest for THIS identity" — the fold edited the `weak` line directly beneath it and left this one. See **[IMPORTANT] 3**. |
| **R3-C2** — the cliff is a word count but §6.2.1 defines it by MODE and BITS | **PARTIAL** | §12.1 is a clean, correct, unambiguous single definition and it resolves the counting-rule ambiguity the finding raised (all-tokens-in-list, not ≥5-in-list-among-any). §6.2.1 opens by deferring to it. But §6.2.1's own table cell still asserts `[cliff]` = "**below**, always" *by mode*, its "deterministic rule an implementer transcribes" blockquote is still mode-shaped, and **five further sites still describe the cliff as strength** — including two §12.1 explicitly forbids. See **[IMPORTANT] 1**. |
| **R3-I3** — item 8 forbids `unlockPassphraseFlow`; the mask lives in `inputWordsFlow` | **PARTIAL** | The two-obstacle table is a real fix and every claim in it is machine-exact (verified above), including the five-call-site count. But it presents itself as complete and omits the third obstacle — `inputWordsFlow`'s fixed length, absent terminator and absent return value — and §2.2 item 9 puts the `done` key on a keyboard type the word path never constructs. See **[IMPORTANT] 2**. |
| **R3-I4** — §8a's "Both feed the same `seal.NormalisePassphrase`" | **PARTIAL** | The mechanism sentence — the Important horn — is gone and replaced correctly: the *rule* is shared, the *function* is not the carrier, normalise into the buffer. The finding's second half is not done: §6.2.2a still scopes its residue acceptance to "the free-text path" while §8a's new imperative is unqualified. See **[MINOR] 8**. |
| **R3-M5** — "exactly 215 bytes" does not say raw or normalised | **PARTIAL** | §12.5 now reads "over the NORMALISED string". §6.2.2 — the section headed *"what is ENTERABLE"* — still states its own 215 with no measurement basis and no pointer to §12.5. See **[MINOR] 5**. |
| **R3-M6** — "cannot be consumed at all" reaches no screen and no test | **NOT FIXED** | `grep -n "^| F[0-9]"` returns F1–F4 only; §8.3 still ends at test 22. The dead end has no screen, no test, and — newly relevant — `me --allow-weak` will now create the unusable payload with no warning that it is unusable. See **[MINOR] 6**. |
| **R3-M7** — the gate is green on defects it names as its own purpose | **PARTIAL** | The docstring lie is fixed honestly and the `EXPECT` table is a real mechanism that caught a bad pin of the author's own — both good. But **mut1 still survives, measured** (exit 0); the mode-count line is still case-sensitive and still killed only by `REQUIRED`; the dead `nums` at `check_tests` l.181–182 is still there; no `--self-test`. And the new `SINGLE_DEF` check is phrasing-shaped, killing **1 of 5** second-definition mutants. See **[IMPORTANT] 4**. |
| **R3-M8** — nothing schedules the EPD§2.2/§8 and `passphrase.rs` amendments | **NOT FIXED** | `grep -n "F-12[0-9]"` returns F-123 and F-124 only; §2.2 item 6, §9 and §10 are unchanged, and `SPEC_encrypted_payload_delivery.md:53` still carries the unqualified "the CLI MUST NOT accept a user-supplied passphrase". See **[MINOR] 7**. |
| **R3-N9** — the buffer blockquote's inequality; the unrestricted `ValidatePassphrase` citation | **FIXED** | The blockquote now reads "allocated once at exactly `[passphrase-bounds]`' 215 bytes (§12.5) and never regrows", and §6.2.2 gained a standalone paragraph: "**Do not call `passphrase.ValidatePassphrase` on this path.** It bundles the range with `MaxLen = 100`… Take the range; write the check." Both sites, both replaced. |

**Score: 1 FIXED, 6 PARTIAL, 2 NOT FIXED.**

**The restructure worked where it was aimed, and stopped short of finishing.** §12.1, §12.2 and §12.3
are correct, unambiguous, mutually consistent, and every one of the 27 `[term]` references resolves —
attack #1's third risk (a reference to a rule §12 does not define) is **clean**, and no definition
lost a qualifier in the move except the one noted in finding 5. What did *not* happen is the sweep:
§6.2.1, §3.2.1, decision 8, §6.1 l.737 and §6.2 l.759/762 still state or imply versions of rules §12
now owns. That is the same cause rounds 1, 2 and 3 each named, for the fifth round — but it is now
**contained to one term and five sites**, and the gate that was supposed to make it structural is the
reason it survived (finding 4).

---

## Part 2 — new defects

### [IMPORTANT] 1. §6.2.1's table and its "deterministic rule an implementer transcribes" still define `[cliff]` by MODE, which §12.1 says is not an input — and they give `me` the opposite answer to §5.6 on one real input. Five further sites still describe the cliff as strength, two of them in terms §12.1 explicitly forbids

**Where:** §6.2.1 l.782–786 (the table) and l.816–817 (the transcription blockquote), against
§12.1 l.1156–1164 and §5.6 l.708. Strength-residue sites: decision 8 l.61; §5.4.1 l.624 and l.628;
§6.1 l.737; §6.2 l.759 and l.762; §3.3.3 F2 l.362.

**Consequence.** §6.2.1's header sentence is exactly right — *"The gate is `[cliff]` (§12.1) — a word
count over the normalised string — and NOT the mode"* — and then the section states the mode gate
twice more anyway:

| site | text | why it is a second definition |
| --- | --- | --- |
| l.785 | `| user-supplied ASCII | not estimated | **below**, always — its tokens are not wordlist entries |` | asserts a `[cliff]` **value** from the **mode**, under a column literally headed `` `[cliff]`? ``. The reason given is false: a free-text ASCII passphrase *may* consist entirely of wordlist tokens |
| l.816–817 | "So the deterministic rule an implementer transcribes: **Secret content + (user-supplied OR no passphrase OR generated with `N < 5`) ⇒ `me` refuses without the explicit flag.**" | the strongest transcription instruction in the document, and its first disjunct is a mode test |

Feed one input to §6.2.1 and to §5.6: a **user-supplied passphrase of five BIP-39 words**, typed on
the free-text keyboard, over secret content.

- **§6.2.1 l.816:** mode is user-supplied ⇒ `me` **refuses** without `--allow-weak`.
- **§5.6 l.708:** `--allow-weak` is *"required by §6.2.1 when secret content meets a **sub-cliff**
  passphrase"* — and by §12.1 this passphrase is **above** the cliff ⇒ the flag is **not required**.

Two normative statements in one spec, opposite answers, one input. Then `me` prints
*"user-supplied, unmeasured, below the cliff"* (l.820) while the device — which has no mode to read,
by §8a's design — computes `[cliff]`-above from the string, leaves `weak` **false**, and shows **no
F2 warning**. Creation-time and use-time tell the operator opposite things about the same payload.

**Why this is IMPORTANT and not Critical, stated so the controller can check the reasoning.** I
looked specifically for the R0-C4 shape and it is **not** present. The only direction the two rules
diverge is *host stricter than device*: `me` computes above-cliff **only** for generated `N ≥ 5`,
and for those the device agrees (N wordlist tokens joined by single spaces normalise to N in-list
tokens; `strings.Fields` cannot merge them). **There is no input `me` seals as above-cliff that the
device computes as below**, so no payload is made unconsumable by this. The security-bearing
consumers of `[cliff]` — §12.2's `compared`, §3.2.1's `weak`, F2 — all reference §12.1 directly and
are clean. What is contaminated is the host's flag requirement and its printed message.

**Five sites still describe the cliff as strength**, which the brief asks be flagged and which §12.1
l.1166 forbids in capitals (*"IT IS A SPEED BUMP, NOT A STRENGTH MEASURE, AND NOTHING MAY DESCRIBE
IT AS ONE"*):

| l. | text | why it breaks |
| --- | --- | --- |
| 624 | "**The AEAD route is scoped to strong keys**" | it is scoped to ≥5 wordlist tokens. `abandon` ×5 is above the cliff and worth zero bits, so the route is *not* scoped to strong keys — §12.2 says so itself ("the open is forgeable in precisely the cases `[cliff]` waves through"). This is the section heading sentence of the paragraph that justifies §12.2 |
| 628 | "treats user-supplied as **0 bits**" | §6.2.1 no longer does; its entropy cell now reads "not estimated" |
| 61 | "§6.2.1 **prices the mode at 0 bits** so nothing downstream mistakes it for protection" | same dangling citation, and it is the *entire* stated justification for overruling EPD§2.2 item 1's MUST NOT |
| 737 | "the *shape* — a cliff between 4 and 5 words — … is **the only property the rule in §6.2 rests on**" | §6.2's rule now rests on a word count. §6.1 l.749–755 was added to say exactly this and l.737 was left as written |
| 759 / 762 | "**Below 5 words (55 bits)**" / "secret material is **protected by** less than the cliff" | the parenthetical equates count with entropy and "protected by" is a strength claim; §12.1 l.1183 severs both |

F2's screen text (l.362) is a sixth: its *condition* now correctly references `[cliff]`, but it still
says "this secret is weakly protected", a strength judgement the flag no longer makes. §3.2.1's
`weak` field took the honest route instead — "Named `weak` for brevity only… `weak == false` does not
mean strong" — and F2 did not get the same treatment.

**Why it is real:** measured, not inferred. `git diff f0ef3bc..8383cc6` has **no hunk** at l.61,
l.624–632, l.737 or l.755–765. `grep -n "0 bits\|strong key\|protected by"` over lines 1–1139 returns
l.61, l.362, l.624, l.628, l.762, l.786 — six live sites, three of them untouched by a fold whose
stated purpose was to make this impossible.

**Suggested resolution:** (1) replace §6.2.1's third column with a single row-independent sentence —
*"`[cliff]` is computed from the normalised string in every mode; see §12.1. `me` reports the mode's
entropy and the string's `[cliff]` separately"* — and delete the `below, always` cell; (2) restate
l.816's blockquote in `[cliff]` terms: *"Secret content + a passphrase that is not `[cliff]`-above ⇒
`me` refuses without the explicit flag"*, which also makes it agree with §5.6 l.708 by construction;
(3) l.624 → "The AEAD route is scoped to `[cliff]`-above passphrases, and R2 is why"; (4) delete the
"0 bits" clauses at l.61 and l.628 and rest decision 8's overrule on something §12.1 preserves —
`[cliff]` places every non-BIP-39 password below it, which is the same protection the 0-bit pricing
was buying; (5) l.737 → "…which is where §12.1's threshold came from, though §6.2's rule now rests on
the word count rather than on this shape"; (6) l.759 → "Not `[cliff]`-above over secret content", and
delete "protected by" at l.762; (7) F2's text → "this secret's passphrase is under 5 BIP-39 words";
(8) **add the two named tests §12.1 currently has none of** — `abandon` ×5 is `[cliff]`-above,
`correct horse battery staple` is below, and the mode is not an input to either. §12.1 is now the
most-referenced normative rule in the document and **no test in §8.3 exercises it**; test 5's
"sub-cliff" is ambiguous between §12.1 and §6.2.1 l.816 for precisely the input above.

---

### [IMPORTANT] 2. §2.2 item 8's table names two obstacles and there is a third — `inputWordsFlow` has a fixed length, no terminator and no return value — and §2.2 item 9 puts the `done` key on `PassphraseKeyboard`, a type the word-entry path never constructs

**Where:** §2.2 item 8 l.149–163 and item 9 l.164, against §8a l.65–67, §8b l.79–83 and §8c
l.85–109; `gui/gui.go:727`, `:728`, `:779`, `:790–794`, `:1048`, `:2541`;
`gui/passphrase_keyboard.go:76`, `:92`; `gui/unlock_kdf.go:359`.

**Consequence.** This is the direct answer to the brief's question 3: **the list is not complete.**
Both rows are exact — I verified each — but they are the two obstacles to *checksum-free* entry, not
the obstacles to *arbitrary-N* entry. Traced:

```
func inputWordsFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic, selected int, title string)
```

- **No return value.** Its only exits are `return` on Back (`:779`) and `return` when
  `selected == len(mnemonic)` (`:792–794`). It cannot report how many words were entered.
- **Length is `len(mnemonic)`, fixed by the caller**, and the slice header is passed by value, so
  the flow cannot grow it. Arbitrary N therefore has to be represented as a 24-slot mnemonic with
  `-1` sentinels in the tail — and `isMnemonicComplete` (`gui/gui.go:2541`) returns false on any
  `-1`, which is how `unlock_kdf.go:164` and `:359` distinguish "operator left" from "operator
  finished". **In that representation a 5-word passphrase is byte-indistinguishable from an
  abandoned entry**, which is the same permanent-unopenability class item 8 exists to prevent,
  reached by an implementer who complied with both of its rows.

Item 9 is where the terminator is supposed to live, and it is on the wrong object. Measured:

| what | type | where |
| --- | --- | --- |
| the keyboard `inputWordsFlow` builds | `*Keyboard` (`gui/gui.go:1048`) — letters + `⌫`, no function row | `gui/gui.go:728`, `NewKeyboard(ctx, wordKeys)` |
| the keyboard §2.2 item 9 and §8c put the `done` key on | `*PassphraseKeyboard` | `gui/passphrase_keyboard.go:76`, used only at `gui/passphrase_flow.go:74` and `gui/gui.go:641` |

§8c cites `gui/passphrase_keyboard.go:80`'s per-instance pattern, which is `newPPKeyboard(ctx, bool,
bool)` — a constructor `Keyboard` does not have and cannot inherit. So an implementer transcribing
§2.2 item 9 verbatim adds a `done` key to the free-text keyboard, and **the word path — which §8c's
own rationale is entirely about (`refreshCands`, "15 of every 16", "any N divisible by 3") — still
has no terminator.** Test 22 is then unimplementable on the path it was written for.

**Why it is real:** §8c's rationale and §8c's mechanism describe different keyboards, and the
mismatch is machine-verifiable in one grep: `grep -rn "NewPassphraseKeyboard(" ` returns no hit in
`gui/gui.go`'s word-entry region, and `inputWordsFlow`'s only keyboard construction is `NewKeyboard`
at `:728`. This is R3-I3's shape repeated one level down: the fold corrected which *function* is
blocked and left the *affordance* attached to the wrong *type*.

**Suggested resolution:** add a third row to item 8's table — *`| length + terminator |
`gui/gui.go:727`, `:792` | returns nothing and exits only at `len(mnemonic)`; arbitrary N needs a
signature that returns the count |`* — and correct item 9 and §8c to say the `done` key is a
per-instance opt-in on **`Keyboard`** (`gui/gui.go:1069`), constructed by `inputWordsFlow` at `:728`,
following `passphrase_keyboard.go:80`'s pattern rather than living on that type. Then state in §8b
that the arbitrary-N flow must not use `isMnemonicComplete` (`gui/gui.go:2541`) as its completion
test, and add a clause to test 19 asserting that a 5-word entry is distinguishable from an abandoned
one.

---

### [IMPORTANT] 3. §3.2.1's store, a NORMATIVE block, still glosses `compared` as the operator route only — the one site of R3-C1 the fold did not touch, with the `weak` line directly beneath it rewritten in the same commit

**Where:** §3.2.1 l.233, against §12.2 l.1188–1192 and §12's own preamble l.1148–1149.

**Consequence.** The block reads:

```
compared   bool          the operator compared the digest for THIS identity
weak       bool          sealed, and its passphrase is NOT `[cliff]`-above
                         (§12.1). Named `weak` for brevity only: ...
```

The `weak` line was rewritten by this fold to reference `[cliff]`. The `compared` line immediately
above it was not, and it states a **definition** — "the operator compared the digest" — that §12.2
contradicts by adding a second route. §12's preamble says every other section "references it by name
… and states no version of its own"; this one states a version and names nothing.

For a secrets-only sealed payload there is no digest to compare, so an implementer who transcribes
§3.2.1 builds a `compared` that can never be set for the sealed variant's principal case — which is
R1-C4's original Critical, and the reason §12.2 exists.

**Why it is real:** measured — `git diff f0ef3bc..8383cc6` shows the hunk at §3.2.1 changing exactly
one line (`weak`) and leaving l.233 untouched, and R3-C1 named this site explicitly with a suggested
replacement. It is invisible to the gate: `SINGLE_DEF`'s `compared` pattern requires
`` `?compared`? (is|flag is) (set|satisfied) by (EITHER|either|a|the) ``, which a field gloss cannot
match — confirmed by the gate being green on the live text.

**Why it is Important and not Critical:** the failure direction is fail-safe (a payload refuses
consumption rather than being falsely marked authenticated), and §12.2 is unambiguous and clearly
labelled the single source. It blocks because it is a second statement of a §12-owned rule inside a
NORMATIVE block — precisely the class this gate is chartered to close.

**Suggested resolution:** l.233 → `compared   bool          the payload was authenticated for THIS
identity — by either route in `[compared]` (§12.2)`.

---

### [IMPORTANT] 4. The single-definition gate kills 1 of 5 second-definition mutants, and §12 tells the reader it makes a second definition "a build failure, not a review finding" — so the restructure's durability rests on a control that is documented by intent rather than behaviour, which is the F-123 mistake this spec invokes five times

**Where:** `scripts/spec-check.py` `SINGLE_DEF` l.150–160 and `check_single_def` l.163–176, against
§12's preamble l.1150–1152; also `FORBIDDEN[3]` l.57–63 and `check_tests` l.181–182.

**Consequence.** **Eight mutants run against the committed gate. Measured results:**

| mutant | inserted before `### 6.2` (i.e. outside §12) | gate |
| --- | --- | --- |
| **M1** | R3's mut1 verbatim — "for every variant, sealed or not, the digest is shown" | **exit 0, SURVIVED** |
| **M2** | "A passphrase counts as above the cliff when it has at least five BIP-39 words." | **exit 0, SURVIVED** |
| **M3** | "A user-supplied passphrase always counts as below the cliff, whatever it contains." | **exit 0, SURVIVED** |
| **M4** | "`compared` becomes true when the operator checks the digest, or when any AEAD open succeeds." | **exit 0, SURVIVED** |
| **M5** | "Payload identity is the SHA-256 of the region bytes under the MNEMSYSW/id/v1 label." | exit 1 — killed |
| **M6** | "The payload identity is simply the EPD digest of the public section." (R1-C3's defect, reworded) | **exit 0, SURVIVED** |
| **M7** | `gui/derive.go:19` redirected to `:3` (a non-pinned citation) | exit 0 — **documented blind spot, not a finding** |
| **M8** | mode count flipped, "Two modes in `me`" / "Two passphrase modes" | exit 1 — killed, but by `REQUIRED["three modes"]`, exactly as round 3 said; `FORBIDDEN["mode count"]` is still case-sensitive |

**One of five second-definition mutants dies.** And M3 is not hypothetical: it is a paraphrase of
text that is **in the spec right now** — §6.2.1 l.785's `` `[cliff]` `` cell — which the gate passes.
The check finds definitional *phrasings*; round 3's own lesson, quoted in `de8a04f`'s commit message,
was "match the claim, not a phrasing", and that lesson was applied to `SINGLE_DEF`'s
case-sensitivity but not to its shape, and not at all to `FORBIDDEN[3]` (M1 survives, measured).

The gap that matters most is the **claim**, not the coverage. §12 l.1150 tells every future folder:
*"`scripts/spec-check.py` enforces that: a definitional phrasing found outside this section is a
build failure, not a review finding."* A folder who believes that will stop looking. The docstring is
scrupulously honest about what it does not cover — the fold fixed that, and it is the best change in
this commit — but **§12 makes the over-claim the docstring refuses to make**, and §12 is what a
folder reads.

Two smaller items: `[digest-shown]` and `[passphrase-bounds]` are §12 definitions with **no
`SINGLE_DEF` entry at all**, and both already have full second statements outside §12 (§5.4 l.537–551
and §6.2.2 l.828–832) — finding 5 is one of them. And `check_tests` l.181–182's dead `nums` survives
round 3's report.

**Why it is real:** measured, exit codes pasted, not argued. The gate is a genuine net win — it
caught its author mid-commit, the `EXPECT` table found a bad pin, and `SINGLE_DEF` is the right
structural idea. These are repairs to it.

**Suggested resolution:** (a) restate each `SINGLE_DEF` entry claim-shaped rather than
phrasing-shaped — for `cliff`, something like
`r"(above|below) the cliff\b(?![^.\n]*§12)"` scoped to sentences that *assign* a value, and for
`compared` a pattern keyed on `compared` within 40 characters of `set|true|satisfied|becomes`;
(b) add `SINGLE_DEF` entries for `digest-shown` and `passphrase-bounds`; (c) apply round 3's
suggestion (a) to `FORBIDDEN[3]` and add `re.I` to the `FORBIDDEN` loop, so M1 and M8 both die on
their own invariant; (d) **add the `--self-test` round 3 asked for, seeded with M1–M6 above**, so the
next fold cannot silently weaken the check; (e) delete the dead `nums`; (f) soften §12 l.1150 to what
the tool actually does — *"`scripts/spec-check.py` fails on the definitional phrasings it knows; it
cannot recognise every rewording, so a fold still owes the sweep"*.

---

### [MINOR] 5. §6.2.2 restates all three `[passphrase-bounds]` values without deferring to §12.5, and drops the "over the NORMALISED string" qualifier under a heading that says ENTERABLE

**Where:** §6.2.2's table l.828–832 and its heading l.823, against §12.5 l.1221–1228.

**Consequence.** §12.5's length row is *"exactly 215 bytes, host and device, **over the NORMALISED
string**"* — R3-M5's fix, and correct. §6.2.2's row is *"exactly 215 bytes, host and device"*, under a
heading reading *"Host and device must agree on what is **ENTERABLE**"*, which points the reader at
entry-time measurement. `NormalisePassphrase` collapses whitespace runs, so raw ≥ normalised: a
219-character user-supplied passphrase with five double spaces normalises to 214. Measured at entry
it is refused; measured normalised it is accepted. That is R3-M5's exact scenario, still open at the
site that faces the implementer.

Structurally it is also the restructure's stated rule broken for a second term: §6.2.2 states its own
version of a §12-owned rule and does not reference `[passphrase-bounds]` — and the gate has no
`SINGLE_DEF` entry for that term (finding 4).

**Suggested resolution:** replace §6.2.2's three-row table with a one-line reference to
`[passphrase-bounds]` (§12.5), keeping only the *rationales* the table carries and §12.5 does not
(why `MaxLen` is rejected, why the checksum is never required). Add to §12.5: *"The device normalises
before applying the cap, not at entry."*

---

### [MINOR] 6. The unusable state still has no screen and no test, and `me --allow-weak` will now create it silently

**Where:** §12.2 l.1199–1203 and §5.4.1 l.634–638, against §3.3.3 l.359–364, §5.6 l.708 and §8.3.

**Consequence.** §12.2 states the outcome plainly — a secrets-only sealed payload under a sub-cliff
passphrase "cannot be consumed at all". `grep -n "^| F[0-9]"` returns F1–F4; there is no F5. §8.3
ends at test 22. So the operator who reaches this state unlocks successfully after ~31 s, sees their
records listed, and watches every program refuse them with no explanation — the natural inference
being a typo, retried at 31 s a time.

The fold makes one thing worse than round 3 found it: `--allow-weak` (l.708) is described only as
permitting a "sub-cliff passphrase", with no mention that for a **secrets-only** payload that
combination is not weak but **unusable by construction**. `me` will create it, print "below the
cliff", exit 0, and the payload is dead on arrival.

**Suggested resolution:** add F5 — *"admitted class is secret, container is sealed secrets-only, and
the open did not qualify → **this payload cannot be used; re-seal it with 5 or more BIP-39
words**"* — a test 23 asserting the refusal is reached and names that reason, and one clause on
`--allow-weak`: *"over secrets-only content this produces a payload no device can consume; `me`
refuses it outright rather than warning."*

---

### [MINOR] 7. Nothing still schedules the EPD§2.2/§8 and `passphrase.rs` amendments decision 8's overrule requires

**Where:** decision 8 l.56–63 against §2.2 item 6 (l.147), §9 and §10;
`SPEC_encrypted_payload_delivery.md:53`, `crates/me-cli/src/seal/passphrase.rs`.

**Consequence.** Unchanged from round 3, restated only so it is not lost from the count.
`grep -n "F-12[0-9]"` returns F-123 and F-124; there is no F-125. EPD l.53 still reads "the CLI MUST
NOT accept a user-supplied passphrase" with no pointer to the overrule, and under the Rust-primary
rule the `passphrase.rs` module-doc edit lands *first*. Finding 1 raises the stakes slightly: decision
8's justification for the overrule is the "0 bits" pricing that §6.2.1 no longer performs, so the
amendment now needs a different reason recorded with it.

**Suggested resolution:** one clause in decision 8 — *"the prohibition is container-scoped: `me seal`
and `MNEMBLOB` keep EPD§2.2 item 1 unchanged (decision 1)"* — and an F-125 in §10 owned by the Rust
phase.

---

### [MINOR] 8. §8a's new "Normalise into the buffer." is unqualified, and §6.2.2a's residue acceptance is scoped to a free-text path that instruction would eliminate

**Where:** §8a l.71–75 against §6.2.2a l.853–864 and §6.2.2's blockquote l.846–849.

**Consequence.** §8a's replacement text is correct and answers R3-I4's mechanism horn well. But its
closing imperative binds both keyboards, while §6.2.2a's premise is that *"the **free-text path**
necessarily holds a seed-equivalent passphrase in allocations nothing can scrub"* — necessary only if
that path calls `seal.NormalisePassphrase`, which §8a now says is "not the required carrier". If the
free-text path normalises into the buffer too, §6.2.2a's "necessarily" is false and its acceptance is
excusing residue that no longer exists; if it does not, §8a's imperative is wrong for half the
sentence it terminates. R3-I4's suggested resolution asked for exactly this clause and the fold added
the first half only.

**Suggested resolution:** one clause in §6.2.2a: *"This binds the free-text path only. The word path
normalises into `passphraseBytes` (§8a) and carries no string copy; a free-text implementation that
also normalises into the buffer is preferred and makes this section moot."*

---

### [NIT] 9. `check_tests`' dead `nums`, and §5.4's third statement of `[digest-shown]`

**Where:** `scripts/spec-check.py` l.181–182; §5.4 l.537–551 against §12.4.

`nums` is computed through an `and` expression that discards the first `findall` and is then never
used — round 3 named it, the fold touched the file and left it. §5.4's prose and table state
`[digest-shown]` in full a third time; unlike finding 5 they are **consistent** with §12.4, so this
is bookkeeping rather than a contradiction — but it is another §12-owned rule with a live second
site and no `SINGLE_DEF` entry.

---

## VERDICT: 0 Critical, 4 Important, 4 Minor, 1 Nit

---

### What the fold got right, recorded so a round 5 does not re-derive it

- **§12.1, §12.2, §12.3 and §12.4 are correct, mutually consistent, and need no further review.**
  §12.1 resolves the counting ambiguity round 3 raised (all-tokens-in-list, not ≥5-among-any), its
  two worked examples are both machine-verified true, and its "pure function of the normalised
  string" property is what makes §8a's no-header-field argument load-bearing rather than merely
  sound. §12.2 correctly dropped the `or user-supplied` mode clause round 3 asked be deleted.
- **All 27 `[term]` references resolve to a §12 definition.** Attack #1's third risk — a reference to
  a rule §12 does not define — is clean. No definition lost a qualifier in the move except
  §12.5's, which *gained* one (finding 5 is the site that did not receive it).
- **I looked specifically for the R0-C4 shape under the new cliff and it is not present.** For every
  input `me` can seal as above-cliff, the device computes above-cliff. No payload is made
  unconsumable by the host/device split in finding 1; that finding's blast radius is a CLI flag and
  a printed message.
- **§2.2 item 8's table is machine-exact in every claim it makes** — both `!m.Valid()` sites, the
  `refreshCands` line, and the five-call-site count (measured: exactly five non-test callers, all of
  which keep the mask). Finding 2 is about the obstacle it omits, not about anything it asserts.
- **§6.2.2's "Do not call `passphrase.ValidatePassphrase`" paragraph is correct and well-placed**;
  verified that the one function enforces the range and `MaxLen = 100` together.
- **§12.5's arithmetic is correct against a declared constant**: `bip39/wordlist.go:9`
  `const LongestWord = 8`, so `24 × 8 + 23 = 215`.
- **Test 20's rewrite is a genuine two-sided test** — an implementation with the bypass fails its
  second half, one requiring a digest fails its first. **No test in 1–22 is unfalsifiable**; the gap
  is coverage, not soundness (§12.1 has no test at all — finding 1, resolution 8).
- **The `EXPECT` pinning table and the docstring correction are the best changes in this commit.**
  The docstring now describes behaviour rather than intent, and the table immediately found a bad
  pin of its author's own. Finding 4 is a repair to `SINGLE_DEF`'s shape and to §12's over-claim
  about it, not a case against either.
- **`SINGLE_DEF` is the right structural idea and it worked once already**, catching §5.4.1's
  original `compared` and `identity` definitions in the same commit that introduced §12. Making it
  claim-shaped is a smaller change than the one that produced it.
