# Exit-code survey: `mr` and `mr-sitting` (surveyor "mr")

Worktree `/home/user/wt/mnemonic-refugium` at `3cabbbf12`; binaries
`/home/user/wt/target/debug/{mr,mr-sitting}` (`mr 0.1.0-alpha.2 (3cabbbf1)`,
`mr-sitting 0.1.0-alpha.2 (3cabbbf1)`). Read-only; nothing built or edited.
Scratch and raw run logs: `/home/user/wt/ec/tmp-mr/` (`m1.out`, `m2.out`,
`s1.out`, scripts `m1.sh`, `m2.sh`, `s1.sh`, `sig.py`, `sig2.py`).
Paths below are relative to the worktree unless absolute. "measured" = I ran
it and recorded `$?`; "source only" = read, not run.

Shorthand used in commands:
- `MR` = `/home/user/wt/target/debug/mr`, `S` = `/home/user/wt/target/debug/mr-sitting`, stdin `</dev/null` unless shown.
- `$C` = the 4-chunk testnet card of corpus vector `m8_chunk_vec_chunked_card`
  (`mr1l900ypsqqk9u…tl6td mr1l900ypsw2jqr…lww5g mr1l900yps4cqsx…mau0m mr1l900ypslpdxh…t9k0d`; set id `edf1721428f5fcfd`, digest `d4ed27fe1c257396`).
- `$BAD` = the same card with chunks 0 and 2 from `m8_chunk_vec_repair_corrected` (3 substitutions).
- `A24` = BIP-39's published `abandon ×23 art` vector. Vectors came from `MR vectors --out /home/user/wt/ec/tmp-mr/vec` (exit 0).

## A. Exit-code inventory

### A.0 How the codes are produced

**The scheme (both binaries): 0 success, 1 refusal/failure, 2 usage; `mr repair` alone 0 / 5 / 2; 101 worker panic.** No 3, 4, 64 anywhere.

Constants — `crates/mr-codec/src/app/exit_codes.rs:1-17` (re-exported by `crates/mr-cli/src/exit_codes.rs:11`):
`SUCCESS=0`, `FAILURE=1`, `USAGE=2`, `REPAIR_CLEAN=0`, `REPAIR_CORRECTED=5`, `REPAIR_FAILED=2`.
Duplicated in the library as `mr_codec::repair::{EXIT_CLEAN=0, EXIT_CORRECTED=5, EXIT_FAILED=2}` (`crates/mr-codec/src/repair.rs:38-43`, `Repair::exit_code()` `:77`); the unit test `exit_codes_pinned` (`crates/mr-cli/src/exit_codes.rs:18-43`) asserts both sets and that clap's `exit_code()` is 2.

Error type — there is **no error enum with an `exit_code()` mapping**. `Failure { code: u8, message, keep }` (`crates/mr-codec/src/app/common.rs:8-21`) carries the code; only two constructors: `Failure::fail` → 1 (`:24-30`), `Failure::usage` → 2 (`:33-39`). `impl From<mr_codec::Error> for Failure` (`:42-57`) maps **every** library error to 1. `repair` overrides the code to 2 by struct update (`crates/mr-cli/src/repair.rs:24-27`, `:47-50`). Verbs return `Result<u8, Failure>`; an "answer is no" is `Ok(1)` (report printed, no `mr:` line needed), a refusal is `Err(Failure)`.

`mr` `main` (`crates/mr-cli/src/main.rs`): returns `std::process::ExitCode`; no `std::process::exit`, no `clap::Error::exit()`.
- `:23-36` help/version (`!e.use_stderr()`): printed through the one writer; exit 0, or **1** if stdout cannot be written.
- `:37-49` clap error: custom renderer `common::clap_message` (never echoes the typed value), exit `e.exit_code()` (=2), fallback `USAGE`.
- `:54-64` every verb runs on a worker thread (`par::STACK_BYTES`); `worker.join().unwrap_or(101)` — a panic is **101**; hook `install_panic_hook` (`crates/mr-cli/src/lib.rs:372-374`) prints a fixed line (no payload).
- `mr_cli::finish` (`crates/mr-cli/src/lib.rs:377-411`): writes the report, prints `mr: <message>`, returns the code; a stdout write error other than a closed pipe prints `mr: cannot write the output: …` and turns **0 or 5 into 1** (other codes kept). A usage failure (code 2) also drops the "usage-quiet" trailer lines (`:382`), so the code changes printed output, not only the status.
- Library twin: `mr_codec::api` `Outcome<T>::code()` (`crates/mr-codec/src/api.rs:51-56`) returns the same number; `api::finish` (`:221-229`) repeats the usage-quiet rule.

`mr-sitting` `main` (`crates/mr-sitting/src/main.rs`): `ExitCode::from(code)`; worker thread, `unwrap_or(101)` (`:224-244`); clap error rendered by clap itself (`e.render()`, may echo the value) with `e.exit_code()` fallback 2 (`:274-285`); non-Linux build: one line, exit 2 (`:9-13`). `root`/`child` reuse `mr_cli::…::run` + `mr_cli::finish` (`:286-311`); the other verbs return a bare `u8` from `Refusal(String, u8)` (`sit.rs:12`), `verify::Stop(String, u8)` (`verify.rs:39`), `kit.rs` helpers `err()`→2 (`:62-65`), `failed()`→1 (`:307-313`). Signal path of a session: `libc::_exit(1)` (`wipe.rs:870`).

### A.1 `mr`

| code | meaning | verbs | decided at | measured |
|---|---|---|---|---|
| 0 | success | all | each verb's `Ok(SUCCESS)`; `main.rs:34` for help | `MR --help`→0; `MR --version`→0; `MR help`→0; `MR --network testnet decode $C`→0; `… units $C`→0; `… verify $C`→0 ("verify: passed"); `… verify $C --set-id edf1721428f5fcfd --digest d4ed27fe1c257396`→0; `… current $C --height 1000000 --time 1900000000 --lead 100`→0; `… export $C --epochs 0..0`→0; `… export $C --exhaustive`→0; `… member $C --epoch 0 --slot 0 --height 1000000 --time 1900000000`→0; `… address $C --epoch 0 --slot 0 --index 0 --height 1000000 --time 1900000000`→0; `MR procedure`→0; `MR procedure the-offline-computer`→0; `MR vectors --out /home/user/wt/ec/tmp-mr/vec`→0. `create`/`upgrade`/`root`/`child`/`lost-or-stolen` success: source + tests only |
| 0 | repair: nothing corrected (also when the strings are **no card** or a lone chunk — warning only) | repair | `repair.rs:64-68` | `MR repair $C`→0; `MR repair mr1qqqqqqqqqqqqqqqq5saqe93dj7z0t`→0 with "warning: the string passes the checksum but is not a card"; `MR repair <chunk 1 alone>`→0 with "warning: this string is chunk 1 of a 4-chunk card" |
| 0 | closed pipe / closed stdout / unwritable stderr are silent | all | `out.rs:23-33` | `MR --network testnet decode $C \| head -1`→0; `MR procedure >&-`→0; `MR … decode $C 2>/dev/full`→0 |
| 1 | "answer is no" (`Ok(FAILURE)`) | verify, child --check | `mr-codec/src/app/verify.rs:851-853`; `mr-cli/src/child.rs:424-428` | `MR --network testnet verify $C --set-id edf1721428f5fcfe`→1; `… verify $C --digest d4ed27fe1c257397`→1; `printf 'A24\n' \| MR … verify $C --seeds -`→1 |
| 1 | invalid artifact (checksum, HRP, lone chunk, header) | every reading verb except repair | `From<mr_codec::Error>` `app/common.rs:42-57` | `MR --network testnet decode $BAD`→1; `… decode <chunk 1>`→1; `… decode notacard`→1; `… decode ""`→1; `… decode -`→1; `MR decode mr1qqqqqqqqqqqqqqqq5saqe93dj7z0t`→1; `… verify $BAD`→1; `… upgrade md1notacard`→1 |
| 1 | value `mr` refuses after the parse; missing mandatory input `mr` checks itself | all | `Failure::fail` sites | `MR create`→1 ("needs a template"); `MR … create --template-preset nosuch`→1; `MR units --template-preset nosuch`→1; `MR units notacard`→1; `MR … create "wsh(garbage"`→1; `… member $C --epoch 999999 …`→1; `… current $C --height 1 --time 1 --lead 100`→1; `… export $C`→1 ("give the tip … --epochs or --exhaustive"); `… verify $C --keys notakey`→1 |
| 1 | no input on stdin where asked | verify/member/address/create `--seeds -`; child | `mr-cli/src/common.rs:1040` | `MR … verify $C --seeds - </dev/null`→1 ("none were given"); `MR child </dev/null`→1 |
| 1 | I/O error | verify `--keys-file`, export `--heir-file`, vectors `--out`, any stdout write | `lib.rs:190-200`, `lib.rs:399-409`, `main.rs:29-33` | `… verify $C --keys-file /nonexistent/file`→1; `… export $C --epochs 0..0 --heir-file /nonexistent/dir/h.txt`→1; `MR vectors --out /proc/nonexistent/x`→1; `MR … decode $C >/dev/full`→1; `MR --help >/dev/full`→1; `MR --version >/dev/full`→1; `MR repair mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t >/dev/full`→1 (a 5 turned into 1) |
| 1 | safety refusal of secret material | child, root, create | `Failure::fail` | `printf 'A24\n\n' \| MR … child --letters B`→1 ("made from a published test vector's draws"); `printf '123\n\n' \| MR … root --letters B --from dice`→1 ("at least 92 rolls") |
| 2 | clap usage error (custom text, value never echoed) | all | `main.rs:37-49` | `MR`→2 (bare: prints help on stderr); `MR --bogus-flag`→2; `MR nosuchverb`→2; `MR repair`→2; `MR --network bogus decode …`→2; `MR … current $C --height abc …`→2; `MR … verify $C --digest zz`→2; `… --digest 0000`→2; `… --set-id <32 hex>`→2; `MR … root --letters Z --from coins`→2; `MR vectors`→2 |
| 2 | "closed list" post-parse usage errors (`Failure::usage`) | create, upgrade, verify, member, address, current, units, root, child, lost-or-stolen, procedure | `Failure::usage` sites: mr-cli `child.rs:90,132`, `root.rs:58,109`, `verify.rs:168`, `procedure.rs:32`, `units.rs:35`, `common.rs:923,1018`, `lost_or_stolen.rs:173-174` (used `:551,556,578,587`); mr-codec `app/{address,create,verify,keys,seat,member,common}.rs`, `app/create/{hotrun,chainrun,rootrun}.rs`, `app/verify/parentage.rs` (~47 sites) | `MR … create --template-preset owner-heir`→2 ("create needs the tip"); `MR units --shape 99 $C`→2; `MR procedure nosuchpage`→2; `MR … lost-or-stolen $C --height … --time …`→2; `… lost-or-stolen $C plate …`→2; `… lost-or-stolen notacard plate …`→2; `MR … root --letters none,B --from coins`→2; `MR … root --letters B --from coins </dev/null`→2 ("give exactly 237 flips … you gave 0") |
| 2 | secret or foreign string on argv (policy refusal filed as usage) | all card verbs, `--seeds`, `--passphrase` | `common.rs:923`, `:1018`, `child.rs:90`; card-argument check in `parse_cli` | `MR … verify $C --seeds "abandon … about"`→2; `MR … create --template-preset owner-heir --passphrase 1=secret`→2; `MR … child --passphrase x`→2; `MR … decode <mr1> <md1> <mr1> <mr1>`→2 ("a card argument is an md1 string (not echoed)"); `MR … upgrade mr1qqqq…`→2 |
| 2 | repair failed (uncorrectable, not mr1, framing) | repair | `repair.rs:24-27`, `:47-50` | `MR repair mr1qqpppppqqqqqqqqq5saqe93dj7z0t`→2; `MR repair notacard`→2; `MR repair ""`→2; `MR repair -`→2; `MR repair <chunks 0,1,2 of 4>`→2; `MR repair <uncorrectable> >/dev/full`→2 |
| 5 | repair: ≥1 substitution corrected (the checksum's code; the result may be no card) | repair | `repair.rs:64-66` | `MR repair $BAD`→5; `MR repair mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t`→5 (corrects to the U=0 string that is no card) |
| 101 | panic in the verb thread | all | `main.rs:63-64` | source only (no test pins it) |
| signal | no handler: default action | all | — | python `Popen` + SIGINT → returncode −2; SIGTERM → −15 (shell 130 / 143), on `MR --network testnet root --letters B --from coins` |

### A.2 `mr-sitting`

| code | meaning | verbs | decided at | measured |
|---|---|---|---|---|
| 0 | success / listing | all | `session.rs:321`; `verify.rs` `Ok((code, out))`; `sit.rs:187`; `main.rs:335`; `kit.rs:389,426,468` | `S --help`→0; `S --version`→0; `S help`→0; `S glossary`→0; `S glossary root`→0; `S practice`→0 (list). A completed `sit`/`verify`/`kit-*`: source + tests only |
| 1 | holder stopped / end of input / signal during a session | sit, practice, verify | `session.rs:322-323`, `:2168-2179`; `sit.rs:54-58`, `:289`, `:361`; `verify.rs:263,306`; `wipe.rs:870` (`_exit(1)`) | `S practice full-1 --dir …/pr3` with stdin closed→1; same after 2 Enters then EOF→1; + SIGTERM→1; + SIGINT (mid-session)→1 (all print ST-END-03) |
| 1 | mainnet on a networked computer not overridden (ST-NET-06) | sit, verify, root, child | `main.rs:306-309`; `sit.rs:360`, `:488` | `S root --network main --letters B --from coins </dev/null`→1 |
| 1 | verify mismatch / no candidate / wrong selector | verify | `verify.rs:269-287`, `:433` | source only |
| 1 | device step failed, read-back not blank, or signal before picotool (ST-KIT-07/11/12/14) | kit-write, kit-wipe, erase-sealed | `kit.rs:50-59`, `:307-313`, `:447-462` | source only (no device) |
| 1 | `mr root` / `mr child` refusals, unchanged | root, child | `mr_cli::finish` | `S child --network testnet --letters B </dev/null`→1; `printf 'y\nA24\n\n' \| S child --network testnet --letters B`→1 |
| 2 | clap usage error (clap's own text; may echo the value) | all | `main.rs:274-285` | `S`→2; `S --bogus-flag`→2; `S nosuchverb`→2; `S sit`→2; `S verify --plan … --key A --root 1`→2; `S verify … --key Z`→2; `S root --network bogus …`→2; `S practice --dir /nonexistent`→2; `S kit-write`→2 |
| 2 | input file unreadable or malformed (plan, hand-back, kit plan), hand-back not this plan's, output path exists | sit, verify, practice, kit-write | `sit.rs:15-25`, `:28-38`, `:319-347`, `:450-476`; `practice.rs:88-104,179,196,218,230,268`; `kit.rs:319-340` | `S sit --plan /nonexistent/plan --hand-back …`→2; `S sit --plan empty.plan --hand-back …`→2; `S verify --plan /nonexistent --hand-back /nonexistent`→2; `S kit-write --kit /nonexistent/kit`→2; `S kit-write --kit empty.plan`→2; `S practice full-1 --dir <dir already holding the files>`→2 |
| 2 | unknown name / ambiguous selector | glossary, practice, verify (ST-V-07, ST-V-08) | `main.rs:330-333`; `practice.rs:179`; `verify.rs:46-47`, `:295-301` | `S glossary nosuchterm`→2; `S practice nosuchcase`→2 |
| 2 | environment not usable: session lock held or untakeable, no RAM-only folder, picotool unusable, random bytes not drawn, device file not creatable, packer self-check | sit, kit-write, kit-wipe, erase-sealed | `sit.rs:234-271`; `kit.rs:280-304`, `:396-400`, `:411-413`, `:416-418` | `S kit-wipe` (no `XDG_RUNTIME_DIR`)→2; `S erase-sealed`→2; `XDG_RUNTIME_DIR=/home/user/wt/ec/tmp-mr S kit-wipe`→2 ("not on tmpfs or ramfs") |
| 2 | `mr root` usage, unchanged | root | `mr_cli::finish` | `S root --network testnet --letters B --from coins </dev/null`→2; with `printf 'y\n'`→2 (0 flips) |
| 2 | not Linux | all | `main.rs:9-13` | source only |
| 101 | worker panic | all | `main.rs:239-240` | source only |
| signal | `root`, `child`, and any verb before `install_signals`: default action | root, child | — | SIGINT→−2, SIGTERM→−15 on `S root --network testnet --letters B --from coins` |

## B. Class matrix

| # | class | `mr` | `mr-sitting` |
|---|---|---|---|
| 1 | success | 0 (repair clean 0) | 0 |
| 2 | answer is no | **1** (`verify` set-id/digest/seat mismatch, `child --check` no match); `Ok(1)` not `Err` | **1** (`verify` ST-V-04/05/06) |
| 3 | invalid input artifact | **1** in every verb; **2 in `repair`**; **2** when the string is another format's (md1/mk1/ms1/seed words) on a card argument | **2** (plan, hand-back, kit plan files); root/child as `mr` |
| 4 | usage error | **2** (clap + closed list) | **2** |
| 5 | no / empty input | missing positional: 2 (clap). Empty string / `-`: 1 (decode), 2 (repair). Empty stdin: **1** (`--seeds -`, `child`), **2** (`root --from coins`, flip count), 1 (`root --from dice`, roll count) | empty file: 2; end of input at a prompt: 1 |
| 6 | policy / safety refusal | secret on argv: **2**; published-vector root, guardrails, hot-cold rules, seating: **1** | mainnet-on-network: 1; brought-seed refusal: 1 |
| 7 | I/O error | **1** (`--keys-file`, `--heir-file`, `vectors --out`, stdout); closed pipe: verb's own code; stderr failure: ignored | input unreadable / output exists / lock / RAM folder: **2**; hand-back write failure mid-session: 1 (stop; source only) |
| 8 | repair | corrected **5** (unverified: may be no card); clean 0; uncorrectable / not mr1 / framing **2**; 5→1 on a write error. No exit 4 | n/a |
| 9 | external dependency | none (no node, no subprocess) | picotool unusable (`version` fails): **2**; picotool `load`/erase fails: **1** |
| 10 | partial / warnings-only | 0 (`verify: passed, but …`, "only filled", `warning:` lines, repair's "not a card" warning) | 0; a failed wipe prints ST-HARD-02 and keeps the stop's code (`wipe.rs:300-313`) |
| 11 | internal error / panic | **101** (worker join; fixed text, no payload) | **101**; a panic inside a session also runs the wipe hook (source only) |
| 12 | other | `--help`/`--version`/`help` 0 (1 if stdout unwritable); bare `mr` 2; signal: killed (130/143) | same for help/version/bare; session signal → **1** via `_exit(1)`; `root`/`child` signal: killed |

## C. Collisions and surprises

1. **2 means two things in `mr repair`**: usage error (`MR repair`→2, `MR repair --bogus`→2) and "could not be corrected / not an mr1 string" (`MR repair notacard`→2). A caller cannot tell a typo in the command from an uncorrectable plate. Inherited from `md repair` by spec §3 (`design/SPEC_refugium_mr1.md:12099-12101`).
2. **Invalid artifact is 1 everywhere but `repair`, where it is 2** (`MR … decode notacard`→1, `MR repair notacard`→2; bad checksum beyond t: decode 1, repair 2).
3. **A foreign-format string is 2, a garbage string is 1** on the same argument: `decode <md1 string>`→2 ("error: a card argument is an md1 string"), `decode notacard`→1. The uniformity spec's "wrong HRP" class therefore splits by whether the HRP is a sibling's.
4. **Exit 5 is "the checksum corrected", not "verified".** `MR repair mr1qqpqqqqqqqqqqqqq5saqe93dj7z0t`→5 and the result is no card (spec's own vector; CLAUDE.md hazard "Repair is not verification"). Against D26 as the brief states it (5 = verified now or verifiable by reassembly, 4 = substitution correction with no self-oracle), a single-string mr1 repair has no self-oracle yet exits 5; `mr` has no exit 4. (Inference: a chunked set does check the content's own chunk-set id in `chunked::check_set`, so the chunked case may count as "verifiable by reassembly"; the single-string case does not.)
5. **Repair exit 0 also covers "passes the checksum but is not a card" and "a lone chunk"** — warnings only (`MR repair <chunk 1>`→0), while `decode` of the same input is 1.
6. **Missing mandatory input: 1 or 2 by which input.** `MR create`→1 (no template); `MR … create --template-preset owner-heir`→2 (no tip); `MR … export $C`→1 (no tip/range); `member`/`address`/`current` without the tip→2 (clap). Deliberate (spec `:27529-27533`, TRACE `jf2_54_missing_inputs_exit_codes`), but not derivable from the class.
7. **Bad value: 1 or 2 by who checks it.** Parser-checked (`--network bogus`, `--height abc`, `--digest zz`) → 2; `mr`-checked (`--template-preset nosuch`, `--epoch 999999`, hot-family values, `--keys notakey`) → 1 (owner ruling "Keep 1", spec `:27539-27551`). But `units --shape 99`, `procedure nosuchpage`, `lost-or-stolen … plate` are `mr`-checked and 2.
8. **Empty stdin: 1 or 2.** `verify --seeds -`→1, `child`→1, `root --from dice`→1, `root --from coins`→2 (flip count is "usage", TRACE `:5728`).
9. **Secret on argv is a usage error (2), not a refusal** (`--seeds <words>`, `--passphrase 1=x`, `child --passphrase`). `me` uses 3 (`EXIT_REFUSED`) for policy refusals per the uniformity spec's table.
10. **I/O errors: `mr` 1, `mr-sitting` 2.** `MR … verify --keys-file /nonexistent`→1; `S sit --plan /nonexistent …`→2; `S kit-write --kit /nonexistent`→2. In `mr-sitting`, 2 is "anything wrong before a secret is read" (usage, files, lock, RAM folder, picotool missing) and 1 is "after" (stops, mismatches, device failures).
11. **External tool: 2 if it cannot start, 1 if its step fails** (`mr-sitting` picotool; `kit.rs:300-304` vs `:307-313`).
12. **The exit code changes printed output**: code 2 drops the usage-quiet trailer lines (`lib.rs:382`, `api.rs:222`); renumbering usage must keep that test keyed on the constant.
13. **`--help >/dev/full` exits 1**, and a 5 becomes 1 on a write error; closed pipe keeps the code; closed stdout (`>&-`) and a failing stderr are silent 0.
14. **Clap error text differs between the two binaries**: `mr` never echoes the value (custom renderer), `mr-sitting` uses clap's rendering (`error: invalid value 'Z' for '--key <L>'`). Same code (2).
15. **Signals**: a `mr-sitting sit`/`practice` session converts SIGINT/SIGTERM to exit 1 (same as a holder's stop); `mr`, and `mr-sitting root`/`child`, die by the signal.
16. Doc mismatch (small): `design/SPEC_mr_sitting.md:98-104` §3.2 says "one closed scheme for all **four** subcommands" and lists only sit/verify/root/child; the binary has nine verbs. The toolkit manual's table (`46-mr-sitting.md:839-845`) does cover practice, glossary and the kit verbs. Spec row 2 lists ST-START-08/09; the manual's row omits them. Neither table mentions 101 or the signal `_exit(1)`.
17. Doc mismatch: `/home/user/wt/cc/shared.md` CLI-14 and uniformity spec §6f have no `mr` row; measured for that table: usage 2, invalid artifact 1 (repair 2), repair applied 5, uncorrectable 2.
18. `mr --help` prints no exit-code table (only procedure prose such as "It must exit 0"); `mr repair --help` states no codes at all (unlike `md repair --help`).

## D. Documented codes

Normative / design (worktree):
- `design/SPEC_refugium_mr1.md:12099-12101` — repair "exits 0 when nothing was corrected, 5 when something was and 2 when it fails, as `md repair` does (1 when its output cannot be written …)".
- `design/SPEC_refugium_mr1.md:27508-27560` — §7 "Input and output" (E45): 0 / 2 usage (closed list) / 1 any other refusal; closed-pipe and write-error rules; foreign strings on a card argument exit 2. Errata summary at `:2310-2319`.
- `design/SPEC_refugium_mr1.md` — about 360 lines in all mention an exit code (per-refusal "exit 1" / "exit 2" notes); `design/SPEC_ERRATA_mr_v0_1.md` ~204; `design/FOLLOWUPS.md` ~225.
- `design/IMPLEMENTATION_PLAN_mr_v0_1.md:71` — decision D9 (repair 0/5/2; else md-cli's 0/2/1).
- `design/SPEC_mr_sitting.md:98-104` — §3.2 table (0/1/2); per-text codes in §15 (e.g. `:1214`, `:1236`, `:1361-1371`); ~144 lines.
- `design/SPEC_mr_codec_api.md:243` — `Outcome::code()` "is the exit code"; `:233-238`.
- `design/TRACE_mr_v0_1.md` — ~914 of 15,198 lines carry an exit code; anchors: `:480` `m11_repair_exit_codes`, `:3702` `jf2_54_missing_inputs_exit_codes`, `:5728` `mr root` "Exit 0 printed, 2 usage, 1 refused". `design/TRACE_mr_sitting.md` ~187 lines.
- Code comments: `crates/mr-cli/src/exit_codes.rs:1-8`, `crates/mr-cli/src/repair.rs:1-10`, `crates/mr-codec/src/repair.rs:24-43`.

Toolkit manual:
- `/home/user/wt/mnemonic-toolkit/docs/manual/src/40-cli-reference/45-mr.md:52-59` — "### Exit codes" table: 0, 1, 2 (incl. "`mr repair`: nothing could be corrected"), 5 ("`mr repair` only: the strings were corrected"). `:2103` "The exit code is the checksum's either way." 134 lines of the chapter contain "exit" (e.g. `:72`, `:97`, `:129`, `:1674`, `:1733`, `:1886`, `:2025`, `:3086`).
- `/home/user/wt/mnemonic-toolkit/docs/manual/src/40-cli-reference/46-mr-sitting.md:839-845` — "## Exit codes" table 0/1/2; also `:244`, `:309`, `:515`.

Printed by the binaries (a renumber changes printed text and its tests):
- `mr --help` long text: "must exit 0" / "exits 0" / "exits 1" / "exit 1" at help lines 308, 312, 361, 364, 457, 463, 536, 706, 711, 712, 904, 988 (only 0 and 1 are named).
- `mr verify --help` line 40: "`--passphrase` … Refused (exit 2)".
- Printed commands/advice in `crates/mr-codec/src/error.rs:774, 808, 876, 1081, 1143-1144`, `crates/mr-codec/src/chain_text.rs:225, 261, 2127`, `crates/mr-codec/src/kits/refuges.rs:695` ("exits 0 only when…", "refuses, exit 1", "must exit 0").
- `mr-sitting`: `crates/mr-sitting/src/verify.rs:297` ("the one that exits 0 is this plate's"); picotool's own code is quoted in text (`picotool.rs:45-53`, "picotool exit 249").
- Not documented anywhere: 101.

## E. Tests that pin exit codes

- `crates/mr-cli/tests/` (184 files, one binary `cli_tests`): 147 files assert a status. ~1,090 `exit(&o, N)` helper calls (`fj1_common/mod.rs:931`; twin `code(&o, N)` at `m11_common/mod.rs:91`), 42 `.exit(N)`, ~285 direct `status.code()` comparisons. By literal (approximate, regex count): **0 ≈ 2,500, 1 ≈ 1,700, 2 ≈ 230, 5 = 7, 101 = 0**. Named rows: `m11_repair_exit_codes`, `jf2_54_missing_inputs_exit_codes`, `jf6_r6_9_malformed_hot_family_values_exit_one`, `m12_fold_usage_error_closed_list` (scans source for `Failure::usage` sites), `pr41_verify_notes_slot_exit_1`.
- `crates/mr-cli/tests/api_parity.rs` (`:81` `assert_eq!(code, o.code(), "exit code")`): CLI code must equal `mr_codec::api` code for every parity case.
- `crates/mr-cli/src`: `exit_codes.rs:18-43` (`exit_codes_pinned`), 11 asserts on `exit_codes::` constants in `lib.rs`.
- `crates/mr-codec`: 46 corpus vector files carry `"exit": N` for repair (`tests/vectors/*.json`, written by `corpus.rs:362,372`, `corpus_m8.rs:892,902`); ~40 literal asserts in `tests/`.
- `crates/mr-sitting`: ~270 code assertions over 20 files (`tests/kit.rs` 37, `w1t_line.rs` 30, `sit_core.rs` 24, `verify.rs` 21, `start.rs` 19, …); `tests/parity.rs` + `tests/parity_refusals.list` (60 lines) require **equal exit codes between `mr root`/`mr child` and `mr-sitting root`/`child`** (CI job `.github/workflows/ci.yml:171-190`).
- No insta snapshot contains an exit code (0 of 47), but help-text snapshots contain the "exit 0/1" prose of §D.
- Renumber cost: changing 1 or 2 touches thousands of assertions across ~170 test files plus ~1,100 TRACE/spec lines and two manual chapters; changing 5 touches 7 CLI asserts, 46 vector files (regenerated by `mr vectors`), spec §3 and the manual.

## F. Consumers

- **refugium-wallet (mr-gui) spawns `mr-sitting`**: `/home/user/wt/refugium-wallet/crates/mr-gui/src/sitting_spawn.rs:17` — shell `SCRIPT` runs `mr-sitting "$@"; s=$?`, keeps the window open unless `s` is 0 and the verb is `sit`, then `exit "$s"`; `:160-164` `s.success()` → `Launch::Started`, else `EndedEarly("exit status <n>" | "signal <n>")` (`:104-112`). Only zero/non-zero is branched on; the number is shown.
- **refugium-wallet consumes the same numbers in-process** through `mr_codec::api` `Outcome::code()` (no spawn of `mr`): ~21 non-test sites, e.g. `crates/refugium-view/src/create/mod.rs:4031-4040` (`if r.code == 0 { 2 } else { r.code }`), `:4325`, `create/sitting.rs:1181` (`called.code == 1`), `:1321-1325`, `:1652`, `create/replay.rs:182`, `create/first_address.rs:218-248`, `balance/receive.rs:350`, `action/job.rs:184,284`, `restore/mod.rs:797-798`, `mr_panel.rs:261`. It branches on 0, 1 and 2 literally; ~20 more sites in its tests.
- **`mr-sitting` consumes `mr`'s codes in-process**: `root`/`child` return `mr_cli::finish`'s code (`crates/mr-sitting/src/main.rs:286-311`); the closing check records `pass` iff the parentage form's code is 0 (`design/SPEC_mr_sitting.md:506`; `crates/mr-sitting/src/closing.rs:287,314`).
- **Scripts in the worktree** read `$?`/`returncode` of `mr`/`mr-sitting` runs: `scripts/spec-gate-sitting-kit-day.sh:71`, `scripts/golden/diff-wrapper.py:126`, `scripts/create-key-kind-fix-gate.sh:141`, `scripts/regtest-spend/` (zero/non-zero).
- **Toolkit manual lint** runs `mr --help` via `MR_BIN` (`/home/user/wt/mnemonic-toolkit/docs/manual/tests/lint.sh:156-186`; CI `.github/workflows/ci.yml:379-442`): needs exit 0 from help only.
- **mnemonic-engrave** scripts name `mr` (`scripts/gen-sysw-mr-vectors.py`, `check-sysw-mr-vectors.sh`, `spec-s1-gate.sh`): not inspected line by line (other surveyors' repo); mnemonic-gui has no `mr` schema or spawn (grep of `src/` and `pinned-upstream.toml` found none).
- **Operators**: printed advice tells people to act on "exits 0" / "exits 1" (§D), so 0 and 1 are part of the human-facing contract.
