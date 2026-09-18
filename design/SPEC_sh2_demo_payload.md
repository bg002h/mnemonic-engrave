# SPEC — an on-device demo payload for the SH2

**Status:** DRAFT. **Baseline:** fork `main` (see §7 for the pin).

**NOT funds-adjacent — operator ruling 2026-09-17: *"This is not funds adjacent.
Keep the review tight."*** An earlier draft of this header classified the work
risk-set on the grounds that it ships key material in firmware. That
classification is withdrawn: the only keys involved are public test vectors, so
there are no funds to put at risk that are not already forfeit by definition.
One tight review, not the multi-lens R0 apparatus.

The §5 residual risk is unchanged and already accepted; it is a property of the
demo artifact, not a reason to re-inflate the process around it.

## 1. What and why

The SH2 demo currently has nothing on the device to engrave. The operator wants
a payload shipped in firmware holding demo keys, so a visitor can choose to
engrave either a **template** wallet (keyless, shareable) or a **concrete**
wallet (keyed), without presenting anything over NFC.

Operator ruling 2026-09-17: *"NFC won't work. The keys must be of the abandon
abandon…abandon or beef beef…beef variety. And they can live in payload."*

## 2. The seeds, and why the choice is what makes this safe

Three seeds, all **self-labelling**: a human reading the words recognises a test
vector immediately, and any wallet derived from one is drained by bots the
moment it is funded.

| seed | distinct words | entropy | fp (BIP-87 acct 0) |
| --- | --- | --- | --- |
| `abandon` ×11 + `about` | 2 | `00000000000000000000000000000000` | — |
| `zoo` ×11 + `wrong` | 2 | `ffffffffffffffffffffffffffffffff` | `3f635a63` |
| `beef` ×12 | 1 | `140280500a0140280500a0140280500a` | `66d455ea` |

`beef ×12` is already a constellation fixture (the `final_word` tests, and two
implementation plans), so this introduces no new test material.

**A trap this spec exists partly to record.** "beef" must be specified as a
PHRASE, never as entropy. `0xbeefbeef…` decodes to *"same law room lava winner
jelly wing water use wash use teach"* — which looks exactly like a real seed and
destroys the self-labelling property. Measured, not assumed:

```text
entropy beefbeef… -> same law room lava winner jelly wing water use wash use teach   NOT self-labelling
phrase  beef ×12  -> entropy 140280500a0140280500a0140280500a                        self-labelling
```

## 3. The invariant: ≤ 2 distinct words — PROVEN, not enforced

> **Every seed in a device-shipped demo payload has at most 2 distinct words in
> its phrase.**

`abandon…about` = 2, `zoo…wrong` = 2, `beef×12` = 1. A real 12-word seed with
≤2 distinct words has probability ~(2/2048)¹² — it does not happen.

Chosen over an allowlist of known entropies because it encodes the property that
protects a person: **someone reading the plate can tell it is fake.** An
allowlist protects a list; this protects the reader.

**Proven at construction, not enforced at runtime** (operator ruling
2026-09-17: *"We don't need to enforce the rule, we can have it be proven."*).
The three seeds are literals in the generator, so the property is visible in the
source and testable there directly. The chain that makes it a proof rather than
a comment is the one the existing payloads already use:

1. the generator (`cmd/buildpayloadcards`) takes the seed phrases as literals —
   a test over those literals settles §3 with no decoding at all;
2. the built artifact is pinned by a digest const, asserted by a host test that
   opens the blob and hashes it (`sysw_cards_payload_host_test.go`);
3. regeneration is an operator step the digest test names in its own failure
   message (`go run ./cmd/buildpayloadcards | me sysw pack …`).

Enforcement WOULD be possible — the review confirmed a host test can open a
payload with `sysw.Open` and recover its records, and two existing tests do. It
is simply not needed when the inputs are literals and the output is pinned.

## 4. There is nothing to narrow — the vehicle is the flashed payload region

An earlier draft proposed narrowing
`TestEveryEmbeddedPayloadIsStructurallyConfined` so a device build could carry
one permitted `//go:embed`. **That was written against the wrong mechanism and
is withdrawn.**

Two facts settle it:

- the guard discovers embeds under `cmd/emu` ONLY
  (`embed_confinement_test.go:154`), and a device payload cannot live there —
  `cmd/controller` cannot import a `main` package. The guard structurally
  cannot see the file it would have had to permit;
- the device **already has a payload mechanism**:
  `cmd/controller/platform_sh2.go:576` — *"PayloadReader returns the real XIP
  read over the §5 payload region (§10.1). This is the ONLY platform that has
  one."*

So the demo payload is a **flashed artifact in the existing payload region**,
not an embed. The confinement guard governs `//go:embed` under `cmd/emu` and is
untouched by this work: it keeps doing exactly what it does today, and no demo
blob is compiled into firmware.

This is strictly better than the narrowing it replaces. A guard that had to
learn a content exception would have become a guard with a hole in it; instead
it keeps its simple, structural property, and the demo payload takes the path
the device already supports for payloads.

## 5. Accepted risk, recorded as a decision

A concrete `md1` plate is **opaque**: `md1fcgp9qqpqsgqpsgvzyxqq…`. Unlike seed
words, it does not announce itself as a demo — a reader would have to decode it
and recognise fingerprint `66d455ea`. It is therefore a correct, real-looking
backup of a wallet whose keys are public, and it outlives the demo.

A marker engraved beside the `md1` was proposed and **declined** (operator,
2026-09-17: *"Accept it."*). Recorded here as a decision, not an oversight: the
demo is supervised and its plates are disposable. Anyone revisiting this should
know it was considered and ruled on, not missed.

Unaffected either way: the **template** plate carries no keys and identifies no
wallet, and any **word** plate is self-labelling per §2.

## 6. Open questions for R0

- ~~**HOW does the payload reach the region?**~~ **ANSWERED — see §8.** `me seal`
  already emits a UF2 addressed at the normative payload base; no new tooling.
- ~~**Firmware size.**~~ **RETIRED.** The payload is flashed to its own region,
  not compiled into firmware, so it costs no firmware flash. Measured: 1024
  bytes for a one-card payload.
- **Digest pinning.** Existing blobs pin a digest (`syswTestDigest`) precisely
  so a published document cannot silently drift from the blob it photographs.
  Does the demo payload need the same, and is it photographed anywhere?
- **Record inventory.** `sysw_cards_payload.go` documents every record in-file
  because a previous gap survived by not being stated. This payload must do the
  same: which records, in what order, and which demo step strands without each.
- **Template vs concrete selection.** Where does the choice live — a payload
  property, or a picker on the device? `sysw_cards_payload.go` notes that a
  wrapper choice was a TEMPLATE-PICKER concern and not a payload property; the
  same question applies here and should not be assumed.
- **Does the demo engrave words at all?** If it offers a seed-backup plate, §2's
  self-labelling covers it directly. If not, §5 is the whole safety story.

## 7. Gates before code (tight)

These survive the narrowed scope because each is a machine check, not a review
round — cheap to run and the only things here that can fail silently.

- `TestEveryEmbeddedPayloadIsStructurallyConfined`, narrowed per §4, with a test
  proving it REFUSES a payload carrying a non-self-labelling seed. A guard that
  has never rejected anything is a hypothesis.
- the ≤2-distinct-words check, run against the actual embedded blob rather than
  against the constants used to build it — otherwise it proves the generator,
  not the artifact.
- a firmware size measurement against the budget (§6).
- baseline revision recorded here before R0, so `scripts/plan-staleness-check.sh`
  has something to compare against.

## 8. ANSWERED — `me seal` already does this, and no tooling changes

Measured 2026-09-17, end to end. The feature is substantially cheaper than §4
and §6 assumed.

```console
$ me seal --plaintext <mk1 chunk 1> --plaintext <mk1 chunk 2> --out demo.uf2
me: wrote 1024 bytes to demo.uf2

public data hash (2 records, UNSEALED):
    8388 417c 38bb 6c24 8d25 39af be1e 9f6a
RECORD THIS WHOLE LINE. The device shows the same value; if it
differs, the payload has been altered or its encryption removed.
```

The UF2 is already addressed for the device — verified by DECODING its blocks,
not inferred from the absence of an `--addr` flag:

```text
block 0/2  addr=0x10E00000  len=256  family=0xE48BFF58  magic_ok=True
block 1/2  addr=0x10E00100  len=256  family=0xE48BFF58  magic_ok=True

seal.PayloadAddr (seal/read_tinygo.go, normative) = 0x10E00000
```

**Four consequences, every one a simplification:**

1. **No new tooling.** `me seal` emits a correctly-addressed, flashable UF2;
   `sh2-flash` learns nothing.
2. **No seeds in the payload.** The request was for a payload holding *keys*,
   and `mk1` cards are keys. No BIP-39 mnemonic ships.
3. **So the plaintext guard never engages.** `me` refuses a mnemonic on argv
   outright (tested: *"argument 3 on ARGV is SECRET key material"*) — but we are
   not carrying one, so nothing is carved out and nothing is weakened.
4. **No passphrase ceremony.** With nothing secret inside, the container is
   cleartext by construction: *"NOT SEALED — no record in this payload is secret
   material, so there is nothing to encrypt."* A visitor-facing demo must not ask
   for a passphrase, and now does not have to.

**A correction to the review, recorded so it does not propagate.** The persisted
report cites the payload region as `0x10D00000`. The normative constant is
**`0x10E00000`**, and the difference is not cosmetic: `seal/read_tinygo.go`
warns that a write past `0x11000000` wraps to `0x10000000` and destroys the
firmware (RP2350 datasheet §5.5.2). That is why `me seal` has no `--addr` flag,
and why the address here was verified by decoding blocks rather than trusting
the flag's absence.

**What is left** is only device-side: where the template-vs-concrete choice
lives, and whether the composer already seats cosigners from payload `key:`
records — `me seal --help` says they "feed the SeedHammer II's Wallet Policy
composer", which would make this payload authoring rather than firmware work.
