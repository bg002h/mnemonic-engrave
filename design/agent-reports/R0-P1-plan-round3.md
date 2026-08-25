# R0 — `IMPLEMENTATION_PLAN_P1_me_container.md`, round 3

**Reviewer:** independent R0 agent, round 3. **Artifact:** v4, 1,496 lines
(`186a2f8`, +883/−214 across the plan **and** `design/SPEC_engrave_transaction.md`,
folded by the architect). **Prior rounds:** round 0 (5C/13I/5M on v1), round 1
(3C/11I/8M on v2), round 2 (2C/7I/4M on v3, 20 FIXED / 2 PARTIAL / 0 NOT FIXED).

**Machine-checked before writing.** Nothing below rests on reading a doc comment,
a table cell, or an earlier report.

- **All three cited gates re-run on this document, and all three reproduce the
  plan's stated values EXACTLY.** `plan-cite-check.sh` → **73 / 90 resolved, 17
  dangling, 0 ambiguous, exit 0**, and the 17 are exactly **9 `mnemonic-transaction`**
  (`pipeline.rs` ×6 — `:17-27`, `:54`, `:66`, `:93`, `:148`, `:160`; `header.rs`
  ×2 — `:4`, `:26`; `lib.rs` ×1) **+ 8 `bitcoin`**. `plan-table-check.sh` →
  **120 rows, 0 malformed, exit 0**. `plan-fold-sweep.sh --terms <the twelve>` →
  **exactly 12 hits, one per term, all inside the self-reference block at lines
  1300–1312**.
- **§1.1a's whole measurement block re-derived from scratch**, with an independent
  Python segwit serialiser over the corpus `even` vector — not with the plan's
  crate. Every row reproduces: 222 B / 113 B / `txid 2dcf2b97…f630` / `wtxid
  d5717c03…6f51` / `txid≠wtxid` / stripped deserialises / **`serialize(deserialize(stripped))
  == stripped` → TRUE (E11 does not fire)** / txid unchanged / **stripped wtxid ==
  its own txid → TRUE** / **stripped wtxid ≠ carried wtxid → E17 FIRES**.
  `top-20 display 0x2dcf2 == set_id`; `top-20 internal 0x30f6e`.
- **V18 and V26 constructed as records.** Both 188 bytes, bodies identical
  (113 B), **differing in exactly 32 positions — the `wtxid` field.** The claim
  holds by construction.
- **Every ceiling and every row of §3.1 and §1.4a recomputed** (see *The
  arithmetic*). Framing 75, metadata record 153, body ceiling 16,290, 5/2 raw
  8,313 (122 over 8191), 10/2 chunks-in-container 18,737 (13,997 spare), v3's
  37,255 / 4,521-over — **all exact, none transcribed.**
- **W1–W10 resolved line by line** against the real tree, and the production chain
  behind them read end to end: `classify` (`sysw/mod.rs:124-147`), `split`
  (`:251-266`), `unknown_reason` (`:107-115`), `UnknownReason` (`:96-105`),
  `SyswError` (`:58-87`), `sysw_error` (`main.rs:1255-1301`),
  `print_mdmk_confirmation` (`main.rs:1156-1181`), `Show` (`main.rs:1045-1088`),
  `read_records` (`:1211-1227`), the exit constants (`:225-228`), the pack arm
  (`:890-985`).
- **Three behaviours EXECUTED on the shipped `target/debug/me`**: a bare corpus
  `mt1` chunk is refused at **exit 4** with `Unrecognised` (§1.4a's cost 1, exact);
  a `tx:` record with a valid hex body is refused with the **same** message; and
  **an `md1` record with a trailing space PACKS at exit 0 and lands in the public
  section verbatim, space included** — which falsifies E13's precedent claim.
- `bitcoin-0.32.9` read at source for `NonMinimalVarInt` (`consensus/encode.rs:59,80,505,513`).
- `mt-codec` read at source: `ChunkHeader{version,chunk_set_id,count,index}`
  (`header.rs:28-38`), `DecodedChunk::corrected` (`pipeline.rs:93`),
  `DecodedSet::bytes` (`pipeline.rs:150`), `decode_chunk` (`:160`), `decode`
  (`:234`), `to_symbols`'s `s.trim().to_ascii_lowercase()` (`:66`).

---

## Part 1 — did round 2's findings land?

### Round 2's two Criticals

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r2-C1** E11 does not fire on V18 | **FIXED** | The `wtxid` field lands at offset 38 and **E17** is the rule: *"The decoded transaction's WTXID (display order, §1.1a) MUST equal the carried `wtxid` field — on BOTH forms."* I rebuilt the measurement from an independent serialiser: on the 113-byte stripped body the computed wtxid equals its own txid and differs from the carried wtxid in all 32 bytes, so **E17 fires where E11 returned `true`**. E11 is demoted to a named exception with an owner (P3) and §1.3 records that it has **no RED test in Rust** — supported: `bitcoin` has `NonMinimalVarInt` and *"data not consumed entirely"*. |
| **r2-C2** no site can carry WHICH rule was violated | **PARTIAL** | The **carrier** landed (W7: `UnknownReason::TxRule(&'static str)`, and the `&'static str` argument against `mod.rs:89-95`'s no-operator-data invariant is correct) and the **renderer** landed (W8). **The producer did not.** `unknown_reason` is the sole producer in the production path — `split` at `mod.rs:255` is its only caller and passes the record string alone — and **W6's prescribed change is to add `TX_PREFIX` to the `NonHexBody` prefix loop**, which returns `NonHexBody("tx:")` for a hex-valid `MTX2` record and therefore makes §6's own W8 assertion RED. See **[C1]**. A bare `mt1` record has no channel at all — see **[C2]**. |

### Round 2's seven Importants

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r2-I1** RAW row prescribes an undecodable body | **FIXED** | §1.4's RAW row now reads *"the transaction's CANONICAL CONSENSUS SERIALIZATION — the BIP-144 segwit form … **when any input carries a witness**, and the legacy form … **when none does**"*, with the `bitcoin` branch cited at `transaction.rs:1057-1065` / `:1244-1246`, and **V26 is the witness-free vector v3 had none of**. |
| **r2-I2** one exit code for every use of "REFUSED" | **FIXED** | §1.5 replaces it with a four-row NORMATIVE table: E1–E20 + §2.3 → **4**, R7 → **2**, the TTY refusal → **2**, R2 → **3**. Verified against `main.rs:225-228` (`EXIT_USAGE=2`, `EXIT_REFUSED=3`, `EXIT_INVALID=4`), the exit-2 path at `:903`, the pack-failure `return 4` at `:983`, and `EXIT_REFUSED` at `:515`. Steps 2, 7 and 11 each assert their own code. |
| **r2-I3** V7 unconstructible under §3's NORMATIVE rule | **FIXED for V7** | §3 gains a NORMATIVE exception — *"V7's transaction is a SECOND, SIZE-ONLY vector … generated by `scripts/gen-tx-record-vectors.py` — new in this repo, committed in §4 step 9"* — and §6's provenance bullet is re-scoped to *"V1–V6 and V8–V27 … and for V7 by `gen-tx-record-vectors.py`"*. **The same defect is reproduced for the new V27 — see [I3].** |
| **r2-I4** the `Class::Transaction` grep is a false PASS | **FIXED** | §6: *"**(r2-I4) THE `grep -c` THAT USED TO CLOSE THIS IS STRUCK — it was a false PASS**"*, replaced by a ten-row table of per-site tests, including `assert!(!Class::Transaction.is_secret())` for the absence W3 requires. |
| **r2-I5** `me sysw show` names a capability it lacks | **FIXED** | **W9** — *"`print_mdmk_confirmation` gains a `tx:` / `mt1` arm … It prints one line per `tx:` record (form and carried txid) and **one line per chunk SET, not per chunk**"*. §6 keeps the gate and builds the capability. (Residual: the wtxid is printed nowhere — **[I4]**.) |
| **r2-I6** no step adds `mt-codec` | **FIXED** | Step 4: *"this is the step that adds **BOTH** manifest lines to `crates/me-cli/Cargo.toml`"*, with §2.2's second decision table naming crate, version and first load-bearing step. (The version string is a caret, not a pin — **[M1]**.) |
| **r2-I7** E13 and E16 half-vectored | **FIXED** | E13's cell: *"**BOTH halves are vectored (r2-I7)**: V20 the case, V23 the whitespace"*; E16's: *"V17 the fee tag, **V17b** the fingerprint tag"*, and V17b is *"`tag=0x03, len=3` and `len=5` REFUSED; `len=4` passes"*. (E13's supporting **precedent** claim is false — **[I2]**.) |

### Round 2's four Minors

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r2-M1** E7 has no removable check | **FIXED** | §1.3's two-row exception table: *"**E7** … it is an **encoder** rule … | nobody — it constrains what P1 EMITS, and V5 is its positive"*, with *"Neither exception is a licence."* |
| **r2-M2** `refusals.toml` in the wrong directory | **FIXED** | §6.2: *"**`refusals.toml` is `mt`'s too but lives at `mnemonic-transaction/crates/mt-cli/tests/refusals.toml`**"*. |
| **r2-M3** build-gate evidence no longer reproduces | **FIXED** | §6.1's blockquote: *"`c8f8557` … made the script refuse on an empty extraction. **Re-run while writing this fold, the same command now prints … at `EXIT=3`**"*. |
| **r2-M4** V15's construction unstated | **FIXED** | V15 is now NORMATIVE: *"the perturbation is applied to the CHUNKS' EMBEDDED `set_id`, leaving the carried txid HONEST"*, with the reason spelled out. |

### Round 1's two PARTIALs

| finding | verdict | the sentence that settles it |
| --- | --- | --- |
| **r1-C2** body definition + enforcement | **FIXED** | The definition is §1.4's two-row table; the enforcement moved from E11 (measured not to fire) to **E17**, and I confirmed E17 fires on the exact input. |
| **r1-C3** nothing creates `ClassTransaction` | **PARTIAL** | Ten sites, all ten resolving, each with a per-site test. But the refusal *message* still has no producer for `tx:` (**[C1]**) and none at all for the `mt1` and payload-level classes the ruling creates (**[C2]**). |

**Tally: 12 FIXED / 1 PARTIAL / 0 NOT FIXED** of round 2's 13, plus **1 FIXED /
1 PARTIAL** of round 1's two carry-overs. The one PARTIAL, r2-C2, is re-raised
below as **[C1]** and **[C2]**.

---

## Part 2 — defects in v4

## [C1] `UnknownReason::TxRule` has NO PRODUCER, and W6 as specified makes §6's own W8 assertion RED

**Severity:** Critical.
**Where:** §2.4's **W6** and **W7** against §6's W6 and W8 closure rows; §1.5's
*"what the operator sees"*; E9; V13; §4 step 6's last clause.

**The failure, concretely.** The fold's answer to r2-C2 is three sites: W6 adds
`TX_PREFIX` to `unknown_reason`, W7 adds the `TxRule(&'static str)` variant, W8
adds the rendering arm. **Nothing constructs a `TxRule` value.** Read at source:

- `unknown_reason` (`crates/me-cli/src/sysw/mod.rs:107-115`) is the **only**
  producer of `UnknownReason` on the production path. `grep -rn "UnknownReason::"
  crates/me-cli/src/` returns five hits: `:111` and `:114` inside
  `unknown_reason`, and `:554`, `:572`, `:588`, which are **tests**.
- Its only caller is `split` at `crates/me-cli/src/sysw/mod.rs:255`, which passes
  `&r` — **the record string, the classification already discarded.**
- W6's stated change is *"`unknown_reason` iterates `[PASS_PREFIX, TEXT_PREFIX]`
  only … **Add `TX_PREFIX`**"*, i.e. add it to the loop that returns
  `NonHexBody(prefix)`.

Implement W1–W10 exactly as written and feed `me sysw pack` a `tx:` record whose
body is perfect lowercase hex but whose magic is `MTX2`: `classify` refuses it,
`split` calls `unknown_reason`, the string starts with `tx:`, and the operator is
told (`main.rs:1266-1271`):

> *"record 0 … begins `tx:`, but its body is **not lowercase hex**. That prefix is
> RESERVED … Encode the body first: `printf '%s' 'your text here' | xxd -p -c 256`"*

**§6's W8 row requires the exact opposite**: *"a `tx:` record with magic `MTX2`
produces a line naming the **magic** rule, and **the string "not lowercase hex"
does NOT appear in it**"*. §6's W6 row requires *"a `tx:` record with a non-hex
body reports `NonHexBody("tx:")`, not `Unrecognised`"*. Both are satisfied by one
function that sees only the string, and it cannot tell the two cases apart
without re-running the codec — **which the plan nowhere says it does.** W7's own
closure test is an existence test (*"`UnknownReason::TxRule("magic")` exists and
is `Copy`"*), so nothing in §6 catches the missing producer except W8, which is
RED.

Confirmed on the current tree: `me sysw pack --no-passphrase 'tx:4d545831'`
already exits 4 today with the `Unrecognised` text, and W6's addition converts
that to the `not lowercase hex` text for **every** `tx:` failure, hex or not.

**Why the plan permits it.** Round 2 named four things a fix must touch —
*"`classify`'s signature, `UnknownReason`, `SyswError` and `sysw_error`'s
`Unclassifiable` arm"*. The fold changed `UnknownReason` (W7) and `sysw_error`
(W8), touched `unknown_reason` (W6) only to add a prefix, and **left `classify`'s
signature and `split`'s call exactly as round 2 found them** — so the rule
identity is still discovered in a function that cannot return it and rebuilt in a
function that cannot know it. §2.4's own sentence applies to itself a third time:
*"The branch is the work; the prefix is not."*

**Confidence:** High. All five `UnknownReason::` sites read; the sole caller read;
the misleading message quoted from source; the present-day refusal executed.

---

## [C2] E13, E19, E20 and R17 have NO refusal channel — no `UnknownReason` variant applies, no `SyswError` variant exists, and §1.5's "index and the rule" is unsatisfiable for a set

**Severity:** Critical.
**Where:** §1.3's **E13**, **E19**, **E20**; §2.4's **W5**, **W7**, **W8**,
**W10**; §1.5's NORMATIVE *"what the operator sees"* row and its scoping sentence
*"this section binds E1–E20 and §2.3"*; §3's V20, V23, V24, V25, V27; §4 steps 8
and 10.

**The failure, concretely — two halves, both measured.**

**(a) A bare `mt1` record that fails E13 or E19 gets a message that is false.**
W5 puts E13 (case, whitespace) and E19 (pristine, `corrected == 0`) inside
`classify`, which returns a bare `Class`. A failure becomes `Class::Unknown`,
`split` calls `unknown_reason`, and a bare `mt1` string matches **no** reserved
prefix — not `pass:`, not `text:`, and not W6's new `tx:` — so it returns
`Unrecognised`. Executed on the shipped binary, on a corpus chunk:

```
$ me sysw pack --no-passphrase 'mt1p9h8jqq9qqqqgqq…2229sax'
me: record 0 (records count from 0) is not a form this container can place: not a
    BIP-39 mnemonic, not an md1/mk1/ms1 string, and not a `text:`/`pass:` record.
    Descriptors and addresses are not yet classifiable here — see sysw::classify   (exit 4)
```

That is the message **V20 (uppercase), V23 (padded) and V24 (BCH-repaired) will
each produce after W1–W10**, and it is false in all three: the record *is* an
`mt1` string, and the operator is sent to look for a format problem that does not
exist. W7's variant is named `TxRule` and W8's text is *"is a **`tx:` record**
that fails rule {rule}"* — neither fits a bare chunk. **This is r2-C2's exact
shape, reproduced for the record class §1.4a's ruling creates**, and the fold's
own §1.4a names that class as *"the work"*.

**(b) E20 and R17 have no error VALUE at all, and §1.5 cannot describe them.**
W10 is *"`split` gains a payload-level pass for E20 — set membership,
completeness, orphans"*. `split` returns `Result<…, SyswError>`
(`crates/me-cli/src/sysw/mod.rs:251`). Its variants are
`Wire | Unclassifiable(usize, UnknownReason) | TooLarge | Crypto |
PassphraseMismatch | EmptyPassphrase | NotEnterableOnDevice | PassphraseTooLong |
NotUtf8` (`:58-87`). **None of them can express "a set is missing index 3 of 6"
or "two chunk sets collide on their top 20 bits":**

- `Unclassifiable(i, …)` is per-record and requires an offending index. For a
  **missing** index there is no offending record — the failure is a property of
  the set. For **R17** the failure is a property of two `tx:` records that are
  each individually valid.
- `UnknownReason` carries `&'static str` only, so it cannot name a set, a count,
  an index, or — as spec §5's R17 NORMATIVELY requires — **"both txids"** in full.
- **No W-site adds a `SyswError` variant** (`mod.rs:58-87` appears in no row of
  §2.4's ten), and **no W-site adds an arm to `sysw_error`'s outer match**
  (`main.rs:1257-1300`) — W8 adds an arm to the **inner** `UnknownReason` match at
  `:1265-1279` only.

And §1.5 is NORMATIVE that every one of E1–E20 produces *"**one line on stderr
naming the record's INDEX and the RULE**"*. For E20's missing-index case and for
R17 there is no single record index, so the section binds a shape its own rules
cannot take.

**Why the plan permits it.** §2.4 was rewritten to close r2-C2 and it enumerates
sites for the **`tx:` per-record** message only; the two new record classes it
created in the same fold — a bare `mt1` chunk, and a *set* — inherited the same
gap one layer further out. §6.2's *"REFUSAL COVERAGE IS EVERY REFUSAL P1 ADDS"*
table lists exactly four refusals (R2, R7, the TTY refusal, §2.3's decode
failure) and **E20 and R17 are in neither that table nor the E1–E20 completeness
claim's message contract.** §6's W10 row asserts only *"V25's three negatives all
refuse; V3's complete payload packs"* — refusal, not what the operator is told,
which is the half §1.5 exists to rule.

**Confidence:** High. `SyswError`'s nine variants read; `sysw_error`'s outer match
read; `unknown_reason`'s loop read; the false message reproduced by execution on
the current tree.

---

## [I1] E12's cited assertion is `seal`'s test on `seal`'s joiner, so E12 has no RED test in `sysw` and §6's completeness claim over E1–E20 is false

**Severity:** Important.
**Where:** §1.3's **E12**; §3.1's (M7) paragraph; §3's **V19**; §6's *"Every rule
E1–E20 has a test that goes RED without its check — EXCEPT E7 and E11"*.

**The failure, concretely.** E12 is RE-HOMED by §1.4a on the grounds that the
container already enforces and asserts it: *"`me` already has it —
`payload.public.join("\n")` at `crates/me-cli/src/sysw/mod.rs:260`, **asserted by
`joins_with_lf_and_no_trailing_lf`**. **The fact did not leave the plan; it moved
to the layer that owns it**"*. §3.1 repeats the pairing, and V19's cell retires
v3's trailing-`\n` vector into it: *"that case moved to the container and is
covered by `joins_with_lf_and_no_trailing_lf`"*.

That test is not `sysw`'s. Located:

```
crates/me-cli/src/seal/container.rs:85:    fn joins_with_lf_and_no_trailing_lf() {
```

It calls `encode_section` (`seal/container.rs`, whose `MAX_RECORDS = 24` and
`MAX_RECORD_LEN = 512` the plan itself insists are **`seal`'s and not `sysw`'s**,
§3.1) and asserts on **`seal`'s** joined section. `sysw`'s four joins —
`mod.rs:192`, `:260`, `:278`, `pubhash.rs:27` — are covered by it nowhere; `grep
-rn 'joins_with_lf_and_no_trailing_lf' crates/` returns that single hit.

So: change `sysw/mod.rs:260` to join with anything else and
`joins_with_lf_and_no_trailing_lf` stays green. **E12 has no negative test in the
container it governs**, it is not in §1.3's two-row exception table, and §6's
completeness sentence is therefore false over a third rule — the precise failure
§1.3 says is *"worse than a narrower one that is true"*.

The plan already knows this shape and states it two sections later, about the
sibling constant: §4.1 — *"**Step 1's test asserts BOTH** … A test that only
checks the raise would pass if someone edited the frozen container instead."*
Applied to E12, the fold cited the frozen container's test to discharge the live
container's rule.

**Why the plan permits it.** §1.4a moved E12 from the record layer to the
container layer and reached for the nearest test with the right *name*. The
function name is unambiguous and the file it lives in was not checked.

**Confidence:** High. The test read in full; its subject (`encode_section`) read;
every `sysw` join site enumerated by grep.

---

## [I2] E13's precedent is false for the whitespace half — MEASURED: `me sysw pack` accepts a padded `md1` record today and packs it verbatim

**Severity:** Important.
**Where:** §1.3's **E13** justification; §3's **V23**; §6's near-miss list.

**The failure, concretely.** E13 requires *"no leading or trailing whitespace"*
and justifies it as parity: *"**`me` already refuses exactly this for md1/mk1** —
`first_noncanonical` plus the uppercase scan at
`crates/me-cli/src/seal/record.rs:118-128` — and `mt1` joins them."*

It does not. `validate_record` **trims first**, and the cited range opens with the
trim:

```
crates/me-cli/src/seal/record.rs:118    let s = s.trim();
crates/me-cli/src/seal/record.rs:119    if let Some((pos, ch)) = first_noncanonical(s) {
```

`first_noncanonical` (`crates/me-cli/src/validate.rs:63-66`) finds
`c.is_whitespace() || c == '-'` — over the **already-trimmed** string, which the
sibling comment states as the design (`validate.rs:79-81`: *"has already
`str::trim`ed `s`, so any remaining whitespace is **interior**"*). So `me`
refuses **interior** whitespace and uppercase, and **tolerates** leading and
trailing whitespace. Executed:

```
$ printf 'md1fv9wjpq…esk2tl3 \nmd1fv9wjpq…lawq374\n' > in.txt   # note the space
$ me sysw pack --in in.txt --no-passphrase --out p.bin ;  echo $?
0
$ python3 -c "print(open('p.bin','rb').read()[52:])"
b'md1fv9wjpq…esk2tl3 \nmd1fv9wjpq…lawq374'          <- the space is IN THE SECTION
```

`split` pushes the record **as given** (`sysw/mod.rs:257`), so the padded string
reaches the public section byte for byte and changes the EPD §6.6 public-data
hash — *exactly* the harm E13's own cell describes, already live for `md1`/`mk1`.

Two consequences the plan does not carry: (1) the precedent it cites for E13 is
not there, so E13's whitespace half is a **new and stricter** posture, not a
joining; and (2) after P1 the container refuses a padded `mt1` record and accepts
a padded `md1` one, and nothing in the plan rules on that asymmetry or files it.
The `seal` container **does** normalise (`seal/container.rs:74` joins `trimmed`,
asserted by `surrounding_whitespace_does_not_change_the_encoding` at `:94`) —
which is the third time in this fold that a `seal` behaviour has been read as a
`sysw` one (see **[I1]**).

**Why the plan permits it.** The claim was written from the function names —
`first_noncanonical` reads like it covers all whitespace — rather than from the
order of the statements in the function that calls it, and the trim is the first
line of the range the plan itself cites.

**Confidence:** High. Both functions read; the behaviour executed end to end on
the shipped binary and the packed bytes inspected.

---

## [I3] V27 cannot be constructed under §3's own NORMATIVE one-transaction rule, and constructed the only way §3 allows it is masked by R10 — so R17 has no RED test and no closure condition

**Severity:** Important.
**Where:** §3's NORMATIVE sentence and its exception blockquote; **V27**; §4 step
10; §1.3's rule table; §6.2's four-row refusal table.

**The failure, concretely.** V27 is *"**two CHUNKS `tx:` records whose txids share
their top 20 bits** → REFUSED"*, and it is the vector for **R17**, the refusal
§1.4a's ruling creates.

§3 rules: *"**NORMATIVE: the transaction under every vector below is the `even`
vector … WITH ONE NAMED EXCEPTION — V7**"*, and closes the exception with
*"**V7 is the only vector this exception covers.** Every other vector below —
including V18 and V26 … — is the corpus transaction."*

Two txids that *share their top 20 bits* but are not equal require **two
different transactions**. Under §3's rule V27 may use only the corpus `even`
transaction, and the corpus's other vector does not collide (`even` set_id
`0x2dcf2`, `uneven` `0x3b426`, both read from `mt1_v1.json`). Built the only way
§3 permits — the same transaction twice — the two records carry **identical**
txids, which is **R10**, not R17; the payload is refused whether or not R17
exists, so **V27 stays green with R17's comparison deleted and R17 has no RED
test.** That is r2-M4's failure mode and r2-I3's construction failure at once.

R17 is uncovered by every completeness mechanism in the plan as well: it has **no
E-number**, so §6's *"Every rule E1–E20 has a test that goes RED without its
check"* does not reach it; and it is **not one of §6.2's four rows**, whose
heading is *"REFUSAL COVERAGE IS EVERY REFUSAL P1 ADDS, NOT JUST E1–E20"*. Its
only trace in the plan is V27's cell and step 10.

**Why the plan permits it.** The exception blockquote was written to rescue V7,
and it was scoped shut — *"V7 is the only vector this exception covers"* — in the
same fold that added V27, four rows below, whose input requirement is a second
transaction. The generator that would supply one (`scripts/gen-tx-record-vectors.py`,
step 9) exists by step 10 and is grindable in *"under a second"* by V27's own
quotation of `mt`'s help; the plan's own rule is what forbids using it.

**Confidence:** High. §3's scoping sentence read verbatim; both corpus set_ids
read from the JSON; R10 read at spec `:1541`.

---

## [I4] The wtxid is printed nowhere, so §1.1a's stated benefit — "a second value to compare against `mt inspect`" — is delivered by no site, and `show`'s txid-only line is exactly the value the accepted cost defeats

**Severity:** Important.
**Where:** §1.1a's ACCEPTED COST paragraph; §2.4's **W9**; §6's W9 row; §4 step 6.

**The failure, concretely.** §1.1a justifies a mandatory 32-byte field in an
immutable wire format on two grounds. The first is E17's refusal — verified, it
fires. The second is: *"the operator gains a **second value to compare against
`mt inspect`**, one that the txid provably cannot substitute for."*

No site emits it. W9 is *"It prints **one line per `tx:` record (form and carried
txid)** and one line per chunk SET"*; §6's W9 assertion is *"`me sysw show` on a
packed `tx:` payload prints the **carried txid**, and on a 202-chunk payload
prints one set line"*; `show` (`main.rs:1045-1088`) otherwise prints `sealed`,
`pub_len`, `ct_len`, `identity` and the digest. **The wtxid appears in no output
in this plan.**

This is not a cosmetic omission, because of what §1.1a itself concedes two
sentences earlier: *"A record whose txid AND wtxid are both recomputed from a
stripped body is internally consistent, and **nothing in the record can tell it
from an honest witness-free transaction**."* That case is **V26**, and the plan
requires it to **PASS**. I built the pair: V18 and V26 are 188-byte records whose
bodies are the same 113 stripped bytes and which differ in exactly 32 positions —
the wtxid. V26 carries **the same txid as the honest 222-byte record**, so an
operator comparing `show`'s carried txid against `mt inspect` gets a **match** on
a payload with 109 bytes of signatures removed. The only value that separates them
is the one the tool never shows.

**Why the plan permits it.** W9 was written to close r2-I5 — *"`show` prints no
line at all for a `tx:` record"* — and it closes exactly that, choosing the txid
because r2-I5's gate named the txid. §1.1a's second justification was written in a
different section and no closure condition binds them together.

**Confidence:** High for the omission (W9's text, §6's row and `show`'s whole arm
read). High for the consequence (V18/V26 constructed; the shared txid measured).

---

## [I5] §6 lists three spec corrections as "still owed" while this fold has already landed the third, and the spec it edited says only two remain

**Severity:** Important.
**Where:** §6's *"Spec corrections this phase owns — THREE, not one"* bullet,
item 3, and the bullet immediately after it; against
`design/SPEC_engrave_transaction.md:411-417` and `:424-430`.

**The failure, concretely.** §6 states the phase still owes three corrections, and
item 3 is: *"the 10/2 chunks row's *"18,583 … ✅"* → the ✅ is **correct**; the
number is the chunk text alone and the container cost is **18,737 chars (13,997
spare)** … **The spec table gains a container column.**"* The next bullet says:
*"**The SPEC EDITS this fold already made are spec §2.1, §2.3, §3.6b and §5**,
and they are **the ruling's, not these three corrections'**. The three above are
still owed by the phase."*

The fold's own §2.3 edit **is** correction 3. The spec's table now reads:

```
| in/out | … | as chars | **chunks IN THE CONTAINER** | fits 32,734? |
| 10/2   | … | 18,583   | **18,737**                  | ✅ (**13,997 spare**) |
```

— the container column, the 18,737 and the 13,997, all of it. And the spec's own
amendment block enumerates what remains, and it enumerates **two**:
*"**STILL OWED BY P1, and deliberately NOT edited here:** the headline *"a
16,367-byte raw transaction"* … **and** the 5/2 row's *"raw-only at 8191, by 31
chars"*"*.

So the plan says three are owed and the spec says two, about the same three items,
after one fold that touched both files. Whoever burns §6's closure list finds item
3 already done and must decide whether the list or the artifact is wrong — the
same class as r1-I8's unsatisfiable close condition, inverted.

**Why the plan permits it.** The corrections list was **re-derived** for this fold
(correctly — I recomputed all three) at the same time as the spec was edited under
the ruling's authorisation, and the two acts were reconciled in the spec's
amendment block and not in the plan's.

**Confidence:** High. Both documents read at the cited lines; all three
corrections' arithmetic independently recomputed.

---

## [I6] §6 and §6.1 carry four v3-era counts the fold did not propagate, in the section whose whole job is to state measured values

**Severity:** Important.
**Where:** §6.1's closing `ROOTS` paragraph, §6.1's `--terms` preamble, §6's
struck-grep bullet.

**The failure, concretely.** §6.1's gate table states the fold's measured values
and I reproduced every one of them exactly — **73/90 with 17 dangling, 9 into
`mnemonic-transaction` and 8 into `bitcoin`**; **120 rows, 0 malformed**; **12
fold-sweep hits**. Four sentences elsewhere in the same two sections still carry
v3's numbers:

1. §6.1, closing paragraph: *"Adding `/scratch/code/shibboleth/mnemonic-transaction`
   to that `ROOTS` list is the one-line change that would bring the **FOUR**
   `mnemonic-transaction` citations inside the gate, **leaving 4**. The `bitcoin`
   **four** cannot be gated at all."* Measured: **nine** and **eight**, leaving
   eight. Three wrong numbers in two sentences, twelve lines after the paragraph
   that gives the right ones.
2. §6.1, `--terms` preamble: *"The `--terms` list is fixed … **These six**, each of
   which this fold removed"* — followed by a block of **twelve**, which the gate
   row above it correctly calls *"the twelve below"* and the paragraph below it
   correctly splits as *"The last six are v4's"*.
3. §6's struck-grep bullet: *"W1–W3 all live in `record.rs` and **W4–W6** all live
   in `mod.rs`, so it sees **two files, not five sites**"* — as a present-tense
   description of ten sites it is wrong twice over: W4–W7 **and W10** live in
   `mod.rs`, W8 and W9 live in `main.rs`, and the cited grep's path
   (`crates/me-cli/src/sysw/`) **cannot see main.rs at all**, so it is *three*
   files and *ten* sites.

§6.1 makes these counts a close condition — *"**Any eighteenth is a defect**"* —
and asserts *"Every command above was executed while writing this fold and its
output is what §1.1, §1.4, §2.2, §3 and §5 state."* A reader auditing the gate is
handed two answers in one section.

**Why the plan permits it.** These are exactly the r1-C1 shape the fold sweep
exists to catch and cannot: numbers, not tokens. `plan-fold-sweep.sh`'s own header
says so — *"It cannot judge SEMANTIC staleness — a sentence that is now false
without containing any removed token will pass clean"* — and none of `4`, `six`,
`two files` or `five sites` was named in the `--terms` list.

**Confidence:** High. All three gates executed; the outputs are in the header of
this report.

---

## [I7] The spec edit falsified five statements outside the diff — the architect named three, two of which check out, and it missed two more

**Severity:** Important.
**Where:** `design/SPEC_engrave_transaction.md` §3.6, §2.1b, §6's P1 row, §6's P3
row, §1's ownership table.

**The failure, concretely.** Taking the architect's three first:

- **§3.6, line 761-762 — STALE, confirmed.** *"It is now **asserted** for a chunks
  payload, and **R15 validates it only *within* a single record** — so two records
  may carry the same txid and hold different transactions."* The fold amended the
  **identical clause** in §5's R10 row — *"within one transaction's own record set
  — amended 2026-08-24: 'one record' became 'one metadata record plus its bare
  chunk records'"* — and left this one standing. §3.6 is the section a P4
  implementer reads for the picker; taken at its word they check R15 inside one
  record and never look at the bare chunks, which is now the whole of the binding.
- **§2.1b — STALE, confirmed, but not where I expected.** Its NORMATIVE minimum
  (*"how the raw form and the chunks form are distinguished … and the carried
  txid"*) is merely incomplete against the new mandatory `wtxid`, since it says
  *"at minimum"*. What is outright **false** is its dependency table, row 2:
  *"**R4′** (both forms in one record) | it must be able to *see* both forms to
  refuse them"* — because §5's own R4′ amendment, written in this diff, says the
  opposite: *"A `tx:` record has one `form` byte, so the two forms **can no longer
  both be present in one record**."*
- **§6's P1 row — STALE, confirmed.** *"`ClassTransaction`, the framed record
  **including the mandatory 32-byte carried txid (§2.1b, §3.6b)**, stdin,
  content-based sealing, `MaxSectionLen` → 32,734 — with vectors."* It omits the
  mandatory **wtxid**, the **bare-`mt1` classifier branch** (`ValidMT`, the plan's
  W5 and its own §2.1's *"P1 and P3 each need the `ValidMT` branch"*), and the
  **payload-level set rule** the ruling creates. This row is P1's scope statement
  and it now understates P1's scope by the two largest things in the fold.

**And two the architect did not name, both the same class as the P1 row:**

- **§6's P3 row, line 1620 — STALE.** *"Port P1, provenance-pinned. **Includes the
  `tx:` branch in `gui/scan.go` (§2.1a)** — the prefix without the branch is the
  C3 defect."* The ruling adds a **second** required branch to that same file, and
  the plan's own §7 assigns it to P3 by name: *"**`gui/scan.go` must route a bare
  `mt1` record** | **P3** | … It needs the Go half of W5, beside the `mdmkText`
  branch at `:91-92`."* Neither §6's P3 row nor §2.1a mentions it, so the sequencing
  table under-scopes the phase that is most likely to read only that table.
- **§1's ownership table, line 107 — STALE.** *"| `me` | the `sysw` container |
  `ClassTransaction`; **a stdin path**; content-based sealing |"*. `me` also gains
  a second record class and a payload-level rule; §2.1's own amendment says so
  fourteen lines later.

**Why the diff permits it.** The edit was authorised narrowly (§2.1, §2.3, §3.6b,
§5) and executed inside that scope. Every one of the five sites above is text the
diff did not touch, which is precisely the class §6.1's own fold-sweep paragraph
says a diff cannot find — and the sweep was run against the **plan**, never
against the spec.

**Confidence:** High. All five read at the cited lines and set against the
amendment text that falsifies each.

---

## [M1] `mt-codec = "0.1.0"` is a caret requirement, not a pin, and the plan calls it "pinned" four times

§2.2's decision table writes the version as **`= "0.1.0"`** — the exact-pin
operator — while §4 step 4 and §5 both write the manifest line as
`mt-codec = "0.1.0"`, which Cargo resolves as `^0.1.0`. §2.2, step 4, §5 and §6
each call it *"the pinned published version"*. A caret requirement lets a
`0.1.1` publish change `decode_chunk`'s tolerance or `decode`'s reassembly under
`me` without a manifest edit, which is the whole thing §5's irreversibility
paragraph is guarding. One character, stated two ways in three places.

## [M2] §4.2 says `--in` "filters blank lines"; it filters EMPTY lines

`read_records` (`crates/me-cli/src/main.rs:1217-1221`) filters on
`!l.is_empty()`, so a line containing a single space survives as a record. §4.2
makes stdin's behaviour NORMATIVE *"exactly as `--in` does"* and describes that
behaviour as *"filters blank lines"*. Under the plan's own E13 the difference is
now load-bearing in a way it was not before: a whitespace-only line becomes a
record, is not any class, and lands in §1.5's refusal path — reported by
**[C2]**'s false `Unrecognised` message.

## [M3] §2.4's "V1–V27 are record-level vectors … stay green with `classify` untouched" is false for five of them

The sentence is the argument for step 6 being irreplaceable. **V20** (uppercase
`mt1`), **V23** (padded `mt1`) and **V24** (BCH-repaired `mt1`) are bare chunk
records with no `tx:` framing at all: nothing but `classify`'s W5 branch can
refuse them, and step 8 nonetheless files V19–V24 under *"the layout codec"*.
**V25** and **V27** are payloads, decidable only in `split` (W10), as §2.2 itself
says. The conclusion (step 6 is still needed) survives; the claim does not, and it
tells an implementer to site three of E13's and E19's tests at a layer that never
sees their input.

---

## The arithmetic — recomputed

Every figure below was **computed here**, not transcribed from the plan, and
compared afterwards. `CAP = 32,734`, framing `FR = 75`, metadata record
`meta = 3 + 2×75`, chunk chars `n × (⌈(b×8+55)/5⌉ + 16) + (n−1)`.

| quantity | plan says | recomputed | verdict |
| --- | --- | --- | --- |
| framing | 75 = `4+1+1+32+32+1+4` | **75** | ✅ |
| E4's fixed constant | 71 | **71** = `4+1+1+32+32+1` | ✅ |
| `n_fields` offset | 70 | **70** | ✅ |
| shortest legal record (E5) | 153 chars | **153** = `3 + 2×75` | ✅ |
| hex chars available | 32,731 → 16,365 B, one char spare | **16,365**, 32,730 used | ✅ (floor, as stated) |
| **body ceiling** | **16,290 B** | **`(32734−3)//2 − 75 = 16,290`** | ✅ |
| v3's ceiling | 16,322 | **`16365 − 43 = 16,322`** | ✅ |
| 1/1 raw record / container | 1,857 / 2,155 | **1,857 / 2,155** | ✅ |
| 1/2 | 1,939 / 2,246 | **1,939 / 2,246** | ✅ |
| 2/2 | 3,537 / 4,109 | **3,537 / 4,109** | ✅ |
| 5/2 | 8,313 / 9,537 | **8,313 / 9,537** | ✅ |
| 10/2 | 16,287 / 18,737 | **16,287 / 18,737** | ✅ |
| 10/2 chunks spare | 13,997 | **`32734 − 18737 = 13,997`** | ✅ |
| 10/2 raw spare | 16,447 | **`32734 − 16287 = 16,447`** | ✅ |
| 5/2 raw over the OLD 8191 cap | 122 | **`8313 − 8191 = 122`** | ✅ |
| v3 10/2 chunks-in-one-record | 37,255, over by 4,521 | **`3+2×(43+18583) = 37,255`; `−32734 = 4,521`** | ✅ |
| v3 10/2 raw | 16,223, 16,511 spare | **16,223 / 16,511** | ✅ |
| 202 bare chunks | `202×91 + 201 = 18,583` | **18,583** | ✅ |
| corpus chunk length | 87 chars for 37 B | **`⌈(37×8+55)/5⌉+16 = 87`**, and the JSON's `string_lengths` are `[87]×6` | ✅ |
| corpus `uneven` | 284 B | **284** | ✅ |
| `body_len` u32 → "4 GiB" (E5) | 4 GiB | **4,294,967,295** | ✅ |

**§2.3's three corrections, each re-derived independently:**

1. *"a 16,367-byte raw transaction"* → **16,290 B of body, minus the fields.**
   ✅ correct, and **still owed** (spec `:398` unedited).
2. *"✅ (raw-only at 8191, by 31 chars)"* → **false in both halves**: at the old
   cap the raw record was 8,313 (122 over) and the chunks form 9,537, so
   **neither** fitted; at 32,734 both do. ✅ correct, and **still owed** (spec
   `:410` unedited).
3. *"10/2 … 18,583 … ✅"* → the ✅ **is** correct and the number is incomplete;
   the container cost is **18,737 (13,997 spare)**. ✅ correct — **and it
   INVERTS**, exactly as claimed: under v3's framing the same row computed to
   37,255, i.e. **4,521 OVER**, so the verdict went *false* → *true but
   incomplete*. **But it has already been landed in the spec** — see **[I5]**.

**Nothing in §1.1a, §1.4, §1.4a, §3.1 or the spec's new container column is
arithmetically wrong.** All twelve rows of the recomputation and all three
corrections reproduce to the digit.

---

## Verdict

**2 Critical / 7 Important / 3 Minor. NOT GREEN. No code.**

Part 1: **12 FIXED / 1 PARTIAL / 0 NOT FIXED** of round 2's thirteen, plus round
1's two carry-overs at 1 FIXED / 1 PARTIAL. **r2-C1 is genuinely closed and I
proved it by construction rather than by reading**: E17 fires on the input E11
returned `true` for, the wtxid's equality-with-txid property for witness-free
transactions is a definition and not an accident, and V18/V26 are the same 113
bytes differing in exactly 32 — the residual is a real vector, not a paragraph.
The measured content of this fold is trustworthy in a way that is now consistent
across four rounds: **every arithmetic claim, every gate value, every corpus
fact and every `bitcoin`/`mt-codec` API fact reproduced exactly** when re-run
from an independent implementation.

What fails is one axis, and it is the same one round 2 named:

- **[C1] and [C2] are r2-C2, still open, and now in three places instead of one.**
  The fold added the *carrier* (W7) and the *renderer* (W8) for a rule name and
  never added a *producer*; W6's prescribed edit makes §6's own W8 assertion RED;
  and the two record classes the ruling created in the same fold — a bare `mt1`
  chunk, and a **set** — have no channel at all, no `SyswError` variant, and a
  §1.5 contract they cannot satisfy. The measured false message is the same one
  round 2 quoted, now reaching three more vectors.
- **The ruling is sound and its arithmetic is right; its TEST surface is where it
  is thin.** [I1] E12's negative test is in the wrong container. [I2] E13's
  precedent does not exist — measured. [I3] V27 cannot be built and R17 has no
  closure condition anywhere. [M3] three vectors are filed at a layer that cannot
  see them.
- **[I4] is the one finding about the wtxid itself**, and it is not about the
  field: it is that the plan's second reason for adding it is delivered by no
  site, while `show` prints the one identifier the accepted-cost case defeats.
- **[I5], [I6] and [I7] are propagation.** The fold re-derived three spec
  corrections correctly and then said all three were owed after landing one;
  restated four v3-era counts in the section that measures them; and left five
  spec statements falsified outside the diff, of which the architect self-flagged
  three (two check out as stated, one for a different reason than given) and
  missed two more.

### W1–W10 — is it complete?

**All ten resolve, and the count of TEN is not.** Every cited range is exact:
`record.rs:27-28` (the two prefixes), `:31-40` (`enum Class`, **eight** variants
today → ten, as §6's W2 row says), `:50-55` (`is_secret`'s `matches!` over three);
`mod.rs:96-105` (`UnknownReason`), `:107-115` (`unknown_reason`), `:124`
(`classify`), `:251-259` (`split`); `main.rs:1156-1181`
(`print_mdmk_confirmation`, whose second statement is the `!= Class::MdMk`
`continue`), `:1263-1280` (`sysw_error`'s `Unclassifiable` arm). The causal chain
§1.4a asserts for cost 1 is also right: `sysw::classify` → `seal::record::validate_record`
→ `crate::classify::classify` → `UnknownHrp("mt")` → `Class::Unknown`, and I
executed the refusal. Adding two `Class` variants breaks **no** exhaustive match
(`split`'s has a catch-all `_`, `is_secret` is a `matches!`, and the only other
use is a test at `vectors.rs:242`), so there is no eleventh site there.

**The eleventh and twelfth sites are the ones [C2] names:** `SyswError`
(`sysw/mod.rs:58-87`) needs a variant no W-row adds, and `sysw_error`'s **outer**
match (`main.rs:1257-1300`) needs an arm — W8 adds one to the **inner**
`UnknownReason` match only. Without both, W10's E20 pass and spec §5's R17 have
no error value to return and no way to name a set, an index, or "both txids".

*(A third site exists but I do not count it against §2.4: content-based sealing
(step 3) must inspect record classes in the pack arm at `main.rs:910`/`:927`,
where `sealing = !*no_passphrase` today. §2.4 is scoped to wiring the class, §4
step 3 owns the change, and §6's W3 row cross-links them explicitly.)*

### What I did NOT examine

- P2–P6, the device, the plate, `design/agent-reports/`, style, wording, length.
  EPD, treated as frozen.
- The **bare-records ruling itself** — an operator decision in spec §8. I report
  its unhandled consequences (**[C2]**, **[I3]**, **[I7]**) and express no
  preference about the ruling.
- Whether `mt-codec` publishes cleanly. `cargo publish --dry-run` still not run;
  the plan names it a precondition, not a close condition, and I left it there.
- The BCH/bech32 correctness of `mt-codec` and of `mt1_v1.json` as a corpus —
  ground truth, per rounds 0–2. I did independently verify the `even` vector's
  222 bytes, both identifiers, its `set_id`, its segwit marker/flag, its
  `txid_is_wtxid: false`, and its six 87-character strings, and I read
  `decode_chunk`, `decode`, `ChunkHeader` and `DecodedChunk::corrected` at source.
- Whether `me`'s existing suite stays green under steps 1–3. I did not run
  `cargo nextest`; I ran the shipped `target/debug/me` only.
- Whether `bitcoin 0.32` builds at `default-features = false, features = ["std"]`
  — settled by rounds 1 and 2 and not re-derived. My E11/E17 measurements used an
  independent Python serialiser precisely so they do not depend on it.
- R2's argv guard as an implementation site. Round 2 confirmed clap does not echo
  an argv record on a malformed `me sysw pack`; I did not re-run it. **One
  adjacent question I could not close and am flagging rather than filing:** R2
  refuses a `tx:` record on argv as bearer material, and §1.4a's ruling now routes
  the same transaction through **bare `mt1` records**, for which the plan states
  no argv posture either way. It may be deliberate; the plan does not say.
