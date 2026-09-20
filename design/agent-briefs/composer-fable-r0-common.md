# Composer fable review r0 — COMMON FACTS (read this first, then your lens brief)

Four independent fable-tier reviewers, one lens each, of the **Wallet Policy
composer** that ships on the SeedHammer II ("SH2"): the program under
`Wallet Policy > Build a new policy`. Each reviewer writes ONE report; the
controller folds. You are one of the four. Do not do another lens's work; if
you trip over something outside your lens, record it in a short "out of lens"
section and move on.

## Where things are (measured 2026-09-19)

| what | path | tip |
| --- | --- | --- |
| SeedHammer fork (the DEVICE code; Go) | `/scratch/code/shibboleth/seedhammer`, branch `main` | `f5b068faf3049ccf97603dfbaa3709f12893df97` |
| Rust primary for the `md` codec (normative) | `/scratch/code/shibboleth/descriptor-mnemonic` | `922778ad0400957840da26081846fc761deaca41` = md-codec 0.44.2, md-cli 0.16.2 |
| mnemonic-toolkit (host `mnemonic` CLI) | `/scratch/code/shibboleth/mnemonic-toolkit` | `4f1d5126` = 0.103.1 |
| mnemonic-engrave (spec, briefs, REPORTS) | `/scratch/code/shibboleth/mnemonic-engrave` | master |
| the spec | `mnemonic-engrave/design/SPEC_wallet_policy_composer.md` (1206 lines; §4 grammar, §5 lowering, §6 inputs, §7 flow, §8 copy, §12 acceptance, §13 not-verified) | |
| prior reports | `mnemonic-engrave/design/agent-reports/composer-*.md` (~80 files) | |
| SH2 demo payload builder | `mnemonic-engrave/demo/sh2/build-payload.sh` (three seeds, `key:` records, `me sysw pack`) | |

Composer surface in the fork: `gui/composer_*.go` (flow, seat, shape, lock,
hash, hashlock, presets, review, consent, engrave, cards, census, door,
discard, selfcheck, state, paged, digitpad, copy), `sysw/composer_records.go`
(payload `key:` / `hash:` / `now:` / `phrase:` records), `md/` (the Go port of
md-codec: `compose.go`, `compose_stubs.go`, `walletpolicyid.go`, `encode*.go`,
`identity.go`, `chunk.go`, `duplicate_keys.go`, `template_*.go`). The
composer emits NO descriptor text on the device: the md1 chunks are the
artifact; the GUI shows the STRUCTURE (`PolicyShape`) and ids.

What the device CUTS (§7f): **form A** "The policy itself" = keyed md1 strings
(only when every slot is seated); **form B** "Template plus key cards" =
keyless md1 WITH fingerprints + one mk1 card per seated slot (both stubs
appended; partially seated: template stub only); **template only** when no
slot is seated. Seed-derived slots: Full (seed + keys; the seed is cut as an
ms1 plate — `composerSecretCards`) or Watch-only (keys). Hashlock policies
also cut a preimage plate (H6 cycle).

Useful fork tooling: `cmd/policyprobe`, `cmd/buildpayloadcomposer`,
`cmd/journeykeys`, `cmd/emu` (the emulator — a GUI window, not scriptable;
the Go test harness IS the scriptable walk: see `gui/run_harness_test.go`,
`gui/wallet_policy_descriptor_walk_test.go`, `gui/composer_*_test.go`).

Host CLIs, rebuilt from the tips above and on PATH (`~/.cargo/bin`):
`md 0.16.2`, `mnemonic 0.103.1`, `me 0.10.0`. Verify with `--version` before
trusting one.

## Toolchain and hygiene — binding

- Go is `/scratch/code/shibboleth/.toolchain/go/bin/go` (1.26.7): put that
  directory FIRST on PATH. `export TMPDIR=/scratch/code/shibboleth/.tmp`.
  `/tmp` is a 32 GB tmpfs: never build or put a datadir there.
- Work in your OWN detached worktree of the fork:
  `rm -rf /scratch/code/shibboleth/.tmp/fable-<lens> && git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/fable-<lens> f5b068faf3049ccf97603dfbaa3709f12893df97`.
  You MAY add test files / mutate code there to construct counterexamples;
  revert every mutation you do not keep as evidence; remove the worktree at
  the end (`git worktree remove --force`).
- READ-ONLY on every repo checkout listed above. Commit nothing anywhere.
  No sub-agents. Never read any `.jsonl` file.
- Tests: `CGO_ENABLED=0 go test ./md/ ./sysw/ ./mk/ ...` for the codec
  packages; for `./gui/` (886+ serial test funcs) use
  `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
  from the worktree, or `-run '^TestName$' -v` for one test. **A `-run`
  filter that matches nothing prints `ok`** — always confirm with `-v` that
  the test you meant actually ran. `gofmt -l .` at this tip lists five
  pre-existing files (`gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`); that is
  baseline, not a finding.
- Harness facts that have burned reviewers before: the `EventRouter` is one
  global pointer — release a hold before taking the next or the walk stalls
  silently; the SH2 has NO camera (NFC only) and a fixed button set — never
  drive an input the hardware lacks; a `Frame` loop without a wakeup blocks
  on the device though the harness passes.
- Bitcoin Core: `/usr/local/bin/bitcoind` is Core **v25.0** (no BIP-389
  multipath, no tapscript miniscript). Nothing is running. Start your own
  regtest node with a private `-datadir` under `/scratch/code/shibboleth/.tmp/`
  and the ports your lens brief assigns, and stop it when done. A newer Core
  is reachable through nix if you need multipath or `tr()` miniscript:
  `export PATH=/nix/var/nix/profiles/default/bin:$PATH; nix shell nixpkgs#bitcoind -c bitcoind --version`
  (downloads; allowed). A regtest node rejects mainnet xpubs — match the
  network before calling a claim wrong.
- 24 cores. Parallelise long independent work; never run a suite twice to get
  counts and failures separately — capture once, grep twice.

## Settled — do NOT re-derive, do NOT re-file

- Composer stages S0–S4 are shipped and on the device; walk fixes W-1..W-7
  shipped; both boards run a build containing them. gui was 1296/1296 at the
  last measured gate.
- The lowering rules (§5) had two independent expert reviews at spec time:
  `composer-lowering-rules-bitcoin-expert-review.md`,
  `composer-lowering-i1-single-key-head-miniscript-review.md`; the spec had an
  adversarial funds-safety-by-counterexample round
  (`composer-spec-R0-r0-adversarial.md`). Those reviewed the SPEC. Your job is
  the CODE and the ARTIFACT as they ship at the tip above.
- Recon on same-fingerprint / two-accounts admission in Core, Sparrow, Nunchuk,
  Liana: `composer-recon-same-fingerprint-two-accounts-import.md`,
  `composer-recon-core-same-fingerprint.md`, `composer-recon-sparrow-same-fingerprint.md`,
  `composer-recon-taproot-multisig-origin-convention.md`.
- Operator ruling (2026-09-19, verbatim): "ReUsing same key is bad. Reusing
  seed to generate different keys at different keypaths is ok." Same key at
  two slots is refused as UNSUPPORTED (never "invalid"); same seed at two
  accounts is admitted (with the §8g notice when it collapses a threshold).
- md1's narrow origin-path vocabulary is DELIBERATE (F-417): never propose
  widening the wire format; check what md1 can express before arguing
  reachability.
- md-codec 0.44.x (Rust, 2026-09-19) changed three things: (1) rendered xpub
  headers now take depth/child from the origin path instead of depth-0;
  (2) an all-zero parent fingerprint is treated as ABSENT, not as an identity
  that must match; (3) the mint-time validators
  (`validate_origin_key_consistency`, `validate_no_duplicate_key_slots`) no
  longer run on the decode path. **The Go port at the fork tip has NOT been
  re-synced to 0.44.x** — that is known and deferred. "The port is behind" is
  not a finding by itself; a MEASURED consequence for something the device
  does or an artifact it cuts IS.
- The fork's `md/` vectors were printed at descriptor-mnemonic `66bdf2f4`
  (2026-09-02); `md/compose.go` is a port of `md-codec::compose` at that rev.
- Secret-handling defects (material on stderr, argv, shell history,
  world-readable files) are NEVER Critical or Important by operator ruling —
  record them as Minor with a reproduction and move on.

## Severity, and what counts as a finding

Critical = wrong result on steel or in a wallet, funds loss/lock/exposure,
an unmet guarantee, a gate that cannot fail. Important = a real defect,
missing case, unsound assumption. Minor/Nit recorded, not gating.

**A finding is a COUNTEREXAMPLE, not an assessment.** Each Critical/Important
must carry: the exact inputs (policy, records, taps, or command line), the
exact observed output, the expected output and WHY (cite the spec § or the
BIP), and the file:line at the tip where it goes wrong. "This looks risky" is
an out-of-lens note, not a finding. Reproduce the defect; do not prescribe the
remedy beyond one sentence — prescribed fixes are not authoritative here.

## Report — your FINAL action

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-<lens>.md`
(the exact filename is in your lens brief). Structure:

1. Verdict line: counts `C/I/M/N`, one sentence answering the ONE QUESTION.
2. Findings, most severe first, each as above.
3. "What I ran" — every command that produced evidence, with the number it
   produced (counts pasted, never hand-counted); every test you added, with
   proof it can fail (the mutation and its RED output).
4. "What I could not verify" and why — scoped precisely.
5. "Out of lens" notes (short).

Then return to the controller ONLY: the verdict line, the counts, and the
path. The report on disk is the artifact; the controller will not read your
transcript.
