# S6a R3 pre-review verification pass (cheap net before the adversarial round)

**Scope:** `git diff 9edc641..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(commit `cac34a0`, "fold S6a R2 -- 2C/5I in the R1 fold, plus the spec-coverage
Important"), checked against the fork at `main` = `b8a23bf` (working tree clean,
matches the plan's stated baseline).

## VERDICT: DIRTY — 1 false claim, 0 additional stale claims (same defect is both)

One load-bearing false claim found: **§4.7b's closing sentence says §4.2 adds a
"leading parameter" to `restoreDocFlow`. It does not — §4.2 is unmodified by this
fold (and by any prior fold) and still specifies a single *trailing* `extra
[]string` parameter with `restoreDocScreen(ctx, th, append(lines, extra...))` —
the exact shape this same fold just declared broken for multisig under R2 C-1**
(status line cannot reach slice index 0 through an appended `extra`). As written,
the plan does not actually specify how single-sig's restore document gets its
verify-status line to index 0; §4.2 and §4.7 contradict each other on this point,
and neither section overrides the other because §4.2 was simply never touched by
the R2 fold. This is the same defect class R2 C-1 fixed on the multisig side,
reintroduced by omission on the single-sig side. Everything else checked below is
TRUE and verbatim.

## QUOTED STRINGS

| quote | cited location | verbatim match? | evidence |
| --- | --- | --- | --- |
| "// inventory, and both engraving callers now gate it on this function's own / // caller returning bundleEngraveDone -- so an operator whose engrave died really / // does not reach it, and this modal really is the only screen they get." | `gui/bundle_flow.go:535`-537 | YES | `sed -n '534,537p' gui/bundle_flow.go` — exact match, 3 lines |
| "No plate in this set carries a seed." | `gui/bundle_flow.go:555` | YES | `sed -n '555p'` — exact, `return msg + "No plate in this set carries a seed."` |
| `gui/multisig_restore.go:106` → `restoreDocScreen(ctx, th, append(lines, extra...))` | §1.8/§4.7b | YES | `sed -n '106p' gui/multisig_restore.go` — byte-identical |
| `gui/multisig_build.go:439` → `Only verifyComplete falls through to the restore document.` | §4.7b table row 1 | YES | `sed -n '439p'` — exact |
| `gui/multisig.go:321-322` → `Only verifyComplete falls through; a refusal or an abandon does not loop` | §4.7b table row 2 | YES | `sed -n '321,322p'` — exact across the line break |
| `gui/multisig_verify.go:78-79` → `FOUR OUTCOMES, NOT A BOOL` … `Only verifyComplete may fall through to the restore document.` | §4.7b table row 3 | YES | `sed -n '78,79p'` — exact |
| Assembled seedCapacityMany + seed-on-plates ruling (§4.3) | new §4.3 | YES, byte-identical to shipped `gui/multisig_build_census.go:86-90` | Python string-concat diff of the plan's four fragments vs. `sed -n '86,90p'` → `MATCH` |
| SPEC quote: "restore doc (R0-M2): display-only + optional NFC; master fp + the concrete descriptor + first receive/change address … greps clean of any xprv/private material." | `SPEC_seedhammer_T6a_singlesig_flagship.md:36` | YES (elision `…` honestly marks an omitted parenthetical) | `grep -n "restore doc (R0-M2)"` on the spec file |
| Glyph set `— – · ' ' " " …` (§4.4/§6.2) | ASCII gate description | YES, same 8 codepoints, same order, as the Go literal `"—–·''""…"` | Python `repr()` of plan line vs. `multisig_build_prose_test.go` line 394 |
| `singleSigEngraveCards` label `"ms1 secret share"` | `gui/singlesig_engrave.go:25` | YES | `sed`/grep confirms `label: "ms1 secret share",` at line 25 |
| `numberedLabel("ms1 secret share", i, n)` | `gui/multisig_engrave.go:37` | YES (call site); the companion quote "reads exactly as it always did" is real text but sits at lines 30-31 of the same file, not at line 37 — the single citation anchors an area, not each sub-quote precisely. Not flagged as false (per-brief, citation-line precision is already gated); content is true either way. | `grep -n` both strings |

## CONTROL-FLOW AND COUNT CLAIMS

| claim | TRUE/FALSE/UNVERIFIABLE | evidence (command + output) |
| --- | --- | --- |
| `gui/multisig_restore.go:106` is `restoreDocScreen(ctx, th, append(lines, extra...))` | TRUE | Read of file, line 106 verbatim |
| `restoreDocScreen` opens at `start := 0`, draws `lines[start]` first, `doneBtn` live same frame | TRUE | `gui/singlesig_restore.go` lines 148-150: `start := 0` / `for !ctx.Done {` / `if backBtn.Clicked(ctx) \|\| doneBtn.Clicked(ctx) { return }` immediately, then the draw loop starts at `i := start` |
| `verifyFailed`/`verifyIncomplete` keep the retry loop alive at `multisig.go:337` and `multisig_build.go:453` | TRUE | Both lines read `if res != verifyIncomplete && res != verifyFailed {` verbatim |
| `multisigVerifyResult` has 5 constants, no "skipped"/"never offered" value | TRUE | `const ( verifyComplete...verifyIncomplete...verifyFailed...verifyRefused...verifyAbandoned )` — 5 named values, none named skip/never-offered |
| `singleSigVerifyFlow` has 11 exit points, returns nothing | TRUE | Function signature has no return type. `grep -n "return"` inside the function: 10 explicit `return` statements (lines 69,78,90,98,112,117,125,130,138,146) + 1 implicit fall-through after the final `showNotice` at line 148 = 11 |
| `multisigVerifyIncompleteText` instructs VERIFY AGAIN | TRUE | Body contains `"Choose VERIFY AGAIN on the next screen and type ALL of this wallet's seeds in one pass..."` |
| `seedEntryFlow` is a source picker admitting a payload-borne seed, so "the words you typed" would be false on a payload run | TRUE | `gui/singlesig.go:18-26` own security-spine comment: "seedEntryFlow is the SOURCE PICKER (systemwide payload / keyboard / scan...) ... A payload-borne ClassMnemonic reaches derivation on purpose" |
| `gui/multisig.go:291`/`gui/multisig_build.go:402` gate on `bundleEngraveDone`; `gui/singlesig.go:127` does not; `gui/bundle_flow.go:39` returns immediately after, no gate needed | TRUE | grep + read: two `if bundleEngrave(...) != bundleEngraveDone { return }` sites vs. bare `bundleEngrave(...)` at singlesig.go:127 (followed by verify offer :130, restore doc :136) vs. bundle_flow.go:39 `bundleEngrave(...)` immediately followed by `return` at :40 |
| `restoreDocFlow` has exactly one production call site, zero test call sites | TRUE | `grep -rn "restoreDocFlow("` → only the definition and `gui/singlesig.go:136` |
| `buildPlateInventoryLines` call sites "all 8, measured" (excl. the new singlesig.go:136) | TRUE | `grep -rn "buildPlateInventoryLines"` → exactly 8 existing call sites: `multisig.go:362`, `multisig_build.go:479` (production) + `multisig_build_perseed_passphrase_test.go:134,246,304` + `multisig_build_prose_test.go:369,424,425` (tests) = 8. The 9th (`singlesig.go:136`) is correctly marked "(new)" and excluded from "measured" |
| §4.7b: "the same new **leading** parameter §4.2 adds to `restoreDocFlow`" | **FALSE** | §4.2 (untouched by this fold — confirmed via `git diff` hunk boundaries, which start at old-line 292/313/416/620/657/717, none overlapping §4.2's ~365-391 range) still reads: `restoreDocFlow(...) gains a **trailing** extra []string ... restoreDocScreen(ctx, th, append(lines, extra...))`. That is a trailing, appended parameter — structurally identical to the shape §4.7b/R2-C1 just ruled broken for multisig. No leading parameter is added anywhere in §4.2, and no other section supplies one for single-sig |
| `numberedLabel` leaves a card unnumbered at n=1; multisig labels "ms1 secret share" (call), single-sig labels it identically | TRUE | `gui/multisig_engrave.go:64-67`: `if n <= 1 { return base }`; `gui/singlesig_engrave.go:25` label is the same literal |
| `gui/multisig.go:355` supports "SUPPLY path holds exactly one seed by construction" | TRUE | Line 355 is the comment "ONE SEED, so ONE FACT. This path has a single seed seam by construction..." |
| §5.1(b) three walks break at the census: exact click/pumpUntil line pairs | TRUE | `singlesig_flow_test.go:82`→`:83` ("Card 1 of 3"), `:121`→`:122` ("Card 1 of 2"), `template_engrave_test.go:128`→`:129` ("Card 1 of 3") all confirmed by grep |
| `pumpUntil` never presses; `confirmReviewScreen` loops `for !ctx.Done` on Button1/Button3-or-Center | TRUE | `gui/slip39_polish_test.go:353` definition only calls `frame()` in a loop, no click; `gui/multisig_build.go:1720-1729` confirms `backBtn:=Button1`, `contBtn:={Button3,AltButton:Center}`, `for !ctx.Done {` |
| Exactly 4 tests drive `engraveSingleSigFlow`; the 4th (`TestEngraveSingleSigFlowSeedScrubbed`) aborts before the engrave | TRUE | `grep -rn "engraveSingleSigFlow(ctx"` across `*_test.go` → 4 call sites; the 4th test backs out (`click(Button1)`) at the wallet-type picker, before Engrave Mode/census/engrave |

## SEQUENCE TABLE (§4.7a)

Grounded in `gui/multisig.go:330-343` and the structurally identical
`gui/multisig_build.go:446-459` (both read verbatim, code quoted above). The loop:
first choice is "Verify now"/"Skip" (`!ok || sel != 0` breaks without ever
calling verify → the `S` row); each `multisigVerifyFn` result that is
`verifyIncomplete` or `verifyFailed` re-loops with "VERIFY AGAIN"/"CONTINUE"
(`sel==0` retries, `sel!=0` breaks without a further verify call); anything else
(`verifyComplete`, `verifyRefused`, `verifyAbandoned`) breaks immediately.

Every row in the table is reachable by this mechanism, including both
historically-buggy instances: `failed → abandoned` (R1 C-1: VERIFY AGAIN then
back out mid-retype) and `incomplete → complete` (R2 C-2: VERIFY AGAIN then a
clean pass). Traced `incomplete → failed → complete` explicitly: iteration 1
incomplete (continues), iteration 2 failed via VERIFY AGAIN (continues),
iteration 3 complete via VERIFY AGAIN (breaks clean) — matches the table's
`disagreed` worst / `VERIFIED on a repeat check` print.

**Completeness caveat, not a defect.** The header "every sequence" is a slight
overclaim: the loop's state space is technically unbounded (arbitrarily many
VERIFY AGAIN retries), so sequences like `incomplete → refused`,
`incomplete → abandoned`, or `incomplete → incomplete → complete` are reachable
but not literally listed as their own rows. Working the stated worst-seen
algorithm by hand, though, every such omitted sequence reduces to one of the
five already-printed output strings (severity ranking `disagreed > did-not-
complete > not-verified > verified` is exhaustive and each unlisted sequence's
worst/final maps onto an existing row's bucket). No omitted sequence produces an
output not already covered, and no listed row is unreachable. Not flagged as an
Important — the table is a representative-and-covering set over the five real
outputs, not a literal path enumeration, and it correctly captures both
historical bugs.

## STALE CLAIMS ELSEWHERE IN THE PLAN

Greps run (script substring, exact strings from the finding classes named in the
brief):

- `no signature change` / `does not change signature` → **0 hits**. The only
  place this phrasing exists is the R2 correction itself, quoting the R1 fold's
  wrong claim in order to overturn it (§4.7b: "Round 1 claimed it did not").
- `superset` → 2 hits, both in the corrected §4.7c passage (the new true claim at
  line 337, and the quoted-to-be-overturned R1 claim at 764-765). No stray
  survivor elsewhere.
- `two false` / `TWO false` → 1 hit, in the R2 correction text itself ("THREE
  false-comment sites, not two"). No other section still asserts "two."
- `prepend` (case-insensitive) → 2 hits, both inside the §4.7b passage explaining
  why R1's "prepend into extra" doesn't work. No other section still proposes
  prepending into `extra`.
- `— the same new leading parameter §4.2 adds —` → **this is the one place the
  propagation failed**: §4.2 itself was never edited to match. See the FALSE
  finding above; I count this as the single defect rather than double-counting
  it under both headings.
- `"the words you typed"` → 1 hit, inside the R2 passage explaining why R1's
  wording was wrong (§4.3). Not used anywhere as live prose for a shipped string.
- `"your seed"` → 2 hits, both affirming it as the correct/current wording
  (§4.3's landmine discussion, §4.4's rule 1); no stale reference to a rejected
  alternative surviving as if adopted.

No other unpropagated corrections found.

## Toolchain / environment notes

`go` came from `nix develop --command go version` (go1.26.3 linux/amd64, exit 0,
stderr empty) — used only to confirm the toolchain per the brief's instructions;
no build/test run was needed for this pass since all checks were static
reads/greps against the fork's working tree (`main` = `b8a23bf`, clean).
