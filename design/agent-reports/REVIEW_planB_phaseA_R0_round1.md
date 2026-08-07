# Plan B Phase A — R0 round 1 (architect review, verbatim)

- **Date:** 2026-08-07
- **Reviewer:** independent opus architect, read-only
- **Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseA.md` @ `de13393`
- **Gate:** R0. No device code may be written until 0 Critical / 0 Important.
- **Stated as machine-verified in the brief:** `scripts/plan-cite-gate.sh` — 20
  citations, all resolve. F-67 fixed (`4192458`). F-69/F-70 closed.

## VERDICT: 3 Critical, 6 Important, 6 Minor, 1 Nit

Controller's independent spot-checks before folding (not taken on the
reviewer's word):

- **C2 confirmed empirically.** A throwaway Go program against the fork:
  `lower ValidMD=true / upper ValidMD=true / mixed ValidMD=false`. Uppercase
  records really do pass, and `gui/codex32_input_test.go:62` really does assert
  the device's own keyboard path emits `strings.ToUpper(valid)`.
- **C3 confirmed by Rust semantics.** `tests/` is a separate crate; it sees only
  `pub` items. `seal_deterministic` is `pub(crate)` and every fixture is inside
  `#[cfg(test)] mod tests`. Task 1 as written cannot compile.
- **C1 confirmed by inspection.** The plan has no container task; `container.rs`
  exists in the normative Rust with `MAX_RECORDS`/`MAX_RECORD_LEN`, and Task 7
  said only "split on LF".

---

```
[Critical] §6.4's record container has no task, no file, and no test in Phase A
Where:   whole plan — Tasks 2–7 and "What Phase A does NOT cover"
Claim:   The plan ports wire.rs, pubhash.rs, crypto.rs and record.rs but never
         ports the §6.4 SECTION DECODE. Task 7 says only "split on LF". None of
         `1 <= record_count <= 24` (across both sections), `1..512` bytes per
         record, no-empty-record, no-CR, no-leading/trailing-LF, the mandatory
         PRE-SPLIT separator scan, or the "too many records (N, max 24)" error
         distinct from "payload unreadable" appears anywhere. Phase B is declared
         UI-only, so this normative surface is owned by nobody.
Proof:   SPEC §6.4 (design/SPEC_encrypted_payload_delivery.md:563-668) is marked
         NORMATIVE and states at :655 "Count the `0x0A` separators and reject
         `record_count > 24` BEFORE splitting. A plaintext of 8191 LF bytes
         satisfies `ct_len <= 8191` ... an implementation that splits first
         materialises ~8192 slice headers (~98 KB on a 32-bit target), a fifth of
         the free heap, transiently." §11.2 (:1425) makes this reachable with NO
         passphrase and NO KDF — "A public section of 8191 LF bytes with
         `ct_len == 0` is rejected on the separator count before any split
         allocates — asserted with `testing.AllocsPerRun` bounded to 0 additional
         allocations", and :1462 "a return-value assertion here is a guaranteed
         false PASS ... `bytes.Split` performs exactly one allocation ... a
         correct `bytes.Count`-style scan performs zero. Both are 'O(1)'."
         The plan's Task 2 constant list (line 155-158) omits MaxRecords and
         MaxRecordLen; crates/me-cli/src/seal/container.rs:10-11 defines both.
         This is entirely headless and host-testable, i.e. squarely Phase A.
Fix:     Insert a task before Task 5 creating `seal/container.go` +
         `seal/container_test.go`: `SplitSection(b []byte) ([]string, error)`
         doing a pre-split `bytes.Count(b, "\n")` bound (reject > 23 separators
         before allocating), per-record 1..512 length, reject CR / empty /
         leading / trailing LF, and a caller-side check that
         `len(public)+len(secret) <= 24`. Tests: the 8191-LF case asserted with
         `testing.AllocsPerRun(...) == 0` (the number, not "O(1)"), and a
         25-record payload returning a record-count-naming error distinct from
         "payload unreadable". Add both constants to Task 2. Add mutation rows
         "split-before-count" (killed by the 0-alloc assertion) and "record-count
         cap dropped" (killed by the 25-record test).
```

```
[Critical] No all-lowercase check — the Go validators accept uppercase by design,
           and that disarms the §6.6 hash
Where:   Task 5, "Step 1: Failing test" and the allow-list table
Claim:   `record.rs::validate_record` rejects any uppercase character before it
         classifies. The plan never mentions case, and the Go function the plan's
         allow-list rests on (`codex32.ValidMD`/`ValidMK`) accepts a fully
         uppercased string on purpose. So the device would admit `MD1FV9W…`,
         engrave it verbatim, and display a §6.6 hash the operator's recorded
         value cannot match — on an untampered payload.
Proof:   Rust: crates/me-cli/src/seal/record.rs:64-70 returns
         `RecordError::NotLowercase(pos)` with the comment "or the same wallet has
         two different public-data hashes (§6.4)". Fork:
         seedhammer/codex32/mdmk.go, `verifyMDMK` — "Feed the ORIGINAL-cased HRP
         (not the lowercase literal) so the engine's case state matches the data;
         this is what makes uppercase strings validate and mixed-case ones fail".
         SPEC §6.4 (:624-637) makes lowercase a normative MUST and states the
         harm: "the device's own keyboard-entry path emits uppercase
         (`gui/codex32_input_test.go:62`). An operator re-deriving with `me hash`
         from their engraved cards would then see a mismatch on an untampered
         payload — and learn that mismatches are normal, which disarms the single
         control §6.6 exists to provide." Confirmed live:
         seedhammer/gui/codex32_input_test.go:62 asserts
         `mdmkText(strings.ToUpper(valid))`. §11.2 (:1409) names the test: "An
         uppercase record is refused (§6.4). `MD1QQQ…` passes `ValidMD`."
         No test in Task 5 or the mutation table touches case; mutant "lowercase
         check removed" survives every named test.
Fix:     Add to Task 5 Step 1: a lowercase check running BEFORE classification,
         mirroring record.rs:64-70, binding BOTH sections, plus
         `TestRefusesAnUppercaseRecord` using `strings.ToUpper` of a vector
         record. Add a mutation row "lowercase check removed → that test".
```

```
[Critical] Task 1's exporter cannot compile: `seal_deterministic` is `pub(crate)`
           and its fixtures are `#[cfg(test)]`
Where:   Task 1, Step 1 ("create `crates/me-cli/tests/emit_vectors.rs` … It calls
         the same `seal_deterministic` seam")
Claim:   An integration test under `tests/` is a separate crate and sees only
         `pub` items. `seal_deterministic` is `pub(crate)`, and every vector
         fixture (`PASS`, `bacon24()`, `bip84()`, `two_of_three()`, `salt()`,
         `iv()`) lives inside `#[cfg(test)] mod tests`. Task 1 as written does not
         build, and it is the root dependency of Tasks 2–5 and 7. The shortest fix
         an implementer reaches for — `pub fn seal_deterministic` — is a security
         regression the Rust explicitly forbids.
Proof:   crates/me-cli/src/seal/mod.rs:150 `pub(crate) fn seal_deterministic`,
         doc comment at :145-149: "`pub(crate)` and never re-exported. A public
         version destroys the one-key-one-message property the moment a caller
         reuses a salt, and there is no legitimate reason for a caller to choose
         one." Fixtures: mod.rs:241 `#[cfg(test)] mod tests`, :244 `const PASS`,
         :256 `fn bacon24`, :263 `fn bip84`, :808 `fn two_of_three`. lib.rs:9 has
         `pub mod seal`, so `seal::pubhash` etc. ARE reachable — only the
         deterministic seam and the fixtures are not.
Fix:     Move the exporter into `src/seal/mod.rs`'s existing `mod tests` as a
         `#[test] fn emit_vectors()` gated on `ME_EMIT_VECTORS`, and change the
         command in Step 1/Step 3 to
         `ME_EMIT_VECTORS=1 cargo test -p mnemonic-engrave --lib seal::tests::emit_vectors`.
         This also gives it `bip84()`/`two_of_three()` directly, removing the
         retyped-input drift Step 2 cannot detect. Explicitly forbid widening
         `seal_deterministic`'s visibility.
```

```
[Important] `ChunkSetID` is not the chunked discriminator — `.Chunked` is, and
            csid 0 is a legal chunked value
Where:   Task 5, Step 2/3
Claim:   The plan tells the implementer to detect non-chunked records but names
         only the field that cannot answer the question. Both Go parsers return
         `ChunkSetID == 0` for a non-chunked record AND allow `ChunkSetID == 0`
         for a chunked one, so keying on csid conflates the two in both
         directions: unrelated single-string cards collide into one group
         (`mk.reassemble` then errors on `len(frags) != 1`), and a legitimate
         chunked card with csid 0 is split into failing singletons. Rust avoids
         this with a distinct type (`Option<u32>`), which the plan does not carry
         over. Both cited expressions are also not valid Go — the functions return
         `(T, error)`.
Proof:   seedhammer/md/chunk.go:194-197 — `if syms[0]&1 == 0 { return
         ChunkHeader{Chunked: false}, nil }` (ChunkSetID left zero);
         chunk.go:66 guards only `h.ChunkSetID >= (1 << 20)`, so 0 is legal.
         seedhammer/mk/mk.go:78 — `case typeSingle: return Header{Chunked: false,
         TotalChunks: 1, ChunkIndex: 0}` (csid zero); csid at mk.go:83 is 4×5 bits,
         0 legal. The live sites the plan cites as doing "exactly this" branch on
         `.Chunked`, not csid: gui/md1_gather.go:34 `if !h.Chunked { return
         gatherIgnored }`; gui/mk1_inspect.go:65 `else if !h.Chunked || ...`.
         Rust: record.rs:113 keys on `(char, Option<u32>, usize)` and record.rs:165
         produces `None` from a failed `ChunkHeader::read`.
         No test in Task 5 constructs a non-chunked record at all — vectors D, E
         and G are all chunked — so the mutant "route every group to
         `md.Reassemble`, drop the `md.Decode` arm" survives
         TestDecodesACompleteCardSet, TestGroupsByHRPAndChunkSetID,
         TestRefusesAnIncompleteCardSet and TestLeftoverRecordRejects. There is
         no mutation-table row for it either.
Fix:     In Task 5 Step 2/3, state the key as `(hrp, chunked, csid, uniq)` where
         `chunked` comes from `h.Chunked` and `uniq = i+1` when `!h.Chunked`, 0
         otherwise; show the two-value call form. Add
         `TestDecodesTwoDistinctNonChunkedCards` (two single-string md1 cards must
         NOT collide, and each must route to `md.Decode`) and a mutation row
         "non-chunked dispatch arm removed → that test".
```

```
[Important] Task 7's "tampered public record fails at the tag" cannot happen in
            the pipeline Task 7 specifies
Where:   Task 7, Step 1, third negative
Claim:   Task 7 orders the pipeline "parse, allow-list, group-decode, hash, and
         where `ct_len > 0`, open". A flipped byte in a public record breaks its
         BCH checksum, so it is rejected at the allow-list — before any key is
         derived and before `Open` is ever called. A test asserting only that an
         error comes back therefore passes under the mutant this negative exists
         to kill (AAD dropping the public section), because the record never
         reaches the tag.
Proof:   Plan Task 7 Step 1 fixes the order; SPEC §10.2 steps 2 and 8
         (design/SPEC_encrypted_payload_delivery.md:1092, :1108) put the
         allow-list at step 2 and the GCM open at step 8. `codex32.ValidMD` is a
         BCH verifier with no correction (codex32/mdmk.go, `verifyMDMK` — "Pure
         verify, no error correction"), so a single-byte flip is caught with
         overwhelming probability. Task 4 already covers the flipped-public-byte
         case correctly at the `Open` level; Task 7's version adds nothing and
         mis-describes the failure point.
Fix:     Replace the tamper with one that survives the allow-list: REORDER the
         public records of vector D. Every record stays BCH-valid, every group
         still reassembles and decodes (`mk.reassemble` slots by ChunkIndex), so
         the pipeline reaches step 8 and fails on the tag — and the §6.6 hash
         moves too. Assert the returned error is the authentication error
         specifically, not merely non-nil.
```

```
[Important] Mutation row "XIP read unbounded → the RegionLen test" names a test
            that cannot exist
Where:   Mutation table, last row; Task 6 Steps 1, 3, 5
Claim:   The unbounded-read mutant lives in `seal/read_tinygo.go`, which carries
         `//go:build tinygo` and is therefore never compiled by a host `go test`.
         Task 6's only target-side verification (Step 4) is a manual flash-and-
         read-serial procedure, not an assertion. So no test kills that mutant,
         and the table claims one does.
Proof:   Plan Task 6 Step 1 creates `seal/read.go` (host stub) and
         `seal/read_tinygo.go` (`//go:build tinygo`); Step 3 places the RegionLen
         bound on "this read"; Step 4 is "Verify on the Pico 2 … read serial with
         `scripts/cdcread.py` … print in a loop". A `RegionLen` test against the
         host file-reading stub exercises a different function body entirely.
Fix:     Either (a) hoist the bound into an untagged helper both implementations
         call — e.g. `func clampRegion(n int) int` in a file with no build tag —
         and point the mutation row at a host test over that helper; or (b) change
         the row to "killed by: nothing automated; verified manually in Task 6
         Step 4" and add an explicit Step-4 sub-step that prints the byte count
         actually read and asserts it against 65536. Do not leave the row as an
         unearned kill.
```

```
[Important] Nothing tests that §6.2 bounds fail closed BEFORE the KDF runs
Where:   Task 2 Step 3; Task 7
Claim:   The plan states the ordering guarantee — the one that stops an unbounded
         iteration count hanging a watchdog-less firmware — and then specifies no
         test that can observe it. Task 2's tests call `ParseHeader` in isolation,
         which does no KDF under any mutant. The mutant "move the bound checks
         after `DeriveKey` in `open.go`" passes every named test in the plan,
         because each still returns an error, just ~31 s later.
Proof:   Plan Task 2 lines 132-134 and 162. SPEC §11.2:1400 requires it and says
         how: "Every §6.2 bound violation fails closed *before* the KDF runs —
         asserted by timing or by instrumenting the KDF call, not merely by return
         value." Nothing in Task 2, Task 4 or Task 7 instruments `DeriveKey`, and
         the mutation table's "bound check dropped from ParseHeader" row is about
         a different mutant.
Fix:     In Task 7, make the pipeline take the KDF as an injectable seam (or a
         package-level counter incremented by `DeriveKey`), and add
         `TestBadHeaderNeverReachesTheKDF`: for each §6.2 violation, assert the
         KDF call count is 0. Add the matching mutation row.
```

```
[Important] The debug-command test as specified is killed by a deny-list mutant
            but not by the check-record-0-only mutant
Where:   Task 5, `TestPublicSectionRefusesDebugCommand`
Claim:   The plan says only "`command: lock-boot` in either section is refused".
         Placed at index 0 — the natural way to write it — the test passes under
         a loop that validates `records[0]` and trusts the rest, which is the
         defect §11.2 singles out as load-bearing on the one branch that burns
         OTP. The plan also does not require asserting that no side effect
         occurred, only that the payload is refused.
Proof:   SPEC §11.2:1441-1448: "a `command: lock-boot` record in **position 3 of
         6**. That last case is the load-bearing one: it proves the allow-list
         runs **per record** rather than on the first only. A deny-list, or a loop
         that checks record 0 and then trusts the rest, engraves records 0-2 and
         only then meets the command — so the test MUST also assert that
         **nothing was engraved**." §10.2.1:1178 confirms "The allow-list runs
         once per record." The branch is real: cmd/controller/platform_sh2.go:545
         `func (p *Platform) LockBoot()` → `writeOTPValues()` →
         `otp.EnableSecureBoot()` → `machine.CPUReset()`, reached from
         gui/gui.go:1672 `case "lock-boot":` with `const cmdPrefix = "command: "`
         (gui/scan.go:55) as the only gate.
Fix:     Specify the record at index 2 of a 6-record section, and require the test
         to assert the returned record slice is empty/nil (Phase A's stand-in for
         "nothing was engraved") in addition to the error. Split the mutation row
         into "allow-list → deny-list" and "allow-list applied to records[0] only",
         each naming its killer.
```

```
[Important] `vectors.json` declares no section lengths, but Task 2's binding test
            asserts them
Where:   Task 1 Step 1 (JSON shape) vs Task 2 Step 1
Claim:   Task 2 calls it "the test that actually binds the port": decode each
         `header_hex` and "assert the parsed fields equal that vector's declared
         `iterations` / `salt_hex` / `iv_hex` / **section lengths**". The schema in
         Task 1 has `iterations`, `salt_hex` and `iv_hex` but no `pub_len` or
         `ct_len`. An implementer will improvise, and the shortest improvisation —
         reading bytes 44..52 of `header_hex`/`blob_hex` — asserts the parser
         against the same bytes it parsed, which cannot fail.
Proof:   Plan lines 76-99 list every JSON key; no length field appears. Rust
         Header carries `pub_len` and `ct_len` (crates/me-cli/src/seal/wire.rs:30,
         :32) and they are the two fields that drive the AAD/ciphertext split in
         Task 4 and the `ct_len == 0` / `pub_len == 0` branches in Task 7.
Fix:     Add `"pub_len"` and `"ct_len"` to the JSON schema, emitted by the
         exporter from the `Header` struct it constructed (not from the encoded
         bytes), and state in Task 2 that the expectations come from those fields.
```

```
[Minor] Vector E is not produced by `seal_deterministic`
Proof:  crates/me-cli/src/seal/mod.rs:363-384 `fn vector_e_public_only` calls
        `seal_public_only(all[1..].to_vec())` (public API at mod.rs:122).
Fix:    Say in Step 1 that E is emitted via `seal_public_only` and that its
        `passphrase`/`iterations`/`salt_hex`/`iv_hex` are `null`/zero.
```

```
[Minor] "Plan A pins, per vector: … header hex, derived key, GCM tag …" is false
Proof:  mod.rs:279-459 (each vector test pins `b.len()` and `sha(&b)` only; B at
        :301 pins sha alone); crypto.rs:86-102 pins derived keys for
        `salt([0xbe,0xef])` at 100_000 and 100_001 only; pubhash.rs:69-80 pins D
        and E; mod.rs:414-418 pins G. No `header_hex` or tag literal exists.
Fix:    Restate as what is actually pinned; say Step 2's cross-check covers
        `blob_sha256` and the D/E/G hashes only.
```

```
[Minor] The RegionLen total-size check in Task 2 is unreachable
Proof:  crates/me-cli/src/seal/wire.rs:119-124 (caps first), :161-165 (total
        after the split); wire.rs has no `TooLarge` test.
Fix:    Keep the check but note it is unreachable behind the section caps and
        excluded from the mutation row.
```

```
[Minor] `Reader` returning `([]byte, error)` has no slot for the required
        "distinguishable no-payload signal"
Fix:    Specify `Read() ([]byte, bool, error)` or an exported `ErrNoPayload`
        sentinel matched with `errors.Is`, and say which.
```

```
[Minor] Vector C's per-record classification assertion is dropped
Proof:  SPEC §11.2:1466-1470 — "classified in order as `ms1`, `mk1`, `mk1`,
        `md1`, `md1`, `md1` … The test MUST assert the classification of each
        record, not merely that the bundle parsed."
Fix:    Add to Task 7: assert the per-record kind sequence for vector C.
```

```
[Minor] Task 6 Step 4's board-identification procedure is not executable
Fix:    Add a concrete enumeration step (`picotool info -a` matching the chipid,
        or `lsblk -o NAME,MODEL`), and a hard precondition: physically disconnect
        the SeedHammer II before Step 4, then confirm exactly one RP2350.
```

```
[Nit] Task 2's constant list omits the two algorithm ids it then checks
Proof:  crates/me-cli/src/seal/wire.rs:9-10.
Fix:    Add `KDF_PBKDF2_SHA256 = 0x01` and `AEAD_AES256GCM = 0x01`.
```

---

Two things checked and **not** filed, so they are not re-derived next round:

- **`corrections_applied != 0` (mk1 "pristine")** — `record.rs:78-84` rejects
  BCH-corrected mk1; the plan does not port it. Genuine no-op in Go:
  `codex32.ValidMK` and `mk.Decode` do no error correction. Parity holds by
  construction.
- **`first_noncanonical` (interior space/hyphen)** — also unported, also
  naturally satisfied: the codex32 engine's `inputChar` has no mapping for `0x20`
  or `-`, so `ValidMD`/`ValidMK` return false. Worth one sentence in Task 5 so the
  omission reads as deliberate, but not a defect.

The §10.2.1 allow-list branch enumeration is **correct and complete** against
`gui/scan.go:55-79`. The classifier-ordering trap does not bite — no BCH-valid
`md1`/`mk1` string can be captured by an earlier branch.

VERDICT: 3 Critical, 6 Important, 6 Minor, 1 Nit
