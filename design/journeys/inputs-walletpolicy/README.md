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

### They share a fingerprint, and that is worth being precise about

All four carry master fingerprint `73c5da0a` because they are **accounts 0..3 of
one seed**. Derived and checked, not assumed:

```
48'/0'/0'/2'  xpub6DkFAXWQ2dHxq2vatrt9…   (@0)
48'/0'/1'/2'  xpub6DzhyrnFFYQ1HimDiM38…   (@1)
48'/0'/2'/2'  xpub6EGx8sPr9FxPPE1rbZaz…   (@2)
48'/0'/3'/2'  xpub6E6Z3Ss5TXJYNJp4U1q3…   (@3)
```

**Not key reuse** — four distinct xpubs, and the device's duplicate-key refusal
has nothing to fire on. But it is a **single-seed** policy, which for a real
four-cosigner wallet would defeat the point: one seed spends everything. It is
here because it is the conformance corpus's wallet, and that corpus is what makes
the journey's device-vs-host comparison checkable against something other than
one run of one script.

**The declared origin USED to be wrong for three of the four (F-217), and is
fixed.** The transcript passed `--path "48'/0'/0'/2'"`, which flattens per-key
origins to Shared, so the card claimed all four keys live at account 0. A
`(fingerprint, path)` pair names exactly one key under BIP-32, so that card
described a wallet that cannot exist — and `md encode` now refuses it.

The origins live in the template instead (`@0/48'/0'/0'/2'/<0;1>/*`), which is
where md has always taken them. **The addresses did not change**, which is the
whole lesson: they come from the xpubs the card carries, so the journey's
device-vs-host comparison passed identically against the impossible card.

The single-seed property above is recorded rather than papered over — it is the
conformance corpus's wallet, and a journey is a record of a run.