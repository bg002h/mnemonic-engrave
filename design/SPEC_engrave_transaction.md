# SPEC — Engrave a Transaction (SeedHammer II program)

**Status:** DRAFT. **R0 round 0 is FOLDED, NOT CLOSED.** The gate stays open
until a re-review returns 0C/0I; **no code before that.**

**Two reviews have run, and both are recorded:**

| review | result | where |
| --- | --- | --- |
| the **operator journey walk** — this spec's review by ruling | 18 findings (A–R), all ruled | `JOURNEY_WALK_engrave_transaction.md` |
| **R0 round 0**, adversarial, opus | **3 Critical / 8 Important / 4 Minor** | `agent-reports/R0-engrave-transaction-round0-adversarial.md`, persisted verbatim at `caa90cb` **before** any of it was folded |
| **R0 round 1**, fold-check, sonnet | **12 FIXED / 3 PARTIAL / 0 NOT FIXED**, plus **0C / 1I / 2M** the round-0 fold itself introduced | `agent-reports/R0-engrave-transaction-round1-foldcheck.md`, persisted at `321cba6` |
| **R0 round 2**, fold-check + **implementability**, opus | **4 FIXED / 2 PARTIAL**, plus **0 Critical / 7 Important / 3 Minor** — the first round with **no Criticals** | `agent-reports/R0-engrave-transaction-round2-foldcheck-implementability.md`, persisted at `31dbdff` |
| **architect**, fable, operator-routed | decided **O11** and **O14**, the two the controller declined to guess | `agent-reports/ARCHITECT-o11-o14.md`, persisted at `eca34c1` |

**The walk made this spec smaller; R0 made it truer.** The walk removed work —
chunks engrave **verbatim**, so v1 needs no `mt1` decoder (§2.2). R0 removed
*false claims*: three sentences this spec asserted were measurably wrong
(*"nothing in the carousel changes"* — §3.1a; *"a pipe has no file mode"* —
§2.5; *"the compare screen names no command"* — §3.2), and two more described
reuse that does not exist (§2.2a, §4.4a).

**Round 2 changed what this spec KNOWS, not only what it says.** Its
implementability lens **refuted** something the document had recorded as merely
unverified: the vendored QR library has **no Structured Append**, so C1's ruling
— the operator's own — had no mechanism (§4.2a). And it found the largest gap in
the document: **`ClassTransaction`'s wire layout is defined nowhere** (§2.1b),
while four other sections read it. Both are invisible to a reader who is not
implementing, which is the entire argument for running that lens.

**The recurring lesson, now three rounds deep, is about AUDITING rather than
code.** Round 0's I2 named ONE program-keyed switch; the fold enumerated that one
site carefully and stopped. Round 1 grepped the class and found **three**, two
failing *silently*. Round 2 then found a **fourth** — because the grep meant to
close the class searched for *"switches on `program`"*, and the fourth switches
on a scanned object's **type**. **Fixing the instance a finding names, and not
the class, is how the next instance survives another round.** §3.1a now states
the *rule* rather than only the list.

**Where a section carries a `####` sub-heading in SHOUTING CAPS, that is an R0
finding folded in place.** They are left visible rather than smoothed away,
because each one is a claim this document previously made and a future reader is
entitled to know which sentences have already been wrong.

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

**The two transports FORK EARLY and never rejoin** — §1.2. A container goes to
flash; a bare record goes on a tag. They enter the device by different doors and
arrive with different guarantees.

```
tx.final.psbt ─▶ mt encode --record --raw|--chunks ─▶ tx: record
                        (mnemonic-transaction)            │
                          ┌───────────────────────────────┴───────────┐
                          ▼                                           ▼
              me sysw pack (stdin or --in)                    write to an NFC tag
                 (mnemonic-engrave)                                   │
                          │  a CONTAINER                              │  a RECORD
                          ▼                                           ▼
              picotool --region 0x10D00000                     gui/scan.go
                          │                                           │
                          ▼                                           ▼
              syswLoadFlow ─▶ digest compare ─▶              engraveObjectFlow
              payload menu                                    (txScan case)
                          └─────────────────┬─────────────────────────┘
                                            ▼
                          comprehend ─▶ confirm ─▶ cut   (§3.4–§3.7)
```

**What the NFC path does NOT get:** the identity digest compare (§3.2) and the
payload menu (§3.3). There is no header to hash and nothing is loaded into the
session. Its provenance rests on `syswSourceAccept` naming the source, and
nothing else.

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

### 1.1 The join is stdin — an earlier draft was wrong about this

The first draft said the two tools *"compose over a pipe."* **They do not**:
`me sysw pack` took `--in <FILE>` and argv only. Measured:

```
$ printf 'text:6869\n' | me sysw pack --no-passphrase
me: no records: pass them on argv or with --in     (exit 2, stdout empty)
```

**RULED: `me sysw pack` gains a stdin path**, so the natural command works.

**Two invariants this pipeline depends on, and both MUST be stated and tested
rather than inherited by luck.** `fish` reports a pipeline's status as the *last*
command's, so an upstream failure is otherwise invisible (`false | true` →
`status=0`, `pipestatus=1 0`):

1. **`mt` contributes NOTHING to stdout on any failure path.** Measured today:
   `mt encode --in bad.hex` → exit 1, **stdout 0 bytes**.
2. **`me sysw pack` refuses empty stdin** rather than packing an empty container.

### 1.2 THE TWO TRANSPORTS DO NOT DELIVER THE SAME THING

**R0 round 0, I4.** An earlier draft's diagram showed `picotool` and `NFC tag` as
two routes for one container. **There is no NFC reader for a `sysw` container.**
`sysw.Reader` has exactly two implementations — `FileReader` (host) and
`XIPReader` (flash XIP, `sysw/read_tinygo.go`) — and NFC arrives through a
different door entirely, `gui/scan.go`, which parses **records**, not containers.

| | picotool → `0x10D00000` | NFC tag |
| --- | --- | --- |
| carries | a **container** | a **record** |
| framed by | `MNEMSYSW` header, `pub_len`/`ct_len` | the `tx:` prefix alone |
| identity digest to compare? | **yes** (§3.2) | **no — there is no header to hash** |
| payload menu (§3.3)? | **yes** | **no — nothing is loaded into the session** |
| sealing (§2.4)? | possible | **not applicable** |
| enters via | `syswLoadFlow` | `engraveObjectFlow` (§3.3's `txScan` case) |

**Consequences the spec must carry rather than gloss:**

- **The NFC path has no digest compare**, so its provenance rests on
  `syswSourceAccept` naming the source (F3) and nothing else.
- **§2.3's "which transports its output fits" is nearly meaningless as written**
  for a container, because a container has only one transport. What `me sysw
  pack` should state is the **section-cap** fact; what `mt encode --record`
  should state is whether its record fits an **NFC tag**, which is `gui/scan.go`'s
  8 KB buffer and not `MaxSectionLen`.
- **R6 as first drafted would have `me` print "fits NFC" for a container that can
  never travel that way.**

## 2. The container

### 2.1 One framed record under `tx:`

`ClassTransaction` is one record carrying the transaction (or its chunks) **and**
the legend fields `mt encode` already computes. One record, not siblings, so the
legend stays bound to what it describes.

**No new secrecy class.** A transaction rides beside `ClassMDMK` and
`ClassFreeText`.

**A `tx:` record on argv is REFUSED.** `mt` refuses a transaction as an argument
because *"an argument lands in shell history and in `ps` for every user on the
machine, and this material is bearer."* `me sysw pack` currently says only
*"**prefer** `--in` for anything real"*. **Prefer is not enough for a bearer
instrument.**

#### 2.1a A RESERVED PREFIX IS NOT A GUARD — it is a route to free text

**R0 round 0, C3.** An earlier draft of this section said a `tx:` record *"can
never fall through to free text and become a plate"*, citing `gui/scan.go:56-80`.
**That is false, and the citation is what makes it false.** Read the block:

```go
} else if isSyswEncoded(buf) {
    body, err := sysw.DecodeBody(string(buf))
    if err != nil { return nil, errScanUnknownFormat }   // MALFORMED -> refused
    if bytes.HasPrefix(buf, []byte(sysw.PassPrefix)) { return passScan(body), nil }
    return freeTextScan(body), nil                       // <- EVERYTHING ELSE
}
```

The hex check catches the **malformed** case only. A **well-formed** `tx:` record —
valid lowercase hex, exactly what `mt encode --record` emits — reaches
`gui/scan.go:79`'s default and becomes a `freeTextScan`, which
`engraveObjectFlow` hands to `engraveTextFlowFrom`. **The transaction is engraved
as free text**, bypassing every §3.4–§3.7 guarantee: no parse, no comprehension,
no confirm screen, no txid, no plate-count warning.

**NORMATIVE:** adding `tx:` to `isSyswEncoded` **without** adding a matching
branch beside the `PassPrefix` one is the defect. The branch is the work; the
prefix is not.

> **The general rule, because this is the third time this shape has appeared
> here.** `mt`'s §8.2f was bypassed by the invocation it refused, because the
> arg parser ran first. A guard placed downstream of a dispatcher has already
> lost. **For every refusal in §5, name what runs BEFORE it.**

#### 2.1b THE RECORD'S WIRE LAYOUT IS NOT DEFINED, AND FOUR THINGS DEPEND ON IT

**R0 round 2, I5.** `ClassTransaction` is named three times in this document and
**defined nowhere**; `TxPrefix` does not exist in the fork. Yet:

| depends on the framing | why |
| --- | --- |
| **P1's "with vectors"** | you cannot write a test vector for a format you have not stated |
| **R4′** (both forms in one record) | it must be able to *see* both forms to refuse them |
| **R15** (the carried-txid cross-check) | it reads the carried txid out of the record |
| **§3.4's asserted column** | `TO` and the fee are read out of the record's legend fields |

And **Rust (`me`) and Go (fork `sysw/`) must agree byte-for-byte** — the
Rust-primary rule makes `me` normative, but "normative" means nothing until the
layout is written down.

**NORMATIVE: P1 defines it before anything reads it**, stating at minimum: how
the raw form and the chunks form are distinguished, how the optional legend
fields are delimited, what an absent optional field looks like, and — **new,
below** — the carried txid.

**THE CHUNKS FORM CARRIES A MANDATORY 32-BYTE TXID, COMPUTED BY `mt`.**
Architect decision on O11, taken **while this layout is still unfrozen**, which
is the cheapest moment it will ever be available. See §3.6b. **The body is
lowercase hex** (the reserved-prefix rule); what the hex *decodes to* is missing.

> **This is the largest single gap in the document**, and it is invisible to a
> reader who is not implementing: every section that uses the record reads
> naturally, because each assumes a framing somebody else defined.

### 2.2 Raw transaction XOR chunks — and chunks are engraved VERBATIM

**RULED: the payload carries the raw transaction OR its `mt1` chunks, never
both.** This supersedes an earlier *"always decode and compare"* — with one form
present there is nothing to compare.

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

| payload | the device does | it needs |
| --- | --- | --- |
| **raw** | parse -> comprehend -> confirm -> **QR plates** | a transaction parser |
| **chunks** | **engrave verbatim** -> **text plates** | see §2.2a — **not "nothing"** |

> **THIS IS A DELIBERATE EXCEPTION TO §3.4's "COMPREHEND, THEN CUT", STATED IN
> THOSE WORDS SO A LATER READER DOES NOT "FIX" IT.** Comprehension did not
> disappear; it **moved upstream**. `mt encode` built those chunks from a
> transaction the operator inspected on the host.

**RULED (operator 2026-08-24): text+QR is NEVER offered for a transaction.** Each
form produces exactly one kind of plate.

> **What that costs, named rather than left implicit.** F-234's own argument is
> that a plate should carry **both** representations — codex32 text for a human
> with a keyboard, standard-form QR for anyone with a camera, two audiences with
> two failure modes. **The XOR ruling forecloses that for transactions**, because
> the device holds only one form and has neither an encoder nor a decoder to
> derive the other. The ruling stands and the cost is accepted; it is written
> here so nobody rediscovers it as a defect.

#### 2.2a `validateMdmk` CANNOT be reused as-is — it QR-encodes the string

**R0 round 0, C2.** An earlier draft routed chunks through `validateMdmk` and
said the path needs **"nothing new"**. Read what it does
(`gui/gui.go:2512-2530`):

```go
qrc, err := qr.Encode(s, qr.L)                                    // :2514 UNCONDITIONAL
engravings := []textEngraving{
    {"TEXT + QR", backup.Paragraph{Text: s, QR: qrc, ...}},       // :2524 and it is FIRST
    {"TEXT ONLY", backup.Paragraph{Text: s}},
    {"QR ONLY",   backup.Paragraph{QR: qrc, ...}},                // :2526
}
```

`s` is the `mt1` string. So reuse produces **an `mt1` codex32 string inside a QR**
— exactly what F-234 forbids and what §2.2 just ruled out — and offers it
**first**, i.e. as the default variant.

**NORMATIVE:** the chunks path engraves **TEXT ONLY**. It may not call
`validateMdmk` unchanged, and the spec may not describe the chunks form as
needing "nothing new". What it needs is small — a text-only plate builder — but
it is not nothing.

> **This is the same live violation §9 O5 records for `md1`/`mk1` cards, reached
> by a different door.** O5 keeps the *existing* four callers out of Goal 1's
> scope. It cannot also license a **new fifth caller**, which is what reuse
> would create.

**ACCEPTED COST:** a chunks plate is cut with the device making **no claim
whatever** about its content — no destination, no amount, no txid.

**PROPOSED, not ruled:** the device verifies **each chunk's own BCH checksum**
before cutting, so a corrupted payload does not become ~21 minutes per plate of
scrap. **The cost is NOT zero (R0 M1):** `codex32.ValidMD`/`ValidMK` hard-code
the `md`/`mk` HRPs and BCH targets, and `mt1` has its own — so this is a new
`ValidMT` over the shared GF engine, not a call to an existing predicate.

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
2. **NFC does not carry a container at all (§1.2)** — `sysw.Reader` is
   `FileReader` (host) or `XIPReader` (flash), and NFC arrives through
   `gui/scan.go` as a **record**. So the sentence an earlier draft had here —
   *"`me sysw pack` MUST say which transports its output fits"* — is close to
   meaningless: a container has exactly one transport. **What each tool must
   state is different:** `me sysw pack` states the **section cap** it is bound
   by; `mt encode --record` states whether its **record** fits an NFC tag, which
   is `gui/scan.go`'s 8 KB buffer and **not** `MaxSectionLen`. (R0 round 1 caught
   that the round-0 fold corrected §1.2 and left this sentence standing.)
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

**RULED (scope: all of `me`, and `mt` too). IMPLEMENTED — F-244, closed.**

```
stdout is a CHARACTER DEVICE  → nothing. Nothing persists.
stdout is anything else       ┐
or --out FILE                 ┘→ mode grants group or other read?
                                   → REFUSE unless --allow-world-readable

--out additionally creates at 0600 AND fchmods an existing target to 0600
```

**KEYED ON MODE BITS, NOT ON "is it a regular file" — R0 round 0, I3.** An
earlier draft said *"stdout is a pipe/FIFO → nothing. **No file mode exists.**"*
and cited a measurement as proof. **Measured false:**

| destination | mode | leaks? |
| --- | --- | --- |
| anonymous pipe (`\|`) | **0600** | no — the mode test passes it unaided |
| **named FIFO** (`mkfifo`) | **0666** | **yes — verified: a third party reading it received the bytes** |
| `/dev/null` | **0666** | no — character device, persists nothing |
| regular file | umask-dependent | yes when group- or other-readable |

So the exemption belongs to **character devices**, not to FIFOs — **and it is
load-bearing in both directions**: `/dev/null` is 0666, so a mode-only check with
no `S_ISCHR` exemption refuses `me … > /dev/null`.

**The `fchmod` half is load-bearing too.** `write_private`'s documented residual
— *"0o600 binds on CREATE"* — was **measured true**: a pre-existing 0644 target
stays 0644. Creating carefully is not enough.

`mt`'s `redirected_output_warning` is **additive** to this, not replaced: it is
about how long a file *lasts*, this is about who can *read* it.

## 3. The device

### 3.1 The carousel's SHAPE does not change. One enumeration must.

**RULED: all carousel items are shown always**, because every program may
eventually start an NFC transfer — so payload-independence of the carousel is
**correct, not a limitation**.

**Two screens, two questions:**

| | asks | content-dependent? |
| --- | --- | --- |
| the carousel | what can this **machine** do? | **no — every program, always** |
| the payload menu | what can **this payload** do? | **by construction** |

> **M3, corrected.** An earlier draft wrote "**never**" content-dependent. The
> carousel already is: `unlockPayload` is shown only when a Sealed Payload is
> present (`StartScreen.lastNav()`). The accurate claim is narrower and still
> sufficient — **this program adds no new conditionality**, and the existing one
> is untouched.

**The wrap and pager machinery is untouched** — `lastNav`, the compile-time guard
`[qaProgram - unlockPayload]struct{}{}`, `layoutMainPager` and every wrap site.
`engraveTransaction` is **inserted mid-enum** before `loadPayload`, per the house
rule for unconditional programs.

#### 3.1a THREE program-keyed sites are LOCKSTEP, and two of them fail SILENTLY

**R0 round 0 (I2) found one. R0 round 1 found that this section named the wrong
one.** An earlier draft said "nothing in the carousel changes" — false — and the
fold answering it cited `layoutMainPlates` alone, which is the site that fails
**loudly**. Enumerated exhaustively (`grep "switch act.prog|switch m.prog|switch
page" gui/*.go`, non-test):

| site | line | if `engraveTransaction` is missing | fails |
| --- | --- | --- | --- |
| `uiFlow`'s program dispatch | `gui/gui.go:2029` | `obj` stays `nil` → `engraveObjectFlow`'s `default: return false` → **`scanUnknownFormat` forever** | **SILENTLY** |
| `StartScreen.draw`'s title | `gui/gui.go:2186` | **blank title** on the carousel page | **SILENTLY** |
| `layoutMainPlates` | `gui/gui.go:2430` | `panic("invalid page")` | loudly |

`layoutMainPager` also takes a `program` but consumes it **numerically**
(`int(lastNav)+1`), so it carries no case list and no lockstep obligation.

**A FOURTH SITE, AND THE ENUMERATION ABOVE MISSED IT — R0 round 2, I4.**

| site | line | if the new type is missing | fails |
| --- | --- | --- | --- |
| `engraveObjectFlow`'s **type** switch | `gui/gui.go:2487` | `default: return false` → **`scanUnknownFormat`** | **SILENTLY** |

**Why it was missed, and this is the lesson.** The table above was built by
grepping `switch .*prog` — *"switches keyed on a `program`"*. `engraveObjectFlow`
switches on the **scanned object's TYPE**, so it is the same defect class through
a different key, and the grep that was supposed to close the class had its
boundary drawn too narrowly.

**It is on the NFC path**, which §1.2 establishes is a *different door* from
flash. So a `tx:` record arriving on a tag lands here — and §2.1a's ruled `tx:`
branch in `gui/scan.go` is **necessary but not sufficient**: `scan.go` must
produce a `txScan`, **and this switch must have a case for it**, or the record is
recognised, decoded, and then silently discarded.

> **THE CLASS, STATED ONCE, so a fifth instance does not need a fourth round:**
> **every enumeration a new program or a new scanned type must be added to, whose
> default is silent.** Not "switches on `program`". The three above plus this one
> are the current membership; the *rule* is what a future reviewer should re-run,
> not the list.

**NORMATIVE: P4 owns the `txScan` case in `engraveObjectFlow`**, and it is named
here rather than only in §1.2's table cell, because a mechanism mentioned only in
a table is a mechanism nobody schedules.

**The silent one is the worse one, and it is the program's front door.** The
operator selects *Engrave Transaction*, the device says **"unknown format"**, and
nothing crashed, nothing logged, and no test that never pages there would notice.
A panic at least announces itself.

> **The lesson is about the AUDIT, not the code.** The fold that answered I2
> enumerated one site carefully and concluded. **The defect class was "program
> enumerations that must move in lockstep", and the fix was to grep for the
> class** — which is what round 1 did, and what produced the two rows above.
> Fixing the instance a finding names, and not the class, is how the second
> instance survives a round.

**NORMATIVE: all four are lockstep sites, and P4 owns every one of them.**
**None is protected by the enum's compile-time guard**, which asserts only
`unlockPayload`'s position.

> **R0 round 2, I3.** An earlier draft said *"P5 must touch `layoutMainPlates`"*
> here while §6's P4 row already claimed it — a straight contradiction, and the
> dangerous reading is the one in this section: taken that way, **P4 closes green
> on a build that panics the moment the operator pages onto the new entry.**
> Resolved in P4's favour: all four sites are what makes the program *reachable*,
> which is P4's job. P5 is the plate.

| site | owner |
| --- | --- |
| `uiFlow`'s dispatch, `StartScreen.draw`, `layoutMainPlates`, `engraveObjectFlow` | **P4** |

### 3.2 The compare screen names the WRONG command today

The load flow displays the identity digest and asks the operator to compare it.
That number was printed by `me sysw pack` to **stderr, possibly an hour ago, on a
laptop now closed**.

**R0 round 0, I7 — an earlier draft said the screen "names no command". It does,
and it names the risky one.** `gui/sysw_load.go:167-171`:

```go
lines := []string{
    "Compare this against what",
    "`me sysw pack` printed:",        // <- the RE-PACK path
    "",
    sysw.FormatHash(d),
}
```

**Measured:** the digest is over the RECORDS, so it is reproducible, identical
for sealed and unsealed, and independent of the passphrase. Re-running `pack`
does return the same number — **but on a sealed payload it prints a brand-new
12-word passphrase every time**, which reads as *"this is a different
container"*. An operator who acts on that belief and **re-flashes** ends up with
a payload whose passphrase they saw once and did not keep.

**`me sysw show` already exists**, is **read-only**, and prints the same digest
plus `sealed:`, `pub_len:`, `ct_len:`.

**NORMATIVE, and it is a REPLACEMENT, not an addition.** The ruling was phrased
*"put `me sysw show` beneath the digits"*, which — taken additively — leaves the
sentence that sends the operator to re-pack sitting directly above it. **The
`me sysw pack` line must go.**

```
Compare this against
  me sysw show <file>

c679 6b68 b993 bc10
793a 3de2 8b3d 46e0
```

`me sysw pack`'s own digest line carries the same pointer, so the host says it too.

### 3.3 The payload menu

**RULED: it appears immediately after a successful load.**

```
boot → "payload present, load it?" → LOAD → compare digest
     → THE PAYLOAD MENU  ("this payload holds: 1 transaction, 2 seeds")
     → BACK exits to the carousel
```

**R0 round 0, I1 — the cited mechanism is NOT a post-load hook.**
`gui/sysw_unload.go:23` documents `syswPayloadMenu` as *"the `Load Payload`
carousel entry"*, and it is reached only from `uiFlow`'s `case loadPayload`.
**The boot path calls `syswLoadFlow` directly** (`gui/gui.go:2011`) and returns
to the carousel. So `syswPayloadMenu` gaining content-derived entries produces
the menu **only when the operator navigates to Load Payload** — never after the
boot load, which is the moment the ruling names.

**NORMATIVE:** two changes, not one. (a) `syswPayloadMenu` gains content-derived
entries; (b) **the boot path must invoke it on a successful load**, which is a
new call `uiFlow` does not make today.

> **Why this is filed as a defect and not a detail:** P4's gate ("the payload
> menu exists and lists what the payload holds") is satisfiable by (a) alone,
> while the ruled behaviour stays untrue. A gate that can pass while its own
> sentence is false is the shape the closure rule exists to catch.

`sysw.Classify` already computes what a payload holds. **BACK is the exit and
must be**, for the same reason `syswUnloadFlow`'s BACK is choice 0.

The carousel entry stays reachable as a **backstop** and must refuse gracefully,
**naming the FIX**: *"this payload holds no transaction — load one with Load
Payload."*

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
second line but is **asserted** and can collide.

**The picker shows a PREFIX. The prefix DISTINGUISHES; it never VERIFIES.**
`mt`'s own help names where truncation turns dangerous: a 20-bit set id is
*"1 in 1,048,576 by accident, and under a second to construct deliberately."*
The **full txid on screen 2** is what gets compared.

**Two identical txids in one payload is the same transaction packed twice** — a
duplicate to refuse or collapse, never two picker entries.

#### 3.6a THE DEVICE CANNOT DERIVE A TXID FOR A CHUNKS PAYLOAD

**R0 round 0, I5.** A txid is `double-SHA256` over the deserialised transaction
with witnesses stripped. For a **chunks** payload the device has **`mt1` strings
and no decoder** (§2.2, ruled) — so it cannot reassemble the transaction and
cannot compute the txid. **The picker's only key is unavailable in exactly the
form that most needs it**, since a chunks job is 22–202 plates.

Three transactions in chunk form therefore present as three identical rows, and
**R10's duplicate rule is unevaluable** — the device cannot tell two entries
apart well enough to know whether they are duplicates.

**What IS available without a decoder**, and this is the constraint the fix must
work inside:

| candidate | available for chunks? | note |
| --- | --- | --- |
| txid | **no** — needs the transaction | |
| `chunk_set_id` (20 bits, in every chunk header) | **yes** — read off the string | not a txid, and 20 bits is not a comparison key (`mt` says so) |
| the legend fields | **yes** — carried in the record (§2.1) | **asserted**, and may collide |
| record order in the payload | yes | positional, and says nothing |

#### 3.6b RESOLVED — the txid is CARRIED, and 20 bits may only REFUTE it

**O11, decided.** The device cannot *derive* a txid without a decoder, so it is
**carried**: `mt` computes it on the host, where the transaction is in hand, and
the chunks record carries it as a mandatory 32-byte field (§2.1b).

**IT IS DISPLAYED AS CARRIED, NEVER AS DERIVED**, and the distinction is
normative rather than cosmetic. For a **raw** payload the device computes the
txid from bytes it holds and stands behind it (§3.4's *derived* column). For a
**chunks** payload it is repeating a number `mt` told it — which belongs in
§3.4's **asserted** column, beside `TO` and the fee. A screen that renders the
two identically would have the device vouching for a value it cannot check.

**BUT IT IS NOT UNCHECKED, and this is what makes the decision sound.** The
`mt1` chunk header's `chunk_set_id` **is the top 20 bits of the txid** —
`SPEC_mt_v0_1.md:943`: *"the set id **is** the top 20 bits of the reassembled
transaction's txid (§10.13 c), so a wrong guess cannot survive reassembly."* The
device reads that field off **every chunk** with no decoder at all.

**NORMATIVE — R15: 20 bits REFUTES; it never CONFIRMS.**

| observation | verdict |
| --- | --- |
| carried txid's top 20 bits ≠ some chunk's `chunk_set_id` | **REFUSE.** The record is internally inconsistent — a mis-assembled payload, and the device says so |
| they match | **nothing is proven.** 1 in 1,048,576 by accident, *"under a second to construct deliberately"* (`mt`'s own help). It stays **asserted** |

This is precisely the role `mt` refuses 20 bits for — *identification* — and
precisely the role it is sound in: **falsification.** A check that can only fail
honestly is worth having; one that claims to confirm is not.

**R14 IS RETIRED.** It refused any payload holding more than one chunks-form
transaction, which was the honest stopgap while the picker had no key. The picker
now has one, so the refusal's reason is gone and backing up several transactions
in one sitting works again.

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
~21 minutes per plate (F-225).

**MULTI-SYMBOL IS THE COMMON CASE, NOT A CORNER.** At the ruled 0.60 mm module
(§4.7) the largest QR that fits an 85 mm plate is **v26 — 1,367 B at ECC L**
(`RESULTS_qr_physical_max_2026-08-22.txt`); a v40 would be 111 mm wide against
79 mm usable. And the search **prefers** several small symbols over one large one
when that buys ECC: the measured 742 B case resolves to **6 symbols on 2 plates**
(`RESULTS_ecc_selection_2026-08-22.txt`). So multi-symbol begins **below every row
in §2.3's table**.

**Plates, symbols and tiling are three different counts** and the spec must not
conflate them:

- **symbol** = one QR code
- **plate** = one piece of steel; several symbols may be tiled on one (`4 up`)
- and a plate may hold **one** symbol yet still be the *second* plate, because
  the legend reservation pushed it there (measured: 1,130 B is `2 pl, 1 qr`)

### 4.2 What the QR carries, and how several of them reassemble

**The raw transaction bytes** — F-234, not re-litigable.

#### 4.2a Multi-symbol uses QR STRUCTURED APPEND

**R0 round 0, C1 — RULED (operator 2026-08-24).** An earlier draft said the QR
carries *"the raw transaction bytes"*, singular and whole, and said **nothing**
about what each symbol holds when there is more than one, or in what order they
concatenate. A recoverer with nine plates from a drawer had nine anonymous byte
blobs, and §4.3's mandatory post-cut test — *scan the QR, then `mt inspect`* —
would be handed a **truncated transaction on every plate but the last**, reporting
failure on a correct plate.

**Structured Append is QR's own standard for this.** Each symbol carries its
index, the total count, and a parity byte over the whole message; standard
decoders reassemble it themselves. So it keeps F-234's promise **intact for
multi-symbol jobs** — a recoverer with an ordinary scanner still gets the
transaction, with no constellation knowledge — which no bespoke header could do.

**THE 16-SYMBOL CAP CAN BIND, and two earlier drafts of this paragraph argued
otherwise on two different wrong grounds.**

> **R0 round 1, M** — the first draft compared 21.9 KB against *"Bitcoin's
> ~100 KB standardness limit"*, a bound **larger** than the thing it was meant to
> reassure about. An argument that refutes its own conclusion.
>
> **R0 round 2, I2** — the second draft fixed the ceiling and kept the **divisor**
> that made it wrong. `16 × 1,367 B` assumes every symbol is a **max-capacity
> v26**. §4.5's ruled objective ranks **maximise ECC ABOVE minimise symbol
> count**, so the search deliberately produces *many small* symbols: this spec's
> own measured case is **742 B → 6 symbols**, about **124 B each — 11× smaller**
> than the divisor assumed. And `RESULTS_qr_physical_max_2026-08-22.txt`, the
> very file cited for 1,367 B, says *"tiling beyond 16 symbols exceeds QR
> Structured Append's limit… Counts above are unconstrained."*

**NORMATIVE: the search MUST treat 16 symbols as a hard bound**, discarding any
configuration above it, exactly as it discards one that does not fit a plate. The
cap is not a comfortable headroom argument; it is a constraint the objective can
walk into while optimising for ECC.

**And the interaction is the point:** the objective *prefers* small symbols, so
the cap binds **soonest on exactly the configurations the search likes best.**

**GATE 1 IS NOT UNVERIFIED — IT IS REFUTED. R0 round 2, I1.**

`github.com/seedhammer/kortschak-qr v0.3.2` **has no Structured Append.**
Verified: zero occurrences of "structured" anywhere in the module, and
`func Encode(text string, level Level) (*Code, error)` returns **one** `*Code`.
`engrave.QR` takes one code; `backup.Paragraph` holds one.

**So C1's ruling currently has no mechanism** — the third instance this session
of a spec instructing something the code cannot do (C2, I8, and now this).

**It is buildable.** The vendored library exposes a `coding` package, which is
where a Structured Append header (mode 3: index, count, parity) would be written.
That is real work, and **NORMATIVE: a phase must own it.** §6 assigns it to
**P5**.

#### 4.2c RESOLVED — S0 keeps the pair, and the fixture becomes the ORACLE

**O14, decided.** S0 runs **first** and is specified to cut a Structured-Append
pair — an artifact nothing in the tree can produce until P5. The resolution
separates the two questions that were tangled together:

| question | when | how |
| --- | --- | --- |
| **can a scanner read SA off engraved steel?** — physics | **S0, first** | cut **hand-built, standard-conformant** symbols from an independent committed generator (`scripts/gen-sa-fixture.py`, `segno`-based), validated on screen before anything is cut |
| **does OUR encoder emit it correctly?** — software | **P5** | its gate must reproduce the S0 fixture **module-for-module** |

**The obvious objection is that the thing tested is not the thing shipped. The
decision turns that into the point.** Because P5's gate must reproduce the S0
fixture module-for-module, **the fixture is a cross-implementation oracle** — an
independent encoder's output that ours must match exactly. That is strictly
stronger than testing our encoder against itself, which is the shape a prior
cycle's pinned corpus had when it was uniformly wrong.

**It is achievable, not aspirational.** A QR symbol is deterministic given
version, level and mask, and the vendored library takes all three explicitly:
`coding.NewPlan(version Version, level Level, mask Mask)` — `coding/qr.go:484`.
So *"module-for-module"* is a byte comparison, not a judgement call.

**NORMATIVE:**

- **S0 cuts the SA pair** from the independent generator, which is **committed**,
  not run once and discarded — P5's gate needs it later.
- **P5's gate has two halves:** reproduce the S0 fixture module-for-module with
  pinned version/level/mask, **and** decode P5's own rendering with **the same
  scanners used at S0**. Neither half alone is sufficient: the first proves our
  encoder agrees with a standard implementation, the second proves the result
  survives the machine.
- **The physics gate therefore runs FIRST and cheaply; the software gate stays
  machine-checked.** S0 keeps its character — two seconds of machine time — and
  gains no code dependency on P5.

**§4.3's post-cut test is only meaningful once (1) and (2) hold.** Until then a
multi-symbol job cannot be verified by the operator at all.

#### 4.2b The byte ENCODING is still a parameter

Resolved by the test plate (F-243), not here. The argument that previously
settled it **does not bind**: base45 was rejected because its alphabet contains
SPACE and EPD §6.4 forbids interior whitespace **in a `sysw` record** — and in
that architecture the record's string **was** the QR's string. **Here they are
decoupled:** the record carries hex; the QR is generated **on-device** from
parsed bytes and never passes through a record.

**F-243 is more urgent than it was filed as.** Filed as *"can a stranger read this
in 15 years"*, §4.3's ruling makes it **"can the operator complete a mandatory
step, today, every time"** — and raw octets make a **good** plate appear to fail,
because phone scanners mangle bytes >= `0x80`.

### 4.3 After the cut, the device says to TEST — and WHAT it says depends on the symbol count

The `mt` cycle's Critical was exactly this shape — **a silent step**. So the
device speaks. But **the sentence it speaks is not the same on a one-symbol job
and a nine-symbol one**, and an earlier draft had only the first.

**R0 round 1, C1 PARTIAL.** Round 0's C1 was answered in §4.2a by ruling
Structured Append — and this screen's text was left untouched. So C1's exact
walkthrough still ran: the operator scans plate 1 of 2, gets **a fragment**, runs
`mt inspect`, and is told the transaction is bad. **A correct plate reporting
failure**, which is the outcome C1 was filed about.

**SINGLE-SYMBOL JOB — the test is per plate:**

```
PLATE 1 OF 1 — CUT
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
DONE
```

**MULTI-SYMBOL JOB — the test is per JOB, and the device must say so:**

```
PLATE 2 OF 6 — CUT
────────────────────────
Scan this plate now to
check it READS.

Do NOT run mt inspect yet:
this is 1 of 6 symbols and
they only decode together.
────────────────────────
Test the whole set after
plate 6.
────────────────────────
NEXT PLATE
```

#### 4.3a THE BRANCH IS ON SYMBOLS; THE INSTRUCTION IS PER PLATE

**R0 round 2, I6.** The two screens above branch on *symbol* count and are shown
*per plate*, and those are different axes. Three cases fall through, each with a
measured witness in this document:

| case | witness | what the screens above say | what is wrong |
| --- | --- | --- | --- |
| several symbols **on one plate** (tiling) | 742 B → **6 qr on 2 plates, `4 up`** | "scan this plate" | the operator scans **one** of four and moves on; three are unchecked |
| **one** symbol across **two** plates | 1,130 B → **2 pl, 1 qr** (the legend pushed the QR to plate 2) | "scan this plate" | **plate 1 has no QR on it at all**, so the instruction is unfollowable |
| a **chunks/text** job | any `--chunks` payload | neither screen applies | zero symbols, no branch — the operator is told nothing after cutting 22–202 plates, which is §4.3's own silent step |

**NORMATIVE: the per-plate instruction is a function of what is ON THAT PLATE**,
not of the job's symbol count:

- **n symbols on this plate** → *"scan all n on this plate"*, and the count is
  named so the operator knows when they are done
- **no symbol on this plate** (legend-only) → say so, and say nothing about
  scanning
- **a text plate** → the check is not scanning at all; it is that the strings are
  legible and complete, which is the operator's eyes

**And the per-JOB instruction stays keyed to the job**: after the last plate,
scan every symbol and run `mt inspect` once.

**NORMATIVE, and this is what closes C1's walkthrough:**

- **Per plate**, the operator checks only that the symbol **scans** — geometry,
  module size, contrast, the things engraving can get wrong. That is checkable on
  one plate and is the failure mode a cut introduces.
- **Per job**, after the last plate, the operator scans **all** symbols and runs
  `mt inspect` once on the reassembled result.
- **The device MUST NOT ask for `mt inspect` on a partial set.** Doing so reports
  failure on correct work and teaches the operator to stop testing — the exact
  silent-step outcome §4.3 exists to prevent, arrived at from the other side.

**This requires new `mt` scope; see §6.** No `mt` verb can read a raw transaction
today, and reassembling a Structured Append set is the reader §4.2a's gate 2
depends on.

### 4.4 The legend is cut LAST — which is a CHANGE, not the status quo

**RULED: legend last; an incomplete plate is discarded; there is no resume.**

**The legend is the plate's claim about itself.** Cut last, a plate only claims to
be `PLATE 2 OF 3` once it is one. This is §3.5's anti-overclaim discipline applied
to the **artifact**.

> **AN UNSIGNED PLATE IS AN UNFINISHED PLATE.**

Visible at a glance, with no tooling — which matters precisely because the device
has no camera (§3.7) and the operator is the only inspector.

#### 4.4a THE BUILDER EMITS THE LEGEND FIRST TODAY

**R0 round 0, I8.** `Engraving` is `iter.Seq[Command]` (`engrave/engrave.go:55`)
— an **ordered sequence executed in emission order** — and `EngraveText`
(`backup/backup.go:363-396`) emits:

```go
offy := params.I(outerMargin)
centerRow(plate.Title, offy)      // the legend row, FIRST
if plate.Title != "" { offy += fontSize }
...                                // the body, after
```

**So the invariant above is FALSE as shipped.** A plate abandoned at minute 20
already carries `PLATE 1 OF 2` and **looks finished** — the exact failure §4.4
exists to prevent, and an operator taught the rule would sort it into the good
stack.

**It is achievable**, unlike C2: plate *position* comes from the `y` offset, not
from emission order, so legend-last is a **reordering of yields**, not a layout
change. **NORMATIVE: P5 must reorder it, and P5's gate must assert the emission
order** — not merely that a finished plate looks right, since a finished plate
looks identical either way.

**No resume, for a mechanical reason:** re-clamping cannot guarantee the plate
returns to the same origin, and this machine has already produced a
misregistration artefact traced to **Y-axis play from a loose screw**, found only
after four software hypotheses failed. A resumed cut would be offset and would
still look finished.

**The device must SAY to discard it.** It knows it stopped mid-cut, and the
operator is holding twenty minutes of steel they will be tempted to keep.

**The two forms fail differently, and only one fails safe in the steel:** a
partial QR will not scan; a partial **text** plate carries chunks whose checksums
all hold and looks real — it fails safe only because `mt` requires chunks
`1..count`. **That safety lives in the host tool, not the artifact.**

### 4.5 The configuration search

**The DEVICE runs it.** §4 of `SPEC_mt_qr_DEFERRED.md` was written for a host verb
this design does not have; its objective is stated in **plates and minutes**, and
only the device holds `EngraverParams`.

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
```

**Both R0 corrections MUST be carried:** tiling is `across × rows`, **not**
`k × k`; and the objective must be a **total order** breaking toward the
**largest** module — the original omitted module size and used strict `<` against
a loop ascending from 0.30 mm, so ties broke toward the **smallest, least
legible** symbol.

#### 4.5a The legend reservation is a FORMULA, and the fields are PACKED

The reservation was a hard-coded **6 lines / 25.5 mm**, and because a QR is
**square**, losing 25.5 mm of height loses 25.5 mm of width with it:

```
plate usable        79.0 × 79.0 mm = 6241 mm²
with the legend     53.5 × 53.5 mm = 2862 mm²
                    the legend costs 54% of plate 1's AREA
at 0.60 mm          full plate = v26   with the legend = v16
```

**Six lines came from one field per line, with the 45-character BEARER line
wrapping.** Packing them instead — measured at the **3.0 mm** face, `font/sh`,
44 columns:

| legend | lines | height | QR gets | version at 0.60 mm |
| --- | --- | --- | --- | --- |
| 6, one field per line (the old reservation) | 6 | 25.5 mm | 53.5 mm | **v16** |
| **packed, all five fields** (153 chars) | **4** | **17.0 mm** | 62.0 mm | **v19** |
| **packed, mandatory three** (99 chars) | **3** | **12.8 mm** | 66.2 mm | **v21** |
| none | 0 | 0 | 79.0 mm | v26 |

**A SIXTH FIELD IS MISSING FROM THIS LIST — R0 round 2, I7.** §4.4 makes
`PLATE n OF m` **normative** — it carries the *"an unsigned plate is an unfinished
plate"* invariant — and the table above omits it, so the computed reservation
**under-charges every multi-plate job**, exactly the class that needs it.

**And plate-1-only vs every-plate was never settled.** The original reservation
said *"6 lines on plate 1, **1 line on every later plate** for `PLATE n OF m`"*,
while §4.5's objective note says only that a plate holds "the QR(s) AND the
legend".

**NORMATIVE: the reservation is computed PER PLATE, from that plate's field set
and the face — never hard-coded, and never computed once for the job.**

| plate | fields |
| --- | --- |
| 1 | the five above **plus** `PLATE 1 OF m` |
| 2..m | **`PLATE n OF m` only** |

Two of plate 1's five fields (`FROM WALLET`, `TO`) are **optional**, so a fixed
charge bills every job for rows that may not exist.

> **The 4.25 mm pitch is stated nowhere in this document (R0 round 2, Minor)**,
> and the table above pairs a 44-column face measurement with a reservation
> derived from a different one. **P5 must state the pitch it uses and where it
> comes from**, or "computed" is computed from a number nobody can find.

**3.0 mm IS THE FLOOR, and it is already the hard case.**
`gui/freetext_proof.go:24` calls it *"the smallest rung and the hardest legibility
case"*. Smaller faces would help a great deal — the same packing at 1.5 mm gives
**2 lines, 4.2 mm, v24**, i.e. the legend costing a single version step — **but
no face below 3.0 mm has been tested.** That is an **S0** question, and it costs
one extra line of text on a plate already being cut.

**And `FORMAT: mt1 codex32` is WRONG on a QR plate.** §5 of `SPEC_mt_v0_1.md`
calls that field *"arguably the most important"* because it is what lets a
stranger start — and on a QR plate the content is raw transaction bytes by F-234,
not a codex32 string. **The field must state what the QR actually carries.**

### 4.6 The plate table must be regenerated — and it measures the wrong family

It measures **PSBTs**; this design carries **signed transactions** (53–91% of PSBT
size). It corrects for a **49-bit** chunk header; the ruled header is **55 bits**
(F-242). And `SPEC_mt_qr_DEFERRED.md` §10.14's **font-metric correction** is owed.

**R0 round 0, M2 — a fourth input, and it is the largest.** The regeneration must
also carry **§4.5a's packed-and-computed legend reservation**. The existing table
was produced with the hard-coded 6-line / 25.5 mm charge, which costs **54% of
plate 1's area**; packing alone moves 0.60 mm from **v16 to v19**. Every plate
count in that table is therefore high by an amount larger than the other three
corrections combined, and re-running it without this input would produce a second
wrong table.

**Until it is regenerated, no plate count in this spec is load-bearing** beyond
§4.1's order-of-magnitude comparison.

### 4.7 Module size

**0.60 mm (two strokes) is the default and what the device suggests.** 0.30 mm is
**optically unvalidated** — a hardware question, and the font work's two-stroke
minimum was for *glyphs*, where a solid square genuinely differs. **Design
against 0.60 mm until the test plate exists.**

---

## 5. Refusals

Generated by the walk and the R0 round, not imagined. **Every refusal gets a
test, and every refusal test must go RED when its check is removed** — `mt` has
this machinery (`refusals.toml`, `check-refusal-coverage.sh`,
`mutate-refusals.sh`); the fork side needs its equivalent.

| # | refusal | why | § |
| --- | --- | --- | --- |
| R1 | a `tx:` record whose body is not lowercase hex | else it is claimed by a sniffer | 2.1 |
| R2 | **a `tx:` record on argv** | argv is world-readable via `/proc` and lands in shell history; this material is bearer | 2.1 |
| R3 | `mt encode --record` with neither `--raw` nor `--chunks` | no default, and the refusal teaches | 2.2 |
| R4 | **see R4′ below — the first draft of this was wrong** | | 2.2 |
| R5 | a chunk set whose per-chunk BCH checksums do not hold *(proposed)* | catches garbage before ~21 min/plate of scrap | 2.2a |
| R6 | a section exceeding `MaxSectionLen`, stating the **section cap** — **not** "which transports it fits" (§1.2) | 2.3 |
| R7 | **empty stdin** to `me sysw pack` | must join the existing exit-2 path, not bypass it | 1.1 |
| R8 | a world-readable output destination, unless `--allow-world-readable` | 2.5 |
| R9 | a transaction the parser rejects | 3.4 |
| R10 | two identical txids in one payload | the same transaction packed twice — **unevaluable for chunks, see §3.6a** | 3.6 |
| R11 | **see R11′ below** | | 3.3 |
| **R12** | **a well-formed `tx:` record reaching `freeTextScan`** — i.e. `tx:` added to `isSyswEncoded` without its own branch | §2.1a; this is the C3 defect made into a test | 2.1a |
| **R13** | **a multi-symbol QR job when Structured Append is unavailable** | §4.2a's two gates; without them the artifact is unrecoverable and §4.3's test cannot pass | 4.2a |
| ~~R14~~ | **RETIRED.** It refused a payload holding several chunks-form transactions, as the honest stopgap while the picker had no key. §3.6b gives it one | — |
| **R15** | **a chunks record whose carried txid's top 20 bits match no chunk's `chunk_set_id`** | §3.6b: the set id **is** those bits, so a mismatch proves the record is internally inconsistent. It **refutes only** — a match proves nothing | 3.6b |

### R4′ — refusing "both forms" was wrong

**R0 round 0, I6.** The first draft refused *"a payload carrying **both** raw and
chunks"*, reasoning that §2.2's XOR made both-present evidence of broken tooling.
**§3.6 makes it legitimate:** a payload may hold **several transactions**, and
nothing says they must share a form. A sensible operator packs a small transaction
`--raw` (one QR plate) and a large one `--chunks`, and the payload then contains
both — correctly.

**NORMATIVE: the XOR is PER TRANSACTION, not per payload.** R4 refuses a **single
`tx:` record** carrying both forms. A payload holding a raw record and a chunks
record is well-formed.

> **And the first draft's refusal text made it worse.** It blamed the operator's
> tooling — *"built by something that does not know this format"* — for a payload
> their own tooling built correctly. The natural recovery is to re-pack
> everything as chunks, which moves every transaction onto the form where the
> device **makes no claim about content at all** (§2.2). A refusal that pushes
> the operator toward the blinder path is worse than none.

### R11′ — the message is wrong in the case that will be most common

**R0 round 0, M4.** The first draft's message was *"this payload holds no
transaction — load one with Load Payload."* But the carousel entry is
**unconditional** (§3.1), so the **most common** way to reach it is with **no
payload loaded at all** — a fresh boot where the operator declined the offer, or
a machine with no payload region. Telling that operator their payload "holds no
transaction" names a payload that does not exist.

**NORMATIVE: two distinct messages.**

| state | message |
| --- | --- |
| no payload loaded | *"No payload is loaded. Load one with Load Payload."* |
| a payload is loaded, with no transaction in it | *"This payload holds no transaction. It holds: <classes>."* |

Both name the fix (finding I's discipline); only the second may speak about
contents.

### Where a refusal RUNS is part of the refusal

**A guard downstream of the parser has already lost.** `mt`'s §8.2f was bypassed
by the invocation it existed to refuse, because clap rejected the positional
first — **and clap's error echoed the bearer transaction**. C3 is the same shape
inside `gui/scan.go`. **For every refusal above, name what runs before it.**

**And every guard must be tested against its NEAREST LEGITIMATE INPUT.** Six
instances in this cycle now, the most recent two found while fixing F-244:
`me sysw wipe` (a fill image with nothing in it) and **`/dev/null` (mode 0666)**.
**Before committing any fold that adds or widens a guard: run the hostile input
(must be caught) AND the nearest legitimate one (must pass), and keep both.**

## 6. Sequencing

| | where | what |
| --- | --- | --- |
| **S0** | this repo | **Cut the test plate.** QR blocks at 0.3 / 0.45 / 0.6 / 0.9 mm; one raw-octet and one base45 symbol; **a Structured-Append pair from `scripts/gen-sa-fixture.py` (§4.2c) — committed, because P5's gate needs it**; and **one legend line at each candidate face below 3.0 mm**. Read with an **external scanner** (§3.7). ~2 s per cut, no dependency on P5 |
| **P1** | `me` (Rust) | `ClassTransaction`, the framed record, stdin, content-based sealing, `MaxSectionLen` → 32,734 — **with vectors** |
| **P2** | `mt` (Rust) | `mt encode --record --raw\|--chunks`; **`mt inspect` gains a raw-transaction subject**; the record must state whether it fits an **NFC tag** (§1.2), which is `gui/scan.go`'s 8 KB buffer, not `MaxSectionLen` |
| **P3** | fork (Go) | Port P1, provenance-pinned. **Includes the `tx:` branch in `gui/scan.go` (§2.1a) — the prefix without the branch is the C3 defect** |
| **P4** | fork | The payload menu (§3.3) **and the boot-path call that invokes it**; the program (§3.4–3.7); **ALL FOUR lockstep sites (§3.1a)** — `uiFlow`, `StartScreen.draw`, `layoutMainPlates`, `engraveObjectFlow` |
| **P5** | fork | The plate: search (**with the 16-symbol cap as a hard bound, §4.2a**), **QR STRUCTURED APPEND over the vendored `coding` package — it does not exist today (§4.2a, I1)**, **the computed legend reservation (§4.5a)**, **the legend-emission REORDER (§4.4a)**, test-the-plate, plate count |
| **P6** | both | Journeys and refusal coverage (§5) |

**S0 first is the closure rule applied rather than quoted.** Four of this
design's gates are hypotheses and S0 is two seconds of machine time each.

**Not in this sequence: F-244** — closed 2026-08-24, and it did not wait.

## 7. What must be true to close

- **0C / 0I** under the R0 loop, over lenses enumerated up front. *Closure is
  lens-closure* — not "a round came back clean".
- **The mode-segmentation gate is green.** Any QR sizing MUST assert measured v40
  capacity against **numeric 7089 / alnum 4296 / byte 2953 at L**.
- **The test plate is cut and read** (S0).
- **§4.2c's TWO Structured-Append gates are both satisfied**, and they are
  separate: **(a)** a real scanner reassembles the S0 fixture off engraved steel
  (physics, answered at S0), and **(b)** P5's encoder reproduces that fixture
  **module-for-module** with pinned version/level/mask **and** its own rendering
  decodes with the same scanners (software, machine-checked). **Until both hold,
  a multi-symbol QR job may not be cut.**
- **The legend reservation is COMPUTED, not hard-coded** (§4.5a), and the plate
  table is regenerated with it as an input (§4.6).
- **P5's gate asserts the legend's EMISSION ORDER** (§4.4a), not merely that a
  finished plate looks right — a finished plate looks identical either way.
- **`check-provenance.sh` green** across both repos. **Not in CI.**
- **Refusal coverage is a bijection, and every refusal test goes red without its
  check** (§5).
- **The carried txid and R15 are implemented** (§3.6b), and **the chunks picker
  renders the txid in the ASSERTED voice, not the derived one** (§3.4). O11 is
  resolved; R14 is retired; a test must show a chunks-form txid is not presented
  as though the device computed it.
- **All THREE program-keyed lockstep sites carry the new program** (§3.1a), and
  the two that fail **silently** are asserted by test — a panic announces itself,
  `scanUnknownFormat` does not.
- **Both pipeline invariants are asserted as pipeline properties** (§1.1).

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
| **Text+QR is never offered for a transaction** | operator 2026-08-24 |
| **Multi-symbol QR uses QR Structured Append** | operator 2026-08-24, R0 C1 |
| **The legend is packed and its reservation computed; 3.0 mm is the tested floor** | operator 2026-08-24 |

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
| O7 | applicability predicates for the **other ten** programs | follow-up |
| ~~O11~~ | **RESOLVED (§3.6b)** — the txid is **carried** by the record, shown in the **asserted** voice, and cross-checked against `chunk_set_id`, which **is** its top 20 bits. R15 refutes; it never confirms. R14 retired | closed → P1 + P4 |
| ~~O12~~ | **ANSWERED, and the answer is NO** — `kortschak-qr v0.3.2` has no Structured Append (verified: zero occurrences; `Encode` returns one `*Code`). Buildable over its `coding` package; **P5 owns it**. See O14 for what that does to S0's order | **closed → P5 + O14** |
| **O13** | a legend face **below 3.0 mm** — untested, and worth ~5 QR versions (§4.5a) | S0 |
| ~~O14~~ | **RESOLVED (§4.2c)** — S0 cuts the pair from an independent committed generator, so the **physics** gate runs first and cheap; P5 reproduces that fixture **module-for-module**, so the **software** gate is a byte comparison against a cross-implementation oracle | closed → S0 + P5 |
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
