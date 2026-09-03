# composer S4 plan — R0 round 1, sonnet fold verification

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S4_acceptance.md`.
**Fold under review:** `fda1d9e` (`git diff d640875..fda1d9e -- <plan>`, 210 insertions / 108 deletions),
responding to `design/agent-reports/composer-S4-plan-R0-r0-journey.md` (persisted `ac2014e`,
1C/12I/5M/2N). Follow-ups `b431406` (F-462, F-463), the commit after the fold, as the brief states.
**Method:** read-only; no commits; no sub-agents; no `.jsonl` read. Every r0 finding compared against
the fold diff finding by finding; every pinned value the fold introduces re-run by path
(`descriptor-mnemonic/target/release/md` 0.14.0, this repo's `target/debug/me`, `~/.cargo/bin/mk`
0.13.0); the plan's own four check scripts re-run; a grep for each superseded form named in the
brief. Did not re-walk the shipped fork code beyond the targeted spot-checks below, taken only to
confirm the fold's own new citations weren't wrong needles.

---

## 1. Per-finding verdict

| finding | verdict | evidence |
| --- | --- | --- |
| C-1 (keyless md1 is unchunked, should be chunked) | **VERIFIED** | `md encode "tr(...)" --force-chunked --group-size 0` re-run: `chunk-set-id: 0xb0884`, `md1fkzyyqq9qjtvyyykjmpprj6tvyy49cqps8ys3psqcsmzu90h5wvl3` — byte-identical to the plan's §2/Task2/row-6/Task-4 string. Old 47-char string (`md15zfd...`) grepped: zero hits anywhere in the plan |
| I-1 (form A census 2 plates) | **VERIFIED** | row 19a: `This engraves 2 plates.` / `md1 policy: 2 plates (the wallet policy, with its keys)`, matches report's quoted frame verbatim |
| I-2 (`strings()` is one entry per plate, newline-joined) | **VERIFIED** | Task 3 narrative + rows 20a/20b: split-on-`\n`, flatten, compare, and assert entry count == census plate count — exactly the report's hypothesis wording |
| I-3 (no key-order Q on the 2-path keyed arm) | **VERIFIED** | keyed row 13: `Done` → straight to `Template`, no `Sorted keys...?`; keyless row 3 keeps it (`asked: ONE path`). Grep confirms `Sorted keys, or your order?` appears exactly once, on the keyless row |
| I-4 (`Seat keys into this template?` in the wrong arm) | **VERIFIED** | keyed row 14 explicitly negates it (`no "Seat keys into this template?"`); keyless row 4 now draws it. Grep: the phrase appears twice total, once negated (keyed), once affirmed (keyless) — no stray copy |
| I-5 (seed is not a source; split row 14/14a) | **VERIFIED** | row 8 loses `+ seed` (`slots: 0 / keys available: 2`, byte-identical to report's quoted frame); row 14 is the 4-row no-seed pick list; row 14a adds the three named screens (`Where from?`, `Seed for the policy`/`Source`, `Add a BIP-39 passphrase?`) verbatim from the report |
| I-6 (`1 key`, never `1-of-1`) | **VERIFIED** | rows 10 and 17 both read `Path 2: 1 key`; grep for `1-of-1` finds only the retiring clause on row 10 (`never "1-of-1", r0 I-6`) |
| I-7 (relative-lock echo has no bound line) | **VERIFIED** | row 11: one line, `12960 blocks (about 90.0 days)`, "carries no `now:` bound line (r0 I-7)" — matches hypothesis (delete the clause rather than add an absolute lock) |
| I-8 (§8i hash-rule modal as an explicit step) | **VERIFIED** | row 12 states the modal draws FIRST, full text quoted, marked shot |
| I-9 (no verify offer; ends on the Bundle-engraved modal) | **VERIFIED** | Task 3 narrative names the 4th handler (`match: "Bundleengraved"`, `act: "confirm"`) and the terminal condition (door's `Build a new policy`, never verify) exactly as hypothesized; rows 20a/6 echo `Bundle engraved → confirm; stop at the door` |
| I-10 (keyless door lead is payload-in-flash on the emulator) | **VERIFIED** | keyless-arm header + row 1 add `shSysw("none")` after the boot offer; row 2 keeps the key-less lead; Task 4 gains the "read the door's Lead FIRST" paragraph answering the report's second question (device state at Task 4) |
| I-11 (row 15 omits the SAME SEED SAME PATH warning) | **VERIFIED** | row 15 page 1 now carries the full §8g body, marked shot, tied to M-4's "deliberate" note |
| I-12 (row 20b names a file Task 2 never produces) | **VERIFIED** | Task 2 file list gains `keyed-template.md1.txt` + `cards/slot{0,1,2}.mk1.txt`; row 20b names them. Re-ran `md encode` with `--fingerprint`s + `--force-chunked --group-size 0`: `chunk-set-id: 0x34c51`, both chunks byte-identical to the fold's pinned strings. Re-ran `mk encode` for all three cards: @0 = 2 chunks, @1 = 3 chunks, @2 = 2 chunks — exactly as the fold states |
| M-1 (`h` vs `'` notation split) | **VERIFIED (declined-with-reason + filed)** | plan cells corrected to `h` throughout the stub-screen rows (13/14/16/keyless-3); shipped copy NOT changed; F-462 filed at `b431406`, content matches the finding |
| M-2 (`--in -` is not a stdin spelling) | **VERIFIED** | Task 1's command drops `--in -`, pipes instead; `me sysw pack --help` confirms "with neither this nor argv records, the same newline-separated form is read from STDIN". Independently re-ran the mechanism (arbitrary `text:` record via `--in FILE` vs. via a pipe): byte-identical output (`sha256sum` matched). The fold's specific `dbe9 e774...` digest claim can't be reproduced yet — `cmd/buildpayloadcomposer` doesn't exist until Task 1 ships — but the general mechanism it rests on checks out |
| M-3 (paged screens wrap; page-until-recurs) | **VERIFIED** | Task 3 narrative states the driver stops on recurrence, citing `readAllPages` in `shots_walletpolicy.js` as existing precedent. Confirmed: that function exists at that file, line 91, and its own comment says "this stops when a page repeats rather than counting pages it cannot know" — an accurate citation, not in the r0 report itself but correct against the shipped driver |
| M-4 (shared-master fixture) | **VERIFIED — DECIDED, not declined** | new §2 paragraph: "Two accounts of ONE master in one 2-of-2 path is deliberate (r0 M-4)... Nobody reading the record later should take that warning for a defect." Matches the brief's characterization exactly; the fixture was NOT re-minted |
| M-5 (two census rows differ only by `@i`) | **VERIFIED** | row 19b: "rows @0 and @2 differ only by the `@i` (A's account 0' and B's account 0'; r0 M-5)" |
| N-1 (46 → 56 characters) | **VERIFIED** | folded together with C-1; "56-character" appears at all four sites the plan discusses the keyless string; the old "46-character" grep returns zero hits; the only surviving "47-character" is in the sentence retiring it |
| N-2 (ms1 reminder modal, watch-only run) | **VERIFIED (declined-with-reason + filed)** | Task 3/6 note it as shipped copy, out of scope; F-463 filed at `b431406`, content matches the finding |

**20/20 findings folded: 0 NOT FOLDED, 0 PARTIAL.**

---

## 2. Propagation grep (superseded forms named in the brief)

| string | result |
| --- | --- |
| `md15zfd` (old keyless string) | 0 hits |
| `46-character` | 0 hits (only "56-character" now; one retiring mention of "47-character") |
| `1-of-1` | 1 hit — inside row 10's own retiring clause ("never `1-of-1`, r0 I-6") |
| `--in -` | 1 hit — inside M-2's retiring clause ("`--in -` is NOT a stdin spelling") |
| `7 strings` | 0 hits |
| the relative-lock `now:` bound-line clause | 1 hit — inside row 11's retiring clause ("carries no `now:` bound line") |
| `Seat keys into this template?` in the keyed arm | 1 hit, negated (row 14: "no `Seat keys into this template?`"); the affirmative copy is on the keyless arm only |
| the key-order question on the keyed arm | 0 hits in the keyed rows; the keyless row 3 keeps it, correctly |

Every superseded form survives only in the sentence that retires it, or not at all. No stray copy found.

---

## 3. Plan-check counts (re-run against the folded plan)

| script | commit-message claim | re-run result | match |
| --- | --- | --- | --- |
| `plan-cite-check.sh` | 0/0 dangling 0 | `citations resolved: 0 / 0 ; dangling: 0 ; ambiguous: 0` | yes |
| `plan-glyph-check.sh` | 46 strings, 0 undrawable | `operator strings scanned: 46 ; undrawable: 0` | yes |
| `plan-table-check.sh` | 29 rows, 0 malformed | `table rows checked: 29 ; malformed: 0` | yes |
| `plan-stepref-check.sh` | 23 prose step numbers | `step numbers in prose: 23` | yes |

All four counts in the fold commit message match a fresh re-run exactly. Spot-checked a sample of the
23 stepref hits: cross-document references (`design/S4_journey_walk_2026-09-02.md`, "step 3"; the S3
plan's "Task C2 Step 5"), ChoiceScreen row indices ("row 0"), and "rows 1-18" naming this plan's own
table — none is a stale by-number reference the check should have caught.

---

## 4. New findings

None rise to Minor or above. Two observations, informational only:

- Rows 9 and 11 (keyed arm) carry small enrichments beyond what the r0 report's row-fidelity table
  literally quoted — row 9's `slots: 2 / keys available: 2` live line, and row 11's full path-menu
  choice list (`Keys`, `Time lock`, `Hash lock`, `Remove path`, `Move up`). Neither contradicts a
  report frame; both were independently spot-checked against the shipped fork
  (`composerSlotsKeysLine`/`composerSlotCount` in `gui/composer_state.go:314-337,184-192`, and the
  choice slice in `gui/composer_shape.go:286-293`) and are correct. Not a defect — noted only because
  they go beyond the letter of "the fold matches what r0 measured."
- M-2's specific `dbe9 e774...` payload digest is asserted in the fold commit message as
  controller-machine-checked but cannot be independently reproduced by this round: `cmd/buildpayloadcomposer`
  is Task 1's deliverable and does not exist yet. The underlying mechanism claim (stdin == `--in FILE`,
  byte-identical) was verified generically and holds.

---

## 5. Closing counts

**Round 1: 0 Critical / 0 Important / 0 Minor / 0 Nit open.** All 20 r0 findings (1C/12I/5M/2N) are
VERIFIED as folded — 18 as direct fixes, M-1 and N-2 as declined-with-reason-and-filed (F-462, F-463,
confirmed present at `b431406` with content matching their findings), M-4 as DECIDED (fixture kept,
deliberate note added). Propagation grep clean. All four plan-check counts reproduce the fold commit
message exactly. Round 1 closes GREEN.
