# R0 review — `SPEC_descriptor_input.md`, round 5 (proportional re-review of the r4 fold)

**Artifact:** `design/SPEC_descriptor_input.md` at `1d054f5` (1363 lines).
**Scope, as briefed:** (1) did the fold close each of r4's seven findings and correctly record
the two rulings it carried; (2) did the fold introduce new defects — concentrated on the
NEW-I1 resolution (option i: `multi` admitted on the `--as md1` path only). Not a fresh audit.
r1's verified-TRUE table, all r1–r4 measured probe results, the citation gate, the operator
rulings (F-417, F-418) and the r1→r4 dispositions were taken as settled.
**Reviewer:** independent agent, opus tier. Read-only; nothing in any of the three repos was
written to. Go probes: scratch modules `…/scratchpad/goprobe5{,b,c}` with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`, built with
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`. md1 probes:
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (the tree-built binary, per
§2's stale-binary note). Keys K1/K2 are the fork's own `nonstandard/parse_test.go` cosigners
(`dc567276` / `f245ae38`).

## Counts — NEW findings only

**0 Critical / 3 Important / 3 Minor / 1 Nit**

**Disposition of r4: 7 FIXED, 0 PARTIAL, 0 NOT FIXED. Both ruling folds RECORDED.**

**The lens does NOT close.** Every one of r4's seven findings is closed by re-measurement, and
the fold's central ruling — that md1 really carries `multi` all the way to the device — is not
merely consistent but **measured end to end for the first time**, in both implementations
(below). But the three Importants are all authored by this fold, and two of them are in the
machinery the fold added to *gate* the new admission:

- **NEW-I1** — §7's rewritten refusal assertion fires on ~14 required rows it must not, so the
  vector file cannot be authored green. This is a widening the fold made for no reason: the
  condition r4 asked to be *narrowed* by a citation clause was simultaneously broadened from
  "on a host-admitted row" to "parses under `me`'s cascade".
- **NEW-I2** — `multi` × `/0/*` is now a two-step refusal LOOP: each `--as` path names the
  other. That is verbatim r2's NEW-I5 and r4's NEW-I1 shape, one input class over, recreated by
  the fix for r4's NEW-I1.
- **NEW-I3** — the one vector row that gates the new admission **cannot fail**. Measured on both
  implementations: at index 0 the `multi` and `sortedmulti` addresses are byte-identical for the
  spec's own key pair, so a `multi` → `sortedmulti` rewrite — the exact rewrite §6 and §10 say
  `me` must never do — passes the row.

None of the three needs a design change. NEW-I1 is a reverted clause, NEW-I2 is one row's text,
NEW-I3 is one word (an index).

---

## Disposition table — r4's seven findings and the two ruling folds

| # | r4 finding | verdict | what re-running it shows |
| --- | --- | :-: | --- |
| NEW-I1 | `wsh(multi)`: three sections promise `--as md1` carries it, §3+§4 forbid `me` parsing it | **FIXED** | Option (i) taken and carried to every surface I could find: §4.7 conjunct 1 admits the three twins on the md1 path only; §3 gains the boundary paragraph; §4.3 states `me`'s parser reads `multi`; §5.5's cell, §6's row, §10's bullet and §7's `neither` bullet all agree. **The central claim is now measured, not argued** — see the section below: the device decodes an md1 `multi` set as `PolicyMulti`, routes it through `complexAddressSource`, and derives addresses byte-identical to Rust `md` at indices 0 **and** 1. §5.5's *"needs a firmware change: no"* is TRUE for `multi`. (Three new defects live in the fix — NEW-I1/I2/I3 below.) |
| NEW-M1 | discriminator's false universal + mislabelled citation | **FIXED** | *"always"* is gone; the citation now names a file that was actually run. **Machine-verified, not read:** the fork's `sh` fixture is **14 lines**, whole-input parse ACCEPT, title `sh`, 3 keys, canonical `…#tk50fvpm`, and `Encode(Parse(canonical)) == canonical` — a fixed point. Every clause of the new citation is true. (The parenthetical's *"no `Format:` … does not [parse]"* half is true of `me` and false of the measurement it sits inside — **new NEW-M1**, Minor. Also: **r4's own claim that `#0dc3ykny` "exists in no repo" is FALSE** — see the corrections section; the fold's text is true either way.) |
| NEW-M2 | `md1_admits` default backwards for most rows | **FIXED** | Now *"REQUIRED on every row — no default"*, with r4's reason quoted, and §11 item 3's counting test asserts presence. The default is gone, so none of r4's four named rows can inherit a wrong value. (The second half of NEW-M2 — the citing clause — was folded and simultaneously broke the assertion's trigger: **new NEW-I1**, Important.) |
| NEW-M3 | zero-fp warning false on the md1 path | **FIXED** | §4.2 now leads *"**The loss is the CANONICAL RE-ENCODING's alone, so the warning is scoped to `--as descriptor`**"*, carries r4's `0xb3602`/`#t2st4md6` measurement, cites §5.3(b)'s label-only rule, and closes *"under `--as md1` nothing is lost and no warning fires."* The false alarm is gone and the mechanism is named rather than asserted. |
| NEW-M4 | offending-key naming missing from §6's two md1-split rows | **FIXED** | Both rows now open *"key `@N` (`[<fp/path>]xpub…`)"* — a substitution slot of the same shape as the sibling rows r4 cited. The unstated (a)+(a″) precedence question is answered in the same cell (*"both fire, both are true, and both name the same remedy — no precedence is needed"*), which is the ruling r4 said would settle it. |
| NEW-N1 | bare `Zpub`/`Ypub` given one multisig form | **FIXED** | Now *"(`wsh(sortedmulti(…))` for `Zpub`, `sh(wsh(sortedmulti(…)))` for `Ypub`)"*, attributed to `Script.DerivationPath()`. Consistent with the mapping r4 verified at `bip380.go:449–451` (`ZpubVer → P2WSH → …/2'`, `YpubVer → P2SH_P2WSH → …/1'`) and with the row's own path list, which is in the same `Zpub`-then-`Ypub` order. |
| NEW-N2 | §4.6's stale single-document justification | **FIXED** | Now *"reads its input WHOLE in both contexts: §5.1's single-document mode when `--as` is present, and §5.1's whole-input-parse discriminator when it is absent"*. Both readers of "the whole input" are covered, which is exactly what r4 said had gone stale. |
| **Ruling** | §8's phase order, F-418 | **RECORDED** | `design/FOLLOWUPS.md:14511` carries `### F-418 — RULING RECORD: descriptor-input phase order is S1 → S3 → S2` with the owning phase (*descriptor-input planning*), the operator's verbatim quote (*"I'm away from sh2 and it's not connected. That should inform s2/3 ordering, I think."*), and the same asymmetry §8 gives. §8's text matches clause for clause: the order, the 2026-08-28 date, the F-418 label, *"reversing the original S2-first order"*, S2 parked until the SH2 is back. (§8 also asserts a scoping of §11 item 6 that §11 does not itself carry — **new NEW-M3**, Minor.) |
| **Ruling** | §11 item 2's non-`/0/*` JSON exemplar | **RECORDED** | Item 2 now closes *"The JSON exemplar must use a non-`/0/*` descriptor: the fork's own JSON fixture is `/0/*`, which `--as md1` refuses per §5.3(a)"*. True (the fixture at `parse_test.go:22` is `/0/*`), correctly attributed to r4-verified-in-passing, and it does **not** collide with §7's row bullet, which mandates the fork's fixture for the *vector row* — a different artefact from item 2's acceptance run. |

---

## The fold's central ruling, measured end to end — and nobody had run it

r4 measured only that `md encode` *accepts* a `multi` template (`0xd5e52`). That leaves the
load-bearing half unmeasured: **can the device read one back and derive the right wallet?** If
the fork's md1 consumer flattened `multi` onto its descriptor model, `--as md1` would engrave a
plate that restores as `sortedmulti` — a different wallet — which is exactly the harm §5.3
exists to prevent. `bip380.MultisigType` has **two** values, `Singlesig` and `SortedMulti`
(`bip380/bip380.go:90–94`); there is no unsorted arm, so the risk is real.

It does not happen. Measured:

```
gui/md1_expand.go:102 scriptForTemplate  -> !ok for unsorted multi ("not bip380-expressible (D2)")
gui/md1_gather.go:162 gatheredDescriptorFlow -> expandUnsupported -> complexAddressSource(...)
gui/policy_address.go: md.EmitWitnessScriptChunks + address.WitnessScriptAddress
md/script_emit.go:474  if n.tag == tagSortedMulti { sortByteSlices(ks) }   <- NOT applied to tagMulti
```

Driving that exact chain from a scratch module on the `0xd5e52` card set:

```
md1 set 0xd5e52  (wsh(multi(2,K1/<0;1>/*,K2/<0;1>/*)))
  device: template Root=Wsh Policy=PolicyMulti K=2 N=2 Renderable=true
  device recv0  bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
  md     recv0  bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a   MATCH
  device recv1  bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y
  md     recv1  bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y   MATCH

md1 set 0x16d62  (wsh(sortedmulti(2,K1/<0;1>/*,K2/<0;1>/*)))
  device: template Policy=PolicySortedMulti
  device recv1  bc1q9edtz99nhdf95kaltjk8xtzrxt0ysekrytv4vp69psdf5y7mnamsar8nzf  != multi recv1
```

So the ruling holds on the facts: md1 carries `multi`, the device decodes it **as** unsorted
multi, derives it through the complex-policy route, and Go and Rust agree at the address layer.
§3's *"MDMK records the device's descriptor parser never reads"* is also true —
`nonstandard.OutputDescriptor` still has exactly the two callers §2.4 measured, neither on the
MDMK path — and conjunct 1 is indeed the **only** place §4.7 widens on the md1 path (conjunct 7
says explicitly that it gates both `--as` values; 2–6 are narrowings). §3's *"exactly once"* is
true.

One fact worth having in the spec, not a finding: a `multi` md1 record lands on the
**complex-policy screen** (`md1PolicyFlow`), not the descriptor screen (`descriptorFlow`),
because `scriptForTemplate` refuses it before the descriptor is built. Addresses are real and
correct there; the operator's screen simply differs from the `sortedmulti` case. §5.5's
*"readable: no [firmware change]"* is unaffected.

---

## NEW — Important

### NEW-I1 — §7's rewritten refusal assertion fires on ~14 required rows whose refusal is not §5.3's, so the vector file still cannot be authored green

**Authored by this fold.** r4's NEW-M2 second point asked for a *narrowing* clause — name the
reason, so an unrelated refusal cannot satisfy the assertion. The fold added that clause and, in
the same sentence, **widened the trigger** from r3's *"where `md1_admits` is false **on a
host-admitted row**"* to:

> Where `md1_admits` is false **and the input parses under `me`'s cascade**, the Rust test
> asserts that the md1 path REFUSES **citing §5.3(a)/(a″)** (line 1194).

"Cascade" is not "admission" anywhere in this spec, and the spec says so itself: §4.7 opens
*"`me` admits a descriptor only if, **after the cascade**, it is one of…"*, and §5.2's predicate
conjoins them as two different tests — *"it parses under §4's cascade **and** matches §4.7's
grammar"*. So the trigger now sweeps in every row that **parses** and is then refused by
conjuncts 2–7 — which is most of §7's own required list.

**Constructed failure.** Take a row §7 requires by name, from the *"every shape §4.7 narrows"*
bullet: `wsh(sortedmulti(0, K1/<0;1>/*, K2/<0;1>/*))` (the `k=0` row).

- `device_admits = true`, and it **parses under the cascade** — measured today:
  `nonstandard.OutputDescriptor` → `ACCEPT type=SortedMulti thr=0 keys=2`.
- `host_admits = false` (conjunct 2).
- `md1_admits` is REQUIRED and must be `false` — conjunct 2 gates both `--as` values.
- Trigger fires ⇒ the Rust test **must assert a refusal citing §5.3(a)/(a″)**.
- But the refusal `me` is required to print is §6's threshold-0 row — *"threshold 0 means NO
  signature is required…"* — citing §4.7 conjunct 2. §5.3 has nothing to do with it.

The row therefore cannot be authored green: the test demands a message `me` must not print, and
if `me` printed it the message would be false. Same construction, all measured ACCEPT-and-parse
today, all `md1_admits=false`, all refused for a non-§5.3 reason:

```
k=0              device: ACCEPT  thr=0    -> refusal cites conjunct 2
k=5 (>n)         device: ACCEPT  thr=5    -> refusal cites conjunct 2
tr(sortedmulti)  device: ACCEPT           -> refusal cites conjunct 1
wpkh(sortedmulti)device: ACCEPT           -> refusal cites conjunct 1
<0;1>/*h         device: ACCEPT           -> refusal cites conjunct 7
<0;2>/*          device: ACCEPT           -> refusal cites conjunct 7
```

plus `pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `wsh(KEY)`, `sh(KEY)`, `k=−1`, `n=16` under
`sh`, `n=21` under `wsh`, and the mixed-network row — **14 required rows**, versus the 5 the
clause is actually for (`/0/*`, `<0;1>`, the JSON fixture, and the two `--as descriptor`-only
mixed rows).

This is r3's NEW-I1 shape exactly — *"the file cannot be authored green"* — reopened by the fold
that closed it. It fails loudly rather than falsely, but the spec as written cannot produce the
file it requires, and the implementer's only options are a red suite or a silent deviation from
§7.

**Required (author's choice).** Restore r3's scoping — *"where `md1_admits` is false **on a
host-admitted row**"* — which is both correct and sufficient, since every row the citing clause
is for is host-admitted; **or** state the trigger as *"where `md1_admits` is false and the input
is otherwise ADMITTED (cascade + §4.7)"*; **or** keep the wide trigger and split the assertion
in two (refuse-citing-§5.3 for admitted rows, refuse-citing-its-own-conjunct for the rest).

**Evidence:** spec lines 1188–1196; §4.7's *"after the cascade"*; §5.2's conjunction; the six
device-parse measurements above (`…/scratchpad/goprobe5b`).

---

### NEW-I2 — `multi` with a `/0/*` (or bare `<0;1>`) use-site is now a two-step refusal LOOP: each `--as` path names the other

**Authored by this fold.** Admitting `multi` on the md1 path put it inside §5.3's
representability rules, and §5.3's refusals all point at `--as descriptor` — the one path that
`multi` can never take.

**Constructed failure.** The operator holds
`wsh(multi(2,[dc567276/48h/0h/0h/2h]K1/0/*,[f245ae38/48h/0h/0h/2h]K2/0/*))`. This is not exotic:
`/0/*` is the shape of the fork's **own shipped JSON fixture** (§5.3 says so), and `multi` is
`md encode --help`'s **own headline example** (§4.3 says so).

1. `me sysw pack --as md1` — `me`'s parser reads `multi` (§4.3's new parenthetical); §4.7
   conjunct 1 admits it on the md1 path; conjunct 7 admits `/0/*`; §5.3(a) then refuses, and §6
   prints: *"…md1 records a use-site path as either a multipath group (`<0;1>`) or a bare
   wildcard (`/*`)… **Use `--as descriptor`, which carries `/0/*` exactly.**"*
2. `me sysw pack --as descriptor` — conjunct 1's seven forms exclude `multi`, and §6 prints:
   *"the device's descriptor parser accepts `sortedmulti` and not `multi`. **This wallet can
   still be engraved: `--as md1` encodes `multi` policies.**"*
3. Back to step 1.

Both messages are individually true and both are false **about this input**: `--as descriptor`
does not carry this `/0/*`, because it does not carry this descriptor at all. This is verbatim
the defect r2's NEW-I5 and r4's NEW-I1 were raised for — *"a spec that promises a flag will
engrave a wallet its own input contract refuses at the door"* — and the whole-tree sweep the
fold ran for `multi` did not reach it because the two rows share no token.

The same loop exists for `wsh(multi(…/<0;1>))` via §5.3(a″) and §6's next row.

A second, smaller consequence at the same site: §4.7 conjunct 7 closes *"(`/i/*` and
`<i;i+1>`-without-wildcard are `--as descriptor`-only)"*. After the fold that is false for a
`multi` input, where those two shapes are **neither**-path.

**Required (author's choice).** Either give the two §5.3 rows an `unless the descriptor is a
`multi` form` clause with its own message (*"`multi` is carried only by `--as md1`, and md1
cannot represent `/0/*` — this wallet cannot be engraved by either path this release"*, F-414/
F-417 shape); **or** state in §4.7 conjunct 1 that the md1-path `multi` admission is further
restricted to md1-representable use-site paths, so the refusal fires at admission with one
honest message; **or** narrow conjunct 1's parenthetical and say plainly that
`multi` + `{/i/*, <i;i+1>}` is out of scope. What must not survive is two refusals that point at
each other.

**Evidence:** §4.7 conjunct 1 (lines 627–638); §4.7 conjunct 7's closing clause; §6's `/0/*` row
(line 1042) and `multi` row (line 1030); measured — `wsh(multi(2,K1/0/*,K2/0/*))` is device
REFUSE, so `--as descriptor` has no other outcome.

---

### NEW-I3 — the one vector row that gates the new `multi` admission CANNOT FAIL: measured, `multi` and `sortedmulti` derive the identical `address_0` for the spec's own keys

**Authored by this fold.** §7's `neither` bullet now reads *"The `multi` row additionally carries
`md1_admits=true` and **its md1-route `address_0`** — the widening direction §4.7 conjunct 1
admits"*, and §7's second assertion makes that address the row's **only** assertion (the Go test
skips it, `device_admits=false`; requirement 4 is vacuous, no `Descriptor` record). So one number
at one index is the entire gate on the fold's new admission.

**Constructed failure — measured on both implementations, with the spec's own K1/K2:**

```
                 md (Rust) recv0                    device (Go) recv0
multi            bc1qadgf37zk0wtu69j7yclswl99e5…    bc1qadgf37zk0wtu69j7yclswl99e5…
sortedmulti      bc1qadgf37zk0wtu69j7yclswl99e5…    bc1qadgf37zk0wtu69j7yclswl99e5…
                 ^^^^^^^^^^^^^^ IDENTICAL — the derived keys are already in sorted order at 0

                 md (Rust) recv1                    device (Go) recv1
multi            bc1q24khjdhxz70zs228ymlljjrtpp…    bc1q24khjdhxz70zs228ymlljjrtpp…
sortedmulti      bc1q9edtz99nhdf95kaltjk8xtzrxt…    bc1q9edtz99nhdf95kaltjk8xtzrxt…
                 ^^^^^^^^^^^^^^ DIVERGENT at index 1
```

An implementation that silently normalised `multi` → `sortedmulti` on the md1 path passes the
`multi` row completely: `host_admits=false` (unchanged, the rewrite is md1-side), `device_admits
=false` (unchanged), `md1_admits=true` (still true), `sha256(input)` (pins the input, not the
output), `address_0` (identical). Nothing else in §7 inspects the emitted template, and §11 item
2's `md decode` read-back is scoped to *"each of the four formats"*, which need not include a
`multi` exemplar.

That rewrite is the precise thing §6 and §10 forbid in identical words — *"`sortedmulti` differs
from `multi` only in key ordering at spend time — it is not a synonym, so **`me` will not rewrite
it for you**"* / *"rewriting `multi` to `sortedmulti` is a **different policy**"*. The gate the
fold built to protect that promise cannot detect its violation, and the failure mode behind it is
a plate that restores as a different wallet at **every index except the one the test checks**.

Under the project severity rule this is the still-blocking class: *"defects in what a tool claims
to have done (a gate that cannot fail…)"*.

**Required.** One clause, any of: require the `multi` row's address assertion at an index where
sorted and unsorted **differ**, and say why (index 1 for these keys, measured above); **or**
require both `address_0` and `address_1` on that row; **or** require the row to pin the emitted
witness script / the `md descriptor` read-back rather than an address. Whichever is chosen, the
spec should state the reason — *a 2-of-2 whose derived keys are already ordered gives sorted and
unsorted the same address* — because the next author will otherwise reach for index 0 again.

**Evidence:** §7 lines 1174–1177 and 1183–1191; the four measured addresses above
(`…/scratchpad/goprobe5`, `md address --index {0,1}` on chunk sets `0xd5e52` / `0x16d62`).

---

## NEW — Minor

**NEW-M1 — §5.1's discriminator says a `Format:`-less BlueWallet file "does not [parse] and
falls through like any other unparseable input", inside a parenthetical labelled *measured* —
and §4.2's own measured table says the device ACCEPTS it.** The new text is:

> If it parses as one descriptor — a WELL-FORMED BlueWallet file or pretty JSON does (measured:
> the fork's own `sh` fixture, 14 lines, read whole is ACCEPT, `#tk50fvpm`, a fixed point; **a
> malformed one — no `Name:`, no `Format:` — does not**, and falls through like any other
> unparseable input) …

For **no `Name:`** it is true (measured REFUSE, the `bw.Title != ""` gate at `parse.go:37`). For
**no `Format:`** it is false as a measurement: §4.2's table two sections earlier records
*"same, `Format:` removed | **ACCEPT**, `Script=Unknown`, 3 keys"*, and §4.2 defect 1 explains
that what fails is `Descriptor.Encode()`, which **panics** — it is not a parse failure and it
does not fall through. The claim is true only of `me`, whose §4.2 NORMATIVE rule refuses the
file at branch 1; the sentence presents a normative narrowing as a measured device fact, sitting
inside the same parenthetical as a genuine device measurement. That is the class r4's NEW-M1 was
about, one clause over. The discriminator's rule and its outcome are both correct either way
(with `--as` absent, `me`'s cascade refuses, so the file does fall through) — only the labelling
and the §4.2 contradiction need fixing. Fix: split the clause — *"a malformed one — no `Name:`
(device REFUSE, measured), or no `Format:` (which `me` refuses at §4.2 even though the device
parses it and then panics on re-encode) — does not."*

**NEW-M2 — §4.7 conjunct 3 and four of §6's rows are stated over `sortedmulti` BY NAME, while
conjunct 1 now requires them to bind `multi` too; §11 item 4 tests §6's text.** Conjunct 1 closes
*"All other conjuncts (2–7) apply to `multi` identically"*, which is normatively sufficient — and
BIP-383's 15-key redeemScript bound does hold identically for `sh(multi(…))`, since 16 compressed
keys need 547 bytes whatever the ordering. But the *text* an operator sees does not transpose.
Conjunct 3 reads *"`n ≤ 15` when the **`sortedmulti`** is the DIRECT argument of `sh(…)`"*, and
§6's rows read *"**`sh(sortedmulti(…))`** carries at most 15 keys"*, *"**`sortedmulti(k, …)`**
with `k > n`"*, *"**`sortedmulti(0, …)`**"*, and the single-key-wrapper row's *"The forms the
device derives are `wsh(sortedmulti(…))`…"*. An operator who supplies
`sh(multi(2, 16 keys))` under `--as md1` gets a refusal describing a form they did not write, and
§11 item 4 asserts that text verbatim. Fix: one parenthetical per row (*"(and the `multi` twin —
the bound is the redeemScript's, not the ordering's)"*), or a single sentence at the head of §6
saying the `sortedmulti` rows read over both forms.

**NEW-M3 — §8 asserts a scoping of §11 item 6 that §11 does not carry, and the fold's commit
message reports that scoping as folded.** §8's ruling paragraph closes *"§11 item 6 binds S2's
ship only, so S1 and S3 can plan, build, demonstrate and ship entirely at the desk"*, and the
commit message lists *"11 item 6 binds S2's ship only"* among the folded changes. The diff does
not touch §11 item 6; it still reads *"a `ClassDescriptor` record has been loaded on a real
device and displayed, at least once, before **this** is called shipped"*, under a §11 preamble
that is one conjunctive list for the whole spec (*"It is **done** when, in addition: …"*) with no
per-phase attribution anywhere. The inference is sound — a `ClassDescriptor` record is S2's
artefact by construction — so nothing in the spec is false. What is missing is that §11, the
section an implementer opens to learn what "done" means, does not know the phases were reordered
or that one of its six items is now parked indefinitely. Under F-418 the first shipping phase is
S3, and §11 gives it no satisfiable acceptance list of its own. Fix: put the scoping in item 6
where the commit message says it went (*"— binds S2's ship only; S1 and S3 close without it"*).

---

## NEW — Nit

**NEW-N1 — §5.2's classification predicate is now `--as`-dependent by reference only.** The
blockquote still reads *"A record is `ClassDescriptor` iff it parses under §4's cascade **and**
matches §4.7's grammar"*, and the carve-out that makes this well-defined — *"for §5.2's
classification predicate, which is `--as`-independent and device-facing, the shape conjunct
remains the seven forms"* — lives only inside §4.7 conjunct 1. The cross-reference resolves and
is not circular in effect, but §5.2 is the sentence *"both sides implement"*, and a Go
implementer reading §5.2 alone now has to notice that §4.7's first conjunct has two readings.
Four words in §5.2 (*"§4.7's grammar (seven forms; conjunct 1's md1-path widening does not apply
here)"*) make it self-contained.

---

## Corrections to the record, so a later round does not re-spend them

- **r4's NEW-M1 was itself wrong on its second and third bullets, and the fold changed a TRUE
  citation to a different TRUE citation on the strength of it.** r4 wrote *"The fork has exactly
  two BlueWallet fixtures"* and *"`#0dc3ykny` comes from a BlueWallet file constructed from the
  JSON fixture's three keys — **a file that exists in no repo**"*. Measured: the fork has
  **three** BlueWallet fixtures in `nonstandard/parse_test.go`, and the third — at
  **`parse_test.go:65`**, title `test`, **12 lines**, keys `dc567276`/`f245ae38`/`c5d87297` — parses
  whole to canonical `wsh(sortedmulti(2,[dc567276/48h/…]xpub…,…))**#0dc3ykny**`, a fixed point.
  So r3's original *"the fork's own 12-line fixture … `#0dc3ykny`"* was accurate, and r4's
  negative inherited the scope of a search that missed one raw string. Nothing to fix in the
  spec: the fold's replacement citation (`sh`, 14 lines, `#tk50fvpm`, fixed point) is independently
  verified TRUE above. Recorded as an instance of *negatives inherit the search scope*.
- **All three fixtures re-measured** at this revision: `sh` 14 lines `#tk50fvpm`, `V2` 11 lines
  `#u4qhgqpj`, `test` 12 lines `#0dc3ykny` — all three ACCEPT whole-input and all three are
  canonical fixed points.
- **The `0xd5e52` and `0x16d62` chunk-set-ids both reproduce** on the tree-built `md`, and
  `md descriptor` on the `0xd5e52` set returns `wsh(multi(2,[dc567276/48'/0'/0'/2']xpub…/<0;1>/*,
  [f245ae38/48'/0'/0'/2']xpub…/<0;1>/*))#656zkmsn` — `multi` preserved through the round trip,
  not normalised.
- **`bip380.MultisigType` has exactly two values** (`Singlesig`, `SortedMulti`,
  `bip380/bip380.go:90–94`) — there is no unsorted arm in the device's descriptor model. This is
  *why* §3's boundary paragraph is load-bearing rather than decorative, and it is worth one line
  in §3, since the reason the boundary is safe is that the md1 consumer never builds a
  `bip380.Descriptor` for `multi` at all (`gui/md1_expand.go:102`, D2).
- **§4.7 conjunct 1 is genuinely the only md1-path widening**, as §3 claims: conjunct 7 states it
  gates both `--as` values, and 2–6 are narrowings on both. Checked, not assumed.

---

## Closing

**The lens does not close: 7/7 FIXED and both rulings recorded, but 0C/3I/3M/1N new, all three
Importants authored by this fold.** The ruling itself is right and is now the best-measured claim
in the document — the device decodes an md1 `multi` set as unsorted multi and derives Rust's own
addresses at two indices, which no previous round had run and which is the fact the whole option
(i) rests on.

What the fold did not survive is its own machinery. Two of the three Importants are in the parts
added to *gate* the new admission — a refusal assertion whose trigger was widened in the same
sentence that narrowed its message, and a vector row whose single assertion is blind to the one
rewrite the spec forbids by name. The third is the older pattern this cycle keeps producing: a
new admission created a new input class (`multi` × `/0/*`), and the two refusals covering that
class point at each other. None of the three is a design defect and none costs more than a clause,
but all three are the kind that reach an implementer as a red suite or a dead-ended operator
rather than as a question.

A note for the next brief, in the spirit of *closure is lens-closure*: this round's three
Importants all came from the **cross-product** question — what happens at the intersection of the
new admission and each existing rule — not from re-reading any section. If a round 6 is run after
the fold, that is the question worth pointing it at, along with the one gate here that has never
been executed: no `descriptor_seam_vectors.json` row has ever been written, so every §7
requirement remains a hypothesis.
