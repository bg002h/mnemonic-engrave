# R0 — IMPLEMENTATION_PLAN_P1_me_container.md, round 7 (failure-states lens)

**Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md` at v9 (`75a833d`).
**Lens:** *For every way this thing can go wrong, what does the OPERATOR actually
see — and can they act on it?* Never run on this plan before.
**Reviewer:** independent; did not write v9 and folded none of rounds 0–6.

**Excluded by the brief and NOT reported:** F-246 (`me sysw pack` prints a
generated passphrase before validating records). It is used below only as the
calibration shape.

---

## Commands run, and their raw output

```
$ git log --oneline -1
75a833d fold: P1 plan v9 -- round 6's 2C/5I/7M, and the first Critical no correctness round could reach

$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
91 / 108 ; DANGLING = 17  (9 mnemonic-transaction, 8 bitcoin-0.32.9)   exit 1
$ ./scripts/plan-table-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
table rows checked: 151 ; malformed: 0                                  exit 0
$ ./scripts/plan-wiring-check.sh design/IMPLEMENTATION_PLAN_P1_me_container.md
                                                                        exit 0
```

All three reproduce the brief's settled figures exactly. Not re-derived further.

```
$ cargo build --quiet --bin me            # target/debug/me, 63,355,360 bytes
```

**(1) a bare `mt1` record today**

```
$ me sysw pack --no-passphrase 'mt1p9h8jqq9qqqqgqq...229sax'
exit=4
strength: no passphrase — BELOW the threshold
me: record 0 (records count from 0) is not a form this container can place: not a
    BIP-39 mnemonic, not an md1/mk1/ms1 string, and not a `text:`/`pass:` record.
    Descriptors and addresses are not yet classifiable here — see sysw::classify
```

**(2) a `--in` line holding a single space survives as a record**

```
$ printf 'abandon ... about\n \nfoo\n' > /tmp/rec.txt
$ me sysw pack --no-passphrase --in /tmp/rec.txt
exit=4
me: record 1 (records count from 0) is not a form this container can place: ...
```

**(3) the section-overflow refusal — the plan never mentions it**

```
$ python3 -c "print('text:'+'61'*5000)" > /tmp/big.txt
$ me sysw pack --no-passphrase --in /tmp/big.txt          # stdout captured
exit=4
strength: no passphrase — BELOW the threshold
me: these records are too long for one payload: a section caps at 8191 bytes.
    Split them across two payloads.
--- stdout bytes: 0
$ me sysw pack --no-passphrase --in /tmp/big.txt --out /tmp/outfile.bin
exit=4 ; ls: cannot access '/tmp/outfile.bin': No such file or directory
```

§1.5's *"nothing on stdout, nothing is written"* is **TRUE** on today's abort
path. Confirmed, not assumed.

**(4) an EMPTY `--in` file packs a container at exit 0**

```
$ : > /tmp/empty.txt
$ me sysw pack --no-passphrase --in /tmp/empty.txt --out /tmp/empty.bin
exit=0
strength: no passphrase — BELOW the threshold
digest:   none — this payload has no public section
-rw------- 1 bcg bcg 52 /tmp/empty.bin

$ printf '\n\n\n' > /tmp/blank.txt ; me sysw pack --no-passphrase --in /tmp/blank.txt --out /tmp/blank.bin
exit=0 ; -rw------- 52 /tmp/blank.bin

$ printf '' | me sysw pack --no-passphrase
exit=2
me: no records: pass them on argv or with --in
```

**(5) `me sysw show` is silent on a record it cannot classify**

```
$ me sysw pack --no-passphrase 'text:6162' --out /tmp/t.bin
$ python3 -c "d=bytearray(open('/tmp/t.bin','rb').read()); i=d.find(b'text:6162'); d[i:i+9]=b'tx:abcdef'; open('/tmp/t2.bin','wb').write(bytes(d))"
$ me sysw show /tmp/t2.bin
sealed:   false
pub_len:  9
ct_len:   0
identity: f1060df265102c442b676f0f57dee4018621b8842ff751dae912533994f3110b
digest:   83c1 441c b2a7 60df 307d 88f1 3b5e 065b
exit=0
```

No line for the record. Exit 0.

**(6) the CHUNKS ceiling, computed with the plan's OWN formula (§6.1)**

```
$ python3 -c "<plan's cc = ceil((b*8+55)/5)+16, CAP=32734, FR=75, meta=3+2*FR>"
10/2 raw record 16287 chunk chars 18583 container 18737 spare 13997   <- reproduces §3.1 exactly
RAW body ceiling      : 16290 B    (the plan's figure)
CHUNKS max chunks     : 354 at 91 chars each
CHUNKS body ceiling   : 14160 B    <- STATED NOWHERE IN THE PLAN
  with a full legend (64 B label + 8 + 4): metadata record 323 chars, 352 chunks => 14080 B
engraveable ceiling (spec §4.1a)   : 14560 B
ordering:  CHUNKS 14160  <  engraveable 14560  <  RAW 16290
```

The model is validated: it reproduces §3.1's published 10/2 row (16,287 /
18,583 / 18,737 / 13,997) to the character.

**(7) grep evidence used below**

```
$ grep -n 'chunk_charset\|chunk_bch' design/IMPLEMENTATION_PLAN_P1_me_container.md
913:| `tag_order` | E1 | `chunk_charset` | E13 |
919:| `hex` | §2.5a | `chunk_bch` | E19 |
      -- two hits each, both inside §2.5a.1's vocabulary table. NO SITE PRODUCES THEM.

$ grep -n 'SectionTooLong\|Split them\|too long\|empty file' design/IMPLEMENTATION_PLAN_P1_me_container.md
      (no hits)
```

---

## Part 1 — the error surface, enumerated

Every failure the document can produce, and what the operator sees. **"—"
means the plan specifies nothing.**

### A. the twenty-one per-record rule names (§2.5a.1), via W11 → W12 → W8

Channel: the `tx:` parse returns `TxRecordError`; `split` re-runs it; `SyswError::TxRecord(usize, TxRecordError)`; `sysw_error` prints
*"record {i} (records count from 0) is a `tx:` record that fails rule {rule}"*; **exit 4**.

| rule | producer | operator sees | distinct? | verdict |
| --- | --- | --- | :-: | --- |
| `magic` `version` `form` | `tx:` parse | W8's line, three distinct | yes | OK (V13) |
| `tag_order` `tag_duplicate` `empty_tlv` `unknown_tag` `n_fields` `label_utf8` `label_len` `tag_width` | `tx:` parse | W8's line | yes | OK |
| `hex` | `tx:` parse | W8's line | yes | OK (§2.5a rules out `NonHexBody`) |
| `wtxid` `txid` `reserialise` `form_body` | `tx:` parse / §2.2 | W8's line | yes | OK |
| `trailing_bytes` (E3) | `tx:` parse | W8's line | **no** — E4 `length_mismatch` describes the same input, and V10's cell says *"E3/E4"* | **[I5]** |
| `length_mismatch` (E4) | `tx:` parse | W8's line | **no** — see above | **[I5]** |
| `body_len` (E5) | `tx:` parse | W8's line | yes | OK (V11) |
| **`chunk_charset` (E13)** | **NONE** — E13 fires on a **bare `mt1`** record, which the `tx:` parse never sees | in practice `Unrecognised`'s false line, or W8's line calling it a `tx:` record | **no** | **[C1]** |
| **`chunk_bch` (E19)** | **NONE** — same | same | **no** | **[C1]** |
| a **truncated** record (< 75 B decoded) | `tx:` parse | — **no name exists** | — | **[I5]** |
| a record over the section cap (V7's near-miss) | **not the codec at all** — `wire::SectionTooLong` | *"these records are too long … Split them across two payloads."* — no index, no rule | — | **[I3] [I2]** |

### B. set-level failures, via W10/W15 → W12's set variant → W13

| failure | rule name | message | exit | verdict |
| --- | --- | --- | :-: | --- |
| E20 missing index | — (excluded from §2.5a.1) | illustrative only: *"chunk 7 of set `0x2dcf2` is missing"* | 4 | **[M5]** singular; six missing is unspecified |
| E20 orphan chunk | — | illustrative only: *"record 12 is an orphan"* | 4 | **[M1]** implied remedy inverts |
| E20 duplicate index | — | — | 4 | thin, but bounded by E20's text |
| **R17 colliding sets** | **none anywhere** — §2.5a.1 excludes it; **V27's RED-ness is defined as *"asserts R17's RULE NAME on stderr"*** | illustrative only: *"two sets share top-20 bits"* | 4 | **[C2]** |
| **W15 reassembly is not a transaction (V28)** | **none anywhere** | **none anywhere** | 4 | **[C2]** |

### C. policy and input refusals

| refusal | exit | message specified? | verdict |
| --- | :-: | --- | --- |
| R2 — `tx:` **or** bare `mt1` on argv (step 11) | 3 | shape only; asserts the record text is absent | OK |
| R7 — empty **stdin** (step 2) | 2 | **—** | **[M2]** collides with the TTY refusal at the same site |
| the TTY refusal (§4.2) | 2 | *"naming both real inputs"* | **[M2]** |
| §2.3 decode failure | 4 | via W8 | OK |
| **empty / blank-only `--in`** | **0** | **no refusal at all — a 52-byte container is written** | **[I1]** |
| a `--in` line holding one space | 4 | §4.2 claims *"via W11"* — **measured false** | **[M3]** |

### D. read-back

| moment | what the operator sees | verdict |
| --- | --- | --- |
| `me sysw show` on a good `tx:` payload | W9's line: form, carried txid, carried wtxid | OK |
| `me sysw show` on a good 202-chunk payload | one set line | OK (content unspecified) |
| **`me sysw show` on a record it cannot parse** | **nothing. exit 0.** Measured, cmd (5) | **[I4]** |

---

## [C1] Two of the twenty-one normative rule names can only be produced by a record that is not a `tx:` record — no site produces them, and W8's message template calls the record a `tx:` record anyway. This is r3-C2 reintroduced through the very channel built to close it.

**Severity:** Critical
**Where:** §2.5a.1 (lines 913, 919); W5 (line 781); W8 (line 784); W12 (line 788); §4 step 6 (line 1341); §2.5's own table (line 876).

**The failure, concretely.** The operator pastes 202 `mt1` chunks into a file
and one line carries a trailing space — the single most likely defect in a
copy-paste of 202 bech32 strings, and the one E13 exists for. Or one symbol is
mistyped and `decode_chunk` silently repairs it — E19's case. Or the block was
pasted from a source that upper-cased it — E13's other half. All three are
plan vectors (V23, V24, V20) and all three are asserted at **step 6**.

Follow the channel the plan builds:

- **W5** is the only site that touches a bare `mt1` record: *"`classify` gains a
  `ValidMT` branch … In Rust it is `mt_codec::decode_chunk(r, None)` plus E13 and
  E19; success is `Class::MtChunk`."* `classify` returns a **bare `Class`** —
  W12 says so in its own words: *"`classify` returns a bare `Class` and
  structurally cannot carry an error out."* So failure yields `Class::Unknown`.
- **W12's rescue clause names W4 only:** *"W4's branch decides only which class,
  while `split` re-runs the parse for the error."* **W4 is the `tx:` branch.**
  Nothing says `split` re-runs W5's chunk validation.
- So the record reaches `split`'s shipped arm
  `Class::Unknown => Err(SyswError::Unclassifiable(i, unknown_reason(&r)))`, and
  the operator gets, **measured on today's binary** (cmd 1):

  > `me: record 7 (records count from 0) is not a form this container can place:
  > not a BIP-39 mnemonic, not an md1/mk1/ms1 string, and not a `text:`/`pass:`
  > record. Descriptors and addresses are not yet classifiable here — see
  > sysw::classify`

  **Every clause of that is false about the record.** It *is* an `mt1` string; it
  is off by one space. And the plan already knows: §2.5's middle row says of
  exactly these three vectors *"they reach `Unrecognised`, whose message is
  **false** for V20/V23/V24 (r3-C2)"*. W11–W13 were the NORMATIVE answer to that
  Critical, and they do not reach the chunk half.
- **And the alternative reading is no better.** §2.5a.1 puts `chunk_charset` and
  `chunk_bch` in the vocabulary and says the omissions are *"E7 … E12 … E20 and
  R17"* — so these two are `TxRecordError`'s by construction. An implementer who
  routes them through W8 gets:

  > `me: record 7 (records count from 0) is a `tx:` record that fails rule chunk_charset`

  which tells the operator their `mt1` chunk is a `tx:` record. The two forms are
  framed differently, hex-encoded differently, and produced by different `mt`
  subcommands. The operator goes looking at their `tx:` record, which is fine.

**Why the plan permits it.** §2.5a.1's brief was *"one per E-number that can fail
a **single record**"* — a scoping rule about arity. E13 and E19 satisfy it (they
are single-record rules) while violating the unstated one that actually matters:
`TxRecordError` is produced by *the `tx:` parse*, and a bare `mt1` record never
enters it. The two rules were derived by walking §1.3 and never re-checked
against W11's producer. `grep -n 'chunk_charset\|chunk_bch'` returns **two hits,
both inside the vocabulary table** — the classic no-producer shape this cycle
already retracted W7 for.

**What the operator cannot do.** With 202 chunks and a one-character defect, the
message must name the record and say the string is not pristine. Neither
candidate message does. §1.5 is NORMATIVE that a refusal is *"one line on stderr
naming the record's index and the RULE"* — for E13/E19 the rule is unreachable,
so §1.5 is unmet for two of its own twenty-one rules.

**Confidence:** high. The producer gap is a grep; the message text is quoted from
W8's own cell and from a measured run.

---

## [C2] R17's and W15's refusals have no rule name and no message anywhere in the document — while §3's V27 row makes *"R17's RULE NAME on stderr"* the assertion that decides whether the vector can go RED, and W15 is the anti-smuggling gate added last round.

**Severity:** Critical
**Where:** §2.5a.1's omissions paragraph (line 924); V27 (line 1102); V28 (line 1103); W12 (lines 788, 892); W15 (line 789); §6's W15 row.

**The failure, concretely.** Five distinct set-level failures exist, with five
different remedies:

| the operator's payload | what they must DO |
| --- | --- |
| chunk 3 of 6 missing | add the missing chunk |
| an orphan `mt1` chunk | add the `tx:` metadata record it belongs to |
| chunk 3 present twice | delete one copy |
| **two transactions whose txids collide in 20 bits (R17)** | **pack them as two separate payloads** — nothing else works, the txids cannot change |
| **the set reassembles to bytes that are not a transaction (W15, V28)** | **the chunks are not a transaction at all — this is the smuggling case** |

§2.5a.1 exists, in its own heading's words, to *"enumerate them, do not leave
them to an implementer"*, and it then removes exactly the two that matter most:
*"**E20** and **R17** are **set**-level, so they are W12's variant and not
`TxRecordError`'s."* W15's failure is not in the table either, and is not
mentioned in the omissions sentence at all — it is simply absent.

**So there is no name, and the plan then asserts one.** §3's V27 row:

> **V27 asserts R17's RULE NAME on stderr**, and goes RED when E20's
> set-completeness message appears in its place

That test cannot be written. There is no R17 rule name to assert, in the fixture,
in `TxRecordError`, or in W12's variant. And V27's own row explains why this
matters more than usual: two colliding sets make **every chunk match both
`set_id`s**, so **E20 refuses the payload unaided** — the bare refusal is not the
assertion, the *name* is. An implementer who reuses E20's set-completeness string
produces a test that **passes with R17's comparison deleted**, which is the
"gate that cannot fail" shape §2.4 and §6 have struck twice already.

**W15 is worse, because it is the gate.** §6's W15 row says only *"**V28 is
REFUSED**"*. §3's V28 row says *"REFUSED by W15's deserialisation"*. Neither
names a message. W15 is set-level, so it travels on W12's set variant. If one
implementer gives it E20's string, an operator handed a payload from a
non-conforming sealer sees *"set `0x2dcf2` is incomplete"* — and does the obvious
thing: **goes looking for a missing chunk that is not missing**. The actual fact
— *these 202 chunks do not reassemble to a transaction, which is the channel §2.1
measured and named C3* — is the one thing they are not told. Round 6's Critical
built the site; its operator-visible half was never specified.

**Why the plan permits it.** §2.5a.1's scope was drawn around `TxRecordError`,
whose variants are per-record. The set-level channel got a variant (W12) and a
printer (W13) and never got a vocabulary, and nothing checks that the two
channels together cover the twenty-six refusals the plan can produce. The
three-way wire contract §2.5a.1 declares — generator, enum, Go port — therefore
covers 21 of 26 failures; the five that are excluded include both of the ones a
plan-reader would call security-relevant.

**Confidence:** high for the absence (grep: `R17` appears in W12, W13, §2.5a.1's
exclusion, V27, and §6.2's coverage table — never as a name); high for the
consequence, which V27's own row states in the same sentence it depends on.

---

## [I1] `me sysw pack --in <empty or blank-only file>` writes a valid container and exits 0 — while step 2 makes exactly the same emptiness on **stdin** a NORMATIVE refusal at exit 2. §4.2 rules the precedence between the three sources and never notices.

**Severity:** Important
**Where:** §4 step 2 (line 1332); §4.2 (lines 1379–1390); §6.2's R7 row (line 1849).
**Measured:** cmd (4).

**The failure, concretely.** `mt encode --record > tx.txt` fails, or writes to
stderr, or the operator redirects the wrong stream. Then:

```
$ me sysw pack --no-passphrase --in tx.txt --out payload.bin
exit=0
strength: no passphrase — BELOW the threshold
digest:   none — this payload has no public section
-rw------- 52 payload.bin
```

A 52-byte container with **zero records**, exit 0, ready to flash. Meanwhile the
plan spends step 2 and a row of §6.2's coverage table making the *stdin* form of
the same mistake a refusal:

| the operator's input | today / after P1 |
| --- | --- |
| `printf '' \| me sysw pack` | **exit 2**, R7, spec §5 rules it *"must join the existing exit-2 path"* |
| `me sysw pack --in /dev/null` | **exit 0**, a container |

Both are "the operator supplied no records". `read_records`
(`crates/me-cli/src/main.rs:1211-1227`) checks `argv.is_empty()` and never checks
the `--in` result for emptiness, so the guard lives on one of three branches.

**Why the plan permits it.** §4.2 is the section that rules the three sources,
and it reasons carefully about **line filtering** parity — *"Stdin filters EMPTY
lines exactly as `--in` does … a line holding a single space SURVIVES as a
record"* — and never asks the zero-records question. R7 is inherited from spec §5
as a *stdin* rule and P1 never widens it, so P1 ships a refusal on the new
channel and leaves the older sibling silently accepting.

**Compounding path (see [M1]).** An operator who packs chunks without their
metadata record is told *"record 0 is an orphan"*; the obvious action is to delete
record 0; after 202 deletions the file is empty — and **then it packs at exit 0.**
Following the messages end to end produces an empty payload and a success code.

**Confidence:** high; measured on the shipped binary. The pre-existing half is
noted honestly — the *asymmetry* is what P1 creates.

---

## [I2] The CHUNKS form has a section ceiling of ~354 chunks (~14,160 B of transaction) that the plan never states, has no vector for, and — because §3.1 says the framing ceiling *"no longer binds the CHUNKS form"* — invites the reading that the chunks path is the roomier one. It is the tighter one. Crossing it prints *"Split them across two payloads"*, which E20 then refuses.

**Severity:** Important
**Where:** §3.1's three-ceiling table (lines ~1150–1160) and the paragraph headed *"THE RECORD-FRAMING CEILING NO LONGER BINDS THE CHUNKS FORM"*; §1.4a; E20.
**Measured:** cmd (6), using the plan's own `cc` formula, validated against §3.1's published 10/2 row.

**The numbers.**

```
RAW    body ceiling   16,290 B      <- §3.1 states this
CHUNKS body ceiling   14,160 B      <- 354 chunks; STATED NOWHERE
                      14,080 B      <- with a full 64-byte TO label
engraveable           14,560 B      <- spec §4.1a
```

§3.1 is titled *"there are THREE ceilings and the spec states only two"* and its
table's third row says the record-framing ceiling *"binds the **RAW form
ONLY**"*. There is a **fourth**, it binds the form §1.4a's ruling created, and it
is **lower than the one the section computes**. The closing line —
*"One ceiling was removed from the path; none was raised"* — is true and
misleading: the path that lost a ceiling still has a tighter one, so
*"§2.2's XOR stays a real choice"* is false in the band 14,160 B–16,290 B, where
only RAW works.

**The failure, concretely.** An operator with a 15 KB consolidation chooses the
chunks form (it is the form for large transactions; the metadata record is a
fixed 153 characters and §3.1 says the framing ceiling does not bind it). They
get, verbatim from the shipped code path (cmd 3, with step 1's raised constant
interpolated):

> `me: these records are too long for one payload: a section caps at 32734 bytes.`
> `Split them across two payloads.`

**They cannot.** E20 requires each CHUNKS `tx:` record to have a **COMPLETE**
set in the payload: *"`count` chunks, indices `0..count-1`, no gap"*. Split the
375 chunks across two payloads and each half is refused for a missing index —
a second refusal, with a different message, that says nothing about size. The
message prescribes an action the next rule forbids.

**And it is indistinguishable from the case where the advice is correct.** Spec
§3.6 contemplates several transactions in one payload; two 10/2 spends are
18,737 × 2 = 37,474 characters, also over the cap, and *there* splitting across
two payloads is exactly right. One message, two failures, one remedy that works
and one that cannot — and nothing in it tells the operator which they have.

**Why the plan permits it.** `grep -n 'SectionTooLong\|Split them\|too long'`
over the plan returns **nothing**. §3.1 computes ceilings and never asks what the
operator sees when one is crossed; §1.5's exit table and §6.2's coverage table
both omit the section-overflow refusal; and the one message that says what to do
about it was written for a payload of independent `md1`/`mk1` cards, where
splitting always worked.

**Confidence:** high on the arithmetic (validated against the plan's own
published row); high on the message (executed); the *severity* rests on the
14,160–14,560 B band being reachable, which a 15-input consolidation is.

---

## [I3] V7's near-miss is not a codec refusal and structurally cannot be one, so it has no rule name and no place in the `expect` schema r6-C2 ruled codec-level — into which §3.3 files it at step 9.

**Severity:** Important
**Where:** §3.3's *"Which step files which vector"* paragraph; §3.3's r6-C2 NORMATIVE block; §3's V7 row; §4 step 9; §6's near-miss bullet.

**The failure, concretely.** §3.3 rules two things four paragraphs apart:

1. *"`tx_record_vectors.json` pins **what the record codec produces and refuses**
   — the parsed fields on `pass`, the **rule name** on `refuse`."* (r6-C2)
2. *"**Step 9** files **V7** and its `16,291 − F` near-miss"* — into that file.

V7's near-miss is a body of `16,291 − F` bytes. Walk it through the codec:
`magic`, `version`, `form` pass; every TLV rule passes; **E4's arithmetic
balances exactly**; **E5's `body_len` matches the bytes remaining**; E11 and E17
pass because it is a real transaction honestly identified. **The record codec
accepts it.** It is refused by `wire::SectionTooLong` at pack time — a
*container* outcome, which r6-C2's own ruling routes to
`crates/me-cli/tests/sysw_cli.rs` *"at steps 6 and 7"*, neither of which is
step 9.

**And it can never become a codec rule**, which is what makes this structural
rather than a mis-filing: the ceiling depends on what else shares the section.
§3.1 says so — *"For `k` records the bound is `Σ(record chars) + (k − 1) ≤
32,734`, so **V7 is explicitly the single-record vector**"*. The identical record
is legal alone and illegal beside a second one, so no function holding one record
can decide it.

**What the implementer at step 9 does.** Three exits, all wrong:

- write `"expect": {"refuse": {"rule": "section_len"}}` — inventing a name outside
  §2.5a.1, which that section forbids in its own words (*"A rule that gains a
  name later must be added HERE first"*), and which the Go port does not have;
- write `"expect": {"pass": …}` — and the near-miss stops being a refusal, so §6's
  near-miss bullet (*"V7 (`16,291 − F` refused / `16,290 − F` passes)"*) has no RED
  test;
- move it to `sysw_cli.rs` — correct, and contradicts §3.3's step-assignment
  paragraph and step 9's own cell.

**What the operator sees, which is the lens's half of this.** The record-framing
ceiling refusal reaches them as the §I2 message: no record index, no rule name,
and a remedy that does not apply. It is the only P1 refusal absent from **both**
§1.5's exit-code table and §6.2's coverage table.

**Confidence:** high. This is the r6-C2 defect one vector over; I raise it here
only for its operator-facing half and note the step-execution half as evidence.

---

## [I4] `me sysw show` prints nothing and exits 0 for any record it cannot classify. W9 specifies only the success case, so the read-back gate §6 leans on under-reports silently.

**Severity:** Important
**Where:** W9 (line 785); §6's W9 closure row; §6's *"`me sysw show` CAN ACTUALLY DO THIS"* bullet.
**Measured:** cmd (5).

**The failure, concretely.** A container carrying a `tx:` record that does not
parse — truncated in transfer, edited, produced by a non-conforming sealer, or
written by an older `me`:

```
$ me sysw show payload.bin
sealed:   false
pub_len:  9
ct_len:   0
identity: f1060df2…
digest:   83c1 441c b2a7 60df 307d 88f1 3b5e 065b
exit=0
```

**No line for the record.** The shipped shape is
`if classify(r) != Class::MdMk { continue; }`
(`crates/me-cli/src/main.rs:1156-1181`), and W9's cell says only *"It prints one
line per `tx:` record … and one line per chunk SET"*. An implementer writes
`if classify(r) != Class::Transaction { continue; }` — the file's own local
convention — and a record `show` cannot parse is invisible.

**Why it matters here specifically.** §6 makes `show` the read-back gate
(*"`me sysw show` reads it back"*), and §6's W9 row makes it the operator's
**only** detector for a witness-stripped payload: *"on **V26** … the printed txid
and wtxid are **EQUAL** … That equality is the signal that the payload carries no
signatures, and it is visible in `show`'s output or it is visible nowhere."*
A detector that renders nothing for what it cannot parse reports "clean" for a
payload it did not read. The operator's inspection before a 21-minute plate cut
returns exit 0.

**Partial mitigation, stated so this is not over-claimed:** `digest:` is printed
and would differ from the one `pack` printed, so an operator who recorded the
digest can detect tampering. That is a different check from the one W9 exists to
provide, and it does not fire for a container that was never seen before.

**Why the plan permits it.** Every W9 clause is phrased for the pass case. The
failure column of `show` is empty across the whole document; §1.5 binds the
*pack* path only (*"binding every RECORD-CODEC refusal"*), and `show` performs no
refusal at all.

**Confidence:** high on the behaviour (executed); medium on how often a
malformed record reaches a container, since `pack` refuses them — the population
is containers `me` did not create.

---

## [I5] The rule-name vocabulary is under-determined for length failures: a truncated record has **no** name among the twenty-one, and a trailing-bytes record has **two**. §2.5a.1 was written this fold to prevent exactly this.

**Severity:** Important
**Where:** §2.5a.1 (lines 908–928); §1.3's E3, E4, E5; §3's V10 row.

**Construction A — two names for one input.** §3's V10 row reads
*"**trailing bytes after the body** | E3/E4"*. §2.5a.1 gives E3 the name
`trailing_bytes` and E4 the name `length_mismatch`, and §1.3's E4 cell says E4
*"makes E3 checkable rather than aspirational"* — i.e. E4 **is** E3's mechanism.
So one input has two legal expected values. The fixture's `refuse.rule` is,
in §2.5a.1's own framing, a **three-way wire contract** — *"a name invented at
one of the three is a cross-language failure that looks like a behaviour
difference"*. Two names for one condition is that failure without anyone
inventing anything: Rust emits `trailing_bytes`, Go emits `length_mismatch`,
`tx_record_vectors.json` goes RED, and the fix someone reaches for is to change
a *behaviour* to match a *string*.

**Construction B — no name at all.** A `tx:` record is `tx:` plus up to **32,731
hex characters**. The likeliest real-world corruption of a 32,731-character
string is truncation — a clipboard limit, a terminal paste, a wrapped email. Feed
`me` a `tx:` record whose hex decodes to 40 bytes:

- `hex` passes (it is valid lowercase hex, even length);
- `magic` passes if the first four bytes survived;
- `version` and `form` pass;
- the parse then runs off the end reading `txid` — **and there is no rule for
  that.** E4's arithmetic cannot even be evaluated: `Σ(3 + len)` needs
  `n_fields`, which is at offset 70 of a 40-byte buffer.

None of the twenty-one names fits. One implementer reports `length_mismatch`,
another `body_len`, another `magic` (because "the header is wrong"), and the
operator gets a different answer per implementation for the most common failure
the format has. §2.5a.1's closing rule — *"A rule that gains a name later must be
added HERE first"* — means the first implementer to hit this is in breach
whatever they do.

**Why the plan permits it.** §2.5a.1 says the vocabulary was *"Derived by walking
§1.3, not invented"*. Walking §1.3 yields one name per E-number; it does not ask
whether every **input** maps to exactly one name, which is the property the
cross-language contract actually needs. E3/E4 collide because two rules describe
one condition; truncation falls through because no rule describes it.

**Confidence:** high on both constructions; both are decidable from §1.3 and
§2.5a.1 alone.

---

## [M1] The orphan message's obvious remedy is the wrong one, and there is no bound on how many orphans one payload can hold.

**Severity:** Minor
**Where:** E20 (line 358); W12 (lines 788, 892); §1.4a's cost 2.

The plausible first attempt is to pack `mt` chunk output on its own — the chunks
are the transaction; the `tx:` metadata record comes from a different invocation.
Then **every** chunk is an orphan. §1.5 aborts on the first, so the operator sees
*"record 0 is an orphan"*, whose obvious action is to delete record 0 — after
which record 1 is an orphan, 202 times, ending at an empty file that **packs at
exit 0** ([I1]). The correct action — *add the `tx:` metadata record this set
belongs to* — appears nowhere in the message the plan sketches. §1.4a's cost 2
worries in writing that *"W9 has to summarise a set rather than print 202 lines"*
and never asks the same question of the refusal path.

---

## [M2] R7 and the TTY refusal share a site and an exit code; only one has specified text, and the text at that site today names the two channels the operator did **not** use.

**Severity:** Minor
**Where:** §4 step 2; §4.2; §6.2's R7 and TTY rows.

Both refusals replace the same branch (`crates/me-cli/src/main.rs:1223-1225`) and
both exit **2**. §4.2 specifies the TTY message (*"naming both real inputs"*);
**nothing specifies R7's**. If the implementer reuses the shipped string, then
`cat tx.txt | me sysw pack` with an empty `tx.txt` prints:

> `me: no records: pass them on argv or with --in`

— which tells an operator who just used the *third* channel, the one step 2 adds,
to use one of the other two, and never says their pipe was empty. Two P1-added
refusals, one message, and E9's *"each with its own message"* standard is applied
to `magic`/`version`/`form` and to nothing else.

---

## [M3] §4.2 says a whitespace-only `--in` line *"lands in §1.5's refusal path via W11"*. Measured false: W11 is the `tx:` parse, and the line reaches `Unrecognised`.

**Severity:** Minor
**Where:** §4.2 (line ~1388).
**Measured:** cmd (2) — `me: record 1 … is not a form this container can place: …`

§4.2 correctly establishes that a single-space line survives `read_records`'s
`!l.is_empty()` filter, then routes it to a channel that cannot see it: W11 is
reached only by a record carrying `TX_PREFIX`. The operator's real outcome is
today's `Unrecognised` line, which is adequate for a blank-ish line — so this is
a wrong statement about the error path rather than a wrong outcome. It matters
because §4.2 is where an implementer looks to write step 2's test.

---

## [M4] Step 3 moves classification in front of the passphrase ceremony and the plan never says whether a classification **failure** there aborts before a passphrase is generated.

**Severity:** Minor
**Where:** §4 step 3; §4.3.

Content-based sealing requires classifying every record **before** deciding
whether to generate a passphrase. That is the first time in `run_sysw`'s Pack arm
that record validity is knowable ahead of the ceremony. The plan rules the
sealing *decision* in detail (§4.3's four numbered rules) and is silent on the
*failure* at that point. This is F-246's shape — deliberately not re-reported —
and the note is that step 3 is the phase's one free opportunity to close it or to
entrench it, and the plan does not choose. An implementer who classifies, decides,
generates, prints, and only then calls `pack` reproduces F-246 for every
transaction payload that also carries a secret record.

---

## [M5] W12's missing-chunk message is singular; a set missing six indices is unspecified.

**Severity:** Minor
**Where:** W12 (lines 788, 892); §6's W12 closure row.

The gate is *"a payload with chunk 7 of set `0x2dcf2` missing is refused naming
the set and the missing index"* — one index. An operator who packs the metadata
record and forgets the chunks entirely is missing indices 0..5, and whether they
are told *"chunk 0 is missing"* (six edit-and-retry rounds) or *"chunks 0–5 of
set `0x2dcf2` are missing"* (one) is left to the implementer. The gate passes
either way.

---

## Part 2 — did round 6's fourteen findings land?

| # | round 6's finding | verdict | evidence |
| --- | --- | --- | --- |
| **C1** | §2.2's CHUNKS decode chain owned by no site, detected by no vector | **FIXED** | **W15** added to §2.4 (line 789) with the full chain; **V28** added (line 1103) and marked *"the only vector that can go RED on W15"*; step 10 builds W15; §6 gains a W15 closure row. `grep -c W15` = 7, `grep -c V28` = 4; `plan-wiring-check.sh` exit 0. |
| **C2** | step 4 must build W14 but `expect` asserted container/process outcomes | **FIXED** | §3.3 gains the NORMATIVE r6-C2 block ruling `expect` **codec-level** — parsed fields on `pass`, rule name on `refuse` — and routing exit code / stderr / blob to `tests/sysw_cli.rs` at steps 6–7. W14's row scoped to match. *(See [I3]: V7's near-miss is the one vector the new ruling cannot express.)* |
| **I1** | W8's arm matches a `SyswError` variant no row declares | **FIXED** | W12 now adds **BOTH** `TxRecord(usize, TxRecordError)` **and** the set-level variant, and states the parse runs in `split`, not `classify` (line 788). *(See [C1]: the clause names W4 only, leaving the chunk half unproduced.)* |
| **I2** | step 3 reverses a shipped, tested default and never says so | **FIXED** | Step 3 now names `omitting_every_passphrase_flag_generates_one` (`sysw_cli.rs:121-131`), says it **goes RED**, quotes its doc comment, and states the rule reaches beyond transaction payloads. |
| **I3** | R2 never re-ruled for bare `mt1` after §1.4a | **FIXED** | Step 11 now refuses a bare `mt1` on argv at **exit 3** on R2's own grounds, and files spec §5's singular wording as a §6.3 correction. |
| **I4** | rule-name vocabulary: five of ~18 named | **FIXED** (as asked) | §2.5a.1 added, twenty-one names in a table, derived by walking §1.3, with the omissions named. *(See [C1] and [C2]: two names have no producer and the set-level five got no names at all.)* |
| **I5** | V8 has no construction clause; the natural build masks its RED test | **FIXED** | V8's row gains the NORMATIVE clause *"perturb ONLY the carried txid and leave the wtxid HONEST"*, with the reason (E17 would refuse it unaided). |
| **M1** | generator assigned to step 4 **and** step 9 | **FIXED** | Step 4 *"WRITES AND COMMITS"* it; step 9 *"USES"* it and says so; §3's blockquote now reads *"committed in §4 step 4 (r6-M1: step 9 until r5-I5 moved it)"*. `grep -c r6-M1` = 3. |
| **M2** | §6.1's cite row states two different citation counts | **FIXED** | The cell now carries one PASS figure (**91 of 108**) and explicitly rules the running figures *"deliberately NOT restated"*; `'90 → 98 → 107'` added to the sweep terms. Re-measured: 91/108, 17 dangling (9 + 8). |
| **M3** | step 1's *"only"* leaves the doc comment asserting 8191 | **FIXED** | Step 1 now says *"the DOC COMMENT at `:40-41` moves with it"* and defines what *"only"* means. |
| **M4** | §3.3's schema example is not an instance of any vector | **FIXED** | The example is now `V13-bad-magic` (`refuse: {rule: magic}`) and `V1-raw-roundtrip` (`pass: {form, txid, body_len}`) — both real rows. `grep -c r6-M4` = 0 because the fix needed no marker; verified by reading lines 1271 and 1276. |
| **M5** | *"27 rows"* counted plan rows, not fixture entries | **FIXED** | Step 4's cell now says *"27 plan ROWS — and (r6-M5) MORE than 27 fixture ENTRIES"*, with V13's three and the near-miss pairs called out. |
| **M6** | every `sysw_cli.rs` test passes records on argv; step 11 then refuses that | **FIXED** | §4 opens with a NORMATIVE blockquote: *"every test that packs a `tx:` or bare `mt1` record uses `--in` or stdin, NEVER argv"*, with the 30-of-31 measurement. |
| **M7** | the record codec has no named home | **FIXED** | A NORMATIVE blockquote after §2.4's table puts parse/serialise in `crates/me-cli/src/sysw/record.rs`, forced by `unhex_lower` being private at `:201`. |

**14 FIXED, 0 PARTIAL, 0 NOT FIXED, 0 WRONGLY FIXED.** Round 6's fold is the
cleanest of the cycle by this measure. Every Critical this round raises is in
territory round 6 did not ask about — two of them (C1, C2) sit *inside* the
machinery round 6's I1 and I4 asked for, in the half those findings did not
reach.

---

## Verdict

**2C / 5I / 5M.** **v9 is NOT GREEN.**

| # | title (abbreviated) |
| --- | --- |
| C1 | `chunk_charset` / `chunk_bch` have no producer, and W8's template calls a bare `mt1` record a `tx:` record — r3-C2 reintroduced |
| C2 | R17's and W15's refusals have no rule name and no message, while V27's RED-ness is defined as asserting R17's rule name |
| I1 | empty/blank-only `--in` packs a container at exit 0 while empty stdin becomes a NORMATIVE exit-2 refusal |
| I2 | the CHUNKS section ceiling (~354 chunks / ~14,160 B) is stated nowhere and its refusal prescribes a split E20 forbids |
| I3 | V7's near-miss is a container refusal, unexpressible in the codec fixture step 9 files it into, and has no rule name |
| I4 | `me sysw show` prints nothing and exits 0 for a record it cannot classify; W9 specifies only the pass case |
| I5 | the rule-name vocabulary is under-determined for length failures — truncation has no name, trailing bytes has two |
| M1 | the orphan message's obvious remedy inverts, and 202 orphans are unbounded |
| M2 | R7 and the TTY refusal collapse to one message at one site and one exit code |
| M3 | §4.2's *"lands in §1.5's refusal path via W11"* is measured false |
| M4 | step 3 never says whether a classification failure aborts before the passphrase is generated |
| M5 | W12's missing-chunk message is singular; six missing indices are unspecified |

**The shape of this round.** Six rounds asked whether the plan is correct,
whether facts propagate, and whether it transfers. The error surface had never
been walked, and it holds up **on the paths the plan designed** — §1.5's *"nothing
on stdout, nothing is written"* is true, measured; the twenty-one rule names are
distinct where they have producers; the four exit codes are consistent with their
siblings. It fails at the **edges of the two channels §2.5a built**: 2 of 21
per-record names have no producer, and 5 set-level failures have no names at all.
Both are round-6-C1's shape — machinery that is normatively described and bound
to nothing — one layer further out.

**A note on what this lens still has not asked.** Nothing here examines the
*device* side of a failure (P4's), nor what an operator sees when a plate is
already cut. Both are outside P1.
