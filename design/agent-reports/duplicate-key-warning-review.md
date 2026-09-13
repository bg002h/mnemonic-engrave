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

---

# Addendum — fold re-review at 22bace1

Range: `git diff ad64629..22bace1`. The branch moved, so I separated the two
given commits from the fold and reviewed only the latter:
`git diff 0e6ade8..22bace1` is exactly the eight files named
(`gui/composer_consent.go`, `composer_copy.go`, `composer_copy_test.go`,
`composer_flow_test.go`, `md1_gather.go`, `wallet_policy.go`,
`md/duplicate_keys.go`, `duplicate_keys_test.go`, +330/−60);
`git diff ad64629..0e6ade8` is F-527/F-528 only (`composer_flow.go`,
`composer_stub.go`, `composer_stub_test.go`, `freetext_sizeproof_test.go`) and I
treated it as given. Fresh detached worktree at `22bace1`, every mutation
reverted, tree left clean, worktree removed.

Scope was "did the fold close each finding, and did it introduce a new defect" —
not a fresh audit. **Two new Importants, both introduced by the fold.**

## Verdicts on the round-1 findings

| finding | verdict |
|---|---|
| C-1 inspect surface silent | **CLOSED** (behaviour) — but the gate for it is in the wrong place, see I-4 |
| I-1 false Core claim on a multisig | **CLOSED** in direction; the replacement sentence carries a new false claim, see I-5 |
| I-2 untested `walletPolicyAddressLines` | **CLOSED** — verified by mutation on all three producers |
| I-3 false mutation note | **CLOSED** in `md`; the same class reappears in `gui`, see M-5 |
| M-1, M-2, M-3, M-4 | not folded, as stated; M-1 now applies to both sentences |

**C-1 — the surface is the one I traced, and it is the only one of its kind.**
`policyIDHeader` (`gui/md1_gather.go:207`) feeds `md1PolicyFlow`'s `header`
parameter at `gui/md1_gather.go:188`, and `md1PolicyFlow` lays that header out
above the Button2 address list (`gui/md1_inspect.go:123-130`, `:166`). There are
exactly two callers of `md1PolicyFlow`: that one, and `md1DisplayFlow`
(`gui/md1_inspect.go:107`), which passes `nil, nil` — no header **and** no
address deriver, so it draws no address and needs no warning. There are two
callers of `policyIDHeader` and both are tests plus that one site. No sibling
entry was missed this time. Rendered for real, the header now returns:

```
Policy id: 799c164288c19c7f5ac03bdb5635d978
Slot @1 is used twice in one script. Bitcoin Core refuses this descriptor ("duplicate public keys"), so a coordinator may not import it. Check before you fund it.
```

**I-2 — closed, verified.** Deleting the `DuplicateKeySlotChunks` block from each
producer in turn reds exactly that producer's row (mutation table below). The
whole-suite hole I measured last round is gone.

**I-3 — closed in `md`, and the correction is accurate.** Re-ran the mutation on
the fold: `duplicateInTapTree` → `duplicateInExpression` reds only
`TestDuplicateKeySlotScopesPerTapLeaf/one_key_in_two_leaves_is_not_a_duplicate`;
both tr rows in `TestDuplicateKeySlotMatchesBitcoinCore` and all four rows in
`TestConsentWarnsOnDuplicateKeys` stay green — which is what
`md/duplicate_keys_test.go:48-56` now says, including the "do not delete it as
redundant" clause.

---

## I-4 — the C-1 gate asserts `policyIDHeader`, not the screen. Dropping the warning from `md1PolicyFlow` leaves 1306/1306 green.

`TestEveryAddressSurfaceCarriesTheDuplicateWarning/inspect descriptor`
(`gui/composer_flow_test.go:653-656`) asserts on `policyIDHeader(chunks)` — the
producer's return value. Its own comment states the assumption:

> policyIDHeader is what md1PolicyFlow lays out above the address button, so
> this is the line set that screen actually draws.

True at `22bace1`, and nothing asserts it. Mutation MF1b — one line in the
consumer, keeping the policy id and silently discarding everything after it:

```go
// gui/md1_inspect.go:123
	if len(header) > 1 { // MUTATION MF1b: only the id line survives
		header = header[:1]
	}
```

```
$ scripts/gui-shard-test.sh ./gui/ 24
=== wall: 24s ===
RESULT: ok -- all 1306 tests ran across 24 shards
```

The Inspect-descriptor screen shows addresses for `keyed_wsh_timelock_hashlock`
with no warning again — C-1 exactly — and every test passes. The coupling is
only *partly* protected by accident: dropping the header **entirely** does red,
but on a pre-existing test and for the wrong reason
(`TestComplexPolicyScreenNamesWhichWalletID`, `policy_address_test.go:320`,
*"the complex-policy screen does not name the wallet id it shows"*). Nothing
anywhere asserts that the warning survives the consumer.

This is I-2's defect moved down one layer. I-2 was "a surface with no test"; the
fix asserted the producer for all three, and for two of them the producer *is*
the line set the caller renders, while for the third there is a consumer in
between that can drop it. The test that would close this is small — the three
lines of `md1_inspect.go:123-130` are extractable, or assert on a drawn frame —
and it would catch M-6 below at the same time.

## I-5 — the replacement multisig sentence overstates, and contradicts a line on its own screen

`composerCopyDuplicateKeys` for `DuplicateInMultisig`:

> Slot @%d fills two seats of this multisig, so one key can **meet the threshold
> alone**.

That holds only when the repeated slot's multiplicity reaches `k`. It was
measured on `sortedmulti(2,A,A,B)`, where two seats *are* the threshold, and then
stated universally — which is the shape of I-1 reproduced inside its own fix.

Counterexample, built by rewriting the real `keyed_wsh_sortedmulti_2of3` to
`sortedmulti(3,@0,@0,@1,@2)` (k=3, four seats, @0 twice, `n` unchanged at 3; it
re-encodes through `md.split` and re-decodes cleanly — `validatePlaceholderUsage`
admits it) and run through the real `walletPolicyConsentLines`:

```
Policy-ID: 3bd80bfbe639910e8558eb076b61d65e
Type: P2WSH 3-of-4 multisig (sorted)
@0 73c5da0a m/48h/0h/0h/2h <0;1>/*
@1 73c5da0a m/48h/0h/1h/2h <0;1>/*
@2 73c5da0a m/48h/0h/2h/2h <0;1>/*

Slot @0 fills two seats of this multisig, so one key can meet the threshold alone. Check this is what you meant before you fund it.
```

Two of three is not the threshold. @0 alone signs twice and is still one
signature short; the policy needs @0 plus one other. The screen says "3-of-4" and
"one key can meet the threshold alone" **five lines apart**, and one of them is
wrong.

The harm is real and worth stating — the wallet is satisfiable by two independent
holders while its label says three — so the answer is not to drop the sentence.
What the predicate actually knows is "this slot fills more than one seat"; what
it does not know, without comparing multiplicity against `k`, is whether that is
sufficient on its own. Either compare them and say the true thing in each case,
or state only the part that is always true (fewer holders than the label names).
`TestDuplicateWarningNamesTheRightHarm` pins the overstating clause verbatim
(`says: "meet the threshold alone"`), so it cannot catch this.

By the fold's own standard, in its own commit
(`gui/composer_copy.go:333-336`): *"a false sentence on the screen that consents
to steel is worse than no sentence."*

## M-5 — the gui half of the I-3 correction reintroduces a false mutation note

`gui/composer_flow_test.go:687-688`:

> MUTATION: make `kindForRoot` always return `DuplicateInMiniscript` and the
> multisig rows fail.

`TestDuplicateWarningNamesTheRightHarm` (`:689`) never calls `kindForRoot`,
never decodes a policy and never calls `DuplicateKeySlot` — it passes the kind in
by hand (`composerCopyDuplicateKeys(0, tc.want)`). Measured:

```
--- MF3: kindForRoot always InMiniscript ---
md:  FAIL TestDuplicateKindSplitsByWhatCoreDoes  (3 multisig rows)
gui: ok   seedhammer.com/gui   0.068s
```

Minor rather than Important, and the difference from I-3 matters: the identical
note on `md/duplicate_keys_test.go:180` is **true**, and
`TestDuplicateKindSplitsByWhatCoreDoes` really does guard the discriminant. So
the rule has coverage; only the gui note points at a test that cannot provide it.
The gui test's real guarantee is "each kind gets its own sentence and neither
carries the other's claim", which is worth having — the note should say that.

## M-6 — the Inspect screen hard-chunks the warning at 20 bytes, mid-word

`md1PolicyFlow` runs every header line through `chunkString(ln, 20)`
(`gui/md1_inspect.go:126`), a blind byte cut with no word boundaries
(`gui/mk1_inspect.go:21-31`). That convention suits the two things the screen
carried before — a 32-hex policy id and `@N` origin rows — and the warning is the
first prose ever put on it. What the operator reads:

```
 3 |Slot @1 is used twic|
 4 |e in one script. Bit|
 5 |coin Core refuses th|
 6 |is descriptor ("dupl|
 7 |icate public keys"),|
 8 | so a coordinator ma|
 9 |y not import it. Che|
10 |ck before you fund i|
11 |t.|
```

Nine centred fragments, seven of them splitting a word. The same string
word-wraps correctly on both consent surfaces, which pass whole lines to
`widget.Labelw`. It also costs ~207 px of a 224 px viewport, so it very nearly
fills the page on its own. No text is lost (paging is gap-free), which is why
this is Minor and not more — but the warning that F-514 exists to deliver is the
one line on the device rendered worse than any other, and the test in I-4 cannot
see it because `strings.Contains` runs on the unchunked producer output.

## N-3 — `kindForRoot` would misclassify `multi_a` under `wsh`/`sh`, which is unreachable

`kindForRoot` (`md/duplicate_keys.go:119`) maps `tagMultiA`/`tagSortedMultiA` to
`DuplicateInMultisig`. Under `tr` that never matters — the `tagTr` arm hardcodes
`DuplicateInMiniscript` and never consults it, which is correct. Under `wsh`/`sh`
it would be reached, and would be wrong twice over: Core cannot parse
`wsh(multi_a(...))` at all, so neither sentence is true of it. Unreachable:
`script_emit.go:520-527` refuses `tagMultiA`/`tagSortedMultiA` when `!e.tap`, so
`complexAddressSource` probes, fails, and the device shows
"Complex policy - display only" with no address and no warning. Nit only.

---

## The three questions, answered

**(1) Is the discriminant right for the shapes you did not test? Yes, for every
one you named.** Run through `DuplicateKeySlot` directly:

| shape | kind | Core | right |
|---|---|---|---|
| `wsh(sortedmulti(2,@0,@0,@1))` | InMultisig | ACCEPTED (your measurement) | yes |
| `wsh(multi(2,@0,@0,@1))` | InMultisig | ACCEPTED (your measurement) | yes |
| `sh(wsh(sortedmulti(2,@0,@0,@1)))` | InMultisig | MultisigDescriptor | yes |
| **`sh(multi(2,@0,@0,@1))` bare** | InMultisig | MultisigDescriptor | yes |
| **`sh(sortedmulti(2,@0,@0,@1))` bare** | InMultisig | MultisigDescriptor | yes |
| **`wsh(v:multi(2,@0,@0))`** one wrapper deep | InMiniscript | miniscript → IsSane | yes |
| **`wsh(and_v(v:multi(2,@0,@0),pk(@1)))`** | InMiniscript | miniscript → IsSane | yes |
| `wsh(thresh(2,pk@0,pk@1,pk@0))` | InMiniscript | miniscript → IsSane | yes |
| `wsh(or_i(multi(2,@1,@2),multi(1,@1,@2)))` | InMiniscript | REFUSED (measured) | yes |
| `wsh(and_v(pk(@0),pk(@0)))` | InMiniscript | REFUSED (measured) | yes |
| **`tr(NUMS,{multi_a(2,@0,@0), pk(@1)})`** | InMiniscript | tapscript IS miniscript → refused | yes |
| **`tr(NUMS,{sortedmulti_a(2,@0,@0), pk(@1)})`** | InMiniscript | same | yes |
| `tr(NUMS,{sortedmulti_a(2,@0,@0)})` sole leaf | InMiniscript | same | yes |
| `tr(NUMS,{and_v(v:pk@0,pk@0)})` sole leaf | InMiniscript | same | yes |
| `wsh(multi_a(2,@0,@0,@1))` | InMultisig | unparseable | **no** — N-3, unreachable |

The taproot cases are right for a reason worth writing down: the `tagTr` arm
never calls `kindForRoot`, and it must not — `multi_a`/`sortedmulti_a` in a
tapleaf *are* miniscript to Core (there is no MultisigDescriptor in tapscript),
so hardcoding `DuplicateInMiniscript` there is the correct answer, not a
shortcut. A future edit that "unifies" the two arms by routing tr through
`kindForRoot` would silently start telling taproot operators Core imports a
descriptor it refuses. Nothing pins that today.

**(2) Does `policyIDHeader` reach every Inspect route? Yes — see C-1 above.** Two
`md1PolicyFlow` callers, one passes the header and an address source, the other
passes neither and shows no address. No sibling missed. But the assertion that
keeps it that way is in the wrong place (I-4).

**(3) Do the new tests fail on the guarantees they name?** Yes, except the one
note in M-5. Full mutation table below.

## Leaving the `expandOK` arm open is defensible, and here is the mechanism

Not a judgement call — `scriptForTemplate` (`gui/md1_expand.go:102-141`) bounds
it. `expandOK` requires `PolicySingle` (wpkh / pkh / tr key-path-only /
sh(wpkh) — one key slot, so `duplicateInExpression` can never report a repeat) or
`PolicySortedMulti` (wsh / sh(wsh) / bare sh — a **top-level sortedmulti**, which
`kindForRoot` maps to `DuplicateInMultisig` by construction). Every other shape —
unsorted `multi`, `multi_a`, `sortedmulti_a`, any taptree, any miniscript —
returns `false` and routes to `md1PolicyFlow`, which now warns.

So **the `expandOK` arm can only ever carry `DuplicateInMultisig`, never
`DuplicateInMiniscript`.** It cannot reproduce C-1: there is no shape it can show
an address for that Core refuses. What it withholds is the multisig warning, and
the same policy carries that warning on the Engrave Wallet Policy consent screen
— the surface that actually precedes cutting steel. Ship it and file it.

One condition, worth putting in the follow-up so the deferral has a trip-wire
rather than an expiry date: **this argument dies the moment `scriptForTemplate`
grows an arm for plain `multi` or for any miniscript shape.** On that day a
Core-refused descriptor reaches `descriptorFlow` unwarned and C-1 is back, on the
one surface with no chunk set to run a predicate over. A line in
`scriptForTemplate` saying so costs nothing now and is the only thing that will
be read at the right moment.

## Mutation table

| # | mutation | result | matches the note |
|---|---|---|---|
| MF2a | delete the block in `composerConsentLinesFor` | `TestConsentWarnsOnDuplicateKeys/keyed_wsh_timelock_hashlock`, `…/composer consent` red | yes |
| MF2b | delete the block in `walletPolicyAddressLines` | `…/wallet policy consent` red | yes |
| MF2c | delete the block in `policyIDHeader` | `…/inspect descriptor` red | yes |
| MF1 | `md1PolicyFlow` ignores `header` entirely | reds, but on a pre-existing test and for the id, not the warning | — |
| **MF1b** | **`md1PolicyFlow` keeps only `header[:1]`** | **1306/1306 green** | **I-4** |
| MF3 | `kindForRoot` always `DuplicateInMiniscript` | md `TestDuplicateKindSplitsByWhatCoreDoes` red ×3; **gui green** | md yes, **gui no (M-5)** |
| M1 | `duplicateInTapTree` → whole taptree | only `…ScopesPerTapLeaf/one_key_in_two_leaves` red | yes — the corrected note is accurate |

Housekeeping at `22bace1`, unmutated: `go build ./...` clean, `go test ./md/` ok,
tracked tree clean after every revert.

## Counts

**0 Critical / 2 Important / 2 Minor / 1 Nit — NOT GREEN.**

Nothing on the device is wrong today except I-5's sentence. I-4 is a gate in the
wrong place, and it is the same gate that was missing last round, one layer
further down — worth fixing now while the reason is in front of you rather than
after the next consumer edit. I-5 is a false claim on a consent screen, held to
the standard the fold's own commit sets. M-3 and M-4 from the first pass remain
open as agreed, and M-1 now reads against both sentences.

---

# Addendum 2 — second fold at 19630dd

Range `git diff 22bace1..19630dd`, six files (+161/−19). Fresh detached worktree,
every mutation reverted, tree clean, worktree removed.

**Core was available this round and I used it.** `/usr/local/bin/bitcoind`
v25.0.0, a throwaway regtest datadir under `/scratch/code/shibboleth/.tmp` on an
isolated RPC port, stopped and deleted afterwards. (A pre-existing `bitcoind`
from Sep 12 on the default datadir was left untouched.) That turned your
question (3) from reasoning into measurement — and the measurement says your
belief is wrong.

## Verdicts

| finding | verdict |
|---|---|
| I-4 gate asserted the producer | **CLOSED** — my own `h = h[len(h)-1:]` mutation now reds, and so does a content mutation |
| I-5 multisig sentence overstates | **CLOSED for k ≥ 2**; one narrow boundary remains at k = 1, see M-7 |
| M-6 mid-word chunking | **PARTLY** — `wrapWords` is correct at every boundary, but the sentence now spans two pages, see I-8 |
| M-5 (round 2) false gui mutation note | still open, unchanged at `gui/composer_flow_test.go:713-714` |
| M-3, M-4 (round 1) | still open, as agreed |

**I-4 — closed, three ways.** The gate now drives `gatheredDescriptorFlow`, so
everything between the operator's tap and the pixels is inside the test.

- MG2, the exact mutation that defeated the previous version — `h :=
  policyIDHeader(collected); h = h[len(h)-1:]` — now **reds**
  `…/inspect descriptor`.
- MG1, mutating the warning's **content** rather than its presence
  ("is used twice in one script" → "repeats a key") also reds it, plus
  `TestComposerCopyIsVerbatimFromTheSpec`. The needle is a hardcoded literal, so
  this subtest is content-bound; note the other two subtests compare against
  `want := composerCopyDuplicateKeys(slot, kind)` and are presence-bound only —
  they would not have caught MG1 on their own.
- MG3, deleting the warning from `policyIDHeader`, reds it too, which answers
  your "is it passing on some other line of the frame": no. The phrase reaches
  the frame from the warning and from nothing else.

**I-5 — closed at every multiplicity I could reach except k = 1.** Measured on
Core 25.0.0, all ACCEPTED, so all four need a true sentence and none may claim
Core refuses: `wsh(sortedmulti(2,A,A,B))`, `wsh(sortedmulti(3,A,A,B,C))`,
`wsh(sortedmulti(2,A,A,A))`, `wsh(multi(1,A,A))`, `sh(multi(2,A,A,B))`. The new
wording is true for a slot appearing three times (k=3, one distinct key meets it)
and for the 3-of-4 that falsified the old one. See M-7 for k = 1.

---

## I-6 — your question (3): the reasoning is wrong. Core 25 ACCEPTS a repeated key inside one tap leaf, and the device says it refuses it.

You wrote that you believe `DuplicateInMiniscript` is right for a tapleaf repeat
because "Core treats tapscript `multi_a` as miniscript". Core 25.0.0 does not —
it has no tapscript miniscript at all:

```
$ getdescriptorinfo "tr(A,and_v(v:pk(B),pk(C)))"
error: Miniscript expressions can only be used in wsh
```

So `multi_a`/`sortedmulti_a` in a tapleaf go through a dedicated descriptor path,
not the miniscript parser, and `CheckDuplicateKey` never runs — the same reason a
top-level `multi` under `wsh` escapes it. Measured:

```
tr(A, multi_a(2,B,B))                ACCEPTED
tr(A, sortedmulti_a(2,B,B))          ACCEPTED
tr(A, multi_a(2,A,A,B))              ACCEPTED
tr(A,{multi_a(2,B,B,K),pk(K)})       ACCEPTED
tr(A,{pk(B),pk(B)})  two leaves      ACCEPTED     (confirms the scoping rule at Core level)
wsh(and_v(v:pk(A),pk(A)))            "... is not sane: contains duplicate public keys"
```

The `tagTr` arm (`md/duplicate_keys.go:65-71`) hardcodes
`DuplicateInMiniscript`, so the device asserts a refusal Core does not make.
Device-side counterexample, built by rewriting `keyed_tr_sortedmulti_a`'s leaf to
`sortedmulti_a(2,@0,@0,@1)` (re-encodes through `md.split`, re-decodes clean,
`complexAddressSource` ok):

```
Inspect header:
  Slot @0 is used
  twice in one script.
  Bitcoin Core refuses
  this descriptor
  ("duplicate public
  keys"), so a
  coordinator may not
  import it. Check
  before you fund it.
  Policy id: f3dd179e77d883e3c16d066baec7fe5a

Engrave Wallet Policy consent: same sentence, then
  Receive 0: bc1pk5pwjf7ly93ljl6zwwx97tjm403vvp0s2katwl3l06cr72tp9p9qqye0fg
```

Core 25.0.0 imports the equivalent descriptor. This is I-1 recurring on the
taproot arm, and it is the same defect on all three surfaces at once.

**Being fair about versions.** Core 26+ added tapscript miniscript, and on that
version a `multi_a` repeat may well be refused. That cuts both ways: the device
asserts a version-specific verdict as a flat fact, on the *one* version the fold
measured against, where it is false. The harm the fold identified for the
multisig case — this wallet needs fewer distinct keys than its label says — is
true of a tapleaf `multi_a(2,@0,@0)` at every Core version. So the shape that
survives the question is `DuplicateInMultisig` for a tapleaf whose top is
`multi_a`/`sortedmulti_a`, and `DuplicateInMiniscript` only for a genuine
miniscript leaf — where, on Core 25, the descriptor is refused anyway but for a
different reason ("Miniscript expressions can only be used in wsh"), so even
there the parenthetical is wrong today.

You were right that the current answer is "by accident rather than by decision".
The accident is not benign. Whatever you decide, pin it — the pin you were unsure
how to write is now a measurement.

## I-7 — the F-530 trip-wire enumerates 3 of `PolicyKind`'s 6 values, and misses all three that would kill the argument

`md.PolicyKind` has six values (`md/md.go:1186-1191`): `PolicySingle`,
`PolicyMulti`, `PolicySortedMulti`, `PolicyMultiA`, `PolicySortedMultiA`,
`PolicyComplex`. `TestScriptForTemplateAdmitsOnlyTwo` loops over three
(`gui/md1_expand_test.go:426-428`):

```go
	for _, policy := range []md.PolicyKind{
		md.PolicySingle, md.PolicySortedMulti, md.PolicyMulti,
	} {
```

The three it omits — `PolicyMultiA`, `PolicySortedMultiA`, `PolicyComplex` — are
exactly the miniscript-bearing ones, whose duplicates are
`DuplicateInMiniscript` and which are therefore the only ones that can reproduce
the Critical on the unwarned `expandOK` route. The one it *does* catch,
`PolicyMulti`, is the one whose duplicate is `DuplicateInMultisig` and so the one
that would **not** kill F-530's argument. The trip-wire is inverted.

Measured. MG5 adds precisely the arms the comment says must never appear:

```go
	case md.PolicyComplex:  // wsh miniscript
		if tpl.Root == md.ScriptWsh { return bip380.P2WSH, bip380.SortedMulti, true }
	case md.PolicyMultiA:   // tr
		if tpl.Root == md.ScriptTr { return bip380.P2TR, bip380.SortedMulti, true }
```

```
$ go test ./gui/ -run TestScriptForTemplateAdmitsOnlyTwo
--- PASS
$ scripts/gui-shard-test.sh ./gui/ 24
RESULT: ok -- all 1307 tests ran across 24 shards
```

Green on the exact change it exists to stop. (MG4, adding a `PolicyMulti` arm,
does red — so the test works, on the one kind that matters least.) The fix is the
pattern this repo already uses in `gui-shard-test.sh`: enumerate from the source
and assert exhaustiveness, rather than hand-listing. Iterating
`PolicySingle..PolicyComplex` would have caught MG5 and would pick up a seventh
kind automatically.

## I-8 — `wrapWords` is correct, but the warning now spans two pages and the instruction is below the fold

`wrapWords` itself is sound at every boundary you asked about — no line over the
width, no empty lines, no content lost or reordered:

| input | lines |
|---|---|
| `""` | 0 |
| only spaces | 0 |
| word exactly 20 | 1 × 20 |
| word 21 | `20`, `1` |
| word 40 | `20`, `20` — and the empty remainder does **not** emit a blank line |
| word 40 + `tail` | `20`, `20`, `tail` |
| `hi` + word 45 | `hi`, `20`, `20`, `5` |
| 19 + 1-char word | `19`, `h` — joins only when `len+1+len <= width` |
| tabs / double spaces | collapsed by `strings.Fields` |

(`width <= 0` would spin forever at `word[:width]`, and a multi-byte rune could
be split by the hard-cut fallback. Both unreachable — one call site, `20`, and
ASCII-only copy enforced by `TestComposerCopyIsDrawable`. Nit, not a finding.)

**The screen is what does not work.** Reproducing `md1PolicyFlow`'s own layout
arithmetic on `sh2DisplaySize`, viewport 224 px:

```
PAGE 1 (7 lines)          PAGE 2 (7 lines)
 |Slot @1 is used|         |import it. Check|
 |twice in one script.|    |before you fund it.|
 |Bitcoin Core refuses|    |Policy id: 799c16428|
 |this descriptor|         |8c19c7f5ac03bdb5635d|
 |("duplicate public|      |978|
 |keys"), so a|            |Complex policy - can|
 |coordinator may not|     |not display safely.|
```

Page 1 ends mid-clause on "so a coordinator may not", and **"Check before you
fund it." is below the fold** — the one actionable sentence in the warning, on
the surface the Critical was about. `assertModalBodyFits`'s own error text names
this class: *"a funds-critical modal must not require the operator to scroll to
reach a warning they may act without."* And `policyIDHeader:211` now says "a
warning below the fold is a warning unread" as the reason the warning leads — the
fix moved the sentence's *start* above the fold and left its *instruction* below
it. The gate asserting the opening rather than the tail is calibrated to that,
rather than catching it.

Width is the whole of it. Measured, warning 162 chars, viewport 224 px:

| pre-wrap width | lines | height | fits one page |
|---|---|---|---|
| whole sentence → `Labelw` | 1 label | **59 px** | yes, 165 px to spare |
| **20 (shipped)** | 9 | **261 px** | **no** |
| 24 | 7 | 203 px | yes |
| 32 | 6 | 174 px | yes |
| 44 | 4 | 116 px | yes |

20 is the only width tested that does not fit, and it misses by one line. **But
do not simply raise it to 24** — `md1PolicyFlow` re-chunks any line longer than
20 (`gui/md1_inspect.go:125`), so a 24-wide pre-wrap gets byte-cut back to 20 and
M-6's mid-word fragments return. The constraint is joint: the consumer's
threshold and the pre-wrap width have to move together, or prose has to bypass
the chunker entirely — which is what the two consent surfaces do, and why the
same string renders in 59 px there and 261 px here.

## C-2 — PRE-EXISTING, NOT this fold, and outside the scope you set: the flat route shows a different address than the policy on the card

Reporting it because it is funds-critical, not because it belongs to this review.
Found while building I-5's and M-7's fixtures.

`expandedToDescriptor` projects a repeated-slot multisig onto a
`*bip380.Descriptor` with **one key per slot**, discarding the repeat, while
`tpl.K` is carried over unchanged. `policyAddressAt` tries that flat route first,
so the address shown is the hash of a script the card does not encode:

| policy on the card | `policyAddressAt` (shown) | `complexAddressSource` (emitted from the real tree) |
|---|---|---|
| `wsh(sortedmulti(1,@0,@0,@1))` | `bc1qvcrd8s7ulynmvr6qk44r6scq4daefmq5vh46qg0yec6c6tv0vjeqhw9yuw` | `bc1qxdqrua39qzx56cl3qezgc9y7gem54lxktkvje0w3x05v63aqc8tsd7vp5p` |
| `wsh(sortedmulti(2,@0,@0,@1))` | `bc1qf4jpv99wj36eqez9fzxrzww6sdy97uw5gmgp38t6trqpk5lre8qsv3ttqz` | `bc1qc6ssvugf29fq9z78d9559tuyplxlf06cwt57zc8uquamnte00w7qeqfts6` |
| `wsh(sortedmulti(3,@0,@0,@0,@1))` | derivation error → "Address derivation failed. Do not engrave this card." | `bc1qwg8yp5h5ewnmsx03tpq8xqxeg3zj8hg3jrfkek36fkw2t0zs9n5s0xkwkv` |

Two routes in the same binary disagree, and the emitter is the one that reads the
encoded tree faithfully — the screen labels it "Type: P2WSH 1-of-3 multisig
(sorted)" while showing a 1-of-2's address. The `k > distinct slots` case is
caught by accident, by an arithmetic error in the address layer, and refuses
safely. The other two do not.

No shipped encoder produces such a card — `EncodeMultisig` always emits
`[0..n-1]` — and F-218's `DuplicateKeySlots` does not catch it (that is one key
in two slots; this is one slot in two seats). But a foreign card is exactly the
threat model `walletPolicyConsentLines` exists for, by its own comment. Worth its
own follow-up; **this fold's verdict should not wait on it**, and the addresses I
quoted for I-5 last round were the projected ones.

## M-7 — the multisig sentence is false at k = 1, and only there

You asked about a 1-of-n. Built `wsh(sortedmulti(1,@0,@0,@1))`; Core 25 ACCEPTS
it; the consent screen renders:

```
Type: P2WSH 1-of-3 multisig (sorted)
...
Slot @0 fills more than one seat here, so this multisig needs fewer separate keys than its k-of-n says.
```

A 1-of-3 needs exactly one separate key to spend, duplicate or not, so "fewer" is
false — the count is unchanged. The general rule: a slot filling `m ≥ 2` seats
reduces the distinct keys needed from `k` to `max(1, k−m+1)`, which is a
reduction **iff k ≥ 2**. So the sentence is true for every k ≥ 2 (including your
three-times case) and false only at k = 1.

Minor rather than Important: a 1-of-n with a duplicate is degenerate, no encoder
builds one, and there *is* a real harm at k=1 — two distinct keys of redundancy
where the label implies three — it is simply not the harm the sentence names. A
false alarm, not a missed one. Recording it because you asked and because it is
the last boundary.

## M-8 — the I-5 retraction reached the string but not three of the four places that state the rule

`grep -rn "threshold alone"` at `19630dd`:

- `gui/composer_copy.go:340-341` — the doc block **above** the function still
  says "one key filling two seats of a threshold can meet that threshold alone,
  so a 2-of-3 with one key twice is a 1-of-2 wearing a 2-of-3's label", while
  lines 350-356 of the same comment retract it. One comment block asserts and
  retracts the same claim.
- `gui/composer_flow_test.go:710` — same claim, in the doc of the test that now
  asserts the replacement wording.
- `md/duplicate_keys_test.go:177` — same claim.

(`gui/wallet_policy.go:257` and `composer_backleg_test.go:210` are F-218's
two-slots refusal and are fine.)

## N-4 — the trip-wire's failure message prints an integer

`md.PolicyKind` has no `String()`, so MG4's failure reads *"scriptForTemplate now
admits 1"*. The message is otherwise the best kind — it explains the consequence
and names the two acceptable responses — and one `String()` method would finish
it.

---

## Mutation table

| # | mutation | result | verdict |
|---|---|---|---|
| MG1 | reword the miniscript warning (content, not presence) | `…/inspect descriptor` + `TestComposerCopyIsVerbatimFromTheSpec` red | gate is content-bound ✓ |
| MG2 | `h = h[len(h)-1:]` in `gatheredDescriptorFlow` | `…/inspect descriptor` red | **I-4 closed** ✓ |
| MG3 | delete the warning from `policyIDHeader` | `…/inspect descriptor` red | no false match elsewhere on the frame ✓ |
| MG4 | add a `PolicyMulti` arm to `scriptForTemplate` | `TestScriptForTemplateAdmitsOnlyTwo` red | trip-wire works for this kind ✓ |
| **MG5** | **add `PolicyComplex` + `PolicyMultiA` arms** | **trip-wire PASS, 1307/1307 green** | **I-7** |

Housekeeping at `19630dd`: `go build ./...` clean, `go test ./md/` ok, `gofmt -l`
clean on all six fold files, `go vet` only the known `testing.ArtifactDir`
go1.25 gap.

## Counts

**2 Important + 1 Important (I-8) / 2 Minor / 1 Nit on the fold — NOT GREEN.**
Separately: **1 Critical, pre-existing and out of scope (C-2).**

I-4 is closed and closed well — driving `gatheredDescriptorFlow` and asserting
the opening are both the right calls, and you were right to run the mutation
rather than assume. I-5 is closed everywhere that matters. What is left is that
two of this fold's three answers rest on a claim about Core that Core, now that
it is installed, does not make (I-6), and the trip-wire written to protect the
one deferral checks the three policy kinds that cannot break it and skips the
three that can (I-7).
