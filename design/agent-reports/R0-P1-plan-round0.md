# R0 — `IMPLEMENTATION_PLAN_P1_me_container.md`, round 0

**Reviewer:** independent R0 agent. **Artifact:** `design/IMPLEMENTATION_PLAN_P1_me_container.md` (205 lines).
**Lens:** *construct a concrete failure this plan permits* — inputs and state, leading to a wrong outcome.
**Machine-checked before writing:** the txid byte order against `mt-codec`'s source and its pinned
corpus; `dSHA256` of both pinned raw bodies; an executed arbitrary-bytes round trip through
`mt_codec::encode`/`decode`; `me`'s dependency set; the callers of `decode_public_set` and
`mdmk_unconfirmed`; the record/section arithmetic.

---

## The txid byte order — RESOLVED, AND §1.1 IS WRONG

**`mt-codec` uses DISPLAY (byte-reversed) order.** Not deferred, not ambiguous — it is in the
function's name.

`/scratch/code/shibboleth/mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:17-27`

```rust
/// Top 20 bits of a txid **in its display form** — the content id (§10.13 c).
///
/// The display form is the byte-reversed one a user reads, and "which 20 bits,
/// from which end" is exactly where two implementations diverge silently. So
/// this takes the display string rather than raw bytes, and takes it as the
/// caller already has it.
pub fn content_id_from_txid_display(txid_hex: &str) -> Result<u32> {
    let head = txid_hex
        .get(..5)
        .ok_or(Error::InvalidStringLength(txid_hex.len()))?;
    u32::from_str_radix(head, 16).map_err(|_| Error::InvalidHrp(txid_hex.to_string()))
}
```

It is the **first five hex characters of the display string**. `pipeline.rs:54` is the only
producer of a `chunk_set_id`: `let set_id = content_id_from_txid_display(txid_display)?;`

The governing spec already ruled it, in the words the plan's §1.1 was afraid nobody had written
down — `design/SPEC_mt_v0_1.md:3546-3549`:

> **The top 20 bits of the txid in its standard display form** — the big-endian hex a user reads.
> Stated to that precision because *"which 20 bits, from which end"* is exactly where two
> implementers diverge silently, and the internal byte order is the reverse of the displayed one.

Verified against the pinned corpus (`crates/mt-codec/src/test_vectors/mt1_v1.json`), recomputed
here rather than read off:

| vector | pinned `set_id` | top-20 of **display** | top-20 of **internal** |
| --- | --- | --- | --- |
| even | `0x2dcf2` | `0x2dcf2` ✅ | `0x30f6e` ❌ |
| uneven | `0x3b426` | `0x3b426` ✅ | `0xb9623` ❌ |

**§1.1 does not agree.** It says, in bold, `NORMATIVE`: *"the `txid` field is the raw
`double-SHA256` result, INTERNAL byte order, unreversed."* That is the losing answer. See **C1**.

---

## [C1] §1.1's NORMATIVE byte order is the one that makes R15 refuse every honest record

**Severity:** Critical.
**Where:** plan §1.1; consumed by spec §3.6b's R15 table and §5's R15 row.

**The failure, concretely.** `mt encode --record --chunks` on the corpus's *even* transaction.
`mt-codec` stamps `chunk_set_id = 0x2dcf2` into all six chunk headers (measured above). §1.1 says
the record's 32-byte `txid` field is the internal order, so it holds
`30f6eb724cb0ee931e0a8b5905ff7380389da5a588c9581e4b04523d972bcf2d`. The device runs R15 exactly as
§3.6b writes it — *"carried txid's top 20 bits ≠ some chunk's `chunk_set_id`"* — and computes
`0x30f6e`. `0x30f6e ≠ 0x2dcf2`, so it **REFUSES a byte-perfect record**, and the operator is told
their payload is internally inconsistent. Every chunks record ever produced fails, on the machine,
after the container has been flashed.

**Why the plan permits it.** §1.1 states the wrong order as `NORMATIVE` in bold, then adds an
escape clause pointing the other way — *"If `mt-codec`'s own choice turns out to be the display
order, this field follows it"* — and leaves both standing. **The plan therefore contains two
contradictory instructions with no rule saying which wins**, so two implementers reading the same
section produce two different 32-byte fields, in a format the plan itself says Go must reproduce
byte-for-byte.

**And the deferral has no referent.** §1.1 says *"the answer is whatever `mt-codec` does"*, but
`mt-codec` **has no txid field**. Its only txid-shaped API is
`content_id_from_txid_display(txid_hex: &str)`, which takes a *string the caller already holds* and
never stores, hashes or orders 32 bytes. There is nothing in `mt-codec` for "this field follows it"
to follow. The question §1.1 defers is one the deferred-to crate does not answer, so V4 cannot
resolve it either — V4 would pin whatever `me`'s author guessed.

**Confidence:** High. Source read, spec ruling located, corpus recomputed.

---

## [C2] "the raw `double-SHA256` result" over a with-witness body is the **wtxid**, not the txid

**Severity:** Critical.
**Where:** plan §1.1 (`the raw double-SHA256 result`) against plan §1.3 (`RAW | the serialized
signed transaction, **with witness**`), and plan §2's RAW row.

**The failure, concretely.** The two statements are 40 lines apart in the same section. An
implementer joins them the obvious way and computes `double_sha256(body)`. Recomputed on both
pinned vectors:

| vector | `dSHA256(raw_hex)` reversed | vector `txid` | vector `wtxid` |
| --- | --- | --- | --- |
| even | `d5717c03…ed836f51` | `2dcf2b97…72ebf630` | `d5717c03…ed836f51` |
| uneven | `483003 98…d0e036f6` | `3b426e92…1b3462b9` | `483003 98…d0e036f6` |

It is the **wtxid**, both times, exactly. §2's RAW row then says *"its txid equals the carried
`txid` field"* — so an implementer who carries `dSHA256(body)` and an implementer who strips
witnesses before hashing disagree on all 32 bytes for every segwit transaction, which is every
transaction this project targets. Downstream, §3.5 puts *"the full 64-hex txid in 16 groups of 4"*
on the confirm screen; the operator compares it against `mt inspect`'s txid on the host, and it
will never match.

**Why the plan permits it.** §1.1 is the plan's own normative layout section and it never says
*over what*. The correct rule exists in the spec (`SPEC_engrave_transaction.md` §3.6a: *"A txid is
`double-SHA256` over the deserialised transaction with witnesses stripped"*) and the plan restates
it lossily in the one place the Go port will read.

**And V4 cannot catch it.** §3 requires *"a transaction whose txid and `chunk_set_id` are both
stated"* and says nothing about segwit. For a non-segwit transaction `txid == wtxid`, so a V4 built
from one **passes in both worlds** — the vector the plan calls *"the one that matters"* would be
blind to the second byte-order defect in the same field.

**Confidence:** High. Both values recomputed from the pinned corpus.

---

## [C3] The CHUNKS-form decode proves nothing. Arbitrary secret bytes pass every check in the plan

**Severity:** Critical.
**Where:** plan §2's decode table, CHUNKS row: *"the body's `mt1` strings reassemble as a chunk set
via `mt-codec`"*.

**The failure, concretely.** Executed, not argued — a scratch crate against the real
`mt-codec` path:

```
smuggled payload  = 030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc   (32 B of entropy)
chunks            = 1
  mt1pqqqqqqqqqqqv9pzxqlyckngw6zf9g9whn9d3eh4qvg37tfmf9tk2uup37w6hwq6da3mt7gka6l3
decoded ok        = 030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc
round-trips       = true
carried set_id    = 0x00000
```

`mt_codec::encode(&secret_bytes, "0000…deadbeef")` accepts **any** byte string and **any** txid
string — it never parses a transaction, never computes a hash, never checks the two against each
other. `mt_codec::decode` verifies BCH, header consistency and chunk completeness, then
concatenates. There is no `bitcoin::` symbol anywhere in `mt-codec` (`grep -rn "bitcoin::"
crates/mt-codec/` → 0 hits, src and tests).

So the attacker's record is: `tx:` ‖ hex of `MTX1` ‖ `0x01` ‖ `form = 0x02` ‖ 32 zero-ish bytes
matching the invented txid ‖ `n_fields = 0` ‖ `body_len` ‖ that one `mt1` string. It decodes. Its
carried txid's top 20 bits are `0x00000`, which **matches** the chunk header, so R15 passes too —
the attacker picked both. `me` admits it to the PUBLIC section, and `picotool save` reaches the
seed **with no passphrase**, which is the precise outcome plan §2 opens by naming
(*"`tx:<hex of a seed>` is exactly that smuggling channel"*).

**Why the plan permits it.** The plan reasons by analogy to `md1`/`mk1` without checking that the
analogy holds. It does not: `md_codec::reassemble` yields a **descriptor** — a framed, versioned,
TLV-structured object — which is why EPD §6.3's own worked example ends *"smuggled entropy →
`md_codec::reassemble(&[s])` → 'wire-format version mismatch'"*. `me`'s source states the rule the
plan needed: `crates/me-cli/src/sysw/record.rs:154-155` — *"The real decoders are the arbiter —
semantics-bound, per §12.6. **A BCH verifier is not one: that is the whole point of the rule.**"*
`mt_codec::decode` is a BCH verifier and a reassembler. It carries no semantics, because `mt1`'s
payload is raw transaction bytes with no envelope of its own.

**Confidence:** High. Executed.

---

## [C4] TLV field order is undefined, so two conforming implementations emit different bytes

**Severity:** Critical.
**Where:** plan §1 layout (`fields  n_fields x { tag:u8, len:u16 BE, value:len bytes }`), §1.2,
and §3's V2 (*"pins TLV order"*).

**The failure, concretely.** A transaction with all three legend fields. Implementer A emits
`0x01, 0x02, 0x03`; implementer B emits `0x03, 0x02, 0x01`. Both records decode to identical
semantics. Both are conformant to every sentence in §1 and §1.2, which state a tag table and an
absence rule and **no ordering requirement at all**. The bytes differ, so the hex differs, so the
record differs, so EPD §6.6's public-data hash over the section differs — and §3.2's identity
digest is the operator's only integrity check on the unsealed path.

**Why the plan permits it.** §3 asserts the vectors *"must therefore pin every choice a second
implementer could make differently"* and then relies on V2 to do it. **A vector is an instance, not
a rule.** V2 shows one ordering of all three tags; it says nothing about the ordering of the
*subsets* — `{0x01, 0x03}`, `{0x02, 0x03}` — which is exactly what V5 ("absent optional field")
creates. An implementer who reads §1's prose, finds no ordering rule, and emits ascending-by-tag
is right by the text and fails the vector; one who emits in the order `mt encode` happened to
compute them is wrong by nothing written down. The plan's own §0 names this shape: *"A format
defined in code first is a format nobody reviewed."*

**Confidence:** High.

---

## [C5] Nothing requires the record to end where the body ends — trailing bytes are a smuggling channel that survives a perfect decode

**Severity:** Critical.
**Where:** plan §1 layout; plan §2's decode requirement.

**The failure, concretely.** Take a **genuine, fully valid** signed transaction. Build the record
exactly as §1 says: `MTX1 ‖ 0x01 ‖ 0x01 ‖ <correct txid> ‖ n_fields ‖ fields ‖ body_len ‖ body`.
Then append 32 bytes of seed entropy **after** the body and hex the whole thing.

- It is lowercase hex → R1 passes.
- Magic, version, form are correct → the byte-0 check passes.
- `n_fields` TLVs parse → the legend passes.
- `body[..body_len]` deserialises as a Bitcoin transaction and its txid equals the carried field →
  §2's RAW decode passes, and V8's consistency check passes.

The record is admitted to the PUBLIC section carrying 32 bytes of key material that no check in the
plan ever looks at. `picotool save`, no passphrase.

**Why the plan permits it.** The layout is expressed as offsets and lengths, and every reader
walks it by length: read 39, read `n_fields` TLVs, read 4, read `body_len`. **Nothing says the
cursor must be at the end of the decoded hex when that finishes.** The plan states a refusal for
exactly one malformation — *"Unknown tags are REFUSED, not skipped"* — and none for the residue.
The same absence bites the other way: with `body_len` **greater** than the remaining bytes there is
no stated verdict either (see I2).

This is the identical shape §2 was written to close, reached by a different door: §2 checks that
what the record *claims* is really there, and never that nothing *else* is.

**Confidence:** High.

---

## [I1] Duplicate tags have no verdict, so `me` and the device can read different legends off one record

**Severity:** Important. **Where:** plan §1.2.

**Concretely.** A record with two `0x01` TLVs: `TO: my cold wallet` and `TO: bc1q…attacker`.
Nothing in §1.2 refuses it — the stated refusal covers *unknown* tags only. A decoder that inserts
into a map (the natural Go port) keeps the **last**; one that takes the first match (the natural
Rust `iter().find()`) keeps the **first**. §3.4 renders the `TO` label in the asserted column and
the operator engraves it onto steel. Two implementations, two legends, one record, and no vector
covers it (V6 is unknown-tag; V2 is all-distinct).

**Confidence:** High.

## [I2] `body_len` is a u32 with no bound stated before it is used, against EPD §6.2's explicit precedent

**Severity:** Important. **Where:** plan §1 layout (`body_len u32 BE`).

**Concretely.** An 89-character record: `tx:` ‖ hex of `MTX1 ‖ 01 ‖ 01 ‖ <32 bytes> ‖ 00 ‖
FFFFFFFF` and nothing after it. A decoder doing `Vec::with_capacity(body_len)` attempts a 4 GiB
allocation; one doing `&data[pos..pos + body_len]` panics on a slice bound (Rust) or index (Go) —
on the **device**, in firmware, from a record read out of flash or off an NFC tag. EPD §6.2 is
titled *"Parameter bounds — checked BEFORE any allocation or KDF work"* and the container this plan
extends already obeys it; §1 introduces a second length field and inherits none of that reasoning.
No vector covers it.

**Confidence:** High.

## [I3] Fixed-width tags carry a variable `len`, and a zero-length value for a known tag has no verdict — so V5 pins nothing it claims to

**Severity:** Important. **Where:** plan §1.2 table and §3's V5.

**Concretely.** §1.2 says tag `0x02` is *"**u64 satoshis**, big-endian, 8 bytes"* and `0x03` is
*"4 bytes"*, but the layout gives every TLV its own `len`. A record with `tag=0x02, len=2, value=0x03E8`:
one implementer refuses (len ≠ 8), one reads the two bytes as a big-endian u64 = 1000 sat, one
left-pads. Nothing says which. Same for `tag=0x01, len=0`: §1.2 says *"There is no 'empty' encoding
and no sentinel"*, which is a **description of the encoder**, not a refusal binding the decoder.

So V5 — *"absent optional field | that absence is *omission*, not an empty TLV"* — is a positive
vector with no negative behind it. A decoder that treats `len=0` as "absent" passes V5 and is
conformant to every sentence written. The vector passes in both worlds, which is the failure mode
§3 exists to prevent.

**Confidence:** High.

## [I4] A bad `magic`, `version` or `form` has no stated verdict

**Severity:** Important. **Where:** plan §1 layout.

**Concretely.** `version = 0x02`, or `form = 0x03`, or `magic = "MTX2"`. §1 justifies the magic
(*"A hex-valid body of the wrong shape is caught at byte 0"*) but never says caught *how* — refused,
or skipped-and-continue. The only refusal §1 states is for unknown tags, and it argues the case in
general terms (*"Skipping is how a format silently diverges between two implementations"*) without
extending it to the three discriminators that come first. A future `version = 0x02` reader that
skips unknown versions and a `me` that refuses them are the same divergence one field earlier, and
no vector covers any of the three.

**Confidence:** High.

## [I5] `me` has no Bitcoin dependency, the plan never says it gains one, and `mt-codec` will not supply it

**Severity:** Important. **Where:** plan §2's RAW decode row; plan §5.

**Concretely.** `crates/me-cli/Cargo.toml` lists `md-codec`, `mk-codec`, `ms-codec`, `clap`,
`zeroize`, `serde`, `serde_json`, `aes-gcm`, `pbkdf2`, `sha2`, `bip39`, `rand`, `rpassword`. **No
`bitcoin`, no `btcd`, no transaction parser.** §2 requires `me` to prove *"the body deserialises as
a Bitcoin transaction, and its txid equals the carried `txid` field"*, which means segwit-aware
deserialisation plus a witness-stripped re-serialisation plus `dSHA256`. §5 — the section titled
*"THE ORDERING CONSTRAINT NOBODY CAN SKIP"* — names only `mt-codec`'s publication.

And the obvious hope does not work: `mt-codec` declares `bitcoin = "0.32"` in its manifest and
**uses it nowhere** (`grep -rn "bitcoin::" crates/mt-codec/` → 0 hits). Depending on `mt-codec`
gives `me` the compile cost of `bitcoin` and none of its API. So P1's largest new dependency
decision — one that lands in a crate `me` publishes — is not in the plan at all.

**Confidence:** High.

## [I6] §5's publication ordering contradicts itself, and §6 forecloses the way out

**Severity:** Important. **Where:** plan §5 and §6.

**Concretely.** §5 sentence 1: *"`mt-codec` 0.1.0 **must be published to crates.io before step 5
can build**."* §5 sentence 4: *"So it happens **after** this plan is GREEN and **after V4 has
pinned the byte order**."* Step 5 **is** V4 (§4's table). Each waits on the other. The normal escape
— develop against a path dependency, swap to the published version at close — is exactly what §6
forbids without saying it is a two-stage thing: *"`me` depends on the **pinned published version**,
not a path or a git URL."* As written the sequence has no entry point, and `cargo publish` is the
one step in this plan that cannot be undone.

**Confidence:** Medium-High. The intent is guessable; the written ordering is not executable.

## [I7] V7's boundary is not constructible, and §1's framing silently falsifies §2.3's headline number

**Severity:** Important. **Where:** plan §3's V7 (*"body at `MaxSectionLen` boundary | the cap,
exactly"*) and spec §2.3.

**Concretely.** §1's fixed framing is `4 + 1 + 1 + 32 + 1 = 39` bytes, plus 4 for `body_len`, = 43
bytes before any TLV. The record is `tx:` plus **hex**, so a record carrying an `N`-byte body with
`F` bytes of TLVs occupies `3 + 2 × (43 + F + N)` characters of the section:

```
3 + 2 × (43 + F + N) ≤ 32,734   ⇒   N ≤ 16,322 − F
```

So a body of `MaxSectionLen` bytes produces a **65,557-character record — twice the cap**. V7 asks
for a vector that cannot exist, and "the cap, exactly" names three different quantities (body,
record, section) without choosing.

The same arithmetic falsifies text the plan never touches. Spec §2.3 says the raise *"buys a
**16,367-byte** raw transaction"* — that is `32,734 / 2`, computed before §1 existed and ignoring
both the `tx:` prefix and the 43 bytes of framing §1 has since introduced. The true single-record
ceiling is **16,322 − F**, short by 45 bytes plus the legend. §2.3's table row *"5/2 | 4,080 B | …
✅ (**raw-only at 8191, by 31 chars**)"* goes the same way: 8,160 hex characters fit 8191 by 31, but
`3 + 2 × (43 + 4,080) = 8,249` does not. Neither is corrected in the plan, and §6's closure list
does not mention them.

(§4.1a's 14,560 B encodeable ceiling binds first in practice, so this is a defect in what V7 *pins*
rather than a live capacity loss — but V7 is the vector whose whole job is to pin the cap.)

**Confidence:** High. Arithmetic shown.

## [I8] The §6.3 precedent the plan cites is the *other* container's, and this one does not refuse

**Severity:** Important. **Where:** plan §2 (*"`me` satisfies this today by calling
`md_codec::reassemble` and `mk_codec::decode`"*).

**Concretely.** Two containers, two mechanisms, and P1 is changing the one that does not refuse:

| container | call site | on decode failure |
| --- | --- | --- |
| `seal` (`me seal pack`) | `crates/me-cli/src/seal/mod.rs:171` → `decode_public_set` | **refuses** — `RecordError::UndecodableSet` |
| `sysw` (`me sysw pack`) | `crates/me-cli/src/main.rs:1141` → `mdmk_unconfirmed` | **packs anyway** |

`me`'s own doc comment at `main.rs:1130-1132` is explicit: *"`[mdmk-decode]` (§12.6) at pack time:
one line per unconfirmed record, **then the container is built anyway**"*, and the operator-facing
line is *"an md1/mk1 this tool could not decode; the device will treat it as a SECRET"*.
`crates/me-cli/src/sysw/mod.rs:251-266` (`split`) classifies and never decodes.

So the plan's `NORMATIVE: me DECODES a tx: record before admitting it to the public section` lands
in a container whose established handling of an undecodable record is *warn and downgrade*. The
plan never says which mechanism `tx:` gets, and "the device will treat it as a SECRET" has no
defined meaning for a transaction record. Under the `sysw` precedent the anti-smuggling gate is a
line on stderr — which, combined with **C3**, is no gate at all.

**Confidence:** High. Call sites resolved.

## [I9] Content-based sealing collides with four existing passphrase flags and resolves none of them

**Severity:** Important. **Where:** plan §4 step 3; spec §2.4.

**Concretely.** `me sysw pack` today has four passphrase modes
(`crates/me-cli/src/main.rs:177-189`, `887-955`): `--passphrase-words N`, `--passphrase-ask`,
`--no-passphrase`, and a default that **generates** one and seals. Step 3's test is *"a payload with
no `IsSecret()` record packs **unsealed**; one with any packs **sealed**"* — with no invocation
named, so it cannot be written until someone decides:

- `--no-passphrase` **with** a mnemonic: seal (against an explicit instruction, using what
  passphrase?), or keep today's behaviour? Today's behaviour is
  `crates/me-cli/src/sysw/mod.rs:186-193`, which **moves the secret records into the cleartext
  public section**.
- `--passphrase-ask` with a transaction-only payload: prompt and discard, or seal and contradict
  the rule?
- `--passphrase-words 12` with a transaction-only payload: print words the operator is told to
  write down, for a container that will not use them?

Whichever way each goes, one of them is a silent confidentiality change to a shipped command, and
none is in the plan. §2.4's *"It MUST say which way it went, and why, on stderr, every time"*
makes it worse, not better: the stderr line cannot be written without the rule.

**Confidence:** High.

## [I10] The stdin path turns the most likely first invocation into a silent hang, and R7's test passes anyway

**Severity:** Important. **Where:** plan §4 step 2.

**Concretely.** Today `me sysw pack` with no argv and no `--in` returns immediately
(`crates/me-cli/src/main.rs:1223-1225`): `no records: pass them on argv or with --in`, exit 2 — the
measurement spec §1.1 quotes. Step 2 replaces that fallback with a stdin read and specifies exactly
one new behaviour: *"empty stdin is refused (R7)"*. It says nothing about **a TTY**. An operator who
types `me sysw pack` to see what it wants now gets a process that blocks forever with no prompt and
no output: a clear error becomes an apparent hang.

R7's test does not see this. `printf '' | me sysw pack` closes stdin immediately and refuses
correctly, in both the guarded and unguarded implementations. This repo has the finding on file
already — *"stdin doesn't mean from the command line?"*, the operator's own first move — and
`me` already owns the `S_ISCHR` machinery §2.5 uses for stdout.

**Confidence:** High.

## [I11] Step 1's test names a symbol that is not in this repo, and `me` has two constants called `MAX_SECTION_LEN`

**Severity:** Important. **Where:** plan §4 step 1.

**Concretely.** Step 1 is *"`MaxSectionLen` is 32,734 and **`boundBlob`'s no-wrap argument still
holds**"*, with *"raise the constant"* singular. `grep -rn boundBlob crates/` → **0 hits**;
`boundBlob` is Go, in the fork's `seal` package, and P3 owns that port (§6). The test as written
cannot be authored in P1's crate.

And "the constant" is ambiguous: `crates/me-cli/src/sysw/wire.rs:42` and
`crates/me-cli/src/seal/wire.rs:21` **both** define `MAX_SECTION_LEN = 8191`. Spec §2.3 rules that
*"`seal` is untouched, keeps its own 8191, stays frozen"* — so raising the wrong one unfreezes a
module the operator froze by decision, and P1's first TDD step is the moment that mistake is
cheapest to make and hardest to see (both files have the same constant name and near-identical
comments naming 8191).

**Confidence:** High.

## [I12] The vectors are produced by the implementation they judge — the constellation already ruled against this

**Severity:** Important. **Where:** plan §3, §6.

**Concretely.** §3 says *"Rust-primary means these vectors are what the Go port is judged against"*
and never says where V1–V8 come from. The default — generate them by running `me` — makes `me`
unfalsifiable by its own corpus: a `me` that implements the wrong byte order (see **C1**) emits a
V4 stating the wrong byte order, and the Go port then conforms to the defect, byte-for-byte, with
every gate green.

The sibling crate states the rule in its own `lib.rs`
(`/scratch/code/shibboleth/mnemonic-transaction/crates/mt-codec/src/lib.rs:9-14`): *"The vectors in
`src/test_vectors/mt1_v1.json` were **not produced by this crate** … A vector this crate generated
could not falsify this crate — that is precisely how a wrong NUMS constant would launder itself
into looking correct."* The plan neither adopts that discipline nor says why P1 is exempt, and §6's
closure list requires only that *"V4's byte order is stated in the vector file itself"* — which a
self-generated vector satisfies while stating the wrong order.

**Confidence:** High.

## [I13] R15 has no negative vector, and §5 requires one

**Severity:** Important. **Where:** plan §3 (V4, V8) against spec §5.

**Concretely.** V8 is *"RAW whose carried txid ≠ the body's txid"* — the RAW form only. The CHUNKS
form's equivalent, R15 (*"a chunks record whose carried txid's top 20 bits match no chunk's
`chunk_set_id`"*), has only V4, which is a **positive** vector: it states a txid and a matching
`chunk_set_id`. Nothing constructs the mismatch. Spec §5 opens with *"Every refusal gets a test,
and every refusal test must go RED when its check is removed"*, and plan §6 restates it as a
closure condition. Delete R15's comparison entirely and V1–V8 stay green.

This is the vector that would have caught **C1** on the first run.

**Confidence:** High.

---

## [M1] `mt-codec 0.1.0` would publish an unused `bitcoin = "0.32"`, irreversibly
`crates/mt-codec/Cargo.toml:19` declares it; `grep -rn "bitcoin::" crates/mt-codec/` → 0 hits.
§5 correctly notes a published version *"can be yanked but never replaced"*; the dead dependency
would be in that immutable artifact, and would pull `bitcoin` into every downstream build of `me`.

## [M2] §1 cites Go for a rule P1 implements in Rust
*"the reserved-prefix rule, `sysw/record.go:41-51`"* — that range is `ErrBadHex` plus `DecodeBody`'s
doc comment; the prefix constants are at `sysw/record.go:13-14`. The Rust site P1 actually edits is
`crates/me-cli/src/sysw/record.rs:24-28`. (`sysw/open.go:74`, cited in §1.3, is correct —
`return strings.Split(string(b), "\n"), nil`.)

## [M3] Nothing says `ClassTransaction` must not be `is_secret()`
`crates/me-cli/src/sysw/record.rs:51-55` is the predicate; getting the new variant wrong there seals
every transaction payload — the exact outcome §2.4 exists to prevent, and the one thing content-based
sealing cannot survive.

## [M4] Three record inputs, no precedence rule
`--in` currently wins over argv silently (`main.rs:1215-1226`), and `--in` filters blank lines.
Step 2 adds a third source without saying what `me sysw pack rec1 --in f.txt < g.txt` does, or
whether stdin filters blanks the way `--in` does.

## [M5] Tag `0x01`'s value is "UTF-8" with a u16 length and no verdict on either
No bound (a 65,535-byte `TO` label is expressible) and no stated behaviour when the bytes are not
valid UTF-8 — refuse, or render lossily onto steel.

---

## Verdict

**5 Critical / 13 Important / 5 Minor. NOT GREEN. No code.**

The two that bind hardest: **C1** (the byte order is already ruled, and §1.1 states the losing
answer as normative while an escape clause states the winning one — two implementers, two fields)
and **C3** (`mt_codec::decode` is a BCH verifier, so the CHUNKS row of the anti-smuggling table
admits the exact payload §2 opens by naming; demonstrated by execution).

**The byte order is RESOLVED and needs no vector to settle it:** `mt-codec` uses **display
(reversed) order**, at
`/scratch/code/shibboleth/mnemonic-transaction/crates/mt-codec/src/string_layer/pipeline.rs:17-27`,
ruled at `design/SPEC_mt_v0_1.md:3546-3549`, and confirmed against both pinned corpus vectors.

### What I did NOT examine

- P2–P6, the device, the plate, the fork's Go implementation beyond the three lines §1 cites.
- The BCH/bech32 correctness of `mt-codec` itself, and the correctness of `mt1_v1.json` as a
  corpus (I treated its `txid`/`set_id`/`wtxid` fields as ground truth after confirming the
  three are mutually consistent under `dSHA256`).
- The `bitcoin 0.32` API surface — I established that `me` lacks a parser and that `mt-codec`
  does not export one, not which crate or version P1 should adopt.
- Whether `me sysw pack`'s existing tests would break under §2.4's sealing change; I read the
  flag definitions and the pack handler, not the CLI test suite.
- Style, wording, and anything in `design/agent-reports/`.
- I did not build `me` or run its suite; nothing in this report depends on doing so.
