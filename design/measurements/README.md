# mt sizing measurements — 2026-08-22

Answers the brainstorm question: *how big is a transaction from our complex
wallets, in their `tr` and `wsh` forms, and how does that compare with ordinary
wallets?* Three subjects, three result files:

- `RESULTS_2026-08-22.txt` — the **pathological** wallet (11 keys, 3 masters),
  swept over 1/2/5/10 inputs.
- `RESULTS_baselines_2026-08-22.txt` — **ordinary wallets** for comparison:
  single-sig (`wpkh` and `tr` key-path), 2-of-3 and 3-of-5 in both `wsh(multi)`
  and single-leaf `tr(NUMS,multi_a)` form.
- `RESULTS_envelope_2026-08-22.txt` — **how many inputs a signed transaction can
  carry** before it overflows the 64-chunk container, per wallet, with the
  marginal cost per input. Both boundaries are **measured, not extrapolated**:
  the sweep builds and signs at `MAX` and at `MAX+1` and reports both. Also
  carries the **plate cost** of a sweep, since one plate per string (F-225) and
  ~21 min/plate make engraving time the binding constraint long before the codec
  ceiling is, and the **key-path** variants of both complex wallets.

  **CORRECTED 2026-08-23 — the ceiling is 2,560 B, not 2,904 B.** The old
  figure was the *theoretical* filled capacity: 64 chunks x (80 data symbols −
  37 header bits) = 363 payload bits each. The real chunker does not fill. It
  sizes by `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 * 5 = 320` bits
  (`md-codec/src/chunk.rs:224`, applied at `:253-254`) — a flat **40 payload
  bytes per chunk**, so 64 chunks is **2,560 B**.

  That error lived in ONE helper, `(n * 8).div_ceil(363)`, copied into **seven**
  probe binaries, so every chunk count in every results file was ~13% low and
  perfectly self-consistent. It is now the named constant `CHUNK_PAYLOAD_BITS`
  with the citation attached. All results files here were regenerated after the
  fix. Capacity conclusions moved: single-sig `tr` key-path 26 inputs -> **23**,
  RCW `wsh` tier 1 4 inputs -> **3**.

  This file previously called the counts "a floor", which was the right hedge —
  but `SPEC_mt_v0_1.md` dropped the hedge and refused against the numbers. Found
  by R0 round 1, finding S-1.

- `RESULTS_rcw_2026-08-22.txt` — the **reasonably complex wallet** (7 keys),
  swept over 1in/1out, 1in/2out and 5in/2out, on three spending tiers. It
  satisfies the fixture with the fixture's **own** preimages — no stand-ins —
  and asserts each `preimage-N.hex` is `sha256(preimage-N.txt)`. It also records
  that stock rust-miniscript accepts the stored (keyed-tier-4) form of both
  wrappers, which it did not before tier 4 was keyed.

Every number in all three files is a **length of bytes that rust-bitcoin
actually serialised**, after a real signature and a real finalize. Nothing is
estimated or hand-counted.

## What was run

`mt-size-probe/` is a two-binary scratch crate pinned to the same versions the
repo already locks (`miniscript 13.1.0`, `bitcoin 0.32.101`).

- `src/main.rs` — builds the unsigned 5-in/2-out PSBT and reports
  `Descriptor::max_weight_to_satisfy()` as an upper bound.
- `src/bin/signed.rs` — derives the real xprvs from the pathological fixture's
  three master seeds, **signs**, **finalizes**, extracts the transaction and
  measures it.
- `src/bin/rcw.rs` — the same for the reasonably complex wallet, reading all
  seven seeds and both wrappers straight from
  `design/journeys/inputs-rcw/`.
- `src/bin/envelope.rs` — sweeps every wallet above to its input ceiling, and
  contrasts it with the bare-PSBT ceiling, which depends on scriptPubKey length
  rather than on the policy. All its rows use **2 outputs**; a 1-output sweep
  buys roughly one more input of headroom.
- `src/bin/baselines.rs` — the same for ordinary single-sig and k-of-n wallets,
  built from the same committed test seeds so key material is not a variable.
  Taproot multisig is modelled as **one `multi_a` leaf** under the NUMS pin, not
  as a taptree of every k-of-n combination; the latter is smaller to spend and
  much larger to describe, and was not measured.

Subject: `design/journeys/inputs-pathological/wallet-policy-tr.txt` and
`wallet-policy.txt` — the four-tier degrading vault, 11 keys, 3 masters.

## Self-checks that had to pass

- Every derived xpub is asserted **byte-equal** to the committed
  `keys/key-NN.xpub`, and every derived fingerprint/origin is asserted present
  in that file. A wrong derivation aborts the run.
- The two binaries agree: `max_weight_to_satisfy` returns 447 WU (tr) and
  756 WU (wsh) per input; the real satisfied witnesses measure 445 B and 755 B.

## Deliberate substitution

Applies to `signed.rs` only. The **pathological** fixture commits a `sha256`
literal whose preimage is not in the repo, so that probe swaps each literal for
`sha256(<a 32-byte value we hold>)` and **asserts the swap is size-neutral**
(both are 32-byte hashes, both preimages 32 bytes).

`rcw.rs` needs no substitution: since the double-hash fix of 2026-08-22 the RCW
carries real 32-byte preimages, so the probe satisfies the actual stored wallet.
Re-running it before and after the swap was removed produced **identical
sizes**, which is the evidence that the pathological probe's substitution is
sound.

## Spending path forced, not assumed — and shown, not claimed

Each line prints the **witness stack element sizes** for input 0, so which
branch the finalizer actually took is visible in the output rather than argued
for in prose. The RCW probe additionally withholds specific keys per scenario
(`Scenario::withhold`) where a timelock alone cannot isolate a tier.

- **tier 1** (3-of-3 + hashlock): locktime = height 1,900,000, sequence
  `0xFFFFFFFE`. This makes tier 2 (a *time* `after`) and tiers 3/4 (relative
  locktimes, disabled by bit 31) unsatisfiable, so the finalizer has one choice.
- **tier 4** (1-of-3, cheapest): locktime 0, sequence `0x0040F0DA`. `after()`
  fails at locktime 0; `older(65535)` is height-typed and mismatches the
  time-typed sequence; only `older(4255898)` survives.

## Reproduce

**Verified end to end on 2026-08-22, before the spec's R0 review**: every binary
below was rebuilt from this committed tree and its output diffed against the
committed results file. All twelve reproduce, and nine are **byte-identical**.
The two exceptions are both capture artifacts, not measurement drift — recorded
under *Known capture artifacts* below.

    cd design/measurements/mt-size-probe
    for b in mt-size-probe signed rcw baselines envelope legend \
             select urover qrmodes qrmax qrplate psbtqr; do
        cargo run -q --bin $b
    done

| binary | results file | feeds |
| --- | --- | --- |
| `mt-size-probe` (`main.rs`) | `RESULTS_2026-08-22.txt` (Probe 1) | bound cross-check |
| `signed.rs` | `RESULTS_2026-08-22.txt` (Probe 2) | pathological wallet sizes |
| `rcw.rs` | `RESULTS_rcw_2026-08-22.txt` | RCW sizes, spec §4 |
| `baselines.rs` | `RESULTS_baselines_2026-08-22.txt` | ordinary-wallet comparison |
| `envelope.rs` | `RESULTS_envelope_2026-08-22.txt` | input ceilings |
| `legend.rs` | `RESULTS_legend_budget_2026-08-22.txt` | **spec §5**, the 136-char legend |
| `select.rs` | `RESULTS_ecc_selection_2026-08-22.txt` | **spec §4**, the plate/ECC table |
| `urover.rs` | `RESULTS_ur_overhead_2026-08-22.txt` | **spec §3**, UR per-fragment cost |
| `qrmodes.rs` | `RESULTS_qr_modes_2026-08-22.txt` | QR mode capacities |
| `qrmax.rs` | `RESULTS_qr_physical_max_2026-08-22.txt` | physical module limits |
| `qrplate.rs` | `RESULTS_qr_vs_text_2026-08-22.txt` | QR vs engraved text |
| `psbtqr.rs` | `RESULTS_psbt_qr_multisig_2026-08-22.txt` | PSBT-over-QR |
| `psbtfinal.rs` | `RESULTS_psbt_envelope_2026-08-23.txt` | **R0 T-3**, the compliant-envelope cost |

### Why this section is longer than "run three binaries"

It used to list three of the twelve, and the first one it listed **did not run**.
Two separate reasons, both of which made the documented path a dead letter:

1. The crate is inside the repo but was in neither `workspace.members` nor
   `workspace.exclude`, so `cargo build` here aborted with *"current package
   believes it's in a workspace when it's not"*. It now carries its own empty
   `[workspace]` table, the same pattern as `crates/me-cli/fuzz`.
2. `main.rs` read `desc-tr.txt` / `desc-wsh.txt` as **cwd-relative** paths, and
   those two files had never been committed — they existed only in an
   out-of-repo scratch copy of this crate. They are now committed beside the
   sources, and `main.rs` resolves them against `CARGO_MANIFEST_DIR` so the cwd
   no longer matters.

A generator nobody re-runs rots while its artifact keeps vouching for it. These
results are cited as measured fact throughout `SPEC_mt_v0_1.md`, so the path that
produces them has to be a command, not a memory.

**`desc-tr.txt` and `desc-wsh.txt`** are the key-expanded form of
`design/journeys/inputs-pathological/wallet-policy-tr.txt` and
`wallet-policy.txt` — same wallet, placeholders resolved to the committed xpubs.
They hold **public material only** (checked: zero `xprv`/`tprv` occurrences,
against a positive control confirming `xpub` is present). They are committed
rather than regenerated, which is a second copy of the same wallet and therefore
a drift risk; nothing currently asserts the two agree.

`Cargo.lock` is now committed too, so *"pinned to the same versions the repo
already locks"* above is enforced rather than asserted.

Paths to the fixture are absolute in `signed.rs`; it reads
`/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-pathological/`.

### Known capture artifacts

Neither affects a measured number; both are noted so a future re-run is not
misread as drift.

- `RESULTS_2026-08-22.txt` is `main.rs` output followed by `signed.rs` output,
  with the two `### Probe N —` headings and their blank lines added by hand at
  capture time. Every measured line is byte-identical to a fresh run.
- `RESULTS_psbt_qr_multisig_2026-08-22.txt` has eight lines of `cargo` build
  warning (`constant STROKE_MM is never used`) captured at the top, because it
  was taken with stderr merged into stdout. The measurement body below it is
  byte-identical.

## Dimensions swept

`signed.rs` runs 1in/1out, 1in/2out, 2in/2out, 5in/2out and 10in/2out, against
both spending paths, for both descriptor forms. Output amounts scale with input
count (100,000 sat per input, 1,000 sat fee) so no case trips `SendingTooMuch`.

## Not covered

- Only one output *type* for the external payment (a P2TR spk). A P2PKH or
  P2WSH destination shifts the non-witness size by a few bytes.
- No fee-rate realism: amounts are synthetic and chosen only to keep the
  transaction valid.


## `psbtfinal.rs` — added 2026-08-23, for R0 finding T-3

The R0 round-0 gate established that §3's `ur:bytes` envelope is forbidden for
production by BCR-2020-005 itself, and that the BCR-2020-006 registry has **no**
type for a raw signed transaction — `psbt` is the only transaction-shaped entry
in all 58 rows. The compliant payload is therefore a **fully finalized PSBT**,
which extracts to the identical raw signed transaction. This probe measures what
that wrapper costs, on the same fixture and the same forced spending paths as
`rcw.rs`.

Three forms are measured, and **each is asserted to extract to a byte-identical
transaction** rather than assumed to:

| form | what it is | extracts via the standard API? |
| --- | --- | --- |
| `full` | what BIP-174's finalizer leaves, untouched | yes |
| `MIN` | UTXO records kept, **output maps cleared** | **yes** |
| `lean` | UTXO records also stripped | **NO** |

**The `lean` row is a correction to this probe's first version, which assumed
stripping the UTXO records was free.** `rust-bitcoin`'s `extract_tx()` runs a
fee check, which needs each input's value, so a UTXO-stripped PSBT fails with
`MissingInputValue`. The bytes are all present and
`extract_tx_unchecked_fee_rate()` returns the right transaction — but the safe
API a recoverer would reach for refuses it, which is exactly the wrong property
for an artifact meant to be read by whatever tooling exists in 2040. The
assertion caught this; the first run panicked rather than reporting a cheap
number that would not have survived contact with a real extractor.

**MIN is the form to specify.** Clearing the *output* maps is what matters: the
change output's descriptor metadata (derivations, taproot leaf scripts) is what
an updater wrote for a signer, BIP-174's finalizer strips only *input* fields, and
no extractor or broadcaster reads it. On 2-output artifacts it is the entire
blowup — `tr` tier 1 1in/2out is **+1202 B** as finalized and **+61 B** with the
output maps cleared.

Measured overhead of MIN over raw: **+58 to +61 bytes at one input**, **+261 to
+271 bytes at five** (~54 B/input), i.e. 7.7%–17.5%.

`select.rs` gained a matching section that runs the same search on both sizes, so
the cost is expressed in **plates** rather than bytes. That section is purely
additive — the original rows are byte-identical, verified by diff.
