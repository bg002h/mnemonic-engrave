# F-530 review — the duplicate-key rule over a `*bip380.Descriptor`

**Scope.** `git diff 9b36ed7..2a40f5d` in `seedhammer`, plus `git show cae0634c`
in `mnemonic-engrave`. One question: does it close F-530, and does it introduce
a new defect. Not a fresh audit.

**Verdict up front: NOT GREEN.** The gate is placed correctly — the three
`descriptorFlow` callers and both branches of the Addresses button are covered,
and the engrave path is untouched — but the *predicate* is wrong. It compares
the SPELLING of a key expression where `address.derivePubKey` compares its
MEANING, and at least four spellings that derive identical pubkeys compare
unequal. One of them produces **byte-identical receive and change addresses** to
the descriptor the diff's own fixture refuses.

Everything below was constructed and run, not assessed. All probes were removed
and both trees verified clean (`git status --porcelain` empty in each).

---

## C-1 (Critical) — `sameKeyExpression` compares spelling, `derivePubKey` compares derivation: four measured false negatives

`gui/descriptor_duplicate_keys.go:74-82`

```go
	if len(a.Children) != len(b.Children) {
		return false
	}
	for i := range a.Children {
		x, y := a.Children[i], b.Children[i]
		if x.Type != y.Type || x.Index != y.Index || x.End != y.End || x.Hardened != y.Hardened {
```

`address/address.go:189-201` normalises before deriving:

```go
	children := k.Children
	if len(children) == 0 {
		// Default to <0;1>/*.
```

and `derivePubKey`'s `ChildDerivation` / `WildcardDerivation` arms read only
`c.Index` / `index` — **`Hardened` is never consulted**, so `/0h/*` derives the
unhardened child and `/*h` derives the plain wildcard.

`bip380.ParseKey` sets `Children` only when a `/` follows the xpub
(`bip380/bip380.go:392-400`), so "no explicit children" is `nil`, never the
materialised `<0;1>/*`. The predicate therefore says "different" where the
deriver says "same".

### The failure case, measured

Input (scanned QR, `gui/scan.go:87` → `gui/gui.go:2599` → `descriptorFlow`):

```
wsh(sortedmulti(2,XPUB_A,XPUB_A/<0;1>/*,XPUB_B))
```

versus the diff's own refused fixture `descRepeatedKey`:

```
wsh(sortedmulti(2,XPUB_A/<0;1>/*,XPUB_A/<0;1>/*,XPUB_B))
```

| descriptor | `descriptorRepeatsAKey` | `address.Supported` | `Receive(0)` | `Change(0)` |
| --- | --- | --- | --- | --- |
| explicit both (refused today) | **true** | true | `bc1qaej2r8zv5l00dndh7u6ch5w5pn8lrv70vayz3xu99j3y2emqdghq25tr6q` | `bc1qxamrpluet28nhf003tvr6mc72cnlw9lza5xl5mpgvdh6dpw06vuqfzua9r` |
| implicit vs explicit | **false** | true | `bc1qaej2r8zv5l00dndh7u6ch5w5pn8lrv70vayz3xu99j3y2emqdghq25tr6q` | `bc1qxamrpluet28nhf003tvr6mc72cnlw9lza5xl5mpgvdh6dpw06vuqfzua9r` |

**Same wallet. Same script. Same addresses. Refused one way, silently derived
the other.**

### Verified at the screen, not just at the predicate

Driving the real flow (`descriptorFlow` under `synctest`, `sh2DisplaySize`, the
descriptor obtained through `nonstandard.OutputDescriptor` exactly as both
no-md1 callers obtain it):

```
first frame: "Type2-of-3multisigScriptSegwit(P2WSH)EngraveDescriptor"
ADDRESSES CHOICE OPENED at frame 0
ADDRESS SHOWN: "Receiveaddresses0:bc1qaej2r8zv5l00dndh7u6ch5w5pn8lrv70vayz3xu99j3y2emqdghq25tr6q
                1:bc1qp58s6mgsaptyt63cwcxfu56asmxzz4rkncr64lv2rn3cwsrsnf7q768tgl
                2:bc1q8q4dscnnrddm7wszaems066gqdpjxqse7k6ngp63yedf5853kj3szt463m"
```

The screen states `2-of-3 multisig`, carries **no** F-530 sentence, opens the
Addresses choice, and pays out the very address the gate exists to withhold.
This is `TestScannedRepeatedKeyDescriptorOffersNoAddresses`'s exact assertion,
failing on a one-token change to the input string.

### The full set of undetected spellings

All reach `descriptorFlow` through the scan route, which runs **no** §4.7
admission (`gui/scan.go:87` calls `nonstandard.OutputDescriptor` directly):

| spelling | `dup` | effect |
| --- | --- | --- |
| `A` vs `A/<0;1>/*` | false | identical pubkey on **both** chains, every index |
| `A/0/*` vs `A/<0;1>/*` | false | identical pubkey on the **receive** chain, every index (the chain the operator funds and the one `descriptorAddressFlow` opens on) |
| `A/0h/*` vs `A/0/*` | false | identical pubkey on both chains — `derivePubKey` ignores `Hardened` |
| `A/*h` vs `A/*` | false | identical pubkey on both chains — same reason |

Control: `A/0/*` vs `A/1/*` → `dup=false`, and the two genuinely derive
different pubkeys. The diff's false-positive guard is correct and must be kept.

### The spec commit states the correct rule; the code does not implement it

`mnemonic-engrave` `cae0634c`, `design/SPEC_wallet_policy_composer.md`:

> the rule there is expressed over a `*bip380.Descriptor` (**two key expressions
> that derive the same pubkey**) rather than over slots

That is the right predicate. `sameKeyExpression` implements "two key expressions
written the same way". The gap is against the diff's own stated contract, not
against an outside reading.

### Fix shape (not authoritative — reproduce the case, not the remedy)

Compare what the key derives, not how it is written: normalise `Children`
through the same rule `derivePubKey` applies (empty → `<0;1>/*`; drop `Hardened`
on `Child`/`Wildcard` since it is not honoured; a `Range{i,i+1}` and a
`Child{i}` agree on the receive chain), or — strictly better, and it settles the
"is a receive-only collision a duplicate?" question by construction — compare
`derivePubKey(a, 0, false)` against `derivePubKey(b, 0, false)` and the same at
`change=true`. That is one secp256k1 derivation per pair, already hoisted out of
the frame loop next to `address.Supported` at `gui/gui.go:3238`, so it costs the
0-alloc path nothing. `DescriptorScreen.Draw` (`gui/gui.go:3355`) calls the
predicate **per frame** and would need the answer threaded in from `Confirm`
rather than recomputed.

**How verified.** `bip380.Parse` + `nonstandard.OutputDescriptor` +
`address.Receive`/`Change`/`Supported` probes in `gui`; a `synctest` walk of
`descriptorFlow` capturing the rendered frames; addresses compared byte-for-byte
against the refused fixture's.

---

## C-2 (Critical) — on the payload-record route the gate catches nothing, and misses exactly what gets through

`sysw/descriptor.go:493`, conjunct 8(b):

```go
			if same && slices.Equal(a.Children, b.Children) {
				return false
			}
```

The same syntactic comparison, one layer up. Running §4.7 admission over the
same inputs:

| descriptor | `admitDescriptor` | `descriptorRepeatsAKey` |
| --- | --- | --- |
| exact dup, implicit both | **false** (refused) | true |
| exact dup, explicit both | **false** (refused) | true |
| implicit vs explicit | **true** (admitted) | **false** |
| `A/0/*` vs `A/<0;1>/*` | **true** (admitted) | **false** |
| `A/0h/*` vs `A/0/*` | false (conjunct 7, hardened) | false |

The payload-record caller at `gui/wallet_policy.go:117-122` receives only
§4.7-admitted records (its own comment asserts this). So on that route the
F-530 gate fires **only on descriptors admission has already rejected**, and is
**silent on both descriptors that actually arrive**. The diff's rationale —
"Two of `descriptorFlow`'s three callers have no md1 behind them … which is why
F-530's remedy has to be the rule over a `*bip380.Descriptor`"
(`gui/descriptor_duplicate_keys.go:10-16`) — names two surfaces and closes, in
practice, one of them partially.

This is not "code the diff does not touch": the diff's claim to close the
payload route depends on it entirely.

**Rust-primary rule applies to this half.** The primary carries the identical
defect —
`mnemonic-engrave/crates/me-cli/src/descriptor/admit.rs:295`:

```rust
            if a.identity() == b.identity() && a.children == b.children {
```

while `crates/me-cli/src/descriptor/derive.rs:52` documents the opposite
normalisation in the same crate:

> `§5.3(a′)`: an absent path IS the device's `<0;1>/*`, made explicit.

So `me` admits `wsh(sortedmulti(2,A,A/<0;1>/*,B))` too. Per the standing rule
this is fixed **in Rust first, with a vector**, and the Go change in
`sysw/descriptor.go` becomes the convergence port. (`gui/` itself is
fork-native and exempt, so C-1's fix may land in Go directly.)

**How verified.** `admitDescriptor`/`keyIdentityOK`/`useSitesOK` called directly
from a temporary in-package `sysw` test; Rust read at the cited lines.

---

## I-1 (Important) — the warning is pushed off a screen that does not scroll by an uncapped, attacker-supplied Title

`gui/gui.go:3355-3359` appends the two sentences after `Title`, `Type` and
`Script`. `desc.Title` comes from a scanned artefact with **no length bound**:
`nonstandard/parse.go:105` (BlueWallet `Name:`) and `nonstandard/parse.go:53`
(JSON `label`).

Measured, real JSON label + a genuine duplicate (both keys implicit, so the
predicate *does* fire):

| Title | funds sentence drawn | both sentences drawn | lost text |
| --- | --- | --- | --- |
| `Company Treasury Multisig Cold Storage 2026` (43) | 105/105 | 169/169 | — |
| `Company Treasury Multisig Cold Storage Vault 2026` (49) | 105/105 | 144/169 | `for a wallet that reuses a key.` |
| 120 chars of words | 105/105 | 105/169 | the whole `No addresses:` sentence |
| 200 chars of words | **38/105** | 38/169 | the funds sentence, cut mid-clause |

At 200 characters the operator sees `2-of-3 multisig`, a truncated fragment, and
an empty Button2 — **the silent refusal the diff's own comment (`gui/gui.go:3344-3350`,
citing F-531 review I-2) says it exists to prevent**, restored by a field the
scanned artefact controls. The plate is still engravable from that screen, so
the engrave decision is made with no warning at all.

**How verified.** `descriptorFlow` rendered through `synctest` at
`sh2DisplaySize`, frames measured with the diff's own `bodyDrawnFully`.

---

## I-2 (Important) — the fit gate's headroom is measured in a unit that does not cover a real Title

`gui/descriptor_duplicate_keys_test.go:196-215`. `drawDescriptorScreen` pads
with `strings.Repeat("x", titlePad)` — a narrow glyph, and one unbroken token.
The gate reports:

```
the warning fits with 44 characters of Title to spare (margin 24)
```

Real titles wrap at word boundaries and use wider glyphs. Sweeping word length
against the same body:

```
wordlen= 8 FIRST FAIL at 3 words, title len=26: "mmmmmmmm mmmmmmmm mmmmmmmm"
wordlen=12 FIRST FAIL at 2 words, title len=25: "mmmmmmmmmmmm mmmmmmmmmmmm"
```

A **25-character** Title can overflow while the gate certifies 44 characters of
slack against a 24-character margin. The comment above the constant — *"a
threshold a test passes EXACTLY will flap … If this fails, the answer is shorter
copy"* — is sound reasoning applied to a number that does not measure the thing
it names. `head` is not "characters of Title to spare"; it is "characters of
`x` to spare", and the two differ by up to ~19 in the direction that hides the
defect.

Pad with a realistic word-wrapped string (or with the widest glyph in the face)
and the gate becomes a claim about titles.

**How verified.** Sweep over word length × word count through the diff's own
`drawDescriptorScreen`/`bodyDrawnFully` seam.

---

## M-1 (Minor) — the predicate's deliberate exclusions are pinned by nothing; 1319/1319 stays green when they are reinstated

`gui/descriptor_duplicate_keys.go:39-45` makes a load-bearing funds claim:

> DELIBERATELY NOT COMPARED: MasterFingerprint and DerivationPath. … Comparing
> them would be a predicate that misses the defect whenever the two seats were
> labelled with different origins — which is what a coordinator bug producing
> this shape would plausibly do.

Mutation applied:

```go
	if a.MasterFingerprint != b.MasterFingerprint || !slices.Equal(a.DerivationPath, b.DerivationPath) {
		return false
	}
```

Result: **`RESULT: ok -- all 1319 tests ran across 24 shards`**. Every fixture in
the new table (`descRepeatedKey`, `descDistinctKeys`, `descDisjointChildren`) is
bare xpubs — `MasterFingerprint == 0` and identical implied `DerivationPath` on
every key — so nothing in the suite distinguishes the shipped predicate from the
one its own comment argues against. Confirmed the excluded behaviour is real
today (`[11111111/48h/0h/0h/2h]A/<0;1>/*` + `[22222222/48h/0h/1h/2h]A/<0;1>/*`
→ `dup=true`); it is simply untested. One origin-labelled fixture in the table
closes it.

**How verified.** Mutation + full `gui` shard run
(`mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`), then reverted.

---

## M-2 (Minor) — the new `Draw` branch is outside the 0-alloc gate's reach

`BenchmarkAllocs` (`gui/gui_test.go:54-95`) drives `DescriptorScreen.Confirm`
with a 5-key descriptor built by `fillDescriptor`, all keys distinct, so
`TestAllocs` never enters `gui/gui.go:3355`'s branch or the per-frame
`descriptorRepeatsAKey` call. Measured directly, the branch is clean —

```
repeated-key screen: 0 allocs/op
distinct-key screen: 0 allocs/op
```

— so this is coverage, not a regression. But `Draw` now calls the predicate
every frame with nothing gating that cost, and a future predicate that allocates
(the natural fix for C-1 does) would ship green. Adding a repeated-key
`DescriptorScreen` to `BenchmarkAllocs`'s `screens` slice is the whole fix.

**How verified.** A temporary in-package benchmark mirroring `BenchmarkAllocs`
over `descRepeatedKey` and `descDistinctKeys`; removed.

---

## N-1 (Nit) — the insertion severed `composerCopyNoAddressesDuplicateKeys`'s doc comment

`gui/composer_copy.go:367-380`. The new function was inserted between
`composerCopyNoAddressesDuplicateKeys`'s doc block and its `func` line, with no
blank line, so the F-531 rationale ("IT EXISTS BECAUSE A SILENT REFUSAL IS A
WORSE SCREEN…", "It is a SECOND line, under composerCopyDuplicateKeys…") now
reads as the head of `composerCopyDescriptorRepeatsAKey`'s comment, and
`composerCopyNoAddressesDuplicateKeys` has none. Insert a blank line before
`// composerCopyDescriptorRepeatsAKey is the F-530 sibling` and move the four
severed lines back down with their function.

## N-2 (Nit) — stale precondition comment

`gui/address_polish.go:19`: *"The caller opens this only when
`address.Supported(desc)`."* As of `gui/gui.go:3259` the condition is
`supported && !reusesAKey`. Still true as stated, but no longer the whole
condition, and it is the sentence a reader consults before assuming a caller
gate exists.

---

## What the diff gets right (checked, no finding)

- **Path coverage of the button.** Both branches (`descriptorAddressFlow`,
  `verifyAddressFlow`) sit behind the one gated `addrBtn`. Enumerated every
  other `*bip380.Descriptor` → address route: `addressListFlow` has exactly two
  callers (`gui/address_polish.go:27`, `gui/md1_inspect.go:166`); the second is
  reached only via `md1PolicyFlow` with a non-nil `at` at
  `gui/md1_gather.go:206`, downstream of `duplicateRefusalBody`.
  `multisigRestoreLines` / `singleSigRestoreLines` / `policyAddressAt` are all
  the md1 route behind F-531's `repeatsASeat`. `PolicyAddressAt` is the
  `cmd/policyprobe` harness, not a screen. `DescriptorScreen` is constructed in
  exactly one non-test place (`gui/gui.go:2874`).
- **The engrave path is deliberately untouched and safe.** Button3 →
  `validateDescriptor` → plate, unchanged; nothing downstream keys off
  `supported` or reads "an address was shown". Consistent with F-531's posture
  (the card stays engravable, the derivation is withheld).
- **The false-positive guard is correct.** `A/0/*` vs `A/1/*` is not flagged and
  still derives (`TestDisjointChildrenStillDerive`), which is the expensive
  direction of wrong and is genuinely protected.
- **BlueWallet-route duplicates are caught.** `parseBlueWalletDescriptor`
  (`nonstandard/parse.go:148-156`) never sets `Children`, so both seats compare
  with empty slices and the predicate fires. C-1 does not reach that route.
- **F-530 closes a hole in F-531 that the diff does not claim.** Two *different*
  md1 slots seated with the same xpub are not a repeated *seat*, so
  `repeatsASeat` passes and `expandedToDescriptor` succeeds — and
  `descriptorRepeatsAKey` now catches it in `Confirm`. Worth recording as a
  deliberate property with a test, since nothing currently pins it.
- **`md1_expand.go:150-175`'s comment was not falsified** — it was written by
  the F-531 fold anticipating F-530 and reads correctly against `2a40f5d`.

---

## Counts

**2 Critical / 2 Important / 2 Minor / 2 Nit**

## Verdict

**NOT GREEN.** The gate is in the right place and the engrave path is safe, but
the predicate answers a different question from the one `address.derivePubKey`
asks. A one-token change to the scanned descriptor — writing one of the two
repeated key expressions with its children implicit — walks straight past the
refusal and displays the identical address, and on the payload-record route the
gate never catches anything admission had not already rejected.
