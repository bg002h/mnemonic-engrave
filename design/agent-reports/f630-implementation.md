# F-630 implementation report — T1, T2, T3 and T4's local gates

**Agent:** single implementer, executing `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md`
(GREEN, 0C/0I after six review rounds).
**Worktree:** `/scratch/code/shibboleth/sh-worktrees/f630`, branch `f630-xpub-header-sync`,
based on fork main `95716e97`.
**Primary (read-only):** `/scratch/code/shibboleth/descriptor-mnemonic` at `b2c5d693`.
**Commits:** `da35965` (T1), `ea3c825` (T2), `a126070` (T3). `git status` clean.
**NOT DONE, by instruction:** T4's push. That step is the controller's.

Every number below was produced by a command in this worktree, not read from
the plan. Where a measurement disagrees with the plan it is called out under
**What the plan got wrong**.

---

## Acceptance, against the brief's list

| acceptance item | plan | measured | |
| --- | --- | --- | --- |
| T1 against the stale corpus | 44 fail / 2 pass (`keyed_tr_keyonly`, `keyed_wpkh`) | **44 fail / 2 pass**, and the two passing names are exactly those | ✅ |
| after T2, same run | 5 pass / 41 fail | **5 pass / 41 fail** (2 correct-header + 3 pinned-legacy) | ✅ |
| after T3 | 43 correct-header + 3 pinned-legacy | **46 of 46 pass (43 correct-header, 3 pinned-legacy), 0 fail** | ✅ |
| re-vendor: descriptor-only records | 41 (82 chain entries) | **41 records, 82 chain descriptor entries** | ✅ |
| re-vendor: wholesale records | 3 | **3** — `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`, `keyed_wsh_timelock_hashlock` | ✅ |
| address lines outside those 3 | 0 | **0** | ✅ |
| id lines outside those 3 | 0 | **0** | ✅ |
| files added or removed | 0 | **0** | ✅ |
| script output | `246 files, 50 vectors` | **`vendored 246 files, 50 vectors, primary b2c5d6938432`** | ✅ |

Two further plan measurements independently reproduced:

- `composeVectorNames` counted with `go/ast`: **36 before, 50 after**. The naive
  `grep -o '"[^"]*"'` returns 78 tokens on that file, which is why the plan
  forbids it.
- D2c's "they agree 284/284": the pin-preferring read of all 46 records covers
  **92 chain descriptors carrying 284 bracketed keys**. Exactly 284.

---

## T1 — the gate, RED first (`da35965`)

`md/conformance_keyed_test.go` gains `Keys` on `keyedConformanceRecord` and
`TestKeyedConformanceDescriptorsAgreeWithTheirTemplates`, implementing D1, D1′,
D1″, D2a, D2b, D2c, D3 and D5g's two tiers plus its membership assertion.
`bip380.validChecksum` → `bip380.ValidChecksum` for D1″.

Measured acceptance:

```
descriptor gate: 2 of 46 vectors pass (2 correct-header, 0 pinned-legacy), 44 fail
  --- PASS .../keyed_tr_keyonly
  --- PASS .../keyed_wpkh
```

**Every one of the 44 failures is the header clause and nothing else** — 280
"origin component(s) but the xpub serialises at depth 0" and 280 "ends at child
N but the xpub serialises child 0", with no D1′/D1″/D2/D3 line among them. That
is the right shape: the stale corpus's key MATERIAL is correct and only its
headers are wrong, so D1′ (which matches slots by the 65 bytes) is green over
it, exactly as the plan predicts for the three pinned records.

Three membership `t.Errorf` lines are present and expected — the pinned set is
legitimately empty at T1 — and the acceptance line still prints, which is the
whole reason that assertion is `t.Errorf` and not `t.Fatalf`.

`./bip380/` and `./codex32/` green, `go build ./...` clean.

## T2 — the fixtures boundary (`ea3c825`)

`gui/vector_fixtures_test.go` and `md/vector_fixtures_test.go` created, each
opening with D5b's pairing rule and naming the sites it exists for.
`loadVectorChunks` moved out of `gui/taproot_script_path_test.go`,
`vectorChunksFor` out of `md/duplicate_keys_test.go`, `loadPhraseChunks` out of
`md/conformance_keyed_test.go`. Loaders return `[]byte`. No scanner (D5c).

Ten record-reading sites routed — more than the plan's "six", see **What the
plan got wrong**. Pins created: `keyed_wsh_timelock_hashlock.md1.txt` plus all
three `.conformance.json`. D6's anti-drift rework and the new
`pinnedDuplicateVectors` shape test landed here.

Gate: `ok seedhammer.com/gui 0.458s`; `md` fails **only** on T1's descriptor
gate, now reading `5 of 46 ... (2 correct-header, 3 pinned-legacy), 41 fail`.
All 50 test funcs in the ten touched files: 50 RUN / 50 PASS / 0 FAIL, verified
with `-v` because a filter matching nothing also prints `ok`.

## T3 — widen and re-vendor, one action (`a126070`)

Vectors copied aside first, then the three edits and
`scripts/vendor-compose-vectors.sh /scratch/code/shibboleth/descriptor-mnemonic`.
Diff expectation verified before committing (table above).

```
go test ./md/ ./sysw/ ./mk/     ok / ok / ok
descriptor gate                 46 of 46 (43 correct-header, 3 pinned-legacy), 0 fail
gui, 24 shards                  all 1374 tests ran, RESULT: ok
```

---

## Mutations — every new assertion proven able to fail

Each applied, run, observed red, restored. All seven descriptor shapes the r4
review measured are now closed.

| # | clause | mutation | result |
| --- | --- | --- | --- |
| M1 | D1 header | re-import the pre-0.44.0 `keyed_wsh_multi_2of3` record | `has 4 origin component(s) but the xpub serialises at depth 0` + `ends at child 2147483650 but ... child 0` |
| M2 | D1′ | derivation suffix `/0/*` → `/7/*`, checksum recomputed | `does not reduce to the record's own template` |
| M3 | D1′ | quorum `sortedmulti(2,` → `sortedmulti(3,` | same |
| M4 | D1′ | script type `wsh(…)` → `sh(wsh(…))` | same |
| M5 | D1′ | `chains["0"]` ↔ `chains["1"]` swap, checksums stay valid | same |
| M6 | D1′ | two `sortedmulti()` key positions swapped | same |
| M7 | D1″ | one character of the BIP-380 checksum | `BIP-380 checksum #qgqy50l4 is wrong for the descriptor body` — D1 and D1′ stay green, which is why D1″ is a separate clause |
| M8 | D2a | `keys[@0]` repointed at a foreign key | `key material is in no keys[] entry of the same record` |
| M9 | **D2b** | `@0` swapped CONSISTENTLY in `keys[]` **and** both descriptors | D1, D1′, D1″ and D2a all PASS — the record agrees with itself — and only `in no slot of the Go port's expansion of the same card` + `the Go port's @0 ... appears in no chain descriptor` fire. This is the "uniformly wrong corpus" class, caught by the cross-language clause alone. |
| M11 | D2c | every bracket fingerprint → `deadbeef`, checksums recomputed | `origin bracket fingerprint record/go`, with **D1′ silent** (the reduction drops the bracket fingerprint) |
| M12 | D3 | `@0`'s bracket re-pointed to account `9'` | `origin path for @0 record/go` |
| M13 | D3 allowlist, Go half | pin `keyed_wpkh` at `m/84h/0h/1h` | `the elided-origin divergence is PINNED for this vector` |
| M14 | D3 allowlist, record half | pin its bracket at `[deadbeef]` | `the record's origin bracket is pinned at [deadbeef] ... and now reads [73c5da0a]` |
| M15 | D3 | drop `keyed_wpkh` from the allowlist | the equality clause fires — a non-allowlisted vector that *starts* diverging is caught too |
| M16 | D5g legacy arm | the PINNED `keyed_tr_multi_a` record's `@0` made CORRECT | `pinned record's header is depth 4 child 2147483650 ... which is depth 0 / child 0 under a NON-EMPTY origin` |
| M17 | D5g membership | **the plan's own reproduction.** Step 1: header regression on `keyed_wsh_multi_2of3` → 45 of 46, 1 fail. Step 2: pin that same defective record → **`46 of 46 vectors pass ... 0 fail`**, and the only objection is the membership assertion | the shortest path from a red gate to a green one is to pin the regression, and this is what sees it |
| M18 | ambiguity | `keys[@1]` set to `keys[@0]`'s xpub | `ambiguous slot material ... slots [0 1] carry identical (chain code ‖ compressed pubkey) bytes ... this gate refuses to guess` |
| M19 | T3 edit 3 | smuggle an unpinned `keyed_tr_smuggled.conformance.json` | **CAUGHT** under the widened `isComposeVectorFile`; **PASSES UNFLAGGED** under the old `keyed_compose_` predicate; a `keyed_compose_smuggled` control is caught either way |
| M20 | T3 edit 2a | drop one name from `composeVectorNames` | `pin says 50 vectors, this test knows 49` |
| M21 | T3 edit 2b | file literal 246 → 245 | `pin lists 246 files, want 245` |
| M22 | D5a pairing | revert ONE gui site to a vendored record read | `keyed_tr_multi_a` chain 0 index 0 → `go: bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt` / `rust: bc1pgrupj0fjv79xtj05mzthds4dqzvdhcptx2gzpgzt90uzc86qzfes2a0yhh` |
| M23 | D5b pairing, inverted | md record from the pin + card from `loadPhraseChunks` | `keyed_tr_multi_a: wallet_policy_id` → `go: fe4d264c6e40999b8695329adfe599d9` / `rust: 1f26f9b7cdb8e745c898bb93f1134ba7` |

M22 and M23 reproduce the plan's cited values **byte for byte**, which is the
strongest available evidence that T2's boundary is load-bearing rather than
tidiness.

Two T2-specific mutations, run before the re-vendor:

- **T2-M1** one character of `md/testdata/forkbuilt/keyed_tr_multi_a.md1.txt`
  (`9q2tvyyy`→`9q2tvyyz`) → `the pin and the vendored keyed_tr_multi_a have
  diverged, and the vendored policy is not the one we recorded the primary
  moving to`. Exercises **both** arms of D6.
- **T2-M2** `pinnedDuplicateVectors` repointed at `keyed_compose_wsh_sole_sortedmulti`
  (a wsh policy with no repeated slot) → `no slot appears twice anywhere under
  the wsh ... (slots seen: 3)`. Restored, it logs the real pin's
  `slots at two or more use sites under one wsh miniscript: @[1 2]`.

**Assertions whose failure is not separable.** D1′ depends on D2a's slot
lookup, so M8 (D2a) necessarily also reds D1′, and M12 (D3 path) necessarily
also reds D1′ because the reduction carries the bracket path. Both are recorded
above with every line they produced. No clause failed to fire.

---

## T4 — local gates (the push is NOT done)

| gate | result |
| --- | --- |
| `go vet ./...` | **exit 1, ten findings, every one `testing.ArtifactDir requires go1.26 or later`** — the diagnostic SET is identical to the documented baseline. `scripts/fork-vet-gate.sh` on this worktree: `known gap ... : 10`, `no other vet findings`, exit 0. |
| `gofmt -l .` | five files — `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`. `diff` against the recorded five-file baseline: **identical**. |
| `./md/ ./sysw/ ./mk/` | `ok / ok / ok` |
| whole `gui`, `gui-shard-test.sh ./gui/ 24` | `RESULT: ok — all 1374 tests ran across 24 shards`, wall 36s |
| every other package | 75 packages listed (gui excluded, covered by the shard run): **55 ok, 20 no test files, 0 FAIL** |
| firmware size, SCOPE check | `1622896 code / 32188 data / 31148 bss → 1655084 flash / 63336 ram` at HEAD, and **byte-identical** with `bip380/` reverted to `95716e97`. The export changes nothing that reaches the device. |

Firmware recipe used, from the worktree with nix on `PATH`:
`nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`.

---

## What the plan got wrong

Three items. None is a Critical or an Important; none changed the design; all
are recorded because a future reader will otherwise trip on them.

**1. "one non-test Go file, and only one: `bip380/checksum.go`" — it is TWO.**
T4's scope check says the plan edits exactly one non-test Go file. Exporting an
unexported identifier necessarily edits its call site's *text* too:

```
$ git diff --name-only 95716e97..HEAD | grep -E '\.go$' | grep -v '_test\.go$'
bip380/bip380.go
bip380/checksum.go
```

`bip380/bip380.go:273` is `!validChecksum(desc, checksum)` → `!ValidChecksum(...)`.
The plan's *reasoning* is sound — "exporting an identifier changes no behaviour
and no call site" is true of semantics — but the scope check as written ("If the
size moves at all, something beyond that export was edited") would be read by a
future gate-runner as "exactly one file in the diff", and that test fails on a
correct implementation. The size check itself is unaffected and passed
(byte-identical flash and RAM).

**2. "The six record reads move to the loaders" — there are TEN.**
D5a names three gui sites and D5b names three md files, which is where "six"
comes from. Applying D5b's rule as stated — *every* read of a `keyed_*`
conformance record goes through the loader, and a file that reads a record
takes its card from the pin-preferring loader too — reaches four more:

| site | note |
| --- | --- |
| `gui/policy_address_test.go:261` (`vectorAddress`) | reads a record by name; paired with `loadVectorChunks` at `:296` |
| `gui/composer_policy_address_test.go:48` | ditto |
| `gui/key_card_seating_test.go:40` | ditto |
| `gui/key_card_seating_test.go:243` | ditto |

All four are inert today (none names a pinned vector) but are the exact shape
D5a calls "a fourth site". Routing them is what "a boundary, not three more
call-site edits" means, so they were routed. The md side's three
`loadPhraseChunks` calls in `md/compose_pkh_emit_test.go` were moved to
`vectorChunksFor` for the same reason (that file reads a record at `:37`).
`loadPhraseChunks` survives, used by `md/compose_shape_test.go` and
`md/compose_test.go`, neither of which reads a record.

**3. A restore hazard the plan does not warn about, and it bit once.**
The plan warns that T3 and the re-vendor are one action and that the vectors
directory must be copied aside first — correct, and followed. It does not warn
that **`git checkout -- md/testdata/` is unusable as a mutation-restore while
the re-vendor is uncommitted**: it reverts the re-vendor itself. This happened
on the first mutation attempt and was caught immediately by the restored run
still printing `FAIL`; the re-vendor was re-run (deterministic, same 246/50
output) and every subsequent mutation was restored from a filesystem snapshot,
verified with `diff -rq` against that snapshot before committing. Recorded in
`a126070`'s message as a note for anyone repeating this.

---

## Things the plan got right that were worth the words

- **D5g's `t.Errorf`.** At T1 the pinned set is empty and three membership
  errors print. A `t.Fatalf` there would have aborted before a single vector
  was examined and the 44/2 acceptance — this cycle's whole T1 deliverable —
  would not exist.
- **The ambiguity rule.** 0 of 46 records trigger it, so it costs nothing; M18
  shows it says so rather than guessing when it would.
- **Counting `composeVectorNames` with `ast`.** The naive grep returns 78 on
  that file.
- **The `isComposeVectorFile` third edit.** M19 shows a smuggled unpinned
  `keyed_tr_*` file passing unflagged without it, while the `keyed_compose_`
  control is caught — i.e. the two edits alone would have delivered hash
  coverage with no directory coverage for the 14 newly-pinned vectors.
- **D1′ and D2c being separate clauses.** M11 fires D2c with D1′ silent; M2–M6
  fire D1′ with D1 silent. Neither subsumes the other.

## Residue, unchanged from the plan

- An `xprv` forged with the rendered xpub's header passes every clause
  (secret-handling class; follow-up, not gated).
- The three VENDORED records under pinned names get no descriptor coverage,
  because the gate reads the pin for those names.
- F-529 is narrowed, not closed.
