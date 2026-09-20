# Composer fable review r0 — the FORK half of the fold, implemented

**Verdict: all twelve items closed. 15 commits on `fable-r0-fold`, tip
`0f435048d93f69dfc5b807b9e4addd2dff704e1b`, nothing pushed.** Gate at the tip:
`go vet ./...` carries the baseline's ten `testing.ArtifactDir requires go1.26`
notes and nothing else; `gofmt -l .` is the five-file baseline exactly; every
non-gui package green; the whole gui shard set **ok — all 1369 tests ran across
24 shards**; all 24 `TestFable*` tests PASS, and every one that was RED at
`f5b068fa` is GREEN by a code fix. Firmware **1,644,840 → 1,652,268 B flash
(+7,428), 63,320 → 63,336 B RAM (+16)**.

Repo `/scratch/code/shibboleth/seedhammer`, worktree
`/scratch/code/shibboleth/.tmp/fold-fork`, base
`f5b068faf3049ccf97603dfbaa3709f12893df97`.

| # | finding | commit |
| --- | --- | --- |
| — | adopt lens 4's Appendix A verbatim (RED) | `3387b2e` |
| 1 | L1 C-1 — at most one key-less path | `42d6272`, vector `63bdca8` |
| 2 | L1 I-2 — same KEY at two slots | `9e01f3b` |
| 3 | L1 I-3 — consent names k-of-n of a locked multi | `39cf244` |
| — | gate: sentinel-mapping table covers 1 and 2 | `5f24661` |
| 4 | L1 I-1 = L3 I-1 — §8a names the import consequence | `f020e2d` |
| 5 | L2 I-1 — §8f's NUMS note was false for Nunchuk | `121f918` |
| 6 | L2 I-2 — mixed lock bases under wsh | `eeb1688` |
| 7 | L4 I-1 — key-order picker opens on the setting in force | `2b9f0c4` |
| 8 | L4 I-2 — Back at a passphrase screen is a decline | `6cea9f2` |
| 9 | L3 I-3 — restore doc derives what the consent derives | `423a2ec` |
| 10 | L4 M-1..M-4, L1 M-1 — four Minors | `fe36323` |
| 11 | L4 M-5 — an emptied path refused by name | `51d2dff` |
| 12 | M-6 (three lenses) — testnet tpub in a `key:` record | `0f43504` |

---

## Adoption

Lens 4's Appendix A is in the tree verbatim as
`gui/composer_fable_r0_flow_test.go` (912 lines, 12 tests), adopted in its own
commit `3387b2e` where it is RED for six findings — that RED is the
counterexample and is quoted per item below. Two of its tests DOCUMENTED a
defect rather than refusing it (M-5, M-6) and are inverted in items 11 and 12.
A clearly delimited "ADDED IN THE FOLD, NOT PART OF APPENDIX A" section at the
end carries two tests the adopted walks did not cover (item 8's question-Back
half, and `discardLast`'s bound).

Lens 1's three preserved RED assertions are in
`gui/composer_fable_r0_funds_test.go`, with the helpers they share. **Deviation:
the reviewer's `FABLE_OUT`-gated evidence drivers (`TestFableFundsHarness`,
`TestFableSameKeyTwoSlotsReserialized`, `TestFableDigitPadEdges`,
`TestFableTpubKeyRecord`) are NOT adopted** — they assert nothing and skip
unless an env var is set, so they would have added ~200 lines of code no gate
can fail. Everything they measured is asserted by the RED tests or by the new
tests beside them. Five more tests were written in the fold for findings that
had no adopted test (items 4, 5, 6, 9, and item 2's mint half).

---

## Per finding

### 1 — L1 C-1, at most ONE key-less path (`42d6272`, vector `63bdca8`)

RED before:

```
zz_fable_funds_red_test.go:25: ValidatePathList admitted [1 key, keyless, keyless];
  md compose --experimental refuses it as malleable
--- FAIL: TestFableRedTwoKeylessPathsAreRefused (0.00s)
```

GREEN after; and `TestFableTwoKeylessPathsAgreeWithTheHostOracle` runs
`md compose --wrapper wsh --experimental` over the same twelve lists and
requires the Go port's answer to match arm for arm (it skips when `md` is not
on PATH; the literal table does not).

The rule is in `md.ValidatePathList`, after the `!anyKeyed` check so the
primary's precedence holds (`[K+older, K+after]` is still "every wallet needs
at least one path with a key"). Sentinel `md.ErrComposeTwoKeylessPaths`, typed
error `md.TwoKeylessPathsError{First, Second}` with ZERO-BASED indices, matching
the vector's `error.first`/`error.second`.

I re-measured the oracle myself before implementing, twelve lists both
directions. Every list with ≥2 key-less paths refused — `[keyed,K,K]`,
`[keyed,K,K+older(5)]`, `[K,K+older(5),keyed]`, `[keyed,K+older(5),K+after(200)]`,
`[K+older(5),keyed,K+after(200)]`, `[keyed,2of2,K,K]`, `[keyed,K,2of2,K]` — and
every list with ≤1 admitted.

New §8m body, refused at Done AND at path creation:

> A wallet can have one key-less path, not two. Two of them make this script
> malleable, and no wallet will import it. A time lock does not help. Give one
> of them a key, or fold them into one path.

**The vector (`63bdca8`), and a deviation with its measurement.** The Rust
vector `compose_refusal_keyless_cap.json` is vendored at descriptor-mnemonic
`4de55155f7bb5dda6896f733331f9ccd97275490`, with a conformance gate
(`md/compose_keyless_cap_test.go`) driving all four cases. Three mutations
fire: adjacent-only refusal fails `two_keyless_split_by_a_keyed_path`; refusing
every key-less path fails the admitted control; an unpinned `compose_refusal_*`
file fails both directory scans.

It is pinned SEPARATELY (`md/testdata/compose_refusal_vectors.provenance.json`,
`scripts/vendor-compose-refusal-vectors.sh`) rather than by adding its name to
`composeVectorNames` and re-vendoring the corpus, **because the commit it
arrives on also carries md-codec 0.44.0, "a rendered xpub's header must agree
with its origin", which rewrote the xpubs in all 33
`keyed_compose_*.conformance.json` records, and that rule is not ported here.**
Measured rather than feared: I ran the full re-vendor to `4de55155`, and
`go test ./md/` came back GREEN — `TestKeyedConformanceAgreesWithRust` parses
`.chains[].descriptor` into a struct field it never asserts, so no gate can see
the rewrite. A fresh whole-corpus pin would have imported an unported normative
change with every gate still reporting agreement. The exclusion is not a
hardcoded prefix: `isComposeVectorFile` skips exactly the files the refusal pin
lists.

### 2 — L1 I-2, the same KEY at two slots (`9e01f3b`)

RED before:

```
zz_fable_funds_red_test.go:54: composerDuplicateXpub did not refuse the same key
  at @0 and @1 (xpub strings "xpub6DkFAXW…KFrf" vs "xpub6DXuQW1…BdGH")
--- FAIL: TestFableRedSameKeyReserializedIsRefused (0.01s)
```

Two holes, both closed. `composerDuplicateXpub` now keys on the 65-byte
chain-code‖point — the same bytes `composerArtifactsFor` binds — so the review
refusal names both slots. The origin is deliberately NOT in the comparison
(operator ruling: the same seed at two accounts is two different keys).

**The second hole is mine to report because the brief asked me to determine it:
the device COULD mint what the primary refuses.** `md/duplicate_keys.go` does
not answer this question at all — it finds the same SLOT INDEX twice in one
expression, not the same key material at two slots — so it reported
`DuplicateNone` and `composerArtifactsFor` emitted six keyed chunks. `md
decompose` of the descriptor they decode to:

```
md: decompose: the same extended key is used at 2 positions — xpub6DXuQW1Q…27EABdGH:
  [73c5da0a/48'/0'/0'/2'] at /<0;1>/*
  [73c5da0a/48'/0'/1'/2'] at /<0;1>/*
```

(`md encode` is not the verb for a concrete descriptor — it says so and points
at `md decompose`, which is what I ran.) `md.Composed.Bind` now refuses the
repeat, carrying `md.RepeatedKeyMaterialError{A, B}` so §8m draws §7d's
existing body naming the real pair. Measured against the oracle: `md decompose`
refuses whether the two positions share a use-site or not, so the material
alone decides.

Refusal copy is §7d's existing body reused verbatim, as the brief required.

### 3 — L1 I-3, the consent names k-of-n (`39cf244`)

RED before:

```
zz_fable_funds_red_test.go:75: consent does not name the threshold of the locked
  1-of-2 path (tiered-recovery preset):
        Path 1: 2-of-2
        Path 2: 2 key(s), custom
          5 blocks (about 0.0 days)
--- FAIL: TestFableRedConsentNamesThresholdOfLockedMulti (0.00s)
```

`md.soleMulti` reports the threshold of the ONE `multi*` node a branch holds at
any depth; a branch with none, or with two, still reports 0/0, so
`TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee` still holds. The consent
now prints `Path 2: 1-of-2`, and 9-of-9 and 1-of-9 behind locks are
distinguished. `composerSelfCheck` needed no change — it already preferred the
K/N arm where the codec reports one.

Four existing `md` expectations moved because they pinned the silence, including
a SHIPPED card (`keyed_wsh_timelock_hashlock`) whose two tiers are 2-of-3 and
1-of-2 and whose consent named neither. `TestComposerSelfCheckStillComparesKeyCountsUnderALock`
moved to the threshold arm and gained a mis-tapped-k case; a new
`...WithNoThresholdNode` keeps the count-only fallback under test on a 1-of-1.

### 4 — L1 I-1 = L3 I-1, the import consequence (`f020e2d`)

RED before (test written in the fold; the finding had no adopted test):

```
the §8a key-less body does not name Bitcoin Core / Nunchuk / Liana
the consent does not restate §8a's key-less body
--- FAIL: TestFableKeylessPathNamesTheImportConsequence (0.00s)
```

Two surfaces, one body. §8a's confirm gains the sentence, and the consent
RESTATES the whole body — driven off the DECODED shape, so a path that became
key-less after the confirm is still named. A policy with no key-less path does
not carry it (asserted). Kept to the modal, as the brief said; the per-path row
is unchanged.

New §8a body (344 chars drawn in full, headroom 146, margin 80):

> KEY-LESS PATH (EXPERIMENTAL)
> This path needs no signature. Whoever knows the preimage of its hash can
> spend it. If that preimage is ever engraved, the plate is bearer access.
> It also makes the WHOLE wallet un-importable, keyed paths included. Bitcoin
> Core, Nunchuk and Liana all refuse it. Only md can rebuild this wallet, and
> md cannot sign: no other wallet will watch it or spend from it.

It says "no other wallet will watch it or spend from it" rather than "only md
restores it" because md rebuilds the descriptor and derives addresses but does
not sign, and what an operator about to fund is deciding is who can watch and
spend.

### 5 — L2 I-1, §8f's NUMS note (`121f918`)

RED before, mutation-checked (restoring the old body fires the assertion):

```
§8f still claims Nunchuk imports the NUMS form; libnunchuk 2.1.1 refuses 7 of 7
§8f does not name the wrapper a Nunchuk operator should choose instead
```

New §8f body (272 chars drawn in full, headroom 277):

> KEY PATH: NONE (NUMS)
> Spends use the script paths only. Bitcoin Core imports this form. Nunchuk
> cannot import a NUMS policy at all: for Nunchuk, use wsh, or a tr policy
> whose first path is a single key. Liana and BIP-388 signers need an
> unspendable xpub instead (see F-449), which is a different wallet with
> different addresses.

The F-449 sentence stays because it is true of Liana and BIP-388 signers, and
now says plainly that it is a different wallet — so the copy does not send a
Nunchuk user to the form that derives other addresses.

### 6 — L2 I-2, mixed lock bases (`eeb1688`)

RED before (two positives failed; the four negatives were already correct, so
the predicate is not trivially satisfiable):

```
wsh: [1 key], [older 100 units], [after height 1000000] -- the lens's own case:
  mixed-lock-bases notice shown=false, want true
wsh: [1 key], [after time 1893456000], [older 5 blocks]: shown=false, want true
```

New body (156 chars drawn in full, headroom 378):

> MIXED LOCK BASES
> Some paths lock by block height and others by time. Nunchuk will refuse this
> wallet; Bitcoin Core imports it. Taproot accepts both, because it checks each
> path on its own.

The axis is the BASE, not relative-vs-absolute — `decaying-multisig`, a preset
the device offers by name, mixes `older(blocks)` with `after(height)`, both
HEIGHT, and Nunchuk imports it. Under `tr` a composer path carries at most one
lock, so no leaf can mix; the table asserts the notice does not fire there.

**DEVIATION, stated.** The brief says "add a notice at the review". It is on the
CONSENT. Three reasons: the mapping review is SKIPPED when the flow has no
sources at all (`composerSeatingStep`'s key-less-template arm), so a notice
there has a coverage hole; every other SHAPE fact — §8f, §8i, §8d, §8a's
restatement — is already on the consent, while the mapping review is about
keys; and one site means one copy of the rule. "In the style of the Liana line
of §8g" is honoured in the register of the copy, and the body is filed under
§8g for that reason.

### 7 — L4 I-1, the key-order picker (`2b9f0c4`)

RED before → GREEN after, from the test's own log line:

```
RED:   second pass: §8b re-fired=false, stub banner=true,  consent has UNSORTED mark=false, decoded Sorted=true
GREEN: second pass: §8b re-fired=false, stub banner=false, consent has UNSORTED mark=true,  decoded Sorted=false
```

`Initial` is seeded from the current `Sorted`, and §8b fires once per DECLINE
rather than once per pass — which is what makes the preselection safe, since
re-firing the hold over an answer already confirmed teaches the operator to hold
through it. `Sorted == false` IS the record of the confirmed decline and needs
no second copy: every other writer sets it TRUE (`composerKeysEdit`,
`composerPresetKeys`) and the only assignment of `false` is downstream of the
hold, so an edited or re-preset key set comes back Sorted and re-earns the
confirm.

### 8 — L4 I-2, Back at a passphrase screen (`6cea9f2`)

RED before:

```
composer_fable_r0_flow_test.go:255: Back on the passphrase keyboard did not return
  to the passphrase question; the device drew: "SeatkeysSlot@0,Path1key1of2:
  chooseakey73c5da0am/48h/0h/0h/2h73c5da0am/48h/0h/1h/2hseed1(anyslots)Typeaseed
  Leaveunseated"
--- FAIL: TestFableSpecBackOnThePassphraseKeyboardIsADecline (0.03s)
```

The two Backs mean different things: on the KEYBOARD it re-asks the question, on
the QUESTION it un-registers the seed. `seedRegistry.discardLast` zeroes the
words and refuses a non-last id (seedIDs are indices; the decline case is always
the last entry).

**`gui/multisig_build.go:775-787` IS the same fix, and it took it.** Both flows
had byte-identical code, so they now share `seedPassphraseStep` rather than
carrying two copies of the rule.

**A false PASS I found and fixed in my own test.** The question-Back half needed
a test of its own, and my first form asserted that the seat prompt does not
offer "seed 1" — which passed under the mutation `_ = seedID` in place of
`reg.discardLast(seedID)`, because a declined source is never appended to
`st.sources` either way. It now asserts the NEXT seed's label, which is keyed on
`st.reg.count()`, and the mutation fires:

```
the NEXT seed is labelled from st.reg.count()+1 and the question drew
"AddaBIP-39passphrase?SkipAddpassphrasePassphraseseed2"
```

Two existing tests moved with the behaviour, neither weakened.
`TestBuildFlowScrubsEverySeedOnEveryExit/"Back at the passphrase prompt"` drives
through the now-re-asked question to reach the SAME slot-review exit with the
same seed live; the scrub assertion is untouched.
`TestTheSeamPassphraseOfferReachesOnlyProgramsThatAdmitIt` scans per FILE as a
proxy for per PROGRAM and the call moved into `multisig_build_slots.go`, so the
`want` row moves with it; the `forbidden` half, which is the guarding half, is
untouched.

### 9 — L3 I-3, the restore document (`423a2ec`)

RED before, over eight shapes; GREEN after. Mutation-checked: disabling the new
branch makes 7 of the 8 fail (the flat `sortedmulti` control stays green, as it
must).

```
the restore document says it has no address for a shape the consent screen just
derived bc1q… for
```

`multisigRestoreLines` calls `policyAddressAt` — the consent's own router — so
the two sets of addresses are the same addresses by construction rather than by
two copies agreeing. That needed the md1 CHUNKS (`complexAddressSource` works
over the wire form), so `md1 []string` is threaded through
`multisigRestoreLines` and `multisigRestoreDocFlow` from the three engraving
callers; a caller with no chunks passes nil and gets exactly the old branch.
`repeatsASeat` is still asked FIRST, because `complexAddressSource` refuses a
key-repeating policy too and asking it first would lose F-531's specific
sentence.

**Partial against the brief's wording, and this is the honest scope.** The brief
says "22 of 22 keyed shapes print Descriptor + first receive/change". There is
no `Descriptor:` line on the complex branch and there cannot be one: this
device's `md` package emits no text by design, so no descriptor string exists
for a complex policy. What the document can and now does give is the pair of
addresses that lets a restorer check they rebuilt the right wallet; the md1
plate in the inventory is the policy. The assertion is EQUALITY WITH THE
CONSENT, not "an address appears".

### 10 — the four Minors (`fe36323`)

All four RED before, all GREEN after:

```
opening Keys on a 2-of-3 path and leaving by the forward button twice rewrote it;
  the menu now reads: "Path1:1key…"
Full vs Watch-only was asked though no slot is seed-derived
one seed at two slots planned 2 ms1 plates (byte-identical: true)
20090101 refused with "This build writes dates up to 2038-01-19. For a later
  time, use a block height instead.", want "This build will not write a date
  before 2009 as a time lock."
```

- **M-1** `composerCountPick` takes the value in force and opens on it. The
  threshold picker is seeded only where k is still reachable (a 2-of-3 edited
  down to 2 keys must not open on 3); an unset key set falls outside `min..max`
  and the picker opens on `min` as before.
- **M-2** `composerSeedDerivedSlots` walks `st.assigned`, the same assignments
  `composerSecretCards` walks, so the question and the plates cannot disagree.
- **M-3** dedup by MASTER FINGERPRINT, the identity `composerSeedAccountFor`
  already keys §4f's account rule on. One seed registered bare and again with a
  passphrase is still TWO plates, which is correct — two secrets, two wallets.
- **M-4** `composerDateToUnixUnbanded` lets the refusal ask WHICH SIDE of the
  band the date fell off, against the same constant `composerDateToUnix` bands
  on, so the two cannot disagree again.

No copy change: every body these four reach already existed.

### 11 — L4 M-5, the emptied path (`51d2dff`)

I chose the REFUSAL over silent removal. `composerAddPath` removes a path that
ends empty AT CREATION because there the operator never had one; an EDIT reaches
this state from a path that already exists in a list the operator is reading and
may carry a timelock they set, so deleting it silently would renumber the list
under them and discard that work — the same silent-default class the rest of
this review is about.

`md.LockOnlyPathError{Path, Lock}` carries which of the sentinel's two states
this is, so the GUI picks the body from what the codec saw rather than re-reading
the path list. An `errors.Is`-only caller still gets the lock-only body, asserted
beside the two typed arms.

The adopted test is REWORDED, as the brief allowed, and now drives BOTH states:
`TestFableEmptiedKeylessPathIsRefusedWithABodyThatNamesIt`.

New §8m body (77 chars drawn in full, headroom 476):

> This path has no key and no hash, so nothing can spend it. Add a key or a
> hash, or remove the path.

### 12 — M-6, three lenses, one defect (`0f43504`)

Done host-first per the coordinator's correction: the 82-row table is vendored
from mnemonic-engrave `1cbecbfd5ae387f19c864712e7be33ff5b6cd9c5`, sha256
`c5f721455b75513b4217c200cf49b4b0eed471fbfcb704121c3cebbd09066770`, and the
provenance pin names that commit. The vendor DROVE the change, in this order:

```
vendor the table + pin      -> RED: fixture has 82 rows, pin says 82, plan says 81
update the three-number gate-> RED: key-testnet-tpub-refused-s1: Classify(…) = 10,
                                    want 0 (host's answer)
                                    ParseKeyRecord admitted a testnet key
add the version-byte check  -> GREEN
```

The rule is `ek.IsForNet(&chaincfg.MainNetParams)` in `sysw.ParseKeyRecord` —
the version bytes, not a `"tpub"` prefix test, because the prefix is what those
bytes base58-encode to. `Classify` then returns `ClassUnknown`, so the door does
not count the record and seating never offers it. The device prints no §8n line.

`sysw/composer_records_test.go`'s `byName["key-testnet-tpub-valid"]` lookup is
INVERTED rather than deleted, and drives BOTH tpub rows — a parser checking the
string prefix would still have to answer for the second. gui's adopted
`TestFableTestnetXpubKeyRecord` is inverted the same way, on a tpub derived in
the test, and gains a mainnet control at the same origin and depth so the
refusal is provably about the network and not the shape.

---

## Gate at the tip

```
go vet ./...                 rc=1, 10 diagnostics, ALL of them
                             "testing.ArtifactDir requires go1.26 or later
                             (file is go1.25)" — identical to the baseline at
                             f5b068fa (measured: baseline rc=1, same 10 lines).
                             Zero non-ArtifactDir diagnostics.

gofmt -l .                   gui/transaction.go
                             gui/transaction_golden_test.go
                             gui/transaction_txrecord_test.go
                             mt/mt.go
                             mt/mt_test.go
                             — the five-file baseline exactly.

go test (all non-gui pkgs)   green, no failures
  ./md/ ./sysw/ ./mk/ ./bip32/ ./bip39/ ./address/ ./codex32/ ./hashlock/ …

gui-shard-test.sh ./gui/ 24  RESULT: ok -- all 1369 tests ran across 24 shards

go test ./gui/ -run '^TestFable' -v
                             24 PASS, 0 FAIL, 0 SKIP
                             (12 adopted from Appendix A, 3 adopted from lens 1,
                              9 written in the fold)

composerCopyTable            85 bodies, AST scan and count agree
```

Modal-fits, every body this fold added or changed (margin 80):

```
§8a key-less path confirm     344 chars drawn in full, headroom 146
§8f NUMS note                 272 chars drawn in full, headroom 277
§8m two key-less paths        155 chars drawn in full, headroom 418
mixed-lock-bases notice       156 chars drawn in full, headroom 378
§8m empty path                 77 chars drawn in full, headroom 476
```

Firmware, both measured with
`nix develop -c tinygo build -size short -o /dev/null -target pico-plus2
-stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`:

```
f5b068fa (baseline)   code 1612668  data 32172  bss 31148 | flash 1644840  ram 63320
0f435048 (fold tip)   code 1620080  data 32188  bss 31148 | flash 1652268  ram 63336
delta                                                     | +7428          +16
```

---

## Deviations from the brief, with reasons

1. **Item 6's notice is on the CONSENT, not the mapping review.** The mapping
   review is skipped for a flow with no sources; every other shape fact is
   already on the consent; one site, one copy of the rule. (Full reasoning in
   the item, and in the commit message.)
2. **Item 9 gives addresses, not a `Descriptor:` line, on the complex branch.**
   `md` emits no text by design, so no descriptor string exists for a complex
   policy on this device.
3. **The Rust vector is pinned separately, not added to `composeVectorNames`.**
   Measured: a full re-vendor imports md-codec 0.44.0's unported xpub-header
   rewrite and leaves `go test ./md/` GREEN, because the conformance gate never
   asserts `.chains[].descriptor`.
4. **Lens 1's `FABLE_OUT`-gated evidence drivers are not adopted.** They assert
   nothing and skip unless an env var is set.
5. **Commit granularity.** Findings 1, 2 and 3 touch overlapping files; my first
   attempt bundled them, and I reset and rebuilt them as three commits so
   `git diff` per finding means something. One extra commit (`5f24661`) carries a
   gate extension that belongs to findings 1 and 2 jointly.

## Follow-ups for the controller

- **F-? md-codec 0.44.0 is unported in the fork, and no gate can see it.** "A
  rendered xpub's header must agree with its origin" rewrote the xpubs in all 33
  `keyed_compose_*.conformance.json` records at descriptor-mnemonic `24ca7225`.
  The fork's corpus is still pinned at `745e0fd0`, and
  `TestKeyedConformanceAgreesWithRust` parses `.chains[].descriptor` into a
  struct field it never asserts — so a re-vendor would import the change in
  silence. Owning phase: whenever 0.44.0 is ported. Worth asserting
  `.chains[].descriptor` at the same time, or the next drift is invisible too.
- **`md`'s own hint for the key-less cap says a timelock helps, and it does
  not.** `md compose` appends "give one of them a key, a timelock, or fold them
  into one path" — I measured `[keyed, K+older(5), K+after(200)]` refused, so
  the timelock clause is false. Host-side, the controller's half. (The fork's
  §8m body says "A time lock does not help.")
- **The device's §7d same-key refusal and md's `Bind` backstop now both fire, but
  `md/duplicate_keys.go` still answers a different question** (the same SLOT
  twice in one expression, not the same MATERIAL at two slots). That is correct
  for what it is named for; noting it so a future reader does not mistake it for
  the BIP-388 pairwise rule.
