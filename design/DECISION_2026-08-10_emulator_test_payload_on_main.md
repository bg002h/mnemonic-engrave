# Decision — the emulator's test payload ships on `main`

**Date:** 2026-08-10. **Operator ruling:** *"We need to include emulator work in
our branch… it will all end up on master / main eventually."*
**Landed:** fork `b2b` @ `75233b8` (merges `5b93924`, `aa315f2`, guard `75233b8`).

## What is being shipped

`cmd/emu` carries a **real §6.1 sealed payload** and **the passphrase that opens
it**, printed to the browser console and shown in `index.html`:

- blob: `cmd/emu/sealed_test_payload.bin`, 1536 bytes, the exact UF2 `me seal`
  produced for the 2026-08-09 Phase B2b rehearsal — Vector F, 15 secret records
  (3 codex32, 6 mk1, 6 md1), all **published test vectors**, 0 public records,
  300,000 iterations. Verified to open with `seal.Opener.Open` before embedding.
- passphrase: `sealedTestPassphrase`, a twelve-word constant.

**No funds are at risk** — every record is a published test vector. What is at
risk is the claim the machine makes about itself: a shipped SeedHammer II must
never boot carrying a pre-known passphrase.

## Why it is acceptable

The emulator is the only place the sealed-payload flow can be exercised without
hardware, and before this it could not be exercised at all: `PayloadReader()`
returned `nil` under `GOOS=js` ("a browser build has no XIP flash"), so the
Sealed Payload entry did not exist — 8 carousel dots, not 9. Every §10.2.4
reading, the whole unlock/KDF/decrypt path, and the abort→resume comparison were
hardware-only as a result.

## What makes it safe, and how that is enforced

Two properties, and **neither fails to compile when broken** — a `//go:build js`
deleted from `sealed_test_payload.go` builds fine everywhere, and an import of
`cmd/emu` from a firmware package builds fine too. So they are tests:

| test | invariant |
| --- | --- |
| `TestTestPayloadIsConfinedToJSOnlyFiles` | any file naming the blob, its symbols, or the words is in `cmd/emu` **and** carries `//go:build js` |
| `TestNothingImportsTheEmulator` | `cmd/emu` is `package main` (Go forbids importing it) and no file in the module names its import path |

Mutants, each verified applied before the run: drop the build tag → KILLED;
paste the passphrase into `gui/wipe_guard.go` → KILLED; name the import path
there → KILLED.

## The correction this produced

**I reported the confinement as proven when it was not.** The claim was:

> `go list -deps ./cmd/controller` contains no `cmd/emu`.

That command **fails** — `cmd/controller` is `//go:build tinygo && rp`, so it
exits with *"build constraints exclude all Go files"* and prints **nothing**.
Piped into `grep -c 'cmd/emu'` it yields `0`, which reads exactly like the
dependency being absent. The measurement was vacuous and I passed it on as
evidence.

The second attempt was better and still wrong: `-tags tinygo,rp` with
`GOOS=linux GOARCH=arm` prints 255 packages **and still exits non-zero**, because
TinyGo's `machine` and `device/rp` are not in the Go stdlib. A **partial** graph
cannot prove absence — `cmd/emu` could sit behind any unresolved edge.

The invariant is now checked where it is *total*: decidable from source, with no
toolchain, target, or tags. Nothing in it can return empty and be mistaken for a
pass, and both tests **fail rather than skip** when they cannot see the tree they
are meant to scan.

**The general lesson, which is the reason this file exists:** *empty output is
not evidence of absence.* A grep for a dangerous thing must be paired with a
positive control proving the search looked where it claimed to. The same shape
produced a false GREEN earlier this cycle (a `_arm_test.go` suffix that put a
whole test file in `IgnoredGoFiles`) and again in a mutation table (a `sed` that
never applied, reported as a surviving mutant).

## Not done, and deliberately

- **A TinyGo binary scan is not a unit test.** Building `cmd/controller` with
  TinyGo and searching the image for `MNEMBLOB`, the passphrase and the blob was
  run once by hand and passed. It costs minutes and needs a toolchain CI does not
  carry, so it belongs in a release check rather than `go test`.
- **This must not go upstream.** Fork `main` is `bg002h/seedhammer`. Upstream
  PRs branch off `upstream/main` and stay small and focused; a demo payload with
  a published passphrase is fork-native and has no business in
  `seedhammer/seedhammer`.
