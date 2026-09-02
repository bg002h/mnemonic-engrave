# Staged plan — the Wallet Policy COMPOSER (arbitrary tr/wsh authoring on the SH2)

**STATUS: DRAFT 2026-09-01, written as the R0 loop on
`SPEC_wallet_policy_composer.md` closes.** This document stages the work; the
detailed, gated implementation plan exists only for the stage about to be
dispatched, because a plan's GREEN expires the moment the tree moves under it
(CLAUDE.md, 2026-08-27 directive). Stage 0's detailed plan is
`IMPLEMENTATION_PLAN_composer_S0_md_compose.md`; each later stage gets its own
detailed plan, written and R0-reviewed immediately before its implementer is
dispatched, with `scripts/plan-staleness-check.sh` run against the heads
recorded here.

Spec: `design/SPEC_wallet_policy_composer.md` (R0 rounds 0-3 folded; see its
header). Rulings: `design/BRAINSTORM_wallet_policy_composer.md` §2 (C1..C29) and
§3.12 (controller defaults 1-20).

| repo | baseline head at staging |
| --- | --- |
| descriptor-mnemonic | `3b0944fb` (md 0.14.0, md-codec 0.42.0) |
| mnemonic-engrave | see `git log` (this document's commit) |
| fork `bg002h/seedhammer` | `169073c` |
| mnemonic-secret | `5f37b43` |
| mnemonic-toolkit | `d8f06483` |

---

## Why staged, and in this order

Rust first, always (CLAUDE.md Rust-primary rule): the lowering is normative
codec behaviour and lands in md-codec with vectors before any Go. The host-side
payload classes are needed before the device can be driven in the emulator, and
the emulator journey is the gate every device stage is measured by. So:

```
S0 md-codec compose + md compose + vectors ──► S1 me sysw key:/hash:/now: + ms bip48-p2tr
      (normative lowering, Rust)                  (host inputs; lockstep vectors)
                     │
                     ▼
S2 fork codec: Go builder + pk_h arm + PolicyShape split ──► S3 fork GUI: door, shape, entry,
      (byte-identical to S0 vectors)                            seating, consent, engrave forms
                                                                         │
                                                                         ▼
                                                        S4 journey EXECUTED on the emulator
                                                           + payload-spec fold + deprecation note
```

Every stage is risk-set work (normative codec behaviour, funds/keys/addresses,
spans repos): R0 to 0C/0I on its detailed plan before code, one implementer,
whole-diff review after, persist-then-fold commits, UC OFF during implementation.

## S0 — md-codec `compose` + `md compose` + the compose vector family (descriptor-mnemonic)

**Delivers (spec §5, §10 items 1 and 3, §12 items 1, 4, 7):** the fixed
lowering as a library (`md_codec::compose`), the `md compose` subcommand, the
structural refusals and the §4c lock-range check as library errors, the tagged
compose vector family in `test_vectors::MANIFEST` (exported by `md vectors` into
`.conformance.json` for the Go gate), the §5b cross-check (parse, sanity with
`top_unsafe` for keyless wsh, round-trip, `lift` equality against the compiler
where a key exists on every path), and the five presets as path lists.

**Gates:** `scripts/plan-build-gate-md.sh` on the plan (builds and runs the
extracted Rust); `cargo nextest run --locked -p md-codec -p md-cli`; `cargo fmt
--check`; `cargo clippy -D warnings`; `scripts/plan-cite-check.sh`,
`plan-glyph-check.sh`, `plan-table-check.sh`, `plan-stepref-check.sh` on the
plan; a tag-coverage test asserting every §5 row / §4c lock row / §4f origin row
tag appears in ≥ 2 vectors.

**Exit:** md-codec and md-cli published to crates.io at the next minor; the
compose vectors vendored into the fork's `md/testdata/vectors/` (S2 consumes
them). Detailed plan: `IMPLEMENTATION_PLAN_composer_S0_md_compose.md`.

## S1 — host inputs: `me sysw pack` classes and `ms derive --template bip48-p2tr` (mnemonic-engrave, mnemonic-secret)

**Delivers (spec §6a, §10 items 2, 5, 6):** `key:`, `hash:`, `now:` record
classes with the §6a body rules and §8n refusal lines in `crates/me-cli`'s
`sysw::pack_with` / `admit_check`; `now:` auto-appended last only when the
operator supplied none, `--no-now`; the payload spec (`SPEC_systemwide_payloads.md`)
folded under its OWN R0: section 3.3.1 rows, the CREATED Wallet Policy row in
3.3.2 (ten cells), section 5.3 prefixes; `ms derive --template bip48-p2tr`
(`m/48'/0'/account'/3'`) with its renamed negative test; the host half of the
§12 item 8 lockstep vectors (a JSON fixture of records × expected class the fork
reads in S2).

**Exit:** `me` and `ms` published; the lockstep fixture vendored into the fork.

## S2 — fork codec: Go builder, `pk_h` emitter, `PolicyShape` split, minting (seedhammer `md/`, `mk/`)

**Delivers (spec §9 items 1, 2, 8; §12 items 1, 6, 7, 8):** a Go tree BUILDER
that constructs a `descriptor` from a path list and emits chunk-form md1 via
`split`, byte-identical to every S0 vector; the `pk_h` emitter arm in both
contexts with the address-changes mutation test; `md.PolicyShape` split of
`or_i`/`or_d`/`andor` into separate branches carrying lock operands and digests;
the `3'` origin arm; `mk.Encode` minting with appended stubs; the device-side
§4c lock-range check; the `sysw.Classify` half of the three record classes
(lockstep with S1's fixture). No GUI yet: package APIs and tests only.

**Exit:** `go test ./md/ ./mk/ ./sysw/` green with the S0 vectors and the S1
fixture vendored; TinyGo device build green; flash/RAM delta recorded against
1,503,652 B / 62,592 B.

## S3 — fork GUI: the composer inside Wallet Policy (seedhammer `gui/`)

**Delivers (spec §7, §8, §9 items 3-7, 9-11):** the door ChoiceScreen in every
state with its key-state lines; the shape flow with presets, the path-list
screen, the digit-pad widget and lock/hashlock entry with echoes and refusals;
the paged stub-teaching screen with the conditional per-slot origin line and
re-show; slot-directed seating from the payload with the paged pick list,
discard-on-numbering-change, the mapping review with the §4f invariant refusal,
C29 warning and §8k line; the composer's consent on the paged review screen
with the extended self-check and the §8l warning; the engrave form choice
including the partially seated form, Full/Watch-only, card minting and the
census counting card chunks; the deprecation comment on Multisig Build; the two
comment rewrites; the admission-row change. Every §8 body under the glyph,
raster and modal-fits gates and a fires-on-condition test.

**Exit:** all §12 items except 2 and 9's ceiling number green in `go test
./gui/` (sharded, `scripts/gui-shard-test.sh`); the per-frame capacities of the
three paged screens and the plate ceilings measured and written into spec §13.

## S4 — the journey, EXECUTED, and the records (mnemonic-engrave)

**Delivers (spec §12 items 2, 3, 9; §13 item 1):** the composer journey on the
emulator with a payload of `key:`, `hash:`, `now:` records and a seed, its
capture refusing to finish on a mismatch and its negative control run; the
no-payload walk ending in a keyless-template engrave; the engrave-surface
acceptance per journey; the journey PDF regenerable by its own README; the
FOLLOWUPS entries this cycle owes (Multisig Build deprecation note; §13 items).
A plan may not close while this gate has never run.

**Exit:** journey artifacts committed; spec §13 items 1 and 5 discharged;
continuity record closed; hardware flash per `~/bin/sh/sh2-flash` when the
operator calls for it (an on-device acceptance is defined by the journey, not
assumed).

## What is NOT in any stage (spec §14)

F-448, F-449, the unhardened-child route, NFC seating, on-device preimage
derivation, removing Multisig Build, on-screen QR, the D8 named backup formats,
Sealed-Payload memory discipline, an `andor` emitter arm, non-mainnet networks,
and the Ledger depth-0 registration recon (descriptor-mnemonic follow-up).
