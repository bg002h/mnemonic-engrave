# FIX — F-410's two surviving items, in `md-cli`

Implementation report. Worktree:
`/scratch/code/shibboleth/_work/f410/descriptor-mnemonic`, branch
`fix/f410-md-cli`, parent `fad69f1f`, commit **`5465253b`**. NOT pushed. The
live checkout at `/scratch/code/shibboleth/descriptor-mnemonic` was not
touched, and neither was any other repo — the Go port is a separate piece of
work.

**`md-codec` is untouched.** Its wire format and derivation were correct as
they stand; both items are in `md-cli`. Every number and every command output
below was measured in this session with `target/debug/md` invoked by absolute
path (the login shell aliases `md` to `mkdir -p`, which exits 0 while silently
creating directories).

## Gates

| gate | command | result |
| --- | --- | --- |
| suite | `cargo nextest run --locked` | `Summary [ 20.172s] 850 tests run: 850 passed, 2 skipped` |
| lint | `cargo clippy --all-targets --locked -- -D warnings` | exit **0** |
| format | `cargo fmt --check` | exit **0** |

Baseline was 832 passed / 2 skipped; **+18 tests**, all new here (11 for item 1
— 4 lexer unit tests plus 7 CLI tests — and 7 for item 2).

Diff is 4 files, 512 insertions, 2 deletions:

```
crates/md-cli/src/cmd/encode.rs                   |  88 ++++++++++++
crates/md-cli/src/parse/template.rs               | 132 +++++++++++++++++-
crates/md-cli/tests/cli_bip388_double_wildcard.rs | 135 ++++++++++++++++++
crates/md-cli/tests/cli_unhardened_origin_note.rs | 159 ++++++++++++++++++++++
```

## Item 1 — `@i/**` accepted as sugar for `@i/<0;1>/*`

### Before / after

```
$ md encode --group-size 0 'wpkh(@0/**)'          # BEFORE (fad69f1f)
exit=1
md: template parse error: @0: derivation steps after the multipath group are not
    representable in md1; the multipath `<…>` must be the final derivation step
    before the wildcard

$ md encode --group-size 0 'wpkh(@0/**)'          # AFTER (5465253b)
exit=0
md1yqpqqxqq8xtwhw4xwn4qh
```

The ruling called the refusal a lexer accident and it is exactly that. The
placeholder regex's group 4 (`/\*(?:'|h)?`) consumes `/*` and leaves the second
`*` as unconsumed path residue, so M5's post-multipath residue check — which
exists to refuse a fixed step AFTER a multipath group — fires on a template that
has no multipath group at all. Hence a message wrong on its face: `/**` contains
no derivation steps.

### Byte-identity proof

The acceptance criterion is equality, not absence of an error. Both spellings
were run through the same binary under four flag settings; **stdout and stderr
were compared byte-for-byte** (`cmp`), 8 pairs, all exit 0:

| flags | stdout identical | stderr identical |
| --- | --- | --- |
| `--group-size 0` | yes | yes |
| `--group-size 5` | yes | yes |
| `--json` | yes | yes |
| `--group-size 5 --policy-id-fingerprint` | yes | yes |

`sha256(stdout)` for `--group-size 0`, both spellings:
`1971409c92fcd34d134fd796c7cb674efe0c1f6ea60112f95d1bab0f7007a237`.

Two independent corroborations, because equal bytes could in principle mean two
identically-wrong cards:

```
$ md decode md1yqpqqxqq8xtwhw4xwn4qh        # the card minted from `@0/**`
exit=0
wpkh(@0/<0;1>/*)

$ md address --index 0 --key @0=xpub6Bos…nMdj --template 'wpkh(@0/**)'
bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m
$ md address --index 0 --key @0=xpub6Bos…nMdj --template 'wpkh(@0/<0;1>/*)'
bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m
```

### How

Not by teaching two regexes a new pattern. **Two** passes read the raw template
— `lex_placeholders`, which decides the recorded `UseSitePath`, and
`substitute_synthetic`, which decides what miniscript parses — and they already
carry a standing "keep these in sync" obligation in their own comments. A card
whose use-site path and structural tree disagree is precisely the divergence
class M5's drift belt exists to catch.

So the fix desugars the text **once, upstream of both**:
`desugar_double_wildcard` rewrites a placeholder-terminal `/**` to `/<0;1>/*`
before either pass sees it. Byte-identity is then structural rather than a
property two patterns have to keep agreeing on. `terminates_placeholder` was
hoisted out of M5's own check so the desugarer and the residue reject cannot
disagree about where a placeholder stops.

### Not a general loosening

`/**` is sugar only when it **ends** the placeholder. These are left untouched
and keep hitting the residue reject:

```
wpkh(@0/**/0)      refused
wpkh(@0/**')       refused
wpkh(@0/<0;1>/**)  refused
```

The two genuine refusals in this neighbourhood are re-asserted **by name and by
reason** in the new tests, so a later reader cannot mistake `/**` acceptance for
either being relaxed:

```
$ md encode 'wpkh(@0/<2;3>/0/*)'   exit=1  "…must be the final derivation step…"
$ md encode "wpkh(@0/<0';1'>/*)"   exit=1  "…is hardened; hardened derivation is
                                             impossible on a watch-only (xpub) card"
```

## Item 2 — encode-time note on an all-unhardened placeholder origin

Warning tier. Never a refusal: the path after `@i` is the same grammatical slot
that carries every legitimate origin declaration, so refusing this shape would
refuse correct templates to catch a misreading.

### The exact text

One line, stderr, house style (`note: `, `\u{2014}` em dashes, the caller's own
path echoed rather than a canned example):

```
note: `/0` read as @0's key ORIGIN, not a derivation step from the provided key — the path after a placeholder IS that key's origin declaration, the same slot that carries `@0/48'/0'/0'/2'`. An origin with no hardened component is where that reading hides: it agrees with the pathless spelling while the key seated here is a MASTER xpub (unhardened steps commute) and DIVERGES for any other key, backing addresses one level above what a descriptor-style reading intends. The card is well-formed either way — confirm the xpub you seat for each slot named is the one its origin descends FROM.
```

Emitted once per run, not once per occurrence; when several slots qualify they
are listed in one line (`` `/0` read as @0's… ``; `` `/1` read as @2's… ``).

### Proof stdout and exit code are unchanged

Not asserted from the code's shape — measured against the binary built at
`fad69f1f`. The source change was stashed, the binary rebuilt, a fixed
12-template matrix captured, then the change restored and the matrix re-run.

- **stdout and exit code: `diff` EMPTY across all 12 templates.**
- **stderr: the only delta anywhere is one added `note:` line.**

The seven CLI tests pin the same thing durably: each stdout golden in
`cli_unhardened_origin_note.rs` was captured from the **pre-note** binary
(e.g. `wpkh(@0/0/*)` → `md1yqzqqqtk8nf99an9vzl`), so a note that ever leaked onto
stdout fails the suite.

### Which inputs trigger it

The predicate is `origin_path.is_some()` and every component unhardened —
measured over the matrix, 7 fire and 5 are silent:

| template | note |
| --- | --- |
| `wpkh(@0/0/*)` | fires |
| `wpkh(@0/0/1/*)` | fires (echoes `/0/1`) |
| `wpkh(@0/0/<0;1>/*)` | fires |
| `wsh(multi(2,@0/0/<0;1>/*,@1/48'/0'/0'/2'/<0;1>/*))` | fires, names **@0 only** |
| `wsh(or_d(pk(@0/0/<0;1>/*),pk(@0/0/<0;1>/*)))` | fires **once** |
| `tr(@0/1/<0;1>/*)` | fires |
| `sh(wpkh(@0/2/*))` | fires |
| `wpkh(@0/*)` | silent (no declaration to misread) |
| `wpkh(@0/<0;1>/*)` | silent |
| `wpkh(@0/84'/0'/0'/<0;1>/*)` | silent (hardened) |
| `wpkh(@0/84'/0'/0'/0/<0;1>/*)` | silent (mixed — see F-411) |
| `wsh(multi(2,@0/48'/0'/0'/2'/…,@1/48'/0'/0'/2'/…))` | silent |

Fires on both the text and the `--json` branch, matching every other `md encode`
advisory; a test pins the `--json` half, since that is where an advisory would
otherwise go quietly missing.

Keyed on the **template's own text**, not the final descriptor: it is the
spelling that gets misread, and `--path` replaces the declaration wholesale
rather than reinterpreting it.

## The ruling's premise, re-measured independently

Not taken on faith — reproduced on this binary, and it reproduces exactly,
including the address in the ruling:

```
$ md address --index 0 --key @0=xpub6Bos…nMdj --template 'wpkh(@0/0/*)'
bc1qr932kkqd95r3chv9sh36wkjez4jvsmlf46xuc9
$ md address --index 0 --key @0=xpub6Bos…nMdj --template 'wpkh(@0/*)'
bc1qr932kkqd95r3chv9sh36wkjez4jvsmlf46xuc9
```

That agreement is the whole reason item 2 is a note rather than nothing: it is
what lets the misreading survive a spot check.

## Filed, not fixed

**F-411** (`design/FOLLOWUPS.md`) — the note's predicate is *all*-unhardened, so
a **mixed** origin (`wpkh(@0/84'/0'/0'/0/*)`) and an all-unhardened `--path`
override are both silent, and the same divergence is reachable through each. The
ruling was explicit about the predicate, so this is recorded as a scope boundary
needing a decision, not widened by reflex. What is owed there is a rule that
separates a chain-level origin from a misread step; without one, widening would
fire on standard BIP-48/84 templates and destroy the note's value.

## Not done, deliberately

- Nothing outside these two items. No `md-codec` change, no `/**` handling
  anywhere the ruling did not name, no edits to neighbouring messages.
- **No CHANGELOG entry and no `--help`/README mention of `/**`.** Item 1 is a
  user-visible acceptance change and would normally earn both; the brief scoped
  this to two items, so it is flagged here rather than taken.
- Not pushed, and no Go port. `md-cli` is the Rust primary, so the port follows
  from this commit under the Rust-primary rule — the new CLI tests are the
  vectors it should be held to.
