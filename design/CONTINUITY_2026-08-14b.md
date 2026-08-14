# Continuity — 2026-08-14b, the walk harness is built and reviewed

Supersedes `CONTINUITY_2026-08-14.md`, which was written earlier the same day and
whose "Where to start" is done. Read this one.

## Where to start

**Present the cosigner mk1 CHUNKS over `shNFC`, complete a Trace A gather, and
watch `shToolpath.strings()`.** Everything needed for it now exists; nothing else
in S0 blocks it.

The route is established and measured, so do not re-derive it:

1. `shSysw("cards")` **then run Load Payload from the MENU.** The boot offer has
   already consumed the first payload read, so switching at boot is too late.
   Confirmed: the cards blob's digest is `25271e58…`, the records blob's is
   `55adb800…` — check that digest to know which one you got.
2. Enter Engrave Bundle. With the cards payload loaded it offers a screen the
   records blob never reaches: **"First card from where? FROM PAYLOAD / ENTER IT"**.
3. FROM PAYLOAD seeds **one chunk, not a whole card**. Proof: the next confirm
   says *"Dropped an incomplete card — scan all its chunks to include it"*, and
   that message is only reachable when the gatherer holds a partial chunk-set.

**So the missing input is the individual chunk strings.** Every mk1 carrying an
xpub is ≥2 chunks. Get them from `cmd/emu/sysw_cards_payload.go` (its provenance
comment states the record inventory), regenerate with `go run
./cmd/buildpayloadcards`, or decode the blob with `me sysw show`. Then
`shNFC.present()` each chunk — the queue landed today, so several tags through
one reader now works.

A standalone **non-chunked** mk1 is refused by design (`clsSingleMK1Refuse`, host
parity). That is gui behaviour, not a harness bug; do not "fix" it.

## State

| | |
| --- | --- |
| Spec | `design/SPEC_multisig_build_repair.md` — R0 GREEN |
| Plan | `design/IMPLEMENTATION_PLAN_multisig_build_repair.md` — R0 GREEN |
| Stage | S0: deliverables 1–3 DONE and reviewed; 4–8 open |
| Review | **loop CLOSED at 0C/0I** — 4 reviews, 3 rounds, all folded |
| Code | fork `main` **9 commits ahead of `origin/main`, UNPUSHED** |
| Docs | `mnemonic-engrave` `master` **6 commits ahead, UNPUSHED** |
| Trees | both clean; all reviewer worktrees removed |

**Nothing is pushed.** Operator said "push later". `master` needs the `ci/staging`
ritual in `CLAUDE.md`; the fork's `main` goes to `bg002h/seedhammer`.

## What the fork's 9 commits established

1. `34f7762` **`gui.FrameAware`** — an optional interface the platform implements,
   offered from inside `run_flow.go`'s `draw`, so §10.2.4's warning reaches it
   too. `!tinygo`. **Costs the image ZERO bytes**, measured across three
   byte-identical builds with a positive control (a stub with a body moves it
   274,110 bytes). The structural guard is derived from the tree, not a names
   list.
2. `3c4eb86` **`shScreen()` / `shScreenSeq()`** plus `shWaitFor`/`shStep` in
   `index.html`. **First automated walk of this firmware**: four screens.
3. `6b3b453` **CI compiles the emulator.** It never did — `go test ./...` builds
   the `!js` stub and touches no `_js.go` file. Proven: with a deliberate typo,
   `go test ./cmd/emu/` said `ok` while the new step failed.
4. `dfbaea6` **`gui.EngravedAware`** — which string each accepted plate carried.
   +1,024 image / +560 code / +16 RAM, measured.
5. `740888d` **`shToolpath.strings()`** — the census.
6. `5374255` **`nfcSource` is a queue** behind a persistent reader. This is what
   made a multi-card gather possible at all; see F-158.
7. `0307aff`, `015150a`, `8e34f53` — the three review folds.

## The traps, so nobody pays for them twice

- **`shScreenSeq()` moving does NOT mean the screen changed.** `shPress` alone
  moves it by two (the button's pressed state); only the release navigates. So
  "wait for the count to move" **is** the stale read. Poll the TEXT via
  `shWaitFor`. The count answers one question: was anything drawn at all.
- **The count cannot tell you a tap missed.** A tap at (5,5) on empty screen still
  drew three frames.
- **The browser caches `emu.wasm`, and a cache-buster on `index.html` does not
  bust it.** A fix of mine "failed" twice against a stale build. Serve on a
  **fresh port**, and check a symbol only the new build has (e.g.
  `typeof shNFC.detach === 'function'`) before believing any negative result.
- **Coordinates** (480×320): nav buttons are on the RIGHT edge —
  Back **(453, 70)**, confirm **(453, 249)**. Start-screen carousel arrows
  **(25, 160)** and **(455, 160)**; it has eight programs.
- **Text has no spaces.** A space inks nothing, so `shScreen()` reports
  `"LoadPayload"`. `shWaitFor` strips whitespace from both sides for you.
- **Never put the engraved string on `gui.Plate`.** It breaks §10.2.2:
  `unlock_session.go` clears the record *before* the engrave screen precisely
  because the plate carries only geometry. A string field would hold
  seed-derived text — unwipeable — for the whole ~21-minute cut. Plates carry an
  opaque **id**; the string travels on a `!tinygo` hook.
- **`shNFC.present()` queues; it no longer replaces.** `detach()`/`attach()` reach
  the genuinely-no-reader machine, which gui treats differently from an idle one.

## Process, and this is the session's real finding

**Four times today, prose asserted more than the code did — and running the thing
found every one.** Two were doc comments the browser contradicted; two were
guards an independent review contradicted.

- I documented "read straight after `shTap` and you get the pre-tap screen".
  False. The real hazard was the opposite shape, and worse.
- I then documented the frame count as distinguishing "tap hit nothing" from
  "flow went elsewhere". Also false.
- The census documented itself as covering "md1/mk1/ms1"; two of three ms1 paths
  bypass `validateMdmk` (**Critical**).
- `tinygo_split_test.go` — a guard whose whole purpose is keeping hook interfaces
  out of the firmware — **never parsed the `_tinygo.go` stub**, so an interface
  declared in the firmware-side file passed cleanly (**Critical**, proven with a
  real tinygo build linking it into a 1.34 MB image).

The guard case is the one to internalise: **I had mutation-tested it 7 ways and
killed all 7.** Every mutation touched the *host* file, because that is where I
was thinking — the mutations inherited the implementation's blind spot, so 7/7
measured only the half already covered. For a guard, derive the mutation set from
the **property**, then ask *which inputs does this never look at?* That set is
where the surviving mutant lives.

And one fold of mine was defective in this project's usual way: facts corrected,
duplicates left. `scripts/fold-propagation-check.sh` caught it — run it after
every fold, with patterns taken from the review's **quoted strings**.

## Reviews, persisted verbatim

- `design/agent-reports/walk-harness-residency-review.md` — 0C/0I. Answers
  mechanically: a probe module referencing the hooks builds on the host and fails
  `undefined:` under `-tags tinygo`.
- `design/agent-reports/walk-harness-false-pass-review.md` — 1 Critical.
- `design/agent-reports/walk-harness-guard-structural-review.md` — 1 Critical,
  1 Important.
- `design/agent-reports/walk-harness-fold-rereview.md` — 0C/1I; the Important was
  in my own fold. Also confirmed the *replacement* claim is true: `bundleEngrave`'s
  ms1 really is covered, because `validateMdmk` is format-agnostic.

## Open follow-ups

- **F-158** — items 2 and 3 still OPEN. gui's test platform still returns a nil
  reader (`gui/gui_test.go:445`), so `gui/multisig_build_flow_test.go:199` still
  stops at the gather and the build flow has **no host-side end-to-end test**.
  That is the half that would catch a regression without a browser.
- **F-160** — the census cannot see an ms1 cut through the standalone codex32
  flows (`engraveCodex32`, `unlockEngraveCodex32`); they carry id 0 and land in
  `unattributed`. Does **not** block S1–S5, whose ms1 goes through
  `bundleEngrave`. The fix needs a source-tagged variant of
  `backupSeedStringFlow`, which also serves BIP-39 backups that must NOT be
  announced. **A gate must treat `unattributed > 0` as "something was cut that
  this census cannot name."**

## Still owed, in order

**Finish the walk** (above) — then the rest of S0: frame-receiver security ·
oracle pinning by **source commit** (a `--version` string is self-reported and
spoofable) · published-BIP vectors — **BIP-383** for `wsh(multi(…))` compared at
*scriptPubKey*, BIP-67 ordering, BIP-141 for P2SH-P2WSH; **not** BIP-382, which
publishes no addresses · `address_test.go` provenance · md vendored-vector re-pin
(0.36→current; measured **zero** byte drift, so coverage catch-up, not correctness
repair).

Then S1–S5 per the plan. **S6 is hardware and needs the operator**: flashing via
`~/bin/sh/sh2-flash`, cutting real plates, restoring at an external coordinator,
including one divergent-origin multi-slot multi-master build and an ms1 plate read
back.

## Toolchain

Go is not on `PATH`. `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`, then
`nix develop --command go test ./...` from the fork root. Emulator:
`nix develop --command ./cmd/emu/build.sh`, then serve `cmd/emu` and open
`index.html`. Device build:
`nix develop --command tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`.

`go vet ./...` has **6 pre-existing** `testing.ArtifactDir requires go1.26`
findings, all unrelated. That is the baseline; 6 is green.
