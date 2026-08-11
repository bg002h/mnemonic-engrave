# R0 round 5 — re-review of the fifth fold on `SPEC_systemwide_payloads.md`

- **Artifact:** `design/SPEC_systemwide_payloads.md` at `3e4382c` (the fold, which also carries
  `scripts/spec-check.py` at `ea66fa7`).
- **Reviewed against:** `design/agent-reports/systemwide-payloads-spec-R0-round4.md` at `248dee1`,
  and the isolated fold diff `248dee1..3e4382c` — measured, `git diff --stat`: **3 files, 123
  insertions / 31 deletions**; the spec's own hunk is **77 changed lines**.
- **Questions answered:** (1) did the fold fix each of round 4's 9 findings; (2) did the 20-site
  mechanical sweep introduce a new defect. **This is not a fresh audit.**

## Severity criteria used

The operator re-scoped severity mid-review, verbatim: *"We don't care much about security for this
feature, only look for things that block workflow."* Every finding below is ranked on one question
— **would this stop the feature working for an operator on the happy path or a reasonable unhappy
path?** Residue, forgeable opens under weak passphrases, and entropy/threshold/strength arguments do
**not** block and are reported as Minor or folded into a larger finding. This changed the rank of
four items relative to round 4's criteria; each is marked where it did.

## Machine-checked before any judgement was formed (values pasted, never described)

- `python3 scripts/spec-check.py design/SPEC_systemwide_payloads.md` → **exit 0**. 27 citations
  resolved, 10 pinned; 3 rules defined once; 3 governed terms appear only as references outside §12;
  23 named tests, numbered 1..23 without gaps.
- **The new BARE check, mutation-tested with round 4's own five surviving wordings, verbatim, with
  the harness asserting the mutation applied before judging:**

  | round-4 mutant | inserted before `#### 6.2.1` | this gate |
  | --- | --- | --- |
  | **R4-M1** | "for every variant, sealed or not, the digest is shown" | exit 0 — **SURVIVED** |
  | **R4-M2** | "A passphrase counts as above the cliff when it has at least five BIP-39 words." | exit 1 — killed by `BARE[cliff]` |
  | **R4-M3** | "A user-supplied passphrase always counts as below the cliff, whatever it contains." | exit 1 — killed by `BARE[cliff]` |
  | **R4-M4** | "`compared` becomes true when the operator checks the digest, or when any AEAD open succeeds." | exit 1 — killed by `BARE[compared]` |
  | **R4-M6** | "The payload identity is simply the EPD digest of the public section." | exit 0 — **SURVIVED** |

  **3 of round 4's 5, not 5 of 5.** `ea66fa7`'s message says *"including all five wordings the
  reviewer used to defeat the previous version: 7/7 killed"* — that claim is false as measured. See
  finding E.
- **Six further mutants, all inserted at the same anchor, all applied (asserted):**

  | mutant | text | gate |
  | --- | --- | --- |
  | M-a | "`[compared]` is set by any successful AEAD open, whatever the passphrase." | exit 0 — **SURVIVED** |
  | M-b | "`[identity]` is the SHA-256 of the region address, which never changes." | exit 0 — **SURVIVED** |
  | M-c | "A passphrase counts as strong enough when it has at least five BIP-39 words, and weak otherwise." | exit 0 — **SURVIVED** |
  | M-d | "Five wordlist tokens or more is the threshold; anything shorter, or containing a non-wordlist token, is under it." | exit 0 — **SURVIVED** |
  | M-e | "The EPD digest is displayed for every payload, including when pub_len is zero." | exit 0 — **SURVIVED** |
  | M-f | "A passphrase is above the `[cliff]` if and only if it has five or more BIP-39 wordlist tokens; otherwise it is below." | exit 1 — killed, **by `SINGLE_DEF`'s `if and only if` pattern, not by BARE** |
- `grep -c "secrets-only"` → **12 sites**. Ten write "secrets-only **sealed**" or
  "secrets-only … (`pub_len == 0`)". **Two do not: F5 (l.380) and test 23 (l.1119)** — both new in
  this fold. See finding C.
- `grep -x` against `bip39/wordlist.txt`: `correct` **IN**, `horse` **IN**, `battery` **NOT**,
  `staple` **NOT**, `foo` **NOT**, `abandon` **IN**.
- `gui/gui.go:1069` = `func NewKeyboard(ctx *Context, alphabet string) *Keyboard` — **one parameter,
  no per-instance opt-in**. `gui/passphrase_keyboard.go:116` =
  `func newPPKeyboard(ctx *Context, newline, settings bool) *PassphraseKeyboard`. `gui/gui.go:1355`
  `(*Keyboard).rune` appends **any** non-`⌫` rune to `Fragment`; `gui/gui.go:1150`
  `updateValidBIP39Keys` then calls `bip39.ClosestWord(frag)` and `panic("invalid fragment")`s when
  it is not a wordlist prefix. See finding B.
- `git diff 248dee1..3e4382c` has **no hunk** at §8a's "Normalise into the buffer", §6.2.2a, §8c,
  §5.4's digest table, decision 8 l.61, §5.4.1 l.641/645, §6.2 l.776/779, F2 l.378, or §6.2.1's mode
  table l.799–803.
- `grep -n "abandon"` over §8.3 → **no hit**. §12.1 still has no named test.

---

## Part 1 — did the fold fix each round-4 finding?

| round-4 finding | status | reason |
| --- | --- | --- |
| **R4-I1** — §6.2.1 defines `[cliff]` by MODE; five strength-residue sites; §12.1 has no test | **PARTIAL** | The transcription blockquote (l.836) is restated in `[cliff]` terms and now agrees with §5.6 by construction — the finding's load-bearing half. Untouched: §6.2.1's table cell still asserts `[cliff]` = "**below**, always" *by mode* with a reason §12.1 falsifies; §6.2 l.776's "Below 5 words (55 bits)"; l.61, l.641, l.645, l.779, F2 l.378. Resolution 8 (two named tests for §12.1's worked examples) not done — measured, no `abandon` in §8.3. See **[MINOR] D**. |
| **R4-I2** — item 8's obstacle table is incomplete; item 9 targets the wrong keyboard type | **PARTIAL** | Item 8 gained three rows and every claim in them is machine-exact; item 9 was retargeted to `NewKeyboard` (`gui/gui.go:728`). But **§8c — the operator ruling item 9 cites — was not touched** and still reads "a per-instance opt-in on the **passphrase instance** only", citing `passphrase_keyboard.go:80`'s `PassphraseKeyboard` pattern. The spec now contradicts itself about which type carries `done`. The finding's third ask (an §8b clause forbidding `isMnemonicComplete` as the completion test, and a test-19 clause) is absent. See **[IMPORTANT] B**. |
| **R4-I3** — §3.2.1's `compared` gloss is a second definition in a NORMATIVE block | **FIXED** | l.245–249 now reads "set per `[compared]` (§12.2)" and explicitly disowns the old gloss by name. The exact replacement the finding asked for, plus the reason it was wrong. |
| **R4-I4** — the single-definition gate kills 1 of 5; §12 over-claims it | **PARTIAL** | BARE is a real mechanism and a large improvement: round 4's kill rate goes **1/5 → 3/5** on its own mutants, and the 20-site sweep is done. But two of those five still survive, `[digest-shown]` and `[passphrase-bounds]` got no entry at all, `--self-test` was not added, and **§12 l.1174–1176's "a build failure, not a review finding" was not softened** — the finding's item (f), and the half that tells a future folder to stop looking. See **[MINOR] E**. |
| **R4-M5** — §6.2.2 restates `[passphrase-bounds]`; drops "over the NORMALISED string" | **PARTIAL** | The dropped qualifier is restored at l.851, which was the substantive half. The three-row restatement remains rather than deferring to §12.5, and §12.5 gained no "the device normalises before applying the cap, not at entry" line. |
| **R4-M6** — the unusable state has no screen and no test; `--allow-weak` creates it silently | **PARTIAL** | F5 and test 23 are added, and test 23 can fail. The third ask — one clause on `--allow-weak` so `me` **refuses** the secrets-only + sub-cliff combination at creation rather than emitting a dead artefact — was not done, and it is the only half that stops the operator making the trip. Folded into **[IMPORTANT] A**. |
| **R4-M7** — nothing schedules the EPD§2.2/§8 and `passphrase.rs` amendments | **FIXED** | F-125 filed, owning phase "systemwide payloads, **before implementation**", both edits named, and the Rust-primary ordering respected. |
| **R4-M8** — §8a's unqualified "Normalise into the buffer." vs §6.2.2a's "necessarily" | **NOT FIXED** | Measured: no hunk at either site. §8a l.74–75 and §6.2.2a l.871–887 are byte-identical to `248dee1`. The one-clause resolution was not applied. Does not block workflow under the current criteria; recorded so it is not lost. |
| **R4-N9** — `check_tests`' dead `nums`; §5.4's third statement of `[digest-shown]` | **PARTIAL** | `nums` is deleted (measured: `check_tests` now runs one `findall`). §5.4's prose and table still state `[digest-shown]` in full, consistently — bookkeeping, and BARE has no entry for that term. |

**Score: 2 FIXED, 6 PARTIAL, 1 NOT FIXED.**

---

## Part 2 — new defects

### [IMPORTANT] A. The mode decision 8 fought to restore produces a payload no device can ever consume, and nothing on the host stops it — this is the trade the operator asked me to rule on

**Where:** §12.2 l.1212–1227 (the `[cliff]` qualifier on the AEAD route), §12.4 l.1240–1243,
§12.1 l.1198–1200, F5 l.380, §5.6 l.723–725 and l.728, §5.4.1 l.641–655.

**Consequence.** Take the single most natural use of the mode decision 8 exists for: the operator
runs `me sysw pack --passphrase-ask <a mnemonic>` — one secret record, nothing public. Trace it:

- `pub_len == 0`, so by `[digest-shown]` (§12.4) **no digest exists**. Route 1 of `[compared]` is
  closed.
- §12.1's own bullet: *"Every user-entered non-BIP-39 password is below the cliff."* Any password
  containing a digit, a symbol, or one non-wordlist word is below. Route 2 of `[compared]` — which
  §12.2 scopes to `[cliff]`-above opens — is closed.
- Both routes closed ⇒ `[compared]` can never be set ⇒ **no record in this payload is admissible for
  consumption, ever, on any machine.**

The host does nothing to stop it. §6.2.1's table prices user-supplied as below, so `me` demands
`--allow-weak`; the operator supplies it (that flag's entire documented meaning is "I accept weaker
protection", §5.6 l.725), `me` exits 0, and — because `pub_len == 0` — §5.6 l.728's "prints the
digest to stderr" prints nothing either, so even the one artefact the operator is told to write down
is absent. The operator discovers the payload is dead on the machine, after a ~31 s KDF, and F5's
remedy is to abandon the mode and re-pack with five or more BIP-39 words.

**Why it is real, and why it is Important under the new criteria.** It is not a mistake in the
spec — the spec is internally correct and says so plainly in three places. It is the **cost of a
security decision whose only benefit is now explicitly declassified.** §12.2's scoping buys exactly
one thing: refusing a forgeable AEAD open under a weak passphrase. The operator's new criteria list
that verbatim under DOES NOT BLOCK. What it costs is on the BLOCKS list twice over — *"a payload
that cannot be opened or consumed at all"* and *"a host/device disagreement that makes a sealed
artefact unusable."* Round 4 rated the same state MINOR because under the old criteria it was, in
§5.4.1's words, "the honest outcome". Under the new criteria the trade inverts.

**So: yes, I agree the trade no longer makes sense.** The mechanism is a security control priced in
workflow, on a feature the operator has ruled is the low-assurance branch (§11, §12.1, decision 2,
EPD§2.2 item 12). I would not have raised it under round 4's criteria and I would not soften it under
these.

**Suggested resolution — (a) is what the new criteria point at, (b) is the floor if the scoping stays:**

- **(a) Unscope the AEAD route.** §12.2's second bullet becomes *"a successful AEAD open"*; delete
  the "**A sub-cliff open does NOT set it**" paragraph and the "Consequence" paragraph beneath it;
  delete the parallel paragraphs at §5.4.1 l.641–655. **F5 and test 23 then become unreachable and
  must be deleted with them** (an unreachable NORMATIVE flag row is worse than none), and **test
  20's second half inverts** — it currently asserts a below-cliff secrets-only payload is NOT
  consumable, which becomes the defect. §3.2.1's `weak` field, F2 and `[cliff]` itself all stay
  exactly as they are: the operator still gets told the payload is weakly protected, they are simply
  no longer prevented from using it.
- **(b) If the scoping stays**, `me` must refuse the combination at creation — R4-M6's unfixed
  clause: *"over secrets-only content a not-`[cliff]`-above passphrase produces a payload no device
  can consume; `me` refuses it outright, and `--allow-weak` does not override this."* That converts a
  round trip to the machine into an instant host-side error. It does not recover the mode: with (b),
  `--passphrase-ask` over secrets-only content is simply unavailable, which should then be said in
  decision 8 rather than discovered.
- Either way this is an operator ruling and belongs in §1 with the round trip recorded, exactly as
  decision 8 records its own.

---

### [IMPORTANT] B. §8c still puts the `done` key on `PassphraseKeyboard` — the defect R4-I2 named — so the fold's correction of item 9 created a contradiction, and neither site names a mechanism that exists on the type the word path builds

**Where:** §8c l.85–94 (untouched by the fold, measured) against §2.2 item 9 l.172–176 (rewritten by
it); `gui/gui.go:728`, `:1069`, `:1355`, `:1150`; `gui/passphrase_keyboard.go:80`, `:116`.

**Consequence.** The two statements now disagree:

| site | says the `done` key lives on |
| --- | --- |
| §2.2 item 9 (new) | "the keyboard the WORD path actually builds" — `NewKeyboard` (`gui/gui.go:728`), explicitly **not** `PassphraseKeyboard` |
| §8c (unchanged) | "a per-instance opt-in on the **passphrase instance** only, following the pattern `gui/passphrase_keyboard.go:80` already documents… because **`PassphraseKeyboard`** is also `NewAddressKeyboard` and BIP-85 index entry" |

§8c is an **operator ruling** and item 9 cites it. An implementer who follows the citation lands back
on the type round 4 proved the word path never constructs. Before this fold the two agreed and were
wrong together; now they disagree, and the wrong one is the normative source.

Worse, item 9's correction names no mechanism, and the mechanism it inherits by reference does not
port. Measured:

- `NewKeyboard(ctx *Context, alphabet string) *Keyboard` (`gui/gui.go:1069`) takes **one parameter**.
  The per-instance pattern §8c cites is `newPPKeyboard(ctx, newline, settings bool)`
  (`passphrase_keyboard.go:116`) — a constructor `Keyboard` does not have.
- `Keyboard`'s keys are **runes from the alphabet string**, and `(*Keyboard).rune` (`gui/gui.go:1355`)
  appends any non-`⌫` rune straight into `Fragment`. `inputWordsFlow`'s loop then calls
  `updateValidBIP39Keys(kbd.Fragment, …)` (`gui/gui.go:1148`), which calls `bip39.ClosestWord` and
  **`panic("invalid fragment")`** when the fragment is not a wordlist prefix. So a `done` key added
  to `Keyboard` the obvious way — a rune in the alphabet — **panics the device on first press**
  unless `rune()` and `Valid()` are special-cased for it exactly as `⌫` already is
  (`gui/gui.go:1256–1261`, `:1357`).

That is not a detail an implementer can be left to find: `PassphraseKeyboard` has a function row that
is structurally separate from its letter grid, which is why the opt-in is a constructor bool there;
`Keyboard` has no function row at all. The pattern does not transfer, and §8c tells the implementer
it does.

**Why it is Important under the new criteria:** *"a missing or wrongly-targeted entry path, keyboard,
terminator or return value"* is on the BLOCKS list, and this is all three at once — wrongly targeted
in §8c, untargeted in item 9, with the one concrete construction path leading to a panic.

**Suggested resolution:** rewrite §8c's mechanism paragraph to match item 9 — *"The `done` key is a
per-instance opt-in on **`Keyboard`** (`gui/gui.go:1069`), the type `inputWordsFlow` constructs at
`gui/gui.go:728`. `NewKeyboard` takes an alphabet only, so the opt-in is a new parameter, and the key
must be excluded from `Fragment` and from `updateValidKeys`' mask the way `⌫` already is
(`gui/gui.go:1256`, `:1355`) — otherwise `updateValidBIP39Keys` reaches `bip39.ClosestWord` with a
non-prefix fragment and panics. `PassphraseKeyboard`'s `newPPKeyboard(ctx, bool, bool)` is the
precedent for a per-instance opt-in, not the site of this one; the free-text path needs no `done` key
at all."* Then add the §8b clause R4-I2 asked for — the arbitrary-N flow must not use
`isMnemonicComplete` (`gui/gui.go:2541`) as its completion test — and a clause to test 19 asserting a
5-word entry is distinguishable from an abandoned one.

---

### [IMPORTANT] C. F5's condition drops the "sealed" qualifier every other site in the spec carries, so as written it fires on a plaintext payload that works — and test 23 shares the wording, so it cannot catch it

**Where:** F5 l.380 and test 23 l.1119, against l.559, l.570, l.616, l.651, l.657, l.809, l.1087,
l.1108, l.1223 and l.1235 — measured: **12 "secrets-only" sites, 10 qualified, 2 not**, and the two
are the two this fold added.

**Consequence.** F5's stated condition is *"secrets-only, and not `[cliff]`-above, so `[compared]` can
never be set"*. §3.3 says of the flag table: *"An implementer transcribes it; nothing here is left to
be derived."* Transcribe it against a **plaintext** container carrying only secret classes — which
decision 6 exists to permit and F1 exists to flag:

- "secrets-only"? Every record is a secret class. **Matches.**
- "not `[cliff]`-above"? There is no passphrase; §6.2.1's table gives mode `none` → below.
  **Matches.**

So F5 fires and the screen says *"this payload cannot be opened for use on this machine; re-create it
with five or more BIP-39 words."* Every word of that is false for this payload. A plaintext container
has `pub_len > 0` by construction — §5.3: *"An unsealed payload has no encrypted section, so a secret
has nowhere else to live"* — so its digest exists, route 1 of `[compared]` is open, the operator
compares it, and every record is consumable. The operator is told to destroy and re-create a payload
that works, and to abandon the plaintext variant to do it.

**Test 23 cannot catch this.** It repeats the same unqualified phrase — *"A secrets-only payload that
is not `[cliff]`-above raises F5"* — and asserts no converse, so an implementation that raises F5 for
**every** not-`[cliff]`-above payload passes it. That is a false-pass path in the only test the new
flag has.

**Why it is real, not a reading:** the qualifier is not implied by context here the way it is at the
other ten sites, because F5 sits in a table whose other four rows are written as mechanically
checkable predicates over `(class, container, source)` — F1 names "container is plaintext" explicitly,
so a reader has every reason to believe F5's silence about the container is deliberate. And the
trailing "so `[compared]` can never be set" reads as a *consequence* of the two stated conditions,
not as a third condition.

**Why it is Important under the new criteria:** a working payload declared unusable is *"a state the
operator reaches with no way forward"* dressed as an explanation — worse than no message, because the
message is confident and wrong, and it points at the plaintext-secret path that decision 6 exists to
provide.

**Suggested resolution:** F5's condition → *"container is **sealed**, `pub_len == 0`, and the
passphrase is not `[cliff]`-above — so neither route in `[compared]` (§12.2) is available"*; test 23 →
*"A **sealed** secrets-only payload (`pub_len == 0`) that is not `[cliff]`-above raises F5 …**and a
plaintext payload carrying only secret classes does NOT** — it has a digest, so `[compared]` is
reachable."* If finding A is resolved by option (a), delete both instead.

---

### [MINOR] D. §6.2's headline rule is a live second statement of the threshold that disagrees with §12.1 on a real input, and it survives BARE because it never writes the governed word

**Where:** §6.2 l.776–779, against §12.1 l.1180–1182. Also §6.2.1's table cell l.802, and the
strength-residue sites at l.61, l.378, l.641, l.645 — all measured untouched.

**Consequence.** §6.2's bold rule reads *"**Below 5 words (55 bits) over secret content, `me` requires
an explicit command-line flag.**"* Feed it `correct horse battery staple foo` — five tokens, of which
`battery`, `staple` and `foo` are **not** in `bip39/wordlist.txt` (measured):

- **§12.1:** not every token is a wordlist entry ⇒ **below** ⇒ the device computes `weak`, and if the
  payload is secrets-only, `[compared]` is unreachable.
- **§6.2 l.776:** it is not "below 5 words" ⇒ **no flag required** ⇒ `me` seals it silently.

That is device-stricter-than-host — the R0-C4 shape — at the one site round 4 named and asked to be
replaced (its resolution 6). Separately, §6.2.1's table still asserts user-supplied is `[cliff]`
"**below**, always — its tokens are not wordlist entries", a reason §12.1 falsifies for any password
that happens to be five wordlist words; that one is host-stricter, so it costs a spurious
`--allow-weak` rather than an artefact.

**Why MINOR and not Important, stated so the controller can check the reasoning.** Under the operator's
criteria, threshold and strength arguments do not block. The bad outcome here needs **two** things to
go wrong: an implementer transcribing §6.2's bold sentence in preference to §6.2.1, which opens by
quoting that very sentence and saying *"That is measurable for a generated passphrase and **meaningless
for a user-supplied one** … This closes it: the gate is `[cliff]` (§12.1)"* — and finding A being
resolved by option (b) rather than (a), since under (a) a below-cliff payload is consumable and the
disagreement costs only a missing warning.

**If the operator keeps §12.2's scoping (A option b), re-rate this Important**: it then becomes the
one remaining path by which `me` silently creates a payload the device will never consume.

**Suggested resolution:** l.776 → *"**Over secret content, a passphrase that is not `[cliff]`-above
requires an explicit command-line flag.**"*; delete "protected by less than `[cliff]`" at l.779;
delete §6.2.1's `[cliff]` column and replace the table's caption with one sentence — *"`[cliff]` is
computed from the normalised string in every mode (§12.1); this table is what each mode is worth,
which is a different question"*. And add round 4's resolution 8: **two named tests for §12.1's own
worked examples** — `abandon` ×5 is `[cliff]`-above, `correct horse battery staple` is below, host and
device agree on both. Measured: §12.1 is the most-referenced rule in the document and `grep` finds no
`abandon` anywhere in §8.3.

---

### [MINOR] E. BARE is a real mechanism and a large improvement, but "no wording evades it" is measurably false, §12's over-claim was not softened, and the fold's own 7/7 claim does not hold

**Where:** `scripts/spec-check.py` l.25–28 (docstring), l.165–177 (`BARE` and its comment), against
§12 l.1174–1176.

**Consequence.** Three claims, all measured false above:

| claim | where | measured |
| --- | --- | --- |
| "including all five wordings the reviewer used to defeat the previous version: **7/7 killed**" | `ea66fa7`'s commit message | **3 of 5.** R4-M1 and R4-M6 survive |
| "BARE is the mechanism … **so no wording evades it**" | docstring l.27–28 | **eight of my eleven mutants evade BARE** (seven evade the gate entirely; M-f is caught by `SINGLE_DEF`, not BARE), including R4-M6 — *the governed word itself*, in plain prose |
| "**No wording evades it, because it never inspects the wording.**" | `BARE` comment l.172 | as above |

The asymmetry is the cause and it is defensible engineering: `cliff` is guarded by `\bcliffs?\b`,
which is genuinely structural, but `compared` and `identity` are guarded by the **backticked literal
only** (`` `compared` ``, `` `identity` ``) — necessarily, since both are ordinary English words the
spec uses legitimately all over (`"the operator compares"`, `"the identity it came from"`). The
consequence is that for two of the three governed terms, plain-prose restatement is completely
unguarded, and `[digest-shown]` and `[passphrase-bounds]` have no entry at all, which is why M-e
survives and why §5.4 and §6.2.2 still carry full second statements (R4-N9, R4-M5).

And **§12 l.1174–1176 was not softened** — round 4's item (f). It still tells every future folder
*"`scripts/spec-check.py` enforces that: a definitional phrasing found outside this section is a build
failure, not a review finding."* That is the sentence a folder reads, and it is false in exactly the
direction that stops them sweeping. This is the fourth consecutive round in which a control in this
cycle is described by its intent rather than its measured behaviour — the habit F-123 was filed
against.

**Why MINOR:** it is a process control. It cannot stop the feature working for an operator; it can only
let a future fold's defect through to a reviewer. Round 4 rated it Important under criteria that no
longer apply.

**Suggested resolution:** (a) §12 l.1174–1176 → *"`scripts/spec-check.py` forbids the bare terms
`cliff`, `` `compared` `` and `` `identity` `` outside this section. It does not read wording, so it
cannot catch a paraphrase that avoids the term — a fold still owes the sweep."* (b) same correction to
the docstring and the `BARE` comment; delete "no wording evades it" from both. (c) add `BARE` entries
for `digest-shown` and `passphrase-bounds` once §5.4 and §6.2.2 defer. (d) commit the eleven mutants
above as `--self-test`, which is the only thing that stops the 7/7 error recurring: the fold measured
its own gate with mutants it wrote, which is the same failure round 4 recorded one round earlier.

---

### [MINOR] F. Three sites where the mechanical sweep changed the meaning of the sentence it edited

**Where:** §6.1 l.766, §6.2 l.779, §2.2 item 6 l.147. I read all 20 swept sites; these are the three
that did not survive substitution. The other 17 are clean — no negation lost, no reference reading as
nonsense.

| site | before → after | what broke |
| --- | --- | --- |
| l.766 | "Entropy falls off **a cliff** between 4 and 5 words" → "Entropy falls off **a `[cliff]`** between 4 and 5 words" | The original was an English metaphor about the entropy curve. The substitution makes entropy fall off the *named normative rule*, which is precisely what §12.1 l.1190 forbids in capitals (*"IT IS A SPEED BUMP, NOT A STRENGTH MEASURE, AND NOTHING MAY DESCRIBE IT AS ONE"*) and what l.1207–1208 severs. Four lines later l.770 says "**But `[cliff]` counts words, not bits**" — §6.1 now contradicts itself in adjacent paragraphs |
| l.779 | "protected by less than **the cliff**" → "protected by less than **`[cliff]`**" | `[cliff]` is a predicate name, so "less than `[cliff]`" does not parse. The loose original at least read as English |
| l.147 | "**the cliff flag**" → "**`[cliff]` flag**" | lost its article; reads as a typo |

Also l.754 — "the *shape* — a `[cliff]` between 4 and 5 words — … **is the only property the rule in
§6.2 rests on**" — kept a claim round 4 showed is false (§6.2's rule rests on a word count) while
substituting the token around it.

**Why MINOR:** all four are explanatory prose in a section the spec itself labels "information about
what generation buys, not this gate", and every normative consumer references §12.1 directly. Under
the operator's criteria these are strength arguments and do not block. Recorded because a mechanical
sweep across 20 sites is exactly where this happens, and because l.766 puts the spec in violation of
its own all-caps rule.

**Suggested resolution:** restore the metaphor and drop the reference where the sentence is *about the
entropy curve rather than about the rule* — l.766 → *"Entropy falls off a cliff between 4 and 5 words
— the ordinary English sense, not `[cliff]` (§12.1), which is a word count. That shape is where
§12.1's threshold came from."* This is the one place the spec needs the bare word, and it should say
why; §12's charter is about *definitions*, not about the noun. l.779 → delete "protected by less than
`[cliff]`", per finding D. l.147 → "**`me` passphrase modes** and the `[cliff]` flag".

---

### [NIT] G. F5 is out of row order, and §5.6's digest promise is unqualified

`grep -n "^| F[0-9]"` returns the flag table in the order **F1, F2, F3, F5, F4** — F5 was inserted
above F4 rather than appended. And §5.6 l.728 says `me sysw pack` *"prints the digest to stderr"*
without the `pub_len > 0` qualifier `[digest-shown]` (§12.4) attaches to it everywhere else, so the
secrets-only case silently prints nothing. Both are one-line fixes; the second is folded into finding
A's story rather than counted twice.

---

## VERDICT: 0 Critical, 3 Important, 3 Minor, 1 Nit

**The blocking set is NOT empty.** Three Importants block: **A** (the `[cliff]`-scoped `[compared]`
makes the restored user-supplied mode produce a payload no device can consume, and the host does not
stop it), **B** (§8c still targets `PassphraseKeyboard`, contradicting the item 9 this fold corrected,
and neither site names a mechanism that exists on `Keyboard`), and **C** (F5's unqualified
"secrets-only" fires on a working plaintext payload, and test 23 shares the wording so it cannot catch
it). Minors D, E, F and Nit G do not block.

**A is a decision, not a repair, and the operator asked for it explicitly.** B and C are both one
paragraph each and are pure repairs — B has an exact replacement above, C is a qualifier this spec
already writes at ten of its twelve sites. If A is resolved by option (a), C disappears with F5 and
test 23, and the fold reduces to B plus the Minors.

---

### What the fold got right, recorded so a round 6 does not re-derive it

- **BARE is the right idea and it works where it is structural.** Round 4's kill rate on its own
  mutants goes 1/5 → 3/5, both `cliff` mutants die, and the 20-site sweep genuinely eliminated the
  class of residue that survived four rounds. Finding E is a correction to three *claims about* the
  gate, not a case against it. The `cliff` pattern in particular is the first control in this cycle
  that is structural rather than phrasing-shaped.
- **R4-I3 is fixed exactly as asked, including the reason it was wrong**, which is the pattern the
  rest of §12 should follow.
- **F-125 is a model follow-up entry** — owning phase, both edits named, Rust-primary ordering
  respected, and an explicit "what this must NOT become".
- **Item 8's five rows are all machine-exact** — I re-verified `gui/gui.go:727`, `:728`, `:758`,
  `:792` and `gui/unlock_kdf.go:168`, `:359`. The table is complete for arbitrary-N *entry*; finding
  B is about the `done` key's target type in §8c, not about anything item 8 asserts. **There is no
  sixth blocker in that table.**
- **Test 23 can fail**, and so can every test 1–22 (round 4 settled that; nothing in this fold
  weakened one). Test 23's gap is a missing *converse*, not unfalsifiability.
- **§12.1, §12.2, §12.3, §12.4 and §12.5 remain correct and mutually consistent**, and the gate
  reports 27 citations resolved with 10 pinned. Finding A is not a claim that §12.2 is *wrong*; it is
  a claim that its price changed when the criteria did.
- **Seventeen of the twenty swept sites are clean.** Finding F is three sentences, all in explanatory
  prose.
