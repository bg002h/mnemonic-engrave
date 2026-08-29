# R0 round 20 — CLOSURE: verification of the terminal fold

**Target:** `design/SPEC_descriptor_input.md` at `5e3c16b` ("spec: the terminal
fold -- the gate becomes intent + invariants + vectors"). `5e3c16b` is HEAD;
tree clean at dispatch.
**Sources of truth:** `design/agent-reports/FOLD-descriptor-input-terminal.md`
(the fold's own record), `R0-descriptor-input-spec-r18.md` (new-M1/new-M2/new-N1),
`R0-descriptor-input-spec-r19.md` (r19-M1/r19-M2, the §11-item-4 plan note),
`R0-descriptor-input-spec-r15.md` (the 10-class/60-cell table, read for the
omitted-rows ruling).
**Scope, as briefed:** verify the terminal fold proportionally (residue against
source-report text; refactor against mandate; spot-derive gate rows; recompute
the manifest arithmetic); run the standing machine checks from an independent
harness; rule on the two deliberately-omitted rows; the closure judgment.
Everything settled r1–r19 taken as settled. **No fresh audit was performed.**
**Reviewer:** independent context — a DIFFERENT agent from the fold's author.
Read-only on every repo file; this report is the only file written; nothing
committed, nothing pushed.
**Diff read in full:** `git diff 37536f4..5e3c16b` — spec **+190/−54**
(1707 → 1843 lines, arithmetic checks), plus the fold report itself (+174).
**Tools:** `python3` (my own sweep harness, written fresh — not the fold's
`sweeps.py`), `./scripts/plan-cite-gate.sh`, `git`, `grep`/`sed`, source reads
of `crates/me-cli/src/sysw/mod.rs` and `crates/me-cli/tests/sysw_cli.rs`.

---

## Counts

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **0** |
| Minor | 2 |
| Nit | 2 |

**GREEN STANDS. The terminal fold is verified; the spec is sound to hand to the
S1→S3 implementation-plan phase.** Both Minors are plan-owned and non-gating
under the repo's severity rules; both are recorded with owning phases below.

---

# Part 1 — fold disposition

## The residue, item by item, against the source reports' own text

| item | source text required | verdict |
| --- | --- | --- |
| **r18 new-M1** (§5.3's closing absolute) | *"Stating the principle unconditionally, or repairing 'names' → 'offers' / 'points the operator at', closes it"* | **FOLDED — both remedies at once.** §5.3 now reads *"No refusal points the operator at a flag that refuses in the current build; any refusal may DESCRIBE a path's future availability — describing routes nothing."* Unconditional, and the verb repaired. No refusal text changed (diff-verified: the hunk touches only the closing clause) |
| **r18 new-M2** ("identifier" undefined; two decisions to be seen made) | name the character class; be seen deciding underscore and `v:pkh(…)` | **FOLDED at its new home.** `[A-Za-z][A-Za-z0-9_]*` stated, underscore included with its reason (bare `or_d(…)` → miniscript row), `v:pkh(…)` failing on every reading stated. The new home is NON-normative guidance — see the omitted-rows ruling for what that costs |
| **r18 new-N1** (§9 item 7 stale) | narrow to *"no systematic row-by-row walk of §6 has been done"* | **FOLDED.** Item 7 no longer claims §6 was never walked; it records the walk RAN, the lens closed at r19, and the true residual (post-walk rows covered as text; row-by-row walk of the final table = plan-phase option, not a gate) |
| **r19-M1** ("uniform per-line scope" over-claims T4) | qualifier: per-line except the JSON test | **FOLDED.** §5.1 guidance reads *"the first three applied to EVERY line of the input and the JSON test to the WHOLE input (per-line except JSON — r19's minor…)"*. The phrase `uniform per-line scope` greps to **0** (see N1 on the fold report's wording of this claim) |
| **r19-M2** (per-line gate vs whole-input cause selection; buried-key outcome) | state the scope divergence and the buried-key outcome; name the design call | **FOLDED.** §6's new Scope paragraph states the divergence as deliberate, gives the buried-refused-key class its explicit outcome (step 5's generic four-forms at exit 3; dedicated rows reachable for the key alone), names cause-selection-follows-the-opening-LINE as a plan-phase design call, and cross-references §7 clause 6 — which is indeed the buried-keys clause. §5.1's routing tail gained the third case (gate opens, NEITHER parse succeeds → §6 chooses), replacing the deleted false alignment claim |
| **r19 plan note** (mnemonic-first ordering) | §11 item 4's multi-record test must use mnemonic-first | **FOLDED twice** — §11 item 4 requires it with the counterexample rationale, and §7 clause 5 states it inline (*"a descriptor-first row passes with or without per-line scope and witnesses nothing"*) |

**6/6 residue items land, each matching its source report's own prescription.**

## The refactor against its mandate

- **§5.1**: intent + two invariants marked NORMATIVE; precision clause (§7's
  `gate` rows NORMATIVE, row beats guidance); shape tests demoted to explicitly
  NON-normative guidance, substance intact with the two residue fixes at their
  new home; routing tail restated as three explicit cases. All four blocks
  present as the fold report describes. Invariant 1's two citations verified at
  source: `crates/me-cli/src/sysw/mod.rs:211` is `pub fn classify_with`, and I
  read the function whole — exactly six admitted shapes (`tx:`/`pass:`/`text:`
  prefixed, BIP-39 mnemonic, `mt1`, seal-validated `md`/`mk`/`ms`), none
  beginning `identifier(`, so the r19-derived sentence invariant 1 leans on is
  true of the current source. `sysw_cli.rs:1928` is
  `fn an_unpackable_record_is_refused_before_a_passphrase_is_minted`.
- **§6**: the five-step rule stays normative (per the mandate); only the false
  alignment parenthetical was removed and the scope stated honestly. Step 4's
  parenthetical now says "the same leading-segment test as §5.1's gate
  guidance" — correct now that the gate text is guidance.
- **§7**: gate-field schema (4 fields, REQUIRED on `gate` rows, absent
  elsewhere, Rust-only assertions), the seven-clause required-row bullet, the
  `gate` manifest tag at min 33, the overlap rule generalised to the named set,
  `covers` schema updated. All present.
- **Status header**: honest — GREEN attributed to r19, the terminal fold
  explicitly flagged as awaiting its own verification round. §9 item 7 named as
  the walk's one narrow residual.
- **No decision-table cell changed**: every outcome in the new text
  spot-checked below re-derives to the settled answer.

## Gate rows spot-derived (the nastiest, as briefed)

| row | my derivation | matches clause? |
| --- | --- | :-: |
| **clause 5, mnemonic-first multi-record** | line 1 mnemonic fires nothing; line 2 `wsh(…` fires T1 per-line → gate OPEN; whole input parses under no branch; line 2 parses alone → multi-record row, exit 4. Identical to r19's derivation 1, both orderings | ✓ |
| **clause 6, buried bare `Zpub…`** | T3 fires on the `Zpub` line (leading segment = 78-byte base58check payload) → gate OPEN; whole input does not parse; the `Zpub` line is a promotion REFUSE (§4.5 row 3), so MREC's "some individual record parses" condition fails; steps 1–4 fail over the WHOLE input (first char is the record line's; not a single token) → step 5 generic four-forms, exit 3 — exactly the divergence §6's Scope paragraph states | ✓ |
| **clause 2, `seed: my wallet (2 of 3)` and the `text:`+real-xpub row** | first token `seed:`/`text:` (identifier + `:`, not `(`) fails T1; key neither header nor 8-hex fails T2; multi-token line fails T3 — gate CLOSED, record refusal, exit 4. Matches r19's derivation 2, which grounded it on the classifier, not the exemplar | ✓ |
| **clause 1, the fifteen §4.5 rows** | accept set {1,2,5,6,7,13,14} = 7 rows, refuse set {3,4,8,9,10,11,12,15} = 8 rows; union 15, intersection 0 (machine-checked). Matches r18's table (a) row for row, including row 13's leading-segment fix, row 14 after §4.6's trim, row 15's device-accept/`me`-refuse | ✓ |
| **clause 7, the edge tokens** | 77-byte payload fails T3's 78-byte test → CLOSED, exit 4 (r18 table d). `[` alone → T3, step 4 fires, no branch-4 row matches → unparseable-file row + branch 4's error, exit 3 (r18 table d). `xpub…/` trailing slash → T3 OPEN; under the r19-mirrored step 4 ("with or without a use-site tail") step 4 now FIRES, no dedicated row matches → unparseable-file row + branch 4's error, exit 3. The clause states the POST-r19 outcome — correct; r18's table (d) showed the pre-mirror outcome, and transcribing that would have been the error | ✓ |
| **clauses 3, 4** | mistyped mnemonic and the four malformed bech32 strings: gate CLOSED (bech32 charset is not a base58check envelope), record refusal naming the class, exit 4 — matches r18 table (c) | ✓ |

**No relocated sentence changed a settled outcome.** The one place where the
fold had to choose between a stale derivation (r18's pre-mirror trailing-slash
row) and the current tree's answer, it chose correctly.

## Manifest arithmetic — recomputed from the bullets as written, not inherited

- Machine-summed from the spec's manifest table (my harness): **9 tags**,
  minima **4+15+14+1+5+3+3+6+33 = 84** tag-slots; stated 84 ✓.
- Gate clause sum: **15+6+1+4+1+3+3 = 33** = the `gate` minimum ✓.
- Overlaps from the overlap rule as written: thirteen §4.5 rows carry 2 tags
  (1 overlap slot each = 13); the `xpub…\n` and bare-`xpub` rows carry 3 tags
  (2 each = 4); total **17** ✓ (equivalently: 15 gate second-tags + the
  original pair's 2 pre-existing overlaps).
- Floor: **84 − 17 = 67** ✓. Cross-check by the other route: r19's verified
  floor 49 + 18 gate-only physical rows (6+1+4+1+3+3) = **67** ✓.
- 1707 + 190 − 54 = 1843 = `wc -l` ✓.

**CONSISTENT on every axis, from two independent directions.**

---

# Part 2 — machine checks (my own harness, not the fold's)

| check | method | result |
| --- | --- | --- |
| **W5 quoted-span sweep** | whitespace-flattened file, all `*"…"*` spans extracted, matched against the r19 pattern widened with `terminal fold`, `gate bullet`, `clause \d`, `invariant \d` | **45 spans, 0 violations** — count identical to r16–r19; independently reproduces the fold's number |
| **substitution reach** | trigger sentence located; substitution-marker sites counted | *"NEITHER-PATH refusals are exempt"* intact; 2 × "Window substitution per §5.3" (§6's `/0/*` and `<0;1>` rows) + 2 × "substitution applies" (§5.3(a), (a″)) = **4 sites taking substitution + 1 exempt** (the `multi`-form replacement, quoted at the `/0/*` row and incorporated by reference at the `<0;1>` row) — the 5-site/4+1 structure unchanged from r18/r19 |
| **citation gate** | `./scripts/plan-cite-gate.sh design/SPEC_descriptor_input.md` | **26 ok; exactly the 5 known cross-repo Rust failures** (`md-codec/src/tlv.rs:10`, `src/encode.rs:17`, `src/tlv.rs:24`, `src/use_site_path.rs:43`, `src/use_site_path.rs:49`) — the settled set, nothing new |
| **stale-phrase sweep** | grep for the fold's list plus my own additions (`49-row`, `8-tag`, `eight tags`, `two permitted overlaps`, `second tag is permitted only`, `the two normative shape tests`) | **0 hits on every superseded phrasing.** `uniform` itself hits 5 times — all pre-existing, unrelated text (3 × the `SPEC_constellation_cli_uniformity` document name, walk W10's "uniform base") — see N1 |
| **normativity-leak scan** | every sentence containing NORMATIVE checked for `shape test`/`four tests`/`T1–T4`/`per-line test` | **0 leaks** — no text still calls the demoted shape tests normative; the only NORMATIVE claims near the gate are the intent/invariants block, the precision clause, and §6's five-step rule, as mandated |
| **manifest arithmetic** | machine-summed (above) | consistent |
| **diff containment** | `git diff 37536f4..5e3c16b --numstat` | spec +190/−54 plus the fold report +174; **nothing else** |

---

# Part 3 — the ruling on the two deliberately-omitted rows

The fold's question 1 — does intent + invariants + 33 rows pin everything the
demoted prose pinned? — has a precise answer: **not everything; the normative
core is now partial where the prose was total, and the frontier is four input
classes, all refusal-only.** Neither invariant claims them (not record material
under invariant 1, not admitted spellings under invariant 2), no gate row pins
them, and none is a cell of r15's 60-cell table (verified against r15's 10-class
list: classes 1–8 are wrapped descriptor exemplars, 9 is garble/mistyped-word,
10 is multi-record). The four: `deadbeef: xpub…`-class 8-hex lines; bare
miniscript fragments (`or_d(…)`); bitcoin addresses; a 78-byte base58check
token of non-key version. **The load-bearing safety property survives on all
four:** any input that PARSES as one descriptor is an admitted spelling and
falls under invariant 2, so frontier divergence can only ever choose between
two refusals (exit 3 with truthful text vs exit 4 shipped record refusal) —
nothing on the frontier can be divergently ACCEPTED or packed. Funds-safe by
construction.

## Bare `or_d(…)` — GENUINELY OPTIONAL

Pre-fold the outcome was already two-valued: r18's new-M2 filed exactly this
("identifier" undefined in then-normative prose), and it was ruled Minor there
because §6's miniscript row stays reachable with `--as` present and via the
realistic `wsh(or_d(…))` spelling, which fires T1 under every reading. The
fold decided the ambiguity in guidance; pinning it executably would be a strict
improvement, but its absence loses nothing r19's GREEN had. One optional row
for the plan.

## `deadbeef: xpub…` — REQUIRED (Minor, plan-owned; see r20-M1)

This one is different in kind, three ways:

1. **It is the only case where the demotion lost a previously-NORMATIVE
   outcome.** Pre-fold, T2's 8-hex disjunct was normative prose: the gate
   OPENS, and the input lands on the headerless-BlueWallet branch-1 error at
   exit 3 — "r17's intended flip", confirmed by r18's table (c). Post-fold,
   nothing normative requires the flip: the 8-hex disjunct is guidance, no
   invariant claims the input, no row pins it. Two conformant implementations
   may now return exit 3 or exit 4 for it.
2. **It sits exactly on invariant 1's boundary.** "Record material … or
   mistyped attempts at them" is the one normative phrase nearby, and an
   implementer who reads a `key: value` line as a mistyped record attempt
   would judge the intended flip a violation of invariant 1. The gate-row
   mechanism ("where any reading of the guidance disagrees with a gate row,
   the row is the answer") was built to arbitrate precisely this, and the
   arbiter row is the one that was omitted.
3. **It is T2's only would-be witness in the adversarial set.** No `gate` row
   fires T2 at all (clause 1 is T3, clause 5 is T1, clauses 2/3/4 and the
   77-byte token are gate-CLOSED, clauses 6/7 are T3): delete T2's 8-hex
   disjunct from an implementation and all 33 rows still pass. It is also
   clause 2's missing counter-witness — six rows pin "colons and payloads
   inside records don't open the gate" and none pins "but an 8-hex key does."

Severity is **Minor, not Important**, by this cycle's own settled calculus:
r18-M2 and r19-M2 — the identical shape (two truthful refusals, divergent exit
code on a non-enumerated input, the specific §6 row still reachable with
`--as` present, no accept/pack divergence) — were both ruled Minor, and I will
not inflate the ladder at the closure gate. The remedy is the fold's own
stated one: one row (extend clause 2 or add a clause 8), `gate` min 33→34,
floor 67→68. Owning phase: **the descriptor-input plan, before S1's vector
file closes** — it is one line of spec text plus one vector row, and per the
proportional re-review rule a fold of that size does not re-trigger a gate.
The addresses and non-key-version rows are optional in the same breath (each
previously pinned gate-CLOSED/gate-OPEN respectively, each one row).

---

# Findings

## r20-M1 (Minor, owning phase: descriptor-input plan, before S1's vector file closes) — the `deadbeef: xpub…` gate row is required; the gate's normative frontier should be pinned where it used to be

As ruled above. Concretely for the plan author: (a) add the `deadbeef: xpub…`
row — gate OPEN via the 8-hex key, outcome `descriptor-refusal`, refusal_row =
the headerless-BlueWallet/branch-1 row, exit 3 — as clause 2's counter-witness
or a clause 8; bump `gate` 33→34 and the floor 67→68 in the same touch;
(b) optionally pin the rest of the frontier (bare `or_d(…)` → miniscript row
exit 3 per the guidance's stated identifier class; a bitcoin address → gate
CLOSED, record refusal exit 4; a 78-byte non-key-version token → gate OPEN,
branch-4 ANY-version row exit 3), one row each. All frontier divergence is
refusal-only and funds-safe; none of this reopens the spec gate.

## r20-M2 (Minor, owning phase: descriptor-input plan, S1) — gate rows do not state their delivery mode, and clause 2's `tx: zz` row cannot meet its asserted exit code on argv

§7 says gate rows are asserted "against the real `--as`-omitted invocation" but
not how the input is delivered. For `tx: zz` on **argv**, the shipped
bearer-on-argv guard exits **3 before classification ever runs** (r18's own
measurement, table (c)), so the row's `exit_code: 4` is only reachable via
`--in`/stdin. Stated so it cannot be mis-fixed: when the freshly-written test
fails on an argv delivery, the correct repair is the delivery, **not** relaxing
the expected exit to 3 — the guard-downstream-of-the-parser trap. One clause in
the plan ("gate rows deliver via `--in`") closes it; it composes with r19's
carried note that per-line scope over a multi-operand argv invocation means the
LF-separated record stream.

## r20-N1 (Nit, fold report only — no spec change) — the fold report's "grep: 0 hits" claim for `uniform` is false as worded

`design/agent-reports/FOLD-descriptor-input-terminal.md` says *"The word
'uniform' is gone from the file (grep: 0 hits)"*. Measured: **5 hits**, all
pre-existing and unrelated (the `SPEC_constellation_cli_uniformity` document
name ×3, walk W10's "uniform base", "uniformity"). The substantive claim is
true — the r19-M1 phrase `uniform per-line scope` greps to 0 — but the report
stated a measurement it did not make accurately. Recorded because mis-measured
report claims are this constellation's known weak half; nothing to fix in the
spec, and the fold report stays verbatim per the persist rule.

## r20-N2 (Nit, cosmetic, batch with any future spec touch) — "an intent, an invariant" vs two invariants

§5.1's intro bullet says the gate's normative content is "an intent, an
invariant, and §7's executable gate rows", and the block heading says "intent
and invariant"; the block then states **two** numbered promises. One word
("invariants"). No reading of the mismatch changes any outcome.

**Process observation, no severity:** `5e3c16b` carries the fold and the fold's
own report in one commit. The persist-before-fold rule binds review reports and
the folds that respond to them (honored: r19's report is `37536f4`, its own
commit, before this fold); a fold author's self-report documenting the same
commit is not that case, and bundling makes the commit self-documenting. Noted
so the lineage is explicit.

---

# Verdict

**GREEN STANDS — 0 Critical / 0 Important. The terminal fold is verified: all
six residue items land as their source reports prescribed, the refactor
matches its mandate, no settled outcome moved, the arithmetic is consistent
from two directions, and every standing machine check passes from an
independent harness. The spec ships to the S1→S3 implementation-plan phase.**

The deliberate structural trade — prose totality exchanged for executable
precision — is sound: what the demotion leaves unpinned is a four-class,
refusal-only, funds-safe frontier, of which one class (r20-M1) must be pinned
back during plan-phase vector authoring and the rest are optional rows.

## What the spec leaves open — consolidated for the plan author, one place

**§9 residuals 1–7** (verbatim in the spec, unchanged in substance): nothing
has run on hardware; the three admission-table cells have never been exercised
(the first `ClassDescriptor` record will be the first ever); change-chain and
testnet address equality unmeasured; the published `md-codec` 0.42.0 tarball
not proven byte-identical to the tree's; TinyGo build of a new `sysw.Classify`
arm unchecked; the negative claims' search scopes named and bounded; §6's
row-by-row walk of the final table is a plan-phase **option**, not a held gate.

**Parked with S2** (F-418 — S1 → S3 → S2 order; needs the device on the
bench): §11 item 1's `sysw.Classify` arm and Go-test `sysw_class` exercise;
§11 item 6's on-device `ClassDescriptor` display before "shipped"; §6's
`--as descriptor`-only refusal rows within §11 item 4.

**Plan-phase work items:**
- Author `descriptor_seam_vectors.json`: **67-row floor, 9-tag manifest, 84
  tag-slots, `gate` min 33** (→ 34/68 with r20-M1), one sha256 pinned in both
  repos, `covers` + `md1_admits` on every row, gate fields Rust-asserted only
  (the Go test ignores them).
- **Two operator rulings due before S1 closes** (both `#ruling-needed`,
  owning phase stated in FOLLOWUPS): **F-413** (host-side SLIP-132
  normalisation vs refusing `ypub`) and **F-422** (consented `/0/*` →
  `<0;1>` transform under `--as md1`).
- **F-416** (owning phase: this cycle, at ship): `SPEC_systemwide_payloads`
  §5.6's `--in` single-document amendment.
- **F-414** (owning phase: post-cycle): descriptor packed together with other
  records — the capability behind the multi-record row; out of this cycle's
  scope but the row's text is written for its absence.
- The multi-record row's test uses the **MNEMONIC-FIRST** ordering (§11 item
  4, now spec text — a descriptor-first input witnesses nothing).
- The cause-selection-follows-the-opening-LINE question is an explicitly named
  **plan-phase design call** (§6's Scope paragraph); until made, the specified
  outcome is step 5's generic text for buried refused keys.
- r20-M1 (the required `deadbeef` row + optional frontier rows) and r20-M2
  (gate-row delivery via `--in`), above.
- Carried plan-phase notes from r18/r19, still true: a mistyped or truncated
  extended key hears the record refusal (the gate and §6 step 4 agree on an
  exact 78-byte payload); class 10 under an explicit `--as` gets the
  unparseable-file refusal, not a split-naming message; per-line scope over a
  multi-operand argv invocation means the LF-separated record stream — one
  clause in the plan.
- A plan's GREEN expires: re-validate the plan against the tree immediately
  before dispatching each implementer, and record the plan's baseline revision
  in the plan (standing rule; `scripts/plan-staleness-check.sh` exists).
