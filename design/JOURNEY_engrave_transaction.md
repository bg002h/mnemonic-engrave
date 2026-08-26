# JOURNEY — engrave a transaction

**Regenerate with `scripts/gen-tx-journey.sh`.** Nothing below is illustrative:
every host block is that script's real stdout+stderr with its actual exit code,
and every device screen is captured by `gui.TestCaptureTransactionJourney`,
which is the **walk, instrumented** — the same harness driving the same flow
that `TestWalkQRPathFromATxRecordToThePostCutScreen` asserts on. A screen that
changes fails the walk before it reaches this document.

**Two things it deliberately does not do**, so nobody reads more into it than is
here:

- The device screens are `op.Drawer.ExtractText` over the firmware's own op
  tree, **not** the emulator's 480×320 framebuffer the way
  `design/journeys/*.pdf` are. Same tree, same text, different renderer: this
  shows **what** the device says and cannot show how it **looks**. The
  framebuffer capture needs a WASM build and playwright, and it belongs with the
  P4 hardware session — where a photograph of real steel goes beside it.
- **No plate is cut.** G-P4.1 owns the steel, and until it runs no QR this
  feature emits has ever been read off metal.

The transaction throughout is the pinned `even` vector: a real signed 222-byte
1-in/2-out P2WPKH transaction, txid `2dcf2b97…`, the same artifact the Rust, Go
and `gui` suites all pin — so every layer of this journey is one transaction.

---

## Part 1 — the host

### The operator has a finalized transaction, as hex

```console
$ head -c 96 'work/tx.hex'; echo '...'
020000000001017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff...
(exit 0)
```

### Turn it into a record. argv is refused for this class, so it is a pipe

```console
$ me tx --in 'work/tx.hex' > 'work/rec.txt'; head -c 40 'work/rec.txt'; echo '...'
tx:020000000001017c8da925af70e49a12b0cea...
txid: 2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
size: 222 bytes, 1 input(s), 2 output(s), segwit
(exit 0)
```

### The refusal that makes it a pipe

```console
$ me sysw pack --no-passphrase 'tx:<the 222-byte transaction, elided>'
me: record 0, as given (records count from 0), is a `tx:` record on ARGV. Refused; nothing was read and nothing was written.
      A raw signed transaction is a BEARER instrument -- anyone who can read it can broadcast it -- and argv is public: /proc, `ps` and your shell history all keep a copy.
      Use a private channel instead:
          me tx --in tx.hex | me sysw pack --no-passphrase --out p.bin
          me sysw pack --in records.txt --out p.bin
(exit 3)
```

### Pack it. Nothing here is secret, so nothing is sealed

```console
$ me sysw pack --region --out 'work/region.bin' --in 'work/rec.txt'
sealing:  NOT SEALED — no record in this payload is secret material, so there 
      is nothing to encrypt. The container is cleartext: anyone holding the file 
      can read it.
strength: no passphrase — BELOW the threshold
digest:   c282 6ca8 4f21 2887 02cc 70f0 91d7 5d34
          re-print it with: me sysw show work/region.bin
me: region image — 499 bytes of container, padded with 0xFF to 65536; write it at 0x10D00000
(exit 0)
```

### What is in it, and the digest to compare on the machine

```console
$ me sysw show 'work/region.bin'
sealed:   false
pub_len:  447
ct_len:   0
identity: a9c3197d5fcc21e5f984b8eb3c26607a2e0ace1f39d9f1d0c7d6b67155aca3c8
public record 0: raw signed transaction — txid 2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630, 222 bytes
digest:   c282 6ca8 4f21 2887 02cc 70f0 91d7 5d34
(exit 0)
```

### A set missing three of its six strings still packs, loudly

```console
$ printf '%s\n%s\n%s\n'       mt1p9h8jqq9qqqqgqqqqqqqyqherdfykhhpey6z2cvafak8804qd7g0dl6v8ex9wr2cvky023skwkeud2229sax       mt1p9h8jqq9qqzj8yqpnzw4vl2rwffqyqqqqqkqq282yyhc2vavd20hvk94pz39hts3u5s9a0qd8pwskxfl7ju5       mt1p9h8jqq9qq9qdcc7h75twfxyf340c4sgqzhfdq6xtgt7zhxngpwa049l0z59l6jqcqqqqqq5k5y2ye5nv8yf       | me sysw pack --no-passphrase --out 'work/partial.bin'
sealing:  NOT SEALED — no record in this payload is secret material, so there 
      is nothing to encrypt. The container is cleartext: anyone holding the file 
      can read it.
strength: no passphrase — BELOW the threshold
me: mt1 set 2dcf2 (records 0, 1, 2, as given; records count from 0) did NOT 
      confirm as one signed transaction. MISSING strings 2, 4 and 5 of 6. Pack every string of the set -- 
      `mt encode` emits them all, and `--elide-prefix` output is refused here.
      It is PACKED and ENGRAVEABLE anyway (ruling 2026-08-25) -- every mt1 
      string is independently valid, so the strings you have are worth cutting 
      and a missing one can be added later.
      The device will REPLACE your plate legend with a re-encode warning, and 
      QR plates will be unavailable: a set that does not reassemble has no 
      transaction bytes to encode.
digest:   c2af 6413 749c 2241 319b 95aa 0ee2 532e
          re-print it with: me sysw show work/partial.bin
(exit 0)
```

### An unsigned transaction is refused, and it names the input

```console
$ printf 'tx:<the 113-byte stripped transaction, elided>\n' | me sysw pack --no-passphrase --out /dev/null
sealing:  NOT SEALED — no record in this payload is secret material, so there 
      is nothing to encrypt. The container is cleartext: anyone holding the file 
      can read it.
strength: no passphrase — BELOW the threshold
me: record 0 (records count from 0) is a `tx:` record whose transaction parses but whose input 0 carries NEITHER a scriptSig NOR a witness — it is unsigned, or its signatures were stripped in transit.
      This is refused because the txid does NOT change when signatures are removed: the record would show the txid you expect, and the plate cut from it could never be broadcast.
      Re-export the FINALIZED transaction from your signer.
      If those inputs are honestly empty (a P2A anchor spend and similar exotica), pass --allow-unsigned-inputs.
(exit 4)
```

---

## Part 2 — the device

The payload is written to `0x10D00000` with `picotool`. At boot the machine
offers to load it, the operator compares the digest above, and the payload menu
says what it holds. Then: **Engrave Transaction**.

### 1. Review — the transaction the payload holds

```
TransactionEngravethistransaction?BEARER:anyon
eholdingtheplatescanbroadcastit.2dcf2b973d5204
4b1e58c988a5a59d38
```

### 2. Which kind of plate

```
ChooseplatekindQRPLATESEngraveTransaction
```

### 3. The plan, before anything is cut

```
Plates1plate(s),1QR,ECCH,0.6mmmodulesabout30mi
nofcuttingEngrave?
```

### 4. The plate, named and numbered

```
EngravethisplateENGRAVETX2DCF2B971/1
```

### 5. After the last plate — test it now

```
PlatesCut1plate(s)cut.TESTTHEMNOW,beforeyoufil
ethem.ScaneveryQRwithaphone,jointhehex,andrun
```

### 5 (page 2). The command that reads it back

```
PlatesCut`mtinspect`onit.Orderdoesnotmatter.Ch
eckthetxiditreportsagainstTX2DCF2B97.
```

### 5 (page 3). Why this machine cannot check it for you

```
PlatesCutThismachinehasnocameraandcanneverread
aplateback.
```

---

## What the operator is left holding

One steel plate carrying one QR symbol and a legend that states the txid, what
the symbol contains, and what to do with it. The plate is a **bearer
instrument**: anyone who scans it can broadcast the transaction. The review
screen says so on its first page — which is where it had to be moved to, because
the screen pages and the warning was below the fold.

**The device never checks its own work.** It has no camera, so the post-cut
screen is the last moment anything can tell the operator to look at what came
out. That is why it says so.
