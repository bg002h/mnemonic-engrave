# hashkinds P4 — SPEC-COVERAGE review

**VERDICT: NOT GREEN — 0 Critical / 2 Important / 7 Minor / 3 Nit.**

Scope: `SPEC_hashlock_kinds.md` walked top to bottom against
`seedhammer` `git diff 0562e81..b361b22` (18 commits, 72 files), spec → code,
never code → spec. Repo left unmodified; all measurements re-run on
go1.26.7 with `/scratch/code/shibboleth/.toolchain/go/bin` on PATH.

Facts taken as given per the brief and NOT re-derived: the full `go test ./...`
result, gofmt/vet baselines, the wasm build, both emulator walks, and the
Go/Rust conformance result.

---

## COVERAGE TABLE

Status key: **IMPL** = implemented + gated · **PART** = partially covered
(clause named) · **MISS** = no code/copy/test · **P1/P2/P3** = §9 assigns it to
another phase · **DECL** = deliberately declined with the decision recorded.

### §5 — Data model

| requirement | status | file:line | gating test |
| --- | --- | --- | --- |
| `HashKind`, four values, not the wire `Tag` set | IMPL | `md/compose.go:171-204` | `md/compose_hashkinds_test.go:86` (`TestEveryHashKindTakesItsOwnWireTag`) |
| `HashKind→Tag` a total function | IMPL | `md/compose.go:223` (`tag()`, panics on a fifth kind) | `md/compose_hashkinds_test.go:86` |
| `digest_len()` is the ONLY place a length is written; hex rule = `*2` | IMPL | `md/compose.go:187` | `gui/composer_gates_test.go:913`; `sysw/composer_records.go:229` |
| `HashLock { kind, digest:[u8;32] }`, accessor slices to `digest_len()` | IMPL | `md/compose.go:273-297` | `hashlock/hashlock_test.go:411` (`TestHashLockIdentityIgnoresPadding`) |
| padding unobservable through identity/equality | IMPL | `md/compose.go:273` (`_ [0]func()` makes `==` a compile error), `:299`, `:309` | `hashlock/hashlock_test.go:411` |
| eleven map/set/equality sites re-keyed on the whole `HashLock` | IMPL | `gui/composer_state.go:53,83`, `gui/composer_preimage_plate.go:81-116`; verified `grep -rn 'map\[\[32\]byte\]'` returns **zero** hits tree-wide | `gui/composer_preimage_plate_test.go`, `gui/composer_provenance_test.go` |
| one local definition per crate; `md` and `sysw` share Go's one | IMPL | `md/compose.go:244` — `HashKindFromToken` is the tree's only token map; `sysw` imports it | `sysw/composer_records_test.go:103` |

### §6 — Record grammar

| requirement | status | file:line | gating test |
| --- | --- | --- | --- |
| `hash: [<kind>:] <hex>`, hex at `digest_len()*2` | IMPL | `sysw/composer_records.go:208-241` | `sysw/testdata/record_class_vectors.json` (81 rows) via `sysw/composer_records_test.go:103` |
| absent kind means sha256 | IMPL | `sysw/composer_records.go:220` | vector `hash-valid`; `sysw/composer_records_test.go:133` asserts `Kind()==sha256` |
| tokens are the lowercase fragment names | IMPL | `md/compose.go:206` | vectors `hash-*-valid` |
| **case rejected, never folded** | IMPL | `md/compose.go:244` (no `ToLower`), `sysw/composer_records.go:186` | vector `hash-kind-uppercase` → `Unknown` |
| producer rule: bare for sha256, tagged for the other three | PART | `sysw/composer_records.go:247` (`HashRecord`) | **no test** — see Minor M-5. Only call site in the tree is a sha256 test fixture (`gui/composer_fixtures_test.go:59`) |
| liberal input: `hash:sha256:<64hex>` accepted, never emitted | IMPL | `sysw/composer_records.go:220-251` | vector `hash-sha256-explicit` (Hash) + `HashRecord`'s sha256 arm |
| **fail-closed**: unknown kind → `ClassUnknown`, inert, never read as sha256 | IMPL | `sysw/composer_records.go:224` returns `ErrHashKind` (never falls back); `gui/composer_hash.go:80` filters on `ClassHash` | vectors `hash-unknown-token`, `hash-kind-uppercase`, `hash-*-wrong-width` |
| bare `hash:` still packs byte-identically | IMPL | `sysw/composer_records.go:248` | pre-cycle vectors unchanged |
| both-axes-or-neither on every surface | IMPL | see §7/§13 rows | `gui/composer_gates_test.go:1705`, `gui/composer_hashlock_test.go:1831` |

### §7 — Device

| requirement | status | file:line | gating test |
| --- | --- | --- | --- |
| 7.1 kind gets its **own** pick screen, not a band on `Which hash?` | IMPL | `gui/composer_hash.go:164` | `gui/composer_hashlock_test.go:1612`, `:1654` |
| 7.1 `composerPickScreenMaxRows = 24` untouched, `Which hash?` gains no band | IMPL | `gui/composer_paged.go:276` unchanged | `gui/composer_hash_test.go:106` |
| 7.1 kind **before** the pad; pad accepts `digest_len()*2` | IMPL | `gui/composer_hash.go:198,608,665` | `gui/composer_gates_test.go:913`; `gui/composer_hashlock_test.go:1755` (`…OnTheDevice`, asserts `0 of 40 hex`) |
| 7.1 rows default to sha256, seeded **once** per screen | IMPL | `gui/composer_paged.go:295-356` (`initial` read once, re-home guarded by `homed`) | `gui/composer_hashlock_test.go:1704`, `:1654` |
| 7.1 routes 1/4/5/6 never reach the kind screen | IMPL | `gui/composer_hash.go:585-700`; routes 4/5 hardcode `md.KindSha256` at `:397`, `gui/composer_hashlock.go:147`, `gui/composer_hashlock_plates.go:74,141` | `gui/composer_hashlock_test.go:1300-1420` |
| 7.1 route 6 (`md compose --preset`) | P1 | — | — |
| 7.1 Back: kind screen → source pick, nothing held | IMPL | `gui/composer_hash.go:660,664` (`break` leaves the inner loop) | `cmd/emu/walk_hashlock_phrase.js:495` (`backToWhichHash`, two taps) |
| 7.1 Back: hex pad → kind screen, kind still selected | IMPL | `gui/composer_hash.go:670` (`continue` re-enters `composerHashKindPick(…, kind)`) | `gui/composer_hashlock_test.go:1654` |
| 7.1 Back: phrase screen → kind screen, phrase still dropped | IMPL | `gui/composer_hash.go:646-657` | `cmd/emu/walk_hashlock_phrase.js:495` (executed green) |
| 7.2 §8i **entry** body kind-generic, names no function | IMPL | `gui/composer_copy.go:190` | `gui/composer_gates_test.go:1706` subtest; `gui/composer_hash_test.go:62` fit gate |
| 7.2 §8i **consent** body names the kind(s) | IMPL | `gui/composer_copy.go:220`, called at `gui/composer_consent.go:224` | `gui/composer_gates_test.go:1705`, `:840` |
| 7.3 the hook is kind-aware, hands out no padding | IMPL | `gui/composer_state_hook.go:50,86` | `gui/composer_state_hook_test.go:154` |
| 7.3 the JS bridge + its documented contract | IMPL | `cmd/emu/composer_js.go:15-80` (`{kind,digest}`) | walk `runKindTrial()` executed green |
| 7.3 the walk's own 64-hex helper | IMPL | `cmd/emu/walk_hashlock_phrase.js:351` (`short8` slices from the end) | walk executed green |
| 7.4 `Sha256Digests` → kind-carrying `Hashlocks` | IMPL | `md/policy_shape.go:75,294-318` | `md/compose_hashkinds_test.go:39` |
| 7.4 consumer 1 — consent display loop re-keys on `Hashlock`, names the kind | PART | `gui/composer_consent.go:100-104` | **executed** by `gui/composer_gates_test.go:1705` + the walk; **asserted nowhere** (Minor M-7) |
| 7.4 consumer 2 — UNSORTED regression, *"announced and tested here"* | **MISS** | `gui/composer_consent.go:107` predicate changed with **no comment** | **no test** — see Important I-2 |
| 7.4 consumer 3 — decoded non-sha256 policy now states the 32-byte rule | IMPL | `gui/composer_consent.go:204-225` | `gui/composer_gates_test.go:1705` |
| 7.4 consumer 4 — self-check compares the **kind** as well as the digest | IMPL | `gui/composer_selfcheck.go:134-152` (`p.Hash.Equal(...)`) | `gui/composer_selfcheck_test.go:65`; `md/compose_hashkinds_test.go:39` |
| 7.5 row form re-measured **as a whole**, in pixels, per kind, with a margin | IMPL | `gui/composer_hash.go:52` (kind replaces `hash`) | `gui/composer_hash_test.go:253` (all four kinds, 20 px margin) |
| 7.5 **the kind screen's own four rows get a row in the geometry gate** | PART | `gui/composer_hash.go:138` (`composerHashKindRow`) | not in `TestWhichHashRowsDrawOnOneLine`; only the page-count test — Minor M-4 |
| 7.5 wire tokens stay full; no abbreviated display token | IMPL (DECL) | `gui/composer_hash.go:53-66` records the decision to decline `rmd160` | `gui/composer_hash_test.go:253` |
| 7.6 payload matching becomes kind-aware (4 comparison sites) | IMPL | `gui/composer_hash.go:545` (`composerHashInPayload`), `gui/composer_hashlock.go:177` (`payloadStatesDigest`), `:214` (`hashlockRelationLine`), `gui/composer_hashlock_plates.go:195` (`hashlockPlatesMatch`) | `gui/composer_hash_test.go:227`; `gui/composer_hashlock_test.go:1390` |

### §8 — The 20-byte warning

| requirement | status | file:line | gating test |
| --- | --- | --- | --- |
| fires for ripemd160/hash160 **only** when not device-derived | IMPL | `gui/composer_hash.go:106-116`; call sites `:594` (payload band 1) and `:671` (typed pad) | `gui/composer_hashlock_test.go:1906` (6 rows: 2 fire, 4 silent) |
| provenance from `hashlockHeld`, no new plumbing | IMPL | `gui/composer_hash.go:111` reads `st.hashlockHeld[h.MapKey()]` | same |
| copy exists and is in the copy table | IMPL | `gui/composer_copy.go:934`; table row `gui/composer_copy_test.go:205` | `TestComposerCopyTableCoversEveryBody` (declared count 78 → **81**) |
| the residual imprecision carried into the code | IMPL | `gui/composer_copy.go:928-932` | — |
| the new modal is in the **fit** gate | PART | — | no `assertModalBodyFits` row — Minor M-3 |

### §9 — Phases (phase-4 rows only)

| requirement | status | evidence |
| --- | --- | --- |
| Go ports of 1-3 **and** the device UI as ONE phase | IMPL | one contiguous range, `md`+`sysw`+`hashlock`+`gui`+`cmd/emu` in the same commits |
| re-pin the ms-codec corpus to `0a911f78…` | IMPL | `hashlock/hashlock_test.go:15` + `hashlock/testdata/hashlock-v0.8.provenance.json` (two independent witnesses) |
| §13's device surface: locator `hash` row, confirm + reconciliation bodies, masked pick lead, census row | IMPL | see §13 rows |
| `md compose` sibling options, `me-cli` grammar, `ms hashlock --kind` | P1/P2/P3 | — |

### §10 — Vectors

| requirement | status | file:line | gating test |
| --- | --- | --- | --- |
| round trip per kind, compose→md1→decode | IMPL | — | `md/compose_hashkinds_test.go:39` (wsh); vendored corpus `md/testdata/vectors/keyed_compose_preset_hashlock_gated_{hash256,ripemd160,hash160}.*` |
| Core's measured verdicts/addresses pinned as data | P1 (recorded) | `md/testdata/vectors/*.conformance.json` carries addresses; **F-538** records that nothing in-tree independently closes item 1 | `md/compose_vectors_pin_test.go:91` |
| keyless shape for the three new kinds, "must be measured before this closes" | P1/P2 | not measured anywhere found; subsumed by F-538's wording | — |
| a 40-hex digest through **every** formatter, no panic | IMPL | all four ex-`[56:]` sites now slice from the end: `gui/composer_consent.go:67`, `gui/composer_hash.go:52,533,545`, plus `gui/composer_hashlock.go:281`, `gui/composer_preimage_plate.go:514` | `gui/composer_hash_test.go:253`; `gui/composer_gates_test.go:1705` drives `composerDigestShort` at 40 hex (a `[56:]` would panic) |
| per-kind digest KAT, rows computed outside both implementations | IMPL | `hashlock/hashlock.go:93-126` | `hashlock/hashlock_test.go:363` (`TestDigestKAT`, ≥8 rows × 4 kinds) |
| **the KAT covers the DISPATCH, not only the four functions** | IMPL | `hashlock/hashlock.go:134` (`DigestOf`, the tree's single named map; panics rather than defaulting to sha256) | `hashlock/hashlock_test.go:388-392` exercises `DigestOf` on every row |
| "caller means a package"; **no inline kind switches at call sites** | IMPL | verified: `DigestOf` has exactly one wrapper, `gui/composer_hashlock.go:266` (`hashlockLockOf`); no other switch computes a digest | `gui/composer_hashlock_test.go:1755` |
| fail-closed: unknown token is `ClassUnknown` and inert | IMPL | `sysw/composer_records.go:224` | vector `hash-unknown-token` |
| right kind, wrong length — all four spellings | IMPL | `sysw/composer_records.go:229` | vectors `hash-{ripemd160,hash160,hash256}-wrong-width` + the sha256/40-hex row |
| both spellings of sha256 parse to the same lock | IMPL | `sysw/composer_records.go:220-241` | vectors `hash-valid` + `hash-sha256-explicit` |
| cross-repo token agreement, both provenance pins | IMPL | `sysw/testdata/record_class_vectors.provenance.json` (81), `md/testdata/compose_vectors.provenance.json` (176 files/36 vectors), `hashlock/testdata/hashlock-v0.8.provenance.json` | `sysw/composer_records_test.go:61`; `md/compose_vectors_pin_test.go:91` |
| **the preimage plate, PER KIND**: QR text | IMPL | `hashlock/hashlock.go:252` | `hashlock/methodline_h6_test.go:32` (10 corpus rows, kind column); `backup/hashlock_test.go:446` |
| …its **locator row** | PART | `gui/composer_preimage_plate.go:249` | asserted at sha256 only (`gui/composer_hashlock_plates_test.go:207,247,312`) |
| …a **layout fit** assertion at the worst case | **MISS at the worst kind** | `backup/hashlock_test.go:49` `h6WorstLocator` still carries the pre-cycle kindless `hash  b867db87..edbc96cb` | `backup/hashlock_test.go:93` runs sha256/kindless — Minor M-2 |
| …goldens for the plate bytes | PART | `backup/testdata/hashlock-phrase-qr-v9.bin` moved (7678→7685 B) | sha256 only |

### §11 — Gates that must move deliberately

| gate | status | evidence |
| --- | --- | --- |
| copy table's declared-body count (78) | IMPL | `gui/composer_copy_test.go:402` → **81**, with the three reasons written out |
| §8 spec rows for every new string (`SPEC_wallet_policy_composer.md`) | PART | §8i rewritten (`design/SPEC_wallet_policy_composer.md:769-795`, commit `26049c8c`); **no row anywhere** for `composerCopyHashKindLead` or `composerCopyTwentyByteUnseen` — Minor M-6 |
| the modal fit gate, **incl. the new screens** | PART | `gui/modal_fits_test.go:366,425,445` re-parameterised at `KindRipemd160` (the widest token); **no row added** for the new 20-byte confirm — Minor M-3 |
| `TestWhichHashRowsDrawOnOneLine` | IMPL | `gui/composer_hash_test.go:253` — now all four kinds + a 20 px margin rule |
| the literal `"Type 64 hex"`, code **and** test | IMPL | `gui/composer_hash.go:527` (`composerHashRowHex = "Type a digest"`); zero live occurrences remain |
| `TestWhichHashPageHoldsFiveRows` | IMPL | `gui/composer_hashlock_test.go:1300` — five rows preserved, no new band |
| `composerHexEntry`'s three 64/32 constants | IMPL | `gui/composer_hash.go:199-240`, all derived from `kind.DigestLen()`; the "unreachable" comment re-derived at `:230-236` |
| `hashlockFirst8Last8` (parameter was `[32]byte`) | IMPL | `gui/composer_hashlock.go:281` takes `*md.HashLock` |
| five sha256-hardcoded `me-cli` strings | P3 | — |
| the JS bridge + its documented contract | IMPL | `cmd/emu/composer_js.go:15-80` |
| the walk's 64-hex helper | IMPL | `cmd/emu/walk_hashlock_phrase.js:351` |
| the compose→decode self-check | IMPL | `gui/composer_selfcheck.go:138-151` |
| the cross-language digest KAT | IMPL | `hashlock/testdata/hashlock-v0.8.json` re-vendored, 3 new digest columns |
| confirm modal + reconciliation body | IMPL | `gui/composer_copy.go:638,743` |
| the hex pad's LIVE COUNTER | IMPL | `gui/composer_hash.go:290` — `"%d of %d hex"` from `want` |
| `ms hashlock`'s operator surface | P2 | — |
| the preimage plate's QR text | IMPL | `hashlock/hashlock.go:252`; `backup/hashlock_test.go:36` now delegates instead of re-spelling |
| the preimage plate's **locator row** | IMPL | `gui/composer_preimage_plate.go:249` |
| the plate **fit gate and its goldens** | PART | goldens moved; the fit gate's worst case did not — Minor M-2 |
| the masked plate-pick lead | IMPL | `gui/composer_copy.go:848,852` |
| `me-cli`'s hashlock help text | P3 | — |
| the §8.3 census row | IMPL | `gui/composer_copy.go:907`; copy-table row uses hash160 + longest form words |
| `me bundle`'s plate checklist | P3 / §13.5 | — |
| **both provenance pin files** | PART | both re-pinned, but the compose-vector generator still prints `156` — Minor M-1 |
| the generator's own stale `_comment` count, **"fixed in this cycle rather than filed"** | **MISS** | `scripts/vendor-compose-vectors.sh:29` untouched in the range; still `"if the file count is not 156"`, actual **176** — Minor M-1 |
| `composerPickScreenMaxRows = 24` NOT on the list | IMPL | unchanged |

### §12 — Acceptance (the spec numbers **nine**, not eight)

| # | item | status | evidence |
| --- | --- | --- | --- |
| 1 | each kind composes end to end, address matches **Core's measured** value | P1, gap **recorded** | addresses vendored in `*.conformance.json`; **F-538** records that the corpus derives them from `rust-miniscript`, so nothing in-tree is an independent Core check. Owner "Phase 4 or the sweep" |
| 2 | emulator walk composes non-sha256 **and asserts kind + KAT digest through the hook** | **EXECUTED** | `cmd/emu/walk_hashlock_phrase.js:775` `runKindTrial()`; ran green storing kind `ripemd160`, digest `09e7bb5051d89788fb4e4b374126721dbcc2946b`; the anchor is the corpus's, not the device's |
| 3 | a 40-hex digest passes every formatter without panicking | IMPL | see §10 row; `gui/composer_hash_test.go:253`, `gui/composer_gates_test.go:1705` |
| 4 | an old parser meeting a new kind token fails closed | IMPL | new-parser half vectored (`hash-unknown-token`, `hash-kind-uppercase`); old-parser half holds by construction (pre-cycle body rule is exactly-64-hex) |
| 5 | `ms hashlock` reproduces a non-sha256 plate's digest | P2 | — |
| 6 | per-kind KAT passes **in both languages** | IMPL (Go half) | `hashlock/hashlock_test.go:363`; Rust half is P2 |
| 7 | **a `hash256` wallet survives its own reconciliation screen** | **EXECUTED** | `gui/composer_hashlock_test.go:1831` — 8 rows (2 methods × 4 kinds), flags parsed **out of the drawn string**, re-derived from what an operator would type. Passes |
| 8 | a plate cut for a non-sha256 kind names that kind, QR still 53 modules | PART | modules: `hashlock/methodline_h6_test.go:110` all four kinds ✓. Naming: QR ✓ (corpus rows), locator ✓ at sha256 only. Physically cutting a plate is the operator's; the spec says H6 item 8 need **not** be re-cut |
| 9 | a record made by following §13.2's list rebuilds the descriptor via `md compose` | PART / unevidenced | the list now names the kind (`gui/composer_copy.go:659`); the `md compose` half is cross-repo and nothing records it being run — Minor M-8 |

### §13 — The engraved surface ("nothing here is optional")

| # | requirement | status | file:line | gating test |
| --- | --- | --- | --- | --- |
| 13.1 | kind in the QR text, unconditional | IMPL | `hashlock/hashlock.go:264` | `hashlock/methodline_h6_test.go:32` (4-line corpus rows, third line `hash: <kind>`) |
| 13.1 | kind on the locator's `hash` row | IMPL | `gui/composer_preimage_plate.go:249` | `gui/composer_hashlock_plates_test.go:207,247,312` (sha256) |
| 13.1 | **not** on `method:`, **not** a new row | IMPL | `hashlock/hashlock.go:240` unchanged; `hashlock/methodline_h6_test.go:93` still pins 73 chars | same |
| 13.1 | QR stays at 53 modules | IMPL | — | `hashlock/methodline_h6_test.go:110` |
| 13.1 | the placement is a **measurement** and must be re-measured | PART | — | Minor M-2 |
| 13.2 | confirm screen names the kind | IMPL | `gui/composer_copy.go:643` | `gui/composer_copy_test.go:152`; `gui/walk_copy_anchors_test.go:56` |
| 13.2 | reconciliation screen names the kind **and the flag** | IMPL | `gui/composer_copy.go:772-777` (names `--kind` **and** `--method`) | `gui/composer_hashlock_test.go:1831` |
| 13.2 | the **third** screen — masked plate-pick lead | IMPL | `gui/composer_copy.go:852` | `gui/composer_preimage_plate_test.go:178`; copy table |
| 13.2 | the write-down list gains the kind | IMPL | `gui/composer_copy.go:659` | `gui/walk_copy_anchors_test.go:67` |
| 13.3 | the census row distinguishes the kinds | IMPL | `gui/composer_copy.go:907` | `gui/composer_preimage_plate_test.go:297`; `cmd/emu/walk_hashlock_phrase.js:471` |
| 13.4 | `ms hashlock --kind` | P2 | — | — |
| 13.5 | **both CLI gaps recorded as follow-ups** | **MISS** | nothing in `design/FOLLOWUPS.md`, `RECON_hashkinds_P4_fork.md`, or `CONTINUITY_hashkinds_2026-09-15.md` | — Important I-1 |

### §14 — Falsifiers

| requirement | status |
| --- | --- |
| `golang.org/x/crypto/ripemd160` still present and free | IMPL — `hashlock/hashlock.go:15`; no `go.mod` change needed (F5 held) |
| the other two falsifiers are conditional on future Core/BIP changes | n/a |

---

## FINDINGS

### Critical

None.

### Important

**I-1 — §13.5's two named follow-ups were never filed. Nothing records a
decision to drop them. This is §8's exact shape.**

§13 opens *"Nothing here is optional."* §13.5 names two gaps and says, verbatim:
*"Both are recorded as follow-ups owned by this cycle's plan, not as silent
scope."* The whole content of the requirement is **that they be recorded**.

Measured — neither exists:

```
grep -n "restoreDoc"      design/FOLLOWUPS.md   # no entry for the composer's missing restore doc
grep -n "ripemd160"       design/FOLLOWUPS.md   # 11 hits, none about `me bundle`
grep "^### F-5"           design/FOLLOWUPS.md | tail   # F-534..F-541 — none is either gap
grep -rn "me bundle\|restoreDoc" design/RECON_hashkinds_P4_fork.md \
                                 design/CONTINUITY_hashkinds_2026-09-15.md   # zero hits
```

The two gaps that vanished:

1. `me bundle` emits **byte-identical six-plate output for a sha256 card and a
   `ripemd160` card**, names no preimage plate, and cannot cut one at all. That
   is an operator-facing consequence of the feature this cycle just shipped: a
   ripemd160 wallet's backup does not mention the thing required to spend it.
2. The composer emits **no restore document** (`restoreDoc` occurs zero times in
   `gui/composer*.go`), which the spec says widens F-132's open half.

There is no phase-4 plan document, so "owned by this cycle's plan" has no
home either. Both items are now invisible to the follow-up burndown, which is
precisely the failure mode §8 demonstrated: *"Nothing recorded a decision to
drop it."*

**Fix:** file two `F-5xx` entries in `design/FOLLOWUPS.md` with owning phases,
quoting §13.5. No code change.

---

**I-2 — §7.4's announced decode-side regression was neither announced nor
tested.**

§7.4, consumer 2, verbatim:

> `composer_consent.go:97` — a `hash160`-locked sole unsorted path today prints
> `UNSORTED (EXPERIMENTAL)` and after this change does not. **A regression if
> left implicit; announced and tested here.**

The behaviour change landed:

```go
// gui/composer_consent.go:107  (was len(b.Sha256Digests) == 0)
if sole && !b.Sorted && b.N >= 2 && len(b.Locks) == 0 && len(b.Hashlocks) == 0 {
    out = append(out, "  UNSORTED (EXPERIMENTAL)")
}
```

Neither half of the spec's clause did:

- **Not announced.** The diff carries no comment on that line at all, while
  every other line in the same hunk gained a paragraph. Consumers 1, 3 and 4 of
  the same §7.4 list each got both a comment and a test; consumer 2 got neither.
- **Not tested.** The only `UNSORTED` assertions in the tree
  (`gui/composer_flow_test.go:100,118`) build **key-only** path lists — no
  `Hash` field on any of them. Measured: `grep -rn "UNSORTED" --include=*.go gui/`
  returns five lines, none of which involves a hashlock.

The new behaviour is, as far as I can tell, correct and consistent (the
exclusion now applies to all four kinds instead of sha256 alone). The defect is
that a decode-side behaviour change on the **consent screen** — the screen an
operator agrees to a policy from — shipped with no comment and no gate, against
a spec clause that named both. A future edit restoring `len(b.Hashlocks)` to a
kind-filtered count turns nothing red.

**Fix:** one comment on the predicate and one table row in
`gui/composer_flow_test.go`'s UNSORTED test — a sole unsorted `N>=2` path
carrying a `hash160` lock draws no mark; the same path with no lock does.

---

### Minor

**M-1 — §11's compose-vector generator still prints the stale count it was
explicitly told to fix, now stale by 20 instead of 5.**

§11: *"The compose-vector pin generator also prints a stale count in its own
`_comment` (156 against 161) … this is a **phase 4** item … **It is fixed in
this cycle rather than filed**, because the re-pin touches it anyway and a
generator that prints a wrong count is how the next wrong count goes
unnoticed."*

```
git log --oneline 0562e81..b361b22 -- scripts/vendor-compose-vectors.sh   # empty
grep -n 156 scripts/vendor-compose-vectors.sh md/testdata/compose_vectors.provenance.json
  scripts/vendor-compose-vectors.sh:29:  "if the file count is not 156, …"
  md/testdata/compose_vectors.provenance.json:6:  "if the file count is not 156, …"
python3 -c "import json;d=json.load(open('md/testdata/compose_vectors.provenance.json'));print(len(d['files']))"
  176
```

`md/compose_vectors_pin_test.go:100` was correctly moved to 176, and commit
`9ad1bd7` corrected the *commit message*'s figure — but the generator's own
comment, which §11 names by file and line, was never touched. The exact
prediction in §11 has already come true once inside this cycle.

**M-2 — §10's per-kind plate vectors: the layout fit gate and its worst-case
locator never gained a kind.**

`backup/hashlock_test.go:49` still reads:

```go
var h6WorstLocator = []string{"path 2", "hash  b867db87..edbc96cb", "mk1 stub (template): 1a2b3c4d"}
```

— the pre-cycle, kindless `hash` row (24 chars), while production emits
`"hash  " + Kind().Token() + " " + …` (34 chars at `ripemd160`). So
`TestHashlockWorstCaseFitsAtExactlyOneRung` (`:93`), which §11 names as *"the
preimage plate **fit gate** … §13.1's placement is a layout claim and must be
measured, not asserted"*, measures a plate 10 characters short of the worst one
the firmware builds, and at `md.KindSha256` for the QR. The engrave-side twin of
this exact drift **was** caught and fixed (`engrave/h6_qr_test.go:52-84` now
fuzzes at `KindRipemd160` after the budget was found sampling 16 bytes short);
the backup-side one was not.

I measured the claim rather than leaving it asserted: `CharsPerLine(prodParams,
constant.Font, 3.0)` is **39**, so the 34-character row still draws on one line
and the plate still fits at 3.0 mm with the same row count. **No operator-visible
defect** — the gap is that nothing in the fork proves it, which is what §13.1
said would decay.

Consequentially, the comment at `backup/hashlock_test.go:47` (*"the template
stub (29 characters, the longest locator row this plate produces)"*) and at
`:245` (*"every one of them is 6 to 30 characters"*) are now false: the hash row
is the longest at 34.

**M-3 — the new 20-byte confirm modal has no fit gate, against §11's "incl. the
new screens".**

`composerCopyTwentyByteUnseen(KindRipemd160)` is **276 characters** — the
longest body this cycle adds — and it is drawn through `composerConfirmScreen`.
`gui/modal_fits_test.go` gained three re-parameterisations and **no new row**;
`TestConfirmScreensThisBlockTouchesAreDrawnInFull` does not list it.
`TestTwentyByteWarningFiresExactlyWhereSpecEightSaysIt` asserts only that the
frame contains the heading `20-BYTE HASH`, which is the **first** line — a body
clipped anywhere below it still passes. There is no structural gate forcing a
new `composerCopy*` body into a fit check (`s6b_p7_modal_fit_sweep_test.go`'s
exclusion list does not cover the composer family), so nothing said so.
`composerCopyHashRuleForKinds`'s four-kind form (215 chars) is likewise ungated,
though it draws on the paged consent screen where the risk is lower.

**M-4 — §7.5's kind rows are not in the geometry gate it names.**

§7.5: *"The kind screen's own four rows are bound by the same arbiter … they
get a row in the geometry gate rather than inheriting an assumption from the
screen they were modelled on."* `composerHashKindRow`
(`gui/composer_hash.go:138`) appears in **no** row of
`TestWhichHashRowsDrawOnOneLine` (`gui/composer_hash_test.go:253`), which is the
arbiter §7.5 names. It is covered indirectly by
`TestComposerHashKindScreenDrawsAllFourRows`, which counts tappable rows on the
first page — that catches a wrap large enough to push row 4 off, but asserts
neither "one line" nor the 20 px margin the same cycle established.

**M-5 — §6's producer rule has no Go gate.**

`sysw.HashRecord` (`sysw/composer_records.go:247`) implements the bare/tagged
split correctly, but its only call site in the whole tree is a **sha256 test
fixture** (`gui/composer_fixtures_test.go:59`). Nothing asserts that a
`ripemd160` lock renders `hash:ripemd160:<40hex>` or that a sha256 lock renders
bare — and the record vectors are parse-direction only. Low risk today (the
device packs no payloads; the host is phase 3), but the port is the half a
future device-side packer would use.

**M-6 — two new operator strings have no spec row anywhere.**

§11 requires *"§8 spec rows for every new string"*. §8i was properly rewritten in
`design/SPEC_wallet_policy_composer.md:769-795`. `composerCopyHashKindLead` and
`composerCopyTwentyByteUnseen` have no row there, and `SPEC_hashlock_kinds` §7.1
and §8 do not quote their wording either. Their `verbatim` column in
`composerCopyTable` therefore restates the function's own output — authored in
the same commit — so `TestComposerCopyIsVerbatimFromTheSpec` cannot fail for
them by construction.

**M-7 — the consent screen's kind-bearing hash row is executed but never
asserted.**

`gui/composer_consent.go:100-104` prepends `d.Kind().Token()` to each branch's
hash row. No Go test reads that row (`grep -rn 'composerBranchLines\|composerDigestShort'
--include=*_test.go` → zero hits), and the walk's `drawnToken`
(`cmd/emu/walk_hashlock_phrase.js:378`) makes the kind group **optional and
uncaptured**, so it still matches if the kind is dropped. Dropping the kind here
turns nothing red, on the one screen where the operator commits to the policy.

**M-8 — §12 item 9 has no recorded execution in any phase.**

*"A written record made by following §13.2's list is sufficient to rebuild the
descriptor — the test is `md compose` with nothing but what the operator was
told to write down."* Phase 4's half (the list naming the kind) is done and
gated. The `md compose` half is cross-repo and appears in no report, no
continuity entry and no follow-up — unlike item 1, which has F-538. Item 9 is
not phase 4's to run, but nothing records that it is outstanding either.

### Nit

**N-1** — `gui/composer_hash_test.go:283` leaves `_ = d` after the loop was
rewritten to build its own per-kind locks; `d` is now dead.

**N-2** — `TestComposerHexEntryTakesExactlySixtyFourCharacters`
(`gui/composer_gates_test.go:626`) never calls `composerHexEntry`; it asserts a
local fixture at a 64-character bound that is now true for only two of four
kinds. Its replacement (`:913`) is the real gate. The old one is green, vacuous,
and its name now misstates the rule — the "a new gate makes old tests vacuous"
class.

**N-3** — `design/CONTINUITY_hashkinds_2026-09-15.md:4,23-24` still says *"phases
1, 3 and 4 are not [shipped]"* and *"4 | seedhammer fork | not started"*, while
the record-vector provenance pins mnemonic-engrave `master` `40e1962e` with the
81-row phase-3 fixture and phase 4 is this diff.

---

## WHAT I LOOKED FOR AND DID NOT FIND

Stated so a later reader knows the negative was searched for, not assumed:

- **No second §8-shaped section with zero code.** Every numbered subsection of
  §5, §6, §7 and §13 resolves to at least one production symbol; the only
  zero-implementation normative item found is §13.5's recording obligation (I-1).
- **No kind→sha256 silent fallback anywhere.** `DigestLen`, `Token`, `tag`,
  `DigestOf` and `hashlockLockOf` all panic on an unknown kind rather than
  defaulting; `ParseHashRecord` returns `ErrHashKind` rather than sha256.
  This is the funds-loss path §4 names, and it is closed by construction.
- **No surviving padded-array read.** `grep -rn 'map\[\[32\]byte\]'` is empty
  tree-wide; every hex/elide/compare site goes through `Digest()`, `Equal()` or
  `MapKey()`.
- **No surviving 64-hex assumption in production.** The four pre-cycle `[56:]`
  sites, the pad's three constants, the live counter, the JS bridge contract and
  the walk helper all moved; the three remaining `[56:]` occurrences are sha256
  test fixtures and three are prose.
- **The §12 item 2 and item 7 gates are real gates.** Item 7 parses the flags out
  of the *drawn string* rather than reading the variables; item 2 compares
  against a corpus anchor the device never supplied. Both can fail.
