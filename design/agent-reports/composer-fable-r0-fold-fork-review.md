# Whole-diff review — fold B (the SeedHammer fork), composer fable review r0

**VERDICT: 0 Critical / 2 Important / 5 Minor / 4 Nit — NOT GREEN.**

Range `f5b068fa..0f435048` on `fable-r0-fold`, 15 commits, 35 files,
+3134/−111. Reviewed read-only in a detached worktree at `0f435048`, removed
on completion; every mutation reverted, tree left clean before removal.

Both Importants are answers to the brief's ONE QUESTION. Neither is a fresh
defect in the implementer's own reasoning: I-1 is the port falling behind a
Rust fold that landed *after* the vendoring, which the brief pre-authorised as
a finding; I-2 is a contract widened correctly for the case it was widened for,
whose anti-invention guard no longer covers the domain it now spans. Twelve
items were closed, the gate reproduces exactly as reported, and both vendored
fixtures are byte-exact against the commits they name.

---

## Important

### I-1 — `ValidatePathList` orders the key-less cap BEFORE `TooManySlots` and `LegacyWrapperShape`; the primary now orders it AFTER

**Counterexample (the primary's own precedence vectors, run through the fork's
Go at the tip):**

| case | Rust `validate()` at `dd7cdffc` | Go `md.ValidatePathList` at `0f435048` |
| --- | --- | --- |
| `precedence_keyless_under_tr` — tr, `[2of3, K, K]` | `KeylessUnderTr{path:1}` | `a key-less path is not expressible under tr: path 2` ✓ |
| `precedence_no_keyed_path` — wsh, `[K, K]` | `NoKeyedPath` | `every path is key-less; at least one path must hold a key` ✓ |
| `precedence_too_many_slots` — wsh, `[9of9 ×4, K, K]` | `TooManySlots{got:36,max:32}` | **`a policy admits at most one key-less path … paths 5 and 6`** ✗ |
| `precedence_legacy_wrapper_shape` — sh, `[2of3, K, K]` | `LegacyWrapperShape` | **`a policy admits at most one key-less path … paths 2 and 3`** ✗ |

Measured by building each `precedence_*` case's `paths` array verbatim from
`descriptor-mnemonic dd7cdffc:crates/md-codec/tests/vectors/compose_refusal_keyless_cap.json`
and calling `md.ValidatePathList` in a throwaway `package md` test (reverted).

**Where.** `md/compose.go:587` (`if len(keyless) > 1`) sits ahead of `:590`
(`slots > ComposeMaxSlots`) and `:593` (`list.Wrapper.isLegacy()`). The Rust
fold `dd7cdffc` ("the key-less cap yields to the structural refusals", M-1)
moved the identical block to the end of `validate()` for a reason that applies
word for word to the port: *the cap's remedy does not cure either of those.*
The fork vendored at `4de55155`, one commit before that fold, so the port is
behind rather than wrong-headed — but CLAUDE.md's Rust-primary rule makes
validation semantics normative in Rust, and this is a divergence in normative
behaviour, not in style.

**The operator-visible half.** `sh` / `sh-wsh` admits a key-less path at
creation (`gui/composer_shape.go:463` only refuses the *second*), so with a
legacy wrapper selected the device draws

> A wallet can have one key-less path, not two. Two of them make this script
> malleable, and no wallet will import it. A time lock does not help. Give one
> of them a key, or fold them into one path.

— on a list whose actual blocker is `sh`'s one-sorted-multisig rule. Folding
leaves two paths under `sh`, still refused; the remedy cannot succeed, and the
body whose remedy *does* work (`composerCopyRefuseLegacyShape`, "Use wsh or tr")
is one arm away at `gui/composer_shape.go:57`. The 36-slot case is the same
shape: 36 slots folded are still 36.

**Expected.** Move the `len(keyless) > 1` block below the legacy-wrapper check,
matching `md_codec::compose::validate()`. Five lines, and it makes the four
`precedence_*` vectors pass on the next re-vendor instead of failing two of
four.

### I-2 — `md.Branch.K/N` widened to "the one multi at any depth", and its anti-invention guard is now vacuous over the widened domain

**Counterexample, measured at the tip and at the base:**

```
tree: wsh(and_v(v:older(10), or_d(multi(2,@0,@1,@2), c:pk_k(@3))))
  base f5b068fa : branch 0: K=0 N=0 Keys=4 Timelock=true
  tip  0f435048 : branch 0: K=2 N=3 Keys=4 Timelock=true

tree: wsh(and_v(v:multi(2,@0,@1,@2), c:pk_k(@3)))
  base f5b068fa : branch 0: K=0 N=0 Keys=4
  tip  0f435048 : branch 0: K=2 N=3 Keys=4
```

(Both built as `node` literals and passed to `policyShape` in a `package md`
test, run in a worktree at each revision; both reverted.)

`splitBranches` (`md/policy_shape.go:196`) splits `or_*`/`andor` but not an
`or` sitting under `and_v` or a wrapper, so a single branch can hold one
`multi` **plus other key material**. `soleMulti` (`md/policy_shape.go:267`)
counts only `multi*` nodes, so it reports that branch's threshold as the
branch's. For the first tree above that is a 2-of-3 label on a branch `@3` can
spend **alone** after 10 blocks — the exact misreading the guard's own
docstring names ("Reporting `1-of-1` for an `and_v(v:pk(A),older(144))` would
tell an operator the timelock is optional").

**The guard cannot see it.** `TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee`
(`md/policy_shape_test.go:84`) states the property *"K/N must stay zero for a
branch that is not a bare threshold over keys"* — which this diff deliberately
falsifies — and still passes, because its single fixture
(`and_v(v:pk(@0),older(144))`) contains no `multi` node at all and so never
enters the changed arm. The implementation report cites it as evidence
("a branch with none, or with two, still reports 0/0, so
`TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee` still holds"); it holds
only because its fixture is outside the domain that moved. The guarded risk
was "two multis"; the unguarded one is "one multi plus another key".

**The second consumer is the one that would print it.**
`policySummaryLines` (`gui/template_engrave.go:179`) renders `b.N > 0` as
`"  %d: %d-of-%d"` and its `else` arm carries the comment *"NOT a plain
threshold. Say how many keys it involves rather than inventing a k-of-N that
would misdescribe the conditions."* That invariant is now enforced by nothing.

**Why Important and not Critical.** I traced all three `PolicyShapeChunks`
call sites and none can be fed such a branch today: the composer's consent
(`gui/composer_consent.go:158`) and self-check (`gui/composer_selfcheck.go:66`)
see only composer-authored policies, where one path carries exactly one
`KeySet` (so `Keys == N` always), and `templateConsentFlow`
(`gui/multisig_build.go:808`) is reached only from Multisig Build's own
`assembledMd1`, which "only authors sortedmulti"
(`gui/multisig_build.go:441`). No operator can see a wrong k-of-n at this tip.
What is defective is the contract plus its guard — `md.Branch` is package-public
and the wire can carry the tree (I encoded and decoded one).

**Expected.** Either narrow `soleMulti` to refuse when the branch holds key
material outside the multi (`Keys != N`), or keep the widening and extend
`TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee` with the mixed-branch
fixture above so the anti-invention rule is asserted where it can now fail.

---

## Minor

### M-1 — the M-6 testnet fix closes the `key:` door and leaves the mk1-card door open

Measured, one probe (reverted):

```
key: record  [73c5da0a/48'/1'/0'/2']tpubDFH9dgz…  -> REFUSED, Classify = ClassUnknown   (the fix)
mk1 card     Network="testnet", same tpub, same origin
             mk.Encode  -> ok          ("mk1qp90hdpqqsq…", 2 chunks)
             sysw.Classify(chunk) = ClassMDMK   (NOT ClassUnknown)
             composerCardSources(ctx) -> 1 source, kind=composerSourceCard,
                                         label "73c5da0a m/48h/1h/0h/2h",
                                         xpub tpubDFH9dgzv…
```

`mk/encode.go:166` admits the testnet public version `043587cf` by design
(`mk.Card.Network` is `"mainnet" | "testnet"`), and the composer's card door
(`gui/composer_sources.go:79` → `gui/key_card_seating.go:108`) reaches the
material through `decodeXpubBytes`, which parses and never asks the network.
`gui/composer_flow.go:67` wires `composerCardSources` into `st.sources` for
**every** composer route, so the seating pick-list offers it, the slot binds
the chain-code‖point, and the consent derives mainnet `bc1q…` for material
declared under coin type `1'` — the same sentence `sysw/composer_records.go:418`
now prevents on the other door. §4f's "mainnet-only by construction" is met at
one of two entrances.

Note the door count will not flag it either: the card is `ClassMDMK`, so it
never reaches `composerCopyNotUnderstood`.

### M-2 — the creation-time key-less guard counts an EMPTY path as key-less, and refuses with the wrong body

`composerKeylessPathCount` (`gui/composer_shape.go:408`) tests `p.Keys == nil`,
which is true of the *emptied* path item 11 exists for. Measured:

```
list: [2-of-3], [Keys nil, Hash nil, older(5)]      row reads: "Path 2: empty + 5 blocks"
composerKeylessPathCount(list, except=2) = 1     -> creation guard refuses a new key-less path
body shown: "A wallet can have one key-less path, not two. … Give one of them a key,
             or fold them into one path."
ValidatePathList(list) at Done = "a path with neither keys nor a hash …: path 2"
§8m body at Done: "A path with only a time lock means anyone can spend after it. …"
```

So at creation the operator is told they already hold a key-less path when they
hold an empty one, and handed a remedy ("fold them into one path") for a
condition the list does not have — the same "refusal that says the wrong true
thing" class M-5 closed one screen later. The primary's own vector fixes the
spelling in the very fold this diff is behind (`refused_when`: *"`keys == null`
**AND** `hash != null`"*). The codec is fine because `LockOnlyPathError` returns
first; only the GUI guard is loose. Expected: count `p.Keys == nil && p.Hash != nil`.

### M-3 — the vendored keyless-cap vector test cannot express the four new precedence kinds

`md/compose_keyless_cap_test.go:233`:

```go
if c.Error == nil || c.Error.Kind != v.Error {
        t.Fatalf("%s: the vector's case names error %+v, the file's rule names %q", …)
}
```

`v.Error` is the file's top-level `"error": "TooManyKeylessPaths"`, so **any**
case whose `error.kind` is something else aborts the whole test rather than
being dispatched; and the case struct (`:131-135`) carries only
`Kind`/`First`/`Second`, with no `path`, `got` or `max`, so even after a `kind`
switch it could not compare `KeylessUnderTr{path}` or `TooManySlots{got,max}`.
The refused arm then does `errors.As(err, &TwoKeylessPathsError{})` and nothing
else.

Consequence: the fork has **no gate at all** for the precedence half of the
contract — which is precisely what I-1 violates — and the next re-vendor (the
updated file carries `precedence_keyless_under_tr`, `precedence_no_keyed_path`,
`precedence_too_many_slots`, `precedence_legacy_wrapper_shape`) will `Fatal` on
the first precedence case instead of reporting that two of four disagree. It
fails loud, not silent, which is why this is Minor and not Important; the fix is
the Rust gate's own shape (dispatch on `kind`, compare `message` only when the
case carries one).

### M-4 — two new instances of the doc-comment theft class the repo built a gate for

Measured with a `go/ast` scan over the changed files (throwaway binary, removed):

```
gui/composer_flow.go:565  composerSeedDerivedSlots    DOC OPENS "composerSecretCards is §7f's \"a seed that filled several slots is cut…"
gui/composer_copy.go:219  composerCopyMixedLockBases  DOC OPENS "composerCopySameSeedThreshold is §8g's FIRST body: the shared seed's…"
```

Both helpers were inserted directly beneath an existing doc block with no blank
line, so `go/ast` re-binds the comment to the new function and the documented
symbol is left with none. `composerSecretCards` (`gui/composer_flow.go:577`) now
has no doc comment; the block that used to be its own — and which now heads
`composerSeedDerivedSlots` — still reads **"THE DEDUP IS BY REGISTERED SEED, not
by slot"**, a rule *this same commit* replaced with dedup by master fingerprint.

`gui/composer_consent.go:244` is the third instance in comment form: the §8i
restatement's explanation ("…a `break` on the first would name sha256 on a
wallet whose second path is ripemd160") now runs straight into the
`MIXED LOCK BASES` block with no separator and heads a predicate it does not
describe.

The repo has a dedicated gate for exactly this (`r0 fidelity I-3`,
`TestComposerHelpersDidNotStealADocComment`, `gui/composer_doc_comment_test.go:35`),
and it is a **named list of four symbols** that this fold did not extend — so it
passes. Expected: move each helper above the doc block it sits under, and add
the two symbols to `composerDocOwners`.

### M-5 — §7f's "a seed is cut ONCE" is still unmet for one seed registered bare and again with a passphrase

Measured (probe reverted):

```
registration 0: pass=""        MasterFP=73c5da0a
registration 1: pass="hunter2" MasterFP=ca2c62d2
composerSecretCards planned 2 plate(s)
  plate 0: ["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"]
  plate 1: ["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"]
BYTE-IDENTICAL ms1: true
```

`composerSecretCards` (`gui/composer_flow.go:612`) dedups on `seed.MasterFP`,
and the fold's comment defends the residue explicitly — *"one seed registered
bare and again with a passphrase is TWO fingerprints and two plates — which is
correct: those are two different secrets and two different wallets."* But the
plate is `codex32.EncodeMS1(seed.Mnemonic.Entropy())`: it carries **no
passphrase**, so the two plates are the same fifty characters. The set therefore
holds two indistinguishable bearer plates of one secret, the census counts two
shares, and nothing on either plate records which pairing needed the passphrase
— which is the defect M-3 was filed for (*"one seed at two slots planned 2 ms1
plates (byte-identical: true)"*), reached through a neighbouring door.

Not a regression (the base deduped by `seedID` and planned two here as well),
and `TestFableSpecOneSeedTypedTwiceIsCutOnce` drives only the bare/bare case —
so its name claims more than the code delivers. Dedup on the entropy (what the
plate actually carries), or say on the census that the second plate is the same
words.

---

## Nit

- **N-1** `seedPassphraseStep`'s `bindPassphrase` failure leg
  (`gui/multisig_build_slots.go:908`) returns `false` without `discardLast`, so
  the caller treats the source as declined while the registry keeps the
  registration. Harmless today — the next seed's label is keyed on
  `st.reg.count()` and would skip a number — but it is the one exit from this
  helper that does not un-register.
- **N-2 (undeclared deviation)** The brief asked for a testnet `key:` record to
  be "classified as unsupported **with a one-line reason**". The host prints one
  (`record 0: key: a testnet key (tpub) is unsupported; …`); the device prints
  only the generic `composerCopyNotUnderstood(inert)` count. Consistent with the
  door's stated one-line contract (`gui/composer_door.go:36-40`), so I accept the
  choice — but it is a deviation and the report states it as a fact rather than
  listing it.
- **N-3** `composerCopyRefuseTwoKeylessPaths` names no path numbers, while the
  primary's message names "paths 2 and 3" and `TwoKeylessPathsError{First,Second}`
  carries them to the call site. §7d's same-key body names its slots; this one
  could.
- **N-4** `gui/composer_selfcheck.go:86-98` quotes `md/policy_shape.go`'s **old**
  K/N contract verbatim ("set ONLY when the branch is exactly a threshold over
  KEYS … Zero means 'not a plain k-of-N'") and asserts "§5 lowers a multi behind
  a timelock to `and_v(v:multi(k,…),older(n))`, **which is not one**". This diff
  made both false, and the comment sits on the arm the change re-routed. The
  `.go` file is unchanged, so no gate saw it.

---

## Declared deviations — each judged

1. **Mixed-lock-bases notice on the CONSENT, not the mapping review — ACCEPTED.**
   Verified the premise: `composerConsentFlow` is called unconditionally at
   `gui/composer_flow.go:159` on every route (with `consent = template` when
   unseated), while the mapping review is skippable; and every other *shape*
   fact (§8f, §8a's restatement, §8d, §8i) is already on that surface. Verified
   the predicate is not trivially satisfiable: 2 positives and 4 negatives in
   `TestFableMixedLockBasesUnderWshAreNoticed`, and the tr negative is real —
   `shape.KeyPath == KeyPathNone` genuinely means "not taproot"
   (`md/policy_shape.go:33-38`; a NUMS tr reports `KeyPathNUMS`). I also checked
   every shipped preset: all six use `LockOlderBlocks` + `LockAfterHeight`, both
   HEIGHT (`gui/composer_presets.go:42-48, 118-121`), so the notice fires on no
   preset the device offers by name.
2. **Addresses but no `Descriptor:` line on the complex restore branch —
   ACCEPTED.** The md package emits no text by design; the assertion the fold
   substitutes is equality with the consent's own `policyAddressAt`, which is
   stronger than "an address appears". Verified the label order matches the flat
   branch ("First receive:" / "First change:") so the two branches cannot
   disagree about which index, and that `hasAddr` has no production consumer
   (`multisigRestoreDocFlow` discards it; only tests read it).
3. **The refusal vector pinned separately, not folded into `composeVectorNames`
   — ACCEPTED, and the measurement is real.** `md/conformance_keyed_test.go:22`
   declares `Descriptor string \`json:"descriptor"\`` and no other line in the
   file references it, so the keyed conformance gate genuinely cannot see
   md-codec 0.44.0's xpub rewrite — a full re-vendor would import an unported
   normative rule green. The exclusion is driven by the pin
   (`md/compose_vectors_pin_test.go:71-82`), not a hardcoded prefix, and fails
   closed if the pin goes missing.
4. **Lens 1's `FABLE_OUT`-gated evidence drivers not adopted — ACCEPTED.** They
   assert nothing and skip without an env var; what they measured is asserted by
   the adopted RED tests.
5. **Commit granularity (three findings rebuilt as three commits, plus
   `5f24661`) — ACCEPTED.** `git log --oneline f5b068fa..0f435048` shows 15
   commits, one per finding plus the adoption and the two vendorings, so
   `git diff` per finding means something.

## New FIXED bodies against §8's rules

All five new/changed bodies are in `composerCopyTable()`
(`gui/composer_copy_test.go`) verbatim, so the ASCII gate at `:311` and the
spec-mirror gate cover them; the declared body count moved 82 → 85 with the
reason recorded. `TestComposerSection8mRefusalsAllFitAndDraw` and
`…AllDrawThroughTheRealPath` gained the two new §8m lines; §8a, §8f and the
mixed-lock notice each carry their own `assertModalBodyFits` (margin 80). All
PASS, measured.

"Dismissed only by CONTINUE" is unaffected: the new two-key-less and empty-path
refusals are `showError` refusals, not confirm-to-proceed screens, and the §8a
confirm still goes through `composerConfirmScreen`'s hold.

I additionally checked the surface the modal gate does **not** cover — §8a and
§8f are now also drawn as single `lines` entries on the paged consent, where
`composerPageLines` draws an over-tall first row clipped and then advances past
it (`gui/composer_paged.go:145-160`). Measured at `sh2DisplaySize`
(480×320, content box 224 px, band 411 px): §8a wraps to **167 px**, §8f to
149 px, the mixed-lock notice to 95 px. All fit; the consent's headroom
(≈57 px ≈ 130 characters) is comparable to the modal's 146, so the modal gate
is close to binding. No finding, but no gate stands on the consent side.

## What I ran, with numbers

Worktree `--detach 0f435048`, Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`,
`TMPDIR=/scratch/code/shibboleth/.tmp`, `CGO_ENABLED=0`.

| check | result |
| --- | --- |
| `gofmt -l .` | exactly the five-file baseline (`gui/transaction*.go ×3`, `mt/mt.go`, `mt/mt_test.go`) |
| `go vet ./...` | rc=0; 10 diagnostics, all `testing.ArtifactDir requires go1.26`; zero others |
| `go test ./md/ ./sysw/ ./mk/ ./bip39/ ./address/ ./codex32/ ./hashlock/ ./backup/` | all `ok` |
| `gui-shard-test.sh ./gui/ 24` | `RESULT: ok -- all 1369 tests ran across 24 shards`, wall 25 s |
| `go test ./gui/ -run '^TestFable' -v` | **24 PASS, 0 FAIL, 0 SKIP** — and `TestFableTwoKeylessPathsAgreeWithTheHostOracle` did **not** skip: `md 0.16.2` is on PATH and agreed with the Go port arm for arm on all twelve lists |
| copy/gate set (`TestComposerCopy*`, `…Section8mRefusals*`, `…RefusalBodyMaps*`, `…ShapeRefusalsActuallyRefuse`, `TestComposerHelpersDidNotStealADocComment`) | 10/10 PASS |
| `TestComposeKeylessCapAgreesWithTheRustVector`, both provenance-pin tests | PASS |
| `sha256 md/testdata/vectors/compose_refusal_keyless_cap.json` | `efc5c2d4…65ca5a` = `descriptor-mnemonic 4de55155:crates/md-codec/tests/vectors/…` byte-exact |
| `sha256 sysw/testdata/record_class_vectors.json` | `c5f72145…066770` = `mnemonic-engrave 1cbecbfd:crates/me-cli/testdata/…` byte-exact; `1cbecbfd` is on engrave `master` |

Probes written and **all reverted** (tree confirmed clean before the worktree
was removed): a `package md` precedence table, a `package md` `policyShape`
node-literal table (run at both `f5b068fa` and `0f435048`), a `package gui`
consent-line height measurement, a `package gui` mk1-tpub seating probe, a
`package gui` bare-vs-passphrased ms1 probe, and an out-of-tree `go/ast`
doc-owner scanner.

## What I could not verify

- **Firmware size** (1,644,840 → 1,652,268 B flash, +16 B RAM). Not re-run; the
  brief says the controller is re-running the gate and told me not to spend
  budget on the whole suite. Unchecked.
- **The external-wallet measurements behind the new copy** — libnunchuk 2.1.1's
  7/7 and 14/14 NUMS refusals, its `Timelock mixing` behaviour, Bitcoin Core
  v25/v31.1's `witnesses without signature exist`, Liana's hashlock refusal, and
  the two differing taproot addresses for the NUMS-vs-unspendable-xpub forms.
  None of that software is on this box. The bodies are internally consistent and
  the tests pin the *wording*; the *facts* are taken from the lens reports.
- **The 859-list Rust measurement** and the key-less rule itself — settled by
  the brief, not re-derived.
- **On-device behaviour.** Everything here is host-side Go; no emulator walk and
  no hardware.
