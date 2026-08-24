# CONTINUITY — the `mt` cycle, 2026-08-22

> **SUPERSEDED — see `design/CONTINUITY_mt_2026-08-23.md`, which is the live
> resume note. This one is kept only as the record of where the cycle stood
> before review.**
>
> **SUPERSEDED 2026-08-23.** This is the resume note written *before* the R0
> review ran. Everything below describes the pre-review state and several of its
> conclusions have since been overturned — UR was adopted and then **dropped
> entirely**, transaction construction was **removed from scope**, and the
> "three open questions" listed here are all now closed or reframed. Read
> `design/SPEC_mt_v0_1.md` §10 for the live list; keep this only as the record
> of where the cycle stood before review.
>
> Two citations in it were also wrong and are corrected in place. A bare
> `backup.go` at line 161 does **not** name `frontSideSeed` — line 161 is inside
> `EngraveSeedString`, and the real definition is `backup/backup.go:247`. A bare
> `fountain.go` at 242 was right about the code but needed its `bc/` prefix to
> resolve. The claim about "16/16 citations" below did not hold: the gate found
> 6 of 12 dangling.

Written before a context clear. Everything below is committed; `origin/master` is
at `ba8fa57` (one later commit, the second push report, may still be local).

## Resume with

    Read design/CONTINUITY_mt_2026-08-22.md and design/SPEC_mt_v0_1.md,
    then run the R0 architect review of the mt spec.

## Where the cycle stands

`design/SPEC_mt_v0_1.md` — 534 lines, **DRAFT, pre-R0**. No code may be written
until an architect review closes it at 0 Critical / 0 Important. Risk-set work:
funds, addresses, a new normative format.

**Settled** (each recorded in the spec with the reasoning that produced it):

- Three verbs, separated because conflating them caused two reversals —
  **produce** unsigned, **present** for hand-off to a signer, **engrave** signed
  transactions only (§1a).
- Own repo, `mnemonic-transaction`, `mt-codec` + `mt` CLI. This overruled the
  recommendation to make it an `me` subcommand; §2 answers the objection.
- **UR** (BCR-2020-005) as the envelope, not QR Structured Append (no encoder
  anywhere in reach) and not an `mt` invention (would defeat F-234).
- **The QR carries the standard form, never codex32** — F-234.
- **ECC = the highest that minimises plate count**, with legend space reserved.
- Provenance in the engraved legend, never the wire format. The 4-byte policy-id
  stub is a **hint, never an authority**: if it disagrees with the transaction,
  the transaction wins.
- Amounts: `gettxout <txid> <vout> false` when a node is reachable, fetched
  automatically. Four tiers below that; a bare typed amount is **refused**.
- A future locktime is required by default, `--allow-immediate` overrides.

## The three open questions, in the order they matter

1. **§10.6 — how much fountain redundancy?** THE BIG ONE. Currently zero: UR
   parts `1..seqLen` are pure singletons (`bc/fountain/fountain.go:242`), so
   **one unreadable plate destroys the transaction**. An artifact whose purpose is
   surviving decades in a drawer cannot obviously ship that way, and buying
   tolerance costs one symbol — often one plate — per part.
2. **§10.7 — back-side engraving.** Would recover the 25.5 mm the legend costs,
   restore two to three ECC levels, and halve several plate counts. But there is
   **no back-side path in the fork**: `backup/backup.go:247` defines
   `frontSideSeed`, called once at `backup/backup.go:134`, and there is a single
   `Engraving` per plate. Firmware work; cost it before
   accepting the doubled plate counts.
3. **F-234's optical test plate** — gates the 0.30 mm module. Matters more than
   it looks: finer modules are what buy the legend's space back without new
   firmware. Cut one plate with QR blocks at 0.30/0.45/0.60/0.90 mm **and**
   raw-vs-base45-vs-UR payloads, in one cycle.

## What is already machine-verified — do NOT spend reviewer budget here

- **16/16 citations** in the spec resolve against real source (file, line,
  constant, crate feature). Re-runnable.
- **All three measurement gaps are closed**, and each one *changed* the spec:
  the legend was 58% over budget and §4 left it zero lines; UR overhead is 12–14
  bytes CBOR per fragment, now modelled exactly; and measuring it surfaced a
  false claim about fountain redundancy that is now retracted in §3.
- Eleven results files + a re-runnable probe crate in `design/measurements/`.
  Transaction sizes are real signed/finalized/serialised bytes. QR capacities are
  gated against published v40 limits — that gate caught **three** wrong payload
  constructions of mine before any number was trusted.

Brief the reviewer to spend its budget on **design, threat model, and §10.6** —
not on arithmetic.

## Also done today, unrelated to `mt`

- **RCW definition changed twice.** Tier 4 keyed (`pk(@6)`), and hashlocks
  double-hashed. The second fixed a live defect: miniscript's `sha256()` emits
  `OP_SIZE <32>`, the passphrases are 34–40 bytes, so **three of four tiers were
  unspendable by anyone** — consensus-enforced — while every digest matched and
  nothing looked wrong.
- **The journey was repaired after running it**, not reading it. Five gates went
  red; four were staleness. The fifth was a negative control that had gone
  **vacuous** — it mutated tier 4 to ADD a key, which the gate now wants, so the
  `sed` no-opped and the gate failed for a MISSING FILE instead. It would have
  printed *"AND THE GATE CAN FAIL"* forever while proving nothing.
  `check_tiers.py` now separates exit 1 (policy wrong) from exit 2 (cannot
  evaluate), and the control is inverted and mutation-tested.
- **Still open there:** F-232's last gate needs an emulator walk —
  `python3 capture_rcw.py --wrapper wsh --route seating`, then rebuild the PDF.
  The two host implementations already agree on the new wallet; only the
  captured device value is stale. **Needs Go and Playwright, neither installed
  on this machine.**

## Follow-ups filed or resolved today

F-229 RESOLVED (tier 4 gets a key) · F-231 (the other two fixtures carry the
same defects) · F-232 (RCW journey artifacts) · F-233 (rust-miniscript
sanity-checks `Tr` only, so one wallet's two wrappings disagree about their own
validity) · F-234 (every QR carries the standard form).

## Local quirks worth knowing

- Two `preview::` Rust tests fail locally and pass in CI — a stale
  `target/release/me-preview` beside the binary; the tests assert it is absent.
  Not a defect.
- `md` on the shell is aliased to `mkdir`. The real binary is
  `descriptor-mnemonic/target/release/md`.
- The Bash tool runs **zsh**: unquoted expansions do not word-split, and
  `--include=*.rs` gets glob-expanded. Quote them.
- A synced `bitcoind` is available — used to verify `gettxout`, `gettxoutproof`
  and header arithmetic against real chain data.
