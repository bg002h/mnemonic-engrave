# The CLI spine — what you drive on the Mac

Roughly ten minutes, while the room taps the emulator. Every command below was
RUN and its output pasted, not written from memory. Re-run `./rehearse.sh`
before you travel: it executes all of it and diffs against what is recorded here.

**Before you go — install BOTH from git, not crates.io:**

```sh
cargo install --git https://github.com/bg002h/descriptor-mnemonic md-cli
cargo install --git https://github.com/bg002h/mnemonic-secret    ms-cli
```

Measured, not assumed: **published `md-cli` 0.13.0 has no `compose` subcommand
at all** (it answers *"tip: a similar subcommand exists: 'compile'"*), so §1
below — the opening of the talk — does not run on it. Published `ms-cli` 0.14.0
has no `--in` on `split` and does not refuse a seed on argv, so §3 does not run
on it either.

(The copy-paste blocks on the demo *web page* are a different set — only
`decode`, `repair` and `inspect` — and those were verified to work on published
`md-cli`. Don't confuse the two install stories.)

---

## 0 · The frame (say this, don't type it)

> A wallet policy says who can spend, and when. Multisig is one shape of that.
> This device builds the policy itself, engraves it into steel, and never needs
> to trust the computer it's plugged into. Here's the same thing on a laptop.

---

## 1 · Four wallets, four shapes

```sh
md compose --wrapper wsh --preset 'plain-multisig,2of3'
```
```
wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))
note: stdout is a keyless descriptor template (no keys)
```

> `@0`, `@1`, `@2` are slots, not keys. The policy exists before anybody's keys
> do — that's what makes it engravable and what makes it checkable.

```sh
md compose --wrapper tr --preset 'decaying-multisig,2of3,2of5,older1=1000,older2=26280,after=900000'
```
> Three tiers: 2-of-3 after 1000 blocks, a wider 2-of-5 after 26280, and a
> single key after block 900000. A wallet that gets easier to spend as time
> passes — inheritance without a lawyer.

```sh
md compose --wrapper wsh --preset 'hashlock-gated,sha256=2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881,older=4320'
```
```
wsh(or_i(and_v(v:pkh(@0/...),sha256(2d711642...)),and_v(v:pkh(@1/...),older(4320))))
```
> Path one needs a key AND a secret phrase. Path two needs a different key, but
> only after a month. A second factor that isn't a key.

```sh
md compose --wrapper tr --path '1of1,older=32768'
```
```
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,and_v(v:pk(@0/48'/0'/0'/3'/<0;1>/*),older(32768)))
```
> One key, and it cannot touch a coin younger than 32768 blocks — about 227
> days. A wallet you deliberately cannot panic-sell from. That long hex is the
> NUMS point: a taproot internal key nobody holds, so the only way to spend is
> the script path.

**The contrast worth naming:** the device offers six vetted archetypes plus a
free-form path builder. The CLI composes any path list. Same lowering rules,
same output.

---

## 2 · Scratch a plate and watch it heal

```sh
md decode md1yqfdsqsjuqqpr5e55uzqqgqqqrqqvf4d7h59r2
```
```
md: codec error: codex32 decode error: BCH checksum verification failed
```
```sh
md repair md1yqfdsqsjuqqpr5e55uzqqgqqqrqqvf4d7h59r2
```
```
# Repair report
#   md1 chunk 0: 4 corrections at position 5: 'q' -> 's', position 11: 'p' -> 'c', position 18: 'z' -> 'q', position 25: 'r' -> '6'
md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
```

> Four characters wrong. It names every one and hands the policy back intact.
> That's the point of engraving into steel with an error-correcting code: the
> failure mode is a scratch, so the encoding is built to survive scratches.

Then the payoff — that repaired string is the zen-hodl wallet from §1:

```sh
md inspect md1yqfdsssjuqqcr5e55uqqqgqqq6qqvf4d7h59r2
```
```
template: tr(50929b74...ac0,and_v(v:pk(@0/<0;1>/*),older(32768)))
wallet-descriptor-template-id: 73c33a5dea17b45376a6246995df0cbb
wallet-policy-id: 891b6c372c99c033c98bc80604deb6f0
```

> **Anyone who finished mission 3 on their phone is looking at that same
> number.** Their device ran Go. This ran Rust. They share no code. Two
> implementations agreeing on what a wallet *is*.

---

## 3 · Split a seed, and prove the threshold holds

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" > seed.txt
ms split --in seed.txt -k 3 -n 5 --group-size 0 > shares.txt
```
> A published all-zeros test vector. Five shares. This is codex32 — BIP-93 —
> Shamir's secret sharing over the same alphabet the plates use.

```sh
sed -n '1p;3p;5p' shares.txt > three.txt && ms combine --in three.txt
```
```
entropy: 00000000000000000000000000000000
phrase: abandon abandon ... about
```
```sh
sed -n '1p;3p' shares.txt > two.txt && ms combine --in two.txt
```
```
error: not enough shares: have 2, need 3
```

> Any three rebuild it. Two rebuild nothing. Not a policy — arithmetic.

**Then the one that always lands:**

```sh
ms split --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" -k 3 -n 5
```
```
ms: argument 3 on ARGV ... is a BIP-39 mnemonic, 93 characters long.
      Refused BEFORE the command line was parsed; nothing was read and nothing was written.
      ... `ps` shows it, and your shell has ALREADY written the line to its history.
```

> It refuses to let you put a seed on a command line, and tells you why. The
> published build on crates.io is a year older and *only* accepts it that way —
> which is why the install line points at git, and worth saying out loud.

---

## 4 · Close

> Everything you just watched runs on a device with no camera, no network and
> no USB data path. It reads NFC and it cuts steel. The policies are BIP-388,
> the strings are BCH-protected, and the identity hash is checkable by anyone
> with the other implementation.

---

## Notes for the 2017 Intel Mac

- `cargo install --git` builds from source: **do it before you travel**, not in the room.
- Both tools are `x86_64` native there; nothing needs Rosetta.
- If the network is hostile, the emulator runs from `dist/` with
  `python3 -m http.server` — already on macOS — and the CLI half needs no
  network at all once installed.
