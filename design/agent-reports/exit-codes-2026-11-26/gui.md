# Exit-code survey: GUI side (surveyor "gui")

Scope: `/home/user/wt/refugium-wallet` (one binary, `mr-gui`; `refugium-view`,
`refugium-node`, `mr-gui-ui` are libraries) and `/home/user/wt/mnemonic-gui`
(binaries `mnemonic-gui`, `gui_render`). **Source only**: neither GUI is built,
so every GUI-process code below is read from source. The CLI codes the GUIs
quote were spot-checked against `/home/user/wt/target/debug/{mnemonic,md,ms,mk}`
(marked "measured"; those are fix-PR heads, not the tags mnemonic-gui pins).
mnemonic-gui HEAD 96ea3047; mnemonic-refugium worktree HEAD 3cabbbf1;
refugium-wallet pins mr-codec at git rev 25051a06 (`Cargo.toml:52`).

## A. Exit-code inventory

### A1. `mr-gui` (refugium-wallet/crates/mr-gui) — hand-rolled argv, no clap

`main()` returns `ExitCode` from `args::main_with` (`src/main.rs:14`,
`src/args.rs:119`). No `process::exit` anywhere in the workspace's `src/`.

| code | meaning | mode | where decided |
|---|---|---|---|
| 0 | user closed the app (UI child reached first frame, then exited 0) | launcher | `launcher.rs:259` |
| 0 | `--version` printed; `--check-libs` window libs present; `--screenshot` wrote PNGs | tools | `args.rs:163`, `:138`, `:148` |
| 1 | window-system libraries missing (message shown) | launcher | `launcher.rs:225` |
| 1 | **crash after first frame** (child exited non-zero or panicked) *and* **start failure** (every renderer failed before first frame, incl. 20 s timeout, spawn error) | launcher | `launcher.rs:262` |
| 1 | `--check-libs`: no window can open; `--screenshot` failed | tools | `args.rs:140`, `:152` |
| 2 | unknown or invalid argument (incl. bad `--screenshot` SCALE, bad `--internal-ui-child` renderer). **`--help`/`-h` are not recognised, so they exit 2** | usage | `args.rs:165-171`, `:105` |
| 3 | internal UI child only (`--internal-ui-child`): backend selection, window creation or event loop failed (`START_FAILED`) | child | `child.rs:137,159,165,215` |
| 1 | internal message child (`--internal-message-child`): backend/window/event-loop failure | child | `message_child.rs:48,54,66` |
| 101 / SIGABRT | panic. Debug/test profile unwinds (101); **release has `panic = "abort"`** (`Cargo.toml:95`) so a release panic is SIGABRT (shell 134). `main` does not catch. A panic hook (`child.rs:24`) replaces the default: it sends a `Panic{version, file:line}` record to the launcher and never prints the payload | children | `child.rs:24-36` |
| signal | message child sets `PR_SET_PDEATHSIG=SIGTERM` (`message_child.rs:25`); a launcher killed by SIGTERM dies by signal, no handler | | |

The launcher never forwards a child's code: it reads only `success()`
(`launcher.rs:339,342`), so child 3 / 101 / SIGABRT all become launcher exit 1.

### A2. `mnemonic-gui` — clap derive, `fn main() -> eframe::Result<()>` (`src/main.rs:31`)

| code | meaning | where decided |
|---|---|---|
| 0 | window closed normally; `--help` / `--version` (clap) | `main.rs:60` returns `Ok` |
| 1 | `eframe::run_native` returned `Err` (no display, GL/wgpu init): Rust's `Termination` for `Err` (inferred, not run) | `main.rs:31,60` |
| 2 | usage error: `Cli::parse()` is stock clap, **not** remapped to 64 (inferred from `main.rs:32`; the only flag is `--debug`) | `main.rs:32` |
| 101 | panic (no `panic =` in `Cargo.toml`, no `catch_unwind`; several `.expect()` at startup, `app_window.rs:156,183,196,209`) | — |
| 130 | SIGINT/SIGTERM (Unix) or Ctrl-C (Windows) when clean shutdown did not finish in 3 s: `process::exit(130)`. If the clean close wins, exit is 0. 130 is used for SIGTERM too (shell convention would be 143) | `app_window.rs:193`, `:207` |

`gui_render` (dev/test binary, `src/bin/gui_render.rs:29-73`): 0 success, 1 any
failure including bad arguments (`ExitCode::FAILURE`); no code 2.

### A3. In-process "exit code": mr-codec `Outcome::code()`

refugium-wallet does **not** spawn `mr`. It links mr-codec and reads
`Outcome::code()`, documented as "The exit code `mr` exits with"
(`mnemonic-refugium/crates/mr-codec/src/api.rs:50-56`). This is the same number
space as `mr`'s process exit status, so an `mr` renumber is a library API
change for the GUI (see F1).

## B. Class matrix

| # | class | mr-gui | mnemonic-gui |
|---|---|---|---|
| 1 | success | 0 | 0 |
| 2 | answer is no | n/a (verdicts are on screen) | n/a |
| 3 | invalid input artifact | n/a at process level (shown in UI; mr-codec code 1) | n/a (child's code shown) |
| 4 | usage error | **2** (hand-rolled; `--help` also 2) | **2** (clap default) |
| 5 | no/empty input | n/a | n/a |
| 6 | policy refusal | n/a | n/a (run refused in UI, no exit) |
| 7 | I/O error | 1 (`--screenshot` write failure) | state-file errors are non-fatal |
| 8 | repair | n/a ("`mr repair`: not part of this version", `refugium-view/src/mr_panel.rs:210`) | n/a (renders child's 4/5) |
| 9 | external dependency failure | 1 (missing libs, child failed to start); helper/terminal failures are non-fatal | 1 (eframe init); CLI spawn failures shown in UI |
| 10 | partial/warnings | none | none |
| 11 | panic | launcher exits 1 after the crash message; the panicking child itself 101 (debug) / SIGABRT (release) | 101 |
| 12 | other | `--version` 0, `--check-libs` 0/1 | `--help`/`--version` 0; signal fallback 130 |

## C. Collisions and surprises

1. **mnemonic-gui paints `md`'s "answer is no" as a green success.**
   `render_exit_badge` (`mnemonic-gui/src/app_window.rs:1316-1356`) keys on the
   bare number for every tab: 4 = amber "VERIFY-ME", 5 = green
   "exit: 5 — Repair Applied (BCH auto-fire succeeded; corrected chunk on
   stdout)". But the GUI surfaces `md descriptor --verify-against`
   (`src/schema/md.rs:1046-1051`, subcommand at `:1232`), whose own help says
   "Exit 0 = spend-equal, 5 = NOT spend-equal" (measured in `md descriptor
   --help`: "5 = NOT spend-equal (`md repair`'s reserved-5 precedent...)").
   So a not-spend-equal verdict is shown with the green repair-succeeded badge
   and wrong text. Inferred from source; not run in the GUI.
2. **4 is treated as "has usable output" everywhere in mnemonic-gui's tests and
   measurements** (`Some(0) | Some(4)`), and the badge text claims 4 is "the
   toolkit's project-wide VERIFY-ME class". Any CLI that uses 4 for something
   else on a GUI-surfaced verb gets the amber VERIFY-ME text.
3. **mr-gui exit 1 means three different things**: crash, start failure, and
   missing libraries (plus `--check-libs` "no window can open"). A script
   cannot tell a crash from a failed start. `SPEC_framework_slint.md:371-372`
   only promises "the exit status is non-zero".
4. **mr-gui has no `--help`**: it is an unknown argument, exit 2, with the
   usage line on stderr.
5. **Usage code 2 in both GUIs**, while three of the four CLIs mnemonic-gui
   drives use 64 (measured: `mnemonic --bogus-flag` 64, `ms` 64, `mk` 64,
   `md` 2). mnemonic-gui is not in `cc/shared.md` CLI-14's list; it is a
   2-binary.
6. **mnemonic-gui 130 for SIGTERM** (`app_window.rs:163-193`), and only when
   the 3 s grace expires; otherwise a signalled close exits 0.
7. **mr-gui treats mr-codec code 1 as the expected result of one call.**
   Create step 3's "path call" expects code 1 plus a path line
   (`refugium-view/src/create/mod.rs:4031-4032`, `create/sitting.rs:1181`,
   `:1321`; spec `SPEC_ui_refugium_wallet.md:1005` "expected code 1"). A code 0
   there is rewritten to 2 and shown as the usage/bug text
   (`create/mod.rs:4039`, `:4407`, `sitting.rs:1324`).
8. **mr-gui folds every code other than 0 and 1 into "usage"**
   (`mr_panel.rs:240-246`), whose text is "Refugium Wallet built a request that
   mr refuses as a usage error. This is a bug in Refugium Wallet; nothing was
   saved." (`mr_panel.rs:23`). A new mr code (say 4 or 5) would be shown as a
   GUI bug.
9. Stale comment, harmless: `mnemonic-gui/src/schema/mnemonic.rs:2358` says the
   `--digits`/`--from` conflict is "exit 64"; measured today
   `mnemonic convert --digits 1234 --from seedqr=-` exits **2** (the argv-secret
   refusal fires first).

## D. Documented codes

### refugium-wallet
- `README.md:47-50`: crash or failed start "reported through a short message
  window or stderr"; no code stated. No `--help` text exists.
- `design/SPEC_framework_slint.md:371-372`: with no helper "the message is on
  stderr and the exit status is non-zero".
- `crates/mr-gui/src/args.rs:135-136`: `--check-libs` "Non-zero when no window
  can open, so a script ... can test it".
- `design/SPEC_ui_refugium_wallet.md:1432-1442` (§4.1.1): the normative mapping
  of `outcome.code()`: 0 success, 1 refusal, 2 usage, "any other code: as 2".
  Further uses: `:862`, `:965`, `:1005-1010`, `:1030`, `:4990`, `:5035-5041`,
  `:5193`, `:5678-5682` (restore table: verify code x export code), `:6102-6105`.
  It also quotes `mr` CLI exits: `:4552`, `:4555`, `:4701`, `:4710`, `:4779`,
  `:4781`, `:5213` (all "exit 1"/"exit 0").
- `design/SPEC_w14s4_root.md:271` ("`mr root` maps to a usage refusal, exit 2");
  `:670-884` a list of ~27 `mr root` examples each ending `exit 0|1|2`.
- `design/SPEC_w2_mr_gui_sitting.md:255-262` (§W2-6): "The shell exits with
  `mr-sitting`'s status"; a non-zero end within 2 s shows W2-SI-03 with
  `exit status <n>` / `signal <n>`. Row `w2_spawn_early_exit` at `:820`.

### mnemonic-gui: user-visible text (schema help strings, shown as tooltips)
- `src/schema/mnemonic.rs:819` `--cosigner`: "A mismatch is a hard error (exit 4) unless --allow-mismatch"
- `src/schema/mnemonic.rs:2808` `--bsms-...token`: "MAC verify failure → exit 2 (typed BsmsMacMismatch)"
- `src/schema/mnemonic.rs:2831` `--bsms-verify-strict`: "fatal (exit 2)"
- `src/schema/ms.rs:649` `--out`: "REFUSES an existing file (ms 0.20.4: exit 64)"
- `src/schema/md.rs:1051` `--verify-against`: "Exit 0 = spend-equal, 5 = NOT spend-equal"
- `src/app_window.rs:1330-1333` badge: "exit: 4 — VERIFY-ME (confirm the result out-of-band before trusting; see stdout/stderr below)"
- `src/app_window.rs:1343-1346` badge: "exit: 5 — Repair Applied (BCH auto-fire succeeded; corrected chunk on stdout)"
- `src/app_window.rs:1349-1353`: `exit: <n>` / `exit: (killed)`
- `src/form/tree_form.rs:195`, `:325`: "... (exit N) ..." fallbacks; `src/cli_version.rs:124`.

### mnemonic-gui: code comments that quote a CLI code (no runtime effect)
- exit 64: `schema/mnemonic.rs:2358` (stale, see C9), `:2424` (bad feerate; measured 64),
  `schema/ms.rs:746` (separator), `form/conditional.rs:1067` (ms `--in` conflict),
  `:1239-1240` (ms hashlock; measured 64 for `ms hashlock --random`),
  `form/invocation.rs:22` (unrecognized subcommand; measured `mnemonic seed-xor-combine` → 64)
- exit 2: `schema/mnemonic.rs:149` (restore `--md1` template refusal), `:326`
  (argv-secret refusal; measured 2), `form/conditional.rs:1164,1167` (md
  descriptor; measured `md descriptor --template ...` with no key → 2)
- exit 1: `form/conditional.rs:1125`, `secrets.rs:196`
- exit 4: `schema/mnemonic.rs:2607` (`--max-subst` VERIFY-ME)
- exit 5: `app_window.rs:1299-1301` (`ToolkitError::RepairShortCircuit { exit_code: 5 }`)

### Toolkit GUI manual (`mnemonic-toolkit/docs/manual-gui/src/`)
224 "exit N" mentions in 65 files: exit 0 x39, 1 x56, 2 x68, 3 x7, 4 x26,
5 x5, 64 x32 (grep `exit:? ?[0-9]+`). Examples: `40-mnemonic/4d-restore.md:430,439,636-638`
(exit 4 mismatch), `4f-ms-shares-split.md:94,100` (exit 64),
`60-ms/6a-repair.md:66`, `70-mk/79-repair.md:76-82`, `4i-repair.md:149`,
`4g-xpub-search-address-of-xpub.md:185` (exit 5). This manual keeps anchor
parity with mnemonic-gui, so a renumber touches it too.

## E. Tests that pin exit codes

### refugium-wallet
- mr-gui process codes: `crates/mr-gui/tests/launcher.rs:180,353,365,394`
  (`Some(1)`, 4 asserts) plus ~13 more success/failure checks in that file;
  `launcher_windows.rs` (6), `xvfb.rs` (4), `version.rs` (2), `screenshots.rs` (1).
  **No process-level test pins usage = 2 or child = 3** (only the unit test
  `args.rs:234-237` on `Mode::Usage`).
- `tests/monkey.rs:1803-1861`: `Some(101)` x5 is the monkey test harness's own
  panic code, not mr-gui's.
- mr-codec `code()` values (0/1/2) asserted in refugium-view tests: roughly
  170 matching lines over ~20 files (`create_menu.rs` 29, `screens_plan.rs` 18,
  `create_round.rs` 16, `w2_sitting.rs` 16, `mr_api_smoke.rs` 13, `send.rs` 12,
  `grid.rs` 11, `create_6_9.rs` 10, `receive.rs` 9, `mr_panel.rs` 4, ...;
  loose grep, approximate).
- `tests/w2_cross_repo_sitting.rs:357-358` runs a real `mr-sitting` and asserts
  `status.success()`.

### mnemonic-gui
- `tests/tutorial/manifest.rs`: `expect_exit` per tutorial step, checked at
  `tests/gui_tutorial_snapshots.rs:650`: `Some(0)` x7, **`Some(2)` x1**
  (`manifest.rs:104`, a `mnemonic` refusal step), `None` x1.
- `tests/secret_channels_t2.rs:107,325`, `tests/secret_channels_t3prime.rs:45`:
  `matches!(code, Some(0) | Some(4))` = "produced output".
- `design/measurements/secret-channels/*.json`: recorded exits 0 x408, 1 x96,
  2 x85, 4 x42; the scripts `run_bytes.py:84-185` and
  `measure_reinterpret.py:40,50` use `in (0, 4)`. Tests compare against these
  tables, so a renumber means re-measuring.
- `tests/f679_argv_secret_admission.rs`: 8 exit_code checks (mostly `Some(0)`
  vs non-zero; comments name exit 1 and 64).
- `src/runner.rs:388-452` unit tests (0, 3 via `sh -c "exit 3"`).
- About 20 other test files only check `status.success()` on `gui-schema` /
  `--spec-schema` / `--version`.
- No test covers `render_exit_badge` (no hit for it or its strings in `tests/`).

## F. Consumers

### F1. refugium-wallet → mr-codec `Outcome::code()` (in-process; same numbers as `mr`)
- Mapping: `crates/refugium-view/src/mr_panel.rs:240-246` (0 Success, 1 Refused,
  else Usage); used by `Shown::of/new` `:260-272`, `:300`.
- `code == 0` plus a value required to proceed: `balance/receive.rs:350-359`,
  `send/build.rs:348-353`, `action/job.rs:184-191`, `:284-291`,
  `create/first_address.rs:218-222`, `:246-251`, `create/mod.rs:4325-4326`,
  `create/sitting.rs:1652`, `create/replay.rs:182`, `restore/check.rs:187`,
  `restore/mod.rs:797-812`.
- `code == 1` required (path call): `create/mod.rs:4031-4032`,
  `create/sitting.rs:1181`, `:1321`.
- 0 rewritten to 2 when the expected shape is absent: `create/mod.rs:4039`,
  `:4407`, `create/sitting.rs:1324`.
- Renumber impact: changing mr's refusal code away from 1 breaks the path call
  (step 3 shows the bug text, the sitting plan cannot be built) and turns every
  refusal into "This is a bug in Refugium Wallet"; changing usage from 2 to 64
  is absorbed by the catch-all. The pin is by git rev, so nothing breaks until
  the pin moves.

### F2. refugium-wallet → child processes
| child | where | how the status is read |
|---|---|---|
| own exe as UI child / message child | `mr-gui/src/launcher.rs:267-320`, `:582-594` | `success()` only (`:339,342`), combined with the first-frame pipe record; exit before first frame is a failure "whatever its status" (`:406`) |
| `zenity`, `kdialog`, `xmessage`, `notify-send` | `launcher.rs:598-635` | `status().is_ok_and(success)`; failure falls through to the next helper |
| `osascript` (macOS) | `launcher.rs:555-568` | status ignored |
| `x-terminal-emulator -e /bin/sh -c SCRIPT ... mr-sitting sit|verify` | `mr-gui/src/sitting_spawn.rs:17,127-173` | watched 2 s: exit 0 or still running = Started; any non-zero = `EndedEarly("exit status N" | "signal N")`, shown verbatim in W2-SI-03 (`refugium-view/src/create/texts.rs:913`). After 2 s the status is reaped and discarded (`:169-171`) |
| `mr-sitting` inside that shell script | `sitting_spawn.rs:17` | `s=$?`; closes the terminal without a prompt only when `s -eq 0` and the verb is `sit`; otherwise "Press Enter to close this window."; the shell `exit "$s"`. So only zero/non-zero matters; the number is shown to the user but not interpreted. Results come back through hand-back files, not the exit code |
| `git` | `mr-gui/build.rs:27-34` | build time, `success()` |
| `bitcoind` / `bitcoin-cli` | **tests only** (`refugium-node/tests/regtest_node.rs:84,117-125`, `refugium-view/tests/regtest_support/mod.rs`) | `success()`. The app talks to Core over HTTP JSON-RPC (`refugium-node/src/http.rs`), no subprocess |
- Printed advice that chains on a CLI's status: `refugium-view/src/create/mod.rs:805`
  prints `[ ... -eq 24 ] && mnemonic convert ... | awk '... exit 1 ...'`. The
  pipeline's status is awk's, so `mnemonic`'s own exit code is not consumed.
- No refugium-wallet source spawns `mr`, `md`, `mk`, `ms`, `mnemonic` or `me`.

### F3. mnemonic-gui → `mnemonic` / `md` / `ms` / `mk`
- Spawn: `src/runner.rs:189-229` (`run`) and `:282-356` (`run_plan`);
  `exit_code = output.status.code()` (`:222`, `:345`). Non-zero is **not** an
  error: `Ok(RunResult{exit_code: Some(n)})`; `None` = killed. It sets
  `MNEMONIC_FORCE_TTY=1` so toolkit auto-repair (exit 5) fires under a pipe
  (`runner.rs:106-125`).
- **Specific codes interpreted at runtime: only 4 and 5**, in
  `render_exit_badge` (`src/app_window.rs:1316-1356`, called at `:461`), for
  all four tabs. 0 and every other code print as `exit: N`. Nothing keys on 64,
  2 or 1; stdout/stderr are always shown.
- `src/cli_version.rs:121-127`: `<bin> --version` must exit exactly 0 before
  any secret-bearing run; any other code refuses the run.
- `src/schema_check.rs:367-368`, `:396-397`: `<bin> gui-schema` must
  `success()`, else fall back / `None` (test/build-tree path).
- `src/form/tree_form.rs:173-203`, `:236`, `:290-330`: `--emit-spec` and
  validate results are deliberately "Parse-keyed, not exit-code-keyed"; the
  code only appears in a fallback message.
- CI: `.github/workflows/schema-mirror.yml` installs the pinned tags and runs
  the tests in E.

### F4. What breaks in mnemonic-gui if a CLI renumbers
- **4 or 5 moved or reused**: the badge mislabels (already true for md's 5,
  C1). Moving VERIFY-ME off 4 loses the amber warning silently and makes
  `secret_channels_t2/t3prime` and the measurement scripts treat those runs as
  "no output" (tests fail until re-measured).
- **64 → 2 (or 2 → 64) for usage**: no runtime effect. One user-visible
  tooltip goes stale (`schema/ms.rs:649`), ~7 comments, `tutorial/manifest.rs:104`
  if that step's code moves, the recorded `"exit": 2` rows (85) in the
  measurement JSON, and 32 "exit 64" / 68 "exit 2" lines in the toolkit GUI
  manual.
- **`--version` or `gui-schema` non-zero**: secret-bearing runs are refused /
  schema tests fail.
- Because the GUI runs whatever binary is first on `$PATH` (minimum version
  only, `cli_version.rs`), a renumber reaches users before any pin bump; the
  badge and tooltips cannot be version-conditional today.
