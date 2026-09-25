# F-687 fold 1 — response to `f687-review.md` (0C/0I/5M/4N)

Agent-written (Opus 5.5), 2026-09-25.

| repo | branch | fold commit | on top of |
|---|---|---|---|
| mnemonic-secret | `f687-passphrase-channels` | `79343e2` | `e534917` |
| mnemonic-toolkit | `f687-passphrase-channels` | `6034922e` | `9846a784` |

All 5 Minors are folded, and all 4 Nits. F1 (echo of flag-like values) is not folded: it is a secret-handling item, which is non-gating per the 2026-08-27 ruling, so it goes in as a follow-up for the controller to file. I did not change how an empty passphrase behaves; that is awaiting the operator's ruling. Nothing is pushed.

## Findings and responses

**M1 — a path that IS stdin now counts as a stdin reader (both repos).**
- `passphrase_input::path_is_stdin` answers true for `/dev/stdin`, `/dev/fd/0` and `/proc/self/fd/0` by name. On Unix it also answers true for any path with the same device and inode as fd 0. For example, `< seed.txt` together with `--secret-file seed.txt` is refused. A pipe fed from that same file is not refused, because in that case stdin is not the file.
- The toolkit applies it to:
  - `silent-payment --secret-file`
  - `bundle --descriptor-file` and `--import-json <path>`
  - `verify-bundle --bundle-json` and `--descriptor-file`
- ms applies it to `derive --in`.
- The check runs through each command's existing one-stdin guard, so it covers `--passphrase-stdin` as well as `-`.
- `--passphrase-candidates-file` needs no check: it is already exclusive with `--passphrase` through clap's group.
- New tests:
  - toolkit: `a_path_that_is_stdin_is_a_second_stdin_reader` (by name ×5 × both spellings, by inode, plus a control)
  - ms: `an_in_path_that_is_stdin_is_a_second_stdin_reader` (by name ×2 × both spellings, and by inode)

**M2 — ms: the guard and the resolver now use one predicate.** Both call `passphrase_input::is_channel_value` (exact `-`, or a value starting `@env:`). ms therefore treats ` -`, a tab before the dash, `- ` and `=-\n` as argv material. Without the override they are refused (exit 1); with it they are the literal. This matches the toolkit, which refuses them with exit 2. The `=`-joined admitted value is now taken untrimmed, exactly as clap sees it. Before, `--passphrase=pw ` was admitted as `pw`. Other secret flags keep their trimmed `-`. New test: `a_padded_dash_is_material_in_every_spelling`, plus vector rows.

**M3 — the vectors now pin those rules.**
- The vector file grows from 27 to 37 cases and stays byte-identical in both repos (`cmp` passes). New rows:
  - `@env:` holding `@env:X` → the literal (cc24affa)
  - a lone trailing CR, stdin and env → 4b53a850
  - a leading LF, stdin and env → 8fa10bb3
  - LF followed by CR, stdin and env → f934e53f
  - a padded dash: refused, or with the override the literal (e20c1882)
  - `--passphrase=- ` → 3ca82432
- ms had two copies of the newline strip. `parse::read_stdin_passphrase` now calls the same `strip_one_newline` that `@env:` uses, so the byte rule has one copy in each repo.
- Each mutation was applied once, asserted to have landed, then restored, and the whole package suite ran each time:

| mutation | result |
|---|---|
| G1 toolkit: `@env:` resolved twice | red (`every_vector_case_holds`) |
| G2 toolkit: a lone CR stripped | red |
| G1 ms: `@env:` resolved twice | red |
| G3 ms: a lone CR stripped | red |
| M1 toolkit / ms: `path_is_stdin` always false | red / red |
| M1b toolkit / ms: inode branch removed, names only | red / red (the by-inode case) |
| M2 ms: guard trims again | red (2 tests) |
| M4 toolkit: `--import-json -` not counted as a reader | red |

**M4 — `bundle --import-json -` is under the guard.** `bundle::run` calls `refuse_second_stdin` before any read, on the passphrase, `--import-json` (`-` or a path that is stdin) and `--descriptor-file`. It is covered by the M1 test.

**M5 — help and advisory text.**
- The `--passphrase` help now describes `-`, `@env:` and the literal on `bundle`, `verify-bundle`, the three `xpub-search` modes (dropping "(inline)") and `slip39 split` (dropping the stale "fires iff `Some(_)`… regardless of value" and an internal "(R0 C1 fold)" reference).
- I ran `--help` on all 12 toolkit subcommands and on `ms derive`. Every `--passphrase` line now states the `-` / `@env:` rule.
- In the manual, the `slip39 split` / `combine` and `xpub-search` flag rows and the `addresses` row that still said "(inline)" are updated.

**Nits (all folded):**
- **N1:**
  - A non-UTF-8 `@env:` value is now reported as "set but not valid UTF-8" rather than "not set". The toolkit gets this through a new `EnvVarMissingReason::NotUnicode`, which covers every `@env:` flag; ms gets it in its resolver.
  - A non-UTF-8 argument is now a usage error: exit 64, naming its position, never its bytes. Before, both mains panicked with exit 101 in `std::env::args()`.
  - Test in both repos: `non_utf8_input_is_refused_by_name_not_panicked`.
- **N2:** the slip39 one-stdin refusal names `--passphrase -` when that is what was typed. The `--passphrase-stdin` wording is byte-unchanged, so the existing tests pass. Test: `slip39_refusal_names_the_dash_spelling`.
- **N3:** the toolkit guard's refusal for a literal `--passphrase` now lists `--passphrase -   (stdin; or --passphrase-stdin, or --passphrase @env:VAR)`. The unit test asserts all three.
- **N4:** the manual's "How `--passphrase` is read" table now says:
  - a literal is refused (exit 2) without `--allow-argv-secret`;
  - `@env:` is resolved once, and an unset, non-UTF-8 or invalid name is an error;
  - only the exact `-` is stdin;
  - file paths that are stdin, and `--import-json -`, count toward one stdin.

**F1 (not folded, non-gating):** in both CLIs, a flag-like value given space-separated to `--passphrase` is echoed back in the error. This is pre-existing. Recommend filing a follow-up.

**Empty passphrase (review item 4):** unchanged, awaiting the operator's ruling. M1 no longer depends on it, since a path that is stdin is now refused outright.

## Gates

| gate | ms | toolkit |
|---|---|---|
| `cargo nextest run --locked --workspace` | 640/640 passed, 11 skipped | 4072/4072 passed, 20 skipped |
| clippy 0.1.85 (pinned bin first on PATH), `--workspace --all-targets -D warnings` | clean | clean |
| `cargo fmt --all -- --check` | clean | clean |

Toolkit only:
- Examples golden: regenerated with `EXAMPLES_BIN_DIR=target/debug`; `git diff` is empty.
- `docs/manual` `make audit`: OK, with MD/MS/MK from a scratch `install.sh --no-gui --no-man --root`. Results: lint OK, 62 transcripts pass, anchor-check matches baseline.
  - The first audit run failed markdownlint MD038 on my new `` ` -` `` code spans. I reworded that line and re-ran, and the second run is the one above.

The scratch directory `/scratch/code/shibboleth/f687-scratch` is deleted.
