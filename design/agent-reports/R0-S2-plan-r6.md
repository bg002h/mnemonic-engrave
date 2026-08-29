# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 6 (fold review)

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`, as folded at
`b5570a2`.
**Round 5:** `design/agent-reports/R0-S2-plan-r5.md` (RED, 0C/2I/1M/2N),
persisted at `9de0bc8`, folded at `b5570a2`. The reviewed text is exactly
`git diff 9de0bc8..b5570a2 -- design/IMPLEMENTATION_PLAN_descriptor_input_S2.md`
— 79 insertions, 31 deletions, **6 hunks, all read**.
**Trees:** mnemonic-engrave `b5570a2`; seedhammer fork `main` @
`a5e29b44637d0657ab8f1ec603f1a375b0cc54cb`.
**Nothing in either repo was modified.** No fork patch was needed this round:
every finding below rests on the vector file / `comment.json` / `gen.py` read as
data, source lines read at `HEAD` in both repos, and r1–r5's settled tables.

**THE ONE QUESTION:** does the r5 fold resolve each of r5's five findings
(I1, I2, M1, N1, N2), and did the fold's edits introduce a new defect?

**Counts: 0 Critical / 1 Important / 3 Minor / 1 Nit — verdict RED.**

**All five of r5's findings are answered, and the hard one — I1's re-ruling —
is answered well.** The new `version-gap` bullet's arithmetic is exactly right:
I recomputed the whole payload from the file and every number the fold states
(89 slots, 72 rows, overlap 17, `SECOND_TAGGED` 15, `THIRD_TAGGED` 2,
`neither` 3, `promotion-near-miss` 15, `gate` 37, `MANIFEST.len()` 10) matches,
and `SPEC:1582`'s "fifteen" does stay TRUE. Every line cite the fold added
resolves. The defect is in **where one of I1(c)'s own sites landed**: the fold
put `scripts/descriptor-seam-vectors/comment.json` under P2.7's ownership, and
`comment.json` is not text that *describes* the vector file — it is a build
**input** that `gen.py` embeds into it, so editing it after P2.6's single
regeneration either strands the shipped file's own NORMATIVE manifest at
`88`/`71` with nothing to red, or forces the second regeneration invariant 1
forbids (**NEW-I1**).

---

## Method

- The vector file, `comment.json`, `gen.py` and `rows.py` read as **data**
  (`python3 -c 'json.load(…)'`, `awk`), never by eye; every count below is
  computed in this round, not transcribed.
- `crates/me-cli/tests/descriptor_seam.rs`, `crates/me-cli/tests/descriptor_refusals.rs`,
  `crates/me-cli/src/descriptor/admit.rs`, `design/SPEC_descriptor_input.md`,
  `scripts/descriptor-seam-vectors/README.md`, and the fork's
  `nonstandard/descriptor_seam_test.go` and `bip380/bip380.go` read at the
  stated revisions.
- r1–r5's verified tables taken as settled per the brief; nothing in them
  re-derived.

### The payload arithmetic, recomputed end to end (brief's pressure point 1)

Simulated on the real file: retag `neither/full-origin-ypub` → `["version-gap"]`
(single), append one single-tagged `neither` witness row.

| quantity | before (measured) | after (computed) | plan says | verdict |
| --- | --- | --- | --- | --- |
| physical rows | 71 | **72** | `ROW_FLOOR` 71 → 72 | ✅ |
| tag-slots | 88 | **89** | `TAG_SLOTS` 88 → 89 | ✅ |
| overlap (`slots − rows`) | 17 | **17** | "overlap stays 17" | ✅ |
| distinct tags = `MANIFEST.len()` | 9 | **10** | `MANIFEST` gains `("version-gap", 1)` | ✅ |
| `neither` | 3 | **3** | `("neither", 3)` does not move | ✅ |
| `promotion-near-miss` | 15 | **15** | unchanged, so `SPEC:1582` stays TRUE | ✅ |
| `gate` | 37 | **37** | `POP.gate_fields` stays 37 | ✅ |
| rows with ≥2 tags | 15 | **15** | `SECOND_TAGGED` stays 15 | ✅ |
| rows with 3 tags | 2 | **2** | `THIRD_TAGGED` unmoved (unstated, correctly) | ✅ |

Every assertion in `descriptor_seam.rs::the_coverage_manifest_is_met_by_count_not_by_reading`
was checked against the post-payload numbers by hand-execution of its body:
`assert!(n >= *min)` ✅ · `assert_eq!(got.len(), MANIFEST.len())` 10 == 10 ✅ ·
`assert_eq!(slots, TAG_SLOTS)` 89 == 89 ✅ · `assert!(rs.len() >= ROW_FLOOR)`
72 ≥ 72 ✅ · `assert_eq!(second, SECOND_TAGGED)` 15 ✅ ·
`assert_eq!(third, THIRD_TAGGED)` 2 ✅ · no row with >3 tags ✅ ·
`assert_eq!(slots − rs.len(), SECOND_TAGGED + THIRD_TAGGED)` 17 == 17 ✅ · the
"only the fifteen §4.5 rows may carry a second tag" loop — **not tripped, because
both moved rows are single-tagged**, which is exactly the property the fold
asserts and the reason it holds ✅. `the_row_set_is_not_vacuous` computes
`neither` from the data (`:429-449`), so the substitution is invisible to it ✅.

### Cite resolution — every line the fold added, checked against the file

| fold's cite | what is actually there | ✓ |
| --- | --- | --- |
| `descriptor_seam.rs:373` | `assert_eq!(slots, TAG_SLOTS, "tag-slot total");` | ✅ |
| `descriptor_seam.rs:62` | `/// The minima sum to 88 tag-slots.` | ✅ |
| (`:64`, `/// 88 − 17 overlap slots = …`, is a second falsified comment — covered by the guard list's existing `:50-69` range) | | ✅ |
| `SPEC:1582` | `- **the promotion near-misses of §4.5** — all **fifteen** rows of that table;` | ✅ TRUE after the payload |
| `SPEC:1610-1615` | the `neither` bullet **plus** "The `multi` row additionally carries …" through the `md_descriptor_contains` pin — the exact span r5's M1 asked for | ✅ |
| `SPEC:1719-1723` | "at least **71 physical rows** (the minima sum to **88** tag-slots … 88 − 17 = 71)" | ✅ |
| `SPEC:1728-1732` | floor-table rows `promotion-near-miss`(1728) … `neither`(1732); insertion point for the new row is inside the span | ✅ |
| `comment.json:101-113` | `THE COVERAGE MANIFEST` header + all nine tag lines + the three-line `88 − 17 = 71` derivation — the whole block, exactly | ✅ |
| `bip380/bip380.go:335` (fork) | `case "sortedmulti":`, `default: return nil, …unknown script type` | ✅ r5 N1's mechanism, correctly re-cited |
| `descriptor_seam_test.go:66` `wantRows = 71` (fork) | present, already on the plan's guard list at `plan:117` | ✅ named |

### Checked, no defect — stated for the record

- **The `ypub` sweep term still reaches every P3.4-falsified site, and the new
  `tag-slots` term reaches what it does not.** Measured: `tag-slots` hits
  `SPEC:1720`, `comment.json:111`, `descriptor_seam.rs:62` and the vectors
  file's `_comment`; `ypub` hits `SPEC:1610-1615`, `SPEC:1732`,
  `comment.json:107`, `refusal.rs:583`, `admit.rs:23` and
  `descriptor_refusals.rs:463`. Union covers every moved site except one
  (Minor 3).
- **No member is owned by both P2.7 and P3.5.** P2.7's §7 group (neither
  bullet + named rows, derivation, floor table, the new bullet) and P3.5's four
  (`SPEC:453-461`, `SPEC:570-574`, `cascade.rs:58-62`, `refusal.rs:583` + pin +
  §6 quote) are disjoint, and each of P3.5's four is falsified by P3.4's arm —
  correctly P3.5's. Two *pre-existing* P2.7 members are on the wrong side of
  the fold's new rule (Minor 1).
- **Pressure point 2 — "retires when the host widens" creates no S2 defect.**
  A min of 1 with exactly one member is well-formed under
  `assert!(n >= *min)` + `assert_eq!(got.len(), MANIFEST.len())`; retiring the
  bullet later requires a coordinated MANIFEST + row edit, which is the
  property the manifest exists to force, and the arithmetic belongs to F-426's
  own cycle. Nothing in S2's gates depends on the bullet being permanent. No
  finding.
- **No `Pop` field beyond the six named moves under the retag.** The `ypub` row
  carries no gate/address/canonical fields; the witness row carries none.

---

## IMPORTANT

### NEW-I1 — `comment.json` is a build INPUT to the sha-pinned vector file, and the fold assigned it to P2.7, the task that runs AFTER P2.6's single regeneration

The fold's new P0.1 member (`plan:227-233`), verbatim:

> §7's named `neither` rows and manifest arithmetic
> (`design/SPEC_descriptor_input.md:1610-1615`, `:1719-1723`, `:1728-1732`,
> plus `comment.json`'s manifest block — the full-origin `ypub` stops being a
> false/false row and the 72nd row moves the slots/floor derivation; see
> invariant 1's `neither`-tag ruling; **owned by P2.7** per the r5 I2 split
> rule, NOT P3.5 — P2.6 is the falsifying diff)

The split rule itself is right, and for the three SPEC citations it is right.
For the fourth it is not, because `comment.json` does not *describe* the vector
file — it is *compiled into* it:

```
scripts/descriptor-seam-vectors/gen.py:209
    "_comment": json.load(open(os.path.join(SP, "comment.json"))),
```

Measured, this round: `json.load(vectors)['_comment'] == json.load(comment.json)`
→ **True**, 125 elements, exact. The vectors file's bytes — and therefore
`SEAM_VECTORS_SHA256` in `descriptor_seam.rs:45` and `seamVectorsSHA256` in the
fork — are a **function of `comment.json`**. The split rule's premise ("owned by
the phase whose diff falsifies it") is a rule about prose. Applied to a
generator input it inverts the dependency: the file that produces the artifact
is scheduled to be edited after the artifact is produced and pinned.

**Why the wrong reading is the one the plan produces.** P0.1 is declared
authoritative — *"this section is the COMPLETE enumeration — P2.7 and P3.5 defer
to it, not the reverse"* (`plan:202-204`). P2.7's own list (`plan:462-467`)
names only the three SPEC sites and the new bullet; `comment.json` is absent.
P2.6's own bullet (`plan:421-427`) names *"`scripts/descriptor-seam-vectors/rows.py`
+ JSON together"* — `comment.json` is absent there too. The only text placing it
anywhere is P0.1's, and P0.1 says **P2.7**.

**Constructed counterexample — the plan followed literally, one repo, no test
red.**

1. **P2.6.** Edit `rows.py` (witness row, retag, `source`/`name`, F-428 cites).
   Do **not** edit `comment.json` — P0.1 owns it to P2.7. Run `gen.py`. The
   emitted file now has 72 rows and a `version-gap` tag, while its own embedded
   `_comment` still reads:
   `"  neither              3   wsh(multi), miniscript, full-origin ypub"` and
   `"The minima sum to 88 tag-slots … carries at least 88 - 17 = 71 PHYSICAL rows"`
   — under its own header `"THE COVERAGE MANIFEST (S7, NORMATIVE; S11 item 3
   counts against it):"`. Bump the pin. Update `MANIFEST`/`TAG_SLOTS 89`/
   `ROW_FLOOR 72`/`Pop`. **Suite green** — measured: no Rust or Go test reads
   `_comment` (`grep -rn '_comment'` over `crates/**/*.rs` and the fork's
   `nonstandard/*.go` returns exactly one hit, a doc comment in
   `src/descriptor/mod.rs:23`), and CI never runs `gen.py`
   (`grep -rn 'gen.py\|descriptor-seam-vectors' .github/workflows/` → no hits).
2. **P2.7.** Edit `comment.json`'s block to 10 tags / 89 / 72. Do not
   regenerate — invariant 1 says *"the regeneration stays single and
   byte-identical"*, *"ONE sha bump per repo"* (`plan:145-147`).
3. **P2 gate green.** Ship state: the vendored, byte-identical, sha-pinned data
   file carries a NORMATIVE coverage manifest that is false about itself in
   three numbers and one row list; `comment.json` and the artifact it generates
   disagree; and the README's documented reproduction —
   `python3 gen.py …` then re-pin — now produces a **different sha than the one
   pinned in both repos**, which is precisely the drift signal the two literals
   exist to give. The repo's own README states the lesson this violates: *"a
   reproduction path nobody re-runs rots while its artifact keeps vouching for
   it."*

The only other reading of step 2 — regenerate again — costs a second sha bump in
both repos, contradicts invariant 1's single-regeneration clause, invalidates
P0.2's BEFORE measurement as the sole baseline, and adds a moving part to P3's
byte-equality gate. Both branches are wrong; the plan picks neither.

**Fix, one clause.** In P0.1's member and in invariant 1's arithmetic list,
split the site: *"`comment.json`'s manifest block is a generator INPUT to the
single regeneration and is edited AT P2.6 with `rows.py`, before `gen.py` runs;
only the three SPEC sites are P2.7's."* Add the same qualification to the split
rule so a future member that is an input rather than a description is not routed
by the same mistake. Nothing else reopens: the arithmetic, the tag name, the
bullet and the SPEC ownership are all correct as written.

---

## MINOR

**M1 — the fold's new ownership rule is not applied to two pre-existing P2.7
members.** The rule is stated absolutely: *"a member is owned by the phase whose
diff falsifies it"* (`plan:457-458`). Two members that sit three lines above it
in the same task are falsified by **P3** diffs:

- *"§7 requirement 3's device-column phrasing"* — P0.1 gives its falsifier as
  *"P3.3's derived rule reads the host column"* (`SPEC:1496-1498`, read:
  *"The Rust test asserts the host column; the Go test asserts the device
  column."*). P3.3 is a fork commit in P3.
- *"§4.2 defect 4's 'PANICS the Go parser' sentence"* — P0.1's own parenthetical
  is *"(false after the convergence fix)"*, and the convergence fix is **P3.1**.
  (Weaker case: P2.6 retires the row's `device_probe` marker in the same window,
  so a P2-side falsifier exists too.)

Amended at P2.7, the P2 gate closes with the spec describing a fork-side test
that fork `main` does not have — the shape r4's M2 moved five other members for.
Graded Minor, not Important, because the contradiction is cross-repo (the plan
already tolerates and *states* a cross-repo transient for the vector copies at
`plan:148-152`) rather than r5 I2's in-repo, single-commit contradiction with a
constant labelled "§7 NORMATIVE". One sentence closes it either way: move them,
or state the transient the way invariant 1's sequencing paragraph does.

**M2 — the retag renames the row, and two by-name sites are outside the
"COMPLETE" enumeration.** Invariant 1 says the flip falsifies *"its
`name`/`covers`"*, and every row name in the file is `<tag>/<slug>`, so
`neither/full-origin-ypub` becomes `version-gap/…`. Measured, the name is
referenced outside `rows.py` at:

- `crates/me-cli/tests/descriptor_refusals.rs:463` —
  `&vector_input("neither/full-origin-ypub")` inside `row_unsupported_key_version`.
  The helper ends `panic!("{VECTORS}: no row named {name:?}")`
  (`descriptor_refusals.rs:48`), so this reds **loudly and unambiguously** at
  P2.6 — which is why this is Minor and not Important. Note the plan already
  names `descriptor_refusals.rs:466`, the refusal-text pin *three lines below
  it*, as P3.5's; `:463` is the same test and moves at P2.6.
- `crates/me-cli/src/descriptor/admit.rs:23` — a source comment citing the row
  by name as evidence for a *"measured fact rather than an oversight"*
  argument. Silently stale after the rename; the plan's own
  `cascade.rs:58-62` member exists for exactly this failure mode.

Both are reachable by the `ypub` sweep term (each string contains `ypub`), so
the sweep would surface them; they are simply not in the enumeration that calls
itself COMPLETE.

**M3 — `scripts/descriptor-seam-vectors/README.md:9` is falsified by the 72nd
row, named nowhere, and reached by no sweep term.** Verbatim: *"`rows.py` — the
**71** row DEFINITIONS."* Measured against the full term list
(`sysw_class`, `panic:parse`, "PANICS the Go parser", `gate_open`, "record
classification fails", `ypub`, `tag-slots`): **zero hits in that file's line 9**.
It is the live operating manual for the regeneration the plan is about, and it
is the one moved count with neither an owner nor a finder.

---

## NIT

**N1 — the new `tag-slots` sweep term hits a completed sibling plan, and the
plan states no disposition for that class.** Measured,
`design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md:33` reads *"≥ **71**
physical rows, **9 tags**, **88** tag-slots, per-tag minima (`formats-happy` 4 ·
`promotion-near-miss` 15 · …)"* — a fourth copy of the manifest, including the
only exhaustive **tag-count** literal in the repo. It is a record of a shipped
plan, so almost certainly it should be left alone; but the P2 sweep's stated
survival set is *"`design/agent-reports/` and historical review text per P0's
inventory"*, and a completed sibling **plan** is neither. Half a sentence in
P0's inventory ("completed plans are records; sweep hits there are triaged, not
amended") makes the hit a non-event instead of a decision the P2.6 commit has to
discover.

---

## Fold vs r5's findings

| r5 | resolved? | evidence |
| --- | --- | --- |
| **I1** — the ruling retagged into a tag no §7 bullet admits, named two tags in one sentence, and declared settled arithmetic that moves in three artifacts | **RESOLVED on all three sub-parts; one of I1(c)'s sites landed in the wrong task → NEW-I1** | (a) Destination is now unambiguous and single: a NEW single-member §7 bullet, explicitly NOT `promotion-near-miss` / NOT `narrowed-4.7` / NOT `gate`, *"the row stays single-tagged"*. Verified: `SECOND_TAGGED` 15 and `THIRD_TAGGED` 2 are unmoved and `POP.gate_fields` stays 37. (b) No closed membership is joined, and `SPEC:1582`'s *"all **fifteen** rows"* is **TRUE** after the payload (measured: `promotion-near-miss` = 15). (c) The arithmetic is enumerated in all three copies and every number is right — 89 / 72 / 17 recomputed from the file, `MANIFEST.len()` 10 == distinct tags 10, and `assert_eq!(slots, TAG_SLOTS)` satisfiable. **But** `comment.json` — the site r5 I1(c) added — is stamped "owned by P2.7", and it is the generator input to P2.6's regeneration. |
| **I2** — a member falsified by P2.6 was owned by P3.5 | **RESOLVED, with a general rule** | The fold states the rule (*"a member is owned by the phase whose diff falsifies it"*) and executes it in all three places: P0.1's member (*"owned by P2.7 … NOT P3.5 — P2.6 is the falsifying diff"*), P2.7 (the §7 group listed under *"Falsified by P2.6's regeneration — they describe the VECTOR FILE, not the device"*), and P3.5 (*"membership corrected by r5 I2 — §7's `neither` rows went BACK to P2.7"*). Verified disjoint: no member is on both lists, and each of P3.5's remaining four is falsified by P3.4. Two pre-existing P2.7 members are on the wrong side of the new rule → **M1**. |
| **M1** — the SUBSTITUTION leaves `SPEC:1612`'s "The `multi` row" referent ambiguous | **RESOLVED, in both places** | Invariant 1: *"the amendment DISAMBIGUATES the next sentence's 'The `multi` row' referent, which stays `neither/wsh-multi` alone … `SPEC:1612-1615` is true of the existing row only"*; P2.7's task line carries *"with the 'The `multi` row' referent disambiguated"*. The cite widened `:1610-1611` → `:1610-1615`, which I verified spans the bullet plus the whole `md_descriptor_contains` sentence. |
| **N1** — "(measured, r3)" mis-attributed the `multi` rejection | **RESOLVED, and the mechanism is now cited** | Now *"(measured, r2 — r5 N1 corrected r4's mis-attribution; `bip380/bip380.go:335` cases `sortedmulti` only)"*. Verified in the fork at `a5e29b4`: `:335` is `case "sortedmulti":` with a `default:` that errors. |
| **N2** — "r4 measured …" overstated a constructed counterexample | **RESOLVED, exactly as asked** | Now *"r4 CONSTRUCTED the counterexample … and MEASURED its two halves: the P3.4-patched probe accepts the bare `ypub`, and `me` refuses the identical string at rc 3"*. That is r5 N2's own wording of what r4 did and did not have. |

---

## What a fold has to decide, not just fix

Nothing this round is a ruling. **NEW-I1 is one clause**, and it does not
reopen the taxonomy, the arithmetic, the tag name or the SPEC ownership — all of
which are correct. It only says *when* `comment.json` is edited, and the answer
is forced by `gen.py:209`: with `rows.py`, at P2.6, before the single
regeneration.

The three Minors and the Nit are all "one more site" or "one more sentence" and
none of them gates. Worth folding together with NEW-I1 so the next round has a
single diff to read: `comment.json` → P2.6; §7 req 3 and §4.2 defect 4 → P3.5
(or a stated transient); `descriptor_refusals.rs:463`, `admit.rs:23` and
`README.md:9` into P0.1's enumeration with owners; and a half-sentence on
completed sibling plans in the sweep's survival set.
