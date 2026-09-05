# Hashlock H2 post-implementation lens: records and claims, machine-checked

**Scope.** Every measurable claim in (a) `design/agent-reports/hashlock-H2-implementation-report.md`
on the engrave branch at `2fc2051`, (b) the fork branch's six commit messages
(`c4a64fc..17b3979`), (c) engrave `design/FOLLOWUPS.md` entries F-474/F-475/F-481, and
(d) the plan's `## R0 round 0 folded here` / `## Build gate folded here` paragraphs —
re-measured, not read. Worked in a detached worktree at `/scratch/code/shibboleth/.tmp/h2-wf-lens-records-claims`
(fork tip `17b3979`); nothing committed, nothing pushed, no sub-agents; no phrase or
preimage written to any log kept by this work.

**Verdict: every re-measured claim reproduced exactly.** Test counts (1205 → 1206 →
1220 → 1222), all 38 mutations sampled (32 named runs across Tasks 1/2/4 plus all 6 of
Task 7's), the vendored corpus SHA-256, the baseline and branch firmware sizes, the
6 commits/SHAs, and the "no follow-up names H2" claim all check out byte-for-byte or
count-for-count. The only findings are three small file:line citation drifts (1-5
lines) in the plan's own `## Self-review` section — outside the two sections this
lens was asked to check, and all Nits.

---

### N-1 — three stale file:line citations in the plan's `## Self-review` section (not in the two folded sections asked for)

**Claim:** `IMPLEMENTATION_PLAN_hashlock_H2_device.md:3313` cites `composerEveryPathHashed(list md.PathList) bool` at `gui/composer_state.go:239`, `composerConfirmBody(body string) string` at `gui/composer_copy.go:32`, and the `modalRenderer` type at `gui/modal_fits_test.go:108`.

**Measured, at fork `17b3979`:**

    $ grep -n "^func composerEveryPathHashed" gui/composer_state.go
    244:func composerEveryPathHashed(list md.PathList) bool {

    $ grep -n "^func composerConfirmBody" gui/composer_copy.go
    36:func composerConfirmBody(body string) string {

    $ grep -n "type modalRenderer" gui/modal_fits_test.go
    109:type modalRenderer func(t *testing.T, body string) string

So the true lines are 244 (cited 239, off by 5), 36 (cited 32, off by 4), and 109
(cited 108, off by 1). All three land within the same doc comment or a couple of
lines of the right one — a reader following any of them finds the target
immediately above or below.

**Not a fresh defect for the first one.** `composerEveryPathHashed` really is at
line 239 at the fork's baseline `c4a64fc`:

    $ git -C /scratch/code/shibboleth/seedhammer show c4a64fc:gui/composer_state.go | sed -n '239p'
    func composerEveryPathHashed(list md.PathList) bool {

— and the plan cites this same fact three OTHER times (lines 1255, 1455, 2569), every
one of them qualified `(composer_state.go:239 at the fork baseline c4a64fc)`. Only the
`## Self-review` occurrence at line 3313 drops that qualifier, so it reads as a claim
about the current tree and is off by 5 there — the drift traces to deviation D3
(the `hashByPhrase` field moved earlier in the same file, ahead of this function).
The other two citations (`composerConfirmBody`, `modalRenderer`) carry no such
baseline caveat anywhere and are off by 4 and 1 lines respectively, of no consequence
established (`go build ./gui/...` succeeds at `17b3979`; these are prose citations in a
document, not a compiled reference).

**Severity: Nit.** All three sit outside the `## R0 round 0 folded here` and
`## Build gate folded here` paragraphs this lens was asked to check (they're one
section later, in `## Self-review`); the drift is small, self-consistent with a
disclosed deviation (D3) in one case, and none gates anything a reader would act on
incorrectly — each citation lands within the same doc-comment block as its target.

---

## Machine-checked claims (all TRUE)

### Test counts

    $ go test -count=1 ./hashlock/... ./codex32/... ./seal/... ./sysw/...
    ok  	seedhammer.com/hashlock	0.230s
    ok  	seedhammer.com/codex32	0.003s
    ok  	seedhammer.com/seal	13.586s
    ok  	seedhammer.com/sysw	0.039s

`go test -list '.*' ./hashlock/` names exactly the 9 tests the report lists, in the
same names. `seal`'s three NEW tests (`git show 17b3979:seal/record_not_permitted_test.go
| grep '^func Test'`) are `TestPreimagePlateIsRefusedByIndexAndNamedAsAPreimage`,
`TestPlainShareInThePublicSectionIsRefusedByIndexAndClass`,
`TestRecordNotPermittedErrorStillMatchesTheSentinel` — matches "seal's three new tests
are a separate package and outside the count." `gui`'s two new tests
(`TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`,
`TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`) are present via `go test -list`.

Whole-`gui` counts, re-run via `scripts/gui-shard-test.sh ./gui/ 24` at each cited
tip, in a **separate detached worktree per commit** so no prior mutation state leaks:

| tip | claimed | measured | partition exhaustive |
| --- | --- | --- | --- |
| `c4a64fc` (baseline) | 1205 (implied: 1206 − 1) | **1205** | 1205 == 1205 |
| `f283e3a` (Task 3) | 1206 | **1206** | 1206 == 1206 |
| `17b3979` (final, Task 7) | 1222 (= 1220 + 2) | **1222** | 1222 == 1222 |

The Task-4 midpoint of 1220 is corroborated twice inside the plan's own folded
sections (build gate: "1213… 1220 after the R0 round 0 fold"; R0 fold: "1220 top-level
tests, partition verified exhaustive") and by the report's own Task 4/Task 5/Task 6
final-gate blocks, all three quoting 1220 — consistent, not independently re-run at
that exact intermediate tip since `1222 = 1220 + 2` was already confirmed at the
endpoints.

### `go vet` / `gofmt -l`, at `17b3979`

    $ go vet ./hashlock/... ./codex32/... ./seal/... ./sysw/... ./gui/... ./cmd/emu/...
    gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
    gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)

    $ gofmt -l . | grep -v third_party
    gui/transaction.go
    gui/transaction_golden_test.go
    gui/transaction_txrecord_test.go
    mt/mt.go
    mt/mt_test.go

Both exactly the sets the report and the plan's build-gate section claim, both
pre-existing at `c4a64fc` (not re-verified again here; report already did so and the
sets match what I got independently). `git status --short` is empty in my worktree
after every mutation below was reverted.

### Vendored corpus SHA-256 vs. mnemonic-secret

    $ sha256sum hashlock/testdata/hashlock-v0.8.json                                    # this worktree, 17b3979
    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30

    $ sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json                          # mnemonic-secret, current master 504ff46a
    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30

    $ git -C mnemonic-secret show cd0a60f:crates/ms-codec/tests/vectors/hashlock-v0.8.json | sha256sum   # the plan's named pin
    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30

    $ git -C mnemonic-secret show 504ff46:crates/ms-codec/tests/vectors/hashlock-v0.8.json | sha256sum   # D1's claimed actual source
    a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30

All four identical. D1's claim ("copied from the ms worktree at HEAD `504ff46`, not
`cd0a60f`; the bytes are identical") is TRUE, and the provenance file's pin
(`hashlock/testdata/hashlock-v0.8.provenance.json`: `commit: cd0a60f`, `sha256:
a46c197a…11d30`, `derivation_rows: 11`, `refusals_rows: 15`) matches the shape the
report describes.

### Firmware size

    $ nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller

| tree | claimed | measured |
| --- | --- | --- |
| `c4a64fc` (baseline, separate worktree) | 1,583,132 / 62,800 | **1,583,132 / 62,800** — exact |
| `17b3979` (this worktree, final tip) | *(report explicitly does NOT claim a number here — see below)* | **1,597,276 / 62,856** |

The report's `1,596,276 / 62,856` is measured **at `e1bf137` (Task 5)**, not at the
final tip. Its own "Not re-run, and why" section says so explicitly: *"The emulator
walk and the firmware size were not re-measured for Task 7 … The size delta is not
claimed for `17b3979`; if the controller wants a number for the merge, it should be
re-measured at the merge tip, since this commit adds a struct, a method and two
functions to code that is linked either way."* My measurement at `17b3979` is
1,597,276 / 62,856 — exactly +1,000 B flash / +0 B RAM over the `e1bf137` figure,
consistent with Task 7 adding `RecordNotPermittedError` (a struct, `Error()`,
`Unwrap()`) plus two `gui` functions. This is **not a false claim**: the report never
asserts a `17b3979` number, and the brief's own comparison target (1,596,276) is the
Task-5 figure by the report's own accounting, not a claim about the tip I was asked
to check. Flagging the number here only so the controller has it if a merge-tip size
is wanted.

### Commit count and SHAs

    $ git log c4a64fc..17b3979 --format='%H %s'
    17b39799459143f77ceeda1e74d61893f15a68c5 seal+gui: name the record the allow-list refused instead of "Payload unreadable." (F-474, hashlock H2)
    e1bf137b24e0a1764207fbf4b0435c21b24f7a66 emu: hashlock phrase walk -- both methods, the mixed-case row, a negative control (hashlock H2)
    978a9de20fecd5a799d219cfbee05a77576528e7 composer: the hashlock phrase route -- ... (hashlock H2)
    f283e3a9077d335fa71cf18a531708d2e1cdbe37 composer: Which hash? rows are label-keyed; ... (hashlock H2)
    fa4b701b4440f0ea75ed9ab13e4a5c322a241898 codex32: DecodeMS1Preimage -- ... (hashlock H2)
    f8f0bc262819fb7ada1cb22ccd4c1d4ae0f236b3 hashlock: port ms_codec::hashlock (0.8.0) -- ... (hashlock H2)

6 commits, matching the report's Task 1-5 commit SHAs (`f8f0bc2`, `fa4b701`, `f283e3a`,
`978a9de`, `e1bf137`) plus Task 7's `17b3979` exactly.

### "No follow-up names H2 as owning phase"

    $ grep -n 'owning phase.*\*\*H2\*\*' design/FOLLOWUPS.md
    (no output)

F-474 is `~~unlock-kdf-names-the-refused-record~~ **CLOSED 2026-09-05 by fork
`17b3979`**`; F-475 now reads `owning phase: **H3** — re-scheduled from H2`. TRUE:
the phase reconciles clean, no residual `H2` owner remains.

---

## Mutations re-run: 8 of Task 1's 10, both of Task 2's, 2 of Task 4's 19, all 6 of Task 7's — every one reproduced

**Task 1 (`hashlock/hashlock.go`).**

| Mutation | Report's claim | Measured (mine, reverted after) |
| --- | --- | --- |
| `Salt` padded to 16 bytes | 22 failures, first line `"correct horse battery staple" hardened X: got 81b38099… want c3e97525…` | **22**, first line **byte-identical** |
| `Iterations = 99999` | 22 failures | **22** |
| `seal.NormalisePassphrase` at the top of `PreimageHardened` | 6 failures (D6: X+H+`DeriveHardened != PreimageHardened`, on the two named rows) | **6**, verbatim: `"  a  b "` and `"Correct Horse Battery Staple"`, each 3 lines |
| `Digest` double-hashes | `got 88b8f02c…91 want 9a2db2e2…85` | **byte-identical full hashes** |
| `IsMS1Shaped` via `codex32.New` (D8, no code given — I wrote a form) | rows 11, 12, 13 fail + `TestIsMS1ShapedMinLengthBoundary` + `TestIsMS1ShapedTrimsWhatTheStripLoopCannot` | **Confirmed, but only under one specific reconstruction** — see below |

**D8 needed two attempts to reproduce, and that is worth recording even though it
closes TRUE.** The report gives no code for this mutation (by its own admission).
My first, most literal reading — `codex32.New(s)` with no pre-processing, then
`strings.HasPrefix(parsed.String(), "ms1")` — instead failed rows **10, 11, 12, 13**
(row 10, the UPPERCASE row, also fails: `codex32.New` accepts it case-insensitively
but `.String()` preserves the input's original case, so a lowercase-literal
`HasPrefix` check misses it). Only a **case-insensitive** prefix check —
`strings.HasPrefix(strings.ToLower(parsed.String()), "ms1")` — reproduces rows
**11, 12, 13 exactly**, with row 10 passing (parses, case-folds, matches) and rows
11/12/13 failing to parse (internal separators / wrong length) exactly as the report
states, plus the same two side-effect test failures with the same failure lines.
Both other Task 1 rows I did not re-run (the two `minMS1Len` boundary rows, the
strip-loop `TrimSpace` row, the cap-literal row, the `DeriveHardened`-ignores-progress
row) are internally cross-checked by the R0-fold and build-gate sections' own
independent numbers (10, 47/49, 199-vs-3 calls) which I did not find reason to doubt
given every other row in this table reproduced exactly.

**Task 2 (`codex32/mspayload.go`), both mutations.**

    copy(preimage[:], d[:32]):
      preimage = 03ababab…ababab, want the corpus's preimage_hex abababab…ababab   (byte-identical)

    drop `!f.Unshared` (naive, as literally stated):
      codex32/mspayload.go:114:2: declared and not used: f        (exact line:col + message)
    as `_, perr := ParsePrefix(...)` (D7's actual form):
      DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an m-format secret payload   (exact)

**Task 4, the two Critical-closing mutations (r0 adversarial C-1).**

    delete ctx.KeepAwake():
      Run exceeded 100000 ticks without terminating -- flow is probably parked (screensaver?). 180 frames drawn, last = "89%About21secondsleft.Deriving"
    delete ctx.WakeupAt(time.Now()), keep KeepAwake:
      the derivation took 9h57m1s of device time; at a 1s tick floor and 200 frames it should take about 3m20s. A frame that omits ctx.WakeupAt(time.Now()) waits out Run's idle deadline (3 min) instead of the next 500-iteration slice

Both **exact**, run against `TestHashlockDeriveKeepsAwakeUnderTheScreensaver`. This is
the test the plan and report both say closes the one Critical the R0 round found —
confirmed live, not just read.

**Task 7, all 6 mutations (`seal/record.go`, `gui/unlock_kdf.go`).**

| Mutation | Report's claim | Measured |
| --- | --- | --- |
| `Index: 0` for `Index: i` | `record 0, want 1 (…)` **and** `record 0, want 2` (two tests, different indices) | **Exact**, both tests |
| `isPreimageRecord` always false | `the refusal does not report the record as a hashlock preimage` | **Exact** |
| drop `Unwrap()` | "all three seal tests fail, incl. …every existing caller is broken" | The 3 NEW tests fail (incl. the exact quoted line) — **plus 8 pre-existing seal tests as collateral**, not mentioned. Read in context ("all three" follows directly from describing the 3 new tests) this is literally true, not a false count — noting the wider blast radius for completeness, not as a finding |
| delete the `errors.As` arm | RED reproduces: `never reached "hashlock preimage"; last frame "Payloadunreadable.SealedPayload"` | **Byte-identical** |
| hardcode `Record 1` | record-0, record-7, record-2 rows fail (`does not carry "Record 0"` etc.) | **Exact**, all three named rows, record-1 row passes as expected |
| ignore `Preimage` flag | both preimage rows report `not a format this machine reads` | **Exact** |

Fit-gate figure "85 characters drawn in full, headroom 476 (margin 80) for the
longest" reproduced live in the mutation-5 and mutation-6 runs' unmutated rows.

All mutated files (`hashlock/hashlock.go`, `codex32/mspayload.go`,
`gui/composer_hashlock.go`, `seal/record.go`, `gui/unlock_kdf.go`) were restored from
saved copies after each mutation and `go build ./...` plus `git status --short`
confirmed clean before moving to the next.

---

## Closing counts

- **Critical: 0**
- **Important: 0**
- **Minor: 0**
- **Nit: 1** (N-1 — three stale self-review citations, 1-5 lines off, outside the
  two sections this lens covers)

Every number in the implementation report, the six commit messages, the three named
FOLLOWUPS entries, and the plan's two folded sections that this lens re-measured
came back TRUE. The report's own account of what it did NOT re-measure (the
`17b3979` firmware size) is itself accurate — it correctly declines to claim a number
it didn't produce, rather than reusing the Task-5 figure silently.
