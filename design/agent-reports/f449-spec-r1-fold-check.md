# F-449 spec r1 — fold verification (mechanical)

**Verdict: 1 finding unaddressed (opus N1, Nit, non-blocking) / 2 new defects found.**

Reviewer: sonnet, mechanical fold-check only. No design merits reviewed. Baseline
confirmed live, both clean trees, both exactly matching the spec's stated
`**Baseline:**` line:

```
$ git -C descriptor-mnemonic log -1 --format=%H   → 6cbd49d80a657e4068e1c509f815330437752a6b  (spec cites 6cbd49d8)
$ git -C seedhammer log -1 --format=%H            → 7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4   (spec cites 7b6f2fb)
```

---

## Checklist

### opus (2C/6I/5M/2N)

| id | verdict | note |
| --- | --- | --- |
| C1 | ADDRESSED | §3c changes the version to 8, replaces the reasoning with a full even/odd dispatch table; §3d/§6 updated to `{4,8}` throughout |
| C2 | ADDRESSED | §3e adds `Descriptor::wire_version()` derived from the tree, names all three `write_node` callers, explicitly rejects the "pass `WF_REDESIGN_VERSION`" reading with the exact collision reasoning |
| I1 | ADDRESSED | §3a states the `{4,8,12}` budget up front and reframes the cost as "one of the two remaining generations" |
| I2 | ADDRESSED | §2 step 1 rewritten to the TLV `[32..65]` byte range; step 5 rewritten to cite the render-time `--network` selector, not wire content |
| I3 | ADDRESSED | §4 splits row 2 (keyless template, now correctly identified as `descriptor_to_template`, a documented input) from row 3 (abstract template, safe); §4a makes the input side mandatory |
| I4 | ADDRESSED | §8 opening paragraph concedes the parse-vs-import gap explicitly; item 8 (live `harnesses/liana` install run) is promoted from Bonus to Required |
| I5 | ADDRESSED | §9 rewritten as a stage table; explicit paragraph states 1a/1b do not stand alone, cites the exact `=0.45.1` pin and says the pin bump + call-site repairs ship in the same commit |
| I6 | ADDRESSED | §7 states the class-2 exclusion and the "still NOT counted as unlocked" ruling as two separate decisions, with the exact constructed failing shape from the original finding |
| M1 | ADDRESSED | §6 row 4 resolves the ambiguity to the non-vacuous reading (`tr` nested under `sh`/`wsh`) |
| M2 | ADDRESSED | §3d bullet + §3e's "read-side / write-side" list cover `chunk.rs:279` and `encode.rs:174` |
| M3 | ADDRESSED (twice) | §7 renames the fourth `KeyPathKind` value to `NumsXpub`; §4b (new) explicitly retires both `policy_shape.rs:119-133`'s doc comment and `DESIGN_coordinator_compatibility.md:155-158` |
| M4 | ADDRESSED | §8 item 2 requires checksum-inclusive byte equality explicitly |
| M5 | ADDRESSED | §9 closing paragraph puts `mnemonic-toolkit` in scope with a pin bump + golden refresh after stage 2 |
| N1 | **NOT ADDRESSED** | §6's last row still reads "version 8 with every `Body::Tr` at `kind = 0`" — unchanged phrasing from r0's "every `Body::Tr`" that N1 flagged as vacuously-quantified for non-`tr` descriptors. No rewording, no acknowledgment. Non-blocking (Nit). |
| N2 | ADDRESSED | §2's "property worth pinning" now includes `decaying-multisig` explicitly, states it's the nested case and "the stronger pin — opus N2"; §8 item 6 requires all three shapes |

### fable (0C/4I/9M)

| id | verdict | note |
| --- | --- | --- |
| I-1 | ADDRESSED | same fix as opus C1 (§3c) |
| I-2 | ADDRESSED | §8 item 3 adds the device leg explicitly (three-way address equality); the two concrete divergence vectors (a) `sortedmulti_a` and (b) non-`<0;1>` use-site are closed by refusal in §6 rather than by an active gate — the checklist's own suggested check ("does §6 refuse it?") resolves true for both |
| I-3 | ADDRESSED | §4a (new) makes `md encode`/`md decompose`/template grammar recognise and verify the recipe on input, closing the phantom-slot and conformance-gate problems; §4b reconciles the design doc |
| I-4 | ADDRESSED | §0 adds the device-reachability promise (a choice screen at stage 4) and explicitly rejects the "default to kind 1" alternative |
| M-1 | ADDRESSED | §4a: "The template grammar gains a substitution rule for `UNSPENDABLE(liana)`"; §4a also covers the JSON schema third-state need |
| M-2 | ADDRESSED | §2's gap list explicitly flags the nested-taptree order as unmeasured (Liana refused that shape) |
| M-3 | ADDRESSED | §7 new bullet on the two switch statements with no default arm |
| M-4 | ADDRESSED | §6's last row documents the hand-crafted second encoding explicitly, tagged `opus/fable M-4` |
| M-5 | ADDRESSED | §7 last bullet: F-633's copy fix must state the Nunchuk probabilistic-import fact |
| M-6 | ADDRESSED | §6 row 3: "must WARN, not silently ignore" |
| M-7 | ADDRESSED (moot) | Root cause (version 5's odd-bit collision) is fixed by C1/I-1's version-8 change; §3c's dispatch table now shows version 8 routes correctly with no `got: 2` confusion at either single-string or chunked. Not separately tagged in r1, but the scenario cannot occur post-fix, exactly as fable's own hedge anticipated ("moot if I-1 changes the version") |
| M-8 | ADDRESSED | §2's opening line states wire/slot order vs. the device's derived-key-sorted order explicitly, tagged `fable M-8` |
| M-9 | ADDRESSED | §2 "Derivation, not just rendering" paragraph pins `0/i`/`1/i` regardless of use-site, and §6 refuses non-`<0;1>` use-sites, tagged `fable M-9` |

### self (author-found)

| id | verdict | note |
| --- | --- | --- |
| SELF-1 | ADDRESSED (untagged) | §8 item 3 already documents the comparison against "the evidence's recorded Liana addresses" as one of three legs; the original "say so, it's stronger than claimed" framing is superseded by I4's rewrite of §8 (which now requires the live Liana run anyway, changing what "without building Liana" would even mean) |
| SELF-2 | ADDRESSED | §6's "INVARIANT, not a refusal" block, explicitly tagged, reproduces the `md encode` RUN evidence and the "no @i placeholder" reasoning verbatim |
| SELF-3 | ADDRESSED | §3f mandates the two-commit split, explicitly tagged; see New Defect 1 below re: one of its supporting numbers |
| SELF-4 | ADDRESSED | §4a's "Ruling" paragraph, tagged `opus I3, fable I-3, SELF-4`, requires md to verify the recipe on input rather than merely emit it, and to replace the internal error with a real refusal |

---

## New defects introduced by the fold

### New defect 1 — §3f's "58 NUMS references … 8+" blast-radius figure does not reconcile with any mechanical count

**Location:** §3f, "Measured blast radius: **88** `is_nums` occurrences across 14
files … **58** NUMS references in the fork's `md/` + `gui/`". This entire
paragraph is new prose in r1 (r0's §3f had no blast-radius numbers at all); it
was pulled from the self-report's SELF-3 table, which used the same "58 / 8+"
pairing without a stated methodology.

**The Rust half is exact.** Machine-checked against the live tree:
```
$ grep -rn "is_nums" crates/md-codec/src/ | wc -l      → 88
$ grep -rln "is_nums" crates/md-codec/src/ | wc -l     → 14
```
Confirms `88` / `14` precisely.

**The Go half does not reconcile under any tested scoping**, against the same
clean, baseline-matching tree (`seedhammer` @ `7b6f2fb`):

| scoping | occurrences | files |
| --- | --- | --- |
| `NUMS` substring, all files | 97 | 26 |
| `NUMS` substring, non-test `.go` only | 39 | 12 |
| `\bNUMS\b` whole word, all files | 69 | 22 |
| `\bNUMS\b` whole word, non-test `.go` only | 22 | 8 |
| `isNums` (the actual Go field name, direct analog of Rust's `is_nums`), all files | 41 | 20 |
| `isNums`, non-test `.go` only | 24 | **8** |

The file count "8+" is plausible under two of these scopings (both land on
exactly 8 non-test files, via different tokens). The occurrence count "58"
matches none of the six triangulations tried — the closest is 41 (all `isNums`
occurrences, tests included) and 69 (`\bNUMS\b` whole-word, tests included);
neither is 58. This is the same class of claim the fold corrected once already
(opus I5's "five md-cli sites" → verified four) but here the correction was not
made — the number went from report to spec unchecked.

**Severity:** cosmetic — it does not change any ruling, refusal, or sequencing
in the spec, only a rhetorical "how big is this refactor" figure supporting
SELF-3's already-accepted two-commit-split ruling. Recorded per the project's
"never hand-count what a tool can count" rule; does not block.

### New defect 2 — §8 item 8 (the live `harnesses/liana` install run, promoted from Bonus to REQUIRED) is never assigned to any stage's gate in §9

**Location:** §8 item 8: *"A live `harnesses/liana` install run at v15.0 —
**REQUIRED**, not a bonus."* (This promotion is itself the fold's correct
response to opus I4.) §9's sequencing table.

**Reproduction.** §9's table gate column, verbatim:
```
1a | ... | suite green at 1400+, no wire bytes changed
1b | ... §8 vectors 1-7, 9 | §8
2  | ... | §8.2
3  | ... | §8 vectors in Go
4  | ... | §8.3 device leg, §7's constructed shape
5  | ... | site 200 + emulator reaches the new screen
```
```
$ grep -n "harnesses/liana\|item 8" design/SPEC_liana_unspendable_internal_key.md
457:8. **A live `harnesses/liana` install run at v15.0 — REQUIRED, not a bonus.**
```
No other match. Stage 1b's gate explicitly enumerates "§8 vectors 1-7, 9" —
skipping 8 by name, not by accident of a range notation. No later stage's gate
mentions `harnesses/liana`, an install run, or item 8 either.

**Why it matters.** r0 could leave this unscheduled consistently, because r0
called it "Bonus, not required" (opus I4's own quote). The fold's response to
I4 changes the requirement's status but does not touch §9, so the spec now
states a required acceptance item with no stage whose gate depends on it — the
project's own standing rule is that "a plan may not close while any of its own
gates has never been run" is a Critical-class risk precisely because an
unscheduled gate is invisible to a reader checking stage-by-stage completion.
This is a propagation gap: I4's fold edited §8 but not the downstream §9 table
that lists what closes each stage.

**Severity:** worth folding before the next round — a required acceptance item
with no stage gate is exactly the shape this project's closure rules are
designed to catch. Does not, by itself, invalidate any existing ruling.

---

## What was NOT re-derived (per the brief)

The eight evidence-shape reproduction, the recipe correctness for the four
ACCEPT shapes, and the overall design decisions (version-8 choice, `InternalKey`
sum type, the two-commit split, the device choice-screen scope) were taken as
settled by the R0 reports and not re-reviewed here — this was a fold-vs-findings
and citation check only.
