# S5 whole-diff gate — LENS: do the gates actually run?

Reviewer: sonnet subagent (mandatory post-implementation gate, pre-merge of
`s5-multislot` into `main`).
Tree: `/scratch/code/shibboleth/wt-s5`, frozen at `7da66bd7841b96f879b7f5a957c66ad16744e3d2`.
Scope: strictly the LENS brief — "is every assertion in the walks and gate
records SATISFIABLE and actually EXERCISED?" Read-only throughout; the frozen
tree was never written to. A local static file server (`python3 -m
http.server 8791`, killed at the end of the session) was used only to serve
the already-built `cmd/emu/emu.wasm` + `index.html` to a headless browser
(Playwright MCP) so the flagship walk could be **actually executed**, not just
read.

## Headline result

**walk_trace_b.js was run live, end to end, against the frozen tree, in a real
headless Chromium instance, by this review — not inferred, not taken from a
commit message.** It completed in 395s with `ok: true`, and its output
(`census.strings`, 17 entries, and `digests`, 17 entries) is **byte-for-byte
identical, in the same order**, to the strings and plate digests already
committed in `oracle/gaterecords/S5-trace-b.record.json` /
`S5-trace-b.expect.json`. This is the strongest form of verification the LENS
asks for: the committed gate record is not stale prose, it reproduces.

Commands/actions run to reach this result are listed under each item below so
the claims are re-derivable.

## (a) Is there any automated path that runs a walk_*.js at all?

**No. Confirmed by direct inspection, not inference.**

- `.github/workflows/test.yml` runs, in order: `go test ./...` (host),
  `go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/` (compiles the
  live-oracle test binaries and runs nothing, by design, per its own comment),
  `./scripts/test-32bit.sh`, and `GOOS=js GOARCH=wasm go vet ./cmd/emu/`. The
  last of these is a **type-check of the emulator package**, explicitly not a
  build+run: the workflow's own comment says so ("A gate whose instrument does
  not compile is not a gate" — about `go vet`, not about walking it).
  Nothing in this workflow starts a browser, a wasm runtime, or Node against
  `cmd/emu`.
- `grep -rn "playwright\|puppeteer\|chromedp\|headless"` across the whole tree
  (excluding `node_modules`) turns up zero automation — every hit is a Go
  doc-comment using the English word "headless" to describe an in-process test
  harness (`gui/unlock_flow.go`, `seal/open.go`, etc.), never a browser driver.
- `cmd/gaterecord/main.go`'s own doc comment spells out the **only** way a
  walk's output ever becomes a gate record: "Build and serve cmd/emu on a
  fresh port … open index.html, and drive the walk fire-and-forget … poll
  `window.__walk`, then save `JSON.stringify(window.__walk)` to a file", then
  run `go run ./cmd/gaterecord …` by hand. There is no `-run`/CI wrapper for
  this step anywhere in the repo.
- `find . -iname walk_*.js` → the 5 files under `cmd/emu/`; `grep -rn
  "walk_"` across `*.go`/`*.sh`/`*.yml` turns up only **doc comments**
  referencing the filenames (`pace.go`, `gaterecord_anchor_test.go`,
  `needle_test.go`, `gui/*.go`, `oracle/record.go`, `cmd/gaterecord/main.go`)
  — never a `go:embed`, `exec.Command`, or subprocess invocation.

So: a walk runs only when a human (or, as here, a reviewing agent) opens a
browser and drives it by hand. This is a known, explicitly-documented design
choice (`oracle-live.sh`'s header explains the analogous decision for the
Rust-oracle checks: "don't skip, don't silently pass, and don't require a
toolchain CI can't have" — the same three-way choice was made for walks by
making them **not exist** as a CI step at all, only as a human procedure whose
*output* — the committed `.record.json`/`.expect.json` pair — is what CI then
checks byte-for-byte). It is exactly the gap Critical #4 of this cycle
(2 walks broken, 3 of 4 breakages from a stage that closed GREEN, CI blind)
came from, and precisely why this LENS gate exists. See the new finding under
"Findings" below: one piece of automated defense-in-depth that *could* exist
(a per-stage "this stage must have a record" test, which S0 already has) is
missing for S5.

## (b) walk_trace_b.js: is every assertion satisfiable and actually exercised, given `S5-trace-b.inputs.json`?

**Yes — actually exercised, live, this session.** Method:

1. Served `cmd/emu/` (already built: `emu.wasm` is 9,947,743 bytes, matching
   the "ALREADY MACHINE-VERIFIED" build-gate note) over `http://127.0.0.1:8791`.
2. Navigated a headless Chromium tab there (Playwright MCP
   `browser_navigate`); console showed the expected boot log, zero JS errors
   besides an irrelevant `favicon.ico` 404.
3. `await import('./walk_trace_b.js')` and called `w.run()` with its
   documented defaults (`n=4, k=3, held=[0,1,2], picks=[skip,skip,skip],
   typedSeedFor="@2"`) — i.e. exactly `S5-trace-b.inputs.json`'s tuple
   (n=4, k=3, slot_order [0,1,2,3], the operator holding @0/@1/@2, card @3
   filling the fourth slot).
4. Polled `window.__done` / `window.shScreen()` every ~15-40s (screenshots of
   progress: keyboard entry of master B's 12 words with post-condition checks
   on every letter, then 17 consecutive "Engraving plate…" screens with
   hold-to-confirm gestures) until completion at `elapsedSec: 395`.
5. Read back `window.__b` in full.

Result: `ok: true`. Every term `ok` depends on was independently true:
`proven` contains all 5 flow-defining needles including the two S5-only ones
(`"Do you hold another slot?"`, `"Which other slot is yours?"`, the second one
twice — correct, `held` has 3 slots so the "one more?" loop runs twice);
`claims.multiAccount` and `claims.cosignerSlot` both true (the Key-sources
review really did read `"@1  yours: derived from your seed for @1, account
1"` and `"@3  a cosigner: payload card 4, taken as supplied"`);
`cardsGathered=4 > openSlots=1`; `censusHeld=true` (the census screen's "This
engraves 17 plates." matched `census.strings.length===17`);
`tail.reachedVerifyOffer=true`; `restoreDoc` non-empty (113 chars, reads
"Type: P2WSH 3-of-4 multisig (sorted) Descriptor: wsh(sortedmulti(3,
xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1ws…"); `census.unattributed===0`;
`census.announced (51) >= strings.length (17)`; `digests.length (17) ===
census.strings.length (17)`; `assertNoNFC` returned 0 both at entry and after
the restore doc (no card crossed the emulated reader — everything really did
come from the payload + the keyboard, as Trace B claims). No assertion in the
689-line driver was unreachable or vacuous; the fixture in
`S5-trace-b.inputs.json` drives the walk to completion exactly as documented.

Cross-check against the committed record: `window.__b.census.strings` (17
entries) is **character-for-character identical, same order**, to both
`oracle/gaterecords/S5-trace-b.record.json`'s `walk.census.strings` and
`S5-trace-b.expect.json`'s derived `artifacts[].string` list.
`window.__b.digests` (17 entries) is likewise identical to
`record.json`'s `walk.plate_digests`. The gate record was not just plausible —
it is reproducible on demand from the frozen tree.

## (c) Do the other three walks still parse and reference symbols/screens that exist?

Checked mechanically, not by re-running all three live (time-budgeted; see
scope note below).

- `node --check` on all 5 `walk_*.js` files: syntax OK, all five.
- `go test ./cmd/emu/ -run
  'TestWalkNeedleLiteralsAreAllPinned|TestBuildFlowNeedlesHaveExactlyOneProductionSite|TestWalkOkContainsNoDriverSuppliedPlateCount'
  -v` (ran live, this session): **PASS**, all three.
  `TestWalkNeedleLiteralsAreAllPinned` globs every `walk_*.js` and reports, per
  file, how many `NEEDLE_*` declarations are unpinned: `walk_build_policy.js:
  11, 0 unpinned`, `walk_s3_nested.js: 13, 0 unpinned`, `walk_s4_gate.js: 11, 0
  unpinned`, `walk_trace_a.js: 0, 0 unpinned` (it has none — by design, per
  `needle_test.go`'s own comment, it drives Engrave Bundle and anchors on
  content strings outside the buildFlowNeedles convention),
  `walk_trace_b.js: 13, 0 unpinned`. `TestBuildFlowNeedlesHaveExactlyOneProductionSite`
  passed, meaning every needle string these 4 build-flow walks reference still
  has **exactly one** production call site in `gui/`, in the file the walk's
  comment claims.
- This is real evidence, not weak evidence: this exact mechanism is what S5.D
  (commit `42a0b99`) used to *catch* the 4-screen drift that had broken
  `walk_build_policy.js` and `walk_s3_nested.js` after S4 landed (documented
  verbatim in that commit's message, with the actual timeout error text from
  each broken run). The same mechanism now reports 0 unpinned needles across
  all 5 walks on the frozen tree.
- Also spot-checked one ambiguous-looking case by hand:
  `grep -rn -F "key on a card?" gui/*.go` (excluding `_test.go`) returns 3
  hits, but 2 are doc-comments (`multisig_build_slots.go:506,545`) and only 1
  is the actual `fmt.Sprintf` call site (`:555`) — not a defect, and consistent
  with `TestBuildFlowNeedlesHaveExactlyOneProductionSite`'s PASS (that test
  parses the AST and only inspects `*ast.BasicLit` strings, so comment
  mentions don't count, per F-184's fix documented in the same commit).

**Scope note, stated per the CLAUDE.md convention of naming what a gate does
not cover:** I did not personally re-execute `walk_build_policy.js`,
`walk_s3_nested.js`, or `walk_s4_gate.js` live in the browser this session
(each is ~3-4 minutes; budget went to the flagship Trace B run, which the LENS
brief calls out by name). Their live-run numbers ("ok true, 9 plates…", "ok
true, 7 plates…" etc.) are attested only in commit `42a0b99`'s message, which
is a *record*, and per this repo's own stated lesson records are the weaker
half. What I verified independently and mechanically is that (i) they parse,
(ii) every needle they anchor on still has exactly one production site in the
file claimed, and (iii) `gaterecord_anchor_test.go` (below) exercises the
*existing* S0 record end to end. That is real coverage of "would a screen
rename go unnoticed" but it is not the same as watching all four walks
complete. If the merge gate wants that closed too, re-running the remaining
three live is the natural next step and would take roughly 12-15 more minutes
of wall clock.

## (d) Counting the "17/17, 2 ms1 + 7 mk1 + 8 md1" claim

Counted programmatically, not by eye:

```
$ python3 -c "
import json
d = json.load(open('oracle/gaterecords/S5-trace-b.expect.json'))
from collections import Counter
print(Counter(a['kind'] for a in d['artifacts']), len(d['artifacts']))
"
Counter({'mk1': 7, 'ms1': 2, 'md1': 8}) 17
```

`2 + 7 + 8 = 17`, matching the claim exactly. `record.json`'s
`walk.census.strings` has 17 entries in the same kind/order sequence
(ms1, ms1, mk1×7, md1×8) as `expect.json`'s `artifacts[]`, confirmed both by
direct JSON diff and by `go test ./oracle/... -run
TestEveryGateRecordCensusMatchesItsCommittedExpectation -v`, which logged
`S5-trace-b.record.json: 17 committed artifact(s) matched the engraved census
byte for byte`. `CompareCensus` (oracle/expect.go:851) is genuinely
order-sensitive (`want[i].String != got[i]`, not a set comparison) and refuses
to pass on 0 comparisons ("nothing was compared, so this check passed by
checking nothing") — not a vacuous check.

And, as noted above, this session's own live walk run reproduced the same 17
strings independently.

## (e) gaterecord_anchor_test.go's converse (built-policy) arm

Ran live: `go test ./cmd/emu/ -run TestGateRecordStringsAreRecordsOfTheCardsPayload -v`
→ **PASS**, logging `anchored 6 engraved mk1 string(s) across 1 gathered
record(s) to the payload's own chunks; 7 minted mk1 string(s) across 1
built-policy record(s) are correctly NOT payload records`.

This confirms the converse assertion is **not vacuous**: `built` (records
whose `inputs.json` `expect.kind != "cosigner-cards"`) is 1
(`S5-trace-b.record.json`, kind `"built-policy-full"`), and `minted` (mk1
plates in that record that are *not* verbatim payload chunks) is 7 — matching
the 7 mk1 entries counted in (d). Had the built arm been unreachable (e.g. if
S5-trace-b.record.json didn't exist, or if its `inputs.json` were miscoded),
the test's own `if built > 0 && minted == 0 { t.Fatalf(…) }` guard would have
fired on an all-md1/ms1 census; instead `built=1, minted=7` is real, exercised
coverage of the "a built policy's own key card must never equal a payload
record" property this cycle's process notes call out as new-since-S5.

## Findings

### Important — S5 has no analogue of `TestS0GateHasARecord`; the flagship gate's continued existence is unenforced

- **File:** `oracle/record_test.go`
- **What's there:** `TestS0GateHasARecord` (line 359) is documented as "the
  last clause of S0's gate… It never skips… the record is a committed file,
  and the question is whether it is there," and it hard-fails if
  `stages["S0"]` is empty. `grep -n "func Test" oracle/record_test.go | grep
  -i gate` shows exactly two gate-shaped tests in the file:
  `TestS0GateHasARecord` and `TestEveryGateRecordOnDiskVerifies` (the latter
  iterates whatever records *are* on disk — it proves nothing about which
  stages *should* have one). `grep -rn "S5" oracle/*.go` (excluding
  `_test.go`) turns up zero hits naming stage S5 as mandatory; the only "S5"
  matches are doc-comment prose in `oracle/expect.go` describing what S5's
  record *contains*, not that one must exist.
- **Failure scenario:** `S5-trace-b.record.json` (or its `.expect.json`,
  `.walk.json`, or `.inputs.json` sibling) is deleted or never re-committed
  after a future change to the build-policy flow, and nobody notices, because
  `TestEveryGateRecordCensusMatchesItsCommittedExpectation`,
  `TestEveryGateRecordOnDiskVerifies`, and
  `TestGateRecordStringsAreRecordsOfTheCardsPayload` all iterate
  `Records(GateRecordsDir)` — a **directory listing of what's already there**
  — rather than asserting a specific stage is present. Every one of those
  tests, and the whole S5 test suite (`go test ./...`, verified: exit 0),
  would stay green with S5's record silently gone. This is precisely the
  shape Critical #4 named this cycle ("two emulator walks were broken and CI
  could not see it") one layer up: not a broken walk this time, but a walk
  gate's *evidence* that could vanish with nothing to say so.
- **How verified:** `grep -n "func Test" oracle/record_test.go | grep -i
  gate` (2 hits, only S0 is name-checked); `grep -rn "S5" oracle/*.go` (no
  hit outside doc comments); confirmed `StagesRecorded()` (oracle/record.go:414)
  itself is generic and *would* correctly bucket `S5-trace-b.record.json`
  under key `"S5"` (its `stage` field is literally `"S5"`, confirmed via
  `python3 -c "import json; print(json.load(open('oracle/gaterecords/S5-trace-b.record.json'))['stage'])"`
  → `S5`) — the mechanism to add an `if len(stages["S5"]) == 0 { t.Fatal(…) }`
  clause already exists and is proven to work for S0; it simply was never
  added for S5.
- Not on the already-filed list (F-189..F-195) and not one of this cycle's
  four named Criticals — a new instance of the "the gate's own presence is
  unenforced" class, one level more abstract than the walk-breakage Criticals
  already fixed this cycle.

No other findings survive. Every claim in the S5 delivery list that this LENS
was scoped to check — the model changes, the engrave-tail ordering and
dedupe-by-string, the multi-select picker, the per-leg verify obligation
carrying both the slot set and the md1, the supply-path dedupe, and the
Trace B walk itself — is not only internally consistent on reading but was
independently, mechanically reproduced this session: live in a real browser
for the walk, live via `go test` for every supporting gate/needle/anchor test
named in the brief.

## Commands run (for reproduction)

```sh
export PATH="/nix/var/nix/profiles/default/bin:$PATH"
cd /scratch/code/shibboleth/wt-s5

# (a) confirm no CI/automation runs any walk
grep -rn "walk_" --include='*.go' --include='*.sh' --include='*.yml' . | grep -v node_modules
grep -rn "playwright\|puppeteer\|chromedp\|headless" --include='*.go' --include='*.sh' --include='*.yml' .

# (c) needle/parse checks, all walks
for f in cmd/emu/walk_*.js; do nix develop --command node --check "$f"; done
nix develop --command go test ./cmd/emu/ -run \
  'TestWalkNeedleLiteralsAreAllPinned|TestBuildFlowNeedlesHaveExactlyOneProductionSite|TestWalkOkContainsNoDriverSuppliedPlateCount|TestGateRecordStringsAreRecordsOfTheCardsPayload' -v

# (d) count expect.json kinds
python3 -c "
import json
from collections import Counter
d = json.load(open('oracle/gaterecords/S5-trace-b.expect.json'))
print(Counter(a['kind'] for a in d['artifacts']), len(d['artifacts']))"

# (d) byte-identity gate
nix develop --command go test ./oracle/... -run \
  'TestEveryGateRecordCensusMatchesItsCommittedExpectation|TestVendoredExpectationsWereDerivedFromThePinnedToolchain|TestS0GateHasARecord' -v

# (b) LIVE re-run of the flagship walk (read-only static server, killed after)
cd cmd/emu && (python3 -m http.server 8791 --bind 127.0.0.1 &) 
# ... Playwright MCP: navigate http://127.0.0.1:8791/index.html,
#     import('./walk_trace_b.js').run(), poll window.__done, read window.__b
# result: ok:true, 17/17 strings + 17 digests byte-identical to the committed record
pkill -f "http.server 8791"
```
