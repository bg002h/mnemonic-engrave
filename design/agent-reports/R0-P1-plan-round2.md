# R0 — `IMPLEMENTATION_PLAN_P1_me_container.md`, round 2

**Reviewer:** independent R0 agent, round 2. **Artifact:** v3, 939 lines (324 → 939,
+696/−92 at `65ad79a`, folded by an independent architect).
**Prior rounds:** `R0-P1-plan-round0.md` (5C/13I/5M on v1),
`R0-P1-plan-round1.md` (3C/11I/8M on v2, plus 13 FIXED / 4 PARTIAL / 1 NOT FIXED).

**Machine-checked before writing** — nothing below rests on reading a doc comment
or an earlier report:

- **All three cited gates re-run on this document.** `plan-cite-check.sh` → **45/53
  resolved, 8 dangling**, and the 8 are exactly 4 × `bitcoin-0.32.9` + 4 ×
  `mnemonic-transaction`. `plan-table-check.sh` → **84 rows, 0 malformed, exit 0**.
  `plan-fold-sweep.sh --terms <the six>` → **exactly 6 hits, one per term, all at
  lines 828–833 inside the self-reference block**. **All three reproduce the plan's
  stated values exactly.**
- **§3.1's whole arithmetic recomputed in Python**: `(32734−3)//2 = 16365`, `−43 =
  16322`; `3+2×(43+4080) = 8249` (58 over 8191); `3+2×(43+18583) = 37255` (4521
  over 32734); and every row of the five-row raw/chunks table. Also verified spec
  §2.3's own `chars` formula reproduces 2001 / 9383 / 18583 with **201** separators,
  i.e. LF-**joined**, so the plan's figures are right and not off by a trailing LF.
- **§1.1/§1.4's measurements re-derived twice**: once with an independent Python
  segwit parser, once with a scratch crate on `bitcoin = { version = "0.32",
  default-features = false, features = ["std"] }`. 222 B / 113 B / 109 signature
  bytes / txid `2dcf2b97…` / wtxid `d5717c03…` / `dSHA(raw_hex)ᵣ == wtxid` /
  top-20 display `0x2dcf2` / top-20 internal `0x30f6e` — **all exact**.
- `bitcoin-0.32.9` read at source: `Encodable for Transaction` (1239–1260),
  `uses_segwit_serialization` (1058–1065), `Decodable` (1263–1302).
- `me-cli`: `EXIT_OK/USAGE/REFUSED/INVALID` (main.rs:225–228), `read_records`
  (1211–1227) and its caller's `return 2` (899–904), the pack-failure `return 4`
  (983), `sysw::classify`/`unknown_reason`/`split`/`pack_deterministic`,
  `UnknownReason`, `sysw_error`, `print_mdmk_confirmation`, `Command::Show`.
- W1–W5 resolved line by line; the corpus `mt1_v1.json` parsed and its 6×87-char
  strings confirmed present; `grep -c 'Class::Transaction' crates/me-cli/src/sysw/`
  executed; `plan-build-gate.sh` executed (now **exit 3**, refusing).

---

## Part 1 — did the fold land?

### Round 1's three Criticals

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r1-C1** layout table says `INTERNAL` | **FIXED** | Line 78 now reads `` 6   32    txid        DISPLAY order (byte-reversed), witness-STRIPPED -- see §1.1 ``, and `plan-fold-sweep.sh` confirms `INTERNAL byte order` survives **only** at line 828 inside the terms list. |
| **r1-C2** body definition deleted | **PARTIAL** | The *definition* landed — §1.4's table gives both forms, and §1.1's dangling `§1.3 says` citation is gone. The *enforcement* did not: E11, the rule §1.4 calls *"the only check that fires"*, **provably does not fire on V18** — measured `serialize(deserialize(stripped_body)) == stripped_body` → **true** on the exact dependency §2.2 chooses. See **[C1]**. The CHUNKS half (E12/E13) did land. |
| **r1-C3** nothing creates `ClassTransaction` | **PARTIAL** | §2.4 names five sites and **all five resolve exactly** (`record.rs:27-28`, `:31-40`, `:50-55`; `mod.rs:124`, `:108-115`), and §4 gains step 6. But no named site can carry *which rule* was violated, so §1.5's *"one line … naming the record's index and the rule"*, E9's three distinct messages and V13 are unimplementable at W1–W5 — see **[C2]**; §6's own `grep` closure for the five sites is a false PASS — **[I4]**; and its `me sysw show` read-back names a capability `show` does not have — **[I5]**. |

### Round 1's eleven Importants

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r1-I1** E1 has no negative vector | **FIXED** | **V16** added: *"TLVs in DESCENDING tag order → REFUSED"*, and V2's row now says in its own words *"**Not E1's negative — that is V16 (r1-I1)**"*. |
| **r1-I2** "REFUSED" defined nowhere | **FIXED** (with a new defect) | §1.5 answers all four of layer / scope / stream+exit / what-runs-before in a normative table. The *exit code it picks* conflicts with the R0-GREEN spec — **[I2]** — but the definitional gap round 1 filed is closed. |
| **r1-I3** §2.2 "closes C3" overstated | **FIXED** | The `ACCEPTED COST` blockquote: *"**The true bound is "the bytes are a well-formed transaction", not "the bytes are not a secret."**"*, with the `OP_RETURN <32 bytes of seed>` construction spelled out. |
| **r1-I4** refusal coverage narrowed to E1–E10 | **FIXED** | §6.2 restores it as a four-row table (R2, R7, the TTY refusal, §2.3's decode-failure refusal) under *"**NORMATIVE: all four get a test that goes RED when its check is removed**"*. |
| **r1-I5** step 3 lost the content rule | **FIXED** | §4.3 rule 1 is now *"**THE BASE RULE** … a payload holding **no** `Class::is_secret()` record packs **UNSEALED**"*, and step 3's test column asserts it first. |
| **r1-I6** build gate is a structural false PASS | **FIXED** | §6.1: *"**NORMATIVE: `plan-build-gate.sh` is NOT a close condition for P1**, and this plan does not cite it"*, replaced by three gates whose measured values I reproduced exactly. |
| **r1-I7** V4's form unstated; V1/V2 unconstrained | **FIXED** (with a new defect) | Split into **V4a** (RAW) and **V4b** (CHUNKS), and §3 rules *"**NORMATIVE: the transaction under every vector below is the `even` vector**"*, which is segwit. That same NORMATIVE sentence makes V7 unconstructible — **[I3]**. |
| **r1-I8** the "opaque" sentence has no referent | **FIXED** | `grep -nic opaque design/SPEC_engrave_transaction.md` → **0**. §2's blockquote records the two commits and §6 says the item is *"**struck: it has no referent**"*. |
| **r1-I9** `--allow-weak` made a sealing determinant | **FIXED** | §4.3 rule 3: *"**`--allow-weak` IS NOT A PASSPHRASE MODE** and does NOT count as "an explicit flag" for rule 2"*, with the test named. |
| **r1-I10** `TO` label: no UTF-8 verdict, no bound | **FIXED** | **E14** + **E15** + **V21** (`74 6f ff 21`) + **V22** (`len=65` refused / `len=64` passes). |
| **r1-I11** fixed-width tag with a variable `len` | **FIXED** (with a residual) | **E16** + **V17** (`len=2`, and the near-misses `7` and `9`). E16's `0x03 → 4` half is unvectored — **[I7]**. |

### Round 1's eight Minors

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r1-M1** V15's justification false | **FIXED** | V15's cell now reads *"**(M1) Its v2 justification was false and is retracted**"*, and re-attributes C1's catch to V4a/V4b. |
| **r1-M2** Go citation for a Rust rule | **FIXED** | §1: *"that is **Go**, and P3's. The Rust site P1 edits is `crates/me-cli/src/sysw/record.rs:24-28`"* — resolved. |
| **r1-M3** near-miss count went six → seven | **FIXED** | §6: *"**The count is dropped; the rule is not**"*, followed by seven named P1 pairs. |
| **r1-M4** a floor stated as an equality | **FIXED** | §3.1: *"**The middle line is a FLOOR, not an equality.** 16,365 bytes is 32,730 characters"*. Recomputed: correct. |
| **r1-M5** the ```rust fence is a paraphrase | **FIXED** | `sed -n '17,27p'` on the real file matches the plan's block character for character, blank `///` line and *"and takes it as the caller already has it"* included. |
| **r1-M6** three record sources, no precedence | **FIXED** | §4.2: *"**NORMATIVE: `--in` > argv > stdin.**"*, plus the blank-line-filtering rule. |
| **r1-M7** 16,322 assumes the record is alone | **FIXED** | §3.1 (M7): *"For `k` records the bound is `Σ(record chars) + (k − 1) ≤ 32,734`, so **V7 is explicitly the single-record vector**"*. |
| **r1-M8** publishing bakes in a dead `bitcoin` | **FIXED** | §5 (M8): *"**Remove the line, land it in `mnemonic-transaction`, then publish.**"* Verified: 0 `bitcoin::` in `mt-codec/src/`, `bitcoin = "0.32"` at `Cargo.toml:19`. |

### Round 0's carry-overs

| finding | round-1 status | now | the sentence that settles it |
| --- | --- | --- | --- |
| **C1** byte order | NOT FIXED | **FIXED** | Both normative places — line 78's table row and §1.1 — now say DISPLAY, witness-stripped, and §6.1 gates the pair by grep. |
| **I3** fixed-width tags | PARTIAL | **FIXED** | E16 binds the decoder; V17 vectors `len ∈ {2,7,9}`. (Residual: tag `0x03` — **[I7]**.) |
| **I5** `me` gains a `bitcoin` dep | PARTIAL | **FIXED** | §2.2's decision table names crate, version `0.32`, `default-features = false, features = ["std"]`, and **"added in: §4, step 4"**. Verified: `bitcoin`'s `default = ["std", "secp-recovery"]`; `me-cli` declares no `bitcoin` today. (The *sibling* omission is **[I6]**.) |
| **I7** V7 + falsified spec numbers | PARTIAL | **FIXED** for the numbers | §6 owns **three** corrections; all three recomputed here and all three correct. V7's number is right but the vector is now unconstructible — **[I3]**. |
| **I9** passphrase-flag precedence | PARTIAL | **FIXED** | §4.3's four-rule ordering, base rule first. |

**Tally: 20 FIXED / 2 PARTIAL / 0 NOT FIXED** of round 1's 22, plus **5 of 5** of
round 0's carry-overs closed. Both PARTIALs are r1-C1..C3's siblings and are
re-raised below as **[C1]** and **[C2]**.

---

## The I4 push-back — was the architect right?

**YES, on the substance. Round 1's I4 stated a false fact and the fold corrected
it correctly.**

Round 1 wrote: *"The machinery exists in this repo — `scripts/check-refusal-coverage.sh`
and `scripts/mutate-refusals.sh` are both present."* Executed:

```
$ ls scripts/          # mnemonic-engrave
build-preview.sh  cdcread.py  fold-propagation-check.sh  gen-mt1-vectors.py
gen-sa-fixture.py  gui-shard-test.sh  mutation-run.py  pico2-bootkey-rehearsal.sh
plan-build-gate-go.sh  plan-build-gate.sh  plan-cite-check.sh  plan-cite-gate.sh
plan-fold-sweep.sh  plan-glyph-check.sh  plan-mutation-anchors.py
plan-table-check.sh  plan-wiring-check.sh  push-master.sh  release-scan-firmware.sh
sh2-flash  sign-firmware.sh  spec-check.py  spec-structure-check.sh
verify-returnsite-sweep.sh  ...

$ find . -name 'refusals.toml' -o -name 'check-refusal-coverage.sh' \
       -o -name 'mutate-refusals.sh'          # mnemonic-engrave
(nothing)

$ ...same find in /scratch/code/shibboleth/mnemonic-transaction
./scripts/check-refusal-coverage.sh
./scripts/mutate-refusals.sh
./crates/mt-cli/tests/refusals.toml
```

None of the three is in this repo; all three are in `mnemonic-transaction`. The
fold's §6.2 correction is right, and its supporting quote of spec §5 is verbatim —
`SPEC_engrave_transaction.md:1426-1428` reads *"`mt` has this machinery
(`refusals.toml`, `check-refusal-coverage.sh`, `mutate-refusals.sh`); the fork side
needs its equivalent."* The consequent scheduling call — P1 owns the per-refusal RED
test by hand, spec §6's **P6** row owns building the bijection sweep — matches spec
§6:1511 (*"P6 | both | Journeys and refusal coverage (§5)"*).

**One byte-level slip in the correction**, filed as **[M2]**: §6.2 says all three
*"live in `mnemonic-transaction/scripts/`"*. Two do; `refusals.toml` is at
`crates/mt-cli/tests/refusals.toml`.

I4's **substance** — that §6 had narrowed refusal coverage from "every refusal" to
"E1–E10", dropping four of P1's own refusals — was true and is separately **FIXED**
by §6.2's four-row table.

---

## Part 2 — defects in the fold

## [C1] E11 does not fire on V18. Measured: `serialize(deserialize(witness_stripped_body)) == witness_stripped_body`

**Severity:** Critical.
**Where:** §1.3's **E11**, §1.4's *"RAW: why 'with witness' needs E11"*, §2.2's RAW
row, §3's **V18**, §4 step 8.

**The failure, concretely.** E11 reads *"**RAW: re-serialising the decoded
transaction MUST reproduce the body BYTE FOR BYTE.**"* V18 is the vector written to
prove it fires: *"RAW body = the same transaction serialized WITHOUT witness,
carried txid correct … it deserialises, and its txid matches, and it must still be
REFUSED."*

It is not refused. Built as a scratch crate on the exact dependency §2.2 chooses:

```
with-witness body       : 222 B
  serialize(tx)==body   : true
  txid  (display)       : 2dcf2b973d52044b1e58c988a5a59d388073ff05598b0a1e93eeb04c72ebf630
  wtxid (display)       : d5717c031917116bbd4fcaff0bcc3abe9d456899991414f2177a5281ed836f51
witness-stripped body   : 113 B
  stripped deserialises : true
  txid unchanged        : true
  *** E11: serialize(deserialize(stripped)) == stripped : true ***
  (so E11 REFUSES V18?  : false)
```

**Why the plan permits it.** §1.4's justification quotes the encoder fact correctly
and draws the opposite conclusion from it: *"`bitcoin`'s `Transaction::consensus_encode`
… emits the segwit form **whenever any input carries a witness**, so E11 is one `==`
on two byte slices."* The witness-stripped body decodes to a transaction whose inputs
carry **no** witness, so `uses_segwit_serialization()` is false
(`bitcoin-0.32.9/src/blockdata/transaction.rs:1058-1065`) and `consensus_encode`
takes the legacy branch (`:1244-1246`) — reproducing the stripped body exactly.
**E11 is a canonicality check, not a witness-presence check.** It rejects a
non-canonical encoding of a transaction; it cannot see that a *different, canonical*
transaction was substituted.

§1.4's own MEASURED block contains four rows and is accurate in every one of them —
I reproduced all four. The row it does not contain is the one that decides the
claim: it measured `serialize(&tx) == body` for the **with-witness** body only, and
never measured re-serialisation equality on the **stripped** body. The load-bearing
sentence *"**E11 is the only check that fires**"* is therefore false; no check fires.

Every consequence §1.4 states remains live, unchanged from round 1's C2: *"Two
conforming records, different bytes, and one plate carries a transaction with every
signature removed — unbroadcastable, which is the one thing this artifact exists to
make possible."* Measured here at **109 bytes of signatures**, silently absent, with
`magic`, `version`, `form`, E1–E16, the deserialise and the txid equality all green.

**Confidence:** High. Executed, twice, from two independent serialisers; the
`bitcoin` branch that decides it read at source.

---

## [C2] No site in W1–W5 can carry *which rule* was violated, so §1.5's refusal line, E9's three messages, V13 and step 7 cannot be built

**Severity:** Critical.
**Where:** §2.4's five-site table, against §1.5's *"what the operator sees"* row,
E9, V13, and §4 step 7. This is the sixth site the fold did not name.

**The failure, concretely.** §1.5 rules that a refusal is *"**one line on stderr
naming the record's index and the rule**"*, E9 requires *"a bad `magic`, an unknown
`version`, or a `form` outside {0x01, 0x02} is REFUSED, **each with its own
message**"*, §1.5 restates that as *"three distinct stderr lines at exit 4, and V13
asserts the text, the stream and the code"*, and step 7's test is *"one stderr line
naming the index and **the rule**"*.

The channel that would carry a rule identity does not exist and no W-site creates it:

- `pub fn classify(record: &str) -> record::Class` (`sysw/mod.rs:124`) returns a
  **bare `Class`**. It has no `Result` and no payload; a `tx:` record that fails
  E1–E16 can only become `Class::Unknown`. W4 says *"a failure is §1.5's refusal"* —
  `classify` cannot express one.
- `split` (`mod.rs:255`) turns that into `SyswError::Unclassifiable(i, unknown_reason(&r))`,
  and `unknown_reason` (`mod.rs:108-115`) **re-derives the reason from the record
  string alone**, the classification result already discarded.
- `UnknownReason` (`mod.rs:96-105`) has exactly two variants — `NonHexBody(&'static str)`
  and `Unrecognised` — and `sysw_error` (`main.rs:1263-1280`) renders exactly two
  messages.

So W5 as specified — *"`unknown_reason` iterates `[PASS_PREFIX, TEXT_PREFIX]` only …
Add `TX_PREFIX`"* — yields **one** message for every `tx:` failure, and it is the
wrong one. A `tx:` record whose body is perfectly good lowercase hex but whose magic
is `MTX2` would be refused with `main.rs:1266-1271`:

> *"record {i} … begins `tx:`, but its body is **not lowercase hex**. That prefix is
> RESERVED … Encode the body first: `printf '%s' 'your text here' | xxd -p -c 256`"*

— a false statement about the record plus an instruction that would corrupt it.

**Why the plan permits it.** §2.4's header is *"**NORMATIVE — five sites, and none of
them is the codec**"*, and §6 makes W1–W5 a close condition. Producing the rule
requires changing at least one of `classify`'s signature, `UnknownReason`,
`SyswError` and `sysw_error`'s `Unclassifiable` arm — **none of which is W1–W5, and
none of which appears anywhere in the plan.** This is r1-C3's shape exactly: the
plan specifies a behaviour completely and never names the call that makes it
reachable. §2.4's own words apply to itself — *"**The branch is the work; the prefix
is not.**"*

(`UnknownReason`'s doc comment at `mod.rs:89-95` also constrains any fix: it *"Carries
NO operator data, and that is load-bearing"*, because a `pass:` body is a passphrase.
Rule names are compile-time constants, so this is satisfiable — but the plan does not
notice the constraint it is walking into.)

**Confidence:** High. All four functions read; the misleading message quoted from
source.

---

## [I1] §1.4's RAW row prescribes a body that cannot exist for a witness-free transaction, and no vector covers that class

**Severity:** Important.
**Where:** §1.4's `form` table, RAW row; §3's NORMATIVE one-transaction sentence.

**The failure, concretely.** §1.4 defines the RAW body as *"the serialized signed
transaction, WITH WITNESS — **the BIP-141 form: version, marker `0x00`, flag `0x01`,
inputs, outputs, each input's witness, locktime**"*. Enumerated like that, it is a
struct layout, and it is the second thing in this plan shaped like one.

For any transaction with no witnesses — every P2PKH/P2SH-only spend, an entirely
ordinary signed transaction to engrave — that layout is undecodable. Measured:

```
witness-free tx serialises to 113 B, first 6 bytes [02, 00, 00, 00, 01, 7c]
  -> does it carry marker 0x00 flag 0x01 at offset 4? false
  §1.4's literal BIP-141 form for a witness-free tx: REFUSED
     -- parse failed: witness flag set but no witnesses present
```

The refusal is structural, not incidental: `bitcoin-0.32.9/src/blockdata/transaction.rs:1280-1282`
returns `ParseFailed` when the segwit flag is set and every witness is empty, and
BIP-144 requires it.

So the plan holds two normative statements that disagree for this class: §1.4 says
the body carries marker+flag, E11 says the body is whatever the encoder emits — and
for a witness-free transaction the encoder emits the legacy form. A Rust implementer
following E11 accepts the 113-byte legacy body; a Go porter transcribing §1.4's
enumeration emits marker+flag and is refused, or accepts what Rust refuses. That is
the r1-C1 shape — a normative table and a normative rule saying different things —
in the half of the format the fold added.

**Why the plan permits it.** §3 rules *"**NORMATIVE: the transaction under every
vector below is the `even` vector**"*, which is segwit (`raw_hex` begins
`02000000 0001 01`, verified). **No vector anywhere in V1–V22 carries a witness-free
transaction**, so nothing tests the case, and §3's own reason for pinning segwit —
*"for a legacy transaction txid == wtxid and the vector passes in both worlds"* —
closes the txid axis while leaving the body-form axis untested for the same class.

**Confidence:** High. Both halves executed against `bitcoin 0.32.9`.

---

## [I2] §1.5's global "exit 4" contradicts the R0-GREEN spec's R7, `me`'s existing exit-2 path, and `me`'s own `EXIT_REFUSED = 3`

**Severity:** Important.
**Where:** §1.5's *"what the operator sees"* row and its scoping sentence; §6.2's
four-refusal table; §4 steps 2 and 11; §4.2.

**The failure, concretely.** §1.5 opens *"**"REFUSED" appears in E2–E16 and §2.3**"*
and closes *"Four answers, once, **binding every use of the word in this plan**"*,
and its answer is *"nothing on stdout, **exit 4**"*. §6.2 then puts **R2**, **R7**
and **the TTY refusal** under the word "refusal" alongside §2.3's, and step 2's test
column reads *"empty stdin refused (R7)"*.

Three collisions:

1. **Spec §5 rules R7 at exit 2, normatively.** `SPEC_engrave_transaction.md:1438`:
   *"R7 | **empty stdin** to `me sysw pack` | **must join the existing exit-2 path,
   not bypass it**"*, and `:125` shows it: `me: no records: pass them on argv or with
   --in     (exit 2, stdout empty)`. The path is real —
   `read_records` fails at `main.rs:1223-1225` and its caller returns **2** at
   `main.rs:903`. An implementer following §1.5 writes exit 4 and breaks an
   R0-GREEN spec rule and a shipped exit code.
2. **§4.2's TTY refusal lands on that same branch.** §4.2 says the new path
   *"**replaces a refusal**"* — precisely the `main.rs:1223-1225` one — so §1.5
   would have two sibling refusals at one site exiting 2 and 4.
3. **`me` already has a named vocabulary and the plan does not mention it.**
   `main.rs:225-228`:

   ```rust
   const EXIT_OK: i32 = 0;
   const EXIT_USAGE: i32 = 2;
   const EXIT_REFUSED: i32 = 3;
   const EXIT_INVALID: i32 = 4;
   ```

   Exit 3 is what `me` calls a **refusal** (`main.rs:323`, `:515`, and
   `bundle.rs:39`'s `RefusedSecret => 3`); exit 4 is *invalid/integrity*
   (`tests/cli.rs:162` comments it in those words). R2 — *"a `tx:` record on argv"*,
   refused on bearer-material policy — is the same shape as `main.rs:509-515`'s
   *"refusing to seal seed material … without `--seal-secret`"*, which returns
   `EXIT_REFUSED`.

**Why the plan permits it.** §1.5 was written to answer r1-I2 and it answers it with
**one** code for a word it then scopes to *every* use in the plan, citing
`main.rs:975-983` — which resolves exactly, is the pack-failure path, and does return
4. **Exit 4 is right for §2.3's decode-failure refusal** (it arrives through
`pack → split → Err`, and I traced that path). It is wrong for R7 against the spec,
unruled for the TTY case, and against `me`'s own convention for R2. §6.2 makes all
four one class and §1.5 gives that class one code.

**Confidence:** High for the R7 conflict (spec text and code path both read). High
for the constant vocabulary; Medium that R2 was intended as 3 rather than 4 — but the
plan states neither, which is the finding.

---

## [I3] V7 cannot be constructed under §3's own NORMATIVE one-transaction rule, so step 9 has no input and §6's provenance bullet over-claims

**Severity:** Important.
**Where:** §3's NORMATIVE sentence, **V7**, §4 step 9, §6's vectors-provenance bullet.

**The failure, concretely.** §3 rules: *"**NORMATIVE: the transaction under every
vector below is the `even` vector of `mt1_v1.json`**"* — **222 bytes**, verified.
V7 is *"body at **16,322 B minus the fields present**"*, and step 9 is *"V7 at
**16,322 − F**"*.

A 16,322-byte body cannot be built from a 222-byte transaction. Padding is not
available: §2.2 requires the RAW body to deserialise as a Bitcoin transaction, E11
requires re-serialisation equality, and the carried txid must match — so the body
must be a genuine ~16 KB transaction. The chunks form does not help either: the
corpus's six `mt1` strings are 87 characters each, a 527-byte body. The corpus holds
exactly two vectors (`even` 222 B / 6 chunks, `uneven` ~288 B / 8 chunks); neither is
within two orders of magnitude of the ceiling.

This is round 0's I7 — *"v1 said 'body at `MaxSectionLen` boundary'. **Unconstructible.**"*
— reappearing by a different route: the fold fixed the *number* and then added a
constraint that removes the *input*.

**Why the plan permits it.** The NORMATIVE sentence was written for r1-I7, whose
concern was the **txid axis** (*"V1 and V2 unconstrained … built from a convenient
small legacy transaction"*). Scoped to "every vector below" it also binds V7, where
the axis is size and the constraint is impossible. §6's bullet then leans on the same
sentence: *"The vectors were not produced by the code they judge — **satisfied by
construction, since §3's transaction comes from a corpus generated by
`scripts/gen-mt1-vectors.py`**"*. Whatever V7's 16 KB transaction turns out to be, it
cannot come from that corpus, so the "by construction" satisfaction does not cover
it and the plan names no other independent source.

**Confidence:** High. Corpus parsed; both vectors' sizes and chunk strings read.

---

## [I4] §6's `Class::Transaction` closure grep is a false PASS: it counts files, two sites contain no such token, and at W3 it asserts the opposite of what W3 requires

**Severity:** Important.
**Where:** §6's `ClassTransaction` bullet, against §2.4's W1, W3 and W5.

**The failure, concretely.** §6 reads: *"`grep -c 'Class::Transaction'
crates/me-cli/src/sysw/` is non-zero at **all five** sites W1–W5"*. Executed on the
current tree, that command returns **per-file** counts over nine files — it has no
notion of a site, and W1–W3 all live in `record.rs` while W4–W5 both live in
`mod.rs`. Two files, not five sites.

Worse, three of the five sites are not `Class::Transaction` sites at all:

- **W1** is *"add `pub const TX_PREFIX: &str = "tx:";`"* — contains no
  `Class::Transaction`.
- **W5** is *"`unknown_reason` … Add `TX_PREFIX`"* — contains no `Class::Transaction`.
- **W3** is *"**`Class::Transaction` is NOT in `is_secret`'s `matches!`**"* — it
  requires the token to be **ABSENT**. A non-zero count there is the defect, not the
  proof. And an absence is not assertable by any non-zero grep.

So the condition is satisfied by W2 + W4 alone — the enum variant and the classifier
branch — while W5's error-message fix goes undone and W3's secrecy ruling goes
unchecked. **W3 is the one §2.4 itself calls out as the site that costs the operator
if it is wrong**: *"Get it wrong and **every transaction payload seals**: a 12-word
passphrase to store, those 12 words typed on the device's on-screen keyboard, and
~31 s of KDF."*

**Why the plan permits it.** The bullet was written to close r1-C3 by making the
wiring machine-checkable, and it reaches for a grep before checking what the grep can
see. The plan already knows this shape and names it twice — *"a close condition that
could pass on the defect it exists to catch"* (§2.4) and *"a gate that is red for
non-defects trains a reader to ignore it"* (§6.1). `Class::Transaction::is_secret()`
is **false** is separately asserted in the same bullet and in step 6, which does cover
W3's substance; the grep half is what fails.

**Confidence:** High. Command executed; all five sites read.

---

## [I5] §6's and step 6's `me sysw show` read-back names a capability `show` does not have

**Severity:** Important.
**Where:** §6's `ClassTransaction` bullet, §4 step 6.

**The failure, concretely.** §6 requires *"**`me sysw pack` followed by `me sysw
show` round-trips a real `tx:` record** — the end-to-end assertion no record-level
vector can substitute for"*, and step 6's test column requires *"`me sysw show` reads
it back"*.

`me sysw show` (`main.rs:1045-1088`) prints, in order: `sealed:`, `pub_len:`,
`ct_len:`, `identity:`, the digest (stderr), and then `print_mdmk_confirmation`.
**It never prints a record.** `print_mdmk_confirmation` (`main.rs:1156-1181`) is the
only per-record output there is, and its second statement is:

```rust
if sysw::classify(r) != sysw::record::Class::MdMk {
    continue;
}
```

so a `tx:` record produces no line at all, before or after W1–W5. There is no
`--records` flag and no other subcommand that lists them.

Two ways out, and the plan takes neither: change `show` — a **sixth** site,
alongside the message channel in **[C2]**, not named in §2.4's *"NORMATIVE — five
sites"* — or weaken the assertion to `pub_len`/digest, which is not a round-trip of
the record and is exactly the substitutability §2.4 says no vector may have.

**Why the plan permits it.** The bullet is the fold's answer to r1-C3, and it reaches
for an end-to-end command without checking what that command emits. The *pack* half
does carry real weight — `split` refuses `Class::Unknown` outright
(`mod.rs:255`), so `me sysw pack` merely *succeeding* on a `tx:` record proves W4
landed — so the C3 fix is not wholly undone. The stated gate is.

**Confidence:** High. `show`'s whole arm read; `print_mdmk_confirmation` read; the
existing `show` tests in `tests/sysw_cli.rs` confirm it prints header fields and
md1/mk1 confirmation lines only.

---

## [I6] No step adds `mt-codec` to `crates/me-cli/Cargo.toml`, so step 4 cannot build V3 — r1-I5's defect, reproduced for the other dependency

**Severity:** Important.
**Where:** §4 step 4, §5, §6's publish bullet.

**The failure, concretely.** §2.2 requires the CHUNKS path to *"**reassemble via
`mt-codec`, THEN deserialise the result**"*. §5's entire ceremony exists to make that
possible: *"publish `mt-codec` 0.1.0 to crates.io, then depend on the pinned
published version"*, and §6 closes on *"`me` depends on the **pinned published
version**"*.

`grep -n 'Cargo.toml' ` over the plan returns **one** manifest edit, at step 4:
*"this is the step that adds `bitcoin = { version = "0.32", default-features =
false, features = ["std"] }` to `crates/me-cli/Cargo.toml`"*. **`mt-codec` is added
in no step**, and the version string to pin is never written anywhere (§5 names
`0.1.0` as what is published, not as what `me` declares).

Step 4's own test is *"V1–V3 round-trip"*, and **V3 is the CHUNKS vector** — so the
first step that needs `mt-codec` is the step that does not add it. Verified `me-cli`
declares no `mt-codec` today: its dependencies are `md-codec`, `mk-codec`,
`ms-codec`, `clap`, `zeroize`, `serde`, `serde_json`, `aes-gcm`, `pbkdf2`, `sha2`,
`bip39`, `rand`, `rpassword`.

**Why the plan permits it.** This is r1-I5 verbatim, one dependency over. Round 1
graded that finding PARTIAL for exactly this reason — *"and **no step in §4 adds
it**"* — and the fold added an *"**added in** | **§4, step 4**"* row to the `bitcoin`
decision table and built no equivalent for `mt-codec`, whose publish is the
**irreversible** action §5 gates on the plan being GREEN.

**Confidence:** High. Plan grepped; `me-cli/Cargo.toml` read.

---

## [I7] E13 and E16 are half-vectored, so §6's "RED without its check" passes with half of each check deleted

**Severity:** Important.
**Where:** §1.3's E13 and E16, §3's V17 and V20, §6's rules bullet.

**The failure, concretely.** §6 requires *"**Every rule E1–E16 has a test that goes
RED without its check**"*, and §6.2 makes that a mutation exercise — *"verified by
deleting the check by hand and watching the test fail for the stated reason"*. Two
rules state two independent conditions and are vectored for one each:

- **E13** — *"CHUNKS: every element is lowercase ASCII **with no leading or
  trailing whitespace**."* Its only vector is **V20**, *"CHUNKS body with an
  UPPERCASE `mt1` string"*. Delete the whitespace half and every vector stays green.
  The whitespace half is not hypothetical: E13's own justification quotes
  `mt-codec`'s tolerance as `s.trim().to_ascii_lowercase()`
  (`pipeline.rs:66`, verified byte-exact) — **naming both halves** — and a padded
  element changes `body_len`, the record hex and the EPD §6.6 public-data hash
  exactly as an uppercase one does.
- **E16** — *"A fixed-width tag MUST carry exactly its width: **`0x02` → 8, `0x03`
  → 4**."* Its only vector is **V17**, *"`tag=0x02, len=2`; and the near-misses
  `len=7` and `len=9`"*. **Tag `0x03` is never vectored at any width.** Delete the
  `0x03 → 4` half and every vector stays green, and a 3-byte value for the master
  fingerprint reaches the plate's `FROM` line.

**Why the plan permits it.** The mutation discipline §6.2 mandates is per-*rule*, and
an implementer deleting "E16's check" deletes the fixed-width guard wholesale and
watches V17 go red — so the exercise reports success while the half-gap survives.
This is r1-I11's own argument one level down: *"E6 refuses only `len = 0`; `1..7` is
the **actual gap**"* — the same reasoning applied to E16 gives tag `0x03`, and applied
to E13 gives whitespace.

**Confidence:** High. Every V-row mapped against every E-rule; E1–E12, E14, E15 each
have a negative that isolates them (E7 excepted — **[M1]**).

---

## [M1] E7 has no removable check and no negative vector of its own

E7 — *"An absent optional field is OMITTED from the list. There is no empty encoding
and no sentinel."* Its listed vector, **V5**, is *positive* (*"absent optional
field"*). The empty-encoding half is enforced by E6/V12; the "no sentinel" half is
semantic and no decoder check can reach it. So §6's *"Every rule E1–E16 has a test
that goes RED without its check"* has nothing to delete for E7. Filed Minor rather
than with **[I7]** because E7 is an encoder rule and V12 discharges its only
checkable clause — but the completeness claim is stated over all sixteen.

## [M2] §6.2 puts `refusals.toml` in the wrong directory of the right repo

*"`refusals.toml`, `check-refusal-coverage.sh` and `mutate-refusals.sh` are **`mt`'s
and live in `mnemonic-transaction/scripts/`**"*. Two of the three do. `refusals.toml`
is at `mnemonic-transaction/crates/mt-cli/tests/refusals.toml`. The correction's
verdict is right (see the I4 section above); its path is not, in the one sentence
whose whole job was to correct a path.

## [M3] §6.1's build-gate evidence no longer reproduces, and the fold's remedy paragraph does not say so

§6.1 quotes round 1's run — `test result: ok. 77 passed; 0 failed … EXIT=0` — and
argues from it. That output is now unreproducible: `c8f8557` made the script refuse
on an empty extraction, and today the same command prints *"Refusing rather than
reporting a pass on an empty extraction"* at **EXIT=3**. The quote is correctly
attributed to round 1 and the conclusion (*"NOT a close condition for P1"*) is
unaffected, so this is not a false claim — but §6.1's *"what would have to change for
it to apply"* paragraph describes only the anchor filter and never mentions that the
structural false PASS it is arguing against has already been closed. A reader running
the command to check the section's premise gets a different answer than the section
shows.

## [M4] V15's construction is unstated in the one way that decides whether it can go RED

V15 is *"a chunks record whose carried txid's top 20 bits ≠ its chunks'
`chunk_set_id`"*, and the plan asserts *"delete R15's comparison and every **other**
vector stays green"*. That holds only if the perturbation is applied to the **chunks'
embedded `set_id`**, leaving the carried txid honest. Perturb the **txid** instead —
which is what the row's wording names first — and §2.2's full txid-equality check
refuses the record on its own, so V15 stays green with R15 deleted and R15 has no RED
test. Minor rather than Important because §6.2's mutation step surfaces it
immediately (unlike r1-I1's V2, which had no mutation step attached), but the row
should not leave the choice to the builder.

---

## Verdict

**2 Critical / 7 Important / 4 Minor. NOT GREEN. No code.**

Part 1: **20 FIXED / 2 PARTIAL / 0 NOT FIXED** of round 1's 22, and **5 of 5** of
round 0's carry-overs closed. The I4 push-back was **correct**.

The fold is a real improvement and its measured content is trustworthy: every
arithmetic claim in §3.1, every gate value in §6.1, every corpus fact in §3 and every
`bitcoin`-API fact in §2.2 reproduced **exactly** when I re-ran them. That is 45/53
citations, 84 table rows, three gate outputs, three spec corrections and eight
measured numbers, all independently confirmed.

What fails is the same axis round 1 named — *"markedly stronger on rules, markedly
weaker on gates"* — plus one new axis: **the fold's two headline fixes were reasoned
rather than executed.**

- **[C1]** E11, the rule invented to close r1-C2, does not fire on V18, the vector
  invented to prove it fires. The plan's four-row measurement block is accurate and
  omits the one row that would have caught this. Four minutes of the scratch crate
  §6.1 already describes would have found it.
- **[C2]** §2.4's five sites are correct and insufficient. The refusal *message* —
  §1.5's rule name, E9's three distinct lines, V13's asserted text — has no channel,
  and W5 as written produces an actively false message for the commonest `tx:`
  failure. r1-C3's shape, one layer in.
- Five of the seven Importants are gates again: **[I3]** V7 has no constructible
  input, **[I4]** the wiring grep passes on three sites it cannot see, **[I5]** the
  end-to-end read-back names a command capability that does not exist, **[I6]** the
  dependency the §5 publish ceremony exists to enable is added by no step, **[I7]**
  two rules are half-vectored.
- **[I1]** and **[I2]** are the two places the plan contradicts something outside
  itself: BIP-144 for a witness-free transaction, and spec §5's R7 exit code.

### What I did NOT examine

- P2–P6, the device, the plate, `design/agent-reports/`, style, wording, length.
- Whether `mt-codec` publishes cleanly (`cargo publish --dry-run` still not run) —
  the plan names this as a precondition, not a close condition, and I left it there.
- `mt-codec`'s reassembly API surface: I confirmed `me` needs it and that no step
  declares it, not that any particular function signature is right.
- The BCH/bech32 correctness of `mt-codec` and `mt1_v1.json` as a corpus — treated
  as ground truth, per rounds 0 and 1. I did verify the `even` vector's txid, wtxid,
  set_id, byte count, segwit marker and six chunk strings against an independent
  Python serialiser.
- Non-minimal `VarInt` and 0-input transactions as E11 edge cases: E11 is
  *checkable* for these (the decoder settles them), and the plan's defect is what
  E11 checks, not whether it can be run.
- `me`'s existing CLI suite, and whether steps 2 or 3 break it. I did confirm clap
  does **not** echo an argv record on two malformed `me sysw pack` invocations, so
  step 11's R2 assertion is implementable as written.
- Whether spec §2.3's three corrections, once applied, leave anything else in the
  spec stale — I verified the three numbers, not their neighbourhood.
