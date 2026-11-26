# Exit-code survey: `md` (descriptor-mnemonic, crate md-cli)

Surveyor: md. Source: `/home/user/wt/descriptor-mnemonic` at `42d87dd`. Binary:
`/home/user/wt/target/debug/md` (`md 0.21.2`, default features: `json` on,
`cli-compiler` OFF). All runs: stdin `</dev/null` unless shown, absolute paths.
Raw transcripts: `/home/user/wt/ec/tmp-md/m1.out`, `m2.out`, `m3.out`, `m4.out`
(scripts `m1.sh`..`m4.sh`, helper `run.sh`). `$T` = `/home/user/wt/ec/tmp-md`.
Paths below are relative to `/home/user/wt/descriptor-mnemonic/crates/md-cli/`
unless they start with a repo name.

Fixtures (all from `--help` examples or minted here from them):
- `OK` = `md1yqpqqxqq8xtwhw4xwn4qh` (keyless `wpkh(@0/<0;1>/*)`, `md decode --help`)
- `C1 C2 C3` = the 3-chunk keyed wpkh card from `md descriptor --help`
- `C2BAD` = chunk 2 of `md repair --help`'s example (one substitution, `7q5` for `7s5`)
- `P` = `md1yppqqxqj2s4dk6hk0wrt5n3` = `md encode 'tr(@0/<0;1>/*,pk(@1/<0;1>/*))'` (a "dead card": no canonical origin)
- `other.md1` = `md encode "wpkh(@0/48'/0'/0'/2'/<0;1>/*)" --key @0=<xpub6Dzhy…, from tests> --fingerprint @0=73c5da0a`
- xprv = BIP-32 test vector 1 master (published)

## A. Exit-code inventory

### How `main` maps outcomes to codes

`src/main.rs:930-961`, `fn main() -> ExitCode`. No `std::process::exit` anywhere
in the `md` binary; no `catch_unwind`, no panic hook, no `panic = "abort"` in
either `Cargo.toml`.

1. `Cli::try_parse()` (`main.rs:933`). On `Err(e)`: prints via `e.print()`,
   returns `e.exit_code()` (clap: **2** for a usage error, **0** for
   `--help`/`--version`/`help`), `main.rs:945`. Not `Error::exit()`; a custom
   arm so help/version go through the stdout write-error rule.
2. `dispatch(cli.command) -> Result<u8, CliError>` (`main.rs:982`):
   - `Ok(code)` -> that code. Verbs return 0, and three verbs return non-zero
     `Ok`: `decode`/`inspect` 4, `repair` 2 or 5, `descriptor` 5.
   - `Err(CliError::BadArg(m))` -> prints `md: {m}`, **2** (`main.rs:949-952`).
   - any other `CliError` -> prints `md: {e}`, **1** (`main.rs:953-956`).
3. Every return passes through `out::exit_code(code)` (`src/out.rs:76-84`): if a
   stdout write failed for a reason other than a closed pipe, it prints
   `md: cannot write the output: <error>` and turns **0, 4, 5 into 1**; 1 and 2
   are kept. A closed pipe (EPIPE) keeps the verb's own code.

`CliError` (`src/error.rs:3-134`) has 15 variants; there is no `exit_code()`
method on it. The whole mapping is the two arms above: `BadArg` = 2, the other
14 (`Codec`, `Render`, `TemplateParse`, `BadXpub`, `BadFingerprint`,
`BadOrigin`, `Compile`, `Mismatch`, `Compose`, `Seat`, `KeyReuse`,
`Unsupported`, `Decompose`, `VerifyAgainstUnreadable`) = 1. `BadArg` has about
58 construction sites (`main.rs` 11, `cmd/mod.rs` 9, `parse/keys.rs` 8,
`decompose/mod.rs` 5, `parse/template.rs` 4, others) and they are not all usage
errors (see C).

The distinct codes `md` can return: **0, 1, 2, 4, 5**, plus 101 (panic, source
only) and 128+n (signal, measured).

### Table

| code | meaning | verbs | where decided | measured |
|---|---|---|---|---|
| 0 | success | all | each `cmd::*::run` `Ok(0)` | `md decode md1yqpqqxqq8xtwhw4xwn4qh` -> 0; `md encode 'wpkh(@0/<0;1>/*)' --group-size 0` -> 0 |
| 0 | help / version | all | clap, `main.rs:945` | `md --help` -> 0; `md -h` -> 0; `md --version` -> 0; `md help` -> 0; `md help decode` -> 0; `md decode --help` -> 0 |
| 0 | repair: nothing to correct | repair | `cmd/repair.rs:120` | `md repair md1yqpqqxqq8xtwhw4xwn4qh` -> 0; `md repair C1 C2 C3` -> 0 |
| 0 | verify: card matches template (prints `OK`) | verify | `cmd/verify.rs:133` | `md verify md1yqpqqxqq8xtwhw4xwn4qh --template 'wpkh(@0/<0;1>/*)'` -> 0 |
| 0 | `--verify-against`: SPEND-EQUAL | descriptor | `cmd/descriptor.rs:185` | `md descriptor C1 C2 C3 --verify-against $T/same.md1` -> 0 |
| 0 | success with warnings on stderr | encode, descriptor, compose | n/a (warnings never change the code) | `md encode 'wsh(pk(@0/<0;1>/*))'` -> 0 (warning: card will partial-decode, "exit 4"); `md descriptor --template … --key @0=<xpub>` -> 0 (warning: no key origins); `md compose --wrapper wsh --path 2of3,unsorted --experimental` -> 0 |
| 1 | generic `CliError` (all variants but `BadArg`) | all | `main.rs:953-956` | see rows below |
| 1 | invalid artifact: bad checksum, wrong HRP, bad payload, incomplete chunk set, unsupported wire version | decode, inspect, bytecode, verify, descriptor, address, shape-key | `CliError::Codec` | `md decode not-a-valid-md-string` -> 1; `md decode md1qqqqqqqqqqqqqqqq` -> 1; `md decode mk1yqpqqxqq8xtwhw4xwn4qh` -> 1; `md decode C1` -> 1 (incomplete); `md decode C1 C2BAD C3` -> 1; `md decode ''` -> 1; same for inspect/bytecode; `md verify not-md --template …` -> 1; `md descriptor not-md` -> 1; `md address not-md` -> 1; `md shape-key garbage` -> 1 |
| 1 | verify MISMATCH (the "no" answer) | verify | `cmd/verify.rs:126` `CliError::Mismatch` | `md verify md1yqpqqxqq8xtwhw4xwn4qh --template 'wpkh(@0/<2;3>/*)'` -> 1; `md verify C1 C2 C3 --template "wpkh(@0/48'/0'/0'/2'/<0;1>/*)" --key @0=<other xpub> --fingerprint @0=73c5da0a` -> 1 |
| 1 | bad template / key / fingerprint value | encode, verify, descriptor, address | `TemplateParse`, `BadXpub`, `BadFingerprint`, `BadOrigin`, `Unsupported`, `KeyReuse` | `md encode 'this is not a template'` -> 1; `md encode ''` -> 1; `md encode 'wpkh(@0/<0;1>/*)' --key @0=notanxpub` -> 1; `… --fingerprint @0=zz` -> 1; `md encode 'wsh(multi(2,@0/<0;1>/*,@0/<0;1>/*))'` -> 1 (unsupported); `md encode 'wsh(multi(2,@0/<0;1>/*,@2/<0;1>/*))'` -> 1 |
| 1 | xprv / wrong-network key refused | encode, descriptor, decompose | `BadXpub`, `Decompose` | `md encode "wpkh(@0/48'/0'/0'/2'/<0;1>/*)" --key @0=<xprv>` -> 1 ("expected mainnet xpub version 0488B21E, got 0488ADE4"); `md decompose "wpkh(<xprv>/<0;1>/*)"` -> 1; `md decompose "<D>" --network testnet` -> 1 |
| 1 | compose refusals (all, including bad `--wrapper`/`--preset`/`--unspendable` values and "needs --experimental") | compose | `CliError::Compose` | `md compose --wrapper bogus --path 2of3` -> 1; `--path garbage` -> 1; `--path 5of3` -> 1; `--preset nosuch` -> 1; `--path 2of3,unsorted` -> 1; `--wrapper wsh --path 2of3 --unspendable liana` -> 1; `--wrapper tr --path 2of3 --unspendable bogus` -> 1 |
| 1 | seating refusals, including a missing/empty `--from-mk1-file` | descriptor, address | `CliError::Seat` | `md descriptor md1yqpqqxqq8xtwhw4xwn4qh --from-mk1 mk1garbage` -> 1; `… --from-mk1-file $T/nonexistent` -> 1; `… --from-mk1-file $T/empty.txt` -> 1; `… --seat @0=abcde` -> 1 |
| 1 | decompose refusals | decompose | `CliError::Decompose` | `md decompose garbage` -> 1; `md decompose ''` -> 1; `md decompose "<D>#aaaaaaaa"` -> 1; two descriptors -> 1; `addr(...)` -> 1 |
| 1 | `--verify-against` target does not decode | descriptor | `VerifyAgainstUnreadable` | `md descriptor C1 C2 C3 --verify-against garbage-not-md1` -> 1; `… --verify-against $T/tpl.txt` (a file, not md1) -> 1 |
| 1 | dead card on a verb with no partial path | bytecode, verify, descriptor, address, shape-key | `Codec` / bytecode's own text | `md bytecode P` -> 1; `md verify P --template 'tr(…)'` -> 1; `md descriptor P` -> 1; `md address P` -> 1; `md shape-key P` -> 1 |
| 1 | value out of range found after parsing | descriptor, address | `Codec` | `md descriptor C1 C2 C3 --chain 7` -> 1; `md address C1 C2 C3 --chain 5` -> 1; `md address C1 C2 C3 --index 2147483648` -> 1 |
| 1 | `--out FILE` not writable | encode (descriptor `--emit md1 --out`: source only) | `cmd/mod.rs:126` (`CliError::Compose`) | `md encode 'wpkh(@0/<0;1>/*)' --out /nonexistent-dir/x.md1` -> 1; `--out $T` (a directory) -> 1; `--out /dev/full` -> 1 |
| 1 | stdout write error (not EPIPE) on an otherwise 0/4/5 run | all, and `--help`/`--version` | `out.rs:76-84` | `md decode OK >/dev/full` -> 1; `md repair md1yq802ggqqpsqwgtua24e7ssf3 >/dev/full` -> 1 (would be 5); `md decode P >/dev/full` -> 1 (would be 4); `md descriptor C1 C2 C3 --verify-against $T/other.md1 >/dev/full` -> 1 (would be 5); `md --help >/dev/full` -> 1; `md --version >/dev/full` -> 1; `md decode OK 1</dev/null` -> 1 (EBADF) |
| 1 | policy compile error (`CliError::Compile`) | compile, encode `--from-policy` | `compile.rs`, `main.rs:1022` | source only (this build has `cli-compiler` off) |
| 2 | clap usage error | all | clap `exit_code()`, `main.rs:945` | `md` -> 2 (no subcommand; help goes to stderr); `md bogus` -> 2; `md --bogus-flag` -> 2; `md decode --bogus-flag` -> 2; `md decode` -> 2; `md verify OK` -> 2 (missing `--template`); `md encode --group-size notanumber …` -> 2; `md encode --network bogusnet …` -> 2; `md encode 'wpkh(…)' --in $T/x` -> 2 (conflict); `md address C1 C2 C3 --count 0` -> 2; `md descriptor C1 C2 C3 --chain x` -> 2; `md descriptor C1 C2 C3 --emit bogus` -> 2; `md decode $'\xff\xfe'` -> 2 (non-UTF-8 argv); `md gui-schema extra` -> 2 |
| 2 | `CliError::BadArg`: hand-written usage errors | all | `main.rs:949-952` | `md encode` -> 2 ("TEMPLATE required"); `md encode 'wpkh(…)' --unspendable-key abc` -> 2; `md encode 'wpkh(…)' --key garbage` -> 2; `md encode 'wpkh(…)' --key @0=X --key @1=Y` -> 2 (unreferenced key); `md encode 'wpkh(…)' --path "m/99999999999'"` -> 2; `md descriptor --template …` (no `--key`) -> 2; `md descriptor C1 C2 C3 --emit md1` -> 2 |
| 2 | `BadArg`: input file missing / unreadable / not UTF-8 / a directory | every `--in` verb, `--verify-against` | `cmd/mod.rs:62,88` | `md decode --in $T/nonexistent` -> 2; `md decode --in $T` -> 2; `md decode --in $T/bin.dat` -> 2; same for encode, verify, inspect, bytecode, repair, decompose |
| 2 | `BadArg`: empty input (empty file or empty stdin) | every card/template/descriptor reader | `cmd/mod.rs:71,91,326`, `decompose/mod.rs` | `md decode --in $T/empty.txt` -> 2; `printf '' \| md decode -` -> 2; same for inspect, bytecode, verify, repair; `md encode --in $T/empty.txt` -> 2; `printf '' \| md decompose -` -> 2; `md descriptor C1 C2 C3 --verify-against $T/empty.txt` -> 2 |
| 2 | `BadArg`: `--out` names the input | encode (descriptor: source only) | `cmd/mod.rs:181` | `md encode --in $T/tpl.txt --out $T/tpl.txt` -> 2 |
| 2 | `BadArg`: keyless card given to a verb that needs keys | descriptor, address | `cmd/descriptor.rs:156`, `cmd/address.rs:71` | `md descriptor md1yqpqqxqq8xtwhw4xwn4qh` -> 2; `md address md1yqpqqxqq8xtwhw4xwn4qh` -> 2 |
| 2 | `BadArg`: descriptor text does not parse | shape-key `--descriptor` | `cmd/shape_key.rs:19` | `md shape-key --descriptor garbage` -> 2 |
| 2 | `BadArg`: output directory cannot be made / written | gen-man, vectors | `cmd/gen_man.rs:23,27`, `cmd/vectors.rs:26,228` | `md gen-man --out /proc/nonexistent/man` -> 2; `md gen-man --out $T/empty.txt` -> 2; `md vectors --out /proc/nonexistent/vec` -> 2 |
| 2 | `BadArg`: feature not compiled in | compile, encode `--from-policy` | `main.rs:1036,1169` | `md compile 'pk(A)' --context tap` -> 2; `md encode --from-policy 'pk(A)' --context tap` -> 2 |
| 2 | repair atomic-fail: uncorrectable, wrong HRP, parse reject, incomplete set, dead card, unsupported wire version (clean card or multi-string) | repair | `cmd/repair.rs:111,193,202` (`Ok(2)`, not `CliError`) | `md repair md1qqqqqqqqqqqqqqqqqqqqqq` -> 2; `md repair not-a-valid-md-string` -> 2; `md repair mk1yqpqqxqq8xtwhw4xwn4qh` -> 2; `md repair ''` -> 2; `md repair C1` -> 2; `md repair C1 C2BAD` -> 2; `md repair P` -> 2; `md repair OK OK` -> 2 |
| 4 | partial decode ("VERIFY-ME"): the card decodes but its origin is unspecified; template is on stdout | decode, inspect (text and `--json`) | `cmd/decode.rs:66,133`, `cmd/inspect.rs:99,189` | `md decode md1yppqqxqj2s4dk6hk0wrt5n3` -> 4; `--json` -> 4; `md inspect md1yppqqxqj2s4dk6hk0wrt5n3` -> 4; `--json` -> 4 |
| 5 | REPAIR_APPLIED: at least one chunk corrected; also a single string corrected at an unsupported wire version | repair | `cmd/repair.rs:120,234` | `md repair md1yq802ggqqpsqwgtua24e7ssf3` -> 5; `md repair C1 C2BAD C3` -> 5; `--json` -> 5; `md repair md1yqpqqxqq8xtwhw4xwn4qq` -> 5; `md repair … \| true` -> 5 |
| 5 | `--verify-against`: NOT spend-equal | descriptor | `cmd/descriptor.rs:185` | `md descriptor C1 C2 C3 --verify-against $T/other.md1` -> 5; `--json` -> 5; `md descriptor C1 C2 C3 --verify-against md1yqpqqxqq8xtwhw4xwn4qh` -> 5 |
| 101 | panic (Rust default; `main` does not catch) | any | no `catch_unwind`; 543 `unwrap()`/`expect(` lines in `src/` (not all reachable) | source only; no panic reproduced |
| 130 / 143 / 129 | killed by SIGINT / SIGTERM / SIGHUP; no handler installed | any | default disposition | `timeout --preserve-status -s INT 0.5 md decode - < <(sleep 3)` -> 130; `-s TERM` -> 143; `-s HUP` -> 129 |

Second binary in the crate, `src/bin/gen_ap2_grind.rs` (a generator, not
installed as `md`): `fn main() -> ExitCode`, ends `ExitCode::from(out::exit_code(0))`
(line 327). Not surveyed further.

## B. Class matrix

| # | class | code(s) |
|---|---|---|
| 1 | success | 0 on every verb |
| 2 | "answer is no" | `verify` MISMATCH: **1** (same code as a decode failure). `descriptor --verify-against` NOT spend-equal: **5**. No other verb has a yes/no answer |
| 3 | invalid input artifact | **1** on decode, inspect, bytecode, verify, descriptor, address, shape-key (card), decompose, `--verify-against` target, `--from-mk1` string. **2** on `repair` (every decode failure is the atomic-fail code) and on `shape-key --descriptor <bad text>` |
| 4 | usage error | **2** (clap, and `BadArg`). Exception: bad *values* for `md compose --wrapper/--path/--preset/--unspendable` are **1** (those flags are free strings validated in `CliError::Compose`) |
| 5 | no input / empty input | **2** on every verb (empty stdin via `-`, empty `--in` file, empty `--verify-against` file, missing positional). Exceptions: an empty-string argument is treated as an artifact: `md decode ''` -> **1**, `md encode ''` -> **1**, `md decompose ''` -> **1**, `md repair ''` -> 2; empty `--from-mk1-file` -> **1** (seating) |
| 6 | policy / safety refusal | xprv where an xpub is expected: **1** (it is a version-bytes mismatch, not a named secret refusal; `md` takes no secrets). Wrong-network key: **1**. `--out` naming the input: **2**. Needs `--experimental` / `--md-only` / `--allow-invalid-miniscript`: **1** (measured for compose `--experimental`; the other two source and manual only). BIP-388 key reuse / unsupported shape: **1**. Keyless card to descriptor/address: **2**. No file-mode checks on inputs |
| 7 | I/O error | input file missing/unreadable/directory/not UTF-8 (`--in`, `--verify-against` file): **2**. `--from-mk1-file` missing: **1**. `--out FILE` unwritable (encode): **1**. Output dir unwritable (gen-man, vectors): **2**. stdout write error: **1** (overrides 0/4/5; keeps 1/2). stdout EPIPE: the verb's own code. stderr write error: ignored (`md decode not-md 2>/dev/full` -> 1, `md decode OK 2>&-` -> 0) |
| 8 | repair | nothing to fix **0**; corrected **5** (always, chunked or not; `md` has no "unverified" 4 for repair); uncorrectable or any other failure **2**; I/O on input **2**; stdout write error **1** |
| 9 | external dependency failure | none: `md` spawns nothing and uses no network. "Feature not compiled in" (`compile`, `--from-policy`): **2** |
| 10 | partial success / warnings-only | warnings on stderr never change the code (**0**). Partial decode of a dead card: **4** on decode/inspect only; the same card is **1** on bytecode/verify/descriptor/address/shape-key and **2** on repair |
| 11 | internal error / panic | not caught; Rust default **101** (source only). The stdout-panic class was removed by `out.rs` (FOLLOWUP `md-output-sink`) |
| 12 | other | `--help`/`--version`/`help`: **0** (stdout). `md` with no subcommand: **2**, help on stderr. Signals: 128+n. Non-UTF-8 argv: **2** |

## C. Collisions and surprises

Same code, two meanings, in `md`:

1. **2 = usage error AND `repair` atomic-fail.** `md repair --bogus` -> 2 and
   `md repair md1qqqqqqqqqqqqqqqqqqqqqq` -> 2. A caller cannot tell "I called
   it wrong" from "this card is uncorrectable". Also `md repair --in
   $T/nonexistent` -> 2 and `printf '' | md repair -` -> 2.
2. **2 = usage error AND input-file I/O error AND empty input AND a content
   refusal.** `BadArg` carries all of: missing file (`--in $T/nonexistent`),
   directory, non-UTF-8 file, empty file/stdin, keyless card to
   `descriptor`/`address`, unparseable `shape-key --descriptor` text,
   unreferenced `--key @1`, output-directory failures in `gen-man`/`vectors`,
   and "feature not compiled in".
3. **1 = invalid artifact AND verify MISMATCH AND bad key/template AND seating
   refusal AND `--out` write failure AND stdout write failure.** In particular
   `md verify` cannot distinguish "the card does not match" from "the card did
   not decode" by code: both 1.
4. **5 = REPAIR_APPLIED (repair) AND NOT spend-equal (descriptor
   `--verify-against`).** Deliberate ("reserved-5 precedent", `main.rs:609`),
   different verbs, but 5 is "a good outcome with a caveat" on one and "the
   answer is no" on the other.
5. **4 = partial decode here; the same number is `ms repair`'s unverified
   repair and `me sysw pack`'s code** (cross-binary, per §6f; not re-measured
   here). `md repair` never returns 4.

Same class, different codes across verbs:

6. **Invalid artifact: 1 everywhere, 2 on `repair`.** `md decode not-a-valid-md-string`
   -> 1; `md repair not-a-valid-md-string` -> 2. Same for wrong HRP, incomplete
   set, unsupported wire version, dead card.
7. **"Answer is no": 1 on `verify`, 5 on `descriptor --verify-against`.**
8. **Missing input file: 2 for `--in`, 1 for `--from-mk1-file`.**
   `md decode --in $T/nonexistent` -> 2; `md descriptor OK --from-mk1-file
   $T/nonexistent` -> 1 (wrapped in "seating refused:"). Empty file likewise
   (2 vs 1).
9. **Output not writable: 1 for `encode --out`, 2 for `gen-man --out` /
   `vectors --out`.**
10. **Bad flag value: 2 for clap-validated flags, 1 for `compose`'s
    string-typed flags** (`md compose --wrapper bogus --path 2of3` -> 1, while
    `md descriptor … --emit bogus` -> 2). And a value that parses but is out of
    range for the card is 1 (`--chain 7` -> 1, `--chain x` -> 2).
11. **Dead card `P`: 4 on decode/inspect, 1 on bytecode/verify/descriptor/
    address/shape-key, 2 on repair.**
12. **Bad descriptor text: 1 on `decompose garbage`, 2 on `shape-key
    --descriptor garbage`.**
13. **Empty string argument vs empty stdin:** `md decode ''` -> 1,
    `printf '' | md decode -` -> 2.

Other surprises:

14. **A printed verdict with a different exit code.** `md descriptor C1 C2 C3
    --verify-against $T/other.md1 --chain 7` prints `--verify-against: NOT
    spend-equal …` on stderr, then fails on `--chain` and exits **1**, with
    nothing on stdout. The verdict is emitted (`cmd/descriptor.rs:184`) before
    the render that can still fail (`:211-218`). Likewise with `>/dev/full` the
    NOT-equal verdict is on stderr and the code is 1.
15. **Closed stdout loses the output at exit 0.** `md decode
    md1yqpqqxqq8xtwhw4xwn4qh >&-` -> **0** with nothing written. `out.rs`
    catches a read-only fd 1 (`1</dev/null` -> 1) but with fd 1 closed
    `fcntl(F_GETFL)` returns -1, the check is skipped (`out.rs:45`,
    `fl >= 0 && …`), and std reports EBADF as success. Not pinned by
    `tests/output_errors.rs` (its "closed" cases are a closed *pipe reader*).
16. **`--in -` is not stdin.** `printf … | md decode --in -` -> 2 (`--in -: No
    such file or directory`); stdin is the positional `-`. And `md encode -`
    takes `-` as the template text (-> 1, "no @i placeholders"); `encode` has
    no stdin form.
17. **`md encode 'wpkh(…)' --out -` -> 0 and writes a file named `-`** in the
    current directory (found at `$T/-`), not stdout. (The `--out`-names-input
    guard does treat standard-stream names specially; a plain `-` with an argv
    template is just a filename. Inferred from the file appearing, not from
    source.)
18. **`md repair` exits 5 for a non-chunked single-string correction that has
    no oracle.** `md repair md1yqpqqxqq8xtwhw4xwn4qq` -> 5, and
    `md1yqpqqxqq8xtwhw4xwnqqq` (two substitutions) -> 5. Under the D26 rule
    quoted in §6f this is the exit-4 case; `mnemonic repair --md1` demotes it
    to 4 since toolkit v0.86.0. Known and open: FOLLOWUP
    `md-cli-non-chunked-single-string-repair-demote`
    (`descriptor-mnemonic/design/FOLLOWUPS.md:2276`), and the manual says so
    (`42-md.md:709-727`).
19. **`md` with no arguments exits 2** (clap `arg_required_else_help`-style
    error; help text on stderr), not 0.
20. **Running as root here**, so "file exists but permission denied" could not
    be measured (`chmod 000` file read fine). By source it is the same
    `BadArg` -> 2 as a missing file (`cmd/mod.rs:62`). Inferred.

Docs that disagree with the binary:

21. Manual `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md:802-807`
    (`md gen-man`, "Exit codes"): "output-dir create / write I/O error | `1`".
    Measured: `md gen-man --out /proc/nonexistent/man` -> **2**.
22. Manual `42-md.md:692` and `crates/md-cli/README.md:131` (`md repair`): "`1`
    | I/O error or other generic failure." Measured: an input I/O error is
    **2** (`md repair --in $T/nonexistent` -> 2); the only measured 1 on
    `repair` is a stdout write failure. (`emit_json` failure -> 1 by source.)
23. `design/SPEC_v0_15_cli.md:67-73`: "1 = user error (bad input, `verify`
    mismatch, decode failure); 2 = clap usage error / internal error". Codes 4
    and 5 are not in it; an internal error (panic) is 101, not 2; and many
    non-clap, non-internal conditions are 2 (items 1, 2).
24. `md repair --help` (`main.rs:889-900`) lists 0, 5, 2 only; it does not say
    that a missing/empty input is also 2 or that a write failure is 1.
25. §6f's `md` row (usage 2, invalid artifact 1, repair applied 5, repair
    uncorrectable 2) and CLI-14's "`md decode -` on empty stdin exits 2" are
    both confirmed by these runs. §6f's note "md encode with no template exits
    2 = clap's 2" is confirmed (`md encode` -> 2, via `BadArg`, `main.rs:1052`).

## D. Documented codes

`--help` (source = `src/main.rs` doc comments):
- `main.rs:889-900` `md repair` long help: 0 / 5 (REPAIR_APPLIED, incl. single
  string at unsupported wire version) / 2 (atomic-fail).
- `main.rs:902` `md repair` `after_long_help`: "the WHOLE call exits 2 with the
  failing chunk index named on stderr".
- `main.rs:609-612` `md descriptor --verify-against`: "0 = spend-equal; 5 = NOT
  spend-equal …; 1/2 = errors, unchanged".
- `main.rs:126`, `main.rs:576` `--out`: "Refused (exit 2) when FILE is the
  [input]".
- `main.rs:64`: a clap-level refusal "(exit 2), before command dispatch".
- Printed at run time: `cmd/encode.rs:1096` warning "(origin unspecified, exit
  4)"; `cmd/partial.rs:32` note "exit 4 (VERIFY-ME)".

Source comments that state codes: `out.rs:5-10,72-75` (101 for std's macros; 1
for a write error; 0/4/5 -> 1); `error.rs:47-49,60-61,75-76,95-96,109-110,
123-125` (which variants are 1 and why not 2); `cmd/repair.rs:14-26`;
`cmd/descriptor.rs:171,389`; `cmd/mod.rs:111-113`.

Repo docs:
- `descriptor-mnemonic/crates/md-cli/README.md:45` ("`md verify` … Exit 0 on
  match, 1 on mismatch"), `:54`, `:92` ("0 = match, 1 = mismatch"),
  `:126-131` (repair table 0/5/2/1).
- `descriptor-mnemonic/design/SPEC_v0_15_cli.md:63` ("Unknown subcommand or arg
  -> exit 2"), `:67-73` (the 0/1/2 table), `:282` (names `tests/exit_codes.rs`).
- `descriptor-mnemonic/design/FOLLOWUPS.md:2276` (repair demote, open),
  `:3432` (`md-output-sink`: write error used to panic with 101).

Toolkit manual, `mnemonic-toolkit/docs/manual/src/40-cli-reference/42-md.md`:
- `:57` encode: dead shape partial-decodes "(exit 4 …)".
- `:105-112` decode: dead card "exits **4**"; corrupted dead card "exit ≠ 0".
- `:122` keyless-template advisories "still complete at exit 0".
- `:184` `--allow-invalid-miniscript`: without it "refused, exit 1".
- `:208` chain index out of range "exit 1".
- `:231` `--verify-against`: 0 / 5 / "1/2 = errors unchanged".
- `:402` compile: more than 8 nested `or`s "refused with exit 1".
- `:526`, `:563` compose `--md-only`: refuses "exit 1".
- `:686-692` repair "Exit codes" table 0/5/2/1; `:694-727` the D26 rule (5 vs
  4) and the non-chunked divergence; `:744` "Exit code: `5`"; `:752` "the WHOLE
  call exits 2".
- `:802-807` gen-man "Exit codes" (0; I/O error 1, wrong, see C21).
- `:811-813` "When stdout fails": closed pipe keeps the verb's code; other
  write errors exit 1; an already failing verb keeps its code.
- No exit-code statement for `md verify`, `inspect`, `bytecode`, `address`,
  `decompose`, `shape-key`, `vectors`, `gui-schema` in the manual.

Cross-repo: `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md` §6f
(`md` row: 2 / 1 / 5 / 2); `/home/user/wt/cc/shared.md` CLI-14.

## E. Tests that pin exit codes

All in `descriptor-mnemonic/crates/md-cli/tests/` (91 `.rs` files; 86 files in
`tests/`+`src/` contain an exit assertion of some kind). Literal-code counts by
grep (approximate; table-driven tests that pass the code as a variable are
undercounted):

| assertion | count |
|---|---|
| `.code(1)` / `code(), Some(1)` | 41 + 22 |
| `.code(2)` / `code(), Some(2)` | 11 + 9 |
| `.code(0)` / `code(), Some(0)` | 3 + 20 |
| `Some(4)` (partial decode) | 5 (4 as `code(), Some(4)`) |
| `Some(5)` (repair, verify-against) | 3 (2 as `code(), Some(5)`) |
| `.failure()` (any non-zero; survives a renumber) | 47 |
| `.success()` | 292 |

So roughly **110 literal non-zero pins** (about 63 on 1, about 20 on 2, about
10 on 4/5), plus table-driven rows. Files that matter most for a renumber:
- `tests/exit_codes.rs` (4 tests: no args 2, unknown subcommand 2, bad template
  1, bad string 1).
- `tests/output_errors.rs` (14 code sites; the 0/4/5 -> 1 rule and
  closed-reader rows, codes passed as table values; also `src/out.rs:108-124`
  unit test over the code set).
- `tests/cli_repair.rs` (9), `tests/cli_repair_dead_card_strict.rs` (3): 0/5/2.
- `tests/partial_decode.rs` (14): exit 4.
- `tests/r3_verify_against.rs` (11): 0/5/1/2.
- Heaviest by assertion count (mostly `.success()`/1/2): `cli_compose_preset.rs`
  63, `seating_vectors.rs` 45, `cmd_encode.rs` 38, `n2_emit_md1.rs` 24,
  `cmd_address.rs` 22, `bughunt.rs` 22, `cli_output_class.rs` 19,
  `cli_p1_origin_key.rs` 18, `cli_compose.rs` 17.
- Gate scripts under `descriptor-mnemonic/scripts/` (`fix-bh1119-md-out-writes-gate.sh`
  line 8: compares "exit code, stdout, exact stderr, file bytes";
  `gen-compose-golden.sh:66-69` branches on `returncode == 0`).

## F. Consumers (spawn `md` and read its exit code)

- **mnemonic-gui** (spawns the pinned `md`):
  `mnemonic-gui/src/runner.rs:222,345` capture `status.code()` for every
  subprocess; `mnemonic-gui/src/app_window.rs:1316-1345` `render_exit_badge`
  gives **4** an amber "VERIFY-ME" badge and **5** a green "repair applied"
  badge for any CLI, all else a plain label. Consequence for `md`:
  `md descriptor --verify-against` NOT spend-equal (5) would show the green
  repair badge (inferred from the code; not run). `cli_version.rs:122`
  requires 0 from `--version`; `schema_check.rs:368,397` require success from
  `gui-schema`. `form/tree_form.rs:173` is parse-keyed, not code-keyed.
- **mnemonic-engrave**: `demo/sh2/rehearse.sh:58-60` (`md decode "$BAD"` must
  fail; `md repair` must exit exactly **5**);
  `scripts/policy-differential.py:366-372` (`_md` returns `returncode`, used as
  encode ok / refused); `scripts/descriptor-seam-vectors/gen.py:47,60,67,148`
  and `design/journeys/restore_from_plates.py:171` (non-zero = failure; which
  of these calls is `md` vs a sibling not checked line by line).
- **mnemonic-toolkit**: `docs/manual/tests/verify-examples.sh` and
  `docs/manual/tests/lint.sh:164,181` run `$MD_BIN` (help and transcripts;
  success/failure only); `crates/mnemonic-toolkit/tests/cli_cross_tool_differential.rs:129,461`
  runs `MD_BIN` (`md encode`, ignored unless set; CI
  `.github/workflows/cross-tool-differential.yml`). The toolkit itself links
  md-codec as a library and does not spawn `md`.
- **descriptor-mnemonic**: its own tests (E) and `scripts/gen-compose-golden.sh:66`.
- No consumer found that branches on `md`'s **1 vs 2**. The only specific
  codes anyone reads are **5** (rehearse.sh, GUI badge) and **4** (GUI badge).
