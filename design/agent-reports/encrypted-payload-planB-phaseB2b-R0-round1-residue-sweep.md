# B2b R0 round 1 — residue sweep (workflow, 36 agents, verbatim)

**Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` at `7c3a625`
**Fork:** `seedhammer` at `a01b666`
**Brief:** the RESIDUE — what round 0's and round 1's deliberately tight briefs left
uncovered: Tasks 6, 7 and 8, the Global Constraints and green criterion, and the
per-phase follow-up reconciliation the standard workflow requires on entering a
phase, which had never been run for B2b. Three disjoint read-only lenses, then a
refute-by-default pass on every candidate, then synthesis.

**Yield:** 32 candidates → **23 survived refutation, 9 refuted**. Final verdict
**0C/4I** (+ 12 Minor, 3 Nit).

Persisted verbatim before folding.

**Controller's independent confirmation of the two most consequential claims,
before folding** (read from source, not taken from this report):

| claim | verified |
| --- | --- |
| **I2** — the green criterion's `GOARCH=386 go test ./seal/ ./bip39/` row FAILS as written at baseline | ran it: `gnu/stubs-32.h: No such file or directory`, `FAIL seedhammer.com/seal [build failed]`. The row is missing `CGO_ENABLED=0`. My transcription error. |
| **M1** — package `gui` will not compile after Task 1's move | `saver` appears in `gui/gui.go` only at the import (`:37`) and at `:2942`/`:3004`, both inside `Run` (2934-3020). Moving the body out orphans the import. |

---

# B2b PLAN — RESIDUE SWEEP (merged, 3 lenses)

**VERDICT: 0 Critical / 4 Important — GATED.** Plus 12 Minor, 3 Nit (recorded, non-gating).

Scope note applying to every entry: repo HEAD is **7c3a625**, one commit past the `da225c0` in the brief (7c3a625 is the F-99 §10.2.4 amendment). `git diff da225c0 HEAD -- design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` and `-- design/FOLLOWUPS.md` are both **empty**, so every line number below resolves at HEAD.

---

## IMPORTANT (blocking)

### I1 — Task 6 / F-87: the prescribed remedy is unimplementable, and the obvious implementation is a vacuous test on seed material
**Where:** plan Task 6, F-87 bullet at :916-919 and step 6.1; `seedhammer/gui/unlock_session.go:250, 257, 266, 272, 291-292`

**What is wrong.** F-87's *diagnosis* is exact — `defer clear(m)` at `unlock_session.go:250` covers exactly three early returns (`:257 !ss.Confirm`, `:266 masterFingerprintFor` err, `:272 engraveSeed` err). Its *remedy*, which Task 6 inherits verbatim ("Drive each with `unlockMnemonicHook` set"), cannot work: `grep -n "unlockMnemonicHook(" gui/*.go | grep -v _test` returns **exactly one** invocation site, `:292`, reached only on the success path *after* `clear(rec)` (:289) and `clear(m)` (:290). No early return reaches it. No other seam reaches `m`: `unlockSecretHook` (:117/:119) carries `rec`, not `m`; `bip39.parseWordsHook` is unexported and unreachable from package `gui`.

**Why it matters.** `bip39.Mnemonic` is `[]Word`, so the natural test is `var got bip39.Mnemonic; unlockMnemonicHook = func(m bip39.Mnemonic){ got = m }`, drive the early return, range over `got` asserting zeros. The hook never fires, `got` stays nil, the range asserts nothing — and the test passes identically with the defer deleted. Task 6 also carries **no ```go block at all** (last ```go fence in the plan is line 601), so `plan-build-gate-go.sh` type-checks nothing here.

**Minimal fix.** Budget a *new* seam in Task 6, in a gated code block: `var unlockMnemonicParsedHook func(bip39.Mnemonic)` fired immediately after the defer at :251. Add a fired-guard requirement to 6.1 (`if got == nil { t.Fatal(...) }`) and a mutation row asserting the hook fired.

**Carry these corrections** (found during refutation, they bound the claim but do not dissolve it): (a) step 6.1's own gate polarity *catches* the vacuous case — a vacuous test PASSES under the mutant, which fails 6.1's "delete the defer → tests fail" criterion, so this stalls the task rather than shipping a false GREEN; (b) the fired-guard idiom already exists 30 lines from where the new test goes (`gui/unlock_session_test.go:678`, `bip39_test.go:393/:436`); (c) the exposure is **not** "the twelve words" — `m` is `bip39.Parse(rec)` of the payload SEED record; the twelve words are §8's KDF passphrase, handled in `unlock_kdf.go`; (d) the window is a `showError` screen, not the ~21 min cut, and F-87 itself already measures this (FOLLOWUPS.md:1458-1463).

---

### I2 — Green-criterion row `GOARCH=386 go test ./seal/ ./bip39/` is RED at the baseline: `CGO_ENABLED=0` was dropped in transcription
**Where:** plan :107

**What is wrong.** Ran it in the fork at a01b666: `nix develop --command bash -c 'GOARCH=386 go test ./seal/ ./bip39/'` → `# runtime/cgo … gnu/stubs-32.h: No such file or directory`, `FAIL seedhammer.com/seal [build failed]`, exit 1. With the flag restored: `ok seedhammer.com/seal 52.2s`, `ok seedhammer.com/bip39 0.084s`, exit 0. The source row this was transcribed from carries the flag (`design/agent-reports/encrypted-payload-planB-phaseB2a-ii-lens8-completeness.md:241`). The row directly above it (:102) already has `CGO_ENABLED=0`, so this reads as a slip, not a decision. The flag was already missing at da225c0.

**Why it matters.** This is the phase's definition of done, run after all eight tasks. One of six rows is red at the *baseline* — the implementer either burns a round chasing a non-regression, or learns to treat a red row as expected, which is how a genuine `seal` regression on the 32-bit target (the firmware word size) gets waved through.

**Minimal fix.** `| `CGO_ENABLED=0 GOARCH=386 go test ./seal/ ./bip39/` | green |`, and note the ~52 s runtime so it is not mistaken for a hang.

---

### I3 — Task 7: 9 of the plan's 20 mutation rows cannot be applied mechanically, and Task 3.3's own "each names a literal token" claim is false for 2 of its 5 rows
**Where:** plan Task 7 (:924-934); mutation tables at 2.3 (:432), 3.3 (:511-520), 4.3 (:842-855), 5.1-5.3 (:885-905), 6.1 (:919)

**What is wrong.** Measured by extracting all seven ```go fences from the plan and counting each named token *inside them*:

| token | occurrences in plan's own blocks |
|---|---|
| `break` | **5** — :289 (harness) and :677, :757, :777, :792 in `run_flow.go`; :677 is the `pl.NextChunk()` chunk-walk |
| `a.idle.start = now` | 2 (:729, :740) |
| `a.idle.active = false` | 2 (:652, :741) — and rows 4 and 5 of Task 4 apply this *identical* token at two different sites |
| `if armed {` | 2 (:733, :772) |
| `if wiping {` | 2 (:464, :698) |
| `if !wiping { return }` | **0** — the block writes it across three lines (:796-798) |
| `if wiping { continue }` | **0** — three lines (:463-467, :698-700) |

Uniquely appliable: `wiping := false`, `ctx.B.Reset()`, `secs < 0`, `a.warnBuf.Reset()`, `&a.warnBuf`, `&& !armed`, `defer func() { ctx.wipe = nil }()` — one each. Task 5.2's mutant ("swap the read past `ctx.Reset()`") is a statement *reordering*, not a substitution, and is not expressible as a row at all. Line 512 asserts as a design property "Each names a literal token so Task 7's runner can apply it mechanically" — false for rows 1 and 3 of that very table.

Separately: `grep -n "file copy\|git checkout\|matched exactly once\|substitution matched\|silently-failing\|anchor"` over the whole plan → **zero hits**. Task 7 carries neither of §11.3's two procedural rules (SPEC:1618-1620: assert the substitution matched, and restore from a **file copy**, never `git checkout`).

**Why it matters.** §11.3 warns "a silently-failing `sed` reads exactly like a surviving mutation", and Task 7.2 makes a surviving mutant blocking — so these rows either block the phase spuriously or pass spuriously. `break`→`return` in the `NextChunk` loop silently truncates every frame's chunk walk. F-96 exists precisely because these substitutions had to be fixed twice mid-run.

**Minimal fix.** Give every row a `(file, unique-anchor, replacement)` triple instead of a bare token — anchor on the enclosing distinctive line (e.g. `if wipeNowHook != nil && wipeNowHook() {` … `break`). Rewrite 3.3 row 3 to the literal three-line form the block contains. Add a ```go block for Task 6 (see I1) so its row has a source. Drop or re-express 5.2 as an ordering assertion. State §11.3's two procedural rules in Task 7 and make "matched exactly once" a hard failure in the runner.

**Two sub-claims corrected during refutation:** `defer clear(m)` occurs **once** in the real fork tree (`gui/unlock_session.go:250`) — that row *is* uniquely appliable against the tree, only against the plan's own blocks is it zero. And `scripts/plan-build-gate-go.sh` does **not** operate on the live fork tree: `:66-69` tars `git ls-files` into a scratch `$WORK` dir. The "a `git checkout` restore would discard uncommitted phase work" framing is wrong; the file-copy rule is still worth stating, on §11.3's authority alone.

---

### I4 — Three B2b-owned follow-ups are deferred to "own cycle", i.e. past the phase that owns them, into no phase
**Where:** plan :76-78 (coverage table) and :1037-1042 ("What B2b does NOT cover"); `design/FOLLOWUPS.md:1053` (F-94), `:1329` (F-90), `:1411`/`:1440` (F-88)

**What is wrong.** FOLLOWUPS records the owning phase of F-88 as "B2b, with F-87" (:1440), F-94 as "B2b, with F-87/F-88" (:1053), F-90 as "owning phase: B2b" (:1329). The plan's coverage table pushes all three out: `F-88, F-94 further seed copies | — | own cycle` and `F-90 items 1 and 3 (ms1 inventory, hook) | — | own cycle`. "Own cycle" is not a later phase — it is no phase, so these leave B2b with no successor owner. `/scratch/code/CLAUDE.md`: "An item that binds the current phase, or is scheduled *to* a phase … is **not deferrable past its owning phase**."

**Why it matters.** All three are secret-residency items, and F-90's items 1 and 3 are on `unlockEngraveCodex32` — the arm six of the seven canonical vectors take, and the one F-90 itself calls "the DEFAULT arm" with "no hook. No inventory. No follow-up." (`grep unlockCodex32Hook` over `gui/*.go` → zero; `unlockMnemonicHook` exists at `unlock_session.go:48`.) The plan says at :21-23 that B2b "makes the feature operator-complete" and that only after it plus the hardware pass and F-85 is a tag defensible — so as written the feature reaches an operator, and a tag, with the default ms1 secret arm never inventoried and nobody scheduled to do it.

**Minimal fix.** Either (a) pull F-90 items 1 and 3 into B2b — the inventory is prose and the hook mirrors `unlockMnemonicHook`, both small next to Tasks 1-4; or (b) get an explicit operator re-assignment and **amend the three FOLLOWUPS entries to name the new owning phase before B2b's gate**. Do not leave the register saying "B2b" while the plan says "own cycle". For F-88: if the retraction of its `clear(words)` remedy (commit 3f4e344) leaves no work, close it rather than defer it.

---

## MINOR (recorded, non-gating)

### M1 — Gate-coverage blind-spot list is incomplete: after Task 1's move, package `gui` does not compile (unused `saver` import)
**Where:** plan "Gate coverage" :960-998 (esp. :971-976, :991); Task 1a at :149-152, steps at :320-334

`grep -n saver gui/gui.go` → import at :37, references at :2942 (`state saver.State`) and :3004 (`a.idle.state = saver.State{}`), plus a prose comment at :3009. `func Run` spans :2934-3020, so **both references are inside Run's body and there are none elsewhere**. Task 1a calls the change "a pure move" and never says to drop the import. **Reproduced:** scratch copy of the fork, Run's body replaced with the plan's one-liner + `gui/run_flow.go` added → `gui/gui.go:37:2: "seedhammer.com/gui/saver" imported and not used` — exactly one error, which also proves no other gui.go import is Run-body-exclusive (`runtime`, `log`, `color`, `text`, `utf8`, `bspline`, `engrave`, `bezier` have 0 uses in Run; `image`/`time`/`op` have 89/24/179 elsewhere).

`plan-build-gate-go.sh` TIER 1 is **additive** (`grep -n "remove\|replace\|delete"` → no matches): it adds the plan's new files to a fork copy and never removes the old `Run` body, so gui.go keeps using `saver` in the scratch copy. The gate reported OK on a configuration that cannot be the shipped one, while the plan's blind-spot list names exactly two items (`ctx.wipe undefined`, `ctx.keepAwake undefined`).

**Fix:** add "remove the `seedhammer.com/gui/saver` import from gui.go" to step 1.1; extend the named blind spot to "gui.go's import set after the body is removed — TIER 1 adds files rather than replacing them." Durable option: make step 1.1's own gate `CGO_ENABLED=0 go build ./gui/` in the real worktree.

---

### M2 — Task 7 / §11.3: the rows B2b owns are never enumerated, and the one unambiguously-B2b row appears in no mutation table
*(merged: `task7-claims-1103-rows-it-does-not-name` + `spec-11-3-row-not-carried` + `s11-3-rows-b2b-owns-are-never-enumerated`)*
**Where:** plan :74 (coverage table), Task 7 :924-934, Task 2.3 :432, Task 4.3 :842-855; SPEC §11.3 row at :1610; F-96 at FOLLOWUPS:1123

The coverage table claims "§11.3 mutation rows + **F-96**'s runner | ✅ Task 7", but `grep -n "11\.3"` over the plan returns **exactly 2** hits — the table row and the Task 7 heading. Task 7's whole body is the runner paragraph + 7.1 "re-run every row this phase owns" + 7.2 "a surviving mutant is blocking". "The rows this phase owns" is defined nowhere.

§11.3 is a **27-row** table (SPEC:1586-1612; the "~28" in one lens is wrong, and 27 matches B2a-ii:1521's independent count). Exactly one row is B2b's: `| idle timer runs during engraving | §11.2 timer-paused assertion (§10.2.4) |` — and B2a-ii's Task 8 explicitly deferred it: "**B2b** — no timer exists in B2a. Record as deferred, with its owning phase, rather than claiming coverage." The precise mutant is deleting `case engraveRunning, engraveStopping: return false` from `wipeGuard.armed()` (plan :388-393). It is in neither Task 2.3's single row nor Task 4.3's ten. **Task 4's `armed` hardcoded true/false does not substitute for it** — under the `armed()` mutant the guard still returns false when `ctx.wipe == nil`, so "the not-armed test — a wipe on the public plate list" does not discriminate. No such test exists at a01b666 (`grep -rnE "timer.*paus|paused.*engrav" --include=*_test.go` → nothing) and cannot, because there is no timer yet.

F-96 has a second deliverable the plan does not address: `grep -in "phase report\|substitution\|file copy\|sed "` over the plan → no hits, so F-96's "Land it with the phase report if that is still owed" and its 6.1/6.5/6.7 match-semantics warning appear nowhere. `ls design/agent-reports/ | grep -ci b2a-ii` → **11** files, all lens reports, **no phase report** — so the row table Task 7 must encode as data has no source in the repo.

**Fix:** in Task 7, enumerate B2b's §11.3 rows by name (at minimum "idle timer runs during engraving"); state where the runner's row table comes from; restate §11.3's two procedural rules (see I3). Add to Task 2.3: `delete the case engraveRunning, engraveStopping arm from wipeGuard.armed()` | killed by 2.1's "`armed()` is false while a job runs" (Task 4.1's post-cut test also drives a job past `idleTimeout`), tagged as §11.3's "idle timer runs during engraving" so traceability is a grep. Either schedule the missing B2a-ii phase report or amend F-96 to drop that half with a reason.

**Note the mitigations:** the behaviour *is* covered by step 2.1's required test, so no defect ships — this is bookkeeping, which is F-96's entire subject. And §11.3 is named by section, and that section carries the procedural rules, so they are reachable rather than un-invented. Task 7's paragraph is a near-verbatim restatement of F-96's own fix sentence, so F-96's primary deliverable *is* addressed.

---

### M3 — "Task 9" belongs to a different plan, and the release-tag precondition set is stated three inconsistent ways, all incomplete
*(merged: `release-tag-prereq-set-incomplete` + `task9-cross-plan-and-tag-preconditions-stated-three-ways`)*
**Where:** plan :21-23 vs :1043-1045; `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md:1574-1613`; `design/CONTINUITY_2026-08-08b.md` §13

Line 22: "Only after it, plus **Task 9's** hardware pass and F-85, is a release tag defensible." `grep -n '^## Task'` on the B2b plan → **8 hits, Task 1 … Task 8**, and Task 8 (:938) is itself "hardware (operator-run)". Task 9 is real but is **B2a-ii's** `## Task 9 — hardware: the in-situ KDF rate, and end-to-end unlock` (:1574, steps 9.1-9.8, outstanding per CONTINUITY §13 item 1). B2b cites it by bare number, so a reader cannot tell it is another plan's task; it appears in neither B2b's task list nor its "does NOT cover" list.

Three inconsistent statements of the tag's preconditions:
- (a) :23 — B2b + "Task 9's" hardware pass + F-85
- (b) :1045 — "**A release tag.** Task 8 and F-85 both precede it." (silently drops the other hardware pass)
- (c) FOLLOWUPS owns **F-92** (:1237), **F-98** (:1191) and F-85 (:1507) as "before the release tag" — the plan lists F-92 and F-98 as deferred three lines above (:1043) but omits them from the tag bullet.

A fourth precondition is in none of them: CONTINUITY §13 item 4, "Push both repos via `ci/staging`. Neither has been pushed this cycle" — measured **42** commits ahead on `origin/master..master`, **28** on `origin/main..HEAD` in the fork.

**Why it matters.** No single place holds the tag's precondition set, and the one bullet presenting itself as the answer under-counts by at least four. F-92 is that the TinyGo wipe caveat this whole feature rests on has never been run under TinyGo. B2b Task 8 and B2a-ii Task 9 are two trips to the same machine; with Task 9 unowned in B2b's lists, the §7.1 in-situ RP2350B measurement — the last unresolved item in SPEC §12.1 — can be missed or closed unrecorded. `ls design/ | grep -i hardware` → only `HARDWARE_INVENTORY.md` and `HARDWARE_RESULT_2026-08-07_phaseB1.md`, so Task 9 is genuinely unperformed.

**Fix:** write "**B2a-ii's** Task 9" wherever it appears in B2b. Replace the ":1045 release tag" bullet with a numbered checklist — B2b Tasks 1-8 green · B2b whole-diff review 0C/0I · merge · B2a-ii Task 9 (in-situ KDF rate, §7.1) · F-85 · F-92 · F-98 · SPEC §11.5 hardware set (see M4) · push both repos via `ci/staging` — cross-referenced to the "does NOT cover" list so the two cannot drift. Since Task 8.1 already unlocks on the real machine, add the derivation time to Task 8.5's record list and §7.1 closes for free, making the two trips one.

---

### M4 — SPEC §11.5's "Confirm firmware reflash preserves the blob" has never been run and is owned by nobody
**Where:** `design/SPEC_encrypted_payload_delivery.md` §11.5 (`:1813` at da225c0, `:1822` at HEAD); vs plan Task 8 (:944-952) and "does NOT cover"; vs `design/HARDWARE_RESULT_2026-08-07_phaseB1.md`

B1's hardware run covered exactly four things, and reflash-preservation is not among them — its headings are (1) write/read-back at the normative address, (2) §10.1 negative path, (3) §10.1 positive path + §6.6, (4) present→absent. Its closest statement is the **converse**: ":104 Only the 64 KB payload region was cleared; B1's firmware was untouched." B2a-ii's Task 9 does not cover it either (9.1-9.2 load the payload *after* the firmware). `grep -rn "11\.5\|reflash preserves" design/` → no plan step, no follow-up, no F- entry. So it meets all three residue conditions: normative and required before a tag, present in the corpus, absent from both of B2b's lists.

Secondary: §11.5 specifies "boot on **PD power**", named by neither Task 8 nor B2a-ii Task 9. `cmd/controller/platform_sh2.go:161-162` sets `minVoltage = 20_000`; `monitorPowerSupply` (:464, called :260, before `ili9488.New` at :287) reboots into BOOTSEL on failure — a bench-USB pass is not the same test.

**Why it matters.** Failure mode is silent loss of the operator's sealed payload on a routine firmware update, and the fork ships a public guide teaching exactly that procedure. It is the one §11.5 check whose result is not predictable from code — it depends on the signed UF2's sector map versus `0x10E00000`.

**Fix:** add as Task 8 step 8.0, where it is nearly free — if B2a-ii Task 9 runs first, vector F's blob is already in flash, so flashing B2b's firmware over it *is* the reflash test: `picotool save -r 0x10E00000 0x10E00040` before and after, record both. Name PD power in 8.1. If Task 8 will not carry it, file it as a follow-up owned "before the release tag" alongside F-85/F-92/F-98.

---

### M5 — F-90 item 2 is "DISSOLVED" in the plan but still reads as open funds-flagged B2b work in the register
**Where:** plan :80-83 and :1041-1042; `design/FOLLOWUPS.md:1359-1365` (F-90 item 2), `:1401-1409` (F-89's 2026-08-09 amendment)

The dissolution is **sound** — verified `SecretsResident` has no production caller at a01b666 (definition `seal/session.go:48`; every call site is in `gui/unlock_session_test.go`, `seal/session_test.go`, `seal/unlock_key_test.go`), so Task 6's rename is safe. But `grep -rn "DISSOLVED\|dissolv" design/` hits the plan (:80, :1014, :1042), the consult, the spec and two agent reports — and **never FOLLOWUPS.md**. The register still carries item 2 as live work ("**Correct `p.SecretsResident()`'s contract** — this is the one with a funds consequence"), and F-89's amendment repeats it, ending "**Fix the predicate's contract before building the timer on it**". `grep -n FOLLOWUPS` over the plan → no matches, so no register edit is scheduled.

**Why it matters.** After B2b closes, a sweep reads two open entries insisting a funds-consequence predicate must be corrected before the timer is built — against a timer already built that deliberately does not use it. A design decision recorded only where the register cannot see it gets re-litigated, on the class of item most likely to stop a phase.

**Fix:** one line each on F-90 item 2 and F-89's amendment, naming the reason (timer keys on the session bracket's lifetime; predicate has no production caller) and the rename to `RecordsResident`. Register edit, in the **same commit as Task 6**, not a plan edit.

---

### M6 — Task 6's rename surface is wider than "callers and docs", and 31 of the 73 design-doc mentions are in verbatim-persisted reports
**Where:** plan Task 6 step 6.2 (:920)

Counted, not estimated. Fork at a01b666: **28 lines / 8 files** carry `SecretsResident` — of which only **10** are compiler-visible (9 `.SecretsResident()` call sites + the declaration at `seal/session.go:48`). The other **18** are comments, two test *function names* (`TestSecretsResidentIsFalseWhenTheSessionEnds` at `gui/unlock_session_test.go:469`, `TestSecretsResidentGoesFalseOnlyWhenTheSECRETSAreGone` at `seal/session_test.go:56`) and seven failure-message string literals — none of which `gopls rename` rewrites. Design repo: **73** mentions across 17 files, **31 of them under `design/agent-reports/`** (persisted verbatim, must not be edited), **42** in live docs.

Also: `SPEC_encrypted_payload_delivery.md` contains **0** `SecretsResident` and **1** `RecordsResident` (:1339) — the GREEN spec already cites the post-rename name for a symbol that does not exist. So Task 6 is what makes the spec's cite resolve; the plan calling it "Doc tightening only" is wrong — it is spec-conformance work. (`plan-cite-gate.sh`'s symbol regex cannot match `` `seal.RecordsResident()` ``, so the gate does not currently see this either.)

**Fix:** say in 6.2 — rename the 10 compiler-visible refs plus the 18 comment/name/string mentions (28 lines, 8 files); update the 42 live-doc mentions; **leave `design/agent-reports/` untouched and say so in the commit message**. Re-run `scripts/plan-cite-gate.sh` and note SPEC:1339 going from unresolvable to resolving.

---

### M7 — Task 8 never exercises `SeedScreen.Confirm`, one of fact 3's two post-`Done` `ctx.Frame` sites, because vector F contains no mnemonic record
**Where:** plan Task 8 steps 8.1-8.4 (:944-952); plan fact 3 (:42-47); SPEC §11.4 Vector F

**Recomputed rather than trusting the §11.4 table:** built a scratch program against `seedhammer.com/seal` at a01b666, loaded `seal/testdata/vectors.json`, ran `seal.AdmitSection(secret, SectionEncrypted)` per vector — A: 1 rec, `ClassMnemonic=1`; B: same; C: 6, 0; D: 1, 0; E: 0; **F: 15 recs, `ClassMnemonic=0`**; G: 3, 0. Every Task 8 step uses vector F (`grep -n "vector"` in Task 8 → one hit, :944; `grep -n "vector A|vector B"` over the plan → none). `SeedScreen.Confirm` is reached on the unlock path from exactly one place, `gui/unlock_session.go:256` inside `unlockEngraveMnemonic`, dispatched only on `case seal.ClassMnemonic` (:143-144). The ms1 arm (`unlockEngraveCodex32`, :162) has no confirm screen (:197 goes straight to `NewEngraveScreen(...).Engrave(...)`). So with vector F the twelve-word confirm screen is unreachable and 8.1/8.2 park on `ChoiceScreen.Choose` (:139). Fact 3 names two post-`Done` `ctx.Frame` screens — `gui.go:2460` (SeedScreen.Confirm) and `gui.go:2758` (EngraveScreen.Engrave); 8.3 covers 2758, **nothing covers 2460**.

**Why it matters.** Task 8 exists because this is the first time `ctx.Done` is ever true on real hardware, and an unguarded post-`Done` yield is "a range-over-func panic — a brick on a watchdog-less device". The untested screen is the one displaying the plaintext words — the canonical §10.2.4 walk-away — and it is unreachable *by construction* with the chosen vector, so an operator following the steps will not discover the gap.

**Fix:** add a step using a vector carrying a BIP-39 mnemonic (§11.4 A or B), unlock, advance to the confirm screen, walk away; confirm the warning, the wipe, and a live start screen. Tell the operator a frozen/blank panel (not a reboot) is the discard-guard's failure signature, since `cmd/controller/main.go:34-36` returns from `main` when `gui.Run` stops yielding.

**Carry this caveat:** reading the fall-through sites, `gui.go:2460` is only reached post-`Done` when `Done` goes true inside the nested discard-confirm loop (:2352-2363); symmetrically :2758 only from the nested `ConfirmDelay` loop (:2727-2746). A plain walk-away parked at either outer `for !ctx.Done` exits cleanly — so 8.3 probably does not exercise 2758 either, and the fix step must park in the nested loop to bite. The guard itself is one screen-agnostic statement (`if wiping { continue }`, plan :692-700) already targeted by Task 3's mutation row :517.

---

### M8 — The TinyGo row of the green criterion gives numbers but no command
**Where:** plan :106

`| TinyGo device build | baseline **1310184 flash / 60584 ram** — report the new numbers |`. Every other row is a runnable command. The invocation exists at `seedhammer/.github/workflows/test.yml:29`: `nix develop --command tinygo build -size full -print-stacks -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`. **Recomputed at a01b666: `1310184 60584 | total` — exact match.** Flag sensitivity measured: re-running with `-opt z` alone gives `893676 / 69340` — a 416 508-byte flash delta and +8756 ram.

Tasks 3.4 and 4.4 both require a device build and the numbers are compared across phases; the plan's own A-C1 finding was a 228 KB buffer growth, so this delta is the phase's only RAM-budget signal. Both predecessor plans (B2a-i:136, B2a-ii:97) carry the full command.

**Fix:** quote the workflow's exact command in the row, or cite `.github/workflows/test.yml:29`.

---

### M9 — Gate-coverage line count for `gui/wipe_guard.go` is 51; measured 52
*(merged: `wipe-guard-linecount-off-by-one` + `gate-coverage-line-count-hand-counted`)*
**Where:** plan :968-969

Ran `./scripts/plan-build-gate-go.sh design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` — TIER 1 prints `gui/run_harness_test.go (154 lines)`, `gui/wipe_guard.go (**52** lines)`, `gui/wipe_warning.go (51)`, `gui/run_flow.go (204)`. Independently replicated with an awk fence extractor: fences at 167→154, 343→**52**, 530→51, 601→204. Three of four match; wipe_guard is 52, plausibly transcribed from wipe_warning's 51 on the line above.

This lands in the R0 brief as a machine-verified fact, and **F-97 recorded the identical defect in the B2a-ii plan** (two of three counts wrong), remedy "correct them with `wc -l` output or drop the numbers". Same class, one plan later. The script *already prints* the counts.

**Fix:** paste `plan-build-gate-go.sh`'s TIER 1 output instead of restating the numbers by hand.

---

### M10 — The §10.2.4 amendment redefined the term the whole design keys on, and no independent review of the amendment is owned by anyone
**Where:** commit `0af8f97` (SPEC §10.2.4); `design/agent-reports/` (no such report); plan preamble :10-13

`git show 0af8f97` — three normative changes to a GREEN spec, including redefining "resident" from a buffer scan to a **lifetime** and forbidding `RecordsResident()` as the timer's key, which is the definition B2b's entire design keys on. The only gate its message names is the **cite** gate ("CITE GATE: 2 unresolvable, and BOTH PRE-DATE this edit"). `git log --all -- design/agent-reports/` shows only B2b *plan* reviews. Round 0's briefs asked "does it faithfully implement §10.2.4", not whether the amended §10.2.4 is right; the fable consult `9980f6e` (08-09 05:55) *precedes* the amendment (09:23) and is its input, not its review. F-98 owns stale cites, F-85 owns a §2.2 addition — neither owns the new text. The plan says "Where this plan and §10 disagree, §10 wins", so an error propagates with the gate pointing the wrong way. (7c3a625 is a second un-reviewed amendment, and its own message documents a markdown defect the first one introduced.)

**Fix (cheapest closure):** record explicitly — in the plan's gate-coverage section or as a one-line FOLLOWUPS entry — that the fable consult plus operator sign-off is the accepted gate for `0af8f97`. A stated decision costs nothing and ends the silence. Otherwise scope a short review of §10.2.4's amended text into the F-85/F-98 spec-opening pass that must precede the tag, so the GREEN spec is opened once and gated once. **Precedent for the low severity:** `5b31211` amended the same spec citing only the cite gate and no review.

---

### M11 — The "## Open" list cannot be swept: 11 entries carry a closure banner, 5 more are demonstrably done in code
**Where:** `design/FOLLOWUPS.md:5-1605`

65 distinct entries between `## Open` (:5) and `## Resolved` (:1606). **Eleven** carry a closure/withdrawal/won't-fix/accepted banner in their own text and were never moved: `seedhammer-plate-carries-only-secret-prefix` (:19), `-fuzzconstantqr-never-reaches-ecc-l-dim37` (:29), `-hash-glyph-still-k4` (:39), `sh2-custom-firmware-public-guide` (:51 "This item is now closed."), `-nfc-secret-refusal` (:57 WON'T FIX), `sizeproof-qr-step-must-not-offer-what-it-drops` (:70), `sizeproof-admission-count-at-its-own-rungs` (:100), F-59 (:422 WITHDRAWN), F-97 (:1152 CLOSED), F-91 (:1309 CLOSED), F-83 (:1541 ACCEPTED LIMITATION).

**Four more read open but are done at a01b666, and every one is owned by a phase that has closed:** F-77 (B2a-i T1 — `seal/label_encrypted.go` exists; `seal/record.go:106` "F-77, closed in B2a"), F-79 (B2a-i T2 — `gui/gui.go:1570` "F-79: the READER is retained, the BYTES are not"), F-84 (B2a-ii T6 — `SeedScreen.NoEdit` at `gui/gui.go:2324`, set at `gui/unlock_session.go:255`, pinned by `TestSeedScreenNoEditClosesBOTHRoutes`), `seedhammer-fontproof-test-pattern` (shipped and renamed, `gui/passphrase_passproof.go:54`). Inside F-80, both bullets the 2026-08-08 split assigned to B2a-ii are also done (`gui/unlock_plates.go:33,55`; `gui/unlock_platelist.go:173`) and unmarked.

**Why it matters.** The standing rule is "on entering a phase, sweep the open follow-ups". Roughly a quarter of what that sweep returns is noise, and the noise is concentrated where it hurts: the four undocumented-done items are B2a-owned, so a *correct* sweep reports four B2a items as OVERDUE and sends someone to redo shipped work. That converts a hygiene rule into a false-alarm generator.

**Fix:** before B2b's gate, move the eleven bannered entries below `## Resolved` (the precedent is FOLLOWUPS.md:1625-1628, which did exactly this for two entries on 2026-08-08), and add closure banners with commit SHAs to F-77, F-79, F-84, the fontproof bullet, and F-80's two B2a-ii bullets. **A register commit, separate from any plan fold.** The plan under review carries none of this stale state — its deferral list correctly omits all four.

---

### M12 — OVERDUE: `bootkey-rehearsal-fidelity-residue` is owned "before the SH2 OTP write", and that write happened 2026-08-03
**Where:** `design/FOLLOWUPS.md:55` (and :53, same position)

:55 reads "**Owning phase: hardware-bringup (before the SH2 OTP write).**" Item (c) resolved 2026-07-26; items **(b) host-vs-on-device sealing** and **(e) the Pico 2 W misdiagnosis** are still open. The write has since executed — corroborated from four sources: `sh2-custom-firmware-public-guide` (:51 "the machine booted to the home screen displaying `(UNLOCKED)`"), `CONTINUITY_2026-08-04.md:26` ("burned into **OTP slot 1** … one-time, permanent"), `REHEARSAL_RESULT_2026-08-03.md`, `RUNBOOK_custom_boot_key.md:471/:496`. The listed blockers are discharged (`nix` present at `~/.nix-profile/bin`; `sh2-precheck` referenced by `scripts/pico2-bootkey-rehearsal.sh` and four gate reports). The sibling `seedhammer-upstream-prs-tracking` (:53) is in the same position — its "Runbook drafted (**not yet executed**)" is now false. `RUNBOOK_custom_boot_key.md:3` still says "not yet executed on hardware" while :471 says "DONE 2026-08-03", independently confirming this is documentation staleness, not a live gate.

**Why it matters.** These are the two OTP entries in the register, two slots remain, and OTP writes are irreversible. A reader consulting them before the next burn gets a pre-write checklist for a write that already happened.

**Fix:** close (b) as unfalsifiable-post-write with the `--sh2-precheck` reads named as the bound; keep (e) as a documentation note re-parked on the next OTP session; rewrite `seedhammer-upstream-prs-tracking`'s status block to say bringup completed.

---

## NIT (recorded)

### N1 — Six open entries record no owning phase at all; F-80's is literally undecided
`design/FOLLOWUPS.md` — a Python parse of the Open region (splitting on `^- **` / `^### `, case-insensitive test for "owning phase") leaves these top-level entries with none: `-titlestring-filter-widened` (:43), `-wdt-id-override-tlv-golden` (:61), `-template-engrave-policy-summary-display` (:63, "own gated cycle when picked up" — a shape, not a phase), `-broad-miniscript-renderer` (:65, same), `-engrave-33word-font-legibility` (:67), and **F-62** (:520), whose only owning-phase string lives in the retained heading at :530 the entry itself labels "(context withdrawn)". The rule is "Record the owning phase in each follow-up entry so reconciliation is a grep" — an entry with no phase is never reconciled by any sweep, ever. Two are not cosmetic: `-wdt-id-override-tlv-golden` is a golden gap on `md.WalletDescriptorTemplateId` the entry itself calls "security-load-bearing (mk1↔md1 binding)"; F-62's stranded phase reads "BEFORE any curve lands, and before `O`/`o`/`8` are drawn" — a firmware panic mid-plate with the needle down for anyone who curves a `font/constant` glyph. **Fix:** assign a phase or an explicit "none (idea, unscheduled)" the way `seedhammer-engraving-font-swap` (:45) already does; lift F-62's phase into the live heading; split F-80's header into per-bullet owners. **Correction:** F-80's block (heading :854, ends :900) has 7 bullets, 3 carrying a phase (L865, L876, L896) — removing the two the header assigns to B2a-ii leaves **1 of 5**, not the claimed 2 of 5.

### N2 — Two open entries describe the same boot-key backup with opposite owning phases and no cross-reference
`seedhammer-bootkey-24word-backup` (:21) is owned "**before any further OTP work**"; F-65 (:704) is titled "back up the SH2 boot signing key", owned "**after the encrypted-payload cycle ships; NOT during it**", and independently re-derives the same encoding ("256 + 8 checksum = 264 bits = **exactly 24 words**"). Neither references the other (`grep -rn seedhammer-bootkey-24word-backup --include=*.md .` → 1 hit total); F-65 cross-references only F-66. The two parkings point in opposite directions relative to the phase being planned, so the register answers "is this owed now?" differently depending on which entry is found first — and the earlier trigger could fire at any time, since two OTP slots remain and F-65 itself proposes burning slot 2. **Fix:** merge into one entry under F-65's (later, better-reasoned) parking; fold the slug's repair-search and label-plate detail in and close it with a pointer, preserving "before any further OTP work" as a *condition* on the surviving entry.

### N3 — Census (measurement, no fix): the shape of the register decides whether per-phase reconciliation is possible
65 distinct entries under `## Open` (67 headings − 2 retained duplicates for F-59 and F-62); 11 bannered → **54 genuinely open**; 4 more done-in-code → **50 carrying real owed work**. Parkings of the 54, verified as an exact set partition (18 buckets, 54 unique, no leftovers, no strays): **B2b — 8** (F-87, F-88, F-89, F-90, F-93, F-94, F-96, F-99); B2a-i — 2 (both done); B2a-ii — 1 (done); split owner — 1 (F-80); after B2b — 1 (F-76); before the release tag — 3 (F-85, F-92, F-98); after the cycle ships — 2 (F-65, F-66); ownerless/explicit-none — 6; "opportunistic" — 1; no phase recorded — 6; GUI/font cycle — 4; Phase D (merged) — 2; Phase A cleanup — 1; after Phase D — 1 (done); O1 — 2; hardware-bringup (passed) — 2; own gated cycle — 3; conditional/event-triggered — 8.

Only **8 of 54** are parked on the phase being planned. **20** are parked on labels that are not phases at all and can never come due (ownerless 6 + opportunistic 1 + none-recorded 6 + GUI/font 4 + own-cycle 3 — note: the census's own prose said 16; the arithmetic is 20, and the buckets are the authority). A further **5** are parked on phases already closed (Phase D ×2, hardware-bringup ×2, after-Phase-D ×1), plus 3 on B2a-i/B2a-ii. That structure is what produces I4, M11, M12 and N1. **Fix:** none — re-run this as a script at each phase entry; entry boundaries are mechanical (`^### ` and `^- **\``) and the owning-phase string is one grep. (`grep -rli 'owning phase' scripts/ .github/` → no hits, so nothing automated depends on the string today.)

---

## WHAT THIS SWEEP DID NOT COVER

Named honestly, because these are the areas where a clean result here means nothing:

- **The unwind mechanism** and the `runWithFlow` / `wipeGuard` design — round 0 (`1d8b3a2`, `a97e65d`) owns it, and the round-1 re-review of fold `da225c0` was in flight while this ran.
- **The idle timer's own correctness** — the state machine, `idleTimeout`, the merged clock, `wipeNowHook`, `maxRunFrames`, the §10.2.4 warn/wipe schedule.
- **The harness** (`gui/run_harness_test.go`) and **Tasks 1-5's test adequacy** — round 0's test-adequacy lens owns it. Where an entry above touches a Task 1-5 artefact (M1 on Task 1's import set, M2 on Task 2.3's missing mutation row, I3 on the mutation tables' *mechanical appliability*), it is a gate-coverage or bookkeeping defect, not a re-derivation of whether those tests work.
- **The §10.2.4 row-1 warn@3:00 vs warn@2:30 ambiguity** — F-99, blocking Task 8, awaiting operator sign-off. Resolved by `7c3a625` after this sweep's brief was written.
- **Whether the amended §10.2.4 is *correct*** — nobody covered it, which is M10 and the reason it is listed rather than answered.
- **Any execution of the plan's Go beyond the build gate's TIER 1** — `gui/run_flow.go:117/:127` (`ctx.wipe`, `ctx.keepAwake` undefined) remain the gate's disclosed blind spot, and M1 shows that blind spot is one item larger than the plan states.

---

## Refuted candidates (recorded so they are not re-raised)

- **f89-in-neither-plan-table** — F-89, a B2b-owned design constraint, appears in neither the coverage table nor "What B2b does NOT cover"
  - _refuted:_ Both halves fail against the artifacts. (1) F-89 IS in the coverage table — row :70 is "| the unwind: session loop + discard guard | ✅ Task 3 | — |", and F-89 is precisely the unwind constraint ("B2b's idle wipe MUST unwind the flow, not just call p.Wipe()"). The plan states the identity at :481, inside Task 3's design — i.e. at the exact place that table row points a reader. The claimed harm ("a reader cannot tell whether it was handled, forgotten, or judged out of scope") therefore does not occur: one grep for F-89 in the plan returns a line naming Task 3 and asserting satisfaction, which is what a reconciliation sweep needs. The finding concedes this itself ("F-89 happens to be satisfied by Task 3"). The table's convention is content-names for architecture rows, IDs only where the row IS a follow-up (F-93/F-87/F-96/F-88/F-94/F-90/F-76/F-80); the unwind is architecture. (2) The F-80 half contradicts what F-80 records. F-80's heading carries a 2026-08-08 operator amendment that supersedes the per-bullet markers for the exact bullet cited: "the layoutMainPager pixel pin does **not** [go to B2a-ii], because a real pin needs a rasterising check, which is F-78's work" — and F-78's own owning phase is "ownerless residue; a font cycle, not a feature cycle". So deferring it "after B2b" tracks its recorded owner rather than defying it. The PlateIndex bullet carries NO owning-phase marker at all; the finding elevates a rationale clause ("keeps B2 from re-deriving it") into a recorded owner, and F-80's amendment note counts exactly three explicitly-owned bullets (verified: grep -c = 3), PlateIndex not among them. The two the operator did assign went to B2a-ii, not B2b. Throughout, the finding silently reads the bullets' "B2" as "B2b", though B2 was split into B2a/B2b (0f19d6e) and F-80's heading records that split. (3) Its one stated count is wrong: it calls :66-78 "the eight-row coverage table"; recomputed, that range holds 13 pipe lines = 11 data rows (8 ✅ + 3 deferred). Residual is at most a Nit: tagging row :70 with "F-89" would make the table greppable by ID like its neighbours — consistency polish, not a gate item.

- **warning-scroll-overdue-and-double-owned** — OVERDUE: `seedhammer-warning-scroll-untouchable`'s owning phase has closed at least five times, and F-95 re-parks the same work on a different phase
  - _refuted:_ The finding's underlying CODE fact is true, but that fact is F-95's content and is already open and recorded. The finding's own contribution — "OVERDUE, its owning phase has closed at least five times" and "two entries name two different owners" — is what I set out to check, and all three of its pillars fail.

(1) THE OVERDUE COUNT IS FALSE ON THE SLUG'S OWN TERMS. The entry does not schedule an unconditional fix; it ends with "**First: measure** whether any warning string actually overflows at current font/box sizes; if none do today, this is latent and drops to LOW." No overflowing warning existed for most of the window the finding counts: `git log -S'unlockWarnUnauthenticated'` returns exactly one commit, 715c79d (2026-08-07, B1 Task 3), so the first warning known to overflow was created FOUR DAYS AFTER the slug was filed. The measurement that lifts it out of LOW was taken 2026-08-09 and IS F-95. So the item was correctly latent/LOW through every cycle the finding indicts, became actionable 08-07, was measured 08-09, and was re-parked the same day. That is deferred, not overdue.

(2) THE "FIVE PHASES" IS A HAND-COUNT THAT TRIPLE-COUNTS ONE PROGRAM. `git merge-base --is-ancestor` returns YES for 9b7ad7a→f6cb8d3 and YES for 80d4d31→f6cb8d3: passphrase Phase C and Phase D are sub-phases OF the BIP-39-password program, not three separate cycles. And all three, plus the sizeproof cycle (f176190/23b171e, 2026-08-06), predate 715c79d entirely. The residue is B1 / B2a-i / B2a-ii — and B2a-ii is the cycle that FOUND it, measured it, and pinned the fit with `TestUnauthenticatedWarningFitsThePanel` (b9acc63, "gui: pin that §10.2.3's 'Do not continue.' still fits the panel").

(3) IT IS NOT DOUBLE-OWNED. F-95's heading reads "owning phase: the GUI/font cycle, with F-78 and `seedhammer-warning-scroll-untouchable`" — it names the slug as a CO-SCHEDULED COMPANION on one cycle, which is consolidation, not a rival owner. The only divergence is wording ("next fork GUI cycle" vs "the GUI/font cycle"). And "neither owner is a phase in this project's history" is wrong: F-78 (:902) and F-86 (:1474) park on that same font cycle, and F-78 states the project's convention explicitly — "a font cycle, not a feature cycle" — which is exactly why the feature cycles the finding counts do not discharge it.

(4) THE HEADLINE FIX IS ALREADY IN THE DOCUMENT. The finding asks to "State in F-95 that restoring `fadeClip` before shortening the copy is forbidden." F-95 already says it verbatim under "**What is still owed, and the order matters.**": "Restoring `fadeClip` *without* shortening the copy makes this **worse**: it would begin enforcing the 19-px overflow the stub currently hides, silently removing the instruction. So either shorten the copy ... **first**, or give `Warning` a touch scroll ... — then restore the clip."

What genuinely survives is small and already filed. One real detail the finding got right that is worth passing on separately: the release-tag prerequisite at plan:1045 names only Task 8 and F-85 — but that is a scheduling gap in a plan whose "What B2b does NOT cover" section deliberately excludes GUI/font work. Separately and more interestingly, the B2b plan contains ZERO occurrences of "scroll" or "Warning" (grep returns nothing) even though its OWN recon and consult docs record the constraint as binding on the new §10.2.4 screen — `RECON_b2b_idle_timer_surface.md:354,514` and `CONSULT_b2b_idle_timer_design.md:361` ("Copy constraints: short enough to need no scrolling (F-95…)"). That is a different, better-founded finding than the bookkeeping one under review, and it is not what this finding claims.

- **keyboard-page3-overdue-phase-d** — OVERDUE: `seedhammer-keyboard-page3-never-rendered` is owned by Phase D, which merged, and the gap is still measurably open
  - _refuted:_ The load-bearing claim — "Page 3 ... is still rendered by no test" at a01b666 — is false. The reviewer checked only the two tests the 2026-08-03 follow-up happened to name and did not search the rest of the `gui` package, which acquired a whole new keyboard test file the day AFTER Phase D merged. `gui/text_keyboard_test.go` (first added 2026-08-04 in commit 691b167, a descendant of the Phase D merge 80d4d31 — `git merge-base --is-ancestor` confirms) contains `TestTextKeyboardEveryKeyReachableByTouch`, which lays the keyboard out through `tkScreen` at the real 480x320 `sh2DisplaySize`, cycles `for range ppPages` (all four pages), renders a frame per page via `h.next`, and calls `h.point` on EVERY key of each page — a helper that `t.Fatalf`s if the target "is not drawn", lies off-panel, or is covered. I replayed that exact loop with a probe test in a scratch copy and captured the frame text: page 3 renders as `%*<>[]{}\^`|~abcspaceshownl0Text` — all 13 of `ppPageSymbols2`'s glyphs inked, 0 missing. `TestNewlineKeyTypesANewlineByTouch` cycles and renders all four pages too, and `TestTextKeyboardFunctionRowFitsPanel` checks page 3's grid and function-row width against the panel for BOTH `NewPassphraseKeyboard` and `NewTextKeyboard`. That is strictly stronger than the "render smoke test for page 3" the follow-up asked for, and it defeats exactly the risk the follow-up named (a future glyph or layout change breaking page 3 silently), since both keyboards are the same `*PassphraseKeyboard` widget built from the same `ppPages` and laid out by the same `Layout`. The two tests the finding quotes are described accurately (`k.page = 2` at :175; the struct walk at :215-238) — but they were never the whole test corpus, and citing them as if they were is the error. What is left after refutation is bookkeeping only: the entry is indeed still under `## Open` (Open starts at FOLLOWUPS.md:5, Resolved at :1606) with "Owning phase: Phase D". The correct action is to move it to Resolved citing 691b167, not the finding's "Add the page-3 render smoke test now" — the fix it prescribes would duplicate coverage that already exists and passes. A stale Open entry whose substance is done is a Nit, not an Important. The sibling sub-claim is weaker still: `seedhammer-address-keyboard-inherits-4th-page` records its owner as "Phase D **or any later GUI cycle**", an explicitly open-ended owner, so by the project's own rule (overdue = owning phase has *passed*) it is parked, not overdue.

- **non-phase-owners-can-never-come-due** — Three open entries are parked on labels that are not phases, so they can never come due — including one inside the code B2b restructures
  - _refuted:_ The finding's headline number is wrong by an order of magnitude and its urgency argument is contradicted by the plan's own file inventory. "Three open entries are parked on labels that are not phases" — measured, it is 35 open entries, because trigger-condition and work-stream owners ("before the release tag", "hardware-bringup", "the next engraving cycle", "ownerless residue") are this file's established convention and are explicitly sanctioned by /scratch/code/CLAUDE.md's ownerless-residue rule. Singling out 3 of 35 makes this a matter of taste about labelling style, not a defect. "There is no cycle named GUI" is false as applied: three sibling entries (:35, :49, F-95 at :1083) schedule the same fork GUI cycle, and F-58 reconciles with them; the finding flagged F-58 but not its siblings. The "pointer into a closed entry" at :31 dereferences cleanly, since the resolved :29 retains "Owning phase: any later engraving cycle". Most decisively, the argument that B2b makes F-58 urgent fails on the facts: B2b writes gui/run_flow.go, gui/run_harness_test.go, gui/wipe_warning.go, gui/wipe_guard.go and named fragments in gui/gui.go and unlock_session.go, and modifies none of F-58's five implicated sites — gui/event.go is cited twice and edited never, and passphrase_keyboard.go, freetext_flow.go, InputTracker and Router.Next return zero hits in the plan. Two of the changes the finding attributes to B2b are misattributed: the discard guard is `if wiping { continue }`, a frame skip during the unwind, and the `!a.idle.active` gate on Router.Events is pre-existing at gui/gui.go:2996-2997 inside a body Task 1 moves verbatim. The plan also already reasons explicitly about EventRouter.Reset leaving r.pointer state behind, which is why it builds a fresh Context per session. The one true residue — that "What B2b does NOT cover" omits F-58 — is not a gap, since that section lists encrypted-payload-cycle items and F-58 is one of 35 open out-of-stream follow-ups, none of which are listed.

- **task8-no-platedone-tap** — Task 8 never observes round 0's A-I1 fix — the plate-done screen silently eating the operator's first tap
  - _refuted:_ The observation is factually true but the inference is backwards: the proposed hardware check cannot distinguish fixed from broken, so adding it would buy false assurance, not coverage.

(1) `a.idle.active` is RECOMPUTED from `a.idle.start` on every single loop iteration (plan lines 759-765: `idleWakeup := a.idle.start.Add(idleTimeout); idle := now.Sub(idleWakeup) >= 0; if a.idle.active != idle { a.idle.active = idle }`). The armed edge also sets `a.idle.start = now` (line 740), which mutation row 4.3-4 leaves in place. Trace of that mutant at the cut-end iteration: armed edge → `idle.start = now`, `active` stays true → the `if !a.idle.active` gate (line 745) drops the events **of that one iteration** → four lines later `idle` computes false, so `a.idle.active` is set false → the branch at line 767 is not taken and normal plate-done content is drawn. `active` self-clears inside the same pass. An operator who returns to a finished plate and taps is tapping many iterations later, when `active` is already false — the tap is accepted on the first press **with or without** the fix. The finding's own suggested wording ("tap once — the tap MUST be accepted on the first press") therefore passes on the mutant. Same trace for the session-head variant (row 4.3-5): `a.idle.start = time.Now()` at line 649 makes the first iteration recompute `idle=false`, so only that first iteration's events (there are none — the wipe fires after 3:30 of no input) can be dropped.

(2) The instrument ranking is inverted. Only the synctest harness can hit the one iteration that matters: the plan's own technique at lines 833-835 injects `p.tap()` **from inside `onDraw`**, i.e. at a chosen frame. A human at a capacitive panel cannot deterministically land a press in the single event-loop iteration containing the armed edge. The harness is not "least likely to catch" this — it is the only thing that can.

(3) There is no touch-path change for hardware to regress. `if !a.idle.active { ctx.Router.Events(d, evts...) }` is pre-existing, unmodified upstream code (fork a01b666, gui/gui.go:2996-2997); the plan copies it verbatim and states at line 819-821 "when armed is false the event loop is byte-identical to today". The harness `tap()` sends the same PointerEvent press+release pair the upstream helper documents as "what the touch driver produces for a fingertip" (gui/start_screen_touch_test.go:46-51).

(4) The line is not unpinned: test 4.1 asserts "a tap on the plate-done screen reaches the flow" (line 838-840), rows 4.3-4 and 4.3-5 name both mutants, and Task 7.2 makes a surviving mutant blocking. Task 8's own preamble scopes it to what only hardware can show ("the first time `ctx.Done` is ever true on the real machine"), and 8.1 already requires the machine be "still usable" after the wipe — the post-wipe interaction the finding wants added to 8.1.

Residual: at most a Nit-grade wording preference, and even that is negative-value here, since the clause as written would report GREEN on the exact mutant it claims to catch.

- **ctxb-reset-row-unkillable** — Task 7 has an unclearable blocker: the `ctx.B.Reset()` mutation row names a killing test that cannot be written from package `gui`
  - _refuted:_ The finding's supporting facts are all correct, but its load-bearing inference — "Package `gui` therefore has no way to read `refs`" — is false, and with it the whole "unclearable blocker / unbudgeted change to package `op`" conclusion collapses.

Read-only reflection on unexported fields is legal Go from any package: `reflect.Value.Len/Cap/Index/IsNil/Slice3` all work on a `flagRO` value; only `Interface()` panics. I ran this against the REAL `seedhammer.com/gui/op` at fork a01b666, from a package that is not `op`, and it read `Buffer.refs`'s length, capacity, and the nil-ness of the abandoned backing array — discriminating the mutant from the clean code with no accessor, no `export_test.go`, and no edit to package `op`:
  MUTANT (no Reset): len=1 cap=1 backing[0]==nil? false
  CLEAN  (Reset ran): len=0 cap=1 backing[0]==nil? true
Note `Slice3(0, cap, cap)` is what makes this a real scrub assertion rather than a length check — it reaches past `refs[:0]` into the abandoned backing array, which is exactly what `clear(b.refs)` (op.go:376) is there to zero.

The seam to hold the abandoned buffer also already exists in the plan: the harness's per-frame `observe`/`onDraw(o op.Op, txt string)` (plan lines 275-283, 618) hands the test the drawn `op.Op`, and `op.Op` embeds `op{ r ops; buf *Buffer; nops int }` (gui/op/op.go:43-47) — so a `package gui` test can capture the session-1 buffer pointer and assert on it after the restart. The technique also has in-repo precedent: `gui/engrave_duration_test.go:54-77` uses reflect for exactly this kind of structural pinning.

Residue after refutation: only the secondary "Task 3.1-3.4 never asks for such a test" point, and that is not specific to this row — the "two-wipe test" named in the adjacent row (line 519) is likewise not enumerated as a numbered step. The table at step 3.3 IS the requirement statement for its named killers; treating one row as unbudgeted and not the other is inconsistent. At most a Nit, not Important, and it is not what the finding was reported for.

Recommended fix (a) — "add a tiny exported accessor to `gui/op`" — would be an unnecessary widening of shared firmware API surface for a need that the stdlib already covers.

- **green-drops-race** — The green criterion drops `-race`, which B2a-ii's green included — in the phase that adds all the new concurrency
  - _refuted:_ The finding's load-bearing premise is false. It asserts B2b "drops" `-race` from a bar "which B2a-ii's green included," making the plan's "Carried forward unchanged from B2a-ii" untrue of that row. But B2a-ii's plan never had a `-race` row, so nothing was dropped and the carry-forward statement is accurate.

Three checks refute it:

1. `grep -c -- "-race"` over both prior plans returns 0 and 0. The string does not occur in `IMPLEMENTATION_PLAN_..._phaseB2a_i.md` or `..._phaseB2a_ii.md`. Across all of `design/*.md`, `-race` appears exactly ONCE, and it is the CONTINUITY line the finding itself cites.

2. B2a-ii's own green table (`..._phaseB2a_ii.md:91-97`) has five rows — `CGO_ENABLED=0 go test ./...`, `go vet ./seal/`, `go vet ./gui/`, `gofmt -l <touched>`, TinyGo device build. B2b's table has six: the same five plus `GOARCH=386 go test ./seal/ ./bip39/`. B2b's green criterion is a strict SUPERSET of B2a-ii's. It adds a row and removes none — the opposite of the claimed weakening.

3. The B2a-ii lens8 completeness report settles what `-race` actually was in that phase. Its §4 "Modalities never run" lists `-race` among things "Never executed by anyone before this report," and §M3 runs it ad hoc as a reviewer gap-fill: `CGO_ENABLED=1 go test -race -run 'TestUnlock|TestSecret|TestPlateList|TestSealBlob|TestSealed' ./gui/ ./seal/`. A filtered one-off a reviewer ran to close a coverage gap is not a standing green criterion. The CONTINUITY:307-308 sentence is a post-merge sweep record of `main`, not a phase gate definition — and note the other item in that same sentence, `GOARCH=386`, IS in B2b's table.

The runtime half of the finding does reproduce: I ran `nix develop --command go test -race ./gui/` at a01b666 → `ok seedhammer.com/gui 169.534s`, exit 0, 2:49 wall. So adding the row is genuinely cheap and, given Task 1's `synctest` bubbles and the plan's own comments at :266-269 and :834-835 that tapping from another goroutine "is a data race," it would be a reasonable strengthening. But that is an enhancement suggestion, not a defect: the plan states nothing false, omits nothing it claims to carry, and its gate is stronger than the phase it inherits from. Downgraded from Minor to Nit and does not gate.

- **f90-dissolution-leaves-ms1-wipe-unquantified** — Dissolving F-90 item 2 is correct, but items 1/3 are NOT separable from B2b's own claim: on the ms1 arm the unwind has nothing to unwind, and neither list says so
  - _refuted:_ The finding's numeric measurement is correct but load-bearing-irrelevant, its mechanism claim is refuted by the code, and the disclosure it says is absent is present in the plan.

1. "On the ms1 arm the unwind has nothing to unwind" is FALSE. The zero-defer count in `unlockEngraveCodex32` (162-198) is real, but the wipe defer that the unwind fires is not in either arm — it is arm-agnostic, one frame up, in `unlockSecretPlate`: `defer func(){ p.WipeSecretAt(i) ... }()` at gui/unlock_session.go:109-110, under the comment "The wipe is a defer registered before anything can return … 'by any route' a property of the code rather than of the author remembering every branch". Both arms are called from inside that defer's scope (:145-150). The finding's own `fix` text concedes this ("the wipe removes seal's record buffers (WipeSecretAt …)"), contradicting its headline.

2. The vector-F argument inverts the actual behaviour. `ChoiceScreen.Choose` is `for !ctx.Done { … }` / `return 0, false` (gui/gui.go:1448, :1497), so on the unwind every *not-yet-offered* secret record's `unlockSecretPlate` returns immediately and its defer wipes a LIVE buffer. Task 8.1 is "Seal vector F, load, unlock, and walk away" — i.e. idle at the first ChoiceScreen, where all three ms1 records are still live and all three are zeroed by the unwind. That is the maximally-effective case for the unwind on the ms1 arm, not the empty one.

3. The residual true fragment is narrow: only in the two armed post-`clear(rec)` windows (hold-to-start, plate-done) does the ms1 arm's unwind add nothing while the mnemonic arm's adds `defer clear(m)` (:250). The plan already states exactly that residue — plan line 350: "seal.RecordsResident() reads false from the instant a plate is built, while the flow still holds codex32.String, the parsed words, and the plate's SPLINE CLOSURE … The spec therefore FORBIDS that predicate as the timer's key" — and line 382 arms those very screens *because* "they are walk-away states with secrets still held". The code carries the same enumeration at gui/unlock_session.go:155-161 ("codex32.String holds the share as a Go string, and backup.SeedString and the Plate derived from it hold further copies. None can be zeroed"). So "neither list says so" is wrong: the plan says it in the seam that defines what "resident" means, which is where a Task-4 reviewer reads it.

4. The Task-8 over-read: 8.1 asks the operator to confirm warning timing, that a wipe occurred, and that the machine "returns to the main menu and is still usable — not a blank screen, not a reboot". It is a liveness/UX observation and claims nothing about the wipe's reach; nothing in it "ratifies" an unmeasured wipe.

What is left is a request to restate an already-stated fact in the scope table / "What B2b does NOT cover" — editorial redundancy, not a defect, and nothing gates on it. At most a Nit.

Separate observation, NOT this finding and outside my brief to adjudicate: design/FOLLOWUPS.md:1329 records F-90 as "(owning phase: B2b)" and ":1354 "Three things close it, all B2b-owned", while the plan's scope table (:77) and :1041 defer items 1 and 3 to "own cycle" with no amendment to the follow-up entry. That is an owning-phase reconciliation gap against /scratch/code/CLAUDE.md's burndown rule — but the finding never raises it, and it argues the opposite framing (that the plan's deferral is fine mechanically and only needs a disclosure line).

- **no-task-owns-whole-diff-review-vs-task8-flash** — No task owns the mandatory whole-diff review and the merge, so Task 8 reads as flashing unreviewed firmware onto the operator's machine
  - _refuted:_ The textual observation is accurate but it is not a defect — it faults B2b for conforming to a five-plan-deep house convention, and its predicted failure mode demonstrably did not occur in the identically-structured predecessor.

1. NOT A DEVIATION — it is the convention, across every plan in the feature. I grepped all five phase plans (A, B1, B2a-i, B2a-ii, B2b) for `whole-diff|whole diff|no-ff|merge to .main.|adversarial .*review|before merge`. ZERO ordering statements in any of them. The only two hits anywhere are in B2a-ii at lines 1244 and 1341, both incidental references to what a *past* review found, not sequencing. B2b is not silent where its siblings speak; no plan in this family has ever carried this line.

2. B2a-ii's structure is identical, and it passed a full R0 loop AND its own whole-diff review with nobody flagging this. B2a-ii's last task is `## Task 9 — hardware: the in-situ KDF rate, and end-to-end unlock` (line 1574) — last task, hardware, operator-run, "Watch what you paste" warning, results to `design/HARDWARE_RESULT_<date>_phase*.md`. B2b's Task 8 mirrors it exactly. No review task, no merge task, no ordering line in B2a-ii either.

3. THE PREDICTED FAILURE EMPIRICALLY DID NOT HAPPEN. The finding's scenario is that the operator runs the hardware pass at the end of the task chain before any review. In the immediately preceding, identically-structured phase the opposite occurred: CONTINUITY_2026-08-08b.md §11 "Then, in order" sequences persist-review → merge `--no-ff` → "3. **Task 9 — the hardware pass, operator-run.**", and §13 item 1 records Task 9 as *still outstanding* on 2026-08-09, i.e. deferred past both review and merge. The plan never said to do that; the continuity doc and CLAUDE.md did, and they worked. Plans own tasks; continuity docs own cross-phase sequencing.

4. The rule it says is missing is a standing, auto-loaded, mandatory one. CLAUDE.md:14 step (4) already makes the whole-diff review "**mandatory, non-deferrable**". There is no project STANDARD_WORKFLOW.md that would supersede it. A plan is not obliged to restate a rule that binds every session regardless.

5. THE FLASH CLAIM IS NOT IN THE ARTIFACT. `grep -i -E "flash|sign"` over the B2b plan returns no flashing or signing instruction in Task 8 at all — only the unrelated TinyGo size baseline (line 106) and 8.4's "payload is intact in flash". "The operator flashes a signed build at the end of Task 8's chain" is the reviewer's inference, not the plan's text. Notably B2a-ii's Task 9 *does* carry the flash line ("Flash with `~/bin/sh/sh2-flash`, never `picotool` by hand"); if anything is missing from B2b's Task 8 it is that line, which is not what was filed.

6. Guardrails against the specific hazard are already present. Task 8 is headed "(operator-run)" — explicitly a human handoff, not the implementer's chain — and Global Constraints line 94 reads "**Stage paths explicitly. Never `git add -A`.** One commit per task. Do not push, do not tag." Line 1045 adds "A release tag. Task 8 and F-85 both precede it." An implementing agent cannot merge, push, tag or flash.

7. The "why" is a non-sequitur. It argues silence cost B2a-ii two Criticals — but B2a-ii's plan was *equally* silent and its whole-diff review ran anyway and caught them (§12: "22 commits, 26 files, +5992/−140, merged at `a01b666`", review found 2 Criticals). That is evidence the review is valuable, not evidence that plan-silence suppresses it.

Minor factual overstatement: "real seed material" — Task 8 uses vector F, a test fixture (`ms1`×3 / `mk1`×6 / `md1`×6, per B2a-i plan lines 200-201), on the operator's own dev machine. No funds are at risk in Task 8.

The one thing the finding gets right is that no continuity entry yet sequences B2b's review/merge — but B2b has not been dispatched (plan Status line 3: "DRAFT — R0 not yet run", round 1 in flight), and B2a-ii's sequencing was likewise written at dispatch time, not at authoring time. The proposed one-sentence fix is harmless and mildly useful, but it corrects no error; it restates a standing rule. That is taste, at most a Nit — not a Minor defect in the plan.
