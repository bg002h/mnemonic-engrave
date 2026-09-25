# The CLI spine — what you drive on the Mac

Roughly ten minutes, while the room taps the emulator. Every command below was
RUN and its output pasted, not written from memory. Re-run `./rehearse.sh`
before you travel: it executes all of it and diffs against what is recorded here.

**Before you go — download the binaries. No Rust, no compiling.**

Every constellation CLI now ships prebuilt for linux (amd64/arm64), macOS
(amd64/arm64) and Windows (amd64). For your 2017 Intel Mac, `macos-amd64` is the
one:

```sh
# md -- the descriptor tool
curl -LO https://github.com/bg002h/descriptor-mnemonic/releases/download/descriptor-mnemonic-md-cli-v0.20.3/md-0.20.3-macos-amd64.tar.gz
# ms -- the seed tool
curl -LO https://github.com/bg002h/mnemonic-secret/releases/download/ms-cli-v0.20.1/ms-0.20.1-macos-amd64.tar.gz

tar -xzf md-0.20.3-macos-amd64.tar.gz
tar -xzf ms-0.20.1-macos-amd64.tar.gz
chmod +x md ms && ./md --version && ./ms --version
```

macOS Gatekeeper will quarantine an unsigned downloaded binary. Clear it with
`xattr -d com.apple.quarantine ./md ./ms`, or right-click → Open once. **Say this
out loud if anyone downloads on the day** — it is the single most likely thing
to make a binary look broken.

Each release carries `SHA256SUMS.portable`; verify before running:

```sh
curl -LO https://github.com/bg002h/mnemonic-secret/releases/download/ms-cli-v0.20.1/SHA256SUMS.portable
shasum -a 256 -c SHA256SUMS.portable --ignore-missing
```

Only `mnemonic-engrave` is signed (minisign); the others ship checksums alone,
and their `VERIFY.txt` says plainly that a checksum proves integrity, **not
origin**. Do not overstate it to the room.

Building from source still works if anyone prefers it —
`cargo install --git https://github.com/bg002h/descriptor-mnemonic md-cli` —
but note published crates.io builds are OLDER and lack `md compose` entirely.

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

> That is one combination. The claim is about *any* three — so don't assert it,
> check it. There are only twenty cases.

```sh
for t in 123 124 125 134 135 145 234 235 245 345; do
  sed -n "$(echo "$t" | sed 's/./&p;/g')" shares.txt > pick.txt
  got=$(ms combine --in pick.txt 2>/dev/null | sed -n 's/^phrase: //p')
  [ "$got" = "$(cat seed.txt)" ] && printf "%s ok  " "$t" || printf "%s FAIL  " "$t"
done; echo

for p in 12 13 14 15 23 24 25 34 35 45; do
  sed -n "$(echo "$p" | sed 's/./&p;/g')" shares.txt > pick.txt
  ms combine --in pick.txt >/dev/null 2>&1 && printf "%s LEAKED  " "$p" || printf "%s refused  " "$p"
done; echo
```
```
123 ok  124 ok  125 ok  134 ok  135 ok  145 ok  234 ok  235 ok  245 ok  345 ok
12 refused  13 refused  14 refused  15 refused  23 refused  24 refused  25 refused  34 refused  35 refused  45 refused
```

> Twenty for twenty. **Any** three is sufficient; **any** two is not. And two
> shares don't merely fail to be *accepted* — they don't contain the seed, so
> there is nothing in them for anyone to extract. Not a policy — arithmetic.

**If someone asks whether the loop is just printing `ok` — and someone will:**

```sh
cp shares.txt shares.bak
# scratch one character of share 2 -- flip it to a DIFFERENT bech32 char, so it
# always changes (a fixed 's/./q/40' is a no-op ~1/32 of the time)
old=$(sed -n 2p shares.txt | cut -c40); sed -i "2s/./$([ "$old" = q ] && echo p || echo q)/40" shares.txt
```
```
123 FAIL  124 FAIL  125 FAIL  134 ok  135 ok  145 ok  234 FAIL  235 FAIL  245 FAIL  345 ok
```

> Exactly the six triples containing share 2 fail. The four that avoid it still
> rebuild the seed. The loop is reading the shares.
> Then `cp shares.bak shares.txt` to carry on.

**Then the one that always lands — and then the part people actually need:**

```sh
ms split --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" -k 3 -n 5
```
```
ms: argument 3 on ARGV ... is a BIP-39 mnemonic, 93 characters long.
      Refused BEFORE the command line was parsed; nothing was read and nothing was written.
      ... `ps` shows it, and your shell has ALREADY written the line to its history.
```

> It refuses to let you put a seed on a command line, and tells you why.

### But sometimes argv really is safe — and then it must not fight you

**This is the half worth showing.** A tool that only ever says no teaches people
to route around it. On a single-user air-gapped box, an amnesic Tails session, or
an offline Blockstream-satellite node there is no other user to read `/proc`, no
network, and — on Tails — no history that survives the session. The threat the
refusal is modelling is simply not present.

`ms` says so itself, in the refusal, and gives you the door:

```sh
ms split --phrase "<your words>" -k 3 -n 5 --group-size 0 --allow-argv-secret
```
```
share 1 of 5:
ms13zereq3arxz33xqwgeswner6uet85ncm6sgsukqw9yykxzy
...
warning: stdout carries private key material (can spend) — redirect or encrypt
```

> Exit 0, five shares. **The other warnings do not go away** — it still tells you
> stdout carries spendable material. Opting out of one protection does not opt
> you out of the rest.
>
> The flag is deliberately **greppable**: `--allow-argv-secret` in a script is a
> thing a reviewer can find, which is the point of spelling it out rather than
> having an `ARGV_OK=1` environment variable nobody ever sees.

**Same escape hatch across the constellation** — `md`, `mk`, `mt` and `mnemonic`
all take `--allow-argv-secret`, with the same meaning.

### And if you already typed it before reading any of this

The refusal tells you how to clean up, and the instruction that matters is the
one people get wrong: **match on the COMMAND, never on the secret** — grepping
for your own seed types it into history a second time.

```sh
zsh:    fc -W; sed -i '/\bms split\b/d' "$HISTFILE"; h=$HISTSIZE; HISTSIZE=0; HISTSIZE=$h; fc -R
bash:   history -w; sed -i '/\bms split\b/d' "$HISTFILE"; history -c; history -r
fish:   history clear-session
```

> Run **all** of the steps: the entry is still in the shell's MEMORY, so editing
> the history file alone changes nothing and the shell writes it back at exit.
> And `shred -u` any file you pasted from.

---

## 3b · Two recoveries the toolkit can already do

`mnemonic` (mnemonic-toolkit v0.104.0) is the constellation's swiss-army tool.
Two of its searches are worth showing, because they answer questions people
actually arrive with. **Both were run to produce the output below.**

### "I forgot my passphrase"

You have the seed. You have the wallet's xpub. You have *some idea* what the
passphrase was.

```sh
printf 'hunter2\ncorrect horse\nbitcoin\nsatoshi\nnakamoto\n' > candidates.txt

printf '<your 12 words>' | mnemonic xpub-search passphrase-of-xpub \
  --phrase-stdin --passphrase-candidates-file candidates.txt \
  --target-xpub xpub6CvHDtn5otAu9fjb7mPpbfizn2A31pQkwLsogfkHaMfsnoGVCwRidN6rZryTBE6G8b6MF152XgJSKiEBpgt3Jx7udU43auRCHB1hvJTRuBu
```
```
match: candidate on line 4 derives the target xpub at m/84'/0'/0' (template=bip84, account=0)
searched: 140 candidate paths per passphrase
```

> It tries each line against BIP-44/49/84/86 + BIP-48 across an account range —
> 140 paths per candidate — and stops at the first hit. **It reports the LINE
> NUMBER, not the passphrase**: your secret does not go to stdout unless you ask
> for `--json`. Five candidates took 31 ms.

**Where does the candidate list come from?** Not from `mnemonic` -- generating
guesses is not its job, and it says so. It is the job of **John the Ripper**,
which everyone in this room has heard of as a password cracker but which here is
just a candidate *generator*: `--stdout` mode prints guesses and computes no
hashes, cracks nothing. You give it the roots you half-remember and the habits
you actually use; its mangling rules expand them.

```sh
# install the "jumbo" build: brew install john-jumbo  |  apt-get install john
#                            |  pacman -S john

printf 'Satoshi\nhodl\nbitcoin\n' > roots.lst   # the words you half-remember
cat > mangle.rules <<'RULES'
[List.Rules:Demo]
:
l
u
c
c $1
c $!
l $2 $0 $0 $9
RULES
john --config=mangle.rules --wordlist=roots.lst --rules=Demo --stdout
```
```
Satoshi        satoshi        Satoshi1      Satoshi!      satoshi2009
hodl           SATOSHI        Hodl1         Hodl!         hodl2009
bitcoin        HODL           Bitcoin1      Bitcoin!      bitcoin2009
               BITCOIN
               Hodl
               Bitcoin
```

> Three roots and seven habits -> 18 candidates: as-is, lowercased, upcased,
> capitalised, with a `1`, with a `!`, lowercased-plus-a-year. The stock
> rulesets (`--rules=Jumbo`, `--rules=Single`) generate thousands more on their
> own; the seven-line file just keeps this legible and its output stable.

Now pipe that straight into the search -- **process substitution, so the list
of guesses never touches disk**:

```sh
printf '<your 12 words>' | mnemonic xpub-search passphrase-of-xpub \
  --phrase-stdin --target-xpub <your xpub> \
  --passphrase-candidates-file <(john --config=mangle.rules --wordlist=roots.lst --rules=Demo --stdout)
```
```
match: candidate on line 4 derives the target xpub at m/84'/0'/0' (template=bip84, account=0)
searched: 140 candidate paths per passphrase
```

> Line 4 is `satoshi` -- the lowercase mangle of the "Satoshi" you'd have typed.
> Two tools composing: John generates, `mnemonic` verifies, and neither the
> guess list nor the winning passphrase is ever written down. This is your own
> wallet and your own forgotten word; that is the whole point.

**What `mnemonic` still cannot do**, because someone will ask: it targets an
xpub rather than an address. That one is filed.

### "I have three cosigner cards and no idea which slot is which"

A keyless multisig template engraves the wallet TYPE — one plate serves
thousands of wallets. The keys are supplied at restore. But which key is `@0`?
Guess wrong and you get a different wallet, silently. `mnemonic` says so:

```
warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED
         key→slot assignment WITHOUT verifying it. A wrong assignment produces
         a wrong wallet silently.
```

So give it the one thing you do have — **an address you know is yours**:

```sh
printf '<your 12 words>' | mnemonic restore --from phrase=- --md1 <template-md1> \
  --account 0 --cosigner <xpubC> --cosigner <xpubB> \
  --search-address bc1q4vxm2xewpdj2ycxyh9c4w923pjyl0e8dnf0y0gvwz2daxg36mecqhzarw0 --count 1
```
```
multisig wallet completed from template:
  first recv: bc1q4vxm2xewpdj2ycxyh9c4w923pjyl0e8dnf0y0gvwz2daxg36mecqhzarw0
  your seed completes cosigner slot @0
```

> The cosigners went in **unpositioned and in the wrong order**. It searched the
> key→slot assignments for the one whose scriptPubKey matches your address, and
> told you which slot your own seed fills. 30 ms.
>
> It refuses to guess: a `Unique` answer is only returned after proving there is
> no SECOND assignment that also matches — it scans the whole space rather than
> stopping at the first hit. Large spaces hit a cost ceiling that requires
> `--accept-search-time` to proceed.

### "I only wrote down the wallet-id"

The same search runs off the **wallet-id** alone — no address needed. This is
the one to reach for when the engraving records an id (the template form prints
one precisely so you can):

```sh
printf '<your 12 words>' | mnemonic restore --from phrase=- --md1 <template-md1> \
  --account 0 --cosigner <xpub> --cosigner <xpub> \
  --expect-wallet-id 72d94d49b0aca695 --count 1
```
```
  first recv: bc1q4vxm2xewpdj2ycxyh9c4w923pjyl0e8dnf0y0gvwz2daxg36mecqhzarw0
✓ wallet-id (completed): 72d94d49b0aca695055b3de0a1f13bea
  your seed completes cosigner slot @0
```

**And it refuses to guess — show this, it is the best part:**

| you supply | what happens (v0.100.0+) |
| --- | --- |
| `72` (2 hex) | **refused** — below the 2-byte floor |
| `72d9` (4 hex) | ≥2 matches → **lists candidates**, reconstructs none; 1 match → completes with `UNIQUENESS NOT PROVEN` |
| `72d94d49` (8 hex) | same band as above |
| `72d94d49b0ac` (10 hex) | **accepted** — the uniqueness threshold for this space |
| `72d94d49b0aca695` (16 hex) | accepted → the right wallet |
| `deadbeef…` (wrong) | **`✗ NO MATCH`**, exit 4 |

> **Changed in v0.100.0.** A short id used to be refused outright. Verified
> against the shipped binary: only a 1-byte prefix still refuses; 2 and 4 bytes
> now enter the enumerate band. Demo it by supplying `72d9` and reading the
> warning aloud — it names the bytes you gave AND the bytes it wanted, which is
> the part operators remember.
>
> A candidate list is deliberately **not importable**: rows carry an id, the
> key→slot assignment and an address, and NO descriptor. Under `--json` it emits
> `candidates`, never `wallets`, so a script reading `.wallets[0].descriptor`
> cannot silently pick up candidate #1.

> It sizes the requirement to the search space and **tells you the number it
> wants**, rather than accepting whatever you typed and hoping. For this 3-slot
> wallet that number is 5 bytes = 10 hex; a bigger search space demands more. A
> wrong id produces NO wallet — never a plausible wrong one. That is the
> behaviour you want from anything that reconstructs a wallet from parts.
>
> Say the number out loud when demoing: the refusal message contains it
> (`need ≥5 bytes … got 4`), so the tool is not just saying no, it is saying
> exactly what would work.

### Short on id? Reach for the address — it is the better key anyway

If fewer digits were recorded than the tool asks for, that is not a dead end.
A known receive address answers the same question with no prefix-length
argument at all:

```sh
printf '<your 12 words>' | mnemonic restore --from phrase=- --md1 <template-md1> \
  --account 0 --cosigner <xpub> --cosigner <xpub> \
  --search-address <a known receive address>
```

An address matches on the **full scriptPubKey**, so unlike an id prefix it is
collision-free. `restore --help` says so itself: *"Recommended over
`--expect-wallet-id` (full-scriptPubKey match — collision-free)"*. Default scan
is receive indices `0..20`; `--search-addr-min` / `--search-addr-max` widen it,
`--search-chain` reaches the change branch.

**ON A REAL WALLET, USE THE mk1 CARDS — this is the trap.** The demo's fixture
has cosigners with **no recorded origin**, which is the only reason
`--cosigner <xpub>` completes it (verified: the completed descriptor carries
`[00000000]xpub...`, and rebuilding that fixture reproduces the demo's ids
`72d94d49b0aca695055b3de0a1f13bea` and `d3c8c613ed3baa89e1fa4f7ae632919a`
byte-for-byte). A bare xpub decodes to `fingerprint: Fingerprint::default()` +
`origin: DerivationPath::master()` (`restore.rs:2160-2162`), so against a wallet
whose cosigners DID record fingerprint+path it can never match — the operator
gets `✗ NO MATCH` with a correct id and a correct address, and will reasonably
conclude the id is wrong. Pass the `mk1` chunks to `--cosigner` instead.

Measured: this is NOT rescued by `--search-address`. The scriptPubKey is
origin-independent, but the OWN key's derivation path is inferred from the
cosigners' origin metadata in both modes, so a bare-xpub cosigner forces the
same canonical BIP-48/account-0 fallback either way. Against a real-fingerprint
template both modes return `NO MATCH`.

**ONE OR THE OTHER — the tool now refuses the combination.** Since v0.100.0
`--expect-wallet-id` and `--search-address` are mutually exclusive (clap
`conflicts_with`, on `restore` AND `verify-bundle`), so supplying both is a
usage error naming both flags.

> Corrected 2026-09-17, twice. This section originally read "USE BOTH TOGETHER
> … together the answer is fully determined". That was false: the dispatch is
> `if id_search { … } else if addr_search { … }`, so the address was **silently
> ignored**. Filed as R0 finding I7 and fixed in v0.100.0 — the silent-ignore is
> now a refusal, which is why this section changed from "do not pair them" to
> "the tool will not let you".

What each pins still differs, and is worth saying:

**One honest caveat if anyone is paying close attention:** with `sortedmulti`
the script sorts the keys, so the address is order-independent and more than one
assignment reproduces it. The completed wallet-id therefore need not equal the
one from an explicit placement — as observed here, `d3c8c613…` vs `72d94d49…`
with an identical address. For sortedmulti that IS the same wallet -- verified: identical
address, identical spending. But do not claim it recovered "the original
order", and do not let a differing id alarm anyone: under BIP-67 the script
sorts the derived pubkeys at EVERY index, so swapping two cosigners' labels
gives byte-identical scripts forever. Measured, for the bound: with unsorted
`wsh-multi` the two orderings give DIFFERENT addresses
(bc1q734855... vs bc1q9mxq6v9...), so this ambiguity exists for sortedmulti
ONLY. Supplying --expect-wallet-id removes it entirely, which is why the
section above exists.

---

## 4 · Close

> Everything you just watched runs on a device with no camera, no network and
> no USB data path. It reads NFC and it cuts steel. The policies are BIP-388,
> the strings are BCH-protected, and the identity hash is checkable by anyone
> with the other implementation.

---

## Notes for the 2017 Intel Mac

- **`mnemonic` (mnemonic-toolkit) is prebuilt for every platform** (crate
  `mnemonic-toolkit`, binary `mnemonic`); no source build in the room, nothing
  needs `cargo`. **The version split this paragraph used to describe is closed:**
  macOS, Windows and both Linux `musl` builds all ship from the SAME release,
  v0.101.0. It said "the Linux musl binary is v0.97.0 for now ... with a v0.98.0
  Linux build on the way" through two releases after that was true, which is the
  failure mode to watch for here -- a note written while something was pending
  reads as current long after it lands.
- Both tools are `x86_64` native there; nothing needs Rosetta.
- If the network is hostile, the emulator runs from `dist/` with
  `python3 -m http.server` — already on macOS — and the CLI half needs no
  network at all once installed.
- **If someone on Linux says the download 404s, they are not wrong.** The two
  tools use different Linux suffixes: `md` ships glibc `linux-amd64`, `ms`
  ships static `x86_64-linux-musl`. There is no `ms-*-linux-amd64`. The page's
  table has both columns, and a ready-to-paste Linux x86-64 block under it.
