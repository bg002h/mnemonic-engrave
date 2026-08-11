# Implementation plan — systemwide payloads

Spec: `SPEC_systemwide_payloads.md` at `08d3239`, R0-closed. **This plan fixes
details; it restates no normative rule.** Where a rule is needed, it is
referenced (`[cliff]`, `[compared]`, spec §N) and defined only there.

Its one job is to make implementation **transcription rather than judgement**.
So every stage below names the files, the signatures, the fixtures and the
command that proves it green. Anything an implementer would otherwise have to
decide is a defect in this plan.

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

Stages 4 and 5 may proceed in parallel with 3 **only** after stage 1's vectors
exist; nothing in Go may lead.

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
cargo test -p me-cli sysw           # includes the vector round-trip
cargo clippy -p me-cli -- -D warnings
```

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
me sysw pack --no-passphrase 'text:...' | wc -c     # 65536
me sysw show f.bin                                   # prints the same digest as pack
me sysw pack --passphrase-words 2 <a secret record>  # warns, exit 0  (spec §13 D3)
```

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
| 5c | `inputWordsFlow` gains the per-invocation checksum switch and a `done` nav button in `Button2` (spec §8b, §8c) |
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

---

## What is NOT in this plan, deliberately

- **No normative rules.** They are in spec §12 and are referenced, never copied.
  Copying one here would recreate the multi-site problem six R0 rounds were
  spent removing.
- **No review round.** The spec absorbed six. This plan is sequencing and
  signatures; it gets the build gate and goes.
- **No test list.** Spec §8.3's 23 named tests are the list; stages map to them
  rather than restating them.
