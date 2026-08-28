# R0 review — `SPEC_descriptor_input.md`, round 6 (fold re-review + the §7 desk-run)

**Artifact:** `design/SPEC_descriptor_input.md` at `a572ed7` (1397 lines).
**Scope, as briefed:** (1) a PROPORTIONAL re-review of the r5 fold — did it close r5's seven
findings, did it introduce defects, and does the cross-product lens find anything in the fold's
NEW sentences; (2) **the §7 desk-run** — author, in full, the eight hardest
`descriptor_seam_vectors.json` rows and walk every §7 requirement against them. Not a fresh
audit. r1's verified-TRUE table, all r1–r5 measured results, the citation gate, F-417/F-418 and
every prior disposition were taken as settled and were not re-derived.

**Reviewer:** independent agent, opus tier. **Read-only** — nothing in `mnemonic-engrave`,
`descriptor-mnemonic` or `seedhammer` was written to, and nothing was committed or pushed.
Go probes: scratch module `…/scratchpad/goprobe6` with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`, built with
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`. md1 probes:
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (the tree-built binary, per
§2's stale-binary note). The fork worktree is checked out at `0b656d7` (branch
`ship/tx-engraving`), not `d402f18`; re-verified this round that
`git diff --stat 0b656d7 d402f18 -- nonstandard/ bip380/ address/` is **empty**, so every Go
measurement below holds against both revisions.

---

## Counts — NEW findings only

**0 Critical / 2 Important / 6 Minor / 3 Nit**

**Disposition of r5: 6 FIXED, 1 PARTIAL, 0 NOT FIXED.**

**Authorability verdict: 8 of 8 rows are authorable** — every schema field has a real,
measured value except two, and both exceptions are findings below (`address_1` and the
read-back pin have no schema field on the `multi` row; `device_admits` has no true value on the
panic row). Nothing needed the unbuilt Rust parser to author: `host_admits` and `md1_admits`
are spec-derived predicates and every address, checksum and digest was RUN.

**The correctness lens does NOT close**, for two reasons, one per Important:

- **NEW-I1 — r5's NEW-I3 is only half fixed, and the half that was left out is the half that
  makes it a gate.** The fold added `address_1` and the `md descriptor` read-back to what the
  `multi` row *carries*. It did not extend the one sentence in §7 that assigns assertions to
  tests, which still names `address_0` alone. A `multi` → `sortedmulti` rewrite therefore still
  passes every stated §7 assertion — the identical constructed failure r5 raised, with two
  unasserted fields now sitting next to it.
- **NEW-I2 — §11 item 3's counting test cannot count.** *"the file's row set covers every
  bullet of §7, checked by a test that counts, not by reading"* is a gate with no field to
  count: no row says which §7 bullet it satisfies, and the boolean columns and `format` do not
  partition the bullets. Constructed below: drop the one mixed row that proves per-KEY
  materialisation and every countable property of the file still holds.

Neither is a design defect. NEW-I1 is one sentence in §7; NEW-I2 is one field or one naming
convention.

---

## Disposition table — r5's seven findings

| # | r5 finding | verdict | what re-running it shows |
| --- | --- | :-: | --- |
| NEW-I1 | §7's refusal assertion fired on ~14 rows whose refusal is not §5.3's | **FIXED** | The trigger is now *"where `md1_admits` is false on a row whose input is otherwise ADMITTED (cascade AND §4.7…)"* (line 1219). Walked against all 14 rows r5 named: `k=0`, `k=5>n`, `k=−1` → conjunct 2; `tr(sortedmulti)`, `wpkh(sortedmulti)`, `pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `wsh(KEY)`, `sh(KEY)` → conjunct 1; `n=16` under `sh`, `n=21` under `wsh` → conjunct 3; mixed network → conjunct 5; `<0;1>/*h`, `<0;2>` → conjunct 7. **All 14 are refused by a §4.7 conjunct, so none is ADMITTED and none triggers.** The five rows the clause is for (`/0/*`, `<0;1>`, the JSON fixture, and two mixed rows) all trigger correctly. **And the converse was measured, not assumed:** I checked whether any *admitted* shape has `md1_admits=false` for a non-§5.3 reason, which would fire the trigger with a false citation. **None does.** All seven admitted shapes encode in md1 — `pkh` `0x3d601`, `wpkh` `0xb8e83`, `sh(wpkh)` `0xf4b8e`, `tr` `0x5ede4`, `wsh(sortedmulti)` `0x16d62`, `sh(wsh(sortedmulti))` `0x16631`, `sh(sortedmulti)` `0xa809f` — and so do both key-count extremes conjunct 3 admits: `wsh(sortedmulti(2,20 keys))` `0x934a3` and `sh(sortedmulti(2,15 keys))` `0xfe4fd`, each deriving the device's own address. The trigger's scope is now exactly the §5.3-split rows. |
| NEW-I2 | the `multi` × md1-unrepresentable-path refusal LOOP | **FIXED** | All four sites agree and none points at a flag that also refuses. §5.3(a) gains *"UNLESS the descriptor is a `multi` form … the refusal states that NO `me` path carries this descriptor this release, and names the re-export remedy"*; §5.3(a″) gains *"including (a)'s `multi` exception"*; conjunct 7's closing clause gains *"for a `multi` form, which has no `--as descriptor` path, carried by NEITHER path"*; §6's three rows carry the same thing — the `--as descriptor` `multi` row is now conditional (*"(for md1-representable use-site paths — otherwise the §5.3 rows state that neither path carries it)"*), the `/0/*` row carries the replacement remedy verbatim, and the `<0;1>` row inherits it by reference. The replacement remedy names **no flag** — it names a re-export. Measured that the loop input has no other outcome: `wsh(multi(2,K1/0/*,K2/0/*))` is device REFUSE. |
| NEW-I3 | the `multi` row's gate CANNOT FAIL | **PARTIAL** | The **data** requirement landed: the row now carries `address_1` and pins the `md descriptor` read-back. Both reproduce — md1 `multi` `0xd5e52` recv0 `bc1qadgf37z…` / recv1 `bc1q24khjdhxz70zs…`, `sortedmulti` `0x16d62` recv0 **identical** / recv1 `bc1q9edtz99n…`, read-back `wsh(multi(2,…))#656zkmsn`. The **assertion** requirement did not: §7's only sentence assigning assertions to tests still reads *"Rows may carry `address_0` … The Rust test asserts **it** … whose ONLY address assertion is the md1 one"*. `address_1` is not in the row schema and no test is told to assert it; the read-back pin has neither a field name nor a test. See **NEW-I1**. |
| NEW-M1 | the split malformed clause / wrong actor per half | **FIXED** | Both halves now carry the right actor, and both re-measured TRUE this round. *"no `Name:` is a device REFUSE (measured)"* — measured `nonstandard: unrecognized output descriptor format`. *"no `Format:` is refused by `me` at §4.2's NORMATIVE rule even though the DEVICE parses it and then panics on re-encode"* — measured device ACCEPT (`type=1 thr=2 keys=2 title="nofmt" script=Unknown`) and `Descriptor.Encode()` **PANIC: unknown script**, with the control (both headers present) ACCEPT + fixed point. *"either way it falls through"* is true of the discriminator. |
| NEW-M2 | conjunct 3 and §6's rows stated over `sortedmulti` by name | **FIXED** | Conjunct 3 now reads *"the `sortedmulti` — or its `multi` twin (the bound is the redeemScript's, not the ordering's)"*, and §6 gains the head sentence r5 offered as the cheaper option: *"the `sortedmulti` rows below read over BOTH multi forms … with the form name substituted"*. (The blanket form over-applies to exactly one row — **NEW-M4**, Minor.) |
| NEW-M3 | §8 asserted a scoping §11 item 6 did not carry | **FIXED** | Item 6 now closes *"— binding **S2's ship only** (F-418): S1 and S3 close without it, and it is parked with S2 until the device is back on the bench."* That is r5's prescribed fix, at r5's site. (Its twin, item 1, is equally S2-bound and did not get it — **NEW-M5**, Minor.) |
| NEW-N1 | §5.2's predicate `--as`-dependent by reference only | **FIXED** | Now *"§4.7's grammar — the seven forms; conjunct 1's md1-path widening does not apply here."* Self-contained. |

**The tightened §3 boundary paragraph, checked against the source rather than against r5's
claim** (the fold strengthened what r5 wrote, so the tightened version is what was verified):

| tightened clause | verdict |
| --- | :-: |
| *"`bip380.MultisigType` is `Singlesig`/`SortedMulti` only, `bip380/bip380.go:90–94`"* | **TRUE** — `type MultisigType int` at line 90, `const (` 92, `Singlesig` 93, `SortedMulti` 94. Two values, no unsorted arm. |
| *"`gui/md1_expand.go:102` (`scriptForTemplate`)"* | **TRUE** — `func scriptForTemplate(tpl md.Template) (bip380.Script, bip380.MultisigType, bool)` is line 102. |
| *"maps only the bip380-expressible template shapes to bip380 scalars and reports !ok for the rest (its own D2 comment)"* | **TRUE** — the switch has `md.PolicySingle` and `md.PolicySortedMulti` arms only, no `PolicyMulti` arm, and falls through to `return 0, 0, false` under the comment *"Unsorted multi / multi_a / sortedmulti_a / taptree / any other shape: not bip380-expressible (D2)"*. Its own doc comment at line 101 says *"or reports !ok for a non-bip380-expressible shape (D2, R0-C2)"*. |
| *"the `multi` route derives from the template directly"* | **TRUE** — `gui/md1_gather.go`'s `default: // expandUnsupported` branch carries *"STAGE 4: 'display only' is no longer true for every complex policy … now derive real addresses (complexAddressSource)"*. r5's end-to-end address measurement is the confirmation; nothing in §3 overstates it. |

Note for the record: `scriptForTemplate`'s own trailing comment still says *"display-only, never
verified"*, which STAGE 4 contradicted. That is a stale comment in the **fork**, not in this
spec, and §3's text is true as written — but it is the class that has produced three Criticals
here, so it is worth a fork-side follow-up rather than being lost.

**Cross-product lens, applied to the fold's NEW sentences only.** The one case the brief named
was checked and MEASURED, and it is clean: a `multi` descriptor with an absent-path key —
`wsh(multi(2,[dc567276/48h/0h/0h/2h]K1,[f245ae38/48h/0h/0h/2h]K2/<0;1>/*))` — is device REFUSE
(so `host_admits=false`, unchanged), (a) does not fire (the path is absent, not fixed), (a″)
does not fire, and (a′) materialises `<0;1>/*` into `@0` exactly as before, giving the `0xd5e52`
card set and `bc1qadgf37z…`. **The new multi exceptions in (a)/(a″) swap the REMEDY SENTENCE
only — they do not change what is refused** — so nothing that was carried before is now caught,
and nothing that was refused before is now admitted. Conjunct 7's new closing clause names only
`/i/*` and `<i;i+1>`-without-wildcard, so it cannot reach the absent case either.

---

## Job 2 — the §7 desk-run

No row of `descriptor_seam_vectors.json` had ever been written. Below are eight complete rows,
every field with a real value, every measured value RUN this round. Keys K1/K2 are the first two of the fork's
own `nonstandard/parse_test.go` cosigners (`dc567276` / `f245ae38`); row 8's sixteen keys are
unhardened children of K1, constructed as recorded in that row's `source`.

### Row 1 — the `multi` row  (`neither`; the fold's new gate)

```json
{
  "name": "multi-2of2-multipath",
  "input": "wsh(multi(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*))",
  "sha256": "6e1bdfb6a4da7c3fe26b4136f5c91f491aaedbbd92f326a0761a7c8ce212235b",
  "host_admits": false,
  "device_admits": false,
  "md1_admits": true,
  "format": "bip380",
  "address_0": "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a",
  "address_1": "bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y",
  "source": "Constructed from the fork's own parse_test.go cosigners dc567276/f245ae38. md1 chunk-set-id 0xd5e52; `md descriptor` read-back ends #656zkmsn and contains `multi(`."
}
```
### Row 2 — the `/0/*` row  (host true, md1 false)

```json
{
  "name": "sortedmulti-fixed-chain-index",
  "input": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/0/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/0/*))",
  "sha256": "cbfe5bf6632013036847c8a9e6dc92718f362be3788aacb18656a02448c694ed",
  "host_admits": true,
  "device_admits": true,
  "md1_admits": false,
  "format": "bip380",
  "canonical": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/0/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/0/*))#0cvwt807",
  "address_0": "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a",
  "source": "The use-site shape of the fork's own JSON fixture (nonstandard/parse_test.go:22), reduced to two of its three cosigners."
}
```
### Row 3 — `<0;1>` with no trailing wildcard  (host true, md1 false)

```json
{
  "name": "sortedmulti-multipath-no-wildcard",
  "input": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>))",
  "sha256": "3c3cf30fedcedac0e76c6327d1dc76a0d480ce4f993601f78bb5105a2886ee49",
  "host_admits": true,
  "device_admits": true,
  "md1_admits": false,
  "format": "bip380",
  "canonical": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>))#djhjmngu",
  "address_0": "bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9",
  "source": "R0 r2's NEW-C1. Device address_1 is byte-identical to address_0 (no index varies without a wildcard) - measured."
}
```
### Row 4 — mixed: childless + `<0;1>/*`  (md1 true, materialised PER KEY)

```json
{
  "name": "mixed-childless-and-multipath",
  "input": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*))",
  "sha256": "89f8da3e15a2140369e3cbabe73a58ad2966bda2bb3ed578842bd87aa25f1c9b",
  "host_admits": true,
  "device_admits": true,
  "md1_admits": true,
  "format": "bip380",
  "canonical": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*))#vh8ktn5p",
  "address_0": "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a",
  "source": "R0 r3's NEW-C1, third mixture. Device route and md1 route agree: (a') materialises <0;1>/* into @0, giving template wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)), chunk-set-id 0x16d62, md address --index 0 = the same string."
}
```
### Row 5 — whitespace: trailing `\n`  (host WIDER than device; canonical required)

```json
{
  "name": "whitespace-trailing-newline",
  "input": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*))\n",
  "sha256": "778babcdd40d6b372bb6f472feda93ba0e3f5f1f47dde02443b75ca9450562fb",
  "host_admits": true,
  "device_admits": false,
  "md1_admits": true,
  "format": "bip380",
  "canonical": "wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*))#j6g8j0fe",
  "address_0": "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a",
  "source": "Section 4.6. The device REFUSES this input (measured: nonstandard: unrecognized output descriptor format) and ACCEPTS the canonical, which is a fixed point - measured."
}
```
### Row 6 — the C1 no-`Derivation:` BlueWallet file  (device true on INPUT, no canonical)

```json
{
  "name": "bluewallet-no-derivation-header",
  "input": "# BlueWallet Multisig setup file\nName: noderiv\nPolicy: 2 of 2\nFormat: P2WSH\n\ndc567276: xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan\n\nf245ae38: xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge\n",
  "sha256": "5b0dd5b566e25325f8d7a9e7c07a12054ef141460a7964454e15d5320d5b57e5",
  "host_admits": false,
  "device_admits": true,
  "md1_admits": false,
  "format": "bluewallet",
  "address_0": "bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a",
  "source": "R0's C1. The device parses the file and Encode() yields wsh(sortedmulti(2,[dc567276]xpub66C1R...,[f245ae38]xpub66Fud...))#fsc8glm6 - which does NOT re-parse (measured REFUSE), so the row carries no `canonical` by construction."
}
```
### Row 7 — the short-fingerprint panic row  (`device_probe: "panic"`)

```json
{
  "name": "bluewallet-short-fingerprint",
  "input": "# BlueWallet Multisig setup file\nName: shortfp\nPolicy: 2 of 2\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n\nab: xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan\n\nf245ae38: xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge\n",
  "sha256": "662ee10e6639851f5baa57ab59d3a2fe3a4dea90eacd48a1f65f99165eaac05a",
  "host_admits": false,
  "device_admits": <<< NO AUTHORABLE VALUE - see NEW-M1 >>>,
  "device_probe": "panic",
  "md1_admits": false,
  "format": "bluewallet",
  "source": "Section 4.2 defect 4. Measured this round: nonstandard.OutputDescriptor panics `runtime error: index out of range [3] with length 1`."
}
```
### Row 8 — `sh(wsh(sortedmulti(2, 16 keys)))`  (the ACCEPTED side of the BIP-383 bound)

Input is 2311 characters, so it is given by construction rather than pasted; the `sha256` below
is the gate on that.

```json
{
  "name": "sh-wsh-sortedmulti-16-keys",
  "input": "sh(wsh(sortedmulti(2,<16 x [aa0000NN/48h/0h/0h/2h]KEY_NN/<0;1>/*>)))",
  "sha256": "b76f74d7e5688e895e6b0bf191873a6ae87f3d0113d495f6d61eee07af2d9403",
  "host_admits": true,
  "device_admits": true,
  "md1_admits": true,
  "format": "bip380",
  "canonical": "sh(wsh(sortedmulti(2,[aa000000/48h/0h/0h/2h]xpub6EEHPqvmsF3UzjzuP46BAzExdryn5JLrRDKBh1J5vWrvVp8Fexx7AgTXd6nVXQTzFV7FGobGq6rGQLdZ3KftsBCt21cYQaF83EoCWMcp2qA/<0;1>/*,[aa000001/48h/0h/0h/2h]xpub6EEHPqvmsF3UyJAXbSVHi6VvGgDb4VDX6jgG3MnwtFTkJZRffJnbVAp8BLZh33i68rtGUjjtLsfnrU8RiG49PKtGXAPVgjVjK2PzedawG7S/<0;1>/*,[aa000002/48h/0h/0h/2h]xpub6EEHPqvmsF3UzZDkfby5JmXVReRfv5zfJ3nx1G65ixASQruDTFmThhiYdTHnx6PmtUvubnCe7RSXCuSEh1LPnfCq5qCvyJijbWUBd7gWysd/<0;1>/*,[aa000003/48h/0h/0h/2h]xpub6EEHPqvmsF3UypjDmsB5YG6H39VQWxo6rLonHcsM6inG5QnsHvzoaAJjvp2aUWsf3eUwEu4swTLWCMDEdLjFPZpFTu7mENw3cu3nFboHMPJ/<0;1>/*,[aa000004/48h/0h/0h/2h]xpub6EEHPqvmsF3UzLG7M9WNzZKEVuaC4teSsZUjqy2QdKQCaFe1U6YL7yvB7P7RFgkzoW18oQM6BXGXjS6QtUe8by6b3jDxyxrNaNwEwfPRuSJ/<0;1>/*,[aa000005/48h/0h/0h/2h]xpub6EEHPqvmsF3UzaUDBtHCPBju4vghpxfr5m2RBQChjS5oza1WU2kVXXv1R8Qyxkq5yfxMA6tbpYWQLsgtmtJmn6oSdwHKwsj7tfGW2s5pAZx/<0;1>/*,[aa000006/48h/0h/0h/2h]xpub6EEHPqvmsF3UyewF4t42WSQhAG4CttQEMEpud3Zd3DwXJ5x649hNFKBgknRtEijYbprT5x7QCnagsoFEqcBNC8GGqE92MTMvPuJD26M5dQv/<0;1>/*,[aa000007/48h/0h/0h/2h]xpub6EEHPqvmsF3UyLB8WNqcuMuy5FSkcxCfGSKca8s6y94LSeHo7k6v2imgGNpsrfpbq8esBsFr5D1KHK7bTx1Aqufv31WHSxdrUmRsHDyARqz/<0;1>/*,[aa000008/48h/0h/0h/2h]xpub6EEHPqvmsF3UyYaJMWCEu8CWSpTRKnEZju8hAX2rZCefFt4x9Vu5f4PwgZfZ2uCzum11AbunommR6b8iRuFSWMPeyw93Y3RTiuoQwg6qJE6/<0;1>/*,[aa000009/48h/0h/0h/2h]xpub6EEHPqvmsF3UzRAym7DuPd3G24fkuDbRiB35LbwitHYRmMznXoQAuurDBLphhXzcCx4igQCZHQZVuYPvvXxgnTxTGyv1Gh5XUYEFF77DR1C/<0;1>/*,[aa000010/48h/0h/0h/2h]xpub6EEHPqvmsF3Uy3q9w5DpeGqUTfU1Teqz2F2fuf6iAsntKGXRVGGkysK65KaM6jeBtX5tJL8HFd63ob2LfovXtR196e39EJqAhy1oPYwdCWr/<0;1>/*,[aa000011/48h/0h/0h/2h]xpub6EEHPqvmsF3UzwWQCuwNimsebaW97MYJN2FPbDGAAYfcwXaWNYUar94bu9Fx1pkRHCy9tBn22R2wM7xbHbmVx9vA6qtDSQzx2V3ERFWcp9L/<0;1>/*,[aa000012/48h/0h/0h/2h]xpub6EEHPqvmsF3UyCoogkg3JXei9hQpUQeasxjDYY31VAHjYfra4tpEAf4vjbuTUhr6BfKjatk1L9F3tEMLtTfAg9tpfeAE8YULRRA8RXLwnH9/<0;1>/*,[aa000013/48h/0h/0h/2h]xpub6EEHPqvmsF3V1MZgzaH3poSmvgwoEZzM4P8Ri5UbhMH6YkRXt86yjkRnNdxT88754eWH9GFuDmBKPpEbNQWqSF2oF7BQ2oS4vC67WHFyDud/<0;1>/*,[aa000014/48h/0h/0h/2h]xpub6EEHPqvmsF3UzMooRuLpEFzLMVDwB56t6CQy2MPzgdDa2bUQJEDQDjFArSc9TSiDXmjqrkAAib23Nm5tfpMFZppYK7GXMSagG8uHb5E2tCy/<0;1>/*,[aa000015/48h/0h/0h/2h]xpub6EEHPqvmsF3Uyq5Bvy1oB1hqhFLLUczyPxP7W8Tzyuv61xxuagsY22jJQPouNCJdx3vdyyfoz7dVVGrndjMWrbEeixd4FJwwB7gx4LbuJZV/<0;1>/*)))#gns38x3p",
  "address_0": "39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q",
  "source": "R0 r2's NEW-I2 - the 16-key sh(wsh(...)) that r1's prescription would have wrongly refused. KEY_NN = the NN-th unhardened child of the fork fixture key dc567276 (xpub6DiYrfRwNnjeX...Uhpan), fingerprint aa0000NN. Device route and md1 route agree at index 0 AND index 1 (md1 chunk-set-id 0xb1234, 29 chunks)."
}
```

Full canonical is 2311+ characters and is reproduced verbatim above; `reparse = ACCEPT, FIXED POINT`
measured, so requirement 4 holds. Device `address_1` = `3AszGumRQUxNkAFngP3Dvs6YPZfifnGauw`, and
`md address --index 1` on the md1 set returns the same string.

---

## The requirement walk — every §7 requirement against those eight rows

| requirement | verdict against the eight rows |
| --- | --- |
| **1. One file, authored in Rust, vendored byte-identically to the fork** | **SATISFIABLE.** Path is stated on both sides. One mechanical constraint the spec does not mention: the fork copy sits under `nonstandard/testdata/`, and the Go test needs `sysw.Classify` (the `sysw_class` column) — but §5.2's arm makes `sysw` import `nonstandard`, so the test cannot be an internal `package nonstandard` file without a cycle. See **NEW-N3**. |
| **2. sha256 pinned as a literal in BOTH tests** | **SATISFIABLE.** The model instance was re-verified this round: `git show d402f18:sysw/testdata/codex32_seam_vectors.json \| sha256sum` and `sha256sum crates/me-cli/testdata/codex32_seam_vectors.json` both return `3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a`, `SEAM_VECTORS_SHA256` is at `crates/me-cli/tests/codex32_seam.rs:25`, and `sysw/codex32_seam_test.go` exists at `d402f18` (it is absent from the checked-out `0b656d7`, which is why it does not appear in the worktree). §7's "existing instance" citations are TRUE. |
| **3. Rust asserts the host column; Go asserts the device column** | **SATISFIABLE.** Rows 2/3/4/5/8 have `host_admits=true`; 1/6/7 false. No row compares one implementation to the other. |
| **4. `host_admits(input) ⇒ device_admits(canonical(input))`, per row, with the fixed-point clause** | **SATISFIABLE and NON-VACUOUS.** All five `host_admits=true` rows carry a `canonical`, all five re-parse ACCEPT, and all five re-encode to themselves — **`reparse = ACCEPT, FIXED POINT` measured for rows 2, 3, 4, 5 and 8**. Row 5 is the one that needs the canonical-level form: its *input* is device-REFUSE and its canonical is device-ACCEPT, exactly as R0's I1 argued. Rows 1, 6 and 7 are `host_admits=false`, so the invariant is vacuous there **by construction** — and row 6 is the class it exists for: `device_admits=true` on the input while `Descriptor.Encode()` yields `…#fsc8glm6`, which **does not re-parse** (measured REFUSE). |
| **5. non-vacuity: at least one `both`, one `device-only`, one `neither`** | **SATISFIABLE by these eight alone.** `both` = rows 2, 3, 4, 8; `device-only` = row 6; `neither` = row 1. Row 5 is a fourth shape (host-only) the requirement does not name; row 7 is unclassifiable (see NEW-M1). |
| **6. a mistyped vector fails loudly — per-row `sha256` of `input`** | **SATISFIABLE.** Eight distinct digests, all computed this round over the exact bytes: `6e1bdfb6…`, `cbfe5bf6…`, `3c3cf30f…`, `89f8da3e…`, `778babcd…`, `5b0dd5b5…`, `662ee10e…`, `b76f74d7…`. Row 5 is the proof the digest is over raw bytes and not the parsed form: the same descriptor with and without the trailing `\n` gives `778babcd…` vs `eff7f223…`. |
| **schema — `device_admits` means `OutputDescriptor` accepts the INPUT** | **SATISFIABLE except on row 7** — a `device_probe:"panic"` row has no true value for a column defined as accept/refuse. **NEW-M1.** |
| **schema — `sysw_class`, optional, Go-asserted once §5.2's arm lands** | **AUTHORABLE-PENDING-IMPL.** Measured today: `sysw.Classify` returns `ClassUnknown` (`0`) for all eight inputs, so no row can carry `sysw_class:"descriptor"` and pass before S2. Correctly gated by the spec's own *"once §5.2's arm lands"*. Note this makes §11 item 1 unsatisfiable at S3's ship — **NEW-M5**. |
| **schema — `canonical` REQUIRED where `host_admits` is true** | **SATISFIED** on rows 2, 3, 4, 5, 8; correctly absent on 1, 6, 7. |
| **schema — `device_probe:"panic"`, Go test must not feed the input to the parser** | **SATISFIABLE** for row 7 — measured `runtime error: index out of range [3] with length 1`. But the marker's vocabulary covers only the *parser* panic; two other REQUIRED §4.2 rows panic in `Encode()`. **NEW-M6.** |
| **schema — `md1_admits` REQUIRED on every row, no default** | **SATISFIED** on all eight: true on 1, 4, 5, 8; false on 2, 3, 6, 7. Every value is spec-derived and, where md1 encodability was the question, measured. |
| **schema — `format`, one of five values** | **AMBIGUOUS.** No definition of whose parse decides it, and rows 1, 5, 6 and 7 each have a different answer depending on the reading. **NEW-M3.** |
| **schema — `host_admits`** | **UNDEFINED.** `device_admits` gets an explicit definition; `host_admits` gets none. **NEW-M2.** |
| **the `address_0` rule (Go where `device_admits`; Rust via md1 where `md1_admits`)** | **SATISFIABLE, and one field serves both routes on every row that needs both** — measured: row 4 device `bc1qadgf37z…` = md1 route (`0x16d62`) `bc1qadgf37z…`; row 8 device `39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q` = md1 route (`0xb1234`) `39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q`. Rows 1 and 5 are Rust-only (device_admits false); row 6 is Go-only. **This closes two of §9 item 3's named gaps** — the md1 address equality was previously measured for `wsh(sortedmulti(2,…/<0;1>/*))` at index 0 only; it now holds for `sh(wsh(…))`, for 15/16/20 keys, and at index 1. Does not say whether the Go test derives from `input` or `canonical` — **NEW-N2**. |
| **requirement 4's fixed-point clause** | Covered above; measured on all five host-admitted rows. |
| **§11 item 3's counting test** | **NOT SATISFIABLE AS SPECIFIED. NEW-I2.** |

---

## NEW — Important

### NEW-I1 — the fold gave the `multi` row two new values and no assertion to consume them, so the gate r5's NEW-I3 asked for still cannot fail

**Authored by this fold.** r5's NEW-I3 required *"one clause, any of: require the `multi` row's
address assertion at an index where sorted and unsorted differ … or require both `address_0`
and `address_1` … or require the row to pin the emitted witness script / the `md descriptor`
read-back"*. The fold did the second and third — but only in the bullet that says what the row
**carries** (lines 1197–1206). The one place in §7 that says which test asserts what was not
touched (lines 1214–1219):

> Rows may carry **`address_0`**, the receive-address-0 the wallet derives. The Go test asserts
> **it** through `address.Receive` on every row where `device_admits` is true and it is present.
> The Rust test asserts **it** through the md1 round trip on every row where `md1_admits` is true
> — including `host_admits=false` rows like `multi`, whose **ONLY address assertion** is the md1
> one.

`address_1` appears nowhere in that paragraph, is not in the row schema at line 1142–1145, and
the "read-back pin" has no field name anywhere in the document. The paragraph's own words —
*"whose ONLY address assertion is the md1 one"* — were true before the fold and now contradict
the bullet two paragraphs above it.

**Constructed failure — the same mutant as r5's, re-run against the fold.** Implementation
defect: `--as md1` normalises `multi` → `sortedmulti` when building the template (the rewrite
§6 and §10 forbid by name: *"`me` will not rewrite it for you"*, *"a **different policy**"*).
Row 1 as authored above:

- `host_admits=false` — unchanged, the rewrite is md1-side. **PASS**
- `device_admits=false` — unchanged. **PASS**
- `md1_admits=true` — still true. **PASS**
- `sha256` — pins the input, not the output. **PASS**
- `address_0` — the Rust test derives it through the md1 round trip. Under the mutant the round
  trip yields `0x16d62`, whose recv0 is `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a`
  — **byte-identical** to the `multi` route's recv0 (`0xd5e52`), re-measured this round on both
  implementations. **PASS**
- `address_1` — carried, unasserted. Would have caught it (`bc1q24khjdhxz70zs…` vs
  `bc1q9edtz99n…`), but no test is told to look. **NOT CHECKED**
- the read-back pin — carried, unasserted; no field, no test. **NOT CHECKED**
- §11 item 3 — a counting test checks *presence*, and the fields are present. **PASS**

Every stated assertion passes. The plate restores as a different wallet at every receive index
except the one the test checks — verbatim r5's finding, one fold later.

Under the project severity rule this is the still-blocking class: *"defects in what a tool
claims to have done (a gate that cannot fail…)"*.

**Required.** Extend the assertion paragraph, not the row-set bullet. One sentence: *"Where a
row carries `address_1`, both tests assert it at index 1 by the same routes; where a row carries
`md_descriptor_contains`, the Rust test asserts the `md descriptor` read-back of the md1 round
trip contains that substring."* Add `address_1` and the read-back field to the row schema at
line 1142 so the file has somewhere to put them, and delete *"whose ONLY address assertion is
the md1 one"*, which is now false of the `multi` row in two ways.

**Evidence:** spec lines 1142–1145 (schema), 1197–1206 (the row bullet), 1214–1219 (the
assertion paragraph); measured this round — `md encode` `0xd5e52` recv0/recv1
`bc1qadgf37z…` / `bc1q24khjdhxz70zs…`, `0x16d62` recv0/recv1 `bc1qadgf37z…` / `bc1q9edtz99n…`,
`md descriptor` on `0xd5e52` = `wsh(multi(2,…))#656zkmsn`.

---

### NEW-I2 — §11 item 3's counting test has nothing to count: no field maps a row to a §7 bullet, and the columns that exist do not partition them

**Not authored by this fold — surfaced by the desk-run.** §11 item 3 is the gate that makes
§7's row list binding:

> `descriptor_seam_vectors.json` exists in both repos with one sha256, both tests pin it, and
> both suites are green — **and the file's row set covers every bullet of §7**, checked by a
> test that counts, not by reading.

*"Checked by a test that counts, not by reading"* is a deliberate anti-eyeball clause. But the
row schema (line 1142) is stated exhaustively — `name`, `input`, `sha256`, `host_admits`,
`device_admits`, `format`, `source`, plus `sysw_class`, `canonical`, `device_probe`,
`md1_admits` — and **none of them says which §7 bullet a row is discharging.** `name` and
`source` are free text; §7 prescribes no naming convention. The only machine-countable
properties are cardinalities and the boolean columns, and those do not partition the seven
bullets:

- bullets 2 and 7 **overlap**: `xpub…\n` is both a §4.5 promotion near-miss and a §4.6
  whitespace row, so any count of "promoted-key rows" and "whitespace rows" double-counts one
  input;
- bullet 1 (four formats, **happy path**) and bullets 3/5 share every `format` value, so a
  `format`-based count cannot tell a happy-path BlueWallet row from a refused one;
- bullets 3, 4 and 5 are all `format:"bip380"` or `format:"bluewallet"` with
  `host_admits`/`device_admits` values that repeat across them.

**Constructed failure.** Author every row §7 requires **except** the third mixed row of bullet 4
— childless + `K2/<0;1>/*`, `md1_admits=true`, the one row that proves (a′) materialises **per
key** rather than per descriptor. Then:

- every row still carries `md1_admits` (the only presence check §7 actually names, line 1166);
- `both` / `device-only` / `neither` are all still present (requirement 5);
- every `host_admits=true` row still carries a `canonical` (schema);
- every `format` value still appears; the total row count is still large;
- both `md1_admits` values still appear; the `/0/*` and `<0;1>` refusal assertions still fire.

**Nothing counts "a row whose input mixes an ABSENT use-site path with a `<0;1>/*` one",**
because no field encodes it. The bullet is silently uncovered — and the omitted row is exactly
the one that catches an implementation that materialises `<0;1>/*` per-DESCRIPTOR instead of
per-KEY, which is R0 r3's NEW-C1. A gate that cannot detect the omission of the row that gates
a Critical is a gate that has never been able to fail.

The desk-run confirms the row is real and authorable — Row 4 above, canonical
`…#vh8ktn5p`, device and md1 routes both `bc1qadgf37z…`. The defect is that nothing in the spec
would notice its absence.

**Required (author's choice).** Either add a required `covers` field (an array of §7 bullet
tags) and state that item 3's test asserts the union of `covers` equals the bullet list; **or**
fix a `name` convention per bullet and have the test enumerate the required names; **or** drop
*"checked by a test that counts, not by reading"* and say plainly that coverage is a review
step — which is worse, but at least is not a gate claiming a check it cannot make.

**Evidence:** spec line 1142–1145 (schema), 1166, 1172–1212 (the seven bullets), 1388–1390
(item 3); Row 4 above, measured.

---

## NEW — Minor

**NEW-M1 — a `device_probe:"panic"` row has no authorable `device_admits` value.** The base
schema makes `device_admits` non-optional, and defines it as *"`nonstandard.OutputDescriptor`
accepts the INPUT"*. Row 7's input does neither: it panics (measured, `index out of range [3]
with length 1`). `true` is false, `false` is false, and the Go test is forbidden from resolving
it (*"must NOT feed the input to `nonstandard.OutputDescriptor`"*). Worse, requirement 5's
non-vacuity counts `device-only` rows over exactly this pair, so a panic row written
`host_admits:false, device_admits:true` would count as a device-only row that is never probed.
Fix: one clause — *"on a `device_probe:"panic"` row `device_admits` is omitted, and requirement
5 does not count it"*.

**NEW-M2 — `device_admits` is defined and `host_admits` is not, in the section whose own text
says one column carrying two meanings caused the round-0 contradiction.** §7 spends a bullet on
*"`device_admits` means `nonstandard.OutputDescriptor` accepts the INPUT — the scan door,
nothing else"*, and never says what `host_admits` means. Three readings are live and give
different answers on rows authored above: (i) `me`'s cascade parses it — would make row 1
(`multi`) `true`, since §4.3 says *"`me`'s own parser DOES read `multi`"*, breaking the
invariant; (ii) `me sysw pack` succeeds on some `--as` — would also make row 1 `true`; (iii)
§5.2's classification predicate, "`me` packs it as a `Descriptor` record" — the only reading
under which the spec's own pinned values (`multi` false, `/0/*` true, whitespace true) are
consistent. Every REQUIRED row has its value pinned in prose, so the ambiguity has no
wrong-answer path for them; it does for any row an author adds. Fix: one sentence next to the
`device_admits` bullet — *"`host_admits` means §5.2's classification predicate: `me` would pack
this input as a `Descriptor` record."*

**NEW-M3 — `format`'s semantics are undefined on exactly the rows where host and device
disagree.** The enum is `bluewallet`/`bip380`/`json`/`promoted-key`/`none`, with no statement of
whether it records the branch that *matched*, the branch the input *resembles* (§6's five-step
ranking rule), or authorial intent. Concretely, from the eight rows: row 1 (`multi`) matches no
branch on either side — `bip380` or `none`? Row 5 (trailing `\n`) matches branch 2 on the host
and nothing on the device. Row 6 matches branch 1 on the device and nothing on the host. Row 7
matches nothing anywhere (it panics). This interacts with **NEW-I2**: if item 3's counting test
uses `format` to check bullet 1 (*"each of the four formats, **on its happy path**"*), rows 6
and 7 — both host-refused — would satisfy a `format=="bluewallet"` count while no happy-path
BlueWallet row exists. Fix: define `format` as the branch of §4's cascade that `me` matched, and
`none` where none did.

**NEW-M4 — §6's new blanket substitution sentence over-applies to the one row whose
`sortedmulti` mentions are in the REMEDY, not the input.** The fold's head sentence reads *"the
`sortedmulti` rows below read over BOTH multi forms … get the same texts **with the form name
substituted**"*. Applied to the single-key-wrapper row — whose text is *"The forms the device
derives are `wsh(sortedmulti(…))`, `sh(wsh(sortedmulti(…)))` and `sh(sortedmulti(…))`"* — a
`wpkh(multi(2,…))` input under `--as descriptor` yields *"the forms the device derives are
`wsh(multi(…))`…"*, which is **false**: §4.3 measures all three `multi` forms as device REFUSE.
The row's own parenthetical measurement is also device-only and does not transpose — measured
this round, `wpkh(sortedmulti(2,K1/<0;1>/*,K2/<0;1>/*))` is device ACCEPT with
`address: multisig script: Segwit (P2WPKH): unsupported descriptor`, while
`wpkh(multi(2,…))` is device REFUSE and never reaches `address.Receive` at all. The input is
reachable: `me`'s parser reads `multi`, conjunct 1 refuses `wpkh(multi(…))` on both paths, and
§11 item 4 asserts this row's text verbatim. Fix: exempt this row from the head sentence, or
give it its own `multi` text naming `--as md1` as the carrier.

**NEW-M5 — the fold scoped §11 item 6 to S2 and left its twin, item 1, unscoped.** r5's NEW-M3
was fixed exactly as prescribed. But item 1 — *"`me sysw pack --as descriptor --in <each of the
four formats>` … and the device's `sysw.Classify` … classifies that record `Descriptor`"* — is
**entirely** `--as descriptor` plus the device arm §8's own table calls S2's device work, so
under F-418 it is as unsatisfiable at S3's ship as item 6 was. Item 4 (*"every refusal in §6 has
a test that reaches it"*) is partly S2's for the same reason: §6 contains `--as descriptor`-only
rows. §11's preamble is still one conjunctive list with no per-phase attribution, so an
implementer opening §11 at S3's gate reads three items they cannot discharge. Fix: the same
clause item 6 got, on item 1, and a scoping note on item 4. This is the
incomplete-propagation shape: the named instance was fixed and its siblings were not.

**NEW-M6 — `device_probe:"panic"` covers the parser panic and not the `Encode()` panic, on rows
§7 REQUIRES.** The marker is introduced as *"marks a row whose input PANICS the device parser
(§4.2 defect 4)"* with the remedy *"must NOT feed the input to `nonstandard.OutputDescriptor`"*.
But §4.2 records **two** panic sites, and the other one is reachable from two rows §7 requires
by name in the *"every shape §4.2 narrows"* bullet: BlueWallet with no `Format:` (defect 1) and
BlueWallet with zero keys (defect 2) both parse fine and panic in `Descriptor.Encode()` —
re-measured this round: no-`Format:` gives device ACCEPT then `ENCODE PANIC: unknown script`. No
*stated* assertion calls `Encode()` on those rows (requirement 4 is scoped to
`host_admits=true`, and both are false), so the suite as specified will not crash. It is one
diagnostic line away from doing so, and the reason the marker exists — *"a panic would crash
the suite rather than fail it, a false-signal shape"* — applies identically. Fix: broaden the
marker to name the function, e.g. `device_probe: "panic:parse"` / `"panic:encode"`.

---

## NEW — Nit

**NEW-N1 — §4.7 conjunct 3's `3HBBPgNtmPjjuRonQq7EWpurZt3zd4Xvtc` belongs to a key set that
appears nowhere in the document, and §7 points an author at it as provenance.** §7's bullet
requires the 16-key `sh(wsh(sortedmulti(…)))` row *"with its `canonical` and `address_0` (R0
r2's NEW-I2)"*, and conjunct 3 quotes r2's measured address. r2's 16 keys are not recorded, so
the address cannot be reproduced or reused; an author who copies it into the row will pin an
address their own keys do not derive. (Row 8 above measures its own: `39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q`,
from 16 unhardened children of the fork's `dc567276` fixture key — a construction that IS
recorded.) Fix: either record r2's key construction or say the row's address is authored fresh.

**NEW-N2 — the `address_0` rule does not say which string the Go test derives from.**
Requirement 4 has the Go test parse `canonical`; the address rule says only *"through
`address.Receive` on every row where `device_admits` is true"*. Since `device_admits` is defined
over the INPUT, the input is the natural reading, and it is the only workable one for row 6
(which has no `canonical`). Four words would settle it.

**NEW-N3 — the fork's descriptor seam test cannot be an internal `package nonstandard` file
once §5.2's arm lands.** Requirement 1 puts the vendored copy at
`nonstandard/testdata/descriptor_seam_vectors.json`, and §7 says the pattern is *"followed
exactly, not approximately"* — but the model instance (`sysw/codex32_seam_test.go`) is
`package sysw`, an internal test. Measured import facts: `sysw` today imports only
`encoding/hex`, `errors`, `strings`, `seedhammer.com/mt`; `nonstandard` imports `bip32` and
`bip380`; `address` does not import `nonstandard` outside its own test file. §5.2 requires
`sysw.Classify` to call `nonstandard.OutputDescriptor`, at which point an internal
`package nonstandard` test importing `sysw` for the `sysw_class` column is an import cycle. The
fix is trivial (`package nonstandard_test`) but it is a compile error the spec currently steers
an implementer into, and it is the kind of thing the build-gate rule says to settle before
implementation rather than in review.

---

## Measurements taken this round, for the record

Everything below was RUN, not read. Nothing was written to any of the three repos.

```
fork tree            0b656d7 (branch ship/tx-engraving)
  git diff --stat 0b656d7 d402f18 -- nonstandard/ bip380/ address/   -> EMPTY
  so every Go measurement holds against d402f18 too

md1 encodability of every §4.7-admitted shape (…/descriptor-mnemonic/target/release/md)
  pkh(@0/<0;1>/*)                      m/44'/0'/0'      0x3d601
  wpkh(@0/<0;1>/*)                     m/84'/0'/0'      0xb8e83
  sh(wpkh(@0/<0;1>/*))                 m/49'/0'/0'      0xf4b8e
  tr(@0/<0;1>/*)                       m/86'/0'/0'      0x5ede4
  wsh(sortedmulti(2,…))                m/48'/0'/0'/2'   0x16d62   addr0 bc1qadgf37z…
  sh(wsh(sortedmulti(2,…)))            m/48'/0'/0'/1'   0x16631   addr0 3Duywi53NTfAt5waqygMREQiwsbRQ6YmPF
  sh(sortedmulti(2,…))                 m/45'            0xa809f   addr0 3413hYL5Ho9MoGjGjwJatArYzueiYEFAiX
  wsh(sortedmulti(2, 20 keys))         m/48'/0'/0'/2'   0x934a3   addr0 = device addr0
  sh(sortedmulti(2, 15 keys))          m/45'            0xfe4fd   addr0 = device addr0
  sh(wsh(sortedmulti(2, 16 keys)))     m/48'/0'/0'/2'   0xb1234   addr0 AND addr1 = device's
  -> no §4.7-admitted shape has md1_admits=false, so §7's refusal trigger
     cannot fire with a false §5.3 citation.

Go device route vs md1 route, single-sig, indices 0 AND 1 -- all four MATCH
  pkh([dc567276/44h/0h/0h]K1/<0;1>/*)       1M88vKcJFc4KPAe5RHXsuJqWcg3muStyK4
                                            1DyJom6LUg98zbcff7Y3vnh6kYpERcMys3
  wpkh([dc567276/84h/0h/0h]K1/<0;1>/*)      bc1qmj7qns4exnh8p6a9xndvz34msj72arnxl3sapx
                                            bc1q3er64jwge5sfezr6ymkt6d9l79zcvs8z20n5xz
  sh(wpkh([dc567276/49h/0h/0h]K1/<0;1>/*))  354hXbgwGRqHXywh9ZESRXWW4zxrpeScXQ
                                            37cG1ZYNKcQYikRkdmJKKKfXxiVbk6ywiJ
  tr([dc567276/86h/0h/0h]K1/<0;1>/*)        bc1ppeya86zv0hnpzrvh7czgqxkn5zjxxymxd6nqplhhx7fejxvhk0ysp7zekg
                                            bc1pqhh2d3sdktkfvneee95mlv99t0cddcy3vpk5fglz78jm3e55zydqj5wycf
  (incidental corroboration: 354hXbgw... and the wpkh mainnet counterpart of
   tb1qmj7qns4... are the two addresses §6's ypub-remedy row cites.)

multi / sortedmulti, both implementations (r5's result, reproduced)
  0xd5e52 multi        recv0 bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
                       recv1 bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y
  0x16d62 sortedmulti  recv0 bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a  IDENTICAL
                       recv1 bc1q9edtz99nhdf95kaltjk8xtzrxt0ysekrytv4vp69psdf5y7mnamsar8nzf  DIVERGENT
  md descriptor 0xd5e52 -> wsh(multi(2,…))#656zkmsn        (multi survives, un-normalised)

cross-product on the fold's new clauses
  wsh(multi(2,[dc…]K1,[f2…]K2/<0;1>/*))   device REFUSE; (a) and (a″) do not fire;
                                          (a′) materialises -> 0xd5e52, bc1qadgf37z…
  wsh(multi(2,K1/0/*,K2/0/*))             device REFUSE  (the NEW-I2 loop input has no
                                          other outcome, so the re-export remedy is right)
  wpkh(multi(2,…))                        device REFUSE
  wpkh(sortedmulti(2,…))                  device ACCEPT, Supported=false,
                                          "multisig script: Segwit (P2WPKH): unsupported descriptor"

§5.1 discriminator halves (NEW-M1 of r5)
  BlueWallet, no Name:    device REFUSE  "unrecognized output descriptor format"
  BlueWallet, no Format:  device ACCEPT (script=Unknown, 2 keys) then Encode() PANIC "unknown script"
  BlueWallet, control     device ACCEPT, canonical a FIXED POINT, addr0 bc1qadgf37z…

§7's existing instance
  sha256 of both copies of codex32_seam_vectors.json:
    3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a
  SEAM_VECTORS_SHA256 at crates/me-cli/tests/codex32_seam.rs:25
  sysw/codex32_seam_test.go present at d402f18, package sysw

sysw.Classify returned ClassUnknown (0) for all 8 authored inputs and every probe input.
```

---

## Closing

**6/7 FIXED, 1 PARTIAL; 0C / 2I / 6M / 3N new; all eight rows authorable.** The lens does not
close, but it is close, and what is left is smaller than what r5 left.

The r5 fold got the hard parts right. The refusal-assertion trigger is now provably scoped —
not by argument, but because I checked every admitted shape and every key-count extreme through
md1 and found no case where an ADMITTED row has `md1_admits=false`, so the clause cannot fire
with a false citation. The `multi` loop is closed at all four sites and the replacement remedy
names a re-export rather than a flag. §3's tightened boundary is TRUE clause by clause against
the source, which is worth saying explicitly because the fold *strengthened* a reviewer's claim
rather than transcribing it, and the strengthened version is the one that held.

What the desk-run found is that §7's two gates are still not gates. **NEW-I1** is r5's NEW-I3
with the data added and the assertion not — the fold edited the bullet that says what a row
*carries* and left the paragraph that says what a test *asserts*, so the mutant that motivated
the finding still passes. **NEW-I2** is the older, quieter one: §11 item 3 promises a machine
check of row-set coverage and the schema gives it nothing to check with, so the file can omit
the row that gates a Critical and every count still passes. Both are one sentence.

The rest is the desk-run doing what only a desk-run does. Four of the six Minors are fields an
author reaches for and finds undefined — `host_admits`, `format`, `device_admits` on a panic
row, a second panic site with no marker — none of them visible to five rounds of reading,
all of them visible within minutes of trying to write a row. That is the second clause of
closure-is-lens-closure paying out again: **§7 had never been executed, and executing it in
miniature cost less than the round that reviewed it.**

One thing worth carrying forward as a positive result rather than a finding: §9 item 3 said the
md1 address equality rested on *one data point* — `wsh(sortedmulti(2,…/<0;1>/*))`, 2 keys,
index 0. It now rests on **ten distinct descriptor shapes** — `wsh(sortedmulti)` at 2 and 20 keys,
`wsh(multi)`, the childless-mixed `wsh(sortedmulti)`, `sh(sortedmulti)` at 15 keys,
`sh(wsh(sortedmulti))` at 16 keys, and all four single-sig forms `pkh`/`wpkh`/`sh(wpkh)`/`tr`
— with Go and Rust agreeing at index 0 on every one and at index 1 on seven of them. §9 item 3
should be narrowed to what is still true of it: **change addresses and testnet**.
