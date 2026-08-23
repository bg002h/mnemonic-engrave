# R5 — `mt` spec, INFORMATION lens

Artifact: `design/SPEC_mt_v0_1.md` @ `322bbb5` (1,893 lines, read in full).
`git log -1` confirms `322bbb5f7ebc04dca3ef95b92c3be86f2a2bd026` — *"spec: mt1's
NUMS constant ruled -- 10.13 has no undecided input left"* — and `git status
--porcelain` is empty. **The spec did not move while this review ran.**

One question: **does `mt` tell the operator what they need to know, at the moment
they need it, in a form they can act on — and does it avoid telling them things
that drown that out?**

Scope honoured. No finding below argues for reinstating a guarantee the operator
ruled out: not script verification, not a locktime refusal, not a minimum fee,
not a legacy-input refusal. **Every finding is about what `mt` SAYS.** The
numbered open questions §10.1, §10.2, §10.14, §10.17, §10.20, §10.21, and the
record framing in §10.9 / §8.7c, are not restated as findings.

---

## Verdict

**2 Critical / 7 Important / 6 Minor / 1 Nit.** (16 findings; counts match the
`### I-` headings below.)

The R3 fold did real work. §10.10's seven-row success report closes the single
largest hole in the artifact, and §6a's no-node block is, on its own merits, the
**best-designed warning in the spec** — it names the checks that did not run,
attaches the cost of being wrong (*"A plate is ~21 minutes"*), and ends with the
one action that changes the outcome (*"Consider re-running with a node before
cutting"*). Nine of R3's eighteen findings are genuinely closed.

But the fold has a shape, and the shape is the problem. **It added seven rows of
material to the one channel that has no ordering, no severity and no summary**,
and it left the two categories of information that are *permanent* untouched:
what the plate will say, and what `mt` never checked. The result is a tool that
is now voluminous about the transaction and still silent about the artifact.

The two Criticals are both consequences of the fold rather than survivals of R3:

- **I-1.** §6a's new list enumerates the three checks a missing node skipped.
  Nothing anywhere enumerates the checks `mt` skips on *every* run — no
  signature verified, no script-hash, no taproot tweak, no k-of-n sufficiency.
  An operator who reads §6a's list, then supplies a node and sees it disappear,
  has been taught by construction that a node completes the check set. It does
  not. The spec states this hazard twice, to the reader of the spec, and never
  to the person at the terminal.
- **I-2.** `mt` composes 136 characters of permanent human-readable text and is
  nowhere required to show it to the operator before it is cut. Six of the seven
  report rows describe the transaction; none describes the artifact. §10.4
  explicitly leaves open what happens to a `TO` label too long for its ~16
  characters, so the truncation an operator will most often hit is discovered on
  steel.

---

## R3 information disposition

| R3 | severity | status at `322bbb5` | evidence |
| --- | --- | --- | --- |
| I-1 success-path output | Critical | **CLOSED** | §10.10 specifies a seven-row `stderr` report; *"every output: address in full, amount"* is now normative. Residual carved out as I-5 (change), I-15 (txid) |
| I-2 `gettxout` value discarded | Critical | **CLOSED for PSBT records** | §6a: *"`mt` compares the fetched `value` against the PSBT's UTXO record for that input and **refuses on mismatch**, naming both numbers."* Not closed for operator-supplied values → **I-4** |
| I-3 legacy warning fires on the bound case | Important | **CLOSED** | §8.2c: *"it fires when, and only when, the value is bound by nothing: no `non_witness_utxo` (§8.2d), no chain fetch (§6a)."* |
| I-4 no-node silently skips unspentness | Important | **CLOSED** | §6a's `WARNING: no bitcoind reachable. These checks did NOT run:` block, three checks enumerated, plate-time cost attached |
| I-5 plate count only on refusal | Important | **CLOSED** | §10.10 report: *"the plate count \| and, since a plate is ~21 minutes (F-225), the **engraving time**"* |
| I-6 fee never stated | Important | **CLOSED** (fee, rate) | §10.10: *"absolute and as sat/vB … printed whether or not a warning fires."* `grep -n -i vsize` → **0 hits**; the CPFP-sizing half survives in **I-13** |
| I-7 `mt string` never costed | Important | **PARTLY CLOSED** | Headroom row gives *"chunks against 64"*. Emitted character count still absent → **I-10**, downgraded to Minor because the operator holds the string on stdout |
| I-8 bearer warning withholds the action | Important | **OPEN, and I decline to press it** | §3b's ruling *"Hand cut plates get a warning on stderr. And that's it"* plus the added *"cannot verify that any warning reached the plate"* is close enough to an operator answer that re-litigating it is out of my lane |
| I-9 `<amount>` undefined | Important | **OPEN** | `grep -n "amount>"` → 1 hit, the §5 field table. No definition anywhere → **I-8** |
| I-10 no cut date | Important | **OPEN** | `grep -n "CUT 20"` → 0 hits → **I-11**, downgraded to Minor with reasoning |
| I-11 all-plates-required / duplicates | Important | **OPEN** | `grep -n "ALL PLATES"` → 0 hits; `grep -n "duplicate cop"` → 1 hit, §1.8's ruling only → **I-9** |
| I-12 module size, no point of choice | Important | **CLOSED** | §10.10's input table now carries *"module size \| §8.8 \| default 0.60 mm"*, so the notice has a moment to fire. The plate-count exchange rate is still unshown, but re-running `mt` costs nothing and cuts no plate — the operator can price it themselves. Not re-filed |
| I-13 no ordering / severity / summary | Important | **OPEN, and worse** | `grep -n -i "severity"` → 0 hits. §10.10's *"Still unspecified"* lists flag spellings, exit codes, refusal format — warnings are still not tracked. The report added 7 rows to the same channel → **I-6** |
| I-14 blank engraves a gap; FROM optional-vs-mandatory | Minor | **OPEN, and now explicit** | §10.10 rules *"warn, engrave blank"*. §5:548 *"Optional"* vs §5:635 *"`FROM WALLET` is a mandatory field"* both survive → **I-12** |
| I-15 low-fee remedies are all for 2040 | Minor | **OPEN** | `grep -n "re-sign\|resign"` → 0 hits → **I-13** |
| I-16 dust | Minor | **OPEN** | `grep -n -i "dust"` → **0 hits in 1,893 lines** → **I-14** |
| I-17 surviving `base45` references | Minor | **CLOSED** | `grep -n "base45"` → 7 hits, all historical or the rejection table. §3a's diagram now reads `-> bech32U ->`; §10.3 and §10.8 both corrected |
| I-18 `NOT ENFORCED` omits its remedy | Nit | **OPEN** | → **I-16** |

**Closed: 9 of 18** (both Criticals, six Importants, one Minor). **Open: 9**, of
which the fold's own additions created two new Criticals that R3 could not have
seen.

---

### I-1 — Critical — §6a, §8.2, §8's closing box, §10.10: `mt` verifies no signature on any run, and the only "what did not run" list it prints is the one that disappears when you supply a node

**What `mt` knows.** Exactly which checks it performed. §8's closing box already
holds the list in prose:

> | unchecked | what it would catch |
> | **script-hash** — does the revealed `witnessScript` hash to the `scriptPubKey`? | a witness script that is not the one being spent |
> | **taproot tweak** — does the internal key + merkle root tweak to the output key? | a control block that does not belong to this output |
> | **k-of-n sufficiency** — are there enough signatures for the policy? | an under-signed multisig that will never validate |

and §8.2 holds the largest one:

> `mt` no longer detects a transaction that is **well-formed but invalid** — most
> importantly one carrying a **bad signature**. Such a transaction has a witness
> present (passes §8.1), balances (passes §8.2b), and carries correct sighash
> flags (passes §8.6). It engraves cleanly and **fails at broadcast**, which for
> this artifact means years later, in exactly the situation it was cut for.

**What the spec says to the operator.** Searched `grep -n -i "did not run\|did
NOT\|not checked\|testmempool"`. **The only enumeration of unperformed checks in
the entire spec is §6a's no-node block**, at line 695, and it lists three items,
all of them node-dependent. §8.2's box and §8's closing box are addressed to the
reader of the spec. §10.10's seven report rows are outputs, fee, locktime, plate
count, configuration, headroom, value provenance — **none of them is "what was
not checked"**. §7 records the hazard as *"accepted, not mitigated"*; the
acceptance is `mt`'s, and the operator is never made a party to it.

**Why the fold made this Critical rather than merely absent.** §6a states the
principle correctly and applies it to exactly one cause:

> **Enumerating the skipped checks is the point.** *"No node"* alone tells the
> operator nothing they can act on; a list of what is therefore unknown tells
> them exactly what they are trading for convenience.

An operator who runs `mt` without a node sees three `UNKNOWN` rows. They connect
a node. The block vanishes. **The tool has now told them, by construction, that
the check set is complete** — while `mt` has verified no signature, hashed no
witness script, checked no taproot tweak and counted no multisig threshold on
either run. This is the repo's own "empty output is not absence" failure with a
twist: the absence is not silent, it is *contradicted* by a list that looks
exhaustive and is scoped to one cause.

The consequence is the artifact's worst outcome and the spec names it: a plate
cut, stored, and found in 2040 to be unbroadcastable. §0's framing binds here —
*"`mt`'s value to its user is what it TELLS them"*.

**Not an argument to reinstate §8.2.** I am not asking `mt` to verify anything.
The operator ruled *"We don't care if transaction is valid for initial
version"*, and that ruling stands. What does not follow from it is that the
operator must not be **told**. Note also that `mt` already opens an RPC
connection to a node for `gettxout` (§6a), and that the same node answers
`testmempoolaccept` in one call — so the action `mt` would be pointing at is one
the operator can already reach, without `mt` embedding anything.

**What the user does differently once told.** Shown, on every run, alongside
§6a's block rather than instead of it:

    mt did NOT check, on this or any run:
      - is any signature valid?          no signature was verified (8.2)
      - does the witness script hash to the scriptPubKey?      (8, closing)
      - does the taproot control block belong to this output?  (8, closing)
      - are there enough signatures for the policy?            (8, closing)

    A transaction that fails all four engraves cleanly and fails at broadcast.
    Validate it in your wallet, or run `bitcoin-cli testmempoolaccept`, before
    you cut ~21 minutes a plate.

…they run one command, or re-open the wallet that built the transaction and use
its own validation, **before** cutting. Told nothing, they cut five plates and
find out in 2040. This is the single cheapest information the spec can add:
`mt` already knows the answer with certainty, because the answer is a constant.

---

### I-2 — Critical — §5, §10.4, §10.10: `mt` composes 136 characters of permanent text and never shows it to the operator

**What `mt` knows.** The exact legend it is about to engrave. It holds every
input: the `BEARER` line (a constant), the `FROM WALLET <8 hex>` the operator
supplied, the `LOCKED TO BLOCK <n> ~<year>` it computed from `nLockTime`,
`nSequence` and `MT_REF_HEIGHT`, the `TO <wallet id, fp or label>  <amount>` it
formatted, and `PLATE n OF m` from §4's search. Six lines, fully determined
before a single module is cut.

**What the spec says.** Searched `grep -n "legend"` across §10.10 and §8: the
report's seven rows are *every output*, *the fee*, *the locktime*, *the plate
count*, *the configuration*, *the headroom*, *the value provenance*. **Six of
seven describe the transaction; the seventh describes the inputs. None describes
the engraving.** The legend is specified in §5, sized in §4, and echoed nowhere.

**The four ways the engraved text can be wrong, all of them silent.**

1. **Truncation.** §10.4 leaves this explicitly open: *"what `mt` does with a
   label too long for the field — §5's budget gives `TO` 34 characters including
   the amount, so a label has roughly 16. Refusing with the limit named fits
   §8's rule that every refusal names its number; silent truncation does not."*
   The spec identifies the hazard and files it as CLI work. Until it lands,
   `TO ALICE-COLD-STORAGE-2026` may reach steel as `TO ALICE-COLD-ST`.
2. **A wrong or mistyped stub.** `FROM WALLET <8 hex>` is operator-supplied
   (§10.10's input table). Eight hex characters have no checksum and no
   cross-check — §5 is explicit that *"nothing may branch on it"*. A transposed
   pair is undetectable by `mt` and permanent on the plate.
3. **The `~<year>` estimate.** Computed from an embedded constant. Correct by
   §8.4's design, and the operator is the only person who will ever see it
   before it is permanent.
4. **The locktime line's *form*.** §8.4 branches on `LOCK_TIME_THRESHOLD` to
   choose between `LOCKED TO BLOCK <n> ~<year>`, `LOCKED UNTIL <t>` and
   `NO BLOCK TIMELOCK`. §8.4 rightly calls a wrong choice here *"a permanent
   falsehood on steel"*. Which branch fired is visible in the report's locktime
   row as two facts — but the **engraved string** that results is not.

**Why the report does not already cover this.** The report gives the operator
the *inputs* to the legend, scattered across rows, in a different vocabulary.
Reconstructing six lines of engraved text from a fee row, a locktime row and a
`FROM` flag they typed twenty minutes ago is exactly the transcription step this
project's own rules forbid elsewhere (*"never describe code from its doc comment
… run it, then paste the value"*). The plate is the output. Print the output.

**What the user does differently once told.** Shown, immediately before the
artifact is written:

    This will be engraved on plate 1, exactly as shown:

      BEARER - ANYONE HOLDING THIS CAN SPEND IT
      FROM WALLET fa568be0
      LOCKED TO BLOCK 1383520 ~2034
      TO ALICE-COLD-ST  0.00399 BTC
      PLATE 1 OF 5

…they see `ALICE-COLD-ST` and re-run with a label that fits; or they see
`fa568be0` and recognise it is last month's wallet; or they see
`NO BLOCK TIMELOCK` on a transaction they believed was timelocked and go back to
the wallet. **None of these is recoverable after the cut**, and each is a
one-glance catch. Cost: five lines of `stderr`, from data `mt` has already
computed.

---

### I-3 — Important — §10.10, §6a, §8.2c: the value-provenance row has three categories and the ordinary case is none of them

**What `mt` knows.** For every input, where the value it used came from.

**What the spec says.** §10.10's report row, in full:

> | **the value provenance** | per input: chain-fetched (§6a), txid-bound (§8.2d), or operator-asserted (§8.2c) |

**The case those three omit.** A **segwit input carrying `witness_utxo`, run
without a node** — which is the ordinary shape of a finalized PSBT from any
modern wallet, on the offline posture §6a itself calls *"the constellation's
posture (§0)"*. That input is:

- not **chain-fetched** — there is no node;
- not **txid-bound** — §8.2d binds only inputs carrying `non_witness_utxo`, and
  searching `grep -n "witness_utxo"` returns **twelve hits, every one of them the
  string `non_witness_utxo`**. The bare `witness_utxo` field — the segwit value
  source — is never named in the spec;
- not **operator-asserted** — the operator asserted nothing; the PSBT did.

So its value is **PSBT-asserted and verified by nothing**, and the report has no
label for it. An implementer must either drop the input from the row (leaving an
incomplete list the operator will read as complete) or file it under
`operator-asserted`, which is false and points the operator at their own notes
rather than at the PSBT.

**Why it matters more than a taxonomy gap.** Post-§8.2, no signature is
verified, so for a segwit input the sighash's commitment to the amount (§8.2c's
own table: BIP-143 / BIP-341 = *"yes"*) is a property of the transaction that
`mt` never exercises. §8.2c's legacy warning does not fire — it *"fires when, and
only when"* the input is legacy. The report prints `fee 0.01000000 BTC (6.0
sat/vB)` as a bare number, and the fee rate is what §8.2b's 10 sat/vB threshold
is evaluated against — so an unverified input value silently decides whether the
low-fee warning fires at all.

**What the user does differently once told.** Shown

    input 2  1.00000000 BTC  PSBT witness_utxo -- NOT VERIFIED (no node)

they know the fee above it rests on the PSBT's own word, and they point `mt` at
a node or check the outpoint on an explorer. Shown `chain-fetched` for the same
input on the next run, they know the fee is real and proceed with justified
confidence — which is the other half of what a provenance row is for.

**Fix:** add the fourth category, name it for what it is (`PSBT-asserted,
unverified`), and let it be the one that reads as a caution rather than a
credential.

---

### I-4 — Important — §6a, §8.2c, §10.10: when the operator types an input value and a node is reachable, `mt` holds the true number and is not required to compare it or print it

**What `mt` knows.** Both numbers. §6a: *"The call is `gettxout <txid> <vout>
false` … it returns `value` and `scriptPubKey` together"*, for every input.
§8.2c: *"Where a record is absent, `mt` requires the operator to supply that
input's value."*

**What the spec says.** §6a's comparison is scoped to one operand:

> `mt` compares the fetched `value` against **the PSBT's UTXO record** for that
> input and **refuses on mismatch**, naming both numbers.

The operator-supplied value is not the PSBT's UTXO record — §8.2c requires it
precisely because *"a record is absent"*. So the one case where the value is a
human guess is the one case §6a's comparison does not cover. §10.10's input
table says only *"input values \| §8.2c, when the PSBT lacks them \| refuse"*,
with no node caveat, and §8.2c's warning explicitly stands down when there was a
chain fetch (*"it fires when, and only when, the value is bound by nothing: no
`non_witness_utxo` (§8.2d), no chain fetch (§6a)"*).

The result: with a node reachable and a UTXO record missing, `mt` fetches the
true value, demands a value from the operator anyway, uses one of the two, warns
about neither, and prints a fee.

**The magnitude is §8.2c's own worked example.** That block exists for exactly
this input — *"If that input actually holds 10 BTC, this transaction pays 9.01
BTC in fees and a miner will simply take it"* — and it is the block that stands
down when a node is present.

**Note the fold's shape.** R3's I-2 was *"the chain's answer is fetched, then
discarded except for its null-ness"*. The fold closed it for the PSBT's records
and left the operator's assertion — the weaker of the two operands — unchecked
against the stronger. This is the repo's "folds fail by incomplete propagation"
class: the fact is right and one of the two sites is left.

**What the user does differently once told.** Shown

    input 0: you supplied 1.00000000 BTC; the chain says 10.00000000 BTC

they stop, because the fee they were about to engrave is wrong by 9 BTC. Whether
that is a *refusal* or a warning is the operator's call and I do not argue it —
§6a already refuses on the analogous PSBT mismatch, so parity would suggest a
refusal, but stating both numbers is the part that is unambiguously in my lane.

---

### I-5 — Important — §10.10: change marking is conditioned on an input the spec's own table does not list

**What the spec says.** The report's first and most decision-relevant row:

> | **every output** | address in full, amount, and which are change **if a
> wallet was supplied** |

**What §10.10's own input table lists**, eight rows later: the PSBT, plate
budget, `FROM` wallet id / fingerprint, `TO` wallet id / fingerprint, `TO`
free-text label, input values, module size, node location. **There is no wallet,
descriptor, or output-derivation input.** `FROM WALLET <8 hex>` is *"a wallet id
or a seed fingerprint"* (§5) — eight hex characters, which §5 is emphatic is
*"a hint, never an authority"* and on which *"nothing may branch"*. Change
marking is a branch.

There is a plausible mechanism the spec never names: if `FROM` is a **seed
fingerprint**, it matches the master fingerprint in a PSBT output's BIP-32
derivation fields, which is the conventional way to identify change and needs no
new input at all. But the spec neither says this, nor says what happens when
`FROM` is a wallet id rather than a fingerprint, nor what happens when `FROM` is
absent — which is precisely the run on which §5 and §10.4 already fire two
"loudly warned" blocks.

**Why this is not a wording nit.** §5 and §7 both defend the `TO` legend line
being a single optional summary on the grounds that *"`mt` prints every output in
full at encode time"*. What makes that print **interpretable** is knowing which
outputs are the operator's own money coming back. An operator reading

    output 0  bc1p8rrz...  0.00399 BTC
    output 1  bc1qa4k9...  0.90000000 BTC

cannot tell whether this pays one counterparty and returns change, or pays two
counterparties. Worse, marking is *conditional*: an operator who has seen
`(change)` annotations on previous runs will read their absence as "no change
here" rather than "marking is off this run", which is a silent inversion.

**What the user does differently once told.** Given an unconditional statement of
which mode they are in — `change marking: ON (matched against seed fingerprint
fa568be0)` or `change marking: OFF (no fingerprint supplied) — every output below
is unclassified` — the operator either reads the outputs correctly, or knows to
reconcile all of them against the wallet by hand. Given the conditional as
written, they cannot tell which report they are reading.

This also gates **I-8**: any non-vacuous definition of the legend's `<amount>`
needs the change distinction.

---

### I-6 — Important — §8, §10.10: nineteen emissions on one channel, with no ordering, no severity, no count and no summary — and the fold made the channel busier

**What the spec says.** Searched `grep -n -i "severity"` → **0 hits**.
`grep -n -i "ordering"` → 1 hit, §10.13 on chunk ordering. §10.10's own
accounting of what remains undone reads, in full:

> **Still unspecified:** the flag spellings themselves, exit codes, and the
> format of the refusal messages §8 promises will *"name the number that caused
> it"*.

Warnings are not listed as unspecified. The gap is not merely open, it is
**untracked** — and the success report was added to the same channel with the
same absence of structure: *"**It goes to `stderr`, with the warnings**"*.

**Count for one plausible invocation** — `mt qr`, RCW `wsh` tier 1, 5 inputs (2
legacy with no `non_witness_utxo`, 3 segwit), no node reachable, `FROM`/`TO` not
supplied, a 6 sat/vB fee, module size 0.45 mm, an enforced block locktime:

| # | emission | § | lines |
| --- | --- | --- | --- |
| 1 | every output, address in full | §10.10 | 2+ |
| 2 | fee, absolute and sat/vB | §10.10 | 1 |
| 3 | locktime, §8.4's two facts | §10.10 | 1 |
| 4 | plate count + engraving time | §10.10 | 1 |
| 5 | configuration | §10.10 | 1 |
| 6 | headroom | §10.10 | 1 |
| 7 | value provenance × 5 inputs | §10.10 | 5 |
| 8 | no bitcoind reachable, 3 checks UNKNOWN | §6a | 8 |
| 9 | `FROM` blank — "loudly warned" | §5, §10.4 | ? |
| 10 | `TO` blank — "loudly warned" | §5, §10.4 | ? |
| 11 | fee rate below 10 sat/vB + CPFP paragraph | §8.2b | 7 |
| 12 | legacy input 0 unbound — capitalised block | §8.2c | 11 |
| 13 | legacy input 3 unbound — the same block again | §8.2c | 11 |
| 14 | module size optically unvalidated | §8.8 | ? |

**Fourteen blocks, comfortably over fifty lines, in unspecified order.** The
report alone is twelve rows before a single warning fires.

**Why ordering is the whole game.** These differ by orders of magnitude in
consequence and nothing distinguishes them:

- **the plate may already be dead** — block 8, the inputs were never checked for
  unspentness;
- **the plate may be wrong about money by 9 BTC** — blocks 12 and 13;
- **cosmetic, fixable at the wallet in seconds** — blocks 9 and 10.

And the ranking that emerges by default is inverted. The **longest, loudest,
most frightening** emissions are 12 and 13 — eleven lines each, capitals, a
worked 9.01-BTC catastrophe — printed **twice** for one transaction. §8.2c's own
fold argued this exact point about a different firing condition: *"A warning that
cries wolf on the normal path has negative value"* and *"training the operator to
ignore the rare case where it is true."* The reasoning is correct and was applied
to one warning's trigger; it was never applied to the **set**.

Meanwhile block 8 — the one that says the transaction may already be
unspendable — is a single block among fourteen, and blocks 9 and 10 are
specified only as *"loudly"*, which is not a specification at all.

**What the user does differently once told.** With the count and ranking emitted
**last**, after the artifact is written, where a fifty-line scroll has not yet
buried it:

    3 things to weigh before you cut 5 plates (~105 minutes):
      [1] inputs NOT checked for unspentness (no node)  -- this plate may already be dead
      [2] 2 inputs have unverified values -- the fee shown may be wrong by any amount
      [3] TO field blank -- the plate will not name a destination

…the operator acts on [1]. Without it they act on whichever block their eye
landed on, which will be the one in capitals.

**Fix:** one paragraph in §10.10, which already owns the CLI contract: warnings
carry a severity, are emitted in severity order, are counted, and are repeated as
a summary block last. Add "the ordering, severity and summary of warnings" to
§10.10's *"Still unspecified"* list so it is at least tracked.

---

### I-7 — Important — §4, §5, §10.8: plates 2..m carry twelve characters, and none of them identifies the set

**What `mt` knows.** The 20-bit `chunk_set_id` — §10.13: *"top 20 bits of the
extracted txid, display form"* — which §3 introduces precisely as the thing that
makes plates unmixable: *"n-of-m **plus a set identifier**, so symbols from two
different transactions cannot be combined. That is strictly stronger than UR,
which has a payload checksum but no set identity."*

**What the spec engraves.** §4's reservation:

> legend: 6 lines reserved on plate 1 (25.5 mm at a 4.25 mm pitch),
>         **1 line on every later plate for "PLATE n OF m"**

So plate 4 of 5 is a steel plate bearing QR symbols, per-symbol `n/m` labels
(§10.8), and the twelve characters `PLATE 4 OF 5`. Nothing names the set.

**Against §10.8's own normative standard.** §10.8 states it as a requirement:

> A recoverer must be able to inventory what they hold and name what is missing
> **without decoding anything**.

Two `mt` jobs, each 11 chunks over 5 plates, sitting in one drawer, produce two
plates reading `PLATE 4 OF 5` with symbols labelled `7/11` and `8/11`. They are
**indistinguishable by eye** and combining them is exactly the failure
`chunk_set_id` was introduced to prevent — machine-readably. The mechanism `mt`
built for this is real, present, and unavailable to the person holding the
plates.

**Distinct from §10.21**, which is *"nothing on the plate names the format"* —
a tool-discovery problem. This is a set-discovery problem, it survives §10.21
being solved, and its fix is different.

**The cost is eight characters on a line already reserved.** I deliberately do
**not** propose putting `BEARER` or any other legend line on later plates: that
needs a second line, §4's search minimises plates, and §10.14 has not yet priced
the per-symbol labels either. `PLATE 4 OF 5 · SET 8f3a1` extends an existing
twelve-character line to twenty-four, well inside the narrowest measured rung
(22 chars/line at the 3.8 mm rung — `backup/sizes_test.go` ladder as quoted in
§5) if abbreviated, and comfortably inside the 26- and 30-char rungs.

**What the user does differently once told.** In 2040, a holder sorts two mixed
sets by eye instead of decoding every symbol on every plate to discover the sets
do not match; and a holder of plate 4 of 5 knows *which* four others to hunt for.
At encode time, the operator can label the storage envelope with the same five
characters.

---

### I-8 — Important — §5: `<amount>` is the only money figure on the plate and it is still undefined

**What the spec says.** §5's field, unchanged since R3:

> | `TO <wallet id, fp or label>  <amount>` | 34 | names the destination
> **wallet**, not one truncated address |

Searched `grep -n "amount>"` → **one hit, that row.** §10.4 mentions it only for
budgeting: *"§5's budget gives `TO` 34 characters including the amount, so a
label has roughly 16."* Nothing defines it.

**Three readings, all plausible, all different**, on any transaction with
change: the total of all outputs; the total paid to the named wallet; the value
of one output. The only concrete instance anywhere is in
`RESULTS_legend_budget_2026-08-22.txt` — `TO bc1p8rrz...s6n0vcl  0.00399 BTC` —
which is the third reading attached to the **truncated-address form §5 replaced
on 2026-08-23**. So the measurement backing the 34-character budget encodes a
semantics the field no longer has.

**What the fold changed, and it strengthens the finding.** §10.10's report now
requires `mt` to identify *"which are change"* (subject to I-5). That is the
exact material a correct definition needs: `<amount>` = the total paid to
non-change outputs. The spec added the capability in one section and left the
field undefined in another.

**Why Important and not a nit.** It is engraved, permanent, and it is the number
a 2040 holder uses to decide whether the plate is worth acting on. A plate
reading `TO ALICE  1.00000000 BTC` on a transaction paying Alice 0.1 and
returning 0.9 as change is **a permanent, specific falsehood about money** — the
precise failure §8.4 refuses to commit elsewhere: *"A `stderr` warning is
disposable; a legend line is forever."*

**What the user does differently once told.** The 2040 holder reads the number as
what it is. The 2026 operator compares it against their wallet's display — which
they can do only once it has one meaning, and only once I-2's echo shows it to
them.

---

### I-9 — Important — §1.8, §5, §10.10: redundancy is zero, `PLATE n OF m` reads as a k-of-n share label, and the operator is told neither at the one moment duplicates are cheap

**What `mt` knows.** That `m` plates are required and none is optional, and that
it has just produced a payload the machine can run again for the cost of steel.

**What the spec rules.** §1.8: *"**Redundancy is zero. `mt` protects against
damage to a plate, not against a missing plate.** The operator is free to engrave
duplicate copies."* §5's rationale column: *"a missing plate must be obvious,
**and all `m` are required** (§3)."*

**What reaches the operator.** Searched `grep -n "ALL PLATES"` → **0 hits**.
`grep -n "duplicate cop"` → **1 hit**, §1.8 itself. §10.10's report has a plate
count row and no redundancy statement. **The ruling is recorded in the spec and
delivered nowhere.**

**What reaches the plate.** `PLATE 2 OF 5`. Twelve characters that state an index
and are silent on the semantics.

**Why the wrong reading is the natural one.** This plate sits in a drawer beside
codex32 and SLIP-39 shares — formats built on the premise that missing shares are
survivable. A holder who has internalised k-of-n reads `PLATE 2 OF 5` as a share
label. Under §1.8 it is not: lose one and the transaction is unrecoverable,
permanently, with the other four plates intact and worthless. This is the same
class as §5's constellation-adjacency hazard, which §7 already records for a
different field: *"an `mt1` plate sits in the same drawer as `md1` and `mk1`
plates, in the same script, differing in **one HRP character**."*

**What the user does differently once told.** At encode time, shown
`5 plates, ~105 minutes. mt provides NO protection against a LOST plate -- only
against a damaged one. If you want redundancy, run this same payload twice now;
the machine is already set up`, they engrave a second set, or store five plates
in five places rather than one envelope. That decision is cheap now and
impossible later. In 2040, shown `ALL PLATES REQUIRED` on the steel, they keep
hunting for the fifth plate instead of concluding that four of five is a partial
recovery and discarding it.

**Cost:** `ALL PLATES REQUIRED` is 19 characters, and it is needed only when
`m > 1` — see the 2040 section for why that conditionality resolves the
line-budget contention.

---

### I-10 — Minor — §10.10: the report costs `mt qr` in minutes and leaves `mt string` uncosted

**What `mt` knows.** The exact string it just emitted, hence its character count.

**What the spec says.** The headroom row gives *"chunks against 64 (`mt string`)
or characters against 8,191 (`mt qr`)"*, which closes R3's proximity concern —
`63 of 64` is now reportable. But the plate-count row attaches a human cost to
`mt qr` (*"since a plate is ~21 minutes (F-225), the **engraving time**"*) and
`mt string` gets no equivalent. A 63-chunk artifact is on the order of thousands
of hand-cut characters, and the report names a number that does not convey it.

**Why only Minor.** `mt string` puts the entire artifact on **stdout**. The
operator holds it and can measure it. `mt qr`'s cost is genuinely hidden inside a
`sysw` payload; this one is not. Stating the count is a courtesy that costs one
`len()`, not a disclosure gap.

**In scope despite §3b's layout ruling.** §3b draws the line itself: *"what `mt`
*emits* is this spec's concern; what a user does with steel is not"*, and keeps
the 64-chunk ceiling on exactly that basis. Character count is a property of the
emission, not of anyone's plate.

**What the user does differently once told.** Shown the count beside the chunk
headroom, an operator weighing hand engraving against `mt qr` weighs it against a
number rather than against an intuition.

---

### I-11 — Minor — §5: nothing dates the plate, and the dropped-fields table justifies the omission with a recovery path that does not recover it

**What `mt` knows.** The date it is running.

**What the spec says.** §5's dropped-fields table:

> | dropped | recoverable how |
> | fee rate and date | inputs − outputs, and the PSBT carries the input amounts |

`inputs − outputs` recovers the **fee**. It recovers nothing about the date — a
Bitcoin transaction carries no creation timestamp. And for `mt string` the row is
false on both counts, as §7 states in terms: *"**Neither is recoverable from an
`mt string` plate's own contents**, since a raw transaction carries no input
amounts (§6)."* Two sections of the same spec disagree on the same fact.

**What is on the plate, temporally.** For a timelocked transaction, the `~<year>`
of the *unlock* — which is a different thing. For `NO BLOCK TIMELOCK`, **nothing
at all**: a holder cannot tell whether the plate was cut in 2026 or 2039.

**Why Minor and not Important, revising R3.** The cut date is partially
re-derivable *after* decoding: every input's outpoint names a confirmed
transaction, whose confirmation height bounds the cut date from below to within
whatever precision a holder cares about. So this is not information that is lost —
it is information that is unavailable **before** decoding, and the action it
changes is diagnostic (how hard to distrust the fee, how likely an input has been
spent) rather than decisive. Real, cheap, worth having; not in the same class as
throwing away four good plates.

**What the user does differently once told.** `CUT 2026-08-23` (14 characters)
tells the 2040 holder: this fee is fourteen years stale, plan CPFP before
broadcasting; the destination wallet is fourteen years old, confirm it exists;
the source wallet has had fourteen years in which an input may have been spent
(§7's Silent-invalidation hazard). It is the field that calibrates how much to
trust every other field.

---

### I-12 — Minor — §5, §10.4, §10.10: a blank legend field engraves a gap, and §5 still calls `FROM` both optional and mandatory

**The contradiction, both inside §5.** Line 548, the field table: `FROM WALLET
<8 hex>` — *"**Optional — loudly warned when absent** (§10.4)"*. Line 635,
eighty-seven lines later:

> **Where the stub comes from is unspecified, and that is an open question**, not
> a settled design: `FROM WALLET` is **a mandatory field** sized into §4's
> reservation, and nothing says what supplies it or what happens when it is
> absent. See §10.4.

§10.4 is marked **CLOSED** and answers both halves — the field is optional, and
§10.10's input table names the operator as the supplier. The paragraph pointing
at it as *"an open question"* with *"nothing says what supplies it"* is stale
text the fold did not sweep. R3 filed this; it survives unchanged.

**And the fold answered the second half the wrong way.** §10.10's input table now
rules: *"`FROM` wallet id / fingerprint \| §5 \| **warn, engrave blank**"*. A
blank line on a bearer plate is indistinguishable, in 2040, from a line that has
corroded away or a plate that was cut short — and §4 reserves six lines
regardless, so the space is spent either way.

This is not contesting the operator's ruling. §10.4's ruling is *"warn if blank
but allow"* — permission for the operator to omit the input. What the plate
*says* in that case is a spec decision, and it was made in a table gloss rather
than argued.

**What the user does differently once told.** `FROM WALLET UNKNOWN` /
`TO UNSTATED` tells the 2040 holder the encoder had no answer, so they stop
looking for a damaged line and start looking for the wallet elsewhere. It costs
nothing and keeps the legend's line count fixed, which §4's reservation assumes.

---

### I-13 — Minor — §8.2b, §10.10: the low-fee warning offers only remedies for 2040, never the one available in the next sixty seconds — and omits the two numbers CPFP sizing needs

**The text, quoted from §8.2b:**

> If it turns out too low, the holder may need CPFP -- spending one of this
> transaction's outputs with a high-fee child, which needs no key from the
> signer -- or out-of-band submission directly to a miner, which bypasses relay
> policy entirely.

**Both remedies belong to *"the holder"*, in the future.** The reader of this
message is the operator, in the present, whose wallet built this transaction
minutes ago and can rebuild it at a higher fee rate before any steel is cut.
Searched `grep -n "re-sign\|resign"` → **0 hits in 1,893 lines**. The one remedy
that is free, immediate, and available only now is the one not mentioned.

**Second half: CPFP needs two numbers and the report supplies one.** A CPFP child
must pay for itself **and** the parent, so sizing it requires the parent's
absolute fee *and* its **vsize**. `grep -n -i "vsize\|vbyte"` → **0 hits.** The
report gives *"absolute and as sat/vB"*, from which vsize is recoverable by
division — but it is never stated, and for an `mt string` plate the fee itself is
unrecoverable from the plate's contents forever (§7: *"a raw transaction carries
no input amounts"*). Encode time is the only moment those numbers exist for
capture.

**What the user does differently once told.** One added sentence — *"If you can
still re-sign in your wallet at a higher fee rate, do that instead of relying on
CPFP"* — sends the operator back to the wallet rather than onto the machine. And
one extra field in the report (`vsize`) is what a 2040 holder needs to size the
child transaction the warning told them to build.

---

### I-14 — Minor — §8.2b: dust outputs are two integers `mt` already holds, and the spec's 1,893 lines never mention them

Searched `grep -n -i "dust"` → **zero hits.**

**What `mt` knows.** Every output's value and `scriptPubKey` type, hence the
standard dust threshold applicable to each.

**Why it is in this lane and not behind an operator ruling.** §8.2's removal
ruled out **script validity**. A below-dust output is consensus-*valid* and
policy-*non-standard*: it makes the transaction unrelayable, so it engraves
cleanly and cannot be broadcast through ordinary channels. Detecting it needs no
script engine, no signature check and no node — it is a comparison of two
integers, squarely inside §8.4's scope line, *"Fields are certain; scripts are
somebody else's job."*

**The parity argument is what carries this.** §8.2b already warns on a
relay-policy fact for exactly this reason, and states the reason: *"A refusal
floor would hardcode today's relay policy into an artifact meant to be broadcast
in 2040."* The same argument that makes a low fee a **warning** rather than a
refusal makes dust a warning. And the same escape hatch applies to both —
out-of-band submission to a miner bypasses relay policy for a dust output as
readily as for a low fee. `mt` warns about one and is silent about the other, on
identical reasoning.

**Why Minor rather than Important.** Wallets do not normally build dust outputs,
so this fires rarely — and per I-6, every added warning is paid for in the
attention of the warnings that fire often. Worth adding; not worth ranking above
the ones that fire every run.

**What the user does differently once told.** `output 1 is 210 sat, below the
330-sat dust threshold for P2TR — most nodes will not relay this transaction`
sends them back to the wallet, before ~105 minutes of engraving produce a plate
that cannot be broadcast in any year.

---

### I-15 — Minor — §10.10: the report never names the transaction it is about to engrave

**What `mt` knows.** The extracted transaction's txid — §10.13 makes it
structural: *"the id derives from the EXTRACTED transaction's txid … The top 20
bits of the txid in its standard display form."* `mt` must compute it to build
the `chunk_set_id`.

**What the spec says.** The report's seven rows do not include it. Searched
`grep -n -i "txid"` → 12 hits, all in §6a's RPC call, §8.2d's binding, and
§10.13's content-id derivation. None is a report row.

**What the user does differently once told.** They compare the txid `mt` printed
against the one their wallet displayed for the transaction they signed —
catching a stale or wrong PSBT file before 105 minutes of engraving. PSBT files
accumulate in a directory and differ by nothing an eye can check; the outputs and
fee catch *most* wrong-file mistakes, and a txid catches all of them in one
token. It also gives the operator the five characters to write on the storage
envelope that I-7 wants on the steel.

---

### I-16 — Nit — §8.4: `NOT ENFORCED` is the one message where the operator has an exact, cheap fix, and it does not say so

The line

    nLockTime 900000 present but NOT ENFORCED (all inputs final)

is correct, and is the output of the single most valuable rule in §8.4 — the
section rightly calls the alternative *"false reassurance on steel, the worst
failure available here"*.

It is also the **only** message in the spec where the operator has an exact,
one-minute remedy: set `nSequence` below `0xFFFFFFFF` on at least one input and
re-sign, which their wallet can do immediately and which cannot be done after the
plate is cut. Every other §8.4 output is a statement of fact where no action
exists; this one is a statement of fact where one does. §8.4's *"facts beat a
verdict"* principle is about not concluding **about the transaction** — it does
not forbid naming an action the operator can take.

---

## The full warning inventory

Channel is `stderr` throughout. "Actionable?" asks whether the message, as
specified, lets the operator do something different **before** steel is cut.
"Prominence" is what the spec specifies, which in every case is nothing (I-6).

### Refusals — all abort; all prominent by construction

| § | refusal | fires when | names its number? | actionable? |
| --- | --- | --- | --- | --- |
| 8.1 | not fully finalized | any input lacks `PSBT_IN_FINAL_*` / non-empty `scriptSig` or witness | should name the input | **yes** — finalize in the wallet |
| 8.2b | `SendingTooMuch` | outputs > inputs | yes, both totals | **yes** — rebuild |
| 8.2b | `AbsurdFeeRate` | rate ≥ 25,000 sat/vB | yes | **yes** |
| 8.2b | duplicate outpoints | any repeated input | should name the outpoint | **yes** |
| 8.2b | empty `vin` | no inputs | n/a | **yes** |
| 8.2c | input value missing | PSBT lacks a UTXO record, operator supplied none | should name the input | **yes** — supply it; but see **I-4**, a reachable node already knows it |
| 8.2d | `non_witness_utxo` txid mismatch | hash ≠ `previous_output.txid` | **yes — "naming both txids"** | **yes** — the PSBT is corrupt |
| 6a | chain value ≠ PSBT's UTXO record | node reachable, records present | **yes — "naming both numbers"** | **yes** — new in this fold, closes R3 I-2. Does **not** cover operator-supplied values (**I-4**) |
| 8.3 | unsigned / unfinalized | duplicate of 8.1 by its own text | — | yes |
| 8.5 | `gettxout` → `null` | node reachable **and** an input spent/absent | should name the outpoint | **yes** — the transaction is dead. Skipped with no node, and §6a now **says so** (R3 I-4 closed) |
| 8.6a | non-`ALL` sighash | `NONE`/`SINGLE`/`ANYONECANPAY` in `scriptSig` or witness | should name the input | **yes** — re-sign `SIGHASH_ALL` |
| 8.6b | no signature in the satisfaction | structural scan finds none (heuristic, self-declared) | should name the input | **yes** |
| 8.7 | over the operator's plate budget | §4's result > stated maximum | **yes — count and what would fit** | **yes** — the model refusal. §10.10 flags that its input is unnamed |
| 8.7b | over 64 chunks (`mt string`) | chunk count > 64 | **yes — count, ceiling, and the alternative verb** | **yes** — the best refusal in §8 |
| 8.7c | over `MAX_SECTION_LEN = 8191` (`mt qr`) | encoded artifact > 8,191 | **threshold pending §10.9** (known open) | **yes** once numbered |
| 8.9 | secrets | an `ms1` / secret payload | n/a | yes |

**Refusals remain in good shape.** §8's closing rule — *"Every refusal names the
number that caused it. A refusal that says only 'too large' costs the operator a
round trip"* — is the right standard, and 8.7 / 8.7b / 8.2d / 6a meet it. The
safety argument §8 says it carries, it carries.

### Warnings and the success report

| § | message | fires when | actionable? | prominence | finding |
| --- | --- | --- | --- | --- | --- |
| 6a | no bitcoind; 3 checks UNKNOWN + plate-time cost | no node reachable | **yes — the model warning**; names the checks, the cost, and the action | unspecified | closes R3 I-4 |
| 8.2b | fee rate below 10 sat/vB | rate < 10 | **partly** — both remedies belong to the 2040 holder | 7 lines, unspecified | **I-13** |
| 8.2c | input value bound by nothing | legacy **and** no `non_witness_utxo` **and** no chain fetch | **yes** — states arithmetic, not advice; best-written block in §8, and its trigger is now correct | 11 lines, capitals, **fires once per input** | closes R3 I-3; volume → **I-6** |
| 8.4 | locktime report, 5 forms | always | **yes** — two numbers side by side, units never mixed | 1 line | — |
| 8.4 | estimated unlock year + reference pair | timelock present | **yes** — freshness disclosed, precision honest | 1–2 lines | — |
| 8.4 | lock height below `MT_REF_HEIGHT` | negative subtraction | **yes** — *"Treat it as spendable now"* | 3 lines | — |
| 8.4 | `nLockTime present but NOT ENFORCED` | all inputs final | **partly** — omits the one-minute fix | 1 line | **I-16** |
| 5 / 10.4 | `FROM` blank | no `FROM` supplied | **yes** if it names the flag; *"loudly"* is not a specification | unspecified | I-12 |
| 5 / 10.4 | `TO` blank | no `TO` supplied | same | unspecified | I-12 |
| 8.8 / 10.1 | module size optically unvalidated | operator picks < 0.60 mm | **yes** — §10.10 now gives it a moment to fire; re-running to price the trade is free | unspecified | closes R3 I-12 |
| 3b | bearer, `mt string` only | every `mt string` run | states the fact; asks for nothing | unspecified | R3 I-8, operator-ruled |
| **10.10** | **success report — outputs, fee, locktime, plates+minutes, config, headroom, value provenance** | **every successful run** | **yes — closes R3 I-1** | **7 rows, no ordering, same channel as the warnings** | **I-3, I-5, I-6, I-10, I-15** |
| — | **what `mt` never checked (no signature, script-hash, tweak, k-of-n)** | *never specified* | — | — | **I-1** |
| — | **the legend text about to be engraved** | *never specified* | — | — | **I-2** |
| — | operator-supplied value vs the chain | *never specified* | — | — | **I-4** |
| — | redundancy is zero; duplicates are cheap now | *never specified* | — | — | **I-9** |
| — | dust outputs | *never specified* | — | — | **I-14** |
| — | ordering / severity / count / summary of all the above | *never specified, and not tracked as unspecified* | — | — | **I-6** |

**The shape has inverted since R3.** Then, eight of seventeen warning rows were
blank in the "fires when" column — the informational half was largely unwritten.
Now six are blank and the report exists, so the remaining gaps are no longer
about *volume*. They are about **structure** (nothing ranks fourteen blocks) and
about **permanence** (nothing describes the artifact, and nothing describes what
was never checked). Both survive adding more rows; neither is fixed by them.

---

## What the 2040 plate should say

The legend is **136 characters, 6 lines**. The binding constraint is not the
300-character text-plate budget — `RESULTS_legend_budget_2026-08-22.txt` is
explicit that 300 is *"35 chars/line x 20 lines, **TEXT-ONLY plate**"* — but the
**strip beside the QR** on plate 1. The same file measures it:

```
  config @0.60mm                  QR mm    strip mm   lines   chars     MINIMAL
  RCW tr key-path  v13 ECC H     46.2mm      32.8mm       7     245        fits
  RCW tr tier4     v22 ECC H     67.8mm      11.2mm       2      70          NO
```

So on the tightest **single-plate** configuration the strip is 7 lines and the
legend is 6: **exactly one spare line**, and every larger configuration already
spends a second plate. §10.14's required regeneration must price this properly —
it must also absorb §10.8's unreserved per-symbol labels and §4's three
unmodelled inputs — so the ranking below is a ranking, not a budget.

### Are the five fields the right five? Four yes, and the fifth is contested

- **`BEARER - ANYONE HOLDING THIS CAN SPEND IT` (41 ch)** — the right call at the
  right length. It is the only fact that changes how the plate is *handled*
  rather than how it is *used*, and it must survive being read by someone with no
  context at all. Do not shorten it. Note it appears on **plate 1 only** (§4),
  which is the correct trade: putting it on later plates needs a new line, and
  §4's search minimises plates.
- **`FROM WALLET <8 hex>` (20 ch)** — earns it, for a reason §5 does not state.
  §5 justifies it as a filing aid (*"to help a human find the right plates"*).
  Its real 2040 value is against §7's **Silent invalidation** hazard: it is the
  only pointer a holder has to the wallet whose UTXOs may have been spent in the
  intervening years. Worth saying, because it changes what the field is *for* —
  and worth engraving as `UNKNOWN` rather than blank (I-12).
- **`LOCKED TO BLOCK <n> ~<year>` (29 ch)** — earns it, **including the
  `~<year>`. R3 argued against the year and I think that argument is wrong.**
  R3's case was that a 2040 reader can convert a height to a date more accurately
  than a 2026 binary could, making the year *re-derivable and drifting* — "the
  worst combination on steel."

  Three reasons it does not hold.

  **First, it fails §5's own test.** §5's stated principle is that *"The legend
  carries only what a human needs BEFORE the QR is decoded."* A bare block height
  is meaningless to a human without a tool; a year is immediately meaningful.
  R3's "any block explorer" concedes a tool, which is precisely the condition the
  field exists to survive without.

  **Second, the re-derivability argument proves too much.** §5's dropped-fields
  principle is *"everything derivable from the decoded transaction is
  duplication"* — and `nLockTime` is in the decoded transaction. If
  re-derivability disqualified the year it would disqualify the height beside it.
  §5 keeps the locktime line anyway, as the exception to that principle, on
  exactly the pre-decode-legibility ground above. The year is the part of the line
  that *is* legible pre-decode.

  **Third, R3 did not weigh the determinism §8.4 bought.** *"Two runs of `mt`, on
  any two machines, with or without a node, produce the **same engraved year** for
  the same transaction."* Without the embedded constant, a permanent number on
  steel would depend on the operator's network. And the drift's direction is
  benign for a *lock*: a year reading later than truth makes a holder wait
  unnecessarily; a year reading earlier makes them try early and fail harmlessly.
  Neither loses funds, and the tilde already marks it as an estimate — which is
  §8.4's own stated reason for the character.

  **Keep `~<year>`.**
- **`TO <wallet id, fp or label>  <amount>` (34 ch)** — earns it **once
  `<amount>` is defined** (I-8). Undefined, it is a permanent, specific claim
  about money with three readings and a measurement file that still shows the
  fourth.
- **`PLATE n OF m` (12 ch)** — earns it, and is incomplete twice over: it does
  not say all `m` are required (I-9), and it does not name the set (I-7).

### What the plate does not say and should — ranked, with the line cost

**1. The set identity, on every plate. Costs no line.** `PLATE 4 OF 5 · SET
8f3a1` extends a line §4 already reserves on every plate from 12 to ~24
characters. It is the cheapest item here and the only one that improves plates
2..m, which today carry twelve characters and nothing else. It gives §3's
`chunk_set_id` — introduced expressly so that *"symbols from two different
transactions cannot be combined"* — to the human §10.8 requires to *"inventory
what they hold and name what is missing **without decoding anything**"*. (I-7)

**2. That all plates are required. Costs one line, but only when it is needed.**
`ALL PLATES REQUIRED` is 19 characters, and the contention with the measured
single spare line **resolves itself**: the line is meaningless on a `PLATE 1 OF
1` artifact, and `PLATE 1 OF 1` is exactly the v13 single-plate configuration
where the strip is tightest. Make it **conditional on `m > 1`**. Multi-plate
artifacts are the ones that already spend a second plate, where §10.14's
regeneration has the most room to work with. (I-9)

**3. When it was cut. Shares the conditional line, or takes the spare one.**
`CUT 2026-08-23 - ALL REQD` is 25 characters, inside every rung of the fork's
ladder from 26 upward; on a single-plate artifact `CUT 2026-08-23` alone (14 ch)
takes the one measured spare line with room over. It is the field that tells the
holder how much to distrust every other field. (I-11)

**4. What it is and what reads it.** Already open as **§10.21** and not re-filed.
Recorded here only for placement: whatever tag §10.21 lands should sit on **every
plate**, on the same line as item 1, because a lone plate 4 of 5 is precisely the
artifact that needs it and precisely the one §4 gives one line to.

### The asymmetry that dominates everything above

For **`mt string`** none of this exists — no legend, no `PLATE n OF m`, no
`BEARER` line, and no human-readable set id or chunk index either, because the
`mt1` header is 41 bits packed into codex32 symbols. §10.8 opens *"Machine-
readably this holds for both verbs"* and its normative sentence then binds *"every
engraved **symbol**"*, which is `mt qr` alone. The requirement is stated and, for
one of the two verbs, unmet.

Layout is the operator's by ruling (§3b, §10.11) and I do not contest it — and
§3b's ruling *"Hand cut plates get a warning on stderr. And that's it"*, plus the
fold's added concession that `mt` *"cannot verify that any warning reached the
plate"*, is close enough to an answer on R3's I-8 that I decline to press it.

What I will note is narrower and stays inside the ruling: the `mt string` warning
is on `stderr` either way, and **its contents are still a spec question**. §8.2c
already establishes the pattern for this exact situation for a *lesser* hazard —
*"An `mt string` operator controls their own plate and **may add a reminder**"*.
Whatever that warning ends up saying, the operator is the only person who will
ever be in a position to put any of items 1–4 onto hand-cut steel, and encode
time is the only moment they can be told which four they are choosing to omit.

---

## Summary of what changed since R3

The fold closed both R3 Criticals and six of eleven Importants, and §6a's no-node
block is a genuine improvement in the *design* of warnings, not just their
coverage. What it did not do is give the growing pile any **structure** (I-6), and
what it never touched is the half of `mt`'s output that is **permanent**: the
legend text nobody reviews (I-2) and the check set nobody discloses (I-1). Both
Criticals are cheap — five lines of `stderr` and four, from data `mt` already
holds — and neither is reachable by adding another row to a report that is already
twelve rows long before the first warning fires.
