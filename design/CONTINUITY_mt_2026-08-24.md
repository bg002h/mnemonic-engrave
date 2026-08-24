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

S0 and P0–P6 shipped. CI green. **203 tests**, and six gates:

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
**Nothing else is open.** Thirteen lenses ran; the last two rounds found no
Criticals, and the final verification returned 0 NOT FIXED and 0 regressions.
Its single PARTIAL — the elided half of the misread-separator case — is fixed
and pinned with one case per mode.

**The last defect is the one to learn from.** It survived TWO fixes, and neither
was wrong: both addressed real defects and neither addressed the half that
mattered. A guard measured a malformed line against `full_len`, derived from the
other `mt1` candidates — and with `--elide-prefix` **exactly one line is full**,
so when that line is the corrupted one the reference does not exist and no
tolerance can help. *When a guard derives a yardstick from other data, ask in
which modes that data exists.*

## Resume

Read `mnemonic-transaction/README.md` first — it names the six gates and the
stdout/stderr boundary. Then `git log --oneline` there: the persist/fold
alternation is the record of what was found and what changed in response.


---

# NEXT — two goals, set by the operator 2026-08-24

The `mt` cycle produced a CLI that turns a signed transaction into `mt1` strings
and reads them back. **It cannot engrave anything.** The next two goals close
that gap from both ends.

## Goal 1 — teach SeedHammer a new program: ENGRAVE A TRANSACTION

The device today engraves seeds, descriptors and (since 2026-08-04) a BIP-39
password. It has no notion of a transaction. This is fork-native firmware work
in `third_party/seedhammer` / `bg002h/seedhammer`, in the same family as the
programs already shipped.

**The design constraint that is already ruled, and must not be re-litigated:**
**F-234** — *"every QR carries the STANDARD form, never a codex32 string"*
(operator directive 2026-08-22). A plate carries two representations with two
audiences:

| | engraved TEXT | engraved QR |
| --- | --- | --- |
| format | codex32 (`mt1`) | **the standard form only** — the raw transaction bytes |
| audience | a human with eyes and a keyboard | anyone with a camera and standard Bitcoin tooling |

**F-234 IS OVERDUE, not deferred.** Its owning phase reads *"the mt cycle"*, and
that cycle closed today with the item still open — because QR was deferred out of
v0.1 entirely (§0a) and nothing re-scoped it. **Re-own it to this goal before
starting**, or the burndown rule quietly loses it a second time.

## Goal 2 — teach the OPERATOR to prepare a payload for that program

Goal 1 is useless without this. The operator needs a written, walkable procedure
from *"I want to back up this transaction"* to *"the device is engraving"* —
which is the **journey-walk** method applied before the firmware exists, so the
journey generates the refusals rather than the other way round.

`mt` already produces the pieces: `mt encode` emits the `mt1` strings, the
suggested legend, the cut size and the shared prefix; `mt inspect` is what a
recoverer runs. What does not exist is **how the payload reaches the device** —
the format, the transport, and what the device does when handed the wrong thing.

---

# WHERE THE QR CALCULATIONS LIVE

**`design/SPEC_mt_qr_DEFERRED.md`** (18.5 KB) — the document the operator asked
for. It holds everything the `mt qr` verb would have needed, parked intact rather
than deleted:

- **§4, the configuration search** — the real content. Search space is
  `module size × QR version (1..40) × ECC (L,M,Q,H) × rectangular tiling`, with
  the objective ordered: **minimise plates → maximise ECC → minimise symbol
  count → tie-break on LARGEST module → then smallest version.** Plate cost is
  ~21 minutes (F-225), so plates are minimised first and every leftover byte is
  spent on error correction: *never trade a plate for redundancy, never leave
  redundancy unbought.*
- **The plate geometry**: 85 × 85 mm, 3 mm outer margin → **79 mm usable**;
  quiet zone 4 modules per side per symbol; 6 legend lines reserved on plate 1
  (25.5 mm at a 4.25 mm pitch), 1 line on later plates.
- **The tie-break correction** — ties resolved to whichever module the ascending
  loop reached first, i.e. toward the SMALLEST and least legible symbol. 4
  configurations tie at the 0.60 mm floor for a 162 B payload; **41 tie** once
  the floor lifts.
- **The parked refusals** — §8.7 (plate budget, and it was *unrunnable as
  written*: its threshold had no input path), §8.7c (`sysw` section ceiling),
  §8.8 (module size). §8.7b stayed in the live spec because the chunk ceiling is
  `mt1`'s, not QR's.
- The open questions and the CLI-surface row carried over from §10.

**Also required reading before any QR sizing work**, in `design/FOLLOWUPS.md`:
the **mode-segmentation caution**. A QR encoder does optimal mode segmentation
and will silently re-encode part of a payload in a denser mode — an all-`0x41`
payload measured *alphanumeric* capacity while claiming byte, a high-byte payload
paid an ECI header, and a mixed payload read **6.6% low**. Every one produced a
plausible number. Only asserting measured v40 capacity against the published
limits (**numeric 7089 / alnum 4296 / byte 2953 at L**) caught them. **Any future
QR sizing must carry that gate.**

Related: `design/cycle-prep-recon-T7-seedqr-bip85.md` and
`design/SPIKE_s6b_q2_results.md` also touch QR, from the SeedQR/BIP-85 side.

---

# HOW TO RESTART

1. Read this file, then `mnemonic-transaction/README.md` for what `mt` does now.
2. Read **`design/SPEC_mt_qr_DEFERRED.md` §4** and the mode-segmentation caution
   in `FOLLOWUPS.md` before touching any QR arithmetic.
3. **Re-own F-234** to Goal 1 — it is overdue, and it rules the format the QR
   carries.
4. Both goals are **brainstorm/design phase**, so the constellation rules say
   orchestration is ON for them and the R0 gate applies before any firmware
   code. Goal 1 is also firmly in the RISK SET: fork-native firmware touching
   funds-adjacent material.
5. Goal 2 is a **journey walk**, and the operator should be walked through it
   live — that method found a Critical on this spec after five clean correctness
   rounds, and the operator's own confusion was two of the best findings.
