# R0 round 10 — PROPORTIONAL re-review of the r9 fold

**Target:** `design/SPEC_descriptor_input.md` at `5e6d9a5` ("spec: fold R0 r9 --
wallet-id semantics, the two-tier block, both retractions").
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r9-walk-fold.md` (1C/5I/6M/1N).
**Scope:** the fold only — fidelity to r9's thirteen findings, and defects the
fold itself introduced. Not a fresh audit. r1–r9 measured results, r9's probe
ids, the walk log, the citation gate, F-417/F-418/F-422 taken as settled.
**Reviewer:** independent context, opus tier. Read-only against
`mnemonic-engrave`, `descriptor-mnemonic`, `seedhammer`. The whole fold diff
(`git diff d0647f4 5e6d9a5`, 218 lines) was read; every hunk is dispositioned
below.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **2** |
| Minor | 5 |
| Nit | 2 |

**The spec does NOT re-close GREEN this round.** Two Importants are open, both
introduced by the fold. r9's thirteen are all closed — the fidelity half of this
review is clean. What is open is one new normative rule the fold wrote without
sweeping its own §4.7 against it (**new-I1**), and one **false measured fact**
the fold added at r9's own request (**new-I2**) — a claim that has now survived
the walk, the fold, and r9's presence-only verification of its citation.

---

## Disposition of r9's thirteen

| r9 finding | verdict | evidence |
| --- | :-: | --- |
| **C1** (wallet-id uncomputable / prints a different wallet's id) | **FIXED** | §5.4 L1066–1079. All three failure modes closed: computed over the (a′)-materialised md1 policy on BOTH paths; **"Emitted only when the wallet HAS an md1 policy form"**; honest-absence line for (a)/(a″); "NEVER computed over a collapsed encoding". Agrees with §7 L1419's "rows **may also** carry `wallet_id`". (One evidence-label slip → **new-M2**.) |
| **I1** (follower set enumerated smaller than the governing clause) | **FIXED** | §5.4 L1048–1060. Two tiers; the partial tier explicitly names §4.7 admission refusals, and all four of r9's underivable rows and both threshold rows are covered by the rule. r9's third site (`--as` omitted) is in the full tier by name; the fourth (§5.3(b) label warning) is placed. (The fix introduces **new-I1** and **new-M3**; those are fold-introduced, not I1 residue.) |
| **I2** (id not identical under both `--as` values) | **FIXED** | §5.4 L1066–1071 states the uniform (a′)-materialised base and transcribes r9's measured pair correctly: `24bcacf5…` (md1 / materialised) vs `3bf32c0e…` (literal childless `/*`). Checked against r9's probe table — correct orientation, not reversed. |
| **I3** (annotation asserts BIP-48 to BIP-84 operators) | **FIXED** | §5.3(a′) L959–964. The quoted text now says *"the standard receive/change continuation below such origins … the BIP-44 family's change level, and BIP-388's canonical tail"* — no origin family named. True for a BIP-84 `zpub` (`m/84'/0'/0'` → change/index) and for a BIP-48 cosigner (`m/48'/0'/0'/2'` → change/index) alike. An explicit origin-family-neutrality note follows at L966–970. |
| **I4** (W11's symmetric half unfolded at four sites) | **FIXED** | New NORMATIVE "Window substitution" block, §5.3 L993–998, plus markers at all four sites: §5.3(a) L928–931, §5.3(a″) L986–989, §6 L1184, §6 L1202. The closed-set row L1201 is "ACCEPTS" not "packs". **Reach verified beyond §5.3/§6:** every other `--as descriptor` occurrence in the file (33 total) is spec prose or a capability cell, not an operator-facing remedy — §5.2, §5.5, §10 (L1527/1534/1558), §4.7 conjuncts 1 and 7 (L647/L695–696) and §4.2's zero-fingerprint warning (L398, which fires only ON the descriptor path and so cannot fire in the window) all check out. §5.1's choice block is handled separately by M6. |
| **I5** (three § leaks in quoted texts; sweep falsely called clean) | **FIXED** | All three moved outside the quotes (L1182, L1183, L1196). **Re-ran the sweep multi-line-aware** over all 45 `*"…"*` spans against `§ \| F-\d{3} \| \bS[123]\b \| R0 \| NEW-[A-Z]\d \| walk W\d \| conjunct \d`: **0 hits**. (The one regex match, `BIP-388` at L963, is an operator-checkable authority W9 required, not an internal identifier.) The `/0/*` row L1184 now leads with the verdict: *"md1 cannot carry this wallet as written: …"*. The prior commit's false "sweep clean" is retracted in the fold message. (The uncounted verdict clause is only partly swept → **new-M4**.) |
| **M1** (citation "correction" made a correct citation wrong) | **FIXED** | §5.4 L1090–1092 restored to "R0 r2's NEW-N2"; the "corrected from r2" parenthetical is gone and the prior claim is retracted in the fold message. |
| **M2** (one-step fact missing from window variant 1) | **FIXED** | §5.1 L831–834: *"Available now: --as md1 — me converts and packs in one step: …"* |
| **M3** (BIP grounding not leading the rationale) | **FIXED** | §5.3(a′) L949–956 now opens with the BIPs; *"the device is the reader of both artefacts"* is demoted to *"The device implements these BIPs"*. W9's three-way grounding (BIP-44-family change level, BIP-388 canonical tail, BIP-389 notation) is now in the spec. (Wrong F-number on one leg → **new-M1**.) |
| **M4** (variant 2 lost the operator's own path) | **FIXED** | §5.1 L834–836 carries a substitution slot. (Singular slot vs a mixed (a)+(a″) descriptor → **new-N2**.) |
| **M5** (the one-plate fact absent from §5.5) | **FIXED as specified — but the added claim is FALSE** | §5.5 L1115 now carries it. r9 asked for it and vouched the citation was "real" by **presence** only. Recomputed: it is wrong. See **new-I2**. |
| **M6** (help block offers `--as descriptor` inside the window) | **FIXED** | §5.1 L762–767 marks the value inline in the choice block. (Dangling pointer in the marking → **new-N1**.) |
| **N1** ("4 wrong characters" is substitution-only) | **FIXED** | Both sites: §5.1 L753–755 and §5.5 L1117, each naming substitutions and excluding a missing/extra strike. |

**Fidelity: 13 of 13 closed.** No PARTIAL, no NOT FIXED.

---

# NEW findings

## new-I1 — the two-tier condition is stated over a predicate §4.7 makes `--as`-DEPENDENT, so for every `multi` input the tier is undetermined at two cases the fold's own list places in the FULL tier

**Where.** §5.4 L1048–1053, the sentence the fold added to close r9's I1:

> The FULL block precedes: a pack, the `--as`-omitted choice block, §5.1's
> window refusal, and §5.3's refusals — **every case where the wallet is
> §4.7-ADMITTED.** A PARTIAL block … **precedes a §4.7 admission refusal** …

**Why it does not close.** "The wallet is §4.7-ADMITTED" is not a property of a
wallet. §4.7 conjunct 1 (L640–644) makes admission depend on the flag, for
exactly one class:

> **Shape:** one of the seven forms above — and, **on the `--as md1` path
> ONLY**, the three `multi` twins …

§7's own schema bullet (L1288–1291) was written to prevent precisely this
ambiguity for a different column, and states the fact plainly: *"`me` parses
`multi`, and `multi` is `host_admits=false` … `--as md1` succeeds on `multi`."*

**Constructed failure.** Input:
`wsh(multi(2,[dc567276/48h/0h/0h/2h]xpub…/<0;1>/*,[f245ae38/48h/0h/0h/2h]xpub…/<0;1>/*))`
— a `multi` policy whose use-site paths md1 represents (§5.5 L1108: `--as md1`
✅, carried natively, chunk-set-id `0xd5e52`). Build: **S3-only**, which is the
shipping order §8 fixes (`S3 before S2`, F-418).

Operator types `me sysw pack --as descriptor --in wallet.txt`:

1. §4's cascade parses it — a successful whole-input parse, so §5.4's block
   fires.
2. **Which tier?**
   - The FULL-block *list* contains this case by name: §5.1's window refusal is
     unconditional on input shape (L818–820: "`--as descriptor` **is** a REFUSAL
     … emitted AFTER the host-side parse and the §5.4 identification block").
   - The FULL-block *qualifier* excludes it: under `--as descriptor` this wallet
     is not §4.7-admitted, by conjunct 1.
   - The PARTIAL-block *trigger* never fires: the refusal that fires here is the
     window refusal, not §6's `wsh(multi(…))`-under-`--as descriptor` admission
     refusal.

   So the list says one thing, the qualifier says the other, and the partial
   tier's own trigger is absent. **Two readings, materially different stderr, no
   rule in the document to choose** — the identical shape r9 opened I1 for.

**The same break at a second full-tier case.** A `multi` input with `--as`
**omitted** gets the choice block (full tier by the list). With no `--as` at all,
"the wallet is §4.7-ADMITTED" is not merely contested — it is undefined.

**Why the wrong answer is the likely one, and why it costs.** §6's
`wsh(multi(…))`-under-`--as descriptor` row (L1182) is a §4.7 conjunct-1
admission refusal that **does not cite its conjunct** (verified: it is the only
admission row in §6 carrying no `conjunct N` annotation), so an implementer
classifying refusals by their annotations reads this input as
"not §4.7-admitted" and selects the PARTIAL block. That strips `wallet-id:`,
`address 0:` and the compare prompt from the one operator whose refusal text
(window variant 1, L831–834) says *"Available now: --as md1"* — i.e. the one
being sent onward to engrave. The fold's own justification for the partial tier
(L1054–1058: underivable, unspendable, or anyone-can-spend, so *"a 'compare
before engraving' prompt would be a wrong instruction on every one of them"*) is
**false for this wallet**: it is derivable, spendable, and packable.

**Acceptance consequence.** §11 item 5 requires the window refusal tested with
"an md1-representable input". A `multi` with `<0;1>/*` is exactly that, and the
test author cannot determine the expected block.

**Not prescribing a fix,** but the gap is one clause: state the tier by the
*kind of refusal* rather than by admission, or qualify admission as "admitted on
at least one `--as` path".

---

## new-I2 — §5.5's new one-plate cell is FALSE, and the fork code cited pins the opposite: plates = strings, so the keyed single-sig case is TWO plates

**Where.** §5.5 L1115, added by this fold to close r9's M5:

> | on the plate (walk W1) | a QR — machine-scan only | text cards: 2 strings,
> ~168 chars for keyed single-sig **= ONE plate** (measured; the engraver's own
> plan test pins one md1 card → one plate) |

**The measurement.** `bundlePlatePlan` (`seedhammer/gui/bundle_flow.go:386–399`)
emits **one plate per string**:

```go
for pi, s := range c.strings {
    plan = append(plan, bundlePlate{
        plateIdx:   pi + 1,
        plateTotal: len(c.strings),
        ...
```

and the fork's tree-wide assertion pins it —
`seedhammer/gui/bundle_engrave_test.go:38`:

```go
if p.plateIdx != pi+1 || p.plateTotal != len(c.strings) {
```

So `plateTotal == len(c.strings)`. **2 md1 strings → 2 plates.**

**The cited test measures a different input.** `TestBundlePlanSingleMD1OnePlate`
(`bundle_engrave_test.go:47`) feeds `singleMD1(t)`, whose own doc comment
(`gui/bundle_test.go:25–26`) reads:

> `singleMD1` returns a **single-string (non-chunked)** md1 — a small descriptor
> that legitimately fits one string. `wpkh_basic` is the in-tree single-string
> vector.

The test is named "**Single**MD1" because the *card* is one string. It says
nothing about a 2-string card, and the file's own general assertion says a
2-string card is 2 plates.

**Provenance — this is not the fold's invention, and that is the point.** The
walk states it as settled fact under the heading *"and the one-plate hope is
TRUE, measured"* (`WALK_descriptor_input_2026-08-28.md:449`, L471–473):

> the bequest card … **2 md1 strings, 85 + 83 = 168 characters**; the fork's own
> `TestBundlePlanSingleMD1OnePlate` pins one md1 card = exactly one plate.

r9's M5 then verified the citation was "real" — that the test **exists** — and
asked for the fact to be folded. It is a presence check where a recomputation
was needed. The fold folded it faithfully. The operator's stated hope (walk
L454–455: *"we are hopeful it will be a short 1 plate engraving to get all the
md1 strings"*) is answered **TRUE** by three artifacts in a row, and it is
**FALSE**.

**Why Important and not Critical.** No funds consequence and no wrong wallet —
the backup produced is correct. But §5.5's own header claims *"Every cell was
run"*, this cell was not run for the shape it names, and it is the cell that
answers "how much steel and how much machine time is this?" on the operator's
flag choice. Journey 2's plan ("one md1 plate cut now") needs two blanks and
roughly double the engraving time.

**Why Critical is arguable and I am not taking it:** §5.5's "Every cell was run"
is a stated guarantee and this breaks it. I hold at Important to stay calibrated
with r9's C1 bar (an identifier for a *different wallet* printed above a compare
prompt).

---

# Minor

**new-M1 — the `/**` byte-identity is cited to F-411; it belongs to F-410, and
the substance is TRUE.** §5.3(a′) L953–954: *"BIP-388 defines `/**` ≡ `/<0;1>/*`
as the canonical tail — machine-verified byte-identical in the **F-411**
cycle"*. Machine-checked, three ways:

- The claim is **true**. Ran the debug `md` (not the stale release):
  `wpkh(@0/**)` and `wpkh(@0/<0;1>/*)` both emit chunk-set-id `0x880c7` and
  byte-identical strings.
- It was verified in the **F-410** cycle, not F-411:
  `descriptor-mnemonic` commit `5465253b` *"md-cli: accept BIP-388 `@i/**` …
  Two items from the **F-410** ruling"*, with the assertion in
  `crates/md-cli/tests/cli_bip388_double_wildcard.rs:1` — *"F-410 item 1"*,
  *"THE ACCEPTANCE IS BYTE-IDENTITY, NOT 'IT STOPS ERRORING'"*.
- **F-411** (`FOLLOWUPS.md:14341`) is the origin-note scoping item
  (`@0/84'/0'/0'/0/*`), filed *"while implementing F-410's two surviving
  items"* — a different question entirely.

Inherited verbatim from the walk (`WALK…:284`), so the walk log carries it too.
Note the symmetry with r9's M1, which was also a misattributed citation: the fold
restored one and introduced another.

**new-M2 — §5.4's collapse evidence cites the pair that measures the OTHER
divergence.** L1075–1079: *"NEVER computed over a collapsed encoding: the
collapse mints a DIFFERENT wallet's id (measured, `AltCountOutOfRange` then
`3bf32c0e…` ≠ `24bcacf5…`, R0 r9's C1)"*. Both numbers are transcribed
correctly, but the inequality is `/*` vs `<0;1>/*` — r9's **I2** (a′)
per-flag divergence. It is not a measurement of the (a) collapse, and there
cannot be one: for `/0/*` the honest attempt is `AltCountOutOfRange` and the
true id **does not exist**, which is C1's whole point. The normative rule is
right; only its supporting citation points at the wrong measurement. The
(a)-collapse evidence that does exist is §5.3(a)'s address pair
(`bc1qadgf37z…` vs `bc1qu2cc6t7…`) and the shared chunk-set-id `0x9bf18`.

**new-M3 — "four of those rows" is five.** §5.4 L1054–1056 enumerates the
underivable admission rows as *"(mixed network, single-key-wrapped multi, bare
key in a script slot, hardened use-site)"*. Machine-counted over §6's rows, the
non-consecutive-multipath row (L1200, conjunct 7) carries the same
underivable language — *"It accepts this descriptor and then errors on every
address"* — and is a fifth. The `sortedmulti` too-many-keys row (L1187) is also
uncovered: it is a conjunct-**3** row, not one of the "threshold rows" the
sentence names, though "unspendable" describes it. The normative rule is stated
over *"a §4.7 admission refusal"* generally and is complete, so this is an
inventory defect in the justification, not in the rule. (Standing repo rule:
never hand-count what a tool can count.)

**new-M4 — the verdict-first fix landed on the (a) row and not on its (a″)
twin.** The fold rewrote §6 L1184 to lead with the verdict; its structural twin
L1202 — the row the spec itself binds to it (*"the `multi`-form remedy
replacement of the previous row applies here identically"*) — still opens with
the key name and the mechanism: *"key `@N` (…) uses `<0;1>` with no trailing
wildcard — md1 cannot represent it; …"*. That is the shape L1110's rule forbids
and the one the fold just fixed one row above. r9 explicitly declined to put a
count on this clause, so I have not re-audited it across all 28 rows; I am
recording only the twin asymmetry, which is unambiguous and one edit wide.

**new-M5 — the watch-only line's referent is absent on every refusal path,
and the tier reasoning did not reach it.** W15's line (§5.4 L1083–1086) says
*"this **artifact** can SHOW the wallet's addresses and balances"*. The fold
strips it from the partial tier but keeps it in the full tier, which now
explicitly includes §5.1's window refusal and §5.3's refusals — where **nothing
is packed and no artifact exists**. The fold reasoned carefully about which
lines are wrong on a refusal (L1056–1058, the compare prompt) and applied that
reasoning to one tier only. Conversely the partial tier still prints the
canonical descriptor in full — watch-only material the operator is now most
likely to carry elsewhere, since `me` refused — with the "share it accordingly"
caveat removed and no stated reason (the fold's justification covers the compare
prompt, not this line). Not Important: no wrong action follows either way, and
the compare prompt's presence on refusals is deliberately justified by W13.

---

# Nit

**new-N1 — "see below" points at nothing.** §5.1 L764, the M6 marking:
*"`--as descriptor (not available in this build — see below)`"*. Below that line
in the block are the `--as md1` entry and *"They are not interchangeable"* —
neither explains the unavailability. If "below" means further down the **spec**,
it is the leak class L1110 bans in operator-facing text. The verdict is
delivered inline either way, so nothing is misled; the three words are simply
dead.

**new-N2 — window variant 2's path slot is singular where two offending paths
can exist.** §5.1 L834–836: *"--as md1 cannot carry this wallet's `<the
operator's own path, substituted>` path either"*. §6 L1184 states that a
descriptor mixing an (a)-shaped and an (a″)-shaped key matches both rows and
that *"both fire"* — so a mixed input has two offending paths and one slot. The
pre-fold generic wording ("this wallet's use-site path") had no such problem;
M4's fix introduced the slot. Also worth noting the slot's prose spelling
diverges from §6's terse convention (`<fp>`, `<path>`, `@N`).

---

# Verified in passing — recorded so a later round does not re-spend it

- **The I5 sweep is genuinely clean, multi-line aware.** All 45 `*"…"*` spans in
  the file, matched across newlines, yield **0** internal-identifier leaks. The
  fold message's "ZERO quoted section references" claim is TRUE — unlike the
  prior fold's, which it correctly retracts.
- **I4's reach is complete outside §5.3/§6.** All 33 `--as descriptor`
  occurrences enumerated and classified; no operator-facing remedy outside the
  four marked sites names the flag. §5.5, §10 and §4.5 are clean.
- **§5.4 and §7 now agree on `wallet_id`.** §5.4's "emitted only when the wallet
  HAS an md1 policy form" and §7 L1419's "rows **may also** carry `wallet_id`"
  are the same modality. Both implementations' entry points are md1-based
  (`compute_wallet_policy_id` over an `md_codec::encode::Descriptor`;
  `md.WalletPolicyIdChunks` from md1 strings), so the (a′)-materialised base is
  forced rather than merely asserted.
- **r9's measured pair is transcribed with the correct orientation** in §5.4
  (`24bcacf5…` = materialised `<0;1>/*`; `3bf32c0e…` = literal childless `/*`) —
  checked against r9's probe table, not reversed.
- **The (a′) annotation is true for both origin families.** BIP-84
  `m/84'/0'/0'` and BIP-48 `m/48'/0'/0'/2'` both have change/index below the
  account origin, so "the standard receive/change continuation below such
  origins" holds for the walk's own journey-2 `zpub` and for a cosigner alike.
- **The `/**` ≡ `/<0;1>/*` substance is TRUE and re-measured this round**
  (chunk-set-id `0x880c7`, byte-identical strings) — only the F-number is wrong
  (new-M1). The sugar shipped at `descriptor-mnemonic@5465253b`; F-410's
  "REFUSED" measurement is stale, not contradictory.
- **§8/§11 unchanged by the fold**, and no citation the fold added resolves to a
  missing target apart from new-M1's.

---

# What would re-close the round

new-I1 and new-I2 folded, then a re-review scoped to *"did the fold fix the two,
and did it introduce a defect"*. new-I1 is one clause in §5.4 (state the tier by
refusal kind, or qualify admission as per-path); new-I2 is one cell in §5.5 plus
a decision about whether the honest fact ("2 strings = 2 plates") changes
anything else the walk concluded from the one-plate premise. The five Minors and
two Nits can ride along.

**The fidelity half is closed and should not be re-opened:** all thirteen of
r9's findings are FIXED, the two mechanical sweeps r9 demanded return zero, and
nothing in this round comes from a question r9 did not ask. Both new Importants
are fold-introduced, and both are the same species r9 named at its close —
a normative rule written without sweeping the artifact against it (new-I1), and
a claim asserted as "measured" that no tool was asked to check (new-I2). The
second one is worth one line in the cycle's record: it passed a live walk, a
fold, and a review round, and it took reading the cited test's fixture — not the
test's name — to see it.
