# S6b P6 implementation report — §4 modal-fit sweep, GATE 4 (F-192)

Worktree: `/scratch/code/shibboleth/wt-s6b`, branch `s6b-pre-flash`, on top of
P5b (`5712052`, "S6b P5b: split GATE 5.1's shared arrow predicate into one per
direction"). This is the last phase of the S6b cycle, run against the frozen
`SPEC_s6b_pre_flash_cycle.md` §4 / `IMPLEMENTATION_PLAN_s6b.md` P6.

Commit: `87e5a5e` "S6b P6 (4): the modal-fit sweep, GATE 4 -- every long modal
this cycle touched" — one file, `gui/s6b_modal_fit_sweep_test.go` (176 lines,
new).

## Scope actually executed

The dispatch brief and `IMPLEMENTATION_PLAN_s6b.md` (authority #2, "it must
sweep every modal this cycle added or changed") both scope P6 to **what S6b
itself touched**, not the whole-firmware backlog F-192's opening sentence
describes ("every other long modal in the firmware carries the same
unmeasured exposure"). F-192's own filed text (`design/FOLLOWUPS.md:6945+`)
confirms this reading explicitly: "the S6b remedy is the fit-gate sweep as
filed: guarantee every long modal fits" for this cycle's own additions — the
firmware-wide sweep is not re-scoped onto P6 by that entry. I followed the
plan's explicit scoping. This is stated here rather than silently assumed, per
the "if the spec is ambiguous... STOP and report it" instruction — I did not
STOP because the plan-level authority is unambiguous, but the ambiguity in
F-192's own wording is worth a future reader knowing about.

## How modal bodies were enumerated

1. `git diff b1479a1..HEAD -- gui/*.go` (`b1479a1` is the fork commit pinned
   at spec line 27), excluding `*_test.go`, restricted to added (`+`) lines
   matching a quoted string of 15+ characters. This is every string literal
   this cycle introduced or edited anywhere in `gui/`. Command and full output
   were run and inspected directly (not hand-scanned) — see the transcript;
   reproducible by any reader with the same `git diff`.
2. Every hit was traced to its call site to determine which **screen shape**
   renders it: `showError`/`showNotice` (→ `showModal` → `ErrorScreen`, the
   `Warning.Layout` body `modal_fits_test.go`'s `errorScreenBody` targets),
   `ConfirmWarningScreen` (the check's other supported shape), or something
   else — `ChoiceScreen`'s fixed `Lead` line, `ppConfirmFlow`'s own
   non-scrolling panel, or a **paged** review/document screen
   (`restoreDocScreen`, `confirmReviewScreen`).
3. Only `showError`/`ConfirmWarningScreen`-shaped bodies are candidates at
   all — F-185's mechanism is specific to `Warning.Layout`'s scroller, which
   (until P5b) was bound to `Up`/`Down`, a control with no reachable affordance
   on real SH2 hardware. Every other shape found is reachable by a button the
   device already has (`Page`/`Button2`) or does not scroll in the first place
   (already pixel-height-gated). This is a **mechanism** exclusion, stated per
   item below, not a length judgement.
4. Among the `showError`/`ConfirmWarningScreen` candidates, a body counts as
   "this cycle changed" when its **bytes** are new or edited by the diff.
   Bytes identical to what shipped before S6b are the pre-existing,
   still-unswept backlog F-192 itself names as broader than this cycle, even
   if the surrounding function gained a new sibling arm or became reachable
   more often.

## Full coverage list

### GATED — `TestS6bModalFitSweep`, `assertModalBodyFits`/`errorScreenBody`

| body | chars | file | why gated | headroom (margin 80) |
| --- | ---: | --- | --- | ---: |
| `multisigVerifyNoSlotBody(true,true)` — R-M's provedInnocent arm | 251 | `gui/multisig_verify.go` | new text, P1/spec §3.2a | 360 |
| F-204 "passphrase entered" arm | 143 | `gui/singlesig_verify.go` | new text, P1/spec §3.2 | 436 |
| F-204 "no passphrase" arm | 72 | `gui/singlesig_verify.go` | bytes pre-existing, but the conditional **selection** is new this cycle | 494 |
| F-199's readback refusal | 67 | `gui/multisig_verify.go:791` | bytes pre-existing; became retry-loop-reachable this cycle (`verifyRefused`→`verifyIncomplete`) | 494 |

All four are **regression pins, green throughout** — no over-budget finding.
The provedInnocent body was already individually gated by P1's own
`TestProvedInnocentBodyPassesTheModalFitClassCheck`
(`gui/multisig_verify_provedinnocent_test.go`), whose own coverage note says
"P6's sweep will re-run over this body too... and state its own coverage
there" — re-included here so this cycle's full coverage reads from one file.

### UNGATED, WITH REASON

- **`multisigVerifyNoSlotBody`'s other two arms** (`passphraseTyped`: 154
  chars; `default`: 152 chars) — bytes and reachability both untouched by
  this diff. Pre-existing backlog, not this cycle's.
- **`"Couldn't derive the bare-seed fingerprint."`** (`gui/singlesig.go`, new
  function `singleSigPassphrasePlateOffer`, P3/§2.6), 42 chars — new text,
  but under half this file's own established gating floor (the shortest body
  `TestModalsThisBlockTouchesAreDrawnInFull` already treats as worth checking
  is 87 chars) and under a tenth of the ~500 chars F-185 measured before this
  panel starts cutting text.
- **`"This passphrase does not fit a plate."`** (`gui/passphrase_flow.go`), 37
  chars — bytes pre-existing (`engravePassphraseFlowFrom` already shipped
  it); P3 added a second, identical-text call site
  (`engravePassphraseFlowPreloaded`). Same trivial-length reasoning as above.
- **`ppConfirmWarning` / `ppConfirmWarningDerived`** (`gui/passphrase_flow.go`,
  100/103 chars) — **not** a `Warning.Layout` body. `ppConfirmFlow` draws
  these in a fixed panel with no scroller (`TestConfirmFitsPanel`'s own doc
  comment: "the codebase's only scroller (Warning.Layout) is bound to
  ButtonFilter(Up/Down), which no production path on SeedHammer II emits").
  Already gated by a **different**, pixel-height-based mechanism
  (`TestConfirmFitsPanel`), which P3 already extended to cover the `derived`
  variant this cycle introduced.
- **`"Engrave a passphrase plate?"`** (`gui/singlesig.go`) — a `ChoiceScreen`
  `Lead` line: short, fixed, never a scrolling body. Different screen type.
- **`"Source: this session's own derivation"`** (`srcDerived`,
  `gui/sysw_source.go`, spec §2.2) — renders through `syswSourceAccept` →
  `confirmReviewScreen`, a **paged** review screen (`Button2` "Page",
  start-index loop, `gui/multisig_build.go:1741`). Reachable to its end by a
  button the device has. Also 30 chars.
- **`verifyStatusMS1Clause`** (F-206, `gui/verify_status.go`, spec §3.3, 40
  chars) and **every line P4 changed** in `buildPassphraseInventoryLines` /
  `buildPlateInventoryLines` (`gui/multisig_build_census.go`, spec §6/§6.1) —
  **the restore-document passphrase lines the dispatch brief names
  explicitly.** Traced and confirmed: both feed `restoreDocFlow` /
  `multisigRestoreDocFlow`, which render through `restoreDocScreen` — its own
  doc comment: *"a plain, paged screen (NOT DescriptorScreen)"*, with a Page
  button mirroring "xpubVerifyFlow's gap-free paging so the long descriptor
  tail is always reachable." F-185's exposure is specific to
  `Warning.Layout`'s inaccessible scroller; a document with its own pager is
  a different, already-solved problem.

## The occlusion boundary (stated, not extended)

GATE 4 and GATE 5.3 answer different questions. `bodyDrawnFully`
(`gui/modal_fits_test.go:81-100`) compares the drawn frame's **op tree**
against the source string — `ExtractText` walks that tree with no notion of
what visually sits on top of what, so a glyph the panel draws **underneath**
P5b's opaque scroll-arrow chip is still "on the frame" as far as this check
can see: the text reached the compositor, whether or not a chip was painted
over it afterwards. That is **occlusion**, not truncation, and it is GATE
5.3's job (one pixel-level assertion that an arrow's chip does not overlap the
body's first/last drawn text rows, `gui/scroll_arrows_test.go`), not this
sweep's. This sweep does not attempt to extend to cover it and makes no claim
that it does.

## Build gate

- `go build ./gui/...` — clean. `gofmt -l gui/s6b_modal_fit_sweep_test.go` —
  clean.
- `go test ./gui/... -run 'TestS6bModalFitSweep$' -v` — all 4 subtests PASS
  before the sharded full run (TDD: assertions written and run before any
  fix was needed; all four came back green immediately, i.e. regression pins,
  not red→green fixes).
- **Sharded gate**: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 6 20m`
  (run with cwd = the worktree, since the script's `$PKG` arg is relative).
  Exhaustiveness line: `partition verified exhaustive: 854 == 854`. Wall:
  **148s**. Result: **exactly one failure**, in shard 2 —
  `TestGate51bMaxScrollAgreesWithVisibility`, reporting "22 diverge" over
  `bodysz.Y` in `[239,260]` against the 321-value probe range, logged as
  "EXPECTED (R-E)". This matches the brief's prediction exactly (22 of 321
  values in `[239,260]`). All 6 shards' stderr empty (`err0.txt`…`err5.txt`
  all 0 bytes). No other failure anywhere.
- **Non-`gui` packages**: `go test $(go list ./... | grep -v '/gui$') -count=1 -timeout 20m`
  — all 68 listed packages `ok` (or `[no test files]`), zero failures.
- **Goldens**: `git status --short` before this commit showed only the new
  test file as untracked (`?? gui/s6b_modal_fit_sweep_test.go`); no tracked
  file, including any `backup/testdata/*.bin`, was modified. No golden moved.
- `go vet ./...` was also run (not a stated P6 requirement, done for due
  diligence): the only findings are the two pre-existing go1.26
  `t.ArtifactDir()` failures the plan already names
  (`gui/op/draw_test.go:176` — `gui/freetext_sizeproof_golden_test.go:111`
  was not re-checked individually but is the plan's other named site) plus
  unrelated pre-existing `bspline_test.go` unkeyed-field vet warnings. Nothing
  new from this phase.

## What the spec/plan got wrong, or was ambiguous about

Nothing required a STOP. The one ambiguity worth recording (see "Scope
actually executed" above): F-192's filed text can be read as demanding a
whole-firmware sweep, while `IMPLEMENTATION_PLAN_s6b.md` explicitly scopes P6
to "every modal this cycle added or changed." I followed the plan (the
higher-precedence, more specific authority for *this phase's* scope) and
recorded the reading here rather than silently picking one.

## Result

GATE 4 is green: the sweep runs, states its own coverage (gated list +
ungated-with-reason for every candidate found, by the enumeration method
above), introduces no regressions, moves no goldens, and stays inside the
occlusion boundary GATE 5.3 owns. This closes P6, the last phase of the S6b
implementation plan. The whole-diff independent adversarial review (mandated,
non-deferrable, per `IMPLEMENTATION_PLAN_s6b.md` §4) is the next and final
step before the pre-flash gate.
