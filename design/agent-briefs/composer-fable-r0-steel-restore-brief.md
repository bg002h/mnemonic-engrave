# Lens 3 — RESTORE FROM STEEL and ARTIFACT FIDELITY (Go encode ↔ Rust decode)

Read `composer-fable-r0-common.md` first. Your worktree slug: `steel`.
Your regtest ports: `-rpcport=18743 -port=18744`.
Your report: `design/agent-reports/composer-fable-r0-steel-restore.md`.

## THE ONE QUESTION

Starting from ONLY what the SH2 cuts — form A keyed md1 strings; form B
keyless md1 (with fingerprints) plus one mk1 card per seated slot; in Full
mode the ms1 seed plate; for hashlocks the preimage plate — and NOTHING from
the device's memory or screens, can a person with the host tools (`md
0.16.2`, `mnemonic 0.103.1`) reconstruct **exactly** the wallet the composer
showed: same descriptor, same policy/template/wallet ids, same addresses, on
every shape the grammar admits? And does the device's Go port agree
byte-for-byte with Rust md-codec 0.44.2 on those artifacts, in both
directions (device-minted → host-decoded; host-minted → device-seated)?

A round trip is NOT a restore test: decode can stay green while keys are
unrecoverable. Assert the READ-BACK — the address — not the success code.

## Method — execution

1. Produce the artifacts the way the device does: through the engraving
   path (`gui/composer_engrave.go`, `gui/composer_cards.go`,
   `composerSecretCards`, `md.ComposerStubs`, `mk/encode.go`) in the harness,
   and/or `md.Compose` + the chunk emitters, for each shape. Capture the
   exact strings that would be cut, and the ids the screens show
   (`WalletPolicyId`, template id, stubs).
2. Restore on the host from those strings alone: `md` (decode / inspect /
   derive), `mnemonic` (`restore`, `verify`, `inspect`, `bundle` — the
   toolkit's `restore` just gained `xpub_from_65_bytes(bytes, network,
   origin)` and `VerifyCheck.md1_origin_match`; use them as a user would).
   Compare descriptor text, ids and the first addresses on both chains
   against the device's own values and against Bitcoin Core.
3. Reverse direction: mint cards on the host (`mnemonic bundle` → mk1;
   `md` → md1) exactly as `demo/sh2/build-payload.sh` does, and seat them on
   the device through `sysw/composer_records.go` and the seat flow — does
   the device accept what the 0.44.2 host mints (zero parent fingerprint;
   depth/child from origin), and does its Go port still run the mint-time
   validators on DECODE (the class Rust just fixed)? A card the host mints
   that the device refuses, or the reverse, is a finding.
4. Cross-language: diff Rust `md-codec` between `66bdf2f4` (the Go vectors'
   provenance) and `922778ad` in `compose`, `encode`, `identity`,
   `to_miniscript`, `validate`, `chunk`. Every changed behaviour is a
   candidate divergence — measure it with real strings through both
   implementations rather than reasoning from the diff.

## Hunt list

- Form B, PARTIALLY seated (§8p): cards carry the TEMPLATE stub only and
  the screen says the policy id does not exist yet — after the missing
  slots are seated later on the host, is the resulting wallet the one the
  operator composed? §4f lowest-free account defaulting for unseated slots:
  do host and device pick the SAME defaults, and do they still agree when
  the operator's seated slots already occupy accounts?
- Stub appending pushing a card into a THIRD chunk (`mk/encode.go:26-29`):
  census counts it; does restore read three chunks correctly, and in any
  chunk order?
- Key ORDER on restore: slot numbering by first appearance in emitted text
  vs the card order the operator stacks; `sortedmulti` re-sorting.
- `tr` internal key: NUMS vs a real key — is the NUMS point on the
  artifact or recomputed, and identical on both sides?
- Hashlock: is the PREIMAGE on steel (H6 plate) or only the digest? Restore
  the hash path from the plates alone and spend it on regtest.
- Network: chain setting on the device vs xpub version bytes on the card vs
  `mnemonic`'s network detection.
- Seed plate: Full mode cuts the seed ONCE for a seed that filled several
  slots — from that one plate plus the cards, are ALL its slots recoverable
  (different accounts, same seed)?
- The xpub header class: the device's Go port still emits/expects depth-0
  headers with a zero fingerprint; the host now emits depth/child from the
  origin. Feed each side the other's output. Also: the all-zero fingerprint
  as ABSENT sentinel — does the Go port treat `[0,0,0,0]` as an identity to
  match?
- The screen's ids vs Rust's ids computed from the artifact
  (`WalletPolicyId`, `WalletPolicyIDStub`, template id, encoding id): any
  byte off is Critical — the operator uses the id to check the plate.
- Legibility is out of lens (the plate renderer had its own cycle); the
  STRING content is in lens.

## Deliverable specifics

A **restore matrix**: shape → form (A/B/partial/full-with-seed/hashlock) →
strings cut (count of md1/mk1/ms1 chunks) → host restore command → descriptor
equal (Y/N) → ids equal (Y/N, which) → addresses equal on both chains (Y/N)
→ Core version. Plus a **direction table** for the cross-language checks:
device-minted→host and host-minted→device, per changed behaviour in the
66bdf2f4..922778ad diff, RUN result each.
