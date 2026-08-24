# R0 round-1 fold-check — SPEC_engrave_transaction.md

**Agent:** R0 round-1 fold-check, sonnet. **Date:** 2026-08-24.
**Scope:** did the fold (`caa90cb..48da287`) fix each of the 15 round-0 findings,
and did the ~545 new lines it added introduce a new defect. Not a fresh audit.

---

## Part 1 — did the fold fix each finding?

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| C1 | **PARTIAL** | §4.2a rules Structured Append and gates cutting on two conditions ("**TWO GATES, and neither may be assumed:** 1. Our encoder must actually emit Structured Append... 2. Real scanners must reassemble it off engraved steel"). But §4.3's literal procedure — *"PLATE 1 OF 2 — CUT ... TEST IT NOW, before you leave the machine. Scan the QR, then run `mt inspect` on what you get"* — is byte-for-byte unchanged by the diff, and nothing anywhere states that a multi-plate SA job's test must wait until every symbol is cut (SA reassembly needs *all* parts). C1's own walkthrough — test plate 1 of 2, get a truncated/unreassemblable read, report failure on a correct plate — still applies verbatim even once both gates are satisfied. |
| C2 | **FIXED** | "**NORMATIVE:** the chunks path engraves **TEXT ONLY**. It may not call `validateMdmk` unchanged, and the spec may not describe the chunks form as needing 'nothing new'." (§2.2a) |
| C3 | **FIXED** | "**NORMATIVE:** adding `tx:` to `isSyswEncoded` **without** adding a matching branch beside the `PassPrefix` one is the defect. The branch is the work; the prefix is not." (§2.1a), backed by new refusal R12 and P3's sequencing line. |
| I1 | **FIXED** | "**NORMATIVE:** two changes, not one. (a) `syswPayloadMenu` gains content-derived entries; (b) **the boot path must invoke it on a successful load**, which is a new call `uiFlow` does not make today." (§3.3) |
| I2 | **FIXED** | "**NORMATIVE:** the enum and this case list are **lockstep sites**." (§3.1a), backed by P4's sequencing line naming `layoutMainPlates`' case list explicitly. |
| I3 | **FIXED** | "So the exemption belongs to **character devices**, not to FIFOs" (§2.5), with the mode table showing named FIFO (0666) as leaking and anonymous pipe (0600) as not. Matches the already-shipped code fix, verified present at `me`'s `609b4b4` and `mt`'s `3b494d6`. |
| I4 | **PARTIAL** | New §1.2 correctly separates the two transports and states "R6 as first drafted would have `me` print 'fits NFC' for a container that can never travel that way" — but §2.3 item 2, **untouched by this diff**, still reads: *"so a large transaction is **picotool-only**, and `me sysw pack` MUST say which transports its output fits."* That is the exact sentence §1.2 itself calls "nearly meaningless as written." The fold diagnosed the defect in the new section and never corrected the original one. |
| I5 | **PARTIAL** | §3.6a honestly opens the gap rather than papering over it: "**OPEN — this spec does not resolve it**... [O11] **unresolved; blocks a multi-transaction chunks payload**." But no new refusal and no §7 closure item actually blocks a payload holding more than one chunks-form transaction from reaching implementation — the concrete hazard I5 constructed (committing 22–202 plates to the wrong transaction with no discriminator) is still reachable by a build that satisfies every stated closure gate. |
| I6 | **FIXED** | "**NORMATIVE: the XOR is PER TRANSACTION, not per payload.** R4 refuses a **single `tx:` record** carrying both forms. A payload holding a raw record and a chunks record is well-formed." (R4′) |
| I7 | **FIXED** | "**NORMATIVE, and it is a REPLACEMENT, not an addition.**... **The `me sysw pack` line must go.**" (§3.2), and the mock-up screen no longer shows it. |
| I8 | **FIXED** | "**NORMATIVE: P5 must reorder it, and P5's gate must assert the emission order** — not merely that a finished plate looks right, since a finished plate looks identical either way." (§4.4a) |
| M1 | **FIXED** | "**The cost is NOT zero (R0 M1):** `codex32.ValidMD`/`ValidMK` hard-code the `md`/`mk` HRPs and BCH targets, and `mt1` has its own — so this is a new `ValidMT` over the shared GF engine, not a call to an existing predicate." (§2.2a) |
| M2 | **FIXED** | "**R0 round 0, M2 — a fourth input, and it is the largest.** The regeneration must also carry §4.5a's packed-and-computed legend reservation." (§4.6) |
| M3 | **FIXED** | "**M3, corrected.** An earlier draft wrote 'never' content-dependent. The carousel already is: `unlockPayload` is shown only when a Sealed Payload is present." (§3.1) |
| M4 | **FIXED** | Two distinct refusal messages, one per state, in R11′ (§5). |

**Tally: 12 FIXED / 3 PARTIAL (C1, I4, I5) / 0 NOT FIXED.**

---

## Part 2 — new defects introduced by the fold

### [Important] `uiFlow`'s main program-dispatch switch has no case for the new program — nothing in the fold's own "lockstep sites" audit names it

**Severity:** Important

**Where:** `gui/gui.go:2029-2069` (`uiFlow`'s `switch act.prog { ... }`), contrasted
with the fold's §3.1a, which audits exactly this class of switch for
`layoutMainPlates` (`gui/gui.go:2429-2436`) but not for this one.

**The failure, concretely:** the switch that actually launches a program's flow —
`case engraveMultisig: engraveMultisigFlow(ctx, th); continue`, one arm per
program — has **no `default` and no compile-time exhaustiveness check**, same as
`layoutMainPlates`. Traced the fall-through: if `act.prog == engraveTransaction`
and no arm is added, the switch body executes nothing, `obj` stays `nil`,
control reaches `if !engraveObjectFlow(ctx, th, obj) { s.Status =
scanUnknownFormat }`. `engraveObjectFlow` (`gui/gui.go:2467-2491`) type-switches
on `obj any`; `nil` matches none of its cases and hits `default: return false`.
So the operator selects **Engrave Transaction** from the carousel and the screen
reports **`scanUnknownFormat`** — the same "unknown format" message a bad scan
produces — every time, with no path to the feature this whole spec exists to
build.

**Why the fold permits it:** §3.1a's own framing — *"**NORMATIVE:** the enum and
this case list are lockstep sites"* — names exactly two sites (the `program`
enum and `layoutMainPlates`) and stops. This switch is at least as central (it
is what makes the carousel entry do anything at all, versus `layoutMainPlates`
which only affects the background image behind an already-broken screen), yet
it receives none of the exact-citation treatment the fold gave the panic. P4's
sequencing line — *"The payload menu (§3.3) **and the boot-path call that
invokes it**; the program (§3.4-3.7); `layoutMainPlates`' case list (§3.1a)"* —
could be read as covering it under "the program," but that is exactly the kind
of "an implementer would obviously do the reasonable thing" reasoning C1/C2/C3/I1/I2
already falsified this round.

**Confidence:** high on the mechanism (both switches read in full, the `nil`
path traced through `engraveObjectFlow` to its `default` arm). Medium on whether
an implementer would add the case unprompted while writing the program itself —
plausible, but that plausibility argument is the same one the round-0 findings
kept disproving.

---

### [Minor] `StartScreen.draw`'s title switch also has no case for the new program

**Severity:** Minor

**Where:** `gui/gui.go:2186-2209` (`func (m *StartScreen) draw`, `switch m.prog {
... }` populating `titleTxt`).

**The failure, concretely:** this switch sets `titleTxt` per program and has **no
default arm**; Go's zero value for `string` is `""`. If `engraveTransaction` is
not added here, the carousel page for the new program renders with `titleTxt =
""` — a blank title bar — while the rest of the screen (icon via
`layoutMainPlates`, once I2 is fixed) looks normal. Not a crash and not a dead
button, just a silent gap the fold's "lockstep sites" language does not name
either.

**Confidence:** high on the code (quoted above); low on operational significance,
which is why it is Minor rather than Important.

---

### [Minor] The Structured-Append "16-symbol cap is not a constraint" reassurance cites a bound that contradicts its own conclusion, and omits the bound that actually makes the conclusion true

**Severity:** Minor

**Where:** §4.2a: *"The 16-symbol cap is not a constraint here: at 1,367 B per
full-area v26 symbol that is ~21 KB, against Bitcoin's ~100 KB standardness
limit and a pathological worst case of 8,067 B."*

**The failure, concretely:** 16 symbols × 1,367 B (the spec's own v26/L figure,
verified against `design/measurements/RESULTS_qr_physical_max_2026-08-22.txt:21`)
= **~21.9 KB**. The sentence compares that to "Bitcoin's ~100 KB standardness
limit" — but 21.9 KB is *smaller* than 100 KB, so citing the 100 KB figure as
supporting evidence for "not a constraint" is backwards: taken at face value, a
transaction anywhere near that 100 KB ceiling would need roughly **4–5× more
symbols than Structured Append supports**, which is the opposite conclusion.

The reasoning that actually makes the claim true is not in this sentence: §2.3
already caps any raw transaction the container format can carry at **16,367
raw bytes** (`MaxSectionLen = 32,734` hex chars ÷ 2, enforced by R6), and
16,367 ÷ 1,367 ≈ 12 symbols — safely under 16. That is the real, load-bearing
bound, and it is not cross-referenced here.

**Why the fold permits it:** the sentence was written as a sanity check against
Bitcoin's general standardness policy rather than against this container's own
already-ruled section cap, and nobody re-derived whether the two numbers agree
before publishing the "not a constraint" conclusion. §2.3's cap has already moved
once in this document (8,191 → 32,734); if it moves again without anyone
re-checking this connection, the untested assumption behind "not a constraint"
could become false with nothing here to catch it.

**Confidence:** high that the sentence's own numbers don't support its own
conclusion (arithmetic above). Medium-high that the true governing bound is
§2.3's `MaxSectionLen`-derived cap rather than the cited 100 KB figure — this
was independently re-derived, not assumed, but I did not verify that "Bitcoin's
~100 KB standardness limit" is itself the figure the author intended (it may be
a rough recollection of policy limits that are more nuanced in reality, which
would only strengthen this finding).

---

## Part 3 — spot-checks of the controller's gate

All four claims from the fold commit's "BUILD GATE ON THE FOLD" section were
independently re-run rather than trusted:

1. **"15/15 R0 findings present in the BODY, not just the rulings table."**
   Ran `grep -c` for each of C1–C3/I1–I8/M1–M4 against the current file body:
   every tag appears at least once (C1: 3, C2: 3, C3: 4, I1–I7: 1 each, I8: 2,
   M1–M4: 1 each), and each occurrence was independently confirmed, during the
   read of the full document, to sit inside the relevant `####`-subheading
   section rather than in one summary list. **Held.**
2. **"placeholders TBD/TODO/FIXME/XXX → none."** Ran a case-insensitive grep for
   all four tokens: the only hit is `X.XXXXXXXX BTC` (§3.5), a decimal-format
   placeholder in an example screen line, not an unfinished-work marker.
   **Held.**
3. **"dangling internal section refs → none (8.2f/6.2/6.4/10.14 are
   cross-document and attributed)."** Extracted every `§N(.N[a])` token (35
   distinct forms) and diffed against the document's actual header list (every
   `##`/`###`/`####` heading). All resolve to real internal headers except
   `§8.2f`, `§6.2`, `§6.4`, `§10.14`, which are explicitly attributed to `mt`'s
   spec and `SPEC_encrypted_payload_delivery.md` / `SPEC_mt_qr_DEFERRED.md` in
   the surrounding prose. **Held.**
4. **"retracted-claim scan → 6 hits, all inside explicit quotes, 0 asserted as
   fact."** Did not reproduce the exact count of 6 (the controller's search
   terms are unknown), but independently grepped a broader set of retracted
   phrases — `"nothing new"`, `"never"` (content-dependent), `8191`, `49-bit`,
   `64-chunk`, `6 lines`/`25.5 mm`, `"No file mode exists"`, `"names no
   command"` — across all their occurrences in the current file. Every hit is
   either (a) inside an explicit quotation marked as an earlier draft's claim,
   or (b) a still-true fact stated in the present tense (`8191` legitimately
   remains the NFC scan-buffer bound, cited as current and correct in §1.2/§2.3).
   None were found asserted as current fact while being false. **Held**, on the
   broader set checked, though the specific count of 6 was not verified.

**No spot-check failed.** The one adjacent thing this pass *did* catch that the
controller's four listed checks would not have — §2.3 item 2 left unedited and
now contradicting the new §1.2 (Part 1, I4) — is a propagation gap, not a
placeholder/dangling-ref/retracted-quote problem, so it sits outside the scope
of the four claims checked here by design, not because those checks are wrong.

---

## Verdict

**Part 1: 12 FIXED / 3 PARTIAL (C1, I4, I5) / 0 NOT FIXED.**

**Part 2: 0 Critical / 1 Important / 2 Minor new defects** (the `uiFlow`
dispatch-switch gap; the `StartScreen.draw` title-switch gap; the Structured
Append 16-symbol/100 KB reasoning inconsistency).

**Part 3: all four of the controller's stated gate checks held** under
independent re-derivation, on the terms available (exact search terms for the
6-hit retracted-claim count were not known and so not reproduced verbatim).

**Not examined this round:**

- §4.5's underlying QR-capacity-vs-millimetre arithmetic in §4.5a's table
  (v16/v19/v21/v26 at given module counts) — spot-checked the outer bounds (v26
  at 79 mm, v16 at 53.5 mm) as internally plausible but did not independently
  derive QR capacity tables to confirm v19/v21 to the byte.
  `RESULTS_qr_physical_max_2026-08-22.txt` and
  `RESULTS_ecc_selection_2026-08-22.txt` were spot-checked and their cited
  figures (1,367 B v26/L; the 742 B → 6 qr/2 pl case; the 1,130 B → 2 pl/1 qr
  case) matched exactly.
- The `mt` side of P2 (raw-transaction `inspect` subject, NFC-fit reporting) —
  not built yet, nothing to check against source.
- Whether `me sysw pack`'s stdin path (§1.1, unchanged by this fold) still
  matches its own measured examples — out of this round's diff, not re-verified.
- A full enumeration of every switch over the `program` type in `gui/gui.go` —
  three were found (`layoutMainPlates`, `uiFlow`'s dispatch switch,
  `StartScreen.draw`'s title switch); a fourth or fifth was not ruled out by an
  exhaustive search of the file, only by targeted greps for `case
  backupWallet` / `switch page` / `switch m.prog` patterns.
