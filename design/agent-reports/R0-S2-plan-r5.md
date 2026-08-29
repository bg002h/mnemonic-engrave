# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 5 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`, as folded at
`d5a22aa`.
**Round 4:** `design/agent-reports/R0-S2-plan-r4.md` (RED, 0C/2I/2M/0N),
persisted at `46c20ce`, folded at `d5a22aa`. The reviewed text is exactly
`git diff 46c20ce..d5a22aa -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`
— 85 insertions, 23 deletions, **7 hunks, all read**.
**Trees:** mnemonic-engrave `d5a22aa`; seedhammer fork `main` @
`a5e29b44637d0657ab8f1ec603f1a375b0cc54cb`.
**Nothing in either repo was modified.** No fork patch was needed this round:
every finding below rests on file data read as data, source lines read at
`HEAD`, and r2's already-measured table.

**THE ONE QUESTION:** does the r4 fold resolve each of r4's four findings
(I1, I2, M1, M2), and did the fold's edits introduce a new defect?

**Counts: 0 Critical / 2 Important / 1 Minor / 2 Nit — verdict RED.**

**All four of r4's findings are answered in the text, and three of them are
answered soundly.** I1's test is well-formed and — verified against the fork's
real API — can actually fail. M1's `Pop` enumeration is exact. M2's move is
made. The defect is in the one thing r4 asked the fold to *rule* rather than
fix: **the `neither`-tag ruling.** It retags the `ypub` row into a tag that no
§7 bullet admits, names two different tags in the two halves of its own
sentence, and declares the manifest arithmetic settled ("the floor of 3 holds,
`MANIFEST`'s `("neither", 3)` does not move") while the counts that *do* move
under it — in three separate copies of the manifest — are named nowhere (**I1**).
And the M2 move carried one member whose falsifying event is in **P2**, not P3,
into P3.5's ownership (**I2**).

---

## Method

- The vector file, `comment.json` and `rows.py` read as **data**
  (`python3 -c 'json.load(...)'` and `sed -n`), never by eye; every count below
  is computed, not transcribed.
- `crates/me-cli/tests/descriptor_seam.rs`, `design/SPEC_descriptor_input.md`,
  the fork's `sysw/record.go`, `sysw/classify.go`, `bip380/bip380.go` and
  `nonstandard/descriptor_seam_test.go` read at the stated revisions.
- r1–r4's verified tables taken as settled per the brief; nothing in them
  re-derived.

### Verified by measurement — the fold's new claims that are SOUND

| fold claim | verdict |
| --- | --- |
| **I1's test is well-formed against the fork's real API** | **CONFIRMED.** `func Classify(record string) Class` exists at `sysw/record.go:97` and `ClassUnknown` at `sysw/record.go:27`. The fold's `sysw.Classify(…) == ClassUnknown` is a real call, not shorthand. |
| **…and it CAN FAIL — it reaches the arm P3.1 adds** | **CONFIRMED, and this is the load-bearing half.** `Classify` matches `PassPrefix`/`TextPrefix`/`TxPrefix` and otherwise falls through to `return classifyConstellation(record)` (`sysw/record.go:120`), which is exactly the function P3.1's descriptor arm is added to (`sysw/classify.go:34`). A bare `ypub` carries none of those prefixes, so the assertion exercises the new arm rather than being absorbed by an earlier guard. Not a gate that cannot fail. |
| **"the same key material as the vector row" is determinate** | **CONFIRMED, exact.** `scripts/descriptor-seam-vectors/rows.py:29` binds `SKYL = "ypub6WyzNbqt7S3quv2F6R9eyqabYpQieQKk7P9uufmRv2LjhLjskjho9N1sEmTvAXSURk5eF9UdiS2jqgLXM3gpHeExWDvj1KyiEaqi47h3Ef1"`, and the row's input is `"sh(wpkh([%s/49h/0h/0h]%s/<0;1>/*))" % (SKFP, SKYL)` (`rows.py:325`). The bare string is a single-line base58 constant with a name; the extraction is mechanical. |
| **M1: the six `Pop` fields, and their current values** | **CONFIRMED, recomputed from the file.** Source: `rows: 71`, `device_admits_true: 37`, `device_admits_false: 33`, `device_admits_absent: 1`, `sysw_class: 4`, `device_probe: 3` (`descriptor_seam.rs:130-147`). File: 71 rows; device_admits true/false/absent = **37/33/1**; `sysw_class` on **4** rows; `device_probe` on **3**. Every value the fold quotes is the file's. The fold correctly declines to predict `device_admits_true`/`_false` (the panic:parse row's boolean is measured, not known). |
| **M1: no seventh `Pop` field moves under the stated payload** | **CONFIRMED.** The new witness row carries no address/canonical fields (`host_admits=false`, `md1_admits=false`), and the `ypub` row carries none either (measured: its keys are `name`/`input`/`sha256`/`host_admits`/`device_admits`/`md1_admits`/`format`/`covers`/`source`). `gate_fields: 37` matches the file's 37 `gate_open` rows and does not move — **provided the retag is single-tagged**, see I1. |
| **The Go half's address guards do NOT move (unnamed, correctly)** | **CONFIRMED.** `wantDeviceAddr0 = 16` / `wantDeviceAddr1 = 4` (`nonstandard/descriptor_seam_test.go:82,85`) count device-derivable `address_N` rows. Measured: the `panic:parse` row (`bluewallet/short-fingerprint`) carries neither `address_0` nor `address_1`, and neither does the new witness row. So the probe retirement and the 72nd row leave both guards alone. |
| **The new witness row is genuinely `false`/`false`** | **CONFIRMED — but the measurement is r2's, not r3's (Nit 1).** `R0-S2-plan-r2.md` measured `wsh(multi(2,…/0/*))`: `--as descriptor` → rc 3, `OutputDescriptor` → *"unrecognized output descriptor format"*, so `device_admits=false`. Mechanism confirmed independently: `bip380/bip380.go:335` cases **`"sortedmulti"`** only, so `multi` is refused irrespective of the use-site path. The substitution's premise holds. |
| **The vacuity check survives the substitution** | **CONFIRMED.** `descriptor_seam.rs:429-449` computes `neither` from the **data** (`(false, false) => neither += 1`) and asserts `neither > 0`, not from the tag. Post-payload the false/false rows are `neither/wsh-multi`, `neither/miniscript` and the new witness row = 3. Requirement 5 is satisfied either way. |
| **M2's move is made in both places** | **CONFIRMED.** P2.7 now reads *"move to **P3.5's ownership**"*, and P3.5 carries the batch by name (`:453-461`, `:570-574`, `:1610-1611`, `cascade.rs:58-62`, `refusal.rs:583` + `descriptor_refusals.rs:466` + the §6 quote). |
| **I2's six sites and the sweep term are on P0.1** | **CONFIRMED, read against the claim.** `:453-461` with `:462` explicitly kept CORRECT; `:570-574`; `:1610-1611`; `cascade.rs:58-62`; `refusal.rs:583` with its pin and §6 quote; and the row's `source`/`name`/`covers` + `rows.py:327` + `comment.json:107` in invariant 1's payload. `rows.py:327` is the `source=SPEC + " S4.3: ypub has no case…"` line — the right line. `ypub` is on the sweep-term list. |
| **The P2 sweep does NOT red at P2 on P3.5-owned text** (brief's pressure point 3) | **CHECKED — NO DEFECT, stated for the record.** The P2 gate's sweep pass condition is scoped: *"propagation sweep whole-repo including the spec (**the S3-parked phrasings** must survive ONLY in `design/agent-reports/` and historical review text per P0's inventory)"*. `ypub` hits at `:453-461`/`:570-574`/`refusal.rs:583` are not S3-parked phrasings, and those sentences are still TRUE at the P2 gate (fork `main` unchanged). The fold did **not** create a sweep that reds at P2 on text scheduled for P3. The sequencing defect that does exist is a different member — see I2. |

---

## IMPORTANT

### I1 — the `neither`-tag ruling retags the row into a tag no §7 bullet admits, names two different tags in one sentence, and moves manifest counts in three artifacts that it declares unmoved

The fold's ruling, verbatim (`plan:71-81`):

> the row is RETAGGED out of `neither` (a false/true row contradicting the tag's
> §7 definition may not keep it) and re-covers the device-wider-than-host bullet
> its data now evidences (the class `promotion/15-bare-tpub-host-refused`
> already covers); the NEW witness row … takes the vacated `neither` slot, so
> the floor of 3 holds, `MANIFEST`'s `("neither", 3)` does not move, and §7's
> named-three sentence amends by SUBSTITUTION

The premise is right and r4 asked for exactly this ruling. Three things under it
do not hold.

**(a) The destination is ambiguous, and one reading reds a shipped assertion.**
`promotion/15-bare-tpub-host-refused` covers **two** tags — measured from the
file: `covers = ['promotion-near-miss', 'gate']`. The sentence says "the class …
already covers" (singular) while pointing at a row with two.

- If the row takes **both**, then `second` (rows with ≥2 tags) goes 15 → 16 and
  `assert_eq!(second, SECOND_TAGGED)` (`descriptor_seam.rs:385`) **fails**; and
  §7 requires the gate fields on *every* `gate`-tagged row (`:1563`, *"REQUIRED
  on every `gate`-tagged row, absent elsewhere"*), which the `ypub` row does not
  carry — so `POP.gate_fields` 37 → 38 too, a seventh moving field the fold's
  own M1 list does not have.
- If it takes `promotion-near-miss` **alone**, see (b).

Neither branch is written down, and r4's instruction was that "the P2.6 commit
cannot discover the question".

**(b) `promotion-near-miss` is a closed §7 membership the row cannot join, and
so is the only tag whose *data* signature actually matches.** Measured:

- §7's bullet: *"**the promotion near-misses of §4.5** — all **fifteen** rows of
  that table"* (`SPEC_descriptor_input.md:1582`); floor table:
  `| promotion-near-miss | §4.5's fifteen-row table | 15 |` (`:1728`).
- §4.5's branch is `bip380.ParseKey(nil, enc)` **on the whole file**
  (`SPEC_descriptor_input.md:540-546`) — a bare-key promotion path. Measured:
  all 15 `promotion-near-miss` rows are bare keys or bare-key-with-origin
  (`promotion/01-bare-xpub` … `promotion/15-bare-tpub-host-refused`); **zero**
  are wrapped descriptors. The `ypub` row is
  `sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))`, which never reaches that
  branch.
- The bullet whose data signature the row now matches is **`narrowed-4.7`**
  (`:1583-1590`, *"`host_admits=false`, `device_admits=true`. These are the rows
  the invariant is for"*) — the actual "device-wider-than-host" bullet in §7.
  But its membership is an explicit enumeration of 14 §4.7-narrowed shapes, and
  the `ypub` row's host refusal is a **§4.3 version-byte** refusal, not a §4.7
  narrowing.

So the fold's two halves name different tags — the *description*
("device-wider-than-host bullet") points at `narrowed-4.7`, the *citation*
(`promotion/15`) points at `promotion-near-miss` — and **under either, the row
contradicts the destination bullet's §7 definition, which is the exact reason
the ruling gives for evicting it from `neither`.**

**(c) The manifest arithmetic is declared settled and is not.** `("neither", 3)`
indeed does not move. What moves, computed from the file (current tag counts:
`formats-happy` 4, `promotion-near-miss` 15, `narrowed-4.7` 14,
`accepted-extreme` 1, `narrowed-4.2` 5, `neither` 3, `whitespace` 3,
`md1-splits` 6, `gate` 37 = **88 slots over 71 rows**):

| after the payload | value | consequence |
| --- | --- | --- |
| slots | 88 → **89** (one tag moved, one added) | `assert_eq!(slots, TAG_SLOTS)` (`descriptor_seam.rs:373`) is **exact and computed from the file** — `TAG_SLOTS` must become 89 |
| `TAG_SLOTS`'s own definition | *"The minima sum to 88 tag-slots"* (`descriptor_seam.rs:62`) | 89 ≠ sum(minima) unless a **minimum rises** — i.e. `promotion-near-miss` 15 → 16 (or `narrowed-4.7` 14 → 15), which is (b) |
| `ROW_FLOOR` | 71 → 72 | `89 − 17 = 72` holds; overlap stays 17 (13 two-tag + 2 three-tag rows, measured) |
| §7's derivation, `SPEC:1719-1723` | *"the file carries at least **71 physical rows** (the minima sum to **88** tag-slots … 88 − 17 = 71)"* | **falsified**, and it contains no sweep term — not `ypub`, not any of the other five |
| §7's floor table, `SPEC:1728` and `:1732` | `promotion-near-miss … 15` and `neither … wsh(multi), miniscript, full-origin ypub … 3` | both falsified; only `:1732` contains `ypub`, so the sweep reaches one of the two |
| `comment.json:101-113` | a **third** copy of the whole manifest — `"  promotion-near-miss 15   S4.5's fifteen-row table"` and *"The minima sum to 88 tag-slots … so the file [carries at least 71 rows]"* | the fold's payload names **only** `comment.json:107`, the `neither` line |

The `>=` in `assert!(n >= *min)` (`descriptor_seam.rs:361`) means the MANIFEST
minima alone would not red — but `TAG_SLOTS` is `assert_eq!`, so the implementer
is forced to touch the sum, and the sum is where §7's normative derivation
lives. `descriptor_seam.rs:49` labels the constant *"SPEC_descriptor_input.md
§7, NORMATIVE"*, so this is code being edited out of conformance with its own
cited authority.

**Fix (two parts, both editing):** (i) name the destination tag explicitly, and
amend that tag's §7 bullet membership with it — or, cheaper and arguably right,
keep the taxonomy honest by giving the row its own justification rather than
borrowing a closed enumeration; (ii) add `SPEC:1582`, `SPEC:1719-1723`,
`SPEC:1728`, and `comment.json`'s manifest block (not just `:107`) to P0.1's
COMPLETE enumeration with an owning task, since no sweep term reaches three of
the four.

### I2 — M2's move carried §7's `neither` member into P3.5, but P2.6 is what falsifies it: the P2 gate now closes with the spec contradicting its own repo's regenerated file and the constants that cite it

The move's stated rationale (`plan:424-427`):

> they describe scan-door behaviour that arrives with P3.4 in P3, and the P2
> gate must not close with the spec asserting device behaviour fork `main` does
> not yet have

That rationale is correct for four of the five members, and I verified each is
still TRUE at the P2 gate: `SPEC:453-461`, `SPEC:570-574`, `cascade.rs:58-62`
and `refusal.rs:583` all describe **device** behaviour, and fork `main` is
untouched until P3. Correctly P3.5's.

**The fifth member is different.** `SPEC:1610-1611` describes **the vector
file**, not the device:

> **`wsh(multi(…))`, a miniscript descriptor, and a full-origin `ypub`** —
> `false`/`false` on the host/device axes, the `neither` rows the vacuity check
> needs.

Invariant 1 puts the whole regeneration payload — including the retag, the
`covers`/`name` change and the 72nd row — in **P2.6**, engrave half, inside P2
(*"the engrave copy updates at P2.6"*). So the constructed failure is:

**At the P2 gate, in one repo at one commit:**
`crates/me-cli/testdata/descriptor_seam_vectors.json` has 72 rows and no
`neither`-tagged `ypub` row; `crates/me-cli/tests/descriptor_seam.rs` carries
`TAG_SLOTS = 89` / `ROW_FLOOR = 72` under a comment naming §7 NORMATIVE (it
*must*, or the suite reds); and `design/SPEC_descriptor_input.md:1610-1611` and
`:1732` still say the `neither` rows are `wsh(multi)`, miniscript and
full-origin `ypub`, min 3, floor 71 from 88 − 17. The gate then runs *"a
proportional opus review over P1+P2 before the Go port starts"* against that
spec.

This is the mirror image of the defect M2 fixed — an amendment scheduled to the
wrong side of the phase that falsifies it — and the plan's own follow-up rule is
that an item is burned down in the phase that owns it. Note the plan *does*
accept a transient for the cross-repo vector-copy window and states the
reasoning for it; it states nothing here.

**Fix, one sentence:** keep `SPEC:1610-1611` (and the floor-table row `:1732`,
per I1) on **P2.7**, since P2.6's regeneration is what falsifies them — or state
the transient explicitly the way invariant 1's sequencing paragraph does. Either
is fine; silence is not, because the P2 reviewer cannot tell a scheduled
transient from a defect.

---

## MINOR

**M1 — the SUBSTITUTION leaves `SPEC:1612` with an ambiguous referent.** After
the ruled substitution the `neither` bullet names *two* `multi` rows
(`wsh(multi(…))` with `<0;1>/*` and the new `wsh(multi(…/0/*))`), and the very
next sentence is *"**The `multi` row** additionally carries `md1_admits=true`,
its md1-route `address_0` AND `address_1`, and pins the read-back via
`md_descriptor_contains: "wsh(multi("`"* (`SPEC:1612-1615`). Measured, that
sentence is true of the *existing* row only: the new witness row is
`md1_admits=false` and carries no address fields. The amendment task should
disambiguate the referent, not just swap the third item — this sentence carries
a load-bearing pin whose whole history (R0 r6 NEW-I1, r7 NEW-I1) is about it
being read too loosely.

---

## NIT

**N1 — "(measured, r3)" mis-attributes the `multi` rejection.** The fold writes
*"the device parser rejects `multi` (measured, r3)"*. Measured: `grep -n multi
design/agent-reports/R0-S2-plan-r3.md` returns three hits, none of them a device
measurement. The measurement is **r2's** (`R0-S2-plan-r2.md`, the
new-witness-row row of its verified table: `OutputDescriptor` → *"unrecognized
output descriptor format"*, `device_admits=false`). The fact is confirmed
(`bip380/bip380.go:335` cases `"sortedmulti"` only) — only the citation is
wrong.

**N2 — "r4 measured that a §4.5 promotion-table port … classifies a bare `ypub`
`ClassDescriptor`" overstates r4's evidence.** r4 labelled that a **"Constructed
counterexample"**; no such port exists yet, so it cannot have been measured.
What r4 *did* measure is the patched probe accepting a bare `ypub` and `me`
refusing the identical string at rc 3. The motivating sentence for a new test is
a place where provenance inflation matters more than usual.

---

## Fold vs r4's findings

| r4 | resolved? | evidence |
| --- | --- | --- |
| **I1** — the bare-key half of C1's remedy is specified and ungated | **RESOLVED, and the instrument is real** | P3.4 gains the third test verbatim as ruled: `sysw.Classify(<the bare ypub string, the same key material as the vector row>) == ClassUnknown`, *"its comment naming P3.4 as the reason it can fail"*. Verified: `Classify` exists (`sysw/record.go:97`), `ClassUnknown` exists (`:27`), and `Classify` falls through to `classifyConstellation` (`:120`) — the exact function P3.1's arm is added to — so the assertion reaches the arm and **can fail**. The key material is determinate: `rows.py:29`'s named `SKYL`, which is literally the row's key (`rows.py:325`). Provenance overstated in one clause → **N2**. |
| **I2** — P0.1 declared COMPLETE while omitting six measured sites; the §4.3 member cited the one line that stays true | **RESOLVED as enumerated; the ruling it demanded is where the defect is → this round's I1 and I2** | All six sites are on P0.1 with the corrected `:453-461` cite and `:462` explicitly kept CORRECT; the row's `source`/`name`/`covers` + `rows.py:327` + `comment.json:107` are in invariant 1's payload; `ypub` is a sweep term. **But** the `neither`-tag ruling r4 asked for (fix (e)) retags into a tag no §7 bullet admits and names two tags at once (**I1a/b**); the manifest counts it declares settled move in three artifacts, three of which no sweep term reaches (**I1c**); and one enumerated member is now owned by the wrong phase (**I2**). |
| **M1** — the `Pop` population literal is unnamed | **RESOLVED, exactly** | `descriptor_seam.rs:130-147` is on the engrave-half guard list with all six moving fields; every current value recomputed from the file and matching (71 / 37 / 33 / 1 / 4 / 3). The `gen.py`-does-not-emit-them note and the vacuous `:265-275` presence assertion are both correct. Verified no seventh field moves — conditional on the retag being single-tagged, which is I1(a). |
| **M2** — a P2 task owns an amendment describing P3 behaviour | **RESOLVED for four of five members; the fifth is inverted → this round's I2** | P2.7 defers the batch to P3.5, and P3.5 carries it by name. Verified `SPEC:453-461`, `SPEC:570-574`, `cascade.rs:58-62` and `refusal.rs:583` are all still TRUE at the P2 gate (fork `main` untouched) — correctly P3.5's. `SPEC:1610-1611` is not: it describes the vector file, which P2.6 regenerates inside P2. |

---

## What a fold has to decide, not just fix

**I1 is a ruling, not an edit.** Two live options, and the fold must pick one in
the text:

1. **Give the row its own §7 home** — amend the destination bullet's membership
   to admit a §4.3-refused, device-admitted descriptor row explicitly (and bump
   that bullet's min, `TAG_SLOTS` to 89, `ROW_FLOOR` to 72, the `88 − 17 = 71`
   derivation at `SPEC:1719-1723`, the floor table at `:1728`/`:1732`, and
   `comment.json`'s manifest block). Honest, and it keeps every tag's definition
   matching its membership.
2. **Reconsider the eviction.** Nothing in the *code* forces it: the vacuity
   check is computed from the data (`descriptor_seam.rs:429-449`), not from the
   tag, and `assert!(n >= *min)` tolerates a stale minimum. The eviction is a
   taxonomy decision, so if it is kept it must be paid for in full at (1); if it
   is dropped, §7's `neither` definition is what gets amended, and the new
   witness row becomes a fourth `neither` row (`("neither", 4)`, slots 89,
   floor 72 — the same arithmetic, one fewer contradiction).

Option 2 was the branch r4 named first ("keep the tag and amend §7's
definition") and it is cheaper by exactly one falsified bullet. Either way the
arithmetic sites in I1(c) must be enumerated with owners, because three of the
four are unreachable by every sweep term the plan lists.

**I2 is one sentence** — move `SPEC:1610-1611` (and `:1732`) back to P2.7, or
declare the transient. It reopens nothing.
