# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 2 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` (488 lines, as
folded at `7877aa5`).
**Round 1:** `design/agent-reports/R0-S2-plan-r1.md` (RED, 4C/6I/7M/3N), persisted
at `191bfb7`, folded at `7877aa5`.
**Trees:** mnemonic-engrave `d962df1` (plan file last touched `7877aa5`);
seedhammer fork `main` @ `a5e29b4`. Nothing in either repo was modified.

**THE ONE QUESTION:** does the folded plan resolve each of r1's 20 findings, and
are the fold's NEW decisions sound — is the plan now buildable as written, with
no gate that cannot be met and no new defect the fold introduced?

**Counts: 1 Critical / 2 Important / 4 Minor / 3 Nit — verdict RED.**

This is a much better artifact than r1's. Sixteen of the twenty findings are
resolved outright, three of the four new phases (P1.0, P2.5, P2.6) are sound
under measurement, and the one Critical is a **single row** in a 71-row file —
the residue of r1's C3, closed to 16 of its 17 members.

---

## Method

- `me` built at the reviewed tree (`cargo build --locked -p mnemonic-engrave`,
  exit 0) and run on constructed inputs.
- A Rust probe crate (`mnemonic-engrave` as a path dependency, target dir
  outside both repos) calling `descriptor::host_admits`, `descriptor::format_of`,
  `descriptor::gate_opens`, `descriptor::cascade::{cascade,normalise}` and
  `sysw::classify` over all 71 vector rows and over synthetic records.
- A Go probe module (`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`,
  go 1.26.3 from the nix store) calling `nonstandard.OutputDescriptor` over all
  71 rows, with `recover()`, both raw and `strings.TrimSpace`d.
- `scripts/plan-staleness-check.sh` against both baselines.

### Verified by measurement — the fold's new decisions that are SOUND

| fold decision | verdict |
| --- | --- |
| **P1.0's consult-first restructure is behaviour-preserving pre-arm** | **CONFIRMED.** Swept all 71 vector inputs plus synthetic classifying records (12-word mnemonic, `text:`/`pass:` hex records, an md1-shaped string, a bare origin-annotated xpub, and mixed documents): **zero** documents where `gate_opens(normalise(doc))` is true AND every non-blank line classifies non-`Unknown`. The reason is structural: every class `classify` can answer needs a reserved prefix (`text:`/`pass:`/`tx:` — and `header_key` stops at `:`, so T2 cannot fire), a BIP-39 word list line, or a bech32 string — none of which can satisfy T1 (`ident(`), T2 (a BW header or 8-hex key), T3 (single token starting `[`, or a 78-byte base58check leading segment) or T4 (the WHOLE document is JSON with a `descriptor` key). Measured on the record corpus's own shapes: `text:2861292862296328` → `gate=false`, `FreeText`. |
| **The named witness is sensitive to the C1 regression** | **CONFIRMED.** `item_5_the_five_case_matrix` (`crates/me-cli/tests/descriptor_refusals.rs:829-849`) uses `vector_input("formats-happy/bip380-sortedmulti-multipath")` with `flags: vec![]` and asserts exit 2. Measured today: that input, `--as` omitted, exits **2**. With a Descriptor arm and no P1.0, `admit_check` returns `Ok` and the run packs at exit 0 — the test reds. Placing P1.0 *before* the arm, with this test green at every commit boundary, is the right instrument. |
| **The MultiRecord branch does not regress under the restructure** | **CONFIRMED.** `mnemonic + descriptor` as two records: today `[Mnemonic, Unknown]` → `admit_check` errs → `consult` → `MultiRecord` → exit 4. Post-arm both records classify, so without P1.0 it would PACK; with P1.0 `consult` runs first and returns the same `MultiRecord` → exit 4. Exit code unchanged. |
| **P2.5's `show` surface is implementable as written** | **CONFIRMED, and the brief's worry does not bite.** `identify::block(d: &Parsed, form: Option<Form>)` (`crates/me-cli/src/descriptor/identify.rs:45`) takes `form` as an **`Option`**, and `descriptor::identification_block(&str, None)` (`crates/me-cli/src/descriptor/mod.rs:74`) is the exact call `main.rs` already makes on the `--as`-omitted path. `print_mdmk_confirmation` (`crates/me-cli/src/main.rs:2048-2063`) already holds the public-section record strings and already calls `sysw::classify` per record. `Class::Descriptor` is **not** secret (`crates/me-cli/src/sysw/record.rs:74-79`), so the record is in the public section even for a sealed container. No `form` is needed at show time. |
| **The transient two-copy window does not red either suite** | **CONFIRMED.** Each repo's test pins the sha of **its own** copy (`crates/me-cli/tests/descriptor_seam.rs:44-47`; `nonstandard/descriptor_seam_test.go:39-41`), and §7 requirement 2 states the rule verbatim: *"Neither test reaches across repos."* Grepped: no engrave test references `third_party/seedhammer`, and the submodule is pinned at upstream `v1.4.2` (`713aee2`) with **no** `nonstandard/testdata` directory. P2.6-then-P3.3 cannot red a suite between phases. |
| **The new witness row behaves as claimed** | **CONFIRMED, measured on `me` at the reviewed tree.** `wsh(multi(2,[dc567276/48h/0h/0h/2h]xpub…/0/*,[f245ae38/48h/0h/0h/2h]xpub…/0/*))`: `--as descriptor` → **rc 3** (conjunct 1's permanent `multi` refusal, *"the device's descriptor parser accepts `sortedmulti` and not `multi`"*); `--as md1` → **rc 3** (*"md1 cannot carry this wallet as written … uses `/0/*`"*); `--as` omitted → **rc 3**, same neither-path text. Probe: `host_admits=false`, `md1_admits` fails at representability, `format_of = "bip380"` (the cascade branch is kept, as the plan says), `OutputDescriptor` → `err: unrecognized output descriptor format` so `device_admits=false`. Post-S2 `carriage()` (`gate.rs:223-245`) still returns `DescriptorRefusal` for it: `descriptor_carries=false` (conjunct 1), `md1_carries=false` (representability), so exit 3 survives the flip — the case-3 witness holds. **No existing row covers it:** the only `multi` rows are `neither/wsh-multi` and `gate/colliding-origin-multi`, and BOTH use `<0;1>/*` (measured), so neither is an uncarried-because-of-`/0/*` admission. |
| **A second `syswOffer` at the walletPolicy door is mechanically supported** | **CONFIRMED.** `syswOfferTitled` (`gui/sysw_session.go:210-223`) returns `("", false)` when `!ctx.sysw.has(want)`, so a second offer for a class the payload lacks is invisible; `newInputFlow` (`gui/gui.go:2747-2767`) is the shipped precedent and its comment states the rule: *"It is a SECOND offer rather than a widened first one because syswOffer takes one class."* `take` is gated on `s.compared`. Routing to `descriptorFlow` (`gui/gui.go:2727-2741`) is coherent with its return semantics — it is `func(...)` with no return value and loops until engraved or backed out, so the caller `return`s after it. `walletPolicyFlow`'s own "a separate insertion path" warning (`gui/wallet_policy.go:35-38`) is about a card joining the **gathered set**; a Descriptor record routed to `descriptorFlow` never joins it, so the warning is not violated. §11 item 6 is satisfiable: `DescriptorScreen.Draw` renders Title/Type/Script and Button 2 reaches `descriptorAddressFlow`. |
| **The `panic:encode` containment claim holds** | **CONFIRMED.** Measured: `OutputDescriptor("Name: my wallet")` returns `type=SortedMulti thr=0 keys=0 script=Unknown title="my wallet"`, and §4.7's conjunct 2 (`crates/me-cli/src/descriptor/admit.rs:129-141`) refuses `threshold < 1` for any `multi` form. A faithful conjunct port therefore refuses it before any screen can `Encode()` it. |
| **The parse-panic fix is genuine Rust-convergence** | **CONFIRMED.** The host refuses the same input cleanly: `bluewallet/short-fingerprint` → `me … --as descriptor` exits **3** with *"cosigner line `ab: xpub…` — a master fingerprint is exactly 8 hex characters (4 bytes)."*, and the single-line `ab: xpub…` variant exits 3 with the same message. The Go panic reproduces (`PANIC: index out of range [3] with length 1`). No fork test pins the panic: the only references are the seam test's `device_probe` bookkeeping (`nonstandard/descriptor_seam_test.go:113-166`), which the plan already schedules. (The plan cites the **wrong line** for the fix — see I2.) |
| **P1.3's `--expect` widening is safe** | **CONFIRMED.** `Kind::Descriptor => card_hrp(record) == Some('d')` (`crates/me-cli/src/sysw/expect.rs:112`) and the module doc's "must not resolve through `Class`" warning (lines 20-23) is about `MdMk` covering *both* `d` and `k` cards. Adding `|| classify_with(record, adm) == Class::Descriptor` cannot confuse a `d` card with a `k` card, because `Class::Descriptor` is a BIP-380 string and never a bech32 card. The doc's `address` paragraph (25-29) is the half that goes false and the plan schedules it. |
| **§4.6's whitespace rows do NOT break the derived rule** | **CONFIRMED — and this is the fold getting something right that looked wrong.** `whitespace/leading-space-bip380` is single-line with `host_admits=true` and `device_admits=false`, so the derived rule looked unsatisfiable on the Go side. It is not: `classifyConstellation` (`sysw/classify.go:34-38`) does `record = strings.TrimSpace(record)` **before** any arm, and P3.1 puts the descriptor arm inside it, last. Measured: `OutputDescriptor(TrimSpace(" wsh(…)"))` → **ok**. §7 requirement 4's "the input-level form is wrong twice over" objection is about `device_admits` (the scan door), not about `sysw.Classify`, which trims — so it does not transfer to this rule. |
| **The Rust half of the derived rule is satisfiable** | **CONFIRMED.** All **58** single-line rows classify `Unknown` today (measured, per row) — including every `gate/record-*` row, because `text:`/`pass:` bodies must be lowercase hex (`crates/me-cli/src/sysw/record.rs:135-142`) and `text: my wallet (2 of 3)` is not. So `classify(input) == Descriptor iff host_admits, else Unknown` holds by construction once the arm delegates to `host_admits`. Clause 2 also holds: all **19** `host_admits: true` rows' canonicals are accepted by `OutputDescriptor` (measured), and `host_admits(canonical) == host_admits(input)` on every row. |
| **M2's choice-block defect is real and the fix is stated correctly** | **CONFIRMED.** `gate.rs:566-570` returns `"      --as descriptor"` unpadded with `\n` after it in the format string, while `md1_head` is `"      --as md1          "` with its description inline; §5.1's NORMATIVE block (`design/SPEC_descriptor_input.md:792`) has `      --as descriptor   the SCANNABLE plate…` inline. `grep SCANNABLE crates/me-cli/tests/` is still 0 hits. |
| **Staleness** | `scripts/plan-staleness-check.sh` vs mnemonic-engrave `4646fa2`: **25 unchanged, 0 DRIFTED**. Vs the fork at `a5e29b4`: **22 unchanged, 0 DRIFTED**. The script states its own gap — *"whether the citation was ever RIGHT"* — which is where I2 below lives. |
| **P4.1's skip is not contradicted anywhere** | **CONFIRMED.** §5.5's plate cell (`SPEC_descriptor_input.md:1314`) currently says *"2 strings … = **TWO plates** (one plate per STRING)"*, which stays TRUE if P4.2 is skipped, so the spec edit P4.2 schedules is genuinely conditional. F-423's owning phase ("with S2's firmware build") and P5.1's reconciliation both admit a measured-no-gain close. No section requires P4.2 to ship. (One communication gap — M4.) |

---

## CRITICAL

### C1 — P3.1's arm is `§4.7 only`, but §5.2's predicate is `§4's cascade AND §4.7`; `promotion/15` is the counterexample, and it makes P3.3 a gate that cannot be met

r1's C3 measured 17 single-line rows where `nonstandard.OutputDescriptor` answers
TRUE and the host refuses. The fold closes it by defining the arm as:

> the arm = parse via `nonstandard.OutputDescriptor` + a port of §4.7's conjuncts
> over the parsed descriptor

and enumerates the divergence as *"anyone-can-spend `sortedmulti(0,…)`, `k > n`,
21 keys, mixed-network, hardened use-sites, conjunct-8 key-identity failures"*.

**That enumeration has 16 members. r1 measured 17.** The missing one is
`promotion/15-bare-tpub-host-refused`, and it is not a §4.7 refusal at all.

**Measured, per row.** The 17 rows split cleanly by `format`:

| rows | `format` | refused by |
| --- | --- | --- |
| 14 `narrowed/*` + `gate/colliding-origin-sortedmulti` + `gate/duplicate-key-same-use-site` | `bip380` — the cascade **succeeded** | §4.7 conjuncts 1, 2, 3, 5, 7, 8 — a conjunct port catches all 16 |
| **`promotion/15-bare-tpub-host-refused`** | **`none` — the cascade FAILED** | **§4.5, not §4.7** |

The vector file's own `source` annotation for that row:

> `SPEC_descriptor_input.md S4.5 NORMATIVE ruling (host refuses tpub promotion entirely)`

and the code agrees — `crates/me-cli/src/descriptor/cascade.rs:529`:
*"§4.5's ruling: `me` refuses `tpub` promotion entirely."*

**Every §4.7 conjunct PASSES on this input, so the arm as specified admits it.**
Measured with the Go probe, after `TrimSpace` exactly as `classifyConstellation`
does: `OutputDescriptor(tpubDCXMbAzeg2Tp…)` → `type=Singlesig thr=1 keys=1
script=Legacy (P2PKH)`. Walking the ported conjuncts against that value:

- conjunct 1 shape (`admit.rs:97-118`): `(None, single=true, _)` → **Ok** (`pkh(KEY)` is one of the seven forms);
- conjuncts 2 and 3 (`admit.rs:129-166`): return `Ok` immediately when `d.multi.is_none()`;
- conjunct 4 versions (`admit.rs:172`): `KeyVersion::admitted()` (`cascade.rs:95-100`) is `Xpub | **Tpub** | Zpub | YpubCap | ZpubCap` — **`Tpub` is admitted**;
- conjuncts 5, 6, 7, 8: a single bare promoted key with a synthetic origin passes all four — `promotion/01-bare-xpub` is `host_admits=true` and differs from this row **only in the version bytes**.

So a faithful §4.7 port answers `ClassDescriptor` on a bare `tpub`.

**Failure scenario 1 — the gate cannot be met.** P3.3 asserts, exhaustively and
per row: *"for every single-line input, `sysw.Classify(input) == ClassDescriptor`
iff `host_admits`, else `ClassUnknown`"*. For `promotion/15`, `host_admits` is
`false`, so the test demands `ClassUnknown`; the arm as specified returns
`ClassDescriptor`. `TestDescriptorSeamSyswClass` reds on its first un-skipped
run and cannot be made green without changing P3.1's arm definition. The plan's
own claim that *"a Go/Rust divergence anywhere in the file goes red on this
test"* is true — and the first thing it reds on is the plan's own arm.

**Failure scenario 2 — and this is the funds-shaped one.** If the implementer
resolves the red the other way (relax the rule to match the arm, or exclude the
row), the shipped device classifies a bare testnet `tpub` as `ClassDescriptor`.
`gui/sysw_admit.go:37,39,45` already admit `ClassDescriptor` to `progBundle`,
`progMultisig` and `progWalletPolicy`, so P3.2's new offer hands it to
`descriptorFlow` → `DescriptorScreen` → `validateDescriptor` → a plate, for a
single-sig **testnet** wallet the host tool refuses outright by a NORMATIVE
ruling. That is r1's C3 class, unclosed on exactly one row.

**Failure scenario 3 — the spec inherits the wrong predicate.** P2.7 schedules a
§5.2 amendment that would write this composition into the spec:

> it becomes *"parses via `nonstandard.OutputDescriptor` AND enforces §4.7's conjuncts"*

But §5.2's stated predicate (`SPEC_descriptor_input.md:1000-1003`, and
`crates/me-cli/src/descriptor/admit.rs:409-411`, verbatim) is:

> A record is `ClassDescriptor` iff it parses under **§4's cascade** and matches
> §4.7's grammar — the seven forms; conjunct 1's md1-path widening does not apply here.

`host_admits` implements exactly that — `cascade::cascade(normalise(input))`
**then** `admit(&d, Path::Descriptor)` (`admit.rs:418-423`) — and §4.5's tpub
ruling lives in the first half. The proposed amendment would replace a correct
sentence with one that is narrower than the code on the primary side, which is
the opposite of what the Rust-primary rule requires.

**What the fold has to decide.** The arm is a port of the **cascade's admission
narrowings** as well as §4.7's — at minimum §4.5's promotion table (which
versions promote, and to what). That is a larger P3.1 than the fold describes,
and it is a plan-level ruling rather than an implementer's judgement call,
because P3.3's assertion, P2.7's §5.2 amendment and §9 item 2's claim all
inherit it. The alternative — state the asymmetry, keep the arm at §4.7, and
scope P3.3's rule to the rows the arm can answer — is also available, but it
must be *written*, because "parity is structural" is the sentence the plan
currently relies on and it is false by one row.

---

## IMPORTANT

### I1 — P2.7's forced-spec-amendment list omits §7 and §4.2, which the fold's OWN new decisions falsify

r1's I3 was *"S2 forces spec amendments no phase owns"*, and the fold's answer is
P2.7, which enumerates §6's table, §11 item 5, §11 item 1, §5.2's Go-arm
sentence, §5.5's row and §8's "S2 is parked" sentence. Good. But three of the
fold's **new** decisions falsify spec text that is on neither P2.7's list nor
P0.1's inventory.

**(a) Retiring the `sysw_class` column falsifies §7's own definition of it and
§11 item 1's mechanism.** §7 (`SPEC_descriptor_input.md:1526-1530`), verbatim:

> **`device_admits` means `nonstandard.OutputDescriptor` accepts the INPUT** —
> the scan door, nothing else. The classifier is a different predicate with a
> different answer (§2.3), so it gets its own optional column, **`sysw_class`**,
> asserted by the Go test only on rows that carry it, against `sysw.Classify`
> once §5.2's arm lands. One column carrying both meanings is how §7 and §11
> contradicted each other in round 0.

and §11 item 1 (`:1917-1920`), verbatim:

> the device's `sysw.Classify` — **exercised by §7's Go test through the
> `sysw_class` column** — classifies that record `Descriptor`.

Invariant 1 deletes the column and P3.3 replaces it with a derived rule. §7's
paragraph then describes a column that does not exist, and §11 item 1 names a
mechanism that does not exist. P2.7 lists §11 item 1 only for *"names P2.5's
surface"* — the `sysw_class` half is unowned, and §7 is not listed at all.

**(b) P3.1's convergence fix falsifies §4.2 defect 4.** `SPEC_descriptor_input.md:382-391`,
verbatim:

> **A fingerprint shorter than 4 bytes PANICS the Go parser.** … and §7 marks
> these rows `device_probe: "panic:parse"` so the Go test never feeds one to the
> parser.

After P3.1 the Go parser does not panic, and after invariant 1 the row carries no
`device_probe`, so both sentences are false of the shipped tree. §7's
`device_probe` bullet (`:1543-1547`) and its `panic:parse`-skips-requirement-5
clause describe a class the plan empties.

**Failure scenario.** The plan's own doctrine — *"a diff falsifies text it never
touches"*, and *"the propagation sweep runs over the SPEC too"* — is stated at
P2.7 and at the P2 gate. But the P2 gate's sweep is scoped to *"the S3-parked
phrasings"* per P0's inventory, and neither §7's `sysw_class` paragraph nor §4.2
defect 4 contains an S3-parked phrasing: they share no token with what breaks
them. So the sweep as scoped cannot find them, and the same class of finding that
produced r1's I3 ships in the fold that closed I3. Concretely: after S2 merges,
`SPEC_descriptor_input.md` tells a reader that a `sysw_class` column asserts the
device classifier and that the Go parser panics on a short fingerprint — both
false, both load-bearing for anyone extending the seam.

### I2 — P3.1 points the parse-panic fix at `nonstandard/parse.go:158`; the panic is at `:149` and the check to change is at `:140`

The fold does two separate things with the number 158, and they are not the same
statement.

**Correct:** invariant 1's *"F-428's citation fixes (the stale `:151` cite → the
measured `nonstandard/parse.go:158` …)"*. F-428 is about the **key-count
error**, and measured, `nonstandard/parse.go:158` is exactly
`return nil, fmt.Errorf("bluewallet: expected %d keys, but got %d", nkeys, len(desc.Keys))`.
That fix is right.

**Wrong:** P3.1 then reuses the same number for a different defect —

> the short-fingerprint parse panic (`nonstandard/parse.go:158`, §4.2 defect 4)
> is fixed as RUST-CONVERGENCE … with a bounds check and a clean error

Measured, at `a5e29b4`:

```
nonstandard/parse.go:136        fp, err := hex.DecodeString(key)
nonstandard/parse.go:140        if len(fp) > 4 {            <- the guard to change
nonstandard/parse.go:149        MasterFingerprint: binary.BigEndian.Uint32(fp),   <- the panic
nonstandard/parse.go:158        return nil, fmt.Errorf("bluewallet: expected %d keys…")
```

The spec has it right and contradicts the plan —
`SPEC_descriptor_input.md:384`: *"`binary.BigEndian.Uint32(fp[:])`
(`nonstandard/parse.go:136–149`)"*.

**Failure scenario.** Invariant 1 makes the harness guard's removal *conditional
on this fix*: *"P3.1's convergence fix makes the Go parser error cleanly, so the
row gains its measured `device_admits` boolean and the 'must NOT feed' harness
rule for it is dropped from the header."* An implementer who edits at `:158`
changes the key-count error, leaves `len(fp) > 4` at `:140` untouched, and
reports the task done. At P3.3 the un-skipped seam test feeds the input to
`OutputDescriptor` with the guard already dropped — and per the seam test's own
note (`SPEC_descriptor_input.md:1547`, *"A panic would crash the suite rather
than fail it, a false-signal shape"*), the fork test **binary crashes** rather
than reporting a failure. `scripts/plan-cite-check.sh` cannot catch this (line
158 exists) and `plan-staleness-check.sh` reports 0 DRIFTED (the file has not
moved) — this is precisely the "whether the citation was ever RIGHT" gap both
tools print.

Fix: cite `nonstandard/parse.go:140` (the guard) and `:149` (the panic), and keep
`:158` where F-428 put it.

---

## MINOR

**M1 — P3.2's sixth checklist touch point is not self-enforcing at the site the
fold chose.** The plan says the checklist's sixth item is *"registration in
`gui/sysw_admit_oracle_test.go`'s `syswConsumers` table — the oracle fails until
the call site is named."* Measured: the oracle indexes by `file + ":" + fn`
(`gui/sysw_admit_oracle_test.go:86-89`), and
`{"wallet_policy.go", "walletPolicyFlow", …}` is **already** in the table
(`:64-68`). A second `syswOffer(ctx, th, sysw.ClassDescriptor, …)` placed inside
`walletPolicyFlow` — which is what P3.2 describes — is already mapped, and
`admits(progWalletPolicy, ClassDescriptor)` is already true
(`gui/sysw_admit.go:45`), so `TestEverySyswConsumptionSiteNamesAnAdmittedClass`
stays green with nothing added. What *does* go stale, unguarded, is that entry's
`why` string: *"ClassMDMK only — this program never derives from a secret, so
progWalletPolicy admits no seed class at all."* Say in P3.2 that the touch point
is a `why`-string update rather than a registration, or the implementer will
correctly observe the oracle is already green and skip it.

**M2 — invariant 1's count-guard citation excludes the guards the new row
actually moves.** The plan cites *"the count guards
(`nonstandard/descriptor_seam_test.go:74-77`)"*. Measured, those four lines are
`wantSyswClass` (74), `wantPanicParse` (75), `wantPanicEncode` (76),
`wantHostWider` (77) — the right range for the column retirements, and the wrong
range for the new witness row, which moves `wantRows` (**:66**),
`wantDeviceFalse` (**:68**) and the `deviceTrue/deviceFalse` assertion at
`:157-159`. The engrave half's equivalents are unnamed entirely: `MANIFEST`,
`TAG_SLOTS = 88`, `ROW_FLOOR = 71`, `SECOND_TAGGED`, `THIRD_TAGGED`
(`crates/me-cli/tests/descriptor_seam.rs:50-69`) all move with a 72nd row and a
new `covers` tag set. P2.6 says only *"engrave-side seam assertions updated in
the same commit"*. Both suites red loudly, so this costs a cycle rather than
correctness — but the plan's own rule is to name what moves.

**M3 — P2.6 writes a device-column value for a device behaviour that does not
exist yet.** Invariant 1's single regeneration lands at P2.6 (engrave half, in
P2) and gives the `bluewallet/short-fingerprint` row *"its measured
`device_admits` boolean"*. But the fix that makes that boolean measurable is
P3.1, a phase later. At P2.6 the value can only be a prediction, and nothing
gates it: the engrave suite asserts host columns only, and the fork suite does
not see the new bytes until P3.3. Say explicitly that P2.6's regeneration takes
this row's value from a **run** of the fixed Go parser (i.e. land P3.1's fix in
the fork worktree first, even though its commit lands in P3), or move the row's
`device_admits` into P3.3's half of the regeneration.

**M4 — a measured-no-gain close on F-423 never reaches the operator who filed
it.** F-423 carries an operator directive quoted verbatim: *"1 plate per string
is something to be addressed, it's wasteful."* P4.1 authorises closing it as
`measured-no-gain` on the plan's own measurement, and P5.4's handover mentions
F-423 only *"IF P4.2 shipped"*. So the one outcome the operator would want to see
— "we measured, and at the shipped font it will not pack" — is written into
FOLLOWUPS and into nothing the handover reads. Add the measurement's number to
P5.4's handover regardless of the outcome; it is one line.

---

## NIT

**N1** — P0.1's inventory names `crates/me-cli/src/descriptor/as_flag.rs:133` as a
comment-only mention (correctly, and it is right to say so), but
`crates/me-cli/src/main.rs:350` is a second comment-only mention of
`DESCRIPTOR_PATH_SHIPPED` and is not listed. Measured, `grep -rn
'DESCRIPTOR_PATH_SHIPPED' crates/` returns exactly seven hits: the five
behavioural sites the plan names, plus those two comments. Naming both keeps the
inventory a grep rather than a reading.

**N2** — P1.0 says *"`consult` runs BEFORE `admit_check`"* and does not fix its
position relative to `--expect`, which today sits between them
(`crates/me-cli/src/main.rs:1474-1490` vs `:1504`). The two readings differ:
`me sysw pack --in <descriptor> --expect mnemonic` exits **4** if `consult` goes
immediately before `admit_check`, and **2** if it goes before `--expect`. The
first is behaviour-preserving and is presumably intended; one clause fixes it.

**N3** — r1's M4 said *"P4.1 names an analytic measurement the code does not
offer"*, and the fold replaced the analytic phrasing with a mandated trial-fit
program. The trial fit is the better instrument and the change is right, but the
premise was not quite true: `backup.CharsPerLine` and `backup.LinesPerPlate`
(`backup/backup.go:88-97`) are exactly an analytic fit, and
`backup.FontSizes` (`:83`) is a shipped descending ladder for free-text plates.
Worth a sentence in P4.1 so the implementer computes an upper bound analytically
and confirms it by trial, rather than trial-fitting blind.

---

## Fold vs r1's findings

| r1 | resolved? | evidence |
| --- | --- | --- |
| **C1** — the arm disables §5.1's gate on the `--as`-omitted path | **RESOLVED** | P1.0 restructures to consult-before-`admit_check`, lands as its own commit, and pins `item_5_the_five_case_matrix` at every P1/P2 boundary. Measured: pre-arm behaviour-preserving (zero gate/classify collisions over 71 vector rows + synthetic records); the named witness does red under an arm without it. |
| **C2** — P1.2's 4-vs-67 split is unsatisfiable | **RESOLVED (Rust half); the Go half is C1 of this round** | The derived rule is satisfiable on the Rust side by construction — measured, all 58 single-line rows are `Unknown` today and all 19 `host_admits` canonicals hold. On the Go side one row (`promotion/15`) cannot pass under P3.1's arm. |
| **C3** — the Go arm is 17 rows wider than the Rust primary | **PARTIALLY — 16 of 17** | P3.1 now ports §4.7's conjuncts, which catches every `format: bip380` divergence. `promotion/15-bare-tpub-host-refused` is `format: none` — a §4.5 cascade refusal — and the fold's own enumeration drops it. **This round's C1.** |
| **C4** — `me sysw show` has no per-record surface | **RESOLVED** | P2.5 builds it additively; measured implementable — `identify::block` takes `Option<Form>`, `print_mdmk_confirmation` already holds the public record strings, `Class::Descriptor` is not secret. |
| **I1** — `--as descriptor --expect descriptor` is a guaranteed false refusal | **RESOLVED** | P1.3 re-ruled: `Kind::Descriptor` widens to `card_hrp=='d' \|\| Class::Descriptor` in the same commit, with the description and module doc updated and three named tests. Verified the widening cannot reintroduce the `d`-vs-`k` confusion the module doc warns about. |
| **I2** — the flip inventory misses three `DESCRIPTOR_PATH_SHIPPED` consumers | **RESOLVED** | P0.1 now lists `gate.rs:42/223/273/566` and `main.rs:365`, carries BOTH "two variants" collections named separately, and states the exit-code flip explicitly. `grep -rn` measured: exactly those five behavioural sites. (One comment-only site unlisted → N1.) |
| **I3** — retiring `WindowNotInBuild` forces spec amendments no phase owns; P2.4's premise is false | **PARTIALLY** | P2.4's premise is corrected verbatim ("S2 SUBTRACTS one … invent nothing") — resolved. P2.7 is created and owns §6/§11-5/§11-1/§5.2/§5.5/§8 — but the fold's own new decisions re-open the same gap for §7 and §4.2. **This round's I1.** |
| **I4** — §11 item 5's case 3 loses its only witness; invariant 1 forbids the replacement | **RESOLVED** | Invariant 1 rewritten to allow exactly one regeneration and to carry the new witness. Measured: `wsh(multi(2,…/0/*))` exits 3 under all three `--as` states, `host_admits=false`, `md1` fails at representability, `format=bip380`, `device_admits=false`, and no existing row covers it (both shipped `multi` rows use `<0;1>/*`). |
| **I5** — P3.2 names no consumer; P5.4 over-claims §9 item 2 | **RESOLVED** | Consumer named (`progWalletPolicy`/`walletPolicyFlow`), the other two cells stay declared+inert with a filed follow-up, P3.5 amends §9 item 2, and P5.4 hands the operator the ONE built cell. Mechanics verified: `syswOffer` is single-class and a second offer is the shipped `newInputFlow` pattern; the oracle admits the cell already. (One claim about the oracle → M1.) |
| **I6** — two crash paths on the payload-load path | **RESOLVED** | Both named and contained. Measured: the host refuses the short-fingerprint input cleanly at exit 3 (so the Go bounds fix is genuine convergence), and `Name: my wallet` parses to `thr=0 keys=0`, which conjunct 2 refuses. (The fix's cited line is wrong → I2.) |
| **M1** — the recon checklist is five, not six | **RESOLVED** | P3.2 states the six touch points. (Sixth is not self-enforcing at the chosen site → M1 of this round.) |
| **M2** — the shipped choice-block branch renders misaligned | **RESOLVED** | P2.2 adds the padding fix AND the verbatim block test that does not exist today. Both halves of the defect confirmed against `gate.rs:566-570` and the spec's normative block at `:792`. |
| **M3** — P3's gate omits the TinyGo device build | **RESOLVED** | The exact CI command is now in P3's gate, with the "an unrun gate is a hypothesis" reason. |
| **M4** — P4.1 names an analytic measurement the code does not offer | **RESOLVED** | P4.1 rewritten around the trial fit (`EngraveText` → `toPlate`), the single plate size, the fixed font, and a `FontSize`-reduction prohibition. (Premise slightly overstated → N3.) |
| **M5** — P4.2 does not state the packing boundary | **RESOLVED** | "packing WITHIN a card only", with `bundlePlate`'s card-scoped fields named and the ms1-marking half correctly capped by the 2026-08-27 severity ruling. |
| **M6** — the clippy gate is the note F-430 says not to write | **RESOLVED** | P0.3 commits `scripts/lint-gate.sh` as a command, every later gate refers to it, and P5.1 reconciles F-430 as resolved-by-script. |
| **M7** — P3.3 does not say which string `sysw_class` is asserted over | **RESOLVED** | The ambiguity dissolves: both bases are asserted separately, over disjoint populations. Measured that both are satisfiable on the Rust side, and that the §4.6 rows do not break the input basis on the Go side (`classifyConstellation` trims first). |
| **N1** — "host-side fixed point" is neither | **RESOLVED** | P2.3 renames it "record-classification check" and names §7 requirement 4's real fixed point as already asserted. |
| **N2** — the identification block is already path-independent | **RESOLVED** | P2.1 records it verified with the call site, no work required. |
| **N3** — §5.3(b)'s label warning is new on the JSON exemplar's path | **RESOLVED** | P2.3 names the line as expected output, with the §5.5 justification. |

---

## What a fold has to decide, not just fix

One decision, and it is the same shape as r1's: **how wide is the device's
descriptor predicate allowed to be?** The fold answered "§4.7's conjuncts" and
that closes 16 of 17 rows; the 17th says the answer is "§4's cascade *and*
§4.7", which is what §5.2 already says and what `host_admits` already
implements. Ruling it also settles P2.7's §5.2 amendment, which currently
proposes to narrow a correct spec sentence.

The two Importants are cheap: add §7 and §4.2 to P2.7's list, and repoint the
parse-panic citation from `:158` to `:140`/`:149`.
