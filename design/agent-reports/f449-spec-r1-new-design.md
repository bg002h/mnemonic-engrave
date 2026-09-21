# F-449 SPEC r1 — review of the NEW design content only

**Verdict: 1 Critical / 4 Important / 2 Minor**

Scope: the five r1 additions named in the brief (§0's one choice screen, §4a's
input side, §6's two new refusals, §7's class-2 ruling, §9's stage table).
Nothing else was reviewed. Every claim below was reproduced against
`descriptor-mnemonic` @ `6cbd49d8`, `seedhammer` @ `7b6f2fb`, `mnemonic-engrave`
@ `49964db1`, with the installed `md 0.17.0`.

---

## C1 — §0/§9: the "one choice screen" is NOT the minimum that makes §0 true; the device cannot derive a kind-1 address and no stage schedules that work

§0 says: *"stage 4 carries **one choice screen** … That is the minimum that
makes §0 true."* §0's promise is that an operator composes a kind-1 wallet **on
the device**. Composing it is not reaching it: the device must also be able to
derive its addresses, and it cannot.

`gui/policy_address.go` has exactly **two** internal-key branches and no third
is expressible without new code:

```
gui/policy_address.go:132   if ikIndex, isNUMS, _, err := md.EmitTapLeavesChunks(collected, probe); err == nil {
gui/policy_address.go:153       if isNUMS {
gui/policy_address.go:154           ikey, err = address.NUMSInternalKey()      // the raw BIP-341 H point
gui/policy_address.go:156       } else {
gui/policy_address.go:156           internal, iok := byIndex[ikIndex]
gui/policy_address.go:158           if !iok { return "", errors.New("gui: taproot internal key has no @N entry") }
gui/policy_address.go:      		   ikey, err = address.DeriveChild(internal, index, change)
```

`md.EmitTapLeavesChunks` returns `(b.keyIndex, b.isNums, …)`
(`md/tapleaves.go:188`, `:204`) — a two-state internal key. And per **§5** the
derived xpub **takes no slot**, so `byIndex` can never hold it. So a kind-1
wallet lands in one of two places, neither correct:

- port maps kind 1 → `isNUMS = true` (the minimal port of today's `bool`): the
  device derives the **raw H point** and shows addresses of a *different
  wallet* — the exact pair §8.3 requires to differ. Silent wrong addresses on
  the consent screen and at plate verify.
- port maps kind 1 → `isNUMS = false`: `byIndex[ikIndex]` misses, `src(0,false)`
  errors, `policyAddressSource` returns `nil, false`, and the consent screen
  falls to its "why there are none" line. The operator consents to steel for a
  wallet the device cannot address.

The work required is a third branch that recomputes §2 over the collected leaf
keys and derives it at `0/i` / `1/i`. **It appears in no stage's content
column.** Stage 3 is scoped to `md/`; `gui/` and `address/` are not in it.
Stage 4's content is *"§7's `KeyPathKind`, the class-2 + unlocked-path ruling,
the two switch arms, F-633 copy, and the one choice screen"* — none of those is
address derivation; the "two switch arms" are the two **print** sites §7 names
(`gui/template_engrave.go:159-164`, `gui/composer_consent.go:205-214`).

The spec knows the file. §8.3 cites it precisely — *"the Go `md/` port never
derives an address — `gui/policy_address.go:132-160` and
`address/taproot_script_path.go` do — so a `md/` vector cannot reach it"* — and
then schedules only the **gate** (stage 4, "§8.3 device leg"), never the work
the gate measures.

Mitigating, and stated so a reader can weigh a downgrade: §8.3 exists and would
catch the wrong-address variant if run. The defect is that §0 asserts a minimum
that is measurably not minimal, on the address path, and §9 gates work it never
schedules.

---

## I1 — §4a's "recompute the recipe and accept kind 1 only on a match" is not implementable at `md encode`: `walk_tr` sees SYNTHETIC keys, not the wallet's

§4 names the re-entry path itself: `parse_template` → `substitute_synthetic` →
`Descriptor::from_str` → `walk_tr`. `substitute_synthetic`
(`crates/md-cli/src/parse/template.rs:1047-1084`) replaces every `@i` with
`synthetic_xpub_for(i, ctx)` (`:1011`) — a deterministic domain-separated
placeholder derived from `sha256(b"md-v0.15" ‖ i ‖ depth)`, *not* the wallet's
key. `parse_template_ext` takes `keys: &[ParsedKey]` but uses them only for
origin/fingerprint resolution; they never reach the tree walk. So at the moment
`walk_tr` must decide the kind, the leaf keys are synthetic.

Reproduced. For `preset-kofn-recovery-tr`'s four real leaf keys the §2 recipe
gives the golden internal key:

```
derived: xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7
golden : xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7
```

The same recipe over the four synthetic keys `walk_tr` actually receives for
`@0..@3` under a `tr` script tree (`ScriptCtx::MultiSig`, depth 4 — my `@0`/`@1`
values match the repo's own goldens at `template.rs:1144-1156` byte for byte):

```
recipe over SYNTHETIC leaves: xpub661MyMwAqRbcFr3PcP5ADj4ZM68SMHM7nbVNbwuQiYCAi1n3LWR2GPbD8seTJrzzxsLb5gHmx6qwMiYe5LYcLfXSZdmS494a9bdH4EL2FNA
real Liana golden           : xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7
```

So at `md encode` the match can **never** succeed, and §4a's prescribed outcome
— *"a real refusal naming the mismatch"* — would tell a Liana user that the
genuine Liana unspendable key for their wallet is not the Liana unspendable key
for their wallet. The current behaviour §4a is replacing is reproduced too:

```
$ md encode "tr(xpub661MyMwAqRbcFswVugWF…Rsak7/<0;1>/*,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,…),and_v(v:pk(@3/…),older(26280))})"
md: template parse error: internal: synthetic key xpub661MyMwAqRbcFswVugWF…Rsak7 not found in key map (rendered: …)
```

Fails closed, so this is not a wrong-wallet risk — it is a ruling that names a
mechanism the named code path cannot execute on one of the three surfaces it
binds. The other two are fine as written: `md decompose` parses a concrete
descriptor and has the real keys, and the template grammar's surface is the
`UNSPENDABLE(liana)` marker (§4a's last paragraph), which carries no key binding
to check. The spec does not distinguish the concrete-descriptor surface from the
template surface, and the distinction is what decides whether the ruling is
implementable.

Note the journey §4a promises does work once the marker rule exists:
`md decompose` → `tr(UNSPENDABLE(liana), …)` + 4 key lines →
`md encode <template> --key …` → `md descriptor`. It is only the literal-xpub
spelling at `md encode` that has no defined, implementable answer.

---

## I2 — §0's choice-screen trigger fires on 5 of 6 tr presets, and on 3 of them §0a's own table says the choice cannot help; on one of those §6 refuses the option the screen offers

§0 sets the trigger as *"when a `tr` policy falls back to NUMS"*. Measured over
all six presets under `--wrapper tr` (RUN, `md compose --wrapper tr --preset …
--experimental`):

| preset under `tr` | internal key | kind-1 changes the Liana outcome? |
| --- | --- | --- |
| `plain-multisig,2of3` | NUMS (`sortedmulti_a` sole leaf) | **no** — §0a class 3 `no locked path` |
| `hashlock-gated,…,older=26280` | NUMS | **no** — §0a class 4 `a hash lock` |
| `decaying-multisig,…` | NUMS | **no** — §0a class 5 `an absolute lock` |
| `kofn-recovery,2of3,older=26280` | NUMS | yes |
| `tiered-recovery,2of2,1of2,older=26280` | NUMS | yes |
| `simple-timelocked-inheritance,older=26280` | real key `@0` | n/a — no screen |

So the screen fires 5 times and the answer matters twice. On the other three,
§0a's own measured table says *"no encoding change helps"*, and choosing kind 1
there is a strict downgrade for the operator who does not know what an
unspendable xpub is: different addresses, a different `WalletPolicyId` and a
different 12-word phrase (§3e), Liana still refusing, and — per §7's own
libnunchuk note — acceptance elsewhere dropping to 1/24 for four keys.

Sharper, because it is a clash between two NEW r1 rulings: `plain-multisig`
under `tr` composes as a **sole `sortedmulti_a` leaf** (RUN, above), which is
exactly the shape **§6 row 1 REFUSES at kind 1**. The choice screen fires there
and offers an option the codec then rejects. Nothing in §0 or §6 suppresses it.

The predicate that separates the two cases from the three already ships and is
already called at the consent screen: with class 2 skipped for the new kind,
`composerLianaOutsideModelClass(tpl.Root, shape)`
(`gui/composer_consent.go:381`, called at `:261`) returns `""` for
`kofn-recovery` and `tiered-recovery` and a non-empty class for the other three
(verified by hand against the classifier; the wsh rows of the same presets are
already pinned at `gui/composer_fable_r0_funds_test.go:896-902`). I am not
ruling on which trigger is right — recording that ask-always presents a
consequential choice in three cases where it can only hurt, and that the
narrower trigger is computable from shipped code.

---

## I3 — §8 vector 8, declared "REQUIRED, not a bonus", is scheduled in no §9 stage

§8.8: *"A live `harnesses/liana` install run at v15.0 — **REQUIRED, not a
bonus**. It is the only measurement of the install path for any unspendable
shape."* §8's opening paragraph is built on this: the evidence measures
`LianaDescriptor::from_str` (**parse**) while §0's promise is **import**.

Grepped across §9: stage 1b's content says *"§8 vectors 1-7, **9**"* — 8 is
skipped by enumeration. Stage 2's gate is §8.2, stage 3's is "§8 vectors in Go",
stage 4's is "§8.3 device leg, §7's constructed shape", stage 5's is
"site 200 + emulator". `grep -n "harnesses/liana\|install run\|v15.0"` over the
whole spec returns the §1 evidence row and §8:457 — and nothing in §9.

The one gate that measures the thing §0 promises has no owning stage, which is
the repo's own "a gate that has never executed is a hypothesis" shape. It is
also the cheapest to place: the install run needs only a rendered descriptor, so
it is satisfiable from stage 1b or 2 onward.

---

## I4 — §4a's "never a phantom slot" refuses origin-less internal keys that are not Liana's, including a real spendable one that `md decompose` handles today

§4a rules: *"A non-matching xpub in that position gets a real refusal naming the
mismatch, never the internal error and **never a phantom slot**."*

The discriminator is "matches Liana's recipe". Everything else in that position
is refused — but the property that makes the slot phantom is **no origin**, not
"not Liana's". Reproduced with a real, spendable, origin-less internal key
(nothing to do with this cycle):

```
$ md decompose "tr(xpub6DXuQW1Q2JpZyteDRGW1pD…qe2nL/<0;1>/*,and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9…nNQe1/<0;1>/*),older(26280)))"
tr(@0/<0;1>/*,and_v(v:pk(@1/48'/0'/1'/3'/<0;1>/*),older(26280)))
note: 1 key(s) state NO origin in this descriptor — @0 (xpub6DXuQW1Q2JpZ…). … The template and
descriptor emissions are unaffected; `--emit commands` refuses.
```

That is a deliberate shipped behaviour — accept, annotate, refuse only
`--emit commands` — and §4a as written reverses it for every tr descriptor whose
internal key is origin-less and not Liana's, without saying it is doing so. Two
concrete classes lose a verb that works today:

- **libnunchuk's PR-1746 form** (sorted + deduped), which §7 discusses as a real
  wallet Nunchuk builds. Under §4a `md decompose` refuses it, and the refusal
  "naming the mismatch" says *"this is not Liana's unspendable xpub"* — which
  reads as an instruction to make it be one, when the truth is that md1 has no
  wire encoding for that recipe.
- **a real spendable internal key whose owner did not record an origin**, the
  case reproduced above, where `@0` is the *correct* answer and not phantom at
  all.

§4a's bullet 2 frames the `@0` outcome as a defect without noticing that the
same `@0` is correct behaviour whenever the key really is a key. The ruling
needs to say which of "refuse" and "fall back to today's annotated slot" applies
to a non-Liana origin-less internal key; as written it says refuse.

---

## M1 — §6 row 1's reachability claim is false for `md encode`, and its stated reason describes a port error rather than a property of the shape

Two small things in one new row.

*Reachability.* §6 row 1 argues the refusal *"costs nothing"* because *"the only
shape that produces it (`plain-multisig`) is out of scope anyway (§0a)"*. True
of the device composer (`md/compose.go:961` sorts only when
`m == 1 && n.path.isBareMulti()`), false of `md encode`, which accepts a
`sortedmulti_a` leaf in any tree shape — `walk_tap_tree` routes each leaf
through `walk_script_root`, whose own comment says a tap leaf *"is the one
position BIP-386/387 makes `sortedmulti_a` legitimate in"*. RUN:

```
$ md encode "tr(<NUMS hex>,{sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pk(@2/<0;1>/*),older(26280))})"
md1yzpqqxqu2fppznxj5uqqqxd2qwcx8qwnw750mu
```

The cost is still near-zero (Liana emits `multi_a`, never `sortedmulti_a`), so
the ruling stands; the argument for it is scoped to the wrong producer, and it
is the argument that let I2's clash go unnoticed.

*The reason.* *"a chain code that changes with the address index"* is not a
property of the shape. §2 hashes the leaves' **account-level** 33-byte pubkeys,
which are index-independent; `MultiALeafScript`
(`address/taproot_script_path.go:261-270`) sorts the **serialized derived**
x-only keys when building the script, a separate operation that does not feed
the recipe. A correct implementation's chain code is index-independent even for
`sortedmulti_a`. The refusal is a sound belt against the port error fable M-8
warns about; it just is not what the row says it is.

## M2 — §9 stage 1b's gate is "§8", which includes legs stages 3 and 4 own

Stage 1b: content *"§8 vectors 1-7, 9"*, gate *"§8"*. Vector 3 is *"Address
equality, three-way … between md, the evidence's recorded Liana addresses, and
**the device**"*; vector 4 requires *"the Go `ParseChunkHeader`/`Decode` pair"*.
The Go port is stage 3 and the device is stage 4, so neither leg can run at 1b,
and stage 4's gate separately re-names "§8.3 device leg". A charitable reading
("the parts that apply at this stage") is available, but as written stage 1b
declares a gate it cannot satisfy and vector 3 is assigned to two stages.

---

## What I checked and found sound

- **§7's class-2 ruling is expressible, and both halves are the default of
  adding a fourth enum value.** Every non-test `KeyPath` site in the fork is an
  exact-equality comparison: `gui/composer_consent.go:206`, `:212`, `:251`,
  `:397`, `:407`; `gui/template_engrave.go:155`, `:160`, `:162`. A new sibling
  is therefore excluded from class 2 at `:397` and does not increment `unlocked`
  at `:407` without either site being touched. §7's claim that these are two
  separate decisions is right, and its claim that the two `switch` statements
  print nothing for a fourth value is right — `template_engrave.go:159-164` has
  no default and its own comment says the key-path line "IS NEVER OMITTED".
- **§7's constructed shape gets the right verdict.**
  `tr(<derived xpub>, and_v(v:pk(@0),older(26280)))`: class 1 no (root is
  `ScriptTr`), class 2 skipped, `unlocked = 0` (key path not counted, sole
  branch locked), `anyLock = true`, `hash/after/olderUnits` all false → falls to
  `case unlocked == 0:` → **"no unlocked path"**, class 7, REFUSE. Exactly as §7
  predicts, and the failure mode §7 names is real: grouping the sibling with
  `KeyPathSpendable` at `:407` makes `unlocked = 1` and class 7 never fires.
- **§7's ruling gives §0a's table for all three out-of-scope presets.**
  `plain-multisig` → `!anyLock` → "no locked path" (3); `hashlock-gated` →
  `hash` → "a hash lock" (4); `decaying-multisig` → `after` → "an absolute lock"
  (5). Matches §0a row for row, and matches the shipped wsh pins at
  `gui/composer_fable_r0_funds_test.go:896-902`. `kofn-recovery` and
  `tiered-recovery` at kind 1 both fall through to `""` (no notice), which is
  §0's measured 1-of-6 → 3-of-6.
- **§6 row 2's RUN claim is true.** `md encode` accepts `<2;3>`, `<0;1;2>` and
  `<0;1>/*h` under `tr(<NUMS hex>,…)` today; all three produced md1 strings.
- **§6 row 2 is sufficient against the per-slot override TLV.** md1 carries
  `use_site_path_overrides` as well as the global `UseSitePath`, and a mixed
  wallet is encodable today (`md encode "tr(H,{pk(@0/<0;1>/*),and_v(v:pk(@1/<2;3>/*),older(26280))})"`
  round-trips through `md decode`). I checked whether that reopens fable I-2b:
  it does not. The internal key holds no slot (§5), so it can carry no override
  and follows the global; with the global pinned at `<0;1>` it derives at `0/i`,
  which is the same alternative Liana pairs positionally. The refusal keyed on
  the global declaration is enough.
- **§2's recipe reproduced independently** as an oracle for I1 (not re-reviewed
  — already settled): a from-scratch implementation gives
  `xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7`
  for `preset-kofn-recovery-tr`, byte-identical to the evidence.
- **§4a's three motivating defects all reproduce**: the `internal: synthetic key
  … not found in key map` leak, the phantom `@0` five-slot template for a
  four-key Liana wallet, and the accompanying "1 key(s) state NO origin …
  `--emit commands` refuses" note.
- **§9's stage-4 ordering for the choice screen is schedulable** in the narrow
  sense the brief asks: the screen needs `md/compose.go:961`'s unconditional
  `isNums: ik < 0` to become three-way, and that file is inside stage 3's `md/`
  scope, which precedes stage 4. The blocker is C1, not the ordering.

## Out of scope, noticed anyway

- §9 stage 5's gate is "site 200 + emulator reaches the new screen", which
  measures that the screen renders but not that the emulator can complete a
  kind-1 composition — the same distinction §8's opening paragraph draws between
  parse and import.
