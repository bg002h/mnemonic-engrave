# Decisions made by Claude — systemwide payloads

Every choice **I** made while implementing `SPEC_systemwide_payloads.md`, with
the reason, so a later reader can overturn one on purpose rather than by
accident.

**Operator rulings are NOT here.** They live in the spec — §1 (decisions 1–9)
and §13 (the demotions) — and duplicating them is how two sources of one rule
start to disagree, which is the failure six R0 rounds were spent removing. Where
a decision below is downstream of a ruling, it cites it.

Scope: `mnemonic-engrave` branch `sysw-container`, fork branch `sysw-port`.

---

## 1. Container and encoding

| # | Decision | Why |
| --- | --- | --- |
| 1.1 | `sysw/` mirrors `seal/`'s module shape one-for-one; **`seal/` is not modified** | The operator froze Sealed Payload (§1 decision 1). Widening the format it depends on would unfreeze it through the back door. |
| 1.2 | Magic is **8 bytes, `MNEMSYSW`** | Matches `MNEMBLOB`'s width, so both containers present a same-width discriminator at offset 0. A reader finding either magic at the wrong address refuses rather than half-parsing. |
| 1.3 | The header is EPD §6's layout **byte for byte** | One decoder's bounds reasoning then transfers to the other. Only the magic differs. |
| 1.4 | `text:` / `pass:` bodies are **lowercase hex** | EPD §6.6 hashes records in canonical LOWERCASE form. Hex is the only common encoding that survives lowercasing unchanged — base64 and base32 do not. Cost is 2×, against a cap of 8191, so nothing real binds. |
| 1.5 | **Uppercase hex is rejected**, not accepted-and-lowercased | §6.6 hashes the record as it appears on the wire, so two spellings of one body would be two different digests. |
| 1.6 | Reserved prefixes **fail closed**: `text:`/`pass:` with a bad body is `Unknown`, never "probably text" | A malformed record must not become an engraved plate. |
| 1.7 | `ClassFreeText` is **not** secret | A class states what the format *guarantees*, not what a human might put in it. A class claiming secrecy it cannot enforce is the over-claim F-123 was filed against. |
| 1.8 | **Descriptor and Address are deliberately unclassifiable**, refused with the record's index | Classifying them needs a descriptor parser and an address decoder, neither a dependency of this crate. A named refusal at creation beats a mis-filed secret. **Revisit if either dependency arrives.** |
| 1.9 | Digest label is **`MNEMSYSW/pub/v1`**, not `MNEMBLOB`'s | The label exists to stop cross-context collisions; reusing it would make two containers the spec separates produce identical digests over identical sections. |
| 1.10 | `[identity]` is **SHA-256 over the region bytes, 32 bytes, untruncated** | The §6.6 digest does not exist at `pub_len == 0`, so using it would give every secrets-only payload one shared identity and let a swapped payload inherit `[compared]`. It is an equality key never read by a human, so truncating it only invites collisions. |
| 1.11 | The overwrite artefact is a **raw region image**, not a container | As a container it would be capped at 16,450 of 65,536 bytes, making "fills the region" false. It carries no magic and is unparseable by design. |
| 1.12 | AAD is taken from the **assembled bytes**, never re-encoded | What is bound is then exactly what a reader will parse. |
| 1.13 | **One implementation of `pack`**; the public entry point only chooses randomness | Two assemblers diverged within one commit and `pack_deterministic` silently dropped secret records. Same defect shape as the §12 single-source restructure, applied to functions. |

## 2. Rust CLI

| # | Decision | Why |
| --- | --- | --- |
| 2.1 | Subcommands live under **`me sysw`**, not beside `seal` | No invocation should be able to produce a systemwide container while the operator believes they are producing a Sealed Payload one. |
| 2.2 | Blob → **stdout**, digest → **stderr** | `me sysw pack > f.bin` must still show the operator the number they compare on the machine. |
| 2.3 | Omitting every passphrase flag **generates** one | The default must not be to leave a payload unprotected through omission. |
| 2.4 | `--allow-weak` is **accepted and ignored, with a message** | §13 D3 demoted the refusal. Silently dropping the flag would break existing invocations for no gain. |
| 2.5 | `rpassword` for the tty rather than a hand-rolled read | argv is world-readable via `/proc` and lands in shell history that outlives the machine. |
| 2.6 | Structural failures say **"not a systemwide container"**, never "payload unreadable" | EPD §2.2 item 4 trains operators to read that exact phrase as *tampering*; a wrong file is not an attack. |
| 2.7 | `sysw::passphrase::generate` is **separate from** `seal`'s, but normalisation is **shared** | `seal`'s emits a checksummed mnemonic and is frozen; this draws N words with no checksum. Normalisation must stay single, because §8a requires host and device to agree byte for byte. |
| 2.8 | `i32` exit codes | Matches the crate's existing convention rather than introducing a second. |

## 3. Go port

| # | Decision | Why |
| --- | --- | --- |
| 3.1 | **No `Pack` in Go** | The device never creates a payload. Omitting it removes any possibility of the device disagreeing with the host about how to build a container it should never build. |
| 3.2 | Crypto is `seal`'s, unchanged | Two AES-GCM implementations in one firmware is two things that can disagree. |
| 3.3 | Vectors are read from the **sibling checkout** via `SYSW_VECTORS`, and `SYSW_REQUIRE_VECTORS=1` makes a missing file a **hard failure** | Copying the file into the fork would let the two copies drift. A differential oracle that silently no-ops reads exactly like one that passes — the fork's NDEF harness already learned this with `ME_REQUIRE_GO=1`. |
| 3.4 | A test guards the **vector set itself** | Conformance over easy cases proves little, so the set must keep covering both container variants, the `pub_len == 0` case, and an encoded record. |

## 4. Device plumbing

| # | Decision | Why |
| --- | --- | --- |
| 4.1 | Reader split **untagged / tinygo / host**, mirroring `seal` | `clampRegion` and `boundBlob` are called by both readers; a bound living only in the tinygo file is never compiled by the test runner. |
| 4.2 | The session holds **one** payload; loading a second replaces it | "Which payload did this record come from" has exactly one safe answer when there is only ever one. |
| 4.3 | Lifetime is the **process**; no flow clears it | A flow that cleared it would silently reintroduce the per-program KDF that once-per-session exists to avoid (§1 decision 5). |
| 4.4 | Records are classified **once, at load** | Re-sniffing at use would let one byte string be admitted as one class and consumed as another. |
| 4.5 | `take()` is gated on `[compared]`; `has()` is not | A menu must be able to offer "from payload" before the operator has compared anything. |
| 4.6 | Admission is **`(class → program)`**; the container selects **flags** | The obvious three-axis matrix has a redundant axis: §1 decision 6 lets the plaintext variant carry any class the sealed one may. Two rules, each testable alone. |
| 4.7 | **Source is a flag input, not an admission input** | Otherwise the NFC path — the one §5.4 removed all integrity checking from — escapes the single admission function. |
| 4.8 | **No flag refuses anything** | A flag that grew a refusal would re-impose what §13 demoted. Asserted by a test. |
| 4.9 | `SyswReader()` returns nil on platforms without a region | nil is a supported value, not a stub — the same contract `PayloadReader` has. |

## 5. The programs

| # | Decision | Why |
| --- | --- | --- |
| 5.1 | **Two entry points** (`seedEntryFlow`, `seedEntryFlowTypedOnly`), not a boolean parameter | §7.4 forbids a payload secret reaching a verification, because a verify that accepted the engrave's own secret compares it against itself and passes unconditionally — certifying a **wrong plate**. A boolean can be passed wrongly and still compile; a flow with no way to *name* the payload source cannot reach it by any argument. |
| 5.2 | The verify guarantee is tested **structurally**, by parsing the AST | R0-C1 showed a behavioural test could be satisfied at the session layer while the UI still offered the option. The match is on the identifier, not a substring — `seedEntryFlowTypedOnly` contains `seedEntryFlow`. |
| 5.3 | The checksum gate is **per-invocation** | With it on, `refreshCands` masks the final slot to 128 of 2048 candidates, so 15 of every 16 generated 12-word passphrases would be permanently unopenable (R1-C2). Seed entry keeps it; passphrase entry must not have it. |
| 5.4 | `inputWordsFlow` **returns the entered count** | Spec §2.2 item 8 records its having no return value as one of five obstacles to arbitrary-N entry: without it a caller cannot tell a 7-word passphrase from a 24-word one abandoned early. |
| 5.5 | `done` is **`Button2`, a nav button**, not a keyboard key | The word path builds `NewKeyboard`, which has no opt-in parameter, and its `rune()` would feed a `done` rune into `Fragment` and **panic** in `bip39.ClosestWord`. `Update` filters only arrows, Center and runes, so a nav slot structurally cannot reach `Fragment`. |
| 5.6 | `syswOffer` returns false when there is nothing to offer *or* the operator declines | Keeps the payload strictly **additive**: a machine with no payload behaves exactly as before. |
| 5.7 | Engrave Text **pre-fills**, never bypasses | A source that returned straight to the engraver would skip title, footer, size and confirm — a plate nobody had seen. |
| 5.8 | Engrave Bundle offers **one** card, entering through the same `offer()` a scanned card takes | A bundle is a *set*; short-circuiting the gather would cap it at whatever the payload held. A separate insertion path would be a second way into a bundle with only one of them checked. |
| 5.9 | BIP-39 Password copies into the **caller-owned buffer** `wipeBytes` scrubs, bounded | Never a Go string. An over-long record truncates rather than overruns. |

## 6. Emulator

| # | Decision | Why |
| --- | --- | --- |
| 6.1 | The NFC source is **one-shot** | A real tag crosses the reader once. A source that replayed forever would let a polling flow see a tag never presented — a behaviour the machine does not have. |
| 6.2 | `shNFC.present()` / `.clear()` are **functions**, not a bare string the page assigns | No physical setup leaves a tag permanently present. |
| 6.3 | `nfc.go` untagged, `nfc_js.go` tagged | Same split `toolpath.go` uses: the js half cannot be tested on this host, so everything that can be is kept out of it. |

## 7. Process — how the work was checked

| # | Decision | Why |
| --- | --- | --- |
| 7.1 | **`cargo-mutants` over hand-written mutants** | Mine scored 8/9 then 9/9 and I reported both as strong; the generator then found **13 holes** I had not, including a function with no test at all. My mutants come from the same mental model that wrote the code, so they probe what I was thinking about, not what I forgot. |
| 7.2 | A surviving mutant is a **question**, not automatically a gap | One of four survivors was genuinely equivalent (`\|` vs `^` on non-overlapping nibbles, verified exhaustively over 256 inputs). Fixing it would have been waste; documenting it stops the next run re-litigating it. |
| 7.3 | `scripts/spec-check.py` **forbids the bare term** rather than recognising definitions | A regex over "definitional phrasings" was measured at 1 kill in 5 — it can only catch phrasings its author imagined. Forbidding the word makes restating a rule *unwritable*. |
| 7.4 | The mutation harness **asserts the mutation applied** before judging | An anchor swept away by an earlier edit meant no mutant ever applied and all seven reported "survived". A harness that fails to run reads exactly like a subject that fails to fail. |
| 7.5 | The plan's code-facing claims are **grepped before each stage** | The plan cited the wrong function's call sites; one minute of grep found it before stage 5 rather than during. |
| 7.6 | **Unrelated `cargo fmt` churn was reverted**, not swept in | `cargo fmt` wanted to reformat six files this work never touched; that is a separate change and belongs in its own commit. |
| 7.7 | Exhaustive AAD coverage runs at the **crypto layer**, with one integration test tying it to `open` | `open` derives a key per call, so a byte-by-byte sweep through it would pay a 100,000-round KDF per position. Exhaustive where it is cheap, integrated where it matters. |
| 7.8 | Tests that could pass by luck **draw repeatedly** | A single generated passphrase satisfies the checksum test 1 time in 16, which is exactly how R1-C2 survived a review round. 32 draws per length. |

---

## Known-open, and deliberately so

- **CI must set `SYSW_VECTORS` and `SYSW_REQUIRE_VECTORS=1`.** Without them the
  cross-language check silently skips — the failure the flag exists to prevent.
- **The submodule pin in `mnemonic-engrave` must move** once the fork branch
  lands, or `firmware/` harnesses cannot see `sysw`.
- **Descriptor and Address remain unclassifiable** (1.8), so `me sysw pack`
  refuses them with a named error. Spec §3.3.2 admits them to Engrave Bundle and
  Multisig, so this is a real gap between spec and implementation — closing it
  needs a descriptor parser and an address decoder as dependencies.
- **F-125** schedules the EPD and `passphrase.rs` amendments the restored
  user-supplied mode requires, before implementation is called done.
