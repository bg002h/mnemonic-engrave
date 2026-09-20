# F-630 — controller measurements, 2026-09-20

Recorded so they survive compaction. Measured against fork `95716e97` and
descriptor-mnemonic `b2c5d693`. These are the controller's own runs, not an
agent's report.

## The fork already emits the 0.44.0-correct header

A throwaway probe in package `gui` (`expandedToDescriptor` → `Encode()`) on
`keyed_compose_preset_plain_multisig`, compared against dm `b2c5d693`'s
`chains["0"]` and `chains["1"]`:

    chain 0: all 3 xpubs identical to the fork render
    chain 0: all 3 origin brackets identical (h/' spelling normalised)
    chain 1: all 3 xpubs identical to the fork render
    chain 1: all 3 origin brackets identical (h/' spelling normalised)

So "byte-identical, key for key" in the plan is all six keys across both
chains, not one spot check.

## Hardening is spelled differently on the two sides — FOLD THIS INTO THE PLAN

The fork renders `[73c5da0a/48h/0h/0h/2h]`; the primary's record carries
`[73c5da0a/48'/0'/0'/2']`. Same path, different spelling of hardening
(`bip32.Path.Encode()` emits `h`; `Derivation.Encode()` likewise, bip380.go).

Consequence for the gate: D1 parses the RECORD's bracket, which is always the
`'` spelling, so counting components is unaffected. But **D3's allowlist
compares a bracket path to the Go port's `OriginPath`**, and that comparison
must normalise `h` ↔ `'` or it will report a divergence that is only a
spelling. Same class as F-627.

Not folded into the plan yet — the R0 reviewer is reading `5cc0a0a2` and the
artifact must not move under it. Fold this with the review findings.

## Corpus drift, for reference

46 keyed conformance records: 2 identical, 41 descriptor-only (88 chain
entries), 3 semantic (F-529's three, none a `keyed_compose_*`). Zero address
lines and zero id lines move outside those 3.
