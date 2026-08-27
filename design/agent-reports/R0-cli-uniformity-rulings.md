# R0 — the operator-ruling sections of `SPEC_constellation_cli_uniformity.md`

**Artifact:** `design/SPEC_constellation_cli_uniformity.md` at `9c6214a`
(worktree `/scratch/code/shibboleth/_work/r0r/mnemonic-engrave`, branch
`review/r0-rulings`).

**Object — exactly four commits, nothing else:**

| sha | section |
| --- | --- |
| `39fcac3` | §5b — the four verbs live in every `m*-cli` binary |
| `3293210` | §5c — what moves to the toolkit |
| `f06774a` | the "could it be a codec?" measurement |
| `90c74ee` + `9c6214a` | the crate name: `mnemonic-io-lib` |

Everything else in the document was reviewed and closed across seven prior
rounds and was **not** re-opened. `repair` staying / D26's parity set / the
exit-code work were taken as settled and not re-derived. `plan-glyph-check.sh`
exiting 1 (F-257) is out of domain and is not reported.

**Method.** Every binary invoked by absolute path, stdin from `/dev/null`, exit
codes read directly and never through a pipe.

---

## VERDICT

**NOT GREEN — 0 Critical / 4 Important / 5 Minor / 1 Nit.**

The rulings themselves are recorded clearly and the verb classification is
**exhaustive** — every one of the 33 subcommands on `md`, `mk` and `ms` is
accounted for (verified below). What fails is the surrounding record: one
measured claim is **false**, one recorded command **cannot produce its recorded
output** (and is the pre-publish gate), and three "NOT settled" / "already lives
where it belongs" statements elsewhere in the document were falsified by these
commits and left standing.

---

## PART 1 — MEASUREMENTS RE-RUN

### 1.1 The four verbs on the four encoders — 16/16 ✅ MATCHES

```
for bin in md mk ms mt; for v in encode decode verify inspect; <abs-path> $v --help >/dev/null </dev/null; echo $?
```

| binary | encode | decode | verify | inspect |
| --- | --- | --- | --- | --- |
| `md` | 0 | 0 | 0 | 0 |
| `mk` | 0 | 0 | 0 | 0 |
| `ms` | 0 | 0 | 0 | 0 |
| `mt` | 0 | 0 | 0 | 0 |

`TOTAL pass=16 fail=0`. §5b's claim and P0's gate line
(*"16 checks; verified passing 2026-08-26"*) are **correct**.

### 1.2 §5b's "verbs beyond the four" table vs actual `--help` — ✅ MATCHES, 4/4 rows

Measured `<abs-path> --help </dev/null`, subtracting the four verbs and clap's
`help`:

| binary | spec says | actually present | verdict |
| --- | --- | --- | --- |
| `mt` | none | *(none)* | ✅ |
| `md` | `bytecode` `vectors` `compile` `descriptor` `address` `repair` `gui-schema` `gen-man` (8) | same 8 | ✅ |
| `mk` | `vectors` `address` `derive` `repair` `gui-schema` `gen-man` (6) | same 6 | ✅ |
| `ms` | `derive` `split` `combine` `vectors` `repair` `gui-schema` `gen-man` (7) | same 7 | ✅ |

### 1.3 §5c's table — the "in" column — ✅ MATCHES, 6/6 rows

`split`→`ms` ✅, `combine`→`ms` ✅, `compile`→`md` ✅ (only `md` has it),
`derive`→`mk`,`ms` ✅ (`md` has none), `address`→`md`,`mk` ✅ (`ms` has none),
`repair`→`md`,`mk`,`ms` ✅.

**Which binary each verb leaves is unambiguous.** Question 1's first half passes.

### 1.4 The classification is EXHAUSTIVE — ✅ (verified; the spec does not say so)

Every verb on every encoder falls in exactly one class:

- `md` (12): 4 uniform + `bytecode` STAYS + `compile`/`descriptor`/`address` →toolkit + `repair` STAYS + `vectors`/`gui-schema`/`gen-man` structural.
- `mk` (10): 4 uniform + `address`/`derive` →toolkit + `repair` STAYS + `vectors`/`gui-schema`/`gen-man` structural.
- `ms` (11): 4 uniform + `split`/`combine`/`derive` →toolkit + `repair` STAYS + `vectors`/`gui-schema`/`gen-man` structural.
- `mt` (4): the four.

Nothing is unclassified, so *"and stuff like that"* has no residue to cover.
The spec never states this (see M-5).

### 1.5 Codec purity — ✅ MATCHES exactly

```
grep -rlE 'use std::io|std::fs|clap|stdin|stdout' <crate>/src | wc -l
```

| crate | spec | measured | hit |
| --- | --- | --- | --- |
| `mt-codec` | 0 | **0** | — |
| `md-codec` | 0 | **0** | — |
| `ms-codec` | 0 | **0** | — |
| `mk-codec` | 1 | **1** | `mk-codec/src/bin/gen_mk_vectors.rs` |

`wc-codec` also measures 0 (not claimed, consistent). The single `mk-codec` hit
is exactly the file the spec names, and it is a `src/bin/` maintainer tool, not
the library. ✅

### 1.6 The lib/bin partition — ✅ MATCHES as stated (see M-4 for the denominator)

| crate | `src/lib.rs` | `src/main.rs` | `src/bin/` |
| --- | --- | --- | --- |
| `md-codec` | yes | no | — |
| `mk-codec` | yes | no | `gen_mk_vectors.rs` |
| `ms-codec` | yes | no | — |
| `mt-codec` | yes | no | — |
| `wc-codec` | yes | no | — |
| `md-cli` | no | yes | — |
| `mk-cli` | no | yes | — |
| `ms-cli` | no | yes | — |
| `mt-cli` | no | yes | — |
| **`me-cli`** | **yes** | **yes** | — |

5 of 5 `-codec` = lib, no main ✅. 4 of 4 encoder `-cli` = main, no lib ✅.
`me-cli` has both ✅.

### 1.7 Every `mnemonic-*` package produces an executable — ✅ first sentence, ❌ second

`cargo install --list` reproduces the spec's block **verbatim, 7/7 rows**:

```
md-cli v0.13.0 → md          mk-cli v0.13.0 → mk
ms-cli v0.16.0 → ms          mt-cli v0.1.0  → mt
mnemonic-engrave v0.7.0 → me
mnemonic-gui v0.59.0 → gui-render, mnemonic-gui
mnemonic-toolkit v0.97.0 → mnemonic
```

"Every `mnemonic-*` package produces an executable — 3 of 3" ✅.
"There is no `mnemonic-*` library" ❌ — **see I-1.**

### 1.8 The crates.io check — conclusion ✅ TRUE, recorded command ❌ CANNOT PRODUCE IT

See **I-2**. Short form: the literal recorded command returns **403** today, for
every crate name including `serde`. With a user-agent, `mnemonic-io-lib` → 404
and `mnemonic_io_lib` → 404, so **the name is genuinely free** and the ruling
stands.

### 1.9 `md descriptor` / `md bytecode` quotes — ✅ EXACT

Both quotes match the shipped `--help` text verbatim (the `…` in the
`descriptor` quote elides *"real xpubs, key origins and the BIP-380 checksum"*).
Quoted from the binary, as the commit message claims.

### 1.10 The two structure gates at `9c6214a` — ✅ as briefed

`./scripts/spec-structure-check.sh` → **exit 0**, `sections: 26 ; cross-refs
checked: 20 ; STRUCTURE OK`.
`./scripts/plan-table-check.sh` → **exit 0**, `table rows checked: 82 ;
malformed: 0`.

---

## PART 2 — FINDINGS

## CRITICAL — none.

---

## IMPORTANT

### I-1 — "There is no `mnemonic-*` library" is FALSE, and the cited command cannot prove a negative

**Site:** §5a, *The name: `mnemonic-io-lib`*, lines 351-353:
*"**Every `mnemonic-\*` package produces an executable — 3 of 3. There is no
`mnemonic-\*` library.**"*

**What is wrong.** All three `mnemonic-*` packages ship a **library target**:

| package | `src/lib.rs` | lib target |
| --- | --- | --- |
| `mnemonic-engrave` (`crates/me-cli`) | yes | **explicit `[lib] name = "mnemonic_engrave" path = "src/lib.rs"`** (Cargo.toml:17-19) |
| `mnemonic-toolkit` | yes | auto-discovered; `documentation = "https://docs.rs/mnemonic-toolkit"` |
| `mnemonic-gui` | yes | auto-discovered |

`mnemonic-toolkit/crates/mnemonic-toolkit/src/lib.rs` opens with
*"`mnemonic-toolkit` library surface … See `design/SPEC_secret_memory_hygiene_v0_9_B.md`
§4 P2 for the locked crate-shape decision (**Option C: hybrid lib + bin**)"* —
so a `mnemonic-*` library is not an accident, it is a recorded design decision
in a sibling spec.

And it is **published and rendered as a library**:

```
$ curl -s -o /dev/null -w '%{http_code}' -A '<ua>' https://crates.io/api/v1/crates/mnemonic-engrave
200            # max_version 0.4.0
$ curl -s -o /dev/null -w '%{http_code}' -L -A '<ua>' https://docs.rs/mnemonic-engrave/latest/mnemonic_engrave/
200
```

A reader of *this* tree who goes looking for a `mnemonic-*` library finds one on
docs.rs, at the exact `mnemonic_engrave` URL.

**The method is also circular.** The evidence offered is `cargo install --list`,
which enumerates **installed binaries by construction**. A library can never
appear in it, whether or not one exists — so that output cannot support the
negative claim it is placed under.

**Why it is Important, not Minor.** This sentence is the *entire* stated basis
for the `-lib` suffix, and the suffix goes into six manifests, every `use` site,
and an irreversible crates.io publish. The document's standing rule is that a
measured claim must be produced by the command shown.

**Not a challenge to the ruling.** `mnemonic-io-lib` is the operator's name and
the decision is untouched — the corrected fact still supports it, because no
`mnemonic-*` package is **library-only**; all three ship a binary, so the prefix
does read as "a program you can run".

**What closes it.** Replace the second sentence with what is true and provable,
e.g. *"No `mnemonic-*` package is library-ONLY — all three ship a binary
(`mnemonic-toolkit` and `mnemonic-engrave` ship a lib target as well;
`mnemonic-engrave` renders at docs.rs/mnemonic_engrave). The prefix therefore
reads as 'a program you can run'."* And either drop the `cargo install --list`
block from under the negative claim or re-label it as evidence for the first
sentence only.

---

### I-2 — the crates.io availability command returns **403**, not 404 — and P0 is told to re-run it before an irreversible publish

**Site:** §5a, *The name*, lines 364-370 (added by `9c6214a`), and the sentence
five lines later instructing P0 to re-check.

**What is wrong.** The command is recorded as:

```
$ curl -s -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/mnemonic-io-lib
  404
```

Re-run today, byte-for-byte:

```
$ curl -s -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/mnemonic-io-lib   -> 403
$ curl -s -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/mnemonic_io_lib   -> 403
$ curl -s -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/serde             -> 403
```

crates.io rejects a request with no user-agent. **`serde` — a name that
certainly exists — returns the same 403 as the free name.** The recorded command
therefore returns an identical answer for "free" and "taken", and there is no
`~/.curlrc` on this machine supplying a default UA.

With a user-agent the check works and the **conclusion is confirmed**:

```
$ curl -s -o /dev/null -w '%{http_code}' -A '<ua>' .../crates/serde            -> 200
$ curl -s -o /dev/null -w '%{http_code}' -A '<ua>' .../crates/mnemonic-io-lib  -> 404
$ curl -s -o /dev/null -w '%{http_code}' -A '<ua>' .../crates/mnemonic_io_lib  -> 404
```

**Why it is Important.** The spec does not merely cite this command as history —
it makes it a **gate**: *"P0 re-checks immediately before the irreversible step
rather than trusting this line."* A pre-publish gate whose command cannot
distinguish its two outcomes is a hypothesis, not a gate; an operator running it
gets 403 and must guess. This is the *"a true number beside a command that
cannot produce it"* class, and this instance sits directly in front of a publish
that cannot be undone.

**What closes it.** Record the command with an explicit user-agent (or
`cargo publish --dry-run` / `cargo search`), keep the 404s, and add the one line
that makes the gate self-checking: *a bare `curl` returns 403 for every name,
including ones that exist — check `serde` → 200 first to prove the request is
being answered at all.*

---

### I-3 — two "NOT settled" declarations survive the sections that settled them

Both are the same defect: an added section closed a question, and the paragraph
that declared the question open was never touched.

**Site A — §5b:448** (added by `39fcac3`, falsified by `3293210` 15 lines
later):

> **NOT settled, and explicitly out of this cycle:** whether the verbs in the
> right column belong where they are. … **Whether those move is the tier
> cycle's question, not this spec's**

§5c opens *"This settles what §5b parked"* and rules all six. §5b still tells a
reader the question is open and belongs to another cycle, with **no forward
pointer to §5c**.

Worse, §5b's characterisation is now contradicted on the merits, not just the
status: §5b groups **`bytecode`** with *"`compile`/`descriptor`/`bytecode` is
miniscript processing"* — i.e. fancy-processing tier — while §5c reads
`bytecode` as **STAYING**, *"`inspect` at higher resolution … the `m*1` string's
own encoding and nothing else."* A reader who stops at §5b gets the opposite
answer for that verb.

**Site B — §5a:284** (falsified by `90c74ee` + `9c6214a`):

> **The name below is NOT settled and does not yet follow the constellation's
> convention — see the measurement.**

The name **is** settled: `### The name: mnemonic-io-lib (operator, 2026-08-26)`
… *"**APPROVED by the operator, 2026-08-26**"*.

**What closes it.** §5b:448 → change to "Settled below in §5c", keep the
measurement as the data §5c reasons from, and drop `bytecode` from the
fancy-processing list (or mark it "read as staying, §5c"). §5a:284 → "The name
is ruled below" (the "does not yet follow the convention" half may stand, since
§5a's own argument is that the convention needs a third layer).

---

### I-4 — §5c never says **which cycle** the five verbs move in, and D7/§9a's justification is now false for them

**Sites:** §5c (whole section, no timing statement anywhere); §9a:1495-1497;
§6d:762-775; §7 P2:1222.

**What is wrong.** §5c is titled *"What moves to the toolkit, **decided**"* and
rules five verbs out of `md`/`mk`/`ms`. It contains **no statement of when**, and
**no reference to D7 or §9a**. Meanwhile:

- **D7 / §9a:1495** — *"**This cycle makes the encoding tier UNIFORM. It does
  not RELOCATE anything.** Every rule in §6 applies to a feature **already
  living where it belongs**:"* — that justification is now false for `split`,
  `combine`, `compile`, `derive` and `address`, which §5c has just ruled are
  **not** where they belong.
- **§6d:762** — *"`ms` alone has **eight** verbs"*, with prescriptive rows
  **`split`** *(add `--in`; argv refused)*, **`combine`** *(document `-`, add
  `--in`, then refuse)*, **`derive`** *(add `--in`; argv refused)*.
- **§7 P2:1222** — *"`ms` FIRST `--in` on **all eight verbs**, THEN the argv
  refusal, THEN the 0600 `--out`"*.

So the plan schedules three channel/refusal changes on three verbs the spec has
ruled are leaving `ms`, and §9a's stated reason for that work being in scope is
contradicted by §5c. Nothing in the document reconciles the two. §9a does not
cite §5c and §5c does not cite §9a; the only bridge is §5b's *"tier cycle's
question"* sentence, which I-3 shows is itself stale.

**Why it is Important.** An implementer working P2 has two sections giving
opposite answers about whether `ms split` / `ms combine` / `ms derive` are part
of this cycle's surface. The safe reading (D7 wins; the destination is decided,
the move is deferred) is correct but must be **read in**, not stated. This is
the *diff-falsifies-untouched-text* class the round exists to catch.

**What closes it — one sentence in §5c**, e.g.: *"**These moves are DECIDED, not
SCHEDULED.** D7 (§9a) still holds: this cycle relocates nothing. Every verb
above keeps its current home through P0–P4 and receives §6's uniformity
treatment there; the tier cycle executes the move and carries that treatment
with it."* Plus a matching qualifier on §9a's *"already living where it
belongs"* — e.g. *"already living where it belongs **for this cycle**; §5c rules
five verbs' eventual destination without moving them."*

---

## MINOR

### M-1 — "`repair` appears **31 times** in this document" is a line count, taken before §5c existed

**Site:** §5c:485.

| command, on the parent `f347feb` | result |
| --- | --- |
| `grep -ic repair` (**case-insensitive LINE count**) | **31** ← the source of the figure |
| `grep -o 'repair' \| wc -l` (occurrences) | 36 |
| `grep -oi 'repair' \| wc -l` | 38 |

At `9c6214a` the same commands give **39 / 48 / 51**. So the number was true of
a *line* count on the *previous* revision, and the text calls it a count of
*times*. Nothing rides on it — 48 > 31 only strengthens the argument that
`repair` is load-bearing.

**Closes it:** *"`repair` appears on 39 lines of this document (`grep -ic
repair`, measured at 9c6214a)"*, or drop the figure.

### M-2 — the manifest count contradicts itself five lines apart: six, then five

**Sites:** §5a:375 (new) vs §5a:380 (orphaned tail); also §5a:283 (inherited).

```
375: … a rename across six manifests once the code exists.
380: an unavailable name is a rename across five manifests if it is discovered
     after the code is written, and one line of the plan if it is discovered before.
```

Six is right — D5 and §5a both enumerate six consumers (`md`, `mk`, `ms`, `mt`,
`me`, the toolkit), and D5:1288-1291 says so *explicitly* while noting *"the
count has now been wrong in three different places, so it is enumerated at D5
rather than totalled."* The new text totals it again and lands next to a stale
five. §5a:283 (*"baked into five `Cargo.toml`s"*) carries the same stale value
but predates this object.

The 379-381 tail is also **dead text**: three successive commits prepended
material above it, so it now repeats *"P0 must confirm the name is free on
crates.io before publishing"* immediately after the paragraph that already says
P0 re-checks, and it offers *"one line of the plan if it is discovered before"*
for a check that **was** done before. No gate requires the sentence (checked
`scripts/*.sh` — `spec-structure-check.sh` matches no such phrase).

**Closes it:** delete the orphaned 379-381 tail; fix §5a:283 to six or point it
at D5.

### M-3 — `vectors` is not structurally unmovable; the clap-tree argument covers only `gen-man` and `gui-schema`

**Site:** §5c's third table + *"They stay, **structurally rather than by
judgement**"* and *"Moving these would require the toolkit to hold every other
binary's clap tree, which inverts the dependency."*

That is exactly right for `gen-man` (roff from that binary's clap tree) and
`gui-schema` (JSON of that binary's own flags). It is **not** the reason for
`vectors`, and the counterexample is a file this very spec cites 150 lines
earlier:

- **`mk-codec/src/bin/gen_mk_vectors.rs`** — mk's vector generation already
  lives **in the codec crate, outside the CLI**. (It is the single
  purity-grep hit in §5a's own codec measurement.)
- **`md vectors`** sources its corpus from **`md_codec::test_vectors::{MANIFEST,
  Vector}`**, a public codec API its own comment calls *"the single source of
  truth"* — not from a clap tree.
- The toolkit already vendors `md-codec`, `mk-codec` and `ms-codec`
  (`mnemonic-toolkit/vendor/`), so the dependency would not invert.

The real argument for `vectors` staying is **repo locality** — the corpus is
SHA-pinned in that format's repo — which is a judgement, and a good one. The
section just should not call it structural, having promised *"structurally, not
by judgement"*.

**Closes it:** split the row — keep `vectors` in the stay-list, label its reason
"corpus is SHA-pinned in that repo (locality, not structure)", and scope the
clap-tree sentence to `gen-man`/`gui-schema`.

### M-4 — "4 of 4 `-cli` crates are the reverse" quietly excludes `me-cli`

**Site:** §5a:337-339.

There are **five** first-party `-cli` crates. `me-cli` has **both** `src/lib.rs`
and `src/main.rs` (measured, §1.6), so it is a `-cli` crate that is *not* "the
reverse" of a codec. Stated as "4 of 4", the partition looks total; it is 4 of 5.

Nothing reverses — the load-bearing half is *"5 of 5 `-codec` crates are lib
with no main"* (true), and including `me-cli` makes the sibling claim *"`-cli`
marks a crate that builds a binary"* **5 of 5** rather than 4 of 4, which is
stronger. It is the denominator that is quietly chosen.

**Closes it:** *"5 of 5 `-codec` crates are `lib.rs` with no `main.rs`; 4 of the
5 `-cli` crates are the reverse, and the fifth (`me-cli`) is both."*

### M-5 — "and stuff like that" is quoted and never interpreted; the classification is exhaustive but never says so

**Site:** §5c:465-476.

The operator's *"Split combine compile **and stuff like that** go to toolkit"* is
quoted verbatim and the phrase is then never addressed. §5c does classify every
remaining verb — I verified all 33 subcommands across `md`/`mk`/`ms` land in
exactly one of {the four, →toolkit, STAYS, structural} (§1.4) — so in practice
the phrase covers nothing beyond the enumeration. But a reader cannot tell that
without running `--help` on three binaries, and "and stuff like that" is exactly
the kind of open-ended clause a later reader will re-open.

**Closes it:** one line — *"'And stuff like that' is taken to cover nothing
beyond the enumeration above: every subcommand on `md`, `mk` and `ms` is
classified here (12 + 10 + 11), leaving no residue."*

---

## NIT

### N-1 — the name subsection is the only unnumbered `###` in the document

`### The name: mnemonic-io-lib (operator, 2026-08-26)` sits at line 330 between
`### 5a.` and `### 5b.`. Every other `###` in the file is numbered
(`5a` `5b` `5c` `6a`–`6i` `9a` `9b`), so this one cannot be cross-referenced as
`§5x` and `spec-structure-check.sh` does not index it (it reports 26 sections
and 20 cross-refs, all resolving). Numbering it — or folding it under §5a as
`5a-continued` — would make it citable. Cosmetic; the gate is clean either way.

---

## What I checked and found CLEAN

- The four rulings are quoted verbatim and each is attributed to the operator
  with a date. The READING-vs-RULING boundary in §5c is drawn **exactly where
  the section says it is**: `md descriptor` and `md bytecode` are the only two
  labelled a reading, both are quoted from their own shipped `--help` (verified
  word-for-word), and neither is presented as an operator ruling.
- Which binary each of the six verbs leaves is unambiguous (§1.3).
- §6a's stdout table covers only the four uniform verbs — untouched by §5c.
- §6f's exit-code table has no row for any moved verb; the only cross-CLI
  normative rule among the six is `repair`/D26, as §5c says.
- The codec-purity block, the lib/bin partition, the `cargo install --list`
  block (7/7 rows) and the 16-check invariant all reproduce exactly.
- Both structure gates are clean at `9c6214a` (exit 0; 82 rows, 0 malformed).

## One thing worth the next author's attention, filed as an observation, not a finding

§5c records with care that the toolkit **keeps its own `repair`** and that the
two disagree by design. It does not note that the toolkit **already ships** two
of the five moving verbs under other names:

```
mnemonic ms-shares split    "Split a secret into N codex32 K-of-N shares"
mnemonic ms-shares combine  "Combine ≥K codex32 shares back into the secret"
mnemonic addresses          "list a wallet's receive/change addresses (batch, read-only)"
```

`ms split` / `ms combine` are the same function (`ms split`: *"Split a secret
(mnemonic / hex entropy) into N codex32 K-of-N shares"*). An implementer handed
*"`split` → toolkit"* with no other context could add a **second** codex32
sharing surface next to `ms-shares`. Not filed as a finding because no work this
cycle acts on it (D7), and because §5c's stated purpose is to let the tier cycle
"start from data" — which is precisely the argument for measuring the
destination before that cycle opens.

---

*Reviewer: independent R0 pass, scoped to `39fcac3`, `3293210`, `f06774a`,
`90c74ee`, `9c6214a`. All binaries invoked by absolute path with stdin from
`/dev/null`; no exit code read through a pipe.*
