# IDEA — Refugium: epoch-based deep cold storage wallet

> Filed 2026-09-24 as F-682. The operator's idea, recorded **verbatim** below;
> only the bullet glyphs were converted to Markdown list markers. **Not specced,
> not ruled, not brainstormed.** Treat everything below as the operator's
> proposal, not as a decision.

---

Refugium: epoch-based deep cold storage wallet (idea summary)

## Concept

A Bitcoin wallet family for multi-year cold storage with inheritance paths, built on Taproot and miniscript. Miniscript can't extend absolute timelocks dynamically, so funds instead live in a series of "epochs." Each epoch is a standard descriptor with the same policy template but later after() heights. To extend the deadlines, you move funds to a newer epoch.

## Structure

- One policy template, instantiated per epoch e with heights H0 + e·Δ, H1 + e·Δ, … (e.g., Δ = 26,280 blocks, about 6 months).
- Example template: a 3-of-3 primary leaf; a 2-of-2 after older(32768); heir key @5 after after(H0 + e·Δ); heir key @6 after after(H1 + e·Δ).
- Internal key: an unspendable NUMS xpub (different per address), or optionally musig(@0,@1,@2) for cheap, private refreshes if signers support MuSig2.
- Keys: m/87'/0'/i' where i is the placeholder index, so each seed's role is self-evident and one seed can fill several roles safely.
- Branches: epoch e uses the multipath <2e;2e+1>/* (receive and change), which stays compatible with BIP 388 wallet policies. Every key is unique across epochs.
- Precompute descriptors for epochs 0 through about 100 and export them as a file, so heirs can recover with standard tools (e.g., importdescriptors).

## Software's job

- Choose the current epoch: the nearest one whose deadline is at least the desired lead time away.
- Hand out receive addresses from it, with one coordinator to avoid address reuse.
- Track "relicts" (prior-epoch wallets) and warn well before any deadline.
- Build sweep PSBTs that migrate funds from relicts to the current epoch, signed by the primary signers.
- Register each new epoch's policy on hardware signers as it comes into use.

## Key trade-off

An older()-only wallet is simpler: one descriptor, automatic per-UTXO reset, standard tools. But it caps delays at about 15 months and requires moving every UTXO within that window. Refugium is only worth it for deep storage that should stay untouched for years.

## Risks to handle

- Funds left in a relict past its deadline become claimable by heir keys.
- Heirs wait until the last-used epoch's deadline.
- Hardware signers need per-epoch registration.
- If MuSig2 is used, nonces must never be reused.
- Rehearse on signet with coin type 1'.

## Vocabulary

- Refugium: the software; a refuge where life waits out an ice age.
- Epoch: one timelock period.
- Relict: a prior-epoch wallet.
- Migration: sweeping funds from a relict into the current epoch.

## Open questions

Final template, Δ, and lead time; whether to use MuSig2; the epoch descriptor file format and backup scheme; which hardware signers accept multipath pairs beyond <0;1>; and the refresh and alert UX.
