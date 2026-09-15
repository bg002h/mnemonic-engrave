# Fold verification — the restore lens + the journey walk

**Artifact:** `design/SPEC_hashlock_kinds.md` at `bb74ac8d` ("fold: the restore
lens and the journey walk -- 1C/10I"), 606 lines. **Fold diff:**
`git diff 0c81a852..bb74ac8d` — one file, +120/-8.

**Scope:** (1) did the fold close each finding of
`spec-hashkinds-restore-lens.md` (`e3151ecd`, 0C/5I) and
`spec-hashkinds-journey-walk.md` (`0c81a852`, 1C/5I); (2) did the fold introduce
a new defect, including a false citation. Not a fresh audit; the three
correctness rounds are not re-opened and §2's operator decisions are not
re-litigated.

**Trees read:** mnemonic-engrave `bb74ac8d`, fork `0562e81`, mnemonic-secret
working tree. All three clean before and after; one temporary Go probe test was
written into `backup/`, run, and deleted (`git status` clean, verified).

---

# Verdict first

**NOT GREEN — 1 Critical, 4 Important.**

The Critical is not a missed finding; it is a **new** one the fold created. §13.1
states, as a measurement, that putting the kind on the preimage plate is free and
needs no layout change. The QR half of that claim is exactly right — I reproduced
194 → 210 bytes at 53 modules. The **engraved-text** half is false: at the
documented worst case the plate **refuses to engrave at every font rung**,
machine-verified against the fork's own layout code, and the spec binds
ACCEPTANCE item 8 to the false half.

Nine of the eleven findings are closed, several of them well — C-1's remedy is
specific enough to build and a correct `hash256` wallet does pass the screen it
describes. Three are PARTIAL.

---

# Machine checks run before any judgement

Every citation the fold added or changed, resolved against the real tree:

| claim | result |
| --- | --- |
| `gui/composer_copy.go:557-571` is the confirm modal and prints `method:` | **TRUE.** `composerCopyHashlockConfirm` is exactly 557-571; `fmt.Sprintf("method: %s   chars: %d", …)` at :559 |
| `gui/composer_copy.go:641-648` is the reconciliation body | **TRUE.** `composerCopyHashlockReconcile` 641-647 (648 blank); the quoted sentence is verbatim at :644-646 |
| `gui/composer_hash.go:135` is the live counter, 30 lines past `:79-105` | **TRUE.** `fmt.Sprintf("%d of 64 hex", len(frag))` at :135; 135 − 105 = 30 |
| `restoreDoc` occurs zero times in `gui/composer*.go` | **TRUE.** 0 in all 65 matching files; the identifier lives only in `multisig_restore.go`, `singlesig*.go`, and three tests |
| QR worst case 194 bytes / 53 modules | **TRUE.** `backup/hashlock.go:69-73` says so in source, and my probe measured 194 bytes → `code.Size == 53` |
| `+ hash: ripemd160` → 210 bytes, still 53 modules | **TRUE.** Probe row E: `qrbytes=210 modules=53`, plate still fits at 3.0mm with 1.20mm spare |
| `cmd/emu/walk_hashlock_phrase.js:339-345` carries the tautology argument | **TRUE**, verbatim at :339-345 (pre-existing citation, reflowed by the fold) |
| H6 ACCEPTANCE item 8 is the QR-scan gate and is unaffected | **TRUE as to the QR.** Item 8 = "THE QR IS A GATE"; module count unchanged, so the optical claim is unchanged. Its **cut** is affected — see C-NEW-1 |
| `ms hashlock`'s `for md compose: … sha256=<h>` | **TRUE.** `ms-cli/src/cmd/hashlock.rs:406` |
| `ms hashlock` without a kind returns the sha256 digest | **TRUE.** `ms-codec/src/hashlock.rs:59-63`, unconditional SHA-256; no `--kind` exists today |
| `hashlockPlateFormWords` is the plate's `method:` line | **FALSE** — it is §8.3's **census form column** (`gui/composer_preimage_plate.go:204-216`, its own doc comment). See I-NEW-1 |

**Probe (temporary, deleted).** `backup` package, production `sh2.Params()`, the
H6 worst-case phrase plate (100-char phrase, hardened method, QR):

```
A baseline worst case                         qrbytes=194 modules=53 rungs_that_fit=[3] fontMM=3.0 rows=10 spare=1.20mm err=<nil>
B kind appended to the method line (88 chars) qrbytes=209 modules=53 rungs_that_fit=[]  err=backup: the hashlock plate does not fit at any font size: 11 rows at 3.0mm need 427520 units against a budget of 416000
C kind appended to the locator hash row (35c) qrbytes=194 modules=53 rungs_that_fit=[3] fontMM=3.0 rows=10 spare=1.20mm err=<nil>
D kind as its own extra body row              qrbytes=194 modules=53 rungs_that_fit=[]  err=… 11 rows at 3.0mm need 427520 units against a budget of 416000
E QR text gains a `hash: ripemd160` line      qrbytes=210 modules=53 rungs_that_fit=[3] fontMM=3.0 rows=10 spare=1.20mm err=<nil>
```

---

# Closure, finding by finding

## Restore lens (0C/5I/3M/1N)

### I-1 — the plate's `method:` line and QR become an incomplete definition — **PARTIAL**

**Closed:** the surface is named (§13.1), §4 no longer closes it on F1, §11 gains
a row, ACCEPTANCE gains item 8, and the QR measurement is reproduced correctly.

**Not closed, three ways:**

1. The **text** half of the remedy is asserted free and is not — C-NEW-1 below.
2. The finding said the v1/v2 QR format-string question is *"a decision the spec
   must **make**, not inherit."* §13.1 mentions *"its `hashlock v1` QR"* in
   passing and makes no decision. After this cycle one `hashlock v1` string names
   four constructions, and the vendored corpus row's own note (*"the middle line
   IS the selector"*) is false — neither is addressed.
3. The remedy's **landing** is wrong or missing — I-NEW-1 below.

### I-2 — the plate-verification path needs a kind the plate lacks — **CLOSED**

§13.4 **requires** `--kind` and **permits** the four-digest fallback, in the
finding's own terms, and §13.1 puts the kind on the plate so ACCEPTANCE item 1's
route works from the plate alone. §9 phase 2 already carries *"`ms hashlock`
learns the kind."*
*Residue (Minor):* the finding also asked that §12 item 5 assert the **plate-only**
route; item 5 is unchanged. Substantively covered by item 8 + §13.4, so this is
recorded, not blocking.

### I-3 — the census row describes four constructions identically — **CLOSED**

§13.3 states it, §5's table puts the census row on the `method` axis, §11 gains
*"the §8.3 census row"*.
*Residue → I-NEW-4:* the sibling **pick lead** screen carries the same
method-only text and is named nowhere.

### I-4 — §4 uses F1 to close a question F1 does not answer — **CLOSED**

§4's bullet is rewritten exactly as asked, in the same "stated twice on purpose"
register as the C-2 paragraph, and hands the surface to §13.

### I-5 — F-132's open half widened without being named — **CLOSED (spec level)**

§13.5 names F-132, names the missing restore document, and cites the measurement
(`restoreDoc` = 0, verified). `FOLLOWUPS.md` itself is untouched — correct at
spec stage, since §13.5 assigns it to this cycle's plan.

## Journey walk (1C/5I/4M/2N)

### C-1 — the kind is absent from the confirm and reconciliation screens — **CLOSED**

§13.2 states both bodies name the kind, the reconcile sentence names the flag
(`ms hashlock … --kind <kind>`), and the write-down list gains the kind; §5's
rule makes it general; §11 gains the two line ranges (both verified exact);
§12 item 7 is a dedicated acceptance item.

**Read as an implementer, it builds, and the resulting screen passes its own
check.** On the phrase arm with `hash256`: the screen names `method: hardened`
and the kind; `ms hashlock --method hardened --kind hash256` derives the same
32-byte preimage (F1) and applies `sha256d` — the same function §10's KAT pins —
so the digests match. `refuse_method` (journey M-4) does not bite on the phrase
route.

**Item 7 can fail** — a screen that omits the kind leaves the operator unable to
construct the command, which is a failing item. *Minor:* item 7 names no
executing mechanism (walk assertion? operator?), unlike item 2, which argues its
own falsifiability at length; the cycle's Critical is operator-facing, so the
plan should bind item 7 to an assertion on the drawn screen text.

### I-1 — `ms hashlock`'s operator surface is sha256-hardcoded — **PARTIAL**

**Limb (a) closed:** §11 gains *"`ms hashlock`'s operator surface, incl.
`for md compose: … sha256=<h>`"*. That is the sharp limb (the line that
*generates* a wrong-but-pasteable operand), and the quoted string is verbatim at
`ms-cli/src/cmd/hashlock.rs:406`.

**Limb (b) not closed:** `me-cli/src/main.rs:195,197` — the `me sysw pack`
`--record` help, which the H6 spec designates *the producer* of this grammar —
still reads `hash:<64 lowercase hex>` and *"a sha256 hashlock digest"*. Verified
present at exactly those lines. §11's me-cli string row still cites only
`main.rs:2275,2685,3196` and `composer_records.rs:144`. See I-NEW-2.

### I-2 — `me bundle` is hashlock-blind — **CLOSED as scoped**

§13.5 names it and files it as a follow-up owned by this cycle's plan, which is
what the finding itself recommended.
*Minor tension:* §11 — *"Gates that must move deliberately"* — also gains
`me bundle`'s plate checklist, so one surface is simultaneously deferred and
listed as in-cycle gate movement. One of the two should say which.

### I-3 — §7.2's "unchanged, kind-generic" is self-contradictory — **CLOSED**

§7.2 now says **not unchanged**, names the false word (SHA-256) and states what
the generic body must say instead (a 32-byte preimage, no function named). The
copy-table count row in §11 gates the resulting string change.

### I-4 — the Back leg of the inserted screen — **PARTIAL**

**Closed:** §7.1 declares the leg normative, cites `SPEC_hashlock_H2_device.md`
§4.6 correctly (verified: §4.6 is the Back contract, enumerates the legs, and
records that `composerHashEdit` returning `false` removes the path at creation),
and states two legs: Back from the kind screen → the source pick, no held
material discarded; Back from the pad → the kind screen, kind still selected.

**Not closed:** both stated legs are the **hex** arm's. The finding's scenario is
the **phrase** arm, and the spec still does not say where the kind screen sits in
that loop, nor what it does to H2 §4.6's enumerated leg *"Back from the phrase
screen → `Which hash?` (phrase **dropped**)"*, which now skips the newly inserted
screen. Walk the journey's own case with the spec as folded: wrong kind seen at
the confirm modal → Back → method pick (§4.6) → Back → phrase screen → Back →
`Which hash?` **with the phrase dropped** → re-pick source, re-pick kind, re-type
100 characters, second ~10 s KDF. That is precisely the cost the finding said was
*"worse than any alternative"*, and it survives the fold unacknowledged. Note
F1 makes this avoidable for free — the preimage is kind-independent, so a kind
re-pick need not re-derive anything — and the spec never considers it. See
I-NEW-3.

### I-5 — the pad's live counter — **CLOSED (with a Minor residue)**

The counter is now its own §11 row at the correct line. The finding's secondary
asks are unfolded: the row for `composerHexEntry` still reads *"three 64/32
constants"* at `:79-105` when a grep measures **four** literals in that range
(`:87` `> 64`, `:88` `[:64]`, `:91` `== 64`, `:98` `!= 32`), and
`"That is not a 32-byte digest."` (`:103`) — wrong for a 20-byte kind — is
covered only by falling inside a range whose label does not describe it. Minor;
"never hand-count what a tool can count" applies.

### Minors / Nits

- **M-1 (route 6)** — folded as an appended paragraph, and the paragraph now
  contradicts the table row it corrects: row 6 still reads *"the preset archetype
  (`--preset` / a composer preset) … the kind comes from the preset's own grammar"*,
  which is false for the device preset the new paragraph excludes
  (`composerPresetDigest` is 32 bytes of `0xa8`, no grammar). Fix the row, not
  just the prose under it. **Minor.**
- **M-2, M-3, M-4 (journey)** — no change earned / one line for the plan; M-4's
  `refuse_method` trap is unrecorded anywhere. **Minor.**
- **N-1** — folded: §7.5 gains *"The kind screen's own four rows are bound by the
  same arbiter."* **CLOSED.**
- **N-2 — not folded and still false:** `"Type 64 hex"` is at
  `gui/composer_hash.go:**349**`; §11 cites `:348`. The test citation
  (`composer_hash_test.go:257`) is correct. **Nit.**
- **Restore M-1, M-2** — M-2 is inside §13.5; M-1 (`policySummaryLines`) is not
  mentioned. **Minor.**
- **Restore M-3** — *"the locator's layout cost is not free the way the QR's is"* —
  not folded, and it is the exact mechanism behind C-NEW-1.

---

# New defects

## C-NEW-1 (Critical) — §13.1's "measured, the remedy is free … no layout change" is false for the engraved text, and ACCEPTANCE item 8 is bound to it

**The claim.** §13.1: *"The plate's `method:` line and its `hashlock v1` QR are …
the complete definition … **Measured, the remedy is free:** adding
`hash: ripemd160` to the 100-character worst-case QR text takes it 194 → 210
bytes and it remains at 53 modules … **so no layout change** and no change to
ACCEPTANCE item 8's 32m12s cut."* §12 item 8 repeats it as acceptance.

**What is measured is the QR only.** The clause names **two** surfaces and
measures **one**. The plate's engraved text is governed by a different budget
entirely, and it is already at its limit:

- `backup/hashlock.go:246-281` — `EngraveHashlock` walks `FontSizes` largest
  first and **refuses** with `ErrHashlockTooLarge` when nothing fits.
- `backup/hashlock_test.go:64-117` (`TestHashlockWorstCaseFitsAtExactlyOneRung`)
  — the worst case fits at 3.0 mm **with 1.20 mm to spare and at no larger rung**,
  and its own mutation note records that a 79-character method line *"wraps to 3
  rows … the body is 11 rows, and EngraveHashlock refuses with `11 rows at 3.0mm
  need 427520 units against a budget of 416000`"*.
- `SPEC_hashlock_H6_preimage_plates.md` §6.5 **pins the method line at 73
  characters** and already ruled on this exact move for a different token:
  *"Appending the selector to the method line costs 32 characters (105 total,
  MEASURED), and §6.5 pins the line at 73 with a 79th character pushing the worst
  case over budget — **so it cannot go THERE**."*

**Measured here, not argued** (probe above, production params, worst-case plate):

| placement | result |
| --- | --- |
| kind appended to the **method line** (88 chars) | **REFUSES at every rung** — `11 rows at 3.0mm need 427520 units against a budget of 416000` |
| kind as **its own body row** | **REFUSES at every rung**, same error |
| kind appended to the **locator's `hash` row** (35 chars) | fits at 3.0 mm, 10 rows, 1.20 mm spare — **free** |
| kind added to the **QR text** only | 210 bytes, 53 modules, fits — **free** (the spec's own claim, correct) |

So the surface §13.1 names is the one place the kind **cannot** go, and there is a
free placement the spec does not name. Consequences as the spec stands:

1. An implementer following §13.1 reds `TestHashlockWorstCaseFitsAtExactlyOneRung`
   and has no spec decision to fall back on — no budget, no alternative surface,
   no ruling on shortening the method line or moving the QR rung.
2. The obvious escape — put the kind in the **QR only** — silently reopens restore
   I-1, because H6 §6.4 makes the **text** authoritative and the QR a convenience;
   the lens's scenario is a person reading steel by eye.
3. **ACCEPTANCE item 8's "unaffected" is half true.** The QR's module count and
   therefore the optical gate are unchanged (correct). Its **cut** is not
   necessarily unchanged: any text-side remedy that alters rows or rungs changes
   the plate that item 8 cuts and every golden
   (`backup/testdata/hashlock-*.bin`, `TestHashlockGoldens`).
4. §11 has no row for the plate's fit gate at all — neither
   `TestHashlockWorstCaseFitsAtExactlyOneRung` nor the goldens nor §6.5's
   73-character pin appear anywhere in this spec.

**Why Critical.** The clause states a measurement that measurement contradicts,
and the remedy it prescribes cannot be built at the worst case. This is the class
§7.5 exists to prevent — *"the row must be measured, not assumed"* — applied to a
screen and then not applied to the plate, on the same page. Restore M-3 warned
about exactly this and was not folded.

**Cheapest fix, already measured:** put the kind on the locator's `hash` row
(`hash  b867db87..edbc96cb  ripemd160`, 35 characters, one line at 3.0 mm, no rung
change), keep the QR line, say plainly that the method line **cannot** carry it
and why (§6.5's pin), and add the fit gate + goldens to §11.

## I-NEW-1 (Important) — §13.1's obligation has no home in §9 or §10, and its §11 row cites the wrong function

- **Wrong identifier.** §11's new row —
  *"the preimage plate's `method:` line and `hashlock v1` QR text |
  `backup/hashlock`, `hashlockPlateFormWords`, `QRText`"* — names
  `hashlockPlateFormWords`, which is **§8.3's census form column**
  (`gui/composer_preimage_plate.go:204-216`: *"hashlockPlateFormWords is §8.3's
  form column"*), i.e. the surface of the *next* row, not the plate's method line.
  The function that actually builds the line, `hashlock.MethodLine`
  (`hashlock/hashlock.go:176-182`), occurs **zero** times in the spec — verified
  by grep — and it was named explicitly in restore I-1's remedy.
- **No Rust primary.** `QRText`/`MethodLine` are declared downstream twins of
  `ms_codec::hashlock::qr_text` (`ms-codec/src/hashlock.rs:208`), pinned against
  vendored rows, per H6 §6.5 item 4a: *"They are DOWNSTREAM of the Rust primary."*
  Under this constellation's Rust-primary rule the change lands in
  **mnemonic-secret first, with vectors**. §9 phase 2 says only *"one digest
  function per kind … `ms hashlock` learns the kind"*; phase 4 says *"Go ports of
  1-3 and the device UI"* — and a plate-text change is neither a port of 1-3 nor
  device UI. `qr_text` occurs zero times in the spec.
- **No vector.** §10 adds digest-KAT rows to `hashlock-v0.8.json` and says nothing
  about the `qr_text` rows in the same file, which restore I-1 showed carry a note
  that this cycle makes false.

An implementer therefore has a mandatory §13.1 clause with the wrong function
cited, no phase that owns it, and a cross-language pin nobody was told to re-cut.

## I-NEW-2 (Important) — journey I-1's second limb was dropped in the fold

`me-cli/src/main.rs:195,197` (verified verbatim: `hash:<64 lowercase hex>` and
*"a sha256 hashlock digest"*) is the `me sysw pack --record` help — the producer's
own statement of the grammar §6 rewrites. After this cycle both halves are false
(40-hex kinds exist; the kind token exists). It is on no §11 row, in no phase
description, and the fold neither folded it nor declined it with a reason. The
fold's other response to I-1 (the `ms hashlock` row) shows the limb was read; this
one simply did not land.

## I-NEW-3 (Important) — the phrase arm's Back leg is still unstated, and the stated legs leave the journey's own case costing a dropped phrase and a second KDF

Detailed under journey I-4 above. The spec says the Back contract is *"normative
and must be stated"* and then states only the hex arm's two legs. Missing, and one
sentence each: where the kind screen sits in the phrase loop, and what happens to
H2 §4.6's *"Back from the phrase screen → `Which hash?` (phrase dropped)"* now
that a screen has been inserted in front of it. This tree's own precedent is
quoted in the same paragraph — *"the Back path that skipped a screen also skipped
a guard"* — and the new leg skips a screen.

## I-NEW-4 (Important) — a third screen prints the bare `method:` token, is bound by §5's new rule, and is named in neither §13.2 nor §11

`composerCopyPreimagePlateLead` (`gui/composer_copy.go:717-725`) draws the masked
plate-pick lead:

```
hash  <first8>..<last8>   path N
phrase: <n> characters   method: <method>
```

By §5's own rule — *"A screen that shows `method:` and no kind on a policy whose
kind is not sha256 violates this spec"* — this screen violates the spec as folded,
yet §13.2 enumerates only the confirm and reconciliation bodies and §11 gates only
those two ranges. It is also the screen where restore I-3's harm actually lands:
the operator choosing **which of two near-identical preimage plates to cut**.

And it carries its own measured budget the fold does not account for: its doc
comment records *"TWO LINES, AND THE NUMBER IS MEASURED … a four-line lead leaves
3 of this screen's 4 rows on the first page, and `do not cut this preimage` is the
row an operator reaches for to UNDO"* (`TestComposerPreimagePlatePickDrawsAllFourRows`).
Same shape as C-NEW-1: naming the kind is not free on every surface, and the spec
asserts the rule without pricing it.

---

# Answers to the brief's specific questions

**Is the §5 two-axis rule enforceable?** The second sentence is (*"no surface
prints one bare token that an operator could take for the other"*). The first —
*"both are named or neither is"* — over-reaches and collides with the spec's own
§6 in three places, all of them harmless in outcome and all of them textual
violations: a **bare `hash:` record** (§6's producer rule *requires* emitting no
kind token, while the method axis is visible in the same records file); a
**`phrase:` record**, whose `method` field is printed with no kind because routes
4/5 are sha256 by construction (§7.1); and **`ms hashlock`'s stdout**, which prints
a method line and, for sha256, a bare `hash:<64hex>` record. Following the first
sentence literally on any of these produces either a wire-format change §6
forbids or the removal of a `method` field. It needs one scope clause —
*"binds surfaces an operator reads where the kind may be other than sha256"* —
which is what the second sentence already implies. **Minor.**

**Did §13 create obligations the rest of the spec does not carry?** Yes, for 13.1
(I-NEW-1: no phase, no vector, wrong identifier, no fit gate) and partly for 13.2
(I-NEW-4: one bound screen unenumerated). 13.4's `--kind` is carried by §9 phase 2
and §12 items 5/8. 13.3 is carried by §11's census row and phase 4.

**Are 13.5's two "named rather than fixed" items honestly scoped?** Yes. Neither
acceptance item 7, 8 nor 9 depends on `me bundle` or the restore document: item 9
tests the **confirm modal's** write-down list, which §13.2 puts in scope. The only
blemish is §11 listing `me bundle` among gates that must move (Minor, above).

**Did the fold falsify anything it did not touch?** One thing. §14's second bullet
still reads *"F1 dies and with it **the scope boundary that keeps H6 out of this
cycle**"* — but §4, rewritten two screens earlier, now says the opposite
(*"F1 does NOT put the preimage plate out of scope"*), and §13 brings the H6 plate
surface in. A stale one-clause echo of the boundary the fold deliberately moved.
**Minor**, and it is the same shape as the C-2 the design review found.

---

# Other Minors / Nits recorded

- **§13's opening count.** *"seven plate- and restore-related identifiers appear
  nowhere in the earlier drafts"* — the restore lens's own table lists **nine**
  identifiers at zero (`hashlockPlateFormWords`, `hashlockPlateLocator`,
  `composerCopyPreimagePlateRow`, `MethodLine`, `QRText`, `qr_text`,
  `backup/hashlock`, `multisig_restore`, `buildPlateInventoryLines`). Hand-count
  where a grep was available; six of the nine are **still** zero after the fold.
- **Two costs for one gate.** §12 item 8 says ACCEPTANCE item 8 is *"~43 min"*;
  §13.1 says *"item 8's 32m12s cut"*. Both numbers are real
  (`ACCEPTANCE_hashlock_H6_preimage_plates.md`: whole worst-case plate 43m31s, QR
  alone 32m12s) but they name different cuts and read as a contradiction.
- **§12 item 9 cannot pass as literally written.** *"A written record made by
  following §13.2's list is sufficient to rebuild the descriptor — the test is
  `md compose` with nothing but what the operator was told to write down."*
  `md compose` also needs the keys, threshold and template, which the confirm
  modal's list never contained and this cycle does not add. The intent (the
  hashlock **operand** must be reconstructible) is right; the wording makes an
  unsatisfiable gate.
- **§9 phase-1 row cites `(§9.1)`, which does not exist** in the document
  (sections run 7.1-7.6, then 8, 9, 10…). Pre-existing at `0c81a852`, not
  fold-introduced.
- **§13.2's quotation** lowercases the reconcile screen's *"If they differ"* to
  *"if they differ"* across the ellipsis. Trivial.

---

# Counts

**1 Critical / 4 Important / 9 Minor / 3 Nit.**

Closure of the folded findings: **8 CLOSED, 3 PARTIAL, 0 NOT CLOSED**
(restore I-2, I-3, I-4, I-5; journey C-1, I-2, I-3, I-5 closed;
restore I-1, journey I-1, journey I-4 partial).

# Verdict

**NOT GREEN.** The fold does the operator-facing work well — C-1's remedy is
buildable and would let a correct `hash256` wallet pass its own check, §4's scope
contradiction is properly retired, and every screen citation it added resolves
exactly. It fails on the plate: the one clause that states a *measurement* states
one that the fork's own layout code refutes, in the direction that makes the
remedy unbuildable, with the acceptance item pinned to it — and the surrounding
obligation has no phase, no vector, and a misattributed function. One more fold,
scoped to §13.1's placement decision plus the three Important gaps, should close
this.
