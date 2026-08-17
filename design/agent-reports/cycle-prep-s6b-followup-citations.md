# Cycle-prep citation verification — F-199, F-204, F-206

**Method:** read-only. Registry repo `mnemonic-engrave` @ `master` = `5fd0b74`
(entries at `design/FOLLOWUPS.md:7094` F-199, `:7193` F-204, `:7233` F-206).
Source repo `seedhammer` (fork `bg002h/seedhammer`) @ `main` =
`b1479a1b38f6b045d27443764c858906e4e6e122`. Both confirmed clean and at the
pinned SHA before reading. All `gui/*.go` citations resolved against the
source repo, not the registry repo. Grepped every symbol both as a bounded
word and as a bare substring to catch prefixed/sibling variants (none found
for `verifyRefused`, `multisigVerifyNoSlotBody`, `passRecord`, or `.legs`).

---

## F-199 — `verifyRefused` dead-ends on a CORRECTABLE readback

**1. `gui/multisig_verify.go:698-702`, string *"Read back one wallet-policy
md1 AND the operator key card(s) (mk1)."***
**DRIFTED-by-54** (pre-settled by controller, confirmed). Now at line 752:
```
752:		showError(ctx, th, "Verify Bundle", "Read back one wallet-policy md1 AND the operator key card(s) (mk1).")
```
Byte-exact match to the quoted string.

**2. That site returns `verifyRefused`.**
**ACCURATE.** Line 753, immediately following: `return verifyRefused`.

**3. "the next screen is the restore document, headed *'If any of them is
missing, this backup is incomplete.'*"**
**ACCURATE**, with a minor looseness in "headed." The string exists verbatim
at `gui/multisig_build_census.go:69`:
```
69:	lines = append(lines, "If any of them is missing, this backup is incomplete.")
```
inside `buildPlateInventoryLines`, which both engrave callers pass into
`multisigRestoreDocFlow` immediately after the verify loop breaks (confirmed
at `gui/multisig_build.go:460-467→491-493` and `gui/multisig.go:345-351→374-376`
— `verifyRefused` fails the loop's continue condition `res != verifyIncomplete
&& res != verifyFailed`, so it falls straight through to the restore-doc
call). It is not literally the *first* line of the inventory (that is `"This
backup is N plates:"`) but is the second line / the completeness statement
immediately following the plate list — "headed" is a loose but not false
characterization.

**4. `gui/multisig_verify.go:82` "pre-existing in main from `b2c3231`, H1".**
**STRUCTURALLY-WRONG.** Checked with git, not current source, per method
rules. At commit `b2c3231` ("fix(gui): multisig verify reads back the
operator mk1 plate (H1)"), `gui/multisig_verify.go` line 82 is **blank**:
```
$ git show b2c3231:gui/multisig_verify.go | sed -n '82p'
(empty)
```
The actual string at that commit is at **line 64**:
```
64:		showError(ctx, th, "Verify Bundle", "Read back one wallet-policy md1 AND the operator key card (mk1).")
```
Note also the wording at b2c3231 was singular — *"operator key card (mk1)"*
— not *"card(s)"*; the pluralization was added later. The substantive claim
("this message is pre-existing in main, not new to this cycle") is TRUE; the
cited line number (82) is wrong — correct is 64.

**5. `verifyRefused` is new in commit `9f93362`.**
**ACCURATE.** `git log --oneline -S"verifyRefused" -- gui/multisig_verify.go`
returns exactly one commit, `9f93362`; the identifier is absent in
`9f93362^`:
```
$ git show 9f93362^:gui/multisig_verify.go | grep -n verifyRefused
(no output)
```
and present (4 return sites) in `9f93362` itself.

**6. `verifyRefused` "also carries two programmer-error refusals (an empty
`expectedSlots`, a missing engraved md1)".**
**ACCURATE** as a description of the two *kinds* of programmer-error
refusal, with a count nuance the brief asked to be surfaced. There are **4**
total `return verifyRefused` sites in `gui/multisig_verify.go` (current
HEAD), not 3:

| line | trigger | message |
| --- | --- | --- |
| 717 | `len(expectedSlots) == 0` (empty obligation list, checked before any card is gathered) | `multisigVerifyNoExpectationBody` |
| 727 | `len(engravedMd1) == 0` (missing engraved policy) | `multisigVerifyNoPolicyBody` |
| 753 | `extractReadbackMd1AndMk1s` fails on the gathered cards (F-199's own site — reachable before any seed is typed) | `"Read back one wallet-policy md1 AND the operator key card(s) (mk1)."` |
| 854 | `verifyFreshSlots` returns `ferr != nil`, reached deep inside the per-seed loop | `multisigVerifyNoExpectationBody` (same message as 717) |

Reading `verifyFreshSlots` (line 324-336) shows its only error path is
`len(expected) == 0 → errVerifyNoExpectedSlots` — i.e. site 854 is a
*defensive re-check of the same condition as site 717* (expectedSlots never
changes inside the function), not a third distinct trigger. So F-199's "two
programmer-error refusals" correctly names the two distinct *conditions*;
it does not claim an exact site count, and none of its language is
contradicted by there being 4 return sites rather than 3.

**7. "neither engrave caller re-offers on `verifyRefused`".**
**ACCURATE.** The two production call sites (found via the `multisigVerifyFn`
test-seam indirection, `gui/multisig_verify.go:666`) are
`gui/multisig_build.go:460` and `gui/multisig.go:345`. Both loops read:
```go
res := multisigVerifyFn(...)
if res != verifyIncomplete && res != verifyFailed {
    break
}
```
`verifyRefused` is neither, so both break out and fall through directly to
restore-doc construction — no re-offer.

**8. B3's `correctable` local, scoped to the seed-entry and ms1-entry
breaks.**
**ACCURATE.** `correctable := false` declared at line 827 (top of the
per-seed loop in `multisigVerifyFlow`), set `true` at line 919 (the
seed-entry coverage-exhausted/no-match arms, "AND ALL THREE ARMS PRESCRIBE A
REMEDY") and at line 946 (`correctable = correctable || rejected`, the ms1
entry rejection arm), and read at line 997 to decide `verifyIncomplete` vs
`verifyAbandoned` when `len(legs) == 0`. It is local to this function and is
not consulted by, or connected to, the `verifyRefused` sites — confirming
F-199's point that the existing mechanism does not already cover the site it
is filing.

---

## F-204 — FAILED single-sig verify says doubt the plates; multisig sibling says doubt the passphrase first

**1. `gui/singlesig_verify.go:145`, string *"Check the engraved plates"*.**
**DRIFTED-by-37** (pre-settled by controller, confirmed). Now at line 182:
```
182:		showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")
```
Context confirms this is genuinely the FAILED-verify path: it fires when
`verifySingleSig(...)` returns a non-nil error (comparator ran and
disagreed), title `"Verify Failed"`, `rec.adverse = true` set immediately
above — matches F-204's framing exactly.

**2. `multisigVerifyNoSlotBody` (`gui/multisig_verify.go:151-165`).**
Symbol: **ACCURATE** — function is defined at line 157:
`func multisigVerifyNoSlotBody(passphraseTyped, provedInnocent bool) string`.
Range: **ACCURATE but loosely drawn.** Lines 151-165 span the tail of the
doc comment (151-156), the function signature (157), the `switch` and first
case `provedInnocent` (158-162), and the opening two lines of the second
case `passphraseTyped` (163-165) — it does not include the `default:` case
or the closing brace, i.e. it is not a clean "start of function to end of
function" bound, but every line in the cited range is genuinely part of
this function/its comment, and the quoted string sits right at the range's
end.

**3. Quoted string *"Check the passphrase before you doubt the plates"*.**
**ACCURATE**, verbatim substring, split across lines 164-165:
```
164:		return "No slot matches that seed with the passphrase you typed. Check the " +
165:			"passphrase before you doubt the plates: one wrong character derives a " +
```
Concatenated: `"...Check the passphrase before you doubt the plates: one wrong character derives a different wallet."` — the cited phrase is an exact substring.

**4. "SPEC 7.4" claim that verify requires the seed RE-TYPED, "so the
engrave source is never compared against itself".**
**ACCURATE.** `design/SPEC_systemwide_payloads.md:1201-1208` (registry
repo), section `### 7.4 The rule that is not negotiable`:
> "The session cache must never answer a verification prompt on the
> operator's behalf. ... Otherwise verify compares the engrave source
> against itself and passes unconditionally — certifying a *wrong plate* as
> good, silently."

This is the same reasoning restated in the source code's own comments citing
§7.4, e.g. `gui/multisig_verify.go:732-736`: "§7.4's reasoning applied to
the bundle rather than to the seed — a readback taken from the session
would compare the engrave source against itself and pass unconditionally,
certifying a wrong plate." F-204's paraphrase is faithful to both the spec
text and the code's own citation of it.

---

## F-206 — the pass line's ms1 clause stays singular on a multi-seed multisig verify

**1. Fixed string `The ms1 secret you typed matched this seed.`**
**ACCURATE.** `gui/verify_status.go:155`:
```
155:	verifyStatusMS1Clause = "The ms1 secret you typed matched this seed."
```
Byte-exact.

**2. Claim that §4.7c clause **B** holds this string.**
**ACCURATE.** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:950`
(`#### 4.7c THE FOUR LINES`), clause table at line 973:
```
| **B** | `The ms1 secret you typed matched this seed.` | iff `rec.pass.full` |
```
Byte-exact match, and confirmed in the current code path:
`buildVerifyPassLine` (`gui/verify_status.go:211-231`) appends
`verifyStatusMS1Clause` whenever `p.full` is true — **unconditionally on
`p.legs`**, i.e. regardless of how many seeds/legs were verified. This is
the exact mechanism F-206 describes: a full multisig verify with `legs == 2`
still emits the singular clause B verbatim, with no pluralization or leg
count.

**3. Claim that the device's own screen says *"the ms1 you typed for each
seed"*.**
**ACCURATE.** `gui/multisig_verify.go:1134-1135`, inside
`multisigVerifyOKMessage` (the multi-leg, full-mode arm):
```
1134:			return fmt.Sprintf("All %d operator key plates verified, and the ms1 you typed "+
1135:				"for each seed. Other cosigners' keys are taken as supplied.", legs)
```
Concatenated, this contains the exact substring "the ms1 you typed for each
seed" — confirming the asymmetry F-206 names: the on-device multi-leg screen
already pluralizes correctly (via `%d ... legs`), while the restore
document's clause B does not.

**4. `passRecord.legs` symbol — does it exist, with that exact name?**
**ACCURATE.** `gui/verify_status.go:60-80`, struct `passRecord`:
```
60:	type passRecord struct {
...
65:		// legs is how many operator key plates were read back and compared.
66:		legs int
```
Field name is exactly `legs`, type `int`. No sibling/prefixed variant found
(`grep -n "PassRecord\b"` and bare `.legs\b` across `gui/*.go` turned up
only this struct's own field, used at `gui/verify_status.go:215,220`).

---

## Verdict per slug

| slug | verdict | notes |
| --- | --- | --- |
| F-199 | **has 1 structural error** | citation 4 (`gui/multisig_verify.go:82` at commit `b2c3231`) is wrong — correct line is 64, that revision's line 82 is blank. Citation 1 has the pre-settled drift (now line 752). All other claims (verifyRefused new in 9f93362, two programmer-error refusals, no re-offer by either caller, `correctable` local, restore-doc string) verified ACCURATE. |
| F-204 | **has drift only** | citation 1 has the pre-settled drift (now line 182). All other citations (symbol, range, quoted string, SPEC §7.4 claim) verified ACCURATE, with one loosely-drawn but not incorrect line range (151-165 covers a comment tail + partial function rather than clean function bounds). |
| F-206 | **clean** | all four citations (fixed string, §4.7c clause B, device screen string, `passRecord.legs`) verified byte-exact ACCURATE against current source. |
