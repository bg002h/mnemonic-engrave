# RECON — can systemwide payload / NFC / keyboard each deliver md1 + N mk1?

Read-only recon. Fork: `/scratch/code/shibboleth/seedhammer`. Spec:
`/scratch/code/shibboleth/mnemonic-engrave/design/SPEC_systemwide_payloads.md`.
All line numbers measured against the working tree at the time of this recon
(2026-08-18); `cargo build --release` in `mnemonic-engrave` succeeded
(`Finished release profile [optimized] target(s) in 3.57s`) and the `me`
binary invocations below are real command output, not transcription.

## Verdict

**Systemwide payload**: the container mechanically CAN hold one md1 plus N
mk1 in one `ClassMDMK` set — verified live with `me sysw pack`, which packed
one template md1 and two mk1 chunks into one payload and `me sysw show`
reported all three as `md1/mk1 — confirmed`. The session can retrieve them
either one-at-a-time (`take`, first match) or all-at-once (`takeAll`, payload
order) — both exist and are both used, by different programs. But the ONE
program shaped like "supply a host-built md1 policy"
(`supplyMultisigPolicyFlow`) uses the single-take path for at most one card
and then explicitly REFUSES if any mk1 card is present in what it gathers
(`gui/multisig.go:105`, `gui/multisig_supply.go:18-34`) — so today this
specific flow cannot deliver md1 + key cards together, even though the
payload format and the session API both already support it. **NFC**: the
device already runs a genuine mixed-transport, N-card gather
(`bundleGatherFlow`, `gui/bundle_flow.go:153`) that seeds from the payload
once and then scans NFC repeatedly for more cards of either kind — but it is
wired to `bundleFlow` (Engrave Bundle) and to `buildMultisigPolicyFlow`'s
cosigner-set step, not to `supplyMultisigPolicyFlow`, which uses the same
gatherer but then throws away anything but exactly one md1
(`extractSuppliedMd1`). Single-object NFC scans (`md1GatherFlow`,
`mk1GatherFlow`) gather chunks of ONE object, not a set of objects, so they
are not the mechanism in question. **Keyboard**: a typed-entry surface for
bech32-family `m*1` strings already exists and already recognizes `md`/`mk`
HRPs (`validateMStar`, `gui/codex32_polish.go:259-281`, returns `mdmkText` for
md/mk), with a character set (`codex32Keys`) that is the full bech32 alphabet.
But every one of its 4 call sites is titled for a SECRET (`"Type ms1"`,
`"Input m*1 string"`) and none is wired into `bundleGatherFlow` or
`supplyMultisigPolicyFlow` — there is no typed-entry option in the gather
loop at all. Practicality is also adverse: a real 2-of-3 `wsh_sortedmulti`
full-policy md1 is 478 characters across 6 chunks of ~80 each (measured,
`md/testdata/vectors/multisig_wsh_full.md1.txt`); the matching TEMPLATE md1
is 28 characters, one chunk (measured, `md/md_test.go:29`).

## Transport matrix

| transport | single md1 | N mk1 cards | mixed with other transports | gap |
| --- | --- | --- | --- | --- |
| Systemwide payload | Yes — `ClassMDMK`, live-tested | Yes, format-wise (`me sysw pack` packed 1 md1 + 2 mk1 in one container, confirmed by `me sysw show`) | Payload seeds `bundleGatherFlow`'s accumulator (`ctx.syswBundleSeeds`), then NFC continues into the SAME set (`gui/bundle_flow.go:190-197`) | The one program matching the ONE QUESTION's shape, `supplyMultisigPolicyFlow`, only `take()`s the FIRST `ClassMDMK` record (`gui/multisig.go:99-101`) and then `extractSuppliedMd1` refuses any mk1 in the gathered result (`gui/multisig_supply.go:26-27`) |
| NFC | Yes — `mdmkText`, `gui/scan.go:91-92` | Yes, via `bundleGatherFlow`'s scan loop — but ONLY inside `bundleFlow` / `buildMultisigPolicyFlow`, never `supplyMultisigPolicyFlow` | Same seam as above (payload pre-fill + NFC continuation) is proven in `buildMultisigPolicyFlow` (`gui/multisig_build.go:135-136`) | `supplyMultisigPolicyFlow` calls the identical `bundleGatherFlow` (`gui/multisig.go:97`) so N mk1 cards DO get gathered mechanically, then are discarded by `extractSuppliedMd1`'s refusal |
| Keyboard | Mechanically yes — `validateMStar` recognizes `md`/`mk` HRPs and returns `mdmkText` (`gui/codex32_polish.go:271-278`); charset is bech32-complete (`codex32Keys`, `gui/codex32_polish.go:242`) | No — no call site of `inputCodex32Flow` is inside any gather loop; `bundleGatherFlow`'s loop offers only Back/Done + NFC scan events, no keyboard branch (`gui/bundle_flow.go:203-267`) | Not applicable — keyboard is not wired into the gather flow at all | The typed-entry surface exists but is orphaned from md1/mk1 gathering; its 4 call sites are Backup-Wallet's typed-seed menu and two VERIFY re-entries, all titled for `ms1` secrets (`gui/gui.go:2686`, `gui/codex32_polish.go:186`, `gui/multisig_verify.go:1136`, `gui/singlesig_verify.go:168`) |

## Gather flows as they exist

**`bundleGatherFlow` — the ONLY existing mixed-transport, multi-card gather** (`gui/bundle_flow.go:153-268`). It seeds from the systemwide payload once, then scans NFC in a loop, accumulating into ONE set of `bundleCard`s that can be a mix of `cardMD1` and `cardMK1`:

```go
// gui/bundle_flow.go:190-197
for _, seed := range ctx.syswBundleSeeds {
    if seed == "" {
        continue
    }
    scr.g.offer(mdmkText(seed))
}
ctx.syswBundleSeeds = nil
// One loop, one shape, one backoff -- see startScanner (F-126). A nil
// reader is handled there and yields a channel that never delivers.
scans, stopScanner := startScanner(ctx, ctx.Platform.NFCReader())
```

The scan loop (`gui/bundle_flow.go:203-267`) offers only `Back` (abandon) and
`Done` (Button3/Center) besides the NFC channel — **no keyboard branch
exists in this loop**.

**Two callers feed it differently.** `supplyMultisigPolicyFlow` seeds it with
AT MOST ONE record (via `take`, first match):

```go
// gui/multisig.go:99-101 (comment) and :103
if body, ok := syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?"); ok {
    ctx.syswBundleSeeds = []string{body}
}
cards, ok := bundleGatherFlow(ctx, th, "Engrave Bundle")
```

`buildMultisigPolicyFlow` seeds it with the WHOLE payload set (via
`takeAll`, through `buildCosignerSource`):

```go
// gui/multisig_build.go:135-137
ctx.syswBundleSeeds = records   // records = buildCosignerSource(ctx), i.e. ALL ClassMDMK
cards, ok := bundleGatherFlow(ctx, th, buildCosignerGatherTitle)
```

**Then `supplyMultisigPolicyFlow` throws away any mk1 the gather produced.**
This is the verified form of the brief's "known constraint":

```go
// gui/multisig_supply.go:18-34
func extractSuppliedMd1(cards []bundleCard) ([]string, bool) {
	var md1 []string
	count := 0
	for _, c := range cards {
		switch c.kind {
		case cardMD1:
			count++
			md1 = c.strings
		case cardMK1, cardMS1:
			return nil, false // a stray key/secret card pollutes the supply.
		}
	}
	if count != 1 {
		return nil, false // 0 md1 (nothing to engrave) or >=2 (ambiguous).
	}
	return md1, true
}
```

```go
// gui/multisig.go:99-105
	suppliedMd1, ok := extractSuppliedMd1(cards)
	if !ok {
		showError(ctx, th, "Engrave Multisig", "Supply exactly one wallet-policy md1 (and no key cards).")
		return
	}
```

The brief's "known constraint to verify" is CONFIRMED, and confirmed to be a
**double** refusal: the gather itself (`bundleGatherFlow`) would happily
accumulate md1 + N mk1 in one pass — it is `extractSuppliedMd1` immediately
after it that rejects the mix. `gui/multisig_supply.go:36-58` describes a
DIFFERENT function, `extractReadbackMd1AndMk1s`, that DOES admit one md1 plus
several mk1s — but it is the VERIFY-side readback filter (reads back
engraved plates for cross-check), not the supply-side input filter, and its
own doc comment says so explicitly (`gui/multisig_supply.go:52-54`: "do NOT
widen [`extractSuppliedMd1`]").

**Single-object gathers are a different mechanism and not what the ONE
QUESTION asks about.** `md1GatherFlow` (`gui/md1_gather.go:76-135`) and
`mk1GatherFlow` (`gui/mk1_inspect.go:175-232`) each scan repeatedly, but
every scan must belong to the SAME chunk set (same `ChunkSetID`) of the SAME
single object — a differently-keyed scan is rejected as `gatherForeign`
(`gui/md1_gather.go:104`, `gui/mk1_inspect.go:201`). Neither ever assembles
more than one object.

**The session's retrieval primitives, both real and both used by different
callers** (`gui/sysw_session.go`):

```go
// gui/sysw_session.go:114-124 — first match only
func (s *syswSession) take(want sysw.Class) (string, bool) { ... }

// gui/sysw_session.go:158-169 — every record of the class, in payload order
func (s *syswSession) takeAll(want sysw.Class) ([]string, bool) { ... }
```

`take` is what `syswOffer`/`supplyMultisigPolicyFlow` use; `takeAll` is what
`buildCosignerSource` (`gui/multisig_build_payload.go:66-74`) uses. The gap
in `supplyMultisigPolicyFlow` is a choice of primitive, not a missing one.

**Machine-verified: the payload format itself already holds md1 + N mk1
together.** Built with `cargo build --release` in `mnemonic-engrave`
(finished, no errors) and run live:

```
$ me sysw pack --no-passphrase --out /tmp/test.bin \
    "md1yzpqqxppcgsc9kdmw6d5dp08f" \
    "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf" \
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x"
strength: no passphrase — BELOW the threshold
digest:   5d97 ecfd 4f8c 109a 9f69 b932 5c0b b3c3
exit=0

$ me sysw show /tmp/test.bin
sealed:   false
pub_len:  221
ct_len:   0
identity: 329eb1bf7557f2a19234b3bdfdd5745967233b395488138a37410251fde0e40e
digest:   5d97 ecfd 4f8c 109a 9f69 b932 5c0b b3c3
public record 0: md1/mk1 — confirmed
public record 1: md1/mk1 — confirmed
public record 2: md1/mk1 — confirmed
```

## Keyboard practicality

Measured, not estimated:

```
$ tr -d '\n' < md/testdata/vectors/multisig_wsh_full.md1.txt | wc -c
478
$ wc -l < md/testdata/vectors/multisig_wsh_full.md1.txt
6
$ awk '{print length($0)}' md/testdata/vectors/multisig_wsh_full.md1.txt
80 80 80 80 80 78
```

A real 2-of-3 `wsh_sortedmulti` FULL policy (`md/testdata/vectors/multisig_wsh_full.meta.json`, k=2, n=3, `shared_origin: m/48'/0'/0'/2'`, `fp_present: false`) is **478 characters, chunked into 6 records of ~80 characters each** — meaning an operator typing this by hand would need to complete SIX separate ~80-character typed entries, each subject to the codex32 keyboard's own BCH-window validation.

```
$ echo -n "md1yzpqqxppcgsc9kdmw6d5dp08f" | wc -c
28
```

The matching TEMPLATE md1 for the SAME `wsh_sortedmulti` 2-of-3 shape (keyless, `md/md_test.go:29`) is **28 characters, one chunk** — 17x shorter, because it carries no xpubs.

Precedent for typing a long code exists but is capped well under 478: `PassphraseKeyboard`'s free-text mode caps at `passphrase.MaxLen = 100` (`passphrase/passphrase.go:12-13`, enforced `passphrase/passphrase.go:36`), and `TestPassphraseKeyboardStaysOnPanel`'s own comment (`gui/passphrase_keyboard.go:439-447`) documents that at exactly 100 characters the readout already overflowed the 480px panel until it was clamped. No existing keyboard flow has ever needed to accept 478 characters in one field; `ftMaxLineLen = backup.MaxTitleLen = 18` (`gui/freetext_flow.go:28`, `backup/backup.go:71`) caps Engrave Text's single-line fields far below that too.

The `inputCodex32Flow` keyboard (`gui/gui.go:1167-1227`, keypad built by `newCodex32Keyboard`, `gui/codex32_polish.go:244-257`) has no length cap visible in the read code and DOES route md/mk fragments to `mdmkText` (`gui/codex32_polish.go:271-278`), so nothing in the validator itself would refuse a long md1 chunk — but N=6 round trips through this screen, per full-policy md1, is what "practical" would mean if it were ever wired to the supply flow, and it is not wired there today (see Transport matrix).

Character set: `codex32Keys = "1234567890\nqwertyuiop\nasdfghjkl\nzxcvbnm"` (`gui/codex32_polish.go:242`) with `b`, `i`, `o` statically disabled (`gui/codex32_polish.go:250-254`) — this is exactly the bech32 charset (`qpzry9x8gf2tvdw0s3jn54khce6mua7l` plus digit `1` as the HRP separator), so character-wise it is sufficient for any md1/mk1/ms1 string.

## Provenance invariants

`syswSourceAccept` — the F3 mechanism, quoted verbatim:

```go
// gui/sysw_source.go:115-136
func syswSourceAccept(ctx *Context, th *Colors, title string, c sysw.Class, src syswSource) bool {
	if src == srcTyped {
		// F3 is "always, for anything not typed". A screen for typed input
		// would be a confirmation the operator has nothing to check.
		return true
	}
	var sealed, weak bool
	if ctx.sysw != nil {
		sealed, weak = ctx.sysw.sealed, ctx.sysw.weak
	}
	var lines []string
	for _, f := range syswFlags(c, false, src, sealed, weak) {
		switch f {
		case flagSource:
			lines = append(lines, "Source: "+syswSourceName(src))
		case flagNFCNoIntegrity:
			lines = append(lines,
				"This secret arrived with NO integrity check at all - nothing "+
					"stands behind a tag's contents, and there is nothing to compare.")
		}
	}
	return confirmReviewScreen(ctx, th, title, lines)
}
```

Spec statement of the invariant (`SPEC_systemwide_payloads.md:255-256`): "The
screen where a record ENTERS a program names its source... Provenance must
never be something established by reading code," scoped 2026-08-12 (§13 D5)
to exactly ONE screen per record — the point-of-entry screen, not every
downstream screen.

**`syswSourceAccept` has exactly 6 call sites, all outside the multisig
supply/build path**, measured by grep:

```
gui/derive_xpub.go:231   sysw.ClassMnemonic, srcNFC
gui/derive_xpub.go:244   sysw.ClassMnemonic, srcPayload
gui/freetext_flow.go:1505  sysw.ClassFreeText, src
gui/passphrase_flow.go:671  sysw.ClassPassphrase, src
gui/passphrase_flow.go:815  sysw.ClassPassphrase, srcDerived
```
(`gui/sysw_source.go:115` is the definition itself.)

**Would a mixed-transport gather (template via payload, key cards via NFC)
break the invariant?** No call to `syswSourceAccept` exists anywhere in
`gui/multisig.go`, `gui/bundle_flow.go`, or `gui/multisig_build_payload.go`
(grepped, zero hits) — `ClassMDMK` records entering
`supplyMultisigPolicyFlow`, `bundleFlow`, or `buildMultisigPolicyFlow` never
reach this screen at all today, so a future mixed-transport gather would not
violate a currently-enforced rule; it would be extending a rule that
currently has a `ClassMDMK`-shaped hole in its own coverage. `bundleFlow`'s
own tally screen (`gui/bundle_flow.go:100-121`) shows running per-kind counts
(`md1 descriptors: N`, `mk1 keys: N`) but not per-card source — a different,
weaker disclosure than F3's "source, per record."

## Open / could not determine

- Whether the spec's own §3.3.2 "reachability" note — "Cells with no
  consumption path today: … `ClassMDMK` at Single-Sig and Multisig (the
  supplied-md1 path)" (`SPEC_systemwide_payloads.md:401-402`) — predates the
  `syswOffer(..., sysw.ClassMDMK, ...)` call now present at `gui/multisig.go:99`
  and `gui/bundle_flow.go:24`, or refers to something narrower than what is
  in the tree now. Not resolved from static reading; would need the spec's
  own revision history or a git-blame pass, which was out of scope for this
  recon.
- Whether `buildMultisigPolicyFlow`'s on-device sortedmulti-only composer
  (`gui/multisig_build.go:39-`) is in tension with the settled fact "the
  device CONSUMES a host-built policy; it does not compose one (LATER
  cycle)." It plainly composes a narrow (`sortedmulti` k-of-n) policy
  on-device today; whether that counts as the "later cycle" capability or is
  considered out-of-scope-of-that-decision is a design/scoping question, not
  a recon fact — flagged, not resolved, per the brief's "do not propose a
  design" instruction.
- Exact character count for a real multi-cosigner mk1 SET (as opposed to one
  mk1 card) was not assembled into a single N-of-M total; only per-card chunk
  lengths were measured (`mk/mk_test.go:10-11`, two chunks, 110 + 80 = 190
  characters for one card). A precise "keystrokes for N mk1 cards" figure
  would need N chosen and is left as an order-of-magnitude note (each card is
  the same order as one md1 chunk group, ~150-200 characters).
- Whether `NFCReader()`'s underlying `nfc/poller`, `nfc/type2`, `nfc/type4`,
  `nfc/type5` packages impose any additional per-tag-type restriction beyond
  what `nfc/ndef/ndef.go`'s `RecordReader` already filters (well-known
  `T`/`U` only, chunked records rejected, TNF must be `tnfWellKnown`) was not
  traced end-to-end — the recon confirmed the NDEF-layer filter but did not
  read the four hardware-tag-type packages line by line.
