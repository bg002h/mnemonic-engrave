# SPEC — `mt qr`, DEFERRED material

> **This is not a specification. It is the QR/machine-engraving material carved
> out of `SPEC_mt_v0_1.md` on 2026-08-23, held for the cross-format QR cycle
> that `SPEC_mt_v0_1.md` §0a defers `mt qr` to.**
>
> **Why it was carved out.** `mt qr` is deferred because QR conversion is a
> concern `md1` and `mk1` share — it is not `mt`'s to settle alone. Material
> that only a deferred verb reads was making the live spec longer without
> making it more buildable, and a reader looking for what `mt` v0.1 does had to
> skip it.
>
> **What did NOT come here, and why — both were checked rather than assumed:**
>
> - **§5, the plate legend, STAYED.** Its heading said *"`mt qr` only"* and that
>   was wrong: §0a rules that **`mt encode` prints those fields on `stderr`** as
>   suggested text an operator may engrave beside hand-cut strings. The field
>   set is live in v0.1. Only its plate-AREA measurements (mm, lines per plate,
>   the ECC-level tradeoff) belong to this cycle, and they are left in place
>   beside the fields they size.
> - **§10.8 (`n of m` labelling) and §10.12 (fill vs balance) STAYED.** Both
>   read as QR material on a keyword scan and neither is: §10.8's ruling
>   explicitly *"holds for both verbs"*, and §10.12 is the fill-versus-balance
>   chunking decision that the live chunking rule in §3b derives from.
>
> **Numbering is preserved exactly.** The open questions keep their original
> numbers — they are cited from the parent spec and from commit messages, and
> renumbering would silently break both.

---


## 4. Choosing the configuration — `mt qr` only, DEFERRED (§0a)

**This section governs `mt qr` and nothing else.** `mt encode`'s layout is
undecided and is §10.10.

> **Rule (operator, 2026-08-22): the Reed-Solomon density is the highest that
> minimises plate count.**

Plate count is the real cost — ~21 minutes per plate (F-225) — so the search
minimises plates first and spends every leftover byte on error correction. Never
trade a plate for redundancy; never leave redundancy unbought.

    search space:  module size x QR version (1..40) x ECC (L,M,Q,H)
                   x rectangular tiling (across x rows)
    objective:     1. minimise plates   <- a plate holds the QR(s) AND the legend
                   2. maximise ECC
                   3. minimise symbol count
                   4. TIE-BREAK: maximise MODULE SIZE
                   5. then minimise QR version
    plate:         85 x 85 mm, outerMargin 3 mm => 79 mm usable
    quiet zone:    4 modules per side, per symbol
    legend:        6 lines reserved on plate 1 (25.5 mm at a 4.25 mm pitch),
                   1 line on every later plate for "PLATE n OF m"

> **Two corrections from R0 round 0, both in the objective above.**
>
> **The tiling is rectangular, not square.** The previous draft's search space
> said `k*k tiling` while the search that produced its own table returned 2-, 3-
> and 6-symbol configurations. None of those is a perfect square. The prose was
> wrong; `across x rows` is what the reference implementation does and what the
> numbers describe.
>
> **The objective was not a total order, and its implicit tie-break ran
> backwards.** Module size was in the search space and absent from the
> comparison key, and replacement was strict `<` against the incumbent, so ties
> resolved to whichever module the loop reached first — and the loop ascends
> from 0.30 mm. Ties therefore broke toward the **smallest and least legible**
> symbol, and would have broken toward the optically **unvalidated** 0.30 mm
> module once F-234 lifts the floor. Measured on the reference search: **4
> configurations tie** on (plates, ECC, symbols) for a 162 B payload at the
> 0.60 mm floor, and **41 tie** once the floor lifts. Steps 4 and 5 above make
> the order total, and make it break toward legibility, which is the direction
> the artifact's purpose demands.

**Measured, at the conservative 0.60 mm module, with the legend reserved**
(`RESULTS_ecc_selection_2026-08-22.txt`, the **RAW** column — the payload is now
PSBT bytes, not bytewords):

| artifact | PSBT bytes | plates, symbols, version, ECC |
| --- | --- | --- |
| RCW `tr` tier 3, 1-in/1-out | 391 | **1 plate**, 1 qr, v15, ECC M |
| RCW `tr` tier 4, 1-in/1-out | 465 | **1 plate**, 1 qr, v15, ECC L |
| RCW `tr` tier 1, 1-in/1-out | 595 | **2 plates**, 1 qr, v23, ECC Q |
| RCW `wsh` tier 3, 1-in/1-out | 626 | **2 plates**, 1 qr, v24, ECC Q |
| RCW `wsh` tier 1, 1-in/1-out | 802 | **2 plates**, 1 qr, v23, ECC M |
| RCW `tr` tier 1, 5-in/2-out | 2769 | **4 plates**, 3 qr, v21, ECC L |
| RCW `wsh` tier 1, 5-in/2-out | 3809 | **5 plates**, 4 qr, v22, ECC L |

> **This table has now been regenerated twice and is STILL provisional.** The
> first version described raw transactions under `ur:bytes`; the second,
> finalized PSBTs under `ur:psbt`; this one, finalized PSBTs with UR dropped
> (§3). Compare the second against this one for what UR cost: a plate on three
> of seven artifacts, one to two ECC levels on the rest.
>
> **Three inputs are still unmodelled here**, all of them additive, so treat
> every row as a lower bound: the **49-bit `mt1` chunk header per symbol**
> (§3), §10.8's **per-symbol `n/m` labels**, and §10.14's **font-metric
> correction** to the legend reservation. §10.14 already requires the
> regeneration; this note names all three inputs it must take.
>
> Ordinary-wallet comparisons (single-sig, 3-of-5, 9-of-11) are **not** here
> because their finalized-PSBT sizes have not been measured.

**What the legend costs, stated plainly**: reserving 25.5 mm drops small
artifacts by two or three ECC levels and doubles the plate count on the larger
ones. The rule still degrades in the right order — H → Q → M → L before it gives
up a plate — and the smallest artifact still gets meaningful ECC free.

**Module size.** 0.30 mm is one engraved stroke: the theoretical floor and
**optically unvalidated**. Whether a camera reads 0.30 mm modules off brushed
steel is a hardware question, gated on the test plate in F-234. **Until that
plate exists, 0.60 mm (two strokes) is what `mt` SUGGESTS** — not a floor it
enforces. **Operator ruling 2026-08-23 (§10.1, §8.8): the operator picks from
every size `mt` can engrave, with 0.60 mm suggested.** `mt` says at the point of
choice that finer modules are optically unvalidated; it does not refuse them.
The 0.30 mm results are recorded for when the plate exists.

> **This paragraph stated a hard floor until 2026-08-23 and was missed when the
> ruling landed.** R2 lens 1 (F-3) found it: commit `fc4179c` rewrote the rule in
> §8 item 8 and §10.1 and never touched §4, so the spec carried both the new rule
> and the old prose. **No superseded-term sweep could have caught it** — every
> word in the sentence was still current, and only the modal verb changed.


---

## Open questions carried over from `SPEC_mt_v0_1.md` §10


1. **The F-234 optical test plate has not been cut — and the module size is now
   the USER's choice.** **Operator ruling 2026-08-23: "User picks from all
   available options, suggesting 0.6."**

   So §8.8's hard refusal below 0.60 mm becomes a **default and a
   recommendation**, not a floor: `mt` offers every module size it can engrave,
   suggests 0.60 mm, and the operator decides. The test plate still wants
   cutting — it is how anyone learns what 0.30 mm actually does on steel — but it
   no longer gates the tool.

   > **The operator's second point, which generalises past this question:**
   > *"just because one size engraves and scans today doesn't guarantee in the
   > future the engraving will scan due to maintenance issues."*
   >
   > A successful scan is evidence about **one plate, one machine, one day** —
   > not a property of the module size. Machine wear, stylus condition, plate
   > stock and lighting all drift, and the artifact must survive decades of that
   > drift. This is why a test plate can **license** a size and can never
   > **certify** it, and it is an independent argument for spending slack on ECC
   > rather than on smaller modules: error correction is the only margin that
   > keeps paying after the machine has changed.


2. ~~Will a wallet reassemble multi-part UR from STATIC symbols?~~ **OUT OF
   SCOPE, operator ruling 2026-08-23: "We will add another verb in the next
   subversion to accept static scan data."**

   `mt` will ship its own reader rather than depend on third-party wallets
   stitching engraved symbols together. That retires the spec's most
   load-bearing unverified assumption by removing the dependency instead of
   testing it.

   **It also removes the main argument for UR**, and §10.3 turns on this. UR was
   defended as the only fragmentation the Bitcoin ecosystem implements; if `mt`
   supplies the reader, the ecosystem is not reassembling anything and that
   defence is void for every multi-symbol artifact.

   **What it costs, stated plainly:** F-234's promise — that a recoverer with
   none of our tools can still read the plate — now holds only for artifacts
   that fit **one** symbol. Multi-symbol recovery requires `mt`'s reader. The
   next subversion's verb is therefore not a convenience; it is what keeps
   multi-plate transactions recoverable at all, and it should be specified
   before anyone engraves a multi-symbol artifact.


3. ~~Is UR worth its expansion? What goes in the QR?~~ **CLOSED.** UR is
   dropped (§3), and the QR payload is **`mt1` chunks, bech32 UPPERCASE** —
   operator rulings 2026-08-23. Codex32-in-QR was measured and rejected at
   63–65% efficiency, worse than the UR it would replace and up to two extra
   plates. **base45 was chosen first and then REVERSED**: its alphabet contains
   SPACE, which EPD §6.4 forbids in a `sysw` record, so it could never have
   reached the machine (§3). bech32 uppercase is the only candidate satisfying
   EPD §6.4, EPD §6.6 and QR-alphanumeric packing together.

   **§10.1's test plate should still confirm scanners read bech32-uppercase QR
   symbols off engraved steel** — the encoding is decided, the optical
   validation is not.


9. ~~How does the engraving reach the machine?~~ **ANSWERED, operator ruling
   2026-08-23: "send via payload unencrypted. We have a format for transferring
   data to SH2 via USB."**

   That format is **`sysw`**, the system-wide payload already used for every
   other constellation artifact. It is Rust-primary in this repo
   (`crates/me-cli/src/sysw/`) and ported to the fork (`sysw/record.go`). A
   payload carries a **`Class`**, and the existing set is `Mnemonic`,
   `Codex32Secret`, `Passphrase`, `FreeText`, `Descriptor`, `MdMk`, `Address`,
   `Unknown` (`crates/me-cli/src/sysw/record.rs:31-40`).

   **There is no transaction class**, which is what R0 lens 4 found. Adding one
   is **necessary and not sufficient**, and **the Rust-primary rule binds**: the
   new class lands in `me-cli`'s Rust `sysw` first, with test vectors, and only
   then ports to the fork's Go.

   > **What "the work" actually is, and a correction to a claim I nearly
   > folded.** R3 lens 3 reported that a new `Class` must pass four gates
   > including `MaxRecords = 24` and `MaxRecordLen = 512`. **Those are `seal`
   > gates, not `sysw` gates** — R4 lens 2 caught the mis-attribution, and it
   > checks out: they are defined in `seal/wire.go`, while `sysw`'s own
   > `splitRecords` is a bare LF split with a UTF-8 check and no caps. The wrong
   > claim reached a persist commit and **never reached this spec**, which is
   > what persisting a report verbatim *before* folding it is for.
   >
   > **The real prerequisite is the RECORD FRAMING**, which nothing has chosen:
   > what a record's text actually contains, and how a multi-symbol artifact maps
   > onto records. Four candidate framings were costed and they give four
   > different transport ceilings (§8.7c), with the only EPD-conformant one
   > refusing §4's largest artifact. **§8.7c cannot state a threshold until this
   > is settled**, and no implementer can build `mt qr`'s output without it.

   **Unencrypted, by ruling.** Note `me` has an encrypted-payload path and this
   deliberately does not use it. The reasoning is consistent with §7: the plate
   the payload produces is **bearer** and sits in a drawer, so the wire is not
   where this artifact's secrecy lives. What the ruling does accept is that
   anyone with access to the USB link sees the transaction before it is cut.

   **Still open underneath this ruling, and it still blocks:** §4 selects an ECC
   level, a module size and a multi-symbol tiling, and the fork's only
   arbitrary-payload QR path is fixed at `freeTextQRScale = 2`
   (`backup/fit.go:19`) with a compile-time ECC level and one code per plate.
   A `sysw` class says how the bytes *arrive*; it does not make the firmware able
   to engrave what §4 chose. **That gap is now §10.17.**


17. **The firmware cannot yet engrave what §4 selects — and will be taught.**
    Operator ruling 2026-08-23: *"we will later teach SH2 how to handle
    transactions."* So this is scheduled firmware work rather than an unresolved
    design question, and §4 keeps its full search space.

    What stands today: the fork's only arbitrary-payload QR path is
    `freeTextQRScale = 2` (`backup/fit.go:19`) with a compile-time ECC level and
    one code per plate, and `sysw`'s `Class` enum has no transaction member
    (`crates/me-cli/src/sysw/record.rs:31-40`). **Until that work lands, `mt qr`
    can produce a payload that no shipped firmware will engrave.** That is a
    real limitation on the verb, not on the spec, and it should be stated
    wherever `mt qr` is documented as usable. **The Rust-primary rule binds the
    new `Class`:** it lands in `me-cli`'s Rust `sysw` with test vectors first,
    then ports to the fork's Go.

---

## Refusals carried over from `SPEC_mt_v0_1.md` §8

> **§8.7 and §8.7c are refusals only `mt qr` can trip.** They were numbered
> entries in the live refusal list while the verb that would reach them does not
> ship — and §8.7 was additionally **unrunnable as written**: R6 found its
> threshold (the operator's stated maximum plate count) has no input path, and
> a refusal whose threshold cannot be supplied is not a refusal.
>
> **§8.7b stayed in the parent.** The 4,096-chunk ceiling comes from `mt1`'s
> own header, so **both verbs share it** — it is not QR material despite sitting
> between these two.
>
> Numbers preserved: the parent keeps `7.` and `7c.` as pointers, which is also
> what keeps `7b.` from becoming an orphan suffix with no base item.

7. **Over the plate budget (`mt qr`)** → refuse, naming the exact plate count
   and what would fit. **Deferred with the verb (§0a).** **"Plate budget" means the operator's stated maximum
   plate count**, which `mt` compares against §4's search result; there is no
   fixed number, because §4's answer depends on module size, ECC and tiling.

7c. **Over the `sysw` section ceiling (`mt qr`)** → refuse. **Deferred with the
   verb (§0a); no v0.1 behaviour depends on it.** `MAX_SECTION_LEN =
   8191` (`crates/me-cli/src/sysw/wire.rs:42`), inherited from EPD. **This is a
   hard transport limit §4's search knows nothing about**, so a transaction can
   pass every plate-count check and still be unsendable.

   **This refusal cannot carry a NUMBER until the record framing is chosen, and
   two earlier attempts to give it one were both wrong — R4 lens 2.** The
   ceiling counts **record text**, so the largest admissible PSBT depends
   entirely on how a chunk is framed into a record. Four candidate framings give
   **four different ceilings — 3,671 / 4,094 / 4,476 / 4,525 B** — and none is
   the 4,537 B computed here previously.
   **The only EPD-conformant candidate refuses §4's own largest artifact by
   322 B**, which would mean the biggest wallet this spec measures cannot reach
   the machine at all.

   > **Its two previous numbers, recorded because the pattern matters more than
   > either.** First *"roughly 40% headroom"*, from comparing QR-capacity
   > **bytes** against a cap counting **characters**. Then *"15.4%, ceiling
   > ~4,537 B"*, arithmetically sound but computed against a record framing the
   > spec had never chosen. Three numbers, three unstated assumptions. The fix is
   > not to compute more carefully — it is that **§10.9's record framing is a
   > prerequisite for this refusal**, and until it is settled the refusal is
   > stated as a rule with its threshold named as pending.

   > **An earlier version of this refusal said "roughly 40% headroom", and that
   > was wrong by a units error — R3 lens 3.** It compared the artifact's
   > **QR-capacity bytes** against a cap that counts **record text characters**.
   > The mistake is instructive because it flattered the design in the same
   > commit that discovered the ceiling: a 40% margin invites "no need to model
   > this", while 15% is close enough that §4's search and this refusal must be
   > reconciled rather than left independent (§10.14's regeneration).

---

## The CLI-surface row, carried over from `SPEC_mt_v0_1.md` §10.10

| `mt qr` output | a **SH2 payload** (`sysw`) carrying the QR — machine engraving |

> Removed from the live CLI-surface table because that table's own `verbs` row
> lists `encode`, `decode`, `verify`, `inspect` and **not** `qr` — so it was
> describing the output of a verb it did not offer. It belongs with the verb.

