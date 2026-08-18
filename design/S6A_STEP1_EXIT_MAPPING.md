# S6a STEP 1 — the two gate artifacts

**Status: for review. No code written, nothing in the fork modified.**

Step 1 of §4.8 is a gate, not a task. This document is its whole output:

- **(a)** the single-sig eleven-exit → `verifyRecord` mapping
- **(b)** the `suppliedCosigners` expression

Nothing in `/scratch/code/shibboleth/seedhammer` was created, edited, branched or
committed to produce it. Every fact below was read out of the tree at
`main` = `b8a23bf3dcf45f0b996bedf8b17f7141f092d282`, verified with a command, and
the command's real output is pasted.

---

## 0. Provenance and re-verification of the "already measured" facts

The brief supplied four measured facts and instructed me to confirm rather than
repeat them. All four confirm.

    $ cd /scratch/code/shibboleth/seedhammer && git rev-parse HEAD
    b8a23bf3dcf45f0b996bedf8b17f7141f092d282
    $ git status --porcelain | wc -l
    0
    $ git rev-parse --abbrev-ref HEAD
    main

**F1 — the function's bounds.**

    $ grep -n "^func singleSigVerifyFlow" gui/singlesig_verify.go
    65:func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool) {
    $ sed -n '149p' gui/singlesig_verify.go
    }

**F2 — ten explicit returns, and their lines.**

    $ awk 'NR>=65 && NR<=149 && /return/{print NR": "$0}' gui/singlesig_verify.go
    69: 		return
    78: 		return
    90: 		return
    98: 			return
    112: 		return
    117: 		return
    125: 			return
    130: 			return
    138: 			return
    146: 		return
    $ awk 'NR>=65 && NR<=149 && /return/{c++} END{print c}' gui/singlesig_verify.go
    10

Ten explicit returns at **69, 78, 90, 98, 112, 117, 125, 130, 138, 146**, plus the
implicit fall-through at **149**, reached after the verify-OK notice at `:148`.
**Eleven exits.** Confirmed.

**F3 — one call site, one-shot, no retry loop.**

> **SUPERSEDED 2026-08-18 by S6b P9 — all three clauses.** The production call
> site now loops (`gui/singlesig.go:217` is `for {`) and dispatches through a
> test seam (`gui/singlesig.go:223`, `singleSigVerifyFn`, declared at
> `gui/singlesig_verify.go:263`), so it is neither one-shot nor the only name
> the flow is reached by. The signature in the grep transcript below is also
> stale: it is now
> `func singleSigVerifyFlow(ctx *Context, th *Colors, full, template, engravedWithPassphrase bool, rec *verifyRecord) bool`
> (`gui/singlesig_verify.go:96`) — it takes a record and returns "can the
> operator still act on this", which is what drives the retry.
>
> **This site was MISSED by the sweep that found the other four** and caught by
> `scripts/fold-propagation-check.sh` on the phrasing `one-shot`. It is the
> reason that script exists: the fold corrected the claim everywhere the
> reviewer pointed and left the doc's own finding heading, three hundred lines
> above, still asserting it.

    $ grep -rn 'singleSigVerifyFlow(' --include='*.go' .
    gui/singlesig_verify.go:65:func singleSigVerifyFlow(ctx *Context, th *Colors, full, template bool) {
    gui/singlesig.go:132:		singleSigVerifyFlow(ctx, th, full, template)

`gui/singlesig.go:130-133` verbatim:

    130		verifyChoice := &ChoiceScreen{Title: "Verify Bundle", Lead: "Verify the engraved plates?", Choices: []string{"Verify now", "Skip"}}
    131		if sel, ok := verifyChoice.Choose(ctx, th); ok && sel == 0 {
    132			singleSigVerifyFlow(ctx, th, full, template)
    133		}

No `for`, no re-offer. One call per engrave. No stub, no function-value
indirection, no test callers. Confirmed.

**F4 — the binding precedent's message is byte-identical.**

    $ python3 -c "
    a=open('gui/singlesig_verify.go').read().split(chr(10))[88]
    b=open('gui/multisig_verify.go').read().split(chr(10))[895]
    print(repr(a.strip())); print(repr(b.strip())); print('IDENTICAL:', a.strip()==b.strip())"
    'showError(ctx, th, "Verify Bundle", "Couldn\'t re-derive the bundle from the seed.")'
    'showError(ctx, th, "Verify Bundle", "Couldn\'t re-derive the bundle from the seed.")'
    IDENTICAL: True

The whole call at `gui/singlesig_verify.go:89` is byte-identical to
`gui/multisig_verify.go:896`, not merely the message string. §4.7b classifies the
multisig site's return (`gui/multisig_verify.go:897`) **benign**, so
`gui/singlesig_verify.go:90` is benign. Confirmed and binding.

---

## 1. THE RULE I APPLIED — stated first, so the table can be checked against it

§4.7b's criterion is *"world-set contains a bad-plate world"* vs *"nothing
observed about the plates"*. That is the right criterion but it is not directly
decidable at a site: almost every world-set contains *some* bad-plate world if
you allow enough conjunction. So I made it decidable by reading what the adverse
cell actually **prints** (§4.7c, `statusCheckDidNotPass`, verbatim):

> `A verification check ran and did not pass: a comparison did not match, or a plate could not be read or accounted for. Do NOT rely on this backup until a full check passes.`

That line **asserts that a check ran**. So:

> **ADVERSE iff the device READ, compared, or accounted for what came back off
> the plates, AND that step produced a negative result.**
> **BENIGN otherwise** — including every path where the device withdrew,
> refused, or never obtained a comparable object at all.

**The three verbs are deliberate, and an earlier draft of this rule had only two
(gate review N-1).** "Comparison or accounting" is *narrower* than the printed
line it is derived from, which says **"read, compared or accounted for"** — and
the missing verb is load-bearing: `gui/multisig_verify.go:724` is a **decode**
failure of a plate that was read, classified **adverse** by §4.7b. A two-verb
rule would have argued it benign and contradicted the table it claims to
reproduce. No single-sig verdict below changes under either wording — single-sig
has no decode-failure exit of that shape — but the rule is the reusable
instrument here, so it matches its source exactly.

This is §4.7b's criterion restated so a site can be decided by inspection, and it
is what makes the classification fail safe: if no check ran, saying "a check ran
and did not pass" is a false statement about the device's own behaviour, which
§4.8 names explicitly as a G2 violation. It also reproduces §4.7b's multisig
table row for row — I checked the classified multisig sites against it and found
no disagreement, including the two that look adverse and are not (`:897` derive
failure, `:979` partial-verify-everything-matched).

**Two clauses of the printed line map onto the two adverse single-sig sites**, which
is the design telling you what it expected to find there:

| clause of the `statusCheckDidNotPass` line | single-sig site |
| --- | --- |
| *"a comparison did not match"* | `gui/singlesig_verify.go:146` — `verifySingleSig` returned an error |
| *"or a plate could not be read or accounted for"* | `gui/singlesig_verify.go:117` — `singleSigReadbackCards` could not account for one mk1 + one md1 |

---

## 2. ARTIFACT (a) — the eleven-exit → `verifyRecord` mapping

**Eleven rows for eleven exits. No row names a `verifyStatus` value** — the status
is derived once, downstream, by §4.7a's switch over the two recorded booleans.

| # | exit (`path:line`) | verbatim source | what the device had observed at this point | bit | writes |
| --- | --- | --- | --- | --- | --- |
| 1 | `gui/singlesig_verify.go:69` | `68 if !ok {` / `69 return` | Nothing. `seedEntryFlowTypedOnly` at `:67` returned `!ok` — the operator pressed Back at the seed keyboard or the machine is unwinding. No plate has been touched; the NFC gather is 43 lines away at `:110`. | **benign** | **NEITHER** |
| 2 | `gui/singlesig_verify.go:78` | `77 if !ok {` / `78 return` | Nothing about the plates. `singleSigPickFlow` at `:76` returned `!ok` — the operator abandoned the purpose/script picker. Still before any readback. | **benign** | **NEITHER** |
| 3 | `gui/singlesig_verify.go:90` | `89 showError(ctx, th, "Verify Bundle", "Couldn't re-derive the bundle from the seed.")` / `90 return` | Nothing about the plates. `deriveSingleSigBundle` at `:87` failed on the **re-typed seed**; this is a fact about the typed words, not about steel. | **benign** | **NEITHER** |
| 4 | `gui/singlesig_verify.go:98` | `97 showError(ctx, th, "Verify Bundle", "Couldn't re-build the template bundle.")` / `98 return` | Nothing about the plates. `templateizeBundle(reDerived)` at `:95` failed — it operates purely on the freshly **re-derived** bundle (`md.StripToTemplate` → `md.FormAwareStubChunks` → `reStubMk1`). No plate has been read. | **benign** | **NEITHER** |
| 5 | `gui/singlesig_verify.go:112` | `111 if !ok {` / `112 return` | Nothing the device drew a conclusion from. `bundleGatherFlow` at `:110` returns `(nil, false)` at exactly two places — `gui/bundle_flow.go:177` (Back) and `gui/bundle_flow.go:241` (`ctx.Done`). Its `bundleDoneEmpty` arm shows a screen and **loops**; it does not return. So `!ok` means the operator withdrew. | **benign** | **NEITHER** |
| 6 | `gui/singlesig_verify.go:117` | `116 showError(ctx, th, "Verify Bundle", "Need one key card (mk1) and one descriptor (md1) read back.")` / `117 return` | **A card set was read back off the plates and could not be accounted for.** `bundleGatherFlow` returned `ok`, so at least one complete card was gathered and the operator pressed Done; `singleSigReadbackCards` at `:114` then failed for one of three reasons — two mk1s (`:28`), two md1s (`:33`), or a missing kind (`:38-40`). | **ADVERSE** | **`adverseRecorded = true`** |
| 7 | `gui/singlesig_verify.go:125` | `124 if !ok {` / `125 return` | Nothing. `inputCodex32Flow` at `:123` returns `!ok` only via Back (`gui/gui.go:1033-1035`) or `ctx.Done` — it returns `(obj, true)` **only** when `valid` (`gui/gui.go:1039-1041`). The operator declined to type an ms1. | **benign** | **NEITHER** |
| 8 | `gui/singlesig_verify.go:130` | `129 showError(ctx, th, "Verify Bundle", "That isn't an ms1 secret share.")` / `130 return` | Nothing about the plates. The type assertion at `:127` failed, so `inputCodex32Flow` returned a **valid** non-`codex32.String` object — per `validateMStar` (`gui/codex32_polish.go:263`) the only other valid returns are `mdmkText(frag)` for HRP `md`/`mk`. The operator typed a descriptor or key string where an ms1 was asked. No comparison ran. | **benign** | **NEITHER** |
| 9 | `gui/singlesig_verify.go:138` | `137 showError(ctx, th, "Verify Bundle", "That isn't a valid ms1 secret share.")` / `138 return` | Nothing about the plates. `codex32.DecodeMS1` at `:135` rejected a **checksum-valid** codex32 `ms` string (`codex32.New` already succeeded inside `validateMStar`) — `errMSBadPrefix`, `errMSBadLength` or `errMSBadLanguage` (`codex32/mspayload.go:34-59`), i.e. a K-of-N share or a wrong-length payload rather than the unshared secret. No comparison ran. | **benign** | **NEITHER** |
| 10 | `gui/singlesig_verify.go:146` | `145 showError(ctx, th, "Verify Failed", "The read-back bundle does NOT match the seed. Check the engraved plates.")` / `146 return` | **The comparator ran and disagreed.** `verifySingleSig(reDerived, ms1Readback, mk1, md1)` at `:144` returned `bundle.Verify`'s first diverging-field error against the read-back mk1 + md1 (plus the typed ms1 when `full`). | **ADVERSE** | **`adverseRecorded = true`** |
| 11 | `gui/singlesig_verify.go:149` — **the fall-through, the only success exit** | `148 showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` / `149 }` | **The comparator ran and matched.** `verifySingleSig` returned `nil` at `:144`. `full` is in scope (parameter, `:65`). | **benign** (not adverse) | **`fullPassRecorded` — write the pass record here** |

### The success write, in full

It goes **before the closing brace at `:149`**, after the notice at `:148`.
`full` and `template` are parameters and are in scope; nothing needs threading.

    // gui/singlesig_verify.go, immediately after the showNotice at :148
    rec.pass = &passRecord{
        full:              full,  // the MODE, captured where it is in scope
        legs:              1,     // one key plate (mk1) was compared
        suppliedCosigners: 0,     // single-sig has no policy cosigners
    }

Three notes a reviewer should check rather than take:

- **`legs: 1` is correct on a template engrave too.** `templateizeBundle` strips
  the **md1** to a template and *re-stubs* the mk1 — `reStubMk1(b.MK1, stub)` — it
  does not remove the key. The key plate compared is a real key plate in both
  forms, so `template` does not change this count.

      func templateizeBundle(b bundle.Bundle) (bundle.Bundle, error) {
          tmplMD1, err := md.StripToTemplate(b.MD1)
          ...
          mk1, err := reStubMk1(b.MK1, stub)
          ...
          return bundle.Bundle{MS1: b.MS1, MK1: mk1, MD1: tmplMD1}, nil
      }

- **`full` is recorded, not inferred.** It is the flow's own parameter. On a
  watch-only run `ms1Readback` stays `""` (`:121`) and `verifySingleSig` drops the
  derived ms1 (`:51-55`), so no ms1 comparison happened and the record must not
  let a line claim one. That is R9's C-1 and it is why the record carries the mode
  rather than a status.
- **`rec.adverse` is untouched here.** The success exit sets no adverse bit. The
  "benign" bit for row 11 means only *not adverse*; the pass write is what
  distinguishes it from the ten.

### The count that falls out

| bit | exits | lines |
| --- | --- | --- |
| adverse | **2** | 117, 146 |
| benign, writes neither (zero cell) | **8** | 69, 78, 90, 98, 112, 125, 130, 138 |
| pass (writes `fullPassRecorded`) | **1** | 149 |
| **total** | **11** | |

### The §4.8 consequence check — `statusVerifiedOnRetry` must be unreachable

§4.8: *"A proposed mapping that reaches `statusVerifiedOnRetry` from within the
eleven exits is wrong, and that is checkable without judgement."*

`statusVerifiedOnRetry` requires `adverse && pass` in one call. Both adverse
sites are **terminal `return` statements** — `:117` and `:146` — with no loop
around them (F3: no retry loop; the single call site is a one-shot `if`). So no
control path writes `adverseRecorded` and then reaches `:149`.
**`statusVerifiedOnRetry` is unreachable. PASS.**

> **SUPERSEDED 2026-08-18 by S6b P9 — this PASS was true when written and is
> false now.** S6b's failure-states review found that both adverse arms
> dead-ended the operator, and the F2 fix replaced the one-shot `if` with a
> `for` loop (`gui/singlesig.go:217`, fork commit `511f7f3`). The premise this
> PASS rests on — "no loop around them" — is exactly what that fix removed, so
> **`statusVerifiedOnRetry` is REACHABLE from single-sig today**, deliberately
> and by design. It is reached and asserted by
> `TestSingleSigVerifyRetryProducesAnHonestStatusVerifiedOnRetryLine`
> (`gui/s6b_p9_failure_states_test.go`), which drives a real fail-then-pass
> sequence and renders the line. The check itself was sound; only its input
> changed. **Left standing rather than rewritten** — it is the record of what
> S6a's gate actually verified, and editing it would make that record lie
> about a different thing.

The other three states are reachable, as §4.8 requires:

| state | reached by |
| --- | --- |
| `statusVerified` | exit 11 (`:149`) |
| `statusCheckDidNotPass` | exits 6 and 10 (`:117`, `:146`) |
| `statusNotFullyChecked` (zero cell) | exits 1-5 and 7-9 **from inside the flow**, and additionally by never entering it (`gui/singlesig.go:131` Skip) |

The zero cell being reachable *from inside* the flow is exactly R16 I-1's
correction, and this mapping delivers it eight times over.

---

## 3. ARTIFACT (b) — the `suppliedCosigners` expression

### The names actually in scope at `gui/multisig_verify.go:987`

Read out of the file, not assumed. `:987` is `return verifyComplete`, the last
statement of `multisigVerifyFlow` (`:662`-`:988`), at function-body scope — so
every binding below is live there.

    $ awk 'NR==662 || NR==721 || NR==750 || NR==768 || NR==987 {printf "%d\t%s\n", NR, $0}' gui/multisig_verify.go
    662	func multisigVerifyFlow(ctx *Context, th *Colors, full bool, expectedSlots []int, engravedMd1 []string) multisigVerifyResult {
    721		_, keys, err := md.ExpandWalletPolicyChunks(readbackMd1)
    750		var legs []verifyLeg
    768		covered := make(map[int]bool, len(keys))
    987		return verifyComplete

| name | type | declared | notes |
| --- | --- | --- | --- |
| `keys` | `[]md.ExpandedKey` | `:721` | **all policy keys**, from the read-back md1 (`md/expand.go:102`). Assigned once; no shadowing rebind inside the function. |
| `covered` | `map[int]bool` | `:768` | written at exactly one place — see below; never deleted, never set `false`. |
| `expectedSlots` | `[]int` | `:662`, parameter | the obligation list this run engraved |
| `legs` | `[]verifyLeg` | `:750` | one per verified slot, appended at `:899` beside the `covered` write |
| `full`, `readbackMd1`, `readbackMk1s`, `typed`, `correctable`, `ctx`, `th` | — | — | also live, not needed |

    $ grep -n "covered\[" gui/multisig_verify.go
    324:		if covered[s] || !slices.Contains(expected, s) {
    900:			covered[s] = true
    971:			if !covered[s] {

`:324` is a read inside `verifyFreshSlots` (on its own parameter), `:971` is a
read. **`:900` is the only write in the whole file**, and it is
`covered[s] = true`.

**Slot numbers are indices into `keys`.** `allUserSlots`
(`gui/multisig_match.go:78-97`) builds its result as `matches = append(matches, i)`
over `for i, k := range keys`, and `:894` does `keys[s].OriginPath`. So the domain
of `covered` is `[0, len(keys))`.

### The expression

**Recommended:**

    // countUncoveredPolicyKeys counts the policy keys this run did NOT verify a
    // leg for -- the cosigners whose keys were taken as supplied rather than
    // checked. It iterates the KEYS and asks whether each is covered, never the
    // other way round: a stray or out-of-range entry in `covered` can then only
    // make this number LARGER, never smaller (G2's direction of failure).
    func countUncoveredPolicyKeys(keys []md.ExpandedKey, covered map[int]bool) int {
        n := 0
        for i := range keys {
            if !covered[i] {
                n++
            }
        }
        return n
    }

used at `gui/multisig_verify.go:987` as:

    rec.pass = &passRecord{
        full:              full,
        legs:              len(legs),
        suppliedCosigners: countUncoveredPolicyKeys(keys, covered),
    }
    showNotice(ctx, th, multisigVerifyOKTitle, multisigVerifyOKMessage(len(legs), full))
    return verifyComplete

**Against the three acceptance criteria:**

1. **Computable from names in scope at `:987`** — `keys` (`:721`) and `covered`
   (`:768`) are both function-body bindings, both live at `:987`, both shown above.
   Nothing new is threaded in and no signature changes.
2. **0 on every single-sig path** — see below; single-sig writes the literal `0`.
3. **Counts policy keys NOT covered by a verified leg, and cannot under-report** —
   it is a count *over `keys`* of the uncovered, which is the criterion's own
   words. The failure direction is the point: the only way `covered` can lie is by
   holding an entry it should not, and this form **ignores** entries outside
   `[0, len(keys))` and would count a wrongly-cleared entry as uncovered. Every
   defect in `covered` therefore inflates the count. Over-reporting supplied
   cosigners renders a clause that says *less was checked*; under-reporting would
   hide an unchecked key. This form cannot do the latter.

**The one-liner I rejected, and why.**

    suppliedCosigners: len(keys) - len(covered)   // REJECTED

It is arithmetically identical **at `:987` today** — at that point
`len(legs) == len(expectedSlots)`, every leg's slot was written into `covered` in
the same loop iteration (`:899-:900`), and `verifyFreshSlots`
(`gui/multisig_verify.go:318-324`) refuses to re-emit a slot that is already
covered or not in `expected`, so `covered` holds exactly the verified slots with
no duplicates. But its correctness rests on a **map-cardinality invariant held
elsewhere**, and it fails in the wrong direction: any future write that puts an
extra key in `covered` silently *shrinks* the reported count.
`len(keys) - len(expectedSlots)` is rejected for the same reason plus one more —
`expectedSlots` is the caller's obligation list, not evidence of anything verified.

### What single-sig writes, and why it is 0

**Literal `0`**, in the `passRecord` at `gui/singlesig_verify.go:149`.

    $ grep -n -i "cosign\|policy\|keys\b\|covered" gui/singlesig_verify.go
    (no output)

Zero hits. `singleSigVerifyFlow` has no wallet policy, no key list and no
`covered` map, because there is nothing to have one of:
`deriveSingleSigBundle(m, passphrase, net, path, script)`
(`gui/singlesig_derive.go:37`) derives from **one** seed at **one** path, and the
readback requires exactly one mk1 and one md1 (`:114-118`). There is no key in the
descriptor that this run did not itself derive and compare, so the number of keys
taken as supplied is 0 by construction, not by omission.

Under G2 this cannot under-report: there is no cosigner key on a single-sig path
for the count to miss. And it is the value that makes the clause *absent* — §4.7c
renders `Other cosigners' keys are taken as supplied.` **iff
`rec.pass.suppliedCosigners > 0`** — which is what §4.7b-seam requires, since
including that clause on a single-sig document would misdescribe the wallet (G1)
and `TestMultisigVerifyNoticeIsHonest` (`gui/multisig_verify_test.go:171`) pins
the split.

---

## 4. WHAT I WAS UNSURE ABOUT

Four. Three I resolved with an argument I am willing to defend; the fourth is a
question for the reviewer, not a classification.

### U1 — `:117`, the readback-composition refusal. The hardest row.

**For benign:** an operator who presents only the mk1 plate because the md1 is in
another drawer, then taps Done, has shown the device nothing wrong with the steel.
The world is entirely innocent and the document would nevertheless print *"Do NOT
rely on this backup until a full check passes."* That is scary language for a
paperwork slip, and §0.1's own `statusNotFullyChecked` reasoning (R7: *"cries wolf
on a backup that is probably fine"*) is a real cost.

**For adverse, which is what I chose:**

1. **The device did observe the plates.** `bundleGatherFlow` returned `ok`, which
   at `gui/bundle_flow.go:207/210` means complete cards were gathered off the
   reader. `singleSigReadbackCards` then ran an accounting over them and it
   failed. The brief's tiebreaker — *did the device observe anything about the
   plates?* — answers **yes**, so the tiebreaker does not apply.
2. **The adverse line is literally true on the innocent world.** *"or a plate
   could not be read or accounted for"* is an exact description of the
   forgot-the-md1 case. Adverse here is not an over-claim; it is the clause the
   line was written for.
3. **§4.7b binds it.** The multisig twin is `gui/multisig_verify.go:701`
   (`extractReadbackMd1AndMk1s` fails → `verifyRefused`), listed in §4.7b's
   **adverse** column as *"readback filter drops cards"*. Same position in the
   flow, same kind of check, same shape of failure.

   > **STALE 2026-08-18 (Minor):** that site returns **`verifyIncomplete`**, not
   > `verifyRefused`, since F-199 (S6b P1, fork commit `c95dd23`) — and it moved
   > to `gui/multisig_verify.go:797`. The verdict change is the point of F-199:
   > `verifyIncomplete` is what both multisig engrave callers re-offer on, where
   > `verifyRefused` was terminal. **The conclusion this bullet supports is
   > unaffected** — the twin is still adverse, still the same position in the
   > flow — so only the quoted verdict and line number are wrong. Current
   > behaviour is documented correctly in `SPEC_s6b_pre_flash_cycle.md` §3.1.
4. **The world-set genuinely contains bad-plate worlds**: an md1 whose NFC data is
   damaged never completes a chunk set and never appears in `cards`; two mk1s means
   a plate from another run is in the pile.

The cost of being wrong is asymmetric in adverse's favour here *because the line
is true either way* — which is not so at `:130`/`:138` below, and that is exactly
what separates them.

### U2 — `:130` and `:138`, the rejected ms1 typing. The one I most expect to be argued.

**For adverse:** the operator is transcribing from the engraved ms1 plate. If the
plate were engraved with the wrong content — say the md1 payload landed on the
seed plate — the operator would faithfully type a valid `md` string and land on
`:130`. That is a bad-plate world, so the world-set contains one.

**For benign, which is what I chose:**

1. **No check ran, so the adverse line would be false.** `verifySingleSig` is
   never reached; nothing was compared against anything. Printing *"A verification
   check ran and did not pass"* would claim a check the device did not perform —
   which §4.8 names in so many words as the G2 violation this classification
   exists to avoid. Unlike U1, there is no clause of the adverse line that is true
   here.
2. **A garbled plate almost cannot reach these lines.** `inputCodex32Flow` returns
   `(obj, true)` only when `valid` (`gui/gui.go:1039`), and `validateMStar`'s `ms`
   arm requires `codex32.New(frag)` to succeed — i.e. the bech32 checksum must
   pass. A mis-engraved character fails the checksum, so the operator never gets
   past the keyboard and exits at `:125` (benign) instead. The residual bad-plate
   world requires a *whole wrong payload* engraved with its own valid checksum,
   which is a bug in the engrave path, not a mis-cut plate.

   **This argument omitted `inputCodex32Flow`'s error-correction arm (gate review
   N-2), and completing it makes the benign verdict stronger, not weaker.**
   `gui/gui.go:1042-1049` offers `codex32.Correct(frag)` on an invalid-but-in-window
   fragment, accepting a fix within 4 changes and re-validating. So a mis-engraved
   character is not always a dead end at the keyboard. But a *corrected* string is
   by construction a valid codex32 string, so it returns `(obj, true)` and flows
   **past** `:130`/`:138` to the comparison at `:146` — which is classified
   **adverse**, correctly. The correction arm therefore routes garbled-plate
   worlds to the site that *does* assert a check ran, and leaves `:130`/`:138`
   holding only the paths where nothing was ever compared.
3. **§4.7b's multisig treatment matches.** `multisigVerifyMS1Entry`
   (`gui/multisig_verify.go:1004-1022`) shows the byte-identical two screens and
   returns `rejected=true`; the caller `break`s (`:887`), which lands on `:938`
   `verifyIncomplete` — §4.7b's **benign** column, *"zero legs, correctable"*.
   The fork already calls this class *correctable operator input*.
4. **The tiebreaker applies and points at benign.** Genuine uncertainty, nothing
   observed about the plates, so the zero cell — which says less.

### U3 — `:112`, a Back *after* cards were scanned.

`bundleGatherFlow` can have accepted several cards into `scr.g.cards` before the
operator presses Back at `gui/bundle_flow.go:176-178`. So it is not strictly true
that the device observed nothing — it observed cards. I still classified it
benign, because **it drew no conclusion**: the accounting at `:114` never ran, and
the multisig twin `gui/multisig_verify.go:696` is §4.7b's benign *"abandons"* row
with the in-code rationale *"Back at the gather is the operator declining to
present plates at all... nothing was compared."* Under the adverse-requires-a-
check rule this is clean; I record it because "nothing observed" is a slightly
loose description of the path and a reviewer reading that phrase literally may
stumble.

### U4 — NOT a classification: can a **multisig** run record `suppliedCosigners == 0`?

**ANSWERED, AND NO DECISION IS OWED BEFORE STEP 2 (gate review N-3).** The
observation below is correct, but the plan already ruled on this exact case and
this section did not cite it: **T27's row in §5 carries a NON-VACUITY clause**
naming the self-multisig fixture, `open == 0` at `gui/multisig_build.go:96`, and
the resulting `suppliedCosigners == 0` — and it requires a fixture with an
uncovered key to be named or built at step 7, precisely so T27 cannot pass while
asserting nothing. That was filed as R15's Part-3 caveat and folded at `4c40973`;
this section reached the same fact independently from the other end. **Nothing
breaks, and going further would be NG1.** The original analysis follows.

Yes, I believe so, and the reviewer should decide whether that matters before
step 2. If a build holds device-side seeds for *every* policy slot
(`buildSlotSources` → `buildEngraveTail`, `gui/multisig_build.go:240,384`), then
`expectedSlots` can be all of `keys`, all legs verify, and `covered` covers every
key — so the expression yields 0 and the cosigner clause does not render on a
**multisig** document.

I think this is correct and not a defect: on that run *nothing was taken as
supplied*, so omitting the clause is the truthful line, and §4.7b-seam's stated
purpose — *"omitting it is an unscoped claim on the multisig document"* — does not
apply when there is no unverified cosigner to scope. It does mean the field is a
**coverage** axis rather than the "path axis" §4.7b-seam names it, and the two
coincide everywhere except this case.

One consequence worth noting and, I think, leaving alone: on that same run the
on-screen notice still says *"Other cosigners' keys are taken as supplied"*
(`gui/multisig_verify.go:1053/1059/1062`) while the document would not. The screen
is the one under-claiming there, which is safe under G2 and is pre-existing
behaviour outside this cycle's scope. Flagging, not proposing.

---

## 5. PLAN-vs-CODE DISAGREEMENTS FOUND

**None that affects either artifact.** §4.7b's classifications, §4.7b-seam's exit
census, §4.8's call-site and retry-loop claims, and the byte-identity precedent all
verified. Two smaller notes:

- **A stale doc comment in the fork, not a plan error.** `gui/bundle_flow.go:125-126`
  says `bundleGatherFlow` returns *"(nil,false) on Back / **an empty bundle**"*.
  The second half is false in the current code: the `bundleDoneEmpty` arm
  (`:181-189`) shows a screen and continues the loop; the only `return nil, false`
  sites are `:177` (Back) and `:241` (`ctx.Done`). **This mapping is built on the
  code, not on that comment** — and had I taken the comment at face value, row 5
  would have needed an argument about whether "an empty bundle" is an observation
  about the plates. It does not, because it cannot happen.
- **§4.7b classifies `gui/multisig_verify.go:897` benign although the code there
  returns `verifyFailed`.** That is not a contradiction — §4.7a says the verdict
  *"is not read here at all"* — but it is the single most confusing thing in the
  table for a reader, and the same split appears in this mapping: `:90` is benign
  and `:146` is adverse even though both sit under a `showError`. **A screen being
  shown is not the bit.** Recorded so the next reader does not re-derive it.

---

## 6. SELF-CHECK

Every acceptance criterion from §4.8, with the command and its real output.
`$D` is this document, `$S` is
`/scratch/code/shibboleth/seedhammer/gui/singlesig_verify.go`.

**C1 — the row count equals the exit count MEASURED IN THE CODE, not quoted.**

    $ awk 'NR>=65 && NR<=149 && /return/{c++} END{print c+1}' $S
    11
    $ grep -c '^| [0-9]* | `gui/singlesig_verify.go:' $D
    11

Equal. **PASS.**

**C2 — every exit appears exactly once, including the fall-through at `:149`.**

    $ grep -oE '^\| [0-9]+ \| `gui/singlesig_verify\.go:[0-9]+' $D | grep -oE '[0-9]+$' | sort -n | uniq -c
          1 69
          1 78
          1 90
          1 98
          1 112
          1 117
          1 125
          1 130
          1 138
          1 146
          1 149

Eleven distinct lines, each with count 1, and the set is identical to F2's ten
returns plus `149`. **PASS.**

**C3 — no row names a `verifyStatus` value.**

    $ awk '/^\| [0-9]+ \| `gui\/singlesig_verify\.go:/' $D \
        | grep -c 'statusVerified\|statusCheckDidNotPass\|statusNotFullyChecked\|statusVerifiedOnRetry'
    0

Zero. The status names appear in this document only in §1, §2's reachability
table and §4 — prose *about* the downstream switch — never in a mapping row.
**PASS.**

**C4 — no exit writes a pass record on a path the device did not observe passing (G2).**

    $ awk '/^\| [0-9]+ \| `gui\/singlesig_verify\.go:/' $D | grep -n 'fullPassRecorded'
    11:| 11 | `gui/singlesig_verify.go:149` — **the fall-through, the only success exit** | ... | **`fullPassRecorded` — write the pass record here** |

Exactly one row names `fullPassRecorded`, and it is row 11, `:149` — the exit
reached only when `verifySingleSig` returned `nil` at `:144`. No other row writes
a pass record. **PASS.**

**C5 — `statusVerifiedOnRetry` unreachable: adverse sites are terminal.**

    $ awk '/^\| [0-9]+ \| `gui\/singlesig_verify\.go:/' $D | grep -c 'adverseRecorded'
    2

Two adverse rows (`:117`, `:146`), both `return` statements per F2, with no loop
in the flow and a one-shot call site per F3. No path writes adverse then falls
through to `:149`. **PASS.**

> **SUPERSEDED 2026-08-18 by S6b P9**, same as the §4.8 consequence check above
> and for the same reason: the "one-shot call site" clause is what S6b's F2 fix
> removed. `statusVerifiedOnRetry` is reachable and tested today. The `awk`
> count of 2 adverse rows still holds; it is the *no loop* half of the argument
> that expired.

**C6 — `suppliedCosigners`: names in scope, and the single write to `covered`.**

Both outputs are pasted in §3 above: `keys` at `:721` and `covered` at `:768` are
function-body declarations of `multisigVerifyFlow` (`:662`), and `covered[s] = true`
at `:900` is the only write in the file. **PASS.**

**C7 — nothing in the fork was modified.**

    $ cd /scratch/code/shibboleth/seedhammer && git status --porcelain | wc -l
    0
    $ git rev-parse HEAD
    b8a23bf3dcf45f0b996bedf8b17f7141f092d282

Clean at the expected SHA, on `main`, no branch created, no commit made. **PASS.**
