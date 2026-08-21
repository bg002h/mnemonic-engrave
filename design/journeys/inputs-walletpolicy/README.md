# Inputs — the Wallet Policy journey

The wallet a *different* operator built, arriving at this device as a card set.

| file | what |
| --- | --- |
| `policy.template` | the BIP-388 template: a depth-2 taproot script tree |
| `key0..3.xpub` | the four cosigner xpubs |
| `origin.path` | the shared origin the template's `tr()` wrapper has no canonical default for |
| `master.fingerprint` | the cosigners' master-key fingerprint |

**The keys are BIP-39's published test mnemonic** ("abandon … about") at
`48'/0'/N'/2'`, master fingerprint `73c5da0a`. Never put funds behind them.

They are the same keys as `seedhammer`'s `keyed_tr_depth2` conformance vector,
on purpose: that vector already pins this policy's addresses against the primary
Rust implementation, so the journey's "the device agrees with the host" claim is
checked against a corpus rather than against one run of one script.

## Why this wallet

Three properties, each load-bearing for what this journey is meant to exercise:

- **A depth-2 taproot script tree.** The Merkle root depends on the tree's
  SHAPE, so this is a policy no flat `bip380.Descriptor` can express — it can
  only reach an address through the complex route Stage 3 added.
- **Seven md1 chunks.** A keyless template of the same policy is one string; the
  keys are what force a chunk set, so the gather is a real multi-card gather
  rather than a single tap.
- **Four distinct cosigners.** A one-key policy would let a wrong key-to-slot
  mapping still produce the right address.

## The fingerprints are not optional

Measured while building this journey: encoding the same template with the same
four xpubs and the same origin, but WITHOUT `--fingerprint`, yields a 7-chunk
card set whose `wallet-policy-id` is `0b3b95a4…` — against `ade5967c…` for the
8-chunk set that carries them. The template id is identical either way
(`6f58bdf7…`), because it is key-stable.

So the fingerprints are part of the wallet's IDENTITY, not decoration, and an
operator who omits them gets a card that proves to a different id than their
coordinator shows. This journey supplies them, and the id it checks against is
the conformance corpus's.
