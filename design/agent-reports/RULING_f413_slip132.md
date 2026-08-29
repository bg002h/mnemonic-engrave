# RULING — F-413: SLIP-132 keys — refuse with the executable remedy, or normalise host-side?

**Date:** 2026-08-29. **Capacity:** architect consult substituting for the operator per the
overnight mandate (`CONTINUITY_2026-08-28-overnight.md`) and plan task P1.0
(`IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`), in the F-410/F-411/F-412 ruling line.
**Repo:** mnemonic-engrave, master @ `6f259a8`.
**Verified before ruling:** F-413's FOLLOWUPS entry read in full; `SPEC_descriptor_input.md`
§4.3, §4.5, §4.6, §5.5, §6 (the SLIP-132 row and the bare-`Zpub`/`Ypub` row), §7
(requirement 4's canonical-level invariant, the `neither` rows, the tag floor) read at the
cited lines; the fork's `bip380/bip380.go` `ParseExtendedKey` read at source — `ypubVer` is
declared in the constants and named in the version-normalisation switch, but the
classification switch has no `ypub` case, so it hits `default` and errors, exactly as the
spec measured. The device's own doc comment says it "returns normalized xpubs" — the device
already normalises the SLIP-132 versions it accepts. No premise in the brief was found
false, except one framing, which turns out to be decisive (below).

---

## THE RULING

**REFUSE STANDS. The spec as written — `me` admits exactly the device's five extended-key
versions `{xpub, tpub, zpub, Ypub, Zpub}`, and refuses `ypub`/`upub`/`vpub`/`Upub`/`Vpub`
with §6's per-version executable remedy — is confirmed. No host-side normalisation this
cycle. Nothing in the spec, the vector file, or the plan changes; P1.0 discharges by this
confirmation and P1.1 builds spec-as-written.**

One recommendation rides along (not a condition of the ruling): the controller should file
a new follow-up (next free number, F-426 at time of writing) for the **device-side** fix —
the fork's missing `ypub` classification case — owned by S2's firmware build. That is where
the asymmetry actually lives, and fixing it there erases it for both doors at once.

---

## Rationale

### 1. "Mechanical byte swap" is false for the commonest arrival shape — the spec's own §6 measurement says so

The whole weight on the normalise side is *"`ypub` is the commonest non-`xpub` key a real
operator holds."* But ask how that `ypub` arrives. Descriptor-emitting coordinators emit
`xpub`/`tpub` inside descriptors — SLIP-132 versions inside a full script expression are
the rare shape. The common `ypub` is **bare**: a wallet app's "your public key" string,
pasted alone. And for a bare key, normalisation-as-byte-swap packs the **wrong wallet**:
§6's row already measured it — *"handing back a bare converted key would PROMOTE to a
different wallet (`pkh(…)`, measured)"*. A bare `ypub` byte-swapped to `xpub` and fed to
the cascade promotes via §4.5's version fallback to `pkh(xpub…)` — mainnet BIP-44
addresses — when the `ypub` version byte declares BIP-49 `sh(wpkh(…))`. The version byte
IS the script declaration; stripping it before the cascade is not a spelling change, it is
deleting the operator's script intent on exactly the path where nothing else carries it.

So the safe normalisation for the common case is not a byte swap at all — it is a **new
§4.5 promotion row** (version byte → script wrapper → invented `49'/0'/0'` origin),
reopening a table the spec marks "NORMATIVE, and this is a ruling rather than a
transcription". The brief's cost framing ("bounded — the conversion is mechanical")
undercounts the only case that matters.

### 2. Three of the five versions cannot be served bare at all, under a standing GREEN ruling

`upub` and `vpub` are testnet. §4.5 rules: *"`me` … refuses `tpub` promotion entirely. A
testnet key whose only claim to being a wallet is a version byte that maps to a mainnet
derivation path is an inference the host declines to make."* Normalising a bare
`upub`/`vpub` to `tpub` lands it in that refusal — normalisation buys those operators
nothing but a worse message. Bare `Upub`/`Vpub` are multisig cosigner keys, not wallets
(§6's bare-`Zpub`/`Ypub` row's logic), refused regardless of spelling. **Bare-key
normalisation could ever serve exactly one version — `ypub` — and only via the new
promotion row of point 1.**

### 3. What a byte swap CAN safely serve is the rare shape, and the reopen cost is real

Inside a full explicit script — `sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))` — the swap
is genuinely safe: script explicit, origin explicit, §7's invariant
`host_admits(input) ⇒ device_admits(canonical(input))` holds because the canonical carries
`xpub`. The whitespace precedent (§4.6) licenses host-wider-than-device where the canonical
is what gets packed. Granted in full. But that shape is the one essentially no real tool
emits, and the price is reopening spec text GREEN through 20+5 rounds: §4.3's NORMATIVE
five-set sentence, §6's SLIP-132 row rewritten from refusal to announcement, §5.5's table,
and §7 — where the full-origin-`ypub` row is one of exactly **three** `neither` rows the
vacuity check requires (flip it and the `neither` floor needs a replacement row), plus the
tag-floor arithmetic this cycle has already shown ripples on every row change (R0 r7's
NEW-M1). A reopen that buys the rare half of the benefit while the common half needs a
promotion-table change anyway is the wrong trade mid-cycle, the night before S1 closes.

### 4. The refusal's remedy is the same conversion with the operator's hands on it

The refusal prints the exact converted spelling — full script wrapper included, operator's
own fingerprint/path substituted, per-version network-correct target (4 of 5 are testnet,
where an `xpub` remedy would name a mainnet wallet the operator does not hold — measured
in the row). The manual step is one copy-paste, and it is also the moment the operator
learns their "public key" was carrying a script-type declaration — the informed-consent
moment a host-side transform, however loud, performs on their behalf. This cycle's
revealed operator preference points the same way twice: F-422's interim ruling took status
quo over even a *consented* transform of the operator's artifact ("file my desire for
status quo"), and F-417 declined a widening for thin measured demand. The §4.5 promotion
precedent does show a LOUD transform is licensable — but promotion was specified, walked,
and reviewed through the full R0 loop; it is a precedent for *how* to spec one in a future
cycle, not a license to fold one in past a closed gate.

### 5. The zpub/ypub asymmetry is real — and it is the device's, so fix it in the device

"The device already normalises SOME SLIP-132 versions, just not `ypub`…" is true, and at
source it is visibly an omission, not a design: `ypubVer` sits in the constants and in the
version-normalisation switch, and only the classification switch lacks its one-line
`case ypubVer: script = P2SH_P2WPKH`. The durable fix adds that case **in the fork** (and
plausibly upstream as a small PR), whereupon the device admits `ypub` at its own scan
door, and `me`'s rule — admit exactly what the device admits — follows it in that cycle's
spec fold with no host-wider-than-device window ever existing. That work is firmware, the
SH2 is away from the bench (F-418), and S2 owns the next firmware build; hence the
recommended follow-up below. Host-side normalisation now would build a permanent host/device
asymmetry to paper over a temporary device bug.

---

## Per-version disposition under this ruling (unchanged from the spec; stated for the record)

| input | disposition |
| --- | --- |
| `ypub` in a full script, or with origin | REFUSE; remedy prints the `xpub` spelling, operator's origin substituted (mainnet BIP-49) |
| bare `ypub` | REFUSE; remedy prints `sh(wpkh(<converted key>/<0;1>/*))` — never a bare converted key, which would promote to `pkh` |
| `upub` / `vpub` | REFUSE; remedy prints the `tpub` spelling (testnet BIP-49 / BIP-84) |
| `Upub` / `Vpub` | REFUSE; remedy: supply the full testnet multisig descriptor (or a BlueWallet file) — no single-key remedy exists |
| `zpub`, `Ypub`, `Zpub` | ACCEPT (device set); device normalises to `xpub` in the canonical, as today |

## What changes

**Nothing.** No spec section, no vector row, no plan task. P1.0 is discharged; P1.1
implements the spec as written. `#ruling-needed` comes off the entry. Recommended (controller
action, not gating): file **F-426** — "fork's `ParseExtendedKey` classification switch lacks
the `ypub` case; add it (and consider the upstream PR), then refresh
`SPEC_descriptor_input.md` §4.3/§6/§7 to the widened device set" — repo: **seedhammer fork**
(+ spec refresh in mnemonic-engrave), owning phase: **with S2's firmware build**, tags
`#fork` `#descriptor` `#device-parity`. A reopen of host-side normalisation before then
requires a new measurement — operators actually hitting the `ypub` refusal in numbers — not
a re-argument of this brief.

## F-413 FOLLOWUPS entry — text to record

> **F-413 — RESOLVED (ruling 2026-08-29, architect consult per the overnight mandate,
> fourth in the F-410/F-411/F-412 line;
> `design/agent-reports/RULING_f413_slip132.md`). REFUSE STANDS; entry closed.** The spec
> as written is confirmed: `me` admits exactly the device's five versions and refuses
> `ypub`/`upub`/`vpub`/`Upub`/`Vpub` with §6's per-version executable remedy. Decisive:
> the commonest SLIP-132 arrival is a BARE `ypub`, and there normalisation is not a byte
> swap — a bare converted key promotes to `pkh(…)`, a different wallet (§6's own
> measurement) — so the safe transform is a new §4.5 promotion row, not a spelling fix;
> meanwhile `upub`/`vpub` bare are barred by the standing testnet-promotion refusal and
> `Upub`/`Vpub` are cosigner keys, so a byte swap could only ever serve the rare
> in-descriptor shape, against reopening GREEN text (§4.3, §5.5, §6, §7's `neither`
> floor). The refusal's printed conversion is the same transform with the operator's
> hands on it — consistent with F-422's status-quo ruling and F-417. The zpub/ypub
> asymmetry is the DEVICE's (one missing classification case, verified at source);
> its durable fix is device-side and is filed as F-426 (with S2's firmware build).
> Reopening host-side normalisation requires measured operator friction, not
> re-argument. No spec, vector, or plan change; P1.0 discharged, P1.1 builds
> spec-as-written.
