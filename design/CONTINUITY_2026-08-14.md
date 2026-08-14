# Continuity — 2026-08-14, on-device wallet-policy authoring

Written at the end of a session that ran spec → plan → fifteen review rounds →
first code. Read this instead of the fifteen reports.

## Where to start

**S0 deliverable 3 is DONE.** The walk harness exists: `shTap`/`shPress`/
`shRelease`/`shSysw` drive, `shScreen`/`shScreenSeq` read, `shToolpath.strings()`
reports what was engraved, and `shWaitFor`/`shStep` in `index.html` make the
correct polling idiom the default one.

**Next: an end-to-end browser walk that ENGRAVES.** Everything is wired at both
ends and nothing has yet joined them in a browser — no walk has cut a
constellation string and seen it come back out of `shToolpath.strings()`. The
one concrete blocker found: Engrave Bundle's gather sits on "Scan a card, or
Done" and `shNFC.present("md1yqpqqxqq8xtwhw4xwn4qh")` left the card count at 0,
so **find the scan affordance** (or the right tag encoding) first. That is S1's
gate, and S1 is where it belongs.

Then S0's remaining deliverables, listed below.

## State

| | |
| --- | --- |
| Spec | `design/SPEC_multisig_build_repair.md` — R0 GREEN |
| Plan | `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` — R0 GREEN, then four lens reviews + a fact audit reopened it; all folded |
| Code | fork `main`, eight commits: `3009f22`, `3ea08f9`, `10286e4`, `34f7762`, `3c4eb86`, `6b3b453`, `dfbaea6`, `740888d` |
| Stage | S0, deliverables 1–3 done, 4–8 open |
| Repos | fork `main` committed and **NOT pushed** — five new commits are local only |

**Do not re-review the plan.** Fifteen rounds have run and the last four found
what reading could not. The prescription from
`design/agent-reports/multisig-build-repair-process-diagnosis.md` is: stop
reviewing text, build S0's harness, walk Trace A until it breaks.

## What the three commits established

1. **`3009f22`** — confinement is now STRUCTURAL. It discovers every
   `//go:embed` under `cmd/emu` rather than matching a names list, checks the
   **AST** not raw text (so a doc comment mentioning a blob is not a leak), and
   is mutation-checked three ways. The two name-keyed guards it subsumes are
   deleted; `TestNothingImportsTheEmulator` is untouched and pins a different
   property.
2. **`3ea08f9`** — a SECOND payload, `sysw_cards_payload.bin`, 978 bytes, four
   cosigner mk1 cards + master A. The first blob has no `ClassMDMK` at all,
   which is why no walk could reach Build policy's gather. Its record inventory
   is in the source **and** pinned by a test, asserted per-ORIGIN: a count tells
   you a card vanished, an origin tells you which stage lost its walk.
   Regenerate with `go run ./cmd/buildpayloadcards`.
3. **`10286e4`** — `shTap`, `shPress`/`shRelease`, `shSysw("records"|"cards"|
   "none")`. Verified driving the real GUI in a browser (ink 7442 → 9009 →
   14091 across two taps).
4. **`34f7762`** — `gui.FrameAware`, the first `gui` change of the cycle. An
   optional interface the platform implements, offered from inside
   `run_flow.go`'s `draw` so §10.2.4's warning reaches it too. `!tinygo`.
   **Costs the image ZERO bytes, measured** — three byte-identical builds, with
   a positive control (a stub with a body moves it 274,110 bytes) so the zero is
   not a build that ignored the edit. The structural guard is now derived from
   the tree (`tinygo_split_test.go`) instead of naming one file and one
   identifier; mutation-checked 7/7.
5. **`3c4eb86`** — `window.shScreen()` / `shScreenSeq()`, plus `shWaitFor` and
   `shStep` in `index.html`. **The first automated walk of this firmware**: four
   screens, boot offer → digest → warnings → keep, each step naming the screen
   it must reach.
6. **`6b3b453`** — CI now compiles the emulator. It never did: `go test ./...`
   builds `main_notjs.go` and touches no `_js.go` file, so the whole harness was
   checked by nobody. Proven with a deliberate typo — `go test ./cmd/emu/`
   reported `ok` while the new step failed.
7. **`dfbaea6`** — `gui.EngravedAware`: which md1/mk1/ms1 each accepted plate
   carried. **+1,024 bytes of image, +560 code, +16 RAM** — measured, and not
   free unlike the frame hook.
8. **`740888d`** — `shToolpath.strings()`, the census.

## The traps, so nobody pays for them twice

- **A walk without `shScreen()` is a hope, not a walk.** Proven the hard way: the
  primitives work, and a full walk still failed because one mis-timed step made
  every later step a guess. Four consecutive "steps" sat on the same screen
  unnoticed. **Now fixed** — and the fix corrected two things I had written
  about it that turned out false when run:
  - **`shScreenSeq()` moving does NOT mean the screen changed.** `shPress` alone
    moves it by two, for the button's own pressed state; only the release
    navigates. So "wait for the count to move" IS the stale read. Poll the
    **text** (`shWaitFor`), never the count.
  - **The count cannot tell you a tap missed.** A tap at (5,5), a corner holding
    nothing, still drew three frames — the GUI redraws on any pointer event.
    The count answers one question: was anything drawn at all.
- **Nav buttons are on the RIGHT edge**, not the bottom: `dims.X - 53` with
  `leadingSize = 44`, so Back ≈ (453, 70) and confirm ≈ (453, 249). The start
  screen is a carousel of eight programs; its arrows are ≈ (25, 160) and
  (455, 160).
- **Never put the engraved string on `gui.Plate`.** It is the obvious way to let
  a walk report what was cut, and it breaks §10.2.2: `unlock_session.go` clears
  the decrypted record *before* building the engrave screen precisely because
  the plate carries only geometry. A string field would hold seed-derived text —
  unwipeable, since Go strings cannot be zeroed — for the whole ~21-minute cut,
  and the comment defending that clear would silently have become false. Plates
  carry an opaque **id**; the string travels on a `!tinygo` hook.
- **`mk.Decode` returns `m/48h/0h/0h/2h`**, not apostrophe notation. An
  expectation asserting apostrophes failed first.
- **Never edit `sysw_test_payload.bin`.** Its digest is pinned in
  `sysw_test_payload.go` *and photographed* in the published Load Payload
  journey PDF. That is why there are two blobs.
- **`sh(wsh)` is a template-PICKER choice, not a payload property.** S3 needs no
  extra record. The plan briefly implied otherwise.
- The `go vet` finding in `gui/freetext_sizeproof_golden_test.go`
  (`testing.ArtifactDir requires go1.26`) is **pre-existing and unrelated**.

## Process rules this cycle changed

Both are now in `CLAUDE.md`; this is the short form.

- **Closure is LENS-closure, not finding-closure.** Both gates closed GREEN under
  a correctness lens and the six rounds after found seven more Criticals — every
  one from a *first-time question*, none from looking harder. Enumerate the
  lenses up front.
- **A plan may not close while one of its own gates has never been run.** The
  acceptance mechanism here was a walk against a payload with no cosigner cards:
  invisible to thirteen readings, an hour to find by trying it.
- **Folds fail by incomplete propagation**, four times this cycle. Run
  `scripts/fold-propagation-check.sh` after every fold, and **derive the patterns
  from the report's own quoted strings**, not from a paraphrase — a gate fed
  paraphrased patterns passed while the defect survived.
- **Gate coverage, not gate quality, is the weak point.** 16/16 *gated* code
  citations were true; **5/22 ungated facts were false**.

## Still owed, in order

S0: ~~`shScreen()` + `shToolpath.strings()`~~ **done** · frame-receiver
security · oracle pinning by **source commit** (a `--version` string is
self-reported and spoofable) · published-BIP vectors — **BIP-383** for
`wsh(multi(…))` compared at *scriptPubKey*, BIP-67 ordering, BIP-141 for
P2SH-P2WSH; **not** BIP-382, which publishes no addresses · `address_test.go`
provenance · md vendored-vector re-pin (0.36→current; measured **zero** byte
drift, so coverage catch-up not correctness repair).

Then S1–S5 per the plan. **S6 is hardware and needs the operator**: flashing via
`~/bin/sh/sh2-flash`, cutting real plates, restoring at an external coordinator,
including one divergent-origin multi-slot multi-master build and an ms1 plate
read back.
