# F-514 duplicate-key warning — independent adversarial execution review

Range: `git diff main..ad64629` in `/scratch/code/shibboleth/seedhammer`
(1 commit, 7 files, +379/-2). Reviewed in a detached worktree at `ad64629`
(`/scratch/code/shibboleth/.tmp/dupkey-review`, removed on completion).
Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`, `TMPDIR=/scratch/code/shibboleth/.tmp`.
Read-only on the fork; every mutation reverted with `git checkout`; final
`git status --porcelain` carried only my own untracked probe files.

**The one question — can a policy be constructed where the warning is wrong in
either direction? YES, in both directions.** One shape Core refuses is shown
with a fundable address and no warning, on a third address surface the fix never
reached (C-1). One shape Core almost certainly accepts is warned about on both
surfaces the fix did reach (I-1).

---

## C-1 — The Inspect-descriptor screen shows addresses for `keyed_wsh_timelock_hashlock` and carries no warning. It is the surface F-514 was measured on.

The warning was added at two call sites: `composerConsentLinesFor`
(`gui/composer_consent.go:216`) and `walletPolicyAddressLines`
(`gui/wallet_policy.go:301`). There is a **third** surface that shows a derived
address for a decoded md1 policy and passes through neither.

`md1GatherFlow` (`gui/md1_gather.go:86`, reached from `mdmkFlow` →
"Inspect descriptor", `gui/gui.go:2785`) hands a complete chunk set to
`gatheredDescriptorFlow` (`gui/md1_gather.go:161`), which routes on
`expandedToDescriptor`:

```go
// gui/md1_gather.go:187-189
		if at, ok := complexAddressSource(collected, keys); ok {
			md1PolicyFlow(ctx, th, tpl, policyIDHeader(collected), at)
			return
		}
```

`md1PolicyFlow` puts that deriver on Button2 and opens the address list
(`gui/md1_inspect.go:166: addressListFlow(ctx, th, at)`). Its body is
`header + md1Summary(tpl)` and nothing else — `md.DuplicateKeySlotChunks` is
never called on this path. The `expandOK` arm (`descriptorFlow` →
`DescriptorScreen.Confirm` → `descriptorAddressFlow`, `gui/gui.go:3249`) has the
same gap.

Measured over every vector in `md/testdata/vectors` (probe rendered the exact
line set `md1PolicyFlow` lays out):

```
vector                                         dup  route          addrs  warned
keyed_wsh_timelock_hashlock                    @1   md1PolicyFlow  YES    NO
```

`complexAddressSource` returns ok, so Button2 is drawn and real mainnet
addresses are listed — the same four the consent screens show:

```
bc1q6h9y4ngdfacaplw0qk67rugxs3vanv0jayk3n3xnhed7uke76zksfqt7py   (Receive 0)
bc1q07uflsu9p3u726guelte2shtezn338xprnq2z9vd6zefj9d37wrqsh5hse   (Change 0)
```

This is not a hypothetical route: the vector is 8 chunks
(`grep -c '^md1' md/testdata/vectors/keyed_wsh_timelock_hashlock.phrase.txt` → 8),
which is exactly the `ErrChunkedUnsupported` arm that calls `md1GatherFlow`, and
F-76's payload priming (`syswPrimeCard`) completes the set with no NFC taps at
all.

**It is also the surface the follow-up itself names.** F-514
(`design/FOLLOWUPS.md:16810-16812`), verbatim:

> The **device** derives it and shows the address with no warning of any kind —
> verified through `policyAddressAt`, the real router **the inspect screen
> uses**, on all three corpus vectors.

The fix landed on the two consent screens and left the inspect screen exactly as
the finding described it. `policyAddressAt`'s own doc
(`gui/wallet_policy.go:319-326`) says `gatheredDescriptorFlow` "makes this choice
too, but as control flow rather than a value" — the two routers were kept in
step for addresses and have now diverged for the warning that qualifies them.

Line violated — the brief's Critical: *"silence on a shape Core refuses that the
device shows an address for."* Also `gui/wallet_policy.go:297-300`'s own
rationale: *"an operator shown an address and nothing else has no way to learn
that before funding it."*

---

## I-1 — The warning fires on a top-level `multi`/`sortedmulti`, where Core's duplicate-key sanity rule does not run at all

`DuplicateKeySlot`'s `case tagWsh, tagSh` arm hands the wrapper's child to
`duplicateInExpression` unconditionally, so a repeated slot inside a **top-level**
`multi`/`sortedmulti` is reported as a duplicate. In Bitcoin Core a top-level
`multi`/`sortedmulti` under `wsh`/`sh` is parsed as a `MultisigDescriptor`, not as
miniscript — and `contains duplicate public keys` comes from
`miniscript::Node::IsSane()` → `CheckDuplicateKey()`, which such a descriptor
never reaches. The corpus's own refused vector is consistent with that: its
`multi` is nested inside `or_i(and_v(...))`, which *is* miniscript.

Counterexample, built and run end to end. Starting from the real
`keyed_wsh_sortedmulti_2of3` descriptor, rewritten to `n=2` with
`sortedmulti(2,@0,@0,@1)` and re-encoded through `md.split` — it re-decodes and
validates cleanly (`validatePlaceholderUsage` admits a repeated slot; only the
first-occurrence ordering is constrained):

```
md1f7as9ps9q2tvyyy5jmpprj5qqcy8pzy9uyw0za5z4eutks2zhqjthgx8egtq4pcwl6u5p2usee8ez2pwt7xpn
md1f7as9psg6r6zsnl2rd0q6gghvalgy07r0ck4wcczrgalt7lhxlg0x6vnl4rdcjgnpya7k5eg7xrcxxccje47g
md1f7as9ps4v487ph7e30f8tpwunu5uevrz28llsnr4qya3jx00arhdt3p75feg52qpln6pv7sc5zk8395ygy5qz
md1f7as9psunhq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqn7wljrhtcrv9f
```

Through the real `walletPolicyConsentLines` (Engrave Wallet Policy), verbatim:

```
Policy-ID: 9e6c32ae3530d57b73add969de01365e
Type: P2WSH 2-of-3 multisig (sorted)
@0 73c5da0a m/48h/0h/0h/2h <0;1>/*
@1 73c5da0a m/48h/0h/1h/2h <0;1>/*

Slot @0 is used twice in one script. Bitcoin Core refuses this descriptor ("duplicate public keys"), so a coordinator may not import it. Check before you fund it.

Receive 0:
bc1qf4jpv99wj36eqez9fzxrzww6sdy97uw5gmgp38t6trqpk5lre8qsv3ttqz
```

`composerConsentLinesFor` prints the identical line. `expandedToDescriptor`
returns `expandOK` for it, so it is a first-class supported shape, not an exotic
one. Note also that the F-218 refusal one screen earlier
(`gui/wallet_policy.go:254`) does **not** catch this: that is the same *key* at
two different *slots*; this is one slot used twice.

**The one premise I could not measure.** Core is not installed and the brief
scopes the settled table to three vectors, so I am reporting the shape, not a
Core verdict. To settle it:

```
bitcoin-cli -regtest getdescriptorinfo "wsh(sortedmulti(2,<tpubA>/<0;1>/*,<tpubA>/<0;1>/*,<tpubB>/<0;1>/*))"
```

If that is accepted — which is what the parser's structure implies — the device
tells an operator that Core refuses a descriptor Core imports, on the two
surfaces the fix did reach, and the copy's central factual claim is false for an
`expandOK` shape. If Core refuses it, this finding evaporates entirely. The same
question covers `sh(sortedmulti(...))` and `sh(wsh(sortedmulti(...)))`, which
take the same arm.

---

## I-2 — The `walletPolicyAddressLines` call site has no test. Deleting it outright leaves 1302/1302 gui tests green.

`TestConsentWarnsOnDuplicateKeys` (`gui/composer_flow_test.go:566`) exercises
`composerConsentLinesFor` only. Nothing in the package touches the second call
site:

```
$ grep -rn "duplicate public keys\|DuplicateKeySlotChunks\|walletPolicyAddressLines" --include=*_test.go gui/
gui/composer_copy_test.go:103:   ... (the copy-table row)
gui/composer_flow_test.go:548,579,581,591   (all inside TestConsentWarnsOnDuplicateKeys)
```

Mutation M4 — the whole block removed from `gui/wallet_policy.go:301-303`,
probe files stashed out of the tree first:

```
$ scripts/gui-shard-test.sh ./gui/ 24
=== wall: 32s ===
RESULT: ok -- all 1302 tests ran across 24 shards
```

Every shard green. The Engrave Wallet Policy surface's half of F-514 is a
hypothesis, not a gate — and it is the surface an operator reaches immediately
before `bundleEngrave` cuts steel (`gui/wallet_policy.go:148-159`). Line
violated: *"a plan may not close while any of its own gates has never been
run"* (repo CLAUDE.md), applied to the guarantee rather than the plan.

---

## I-3 — Two of the diff's three mutation notes are false, and they point a future author at deleting the only test that guards the scoping rule

`md/duplicate_keys_test.go:48-50`:

> MUTATION: make `duplicateInTapTree` count the whole taptree at once instead of
> per leaf, and **both tr rows fail**.

`gui/composer_flow_test.go:563-565`:

> MUTATION: ... make `md.DuplicateKeySlot` ignore the tap-leaf scoping and **the
> taproot cases fail**.

Mutation M1 applied — `return duplicateInTapTree(*b.tree)` →
`return duplicateInExpression(*b.tree)` at `md/duplicate_keys.go:51`:

```
--- md ---
--- FAIL: TestDuplicateKeySlotScopesPerTapLeaf/one_key_in_two_leaves_is_not_a_duplicate
        duplicate_keys_test.go:130: reported @0 duplicated across two LEAVES; ...
FAIL    seedhammer.com/md

--- gui ---
ok      seedhammer.com/gui      0.016s
```

`TestDuplicateKeySlotMatchesBitcoinCore` **passes** under the mutation its own
comment says makes both its tr rows fail. `TestConsentWarnsOnDuplicateKeys`
**passes** likewise. Only the hand-built `TestDuplicateKeySlotScopesPerTapLeaf`
reds.

The diff already contains the reason, at `md/duplicate_keys_test.go:99-101`:

> A probe over every tr vector found NONE where per-leaf and whole-taptree
> counting disagree.

I re-derived that independently rather than taking it: over all 21 tr vectors
carrying a taptree, comparing `duplicateInTapTree(*b.tree)` against
`duplicateInExpression(*b.tree)` — **0 disagreements**. The claim is correct,
and it directly contradicts the other two mutation notes, which were written as
if the corpus rows guarded the scoping.

Why it is more than a wrong comment: the notes tell the next author that the
corpus table already covers tap-leaf scoping, which makes
`TestDuplicateKeySlotScopesPerTapLeaf` look redundant. Delete it and the scoping
rule — the load-bearing decision of the whole feature — has zero coverage while
every corpus row stays green.

The tr rows are *not* vacuous, so the fix is to correct the note, not the test.
Mutation M7 (fold the internal key and every leaf into one expression — the
BIP-388 reading) reds them exactly as a note should describe:

```
--- FAIL: TestDuplicateKeySlotMatchesBitcoinCore/keyed_tr_multi_a
        duplicate=true (slot @0), want false -- Core ACCEPTS: the internal key is outside the miniscript
--- FAIL: TestDuplicateKeySlotMatchesBitcoinCore/keyed_tr_sortedmulti_a
--- FAIL: TestConsentWarnsOnDuplicateKeys/keyed_tr_multi_a
--- FAIL: TestConsentWarnsOnDuplicateKeys/keyed_tr_sortedmulti_a
```

---

## M-1 — `@N` has no referent on the composer consent screen

On the Engrave Wallet Policy screen the slot numbers are defined two lines
above the warning (`@0 73c5da0a m/48h/...`), so `@0` reads. On the composer's own
consent screen, captured in full, the vocabulary is `Path 1` / `Path 2` and no
`@N` appears anywhere else:

```
Script: Segwit (wsh)
Path 1: 3 key(s), custom
Path 2: 2 key(s), custom
...
Slot @1 is used twice in one script. ...
```

The composer's seating prompts do use `@N` (`composerCopySeatPrompt`), several
screens earlier. On this screen the number is a codec index with nothing to
match it against, and "Check before you fund it" is an instruction the operator
cannot act on precisely.

## M-2 — Only the lowest duplicated slot is named, and the shipped vector duplicates two

`keyed_wsh_timelock_hashlock` is
`or_i(and_v(...,multi(2,@0,@1,@2)), and_v(...,multi(1,@1,@2)))` — **@1 and @2
both** appear twice. `duplicateInExpression` returns the lowest, so the warning
says "Slot @1 is used twice" and never mentions @2. Accurate but partial: an
operator who checks @1 and re-mints has not fixed the descriptor. The
lowest-slot rule is right (order-independence is worth having); the *copy* is
what over-claims by naming exactly one.

## M-3 — `composerCopyOriginsChanged` lost its doc comment to the new function

The new function was inserted between `composerCopyOriginsChanged`'s comment
block and its declaration (`gui/composer_copy.go:311-340`). The paragraph about
stale origins and `slotMatchesCard` is now the opening of
`composerCopyDuplicateKeys`'s doc, and `composerCopyOriginsChanged` (line 339)
has no doc comment at all — there is no blank line separating them:

```
// ... Saying "this id changed" there would be false, and saying nothing at all was
// the defect review I-2 constructed.
// composerCopyDuplicateKeys is the §8s warning for a policy whose miniscript
```

## M-4 — The new copy row has no §8s spec entry, so the "verbatim from the spec" test compares it against itself

`TestComposerCopyIsVerbatimFromTheSpec` is documented as comparing "every
shipped string with SPEC §8 word for word". Every other row's `verbatim` column
is transcribed from `design/SPEC_wallet_policy_composer.md` (e.g. line 849 for
`composerCopyIdChanged`). The new string appears in no spec file in
`mnemonic-engrave/design/` — only in `FOLLOWUPS.md` and `POLICY_HARNESS.md` as
prose about Core. For this row the test asserts the function against a literal
written in the same commit, and cannot detect divergence from a §8s that does
not exist. (`assertModalBodyFits` is applied to one body only, line 386, so no
fit gate covers it either — though see below: it does fit.)

## N-1 — `countKeySlots`'s "a nested tr cannot occur" is an unenforced invariant, though nothing reachable exercises it

`md/duplicate_keys.go:106` asserts a property the decoder does not check.
`readNode` gives `tagSh`/`tagWsh`/`tagOrI`/… children of any tag, and
`validateTapScriptTree` inspects only a leaf's **top** tag, so a `tr` buried
inside a leaf's miniscript passes validation. Probed directly:

| shape | verdict | key refs `countKeySlots` saw |
|---|---|---|
| `wsh(tr(@0,{pk(@0)}))` | no dup | **0** |
| `sh(tr(@0,{pk(@0)}))` | no dup | **0** |
| `tr(NUMS,{and_v(v:pk(@0),tr(@0,{pk(@0)})), pk(@1)})` | no dup | **0** |

Every key is invisible. It is a Nit and not a finding because (a) none of those
is a descriptor Core can parse, so none can earn a "duplicate public keys"
verdict, and (b) `script_emit.go` has no `tagTr` case (default →
`ErrScriptUnsupported`, line 560), so `complexAddressSource` probes, fails and
returns `!ok` — the device shows "Complex policy - display only" and no address.
Worth noting only because the sibling walker in the same package,
`walkForPlaceholders` (`md/md.go:956`), *does* handle `trBody`, so the asymmetry
is a trap for the next editor.

## N-2 — `err == nil && dup` is fail-silent, and provably unreachable (brief item 3)

Both call sites run `md.ExpandWalletPolicyChunks(chunks)` on the same slice
before reaching the warning (`gui/composer_consent.go:131`,
`gui/wallet_policy.go:193`), and that function's first statement is
`Reassemble(strs)` (`md/expand.go:103`) — exactly what
`DuplicateKeySlotChunks` calls. A non-nil error here requires `Reassemble` to
be non-deterministic. Failing silent is therefore the right side and costs
nothing today; it is only a hazard if a future caller reaches the block without
having decoded first. No change needed.

---

## Shapes exercised

`DuplicateKeySlot` run over hand-built trees (`md` package, in-package probe):

| # | shape | predicate | expected (Core's rule) | agree |
|---|---|---|---|---|
| S1 | `tr(@0,{pk(@0),pk(@0)})` — internal key AND two leaves | no dup | no dup | yes |
| S2 | `tr(@0,{multi_a(2,@0,@0),pk(@1)})` — repeat in ONE leaf | DUP @0 | dup | yes |
| S3 | depth-3 taptree, @2 in two DIFFERENT leaves | no dup | no dup | yes |
| S4 | depth-3 taptree, @2 twice in ONE leaf | DUP @2 | dup | yes |
| S5 | `sh(wsh(multi(2,@0,@0)))` | DUP @0 | **see I-1** | unsettled |
| S6 | `sh(sortedmulti(2,@0,@0))` — bare sh arm | DUP @0 | **see I-1** | unsettled |
| S7 | `wsh(thresh(2,pk(@0),pk(@1),pk(@0)))` — `variableBody` | DUP @0 | dup | yes |
| S8 | `wsh(and_v(v:pk(@0),c:pk_h(@0)))` — wrapper chain | DUP @0 | dup | yes |
| S9 | `wsh(tr(@0,{pk(@0)}))` — nested tr in an expression | no dup (0 refs) | unparseable | N-1 |
| S10 | `sh(tr(@0,{pk(@0)}))` | no dup (0 refs) | unparseable | N-1 |
| S11 | tr leaf containing a buried `tr` | no dup (0 refs) | unparseable | N-1 |
| S12 | `wsh(andor(pk(@0),pk(@1),pk(@0)))` | DUP @0 | dup | yes |
| S13 | `wsh(multi(2,@0,@0,@0))` — three times | DUP @0 | **see I-1** | unsettled |
| S14 | `tr(NUMS,{multi_a(1,@0),multi_a(1,@0)})` — one key, two leaves | no dup | no dup | yes |
| S15 | `wpkh(@0)` root (default arm) | no dup | no dup | yes |
| S16 | `tr(@0,nil)` key-path only | no dup | no dup | yes |
| S17 | the wsh vector's shape, rebuilt by hand | DUP @1 | dup (measured) | yes |

Real vectors, through `DuplicateKeySlotChunks` and the three device routes
(51 vectors; the `decode-err` rows are single-string legacy fixtures
`Reassemble` declines):

| vector | dup | route | addresses | warned |
|---|---|---|---|---|
| `keyed_wsh_timelock_hashlock` | **@1** | `md1PolicyFlow` | **YES** | **NO** (C-1) |
| `keyed_tr_multi_a` | no | `md1PolicyFlow` | YES | n/a |
| `keyed_tr_sortedmulti_a` | no | `md1PolicyFlow` | YES | n/a |
| 26 further `md1PolicyFlow` vectors | no | `md1PolicyFlow` | YES | n/a |
| 9 `descriptorFlow` vectors (wpkh, sh, sortedmulti, tr key-only) | no | `descriptorFlow` | yes | n/a |
| synthesized `wsh(sortedmulti(2,@0,@0,@1))` | **@0** | `descriptorFlow`/consent | YES | **YES** (I-1) |

Scoping probe, answering the brief's explicit ask: **21** tr vectors carry a
taptree; per-leaf and whole-taptree counting disagree on **0** of them. The
diff's claim that no vendored vector separates the two scopings is correct, and
the hand-built fixture is justified.

Screen fit (brief item 4), measured on `sh2DisplaySize` = 480×320,
`lineWidth` 464, viewport 224 px:

| line | height | fits a page alone |
|---|---|---|
| the warning (162 chars) | 59 px | yes |
| `composerCopyIdChanged` | 41 px | yes |
| a bech32 address | 41 px | yes |

`confirmReviewScreen` pages gap-free and draws the pager whenever a second page
exists, and the warning precedes the addresses in the line list, so an operator
must page past it to reach them. It is not on the first frame — but neither are
the addresses it qualifies, on either surface. No clipping, no lost text.

## Mutations

| # | mutation | tests that red | matches the diff's claim |
|---|---|---|---|
| M1 | `duplicateInTapTree` → whole taptree at once | `TestDuplicateKeySlotScopesPerTapLeaf/one_key_in_two_leaves` only | **NO** — both notes claim the tr corpus rows red; they pass (I-3) |
| M2 | scope a wsh expression per `or_i` arm | `…MatchesBitcoinCore/keyed_wsh_timelock_hashlock`, `TestConsentWarnsOnDuplicateKeys/keyed_wsh_timelock_hashlock` | yes |
| M3 | remove the block from `composerConsentLinesFor` | `TestConsentWarnsOnDuplicateKeys` | yes |
| M4 | remove the block from `walletPolicyAddressLines` | **none** — 1302/1302 green | untested call site (I-2) |
| M5 | `countKeySlots` drops `multiKeysBody` | `…MatchesBitcoinCore/keyed_wsh_timelock_hashlock`, `…ScopesPerTapLeaf` ×2, `TestConsentWarnsOnDuplicateKeys/keyed_wsh_timelock_hashlock` | yes |
| M6 | report first-in-traversal instead of lowest slot | `…ScopesPerTapLeaf/the_lowest_duplicated_slot_is_reported` (`got @3, want @1`) | yes |
| M7 | fold the tr internal key + all leaves into one expression | `…MatchesBitcoinCore` ×2 tr rows, `…ScopesPerTapLeaf`, `TestConsentWarnsOnDuplicateKeys` ×2 tr rows | the note M1 *should* have carried |

Housekeeping at `ad64629`, unmutated: `go build ./...` clean;
`go test ./md/` ok; `gofmt -l .` lists exactly the known five-file baseline
(`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`);
`go vet ./md/ ./gui/` reports only the known `testing.ArtifactDir` go1.25 gap.

## Counts

**1 Critical / 3 Important / 4 Minor / 2 Nit — NOT GREEN.**

C-1 is the blocker: the fix does not reach the surface the follow-up that
motivated it was measured on, and that surface derives a fundable address for
the one corpus policy Core is measured to refuse. I-1 is the mirror image and
needs one `getdescriptorinfo` call to settle or dismiss. I-2 and I-3 are gate
defects rather than behaviour defects — one guarantee with no test, and two
recorded mutations that do not red — and both would have been caught by running
the notes rather than writing them.
