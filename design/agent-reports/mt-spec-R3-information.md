# R3 — `mt` spec, INFORMATION lens

Artifact: `design/SPEC_mt_v0_1.md` @ `9907348` (1,585 lines, read in full).
One question: **does `mt` tell the operator what they need to know, at the moment
they need it, in a form they can act on — and does it avoid drowning that out?**

Scope honoured: no argument for reinstating any guarantee the operator ruled out.
Every finding below is about what `mt` **says**, not what it refuses. Numbered
open questions (§10.1, §10.2, §10.10, §10.13, §10.14, §10.17, §10.20, §10.21) are
not restated as findings; where the 2040 section touches §10.21 it says so.

## Verdict

**2 Critical / 11 Important / 4 Minor / 1 Nit.** (18 findings; counts machine-checked against the `### I-` headings below.)

The spec's *reasoning* about warnings is unusually good — §8.4's "facts beat a
verdict", §8.2c's arithmetic-not-advice, §3b's stdout/stderr split, and §7's
refusal to invent mitigations are all correct and worth keeping verbatim. The
failure is not judgement, it is **coverage and delivery**: `mt` computes a large
amount of decision-relevant material and the spec never requires it to be
uttered. The two Criticals are both cases where a *mitigation the spec relies
on* is asserted in prose and specified in no normative section — the same class
the spec has already been bitten by three times (the ten-field legend, the
phantom engraved reminder, the unperformed `non_witness_utxo` binding).

The single sharpest structural observation: **there is no specified success-path
output at all.** §10.10's CLI table gives `stdout` = the artifact, `stderr` =
"every warning and refusal a human must see". Outputs, fee, plate count, minutes,
chunk count and the chosen configuration are none of those three things, so under
the spec as written they have no channel. `mt` is fully specified as a tool that
is silent when everything is fine — for an artifact whose cost is ~21 minutes a
plate and whose failure mode is discovered in 2040.

---

### I-1 — Critical — §5, §7, §10.10: "`mt` prints every output in full at encode time" is load-bearing and specified nowhere

**What `mt` knows.** Every output's value and `scriptPubKey`, hence every
destination address and amount, with certainty — §8.2c: *"The output total is the
anchor and `mt` knows it with certainty — it is in the transaction."*

**What the spec says.** The phrase appears exactly twice, in prose, and both
times as a *mitigation*:

- §5: *"It is still not a full disclosure: it is one line, it is optional... `mt`
  prints every output in full at encode time; the plate carries the summary."*
- §7, Pinned-destination row: *"`mt` displays every output in full at encode
  time; the plate carries a summary."*

Searched: `grep -n "every output\|outputs in full\|per-output"` — two hits, lines
529 and 651, both the above. §8 does not mention it. §10.10's CLI table does not
mention it. There is no statement of channel, content (addresses? amounts? both?
index? change?), or format.

This is exactly the class §5 already indicts itself for — *"§7's mitigations were
written against the ten-field legend and were not re-read when it became five...
A diff falsifies text it never touches"* — and the class of the phantom engraved
reminder R2 lens 2 found. Two sections justify a design choice (the `TO` line is
optional and only a summary) by pointing at a behaviour no section defines.

**Second, sharper edge: the channel is not merely unspecified, it is dangerous.**
§3b fixes *"stdout carries the artifact, stderr carries everything the human must
see"*. An implementer printing an output table to stdout corrupts `mt string`'s
piped artifact. An implementer reading §10.10 literally — stderr carries
"warnings and refusals" — has no licence to put a non-warning there at all.

**What the user does differently once told.** They see the actual destinations and
amounts, compare them against what their wallet showed, and **abort before cutting
a plate that pays somewhere they did not intend.** This is the only moment such a
comparison is possible: after the cut, `TO <wallet id>` is a 34-character summary
that is optional and may be blank.

**Fix:** one normative clause — `mt` prints, to stderr, before any warning block,
every output as `index, address, amount`, plus the output total; and the same for
inputs as `outpoint, value, source (PSBT | chain | operator)`.

---

### I-2 — Critical — §6a, §8.5: the chain's answer to "what is this input worth" is fetched, then discarded except for its null-ness

**What `mt` knows.** §6a calls `gettxout <txid> <vout> false` for every input, and
the spec's own justification for choosing that RPC is: *"it returns `value` and
`scriptPubKey` together, so the PSBT's claimed UTXO records **can be checked
against the chain rather than trusted**"*.

**What the spec says.** The only rule built on that call is §8.5: *"`gettxout`
returns `null` for any input → refuse, when a node is reachable."* Searched:
`grep -n "scriptPubKey\|gettxout\|checked against the chain\|UTXO record"` — no
rule anywhere compares the returned `value` or `scriptPubKey` against the PSBT's.
The comparison is named as a *capability* in §6a and performed by nothing.

**Why this is Critical rather than tidy-up.** §8.2's removal deleted every
signature check. For a **segwit** input, the sighash commits to the amount
(§8.2c's own table: BIP-143 / BIP-341 = "yes"), but *nothing in `mt` verifies that
signature any more* — so a PSBT that misstates a segwit input's value now sails
through §8.2b's balance check (which uses the misstated value), produces a wrong
fee, and is caught by no refusal. `gettxout`'s `value` is, post-§8.2-removal,
**the only value check `mt` has for a segwit input** — and it is thrown away.

§8.2c's warning is not a substitute: it fires only for legacy inputs, and its
whole premise is *"NOTHING HAS VERIFIED THAT VALUE"* — which is false in exactly
the case at issue, because the node just did.

**What the user does differently once told.** Shown `input 2: PSBT claims
1.00000000 BTC, chain says 10.00000000 BTC`, they stop, because the fee they were
about to engrave is wrong by 9 BTC. Shown the confirming case — `input 2:
1.00000000 BTC (confirmed against chain)` — they know the fee figure is real and
proceed with justified confidence, which is the other half of the value.

**Fix (in lane, warning-shaped):** `mt` states, per input, the value it used and
where it came from, and states loudly when the chain disagrees. Whether
disagreement is a *refusal* is the operator's call and I do not argue it.

---

### I-3 — Important — §8.2c: the legacy warning's firing condition contradicts its own body, so it is false for almost every input that triggers it

**What the spec says.** Firing condition, normative: *"**The legacy warning fires
whenever any input is legacy**, whether the value came from the PSBT or from the
operator, because `mt` verifies neither."*

Body, quoted in full in the spec:

> `NOTHING HAS VERIFIED THAT VALUE. This input carries no non_witness_utxo, so mt`
> `could not bind it by txid (see 8.2d), and a legacy signature does not commit to`
> `the amount either.`

**The contradiction.** §8.2d: *"BIP-174 **requires** `non_witness_utxo` for legacy
inputs"*, and `mt` hashes it and matches the txid. §8.2d's own closing note says
so explicitly: *"**This materially narrows §8.2c's hazard.** A legacy input
carrying `non_witness_utxo` now has its value bound by proof-of-work-anchored
history rather than by the operator's word. What remains unbound — and what
§8.2c's warning still exists for — is an input whose value arrives with **no**
`non_witness_utxo` at all."*

So for every BIP-174-conformant legacy input — which is every legacy input a
wallet will emit — the warning fires and **asserts something the spec elsewhere
proves false**: it says `mt` could not bind the value by txid, when §8.2d just
did. And it says `NOTHING HAS VERIFIED THAT VALUE` in capitals, when two things
have (the txid binding, and `gettxout` per I-2).

**Why this is the fatigue mechanism, not a wording nit.** This is the longest,
loudest, most frightening block `mt` emits — eleven lines, capitals, a worked
9.01-BTC catastrophe — and it will fire on transactions where the value is
cryptographically bound to a confirmed block. An operator who sees it three times
and verifies out of band three times for nothing will not verify the fourth. The
fourth is the one where it is true.

**What the user does differently once told.** With the firing condition narrowed
to match the body — *legacy input with no `non_witness_utxo`, or a value supplied
by the operator* — the operator who sees it **actually goes and verifies the input
value out of band**, which is the one action the warning asks for and the only
thing standing between them and the fee absorbing the difference.

**Fix:** make the firing condition the body's condition. Keep the arithmetic and
the 9.01-BTC worked example verbatim; they are the best writing in §8.

---

### I-4 — Important — §6a, §8.5: with no node, the unspent check silently does not run, and nothing says so

**What `mt` knows.** Whether it has a node. §6a: *"If a node is available, `mt`
resolves every input itself and the operator is asked for nothing."* §8.5 fires
*"when a node is reachable"*.

**What the spec says when it is not reachable.** Searched: `grep -n "no node\|node
is reachable\|unreachable\|when a node\|node is available"` — four hits. Three are
about the reachable case. The only text covering the unreachable case is §8.4's
locktime line, `LOCKED TO BLOCK 900000    current height unknown (no node)`, which
discloses the absence of a node **as a footnote to the timelock report** and never
connects it to unspentness.

So the operator's terminal, on a no-node run, contains no statement that `mt` did
not check whether the inputs are still there. §8.5's refusal simply does not
happen, and its non-happening is indistinguishable from its passing. This is the
repo's own "empty output is not absence" failure: a negative that means the check
never ran.

**Why it matters here specifically.** §7 lists **Silent invalidation** — *"one
ordinary spend of any input voids the plate"* — as a hazard whose only mitigation
is *"`mt` checks it at encode time (§6a, §8.5)"*. On a no-node run that mitigation
is absent and §7 still claims it. §6a's own framing is the standard to hold it to:
*"before you spend ~21 minutes a plate, is this transaction still worth
engraving?"* — with no node, `mt` has not asked.

**What the user does differently once told.** Told `NO NODE — inputs NOT checked
for unspentness; if any input has been spent this plate is worthless`, they point
`mt` at a node, or check the outpoints on a block explorer, **before** committing
21 minutes a plate. Told nothing, they engrave a plate that may already be dead.

---

### I-5 — Important — §4, §8.7: `mt` computes the plate count and reports it only when it refuses

**What `mt` knows.** §4's search returns plates, symbols, QR version, ECC level,
module size and tiling — and §4 states the price: *"Plate count is the real cost —
~21 minutes per plate (F-225)"*.

**What the spec says.** Searched: `grep -n "plate count\|21 minutes\|F-225"`. The
only place a plate count is required to be uttered is §8 refusal 7: *"Over the
plate budget (`mt qr`) → refuse, **naming the exact plate count** and what would
fit."* On the success path, nothing. `mt` is specified to tell the operator how
many plates it will take **only when it has decided not to do it.**

**The stakes, from the spec's own measurements.**
`RESULTS_legend_budget_2026-08-22.txt` shows the 6-line legend flips six of seven
measured configurations onto a second plate (`v18` through `v26`: each
`+ legend 25.5mm` → `NEEDS A SECOND PLATE`), and §4's largest artifact is 5
plates ≈ **105 minutes**. The operator learns this by watching the machine.

**What the user does differently once told.** Shown `5 plates, 4 symbols, v22,
ECC L, 0.60 mm module — approximately 105 minutes of engraving`, they can:
consolidate inputs in the wallet and re-sign; switch to `mt string`; choose a
different module size (I-12); or simply not start a 105-minute job at 5 p.m.
None of those decisions is available after the first plate is cut.

---

### I-6 — Important — §8.2b: `mt` never states the fee, and the gap between "warn" and "refuse" is three orders of magnitude wide

**What `mt` knows.** The fee (inputs − outputs) and the fee rate — it must compute
both to produce §8.2b's `WARNING: fee rate is 3.2 sat/vB`, which means it also
computes the vsize.

**What the spec says.** Two thresholds and nothing between them:

- below **10 sat/vB** → warn (§8.2b);
- at or above **25,000 sat/vB** (`DEFAULT_MAX_FEE_RATE`, raised as
  `AbsurdFeeRate`) → refuse (§8.2b).

Between 10 and 25,000 sat/vB, `mt` says nothing at all — the fee is not printed on
the success path (searched: `grep -n "sat/vB\|fee rate"`; every hit is one of the
two thresholds or §7's discussion of them). For a ~111 vB single-input taproot
spend, `AbsurdFeeRate` permits a fee up to ~0.0278 BTC in silence; for §4's
5-input artifact it permits roughly **0.15 BTC** in silence.

**Why the spec should care by its own argument.** §8.2b names high fees as *"the
direction that loses money"* and then places its only guard at a library's
absurdity backstop, which exists to catch fat-fingered *satoshi/BTC* unit errors,
not fee mistakes. A misplaced decimal in a wallet's fee field — the commonest real
way to overpay — lands squarely in the silent band.

**What the user does differently once told.** Shown `fee 0.04210000 BTC
(2,140 sat/vB) over 1,967 vB` on every run, they catch a wallet fee error before
cutting. Today they discover it when the transaction confirms in 2040 and a miner
takes the difference — the exact outcome §8.2c's warning is written to prevent,
arriving by a route §8.2c does not cover.

**Fix:** state fee, rate and vsize unconditionally; keep both thresholds as they
are. This is a report, not a new refusal.

---

### I-7 — Important — §3b: `mt string` never tells the operator how much steel they are about to cut

**What `mt` knows.** The chunk count and therefore the exact character count of
the string it just emitted, and the distance to the 64-chunk ceiling.

**What the spec says.** §8.7b names the chunk count **only on refusal** (*"→
refuse, naming the chunk count and the ceiling"*). On success, §10.10's table gives
`mt string` exactly one output: *"the **codex32 string on stdout**"*. §3b's own
table contains the row `RCW tr tier 1, 5-in/2-out | 2498 | **63** | yes, barely` —
one chunk from the ceiling — and `mt` is not required to say "63 of 64" to the
person about to hand-engrave it.

**Why this is in scope, against §3b's own scope ruling.** §3b rules plate layout
out: *"How many codex32 characters fit a hand engraved plate? As many as a user
wants. It is not our concern."* But it draws the line explicitly: *"what `mt`
*emits* is this spec's concern; what a user does with steel is not"*, and it keeps
the 64-chunk ceiling on exactly that basis — *"the one part that **is** a property
of the codec rather than of anyone's steel"*. The **length of the emitted string**
is the same kind of fact. Character count is not a layout decision.

**The magnitude.** A chunk is a BCH codeword of at most **93 symbols** (§10.12,
`REGULAR_CHECKSUM_SYMBOLS` over `BCH(93,80,8)`). 63 chunks is therefore on the
order of **5,900 hand-cut characters**, before HRP and separators. That is the
dominant cost of the whole exercise and it is never named. (Exact figure needs
measuring once §10.13 fixes `mt1`'s framing — the point is the absence, not my
arithmetic.)

**What the user does differently once told.** Shown `63 chunks, ~5,900 characters,
1 chunk below the 64-chunk ceiling`, the operator switches to `mt qr`, or splits
the spend, or budgets the weeks — rather than starting a hand-engraving job whose
size they discover in the middle of it.

---

### I-8 — Important — §3b, §7: the `mt string` bearer warning states the fact and withholds the action, and the spec has already established the operator can act

**What the spec says.** §3b: *"`mt string` prints a warning at encode time that
the artifact is **bearer** — anyone holding the resulting plate can spend it — and
takes no further interest in the steel."* §7's mitigation row: *"**accepted risk,
not mitigated on the plate.** `mt` emits a string, not an engraving, so it **has
no mechanism to put a warning on hand-cut steel** (§3b)."*

**The inconsistency, inside the spec's own text.** §8.2c reaches the opposite
conclusion about the same operator, for a *lesser* hazard: *"the instruction only
lands where the operator controls the plate — **`mt string`**, whose layout is
theirs by ruling (§3b)... An `mt string` operator controls their own plate and
**may add a reminder**; `mt qr`'s operator cannot."*

So the spec credits the `mt string` operator with the ability to engrave an
out-of-band-value reminder, and simultaneously declares that the bearer hazard —
which §7 ranks first, and which §5 spends its **longest legend field** (41 of 136
characters) on for `mt qr` — cannot be mitigated because `mt` "has no mechanism".
`mt` does have a mechanism. It is the same mechanism §8.2c uses: **ask.**

`mt` cannot verify compliance, which is true and worth saying — but the difference
between "we did not ask" and "we asked and cannot check" is the whole difference
between an accepted risk and an unenforced mitigation.

**What the user does differently once told.** Given a warning that ends
`Engrave this line beside your string:  BEARER - ANYONE HOLDING THIS CAN SPEND
IT`, a meaningful fraction of operators cut those 41 characters — and the 2040
holder of a hand-cut plate learns the one fact that decides how they handle it.
Today, per §3b's own honest statement, *"That warning is seen **once**, by the
person doing the encoding. The person holding the plate in 2040 is a different
person, and the plate itself says nothing."*

---

### I-9 — Important — §5: `<amount>` on the `TO` line is undefined, and it is the only money figure on the plate

**What the spec says.** §5's field is `TO <wallet id, fp or label>  <amount>`,
34 characters. Searched: `grep -n "amount"` — 16 hits, none of which defines this
one. §10.4 refers to it only for budgeting: *"§5's budget gives `TO` 34 characters
including the amount, so a label has roughly 16."*

**Three readings, all plausible, all different.** With change present:

1. the **total** of all outputs (includes money returning to the sender);
2. the total paid **to the named wallet**;
3. the value of **one** output.

The only concrete instance anywhere is in `RESULTS_legend_budget_2026-08-22.txt`:
`TO bc1p8rrz...s6n0vcl  0.00399 BTC` — reading (3), attached to the truncated-
address form §5 **replaced** on 2026-08-23. So the measurement backing the
34-character budget encodes a semantics the field no longer has.

**Why this is Important and not a nit.** It is engraved, permanent, and it is the
number a 2040 holder will use to decide whether the plate is worth acting on.
A plate reading `TO ALICE  1.00000000 BTC` on a transaction that pays Alice
0.1 BTC and returns 0.9 BTC as change is a **permanent, specific falsehood about
money** — the precise failure §8.4 refuses to commit with `IMMEDIATELY SPENDABLE`
(*"A `stderr` warning is disposable; a legend line is forever"*).

**What the user does differently once told.** The 2040 holder reads the number as
what it is. The 2026 operator can check it against their wallet's display — which
they cannot do while its meaning is undefined.

**Fix:** define it, and pick the reading that cannot be a lie about a
counterparty: the total paid to the named destination, or the output total
labelled as such (`OUT TOTAL 1.00000000`). Either is 34 characters.

---

### I-10 — Important — §5: nothing on the plate dates it, and the dropped-fields table claims a recovery path that does not recover it

**What `mt` knows.** The date it is running.

**What the spec says.** §5's dropped-fields table:

| dropped | recoverable how |
| --- | --- |
| fee rate and date | inputs − outputs, and the PSBT carries the input amounts |

`inputs − outputs` recovers the **fee**. It recovers **nothing about the date** — a
Bitcoin transaction carries no creation timestamp. The justification for dropping
the field is a recovery path that does not exist.

For **`mt string`** the row is false on both counts, and §7 says so in terms:
*"**Neither is recoverable from an `mt string` plate's own contents**, since a raw
transaction carries no input amounts (§6)."* Two sections of the same spec, in
direct contradiction on the same fact.

**What is actually on the plate, temporally.** For a timelocked transaction, the
`~<year>` of the *unlock*. For a transaction with `NO BLOCK TIMELOCK` — which §5
says the field reads in that case — **zero temporal information**. A holder in
2040 cannot tell whether the plate was cut in 2026 or in 2039.

**What the user does differently once told.** `CUT 2026-08-23` (14 characters)
tells the 2040 holder: this fee is fourteen years stale, so plan CPFP before
broadcasting; the destination wallet is fourteen years old, so confirm it still
exists before paying it; and the source wallet has had fourteen years in which one
of these inputs may have been spent (§7's Silent-invalidation hazard). Without it,
every one of those judgements is made blind.

---

### I-11 — Important — §5, §1.8: `PLATE n OF m` does not say that all m are required, and nothing tells the operator that `mt` has no answer for a lost plate

**What the spec intends.** §5's rationale column: *"a missing plate must be
obvious, **and all `m` are required** (§3)"*. §1.8: *"Redundancy is zero. `mt`
protects against damage to a plate, not against a missing plate. The operator is
free to engrave duplicate copies."*

**What is actually engraved.** `PLATE 2 OF 5`. Twelve characters that state an
index and are silent on the semantics. The rationale is in the spec; it does not
reach the steel.

**Why the wrong assumption is the natural one.** This plate sits in a drawer beside
codex32 shares and SLIP-39 shares, formats built on the premise that **missing
shares are survivable**. A holder who has internalised k-of-n will read
`PLATE 2 OF 5` as a share label. Under `mt` it is not: lose one and the
transaction is unrecoverable, permanently, with the other four plates intact and
worthless.

**The encode-time half.** §1.8's *"the operator is free to engrave duplicate
copies"* is a ruling recorded in the spec and delivered to the operator **nowhere**.
Searched: `grep -n -i "warn|WARNING"` — no warning mentions redundancy or
duplication. An operator about to cut a 5-plate artifact is never told that `mt`
provides zero protection against losing one of them, at the only moment duplicates
are cheap to add (the machine is already set up, the payload already generated).

**What the user does differently once told.** At encode time: engraves a duplicate
set, or stores the five plates in five places rather than one envelope. In 2040:
keeps searching for the missing plate instead of concluding that four of five is
a partial recovery.

**Cost:** `ALL PLATES REQUIRED` is 19 characters and can share a line (see the
2040 section).

---

### I-12 — Important — §8.8, §4, §10.10: the module-size notice is specified to fire "at the point of choice", and no point of choice exists in the CLI

**What the spec says.** §8 item 8: *"Sizes below that are **optically
unvalidated**, and `mt` says so **at the point of choice** rather than refusing."*
§10.1: *"`mt` offers every module size it can engrave, suggests 0.60 mm, and the
operator decides."*

**What §10.10 gives them to decide with.** The CLI table's `flags` row reads, in
its entirety: *"**none for locktime** (§8.4)"*. There is no module-size flag, no
interactive selection, no enumeration surface. The warning is specified with a
delivery moment the tool does not have.

**The second half, which is the actionable one.** §4's objective already searches
`module size × QR version × ECC × tiling` and its tie-break 4 *maximises* module
size — so `mt` evaluates the alternatives and reports only the winner. The
operator choosing a module size is choosing **plate count**, i.e. ~21 minutes per
step, and the spec gives them no way to see the exchange rate. §4's own R0 note
measures how flat this surface is: *"**4 configurations tie** on (plates, ECC,
symbols) for a 162 B payload at the 0.60 mm floor, and **41 tie** once the floor
lifts."*

(Noted in passing, not filed: §4 treats module size as a **search dimension** with
a maximising tie-break while §8.8/§10.1 treat it as an **operator input**. The
spec does not say whether the operator's pick constrains the search or replaces
it. Either way the recommendation below holds.)

**What the user does differently once told.** Shown

    0.60 mm  ->  5 plates, ECC L   (suggested; two engraved strokes)
    0.45 mm  ->  4 plates, ECC L   (OPTICALLY UNVALIDATED - no test plate exists)
    0.30 mm  ->  3 plates, ECC M   (OPTICALLY UNVALIDATED - one engraved stroke)

they make a 42-minute decision with the trade in front of them, and they see what
"unvalidated" is buying. Today they are told a size is unvalidated without being
told what it saves — a caution they can only nod at, which is precisely the
failure mode this lens exists to catch.

---

### I-13 — Important — §8, §10.10: a realistic run emits up to ten stderr blocks with no ordering, no severity and no summary

**Count for one plausible invocation** — `mt qr`, RCW `wsh` tier 1, 5 inputs, two
of them legacy, no node reachable, `FROM`/`TO` not supplied, a 6 sat/vB fee:

| # | block | §  |
| --- | --- | --- |
| 1 | every output printed in full (if implemented at all — I-1) | §5, §7 |
| 2 | `FROM` blank — "loudly warned" | §5, §10.4 |
| 3 | `TO` blank — "loudly warned" | §5, §10.4 |
| 4 | fee rate 6 sat/vB + CPFP/out-of-band paragraph (5 lines) | §8.2b |
| 5 | legacy input 0 — the 11-line capitalised block | §8.2c |
| 6 | legacy input 3 — the same 11-line block again | §8.2c |
| 7 | locktime report + `current height unknown (no node)` | §8.4 |
| 8 | estimated unlock year + the reference `(height, unix_time)` pair | §8.4 |
| 9 | module size below 0.60 mm is optically unvalidated | §8.8 |
| 10 | *(missing, per I-4)* no node ⇒ unspentness not checked | — |

That is well over forty lines of stderr. **The spec states no ordering, no
severity ranking, no count, and no summary.** Searched: §10.10's "Still
unspecified" list names *"exit codes, and the format of the refusal messages"* —
warnings are not mentioned as unspecified at all, so the gap is not even tracked.

**Why ordering is the whole game here.** The blocks differ by orders of magnitude
in consequence, and nothing distinguishes them:

- **the plate will be worthless**: an input already spent (I-4's unchecked case);
- **the plate will be wrong about money**: a mis-stated input value (I-2, I-3);
- **cosmetic and recoverable at the wallet**: `FROM` is blank.

`FROM` blank is also the block that fires most often, so under a naive
implementation the loudest and most frequent message is the least dangerous one.
§8.2c's block is the longest and, per I-3, the most often spurious. A tool that
prints six warnings has printed none.

**What the user does differently once told.** With a ranked summary as the **last**
thing on the terminal — the top of a forty-line scroll is gone — the operator acts
on the one that matters:

    3 warnings before you cut:
      [1] inputs NOT checked for unspentness (no node)  <- this plate may already be dead
      [2] fee rate 6.0 sat/vB
      [3] TO field blank - the plate will not name a destination

**Fix:** state that warnings are severity-ranked, counted, and repeated as a
summary block emitted last, after the artifact is written. One paragraph in
§10.10, which already owns the CLI contract.

---

### I-14 — Minor — §5: a blank legend field should engrave a word, not a gap, and §5 contradicts itself on whether `FROM` is optional

**The contradiction, both in §5.** Line 484 (the field table): `FROM WALLET
<8 hex>` — *"**Optional — loudly warned when absent** (§10.4)"*. Line 571, sixty
lines later: *"`FROM WALLET` is **a mandatory field** sized into §4's
reservation, and nothing says what supplies it or what happens when it is
absent."* §10.4 closed it as optional; §5 still carries both readings, and an
implementer has no guidance on what the plate says when the value is missing.

**What the 2040 holder sees under each reading.** If the line is *omitted*, the
legend has five lines where the format has six, and a holder cannot distinguish
"the encoder did not know" from "this line has corroded away" or "this plate was
cut short". On a bearer artifact that ambiguity is worth removing for free.

**What the user does differently once told.** `FROM WALLET UNKNOWN` /
`TO UNSTATED` tells the 2040 holder the encoder had no answer, so they stop
looking for a damaged line and start looking for the wallet elsewhere. It also
keeps the legend's line count fixed, which §4's reservation assumes anyway.

---

### I-15 — Minor — §8.2b: the low-fee warning offers only remedies available in 2040, never the one available now

**The text, quoted from §8.2b:**

> `If it turns out too low, the holder may need CPFP -- spending one of this`
> `transaction's outputs with a high-fee child, which needs no key from the`
> `signer -- or out-of-band submission directly to a miner, which bypasses relay`
> `policy entirely.`

Both remedies belong to *"the holder"*, in the future. The reader of this message
is the operator, in the present, holding a wallet that can produce a
higher-fee-rate signed transaction in under a minute — **before** any steel is
cut. That remedy is not mentioned.

The warning also names CPFP as the escape hatch and supplies neither number CPFP
sizing requires: the parent's **absolute fee** and its **vsize** (a CPFP child
must pay for both itself and the parent). Per I-6 neither is printed. For an
`mt string` plate the fee is unrecoverable from the plate's contents **forever**
(§7: *"a raw transaction carries no input amounts"*), so encode time is the only
moment those two numbers can be captured.

**What the user does differently once told.** One added sentence — *"If you can
still re-sign in your wallet at a higher fee rate, do that instead of relying on
CPFP"* — sends the operator back to the wallet rather than onto the machine.

---

### I-16 — Minor — §8.2b: dust outputs are arithmetic `mt` already has and never mentions

Searched: `grep -n -i "dust"` — **zero hits in 1,585 lines.**

**What `mt` knows.** Every output's value and `scriptPubKey` type, hence the
standard dust threshold for each (546 sat P2PKH, 330 sat P2WPKH/P2TR at the
default dust relay fee).

**Why it belongs to this lens rather than to the removed guarantees.** A
below-dust output makes the transaction **non-standard and unrelayable** — it
engraves cleanly and fails at broadcast, which is exactly §7's accepted
*"Well-formed but INVALID"* hazard. But dust is not that hazard's cause: it needs
no script engine, no signature check and no node. It is a comparison of two
integers, squarely inside §8.4's scope line — *"Fields are certain; scripts are
somebody else's job."*

**Warning, not refusal, and the spec's own reasoning says why.** Dust relay policy
is policy, so it ages exactly as §8.2b's fee floor does: *"A refusal floor would
hardcode today's relay policy into an artifact meant to be broadcast in 2040."*
The same argument that makes the fee a warning makes dust a warning.

**What the user does differently once told.** `output 1 is 210 sat, below the
330-sat dust threshold for P2TR — most nodes will not relay this transaction`
sends the operator back to the wallet, before 105 minutes of engraving produce a
plate that cannot be broadcast in any year.

---

### I-17 — Minor — §3a, §10.3, §10.8: five surviving `base45` references after §3 reversed to bech32 uppercase

**Explicitly not a re-report of the base45/EPD-§6.4 collision**, which is folded
and correct. This is the fold's incomplete propagation — the repo's known
"folds fail by incomplete propagation" class.

`grep -n "base45"` returns 10 hits. Five describe the *history* correctly (lines
160, 187, 191–192, 209, 221). Five still assert base45 as the **current** design:

| line | text |
| --- | --- |
| 262 | `mt qr:  chunk header + payload -> base45 -> QR (Reed-Solomon) -> modules` |
| 1176 | *"the QR payload is **`mt1` chunks, base45-encoded**"* (§10.3, marked **CLOSED**) |
| 1179 | *"base45 was chosen over 3%-denser raw binary for scanner compatibility"* |
| 1181 | *"§10.1's test plate should still confirm scanners read **base45** off engraved steel"* |
| 1248 | *"for `mt qr` it rides in the base45 payload"* (§10.8) |

Line 262 directly contradicts §3's diagram sixty lines earlier
(`mt qr:  mt1 chunk -> bytes -> engraved as a QR symbol`), so the spec's two
summary diagrams disagree about the payload alphabet.

**What changes once told.** Line 1181 is the operationally live one: it directs
the F-234 optical test plate — the plate that gates module size — to validate
**the wrong alphabet**. Cutting it against base45 would produce evidence about a
payload form the spec no longer emits.

---

### I-18 — Nit — §8.4: `nLockTime present but NOT ENFORCED` states the condition and not the fix

The line `nLockTime 900000 present but NOT ENFORCED (all inputs final)` is
correct and is the output of the single most valuable rule in §8.4 — §8.4 rightly
calls the alternative *"false reassurance on steel, the worst failure available
here"*.

It is also the one message in the whole spec where the operator has an exact,
cheap remedy: **set `nSequence` below `0xFFFFFFFF` on at least one input and
re-sign**, which their wallet can do in a minute and which cannot be done after
the plate is cut. The message does not say so. Every other §8.4 output is a
statement of fact where no action is available; this one is a statement of fact
where an action is.

---

## The full warning inventory

Channel is `stderr` throughout unless noted. "Actionable?" asks whether the
message, as specified, lets the operator do something different **before** steel
is cut.

### Refusals — all `stderr`, all abort

| § | refusal | fires when | names a number? | actionable? |
| --- | --- | --- | --- | --- |
| 8.1 | not fully finalized | any input lacks `PSBT_IN_FINAL_*` / non-empty `scriptSig` or witness | should name the input | **yes** — finalize in the wallet |
| 8.2b | `SendingTooMuch` | outputs > inputs | yes (both totals) | **yes** — rebuild |
| 8.2b | `AbsurdFeeRate` | rate ≥ 25,000 sat/vB | yes | **yes** — but see I-6, the band below it is silent |
| 8.2b | duplicate outpoints | any repeated input | should name the outpoint | **yes** |
| 8.2b | empty `vin` | no inputs | n/a | **yes** |
| 8.2c | input value missing | PSBT lacks a UTXO record and operator supplied none | should name the input | **yes** — supply it (and see I-3) |
| 8.2d | `non_witness_utxo` txid mismatch | hash ≠ `previous_output.txid` | **yes — "naming both txids"** | **yes** — the PSBT is corrupt or wrong |
| 8.3 | unsigned / unfinalized | duplicate of 8.1 by its own text | — | yes |
| 8.5 | `gettxout` → `null` | node reachable **and** an input is spent/absent | should name the outpoint | **yes** — the transaction is dead; **silently skipped with no node (I-4)** |
| 8.6a | non-`ALL` sighash | `NONE` / `SINGLE` / `ANYONECANPAY` in `scriptSig` or witness | should name the input | **yes** — re-sign `SIGHASH_ALL` |
| 8.6b | no signature in the satisfaction | structural scan finds none (heuristic, self-declared) | should name the input | **yes** |
| 8.7 | over the operator's plate budget | §4's result > stated maximum | **yes — "the exact plate count and what would fit"** | **yes** — the model refusal; every other one should copy it |
| 8.7b | over 64 chunks (`mt string`) | chunk count > 64 | **yes — count and ceiling, and points at `mt qr`** | **yes** — excellent; names the alternative verb |
| 8.7c | over `MAX_SECTION_LEN = 8191` (`mt qr`) | encoded artifact > 8191 B | should name both | **yes** — but no proximity warning below it (I-7's class) |
| 8.9 | secrets | an `ms1` or secret payload | n/a | yes |

Refusals are in good shape. §8's closing rule — *"Every refusal names the number
that caused it. A refusal that says only 'too large' costs the operator a round
trip"* — is the right standard, and 8.7/8.7b/8.2d already meet it.

### Warnings and reports

| § | message | fires when | actionable? | finding |
| --- | --- | --- | --- | --- |
| 3b | bearer, `mt string` only | every `mt string` run | **partly** — states a fact, asks for no action the operator can take | **I-8** |
| 5 / 10.4 | `FROM` blank — "loudly warned" | no `FROM` supplied | **yes** if it names the flag; text unspecified, "loudly" is not a spec | I-14 |
| 5 / 10.4 | `TO` blank — "loudly warned" | no `TO` supplied | same | I-14 |
| 8.2b | fee rate below 10 sat/vB | rate < 10 | **partly** — remedies are all for the 2040 holder, none for the reader | **I-15**, I-6 |
| 8.2c | legacy input present | **any** legacy input | **yes in principle** — states arithmetic, not advice; the best-written block in §8 — but its body is false whenever `non_witness_utxo` is present | **I-3** |
| 8.4 | locktime report (5 forms) | always | **yes** — two numbers side by side, correct design; `NOT ENFORCED` form omits its remedy | I-18 |
| 8.4 | estimated unlock year + reference pair | timelock present | **yes** — freshness disclosed, honest about precision | — |
| 8.8 / 10.1 | module size optically unvalidated | operator picks < 0.60 mm | **no** — no point of choice exists in §10.10, and the plate-count trade is not shown | **I-12** |
| 5 / 7 | every output printed in full | *unspecified — asserted twice as a mitigation, defined nowhere* | **cannot say** | **I-1** |
| — | inputs checked against the chain | *never specified* | — | **I-2** |
| — | no node ⇒ unspentness unchecked | *never specified* | — | **I-4** |
| — | plate count / engraving minutes | *never specified on success* | — | **I-5** |
| — | fee, fee rate, vsize on success | *never specified* | — | **I-6** |
| — | chunk count / character count (`mt string`) | *never specified on success* | — | **I-7** |
| — | dust outputs | *never specified* | — | **I-16** |
| — | redundancy is zero; consider duplicates | *never specified* | — | **I-11** |
| — | ordering / severity / summary of the above | *never specified* | — | **I-13** |

**Eight of the seventeen rows in the second table are blank in the "fires when"
column.** That is the shape of this artifact: the refusals are specified with
care, and the informational half — which is where the operator ruling placed all
of `mt`'s remaining value — is largely unwritten.

---

## What the 2040 plate should say

The legend is **136 characters, 6 lines**, against a 300-character / 20-line
text-plate budget (`RESULTS_legend_budget_2026-08-22.txt`). The binding
constraint is not that budget but the **strip beside the QR** on plate 1, which
§4 reserves at 25.5 mm / 6 lines. In the single-plate configuration the
measurements leave slack: `v13 46.2mm + legend 25.5mm = 71.7mm` against 79 mm
usable, i.e. **7.3 mm ≈ 1.7 lines spare**. For every larger configuration the
legend already forces a second plate, whose strip is nearly empty (it carries
only `PLATE n OF m`). So **one additional line is affordable**, and §10.14's
required regeneration of §4's table should price it rather than take my word.

### Are the five fields the right five?

**Four of them earn their place, and one is the weakest character-for-character.**

- `BEARER - ANYONE HOLDING THIS CAN SPEND IT` (41 ch) — **the right call, and the
  right length.** It is the only fact that changes how the plate is *handled*
  rather than how it is *used*, and it must survive being read by someone with no
  context at all. Do not shorten it. Its absence from `mt string` is I-8.
- `FROM WALLET <8 hex>` (20 ch) — **earns it, for a reason §5 does not state.**
  §5 justifies it as *"the transaction does **not** say what it spends from"* and
  as a filing aid — *"to help a human find the right plates"*. Its real 2040 value
  is against §7's **Silent invalidation** hazard: it is the only pointer a holder
  has to the wallet whose UTXOs may have been spent in the intervening years.
  Worth saying so, because it changes what the field is *for*.
- `LOCKED TO BLOCK <n> ~<year>` (29 ch) — the height earns it. **The `~<year>`
  (6 ch) is the weakest item on the plate**, on the spec's own reasoning: §8.4
  concedes *"The reference pair ages. A binary built in 2026 and run in 2031
  carries a five-year-old anchor, and the error grows with that gap."* A 2040
  reader can convert a height to a date more accurately than a 2026 binary could,
  from any block explorer or node — so this is the one engraved field that is
  **re-derivable and drifting**, the worst combination on steel. Its real audience
  is the encode-time operator, who is told the same thing on stderr where it can
  be corrected. (Dropping it saves 6 characters but no line, so it does not by
  itself buy the additions below.)
- `TO <wallet id, fp or label>  <amount>` (34 ch) — earns it, **once `<amount>` is
  defined** (I-9). Undefined, it is a permanent, specific claim about money with
  three readings.
- `PLATE n OF m` (12 ch) — earns it, **and is incomplete** (I-11).

### What the plate does not say and should — ranked

**1. That all plates are required.** The highest-value missing field.
`PLATE 2 OF 5` reads as a k-of-n share label to anyone who has handled codex32 or
SLIP-39, and under §1.8 it is not one: lose one plate and the other four are
worthless. Nineteen characters. (I-11)

**2. When it was cut.** `CUT 2026-08-23`. The plate carries no temporal
information at all when there is no timelock, and §5's dropped-fields table
justifies the omission with a recovery path — *"inputs − outputs"* — that recovers
the fee and not the date, and for `mt string` recovers neither. Fourteen
characters, and it is the field that tells the holder how much to distrust every
other field. (I-10)

**These two fit on one line together**, at 32 characters against a ~34-character
rung:

    CUT 2026-08-23 - ALL PLATES REQD

Legend goes 136 → 168 characters, 6 → 7 lines, ~4.25 mm — inside the 7.3 mm of
measured slack on the single-plate configuration, free on every multi-plate one.

**3. What it is and what to do with it.** Already open as **§10.21** (*"Nothing on
the plate names the format"*) and not re-filed. Recorded here only because this
lens ranks it: the holder's *first* problem is not decoding the QR, it is knowing
that a steel plate covered in QR symbols is a **signed Bitcoin transaction ready
to broadcast** rather than a seed backup. §10.21's *"short format tag"* framing
under-sells what is needed — `mt1` identifies the format to someone who already
knows the constellation, which is the one person who does not need telling.
§10.21's own budget note is correct that the room exists in characters; the line
accounting above is what it should be priced against.

**4. Check the inputs are still unspent before relying on this.** §7 concedes this
is *"not mitigated on the plate... they must decode the QR first"*. Decoding does
recover it, so this is an instruction rather than data — ~31 characters, and it
competes directly with 1 and 2. **Ranked below both**, because a holder who
decodes the plate at all will see the inputs; a holder who never learns the plate
is all-or-nothing may throw four fifths of it away.

### The asymmetry that dominates everything above

For **`mt string`** none of this exists. There is no legend, no `PLATE n OF m`, no
`BEARER` line, and — per §10.8's own normative standard, *"A recoverer must be
able to inventory what they hold and name what is missing **without decoding
anything**"* — no human-readable `n/m` either, because the `mt1` chunk header is
37 bits packed into codex32 symbols. §10.8 opens *"Machine-readably this holds for
both verbs"* and its normative sentence then binds *"every engraved **symbol**"*,
which is `mt qr` alone. The requirement is stated, and for one of the two verbs it
is unmet and unremarked.

Layout is the operator's by ruling and I do not contest it. But **§8.2c already
establishes the pattern for exactly this situation** — `mt` cannot engrave the
plate, so it tells the operator what to engrave. Applying that same pattern to
`mt string` costs one paragraph of stderr and converts §7's largest accepted risk
into an unenforced mitigation:

    Cut this beside your string:
      BEARER - ANYONE HOLDING THIS CAN SPEND IT
      CUT 2026-08-23 - ALL 14 CHUNKS REQUIRED
      LOCKED TO BLOCK 1383520
    ...and number each chunk 1/14 .. 14/14 as you cut it. The chunk index is
    inside the BCH-protected text and cannot be read by eye.

`mt` cannot verify any of it. It can ask, and the difference between "we did not
ask" and "we asked and cannot check" is the difference between an accepted risk
and an unenforced mitigation.
