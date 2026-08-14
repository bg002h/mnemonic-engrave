# Final fold-check — `468e044`, the fold of all four lens reports

Reviewer: mechanical fold-verification + S0-buildability lens, sonnet,
2026-08-13. Scope, per brief: (1) did `468e044` correctly absorb the four lens
reports, (2) is S0 buildable as written. Out of scope: new lenses, re-reviewing
the spec, re-litigating operator decisions, style, anything in S1–S6 not
touched by this fold. Isolated diff: `git diff 32e543f..468e044 -- design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
(158 insertions / 64 deletions). No separate follow-up commit exists after
`468e044` touching the plan (`git log --follow` confirms `468e044` is the last
plan-touching commit; `15842bb` and `caf4dcc` are a process-diagnosis report and
a `CLAUDE.md` edit, neither touches the plan file) — the line-63 "oracle
versions" contradiction the commit message describes catching was corrected
**within** `468e044` itself, and `fold-propagation-check.sh` confirms it is
gone (see below). Both repos left clean.

## Verdict

**NOT clean — 2 Critical, 2 Important, 3 Minor.** The mechanical parts of the
fold (A2 propagation, the S4/S5 renumbering, the eight-test S0 gate, the I5
split) are done correctly and verified against the real source tree. But the
commit's own title — "8C/14I addressed" — overclaims: **8 of the 14 Important
findings are not in the plan at all**, with no acknowledgment anywhere that
they were seen and deferred. Two of those eight (journeys I1, I2) land
directly on S0's second-payload deliverable, which is under-specified in
exactly the way they predicted. S0's harness deliverable (3) also lost the
concrete API shapes the source report had already worked out, and now only
*names* two APIs rather than specifying them.

## Per-finding fold table

### `multisig-build-repair-plan-lensfold-check.md` (reviews prior fold `4bbaa16`)

| Finding | Landed in `468e044`? | Verified |
| --- | --- | --- |
| C1 — A2 oracle-pinning propagation (3 surviving "by version"/"oracle versions" sites) | **Yes.** | `grep -n -i "by version\|oracle version"` over the current plan: **zero hits**. `fold-propagation-check.sh` with the report's own patterns (`resolves the primary toolchain[^.]*by version`, `print(s)? the (resolved )?oracle versions`, `the walk script must print the oracle versions`) → all **gone**, exit 0. |
| C2 — stale "S4 test 8" cross-reference | **Yes, correctly.** | Now reads `S4's `TestGateDerivesAtTheCardsOwnOrigin` fixture` — named, not indexed. `grep -n "S[0-9] test [0-9]"` over the whole plan: **zero hits**. This also survives the *further* renumbering the fold itself introduced (old S4-test-8 is now test 10), which an index fix would not have. |
| F1 (Important) — interrupted-tail fix only 1/3 landed | **Partial, 2/3.** | (a) `TestReRunMintsByteIdenticalPlates` in S5 — present, unchanged. (b) S6 hardware confirmation — **added**: new S6 item 3, "Confirm the interruption story on hardware." (c) make the interrupted/resumed plate an explicit **S5 Gate** walk requirement — **not present**; S5's Gate line still only says "Trace B completes... by test and by emulator walk," no interruption clause. Not blocking (S6 now covers the substance on hardware, where it matters most), but not a completed fold of the finding as written either. |
| Minor — 5 dropped minors from the *original* adversarial/failure-state reports (A3, A4, A5, M1, M2) | **Still dropped.** | `grep -in "SEALED\|out-of-band\|PublicDataHash\|Back semantics\|rides as a Go"`: zero hits, same as before this fold. Recorded only; not this fold's job to have picked up, but still unacknowledged anywhere in the doc. |

### `multisig-build-repair-plan-lens-journeys.md`

| Finding | Landed? | Verified |
| --- | --- | --- |
| C1 — no cosigner payload in `cmd/emu` | **Yes.** | New S0 deliverable 2, a second js-only payload. Facts it's built on re-verified at `a10d007`: `cmd/emu/sysw_test_payload.bin` is exactly **265 bytes**; `SyswReader()` hardwires `embeddedSyswReader{data: []byte(syswTestPayload)}`; only **2** `//go:embed` directives exist under `cmd/emu` today (`sealed_test_payload.bin`, `sysw_test_payload.bin`) — all match the plan's cited numbers. |
| C2 — no walk harness / input-driving API | **Yes, in substance; see Important #3 below on specificity.** | New S0 deliverable 3. `window.shTap`, `window.shScreen`, `shSysw` all confirmed absent from `cmd/emu` today (`grep -rn` returns nothing) — genuinely new work, correctly scoped to S0. |
| C3 — no artifact-extraction mechanism | **Yes, in substance; same caveat.** | Folded into the same deliverable 3 ("an artifact-extraction API... the engraved md1/mk1/ms1 strings out of a walk") and into S0's Gate ("one end-to-end smoke walk... returns the md1 string"). |
| **I1 (Important) — S3's `sh(wsh)` gate has no matching trace** | **No — not folded at all.** | `grep -n "sh(wsh)"`: only the pre-existing S3 Gate line (still demanding a walk of an `sh(wsh)` build) and S6 item 2. `grep -n "Trace A′\|Trace A'"`: zero hits. §2 still defines only Trace A and Trace B, neither `sh(wsh)`. S3 is untouched by this fold's diff. |
| **I2 (Important) — S4's `both`-slot gate has no matching trace** | **No — not folded at all.** | `grep -n "Trace C"`: zero hits. S4's Gate still reads "Emulator walk of the `both` happy path and of one loud failure" with no trace containing a `both` slot anywhere in §2. |
| I3 (Important) — artifact census can't gate a walk producing no artifacts (needs `shScreen()` checkpoints for S1/S3) | **No.** | §3's census paragraph is byte-identical to the pre-fold version (not in the diff). `grep -n "checkpoint"`: zero hits. |
| M1/M2 (Minor) | Not folded; recorded only, not blocking. | |

### `multisig-build-repair-plan-lens-comprehension.md`

| Finding | Landed? | Verified |
| --- | --- | --- |
| CH-1 (Critical) — passphrase absent from backup/restore doc | **Yes, in substance.** | New S5 paragraph "The backup must say what is NOT in it." Covers both halves of the bounded fix (mode label + restore doc) as a ruling; does not add the specific "asserts the line appears iff a passphrase was set" test the report asked for — the general "tests first" stage convention covers this, so not scored as missing, only noted. |
| CH-2 (Critical) — gate FAIL screen's only visible route silences the check | **Yes, in substance.** | New S4 paragraph "The gate's FAIL screen must not make silencing it the obvious next move," covering all three of the report's required elements (causes, terminal route, suppression warning) in compressed form. Same test-bullet caveat as CH-1. |
| **CH-3 (Important) — slot-source labels are spec-language, ambiguous on screen** | **No.** | `grep -in "Cosigner card (from payload)\|My seed, account\|check it against my seed"`: zero hits. S4's implementation bullet is unchanged verbatim. |
| **CH-4 (Important) — "unambiguous digest" and "match your coordinator" name unperformable checks** | **No.** | The exact criticized phrase — *"show the per-slot keys, or an unambiguous digest of them"* — is still present, unchanged, at line 596. No host-command binding was added, no reworded stub line. |
| **CH-5 (Important) — recovery procedure filed where an interrupted operator can't reach it, and the abort screen still asserts the opposite of what S5 test 7 proves** | **No.** | `grep -in "recovery procedure\|Plates already engraved"`: the S5-test-7 paragraph ("put the recovery procedure in the restore doc...") is byte-identical to before this fold — not in the diff. The abort-warning fold (I5's SHIPS-HERE block) only changed the *secret-plate* DESTROY wording; the *public/wasteful-plate* "discard" contradiction CH-5 flagged is untouched. **This is the closest candidate to "a surviving contradiction requiring a follow-up commit"** referenced in the review brief — but no commit corrects it; it is simply still there. |
| **CH-6 (Important) — multi-seed screens don't name which seed/slot they're for** | **No.** | `grep -in "Seed 2 (for slot\|Passphrase for seed"`: zero hits. |
| **CH-7 (Important) — operator never told the plate count; restore doc doesn't enumerate the set** | **No.** | `grep -in "This will engrave\|Have.*blanks ready\|Backup set:"`: zero hits. |
| CH-8/CH-9 (Minor) | Not folded; recorded only, not blocking. | |

### `multisig-build-repair-plan-lens-spec-coverage.md`

| Finding | Landed? | Verified |
| --- | --- | --- |
| C1 (Critical) — §4.5 walk has nothing to walk with | **Yes** (same fix as journeys C1/C2/C3 — this is the "three of eight Criticals are ONE defect" the commit message names). | |
| I1 (Important) — confinement guard name-keyed, won't cover new material | **Yes.** | New S0 deliverable 1, structural (`//go:embed` discovery + `//go:build js` + identifier-scope check + `checked < 50`-style floor), matches the real, verified name-keyed guard at `cmd/emu/confinement_test.go` / `sysw_test_payload_host_test.go:113-115` (`names := []string{"syswTestPayload", "syswTestDigest", "sysw_test_payload.bin"}`, confirmed verbatim in the source tree). |
| I2 (Important) — source seam absent, S1 actively regresses it | **Yes.** | New S1 paragraph "Preserve the per-key source seam while replacing its picker." |
| I3 (Important) — per-seed passphrase binding untested | **Yes.** | New S4 test 9, `TestPerSeedPassphraseBindsToItsOwnSeed`, with the mutation the report specified (hoist the first seed's passphrase to all seeds). |
| I4 (Important) — frame-receiver security absent from S0 | **Yes.** | New S0 deliverable 4, names `shot_server.py` as precedent and both restrictions (origin pin, flat filenames). |
| I5 (Important) — plate-order ruling over-reaches spec scope | **Yes, cleanly split** — see below. | |
| Minors (8) | Mostly not individually re-verified here (out of scope / not Critical-or-Important); M8's "mark as plan-originated" suggestions remain unaddressed but are explicitly Minor/recorded-only in the source report. | |

## I5 split — verified clean

Per the brief, not re-litigated, only checked for a clean cut: `grep -n -i
"public.*first\|secret.*last\|ms1.*first\|reorder"` finds exactly 3 hits, all
inside or immediately describing the **DEFERRED** paragraph ("Public plates
first, secret last" is a design change... rests on 'ms1-first is inherited
convention, not a ruling'... **Filed for the spec**"). The **SHIPS HERE** block
carries only the DESTROY-not-discard wording and explicitly states "no other
flow's call site changes." No half-reordering language survives outside the
DEFERRED paragraph, and no stray reordering instruction exists elsewhere in S0,
S1, or S6. Clean.

## Propagation and cross-reference sweeps

```
$ bash scripts/fold-propagation-check.sh design/IMPLEMENTATION_PLAN_multisig_build_repair.md \
    'resolves the primary toolchain[^.]*by version' \
    'print(s)? the (resolved )?oracle versions' \
    'the walk script must print the oracle versions' \
    'S4 test 8' 'S5 test 7'
== propagation check: IMPLEMENTATION_PLAN_multisig_build_repair.md ==
  gone   resolves the primary toolchain[^.]*by version
  gone   print(s)? the (resolved )?oracle versions
  gone   the walk script must print the oracle versions
  gone   S4 test 8
  gone   S5 test 7
   no superseded phrasing survives
EXIT: 0

$ grep -n "S[0-9] test [0-9]" design/IMPLEMENTATION_PLAN_multisig_build_repair.md
(zero hits — no index-based cross-references remain anywhere in the document)
```

Numbered-list sequencing swept by hand for every stage (S1: 1–7, S2: 1–4, S3:
1–2, S4: 1–10, S5: 1–8): all sequential, no gaps, no duplicates. S0's "Gate.
All eight tests pass" is a real, countable claim — the Tests-first list has
exactly 8 entries.

## S0 buildability, deliverable by deliverable

1. **Confinement guard, structural.** Buildable. Order constraint ("1 precedes
   2, and the order is load-bearing") is explicit and justified. Points at a
   real, existing pattern to generalize (`cmd/emu/confinement_test.go`'s
   `guarded`/`allowed` + `checked < 50` floor — verified present at `a10d007`).
   The red-then-green mutation sequence is narrated clearly enough to build
   without more.
2. **Second js-only payload.** Buildable *as a payload*, but **under-specified
   relative to what the plan's own later gates need.** It anticipates Trace A
   and Trace B by name ("Trace B needs several cards") but §2 defines only
   those two traces — no `both`-slot scenario, no `sh(wsh)` build — and the
   deliverable's only guidance on scope is "for which traces," deferred to
   whoever writes it. Since S3's Gate demands an `sh(wsh)` walk and S4's Gate
   demands a `both`-slot walk (both unchanged, still present), an implementer
   who builds deliverable 2 from this text alone, plus §2 as written, produces
   a payload that satisfies S1/S2/S5 and fails S3/S4 later — reproducing
   journeys I1/I2 exactly, just one stage later than where the review found
   them. **One sentence that would fix it:** "Include a card whose account key
   derives from a seed also on the payload (for S4's `both`-slot walk), and
   either an `sh(wsh)`-template card or a stated decision that S3's gate is
   satisfied by a screen checkpoint instead of a full engrave."
3. **Walk harness (input-driving + artifact-extraction APIs).** Buildable, but
   the brief's suspicion is correct: **named, not specified.** The source
   report (journeys C2/C3) had already worked this out concretely —
   `window.shTap(x, y)` / `window.shScreen()` returning the screen title,
   `cmd/emu/drive_js.go`, `design/journeys/walk_multisig.py` for the driver,
   and `shToolpath.strings()` plumbed through the existing `gui.PlateAware`
   hook (`gui/plate_hook.go:49`) for extraction — and the fold flattened all of
   it into "an input-driving API, an artifact-extraction API... and a
   `shSysw`-style injection point." None of `shTap`, `shScreen`, or `shSysw`
   exist in the tree today (confirmed by grep at `a10d007`), so this is real
   new work with real design choices (raw pixel taps vs. semantic input;
   string extraction vs. geometric SVG comparison) that the plan no longer
   makes for the implementer. **One sentence that would fix it:** restore the
   concrete shapes — "Concretely: `window.shTap(x,y)` and `window.shScreen()`
   (screen title) for input/checkpoint; `shToolpath.strings()`, plumbed through
   `gui.PlateAware`, for extraction."
4. **Frame receiver security.** Buildable — names a real precedent file and
   both concrete restrictions.
5. **Oracle resolution by source commit.** Buildable — concrete mechanism
   (pinned checkout or checked hash), concrete gate-record requirement.
6. **Published-BIP vectors.** Buildable — concrete shape, concrete precedent
   reports to model, explicit "open the sources first" instruction.
7. **`address_test.go` provenance.** Buildable — clean binary choice (cite or
   replace).
8. **md vector re-pin.** Buildable — concrete version delta, concrete gate,
   measured fact (zero byte drift) stated rather than assumed.

**S0's own Gate** ("all eight tests pass... confinement mutation demonstrated
red then green... prints resolved oracle commits... one end-to-end smoke walk
... returns the md1 string") is internally checkable and self-consistent: the
smoke walk only needs Trace A, which deliverable 2 as written does support.
The gap in deliverable 2 does not block S0's own gate — it surfaces one stage
later, at S3 and S4, which is exactly why it reads as buildable in isolation
while still being a real defect.

**Order constraint (1 before 2):** clearly stated, well justified, not a
concern. **S0's gate:** checkable, no unevaluable clause.

## Findings

1. **[Critical] The commit's "14I addressed" claim is false for 8 of 14.**
   Comprehension's CH-3, CH-4, CH-5, CH-6, CH-7 and journeys' I1, I2, I3 are
   not in the plan text anywhere, and nothing in §4 ("What is NOT in this
   plan") or §5 ("Known blind spots") acknowledges them as seen and deferred.
   This is the same "silently dropped, unacknowledged" failure this cycle's
   own `lensfold-check.md` flagged in the prior fold — repeated here at larger
   scale (8 of 14 vs. 5 minors) and at Important rather than Minor severity.
   **Fix:** either fold each of the 8, or add one line per item to §4/§5
   naming it as deliberately deferred and why — the silence, not the
   deferral, is the defect.
2. **[Critical] Two of those eight (journeys I1, I2) land directly on S0.**
   S3's `sh(wsh)`-walk gate and S4's `both`-slot-walk gate are both still
   present, unchanged, and neither §2's traces nor S0 deliverable 2 provide a
   fixture that satisfies them. An implementer building S0 tomorrow morning
   from this text alone will not know to build a `both`-matching card or
   `sh(wsh)` coverage into the payload, and will discover the gap only when
   S3/S4 are reached — the exact "first-time question, an hour to find by
   trying it" pattern `15842bb`'s process diagnosis names as this cycle's root
   cause. **Fix:** the one-sentence addition to deliverable 2 above.
3. **[Important] S0 deliverable 3 names two APIs instead of specifying them.**
   The source report already had concrete signatures and file names; the fold
   dropped them in favor of prose. Confirmed none of `shTap`/`shScreen`/`shSysw`
   exist yet, so this is real design work the plan no longer does for the
   implementer. **Fix:** the one-sentence restoration above.
4. **[Important] lensfold F1's part (c) — an explicit interrupted/resumed-plate
   walk requirement in S5's Gate — never landed**, though parts (a) and (b)
   did. S6 item 3 covers the substance on hardware, which may be an adequate
   substitute, but the plan doesn't say that's a deliberate substitution.
5. **[Minor, recorded only]** CH-1/CH-2 are folded in substance but without the
   specific new test bullets their source reports asked for; the general
   "tests first" stage convention covers this, so not scored higher. The 5
   original adversarial/failure-state Minors (A3, A4, A5, M1, M2) remain
   dropped and unacknowledged, as they were before this fold — not this fold's
   obligation, but still open. No follow-up commit exists after `468e044`
   touching the plan; the line-63 contradiction the commit message describes
   was self-corrected inside `468e044`, verified gone by the propagation
   script above.
