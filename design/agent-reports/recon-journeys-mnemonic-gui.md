# Recon: round-trip journey inventory — `mnemonic-gui`

**Scope:** read-only inventory of round-trip journeys that ALREADY EXIST in
`/scratch/code/shibboleth/mnemonic-gui`, per the definition in
`mnemonic-engrave/design/DRAFT_round_trip_journey_definition.md` (§1–§7, and the
§8 operator rulings, all inherited as binding). This audit does **not**
enumerate journeys that "should" exist (deferred by ruling 3) and does **not**
step into any other repo's code (cross-repo invocations are recorded as facts
only).

## What I actually ran

Every claim below with a concrete number, hash, exit code, or "ran/passed" was
produced by one of these commands, executed against the real working tree at
HEAD (`82fc3f8`) with the pinned CLIs actually installed on `$PATH`
(`mnemonic 0.97.0`, `md 0.13.0`, `ms 0.16.0`, `mk 0.13.0` — matching
`pinned-upstream.toml`'s `mnemonic-toolkit-v0.97.0` pin):

```
grep -rIl -i "round.trip" tests src design
grep -rIl -i "independent.oracle" tests src design
grep -rIl "Command::new" tests
grep -rIl -i "journey" tests src design
find . -iname "*journey*" -not -path "./target/*" -not -path "./.git/*"
cargo test --test bundle_restore_independent_oracle -- --nocapture
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json WGPU_BACKEND=vulkan \
  GUI_TUTORIAL_SNAPSHOTS=1 cargo test --test gui_tutorial_snapshots \
  gui_tutorial_snapshots -- --nocapture --test-threads=1
cargo test --test gui_tutorial_snapshots -- --test-threads=1   # all 12 cells, unskewed
cargo test --test runner_integration -- --test-threads=1
MNEMONIC_BIN=mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk \
  cargo test --test ui_harness_i4_realcli -- --test-threads=1
mnemonic bundle --template bip84 --slot @0.phrase=<S0> --json      # confirm bundle JSON shape
mnemonic restore --from phrase=<S0> --template bip84 --json        # confirm restore JSON shape (no change field)
mnemonic addresses --help                                          # confirm --chain change EXISTS (capability check)
sha256sum tests/tutorial/fixtures/{policy.desc,taproot.desc,taproot-4leaf.desc,policy.json}
grep -rn "MNEMONIC_BIN|GUI_TUTORIAL_SNAPSHOTS|MD_BIN|MS_BIN|MK_BIN" .github/workflows/
```

I read full source for: `tests/gui_tutorial_snapshots.rs` (1661 lines, both
halves), `tests/tutorial/mod.rs`, `tests/tutorial/manifest.rs`,
`tests/bundle_restore_independent_oracle.rs`, `tests/ui_harness_i4_realcli.rs`,
`tests/runner_integration.rs`, `tests/tree_round_trip.rs`,
`tests/tutorial/fixtures/README.md`, `pinned-upstream.toml`, and the relevant
slices of `.github/workflows/build.yml` / `schema-mirror.yml`. Everything else
(`ui_harness_i1_roundtrip.rs`, `wire_shape_snapshot.rs`, `canonicity_drift.rs`,
`gui_render_emit.rs`, `kittest_import_wallet_form.rs`) I opened enough (header
comment + first ~40 lines + the assertion shape) to classify and rule out as
NOT a round-trip journey per §1/§4 — none of them assert a funds-controlling
structural+functional pair; they check UI wiring, rendered ASCII, or schema/
wire-shape conformance instead. I did not open every test file in `tests/`
(103 files) byte-for-byte; the two greps above (`round.trip`, `Command::new`)
were the completeness mechanism for the negative claim below, cross-checked by
the `journey` keyword grep and a `find -iname '*journey*'` (empty — no
`design/journeys/` directory exists in this repo, unlike the sibling
`mnemonic-engrave`).

---

## Journeys found: 3 files, 7 distinct journeys

Two files hold journeys that cleanly satisfy §1: `bundle_restore_independent_oracle.rs`
(1 journey) and `gui_tutorial_snapshots.rs` (5 journeys, J1–J5, sharing one
underlying test binary — see the shared-command caveat below each). A third,
`runner_integration.rs::cell_1`, is a borderline custodial journey with a
combined (not split) assertion. `ui_harness_i4_realcli.rs` is recorded
separately below as **NOT journeys** (decode-only, no encode leg, and 3 of 4
cells have no funds-relevant functional field at all) — included because the
definition's own words ("custodial... exercises only the decode side") made me
check hard before excluding it, and the finding is itself informative.

### J-A. `bundle_restore_independent_oracle` — the strongest journey in the repo

| field | value |
|---|---|
| name | `bundle_bip84_all_zero_then_restore_matches_external_oracles` |
| kind | **generative** — origin is a fixed BIP-39 phrase (S0, all-zero 12-word), per §8 ruling 2 |
| tier | **T2** (CLI→CLI, separate processes, real exit codes) — with a caveat: the two legs are NOT file-mediated; the bundle leg's stdout is captured in-process and its `ms1`/`md1` fields are spliced straight into the restore leg's `FormState` by the test itself, not via a written file. Named as non-coverage below. |
| origin artifact | `SEED_PHRASE_ALL_ZERO` (`abandon…about`), a manifest literal |
| ordered invocations, repo-by-repo | `mnemonic-gui`: `assemble_argv` (own assembler, not hand-rolled) → `runner::run` spawns **`mnemonic`** (external binary, built from the sibling `mnemonic-toolkit` repo, pinned `v0.97.0` — cross-repo fact, not audited) `bundle --template bip84 --slot @0.phrase=<S0> --json` → parse stdout JSON → `mnemonic restore --from ms1=<emitted> --md1 <chunks…> --json` → parse stdout JSON. All one process (`cargo test`), two child-process spawns. |
| structural assertion | `bundle`'s emitted `ms1` == `MS1_ALL_ZERO` (a published `ms vectors` constant, external to the GUI's encode path) **and** `master_fingerprint` == `73c5da0a` (R0-verified literal) |
| functional assertion | `restore`'s `wallets[0].first_addresses[0]` == `BIP84_FIRST_RECEIVE_ADDRESS_ALL_ZERO`, stated as "triple-verified" (bip-0084.mediawiki + an independent stdlib derivation + the pinned binary's own round trip) |
| the ONE command | `cargo test --test bundle_restore_independent_oracle` (needs `mnemonic` on `$PATH` or `MNEMONIC_BIN` set; else early-return-skip — see the skip-passes-green finding below) |
| stated non-coverage | Explicit in its own doc comment: scoped to `--template bip84` ONLY; a `--descriptor`-mode variant needs a toolkit pin bump the GUI hasn't taken, tracked as follow-up `gui-descriptor-mode-bundle-restore-independent-oracle`. Multisig, taproot, and miniscript-vault shapes are **not** covered by this journey at all (see J1–J5 below, which cover those shapes but with the opposite weakness). |
| **EXECUTED (measured)** | `cargo test --test bundle_restore_independent_oracle -- --nocapture` → `test bundle_bip84_all_zero_then_restore_matches_external_oracles ... ok`, 1 passed, 0.02s. |

**Finding (§4, "Change addresses are not optional"):** this journey asserts
**receive only**. I confirmed independently that `mnemonic restore --json`'s
JSON shape carries a single-element `first_addresses` array (no change-address
field) for this template/network — ran it myself:
`mnemonic restore --from phrase=<S0> --template bip84 --network mainnet --json`
→ `"first_addresses": ["bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"]`, nothing
else address-shaped. But the tooling is NOT the blocker: `mnemonic addresses
--help` shows `--chain change` exists precisely for this. So the missing
change-address check is a genuine, fixable test gap, not a CLI limitation.
Grepped the whole `tests/` tree for `change_address`/`"change"` address
assertions — none found anywhere in the repo (the many other `change`-string
hits are unrelated: FormState-change events, GUI diff-change, etc.).

---

### J1–J5. `gui_tutorial_snapshots` — the whole-window GUI capture harness

One `#[test] fn gui_tutorial_snapshots()` drives the REAL compiled app
(`MnemonicGuiApp::new_headless`, whole `app.ui()` render tree) through
`egui_kittest::Harness` at a fixed 920×720 @ ppp 2.0 under a software (CPU)
Vulkan rasterizer (verified: `TUTORIAL-ADAPTER: … device_type: Cpu … llvmpipe`
in my own run), simulating clicks/typing/scrolling via injected AccessKit
action requests and pointer/wheel events, capturing PNG screenshots and
byte-persisting every spawned CLI's stdout/stderr/exit-code. It spawns the
real pinned `mnemonic` binary for every `runs: true` step (version-gated
first: `TUTORIAL-GATE-OK: mnemonic 0.97.0`). This is the closest thing in the
repo to §3's T3 ("real screens, real input transport... proves a user can do
the thing"), analogized from device to GUI-app since there is no physical
device here — **that analogy is itself a substitution and is named as
non-coverage below.**

**EXECUTED (measured), the whole harness in one run:**
`VK_ICD_FILENAMES=… WGPU_BACKEND=vulkan GUI_TUTORIAL_SNAPSHOTS=1 cargo test
--test gui_tutorial_snapshots gui_tutorial_snapshots -- --nocapture
--test-threads=1` → `ok`, **50 PNGs, 27.109 MiB** (under the 32 MiB hard
ceiling), **104.73s**, all 25 shot-bearing + transcript-only steps across
J1–J5 passed byte-for-byte against the committed corpus. I additionally ran
the file's other 11 always-on cells (`cargo test --test gui_tutorial_snapshots`,
no env var) — all 12 tests `ok` in that same invocation.

**Shared-command caveat (a §7 finding in itself):** all five journeys below
are steps inside ONE `MANIFEST: &[Step]` array processed by ONE `#[test] fn`.
There is no way to run "just J3" with a single command — the one command that
exists runs J1 through J5 together, sequentially, in one process. Individually
addressable-by-name journeys would need the manifest split or filtered.

Per §2/§8-ruling-4, I inventoried these as five separate journeys (not
variations) because their **kind differs** — J1/J2 originate at raw seed
phrases (generative); J3/J4/J5 originate at already-public watch-only
descriptors (custodial) — and conflating that distinction is exactly the trap
§2 names first.

| # | name (stem prefix) | kind | origin artifact |
|---|---|---|---|
| J1 | `tut-j1-01-bundle-single-sig` | generative | S0 phrase (typed into the slot editor) |
| J2 | `tut-j2-02..08` (convert ×2 shot + 4 transcript-only + canonicalise/bsms/bundle/restore) | generative | S0, S1, S2 phrases (3 cosigners) |
| J3 | `tut-j3-09..13` | custodial | `policy.desc` fixture — an 11-key pathological wsh vault descriptor, watch-only, vendored byte-copy of `mnemonic-toolkit`'s `.examples-build/degrade2.desc` |
| J4 | `tut-j4-14..21` | custodial | `taproot.desc` / `taproot-4leaf.desc` fixtures — watch-only tr(...) descriptors, same vendoring |
| J5 | `tut-j5-22..24` | custodial | `MULTISIG_DESC` — a reviewed public constant assembled by hand from S0/S1/S2's derived xpubs (NOT produced by the journey itself — see finding below) |

Tier for all five: **T3-by-analogy** (real rendered screens via a software
rasterizer, simulated-but-real widget interaction via AccessKit, real spawned
CLI). Non-coverage, named explicitly per §3.1's binding mitigation-style
requirement: input transport is `egui_kittest`-injected (AccessKit
`Action::Click`/`SetValue`, injected `PointerMoved`/`MouseWheel` events), not
literal OS-level mouse/keyboard/touch; rendering is CPU (`llvmpipe`/lavapipe),
not a real GPU/display path; the intra-journey "chain" (parsing a bundle run's
`md1` output and typing it into the next step's `--md1` rows) is glue code the
**test harness** performs (`ChainStore`, `parse_md1_chunks`), not something a
real operator does by reading the screen and re-typing — a real user's transfer
step (screenshot→typed, or copy-paste) is not exercised.

One command that runs ALL five (shared, per the caveat above):
```
GUI_TUTORIAL_SNAPSHOTS=1 WGPU_BACKEND=vulkan cargo test --test gui_tutorial_snapshots gui_tutorial_snapshots -- --nocapture
```
(plus a software Vulkan ICD on `$PATH`/`VK_ICD_FILENAMES` — CI installs
`mesa-vulkan-drivers`; I used the already-present `lvp_icd.json`.)

**Two findings apply to ALL FIVE of J1–J5 equally, and are the headline
result of this recon:**

1. **No independent functional assertion, anywhere in J1–J5.** `execute_step`'s
   post-run checks (`tests/gui_tutorial_snapshots.rs:604-666`) are generic across
   every step: exit code matches `expect_exit`, `argv[0]` is the bare CLI name,
   stderr-presence matches `expect_stderr`, secret values are display-masked.
   **None of these read or compare a receive address, change address, master
   fingerprint, or wallet id against an independent source.** The only
   correctness signal for "did the restore actually reconstruct the same
   wallet" is `persist_transcripts`'s byte-comparison against a **committed
   golden `.stdout.txt`** — which the SAME harness, run with
   `UPDATE_SNAPSHOTS=1`, regenerates from the SAME code path
   (`tests/gui_tutorial_snapshots.rs:1482-1526`, the `update` branch calls
   `std::fs::write` with the just-produced bytes). This is **exactly** the §5
   anti-requirement ("must not assert against a value the journey itself
   produced with no independent source... a snapshot test blesses whatever the
   code did, bug included"). **This is not my inference** — the repo's own
   `bundle_restore_independent_oracle.rs` doc comment (lines 6-11) says so
   explicitly about this exact file: *"`tests/tutorial/*` drives bundle→restore
   end-to-end but byte-gates against a COMMITTED golden it re-captures — a
   symmetric bug shifting both the encode path and the expected golden in
   lockstep would stay green forever."* I independently traced the mechanism
   (`persist_transcripts` + the `UPDATE_SNAPSHOTS` regen path) and confirm the
   claim is structurally accurate, not just asserted. `bundle_restore_
   independent_oracle.rs` was built specifically to plug this gap — but **only**
   for J1's shape (bip84 template-mode single-sig); J2 (multisig), J3
   (pathological miniscript vault), J4 (taproot), and J5 (format variants) have
   **no** independent-oracle coverage at all.

2. **Skip prints `ok`, not fail (§5 anti-requirement, verified structurally).**
   `gui_tutorial_snapshots()` and `same_frame_completion_direct_click_class()`
   both open with `if env::var("GUI_TUTORIAL_SNAPSHOTS") != Ok("1") { eprintln!(…); return; }`
   — an early return from a `#[test] fn` reports `ok`. I confirmed this is not
   theoretical: my second run (`cargo test --test gui_tutorial_snapshots`, no
   env var set) printed both as `ok` in 0.00s. The gate's teeth live entirely
   in CI wiring: `.github/workflows/build.yml`'s `tutorial-snapshots` job sets
   `GUI_TUTORIAL_SNAPSHOTS: "1"` and separately installs `mesa-vulkan-drivers`
   + the pinned `mnemonic` binary (verified: lines 179-234, this is the "P1.6
   permanent tutorial-corpus gate" the file's own header names as its enforcing
   consumer). So the skip-passes-green shape is real but **currently dormant**
   in the enforcing CI job specifically — it would bite on any other job,
   fork, or local run that runs `cargo test --workspace` without the env var
   and without objecting to the resulting easy green.

**J5-specific finding:** `MULTISIG_DESC` / `TAPROOT_MULTI_DESC` (the
descriptors J2/J5 and J4's NUMS step feed into `export-wallet`/`restore`) are
**hand-assembled public constants**, not chained from the J2 convert steps'
own live output — `tests/tutorial/mod.rs:56-69` says so explicitly ("Examples'
`gen.sh` assembles them INLINE from the three cosigners' live `mnemonic
convert` fp+xpub outputs... NOT chained [here]... assembling them from six
convert runs would add bespoke fp+xpub string-templating for zero determinism
benefit"). The J2 convert steps DO run for real and display each cosigner's
real derivation on screen, but the descriptor used downstream is a
reviewed-by-hand literal, not the journey's own produced artifact. This is a
deliberate, documented design choice (not a bug the repo is unaware of), but
per the §7 rule ("a path whose 'expected' values were transcribed by hand...
is A FINDING") it belongs in this inventory as exactly that: a hand-transcribed
value standing in for a chained one, at the one point in J2/J5 where a
chained value would have been possible.

---

### Borderline: `runner_integration.rs::cell_1_mnemonic_export_wallet_byte_exact`

| field | value |
|---|---|
| kind | **custodial** — origin is an already-derived zpub + master fingerprint (a Trezor 24-word test vector's PUBLIC output), not a seed |
| tier | T2 |
| origin artifact | `TREZOR_24_BIP84_ZPUB` + `TREZOR_24_MASTER_FP` (manifest literals) |
| invocations | `mnemonic-gui`: `assemble_argv` → `runner::run` spawns `mnemonic export-wallet --template bip84 --network mainnet --format coldcard --output -` with the zpub/fp fed via slot rows |
| structural + functional | **Combined into ONE `assert_eq!`**, not split: `stdout == expected` where `expected` is `tests/fixtures/coldcard_generic_bip84_mainnet.json`, vendored (per its own doc comment) from `mnemonic-toolkit-v0.14.0`'s own test fixture — i.e. an externally-sourced, not self-produced, oracle (compliant with §5's independent-source rule). That fixture's JSON DOES carry a `"first": "bc1qzmt…"` receive address (I read the file directly), so a functional signal is present, but the code asserts it only as part of one big byte-exact string compare, not as a separately named structural-vs-functional pair the way §7's schema asks for. |
| ONE command | `cargo test --test runner_integration cell_1_mnemonic_export_wallet_byte_exact` |
| non-coverage | Receive address only (again no change address); single-sig bip84/coldcard format only. |
| **EXECUTED (measured)** | `cargo test --test runner_integration -- --test-threads=1` → all 4 cells `ok` in 0.13s, including `cell_1`. |

---

## NOT journeys (checked and excluded, with reasons — not a first-hit stop)

- **`ui_harness_i4_realcli.rs`** (4 cells: `mnemonic decode-address`, `md
  decode`, `ms decode`, `mk decode`) — custodial, T2, decode-only, real CLI
  spawn (confirmed genuinely spawned, not silently skipped: `md` is a `fish`
  shell alias for `mkdir -p` in this environment, but `std::process::Command`
  bypasses shell aliases entirely — I independently located the real `md`
  binary at `/home/bcg/.cargo/bin/md`, version `0.13.0`, and ran the suite
  with `MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN` set: all 4 cells `ok` in 0.05s).
  **Excluded as a "journey"** because 3 of its 4 cells assert only
  structural-shaped fields (schema tag, tree tag, entropy hex, word count,
  language) with **no** receive/change address, fingerprint, or wallet-id
  check at all — no functional assertion per §4's definition. The 4th
  (`mk decode`) asserts `origin_fingerprint`, which plausibly qualifies as
  §4's "master fingerprint" category, but even there the cell has no paired
  *structural* equality distinct from that same field. None of the 4 exercise
  an encode leg, so there is no "round trip" shape to speak of beyond a single
  decode-and-check — legitimate under §2's custodial definition in principle,
  but disqualified here by the missing-functional-assertion rule in §7.

- **`ui_harness_i1_roundtrip.rs`** — "round-trip" in the name refers to
  form-render → widget-inject → `assemble_argv` wiring fidelity (does a typed
  value reach argv under its flag name), entirely in-process, no funds
  material, no CLI spawn. Not a round-trip journey per §1/§4.

- **`wire_shape_snapshot.rs`** — pins the `--json` key-SET shape of
  `xpub-search`/`import-wallet` against goldens the file's own header says
  were "captured FROM the v0.70.0 binary" (an external capture, not a live
  independent oracle, and not self-regenerated in this file — no
  `UPDATE_SNAPSHOTS` branch present). No bundle/restore, no address/fingerprint
  equality. Named explicitly by `bundle_restore_independent_oracle.rs`'s own
  doc comment as NOT invoking bundle/restore at all.

- **`tree_round_trip.rs`** — "round trip" here means the descriptor-builder's
  in-process data-model laws (`to(from(j)) == j`, TreeState serde
  round-trip, xprv-redaction round-trip). One cell,
  `golden_fixtures_exit_zero_through_binary`, DOES spawn the real `mnemonic
  build-descriptor --spec - --json` CLI, but asserts only `exit 0` ("a
  staleness tether," its own comment says) — no structural or functional
  content assertion at all, so it fails §7's bar even before kind/tier
  matter.

- **`canonicity_drift.rs`, `gui_render_emit.rs`, `kittest_import_wallet_form.rs`**
  — schema/wire/render conformance and argv-wiring cells; none carry a
  funds-relevant structural+functional equality pair.

## Anti-requirement (§5) sweep, summary

| anti-requirement | verdict for this repo |
|---|---|
| reads an intermediate nothing writes | Not found. `gui_tutorial_snapshots`'s `TypeMd1Chain` reads `chain.get(chain_from)`, and I traced the writer: `execute_step` populates `chain.insert(step.stem, …)` after every run (line ~655), unconditionally — the reader always has a prior writer in program order. |
| asserts against a self-produced value | **Found** — J1–J5 (see above), acknowledged by the repo's own comments elsewhere. |
| a skipped step passes instead of failing | **Found**, structurally confirmed by direct execution (see above) — pattern repeats across `bundle_restore_independent_oracle.rs`, `ui_harness_i4_realcli.rs` (×4), `runner_integration.rs` (uses a default rather than skip, so NOT applicable there — `mnemonic_bin()` defaults to `"mnemonic"` and always attempts the spawn), `canonicity_drift.rs`, `tree_round_trip.rs`'s tether cell. All are CI-covered today (verified: `schema-mirror.yml`'s `cargo-test-full-suite` sets all four `*_BIN` vars for `cargo test --workspace`; `build.yml`'s `tutorial-snapshots` job sets `GUI_TUTORIAL_SNAPSHOTS=1`), so the failure mode is dormant, not live — but the shape is real. |
| every gate has executed at least once | Every journey reported above I executed personally in this session (see "what I actually ran"); none is a hypothesis. |
| empty output is not proof of absence | Applied: I did not rely on the single `round.trip` grep to conclude "3 files"; I cross-checked with a `Command::new` grep (finds real-CLI-spawning tests independent of naming), a `journey` keyword grep, and a `find -iname '*journey*'` (confirms no dedicated journeys directory, unlike the sibling `mnemonic-engrave` repo — a real absence, verified two ways, not a grep miss). |

## The known blind spot (§8 ruling 3, restated as required)

This is a single-repo sweep. It cannot see gaps *between* `mnemonic-gui` and
`mnemonic-toolkit` (the actual `mnemonic`/`md`/`ms`/`mk` CLI implementations,
a separate repo, not audited here) or `mnemonic-engrave` (the SeedHammer
engraving side — no journey found in this repo reaches an engraved plate; the
GUI's own `bundle` output is the furthest any journey here goes). Every
journey above treats the pinned CLI binaries as an opaque external dependency,
which is exactly where a round trip most often breaks per the operator's own
prior finding history.

## Fields I could not determine

None left as `UNKNOWN` — every journey's eight §7 fields resolved to either a
concrete value or an explicit finding (missing field), and every finding above
is traced to a specific line range or a command I ran myself, not inferred
from a doc comment or file name.
