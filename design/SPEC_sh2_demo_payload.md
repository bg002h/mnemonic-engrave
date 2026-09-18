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
- ~~**Digest pinning.**~~ Not applicable: those digests pin blobs embedded in
  emulator source. This payload is a flashed artifact, and `me seal` already
  prints a **public data hash** the device echoes for tamper-detection — the
  same property, delivered by the tool.
- ~~**Record inventory.**~~ Still worth writing down, but in the payload's build
  script rather than a Go doc comment, since there is no Go file.
- ~~**Template vs concrete selection.**~~ **ANSWERED — see §9.** It already
  exists on the device.
- ~~**Does the demo engrave words at all?**~~ No. The payload carries `key:`
  records, not seeds (§8), so §5's accepted risk is the whole safety story and
  §2's self-labelling applies to the wallets the keys derive from.

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

## 8. ANSWERED — no tooling changes, but it is `me sysw pack`, not `me seal`

> **SUPERSEDED IN TWO PLACES, 2026-09-18, by building the payload.** The
> conclusion — *no new tooling* — survives. The command and the region do not.
> Both errors are recorded in full at §8a rather than edited away, because each
> one would have cost a flash cycle and the reasoning that produced them is the
> part worth keeping.

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

## 8a. The two corrections — MEASURED 2026-09-18 while building it

**(1) `me seal` cannot carry a `key:` record. `me sysw pack` can.**

§9 step 2 below prescribes `me seal --plaintext key:… --out demo.uf2`. Run, on
both of `seal`'s channels:

```console
$ me seal --plaintext "key:6162" --out /tmp/t.uf2
me: record 0 in the public section: unrecognised record:
    unrecognized HRP 'key:6' (expected md, mk, ms, or mt)

$ me seal --in records.txt --out demo.uf2
me: record 0 in the secret section: unrecognised record:
    unrecognized HRP 'key:5b37336335646' (expected md, mk, ms, or mt)
```

`me seal` carries constellation STRINGS — md1/mk1/ms1/mt1. The composer record
classes (`key:`, `hash:`, `now:`) are a `me sysw pack` surface, and `pack --help`
documents them by name. §8 measured the seal route with **mk1 chunks**, which it
does carry, and generalised to `key:` records without running one.

**(2) The region is `0x10D00000`, and §8's "correction" of it was wrong.**

§8 carried a paragraph correcting the review's `0x10D00000` to `0x10E00000` as
"the normative constant", citing `seal.PayloadAddr`. That citation is right and
irrelevant: `0x10E00000` is the **Sealed Payload** region, and the composer reads
the **systemwide** container — `ctx.sysw.takeAll(sysw.ClassKey)`
(`gui/composer_sources.go`), whose region is `0x10D00000`–`0x10D10000`
(`SPEC_systemwide_payloads.md` §4). The review was right the first time.

The two are deliberately a megabyte apart, and `me sysw --help` states why: *"A
different container from `seal`, in a different flash region, read by a different
set of programs… no invocation should be able to produce a systemwide container
while the operator believes they are producing a Sealed Payload one."* §8 did
exactly what that separation exists to prevent, on paper — and §9's own code
citation (`ctx.sysw`) contradicted §8's mechanism for a day without either
noticing the other.

**The route that works**, and the one `demo/sh2/build-payload.sh` now runs:

```sh
me sysw pack --in records.txt --no-passphrase --region --out demo-payload.bin
picotool load --verify -t bin -o 0x10D00000 demo-payload.bin   # BOOTSEL, laptop power
```

`--region` pads to the full 64 KiB with `0xFF` (erased NOR), so the file is
byte-for-byte what the sector looks like with only this container written.
`sh2-flash` is NOT the tool: it signs and flashes FIRMWARE images (`-p` picks a
commit to build), and has no payload path.

**Built and verified, 2026-09-18.** 895 bytes of container padded to 65536;
`me sysw show` classifies all three as `cosigner key (key:)` plus an appended
`now:`. The three records were then parsed by the **device's own Go parser** —
`sysw.ParseKeyRecord`, not the Rust host classifier — and all three returned
`depth=4`, `origin=m/48h/0h/0h/2h`. That cross-language check is the one that
matters: the host writes the payload and the firmware reads it, and only the
firmware's answer decides whether a visitor sees three seatable keys or an empty
composer.

**Why the keys come from the mk1 CARDS and not from a printed descriptor.**
`ParseKeyRecord` refuses a record whose BIP-32 header disagrees with its origin
(`sysw/composer_records.go:399-408`): depth must be 3 or 4, must equal the
origin's component count, and the child index must equal the origin's terminal
component. Until md-codec 0.44.0 (2026-09-18) every rendered descriptor served
depth-0 xpubs under depth-4 origins, so records built from one would have been
refused by the device, one by one, with no diagnosis beyond a composer that
offered nothing. The cards always carried real headers. This is why the payload
was blocked on that fix, and it is the reason to keep reading keys off cards.

## 9. ANSWERED — the device already offers the choice

`gui/composer_engrave.go` defines three forms, offered by seating state
(`composerFormsFor`):

| form | engraves | offered when |
| --- | --- | --- |
| `composerFormConcrete` | the keyed policy — text/QR plates or keyed `md1` | every slot seated |
| `composerFormTemplateAndCards` | keyless `md1` **with fingerprints**, plus one `mk1` per seated slot | partially seated |
| `composerFormTemplateOnly` | a key-less composition; no form A, no cards | nothing seated |

Single-sig offers the same choice explicitly — `gui/singlesig.go:194`,
`["Full policy md1", "Template-only md1"]`, behind a warning screen.

And `composerKeySources` (`gui/composer_sources.go:40`) "reads every `key:`
record the payload holds", which is precisely the path a flashed demo payload
feeds.

**So this request is payload authoring, not firmware work:**

1. build `key:` records for the demo wallets — `key:<hex of "[fingerprint/path]xpub">`;
2. `me sysw pack --no-passphrase --region --out demo-payload.bin` (**not
   `me seal`** — see §8a correction 1);
3. flash once, at `0x10D00000` (**not `0x10E00000`** — see §8a correction 2).

Steps 1 and 2 are done: `demo/sh2/build-payload.sh`.

A visitor then composes from the seated demo keys and picks concrete or template
at engrave time. **Zero firmware changes**, and the device offers a richer choice
than the request asked for — the middle form carries fingerprints *and* cosigner
cards.

**One gap, recorded because it is easy to trip over later.**
`templateizeMultisigBundle` — named in `gui/template_engrave.go` as the
multisig counterpart and deferred there to "Task 7" — **does not exist**. That
path strips a device-BUILT bundle to keyless. The composer's
`composerFormTemplateOnly` is a different mechanism: composing keylessly from
the start rather than stripping afterwards. Irrelevant to this demo, because the
composer route is the one payload keys feed; it matters only if someone later
wants "build a full multisig, then strip it to a template".

## 10. What is actually left

Nothing in firmware, and the authoring is now done.

- ~~choose the wallet shape~~ **DONE.** A 2-of-3 `wsh(sortedmulti)` over the
  three §2 seeds, every cosigner at BIP-48 account 0, `m/48'/0'/0'/2'` — which is
  `md.DefaultOrigin(wsh, 0)` on the device, so the keys seat into the composer's
  own wsh presets. Its wallet-policy-id is `9db5d8b6d3a0bcbfd1998679da280f6e`.
- ~~derive the `key:` records and record the commands~~ **DONE.**
  `demo/sh2/build-payload.sh`, which re-checks §3's ≤2-distinct-words invariant
  on the literals, reads each key off its mk1 card, packs, and prints both the
  `me sysw show` verification and the `picotool` line. Built and verified
  2026-09-18, including against the device's own Go `ParseKeyRecord` (§8a).
- **flash once and walk it** — still the only step left, and still the only claim
  reading cannot settle: that a visitor can actually reach both engrave forms on
  hardware.

**A note on reproducibility.** `me sysw pack` appends a `now:` record (the pack
time) whenever the payload holds a `key:`, so two builds differ in that record
and in the digest. That is wanted here — the device echoes it as a lower bound
beside a time lock — but it means the blob is not byte-reproducible and must not
be pinned by a digest const. `--no-now` makes it a pure function of its inputs if
some later fixture needs that.
