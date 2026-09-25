# F-687 adversarial review — ms `e534917`, toolkit `9846a784`

Agent-written (Opus 5.5), 2026-09-25. Brief: `design/briefs/f687-review.md`.

**One question:** after this change, can any passphrase input in either CLI derive a wallet other than the one the user's bytes specify, or silently use an empty or wrong passphrase?

**Answer:** Through `--passphrase` itself, no. Every subcommand and every input mode I could reach agrees across all channels. Three narrow paths still give an unintended passphrase at exit 0:

- two of them are pre-existing and shared with `--passphrase-stdin` (M1, and item 4 below);
- one is an ms-only guard/resolver disagreement over a whitespace-padded `-` (M2).

None of them is new to F-687's resolver. The vector file has two real gaps (M3).

Counts: **0 Critical / 0 Important / 5 Minor / 4 Nit**, plus 1 item for operator ruling and 1 non-gating secret-handling follow-up.

Binaries built from the two SHAs (`cargo build --locked`, debug; toolkit reports `mnemonic 0.104.0`, ms reports `ms 0.19.1`), each with its own `CARGO_TARGET_DIR` under `/scratch/code/shibboleth/review-f687-scratch/`. The branches were not edited: `git status` is clean and HEADs are unchanged. Mutations ran on `git archive` copies.

---

## 1. Every passphrase-taking subcommand (enumerated from `--help`)

I enumerated by walking `--help` recursively on both binaries. Toolkit: 12 subcommands declare `--passphrase`: `addresses`, `restore`, `derive-child`, `bundle`, `verify-bundle`, `convert`, `silent-payment`, `slip39 split`, `slip39 combine`, `xpub-search path-of-xpub | account-of-descriptor | passphrase-of-xpub`. ms: `derive` only. This matches the report's list, and no subcommand is missing.

I went beyond one mode per subcommand, because the implementer's test covers only one. Every internal dispatch path that receives a passphrase was exercised. There are **27 command/mode rows**:

- addresses
- restore:
  - single `phrase=`
  - single `ms1=`
  - keyless single-sig template (`run_singlesig_template_completion`)
  - keyless multisig template (`run_multisig_template_completion`)
  - keyed policy md1 (`run_multisig`)
- derive-child
- bundle:
  - single
  - multisig template
  - `--descriptor` (`bundle_run_unified_descriptor`)
  - multisig `--json`
- convert
- silent-payment
- slip39 split and combine
- verify-bundle, each against a matching bundle:
  - slot/`--bundle-json`
  - single-sig template
  - multisig template
  - `--descriptor`
  - multisig slot
- the three xpub-search modes
- ms derive:
  - `--in`
  - positional ms1

**Forms per row**, all with passphrase `TREZOR`:

- `--passphrase-stdin` (the reference)
- `--passphrase -`
- `-` with stdin `TREZOR\n`
- `-` with stdin `TREZOR\r\n`
- `--passphrase=-`
- `@env:V`
- `--passphrase=@env:V`
- `@env:V` where `V=TREZOR\n`
- literal with the override, space-separated
- literal with the override, `=`-joined
- none, as a control

**Result:** every row gives `same(0)` in every form. The note count is `n0` for all private forms and `n1` for literals. None always differs from the reference, so each row has discriminating power.

**verify-bundle** prints `result: ok` for every form on the slot, descriptor and multisig paths. The single-sig and multisig template paths print `OK (… recomposed)`. None gives exit 4 `mismatch`.

**Double-resolution probes, all rows:**

- `@env:V` with `V=-` equals `--passphrase-stdin` fed `-`, i.e. the literal `-` (fp `66d564d1`). It never reads stdin.
- `@env:V` with `V=@env:QQ` equals stdin `@env:QQ`, i.e. the literal. It never resolves twice.

slip39 split was checked separately by combining under the same literal, because shares are randomised.

**Conflicts, all 21 rows** (the earlier harness version; the descriptor and multisig verify-bundle rows were added afterwards): these all exit 64:

- `- --passphrase-stdin`
- `@env:V --passphrase-stdin`
- `--passphrase` given twice
- `--allow-argv-secret --passphrase TREZOR --passphrase-stdin` (on ms, the guard's `-` placeholder conflicts)

## 2. Byte edges (convert fingerprint and ms derive, all four channels)

stdin flag, `-` and `@env:` agree on every value the OS can carry, and **ms and toolkit agree cell for cell**:

| bytes | stdin / `-` / env | literal |
|---|---|---|
| `TREZOR\r` | 4b53a850 (lone CR kept) | 4b53a850 |
| `TREZOR\n\n` | 48efb44f | d1f56c4d |
| `TREZOR\r\n\r\n` | 45494db3 | ef47a8e3 |
| `\nTREZOR` | 8fa10bb3 | 8fa10bb3 |
| `TREZOR\n\r` | f934e53f | f934e53f |
| `TRE\0ZOR` | 46944aeb (stdin only; the OS forbids NUL in env and argv) | — |
| `""`, `\n`, `\r\n` | 73c5da0a (the empty passphrase) | literal `\n` → d493bc20 |
| 100,000 × `A` | 388ee739 | 388ee739 |
| 1,000,001 bytes (stdin only; argv/env are E2BIG) | 380d4f42 | — |
| `TRÉZOR` precomposed vs decomposed | 2ef42bb5 both (NFKD) | same |
| non-UTF-8 `\xff` | error, exit 1 | **panic, exit 101** (see N1) |

No channel ever disagrees with the chosen rule.

## 3. stdin sharing

I paired `--passphrase -` (and, separately, `--passphrase-stdin`) with every other stdin-capable input listed in each subcommand's `--help`. That is 34 probes. Refused as intended:

- `--from <node>=-` (addresses, restore, convert, derive-child, slip39)
- `--slot @N.<secret>=-`
- `--secret -`
- `--share -`
- `--bip38-passphrase-stdin`
- `--phrase -`, `--phrase-stdin`, `--ms1-stdin`
- `--descriptor-from <node>=-`
- `--passphrase-candidates-file`
- ms `--hex -`, `--phrase -`, positional `-`, and no source at all (implicit stdin ms1)

`--from phrase=@env:X` with `X=-` also refuses beside a stdin passphrase in convert, derive-child and bundle. So even the pre-existing "env-resolved `-` becomes stdin" quirk the implementer flagged cannot double-read.

Survivors are listed as M1 and M4 below. Both behave identically with `--passphrase-stdin`, so they are pre-existing.

## 4. Empty passphrase — for operator ruling (not counted)

All of these derive the no-passphrase wallet (`73c5da0a`) at exit 0 with **nothing on stderr**, in both CLIs:

- `--passphrase -` with empty stdin, or stdin `\n` / `\r\n`
- `--passphrase-stdin` with the same
- `@env:V` with `V=""` or `V=$'\n'`

A literal `--passphrase ""` gets only the generic argv note. `restore` alone makes it visible, printing `(passphrase: none)`.

The case where the user clearly meant to supply one: they named a private passphrase channel and it came back empty. Examples: `pass show x | mnemonic … --passphrase -` when `pass` fails, or `export PP=` by mistake. The result is a different wallet, silently.

Cheap remedy: one stderr line whenever a stdin or `@env:` passphrase resolves to empty. That remedy would also close M1.

Reproduction:

```
F687_SEED="abandon … about" mnemonic convert --from phrase=@env:F687_SEED --to fingerprint --template bip84 --passphrase - </dev/null
```

The same with `ms derive --in card.ms1 --template bip84 --passphrase @env:PP` and `PP=`.

## 5. The stderr note

- Exactly one note for a literal, in both spellings.
- None for `-`, `@env:` or `--passphrase-stdin`.
- stdout is byte-identical to the stdin-flag run.

Tested with `=`-joined values `--foo`, `-x`, `--passphrase-stdin`, `--help`, `--`, `-h`, `it's "quoted"`, `'`, `"`, `$(id)`, `@ENV:PP`, `x@env:PP`, and the note's own text. None of them was echoed. `@env:`, `@env:lower` and `@env:PPX ` (trailing space) are errors, not literals. The space-separated flag-like values are refused by clap or the ms guard before any derivation (see F1).

## 6. Test vectors

- Byte-identical: `cmp` passes; 27 cases.
- Re-ran three claimed mutations on `git archive` copies. Each landed exactly once, the file was restored and the restore asserted.
  - **T1** (toolkit `-` → Argv): **9 red.** Matches the claim.
  - **T4** (verify-bundle template back to AsWritten): **1 red.** Matches.
  - **S3** (ms `@env:` keeps the newline): **2 red.** Matches.
- **Could the resolver regress with the vectors green? Yes.** See M3.

## 7. CHANGELOG and help

The CHANGELOG entries in both repos are accurate against the binaries: behaviour, note text, vector count and refusal list. Help is not fully accurate (M5).

---

## Minor

**M1. A `/dev/stdin` file path beside a stdin passphrase silently gives the empty passphrase (pre-existing; also true of `--passphrase-stdin`).**
- `silent-payment --secret-file /dev/stdin --passphrase -` with the seed on stdin exits 0 with the **no-passphrase** sp address, identical to omitting `--passphrase`.
- `ms derive --in /dev/stdin --template bip84 --passphrase -` with the ms1 on stdin exits 0 with `master_fingerprint: 73c5da0a`.
- Both behave the same with `--passphrase-stdin`.

It needs the user to name `/dev/stdin` explicitly as a file, so it is not Important. The item-4 "empty private passphrase" warning would surface it. Alternatively, refuse a file path that canonicalises to `/dev/stdin` or `/proc/self/fd/0`.

**M2. ms: the argv guard and the resolver disagree on a whitespace-padded `-`.**
The guard trims (`value.trim() != "-"`), so it treats `" -"` as the stdin channel and lets it through without `--allow-argv-secret`. The resolver matches `-` exactly, so the same value is a literal. Result: `ms derive --in card.ms1 --template bip84 --passphrase " -"` with `TREZOR` piped gives exit 0, fingerprint `e20c1882` (the passphrase `" -"`), with the piped passphrase ignored and one note. The same happens for `"\t-"`, `--passphrase="- "` and `--passphrase=$'-\n'`.

The toolkit refuses all of these (exit 2, material). This contradicts the ms module's own claim that "the guard and the resolver cannot disagree about what is material". The fix is to classify on the same predicate in both places, either exact `-` or trimmed in both.

**M3. The vectors do not pin two parts of the rule; both mutations survive the whole suite in both repos.**
- **G1** (toolkit `resolve_env` resolves a second time when the value begins `@env:`): 3959/3959 green. Behaviour: `PP='@env:QQ'`, `QQ=TREZOR` would derive `b4e3f5ed` instead of the literal `@env:QQ`, a different wallet.
- **G2** (toolkit) and **G3** (ms): `strip_one_newline` also strips a lone trailing `\r`. Suites are 3959/3959 and 430/430 green. Behaviour: `TREZOR\r` would derive `b4e3f5ed` instead of `4b53a850`.

Add vector rows for:
- `@env:` holding `@env:X`
- stdin and env `TREZOR\r`
- `\nTREZOR`
- `TREZOR\n\r`

The expected values are in the §2 table.

**M4. `bundle --import-json -` is not in the single-stdin guard.**
`bundle --network mainnet --import-json - --slot @0.phrase=@env:S --passphrase -` with the JSON on stdin: the passphrase reader swallows the whole JSON as the passphrase, then exits 1 with `--import-json: envelope JSON parse: EOF`. It fails loudly, so there is no wrong wallet, but the error misdirects. The same happens with `--passphrase-stdin`. Add `--import-json -` to the readers passed to the guard.

**M5. Help is not updated on 5 of the 12 subcommands; the report's "Help text updated on every `--passphrase`" is false.**
- `bundle`, `verify-bundle` and all three `xpub-search` modes: `--passphrase` help never mentions `-` or `@env:`.
- `xpub-search --passphrase` still says "(inline). Emits an argv-leakage advisory" unconditionally.
- `slip39 split --passphrase` still says "the argv-leakage advisory fires iff this field is `Some(_)` … regardless of value", which is now false for `-` and `@env:`.

These flow into the man pages.

## Nit

- **N1.** Non-UTF-8 values:
  - `@env:V` with non-UTF-8 bytes errors `env-var V referenced by sentinel is not set`, but it is set. The message should say "not valid UTF-8". Both CLIs.
  - A non-UTF-8 argv value panics with exit 101 (`std::env::args`), both CLIs. Pre-existing.
- **N2.** The slip39 two-stdin refusal names `--passphrase-stdin` when the user typed `--passphrase -`: `slip39 combine --share - --share @env:S --passphrase -` prints "(across --share, --from, and --passphrase-stdin)". The report says "the error names the spelling the user typed".
- **N3.** The toolkit argv-guard refusal for a literal `--passphrase` lists only `--passphrase-stdin` under "these exist today". It could name `--passphrase -` and `@env:VAR` too.
- **N4.** The manual table (`41-mnemonic.md`, "How `--passphrase` is read") says a literal is "that string, verbatim, plus one stderr line". Without `--allow-argv-secret` the toolkit refuses it (exit 2), and so does ms. The table also omits the invalid-name error.

## Secret-handling follow-up (non-gating per the 2026-08-27 ruling)

- **F1.** Space-separated flag-like passphrases are echoed in errors, and this is pre-existing:
  - toolkit: `--allow-argv-secret --passphrase --foo` prints clap's `unexpected argument '--foo' found`;
  - ms: its own refusal prints `--passphrase was given "--foo"`.

---

## Reproduction harness

The scripts lived under `/scratch/code/shibboleth/review-f687-scratch/h/`, which was deleted per the brief:

- `matrix.py`: the §1 matrix
- `stdin2.py`: 34 second-reader probes
- `bytes.py`: the §2 table
- `mutate.py`: T1, T4, S3, G1, G2, G3

Each finding above carries its standalone command.

ready to ship: yes
