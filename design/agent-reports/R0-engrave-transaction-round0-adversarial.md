# R0 round-0 adversarial review — SPEC_engrave_transaction.md

**Agent:** R0 adversarial, round 0. **Date:** 2026-08-24.
**Artifact under review:** `design/SPEC_engrave_transaction.md` (708 lines).
**Brief:** construct a concrete failure the spec permits. Not an assessment.

**Method note.** Every finding below was checked against source, not against the
spec's description of source. Fork pinned at the working tree of
`/scratch/code/shibboleth/seedhammer`; `me` at `crates/me-cli/`; `mt` at
`/scratch/code/shibboleth/mnemonic-transaction`. The §8 rulings were treated as
decisions; where a ruling appears below it is because the spec fails to handle a
consequence of it, never because a different ruling would be better.

---

## [C1] A multi-symbol QR backup is unrecoverable as specified, and §4.3's mandatory post-cut test cannot pass on any plate but the last

**Severity:** Critical

**Where:** §4.2, §4.3, §4.5, §6 (P2, P5), §7, §9 O6. Cross-reference
`design/SPEC_mt_qr_DEFERRED.md` §10 item 2 (lines ~157-176).

**The failure, concretely:**

Take the spec's own measured case, §2.3's table row `5/2`: a signed transaction
of **4,080 bytes**. The maximum a single QR symbol can hold in byte mode is
**2,953 bytes** (version 40, ECC L) — the exact figure §7 names in its
mode-segmentation gate. So this transaction requires **at least two symbols**,
independently of module size, ECC choice or tiling. §4.1 states the 10-in/2-out
case is **~9-11 QR plates**. Multi-symbol is not a corner; it is every
transaction above ~3 KB, and at the ruled 0.60 mm module size (§4.7) it starts far
lower, because a v40 symbol at 0.60 mm is 111 mm wide against 79 mm of usable
plate (§4.5).

Now walk it:

1. Operator engraves plate 1 of 2. §4.4 cuts the legend last, so the plate is
   finished and reads `PLATE 1 OF 2`.
2. The device shows §4.3's screen verbatim: *"TEST IT NOW, before you leave the
   machine. Scan the QR, then run `mt inspect` on what you get."*
3. The operator scans plate 1. They receive **the first N bytes of a
   transaction** — a fragment.
4. `mt inspect` — even after P2 gives it "a raw-transaction subject" (§6, finding
   O) — is handed a truncated transaction. It refuses.

The operator has a correct plate and a mandatory test that reports failure. Two
outcomes, both bad:

- They believe plate 1 is defective and re-cut it — **~21 minutes of steel
  discarded per attempt (F-225)**, and the next attempt fails identically.
- Or they learn the test always fails on multi-plate jobs and stop running it.
  The device has **no camera** (§3.7) and cannot check its own work, so from that
  point nothing inspects any plate at all — which is the exact silent-step defect
  §4.3 exists to close.

And the artifact itself is worse than the test. **Nothing in §4 specifies what
each symbol contains or in what order the symbols concatenate.** §4.2 says the
QR carries *"the raw transaction bytes"* — singular, whole. §4.5's search space
selects `rectangular tiling (across × rows)` and an objective that minimises
symbol count, and says nothing about framing, sequence numbers, or a manifest.
Compare the chunks form, which carries a 55-bit header with `chunk_set_id`,
`count` and `index` in **every chunk** (`mt-codec/src/consts.rs:44-52`). The raw
QR form carries none of that. A recoverer holding nine plates from a drawer has
nine anonymous byte blobs.

**Why the spec permits it:**

`SPEC_mt_qr_DEFERRED.md` §10 item 2 already ruled the precondition, in the
operator's own decision, and states it as a gate:

> *"F-234's promise — that a recoverer with none of our tools can still read the
> plate — now holds only for artifacts that fit **one** symbol. Multi-symbol
> recovery requires `mt`'s reader. The next subversion's verb is therefore not a
> convenience; it is what keeps multi-plate transactions recoverable at all, and
> **it should be specified before anyone engraves a multi-symbol artifact.**"*

`SPEC_engrave_transaction.md` carries no such gate. §6 sequences P5 as *"The
plate: search, legend-last, test-the-plate, plate count (§4)"* with no
prerequisite. §7's "What must be true to close" lists seven items and the reader
is not among them. §9 parks it as **O6 — "multi-symbol recovery without `mt`'s
reader", owner `SPEC_mt_qr_DEFERRED.md:169"`** — i.e. owned by a *deferred*
document and by no phase in this sequence. Under the constellation's follow-up
rule an item with no owning phase batches to the end, so P5 can close green, the
device can cut a nine-plate backup, and the reader that makes those nine plates
mean anything is unwritten.

The sentence that is missing is one that binds P5 to the multi-symbol framing and
reader — or that refuses to cut a multi-symbol QR job until they exist.

**Confidence:** high. The 2,953-byte v40-L byte-mode ceiling is named by the spec
itself (§7); the 4,080-byte case is the spec's own measured row (§2.3); the
absence of framing is the absence of text in §4.2/§4.5; O6 and
`SPEC_mt_qr_DEFERRED.md` §10.2 are quoted above. What would settle the remaining
question — whether the operator is expected to concatenate by legend order — is a
sentence in §4.2 stating the split, which does not exist.

---

## [C2] §2.2 routes chunks through `validateMdmk`, which encodes the string as QR unconditionally and offers "TEXT + QR" and "QR ONLY" — the artifact §2.2 and F-234 forbid

**Severity:** Critical

**Where:** §2.2, §9 O5. Code: `gui/gui.go:2512-2530` (`validateMdmk`), especially
`gui/gui.go:2514`, `:2524`, `:2526`.

**The failure, concretely:**

```
mt encode --record --chunks < tx.final.psbt | me sysw pack --region --out region.bin
picotool load --verify -t bin -o 0x10D00000 region.bin
```

Device: load → compare digest → payload menu → *Engrave Transaction*. §2.2 rules
the device does exactly one thing with this payload:

> *"Chunks take the existing `mdmkText` / `validateMdmk` path already used for
> `md1`/`mk1` cards."*

and the table beside it says the chunks form needs **"nothing new"**.

`validateMdmk` is that path, and here is what it actually does:

```go
func validateMdmk(pl Platform, s, title, footer string) ([]string, []Plate, error) {
	params := pl.EngraverParams()
	qrc, err := qr.Encode(s, qr.L)          // gui/gui.go:2514 — UNCONDITIONAL
	...
	engravings := []textEngraving{
		{"TEXT + QR", backup.Paragraph{Text: s, QR: qrc, QRScale: qrScale}},   // :2524
		{"TEXT ONLY", backup.Paragraph{Text: s}},                              // :2525
		{"QR ONLY", backup.Paragraph{QR: qrc, QRScale: qrScale}},              // :2526
	}
```

It QR-encodes the string before it does anything else, and it offers the operator
three variants of which **two carry a QR**, with `TEXT + QR` **first**. The
operator selecting the first option — which is what every `md1`/`mk1` card
already does — cuts a plate whose QR content is an `mt1` chunk string.

If they select `QR ONLY`, they cut 22 plates (the §2.3 1-in/1-out chunk count)
each holding a QR of an `mt1` chunk and **no human-readable text at all**. That
artifact is readable by `mt` and by nothing else: F-234's stated promise — a
stranger with a camera and standard tools recovers the transaction — is void, and
the device has no camera to notice (§3.7). ~21 minutes per plate, ~7.7 hours of
machine time for the smallest chunk job in the spec's own table, and up to 202
plates for the pathological one (§4.1).

**Why the spec permits it:**

§2.2 states the prohibition and the mechanism in adjacent paragraphs and they
contradict each other:

> *"Chunks take the existing `mdmkText` / `validateMdmk` path already used for
> `md1`/`mk1` cards."*

> *"**Chunks means text plates ONLY, never QR.** F-234 forbids an `mt1` string as
> QR content, and without a decoder the device cannot recover the bytes."*

And §9 records the behaviour as already known, while ruling it out of scope:

> **O5** — *"**`validateMdmk`'s four callers** engrave an `md1`/`mk1` codex32
> string as QR content — a live F-234 violation … **NOT this spec.**"*

O5 is correct that fixing the `md1`/`mk1` callers is not this spec's job. But
§2.2 **adds a fifth caller** on the same function and then asserts the opposite
property for it. Nothing in §2.2, §5 or §6 (P4/P5) requires a text-only variant
of the path, and the "nothing new" claim is what tells an implementer not to
build one.

Second, smaller, same sentence: `mdmkText` is not reachable for an `mt1` string
in the first place. Its only constructors are `gui/scan.go:70-73` and
`gui/codex32_polish.go:273,277`, all gated on `codex32.ValidMD || codex32.ValidMK`
(`codex32/mdmk.go:137,152`), and both hard-code the HRPs `"md"` and `"mk"` with
md/mk-specific BCH generators and targets. An `mt1` string fails both.

**Confidence:** high. `validateMdmk`'s three variants and its unconditional
`qr.Encode` are quoted from source above; the two §2.2 sentences are quoted from
the spec.

---

## [C3] Making `tx:` a reserved prefix, as §2.1 requires, routes a well-formed `tx:` record scanned over NFC into `freeTextScan` — the free-text plate §2.1 says can never happen

**Severity:** Critical

**Where:** §2.1, §1 (the pipeline diagram's NFC branch). Code: `gui/scan.go:62-79`
(the block §2.1 cites as `gui/scan.go:56-80`), `gui/scan.go:102-105`
(`isSyswEncoded`), `gui/gui.go:2479` (the `freeTextScan` case).

**The failure, concretely:**

§2.1 rules that `tx:` **inherits the reserved-prefix rule** and cites
`gui/scan.go:56-80` as the mechanism. Doing that means adding `tx:` to
`isSyswEncoded`, which is the function that decides whether a scanned buffer is
matched before the sniffers:

```go
func isSyswEncoded(buf []byte) bool {                              // gui/scan.go:102
	return bytes.HasPrefix(buf, []byte(sysw.TextPrefix)) ||
		bytes.HasPrefix(buf, []byte(sysw.PassPrefix))
}
```

Here is the block it gates, complete:

```go
} else if isSyswEncoded(buf) {                                     // gui/scan.go:62
	body, err := sysw.DecodeBody(string(buf))
	if err != nil {
		return nil, errScanUnknownFormat                    // the MALFORMED case
	}
	if bytes.HasPrefix(buf, []byte(sysw.PassPrefix)) {
		return passScan(body), nil
	}
	return freeTextScan(body), nil                             // gui/scan.go:79 — DEFAULT
}
```

The reserved-prefix rule guards the **malformed** body. A **well-formed** one
falls to line 79, which is not a `text:` branch — it is the *else* for every
reserved prefix that is not `pass:`. So a `tx:` record whose body is valid
lowercase hex returns `freeTextScan(<raw transaction bytes>)`, and
`engraveObjectFlow` dispatches that to Engrave Text:

```go
case freeTextScan:                                                 // gui/gui.go:2479
	engraveTextFlowFrom(ctx, th, string(scan), srcNFC)
```

Concrete sequence: the operator writes a `tx:` record to an NFC tag — the
transport §1's own pipeline diagram draws as one of two branches — and taps it.
The device hex-decodes it and hands ~850 bytes of binary transaction to the free
text engraver. No parse, no txid, no comprehend screen, no confirm screen, no
network row, no anti-overclaim discipline: **every guarantee in §3.4-§3.6 is
bypassed**, because none of them are on this path. The best case is a refusal
somewhere inside the text flow; the case the spec claims is impossible is a plate.

The alternative implementation is no better as a spec matter: if `tx:` is added to
`sysw/record.go` and *not* to `gui/scan.go`, a `tx:` record over NFC falls past
every sniffer to `errScanUnknownFormat` — safe, but then §2.1's citation of
`gui/scan.go:56-80` as the inherited mechanism is simply false, and §1's NFC
branch dead-ends (see I4).

**Why the spec permits it:**

§2.1, in one sentence, states both the mechanism and a guarantee the mechanism
does not provide:

> *"`tx:` inherits the reserved-prefix rule (`sysw/record.go:41-51`,
> `gui/scan.go:56-80`): a body that is not lowercase hex is `ClassUnknown` and
> **refused before any sniffer sees it**, so **it can never fall through to free
> text and become a plate**."*

The clause after the colon is about the not-lowercase-hex case. The clause after
"so" claims a property of the whole prefix. `gui/scan.go:79` is the missing case,
and no sentence anywhere in §2.1, §5 or §6/P3-P4 requires a `tx:` arm in that
switch.

**Confidence:** high on the code path (quoted above, three call sites resolved).
Medium on whether an implementer would add the arm unprompted — Go has no
exhaustive-switch check here and the default branch is a silent success, not a
compile error, which is precisely why the omission survives.

---

## [I1] §3.3's ruled behaviour — "the payload menu appears immediately after a successful load" — is not produced by the change §3.3 specifies, and P4's gate is satisfiable while the ruling stays untrue

**Severity:** Important

**Where:** §3.3, §6 (P4). Code: `gui/sysw_unload.go:34` (the function §3.3 cites),
`gui/gui.go:2011` (the boot load call), `gui/sysw_load.go:25`.

**The failure, concretely:**

§3.3 rules the flow and names the mechanism in the same paragraph:

> **RULED: it appears immediately after a successful load.**
> …
> *"`syswPayloadMenu` exists (`gui/sysw_unload.go:34`) and today offers only
> `LOAD AGAIN` / `UNLOAD`; it gains content-derived entries above them."*

`syswPayloadMenu` is not a post-load hook. It is the **`Load Payload` carousel
entry handler**, and its own doc comment says so:

```go
// syswPayloadMenu is the `Load Payload` carousel entry.                 // :23
func syswPayloadMenu(ctx *Context, th *Colors) {                         // :34
	if ctx.sysw == nil || !ctx.sysw.loaded {
		syswLoadFlow(ctx, th, ctx.Platform.SyswReader(), false)  // it CALLS the load flow
		return
	}
	cs := &ChoiceScreen{ Choices: []string{"LOAD AGAIN", "UNLOAD"} }
	...
}
```

It runs *before* a load, not after one. And the boot path — the one §3.3's own
diagram starts from (`boot → "payload present, load it?" → LOAD → …`) — does not
go through it at all:

```go
syswLoadFlow(ctx, th, ctx.Platform.SyswReader(), true)                   // gui/gui.go:2011
```

An implementer who does exactly what §3.3 says — add content-derived entries to
`syswPayloadMenu` — ships a device where, at boot, the payload loads, the digest
is compared, and control returns to the carousel. **The menu never appears after
a load.** The operator is back at the divergence Thread 3 of the walk was written
to close: they page the carousel guessing which program applies, on a machine
where a wrong selection costs ~21 minutes of steel, and R11's refusal — which
§3.3 insists must be *"a backstop, not the path"* — becomes the only path.

P4's gate in §6 reads *"The payload menu (§3.3) and the program (§3.4-3.7)"*. The
cited edit is done; the gate reads green; the ruled behaviour is untrue.

**Why the spec permits it:** nothing in §3.3 or §6 says `syswLoadFlow` must invoke
the menu on success, or that `gui/gui.go:2011`'s boot call site must change. The
missing sentence is the one that names the *caller*, not the callee.

**Confidence:** high. Both call sites and the function's own comment are quoted.

---

## [I2] "Nothing in the carousel changes" omits `layoutMainPlates`, which panics on any program not in its enumeration — and no compile-time guard catches it

**Severity:** Important

**Where:** §3.1. Code: `gui/gui.go:2429-2437`.

**The failure, concretely:**

§3.1 rules `engraveTransaction` *"inserted mid-enum before `loadPayload`"* and
then enumerates what is unaffected:

> *"**Nothing in the carousel changes** — `lastNav`, the compile-time guard
> `[qaProgram - unlockPayload]struct{}{}`, `layoutMainPager` and every wrap site
> are untouched."*

Four things named. The fifth is this:

```go
func layoutMainPlates(buf *op.Buffer, page program) (op.Op, image.Point) {   // :2429
	switch page {
	case backupWallet, engravePassphrase, engraveText, engraveXpub, engraveBundle,
		engraveSingleSig, engraveMultisig, walletPolicy, loadPayload,
		bip85Derive, unlockPayload:                                  // :2431
		img := assets.Hammer
		o := op.Image(buf, img)
		return o, img.Bounds().Size()
	}
	panic("invalid page")                                                // :2436
}
```

An **explicit list of every program**, with `panic` as the default. Go does not
check switch exhaustiveness, and the compile-time guard §3.1 names
(`[qaProgram - unlockPayload]struct{}{}`) tests only that `qaProgram - unlockPayload
== 1`, which an insertion *before* `loadPayload` preserves. So the build is clean.

Concrete sequence: firmware flashed, operator at the carousel presses the right
arrow to reach the new entry → `layoutMainPlates(buf, engraveTransaction)` →
`panic("invalid page")`. The device is dead until it is reflashed, and the
operator's only signal is a machine that stops.

A second, quieter consequence of the same sentence: `lastNav` and `layoutMainPager`
are untouched *as source text* but not *as values*. `npages := int(lastNav) + 1`
(`gui/gui.go:2445`) grows by one and every program's ordinal shifts. That is the
intended behaviour; "nothing changes" is the wrong way to say it, and it is what
licenses skipping the audit that would have found `layoutMainPlates`.

**Why the spec permits it:** the sentence quoted above is a closed enumeration
presented as complete. There is no instruction anywhere in §3.1 or §6/P4 to add a
case to `layoutMainPlates`.

**Confidence:** high. The switch, its enumeration, and the panic are quoted from
source.

---

## [I3] §2.5's "stdout is a pipe/FIFO → nothing. No file mode exists" is false and machine-checkably so — a named FIFO fstats with a real, world-readable mode

**Severity:** Important

**Where:** §2.5 (the ruled table and the "mechanically verified" paragraph).

**The failure, concretely:**

Measured just now on this machine, `fstat(1)` from inside the process, three
destinations:

```
named FIFO, mkfifo -m 666 :  ISREG=False ISFIFO=True  mode=0666
anonymous pipe (`| cat`)  :  ISREG=False ISFIFO=True  mode=0600
regular file, 0644        :  ISREG=True  ISFIFO=False mode=0644
```

A named FIFO reports `S_ISFIFO` **and** a permission mode of 0666. §2.5 rules the
check does not fire on `S_ISFIFO`, so:

```sh
mkfifo -m 666 /tmp/p
me sysw pack --in tx.rec > /tmp/p &
cat /tmp/p > payload.bin
```

Any other user on the machine can `cat /tmp/p` and take the container while it is
in flight. The material is bearer — a signed transaction under §2.1's own
argument, or, since §2.5's scope is ruled as **all of `me`**, a cleartext BIP-39
mnemonic (`pack_deterministic` moves secret-class records into the *public*
section when unsealed, `crates/me-cli/src/sysw/mod.rs:186-194`). That is the
material class F-244 was Critical for.

The distinction is not merely available, it is in the same `fstat` the ruling
already performs: 0666 versus 0600 separates the two cases exactly.

**Why the spec permits it:**

> *"stdout is a pipe/FIFO    → nothing. **No file mode exists.**"*

and, two paragraphs down, the claim that closes the question:

> *"Mechanically verified, not assumed: a process can `fstat(1)` its own
> redirected stdout and sees `S_ISREG` + mode 0644, and sees `S_ISFIFO` for a
> pipe — **so the check fires exactly where the exposure is.**"*

The measurement was performed on an anonymous pipe and generalised to every
`S_ISFIFO`. A named FIFO is an exposure where the check does not fire, so the
stated guarantee is not met.

**Confidence:** high on the mechanism (measured above, reproducible in three
lines). Medium on operational likelihood — routing a container through a
world-readable named FIFO is not a common operator move. The load-bearing part is
that the spec asserts the guard is exhaustive *and cites a measurement as proof*,
which is what stops anyone re-checking it.

---

## [I4] §1's pipeline diagram and §2.3/R6 assert an NFC transport for a `sysw` container that does not exist — implementing R6 makes `me` tell the operator a container "fits NFC"

**Severity:** Important

**Where:** §1 (the diagram), §2.3 item 2, §5 R6. Code: `sysw/read.go:9-14`,
`gui/gui.go:3399-3405`, `gui/sysw_load.go:25,55`, `gui/scan.go:29-105`.

**The failure, concretely:**

§1's diagram branches `me sysw pack`'s output two ways — `--region` / `picotool` /
`0x10D00000`, **and `NFC tag`** — and both arrows converge on
`SeedHammer: load ─▶ payload menu`.

There is no such path. A `sysw` container reaches the device through exactly one
interface:

```go
type Reader interface {          // sysw/read.go:9
	Probe() bool
	Read() ([]byte, error)
}
```

whose only supplier is `Platform.SyswReader()` — *"returns a reader for the
SYSTEMWIDE region (0x10D00000)"* (`gui/gui.go:3399-3405`) — consumed only by
`syswLoadFlow` (`gui/sysw_load.go:25`, called at `gui/gui.go:2011`,
`gui/sysw_unload.go:36,52`). The NFC path is `gui/scan.go`'s `scanner.Scan`, which
sniffs **individual records** (`text:`/`pass:`, BIP-39, descriptor, codex32,
md1/mk1, address) and has no container branch. A container begins with the eight
bytes `MNEMSYSW` (`sysw/wire.go:20`), matches no sniffer, and returns
`errScanUnknownFormat`.

Concrete sequence: the operator packs a small transaction container, writes it to
an NFC tag because §1 says that is a transport, and taps it. The device says
unknown format. They have a correct container, a spec-documented transport, and no
way to tell a bad tag from a design that never existed — the same
indistinguishable-failure shape the walk classified under finding G.

R6 makes it worse rather than better. §2.3 item 2 rules:

> *"**NFC stays at 8191** — `gui/scan.go`'s buffer bounds it — so a large
> transaction is **picotool-only**, and `me sysw pack` MUST say which transports
> its output fits."*

"so a *large* transaction is picotool-only" carries the implication that a small
one is not. Implementing R6 as written means `me sysw pack` will print an
affirmative "fits NFC" line for every container under the threshold — a claim the
tool makes that is not true at any size.

The 8191 figure is also attached to the wrong object. `gui/scan.go:31` allocates
`8*1024` and overflows at `s.n == len(s.buf)`, i.e. 8191 usable — but that bounds
the **whole scanned message**, and the thing it scans is a record, not a
container. A container's own ceiling is `HeaderLen + pub + ct + TagLen`; a legal
8191-byte section already makes a 8243-byte container, over the scan buffer by 51
bytes, today, before this spec changes anything.

**Why the spec permits it:** the diagram draws the branch; §2.3 item 2 and R6
require a tool to make a transport claim on the strength of it. The missing
sentence is one stating that `me sysw pack`'s output has exactly one transport.

**Confidence:** high. The `Reader` interface has one implementer surface and four
call sites, all resolved above; `gui/scan.go`'s dispatch was read in full.

---

## [I5] The picker's only key is the txid, which the device cannot derive for a chunks payload — so a payload holding several chunk-form transactions offers no way to pick the right one

**Severity:** Important

**Where:** §3.6, §3.3, §2.2, §5 R10.

**The failure, concretely:**

§2.2 rules that for a chunks payload the device *"engraves verbatim"*, needs
*"nothing new"*, has **no `mt1` decoder in v1**, and makes — the spec's words —
*"**no claim whatever** about its content — no destination, no amount, no txid."*

§3.6 then rules the multi-transaction picker:

> *"The picker is keyed on the **txid** — the derived, collision-free identifier,
> and the one that matches `mt inspect` on the host."*

For a chunks payload there is no txid and no way to compute one without the
decoder v1 does not have. Concrete sequence: the operator packs three chunk-form
transactions in one payload — §3.3's payload menu is specified to report exactly
this (*"this payload holds: 3 transactions"*), and §3.6 is titled "Several
transactions in one payload" without qualifying the form. They select *Engrave
Transaction*. The picker has three entries and nothing to put in them. Whatever
the implementer chooses — position, or the asserted `TO` label §3.6 says *"can
collide; three transactions could all read 'cold storage'"* — the operator picks
without a derived discriminator and commits to **22 to 202 plates** at ~21 minutes
each (§2.3 table, §4.1). Getting it wrong costs between 7 and 70 hours of machine
time and a stack of steel for the wrong transaction.

R10 fails on the same gap from the other side: *"two identical txids in one
payload is the same transaction packed twice — a duplicate to refuse or collapse"*
is unevaluable for chunks, so the duplicate-refusal §5 lists as a guard cannot run
on half the payload forms the spec admits.

**Why the spec permits it:** §3.6 states one key and no alternative, and never
scopes itself to the raw form. §2.2 states the device knows nothing about a chunks
record. Neither section mentions the other. The `chunk_set_id` that would
discriminate is in every chunk's 55-bit header (`mt-codec/src/consts.rs:46-52`) and
reading it is decoder work §2.2 rules out of v1.

**Confidence:** high that the gap exists (both sections quoted, and the decoder
exclusion is a §8 ruling). Medium on severity — an implementer will notice the
missing key at build time, but what they invent is unreviewed and the wrong
invention costs a day of engraving.

---

## [I6] R4 refuses a payload holding both forms, which §3.6 makes a legitimate build — and the refusal text tells the operator their tooling is broken

**Severity:** Important

**Where:** §5 R4, §2.2, §3.6.

**The failure, concretely:**

R4:

| # | refusal | why |
| --- | --- | --- |
| R4 | a payload carrying **both** raw and chunks | *"§2.2 is XOR; both means it was built by something that does not know this format"* |

§2.2's XOR is stated over the **payload**: *"the payload carries the raw
transaction OR its `mt1` chunks, never both."* §3.6 rules that one payload may
hold several transactions. The two together forbid a build that is not only
legitimate but sensible: the small transaction as `--raw` (one QR plate, and the
device comprehends and confirms it), the 8 KB one as `--chunks` (text plates,
because 8,067 bytes is ~9-11 QR plates versus a form the operator can read).

```sh
mt encode --record --raw    < small.psbt > a.rec
mt encode --record --chunks < big.psbt   > b.rec
me sysw pack --in a.rec --in b.rec --region --out region.bin      # REFUSED by R4
```

The operator's tooling is correct, `mt` produced both records exactly as §2.2's
teaching refusal instructed, and the pack refuses with a message asserting the
payload *"was built by something that does not know this format."* The
near-miss-legitimate input is caught by a guard aimed at a hostile one, and the
diagnosis points the operator away from the real cause.

The likely recovery is worse than the refusal: re-pack everything as `--chunks`,
because that is the form that always works. That silently moves the operator onto
the path where the device makes **no claim whatever** about the transaction (§2.2)
— no destination, no amount, no txid — for a transaction that would have been
comprehended and confirmed under `--raw`. The guard degrades the safety property
it sits next to.

**Why the spec permits it:** R4 is stated per-payload; §3.6 is stated per-payload;
neither says whether XOR binds the payload or the record, and §2.2's own prose
(*"`ClassTransaction` is one record carrying the transaction (or its chunks)"*,
§2.1) reads as per-record while R4 reads as per-payload.

**Confidence:** high that the two sections conflict. Medium on which reading was
intended — that is precisely what makes it a finding, since the implementer picks.

---

## [I7] §3.2 misstates what the compare screen says today; the ruled change is additive, so the sentence that sends the operator to re-pack survives it

**Severity:** Important

**Where:** §3.2. Code: `gui/sysw_load.go:165-176`.

**The failure, concretely:**

§3.2 states the problem it is fixing:

> *"the screen currently says *"Compare this against what"* **without naming
> what, or how to get it.**"*

The shipped screen is two lines, not one:

```go
lines := []string{
	"Compare this against what",        // gui/sysw_load.go:168
	"`me sysw pack` printed:",          // gui/sysw_load.go:169
	"",
	sysw.FormatHash(d),
}
```

It does name what. It names **`me sysw pack`** — the command finding I of the walk
identified as the risky recovery, the one that *"prints a brand-new 12-word
passphrase every time"* and leads to a re-flash whose passphrase the operator saw
once. The device is not silent; it is actively pointing at the trap.

§3.2's ruling is phrased as an addition:

> *"**RULED: the device prints the command beneath the digit groups.**"*

"Beneath the digit groups" describes appending a line. An implementer who appends
`me sysw show <file>` and leaves lines 168-169 in place ships a screen that names
**both** commands, and `me sysw pack` is still the one at the top, above the
digest, in the position the operator reads first. The failure mode the ruling
exists to prevent — re-pack, see a new passphrase, conclude the container changed,
re-flash, lose the words — is untouched.

§3.2's mock-up does show the replacement (`Compare this against / me sysw show
<file>`), so the intent is visible. The stated *premise* is what is wrong, and it
is the premise that tells an implementer there is nothing to remove.

**Confidence:** high on the shipped text (quoted from source). Medium on whether
an implementer would delete the existing lines — the mock-up suggests replacement,
the ruling sentence suggests addition, and §3.2's diagnosis says there is nothing
there to replace.

---

## [M1] O3/R5's cost argument rests on a reuse that does not exist: `ValidMD`/`ValidMK` hard-code the md/mk HRPs and BCH targets, and `mt1` has its own

**Severity:** Minor

**Where:** §2.2 (the PROPOSED per-chunk checksum), §5 R5, §9 O3.

**The failure, concretely:** §2.2 argues the per-chunk BCH check is cheap because
*"the fork already carries the engine (`codex32/gf32.go`, `gf1024.go`,
`checksum.go`) and already exposes `ValidMD`/`ValidMK`. Far less than a decoder."*

The engine is reusable; the exposed predicates are not. Both pin their format:

```go
func ValidMD(s string) bool {                                  // codex32/mdmk.go:137
	if _, data := splitHRP(s); len(data) > mdRegularMaxLen { return false }
	return verifyMDMK(s, "md", newShortChecksum().generator,
		mdRegularTargetHi, mdRegularTargetLo, mdmkShortSyms)
}
```

`mt1` uses HRP `"mt"` and its own NUMS-derived residue
`MT_REGULAR_CONST = 0x0001_a2fc_877f_9528_d7c1`, derived from the domain string
`shibbolethnumstransaction` (`mt-codec/src/consts.rs:15,23-30`), with its own
length brackets. So R5 needs new constants and a new bracket ported into the fork
under the Rust-primary rule, not a call to an existing predicate. `mt-codec`'s own
header warns about exactly this: *"a constant copied from a sibling codec … yields
chunks that are self-consistent and unreadable by every other implementation."*

The decision O3 defers is being costed against a reuse that is not there. Not a
wrong outcome on its own — hence Minor — but it is an input to a ruling the spec
leaves open.

**Confidence:** high. Both sources quoted.

---

## [M2] §4.6 names three inputs to the plate-table regeneration and conflates two artifact families; the QR family's biggest input is not among them

**Severity:** Minor

**Where:** §4.6, §4.2, §7.

**The failure, concretely:** §4.6 rules the table must be regenerated and lists the
inputs: PSBT→signed-transaction sizes, a **49-bit → 55-bit header** correction, and
`SPEC_mt_qr_DEFERRED.md` §10.14's font-metric correction. *"One job, three
inputs."*

Those three inputs belong to two different families. The 55-bit header is the
`mt1` chunk header — a **text plate** quantity. §4.2 rules the QR carries *"the raw
transaction bytes"*, which have no `mt1` header at all. Meanwhile the QR family's
own dominant change is absent: the source table was built for
`SPEC_mt_qr_DEFERRED.md` §10.3's ruled QR payload, *"`mt1` chunks, bech32
UPPERCASE"* — QR **alphanumeric** mode, 4,296 characters at v40-L — and §4.2 moves
it to **byte** mode, 2,953 bytes at v40-L. That is a ~31% capacity change per
symbol and it drives plate count directly.

§7's mode-segmentation gate would probably catch a table still computed in
alphanumeric. But §4.6 is what tells the person doing the regeneration what to
correct for, and it lists the text-plate header and omits the QR mode.

**Confidence:** medium-high. The 4296/2953 figures are §7's own; the alphanumeric
ruling is quoted from `SPEC_mt_qr_DEFERRED.md` §10 item 3. What would settle it is
whether the source table was ever computed for byte mode — I did not open
`RESULTS_qr_modes_2026-08-22.txt`.

---

## [M3] §3.1's table says the carousel is "never" content-dependent; it already is

**Severity:** Minor

**Where:** §3.1. Code: `gui/gui.go:2098-2101`, `:2445`.

**The failure, concretely:** §3.1's table:

| | asks | content-dependent? |
| --- | --- | --- |
| the carousel | what can this **machine** do? | **never** |

`lastNav()` returns `unlockPayload` when a payload is present and `bip85Derive`
when one is not (`gui/gui.go:2098-2101`), and `npages := int(lastNav) + 1`
(`:2445`) — so the carousel gains and loses an entry, and a dot, with payload
presence. The spec's own §3.1 acknowledges the mechanism in the next paragraph
(*"that endpoint is already spent on `unlockPayload`"*) and the table still says
"never".

The ruling is unaffected — `engraveTransaction` is unconditional either way. The
consequence is a reader who trusts the table as an invariant and does not expect
the carousel's length to vary. Minor.

**Confidence:** high.

---

## [M4] R11's refusal message is wrong in the case an unconditional carousel entry makes common: no payload loaded at all

**Severity:** Minor

**Where:** §3.3, §5 R11.

**The failure, concretely:** §3.1 rules `engraveTransaction` shown always, including
on a machine with nothing loaded (`ctx.sysw == nil`, the state
`gui/sysw_unload.go:35` and `gui/multisig_build_payload.go:67` both test for).
R11's message is specified as *"this payload holds no transaction — load one with
Load Payload."*

With nothing loaded there is no payload. The operator reads a sentence asserting
something about a payload they do not have, and the instruction ("load one with
Load Payload") is right by accident rather than by diagnosis. §3.3's own rule —
*"its refusal names the FIX, not just the problem"* — is met; the *problem*
statement is false. Two distinct states, one message.

**Confidence:** high on the state existing; low on it mattering much, which is why
it is Minor.

---

## Verdict

**3 Critical / 7 Important / 4 Minor / 0 Nit.**

Criticals: C1 (multi-symbol QR is unrecoverable and §4.3's mandatory test cannot
pass), C2 (`validateMdmk` QR-encodes the chunks the spec forbids in QR), C3 (`tx:`
as a reserved prefix falls through `gui/scan.go:79` to a free-text plate).

**Not examined**, so the next round should not read this as coverage:

- **§4.5's search itself.** I checked that the objective is a total order and that
  both R0 corrections are carried, but I did not re-derive the "41 configurations
  tie" claim, did not verify the 79 mm usable / 4-module quiet zone / 25.5 mm
  legend geometry against `SPEC_mt_qr_DEFERRED.md` §4, and did not open
  `RESULTS_qr_modes_2026-08-22.txt`.
- **§3.4's field taxonomy** (derived vs asserted). I found no constructible failure
  in the fee/`TO` split, but I did not test whether an asserted fee that is
  materially wrong can be relied on by an operator confirming a pre-signed
  timelocked spend — that needs a journey walk, not a read.
- **§2.3's arithmetic and the `boundBlob` no-wrap argument** — declared settled in
  the brief; I did not re-derive it, and I did not check whether the Go
  `ParseHeader` bound survives `MaxSectionLen = 32,734` on a 32-bit target.
- **§2.4's content-based sealing across the non-transaction classes.** I confirmed
  `Class::is_secret()` is `Mnemonic | Codex32Secret | Passphrase` in both Rust
  (`crates/me-cli/src/sysw/record.rs:42-48`) and Go (`sysw/record.go:36-38`), and
  that `MdMk`/`Descriptor`/`Address`/`FreeText` therefore pack unsealed by
  default under the new rule. I did **not** establish whether any `mk1` form can
  carry private key material — if one can, that is a Critical this round missed.
- **§4.4's legend-last mechanism** against the actual engrave plan builder. I took
  the ordering claim on its face; whether `backup.EngraveText` / `toPlate` can
  even emit the legend last was not checked, and it is the same
  affordance-without-a-mechanism class as C2.
- **The `mt` side of §6/P2** beyond confirming that `Inspect` exists and that
  `mt1` has its own HRP and BCH constants.
