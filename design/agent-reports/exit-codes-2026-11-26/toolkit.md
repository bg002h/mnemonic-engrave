# Exit-code survey: `mnemonic` (mnemonic-toolkit) + toolkit manual / GUI manual inventory

Surveyor: toolkit. Binary: `/home/user/wt/target/debug/mnemonic` (`mnemonic 0.107.1`),
source `/home/user/wt/mnemonic-toolkit` at `40217d8b`. Read-only; no code changed.
All runs: stdin `</dev/null` unless a file is shown; inputs are the BIP-39 all-`abandon`
test phrase, cards minted from it, mk-codec vector N12, and throwaway values.
Full run logs (exact command -> code -> first stderr bytes):
`/home/user/wt/ec/tmp-toolkit/log.txt` (mnemonic, ~230 runs) and
`/home/user/wt/ec/tmp-toolkit/log-others.txt` (md/mk/ms/mt/me/mr/mr-sitting spot-checks).
Card variables used below are in `/home/user/wt/ec/tmp-toolkit/vars.sh`
(`$MD1 $MD2 $MD3` = 3-chunk md1, `$MK1 $MK2` = 2-chunk mk1, `*BAD` = 1 substituted
character, `$MDGARB` = >4 errors, `ms.txt`/`msbad.txt` = ms1 on stdin).

Caveats on my own evidence:
- The sandbox runs as uid 0, so "unreadable / unwritable by permission" could not be
  tested. I/O failures were forced with ENOENT, EISDIR, ENOTDIR and `/dev/full`.
- Two of my early I/O probes wrote to `/nonexistent-dir/` because another surveyor had
  created that directory seconds before; I deleted the two files I made there
  (`x.txt`, `x.json`, test-vector content only). The other files in it are not mine.
- "source only" = read, not run. "inferred" is marked.

---

## A. Exit-code inventory

### A.1 How `main` maps outcomes to codes

`src/main.rs:4` returns `mnemonic_toolkit::run()` (`src/cli.rs:131`), an `ExitCode`. No
`std::process::exit` anywhere in `src/`. Order of decisions in `run()`:

| Step | Where | Code |
|---|---|---|
| non-UTF-8 argv | `cli.rs:144-158` | 64 |
| argv-secret guard, runs BEFORE clap | `cli.rs:159-167`, `argv_guard.rs:91` `EXIT_REFUSED: u8 = 2` | 2 |
| clap error: `Cli::try_parse()`, custom handler (not `Error::exit()`): `base = if e.exit_code()==0 {0} else {64}` | `cli.rs:169-187` | 64 (usage), 0 (`--help`/`--version`) |
| verb returns `Ok(u8)` | `cli.rs:194-254` | the verb's own code (0, 1, 2, 4, 5) |
| verb returns `Err(RepairShortCircuit{exit_code})` (auto-repair) | `cli.rs:261`, `repair.rs:2141` | 5 |
| verb returns `Err(e)`: prints `error: ...`, `e.exit_code()` | `cli.rs:262-266`, `error.rs:641-702` | 1, 2, 3, 4 |
| stdout write failure override (not EPIPE) | `cli.rs:275-279`, `out_sink.rs:89-98` | 0 and 5 become 1; 4 becomes 1 only for `repair` and `inspect`; other codes kept |

`ToolkitError::exit_code()` (`error.rs:641-702`), by code:

- **1**: `BadInput`, `Bip39`, `Bitcoin`, `CosignerSpec`, `CosignersFile`, `DecodeAddress`,
  `EnvVarMissing`, `ImportWalletAmbiguousFormat`, `ImportWalletFormatMismatch`,
  `ImportWalletNetworkClassMismatch`, `Io`, `MultisigConfig`, `NostrKeyParse`,
  `SilentPayment`, `VerifyMessage`; codec fallthrough `_ => 1`; `ms_codec`
  `Codex32`/`UnexpectedStringLength`/`PayloadLengthMismatch` (`error.rs:449-479`);
  `mk_codec` `InvalidStringLength`/`InvalidChar`/`BchUncorrectable` (`:482-509`);
  `md_codec` `Codex32DecodeError`/`Codex32EncodeError` (`:522-524`).
- **2**: `Bip388Distinctness`, `BsmsMacMismatch`, `BsmsRound1Malformed`,
  `BsmsSignatureMismatch`, `BsmsTaprootImportRefused`, `BsmsTaprootRefused`,
  `BuildDescriptorSpec`, `ConvertRefusal`, `DeriveChildLength*`, `DeriveChildUnsupportedApp`,
  `DescriptorParse`, `ExportWallet*` (5 variants), `HrpMismatch`, `ImportWalletParse`,
  `ImportWalletWatchOnlyViolation`, `ImportWalletXprvForbidden`, `Md1CardNotADescriptor`,
  `ModeViolation`, `NetworkMismatch`, `Render`, `Repair`, `SlotInputViolation`,
  `TemplateFormUnsupportedShape`, `UnknownHrp`, `WordCard`, `CompareCost(Parse)`
  (`cost/mod.rs:91-100`); all structural codec rejects (`error.rs:454-475`, `:487-505`,
  `:525-634`).
- **3**: `FutureFormat` (from `ms_codec::ReservedTagNotEmittedInV01` `error.rs:1074`,
  `mk_codec::UnsupportedVersion` `:1089`, `md_codec::WireVersionMismatch` `:1104`, BSMS
  version `wallet_import/bsms.rs:92`); `CompareCost` non-parse refusals (`cost/mod.rs:95-99`).
- **4**: `Bip388VerifyDistinctness`, `BundleMismatch`, `DescriptorReparseFailed`,
  `ImportWalletSeedMismatch`, `RestoreMismatch`, `XpubSearchNoMatch`,
  `XpubSearchPassphraseCandidatesExhausted`.
- **carried**: `RepairShortCircuit { exit_code }` (only ever 5).

`kind()` (`error.rs:707-776`) is the variant name; it is `#[allow(dead_code)]`, "reserved
for the JSON-error envelope".

Verb-level `Ok(n)` sites (not via the enum): `verify_bundle.rs:706, 988, 1231, 2112`
(4 on any failed check or `partial`), `import_wallet.rs:447, 1408` (4 when a BSMS Round-1
signature is unverified), `inspect.rs:177` (4 on md1 partial decode), `build_descriptor.rs:189,
210, 239` (2 on gate diagnostics), `verify_message.rs:137` (1 when the signature is invalid),
`cmd/repair.rs:290` -> `repair.rs:1749` `indel_exit_code` (4 > 5 > 0).

### A.2 Inventory table

| Code | Meaning | Verbs | Decided at | Measured |
|---|---|---|---|---|
| 0 | success | all | `cli.rs:256` | `mnemonic inspect --md1 $MD1 $MD2 $MD3` -> 0; `bundle --network mainnet --template bip84 --slot @0.phrase=- <phrase.txt` -> 0 |
| 0 | `--help`, `--version`, `help` verb | all | `cli.rs:173` | `mnemonic --help` -> 0; `--version` -> 0; `help` -> 0; `seedqr decode --help` -> 0 |
| 0 | success with warnings / unverified banner | restore, final-word, import-wallet, bundle | per verb | `restore --from phrase=- --template bip84 <phrase.txt` -> 0 with `UNVERIFIED:` on stderr; `restore ... --expect-fingerprint deadbeef --allow-mismatch` -> 0 |
| 0 | "no" answer that is not an error | gui-schema `--classify-descriptor` (`non-canonical`) | `cmd/gui_schema.rs` | `gui-schema --classify-descriptor "wpkh(@0/<0;1>/*)"` -> 0 (`canonical`); non-canonical branch source only |
| 1 | user-input / generic error (`BadInput`, `Bip39`, `Bitcoin`, ...) | all | `error.rs:643-697` | `convert --from phrase=- --to xpub --template bip84 <badphrase.txt` -> 1; `addresses --from xpub=garbage --address-type p2wpkh` -> 1; `decode-address notanaddress` -> 1 |
| 1 | codex32/BCH checksum or alphabet/length failure of a card | inspect, convert, restore, xpub-search | `error.rs:449-453, 484-486, 524` | `inspect md1nonsense` -> 1; `inspect --md1 $MD1BAD $MD2 $MD3` (piped) -> 1; `restore --md1 $MD1BAD --md1 $MD2 --md1 $MD3` -> 1 |
| 1 | empty stdin / empty secret | most | `BadInput` / `Bip39` | `inspect --md1 -` -> 1; `repair --ms1 -` -> 1; `convert --from phrase=- ...` -> 1; `compare-cost` -> 1; `word-card` -> 1 |
| 1 | I/O error (`Io`, or `BadInput` naming the flag) | most | `error.rs:680` | `import-wallet --blob /nonexistent` -> 1; `verify-bundle ... --bundle-json /nonexistent.json` -> 1; `gen-man --out <file>/x` -> 1; `export-wallet ... --output <file>/x` -> 1 |
| 1 | env sentinel unset | all secret flags | `error.rs:665` | `convert --from "phrase=@env:NOPE_UNSET" --to xpub --template bip84` -> 1 |
| 1 | signature does not verify (answer "no") | verify-message | `verify_message.rs:137` | `verify-message --address 1F3s... --message tampered --signature H9L5...` -> 1, stdout `INVALID  (format: legacy)`, stderr empty |
| 1 | stdout write failure on a run that would have exited 0 or 5 (or 4 in repair/inspect) | all | `out_sink.rs:89` | `inspect --md1 ... >/dev/full` -> 1; `repair --md1 $MD1BAD $MD2 $MD3 >/dev/full` -> 1; `repair --mk1 $MK1BAD >/dev/full` -> 1; `--help >/dev/full` -> 1 |
| 2 | secret on argv, refused before clap | all | `cli.rs:166`, `argv_guard.rs:91` | `bundle --phrase "<12 words>" --template bip84` -> 2; `inspect --ms1 ms10entrs...` -> 2; `bundle ... --passphrase hunter2` -> 2 |
| 2 | format / structural violation of a card (wrong HRP, incomplete chunk set, unknown HRP) | inspect, repair, convert, word-card, ms-shares | `error.rs:672, 696`, codec maps | `inspect notanartifact` -> 2; `inspect --md1 $MD1 $MD2` -> 2; `inspect --md1 $MK1` -> 2; `convert --from mk1=$MD1 --to xpub` -> 2; `ms-shares combine --share - <ms.txt` -> 2 |
| 2 | mode violation / refusal / unsupported combination | bundle, convert, export-wallet, derive-child, restore | `error.rs:656-695` | `convert --from xpub=$XPUB --to xprv` -> 2; `export-wallet ... --slot @0.phrase=- <phrase.txt` -> 2; `derive-child ... --application rsa` -> 2; `restore --from phrase=- --format sparrow` -> 2; `bundle --network mainnet --template bip84` (no slots) -> 2 |
| 2 | descriptor / spec / foreign-blob parse error | bundle, verify-bundle, import-wallet, build-descriptor, compare-cost, gui-schema | `error.rs:653, 663, 676`, `cost/mod.rs:93` | `bundle ... --descriptor "wpkh(@0/0/*)" ...` -> 2; `import-wallet --blob garbage.json --format sparrow` -> 2; `compare-cost --miniscript "garbage("` -> 2; `gui-schema --classify-descriptor "garbage("` -> 2 |
| 2 | build-descriptor gate diagnostics (returned as `Ok(2)`, diagnostics printed) | build-descriptor | `build_descriptor.rs:189, 210, 239` | `build-descriptor --archetype kofn-recovery` -> 2 |
| 2 | I/O error on two specific flags | bundle `--descriptor-file`, build-descriptor `--spec` | `DescriptorParse` / `BuildDescriptorSpec` | `bundle --network mainnet --descriptor-file /nonexistent` -> 2; `build-descriptor --spec /nonexistent` -> 2 |
| 2 | repair: uncorrectable / post-correction decode failure | repair | `error.rs:690` | `repair --md1 $MDGARB $MD2 $MD3` -> 2; `repair --md1 $MD1BAD` (1 of 3 chunks) -> 2 |
| 2 | network mismatch (typed variant) | convert, silent-payment | `error.rs:687` | `convert --from xpub=$XPUB --to address --script-type p2wpkh --path m/0/0 --network testnet` -> 2 |
| 3 | future format | inspect, convert, import-wallet | `error.rs:671` | `inspect --mk1 <vector N12, version 1>` -> 3; `import-wallet --blob bsms2.txt --format bsms` (`BSMS 2.0`) -> 3 |
| 3 | compare-cost refusal (unsupported wrapper, cap, no conditions) | compare-cost | `cost/mod.rs:95-99` | `compare-cost --descriptor "wpkh($XPUB/0/*)"` -> 3; `--miniscript "or_d(...)" --max-conditions 1` -> 3 |
| 4 | verify mismatch (report still printed on stdout) | verify-bundle | `verify_bundle.rs:706, 988, 1231, 2112` | `verify-bundle --network mainnet --template bip84 --slot @0.phrase=- --bundle-json bundle.json <phrase2.txt` -> 4 |
| 4 | reference mismatch (hard error, stderr) | restore | `error.rs:692` | `restore --from phrase=- --template bip84 --expect-fingerprint deadbeef <phrase.txt` -> 4 |
| 4 | search found nothing | xpub-search (4 modes) | `error.rs:699-700` | `xpub-search path-of-xpub --target-xpub $XPUB --phrase-stdin <phrase2.txt` -> 4; `address-of-xpub ... --target-address <other>` -> 4; `passphrase-of-xpub ... --passphrase-candidates-file cands.txt` -> 4 |
| 4 | repair candidate, unverified or ambiguous (VERIFY-ME) | repair | `repair.rs:1749` | `repair --ms1 - <msbad.txt` -> 4; `repair --mk1 $MK1BAD` (1 of 2 chunks) -> 4 |
| 4 | partial decode (origin unspecified) | inspect | `inspect.rs:177` | source only |
| 4 | seed does not match the blob; BSMS Round-1 signature unverified in lenient mode | import-wallet | `error.rs:677`, `import_wallet.rs:447, 1408` | source only |
| 4 | bundle violates BIP-388 distinctness; descriptor re-parse failed | verify-bundle | `error.rs:646, 664` | source only |
| 5 | repair applied and verified by reassembly | repair | `repair.rs:1749` | `repair --md1 $MD1BAD $MD2 $MD3` -> 5; `repair --mk1 $MK1BAD $MK2` -> 5 |
| 5 | auto-repair short-circuit: the verb did NOT do its job; it printed a repair report and the corrected card instead. Only when stdout is a TTY or `MNEMONIC_FORCE_TTY=1`, and not under `--no-auto-repair` | inspect, convert, verify-bundle, xpub-search seed intake (the callers of `repair.rs:2075` `try_repair_and_short_circuit`: `cmd/convert.rs`, `cmd/inspect.rs`, `cmd/verify_bundle.rs`, `cmd/xpub_search/seed_intake.rs`). The manual also lists 5 for import-wallet (`41-mnemonic.md:1906`) and says 5 is unreachable from xpub-search (`:4339`); neither was run | `repair.rs:417, 2141`, `cli.rs:261` | `MNEMONIC_FORCE_TTY=1 mnemonic inspect --md1 $MD1BAD $MD2 $MD3` -> 5 (same command piped -> 1); `MNEMONIC_FORCE_TTY=1 ... verify-bundle ... --md1 $MD1BAD ...` -> 5 (piped -> 4) |
| 64 | clap usage error (unknown flag/verb, missing required, bad value, conflict) | all | `cli.rs:173` | `--bogus` -> 64; `bogusverb` -> 64; `convert` -> 64; `convert --to bogus` -> 64; `compare-cost --miniscript .. --descriptor ..` -> 64; `--account 2147483648` -> 64 |
| 64 | no arguments at all (help printed on stderr) | top level, `seedqr`, `xpub-search`, `seed-xor`, `slip39`, `ms-shares` | clap `DisplayHelpOnMissingArgumentOrSubcommand` | `mnemonic` -> 64; `mnemonic seedqr` -> 64 |
| 64 | non-UTF-8 argument | all | `cli.rs:157` | `mnemonic inspect $'md1\xff'` -> 64 |
| 101 | panic (Rust default; `main` installs no hook and no `catch_unwind`; no `panic = "abort"` in `Cargo.toml`) | any | none | inferred, not measured. `verify_message.rs:210-214` isolates BIP-322 verification with `catch_unwind` and turns an unwind into an ordinary exit-1 error. Tests assert "not 101" in 4 places. |
| 130 / 143 | SIGINT / SIGTERM (default disposition; the passphrase prompt's handler restores termios and re-raises, `passphrase_input.rs:321-333`) | any | none | `inspect --md1 - <fifo` + `kill -INT` -> 130; `kill -TERM` -> 143 |

EPIPE is not an error: `inspect ... | true` -> 0; `gui-schema | head -c 10` -> 0. Closed
stdout (`>&-`) -> 0. A failing stderr never changes the code (`2>/dev/full` -> verb's code).

`mnemonic` spawns no subprocess (`grep Command::new src/` is empty outside tests).

---

## B. Class matrix (`mnemonic`)

| # | Class | Code(s) |
|---|---|---|
| 1 | success | 0 |
| 2 | answer is "no" | **4**: verify-bundle (mismatch or `partial`), restore (`--expect-*`, `--cosigner`, `--expect-wallet-id`), xpub-search (all four modes), import-wallet (seed mismatch; lenient BSMS Round-1 unverified). **1**: verify-message invalid signature. **2**: import-wallet `--bsms-verify-strict` signature mismatch, BSMS MAC mismatch. **0**: `gui-schema --classify-descriptor` non-canonical; `restore --allow-mismatch` |
| 3 | invalid input artifact | **1**: checksum / alphabet / length (ms1, mk1, md1), bad xpub, bad address, bad phrase. **2**: wrong or unknown HRP, incomplete chunk set, structural wire rejects, share-vs-secret shape, foreign-wallet parse error, descriptor parse error, word-card decode refusal. **3**: newer wire version. **4**: verify-bundle reports an undecodable card as a failed check row. **5**: any of the above that BCH can fix, on a TTY (auto-repair). **0**: mk1 with up to 4 substitutions under `inspect` (see C5) |
| 4 | usage error | **64**: everything clap rejects, non-UTF-8 argv. **2**: argv guard firing on a malformed `--from`/`--slot` value (C3); runtime flag-combination checks (`restore --format` without `--template`, `derive-child --length`, `bundle` with no `--slot`, build-descriptor archetype params). **1**: runtime value checks that are `BadInput` (`seed-xor split --shares 1`, `ms-shares split --threshold 1`, `word-card --raid 9`, `export-wallet` with neither `--template` nor `--descriptor`, `import-wallet` format not detected) |
| 5 | no / empty input | **1**: empty stdin for a card, phrase, secret or miniscript (`inspect --md1 -`, `repair --ms1 -`, `convert --from phrase=-`, `compare-cost`, `word-card --from -`, `nostr --secret-stdin`, `slip39 combine --share -`). **2**: `build-descriptor` with empty stdin (JSON parse error). **64**: a required positional or flag absent |
| 6 | policy / safety refusal | **2**: secret on argv (pre-parse), `export-wallet` given a secret slot, xprv in an imported blob, `convert` one-way refusals, `derive-child` RSA, BSMS taproot, unsorted-multisig export. **1**: a few refusals typed `BadInput` (`export-wallet --format electrum` with taproot, `--format bip388` with NUMS, `wif` at mint, cross-class network on import). Unsafe file mode is a warning only, exit 0 (`final-word --json-out` prints `inherits umask`) |
| 7 | I/O error | **1**: missing/unreadable input file, unwritable output, `gen-man --out`, stdout write failure. **2**: missing file behind `bundle --descriptor-file` and `build-descriptor --spec`. EPIPE: the verb's own code |
| 8 | repair | applied + verified **5**; applied, unverified or ambiguous **4**; uncorrectable **2**; already valid **0**; empty stdin / I/O **1**. Auto-repair inside other verbs: **5** on a TTY only |
| 9 | external dependency failure | not applicable: no subprocess, no network |
| 10 | partial success / warnings only | warnings and notices: **0**. `verify-bundle` `result: partial` **4**; `inspect` md1 partial decode **4**; `import-wallet` lenient unverified signature **4** with full output; `restore` with no reference **0** plus `UNVERIFIED:` |
| 11 | internal error / panic | **101** (inferred), not caught by `main`; BIP-322 verify unwinds are caught and become **1** |
| 12 | other | `--help`/`--version`/`help` **0**; bare `mnemonic` **64**; SIGINT **130**, SIGTERM **143** |

---

## C. Collisions and surprises

Within `mnemonic`:

1. **5 means two things.** `repair` exit 5 = "I repaired it, here it is, verified".
   `inspect`/`convert`/`verify-bundle` exit 5 = "I did not run your command; here is a
   repair report instead" (`repair.rs:2141`). The JSON envelope even carries
   `"auto_repair_short_circuit":true,"exit_code":5` (`repair.rs:2237-2238`).
2. **The same command gives different codes on a TTY and in a pipe.**
   `inspect --md1 $MD1BAD $MD2 $MD3`: piped -> 1, `MNEMONIC_FORCE_TTY=1` -> 5.
   `verify-bundle` with the same bad card: piped -> 4, TTY -> 5. Decided by
   `repair.rs:411-420` (`stdout().is_terminal()` or the env var). mnemonic-gui sets the
   env var on every spawn (`mnemonic-gui/src/runner.rs:106-136`).
3. **2 is both "secret refused" and "your argument is malformed".**
   `convert --from bogus=1 --to xpub` -> 2 with "is secret key material -- 7 characters";
   with `--allow-argv-secret` the same line -> 64 (clap: unknown `--from` node).
   `bundle ... --slot bogus` -> 2 the same way. Also the guard's text offers
   `--phrase-stdin` for `bundle`, which has no such flag (-> 64).
4. **4 means three things**: a "no" answer (verify-bundle, restore, xpub-search), an
   unverified repair candidate (repair), and a partial result (inspect, verify-bundle
   `partial`, import-wallet lenient). mnemonic-gui labels every 4 as "VERIFY-ME".
5. **A corrupted mk1 passes `inspect` at exit 0 with no notice.**
   `inspect --mk1 $MK1BAD $MK2` -> 0 and prints the same output as the clean card;
   `mk decode $MK1BAD $MK2` also exits 0 but prints `note: BCH error-correction repaired
   this card while decoding`. The md1 equivalent exits 1 (or 5 on a TTY).
6. **The same class, different codes across verbs:**
   - "answer is no": 4 everywhere except `verify-message` (1, same code as its errors).
   - missing input file: 1 (`import-wallet --blob`, `verify-bundle --bundle-json`,
     `bundle --import-json`, `nostr --secret-file`) vs 2 (`bundle --descriptor-file`,
     `build-descriptor --spec`).
   - empty stdin: 1 everywhere except `build-descriptor` (2).
   - bad card: 1 for checksum failure, 2 for wrong HRP or missing chunk, 4 inside
     `verify-bundle`.
   - network mismatch: 2 (`NetworkMismatch`, `convert`) vs 1 (`bundle --network testnet`
     with a mainnet xpub: `--cosigner[0]: xpub network Main does not match`; and
     `ImportWalletNetworkClassMismatch`).
   - out-of-range numeric flag: 64 (`--account`, `--max-conditions 0`, `--feerate`) vs 1
     (`--shares`, `--threshold`, `--raid`) vs 2 (`derive-child --length`).
7. **`repair` and `inspect` disagree on a future-version mk1.** Vector N12 (version 1):
   `inspect --mk1` -> 3, `repair --mk1` -> 0 ("already valid", echoed).
8. **stdout failure hides an "answer is no" only in some verbs.** A `/dev/full` stdout turns
   `repair`'s 4 and 5 into 1, but `verify-bundle`'s 4 stays 4 (`out_sink.rs:93-96`,
   `cli.rs:270`).
9. **3 means two things**: future format, and `compare-cost` "cannot compare this".
10. **The v0.1 spec's own routing rule is not what the code does.**
    `design/SPEC_mnemonic_toolkit_v0_1.md:619-626` puts "BCH uncorrectable" in exit 2;
    `error.rs:486` maps `mk_codec::Error::BchUncorrectable` to 1 and `:524` maps md1
    checksum failure to 1 (measured 1). The same section says exit 4 is "only
    `BundleMismatch`"; seven variants plus five `Ok(4)` sites now use it.

Doc claims that disagree with the binary (quotes and paths in D.3).

---

## D. Documented codes

### D.1 `mnemonic --help` text (source of each line is the clap doc comment)

There is no exit-code section in `mnemonic --help` or in any verb's `--help`. Codes appear
only inside flag descriptions:

| Verb / flag | Text | Matches? |
|---|---|---|
| `bundle --import-json-index` | "Out-of- range -> exit 2" | **No**: measured 1 |
| `build-descriptor --json` | "On a gate failure: `{diagnostics: [...]}` with exit 2" | yes |
| `compare-cost --descriptor` | "refused with exit 3" | yes |
| `gui-schema --classify-descriptor` | "Exit 0 on parse success; exit 2 on parse failure" | yes |
| `import-wallet --bsms-encryption-token` | "MAC verify failure -> exit 2" | source only |
| `import-wallet --bsms-verify-strict` | "`BsmsSignatureMismatch` exit 2" | source only |
| `repair --max-subst` | "VERIFY-ME candidate (exit 4)" | yes (code path) |
| `restore --cosigner`, `--expect-fingerprint`, `--expect-xpub`, `--expect-wallet-id` | "exit 4 (unless `--allow-mismatch`)" | yes |
| `restore --allow-mismatch` | "loud banner, exit 0" | yes |
| `restore --format` | "`--format` with no `--template` ... -> exit 2" | yes |
| `verify-bundle --expect-wallet-id` | "overall `mismatch`, exit 4" | source only |

### D.2 Repo docs (mnemonic-toolkit)

- `design/SPEC_mnemonic_toolkit_v0_1.md:576-585` "§6.1 Exit-code table" (0, 1, 2, 3, 4, 64;
  no 5), `:606-615` variant mapping, `:619-630` routing principle.
- `design/SPEC_network_rule.md:435-456` "§8. Exit-code tables per tool (after)": a
  constellation-wide table of "codes today" per tool and a plan to add **exit 8 = network
  mismatch** to every tool. Not implemented in this binary (no 8 anywhere in `src/`).
  `scripts/spec-network-rule-gate.sh:35-40, 183-184` greps every repo to keep 8 unused.
  The table's "codes today": mnemonic 0,1,2,3,4,5,64; md 0,1,2,5; mk 0,1,2,3,4,5,64;
  ms 0,1,2,4,64; mr 0,1,2,5,101; mr-sitting 0,1,2,101; me 0,2,3,4; mt 0,1,2.
- `design/SPEC_docs_repair_exit_code_gui_manual.md:6` ground-truth repair table.
- `docs/architecture/README.md:56`: shared plumbing "does not make exit codes or secret
  handling identical".

### D.3 Toolkit manual, `docs/manual/src/` (paths relative to that directory)

The manual has chapters for `mnemonic`, `md`, `ms`, `mk`, `mr`, `mr-sitting` only. **There
is no `mt` chapter and no `me` chapter**, and no constellation-wide exit-code appendix.
Raw grep hits (487 lines) are in `/home/user/wt/ec/tmp-toolkit/manual-hits.txt`.

Exit-code tables and sections:

| Binary | Place | Content |
|---|---|---|
| mnemonic | `40-cli-reference/41-mnemonic.md:46` | argv refusal "at **exit 2**" |
| mnemonic import-wallet | `41-mnemonic.md:1897-1906` | table 0,1,2,3,4,5; stderr templates `:1912-1932`; refusal table `:2026-2042` |
| mnemonic seedqr | `41-mnemonic.md:3120-3123` | 0, 1 |
| mnemonic verify-message | `41-mnemonic.md:3475-3477` | 0 valid, 1 invalid, 1 malformed |
| mnemonic gui-schema | `41-mnemonic.md:3515-3525` | 0, 0, 2 |
| mnemonic repair | `41-mnemonic.md:3606-3614` | table 0,5,4,2,1; prose to `:3880` |
| mnemonic xpub-search | `41-mnemonic.md:4330, 4448, 4567, 4737` | four tables 0,1,4,64 |
| mnemonic compare-cost | `41-mnemonic.md:4942-4956` | 0,2,1,3,64 |
| mnemonic build-descriptor | `41-mnemonic.md:5166-5172` | 0, 2, 2 |
| mnemonic gen-man | `41-mnemonic.md:5217-5222` | 0, 1 |
| mnemonic word-card | `41-mnemonic.md:5389-5395` | 0, 1, 2 |
| mnemonic stdout failure | `41-mnemonic.md` "When stdout fails" (after `:5395`) | exits 1 |
| mnemonic verify-bundle | `41-mnemonic.md:353-360, 1045-1058, 1075-1078, 1100-1147, 1204-1211` | 2 for input errors, 4 mismatch/partial, 5 auto-fire |
| mnemonic restore | `41-mnemonic.md:449-466, 1476-1499, 1511-1567, 1676, 1748-1776` | 4 mismatch, 2 refusals, 1 some refusals |
| mnemonic (flags) | `41-mnemonic.md:381, 1021, 1245, 1300, 1483, 3354, 3392` | `--account` >= 2^31 "usage error, exit 64" |
| md repair | `42-md.md:686-693` | 0,5,2,1; `:697-723` prose |
| md gen-man | `42-md.md:802-807` | 0, 1 |
| md stdout failure | `42-md.md:813` | 1 |
| md (scattered) | `42-md.md:57, 110` (partial decode 4), `:178, 184, 208, 402, 526, 563` (refused, exit 1) | |
| ms repair | `43-ms.md:784-791`, `:793-837` | 0,4,2,1; "Never exit 5" |
| ms gen-man | `43-ms.md:889-894` | 0, 1 |
| ms (scattered) | `43-ms.md:47, 87, 90, 249, 255, 402, 410` (exit 64), `:368` (verify mismatch 4), `:411` (exit 1), `:900` (stdout 1) | |
| mk overall | `44-mk-cli.md:617-627` | the only whole-binary table: 0,1,2,3,4,5,64 |
| mk repair | `44-mk-cli.md:259-266`, `:269-322` | 0,5,2,1 |
| mk (scattered) | `44-mk-cli.md:35-56, 121, 498` (64), `:451-452` (verify 0/1/4), `:602` (`"exit_code": 1` in JSON), `:629`, `:665` (gui-schema/gen-man) | |
| mr overall | `45-mr.md:53-60` | 0, 1, 2, 5 (repair only) |
| mr (scattered) | `45-mr.md`, ~115 further lines; counted mentions: exit 1 x60, exit 2 x45, exit 0 x28; `:5399` (`letters` exit codes), `:5508` | |
| mr-sitting overall | `46-mr-sitting.md:839-845` | 0, 1, 2 |
| mr-sitting (scattered) | `46-mr-sitting.md:309, 374, 515` | |
| cross-binary | `60-appendices/65-bch-codex-primer.md:215-239` | repair 5/4/2/0 |
| cross-binary | `60-appendices/61-glossary.md:246` | "exit code `5`" |
| mnemonic | `45-foreign-formats.md:132-1024` (13 lines) | all "exit 2" refusals |
| mnemonic | `30-workflows/39-cross-format-conversion.md:114, 120, 284`; `3A-bsms-round1-verify.md:71-90`; `38-bip85-children.md:49, 74`; `35-recovery-paths.md:38`; `37-wallet-export.md:177, 347` | |

Manual claims that disagree with the binaries (all measured):

| Path:line | Claim | Measured |
|---|---|---|
| `40-cli-reference/41-mnemonic.md:393` | `--import-json-index` "out-of-range is `BadInput` exit 2" | `mnemonic bundle --network mainnet --import-json env.json --import-json-index 5` -> **1** (`BadInput` is 1 in `error.rs:643`) |
| `30-workflows/39-cross-format-conversion.md:284` | multi-entry envelope "without an index is `BadInput` exit 2" | `mnemonic bundle --network mainnet --import-json env2.json` -> **1**; `export-wallet --from-import-json env2.json --format descriptor` -> **1** |
| `40-cli-reference/41-mnemonic.md:1926, 1931` | stderr templates `supply --format <bsms\|bitcoin-core>` and `error: future format: bsms: version ...` | codes match (1, 3); the printed text differs (`<bitcoin-core|bsms|coldcard|...>`; `error: bsms reserved-not-emitted: version "2.0"; ...`) |
| `40-cli-reference/41-mnemonic.md:4330-4337` etc. | xpub-search tables list only 0, 1, 4, 64 | 2 is reachable (argv guard; `--ms1` on argv), not listed |
| `40-cli-reference/43-ms.md` `#ms-argv-guard` | states no code for the argv refusal | `ms verify <ms1 on argv> --phrase -` -> **1** (mnemonic's is 2, `41-mnemonic.md:46`) |
| `40-cli-reference/42-md.md` | no usage-error code and no whole-binary table | `md --bogus-flag` -> 2; `md decode -` (empty) -> 2 |

Confirmed by run: mk table (`mk decode <N12>` -> 3, `mk decode mk1nonsense` -> 1,
`mk decode -` -> 64, `mk decode --in /nonexistent` -> 64, `mk verify ... --origin-fingerprint
deadbeef` -> 4, `mk repair` 0/5/2, single chunk of a 2-chunk card -> 5); md repair 0/5/2;
ms repair 0/4, `ms repair --ms1 -` empty -> 2; `ms verify` mismatch -> 4; mr/mr-sitting
usage -> 2; all seven binaries `--help`/`--version` -> 0. Bare invocation: md, mt, mr,
mr-sitting, me -> 2; mk, ms, mnemonic -> 64. I did not survey the other binaries beyond this.

### D.4 GUI manual, `docs/manual-gui/` (paths relative to it)

Raw hits (256 lines): `/home/user/wt/ec/tmp-toolkit/gui-hits.txt`. Chapters that state
codes: `src/40-mnemonic/` 42-bundle (`:274, 382, 657-663, 682`), 43-verify-bundle (`:276`),
44-convert (`:476`), 45-export-wallet (`:327`), 4c-import-wallet (`:157, 175, 178, 244,
312`), 4d-restore (`:51, 97, 142, 286, 420-446, 496, 504, 626-638`), 4d/4e/4f/4g
xpub-search (`4d..:67, 274-276`; `4e..:78, 96`; `4f..:232-233`; `4g..:172-185`),
4e-build-descriptor (`:12, 128, 219, 290, 319-324`), 4e-seedqr-decode (`:87`),
4f-ms-shares-split (`:94, 100`), 4i-repair (`:79-119`), 4j-electrum-decrypt (`:51, 82`),
4j-nostr (`:99`), 4k-verify-message (`:14-16, 103-106`), 4m-compare-cost (`:42, 51`);
`src/50-md/` 53-encode (`:294`), 59-address (`:224`), 5A-repair (`:15, 72, 87-90`),
5C-compose (`:22-177`), 5D-shape-key (`:31-62`), 5E-descriptor (`:19, 152, 197, 238-241`;
`:197` "Exit `0` = spend-equal, `5` = not"), 5F-decompose (`:116-154`); `src/60-ms/`
62-inspect (`:84`), 63-encode (`:99, 270-276`), 64-decode (`:158-161`), 65-verify (`:5-7,
186-191`), 66-vectors (`:37`), 67-split (`:231-234`), 68-combine (`:97-100`), 69-derive
(`:223`), 6a-repair (`:66-136`), 6c-hashlock (`:129, 191, 233-237`); `src/70-mk/`
72-inspect (`:74-76`), 73-encode (`:123-253`), 74-decode (`:61-64`), 75-verify (`:5-8,
123-130`), 76-vectors (`:78-80`), 77-address (`:105-107`), 78-derive (`:66-69`), 79-repair
(`:66-131`); `src/30-tour/32-run-and-output.md:52, 61-69`; `src/80-troubleshooting/
82-binary-and-launch.md:66-233`; `src/10-foundations/14-secret-handling.md:386`;
`tutorial/30-j2-multisig.md:266`, `40-j3-degrading-vault.md:37`, `50-j4-taproot-twin.md:33, 213`.

GUI-manual claims that disagree with the binaries (measured):

| Path:line | Claim | Measured |
|---|---|---|
| `src/70-mk/72-inspect.md:74`, `74-decode.md:61`, `75-verify.md:6, 123`, `77-address.md:107`, `78-derive.md:69` | an unparseable mk1 is "exit 2" | `mk inspect mk1nonsense` -> **1**; `mk decode mk1nonsense` -> 1; `mk decode $MD1` (wrong HRP) -> 1; `mk verify mk1nonsense` -> 1; `mk address mk1nonsense` -> 1; `mk derive mk1nonsense --index 0` -> 1. The same manual's `80-troubleshooting/82-binary-and-launch.md:158-159` says they "now exit 1 ... where they exited 2" |
| `src/70-mk/73-encode.md:249` | `--from-md1` not a parseable md1 -> "exit 2" | `mk encode --xpub ... --from-md1 md1nonsense` -> **1** |
| `src/30-tour/32-run-and-output.md:63-69` | `mk inspect` with no arguments shows `exit: 2` | `mk inspect` -> **64** |
| `src/40-mnemonic/4f-ms-shares-split.md:94, 100` | K outside 2..=9 / N outside K..=31 "is a usage error (exit 64)" | `mnemonic ms-shares split --from phrase=- --threshold 1 --shares 3` -> **1**; `--threshold 10 --shares 12` -> 1; `--threshold 2 --shares 40` -> 1 |
| `src/40-mnemonic/42-bundle.md:274` | out-of-range `--import-json-index` "is `BadInput` (exit 2)" | **1** (as above) |
| `src/60-ms/65-verify.md:5-7, 188` | wrong HRP is "exit 2" format violation | not reached with my throwaway input (`ms decode`/`verify` of a 7-char `mk1...` string -> 1, length check first); unverified either way |

Consistent with runs: `60-ms/67-split.md:231-232` (ms split bad K/N -> 1), `60-ms/6a-repair.md`
(4), `50-md/5D-shape-key.md:60` (md clap 2), `70-mk/75-verify.md` mismatch 4 and bad
`--origin-fingerprint` 64, `4k-verify-message.md` (0/1/1), `4m-compare-cost.md` (3, 64).

---

## E. Tests that pin exit codes (mnemonic-toolkit)

`crates/mnemonic-toolkit/tests/`: 252 files, 221 of them assert on process status.

| Pattern | Count |
|---|---|
| `.code(0)` | 119 |
| `.code(1)` | 92 |
| `.code(2)` | 162 |
| `.code(4)` | 55 |
| `.code(5)` | 19 |
| `.code(64)` | 12 |
| `Some(N)` comparisons on `status.code()` (0/1/2/3/4/5/64) | about 128/35/49/6/31/5/6 |
| `.success()` | 1134 |
| `.failure()` (code not pinned) | 382 |
| unit tests in `src/` asserting `exit_code() == N` | 28 |

Heaviest files: `cli_build_descriptor.rs` (42), `bughunt_s1.rs` (39),
`cli_xpub_search_path_of_xpub.rs` (21), `cli_compare_cost.rs` (19), `cli_repair.rs` (18),
`cli_inspect.rs` (18), `bughunt_1119_unbounded_args.rs` (18),
`cli_xpub_search_account_of_descriptor.rs` (17). Named contract tests:
`src/out_sink.rs:215` `r21_exit_code_mapping`, `src/repair.rs:3144`
`indel_exit_code_precedence`, `tests/cli_p3_argv_refusal.rs:35` (argv refusal is 2, "not 3
and not 64"). Rough renumber cost: moving 2 touches ~210 assertions, 4 ~85, 1 ~125, 64 ~18,
5 ~25, 3 ~6.

---

## F. Consumers of `mnemonic`'s exit code

| Consumer | Path:line | What it reads |
|---|---|---|
| mnemonic-gui runner | `mnemonic-gui/src/runner.rs:222-235, 345-354` | `output.status.code()` into `RunResult.exit_code`; non-zero is `Ok`, shown to the user |
| mnemonic-gui exit badge | `mnemonic-gui/src/app_window.rs:461, 1316-1345` | **4 -> amber "VERIFY-ME", 5 -> green "Repair Applied (BCH auto-fire succeeded; corrected chunk on stdout)"**, anything else plain. The badge is not keyed on the binary or verb, so (inferred from the code, not run) `md descriptor --verify-against` exit 5 "not spend-equal" and `mk`/`md repair` 5 get the same green auto-fire text |
| mnemonic-gui version probe | `mnemonic-gui/src/cli_version.rs:122` | requires 0 |
| mnemonic-gui tree form | `mnemonic-gui/src/form/tree_form.rs:197, 327` | prints the code in an error string (build-descriptor `--emit-spec` / validate) |
| mnemonic-gui env | `mnemonic-gui/src/runner.rs:106-136` | sets `MNEMONIC_FORCE_TTY=1`, which is what makes exit 5 reachable from the GUI |
| mnemonic-gui tests | `mnemonic-gui/tests/` | 11 lines pin a specific code |
| toolkit script | `scripts/prove-envelope-consistency.sh:66, 72, 89, 96, 159` | expects `bundle --import-json` / `export-wallet` refusal = **1**, success = 0 |
| toolkit script | `scripts/spec-network-rule-gate.sh:35-40, 183-184` | keeps exit 8 unused in every repo and in the manual's tables |
| toolkit manual tests | `docs/manual/tests/root-shaped-test.sh:19-26`, `docs/manual/tests/verify-examples.sh` | pass/fail on the code of example runs |
| GUI manual transcripts | `docs/manual-gui/transcripts/*.cmd/.out/.err` | no exit code recorded (0 `exit:` lines) |
| other binaries | none found | `me`, `mr`, `mt`, refugium-wallet do not spawn `mnemonic` (the `"mnemonic"` hits in me-cli are a record-kind name) |

---

## Summary

1. `mnemonic` uses 0, 1, 2, 3, 4, 5, 64 (plus 101 on panic, inferred; 130/143 on signals);
   the mapping is `ToolkitError::exit_code()` (`error.rs:641-702`) plus eleven verb-level
   `Ok(n)` sites, a pre-clap argv guard (2), a clap override (2 -> 64) and a stdout-failure
   override (0/5 -> 1).
2. Codes are overloaded: 5 = verified repair AND "verb aborted, repair report printed"
   (TTY-only, so the same command returns 1 or 4 in a pipe and 5 on a terminal);
   4 = "no" answer, unverified repair and partial result; 2 = secret refusal, format
   violation, mode refusal, parse error and (two flags) missing file; 3 = future format
   and compare-cost refusal.
3. Same class, different codes across verbs: "answer is no" is 4 except `verify-message`
   (1); missing file is 1 except `--descriptor-file`/`--spec` (2); bad card is 1, 2, 3, 4,
   5 or (mk1 under `inspect`, silently corrected) 0.
4. Manual: per-verb tables only for `mnemonic`, whole-binary tables only for `mk`, `mr`,
   `mr-sitting`; no `mt` or `me` chapter. Disagreements found: `--import-json-index`
   "exit 2" is 1 (manual, GUI manual and `--help`); GUI manual says mk invalid card is 2
   (it is 1), `mk inspect` usage is 2 (it is 64), `ms-shares split` bad K/N is 64 (it is 1).
5. An existing spec, `mnemonic-toolkit/design/SPEC_network_rule.md:435-456`, already holds
   a per-tool "codes today" table and reserves exit 8 (network mismatch) for all tools;
   it is not implemented. Renumber cost in the toolkit: ~470 pinned assertions in 221
   test files; mnemonic-gui hard-codes 4 and 5.
