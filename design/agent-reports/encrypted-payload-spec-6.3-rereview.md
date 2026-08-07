# Scoped re-review — §6.3 amendment (`0d19c27`), the post-GREEN change

Reviewer: opus, single-section scope. Dispatched 2026-08-07 to close a
gate-record gap: the spec passed R0 GREEN at `00da6a8`, was then amended at
`0d19c27` (a normative change to §6.3), and that change did not re-enter the gate.

Verdict: **1 Critical / 3 Important / 1 Minor / 0 Nit — GATE BLOCKED.**
Everything verified by execution against the real crates and the real fork
packages, on the spec's own vector records.

## The two questions flagged as genuinely open

- **Probe 2 — smuggling holds; GROUPING is the hole, and in the opposite
  polarity.** No false-accept exists: extra record → `chunk set incomplete`;
  duplicate index → `chunk index gap`; index-complete cross-card mix →
  `chunk set inconsistent`. `reassemble` demands exactly `count` chunks with
  indices `0..count-1` sharing one csid, then re-derives the csid from the
  decoded descriptor and compares. No slack for a rider. The defect is
  **false-REJECT of every multisig**.
- **Probe 5 — clean, no new hazard.** 4000 adversarial sets at §6.4's bounds:
  **0 panics, 0 unexpected successes, worst case 78 µs**; 24 × 512-byte records
  in 15 µs. The `(symbol_aligned_bit_count - 37) / 8` subtraction cannot
  underflow (the 37-bit header read is gated first by `BitStreamTruncated`);
  every allocation is sized by a 3- or 5-bit read, never a varint; recursion is
  capped by `maxDecodeDepth`. Safe pre-authentication even with no watchdog.

## CRITICAL — the grouping key was never defined, and the natural reading rejects every multisig

"Group the public records by card" is not operational. The only key that works is
the 20-bit `chunk_set_id`; the obvious alternative — one group per HRP — is
wrong, and §6.3's own evidence block showed only one card per HRP, which reads as
confirmation that HRP grouping is fine.

A 2-of-3 `wsh-sortedmulti` has **three separate `mk1` cards**:
```
mk1 CARD 0 alone (2 chunks)   → Ok
mk1 ALL SIX as one HRP group  → "received 6 chunks, header declares 2"
```
So the flagship shape — `ms1` encrypted, `mk1`+`md1` public — is rejected
outright. **Invisible to the suite**: D and E carry one card per HRP; F is
`pub_len = 0`. An HRP-grouping implementation passes A–F. That is the blindness
vector F was added to close, one level down.

**Folded:** `(HRP, chunk_set_id)` stated normatively, with the card counts, the
error strings, and **vector G** — a 2-of-3 mixed payload whose 12 public records
span six cards.

## IMPORTANT — the md-codec 0.42 floor was normative and FALSE

Three linked claims, all wrong: the records carry version **4**, not 9; 0.40
**reassembles all of them** including a 6-chunk multisig card; and no bump is
required. The `9` came from one call — `decode_md1_string` (single-string) on a
chunked record — and is `0b01001`, version 4 with the chunked flag, misread as a
5-bit version. The granularity diagnosis was right; the version skew attached to
it was the same error misread a second time, then made a MUST.

Compounding: the device's Go port is provenance-pinned to md-codec **0.36.0**, so
a host-only bump would widen a real host/device gap for no demonstrated reason.

**Folded:** claim deleted, and the misreading recorded in place so it is not
reintroduced.

## IMPORTANT — the amendment was never propagated

§6.4, §10.2.1 and §9 still mandated per-record `md.Decode` — the one API that
explicitly refuses chunked input (`md/md.go:1231`). Following §10.2.1's table
literally rejects **every** payload including D and E. §10.2 step 2 named no
grouping, reassembly or decode step at all.

**Folded** into all four.

## IMPORTANT — reassemble-only rejects a legitimate non-chunked md1

The old rule rejected chunked; the new rule rejected non-chunked. Neither covered
both. `md_codec::encode_md1_string` is a public API emitting the single-string
form, and `me seal` accepts operator-supplied records.

**Folded:** dispatch on the chunked flag, which `ParseChunkHeader` already
surfaces.

## MINOR — Go lacks Rust's 93-symbol codeword cap

Host and device disagree on admissible records at §6.4's 512-byte bound, on the
function §6.3 just made normative. Not introduced by this amendment. Rust is
already correct, so per the Rust-primary rule this is a convergence port.
**Filed as F-67**, owning phase Plan B.
