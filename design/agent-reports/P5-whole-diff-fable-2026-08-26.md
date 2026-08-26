# P5 whole-diff adversarial review — engrave a transaction (pre-tag gate)

**Reviewer:** fable (P5 gate agent), 2026-08-26
**Scope reviewed:**
- `mnemonic-engrave` `master...ship/tx-engraving` (tip 2ec2e4a)
- `mnemonic-transaction` `main...ship/tx-engraving`
- `seedhammer` `a91df84..ship/tx-engraving` (tip d305713)

**The one question:** is there a defect in this diff that would cause an operator to
engrave a WRONG or UNRECOVERABLE artifact, or to leak bearer material, that the
phase gates and the operator walk did not catch?

**Method note.** All three code diffs were read in full at the branch tips (the
seedhammer worktree was checked out 5 commits behind `ship/tx-engraving`; every Go
file cited here was read via `git show ship/tx-engraving:`). The split-loop
arithmetic in finding 4 was verified by executing a replica of the loop, not by
inspection. The 2026-08-25c ruling ("two transactions sharing a txid: engrave both
→ operator re-ruled: the code DROPS, leave it") was located and honoured — the
txid-keyed merge at `gui/transaction.go:449` is **not** reported, per the ruling's
own "read this paragraph and move on".

---

## Findings

### I-1 (Important) — a complete mt1 set on argv is accepted; the identical bytes as a `tx:` record are refused for exactly that channel

**File:** `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/main.rs:1902-1948`
(`read_records`; the argv loop at :1933 checks only `TX_PREFIX`).

**Defect:** the R2/G-P3.5 argv gate refuses a `tx:` record on argv because "argv is
public: /proc, `ps` and your shell history all keep a copy" — but the *same
transaction*, encoded as its six `mt1` chunk strings, passes through the same argv
channel with no refusal and no warning. An mt1 set is the complete bearer
instrument (`mt decode` on the six strings yields broadcastable hex); a single
chunk already carries real transaction bytes.

**Concrete failure scenario:** `me sysw pack $(cat strings.txt) --no-passphrase
--out p.bin` — the natural shell splat of `mt encode`'s output, using the CLI's
own first-documented channel ("Records, on argv"). Exit 0, container written, and
the complete signed transaction is now in `~/.local/share/fish/fish_history` and
was visible in `/proc/<pid>/cmdline` to every user on the box. The identical
material as `tx:<hex>` on argv exits 3 with the bearer refusal.

**Why the tests do not catch it:** `tests/sysw_cli.rs` asserts the argv refusal
for the `tx:` class (G-P3.5) and asserts mt1 records pack from stdin/`--in`; no
test asks whether mt1-on-argv *should* pack. Nothing in `FOLLOWUPS.md` or the
journey rules on it — the only argv discussions are F-102 (`me seal` **warns**
when argv carries seed material — the house precedent this diff's new class
skipped) and G-P3.5 (tx: only).

**Fix shape:** extend the same loop to refuse (or at minimum F-102-style warn on)
argv records classified `Class::Mt`. One clause, same message family.

---

### I-2 (Important) — a same-csid group with conflicting duplicate indices is silently thinned by `orderByIndex`; the engraved subset can later decode-and-CONFIRM under a legend that says the opposite

**Files:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:476-495`
(`orderByIndex`, last-wins at :484), consumed by `payloadTransactions` (:424) and
`txGather.offer` (:570).

**Defect:** when one chunk_set_id group contains two different payloads for the
same index, `mt.Decode` correctly fails (`errAmbiguousChunk` /
`errCountMismatch`), the candidate is offered unconfirmed with a substituted
legend — and then `orderByIndex` builds the TEXT-plate content by `byIdx[h.ChunkIndex] = s`,
so **all but the last string per index are silently dropped from the steel**. The
review screen shows the deduped count; nothing names the dropped strings. This
contradicts what the host just promised on stderr for this exact case
(`me-cli/src/main.rs` `report_unconfirmed`: *"every mt1 string is independently
valid, so the strings you have are worth cutting"*) — the device does not cut the
strings you have; it cuts a payload-order-dependent subset.

**Concrete failure scenario (no collision luck required):** the operator signs the
same PSBT twice (a re-run of the ceremony — new signature nonce, different witness
bytes, **identical txid**, therefore identical csid, and with equal-length DER
signatures — roughly a coin flip — identical chunk count and lengths). They run
`mt encode` on both and pack strings from both runs, interleaved, into one
payload. Host warns "duplicates that disagree" and packs (ruling 2026-08-25b).
Device offers the set unconfirmed with legend `INCOMPLETE - DOES NOT DECODE -
RE-ENCODE PAYLOAD` and engraves the last-wins mix: chunk 0 from run 1, chunk 1
from run 2. At recovery, that engraved set is a complete, consistent set that
**decodes and passes the txid binding** (the splice differs only in witness bytes,
which the txid ignores) and passes the signature predicate (witness present) — so
it CONFIRMS, while its spliced signature is cryptographically invalid and the
transaction can never be broadcast. The plate's permanent legend ("DOES NOT
DECODE") is false in both directions: the set does decode, and what it decodes to
is not the operator's transaction. Meanwhile the two run-1 strings that would have
let a recoverer reassemble the real transaction were never cut.

If the operator instead packs the runs contiguously, last-wins keeps a coherent
run-2 set (a good plate under a false scary legend) — the drop of run-1 strings is
common to both orderings.

**Adjacent ruling, distinguished:** 2026-08-25c retired the txid-keyed *merge*
drop on "odds are low" grounds. This site is different: no low-odds event is
needed (same tx re-signed has the same csid by construction), the ambiguity is
*detected* and then the information discarded, and the drop breaks the loudness
2026-08-25b made normative. The operator may still choose to extend 25c here —
but it should be a ruling, not an accident of `map` assignment.

**Why the tests do not catch it:** `TestTextPlatesKeepIndexOrder` feeds
`orderByIndex` six distinct indices; no test in `gui/` constructs a same-csid
group with conflicting duplicates, and the mt-package ambiguity tests stop at
`Decode` returning an error — nobody asks what the GUI then engraves.

**Fix shape:** when `mt.Decode` fails with the ambiguous/mismatch errors, engrave
**all** strings in payload order (they are each independently valid — the 25b
rationale), or refuse the dedup and name the colliding indices on the review
screen. Either is a few lines in `payloadTransactions`/`offer`.

---

### M-1 (Minor) — `txqr.EncodeSet` has a latent slice-bounds panic in the split loop; the comment guarding `qrCeilingBytes` against it is false

**File:** `/scratch/code/shibboleth/seedhammer/txqr/txqr.go:80-90` (`per :=
(len(data)+k-1)/k`, `lo := i*per`); false comment at
`gui/transaction.go:1128-1130` ("EncodeSet REFUSES a payload it cannot split into
k non-empty parts" — it refuses only `k > len(data)`).

**Defect:** for `(k-1)*ceil(len/k) > len(data)` the loop computes `lo > hi`
(`data[lo:hi]` with lo > len) and panics. Verified by executing the loop's
arithmetic: `len=113, k=16` → `data[120:113]`; `len=60, k=14` → `data[65:60]`;
`len=17..29, k=16` all panic. 113 bytes is exactly the corpus stripped
transaction — a real artifact shape.

**Concrete failure scenario:** none reachable with shipped geometry, and I
downgrade it accordingly. `planTransactionQRPlates` reaches high `k` only when
low `k` does not fit, which for transactions small enough to trigger the bound
(len < ~240) cannot happen on an 85 mm plate; `qrCeilingBytes`' binary search
only evaluates panic-range sizes if `fits(32)` is false, i.e. absurd
`engrave.Params`. On firmware a panic is a reboot during *planning* (before any
cut), so no wrong plate. It is one `engrave.Params` change away from being
reachable, and it is a public API other callers may reach with small payloads.

**Why the tests do not catch it:** `capgate_test.go` and `txqr_test.go` exercise
realistic payload sizes; nothing sweeps the small-len/high-k corner.

**Fix shape:** clamp `lo = min(lo, len(data))`, or refuse `k > len(data)` with
the split arithmetic actually implied (`per*(k-1) >= len`), and correct the
comment at `gui/transaction.go:1128`.

---

### M-2 (Minor) — host and device diagnose the forged-AND-stripped set in opposite orders; the host's remedy is wrong for that case

**Files:** `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/sysw/mt.rs:172-183`
(`diagnose` checks `UnsignedInputs` **before** `TxidDoesNotBind`);
`/scratch/code/shibboleth/seedhammer/mt/mt.go:250-259` (`Decode` checks the txid
binding **before** the unsigned predicate).

**Defect:** for a complete set whose bytes parse, carry an unsigned input, AND
fail the 20-bit binding, the host reports UNSIGNED ("Re-export the FINALIZED
transaction from your signer") while the device classifies it DOES-NOT-DECODE.
The host's remedy sends the operator to re-sign a transaction the strings were
never made from; the binding failure — the forgery/mismatch signal, strictly
stronger evidence — is masked.

**Concrete failure scenario:** a stripped transaction encoded under a foreign set
id (one `mt_codec::pipeline::encode(&stripped, "00000...")` call away). Host says
"unsigned, re-export"; device legend says "does not decode". Both loud, no wrong
plate — hence Minor — but it is a cross-language divergence of exactly the class
the brief flags, and one assertion (`set_problems(...)[0].2` on that input) pins
whichever order is ruled correct.

**Why the tests do not catch it:** `every_way_a_set_can_fail_is_named_separately`
constructs each failure alone, never two at once; the Go tests likewise.

---

### M-3 (Minor) — the UNCONFIRMED SET review screen states "does NOT reassemble" and "the set is not complete" for sets that are complete and do reassemble

**File:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:663-679`
(`transactionReviewLines`, `!c.confirmed` branch).

**Defect:** one blanket paragraph serves all five non-confirm reasons. For a
complete set that reassembles but fails the txid binding, and for a complete set
carrying an unsigned transaction (`ErrUnsignedInputs`), the screen's two claims
are both false — while the *legend* (`substitutionFor`) correctly distinguishes
three cases. The operator confirming the cut reads "the set is not complete" and
goes hunting for a string that does not exist, on the same screen that promises a
legend saying something else.

**Failure scenario:** stripped-set case from `me sysw pack` of
`pipeline::encode(&stripped, &its_txid)` — screen says "does NOT reassemble",
legend says `UNSIGNED INPUT - CANNOT BE BROADCAST`. Contradictory words above an
irreversible action; no wrong plate (the legend on steel is the right one).

**Why tests miss it:** `transaction_messages_test.go` asserts the words are
present, not that they are true of the candidate shown.

---

### M-4 (Minor) — the post-cut screen for an UNSIGNED `tx:` record predicts "expect it to fail there too"; `mt inspect` will succeed

**File:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:830-841`
(`transactionPostCutLines`, `c.subst != ""` branch; also "This set" for a
candidate that is not a set).

**Defect:** the subst branch was written for unconfirmed *sets* (whose `mt
decode` genuinely fails). An unsigned `tx:` record admitted via
`--allow-unsigned-inputs` engraves QR plates with `c.subst != ""` — and its
post-cut instruction tells the operator the scan-back check should fail. It will
not: the scanned bytes parse, and `mt inspect` prints the expected txid and a
clean report. The operator is taught either to distrust a passing check or to
wave off a "failure" that never comes — and the txid-comparison line (the actual
verify step) is suppressed on exactly this path.

**Failure scenario:** P2A-anchor-style transaction packed with
`--allow-unsigned-inputs`, QR plates cut, operator scans and runs `mt inspect`,
gets a clean report contradicting the device's parting words. The plate itself is
right (legend `UNSIGNED INPUT - CANNOT BE BROADCAST`); only the instruction is
wrong. Minor.

**Why tests miss it:** `transaction_instruction_test.go` asserts wording per
branch, not the truth of the prediction against `mt inspect`'s behavior.

---

### M-5 (Minor) — `mt inspect`'s raw-transaction discriminator misroutes base64 PSBTs that contain the substring "mt1"

**File:** `/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/crates/mt-cli/src/main.rs:1386`
(`if !text.trim().is_empty() && !lower.contains("mt1")`).

**Defect:** the branch comment argues hex contains no `m`/`t` and a base64 PSBT
"begins `cHNidP8`" — but the test is `contains`, case-folded, over the whole
input, and the base64 alphabet contains `m`, `M`, `t`, `T`, `1`. A base64 PSBT
with "mt1"/"Mt1"/"MT1" anywhere in its body is routed to the *strings* path and
refused with a strings-shaped error instead of being inspected. Expected
frequency ≈ 4/64³ per character ≈ 6 % for a 4,000-character PSBT — a real
fraction of large multisig PSBTs.

**Failure scenario:** recoverer follows the device's post-cut instruction chain,
later runs `mt inspect` on the wallet's base64 PSBT for comparison; on an
unlucky-but-common export they get "no strings found"-class output for a valid
PSBT. Wrong refusal, no wrong artifact. Fix: anchor the discriminator
(`starts_with("cHNidP8")` for PSBT, or match `mt1` only at a string boundary).

**Why tests miss it:** `inspect.rs` tests use PSBTs that happen not to contain
the substring.

---

### M-6 (Minor) — both parsers admit a segwit-marked transaction whose every witness stack is empty; Bitcoin Core will not deserialize those bytes

**Files:** `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/sysw/tx.rs:195-207`
and `/scratch/code/shibboleth/seedhammer/mt/mt.go` `ParseTx` (identical
behavior; no cross-language divergence).

**Defect:** a body of the form `version ‖ 00 01 ‖ inputs-with-scriptSigs ‖
outputs ‖ (empty witness stacks) ‖ locktime` parses in both implementations
(marker+flag accepted, all-empty witnesses accepted) and — if every scriptSig is
non-empty — passes the signature predicate. Bitcoin Core's deserializer rejects
exactly this shape ("Superfluous witness record"), and the legacy re-parse fails
on the 0x00 input count, so **no node accepts the bytes as engraved**. A QR plate
cut from such a `tx:` record states "raw signed bitcoin tx … then broadcast" for
bytes no node will take.

**Failure scenario:** only a hand-crafted record reaches it — `mt encode --qr`
cannot emit the shape (rust-bitcoin serializes the legacy form when all
witnesses are empty), and the spec says records come from `mt encode`. Recovery
is possible by re-serializing without the marker, and the txid on the plate is
correct. Minor: refusing the shape (`BadSegwitFlag`-style, in both languages, one
vector) closes a smuggling-adjacent admission for free.

**Why tests miss it:** every segwit vector in both suites carries at least one
witness item.

---

### M-7 (Minor, S0 tie-in) — the post-cut verify instruction presumes an "ordinary" phone scanner presents the QR's binary payload as joinable hex

**Files:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:825-828`
("Scan every QR with a phone, join the hex, and run `mt inspect`");
`mt-cli/src/blocks.rs` `verify_the_steel(RawRecord)` ("SCAN the cut symbol with
an ordinary QR reader and run `mt inspect --in scanned.hex`").

**Defect:** the symbols carry raw transaction *bytes* (byte-mode, settled by the
measured QR findings and the ZXing decode gate — not re-litigated here). Generic
phone camera apps render byte-mode content as text; raw transactions contain
0x00 and non-UTF-8 bytes, which many apps truncate or replace. "Join the hex" is
an instruction about a presentation layer nothing in this cycle has measured —
the ZXing gate proved byte-level decodability, not that a phone hands the
operator hex. If the operator's app garbles, they will conclude a good plate is
bad (or file it unverified). This is precisely the steel/scanner half the
operator deferred to S0 (ruling 2026-08-25d), so it is filed as the one item
that could make the S0 session confusing rather than exploratory: **S0 should
include one named scanner app whose output the instruction's wording matches.**
No spec change proposed pre-tag.

---

### N-1 (Nit) — `decide_sealing` prints the `sealing:` line before the `--iterations` range gate can still abort the run

**File:** `/scratch/code/shibboleth/mnemonic-engrave/crates/me-cli/src/main.rs:1211-1226`.
The diff's own F-246 rule is "no line describing a container may print until
every gate that can abort the write has run"; `me sysw pack --iterations 5
<records>` prints `sealing: SEALED — …` and then exits 2. One line, wrong order,
same class the diff fixed elsewhere.

### N-2 (Nit) — the R16 QR refusal advises "Use TEXT plates" for a candidate that may have none

**File:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:1078-1088`. A
`tx:`-record-only candidate has no mt1 strings, so the remedy is impossible for
it. Almost certainly unreachable: the payload section cap (32,734 chars ≈ 16.3 kB
of transaction) sits below the measured 16-symbol/ECC-M ceiling at 0.6 mm, so a
packed record that classifies should always fit QR. Recorded because the refusal
text is permanent operator guidance and the ceiling logic may move.

### N-3 (Nit) — `qrCeilingBytes` measures module fit only, not the full `build`

**File:** `/scratch/code/shibboleth/seedhammer/gui/transaction.go:1096-1145`. The
refusal's quoted ceiling checks `c.Size*stroke*scale <= usable` but not
`toPlate`/legend/title assembly, which `buildQRPlates` additionally requires — so
the stated ceiling can exceed what the planner would actually accept by a small
margin. Refusal-message accuracy only.

---

## Checked and clean (so the fold does not re-derive them)

- **Signature predicate parity, Rust↔Go:** per-input, scriptSig-or-witness,
  identical semantics including the mixed-transaction case; verdict defined from
  the index list on both sides (`me-cli sysw/tx.rs` / `mt/mt.go` at tip).
- **mt1 header layout parity:** 5|20|15|15, count−1 on the wire, version and
  index<count checks in the same order (`mt-codec string_layer/header.rs` /
  `mt/mt.go parseHeaderSyms`); Go's shift/mask precedence verified.
- **Whitespace/case handling parity:** mt-codec's pipeline trims and lowercases
  (pipeline.rs:66); both classifiers are strict; padded records converge.
- **Section-cap raise:** 32,734 formula identical both sides, compile-time
  asserted both sides (negative-array trick in Go), seal frozen at 8191 both
  sides, cross-repo source-reading gate present.
- **Ordering:** R2 (tx-on-argv, exit 3) runs before the hoisted write gate
  (exit 2) in `run_sysw`; all mt-cli encode refusals precede the first stdout
  byte; the bearer warning precedes the txid on the device review screen.
- **Bearer hygiene elsewhere:** no refusal or warning echoes a record body in
  either CLI; the terminal-destination refusal does not honor
  `--allow-world-readable`; `--out` files are 0600; the picotool-pipe no-op is
  named rather than offered.
- **mt-codec dependency pin:** rev `72b79b8` is byte-identical to
  `ship/tx-engraving`'s `crates/mt-codec/` (`git diff` empty).
- **Structured Append encoding:** mode 0011, 4-bit position, 4-bit total−1,
  8-bit parity = XOR of the undivided message — matches ISO/IEC 18004; byte-mode
  pinned on both the k=1 and SA paths.
- **The host-packed-payload seam test** (`transaction_crosslang_test.go`) really
  crosses the seam: Rust-written bytes, Go-read, both plate kinds planned.
- **PSBT path (the walk's coverage gap):** `finalized_guard_psbt` is per-input on
  the final fields; `extract_tx_unchecked_fee_rate` + `consensus::serialize`
  yields the witness-carrying form `--qr` records and both parsers accept;
  an all-legacy finalized PSBT serializes legacy and parses. No divergence found.

## Verdict

**BLOCK** — two Important findings, both small point fixes:

1. **I-1**: refuse (or F-102-warn) `mt1` records on argv in
   `me sysw pack` — the diff's own bearer-channel argument, applied to the class
   it missed.
2. **I-2**: stop `orderByIndex` from silently thinning a detected-ambiguous set —
   engrave everything or say what was dropped; alternatively obtain an explicit
   operator ruling extending 2026-08-25c to this site, which would clear it.

Neither taints the committed vectors, the wire format, or anything already cut.
Everything else recorded is Minor/Nit and does not hold the tag.
