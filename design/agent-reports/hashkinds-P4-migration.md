# hashkinds phase 4 — finishing the `*md.HashLock` migration in `seedhammer`

Repo `/scratch/code/shibboleth/seedhammer`, branch `hashkinds-p4`, tree left
**dirty and uncommitted** as instructed. Toolchain `go1.26.7`
(`/scratch/code/shibboleth/.toolchain/go/bin`).

## 1. Final gate results

| gate | result |
| --- | --- |
| `go build -gcflags=-e ./...` | **0 errors** (was 33, all in `gui/`) |
| `go vet ./...` | **10 lines, all pre-existing** — see below |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **0 lines** |
| `gofmt -l .` | **5 files — exactly the pristine baseline** |
| `go test ./...` | **54 ok / 20 no-test-files / 2 FAIL** — `backup`, `hashlock` |

`gofmt -l .` returns exactly `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`,
`mt/mt.go`, `mt/mt_test.go` — the five-file baseline, nothing else.

`go vet ./...` returns only ten `testing.ArtifactDir requires go1.26 or later
(file is go1.25)` diagnostics. **Verified pre-existing, not inherited from this
work:** I ran `go vet ./...` in a throwaway detached worktree at `HEAD`
(`0562e81`) and got a byte-identical ten-line set. The cause is `go.mod`'s
`go 1.25.10` directive against `t.ArtifactDir()` call sites; none of those files
is touched here. The worktree has been removed.

`gui` is green: `ok seedhammer.com/gui 151.927s` in the full run, and
independently **1324 tests across 24 shards, all ok**, via
`mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24 20m` (wall 26 s) — whose
exhaustiveness assertion passed, so no gui test was dropped.

## 2. The two failing tests, and why I did not make them pass

- `seedhammer.com/hashlock` — `TestH6MethodLineAndQRTextMatchTheMSCorpus`
- `seedhammer.com/backup`   — `TestHashlockQRTextMatchesTheMSCorpus`

**Both fail for one reason, and it is not the type migration.** The re-vendored
corpus (already-done work) is ms-codec **0.10.0**, whose `qr_text` format gained
a `hash: <kind>` line between the method line and the phrase line. Every one of
the seven pre-existing rows changed, and three per-kind rows were added (7 → 10):

```
got  "hashlock v1\nmethod: sha256\nphrase: correct horse battery staple"
want "hashlock v1\nmethod: sha256\nhash: sha256\nphrase: correct horse battery staple"
```

`hashlock.QRText(hardened bool, phrase string)` (hashlock/hashlock.go:249) still
emits the three-line v0.9 form. It was **not** in the already-done list and
takes no kind.

**Both tests were green at `HEAD`** — I ran `go test ./hashlock/ ./backup/` in
the clean worktree and got `ok / ok`. They went red when the corpus was
re-vendored, before I touched anything.

I did not port the format, for three reasons:

1. It is a **normative change to an engraved artifact**, not a type change. The
   brief's rule 2 is "preserve today's behaviour EXACTLY"; adding a line to the
   QR changes what every phrase plate cuts.
2. It **moves an existing golden**. `backup/TestHashlockGoldens` pins
   `hashlock-phrase-qr-v9` and says in its own doc comment "New goldens are
   permitted; no existing one may move." Different QR payload → different
   modules → the `.bin` moves, which is a deliberate re-baseline with its own
   review, not a migration chore.
3. It belongs with the kind-selection commit: the line names a kind, so `QRText`
   needs a kind parameter threaded from `p.lock.Kind()` through
   `gui/composer_preimage_plate.go:303` and `backup.Hashlock.QRText`.

I did **not** weaken, skip or delete either test. They stand, red, naming the
real gap — which is what the lockstep pin exists to do.

**One measurement for whoever takes that commit, so it is not guessed at.** I
probed the QR version with a throwaway test in `backup/` (written, run, deleted
— `git status` confirms no such file remains), over the worst-case §8.6 phrase:

```
kind=""          qrtext=194 bytes -> QR dim 53
kind="sha256"    qrtext=207 bytes -> QR dim 53
kind="hash256"   qrtext=208 bytes -> QR dim 53
kind="ripemd160" qrtext=210 bytes -> QR dim 53
kind="hash160"   qrtext=208 bytes -> QR dim 53
```

So the **QR version does not move** — dim 53 (v9) for all four kinds, and
`TestHashlockQRCodeIsVersion9`'s `code.Size != 53` assertion survives. The
§6.5 416000-unit plate budget is not at risk from the extra line. What moves is
the golden's module pattern, and only that.

## 3. Judgement calls (everything that was not a mechanical substitution)

1. **`hashlockLockOf(k md.HashKind, x *[32]byte) *md.HashLock`** — one new
   helper in `gui/composer_hashlock.go`, wrapping `hashlock.DigestOf` in
   `md.NewHashLock`. It **panics** on `ok == false` rather than returning it.
   `ok` is unreachable by construction (`DigestOf` and `DigestLen` switch on the
   same kind), and the alternative was threading `(*md.HashLock, bool)` through
   five GUI routes, each needing a refusal screen for a branch that cannot be
   taken. It follows two precedents already in the tree: `composerPresetDigest`
   (already-done work) panics identically, and `composerHashEdit`'s default arm
   panics on an impossible pick row. Every call site passes `md.KindSha256`.

2. **`composerHexEntry` now returns `(*md.HashLock, bool)`**, not
   `([32]byte, bool)`. The width check moved into `md.NewHashLock` and its false
   arm joins the existing `hex.DecodeString` refusal — same screen, same
   message, no pad, no truncation, no zero digest. Documented at the function as
   sha256 *because the pad's 64-character bound says so*; a kind pick would come
   with its own bound, since the rule is `DigestLen()*2`.

3. **`ComposerPathHashes()` changed from `[]*[32]byte` to `[]*md.HashLock`** —
   an exported API with one consumer, `cmd/emu/composer_js.go` (GOOS=js). The
   alternative was keeping `[]*[32]byte` and copying `Digest()` into a fixed
   array, which silently pads a 20-byte kind. `composer_js.go` now hexes
   `h.Digest()`, so the documented JS contract (`"<64 hex>" | null`) is
   byte-identical today. `GOOS=js GOARCH=wasm go vet ./cmd/emu/` is clean — CI
   runs exactly that step (`.github/workflows/test.yml:123`). The hook still
   copies (`d := *p.Hash; out[i] = &d`), and I kept the copy and said why:
   the reason is aliasing, not the type.

4. **`composerPreimagePlates`' deterministic sort key** was the raw 32-byte
   digest; it is now `MapKey()` = `"<token>:" + digest`. Within one kind every
   key shares the `"sha256:"` prefix, so the order is **identical** to what
   shipped; two kinds sort into blocks rather than interleaving.
   `TestComposerPreimagePlatesAreOrderedByPathNotByMap` is green.

5. **`composerSelfCheck`** — `b.Sha256Digests[0] != *p.Hash` became a
   `md.NewHashLock(md.KindSha256, got[:])` plus `p.Hash.Equal(want)`, so a path
   carrying another kind is a *mismatch* rather than a byte compare.
   Fail-closed twice over: the count arm above it already refuses such a path,
   because `md/policy_shape.go:284` only puts `tagSha256` digests in
   `Sha256Digests`.

6. **`composerEditCanRenumber`'s probe** (`gui/composer_discard.go`) builds a
   sha256 lock from the 32 `0x01` bytes. Its value is arbitrary — only its
   presence moves the signature — but it goes through `md.NewHashLock` and
   panics on the impossible refusal, because probing with `nil` would make the
   two signatures equal and the answer wrong.

7. **`hashlockPlatesRecord.digest` is `*md.HashLock`, nil until derived.** The
   `derived bool` and a zero `[32]byte` used to answer the same question twice;
   nil now says it directly. I also added `recs[i].digest = nil` to
   `hashlockPlatesScrub` for consistency — behaviourally inert (the slice is
   dead after the flow's own defer), and no test asserts on it.

8. **`TestComposerStateHookReportsEachPathAndHandsOutCopies`** — see §4; this is
   the one test whose *mechanism* I had to change.

9. **`hashlock/hashlock_test.go`'s `corpusSHA256`** updated
   `4f1819cd…` → `0a911f78f3cdc867dcc44483b7f4c0c1ac87b6d9b30b79f52094e8979bc3d8ce`,
   the pin the brief names and `testdata/hashlock-v0.8.provenance.json` already
   carries, and the `ms-codec 0.8.0` in its failure message → `0.10.0`. Without
   this every corpus test in the package was `t.Fatal`-ing before it ran. With
   it, **all of them pass on the new corpus except the qr_text one** — so the
   re-vendored derivation, refusal and kind rows agree with the current Go code.

10. **`sysw/composer_records_test.go`** — the all-`0xa8` byte loop now runs over
    `h.Digest()`, and I added two assertions rather than dropping any: the
    bare-form record parses as `md.KindSha256`, and its digest is 32 bytes. The
    kind is half of what the record means.

11. **`md/compose_test.go`'s `composeH`** is now a `*HashLock` at `KindSha256` —
    the corpus rows were generated against `sha256(H)` fragments, which is what
    keeps every expected chunk string byte-identical. `md` is green.

12. **`TestComposerAnyPathByPhraseIsPerDigest`'s `retyped` row.** It was
    `retyped := phrase` — a copy of the array, hence a distinct pointer with the
    same bytes, which is the whole point of the row (r0 fidelity M-1). It is now
    `hashlockMustLock(t, hashlockAnchorSHA_H)`, a fresh allocation with the same
    bytes. The "distinct pointer, same value" property is preserved, and I said
    so in the comment. Likewise `TestReassigningTheSameDigestStaysByPhrase`'s
    `got == &phrase` became `got == phrase` — still a pointer-identity check,
    still meaningful, because `composerHashEdit` writes the pointer
    `sysw.ParseHashRecord` freshly allocated.

13. **Two test helpers added** in `gui/composer_hashlock_test.go`:
    `composerTestLock([32]byte) *md.HashLock` and
    `hashlockMustLock(t, hex) *md.HashLock`, plus
    `composerTestFlipDigest(*md.HashLock) *md.HashLock` for the self-check
    mutation row (a digest is unexported with no setter, so `d[0] ^= 0xff`
    cannot be written in place any more; it is rebuilt through `md.NewHashLock`
    at the same kind, which is the same perturbation).

14. **One stale MUTATION comment corrected**, in
    `gui/composer_hashlock_plates_test.go`: "build the locator before
    `hashlockPlatesDerive` → the row reads `hash 00000000..00000000`" is no
    longer true — the record's digest is nil at that point and
    `hashlockFirst8Last8` panics. The mutation is still caught, **louder** than
    before. I updated the text rather than leaving a comment that describes an
    outcome that cannot happen.

## 4. The one test whose meaning I could not preserve as written

`gui/composer_state_hook_test.go` —
`TestComposerStateHookReportsEachPathAndHandsOutCopies`.

Its second half did `got[1][0] ^= 0xff` and asserted the policy had not moved —
a *runtime* proof that the hook hands out copies. **That statement no longer
compiles**: the digest lives in `md.HashLock`'s unexported field with no setter,
so no caller outside package `md` can write one at all. The property is now
enforced by the type system instead of by the test.

What the test's own named mutation (`out[i] = p.Hash` in `setComposerStateHook`)
actually changes is the pointer *identity*, and that is still observable. I
replaced the write-through with:

```go
if got[1] == st.list.Paths[1].Hash { t.Error("... handed out the POLICY's own pointer ...") }
if !st.list.Paths[1].Hash.Equal(d)  { t.Errorf("the policy's digest moved: ...") }
```

This is a **stronger** check of the named mutation than the write-through was
(it fails directly rather than via a side effect), and I wrote the reasoning
into the test so a later reader does not restore the uncompilable form. Nothing
was skipped or deleted.

## 5. Defects found — things that are NOT migration chores

**D-1 (highest). No test anywhere exercises a non-sha256 kind.** Measured by
grep over the whole tree: `hashlock.DigestHash256`, `DigestRIPEMD160` and
`DigestHash160` have **zero callers and zero tests**; `hashlock.DigestOf` is
called from exactly one place (my `hashlockLockOf`), always with
`md.KindSha256`, so **three of its four arms never execute**. Nothing tests
`md.HashKind.DigestLen()/Token()/tag()` off the sha256 arm, nothing builds a
20-byte `HashLock`, nothing asserts that `MapKey()` distinguishes
`sha256:X` from `hash256:X`, nothing parses a tagged `hash: hash256: <hex>`
record, and `sysw.HashRecord` has no caller at all.

This matters twice over:

- The non-comparability of `HashLock` exists so alloc-gate padding can never be
  observed — and **there is no test that a 20-byte lock's `Digest()` is 20 bytes.**
  The property the design is built around is unasserted.
- Two doc comments in `hashlock/hashlock.go` assert a KAT that does not exist:
  `DigestHash256` says *"Only the KAT catches it, which is why hashlock_test.go
  measures all four against the vendored corpus rather than against this file"*,
  and `DigestOf` says *"the KAT exercises this and not only the four"*. Both are
  false today. This is the "records are the weak half" shape — a comment
  claiming a gate that was never built.

It is cheap to close: the re-vendored corpus **already carries the columns**.
Each of the 11 `derivation` rows now has `hardened_h_hash256`,
`hardened_h_ripemd160`, `hardened_h_hash160`, `sha256_h_hash256`,
`sha256_h_ripemd160`, `sha256_h_hash160` and a `provenance_kinds` field, and
**no Go struct field reads any of them** — `corpus.Derivation` in
`hashlock/hashlock_test.go` still declares only the six v0.9 columns. The
vectors were vendored and then not wired up.

**D-2. A fifth hash kind would silently become sha256, and a comment claims the
opposite.** `md/compose.go`'s `pathBody` says *"Switched on the KIND so a fifth
kind is a compile error rather than a fall-through."* Go does not check switch
exhaustiveness, so that is not true. Worse, the surrounding helpers all have
`default:` arms returning the **sha256** answer — `HashKind.DigestLen()`
(`default: return 32`), `Token()` (`default: return "sha256"`), `tag()`
(`default: return tagSha256`), and `hashlock.DigestOf` (`default: DigestSHA256`).
A `KindXyz` added to the const block would lower to `tagSha256` with a sha256
digest and no error anywhere — and `pathBody`'s own `var b body` would stay
`nil` in the switch it claims is exhaustive. Four fail-open defaults behind a
comment promising a compile error. In the already-done set, so I changed
nothing; flagging it because it is exactly the class the non-comparable-struct
trick was introduced to prevent.

**D-3. `sysw.ParseHashRecord`'s doc comment is stale.** It still reads
`// ParseHashRecord: exactly 64 lowercase hex characters.` (sysw/composer_records.go:203)
directly above the kind-aware parser that accepts 40 hex for ripemd160/hash160.
In the do-not-change set, so untouched.

**D-4. The corpus sha256 is pinned in two places** — `hashlock_test.go`'s
`corpusSHA256` constant and `testdata/hashlock-v0.8.provenance.json`'s
`"sha256"` field — and this re-vendor updated only the JSON, which is why six
tests were fatal. The duplication is arguably deliberate (a second,
independently edited witness, so a re-vendor that updates only the provenance is
not silent — which is exactly what it caught here). I left the mechanism alone
and noted the intent in the constant's new comment.

**D-5. `md/policy_shape.go` decodes the three new tags to `Hashlock = true` and
discards the digest** (`case tagHash256, tagRipemd160, tagHash160:` at :290,
with no append). `Branch` has only `Sha256Digests`. A non-sha256 composition
therefore cannot round-trip through the shape, and `composerSelfCheck` will
refuse it on the count arm. Fail-closed, not wrong — but the next commit needs
it before any non-sha256 policy can be composed.

**D-6 (benign).** `go.mod` moved
`github.com/btcsuite/btcd/chainhash/v2` from the indirect block to the direct
one. That is a plain `go mod tidy` correction —
`address/taproot_script_path.go:12` has imported it directly all along. Part of
the already-done work; noted so it is not mistaken for drift.

## 6. Files touched

18 files newly touched by me (16 were already dirty from the done work):

```
cmd/emu/composer_js.go                    hashlock/hashlock_test.go
gui/composer_discard.go                   md/compose_test.go
gui/composer_selfcheck.go                 sysw/composer_records_test.go
gui/composer_state_hook.go
gui/composer_backleg_test.go              gui/composer_selfcheck_test.go
gui/composer_copy_test.go                 gui/composer_shape_test.go
gui/composer_discard_test.go              gui/composer_sources_test.go
gui/composer_flow_test.go                 gui/composer_state_hook_test.go
gui/composer_gates_test.go
gui/composer_hashlock_held_test.go
gui/composer_provenance_test.go
```

`git diff --stat` over the whole branch state: **34 files, 917 insertions,
351 deletions**. Nothing committed; nothing staged. No `.json` testdata was
touched, and `sysw/testdata/record_class_vectors.json` is untouched.
