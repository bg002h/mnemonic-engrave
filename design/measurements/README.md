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

  **Caveat on the 2,904 B ceiling.** That is the *theoretical* chunked-codex32
  capacity (64 chunks x (80 data symbols - 37 header bits)). `md`'s chunker does
  not fill to 80 — it balances, so a real chunk measures **85 characters**, not
  the 96 a filled one would. A new `mt1` codec could choose to fill; today's
  encoder does not, so treat chunk counts here as a floor.
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

    cd design/measurements/mt-size-probe
    cargo run -q --bin mt-size-probe
    cargo run -q --bin signed
    cargo run -q --bin rcw

Paths to the fixture are absolute in `signed.rs`; it reads
`/scratch/code/shibboleth/mnemonic-engrave/design/journeys/inputs-pathological/`.

## Dimensions swept

`signed.rs` runs 1in/1out, 1in/2out, 2in/2out, 5in/2out and 10in/2out, against
both spending paths, for both descriptor forms. Output amounts scale with input
count (100,000 sat per input, 1,000 sat fee) so no case trips `SendingTooMuch`.

## Not covered

- Only one output *type* for the external payment (a P2TR spk). A P2PKH or
  P2WSH destination shifts the non-witness size by a few bytes.
- No fee-rate realism: amounts are synthetic and chosen only to keep the
  transaction valid.
