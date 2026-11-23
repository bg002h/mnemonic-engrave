# FIX sec-me-kitplan-wipe-hook: the kit plan's wipe rows fail when the hook never fires

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: small (plan text only; no flag, type, command or
holder-visible text). Class: other (not a funds path). Grade I.

Row closed: `plankits2-r1-s1-wipe-hook-vacuous` (coordinator `status/secret-criticals.md`), from
`review-plan-kit-S2-r1.md` S-1 (coordinator `agent-reports/`, lines 149-156).

## Where the text lives

The defect is in `design/IMPLEMENTATION_PLAN_fork_refugium_kit.md`, which exists only on branch
`plan/fork-refugium-kit` (head `1c079d95`); `master` has no kit plan. As the brief directs, this branch starts at
origin `master` (`16356ac1`) and carries only this document and the patch `design/FIX_sec-me-kitplan-wipe-hook.diff`.
The patch is made against `1c079d95` and applies there (`git apply --check` run, OK). It can be applied as its own
commit on the plan branch or folded into the next plan fold (S2 r4). The texts are the same either way; the
coordinator picks the delivery route (recon open question).

## The defect (anchors at 1c079d95)

- `TestMS1LetterIsTheTopNibbleOfTheEntropy` (plan :355-357) Rule: "the entropy slice handed to the §7 hook is all
  zero when `ms1Letter` returns".
- `TestRefugiumSealedMS1WipePoints` (plan :437-440) Rule: "the entropy slice and the `bip39.Mnemonic` handed to the
  §7 hooks are all zero before the engrave screen's job starts".
- §7 "Wipes" (plan :530-532): "tests keep the slice and assert it zero afterwards".

Neither Rule counts hook calls. A build that never calls `ms1LetterEntropyHook`, `unlockSeedQREntropyHook` or
`unlockSeedQRWordsHook` leaves the test's captured slice nil, and every byte of a nil slice is zero, so both rows
pass with no wipe checked. A second gap, not named by S-1: the S1b vectors `A16` (16 x `0x00`) and `A` (32 x `0x00`)
have all-zero entropy while live, and `A16`'s mnemonic is "abandon" x 11, "about" (`bip39.Word` 0 is "abandon"), so
the zero-after check on those vectors is vacuous even when the hook fires. The fix requires a call count, a live
copy taken inside the hook, and names the vectors that carry the proof (`B` ... `G`, `P`).

Review r1 (Opus 5.5) added two more vacuous paths, folded here: (I-1) the refusal path of `TestRefugiumSealedMS1WipePoints`
ran only `A16` (all-zero entropy), so a build that skips `clear(entropy)` on the SeedQR refusal passed; the row now also
runs `P` with its padding bit set (`ms10entrsqrlll...ak6svw5c6lqsf`, 32 x `0xff`, refused). (I-2) the row's Rust-mirrored
`0x02` payload is all-zero, so an early return on a non-`0x00` prefix before `clear` passed; the row now also runs a
`0x02` payload of 16 x `0x11` (`ms10entrsqgqpz...zkvk8nkwsv6rl`). Both strings re-decoded with the fork's `codex32.New` +
`DecodeMS1` (probe `/scratch/cache/rv-wipehook/probe`, go1.26.7): prefix 0 / 32 x `0xff`, and prefix 2 / language 0 /
16 x `0x11`. M-1 (wording: "keeps the slice itself and a copy of its contents taken inside the hook; asserts the copy
equals the vector and the kept slice is zero afterwards"), N-1 (`len` 16 for the `0x02` payload) and N-2 ("This row also
runs") are applied in the three texts below.

Change from S-1's fix text: S-1 asked that "the hook's slice shares its backing array with the function's own (cap
unchanged)". A test cannot see the function's own slice, so that clause is written as an obligation on the call
("called with the function's own slice, not a copy, while it still holds the secret"), and the test half is the live
copy: a hook given an uncleared copy fails the zero-after check on `P` (and on `B` ... `G`). Mutant "hook called with
a copy" is added beside S-1's "hook not called".

## Spec delta (exact plan texts)

1. `TestMS1LetterIsTheTopNibbleOfTheEntropy`: plan :356-357 (the Rule bullet) are replaced by:

```text
- Rule: as the Rust row; the share and a non-`0x00` prefix → not ok. Wipe (§7 `ms1LetterEntropyHook`; secret rows
  `s1b-r2-letter-check-materialise`, `plankits2-r1-s1-wipe-hook-vacuous`): the hook fires exactly once per
  `ms1Letter` call in which `DecodeMS1` returns entropy and never otherwise: once each for `A16`, `A` … `G`, `P` and
  the row's `0x02` payload, never for the N-share string (the threshold digit is tested first). Also a `0x02`
  payload with non-zero entropy, `ms10entrsqgqpzyg3zyg3zyg3zyg3zyg3zyg3zzkvk8nkwsv6rl` (`DecodeMS1`: prefix 2,
  language 0, 16 × `0x11`; not ok): the hook fires once, the copy is 16 × `0x11`, and the kept slice is zero after
  `ms1Letter` returns (the row's Rust-mirrored `0x02` payload is all-zero while live). The hook is called with
  `ms1Letter`'s own entropy slice (not a copy) while that slice still holds the secret; the test keeps the slice
  itself and a copy of its contents taken inside the hook, asserts the copy equals the vector's entropy bytes
  (`B`: 32 × `0x11`, `P`: 32 × `0xff`) and, after `ms1Letter` returns, that every byte of the kept slice is zero.
  The copy has `len` 16 for `A16` and for the `0x02` payload, and 32 for `A` … `G` and `P`. A hook call count of 0
  for any of those vectors fails: a nil capture is not "all zero", and `A16`, `A` and the row's Rust-mirrored
  `0x02` payload are all-zero while live, so `B` … `G`, `P` and the non-zero `0x02` payload carry the wipe proof.
  RED: absent. Mutants: no `clear`; hook not called; hook called with a copy (`bytes.Clone(entropy)`); return on a
  non-`0x00` prefix before `clear`.
```

2. `TestRefugiumSealedMS1WipePoints`: plan :438-440 (the Rule bullet) are replaced by:

```text
- Rule (§5): the entropy slice and the `bip39.Mnemonic` handed to the §7 hooks are all zero before the engrave
  screen's job starts; `rec` is zero as today. Each hook (`unlockSeedQREntropyHook`, `unlockSeedQRWordsHook`) fires
  exactly once per `unlockCodex32SeedQRPlan` call, on the success and the error path alike, with the plan's own
  slice (not a copy) while it still holds the secret; the test keeps the slice itself and a copy of its contents
  taken inside the hook, asserts the copy equals the vector and, after the call returns, that the kept slice is
  zero. V2-A: each hook
  fires twice; the entropy copies have `len` 16 (`A16`, 16 × `0x00`) and 32 (`P`, 32 × `0xff`), the mnemonic copies
  `len` 12 ("abandon" × 11, "about") and 24 ("zoo" × 23, "vote"). This row also runs the `A16` padding-bit
  string of `TestRefugiumSealedMS1TextOnlyFallback` (refused by `EngraveSeedStringSeedQR`): each hook fires once,
  `len` 16 and 12, the same contents as `A16`'s. Also `P` with its padding bit set,
  `ms10entrsqrlllllllllllllllllllllllllllllllllllllllllllllllllllak6svw5c6lqsf` (checksum recomputed; `DecodeMS1` ok,
  32 bytes of `0xff`; not equal to `codex32.EncodeMS1` of its entropy, so `EngraveSeedStringSeedQR` refuses it):
  each hook fires once, `len` 32 and 24, the contents of `P`'s. After each
  `unlockCodex32SeedQRPlan` call returns, every kept byte is 0 and every kept word is 0 (the zero `bip39.Word`, "abandon"). A hook call count of 0 fails: a nil capture is not
  "all zero", and `A16`'s entropy is all zero while live, so `P` carries the wipe proof on the success path
  and `P`'s padding-bit string on the refusal path. RED: absent. Mutants: `clear(m)` after `Engrave` (resident
  through the ~21-minute cut); hook not called; hook called with a copy; no `clear(entropy)` on the refusal path.
```

3. §7 "Wipes": plan :532 ("tests keep the slice and assert it zero afterwards (the `master_key_residue_test.go`
   pattern).") is replaced by:

```text
  each is called with the function's own slice while it still holds the secret; tests count the calls, keep the
  slice itself and a copy of its contents taken inside the hook, assert the copy equals the vector and the kept slice
  zero afterwards, and a hook that never fired fails the test (the
  `master_key_residue_test.go` pattern, with its "never fired" check).
```

Facts used, checked: S1b `844572cd` `design/SPEC_encrypted_payload_delivery.md:2576-2577` (`A16` = 16 x `0x00`;
`A` ... `G` = 32 x `0x00`, `0x11` ... `0x66`; `P` = 32 x `0xff`); the BIP-39 words are the reference vectors
(`00`x16 → "abandon" x 11 "about"; `ff`x32 → "zoo" x 23 "vote"), as pinned in the fork's
`bip39/bip39_test.go:113-114,157-158`; `bip39.Word` is `int` and `Mnemonic` is `[]Word` (fork `bip39/bip39.go:22,24`
@`25abe8e`); `DecodeMS1` returns entropy only for prefix `0x00` / `0x02` and lengths 16/20/24/28/32, else a nil slice
(fork `codex32/mspayload.go:35-61` @`25abe8e`); the plan's §5 (:478-489) sends the padding-bit string through
`unlockCodex32SeedQRPlan` (decode, `bip39.New`, `EngraveSeedStringSeedQR` refuses, entropy and `m` zeroed).

No holder-visible text, exit code, flag, type or command changes. No PROPOSED texts. No toolkit-manual lockstep (no
`mr` / `md-cli` surface). The coverage map (`design/IMPLEMENTATION_PLAN_fork_refugium_kit-coverage.tsv`) is
unchanged: its S1b §14.6 lines already map to these two rows (map lines 143, 144, 146).

## RED rows (run 2026-11-22 on the plan branch head 1c079d95, where the text lives)

- R1 (call-count rule absent):
  `git -C /scratch/code/shibboleth/mnemonic-engrave show 1c079d95:design/IMPLEMENTATION_PLAN_fork_refugium_kit.md | grep -c 'hook call count of 0'`
  → `0` (RED). After the patch: `2`.
- R2 (mutant absent): same command with `grep -c 'hook not called'` → `0` (RED). After: `2`.
- R3 (own-slice obligation absent): same command with `grep -c 'not a copy'` → `0` (RED). After: `2`.
- R1b (r1 folds absent), same command with each phrase: `no `clear(entropy)` on the refusal path` → `0`;
  `ak6svw5c6lqsf` → `0`; `zkvk8nkwsv6rl` → `0`; ``non-`0x00` prefix before `clear` `` → `0` (RED). After: `1`, `1`, `1`, `1`.
- R4 (the vacuity itself): a Go probe (`go1.26.4`, outside the repo, `/scratch/cache/probe-wipe-hook/main.go`) that
  applies the current Rule's check (`every byte == 0`) to a nil capture and to live, never-cleared entropy. Its
  source (with a `go.mod` of `module probe` / `go 1.26`; run `GOCACHE=<dir>/.gocache go run .` in its directory):

```go
package main

import "fmt"

func allZero(b []byte) bool {
	for _, x := range b {
		if x != 0 {
			return false
		}
	}
	return true
}

func main() {
	var captured []byte // the hook never fired: the test's captured slice stays nil
	fmt.Printf("hook never fired: captured len %d, old rule all-zero check passes: %v\n", len(captured), allZero(captured))
	a16 := make([]byte, 16) // A16's entropy (S1b vectors: 16 bytes of 0x00), live, never cleared
	fmt.Printf("A16 entropy never cleared: old rule all-zero check passes: %v\n", allZero(a16))
	p := make([]byte, 32) // P's entropy (32 bytes of 0xff), live, never cleared
	for i := range p {
		p[i] = 0xff
	}
	fmt.Printf("P entropy never cleared: old rule all-zero check passes: %v\n", allZero(p))
}
```

  Printed:

```text
hook never fired: captured len 0, old rule all-zero check passes: true
A16 entropy never cleared: old rule all-zero check passes: true
P entropy never cleared: old rule all-zero check passes: false
```

  Lines 1 and 2 are the defect (the Rule passes with no hook call and on `A16` with no wipe); line 3 is why the new
  Rules name `P` (and `B` ... `G`) as the proof vectors.

## GREEN (run 2026-11-22 on a detached checkout of 1c079d95 with the r1-folded patch applied)

- `git apply --check design/FIX_sec-me-kitplan-wipe-hook.diff`: OK.
- R1, R2, R3 → `2`, `2`, `2`; R1b → `1`, `1`, `1`, `1`.
- `scripts/plan-gate-fork-refugium-kit.sh static` → rc 0, `static: 76 ROW lines, 89 named lines, 152 map lines
  (149 to rows/tasks), 9 phases, 2 owed vectors recomputed, 0 failures` (the same line as before the patch).
- `scripts/plan-gate-fork-refugium-kit.sh selftest` → rc 0 (selftest 7 skipped: needs FORK, ME_WT,
  CARGO_TARGET_DIR).

## Tests that change

None exist yet: both rows are `now=0 | built=1` ROWs of phases SH-3 (`./seal/`) and SH-4 (`./gui/`, Refugium tag) in
the fork, written by those builds. The change is to their Rules and Mutants only; ROW lines, phases, counts and the
coverage map are unchanged. The SH-3 / SH-4 builders write each row's call counter and in-hook copy, and run the
"hook not called" and "hook called with a copy" mutants once each.

## Gate

Plan-text gate (this FIX): on the plan branch after the patch,
`ME_REPO=/scratch/code/shibboleth/mnemonic-engrave scripts/plan-gate-fork-refugium-kit.sh static` → `0 failures`,
plus R1-R3 → `2`. Not covered: whether the fork builds meet the new Rules (SH-3 / SH-4 run the rows and mutants).

## FOLLOWUPS

None closed in this repo (the row is in the coordinator's secret list). Two new lines are proposed in the report (two
further unwiped decode copies the plan's §5 leaves open, found while writing this fix); per the PR rule they are not
appended here.
