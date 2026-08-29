# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 3 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` (558 lines, as
folded at `142258b`).
**Round 2:** `design/agent-reports/R0-S2-plan-r2.md` (RED, 1C/2I/4M/3N), persisted
at `59915b8`, folded at `142258b`. The reviewed text is exactly
`git diff 59915b8..142258b -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`
(99 insertions, 29 deletions).
**Trees:** mnemonic-engrave `142258b` (clean); seedhammer fork `main` @
`a5e29b44637d0657ab8f1ec603f1a375b0cc54cb`. **Nothing in either repo was
modified** — the one fork edit below was made to a `tar`-copy in the scratchpad.

**THE ONE QUESTION:** does the r2 fold resolve each of r2's 10 findings, and did
the fold's edits introduce a new defect?

**Counts: 1 Critical / 1 Important / 4 Minor / 3 Nit — verdict RED.**

**Nine of r2's ten findings are resolved, and the tenth (I1) is resolved in
mechanism but not in coverage.** The Critical is not a fold defect in the
narrow sense: it is what the fold's own new absolutes — *"P3.3's derived rule
stays EXACT … it is never relaxed to fit the arm"* and the newly enumerated
single regeneration — collide with, four bullets further down the same phase.

---

## Method

- `me` rebuilt at the reviewed tree (`cargo build --locked -p mnemonic-engrave`,
  `Finished`) and run on constructed inputs.
- A Go probe (`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`,
  go 1.26.3 from the nix store) calling `nonstandard.OutputDescriptor` over all
  71 vector rows, raw and `strings.TrimSpace`d, with `recover()`, reporting
  `Type`/`Threshold`/`len(Keys)`/`Script`/`Title`.
- **A second probe against a PATCHED COPY of the fork** — `tar`-copied to the
  scratchpad, one arm added to `bip380/bip380.go`'s classification switch
  (`case ypubVer: script = P2SH_P2WPKH`), which is P3.4 verbatim per F-426 — and
  the fork's own `go test ./bip380/ ./sysw/ ./nonstandard/` run against it.
- The vector file read as data (`python3 -c 'json.load(...)'`), never by eye.
- `grep -n` / `git grep` for every count and citation the fold added.

### Verified by measurement — the fold's new claims that are SOUND

| fold claim | verdict |
| --- | --- |
| **`nonstandard/parse.go:140` is the guard, `:149` the panic, `:158` F-428's key-count error** | **CONFIRMED at `a5e29b4`.** `140: if len(fp) > 4 {`; `149: MasterFingerprint: binary.BigEndian.Uint32(fp),`; `158: return nil, fmt.Errorf("bluewallet: expected %d keys, but got %d", …)`. The spec's own cite (`SPEC_descriptor_input.md:384`, "`nonstandard/parse.go:136–149`") agrees, and invariant 1 still points F-428 at `:158`. |
| **§5.2's predicate sentence is left intact; only the implementation sentence is amended** | **CONFIRMED.** `SPEC_descriptor_input.md:995` is the *"calls `nonstandard.OutputDescriptor`"* sentence; `:1001-1003` is the predicate blockquote, and `crates/me-cli/src/descriptor/admit.rs:408-412` carries it verbatim as `host_admits`' doc. The fold amends the first and names the second as CORRECT and untouched. |
| **`host_admits` is `cascade(normalise(input))` then the conjuncts** | **CONFIRMED**, `admit.rs:417-422`, exactly as cited (`:418-423`). |
| **The narrow direction of P3.3's derived rule is satisfiable on the Go side** | **CONFIRMED, measured per row.** All **15** single-line `host_admits: true` rows parse under `OutputDescriptor(TrimSpace(input))` — including `whitespace/leading-space-bip380`, whose RAW form errors and whose trimmed form returns `type=SortedMulti thr=2 keys=3 script=P2WSH`. |
| **The wide direction is 17 rows, 16 conjunct-catchable + the bare `tpub`** | **CONFIRMED, re-measured independently.** 17 single-line `host_admits: false` rows have `TRIM=ok`; 16 carry `format: bip380`, and `promotion/15-bare-tpub-host-refused` carries `format: none`. No 18th. |
| **`Tpub` is in `KeyVersion::admitted()`, so conjunct 4 passes on it** | **CONFIRMED.** `cascade.rs:94-100` — *"§4.3's five-member admitted set"* — `Xpub \| Tpub \| Zpub \| YpubCap \| ZpubCap`; `cascade.rs:529` is the `TestnetKey` doc comment the fold quotes verbatim. |
| **r2 M1's oracle claim (a `why`-string update, not a registration)** | **CONFIRMED.** `syswConsumers` is indexed `c.file+":"+c.fn` (`gui/sysw_admit_oracle_test.go:88`), and `{"wallet_policy.go", "walletPolicyFlow", …}` is already present with the `why` string the fold quotes — *"ClassMDMK only — this program never derives from a secret, so progWalletPolicy admits no seed class at all"*. (Cited `:64-68`; measured `:66-69` — N1.) |
| **r2 M2's guard names** | **CONFIRMED.** Go half: `wantRows` `:66`, `wantDeviceFalse` `:68`, `wantSyswClass/wantPanicParse/wantPanicEncode/wantHostWider` `:74-77`, and the `deviceTrue != wantDeviceTrue \|\| deviceFalse != wantDeviceFalse` assertion at `:157-159`. Engrave half: `MANIFEST` `:51`, `TAG_SLOTS` `:63`, `ROW_FLOOR` `:65`, `SECOND_TAGGED` `:67`, `THIRD_TAGGED` `:69` — all inside the cited `:50-69`. |
| **r2 N2's ordering claim** | **CONFIRMED.** `--expect` resolution is `main.rs:1473-1491` (`parse_kinds` `:1474`, `check` `:1484`, `EXIT_INVALID` `:1489`); `admit_check` is `:1504`; `consult` is `:1516`. The fold's *"IMMEDIATELY BEFORE `admit_check` … AFTER `--expect` resolution"* fixes exactly the ambiguity r2 named. |
| **r2 N3's analytic-bound citations** | **CONFIRMED.** `backup.CharsPerLine` `backup/backup.go:88-91`, `LinesPerPlate` `:93-97`, `FontSizes` `:83` (*"the descending ladder of free-text plate sizes"*), `plateSize = 85` `:77`, `fontMM` `:58-63`. |
| **The transient two-copy window still cannot red the engrave suite after the `panic:parse` probe retires** | **CONFIRMED (the one new way this could bite).** The engrave repo's only Go-invoking tests are `crates/me-cli/tests/cross_lang.rs` (an NDEF reader harness) and `preview_cross_lang.rs`; neither feeds a vector row to `nonstandard.OutputDescriptor`. So P2.6 dropping the "must NOT feed" rule cannot crash the engrave suite against the unfixed submodule. |

---

## CRITICAL

### C1 — P3.4 widens the device's key-version admission inside S2, which flips a measured `device_admits` value (a SECOND vector-file byte change invariant 1 forbids) and reds P3.3's never-relaxed rule; measured on the shipped test

The fold's answer to r2 C1 ends with an absolute:

> The Rust-primary rule makes parity mandatory, and P3.3's derived rule stays
> EXACT — it is the gate that caught this row, and it is never relaxed to fit
> the arm.

and invariant 1's fold ends with another:

> The short-fingerprint row's new `device_admits` value is MEASURED, never
> predicted (r2 M3) … the regeneration stays single and byte-identical.

Four bullets below the first of those, in the same phase, sits P3.4:

> **P3.4** F-426: the one `ypubVer` case in `bip380/bip380.go`'s classification
> switch (`bip380/bip380.go:442-455` …), with a test per direction (bare `ypub`
> classifies and normalises to `xpub`; **the host's five-version admission is
> UNCHANGED in S2** — the convergence widening is F-426's later cycle …).

F-428's neighbour F-426 states the intended edit verbatim: *"One case-arm makes
the device symmetric (`ypub` → `P2SH_P2WPKH`, normalise to `xpub`), after which
the host's five-version admission widens to match **in its own convergence
cycle**."* So S2 deliberately leaves the **device wider than the host by one key
version** — inside the phase whose gate asserts exact classification parity.

**Measured, not argued.** I `tar`-copied the fork to the scratchpad, added the
one arm (`case ypubVer: script = P2SH_P2WPKH`) to `bip380/bip380.go`'s switch,
changed nothing else, and ran the fork's own shipped test:

```
$ go test ./bip380/ ./sysw/ ./nonstandard/          # patched copy, a5e29b4 + P3.4
ok      seedhammer.com/bip380   0.008s
ok      seedhammer.com/sysw     0.036s
--- FAIL: TestDescriptorSeamDeviceColumn (0.00s)
    descriptor_seam_test.go:148: neither/full-origin-ypub: device admits = true, want false (OutputDescriptor err = <nil>)
FAIL    seedhammer.com/nonstandard      0.019s
```

The same command on the untouched fork: `ok seedhammer.com/nonstandard`. One
row moves, and only one:

```
neither/full-origin-ypub   before: err:nonstandard: unrecognized output descriptor format
                            after: ok type=Singlesig thr=1 keys=1 script=Nested Segwit (P2SH-P2WPKH) k0children=2
```

The row is `sh(wpkh([4bbaa801/49h/0h/0h]ypub6WyzNbqt…/<0;1>/*))`, single-line,
`host_admits: false`, `device_admits: false`, `format: none`. The host still
refuses it at the reviewed tree — measured just now:

```
$ me sysw pack --as descriptor --in ypub.txt   →  rc=3
me: the device admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is `ypub`, …
```

**Three consequences, each of which independently blocks the plan as written.**

**(a) Invariant 1 is violated — a SECOND byte change is forced.**
`TestDescriptorSeamDeviceColumn` asserts `device_admits` per row against
`OutputDescriptor(input)` (`nonstandard/descriptor_seam_test.go:146-150`), so the
row's boolean must flip `false → true`. Invariant 1 enumerates the single
regeneration's payload — new witness row, `sysw_class` retirement, `panic:parse`
`device_probe` retirement, F-428's citation fixes — and this is not in it. Once
the boolean is corrected, `wantDeviceTrue` (`:67`, 37 → 38) also moves, and
`:67` is the one guard the fold's new r2-M2 list does **not** name. Fixing it at
P3.4 means regenerating a file the plan says changes bytes exactly once, and the
P3 gate re-asserts *"vector-copy byte-equality + both pins"*, so the phase cannot
close green on the plan's own sequence.

**(b) The arm's missing narrowing is §4.3's, and P3.1 names §4.5's and §4.2's.**
P3.1 defines the arm as *"parse via `nonstandard.OutputDescriptor` + a port of the
cascade's single-line-reachable admission narrowings (at minimum §4.5's promotion
table … enumerate the §4.2 single-line narrowings while porting) + a port of
§4.7's conjuncts"*. The narrowing that refuses this row on the host is neither:
`KeyVersion::admitted()` is documented in the source as **"§4.3's five-member
admitted set"** (`cascade.rs:94-100`), and it fires inside `parse_extended_key`,
so `format` is `none` and no conjunct ever runs. Worse, it **cannot** be ported
as a conjunct at all: `bip380.Key` (`bip380/bip380.go:28-36`) has no version
field, and `ParseExtendedKey` normalises the version away (`:456-462`), so a port
over the parsed value is blind to it. It has to be a string-level check in the
arm, which nothing in the plan names. Walking the ported conjuncts on the parsed
value: conjunct 1 `(None, single=true)` → Ok (`sh(wpkh(KEY))` is one of the seven
forms, `admit.rs:99-103`); conjuncts 2 and 3 return Ok on `multi.is_none()`;
conjunct 4 is unavailable; 5–8 pass on a single mainnet key with `<0;1>/*`. **So
the arm answers `ClassDescriptor` on a record the host refuses**, and P3.3's
exhaustive rule — *"for every single-line input, `sysw.Classify(input) ==
ClassDescriptor` iff `host_admits`"* — reds. This is r2 C1's own failure
scenario, one row further along, and the fold's *"never relaxed to fit the arm"*
sentence removes the escape it would otherwise have.

**(c) It is not confined to the file.** A **bare** `ypub` is affected too, and no
row covers it — measured on the same patched copy: `ypub6WyzNbqt…` alone returns
`ok type=Singlesig thr=1 keys=1 script=Nested Segwit (P2SH-P2WPKH)` after P3.4
and `err: unrecognized output descriptor format` before, while `me` refuses it at
rc 3. So the device's promotion path widens as well as its descriptor path, and
P3.3's gate — which is the plan's only instrument here — cannot see it.

**And a fourth, smaller:** P3.4 falsifies a shipped operator-facing sentence and
a NORMATIVE spec sentence that P2.7's amendment list does not carry.
`crates/me-cli/src/descriptor/refusal.rs:583` tells the operator *"the device
admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`"* — pinned verbatim by a §6
row test (`crates/me-cli/tests/descriptor_refusals.rs:466`) and quoted in §6's
table (`SPEC_descriptor_input.md:1432`) — and §4.3 states **"NORMATIVE: `me`
admits exactly the same five"** (`:462`) about a device that will admit six. The
tests stay green (the host's behaviour does not change), so nothing catches it.

**What the fold has to decide** — this is a plan-level ruling, not an
implementer's call, because P3.3's assertion, invariant 1's payload and P2.7's
amendment list all inherit it:

1. **Keep P3.4 in S2** → then say so end to end: the arm carries a string-level
   §4.3 version check (both the descriptor and the bare-key paths); invariant 1's
   single regeneration carries `neither/full-origin-ypub`'s flipped
   `device_admits` **measured from a run of the P3.4-patched parser**, exactly as
   the r2-M3 rule already does for the short-fingerprint row; `wantDeviceTrue`
   (`:67`) joins the named guard list; and P2.7 gains §4.3's five-version
   NORMATIVE sentence plus `refusal.rs:583`'s text (host-side convergence, or an
   explicit "the device is wider by one version, and here is where that is
   written down").
2. **Defer P3.4 out of S2** to F-426's own convergence cycle, where the host
   widening lands first (which is also what the Rust-primary rule asks for: this
   is the device leading the host on admission, and F-426 says so in as many
   words). Then invariant 1, P3.3 and P2.7 all stand as folded, and P5.1's
   *"F-426 → resolved-in-build"* becomes *"still open"*.

Either is fine. What is not available is the plan as it stands, where P3.4 and
P3.3 are both unconditional.

---

## IMPORTANT

### I1 — the new spec-falsification inventory is complete in mechanism and not in coverage: §5.1's and §7's "after record classification fails" trigger sentences are falsified by P1.0 and owned by no task

r2's I1 asked for the falsified-spec set to be enumerated and swept with its own
tokens. The fold builds exactly that, in two places, and both are good work:
P0.1 gains a **SPEC-FALSIFICATION section**, and P2.7 gains the sentences plus
the sweep terms — *"with the falsified sentences' OWN tokens (`sysw_class`,
`panic:parse`, "PANICS the Go parser") as sweep terms, because they share no
token with the S3-parked phrasings"*. Measured: `grep -n 'sysw_class'` finds
`:1491`, `:1528`, `:1919`; `panic:parse` finds `:1541`, `:1546`, `:1604`. The
token choice works — every §7 sentence the fold names is reachable from it.

**But the inventory misses the falsifications from the plan's own biggest
structural decision, P1.0.** Two spec sentences define the gate's trigger as
record-classification failure:

`SPEC_descriptor_input.md:857-859`, inside §5.1's NORMATIVE boundary rule:

> **When `--as` is absent and record classification fails,** `me` consults the
> gate, and re-reads the whole input through §4's cascade ONLY when the gate
> opens — when the input is DESCRIPTOR-SHAPED.

`SPEC_descriptor_input.md:1563-1566`, §7's definition of the `gate_open` column:

> `gate_open` (boolean — **after record classification fails**, does §5.1's gate
> open?)

P1.0 abolishes that precondition — that is its entire purpose. The fold's own
sharpened wording makes it unambiguous: *"`consult` runs IMMEDIATELY BEFORE
`admit_check` … If `consult` identifies a descriptor, its outcome … applies
**regardless of classifiability**"*. Post-S2 `me` consults the gate on **every**
`--as`-omitted pack, including the ones where classification now SUCCEEDS (which
is precisely the r1-C1 case P1.0 exists for: after P1.1 a single-line descriptor
classifies `Descriptor`, so `admit_check` no longer fails and the old trigger
would never fire).

**Failure scenario, and it is the one r2 I1 named.** Neither list carries these
sentences, and neither sweep term reaches them: §5.1:857 and §7:1566 contain no
`sysw_class`, no `panic:parse`, no "PANICS the Go parser", and no S3-parked
phrasing. So the P2 sweep as scoped cannot find them, and after S2 merges the
spec tells a reader that the descriptor gate is consulted only after record
classification fails — the exact sentence a future implementer would re-read
before touching `main.rs:1504`, and the exact restructure r1's C1 forced. §7's
`gate_open` column then carries a definition whose precondition no longer
exists, on a column the Rust gate tests assert against every `gate`-tagged row
(`:1574-1575`).

Lesser, same class, worth one line in the same edit: §7 requirement 3
(`:1496-1498`) says *"The Rust test asserts the host column; the Go test asserts
the **device** column"*, and P3.3 makes the Go test assert a rule derived from
the **host** column. The second half of the requirement — *"Neither
implementation is ever compared to the other — both are compared to the file"* —
survives, so this is a phrasing repair, not a design break.

Fix: add both sentences to P0.1's SPEC-FALSIFICATION section and to P2.7's list,
with `gate_open` and "record classification fails" as sweep terms.

---

## MINOR

**M1 — the fold writes a machine-checkable count into the plan that is false; it
was transcribed from r2 N1 rather than recomputed.** P0.1 now says *"the grep
returns exactly seven hits, five behavioural plus these two (r2 N1)"*. Measured
at the reviewed tree and at r2's tree:

```
$ git grep -n 'DESCRIPTOR_PATH_SHIPPED' d962df1 -- crates/ | wc -l   →  9
$ grep -rn 'DESCRIPTOR_PATH_SHIPPED' crates/ | wc -l                 →  9
```

Nine, not seven. The two unlisted hits are `crates/me-cli/src/main.rs:360` (the
`use mnemonic_engrave::descriptor::{DESCRIPTOR_PATH_SHIPPED, MD1_PATH_SHIPPED};`
that feeds the `:365` conditional) and `crates/me-cli/src/descriptor/mod.rs:59`
(the public re-export). Both are inert — **no flip-set member is missing**, which
is why this is Minor and not Important. But P0.1's promise is *"Machine-count the
flip set"*, and an implementer who runs the stated grep gets a different number
than the plan and has to work out which two are unaccounted for. r2 asserted
"exactly seven"; the fold adopted it without re-running it, which is the
prescribed-fixes-are-not-authoritative shape.

**M2 — two places still describe the arm as a §4.7 port only, which line 361
superseded.** The one-paragraph summary (line 29): *"the device learns to
classify that record with the SAME predicate as the host (`sysw.Classify`
descriptor arm — **a real §4.7 port**, Rust-first …)"*, and the P3 gate's review
brief (line 465): *"proportional opus review of the port … (brief: predicate
parity **including the conjunct port**, arm order, the containment fixes, the
first-execution walk)"*. The second is the one that costs something: the P3
reviewer is briefed to check the conjunct port, so the cascade-narrowing port —
the half r2's C1 forced into existence, and the half C1 above shows is the
fragile one — is outside the brief they are given.

**M3 — the r2-M3 measured-provenance rule creates a cross-phase dependency the
plan does not schedule.** *"P3.1's parse fix is authored in the fork worktree
FIRST and P2.6 takes the boolean from a run of that fixed parser, even though the
fix's commit lands in P3."* Coherent as an instruction, and the value it produces
is robust (any clean-error fix gives `device_admits=false`). But: no owner is
named (the plan's discipline is *"ONE implementer per phase"*, and this is P3's
code authored during P2), no worktree is bound to it (`s2/descriptor-arm` is
named only in "Review cadence and scale"), it is absent from P2.6's own task
bullet and from the P2 gate, and it sits uncommitted across the P1+P2 review the
plan says must precede the port (*"the port must not begin from unreviewed
semantics"*). One clause in P2.6 naming the worktree and the owner closes it.

**M4 — P0.1's falsification section and P2.7's list disagree, and P0.1 claims
completeness.** P0.1: *"every spec sentence S2's decisions make false — §7's
`sysw_class` and `device_probe` paragraphs, §4.2 defect 4, §11 item 1's mechanism
sentence, §5.5's firmware row, §8's parked sentence"* — five items. P2.7 lists
eight (those five plus §6's table, §11 item 5, and §5.2's implementation
sentence). Nothing is unowned, because the three extra are named with their own
task in P2.7 — but the P2 gate's sweep is scoped *"per P0's inventory"*, so the
inventory is the artifact that has to be complete, and the word "every" is doing
work it cannot back. Make P0.1's section the union, or say it defers to P2.7.

---

## NIT

**N1** — the fold cites `gui/sysw_admit_oracle_test.go:64-68` for the
`walletPolicyFlow` entry; measured, that entry is `:66-69` and `:64-65` is
`{"multisig_build.go", "buildMultisigPolicyFlow", …}`. The number came from r2
verbatim. `scripts/plan-cite-check.sh` cannot see this — the lines exist.

**N2** — invariant 1's guard list names four Go-side guards; a fifth moves.
`wantDeviceAbsent` (`nonstandard/descriptor_seam_test.go:69`, `= 1 // the
panic:parse row`) becomes 0 when the probe retires. It is declared and **never
read** (grepped: no use site), so it cannot red anything — a stale constant, and
arguably a thing to delete in the same commit.

**N3** — *"`MANIFEST`, `TAG_SLOTS`, `ROW_FLOOR`, `SECOND_TAGGED`,
`THIRD_TAGGED` …, which all move with a 72nd row and its `covers` tags"*
over-states by two: `SECOND_TAGGED` (15) and `THIRD_TAGGED` (2) count rows
carrying a second and third tag, so they move only if the new witness row is
multiply tagged. Over-naming costs nothing here — the alternative failure (a
guard that moves and is unnamed) is the one r2 M2 was about.

---

## Fold vs r2's findings

| r2 | resolved? | evidence |
| --- | --- | --- |
| **C1** — the arm is §4.7-only; `promotion/15` diverges; P3.3 becomes unmeetable; P2.7 would narrow a correct §5.2 sentence | **RESOLVED as asked; a second row of the same class is this round's C1** | P3.1 now states *"The predicate is §4's cascade AND §4.7"*, names §4.5's ruling with `cascade.rs:529` quoted correctly, gives the bare `tpub` as the witness, and keeps P3.3 exact. P2.7 now protects the predicate sentence explicitly (*"is CORRECT and is NOT touched"*) and amends only `:995`. Re-measured: 17 wide rows, 16 `bip380` + the `tpub`, no 18th. **Determinacy:** "the cascade's single-line-reachable admission narrowings" is open-ended by construction ("at minimum"), but P3.3 pins the answer exactly on all 71/72 rows, so two implementers cannot diverge where it is measured — except where S2 itself creates an unmeasured divergence, which is C1(b). |
| **I1** — P2.7 omits §7 and §4.2, which the fold's own decisions falsify | **PARTIALLY** | Both are now carried, in P2.7 *and* in a new P0.1 SPEC-FALSIFICATION section, with own-token sweep terms — the mechanism r2 asked for. Two more falsified sentences (§5.1:857-859, §7:1566), from P1.0, are on neither list and are unreachable from the chosen terms. **This round's I1.** |
| **I2** — the parse-panic cite points at `:158`; guard is `:140`, panic `:149` | **RESOLVED** | P3.1 now reads *"guard `nonstandard/parse.go:140` — `len(fp) > 4`; panic at `nonstandard/parse.go:149` — `binary.BigEndian.Uint32(fp)`; §4.2 defect 4, whose spec cite is 136-149. NOT `:158` — that line is F-428's key-count error"*. All three lines measured at `a5e29b4`; invariant 1's F-428 clause still points `:158`, correctly. |
| **M1** — the sixth checklist touch point is not self-enforcing at that site | **RESOLVED** | P3.2 now says it is a `why`-string update, not a registration, quotes the stale string, and adds *"A consumer landing in a NEW function or file would need real registration"*. Oracle index (`file:fn`, `:88`) and the existing entry verified. (Line range off by two → N1.) |
| **M2** — the count-guard cite excludes the guards the new row moves | **RESOLVED** | Both halves now enumerated and all five engrave-side constants verified inside `descriptor_seam.rs:50-69`; Go-side `:66`, `:68`, `:74-77`, `:157-159` all verified. (One unused constant unnamed → N2; two over-named → N3; `wantDeviceTrue` at `:67` moves only under C1.) |
| **M3** — P2.6 writes a device-column value for behaviour that does not exist yet | **RESOLVED** | Invariant 1 now says the boolean is *"MEASURED, never predicted"*, from a run of the fixed parser authored in the fork worktree first. Coherent with the phase order and self-correcting (a wrong value reds `TestDescriptorSeamDeviceColumn` at P3.3), but unowned and unscheduled → M3 of this round. |
| **M4** — a measured-no-gain close on F-423 never reaches the operator | **RESOLVED** | P5.4 now carries *"P4.1's measured strings-per-plate number REGARDLESS of outcome"* with the reason quoted. |
| **N1** — `main.rs:350` is a second comment-only mention | **RESOLVED, with a false count attached** | Both comment sites are now named. The accompanying claim *"the grep returns exactly seven hits"* is wrong — measured 9 → M1. |
| **N2** — P1.0 does not fix `consult`'s position relative to `--expect` | **RESOLVED** | Now *"IMMEDIATELY BEFORE `admit_check` … AFTER `--expect` resolution (`main.rs:1474-1490`), whose position does not move"*, with the exit-4-vs-2 consequence stated. Line ranges verified. |
| **N3** — the analytic fit does exist (`CharsPerLine`/`LinesPerPlate`) | **RESOLVED** | P4.1 now computes the analytic upper bound first and confirms by trial, with `backup/backup.go:88-97` cited and the `FontSizes` ladder at `:83` explicitly ruled out as licence to shrink. All four citations read against the source. |

---

## What a fold has to decide, not just fix

One ruling, and it is smaller than r2's: **does F-426's device widening ship
inside S2, or with its own host-side convergence?** Everything in C1 follows from
that answer — the arm's version check, the single regeneration's payload,
`wantDeviceTrue`, P2.7's list, and P5.1's F-426 line. Deferring it is the
cheaper answer and is what the Rust-primary rule points at; keeping it is
defensible but costs four coordinated edits.

The Important is one edit: two sentences onto P0.1's and P2.7's lists, with
`gate_open` and "record classification fails" as sweep terms.
