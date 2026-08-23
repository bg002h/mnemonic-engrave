# `mt` v0.1 spec — usability review by journey walk

Artifact: `design/SPEC_mt_v0_1.md` at `7bcd890c78f5c90dc92eed31eb392cd5b938f669`
(`git log -1` confirmed frozen at that commit; the tree did not move while I worked).
Lens: walk the two journeys step by step and ask, at every step, *what else would a
real person do here*. Not a correctness audit — citations and structure are
machine-gated and I checked neither.

## Verdict

| severity | count |
| --- | --- |
| **Critical** | 1 |
| **Important** | 9 |
| **Minor** | 4 |
| **Nit** | 1 |

**15 findings.** The single Critical (U-2) is that `mt verify` can return a pass on a
plate that has consumed its entire per-chunk error budget, and the spec never says
whether correction is applied or how much of it was used — so the operator's only
self-check can report GREEN on a plate that one further scratch makes unrecoverable.

The pattern underneath the Importants: **this spec is written for the person holding
a PSBT, and thins out sharply at every step after the string leaves stdout.** `encode`
is specified in detail; `decode`, `verify` and `inspect` — the three verbs the 2040
recoverer and the plate-proofreading operator actually run — are specified in one
paragraph each, and none of them says what it prints, what input text it tolerates,
or what it does when something is wrong. Journey B has no defined output artifact at
all: §1a says `decode` "reassembles the transaction" and no section says in what form.

Nothing here argues against an operator ruling. Two findings (U-5, U-7) are
consequences of rulings that are correct and whose *user-facing* consequence is
undisclosed, which the brief invites; U-12 is a stale-number defect adjacent to my
lens that I report because it lands on the operator as a refusal.

---

## Journey A, step by step

### A1 — The operator obtains a finalized PSBT from their wallet

**What they have.** Whatever their wallet gave them. In practice one of: a base64
PSBT string on the clipboard (Sparrow's "Copy", Core's `finalizepsbt` → `.psbt`
field), a binary `.psbt` file (Sparrow "Save"), or a raw signed transaction in hex
(Core's `finalizepsbt` → `.hex` field, and the thing Sparrow's broadcast tab shows).
Very often the PSBT is *signed but not finalized*, because finalizing is a separate
click.

**What `mt` does.** §10.10 rules the input: "**a finalized PSBT, and nothing else** —
from a file or stdin, equivalently". §8.1 refuses anything not fully finalized. §10.10
gives a whole table explaining why raw hex is refused.

**What else they might do.** All three of the above. I searched the spec for
`base64` — one hit, line 313, in the QR mode-efficiency table, about payload density.
There is **no statement anywhere of which PSBT serialisation `mt` accepts.** → **U-1**.

They also, very plausibly, hand it the hex or the unfinalized PSBT on the first try.
Both are correctly refused; neither refusal is specified to name the remedy. → **U-14**.

### A2 — They run `mt encode`

**What they have.** A file or a paste, and the sentence from §0: "`mt` engraves a
**signed Bitcoin transaction** on steel."

**What `mt` does.** Validates per §8, writes `mt1` strings to stdout, writes
everything human to stderr (§3b's stdout/stderr argument, which is the best-argued
thing in the document and survives this walk intact).

**What else.** The overwhelmingly likely invocation is the one §3b itself names —
"the ordinary invocation pipes it to a file". So the operator now has `strings.txt`
and a terminal full of stderr. Two things follow, at A6 and A7.

Also: **how many strings are in that file, and how are they separated?** §10.10's
output row reads "the **codex32 string on stdout**" — singular. §0a: "emit an `mt1`
chunked codex32 string on stdout". The set is 5, 14, 19, 63 or 89 strings depending on
the transaction. Nothing says one per line. → part of **U-9**.

### A3 — They read what it prints

**What they have.** For a 5-input legacy-bearing transaction: the §10.10 report (every
output in full, fee, locktime, plate count, engraving size, per-input provenance),
plus §0a's five suggested legend fields, plus the §3b bearer warning, plus possibly
§8.2b's 8-line fee warning, §8.2c's 11-line legacy-value warning, and §6a's 9-line
no-node block. Comfortably 50+ lines on one stream.

**What `mt` does.** Prints them. The spec fixes no order, no prominence, no summary.

**What else.** They scroll. §8.2c already argues warning discipline matters ("a warning
that cries wolf on the normal path has negative value") but nothing carries that
principle to the composite output. → **U-13**.

More seriously: two things they need at this moment are **not in the report at all**.

- Nothing states that the artifact has **zero redundancy** — that all N strings are
  required and losing one loses the transaction. §1.8 rules redundancy to zero and
  says "the operator is free to engrave duplicate copies", but that is a design note
  in §1, not a line the operator ever sees, and the decision to cut duplicates has to
  be made *now*, before the first plate. → **U-6**.
- Nothing names the **input outpoints**. The operator is about to create an artifact
  that any ordinary spend of any of those UTXOs silently voids (§7's "Silent
  invalidation" row says exactly this), and the report never tells them which coins to
  leave alone. §6a's node call has already resolved every one of them. → **U-7**.

And the report itself cannot contain what §1 promises — see **U-10**, which surfaces
properly at B6 but bites here first: §1 rules that "`encode` … invokes `inspect` on
what it just produced", and what it just produced is a *raw transaction*, which
carries no input amounts, so the fee row and the value-provenance row of §10.10's
table are not derivable from it.

### A4 — They decide whether to proceed

**What they have.** The above, minus the missing pieces.

**What `mt` does.** Nothing — it has already exited.

**What else.** They may run it again, tomorrow, or on another machine. §3b's chunk
sizing (`chunks_needed` against 320 bits, then `ceil(len/count)`) and §10.13(c)'s
content id (top 20 bits of the extracted txid) are pure functions of the PSBT, and
§8.4 explicitly guarantees the one derived *engraved* value — the `~SEASON year`
estimate — is machine-independent. So **yes, `mt encode` is deterministic**, and the
resume journey at A8 works. The spec never states it as a property, but everything
that would break it is already ruled out. Listed under "What I would NOT change".

They may also read §0's summary table, which says `mt encode`'s size limit is **64
chunks**, and §0a, which tells them the 5-input `wsh` spend at 89 chunks "loses its
path". Under the ruled ceiling (§3, §3b, §8.7b: 4,096 chunks) both statements are
false and every measured artifact fits. → **U-12**.

### A5 — They engrave N strings by hand onto steel

**What they have.** `strings.txt`, ~90 characters per string, 5–89 of them, and §0a's
suggested legend text.

**What `mt` does.** Nothing. §3b: layout, font size, characters per plate, plate count
are all the operator's, by ruling. Correct, and I am not relitigating it.

**What else.** They lay the string out in groups, because a 90-character unbroken run
is close to untranscribable by eye — and §3b's ruling *invites* exactly that ("as many
as a user wants"). They very likely engrave in **UPPERCASE**, which is what the fork's
engraving font and every steel-backup convention use. Neither of those choices has a
stated consequence anywhere; both land on the recoverer at B4. → **U-9**.

They also engrave the suggested legend, which includes `PLATE n OF m` — a field `mt`
cannot compute, because plate count is theirs by the ruling one section earlier — and
which does **not** include the per-string `n/m` that `mt` does know and that §10.8's
own argument ("a recoverer must be able to inventory what they hold and name what is
missing without decoding anything") demands. → **U-5**.

### A6 — They stop halfway and resume next week

**What they have.** 40 strings cut, 49 to go, and `strings.txt`.

**What `mt` does.** On a re-run: revalidates from scratch. If §6a's node is now
reachable and any input reads `null`, §8.5 refuses — and a `null` cannot distinguish
"spent" from "this node is still syncing or on the wrong chain", which §6a states
plainly and §10.5 rules `mt` will not adjudicate. So an operator 40 strings into an
89-string set can be hard-stopped by a resyncing node, with a refusal message the spec
does not require to disclose the ambiguity or name the alternative (run without a
node, and take §6a's warning). → **U-11**.

**What else.** Mid-set, they want to check the strings they have already cut. See A7 —
there is no verb that will do it.

### A7 — Later, they run `mt verify` to check their work

This is the step where the walk pays.

**What they have.** Steel, and `strings.txt`.

**What `mt` does.** §1: "`mt verify` is STRUCTURAL ONLY… It checks: every string
parses, every BCH checksum holds, the set is complete (`count` chunks, indices
`0..count-1`, no duplicates), every chunk carries the same `chunk_set_id`, and the
reassembled transaction re-derives that id." Optionally `--transaction <psbt|hex>`.

**What else — three divergences, and each is a finding.**

1. **They point it at `strings.txt`.** That is the file the documented flow created,
   it is on disk, and it is what tab-completion offers. That check is vacuous: it
   re-verifies the encoder's own output and says nothing whatsoever about the steel.
   The spec never says which artifact to verify, and never says that the check only
   means something when the characters were re-read off the plate. → **U-3**.
2. **They point it at a transcription, and it says OK — after correcting four
   errors.** §3a puts the per-chunk BCH budget at `t = 4`; §3b advertises `md-codec`'s
   corrector and `CorrectionDetail` by name. Nothing in §1 says whether `verify`
   applies correction, and nothing says it reports how much was used. A chunk cut with
   4 wrong characters is one scratch from unrecoverable, and the operator is told
   "OK". → **U-2, the Critical.**
3. **They try to check one plate, after cutting it.** `verify` is specified over the
   *set*: "the set is complete (`count` chunks…)". A partial set fails. `decode` and
   `inspect` both reassemble, so they need the set too. There is no verb that
   validates a single engraved string, although the BCH checksum is per-chunk and
   self-contained and makes it trivial. → **U-4**.

### A8 — They put the plates in a drawer

Out of scope, correctly. §9 explicitly excludes watching the chain for post-engraving
invalidation, and §7 records the hazard as open. Nothing to add — see "What I would
NOT change".

---

## Journey B, step by step

Assumption per the brief: a different person, in 2040, who did not cut the plates, may
not know the constellation exists, and cannot ask anyone.

### B1 — They find plates in a drawer

**What they have.** N steel plates carrying `MT1…` (or `mt1…`) strings, possibly a
five-field legend, possibly `PLATE 1 OF 1` on each of five plates (U-5), possibly
nothing else.

**What `mt` does.** Nothing. §10.21 is open and numbered: nothing on the plate names
the format or the tool. I am not restating it.

### B2 — They work out what they are

**What else.** They search the web for `mt1`. §10.21 owns this. One observation that
is *not* a restatement: §10.21 weighs a format tag against "§5's budget — which is 136
characters of a 300-character allowance". §5's budget belongs to the deferred QR verb.
For the verb that actually ships, layout is unbudgeted by ruling (§3b), so the tag is
**free**, and the trade §10.21 poses has a different answer for v0.1 than the one it
is framed to have. → **U-15**.

### B3 — They inventory what they hold

**What they have.** N plates. Are they all here?

**What `mt` does.** `count` and `index` are in every chunk header, so the machine can
answer — after decoding.

**What else.** They want to know *before* decoding, which is precisely §10.8's own
normative requirement, ruled for engraved QR symbols and never carried across to the
string form even though `mt` knows both numbers and the operator is printing suggested
text anyway. Every string starts `mt1` and differs in a handful of header symbols, so
by eye the plates are indistinguishable and unorderable. → **U-5**.

If a plate is genuinely missing, the transaction is gone — zero redundancy, §1.8. The
recoverer learns this here, at the worst possible moment, and the *operator* was never
told at A3 that it was the deal they were taking. → **U-6**.

### B4 — They transcribe the strings

**What they have.** Steel in uppercase, possibly grouped into blocks of 4–8 characters
across several lines, possibly with hyphens the operator added for legibility.

**What `mt` does.** Unspecified. I searched for `lowercase`, `uppercase`, `whitespace`,
`mixed case`, `normalis`, `normaliz`, `grouping`: **every hit is about the QR payload
or the `sysw` record** (§3's EPD §6.4 discussion, lines 393–463). The 40 lines this
spec spends on case are all about an artifact no human types. For the artifact humans
*do* type by hand, there is no rule for case, whitespace, line breaks or grouping —
and bech32 rejects mixed case, so a plate cut in caps and a header typed in caps
against a lowercase transcription of the body is a hard failure with a misleading
diagnosis. → **U-9**.

### B5 — They run something and get a transaction

**What they have.** A text file of transcribed strings.

**What `mt` does.** §1a: "It takes `mt1` strings — from a file, from stdin, typed or
pasted, in any order — and reassembles the transaction." That is the whole
specification of `mt decode`'s behaviour.

**What else.** They need something they can paste into a broadcast form or hand to
`sendrawtransaction`. **The spec never says what `decode` emits.** §10.10's table gives
an output row for `mt qr` and for `mt encode` and none for `decode`, `verify` or
`inspect`. → **U-8**.

**And when it does not work** — one string mistyped past `t = 4`, one plate genuinely
missing, one string from a *different* transaction that was in the same drawer — the
spec specifies no diagnosis. §8's "every refusal names the number that caused it"
binds §8's refusals, not decode failures. §3a even records the misdiagnosis this
format produces at the content-id compare ("pointing the recoverer at the wrong plate
rather than at the wrong software") and files it as an implementer-divergence note
rather than as a requirement on the message. → **U-2** (corrections reporting) and
**U-8** (per-chunk status: which index failed, which are good, which are missing).

### B6 — They check what they have before broadcasting

**What they have.** A decoded transaction and no other context in the world.

**What `mt` does.** §1: "`inspect` reports what is IN the artifact: chunk count and
indices, the set id, and the decoded transaction's own facts — outputs, fee, locktime,
per-input value provenance." And §1's ownership box: "the operator and the 2040
recoverer are looking at the same output".

**What else.** They cannot be. An `mt encode` payload is the raw signed transaction,
and §6 says so twice: "a raw transaction carries outpoints only, so a string plate is
silent about both the input amounts and the source scripts". So from a decoded plate
there is **no fee**, and **no value provenance** — the three provenance categories
(chain-fetched / txid-bound / operator-asserted) are properties of a PSBT that no
longer exists. The recoverer gets a strictly smaller report than the one §1 promises,
and if the fee row is simply absent they have no way to know whether the transaction
is broadcastable in 2040 — which is §7's own pinned-fee hazard, arriving at the person
least able to act on it. → **U-10**.

### B7 — They broadcast

**What `mt` does.** Nothing; §9 rules broadcasting out, correctly.

**What else.** The node says `txn-already-known`, or `missing inputs` because the
transaction was broadcast in 2031, or the outputs were spent, or a malleated variant
confirmed (§10.20, open and numbered). All of this is disclosed in §7 and §9 as
accepted and unmitigated, honestly. Nothing to add — see "What I would NOT change".

---

## Findings

### U-1 — The input serialisation is unspecified, and the most likely paste is base64

**Severity: Important. Classification: default.**

**The moment.** Journey A1. The operator copies the PSBT out of Sparrow or out of
`bitcoin-cli finalizepsbt`'s JSON — in both cases a **base64 string** — and runs
`mt encode < psbt.txt`. An implementation that reaches for `Psbt::deserialize` gets
binary and rejects it.

§10.10 rules the input surface: "**a finalized PSBT, and nothing else** — from a file
or stdin, equivalently", and lists what is still unspecified ("the flag spellings
themselves, exit codes, and the format of the refusal messages"). The **serialisation**
is in neither list, and is not an open question in §10. I searched `base64` — one hit,
line 313, in the QR density table.

**Why the wrong outcome is worse than saying nothing.** The failure is at first
contact, on the single most common form of the input, and the diagnosis ("not a PSBT")
points at the wallet rather than at the encoding. The right behaviour is obvious and
free: accept both, detecting binary by the `psbt\xff` magic and otherwise decoding
base64, and say which was detected.

### U-2 — A plate that consumed its whole error budget verifies as OK

**Severity: Critical. Classification: default.**

**The moment.** Journey A7. The operator re-reads the characters off plate 3, types
them into `chunk3.txt`, runs `mt verify`. That chunk carries four miscut characters —
exactly `t = 4`, exactly the per-chunk budget §3a specifies. BCH corrects all four,
every checksum holds, the set id matches, the transaction re-derives. `mt` says the
set is good. The operator files the plates. In 2040 one scratch across that plate adds
a fifth error, `decode_regular_errors` returns `None`, there is no redundancy (§1.8),
and the transaction is gone.

§1's specification of `verify` — "every string parses, every BCH checksum holds" —
does not say whether correction is applied, and nothing anywhere says the amount of
correction used is reported. Both readings of the ambiguity are wrong for the
operator: without correction, a plate with one recoverable error fails verify and
they re-cut 21 minutes of steel they did not need to; with silent correction, this
Critical. The machinery to fix it is already cited in this spec — §3b names
`decode_with_correction` and **`CorrectionDetail`** by name, in the paragraph arguing
that hand engraving is fault tolerant.

**Required behaviour.** `verify` and `decode` apply correction, and report per chunk
how many symbol errors were corrected against the budget of 4; any chunk at 3 or 4
produces a loud, named warning that the plate should be re-cut, because a passing
verdict on a plate with no margin left is a false green on an artifact with no second
copy. This is the one finding here whose absence loses funds.

### U-3 — Nothing says to verify the steel rather than the file

**Severity: Important. Classification: default (one normative sentence) + documentation.**

**The moment.** Journey A7. `mt encode tx.psbt > strings.txt` is the flow §3b itself
describes ("the ordinary invocation pipes it to a file"). The operator then runs
`mt verify strings.txt`, gets a pass, and believes the engraving is checked. It is
not: that run compares the encoder's output against itself and never touches the plate.

The spec's `verify` section is written entirely about strings-as-data and never
mentions steel. Yet the *only* reason a hand-engraved format needs a BCH corrector at
all is the miscut character, and the only way to detect one is to re-read the plate.

**Required.** State that `verify`'s purpose is to check the **engraving**, and that
the characters must be re-read from the steel; and promote the round-trip invocation
from "Optionally, `--transaction <psbt|hex>`" to the named operator check —
`mt verify --transaction <the PSBT you encoded> <what you read off the plates>` is the
cryptographic identity proof §1 correctly brags about, and it is currently one
parenthetical word.

### U-4 — No verb checks a single engraved string

**Severity: Minor. Classification: default.**

**The moment.** Journey A6. The operator has cut 12 of 89 strings over two evenings and
wants to know whether string 7 is right before cutting 77 more. `verify` is specified
over the complete set ("the set is complete (`count` chunks, indices `0..count-1`, no
duplicates)"); `decode` and `inspect` both reassemble. A partial set fails all three.

The BCH checksum is per chunk and entirely self-contained, so checking one string in
isolation is not merely possible, it is the natural granularity of the format.

**Required.** `verify` validates whatever it is given — parse, checksum (with U-2's
correction report), and `chunk_set_id` agreement across the strings supplied — and
reports set incompleteness as a **separate**, clearly-labelled line naming the missing
indices, rather than as a failure. Minor rather than Important because the operator can
still check everything at the end; the cost of not fixing it is delayed discovery and,
realistically, an operator who stops checking.

### U-5 — The suggested legend names a field `mt` cannot compute, and omits the one it can

**Severity: Important. Classification: default.**

**The moment.** Journey A5 and B3. §0a rules that `mt encode` "**PRINTS the suggested
legend text on `stderr`** — the same five fields §5 specifies". One of those five is
`PLATE n OF m`, whose stated purpose is "a missing plate must be obvious, and all `m`
are required". `mt` cannot know `m`: §3b rules two paragraphs earlier that "how many
plates … are all the user's decisions". So the operator engraves either a placeholder,
or whatever the implementation guessed — and `PLATE 1 OF 1` cut onto each of five
plates is a **false completeness claim on permanent steel**, read by a recoverer who
will stop looking for the other four.

Meanwhile the field `mt` *does* know is absent. §10.8 is normative and its reasoning is
verbatim applicable: "every engraved symbol carries its own human-readable `n/m` beside
it, for the chunk it holds… A recoverer must be able to inventory what they hold and
name what is missing **without decoding anything**." §10.8 extends only the
machine-readable header to `mt encode` ("for `mt encode` that header sits inside the
BCH-protected chunk") — which is the one place a human cannot read it.

**Required.** For `mt encode`, the suggested text per string carries `n/m` over
**strings** (which `mt` knows exactly), and drops or re-expresses `PLATE n OF m`, whose
denominator is the operator's to choose. This does not touch the §3b layout ruling: it
changes what `mt` suggests, not what it controls.

### U-6 — Zero redundancy is never disclosed to the person who could act on it

**Severity: Important. Classification: warning.**

**The moment.** Journey A3, and its consequence at B3. §1.8 closes "the previous
draft's largest open question" with "Redundancy is zero… The operator is free to
engrave duplicate copies." That freedom is only exercisable **before the first plate is
cut**, and the operator never hears about it: §10.10's report table has no such row,
and §3b's stderr warning is about bearer risk, not loss.

The recoverer discovers the property at B3, holding 13 of 14 plates, when nothing can
be done.

**Required.** One line in the encode report, beside the "how many strings to cut"
row: all N are required, there is no redundancy, a lost string loses the transaction,
and duplicate copies are the only defence. This is disclosure of a ruled property, not
a re-opening of it — §3b's "hand cut plates get a warning on stderr" ruling scopes
`mt`'s interest in the steel, and the spec already prints several other stderr
warnings (§8.2b, §8.2c, §6a, §0a's legend).

### U-7 — The report never names the UTXOs the plate depends on

**Severity: Important. Classification: default.**

**The moment.** Journey A3 → A8. The operator engraves a transaction spending three
UTXOs, files the plates as an inheritance path, and six weeks later their wallet's coin
selection spends one of those UTXOs to pay for dinner. The plates are now scrap, and
nobody finds out for fourteen years.

§7's "Silent invalidation" row states the hazard exactly — "one ordinary spend of any
input voids the plate, and nothing on it says so" — and its mitigation column ends
"`mt` checks it at encode time (§6a, §8.5); after that the hazard is open". But the
encode-time check is a *pass/fail against the chain*; it never tells the operator
**which outpoints** they must now protect. I searched `outpoint`: six hits, all about
the legend, §6's provenance argument, or `no duplicate outpoints` in §8.2b. §10.10's
report table lists "the value provenance | per input", which enumerates inputs for a
different purpose and is not specified to name them.

`mt` has the outpoints in hand — §6a resolves every one of them via `gettxout`.

**Required.** The encode report lists each input outpoint and its value, under a line
stating that spending any of them voids the plate. This is the one action the operator
can take that keeps the artifact alive, and it costs three lines of output.

### U-8 — `mt decode` has no specified output, and no specified failure diagnosis

**Severity: Important. Classification: default.**

**The moment.** Journey B5. The recoverer has typed in 14 strings. `mt decode`
"reassembles the transaction" (§1a) — and then what? §10.10's CLI table has output rows
for `mt qr` and `mt encode` and **none for `decode`, `verify` or `inspect`**. The
recoverer's entire objective is a hex blob they can hand to a broadcast endpoint, and
no section of this spec names it.

The failure path is equally undefined, and matters more, because a 2040 recoverer gets
one shot at diagnosing their own transcription. With one string mistyped past `t = 4`,
they need "chunk 7 of 14 failed BCH beyond correction; 0–6 and 8–13 are good; re-read
the string labelled 7" — not a global "invalid". §3a records that this format's natural
failure surface, the content-id compare, actively misdirects ("pointing the recoverer
at the wrong plate rather than at the wrong software") and leaves it as a note.

**Required.** Name `decode`'s stdout artifact — the raw signed transaction in hex is
the form every broadcast path takes and the form the payload already is — and require
per-chunk status on failure: which indices parsed, which failed checksum, which are
missing, and which carry a foreign `chunk_set_id`.

### U-9 — The human text surface is unspecified at both ends

**Severity: Important. Classification: default.**

**The moment.** Journey A5 and B4. The operator engraves in **uppercase** (which is what
steel-backup convention and the fork's engraving font do) and breaks the ~90-character
string into legible groups (which §3b's layout ruling explicitly permits: "as many as a
user wants"). In 2040 the recoverer types what they see — caps, spaces, line breaks,
maybe hyphens — and `mt decode` rejects it, or worse rejects it as "damaged", sending
them to inspect steel that is fine.

I searched `lowercase`, `uppercase`, `whitespace`, `mixed case`, `normalis`,
`normaliz`, `separator`, `grouping`. Every substantive hit (lines 393–463, 1674–1682)
is about the QR payload or the `sysw` record — an artifact no human ever types. This
spec spends forty lines and a retracted Critical on the case of the machine path and
says nothing about the case of the hand path. bech32 forbids *mixed* case, so this is
not a free-for-all: a transcription that is caps for the HRP and lower for the body is
genuinely invalid and needs a real diagnosis.

The same gap runs the other way: `mt encode` emits N strings and the spec never says
how they are separated on stdout (§10.10 and §0a both say "string", singular).

**Required.** State the emitted layout (one string per line), and state decode's
normalisation: strip all whitespace and line breaks, accept all-upper or all-lower,
reject mixed case with a message that says *mixed case* rather than *damaged*.

### U-10 — `inspect` cannot produce its promised report from a plate

**Severity: Important. Classification: default.**

**The moment.** Journey B6, and Journey A3. §1 rules that "`inspect` OWNS the report;
`encode` CALLS it", and the box justifying it claims "**the operator and the 2040
recoverer are looking at the same output**".

They structurally cannot be. §1 says `inspect` reports "outputs, fee, locktime,
per-input value provenance", but §6 states twice that an `mt encode` payload is a raw
transaction that "carries outpoints only, so a string plate is silent about both the
input amounts and the source scripts". No input amounts means **no fee** and **no value
provenance** — the three provenance categories (chain-fetched §6a, txid-bound §8.2d,
operator-asserted §8.2c) are properties of a PSBT the recoverer never has.

Read literally the ownership rule also damages Journey A: "`encode` … invokes `inspect`
on what it just produced" — the chunk set — which would strip the fee out of the
operator's own pre-cut report, and the fee is the number §8.2b's warning thresholds are
defined against.

**Required.** Say what `inspect` accepts (a PSBT *or* an `mt1` set) and which rows each
input can populate, and have `inspect` on a decoded set state explicitly that the fee
and input values are **not in this artifact** and why. Narrow the "same output" claim
to the rows that survive. A recoverer who is silently shown no fee cannot tell whether
the transaction is broadcastable in 2040 — which is §7's own pinned-fee hazard landing
on the person least able to act on it.

### U-11 — §8.5's refusal is unoverridable and cannot tell "spent" from "still syncing"

**Severity: Important. Classification: warning (refusal message content).**

**The moment.** Journey A6. The operator is 40 strings into an 89-string set. They
resume next week; their node was upgraded on Tuesday and is re-syncing. `gettxout`
returns `null`; §8.5 refuses. The operator reads that their inputs are spent and
concludes the plates in progress are worthless.

§6a already knows: "a `null` cannot distinguish 'already spent' from 'this node is
still syncing, or is on the wrong chain'". §10.5 rules — correctly, and I am not
arguing with it — that vouching for the node's sync state is not `mt`'s job. But the
*consequence for the user* is undisclosed at the only place they meet it. §6a's no-node
block is a model of good disclosure ("These checks did NOT run… Consider re-running
with a node before cutting"); the inverse case has nothing.

**Required.** §8.5's refusal names the input, and states both readings of a `null`
(spent, or a node that does not know yet) and the alternatives available — query a
different or fully-synced node, or run with no node and accept §6a's warning. No new
behaviour, no override flag, no re-opened ruling: message content only.

### U-12 — Six live sites still state the retracted 64-chunk / 2,560 B ceiling

**Severity: Important. Classification: spec correction.**

**The moment.** The operator with a 5-input `wsh` spend (3,538 B, 89 chunks — §3b's own
table) reads §0's summary table, whose `mt encode` row says the size limit is
**64 chunks**, and §0a, which tells them their wallet "loses its path". They conclude
v0.1 cannot serve them. An implementer reading §10.12 ("The 2,560 B ceiling stands, and
§8.7b refuses past it") builds a refusal at 2,560 B and their tool rejects it — while
§8.7b itself refuses only above 4,096 chunks / 163,840 B, and §3b's table says the
artifact fits at "2% of `mt1`'s 4,096".

Live sites (as opposed to the retraction boxes at 331, 333, 369, 559, 1518, 1750, which
correctly describe the old state): **line 24** (§0's table), **line 68** (§0a's cost
claim), **line 643** (§3b: "the **64-chunk ceiling** above, which binds regardless of
how the string is engraved, and which §8.7b refuses against" — contradicting the "What
fits" subsection immediately above it), **line 1932** (§10.11), **line 1959** (§10.12),
**line 2231** (§11: "against the 64-chunk container").

**Not strictly my lens** — this is a stale-number defect four correctness rounds could
have caught — but I report it because it reaches the user as a refusal and as a false
"v0.1 cannot do this". Its most consequential form is §0a's headline: **nothing is lost
to the QR deferral**, because every artifact measured in §3b fits `mt encode` under the
ruled ceiling.

### U-13 — Warning prominence in a 50-line report is unspecified

**Severity: Minor. Classification: default.**

**The moment.** Journey A3. A 5-input transaction with one unbound legacy input and no
node produces §10.10's full report plus §0a's legend suggestion plus the bearer warning
plus §8.2b's 8 lines plus §8.2c's 11 lines plus §6a's 9 lines, all interleaved on
stderr with no specified order and no summary. The operator scrolls to the prompt and
cuts steel.

§8.2c already reasons about warning value ("a warning that cries wolf on the normal
path has negative value") and §8's closing rule already reasons about message quality.
Nothing carries either to the composite.

**Required.** Warnings are re-emitted as a final summary block after the report, with a
count, so the last thing on screen before a 21-minute-per-plate commitment is the list
of things that are wrong. One rule, no new content.

### U-14 — First-contact refusals do not name the remedy

**Severity: Minor. Classification: documentation only.**

**The moment.** Journey A1. The operator hands `mt` a *signed but not finalized* PSBT —
the default state of a PSBT in most wallets, since finalizing is a separate click — and
§8.1 refuses. Or they hand it the raw signed transaction hex, having read §0's opening
sentence ("`mt` engraves a **signed Bitcoin transaction** on steel") and §3b's ("The
payload is the raw signed transaction, NOT the PSBT"), and §10.10 refuses.

Both refusals are correct rulings. §8's rule is that "every refusal names the number
that caused it", which here would be an input index — accurate and useless. The remedy
is one clause: *finalize it first* (`finalizepsbt`, or the wallet's Finalize button),
and *a PSBT is required because the raw form silently disables §8.2b and §8.2c* — which
§10.10's own table already explains to the reader but not to the user.

Man-page material, not a spec rule beyond a sentence saying refusal text names the
remedy where one exists.

### U-15 — §10.21's cost is weighed against a budget the shipping verb does not have

**Severity: Nit. Classification: documentation only.**

**The moment.** Journey B2. §10.21 (open, numbered, and not restated here) frames the
format-tag question as "weigh a short format tag against §5's budget — which is 136
characters of a 300-character allowance". §5 is the QR legend, deferred with the verb
(§0a). For `mt encode`, layout is the operator's and unbudgeted by ruling (§3b), and
`mt` is already printing suggested text on stderr — so a line naming the format and the
tool costs **nothing at all** for the verb that ships in v0.1. The open question's
stated trade has a different answer for `mt encode` than the framing implies, and a
future reader closing §10.21 against §5's budget would close it wrongly for v0.1.

---

## What I would NOT change

Divergences I walked into and deliberately judged not worth a spec rule.

1. **`mt encode` run twice gives identical output — and the spec need not say so.**
   Chunk count derives from the 320-bit ceiling, `bytes_per_chunk` from
   `ceil(len/count)`, the `chunk_set_id` from the extracted txid, and §8.4 already
   *explicitly* guarantees the only derived value that reaches steel is machine- and
   network-independent ("Two runs of `mt`, on any two machines, with or without a node,
   produce the **same engraved year**"). The chain data §6a fetches is compared, never
   embedded. A normative determinism sentence would be cheap and I would not object to
   it, but nothing in the design is currently free to drift, so it does not earn a
   finding.

2. **Chunks engraved out of order, or two transactions' plates mixed in one drawer.**
   Fully solved and clearly stated: `index` and `count` in every header, plus the 20-bit
   `chunk_set_id`, plus the content-id re-derivation. §1a's table is exactly the right
   answer to exactly this journey. No change.

3. **The transaction was broadcast years ago, or a malleated variant confirmed.** §7's
   "Silent invalidation" row and §9's explicit exclusion of chain-watching state this
   honestly as unmitigated, and §10.20 covers the txid-malleability variant. `mt` cannot
   watch the chain from a drawer and should not pretend to. The only actionable slice of
   this is U-7 (tell the operator which UTXOs to protect *while they can still act*),
   which is why that is filed and this is not.

4. **A plate handed to someone who has never heard of `mt`.** §10.21 owns it, is open
   and numbered, and I am not restating it. My only addition is U-15's Nit about how its
   cost is framed.

5. **How many characters fit a plate, font size, plate layout.** Operator ruling, §3b
   and §10.11, and correctly out. The walk turned up no case where `mt` needed to know.

6. **`mt verify --transaction` accepts hex while `mt encode` refuses it.** Looks
   inconsistent at the CLI, and is right: §10.10's PSBT-only rule exists because raw hex
   "would silently disable two refusals", and `verify`'s comparison is a txid identity
   check that needs no input amounts. Worth one man-page sentence; not worth a rule.

7. **A resume run that refuses because an input really was spent.** Correct behaviour,
   and the ~21-minutes-per-plate framing in §6a already makes the case for stopping. Only
   the *ambiguous* null is a problem, and that is U-11.

8. **A fee that is too low, or a destination whose keys are lost by 2040.** Both are
   pinned-at-encode-time hazards, both are disclosed at length in §7 and §8.2b including
   the CPFP and out-of-band escape hatches, and neither is fixable by a tool that does
   not build transactions. The disclosure is already better than most of this document.

9. **The operator ignoring the bearer warning.** Ruled, disclosed, and recorded in §7 as
   an accepted risk with the asymmetry between the verbs stated plainly. Nothing a spec
   rule can add to a warning the operator chose to skip.

10. **`mt` not offering to broadcast, sign, or build.** §0 and §9. The walk gave me no
    step where the absence hurt a user who had been told about it, and they are told.

11. **The stdout/stderr split itself.** §3b's argument ("a warning on stdout would be
    captured by that redirection and silently swallowed") is correct, load-bearing, and
    survives every step of both journeys. Two of my findings (U-3, U-13) are consequences
    of the pipe-to-a-file flow it creates, and neither is an argument against it.
