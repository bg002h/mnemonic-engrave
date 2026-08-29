# R0 round 16 — VERIFICATION of the r15 fold

**Target:** `design/SPEC_descriptor_input.md` at `52c177a` ("spec: fold R0 r15 --
the last four cells resolved"). `52c177a` is HEAD; tree clean (`git status
--short` empty).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r15.md` (0C/4I/2M/1N, 60-cell
table, 54 cells clean).
**Scope, as briefed:** (1) disposition r15's seven findings by re-deriving ONLY
the six cells they touched; (2) confirm the fold's diffs do not move the other 54;
(3) re-run the mechanical sweeps. Everything from r1–r15, including the 54 clean
cells, taken as settled. **No fresh audit was performed.**
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git diff f1dd328..52c177a -- design/SPEC_descriptor_input.md`
— one file, +32/−19, seven hunks. Every hunk dispositioned below.
**Binaries used:** `/home/bcg/.cargo/bin/me` (`me 0.7.0`), `cargo nextest`
(0.9.140).

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **2** |
| Minor | 1 |
| Nit | 0 |

**The spec does NOT re-close GREEN this round.** The walk lens is complete and
r15's table residue is fully discharged — **all seven of r15's findings are
FIXED, and all six bad cells re-derive clean and single-valued** — but one of the
four fixes (new-I4's discriminator gate) is written as a paraphrase of a rule set
that was never designed to be a gate, and it is wrong in **both** directions.
Both new Importants are fold-introduced and both live in the same seven-line
hunk; the other six hunks are clean.

**Cell verdict: 60/60 single-valued** for r15's ten enumerated classes — the
six bad cells resolved, the 54 clean cells untouched. The two new Importants sit
on a class r15's table never enumerated (**§4.5's format 4 with an origin
prefix**, and colon-bearing non-descriptor records), which is precisely the blind
spot r15 named when it wrote *"a section-by-section lens cannot reach either"*.

---

# Disposition of r15's seven

| finding | verdict | evidence |
| --- | --- | --- |
| **new-I1** (three internal identifiers — `window substitution`, `r14`, `new-M3` — inside the operator-visible quote at §6's `multi`+`/0/*` replacement) | **FIXED** | The quote was rewritten and the directive moved outside, into the row's existing annotation site next to `(R0 r5's NEW-I2 …)`. My own multi-line span extractor over all 45 `*"…"*` spans, pattern `§\d\|F-\d{3}\|R0\|NEW-[A-Z]\d\|new-[A-Z]\d\|walk W\d\|conjunct \d\|EXIT_\|r1[0-6]\b\|carriage rule\|window substitution` → **0 violating spans** (r15 measured 1 at `f1dd328`). **Span count unchanged at 45.** |
| **new-I2** (the semantic substitution newly reached that clause and produced a false, ungrammatical tail) | **FIXED** | §5.3 L1026–1027 now reads *"NEITHER-PATH refusals are exempt, routing nowhere — r15's new-I2"*. I re-enumerated the substitution's reach independently (six candidate sites, below): the exemption removes **exactly the one site** r14's widening newly captured and touches none of the other five. The stock replacement no longer fires there, and the rewritten quote's four clauses are each true in **both** builds (re-derived below). One Minor on the exemption's stated *reason*, not its ruling — **new-M1**. |
| **new-I3** (§5.4 answered `multi`-under-`--as descriptor`-in-the-window twice, nine lines apart, in opposite directions; the window arm's promise permanently false) | **FIXED** | Ruled to conjunct 1's permanent refusal, and propagated to **all seven** sites that speak to the cell — §5.1 L845–850 (window scoped to *"wallets the descriptor path WOULD CARRY in a full build — conjunct 1's shape test included"*), §5.4 TIER L1096–1105 (rewritten), §5.4 FOLLOWER L1107–1112 (already said this; now in agreement, unchanged), §6 L1257, §5.5 L1184, §10 L1609, §11 item 5 case 5. **All seven checked and concordant.** No window text reaches a `multi` in any build. r15's second, independent clause — *"give the window arms a criterion that separates not-yet from never"* — is discharged **structurally** rather than by adding a criterion: the only shape that could reach an arm with a permanently-false promise was `multi`, and it is now outside the window refusal's domain entirely. I checked the resulting domain is still exhaustive and disjoint (four cases, below). |
| **new-I4** (every `--as`-absent classification failure routed into §4's cascade, so a mistyped mnemonic gets descriptor vocabulary at exit 3, falsifying a shipped green test) | **FIXED for the filed defect; the fix's own gate is defective in both directions** | The shape gate at §5.1 L809–816 preserves the shipped exit-4 behaviour for r15's exemplar. Re-measured: `me sysw pack "…abandoz"` → rc=**4**, record vocabulary; `crates/me-cli/tests/sysw_cli.rs:1928` (**citation verified** — `#[test]` at 1927, `fn an_unpackable_record_is_refused_before_a_passphrase_is_minted` at 1928) re-run green: `PASS [0.029s] … 1 passed, 431 skipped`. r15's class-9 pair is single-valued. **But** the gate's four tests are a paraphrase of §6's *five*-step rule, mis-cited as *"rule-4"*, and they drop one of rule 4's two disjuncts (**new-I1**) while inheriting rule 2's deliberate looseness (**new-I2**). |
| **new-M1** (one-carries ruling vs M6 give opposite marking instructions in the window) | **FIXED** | §5.1 L775–777: *"This unmarked ruling covers INPUT-dead values only; a BUILD-dead value is always marked (M6), and where both apply the build marking wins."* Cells 1×WIN×omitted and 2×WIN×omitted are now single-valued. **The outcome matches r15's own suggested fix** (scoping the trigger to input-deadness) at every cell — I checked both readings agree on all six omitted-column cells. |
| **new-M2** (§11 item 5's pin cannot fail on new-I3) | **FIXED** | Item 5 now reads *"Five cases tested"* and case 5 is *"a `multi` form with explicit `--as descriptor` in the window — conjunct 1's permanent refusal, never the window text"*. The gate can now execute against the defect r15 constructed. |
| **new-N1** (dangling *"followed by"*; md1-representability stated twice) | **FIXED** | L845–850 now ends *"…in every build (r15's new-I3). The verdict line is then followed by ONE of two alternative clauses, decided by md1-representability (walk W11 …)"*. The *"followed by"* has a subject; the duplicated selector is gone; one sentence, one criterion. |

**Seven of seven fixed.** No fix was cosmetic, and none of the four Importants
was closed by deletion of the claim rather than resolution of the defect.

---

# The six-cell re-derivation

r15's exemplars, r15's column scheme (`tier · follower · exit · §6 row`). Line
numbers are `52c177a`'s.

### Class 5 (`wsh(multi(2,K1/0/*,K2/0/*))`) × `--as md1` and omitted, FULL build — was ⚠I1

The quote at §6 L1259 now reads, in full:

> *"this is a `multi` policy, which only `--as md1` carries — and md1 cannot
> represent `/0/*`. No `me` path engraves this file as written, in any build.
> Re-export with `<0;1>/*` — carried in every build. (Re-exporting as a
> `sortedmulti` policy keeps `/0/*` but is a DIFFERENT policy — `me` will not
> rewrite it — and needs the scannable-plate path.)"*

Clause by clause, FULL build:

1. *"only `--as md1` carries"* — TRUE. §4.7 conjunct 1's widening is md1-path
   only; §5.5 L1184 `❌ device refuses (§4.3)`; §10 L1609 declines the widening
   permanently.
2. *"md1 cannot represent `/0/*`"* — TRUE, §5.3(a).
3. *"No `me` path engraves this file as written, in any build"* — TRUE, 1 ∧ 2.
4. *"Re-export with `<0;1>/*` — carried in every build"* — TRUE. `wsh(multi(2,
   K1/<0;1>/*,K2/<0;1>/*))` is r15's class 2, md1-carried, `0xd5e52`, and md1
   ships in every build including the window. **This is the clause r15's new-I2
   named as false in its old spelling `(works in every build)`;** the fold
   narrowed the subject from the wallet to the re-export and it is now true.
5. Internal identifiers: **none** (sweep = 0).

Verdict both cells: **F · 5.3m · 3 · L1259 · CLEAN.** (Omitted column reaches the
same text via §5.4's carriage rule — neither path carries, so the input's own
refusal fires directly at 3 rather than the choice block. Unchanged and correct.)

### Class 5 × `--as md1` and omitted, WINDOW — was ⚠I1 ⚠I2

The substitution is now **exempt** here, so the quote above prints unchanged.
Re-checking every clause *in the window*: 1 ✓ (build-independent), 2 ✓, 3 ✓,
4 ✓ (md1 ships in the window), 5 ✓.

The tail *"needs the scannable-plate path"* is the one clause whose window
reading needed work. It is **true**: the re-exported `sortedmulti(…/0/*)` wallet
does need that path. And the journey terminates truthfully — an operator who
takes it runs `--as descriptor` on a `sortedmulti` + `/0/*` file, which is
(a)-shaped, so §5.1's **arm 2** fires: *"…No path in this build engraves this
file. It loses nothing by waiting: keep it, and it packs the day the device
update ships"* — true for that file, because the descriptor path does carry
`sortedmulti` + `/0/*` once it ships (§5.5 row 2, ✅). The primary executable
remedy (clause 4) is stated **first**, works **now**, and the alternative is
explicitly flagged as a different policy. The r14 Minor analysis holds and the
r15 escalation is discharged: nothing tells the operator to wait for a file that
will never pack.

Verdict both cells: **F · 5.3m · 3 · L1259 · CLEAN.** One Minor on the
exemption's stated justification — **new-M1** below.

### Class 2 (`wsh(multi(2,K1/<0;1>/*,K2/<0;1>/*))`) × `--as descriptor`, WINDOW — was ⚠I3

- Tier: passes conjuncts 2–7, md1 admits the shape → **FULL**. §5.4 L1096
  unchanged on this, and the FOLLOWER paragraph's *"any tier may precede any
  follower"* keeps tier and follower decoupled. ✓
- Follower: **one** answer now. §5.1 L845–850 removes the cell from the window
  refusal's domain; §5.4 L1096–1105 and L1107–1112 both send it to conjunct 1's
  admission refusal; §11 item 5 case 5 pins it.
- Text: §6 L1257. **The window substitution does not touch it** — it routes to
  `--as md1`, which is live in the window — so the operator sees byte-identical
  text in both builds, which is what *"in every build"* requires. Its
  parenthetical *"(for use-site paths md1 can represent — otherwise no path
  carries it, and the refusal says so)"* is true for class 2 (md1 does carry it).

Verdict: **F · ADM(c1) · 3 · L1257 · CLEAN, both builds.**

### Class 5 × `--as descriptor`, WINDOW — was ⚠I3

Same route. §6 L1257's parenthetical takes its **second** branch here (`/0/*` is
not md1-representable → *"otherwise no path carries it, and the refusal says
so"*), which is true and consistent with the class-5 `--as md1` cell above.
Matches the FULL-build cell exactly, as the ruling requires.

Verdict: **F · ADM(c1) · 3 · L1257 · CLEAN, both builds.**

**Exhaustiveness of the rescoped window domain, checked rather than assumed.**
Under `--as descriptor` in the window a wallet falls in exactly one of:
(i) no path admits → §4.7 admission refusal (ordering paragraph, sentence 1);
(ii) `multi` — md1 admits, descriptor does not → conjunct 1's refusal (sentence
2); (iii) the descriptor path would carry it in a full build → window refusal,
arms by md1-representability; (iv) descriptor admits but would not carry for a
**non-shape** reason → **empty**, because §5.5's `--as descriptor` column is ✅
for every admitted row and the only ❌ entries are `multi` (case ii) and
miniscript (case i, fails conjunct 1 on both paths). Partition holds; no gap.

### Classes 1 and 2 × omitted, WINDOW — was ⚠M1

`--as descriptor` is BUILD-dead and (for class 2) also INPUT-dead. The new
tie-break gives one answer: **marked** `--as descriptor (not available in this
build)`. Single-valued.

Verdict: **F · BLOCK · 2 · L1252 · CLEAN.**

### Class 9 (`a mistyped BIP-39 word`) × omitted, both builds — was ⚠I4

Shape gate: no `(`, no `Key: value`, not a 78-byte base58check token, not JSON →
not descriptor-shaped → the shipped record-classification refusal stands.
Measured at `52c177a`, `me 0.7.0`: rc=**4**, *"not a form this container can
place: not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a
`text:`/`pass:`/`tx:` record"*. The pinned test is green.

Verdict both cells: **— · NOREC · 4 · L1233-principle · CLEAN.**

**Six of six resolved. The table is 60/60 single-valued for the ten enumerated
classes.**

---

# The other 54 cells — hunk-by-hunk

| hunk | lines | what it changes | cells it can move |
| :-: | --- | --- | --- |
| 1 | §5.1 L775–777 | one-carries ⇄ M6 tie-break | 1×WIN×omitted, 2×WIN×omitted (the two ⚠M1 cells). Adds a scoping clause only; no other cell's marking is mentioned anywhere else in the file (`grep -n "not available in this build"` → 1 hit). |
| 2 | §5.1 L809–816 | the discriminator's shape gate | class 9 omitted ×2 (the two ⚠I4 cells). **Also reaches inputs outside r15's ten classes — new-I1/new-I2 below.** All ten exemplars are unaffected: classes 1–8 are `(`-bearing, class 10 contains a `(`-bearing descriptor line, class 9 is bare text. |
| 3 | §5.1 L845–850 | window domain rescoped + N1 | classes 2, 5 × `--as descriptor` × WIN. Verified inert on the rest: classes 1/3/4 are carried by the descriptor path in a full build → still WIN1/WIN2; classes 6–8 admit nowhere → still ADM; classes 9–10 are not wallets → still CASC. |
| 4 | §5.3 L1026–1027 | substitution exemption | one site (§6 L1259's `multi` replacement). Enumerated below. |
| 5 | §5.4 L1096–1105 | TIER parenthetical rewrite | classes 2, 5 × `--as descriptor`. Tier text untouched, so the FULL/PARTIAL partition is unchanged and `wallet-id:` emission is unchanged. |
| 6 | §6 L1259 | the quote rewrite | class 5 × `--as md1`/omitted × both builds (the four ⚠I1 cells). |
| 7 | §11 item 5 | fourth → fifth case | acceptance only. |

**Substitution reach, re-enumerated independently.** Six sites in §5.3 and §6
mention the descriptor path as a remedy: L958 (§5.3(a)), L1017 (§5.3(a″)),
L1259's main quote, L1277 (§6's (a″) row), L1259's `multi` replacement, and
L1257 (which routes to `--as md1`, not in scope). The exemption fires on
**exactly one** — L1259's `multi` replacement, the site r14's semantic widening
newly captured — and the other four keep the substitution. This is the minimal
possible reach for the fix. ✓

**No hunk moves a cell r15 certified clean.** The 54 stand.

---

# NEW findings

## new-I1 (Important) — the shape gate drops one of §6 rule 4's two disjuncts, so every ORIGIN-ANNOTATED bare key — the ordinary single-sig export shape — is routed to the record-vocabulary refusal at exit 4 and can never reach §5.1's choice block

**Where.** §5.1 L809–813 (the fold's new gate), against §6 L1211–1215 (the rule
it cites) and §4.5's accept table.

**The gate as written.** *"`me` re-reads the whole input through §4's cascade
ONLY when the input is DESCRIPTOR-SHAPED (§6's rule-4 shape tests: a
`(`-bearing expression, `Key: value` lines, a 78-byte base58check token, or JSON
with a descriptor field)."*

**What §6 actually says.** The four tests are not "rule 4" — they are steps
**1, 2, 3 and 4** of §6's five-step rule (`grep -n` at L1208–1216: 1 JSON, 2
`": "`, 3 `(`, 4 extended key, 5 otherwise). The mis-citation is the mechanical
root; the defect is what it dropped. §6 rule 4 has **two** disjuncts:

> 4. input LOOKS like an extended key — **its first non-whitespace character is
>    `[`**, or it is a single base58check token whose payload is 78 bytes …

The gate kept the second and dropped the first.

**The consequence, and it is not a corner.** Every §4.5 format-4 input carrying
an origin prefix begins with `[`, is **not** a single base58check token (the
bracket prefix is not base58), has no `(`, no `": "`, and is not JSON. So it
fails all four gate tests, is declared not-descriptor-shaped, and never reaches
§4's cascade when `--as` is absent. §4.5's own accept table lists three such
inputs as **ACCEPT**:

| §4.5 row | verdict | under the gate, `--as` absent |
| --- | :-: | --- |
| `[4bbaa801/44'/0'/0']xpub…` | ACCEPT → `pkh` | exit **4**, record vocabulary |
| `[4bbaa801/49'/0'/0']xpub…` | ACCEPT → `sh(wpkh(…))` | exit **4**, record vocabulary |
| `[4bbaa801/84'/0'/0']zpub…` | ACCEPT → `wpkh` | exit **4**, record vocabulary |

Measured at `52c177a`, `me 0.7.0`:

```
$ me sysw pack '[4bbaa801/84h/0h/0h]zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4w…'
rc=4
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

That is the **status quo the spec exists to remove** (§1, §2.1). An operator
holding the most ordinary single-sig coordinator export there is — `[fp/84h/0h/
0h]zpub…` from a Coldcard, Sparrow or Passport — types `me sysw pack <key>`, is
refused, and the message names BIP-39, md1/mk1/ms1/mt1 and `text:`/`pass:`/`tx:`
records. It never mentions descriptors as a route, never mentions `--as`, and
names no next action that reaches the wallet. The wallet is admitted, carried by
both paths, and unreachable through the documented discovery path.

**It also falsifies a claim eleven lines below it.** §5.1 L826 states the block
fires *"matching §11 item 5's tested cases **across all four formats**"*. Under
the gate that is false for format 4's origin-annotated members. A conformant
implementation satisfies §11 item 5 by choosing a **bare** key as the format-4
witness (bare `xpub…`/`zpub…` **is** a 78-byte base58check token and passes the
gate), so — exactly as in r15's new-M2 — the acceptance gate cannot execute
against the defect.

**Three further routing losses from the same disjunct**, recorded for
completeness rather than filed separately: §6's three `[`-leading branch-4 rows
(*"a bare key whose path matches no script"* — `[4bbaa801/86'/0'/0']xpub…`;
*"a bare key at account ≠ 0"*; *"a bare key with a fingerprint and no path"* —
`[4bbaa801]xpub…`) all become unreachable with `--as` absent, and a multi-record
input whose descriptor record is an origin-annotated key loses §6's multi-record
row. §11 item 4 survives because those rows stay reachable with `--as` present.

**Not prescribing the fix.** The gate needs to cite §6's rule set rather than
paraphrase it, and the dropped disjunct is the load-bearing half for format 4 —
but note that citing §6 verbatim fixes this finding and **not** new-I2, so the
two need separate edits.

## new-I2 (Important) — the gate inherits §6 rule 2's deliberate looseness, so the fold's own absolute — *"a mistyped mnemonic word must never hear descriptor vocabulary"* — has a one-token counterexample

**Where.** §5.1 L809–816, the `Key: value` test, against §6 L1210 (*"input's
first non-comment line contains `": "` → report branch 1"*).

**Why the source rule cannot serve as a gate.** §6's five-step rule **ranks
cascade failures** for an input already known to be a descriptor attempt. It is
deliberately loose because at that point the only question is *which* branch's
error to report — steps 1–4 are a "most resembles" ordering, not an admission
test, and step 5 catches everything. The fold promotes that ordering into an
**admission gate**, where looseness routes non-descriptors into descriptor
vocabulary. Rule 2's test is a bare substring search for `": "` anywhere in the
first non-comment line.

**The counterexample.** Prefix any label to the input r15 filed:

```
$ me sysw pack 'seed: abandon abandon … abandon abandoz'
rc=4
me: record 0 … is not a form this container can place: not a BIP-39 mnemonic,
not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record. …
```

Classification fails (rc=4, measured). The first non-comment line contains
`": "`, so the gate declares it descriptor-shaped, §4's cascade runs, the cascade
fails, and §6 step 5 yields §6 L1245 at `EXIT_REFUSED` (**3**): *"this is not a
wallet descriptor in any of the four forms `me` reads: a BlueWallet `Key: value`
setup file, a plain BIP-380 descriptor, …"*. **A mistyped mnemonic hears
descriptor vocabulary**, at the wrong exit code, with the record forms it
actually used no longer named — which is exactly the outcome the fold's own
sentence at L814–816 declares must never happen.

The same construction reaches ordinary records: `me sysw pack 'text: hello'`
→ rc=**4** today (measured; the space after the colon is what makes it
unclassifiable), and under the gate it becomes exit **3** with descriptor
vocabulary. A plausible spacing typo of a `text:` record is told its input is not
a wallet descriptor.

**Why Important and not Minor.** This is r15's new-I4 defect class, at the same
severity r15 assigned it, surviving inside its own fix: a missing case in
normative routing whose single derivable answer regresses the shipped surface at
**both** the exit code (4 → 3) and the text. It is not a re-litigation — r15's
exemplar is genuinely fixed and I re-measured it green — but the rule that fixed
it admits a one-token counterexample, and the spec states the guarantee as an
absolute. Under the standing severity rule, a defect in what an artifact
**claims** to have done is blocking; the claim here is *"must never"*.

I checked whether it breaks a green test: it does not
(`grep -rn 'not a form this container can place' crates/me-cli/tests/*.rs` → two
hits, neither operand colon-bearing). That is the one respect in which it is
weaker than r15's new-I4, and it is why the severity rests on the absolute
rather than on a regression.

---

# Minor

**new-M1 — the substitution exemption's stated REASON is false about the only
site it governs, though its ruling is right.** §5.3 L1026–1027: *"NEITHER-PATH
refusals are exempt, **routing nowhere** — r15's new-I2"*. The exempted clause
is §6 L1259's tail *"(Re-exporting as a `sortedmulti` policy … needs the
scannable-plate path)"*, which **does** route somewhere: it routes a **different,
re-exported** wallet to the descriptor path. The trigger (*"NEITHER-PATH
refusals"*) selects the right site and the ruling is correct — the substitution's
stock text would be false there — but a future reader applying the stated test
(*"routes nowhere"*) gets the wrong answer at the one site the rule exists for,
and §5.3's own closing sentence *"No refusal names a flag that refuses in the
current build"* now has a semantic exception that the justification does not
name. One clause: replace *"routing nowhere"* with the true reason — their
routing clauses are about a re-exported wallet, which the stock replacement's
*"keep the export file"* would misdescribe.

---

# Mechanical sweeps — run from my own harness

| sweep | result |
| --- | --- |
| quoted-span identifiers, multi-line aware, 45 spans, pattern `§\d\|F-\d{3}\|R0\|NEW-[A-Z]\d\|new-[A-Z]\d\|walk W\d\|conjunct \d\|EXIT_\|r1[0-6]\b\|carriage rule\|window substitution` | **0 violating spans** (was 1 at `f1dd328`). Span count **45**, unchanged. |
| substitution reach | 6 candidate sites; exemption fires on **1**; over-applies nowhere; under-applies nowhere. |
| `sysw_cli.rs:1928` (new citation) | **correct** — `fn an_unpackable_record_is_refused_before_a_passphrase_is_minted` at 1928. Re-run: `PASS [0.029s] … 1 passed, 431 skipped`. |
| `§6's rule-4 shape tests` (new citation) | **WRONG** — the four tests are §6's steps 1–4, and step 4's `[`-disjunct is dropped. Root of new-I1. |
| `(§5.5, §10; …)` (new citation) | **correct** — §5.5 L1184 `❌ device refuses (§4.3)`; §10 L1609 the permanent-decision bullet. |
| §6 data rows | **34**, unchanged; one row edited in place, none added. |
| §5.4/§5.1/§6/§5.5/§10/§11 agreement on the newly-ruled cell | 7/7 concordant. |
| FULL/PARTIAL partition, `wallet-id:` emission | unchanged — no hunk touches tier text. |

---

# Verified in passing — recorded so a later round does not re-spend it

- **The `multi`+`<0;1>/*` re-export remedy is genuinely executable in the
  window.** It is r15's class 2, md1-carried, and md1 ships in every build. This
  is the substantive difference between the old `(works in every build)` and the
  new *"Re-export with `<0;1>/*` — carried in every build"*: the subject moved
  from the wallet the operator holds to the wallet they would produce.
- **The rescoped window domain closes new-I3's second clause structurally.**
  r15 asked for a not-yet/never criterion in the arms; none was added, and none
  is needed, because the only shape that could reach an arm with a permanently
  false promise was `multi`, now outside the domain. Checked exhaustively (four
  cases, above) rather than assumed.
- **In the window, class 2 omitted marks `--as descriptor` "(not available in
  this build)" for an input it can never carry in any build.** Deliberate under
  the new tie-break, and the block simultaneously offers a live, working
  `--as md1`; the alternative is the longer menu rule §5.1 has now declined
  twice (r14's new-M2, r15's new-M1). Recorded, not filed.
- **Empty / whitespace / class 6–8 / class 10 cells** re-spot-checked against the
  hunks: none of the seven hunks reaches them.
- **Both walked journeys still compose.** Journey 1 (BlueWallet `sh` fixture,
  window, `--as descriptor`) is class 1 column 5 — unaffected by every hunk, and
  the BlueWallet file passes the gate on `Key: value`. Journey 2 (bare BIP-84
  `zpub`, childless) passes the gate on the 78-byte-token disjunct — **it is the
  bare form, so it does not exercise new-I1.** A journey using the origin form
  would have caught it.
- **No cross-document copy.** `grep -rn` over `design/` (excluding
  `agent-reports/`) for the fold's new phrases → 0 hits outside the spec. No
  implementation plan exists yet.

---

# What would re-close the round

**new-I1** — cite §6's five-step rule instead of paraphrasing it, and restore
step 4's first disjunct (*"its first non-whitespace character is `[`"*). Fixing
the *"rule-4"* mis-citation alone does not fix it; the disjunct is the edit.
Then either widen §11 item 5's format-4 witness to the origin-annotated form, or
say plainly that the bare form is the witness — otherwise the gate still cannot
fail on this.

**new-I2** — the gate needs a test §6 rule 2 does not provide. §6's rules rank
branches for a known descriptor attempt; an admission gate needs the converse
question. The distinction the spec already owns is the one r15 named: with `--as`
absent the operator has declared nothing, so an input that looks like a **record
form** should keep the shipped exit-4 refusal even when it also contains `": "`.
Ordering the record-shape test before the descriptor-shape test would do it in
one sentence.

**new-M1** — one clause, as above; it can ride along.

**What is closed and should not be re-opened:** all seven of r15's findings; all
six previously-bad cells, re-derived clean here; the 54 cells r15 certified;
the rescoped window domain and its exhaustiveness; the substitution exemption's
*ruling* and its one-site reach; the conjunct-1 ruling and its seven-site
propagation; the 34-row count; the 45-span count; the FULL/PARTIAL partition;
and everything r15 listed as closed.

One line for the cycle's record: **the fold fixed all seven, and the only defects
are in the one fix that was written as a paraphrase.** Six hunks quoted or ruled;
the seventh restated a rule in its own words, and lost a disjunct and gained a
false positive in the restatement. r15 predicted the shape without predicting the
site — *"never describe code from its doc comment"* generalises to *never
describe a rule from memory when the rule is eleven lines away in the same
file*. Both defects are on inputs r15's ten classes did not enumerate, because
every exemplar in that table was `(`-bearing; a table is only as wide as its
class list, and format 4 was never in it.

---

# What the spec's own text leaves open (carried forward; the round that closes it inherits this list)

**§9 residuals (7), verified unchanged at `52c177a`:** (1) nothing run on
hardware; (2) the three admission-table cells have never been exercised — *a gate
that has never executed*; (3) change addresses and testnet unmeasured in the
`--as md1` address equality; (4) the published `md-codec` 0.42.0 tarball not
byte-compared to the tree; (5) TinyGo compilation of a new `sysw.Classify` arm
unchecked; (6) two negative claims with named, narrower search scopes; (7) §6's
refusal texts *"have not been walked with the operator"* — still stated as open
even though the walk reached refusal text at W5/W11/W13; flagged for a scope
update by r11–r15 and still not updated.

**Parked with S2 (F-418, S1 → S3 → S2):** §11 item 1 (the `Descriptor` classify
round trip), §11 item 4's `--as descriptor`-only refusal rows, and §11 item 6 (a
`ClassDescriptor` record loaded and displayed on a real device, the discharge of
§9 item 2). All three need the device on the bench.

**Named follow-ups the spec defers to:** F-413 (host-side SLIP-132
normalisation), F-414 (descriptor + other records in one container), F-416
(`--in`'s contract note in `SPEC_systemwide_payloads` §5.6), F-417 (md1 wire
extension seam), F-422 (**RULING WANTED**, owning phase *"descriptor-input plan,
before S1 closes"* — only an interim status-quo ruling is recorded), F-420/F-421
(cross-tool referrals, owning phase "with or after S1"), F-423 (plate packing,
fork-side, with S2).

**Plan-phase notes (not findings), carried from r15 and added to here:**
(1) class 10 under any `--as` value gets the unparseable-file refusal rather than
a message naming the record split — truthful, but the multi-record row's own
remedy sends the operator there; (2) **new this round** — §6's three `[`-leading
branch-4 rows and the multi-record row are reachable only with `--as` present
once new-I1 is fixed as written; the plan should say which invocation each §11
item 4 test uses, so the reachability is a decision rather than an accident.
