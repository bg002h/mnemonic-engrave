# Exit-code survey: `mk` (mk-cli) and `ms` (ms-cli)

Surveyor: mk-ms. Read-only. Binaries: `/home/user/wt/target/debug/{mk,ms}`
(`mk 0.18.0-dev`, `ms 0.21.0`). Source heads: mnemonic-key `6957f37`,
mnemonic-secret `e131854`. Paths below are relative to `/home/user/wt/`;
`K/` = `mnemonic-key/crates/mk-cli/`, `S/` = `mnemonic-secret/crates/ms-cli/`.
Every run used stdin from `/dev/null` unless a redirect is shown. The full
transcript of every run (command, rc, head of stdout/stderr) is
`/home/user/wt/ec/tmp-mk-ms/runs.log`; input fixtures are beside it.

Inputs: `mk vectors` / `ms vectors` corpus entries (V1, V2, N1-N21; the
all-zero "abandon ... about" vector), one-character substitutions of them, and
codex32 strings I checksummed myself for the cases no vector covers (tag `seed`,
tag `zzzz`, prefix byte 0x01, HRP `mq` with a valid checksum).

**Caveats on what was measured**
- All runs were as **root**, so "unreadable file" and "read-only directory"
  cases could not be produced (root reads a mode-000 file and writes into a
  0555 directory). Those rows are "source only".
- Panics were not triggered. 101 is inferred (neither `main` uses
  `catch_unwind`; neither `Cargo.toml` sets `panic = "abort"`).
- **Side effect to clean up:** `mk vectors --out /nonexistent-dir/y` exited 0
  and *created* `/nonexistent-dir/y` (41 files) because `vectors --out` does
  `create_dir_all`. I could not remove it (the removal was blocked by the
  harness). Another surveyor's file `/nonexistent-dir/x.md1` appeared in that
  directory seconds later, so **any other surveyor's "missing directory" test
  that used `/nonexistent-dir` after 05:37 may have falsely succeeded**. Please
  delete `/nonexistent-dir` and treat such results as suspect.

---

## A. Exit-code inventory

### A.0 How `main` maps outcomes to codes (both binaries share the shape)

Both: `fn main() -> ExitCode`; no `std::process::exit` anywhere in either
`src/`; no `catch_unwind`; clap is driven with `try_parse` and **never**
`Error::exit()`. Each verb returns `Result<u8, CliError>`: `Ok(code)` is the
verb's own code (0, or the repair codes), `Err(e)` goes through
`CliError::exit_code()`. A final filter adjusts the code if a stdout write
failed.

| step | `mk` | `ms` |
|---|---|---|
| pre-clap gate | none | `S/src/main.rs:198-211` non-UTF-8 argv -> 64; `:218-228` argv guard: `Refuse` -> **1**, `Usage` -> 64 |
| clap error | `K/src/main.rs:83-88`: DisplayHelp/DisplayVersion -> 0 (through the stdout filter), every other clap error -> **64** | `S/src/main.rs:248-253`, same |
| verb result | `K/src/main.rs:107-114` | `S/src/main.rs:275-288` |
| error enum -> code | `K/src/error.rs:135-145` | `S/src/error.rs:60-73` |
| stdout-failure filter | `K/src/out.rs:76-83`: on a non-EPIPE stdout write error, codes **0 and 5** become 1; any other code is kept | `S/src/sink.rs:92-103`: an `Ok` return (0 or repair's 4) becomes 1; an `Err` code is kept |
| JSON error envelope | stdout, carries `exit_code`; `K/src/main.rs:132-141` | stdout, carries `exit_code`; `S/src/main.rs:308-341` (two errors are stderr-only in both modes: `OutputWrite`, `StdoutIsOut`) |

`mk` `CliError::exit_code` (`K/src/error.rs:135`):

```
Codec(UnsupportedVersion) => 3      Codec(_) | MdCodec(_) => 1
FutureFormat(_) => 3 (unconstructed) ContentMismatch => 4
UsageError(_) => 64                  IoError(_) => 1
SetReassemblyMismatch => 2
```

`ms` `CliError::exit_code` (`S/src/error.rs:60`):

```
BadInput | Bip39 | Codex32 | UnexpectedStringLength | PayloadLengthMismatch => 1
Usage | StdoutIsOut => 64            OutputWrite => 1
FormatViolation => 2                 FutureFormat => 3
VerifyPhraseMismatch => 4
```

`ms` `From<ms_codec::Error>` (`S/src/error.rs:174-328`) decides 1 vs 2 vs 3:
`Codex32` -> 1; `WrongHrp`, `ThresholdNotZero`, `ShareIndexNotSecret`,
`TagInvalidAlphabet`, `UnknownTag`, `ReservedPrefixViolation`, `TooManyErrors`,
`IsShareNotSingleString`, `SecretShareSuppliedToCombine`,
`InconsistentShareSet`, `PreimageLengthMismatch`, `TagKindMismatch` -> 2;
`ReservedTagNotEmittedInV01` -> 3; `InvalidThreshold`, `InvalidShareCount`,
`RandomnessUnavailable`, and the `other =>` wildcard -> 1.

Sites that return a code without going through the enum:
- `K/src/cmd/repair.rs:160` `return Ok(2)` (any codec error out of the
  correcting decode; "the split is by VERB, not by error kind").
- `K/src/cmd/repair.rs:242` `Ok(if any_correction { 5 } else { 0 })`.
- `K/src/cmd/encode.rs:498` `return Ok(1)` (failed `--out` write).
- `K/src/cmd/encode.rs:599` `return Ok(Some(64))` (stdout is an input file).
- `S/src/cmd/repair.rs:248` `Ok(if any_correction { 4 } else { 0 })`.
- `S/src/main.rs:209,221,225` (pre-clap, above).

### A.1 `mk` inventory

| code | meaning | verbs | decided at | measured |
|---|---|---|---|---|
| 0 | success | all | each `run` returns `Ok(0)` | `mk decode --in v2.txt` -> 0; `mk verify --in v2.txt --xpub <V2 xpub>` -> 0; `mk encode --xpub ... --origin-fingerprint deadbeef --origin-path "m/84'/0'/0'" --policy-id-stub c0ffee00` -> 0; `mk address`, `mk derive --index 0`, `mk inspect`, `mk gui-schema`, `mk vectors`, `mk gen-man --out DIR` -> 0 |
| 0 | `--help`, `--version`, `help` | all | `K/src/main.rs:84-86` | `mk --help`, `mk --version`, `mk help`, `mk decode --help` -> 0 |
| 0 | repair: nothing to correct (BCH valid), **even when the card does not decode** | repair | `K/src/cmd/repair.rs:242` | `mk repair --in v2.txt` -> 0; `mk repair --in N12.txt` (unsupported version) -> 0; also N6, N7, N8, N9, N10, N11, N13, N18 -> 0 |
| 0 | decode/verify succeeded after silent BCH correction (stderr note only) | decode, verify, inspect, address, derive | `K/src/cmd/mod.rs` (correction note) | `mk decode --in v2c1.txt` -> 0; `mk verify --in v2c1.txt` -> 0 |
| 0 | stdout closed (`>&-`): output lost, not detected | all | `K/src/out.rs:37-57` (only a read-only fd is detected) | `mk decode --in v2.txt >&-` -> 0 |
| 1 | invalid artifact (any `mk_codec::Error` except `UnsupportedVersion`) | decode, verify, inspect, address, derive | `K/src/error.rs:138` | `mk decode --in N1.txt` (HRP `bt`) -> 1; N2, N3, N4, N5 (BCH uncorrectable), N6, N7, N8, N9, N10, N11, N13, N18 -> 1 on decode and verify; `mk inspect --in N5.txt` -> 1; `mk address --in N5.txt` -> 1; incomplete chunk set `mk decode <one of two chunks>` -> 1 |
| 1 | encoder refusal from the codec (xpub/origin mismatch, key-origin rules) | encode, verify | `K/src/error.rs:138` | `mk encode --xpub <V2> ... --origin-path "m/44'/0'/0'/5"` -> 1 |
| 1 | invalid md1 given as `--from-md1` | encode, verify | `K/src/error.rs:138` (`MdCodec`) | `mk encode --from-md1 md1garbage ...` -> 1; `mk verify --in v2.txt --from-md1 md1garbage` -> 1 |
| 1 | I/O error: `IoError` (stdin read; `vectors --out`, `gen-man --out`) | all | `K/src/error.rs:142` | `mk gen-man --out /proc/nope/x` -> 1; `mk gen-man --out <existing file>` -> 1; `mk vectors --out <existing file>` -> 1 |
| 1 | `encode --out` write failed | encode | `K/src/cmd/encode.rs:498` | `mk encode --keys keys.txt --policy-id-stub c0ffee00 --out .../nodir/x` -> 1; `--out <a directory>` -> 1 |
| 1 | stdout write failed (not EPIPE) on a run that would have been 0 or 5 | all, `--help`, `--version` | `K/src/out.rs:81` | `mk decode --in v2.txt >/dev/full` -> 1; `mk repair --in v2c1.txt >/dev/full` -> 1 (would be 5); `mk --help >/dev/full` -> 1; `mk --version >/dev/full` -> 1; `mk decode --in v2.txt 1</dev/null` -> 1 |
| 2 | repair: any codec error from the correcting decode (uncorrectable, wrong HRP, bad char, bad length, mixed case) | repair | `K/src/cmd/repair.rs:160` | `mk repair --in N5.txt` -> 2; N1, N2, N3, N4 -> 2; `mk repair --json --in N5.txt` -> 2 (envelope `exit_code:2`); `mk repair --in v2c5.txt` -> 2 |
| 2 | repair: corrected complete set does not reassemble (`SetReassemblyMismatch`) | repair | `K/src/error.rs:143`, raised `K/src/cmd/repair.rs:467` | `mk repair --in N11c1.txt` -> 2; `--json` -> 2 |
| 3 | `UnsupportedVersion` (future format) | decode, verify, inspect, address, derive | `K/src/error.rs:137` | `mk decode --in N12.txt` -> 3; `mk verify --in N12.txt` -> 3; `mk derive --index 0 --in N12.txt` -> 3; `--json` envelope `exit_code:3` |
| 4 | verify content mismatch ("answer is no") | verify | `K/src/error.rs:140`, raised `K/src/cmd/verify.rs:90-298` | `mk verify --in v2.txt --origin-fingerprint 00000000` -> 4; `--policy-id-stub 00000000` -> 4; `--json` -> 4 |
| 5 | repair applied: at least one correction; set reassembles (verified) **or** set incomplete (unverified, stderr advisory only) | repair | `K/src/cmd/repair.rs:242` | `mk repair --in v2c1.txt` (complete set) -> 5; `mk repair --in v2c1only.txt` (one chunk of two, "correction UNVERIFIED") -> 5; `--json` -> 5 |
| 64 | clap usage error | all | `K/src/main.rs:87` | `mk bogus` -> 64; `mk vectors --bogus` -> 64; `mk encode` -> 64; `mk gen-man` -> 64; `mk decode --in a --in b` -> 64; `mk decode $'\xff\xfe'` -> 64; `mk derive --index 99999999999` -> 64 |
| 64 | no subcommand (help goes to **stderr**) | - | `K/src/main.rs:87` | `mk` -> 64 |
| 64 | verb-level `UsageError` (bad value, exclusive flags, missing binding) | encode, address, derive, vectors, ... (79 construction sites) | `K/src/error.rs:141` | `mk encode ... (no fingerprint flag)` -> 64; `--origin-fingerprint zz` -> 64; `--origin-path garbage` -> 64; no `--policy-id-stub` -> 64; `mk address --range 5..2` -> 64; `mk derive --path "0'"` (hardened) -> 64 |
| 64 | **no input / empty input** | decode, verify, inspect, repair, address, derive; encode `--keys` | `K/src/cmd/mod.rs` (`read_mk1_strings`), `K/src/keyfile.rs` | `mk decode` -> 64; `mk decode -` (empty stdin) -> 64; `mk decode --in empty.txt` -> 64; `mk verify`, `mk repair`, `mk repair --in empty.txt`, `mk address`, `mk inspect` -> 64; `mk encode --keys empty.txt --policy-id-stub c0ffee00` -> 64; `mk encode --keys - ...` -> 64 |
| 64 | **input file missing / is a directory** (an I/O error reported as usage) | every `--in`, `--keys`, `--from-md1-set` | `K/src/cmd/mod.rs` | `mk decode --in nonexistent.txt` -> 64; `mk decode --in <dir>` -> 64; `mk encode --keys nokeys.txt ...` -> 64; `--from-md1-set nofile.txt` -> 64 |
| 64 | invalid input *content* on the encode side (not an artifact): bad xpub, bad key record | encode | `K/src/cmd/mod.rs`, `K/src/keyfile.rs` | `mk encode --xpub notanxpub ...` -> 64; `mk encode --keys badkeys.txt ...` -> 64 |
| 64 | policy refusal: private key offered as `--xpub` | encode | `K/src/cmd/mod.rs` | `mk encode --xpub xprv9s21Zr...(BIP-32 test vector 1) ...` -> 64 |
| 64 | policy refusal: multisig-cosigner card asked for single-key addresses | address | `K/src/cmd/address.rs:160` | `mk address --in v1.txt` -> 64 |
| 64 | safety refusal: output is one of the inputs | encode | `K/src/cmd/encode.rs:599` and `UsageError` just below | `mk encode --keys keys2.txt --policy-id-stub c0ffee00 >> keys2.txt` -> 64; `--out keys.txt` (same file as `--keys`) -> 64 |
| 101 | panic (inferred; no `catch_unwind`, default unwind) | - | - | source only |
| 143 / signal | killed by signal; no handlers installed | all | - | `mk decode - < <(sleep 5)` + `kill -TERM` -> shell status 143 |
| EPIPE | reader closed the pipe: the verb's own code | all | `K/src/out.rs:62` | `mk vectors | head -1` -> `PIPESTATUS[0]` = 0 |

### A.2 `ms` inventory

| code | meaning | verbs | decided at | measured |
|---|---|---|---|---|
| 0 | success | all | each `run` returns `Ok(0)` | `ms decode --in ms0.txt` -> 0; `ms decode - <ms0.txt` -> 0; `ms encode --phrase - <phrase.txt` -> 0; `ms encode --hex - <hex.txt` -> 0; `ms verify --in ms0.txt` -> 0; `ms verify --in ms0.txt --phrase - <phrase.txt` -> 0; `ms split -k 2 -n 3 --phrase - <phrase.txt` -> 0; `ms combine --in shares2.txt` -> 0; `ms derive --in ms0.txt [--template bip84]` -> 0; `ms hashlock --hashlock-phrase-stdin <phrase.txt` -> 0; `ms hashlock --random --out FILE` -> 0; `ms inspect`, `ms vectors`, `ms gui-schema`, `ms gen-man --out DIR` -> 0 |
| 0 | `--help`, `--version`, `help` | all | `S/src/main.rs:249-251` | `ms --help`, `ms --version`, `ms help` -> 0 |
| 0 | repair: input already valid | repair | `S/src/cmd/repair.rs:248` | `ms repair --in ms0.txt` -> 0 |
| 0 | **inspect says "FAIL: would NOT decode v0.1"** (verdict on stdout, exit 0) | inspect | `S/src/cmd/inspect.rs:74,84` | `ms inspect --in ms_future.txt` -> 0; `ms_unknowntag.txt` -> 0; `ms_badprefix.txt` -> 0; `ms_wronghrp2.txt` -> 0; a K-of-N share -> 0 ("OK: K-of-N share") |
| 0 | empty passphrase from stdin (warning only) | derive | `S/src/passphrase_input.rs` | `ms derive --in ms0.txt --passphrase-stdin` -> 0 |
| 0 | stdout closed (`>&-`): output lost, not detected | all | `S/src/sink.rs` | `ms encode --phrase - >&- <phrase.txt` -> 0 |
| 1 | **secret on argv refused, before clap** | every verb that takes secret material | `S/src/main.rs:219-222` | `ms decode ms10entrs...` -> 1; `ms encode --hex 00..00` -> 1; `ms encode --phrase 'abandon ...'` -> 1; `ms encode --nosuchflag --hex 00..00` -> 1 (refusal wins over the usage error); `ms derive --in ms0.txt --passphrase hunter2` -> 1; `ms hashlock --hashlock-phrase '...'` -> 1 |
| 1 | invalid artifact: checksum invalid, bad length (`Codex32`, `UnexpectedStringLength`, `PayloadLengthMismatch`) | decode, verify, inspect, derive | `S/src/error.rs:62-66` | `ms decode --in ms_c1.txt` (1 substitution) -> 1; verify/inspect/derive -> 1; `ms decode --in ms_short.txt` -> 1 |
| 1 | invalid input content: bad BIP-39 phrase, bad hex, bad k/n, bad account, unset `@env:` var, wrong payload kind | encode, split, verify, derive, hashlock, combine | `S/src/error.rs:62-66` (`BadInput`, `Bip39`) | `ms encode --phrase - <badphrase.txt` -> 1; `ms encode --hex - <phrase.txt` -> 1; `echo 0011 | ms encode --hex -` -> 1; `ms split -k 1 -n 3 ...` -> 1; `-k 3 -n 2` -> 1; `ms verify --in ms0.txt --phrase - <badphrase.txt` -> 1; `ms derive ... --account 4294967295` -> 1; `--passphrase @env:NOPE_UNSET` -> 1; `ms hashlock --hex - <<<00` -> 1; `ms hashlock --in ms0.txt` (a seed, not a preimage) -> 1; `ms combine --in shares1.txt` (too few) -> 1; duplicate share index -> 1 |
| 1 | **no input / empty input** | encode, decode, verify, inspect, derive, combine, hashlock | same arm | `ms decode --in empty.txt` -> 1; `ms decode --json` (empty stdin) -> 1; `ms derive` -> 1; `ms encode --phrase - </dev/null` -> 1; `ms encode --in empty.txt` -> 1; `ms verify --in ms0.txt --phrase - </dev/null` -> 1; `ms combine --in empty.txt` -> 1; `ms combine -` -> 1; `ms hashlock --hashlock-phrase-stdin </dev/null` -> 1 |
| 1 | I/O error reading input | all | `S/src/parse.rs:105,154` (`BadInput`) | `ms decode --in nofile.txt` -> 1; `ms decode --in <dir>` -> 1; `ms encode --in nofile.txt` -> 1; `ms combine --in nofile.txt` -> 1 |
| 1 | `--out` write failed (`OutputWrite`, stderr in both modes) | encode, repair, split, hashlock | `S/src/error.rs:68` | `ms encode --phrase - --out .../nodir/o <phrase.txt` -> 1; `--json` same -> 1; `ms repair --in ms_c1.txt --out .../nodir/o` -> 1; `ms split ... --out .../nodir/s` -> 1 |
| 1 | I/O error writing `gen-man` output; internal serialize failures (`BadInput`) | gen-man, vectors, repair, gui-schema | `S/src/cmd/gen_man.rs:31,35`, `S/src/cmd/vectors.rs:25,27` | `ms gen-man --out <existing file>` -> 1; rest source only |
| 1 | policy/safety refusal at verb level: phrase looks like a digest; two things on one stdin | hashlock, derive | `BadInput` | `ms hashlock --hashlock-phrase-stdin <<<(64 hex chars)` -> 1; `ms derive - --passphrase-stdin <ms0.txt` -> 1 |
| 1 | stdout write failed (not EPIPE) on a run that returned `Ok` | all, `--help` | `S/src/sink.rs:97-98` | `ms encode --phrase - >/dev/full <phrase.txt` -> 1; `ms repair --in ms_c1.txt >/dev/full` -> 1 (would be 4); `ms --help >/dev/full` -> 1; `ms encode --phrase - 1</dev/null` -> 1 |
| 2 | format violation: well-formed codex32 that is not a v0.1 ms1 (wrong HRP, unknown tag, bad prefix byte, a share where a single string is needed, the `s` share given to combine, inconsistent share set, preimage kind/length) | decode, verify, derive, repair, combine, hashlock | `S/src/error.rs:69` | `ms decode --in ms_wronghrp2.txt` -> 2; verify, derive, repair -> 2; `ms decode --in ms_unknowntag.txt` -> 2; `ms decode --in ms_badprefix.txt` -> 2; `ms decode --in shares1.txt` -> 2; `ms verify`, `ms repair --in shares1.txt` -> 2; `ms combine --in ms0.txt` -> 2 |
| 2 | repair: uncorrectable (`TooManyErrors`) | repair | `S/src/error.rs:69` via `:235` | `ms repair --in ms_c6.txt` (6 substitutions) -> 2; `--json` -> 2; `ms repair --in ms_short.txt` -> 2 |
| 2 | repair: **empty input** (reported as "wrong HRP: got \"\"") | repair | same | `ms repair --in empty.txt` -> 2; `ms repair --ms1 -` (empty stdin) -> 2 |
| 3 | valid future format (reserved tag) | decode, verify, derive, repair | `S/src/error.rs:70`; verify prints `OK: valid future format` on **stdout** and still exits 3 (`S/src/cmd/verify.rs:200`, `S/src/main.rs:318`) | `ms verify --in ms_future.txt` -> 3 (stdout "OK: valid future format (v0.2+, tag seed)"); `ms decode --in ms_future.txt` -> 3; derive -> 3; `ms repair --in ms_future.txt` -> 3; `--json` envelope `exit_code:3` |
| 4 | verify phrase mismatch ("answer is no") | verify | `S/src/error.rs:71`, raised `S/src/cmd/verify.rs:156` | `ms verify --in ms0.txt --phrase - <wrongphrase.txt` -> 4; `--json` -> 4 |
| 4 | repair applied, **unverified candidate** (VERIFY-ME) | repair | `S/src/cmd/repair.rs:248` | `ms repair --in ms_c1.txt` -> 4; `--json` -> 4 (`"verdict":"candidate"`); `ms repair --in ms_c1.txt --out FILE` -> 4 **and FILE is written** |
| 5 | never returned | - | `S/src/cmd/repair.rs` module doc ("exit 5 is ... unreachable") | no run produced 5 |
| 64 | clap usage error | all | `S/src/main.rs:252` | `ms bogus` -> 64; `ms decode --bogus` -> 64; `ms encode` -> 64; `ms repair` -> 64; `ms combine` -> 64; `ms gen-man` -> 64; `ms split -k 2 --phrase -` -> 64; `ms decode --language klingon ...` -> 64; `ms decode --in ms0.txt -` -> 64; `ms hashlock --kind bogus` -> 64; `ms derive --hex - --phrase -` -> 64 |
| 64 | no subcommand (help goes to **stderr**) | - | `S/src/main.rs:252` | `ms` -> 64 |
| 64 | non-UTF-8 argv, argv-guard usage | - | `S/src/main.rs:209,225` | `ms decode $'\xff\xfe'` -> 64 |
| 64 | verb-level `Usage` (source arithmetic, flag gates) | encode, hashlock, split | `S/src/error.rs:67` | `ms encode --hex - --language japanese <hex.txt` -> 64; `ms hashlock` (no source) -> 64; `ms hashlock --random` (no `--out`) -> 64; `ms hashlock --hex - --emit-record` -> 64; `--emit-record --no-engraving-card` -> 64 |
| 64 | safety refusal: `--out` is stdout (`StdoutIsOut`, stderr in both modes) | encode, repair, hashlock | `S/src/error.rs:67`, `S/src/out.rs:98` | `ms encode --phrase - --json --out ms2.out > ms2.out <phrase.txt` -> 64 |
| 64 | safety refusal: `--out` exists and must not be overwritten | split, hashlock `--random` | `Usage` | `ms split ... --out <existing>` -> 64; `ms hashlock --random --out <existing>` -> 64 |
| 101 | panic (inferred) | - | - | source only; three tests assert `!= Some(101)` |
| 143 / signal | killed by signal; handlers only while a passphrase prompt has echo off, and they re-raise (`S/src/passphrase_input.rs:161-191`) | all | - | `ms decode - < <(sleep 5)` + `kill -TERM` -> 143 |
| EPIPE | the verb's own code | all | `S/src/sink.rs:78` | `ms vectors | head -1` -> `PIPESTATUS[0]` = 0 |

---

## B. Class matrix

| # | class | `mk` | `ms` | agree? |
|---|---|---|---|---|
| 1 | success | 0 | 0 | yes |
| 2 | "answer is no" | verify: **4** (`ContentMismatch`) | verify `--phrase`: **4** (`VerifyPhraseMismatch`); inspect "would NOT decode": **0** | verify agrees; `ms inspect` reports a no as 0 |
| 3 | invalid input artifact | decode/verify/inspect/address/derive: **1**; unsupported version: **3**; repair: **2** (or 0 if BCH-clean) | checksum/length: **1**; well-formed but wrong HRP/tag/prefix/share: **2**; reserved tag: **3**; inspect: 1 for checksum/length, **0** for the format-violation class | no: `ms` splits one class into 1 and 2; `mk` uses 1 everywhere except repair |
| 4 | usage error | **64** (clap and verb-level) | **64** (clap and verb-level) | yes |
| 5 | no input / empty input | **64** on every verb | **1** on decode/verify/inspect/derive/encode/combine/hashlock; **2** on repair | no |
| 6 | policy / safety refusal | xprv as `--xpub`: 64; multisig card to `address`: 64; output is an input: 64 | secret on argv: **1** (pre-clap); `--out` is stdout: 64; `--out` exists (split, `--random`): 64; digest-shaped phrase: 1; two readers on stdin: 1 | no: `mk` always 64; `ms` 1 or 64 depending on the refusal |
| 7 | I/O error | input file missing/dir: **64**; `encode --out` unwritable: **1**; `vectors`/`gen-man --out` failure: 1; stdin read error: 1 (source only); stdout write error: 1 | input file missing/dir: **1**; `--out` unwritable: 1; `gen-man --out` failure: 1; stdout write error: 1 | no on input files (64 vs 1); yes on output |
| 8 | repair | verified (complete set reassembles): **5**; unverified (incomplete set): **5** + stderr advisory; uncorrectable or any codec error: **2**; corrected set fails reassembly: **2**; nothing to correct: 0 | applied: always **4** (unverified candidate); uncorrectable: **2**; nothing to correct: 0; reserved tag: 3 | 2 and 0 agree; applied is 5 vs 4 by design |
| 9 | external dependency failure | none (no subprocess, no network) | none (OS random source unavailable for `hashlock --random` -> 1, source only, `S/src/error.rs:320`) | n/a |
| 10 | partial success / warnings only | 0: BCH correction during decode/verify (stderr note); stale chunk-set id warning; `address --count 0` prints nothing | 0: language defaulted; empty passphrase from stdin; mlock soft-fail summary (`S/src/main.rs:286`, stderr only) | yes (both 0) |
| 11 | internal error / panic | 101 (inferred); `main` does not catch | 101 (inferred); `main` does not catch; internal serialize failures map to 1 | yes |
| 12 | other | help/version 0; bare `mk` 64 with help on stderr; EPIPE keeps the verb's code; stdout full -> 1 (0 and 5 only); closed stdout -> 0; SIGTERM -> 143 | same, except the stdout-full rule rewrites any `Ok` code (0 and 4) | yes |

---

## C. Collisions and surprises

Same code, two meanings, one binary:
1. **`ms` 4** = verify phrase mismatch **and** repair-applied candidate. Both
   are "do not trust this yet", but one is a failure and one carries the
   result on stdout.
2. **`mk` 4** = verify content mismatch, while the toolkit's `mnemonic repair
   --mk1` uses 4 for an incomplete-set repair that `mk repair` reports as 5
   (the manual says so, `44-mk-cli.md:268-284`).
3. **`ms` 1** = invalid artifact, bad user input, empty input, missing input
   file, unwritable output, stdout failure **and** the argv-secret refusal.
   A script cannot tell "your card is damaged" from "you put a secret on the
   command line" from "disk full".
4. **`mk` 1** = invalid artifact, encoder refusal, unwritable `--out`, I/O
   error, stdout failure.
5. **`mk` 2** = uncorrectable **and** "corrected, but the set does not
   reassemble - not trustworthy" (`SetReassemblyMismatch`). **`ms` 2** =
   uncorrectable **and** format violation on every reading verb **and** empty
   input to repair.
6. **`mk` 64** = usage error, empty input, missing input file, invalid xpub /
   key record content, private-key refusal, multisig refusal, output-is-input
   refusal.

Same class, different codes across verbs of one binary:
7. `mk`: an invalid card is **1** on decode/verify/inspect/address/derive and
   **2** on repair (deliberate, `K/src/cmd/repair.rs:121-160`).
8. `mk repair` exits **0** on cards that `mk decode` rejects with 1 or 3
   (N6-N13, N18): repair only checks BCH, so "already valid" means "checksum
   valid", not "decodes". Measured: `mk repair --in N12.txt` -> 0, `mk decode
   --in N12.txt` -> 3.
9. `mk`: unwritable output is **1** on `encode --out`; unreadable input is
   **64**.
10. `ms`: empty input is **1** on six verbs and **2** on repair (`ms repair
    --in empty.txt` -> "wrong HRP: got \"\"").
11. `ms inspect` exits **0** for inputs every other verb rejects with 2 or 3
    (wrong HRP, unknown tag, bad prefix, reserved tag); the verdict is the
    stdout line `FAIL: would NOT decode v0.1`. For checksum/length failures it
    exits 1 like the others.
12. `ms verify` on a reserved tag prints `OK: valid future format` on stdout
    and exits **3** (a non-zero "OK").
13. `ms`: one HRP defect gives two codes depending on the checksum: `mq1...`
    with the original ms checksum -> 1 on decode but 2 on repair; `mq1...`
    with a valid `mq` checksum -> 2 on both.
14. `ms repair --out FILE` writes FILE and exits 4 (measured); `mk` has no
    `repair --out`.

Differences between the two binaries (summary of B): empty input 64 vs 1/2;
missing input file 64 vs 1; invalid artifact 1 vs 1/2; repair applied 5 vs 4;
safety refusals 64 vs 1/64; stdout-failure filter rewrites {0,5} vs any `Ok`.

Docs that disagree with behaviour:
15. `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md:622` says
    "an unreadable `--in FILE` and an unwritable `mk encode --out FILE` are
    `64`". Measured: unwritable `mk encode --out` is **1** (`mk encode ...
    --out .../nodir/x` -> 1, and `K/src/cmd/encode.rs:498`, comment "R3: one
    line on stderr, exit 1").
16. Same table, `:623` "2 | `mk repair` only" and `:627` "64 | CLI usage error
    per clap convention": 64 also covers empty input, missing files and the
    refusals in item 6; clap's own convention is 2, not 64.
17. `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md` §6f table
    (line ~1050) still shows `mk` invalid artifact = **2**; it is now 1
    (`/home/user/wt/cc/shared.md` CLI-14 already says so).
18. `/home/user/wt/cc/shared.md` CLI-14: "`mk decode -` 64" confirmed. It does
    not list `ms`: `ms decode -` on empty stdin is **1**.
19. `mnemonic-secret/design/SPEC_ms_cli_v0_1.md:444-449` (the "locked" table)
    has no row for the argv refusal (1), `OutputWrite` (1), `Usage` /
    `StdoutIsOut` (64 from a verb), the K-of-N format violations, or repair's
    4. `:209` says an invalid `verify` prints `FAIL: <reason>` on stdout;
    measured `ms verify` on an invalid card prints `error: ...` on stderr and
    nothing on stdout. `:214` says the mismatch prints `FAIL: phrase mismatch`
    on stdout; measured it is `error: phrase mismatch ...` on stderr.
20. `ms repair --help` and `43-ms.md:790` say 2 = "BCH-uncorrectable (> 4
    substitution errors ...)"; the message the binary prints is "more than 8
    errors; uncorrectable". Not an exit-code defect, but the same sentence the
    code table lives in.
21. `ms repair` help/manual list codes 0, 4, 2, 1 only; a reserved-tag input
    gives **3** (measured).
22. `K/src/error.rs:22` doc comment on `Codec` still says "Exit 2 (or 3 for
    `UnsupportedVersion`)"; the mapping below it returns 1.
23. `ms combine <grouped shares pasted on argv>` was not refused by the argv
    guard and exited 1 with per-share notes (20 arguments, each a 5-character
    group). Not an exit-code finding; noted because the guard is what owns
    `ms`'s refusal code.

---

## D. Documented codes

`--help` text:
- `mk --help`: "repair ... (exit 5 = REPAIR_APPLIED)" (`K/src/main.rs:53`).
- `mk encode --help`: "(exit 64 otherwise)" on `--origin-fingerprint` and
  `--privacy-preserving` (`K/src/cmd/encode.rs`).
- `ms --help`: "repair ... (exit 4 = VERIFY-ME candidate)".
- `ms repair --help`: "Exit 4 on correction-applied ...; exit 0 if input was
  already valid; exit 2 if BCH-uncorrectable (`TooManyErrors`)"
  (`S/src/cmd/repair.rs`, module doc `:11-37`).
- `ms verify --help`: "exit 4 on mismatch"; examples "exit 0 = valid v0.1"
  (`S/src/cmd/verify.rs:44`).
- `ms encode|repair --help` `--out`: "refused (exit 64)"; `ms split --help`
  `--out`: "refused with exit 64"; `ms hashlock --help` `--out` and
  `--emit-record`: "(refused, exit 64)".

Toolkit manual (`mnemonic-toolkit/docs/manual/src/40-cli-reference/`):
- `44-mk-cli.md:35,43,56,121` (exit 64 notes); `:259-266` `mk repair` table
  (0, 5, 2, 1); `:268-284` 5-vs-4 asymmetry with the toolkit; `:311-322`
  set-level re-verify codes; `:451-453` `mk verify` (0, 1, 4); `:617-627`
  the `mk` exit-code table (0, 1, 2, 3, 4, 5, 64); `:629` stdout-failure rule.
- `43-ms.md:47` (existing `--out` refused, 64); `:87,90` (64); `:249,255`;
  `:368` (verify 4); `:402,410` (hashlock 64); `:784-791` `ms repair` table
  (0, 4, 2, 1); `:793-814` why never 5; `:837`; `:889-894` gen-man (0, 1);
  `:900` stdout-failure rule. **There is no whole-binary exit-code table for
  `ms` in the manual** (mk has one at `44-mk-cli.md:617`).

Repo docs:
- `mnemonic-secret/design/SPEC_ms_cli_v0_1.md:440-468` (§6 table and
  rationale for 1 vs 2, 3, 4, 64); `:82-85,190-224,239` per-verb codes;
  `:475-494` enum doc; `:722-744` test list with codes.
- `mnemonic-secret/crates/ms-cli/README.md:54,64-71` (`ms repair` table).
- `mnemonic-secret/design/SPEC_ms_derive.md:76-78`;
  `mnemonic-secret/design/SPEC_ms_hashlock.md:320,335`.
- `mnemonic-key/docs/MK_CODEC_RUST_API.md:93` ("`mk verify` ... exits 1").
- `mnemonic-key/design/SPEC_mk_derive_address.md:111-123` (exit 64 cases);
  `SPEC_mk_slip0132_acceptance.md:39-58,87`;
  `SPEC_chunk_set_id_verification.md:145-146,272`.
- `mnemonic-key/crates/mk-cli/CHANGELOG.md:18-62` (per-fix codes).
- `K/src/error.rs:1-11`: "the exit-code table [is a] CLI convention with NO
  mk SPEC section governing [it] -- pinned by `tests/` and the CHANGELOG".
  mk has no spec-level exit-code table; ms has one (SPEC §6).
- Cross-repo: `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md`
  §6f (line 1017); `/home/user/wt/cc/shared.md` CLI-14 (line ~479).

---

## E. Tests that pin exit codes

Counts are from a regex over `.code(N)`, `Some(N)` and `exit_code ... N`, so
they are approximate (the 0 row over-counts).

| | `mk-cli` | `ms-cli` |
|---|---|---|
| test files | 40, of which 37 mention a code | 96, of which 64 mention a code |
| code 0 | ~89 (+53 `.success()`) | ~134 (+187 `.success()`) |
| code 1 | ~17 | ~68 |
| code 2 | ~7 | ~20 |
| code 3 | 0 in `tests/` (one row in the unit table) | ~7 |
| code 4 | ~12 | ~11 |
| code 5 | ~9 | 0 |
| code 64 | ~48 | ~40 |
| bare `.failure()` | 1 | 26 |
| `!= Some(101)` | 0 | 3 |

Unit tables that pin the whole mapping: `K/src/error.rs:254-319`
(`mod exit_code_table`) and `S/src/error.rs:335-515`
(`exit_code_table_per_variant` and friends); `S/src/cmd/split.rs:290`.

Heaviest files. mk: `tests/bh1119_mk_out_writes.rs` (23),
`cli_repair.rs` (12), `bh1125_verify_membership.rs` (11),
`csid_verification.rs`, `bh1125_private_key_echo.rs`, `baseline_gapb.rs` (10
each), `exit_code_invalid_artifact.rs` (dedicated to the 2 -> 1 move),
`version_help_exit_codes.rs`, `output_errors.rs`,
`cli_mk1_repair_reverify.rs`. ms: `hashlock_sources.rs` (20),
`bh1119_printed_commands.rs` (17), `single_print.rs` (16), `bughunt.rs` (16),
`bh1119_ms_out_writes.rs` (15), `output_errors.rs` (13),
`allow_argv_secret.rs` (13), `json_error_envelope_per_kind.rs`,
`verify_future_format.rs`.

Also pinned outside the test crates: the JSON error envelope's `exit_code`
field (both binaries), and shell row scripts in
`mnemonic-key/scripts/` (`mk-out-writes-rows.sh` ~30 `rc=$?` rows,
`mk-fa-line-readers-rows.sh:105`, `mk-fe-keyed-policy-rows.sh:110`,
`mk-fd-derive-guards-rows.sh:63`, `fix-bh1119-mk-out-writes-gate.sh:33`).

Renumber cost, roughly: changing `mk`/`ms` usage from 64 touches ~48 + ~40
assertions; changing `ms`'s 1 touches ~68; unifying repair-applied touches ~9
(mk 5) or ~11 (ms 4, shared with verify mismatch).

---

## F. Consumers

Production code that reads these binaries' exit codes:
- **mnemonic-gui** spawns `mk` and `ms` (and `md`, `mnemonic`) and keeps the
  code: `mnemonic-gui/src/runner.rs:222-235, 345-354`
  (`output.status.code()`). `mnemonic-gui/src/app_window.rs:1316-1345`
  `render_exit_badge` gives meaning to two codes for **every** CLI tab:
  `Some(4)` -> amber "VERIFY-ME" badge, `Some(5)` -> green repair-applied
  badge, everything else a plain `exit: N`. So `mk verify` / `ms verify`
  mismatches (4) show the VERIFY-ME badge, the same as an `ms repair`
  candidate. `mnemonic-gui/src/cli_version.rs:122` requires `Some(0)` from
  `--version`. `mnemonic-gui/src/form/tree_form.rs:197,327` prints the code
  in an error string.
- No other Rust `src/` in the constellation spawns `mk` or `ms`
  (`mnemonic-engrave/crates/me-cli/src/sysw/advice.rs:31` records that `me`
  names `ms` only in advice text; grep of `Command::new` in toolkit, engrave,
  refugium, transaction and refugium-wallet `src/` found none).

Tests in other repos that run these binaries and assert a code:
- `mnemonic-engrave/crates/me-cli/tests/ms_remedy_runs.rs:109-111`
  (`Some(0)` from the `ms` line `me` advises).
- `mnemonic-refugium/crates/mr-cli/tests/jf8_pr64.rs:1297` (returns the code
  of an `ms`/`mk` shell line); `jf8_b_advice.rs:477,687` (`Some(0)`);
  `fj1_kits.rs:957`, `m13a_create_seeds.rs:646,679`, `fj3_pages.rs:734`
  (run `mk`/`ms` through `tool_bin`).
- `mnemonic-gui/tests/ui_harness_i4_realcli.rs:242,268` (drives `ms decode`
  and `mk decode` through the runner).

CI: no workflow in `mnemonic-key`, `mnemonic-secret` or `mnemonic-toolkit`
branches on a specific `mk`/`ms` exit code (grep of `.github/workflows`); the
manual lint (`mnemonic-toolkit/docs/manual/tests/lint.sh`) checks flags, not
codes (not opened beyond that grep - inferred).
