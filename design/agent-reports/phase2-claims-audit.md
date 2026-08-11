# Phase 2 mechanical claims audit — SeedHammer II encrypted-payload program

Read-only audit. No code, `design/FOLLOWUPS.md`, or working tree was modified except
inside a throwaway `git worktree` under `/tmp` (removed at the end). This file is
the only write.

Repos audited:
- Fork (Go): `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD `823499c`.
- Rust CLI: `/scratch/code/shibboleth/mnemonic-engrave`, branch `master`, HEAD `4d5ef3f`.
- `/scratch/code/shibboleth/mnemonic-toolkit` was consulted opportunistically for two
  cross-repo citations in FOLLOWUPS.md (HEAD `c14b1e2`); it is not one of the two
  repos in scope and was not otherwise audited.

Go toolchain used: `/home/bcg/.local/go/bin/go` (`go version go1.26.4 linux/amd64`) —
not on the default `PATH` in this environment; every command below was run with it
exported explicitly.

**Baseline reproduced exactly as given**, both checked by exit status, not by piping
into `tail`:
```
$ CGO_ENABLED=0 go test ./...   (fork)   -> exit 0
$ grep -c '^ok'   -> 48        $ grep -c FAIL -> 0
$ cargo test --all  (mnemonic-engrave) -> exit 0
  124+1+30+1+3+1+6+14+0 = 180 passed, 0 failed (9 test binaries)
```

---

## Audit 1 — Do the cited references resolve?

`scripts/plan-cite-gate.sh` was read first (per F-115 it resolves by basename,
first-only when unqualified, and now refuses ambiguous basenames — verified by
reading the script, not just its comment). It was run against all four documents,
then its output was **independently re-verified**: for every citation the printed
line's *content* was checked against the claim it's cited for, and a second,
broader Python sweep (`resolve_cites.py`, below) re-extracted every
`path.go:NNN`/`path.rs:NNN`-shaped string in FOLLOWUPS.md — including range
(`:2028-2039`) and comma/slash-list (`:49,182,286`, `:2687/2694`) forms the gate's
own regex does not match — to check for citations the gate silently never attempts.

### 1a. `design/SPEC_encrypted_payload_delivery.md`

```
$ bash scripts/plan-cite-gate.sh design/SPEC_encrypted_payload_delivery.md
== file:line citations ==   (15 citations)      all ok
== pkg.Symbol citations ==  (14 citations)      all ok
== RESULT ==   every citation resolves
```

The gate's "ok" only proves the line *exists*; it does not check the claim. Every
`ok` line was cross-checked against the text around its citation. Two decayed
citations, invisible to a resolve-only check, were found this way:

| Citation (as written) | Cited for | Gate says | Actually at that line | Correct location | Sev |
| --- | --- | --- | --- | --- | --- |
| `gui/gui.go:2932` (SPEC lines 1605, 1620, 1682, 2199 — all four instances, §10.2.4) | `idleTimeout`, the 3-minute constant §10.2.4 reuses | ok (line exists, has content) | `NextChunk() (draw.RGBA64Image, bool)` — an unrelated interface method inside `Platform` | `gui/gui.go:2955`: `const idleTimeout = 3 * time.Minute` (`grep -n idleTimeout gui/gui.go`) | **Important** |
| `crates/me-cli/src/main.rs:590` / `:597` (SPEC lines 1204–1205, §9) | `write_private` / its `0o600` mode-set | ok (line exists, has content) | `Ok(entries) => {` / `dir.display()` — inside an unrelated preview-directory-scan match arm | `fn write_private` at `:662`, `opts.mode(0o600)` at `:669` (`grep -n "fn write_private\|0o600" crates/me-cli/src/main.rs`) | **Important** |

Both are **false OKs** in exactly the shape F-115 exists to prevent ("the dangerous
half is a wrong `ok`") — the difference from F-115 is these are not
*wrong-repo-basename* resolutions, they are **ordinary post-fix decay**: the line
number was correct when written and the file grew above it.

For `main.rs:590`, this is provably F-98's own fix decaying:
```
$ git show 3be5fc8:crates/me-cli/src/main.rs | sed -n '588,598p'
... fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
```
`fn write_private` WAS at line 590 the moment F-98 closed (`3be5fc8`, 2026-08-10).
`git log --oneline 3be5fc8..HEAD -- crates/me-cli/src/main.rs` shows exactly one
commit since: `2ed6ac1` ("me seal: take records over a private channel, not argv
(F-102)"), which added a preview-directory scan above `write_private`, pushing it
from 590 to 662. F-98 has never been re-verified since.

For `gui/gui.go:2932`: `git log --oneline -S"gui.go:2932" -- design/SPEC_encrypted_payload_delivery.md`
shows the citation was introduced once, at `844bd35` (the spec's own R0-GREEN
commit), and never touched since. `idleTimeout` itself predates the encrypted-payload
feature entirely — it's the pre-existing screensaver timeout from the 2023 public
release (`git show 3398580:gui/gui.go | grep idleTimeout` → line 2635 then), reused
by §10.2.4 rather than introduced by it — so the citation was written against
whatever `gui.go` looked like at spec-drafting time and has drifted with every
edit above it since.

**One additional, already-known-and-flagged citation was re-confirmed still live**:
SPEC §10.1 (line 1279) cites `driver/otp/otp_rp2350.go:13` as "the cgo fixed-address
dereference" precedent for the blob-detection read. Line 13 is `#define
RT_FLAG_FUNC_ARM_SEC 0x0004`; the actual dereference (`*(uint16_t
*)(BOOTROM_TABLE_LOOKUP_OFFSET)`) is at line 31 (`sed -n '1,40p'
driver/otp/otp_rp2350.go`). This is not a new finding — it is *literally* the
motivating example in `plan-cite-gate.sh`'s own header comment and is named twice
in `FOLLOWUPS.md` (F-98, F-99) as a citation that "survived nine review rounds."
What's new here: **it is still there.** `git log -S"otp_rp2350.go:13" --
design/SPEC_encrypted_payload_delivery.md` shows one commit, `844bd35`, never
touched. F-98 fixed the spec's other two broken citations (`checksum.go:132`,
`main.rs:375`, see below) in the same pass but not this one, despite naming it.
Severity **Minor**, not Important: the claim it supports is explicitly
non-normative ("implementation-time question, settled by a test, not a design
question" — SPEC line 1281-1282), and the actual normative requirement two lines
above (read 8 bytes via XIP) is unaffected.

### 1b. `design/FOLLOWUPS.md`

```
$ bash scripts/plan-cite-gate.sh design/FOLLOWUPS.md
== file:line ==  109 checked, 3 FAIL
== pkg.Symbol == 15 checked, all ok
```

The 3 FAILs, checked individually:

| Citation | Gate verdict | Disposition |
| --- | --- | --- |
| `checksum.go:132` (lines 1373, 2208) | AMBIGUOUS: 2 files match (`bip380/checksum.go` 89 lines, `codex32/checksum.go` 170 lines) | **Not a live defect.** Both occurrences are inside F-98's and F-115's own historical write-ups, *quoting the old gate's output verbatim* to document the bug that motivated F-115's fix (repaired in `51ff889`, verified: `grep -n "F-115" scripts/plan-cite-gate.sh` — the AMBIGUOUS-not-first-match behavior is exactly what F-115 added). SETTLED. |
| `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:2473` | no such file | Citation is fully qualified into a **third repo** not in this audit's scope (neither the fork nor mnemonic-engrave has a `mnemonic-toolkit/` subtree, so the gate correctly reports "no such file" rather than mis-resolving). Opportunistically checked against the actual toolkit repo on disk: **decayed**. `self_check_bundle` was at :2473 when this FOLLOWUPS.md entry was written (2026-06-20, "HANDED OFF" to another instance); it is now at **:2486** (`grep -n "fn self_check_bundle" crates/mnemonic-toolkit/src/cmd/bundle.rs` in the toolkit repo, HEAD `c14b1e2`) — a 13-line drift. Minor, out of primary scope, already marked handed-off. |
| `verify_bundle.rs:645` | no such file | Same repo, same paragraph. `md1_match` was at :645; now at **:716/725/726** (`grep -n md1_match crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`) — 71-line drift. Same disposition as above. |

A third same-paragraph citation, `bundle.rs:2028-2039` ("mirror the in-file sibling
idiom"), uses a line-**range**, which `plan-cite-gate.sh`'s regex (`` `path:N` ``,
single trailing integer, nothing else inside the backticks) does not match at all
— it was never attempted by the gate, pass or fail. Checked by hand: lines
2028-2039 in the current toolkit `bundle.rs` are `--slot @{i}.phrase=` conflict
logic, not the `Zeroizing` idiom described — also decayed, exact current location
not pinned down (time-boxed; this is a third-repo, already-handed-off item).

**Gate coverage measured, not assumed.** A broader Python sweep
(`/tmp/.../resolve_cites.py`, discarded with the scratchpad) matched every
`path.(go|rs):N[-M][,N...]` shape in FOLLOWUPS.md regardless of backtick-wrapping:
**175 raw matches, 165 unique (path, line-set) pairs** vs. the gate's **107**
backtick-exact matches it actually evaluates. The 68-citation gap is entirely
range/comma/slash forms the gate's regex structurally cannot match — a real,
measured blind spot, distinct from F-115's basename problem. Running the sweep
found **no additional broken same-repo citations** beyond the ones above: of the
165, 157 resolve cleanly and the 8 problems are exactly the ones already discussed
(1 self-referential-settled `checksum.go`, 3 third-repo `mnemonic-toolkit` files
[`tree.rs`, `identity.rs`, `key_card.rs` — unrelated 2026-06-19/20 T6c/template
entries, never in the fork or mnemonic-engrave to begin with], 4 more
`mnemonic-toolkit` bundle/verify_bundle lines already covered above).

**Section-number (`§X.Y`) cross-references** — not covered by the gate at all, so
checked separately by diffing every `§N(.N)*` reference against the doc's own
`^#{1,4} N(.N)*` headings:
```
$ grep -oE '^#{1,4} [0-9]+(\.[0-9]+[a-z]?)*' design/SPEC_encrypted_payload_delivery.md | ...  -> 45 defined sections
$ grep -oE '§[0-9]+(\.[0-9]+[a-z]?)*' design/SPEC_encrypted_payload_delivery.md | ...          -> 39 unique internal refs, all 45-defined-superset
$ comm -23 referenced defined  -> (empty, after excluding "datasheet §N" / "NIST SP 800-132 §N" / "SP 800-38D §N" external-standard refs, confirmed by grep -c those 6 external hits separately)
```
All internal §-references in the SPEC resolve. All F-NNN cross-references in
FOLLOWUPS.md resolve 1:1 (`63` `### F-NNN` headings defined, `63` unique `F-NNN`
tokens referenced anywhere in the file — exact match, `comm -23` empty).

One **Nit**: FOLLOWUPS.md lines 33 and 39 say "spec §3.5.0" with no file named.
The adjacent line 76 names `SPEC_sizeproof.md` for a different §3.0 citation, which
invites reading §3.5.0 as the same doc — but `SPEC_sizeproof.md` has no §3.5
section at all (`grep -n '^#\{1,4\} 3\.5' design/SPEC_sizeproof.md` → empty).
§3.5.0 actually resolves in `design/SPEC_seedhammer_engrave_bip39_password.md:362`
(`### 3.5.0 Multi-run glyphs...`). Both entries are font/glyph work, unrelated to
the encrypted-payload program.

### 1c. `README.md`

No `file:line` or `pkg.Symbol` citations exist in this file at all (`grep -noE
'[a-zA-Z0-9_./-]+\.(go|rs):[0-9]+' README.md` → 0 hits) — it links only to design
docs. All 4 markdown links checked directly:
```
ok    design/CONSULT_b2b_idle_timer_design.md
ok    design/FOLLOWUPS.md
ok    design/SPEC_encrypted_payload_delivery.md
ok    design/SPEC_seedhammer_engrave.md
```

### 1d. `design/CONTINUITY_2026-08-11.md`

```
$ bash scripts/plan-cite-gate.sh design/CONTINUITY_2026-08-11.md
file:line: 2/2 ok      pkg.Symbol: 1/1 ok
```
Both file:line citations were content-verified, not just existence-verified:

| Citation | Claim | Verified against source |
| --- | --- | --- |
| `gui/run_flow.go:251` (F-103) | "refreshes `a.idle.start` on raw `len(evts) > 0`, with no requirement that an event resolve to *effective* input" | `sed -n '251p'` → `if len(evts) > 0 \|\| (ctx.keepAwake && !armed) { a.idle.start = now }` — matches exactly |
| `engrave/engrave.go:1664` (F-114) | "`SafePointer.Resume` synthesises its approach line from `bezier.Point{}`" | `sed -n '1664,1667p'` → `func (s *SafePointer) Resume(...) { move = appendLine(move, conf, false, bezier.Point{}, s.safePoint) ...}` — matches exactly |

All git-state claims in §1 of CONTINUITY (a table of SHAs/tags) independently
re-derived and matched:
```
d0c0a9d is ancestor of mnemonic-engrave HEAD 4d5ef3f (1 commit ahead: the continuity doc's own commit)
93ee004 is ancestor of fork HEAD 823499c (1 commit ahead: the kdfbench/sealread build-tag fix, F-98-adjacent, correctly noted as "already fixed" in the audit brief)
tag v0.5.0 -> d0c0a9d  (git log -1 v0.5.0)
tag fork-v0.0.0-g93ee004 -> 93ee004  (git log -1 fork-v0.0.0-g93ee004)
```
All referenced design docs (`HARDWARE_RESULT_2026-08-10c_b2b_gate.md`,
`TOOLPATH_EQUIVALENCE_2026-08-10.md`,
`agent-reports/encrypted-payload-spec-v2-R0-round1-fable.md`,
`CONTINUITY_2026-08-10.md`, `scripts/release-scan-firmware.sh`) exist.

### Audit 1 summary

| Severity | Count | Items |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 2 | `gui/gui.go:2932` ×4 occurrences (should be `:2955`); `crates/me-cli/src/main.rs:590`/`:597` ×1 occurrence (should be `:662`/`:669`) — both in NORMATIVE sections (§9, §10.2.4) of the GREEN spec |
| Minor | 3 | `driver/otp/otp_rp2350.go:13` still-live (non-normative text); 4 third-repo `mnemonic-toolkit` citations decayed (out of primary scope, already handed off); gate's regex silently skips range/comma/slash citation forms (68/175 in FOLLOWUPS.md never attempted) |
| Nit | 1 | Ambiguous unqualified "spec §3.5.0" cross-reference (font/glyph domain, not encrypted-payload) |

---

## Audit 2 — Do the tests actually assert?

Delegated to an independent sonnet fork (same context, disjoint scope, read-only)
scoped to: `seal/*_test.go` (15 files), `gui/wipe_guard_test.go`,
`gui/wipe_inventory_audit_test.go`, `gui/idle_late_arm_edge_test.go`,
`gui/idle_realclock_diag_test.go`, the §10.2.4-relevant subset of
`gui/run_flow_test.go`, `gui/unlock_wipe_test.go` (full), `gui/unlock_passphrase_wipe_test.go`
(full), the wipe-relevant subset of `gui/unlock_session_test.go`,
`seal/session_test.go` (wipe tests), and `crates/me-cli/src/seal/crypto.rs` (full).

**Result: 0 Critical / 0 Important / 0 Minor / 0 Nit.** No tautological,
unreachable, or non-discriminating assertion found in the scope covered. The
fork's methodology used positive controls throughout — e.g.
`TestWipeZeroesEveryPinnedBufferAtRunLevel` captures non-zero buffer state
*before* asserting post-wipe zero, so a wipe that never ran would fail the
pre-check rather than silently pass; `seal/session_test.go`'s
`TestWipeSecretAtZeroesExactlyOneRecord` asserts the target record zeroed **and**
every other record NOT zeroed, ruling out "wipes everything" passing as "wipes the
right one." Full detail (per-file rationale, grep sweeps run, exact quotes) is in
the fork's own transcript; the material fact for this report is the clean result
plus the explicit list of what it did **not** get to, reproduced below in "What we
could not check."

---

## Audit 3 — Hidden tests: files Go silently declines to compile

```
$ go list -f '{{.Dir}} {{.IgnoredGoFiles}} {{.TestGoFiles}}' ./...
```
Non-empty `IgnoredGoFiles` occur in exactly 7 packages (`cmd/emu`, `driver/ili9488`,
`driver/mjolnir2`, `driver/otp`, `driver/pio`, `driver/tmc2209`, `gui`, `seal`) —
every one of them is a **non-test** source file gated by a `//go:build tinygo`/
`rp2350`/similar tag (`otp_rp2350.go`, `read_tinygo.go`, `ili9488.go`, `pio.go`,
`uart.go`, `mjolnir2.go`, `debug.go`, `sealed_test_payload.go`\*, `toolpath_js.go`).
**Zero `_test.go` files appear in any `IgnoredGoFiles` list** — the exact class
that bit this project twice (`idle_late_arm_test.go`'s `_arm` GOARCH-suffix
accident) does not currently exist.

\* `sealed_test_payload.go` is not a `_test.go` file — it's a regular `.go` source
file whose name happens to contain "test" (it's the deliberately-shipped
`cmd/emu` sealed payload constant, SETTLED per the audit brief); confirmed by
checking it's absent from every package's `TestGoFiles` list and present only in
`IgnoredGoFiles`.

Further checks, each with a positive control:
```
GOOS/GOARCH-suffix-named test files (_arm/_js/_linux/_386/_wasm/...):
  find . -name "*_test.go" | check each basename's final _-segment against the GOOS/GOARCH list
  -> 0 hits

Orphan test directories (a _test.go file in a dir go list doesn't see as a package):
  comm -13 <package dirs> <test-file dirs>  -> empty
  positive control: injecting "./zzz_bogus_dir" into the test-dir list -> correctly reported
  -> 0 orphans

Explicit ignore/build tags inside _test.go files (//go:build ignore, etc.):
  grep -rn "^//go:build\|^// +build" --include="*_test.go" .  -> 0 hits

Test functions actually compiled vs. present in source:
  go test -list '.*' ./... | grep -c '^Test'          -> 1136
  grep -rhoE '^func Test[A-Za-z0-9_]*\(' --include="*_test.go" . | wc -l  -> 1136
  -> EXACT MATCH, no test function is silently excluded from any test binary

Package accounting: go list ./... -> 66 packages; 48 ok + 18 "no test files" = 66
  (matches the stated baseline exactly)
```

**Result: 0 hidden tests found.** Both historically-bitten classes (GOARCH-suffix
accident, missing build tag on a `cmd/` package) are absent from the current tree;
the second class is the one already fixed at `823499c` per the brief, and it does
not recur elsewhere (`grep`-verified no other `cmd/*` package lacks the pattern
`823499c` introduced — spot-checked `cmd/kdfbench`, `cmd/sealread`).

---

## Audit 4 — False-PASS hunting on the §10.2.4 idle-wipe tests

Performed in a throwaway `git worktree` at `/tmp/sh2-mutation-worktree` (detached
at `823499c`), never the checked-out fork. Removed at the end
(`git worktree remove --force`, confirmed `git worktree list` no longer shows it).

Baseline reproduced in the worktree before mutating:
```
$ go test -run '<16-test regex, below>' -v .
16 tests targeted: TestWipeGuardLifecycleAndArmed, TestIdleWindowIsNotDoubledByALateArmEdge,
TestCutEndingDuringTheParkStartsAFreshWindow, TestCutEndingAfterTheDeadlineStartsAFreshWindow,
TestIdleTimerUnderSH2ShapedEventLoop, TestRunWipeUnwindsAndRestartsTheFlow,
TestRunTwoWipesEachRestartCleanly, TestRunWarningThenWipe, TestRunWarningCountdownIsReal,
TestRunTapDuringWarningResetsAndReturnsContent, TestRunNotArmedNeverWarns,
TestRunPostCutWindowRestartsFromCutEnd, TestRunWarningBufferDoesNotGrow,
TestWipeWarningOpClampsNegativeRemaining, TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver,
TestRunKeepAwakeCannotPostponeAnArmedWipe, TestWipeZeroesEveryPinnedBufferAtRunLevel
-> 15 PASS, 1 SKIP (TestIdleTimerUnderSH2ShapedEventLoop, gated behind SH2_REALCLOCK=1,
   ~3.5 min real-wall-time cost — informational, matches audit-2 fork's independent note)
ok seedhammer.com/gui 7.807s
```

Each mutation: applied by line-anchored `sed` to a unique match (verified `grep -c`
= 1 before mutating), diff-verified against a copy of the original (`md5sum`, then
`diff` — never `git checkout`, per SPEC §11.3's procedural rule), tests run, then
**restored from the file copy** and re-verified byte-identical (`diff` clean)
before the next mutation. Final state confirmed clean (`git status --short` empty,
full target-test run green again) before the worktree was removed.

| # | Mutation | File:line | KILLED / SURVIVED | Killed by |
| --- | --- | --- | --- | --- |
| M1 | Invert `if armed {` → `if !armed {` (the warn/wipe gate) | `run_flow.go:282` | **KILLED** | 11 of 16 tests fail, incl. `TestRunNotArmedNeverWarns` ("a warning was drawn while unarmed") |
| M2 | `wipeGuard.armed()` hardcoded to `return false` (guard never arms — "delete a wipeGuard call"'s effect) | `wipe_guard.go:49` | **KILLED** | 11 of 16, incl. `TestWipeGuardLifecycleAndArmed` itself |
| M3 | Deadline made effectively infinite: `idleWakeup := a.idle.start.Add(24*time.Hour)` instead of `idleTimeout` | `run_flow.go:269` | **KILLED** | `TestIdleWindowIsNotDoubledByALateArmEdge`, `TestCutEndingDuringTheParkStartsAFreshWindow`, `TestCutEndingAfterTheDeadlineStartsAFreshWindow` — each reports "warning appeared 24h0m0s..., want ~3m0s" |
| M4 | `syncArmed` made a no-op (`return a.armed` before any of its body runs — ignores `ctx.wipe.armed()` and never stamps `a.idle.start`) | `run_flow.go:95-96` | **KILLED** | 9 of 16 |
| M5 | Delete the pre-block `syncArmed(time.Now())` call — the exact mutation `idle_late_arm_edge_test.go:31`'s doc comment names | `run_flow.go:219` | **KILLED** | `TestIdleWindowIsNotDoubledByALateArmEdge` (the exact test the comment names) + `TestCutEndingDuringTheParkStartsAFreshWindow` |
| M6 | Post-block `armed := syncArmed(now)` → `armed := ctx.wipe.armed()` — the exact mutation `idle_late_arm_edge_test.go:213`'s doc comment names | `run_flow.go:241` | **KILLED** | `TestCutEndingAfterTheDeadlineStartsAFreshWindow` (the exact test the comment names) — "warning appeared 0s after the cut finished, want ~3m0s" |
| M7 | Remove the `if armed != a.armed` idempotency guard (`if true {`) — the code comment claims "fails 12 tests across ./gui/" | `run_flow.go:97` | **KILLED** | 9 of 16 in the targeted set (comment's "12" is over the whole `./gui/` package, a superset of this 16-test regex; not re-measured against the full package) |
| M8 | Delete the wipe-guard installation `ctx.wipe = g` in `unlockSecretSession` | `unlock_session.go:89` | **KILLED (build failure)** | `./unlock_session.go:88:2: declared and not used: g` — per SPEC §11.3's own procedural rule ("a mutant that fails to build is KILLED, not an error"), classified as killed |
| M9 | Disable the actual wipe-firing check `if now.Sub(wipeAt) >= 0` → `if false` (warning still draws, wipe itself never fires) | `run_flow.go:284` | **KILLED (partial)** | 3 explicit FAILs observed (`TestIdleWindowIsNotDoubledByALateArmEdge`, `TestCutEndingDuringTheParkStartsAFreshWindow`, `TestCutEndingAfterTheDeadlineStartsAFreshWindow`) before a later test in the same `-run` batch hung past my own 2-minute harness timeout (consistent with a test that blocks waiting for a wipe that can no longer fire — not re-run to conserve time; mutation was restored regardless and confirmed clean) |

**Result: 9/9 mutations KILLED, 0 SURVIVED — no false-PASS found in this battery.**
This is a clean result, not a null result: every mutation was confirmed to apply
(diff before test) and confirmed restored (diff after). It differs from this
project's prior track record of finding false-passing tests via mutation (5 found
elsewhere, historically) — the §10.2.4 mechanism has already been through at least
one R0 round of exactly this kind of mutation table (SPEC §11.3, two rows marked
"survived round 1" and since fixed), so a clean result on a second pass through
its core branches (arm/disarm gate, deadline computation, the two `syncArmed`
call sites, the idempotency guard, guard installation, and the wipe-firing check
itself) is consistent with that history rather than surprising.

---

## What could not be checked, and why

- **M9's full collateral run.** Only 3 of 16 targeted tests were confirmed FAIL
  before the run exceeded my 2-minute harness timeout; the mutation was restored
  and the tree re-verified green, but the remaining 13 tests' individual verdicts
  under M9 were not captured. The 3 that did fail are sufficient to call the
  mutant KILLED; a full accounting of which others would also fail (or hang) was
  not obtained.
- **Whole-`./gui/` collateral damage per mutation.** Every mutation was run against
  the 16-test filtered regex (per SPEC §11.3's own procedural convention — "every
  per-row run is FILTERED... a filtered run proves that test fails, and nothing
  about any other test"), not the full package, except for the clean-baseline and
  clean-restore checks which did run the filtered set at each boundary. Whether
  any mutation also breaks unrelated `./gui/` tests outside the wipe-test set was
  not measured.
- **Exhaustive line-by-line mutation of `wipe_guard.go`/`run_flow.go`/
  `unlock_session.go`/`unlock_kdf.go`.** 9 mutations were chosen to cover the
  example classes given (invert a condition, delete a wipeGuard call, infinite
  deadline, no-op `syncArmed`) plus the two mutations the code's own comments
  name as pinned, plus the idempotency guard and the wipe-firing check itself.
  This is not exhaustive coverage of every branch in the mechanism (e.g. row 4's
  `unlockPassphraseFlow` bracket in `unlock_kdf.go:136-138` was not separately
  mutated; `wipeNowHook`'s test-only path was not mutated since it's nil in
  production and not itself the production mechanism).
- **Audit 2's unread files** (from the fork's own report, not re-verified by me):
  `seal/container_test.go`, `engraveable_test.go`, `grouping_test.go`,
  `label_encrypted_test.go`, `pbkdf2_test.go`, `pubhash_test.go`, `record_test.go`,
  `unlock_key_test.go`, `vectors_test.go`, `wire_test.go`, `boundblob_test.go`,
  `open_test.go`; `gui/unlock_flow_test.go`, `unlock_platelist_test.go`,
  `unlock_cancel_test.go`, `unlock_engraveable_test.go`, `unlock_kdf_test.go`,
  `unlock_plates_test.go`, `unlock_program_test.go`, `unlock_session_scrub_test.go`,
  `run_flow_scrub_test.go`; `crates/me-cli/src/seal/{container,mod,passphrase,
  pubhash,record,uf2,wire}.rs`. The fork's report is explicit that this is
  unaudited territory, not "probably fine" — restated here so it isn't lost.
- **The third repo, `mnemonic-toolkit`, was not audited.** Two FOLLOWUPS.md
  citations into it were opportunistically spot-checked (found decayed, Minor,
  already out-of-scope/handed-off per that entry's own text) but no systematic
  citation, test-assertion, or hidden-test sweep was run there — it is outside
  the two repos this audit was scoped to.
- **Hardware behavior.** This is a software-only mechanical audit; nothing here
  confirms or contradicts any of the hardware-measured facts the SPEC and
  CONTINUITY docs cite (F-106 timing, F-100's pager-dot count, etc.) — those were
  only checked for whether their *citations* resolve, not re-measured.
- **Whether `otp_rp2350.go:31`'s dereference is itself correct C/cgo** — only
  checked that it is the line the SPEC's precedent claim should have pointed to,
  not whether the code at that line is itself sound (out of scope; SPEC itself
  calls this an implementation-time, test-settled question, not a design one).
