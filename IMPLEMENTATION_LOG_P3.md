# IMPLEMENTATION LOG — phase P3a

**Scope:** G-P3.3 through G-P3.18 and G-P3.20 of
`design/ACCEPTANCE_engrave_transaction.md` §6, plus verification of G-P3.1 and
G-P3.2. **Out of scope by instruction:** G-P3.10, G-P3.14, G-P3.19 — all three
need an operator ruling in the journey walk that follows this phase, and their
sheet rows are left intact.

**Result: 17 gates closed with tests, 0 skipped.** G-P3.1 turned out to be only
half-done and was completed. Six defects were found that no gate asked for,
three of them Critical, and two of the sheet's own assertions were measured
false.

**Branches:** `p3/ui-walk` in both worktrees. Nine commits in
`mnemonic-engrave`, seven in `seedhammer`. Nothing pushed.

## Final verification

Logs read, not exit codes.

```
me:    cargo nextest run --locked --no-fail-fast  -> 366 tests run: 366 passed, 1 skipped
       cargo clippy --all-targets --locked        -> 0 warnings
fork:  scripts/gui-shard-test.sh ./gui/ 24        -> all 982 tests ran across 24 shards, ok
       go test $(go list ./... | grep -v /gui$)   -> every package ok
sheet: scripts/acceptance-count.py                -> clean
```

Baselines at the start of the phase: `me` 334 passed; `gui` 932 tests. So this
phase added **32 Rust tests and 50 `gui` tests**.

> **Two `me` gates had been skipping silently.** `cross_lang` and
> `preview_cross_lang` — the constellation's Rust↔Go seam tests — `return`
> early when `go` is not on `PATH`, and `go` is not on `PATH` on this box by
> default. They also need `third_party/seedhammer`, which `git worktree add`
> does not create. The 366 above is with both present, so those two RAN. Same
> class as G-P5.8; recorded, not fixed, because changing a skip into a failure
> is CI's business and G-P5.8 owns it.

---

# Part 1 — the gates

## G-P3.3 — `--allow-unsigned-inputs`  · CLOSED

**Was:** the flag did not exist; clap rejected it. Overdue from P0.

**Did:** `TxSummary` gained `unsigned_inputs: Vec<usize>`, and
`every_input_signed` is now *defined* from it so the verdict and the indices
the messages print cannot drift. `sysw::Admission` is the seam
(`classify_with` / `pack_with` / `pack_deterministic_with`); `classify` and
`pack` keep their signatures and delegate. The refusal, the override warning
and `me sysw show` all name the failing inputs by number.

**Scope, stated because a reviewer will ask.** It loosens nothing else — the
body must still be lowercase hex and still parse — and it is *silent* on a
fully-signed transaction, because a flag that shouts on every payload trains
the operator to ignore the one payload where it matters. It deliberately does
**not** reach the `mt1` chunk class: nothing there refuses (ruling 2026-08-25b)
and the device recomputes confirmation for itself, so a host flag reporting
"confirmed" could only make the two disagree.

**Tests:** 6 in `tests/sysw_cli.rs` + 1 in `sysw/tx.rs`. RED first — clap
rejected the flag on 4 of 6.

**Mutation:** per-input → whole-transaction predicate reddens 4; the flag
ignored inside `classify` reddens 3.

**Found on the way:** `me sysw show` classified strictly and therefore said
*nothing at all* about a record the container demonstrably held — a 229-byte
public section with no account of what was in it. A reader may disagree with
the writer; it may not go quiet.

## G-P3.4 — `me sysw pack` reads stdin  · CLOSED

**Was, measured:** `printf 'text:6869\n' | me sysw pack --no-passphrase` →
exit 2, *"no records: pass them on argv or with --in"*. SPEC §1.1's ruled
pipeline could not be typed.

**Did:** precedence `--in` > argv > **stdin**, so no existing invocation
changes meaning. Blank lines skipped on stdin exactly as with `--in`, so the
two channels cannot disagree about what record 3 is. R7 preserved rather than
bypassed: empty *or whitespace-only* stdin lands on the same exit-2 path,
because `fish` reports a pipeline's status as the last command's and an
upstream `mt encode` failure otherwise arrives as silence. A TTY stdin says
what it is waiting for before it blocks — the "looks like a hang" shape a
journey walk already found in the sibling.

**Tests:** 5. RED first, 3 of 5 for the stated reason.
**Mutation:** `if recs.is_empty()` → `if false` reddens 2; dropping argv
precedence reddens 1.

## G-P3.5 — a `tx:` record on argv is refused  · CLOSED

**Was:** accepted unconditionally. On the default path `me sysw pack tx:<hex>`
**generated a passphrase, told the operator to write it down, and then wrote
the container to stdout at exit 0.** The RED run captured both.

**Did:** the guard is the first thing `pack` does — a guard downstream of the
work it exists to prevent has already lost. **Exit 3**, policy refusal: the
record is understood and well-formed and this tool will never take it *here*;
the same record on `--in` or stdin still packs, byte-identically. Matched on
the trimmed, case-folded prefix rather than through `classify`, so ` TX:<hex>`
is refused for the bearer reason rather than three screens later for a
formatting one. Neither message names the body.

**Tests:** 5, including one asserting the refusal echoes neither the
transaction nor a generated passphrase. Three existing tests moved from argv to
stdin so R1/R9's messages and `show`'s `tx:` row keep their coverage on the
channel a real operator uses. The exit-code vocabulary table gained a row.

**Mutation:** disabling the prefix match reddens 4.

## G-P3.6 — sealing is decided by CONTENT  · CLOSED

**Was:** `let sealing = !*no_passphrase;` — right for a mnemonic, wrong for a
transaction, and contrary to the operator's 2026-08-23 ruling *"send via
payload unencrypted"*.

**Did:** seal iff some record is `Class::is_secret()`. The flags choose only
*how*. §2.4's second sentence is the harder half and is implemented: `pack`
says which way it went, and why, **every time**.

**A defect surfaced doing it, and it predates the gate:**

```
$ me sysw pack --passphrase-words 4 <md1>
passphrase — write this down and store it APART from the machine: …
$ me sysw show p.bin
sealed:   false
ct_len:   0
```

`pack` moves only *secret* records into the ciphertext, so with none there the
plaintext is empty, `sealed()` is `ct_len > 0`, and the 16-byte AEAD tag lands
past `total_len()` where nothing authenticates it. The operator was told to
keep, forever, a passphrase that protected nothing and opened nothing. My first
draft of the new message would have asserted that protection *in words*. The
flag is reported IGNORED now, with the reason.

**Test:** `what_pack_says_about_sealing_is_what_show_reads_back` — over six
invocations, what `pack` SAYS must equal what `show` READS BACK, and a
passphrase is minted only for a container it opens. **A message cannot lie if a
second program has to agree with it.**

**Mutation:** letting the flag "seal" a secret-free payload reddens it with
exactly that sentence.

## G-P3.7 — "loudly" means the set, and every missing string  · CLOSED

**Was:** one line per record saying only *"an mt1 chunk whose set this tool
could not confirm"*. No set id, no indices. An operator holding 201 of 202
strings was told nothing about which one to find — the r7-M1 the ruling names.

**Did:** `sysw::mt::set_problems` groups once and diagnoses per set.
`SetProblem` distinguishes **five** failures whose remedies are not close:
`Missing{count,missing}` · `DoesNotReassemble` · `NotATransaction` ·
`TxidDoesNotBind` · `UnsignedInputs`. The last is invisible to every other
check in the walk — stripping the witnesses leaves the txid unchanged, so the
set still binds.

```
me: mt1 set 2dcf2 (records 0, 1, 2, as given; records count from 0) did NOT
      confirm as one signed transaction. MISSING strings 2, 4 and 5 of 6.
```

`me sysw show` carries the same diagnosis, marked INCOMPLETE — a stderr line is
gone in a week and `show` is the one an operator can re-run. Chunk numbers are
**1-based** on every operator surface, `mt`'s own convention.

**Follow-on, same day:** `set_confirmed` and `decode_confirmed` were two copies
of one walk, with a doc comment promising they agreed — which is how the G-P3.1
predicate came to be added to each separately. Both ask `diagnose` now, so
"None exactly when the set confirms" is structural rather than a promise.

**Mutation:** `.take(1)` on the missing list reddens 3. **It reddened only 2 at
first**: the CLI assertion checked for the digits `"2"`, `"4"`, `"5"`, `"6"`
one at a time — all of which also occur in `"set 2dcf2 (records 0, 1, 2"`. A
digit-at-a-time check passed exactly the defect the ruling names. Tightened to
the whole phrase.

## G-P3.8 — the four lockstep sites  · CLOSED

Adding a program means editing four places and **three fail silently**: a
missing title case leaves the bar blank, a missing `layoutMainPlates` case
panics at draw time, a missing `engraveObjectFlow` case drops the scan.
`gui/scan_test.go` — the tree's one place that drives a string *through* the
scanner — had no `mt1` row either, so the branch deciding the carrier type, and
therefore the routing, was exercised by nothing.

Five tests. All passed the moment they were written, which proves nothing, so
each site was deleted in turn: `mtText` case → 1 red, title case → 1,
`layoutMainPlates` → 2, scanner `ValidMT` branch → 2.
`TestEveryNavigableProgramHasATitleAndAPlate` sweeps `0..lastNav()` rather than
naming one program, so the next program added is covered without anyone
remembering.

## G-P3.9 — one condition, one behaviour  · CLOSED

**Was:** a complete-but-non-decoding set was *offered and engraveable* from the
payload and **dropped** from the NFC gather — *"Set complete but does not
confirm as one transaction. Dropped."* — throwing away every string the
operator had just scanned, off tags, one at a time, in direct contradiction of
ruling 2026-08-25b that the payload path three functions up obeyed.

**Did:** `substitutionFor` is the one function both paths ask. The gather's
decision moved out of the frame loop into `txGather.offer` — it was a closure
inside a screen loop, so the line that decided a broken set's fate could not be
reached by any test, which is why the divergence survived.

A **third** substitution fell out: a set that reassembles, parses and *binds*
and still cannot be broadcast is not "DOES NOT DECODE". `mt.ErrUnsignedInputs`
is exported and earns `legendUnsigned`.

**Scoped to COMPLETE sets, and the scoping is a finding, not a convenience.**
An incomplete set is not a divergence: the payload is finite so the payload
path knows the set will never grow, while the gather's operator can always
present another tag — "String 3 of 6. 3 to go." is the right answer there.
**What that leaves open:** an operator holding 3 of 6 tags has no way to engrave
the three *from the gather*, though the payload path offers exactly that.
Closing it means a second button inside a live scanning loop and changes what
Back means — operator-shaped, not mechanical. **Filed for the walk.**

**Mutation:** restoring the drop reddens both parity subtests; collapsing three
substitutions into two reddens the naming test.

## G-P3.11 / G-P3.12 / G-P3.13 — three message gates  · CLOSED

**G-P3.11.** R11′'s third branch tested, plus the orphan suffix. **Recorded:**
the branch is *defensive*, not reachable from the load flow (`syswLoadFlow`
nils an uncompared session) — which is exactly why it had no test: nobody could
get to it to notice it was wrong.

**G-P3.12.** R16 now reads

```
20064 bytes is too large for QR plates.

At 0.6mm modules - the smallest this machine cuts - 16 Structured Append
symbols at ECC M hold at most 17968 bytes.

Use TEXT plates.
```

The ceiling is **measured by search**: it depends on plate geometry, stroke
width *and* txqr's version selection, so a constant would go stale silently in
a message nobody reads until the day it matters. Two tests check N fits and
N+1 does not.

- **The first draft printed a ceiling of 0.** `EncodeSet` refuses a payload it
  cannot split into 16 non-empty parts, so `fits` is false at the bottom for a
  reason that has nothing to do with the plate, and a bottom-up doubling search
  never leaves the ground.
- **R16 is unreachable through the container.** QR ceiling 17,968 B; the
  largest `tx:` record a section can carry is (32,734−3)/2 = **16,365 B**,
  because the body is hex. `me sysw pack` refuses first, naming the section
  cap. It stays as defence in depth for the NFC gather;
  `TestTheQRCeilingIsAboveWhatTheContainerCanDeliver` asserts the relation so
  the day it inverts is a failing test rather than a surprise.

`eccName` and `moduleLabel` are one table each now — they appear on the plate
legend, in the plan note and in the refusal, and three copies is three chances
to disagree about the units two operators are comparing.

**G-P3.13.** The stop screen says **DISCARD**, says why (*"half cut and nothing
will finish it"*), and says what keeping it costs: a re-run mints
byte-identical plates *from plate 1*, so the drawer ends up with two plates
numbered n/m that are not the same, on a machine with no camera to tell them
apart.

**Mutation:** four, one per change, each reddening only its own test.

## G-P3.15 — the payload menu, and the moment it appears  · CLOSED

Two changes, because §3.3 is two sentences.

**(a)** The lead reads *"Loaded. It holds: 6 mt1 chunk, 1 free text."* — the
ruling's own example is *"this payload holds: 1 transaction, 2 seeds"*. One
content-derived **entry**, ENGRAVE TRANSACTION, appears only when the payload
holds a class `progTransaction` admits — asked through the admission table, not
a second list of what that program eats.

**(b)** `uiFlow` invokes the menu on a successful boot load. It called
`syswLoadFlow` directly and returned to the carousel, so `syswPayloadMenu` —
documented in its own file as *"the Load Payload carousel entry"* — was
reachable only by navigating there afterwards, never at the moment the ruling
names. The spec filed this as a defect for a reason worth repeating: **a gate
reading "the payload menu exists and lists what the payload holds" passes on
(a) alone while the ruled behaviour stays untrue.**

BACK is the exit, as §3.3 requires; the menu appears unbidden now.
`TestTheBootLoadEndsAtThePayloadMenu` drives the **real** `uiFlow` from
power-on over a real region, because calling the menu directly is exactly what
cannot tell (a) from (b).

**Mutation:** reverting boot to the direct call reddens the boot walk; dropping
the inventory reddens two; offering the entry unconditionally reddens its own.

## G-P3.16 — one command, named on both sides of the air gap  · CLOSED

The compare screen said *"Compare this against what `me sysw pack` printed"* —
the WRITE path. Re-running `pack` means re-supplying every record and re-running
the ceremony, and on the sealed path it mints a **fresh passphrase**. The
operator standing at the machine has the FILE.

Both sides name `me sysw show <file>` now, and `pack` fills the path in so it
can be pasted; on stdout it says *"the file you just wrote"* rather than
inventing one. `the_named_command_prints_the_same_digest` **runs** the named
command and compares — a pointer to a command that prints something else is
worse than no pointer, because the operator compares two numbers that were
never meant to match and concludes the payload is tampered with.

## G-P3.17 / G-P3.18 — the instruction and the clock  · CLOSED

**G-P3.17(a).** §4.3a: the per-plate instruction is a function of what is on
*that* plate. The legend-only layout — chosen whenever P−1 symbols plus a
legend plate beats P symbols inline — cut *"scan all qr, any order"* into the
one plate with **no symbol on it**. `transactionLegend` takes `plateHasQR`: the
legend plate says where the symbols are, an inline one says scan these, a
single-symbol plate does not mention order, because there is none.

**G-P3.17(b).** A per-job screen after the last plate — the only place the
device can say TEST THE PLATE, since it never tests one itself: the SH2 has no
camera, so if the operator does not check the artifact now nobody checks it
until the day it is needed. One command per plate kind, named once.

> **It was a modal and the modal truncated it.** `showNotice` → `ErrorScreen`
> does not page: the body stopped mid-sentence at *"Order does not matter — it
> is inside"*, so the two lines that matter most — what to check the txid
> against, and that this machine can never read a plate back — were
> **unreachable, with three assertions on their wording passing**. That is
> F-151's shape one step along: there, text submitted and not drawn; here,
> drawn and not shown. It pages now, and
> `TestEveryLineOfThePostCutScreenIsReachableOnThePanel` pages the real screen
> and asserts every line arrives — with a control proving that walk can fail.

**Scoped, recorded:** TEXT plates gain no on-plate instruction. A line trades
directly against the brief's own stated priority (fewest plates), and the `mt1`
hrp is self-describing to every constellation tool. Their instruction is the
post-cut screen.

**Depends on G-P5.7:** `mt inspect`'s raw subject is unmerged, so the command
the screen names is not yet on `mnemonic-transaction@main`.

**G-P3.18.** The code claimed plate count and ECC were *"the two numbers the
operator budgeted blanks and TIME by"* while no time appeared anywhere. At ~21
minutes a plate a four-plate job is most of an afternoon, and stopping mid-set
now costs a blank (G-P3.13). `transactionJobTime` sums `Plate.Duration` over
`TicksPerSecond` — **the same clock the live remaining-time readout uses**,
because two clocks for one machine would disagree in front of the operator —
and says "unknown" at tps 0 rather than dividing by it on a confirm screen.

## G-P3.20 — the end-to-end UI walk  · CLOSED

`runUI`/`runUITouch` are used in 39 other test files in this package. Not one
was this program's. Every transaction test called a planner, a formatter or a
predicate, so the spine —

```
choice → review → plate kind → plan confirm → engrave loop → post-cut
```

— was exercised by **nothing**.

**Five walks** drive the real flow through the real screens and finish the
engrave through a real `EngraveScreen`: the QR path from a `tx:` record, the
TEXT path from a confirmed set, the two legend-substitution paths (incomplete
set; unsigned `tx:` record), and the picker.

**Four goldens** of the plates themselves — `tx-qr`, `tx-text`,
`tx-unconfirmed`, `tx-unsigned-qr` — through the real planner and `toPlate`.
Every other assertion here is about strings, and a legend can carry the right
words while sitting on top of the QR symbol. Mutation-proven: reverting the
substituted legend reddens `tx-unconfirmed` (9,702 knots vs 11,114), and
dropping it from the QR legend reddens `tx-unsigned-qr` — so a warning lost
between the review screen's promise and the steel is caught **at the artifact**,
not at a string.

**Journey:** `design/JOURNEY_engrave_transaction.md`, produced by
`scripts/gen-tx-journey.sh`. Every host block is that script's real
stdout+stderr with its actual exit code; every device screen comes from
`gui.TestCaptureTransactionJourney`, which is **the walk, instrumented** — so
document and test cannot drift. **Verified regenerable: two consecutive runs
are byte-identical**, which is the fix for F-156's class applied before the
document is published rather than after somebody tries.

> **Its one limit, stated in the document itself.** The frames are
> `op.Drawer.ExtractText` over the firmware's own op tree, **not** the
> emulator's 480×320 framebuffer the way `design/journeys/*.pdf` are. Same
> tree, same text, different renderer: it shows **what** the device says and
> cannot show how it **looks**. The framebuffer capture needs a WASM build and
> playwright, and it belongs with the P4 hardware session — beside a photograph
> of real steel. Recorded rather than quietly omitted.

---

# Part 2 — G-P3.1 and G-P3.2, verified

Instruction: *"already closed, do not redo — verify and mark them closed."*
Verification found **G-P3.1 was half-open**, and its gate text names the half
that was missing.

**G-P3.2 — verified closed.** Rust: `sysw::mt::diagnose`, now the single
implementation both `mt_unconfirmed` and `decode_confirmed` ask. Go:
`mt.Decode` → `ErrUnsignedInputs`. Mutation-tested on the collapsed
implementation: disabling it reddens 3.

**G-P3.1 — the `tx:` RECORD half was not done.** Its done-condition reads *"a
`tx:` record **or** reassembled set with an unsigned input reaches the device
flagged with the mandatory legend substitution"*. Only the set half existed.
`sysw.Classify` (Go) requires a structural parse and nothing more, so an
unsigned `tx:` record was `ClassTx`, reached `payloadTransactions`, and became a
candidate **with no flag of any kind** — carrying the honest transaction's
txid, because stripping signatures is precisely what the txid ignores. The
sheet's §5 check 1 was unmet for the class that carries the whole QR path.

Closed as part of the `tx:`-record work: flagged, not refused — it reaches the
device only through `--allow-unsigned-inputs`, a deliberate operator act — with
`legendUnsigned`, the failing input named on the review screen, and the
substitution carried onto the plate.

---

# Part 3 — six defects no gate asked for

Five came from **executing** the thing; one from writing a message that would
have had to be true. All are in the sheet's new §7.

| # | severity | what |
| --- | --- | --- |
| F1 | **Critical** | **The `tx:` record path was inert.** `confirmed` was never set, and its zero value is false — so a signed transaction got the *"UNCONFIRMED SET / Set 00000, 0 string(s) / QR plates are unavailable"* screen, then `transactionReviewAndEngrave` found no TEXT (no strings) and no QR (`!confirmed`), so `len(choices)==0` and it **returned silently**. `me tx \| me sysw pack` → device produced nothing, with no screen at all. 16 transaction tests green throughout; not one drove a `tx:` record past `payloadTransactions`. |
| F2 | **Critical** | **The program PANICKED** — `slice bounds out of range [:8] with length 0`. The picker built a row per candidate reading `c.tx.TxidDisplay[:8]`; an unconfirmed candidate carries the zero-value `mt.Tx`. Rows are built for **all** candidates *before* `len(choices) > 1` decides whether to show the screen, so **one incomplete `mt1` set crashed the program with no picker ever displayed**. Live since the ruling-2026-08-25 fold (verified at the phase's base commit `82fad4a`), and the triggering payload is the ordinary one that ruling exists FOR. The sheet recorded the row as MET. |
| F3 | **Critical** | **No signature predicate on the `tx:` class on the device** — G-P3.1's other half, above. |
| F4 | Important | **The BEARER warning was below the fold.** `confirmReviewScreen` pages, and the warning sat last, so page 1 held the question and the four txid lines and nothing else: an operator pressing Continue from the screen showing the number they came to check never saw it. A **position**, not a wording — which is why every assertion on the sentence passed. Moved above the txid. |
| F5 | Important | **`--passphrase-words` on a secret-free payload minted a passphrase for a cleartext container.** See G-P3.6. |
| F6 | Important | **The post-cut screen was truncated by its own modal.** See G-P3.17. |

## Two things the sheet asserted that are false

1. **§4.1 R10 and G-P3.10.** The sheet says duplicate candidates *"merge on
   **bytes**, not on the txid … different ones stay two candidates"*, and files
   G-P3.10 as *"two byte-different transactions sharing a derived txid present
   as two identical picker rows"*. **The merge reads `c.tx.TxidDisplay`.** A
   byte-different transaction sharing a txid is **dropped, not duplicated** —
   and the pair that does it is not exotic: a transaction and its own
   signature-stripped form share a txid by construction.
   `TestTheMergeIsKeyedOnTheTxidNotOnTheBytes` pins today's behaviour
   **without changing it**, so G-P3.10's operator ruling starts from what the
   code does rather than from what the sheet says it does. *(G-P3.10 is out of
   scope by instruction.)*
2. **R16 is unreachable through the container.** See G-P3.12.

## For P4

The pinned 222-byte vector plans to **ECC H at 0.6 mm modules**, not 0.9 mm.
That is L11 working exactly as specified — ECC outranks module size — but it
means the **default** plate for the constellation's own reference transaction
uses the smaller of the two faces, and **0.6 mm has never been read off steel**.
G-P4.1 should cut *this* plate.

---

# Part 4 — what I did NOT do, and why

- **G-P3.10, G-P3.14, G-P3.19** — out of scope by instruction; sheet rows left
  intact. G-P3.10's premise is corrected above so the ruling starts from
  measured fact. G-P3.14's row gains a P3a note: the *unsigned* review screen
  now does carry the txid's limit (*"the txid above is the same one a signed
  version would have"*), so the shape exists; the confirmed screen still does
  not.
- **Engraving a partial set gathered over NFC** — the payload path offers it,
  the gather does not. It needs a second button inside a live scanning loop and
  a decision about what Back then means. Recorded under G-P3.9.
- **The emulator framebuffer capture** — needs a WASM build and playwright, and
  belongs beside a photograph of steel in P4. The journey says so in its own
  text rather than leaving a reader to assume otherwise.
- **`cross_lang` / `preview_cross_lang` skipping without `go` on `PATH`** —
  recorded, not fixed. Turning a skip into a failure is a CI change and G-P5.8
  owns that class.
- **The stranded 16-byte AEAD tag** on a `ct_len == 0` "sealed" container is now
  unreachable, because that container can no longer be produced. The wire-level
  oddity itself (`total_len()` excludes the tag when `!sealed()`) is untouched —
  it is a container-spec question, not this feature's.

---

# Part 5 — commits

`mnemonic-engrave`, branch `p3/ui-walk` (9):

```
7253a33  me: `me sysw pack` reads stdin, so the ruled pipeline can be typed (G-P3.4)
4f1f1b1  me: a `tx:` record on argv is refused, before anything is emitted (G-P3.5)
60ae08e  me: --allow-unsigned-inputs, and it names which inputs (G-P3.3)
9c332e8  me: sealing is decided by CONTENT, and pack says which way and why (G-P3.6)
ee725dc  me: "loudly" means the set, and every missing string (G-P3.7)
c296bbb  me: one implementation of the mt1 confirmation walk, not two (G-P3.7 follow-on)
fc00675  me: pack points at `me sysw show <file>`, the command the device names (G-P3.16)
9a067ac  journey: engrave a transaction, host to steel, regenerable (G-P3.20)
dd79b57  acceptance: P3a's 17 closed gates, six defects no gate asked for, counts measured
```

`seedhammer`, branch `p3/ui-walk` (7):

```
67a96db  gui: the tx: record path was inert, and unsigned ones reached the plate unflagged
43c0dc5  gui: the four lockstep sites, asserted directly (G-P3.8)
30bc92d  gui: one condition, one behaviour, whichever door it came in (G-P3.9)
9c62260  gui: three message gates -- R11's third branch, R16's module size, and DISCARD
a60060e  gui: the payload menu appears at boot and says what it holds; the compare screen
         names the READ path (G-P3.15, G-P3.16)
4cee75d  gui: the end-to-end UI walk, and what it found (G-P3.20, G-P3.17, G-P3.18)
d305713  gui: the journey capture is the walk, instrumented (G-P3.20)
```

`4cee75d` carries **three** gates, and the bundling is stated in its own message
rather than hidden: the walk is what covers the other two's call sites —
**measured**, deleting the post-cut screen's call site broke no test until the
walk existed, because every post-cut assertion called the function directly.
Splitting them would have put unreachable code in one commit and its only
evidence in another.

Nothing pushed. Both worktrees clean.
