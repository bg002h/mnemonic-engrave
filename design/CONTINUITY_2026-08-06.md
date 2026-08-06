# Continuity — 2026-08-06

Supersedes `CONTINUITY_2026-08-05b.md`. Written at a context reset.

## 1. Start here

**A hard-steel plate carrying the halved-acceleration test has been engraved and
the operator has read it. Their observations are the input this session is
waiting on — ask for them first and do not re-derive the experiment.**

The question that plate answers, and the only reason it was cut:

> The diagonal wiggle on hard steel — is it **following error at direction
> changes**? Halving acceleration and jerk is the direct test of that.

What to read, in order:

- **`~` first.** Two 108° interior corners and the shortest segments in the set;
  by the operator's account the worst glyph. Prior description, to compare
  against: the middle segment bends upward near its left end (shallowing the
  slope), dips slightly near its right end, and the final upward segment then
  appears truncated because it starts too high.
- **`^` second.** Its defect is *truncation on the right* — an exit-side
  symptom, not an interior corner. If both improve, they share one cause.
- **A null result is a real result.** If `~` is unchanged with feed *and*
  acceleration both halved, motion is not the mechanism, and the next suspect is
  the hammer itself: strike rate, strike force, or tool deflection under load.
  Different investigation, different levers. Say so plainly rather than
  proposing a fourth motion parameter.

### What was flashed

```
test-e4-a125-j1300.signed.uf2   (untracked in the seedhammer repo root)
sha256 daf4100e490ede04fa598451e494309dfca1ce84bd418e42d84465963d62639f
engraving 4 mm/s · acceleration 125 · jerk 1300   (from 8 / 250 / 2600)
```

Both source files were reverted after the build; both trees are clean of it.
Estimated plate time ~21m22s against 19m15s for the half-speed-only plate (+11%).

**The machine is still running this test build.** Before any real work, reflash
the release: `~/bin/sh/sh2-flash -p`, pick `fork-v0.0.0-g1945251`.

**Name test artifacts by their config.** Both test builds otherwise produce
`seedhammerii-v0.0.0-gd7155b9-dirty.signed.uf2`, and the second silently
overwrote the first.

## 2. The engraving-parameter matrix

| | soft steel | hard steel |
| --- | --- | --- |
| 8 mm/s (stock) | dots least visible; retrace widening **bad** | wiggle visible; retrace widening **bad** |
| 4 mm/s | **NOT YET CUT** | dots closer, much nicer; retrace widening **much improved**; wiggle unchanged |
| 4 mm/s + half accel/jerk | — | **just cut — awaiting the read** |

Established, and not to be re-litigated:

- **Retrace widening tracks SPEED, not material.** Equally bad at soft@8 and
  hard@8; good at hard@4. That is what makes the missing soft@4 cell decisive:
  it settles whether this becomes a soft/hard *material* setting or simply a
  slower feed for every plate.
- **The machine is a hammer** making discrete dots, not a continuous cut. Slower
  feed spaces the dots more closely, which is why hard@4 approaches the smooth
  look of soft@8.

### Ruled out as the wiggle's cause, with evidence

Do not re-test these.

- **Tripled control points** — `|` has them and is clean.
- **Commanded geometry** — measured 0.0004 mm from straight.
- **Speed alone** — tested directly; hard@4 still wiggles.
- **Start-of-run effects** — `^` fails at its *end*.
- **Material alone** — the same plate art differs between the two steels, but
  speed changes the retrace defect on *both*.

## 3. F-58 — the input wedge (parked, deliberately)

Full entry in `FOLLOWUPS.md`. The operator has parked it; **do not reopen it
without being asked.** Two things are worth carrying forward:

1. **It is safe to retry.** The wedge happens on the Footer text-entry screen,
   several screens before any motion. Nothing is commanded, no plate is at risk,
   and the workflow ran normally on retry. Cost is re-typing.
2. **Three diagnoses were recorded and all three were wrong**, each corrected by
   the operator: driver state surviving a flash (impossible — one USB-C port, so
   the PSU swap *is* a power cycle), a hang starting the engrave, and a hang on
   the post-engrave Accept. The lesson is recorded in `FOLLOWUPS.md` because the
   pattern mattered more than any of the guesses: **do not name a mechanism
   before the evidence names it.** Ask which screen, ask what else responded.

The one solid finding, if it is ever picked up: **every button died, not just the
checkmark.** That is the shared input queue, not a widget. `EventRouter.Next`
(`gui/event.go:266`) is strict head-of-queue, and filters are registered only as
a side effect of calling it, so any consumer returning early registers nothing
and `Reset` then judges the head against an incomplete filter set. The
reproduction test is the deliverable, not a fix.

## 4. Open work

Ordered by what unblocks what.

- **Read the accel/jerk plate.** §1. Blocking everything else on the wiggle.
- **Cut soft @ 4 mm/s.** Fills the last matrix cell. Decides material-setting vs
  plain speed choice.
- **Reflash the release** — `~/bin/sh/sh2-flash -p` → `fork-v0.0.0-g1945251`.
  The machine is on a test build right now.
- **Three glyphs remain of the sixteen: `O`, `o`, `8`.** The other thirteen
  landed. Scope directive still in force: **only the sixteen** — the operator
  explicitly excluded `w`, `m` and the rest. (`W` was added by later request and
  is done.)
- **`{` then `}` clears by only 10 font units** (0.033 mm). Open; pinned by
  `font/constant/ink_clearance_test.go` so it cannot silently get worse.
- **Nothing pins the machine's actual engraving speed.** Four independent copies
  of `engravingSpeed = 8 * mm` exist and goldens use test-local ones, so a
  firmware speed change moves no test. Real gap.
- **Push.** `mnemonic-engrave` has 12 unpushed commits; `seedhammer` has 1
  (`d7155b9`) on `main`.

## 5. Standing constraints

- **Always `~/bin/sh/sh2-flash`, never picotool by hand.** The build output is
  unsigned and cannot boot; a laptop USB port cannot boot the machine either
  (needs 20–28 V USB-PD). **Judge a boot only on machine power.** If it does not
  boot, do **NOT** burn another OTP slot.
- `gh`'s default repo is UPSTREAM `seedhammer/seedhammer`. Every fork operation
  needs `--repo bg002h/seedhammer` explicitly.
- **Never a bare `go test ./... -update`.** backup's sixteen goldens are frozen.
  Re-record scoped with `-run`, then confirm `git status` lists only the
  expected files. The SIZEPROOF goldens are the exception that *is* meant to
  move on a glyph edit — that is documented in
  `gui/freetext_sizeproof_golden_test.go`.
- **max k = 2 is a security property** (timing disclosure). The k=3/k=4 buckets
  are pinned empty by `TestPassphraseRunPartition`.
- Insert `FOLLOWUPS.md` entries **before** the `## Resolved` heading.
- Stage paths explicitly; no `git add -A`.
- All Go work runs under `nix develop --command`; nix lives at
  `/nix/var/nix/profiles/default/bin`.

## 6. Repo state at the reset

| | |
| --- | --- |
| `mnemonic-engrave` | clean, 12 unpushed, HEAD `3d90722` |
| `seedhammer` | `main`, HEAD `d7155b9`, 1 unpushed; `test-e4-a125-j1300.signed.uf2` untracked |
| `seedhammer-wt-glyphcleanup` | `constant-glyph-cleanup` @ `b9b7831`, merged to main, clean |

45 packages green at the last full run.
