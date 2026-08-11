# R0 round 6 — re-review of the sixth fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `2f83712`.
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round5.md` at `b4fb7b3`,
  and the isolated fold diff `b4fb7b3..2f83712` — measured, `git diff --stat`: **1 file, 90
  insertions / 51 deletions**.
- **Questions answered:** (1) did the fold fix each of round 5's 7 findings, re-rated under the
  workflow-only criteria; (2) did the fold — which **deleted** mechanisms (F5, the `[compared]`
  scoping, `me`'s refusal) — leave a dangling reference, an unhandled state, or a test asserting a
  rule that no longer exists. **This is not a fresh audit.**

## Severity criteria used

The operator's ruling, verbatim: *"We don't care much about security for this feature, only look for
things that block workflow."* Every finding is ranked on one question — **would this stop the feature
working for an operator on the happy path or a reasonable unhappy path?** Residue, forgeable opens,
entropy and threshold arguments are declassified and are not reported as blockers.

## Machine-checked before any judgement was formed (values pasted, never described)

- `python3 scripts/spec-check.py design/SPEC_systemwide_payloads.md` → **exit 0**. 26 citations
  resolved, 10 pinned; 3 rules defined once; 3 governed terms appear only as references outside §12;
  **23 named tests, numbered 1..23 without gaps.**
- `grep -n "F5"` over the spec → **one hit, l.1283 (§13's D2 row)**. The flag table is
  `grep -n "^| F[0-9]"` → **F1, F2, F3, F4**, in order. F5's deletion left no dangling reference and
  incidentally closed round 5's Nit G's first half.
- `grep -n "consumab"` → **l.650, l.806, l.1110–1111, l.1124, l.1283**. Only **l.1110–1111 (test
  20)** states the demoted rule as a live requirement. See finding **A**.
- `git diff b4fb7b3..2f83712 | grep -c "^[+-].*R1-C4"` → **0**. Test 20 is byte-identical to
  `b4fb7b3`; the fold did not touch it.
- `grep -n "refuse"` → 15 hits. Three assert the D3-demoted host refusal: **l.721** (§5.6's
  `--allow-weak` row: *"Refuses with a non-zero exit otherwise"*), **l.833–834** (§6.2.1's
  "the deterministic rule an implementer transcribes" blockquote), **l.1061** (test 5). See **B**.
- **`gui/gui.go` at fork `345d79c`, read directly, for §8c's screen-level `done`:**
  - `:727` = `func inputWordsFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic, selected int, title string) {` — **no return value** (item 8 row 5 holds).
  - `:728` = `kbd := NewKeyboard(ctx, wordKeys)`; `:758` = `refreshCands := func() {`.
  - `inputWordsFlow` declares `backBtn := &Clickable{Button: Button1}` and
    `okBtn := &Clickable{Button: Button3}`, laid out by `layoutNavigation` (`gui/gui.go:1963`) whose
    slot array is `ys := [3]int{leadingSize, (dims.Y-btnsz.Y)/2, dims.Y-leadingSize-btnsz.Y}`,
    indexed `int(clk.Button - Button1)`.
  - `sed -n '727,865p' gui/gui.go | grep -n "Button2"` → **no match**. The **middle nav slot is
    free** on the word-entry screen.
  - `(*Keyboard).Update` (`gui/gui.go:1264`) filters **only** `Left, Right, Up, Down, Center` and
    `RuneFilter()` — it never consumes `Button1/2/3`. A `Clickable{Button: Button2}` therefore
    cannot reach `k.rune()`, cannot reach `Fragment`, and cannot reach `bip39.ClosestWord`.
  - **§8c's mechanism is buildable exactly as written.** Verified, not assumed.
- `gui/unlock_kdf.go:168` = `if !m.Valid() {`; `:359` = `if !isMnemonicComplete(m) || !m.Valid() {`.
  **All five rows of §2.2 item 8 still hold at fork HEAD**; the fold invalidated none of them, and
  moving `done` off the keyboard created **no sixth blocker** (the slot and the input path both
  already exist).
- `gui/passphrase_flow.go:73` = `func passphraseEntryFlow(ctx *Context, th *Colors, dst []byte, n int, loadProof func()) (int, bool)` — the free-text path has an accept affordance (`okBtn`, `Button3`),
  a back affordance, a caller-owned `dst []byte`, and **returns the length**. Journey (c)'s entry
  screen is a real mechanism. It calls `passphrase.ValidatePassphrase`, which §6.2.2 already forbids
  on this path by name.
- `scripts/spec-check.py` is at `ea66fa7`, **untouched by this fold**: l.28 still reads *"so no
  wording evades it"* and l.172 still reads *"No wording evades it, because it never inspects the
  wording."*
- `grep -n "isMnemonicComplete"` over the spec → **no hit**. `grep -n "abandon"` over §8.3 → **no
  hit**.

---

## Part 1 — did the fold fix each round-5 finding?

| round-5 finding | status | reason |
| --- | --- | --- |
| **A** [IMPORTANT] — the `[cliff]`-scoped `[compared]` makes `--passphrase-ask` over secrets-only unconsumable | **PARTIAL** | The rule is fixed where it lives: §12.2's second bullet is now *"any successful AEAD open, whatever the passphrase"*, §5.4.1's parallel paragraphs are withdrawn, F5 is deleted with no dangling reference, and §13 D1 records the decision. **But round 5's resolution (a) had three parts and the fold applied two.** Test 20 — "and is NOT consumable when it is below" — was not inverted, and it is now the only live statement of the demoted rule, mutually unsatisfiable with the test 23 this fold wrote. See **[IMPORTANT] A**. |
| **B** [IMPORTANT] — §8c targets `PassphraseKeyboard`; neither site names a mechanism that exists on `Keyboard` | **FIXED** | §8c is rewritten to a **screen-level button beside the nav controls**, §2.2 item 9 agrees with it, and both give the reason the keyboard route is unbuildable. **Verified against the fork rather than accepted:** `inputWordsFlow` already uses `Button1`/`Button3` in a 3-slot nav strip, `Button2` is free, and `(*Keyboard).Update` never consumes `Button1/2/3` — so the button cannot reach `Fragment` or `bip39.ClosestWord`, exactly as §8c claims. The two secondary asks (an §8b clause forbidding `isMnemonicComplete`, a test-19 clause) are still absent — measured, no hit — but neither blocks: both obstacles are already named in item 8's table. |
| **C** [IMPORTANT] — F5's unqualified "secrets-only" fires on a working plaintext payload | **FIXED (by deletion)** | F5 is gone, `grep` finds it only in §13's D2 row, and the flag table renumbers cleanly to F1–F4. The mis-scoped condition no longer exists to be transcribed. |
| **D** [MINOR] — §6.2's headline rule restates the threshold in bits and disagrees with §12.1 | **PARTIAL** | l.772 now reads *"a passphrase that is not `[cliff]`-above"* — the bits restatement is gone, which was the substantive half. Untouched: l.774–775's "protected by less than `[cliff]`", §6.2.1's table cell asserting user-supplied is "below, **always** — its tokens are not wordlist entries" (a reason §12.1 falsifies), and the two named tests for §12.1's worked examples (no `abandon` in §8.3). Still Minor: threshold arguments do not block. |
| **E** [MINOR] — the gate is over-claimed in three places | **PARTIAL** | §12 l.1178–1184 is fixed exactly as asked — *"helps, and is not sufficient"*, with the measured **3 kills in 11 mutants** pasted in and the old "build failure, not a review finding" disowned by name. But the fold touched **one file**: `scripts/spec-check.py`'s docstring (l.28) and `BARE` comment (l.172) both still say *"no wording evades it"*, and `--self-test` was not added. The over-claim now lives only where a folder reads it *after* deciding to skip the sweep. Process control; does not block. |
| **F** [MINOR] — three swept sites where substitution changed the sentence | **PARTIAL** | l.766 is fixed — *"Entropy falls off **sharply** between 4 and 5 words"* — which was the site that put the spec in violation of §12.1's all-caps rule. Untouched: l.750–751's *"the shape — a `[cliff]` between 4 and 5 words — … the only property the rule in §6.2 rests on"* (same metaphor defect, and "rests on" is looser now that §6.2's rule is a warning), l.774–775's "less than `[cliff]`" which still does not parse, and l.155's article-less "`[cliff]` flag". Explanatory prose; does not block. |
| **G** [NIT] — F5 out of row order; §5.6's digest promise unqualified | **PARTIAL** | Row order resolved by F5's deletion (measured: F1, F2, F3, F4). §5.6 l.724 still says `me sysw pack` *"prints the digest to stderr"* with no `pub_len > 0` qualifier, so journey (a) silently gets no digest. Harmless now that D1 makes the open itself sufficient. |

**Score: 2 FIXED, 5 PARTIAL, 0 NOT FIXED.** No round-5 finding was made worse, and the two
Importants the fold set out to repair (B, C) are genuinely closed.

---

## Part 2 — new defects

### [IMPORTANT] A. The deletion left test 20 behind: it is the last live statement of the demoted rule, it is mutually unsatisfiable with the test 23 this fold wrote, and building it re-kills the exact journey the fold existed to unblock

**Where:** §8.3 test 20, l.1110–1115 (measured untouched by the fold), against §8.3 test 23
l.1121–1124 (written by it) and §12.2 l.1220–1224.

**Consequence.** The two named tests, verbatim:

| test | requires, for a secrets-only sealed payload under a not-`[cliff]`-above passphrase |
| --- | --- |
| **20** | *"is NOT consumable when it is below — per `[compared]` (§12.2)"* |
| **23** | *"opens and its records are usable **whatever the passphrase**"* |

Instantiate both on one payload — `me sysw pack --passphrase-ask <a mnemonic>`, which is journey (c)
and which §12.1's own bullet (*"Every user-entered non-BIP-39 password is below the cliff"*) puts
below the threshold. Test 20 demands the records be refused; test 23 demands they be usable. **No
implementation satisfies both**, so §8.3 can never go green, and this repo's rule is tests before
implementation.

If the implementer resolves it in test 20's direction — and test 20 is the older, longer, more
heavily annotated entry, carrying two round citations (`R1-C4, corrected R3-C1`) — then
`--passphrase-ask` over secret-only records is once again a payload no device can consume. That is
**round 5's finding A restored verbatim**, in the one place the fold did not sweep.

**Why it is real, not a reading.** Test 20's own citation now points at a rule that contradicts it:
it says *"per `[compared]` (§12.2)"*, and §12.2 l.1224 says `[compared]` is set by *"any successful
AEAD open, **whatever the passphrase**"*. §13's D1 row asserts the demotion was applied; the spec
does not carry it at this site. And §5.4.1 l.643–645 cites **R1-C4** — *"it found the sealed
variant's main case unusable and tests 9 and 15 mutually unsatisfiable, because two sections had
each answered the question differently"* — as the whole reason §12 exists. This is that defect
again, one section later, between tests rather than between sections. The build gate cannot see it:
it counts 23 tests numbered without gaps and reports `ok`.

Round 5's resolution (a) named this fix explicitly — *"**test 20's second half inverts** — it
currently asserts a below-cliff secrets-only payload is NOT consumable, which becomes the defect"* —
alongside deleting F5 and test 23. The fold deleted F5 and repurposed test 23, and stopped. That is
the fixed-what-was-named habit §2.2 item 8 records this spec being caught by three rounds running.

**Why it is Important under the workflow criteria:** *"a payload that cannot be opened or consumed"*
and *"an obstacle list incomplete enough that the implementation will not work"* are both on the
BLOCKS list. This is both — a suite that cannot be made green, whose only green-able resolution in
one direction is the dead artefact.

**Suggested resolution:** rewrite test 20 to assert what §12.2 now says, keeping the half that is
still load-bearing —

> 20. **(R1-C4, corrected R3-C1, re-scoped by §13 D1)** A secrets-only sealed payload
>     (`pub_len == 0`) is consumable **after a successful AEAD open, whatever the passphrase's
>     `[cliff]` status** — per `[compared]` (§12.2) — and F2 fires when it is not `[cliff]`-above.
>     A payload whose open FAILS is not consumable. Test 15 asserts no digest is displayed; neither
>     alone is enough.

That keeps test 20 falsifiable (an implementation that gates consumption on `[cliff]` fails it, and
so does one that admits a failed open), keeps its distinct job relative to test 15, and makes it
agree with test 23 rather than duplicating it. Alternatively delete test 20 and renumber — but then
D2's claim that test 23 was deleted has to be reconciled too (see Nit D).

---

### [MINOR] B. §13's D3 is applied at one site of four — §5.6's flag table, §6.2.1's "transcribe this" blockquote and test 5 all still say `me` REFUSES

**Where:** §6.2 l.772–773 and §13 D3 l.1284 (say **warn and proceed**) against §5.6 l.721,
§6.2.1 l.833–834, and §8.3 test 5 l.1061 (all say **refuse**). Measured: `grep -n "refuse"`, three
of fifteen hits.

**Consequence.** Verbatim, the three unswept sites:

| site | says |
| --- | --- |
| §5.6, `--allow-weak` row, NORMATIVE | *"required by §6.2.1 when secret content meets a not-`[cliff]`-above passphrase. **Refuses with a non-zero exit otherwise**"* |
| §6.2.1, blockquote introduced as *"the deterministic rule an implementer transcribes"* | *"Secret content + a passphrase that is not `[cliff]`-above ⇒ `me` **refuses** without the explicit flag."* |
| test 5 | *"`me` **refuses** not-`[cliff]`-above + secret without the flag, and permits it with."* |

So test 5 is unsatisfiable against a §6.2-conformant `me`, the same shape as finding A. **It does not
block, and that is the whole difference.** Here the majority of the spec says "refuse", the minority
says "warn", and the refusal has a documented escape hatch on the same command line: the operator
adds `--allow-weak` and the artefact is produced. Journey (c) completes either way. The cost is one
spurious flag, not an artefact.

What is wrong is the **record**: §13 exists so *"a later reader sees decisions, not drift"*, and its
D3 row states a demotion the spec carries at one site of four. A future folder who trusts §13 will
believe the refusal is gone and will not find the three places it still lives.

**Suggested resolution** — pick one and sweep it. If D3 stands: §5.6's row → *"accepted whenever
secret content meets a not-`[cliff]`-above passphrase; `me` warns and proceeds (§6.2, §13 D3). It
suppresses the warning, it does not authorise anything"* — or delete the flag outright and say so in
§13; §6.2.1's blockquote → *"Secret content + a passphrase that is not `[cliff]`-above ⇒ `me` prints
a warning naming what the choice bought, and proceeds"*; test 5 → *"`me` warns on not-`[cliff]`-above
+ secret and **still writes the payload**; it does not exit non-zero. A `me` that refuses fails this
test."* If D3 is withdrawn instead, revert §6.2 l.772–773 and strike the D3 row.

---

### [MINOR] C. Journey (b) has one step with no stated mechanism: nothing says the `text:`/`pass:` hex body is DECODED before it reaches the consuming program, and §5.3.1 itself records that records are engraved verbatim

**Where:** §5.3.1 l.484–509 (the encoding), §3.2.1 l.261 (`records []{class, body}` — `body`
undefined as encoded or decoded), §3.3.2's Engrave Text row. Not introduced by this fold; surfaced by
the end-to-end walkthrough.

**Consequence.** §5.3.1 specifies the wire form (`text:<lowercase hex of the UTF-8 bytes>`), the
classification order, and that a bad hex body is `ClassUnknown`. It never states that what Engrave
Text receives is the decoded UTF-8. The pull in the other direction is stated in the same section:
*"`mdmkFlow`/`bundleEngrave` engrave records **verbatim**"*. An implementer transcribing §3.3's
*"nothing here is left to be derived"* literally can hand `text:48656c6c6f` to Engrave Text and cut
that into steel.

**Why MINOR:** the section's own framing — *"the body is **encoded**, and the record stays
canonical"* — makes the intent unmistakable to a careful reader, and the same one-line rule covers
`pass:` for the BIP-39 Password program. It is an unstated obvious step, not a contradiction, and no
round 0–5 finding depends on it.

**Suggested resolution:** one clause in §5.3.1 after the code block — *"The **body** stored in the
session (§3.2.1) and handed to a program is the **decoded** UTF-8, never the `text:`/`pass:`
prefixed hex; the encoding is a transport form for the record set and the digest, and no program
ever sees it."*

---

### [NIT] D. Three bookkeeping residues around the demotion

- **§13's D2 row says F5 and test 23 were "deleted".** F5 was; **test 23 was not** — it exists at
  l.1121 and asserts the opposite of what it used to. The spec's own gate confirms 23 tests. D2
  should read *"F5 deleted; test 23 **inverted** to assert the demoted rule cannot creep back"*.
- **Test 23 cites `(§13)` as its authority.** §13 is a history table, explicitly non-normative
  ("so a later reader sees decisions"). The rule it tests lives in §12.2; cite that.
- **Test 23's *"whatever the passphrase"*** reads, on a literal transcription, as "even a wrong one".
  It means *whatever the passphrase's `[cliff]` status*. Say so, or an implementer writes a test that
  expects an AEAD failure to still yield usable records.
- §5.6 l.724's *"prints the digest to stderr"* is still unqualified by `[digest-shown]` — round 5's
  Nit G, unchanged, now harmless because D1 makes the open sufficient.

---

## End-to-end walkthroughs

**(a) `me sysw pack --passphrase-words 12 <mnemonic>` → device unlock → BIP-85 consumes a mnemonic.**
Host: 12 wordlist tokens ⇒ `[cliff]`-above (§12.1) ⇒ no flag question arises under either reading of
D3. `pub_len == 0`, so §12.4 shows no digest and §5.6's unqualified stderr line prints nothing
(Nit D). Device: magic `MNEMSYSW` at `0x10D00000` (§4, §4.1); sealed; §8a's keyboard picker lands on
the word keyboard; §2.2 item 8's five obstacles are addressed; §8c's `done` is the free `Button2` nav
slot — **verified to exist**; `N words — unlock?` confirms; KDF; open succeeds ⇒ `[compared]` set by
§12.2's second route. BIP-85 admits `ClassMnemonic` (§3.3.2) via `seedEntryFlow` at `bip85.go:271`
(§3.1). **Completes. Every step has a named mechanism.**

**(b) `me sysw pack --no-passphrase <free text>` → Engrave Text.** Host: content is `ClassFreeText`,
not secret, so §6.2's *"public-only content is unrestricted"* applies and no flag is needed under
either reading of D3. Plaintext ⇒ `pub_len > 0` ⇒ digest printed and displayed; operator compares ⇒
`[compared]` route 1. F1 does not fire (not a secret class); F3 does. Engrave Text is one of §3.1's
four individual wirings and admits `ClassFreeText` alone. **Completes — with one unstated step: the
hex decode, finding C.**

**(c) `me sysw pack --passphrase-ask <a mnemonic>` → device unlock → Account Xpub.** Host: secret +
below-cliff. §6.2/§13 say warn and proceed; §5.6/§6.2.1/test 5 say refuse without `--allow-weak`
(finding B) — **either way the operator gets the artefact.** `pub_len == 0`, no digest. Device:
free-text keyboard, `passphraseEntryFlow`'s shape exists and returns a length into a caller-owned
buffer; §6.2.2 already forbids `ValidatePassphrase` and §12.5 sets the 215-byte cap the existing
`MaxLen = 100` would otherwise break. Open succeeds ⇒ `[compared]` set — **this is the step round 5
found dead, and D1 revives it.** F2 fires ("weakly protected"); Account Xpub admits `ClassMnemonic`
via `derive_xpub.go:107`. **Completes as written in §12.2 — and is refused by test 20. Finding A is
exactly this journey.**

---

## VERDICT: 0 Critical, 1 Important, 2 Minor, 1 Nit

**The blocking set is NOT empty.** One Important blocks: **A** — test 20 still asserts the rule §13
D1 demoted, is mutually unsatisfiable with the test 23 this fold wrote, and resolving it in test 20's
direction restores the unconsumable `--passphrase-ask` payload the fold existed to remove. It is a
**one-entry edit**, and it is the third part of a three-part resolution round 5 spelled out; the fold
applied the other two.

Minors B, C and Nit D do not hold the gate. B is a genuine four-site contradiction whose worst
outcome is a spurious flag, not a dead artefact — recorded fully so it is not lost, but under the
operator's workflow-only criteria it does not block.

**The trend is 4C → 4C → 1C → 2C → 0C → 0C → 0C, and the blocking set has fallen 3 → 1.** Round 7
should be scoped to *"did the test-20 edit land, and did it introduce a new defect"* — nothing else
in this document needs re-deriving.

---

### What the fold got right, recorded so a round 7 does not re-derive it

- **§8c is fixed and I verified the mechanism rather than accepting it.** `inputWordsFlow` lays out
  three nav slots, uses `Button1` and `Button3`, and leaves `Button2` free; `(*Keyboard).Update`
  filters only `Left/Right/Up/Down/Center` and runes, so a nav-slot `done` structurally cannot reach
  `k.rune()`, `Fragment`, or `bip39.ClosestWord`. Every claim in §8c's new paragraph is true of the
  fork at `345d79c`. This is the first mechanism in this cycle that was specified *after* the code
  was measured rather than before.
- **The F5 deletion is clean.** One reference remains and it is the history row that explains the
  deletion. The flag table renumbers to F1–F4 in order, which also closed round 5's Nit G.
- **§2.2 item 8's five rows all still hold at fork HEAD** — I re-resolved `gui/gui.go:727`, `:728`,
  `:758`, `:792` and `gui/unlock_kdf.go:168`, `:359`. Moving `done` off the keyboard created **no
  sixth blocker**: the slot and the input path both already exist. **There is still no sixth row.**
- **§13 is the right artefact to have written.** A demotion table with a "what it cost" column and a
  "what was NOT demoted" paragraph is how a security decision should be reversed on the record.
  Findings B and D are corrections to two of its rows, not a case against the section.
- **§12's over-claim is fixed exactly as round 5 asked**, with the measured 3-in-11 number pasted in
  and the old sentence disowned by name — the same pattern R4-I3 was fixed with.
- **Tests 1–19, 21 and 22 are unchanged and were settled in rounds 4 and 5.** Nothing in this fold
  weakened one. Test 22 survives the `done` relocation untouched: it names the confirmation, not the
  keyboard.
- **§12.1, §12.3, §12.4 and §12.5 are untouched and remain mutually consistent**, and with D1 applied
  there is **no reachable state in which `[compared]` can never be set** — I traced sealed with and
  without public records, plaintext, and NFC. Finding A is a leftover assertion, not a live hole in
  the rule.
