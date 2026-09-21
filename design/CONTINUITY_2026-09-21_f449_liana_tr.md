# CONTINUITY — F-449, the Liana unspendable internal key (2026-09-21)

Resume anchor. Operator is on a transatlantic flight and asked for autonomous
work "through deploy to /sh2 and ship". Everything below is on git (durable).

## What "deploy to /sh2" means

**The website, not the board.** https://quantoshi.xyz/SH2/ — the WASM emulator
demo at `demo/sh2/`, deployed by `demo/sh2/deploy.sh --host user@quantoshi` to
`/opt/quantoshi/sh2/`. The operator clarified this mid-session. The SH2 *board*
flash is NOT in scope and is never unattended anyway.

## THE ONE BLOCKER FOR THE OPERATOR

**The quantoshi deploy is credential-blocked.** Measured:
- `ssh quantoshi` → `Could not resolve hostname`
- `ssh bcg@quantoshi.xyz` → `Permission denied (publickey)`
- `SSH_AUTH_SOCK` unset, no agent
- the repo only ever records the literal placeholder `user@quantoshi`
- the site itself is live (`curl -o /dev/null -w %{http_code}` → **200**)

Everything upstream of the deploy can be finished without the operator. The
final `deploy.sh` run needs the real `user@host` or a key. `demo/sh2/build.sh`
is green at baseline and produces `dist/` with an 11 MB `emu.wasm`.

## Where the cycle is

Brainstorm → **spec r4** → closing review dispatched. **No implementation yet.**
The R0 rule binds: NO code before the spec is GREEN (0C/0I).

### Review history — every lens found what the previous one could not

| round | lens | verdict |
| --- | --- | --- |
| r0 | opus, correctness | **2C**/6I/5M/2N |
| r0 | fable, adversarial funds | 0C/**4I**/9M |
| r0 | author self-review | 2I/1M |
| r1 | sonnet, mechanical fold-check | 1 unaddressed / 2 new defects |
| r1 | opus, new design only | **1C**/4I/2M |
| r2 | sonnet, mechanical fold-check | 0 unaddressed / 1 cosmetic |
| r2 | **opus, JOURNEY WALK** | **2C**/6I/4M |
| r3 | sonnet, mechanical fold-check | 4 partial / 0 new defects |
| r3 | opus, new design only | **0C**/6I/4M |

**The three highest-value findings, for anyone tempted to shorten this:**

1. **Version 5 was unimplementable** (r0, found independently by two lenses).
   Single-payload md1 versions must be EVEN. A v5 plate routes to the chunk
   reassembler and the device refuses the plate it just cut.
2. **The device cannot derive a kind-1 address** (r1 new-design). A naive port
   maps kind 1 onto the NUMS branch and shows addresses of a DIFFERENT WALLET
   at consent and at plate verify.
3. **The journey walk's two Criticals, both the author's own folds** (r2), after
   four correctness-shaped lenses had closed clean: a choice-screen predicate
   that was measurably INVERTED, and an identity ruling written for Rust that
   left the Go half — the mk1 card stub the operator reads — ungated.

Pattern worth keeping: **three times** this spec stated a ruling in prose and
gated none of it (§8.8 REQUIRED and scheduled nowhere; §8.5 Rust-only; §0b/§6a
ungated). §8.9 now exists to make that structural rather than remembered. And
**four reviewer claims were corrected rather than propagated** — five md-cli
sites were four, `template_id.go:116` was the else branch, the placement window
was 96-128 not 97-118, and a Go blast-radius figure of 58 was the author's own
error (136 across 32 files).

| artifact | commit |
| --- | --- |
| spec r0 (draft) | `72445995` |
| self findings (4) | `b4232ac9`, `3f427826`, `af8aea68` |
| opus R0 report (2C/6I/5M/2N) | `8bfe5a86` |
| fable adversarial report (0C/4I/9M) | `1fdb299f` |
| **spec r1 fold** | `946fb24d` |

Baselines: descriptor-mnemonic `6cbd49d8`, seedhammer `7b6f2fb`,
mnemonic-engrave at the commits above. **Suite at baseline: 1400 passed /
3 skipped** (the 3 skips are deliberate: release-only BCH sweep, bitcoind
differential, daily address job).

## The decisions that are SETTLED — do not re-derive

1. **Scope.** The tr menu is 6 presets. 1 works today
   (`simple-timelocked-inheritance`). 2 need only F-449 (`kofn-recovery`,
   `tiered-recovery`) — both MEASURED ACCEPT once the internal key is Liana's
   xpub. 3 are outside Liana's model **by shape** and no encoding change reaches
   them (`plain-multisig` class 3, `hashlock-gated` class 4,
   `decaying-multisig` classes 5+7).
2. **Sequencing, operator's choice:** F-449 first, target-selection mode second.
3. **The recipe is VERIFIED**, reproduced from scratch byte-for-byte against all
   four Liana-accepted golden xpubs. sha256 over each leaf xpub's 33-byte
   pubkey, descriptor left-to-right, depth 0 / parent 0 / child 0.
4. **The wire version is 8, NOT 5.** Single-payload versions must be EVEN — the
   auto-dispatch reads bit 0 of the first symbol as the chunked flag. Usable set
   is `{4, 8, 12}`; this spends 8 and leaves 12 as the last generation.
5. **There is no cheaper seam.** All four alternatives tested and rejected:
   trailing bit desyncs, unknown TLVs are preserve-and-skip (`tlv.rs:273-274`,
   RUN), the `key_index` sentinel does not fit (`ceil(log2(n))`), so the header
   version bump is the only fail-closed option.
6. **Identity hashes must derive the version from the tree**, never a constant —
   `write_node`'s two non-wire callers are `WalletDescriptorTemplateId` and
   `WalletPolicyId`, and a constant v4 would give kind 0 and kind 1 one
   identity phrase while their addresses differ.

## Evidence that already exists — do not re-measure

- `design/evidence/composer-fable-r0/fable-liana-parse-in.jsonl`, variant
  `liana-unspendable-xpub`: **8 golden descriptors**, each carrying the Liana
  unspendable xpub. 4 ACCEPT (kofn-recovery, tiered-recovery,
  same-seed-two-paths, X19), 4 REFUSE (3 hash shapes + decaying).
- `…-parse-out-v15.jsonl` records **Liana's own receive and change addresses**,
  indices 0..2, per accepted shape — so the acceptance test is a real
  cross-implementation address check without building Liana.
- v8.0 and v15.0 agree on every verdict.

## Next steps, in order

1. Read the two re-review reports when they land
   (`f449-spec-r1-fold-check.md`, `f449-spec-r1-new-design.md`), persist each in
   its own commit, fold, re-gate until **0C/0I**.
2. Only then: `writing-plans` for stage 1a (behaviour-preserving `InternalKey`
   refactor) and 1b (the version-8 kind bit).
3. SDD execution, one implementer, UC off for implementation.
4. Stages 2-5 per the spec's §9 table.
5. Build `demo/sh2/dist/` and stop at the deploy — hand the operator the
   credential gap.

## Standing constraints that bind here

- Persist and fold are **two commits**, never one. The agent writes its own
  report; the controller never transcribes it.
- Push only via `scripts/push-via-staging.sh` with main/master FROZEN for the
  window. A "Bypassed rule violations" message means the staging step was missed.
- Rust-primary: `md-codec` leads, the fork's Go `md/` follows. Never the reverse.
- A bumped `md-codec` version and md-cli's `version = "=0.45.1"` exact pin
  (`crates/md-cli/Cargo.toml:28`) must move in the SAME commit or cargo cannot
  resolve.

## Gates proven EXECUTABLE (not hypotheses) — 2026-09-21

r1 promoted two acceptance items from aspiration to REQUIRED. Both were then
proven to run, because this project treats a gate that has never executed as a
hypothesis rather than a gate.

**§8.8 — the live Liana install run.** The committed harness was NOT buildable:
`harnesses/liana` was neither a workspace member nor excluded and had no
`[workspace]` table. Fixed at `bca2ff36` using the pattern the repo already uses
for `crates/me-cli/fuzz`. After the fix, run over the 8 `liana-unspendable-xpub`
evidence descriptors:

    4 ACCEPT / 4 REFUSE   (matches the recorded evidence exactly)
    preset-kofn-recovery-tr recv[0] = bc1pj6davmeetfe2uutrjlgxcjytq5… (golden match)

The Liana v15.0 checkout is present at `.tmp/fable-liana-src-v15`; the harness
`Cargo.toml` already points at it. Build with
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/liana-harness-target cargo build`.
**Take cargo's exit code directly — piping through `tail` reported exit 0 on a
build that had failed.**

**§8.3 — the device address leg.** Achievable: `gui/policy_address_test.go` and
`gui/policy_address_export_test.go` already exercise the real device function
(`TestPolicyAddressAtIsTheDeviceFunction` exists precisely to pin that the test
does not wrap a different function). Green at baseline, 4 top-level tests,
58 RUN/PASS lines, 0.14s when scoped with `-run` — and the `-run` filter was
verified with `-v` to have actually matched, since a filter matching nothing
also prints `ok`.
