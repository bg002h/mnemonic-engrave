# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 4 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` (639 lines, as
folded at `438992f`).
**Round 3:** `design/agent-reports/R0-S2-plan-r3.md` (RED, 1C/1I/4M/3N), persisted
at `60e7868`, folded at `438992f`. The reviewed text is exactly
`git diff 60e7868..438992f -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`
(115 insertions, 34 deletions).
**Trees:** mnemonic-engrave `438992f` (the plan is byte-identical at `HEAD`
`bbf1a71`, a continuity commit — verified `git diff 438992f..HEAD -- <plan>` is
empty); seedhammer fork `main` @ `a5e29b44637d0657ab8f1ec603f1a375b0cc54cb`.
**Nothing in either repo was modified** — the fork edits below were made to a
`tar`-copy in the scratchpad, and the `me` binary was run, not rebuilt.

**THE ONE QUESTION:** does the r3 fold resolve each of r3's 9 findings, and did
the fold's edits introduce a new defect?

**Counts: 0 Critical / 2 Important / 2 Minor / 0 Nit — verdict RED.**

**All nine of r3's findings are answered in the text, and r3's Critical is
executed as ruled: every one of the five coordinated edits option 1 demanded is
present.** Both Importants are incompletenesses *in that ruling's own
propagation*, not disputes with it: the fold decided to widen the device inside
S2, and then (I1) left the half of the widening no gate can see without a test,
and (I2) declared its falsification inventory COMPLETE while the sentences
P3.4 actually falsifies — six sites, measured — are not on it. The one sentence
the inventory *does* name for P3.4, `SPEC_descriptor_input.md:462`, is the one
clause in that paragraph P3.4 leaves TRUE.

---

## Method

- The `me` binary at `target/debug/me` (crates unchanged since `f244442`;
  `git log -1 -- crates/` confirms) run on constructed inputs.
- A Go probe (go 1.26.3 from the nix store) calling `nonstandard.OutputDescriptor`,
  built TWICE — once with `replace seedhammer.com => /scratch/code/shibboleth/seedhammer`
  (`a5e29b4`, untouched) and once against a `tar`-copy carrying **only** P3.4's
  arm. The patch was re-verified byte-for-byte with `diff -u` against the live
  repo: `+ case ypubVer:` / `+ script = P2SH_P2WPKH`, two lines, nothing else.
- The vector file read as data (`python3 -c 'json.load(...)'`), never by eye.
- `grep -n` / `git grep` for every count, line number and phrase claimed.

### Verified by measurement — the fold's new claims that are SOUND

| fold claim | verdict |
| --- | --- |
| **`ParseExtendedKey` normalises the version away (`bip380/bip380.go:456-462`)** | **CONFIRMED, exact.** `456: // Now we have a derivation path, normalize the version bytes to xpub.` … `462: }`. The classification switch P3.4 patches is `:444-455`. Both cites in the fold are right to the line. |
| **`bip380.Key` has no version field, so the check cannot be a conjunct over the parsed value** | **CONFIRMED.** `bip380/bip380.go:28-36`: `Network`, `MasterFingerprint`, `DerivationPath`, `Children`, `KeyData`, `ChainCode`, `ParentFingerprint`. No version, and `ParseExtendedKey` returns a re-netted `*hdkeychain.ExtendedKey`. The fold's reason for making it string-level is the true reason. |
| **The two inert `DESCRIPTOR_PATH_SHIPPED` hits are `main.rs:360` and `mod.rs:59`** | **CONFIRMED, read against the claim.** `main.rs:360` is `use mnemonic_engrave::descriptor::{DESCRIPTOR_PATH_SHIPPED, MD1_PATH_SHIPPED};` and `:365` is the `if DESCRIPTOR_PATH_SHIPPED` it feeds — exactly as the fold describes. `descriptor/mod.rs:59` is the `pub use gate::{…, DESCRIPTOR_PATH_SHIPPED, MD1_PATH_SHIPPED}` re-export. |
| **The new sweep terms reach the sentences r3 I1 named** | **CONFIRMED, measured.** `grep -n 'record classification fails' design/SPEC_descriptor_input.md` → `857:` and `1566:` — exactly the two sentences and nothing else. `gate_open` → 1 hit. `sysw_class` → `:1491`, `:1528`, `:1919`. `PANICS the Go parser` → `:382`. Every term the fold lists resolves. |
| **§7 requirement 3 is `:1496-1498`** | **CONFIRMED.** `1496: 3. **The Rust test asserts the host column; the Go test asserts the device` / `1497: column.** Neither implementation is ever compared to the other — both are` / `1498: compared to the file.` The fold's characterisation (the second half survives) is right. |
| **The ypub flip is ONE column, not two** | **CONFIRMED — this is the load-bearing one and it holds.** `format` is a HOST-side column (`scripts/descriptor-seam-vectors/rows.py:4`: *"Host-side columns (host_admits, md1_admits, format, the gate fields, covers)"*), so the row keeps `format: "none"` after P3.4 even though the device now parses it. Precedent exists in the file: `promotion/15-bare-tpub-host-refused` is already `format: none, device_admits: true`. No hidden second column moves. |
| **P3.4's "seam-SAFE direction" argument** | **CONFIRMED against the file's own text.** The JSON's `invariant` field is verbatim `"host_admits(input) => device_admits(canonical(input))"`, so widening the device cannot falsify it. |
| **F-426's entry states device-first sequencing as its own design** | **CONFIRMED, verbatim at `design/FOLLOWUPS.md:14719-14721`:** *"One case-arm makes the device symmetric (`ypub` → `P2SH_P2WPKH`, normalise to `xpub`), after which the host's five-version admission widens to match in its own convergence cycle."* P5.1's split (device half resolved-in-build, host half open as its own convergence cycle) is what the entry already says; it needs no task the plan does not schedule. |
| **r3 M3's provenance rule is executable, and does not collide with "ONE implementer per phase" or the P2 gate** | **CONFIRMED, and better than the fold claims.** `gen.py`'s own header: *"EVERY device-side and value column is MEASURED here — the Go probe … point `goprobe/go.mod`'s `replace` at the fork worktree this corpus is pinned to"*, and `README.md:27` documents the knob (`$EDITOR goprobe/go.mod`). So pointing it at `s2/descriptor-arm` measures **both** booleans mechanically rather than by hand. No conflict with "ONE implementer per phase" (disjoint repos, disjoint worktrees — the parallel-isolation rule is satisfied), and none with the P2 gate's *"the port must not begin from unreviewed semantics"*: the two early-authored fixes are F-426's arm and the `parse.go` convergence fix, **neither of which is the classifier port**. |
| **Bare `ypub` diverges after P3.4 (r3 C1(c))** | **RE-MEASURED INDEPENDENTLY.** Patched: `OK title="" script=Nested Segwit (P2SH-P2WPKH) type=0 thr=1 keys=1 / key0 path=m/49h/0h/0h children=[]`. Unpatched: `ERR nonstandard: unrecognized output descriptor format`. Host, same string: `me sysw pack --as descriptor` → **rc=3**, *"the device admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is `ypub` …"*. |
| **No third site still describes the arm as a §4.7 port only (r3 M2)** | **CONFIRMED.** `grep -n '§4.7'` returns 9 hits (`:31`, `:231`, `:293`, `:370`, `:375`, `:415`, `:421`, `:435`, `:472`); each read. `:31` and `:435` are the two the fold repaired; `:231`/`:370` are §5.2's predicate sentence quoted as CORRECT; the rest are about admission or the `Encode` argument. Nothing stale. |

---

## IMPORTANT

### I1 — the bare-key half of C1's remedy is specified and ungated: every gate in the plan stays green on a classifier that admits a bare `ypub` the host refuses

The fold executes r3 C1's option 1 faithfully, including the clause that
answers C1(c):

> AND §4.3's five-version admitted set — `xpub`/`tpub`/`zpub`/`Ypub`/`Zpub` —
> as a STRING-LEVEL check, r3 C1 … so without this check the classifier
> answers `ClassDescriptor` on a record the host refuses — **on both the
> descriptor-embedded and bare-key paths**

The instruction is right. What C1(c) was *about*, though, was visibility — r3's
own words: *"P3.3's gate — which is the plan's only instrument here — cannot see
it."* After the fold, P3.3 still cannot see it, and nothing else was added that
can.

**Constructed counterexample — an implementation that satisfies every stated
gate and ships the divergence.** Port §4.5's promotion table as the *device's*
version→script mapping plus the three-type promotion loop (which is exactly how
`SPEC_descriptor_input.md:570-574` writes it), and apply the new §4.3 string
check where the fold motivates it — over the descriptor's key positions, the
place `bip380.Key`'s missing version field made unavailable. Then:

- `sysw.Classify("ypub6WyzNbqt7S3quv…")` → the record has no descriptor key
  positions, the promotion mapping now contains `ypubVer → P2SH_P2WPKH`
  (P3.4 put it there), `P2SH_P2WPKH` is one of §4.5's three promotable types
  (`SPEC_descriptor_input.md:560`), so the arm answers **`ClassDescriptor`**;
- `me` on the identical string answers **rc=3** (measured above);
- and every gate passes: **P3.3** asserts over the file's rows and there is no
  bare-`ypub` row (measured — the only two `ypub` inputs in all 71 rows are
  `promotion/04-bare-Ypub-refused`, a *capital* `Ypub`, and
  `neither/full-origin-ypub`, the descriptor-embedded one); **P3.4's** own tests
  are `bip380`-level (*"bare `ypub` classifies and normalises to `xpub`"* — they
  assert the new acceptance, not a `sysw` refusal); **P3.1's** named unit tests
  are the two panic inputs; **P1.2's** negative sweep runs over the pre-S2
  record corpus; and the **P3 review brief** now names *"the §4.3 string-level
  version check"*, which a reviewer would find present, because it is.

The divergence that ships is the exact class the seam file exists to measure —
device classifier wider than host — on the funds path, in the phase whose gate
the plan calls *"the gate that caught this row"*.

The plan is also sending the implementer to the falsified paragraph: P3.1 says
port *"at minimum §4.5's promotion table: which bare-key versions promote"*, and
§4.5's promotion prose (`:570-574`) is prose about the **device's** fallback that
P3.4 makes false (see I2). A port taken from that paragraph post-P3.4 admits
bare `ypub`; a port taken from the host's `cascade.rs` refuses it. Nothing in
the plan distinguishes the two readings, and no test does either.

**Fix, and it is one line, not a redesign:** add to P3.1's named unit tests
(or P3.4's test-per-direction) a `sysw`-level negative — `sysw.Classify(<the
bare `ypub` string, e.g. rows.py:29's SKYL>) == ClassUnknown`, with the comment
naming P3.4 as the reason it can fail. That costs no vector-file byte and makes
the bare-key half of the string check falsifiable. (A 73rd row would work too,
but it re-opens invariant 1's payload and the tag manifest for no extra
coverage.)

### I2 — P0.1's section is declared "the COMPLETE enumeration" and omits six measured sites that P3.4 falsifies; the one §4.3 line it names is the clause that stays TRUE

The fold's answer to r3 M4 makes a claim the artifact now has to carry:

> and this section is the COMPLETE enumeration — P2.7 and P3.5 defer to it, not
> the reverse (r3 M4)

Its P3.4 member is:

> §4.3's five-version NORMATIVE sentence (`design/SPEC_descriptor_input.md:462`)
> and the operator-facing "the device admits exactly" refusal text … — false of
> the scan door once P3.4 lands.

**`:462` is the wrong line, and it is wrong in the direction that matters.**
Measured, with line numbers:

```
462:  its `xpub` twin ACCEPT. **NORMATIVE: `me` admits exactly the same five.** A
```

The normative clause on `:462` is about **`me`**, whose admission P3.4 does not
touch — the plan says so itself twice (P3.4: *"the host's five-version admission
is UNCHANGED in S2"*; P2.7: *"`me`'s admission is unchanged at five"*). The
sentences P3.4 *does* falsify are the four device sentences above it:

```
453:- **Extended-key version bytes — the admitted set is exactly five:** `xpub`
456:  every key in every branch, and the classification switch has **no `ypub`
457:  case** — `ypub` (`049d7cb2`) is declared in the constants and named in the
458:  later normalisation switch, but classification hits `default` and errors
459:  (`bip380/bip380.go:428–466`, re-read at fold time). So `ypub`, `upub`,
460:  `vpub`, `Upub`, `Vpub` are refused by the device **even with a full explicit
461:  origin** — measured: `sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))` REFUSE,
463:  standard BIP-32 library accepts `ypub`, so a host built on one without this
464:  gate is WIDER than the device on the commonest non-`xpub` key there is
```

`:461`'s measured claim is the **exact input of the row invariant 1 flips**, and
`:463-464`'s rationale inverts (post-S2 the device is wider than `me`, not the
reverse). An implementer handed "amend `:462`" can leave all of that standing.

**Five more sites, none on the section, all measured:**

| site | text | why P3.4 falsifies it |
| --- | --- | --- |
| `SPEC_descriptor_input.md:570-574` (§4.5) | *"`xpub`/`tpub` → `P2PKH`, `zpub` → `P2WPKH`, `Ypub` → `P2SH_P2WSH`, `Zpub` → `P2WSH`. Note `ypub` is listed in the version constants but **has no case in the switch**, so it hits `default` and is refused."* | measured false: patched probe promotes a bare `ypub` to `sh(wpkh(…))`, `path=m/49h/0h/0h`. This is also the paragraph P3.1 sends the porter to — see I1. |
| `SPEC_descriptor_input.md:1610-1611` (§7) | *"**`wsh(multi(…))`, a miniscript descriptor, and a full-origin `ypub`** — `false`/`false` on the host/device axes, the `neither` rows the vacuity check needs."* | after the flip the ypub row is `false`/**`true`**, so one of the three named rows is no longer a `neither` row. |
| `crates/me-cli/testdata/descriptor_seam_vectors.json` — the row itself | `covers: ["neither"]`, `name: "neither/full-origin-ypub"`, and `source: "SPEC_descriptor_input.md S4.3: ypub has no case in the classification switch -- refused even with a full explicit origin"` | the regeneration writes `device_admits: true` **next to** an annotation saying the device refuses it. Invariant 1's enumerated payload carries `source` fixes only for F-428. |
| `scripts/descriptor-seam-vectors/rows.py:327` and `comment.json:107` | the generator for that `source` string, and the shipped `_comment` manifest line `"  neither              3   wsh(multi), miniscript, full-origin ypub"` | same fact, in the two generator artifacts invariant 1 already regenerates "together". |
| `crates/me-cli/src/descriptor/cascade.rs:58-62` | *"The first five are the ones `ParseExtendedKey`'s classification switch admits (`bip380/bip380.go:428–466`) and **therefore** the ONLY five `me` admits (§4.3, NORMATIVE) … `ypub` is declared in the device's constants and has no case in the switch, so it is refused there even with a full explicit origin."* | host **source code** asserting the device's behaviour; false in two ways after P3.4 (six, not five; and the `therefore` linking `me`'s five to the device's is severed). Comments outlive their conditions. |

**Why the sweep does not rescue it.** The P2 gate's sweep is scoped *"per P0's
inventory"*, and P2.7's own term list is `sysw_class`, `panic:parse`, "PANICS the
Go parser", `gate_open`, "record classification fails". Measured: not one of the
five appears at `:453-465`, `:570-574`, `:1610`, `:1432`, in `cascade.rs`, or in
the vector artifacts. The term that reaches all of them is `ypub` — 10 hits in
the spec, 10 in FOLLOWUPS, 5 in the vector JSON, plus `cascade.rs`, `admit.rs`,
`refusal.rs`, `descriptor_refusals.rs`, `rows.py`, `comment.json` — and it is not
on the list. This is r2 I1's failure mode reproduced one round later for the C1
members.

**One coupled decision falls out of it, and it is unowned.** If the row keeps
`covers: ["neither"]`, the file ships a tag whose §7 definition it contradicts;
if it is retagged or renamed, `MANIFEST`'s `("neither", 3)`
(`crates/me-cli/tests/descriptor_seam.rs:57`), `TAG_SLOTS`, `ROW_FLOOR` and §7's
floor table (`SPEC_descriptor_input.md:1732`) all move. Neither branch is
written down. Note the plan must choose deliberately: the tag counts do **not**
red on their own, because a row keeping a now-false tag still counts.

**Fix:** (a) re-cite the §4.3 member as `:453-461` (the device clauses) and say
explicitly that `:462`'s NORMATIVE `me` sentence is CORRECT and untouched — the
same shape the fold already used for §5.2's predicate sentence; (b) add §4.5's
`:570-574` and §7's `:1610-1611` to the section; (c) add the row's
`covers`/`name`/`source`, `rows.py:327` and `comment.json:107` to **invariant
1's** regeneration payload, and `cascade.rs:58-62` to P2.7; (d) put `ypub` on
the sweep-term list; (e) rule the `neither`-tag question.

---

## MINOR

**M1 — "EVERY count guard that moves is named" is false on the engrave half: the
whole population table is unnamed and outside the cited range.** The fold's
engrave-side list is *"`MANIFEST`, `TAG_SLOTS`, `ROW_FLOOR`, `SECOND_TAGGED`,
`THIRD_TAGGED` (`crates/me-cli/tests/descriptor_seam.rs:50-69`)"*. Measured, the
Rust half also pins a `Pop` literal at **`:130-147`** — the mirror of the Go
half's `wantRows`/`wantDeviceTrue`/… block, which the fold *does* enumerate — and
six of its fields move under invariant 1's own payload:

```
131:    rows: 71,                    → 72   (the new witness row)
134:    device_admits_true: 37,      → moves (ypub flip + the measured panic:parse row)
135:    device_admits_false: 33,     → moves
136:    device_admits_absent: 1,     → 0    (panic:parse probe retires)
142:    sysw_class: 4,               → 0    (column retires)
143:    device_probe: 3,             → 2    (two panic:encode remain)
```

`gen.py` does not emit these (its only output line is
`"wrote %s: %d bytes, %d rows, sha256 %s"`), so they are hand-updated. Minor
rather than Important because each is asserted (`:490-504`) and a stale value
reds `descriptor_seam` loudly at the P2 gate — the failure is noise, not silence.
The paired presence assertion at `:265-275` (*"device_admits presence must track
the panic:parse marker"*) also goes vacuous, harmlessly. Add `POP`
(`descriptor_seam.rs:130-147`) to the engrave-half list.

**M2 — P2.7, a P2 task, owns the amendment that describes P3.4's device
widening, which lands in P3.** P2.7's C1 member is *"reworded to the two-door
truth: … the scan door accepts `ypub` after P3.4"*, and the plan already has
P3.5 for device-half spec amendments. As written, the P2 gate closes with the
spec (and the operator-facing `refusal.rs` string) asserting a device behaviour
the fork's `main` does not yet have. Small — it is one cycle, and the plan
already accepts a transient two-copy window for the vector file with the
reasoning stated — but it is free to fix: move that member's owning task from
P2.7 to P3.5, or state the transient explicitly the way invariant 1's sequencing
paragraph does.

---

## Fold vs r3's findings

| r3 | resolved? | evidence |
| --- | --- | --- |
| **C1** — P3.4 widens the device inside S2: flips a measured `device_admits` (a second byte change), reds P3.3's never-relaxed rule, escapes to a bare key, falsifies §4.3 + `refusal.rs:583` | **RULED AND EXECUTED — all five coordinated edits present; two consequences under-propagated → this round's I1 and I2** | Option 1 taken end to end: invariant 1 carries the flip *"measured, the ONLY row that moves"* with the measured-provenance rule extended to both booleans; `wantDeviceTrue` (`:67`) joins the guard list; P3.1 carries the string-level §4.3 check naming both paths; P0.1/P2.7 carry §4.3 + the refusal text + its pinned row; P5.1 splits F-426. Independently re-measured: patched fork moves exactly the one row; `format` does not move (host-side column, `rows.py:4`); the seam invariant is verbatim as quoted; F-426's entry says device-first in as many words. **What is missing is not in the ruling but under it:** the bare-key half has no instrument (**I1**), and the falsification set named is smaller than the set P3.4 falsifies (**I2**). |
| **I1** — §5.1:857-859 and §7:1563-1566's "after record classification fails" trigger is falsified by P1.0 and owned by nobody | **RESOLVED** | Both sentences are on P0.1's section AND P2.7's list, with `gate_open` and "record classification fails" as terms. Measured: the phrase resolves to `:857` and `:1566` and nowhere else; `gate_open` to 1 hit; §7 requirement 3's `:1496-1498` verified word for word, and the fold's *"the never-compare-implementations half survives"* is accurate. |
| **M1** — the "exactly seven hits" count was transcribed and is false (9) | **RESOLVED** | Now *"exactly NINE hits (recomputed this round, r3 M1: r2's 'seven' was transcribed, not measured)"*, with both extra sites named. `main.rs:360` and `descriptor/mod.rs:59` read against the claim — the `use` does feed `:365`'s conditional, and `:59` is the re-export line. |
| **M2** — two places still describe the arm as a §4.7 port only | **RESOLVED** | Line 31 now *"a real port of §4's cascade narrowings AND §4.7's conjuncts"*; the P3 gate brief now *"predicate parity including the CASCADE-NARROWING port — the §4.5 promotion table and the §4.3 string-level version check, the fragile half per r3 C1"*. All 9 `§4.7` mentions in the plan re-read; no third stale site. |
| **M3** — the measured-provenance rule is unowned and unscheduled | **RESOLVED, and mechanically supported** | Owner and worktree named in invariant 1 *and* scheduled in P2.6's own bullet. Verified no collision with "ONE implementer per phase" (disjoint repos/worktrees) or with the P2 gate's unreviewed-semantics rule (the early work is F-426's arm + the `parse.go` fix, not the port). Better than claimed: `gen.py` measures every device column through `goprobe/go.mod`'s `replace`, and `README.md:27` documents pointing it at a worktree, so "measured, never predicted" is a knob rather than a discipline. |
| **M4** — P0.1's section and P2.7's list disagree while P0.1 claims completeness | **RESOLVED in mechanism, falsified in fact → this round's I2** | P0.1 is now declared authoritative and P2.7 defers to it; the two lists agree member for member. The word "COMPLETE" is what fails: six measured sites P3.4 falsifies are on neither, and the §4.3 member cites the one line that stays true. |
| **N1** — the oracle entry is `:66-69`, not `:64-68` | **RESOLVED** | *"(`gui/sysw_admit_oracle_test.go:66-69` — r3 corrected r2's `:64-68`)"*. |
| **N2** — `wantDeviceAbsent` moves and is unnamed | **RESOLVED** | *"the never-read `wantDeviceAbsent` (`nonstandard/descriptor_seam_test.go:69`) retires in the same commit (r3 N2: declared, zero use sites, and its panic:parse population goes to 0)"*. Confirmed at `:69` as `wantDeviceAbsent = 1 // the panic:parse row`. |
| **N3** — `SECOND_TAGGED`/`THIRD_TAGGED` move only if the new row is multiply tagged | **RESOLVED** | Now *"which move with a 72nd row and its `covers` tags (`SECOND_TAGGED`/`THIRD_TAGGED` only if the new row is multiply tagged — r3 N3)"*. |

---

## What a fold has to decide, not just fix

Nothing this round is a ruling — both Importants have one obvious shape, and
neither reopens C1's decision.

**I1 is one test.** A `sysw`-level assertion that a bare `ypub` classifies
`ClassUnknown`, in P3.1's or P3.4's named tests. It needs no vector row and no
byte change, and it converts the plan's own sentence *"on both the
descriptor-embedded and bare-key paths"* from an instruction into something that
can fail.

**I2 is one editing pass** over P0.1's section, invariant 1's payload and the
sweep-term list, plus a two-way ruling on the `neither` tag (keep the tag and
amend §7's definition, or retag and move `MANIFEST`/`ROW_FLOOR`/§7's floor table
— either is fine, but the P2.6 commit cannot discover the question).

Both Minors are additive: name `POP` (`descriptor_seam.rs:130-147`) in the
engrave-half guard list, and move the "scan door accepts `ypub`" amendment from
P2.7 to P3.5.
