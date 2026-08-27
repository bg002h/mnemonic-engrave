# R0 closing round — `SPEC_constellation_cli_uniformity.md` @ `ceec3b7`

**The one question:** is the spec now sufficient and accurate enough that an
implementation plan written from it would be correct?

**Scope, as briefed:** the four fold commits (`0efc5d4`, `4c6ea7f`, `fe2cf1e`,
plus the two architect-ruling commits they carry), a narrow re-run of the
constructive test from `R0-cli-uniformity-plan-draft.md` Part C, and execution of
every command the document contains. Not a fresh audit.

**VERDICT: NOT GREEN — 0 Critical / 4 Important / 4 Minor.**

The three Criticals are genuinely closed and I could not construct a reading that
reopens any of them. What is not closed is **three of the five Importants from the
same constructive round, which were never folded at all**, plus one that was
folded to one fifth of its extent while its commit message reported it done.

Everything below was measured at `ceec3b7` against the built binaries
(`*/target/debug/*`, not the stale `~/.cargo/bin` copies) during this round.

---

# PART A — are C-1, C-2, C-3, §5a and D7 closed?

| item | closed? | one line of evidence |
| --- | --- | --- |
| **C-1** | **YES** | §6g:837 binds `transaction` to `Class::Mt` OR `Class::Tx`. Premise executed: `mt encode --in even.hex` emits 6 `mt1` lines; `mt encode --qr --in even.hex` emits a `tx:`-prefixed record; both are separate variants of `Class` (`crates/me-cli/src/sysw/record.rs`, one `MdMk` variant, distinct `Mt` and `Tx`). The plan-draft's blocked `kind_of` row is now writable. |
| **C-2** | **YES** | §6d:473-504 specifies two layers, both pre-parser. Every mechanical claim executed: `command_line_guard` at `mt-cli/src/validate.rs:448`, `looks_like_a_transaction` at `:503`, the guard on `std::env::args()` *before* `Cli::parse()` at `mt-cli/src/main.rs:219-238`, `me`'s `read_records` reached post-parse at `me-cli/src/main.rs:1127`, and the clap echo reproduced verbatim — `mt encode --qr deadbeefcafe` → `error: invalid value 'deadbeefcafe' for '[-]'`, exit **2**. |
| **C-3** | **YES** | §7:943-950 keys P3 off the existing predicate. Both cited artifacts exist: `NodeType::is_argv_secret_bearing` at `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/convert.rs:117`, and the lockstep test `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` named at `secret_taxonomy.rs:92`. (Its command misreports its own count — Minor B-2 below.) |
| **§5a** | **PARTLY** | Home and distribution are decided, and every hard constraint verified: `wc-codec = { path = "../wc-codec" }` at `crates/mnemonic-toolkit/Cargo.toml:37`, no version; `mt-cli` has exactly three dependencies (`mt-codec`, `bitcoin`, `clap`); the workspace `[patch.crates-io]` miniscript git-rev pin at the toolkit root `Cargo.toml:34-35`. **Still unanswerable: the crate's NAME, and its consumer set** (Minor C-d). |
| **D7** | **YES** | §9a states the boundary, reasons the in-scope table per row rather than collectively, and names what follows from the deferral including whether §10 keeps its shape. Consistent with §6g and P0's content. `spec-structure-check` → STRUCTURE OK (24 sections, 17 cross-refs); `plan-table-check` → 67 rows, 0 malformed. |

**A note in C-1's favour that no round has recorded.** The `Mt | Tx` union is
*robust to the relocation `ARCH-qr-record-placement.md` defers*: if the `tx:`
record later moves out of `mt encode --qr` into `me sysw pack`, a union over both
classes still matches, where either single binding would have had to be revised.

---

# PART B — defects introduced or left standing by these folds

## B-1 (Important) — the propagation fold closed 1 of 5 sites, and its commit message reports it done

`fe2cf1e`'s message ends *"fold-propagation → both superseded phrasings gone"*.
The constructive round (A-1) named **five** superseded sites. Run at `ceec3b7`
with that round's own five phrasings:

```
$ ./scripts/fold-propagation-check.sh design/SPEC_constellation_cli_uniformity.md \
    'not .combine.' 'positionals ONLY' 'ordering constraint that makes P2 non-negotiable' \
    'real .-. gap' 'confined to the two rulings'
  LEFT   not .combine.                                     61
  LEFT   positionals ONLY                                 542
  LEFT   ordering constraint that makes P2 non-negotiable 545
  LEFT   real .-. gap                                     351
  gone   confined to the two rulings
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.        (exit 1)
```

`git show fe2cf1e -- design/SPEC_constellation_cli_uniformity.md | grep '^-'` shows
exactly one of the five removed. The gate was run against two phrasings, not the
reviewer's five, and reported clean — **a negative inherits the scope you gave
it**.

**The underlying claim is false, verified independently this round:**

```
$ head -2 shares.txt > two.txt
$ ms combine - < two.txt ; echo $?
entropy: 00000000000000000000000000000000
phrase: abandon abandon abandon ... about
0
```

`ms combine -` reads shares from stdin and recovers the secret at exit 0. The
four survivors:

- **§2:61** — *"7 of 8 verbs — **not `combine`**"*. False.
- **§6b:351** — *"the real `-` gap is `md`'s four other verbs and `ms combine` —
  two targeted additions"*. False twice: `combine` is not a gap, and `mt`'s
  `decode`/`verify`/`inspect` are — work P1's row now owns.
- **§6d:542** — the normative per-verb channel table: `` `combine` | **positionals
  ONLY — no `--in`, no `-`** ``. **This is the table a plan author enumerates P2's
  `ms` work from.** Half-false: `--in` is genuinely absent on all eight verbs
  (verified: `ms encode --in x` → `error: unexpected argument '--in' found`), but
  `-` ships.
- **§6d:545** — *"`ms combine` is the ordering constraint that makes P2
  non-negotiable … Refusing argv there before `-` and `--in` exist removes the
  **only** way to recombine split shares — the recovery path"*. This is the
  **stated reason P2's ordering is non-negotiable**, and it rests on a fact that
  is false. The ordering may still be right on the `--in` half; the argument
  written for it is not the one that supports it.

**Cost:** an implementer building P2 from §6d:542 writes a stdin channel that
already ships, and one reading the `-` scope from §6b:351 builds `md`×4 + `ms
combine` instead of `md`×4 + `mt`×3. Recoverable in-phase — Important, not
Critical, unchanged from the round that first raised it.

## B-2 (Minor) — §7's new C-3 command does not produce §7's new C-3 numbers

`fe2cf1e` added, as the fix for *"a number without its command"*:

```
grep -rl 'secret_in_argv_warning' --include='*.rs' /scratch/code/shibboleth/mnemonic-toolkit
  -> 26 files, 86 references
```

Run verbatim at `ceec3b7`: **48 files.** `grep -r` descends into `target/`, which
holds 22 of the 48 hits. And `grep -rl` prints filenames only — it cannot produce
"86 references" under any input; that needs `-ro`.

Restricted to tracked source the figures are exactly right:

```
$ git ls-files '*.rs' | xargs grep -l  secret_in_argv_warning | wc -l   -> 26
$ git ls-files '*.rs' | xargs grep -o  secret_in_argv_warning | wc -l   -> 86
```

So the **fact is sound and no decision moves** — the point is that both numbers
are far above five, and they are. The defect is that the command offered *so the
number can be re-derived rather than believed* re-derives a different number.
Same class §7 itself names three paragraphs later about the `chunk-set-id` sweep.

## B-3 (Minor) — §5a decided the distribution mechanism and left §7 P0 instructing the plan to decide it

`fe2cf1e` added §5a (line 251) ruling crates.io, `0.1.0`, published when P0 closes
GREEN. §7's paragraph four sections below is untouched:

- **:981** header — *"**P0 — the distribution mechanism, which no earlier draft
  named (I-5).**"* §5a names it.
- **:995** body — *"**P0 must name which mechanism it uses.**"* §5a already has.
- **:995** same sentence — *"shipping a change to **four consumers**"*, against
  D5:228's *"depended on by all five"* and §5a:275's *"the toolkit becomes the
  **fifth consumer**"*.

The two sections agree on *direction* (both argue against the rev pin), so a plan
author lands in the right place; they disagree on whether the decision is open and
on how many consumers there are. This is the fold-falsifies-text-elsewhere shape
the cycle keeps hitting, at four sections' distance rather than one paragraph's —
and `fold-propagation-check.sh` finds it in a second given the phrasing.

## B-4 (Minor) — §8a's self-referential sigil count is stale, and the folds are what staled it

§8a asserts *"Today it reports **16 distinct sigils over 50 occurrences**, and each
was read."* Run at `ceec3b7`: **20 distinct over 62 occurrences**. The folds added
12 occurrences, including the new `§9a` and `§8a` sigils, after the "each was
read" claim was written. All 20 resolve (`spec-structure-check` → STRUCTURE OK,
17 cross-refs), so nothing is pointing at the wrong target — but §7 declines to
pin exactly this kind of count on exactly this reasoning, and §8a pins one.

---

# PART C — what a plan author still cannot answer

Re-running the constructive test narrowly against Part C of
`R0-cli-uniformity-plan-draft.md`:

| Part C question | now answerable? |
| --- | --- |
| crate **location** | **YES** — §5a: workspace member of `mnemonic-engrave`, then crates.io `0.1.0` at P0 GREEN, publication operator-gated, recorded as symmetry debt with a non-breaking reversal path |
| **distribution** mechanism | **YES** — §5a: crates.io version deps (was `[GUESS by elimination]`) |
| crate **name** | **NO** — no name anywhere in 1,307 lines (C-d) |
| how **five consumers** depend on it | **PARTLY** — 4 vs 5 contradiction (B-3), and `me`'s status never stated (C-d) |
| public **API** | **MOSTLY** — `command_line_guard`'s detector signature is now writable (C-2); `write_private`'s clobber policy is still unruled (C-a) |
| what each item is **extracted from** | **MOSTLY** — §5a/§6b/§6d give provenance per item; the purge text is still *"FROM `mt`/`me`"* where the two conflict (C-b) |
| the **test list** | **MOSTLY** — T5/T6 (detector) and T9 (`--expect transaction`) unblocked; T11 blocked on a §6f ruling (C-c); T12 blocked and acknowledged as such by §6h |

## C-a (Important, carried from the last round, never folded) — `--out` clobber is unruled, on the channel for material that cannot be regenerated

§6b:356 is the whole ruling: *"`--out FILE` — write the artifact to a file,
**created 0600 by `me`'s `write_private`**, never `std::fs::write` (F-244)."*

`me-cli/src/main.rs:856-876`, read this round:

```rust
opts.write(true).create(true).truncate(true);
```

`.truncate(true)` silently destroys an existing target, and its doc comment
accepts that with a justification scoped to `me`'s own targets. Grepped for a
ruling under every wording I could think of —
`clobber|truncate|overwrit|destroy|existing file|already exists|create_new` — the
spec contains **zero** occurrences of any of them.

An implementer following §6b literally lifts the function verbatim, and
`ms encode --out seed.ms1` overwrites a seed backup without asking. This is the
same shape §6e already caught once and retracted for the terminal gate: *"`me`'s
refusal states a reason that is specific to a binary container … the predicate is
false for all four CLIs."* The clobber acceptance is specific in exactly that way
and was lifted anyway.

**What closes it:** one sentence in §6b ruling clobber — refuse, or `--force`, or
explicitly inherit `me`'s behaviour with the reason restated for a non-container
target — and a P0 test.

## C-b (Important, carried, never folded) — which purge text, and nothing catches the wrong one

§7:938 P0 still reads *"remedy text per §6h … Extracted FROM `mt`/`me`"*. §6h
names `me` (*"The reference implementation is `me sysw pack`'s widened argv
refusal"*), and §6h:886 forbids by name the trap `mt` ships: *"Do not tell a zsh
user to run the history builtin with `-d` … `-d` is a **display** flag."*
`mt-cli/src/validate.rs:541-548` advises `history -d $HISTCMD` for zsh and anchors
its fish branch on the material.

The spec never states that `mt`'s shipped text is non-conforming, never assigns
its replacement to P1, and no gate in any phase asserts purge text at all. `mt`'s
is the self-contained `fn() -> &'static str` — the obvious thing to lift.

Verified that §6h's own recipe runs: with `HISTFILE` set,
`sed -i '/me sysw pack/d' "$HISTFILE"` deletes the matching entry and leaves the
others, exit 0.

**What closes it:** §7 P0 says *"remedy text from `me` — `mt`'s is superseded and
P1 replaces it"*, plus one P0 test asserting the zsh branch does not advise
`history -d`.

## C-c (Important, carried, never folded) — filling P0's `mnemonic` exit cells falsifies §6f, and the cell has two defensible answers

P0's gate orders *"the two `mnemonic` exit cells still marked 'not measured'
filled"*. Filled this round:

```
$ mnemonic inspect notanartifact                 -> 2
$ mnemonic repair md1zzzzzzzz8xtwhw4xwn4qh       -> 2
  (against: md 1, ms 1, mk 2)
```

`mnemonic` diverges exactly the way `mk` does. §6f:749 rules *"**`mk`'s
invalid-artifact 2 becomes 1** … **This is the only code this cycle changes**"*,
§9 lists no `mnemonic` exit-code work, and no phase owns the consequence. A gate
item whose likely answer contradicts the section that ordered it, with no
response specified.

And the cell is operationally undefined:

```
$ mnemonic convert --from ms1=notanartifact      -> 64
```

2 or 64 depending on which verb "invalid artifact" means, and §6f never defines it
per CLI.

**What closes it:** §6f names the verb each cell is measured with, and states in
advance what happens if `mnemonic`'s number matches `mk`'s — converge it under the
same ruling, or record it as a second reasoned divergence.

## C-d (Minor) — no crate name; consumer set stated three ways; `me`'s own status unstated

- **Name.** None proposed anywhere. A plan author can invent one, but it is baked
  into five `Cargo.toml`s, every `use`, and an irreversible registry publish.
- **Count.** D5:228 "all five" / §5a:275 "fifth consumer" / §7:995 "four
  consumers" (B-3).
- **`me`.** §5a rules the toolkit is a consumer and never says whether `me` is
  one. If `me` is a donor only, `write_private` and `is_argv_forbidden` exist in
  two copies the day the crate ships — the condition D5 exists to prevent. Two
  facts sharpen it: §6d rules the override's *own parse* must run on raw argv,
  and `me` ships it as an ordinary clap flag (`me-cli/src/main.rs:252`,
  `#[arg(long)] allow_argv_secret`), with no phase owning the fix; and §2a still
  does not name `me` as an affected CLI (the constructive round's A-3, also
  unfolded).

---

# Commands executed — every command the document contains

All of these **ran and matched** the spec:

| §  | command | spec says | measured |
| --- | --- | --- | --- |
| 1 | `mt encode --qr "$(cat even.hex)"` | refused, exit 1, no echo | exit **1**, *"a transaction was passed as a command-line argument (444 characters)"* |
| 2 | `md encode 'wpkh(@0/<0;1>/*)'` | no header | no `chunk-set-id:`, grouped by 5 |
| 2 | `mk encode --xpub … --origin-path … --policy-id-stub 11223344` | 2 `mk1` lines, no header ever | 2 lines, **0** header lines |
| 2/6f | `mk decode notanartifact` / `md` / `ms` | 2 / 1 / 1 | **2 / 1 / 1** |
| 3 | `ms encode --phrase <all-abandon>` → `me sysw pack --no-passphrase` | exit 4 | exit **4**, *"not a form this container can place"* |
| 3 | same, `--separator hyphen` | exit 4 | exit **4** |
| 3 | same, `--group-size 0` | exit 0, 102 B | exit **0**, **102** B |
| 3 | same without `--no-passphrase` | 118 sealed | **118** B |
| 6c | hyphen round-trip, each tool fed its own output | md/mk/ms exit 0 | **0 / 0 / 0**; `comma` offered by all three |
| 6e | `mt encode --in even.hex` (report) | report ends stderr line 69; destination refusal at 105 | **69** (`SUGGESTED LEGEND`), refusal at **105** |
| 6e | `mt encode --quiet` | `TX`/`CUT`/`PREFIX` suppressed | **0** hits quiet vs **3** plain |
| 6f | `md repair md1yqpqqzqq8xtwhw4xwn4qh` | exit 5 | exit **5**, `md1yqpqqxqq8xtwhw4xwn4qh` |
| 6f | `mnemonic repair` same | exit 4, identical correction, UNVERIFIED banner | exit **4**, identical string, banner present |
| 6g | `md encode --group-size 0 --from-policy 'pk(@0)' --context segwitv0 --key '@0=<xpub>'` | 2 `md1` strings | **2** |
| 6g | `md encode 'pk(@0)'` | refused, *unsupported descriptor wrapper* | exit **1**, that wording |
| 6g | `mt encode --in <even raw_hex>` | 6 `mt1` strings | **6** |
| 6g | C-1 reproduction, valid vs junk `mt` input | both exit 0, payloads differ | both exit **0**, **550 B** vs **102 B** |
| 6g | 1 of a 2-chunk `md1`/`mk1` set into `pack` | quoted refusal line, writes payload, exit 0 | the quoted line verbatim, **133 B** written, exit **0** |
| 6h | `sed -i '/me sysw pack/d' "$HISTFILE"` | verified working | deletes the entry, leaves the others, exit 0 |
| 7 | `git ls-files design/journeys \| xargs grep -l 'chunk-set-id:'` | 7 tracked | **7**; `git ls-files design/journeys/out` → **0** |
| 7 | `git ls-files design \| xargs grep -l 'chunk-set-id:' \| wc -l` | deliberately unpinned | **33** (28/29/30 historically — the section's point stands) |
| 8a | both structure gates | clean | STRUCTURE OK (24 sections, 17 cross-refs); 67 rows, 0 malformed |
| 10 | `--expect`, `--from-md1-set`, `ms/md --in` | do not exist yet | all four → `unexpected argument` (target state, as intended) |

**Three did not reproduce as written**, all reported above: the §7 `grep -rl`
(B-2), the §8a sigil count (B-4), and §6e's `70 / 108` stderr-line pair — that
last one reproduces its *conclusion* exactly (the `--quiet` suppression, and lines
69 and 105 both land on the nose) but the absolute totals depend on the input
file's mode and the terminal width, neither of which the parenthetical states.
I measured 120/82 with a 0644 input, 105/67 with 0600, 66/104 to `/dev/null`,
60/98 on a pty. The delta is a stable **38** in every condition. Not counted as a
finding: it is pre-existing §6e text, no fold touched it, and no conclusion moves.

**One thing I checked and found is NOT a defect**, recorded so the next round does
not spend a lens on it: P3 makes `mnemonic` refuse argv on five channels, which
would repeat the `ms combine` hazard if those channels had no private
alternative. They all do — `bundle`, `convert`, `derive-child` and `restore` each
ship `--passphrase-stdin`, `electrum-decrypt` ships `--decrypt-password-stdin`,
and `@env:` values are honoured (`cmd/addresses.rs:126`). No ordering constraint
is needed for P3.

---

# Counts

| severity | items |
| --- | --- |
| **Critical (0)** | — |
| **Important (4)** | B-1 propagation fold closed 1 of 5 sites and its message reports it done; C-a `--out` clobber unruled; C-b which purge text, with no gate; C-c the `mnemonic` exit cells falsify §6f when filled |
| **Minor (4)** | B-2 the C-3 grep prints 48 not 26; B-3 §5a decided what §7 P0 still tells the plan to decide (4 vs 5 consumers); B-4 §8a's sigil count is stale at 20/62; C-d no crate name, `me`'s consumer status unstated |
| **Nit (0)** | — |

**VERDICT: NOT GREEN (0C / 4I).**

## What would close it

Four sentences and one gate item — none of them a design question:

1. **Run `fold-propagation-check.sh` with the constructive round's own five
   phrasings** and fix the four survivors: §2:61, §6b:351, §6d:542, §6d:545.
   `ms combine -` works; the `-` gap is `md`×4 + `mt`×3.
2. **§6b: rule `--out` clobber.** Refuse / `--force` / inherit-with-reason.
3. **§7 P0: remedy text from `me`; `mt`'s is superseded and P1 replaces it** —
   plus one P0 test on the zsh branch.
4. **§6f: name the verb each `mnemonic` cell is measured with, and rule in advance
   what happens when the number comes back 2.**

The Minors are one-line edits and do not gate.

**On whether another round is warranted after that fold.** Items 1–4 are
mechanical corrections to settled rulings, not new logic. Under the proportional
re-review rule they need a *verification* pass — did the fold fix each of the four,
and did it introduce a new one — not a fresh design round. The design questions
this cycle opened are closed: C-1, C-2, C-3, §5a's mechanism and D7's boundary all
survived a constructive re-test, and I could not find a sixth question to ask of
them.
