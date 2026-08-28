# R0 review — `SPEC_descriptor_input.md`, round 8 (proportional re-review of the r7 fold)

**Artifact:** `design/SPEC_descriptor_input.md` at `64af075` (1475 lines).
**Scope, as briefed:** (1) did the fold close each of r7's six findings; (2) did the six edits
introduce defects — **only in what changed**, at the pressure points named in the brief (the
49-row floor arithmetic, the "exactly two rows qualify" claim, the `#w47tv00x` citation, the new
remedy's three named forms against §4.7 conjunct 1's admitted twins, and consistency with the
surrounding sections). Not a fresh audit. Every r1–r7 measured result, the citation gate,
F-417/F-418 and all prior dispositions were taken as settled and were not re-derived.

**Reviewer:** independent agent, opus tier. **Read-only** — nothing in `mnemonic-engrave`,
`descriptor-mnemonic` or `seedhammer` was written to, and nothing was committed or pushed.
md1 probes: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (`md 0.13.0`).
Go probes: scratch module `…/scratchpad/goprobe8` with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer` (worktree at `0b656d7`, the same
revision r7 measured), built with
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`.

---

## Counts — NEW findings only

**0 Critical / 0 Important / 1 Minor / 1 Nit**

**Disposition of r7: 6 FIXED, 0 PARTIAL, 0 NOT FIXED.**

**The correctness lens CLOSES.** Both of r7's Importants were single-string / single-cell
defects in text the r6 fold had authored, and both are now closed by measurement rather than by
reading: the read-back pin fails the mutant it exists to catch, and the wrapped-multi remedy
names a wrapper change whose three forms all encode through md1. The one Minor and the one Nit
below are **not** in the six edits — the Minor is a pre-existing §4.2/§6 gap that seven rounds
have not surfaced, and the Nit is a pointer the fold could have updated. Neither blocks.

---

## Disposition table — r7's six findings

| # | r7 finding | verdict | what re-tracing it shows |
| --- | --- | :-: | --- |
| NEW-I1 | the read-back pin `"multi("` passes on the `sortedmulti` mutant | **FIXED** | The pin is now `md_descriptor_contains: "wsh(multi("` (line 1236), with the reason recorded at the pin. **Re-measured both ways, and on both read-back surfaces.** Reproduced r7's defect first: old pin `"multi("` CONTAINS in both read-backs (PASSES on the mutant). New pin `"wsh(multi("`: CONTAINS in the `multi` read-back (`#656zkmsn`, PASSES), **absent** from the `sortedmulti` read-back (`#l9ucx0pn`, FAILS). Run against the `md descriptor --template` read-back *and* against the full md1 round trip (`md encode` → phrases → `md descriptor <phrases>`), because the assertion is stated over "the round trip's read-back" — the pin discriminates on both. Checksums reproduce r7's exactly. |
| NEW-I2 + r6-M4 (2nd half) | the `multi` remedy was self-referential, omitted the wrapper change, and left the device parenthetical transposing and false | **FIXED** | The head sentence (1042–1047) now names **both** non-transposing parts — *"NEITHER the remedy's `sortedmulti` forms NOR the device-measurement parenthetical transposes"* — closing r6-M4's second half. The row cell (1075) substitutes a complete message: *"a multisig policy cannot live inside a single-key script on EITHER path. Change the wrapper — `wsh(multi(…))`, `sh(multi(…))` or `sh(wsh(multi(…)))` — and use `--as md1`, which carries those forms."* Journey walked end to end below — every message true, every named next action executable, measured. |
| NEW-M1 | the double-count permission was unconstrained and no row total was pinned | **FIXED** | 1257–1263: a second tag is permitted **only** where the input genuinely discharges both bullets, exactly two rows named; `covers` entries distinct within a row; a **49 physical row** floor. Arithmetic recomputed independently — see below. The floor is both **correct** (satisfiable by an honest file) and **operative** (r7's retag construction now fails it at 48). |
| NEW-M2 | the zero-key `panic:encode` marker was true of one spelling only | **FIXED** | 1227–1232 pins the row to §4.2 defect 2's exact `Name: only` spelling and states that a zero-key file *with* `Format:` encodes cleanly. Re-measured: `Name: only\n` → ACCEPT `keys=0 script=Unknown` → **ENCODE PANIC: unknown script**; zero keys + `Format: P2WSH` → ACCEPT `keys=0 script=Segwit (P2WSH)` → **ENCODE ok**. `§4.2 defect 2` resolves to line 335, which is the `Name: only\n` row — citation correct. |
| NEW-N1 | §11 item 3's *"the field"* was singular while two bullets claimed it | **FIXED** | Item 3 (1465–1466) now reads *"the row floor met, and both `covers` and `md1_admits` present on every row"*. Both fields named. |
| NEW-N2 | §8 named only §11 item 6 as S2-bound | **FIXED** | §8 (1336–1339) now reads *"§11 items 1, 4 (its `--as descriptor` rows) and 6 bind S2's ship"*. Checked against the items themselves: item 1 is *"**S2's item** (F-418)"* (whole item), item 4 scopes only its `--as descriptor` rows to S2, item 6 binds *"**S2's ship only**"*. §8's parenthetical narrows item 4 correctly, and its conclusion (*"S1 and S3 can plan, build, demonstrate and ship entirely at the desk"*) still holds. |

---

## The 49-row floor, recomputed

Not transcribed from the fold — summed from the manifest table itself.

| tag | min |
| --- | :-: |
| `formats-happy` | 4 |
| `promotion-near-miss` | 15 |
| `narrowed-4.7` | 14 |
| `accepted-extreme` | 1 |
| `narrowed-4.2` | 5 |
| `neither` | 3 |
| `whitespace` | 3 |
| `md1-splits` | 6 |
| **sum** | **51** |

`4+15+14+1+5+3+3+6 = 51` tag-slots. Two permitted overlapping rows consume 4 slots across 2
rows, leaving 47 slots on 47 single-tag rows: **47 + 2 = 49 physical rows.** The stated floor
is arithmetically correct.

**Is the floor SATISFIABLE — i.e. is 49 too high?** This is the dangerous direction: a floor
above what an honest complete file can produce would fail a correct file. It is not too high.
Both named overlaps are genuine, and the second is effectively forced:

- **`xpub…\n`** is literally row 14 of §4.5's fifteen-row table *and* §4.6's trailing-`\n`
  shape — it genuinely discharges `promotion-near-miss` + `whitespace`.
- **bare `xpub`** is row 1 of §4.5's table (ACCEPT → `pkh`) *and* the `formats-happy`
  promoted-key row. Every promotable input the spec exhibits sits in §4.5's table, so a
  promoted-key happy path is almost unavoidably also a table row. r7 hedged this as *"50 rows
  (49 if the promoted-key row is authored as §4.5's own bare-`xpub` row)"*; the fold resolved
  the hedge in the direction that keeps the floor satisfiable. **Picking 50 would have been the
  defect** — an honest 49-row file would have failed it.

**Is a third overlap forced anywhere?** Cross-checked the eight bullets' enumerations for a
shared input and found none beyond the two named. The nearest candidates are distinct inputs:
§4.5's `xpub…/<0;1>/*` (a wildcard multipath, md1-representable) is not `md1-splits`'
`<0;1>`-**without**-wildcard row; the `neither` bullet's `wsh(multi)` and full-origin `ypub` are
absent from `narrowed-4.7` and from §4.5's table; the `narrowed-4.2` no-`Format:` row (2 keys)
and zero-key row (0 keys) are distinct inputs even though both lack `Format:`. So no bullet is
starved by the two-overlap cap.

**Is the floor OPERATIVE?** Re-ran r7's surviving construction. Drop the childless+`<0;1>/*`
mixed row and retag one `narrowed-4.7` row as `["narrowed-4.7","md1-splits"]`: `md1-splits` = 6
✓, `narrowed-4.7` = 14 ✓, no unknown tags ✓ — but physical rows = **48 < 49**, so the floor
**FAILS**. The duplicate-tag variant (`["md1-splits","md1-splits"]`) is blocked by the
distinct-entries rule. Both mechanisms the fold names as closed are closed.

*Residual, not a finding:* a determined author could drop a row, retag, **and** add a padding
row carrying an existing tag to restore the count to 49 — minima are floors, so an over-count
is legal. That is a third mechanism the fold does not claim to close (its sentence names
retagging and duplicate tags, and both are genuinely closed), it needs two deliberate
falsifications rather than one, and it is the irreducible residue of any hand-authored coverage
annotation. Stated so the next round does not re-derive it as new.

---

## The `wpkh(multi(2,…))` operator journey, walked end to end

The input is `wpkh(multi(2,[dc567276/48h/0h/0h/2h]K1/<0;1>/*,[f245ae38/48h/0h/0h/2h]K2/<0;1>/*))`
and the operator types `me sysw pack --as md1 --in <that>`.

**Step 1 — what refuses, and is it the right refusal?** §4.7 conjunct 1 admits, on the md1 path
only, exactly three `multi` twins: `wsh(multi(k,…))`, `sh(multi(k,…))`, `sh(wsh(multi(k,…)))`
(read at lines 636–639). `wpkh(multi(…))` is not among them → conjunct 1 refuses. Correct.

**Step 2 — what does the operator read?** Per the head sentence's exemption, the `multi` input
gets the substituted cell, and the `sortedmulti` device parenthetical and the `sortedmulti`
forms sentence both do **not** transpose. So the message is exactly:

> *"a multisig policy cannot live inside a single-key script on EITHER path. Change the wrapper
> — `wsh(multi(…))`, `sh(multi(…))` or `sh(wsh(multi(…)))` — and use `--as md1`, which carries
> those forms."*

**Every clause checked:**

- *"on EITHER path"* — TRUE. Under `--as descriptor` conjunct 1's seven forms exclude
  `wpkh(multi(…))`; under `--as md1` the three twins exclude it.
- **The three named forms are exactly §4.7 conjunct 1's three md1-path twins** — compared
  literally against lines 636–639. No fourth form named, none omitted.
- *"use `--as md1`, which carries those forms"* — **executable, measured.** All three encode
  clean through md1 with the fixture keys:

  ```
  wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))        ENCODE OK
  sh(multi(2,@0/<0;1>/*,@1/<0;1>/*))         ENCODE OK
  sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))    ENCODE OK
  ```

- **No longer self-referential.** r7's defect was that the remedy's only instruction was the
  flag the operator had just typed. The mandatory action is now the *wrapper change*; `--as md1`
  appears as a conjunct of a real edit, not as the whole remedy. The operator who follows it
  lands on an accepting invocation rather than back where they started.
- **The false device claim never reaches a `multi` operator.** The row states the parenthetical
  does not apply to `multi` inputs, and re-measurement confirms why: all three single-key
  `multi` twins are REFUSE at the parse door, so `address.Receive` is never reached and the
  quoted `address: multisig script: …` error is never produced.

  ```
  wpkh(multi(2,…))                REFUSE: nonstandard: unrecognized output descriptor format
  pkh(multi(2,…))                 REFUSE: nonstandard: unrecognized output descriptor format
  sh(wpkh(multi(2,…)))            REFUSE: nonstandard: unrecognized output descriptor format
  CONTROL wpkh(sortedmulti(2,…))  ACCEPT script=Segwit (P2WPKH) keys=2, Encode ok #6cc6zuge
  ```

  The exemption's *"device REFUSE at PARSE (measured) and never reach address derivation"* is
  TRUE on all three, against a control that accepts.

The journey terminates on an executable action with no false statement on the path.

---

## Propagation check — the recurring failure mode in this file

Three of the six edits changed a value that appears in more than one place. Grepped each:

- **`md_descriptor_contains`** — four sites (1160 schema, 1236 the pin, 1279 definition, 1290
  assertion). Only 1236 carries a literal, and it is the new `"wsh(multi("`. **No stale
  `"multi("` literal survives anywhere in the file.**
- **The old `multi` remedy** (*"keep `multi` and use `--as md1, which carries it"*) — **gone**.
  The one surviving *"which carries it"* (line 924) is about `--as descriptor` and an offending
  key, unrelated and pre-existing.
- **Row counts** — `49` / `51` appear only at 1261–1262, with item 3's *"row floor met"* at
  1466 pointing at them. No competing total anywhere.

---

## NEW — Minor

**NEW-M1 — §4.2 normatively refuses a BlueWallet file with zero cosigner lines and says *"Each
refusal names its cause (§6)"*, but §6 has no zero-cosigner-lines row.** §6's table has 34 rows;
its BlueWallet refusals are no `Name:`, no `Format:`, `Policy: k of n` count mismatch, no origin
path, and a fingerprint that is not 8 hex — enumerated and grepped, none covers zero cosigner
lines. Consequence for the file the fold has now *named*: `Name: only\n` has no `Format:` header,
so the refusal that actually fires for it is the no-`Format:` one — *"…no `Format:` header, so
the script type is undefined. Add `Format: P2WSH`…"* — and an operator who follows that remedy
produces the zero-key-plus-`Format:` file, which is a **second** refusal with no text of its own
(and which the device encodes cleanly to `wsh(sortedmulti(0,))#w47tv00x`, measured). That is the
remedy-leads-to-another-refusal shape r5 and r7 both hit.

**Why Minor and why it does not hold this gate:** it is **not fold-introduced**. §4.2's normative
list and §6's table both predate this fold, which touched neither; the fold's pin only made an
existing gap easier to see by naming the exact file. It is out of this review's scope, seven
correctness rounds have not surfaced it, and it is precisely the shape §9 item 7's journey walk
is scheduled to generate. **Recommend filing it as a follow-up owned by the §6 journey walk**
rather than reopening the correctness lens for it.

---

## NEW — Nit

**NEW-N1 — the `covers` schema bullet still says *"a row may carry two"* without the manifest's
new restriction.** Line 1187–1189 reads *"which required-row bullets this row discharges; a row
may carry two"*, while the manifest (1257–1260) now permits a second tag **only** on two named
rows. Not a contradiction — the schema states a permission and routes counting to the manifest
(*"§11 item 3 counts these against the manifest below the bullets"*), and the manifest is the
NORMATIVE text — so no wrong outcome follows. But a reader who stops at the schema bullet gets
the pre-fold rule. One clause: *"…a row may carry two, under the manifest's restriction below."*

---

## Measurements taken this round

Everything below was RUN. Nothing was written to any repo.

```
md1 (descriptor-mnemonic/target/release/md 0.13.0), keys dc567276 / f245ae38,
--path m/48'/0'/0'/2'

  read-backs, via --template
    wsh(multi(2,…))        -> wsh(multi(2,[dc567276/48'/0'/0'/2']xpub661My…))#656zkmsn
    wsh(sortedmulti(2,…))  -> wsh(sortedmulti(2,[dc567276/48'/0'/0'/2']xpub661My…))#l9ucx0pn
    (both checksums reproduce r7's exactly)

  read-backs, via the FULL md1 round trip (md encode -> phrases -> md descriptor <phrases>)
    multi phrases (345 B)        -> wsh(multi(2,[dc567276/48'/0'/0'/2']xpub661My…
    sortedmulti phrases (345 B)  -> wsh(sortedmulti(2,[dc567276/48'/0'/0'/2']xpub661My…

  PIN TEST — old pin "multi("        (r7's NEW-I1 defect, reproduced)
    vs multi read-back        CONTAINS -> PASSES
    vs sortedmulti read-back  CONTAINS -> PASSES     <-- the defect r7 found
  PIN TEST — new pin "wsh(multi("    (the fold's fix)
    vs multi read-back        CONTAINS -> PASSES
    vs sortedmulti read-back  absent   -> FAILS      <-- the gate now fails the mutant
    (identical result against the --template read-back and the round-trip read-back)

  the remedy's three named forms, encoded through md1
    wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))       ENCODE OK
    sh(multi(2,@0/<0;1>/*,@1/<0;1>/*))        ENCODE OK
    sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))   ENCODE OK

Go (nonstandard.OutputDescriptor / bip380.Descriptor.Encode), fork 0b656d7, go 1.26.3

  NEW-M2, zero-key spellings
    "Name: only\n"                    ACCEPT keys=0 script=Unknown       -> ENCODE PANIC: unknown script
    zero keys + Format: P2WSH         ACCEPT keys=0 script=Segwit(P2WSH) -> ENCODE ok: wsh(sortedmulti(0,))#w47tv00x
    no Format:, two keys              ACCEPT keys=2 script=Unknown       -> ENCODE PANIC: unknown script
    -> the #w47tv00x citation is EXACT; the "Name: only" pin is correct

  NEW-I2, the three single-key multi twins
    wpkh(multi(2,…))                  REFUSE: nonstandard: unrecognized output descriptor format
    pkh(multi(2,…))                   REFUSE: nonstandard: unrecognized output descriptor format
    sh(wpkh(multi(2,…)))              REFUSE: nonstandard: unrecognized output descriptor format
    CONTROL wpkh(sortedmulti(2,…))    ACCEPT Segwit (P2WPKH) keys=2, Encode ok #6cc6zuge

Textual / structural
  §4.7 conjunct 1 md1-path twins (636-639) == the remedy's three named forms  MATCH
  §4.2 defect 2 (line 335) == the `Name: only\n` row                          MATCH
  manifest minima sum                                                          51
  51 - 2 permitted overlaps                                                    49  == stated floor
  r7's retag construction against the floor                                    48  -> FAILS
  §6 table rows                                                                34, none for zero cosigner lines
  stale `"multi("` literals remaining                                          0
```

---

## Closing

**6/6 FIXED, 0 PARTIAL; 0C / 0I / 1M / 1N new. THE CORRECTNESS LENS CLOSES.**

Both of r7's Importants are shut by measurement, not by reading. The read-back pin — open in one
form or another since r5, and the third round running on the same row — now fails the
`multi` → `sortedmulti` mutant on both read-back surfaces, and I reproduced the old pin's
false PASS first so the fix is demonstrated against the defect rather than merely asserted. The
wrapped-multi remedy names a wrapper change whose three forms are exactly conjunct 1's three
md1-path twins and all three encode; the operator journey terminates on an executable action;
and the false device parenthetical is excluded for `multi` inputs, re-measured on all three
twins against an accepting control.

The fold's own arithmetic holds under independent recomputation, including the judgement call
r7 left open: **49 was the right number and 50 would have been a defect**, because an honest
complete file has 49 rows and a 50-row floor would have failed it.

The two new items are outside the six edits — a pre-existing §4.2/§6 gap and a stale pointer —
and neither is a defect in the fold. **Do not run another correctness round for them.**

**What the spec's own text says remains before GREEN/done** — records, not new findings:

- **§9 item 7 — the §6 journey walk with the operator has not been done.** The spec says it
  *"should be walked before the plan closes"*. This is the outstanding lens, and it is the one
  that would have generated both of r7's Importants and this round's NEW-M1 (all three are
  "the operator does what the message says and lands somewhere wrong"). It is the next thing
  to do, not another read-through.
- **§11 items 1, 4 (its `--as descriptor` rows) and 6 are parked with S2 under F-418** — they
  need the device on the bench. S1 and S3 close without them.
- **§9's other residuals:** nothing run on hardware (item 1); the `ClassDescriptor` display path
  has never executed (item 2, = §11 item 6); change-chain and testnet address equality unmeasured
  (item 3); the published `md-codec` 0.42.0 tarball not byte-checked against the tree (item 4);
  TinyGo/RP2350 build of a new `sysw.Classify` arm unchecked (item 5); negative-claim scope
  limits (item 6).
