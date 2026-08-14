# Continuity — 2026-08-14c, the walk runs in 3 minutes and the BIP list is wrong

Supersedes `CONTINUITY_2026-08-14b.md`, whose "Where to start" is **done**: the
Trace A gather completes, the census names what was cut, and it is a committed
script rather than a session. Read this one.

That file's State table is also stale in one line — it says both repos are
UNPUSHED. They were pushed later the same day, and pushed again since. See
**Push state** below for what is actually outstanding.

## Where to start

**Write S0 deliverable 6's three tests.** The recon that must precede them is
done and persisted: `design/RECON_bip_vectors_S0.md`. Read that first — it is
the whole reason the list below differs from the plan's.

The plan's §1a warned that the *previous* test list "followed an author's memory
and two of its three tests were unwritable." **The replacement list scored the
same.** Verified against authoritative text, `bitcoin/bips` pinned at
`60f5b33b0a7be3cf09b933d97b78071d684db7d1`:

1. `TestBip383SortedMultiScriptMatchesPublishedVectors` — **NOT** the plan's
   `wsh(multi)` version. `bip380` has no unsorted `multi` (two enum values;
   `Parse` accepts only `"sortedmulti"`; `address.go` sorts unconditionally),
   and every `wsh(...)` vector in BIP-383 is `multi`. Use 383's **bare
   `sortedmulti`** script hex as the **witnessScript** anchor, then derive
   `0020||sha256(script)` → bech32 locally and say in the test that the wrap is
   derived. Best vector: the two-xpub one with three derived child scripts — it
   sorts *after* derivation, which is our real path.
2. `TestBip67SortedMultiKeyOrderScriptAndAddress` — BIP-67 publishes **four**
   fields per vector (List · Sorted · Script · Address, 5 P2SH addresses), so
   assert sorted → script → address end to end, not ordering alone. Vector 2 is
   an already-sorted no-op; vector 3 differs only in final byte and `02`/`03`
   prefix, which is what a naive comparator gets wrong.
3. `TestBip143NestedP2wshScriptPubKeyMatchesPublishedVector` — **replaces the
   BIP-141 test.** BIP-141 publishes NO vectors; every example is a template and
   `grep -cE '[0-9a-f]{40,}'` over it returns 0. BIP-143 §P2SH-P2WSH has a
   concrete 6-of-6 multisig with scriptPubKey + redeemScript + witnessScript,
   and its chain was machine-checked (both hashes MATCH — see the recon).

Then **vendor the vectors with provenance** in the shape of
`md/testdata/README.md` (source repo, commit, path, per-file meaning) — that is
the rest of D6.

**Still owed and NOT satisfiable by quotation:** no BIP publishes a
`wsh(sortedmulti(...))` vector, which is the device's actual output shape. Cover
it by composition and label it composed. An unattributed expected-address here
is self-agreement wearing the costume of a test — exactly what D7 exists to stop.

## Push state

| repo | branch | unpushed |
| --- | --- | --- |
| `seedhammer` (fork) | `main` | `88c028e`, `a46a9ce` |
| `mnemonic-engrave` | `master` | `9e094ef`, `bfcb5b9`, `b0ff2f8`, `825f7d7` |

Both trees clean. `master` needs the **`ci/staging` ritual** in `CLAUDE.md` — it
was followed successfully today (run `31845520163` passed, no "Bypassed rule
violations"), so it works; do not push `master` directly.

## What today established

**The emulator detour, which the operator judged worth the time.**

- **A 6-plate bundle walk now takes ~165s**, from over an hour. Two levers:
  `shPace` (writes between yields — frequency, not sleep duration, because the
  browser's timer granularity floors the latter), and replacing the driver's
  fixed sleeps with waits on observable conditions.
- **`shPace` defaults to 2048 in Go** (`pace.go`, `defaultPace`), so a walk that
  forgets to ask is still fast. Capped at 4096 — nothing above 2048 measured any
  faster (2048→186s vs 8192→183s). `shPace()` with no args READS.
- **The step stream is pace-independent** — the same bundle at pace 64 and 512
  produced the same six toolpath digests. That is what makes the knob safe, and
  it is why "assume engraving worked" was declined: throwing the stream away
  re-opens the F-121 class where geometry zeroed too early sends the head
  somewhere wrong and leaves no residue.
- `design/agent-reports/emulator-speedup-measurement.md` — **a sonnet agent was
  measuring the pace-1 baseline when this session ended.** If the file exists,
  it has the overall ratio; if not, the measurement never finished.

**F-162 found and FIXED** (`88c028e`). `mk1Gatherer.collected()` ranged a map
keyed by ChunkIndex, so a card's plates were cut in a random order and
"Plate 1 of 2" was wrong about half the time. Found by running the walk three
times and getting three orders. Not a funds defect — verified by reading, not
inference: `equalStrings` is MD1-only, `mk.Decode` reassembles by index (as does
the Rust primary), all `collected()` call sites are gated on `complete()`, and
`ParseHeader` rejects an out-of-range index so the walk cannot read a `""` gap.
**Firmware-visible: +160 bytes flash**, since `gui` compiles into the controller.

## Traps, so nobody pays for them twice

- **`shScreen()` inks no spaces.** `"Engraving plate"` never matches; use
  `"Engravingplate"`. Cost me a whole measurement round.
- **The screen LAGS during a cut, it does not freeze.** F-161 claimed a freeze
  and was **WITHDRAWN** — the entry keeps the retraction because how the wrong
  claim was made is the useful part. The refresh is a real 2.0 frames/s at pace
  1, degrading to 1.38 at 2048 and 0.87 at 4096. Key plate progress off
  `shToolpath` anyway: at walk paces a screen read can be a second stale.
- **Never sweep `clearInterval` over an id range to "clear every page timer".**
  `setTimeout` shares that id space and Go's wasm runtime schedules its
  goroutine wakeups through it, so the sweep freezes the Go scheduler — the
  instrument becomes the thing it claims to measure. That bogus experiment is
  what produced F-161.
- **Do not judge absence through `grep | head`.** F-161 also rested on a
  truncated pipe hiding the `WakeupAt` call that disproved it.
- **The browser caches `emu.wasm`;** a cache-buster on `index.html` does not
  bust it. Serve on a **fresh port** and check a symbol only the new build has.
- **Only one browser session exists.** Two agents driving it collide; do not
  start a measurement while another holds it.

## Open items

- **`a46a9ce` (default pace) has not been verified through the JS surface** — a
  measurement agent held the browser. The Go tests pass (7/7) and the build is
  green; confirm `shPace()` reads 2048 on a fresh page before relying on it.
- **F-158's premise is STALE.** It says gui's test platform returns a nil
  reader; in fact `testPlatform.NFCReader()` has a working hook that
  `sysw_source_test.go` (×4) and `run_reentry_test.go` already use. The
  host-side build-flow test is not blocked by NFC infrastructure. **Operator
  directive: leave NFC alone for now — focus on payload and keyboard input.**
  The cheap host-side twin is already specified as S1's third test,
  `TestBuildGathersEveryCosignerFromPayload` (n=3, two mk1 cards on the payload,
  **zero scans**), which needs no reader at all.
- **F-160** — the census cannot name an ms1 cut through the standalone codex32
  flows; a gate must treat `unattributed > 0` as "something was cut this census
  cannot name". Does not block S1–S5.
- **S0 D4, D5, D7, D8 untouched** — frame-receiver security properties, oracle
  pinning by source commit (the design-heavy one), `address_test.go` provenance,
  md vendored-vector re-pin. Note for D7: replacing its fixtures with BIP-383
  cannot be like-for-like, because 383 publishes **zero** addresses.

## Toolchain

Go is not on `PATH`: `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`,
then `nix develop --command go test ./...` from the fork root.
Emulator: `nix develop --command ./cmd/emu/build.sh`, serve `cmd/emu` on a fresh
port, open `index.html`, then in the console:

    const w = await import("./walk_trace_a.js");
    await w.run();          // ~165s, 6 plates, unattributed 0

Device build: `nix develop --command tinygo build -size short -o /dev/null
-target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks
./cmd/controller` — currently **1,342,468** flash.

`go vet ./...` has **6 pre-existing** `testing.ArtifactDir requires go1.26`
findings, all unrelated. That is the baseline; 6 is green.
