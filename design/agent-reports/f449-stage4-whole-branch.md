# F-449 stage 4 — whole-branch adversarial review (post-implementation)

**Verdict: GREEN — 0 Critical / 0 Important / 0 Minor / 2 Nit.** I tried to build a counterexample for each question in the brief, and every attempt failed. Nothing blocks the landing.

Scope reviewed:
- fork `d2350cb..a66491f` (6 commits), in worktree `sh-worktrees/f449-stage4`;
- engrave `d4b730fc..deebbd0a` (4 commits), in worktree `me-worktrees/f449-stage4`.

Oracle: md-cli **0.19.0**, built fresh from descriptor-mnemonic `d269c556` into `/scratch/code/shibboleth/.tmp/wb-s4/dm-target`. The Liana oracle is the v15.0 gate-build harness (`me-worktrees/f449-stage4/harnesses/liana/target/gate-build/debug/liana-harness`).

All my probes ran in scratch clones under `/scratch/code/shibboleth/.tmp/wb-s4/`:
- `fork` is a66491f plus my `gui/wb_*_test.go` files and the copied r0/r1 harnesses;
- `base` is d2350cb;
- `mut` is a clean a66491f, used for mutations and the walk;
- `eng` is a scratch copy of `design/journeys`.

Both worktrees are clean. They are at `a66491f` and `deebbd0a`, with 0 porcelain lines each.

## Findings

### Critical
None.

### Important
None.

### Minor
None.

### Nit
- **N1 (records).** The committed host transcript names the wrong fork commit. `design/journeys/transcript_composer.txt:24` (engrave `62ad06ba`) records fork HEAD **`d64695c`**, but the walk it accompanies ran at **`a66491f`**. I re-ran `transcript_composer.sh` with FORK = a clean a66491f, and the content matches apart from that line and the `ls` timestamps. All 32 lines read `GATE PASS`, including `liana md1 string`, `liana Template-ID`, `NUMS twin Template-ID` and `md is 0.19.0`. There is no correctness impact. Re-running the transcript at the landing SHA would fix it.
- **N2 (gate mechanics, outside this diff).** `scripts/f449-stage4-mut.py` has two weak spots:
  - It decides CAUGHT from the return code alone and never checks that the plan's named test is among the failures.
  - Its vet check (`"ArtifactDir" not in b.stderr`) is always bypassed for `./gui/`, so a gui mutation that does not compile is caught only by the later `build failed` check.
  - Neither weakness produced a false PASS here. `.tmp/f449s4-impl/mut-full.txt` lists the failing tests on each row, and every CAUGHT row names the plan's test.

## What I verified sound (measured)

### Lens 1: addresses on the real branch

**Plan reviewers' differentials, re-run against a66491f and a fresh md 0.19.0** (`.tmp/wb-s4/diff1.txt`). All seven tests pass, with 0 MISMATCH lines.
- `TestR0LianaDeviceVsRust` (keyed route): 9/9 shapes derive. Each is checked at indices 0, 1, 7, 1000 and 2^31-1, on receive and change.
- `TestR1LianaTemplatePlusCards`: 13 encodable shapes. Cards in order, reversed and rotated all match. A wrong card in the last slot matches Rust's wrong-key wallet. A duplicated xpub is refused at consent.
- `TestR0DeviceVsLianaHarness`, `TestR0LianaTemplatePlusCardsDerives`, `TestR1InspectTemplateAlone` and `TestR0NoShippedTrVectorIsKeyPathUnknown` also pass.

**New: composer-built wallets on both routes** (`gui/wb_route_test.go`, 240 firing shapes, taken from the enumeration in Lens 3):
- The composer builds each wallet through `composerArtifactsFor`, seated with real xpubs, and `composerSelfCheck` passes on the keyed card.
- **Keyed route, chunks shuffled:** consent shows `Key path: Liana key` and receive 0. The device address equals `md address` run on the Go-encoded card, at indices 0 and 2^31-1, receive and change (960 addresses).
- **Template plus mk1 cards, reversed:** matches Rust at 10 points per shape.
- **Wrong card in the last slot:** derives Rust's address for the wrong wallet, never the original's.
- **A foreign chunk swapped into the keyed set:** refused.
- **NUMS twin:** its receive 0 always differs from the Liana wallet's.
- **Negative control:** mutating the Liana arm to ignore `change` produced **2,880 MISMATCH** lines. The harness can fail.

### Lens 2: regression, base d2350cb against a66491f
- **Screen dump** (`.tmp/wb-s4/dump-{base,fork}.txt`). The dump covers every md1 in `md/testdata/{vectors,forkbuilt,template}` (phrase and md1 files) and every composer preset under all 4 wrappers, both seated and unseated. For each it records:
  - the Wallet Policy consent, the composer consent, `policySummaryLines`, the classifier, `md1Summary` and addresses 0..2 (receive and change);
  - for presets, the template and keyed chunks, the stub lines and the self-check.

  I normalized only the three authorised copy changes:
  - F-633's "Liana (as of v15.0)";
  - §8f's last sentence;
  - the new inspect `Key path:` line (74 added, only the values `a key`, `NUMS` and `Liana key`).

  After that, the **only** difference is the two kind-1 vectors (28 diff lines):
  - At base they showed "This device can't derive addresses". Base's composer consent also mislabelled them `KEY PATH: NONE (NUMS)` and class "NUMS key path".
  - They now derive, show `KEY PATH: NONE (LIANA KEY)` and class "", and match Rust.

  Every one of these differences is authorised by SPEC §9's stage-4 row.
- **Every tr and wsh path list up to 3 paths** (`gui/wb_regress_test.go`), with keysets {1of1, 1of2, 2of2, 2of3}, locks {none, older 100, older 200, older 100u, after 800000} and hash {none, sha256}:
  - **131,280 compositions**, whose template chunks are **byte-identical** at base and branch;
  - their composer consent text is identical after the same normalization (0 differing lines).
- For wallets the previous firmware handled, no address, id or chunk moved. The only new refusal, `md1KeyPathUnknown` (`gui/wallet_policy.go:264`), fires only on a tr root with `KeyPathNone`. No decoder produces that today, so it cannot misfire.

### Lens 3: the composer
- **The predicate over the whole enumeration** (`gui/wb_enum_test.go`, 65,640 valid tr lists): it fires on **336**. For every one of them:
  - `composerCompose` succeeds and the self-check passes;
  - Go's kind-1 template chunks are **byte-equal** to `md compose --unspendable liana | md encode --force-chunked --group-size 0`;
  - Liana v15.0, over the descriptor with its internal key recomputed by `unspendable`, **accepts it and re-renders the identical descriptor** (imported as built): 336/336.
- **Six further accepted shapes up to the composer's limits** also import as built:
  - 8 paths (one 2of2 primary plus 7 recoveries);
  - `older=1` together with `older=65535`;
  - 9-of-9 primary and recovery;
  - a 1of9 primary;
  - recovery paths listed first;
  - the primary listed last.
- **Both controls are refused**: two unlocked paths, and a duplicated older.
- **The device half of F-644 holds.** I found no input sequence that composes a Liana-key wallet Liana refuses, or imports in a different form:
  - `st.unspendable` becomes Liana only through `composerUnspendableStep` when the predicate fires;
  - every write to `st.list` sits in the shape flow or the start step, before the step;
  - seating does not touch `st.list`.
- **Where the device declines but Liana accepts.** Liana accepts 72 shapes on which the device does not fire, all of class "a second unlocked path". Liana imports them, but reports the primary as, for example, a 2-of-5 when the script is 2-of-3 OR 1-of-2. The exclusion is therefore correct, not a false negative worth offering.
- **The screen works as the spec requires:**
  - **Placement** is at `composer_flow.go:101`, between `:96` and `:104`; the first stub flow is at `:135` (the spec's Status line is accurate).
  - The **default row** is seeded from `st.unspendable`, and the zero value is NUMS.
  - The **reset** is the predicate itself, with a modal naming the cause.
  - The **copy** matches `composerCopyTable` and SPEC_wallet_policy_composer §8y.
  - I looked at `l01-key-path.png` and `l04-consent-p1.png` from my own walk: both rows are drawn with NUMS highlighted, and the consent shows the full LIANA KEY body and Template-ID `f99cc42e…`.
- **Changed-id banner (Review Focus 1).** The Liana and NUMS template ids differ (`f99cc42e…` against `81072164…`), so `composerStubDelta` returns `composerStubIdMoved` at `gui/composer_stub.go:107`.
- **The §6 refusals never reach a read path.** In non-test code, `ValidateUnspendableShape` and `UnspendableRequestUnmet` are called only from `composerCompose` (`gui/composer_unspendable.go:163,170`). `validateUnspendableShape` has no caller in `encode.go` or `Reassemble`.

### Lens 4: false passes
- **Mutations**, each on a clean clone with the file touched after apply and after restore. Every clone was restored to 0 porcelain lines:

  | row | result |
  | --- | --- |
  | K1 | FAIL `TestTemplatePlusKeyCardsDerivesTheLianaWallet/keyed_tr_liana_kofn_recovery` |
  | L1 | FAIL `TestComposerKeyPathChoiceIsPlacedBeforeTheChunks` |
  | R1 | FAIL `TestComposerUnspendableResetIsThePredicate` |
  | N1 (new: Liana key derived at `change=false`) | FAIL `TestDeviceDerivesLianasOwnAddressesForKind1` (all 5 cases) and `TestEveryKeyedVectorReachesAnAddress` (both kind-1 vectors) |

- **The emulator walk, which I ran myself** at a clean a66491f, after my own host transcript (exit 0, 32 GATE PASS):
  - `capture_composer.py --arm both` exits 0 with "all legs matched the host". The liana leg (28 s, 11 shots) engraves `md13ls8a…` with Template-ID `f99cc42e…`.
  - The negative control `--arm keyed --prove-it-can-fail` reports **NEGATIVE CONTROL PASSED**.
  - **With L1 applied, the liana arm FAILS** with "the screen does not carry Template-ID: f99cc42e…" (the screen showed `81072164…`). The arm cannot pass on the wrong wallet: its re-entry press-through and its byte comparison both depend on the chosen kind.

### Lens 5: records
- **SPEC_liana Status lines** match the code:
  - `md/policy_shape.go:46`, `gui/policy_address.go:221` and `:243`, `gui/wallet_policy.go:264`, and `composer_flow.go` `:96/:101/:104/:135` all resolve as cited;
  - every test named in §8.9 and §9 exists, introduced in the commit cited.
- **F-654**: its closure is true. `encodePayload(d)` takes no version and derives it (`md/encode.go:425,456`), so leaving `validate_minimal_wire_version` unported is by design, as the closure says.
- **F-644**'s device-half ruling is confirmed and strengthened: see Lens 3 (336/336 plus 6, imported as built).
- **F-659** is accurate: `composerRestoreDoc` is at `gui/composer_flow.go:534`, and its `len(keyed)==0` branch still promises cards "listed below". The walk's keyless and liana legs both cut 1 plate.
- **F-660** is accurate: 28 s per arm, measured again by me.
- **Composer spec, §8f and §8x claims:**
  - the lengths 328 and 360 are correct (measured);
  - "289/289" traces to `DESIGN_coordinator_compatibility.md:683` and FOLLOWUPS F-633.
- **Core evidence**: the three receive addresses equal `liana_cases.json`'s `preset-kofn-recovery-tr` receive 0..2.
- `scripts/followups-status.sh` reports OK.

ready to ship: yes
