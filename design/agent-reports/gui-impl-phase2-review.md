# Whole-diff review: mnemonic-gui phase 2 (secret channels + five forms)

**Reviewer:** opus, independent context. **Target:** `gui-followups` at `ee26ef1`, `git diff origin/master..ee26ef1` (phase 1a `5956da2`, `e3a55a4` + phase 2 `d35033a..ee26ef1`). **Binaries:** installed with `install.sh --no-gui --no-man`; sha256 of all four == `measured_with.json` (mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0).

**Verdict: 0 Critical / 0 Important / 2 Minor / 3 Nit.** I found no way for the GUI as built to send a secret to the wrong flag, send wrong bytes, put a planner-routed secret on argv on Linux, or show one invocation and run another. Every leg below ran against the real window (kittest) and the release binaries, with an interceptor recording the child's argv, env, stdin and fds.

## Method (what was executed, not read)

All work was done in a scratch clone at `ee26ef1`, with three untracked test files added. The branch itself was not touched.

**Interceptor.** `mnemonic`, `ms`, `md` and `mk` were put on `PATH` as a Python wrapper. For each run it:
- records the child's argv, every `MNEMONIC_GUI_*` variable, the stdin bytes, the bytes of every inherited pipe fd, and the set of open fds;
- then re-creates stdin and the fds and `execv`s the real binary.

The GUI's `runner::run_plan` resolves `mnemonic` through `PATH`, so the recording is exactly what the GUI spawned. Two decoy variables were planted in the GUI's own environment before any run: `MNEMONIC_GUI_S0=DECOY` and `MNEMONIC_GUI_S7=DECOY`.

**Window driver.** For every `shapes.py` shape the design expects to run (23 shapes), plus variants:
- It built a `FormState` through the real widgets: secret rows, slot rows, composites, node Text, secret positionals.
- It opened `MnemonicGuiApp` in `egui_kittest`, read Preview and the binding lines, and clicked **Copy command (POSIX)** (the text is taken from `platform_output` `CopyText`, i.e. the real clipboard payload).
- It clicked **Run**, read the confirm dialog, then clicked the dialog's **Run** and read `last_run`.

**Oracle.** Each run was compared with an argv-exact invocation: the public args from the recorded run, plus `--allow-argv-secret`, plus the scenario's literal values in `shapes.py` spelling. The variants were:
- base values;
- a swap of two same-key sources, or a changed value;
- an edge-whitespace passphrase;
- every source typed as `@env:VAR` (`envall`);
- only the last source as `@env:VAR` (`envlast`);
- a passphrase beginning with `-`.

## Results per brief item

### 1. Rust planner vs the reference planner

**Differential fuzz.** 50,000 random source lists were run through both `plan.py` and `plan_sources` and compared on bindings (JSON), provenance, payload bytes, or the refusal code:
- Each list mixes 1–4 sources from one subcommand (70%) or any subcommand.
- It includes repeated keys, `@N.` slot prefixes, `ms combine` groups of 1–3, and an unmeasured key.
- Values include lookalikes, CR/LF endings, NUL, 4094/4095/5000-byte payloads, every Python `isspace` character at the edges, and `@env:` names that are reserved, invalid, unset, empty, `\n`-ended, `\r\n`-ended, `-` or a lookalike.
- Platforms: linux, macos, windows.

**0 mismatches** (20,000 + 30,000 cases). Twelve outcomes were exercised, including OK, `two-stdin`, `payload-too-large`, `value-starts-with-dash` and every C1 code. A deliberately corrupted expectation was caught, so the comparison is not vacuous.

**`is_clean` whitespace set.** Python `str.isspace` over all code points equals Rust `char::is_whitespace` ∪ U+001C..U+001F.

**GUI flag order, repeated secret fields, bundle slots.** These are covered through the window, under item 2.

### 2. The runner, by effect

**Base, swap and edge scenarios (69).** Every GUI run == argv-exact: exit code and whole stdout. slip39 split is random, so for it only the exit code is compared. Every swap or changed value changed the output. Shapes include:
- bundle with 2 slots + passphrase (`@0.phrase` + `@1.ms1`), and slot swap;
- slip39 combine, 2 shares + passphrase (repeated `--share` over EnvRef);
- seed-xor combine (repeated composite);
- import-wallet with two `--ms1` (swap → exit 4);
- silent-payment (`--secret-file /dev/fd/3` + stdin);
- ms derive (`--in /dev/fd/3` + stdin);
- ms combine (StdinMulti).

**`@env:VAR` scenarios (53).** All equal argv-exact.

**Dash scenarios (18).** All equal the correct baseline. `ms derive --passphrase "-x dash pw"` matches `--passphrase=-x dash pw` (fingerprint 34f38174). Only the naive `--passphrase -x…` spelling mis-parses in clap, and that is the oracle's fault, not the GUI's.

**Env scrub and fds.**
- On every run the child's `MNEMONIC_GUI_*` set equals exactly the planned names; both decoys were replaced or removed.
- No extra fds reached the child.

**Interim path.** `plan(…, "macos")` is resolved bytes + `--allow-argv-secret`, with the secret tokens masked. Run on this box through `run_plan`, it equals the Linux private plan on all 69 (the base and swap set) and on the dash set. `ms derive --passphrase -x…` refuses `value-starts-with-dash`, as designed (`argv_eq_exact: false`).

**The e2e harness catches a planted swap.** With the implementer's "swap two secrets" mutation applied (`resolved[(i+1)%n]`), the analyzer flagged 64 of 69 scenarios. The 5 unflagged are single-source shapes, where the mutation is a no-op. The source was restored and verified clean.

### 3. Shown vs run

On all 69 + 53 scenarios:
- Preview (`shlex.split`) == the recorded run argv.
- The confirm dialog shows "sends these secrets privately to …". Its argv tokens == the recorded argv, and its binding lines == Preview's.
- Each binding line matches a delivery: env name present, stdin non-empty, fd present.
- No secret value appears in any accessible label or value (Preview, binding lines, dialog, widgets).

**Copy.** The clipboard text was executed as a user would, in both **bash and zsh**:
- the `IFS= read -rs` lines, with the value fed on stdin;
- "type it, then Enter, then Ctrl-D" (value + `\n`);
- "paste each share";
- `<FILE>` written with value + `\n`;
- for `$VAR`, the `printf '%s\r\n' "$VAR" |` pipeline and the user's own `@env:VAR`.

It matched the GUI run (exit + stdout) on **122/122** scenarios in each shell. fish is covered by the implementer's T6 only; I did not re-run it.

### 4. Part B

**Hashlock phrase fidelity.** GUI stdin is exactly `phrase + "\r\n"`, and every digest equals an independent PBKDF2-HMAC-SHA256 (salt `ms-hashlock-v1`, 100000 iterations) → SHA-256 oracle, and argv-exact:
- `"  pad  "` → `a6431448…`; `"pad"` → `ce49c0c1…`; `"p a d "` and `"a=b -x"` also match;
- `--method sha256` matches the double-SHA-256 oracle;
- `"--kind"` as a phrase matches the oracle; the naive argv baseline mis-parses it.

**Refusals and CLI errors.** Run is disabled with the named refusal for:
- `pad\n`, `pad\r`, `pad\r\n` → `value-ends-in-newline`;
- `-` → `C1-dash`.

`\tpad`, `ünïcødé  `, a 300-character phrase and `p\na d` fail with the same CLI error as argv-exact (printable-ASCII / 100-character limits).

**`--kind`.**
- `(choose)`: Run disabled with the tooltip, and no `--kind` token.
- "all kinds — lookup only": no `--kind` on argv, and the banner is shown.
- `--json`: the notice is shown.

**Sources.**
- `--hex` → `--hex -`: the hash160 oracle matches.
- The plate → `-- -`: the ripemd160 oracle matches.
- phrase + hex → phrase only, hex suppressed.
- plate + phrase → plate only, phrase suppressed; the sha256 oracle of the preimage matches.

**B2–B4 xprv, and persistence.** The fields are `md shape-key --descriptor`, `md descriptor --key` and the `md decompose` positional. A pasted xprv is bulleted in the widget and shows as `••••` in Preview. After `eframe::App::on_exit` writes `state.json`, the file contains no xprv. It also contains no hashlock phrase, hex or plate, and no `restore --from ms1=` value.

**Separator deviation confirmed.** ms 0.19.1 refuses `--separator hyphen` and `comma` with exit 64.

### 5. CI gate T10

`os_gate.gate` returns **false** for each of these edits to `schema-mirror.yml`:
- the real-binary step removed;
- `MNEMONIC_BIN` removed;
- `--test secret_channels_t7` dropped;
- `continue-on-error: true` on the step;
- `if: ${{ false }}` on the step;
- `|| true` appended;
- the job moved to macOS;
- `if: false` on the job;
- triggers reduced to `workflow_dispatch` only, or to a schedule only.

In the clone with the step deleted, `cargo test --test secret_channels_t10` **fails**: "T10: `linux` is in private_channels_on but no workflow runs …". Restored, it passes 2/2.

### 6. Mutations

I re-ran the implementer's script with filters. The CONTROL survived, and each of the four below was **KILLED**, with the mutated line shown to run:

| mutation | failing tests |
|---|---|
| delivery: swap two secrets | t1_plan_property…, t3prime_every_admitted_plan…, t6_copy_disabled… |
| delivery: pipe write end left open | t7_each_channel_delivers… |
| B5: Run no longer blocked on (choose) | b5_kind_choose_blocks_run… |
| NI9: only LF refused (CR admitted) | t1_ni9_trailing_cr_lf_refused_on_every_path |

### 7. Phase 1a (`restore --from ms1=`)

All four sites were checked in the live window:
- the **widget** shows a 50-bullet mask;
- **Preview and the confirm body** show only `ms1=@env:MNEMONIC_GUI_S0`, and on the interim plan `<masked>`;
- **`should_confirm_run`** opens the dialog;
- **persistence** keeps the value out of `state.json`.

### Claims re-run

- The full nextest suite against the pinned binaries gives **768 passed, 6 skipped** (31.7 s), matching the report.
- CI run IDs were not re-checked; they are controller-verified per the brief.

## Findings

### Critical
None.

### Important
None.

### Minor

**M1. `ms hashlock`: Copy stays enabled while `--kind` is `(choose)`, and the copied command has no `--kind`.**
- Run is disabled for exactly this state, because omitting `--kind` "puts a sha256 record on stdout" (§B5, the F-553 pipe hazard). The copied command reproduces that hazard in the user's shell.
- Reproduction: phrase `pad`, `--kind` left at `(choose)`. Run is disabled with the tooltip. **Copy command (POSIX)** yields:
  ```
  # stdin (--hashlock-phrase): type it, then Enter, then Ctrl-D
  ms hashlock --hashlock-phrase-stdin
  ```
- §B5 binds only Run, so this is not a design violation. Suggested fix: gate Copy on `conditional::run_blocker` with the same tooltip.

**M2 (secret handling; does not gate, operator ruling 2026-08-27). A pasted xprv in B2–B4 goes to the clipboard, and onto argv, without a warning.**
- It is masked on screen and not persisted, as §B2–B4 asks. But Copy returns the full `md shape-key --descriptor 'wpkh([…]xprv9s21…/0/*)'`.
- The old "— reveals secret" button label was removed on the grounds that Copy never carries a secret. That holds for planner sources, not for these content-masked md fields.
- Run spawns md with the xprv on argv and no confirm dialog; md then refuses it (exit 2 for shape-key, exit 1 for descriptor/decompose).
- Reproduction: `--descriptor` = `wpkh([00000000/84h/0h/0h]xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu/0/*)` in `md shape-key`, then Copy or Run.
- Suggested follow-up: log it; restore the reveal label (or disable Copy) when `mask` has a bit set, and confirm before Run.

### Nit

**N1.** Binding labels keep the sub-subcommand word on nested subcommands, e.g. `combine --share ← env …`, `path-of-xpub --phrase ← …`, `# combine --share: IFS= read -rs …`. This mirrors `plan.describe`'s `split(' ', 2)[-1]`, so T8 parity holds, but it reads oddly in Preview, the dialog and Copy comments.

**N2.** Under a name filter, `scripts/secret-channels-mutations.py` prints `1/0 mutations killed …`: the denominator `len(rows) - 1` assumes the CONTROL row is in the run. This is display only; the verdicts are correct.

**N3 (upstream ms).** `ms hashlock --help` (0.19.1) still says `--separator` accepts `space|hyphen|comma`, but the binary refuses hyphen and comma with exit 64. The GUI's space-only choice is right; the ms help text has drifted.

## Settled points, not re-derived

- The design's lookalike, CR/LF, Linux-only private-channel and derived-data decisions.
- `--separator` space only.
- T9 asserting refusal for newline-terminated values.
- The secret-handling severity rule.

ready to merge: yes
