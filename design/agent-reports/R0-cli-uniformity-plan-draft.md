# R0 plan-authoring round — `SPEC_constellation_cli_uniformity.md` @ `dcea954`

**Lens:** *write P0's implementation plan from this spec alone, and report where
the spec stopped answering.* Not a correctness pass — four of those have run.
The question is **sufficiency as an instruction**.

**Verdict: NOT GREEN — 3 Critical / 5 Important / 7 Minor / 1 Nit.**

Everything below was measured against built binaries and real source during this
round. Where a number appears, the command that produced it appears with it.

---

# PART A — did the fold damage anything?

`dcea954` rewrote all five rows of §7's phase table. Per-item disposition of the
final round's six Importants:

| | disposition | one line |
| --- | --- | --- |
| **I-1** | **CLOSED** | `ms`'s grouping + separator now in P2's content, with a gate clause (`ms encode --in <file>` into `me sysw pack`, no flags, exit 0) that fails today because `ms --in` does not exist. |
| **I-2** | **PARTIAL** | P2's row no longer schedules building `-` on `combine`, and the unfailable gate clause is gone — but **four other sites still assert the false claim**, and one of them is the normative per-verb channel table. See A-1. |
| **I-3** | **PARTIAL** | P1's row and gate now own `-` on `decode`/`verify`/`inspect` — but §7:932 still reads *"Its diff is confined to the two rulings above"*, which the same fold made false, and §6b:307 still excludes `mt` from *"the real `-` gap"*. See A-1. |
| **I-4** | **CLOSED** | `mk`'s `2 → 1` is in P3's content and P3's gate gained a `mk` clause that fails today (measured: `mk inspect notanartifact` → **2**). |
| **I-5** | **CLOSED** | P2's gate now asserts `ms encode --phrase <phrase>` refuses and `--allow-argv-secret` proceeds; that clause fails today (measured: exit **0**). Residual Minor A-2. |
| **I-6** | **CLOSED** | `--expect` has one owner (P0 builds it in full, including §6g's incomplete-set refusal); P4's row says so explicitly. Residual Minor A-3. |

## A-1 (Important) — the repo's own propagation gate is RED on this fold

`scripts/fold-propagation-check.sh` exists in this repo precisely for this, and
the fold did not run it. Run now:

```
$ ./scripts/fold-propagation-check.sh design/SPEC_constellation_cli_uniformity.md \
    'not .combine.' 'positionals ONLY' 'ordering constraint that makes P2 non-negotiable' \
    'real .-. gap' 'confined to the two rulings'
  LEFT   not .combine.                                    61
  LEFT   positionals ONLY                                467
  LEFT   ordering constraint that makes P2 non-negotiable 470
  LEFT   real .-. gap                                    307
  LEFT   confined to the two rulings                     932
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.
```

All five hits are **live assertions**, not retraction context:

- **§2:61** — `` `-` for stdin … `ms` … 7 of 8 verbs — **not `combine`** ``. False.
- **§6b:307-308** — *"the real `-` gap is `md`'s four other verbs and `ms combine`
  — two targeted additions"*. False twice over: `ms combine` is not a gap (I-2)
  and `mt`'s three verbs are (I-3). The real gap is `md`×4 + `mt`×3.
- **§6d:467** — the per-verb channel table: `` `combine` | **positionals ONLY —
  no `--in`, no `-`** | `-` and `--in` FIRST, then refuse ``. This is the table a
  plan author enumerates `ms`'s work from.
- **§6d:470-476** — the *"ordering constraint that makes P2 non-negotiable"*
  paragraph, still ending *"§7 P2 is ordered accordingly"* — an ordering P2's row
  no longer states.
- **§7:932** — *"Its diff is confined to the two rulings above"*, one paragraph
  below a P1 row the same fold gave a third ruling.

**The fold also created a new orphan of exactly the I-1 shape it was fixing.**
P2's content dropped *"`-` on `combine`"*; §6d:467's "after D3" column still
requires *"`-` and `--in` FIRST"*. If the document's four surviving claims are
believed, that `-` work is now owned by no phase. If the fold is believed, four
sites are wrong. A plan author cannot tell which, and §6d:467 is the site they
will read.

**Cost of getting it wrong:** an implementer enumerating P2 from §6d builds a
stdin channel that already ships (one wasted slice), and one reading P1 from
§7:932 under-scopes `mt`. Both are recoverable in-phase. Important, not Critical.

## A-2 (Minor) — I-5's gate closes the harm but covers 1 of `ms`'s 8 verbs
§6d's table rules all eight verbs refuse argv. P2's new gate names only
`ms encode --phrase`. P3's sibling clause is *"named one by one"*. The content is
unambiguous, so an implementer building from §6d builds all eight; only the gate
is narrower than its sibling.

## A-3 (Minor) — §2a still does not name `me` as an affected CLI
The fold gave P0 a flag in `me`'s own binary. §2a — the scope statement §7 is
required to cover — still says this repo is affected *"through its committed
journey drivers"* and points at P2/P3. The final round asked for this in I-6's
remedy; the fold took the phase-table half only.

**No structural damage.** Re-run at `dcea954` this round: `spec-structure-check.sh`
→ *sections: 21 ; cross-refs checked: 16 ; STRUCTURE OK*; `plan-table-check.sh` →
*table rows checked: 58 ; malformed: 0*; all five phase rows
are 3 cells; no cell contains a bare pipe. Nothing orphaned, nothing dropped.

---

# PART B — the plan-authoring log

In the order I hit them, writing P0.

## B-1 (CRITICAL) — `--expect transaction`: does it mean `Class::Mt` or `Class::Tx`? Both come out of `mt encode`, and the spec's two worked examples contradict its own narrative

**The question.** §6g requires the kind vocabulary to be *"fixed and enumerated
in the plan"* and to *"map onto exactly one of those two discriminants per
kind"*. I got as far as `descriptor` (HRP `'d'`) and `cosigner` (HRP `'k'`) and
then could not write the row for `transaction`.

**What the spec says instead.** It uses the token `transaction` three times and
never binds it. Measured — `mt encode` has two mutually exclusive output forms,
and they classify differently:

```
$ mt encode --help | head -3
Default: `mt1` strings — to engrave by hand, or to pipe into `me sysw pack` as
bare records for text plates. `--qr`: a `tx:` record carrying the transaction's
bytes, for QR plates.
```

`crates/me-cli/src/sysw/record.rs:44` — `Class::Mt` (an `mt1` chunk) and
`Class::Tx` (a `tx:` record) are separate variants with separate doc comments.
`classify_with` produces both.

- §10's acceptance criterion — the spec's one executable gate, of which it says
  *"P4 does not close until this has been RUN"* — pipes **`mt encode --qr`**, i.e.
  `Class::Tx`.
- §6g's C-1 reproduction also uses `--qr`, i.e. `Class::Tx`.
- §1, §2, §3, §4, §6a, §6c and P1 are about the **`mt1`** path, i.e. `Class::Mt`.
  §6a's table gives `mt encode` stdout as *"the artifact"*, which is `mt1`.

**What I had to do to proceed.** Guess. There is no reading that leaves the
document consistent:

| reading | what breaks |
| --- | --- |
| `transaction` = `Class::Tx` | An operator on the **default** (hand-engrave) path who passes `--expect …,transaction` gets a **false refusal on a correct, complete payload**. C-1 — the defect D6 exists to close — stays open on the path the rest of the spec is about. |
| `transaction` = `Class::Mt` | **§10's acceptance criterion fails.** Its `mt encode --qr` line emits a `tx:` record, `--expect …,transaction` finds no `Class::Mt`, and `pack` refuses. The spec's only run-it-to-close gate is unsatisfiable as written. |
| either satisfies it | `--expect transaction` cannot tell a QR plate from a text plate. An operator who meant `--qr` and got `mt1` (or the reverse) packs the wrong plate type and is not caught — the silently-wrong-payload class C-1 exists to close. |

**Cost if guessed wrong.** The kind vocabulary is P0's public surface, shipped in
a released crate under whatever distribution mechanism P0 picks. The wrong guess
surfaces at **P4**, when the journey capture is run against §10 — four phases and
a release later. Fixing it changes the vocabulary (a semver event), `me sysw
pack`'s flag semantics, and P4's captured journey. **One sentence in §6g fixes it
now.** This is the costliest thing I found.

## B-2 (CRITICAL) — the argv guard's DETECTOR is unspecified, and neither of the two sources P0 is told to extract from has one that can work

**The question.** What input does the guard inspect to decide a token is secret?

**What the spec says instead.** §6d specifies the guard's *placement*
(*"runs on raw `std::env::args()` BEFORE any argument parser sees the material.
This is NORMATIVE"*), its *output* (*"Report the CLASS and the LENGTH, not the
value"*), and its *override*. It never specifies the predicate. §5 says
*"**P0 extracts that code; it does not re-derive it**"*, and §7 P0 says
*"Extracted FROM `mt`/`me`"*.

**What I found when I went to extract it.** The two named sources implement two
*different* mechanisms, and neither generalises:

- **`mt` — pre-clap, SHAPE-based.** `mt-cli/src/main.rs:233-238` calls
  `validate::command_line_guard(&argv)` before `Cli::parse()`. The predicate is
  `looks_like_a_transaction(a)` (`validate.rs:503`): the `cHNidP8` PSBT prefix,
  or an `mt1` string. It matches on the *shape of the material*.
- **`me` — POST-clap, class-based.** The widened refusal §6d calls *"the
  reference"* runs inside `run_sysw`, on the clap-parsed `records: &[String]`
  (`me-cli/src/main.rs:1103-1127` → `read_records`, `main.rs:1932-1966`). It
  is **not** on raw `std::env::args()`. Its own comment says *"THIS RUNS BEFORE
  ANYTHING ELSE `pack` DOES"* — before anything *`pack`* does, after clap. It
  works only because `me`'s records are self-typed (`tx:`, `pass:`, an `ms1`
  HRP), which is the thing that makes a class computable from the token alone.

**So the widened class union (from `me`) exists only post-clap, and the
normative pre-clap placement (from `mt`) exists only with a narrow shape
predicate. The intersection the spec asks for is not present in either.**

**And a shape predicate provably cannot cover three of the five CLIs' channels.**
Measured:

```
$ mnemonic bundle --help | grep -A1 -- '--passphrase <'
      --passphrase <PASSPHRASE>
          BIP-39 mnemonic-extension passphrase ("25th word") …
$ mnemonic electrum-decrypt --help | grep -- '--decrypt-password <'
      --decrypt-password <VALUE>      Decryption password (inline) …
```

A passphrase and a decryption password are **arbitrary text with no shape**. No
`looks_like_*` predicate can recognise them. The only workable detector is
**flag-name-driven** — "the token after `--passphrase` on subcommand `bundle` is
secret" — which means a pre-clap guard that re-implements enough of clap to
handle `--flag value`, `--flag=value`, short forms and subcommand context, and
that carries a per-CLI table of secret-bearing flags. That is the crate's central
data structure. **The spec does not mention it, and calls the whole thing an
extraction.**

The reverse error is equally live: a naive hex-shape detector would refuse
`mk encode --origin-fingerprint 11223344` and `--policy-id-stub 11223344`, which
are legitimate watch-only flag values §4 explicitly protects.

**Cost if guessed wrong.** An implementer extracts `mt`'s shape guard (it is the
self-contained one, and it is the one that satisfies C-4's normative ordering),
ships the crate, and P1 and P2 adopt it. The guard **silently never fires** for
`mnemonic`'s passphrase channels. The failure is first detectable at **P3's**
gate. The fix changes the crate's public API — the guard needs a flag table as an
argument — which invalidates P1's and P2's merged adoption and forces a
re-release. Multi-phase rework, three phases downstream.

## B-3 (CRITICAL) — `mnemonic`'s secret-on-argv surface is 48 call sites across 20 files; §7 names five channels and calls that enumeration the safeguard

**The question.** P0's guard needs the list of secret-bearing argv sites it must
cover. §7 gives five for `mnemonic`. Is that the list?

**What the spec says instead.** §7: *"**Five channels carry secret material**…
`bundle`, `convert`, `derive-child`, `restore --passphrase`, and
`electrum-decrypt --decrypt-password`. They are named in P3's row rather than
left to 'the `mnemonic` work', **because a phase item that does not enumerate its
sites is one a later reader satisfies by doing less**."* P3's gate: *"`mnemonic`'s
five secret-material argv channels each refused, named one by one."*

**Measured.** `mnemonic-toolkit` already ships a complete argv-secret subsystem
the spec never mentions — `src/secret_advisory.rs::secret_in_argv_warning(stderr,
flag, alternative)`, backed by `src/secret_taxonomy.rs` (`SECRET_NODE_TYPES_ARGV`,
kept in lockstep with `NodeType::is_argv_secret_bearing` at `cmd/convert.rs:117`
by a named parity test) and `src/process_hardening.rs` (PR_SET_DUMPABLE):

```
$ grep -rn 'secret_in_argv_warning(' crates/mnemonic-toolkit/src \
    | grep -v src/secret_advisory.rs | grep -v '^.*use ' | wc -l
48
$ grep -rln 'secret_in_argv_warning(' crates/mnemonic-toolkit/src \
    | grep -v secret_advisory | wc -l
20
```

The 20 files are `addresses, bundle, convert, derive_child, electrum_decrypt,
final_word, import_wallet, ms_shares, nostr, restore, seedqr, seed_xor,
silent_payment, slip39, verify_bundle, xpub_search/{account_of_descriptor,
passphrase_of_xpub, path_of_xpub, seed_intake}, repair` — roughly **18 of
`mnemonic`'s 26 subcommands**, against the spec's five.

Two of the 48 offer an **environment variable** as the private channel
(`import_wallet.rs:297,301` → `@env:VAR`), not stdin or a file — so §6d's
required refusal text (*"Name the private channels: `--in FILE`, `-` for
stdin"*) has no correct form at those sites. The fold's *"RECORDED SO IT IS NOT
RE-RUN"* note — *"All five already ship stdin or file alternatives"* — is true of
the five and does not generalise to the 48.

**Cost if guessed wrong.** P3's gate passes with five channels refused and ~13
subcommands still accepting seed phrases, xprvs, WIFs, `ms1` strings and BIP-38
passwords on argv at exit 0 with a warning. The cycle closes GREEN having shipped
§1's motivating finding — *"the tool holding the most dangerous material has the
weakest handling of it"* — mostly intact, on the tool the spec added last and
enumerated least. This is an unmet guarantee of D3, and it is one `grep` away
from being known.

**It also inverts D5's rationale.** D5 exists because *"four copies of one rule is
the shape that let `pack` and `pack_deterministic` drift"*. `mnemonic-toolkit`
already owns a fifth copy — with a lockstep parity test proving the drift risk is
live and already managed — and P0 is instructed to build a sixth without knowing
about it. `secret_taxonomy.rs` is the natural donor for the crate's predicate and
`secret_advisory.rs::secret_in_argv_warning`'s `(flag, alternative)` signature is
the flag-table shape B-2 needs. The spec names `mt` and `me` and not this.

## B-4 (Important) — `mt`'s shipped purge text VIOLATES §6h in two of four branches, and P0 is told to extract from `mt`

**The question.** §7 P0: *"remedy text per §6h … Extracted FROM `mt`/`me`."*
Which one?

**What the spec says instead.** §6h names `me` (*"The reference implementation is
`me sysw pack`'s widened argv refusal"*). §7 P0 names both and does not say they
differ. They do:

```
mt-cli/src/validate.rs:541-548           me-cli/src/main.rs:2014-2017
  zsh  → history -d $HISTCMD && fc -W      bash/zsh → sed -i '/me sysw pack/d' "$HISTFILE"
  fish → history delete --contains <tx>    fish     → history delete --prefix 'me sysw pack'
  bash → history -d $HISTCMD && history -w (plus an explicit note that zsh's
                                            `history -d` does NOT delete)
```

`mt`'s zsh branch is the exact trap §6h forbids by name: *"Do not tell a zsh user
to run the history builtin with `-d` … `-d` is a **display** flag … Advising it
would report success while purging nothing."* `mt`'s fish branch anchors on the
material (`--contains <tx>`), which §6h forbids because *"anchoring the pattern
on the material types it into history a **second** time."*

**What I had to do to proceed.** Read both, notice the conflict, and take `me`'s.
Nothing in §7 P0 tells an implementer to.

**Cost if guessed wrong.** `mt`'s `purge_command()` is a self-contained
`fn() -> &'static str` — the obvious thing to lift; `me`'s is embedded in a
refusal block. **No test in `mnemonic-transaction` asserts the purge text**
(`grep -rn HISTCMD crates/` → source only), so P1's gate — *"`mt`'s 237 tests
pass"* — passes either way, and P0's *"its own tests"* would assert whatever
string was extracted. **No gate in the plan can catch this.** After P2, the
material being not-purged is a BIP-39 seed phrase. Important rather than Critical
only because §6h resolves it for a reader who notices the conflict.

## B-5 (Important) — the crate has no name, no home repo, and no stated consumer set

Three questions I could not answer, all at step 1:

- **Name.** The spec says *"the shared crate"* (§7 P0) and *"One shared crate"*
  (D5). No name is proposed anywhere in 1,132 lines.
- **Home.** No repo is named. Every option has consequences the spec does not
  weigh: a **new repo** needs CI, branch protection, licence headers and a
  release workflow, none of which is in P0's content or gate; **inside
  `mnemonic-engrave`** inverts the dependency (four codec CLIs would depend on
  the engraver, whose package is `mnemonic-engrave` — note the package name is
  not `me-cli`, which is only the directory); **inside `mnemonic-toolkit`** is
  where the argv taxonomy of B-3 already lives.
- **Consumers.** D5 says *"depended on by all five"* — md, mk, ms, mt, mnemonic.
  §7 P0 says *"Extracted FROM `mt`/`me`"*. **Is `me` a consumer or only a
  donor?** If it is only a donor, `write_private` and `is_argv_forbidden` exist
  in two copies on the day the crate ships, which is the exact condition D5 was
  written to prevent. The spec never says.

**Distribution** is the one part the spec *deliberately* hands over
(*"**P0 must name which mechanism it uses**"*), and it eliminates enough to be
decidable: `path =` is out (N-1 — `mnemonic-transaction` exists at two locations,
plus the fresh-CI-checkout argument already in `me`'s Cargo.toml), and a rev pin
*"exists precisely to avoid a release step, and D5 needs one"*. That leaves
crates.io. But then **P0 contains a `cargo publish`** — irreversible, therefore
risk-set by this project's own CLAUDE.md — and P0's gate has no clause for it, no
name reservation step, no version policy, and no statement of what P1/P2/P3 do
when they need a crate change (a new release each time, four lockfiles).

**Cost if guessed wrong.** The name is baked into five `Cargo.toml`s, every
`use`, and an irreversible registry publish. Renaming after P1 and P2 land is
five repos plus a yank.

## B-6 (Important) — filling the two `mnemonic` exit cells falsifies §6f's own ruling, and the spec does not say which command fills them

P0's gate: *"the two `mnemonic` exit cells still marked 'not measured' filled"*
— invalid-artifact and repair-uncorrectable. I filled them.

```
$ mnemonic inspect notanartifact ; echo $status
error: positional argument 'notanartifac…' does not begin with a recognized HRP
prefix (expected one of: ms1, mk1, md1)
2
$ mnemonic repair md1zzzzzzzz8xtwhw4xwn4qh ; echo $status
error: repair: chunk 0 has too many errors to correct uniquely …
2
```

so **invalid-artifact = 2, repair-uncorrectable = 2**. Against the same input:

```
$ md inspect notanartifact → 1     $ mk inspect notanartifact → 2
$ ms inspect notanartifact → 1
```

**`mnemonic` diverges exactly the way `mk` does.** §6f says *"`mk`'s
invalid-artifact 2 becomes 1 … **This is the only code this cycle changes**"* and
§9 lists no `mnemonic` exit-code work. So satisfying P0's gate falsifies a
normative ruling two sections up, and no phase owns the consequence. The spec
placed a fact-finding item in a gate whose likely answer contradicts the section
that ordered it, and specified no response.

**And which command fills the cell is undefined.** A second plausible reading of
"invalid artifact" gives a different number:

```
$ mnemonic convert --from ms1=notanartifact ; echo $status
64
```

2 or 64 depending on the verb chosen. The spec never defines "invalid artifact"
operationally per CLI, and the row for `md` in the same table (1) came from a
verb it does not name either.

## B-7 (Important) — `--out` clobber is unruled, and `write_private`'s acceptance argument does not transfer out of `me`

§6b: *"`--out FILE` — write the artifact to a file, **created 0600 by `me`'s
`write_private`**, never `std::fs::write` (F-244)."* That is the whole ruling.

`me-cli/src/main.rs:856` — `write_private` opens with `.create(true)
.truncate(true)`, i.e. it **silently destroys an existing file**. Its doc comment
accepts that residual with a justification that is explicitly scoped to `me`'s
own targets: *"NDEF/manifest targets are user-named; preview targets are
forced-fresh by the dirty-dir refusal."*

Neither condition holds for `ms encode --out seed.ms1`, `mt encode --out tx.mt1`,
or §10's `me sysw pack --out payload.bin` re-run with a different input. §6e's
write gate covers world-readable and terminal, not clobber.

**This is the same defect shape §6e already caught once.** §6e retracts the
terminal-gate lift because *"`me`'s refusal states a reason that is specific to a
binary container … the predicate is false for all four CLIs."* `write_private`'s
clobber acceptance is specific in exactly the same way, and was lifted anyway.

**Cost if guessed wrong.** An implementer lifts the function verbatim — which is
what §6b instructs — and every `--out` in the constellation overwrites without
asking, on a channel whose entire reason for existing is handling material that
cannot be regenerated.

## B-8 (Minor) — "exit codes" as a P0 deliverable has no definable content
§6f freezes the repair codes at values that deliberately diverge (`md` 5, `ms` 4,
`mnemonic` 4), records the clap 2-vs-64 split as *"not resolved here"*, and gives
the one change (`mk` 2→1) to **P3**. So a crate that "owns exit codes" can export
constants but cannot own the mapping, and the item is vestigial. Low cost — an
implementer defines something plausible and the phase closes.

## B-9 (Minor) — does the crate own the clap flags? and against what toolchain?
D5 rejects *"a pure-logic crate with per-repo flag wiring"*, so the crate exports
clap types — but §6d rules the guard and **the override's own parse** must run
before clap, so `--allow-argv-secret` cannot be one of them. The plan must say
which flags are clap `Args` and which are raw-argv. Facts I had to measure rather
than read: all five CLIs are `clap 4` with `derive` (`md-cli` pins `4.5`, the rest
`4`); `rust-version = 1.85` in md/mk/ms/mt/toolkit and **absent** in
`mnemonic-engrave`; editions split 2024 (md/mk/mt) vs 2021 (ms/toolkit/engrave).

## B-10 (Minor) — the in-memory-history gate item has no success criterion
§6h: *"**NOT YET VERIFIED** … **No command for it is stated here because none has
been verified**; P0 owes the measurement."* P0's gate: *"the in-memory-history
question of §6h measured"*. If the measurement returns *"no reliable command
exists for fish"*, P0 is green by the letter of the gate and §6h's *"remedy text
must be executable"* rule keeps a known unmet case with no owner.

## B-11 (Minor) — `descriptor` resolves only by reading source; `Class::Descriptor` is a dead variant
`--expect descriptor` looks ambiguous between HRP `'d'` and `Class::Descriptor`.
It is not, but only source says so: `classify` never produces `Class::Descriptor`
(`sysw/mod.rs:173` — *"Descriptor and Address are deliberately absent … classifying
them needs a descriptor parser"*). §6g should say the variant is unreachable, or
the plan author checks it the way I did.

## B-12 (Minor) — `--in` together with argv material: unruled
`me` refuses (`main.rs:534`, *"pass records on argv OR via --in, not both"*).
Does the crate generalise that? At which exit code? §6b lists the three channels
and never states their interaction, and `ms`'s eight verbs each have a positional
*and* will gain `--in`.

## B-13 (Nit) — P0's gate says *"an R0 round closing 0C/0I"* — over which artifact?
P0's own plan document, or P0's code diff? The project's R0 gate is a pre-code
document gate; here it is listed as a completion condition after code exists.

---

# PART C — the P0 plan I could actually write

Everything marked **[GUESS]** is mine, not the spec's. Everything marked
**[BLOCKED]** could not be written at all.

## C.0 Crate identity

| | value | source |
| --- | --- | --- |
| name | `m-cli-io` | **[GUESS]** — no name exists in the spec (B-5) |
| home | new repo `shibboleth/m-cli-io`, workspace with one crate | **[GUESS]** (B-5) |
| edition / MSRV | 2021 / `rust-version = "1.85"` | measured: 2021 is the lower of the two editions in use; 1.85 is the declared MSRV of all four repos that declare one |
| deps | `clap 4` (derive) | measured across the five CLIs |
| distribution | crates.io, `0.1.0`, consumers take `= "0.1"` | **[GUESS by elimination]** — the spec eliminates `path =` and argues against the rev pin, but never rules. Carries an irreversible `cargo publish` inside P0 with no gate (B-5). |
| consumers | md-cli, mk-cli, ms-cli, mt-cli, mnemonic-toolkit **and** me-cli | **[GUESS]** — D5 says five, §7 says extract from `me`; I include `me` so the donor does not keep a second copy (B-5) |

## C.1 Public API

Signatures below are what the five consumers need. Provenance is the file and
symbol each is extracted from, measured this round.

```rust
// ── channels ────────────────────────────────────────────────────────────────
/// Resolve the input channel. `--in FILE` | `-` | bare stdin | argv positional.
pub enum Source { File(PathBuf), Stdin, Argv(String) }
pub fn resolve_input(in_flag: Option<&Path>, positional: Option<&str>) -> Result<Source, IoRefusal>;
//   FROM: mt-cli/src/main.rs `encode()`'s `match &args.r#in` arm (the --in half)
//         me-cli/src/main.rs `read_records` (the argv-XOR---in refusal)
//   [GUESS] the argv-XOR---in refusal generalises; §6b never rules (B-12)

/// Create 0600 and write. Truncates.
pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()>;
//   FROM: me-cli/src/main.rs:856 — verbatim, incl. the post-open re-chmod (F-244)
//   [BLOCKED] clobber policy unruled (B-7). Written as-is = silent destroy.

// ── write gate ──────────────────────────────────────────────────────────────
pub fn stdout_world_readable_mode() -> Option<u32>;
//   FROM: me-cli/src/main.rs:896 (unix) / :921 (non-unix) — fstat on fd 1.
//   Keyed on MODE BITS, not S_ISREG, and char devices are exempt: both are
//   load-bearing per its own comment. Lift verbatim, both cfg arms.
pub fn file_mode_warning(path: Option<&Path>) -> Option<Warning>;
//   FROM: mt-cli/src/validate.rs:561 — verbatim (F-252 wording: own mode only)
//   NOT extracted: me's terminal gate. §6e RETRACTS the lift; it stays in `me`.

// ── argv guard ──────────────────────────────────────────────────────────────
pub fn command_line_guard(argv: &[String], /* ??? */) -> Result<(), Refusal>;
//   FROM: mt-cli/src/main.rs:233 (placement) + mt-cli/src/validate.rs:448 (body)
//   [BLOCKED] — the `???` is the detector. B-2. `mt`'s is shape-based and cannot
//   see `mnemonic bundle --passphrase <text>`; `me`'s is class-based and runs
//   POST-clap, so it does not satisfy C-4. The signature cannot be written until
//   §6d says what the predicate is. My best reconstruction, unsupported by the
//   spec, is a caller-supplied table:
//       pub struct SecretFlags { pub by_verb: &'static [(&'static str, &'static [&'static str])] }
//   modelled on mnemonic-toolkit's shipped `secret_in_argv_warning(stderr, flag,
//   alternative)` — the closest working precedent, which the spec never names.

pub fn allow_argv_secret(argv: &[String]) -> bool;
//   Raw-argv parse, NOT a clap flag — §6d rules that wiring it through clap
//   reinstates the echo. FROM: nothing; me's is `#[arg(long)]` (main.rs:252).
//   This is NEW code, not an extraction.

pub fn is_argv_forbidden(c: Class) -> bool;   // = is_secret() || is_bearer()
//   FROM: me-cli/src/sysw/record.rs:105

// ── remedy text ─────────────────────────────────────────────────────────────
pub fn purge_command(command_name: &str) -> String;
//   FROM: me-cli/src/main.rs:2014-2017 — NOT mt-cli/src/validate.rs:541 (B-4).
//   Takes the command NAME so the pattern anchors on it, never on the material.
//   [BLOCKED] the in-memory-history clause (§6h final bullet, B-10).

// ── exit codes ──────────────────────────────────────────────────────────────
// [BLOCKED] B-8 — no definable content. Constants only, no mapping.
```

## C.2 `me sysw pack --expect` (P0's second deliverable — `me`-only, not the crate)

```rust
pub enum Kind { Descriptor, Cosigner, Transaction, Seed, Passphrase, Text, Address }
fn kind_of(record: &str) -> Option<Kind>;
//   descriptor -> chunk_key(r, RecordKind::Md).0 == 'd'   [me-cli/src/sysw/record.rs:236]
//   cosigner   -> chunk_key(r, RecordKind::Mk).0 == 'k'
//   transaction-> [BLOCKED — B-1] Class::Mt? Class::Tx? both?
//   seed       -> Class::Mnemonic | Class::Codex32Secret   [GUESS: name unstated]
//   passphrase -> Class::Passphrase                        [GUESS: name unstated]
//   text       -> Class::FreeText                          [GUESS: name unstated]
//   address    -> Class::Address                           [GUESS: name unstated]
//   (Class::Descriptor and Class::Unknown are unreachable / not a kind — B-11)
```

Incomplete-set refusal reuses `sysw::record::mdmk_unconfirmed(&[String]) -> Vec<usize>`
(`record.rs:168`) exactly as §6g directs — escalate its report to a refusal for
named kinds, no second walk.

**Four of seven kind names are mine.** §6g requires the vocabulary to be *"fixed
and enumerated in the plan, not invented per call site"*; the spec pins three by
usage (`descriptor`, `cosigner`, `transaction`) and I invented the rest.

## C.3 Test list

| # | asserts | can it fail today? |
| --- | --- | --- |
| T1 | `write_private` creates 0600; overwriting a 0644 target ends 0600 | yes (regression guard on the F-244 re-chmod) |
| T2 | `resolve_input` rejects `--in` + argv positional together | yes |
| T3 | `stdout_world_readable_mode` returns the mode for a 0644 redirect, `None` for a pipe/tty | yes |
| T4 | `file_mode_warning` names only the file's own mode (F-252 wording) | yes |
| T5 | the guard refuses a BIP-39 phrase on argv, naming class + length, **never the value** | **[BLOCKED — B-2]** cannot write the call |
| T6 | the guard refuses `mnemonic bundle --passphrase <text>` | **[BLOCKED — B-2]** no detector exists for it |
| T7 | `--allow-argv-secret` proceeds, parsed from raw argv, with clap never reached | yes |
| T8 | `purge_command("ms encode")` anchors on the command name, and the zsh branch does not advise `history -d` | yes — and it fails against `mt`'s text (B-4) |
| T9 | `--expect descriptor,transaction` refuses a stream with no transaction | **[BLOCKED — B-1]** the fixture depends on the mapping |
| T10 | `--expect descriptor` refuses 1 of a 2-chunk `md1` set | yes (today: exit 0 with a warning) |
| T11 | `mnemonic` exit cells, invalid-artifact and repair-uncorrectable | **done this round: 2 and 2** — and it falsifies §6f (B-6) |
| T12 | the in-memory-history recipe | **[BLOCKED — B-10]** no success criterion |

## C.4 Order of work, and what "done" is

1. **Decide crate identity** (name / home / consumers / distribution) — **[GUESS]**,
   all four. Done = five Cargo.tomls resolve and `cargo build` is green in each.
   *Contains an irreversible `cargo publish` with no gate.*
2. **Lift the mechanical half** — `write_private`, `file_mode_warning`,
   `stdout_world_readable_mode`, `is_argv_forbidden`, `resolve_input`. Done = T1–T4.
   This is the only step of P0 that is genuinely an extraction.
3. **Remedy text** — from `me`, not `mt`. Done = T8. *Blocked on T12.*
4. **Argv guard.** **[BLOCKED]** — cannot begin. Needs §6d to state the detector.
5. **`me sysw pack --expect`.** Partially blocked — the incomplete-set half (T10)
   is writable today; the vocabulary (T9) is not.
6. **Fill the `mnemonic` cells.** Done above; the result needs a §6f ruling.

**Three of six steps are blocked on the document, and one of the three blocked is
the crate's reason for existing.**

---

# Counts

| severity | items |
| --- | --- |
| **Critical (3)** | B-1 `--expect transaction` unbound between `Class::Mt` and `Class::Tx`; B-2 the argv guard's detector is unspecified and unextractable; B-3 `mnemonic`'s argv surface is 48 sites / 20 files, not 5 |
| **Important (5)** | A-1 five superseded claims survive the fold (repo's own propagation gate is RED); B-4 `mt`'s purge text violates §6h and P0 is told to extract from `mt`; B-5 the crate has no name, home or consumer set; B-6 filling P0's `mnemonic` exit cells falsifies §6f; B-7 `--out` clobber unruled and `write_private`'s acceptance argument does not transfer |
| **Minor (7)** | A-2, A-3, B-8, B-9, B-10, B-11, B-12 |
| **Nit (1)** | B-13 |

**VERDICT: NOT GREEN (3C / 5I).**

**The single costliest gap is B-1.** It is one sentence in §6g, it is undecidable
from the document today, every reading falsifies something the spec asserts, and
the wrong choice is first detectable at **P4** — after the vocabulary has shipped
in a released crate and been consumed by the acceptance criterion the spec says
must be RUN before the cycle closes.

**The fold itself introduced no structural damage** and closed four of six
Importants cleanly. Its defect is the one this project has recorded before and
built a script for: the facts were corrected where the reviewer pointed and left
standing four sections away. `scripts/fold-propagation-check.sh` reports it in
under a second.
