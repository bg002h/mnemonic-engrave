# B2a-i — whole-diff execution review, round 0 (opus)

Artifact: `git diff 78949e7..HEAD` on `feat/encrypted-payload-b2a-i` in `/scratch/code/shibboleth/seedhammer-wt-b2ai` — 3 commits, 14 files, +790/−81.
Implements Tasks 1–3 of `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_i.md` (GREEN over R0 rounds 0/1/2 plus a scoped §3d review).
Persisted verbatim, before any fold. HTML entities from the transport layer have been restored; nothing else is altered.

The reviewer restored the worktree after every mutation; `git status` was clean afterwards.

---

```
VERDICT: 0 Critical / 1 Important / 2 Minor / 2 Nit
```

## [I1] `Probe()`'s false branch is untested — the mutant `return true` survives the whole suite, unpinning §10.1's "absent → invisible"
WHERE: `/scratch/code/shibboleth/seedhammer-wt-b2ai/seal/read_host.go:30-41` (impl); `/scratch/code/shibboleth/seedhammer-wt-b2ai/seal/read_test.go` (the §10.1 test file, which gained nothing)

DEFECT: The diff moves §10.1's present/absent decision off `Read()` — whose no-payload path is covered by a five-case table (`TestFileReaderReportsNoPayloadOnAbsentBlob`, `…WhenTheRegionIsMissing`) — and onto a brand-new `Probe()` that **no test exercises on its false path**. `grep Probe` over all tracked `.go` files finds exactly one production call site (`gui/gui.go:1554`) and three test references, all of which hand it a *valid* vector blob. `TestUnlockPayloadInvisibleWithoutAPayload` — the one test that asserts invisibility — sets `p.payload = nil`, so `r != nil` short-circuits and `Probe` is never called at all. Nothing anywhere asserts that `Probe` and `Read` agree about what "present" means, which is the premise §10.1's asymmetry rests on.

EVIDENCE: Measured, not argued. Applied `func (r FileReader) Probe() bool { return true; … }` and ran the packages that could observe it:
```
$ CGO_ENABLED=0 go test ./gui/ ./seal/
ok  	seedhammer.com/gui	11.009s
ok  	seedhammer.com/seal	12.377s
```
Fully green with the mutant in place. (Restored from a file copy; `git status` clean.) The inverse mutant (`return false`) *is* killed, by `TestUnlockPayloadVisibleWithAPayload` — so the coverage gap is exactly one-directional, which is why it reads as covered.

CONSEQUENCE: §10.1 is normative: *"Present → the unlock entry point appears in the menu. Absent → the feature is invisible."* A future edit to `Probe` — or to `hasMagic`'s bound, or a refactor that drops the `hasMagic` call — makes every machine advertise "Sealed Payload" whether or not it carries one, and no test fails. The operator selects it and gets "Payload unreadable." on a machine that has never been sealed, which is precisely the string §2.2 item 4 trained them to read as *"someone replaced my payload."* The feature's own detection contract is now the least-tested code in the diff.

FIX: Extend `seal/read_test.go`'s existing table rather than adding a new one — the five bodies (`erased flash`, `zeroed flash`, `altered magic`, `shorter than the magic`, `empty`) plus the missing-file case already state exactly what "absent" means:
```go
if r.Probe() {
    t.Errorf("%s: Probe reports present", c.name)
}
```
and one positive assertion (`FileReader{Path: writeRegion(t, vectorNamed(t,"A").Blob(t))}.Probe() == true`) so Probe and Read are pinned to agree. Optionally add the gui half: a `uiFlow` test with a reader over erased flash asserting "Sealed Payload" never appears — that is the §10.1 sentence stated as a test.

## [M1] The §3d fold deleted `DeriveKey`'s fail-closed `nil`, turned it into a silent clamp, and left four comments citing the rule it deleted
WHERE: `/scratch/code/shibboleth/seedhammer-wt-b2ai/seal/crypto.go:54-61`; citations at `seal/pbkdf2.go:108`, `seal/pbkdf2.go:122-123`, `seal/pbkdf2.go:131`, `seal/pbkdf2_test.go:147`

DEFECT: Two halves of one root cause.
(a) Old `DeriveKey(p, s, iterations)` with `iterations < 1` returned `nil` — `pbkdf2.Key` errored and the documented contract was "fail closed; a nil key makes `aes.NewCipher` fail". New `DeriveKey` cannot return `nil` for **any** input: `NewDeriver` clamps `iterations < 1` to `1`, `done` is already `1`, so `Key()` returns a real PBKDF2(c=1) key. An out-of-range iteration count now yields a plausible weak key instead of a refusal.
(b) `crypto.go:47-52` no longer contains the rule; the fold replaced those lines with the `[]byte`-conversion note. Four surviving comments still cite it, one of them quoting text that exists nowhere in `crypto.go` any more, and `pbkdf2.go:131` asserts *"Same reason DeriveKey returns nil rather than panicking"* — which is now false about the function two files away.

EVIDENCE: `crypto.go:47-52` reads verbatim: *"The []byte conversion is an unwipeable copy of the passphrase … iterations ALWAYS comes from the header"*. `grep -n "VALID AES key"` over all tracked `.go` files returns only `pbkdf2.go:123` (the quote) and `pbkdf2_test.go:146` — the quoted source is gone. Traced (a) by hand: `NewDeriver(…, 0)` → `total = 1`, `done = 1`; `Step(0)` runs zero iterations, `done >= total`; `Key()` passes both guards. Reachability: `ParseHeader` bounds `iterations` to `[MinIterations, MaxIterations]` when `ctLen > 0` (`wire.go:161-164`) and `Unlock` returns before `derive` when `!h.Sealed()` (`open.go:200`), so **(a) is unreachable through `Opener.Unlock` today** — no current caller passes a value the wrapper can mishandle.

CONSEQUENCE: A B2a-ii/B2b author reading `pbkdf2.go:131` learns that `seal` has a derive-returns-nil fail-closed behaviour and calls `DeriveKey` with an iteration count from a source `ParseHeader` did not gate; they get a 1-iteration key that opens nothing and reports as a tag mismatch — indistinguishable from a wrong passphrase, ~0 s instead of ~31 s. This is the exact class the §3d review named ("a guarantee can migrate without anyone noticing"), reintroduced one level down.

FIX: Two lines. In `DeriveKey`, restore the guard the fold removed: `if iterations < 1 { return nil }` before `NewDeriver` (leave `NewDeriver`'s clamp alone — a `Deriver` must be constructible). Then repoint the four citations at the surviving statement of the rule (`pbkdf2.go`'s own `Key()` comment) instead of `crypto.go:47-52`, and delete the "DeriveKey returns nil" clause at `pbkdf2.go:131` or make it true.

## [M2] A `nil seal.Reader` now reaches `r.Read()` unguarded — a brick path where the old signature had none
WHERE: `/scratch/code/shibboleth/seedhammer-wt-b2ai/gui/unlock_flow.go:26-27`; dispatch at `gui/gui.go:1592-1593`

DEFECT: `unlockPayloadFlow` takes an interface and dereferences it on the first statement. With `hasPayload == false`, `payloadReader` is a nil interface, and a nil-interface method call panics. The only thing preventing it is arithmetic: `lastNav()` returns `bip85Derive` (7) when `!hasPayload`, `unlockPayload` is 8, and `m.prog` is bounded by `lastNav()` at both wrap sites. Before this diff the same mistake was harmless — `blob` was `nil`, `Inspect(nil)` errored, and the operator saw "Payload unreadable."

EVIDENCE: Traced every producer of `startScreenAction.prog`: `selectBtn` returns `m.prog` (bounded), the debug path returns `qaProgram` (its own case). `s.hasPayload` is written once at construction and never mutated. So it is genuinely **unreachable today** — this is a hardening regression, not a live bug. But the same file's own convention is explicit about structurally-unreachable panics: `crypto.go:77-83` guards `len(iv) != IVLen` because *"this is structurally unreachable — but a panic on a device is a brick, and this costs one comparison"*, and `pbkdf2.go:129-133` guards a nil `mac` on identical reasoning.

CONSEQUENCE: Any future edit that decouples `hasPayload` from `lastNav()` — a tenth program inserted, `lastNav()` made unconditional, a `StartScreen` built elsewhere — turns an invisible menu entry into a firmware panic on a watchdog-less device: the machine is dead until it is re-flashed, mid-session.

FIX: One comparison at the head of `unlockPayloadFlow`, matching the file's own idiom:
```go
if r == nil {
    showError(ctx, th, unlockTitle, "Payload unreadable.")
    return
}
```

## [N1] `label_encrypted.go`'s `record.go` citations went stale by one line in the same commit that wrote them
WHERE: `/scratch/code/shibboleth/seedhammer-wt-b2ai/seal/label_encrypted.go:7` and `:31`

DEFECT: Task 1d shortened `AdmittedRecord`'s doc comment by one line, shifting everything below it. `record.go:214` (cited as "pass 3 … runs only there") is now the comment line *inside* the branch; the branch is at 213. `record.go:217-220` (cited for the string conversion) is now 216-219. Same drift in commit `e42f2aa`'s message.

EVIDENCE: `awk` over the current file: `213: if section == SectionPublic {` … `216: strs := make([]string, len(records))` … `219: }`.

FIX: `record.go:213` and `record.go:216-219`.

## [N2] `plateLabel`'s "Unreachable" default branch is what F-77's own safety argument depends on being reachable
WHERE: `/scratch/code/shibboleth/seedhammer-wt-b2ai/gui/unlock_platelist.go:51-55`

DEFECT: `plateLabel`'s default says *"Unreachable: §10.2.1's allow-list admits only ClassMDMK into the public section and pass 3 sets HRP for every one of them."* True today. But `label_encrypted.go:22-23` and `label_encrypted_test.go:114-116` both justify discarding a grouping failure by pointing at that branch rendering "record N" — and B2a-ii will feed the same function encrypted records where `ms1`, mnemonics and every card in a failed grouping carry `HRP == 0`. The comment will be false the moment the secret plate list is wired, in a phase that will not be reading this file.

FIX: One clause now, while it is cheap: "…unreachable for the public section; reachable for the encrypted one (F-77), where `ms1`, mnemonics and a failed grouping all keep HRP 0."

## What I checked and found sound

- **Plan fidelity, mechanically.** Extracted every ```go block from plan §1b, §1e, §3b, §3c and diffed against the shipped files: `label_encrypted.go`, `label_encrypted_test.go`, `pbkdf2.go`, `pbkdf2_test.go` are all **byte-identical** to the plan. §1c, §1d, §2a, §2b, §2c, §3d match their fragments verbatim. No transcription defects.
- **F-77 call site.** `labelEncryptedCards` runs after the pass-1/pass-2 loop and after the `SectionPublic` block, on a mutually exclusive branch; every rejection path (`wipe(out); return nil, err`) exits inside the loop and never reaches it. `TestGroupingRunsAfterTheAllowList` is untouched and still passes. Public-section labels are provably unchanged (the public branch is `section == SectionPublic`).
- **The index scatter, by hand and by mutation.** Vector C's `ms1` sits at index 0 with cards at 1-5; vector F is `ms1`×3 / `mk1`×6 / `md1`×6 in that order. Mutated `out[at[j]]` → `out[j]`: killed three ways (ms1 at 0-2 gained `k` labels, records 12-14 lost theirs, `mk` count fell 6→3). Mutated the body to a no-op: killed. `cardKey`'s `uniq: i+1` using subset coordinates is sound — uniqueness within the grouping is all it needs, and `chunked=true` keys can't collide with it.
- **The INVERTED test can fail**, measured, and keeps its `cards != 12` premise (`t.Fatalf` at grouping_test.go:149). Its non-card branch still asserts all five label fields zero, which is what caught the scatter mutant.
- **F-79 is a closure, not `defer clear(blob)`** — `unlock_flow.go:47` is `defer func() { clear(blob) }()`. I1 satisfied.
- **The §2d harness claim is true.** Ran the plan's version verbatim against the *correct* implementation: `FAIL — the menu entry never appeared`. `pumpUntil` sends no input, so the carousel never leaves "Backup Wallet". The repair is sound and **both** of its assertions are live: mutating startup to `Read` instead of `Probe` fails on `startup never probed`; mutating it to probe *and* read fails on `startup called Read 1 times`.
- **Menu conditionality and the pager.** `unlockPayload = 8`, `qaProgram = 9`, guarded by `[qaProgram - unlockPayload]struct{}{}`; both wrap sites bound on `lastNav()`; `layoutMainPager` draws `lastNav()+1` dots, pinned to literal 8 and 9 by `pagerDots`. Present-but-corrupt still shows the entry then reports "Payload unreadable" — and the mangled-blob test's "bad magic" case is now the only coverage of the new `r.Read()` error branch (its old target, `ErrBadMagic` via `Inspect`, remains covered at `open_test.go:206` / `wire_test.go:76`).
- **`Probe` bounds and semantics.** `XIPReader.Probe` maps `clampRegion(len(Magic))` = 8 bytes — right bound, and `unsafe.Slice` is a mapping. `FileReader.Probe` returns false (never errors) for absent, short, unreadable and non-magic regions, consistent with `Read`'s `ErrNoPayload`. Adding `Probe()` to the interface breaks no implementation: `cmd/emu` returns nil, `cmd/controller` returns `XIPReader{}`, `cmd/sealread` calls `XIPReader{}.Read()` directly.
- **`crypto.go`'s rewritten header is accurate**, checked against the real packages: `bip85/bip85.go:7` and `nonstandard/parse.go:7` import `crypto/hmac`; `slip39` imports both; `bip39` and `slip39` import `golang.org/x/crypto/pbkdf2`, and v0.52.0's `pbkdf2.go:15` does import `crypto/pbkdf2`. `go list` confirms `.Imports` carries no `crypto/pbkdf2` and `.TestImports` does.
- **`Deriver` arithmetic and lifetime.** `done=1` after `NewDeriver`'s U_1, `Step` runs to `total`, so `c` iterations total. The `Sum(d.u[:0])` aliasing is correct: `cap == sha256.Size` exactly, so neither `inner.Sum` nor `outer.Sum` can reallocate, and `Write` consumes the previous U before `Sum` overwrites it. `Key()`'s `d.total == 0` clause catches the zero value; `Wipe` resets `done` but not `total`, so post-Wipe `Key()` is nil for every constructible `Deriver`. Measured the "allocates nothing per Step" claim: `AllocsPerRun` = **0** for both `Step(1)` and `Step(1000)`.
- **Nothing else assumed the stdlib.** No test or caller depended on `DeriveKey` returning nil; `Opener.Unlock` is unchanged and its nil-key path still fails closed through `aes.NewCipher`.
- **No drive-by hunks.** `gui/gui.go` has exactly two hunks, both plan-mandated. Every one of the 14 files traces to Task 1, 2 or 3.
- **Blast radius of F-77 in this phase is zero:** `labelEncryptedCards` runs only from `AdmitSection(…, SectionEncrypted)`, which only `Opener.Unlock` calls, which B2a-i never reaches — consistent with the byte-identical TinyGo image the commit reports.
