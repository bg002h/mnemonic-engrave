**⚠ 3 Important (#2, #4, L5), plus 1 Important against the proposed fix-03 patch (it removes passphrase validation and fails 3 existing tests).**

# Grade and rulings: mnemonic-engrave bug hunt 2026-10-31

The grader is the Opus architect. Inputs: triage `design/agent-reports/bughunt-triage-me.md` (2123d725), hunt
`design/agent-reports/bughunt-2026-10-31-me.md`, RED tests 602ddba1
(`crates/me-cli/tests/bughunt.rs`) and the fixes `/scratch/cache/bughunt/me/fix-01..05.patch`.
Settled facts, not re-derived:
- Tests 01-05 fail on unmodified code, so #1, #2, #3, #4 and L5 are CONFIRMED.
- L6 and L7 are FALSE POSITIVE.

The likelihood test is applied as the brief quotes it. A finding is Minor only if all three hold:
(1) it fails closed; (2) no printed command fails as printed; (3) it needs two or more listed unusual conditions.
A wrong result, data loss, a check that silently does not run, a panic on holder input or a silently ignored input stays C/I.

## Grades

| item | grade | triage proposal |
|---|---|---|
| #4 `sysw pack --in` drops argv records | **I** | I (agree) |
| #2 stale `plate-N` passes the F9 check | **I** | M (raised) |
| L5 `seal::uf2::to_uf2` has no size bound | **I** | I (agree) |
| #1 `write_private` empties the old file when chmod fails | **M** | I (lowered) |
| #3 `sysw::pack` seals when there are no secrets | **M** | I (lowered) |
| fix-03 patch as proposed | **I** (on the patch, not on production code) | n/a |
| L7 unreachable underflow | Nit | n/a |

**#4: I.** A silently ignored input.
- What happens: `me sysw pack --no-passphrase --in recs.txt --out p.bin text:6869` exits 0. The argv record is dropped from the payload.
- Cause: in `read_records` (main.rs:3039), the `if let Some(p) = in_path` branch returns early and never looks at argv.
- Why not M: no condition makes this unusual. Both forms are in the synopsis (SPEC_systemwide_payloads.md:895).

**#2: I.** A check that silently does not run, reachable from the library.
- What happens: `preview::render_plate` (preview.rs:136) judges `plate-{idx}.{ext}` after the sidecar exits. If the sidecar writes nothing and a stale file sits at that path, the stale render passes F9 as this run's output.
- CLI reach: the A1 entry refusal (main.rs:1195-1207) rules out a stale file at entry, so the CLI reaches this only through a concurrent writer.
- Library reach: any caller of the public `render_plate` reaches it directly. Test 02 does exactly that.
- Why not M: "a check that silently does not run" stays C/I however rare.

**L5: I.** A wrong result through the public library.
- What happens: `seal::uf2::to_uf2` (public through lib.rs `pub mod seal`) writes block i at TARGET_ADDR + i*256 with no bound. Any blob over 64 KiB lands outside the seal region (SPEC_encrypted_payload_delivery.md:466-483, base 0x10E00000, 64 KiB). Over 2 MiB it addresses past 0x11000000, where the write wraps and destroys the firmware (same spec, line 1197).
- Why the CLI cannot reach it: the only caller (main.rs:1022) passes a parsed seal blob. Seal blobs are at most 16450 bytes (MAX_SECTION_LEN 8191, wire.rs:21), and the wire parse requires total ≤ REGION_LEN (wire.rs:158-165).
- Why I: library-only reach counts, and the result is a wrong address.

**#1: M.** All three conditions of the likelihood test hold:
1. It fails closed. The holder sees rc 2 and `cannot write X: Operation not permitted`. No plate, card, key or address is wrong. The only bytes lost are the ones the holder ordered replaced, and every success path destroys those too.
2. No printed command fails as printed.
3. Three unusual conditions, all required:
   - (a) `--out` names an existing file owned by another uid;
   - (b) that file grants the holder write access through its group or other bits (`me` itself only ever creates 0600 files);
   - (c) the filesystem enforces chmod ownership (vfat and exfat do not).

There is a contrary reading: a holder may expect a failed write to leave the old file intact. That reading is why I still recommend folding fix-01. It is a three-line change. As M it would otherwise be filed `deferred-minor`.

**#3: M.** A close call between M and I.
- What happens: `sysw::pack(records, Some(pass), iters)` with no secret records emits a sealed container (an encrypted, empty secret section). The CLI's decide_sealing rule emits the unsealed form instead.
- Every reader decodes the same records from both blobs. Probes `/scratch/cache/bughunt/me/grade/b77.bin` and `b61.bin` give identical `show` output; only the container identity differs. No data is wrong and nothing is lost.
- The passphrase is not silently ignored: it is used.
- Unusual conditions: (a) a library caller, though the only callers today are tests; (b) a passphrase supplied with zero secret records.
- Recommendation: fold the corrected fix (below), because the cost is one `if`.

**fix-03 as proposed: I (on the patch).** It inserts `let passphrase = if payload.secret.is_empty() { None } else { passphrase };` before validation. So with no secrets the library would silently accept an empty, too-long or not-enterable passphrase.
- Verified: with fix-03 applied, `sysw_cli` has 32 tests run, 29 passed, 3 failed. The failures are the library passphrase-refusal tests at tests/sysw_cli.rs:605-665, which call `sysw::pack(vec![TEXT.into()], Some(p), 100_000)` with no secrets. The run is in `/scratch/cache/bughunt/me/grade/fix03-run.txt`.
- The FIX doc must use the corrected placement under ruling R3.

**L7: Nit.** The underflow the hunt named cannot be reached; the triage's planted fault shows the guard holds. One line in FOLLOWUPS as `deferred-minor`, if the controller wants it.

## Rulings

**R1 (#4): refuse.** Exact behaviour:
- When `--in` is given and argv holds records, print `me: pass records on argv OR via --in, not both` to stderr and exit 2 (EXIT_USAGE, main.rs:480-483). Write nothing.

Why: seal already refuses this case with exactly this text and exit code (main.rs:936-939). Merging would add an order rule (argv first, or file first?) that the spec would have to decide, and it would weaken the reason `--in` exists: line 908 calls argv "a public channel". Refusing is the simpler fix and matches the seal refusal style the brief asks for.

Changes:
- fix-04 as written: an `Err(("pass records on argv OR via --in, not both", EXIT_USAGE))` at the top of the `--in` branch. The caller at main.rs:1511 adds the `me: ` prefix.
- Spec: append to the `--in FILE` row at SPEC_systemwide_payloads.md:908: "Records on argv together with `--in` are refused: `me: pass records on argv OR via --in, not both`, exit 2."
- Toolkit manual: no row changes. origin/master has no `me` chapter in `40-cli-reference/`, and the only `--in` uses (30-workflows/3B-payload-unlock-refusals.md:47,51) pass no argv records.
- Fold needed: no.

**R2 (#2): remove the stale file before spawning.**
- Change: `render_plate` removes `plate-{idx}.{ext}` before spawning `me-preview`. NotFound is ignored. Any other error returns `PreviewError::Spawn(e)`, which displays as `cannot run me-preview: <io error>`.
- No new variant and no new text.

Why: this is the smallest change that makes F9 judge only this run's output. Re-checking per plate (a timestamp or inode comparison) adds state and its own races.

It deletes no user file the A1 guarantee protects:
- In the CLI, the A1 entry scan (main.rs:1195-1207) has already refused any directory that holds `plate-*`, so the remove finds nothing unless a concurrent writer put a file there.
- The sidecar would overwrite that path anyway.

Changes:
- fix-02 as written.
- Spec: add to the A1/F9 text in SPEC_me_preview_hardening.md (lines 33-44): "`render_plate` removes any existing `plate-<idx>.<ext>` before it spawns `me-preview`, so the F9 check judges only this run's output; a removal error other than not-found is reported as `cannot run me-preview: <error>`." This keeps A1's "without silently deleting user files" true for the CLI: entry refusal still comes first.
- Manual: none. Fold needed: no.

**R3 (#3): emit the unsealed form after validation, not before.**
- In `pack_deterministic_with` (sysw/mod.rs:458-514), all passphrase validation stays where it is: EmptyPassphrase, PassphraseTooLong, NotEnterableOnDevice and the iterations check.
- Only the final `seal_with(...)` gains a guard: when `payload.secret.is_empty()`, return `bound(header.encode() ++ pub_bytes)`, the same unsealed container the `None` path builds. Otherwise call `seal_with` as now.

Why:
- It matches the CLI's decide_sealing rule.
- A bad passphrase is still refused, so test 03 passes and the 3 existing refusal tests stay green.
- Verified as `/scratch/cache/bughunt/me/grade/fix-03b.patch`: 116 sysw tests plus test 03, 116 passed (`fix03b-sysw.txt`).

Changes:
- Use fix-03b, not fix-03.
- Spec: none. SPEC_systemwide_payloads.md:151 already says "ct_len == 0 means no encryption, no tag"; this makes the library obey the CLI rule the spec states.
- Fold needed: no.

**R4 (L5): cap at `seal::wire::REGION_LEN` (65536) with an `assert!`, not a Result.**
- Exact text: `assert!(blob.len() as u64 <= REGION_LEN, "seal blob {} bytes exceeds the {REGION_LEN}-byte seal region", blob.len())`.

Why 64 KiB rather than 2 MiB:
- Spec §5 (SPEC_encrypted_payload_delivery.md:466-483) makes the region 64 KiB, sharing no sector, and the wire parse already holds every blob to it (wire.rs:158-165). A 2 MiB cap would still let a write leave the region and overwrite whatever lies between it and the end of flash.

Why assert rather than Result (a tie, broken by the smaller change):
- The one caller passes a parsed blob, which can never exceed the cap, so a Result would add a signature change and an error variant for an unreachable branch.
- The brief's "panic on holder input" rule does not apply: no holder input reaches this cap.
- Test 05 (bughunt.rs:147) accepts a panic, and its 3 MiB blob trips this assert.

Changes:
- fix-05 with its constant changed to REGION_LEN.
- Spec: append to §5: "`to_uf2` refuses (panics on) a blob longer than the 64 KiB region; the CLI never passes one."
- Manual: none. Fold needed: no.

**R5 (#1, fold recommended though M): set the mode before truncating.**
- `write_private` (mnemonic-io-lib/src/write.rs ~45-62) opens with write+create, mode 0o600 and no `truncate`. It then calls `set_permissions(0o600)`, then `set_len(0)`, then `write_all`.

Why:
- A chmod EPERM now leaves the old file intact.
- On the success path the bytes are identical to today's.

Changes:
- Spec SPEC_me_preview_hardening.md:104-119. Replace "**`truncate(true)` is load-bearing (R0 I2)**" with: "Truncation is load-bearing (R0 I2) and happens after the mode is set (`set_len(0)` after `set_permissions`), so a refused chmod leaves the existing file unchanged."
- Correct "0o600 binds on CREATE" to cite F-244's explicit `set_permissions`.
- Manual: none. Fold needed: no.

## FIX documents

**One FIX doc, small-fix track:** `FIX-bughunt-2026-10-31-me.md`. It covers #4, #2, L5 and #3 (corrected), with #1 folded as a recommended M.

Why it qualifies:
- One repo (mnemonic-engrave, including mnemonic-io-lib in the same workspace).
- No new flag, type, command or error variant.
- No manual change.
- Five spec sentences across three specs, all given above as exact texts.

RED rows:
- `bughunt_me_01` through `bughunt_me_05` (602ddba1).

Must-stay-green rows:
- The three library passphrase-refusal tests at tests/sysw_cli.rs:605-665.
- The F8 entry-refusal tests.

The doc must name fix-03b, not fix-03, and give R4's constant change to fix-05.

Open question for the coordinator (not decided here): whether the M items #1 and #3 ride in this FIX doc as recommended or go to FOLLOWUPS `deferred-minor`. The grades do not depend on it.

Scratch state: `git worktree` `/scratch/cache/bughunt/me/grade/wt3` (fix-03b applied, uncommitted) and `target3/` remain under `/scratch/cache/bughunt/me/grade`. Nothing in the repo was changed apart from this report.

Counts: Critical 0, Important 3 (+1 against the proposed fix-03 patch), Minor 2, Nit 1.
