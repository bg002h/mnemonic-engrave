# Journey walk — "build our reasonably complex wallet with a hashlock, and engrave it"

**Date:** 2026-09-16 · **Walker:** journey agent (operator role) · **Findings:** F-578 … F-597
**Tree:** mnemonic-engrave `97b7ff83`, seedhammer `ec11fab` · **CLIs:** md 0.15.0, ms 0.19.0, me 0.10.0, mk 0.13.0
**Scope:** one journey, end to end, host → payload → emulator. Not a code audit. No tracked file modified
(`git status --porcelain` empty in both repos at close).

**Counts:** 2 Critical · 7 Important · 5 Minor · 6 Nit.

---

## 1. The journey, as it actually went

### Step 1 — what do I have in hand?

Two policy files for the same fixture, differing only in the account index:

```
$ diff design/journeys/inputs-rcw/policy-tr.txt design/fixtures/reasonably-complex-wallet/tr.policy
< …@0/270028'/0'/8'/0'/<0;1>/*…
> …@0/270028'/0'/0'/0'/<0;1>/*…
```

I picked `inputs-rcw/policy-tr.txt` (account 8) because the brief fixes the accounts. The fixture README's
id/address table is computed at account 0, so none of its numbers apply to the wallet I am building. → **F-593**.

### Step 2 — turn seven seed phrases into keys

I have seven files of 24 BIP-39 words. `ms derive` only takes a phrase on argv, so I tried the private channels first:

```
$ cat seeds/key-0.seed | ms derive - --template bg002h-tr --account 8
error: string length 167 not in v0.1 set [50, 56, 62, 69, 75]

$ ms derive --in seeds/key-0.seed --template bg002h-tr --account 8
error: string length 167 not in v0.1 set [50, 56, 62, 69, 75]
```

Both refuse — `derive`'s `--in` means an **ms1**, not a phrase. So I used argv, and got a long, well-written refusal
whose prescribed remedy is the two spellings I had just measured as broken:

```
$ ms derive --phrase "$(cat seeds/key-0.seed)" --template bg002h-tr --account 8
ms: argument 3 on ARGV … is a BIP-39 mnemonic, 190 characters long.
      Use a private channel instead:
          ms derive --in FILE      # read it from a file
          ms derive -              # or pipe it on stdin
```

The *correct* recipe is four lines further down, framed as being about phrase-plus-passphrase. → **F-581**.
It works:

```
$ ms encode --in seeds/key-0.seed --out /tmp/rcw-journey/key-0.ms1
$ ms derive --in /tmp/rcw-journey/key-0.ms1 --template bg002h-tr --account 8
master_fingerprint:  39ec1b6e
account_path:        m/270028'/0'/8'/0'
account_xpub:        xpub6Dn2FdZCkR6b6nCFWKMZraCwEcC7aedkRrVHf8DK5gSBrny6YJkiUZp9bL8j2a4osHSuhDsk1FBnSS81C82qvZ4uiDofJGQKmNmmxqfUxCr
```

### Step 3 — the loop over seven seeds silently produced six

I looped, suppressed stderr, and ended up with six xpubs and one `NoneType`. Key 5 had no card. Re-run showing stderr:

```
$ ms encode --in seeds/key-5.seed --out /tmp/rcw-journey/key-5.ms1
thread 'main' (3757539) panicked at bip39-2.2.2/src/lib.rs:578:65:
called `Result::unwrap()` on an `Err` value: AmbiguousLanguages(AmbiguousLanguages([true, false, false, false, true, false, false, false, false, false]))
EXIT=101
```

**`ms encode` panics on a valid 24-word English mnemonic** — the repo's own tier-3 recovery seed. → **F-579**.
This is where the journey nearly ended: there is no route from those words to an ms1 plate.

### Step 4 — encode the policy

```
$ md encode --in policy-tr.txt
chunk-set-id: 0x3f541
… 4 chunks …
warning: this keyless template's slots cannot be told apart — @0, @1, @2, @3, @4, @5, @6 all declare
m/270028'/0'/8'/0'. … a device that will not guess must refuse the whole set. Pass one --fingerprint
@N=HEX per slot; it costs about one extra md1 chunk and changes no path, no key and no policy.
```

Excellent warning. I did what it says, and got five chunks and no warning:

```
$ md encode --in policy-tr.txt --fingerprint @0=39ec1b6e … --fingerprint @6=26bd1e33 --group-size 0
chunk-set-id: 0x8ce75
md1f3nn4zqppj040veppfzzqqvpc4xdw60mct5sk6mftznwaetcuxc4eqqm2tafpafqluajpvqlpuqppt
md1f3nn4zqfwyfaxrskrj2dnlzgzzq59fntsqqpqqqfnth5e2hm0tdyl72yv87pkpe48xq3sgms9wn9a3
md1f3nn4zqkau8254fvsad4zpu5ty0dnv820px6zqgts2nxkcqz85pq24xddsq9guvpxcu6s3gtrhlez9
md1f3nn4zqmt4u4qzzae20epdnmhnlts9q5nx5cm7f7xfcfsp6zpjr09lc8r440w2cx8ahd27kcs26l07
md1f3nn4zp9g88kpkm36q4uah6tltcavmjtzv3v5ppkqgl4emuzynxy673uvcqghqgu75ss02c3
```

`md inspect` on the set prints the template, `n: 7`, the ids and all seven origins — and says **nothing about the
three sha256 hashlocks**.

### Step 5 — the checklist

```
$ me bundle --in plate-tr.md1
…
me: backup needs 6 plates (5 public + ms1 on device):
  plate 1/6  md1 policy  → push via NFC & engrave
  …
  plate 6/6  ms1 secret  → TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool
```

Six plates, for a wallet with **seven** seeds and **three** hashlock preimages. No hashlock note at all.
The ms1 line is a constant — a 2-of-3 gets the same "2 plates (1 public + ms1 on device)". → **F-580**.

The missing hashlock note is not a missing feature: it exists, and I found the exact condition under which it
does and does not fire. **The same wallet, wsh-wrapped, warns.** → **F-578**:

```
$ me bundle --in plate-wsh.md1
me: backup needs 6 plates (5 public + ms1 on device):
me: NOTE — this policy has a hashlock path (sha256). Its preimage is NOT one of these plates and is not
counted above: keep the phrase or the preimage plate apart, or the hashed path cannot be spent.
```

### Step 6 — the hashlocks themselves (this part worked)

```
$ ms hashlock --kind sha256 --method sha256 --hashlock-phrase-stdin < preimages/preimage-0.txt
hash:a7ef0ba42dada5629bbb95e386c572006d4bea43d483e5c44f4c3858725367f1
preimage (ms1):  ms10h ashsq 0k7pv 52sp0 7egy6 wlzgl q46hs fyn3u 64pqd a986f 2zm74 dy20y pxu74 sahyl qpzw8
preimage (hex):  ede0b28a805feca09a77c48f82babc1249c79aa840de94fa4a85bf55a453c813
```

All three digests match the policy literals byte for byte. The default method (`hardened`) gives a different
digest (`6de851a0…`), and the card says so in the `method:` line — correct and well handled.

Feeding those three preimage plates back to `me bundle` alongside the md1 set refuses the **whole** set:

```
me: refusing to process ms1 over this tool: ms1 is secret seed entropy — enter it by hand on the device
(New > Input Seed > CODEX32), never via NFC/this tool
EXIT=3
```

It is not seed entropy — `ms decode` calls it `kind: preimage (hashlock, 32 bytes / 64 hex characters)`. → **F-586**.

### Step 7 — previews

The installed sidecar is version-locked out of the rebuilt `me`:

```
me: me-preview version mismatch: sidecar is "0.7.0", expected "0.10.0"; refusing to render
```

`scripts/build-preview.sh` fixes it in 0.6s, but only into `target/release`; the sidecar `me` actually discovers is
`~/.cargo/bin/me-preview`, which `cargo install` does not update. → **F-588**. With `ME_PREVIEW_BIN` set, five PNGs
rendered. I read `plate-1.png`: the four text lines concatenate to exactly the chunk string, so nothing is clipped;
lines 3–4 sit flush at x=0 while 1–2 are inset. → **F-596**.

### Step 8 — the device payload

```
$ me sysw pack --in records.txt --no-passphrase --out payload.bin     # 7 key: + 3 hash:
sealing:  NOT SEALED — no record in this payload is secret material, so there is nothing to encrypt.
```

Adding the three `phrase:` records without the flag refuses precisely, with the record index and the fix:

```
me: record 10 (records count from 0) is a hashlock PHRASE record (phrase:), not a seed record; this payload
did not ask for one. … Re-run with --pack-preimage if that is what you intend.
```

With `--pack-preimage` it packs, with three correct and prominent bearer warnings. `me sysw show` reads back all
13 records correctly. Record indices shift by one between the refusal and `show`, because `now:` is inserted at 10. → **F-590**.

### Step 9 — the emulator

Built first try (`go version go1.26.7`, `built emu.wasm (11089274 bytes)`), served on 8391, driven with Playwright.

Then the wall: **`shSysw` accepts only `"records" | "cards" | "composer" | "none"`** — three blobs compiled into the
wasm. There is no way to load `payload2.bin`. I could not rehearse my own wallet on the device at all. → **F-585**.

Driving the built-in composer payload anyway, the digest screen matched the host exactly
(`dbe9e774e9a492310b62626c2b41cf4b`). The next screen did not:

```
Payload Warnings — A SECRET is stored unencrypted in flash.
```

`me sysw show` on those same bytes reports `ct_len: 0` and four records: two `key:`, one `hash:`, one `now:`.
Host and device disagree about whether the payload holds a secret. → **F-584**.

### Step 10 — pushing the plates by NFC

`me bundle` said "push via NFC & engrave", so I did, from a clean boot:

```
chose Supply policy (md1) | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 1/5 | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 2/5 | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 3/5 | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 4/5 | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 5/5 | EngraveBundle md1descriptors:1 mk1keys:0 Scanacard,orDone.Cardadded.
```

Four taps, nothing on screen. → **F-583**. (The *Inspect descriptor* screen, measured separately, does say
`Captured 2 of 5. Scan the next chunk.` — so one screen in this program tells you and the other does not.)

Then Done — and this is where the journey ends:

```
CAPTURED: EngraveBundle md1descriptors:1 mk1keys:0 Scanacard,orDone.Cardadded.
TARGETS HERE: []
AFTER ONE CONFIRM: The supplied descriptor has no public keys to match. EngraveMultisig
```

**The device refuses the exact plate set `me bundle` prescribed.** The keyed 16-chunk form is accepted
(`after 16/16: md1descriptors:1 … Cardadded.` → `Where from? TYPE IT / SCAN — Input Seed`), so the route exists —
it is just not the one the checklist describes.

The other route is keyless md1 + mk1 key cards (the `mk1 keys: 0` counter). It cannot be walked here:

```
$ mk encode --xpub … --origin-path "m/270028'/0'/8'/0'" --origin-fingerprint 39ec1b6e --from-md1 <RCW chunk>
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
```

Every RCW form (keyless, keyless+fp, keyed) is rejected; a simple 2-of-2 md1 is accepted. → **F-587**.

### Step 11 — verifying the steel before cutting it

```
$ md verify --template "$(cat policy-tr.txt)" --in plate-tr.md1
md: MISMATCH: expected 1140-bit payload, got 1402-bit (143 vs 176 bytes)
EXIT=1
```

MISMATCH on the plates I had just made from that file. The cause is the missing `--fingerprint` flags, which the
message never names. The tempting fix is worse than the problem → **F-582**:

```
$ md encode --in policy-tr.txt --group-size 0 | sort -u > nofp.md1     # 4 chunks
$ md verify --template "$(cat policy-tr.txt)" --in nofp.md1
OK
EXIT=0
$ me bundle --in nofp.md1
me: backup needs 5 plates (4 public + ms1 on device):
```

Both gates pass on the set `md encode` warned "a device that will not guess must refuse".

Restoration itself is sound: `md decode` on the five plates returns the template and all seven origins, and
`md address` gives `bc1pjwwr9p50xss2uc6tmlslsfdmgy6dssk0zwjrx23znttvn4l5r30s99rh3d`.

---

## 2. Findings

### F-578 — `me bundle`'s hashlock completeness note is dead for every taproot wallet

**Critical.** Classification: **warning** (a warning that exists and never fires).

F-557 is recorded in `design/FOLLOWUPS.md` as *"`me bundle`'s plate count claimed completeness while omitting the
preimage plate — fixed in me `77c4026e`"*. The note is in the binary and never printed for this wallet.

Isolated to the wrapper, same hashlock, same digest, both unchunked with an origin:

```
$ md encode "wsh(and_v(v:sha256(a7ef0ba4…67f1),pk(@0/<0;1>/*)))" --path bip84 --group-size 0 > d.md1
$ me bundle --in d.md1
me: backup needs 2 plates (1 public + ms1 on device):
me: NOTE — this policy has a hashlock path (sha256). Its preimage is NOT one of these plates …

$ md encode "tr(50929b74…3ac0,and_v(v:sha256(a7ef0ba4…67f1),pk(@0/<0;1>/*)))" --path bip86 --group-size 0 > e.md1
$ me bundle --in e.md1
me: backup needs 2 plates (1 public + ms1 on device):
  plate 1/2  md1 policy  → push via NFC & engrave
  plate 2/2  ms1 secret  → TYPE ON DEVICE …
```

And the manifest key is absent for all four kinds in tr form:

```
sha256     manifest has hashlock_kinds? '<KEY ABSENT>'
hash256    manifest has hashlock_kinds? '<KEY ABSENT>'
ripemd160  manifest has hashlock_kinds? '<KEY ABSENT>'
hash160    manifest has hashlock_kinds? '<KEY ABSENT>'
```

Mechanism, visible without inference: `descriptor_hash_kinds` (`crates/me-cli/src/bundle.rs:203-227`) recurses
through `Body::Children` and `Body::Variable` only, and a taproot descriptor's taptree lives in
`Body::Tr { tree: Option<Box<Node>> }` (`md-codec/src/tree.rs`, variants `Children, Variable, MultiKeys, Tr,
KeyArg, Hash256Body, Hash160Body, Timelock, Empty`). The field is
`#[serde(skip_serializing_if = "Vec::is_empty")]`, so an empty result is indistinguishable from "no hashlock".

Why it survived: the mutation test at `crates/me-cli/src/manifest.rs:332-341` constructs
`hashlock_kinds: vec!["ripemd160"]` **by hand** and asserts on `checklist()`. It tests the renderer. The detector
has no end-to-end test, so the gate cannot fail.

Second, narrower hole in the same function: the unchunked path is `if let Ok(d) = decode_md1_string(s)`, so a
decode failure drops the note with no message. The chunked path hard-errors on the identical input:

```
$ md encode "wsh(and_v(v:sha256(H),pk(@0/<0;1>/*)))" --group-size 0 > a.md1   # no origin
$ me bundle --in a.md1
me: backup needs 2 plates (1 public + ms1 on device):        ← silent, note dropped
$ md encode "wsh(and_v(v:sha256(H),pk(@0/<0;1>/*)))" --force-chunked … | me bundle --in -
me: md1 set 0x7c60d is incomplete/inconsistent: non-canonical wrapper requires explicit origin for @0
```

**Worse than saying nothing? YES.** The note's entire purpose is to stop an operator cutting a complete-looking
backup of a wallet three of whose four tiers are unspendable without the preimages. Taproot is the fixture's
primary form, and the tool tells the tr operator nothing while telling the wsh operator the truth about the
same four tiers.

---

### F-579 — `ms encode` and `ms split` panic on a valid English mnemonic; the fixture's own tier-3 seed is one

**Important.** Classification: **refusal** (that should be a clean one, and is a crash instead).

```
$ cat seeds/key-5.seed
abandon abandon … abandon surface
$ ms encode --in seeds/key-5.seed --out /tmp/rcw-journey/key-5.ms1
thread 'main' panicked at bip39-2.2.2/src/lib.rs:578:65:
called `Result::unwrap()` on an `Err` value: AmbiguousLanguages(AmbiguousLanguages([true,false,false,false,true,false,false,false,false,false]))
EXIT=101
```

All three input channels panic, and so does `ms split`:

| invocation | result |
| --- | --- |
| `ms encode --in seeds/key-5.seed` | panic, exit 101 |
| `ms encode --phrase - < repro.txt` | panic, exit 101 |
| `ms encode --in repro.txt --language english` | panic, exit 101 |
| `ms encode --in repro.txt --language french` | `error: BIP-39 checksum failure` (clean, exit 1) |
| `ms split --in repro.txt -k 2 -n 3` | panic, exit 101 |
| `ms derive --phrase … --allow-argv-secret` | **succeeds**, `master_fingerprint: cef8224c` |

Indices 0 and 4 of the bip39 crate's language array are English and French. The English∩French BIP-39
intersection is **100 words** (measured from the crate's own wordlists), and the panic fires when *every* word of
the mnemonic is in that set — which `abandon ×23 + surface` is.

The divergence between `encode` (panics) and `derive` (works) matches the call sites: `derive.rs:424` uses
`parse_in` + seed derivation, while `encode.rs:144-147` uses `parse_in` then `mnemonic.to_entropy()`, and
`to_entropy_array()` does `Mnemonic::language_of_iter(self.words()).unwrap()` — it re-detects the language and
discards the `lang` that `parse_in` was already given, on the crate's own comment that
*"this method can only be called on values that were already previously validated"*.

Not data loss: an existing `--out` target is left intact (`PRECIOUS-EXISTING-BACKUP-DO-NOT-LOSE`, 37 B, unchanged
after the panic). And it is why the fixture generator never caught it — `derive-rcw-keys.sh:71` builds cards with
`ms encode --hex -`, not from phrases, so the operator's route is the only unexercised one.

**Worse than saying nothing? YES.** The operator has seed words on steel and the tool cannot make a plate from
them — and a Rust panic trace tells them nothing about what to do. Blast radius for random mnemonics is
negligible, but it covers every BIP-39 test vector of the `abandon…` family and the repo's own fixture.

---

### F-580 — `me bundle`'s "backup needs N plates" is not a set the device will accept

**Critical.** Classification: **warning / default**.

The checklist is a completeness claim and it is wrong in two directions at once.

*Under-counts seeds.* The `ms1` line is a fixed constant of one, regardless of `n`:

```
7-cosigner RCW:  me: backup needs 6 plates (5 public + ms1 on device):  … plate 6/6 ms1 secret
2-of-3 multisig: me: backup needs 2 plates (1 public + ms1 on device):  … plate 2/2 ms1 secret
wpkh 1 key:      me: backup needs 2 plates (1 public + ms1 on device):  … plate 2/2 ms1 secret
```

*And the public half is not engravable as given.* Handing the device exactly those five md1 plates over NFC:

```
CAPTURED: EngraveBundle md1descriptors:1 mk1keys:0 Scanacard,orDone.Cardadded.
AFTER ONE CONFIRM: The supplied descriptor has no public keys to match. EngraveMultisig
```

The keyed 16-chunk form is accepted and proceeds to `Where from? TYPE IT / SCAN — Input Seed`. So the true plate
set is either 16 md1 + seeds, or 5 md1 + N mk1 key cards (which F-587 blocks) — never the 6 the tool names.

This shape has been reported twice before in `design/agent-reports/` (`REVIEW_pathological_lens2_operator_effort.md`
counted 36 real plates against the tool's 34; `REVIEW_pathological_lens3_comprehension.md` the same) and a grep of
`design/FOLLOWUPS.md` for it returns nothing — it was never filed.

**Worse than saying nothing? YES.** "backup needs 6 plates" reads as the requirement, is followed to completion,
and the operator discovers the gap at the device with the steel already cut. With no checklist they would at
least count their own seeds.

---

### F-581 — the argv-secret guard prescribes a remedy that does not work for the verb and secret it just named

**Important.** Classification: **refusal** (correct) with **documentation** that is false. *(If a reader classifies
this under the secret-handling ruling it is non-gating; the defect is the wrong instruction, not a leak.)*

The guard identifies the verb and the secret kind precisely, then prints one generic remedy block:

```
$ ms derive --phrase "$(cat seeds/key-0.seed)" …
ms: argument 3 … is a BIP-39 mnemonic, 190 characters long.
      Use a private channel instead:
          ms derive --in FILE      # read it from a file     ← --in on derive means an ms1
          ms derive -              # or pipe it on stdin     ← stdin on derive means an ms1
```
```
$ ms derive --in seeds/key-0.seed …
error: string length 167 not in v0.1 set [50, 56, 62, 69, 75]
```

Same block, other direction:

```
$ ms encode --hex 0000…0006 …
      Use a private channel instead:
          ms encode --in FILE      ← --in on ENCODE means a phrase, "never hex" per its own --help
$ ms encode --in ent5.hex
error: BIP-39 word count 1 invalid (must be 12, 15, 18, 21, or 24)
```

The working spellings exist (`ms encode --in seed.txt --out card.ms1` then `ms derive --in card.ms1`; and
`--hex - < FILE`) — the first is buried below, framed as being about phrase-plus-passphrase, and the second is not
in the block at all. Credit where due: the *second-order* error above does print the right hint
(`ms encode --hex - < ent5.hex`).

**Worse than saying nothing? YES.** A refusal that hands the operator two commands that both error is worse than
a refusal that just says "not on argv" — they will reasonably conclude the tool is broken rather than that the
advice is.

---

### F-582 — `md verify` MISMATCHes a correct plate set, and the obvious fix produces the unseatable one

**Important.** Classification: **warning**.

```
$ md verify --template "$(cat policy-tr.txt)" --in plate-tr.md1
md: MISMATCH: expected 1140-bit payload, got 1402-bit (143 vs 176 bytes)
EXIT=1

$ md verify --template "$(cat policy-tr.txt)" --fingerprint @0=39ec1b6e … --in plate-tr.md1
OK
EXIT=0
```

The flag exists; the message never names it. The failure mode is the reaction, not the message: the operator who
drops `--fingerprint` to make verification pass gets a set that two gates bless and a third had already condemned.

```
$ md encode --in policy-tr.txt --group-size 0 | sort -u > nofp.md1
chunks: 4
$ md verify --template "$(cat policy-tr.txt)" --in nofp.md1     → OK, exit 0
$ me bundle --in nofp.md1                                       → "backup needs 5 plates", no complaint
```

…against `md encode`'s own warning on that very set: *"a card here matches several slots and a device that will
not guess must refuse the whole set."*

**Worse than saying nothing? YES.** A bare "differs" would leave the operator to investigate. A bit-count MISMATCH
that clears the moment they remove the flag actively teaches them to remove it.

---

### F-583 — the Engrave Bundle capture screen gives no per-chunk progress; four of five NFC taps change nothing

**Important.** Classification: **default** (add progress).

From a clean boot, `shSysw("none")`, Engrave Multisig → Supply policy (md1):

```
chose Supply policy (md1) | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 1/5 | EngraveBundle md1descriptors:0 mk1keys:0 Scanacard,orDone.
after chunk 2/5 | (identical)
after chunk 3/5 | (identical)
after chunk 4/5 | (identical)
after chunk 5/5 | EngraveBundle md1descriptors:1 mk1keys:0 Scanacard,orDone.Cardadded.
```

Sampling at 30 ms found no transient frame either. The state *is* tracked — re-presenting a captured chunk says
`Already captured that card.` — and the **Inspect descriptor** screen in the same program shows
`Captured 2 of 5. Scan the next chunk.` This wallet is 5 chunks keyless and 16 keyed, so the keyed route is
fifteen silent taps.

**Worse than saying nothing? YES.** Silence is indistinguishable from a dead reader or a bad tag, and the
operator's recovery (re-tap everything) is also silent. One screen in this program already has the right copy.

---

### F-584 — the device says a secret is in flash for a payload the host says holds none

**Important.** Classification: **warning** (one that fires when it should not).

Built-in composer payload, digest confirmed identical on both sides (`dbe9e774e9a492310b62626c2b41cf4b`):

```
$ me sysw show cmd/emu/sysw_composer_payload.bin
sealed: false   ct_len: 0
public record 0: cosigner key (key:) …
public record 1: cosigner key (key:) …
public record 2: sha256 hashlock (hash:) — abababab..abababab
public record 3: pack time (now:) …
```

Device, after confirming that digest:

```
Payload Warnings — A SECRET is stored unencrypted in flash.
```

`me sysw pack` on my own same-shaped records (7 `key:` + 3 `hash:`) states the opposite explicitly:
*"NOT SEALED — no record in this payload is secret material, so there is nothing to encrypt."*
The flag is `flagSecretInPlaintext` from `secret := c.IsSecret() || unconfirmed` (`gui/sysw_admit.go:154`), and the
plain (not "could not confirm") wording means a record's class is being read as secret. I did not determine which.

**Worse than saying nothing? YES.** This is the one warning that must land on my `payload2.bin`, which really does
carry three bearer preimages. A warning that fires on a key-and-digest payload teaches the operator to tap past it.

---

### F-585 — the emulator cannot load an operator-built payload

**Important.** Classification: **default**.

```go
case "records", "cards", "composer", "none":   // cmd/emu/walk_js.go:119
```

Three blobs compiled into the wasm, selected by name. `me sysw pack` produces a file; nothing consumes one. My
`payload2.bin` — the actual wallet, with its three `phrase:` records — could not be put on the emulated device at
all, so the preimage-plate leg of this journey is unreachable without flashing hardware.

**Worse than saying nothing? YES** for a device whose only other surface is an irreversible flash. The emulator is
the rehearsal surface, and it can only rehearse three wallets nobody owns.

---

### F-586 — `me bundle` refuses a preimage plate as "secret seed entropy" and points at CODEX32 seed entry

**Important.** Classification: **refusal** (correct) with wrong guidance.

```
$ me bundle --in full.txt        # 5 md1 chunks + 3 preimage ms1 plates
me: refusing to process ms1 over this tool: ms1 is secret seed entropy — enter it by hand on the device
(New > Input Seed > CODEX32), never via NFC/this tool
EXIT=3
```

It is not seed entropy, and `ms` knows exactly what it is:

```
$ ms inspect --in pre0.ms1
tag: hash     share_index: s     prefix_byte: 0x03
$ ms decode --in pre0.ms1
kind:      preimage (hashlock, 32 bytes / 64 hex characters)
```

`me seal` and `me sysw pack` refuse this record **by name** (F-489, v0.8.1). `me bundle` is the verb left giving the
generic message — and it is the verb an operator reaches for while holding all their plate strings.

**Worse than saying nothing? YES.** It sends the operator to a seed-entry screen with a 32-byte preimage, and it
discards the md1 manifest for the whole set in the same breath.

---

### F-587 — `mk 0.13.0` cannot bind a key card to this wallet's md1

**Important** (with a caveat: `mk` may simply be stale on this PATH — the brief says md/ms/me were rebuilt, and
does not mention mk). Classification: **refusal**.

```
RCW keyless+fp chunk1        exit=2 error: md1 input rejected: wire-format version mismatch: got 9, expected 4
RCW keyless NO fp chunk1     exit=2 error: md1 input rejected: wire-format version mismatch: got 9, expected 4
RCW keyed chunk1             exit=2 error: md1 input rejected: wire-format version mismatch: got 9, expected 4
simple 2of2 with path        exit=0 (accepted)
```

`mk encode` requires `--policy-id-stub` or `--from-md1`, so with `--from-md1` refused there is no way to produce the
mk1 key plates the device's `mk1 keys:` counter wants — which is the only thing that would make F-580's keyless
5-plate set engravable.

**Worse than saying nothing? WEAKLY YES.** The refusal is correct and fail-closed. But "wire-format version
mismatch: got 9, expected 4" does not tell an operator that their `mk` is older than their `md`, and the two
binaries sit side by side in `~/.cargo/bin`.

---

### F-588 — the `me-preview` sidecar goes stale on every `me` upgrade and `cargo install` does not fix it

**Minor.** Classification: **documentation only**.

```
$ me bundle --in plate-tr.md1 --preview prev --png
me: me-preview version mismatch: sidecar is "0.7.0", expected "0.10.0"; refusing to render
(install the matching me-preview)
$ ls -la ~/.cargo/bin/me-preview
-rwxr-xr-x 1 bcg bcg 1994936 Aug 31 06:58 me-preview
```

Exit code is 2, so it fails loudly — good. `scripts/build-preview.sh` rebuilds it in 0.6 s, but writes to
`target/release/`, while discovery is "alongside the `me` executable" (`$PATH` deliberately not searched). Nothing
in the refusal names the script or the copy step.

**Worse than saying nothing? NO** — the message is accurate and the exit code is right. Recorded because the
recovery step is undiscoverable from the error.

---

### F-589 — `ms encode --out FILE` warns about stdout when stdout is empty, and the secret goes to stderr

**Minor**, non-gating (secret-handling ruling 2026-08-27). Classification: **warning**.

```
$ ms encode --in seeds/key-1.seed --out key-1.ms1 2>err.txt >out.txt
stdout bytes: 0
stderr bytes: 340
$ cat err.txt
engraving card: ms10e ntrsq qqqq… qy8e2 ppje5 wvzcu
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' …)
$ ls -l key-1.ms1 err.txt
-rw------- 1 bcg bcg  76 key-1.ms1
-rw-r--r-- 1 bcg bcg 340 err.txt
```

`--out` is documented as existing precisely so the artifact lands 0600 "which a shell redirect cannot do". The
complete ms1 then goes to stderr unconditionally, where an ordinary redirect made it 0644 — and the warning points
at the stream that is empty.

**Worse than saying nothing? WEAKLY YES** for the misdirected warning (it trains the operator to distrust the
warning text). Reproduction recorded per the ruling; does not gate.

---

### F-590 — record indices disagree between `me sysw pack`'s refusal and `me sysw show`

**Nit.** Classification: **documentation only**.

```
me: record 10 (records count from 0) is a hashlock PHRASE record (phrase:) …
```
```
$ me sysw show payload2.bin
public record 10: pack time (now:) …
secret record 11: hashlock phrase (phrase:) — not shown
```

`now:` is appended at pack time and lands at 10, shifting the phrases to 11-13.

**Worse than saying nothing? NO.** The refusal's index is right for the input file, which is what the operator edits.

---

### F-591 — device copy: "Scan a card" on a device with no camera; "4 unrecognised record"

**Nit.** Classification: **documentation only**.

```
EngraveBundle md1descriptors:0 mk1keys:0 Scan a card, or Done.
Loaded. It holds: 1 BIP-39 mnemonic, 4 unrecognised record.
```

The SH2 has no camera; the gesture is holding an NFC tag to the reader. And the noun does not pluralise.

**Worse than saying nothing? NO.**

---

### F-592 — `md verify`'s conflict error prints a usage line containing the conflict

**Nit.** Classification: **documentation only**.

```
$ md verify --in policy-tr.txt <strings…>
error: the argument '--in <FILE>' cannot be used with '[STRINGS]...'
Usage: md verify --template <TEMPLATE> --in <FILE> [STRINGS]...
```

The suggested usage shows `--in` and `[STRINGS]` together. The deeper trap is that `--in` means the *strings* file
on `verify` and the *template* file on `encode`.

**Worse than saying nothing? NO.**

---

### F-593 — two policy files for one fixture differ only in the account index, and the README's numbers match neither journey

**Nit.** Classification: **documentation only**.

`design/fixtures/reasonably-complex-wallet/tr.policy` is account `0'`;
`design/journeys/inputs-rcw/policy-tr.txt` is account `8'`. The fixture README's table of template-ids, policy-ids
and first-receive addresses is computed at account 0, and says so — but it is the file an operator opens first, and
its addresses will never appear in a journey run at account 8.

**Worse than saying nothing? NO** — it is documented in the README itself.

---

### F-594 — `md bytecode` refuses an md1 that `md encode` just produced

**Minor.** Classification: **refusal**.

```
$ md encode "wsh(and_v(v:sha256(H),pk(@0/<0;1>/*)))" --group-size 0
md1yqpqqxpye4mfl0pwjzmtd9v2dmh90rsmzhyqrdf04y84yruhzy7npctpe9xel39qll8fe3cjrapd9
$ md bytecode md1yqpqqxpye4mfl0…
md: codec error: non-canonical wrapper requires explicit origin for @0, but none provided
```

One verb emits a string another verb in the same binary calls invalid. (This is the same decode failure F-578's
second hole swallows silently in `me bundle`.)

**Worse than saying nothing? WEAKLY YES** — the operator's natural conclusion is that their plate is corrupt.

---

### F-595 — `md address` cannot consume `md decode`'s own output

**Minor.** Classification: **refusal**.

```
$ md address --template "$(md decode <5 plates>)" --key @0=… --count 1
md: codec error: non-canonical wrapper requires explicit origin for @0, but none provided
```

`md decode` prints the template on stdout and the origins as a `note:` on stderr, so the pasteable half is missing
exactly what `md address` needs. Passing the *original* policy text plus `--key` and `--fingerprint` works:
`bc1pjwwr9p50xss2uc6tmlslsfdmgy6dssk0zwjrx23znttvn4l5r30s99rh3d`. But the restoring operator has plates, not the
original policy file — that is the whole point of the backup.

**Worse than saying nothing? WEAKLY YES** on the restore path, where the operator has only what is engraved.

---

### F-596 — preview text block left margin is inconsistent across wrapped lines

**Nit.** Classification: **not our concern / cosmetic**.

`plate-1.png`: lines 1-2 are inset ~115 px, lines 3-4 flush at x=0. I concatenated the four rendered lines and they
reproduce the chunk string exactly, so nothing is clipped.

**Worse than saying nothing? NO.**

---

### F-597 — sibling CLIs spell the same concept differently

**Nit.** Classification: **documentation only**.

`md encode --fingerprint @i=HEX` vs `mk encode --origin-fingerprint HEX` vs `mk encode --origin-path` (my first two
guesses, `--origin` and `--fingerprint`, both bounced). Clap's "a similar argument exists" tip recovered both times.
Separately, `ms derive --phrase-stdin` does not exist and the tip offers `--passphrase-stdin`, which is a different
secret entirely.

**Worse than saying nothing? NO** — the tips work.

---

## 3. What worked without friction

Do not spend effort re-testing these; they were exercised in this walk and behaved.

- **`ms hashlock`** — the best-documented surface in the set. All three fixture phrases reproduced their preimages
  and policy literals byte-exactly with `--method sha256`; the default `hardened` produced a different digest and
  the card's `method:` line said so unprompted. `--kind` guidance, the brainwallet warning, and the "write the
  method line AND the hash line" instruction are all correct and in the right place.
- **`md encode`'s unseatable-slots warning** — names the problem, the seven affected slots, the fix, and its cost
  ("about one extra md1 chunk and changes no path, no key and no policy"). I followed it without hesitating.
- **`md decode` / `md encode` round trip** — five plates back to the exact template plus all seven origins.
- **`me sysw pack` / `show`** — the `--pack-preimage` refusal names the record index and the fix; the bearer
  warnings are prominent and correctly worded; `show` reads back all 13 records and withholds the three secret ones.
- **The payload digest handshake** — `me sysw show` and the device screen agreed exactly
  (`dbe9e774e9a492310b62626c2b41cf4b`), and the device screen names the host command to compare against.
- **`me bundle --preview`** — once the sidecar matched, five PNGs, 0600, text complete and legible.
- **The emulator build** — `./cmd/emu/build.sh` worked first try and printed the artifact size; `shTap`/`shScreen`/
  `shNFC`/`shTargets` are a good driving surface and `walk_hashlock_phrase.js` documents it well.
- **Preview/sidecar failure modes** — version mismatch and a bad `ME_PREVIEW_BIN` both exit 2. No silent skip.
- **`ms` argv guard** — it does refuse, before parsing, on every channel I tried. (Its *remedy text* is F-581; the
  refusal itself is sound.)

## 4. What I could not reach, and why

- **Cut plates, or any engraving.** No hardware by instruction; the emulator's engrave leg needs a hold gesture and
  a seated cosigner, and I stopped at the Done refusal.
- **The preimage-plate leg on the device.** Blocked twice over: `shSysw` cannot load my payload (**F-585**), and the
  compiled-in composer blob carries no `phrase:` record, so there is no material for the device to cut a preimage
  plate from. The only route is retyping a phrase on the device by hand, which is a different journey.
- **Engraving this wallet from the 5-plate keyless set.** The device refuses it (**F-580**) and the mk1 key cards
  that would complete it cannot be produced (**F-587**). The 16-chunk keyed set reaches `Input Seed` and I stopped
  there rather than seat a cosigner.
- **Which record triggers the device's secret-in-flash flag** (**F-584**). Establishing the host/device disagreement
  needed no more than `me sysw show`; identifying the record would have meant a code audit.
- **Whether `mk 0.13.0` is stale or genuinely incompatible.** Per the brief I report the version rather than fix it.
