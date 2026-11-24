# Independent review of HOST_INTERFACES.md

Review date: 2026-10-30. Reviewer: Codex, independent of the document author.

Result: **12 confirmed findings: 2 P1 and 10 P2.** A shared shell-placeholder defect is recorded separately for deduplication with the parent's GEMINI.md review. These are documentation findings against the specified sources, not findings that the implementations should be changed to match the document.

## Scope and exact sources

Only host/device contract facts and executable recipes in `design/firmware-architecture/HOST_INTERFACES.md` were reviewed. Document line numbers below refer to commit `03776d704d2bf85a32f89c900b0a7368d06e0b5c`.

Source abbreviations used in every evidence citation:

- **ME**: `/scratch/code/shibboleth/me-worktrees/firmware-architecture`, document branch `docs/firmware-architecture`, HEAD `03776d704d2bf85a32f89c900b0a7368d06e0b5c`; production source baseline `a75719996304086e5f293aaf26812ff242a25244`.
- **SH**: `/scratch/code/shibboleth/sh-worktrees/gemini-init`, branch `docs/gemini-init`, HEAD `ec4fffddcd10b36409ae6d3dba9b8354f6367af2`; production source baseline `abdcb1825ed3bde5e01ff0e4da33f76f08f604c2`.
- **MS**: `mnemonic-secret` at `e5dff6f9c2b7ae7dc02a4a137e7322abab32211e`, read with `git show` from `/scratch/code/shibboleth/mnemonic-secret`.
- **MT**: `mnemonic-transaction` at `72b79b8762ec208b6143afade8c4b66a551bd29d`, read with `git show` from `/scratch/code/shibboleth/mnemonic-transaction`.

The ME commit adds only the four architecture documents relative to its source baseline. The SH commit adds only GEMINI.md relative to its source baseline. Thus the cited ME/SH implementation files in those worktrees are the exact requested source versions. Both reviewed worktrees were clean at the final pre-report check.

This report is the sole file added by this review, in the separately authorized worktree `/scratch/code/shibboleth/me-worktrees/firmware-architecture-codex-review`, branch `docs/firmware-architecture-codex-review`, initially clean at the exact document commit. No commits were made. The earlier off-repository report destination was not used.

Applicable ancestor and repository guidance was read. This is the independent interface slice of the parent's parallel review. Navigation, other architecture documents, general workflow, startup/homing, steps/mm, and broader provenance review remain with the parent. The signing/toolchain evidence below is included only where it directly contradicts this document's host/device contract.

## Confirmed findings

### H01 — P1: The OTP table names incorrect rows, redundancy, validity bits, and page locks

**Document:** lines 199–203; related summary at line 19.

The table places CRIT1 at `0x0005` with threefold redundancy, describes boot-key validity in BOOT_FLAGS0 at `0x0048` with a `slot + 8` offset, and names PAGE1_LOCK1/PAGE2_LOCK1 at `0x0f85`/`0x0f89`. These disagree with both the firmware driver and host provisioning reader.

**Exact evidence:**

- `SH/driver/otp/otp.go:19–21` defines `BOOTKEY0_0 = 0x080`, `CRIT1 = 0x040`, and `BOOT_FLAGS1 = 0x04b`.
- `SH/driver/otp/otp.go:93–99` enables secure boot with `writeOrRow(CRIT1, 8, 0b1)` and reads eight redundant rows. It does not implement the table's universal `0x43` provisioning recipe.
- `SH/driver/otp/otp.go:159–169` writes the key hash and then sets `1 << slot` in BOOT_FLAGS1 with redundancy three. Lines 289–295 require that validity bit and require the corresponding bit shifted by eight to be clear. The table's purported validity offset is the invalidation offset in the actual register.
- `ME/scripts/refugium-otp.sh:398–405` independently reads CRIT1 at `0x040` with redundancy eight, BOOT_FLAGS1 at `0x04b`, BOOT_FLAGS0 separately at `0x048`, PAGE1_LOCK1 at `0xf83`, and PAGE2_LOCK1 at `0xf85`.
- `ME/scripts/refugium-otp.sh:583–588` checks the applicable profile's LOCK1 value as `0x040404` and LOCK0 as zero, not the document's `0x15`.

**Consequence:** An operator translating this normative table into provisioning operations would target incorrect OTP state. Applying its validity offset to the actual BOOT_FLAGS1 register would invalidate a key. OTP mistakes can permanently consume key capacity or prevent the intended boot configuration. This severity follows from the erroneous irreversible provisioning contract; no physical failure was induced or claimed.

**Correction:** Replace the table from the exact driver and profile-specific provisioning sources. Keep register identity, redundancy, and bit meanings separate from profile policy. The host script's checked values are evidence for that profile, not a replacement universal burn recipe.

### H02 — P1: The boot-key hash preimage has the wrong coordinate byte order

**Document:** line 200.

The document says the boot-key digest hashes a 64-byte *little-endian* secp256k1 public key. The signing and provisioning implementations use the uncompressed point's big-endian coordinates, `X || Y`, without the leading `0x04`.

**Exact evidence:**

- `SH/cmd/picosign/main.go:142–143` serializes X and Y with `big.Int.FillBytes` into the two 32-byte halves of the image key. The local Go standard library, `/scratch/code/shibboleth/.toolchain/go/src/math/big/int.go:535–542`, specifies and implements `FillBytes` as zero-extended big-endian serialization.
- `ME/scripts/lib/otp-read.sh:464–473` extracts the 65-byte uncompressed point, verifies its `0x04` prefix, and hashes the following 64 bytes unchanged. There is no coordinate reversal.
- `ME/scripts/pico2-bootkey-rehearsal.sh:227–234` discusses little-endian two-byte OTP row storage. That storage convention is distinct from the coordinate serialization used to derive the digest.

**Consequence:** Reversing coordinates before hashing produces an OTP digest that does not identify the public key embedded by the actual signing tool. Burning it permanently spends a slot on the wrong digest and can leave the intended image unable to authenticate after secure boot is enabled.

**Correction:** Specify SHA-256 over 32-byte big-endian X followed by 32-byte big-endian Y, excluding the uncompressed-point prefix. Describe OTP row storage byte order separately.

### H03 — P2: The claimed picotool SRAM patch and emitted clear range contradict the actual signer

**Document:** lines 191, 197, and 206; related summary at line 19.

The document describes a patch that changes the SRAM clear endpoint from `0x20082000` to `0x20080000`, claims this is the emitted range, and attributes boot failure to the original endpoint. The actual patch does not change that endpoint, and the host signing gate explicitly requires `0x20082000`.

**Exact evidence:**

- `ME/flake.nix:27–40` selects picotool 2.3.1 and applies `nix/patches/picotool-2.3.1-seal-clear-fix.patch`. The document's named SRAM-range patch is not the selected patch.
- `ME/nix/patches/picotool-2.3.1-seal-clear-fix.patch:30–55` contains two implementation changes: appending image bytes to the hash input instead of inserting them at the beginning, and passing `model` instead of `in.get_model()`. Neither changes the SRAM endpoint.
- `ME/scripts/lib/seal-check.sh:22` requires the exact load-map text `Clear 0x20000000->0x20082000`.
- `ME/scripts/sign-firmware.sh:193–199` enforces the clear-entry check on the signed output.
- `SH/flake.nix:73–74` imports ME's picotool package, and lines 130–132 use it to seal the firmware image.

**Consequence:** A maintainer implementing the described patch would build a different tool, and an image carrying the document's range would fail the actual signing gate. The explanation also misstates why the selected patch is required. A source contradiction is confirmed; this review does not infer new boot-ROM behavior from it.

**Correction:** Describe the actual hash-order/model fixes and the range required by the source gate. Remove the unsupported stack-wipe causal account. Coordinate this cross-cutting finding with the parent's toolchain review.

### H04 — P2: The picosign recipe invokes a nonexistent CLI and omits the key-dependent digest order

**Document:** line 198; adjacent recipe at line 197.

`cmd/picosign -uf2 ... -key ... -sigfmt raw` is not this tool's command interface. The tool does not load a private key and generate a signature. Its `sign` subcommand inserts a supplied public key and supplied signature. The digest also includes image-block data, not just the described load-map ranges.

**Exact evidence:**

- `SH/cmd/picosign/main.go:26–34` defines `hash`, `sign`, and `extract` flag sets; the sign options are `-clear`, `-pubkey`, `-sig`, and `-sigfmt`.
- `SH/cmd/picosign/main.go:36–65` dispatches on the first argument as a subcommand; the documented `-uf2` is therefore an unknown command.
- `SH/cmd/picosign/main.go:121–169` decodes an externally supplied public key/signature and obtains the image from the positional argument. It does not sign with a private key.
- `SH/picobin/picobin.go:300–347` hashes the load-map inputs and then the defined portion of the image block at lines 343–344.
- `ME/scripts/sign-firmware.sh:118–142` first inserts the real public key with a placeholder signature, computes `picosign hash`, signs that digest externally, inserts the signature, and verifies that inserting the signature did not change the digest.

**Consequence:** The shown command fails immediately. An independent implementation following the abbreviated digest description, or hashing before replacing the dummy public key, can produce a signature over the wrong message.

**Correction:** Describe `picosign` as a hash/extraction/embedding utility and reference the existing script's ordering. Do not present the nonexistent private-key CLI as executable guidance. No signing operation was run for this review.

### H05 — P2: The sysw ciphertext slice omits the separate 16-byte authentication tag

**Document:** lines 68–71; the shared seal header description at line 96 should also make this distinction explicit.

Line 71 says `blob[52+pub_len : 52+pub_len+ct_len]` contains ciphertext *and* the GCM tag. In both implementations, `ct_len` excludes the 16-byte tag.

**Exact evidence:**

- `ME/crates/me-cli/src/sysw/mod.rs:634–650` sets `header.ct_len = plaintext.len()` at line 639, then appends the AEAD output containing the additional tag.
- `ME/crates/me-cli/src/sysw/wire.rs:181–187` adds the tag separately to the total length when sealed.
- `SH/sysw/wire.go:102–109` computes `HeaderLen + PubLen + CtLen`, then adds `TagLen` when sealed.
- `SH/sysw/open.go:44–56` passes the bytes from the public-section end through `h.TotalLen()` to the AEAD opener.
- `SH/seal/wire.go:196–200` uses the same separate-tag accounting for MNEMBLOB.
- Read-only fixture arithmetic confirmed sealed sysw examples with `52 + 67 + 93 + 16 = 228` bytes and `52 + 0 + 93 + 16 = 161` bytes.

**Consequence:** A producer or reader implementing the published slice truncates the tag or misstates the header length, causing container rejection or authentication failure.

**Correction:** Define ciphertext as the next `ct_len` bytes and the tag as the following 16 bytes; sealed total length is `52 + pub_len + ct_len + 16`.

### H06 — P2: NDEF decoding, URI transformation, and the 8 KiB boundary are assigned incorrectly

**Document:** lines 28, 31–34, and 47–48; summary line 11.

The document assigns NDEF unwrapping to a nonexistent `gui/scan.go:parseNDEF`, treats URI payloads as pass-through, and calls 8192 bytes the maximum NDEF message size. The poller decodes records before the scanner receives their content. The scanner limit applies to decoded record content, and filling all 8192 bytes is already overflow.

**Exact evidence:**

- `SH/nfc/poller/poller.go:84–90` constructs `ndef.NewMessageReader` and then `ndef.NewRecordReader`.
- `SH/nfc/ndef/ndef.go:95–113` exposes record content and returns EOF at a record boundary. Lines 123–179 parse record framing and skip unsupported/chunked forms.
- `SH/nfc/ndef/ndef.go:183–201` strips Text status/language information and skips UTF-16 records. Lines 202–224 consume the URI identifier byte and expand supported HTTP/HTTPS prefixes; unsupported prefixes are skipped. Lines 225–227 skip other record types.
- `SH/gui/scan.go:29–39` allocates 8192 bytes and sets overflow when the accumulated content length equals that allocation. It receives the already-decoded record content.
- `ME/crates/me-cli/src/ndef.rs:29–61` emits one short Text record and a one-byte-length TLV. Its current TLV encoder permits at most 249 text bytes, not an 8 KiB message.
- A synthetic in-memory decoder probe returned `http://example` from a URI record with prefix byte `0x03`, demonstrating transformation rather than pass-through.

**Consequence:** A host can emit the wrong bytes or misunderstand which limit it must satisfy. The stated lockstep instructions direct a maintainer to the wrong layer and a nonexistent helper; enlarging the GUI buffer does not enlarge the host encoder's framing limits.

**Correction:** Document message framing, record decoding, decoded-content limits, and the ME writer's narrower encoding separately. Point wrapper changes to `nfc/ndef`, not an invented GUI helper.

### H07 — P2: The NFC admission matrix promises unsupported multiline and format routes

**Document:** lines 35–43; summary line 17.

The described line splitter and several caller-specific parsers are absent. Bundle NFC does not split an LF/CRLF-separated Text payload into multiple strings, does not admit plaintext descriptors or UR crypto-output, and transaction NFC does not admit `tx:` records. The scanner also does not implement the advertised SeedQR or enabled SLIP39 input arms.

**Exact evidence:**

- `SH/gui/scan.go:58–105` classifies one complete decoded record. Its active seed arms are `bip39.Parse` and `codex32.New` with preimages excluded. The SLIP39 arm is explicitly commented out at lines 82–86. There is no SeedQR or UR decoder in this dispatch.
- `SH/gui/bundle_flow.go:263–267` passes the single `scan.Object` directly to the gatherer; there is no line split there.
- `SH/gui/bundle.go:157–191` classifies `codex32.String` as refused, admits appropriate `mdmkText` forms, and drops other object types. Thus even a descriptor recognized by the general scanner is not accepted through this bundle door. The named `classifyScanLine` is not the actual helper.
- `SH/gui/transaction.go:686–692` requires `mt.ParseHeader` in the NFC gatherer, and lines 750–765 accept only scanner objects of type `mtText`.
- `SH/gui/transaction.go:451–484` obtains `tx:` records separately from `ctx.sysw.takeAll(sysw.ClassTx)`.
- A synthetic Text-record probe preserved `a\nb` as one decoded record; it did not yield two scan inputs.

**Consequence:** Writing multiple chunk lines into one NDEF Text record, a UR/descriptor for the bundle door, or a `tx:` record for transaction NFC does not implement the documented workflow. Valid information can be rejected simply because it was sent through a falsely advertised transport or admission path.

**Correction:** Give an explicit per-program/per-transport admission table. Distinguish separate NDEF records from embedded newlines, scanner recognition from caller admission, and sysw transaction records from NFC mt1 chunks.

### H08 — P2: The ms1 seed profile has the wrong index, ID, entropy widths, and checksum boundary

**Document:** lines 145–149; related secret-record word-count restriction at line 99.

The document says constellation binary secrets use share index `m`/`M`, assigns mnemonic payloads ID `mnem`, limits entropy to 16 or 32 bytes, calls it raw BIP-32 seed entropy, and places long-checksum strings within a 90-character maximum. These are not the pinned profile's rules. Line 99 also incorrectly limits admitted BIP-39 secret records to 12 or 24 words.

**Exact evidence:**

- `MS/crates/ms-codec/src/consts.rs:19–33` sets threshold `0`, share index `s`, entropy lengths `[16, 20, 24, 28, 32]`, and regular-profile string lengths `[50, 56, 62, 69, 75]`.
- `MS/crates/ms-codec/src/payload.rs:1–14` identifies the seed material as BIP-39 entropy. Lines 19–27 assign **both** Entr and Mnem singles the ID/tag `entr`; `hash` is used for preimages.
- `MS/crates/ms-codec/src/envelope.rs:108–117` specifies threshold `0`, index `s`, and prefix-byte dispatch: `0x00 || entropy`, `0x02 || language || entropy`, or `0x03 || preimage`.
- `SH/codex32/msencode.go:17–38` accepts all five entropy widths and emits `NewSeed("ms", 0, "entr", 's', payload)`.
- `SH/codex32/mspayload.go:35–60` decodes the seed prefixes and all five widths. `SH/codex32/codex32.go:169–170` rejects a threshold-zero string whose index is not `s`.
- `SH/seal/record.go:94–108` documents the parser's short band as 48–93 characters and long band as 125–127, while the engravable limit is 90. Every long-code string is outside that limit.
- `SH/seal/record.go:263–269` admits checksum-valid `bip39.Parse` results, without a 12/24-only filter. `ME/crates/me-cli/src/seal/passphrase.rs:67–70` likewise delegates mnemonic validity to the BIP-39 parser; `ME/crates/me-cli/src/seal/mod.rs:297–339` uses the mnemonic admission path.
- A synthetic checksum-valid 15-word mnemonic classified as `ClassMnemonic` in both Go sysw and seal probes.

**Consequence:** A host emitting threshold-zero index `m` produces a rejected string; a mnemonic single tagged `mnem` violates the Rust profile's tag rule. Conversely, an implementation based on the document would unnecessarily reject valid intermediate entropy/word counts. Confusing BIP-39 entropy with a BIP-32 seed also changes recovery semantics.

**Correction:** Separate general BIP-93 parsing from the constellation payload profile and the plate-length cap. Use index `s`, the actual single IDs, all five supported entropy widths, and the payload prefix to distinguish seed forms. Do not infer payload kind from an invented `m` share index.

### H09 — P2: Hashlock preimages and phrase records have incompatible wire grammars in the document

**Document:** lines 75 and 150.

`ClassPreimage` is not a `preimage:` record, phrase records are not `phrase:<method>:<hex>`, and the binary preimage has no hash-function-ID byte.

**Exact evidence:**

- `SH/sysw/classify.go:48–55,133–141` classifies an admitted ms1 preimage plate as `ClassPreimage`, using `codex32.New` and `codex32.IsPreimagePlate`.
- `SH/codex32/msencode.go:61–69` emits `0x03 || X[32]` under threshold zero, ID `hash`, index `s`.
- `SH/codex32/mspayload.go:145–157` requires exactly 33 decoded bytes, including the prefix. Lines 173–178 require ID `hash` for the plate admission predicate.
- `MS/crates/ms-codec/src/consts.rs:41–48` and `envelope.rs:113–117` specify the same 33-byte shape and 75-character preimage single.
- `SH/sysw/composer_records.go:150–170` unhexes the entire body after `phrase:`, then splits the decoded string at its first comma into method and phrase. Lines 197–198 encode `phrase:` plus lowercase hex of `method + "," + phrase`. `ME/crates/me-cli/src/sysw/composer_records.rs:27–40` uses the same representation.
- Synthetic classifier probes returned Unknown for `phrase:sha256:74657374` and for `preimage:` followed by 64 zero hex digits. `phrase:7368613235362c74657374` classified as Phrase. An in-memory encoded preimage had index `s`, 33 payload bytes, 75 characters, and class Preimage.

**Consequence:** Each documented grammar can cause a host producer to create records that the actual classifier rejects. Adding the claimed function-ID byte makes a preimage the wrong length, independently of checksum validity. Hash algorithm selection for digest records must not be inserted into the preimage plate.

**Correction:** Specify the actual ms1 preimage shape and `phrase:hex(UTF8(method + "," + phrase))`; keep digest-record algorithm selection separate.

### H10 — P2: The interface reference omits the intentional host/device codex32 admission asymmetry

**Document:** lines 16, 74–84, and 97–99.

The combined producer/consumer description presents a valid ms1 share/secret as a common contract without explaining that ME's pack/seal admission is narrower than the firmware's BIP-93 admission. This is an existing, explicitly tested boundary, not a proposed compatibility policy.

**Exact evidence:**

- `ME/crates/me-cli/src/sysw/mod.rs:381–385` delegates seed classification to `seal::record::validate_record`.
- `ME/crates/me-cli/src/seal/record.rs:177–231` calls the constellation `ms_codec::decode` and accepts seed payload kinds Entr or Mnem, rather than treating every BIP-93-valid share/secret as a seed record.
- `SH/sysw/classify.go:147–162` admits the wider BIP-93-valid ms-HRP population within its length cap, subject to its explicit preimage exclusions.
- `ME/crates/me-cli/tests/codex32_seam.rs:1–18` and `SH/sysw/codex32_seam_test.go:12–24` expressly define the conformance invariant as **host-admits implies device-admits**, not equivalence. Both consume the same seam corpus with separate host and device expected columns.
- The checked-in seam corpus contains device-only admissions, including plain BIP-93 128-bit/256-bit secrets and a BIP-93 share. The targeted Go seam test passed.

**Consequence:** A caller cannot assume that a secret accepted by firmware can be round-tripped through `me sysw pack` or `me seal`. The listed conformance evidence does not establish symmetric admission, and a future maintainer must not erase this deliberate difference while attempting to make classifiers match.

**Correction:** State the two admission boundaries and cite the existing seam test/corpus. Keep container integrity, record classification, and later semantic decoding distinct. Do not advertise arbitrary BIP-93 share production through the narrower ME path.

### H11 — P2: Unconfirmed mt1 state is not header version zero, and unsigned detection is structural

**Document:** line 165.

The document associates unconfirmed sets with `Header.Version == 0` and unsigned records with `IsEffectivelyUnsigned()`. Version zero is rejected, and the latter is not the implemented predicate. Warning/substitution behavior exists, but its trigger is misdescribed.

**Exact evidence:**

- `SH/mt/mt.go:39–45` defines wire version 1; lines 82–103 reject every other header version.
- `MT/crates/mt-codec/src/string_layer/header.rs:99–105` likewise rejects a version that does not match the codec's VERSION. `ME/crates/me-cli/src/sysw/mt.rs:80–82` requires a valid known-version header.
- `ME/crates/me-cli/src/sysw/mt.rs:95–113` distinguishes missing chunks, failure to reassemble, invalid transaction bytes, txid/set-ID mismatch, and unsigned inputs.
- `SH/gui/transaction.go:432–448` represents failure of `mt.Decode(set)` with `confirmed: false` and a substituted legend while preserving the strings. This includes ordinary version-1 chunks whose set does not confirm.
- `SH/gui/transaction.go:471–484` tests `tx.EveryInputSigned` for `tx:` warnings.
- `SH/mt/mt.go:136–147` defines that predicate in terms of a nonempty scriptSig or at least one witness item per input. `ME/crates/me-cli/src/sysw/tx.rs:20–24` explicitly distinguishes its structural parsing from script/signature validation.

**Consequence:** A producer using version zero to mark an unconfirmed backup creates an invalid chunk. A reviewer using the published rule misses unconfirmed version-1 sets. Treating the signed-input predicate as cryptographic validation would also overstate what these offline parsers establish.

**Correction:** Document confirmation as the result of set reassembly/transaction decoding and binding checks, independent of a new wire version. Name `EveryInputSigned` and its precise structural meaning; it is not blockchain confirmation or signature verification.

### H12 — P2: The preview command omits its required subcommand

**Document:** line 174; summary line 18 lacks the full invocation.

`me --preview <DIR> [--png]` is not the CLI surface defined by the source. These flags belong to `me bundle`.

**Exact evidence:**

- `ME/crates/me-cli/src/main.rs:47–54` defines the optional top-level subcommand and its `Bundle` variant.
- `ME/crates/me-cli/src/main.rs:61–71` declares `--preview` and `--png` inside that variant; `--png` requires `--preview`.
- `ME/crates/me-cli/src/preview.rs:111–195` implements the subsequent sidecar handshake/render behavior, consistent with the document's separate protocol description.

**Consequence:** The advertised invocation fails argument parsing before invoking the sidecar and produces no previews.

**Correction:** Show the command shape as `me bundle --preview DIR`, adding `--png` when requested and supplying the bundle input. The handshake and sidecar render protocol themselves were not found erroneous.

## Shared signing-recipe evidence for parent deduplication

**Document:** line 197. **Severity:** P2 if reported independently; not included as an additional finding in the count above.

The literal sequence `<dummy.pem> --no-overwrite` has the same shell-redirection defect the parent identified in GEMINI.md:44 with `<key.pem>`. Bash parses an input redirection from `dummy.pem` and an output redirection to a file named `--no-overwrite`. Neither the dummy-key pathname nor that flag is delivered as the intended picotool argument. If the input file exists, the output redirection may create or truncate that pathname before the command runs. `bash -n` does not reject this syntactically valid shell expression.

Treat this as the HOST_INTERFACES occurrence of the parent's shared command defect, not a second independent issue. The installed pinned picotool's read-only `help seal` output also does not list `--no-overwrite`; this review did not execute a seal operation to test parser acceptance or output behavior.

## Validation performed

Only read-only inspection, checked-in fixture hashing, and harmless targeted software probes were used. No full runtime suite or vector generator was run. The following tests passed in SH using `/scratch/code/shibboleth/.toolchain/go/bin/go`, with `GOPROXY=off`, `GOTOOLCHAIN=local`, and `GOFLAGS=-mod=readonly`:

```text
go test ./nfc/ndef -run '^(TestRecords|TestMessages)$' -count=1
go test ./sysw -run '^(TestConformance|TestComposerRecordsClassifyExactlyAsTheHost|TestCodex32SeamDeviceAdmitsEverythingTheHostDoes|TestPreimagePlateIsItsOwnClassAndOnlyUnderTheHashID)$' -count=1
go test ./txqr -run '^(TestStructuredAppendHeaderBits|TestSingleSymbolMatchesThePlainEncoder|TestEncodeSetBounds|TestOneSymbolIsByteModeToo)$' -count=1
go test ./gui -run '^(TestScan|TestSessionMarksIncompleteMtSetsUnconfirmed|TestAnUnsignedTxRecordIsFlaggedAndItsLegendReplaced)$' -count=1
```

Additional pure probes used synthetic in-memory records to check URI expansion, retention of embedded newlines, phrase/preimage classification, preimage length/index, and a valid 15-word mnemonic. The temporary probe source was outside the repositories and removed automatically. No real wallet material was read or printed.

The three fixture SHA-256 values cited at document lines 78, 79, and 103 matched the checked-in files; the classifier corpus contained the stated 82 rows. The sysw fixture length arithmetic demonstrated the separate tag. These matches and passing selected tests establish only those specific checks, not complete cross-language or firmware conformance.

No additional defect was established in the reviewed public-data-hash formulas, txqr byte-mode/Structured Append description, or preview version/render handshake. This is not an exhaustive certification of those interfaces. Rust implementations were inspected at the exact revisions; a Rust build/test suite was not run.

## Unverified questions and limits

- Physical RP2350 boot behavior, OTP state, board-specific profiles, and the claimed Scratch Y failure mechanism were not tested. H01–H03 are demonstrated source/document contradictions; they do not substitute for a hardware rehearsal or a provisioning approval.
- No flashing, signing, OTP access, hardware operation, live node access, vector regeneration, or full runtime suite was performed. The complete signing recipes were deliberately not executed.
- The presence/absence of a help entry alone is not a parser-execution test of `--no-overwrite`; only the shell-redirection defect is asserted independently for the shared recipe.
- Real NFC-tag capacity/interoperability and phone/QR-scanner behavior were not exercised. The NFC and QR conclusions concern the pinned software's framing, admission, and selected unit tests.
- No real secrets or SSH key contents were inspected. No reviewed production source or architecture document was edited. No findings are offered on the parent's other documents or completed workflow/startup/mechanical review.

The review stopped after consolidating confirmed findings at the controller's request. No speculative concern is promoted to a finding, and no additional exploration or full-suite execution is required to preserve these results.
