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

### The obvious two-step form: write the record to a file first

```console
$ mt encode --qr --bitcoin-cli /nonexistent/bitcoin-cli --in 'work/tx.hex' > 'work/rec.txt'
WARNING: nLockTime 96 is BELOW this build's reference height 963759.

  This transaction is not meaningfully time-locked -- its lock height
  passed before mt was built. Treat it as spendable now.

WARNING: no bitcoind reachable — mt could not check the chain before you cut.

  These are the questions a node would have answered, and mt has NOT:
  
  - are these inputs still unspent, or did something else take them?
  - what fee does this actually pay?
  - how far away is the locktime, in real blocks?
  
  Engraving takes about 21 minutes per plate and is permanent. Running
  mt again with a node reachable takes seconds and answers all three.
  If the inputs turn out to be spent, the plate is scrap the moment it
  leaves the machine.

WARNING: this is a RAW TRANSACTION, not a PSBT.

  A raw transaction carries its inputs' OUTPOINTS but not their
  VALUES, so mt cannot compute the fee from it alone.
  
  THE FEE IS UNKNOWN. mt cannot tell you whether it is 0.0001 BTC or 9
  BTC. Supply the values with --input-value <index>:<amount>, or
  re-run with a node reachable so mt can fetch them. (§8.2e)

WARNING: anyone holding this engraving can broadcast this transaction.

  mt checked that every input carries a signature committing to the
  outputs, so a holder should not be able to send the money anywhere
  else. That check reads WITNESS SHAPE, not script — mt has no
  script engine (§8.2). An exotic or hostile input CAN defeat it.
  Treat the engraving as if a holder could take the funds.

WARNING: when you are done, verify the ENGRAVING — not this output.

  SCAN the cut symbol with an ordinary QR reader and run:
  
  mt inspect --in scanned.hex
  
  It must report the same txid as the report above. Inspecting the
  file mt just produced tests nothing that can fail — and this
  machine has no camera, so nothing but you will ever look at the
  plate.

TX        2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
OUT       2 output(s)   (addresses shown as MAINNET — no node to ask)
            bc1qc80qm4p46822m9ldragav0u3eqqvcn4th8q3sl   0.05000000 BTC
            bc1qw5gf0s5e6c65lwevt2z9ztwhprefqt67ng6mjz   49.94998590 BTC
FEE       UNKNOWN   (needs input values, which the transaction
          does not carry)
LOCKTIME  LOCKED TO BLOCK 96          current height unknown (no node)
INPUTS    1 input(s)
            1abaa37ac432ac86…   UNKNOWN
STATUS    UNKNOWN — no node reachable
RECORD    one tx: record, 447 characters — for QR plates
          the device chooses the plate layout; mt does not

SUGGESTED LEGEND — cut this beside the symbol. mt cannot see your
plate, so the layout is yours (§3b); these are the five facts a
stranger needs BEFORE they can do anything with the steel.

    BEARER - ANYONE HOLDING THIS CAN BROADCAST IT
    FORMAT: raw transaction, QR — scan it, then broadcast
    FROM WALLET ????????        <-- NOT SUPPLIED
    TO ????????   <-- NOT SUPPLIED
    LOCKED TO BLOCK 96

  FROM WALLET and TO are NOT SUPPLIED. The transaction does not carry either
  fact — it names outpoints and scripts, not wallets — so mt cannot
  fill it in and will not guess. Supply --from / --to, or engrave the
  line by hand. A plate that says neither leaves a recoverer holding
  steel they cannot place.

  NO AMOUNT on the TO line: this transaction has 2 outputs and mt
  cannot tell which is the destination and which is CHANGE — that
  needs the sending wallet's descriptor, which mt never sees. Write
  the amount yourself if you want it on the plate; the report above
  lists every output.

WARNING: the record just left this terminal — and it is BEARER, exactly like the plate.

  stdout is not a terminal, so the record went somewhere that keeps it
  — a file, a pipe, or another program. Wherever that is, anyone who
  reads it can broadcast this transaction: it is the engraving, in a
  form that copies itself.
  
  If it landed in a FILE, destroy it once the plates are cut and
  verified: `shred -u <file>` on Linux, `rm -P <file>` on macOS. Plain
  `rm` unlinks the name and leaves the bytes. And check it is not
  already in a backup, a sync folder, or your editor's undo history.

mt encode: REFUSED — §8.2h, stdout is a file of mode 0644 — its permissions grant read to group or others.

  This record IS the engraving, and a finalized transaction is BEARER:
  anyone who can read that file can broadcast it.

  Only the file's OWN mode was checked. If a directory above it denies
  search to others -- a 0700 home directory does -- nobody else can
  open it today; the mode still becomes dangerous the moment the file
  is moved, copied, or its parent relaxed (F-252).
  
  mt has no --out: stdout IS the record, by design (§3b). So the
  remedies are the shell's:
  
  umask 077 then re-run; the shell creates it 0600
  chmod 600 <file> then re-run -- `>` truncates but keeps the mode
  --allow-world-readable proceed anyway
(exit 1)
```

### So it is a PIPE — mt owns transactions, me owns the container

```console
$ mt encode --qr --bitcoin-cli /nonexistent/bitcoin-cli --in 'work/tx.hex' | me sysw pack --region --out 'work/region.bin'
WARNING: nLockTime 96 is BELOW this build's reference height 963759.

  This transaction is not meaningfully time-locked -- its lock height
  passed before mt was built. Treat it as spendable now.

WARNING: no bitcoind reachable — mt could not check the chain before you cut.

  These are the questions a node would have answered, and mt has NOT:
  
  - are these inputs still unspent, or did something else take them?
  - what fee does this actually pay?
  - how far away is the locktime, in real blocks?
  
  Engraving takes about 21 minutes per plate and is permanent. Running
  mt again with a node reachable takes seconds and answers all three.
  If the inputs turn out to be spent, the plate is scrap the moment it
  leaves the machine.

WARNING: this is a RAW TRANSACTION, not a PSBT.

  A raw transaction carries its inputs' OUTPOINTS but not their
  VALUES, so mt cannot compute the fee from it alone.
  
  THE FEE IS UNKNOWN. mt cannot tell you whether it is 0.0001 BTC or 9
  BTC. Supply the values with --input-value <index>:<amount>, or
  re-run with a node reachable so mt can fetch them. (§8.2e)

WARNING: anyone holding this engraving can broadcast this transaction.

  mt checked that every input carries a signature committing to the
  outputs, so a holder should not be able to send the money anywhere
  else. That check reads WITNESS SHAPE, not script — mt has no
  script engine (§8.2). An exotic or hostile input CAN defeat it.
  Treat the engraving as if a holder could take the funds.

WARNING: when you are done, verify the ENGRAVING — not this output.

  SCAN the cut symbol with an ordinary QR reader and run:
  
  mt inspect --in scanned.hex
  
  It must report the same txid as the report above. Inspecting the
  file mt just produced tests nothing that can fail — and this
  machine has no camera, so nothing but you will ever look at the
  plate.

TX        2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
OUT       2 output(s)   (addresses shown as MAINNET — no node to ask)
            bc1qc80qm4p46822m9ldragav0u3eqqvcn4th8q3sl   0.05000000 BTC
            bc1qw5gf0s5e6c65lwevt2z9ztwhprefqt67ng6mjz   49.94998590 BTC
FEE       UNKNOWN   (needs input values, which the transaction
          does not carry)
LOCKTIME  LOCKED TO BLOCK 96          current height unknown (no node)
INPUTS    1 input(s)
            1abaa37ac432ac86…   UNKNOWN
STATUS    UNKNOWN — no node reachable
RECORD    one tx: record, 447 characters — for QR plates
          the device chooses the plate layout; mt does not

SUGGESTED LEGEND — cut this beside the symbol. mt cannot see your
plate, so the layout is yours (§3b); these are the five facts a
stranger needs BEFORE they can do anything with the steel.

    BEARER - ANYONE HOLDING THIS CAN BROADCAST IT
    FORMAT: raw transaction, QR — scan it, then broadcast
    FROM WALLET ????????        <-- NOT SUPPLIED
    TO ????????   <-- NOT SUPPLIED
    LOCKED TO BLOCK 96

  FROM WALLET and TO are NOT SUPPLIED. The transaction does not carry either
  fact — it names outpoints and scripts, not wallets — so mt cannot
  fill it in and will not guess. Supply --from / --to, or engrave the
  line by hand. A plate that says neither leaves a recoverer holding
  steel they cannot place.

  NO AMOUNT on the TO line: this transaction has 2 outputs and mt
  cannot tell which is the destination and which is CHANGE — that
  needs the sending wallet's descriptor, which mt never sees. Write
  the amount yourself if you want it on the plate; the report above
  lists every output.

WARNING: the record just left this terminal — and it is BEARER, exactly like the plate.

  stdout is not a terminal, so the record went somewhere that keeps it
  — a file, a pipe, or another program. Wherever that is, anyone who
  reads it can broadcast this transaction: it is the engraving, in a
  form that copies itself.
  
  If it landed in a FILE, destroy it once the plates are cut and
  verified: `shred -u <file>` on Linux, `rm -P <file>` on macOS. Plain
  `rm` unlinks the name and leaves the bytes. And check it is not
  already in a backup, a sync folder, or your editor's undo history.

sealing:  NOT SEALED — no record in this payload is secret material, so there 
      is nothing to encrypt. The container is cleartext: anyone holding the file 
      can read it.
strength: no passphrase — BELOW the threshold
digest:   c282 6ca8 4f21 2887 02cc 70f0 91d7 5d34
          re-print it with: me sysw show work/region.bin
me: region image — 499 bytes of container, padded with 0xFF to 65536; write it at 0x10D00000
(exit 0)
```

### The refusal that keeps it off argv

```console
$ me sysw pack --no-passphrase 'tx:<the 222-byte transaction, elided>'
me: record 0, as given (records count from 0), is a `tx:` record on ARGV. Refused; nothing was read and nothing was written.
      A raw signed transaction is a BEARER instrument -- anyone who can read it can broadcast it -- and argv is public: /proc, `ps` and your shell history all keep a copy.
      Use a private channel instead:
          mt encode --qr --in tx.hex | me sysw pack --out p.bin
          me sysw pack --in records.txt --out p.bin
(exit 3)
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

### An unsigned transaction never becomes a record — the PRODUCER refuses

```console
$ mt encode --qr --bitcoin-cli /nonexistent/bitcoin-cli --in 'work/stripped.hex' | me sysw pack --no-passphrase --out 'work/never.bin'
mt encode: REFUSED — §8.3, 1 of 1 inputs carry no signature (input 0)

  Each of these inputs has an empty scriptSig AND an empty witness, so
  nothing satisfies it. An unsigned transaction cannot be broadcast,
  so engraving it produces a plate that is not a backup of anything.

  Sign it first — `walletprocesspsbt`, then `finalizepsbt`.
me: no records on stdin: pass them on argv, with --in, or on stdin.
      An EMPTY input is what a FAILED upstream command leaves behind -- `mt encode --qr > rec.txt` writes nothing when it refuses -- so it is refused here rather than packed into a container that holds nothing and still flashes.
(exit 2)
```

### …and the consumer still refuses it on its own, if one reaches it by hand

```console
$ printf 'tx:%s\n' '<the 113-byte stripped transaction, elided>' | me sysw pack --no-passphrase --out /dev/null
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
