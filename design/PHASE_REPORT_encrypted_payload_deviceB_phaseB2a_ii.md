# Phase report — Plan B Phase B2a-ii (unlock and the secret session)

**Written 2026-08-09, after the fact**, to close the second half of **F-96**.
The phase itself merged at `a01b666` on 2026-08-08.

**Why it is late, stated plainly.** F-96 asked for two things: commit the
mutation runner, and land it *with the phase report if that is still owed*. The
runner was never committed, and this report was never written — so a 30-mutant
run existed only as prose in one commit message. The results were not lost; the
**method was**. See "The runner is gone" below, which is the part of this that
cannot be recovered.

---

## What the phase built

Unlock and the secret session for encrypted payload delivery: the checksum gate,
the chunked KDF with its progress screen, the retry loop, §10.2.2's secret
session (every secret offered first, each wiped as its plate leaves the screen),
and the post-session plate list.

**Commit range:** `421dca8..a01b666` in `bg002h/seedhammer`. Tasks 4–8. Task 8
changed no code — its deliverable was the mutation record.

**It was not operator-complete at merge**, and the merge commit says so:
§10.2.4's residency-keyed idle wipe was left to B2b, and a release tag was
explicitly forbidden.

---

## Review record — 11 lens reports, all persisted verbatim

All in `design/agent-reports/encrypted-payload-planB-phaseB2a-ii-*.md`.

| lens | verdict |
| --- | --- |
| lens1 — wipe lifecycle | **1C** / 1I / 1M |
| lens1 — C1 fold re-review | FIXED |
| lens1 — I1/M1 fold re-review | FIXED |
| lens1 — pass 3 | 0C / 0I / 3M / 1N |
| lens2 — false-PASS hunt | 0C, 10 findings (I/M/N) |
| lens3 — crypto | 0C / 1I / 2M / 2N |
| lens4 — flows | 0C / 2I / 2M / 2N |
| lens5 — conformance | 0C / 1I / 3M / 2N |
| lens6 — hostile input | 0C / 0I / 1M / 2N |
| lens7 — concurrency | **1C** / 0I / 2M / 1N |
| lens8 — completeness critic | 3 files with **zero** mentions across all prior lenses; 3 more effectively unexamined |

**Two Criticals**, both folded before merge: lens1's wipe-lifecycle finding and
lens7's frame-scheduling finding (`051d423` — the KDF progress loop must ask for
its frame *before* submitting one).

**lens8 earned its place.** Asking "what did nobody look at" found three test
files with zero mentions across ten prior reports, two of which held real
findings. That pattern repeated in B2b, where the equivalent sweep found a
green-criterion row that was red at baseline.

---

## §11.3 mutation rows this phase owns — reproduced from `3db3bfe`

Every row names the test that kills it. **A mutant with no named killer is a gap
in the suite, not a passing result.**

| §11.3 mutant | result | killed by |
| --- | --- | --- |
| BIP-39 checksum check removed | killed | `TestUnlockChecksumGateRunsNoKDF` — "the checksum-invalid passphrase ran the KDF 1 times; §11.2 requires the rejection happen WITHOUT invoking it" |
| KDF run before the checksum gate | killed | the same **counter**, not the return value — both orders return the identical `errUnlockChecksum` |
| iteration count read as a constant | killed | `TestDeriveKeyMatchesTheVectors/B` (the 100,001-iteration vector) |
| tag verification unconditional-pass | killed | `TestOpenFailsOnAFlippedCiphertextByte` |
| public section left out of the AAD | killed | `TestOpenDrivesEveryVector/D` — D is the vector with a public section |
| only the first secret record offered | killed | the vector-F offer-order test |
| `ms1` not wiped after its plate | killed | the resident-during-engrave buffer assertion |
| wipe omitted on the Back exit path | killed | the Back test **and** `defer p.Wipe()` |
| passphrase prompted when `ct_len == 0` | killed | `TestUnlockNeverPromptsWhenNothingIsEncrypted`, which instruments the hook — a return-value assertion passes over exactly this defect |
| `ms1` accepted in the public section | killed | `TestPublicSectionRefusesASecret` + `TestGroupingRunsAfterTheAllowList` |
| idle timer runs during engraving | **DEFERRED → B2b** | no timer existed in B2a. Recorded with its owning phase rather than claiming coverage. |

**Whole-phase total: 30 mutants run across Tasks 5–8. 29 KILLED, 1 SURVIVING.**

### The survivor, recorded rather than papered over

`clear(blob)` / `blob = nil` removed. **The plan predicted it**: *"not
test-observable — record it as a surviving mutant rather than inventing a test
that appears to cover it."* It is F-79's release of the 65,536-byte payload
region before the engrave; nothing the suite can reach observes heap residency.

### Three rows that first reported the wrong verdict

Worth preserving, because **a false survivor misleads exactly as much as a false
kill** — and two of the three were the *runner's* fault, not the suite's:

- **6.1 and 6.5** reported SETUP-FAIL because the runner rejected any mutant
  whose replacement *contains* the original text. An inserted line or a widened
  condition is a perfectly ordinary mutant. Fixed by comparing against the exact
  expected text.
- **6.7's first form** resliced the record instead of skipping the wipe, so the
  deferred wipe zeroed the new copy anyway — it SURVIVED while testing nothing.
- **5.7** was a genuine suite gap: `SecretsResident` ignoring `IsSecret` survived
  a **one-directional** test. Fixed by asserting the discriminating direction —
  wipe the secrets and leave twelve non-zero cards resident. The standing "mutate
  in BOTH directions" rule landing on its first real case.

---

## The runner is gone, and that is the irrecoverable part

`3db3bfe` says: *"It is a single self-contained Python file and is reproduced in
the phase report."* **This report is that report, and the file is not in it** —
it was written inline, used, and discarded with the tool call that ran it. The
results above survived because they were written into a commit message; the
procedure did not, because nothing wrote it anywhere.

**That is F-96's whole point, and the fix is structural rather than a promise:**
B2b Task 7 commits `scripts/mutation-run.py`, which **derives its rows from the
plan's own mutation tables** instead of a transcribed list, so the check is a
command rather than a discipline. It supersedes the lost script. The rows above
are not re-runnable as they were; the B2b rows are.

**The lesson, for the record:** we saved the conclusions and threw away the
method. A result is not reproducible — only a procedure is. "30 mutants, all
killed" in a commit message reads like a record and cannot be re-run or audited
except by trusting whoever wrote it.

---

## Green at merge, measured

```
CGO_ENABLED=0 go test ./...   exit 1, exactly TWO [setup failed]
                              (cmd/kdfbench, cmd/sealread) — unchanged from
                              the 421dca8 baseline
                              ok seedhammer.com/gui   16.174s
                              ok seedhammer.com/seal  12.631s
go vet ./seal/                clean
go vet ./gui/                 only the pre-existing go1.26 ArtifactDir line
gofmt -l <all 16 changed>     empty (tested as out=$(...); [ -z "$out" ])
tinygo pico-plus2             1307232 flash / 60584 ram
                              baseline 421dca8: 1285664 / 60544
                              delta: +21568 flash (+1.68%), +40 ram (+0.07%)
```

---

## What was left open at merge

- **Task 9 — the hardware pass**, operator-run, closing §7.1's in-situ KDF
  measurement on RP2350B silicon, which no host run substitutes for. **Still
  outstanding**; it is on B2b's release-tag checklist.
- **§10.2.4's residency-keyed idle wipe** — the whole of B2b.
- The follow-ups this phase filed: F-87, F-88, F-89, F-90, F-91, F-92, F-93,
  F-94, F-95, F-96, F-97, F-98. Their current owners are in `FOLLOWUPS.md`;
  three (F-88, F-90 items 1/3, F-94) were re-assigned to **B2c** on 2026-08-09.
