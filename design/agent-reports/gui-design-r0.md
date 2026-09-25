# R0 review — mnemonic-gui `DESIGN_secret_channels_and_new_forms.md`

**Reviewer:** opus (independent context), 2026-09-25
**Artifact:** `gui-followups` @ `ec31aed`, `design/DESIGN_secret_channels_and_new_forms.md` + `design/measurements/secret-channels/`
**Binaries:** fresh `install.sh --no-gui --no-man` into scratch → mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0 (versions printed)
**Verdict:** **NOT GREEN — 1 Critical, 3 Important, 7 Minor, 2 Nit**

## What was machine-checked (so the findings below are not about these)

- **Harness re-run.** I copied it to scratch and ran `run_all.py`, `run2.py` and `table.py` against the fresh binaries. `channels.txt` and `channels2.txt` matched the committed files byte for byte (after stripping temp paths). The generated `table.md` is byte-identical to the design's A2 table (`diff` on lines 29–102). The table is honest to its harness.
- **Hand re-measurement, independent of the harness.** I wrote my own bash script and compared hashes of stdout (which carry addresses, master fingerprints and digests) against the argv baseline, and for each WRONG cell against the argv run with the literal value:

| row | channel | result |
|---|---|---|
| addresses `--passphrase` | `-` (stdin=PW) | = argv literal `-` (`fc83…`), ≠ PW (`a553…`) → **WRONG confirmed** |
| addresses `--passphrase` | `@env:V`, `--passphrase-stdin` | = PW → OK |
| restore `--from ms1=` | `-`, `@env:` | = argv (`c900…`) → OK |
| silent-payment `--passphrase` | `@env:V` | = argv literal `@env:V` (`c76d…`) → **WRONG confirmed** |
| silent-payment | `--passphrase-stdin`; `--secret-file F` + stdin | = argv → OK |
| silent-payment | `--secret-file /dev/fd/3` (pipe) + stdin | = argv → OK. **Not in the design's measurements**; it is true on Linux |
| ms derive `--passphrase` | `-` / `@env:V` | = argv literal `-` / literal `@env:V` → **both WRONG confirmed** |
| ms derive | `--in /dev/fd/3` + `--passphrase-stdin` | = argv → OK |
| bundle `--passphrase` | `@env:V` OK; `-` = argv literal `-` | confirmed |
| ms hashlock `--hashlock-phrase-stdin` | `"  pad  "` with no newline, and with `\r\n` | = argv `"  pad  "` (`b9c7…`), ≠ `"pad"` (`c6b9…`) → byte-verbatim; one `\r?\n` stripped |
| ms hashlock `--hex -` | stdin | = argv → OK |
| import-wallet `--ms1 -` | stdin | fails closed (ms1 length-1 decode error) → confirmed |
| verify-bundle (design: unmeasured) | `--slot @0.phrase=@env:` + `--passphrase-stdin` + `--ms1 @env:` | = argv baseline, `result: ok` (see Q4) |

- **Coexistence guard.** A user-typed `<node>=-` plus a planner `--X-stdin` is refused by the CLI on restore, addresses, convert, derive-child, slip39 combine, ms derive and ms verify. On silent-payment and xpub-search, the literal `-` is refused as argv material. So stray `-` tokens fail closed; the `@env:` spelling does not (C1).
- **Swap invariance, measured.** Swapping the two env-bound shares gives identical output for `seed-xor combine` (`e268…` = `e268…`), `slip39 combine` (`1a3b…` = `1a3b…`) and `ms combine -` (`f032…` = `f032…`). `ms combine --in /dev/fd/3` gives the argv result.
- **Part B.** I checked every B1–B5 table against the release `gui-schema` and `--help`, and ran 8 conditional rules: unspendable+wsh → 1, unspendable+sh-wsh → 1, path+preset → 2, wrapper `TR` → 1, template without key → 2, `--emit md1` in mode B → 2, chain+change → 2, decompose positional+`--in` → 2. **All match the design.** The only schema/table difference is the `md decompose` positional: `repeating: true` in the schema, but it is runtime-refused above one, which the design correctly models as "exactly one".

---

## Critical

### C1. User-typed `-` / `@env:…` in a secret field is unspecified, and either reading gives a wrong wallet with exit 0

The design owns stdin and the environment for secret sources, and it deletes the argv path. It says nothing about a secret field whose *value* the user typed as a channel spelling. That input is not hypothetical:

- **The GUI invites it.** `src/schema/mnemonic.rs:757` (RESTORE_FLAGS) and `:4103` (ADDRESSES_FLAGS) both render `--passphrase` help as "BIP-39 passphrase (seed sources). @env:VAR supported." `:600` and `:4033` say "@env:VAR / - (stdin) for secret values."
- **The GUI tests it.** `tests/kittest_import_wallet_form.rs` Cell 8 pins `@env:MNEMONIC_MS1_0` typed into a *secret* row flowing verbatim to argv. Its comment says any env-channel change "must explicitly opt into a behavior break here". The design does not mention it.
- **The GUI code expects it.** `invocation::is_private_channel_value` and `masked_token_is_private_channel` exist to recognise exactly these values in masked secret tokens.

Two implementations are natural, and both are wrong:

1. **"The value is the secret bytes."** The planner sends the literal `@env:MY_PW` over its channel. Measured on `restore --from phrase=<abandon…about> --template bip84`, intended passphrase `hunter2`:
   - argv `hunter2`, and today's pass-through `--passphrase @env:MY_PW`, both give **master fingerprint `ca2c62d2`**;
   - the design's channel with those bytes, `MNEMONIC_GUI_S1=@env:MY_PW` + `--passphrase @env:MNEMONIC_GUI_S1` (or `--passphrase-stdin`), gives **`fee4dc32`, exit 0**.

   The toolkit resolves `@env:` only once. This is a wrong wallet on a path that works today and that the GUI's help text teaches.
2. **"A sentinel passes through to argv, as today."** This keeps every WRONG cell the design exists to avoid, now reachable by typing:
   - `--passphrase -` / `--bip38-passphrase -` is the literal `-` on every mnemonic subcommand measured (the F-687 class);
   - `--passphrase @env:X` is literal on `silent-payment` and `ms derive`.

   It also opens a namespace collision. Copy (A7) prints the planner's variable names (`MNEMONIC_GUI_S1`), and a user-typed `@env:MNEMONIC_GUI_S0` in another pass-through field then receives *another source's* secret: a secret reaching a flag the user did not fill. The env scrub (A4) removes only *inherited* variables, not ones the planner sets.

**Remedy (the design must pick one and test it):**
- In every secret source (schema-secret flag, secret slot subkey, secret positional, classifier-matched Text), a value that is exactly `-` or starts with `@env:` is a `ChannelRefusal`, with the text "type the value itself; the GUI delivers it privately". Or `@env:NAME` is resolved GUI-side from the GUI's own environment and the *bytes* go through the planned channel, and `-` is refused.
- Refuse any user-typed `@env:MNEMONIC_GUI_*` in pass-through Text fields.
- Update the four help strings, retire or re-pin Cell 8 explicitly, and add a T1 leg covering both spellings in every secret source.

---

## Important

### I1. The A4 assignment rule does not generate the A5 plans, so the implementer has to guess

A4 is normative ("a source whose table entry has `EnvRef` gets a unique `MNEMONIC_GUI_S<n>`"). Applied to A2's OK cells, it gives these rows different plans from A5:

| A5 row | A5 says | the A4 rule gives (the cell is OK in A2) |
|---|---|---|
| addresses / restore / derive-child, `--from` + `--passphrase` | passphrase `--passphrase-stdin` | passphrase `@env:` |
| bundle / import-wallet, N slots + `--passphrase` | passphrase `--passphrase-stdin` | `@env:` |
| convert, node + `--passphrase` / `--bip38-passphrase` | `-stdin` toggle, or env when the node took stdin | always `@env:` |
| xpub-search `--phrase` + `--passphrase` | passphrase `--passphrase-stdin` | `@env:` |
| slip39 combine, shares + passphrase | passphrase `--passphrase-stdin` | `@env:` |
| `ms combine`, N shares | all on stdin via one `-`, one per line | share 1 `PosDash`; share 2 has no channel (`--in` fails closed in A2) → **refuse** |

Consequences:
- The `ms combine` plan is not expressible in the `Channel` enum at all. Positional `-` today means one share.
- The combinations the rule actually produces (for example `--from phrase=@env:A --passphrase @env:B`, two env references in one run) are **not among the measured combos** (a–j).
- T6 and the tutorial re-pin will freeze whichever reading the implementer picks.

None of these is a wrong wallet (every cell involved is OK on its own), but it fails "implementable without guessing".

**Remedy:** make one of the two authoritative. Either the rule gains a preference ("a passphrase-class source prefers its `StdinToggle`") and A5 is regenerated from it, or A5 becomes the table. Add an explicit `StdinMulti` channel for `ms combine`, and measure every shape the rule emits as a combo.

### I2. The multi-secret acceptance tests cannot catch a swap in the runner, and T3 cannot pass on symmetric shapes

**The swap-regression question (brief item 8):**

- **T1 is pure.** It checks the `RunPlan` data structure. It catches a planner swap, but not a defect in the code that turns a `RunPlan` into a `Command`: env values applied to the wrong names, fds dup'd to the wrong number, or stdin written from the wrong binding.
- **T2 is single-entry.** It never runs a multi-secret plan against the argv baseline.
- **T3 compares the swapped-values run with the unswapped run, both through the planner.** If the runner systematically swaps two env bindings, both runs are swapped relative to intent and still differ from each other, so **T3 passes**. T5's "swap two env bindings (T3)" mutation therefore stays green if it is applied in the runner rather than in `plan()`.
- **T3 as written ("assert stdout changes, or the CLI refuses") is unsatisfiable** for `seed-xor combine`, `slip39 combine`, `ms-shares combine` and `ms combine`. I measured identical output after a swap on the first, second and fourth. An implementer will either weaken T3 or special-case those shapes silently.

**Remedy.** Add a T3′ (this is the test that catches a swapped-secret regression). For each asymmetric multi-secret shape in A5, run `plan()` through the real `runner` and assert **stdout equals the argv + `--allow-argv-secret` baseline**: address or fingerprint equality, not just exit 0. Also assert that the values-swapped plan differs from that baseline. Asymmetric fixtures:
- bundle `wsh-multi` two slots (a/a2 measured different);
- `addresses --from` + `--passphrase`;
- `convert --from wif=` + `--bip38-passphrase`;
- `silent-payment` secret + passphrase;
- `ms derive` fd + passphrase;
- `ms verify` phrase + fd.

Carve the four symmetric combines out of the "must change" leg explicitly, keeping only their baseline-equality leg.

### I3. The Windows and macOS legs have no executing gate, yet T5 claims a Windows mutation check

- Real-binary tests run only in `schema-mirror.yml` (`ubuntu-latest`, `cargo test --workspace` with `*_BIN` set). Every other job runs without the CLIs.
- The A6 Windows temp-file path would be new code with zero real-binary execution. T5's "delete the temp file before spawn (Windows cell)" cannot turn anything red in CI: a gate that has never run.
- macOS `/dev/fd/N` for `ms --in` and `--secret-file` is claimed under "Unix" but measured only on Linux.

The failure modes are fail-closed or exposure-only, so this is not a funds finding. It is the "claimed gate that cannot fail" class.

**Remedy:** either take Q3 = refuse on Windows (then there is nothing to gate) and add a macOS real-binary job, or refuse the fd shapes on macOS until measured. Remove the unrunnable T5 claim.

---

## Minor

- **M1. "Secret source" is not defined in A4.** State it as schema `secret: true` flags ∪ secret slot subkeys (`SECRET_SLOT_SUBKEYS`) ∪ Text values matching `text_value_is_secret_node_token` ∪ secret positionals ∪ the hand-marked hashlock inputs. If an implementer uses schema-secret only, `restore --from ms1=<card>` is not a source: it reaches argv with no admission, and the CLI refuses it. That fails closed, but it breaks the main restore flow.
- **M2. The inventory is incomplete.** These secret inputs are in neither A2 nor the "not measured" list, so they will refuse silently after the admission is deleted:
  - `convert --from bip38=` (A5 row h even plans it);
  - `slip39 split --from entropy=`;
  - `ms-shares split --from entropy=`;
  - `xpub-search passphrase-of-xpub --ms1`;
  - `ms hashlock <ms1>` positional (`-` / `--in` are claimed in B5 but not measured).

  List them, or measure them.
- **M3. Copy in a refusal case is undefined.** A8 says "the Copy command is still produced", and A7 says it "uses the same channel spellings", but a refused plan has no spelling. It must never synthesise an unmeasured spelling (a generic `<flag> -` pasted into a shell is the F-687 wrong wallet). Specify it: either the CLI's own recipe from its refusal message, or the old masked argv with a note.
- **M4. The fd pipe mechanics are underspecified.**
  - Write the whole secret into the pipe and close the write end **before spawn**. Secrets are ≪ 4 KiB, the minimum pipe capacity even under `pipe-user-pages-soft` pressure. That removes both the deadlock and the EPIPE/SIGPIPE classes. The design leaves the ordering open.
  - The write end must be CLOEXEC. A leaked write end means the child never sees EOF on fd 3, and the synchronous runner freezes the GUI (A6: no cancel).
  - Guard the `dup2(n, n)` no-op that leaves CLOEXEC set when the read end already has the target number (`command-fds` handles this; a hand-written `pre_exec` might not).
- **M5. The greedy stdin rule depends on schema order.** It works for silent-payment only because `--passphrase` (stdin-only) sorts before `--secret` (stdin or file). Reversed, it refuses a satisfiable plan, and T1's refusal leg would not notice (it asserts refusal only for missing entries). Assign stdin first to sources with no other non-env channel.
- **M6. ms hashlock.**
  - The no-trim test is required in B5 but absent from A9. Add it to A9 (the real `ms`, `"  pad  "` ≠ `"pad"`).
  - Add a test that the default `--kind` state emits no `--kind` token. The "(not specified)" sentinel matches the CLI, but the result pane then shows a **sha256 record on stdout** (measured: `hash:3cf5…` and "no --kind given; stdout carries the sha256 record"). That is the F-553 pipe hazard in GUI form. Consider showing the per-kind notice prominently, or requiring an explicit choice before Run.
  - Filling both phrase and `--hex` would hit A8.2's "no second channel" refusal, while the real reason is "exactly one source". Give it a source-exclusivity conditional instead.
- **M7. Tree mode.** The POSIX Copy "printf pipeline stays as it is", but if a tree-mode secret source is env-bound, the pipeline needs the same `export` comment lines as A7's argv Copy.

## Nit

- **N1.** A2's legend says "WRONG means exit 0 (or 4)", but `table.py` prints the fixed string "WRONG (exit 0/4, literal)" without distinguishing which. Harmless.
- **N2.** A6 says the Windows temp directory "inherits the user-profile ACL (owner-only in practice)". That is exposure-class (non-gating), but the claim is unmeasured, and the startup sweep deletes a *concurrent* GUI instance's in-flight file (fails closed).

---

## Open questions — recommendations

- **Q1 (keep the confirm modal for fully private runs): keep it.** It is the only surface besides Preview that shows the binding list ("`--passphrase ← stdin`"), which is where the user can catch a channel they did not expect. Reword the first sentence as planned; the J1 re-pin is needed either way.
- **Q2 (hashlock before or after Part A): after.** Its phrase has exactly one private channel (`--hashlock-phrase-stdin`), which is Part A's `StdinToggle` case. Building it first means building it on the admission path that Part A deletes, then re-plumbing it. The phrase is byte-verbatim on both paths (measured), so there is no correctness gain from going first.
- **Q3 (Windows temp files vs refusal): refuse the three shapes on Windows this cycle** (`ms verify` phrase+ms1, `ms derive` ms1+passphrase, `silent-payment` secret+passphrase), with the CLI recipe in the refusal. A temp file adds crash residue, a multi-instance sweep race and an unrunnable gate (I3), for three rare shapes. File a follow-up to measure a Windows named pipe (`\\.\pipe\…` opened by path), which may avoid disk entirely.
- **Q4 (measure the unmeasured rows or ship refusing): measure them in the implementation cycle, before deleting the admission.** A4 already makes deletion conditional on full coverage, and the refusal set includes `verify-bundle`, the post-engrave check. The harness's verify-bundle rows were invalid because the fixture passed no `--ms1` and split card text on whitespace (tokens like `md1:`), not because the rows cannot be measured. With `bundle --json`'s `ms1`/`mk1`/`md1` arrays plus `--ms1`, the baseline is `result: ok`. `--slot @0.phrase=@env:` + `--passphrase-stdin` + `--ms1 @env:` reproduced it exactly. A wrong passphrase and the literal `-` both give the *same* "mismatch" text, so verify-bundle rows must use a **matching** baseline, or dependence is vacuous. Also measure M2's list.

---

**NOT GREEN (1C / 3I)**. C1 blocks: the design must decide what a user-typed `-` / `@env:` in a secret field means, and one natural reading silently yields fingerprint `fee4dc32` instead of `ca2c62d2`. I1 and I2 block implementability and the swap gate respectively.
