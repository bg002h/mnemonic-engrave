# CONTINUITY — the `mt` cycle, 2026-08-24

> Supersedes `CONTINUITY_mt_2026-08-23.md`, which described the pre-implementation
> state. **The plan is EXECUTED.** Everything below is about what shipped and
> what reviewing it cost.

## Where the code is

**A different repository.** `bg002h/mnemonic-transaction`, **PRIVATE**, at
`/scratch/code/shibboleth/mnemonic-transaction`. The spec and plan stay here, in
`mnemonic-engrave/design/`; copies under `mnemonic-transaction/design/` are
checked against them by `scripts/check-provenance.sh`.

**No `ci/staging` ritual there.** That dance exists for `mnemonic-engrave`, whose
`master` has branch protection. `mnemonic-transaction` is private on GitHub Free,
so protection is unavailable and a plain `git push origin main` is correct — CI
still runs and still has to be checked per-job.

## State

S0 and P0–P6 shipped. CI green. **198 tests**, and six gates:

| gate | what it asserts | in CI? |
| --- | --- | --- |
| `cargo nextest run --locked` | the suite | yes |
| `check-refusal-coverage.sh` | `refusals.toml` ↔ tests is a bijection, over **every** suite | yes |
| `mutate-refusals.sh` | all **30** refusal tests go RED without their check | yes |
| `journeys.sh` | A (encode) / B (recover, both forms) / C (miscut), on what the operator SEES | yes |
| `check-provenance.sh` | the copied design files still match `mnemonic-engrave` | **no** — needs the other repo |
| `live-smoke-test.sh` | encode → verify → inspect → decode → **`testmempoolaccept`** | **no** — needs a funded `bitcoind` |

**`mutate-refusals.sh` OWNS THE TREE while it runs** — it does `rm -rf src` and
restores between entries. Do not edit, commit or format anything until it exits;
running it against a copy is safer. This bit me: a restore landed on an edit,
reverted it silently, and the run then reported a failure that was not real.

## What the review actually cost, and what it bought

**The mandatory post-implementation review found NINE CRITICALS in code that was
already green** — 160 tests, three gates and CI all passing before a reviewer was
dispatched. Eleven lenses ran; every report is persisted verbatim in
`mnemonic-transaction/design/agent-reports/`, each in its own commit *before* the
fold answering it, so `git diff <persist>..<fold>` means something.

The four worth remembering as classes:

- **`verify` asserted a check that did not exist.** §1.1's content-id
  re-derivation had no code behind it, so `verify` printed *"transaction
  re-derives"* on every run — and `decode` emitted the **wrong transaction's**
  broadcastable hex for a forged set. *A sentence in a spec is not a test.*
- **§8.2f was bypassed by the invocation it exists to refuse.** `mt encode <hex>`
  never reached the guard: clap rejects the positional argument first, and
  **clap's error echoes the bearer transaction**. *A guard downstream of the
  parser has already lost.*
- **§8.4 was essentially unbuilt** — no `LOCK_TIME_THRESHOLD` branch, no
  `nSequence` rule — so both failures §8.4 names in its own text were shipped.
  *The spec-first lens found it; a code-first pass finds only what the code
  chose to do.*
- **`encode` told a CONFIRMED transaction it "can never be broadcast"** and
  advised building a new one — advice to pay twice. *Found by running it against
  a real node; no stub models the difference, because all three §8.5 cases share
  `gettxout → null`.*

### The pattern that dominated the folds

**Every guard added in response to a finding broke on the NEAR MISS** — the input
that merely resembles the one the finding named. Five instances:

| the fix | what it broke |
| --- | --- |
| widen §8.2f to catch `mt1…` | refused a legitimate **filename** `mt1-…-transfer.txt` |
| print the plate legend | printed the sum of **all** outputs as the destination amount, for steel |
| repair `mtl` → `mt1` | rewrote any **elided** line whose first three symbols were `m,t,l` (1 set in ~4,000) |
| refuse that `mtl` case instead | refused the legitimate elided line — the same defect reversed |
| repair `b` → `6` | mt guessed, BCH spent **2 of 4 repairs** undoing the guess, and the notice called it free |

The finding hands you a hostile X and never the legitimate near-X, because the
reviewer was hunting the hole, not the rim. **Before committing a fold that adds
or widens a guard: run the hostile input (must be caught) AND the nearest
legitimate one (must pass), and keep both as tests.**

## Open

- Three **spec** defects found by implementing the spec: **F-238** (`~FALL 2034`
  disagrees with §8.4's own algorithm, which gives SUMMER), **F-239** (§8.4 gives
  one state two spellings without saying they are two surfaces), **F-240**
  (§1.1's row table disagrees with its own `verify` example).
- Three v0.1 residues: **F-235** (addresses render with mainnet parameters),
  **F-236** (`--input-value` `f64` — since fixed, entry stale), **F-237** (no
  hint for a sibling `md1`/`mk1` string).
- A round-4 fold check was in flight at the time of writing; its report will be
  `R-round4-fold-check.md`.

## Resume

Read `mnemonic-transaction/README.md` first — it names the six gates and the
stdout/stderr boundary. Then `git log --oneline` there: the persist/fold
alternation is the record of what was found and what changed in response.
