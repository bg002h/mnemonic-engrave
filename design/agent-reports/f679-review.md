# F-679 review — mnemonic-gui `f679-pins` @ `123f09a` (v0.62.0, untagged)

**Question:** can this be tagged v0.62.0?
**Answer: not yet.** Two Important findings. In each, the GUI builds a command that today's CLI refuses, on a path the form offers as its natural use:

1. `ms verify` with a phrase and an ms1 card: the root cause is in ms 0.19.0.
2. The new `md address --from-mk1` field: the root cause is in the GUI.

Everything else the brief asked about holds up:

- The shown and run commands agree.
- The separator gate works.
- The new fields work, apart from I2.
- The glibc-floor gate works.
- All six mutations turned tests red.

The glibc floor is proven only on a stand-in binary; the reason is given below.

## Method, and what was re-run

- Release CLIs were installed with `install.sh --no-gui --no-man` into a scratch root: mnemonic 0.104.0, md 0.20.3, ms 0.19.0, mk 0.13.0. `--version` was checked on each.
- Full suite: `cargo nextest run --locked --no-fail-fast` with all four `*_BIN` set gave **702 passed, 6 skipped**, which matches the implementer's report.
- Tutorial harness: `gui_tutorial_snapshots --include-ignored` with `GUI_TUTORIAL_SNAPSHOTS=1` and the release CLIs on PATH gave **12 passed**.
- A temporary probe test (untracked, deleted afterwards) drove the library's real assemblers against the release binaries:
  - `assemble_argv` is the Copy/Preview argv.
  - `assemble_argv_with_secret_mask` gives the Preview mask.
  - `assemble_argv_for_run` is the Run argv.
  - It covered **22 mnemonic/ms secret-bearing cases** and **12 new-field cases**.
- Clippy and MSRV were not re-run. They are settled by CI run 36100903163, whose clippy and msrv jobs passed at `123f09a`.

## Critical

None.

## Important

### I1 — `ms verify --phrase <P> <ms1>` cannot run from the GUI (ms 0.19.0 defect exposed by the pin)

The GUI runs `ms verify --allow-argv-secret --phrase '<12 words>' ms10entr…34v7f`, which is exactly what the form shows. The result is **exit 1**:

```
error: cannot read both ms1 and --phrase from stdin
```

The user supplied nothing on stdin.

**Cause.** ms's override (`argv_guard::substitute`) rewrites every admitted value to `-` and routes it through a side channel. `cmd/verify.rs:69-75` then applies its concurrent-stdin guard to the *rewritten* argv: `ms1_src.reads_stdin() && phrase == "-"`. So two admitted values always trip it. The inline `--phrase=<P>` spelling fails the same way.

**Reproduction** (release binary, no GUI needed):

```
ms verify --allow-argv-secret --phrase "$AB" ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f   # exit 1
ms verify --allow-argv-secret --in card.ms1 --phrase "$AB"                                       # exit 0, "OK: round-trip valid"
```

Other two-secret verbs were measured and work:

- `ms derive --phrase --passphrase`: exit 0
- `ms derive <ms1> --passphrase`: exit 0
- `ms combine <share> <share>` with real split shares: exit 0, recovers the phrase

**Why Important and not "upstream's problem":**

- The GUI's `ms verify` form offers the ms1 positional plus `--phrase` as its round-trip check. That worked at the old pin.
- After this release the check fails, with an error that is false from the user's point of view.
- No test covers it: the i4, f679 and tutorial suites never run `ms verify` with `--phrase`.

**Remedy:** a Rust-primary fix in ms, where the guard must not treat an admitted channel as stdin, with a test vector; then pin ms 0.19.1. The GUI-only alternative is a conditional that disables the positional when `--phrase` is set and points to `--in`, which does work.

### I2 — `md address <md1> --from-mk1 <mk1>` fails as the GUI builds it (new field, GUI defect)

md 0.20.3 declares `--from-mk1 <STRING>...`, a multi-value flag. The GUI emits every positional at the **end** of argv (`assemble_argv_with_secret_mask`, the positional block after the flag loop), so clap swallows the md1 policy phrase as another `--from-mk1` value.

The GUI builds and runs this:

```
md address --from-mk1 mk1qprsqhpq… --from-mk1 mk1qprsqhpp… md1yq802gggqpsqwgtua24e7ssf3
```

It gets **exit 1**:

```
md: seating refused: `md1yq802…` is an md1 policy-card string, not an mk1 key card …
put it back on address's positional instead
```

The user already put it on the positional.

This fails for **every** use of the `--from-mk1` text field together with the policy card, which is the only way the flag is meant to be used. A single `--from-mk1` row fails the same way.

Controls, all against the release md:

| Command | Exit | Output |
|---|---|---|
| `md address --from-mk1 A --from-mk1 B -- md1…` | 0 | `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` |
| `md address md1… --from-mk1 A B` | 0 | same address |
| GUI-built `--from-mk1-file mk.txt md1…` (with or without `--seat '@0=1c017'` or `--experimental`) | 0 | same address |

Of the mirrored surfaces, only `md address` has a `<…>...` value flag. I scanned `--help` of every subcommand of all four release CLIs; the other hit, `md descriptor`, is not mirrored.

**Remedy:** emit `--` before the positionals for `md address`, or emit its positionals first, and add a real-CLI cell.

## Minor

**M1 — ms `--in` versus positional/`--ms1` is not excluded in the GUI.**
- `ms_encode` is the only ms conditional. decode, verify, inspect, derive, repair and combine all have `conditional: None` (`src/schema/ms.rs` subcommand table).
- Filling both `--in` and the positional gives clap `error: the argument '--in <FILE>' cannot be used with '[MS1]'`, exit 64.
- It is a clear refusal and gives no wrong result.

**M2 — `md address` with `--from-mk1` and no policy phrase shows no Required marker.**
- In `md_address`, `!has_phrases_pos && !has_template && !has_mk1` marks nothing Required once mk1 is present.
- The CLI refuses clearly: "no md1 policy phrase is on the positional …".

**M3 (secret-handling class, per the 2026-08-27 ruling) — `restore --from ms1=…` / `phrase=…` runs silently with the opt-in.**
- The node-value classifier admits these, which is correct: they would otherwise be refused. Measured: exit 0, fingerprint 73c5da0a.
- But `should_confirm_run` does not see a Text `--from`, so there is **no confirm modal**, and the Preview shows the secret in cleartext.
- Before this change the CLI would have refused the run. Now it proceeds with no prompt.
- This is the implementer's filed `restore-from-secret-node-unmasked-and-persisted`, recorded here only because admission makes the path live. It does not gate.
- `verify-bundle --from` (also a Text flag) has the same shape.

## Nit

- **N1.** `--allow-argv-secret` is added when a secret field holds a private-channel value.
  - This applies to `@env:VAR` or `-` in a masked flag, slot or composite, because the mask ignores the value. The `restore --from` classifier does exempt these.
  - It is harmless: mnemonic's guard returns `Clean` and runs. Measured: `convert --from phrase=@env:REVIEW_SEED` gives exit 0 on both the Run and Copy argv.
- **N2.** A record discrepancy. The implementer says dropping the node-value classifier "failed 2" tests. Forcing `is_secret_node_value_token`'s predicate to `false` fails **1**: `restore_from_a_secret_node_is_admitted_but_a_private_channel_is_not` (11/12 pass in that binary). Their mutation shape may have differed.

## Shown versus run (brief item 1)

**The shown and run commands agree, apart from the documented flag.**
- The confirm modal and the spawn use the same `admit_argv_secret_for_run` output. `pending.argv` is rendered, then cloned into `spawn_and_capture`, so they cannot diverge.
- The post-run `argv:` line renders `result.argv` with the flag.
- Copy and Preview both use the unadmitted argv.
- In all 34 probe cases, the Run argv with the flag removed equals the Copy argv exactly (`run_eq_copy_plus_flag=true` every time).

**The flag is never missing where a CLI needs it.**
- *mnemonic.* The guard table (`argv_guard.rs` at v0.104.0) covers nine flags. Every one maps to a GUI mask source:
  - secret Text flags: `--passphrase`, `--bip38-passphrase`, `--decrypt-password`, `--phrase`, `--secret`, `--digits`, `--ms1`
  - the `--slot` subkey mask, from the same `SECRET_SLOT_SUBKEYS` const
  - the `--from` composite node mask, from the same `SECRET_NODE_TYPES_ARGV`
  - the Text `--from` token classifier
- All 22 cases gave exit 0, or a non-refusal usage error from my own incomplete fixture. Every Copy argv was refused with exit 2 as designed. Near-miss inputs (`' ms1='`, `MS1=`) behave the same with and without the flag.
- *ms* is shape-based: any ms1-, phrase- or hex-entropy-shaped token. Its GUI non-secret fields (`--in`/`--out` paths, `--account`, dropdowns and numbers) cannot normally carry such shapes.
- *md and mk* have no guard at their tags, and the flag is never added to them (`declared` check).

**Can the flag reach a command with no secret?** Only in N1's private-channel case, where it is harmless.

## Separator (item 2)

- `--help` does not short-circuit value validation, measured on each binary. mnemonic, md and ms reject hyphen, comma and bogus with exit 64 / 2 / 64. mk accepts space, hyphen and comma, and rejects bogus with 64.
- mnemonic, md and ms offer only `space`, across all 7 `--separator` flags. mk offers all three.
- Mutations, each red with the refusing binary named in the message, then restored:
  - mnemonic `+hyphen`: red
  - ms `+hyphen`: red
  - mk `+bogus`: red
- Unmutated: 1 passed.

## New fields against the release binaries (item 3)

All GUI-assembled and run through the Run path:

| Field | Result |
|---|---|
| md `encode --in --out --path` | exit 0; `--out` file **0600** |
| md `decode --in` | exit 0 |
| md `verify --template --path` | exit 0, `OK` |
| md `address --from-mk1-file` / `--seat` / `--experimental` | exit 0, correct address |
| md `address --from-mk1` (text) | **fails, see I2** |
| ms `derive --template bip48-p2tr` (positional, admitted) | exit 0, `m/48'/0'/0'/3'` |
| ms `derive --in --template bg002h-wsh` | exit 0 |
| ms `repair --in --out` | exit 0; file **0600** |
| ms `verify --in --phrase` | exit 0 |
| mnemonic `export-wallet --format bitcoin-core-addresses --count 3 --allow sigless-branch` | exit 0, with a "did not fire" note |

## Glibc floor (item 4) — what was and was not proven

- **CI gate logic is proven.** I ran the step's exact script under bash with `-eo pipefail`:
  - v0.61.0 x86_64 asset: GLIBC_2.39, **fails**
  - v0.61.0 aarch64 asset (cross): GLIBC_2.18, passes
  - host-built binary: GLIBC_2.44, **fails**
  - missing binary: **fails**
- **The old failure is real.** Setup: an Ubuntu 20.04 base rootfs (glibc 2.31) via unprivileged `unshare -r chroot`. Docker is permission-denied for this user, and `cross` is not installed.
  - The v0.61.0 x86_64 asset **does not start** there: `GLIBC_2.35 … GLIBC_2.32 not found`.
  - The host binary also fails there, which serves as a negative control.
- **The shipped v0.62.0 x86_64 asset itself is NOT proven to start.** `build.yml` uploads no artifacts (run 36100903163 has zero), nothing is tagged, and I could not run `cross` locally.
- **Stand-in proof.** `cargo zigbuild --release --locked --target x86_64-unknown-linux-gnu.2.17` of `123f09a` gives a binary whose floor is GLIBC_2.17. Its NEEDED libraries are libc, libm, libpthread, libdl and ld-linux.
  - It **starts** on glibc 2.31 (`mnemonic-gui 0.62.0`).
  - Under Xvfb in the chroot, it **runs the full GUI**: an X11 window, and egui_wgpu found Vulkan and GL llvmpipe adapters. It ran for 15 s and shut down cleanly on SIGTERM.
  - This shows the code and dependency set has no hidden newer-glibc need at runtime (including the dlopen paths). It does not measure the `cross` artifact.
- **Suggested follow-up (non-gating):** upload the linux-gnu build output as a workflow artifact, or add a job that runs `--version` in an old-glibc container. Then the next release can prove "starts" rather than infer it.

## Mutations (item 5) — each applied, confirmed with `git diff --stat`, run, and restored

| # | Mutation | Test | Result |
|---|---|---|---|
| 1–3 | separator lists (mnemonic +hyphen, ms +hyphen, mk +bogus) | `real_every_offered_separator_is_accepted` | red ×3; the unmutated run passes |
| 4 | `is_secret_node_value_token` → false | f679 binary | 1 failed / 11 passed |
| 5 | drop `if has_mk1 { --template Disabled }` in `md_address` | `conditional_visibility` | 1 failed / 94 passed (`cell_f679_md_address_from_mk1_and_template_exclude_each_other`) |
| 6 | `tut-j4-14-depth2-export.exit.txt` 0 → 2 | `gui_tutorial_snapshots` (real app + release mnemonic) | panicked `byte-diff vs committed`; the `.new` file was removed |

Final `git status --short` in `/scratch/code/shibboleth/gui-worktrees/f679`: clean at `123f09a`. The scratch tree `/scratch/code/shibboleth/review-f679-scratch/` was deleted with `find … -delete`.

**ready to tag: no.** Both Importants must be resolved first:
- **I1:** an ms 0.19.1 fix plus a pin, or a GUI conditional steering verify-with-phrase to `--in`.
- **I2:** `--` or positional-first emission for `md address`, plus a real-CLI cell.
