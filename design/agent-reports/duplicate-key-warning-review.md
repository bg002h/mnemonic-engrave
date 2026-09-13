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
