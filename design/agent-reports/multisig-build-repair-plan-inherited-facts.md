# Inherited-fact audit — IMPLEMENTATION_PLAN_multisig_build_repair.md

**Verdict: NOT clean — 1 Critical, 2 Important, 5 Minor. The Critical is S0's own
oracle table: `BIP-382` contains no `multi()` and no address strings at all, and
neither BIP-141 nor BIP-143 contains a single address string, so two of the three
tests that constitute S0's gate cannot be written from the sources the plan cites.**

Scope: inherited-vs-tested only. No correctness, design or severity re-review.
Reviewer: independent context, 2026-08-13. Both repos left clean
(`git status --short` empty in `seedhammer` @ `a10d007` and `mnemonic-engrave`);
all scratch artifacts removed.

Headline counts: **41 material assertions** enumerated. **19 were already TESTED**
(this cycle or by me). **22 were INHERITED**; I executed a check against **18** of
them. Of those 18, **13 came back TRUE**, **5 came back FALSE or hollow**.

---

## 1. The inherited-vs-tested table

`T` = tested (a command was run and its output read). `I` = inherited (entered
from a doc comment, changelog, prior report, another design doc, or reasoning).
`I→T` = inherited into the plan, executed **by me** in this pass.

### 1a. The oracles

| # | the fact, as the plan states it | where it entered | state | cost to check / what breaks if false |
| --- | --- | --- | --- | --- |
| 1 | `ms encode --hex <entropy>` exists | §1a md-table, "this is C1" | **I→T PASS** | 1 cmd. C1's whole fix |
| 2 | …and is deterministic | same | **I→T PASS** | 20 runs. S5 gate is a string compare |
| 3 | the fork's ms1 equals that string | §1a "full string equality", S5 gate | **I→T PASS** | 20 min. **S5's ms1 gate** |
| 4 | primary `mk` mints a random `chunk_set_id` per encode | §1a mk-table | **I→T PASS** | 5 runs |
| 5 | …with **no CLI override** | §1a; "file it, do not build it" | **I→T PASS** | `mk encode --help` |
| 6 | `canonical_payload_bytes` exists in mk-codec | §1a relation (b) | **I→T PASS** | grep |
| 7 | …and is chunk-set-id-independent | §1a relation (b) | **I→T PASS** (doc + `encode_bytecode` call) | reading |
| 8 | **relation (b) is computable with the pinned primary toolchain** | *never stated — assumed* | **I→T FAIL** | **I-1 below** |
| 9 | relation (a): primary `mk decode` accepts fork chunks | §1a relation (a) | **I→T PASS** | fork-encoded card decodes, exit 0 |
| 10 | md1 is "deterministic on both sides" → full string equality | §1a md1 row | **I→T PASS** | 20 identical md5s |
| 11 | pins are md-codec 0.42.x, mk-codec 0.4.2, ms-codec 0.7.0 | §1a | **I→T PARTIAL** | **M-2 below** |
| 12 | "or `me`, which pins it" | §1a | **I→T FAIL for mk** | **M-2 below** |
| 13 | md-codec byte-stable 0.36.0 → 0.42.0 | changelog; plan says *no machine has checked it* | **I→T PASS** | **M-1 below** |
| 14 | fork md vectors pinned v0.36.0 / `c85cd49` | README | T (prompt) | — |
| 15 | "the drift is already on disk and **measured**" | §1a | **I→T FALSE** | **M-1** |
| 16 | F-127 records what vendored 0.34 vs 0.42 primary cost | FOLLOWUPS | **I→T PASS** (cause is exactly that) | note: F-127 was *downgraded to Minor* |
| 17 | four host-side defects F-127/128/130/140 exist | FOLLOWUPS | **I→T PASS** | all four resolve |
| 18 | `bip341-wallet-test-vectors.json` vendored in md-codec | §1a precedent | **I→T PASS** | present |
| 19 | `bip-test-vector-audit-matrix` reports exist in mk/ms | §1a precedent | **I→T PASS** | present |

### 1b. Oracle 2 — the published-BIP table (§1a)

| # | the fact, as the plan states it | where it entered | state | what breaks if false |
| --- | --- | --- | --- | --- |
| 20 | **BIP-67** gives deterministic key-sorting vectors | §1a BIP table, S0 test 2 | **I→T PASS** | — |
| 21 | **BIP-382** gives `wsh(multi(…))` → **address** vectors | §1a BIP table, S0 test 1 | **I→T FALSE ×2** | **C-1** |
| 22 | **BIP-141/143** give P2SH-P2WSH **address** vectors | §1a BIP table, S0 test 3 | **I→T FALSE** | **C-1** |
| 23 | **BIP-32** gives derivation vectors at `m/48'/0'/…` | §1a BIP table, S5 Trace B | **I→T FALSE** | **C-1** |
| 24 | BIP-39 `abandon…about` already used | §1a | T | — |
| 25 | `address/address_test.go` fixtures carry no provenance | plan §1a | **I→T PASS** | — |
| 26 | `bip380/bip380_test.go` has two tests, neither cites a BIP | plan §1a | **I→T PASS** | — |

### 1c. Severity and consensus claims

| # | the fact, as the plan states it | where it entered | state | what breaks if false |
| --- | --- | --- | --- | --- |
| 27 | `sortedmulti(2,K,K,X)` is spendable by K alone | §4.1 / S4 test 4 — *reasoning* | **I→T PASS** | severity of the duplicate-key refusal |
| 28 | the device has no miniscript-level duplicate-key guard | CLAUDE.md Rust-primary note | **I→T PASS** (miniscript `sanity_check` catches it; fork omits miniscript) | reinforces S4 |

### 1d. Code citations (16 claims, delegated and machine-checked)

| # | claim | state |
| --- | --- | --- |
| 29 | `gui/multisig_build.go:54` seeds **one** `syswOffer` record | **I→T TRUE** (`syswBundleSeed` is a `string`, `gui/gui.go:69`) |
| 30 | `gui/bundle_flow.go:100-103` states why not to add a 2nd insertion path | **I→T TRUE** |
| 31 | `md/encode_multisig.go:13-21` — slot order is identity-bearing | **I→T TRUE** |
| 32 | `md/encode_multisig.go:104-106` — `errMultisigEmptyDivergent` on `Path=="m"` | **I→T TRUE** (`bip32.ParsePath("m")` → empty slice) |
| 33 | `gui/multisig_match.go:34` `findUserSlot` derives at each key's own `OriginPath` | **I→T TRUE** |
| 34 | three `scriptName` callers are the complete set; none outside `gui` | **I→T TRUE** |
| 35 | `md.Template` carries `Root ScriptKind` / `InnerWsh` / `InnerWpkh`; `ScriptSh` exists | **I→T TRUE** |
| 36 | `buildPolicyParams.SelfSlot` is an `int` | **I→T TRUE** |
| 37 | `cosignerFromCard` discards `card.Origin` | **I→T TRUE** |
| 38 | `buildCosignerCards` refuses md1 records | **I→T TRUE** |
| 39 | `multisigSharedOrigin()` returns `m/48'/0'/0'/2'` | **I→T TRUE** |
| 40 | `deriveMultisigLeg` derives at the slot's origin | **I→T TRUE** |
| 41 | `syswSession.take` guard is `!loaded \|\| !compared` | **I→T TRUE** (literal) |

Also tested and TRUE: `testing/synctest` is already in use in `gui/`; `~/bin/sh/sh2-flash`
exists; `design/SPEC_systemwide_payloads.md` exists; F-151's "first guess of 2000 px
passed the defect" matches the record (blank = 2652 px, fix = 6688 px).

Not checkable, correctly flagged by the plan as such: the S6 hardware runs, the
external-coordinator restore (Oracle 3), and the emulator's inability to reach NFC.

---

## 2. What I actually ran

### 2.1 `ms encode --hex` exists and is deterministic — **PASS**

```
$ ms --version
ms 0.14.0
$ ms encode --help | head -5
Usage: ms encode [OPTIONS] <--phrase <PHRASE>|--hex <HEX>>
      --hex <HEX>   Hex-encoded entropy bytes (16/20/24/28/32 B = 32/40/48/56/64 hex chars)

$ for i in $(seq 20); do ms encode --hex 0f0e...0100 --no-engraving-card --group-size 0; done | sort -u | wc -l
1
```

### 2.2 The fork's ms1 **byte-matches** the primary at all five BIP-39 lengths — **PASS**

Scratch Go module with `replace seedhammer.com => /scratch/code/shibboleth/seedhammer`,
calling `codex32.EncodeMS1` directly, compared against `ms encode --hex`:

```
MATCH  len=32 ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
MATCH  len=32 ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
MATCH  len=32 ms10entrsqrlllllllllllllllllllllllllsnrx3qek9l5q60
MATCH  len=64 ms10entrsqq8surgvpv9qjzq8qczsgqczqyqq7rsdps9s5zggqurq2pqrqgqsqq99z3pj27f5fk
MATCH  len=40 ms10entrsqplhultu0da8j7rhwe6hgumjw9cx7mndds5cs4tk7tysmg8
MATCH  len=48 ms10entrsqzyfn24menw7alcqzy3rx3z4vemc3xd2h0xdmmhldqvkeslx9sv9x
MATCH  len=56 ms10entrsqqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8q4cvxxq7ewf8je
```

7/7 byte-identical, covering 16/20/24/28/32-byte entropy. **C1's oracle is real and
the relation it asserts already holds today.** This is the strongest result in this
pass and it removes the largest unknown from S5.

### 2.3 `mk` randomizes `chunk_set_id`, no CLI override — **PASS**

```
$ for i in 1..5; do mk encode --xpub xpub6Den8… --origin-fingerprint aabbccdd \
    --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344 --group-size 0; done
mk1qpx9rtpqq…  |  mk1qpx9rtpps…
mk1qpq065pqq…  |  mk1qpq065pps…
mk1qpv5lmpqq…  |  mk1qpv5lmpps…
mk1qp9hvgpqq…  |  mk1qp9hvgpps…
mk1qpf2depqq…  |  mk1qpf2depps…
```

Five distinct outputs from identical inputs. `mk encode --help` has no
`--chunk-set-id`. Additionally: **every** card in the 19-entry `mk vectors` corpus
is ≥ 2 chunks (min `total_chunks` = 2), so there is no single-chunk deterministic
escape hatch. The plan's ruling of a weaker mk1 plane is fully justified.

### 2.4 `sortedmulti(2,K,K,X)` is spendable by K alone — **PASS (confirmed)**

Scratch crate, `miniscript 13.1.0` + `bitcoin 0.32.102`:

```
descriptor: wsh(sortedmulti(2,034f355b…71aa,034f355b…71aa,02466d7f…3f27))
PARSE: OK
address: bc1qkgun8x9xkdrr4835tfrsz8qx8xnt3q3zhew24l4cktvnfutsrx5q2jg93d
sanity_check: ERR Miniscript contains repeated pubkeys or pubkeyhashes
witness script: OP_PUSHNUM_2 <X> <K> <K> OP_PUSHNUM_3 OP_CHECKMULTISIG
SATISFACTION WITH K ALONE: FOUND, 4 witness items
  [1] 72 bytes: 3045022100b5bc82ee…8f3301
  [2] 72 bytes: 3045022100b5bc82ee…8f3301     <-- byte-identical, RFC-6979
```

A complete witness is produced from K's signature alone. `OP_CHECKMULTISIG` does not
require distinct signatures, and deterministic nonces make the two byte-identical.
**The severity argument behind S4 test 4 holds.** Two useful corollaries: the
descriptor still yields a spendable address (so this is fund loss, not an
unparseable backup), and rust-miniscript's `sanity_check` *does* refuse it — a guard
the device does not have, because the fork deliberately omits miniscript for TinyGo.
S4's gate is the only thing standing here.

### 2.5 md-codec is byte-stable 0.36.0 → 0.42.0 — **PASS (first machine check)**

```
$ for f in $FORK/*.{bytes.hex,phrase.txt,descriptor.json}; do cmp -s "$f" "$PRIM/$(basename $f)"; done
identical=30 differs=0 missing=0
```

All 10 vendored vectors × 3 files, byte-identical between the fork's v0.36.0 copy and
`descriptor-mnemonic` @ `5a0a4f41` (md-codec 0.42.0). See M-1.

### 2.6 md1 chunked encoding is deterministic — **PASS**

```
$ for i in $(seq 20); do md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))" \
    --key @0=… --key @1=… --fingerprint … --path bip48 --force-chunked --json > r$i.json; done
$ md5sum r*.json | awk '{print $1}' | sort | uniq -c
     20 9547b7bc132a5b2e4fa58ba4c92bd86f
```

`chunk_set_id` is derived, not drawn (`md/chunk.go:130 deriveChunkSetID(id)` on the
fork side; same result on the primary). §1a's md1 full-string-equality plane holds.

Two by-products worth recording, because neither is in the plan:

- **Every multisig md1 in this plan is chunked.** A minimal 2-key shared-origin
  `wsh(sortedmulti)` measures **246 data symbols against a single-string cap of 80**
  (`md: payload is 246 data symbols; the codex32 regular code caps single strings at
  80`). S2's and S5's md1 comparisons are chunked-string comparisons, and the fork's
  own `md/testdata/README.md` excludes its chunked vector from the parity table.
- **Divergent origins *are* expressible on the primary CLI**, via inline per-`@N`
  origins in the template — `md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,
  @1/48'/0'/1'/2'/<0;1>/*))"` produced a 4-chunk md1. `--path` alone cannot (it
  "flattens Divergent mode to Shared"), so anyone reading only `--help` would
  conclude Trace B has no md1 oracle. It does. Worth one sentence in S0.

### 2.7 Relation (a) works; relation (b) has no tool — see I-1

```
$ mk decode --json <two chunks produced by the FORK's mk.Encode>
{"chunks":2,…,"origin_fingerprint":"73c5da0a","origin_path":"48'/0'/0'/2'",
 "policy_id_stubs":["11223344"],"xpub":"xpub6DkFAXW…"}       exit=0
$ mk verify --xpub xpub6DkFAXW… --origin-fingerprint 73c5da0a \
      --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344 <chunks>
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub … match)
exit=0
$ mk bytecode --help
error: unrecognized subcommand 'bytecode'
```

### 2.8 The BIP documents — see C-1

```
$ curl -s .../bip-0382.mediawiki | grep -c 'multi('      → 0
$ curl -s .../bip-0382.mediawiki | grep -ci 'address'    → 0
$ curl -s .../bip-0383.mediawiki | grep -c 'multi('      → 23   (title: Multisig Output Script Descriptors)
$ curl -s .../bip-0383.mediawiki | grep -ci 'address'    → 0
$ curl -s .../bip-0143.mediawiki | grep -ci 'address'    → 0
$ curl -s .../bip-0141.mediawiki | grep -ci 'address'    → 4    (all prose; no address string)
$ curl -s .../bip-0032.mediawiki | grep -n "48'"         → (no output)
$ curl -s .../bip-0067.mediawiki | grep -ci 'address'    → 13   (List / Sorted / Script / Address, ×4)
```

### 2.9 `TYPED-ONLY` occurrence count — see I-2

```
$ grep -rn 'TYPED-ONLY' --include='*.go' .
gui/multisig_build.go:67   gui/singlesig.go:18   gui/singlesig.go:32
gui/bip85.go:264           gui/bip85.go:270      gui/multisig.go:17
gui/multisig.go:24         gui/multisig.go:60    gui/multisig.go:102
$ … | wc -l
9
```

---

## 3. Findings

### C-1 (Critical) — S0's BIP table is wrong in three of its five rows, and two of S0's three gate tests cannot be written from the sources cited

S0 exists so that "a gate anchored to a stale or unattributed oracle" does not read
as proof. Its own oracle table was never opened.

| plan's row | reality, fetched from `bitcoin/bips@master` |
| --- | --- |
| **67** — deterministic key sorting → S5 | **TRUE.** 4 vectors, each `List` → `Sorted` → `Script` → **`Address`** (P2SH). Directly usable |
| **382** — `wsh(multi(…))` → **address** → S2 | **FALSE twice.** BIP-382 is *Segwit Output Script Descriptors*; `grep -c 'multi('` = **0** — every `wsh()`/`wpkh()` vector wraps `pk()`/`pkh()`. `multi()`/`sortedmulti()` vectors live in **BIP-383** (*Multisig Output Script Descriptors*, 4 `wsh(multi/sortedmulti` vectors). And **neither document contains a single address string** — both give scriptPubKey hex only |
| **141/143** — P2SH-P2WSH **addresses** → S3 | **FALSE.** `grep -ci address` = **0** in BIP-143 and 0 address *strings* in BIP-141 (its 4 hits are prose). BIP-143's P2SH-P2WSH case is a sighash worked example supplying scriptPubKey / redeemScript / witnessScript hex. There is no published address to compare against, so `TestBip143NestedSegwitAddressDiffersFromLegacy` "at the **address** level" has no source |
| **32** — derivation at `m/48'/0'/…` → S5 Trace B | **FALSE.** The five vector sets cover `m/0H/1/2H/2/1000000000`, `m/0/2147483647H/…` and three short chains. `grep -n "48'"` returns nothing. Trace B's held-slot origins have no published vector |
| **39** — mnemonic → seed | TRUE, already in use |

Consequence, stated in the plan's own gate language — S0's gate is *"The three BIP
vector tests pass"*. As specified, **one of the three is buildable**.
`TestBip382WshMultiAddressesMatchPublishedVectors` names a BIP that has no `multi()`
in it; `TestBip143NestedSegwitAddressDiffersFromLegacy` names an assertion level that
no cited BIP supplies. Every later stage's address correctness is anchored to S0.

This is the same shape as the `mk` V19 claim and D-5: a plausible, load-bearing,
never-executed citation. It is the third instance and the most expensive, because
S0 is the stage whose entire purpose is to be the thing others trust.

**It is repairable without redesigning S0**, and cheaply:
- retarget test 1 at **BIP-383**, and compare **scriptPubKey**, not address (the
  `wsh(multi(…))` and `wsh(sortedmulti(…))` vectors are there and are
  copy-pasteable);
- for test 3, take the P2SH-P2WSH **scriptPubKey/redeemScript** from BIP-141's
  Example section and derive the address locally — then the address is *derived from*
  a published vector rather than *quoted from* one, which is a weaker but honest
  claim, and the plan should say which it is;
- for BIP-32 at `m/48'`, accept that no published vector exists and say so in S0's
  provenance README rather than implying one was used. (BIP-48 publishes no vectors
  either — its `==Examples==` table is a path-semantics table with no keys.)

What must **not** happen is the gate quietly relaxing to "the tests we could write
passed". S0's own deliverable 3 makes exactly this point about
`address/address_test.go`'s unattributed fixtures.

### I-1 (Important) — the mk1 comparison plane's relation (b) has no host-side tool

§1a rules mk1 to **(a)** primary decode accepts, **AND (b)** `canonical_payload_bytes`
equality. (a) works (§2.7). (b) is a Rust **library** API — `pub fn
canonical_payload_bytes` at `mk-codec/src/key_card.rs:116` — and the `mk` CLI has no
surface that emits it. `mk`'s subcommands are encode/decode/inspect/verify/vectors/
gui-schema/repair/address/derive/gen-man; there is **no `bytecode`** (md-cli has one).
`mk decode --json` and `mk inspect --json` carry no bytecode field, and `mk vectors`
prints `canonical_bytecode_hex` only for the pinned corpus, not for arbitrary input.
`mk-cli/src/cmd/inspect.rs:4-6` records the reason and is itself now stale: *"mk-codec's
bytecode-layer surface isn't public yet … `bytecode` subcommand deferred to v0.3"* —
the API **is** public at 0.4.2; only the CLI never caught up.

Why it matters: S0 must build the harness that later gates call, and the plan
explicitly refuses host-side CLI work this cycle ("A `--chunk-set-id` flag … **File
it, do not build it** — a host-side change with its own cycle"). Relation (b) needs
the same class of change and the plan does not notice.

**A substitute exists and I ran it** (§2.7): `mk verify --xpub --origin-fingerprint
--origin-path --policy-id-stub` returns `OK`/exit 0 on fork-produced chunks and pins
every field `canonical_payload_bytes` covers, with version and network pinned by the
decode itself. S0 should either name `mk verify` as relation (b)'s realisation, or
file the `mk bytecode` subcommand alongside `--chunk-set-id`. Left as written,
relation (b) is an assertion with no executor.

### I-2 (Important) — S3's grep gate asserts a post-state that does not exist: there are 9 `TYPED-ONLY` occurrences, not 4, and no verify sites

The plan says the *four* stale comments die (stage table, §3 S3, "a future reader
greps `TYPED-ONLY`, finds four hits"), and gates on:

> `grep -rn TYPED-ONLY` returns only the two verify sites, which are true.

Measured: **9 occurrences across 4 files** (§2.9). The plan names four
(`bip85.go:264`, `singlesig.go:18`, `multisig.go:24`, `multisig_build.go:67`); the
other five — `singlesig.go:32`, `bip85.go:270`, `multisig.go:17`, `:60`, `:102` — make
the *same retired claim* ("the seed comes from `seedEntryFlow` ONLY … never a scan",
"Never a scan"), in the same flows, and are equally false under the settled D-5
finding. **None of the nine is in a verify flow.** 9 − 4 = 5, so the gate as written
cannot pass, and there is no pair of "verify sites" for it to land on.

The spec is not at fault: §2.2 D-5's table is a table of four **`seedEntryFlow` call
sites**, and it is accurate. The plan converted that into a grep over **occurrences**
without running the grep — "never hand-count what a tool can count", one layer up.
Fix: name all nine, and set the gate's expected residue from a measured number.

### M-1 (Minor) — "the drift is already on disk and measured" is false for md; S0-4 is smaller and differently shaped than described

§1a justifies the re-pin with *"The drift is already on disk and measured … The
changelogs claim byte-stability across that gap; no machine has checked it."* Now
checked: **30/30 files byte-identical, zero drift** (§2.5). What is measured is a
version-*label* gap, not content drift. The changelogs were right.

What the re-pin actually buys is **coverage**, not repair: the primary has grown five
vectors the fork does not vendor — `nums_taproot`, `sh_wpkh`, `single_string_boundary`,
`tr_with_leaf`, `wsh_sortedmulti_2chunk`. `sh_wpkh` is directly relevant to S3's
P2SH-P2WPKH naming work. S0-4 should be restated as *provenance-header update + pick
up 5 new vectors*, which also settles its own escape hatch ("if it proves larger than
S0 should carry it becomes its own stage") — it will not.

Related nuance on the same paragraph: **F-127, cited as the precedent for what
vendored drift costs, was explicitly downgraded to Minor** ("ergonomics, not a binding
failure"). Its cause *is* a stale vendored md-codec (0.34 vs 0.42), so the citation is
accurate; the implied severity is not. F-130 is the entry that carries the wrongness
argument, and the four-defect list survives on it.

### M-2 (Minor) — the pin line is off by one patch, and `me` is not a valid mk oracle

§1a: *"The pins today are `md-codec 0.42.x` (or `me`, which pins it), `mk-codec 0.4.2`,
`ms-codec 0.7.0`."* Measured in `mnemonic-engrave/Cargo.lock`:

| crate | `me` pins | source of truth | match |
| --- | --- | --- | --- |
| md-codec | 0.42.0 | `descriptor-mnemonic` 0.42.0 | ✅ |
| ms-codec | 0.7.0 | `mnemonic-secret` 0.7.0 | ✅ |
| **mk-codec** | **0.4.1** | `mnemonic-key` **0.4.2** | ❌ one patch behind |

The plan makes the walk script print oracle versions precisely so a stale oracle is
visible. Since `me` is offered as the resolution route, it must not be the stale one.
Separately, the `ms` on `PATH` is **0.14.0** against a repo at **0.14.1** — harmless
here (I read the changelog: 0.14.1 is test-only, `ms-codec` NO-BUMP, "no `ms` binary
… change", consistent with §2.2's clean match) but it is exactly the situation §1a's
"not whatever binary is on `PATH`" rule exists to catch, and it is live today.

### M-3 (Minor) — F-158's recorded owning phase contradicts the plan

`design/FOLLOWUPS.md:5197` still reads *"(owning phase: **`SPEC_multisig_build_repair.md`
P0**)"*. The GREEN spec moved it four times over (`SPEC:198, 239, 470, 773` — "moves to
its own later plan"), and plan §4 restates that. The decision is settled; the record
was never updated, so the grep the burndown rule depends on returns a stale answer and
reads as a P0-owned item being carried past its gate. One-line edit.

### M-4 (Minor) — README location

Plan §3 S0-2 says the vendored-vector provenance follows "the shape of
`md/testdata/README.md`". `README_multisig.md` and `README_singlesig.md` are one level
down in `md/testdata/vectors/`, not alongside it. All three carry versions/commits
(`c85cd49` / v0.36.0; the singlesig one also pins `mnemonic-toolkit` v0.58.1 @ `4e21d94`).

### M-5 (Minor) — the fork's ms1 encoder carries no provenance pin

`mk/mk.go:5` has one (stale, "mk-codec 0.2" — the known case). `codex32/msencode.go`'s
`EncodeMS1` has **none**: its header cites a design-doc recipe ("T6a-1, C4"), not an
ms-codec version or SHA. Project CLAUDE.md requires "a **provenance pin** (Rust crate +
version/SHA it tracks) … updated on every sync" per ported package. §2.2 shows the
behaviour is correct *today*, which makes this the cheapest possible moment to pin it —
S5 is where the ms1 gate lands.

---

## 4. What I could not check

- **S6 hardware** and **Oracle 3** (external coordinator) — no device in this context.
  The plan flags both.
- **Whether the fork's md1 chunk-set-id matches the primary's for the S2/S5 fixtures.**
  Both sides derive it rather than draw it (`md/chunk.go:130` / 20-identical-md5s on the
  primary), so the mechanism agrees, but I did not construct a fork-side `descriptor`
  for a Trace-A shape and diff the strings. It is a ~30-line Go harness and it is the
  one remaining piece of §1a's md1 row that is mechanism-verified rather than
  output-verified. Worth doing inside S0 rather than discovering at S2's gate.
- **Whether the four host-side defects F-127/128/130/140 are each still open** — I
  confirmed they exist and that F-127 was downgraded; I did not audit the other three's
  current status.

---

## 5. Bottom line

The plan's **secret-material** and **funds-severity** foundations are sound and are now
executed rather than argued: the ms1 oracle exists, is deterministic, and the fork
already byte-matches it at all five entropy lengths; `mk`'s randomised `chunk_set_id`
is real, so the weaker mk1 plane is correctly ruled; md1 really is deterministic on both
sides; and `sortedmulti(2,K,K,X)` really is spendable by K alone. All sixteen code
citations say what the plan claims they say.

What was never executed is the **outward-facing** layer — the published-BIP citations in
S0, the CLI reachability of relation (b), and a grep count. Two of those three are the
"S0 is the oracle everything else trusts" stage, and one of them (C-1) would have been
discovered by a reviewer only after the tests were written and found unwritable.

Three inherited-fact instances went into this cycle. This pass adds five, of which C-1
is the one that blocks.
