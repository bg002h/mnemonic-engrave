# R0 round 14 — PROPORTIONAL re-review of the r13 fold

**Target:** `design/SPEC_descriptor_input.md` at `b331c77` ("spec: fold R0 r13 --
the carriage rule, propagated everywhere, stated once"). Tree clean at that
commit (`git status --short` empty; `b331c77` is HEAD).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r13.md` (0C/2I/3M/1N).
**Scope:** the fold only — fidelity to r13's six findings, and defects the fold
itself introduced. Not a fresh audit. r1–r13 measured results, sweeps, the walk
log, the citation gate, all rulings and dispositions taken as settled.
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git show b331c77` — `SPEC_descriptor_input.md` only,
+65/−47, one file. Every hunk dispositioned below.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **1** |
| Minor | 3 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round**, and the walk lens is therefore
not complete.

The rewrite is a real improvement and the propagation is, this time, complete —
I re-ran r13's new-I1 construction against every exit-code site in the file with
a multi-line-aware sweep and found **no straggler**. What did not close is one
of r13's own three constructions under new-I2: **construction C, the sibling
invocation, is untouched.** The fold correctly keyed the `--as`-*omitted*
follower on carriage; it left the `--as descriptor`-*explicit* follower keyed on
nothing at all, and left the window refusal's two-arm variant selector — which
still has no arm for an inadmissible input — exactly as r13 measured it. The
asymmetry is now sharper than it was before the fold, because the spec has just
established the ordering `admission refusal > menu` and still does not state
`admission refusal > window refusal`.

Second observation for the cycle's record, and it is the good news: **for the
first time in five rounds the fold did not manufacture a false claim.** r11, r12
and r13 each introduced a wrong sentence in justification prose; this fold's
three new Minors are an off-by-one citation to a number the same commit changed,
an absolute that the new rule's own vocabulary makes false, and a pre-existing
remedy the new rule newly routes traffic to. None is a false statement about the
mechanism.

---

## Disposition of r13's six

| r13 finding | verdict | evidence |
| --- | :-: | --- |
| **new-I1** (the `--as`-omitted exit code asserted unqualified at 3 of 5 sites, including §5.1's topic sentence) | **FIXED — all five sites, no straggler** | Re-run below. §5.1 L739–743 (topic), §5.1 L766–771 (M6 paragraph), §5.1 L810–813 (discriminator), §6 L1221 (`--as`-omitted row), §6 L1251 (multi-record row), §11 item 5 L1644–1649 — six sites now, all carrying the rule. Independent multi-line-aware sweep found no seventh. |
| **new-I2** (the block's firing condition keyed on ADMISSION where the invariant is CARRIAGE) | **PARTIALLY FIXED — constructions A and B closed; construction C untouched** | The predicate is now carriage (§5.4 L1087–1099), stated once and named in five places. A and B re-run below, both clean. **C is not addressed by any hunk in the diff**, and the sentence C turns on survives verbatim in the rewrite (L1079–1080). Re-filed as r14's **new-I1**. |
| **new-M1** (`deeper tails` named in both the PARTIAL class list and the FULL-block `wallet-id:` bullet) | **FIXED** | L1114–1116 now reads *"For a wallet md1 cannot represent — (a)/(a″) shapes; **deeper tails never reach this line, being partial-tier** (r13's M1b) — the line is instead:"*. `grep -n 'deeper tail'` → exactly two hits, L1070 (PARTIAL class list) and L1115 (the disclaimer). The dead clause is gone and its replacement states the correct reason. Verified against the tier rule: `/0/1/*` fails conjunct 7 on both paths → no path admits → PARTIAL → the bullet is unreachable, which is now what the bullet says. |
| **new-M2** (the premise *"admission is `--as`-independent"* is false) | **FIXED** | The sentence is deleted, not weakened. The carriage rule states the correctly-quantified form instead: L1092–1093, *"the §4.7 admission refusal where no path admits the wallet (**that determination quantifies over both paths, so it needs no flag**)"*. §4.7 conjunct 1's md1-only `multi` admission is no longer contradicted; §5.5 L1157's ✅ stands. |
| **new-M3** (the PARTIAL line list sat under an *"`--as` omitted"* lead-in) | **FIXED** | §5.4's preamble is now three labelled paragraphs — **The TIER** (L1061–1077), **The FOLLOWER** (L1079–1085), **The CARRIAGE rule** (L1087–1099) — with the label-warning sentence given its own paragraph at L1101. Measured: blank lines at L1060, L1078, L1086, L1100, L1102. The PARTIAL line list is inside the TIER paragraph where it belongs, and nothing normative sits under an `--as`-omission heading that is not about `--as` omission. |
| **new-N1** (the PARTIAL tier rule stated twice, 21 lines apart) | **FIXED** | Stated once, L1063–1065. The duplicate is gone with the rewrite. |

**Fidelity: 5 of 6 closed outright; 1 partially, re-filed.**

### Re-run of r13's new-I1 construction — BlueWallet `Policy: 0 of 3`, `--as` omitted

**Result: ONE outcome at every site. CLOSED.**

Input: a well-formed BlueWallet export with `Policy: 0 of 3` saved as
`wallet.txt`, invoked `me sysw pack --in wallet.txt` with no `--as`. It parses
whole (§5.1's discriminator, measured on the fork's own `sh` fixture); §4.7
conjunct 2 fails (`1 ≤ k` is false), and conjunct 2 is path-independent, so **no
path admits** and nothing carries it.

| site | says | agrees? |
| --- | --- | :-: |
| §5.1 L739–743 (topic) | *"`EXIT_USAGE` (2) … — FOR AN INPUT AT LEAST ONE `--as` VALUE CARRIES IN THIS BUILD (§5.4's carriage rule; an input nothing carries gets its own refusal directly at (3), never a menu of dead flags)"* | ✅ 3 |
| §5.1 L766–771 (M6 paragraph) | *"And when NEITHER value carries the input, the block does not fire at all"* | ✅ 3 |
| §5.1 L810–813 (discriminator) | *"…at `EXIT_USAGE` (2) when at least one `--as` value carries it in this build — otherwise its own refusal, directly"* | ✅ 3 |
| §5.4 L1087–1099 (carriage rule) | *"the §4.7 admission refusal where no path admits the wallet"* | ✅ 3 |
| §6 L1221 (`--as`-omitted row) | *"For an input nothing carries, the input's own refusal fires directly at (3)"* | ✅ 3 |
| §6 L1251 (multi-record row) | *"…gets the "--as decides" block — or, if nothing carries it, its own direct refusal per §5.4's carriage rule — instead"* | ✅ 3 |
| §11 item 5 L1644–1649 | *"with an input nothing carries, the input's own refusal fires directly at **3**"* | ✅ 3 |

The message the operator gets is §6 L1234's — *"threshold 0 means NO signature is
required: anyone who can see this script can spend from it… if it already holds
funds, treat them as at risk now."* No menu, at any site. r13's stake is
discharged.

**Sweep method (multi-line aware, because r13 recorded that the fold's own sweep
had line-wrap false negatives).** The file was flattened to a single string with
a line-number map, then searched for `EXIT_USAGE`, `as decides`, `choice block`,
`choice text`, `§5.1's block`, `menu`, `usage error`, `exits **2**`, `at **2**`,
each hit printed with ±260 characters of surrounding context so a qualifier that
wrapped across a line break could not be missed. **17 hits, every one
classified.** The ones that speak to a descriptor input are the seven rows above,
all qualified. The rest are the empty-file row (L1219), the whitespace row
(L1220), the multi-operand `EXIT_USAGE` (L790), and the constants recital
(L77) — none of which reaches admission or carriage. A second sweep on
`at least one`, `no path admits`, `carriage`, `CARRIES`, `carries it`,
`uncarried`, `precedence` confirmed no site still states the r12 predicate.

### Re-run of r13's new-I2 construction A — `wsh(multi(2, K1/0/*, K2/0/*))`, `--as` omitted, ANY build

**Result: direct neither-path refusal, no menu. CLOSED.**

Admission: conjunct 1 admits the `multi` twin on the md1 path; conjuncts 2–7 all
hold (`k=2≤n=2`; `n=2≤20`; xpub versions; one network; origins present; `/0/*` is
`/i/*`, inside conjunct 7's closed set L679). So the input **is admitted** — the
case r13 constructed, where r12's admission predicate produced a menu.

- **Tier:** FULL (L1061–1063) — passes 2–7, some path admits the shape.
- **Carriage:** `--as descriptor` does not carry it (device parser takes
  `sortedmulti`, not `multi` — §6 L1230). `--as md1` admits the shape but §5.3(a)
  refuses `/0/*`. **Nothing carries it.**
- **Follower:** L1093–1096, *"the neither-path refusal (§5.3's `multi` clause, or
  the window's variant 2) where an admitted wallet is carried by nothing —
  `wsh(multi(…/0/*))` in every build"*. The rule names this exact input.
- **Message:** §5.3(a)'s `multi` exception (L996–1000) → §6 L1232's replacement
  remedy: *"this is a `multi` policy, which only `--as md1` carries — and md1
  cannot represent `/0/*`. No `me` path engraves this descriptor this release;
  re-export with `<0;1>/*`, or as a `sortedmulti` policy if sorted signing order
  is acceptable (a DIFFERENT policy — `me` will not rewrite it)."* at
  `EXIT_REFUSED` (3).

Checked the message is true as written: `multi` + `<0;1>/*` is carried by md1
(§5.5 L1157 ✅, chunk-set-id `0xd5e52`), so the primary remedy is executable in
every build including the S3-only window. The `sortedmulti` half of the remedy is
not executable in the window — that is **new-M3** below, a Minor, and the
operator still has a working first remedy in the same sentence.

The §5.4 → §5.3 → §6 pointer chain resolves, but note that §6 L1232's row header
reads *"under `--as md1`"* while this invocation supplied no flag; the row is
reached through §5.4's naming of §5.3's clause rather than through the row
header. Recorded, not filed — §5.4 names the clause, not the row.

### Re-run of r13's new-I2 construction B — the Specter `/0/*` JSON, S3-only window, `--as` omitted

**Result: window variant 2, directly, no menu. CLOSED.**

Input: the Specter-era `{label, descriptor}` JSON with `/0/*`
(`nonstandard/parse_test.go:22`, the fork's own fixture; walk W11's *"engraving
tool's core clientele"*).

- Admitted by both paths (conjunct 7 admits `/i/*`) → FULL tier.
- Carriage in the S3-only window: descriptor path not shipped; md1 refuses
  §5.3(a). **Nothing carries it** → L1096, *"and the Specter `/0/*` file in the
  S3-only window"* → the window's variant 2.
- Message (§5.1 L830–831 lead + L842–845 variant 2): *"me: --as descriptor is not
  available in this build. The QR plate needs device firmware this release does
  not include. — --as md1 cannot carry this wallet either — key `@0` uses `/0/*`.
  No path in this build engraves this file. It loses nothing by waiting: keep it,
  and it packs the day the device update ships."*

**Checked for truth, not just for firing.** *"it packs the day the device update
ships"* is true for this input: §5.5 L1155 measures `wsh(sortedmulti(k, …/0/*))`
as descriptor-path ✅. The lead line names a flag the operator did not type, but
variant 2's *"either"* chains to it and the pair reads as an account of both
paths — informative, not false. No finding.

W11's door 3 is closed. r13's central stake — an archival user handed a
two-option menu where neither option can help — is discharged for both classes.

---

# NEW findings

## new-I1 (Important) — r13's construction C survives the fold: the window refusal's variant selector still has no arm for an INADMISSIBLE input, and nothing orders it against the admission refusal. Two §6 rows fire on the same input and say opposite things

**Where.** §5.1 L823–847 (the S3-only window rule and its two-arm selector,
untouched by the fold), §6 L1226 (the window row, untouched), against §6
L1196–1198 (the cause rule ranks cascade failures only), §5.4 L1079–1080 (the
FOLLOWER rule, preserved verbatim in the rewrite), §6 L1247 and L1234 (the
admission rows), §5.3 L1006–1007 (NORMATIVE: *"No refusal names a flag that
refuses in the current build"*), §6 L1207–1208 (NORMATIVE: every row *"names only
next actions executable in the CURRENT build (walk W11)"*).

**What the fold did and did not do.** It keyed the `--as`-omitted follower on
carriage and stated, for the first time, an ordering between two followers:
L1091–1093, the admission refusal beats the menu. It did not state the sibling
ordering. The `--as descriptor`-explicit invocation still has two followers that
both fire from their own checks, with nothing selecting between them:

- §5.1 L823–825 is unconditional on admissibility: *"In a build where the
  `--as descriptor` path has not shipped, `--as descriptor` **is** a REFUSAL at
  `EXIT_REFUSED` (3)"*.
- §6 L1196–1198: the five-step rule *"ranks CASCADE (parse) failures only"* and
  admission rows *"fire from their own checks… the rule never selects them"*.
- §5.4 L1079–1080 says the follower is *"decided independently by §5's own
  logic"* — and §5's logic does not decide this pair. This is the sentence r13
  quoted; the rewrite preserved it unchanged.

**Constructed failure A — two §6 rows, same input, opposite claims. The cleanest
form, because no judgement about which refusal "should" win is needed: one of
them is false whichever wins.**

Input `wsh(sortedmulti(2, [fp/48h/0h/0h/2h]xpub…/<0;1>/*h,
[fp2/48h/0h/0h/2h]xpub…/<0;1>/*h))` — a hardened use-site wildcard — in the
S3-only window, invoked `me sysw pack --in wallet.txt --as descriptor`.

- §6 L1247 fires (conjunct 7, hardened use-site): *"a hardened use-site step
  cannot be derived from an xpub (BIP-32). The device would silently derive the
  UNhardened child and display addresses for a wallet that cannot exist, so this
  is **refused on both `--as` paths**."*
- §6 L1226 also fires (`--as descriptor` in a build where its path has not
  shipped) → §5.1's window refusal, *"alternative conditional on
  md1-representability"*.
- The selector's two arms (L837, L842) are **`{md1-representable}` and
  `{(a)/(a″)-shaped}`** — a partition of ADMITTED inputs by §5.3's
  representability limits. `<0;1>/*h` is neither (a) (a single fixed index) nor
  (a″) (a group without a trailing wildcard); and md1 **can** represent it —
  §5.3 L913–915 records `UseSitePath` as *"`Option<Vec<Alternative>>` plus a
  **wildcard-hardened bit**"*. So **arm 1 fires**: *"Available now: --as md1 — me
  converts and packs in one step… nothing is lost by waiting."*
- **`--as md1` refuses this input, in this build and every build.** §6 L1247 says
  so in the same table. §5.3 L1006–1007's NORMATIVE invariant and §6 L1207–1208's
  binding rule are both violated by a row the spec specifies.

Arm 2 could not have been written for this input even if it were selected: its
text substitutes *"key `@N` uses `<path>`"* against §5.3's (a)/(a″) offenders,
which a conjunct-7 failure is not. **The selector is a partition of the wrong
set.** The same construction runs on every inadmissible-with-representable-paths
class: non-consecutive multipath (§6 L1248), `/0/1/*` and bare fixed indices (§6
L1249), `wsh(KEY)`/`sh(KEY)` (§6 L1246), the single-key-wrapper forms (§6 L1245),
mixed network (§6 L1236), 21 keys (§6 L1235), version bytes (§6 L1237).

**Constructed failure B — the same gap on a funds-relevant refusal.** Input
`wsh(sortedmulti(0, K1/<0;1>/*, K2/<0;1>/*))` in the S3-only window,
`--as descriptor`. Conjunct 2 fails, so it is inadmissible; its paths are
md1-representable, so arm 1 fires again. The operator holding an anyone-can-spend
wallet can be told *"Available now: --as md1… nothing is lost by waiting"* instead
of §6 L1234's *"threshold 0 means NO signature is required: anyone who can see
this script can spend from it… if it already holds funds, treat them as at risk
now."* **Waiting is exactly what they must not do**, and the message that says so
is the one the spec does not order first.

**Why this is Important and not a re-litigation.** It is a missing case in
normative refusal-selection logic, with an operator-visible outcome that is worse
than silence on the journey rule's own test, on a funds-safety refusal, and it
falsifies two sentences the spec states absolutely. It is also **unfalsifiable by
the spec's own acceptance gate**: §11 item 5's sibling test pins *"BOTH
alternative variants… (an md1-representable input, and an (a)/(a″)-shaped one)"*
— the same two-arm partition, so a conformant implementation passes item 5 with
arm 1 wired to an inadmissible input. A gate that cannot fail on the defect is
the closure-is-lens-closure second clause.

**Why it did not close.** r13's *"What would re-close the round"* named the
remedy for new-I2 as *"naming which predicate governs the choice block"* — and
the fold did precisely that, correctly and completely. Construction C is a defect
the prescribed remedy does not reach. Per the standing rule that prescribed fixes
are not authoritative, the defect governs, not the prescription.

**Not prescribing the fix.** The spec already has the ordering vocabulary it
needs: §5.1 L825 says the window refusal is emitted *"AFTER the host-side parse
and the §5.4 identification block"*, and §5.4's tier already requires admission
to be evaluated before the block prints — so admission is decided before the
window refusal is composed. What is missing is one sentence saying which of the
two reaches the operator, and (if the window refusal can reach an inadmissible
input at all) what the selector does with it. The `--as`-omitted branch the fold
just wrote is the natural precedent for whichever way it is ruled.

---

# Minor

**new-M1 — the fold changed §11 item 5 from two cases to three and left its own
citation to item 5 at two, in the same commit.** §5.1 L810–813 (fold-written):
*"…otherwise its own refusal, directly, per §5.4's carriage rule — matching §11
item **5's two tested cases** across all four formats."* §11 item 5 L1648
(fold-written, same commit): *"**All three cases tested** (carried;
inadmissible; admitted but uncarried)."* This is the same shape r13 filed against
this exact sentence — *"the fold edited item 5 out from under its own citation"* —
recreated one round later with a different number. It does not contradict the
behaviour (the rule stated in L810–812 is a correct two-way branch, and "two
cases" reads naturally as its two arms), and §11 is the authority a test author
works from, so it costs a wrong test count only if someone writes tests from
§5.1. Minor, not Important, for that reason. One word.

**new-M2 — §5.1 L768's absolute *"the choice text itself never offers a dead
flag"* is false under the carriage rule's own vocabulary, for the
exactly-one-value-carries case.** The fold gave the block three treatments of a
non-carrying `--as` value and specified two of them: build-unavailable → marked
inline (L766–768, M6); neither carries → the block does not fire (L770–771, new).
The third is unspecified and unmarked: **when exactly one value carries because
the other refuses *this input*, the block fires and offers the non-carrying value
with no mark.** Constructed: `wsh(multi(2, K1/<0;1>/*, K2/<0;1>/*))` with `--as`
omitted in a build where both paths have shipped — md1 carries it, so the block
fires at 2 and offers `--as descriptor`, which §6 L1230 refuses for every `multi`
input in every build. Mirror case: `sortedmulti(…/0/*)` post-window, where the
block offers `--as md1`, which §5.3(a) refuses. **By the journey rule this earns
no behaviour change** — the operator who picks the dead option gets §6 L1230's
refusal, which names the working flag and is better than silence — so the finding
is that L768 states an absolute the spec no longer keeps, three lines above the
sentence that introduces the distinction. The fold's own capitalised *"NEITHER"*
at L770 shows the author knew the case existed.

**new-M3 — in the S3-only window the `multi` + `/0/*` remedy's second half is not
executable, and §5.3's window substitution is keyed lexically on a phrase that
clause does not contain.** §6 L1232's `multi` replacement offers two remedies:
*"re-export with `<0;1>/*`, or as a `sortedmulti` policy if sorted signing order
is acceptable"*. In the window, remedy 1 works (md1 carries `multi` +`<0;1>/*`)
and **remedy 2 does not**: a `sortedmulti` with `/0/*` is refused by md1 (§5.3(a))
and the descriptor path has not shipped, so the operator who re-exports as
`sortedmulti` — a **different wallet**, as the row itself says — is refused again.
§5.3 L1002–1005's window substitution replaces *"every remedy… that **names
`--as descriptor`**"*; this clause routes to the descriptor path without naming
the flag, so the substitution misses it lexically. Against §6 L1207–1208's binding
rule (*"names only next actions executable in the CURRENT build"*), remedy 2 is
non-conformant in the window. **Pre-existing** (the remedy is r5's NEW-I2, the
substitution rule is r9's I4) and reachable before this fold via explicit
`--as md1`; the fold newly routes `--as`-omitted traffic to it via the carriage
rule, which is why it is filed here. Minor because the executable remedy is first
in the same sentence and the wasted path ends in variant 2's honest refusal.

---

# Nit

**new-N1 — the rewrite kept r11's content and dropped r11 from the provenance
line.** §5.4's header now reads *"(walk W13; R0 r9 I1, r10 new-I1, r12–r13: the
three rules below, each stated once)"* — **r11 is not in the list**, while three
r11-originated pieces survive in the rewritten text: the not-an-inventory clause
(L1071–1072, r11's new-M1 with r10's new-M3), the `multi`-in-the-window
parenthetical (L1073–1077, r11's new-M2), and *"the two prior attempts to couple
tier to follower each manufactured a false claim"* (L1084–1085, r11's new-N1 with
r12's new-I1). Five inline attributions were dropped in the rewrite and the
aggregate header did not absorb one of the rounds. The content is intact and this
changes no behaviour; it costs a future round the `git log -S` trail on a
paragraph that has now been rewritten four times. (Recorded and dismissed in
passing: the parenthetical also lost the hedge *"either way"*, which modified the
identification-stripping clause and not the refusal assignment — no meaning
changed.)

---

# Verified in passing — recorded so a later round does not re-spend it

- **Quoted-span sweep clean at `b331c77`:** 45 `*"…"*` spans extracted
  multi-line-aware, searched for `§\d`, `F-\d{3}`, `R0`, `NEW-[A-Z]\d`,
  `new-[A-Z]\d`, `walk W\d`, `conjunct \d`, `EXIT_`, `S[123]`, `r1[0-4]`,
  `carriage rule` → **0 hits**. The fold's new annotations (*"(r13's new-I2)"*,
  *"(§5.4's carriage rule, r12–r13)"*, *"(r13's M1b)"*) all sit outside the
  operator-visible quotes, per walk W5. Span count unchanged from r12/r13.
- **§6 still has exactly 34 data rows**; the fold edited two in place (L1221,
  L1251) and added none.
- **Line widths in the fold's regions:** max 75 (§5.1 topic), 74 (M6 paragraph),
  75 (§5.4 rewrite), 73 (`wallet-id` bullet), 80 (§11 item 5 L1649). The file has
  **140 prose lines over 78 columns**, so 80 is inside its own norm — r13's N1
  was a 105-column outlier and does not recur. No finding.
- **The FULL/PARTIAL partition survives the rewrite exhaustive and disjoint.**
  Conjuncts 2–7 are path-independent, so FULL ≡ (2–7) ∧ (∃path shape-ok) and
  PARTIAL ≡ ¬(2–7) ∨ ¬(∃path shape-ok) — exact complements. A conjunct-3 failure
  with a both-path-admitted shape still lands PARTIAL.
- **The three rules are mutually consistent.** TIER is a function of admission
  only; FOLLOWER is explicitly independent of tier; CARRIAGE is a function of
  admission ∧ representability ∧ window and governs only `--as` omission. No rule
  reads on another's inputs. Cross-checked against every §6 row class: cascade
  rows (L1218, L1222–L1225, L1227–L1229) never reach §5.4; the two
  `EXIT_USAGE`-2 rows (L1219, L1220) precede the parse; the admission and §5.3
  rows are followers under the FOLLOWER rule; L1226 is the row new-I1 is about.
- **The three uncarried classes get correct messages, with one Minor.**
  (1) inadmissible → the §4.7 admission refusal, cause-specific per row ✅;
  (2) admitted `multi` + (a)/(a″), any build → §5.3's `multi` clause → §6 L1232's
  replacement — true as written, with new-M3's window residue on its second
  remedy; (3) admitted, window, (a)/(a″) → variant 2 — true as written, since
  §5.5 L1155 measures the descriptor path as carrying `/0/*` once it ships, so
  *"it packs the day the device update ships"* is not a promise the spec cannot
  keep.
- **The carriage rule's exemplar list is illustrative, and reads that way.**
  L1095–1096 names `wsh(multi(…/0/*))` and the Specter file; the (a″) `multi`
  case (`wsh(multi(…/<0;1>))`) is not listed but is covered, because §5.3(a″)
  L998–1000 states *"the same split as (a), including (a)'s `multi` exception"*.
  No finding.
- **"Fires directly at (3)" does not skip the identification block.** §5.4's
  NORMATIVE opener (L1057) binds the block to *"EVERY successful whole-input
  parse"*, and §6 L1226 says the window refusal comes *"after the §5.4
  identification block"*. "Directly" contrasts with the menu, not with the block.
  Checked because the fold introduced the word at four sites.
- **`wallet-id:` emission is consistent after the fold.** FULL tier ⊃ (a)/(a″)
  shapes (admitted, so FULL) → the *"wallet-id: none"* line; PARTIAL tier prints
  no `wallet-id:` line at all, which is now what L1115 says. A `multi` with
  `/0/*` is FULL and (a)-shaped → *"none"* ✅. Childless inputs materialise per
  §5.3(a′) → a real id ✅.
- **No cross-document copy.** `grep -rn` over `design/` (excluding
  `agent-reports/`) for `as decides`, `carriage rule`, `at least one .*admits`
  → 0 hits outside the spec. No implementation plan exists yet, so all
  propagation remains spec-internal.
- **Both walked journeys still compose.** Journey 1 (BlueWallet `sh` fixture, S3
  window, `--as descriptor` explicit) — admitted, md1-representable, window
  variant 1, correct and unaffected: new-I1 does not touch it because the input
  IS admissible. Journey 2 (bare BIP-84 `zpub`, childless → (a′) materialises
  `<0;1>/*`) — admitted, carried by md1, block fires at 2 with `--as descriptor`
  marked per M6. Neither walked journey is broken by the fold, and new-I1's
  failing inputs are again inputs the walk reached by question rather than by
  step.

---

# What would re-close the round

new-I1 folded: state which of the window refusal and the admission refusal
reaches an operator who types `--as descriptor` on an inadmissible input in the
S3-only window, and — if the window refusal can reach one at all — give the
variant selector an arm for it, since its current two arms partition admitted
inputs only. §11 item 5's sibling test needs the same third case if the answer is
a third variant, since the present two-arm pin cannot fail on this. The three
Minors and the Nit are single-clause edits and can ride along: `two` → `three` at
§5.1 L812; soften L768's absolute or say what the block does with a value that
does not carry this input; bring §6 L1232's `sortedmulti` remedy under §5.3's
window substitution (or key that substitution on routing rather than on the
literal string `--as descriptor`); restore r11 to §5.4's provenance line. Then a
re-review scoped to *"did the fold fix the one, and did it introduce a defect"*.

**What is closed and should not be re-opened:** the carriage predicate and its
propagation to all six sites (r13's new-I1, measured clean by an independent
multi-line-aware sweep), new-I2's constructions A and B, the `deeper tails`
collision, the false `--as`-independence premise, the paragraph structure of
§5.4, the duplicate PARTIAL statement, the quoted-span sweep, the 34-row count,
and the FULL/PARTIAL partition's exhaustiveness under the rewrite.

One line for the cycle's record: **the fold's propagation was complete this time,
and the round did not close anyway — because the finding had three constructions
and the remedy that was written down addressed two.** r13 named a remedy; the
fold executed the remedy exactly; construction C was never in the remedy. When a
finding carries more than one construction, the fold's checklist should be the
constructions, not the "what would re-close" paragraph.

---

# What the spec's own text leaves open (carried forward unchanged; the round that closes it inherits this list)

**§9 residuals (7), verified unchanged at `b331c77`:** (1) nothing run on
hardware; (2) the three admission-table cells have never been exercised — *a gate
that has never executed*; (3) change addresses and testnet unmeasured in the
`--as md1` address equality; (4) the published `md-codec` 0.42.0 tarball not
byte-compared to the tree; (5) TinyGo compilation of a new `sysw.Classify` arm
unchecked; (6) two negative claims with named, narrower search scopes; (7) §6's
refusal texts *"have not been walked with the operator"* — still stated as open
even though the walk reached refusal text at W5/W11/W13; flagged for a scope
update by r11, r12 and r13 and still not updated.

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor` classify
round trip), §11 item 4's `--as descriptor`-only refusal rows, and §11 item 6 (a
`ClassDescriptor` record loaded and displayed on a real device, the discharge of
§9 item 2). All three need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — only an interim status-quo ruling is recorded), F-420/F-421
(cross-tool referrals, owning phase "with or after S1"), and F-423 (plate
packing, fork-side, with S2).
