# PLAN — wallet-file export for the reasonably complex wallet

**Status:** DRAFT. Two of five planning reports still outstanding
(`PLAN_export_nunchuk.md`, `PLAN_export_bitcoin_core.md`); their sections below
are marked **PENDING** and this plan may not be implemented past Phase 1 until
they land and are folded.

**Operator ask (2026-08-22):** output Nunchuk, Sparrow and Bitcoin Core
**watch-only and hot** wallets via the m* utilities, and the same on SeedHammer
II.

---

## 0. The correction this plan is built on

I briefed five agents with a "settled fact" that was **false**:

> *"no export surface exists anywhere in ms/md/mk/me today"*

True as written and wrong as asserted. I checked four CLIs and wrote *anywhere*,
forgetting the constellation's fifth repo. **`mnemonic export-wallet` has
shipped in `mnemonic-toolkit` since v0.97.0** — 11 formats, watch-only by
definition, ~4k lines of emitters. This plan therefore EXTENDS a shipped
surface; it does not build one.

Everything in §1 was measured by running the tools, not read from a report.

---

## 1. The gaps, measured

Against this wallet's own concrete descriptors
(`design/journeys/out/rcw/{tr,wsh}/descriptor.txt`):

| what | result | exit |
| --- | --- | --- |
| `export-wallet --descriptor <wsh> --format bitcoin-core` | **works** — 2694 bytes of `importdescriptors` JSON, valid checksum | 0 |
| `export-wallet --descriptor <tr> --format bitcoin-core` | **refused** — *"All spend paths must require a signature"* | 2 |
| `export-wallet --descriptor <wsh> --format sparrow` | **refused** — *"requires --template; descriptor passthrough is not supported"* | 1 |
| `--format nunchuk` | **does not exist** (11 formats, no nunchuk) | — |
| hot-wallet export, any format | **does not exist** — watch-only by definition | — |

Four gaps, in the order they should be closed:

### G1 — `export-wallet` has no `--allow`, so the tr form cannot export

The tier-4 keyless spend path trips `rust-miniscript`'s `sanity_check()` at
`cmd/export_wallet.rs:524` (`MsDescriptor::from_str`, the strict parse).

**The capability already exists and is not wired here.**
`descriptor_builder/gate.rs` has `AllowSet` with a `sigless_branch` field
mapping to `ExtParams::top_unsafe`, and `build-descriptor` exposes all five
rules as `--allow <RULE>` including `sigless-branch`. `md encode` solves the
same problem with `--experimental`.

So G1 is *parity*, not invention: the smallest change that unblocks the wallet
the operator actually asked about.

> I initially recorded `sigless-branch` as "deliberately excluded" from
> `--allow`. That was false and came from my own `grep -A6` truncating a
> five-item list. It is exposed. Same class of error as reading an exit code
> through a pipe, which I also did in this cycle.

### G2 — no Nunchuk format — **PENDING** `PLAN_export_nunchuk.md`

Open question the report must settle: whether Nunchuk is reachable through the
**already-implemented `bsms` emitter** rather than a new format. If it is, G2
collapses to documentation.

### G3 — no hot-wallet export anywhere

`export-wallet` is watch-only *by definition* — `validate_watch_only` rejects
phrase/entropy/xprv/wif at slot resolution.

The CLI-surface report's ruling, which this plan adopts: **hot export never
lives in `md`, and never as a flag on the watch-only surface.** If built at all
it is a distinct subcommand (`mnemonic export-signer`) with secret slots,
`--output` required, `0600` + `create_new`, an always-on advisory, and no
interactive confirmation.

**This is the largest and least-safe item. It is deliberately last.**

### G4 — SeedHammer II has no wallet-file output

The one-output claim is true at the platform interface (`gui/gui.go:3385-3415`)
but the useful finding is that the transport is **~90% built**: the Type 4 tag
emulator already implements `READ_BINARY` and transmits over RF, serving a
hardcoded 2-byte `emptyFile` with 8 KB advertised capacity
(`nfc/type4/type4.go:103-106, 241-242`). An NFC share is ~200-400 LOC with no
new dependency, ranked above a paged-display QR route.

---

## 2. What is NOT a gap

**Sparrow.** `PLAN_export_sparrow.md` finds Sparrow has no miniscript engine at
all (its `Miniscript` class is a 59-line regex shim; miniscript is open feature
request #1700, absent through v2.5.3). The tr form is rejected loudly. **The wsh
form is silently imported as a `sortedmulti` 3-of-6 P2WSH with wrong
addresses.**

Our emitter refuses descriptor passthrough, so the constellation **cannot**
produce that file. That is the right outcome — but note it is **incidental**:
the emitter refuses because it only accepts templates, not because anyone
reasoned about Sparrow's miniscript gap. A future change that adds descriptor
passthrough to the Sparrow emitter would silently create a funds-safety defect.

**Action: none in code; one regression test pinning the refusal, so the
incidental safety becomes deliberate.**

---

## 3. Phases

Each phase ends with a fable review before the next begins, per the operator's
instruction. Phase 1 may start now; Phases 2-4 are blocked on the pending
reports being folded.

### Phase 1 — G1, `--allow` parity on `export-wallet`

**This changes ADMISSION**, which is risk-set work (project CLAUDE.md item (c)),
so it takes the R0 gate: this plan reviewed to 0C/0I before code.

- Add `--allow <RULE>` to `export-wallet`, repeatable, reusing `CliAllow` and
  `allow_set()` from `cmd/build_descriptor.rs` rather than a parallel enum.
- Route the `--descriptor` parse at `export_wallet.rs:524` through the relaxed
  path when any allowance is requested, mirroring what `md encode
  --experimental` does with `ExtParams`.
- **The warning is not optional.** `build-descriptor` already has the
  never-silent surface (`emit_allow_notes`): an unmissable stderr warning for
  every allowed rule that actually FIRED, plus a note for each requested
  allowance that did not. Reuse it; do not reimplement a quieter one.
- Tests: tr form exports with `--allow sigless-branch` and refuses without it;
  the warning appears; a rule requested but not fired says so; the wsh form is
  unaffected with no flag.
- **Rust-primary:** `mnemonic-toolkit` is Rust and upstream of the Go port. No
  Go change is due unless the fork gains an export surface, which it has not.

### Phase 2 — G2, Nunchuk — **BLOCKED** on `PLAN_export_nunchuk.md`

If the answer is "use `bsms`", this phase is a test plus documentation.

### Phase 3 — Sparrow refusal pinned as deliberate

One test asserting `--format sparrow --descriptor` refuses, whose comment
states WHY: Sparrow would misimport this shape with wrong addresses.

### Phase 4 — G3, hot export — **gated on an explicit operator go-ahead**

Not because it is hard, but because it writes spendable key material to disk and
the operator asked for it in one clause of a long request. It deserves its own
confirmation before anyone builds it.

### Phase 5 — G4, SH2 NFC share — separate cycle

Firmware work with its own risk profile. Not in this plan's scope beyond
recording that the route exists and is cheaper than assumed.

---

## 4. Open questions

1. Does Nunchuk import BSMS? (PENDING)
2. Which Bitcoin Core version watches vs solves each wrapper? (PENDING) —
   `export-wallet` already has `--bitcoin-core-version`, so the answer is a
   parameter, not new plumbing.
3. Should `--allow` on `export-wallet` be named `--allow` (matching
   `build-descriptor`) or `--experimental` (matching `md encode`)? The two
   surfaces disagree today and this plan picks `--allow` for locality. Worth one
   line of operator input if they care.
4. A measured tr/wsh sanity-check asymmetry and `md`'s depth-0 xpub
   re-serialization are flagged in the CLI-surface report as *potentially
   normative*; if either is touched, it is Rust-first with vectors.
