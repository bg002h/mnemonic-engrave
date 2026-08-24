# SPEC — Engrave a Transaction (SeedHammer II program)

**Status:** DRAFT, pre-R0. Written 2026-08-24; **folded 2026-08-24 against the
operator journey walk** (`JOURNEY_WALK_engrave_transaction.md`), which by ruling
is this document's review. The walk produced **18 findings (A–R)** and one
Critical in shipped code (F-244).

**The fold made this smaller.** Two rulings removed work the first draft
described: chunks are engraved **verbatim**, so v1 needs no `mt1` decoder (§2.2);
and applicability lives in a **payload menu**, so the carousel is untouched
(§3.1). Scope shrank rather than grew.

**Goal 1 of the two set by the operator 2026-08-24** (`CONTINUITY_mt_2026-08-24.md`).

---

## 0. Scope

**In:** a device program that accepts an already-signed Bitcoin transaction,
comprehends it, shows the operator what it is, and engraves it — plus the
container and host plumbing that gets it there.

**Out:** transaction construction, signing, broadcasting, script evaluation,
on-device fee derivation, and any change to the Sealed Payload (`seal/`,
`0xE1000000`), which stays frozen.

**Also out, deliberately: F-244.** The walk found `me sysw pack` writes an
unsealed mnemonic-bearing container at mode 0644. It is Critical, it is
pre-existing, it affects seeds today, and **it must not wait for Goal 1.**

**Risk set.** Fork-native firmware touching funds-adjacent material, plus a
normative container change. R0 gate applies: **no code before 0C/0I.**

---

## 1. The pipeline, and who owns each stage

```
tx.final.psbt ─▶ mt encode --record --raw|--chunks ─▶ tx: record
                        (mnemonic-transaction)            │
                                                          ▼
                                             me sysw pack  (stdin or --in)
                                                (mnemonic-engrave)
                                                          │
                                        ┌─────────────────┴──────────────┐
                                   --region                          NFC tag
                                   picotool                              │
                                   0x10D00000                            │
                                        └─────────────────┬──────────────┘
                                                          ▼
                                        SeedHammer: load ─▶ payload menu
                                                    ─▶ comprehend ─▶ confirm ─▶ cut
```

| owner | owns | gains |
| --- | --- | --- |
| `mt` | the transaction and the `mt1` codec | `--record`, and a **raw-transaction subject for `inspect`** (§6, finding O) |
| `me` | the `sysw` container | `ClassTransaction`; **a stdin path**; content-based sealing |
| fork `sysw/` | reading the container | a port of the above, provenance-pinned |
| fork `gui/` | comprehension and the plate | the program, and the payload menu |

**`me` packs; `mt` does not.** `sysw/wire.go:10-12` forbids a second packer by
the same argument that forbids the device packing: two implementations of one
container can disagree.

**Rust-primary.** The container lands in `me` with vectors first and reaches the
fork as a port.

### 1.1 The join is stdin — the first draft was wrong about this

The first draft said the two tools *"compose over a pipe."* **They do not**:
`me sysw pack` took `--in <FILE>` and argv only. Measured:

```
$ printf 'text:6869\n' | me sysw pack --no-passphrase
me: no records: pass them on argv or with --in     (exit 2, stdout empty)
```

**RULED: `me sysw pack` gains a stdin path**, so the natural command works:

```
mt encode --record --raw < tx.final.psbt | me sysw pack --region --out region.bin
```

**Two invariants this pipeline depends on, and both MUST be stated and tested
rather than inherited by luck** (finding D). `fish` reports a pipeline's status
as the *last* command's, so an upstream failure is otherwise invisible
(`false | true` → `status=0`, `pipestatus=1 0`):

1. **`mt` contributes NOTHING to stdout on any failure path.** Measured today:
   `mt encode --in bad.hex` → exit 1, **stdout 0 bytes**.
2. **`me sysw pack` refuses empty stdin** rather than packing an empty container.
   Stdin must join the existing "no records" exit-2 path, not bypass it.

Together these make a failed encode produce a non-zero pipeline status. Neither
is currently asserted as a *pipeline* property.

---

## 2. The container

### 2.1 One framed record under `tx:`

`ClassTransaction` is one record carrying the transaction (or its chunks) **and**
the legend fields `mt encode` already computes. One record, not siblings, so the
legend stays bound to what it describes.

`tx:` inherits the reserved-prefix rule (`sysw/record.go:41-51`,
`gui/scan.go:56-80`): a body that is not lowercase hex is `ClassUnknown` and
**refused before any sniffer sees it**, so it can never fall through to free text
and become a plate.

**No new secrecy class.** A transaction rides beside `ClassMDMK` and
`ClassFreeText`.

**A `tx:` record on argv is REFUSED** (finding B). `mt` refuses a transaction as
an argument because *"an argument lands in shell history and in `ps` for every
user on the machine, and this material is bearer."* `me sysw pack`'s help
currently says only *"**prefer** `--in` for anything real"* — so the
constellation would refuse the leak upstream and permit it downstream. **Prefer
is not enough for a bearer instrument.**

### 2.2 Raw transaction XOR chunks — and chunks are engraved VERBATIM

**RULED: the payload carries the raw transaction OR its `mt1` chunks, never
both.** This supersedes the first draft's *"always decode and compare"* — with
one form present there is nothing to compare, and a requirement its own
architecture made unreachable is how a check that does not exist gets asserted.

**RULED: no default.** `mt encode --record` refuses without `--raw` or
`--chunks`, and **the refusal teaches**, because a bare blocking refusal is what
gets aliased away:

```
mt: --record needs a form. Say which:

  --raw      the transaction's bytes. QR plates only.
             The device needs no mt1 decoder.

  --chunks   mt1 strings. Text plates.
             The device engraves them verbatim.
```

**RULED, and this is what shrank v1:**

| payload | the device does | it needs |
| --- | --- | --- |
| **raw** | parse → comprehend → confirm → **QR plates** | a transaction parser |
| **chunks** | **engrave verbatim** → **text plates** | **nothing new** |

Chunks take the existing `mdmkText` / `validateMdmk` path already used for
`md1`/`mk1` cards. **There is no `mt1` decoder in v1.**

> **THIS IS A DELIBERATE EXCEPTION TO §3.3's "COMPREHEND, THEN CUT", AND IT IS
> STATED HERE IN THOSE WORDS SO A LATER READER DOES NOT "FIX" IT.** Comprehension
> did not disappear; it **moved upstream**. `mt encode` built those chunks from a
> transaction the operator inspected on the host. The device adds nothing by
> re-deriving it, and would need a decoder to try.

**Chunks means text plates ONLY, never QR.** F-234 forbids an `mt1` string as QR
content, and without a decoder the device cannot recover the bytes. Each form
produces exactly one kind of plate.

**ACCEPTED COST, stated plainly:** a chunks plate is cut with the device making
**no claim whatever** about its content — no destination, no amount, no txid.

**PROPOSED, not ruled:** the device verifies **each chunk's own BCH checksum**
before cutting. The fork already carries the engine (`codex32/gf32.go`,
`gf1024.go`, `checksum.go`) and already exposes `ValidMD`/`ValidMK`. Far less
than a decoder, and it catches garbage before ~21 minutes per plate of scrap.

### 2.3 The section cap rises to 32,734

```
MaxSectionLen = (RegionLen − HeaderLen − TagLen) / 2
              = (65536 − 52 − 16) / 2 = 32,734
```

**Why 8191 existed.** It is the **NFC scan buffer minus one**: `gui/scan.go:31`
allocates `8*1024` and flags overflow at *exactly full*. EPD §6.2 ruled it
(`SPEC_encrypted_payload_delivery.md:569`); `sysw/wire.go:39-41` **inherited it
unchanged**, so the flash path was capped at an eighth of its region for a reason
belonging to NFC.

**Why the formula.** It preserves the property 8191 has and that `boundBlob`'s
32-bit no-wrap argument rests on — two maxed sections plus header plus tag still
fit the region. Today that is `52 + 8191 + 8191 + 16 = 16,450`, the figure
`gui/gui.go` quotes. A round 32,768 breaks it by 34 bytes.

**What it buys**: a **16,367-byte** raw transaction — 2× the worst measured
pathological spend. Measured (`RESULTS_2026-08-22.txt`, pathological wallet,
11 keys / 3 masters, `wsh`, tier 1 = 3-of-3 + hash, the most expensive path):

| in/out | signed tx | as hex | chunks | bytes/chunk | as chars | fits 32,734? |
| --- | --- | --- | --- | --- | --- | --- |
| 1/1 | 852 B | 1,704 | 22 | 39 | 2,001 | ✅ |
| 1/2 | 893 B | 1,786 | 23 | 39 | 2,092 | ✅ |
| 2/2 | 1,692 B | 3,384 | 43 | 40 | 3,955 | ✅ |
| 5/2 | 4,080 B | 8,160 | 102 | 40 | 9,383 | ✅ (**raw-only at 8191, by 31 chars**) |
| 10/2 | 8,067 B | 16,134 | 202 | 40 | 18,583 | ✅ |

Computed, not estimated: `chunks = ceil(bytes/40)`,
`bytes_per_chunk = ceil(bytes/chunks)` (**balanced, not filled**),
`chars = ceil((bytes_per_chunk×8 + 55)/5) + 16` per chunk plus a newline. The
character formula reproduces the three shipped vector lengths (79 / 85 / 87 at
32 / 36 / 37 bytes) exactly. A full 40-byte chunk is **91** characters, not the
`~96` at `SPEC_mt_v0_1.md:1562` — see F-242, and note `:1308` already says 91.

**Four things the raise touches:**

1. `boundBlob`'s comment names `8191` and goes stale on landing.
2. **NFC stays at 8191** — `gui/scan.go`'s buffer bounds it — so a large
   transaction is **picotool-only**, and `me sysw pack` MUST say which transports
   its output fits.
3. Rust-primary: `me`'s `sysw` first, with vectors.
4. **`seal` is untouched**, keeps its own 8191, stays frozen.

### 2.4 Sealing is decided by CONTENT

`me sysw pack` seals by default — correct for mnemonics, **wrong here**, and
contrary to the operator's 2026-08-23 ruling *"send via payload unencrypted"*.
Sealing a transaction payload costs the operator a 12-word passphrase to store,
**typing those 12 words on the device's on-screen keyboard**, ~31 s of KDF
(`gui/sysw_load.go:103`), and a new failure mode — lose the words, lose the
backup — to protect a payload whose whole purpose is to become **a steel plate
anyone can read**.

**RULED: `me sysw pack` seals when the payload holds any `Class::IsSecret()`
record and does not when it holds none.** The predicate exists
(`sysw/record.go:37`) and names `ClassMnemonic`, `ClassCodex32Secret`,
`ClassPassphrase`.

**It MUST say which way it went, and why, on stderr, every time.** A
content-dependent default that is silent is worse than the default it replaces.

### 2.5 Output files

**RULED (scope: all of `me`, and `mt` too):**

```
stdout is a TTY          → nothing. Nothing persists.
stdout is a pipe/FIFO    → nothing. No file mode exists.
stdout is a regular file ┐
or --out FILE            ┘→ mode grants group or other read?
                              → REFUSE unless --allow-world-readable

--out additionally creates at 0600 AND fchmods an existing target to 0600
```

Mechanically verified, not assumed: a process can `fstat(1)` its own redirected
stdout and sees `S_ISREG` + mode 0644, and sees `S_ISFIFO` for a pipe — so the
check fires exactly where the exposure is.

**The `fchmod` half is load-bearing.** `write_private`'s documented residual —
*"0o600 binds on CREATE"* — was **measured true**: a pre-existing 0644 target
stays 0644. Creating carefully is not enough.

`mt`'s existing `redirected_output_warning` becomes mode-aware in the same pass;
today it fires on *any* redirection and never reads the mode, so it cries wolf on
a 0600 file and warns no harder on a 0644 one.

---

## 3. The device

### 3.1 The carousel does not change. Applicability lives in the payload menu.

**RULED: all carousel items are shown always**, because every program may
eventually start an NFC transfer — so payload-independence of the carousel is
**correct, not a limitation**.

**Two screens, two questions:**

| | asks | content-dependent? |
| --- | --- | --- |
| the carousel | what can this **machine** do? | **never** |
| the payload menu | what can **this payload** do? | **by construction** |

**Nothing in the carousel changes** — `lastNav`, the compile-time guard
`[qaProgram - unlockPayload]struct{}{}`, `layoutMainPager` and every wrap site
are untouched. Two earlier forms of this ruling (hide inapplicable entries; show
them dimmed) were retracted; see finding P for why each was worse.

`engraveTransaction` is still **inserted mid-enum** before `loadPayload`, per the
house rule for unconditional programs, and placed beside the other engrave
programs.

### 3.2 The compare screen names the command

Before the payload menu comes the load flow's authentication: the device displays
the identity digest and asks the operator to compare it
(`gui/sysw_load.go:168`). **That number was printed by `me sysw pack` to stderr,
in a terminal, possibly an hour ago and possibly on a laptop now closed** — and
the screen currently says *"Compare this against what"* without naming what, or
how to get it.

**Measured during the walk:** the digest is over the RECORDS, so it is
**reproducible**, **identical for sealed and unsealed**, and **independent of the
passphrase**. Re-running `me sysw pack` does return the same number.

**But re-packing is the risky recovery, and it is the one an operator reaches
for.** On a sealed payload it prints a **brand-new 12-word passphrase every
time**, which reads as *"this is a different container"*. An operator who acts on
that belief and **re-flashes** ends up with a payload whose passphrase they saw
once and did not keep.

**RULED: the device prints the command beneath the digit groups.**

```
Compare this against
  me sysw show <file>

c679 6b68 b993 bc10
793a 3de2 8b3d 46e0
```

`me sysw show` already exists — *"Print what a container holds, and its digest"* —
is **read-only**, and prints `sealed:`, `pub_len:`, `ct_len:` and the same digest.
Putting it on this screen is better than documenting it, for the reason the walk
found it: **the operator is standing at the machine**, and a manual they would
have to go and open is exactly what they cannot reach. It also steers them off
the re-pack path *before* they take it, rather than warning them afterwards.

`me sysw pack`'s digest line carries the same pointer, so the host says it too.

### 3.3 The payload menu

**RULED: it appears immediately after a successful load.**

```
boot → "payload present, load it?" → LOAD → compare digest
     → THE PAYLOAD MENU  ("this payload holds: 1 transaction, 2 seeds")
     → BACK exits to the carousel
```

`syswPayloadMenu` exists (`gui/sysw_unload.go:34`) and today offers only
`LOAD AGAIN` / `UNLOAD`; it gains content-derived entries above them.
`sysw.Classify` already computes what a payload holds.

**BACK is the exit and must be**, for the same reason `syswUnloadFlow`'s BACK is
choice 0: the resting position is the one that costs nothing.

The carousel entry stays reachable and must still refuse gracefully — as a
backstop, not the path — and **its refusal names the FIX, not just the problem**:
*"this payload holds no transaction — load one with Load Payload."*

### 3.4 Comprehend, then cut — the raw form

The device parses with `btcd/wire/v2` (an existing indirect dependency). The
organising rule:

> **The screen separates what the transaction PROVES from what the operator
> ASSERTED, and never renders them in one voice.**

| | source | fields |
| --- | --- | --- |
| **Derived** | the bytes; the device stands behind it | txid, input count, each output's address and amount, locktime, `nSequence` |
| **Asserted** | the payload's legend fields; the operator's words | the `TO` label, the fee |

Forced, not stylistic: **fee is not in a signed transaction** (it needs input
values), and F-235 established `TO` comes from `--to-label`.

**Network.** A `scriptPubKey` carries no network, so the device cannot know it.
The address row MUST state which parameters it rendered under. Do **not** fix
this by suppressing the address — it is the most useful row a recoverer has.

### 3.5 Two screens, because 240×240 will not hold one

The display is **240×240** (`gui/gui_test.go:383`). A single confirm screen is at
its limit for a 1-in/1-out transaction **with no change** — and change is the
normal case. The split follows the two jobs the screen was doing at once:

| screen | job | contents |
| --- | --- | --- |
| 1 | **where the money goes** | outputs (paged), amounts, network params, IN count, FEE, locktime |
| 2 | **which transaction** | the **full 64-hex txid** in 16 groups of 4 — the shape the identity digest already uses. **`ENGRAVE` lives here and nowhere else** |

**RULED: the operator is NOT forced through every output page.** Page 1 shows the
first output plus a total, and `ENGRAVE` is reachable. The argument is stronger
than the one against: **a forced page-through that operators learn to tap past is
worse than an honest summary, because it manufactures the appearance of review.**
Verification happens on the host with `mt inspect`; the device is a second look.

**ACCEPTED RESIDUAL:** an operator can reach `ENGRAVE` having seen one
destination of N.

**The total MUST NOT be spelled as a destination amount.** This is a known defect
class here — one of the `mt` cycle's five near-miss failures *"printed the sum of
all outputs as the destination amount, for steel."* **Change outputs are yours.**
So the line reads `N outputs, X.XXXXXXXX BTC total` and MUST NOT be labelled
`TO`, `SENDING`, or any word implying a recipient.

**The txid is shown for RECOGNITION and MUST NOT be claimed as proof.** It is
blind to the **entire witness region** — where the signatures live and where most
of the bytes are — so damage there re-derives the expected id and passes **always**.
`mt` withdrew this exact claim after making it; `mt verify` now says *"this check
identifies the transaction. It does NOT prove every byte."* **The device carries
the same limit on the same screen.**

Byte integrity in transit is covered elsewhere, which is what makes the division
clean: by the identity digest the operator compared at load (unsealed), or by
AEAD (sealed). Two checks, two jobs, neither claiming the other's.

### 3.6 Several transactions in one payload

The picker is keyed on the **txid** — the derived, collision-free identifier, and
the one that matches `mt inspect` on the host. The `TO` label may ride as a
second line but is **asserted** and can collide; three transactions could all read
"cold storage".

**The picker shows a PREFIX. The prefix DISTINGUISHES; it never VERIFIES.**
`mt`'s own help names where truncation turns dangerous: a 20-bit set id is
*"1 in 1,048,576 by accident, and under a second to construct deliberately."* The
prefix separates transactions inside a payload the operator packed; the **full
txid on screen 2** is what gets compared.

**Two identical txids in one payload is the same transaction packed twice** — a
duplicate to refuse or collapse, never two picker entries.

### 3.7 What the device does not do

- **No script evaluation.** Signatures are recognised **by shape**, as in `mt`,
  and the device says so. It cannot detect a bad signature — an accepted,
  recorded hazard, restated here so it is not rediscovered as a defect.
- **No construction, signing or broadcast. No fee derivation.**
- **NO CAMERA — no on-device read-back, ever.** `driver/` holds two NFC readers
  (`clrc663`, `st25r3916`), touch, display, steppers, USB-PD and the machine;
  **no image sensor**, and `scanner.Scan` has exactly one feed
  (`gui/nfc_scan.go:62`). **The device writes a QR it has no way to read.** This
  is recorded so no future phase plans a read-back, and it is why §4.3's plate
  test is the operator's.

---

## 4. The plate

### 4.1 Composition

**Default: QR + legend** (raw payload). **Text plates** come only from a chunks
payload. The device states plate count and cut time before the operator commits —
~21 minutes per plate (F-225), and the forms are not close: the pathological
10-in/2-out spend is **~9–11 QR plates** or **202 text plates**.

### 4.2 What the QR carries

**The raw transaction bytes** — F-234, not re-litigable.

**The ENCODING of those bytes is a PARAMETER, resolved by the test plate**
(F-243), not ruled here. The argument that previously settled this **does not
bind**: base45 was rejected because its alphabet contains SPACE and EPD §6.4
forbids interior whitespace *in a `sysw` record* — and in that architecture *"the
record stores lowercase; `mt` uppercases only when encoding the QR symbol"*, so
record and QR were **one string**. **Here they are decoupled**: the record carries
hex, the QR is generated **on-device** from parsed bytes and never passes through
a record.

**F-243 is more urgent than it was filed as.** It was filed as *"can a stranger
read this in 15 years"*. §4.3's ruling makes it **"can the operator complete a
mandatory step, today, every time"**:

- **raw octets** — phone scanners mangle bytes ≥ `0x80`. **A good plate appears
  to fail.**
- **base45 / bech32-uppercase** — every scanner shows clean text, but the
  operator cannot tell it is the *right* transaction without a tool.

### 4.3 After the cut, the device says to TEST THE PLATE

The `mt` cycle's Critical was exactly this shape — **a silent step**. So:

```
PLATE 1 OF 2 — CUT
────────────────────────
TEST IT NOW, before you
leave the machine.

Scan the QR, then run
  mt inspect
on what you get.
────────────────────────
This machine has no camera.
It cannot check its own work.
────────────────────────
NEXT PLATE          DONE
```

The last block tells the operator **why** the job is theirs, which is what makes
them do it. **RULED: the test is "scan, then `mt inspect`."** A host round trip,
and the right cost — the plate is still in the machine, the cheapest moment to
re-cut.

**This requires new `mt` scope; see §6, finding O.** No `mt` verb can read a raw
transaction today.

### 4.4 The legend is cut LAST

**RULED: legend last; an incomplete plate is discarded; there is no resume.**

**The legend is the plate's claim about itself.** Cut last, a plate only claims to
be `PLATE 2 OF 3` once it is one. Cut first, it is a claim the plate has not
earned. This is §3.4's anti-overclaim discipline applied to the **artifact**.

> **AN UNSIGNED PLATE IS AN UNFINISHED PLATE.**

Visible at a glance, in a drawer, with no tooling — which matters precisely
because the device has no camera and the operator is the only inspector.

**No resume, for a mechanical reason and not a preference:** re-clamping cannot
guarantee the plate returns to the same origin, and this machine has already
produced a misregistration artefact traced to **Y-axis play from a loose screw**,
found only after four software hypotheses failed. A resumed cut would be offset
and would still look finished.

**The device must SAY to discard it.** It knows it stopped mid-cut, and the
operator is holding twenty minutes of steel they will be tempted to keep.

**The two forms fail differently, and only one fails safe in the steel:** a
partial QR will not scan; a partial **text** plate carries chunks whose checksums
all hold and looks real — it fails safe only because `mt` requires chunks
`1..count`. **That safety lives in the host tool, not the artifact.**

### 4.5 The configuration search

**The DEVICE runs it.** §4 of `SPEC_mt_qr_DEFERRED.md` was written for a host
verb this design does not have; its objective is stated in **plates and minutes**,
and only the device holds `EngraverParams`.

```
search space:  module size × QR version (1..40) × ECC (L,M,Q,H)
               × rectangular tiling (across × rows)
objective:     1. minimise plates    ← a plate holds the QR(s) AND the legend
               2. maximise ECC
               3. minimise symbol count
               4. TIE-BREAK: maximise MODULE SIZE
               5. then minimise QR version
plate:         85 × 85 mm, outer margin 3 mm ⇒ 79 mm usable
quiet zone:    4 modules per side, per symbol
legend:        6 lines reserved on plate 1 (25.5 mm at 4.25 mm pitch),
               1 line on every later plate
```

**Both R0 corrections MUST be carried, because they are easy to lose:** tiling is
`across × rows`, **not** `k × k`; and the objective must be a **total order**
breaking toward the **largest** module — the original omitted module size and used
strict `<` against a loop ascending from 0.30 mm, so ties broke toward the
**smallest, least legible** symbol. **41 configurations tie** once the floor lifts.

### 4.6 The plate table must be regenerated

It measures **PSBTs**; this design carries **signed transactions** (53–91% of PSBT
size). It corrects for a **49-bit** header; the ruled header is **55 bits**
(F-242). And `SPEC_mt_qr_DEFERRED.md` §10.14's **font-metric correction** is
already owed. One job, three
inputs. **Until then no plate count here is load-bearing** beyond §4.1's
order-of-magnitude comparison.

### 4.7 Module size

**0.60 mm (two strokes) is the default and what the device suggests.** 0.30 mm is
**optically unvalidated** — a hardware question, and the font work's two-stroke
minimum was for *glyphs*, where a solid square genuinely differs. **Design
against 0.60 mm until the test plate exists.**

---

## 5. Refusals

Generated by the walk, not imagined. **Every refusal gets a test, and every
refusal test must go RED when its check is removed** — `mt` has this machinery
(`refusals.toml`, `check-refusal-coverage.sh`, `mutate-refusals.sh`, 30/30 red);
the fork side needs its equivalent.

| # | refusal | why | §
| --- | --- | --- | --- |
| R1 | a `tx:` record whose body is not lowercase hex | else it falls through to free text and becomes a plate | 2.1 |
| R2 | **a `tx:` record on argv** | argv is world-readable via `/proc` and lands in shell history; this material is bearer | 2.1 |
| R3 | `mt encode --record` with neither `--raw` nor `--chunks` | no default, and the refusal teaches | 2.2 |
| R4 | a payload carrying **both** raw and chunks | §2.2 is XOR; both means it was built by something that does not know this format | 2.2 |
| R5 | a chunk set whose per-chunk BCH checksums do not hold *(proposed)* | catches garbage before ~21 min/plate of scrap, without a decoder | 2.2 |
| R6 | a section exceeding `MaxSectionLen` — **naming the transport**, since NFC's bound is lower | 2.3 |
| R7 | **empty stdin** to `me sysw pack` | must join the existing exit-2 path, not bypass it | 1.1 |
| R8 | a world-readable output destination, unless `--allow-world-readable` | 2.5 |
| R9 | a transaction the parser rejects | 3.3 |
| R10 | two identical txids in one payload | the same transaction packed twice — collapse or refuse, never two picker entries | 3.5 |
| R11 | *Engrave Transaction* on a payload holding no transaction — **naming the fix** | backstop to §3.3 | 3.3 |

**A guard downstream of the parser has already lost.** `mt`'s §8.2f was bypassed
by the invocation it existed to refuse, because clap rejected the positional
argument first — **and clap's error echoed the bearer transaction**. Every refusal
above must be checked against *where in the pipeline it actually runs*.

**And every guard must be tested against its NEAREST LEGITIMATE INPUT.** Five
fixes in the `mt` cycle broke on the near miss. **Before committing any fold that
adds or widens a guard: run the hostile input (must be caught) AND the nearest
legitimate one (must pass), and keep both as tests.**

---

## 6. Sequencing

| | where | what |
| --- | --- | --- |
| **S0** | this repo | **Cut the test plate.** QR blocks at 0.3 / 0.45 / 0.6 / 0.9 mm, plus one raw-octet and one base45 symbol, scanned off brushed steel **with an external scanner** (§3.7). ~2 s per cut. Resolves module size **and** the encoding parameter |
| **P1** | `me` (Rust) | `ClassTransaction`, the framed record, stdin, content-based sealing, output-mode refusal, `MaxSectionLen` → 32,734 — **with vectors** |
| **P2** | `mt` (Rust) | `mt encode --record --raw\|--chunks`; **`mt inspect` gains a raw-transaction subject** (finding O); mode-aware output refusal |
| **P3** | fork (Go) | Port P1, provenance-pinned |
| **P4** | fork | The payload menu (§3.3) and the program (§3.4–3.7) |
| **P5** | fork | The plate: search, legend-last, test-the-plate, plate count (§4) |
| **P6** | both | Journeys and refusal coverage (§5) |

**S0 first is the closure rule applied rather than quoted.** Two of this design's
gates are hypotheses and one is two seconds of machine time.

**Finding O is real new scope.** `mt inspect`, `mt verify` and `mt decode` all
take `mt1` strings — so **no `mt` verb can read a default plate.** For
*broadcasting* that is F-234 working as designed (raw bytes go straight into
`bitcoin-cli`). The gap is **inspection**, and §4.3 makes it mandatory.

**Not in this sequence: F-244.** Critical, pre-existing, affects seeds today, and
must not wait.

---

## 7. What must be true to close

- **0C / 0I** under the R0 loop, over lenses enumerated up front. *Closure is
  lens-closure* — not "a round came back clean".
- **The mode-segmentation gate is green.** Any QR sizing MUST assert measured v40
  capacity against **numeric 7089 / alnum 4296 / byte 2953 at L**. An all-`0x41`
  payload once measured *alphanumeric* capacity while claiming byte; a mixed one
  read **6.6% low**. Every wrong number looked plausible; only this gate caught them.
- **The test plate is cut and read** (S0).
- **`check-provenance.sh` green** across both repos. **Not in CI** — it needs a
  second repository — so it will not catch itself.
- **Refusal coverage is a bijection, and every refusal test goes red without its
  check** (§5).
- **The plate table is regenerated** (§4.6).
- **Both pipeline invariants are asserted as pipeline properties** (§1.1).

---

## 8. Ruled, and not to be re-litigated

| ruling | source |
| --- | --- |
| Every QR carries the STANDARD form, never a codex32 string | operator 2026-08-22, F-234 |
| The device comprehends before it cuts | operator, brainstorm |
| Plate default is QR + legend; text is optional | operator |
| Payload carries raw tx **XOR** chunks | operator |
| `mt` emits the record, `me` packs the container | operator |
| No new secrecy class for transactions | operator |
| `MaxSectionLen` → 32,734 for flash; NFC keeps 8191 | operator |
| The QR's byte ENCODING stays a parameter until the test plate | operator, F-243 |
| The journey walk is the review of this spec | operator |
| `me sysw pack` gains stdin | walk A |
| A `tx:` record on argv is refused | walk B |
| No `--record` default; the refusal teaches | walk C |
| **Chunks are engraved verbatim — no `mt1` decoder in v1** | walk C |
| World-readable output refused + override, across `me` **and** `mt` | walk E |
| Sealing decided by content | walk F |
| Overwriting the region is intended — it is a **courier, not a vault** | walk H |
| The device names `me sysw show` under the digest | walk I |
| The txid is for recognition and never claimed as proof | walk K |
| Show a total, allow skip | walk L |
| The total is never spelled as a destination amount | walk M |
| The device says "test the plate"; it never tests it | walk N |
| `mt inspect` gains a raw-transaction subject | walk O |
| **The carousel is payload-independent; applicability is the payload menu's** | walk P |
| The payload menu appears right after a successful load | walk P |
| The picker is keyed on the txid; the prefix never verifies | walk Q |
| Legend cut last; incomplete plates discarded; no resume | walk R |

---

## 9. Open, and owned

| # | open question | owner |
| --- | --- | --- |
| O1 | the QR's byte encoding | S0's test plate (F-243) |
| O2 | module size below 0.60 mm | S0's test plate (F-234) |
| O3 | **per-chunk BCH checksum before cutting** — proposed, not ruled | §2.2 |
| O4 | the network the address row renders under | F-235's unresolved half |
| O5 | **`validateMdmk`'s four callers** engrave an `md1`/`mk1` codex32 string as QR content — a live F-234 violation | **NOT this spec.** For an `md1` card the "standard form" is not obvious the way transaction bytes are |
| O6 | multi-symbol recovery without `mt`'s reader | `SPEC_mt_qr_DEFERRED.md:169` |
| O7 | applicability predicates for the **other ten** programs (§3.2 builds the mechanism plus this one) | follow-up |
| O8 | **Journey B — recovery.** Someone finds the plate years later. Not yet walked | next walk |
| O9 | the documented `picotool` line stops before the move to 20V/28V power, so a correct payload reads as a failed one | walk G, documentation |
| O10 | the **courier model** is nowhere written down (§ walk H) | documentation |

---

## 10. Provenance of the numbers

Every measured figure was resolved against its source, because **three facts
turned out to be stale in the process** — the 64-chunk cap (retracted; `mt1` uses
15 bits, 32,768 chunks), F-234's chunk counts (~13% low), and the `~96`-character
chunk (91). See F-241, F-242, F-243, and commits `d6c735a` / `0c0d11e`.

- `sysw` constants, scan buffer, drivers, display — the fork at `a91df84`
- pathological sizes — `design/measurements/RESULTS_2026-08-22.txt`
- QR density — `design/measurements/RESULTS_qr_modes_2026-08-22.txt`
- chunk rule and header — `design/SPEC_mt_v0_1.md` §3
- QR search and geometry — `design/SPEC_mt_qr_DEFERRED.md` §4

> **A caution on two of those files.** `RESULTS_2026-08-22.txt` and
> `RESULTS_rcw_2026-08-22.txt` mark rows `fits`/`OVER` against `ch(n) <= 64`
> (`signed.rs:209`, `rcw.rs:189`) — **the retracted 64-chunk cap**. Under `mt1`'s
> real ceiling **everything in both files fits**. Byte and chunk counts are sound;
> **the verdict column is not. Do not cite it.**
