# Prep — documenting operator journeys

Written 2026-08-11 as a starting map for the next context. It records only what
would otherwise have to be re-derived; everything else is in
`CONTINUITY_2026-08-11b.md`, which this does **not** replace.

## The spine: nine programs, in the order the operator pages through them

From `gui/gui.go`'s `program` enum (`:166`) and the titles in
`StartScreen.draw` (`:1871`) — read from the enum, not from the switch, because
the switch could omit one silently:

| # | enum | title on screen |
| --- | --- | --- |
| 1 | `backupWallet` | **Backup Wallet** |
| 2 | `engravePassphrase` | **BIP-39 Password** |
| 3 | `engraveText` | **Engrave Text** |
| 4 | `engraveXpub` | **Account Xpub** |
| 5 | `engraveBundle` | **Engrave Bundle** |
| 6 | `engraveSingleSig` | **Engrave Single-Sig** |
| 7 | `engraveMultisig` | **Engrave Multisig** |
| 8 | `bip85Derive` | **BIP-85 Child Seed** |
| 9 | `unlockPayload` | **Sealed Payload** |

Two ordering facts the enum's own comments record, and which a journeys doc
will trip over if it re-orders anything: `engravePassphrase` and `engraveText`
were **inserted** rather than appended so that `bip85Derive` stays the last
*navigable* program (wrap and pager sites are keyed to it), and `unlockPayload`
was **appended** deliberately — it is the one program that does not always
appear. It is conditional on a payload being present in flash.

Below these sit ~40 sub-flows (`gui/*_flow.go`, `*_inspect.go`, `*_pick.go`…).
Enumerate them with:

```sh
cd /scratch/code/shibboleth/seedhammer
grep -rn 'func .*Flow(ctx \*Context' --include='*.go' gui/ | grep -v _test
```

## Constraints any operator-facing document inherits

These are already normative and must not be contradicted or quietly softened:

- **The sealed-payload wipe is INCOMPLETE**, by explicit operator decision.
  `README.md` "Security limitation" and `SPEC_encrypted_payload_delivery.md`
  §2.2 item 16. **What actually protects the operator is physical custody, not
  the wipe** — the device is deliberately debuggable (SWD readable, BOOTSEL not
  disabled). Any journey that ends "and then you are safe" is wrong.
- **Program scope (§2.2 item 12, operator ruling).** Only data entering via
  **Sealed Payload** gets the security wipes. Other programs — including legacy
  ones that read encrypted payloads — do not. A journeys doc must not imply
  otherwise; this is the single most misread rule in the project.
- **`ms1` never travels over NFC.** It is typed on the air-gapped keypad, or
  delivered through the sealed-payload path.
- **§10.2.4's idle wipe**: warning at 3:00, wipe at 3:30, and it is keyed on
  *effective* input as of F-103. Row 4 (the passphrase wipe) shares that
  mechanism.
- **F-83, accepted:** a plate under the needle cannot be wiped mid-cut.

## Existing material to build on rather than duplicate

- `SPEC_encrypted_payload_delivery.md` §10.2.x — the Sealed Payload journey is
  already specified screen by screen. Journeys documentation should *reference*
  it, not restate it, or the two will drift and the spec is normative.
- `README.md` — has the operator-facing security section already.
- `design/RUNBOOK_custom_boot_key.md` — the shape to imitate for a procedure a
  human follows at the machine.
- `cmd/emu` + `sh-sim` — run any firmware ref in a browser. **This is how to
  walk a journey without a plate**, and it carries a real sealed test payload
  and its passphrase deliberately, so the Sealed Payload journey is walkable
  end to end. Note **F-121**: the emulator does *not* home, so anything about
  head motion or resumed cuts observed there is not what the machine does.

## Open threads that touch this work

- **The flashed firmware's boot has not been judged.** Hardware carries
  `v0.0.0-g97e38c1` (flashed 2026-08-11, signature verified, flash verified),
  but it must be powered from the machine supply before anything is concluded —
  a laptop port gives a dark screen indistinguishable from a rejected signature.
  Several journey-visible changes are in that build: the `%` glyph in the KDF
  progress screen, `|` replacing the invisible `·` on four screens, and a
  shortened §10.2.3 warning.
- **`me` is at `v0.5.1`**; `v0.5.0`'s archives self-report `0.4.0` and are left
  as published. If a journey tells an operator to check `me --version`, say
  which answers are expected and what `0.4.0` means.
