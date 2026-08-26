# SPEC — constellation CLI uniformity (`md`, `mk`, `ms`, `mt`, `mnemonic`)

**Status:** DRAFT, R0 round 0 folded 2026-08-26. The round returned **NOT GREEN
— 4 Critical / 11 Important / 6 Minor / 2 Nit**; this revision responds to all
23. The verbatim report is `design/agent-reports/R0-cli-uniformity-spec-round0.md`
and the disposition table is `design/agent-reports/R0-cli-uniformity-fold-round0.md`.
**No code may be written until a round closes 0C/0I** (project CLAUDE.md — this
is risk-set work: it changes normative CLI behaviour and it touches secret and
bearer material).

**Goal, in the operator's words:**

> *"The goal is for the user to not care if they are dealing with mk or md or mt
> or ms."*

**Every measurement in this document was re-run against the built binaries
during the fold.** Where round 0 and the fold disagree, the fold's number is the
one written down and the disagreement is named at the point it occurs.

---

## 1. The finding that motivates all of it

**The safety gradient runs backwards.** Measured 2026-08-26 against the built
binaries, not read from source:

| tool | material it handles | material on argv | its advice for stdout |
| --- | --- | --- | --- |
| `mt` | one transaction — spends once | **refused, exit 1**, with purge advice | refuses mode 0644, names three remedies |
| `ms` | **BIP-39 seed entropy — spends everything, forever** | **accepted silently**, exit 0 | *"redirect … e.g. `> file.txt`"* |

`ms encode --phrase "<12 words>"` exits 0 and says nothing about that phrase now
being in shell history and in `ps`. Its stderr then recommends `> file.txt`,
which under the default umask produces **mode 0644** — the exact disposal `me`
and `mt` refuse at exit 2 (F-244, F-252).

**The tool holding the most dangerous material in the constellation has the
weakest handling of it.**

**Correction folded from round 0 (I-9).** An earlier draft of this table gave
`mt`'s argv refusal as exit 3. It is **exit 1**. `mt` returns only
`ExitCode::SUCCESS` and `ExitCode::FAILURE` (`mt-cli/src/main.rs:237,253,256`)
and no third code exists anywhere in `mnemonic-transaction/crates`. Reproduced:
`mt encode --qr "$(cat tx.hex)"` prints the refusal and exits **1**. `mt`
therefore has **no distinguishable exit code for a refusal**, which is an input
to the exit-code table in §6f rather than a defect in `mt`.

## 2. What is already uniform, and what is not

**The verbs are already uniform.** `encode`, `decode`, `verify`, `inspect` exist
in all four. Nothing in this spec changes a verb name. What diverges is
everything around them.

Measured, all cells re-run during the fold:

| | `md` | `mk` | `ms` | `mt` |
| --- | --- | --- | --- | --- |
| `--json` | yes | yes | yes | yes |
| `--in FILE` | — | `--keys FILE` only | — | **yes** |
| `--out FILE` | — | — | — | — |
| `-` for stdin | **`repair` only** | all 5 artifact verbs, and `--keys -` | 7 of 8 verbs — **not `combine`** | default, plus bare `-` |
| `--group-size` default | **5** | **5** | **5** | **off** |
| separator choices | space, hyphen, comma | space, hyphen, comma | space, hyphen, comma | whitespace only |
| non-artifact lines on stdout | **`chunk-set-id:`, chunked output ONLY** | **none, ever** | none | none |
| material on argv | positional | flags | `--phrase` | **refused (SPEC_mt_v0_1 section 8.2f)** |
| stdout bytes on failure | **0** | **0** | **0** | **0** |
| exit code, clap usage error | 2 | 64 | 64 | 2 |
| exit code, invalid artifact | 1 | **2** | 1 | 1 |

**The `mk` and `md` header cells are corrections folded from round 0 (I-1), and
they change what P3 has to do.**

- **`mk` emits no `chunk-set-id:` header, ever** — not even on a 2-chunk card.
  Measured: `mk encode --xpub … --origin-fingerprint … --origin-path … 
  --policy-id-stub 11223344` prints two `mk1` lines and nothing else on stdout.
  There is no such `println!` anywhere in `mk-cli`; `--chunk-set-id` exists only
  as an *input* flag. **`mk` has no header to remove.**
- **`md` emits the header only on the chunked branch** — one emission site,
  `descriptor-mnemonic/crates/md-cli/src/cmd/encode.rs:172`, inside the arm that
  chunks. Measured: `md encode 'wpkh(@0/<0;1>/*)'` prints no header;
  `--force-chunked`, or a keyed 2-of-2 that chunks naturally, prints one.

**The pipeline-safety invariant holds for a SINGLE producer**: every tool
contributes 0 bytes to stdout on a failure path. That matters because `fish`
reports a pipeline's status as the LAST command's, so a failed upstream is
invisible except by its silence.

**It does NOT hold for the composition D1 mandates, and that is C-1.** Zero
bytes from producer 3 of 3 is not silence — it is a shorter payload that packs
clean at exit 0. An earlier draft of this section called the invariant the one
thing here that needed no work; **that claim is retracted**, and §6g is the
response. (Described rather than quoted — the exact phrasing is a swept term in
`design/SUPERSEDED_TERMS.txt`, and reproducing it here would make the sweep red
for prose that is doing its job.)

## 2a. The affected surface is larger than four CLIs

Round 0 found a fifth CLI (I-4) and a sixth repo (I-11). The fold measured both
and found the reach wider than reported. **The spec's title now names five
CLIs, and this section is the scope statement the phasing in §7 must cover.**

**`mnemonic` (`/scratch/code/shibboleth/mnemonic-toolkit`) is a fifth
constellation CLI**, binary built at `target/debug/mnemonic`, and it is inside
**three** of this spec's decisions, not just the exit-code one:

- **D3 (argv).** `mnemonic bundle` takes `--passphrase <PASSPHRASE>` on argv.
  It *also* ships `--passphrase-stdin` — **an existing in-constellation
  precedent for D3's private channel**, which this spec previously cited only
  from `mt`.
- **D4 and §6c (grouping).** `mnemonic bundle` carries `--group-size` defaulting
  to **5**, `--separator` accepting **space, hyphen, comma**, and
  `--no-engraving-card` — the same surface §6c removes from the other three.
- **The D26 exit-code parity ruling names it explicitly** (§6f).

**`mnemonic-gui` (`/scratch/code/shibboleth/mnemonic-gui`) hand-mirrors the flag
surface §6 changes, and its drift gate is scoped to exclude most of it.**
Measured during the fold — wider than round 0 reported:

- `const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];` in **four**
  schema files: `src/schema/md.rs:24`, `mk.rs:15`, `ms.rs:33`, `mnemonic.rs:47`.
  Round 0 named two of the four.
- `default_value: Some("5")` at **eight** sites across those same four files
  (`md.rs:77`, `mk.rs:71`, `ms.rs:78`, `ms.rs:414`, and four in `mnemonic.rs`).
  Round 0 counted seven and missed `md.rs:77`.
- The drift gate disqualifies itself for exactly the CLIs this spec changes.
  `mnemonic-gui/tests/schema_mirror_defaults_drift.rs` states its scope as
  `mnemonic` only, and calls extending it to `md`/`ms`/`mk` a follow-on
  deliberately left out of that cycle.

So flipping the `--group-size` default and removing `hyphen`/`comma` produces
**zero test failures in the GUI**, and the GUI keeps offering a dropdown value
the CLI has deleted.

**One correction to round 0's premise, measured.** I-11 asserts that *all four*
CLIs carry a `gui-schema` subcommand. They do not: `md`, `mk`, `ms` and
`mnemonic` do; **`mt` does not** (`mt --help` contains no `gui-schema`). The
finding stands and is if anything sharper — the published-schema set is exactly
the set whose flags §6 rewrites, minus the one tool that already behaves.

**`mnemonic-engrave` — this repo — is itself an affected repo (I-10)**, through
its committed journey drivers. See §7 P2 and P3.

## 3. THE DEFAULT OUTPUT OF THREE TOOLS CANNOT BE PACKED

The decisive measurement, and the reason this is a defect rather than a
preference:

| `ms encode …` | piped into `me sysw pack` |
| --- | --- |
| **default** (grouped by 5, space separator) | **exit 4 — "not a form this container can place"** |
| `--separator hyphen` | **exit 4** |
| `--group-size 0` | **exit 0, 102-byte payload** |

The same holds for `md`. Ungrouped and unchunked, `md encode --group-size 0`
piped into `me sysw pack` gives exit 0. Add the `grep` that strips the header
and a **chunking** policy works too. The `--group-size 0` and the `grep` are
precisely the two defects — the flag works around the grouped default, the grep
works around the `chunk-set-id:` header.

**Measured during the fold, and this is the case P3's gate must use.** A keyed
2-of-2 chunks into four `md1` strings preceded by one header line; piped
straight into `me sysw pack` it fails at **exit 4, on record 0** — record 0
being the header. An unchunked policy emits no header and so **cannot exercise
the defect at all**, which is why §7 P3's gate is pinned to a chunking policy.

**So a user today must know two workarounds to compose anything.** That is the
"not care" requirement, unmet.

Grouping is a display concern by these tools' own admission — `md`'s help says
*"Display only; `--json` stays unbroken."* It is already understood as
presentation. It simply leaks into the canonical artifact.

## 4. The principle

**Generalise `SPEC_mt_v0_1` section 3b to the others.** Verbatim, from that
section:

> *"stdout carries the artifact, stderr carries everything the human must see."*

**Provenance correction, found during the fold and missed by round 0.** An
earlier draft of this section carried a *different* sentence in a block quote,
attributed to the same place, ruling that grouping is opt-in and that the
canonical artifact is ungrouped. **That sentence does not appear anywhere in
`SPEC_mt_v0_1.md`.** Its nearest ancestor is a *proposal* in
`design/agent-reports/R6-lens-implementability.md:489`, which recommended adding
such a ruling and explicitly said either direction would do so long as one was
chosen. So the spec's stated principle was a paraphrase of a review agent's
suggestion, presented as a verbatim quotation of a normative document. The real
sentence above is the citable one, it is in section 3b, and it says what this
spec needs. The ungrouped-canonical rule is therefore **this spec's ruling (D4),
not an inherited one**, and §6b states it in its own voice.

Every divergence in §2 is a violation of one rule:

> **stdout carries the canonical artifact and nothing else. Presentation goes to
> stderr. Secret and bearer material never arrives on argv.**

**The last clause carries D3's qualifier deliberately (I-6), and the qualifier
is load-bearing.** An absolute *"material never arrives on argv"* would delete
`md`'s and `mk`'s positionals, breaking every documented invocation of both.
`mt`'s own shipped refusal rules the other way in as many words: *"md and mk DO
take their strings as arguments; md1/mk1 are watch-only, so a leak there costs
privacy rather than the money."* `md`'s and `mk`'s stderr agree — *"stdout is
watch-only — public keys only, cannot spend"*. **Watch-only material stays on
argv.** §6d enumerates what does not.

`mt` is the only tool that obeys the rule, and the only one that composes.

## 5. Decisions taken (operator, 2026-08-26)

| # | Decision | Rejected alternatives |
| --- | --- | --- |
| D1 | **Composition is a shell pipeline into `me sysw pack`.** No new umbrella binary. | a new `m` front-end that sniffs and dispatches |
| D2 | **Host-side only this cycle.** "Engravable as plaintext in a QR" for `md`/`mk`/`ms` needs new record classes and device rendering; it becomes its own spec with a firmware gate. | doing both at once; device first |
| D3 | **Refuse secret/bearer material on argv, with `--allow-argv-secret`.** | refusing with no override; warning only; a declared-posture mechanism |
| D4 | **stdout is canonical (ungrouped); the grouped form moves to the stderr engraving card.** | flipping the default only; teaching `me sysw pack` to strip separators |
| D5 | **One shared crate owns the IO + safety layer**, depended on by all five. | spec + conformance vectors with copied code; a pure-logic crate with per-repo flag wiring |
| D6 | **`me sysw pack --expect <kinds>` closes C-1**, opt-in, keyed on kinds. | a record COUNT; a conjunction-shaped acceptance form; `pack` running the producers itself |

**On D3's rejected third option, recorded so it is not revisited.** A declared
machine posture — an environment variable or config file saying *"this box is
single-user and ephemeral, so argv is safe here"* — was proposed during the fold
and **declined by the operator**: *"We don't have to split hairs so finely.
Refuse argv with override is fine."* The override **is** the escape hatch for
the offline, Tails and satellite-link cases; it does not need a second mechanism
on top of it.

**On D4's rejected third option, recorded so it is not revisited:** teaching
`me sysw pack` to strip separators would make admission LOOSER on the
funds-sensitive side. `me` refuses an elided `md1` deliberately; *"strip whatever
looks like a separator"* is how a mangled string gets silently accepted.

**On D5:** four copies of one rule is the shape that let `pack` and
`pack_deterministic` drift until a vector round-trip caught it — that module's
own comment records it. §6 of this document is a fifth copy risk in waiting.
Round 0's I-5 correctly attacked the *precedent* originally cited for D5; §7 P0
now names the distribution mechanism instead of gesturing at one.

**D3 is already implemented in `me`, and that is the reference.** Measured in
the main checkout's working tree during the fold: `me sysw pack` refuses secret
**and** bearer records on argv, keyed on `Class::is_argv_forbidden()`
(`is_secret() || is_bearer()`), with `--allow-argv-secret` as the override and a
remedy block that names per-shell purge commands. Its doc comment carries the
operator's ruling of 2026-08-26: *"we want uniform behavior with secret bearing
between ms1 and passwords and mt1 to the extent we can."* **P0 extracts that
code; it does not re-derive it.**

*(An earlier revision warned that this work was newer than the branch the spec
was folded on. That was true while the fold was in flight and is false now: the
merge landed both, and `Class::is_argv_forbidden` is in this branch's source.
Kept as a note rather than deleted, because it is the second branch-relative
claim this spec has had to retract — a statement whose truth depends on which
commit you are standing on does not belong in a spec.)*

## 6. The surface, after

### 6a. Which verbs the stdout rule binds

Round 0's I-7 is correct: §3 and §4 reason entirely about `encode`, and applying
*"the canonical artifact and nothing else"* to every verb silently puts two
tools' machine-readable output in scope for a rewrite that no phase gates.
Measured stdout, per verb:

| verb | `md` | `mk` | `ms` | `mt` |
| --- | --- | --- | --- | --- |
| `encode` | the artifact | the artifact | the artifact | the artifact |
| `decode` | bare template | **labelled table, 5 fields** | **3 labelled lines** | bare hex |
| `verify` | `OK` | report | report | report |
| `inspect` | report | report | **report, 8 fields** | report |

**The rule binds `encode` only, this cycle.**

- **`encode`** — stdout is the canonical artifact, ungrouped, nothing else. This
  is the whole of §3's defect and the whole of D4.
- **`decode`** — **explicitly out of scope, and named so rather than left
  ambiguous.** `mk decode` and `ms decode` emit labelled multi-field reports
  that scripts read today. Changing them is a breaking change to a
  machine-readable surface with no funds-safety argument behind it, and it is
  not what the operator asked for. If it is ever wanted it gets its own phase
  and its own gate.
- **`verify` and `inspect` are REPORT verbs and are exempt.** Their entire
  output is commentary; there is no artifact for the rule to be about.

### 6b. Channels: `--in`, `-`, `--out`

- `--in FILE` — read the tool's **own input material** from a file. "Own input
  material" is not a formality: see §10 and C-2 below, where a filename implying
  a *different* artifact is what made the previous acceptance criterion
  unsatisfiable.
- `-` as a positional — read stdin. Accepted and ignored where stdin is already
  the default (F-250's fix in `mt` is the reference implementation).
  **Mostly shipped already, and much more widely than round 0's I-3 implies.**
  Measured per verb: `mk` documents `-` on all five artifact verbs *and* on
  `--keys -`; `ms` documents it on seven of eight, with `decode`, `verify`,
  `inspect` and `derive` reading stdin even when the positional is omitted;
  `md` documents it on `repair` alone. **So the real `-` gap is `md`'s four
  other verbs and `ms combine`** — two targeted additions, not a constellation
  rollout. `--in FILE` is the genuinely missing channel: only `mt` has it, and
  `mk` has `--keys FILE` for one flag.
- `--out FILE` — write the artifact to a file, **created 0600 by `me`'s
  `write_private`**, never `std::fs::write` (F-244).
- **stdout is used when `--out` is not given.** An earlier draft said *"when
  neither is given"* after a three-item list (M-5); the antecedent is `--out`
  alone, and the input channel has no bearing on where output goes.
- `--group-size` / `--separator` affect **the stderr card only** (D4).
- **`--json` is UNCHANGED and explicitly OUT OF SCOPE this cycle.**

**On `--json`, correcting a false premise (I-8).** An earlier draft excluded
`--json` on the grounds that it was *already uniform*. It is **unbroken** — all
three genuinely ignore `--group-size` and `--separator`, verified — but it is
not uniform. Measured with identical flags:

| | key holding the artifact | schema key | formatting |
| --- | --- | --- | --- |
| `ms` | `ms1` | `schema_version` (string `"1"`) | compact |
| `mk` | `mk1_strings` | `schema_version` (number `1`) | compact |
| `md` | `phrase` | `schema` (`"md-cli/1"`) | **pretty-printed** |

Three names for the artifact, three schema conventions, one outlier on
formatting. The exclusion stands on its own merits — `--json` is not what makes
a pipeline fail, and widening this cycle to a cross-CLI JSON schema would delay
the funds-safety work in P2 — but it no longer stands on a false claim. Filed as
a follow-up owned by a later cycle, not by this one.

**RULING, new in this fold: `mt` gains `--out` (I-2).** Round 0 is right that
this contradicts a shipped refusal string, and right to demand an explicit
ruling. The refusal `mt` prints today states that `mt` has no `--out` and that
stdout *is* the strings *by design*, citing section 3b. **The fold checked that
citation and section 3b does not say it** — section 3b rules that stdout carries
the artifact and stderr carries what the human must see, which is a rule about
*which stream*, not about whether a file channel exists. So `--out` does not
contradict section 3b; it contradicts only `mt`'s own restatement of it.

The reason to add it is F-244: `--out` exists so the artifact is **created
0600**, which a shell redirect cannot do. `mt`'s current remedies for a
world-readable stdout are `umask 077` and `chmod 600` — remedies that exist
*because* there is no `--out`. Consequences, which P1 owns and must not defer:

- `mt`'s mode-0644 refusal text changes; its remedy list gains `--out`.
- The test asserting that text changes with it. **That is P1's diff, and P1's
  gate below is written to expect it** rather than to forbid it.
- This ruling is **operator-reversible**. If it is reversed, §6b must instead
  carve `mt` out by name, because a silent exception is what round 0 caught.

### 6c. The separator rule

Whitespace only, everywhere. `mt` restricts to whitespace because its decoder
strips whitespace and nothing else, so a hyphen-grouped string is one `mt`'s own
verbs refuse — after the plates are cut.

**The cost is larger than an earlier draft stated (M-2), and stating it
accurately is the point.** That draft said `ms` alone tolerates hyphens on
decode. Measured during the fold, feeding each tool **its own** hyphen-grouped
output back:

| | `--separator hyphen` round-trips? | `--separator comma` offered? |
| --- | --- | --- |
| `md` | **yes, exit 0** | yes |
| `mk` | **yes, exit 0** | yes |
| `ms` | **yes, exit 0** | yes |
| `mnemonic` | offered, not measured | yes |
| `mt` | refuses | not offered |

So the removal costs **two** cosmetic options across **three** measured CLIs and
one unmeasured one, not one option in three — and `comma`, which no earlier
draft mentioned, goes with `hyphen`.

The cross-tool argument is unchanged and still decides it: the card is what a
human types back, and a rule that is safe per-tool and unsafe across tools is
exactly the kind an operator carries between tools. **The cost of uniform
whitespace-only is two cosmetic options; the cost of getting it wrong is a
plate.**

**D4 moves the grouped form "to the stderr engraving card", and three of the
five tools have no card to move it to (M-1).** Measured stderr on a successful
`encode`:

| | stderr on success | card-shaped? |
| --- | --- | --- |
| `md` | **1 line**, a `note:` — the same whether the output chunks or not | no |
| `mk` | **1 line**, a `note:` | no |
| `ms` | **4 lines** — word count, language, a passphrase note, a private-material warning | yes, and `--no-engraving-card` cuts it to the 1 warning |
| `mt` | a full report plus a `SUGGESTED LEGEND` | yes |
| `mnemonic` | not measured; it carries `--no-engraving-card` | presumed yes |

So D4 requires **inventing an engraving card for `md` and `mk`**, and an earlier
draft compressed that into three words in the P3 row. **P3 owns the card's
contents, and the plan must specify them**, or two implementers render them two
ways. The minimum the card must carry is the grouped string itself, since after
D4 that is the only place it exists.

**A consequence operators must be told about, because it is new.** After D4,
`ms encode --no-engraving-card`, and any pipeline using `2>/dev/null`, yield
**no grouped form anywhere** — today grouping arrives on stdout unconditionally,
so it survives both. That is the correct behaviour and it is a real change in
what a redirect throws away.

**Open interaction, for the plan's reconciliation sweep (M-3).** F-245 is open:
`me sysw pack` packs a record's trailing whitespace verbatim into the public
section. Reproduced during the fold — a record with one trailing space is
accepted at exit 0 with no warning. Making the separator whitespace-only makes
whitespace the only thing that can appear inside a grouped string, so the two
decisions meet. Not a blocker for this spec; it must not be discovered late.

### 6d. Argv refusal (D3)

Shape follows `SPEC_mt_v0_1` section 8.2f, which is already tested and already
lands correctly, and `me sysw pack`, which already implements the widened form:

- **The guard runs on raw `std::env::args()` BEFORE any argument parser sees the
  material. This is NORMATIVE, not an implementation note (C-4).**
- **Never echo the argument.** Report the CLASS and the LENGTH, not the value.
- Name the private channels: `--in FILE`, `-` for stdin.
- Give the purge commands, per §6h.
- `--allow-argv-secret` proceeds. It is greppable in a script, so a reviewer can
  find it. **This is the name already shipped on `me sysw pack`.**

**Why the ordering is normative and not an implementation detail.** `mt`'s guard
is correct *only* because it precedes clap. Its own source says so at
`mt-cli/src/main.rs:219-238`: the guard sits on `std::env::args()` and runs
before `Cli::parse()`, because when the check lived inside the `encode`
subcommand, clap rejected the unexpected positional first — **and clap's error
message echoed the entire bearer transaction back to stderr.** The refusal
written to stop a leak leaked it itself, through the argument parser.

**That echo is still live and the fold reproduced it.** A token the guard does
not classify as a transaction reaches clap and is printed back verbatim:
`mt encode --qr deadbeefcafe` prints `error: invalid value 'deadbeefcafe' for
'[-]'` and exits **2**. So:

- The override's **own parse** must also happen on raw argv. Wiring
  `--allow-argv-secret` as an ordinary clap flag moves the decision after clap
  and reinstates the leak — and that is the obvious implementation.
- The spec must say what happens to the material **after** the override admits
  it. **Ruling: admitted material is passed to the tool through the same
  internal path as `--in` content, and never re-presented to clap as a
  positional**, because a later, unrelated clap error would echo it.
- *"Never echo the argument"* is a property of **where the check sits in the
  process**, not of the refusal's wording. A spec stating only the wording will
  be implemented as only the wording.

**`mt` gains the override too**, for uniformity — it has none today.

**The channels are enumerated per tool, not left as a shape (I-3).** `ms` alone
has **eight** verbs, and `ms1` strings and codex32 shares are seed-equivalent —
`ms decode <ms1>` prints the mnemonic. Measured usage lines:

| `ms` verb | material channel today | after D3 |
| --- | --- | --- |
| `encode` | `--phrase` / `--hex`, each accepting `-` | add `--in`; argv refused |
| `decode` | positional, or `-`, or omitted | add `--in`; argv refused |
| `verify` | positional, or `-`, or omitted | add `--in`; argv refused |
| `inspect` | positional, or `-`, or omitted | add `--in`; argv refused |
| `repair` | `--ms1`, accepting `-` | add `--in`; argv refused |
| `split` | `--phrase` / `--hex`, each accepting `-` | add `--in`; argv refused |
| `combine` | **positionals ONLY — no `--in`, no `-`** | `-` and `--in` FIRST, then refuse |
| `derive` | positional, or `-`, or omitted | add `--in`; argv refused |

**`ms combine` is the ordering constraint that makes P2 non-negotiable, and
the fold narrowed it to exactly one verb.** Round 0's I-3 reads as though the
private channel is missing across `ms`; measured, it is present on seven of
eight verbs. **`combine` is the exception, and it is the worst possible one to
have.** Refusing argv there before `-` and `--in` exist removes the **only** way
to recombine split shares — the recovery path, the one that matters when
everything else has failed. §7 P2 is ordered accordingly. Note also that
`ms derive` already ships `--passphrase-stdin`, a second in-constellation
precedent for D3's private channel alongside `mnemonic bundle`'s.

**On the flag's name (M-4), and this is the one round-0 finding this fold
DECLINES.** M-4 argues that `--allow-argv-secret` mis-names bearer material,
since `mt`'s artifact is bearer rather than secret and the spec is careful about
that distinction elsewhere. The argument was sound when it was written and the
fact changed under it. `me sysw pack` now ships `--allow-argv-secret` as the
override for a predicate that is *deliberately* the union of both classes, under
a ruling asking for uniform handling of secret and bearer at a public channel;
its own comment states that the two differ in kind but are the same problem at
argv, so they get the same answer. Renaming is therefore no longer a naming
choice but a rename of shipped surface, against a ruling that says the union is
the point. **Declined. The name is `--allow-argv-secret` in all five CLIs.**

### 6e. The write gate

The layer `me` grew during the 2026-08-25 burndown, generalised — **with one
part of it retracted (C-3).**

- A **world-readable file** is refused, naming the MODE measured and stating that
  only the file's own mode was checked (F-252) — an ancestor directory may
  already make it unreachable.
- `--allow-world-readable` overrides the mode gate. Verified during the fold
  that it does **not** override `me`'s terminal gate, which is correct as
  specified.
- **A refusal about the INPUT outranks one about the destination** (the ordering
  regression the journey caught). Verified in `mt`: with a bad transaction *and*
  a 0644 stdout, the input refusal is printed first.
- **No line carrying SECRET MATERIAL prints until every gate that can abort the
  write has run** (F-246).

**RETRACTED: the terminal gate does NOT generalise (C-3).** An earlier draft
lifted F-253 — a terminal is refused for any artifact that is bearer or secret —
from `me` to all four tools. The fold measured both halves and the lift is
wrong:

- **`me`'s refusal states a reason that is specific to a binary container.** Run
  on a pty, `me sysw pack` refuses at exit 2 and the load-bearing clause is that
  writing there would paint **raw binary** across a scrollback that is often
  logged. `md1`/`mk1`/`ms1`/`mt1` strings and a `tx:` record are short printable
  ASCII that a human must *read* in order to hand-engrave. **The predicate is
  false for all four CLIs.**
- **`mt`, the tool this spec generalises FROM, deliberately prints to a
  terminal.** Measured on a pty, naming the input because a count without one is
  not a measurement (round-1 B6, same defect as B2): `mt encode --quiet --in
  <the corpus `even` vector>` prints all **six** of that vector's strings and exits **0**. Its bearer-exposure warning fires on the *opposite*
  condition — it warns that stdout is **not** a terminal, so the strings went
  somewhere that keeps them. `mt` treats the terminal as the **safe** disposal
  and the file as the dangerous one. The earlier draft inverted that for its own
  reference implementation.
- **And the gate would not close the hole it names.** Refusing a terminal directs
  the operator to `--out FILE`. To hand-engrave they must then read the file,
  putting the material on the same terminal with no gate in the way — and
  leaving a disk copy that would not otherwise exist. Net effect on `ms encode`:
  a screen-only exposure becomes a screen exposure **plus** a disk artifact.

**Ruling: the terminal gate stays scoped to `me`'s binary container.** It is
justified by binary-in-a-scrollback and by nothing else. `mt`'s
print-to-terminal behaviour is **deliberate and is recorded here as such**, so
that a future reader does not "fix" it. If a terminal gate is ever wanted for
`ms`, it needs its own justification and a remedy the operator can act on
*without* writing the secret to disk — neither of which exists today.

**F-246 is restated narrowly, and the earlier generalisation is withdrawn.**
F-246's actual title (`design/FOLLOWUPS.md:10344`) is that `me sysw pack`
generates and **prints a passphrase** before it validates the records — it is
about emitting *secret material* early, not about any line that describes the
artifact. The broad form would have been a real change to `mt`: measured, `mt
encode` prints its complete report — `TX`, `OUT`, `FEE`, `LOCKTIME`, `INPUTS`,
`STATUS`, `CUT`, `PREFIX` and the full suggested legend, ending at stderr line
69 — and the destination refusal does not appear until line 105. None of that
report is secret material, and printing it before a refusal is not a defect.

*A note on round 0's evidence for this point, since a later reader will check
it.* I-2's quoted reproduction passes `--quiet` and then shows the report lines.
The fold could not reproduce that: with `--quiet` those lines are suppressed
(on the corpus `even` vector, `--quiet` gives **70** stderr lines against
**108** without it, and none of `TX`/`CUT`/`PREFIX` appear), and the report shows only
**without** `--quiet`. **The finding is correct and is folded; its evidence
command is not the one that demonstrates it.**

### 6f. Exit codes

An earlier draft disposed of this in one sentence — one table, all four, `mk`'s
2 becomes 1, details deferred to the plan — and supplied no table. Round 0's I-4
is correct that this is not deferrable: the space is a **five**-CLI system with
a recorded cross-CLI ruling that the sentence would have broken.

**The existing ruling, verbatim from `md repair --help`:**

> Exit codes (**D26 cross-CLI parity** with `ms repair` / `mk repair` /
> **`mnemonic repair`**): 0 — every input was already valid (no corrections
> applied) 5 — at least one chunk had corrections applied (REPAIR_APPLIED) 2 —
> atomic-fail […]: ANY chunk failing BCH capacity fails the whole call …

The elision is a citation to D28 of another document's plan. It is dropped
rather than reproduced: it carries a **section sigil**, and this file's
structure gate resolves a bare sigil against *this* document — so quoting it
verbatim would create a cross-reference that passes the gate while pointing at
§1 of this spec, which is exactly the false-clean citation the gate's own
comments warn about.

And `ms repair --help` records a deliberate **divergence** from that parity —
exit **4** rather than 5, because a corrected `ms1` is an unverified candidate
that cannot self-verify. **The non-uniformity is reasoned and load-bearing.**

**Measured where the cell says a number; cells that say "not measured" were
not run.** The header previously claimed every cell was run while three cells
in the last row said otherwise — a table cannot assert more than its contents.

| CLI | clap usage error | invalid artifact | repair applied | repair uncorrectable |
| --- | --- | --- | --- | --- |
| `md` | 2 | 1 | 5 | 2 |
| `mk` | 64 | **2** | 5 | 2 |
| `ms` | 64 | 1 | **4** | 2 |
| `mt` | 2 | 1 | n/a | n/a |
| `mnemonic` | 64 | not measured | **4** | not measured |
| `me` | 2 | 4 = unplaceable record; 2 = terminal refusal | n/a | n/a |

**On `mnemonic repair` — MEASURED at last, and it settles against the ruling
(round-2 N3).** Round 0 inferred this cell from the parity ruling instead of
running it. Round 1 measured 4; the round-1 fold declined to adopt that number
because the controller could not reproduce it, and left the cell empty. **That
refusal was right and the outcome was wrong** — declining to transcribe a
reviewer's figure is discipline; leaving the cell unmeasured afterwards is just
the defect with a different label. Round 2 supplied the input that repairs, and
the controller reproduced it:

```
$ md repair       md1yqpqqzqq8xtwhw4xwn4qh   -> exit 5
$ mnemonic repair md1yqpqqzqq8xtwhw4xwn4qh   -> exit 4
```

Corrupt one character of `md encode`'s own help example. **Both tools apply the
IDENTICAL correction** — each returns `md1yqpqqxqq8xtwhw4xwn4qh` — and they
disagree only on the exit code. `mnemonic` prints an UNVERIFIED banner
explaining its 4: a single-string `md1` has no cross-chunk oracle, so the
correction cannot be confirmed.

**So D26's repair parity does NOT hold across five CLIs, and this spec must
stop citing it as though it does.** The ruling was written when the
constellation was four tools; `mnemonic` diverges, for a stated and defensible
reason. **The codes cannot be declared FROZEN on the strength of a parity that
is measurably false** — §6f's freeze is retracted below, and P0 owes a restated
D26 that either admits `mnemonic`'s divergence as deliberate or changes it.

Two collisions this table makes visible and the one-sentence version hid:

- **`md encode` with no template exits 2**, colliding with clap's own 2 on the
  same binary.
- **`ms repair`'s 4 collides numerically with `me sysw pack`'s
  unplaceable-record 4**, and both are visible in the same `$?` in a pipeline.

**What this spec rules, and what it hands to the plan:**

- **The repair codes are NOT FROZEN, and the earlier claim that they were is
  retracted (round-2 N3).** It rested on a five-CLI parity that is measurably
  false: `md repair` exits 5 and `mnemonic repair` exits 4 on the same input,
  applying the same correction. D26 was written when the constellation was four
  tools. **What this cycle rules is narrower and true:** it does not renumber
  any repair code, because a plan that renumbers them silently changes what
  callers read. **P0 owes a restated D26** that either admits `mnemonic`'s
  divergence as deliberate — it prints an UNVERIFIED banner and has a reason: a
  single-string `md1` has no cross-chunk oracle — or changes it. Freezing is a
  decision for that restatement, not for this spec.
- **`mk`'s invalid-artifact 2 becomes 1**, converging on `md`/`ms`/`mt` and
  removing its collision with `md repair`'s atomic-fail 2. This is the only
  code this cycle changes.
- The clap-usage split (2 versus 64) is **recorded and not resolved here**. It
  is a clap convention difference, it breaks no pipeline measured, and
  normalising it would touch five CLIs for no safety gain. Filed with an owning
  phase, not folded into P0.
- The plan **must fill the two `mnemonic` cells still marked "not measured"**
  — invalid-artifact and repair-uncorrectable — before P0 closes. The third,
  repair-applied, is now measured at 4. They are marked, never guessed.

### 6g. `me sysw pack --expect <kinds>` — the C-1 contract

**The defect.** §10's acceptance form is a brace group of producers feeding one
`me sysw pack`. When *one* producer refuses, the group still exits 0, `pack`
still exits 0, and the operator gets a payload with a record silently missing.
Reproduced: identical pipelines differing only in whether `mt`'s input is valid
produced payloads of **1794 B and 102 B**, and **both exited 0**. `me sysw show`
gives no hint that a record was expected. Substitute `mk` for `mt` and the
missing record is a cosigner card — a backup the operator believes is complete
and that cannot restore the wallet.

**The defence exists and D1 steps around it.** `me` already closed the *total*
case: `: | me sysw pack` exits **2** with a message naming the mechanism exactly
— an empty input is what a failed upstream leaves behind, so it is refused
rather than packed into a container that holds nothing and still flashes. A
three-producer group guarantees the input is never empty, so that guard never
fires.

**Operator ruling, 2026-08-26.**

- **`me sysw pack` gains `--expect <kinds>`** — for example
  `--expect descriptor,cosigner,transaction`. `pack` refuses when a named kind
  is absent from the stream.
- **Keyed on KINDS, not counts, because a chunked set is N records and N is
  unpredictable.** Measured 2026-08-26, each with the invocation that produced
  it — a bare number is not a measurement:

  ```
  md encode --group-size 0 --from-policy 'pk(@0)' \
            --context segwitv0 --key '@0=<account xpub>'   -> 2 md1 strings
  mt encode --in <the corpus `even` vector's raw_hex>      -> 6 mt1 strings
  ```

  Two records for the simplest descriptor the compiler accepts, six for one
  small transaction, and neither number is predictable from the input. So
  `--expect 3` would be wrong more often than right.

  **Note the first command's SHAPE.** `md encode 'pk(@0)'` — a bare template
  argument — is REFUSED (`unsupported descriptor wrapper`); the count comes
  from the `--from-policy` compiler path with a concrete key. An earlier
  revision of this bullet quoted the number without the invocation and was
  therefore unverifiable, which is how it stood while being wrong about the
  command and citing an unnamed "reference transaction" for the second figure.
- **When `--expect` names a kind, an INCOMPLETE chunk set of that kind must
  REFUSE rather than warn.** Without this, C-1's smaller sibling survives.
  Reproduced during the fold: feeding `pack` 1 of a 2-chunk `mk1` set prints
  `me: record 0 … an md1/mk1 this tool could not decode; the device will treat
  it as a SECRET`, reports the record as unconfirmed, **writes the payload, and
  exits 0.**
- **`--expect` is OPT-IN, not required.** Requiring it would put ceremony back
  into the common single-record case, which is what this decision avoids.
- **Known and accepted limitation, stated plainly:** an operator who never
  passes `--expect` gets no protection. The only design that gives the property
  unconditionally is `pack` running the producers itself, which was considered
  and **rejected** because it makes `me` a program that invokes other programs.

**Implementation constraint the ruling's own wording gets wrong, and P0 must not
inherit (found during the fold; round 0 did not reach it).** The ruling is
stated as keying on classes `me` already computes, naming `Class::Md` and
`Class::Mk`. **Those variants do not exist.** `me`'s `Class` enum
(`crates/me-cli/src/sysw/record.rs:44`) has a **single `MdMk` variant** covering
both, alongside `Mnemonic`, `Codex32Secret`, `Passphrase`, `FreeText`,
`Descriptor`, `Mt`, `Tx`, `Address` and `Unknown`. `--expect descriptor,cosigner`
keyed on `Class` alone **cannot distinguish a descriptor card from a cosigner
card.**

The discriminant exists one level down and P0 uses it: `mdmk_unconfirmed`
already groups by `(hrp, chunk_set_id)` via `seal::record::chunk_key`, and
switches on the HRP character — `'d'` reassembles through `md_codec`, `'k'`
decodes through `mk_codec`. So:

- **`--expect`'s kind vocabulary resolves through the HRP for `md1`/`mk1`**, and
  through `Class` for everything else.
- **`mdmk_unconfirmed` already computes the incomplete-set predicate** the third
  bullet needs. `--expect` escalates its report to a refusal for named kinds; it
  does not need a second walk, and a second walk would be a second thing to
  drift.
- The kind vocabulary must be **fixed and enumerated in the plan**, not invented
  per call site, and it must map onto exactly one of those two discriminants per
  kind.

### 6h. Remedy text must be executable

**Operator instruction, 2026-08-26: a message that tells the operator to clean
up must tell them HOW, with the exact command, at the step doing the telling.**
Telling someone to *"remove the line from your shell history"* says WHAT and not
HOW, and that phrasing is shipped today in `me`'s argv refusal.

The reference implementation is `me sysw pack`'s widened argv refusal, landed
2026-08-26. Every rule below is drawn from it or verified during the fold:

- **Name the command for the operator's shell.** The commands differ, and a
  generic paragraph is wrong for at least one shell. `bash`/`zsh` edit
  `$HISTFILE` in place; **`fish` does neither** — verified: fish 4.8.1 provides
  `history delete [--exact | --prefix | --contains]`, its history lives at
  `$XDG_DATA_HOME/fish/fish_history` rather than `$HISTFILE`, and that file is a
  two-line-per-entry `- cmd:` / `when:` format, so a stream edit that deletes
  matching lines would strip the command and leave an orphaned timestamp.
- **Match on the COMMAND NAME, never on the secret.** Anchoring the pattern on
  the material types it into history a **second** time. The message must say
  this, because it is the obvious wrong move. Verified working:
  `sed -i '/me sysw pack/d' "$HISTFILE"`.
- **Do not tell a zsh user to run the history builtin with `-d`.** Measured on
  zsh 5.9.2: `-d` is a **display** flag that prints timestamps, and the builtin
  rejects the invocation. Advising it would report success while purging
  nothing. `me`'s shipped text names this trap explicitly and this spec requires
  that.
- **Name the override at the point of refusal**, with the condition under which
  it is reasonable — a single-user air-gapped box or an amnesic Tails session.
- **The remedy must not forward-reference a channel that does not exist.**
  This rule was earned rather than imagined: `me`'s refusal did advise a
  secret-class operator to reach `ms encode` through a `--in` flag, and that
  flag does not exist — the binary exits 64. **It was fixed in `956eea3`,
  before this spec shipped**; `me` now advises `ms encode --phrase - <
  seed.txt`, which is `ms`'s actual stdin idiom and is verified to pipe into
  `me sysw pack`. **Stated forward, because the tree already satisfies it:**
  when P2 gives `ms` an `--in`, that line becomes the `--in` form — and not
  before. This rule is not licence to write the older advice back in.
- **NOT YET VERIFIED, and marked rather than specified:** an interactive shell
  holds history in memory and can rewrite the file on exit, so an in-place edit
  may be undone. A complete recipe has to address that. **No command for it is
  stated here because none has been verified**; P0 owes the measurement before
  it writes the sentence.

### 6i. Refusal taxonomy — posture versus correctness

**Recorded as a taxonomy. No behaviour follows from it.** The word "refuse" is
used in this codebase for two different reasons, and a future reader must not
merge them.

- **Environment posture** — the refusal depends on who can observe the machine.
  Material on argv; a world-readable stdout; `me`'s terminal gate.
- **Artifact correctness** — the refusal depends on the artifact being wrong or
  worthless, regardless of where it runs.

Measured in `mt`: `Refusal::new(` is constructed at **56** sites, naming **12**
distinct section-8 subsections of `SPEC_mt_v0_1` (written without the sigil —
see the note in §8a; a sigil here resolves against THIS document and goes green
against the wrong target). Exactly **two** of the twelve are posture — the argv
section and the world-readable-stdout section. The other ten are correctness:
not finalized, fee rate over the ceiling, a malformed input-value argument, a
`non_witness_utxo` that hashes wrong, an input that looks like base64 PSBT and
does not decode, inputs carrying no signature, an input absent from the UTXO
set, a satisfaction with no signature, a chunk count over the ceiling, and an
`ms1` string handed to a transaction tool.

**Why it matters here:** an air-gap or an amnesic session changes who can see
your machine. It does not make an unsigned transaction worth engraving. D3's
override is scoped to the posture pair and to nothing else.

## 7. Phasing

| phase | content | gate |
| --- | --- | --- |
| **P0** | the shared crate: `--in`/`--out`/`-`, argv guard with pre-parser ordering, write gate, exit codes, remedy text per §6h, the `--expect` kind vocabulary per §6g. Extracted FROM `mt`/`me`. Plus the distribution mechanism below. | its own tests + an R0 round closing 0C/0I + the two `mnemonic` exit cells still marked "not measured" filled + D26 restated against the measured 5-vs-4 repair divergence + the in-memory-history question of §6h measured |
| **P1** | `mt` adopts the crate, and gains `--out` (§6b) and `--allow-argv-secret` (§6d). | `mt`'s 236 tests pass, **with the diff to them enumerated and each edit justified by a named §6 ruling** |
| **P2** | `ms` FIRST `-` on `combine` and `--in` on all eight verbs, THEN the argv refusal, THEN the 0600 `--out`. Plus this repo's journey drivers. **Highest safety value; do it before the cosmetic work.** | round-trip vectors; `ms combine` and `ms repair` each driven through the private channel; the 18 argv call sites migrated; `me`'s remedy text still naming only channels that exist |
| **P3** | `md`, `mk` header off stdout, grouping to stderr, `--in`/`--out`. Plus `mnemonic`'s grouping surface AND its argv refusal across all five of its secret-material channels (`bundle`, `convert`, `derive-child`, `restore --passphrase`, `electrum-decrypt --decrypt-password`), and the GUI mirror. Plus golden regeneration. | `md encode` into `me sysw pack` runs with **no flags and no grep, on a CHUNKING policy**; `mnemonic-gui`'s schema mirror regenerated; the 7 goldens regenerated; **`mnemonic`'s five secret-material argv channels each refused, named one by one** |
| **P4** | the operator journey: several inputs of different kinds, one payload, `--expect` engaged. | a captured journey that regenerates, and that FAILS when one producer is made to refuse |

**On `mnemonic`'s argv surface (round-1 B4).** The fold widened this spec from
four CLIs to five and ruled the override name uniform across all of them — then
gave `mnemonic` only its *grouping* work, leaving its argv exposure with no
owning phase at all. That is the same defect as the one round 0 raised for the
golden files, re-introduced for the tool the fold had just added.

**Five channels carry secret material**, and each is a place a seed phrase or a
decryption password reaches argv: `bundle`, `convert`, `derive-child`,
`restore --passphrase`, and `electrum-decrypt --decrypt-password`. They are
named in P3's row rather than left to "the `mnemonic` work", because a phase
item that does not enumerate its sites is one a later reader satisfies by doing
less.

**P2 before P3 is deliberate**: the seed-phrase-on-argv hole is the finding with
funds behind it; the grouped default is a usability defect.

**P0 — the distribution mechanism, which no earlier draft named (I-5).** D5's
crate becomes a cross-repo dependency that must be released or re-pinned before
any of P1/P2/P3 can consume a change to it. The constellation already uses
**both** mechanisms and they are not interchangeable:

- **crates.io version deps.** `me-cli` takes `md-codec = "0.42"`,
  `mk-codec = "0.4"`, `ms-codec = "0.7"`.
- **A git-rev pin.** `me-cli` takes `mt-codec` from a GitHub rev, and the
  Cargo.toml comment says why in as many words: a path dep does not resolve in a
  fresh CI checkout, and a rev pin keeps `cargo publish` **deferred**, because
  publishing is irreversible and pinning is not.

An earlier draft cited `mt-codec` as the precedent for D5. It is a real
precedent for *consuming a crate across repos* and the **opposite** of what D5
needs for *shipping a change to four consumers* — the rev pin exists precisely
to avoid a release step, and D5 needs one. **P0 must name which mechanism it
uses.** Two further facts it has to absorb:

- **The code being extracted is not in a library.** `write_private` is at
  `crates/me-cli/src/main.rs` (line 856 at the time of writing — a line number
  in a file this cycle keeps editing is a fact with a short shelf life; grep for
  the name), inside a **binary** crate, not exported by
  `me`'s `lib.rs`. It is tested through the binary's integration tests, not
  through an API. Extraction is fresh work with no existing consumers holding it
  steady.
- **Cadences and versions are already independent:** `md-cli` 0.13.0, `mk-cli`
  0.13.0, `ms-cli` 0.16.0, `mt-cli` 0.1.0, `me` 0.7.0.
- **A `path =` scheme would be ambiguous for `mnemonic-transaction`, which
  currently exists at two locations (N-1).** Verified: the checkout is at
  `/scratch/code/shibboleth/mnemonic-transaction` and a **git worktree** of it
  sits at `/scratch/code/shibboleth/_work/p3b/mnemonic-transaction` — the
  worktree's common git dir resolves back to the checkout's `.git`. From
  `me-cli` those are two different relative paths for one repo, and the worktree
  is transient. This is a further argument against `path =` on top of the
  fresh-checkout argument the existing Cargo.toml already records.

**Mixed states are ACCEPTABLE and are stated so rather than left to be
discovered.** A constellation where `ms` has the argv guard and `md` does not is
an unavoidable intermediate given the phase order, and it is safe: the guard is
per-tool and per-class, and the tools that lag are the watch-only ones §4 carves
out anyway.

**P1's gate, corrected (I-2).** *"`mt`'s suite unchanged"* is unsatisfiable and,
as a gate, would be met by weakening §6 rather than by proving the port — three
parts of §6 change `mt` regardless of the crate's correctness: `--out` (§6b),
`--allow-argv-secret` (§6d), and the refusal text those imply. The count is
right — `grep -rc "#\[test\]"` over `mnemonic-transaction/crates` totals
**236**, re-run during the fold — but the gate is now *enumerate the diff and
justify each edit*.

**P1 remains the least risk, for the reason the earlier draft gave badly.** `mt`
needs **zero** test changes for D3, D4 and §6c — it already implements all
three. Its diff is confined to the two rulings above.

**P2 owns this repo's journey drivers, and that ordering was a latent blocker
(I-10).** `mnemonic-engrave`'s own committed drivers shell out to `ms` with the
seed on argv: **18 call sites across 7 scripts** under `design/journeys/` —
`transcript.sh`, `transcript_hashvault.sh`, `transcript_pathological.sh`,
`transcript_tr_pathological.sh`, `derive-rcw-keys.sh`,
`derive-pathological-keys.sh`, `derive-hashvault-keys.sh`. D3 lands in P2, two
phases before P4's *"a captured journey that regenerates"*, and breaks every one
of them. **Either P2 carries the migration or P4's gate is unsatisfiable when it
is reached.** P2 carries it.

**P3 owns the golden regeneration, and the count is smaller than round 0
reported.** I-10 gives 12 files carrying `chunk-set-id:`, including generated
HTML under `design/journeys/out/`. Measured during the fold: **`design/journeys/out/`
is not tracked** — `git ls-files design/journeys/out` returns nothing — so those
are build products that regenerate themselves. **7 tracked files** under
`design/journeys/` carry the line (5 transcripts and 2 drivers), and **P3 owns
exactly those 7** — they are the ones a regeneration changes.

**The wider count across `design/` is deliberately not pinned.** It has been
written as 28, reported as 29 by round 1, and measured as 30 by the controller,
all correctly at the moment each was taken: every new document that *discusses*
the header increments it, and this spec and its own review reports are three of
them. A self-referential count is a fact with a shelf life measured in commits.
The actionable number is 7; the rest are prose about the format and are a
documentation sweep whose size is whatever this prints at the time anyone
asks — **31 as of this commit**, and the command is written out because the
previous revision replaced a wrong number with a command that silently printed
a different wrong one (it had no pattern to grep for):

```
git ls-files design | xargs grep -l 'chunk-set-id:' | wc -l
```

## 8. What is NOT verified, and must be before the plan closes

Two of the three open items an earlier draft carried are **closed by
measurement** and struck.

- **STRUCK — `mk`'s stdout shape.** Closed in §2. `mk encode` runs; it needs
  `--xpub` plus `--origin-path` plus one of `--policy-id-stub` / `--from-md1`.
  It emits **no header, ever**, and its default grouping is 5. Nothing about it
  is inferred.
- **STRUCK — whether `mk encode --from-md1` accepts a multi-chunk set (M-6).**
  It does. The flag is documented **Repeatable**, and repeating it across a real
  4-chunk `md1` set exits 0. The failure an earlier draft recorded was a
  **space-joined single value**, which is a usage error, not a capability gap.
  There is nothing here for this cycle's scope.
- **PARTLY DONE, and it must not be re-deferred — which existing invocations
  break.** An earlier draft deferred this wholesale to the plan. Doing it is
  what surfaced the P2/P4 ordering blocker above, so it belongs to the scope
  decision rather than after it. Enumerated so far: this repo's 18 argv call
  sites; `ms`'s own suite, where **31 of 76 test files** under
  `mnemonic-secret/crates/*/tests/` reference `--phrase` or `--hex`, out of 276
  test functions; `mt`'s refusal-text tests; `mnemonic-gui`'s four schema files.
  **Still to enumerate:** `md`'s, `mk`'s and `mnemonic`'s own suites, and any
  script outside these repos.
- **STILL OPEN — the in-memory shell-history question in §6h.** No command is
  specified for it because none is verified.
- **STILL OPEN — two `mnemonic` exit-code cells** in §6f: invalid-artifact
  and repair-uncorrectable. The repair-applied cell is measured (4).
- **RECORD HYGIENE, so the plan does not re-open closed work (N-2).** This spec
  cites F-246, F-250, F-251, F-252 and F-253 as settled, and the behaviour is
  present in the binaries — verified during the fold: `mt encode -` works
  through a pipe, `mt`'s world-readable refusal carries the F-252 wording about
  only the file's own mode, and `me`'s terminal refusal fires at exit 2. **None
  of those five carries a `CLOSED` marker in `design/FOLLOWUPS.md`**, while
  F-244 does. The citations are correct about the code and stale about the
  record. Not a spec defect; a sweep the plan performs before it schedules
  anything against those numbers.

## 8a. What the structure gates say

**Both gates are now CLEAN on this file, and that is a change from the previous
revision** — which reported 7 structural defects and 2 malformed table rows, all
of them false positives, and documented them here rather than removing them.
Documenting a permanent FAIL teaches a reader to skim the gate's output, which
is how a real finding hides.

- **The five duplicate-section reports are gone.** They arose because
  `scripts/spec-structure-check.sh` keys a heading on its leading integer plus
  at most one letter, so five decimal-numbered subsections of section 6 all
  keyed to `6`. The subsections are now lettered, which the gate keys
  distinctly, and which its own comments show was the intended form. Real
  duplicates in section 6 are now visible again. `SPEC_engrave_transaction.md`
  still trips the same class 21 times and is a candidate for the same fix.
- **The two table-cell reports are gone.** They arose from escaped pipes inside
  phase-table cells, which neither gate honours — `plan-table-check.sh` says so
  in its own footer. The affected cells are rewritten to describe the pipeline
  in words instead of drawing it, so no cell contains a pipe and both gates
  count correctly without an exception.
- **The two cross-reference failures from the previous revision stay fixed.**
  They were sections of another document. **Naming the file is not enough**: the
  gate matches the section sigil wherever it appears, so an external reference
  that keeps the sigil after the filename still resolves against *this* file and
  still fails. External references here therefore drop the sigil entirely and
  read as *"section 3b"*. The bad form is described rather than reproduced,
  because writing it out re-creates it — and re-running the gate is what caught
  that.
- **THAT CLAIM WAS TOO CONFIDENT, and round 1 (B5) proved it.** It was stated
  document-wide while a third external reference still carried a sigil inside a
  code span — and because it named a section number this document also has, the
  gate resolved it against the wrong target and reported STRUCTURE OK. **A green
  gate is evidence about the references it could resolve, never about the ones
  it resolved to the wrong place.** That site is now de-sigilled, and the sweep
  that finds this class is to list EVERY sigil in the file and read each one,
  not to trust the exit code.

## 9. Out of scope, explicitly

- **QR-for-everything** (D2). `md`/`mk`/`ms` payloads engraved as plaintext QR
  needs new record classes, device rendering, plate layout and an S0 hardware
  gate. Its own spec.
- **A new umbrella binary** (D1).
- **`--json` schema uniformity** (§6b). Unbroken today, not uniform, and
  excluded on cost rather than on the false premise an earlier draft gave.
- **`decode`, `verify` and `inspect` stdout shapes** (§6a).
- **The clap-usage exit-code split, 2 versus 64** (§6f).
- **A declared-posture mechanism for argv** (D3). Declined by the operator; the
  override is the escape hatch.
- **F-247**, the NFC-fit line on `mt encode --qr` — deferred by the operator,
  unrelated to this cycle.

## 10. Acceptance

**The previous acceptance criterion could not be satisfied by any implementation
of §6, and this is C-2.** It named `md encode --in wallet.desc` and
`mk encode --in cosigner1.xpub`. Measured:

- **`md encode` does not consume a descriptor.** Handed a concrete output
  descriptor it exits 1 with *"template parse error: template contains no @i
  placeholders"*. It consumes a **BIP-388 template** plus `--key`/`--fingerprint`
  flags, and nothing in §6 gives it descriptor parsing.
- **`mk encode` does not consume a bare xpub, and needs a policy binding no
  `.xpub` file carries.** A bare xpub in a key file exits 64 — *"expected BIP-380
  origin notation `[fingerprint/path]xpub`"*. With a valid xpub and origin but no
  binding it exits 64 — *"at least one of --policy-id-stub or --from-md1 is
  required"*.
- **`md` and `mk` are therefore NOT independent producers.** A real cosigner card
  is bound to the descriptor's `md1` set through repeated `--from-md1`, so the
  two cannot sit side by side in one brace group. That is a sequencing fact, not
  a flaw, and the acceptance form now shows it.
- Only the `mt` line worked, and it is the one this spec did not need to change.

**The criterion below is written from what the tools consume.** Per the project
rule, *a gate that has never executed is a hypothesis, not a gate* — **P4 does
not close until this has been RUN**, with no `grep` and no `--group-size`.

Stage 1, because the cosigner card is derived from the descriptor:

```
md encode --in wallet.template --out wallet.md1
```

Stage 2, one group, one pack:

```
{
  cat wallet.md1
  mk encode --in cosigner1.keys --from-md1-set wallet.md1
  mt encode --qr --in tx.hex
} | me sysw pack --expect descriptor,cosigner,transaction --out payload.bin
```

- `wallet.template` holds a BIP-388 template — what `md encode` actually eats.
- `cosigner1.keys` holds BIP-380 origin notation, one record per line — the form
  `mk encode --keys` already requires today.
- **`--from-md1-set FILE` is the one piece of new surface this criterion
  introduces, and P3 owns it.** It is ergonomics, not capability: M-6 proved that
  repeating `--from-md1` across a 4-chunk set already works. Without it the
  binding needs a shell loop to expand the repeated flag — which would be the
  third workaround this spec exists to remove.
- `--expect` is what makes the group's exit status mean something (§6g).

**And the acceptance has a second half, which is the negative case.** The cycle
is not done until the same pipeline, with one producer made to refuse, **fails**
— exits non-zero and writes no payload. The C-1 reproduction is the test: today
those two runs are indistinguishable at exit 0.

Finally, the cycle is done when `ms encode --phrase "<a real seed>"` refuses,
names the exposure without echoing it, and prints the **exact** purge command
for the operator's shell (§6h).
