# Implementation plan — systemwide payloads

Spec: `SPEC_systemwide_payloads.md` at `08d3239`, R0-closed. **This plan fixes
details; it restates no normative rule.** Where a rule is needed, it is
referenced (`[cliff]`, `[compared]`, spec §N) and defined only there.

Its one job is to make implementation **transcription rather than judgement**.
So every stage below names the files, the signatures, the fixtures and the
command that proves it green. Anything an implementer would otherwise have to
decide is a defect in this plan.

**FOLDED 2026-08-12**, against the journeys review
(`design/agent-reports/journeys-fable-specplan-review.md`). Stages 1–6 are
built; they stand below as the record of what they were, with corrections
marked. The review walked the operator journeys and found five that do not
close, every one traceable to a step no stage owned — F-144's shape, five more
times: this plan enumerated components and never the joins, and its own closing
note ("stages map to the 23 named tests rather than restating them") named a
mapping that was never written down anywhere. Stages 7–13 own the missing
halves — everything after `take()`, and every source other than typing and
flash. **The journeys section is the actual fix**: it states the operator
journeys and maps stages to them, so a step without an owner reads as a broken
sentence rather than an absent table row.

---

## Stage order, and the one constraint that forces it

| stage | repo | why here |
| --- | --- | --- |
| 1 | `mnemonic-engrave` (Rust) | **Rust-primary rule.** Spec §5.3 makes the container normative, so it lands in Rust with vectors before Go may exist |
| 2 | `mnemonic-engrave` (Rust) | `me sysw` CLI — needs stage 1's types |
| 3 | `seedhammer` (Go) | behaviour-faithful port of stages 1–2, vectors shared |
| 4 | `seedhammer` (Go) | device: region reader, session, admission |
| 5 | `seedhammer` (Go) | device: the eight programs' wiring |
| 6 | `seedhammer` (Go) | emulator NFC source, so the path is walkable |
| 7 | `mnemonic-engrave` (Rust) | `[mdmk-decode]` (spec §12.6) — normative flag behaviour, Rust with a vector first |
| 8 | `seedhammer` (Go) | `[mdmk-decode]` port + load-time flag wiring — may not lead stage 7 |
| 9 | `seedhammer` (Go) | §8c's count confirmation; Back becomes distinguishable from `done` |
| 10 | `seedhammer` (Go) | the CONSUMING half of NFC — stage 6 built only the source |
| 11 | `seedhammer` (Go) | the §5.3.2 erase item — the tree's first flash write; hardware-gated |
| 12 | `seedhammer` (Go) | §7's word-plate verify, with §7.1.1 provenance |
| 13 | `seedhammer` (Go) | carrier-ready admitted cells + the reconciliation tests that keep the map honest |

Stages 4 and 5 may proceed in parallel with 3 **only** after stage 1's vectors
exist; nothing in Go may lead. Same constraint one level up: stage 8 follows 7.
Stages 9–12 are mutually independent; 13 runs last, because its witness tests
reconcile everything the others produced.

---

## The journeys — the map that keeps a missing stage visible

**Added 2026-08-12.** F-144 happened because a stage list cannot show an absent
stage. A journey walked step by step can: **every step below names its owner,
and a step that names neither an owner nor a recorded reason is a plan
defect.** Journey letters follow the review.

| journey | steps → owner | state (2026-08-12) |
| --- | --- | --- |
| **J-A** plaintext, flash → any program | pack (st 2) → write region (spec §5.6's delivery command — UNREHEARSED) → boot/carousel load, digest compare, F1 (st 4–5 + the F-145 load flow) → consume (st 5a–5d) → engrave (pre-existing) | **closes** — driven end-to-end by the review |
| **J-B** sealed, generated words | pack (st 2) → word entry + `done` (st 5c) → count confirmation (**st 9**) → KDF → consume as J-A | open at one step: st 9 |
| **J-B′** sealed, user-supplied | RE-SCOPED by spec §13 D4: every token is now a wordlist word, so entry converges with J-B; the free-text keyboard journey is withdrawn, and `me` refuses what the device cannot type | closed by narrowing |
| **J-C** NFC → the eight programs | emulator source (st 6) → scanner parses `text:`/`pass:` (**st 10a**) → `Scanned` at the seam (**st 10b**) → routing to the two new consumers (**st 10c**) → F3/F4 at entry (**st 10d**) → consume | open: st 10 |
| **J-D** rid of a payload | host: `wipe` (st 2) → write (spec §5.6) → probe-false silence (st 4–5). Device: F1 warning → erase offer (**st 11**). The post-engrave reminder is WITHDRAWN (spec §13 D5) — deliberately not a step | host closes (driven); device open: st 11 |
| **J-E** a second payload | re-read → new identity → fresh compare (st 4–5 + load flow) | **closes** (driven) |
| **J-F** a class a program must NOT receive | per-site hard-coding (st 5) — closes negatively today, by convention; **st 13d** makes the convention a machine check | closes; hardening open: st 13d |
| **J-G** verify the plate just cut (spec §7) | menu → selection → typed comparison → provenance rendering — **st 12, wholesale** | open: st 12 |
| **J-H** consume every admitted cell | Mnemonic cells (st 5), FreeText (st 5d), Passph→BIP-39 Password (st 5d), MDMK→Bundle (st 5d) exist. Cdx32→Backup Wallet (**st 13a**); Passph→the four seam programs (**st 13b**); MDMK→Multisig (**st 13c**). Cdx32→the seam: **OPEN, no stage** — blocked on §3.1's seam type, recorded in spec §3.3.2. MDMK→Single-Sig: **OPEN, no stage** — no supplied-md1 carrier exists in that program (measured: the non-verify `bundleGatherFlow` sites are Multisig's and Bundle's) | partly open: st 13a–c, two recorded gaps |
| **J-I** a smuggled `md1` warns instead of hiding | `me sysw pack`/`show` warn (**st 7**) → device flags at load (**st 8**) — `[mdmk-decode]` | open: st 7–8 |

The review also asked for the reconciliation that would have caught this
table's absences mechanically; st 13d's witness test walks the device-owned
§8.3 tests against the gui tree — the level `assert_every_named_test_is_placed`
cannot see.

---

## Stage 1 — the container, in Rust

### Files

New module `crates/me-cli/src/sysw/`, mirroring `seal/`'s shape exactly:

| file | holds |
| --- | --- |
| `mod.rs` | `Payload`, `SyswError`, `pack`, `open` |
| `wire.rs` | `MAGIC`, `Header`, encode/decode |
| `record.rs` | `Class`, `classify`, the `text:` / `pass:` codecs |
| `pubhash.rs` | the displayed digest (spec `[digest-shown]`) |
| `identity.rs` | the 32-byte identity (spec `[identity]`) |
| `overwrite.rs` | the raw region image (spec §5.5) |

`seal/` is **not** modified. The two containers stay apart (spec §4).

### Signatures — transcribe these

```text
// sysw/wire.rs
pub const MAGIC: [u8; 8] = *b"MNEMSYSW";
pub const REGION_ADDR: u32 = 0x10D0_0000;
pub const REGION_LEN: usize = 65_536;

pub struct Header { pub iterations: u32, pub salt: [u8;16], pub iv: [u8;12],
                    pub pub_len: u32, pub ct_len: u32 }
impl Header { pub fn sealed(&self) -> bool; pub fn encode(&self) -> [u8; 52];
              pub fn parse(b: &[u8]) -> Result<Header, WireError>; }

// sysw/record.rs
pub enum Class { Mnemonic, Codex32Secret, Passphrase, FreeText,
                 Descriptor, MdMk, Address, Unknown }
impl Class { pub fn is_secret(&self) -> bool; }        // spec §3.3.1
pub fn classify(record: &str) -> Class;                 // prefixes BEFORE sniffers
pub fn encode_text(s: &str) -> String;                  // -> "text:<lowercase hex>"
pub fn encode_pass(s: &str) -> Zeroizing<String>;       // -> "pass:<lowercase hex>"
pub fn decode_body(record: &str) -> Result<Zeroizing<Vec<u8>>, RecordError>;

// sysw/pubhash.rs   label "MNEMSYSW/pub/v1", otherwise EPD§6.6 verbatim
pub fn public_data_hash(records: &[&str], sealed: bool) -> [u8; 16];
pub fn format_hash(h: &[u8; 16]) -> String;             // 8 groups of 4

// sysw/identity.rs  label "MNEMSYSW/id/v1"
pub fn identity(region: &[u8]) -> [u8; 32];

// sysw/overwrite.rs
pub enum Fill { Random, Zeros, Ones }
pub fn region_image(fill: Fill) -> Zeroizing<Vec<u8>>;  // exactly REGION_LEN bytes

// sysw/mod.rs
pub fn pack(records: Vec<String>, passphrase: Option<&str>, iterations: u32)
    -> Result<Vec<u8>, SyswError>;
pub fn open(blob: &[u8], passphrase: Option<&str>)
    -> Result<Payload, SyswError>;
pub fn cliff_above(normalised: &str) -> bool;           // spec [cliff]
```

**`cliff_above` lives in `sysw`, not in `seal`** — `seal`'s passphrase rules are
frozen (spec decision 1) and this is a different container's rule.

### Reuse rather than re-implement

| need | use |
| --- | --- |
| PBKDF2 + AES-256-GCM | `seal::crypto` unchanged |
| passphrase normalisation | `seal::passphrase::normalise` — spec §8a requires byte-identical KDF input |
| passphrase generation | `seal::passphrase::generate`, **extended to N words** |
| BIP-39 wordlist | the `bip39` crate already in `me-cli` |

`generate()` currently returns a fixed 12-word mnemonic. It becomes
`generate(n: usize)` drawing `n` words uniformly, **not** via
`Mnemonic::from_entropy_in` — spec §6.3: no checksum at any length.

### Vectors — extend, do not fork

Add to `crates/me-cli/testdata/seal_vectors.json` under a new `sysw` key. Minimum set:

| vector | covers |
| --- | --- |
| S-A | plaintext, one `text:` record — journey (b) |
| S-B | plaintext, secret class present — flag F1 |
| S-C | sealed, `pub_len > 0`, 12-word passphrase — digest shown |
| S-D | sealed, `pub_len == 0`, secrets only — **no digest**, `[compared]` by open |
| S-E | sealed, 2-word passphrase — F2 warns, still opens |
| S-F | S-C with one public record altered — AEAD fails (spec test 18) |
| S-G | `text:` body containing a space, a newline and a non-ASCII byte |
| S-H | region image, each `Fill` |

**S-G is the one that would be skipped and must not be**: it is the only vector
proving the hex encoding survives the exact characters EPD§6.4 forbids raw.

### Green

```sh
cargo test -p mnemonic-engrave sysw           # includes the vector round-trip
cargo clippy -p mnemonic-engrave --all-targets -- -D warnings
```

*(CORRECTED 2026-08-12: these lines said `-p me-cli`, which is the directory
name, not the package name — cargo rejects it. Executed as written above:
64 passed.)*

---

## Stage 2 — `me sysw`

`crates/me-cli/src/main.rs` gains one `Command::Sysw { .. }` arm with a nested
subcommand enum. Surface is spec §5.6 verbatim; nothing here restates it.

**Two details the spec leaves to the CLI layer:**

- `--passphrase-ask` reads from the tty via `rpassword`, never argv or env
  (spec §5.6 says why). If `rpassword` is not already a dependency, add it —
  do not hand-roll a tty read.
- the digest goes to **stderr**, the blob to stdout, so `me sysw pack > f.bin`
  works and still shows the operator their number.

### Green

```sh
me sysw pack --region --no-passphrase 'text:...' | wc -c   # 65536
me sysw show f.bin                                   # prints the same digest as pack
me sysw pack --passphrase-words 2 <a secret record>  # warns, exit 0  (spec §13 D3)
```

*(CORRECTED 2026-08-12: the first line lacked `--region` and prints 67 without
it — the review executed it both ways. The plan's own rule — commands get
executed — had not been applied to the plan.)*

---

## Stage 3 — the Go port

`seedhammer/sysw/`, mirroring the Rust module names one-for-one. Ports
`classify`, `decode_body`, `public_data_hash`, `identity`, `cliff_above`,
`Header`, and `open`. **No `pack`** — the device never creates payloads.

Conformance: a Go test reads the same `seal_vectors.json` (`sysw` key) the Rust
tests use. Vectors are the contract; the two implementations never compare to
each other, only to the file.

### Green

```sh
go test ./sysw/          # every vector, both directions
```

---

## Stage 4 — device plumbing

| item | where |
| --- | --- |
| region reader at `0x10D00000` | `sysw/read_tinygo.go` + `read_host.go`, exactly `seal`'s two-file split and its `clampRegion` pattern |
| session store | `gui/sysw_session.go` — the struct is spec §3.2.1, transcribed |
| admission | `gui/sysw_admit.go` — the table is spec §3.3.2, transcribed as a `map` |
| flags F1–F4 | evaluated in `sysw_admit.go`, rendered by the consuming screen |

**`gui/platform.go`'s `Platform` interface gains one method:**

```text
SyswReader() sysw.Reader     // nil when the platform has none, exactly as PayloadReader
```

nil is a supported value, not a stub — same contract as `PayloadReader` and for
the same reason.

---

## Stage 5 — the eight programs

**Order matters: do the shared seam first, then the four singletons.**

| step | change |
| --- | --- |
| 5a | `seedEntryFlow` gains the source picker; **`seedEntryFlowTypedOnly` added** (spec §3.1) |
| 5b | the two verify sites switch to `…TypedOnly` — 4 programs now done |
| 5c | `inputWordsFlow` gains the per-invocation checksum switch and a `done` nav button in `Button2` (spec §8b, §8c). **Under-transcribed (found 2026-08-12): §8c's count CONFIRMATION and a Back distinguishable from `done` were never staged — stage 9 owns them** |
| 5d | `backupWallet`'s `newInputFlow`, then BIP-39 Password, Engrave Text, Engrave Bundle |

`inputWordsFlow`'s signature changes — the one existing signature that must.

**CORRECTED 2026-08-11 by grep, before stage 5 rather than during it.** An
earlier draft said "every call site is in spec §3.1's table". That table
enumerates **`seedEntryFlow`**'s sites; `inputWordsFlow` is a different function
with different callers. Measured:

| callers | where |
| --- | --- |
| 5 non-test | `derive_xpub.go:90`, `seedxor_polish.go:52`, `unlock_kdf.go:160`, `gui.go:2346`, `gui.go:2445` |
| 8 test | `gui_test.go` ×8 |

**One of them is inside the frozen path.** `unlock_kdf.go:160` is the Sealed
Payload unlock, which decision 1 froze. Adding an options struct whose zero
value is today's behaviour touches it mechanically and changes nothing it does —
but say so at the call site, because "frozen" and "I edited a function it calls"
must not be reconciled by a future reader guessing.

All five non-test sites are seed or passphrase entry that WANT the checksum, so
every existing site passes `checksumGate: true`.

```text
func inputWordsFlow(ctx *Context, th *Colors, mnemonic bip39.Mnemonic,
                    selected int, title string, opt wordEntryOpts) (n int, ok bool)

type wordEntryOpts struct {
    checksumGate bool   // true for seed entry, false for passphrase entry
    terminator   bool   // draws `done` in Button2
}
```

It returns `n` because spec §2.2 item 8 records that its having **no return
value** is one of the five obstacles.

### Green

```sh
go test ./gui/ ./sysw/
GOOS=js GOARCH=wasm go build ./cmd/emu/
go build -tags tinygo ./gui/
```

---

## Stage 6 — emulator NFC source

`cmd/emu/platform.go`'s `NFCReader()` returns nil today. Give it a source fed
from a JS global, the mechanism that file already sketches:
`window.shNFC = "<record>"`. Read it in `nfc_js.go`; keep the untagged/`js`
split `toolpath.go` uses so the decode half stays host-testable.

Without this the emulator cannot walk any NFC journey, which is spec §8.2.

**Stage 6 built the SOURCE only (found 2026-08-12).** No stage owned the
consuming half — the §3.1 `Scanned` option, the scanner cases for the two new
record forms, F3/F4 — which is how J-C stayed partial through a green gate.
Stage 10 owns it now.

---

## Stage 7 — `[mdmk-decode]`, in Rust (spec §12.6; §13 D6)

Rust-primary: normative classification/flag behaviour lands here with a vector
before the Go port may exist — stage 1's constraint, one level up.

### Files

| file | change |
| --- | --- |
| `crates/me-cli/src/sysw/record.rs` | the confirmation walk |
| `crates/me-cli/src/main.rs` | `pack` warns per unconfirmed record; `show` prints per-record confirmation |
| `crates/me-cli/src/sysw/coverage.rs` | test 14 re-pointed to `Vector("S-J")` — off the false `S-I` placement the review found |
| `crates/me-cli/testdata/sysw_vectors.json` | vector S-J |

### Signatures — transcribe these

```text
// sysw/record.rs
/// Indices of ClassMDMK records that are NOT decode-confirmed (spec §12.6).
/// Groups by (hrp, chunk_set_id) exactly as seal::record::decode_public_set
/// does (crates/me-cli/src/seal/record.rs:192) — R1-I2: filter the iteration,
/// never the indices — but REPORTS instead of refusing: an incomplete or
/// non-decoding group marks its members unconfirmed and returns.
pub fn mdmk_unconfirmed(records: &[String]) -> Vec<usize>;
```

`me sysw pack` prints one line per unconfirmed record — `record 3: an md1/mk1
this tool could not decode; the device will treat it as a SECRET` — and
proceeds (nothing refuses, spec §13 D6). `me sysw show` prints `confirmed` /
`unconfirmed` beside each `ClassMDMK` record.

### Vector

S-J: a single chunk of a declared multi-chunk `md1` set — real cards, so the
reassembler is the arbiter, not a hand-built fixture. Expected: classifies
`ClassMDMK`, unconfirmed. The decode-FAILURE arm (BCH-valid, complete,
reassembles, does not decode) is a Rust unit test against the codec directly:
building such a card needs the encoder, and the vector file carries only
inputs `pack` accepts.

### Green

```sh
cargo test -p mnemonic-engrave sysw
cargo clippy -p mnemonic-engrave --all-targets -- -D warnings
```

---

## Stage 8 — `[mdmk-decode]` on the device

Port of stage 7, plus the flag wiring. May not lead stage 7.

| item | where |
| --- | --- |
| the confirmation walk, same grouping, over `md.Reassemble` and the `mk` decoder | `sysw/` (new `confirm.go`, mirroring the Rust module split) |
| `syswRecord` gains `unconfirmed bool`, set ONCE at `load` — classification is at-load (spec §3.2.1) and this rides with it | `gui/sysw_session.go` |
| flag evaluation reads secrecy through it: `syswFlags(c sysw.Class, unconfirmed bool, src syswSource, sealed, weak bool)` — the one existing signature this stage changes — and `syswLoadWarnings` names the case distinctly (“an md1/mk1 the device could not confirm — treated as a secret”) | `gui/sysw_admit.go`, `gui/sysw_load.go` |
| conformance: S-J through the shared vectors file, both directions | `sysw/conformance_test.go` |

### Green

```sh
go test ./sysw/ ./gui/
go build -tags tinygo ./gui/ && GOOS=js GOARCH=wasm go build ./cmd/emu/
```

---

## Stage 9 — §8c's confirmation, and Back ≠ `done`

The truncation trap is live: in `inputWordsFlow`, Back and `done` both
`return entered()` (`gui/gui.go:841` and `:846`), so backing out with three
words filled is indistinguishable from finishing at three — the load flow runs
the ~31 s KDF on a passphrase the operator meant to abandon, and the failure
reads as "wrong passphrase". Spec §8c exists verbatim to prevent this; test 22
is its test; stage 5c delivered the button and under-transcribed the rest.
Worth naming: stage 5's own signature block already promised
`(n int, ok bool)`, the implementation shipped bare `int`, and nothing caught
the dropped return — the review did, by walking the journey.

| change | where |
| --- | --- |
| `func inputWordsFlow(...) (n int, done bool)` — Back returns `done == false`. The five existing non-test callers invoke it in statement position and need no edit; only the load flow reads the returns | `gui/gui.go:769` |
| on `done` with `n > 0`: a `ChoiceScreen` — Title `"Passphrase"`, Lead `"N words — unlock?"`, Choices `{"BACK", "UNLOCK"}` — BEFORE the KDF; BACK re-enters entry with the slots intact | `gui/sysw_load.go` (today the KDF follows entry immediately) |
| on Back out of entry: abort the load — no KDF, no error screen | same |
| flip test 22 `DeviceUnbuilt` → `Device` | `crates/me-cli/src/sysw/coverage.rs` |

### Green

```sh
go test ./gui/ -run Sysw     # incl. test 22: done at 3 of an intended 12 confirms "3 words"
go build -tags tinygo ./gui/ && GOOS=js GOARCH=wasm go build ./cmd/emu/
```

---

## Stage 10 — the CONSUMING half of NFC

Spec §3.1 is NORMATIVE — `seedEntryFlow … offers Typed / Scanned / Payload` —
and the code offers two of the three. §2.1 made NFC-for-everything a
deliberate, emphasised capability; no stage ever sequenced the consuming half
(review J-C, Important-3). Four steps, one stage, because they only close the
journey together:

| step | change | where |
| --- | --- | --- |
| 10a | the scanner parses the two new record forms: `text:`/`pass:` prefixes checked FIRST (spec §5.3.1's order — prefixes before sniffers), bodies hex-decoded through the `sysw` codecs; new scan types `freeTextScan` / `passScan` (a secret) | `gui/scan.go` — today both fall to `errScanUnknownFormat` |
| 10b | the seam offers Scanned: `syswSeedPicker`'s picker becomes `{"TYPE IT", "SCAN", "FROM PAYLOAD"}` (the payload row only when the session holds a seed, as now); SCAN polls `ctx.Platform.NFCReader()` with the `md1GatherFlow` goroutine pattern (`gui/md1_gather.go:87`) and accepts a `bip39.Mnemonic` only | `gui/derive_xpub.go:126` |
| 10c | `engraveObjectFlow` routes the two new types; Engrave Text and BIP-39 Password gain object-accepting entries — `engraveTextFlowFrom(ctx, th, body string, src syswSource)`, `engravePassphraseFlowFrom(...)` — and the existing no-argument flows become their `srcTyped` wrappers | `gui/gui.go:2213`, `gui/freetext_flow.go`, `gui/passphrase_flow.go` |
| 10d | F3/F4 finally fire: the acceptance screen of every non-typed entry renders the source line (F3, spec §3.2 as scoped 2026-08-12), and the 10b/10c sites construct `srcNFC` — its first production construction — so `syswFlags` yields F4 for a scanned secret | the 10b/10c acceptance points; `gui/sysw_admit.go` itself is unchanged — the rules exist, unfired |

What 10b must NOT touch: `seedEntryFlowTypedOnly` and the two `*_verify.go`
callers — test 16's structural guarantee. A scanned seed at a verify prompt is
§7.4's self-comparison with extra steps.

### Green

```sh
go test ./gui/ ./sysw/
GOOS=js GOARCH=wasm go build ./cmd/emu/   # then shNFC.present("<record>") walks J-C in the browser
go build -tags tinygo ./gui/
```

---

## Stage 11 — UNLOAD (spec §13 D10; NOT a flash write)

**Rewritten 2026-08-12 by operator ruling D10, which deleted this stage's
reason for being late and dangerous.** It previously added an `Eraser`, an
RP2350 flash-range erase behind interrupt masking, a `SyswEraser()` on
`Platform`, a hardware rehearsal and a RISK-SET classification — because it was
to be the tree's first flash write. **The firmware now never writes flash.**
All of that is gone. What remains is a session operation.

The operator may UNLOAD: the loaded records are dropped, and the region at
`0x10D00000` is untouched. **The word "erase" does not appear on the device**,
because the bytes are still there and saying otherwise would be a lie the
operator might act on. Overwriting the region stays a HOST operation
(`me sysw wipe`), and the unload screen says so.

| item | where |
| --- | --- |
| `func syswUnloadFlow(ctx *Context, th *Colors) bool` — confirm (UNLOAD / BACK), drop the session, then a result screen that states plainly what did and did not happen: *"Payload unloaded. It is still in flash — overwrite it from the host with `me sysw wipe`."* | new `gui/sysw_unload.go` |
| **The CONFIRM screen states what RELOADING will cost, not just what unloading does** (operator ruling 2026-08-12). Reload is one carousel entry away — `Load Payload` is unconditional and `syswLoadFlow` assigns a fresh session — but `[compared]` (§12.2) must be re-earned every time, so a SEALED payload costs a full passphrase entry and its ~31 s KDF, and an unsealed one a fresh digest comparison. Word it from the loaded payload's own state: sealed → *"You can load it again from the menu. You will need the passphrase."*; unsealed → *"You can load it again from the menu. You will need to compare the digest again."* Someone will unload by accident, and the screen that told them the cost beforehand is the difference between a shrug and a hunt for the passphrase | `gui/sysw_unload.go` |
| `ctx.sysw = nil`, nothing else. No `Eraser`, no `Platform` method, no `_tinygo.go` file, no flash call anywhere | `gui/sysw_unload.go` |
| offered from the `loadPayload` carousel entry when a payload is LOADED (UNLOAD / BACK), and alongside the F1 warning where an ERASE choice was planned | `gui/sysw_load.go` |

**No hardware gate, and that is the point of the ruling.** Nothing here is
irreversible: the flash is not touched, and a session dies at power-off anyway
(§3.2.1). This stage carries the same risk as any other menu item.

**Do not** add `sysw/erase.go`, `sysw/erase_tinygo.go`, `sysw/erase_host.go`, or
`SyswEraser()`. If a later cycle wants device-side overwrite it re-earns a
ruling; D10 is not a deferral, it is a decision.

### Green

```sh
go test ./gui/ ./sysw/
go build -tags tinygo ./gui/ && GOOS=js GOARCH=wasm go build ./cmd/emu/
grep -rn "Erase\|erase" gui/ sysw/ | grep -v _test   # must find no flash-write path
```

A test must assert the confirm screen names the passphrase when the loaded
payload is SEALED and does not when it is not — the wording is the feature here,
so an assertion that only checks the screen appeared would pass on silence.

---

## Stage 12 — §7: the word-plate verify, with provenance

Spec §7 has no owner and no implementation (review J-G — the largest
F-144-shaped hole: an operator-ruled, normative section, invisible to
stage-vs-stage checking because no stage claimed it). Scope per §7.2's
2026-08-12 note: the WORD-PLATE verify — Backup Wallet's mnemonic plates. The
bundle verifies stay exactly as they are.

### Files and signatures

The file is named `plate_verify.go` DELIBERATELY: the test-16 structural test
(`gui/sysw_verify_test.go`) scans every `*_verify.go` file for the
`seedEntryFlow` identifier, so the new flow sits under that guarantee from its
first line. It also names neither `syswOffer` nor `take`, and stage 13d's
oracle test enumerates every such call site — a future addition here fails it.

```text
// gui/plate_verify.go
type verifyProvenance int

const (
    provDeviceComparedAll verifyProvenance = iota  // renders "device-compared (every word)"
    provDeviceComparedSubset                       // renders "device-compared (N of M)"
    provOperatorAsserted                           // renders "operator-asserted"
    provNotVerified                                // renders "not verified"
)

// plateVerifyFlow offers the spec §7.2 menu (all six rows, labels verbatim,
// incl. `skip`), draws positions per §7.2.1 (even/odd 1-indexed; 6/3 without
// replacement, uniform, FRESH per attempt, from crypto/rand — TRNG-backed on
// device), prompts each drawn position on the word keyboard
// (checksumGate false — a subset has no checksum), compares against m, and
// returns the outcome with its provenance.
func plateVerifyFlow(ctx *Context, th *Colors, m bip39.Mnemonic) verifyProvenance
```

The outcome screen renders spec §7.1.1's strings, and nothing renders the bare
word "verified" — test 17 asserts over the rendered strings, not the enum. A
failure names the mismatched positions and offers retry, which re-draws
(§7.2.1's fresh-draw rule; test 3 pins it, with uniformity over many draws).

### Integration

Offered at the end of `backupWalletFlow` (the mnemonic word-plate program),
after the engrave completes, regardless of source — spec §7.1: there is no
gate the device can honestly evaluate. The typed side is operator keystrokes
from the plate; the baseline is the just-engraved mnemonic, still in scope at
that call site — no session read, which is what test 1 pins structurally.

Flip tests 2, 3, 17 `DeviceUnbuilt` → `Device` in
`crates/me-cli/src/sysw/coverage.rs`.

### Green

```sh
go test ./gui/ -run 'PlateVerify|Provenance'
go test ./gui/                   # incl. the structural passes over the new file
go build -tags tinygo ./gui/ && GOOS=js GOARCH=wasm go build ./cmd/emu/
```

---

## Stage 13 — close the record: carrier-ready cells, and the reconciliation tests

### 13a–13c — admitted cells that already have carriers (spec §3.3.2's reachability note)

| cell | change | where |
| --- | --- | --- |
| 13a `Cdx32 → Backup Wallet` | `newInputFlow` gains `syswOffer(sysw.ClassCodex32Secret, "Seed from where?")` beside its mnemonic offer; the body goes through `codex32.New` and returns as `codex32.String`, the typed M*1 menu's own object | `gui/gui.go:2415` |
| 13b `Passph → the four seam programs` | `passphraseFlow` gains `syswOffer(sysw.ClassPassphrase, "Password from where?")` at its head — the optional-passphrase step the spec's cell reasons name, one edit serving all four callers | `gui/gui.go:654` |
| 13c `MDMK → Multisig` | the supplied-md1 gathers seed from the payload exactly as `bundleFlow` does — the once-before-gathering offer (`gui/bundle_flow.go:29`, `ctx.syswBundleSeed`) | the two non-verify `bundleGatherFlow` sites: `gui/multisig.go:67`, `gui/multisig_build.go:47`. **NOT** `singlesig_verify.go:103` / `multisig_verify.go:69` — a verify readback must come from the plate's own cards (spec §7.4's reasoning; say so at the site) |

Two cells stay OPEN, recorded in the journey map (J-H): `Cdx32 → the seam`
(spec §3.1's seam type cannot carry it — the inconsistency spec §3.3.2 records)
and `MDMK → Single-Sig` (no supplied-md1 carrier exists in that program;
measured above). Open cells fail closed — a permission without a path admits
nothing.

### 13d — the tests that keep the table and the map honest

| test | what it pins |
| --- | --- |
| `gui/sysw_admit_oracle_test.go` | walks every non-test `syswOffer(...)` / `.take(...)` call site by AST (the `sysw_verify_test.go` pattern), maps site → program through an in-test table, and asserts each named class against `admitted` — spec §13 D7's mechanism. A new consumption site fails until it appears in the map with an admitted class |
| `gui/sysw_coverage_witness_test.go` | for each spec §8.3 test the coverage map calls device-owned (1, 2, 3, 8, 10, 16, 17, 21, 22), names the Go test function that discharges it and asserts by AST that the function exists in the package — the reconciliation the review's closing paragraph asked for, one level above `assert_every_named_test_is_placed`. Run against the pre-fold tree it names 2, 3, 17 and 22 as missing, which is the review's finding reproduced as a command |

### Green

```sh
go test ./gui/
go build -tags tinygo ./gui/ && GOOS=js GOARCH=wasm go build ./cmd/emu/
```

---

## What is NOT in this plan, deliberately

- **No normative rules.** They are in spec §12 and are referenced, never copied.
  Copying one here would recreate the multi-site problem six R0 rounds were
  spent removing.
- **No review round.** The spec absorbed six. This plan is sequencing and
  signatures; it gets the build gate and goes. *(2026-08-12: the
  post-implementation journeys review happened anyway — the risk-set rule
  requires it — and stages 7–13 plus the journey map are its fold. The bullet
  stands for what it meant: the plan itself does not re-enter R0.)*
- **No test list.** Spec §8.3's 23 named tests are the list; stages map to them
  rather than restating them. *(2026-08-12: "stages map to them" was a mapping
  nobody had written down, and that is precisely how §7 went unowned. The
  journey map above and stage 13d's witness test are that mapping, written
  down and then made a command.)*
