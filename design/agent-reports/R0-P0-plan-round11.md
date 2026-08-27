# R0 ROUND 11 — `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` at `e3b95cb`
(worktree `review/p0-r11`).
**Round 10's report:** `design/agent-reports/R0-P0-plan-round10.md` (2C/4I/6M/1N).
**The fold under review:** `e3b95cb` — 20 insertions, 8 deletions in the plan,
**6 hunks at `-U0`**, new-file lines **270-280, 290-294, 486, 491, 528, 580**
(4 hunks at default context).
**Object:** (1) did moving the guard into `me` break anything; (2) did the fold
close round 10's findings, verified against the DIFF; (3) can an implementer
execute all twelve rows.
**Date:** 2026-08-27.

## VERDICT — **NOT GREEN**

| severity | count |
| --- | --- |
| **Critical** | **2** |
| **Important** | **2** |
| Minor | 6 |
| Nit | 1 |

**The ruling is right and the fold reached four of C-1's five sites.** Keeping
the guard in `me` is the correct answer to the cycle, `records.rs` is now
honestly scoped in §3's file table, and the trim+lowercase gap and the override
gate are real improvements. **The design is not re-opened here and nothing below
argues against it.**

**What did not happen, again, is propagation — and this time it landed on the
one site that assigns the crate's contents.** §3's file *tree* (line 208) still
reads `src/records.rs — record stream splitting, **the argv gate**, kind
vocabulary`. That line is round-10 C-1's **first** named site, it is untouched by
the diff, and the plan's own propagation gate flags it. It is worse than a stale
sentence: it is the **only** place in the plan that assigns the `--expect` kind
vocabulary to a file, and that vocabulary resolves through `Class`, `Admission`
and `chunk_key` — all `me`-internal. So the tree schedules P0's *second*
funds-path feature into the crate that cannot hold it, by the same cyclic
package dependency round 10 reproduced for the first one.

And the third site round 10 named — §3's representation ruling — was edited into
a sentence that asserts the crate has string recognisers **and** that it has
none, ends mid-clause, and leaves an unmatched `**` (file total went 662 → 681,
even → odd).

---

## QUESTION 1 — DID MOVING THE GUARD INTO `me` BREAK ANYTHING?

### 1a. Does the crate still have a coherent job? — **YES. This is not a finding.**

Recorded so it is not re-opened. With the guard gone, §3's file table gives
`records.rs` two functions:

| function | measured |
| --- | --- |
| `split_record_stream` (`crates/me-cli/src/main.rs:1867`) | **6 lines**, body is `raw.lines().map().filter().collect()` |
| `no_records_guard` (`crates/me-cli/src/main.rs:1896`) | **~25 lines**, carrying R7's empty-input refusal and its reasoning |

Thin, but `no_records_guard` is a *ruling* — an empty input is refused rather
than packed, because a container built from nothing is a silent success — and
that is exactly the kind of thing six binaries should not each decide
separately. The crate still earns its existence on `fd.rs` (the two `None`s and
the raw mode), `exit.rs` (the write-gate decision and ordering), `remedy.rs`
(the purge text `mt` gets wrong) and `observation.rs` (F-259's type). **The
guard leaving does not hollow it out.** No change needed.

### 1b. Does anything else in the plan still assume the guard is in the crate? — **YES, one site, and it is C-1**

`grep -n 'argv'` over the plan, all 13 hits read. Twelve are correct. The
thirteenth is **line 208**.

### 1c. Is row 9b's gate now satisfiable? — **UNDER §3's TABLE YES; UNDER §3's TREE NO**

Checked directly against the donor, not inferred:

| would live in the crate (per §3's **table**) | names `Class`? | names `EXIT_*`? |
| --- | --- | --- |
| `split_record_stream` (`main.rs:1867`) | no | no |
| `no_records_guard` (`main.rs:1896`) | no | **yes today — row 1 removes it** |
| `destination` (`main.rs:940`) | no | no |
| `write_block` (`main.rs:971`) | no | no |
| `stdout_world_readable_mode` (`main.rs:896`/`:921`) | no | no |

So *"the crate builds standalone; no `EXIT_*` and no `Class` in it"* **passes**
if the implementer follows the file table. It **fails** if they follow the file
tree, which assigns two more things to `records.rs` that cannot be there. The
plan gives both, and the tree is unopposed on one of them.

---

## CRITICAL

### C-1. §3's file tree still assigns *"the argv gate"* and *"kind vocabulary"* to `src/records.rs`. It is round-10 C-1's first named site, untouched by the diff, and it is the ONLY place the plan assigns the `--expect` kind vocabulary to a file — which cannot live in the crate for the same reason the guard cannot.

**Site:** §3, **line 208**.

```
  src/records.rs      — record stream splitting, the argv gate, kind vocabulary
```

**Machine-checked with the plan's own gate**, run with the phrasings this fold
retracts:

```
$ ./scripts/fold-propagation-check.sh design/IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md \
      'prefix / HRP / wordlist' 'by HRP' 'by wordlist' 'string-level recognisers' \
      'the argv gate' 'P0 builds both'
  gone   prefix / HRP / wordlist
  gone   by HRP
  LEFT   by wordlist
           281:BIP-39 mnemonic by wordlist.
  gone   string-level recognisers
  LEFT   the argv gate
           208:  src/records.rs      — record stream splitting, the argv gate, kind vocabulary
  gone   P0 builds both
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.
rc=1
```

**The argv half** is round 10's C-1 verbatim, and the plan now contradicts itself
three ways about one file: line 208 says `records.rs` holds the argv gate, line
274 says *"`records.rs` keeps stream shaping; the guard is the donor's"*, and
line 486 says *"stream shaping only"*.

**The kind-vocabulary half is the part nobody has looked at, and it is worse.**
`grep -n 'kind vocabulary'` over the plan returns **one hit — line 208**. That
is the only assignment of `--expect`'s vocabulary to a file anywhere in the
plan, and §3's own table (line 486) now excludes it from `records.rs` without
naming a replacement home. Meanwhile §3's `--expect` table resolves kinds
through symbols that are all `me`-internal, verified by path this round:

| symbol `--expect` needs | where it lives |
| --- | --- |
| `Class` (`Mt`, `Tx`, `Mnemonic`, `Codex32Secret`) | `crates/me-cli/src/sysw/record.rs:44` |
| `Admission` (probe C-2: *"one parameter fixes it"*) | `crates/me-cli/src/sysw/mod.rs:165` |
| `mdmk_unconfirmed` | `crates/me-cli/src/sysw/record.rs:168` |
| `mt_unconfirmed` | `crates/me-cli/src/sysw/mt.rs:207` |
| `chunk_key` (the HRP discriminant) | `crates/me-cli/src/seal/record.rs:247` (`pub(crate)`) |

All five are in crate `mnemonic-engrave` (`crates/me-cli/Cargo.toml`).

**The concrete failure.** An implementer reaches row 8, opens §3 to find where
`--expect` goes, reads line 208, and builds the kind vocabulary in
`mnemonic-io-lib::records`. That module must then name `Class::Mt`,
`Class::Codex32Secret` and `Admission`, so `mnemonic-io-lib` needs a dependency
on `me-cli` — **`error: cyclic package dependency`**, the exact build failure
round 10 reproduced, at a second feature. Row 9b's gate *"no `Class` in it"*
goes RED with it. And the alternative reading — leave `--expect` in `me` — means
P0's crate contributes **nothing** to either of the two features §7 P0's row
calls out by name, which no sentence in the plan says.

**What closes it.** Rewrite line 208 to match the table it contradicts —
`src/records.rs — record stream splitting only; the argv guard and the
`--expect` kind vocabulary stay in `me` (both need `Class`)` — and re-run
`fold-propagation-check.sh` with `'the argv gate'` and `'kind vocabulary'` among
the patterns, quoting rc in the fold commit. Then answer, once, in §3: what does
the crate contribute to the argv and `--expect` work? (→ **I-1**.)

### C-2. §3's representation ruling — round-10 C-1's third named site — was edited into a sentence that asserts the crate has string recognisers and that it has none, ends mid-clause, and leaves an unmatched `**`. The next sentence's *"that kind"* now has no antecedent.

**Site:** §3, **lines 527-530**.

```
**So the split is by REPRESENTATION.** The crate's recognisers work on strings —
**the crate holds no recognisers at all** — classification is the donor's, and the crate's**
kind type. `me` maps that kind onto its `Class` and keeps the three predicates.
Nothing in the crate ever names a `Class` variant.
```

The fold replaced `a `tx:` prefix, an HRP character, a BIP-39 word — and return
the crate's **own**` with `**the crate holds no recognisers at all** —
classification is the donor's, and the crate's**`, dropping the word the
sentence turned on and leaving the emphasis marker behind.

**Machine-checked:** `grep -o '\*\*' <plan> | wc -l` → **681** at `e3b95cb`,
**662** at `ce5a206`. Even → **odd**: the fold introduced exactly one unmatched
bold delimiter, and it is at line 528.

**The concrete failure.** This is §3's normative boundary rule — the paragraph an
implementer consults to decide what may go in the crate — and its first clause,
*"The crate's recognisers work on strings"*, instructs building recognisers
there. That is the design round 9 called Critical and round 10 confirmed
retracted. Read alongside line 208 the two compose into a coherent wrong plan:
string recognisers in `records.rs` producing a kind type. **Deleting the sentence
outright would leave the reader better informed than it does**, because the file
table and the guard paragraph are both correct — which is the test for whether
text earns its place.

**What closes it.** One sentence stating the split as the fold actually ruled
it: the crate holds **no** classification — no recognisers, no kind type derived
from a record's shape; `me` classifies, and `me` keeps the three predicates.
Then `**` count is even again, and *"that kind"* either gains an antecedent or
goes.

---

## IMPORTANT

### I-1. Row 6's new trim+lowercase requirement has no observable that can fail, and the warrant it cites for it is false — so the obvious implementation (copy the donor's two lines) still leaks uppercase `MS1…`.

**Site:** §4 row **6**, line **580**, step column and gate column.

**Half one — no gate.** The step column now requires *"every token is TRIMMED AND
LOWERCASED before classification"*. The gate column's cross-product has three
axes:

```
{bare, bundle, sysw, sysw pack, sysw show, sysw wipe, help, sysw help}
  × {positional, --in X, --in=X}
  × {every argv-forbidden class, `pass:` included}
```

The third axis is *a set defined by what `classify()` returns*, so every carrier
it generates is a **canonical** spelling. No generated row can carry ` TX:<hex>`
or `MS10ENTRS…`. A guard written without `.trim().to_ascii_lowercase()` passes
all 8×3×5 rows, passes the ordering test (canonical `ms1`), and passes all three
positive controls. Round 10 measured what then ships: a 447-char signed
transaction echoed to stderr on 4 surfaces, a codex32 secret on 2. **The plan's
own standard, §3: *"A gate that cannot fail is not a gate."*** Round 10 named the
fix — *"add one carrier to the cross-product that is a near-miss"* — and the fold
added the requirement without it. This is the same shape as round-10 I-4, which
this fold correctly closed; it was recreated one clause away.

**Half two — the warrant is false, verified in source this round.** Row 6 says
*"The donor's shipped post-parse gate already does trim+lowercase and its comment
says why."* The donor does **not** normalise before classification:

```
main.rs:1952   let trimmed = r.trim_start().to_ascii_lowercase();
main.rs:1958   let by_prefix = trimmed.starts_with(...TX_PREFIX);
main.rs:1978   let class = mnemonic_engrave::sysw::classify(r);   // ← RAW r
```

`trimmed` feeds the **`tx:` prefix arm only**; `classify` receives the raw token.
And `classify` is case- and whitespace-sensitive by construction —
`record.starts_with(record::TX_PREFIX)` at `crates/me-cli/src/sysw/mod.rs:184`,
with no normalisation anywhere in it.

So the guard row 6 specifies is *stronger* than the donor's shipped gate, and the
sentence says the opposite. An implementer who follows round 10's own advice —
*"copying two lines the donor already has"* — reproduces `by_prefix` on the
normalised token plus `classify(raw)`, which catches ` TX:` and `TX:` and stays
**blind to uppercase `MS1…`, ` pass:` and an uppercase mnemonic**. Every gate row
still passes.

**What closes it.** (a) Add a fourth axis or one explicit extra carrier to the
cross-product — a near-miss spelling (` TX:<hex>` and `MS1…` uppercase), which is
RED today on six surfaces. (b) Correct the warrant: say the donor normalises for
its **prefix arm only** and passes the raw token to `classify` (`main.rs:1952`,
`:1958`, `:1978`), so the pre-parser guard normalises *before* `classify` and is
deliberately stronger. Both are one line.

### I-2. The plan defers two things §7 P0 assigns to the shared crate — the argv guard's residence, and §6d's *primary* flag-name layer — and §7 OUT OF SCOPE records neither. F-268's stated trigger is already satisfied today, so the layer P3 needs is scheduled to nothing.

**Sites:** §7 (whole section, lines 828-851); §6's eleven conditions; §3 line
**299**; `design/FOLLOWUPS.md:11853` (F-268).

Round 10's C-1 closure said it explicitly: *"§3 must say what P0's crate
contributes to the argv work instead (plausibly: nothing, and that is an honest
answer)."* Round 10's I-3 closure said *"a `FOLLOWUPS.md` entry with an owning
phase **plus a sentence in §7**"*. The fold did neither §7 half.

**What the spec assigns to the crate at P0** (`SPEC_constellation_cli_uniformity.md`
§7, P0 row): *"**the shared crate**: `--in`/`--out`/`-`, **argv guard with
pre-parser ordering**, write gate, exit codes, remedy text …, **and `me sysw
pack --expect` in full — the kind vocabulary, the flag, and §6g's refusal**"*.
After this fold the crate holds the argv guard: **no**. `--expect`: **no**
(C-1). §6d's primary layer: **no** (F-268). The plan's §7 lists the verb
migration, the mask disagreement, `mnemonic-toolkit`'s adoption and F-260 — and
none of these three. §6's closing conditions mention none of them either, so
**P0 closes GREEN with three spec-assigned deliverables absent from the crate
and unrecorded as departures.**

**The second-order consequence, which nothing owns.** §7 P1 is *"`mt` adopts the
crate, and gains … `--allow-argv-secret` (§6d)"* — an override to a guard the
crate does not supply, so `mt` keeps its shape-based `command_line_guard` and
§6d's two divergent implementations survive the cycle whose D5 exists to end
them. P2 (`ms`'s argv refusal) and P3 (`mnemonic`'s, across five channels) each
inherit the same non-seam.

**F-268's trigger is already true — verified live this round, not read.**
`design/FOLLOWUPS.md:11853` gives its owning phase as *"whichever cycle first
adds a secret-bearing flag to any m-format CLI."* That flag exists and ships:

```
$ /scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic restore --help
      --passphrase <PASSPHRASE>
          BIP-39 mnemonic-extension passphrase. …
      --passphrase-stdin
```

declared at `crates/mnemonic-toolkit/src/cmd/slip39.rs:101`, and enumerated in
`flag_is_secret` at `crates/mnemonic-toolkit/src/secrets.rs:60-68` alongside
`--bip38-passphrase` and `--decrypt-password`. **Spec §7 P3 names two of those
flags in its own gate** — `restore --passphrase` and `electrum-decrypt
--decrypt-password` — and §6d rules that the value-shape layer cannot see them
(*"arbitrary text has no shape"*). So P3 is gated on a layer P0 deferred to a
condition that was already met before the deferral was written. Under the
constellation rule, that is **overdue, not deferred**.

**What closes it.** Three sentences, all in this plan:

1. §7 gains an entry: *P0 ships §6d's value-shape layer only; the flag-name
   layer is F-268* — with F-268's owning phase corrected from *"whichever cycle
   first adds a secret-bearing flag"* to **P3**, the phase whose gate needs it.
2. §3 (or §7) states what the crate contributes to the argv and `--expect` work.
   *Nothing* is an acceptable answer and round 10 said so — but it has to be
   written, because §7 P0 says otherwise and the plan cites §7 P0 as its
   authority at line 100.
3. §3 line 299's *"the layer's own conformance test is asserted against
   `mnemonic-toolkit`"* describes F-268's test now, not P0's — mark it as such
   or delete it.

---

## MINOR

**M-1. The retracted wordlist recogniser survives as an orphaned fragment at the
fold's own site, and the propagation gate was run with patterns that missed it —
second consecutive round.** Line **281** is `BIP-39 mnemonic by wordlist.`, the
tail of the four-shape sentence the fold deleted the head of, now reading as *"…
so there is no list to be short. A / BIP-39 mnemonic by wordlist."* The commit
message reports *"fold-propagation 0 … and it caught one"*; run with round 10's
own four patterns plus this fold's, it returns **rc 1** (see C-1). Round 10's
C-2 was precisely *"the gate was run, with patterns that did not include what
this fold retracted"*. Delete the fragment; add `'by wordlist'` and
`'the argv gate'` to the pattern list.

**M-2. Round-10 I-4's residue: the override's scoping is still unstated and its
second assertion is still missing.** Row 6 gained *"`me sysw pack
--allow-argv-secret <ms1>` must still exit 0"* ✔ — the half that carried the
harm, since round 10's stated failure was a regression caught four rows late.
Not added: the five words saying the override is honoured **only where the flag
is declared** (`me` declares it on `sysw pack` alone, `crates/me-cli/src/main.rs:252`),
and the companion row *`me bundle --allow-argv-secret <ms1>` is still refused by
the guard*. **Graded Minor rather than Important, deliberately:** round 10
measured that a global bypass does not leak on this donor today, because clap
names the unknown *flag* on every surface that does not declare it, and `me sysw
pack` is the surface that declares it. The residue is latent for consumers, not
live for P0.

**M-3. Round 9's five carried Minors are untouched for an eighth round.** None is
claimed in the commit message. Verified present this round:

| round 9 | phrase | `grep -c` |
| --- | --- | --- |
| M-6 | `so P0 fixes the recipe` (condition 9's splice + duplicated recipe) | 1 |
| M-7 | `really 11 and not more` (the move relocates five + the stub) | 1 |
| M-8 | `crates/me-cli/src/io.rs` (nested backticks, a module §3's tree never lists) | 1 |
| M-9 | `the digit-pinning work` | 1 |
| M-10 | `enumerate every type and constant` (line-wrapped after *"and"*, so a single-line grep returns 0 — it is present at §3's TYPES paragraph) | 1 |

**M-4. §2.4's conclusion is now stale, and the fold made it so.** Line **196**:
*"The seam buys unit-testability for the **argv and record** half, not the fd
half."* The argv half no longer crosses into the crate at all, so the seam buys
it for the record half only. This is the *"a diff falsifies text it never
touches"* class — it is not in any hunk.

**M-5. Round 10's M-2, M-3, M-4, M-6 and N-1 are all untouched.** Not claimed, so
not named-and-not-done; recorded so the count is honest. Line **351** still
carries *"the leaking cases are UNEXPECTED POSITIONALS, not declared flags"*
(M-2); `me bundle` is still a positive control with no observable and exits **2**
today (M-4); the unquoted-twelve-word-mnemonic direction is still unstated (M-5);
row 6 still names two BIP-39-word surfaces where the cross-product has three
(`bundle`, `help`, `sysw help` — N-1); and the donor's own post-parse
normalisation gap (M-6) is still unfiled — `grep -n 'post-parse'
design/FOLLOWUPS.md` returns **1 hit**, inside F-266's table, not an entry.

**M-6. The `FOLLOWUPS.md` F-266 splice from round-10 M-3 is unaddressed** and now
sits beside two new entries that are cleanly written, which makes it read as
deliberate.

---

## NIT

**N-1.** The fold's new §3 paragraph carries a splice of its own: *"…a promise
with nothing behind it (round-10 I-3). **The honest position: The classifier
scan** catches everything that *looks* like an artifact…"* — capital `T` after
the colon, with ragged wrapping (`then scheduled no / row, no condition and no /
follow-up for it`) marking where the old sentence was cut. Same shape as the
condition-9 splice this cycle has carried since round 7.

---

## QUESTION 2 — DISPOSITION OF ROUND 10's 2C/4I/6M/1N, AGAINST THE DIFF

`git diff -U0 ce5a206..e3b95cb` on the plan: **6 hunks**, new-file lines
**270-280, 290-294, 486, 491, 528, 580**. Line **527** is context, not changed. **"claimed?"** = named in `e3b95cb`'s
commit message.

| # | claimed? | disposition | the diff line that closes it, or why not |
| --- | --- | --- | --- |
| **C-1** — guard sited in the crate = cyclic dep | yes | **PARTIAL — 4 of 5 sites** | file table **486**/**491** rewritten to *"stream shaping only"* / *"the entire pre-parser argv guard"* stays in `me` ✔; new ruling paragraph at **270-274** ✔; row 6 unaffected ✔; row 9b now satisfiable **under the table** ✔. **Line 208 untouched** → **C-1 above**. Line **528** edited into a self-contradiction → **C-2 above**. The *"say what the crate contributes instead"* half not done → **I-2** |
| **C-2** — four-shape list survives in §3 | yes | **CLOSED but for a fragment** | **272-273** and **486** both now derive the set from `is_argv_forbidden()` ✔. `by wordlist` survives orphaned at **281** → **M-1** |
| **I-1** — `classify()` does not normalise | yes | **PARTIAL** | the requirement is in row 6's step column ✔; **no near-miss carrier in the gate**, and the cited warrant is false against `main.rs:1978` → **I-1 above** |
| **I-2** — gate asserts more than the design delivers | yes | **CLOSED** | gate headline narrowed to *"carrying it AS A TOKEN"* ✔; *"and M-3"* dropped from the settles claim ✔; residue filed as F-267 with an owning phase (`FOLLOWUPS.md:11832`) ✔ |
| **I-3** — flag-name layer owned by nothing | yes | **PARTIAL** | F-268 filed with an owning phase ✔, and §3 now says plainly *"FILED, not built"* ✔ — the ownership core **is** closed. The §7 sentence is absent, §3:299 is stale, and F-268's trigger is already met → **I-2 above** |
| **I-4** — override has no observable | yes | **PARTIAL** | the exit-0 assertion is in row 6's gate ✔ — the half that carried the harm. Scoping and the `bundle` row still absent → **M-2 above** |
| **M-1 … M-6, N-1** (round 10) | no | **NOT CLOSED** | all seven verbatim → **M-1, M-3, M-5, M-6, N-1 above** |

**Score: 0 of 2 Critical fully closed (C-1 at 4 of 5 sites, C-2 but for a
fragment), 1 of 4 Important closed, 0 of 6 Minor, 0 of 1 Nit.**

**No named-and-not-done, for a fourth round.** Every claim in `e3b95cb`'s message
is in its diff, and the message again leads with the author's own error. The one
claim that overstates is the gate line — *"fold-propagation 0 on this fold's own
retracted phrasings"* — true of the command run, false of the fold, for the
second round running (**M-1**).

---

## QUESTION 3 — CAN AN IMPLEMENTER EXECUTE ALL TWELVE ROWS?

§4 carries **12** step rows: `1, 2, 3, 4, 5, 6, 7, 8, 9, 9b, 10, 11` (counted:
`grep -c '^| [0-9]'` = 13, of which one is §2.1's `0600 — control` row). The diff
changed **1** table line (row 6). Rows 1-5 and 7-11 are unchanged from rounds 8-10
and are not re-derived.

| # | executable? | can its gate fail? | this round's note |
| --- | --- | --- | --- |
| 1 signature change | yes | yes | unchanged |
| 2 the move | yes | yes | still names `io.rs` (**M-3**) |
| 3 mask split | yes | yes | unchanged |
| 4 `observation.rs` + pty | yes | yes | unchanged |
| 5 `remedy.rs` | yes | yes | unchanged |
| **6 the argv guard** | yes | **partly** | materially better than round 10's. The normalisation requirement has **no row that can fail**, and its warrant is false (**I-1**); override scoping unstated (**M-2**) |
| 7 F-265 five-site digit pin | yes | yes | unchanged — still the model row |
| **8 `--expect`** | **ambiguous** | yes | the gate is sound; **where the kind vocabulary lives is now contradictory** — §3's tree says the crate, §3's table excludes it, and it needs `Class`/`Admission`/`chunk_key` (**C-1**) |
| 9 `exit.rs` + `channel.rs` | yes | yes | unchanged |
| **9b create the crate** | **ambiguous** | **yes under the table, no under the tree** | *"no `Class` in it"* passes for the five functions the table assigns (verified against the donor); line 208 assigns two more that make it RED (**C-1**) |
| 10 consume | yes | yes, as regression | unchanged |
| 11 publish | n/a | n/a | operator-gated |

**Ten of twelve rows are unambiguously executable with a gate that can fail** —
up from ten with one blocked. Rows 8 and 9b now hinge on the same single line.

---

## WHAT I VERIFIED HERE

Absolute paths throughout. Nothing re-derived that the brief listed as
machine-checked; `fold-propagation-check.sh` was re-run because the question was
*which patterns*, not *does it pass*.

| check | result |
| --- | --- |
| `fold-propagation-check.sh` with round 10's four patterns + `'the argv gate'` + `'P0 builds both'` | **rc 1**, 2 hits: **208**, **281** |
| `grep -n 'the argv gate'` over the plan | **1 hit — line 208** |
| `grep -n 'kind vocabulary'` over the plan | **1 hit — line 208**, the only file assignment anywhere |
| `grep -o '\*\*' \| wc -l` at `e3b95cb` / at `ce5a206` | **681 (odd)** / **662 (even)** — one unmatched delimiter introduced, at line 528 |
| `git show e3b95cb --stat` / `git diff -U0` hunks | 1 file, **20 insertions, 8 deletions** / **6 hunks**: 270-280, 290-294, 486, 491, 528, 580 |
| `Class` definition | `crates/me-cli/src/sysw/record.rs:44` — crate `mnemonic-engrave` |
| `Admission` | `crates/me-cli/src/sysw/mod.rs:165` |
| `mdmk_unconfirmed` / `mt_unconfirmed` / `chunk_key` | `sysw/record.rs:168` / `sysw/mt.rs:207` / `seal/record.rs:247` (`pub(crate)`) |
| `split_record_stream` / `no_records_guard` — do they name `Class`? | `main.rs:1867` / `:1896` — **neither does**; row 9b's gate passes for them |
| `destination` / `write_block` / `stdout_world_readable_mode` — `Class`? | `main.rs:940` / `:971` / `:896`,`:921` — **none does** |
| donor post-parse gate: what is normalised | `trimmed` at **`main.rs:1952`** feeds `by_prefix` at **`:1958`**; `classify` receives **raw `r`** at **`:1978`** |
| `classify` normalisation | none — `record.starts_with(TX_PREFIX)`, `sysw/mod.rs:184` |
| F-267 / F-268 filed with owning phases | `FOLLOWUPS.md:11832` (**post-P0**) / `:11853` (**after P0 / "whichever cycle first adds a secret-bearing flag"**) |
| does a secret-bearing flag exist today? | **yes** — `mnemonic restore --passphrase <PASSPHRASE>` and `--passphrase-stdin`, run live from `/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic`; declared `crates/mnemonic-toolkit/src/cmd/slip39.rs:101`; `flag_is_secret` at `src/secrets.rs:60-68` also lists `--bip38-passphrase`, `--decrypt-password` |
| spec §7 P0's row assigns to *the shared crate* | `--in`/`--out`/`-`, **argv guard with pre-parser ordering**, write gate, exit codes, remedy text, **`--expect` in full — the kind vocabulary** |
| plan §7 mentions of the flag-name layer / the argv guard / `--expect` | **0 / 0 / 0** |
| round-10 M-6 (donor post-parse gap) filed? | **no** — `grep -n 'post-parse' FOLLOWUPS.md` → 1 hit, inside F-266's table |
| five carried round-9 Minors | **present, 1 each** (M-10 line-wrapped) |
| §4 step-row count | **12** (`1 2 3 4 5 6 7 8 9 9b 10 11`) |

---

## WHAT THE FOLD GOT RIGHT

Recorded so round 12 does not re-open it.

- **The ruling is correct and it is the small answer.** The guard calls
  `classify()`; `me` depends on the crate; therefore the guard is `me`'s. That
  follows from a build rule, not a preference, and choosing it over the
  predicate-seam alternative was the right call for P0 — a seam that exists to
  host one caller is a design looking for a second consumer it does not have.
  **Nothing in this report argues for reversing it.**
- **The file table is now honest.** *"`split_record_stream`, `no_records_guard` —
  stream shaping only"* is exactly what the crate can hold, and the *stays in
  `me`* row carries the reproduced error string rather than a claim.
- **I-2 is closed cleanly and by the right mechanism.** *"carrying it AS A
  TOKEN"* is the narrower claim and the true one; F-267 records the residue with
  a phase instead of papering it. Taking the smaller true statement over the
  larger promised one is the move this cycle has had to learn four times.
- **I-4's expensive half is closed.** The exit-0 assertion moves a regression
  catch from row 10 to row 6, which is what round 10 filed it for.
- **F-268 is a real filing, not a shrug** — it states plainly that P0 *declines*
  to close a gap rather than that no gap exists, and it says why the value scan
  does not dominate the flag layer. The only thing wrong with it is the trigger
  date.
- **Named-and-not-done stayed closed for a fourth round.**

---

**VERDICT: NOT GREEN — 2 Critical, 2 Important.** No code may be written against
this plan.

**The one sentence for round 12.** The ruling is settled and four of C-1's five
sites took it — what is left is a **one-line fold that reaches §3's file tree**,
because line 208 is now the only place in the plan that assigns *anything* to a
file, and it assigns two things to the crate that need `Class`: the argv gate,
which round 10 already proved is a cyclic dependency, and the `--expect` kind
vocabulary, which nobody has checked and which needs `Admission` and `chunk_key`
as well. **Fix line 208, un-break the sentence at 527-528, give row 6's
normalisation a carrier that can fail, and say in §7 what the crate contributes
to the argv and `--expect` work — then this plan is done.**
