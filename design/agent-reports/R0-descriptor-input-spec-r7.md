# R0 review — `SPEC_descriptor_input.md`, round 7 (proportional re-review of the r6 fold)

**Artifact:** `design/SPEC_descriptor_input.md` at `edf84ee` (1459 lines).
**Scope, as briefed:** (1) did the fold close each of r6's eleven findings, re-traced by
construction; (2) did the fold introduce defects — **only in what changed**, at the three
pressure points named in the brief (the manifest's arithmetic, the assertion paragraph against
r6's eight desk-run rows, the cross-references). Not a fresh audit. Every r1–r6 measured
result, the citation gate, F-417/F-418 and all prior dispositions were taken as settled and
were not re-derived.

**Reviewer:** independent agent, opus tier. **Read-only** — nothing in `mnemonic-engrave`,
`descriptor-mnemonic` or `seedhammer` was written to, and nothing was committed or pushed.
md1 probes: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`. Go probes:
scratch module `…/scratchpad/goprobe7` with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer` (worktree at `0b656d7`), built
with `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`.

---

## Counts — NEW findings only

**0 Critical / 2 Important / 2 Minor / 2 Nit**

**Disposition of r6: 10 FIXED, 1 PARTIAL, 0 NOT FIXED.**

**The correctness lens does NOT close.** Both Importants are in text this fold authored, both
are one sentence, and both are the same shape the fold was written to close:

- **NEW-I1 — the read-back pin the fold made operative cannot fail.** `md_descriptor_contains:
  "multi("` is a substring of the mutant's own output: the `sortedmulti` read-back is
  `wsh(sortedmulti(2,…))#l9ucx0pn`, and `"sortedmulti("` **contains** `"multi("`. Measured on
  both read-backs this round. The fold's commit message claims *"The multi->sortedmulti mutant
  now fails on address_1 and the read-back"* — the first half is true (measured), the second is
  false.
- **NEW-I2 — §6's new `multi` remedy points the operator at the invocation that just refused.**
  *"keep `multi` and use `--as md1`, which carries it"* is the remedy shown when
  `me sysw pack --as md1` refuses `wpkh(multi(…))` under conjunct 1. It also omits the mandatory
  wrapper change, and the row's *parenthetical* device measurement — the second non-transposing
  part r6 named — was left transposing and is measurably false (all three `multi` twins are
  device REFUSE, measured).

**Positive result worth stating, because it is the fold's main job and it holds:** the coverage
manifest's arithmetic is **correct on all eight tags**, counted against the bullets' own row
enumerations this round, and under honest tagging the manifest catches the omission of **any**
required row — including r6's constructed one. See "The manifest, counted" below.

---

## Disposition table — r6's eleven findings

| # | r6 finding | verdict | what re-tracing it shows |
| --- | --- | :-: | --- |
| NEW-I1 | the `multi` row's two new values had no assertion to consume them | **FIXED** | The assertion paragraph (1259–1276) now consumes all three: *"EVERY such field a row carries is asserted"*, with the Go route (`address.Receive(…, N)` on the parsed INPUT) and the Rust route (md1 round trip) each named, and `md_descriptor_contains` asserted against the read-back. The schema (1155–1159) carries `address_0` / `address_1` / `md_descriptor_contains`. *"whose ONLY address assertion is the md1 one"* is deleted. **Mutant re-run against the stated assertions:** `address_1` **FAILS** — measured `0xd5e52` recv1 `bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y` vs `0x16d62` recv1 `bc1q9edtz99nhdf95kaltjk8xtzrxt0ysekrytv4vp69psdf5y7mnamsar8nzf`. The gate now fails on the mutant, which is what r6 required. The **read-back** half PASSES on the mutant — a new finding on the field's *value*, not on the fold's response (**NEW-I1** below). |
| NEW-I2 | §11 item 3's counting test had nothing to count | **FIXED** | `covers` is REQUIRED and non-empty (1185–1187); the NORMATIVE manifest (1244–1257) gives eight tags with per-tag minima; §11 item 3 (1449–1451) counts them (*"per-tag minima met, no unknown tags, the field present on every row"*). **Drop-the-mixed-row re-run:** author every required row except childless+`K2/<0;1>/*` → `md1-splits` carries **5** of its minimum **6** → the count fails. r6's construction is now a **counted failure**. Every other single-row omission is also caught (worked below). One residue on the unconstrained double-count rule — **NEW-M1**, Minor. |
| NEW-M1 | no authorable `device_admits` on a panic row | **FIXED** | Verbatim r6's prescription, at r6's site (1181–1183): *"On a `"panic:parse"` row `device_admits` is OMITTED — the predicate cannot be evaluated, so either boolean is a false claim — and requirement 5's non-vacuity count skips such rows."* Walked against r6's row 7: the address rule keys on *"wherever `device_admits` is true"*, and absent is not true, so no address is derived; requirement 4 is vacuous (`host_admits=false`); nothing else in §7 reads the column. Authorable. |
| NEW-M2 | `host_admits` undefined | **FIXED** | 1168–1171 defines it as §5.2's classification predicate and **names both wrong readings** (`me` parses `multi`; `--as md1` succeeds on `multi`). Re-derived against all eight desk-run rows: `multi` false, `/0/*` true, `<0;1>` true, mixed true, whitespace true, C1 false, short-fp false, 16-key true — **8/8 agree with r6's authored values**, and the ambiguity has no remaining wrong-answer path. |
| NEW-M3 | `format` undefined on host/device-disagreeing rows | **FIXED** | 1152–1156 defines it as *"the branch of §4's cascade that `me` MATCHED … or `none` where no branch matched"*. Re-resolved on r6's four ambiguous rows: row 1 `multi` → branch 2 matches (§4.3: `me`'s parser reads `multi`) → `bip380`; row 5 trailing `\n` → §4.6 trims first, branch 2 → `bip380`; row 6 C1 → branch 1 matches, §4.2's origin rule refuses after → `bluewallet`; row 7 short-fp → branch 1 matches, the 8-hex rule refuses after → `bluewallet`. **4/4 agree with r6's authored values.** |
| NEW-M4 | §6's substitution rule over-applies to the single-key-wrapper row | **PARTIAL** | The exemption landed (1042–1046) and its new factual claim **measures TRUE** — probed this round, `wpkh(multi(2,…))`, `pkh(multi(2,…))` and `sh(wpkh(multi(2,…)))` are all `nonstandard: unrecognized output descriptor format`, against a `wpkh(sortedmulti(…))` control that ACCEPTs. But r6 named **two** non-transposing parts, the remedy *and* the parenthetical, and the fold replaced only the remedy — with a self-referential one. **NEW-I2** below. |
| NEW-M5 | §11 item 6 scoped, its twins item 1 and item 4 not | **FIXED** | Item 1 gains *"**S2's item** (F-418): it needs the device arm, so S1 and S3 close without it"*; item 4 gains *"The `--as descriptor`-only rows among them are S2's (F-418); the rest bind S3"*. Checked against §8's F-418 text (1322–1329): §8 asserts only that item 6 binds S2's ship and that S1/S3 ship at the desk — both still true, and now better supported. **Mutually consistent**; one propagation nit below. |
| NEW-M6 | `device_probe:"panic"` covered the parser panic only | **FIXED** | Split into `"panic:parse"` / `"panic:encode"` with distinct remedies (1174–1183); §4.2's marker reference at line 354 **says `panic:parse`** — verified; and §7's `narrowed-4.2` bullet marks the two encode-panic rows (1224–1226). Re-measured: no-`Format:` → ACCEPT `script=Unknown keys=2` then `ENCODE PANIC: unknown script`; `Name: only\n` → ACCEPT `keys=0` then `ENCODE PANIC: unknown script`. Also probed the adjacent hole and it does **not** exist: `address.Receive` on an Unknown-script descriptor returns an error (`address: multisig script: Unknown: unsupported descriptor`), not a panic, so a `panic:encode` row carrying an address cannot crash the suite. One spelling caveat — **NEW-M2**, Minor. |
| NEW-N1 | the 16-key address came from unrecorded keys | **FIXED** | Conjunct 3 (654–660) now cites `39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q` *"measured with a RECORDED construction, 16 unhardened children of the `dc567276` fixture key"*, and retires `3HBBPgNtm…` by name as non-reproducible. §7's bullet (1213) points the author at that construction. |
| NEW-N2 | the Go address derivation source unnamed | **FIXED** | *"on the parsed INPUT (the scan door's own string — the C1 row has no `canonical`)"* (1271–1273). Re-measured on the row that forced it: `address.Receive` on the parsed C1 no-`Derivation:` file returns `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a` — the row's `address_0`, derivable with no `canonical` in existence. |
| NEW-N3 | the Go seam test would be an import cycle | **FIXED** | Requirement 1 (1120–1126) specifies **`package nonstandard_test`** and states the mechanism. Verified implementable: `nonstandard`'s existing test file is `package nonstandard` (internal) and Go permits both packages in one directory; `nonstandard` does not import `sysw` today, so the cycle is exactly the future one the clause describes. |

---

## The manifest, counted

The brief asks whether the per-tag minima match the bullets' own enumerations. **Counted by
hand from the bullet text, tag by tag — 8/8 match.**

| tag | min | the bullet's own named rows, enumerated | count |
| --- | :-: | --- | :-: |
| `formats-happy` | 4 | bluewallet, bip380, json, promoted-key | **4** |
| `promotion-near-miss` | 15 | §4.5's table: bare `xpub`, bare `zpub`, bare `Zpub`, bare `Ypub`, `[…/44'/0'/0']xpub`, `[…/49'/0'/0']xpub`, `[…/84'/0'/0']zpub`, `[…/86'/0'/0']xpub`, `[…/48'/0'/0'/2']xpub`, `[…/84'/0'/1']zpub`, `[…/84/0/0]zpub`, `[4bbaa801]xpub`, `xpub…/<0;1>/*`, `xpub…\n`, bare `tpub` | **15** |
| `narrowed-4.7` | 14 | `tr(sortedmulti)`, `wpkh(sortedmulti)`, `pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `wsh(KEY)`, `sh(KEY)`, `k=0`, `k=−1`, `k>n`, `n=16` under `sh`, `n=21` under `wsh`, mixed network, `<0;1>/*h`, `<0;2>/*` | **14** |
| `accepted-extreme` | 1 | `sh(wsh(sortedmulti(2, 16 keys)))` | **1** |
| `narrowed-4.2` | 5 | no `Format:`, zero keys, `Derivation:` after keys, no `Derivation:` at all, short fingerprint | **5** |
| `neither` | 3 | `wsh(multi)`, miniscript, full-origin `ypub` | **3** |
| `whitespace` | 3 | trailing `\n`, CRLF, leading space | **3** |
| `md1-splits` | 6 | `/0/*`, `<0;1>`, childless, and the three mixed rows | **6** |

The split of bullet 3 into `narrowed-4.7` (14) + `accepted-extreme` (1) is correct: that bullet
enumerates fourteen refused shapes and then one accepted one, and giving the accepted row its
own tag is what stops the extreme being swallowed by the refusals' count.

**Single-row omission is now caught for every required row**, not only r6's. Each required row
sits in exactly one bullet, and each tag's minimum equals its bullet's row count, so dropping
any row drops its tag below minimum. The one blessed double-tag (`xpub…\n`) is covered too:
dropping it fails **both** `promotion-near-miss` (15→14) and `whitespace` (3→2).

---

## NEW — Important

### NEW-I1 — `md_descriptor_contains: "multi("` passes on the very mutant it was added to catch: `"sortedmulti("` contains `"multi("`

**Authored by this fold** — the fold is what turned an unasserted carried value into an
operative assertion (*"EVERY such field a row carries is asserted"*), and what gave it a field
name and a quoted literal value at 1227–1232.

> The `multi` row additionally carries `md1_admits=true`, its md1-route `address_0` AND
> `address_1`, and pins the read-back via **`md_descriptor_contains: "multi("`** (measured:
> `#656zkmsn` — `multi` survives the round trip un-normalised).

**Constructed failure — the multi → sortedmulti mutant, run against the read-back assertion.**
Implementation defect: `--as md1` normalises `multi` → `sortedmulti` when building the template
(the rewrite §6 and §10 forbid by name). Both read-backs measured this round from the real
chunk sets:

```
md encode wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))        -> chunk-set-id 0xd5e52
md encode wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))  -> chunk-set-id 0x16d62
  (keys dc567276 / f245ae38, --path m/48'/0'/0'/2' — r6's row 1 ids, reproduced)

md descriptor 0xd5e52  -> wsh(multi(2,[dc567276/48'/0'/0'/2']xpub661My…/<0;1>/*,…))#656zkmsn
md descriptor 0x16d62  -> wsh(sortedmulti(2,[dc567276/48'/0'/0'/2']xpub661My…/<0;1>/*,…))#l9ucx0pn

grep -F 'multi('      against the multi read-back        -> CONTAINS  (assertion PASSES)
grep -F 'multi('      against the SORTEDMULTI read-back  -> CONTAINS  (assertion PASSES)  <-- the defect
grep -F 'wsh(multi('  against the multi read-back        -> CONTAINS  (PASSES)
grep -F 'wsh(multi('  against the SORTEDMULTI read-back  -> absent    (FAILS)
```

`"sortedmulti("` is `sorted` followed by `multi(`, so a substring test for `"multi("` cannot
distinguish the two forms — ever, for any key set, on any row. The assertion the fold added to
pin *"`multi` survives the round trip un-normalised"* is satisfied by a read-back in which
`multi` did **not** survive.

**What this does and does not cost.** The row is not fully inert: `address_1` independently
catches this mutant, measured above, and I confirmed it catches it in **both** cosigner
orderings (with the keys listed in the reverse order the divergence moves from index 1 to index
0, so carrying both addresses covers both orderings). So the row as specified still fails the
mutant. What is lost is that the read-back is the row's **only shape assertion** — both address
fields check derived values — and it is dead. The spec presents it as a working gate, and the
fold's commit message states as fact that the mutant *"now fails on address_1 and the
read-back"*. Under the project severity rule this is the still-blocking class: *"defects in what
a tool claims to have done (a gate that cannot fail…)"*.

**Required.** One string. `md_descriptor_contains: "wsh(multi("` — measured above to pass on
the correct read-back and fail on the mutant. Pinning the checksum (`"#656zkmsn"`) also works
and is stricter, at the cost of re-pinning if the keys ever change.

**Evidence:** spec lines 1227–1232 (the row bullet), 1259–1276 (the assertion paragraph); the
four `grep -F` results above, all run this round against real `md descriptor` output.

---

### NEW-I2 — §6's new `multi` remedy sends the operator back to the invocation that just refused, omits the mandatory wrapper change, and leaves the row's device measurement transposing and false

**Authored by this fold**, at 1042–1046 and 1075.

r6's NEW-M4 named **two** parts of the single-key-wrapper row that do not transpose to `multi`:
the remedy, *and* the row's parenthetical device measurement (*"The row's own parenthetical
measurement is also device-only and does not transpose … `wpkh(multi(2,…))` is device REFUSE and
never reaches `address.Receive` at all"*). The fold's exemption sentence names only the first:

> The one exception is the single-key-wrapper row, whose `sortedmulti` mentions are in the
> **REMEDY** and do NOT transpose — all three `multi` twins are device REFUSE (measured) — so
> for a `multi` input that row's remedy names `--as md1` instead.

**Half a — the remedy is self-referential.** The row's replacement text is:

> For a `multi` input the remedy is instead: *"keep `multi` and use `--as md1`, which carries
> it."*

Walk it. The operator has `wpkh(multi(2,K1,K2))` and types
`me sysw pack --as md1 --in <that>`. §4.7 conjunct 1's md1-path widening admits exactly three
twins — `wsh(multi(k,…))`, `sh(multi(k,…))`, `sh(wsh(multi(k,…)))` — and `wpkh(multi(…))` is not
among them, so conjunct 1 refuses. The refusal then tells the operator to **use `--as md1`**,
the flag they just used, on the input that just refused. This is r5's NEW-I2 shape (a remedy
pointing at a path that also refuses) in a sharper form: the earlier one named a *different*
flag; this one names the invocation that produced the message. §4.7 conjunct 7 states the rule
this violates in its own closing clause: *"§5.3's refusals say so rather than pointing at a flag
that also refuses."*

It is also **incomplete**: the `sortedmulti` text names the three wrapper forms the device
derives, so the operator learns what to change. The `multi` text names none, and the wrapper
change is mandatory — `--as md1` carries `multi` only under `wsh` / `sh` / `sh(wsh)`.

**Half b — the parenthetical still transposes, and is false.** Under the head sentence, a
`multi` input gets *"The device's parser accepts this spelling and then cannot derive any
address from it (measured: `address: multisig script: … unsupported descriptor`)"*. Measured
this round through `nonstandard.OutputDescriptor`:

```
wpkh(multi(2,[dc567276/48h/0h/0h/2h]K1/<0;1>/*,[f245ae38/48h/0h/0h/2h]K2/<0;1>/*))
                                     REFUSE: nonstandard: unrecognized output descriptor format
pkh(multi(2,…))                      REFUSE: nonstandard: unrecognized output descriptor format
sh(wpkh(multi(2,…)))                 REFUSE: nonstandard: unrecognized output descriptor format
CONTROL wpkh(sortedmulti(2,…))       ACCEPT script=Segwit (P2WPKH) keys=2 thr=2
                                     ENCODE ok: wpkh(sortedmulti(2,…))#6cc6zuge
```

The device's parser does **not** accept the `multi` spelling, and the quoted `address:` error is
never produced because `address.Receive` is never reached. So the transposed row tells the
operator, verbatim, that the device accepted something it refused — and §11 item 4 asserts §6's
rows *"assert the *text*, not just the exit code"*, so this text gets pinned in a test.

(The exemption's own new claim, *"all three `multi` twins are device REFUSE (measured)"*, is the
same probe and is **TRUE** — r6 had measured only `wpkh(multi(…))`; all three now stand.)

**Required.** Give the `multi` case a complete text rather than a remedy fragment, and exempt
the whole cell rather than its remedy clause — e.g. *"a multisig policy cannot live inside a
single-key script. The device's parser refuses this spelling outright (measured), and `me`
refuses it on both `--as` paths. `multi` is carried by `--as md1` under `wsh(multi(…))`,
`sh(wsh(multi(…)))` or `sh(multi(…))` — change the wrapper, keep the form."* Then correct the
head sentence to say the row's `sortedmulti` mentions are in the remedy **and in the device
measurement**.

**Evidence:** spec lines 1042–1046 (the exemption), 1075 (the row), 630–646 (conjunct 1's three
md1-path twins), 712–716 (conjunct 7's own no-loop rule), 1452–1454 (§11 item 4); the four-probe
Go measurement above.

---

## NEW — Minor

**NEW-M1 — the double-count permission is unconstrained, and no total row count is pinned, so
the minima are not equivalent to physical rows.** The manifest says *"A row may carry two tags"*
and justifies it with one example (`xpub…\n` is both a promotion near-miss and a whitespace
row). Nothing says **which** pairings are legitimate, and the test cannot check a tag against
the row's content. So r6's construction has a surviving variant: drop the childless+`<0;1>/*`
mixed row **and** retag any one of the fourteen `narrowed-4.7` rows as
`["narrowed-4.7","md1-splits"]`. Counts: `md1-splits` 6 ✓, `narrowed-4.7` 14 ✓, no unknown
tags ✓, `covers` present on every row ✓ — the manifest passes with the row that gates R0 r3's
NEW-C1 absent. (A second, cheaper variant if the test counts tag *occurrences* rather than
distinct rows, which is the natural implementation of *"counts the `covers` tags"*: one row
carrying `["md1-splits","md1-splits"]`. The column header says "min rows"; the prose says
"counts tags". Nothing forbids a duplicate inside the array.) This is a strictly weaker threat
than r6's — it needs a deliberately false tag rather than a silent omission, and it is the
residual any hand-authored coverage annotation carries — so it is Minor, not a re-raise of
NEW-I2. Two sentences close it: *"a row may carry a second tag only where its input genuinely
discharges both bullets; in the required set the only such row is `xpub…\n`"*, plus a pinned
floor — the minima sum to **51** tag-slots over **50** physical rows (49 if the `formats-happy`
promoted-key row is authored as §4.5's own bare-`xpub` row), so *"the file has at least 50 rows,
and `covers` entries are distinct within a row"* makes the omission uncountable-around.

**NEW-M2 — the `panic:encode` requirement on the zero-key row is true of one spelling and the
bullet does not pin it.** §7 now requires *"the no-`Format:` and zero-key rows carry
`device_probe: "panic:encode"` (parse ACCEPTS, `Encode()` panics — measured)"*. Measured this
round, the zero-key case is spelling-dependent, and only §4.2 defect 2's exact spelling holds:

```
Name: only\n                                  ACCEPT keys=0 script=Unknown  -> ENCODE PANIC: unknown script
Name/Format: P2WSH/Derivation, no key lines   ACCEPT keys=0 script=P2WSH    -> ENCODE ok: wsh(sortedmulti(0,))#w47tv00x
Name/Policy: 2 of 2/Format/Derivation, none   REFUSE: unrecognized output descriptor format
```

The panic is the *unknown-script* panic (defect 1's), not a zero-key panic — a zero-key file
that carries `Format:` encodes cleanly, and one that also carries `Policy:` is a parse refusal
at the `nkeys != len(desc.Keys)` check. So an author who writes the realistic four-header
zero-key file and marks it `panic:encode` as §7 instructs has written a false marker. The cost
is only a suppressed `Encode` call (no false pass, no crash — and `address.Receive` on an
Unknown-script descriptor errors rather than panicking, probed above), which is why this is
Minor. Fix: pin the spelling in the bullet — *"the zero-key row is §4.2 defect 2's `Name: only`
file; with a `Format:` header present the encode succeeds and the marker does not apply."*

---

## NEW — Nit

**NEW-N1 — §11 item 3's *"the field present on every row"* is singular, and two schema bullets
now claim it.** The `md1_admits` bullet (untouched by this fold) says *"§11 item 3's counting
test asserts the field is present on every row"*; the `covers` bullet routes its own presence
check to item 3; and item 3's new parenthetical names *"the field"* once, reading as `covers`.
Name both.

**NEW-N2 — §8 still names only §11 item 6 as S2-bound, now that items 1 and 4 carry the same
scoping.** No contradiction — §8 asserts nothing false, and its conclusion (*"S1 and S3 can
plan, build, demonstrate and ship entirely at the desk"*) is better supported after the fold
than before it. But §8 is where a reader goes for the phase story, and it now understates which
acceptance items are parked. One clause: *"§11 items 1, 4 (its `--as descriptor` rows) and 6
bind S2's ship."*

---

## Cross-reference check

Every `R0 r6's NEW-XX` citation resolves to a real r6 finding, and each lands on the finding it
names. All eleven are cited; none is orphaned.

| citation | spec lines | lands on |
| --- | --- | :-: |
| NEW-I1 | 1159, 1267 | assertion paragraph ✓ |
| NEW-I2 | 1187, 1244, 1451 | `covers` / manifest / item 3 ✓ |
| NEW-M1 | 1183 | `device_admits` omitted on panic rows ✓ |
| NEW-M2 | 1171 | `host_admits` defined ✓ |
| NEW-M3 | 1156 | `format` defined ✓ |
| NEW-M4 | 1046, 1075 | single-key-wrapper exemption ✓ |
| NEW-M5 | 1439, 1454 | §11 items 1 and 4 ✓ |
| NEW-M6 | 1176, 1226 | panic split + the two encode rows ✓ |
| NEW-N1 | 660, 1213 | 16-key recorded construction ✓ |
| NEW-N2 | 1272 | Go derivation source ✓ |
| NEW-N3 | 1126 | `package nonstandard_test` ✓ |

§4.2's marker reference (line 354) reads `device_probe: "panic:parse"` — correct for defect 4,
the parser panic. §11 items 1 / 4 / 6 and §8's F-418 text are mutually consistent (§8 asserts
only item 6's scoping and the S1/S3-at-the-desk conclusion; both hold).

---

## The eight desk-run rows, walked against the NEW assertion paragraph

Every assertion is now assignable, and no row carries a field with contradictory instructions.

| row | `device_admits` | `md1_admits` | fields carried | Go asserts | Rust asserts | verdict |
| --- | :-: | :-: | --- | --- | --- | :-: |
| 1 `multi` | false | true | `address_0`, `address_1`, `md_descriptor_contains` | nothing (device false) | both addresses via md1 + the read-back | assignable; read-back inert (**NEW-I1**) |
| 2 `/0/*` | true | false | `address_0` | `address_0` on parsed input | md1 REFUSES citing §5.3(a) | ✓ |
| 3 `<0;1>` | true | false | `address_0` | `address_0` on parsed input | md1 REFUSES citing §5.3(a″) | ✓ |
| 4 mixed | true | true | `address_0` | `address_0` on parsed input | `address_0` via md1 | ✓ both routes, one field |
| 5 trailing `\n` | false | true | `address_0`, `canonical` | req. 4 only: parse `canonical`, fixed point | `address_0` via md1 | ✓ no address on the Go side, by design |
| 6 C1 no-`Derivation:` | true | false | `address_0` (no `canonical`) | `address_0` on the parsed **INPUT** — re-measured `bc1qadgf37z…` | not "otherwise ADMITTED" (§4.2 refuses it), so no refusal-citation fires | ✓ NEW-N2 is what makes this row work |
| 7 short-fp panic | **omitted** | false | none | nothing — the address rule keys on *"is true"*, and absent is not true; req. 5 skips | nothing | ✓ |
| 8 16-key `sh(wsh(…))` | true | true | `address_0` | on parsed input | via md1 | ✓ |

The brief's two named traps are both clean. Row 6 — `address_0` on a `host_admits=false,
device_admits=true` row — is well-defined because the Go route keys on `device_admits` and
derives from the **input**, which is the only string that exists for that row. And the
`panic:encode` rows keep `device_admits=true` (parse accepts; measured) while only `panic:parse`
rows omit it, so requirement 5's `device-only` count includes the encode-panic rows and excludes
the parse-panic one — which is exactly what the two markers mean.

---

## Measurements taken this round

Everything below was RUN. Nothing was written to any repo.

```
md1 (descriptor-mnemonic/target/release/md), keys dc567276 / f245ae38, --path m/48'/0'/0'/2'
  wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))         chunk-set-id 0xd5e52
    address --index 0  bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
    address --index 1  bc1q24khjdhxz70zs228ymlljjrtppfp7swz90j4l82ph65zcmwujx0sj6v06y
    md descriptor      wsh(multi(2,…))#656zkmsn
  wsh(sortedmulti(2,…))                        chunk-set-id 0x16d62
    address --index 0  bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a   IDENTICAL
    address --index 1  bc1q9edtz99nhdf95kaltjk8xtzrxt0ysekrytv4vp69psdf5y7mnamsar8nzf   DIVERGENT
    md descriptor      wsh(sortedmulti(2,…))#l9ucx0pn
  substring test, md_descriptor_contains: "multi("
    vs multi read-back        CONTAINS  -> PASSES
    vs sortedmulti read-back  CONTAINS  -> PASSES   <-- NEW-I1
  substring test, "wsh(multi("
    vs multi read-back        CONTAINS  -> PASSES
    vs sortedmulti read-back  absent    -> FAILS    (the working pin)
  cosigner order reversed (@0=K2, @1=K1)
    multi 0x57db8       idx0 bc1q93wr5sn…  idx1 bc1q9edtz99n…
    sortedmulti 0xfcb08 idx0 bc1qadgf37z…  idx1 bc1q9edtz99n…
    -> the divergence moves from index 1 to index 0; carrying BOTH addresses covers
       both orderings, so the row is not inert despite the dead read-back

Go (nonstandard.OutputDescriptor / bip380.Descriptor.Encode / address.Receive), fork 0b656d7
  wpkh(multi(2,…))                     REFUSE: unrecognized output descriptor format
  pkh(multi(2,…))                      REFUSE: unrecognized output descriptor format
  sh(wpkh(multi(2,…)))                 REFUSE: unrecognized output descriptor format
  CONTROL wpkh(sortedmulti(2,…))       ACCEPT Segwit (P2WPKH), Encode ok #6cc6zuge
    -> the exemption's "all three multi twins are device REFUSE (measured)" is TRUE
    -> the row's parenthetical "the device's parser accepts this spelling" is FALSE
       for multi, and still transposes (NEW-I2)
  BlueWallet no Format:                ACCEPT keys=2 script=Unknown -> ENCODE PANIC: unknown script
  BlueWallet "Name: only\n"            ACCEPT keys=0 script=Unknown -> ENCODE PANIC: unknown script
  BlueWallet zero keys + Format:P2WSH  ACCEPT keys=0 script=P2WSH   -> ENCODE ok: wsh(sortedmulti(0,))#w47tv00x
  BlueWallet zero keys + Policy+Format REFUSE: unrecognized output descriptor format
  address.Receive on Unknown-script    ERROR "address: multisig script: Unknown: unsupported
                                       descriptor" — an ERROR, not a panic, so a panic:encode
                                       row carrying an address cannot crash the suite
  address.Receive on parsed C1 file    bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a

Package layout
  nonstandard/*_test.go is `package nonstandard`; nonstandard does not import sysw today
  -> `package nonstandard_test` is legal alongside it, and NEW-N3's cycle is the future one
```

---

## Closing

**10/11 FIXED, 1 PARTIAL; 0C / 2I / 2M / 2N new. The correctness lens does not close.**

The fold got the two hard things right. The assertion paragraph now consumes every value a row
carries and the `multi` → `sortedmulti` mutant **fails**, measured — that gate, open since r5,
is shut. And the coverage manifest is arithmetically sound: I counted all eight minima against
the bullets' own enumerations and all eight match, and under honest tagging the omission of
*any* required row now breaks a count, not only r6's constructed one.

What is left is the fold's own new sentences, and both Importants are the same lesson the file
keeps teaching. **NEW-I1** is the third round in a row on the `multi` row's gate: r5 added the
values without the assertions, r6 got the assertions added, and the assertion that landed
carries a substring that is a substring of what it is supposed to reject. A `grep -F` would have
settled it in a second, which is the build-gate rule pointing straight at the class of claim
that keeps surviving: a value quoted inside prose, never executed. **NEW-I2** is
incomplete propagation, this time inside the fix for an incomplete propagation — r6 named two
non-transposing parts of the §6 row and the fold replaced one of them, with a remedy that names
the flag the operator already typed.

Neither is a design defect. NEW-I1 is one string (`"wsh(multi("`, measured working above).
NEW-I2 is one table cell and one clause.

**For the record, on what remains after these close** — the spec's own text, not new findings:
§11's items 1, 4 (partly) and 6 are parked with S2 under F-418; §9's residual unverified items
are change-chain and testnet address equality (item 3, correctly narrowed by this fold), the
never-executed `ClassDescriptor` display path (item 2), and the `md`-binary-vs-`md-codec`-0.42.0
version note (item 4). The §6 journey walk has not been done, and NEW-I2 is precisely the shape
a journey walk generates — an operator who does the thing the refusal told them to do and lands
back where they started — which is an argument for scheduling it rather than for another
correctness round.
