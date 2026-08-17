# S6b — proposal for §3 Q1, Q3, Q6

**Status: APPROVED by the operator, 2026-08-17** — *"I accept your
recommendations"* — for **Q1** and **Q6**; **Q3** was settled separately against
the primary Rust implementation (see the SETTLED block below). Recorded as
rulings **R-F** and **R-G** in `REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis,
which is the authority the spec reads.

Still not a spec, not gated, not implementable — an approved proposal is input
to a spec, and the R0 gate is untouched. Every code fact below is read from fork
`main` =
`b1479a1b38f6b045d27443764c858906e4e6e122` and cited; the groundwork is
`design/agent-reports/s6b-plate-mechanism-facts.md`, persisted verbatim in
`3922fcf` before this document was written.

**It rests on a corrected §2.** Two of §2's "measured facts" were false and are
corrected in place — `TitleString` has no production callers, and
`COMB FP: FC60 C6DF` is not a string in the source. Read §2.3's correction note
and §2.4 before this.

---

## Q1 — which plate-text mechanism do `mk1`/`md1` move to?

### The facts that decide it

1. **`backup.Text` has no title or footer capability whatsoever.** Three fields:
   `Paragraphs`, `Font`, `FontSize` (`backup/backup.go:33-41`). `EngraveText`
   (`backup/backup.go:350-446`), read in full, renders no title row and no
   footer row — every line comes from `WrapText`. There is no dead branch and
   no commented-out band to revive.
2. **Mechanisms 1 and 4 already share the layout engine.** `EngraveText` calls
   `textLayout` (`backup/wrap.go:224`) and `qrPlaceAt` (`backup/wrap.go:196`) —
   the same package-private helpers `Fitted` uses, defined once in
   `backup/wrap.go`, along with `WrapText` which both call.
3. **The passphrase band shares nothing with either.** It is a self-contained
   ~8-line closure local to `backup/passphrase.go:228-235`: fixed small font, plain
   centred `engrave.String`, **no wrap, no screw-hole clamp, no QR narrowing**.
   It is not exported and was not written to be reused.
4. **`Text`'s unbounded length is load-bearing, and the code says why.**
   `backup/backup.go:386-392`:

   > *"The descriptor and mdmk callers (validateDescriptor, validateMdmk) keep
   > an UNBOUNDED path here: they offer whichever of TEXT+QR / TEXT-ONLY /
   > QR-ONLY fit, which depends on toPlate rejecting overflow, so a maxLines
   > refusal here would silently change which variants they offer."**

   Measured thresholds from the same comment: **TEXT+QR fails first (works to
   268 chars, fails at 269)**, then QR-ONLY (641/642), TEXT-ONLY last (645/646).

### Recommendation — **add optional `Title`/`Footer` to `backup.Text`, rendered through mechanism 1's shared helpers**

Not the passphrase band (fact 3: not a shared primitive, and md1/mk1 plates
carry a QR, so they need the narrowing and screw-hole logic the band lacks).
Not routing md1/mk1 through `Fitted` either — `Fitted`'s model is one row per
string, while md1/mk1 need wrapped paragraphs, so the move is larger than it
looks and would drag one flow's machinery across four call sites.

**The fields must be OPTIONAL, and that is the whole design.** Empty title →
no row rendered → no vertical budget consumed → the plate is **byte-identical
to today**. This is what makes the rest of the cycle safe, and it falls
directly out of the operator's own rulings:

- **R-A** — marking applies only when *the set contains a seed*, so watch-only
  plates pass an empty title and are unchanged.
- **R-B** — marking is single-sig only this cycle, so the other three
  `validateMdmk` callers pass an empty title and are unchanged. **This is how
  the marking gets "conditioned, not merely located"** without teaching
  `validateMdmk` about flows: the *caller* decides, by supplying a string or
  not.

### The one hazard, and the gate for it

Adding a band consumes vertical budget, and **TEXT+QR is already the tightest
variant at 269 characters**. So a title could push a plate from "TEXT+QR fits"
to "TEXT+QR does not fit", silently changing which engraving variants the
operator is offered — precisely the failure the code comment at
`backup/backup.go:386-392` warns about, and an F-198-class silent change to what gets
cut.

**Gate:** a test that pins, for a representative `md1` and `mk1`, **which
variants are offered** — with the title empty and with it set — and fails if
the offered set changes unexpectedly. Assert the *variant set*, not the
rendering.

---

## Q3 — which identifier is the "key-id"?

### §2.6 posed a false dichotomy

It framed `md.WalletPolicyIDStub` and `mk.Header.ChunkSetID` as two candidates
for "key-id". **`ChunkSetID` is not an identity at all:**

```go
ChunkSetID  uint32                                        // mk/mk.go:50
errChunkSetIDMismatch = errors.New("mk: chunk_set_id mismatch")  // mk/mk.go:31
```

It groups **chunks of a split `md1` for reassembly** — a transport artifact.
Engraved on a plate as a "key-id" it would identify nothing a reader could use.
**Ruled out.**

And the other candidate names itself (`md/walletpolicyid.go:104-105`):

> *"WalletPolicyIDStub is the top-4 bytes of WalletPolicyId — the mk1 KEY
> card's policy_id_stub for the md1 POLICY card it belongs to."*

That is a **wallet policy** identifier — the operator's *second* term, not the
first.

### Recommendation — the two terms map to two things that already exist

The operator's R6 named **both**: *"associated key-id and wallet policy id"*.

| operator's term | proposal | status today |
| --- | --- | --- |
| **wallet policy id** | `md.WalletPolicyIDStub` — 4 bytes → 8 hex, groups `XXXX XXXX` | **missing** from the passphrase plate |
| **key-id** | the **master fingerprint** — the standard identifier for a key, and what `SeedFP`/`CombinedFP` already are | **already present** (`backup/passphrase.go:176-180`) |

**If that reading is right, R6 is mostly satisfied already** and the remaining
work is one field and one line.

### It is a small change, and `backup` stays clean

```go
type Passphrase struct {
	Passphrase string
	// SeedFP and CombinedFP are canonical 8-hex-digit fingerprints, or empty.
	SeedFP     string
	CombinedFP string
}
```

The plate takes **pre-formatted hex strings, not a descriptor**
(`backup/passphrase.go:23-29`), and its band lines are already conditional
(`if plate.SeedFP != ""`). So:

- add one optional field (8 hex, or empty) and one conditional line, mirroring
  the two that exist;
- **`backup` needs no dependency on `md`** — the caller computes the stub and
  passes hex, exactly as it already does for fingerprints;
- the standalone passphrase path, which has no descriptor, passes `""` and
  renders no line. **No forced coupling.**

**Budget caution:** the band's "at most two lines" limit is a **comment, not a
check** (`backup/passphrase.go:171-174`) — a third line renders past the 7 mm
budget and off the plate with no refusal. `topLines` already holds up to two
(SEED FP, EXPECTED COMB FP). **A third line does not fit.** So the policy id
needs either `bottomLines`, or a decision about which line it displaces —
**this is a real constraint, and it needs the Q2 spike to settle.**

### SETTLED 2026-08-17 against the PRIMARY Rust implementation

Operator: *"Key-Id: send an agent to look at mk repo code."* Done —
`mnemonic-key` at `8dc5dcbf31947762a354d165ca2350ddbb15ba28` (clean), crate
`mk-codec`, the provenance pin the Go port records. Report:
`design/agent-reports/s6b-mk-key-identifier-facts.md`, persisted in `6d0de2d`.

**`mk1` defines no "key id" / "kid" concept at all** — not in `crates/mk-codec`,
not in `design/SPEC_mk_v0_1.md`, not in the BIP draft. Zero hits. The Go port
agrees; no Rust-vs-Go disagreement. **The proposal above is confirmed:** the
only per-key identifier an `mk1` carries is the master fingerprint
(`origin_fingerprint`), so that is what "key-id" can mean.

### One phrase in the research needed correcting, and it matters for funds

The report's summary says `origin_fingerprint` is *"the BIP-32 master/seed
fingerprint, **not a passphrase-combined value**"*. **That is right about the
field's definition and wrong about its value**, and taking it literally would
put the wrong fingerprint on steel.

The Rust doc comment (`mnemonic-key/crates/mk-codec/src/key_card.rs:36-37`)
reads:

> *"Master-key fingerprint identifying the seed from which `xpub` was derived.
> Verbatim from BIP 380 origin notation `[fp/...]`."*

Under BIP-39, **a passphrase changes the master key**, so the master fingerprint
of a passphrase-derived wallet *is* what this device calls `CombinedFP`. Two
independent confirmations:

- `deriveAccountXpub(m bip39.Mnemonic, passphrase string, …) (xpub string, masterFP uint32, err error)`
  (`gui/derive.go:19`) — `masterFP` comes out of a derivation that **takes the
  passphrase**.
- §2.1's own measurement: bare seed → `73c5da0a`, same seed + passphrase →
  `fc60c6df`, and both `mk1` and `md1` differ between them.

**So: `origin_fingerprint` = `CombinedFP` when a passphrase was used, `SeedFP`
when not.** That is precisely why both sit on the passphrase plate, and it is
the mechanism §2.1 says R4 exploits — restoring the words alone yields a
fingerprint that does **not** match what the key and descriptor plates encode,
which is what makes a wrong-wallet restore self-diagnosing instead of silent.

### Two constraints the research surfaced, for the spec

1. **`origin_fingerprint` is OPTIONAL.** It is `Option<Fingerprint>`, omitted
   when bytecode-header bit 2 is unset — *"the privacy-preserving mode"*
   (`mnemonic-key/crates/mk-codec/src/key_card.rs:15-18` and `:38-39`). **An
   `mk1` may carry no fingerprint at all**, in which case a plate asserting a
   key-id has nothing on the card to bind to. The spec must say what the marking
   does in that case; under **R-D** it may not assert a binding that does not
   exist.
2. **The policy stub is FORM-AWARE, so "wallet policy id" is not one thing.**
   Per `mnemonic-key/crates/mk-codec/src/key_card.rs:25-32`, each stub is the
   top 4 bytes of *either* the **`WalletPolicyId`** (keyed wallet-policy `md1`)
   *or* the key-stable **`WalletDescriptorTemplateId`** (keyless template
   `md1`). A label reading "wallet policy id" would be false on the template
   form. The spec must either choose a label true of both, or distinguish them.

> **GATE BLIND SPOT — read this before trusting the citation gate on this
> document.** `./scripts/plan-cite-check.sh` resolves paths under this repo and
> the `seedhammer` fork only. It does **not** reach `mnemonic-key`, and it
> reports an unreachable path as `DANGLING  … (no such file under any root)` —
> **indistinguishable from a wrong citation.** The **three**
> `mnemonic-key/crates/mk-codec/src/key_card.rs` citations above (`:15-18`,
> `:25-32`, `:36-37`, plus the `:38-39` range named inline) were therefore
> **verified by hand** against `8dc5dcbf31947762a354d165ca2350ddbb15ba28`, and
> every range was corrected by one line in the process — the first draft's
> ranges were all off by one. A green gate on this file does not cover them,
> and the gate's own output for them is `dangling: 3`, which is **expected
> here and must not be "fixed"** by deleting the citations.

---

## Q6 — what happens to the goldens?

### The inventory, measured

| set | count | sensitivity to this cycle |
| --- | --- | --- |
| `backup/testdata/text-{0,1,2}-shards-1.bin` | **3** | **directly** — mechanism 4, what a band change touches |
| `backup/testdata/freetext-{0,1}-*.bin` | 2 | only if shared `backup/wrap.go` layout changes |
| `gui/testdata/sizeproof-{front,back}.bin` | 2 | mechanism 1; **designed to move** |
| `backup/testdata/passphrase-*.bin` | 4 | mechanism 3 — moves if Q3's line lands |
| `backup/testdata/seed-*`, `slip39-*`, `codex32-*` | 7 | mechanism 2, independent — should not move |
| `gui/op/testdata/*.png`, `engrave/testdata/*.bin`, `bspline`, `s2_md1_golden.expect.json` | 8 | not plate-layout at all |

**`backup/testdata`'s sixteen are FROZEN — and only by a comment.** Quoted from
`gui/freetext_sizeproof_golden_test.go:63-64`: *"Those sixteen goldens are
FROZEN: a moved byte is a finding, and -update has never been run on them."*
Nothing in code enforces it.

**A live footgun:** a bare `go test ./... -update` **rewrites the frozen
sixteen**. The same file warns: *"Scope it with -run."*

### Recommendation — make "no churn" the assertion, not the review burden

Q6 asks how a churned golden gets reviewed. **The better answer is to arrange
for there to be no churn to review.**

1. **Because the title is optional, the unmarked path must be byte-identical.**
   So after implementing, run the goldens **without** `-update`. If
   `text-{0,1,2}-shards-1.bin` move, **the band is not as optional as designed
   and that is a finding**, not a golden to refresh. This converts Q6 from a
   review problem into a **regression test that already exists**.
2. **Add NEW golden files for the marked states** rather than updating frozen
   ones. A new state deserves a new artifact; the frozen sixteen keep meaning
   what they meant.
3. **Expect `passphrase-*.bin` to move** if Q3's line lands — that is genuine,
   intended churn on 4 files, and it should be re-recorded **in the same commit
   as the change that caused it**, which is the contract
   `sizeproof-{front,back}.bin` already documents.
4. **Never run a bare `-update`.** Scope every regeneration with `-run`.

---

## What this proposal does NOT settle

- **§3 Q2 — does a title/footer actually fit alongside text+QR?** Still the
  gating measurement, now with two extra jobs: measure the TEXT+QR variant
  threshold with a band present, and settle the passphrase band's third-line
  problem (Q3).
- **Whether to restore `fadeClip`'s clip mask in S6b** — with the operator.
- **Whether "key-id" means the master fingerprint** — with the operator.
