# Pathological-wallet journey — 2026-09-16

**Question walked:** *"I want to build the nastiest wallet these tools will accept, and find
out where they stop being honest with me."*

**Tools as found on PATH:** `md 0.15.0`, `ms 0.19.0`, `me 0.10.0`, plus `mk`/`mt` from the
same install. Emulator built fresh from `/scratch/code/shibboleth/seedhammer` at
`cmd/emu/build.sh` (11,089,274 bytes, 2026-09-16 07:02:41) and served on 127.0.0.1:8392.
Nothing in any repo was edited; the only write is this file.

Findings are numbered **F-598 onward**.

---

## 1. The wallet I built, and how far it got

Built up in stages rather than starting at maximum. Working directory `/tmp/pathwallet`.

### 1a. The preimages

Four phrases, four kinds, plus the same preimage re-read under all four:

```
$ ms hashlock --kind sha256 --hashlock-phrase-stdin --out pre1.ms1 < p1.txt
hash:443816468ff6e25c543904c92a3b2cdaceca70ef366b5829462b0fc7f608b34c
   preimage (hex):  562182dccf7b5e02d9d241544af4ebfb3787c061381f711ef9966732986320ea
$ ms hashlock --kind hash256   ... < p2.txt -> hash:hash256:a84fda839233750f95aba2d6b778448ff2168b07c4e078ad5808dda2f74db7fd
$ ms hashlock --kind ripemd160 ... < p3.txt -> hash:ripemd160:72616769b60bead812d229190349dc3d5102a70d
$ ms hashlock --kind hash160   ... < p4.txt -> hash:hash160:fe2bb51a8a018fbb137e022e48956865a64b0204
```

Same preimage, all four kinds (`ms hashlock --hex -`, no `--kind`):

```
  sha256     443816468ff6e25c543904c92a3b2cdaceca70ef366b5829462b0fc7f608b34c
  hash256    aae6f48ed518588980f4b556d98249eb331c5d7c739e493df561cd47f1424663
  ripemd160  dc3004b2228ca886cd79dce3d8af04aff87dfc83
  hash160    7ca7e60276a2a10ad4b034edc35c19f006b3fca6
```

### 1b. The maximal composition — REFUSED downstream

Six paths, all four hash kinds, both timelock flavours, three key-less paths, and paths 3
and 4 committing to **the same 32-byte preimage under two different kinds**:

```
$ md compose --wrapper wsh --experimental \
    --path '2of3' \
    --path '2of3,older=26280' \
    --path 'keyless,sha256=4438…b34c' \
    --path 'keyless,ripemd160=dc30…fc83' \
    --path '1of2,hash256=a84f…b7fd' \
    --path 'keyless,hash160=fe2b…0204,after=800000'
exit=0  (3 EXPERIMENTAL warnings, one template on stdout)

$ md encode --in nasty.tmpl --experimental
md: template parse error: miniscript parse failed even with --experimental: Miniscript is
malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits,
repeated keys and timelock mixing still apply)
exit=1
```

That is **F-600**. The composer minted it at exit 0 and nothing downstream will take it.

### 1c. The wallet that actually got all the way through

Moving each hash onto a keyed path (one key-less path left) produced a composition that
encodes, decodes, verifies, renders a descriptor, derives addresses, bundles and packs:

```
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/…,@2/…),
   or_i(and_v(v:multi(2,@3,@4,@5),and_v(v:hash256(a84fda83…f74db7fd),older(26280))),
   or_i(and_v(v:multi(1,@6,@7),ripemd160(dc3004b2…f87dfc83)),
   or_i(and_v(v:pkh(@8),and_v(v:hash160(fe2bb51a…a64b0204),after(800000))),
        sha256(44381646…f608b34c))))))
```

9 key slots, all four hash kinds, `older()` and `after()`, one bearer path.
`md encode` → 5 md1 chunks, `chunk-set-id: 0x38958`, `wallet-policy-id-fingerprint:
0x2c2e1a5d`. All nine xpubs are accounts 0..8 of the *same* seed (`master_fingerprint
73c5da0a`) — a nine-cosigner-looking wallet that is one seed. Nothing anywhere remarked on
that; I do not count it as a finding (no tool is given the fingerprints unless asked).

How far it got: **all the way through the host chain** — `md decode`/`inspect`/`verify`,
`md descriptor --experimental` (with the defect in F-601), `md address`, `mk encode` ×9
cards (26 mk1 chunks), `me bundle` (32 plates), `me sysw pack` (2,766-byte container,
digest `e5ff 2b6b 2e9a b7b8 e9af d2c7 8fe9 3316`). It did **not** get onto the device: see
§4.

---

## 2. Findings

### F-598 — `me sysw show` lists no record at all for a plaintext seed, a passphrase or free text; a container holding only a seed prints as empty

**Severity: Critical.** Classification: **refusal/disclosure missing** — the verb's whole
job is disclosure.

`me sysw show`'s one-line description is *"Print what a container holds, and its digest."*
It does not print BIP-39 mnemonic records, codex32/`ms1` secret records, `pass:` records,
or `text:` records. There is no count, no "N records withheld", no non-zero exit — the
omitted records are simply absent from the output.

Reproduction 1, built with `me` itself:

```
$ printf 'legal winner thank year wave sausage worth useful legal winner thank yellow\n' > seedonly.txt
$ me sysw pack --in seedonly.txt --no-passphrase --no-now --out seedonly.bin
sealing:  NOT SEALED — you passed --no-passphrase, and this payload HOLDS
      SECRET MATERIAL (record 0 (BIP-39 mnemonic)). It will sit in flash in cleartext.
…
$ me sysw show seedonly.bin
sealed:   false
pub_len:  75
ct_len:   0
identity: a78145a82e62054c03c7624882172e60965a97df2c836504139f52369cf825a5
digest:   5099 10d3 bf22 eac3 3394 21fe a0ae a6ea
$ echo $?
0
```

Five header lines, **zero record lines**, exit 0. The file itself:

```
00000030: 0000 0000 6c65 6761 6c20 7769 6e6e 6572  ....legal winner
00000040: 2074 6861 6e6b 2079 6561 7220 7761 7665   thank year wave
…
```

Reproduction 2 — **on a tracked fixture in the fork**, which makes this reproducible with
no setup at all. `cmd/emu/sysw_test_payload.bin` (265 bytes) holds three cleartext records:

```
offset  52  text:5345454448414d4d45522049492044454d4f205041594c4f4144   ("SEEDHAMMER II DEMO PAYLOAD")
offset 110  abandon abandon … abandon about                              (12-word BIP-39 mnemonic)
offset 204  pass:636f727265637420686f727365206261747465727920737461706c65 ("correct horse battery staple")
```

```
$ me sysw show cmd/emu/sysw_test_payload.bin
sealed:   false
pub_len:  213
ct_len:   0
identity: deb91a0dec3d86d642e9f2cee0ea5b02c80088e814f4fb20ffb98d54eb321318
digest:   55ad b800 6ec6 a066 94f3 6a0e 900a c8d5
```

Zero record lines for a 213-byte public section holding a seed **and** a passphrase.

Reproduction 3 — the same file, read by the **device**. Loading that blob in the emulator:

```
Payload Warnings
A SECRET is stored unencrypted in flash.
```

and the Wallet Policy front door then reads `A seed is loaded. It can fill any number of
slots.` So the firmware sees the seed and says so; the host inspection verb sees the same
bytes and says nothing. `cmd/emu/sysw_composer_payload.bin` is the same story: `me sysw
show` prints 4 records (2 × `key:`, 1 × `hash:`, 1 × `now:`) and omits the fifth at offset
707, `legal winner thank year wave sausage worth useful legal winner thank yellow`
(file is 782 bytes, `pub_len: 730`; the four shown records total 655 bytes).

The `text:` case rules out "deliberate redaction of secrets" as a complete explanation —
`text:` is not a secret class (`me sysw pack` on a text-only payload says *"no record in
this payload is secret material"*) and it is omitted too. This is a class-coverage gap in
the renderer: it has labels for `key:`, `hash:`, `now:` and md1/mk1, and drops everything
else on the floor.

**Worse than saying nothing? YES.** `me sysw show` is the verb an operator runs to answer
"what is on this flash image before I write it / store it / hand this machine to someone".
It answers "one cosigner xpub" for an image carrying a seed phrase and a passphrase in
cleartext. A tool that prints a *shorter truthful* list would be no worse than silence;
this prints a *complete-looking* list that is wrong.

**Why the secret-handling ruling does not cover this.** The 2026-08-27 ruling downgrades
*"failure to handle secret material secretly"*. Nothing here leaks — the defect is the
opposite: the tool fails to **disclose** that a secret is present. It is a wrong result
from the verb whose contract is to report contents, which is still blocking.

---

### F-599 — `md encode --experimental` asserts the descriptor has a key-less spend path even when it provably has none

**Severity: Important.** Classification: **warning whose condition is never checked**.

The warning is gated on the *flag*, not on the *descriptor*:

```
$ printf '%s' 'wpkh(@0/<0;1>/*)' > pk.tmpl
$ md encode --in pk.tmpl --experimental
warning: --experimental relaxed the signature rule. This descriptor has at least one spend
path that needs NO key, so whoever learns its preimage can spend it alone. If that preimage
is engraved, THE PLATE IS BEARER ACCESS. Malleability, resource limits, repeated keys and
timelock mixing were still checked.
md1yqpqqxqq8xtwhw4xwn4qh
```

`wpkh(@0/<0;1>/*)` is a single-key wallet with exactly one spend path, and that path
requires a signature. The same false line appears for `wsh(sortedmulti(2,@0,@1,@2))`.
Confirmed in `crates/md-cli/src/cmd/encode.rs:78` — `if args.experimental { eprintln!(…) }`,
with no consultation of the parsed descriptor.

Contrast: `md compose --experimental` gets this right, naming only the paths that are
actually key-less (`warning: EXPERIMENTAL: path 3 has no key`, `path 4`, `path 6` on the
six-path wallet; silent on `--path '2of3'`).

**Worse than saying nothing? YES.** This is the loudest safety line in the toolchain and
the only signal distinguishing a bearer plate from a normal one. Because `md compose`'s own
refusal *prescribes* `--experimental` ("pass --experimental here"), an operator ends up
with it in their standard invocation, and then sees "THE PLATE IS BEARER ACCESS" on every
plate they ever cut — including the ones that are not. A warning that cannot fail is a
warning that stops being read, and it also means the line cannot be used as evidence either
way.

---

### F-600 — `md compose` emits, at exit 0, a template that every downstream `md` verb refuses as malleable, whenever two key-less hash paths are adjacent

**Severity: Important.** Classification: **missing refusal (or missing warning) at compose
time**.

Minimal reproduction — three paths, two of them key-less hashlocks of different kinds,
which is precisely the mixed-kind shape the hashlock-kinds work exists for:

```
$ md compose --wrapper wsh --experimental \
    --path '2of3' \
    --path 'keyless,sha256=443816468ff6e25c543904c92a3b2cdaceca70ef366b5829462b0fc7f608b34c' \
    --path 'keyless,ripemd160=dc3004b2228ca886cd79dce3d8af04aff87dfc83'
compose exit=0
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),
  or_i(sha256(443816468ff6e25c543904c92a3b2cdaceca70ef366b5829462b0fc7f608b34c),
       ripemd160(dc3004b2228ca886cd79dce3d8af04aff87dfc83))))

$ md encode --in minimal.tmpl --experimental
md: template parse error: miniscript parse failed even with --experimental: Miniscript is
malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits,
repeated keys and timelock mixing still apply)
encode exit=1

$ md descriptor --experimental --template "<that template>" --key @0=… --key @1=… --key @2=…
md: template parse error: miniscript parse failed even with --experimental: Miniscript is
malleable (…)
```

Bisected on the six-path wallet: 1, 2 and 3 paths encode; the refusal begins the moment the
**second** key-less hash path is added, and persists at 5 and 6 paths.

The composer emits three EXPERIMENTAL warnings naming each key-less path, so it knows
exactly which paths they are. It does not know, or does not say, that two of them side by
side make the result unencodable.

**Worse than saying nothing? YES.** The composer's only consumers are `md encode`,
`md descriptor` and `md address`; a template is not a deliverable. An operator who
deliberately wants two escape hatches at two different hash functions — the headline use of
four kinds — gets a clean exit 0, a template, and then a wall. The refusal they eventually
hit names miniscript malleability, not "your path 2 and path 3 cannot both be key-less",
so the remedy is not discoverable from it. Telling them nothing at compose time is strictly
worse than the composer refusing (or warning) with the path numbers it already has in hand.

---

### F-601 — `md descriptor` re-serialises supplied xpubs at depth 0 and silently drops the origin when no `--fingerprint` is given; `md decompose` then refuses `md`'s own output and prescribes a fix that would break the wallet

**Severity: Important** (funds-adjacent). Classification: **silent data loss + a refusal
that misattributes blame**.

`md descriptor --help` promises *"the CONCRETE output descriptor -- real xpubs, key origins
and the BIP-380 checksum -- for pasting into a coordinator."*

**(a) The origin is dropped, silently.** Supplying a template that carries inline origins
and real account xpubs, with no `--fingerprint`:

```
$ md descriptor --template "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))" \
    --key @0=xpub6DkFAXWQ2dHxq… --key @1=xpub6Dzhyrn… --key @2=xpub6EGx8sP…
wsh(sortedmulti(2,xpub661MyMwAqRbcGQnC8…/<0;1>/*,xpub661MyMwAqRbcG5161…/<0;1>/*,xpub661MyMwAqRbcGS5QF…/<0;1>/*))#4v9pksfg
note: stdout is watch-only — public keys only, cannot spend
```

No `[fingerprint/path]` anywhere, exit 0, a valid BIP-380 checksum, and the only stderr
note is the generic watch-only one. A descriptor with no key origin cannot be signed by any
hardware wallet — there is nothing for a signer to match against its own master
fingerprint. With `--fingerprint` supplied the origins do appear, and **both variants derive
byte-identical addresses**:

```
A (no --fingerprint):  bc1q2sz6vvu6k7y9gtc6kfgfe0p6xkhmvmdlu97eecjkykpdktvps08scdjgr5
B (with --fingerprint): bc1q2sz6vvu6k7y9gtc6kfgfe0p6xkhmvmdlu97eecjkykpdktvps08scdjgr5
```

so the operator's obvious cross-check — "do the addresses match?" — cannot detect the loss.

**(b) The xpub metadata is rewritten.** The supplied and emitted keys are the same EC point
and chain code with `depth`, `parent_fingerprint` and `child_index` zeroed:

```
supplied @0 depth 4 pfp 1cf29716 idx 2147483650
   cc  bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763
   key 021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29
emitted  #1 depth 0 pfp 00000000 idx 0
   cc  bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763
   key 021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29
```

Even in variant B, the printed descriptor is internally inconsistent: the origin says
`[73c5da0a/48'/0'/0'/2']` (depth 4) on an xpub whose own header says depth 0.

**(c) `md`'s own inverse verb refuses `md`'s own output, and blames the input.**

```
$ md decompose --in roundtrip.desc     # roundtrip.desc is md descriptor's variant-B output
md: decompose: key @0 is depth-inconsistent IN THE INPUT: the extended key states depth 0,
but its origin path `[73c5da0a/48'/0'/0'/2']` has 4 component(s) — depth 4. An mk1 key card
drops depth and child number from the wire and reconstructs BOTH from the origin path, so
`mk encode --keys` refuses such a record outright ("xpub origin-path mismatch: xpub depth 0
… vs origin_path depth 4 …"). decompose will not emit a key line that could not be minted:
correct the origin in the descriptor so it states the path this key was actually derived at.
exit=1
```

The origin is not the wrong half. The origin is correct and the xpub is what `md descriptor`
mangled. An operator who follows the prescription — "correct the origin … to the path this
key was actually derived at", i.e. truncate it to `[73c5da0a]` to match depth 0 — ends up
with a descriptor that claims the key *is* the master key, which derives an entirely
different wallet.

**Worse than saying nothing? YES**, for (a) and (c). For (a): a coordinator import that
looks complete, checksums, and derives the right addresses, yet can never be signed, is a
worse outcome than a refusal saying "no fingerprint supplied for @0, so the origin cannot
be rendered". For (c): the refusal is correct to refuse but the prescribed remedy is
actively harmful, which is worse than "these two disagree, and I do not know which is
right". For (b) alone the verdict is weaker — most consumers ignore xpub depth — but it is
what causes (c).

---

### F-602 — `me bundle` states a total plate count that omits every cosigner card a key-less policy needs

**Severity: Important.** Classification: **a count that omits plates**.

```
$ me bundle --in keyless3.md1          # md1 of wsh(sortedmulti(2,@0,@1,@2)), keyless
me: backup needs 2 plates (1 public + ms1 on device):
  plate 1/2  md1 policy  → push via NFC & engrave
  plate 2/2  ms1 secret  → TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool
```

That backup is not restorable. The card is key-less; restoring needs three mk1 cosigner
cards that nobody has been told to cut. Cut those two plates and the wallet is gone.

Same shape on the pathological 9-slot wallet: given only the 5 md1 chunks, `me bundle`
says *"backup needs 6 plates (5 public + ms1 on device)"*; given the same md1 plus the nine
mk1 cards, it says *"backup needs 32 plates"*. The first number was not a floor, it was
stated as the answer.

Three things make this squarely a defect rather than a scoping choice:

1. `me bundle` **decodes the template** — it names all four hash kinds in the same block
   (`NOTE — this policy has a hashlock path (hash160, hash256, ripemd160, sha256)`), so it
   has the descriptor and knows there are 9 unfilled slots.
2. It already knows how to say "and this is not counted": the hashlock note reads *"Its
   preimage is NOT one of these plates and is not counted above"*. The pattern exists; the
   cosigner cards just are not in it.
3. `me` itself calls this failure mode out by name elsewhere — `me sysw pack --expect`'s
   help: *"A backup can be silently incomplete … A plate is then cut from a wallet nobody
   can restore. `--expect descriptor,cosigner` turns that into a refusal."* And that gate
   really does fire (§3). `me bundle` has no equivalent.

A keyed card gets the right answer, which shows the count is simply blind to the keyless
case: `me bundle` on the same policy encoded **with** the three xpubs says *"backup needs 7
plates (6 public + ms1 on device)"*, which is correct.

**Worse than saying nothing? YES.** "backup needs 2 plates" is the operator's checklist for
an irreversible, ~21-minute-per-plate physical process. A tool that printed only the plates
it was handed, with no total, would leave them to work it out; this one answers the question
they asked, wrongly, and the error is silent until a restore years later.

---

### F-603 — `md compose --json` numbers paths 1-based in `experimental[]` and 0-based in `slots[].path`, so one object contradicts itself

**Severity: Minor.** Classification: **documentation / output consistency**.

From the six-path wallet's `md compose --json`:

```
experimental: ['path 3 has no key (bearer access to whoever holds the preimage)',
               'path 4 has no key (bearer access to whoever holds the preimage)',
               'path 6 has no key (bearer access to whoever holds the preimage)']
slots[].path values: [0, 0, 0, 1, 1, 1, 4, 4]
slots on path 4: [6, 7]
```

`experimental` counts from 1 (paths 3/4/6 are the three key-less ones). `slots[].path`
counts from 0. Read in the JSON's own numbering, the object says "path 4 has no key" while
listing two key slots, @6 and @7, on path 4.

**Worse than saying nothing? NO** — a human reading stderr gets consistent 1-based numbers
and the slot map is correct on its own terms. Recorded as a Minor because `--json` exists
to be consumed by `mnemonic-gui`, and a consumer joining the two arrays gets a false
statement with no parse error to warn it.

---

### F-604 — `mk encode --from-md1` cannot consume a chunked md1 set, and reports it as a wire-format version mismatch

**Severity: Minor** (fails closed; a working alternative exists). Classification:
**refusal with a false stated reason and no named remedy**.

Every chunk of the pathological wallet's 5-chunk policy card is rejected, singly or all
five at once:

```
$ mk encode --xpub xpub6DkFAXWQ2dHxq… --origin-fingerprint 73c5da0a \
    --origin-path "m/48'/0'/0'/2'" --from-md1 md1f8z2czq9zztvyy…m4ek5
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
```

The version is not 9; that is a chunked-envelope byte read as a version field. The same
command with a short unchunked md1 works:

```
$ mk encode … --from-md1 md1yzpqqxppcgsc9kdmw6d5dp08f
mk1qpqv4jpqqsqmq26yqdeutks2q5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5vqqsdp25fj9xlme
mk1qpqv4jpp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl9ypjktgjsuw7s6l99kxgm
```

Any policy large enough to chunk — i.e. every real multisig — therefore cannot have its key
cards stubbed through `--from-md1`. The route that works is `--policy-id-stub 2c2e1a5d`
(the value `md inspect` prints as `wallet-policy-id-fingerprint: 0x2c2e1a5d`), which the
error message does not mention.

**Worse than saying nothing? NO** — it refuses rather than minting a wrongly-stubbed card,
which is the safe direction. Recorded because the stated reason is false and sends the
operator looking at version skew between `md` and `mk` rather than at chunking.

---

### F-605 — `me sysw pack` prints the full success card, including the container digest, before the write that fails

**Severity: Minor.** Classification: **ordering / claim about an artifact that does not exist**.

```
$ me sysw pack --in pr.txt --no-passphrase --no-now --out /dev/null
sealing:  NOT SEALED — …
strength: no passphrase — BELOW the threshold
digest:   0a47 f7c8 7bad 255a dad9 7f88 ea1f 1402
          re-print it with: me sysw show /dev/null
me: /dev/null: Operation not permitted (os error 1)
exit=2
```

The report is emitted, then the write fails. The `digest:` line is the value an operator
transcribes to compare against the device's Payload Digest screen, and here it describes a
container that was never written; the follow-up instruction (`me sysw show /dev/null`)
cannot work either.

**Worse than saying nothing? NO** — the failure is printed immediately after and the exit
code is non-zero, so a careful operator and any script catch it. Recorded because the
report-then-write order is the wrong way round for the one line that is meant to be copied
by hand.

---

### F-606 — `md verify`'s MISMATCH message cites two identical numbers as its evidence

**Severity: Minor.** Classification: **documentation / message quality**. The verdict
itself is correct — see §3.

```
$ md verify --in nasty2.md1 --template "<template with one digest changed>" --experimental
md: MISMATCH: expected 1447-bit payload, got 1447-bit (181 vs 181 bytes)
exit=1
```

Correct refusal, but the stated evidence is "181 vs 181". The operator standing over a plate
and a descriptor learns only that something differs, not what.

**Worse than saying nothing? NO** — the exit code and the word MISMATCH carry the safety-
relevant answer. Recorded because this fires at exactly the moment the operator needs to
know *which* field drifted.

---

### F-607 — `md compose` accepts two byte-identical hash paths and emits a doubled branch

**Severity: Nit.** Classification: **warning**.

```
$ md compose --wrapper wsh --experimental --path '2of3' \
    --path 'keyless,sha256=4438…b34c' --path 'keyless,sha256=4438…b34c'
exit=0
wsh(or_d(multi(2,…),or_i(sha256(4438…b34c),sha256(4438…b34c))))
```

Two identical branches; the second is unreachable and pays script weight forever. A
duplicated `--path` is almost always a copy-paste where one digest was meant to change.

**Worse than saying nothing? NO** — the wallet is exactly what was asked for and is not
unsafe. Recorded as a Nit.

---

### F-608 — `me bundle` renumbers plates in an order unrelated to the input

**Severity: Nit.** Classification: **not our concern / documentation**.

`allpublic.txt` lists the nine mk1 cards in account order 0..8. The manifest lists them
7, 0, 3, 5, 4, 1, 6, 2, 8:

```
  plate 6/32  mk1 [73c5da0a/48'/0'/7'/2'] chunk 1/3  → push via NFC & engrave
  plate 9/32  mk1 [73c5da0a/48'/0'/0'/2'] chunk 1/2  → push via NFC & engrave
  plate 11/32 mk1 [73c5da0a/48'/0'/3'/2'] chunk 1/3  → push via NFC & engrave
```

The set is complete and each row is labelled with its origin, so nothing is lost — but an
operator cross-checking the checklist against their own file is comparing two different
orders.

**Worse than saying nothing? NO.**

---

### F-609 — `me sysw show` prints records out of numerical order when classes mix

**Severity: Nit.**

```
$ me sysw show one.bin          # key: record then an md1 record
public record 1: md1/mk1 — unconfirmed — engraveable, but the device REPLACES the legend
public record 0: cosigner key (key:) — [73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq…
```

Record 1 before record 0. Harmless on its own; it is the same renderer as F-598 and is
evidence that the record loop is a per-class pass rather than a single ordered walk.

**Worse than saying nothing? NO.**

---

### F-610 — the bearer-access warning is minted-side only; `md decode` of the same card is silent

**Severity: Minor.** Classification: **warning**.

`md encode --experimental` shouts "THE PLATE IS BEARER ACCESS". `md decode` of the resulting
card — the verb the *restorer* runs, possibly a different person years later — prints the
template, the origins, and `note: stdout is a keyless descriptor template (no keys)`, with
no mention that one spend path needs no signature at all. Exit 0, no flag required.

The source comment at `crates/md-cli/src/cmd/encode.rs:76` names the gap itself: *"the card
itself carries no record that a flag was used to create it — the operator's memory and this
line are the only trace."*

**Worse than saying nothing? NO, marginally** — the key-less path is visible in the
template that `md decode` prints, for a reader who can read miniscript. Recorded because
the decode side is where the audience least able to read it is standing, and because
F-599 has made the encode-side warning unreliable anyway.

---

## 3. What held up under stress — do not spend effort here

Every one of these was probed with a negative control and behaved:

- **Digest width and case, everywhere.** 63 hex, 65 hex, mixed-case hex, a 40-hex digest
  under a 64-hex kind and vice versa are refused by `md compose` *and* `me sysw pack`, each
  naming the clause: `sha256 needs exactly 64 hex characters, got 63`; `sha256 is 64
  LOWERCASE hex; case is rejected, never folded (SPEC_hashlock_kinds §6 …)`.
- **Kind tags never case-fold.** `SHA256`, `Sha256`, `Ripemd160` and unknown `sha512` are
  refused by `md compose` (`unknown option \`SHA256\` -- the four hash kinds are … LOWERCASE;
  case is rejected, never folded. This md supports all four`), by `me sysw pack`, and by
  `ms hashlock --kind` (clap-level, exit 64).
- **`hash:sha256:` really is normalised.** Packing the bare and the tagged form produces
  byte-identical containers (`cmp` clean, same identity `ba79153b…`, same digest `0a47
  f7c8 …`).
- **The tag-stripping trap behaves exactly as documented.** An untagged 64-hex hash256
  digest is silently read as sha256 — and `me sysw pack` says so unprompted, in the
  stderr note and in the refusal help, in capitals.
- **The 12 zero pad bytes are unobservable.** The same 32-byte preimage written out under
  `--kind sha256`, `--kind ripemd160` and `--kind hash160` produces three byte-identical
  ms1 plates (`md5sum` all `144a7c4e…`). Re-reading a plate with no `--kind` lists all four
  digests at their own widths.
- **`md verify` is falsifiable, including on kind.** Correct template → `OK`, exit 0.
  A changed digest → MISMATCH, exit 1. **The same digest with sha256 swapped for hash256 →
  MISMATCH, exit 1** — the kind is genuinely in the wire format, not cosmetic.
- **`me sysw pack --expect` is a gate that can fail.** `--expect descriptor` on 4 of 5 md1
  chunks: *"records of that kind ARE present, but the set does not reassemble. Unconfirmed
  at record 0, 1, 2, 3 … Nothing was written."* `--expect descriptor,cosigner` on md1 alone:
  *"NO record of that kind is in the stream."* On md1 + a truncated mk1 set: refused at
  record 28, 29. The complete set passes at exit 0.
- **`me bundle` refuses an incomplete chunk set** (`md1 set 0x38958 is incomplete/
  inconsistent: chunk set incomplete: got 4 chunks, expected 5`, exit 4) — the *set*
  integrity claim is honest; only the *total* claim (F-602) is not.
- **The F-539 digest-looking-phrase guard.** A 64-hex, 40-hex or 40-hex-uppercase phrase
  stops with an explanation and the right next step; a 39-hex phrase proceeds. The boundary
  is where it says it is.
- **argv secret guards.** `ms hashlock --hex <64 hex>` on argv is refused before the parser
  runs, with per-shell history-purge instructions.
- **`me sysw show` renders the kind beside every abbreviated digest** —
  `public record 9: sha256 hashlock (hash:) — 44381646..f608b34c` followed by the full
  `sha256:4438…b34c` on the next line — so the `first8..last8` window is never the identity
  on this surface. Two records carrying the same digest under two different kinds are
  distinguishable here.

---

## 4. What I could not reach, and why

**The device composer, past the front door.** The Playwright MCP shares a single browser
with another agent working on port 8391. My tab was navigated to their URL twice mid-walk
(`Execution context was destroyed`), and opening a second tab did not survive either. I got
as far as: boot → skip offer → Load Payload → Payload Digest (`55adb800…`, matching the
host) → **Payload Warnings ("A SECRET is stored unencrypted in flash.")** → Keep → Wallet
Policy → `A seed is loaded. It can fill any number of slots.` → `Which script?` (4 rows) →
`Start from?` (7 rows: Build my own paths, plain-multisig, simple-timelocked-inheritance,
kofn-recovery, tiered-recovery, hashlock-gated, decaying-multisig) → `Spend paths / slots:
0` → `What can spend on this path? / Keys / A hash, no keys`. Row taps past that point
started landing in other programs as the session drifted, and I stopped rather than report
guesses. Consequently **unverified**:

- whether the device's `Spend paths` list and the Done census show the hash **kind** beside
  the `first8..last8` token, or only the token — which decides whether two paths committing
  to the same preimage under two kinds are distinguishable on the machine;
- whether the device's own `md.Compose` reproduces **F-600** — i.e. whether an operator can
  build two key-less hash paths on the machine and only discover at Done that the policy
  cannot be lowered. This is the highest-value unreached item and should be walked next.

**A custom payload on the device.** `window.shSysw` accepts only the four blobs baked into
`emu.wasm` (`records`, `cards`, `composer`, `none`); loading the four-kind payload I packed
would mean replacing a tracked `.bin` and rebuilding, which the brief forbids. The shipped
`composer` blob carries a single sha256 record (`abababab…abab`), so the four-kind door
census could not be exercised.

**A real `first8..last8` display collision.** Matching both ends of the abbreviation is a
64-bit grind; matching only the first 8 hex (2^16 by birthday) would not collide on any
surface I measured, since every one of them prints the last 8 too and names the kind.

**Physical engraving / flashing.** Out of scope by the brief; no device present.
