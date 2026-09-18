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

## 3. The invariant: ≤ 2 distinct words

> **Every seed in a device-shipped demo payload must have at most 2 distinct
> words in its phrase.**

`abandon…about` = 2, `zoo…wrong` = 2, `beef×12` = 1. A real 12-word seed with
≤2 distinct words has probability ~(2/2048)¹² — it does not happen.

This is chosen over an allowlist of known entropies because it encodes the
property that actually protects a person: **someone reading the plate can tell
it is fake.** An allowlist protects a list; this protects the reader. It also
cannot be satisfied by adding a line to a constant.

## 4. The confinement guard NARROWS, it does not go away

`cmd/emu/embed_confinement_test.go::TestEveryEmbeddedPayloadIsStructurallyConfined`
today requires every `//go:embed` under `cmd/emu` to live in a `//go:build js`
file, so no demo payload can reach a device build. A device payload breaks that
by design.

It must be **narrowed, not deleted**:

- an embed in a non-`js` file remains forbidden **unless** it is the demo
  payload and every seed it carries passes §3;
- the guard keeps discovering embeds by AST rather than by name, which is why
  the existing blobs could be added without editing it and why this one can be;
- its INCONCLUSIVE arms (too few files parsed, no embeds found) stay — they are
  what stop the guard silently protecting nothing.

Today the guard protects by keeping payloads OFF the device. After this it
protects by **proving what is in them**, which is the stronger property.

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

- **Firmware size.** The payload adds flash. Fork `main` measured 1,506,884 B
  flash / 62,592 B RAM at `321acb56`; the budget and the payload's cost must be
  measured before this is called done, not estimated.
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
