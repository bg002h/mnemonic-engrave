# R0 round 3 — hashlock-phrase brainstorm, fold verification of the r2 security-software fold

**Date:** 2026-09-03
**Model:** sonnet (single agent, read-only)
**Repo HEAD at time of review:** `203f3bb` (unrelated commits landed after the
fold; `git log --oneline c20ec9e..HEAD -- design/BRAINSTORM_hashlock_phrase.md`
is empty, so the record on disk is byte-identical to its state at the fold
commit)
**Commits compared:** round-2 report persisted at `e9d7895` vs fold `c20ec9e`
(`git diff e9d7895..c20ec9e -- design/BRAINSTORM_hashlock_phrase.md`)
**Brief:** `design/agent-briefs/hashlock-brainstorm-R0-r3-fold-verification-brief.md`

**The one question:** did the fold address each of the 20 findings and
reviewer question 5 exactly as section 7.1 claims, without introducing a new
defect, a contradiction, or a wrong citation? **Mostly yes — 20 of 21 items are
cleanly FIXED. One (C-2) is PARTIAL: a stale pre-fold sentence in section
4.6's "Fork (H2)" bullet (untouched by this fold) still describes the OLD,
rejected design — a `0x03` decoder arm and a new payload class — directly
contradicting ruling L22.** Two further new findings: a length-check test row
added by this fold names the wrong function, and a test-coverage narrowing
that looks intentional but is worth a line. All citations and numbers
recomputed exactly.

## Counts

`FIXED:20 PARTIAL:1 NOT:0` (21 items = C-1..C-4, I-1..I-6, M-1..M-7, N-1..N-3,
reviewer question 5)

`C:0 I:2 M:1 N:0` (new findings from this round)

## Per-finding table

| finding | verdict | where in the record (line) | note |
| --- | --- | --- | --- |
| C-1 (`--in` ambiguity) | FIXED | L20 (54), 4.2 (246-266, 299-301), 7.1 (692) | `--in`/`-`/positional = ms1 everywhere; ms1-shaped phrase refused on both phrase channels; no stray "three channels" or "`--in` carries the phrase" text anywhere (grepped, clean) |
| C-2 (`0x03` arm fail-open at 5 sites; `isStrictMs1` misclassifies) | **PARTIAL** | Fixed at 4.1 (223-231), 4.4 (487-493), 5 (625), 7.1 (693). **Stale at 4.6 line 552-553**: "The `0x03` decoder arm and its length rule; every seed call site refuses by name; the payload class row in the record-class vectors" | Pre-existing "Fork (H2)" bullet, untouched by this fold, still states the pre-review design the fold explicitly rejected (see New Finding 1) |
| C-3 (`--random` data loss) | FIXED | L21 (55), 4.2 (267-274), 7.1 (694) | `--random` refuses without `--out`/`--json`; no surviving sentence lets it run without either (grepped) |
| C-4 (index-keyed switch clears lock) | FIXED | 4.4 (441-452), 4.6 (599-602), 7.1 (695) | Label-keyed rows, unreachable default, §8i restated, every row tested by label including displaced ones |
| I-1 (`--allow-argv-secret` inert) | FIXED | 4.2 (246-253), 7.1 (696) | `SUBCOMMANDS`, `override_applies`, `flag_class` all named as edits |
| I-2 (side channel must be opted into) | FIXED | 4.2 (253-256), 7.1 (697) | `.on("--hashlock-phrase")` named; `/dev/null` gate stated |
| I-3 (shipped readers normalise) | FIXED | 4.2 (302-306), 7.1 (698) | New byte-verbatim reader named; `read_input`/`read_phrase_input` explicitly forbidden |
| I-4 (Go leads Rust) | FIXED | L22 (56), 4.1 (219-222), 4.5 (516-517), 7.1 (699) | H1b precedes H2 in both the stage list and the Order line |
| I-5 (`--out`/stdout unstated) | FIXED | 4.2 (312-316), 4.6 (586-587), 7.1 (700) | `--out` never suppresses stdout; test row added for `--out` + stdout together |
| I-6 (CI job unnamed; macOS lacks `openssl kdf`) | FIXED | 4.3 (418-424), 7.1 (701) | ms-codec's Ubuntu-only job named; preflight step added |
| M-1 (card unlabelled) | FIXED | 4.2 (319-322), 7.1 (702) | First card line names the preimage |
| M-2 (JSON errors to stdout) | FIXED | 4.6 (582-586), 7.1 (703) | Negative-content rows assert neither stdout, stderr, nor the JSON envelope carry the phrase/preimage |
| M-3 (`flag_class` misnames material) | FIXED | 4.2 (252-253), 7.1 (704) | Folded with I-1 |
| M-4 (keyboard/flow unnamed) | FIXED | 4.4 (453-457), 7.1 (705) | `NewPassphraseKeyboard` + new flow function named |
| M-5 (`unlockDerive` zero-pads salt) | FIXED | 4.4 (468-475), 7.1 (706) | New driver taking `salt []byte`; constant-comparison rule stated |
| M-6 (`--method` unspecified) | FIXED | 4.2 (278-281), 7.1 (707) | Refused at exit 64; `--json` omits `method` for sourced X |
| M-7 (tty stdin hangs silently) | FIXED | 4.2 (256-259), 7.1 (708) | One stderr prompt line specified |
| N-1 (hardened H unpinned) | FIXED | 4.3 (409-413), 7.1 (709) | H pinned, value verified below |
| N-2 (64 visible bits) | FIXED | 4.4 (480-484), 7.1 (710) | Adequacy tied explicitly to the CI vector; walk records full digests |
| N-3 (`hmac` not a direct dep) | FIXED | 4.3 (398-402), 7.1 (711) | `me`'s spelling copied verbatim, measured feature set cited |
| Reviewer Q5 (card first line) | FIXED | 4.2 (319-322), 7.1 (712) | Same text as M-1, correctly cross-referenced |

## Citations and numbers

All commands run against the pinned repos: mnemonic-secret at `7fc1e58`
(confirmed: `git -C /scratch/code/shibboleth/mnemonic-secret log -1 --format=%H`
→ `7fc1e589e475e1b9024afc104dc58d0687e085e2`) and seedhammer fork at `70008da5`
(confirmed: `git -C /scratch/code/shibboleth/seedhammer log -1 --format=%H` →
`70008da5f935b36635a442cb2738f8dcc2fce7f1`).

**`SUBCOMMANDS`:**
```
$ grep -n "SUBCOMMANDS" crates/ms-cli/src/argv_guard.rs | head -1
67:const SUBCOMMANDS: [&str; 12] = [
```
Matches "`[&str; 12]`" cited in the record (7.1, 4.2).

**`override_applies` (eight-verb fixed match):**
```
$ sed -n '/fn override_applies/,/^}/p' crates/ms-cli/src/argv_guard.rs
fn override_applies(argv: &[String]) -> bool {
    argv.iter().any(|t| t == ALLOW_FLAG)
        && matches!(argv.get(1).map(|t| t.trim()),
            Some("encode") | Some("decode") | Some("inspect") | Some("verify")
            | Some("repair") | Some("split") | Some("combine") | Some("derive"))
}
```
Eight verbs, `hashlock` absent. Confirms I-1.

**`flag_class` fallthrough:**
```
$ sed -n '/fn flag_class/,/^}/p' crates/ms-cli/src/argv_guard.rs
fn flag_class(flag: &str) -> &'static str {
    match flag {
        "--phrase" => "a BIP-39 mnemonic",
        "--hex" => "raw hex entropy",
        "--ms1" => "an ms1 string",
        _ => "a BIP-39 passphrase",
    }
}
```
Confirms M-3's "a BIP-39 passphrase" fallthrough.

**`is_ms1_shaped` and the `--in FILE` remedy line:**
```
$ sed -n '/fn is_ms1_shaped/,/^}/p' crates/ms-cli/src/argv_guard.rs
    t.len() >= MIN_MS1_LEN && t.starts_with("ms1")
        && t[3..].chars().all(|c| BECH32_CHARSET.contains(c))
$ grep -n -- "--in FILE" crates/ms-cli/src/argv_guard.rs
425:         \x20   ms {verb} --in FILE      # read it from a file\n      \
```
Both match the record's citations (4.2 line 264-265, 7.1).

**`parse.rs` helpers:**
```
$ sed -n '/fn read_input/,/^}/p' / '/fn read_phrase_input/,/^}/p' / '/fn read_in_file/,/^}/p' crates/ms-cli/src/parse.rs
read_input      -> strip_whitespace(&raw)              (strips all whitespace + "-" + "," via strip_display_separators)
read_phrase_input -> normalize_phrase(&raw)             (trims + collapses runs)
read_in_file    -> std::fs::read_to_string(path)        (strips nothing)
$ grep -n 'channel: ""' crates/ms-cli/src/parse.rs
43:            channel: "",
```
All four confirmed as the record states (4.2 302-306, 7.1).

**`rust.yml` job platforms:**
```
$ sed -n '108,130p' .github/workflows/rust.yml
  test-ms-codec:
    name: test (ms-codec)
    runs-on: ubuntu-latest
...
  test:
    name: test (${{ matrix.os }})
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
```
Confirms 7.1's "`test (ms-codec)` job is Ubuntu-only and the ms-cli matrix is
ubuntu + macos" and 4.3's preflight-step placement.

**Five `DecodeMS1` callers, by file:line:**
```
$ grep -rn -B1 'codex32.DecodeMS1(' --include=*.go . | grep -v '_test.go' | grep ':=\|='
gui/ms1_decode.go:22:      _, language, entropy, err := codex32.DecodeMS1(scan)
gui/codex32_polish.go:106: _, _, ent, msErr := codex32.DecodeMS1(scan)
gui/singlesig_verify.go:185:       _, _, ent, err := codex32.DecodeMS1(s)
bundle/verify.go:138:      _, language, ent, err := codex32.DecodeMS1(str)
gui/multisig_verify.go:1237:       _, _, ent, err := codex32.DecodeMS1(s)
```
Exactly the five cited in 4.6 (593-596), all discarding the prefix with `_`.

**`gui/codex32_polish.go` `showSecret`:**
```
$ sed -n '105,108p' gui/codex32_polish.go
	_, _, ent, msErr := codex32.DecodeMS1(scan)
	wipeBytes(ent)
	showSecret := f.Unshared && msErr == nil
```
Matches.

**`sysw/classify.go:48`:**
```
$ sed -n '45,49p' sysw/classify.go
	if isStrictMnemonic(record) {
		return ClassMnemonic
	}
	if isStrictMs1(record) {
		return ClassCodex32Secret
```
Line 48 is `if isStrictMs1(record) {`, tested first, before any other class.

**`gui/sysw_admit.go` `admits` and six `ClassCodex32Secret` admissions:**
```
$ sed -n '/func admits/,/^}/p' gui/sysw_admit.go
func admits(p syswProgram, c sysw.Class) bool { return admitted[p][c] }
$ grep -c "sysw.ClassCodex32Secret: true" gui/sysw_admit.go
6
```
Exactly six, matching "six `ClassCodex32Secret` admissions" (7.1).

**`gui/unlock_kdf.go:242` `unlockDerive` and `seal/wire.go` `SaltLen = 16`:**
```
$ grep -n "func unlockDerive" gui/unlock_kdf.go
242:func unlockDerive(ctx *Context, th *Colors, h seal.Header, pass []byte) ([]byte, bool) {
$ grep -n "SaltLen" seal/wire.go
32:	SaltLen   = 16
85:	Salt       [SaltLen]byte
```
Both match exactly.

**`pbkdf2 0.12.2`'s default features:**
```
$ sed -n '/\[features\]/,/^\[/p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pbkdf2-0.12.2/Cargo.toml
[features]
default = ["hmac"]
...
simple = ["hmac", "password-hash", "sha2"]
```
Confirms N-3: default is exactly `["hmac"]`.

**Hardened H recomputed (python3 hashlib):**
```
$ python3 -c "
import hashlib
x = hashlib.pbkdf2_hmac('sha256', b'correct horse battery staple', b'ms-hashlock-v1', 100000, 32)
print('X', x.hex()); print('H', hashlib.sha256(x).hexdigest())"
X c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
H 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```
Byte-identical to the record's pinned H (4.3 line 412) and the report's N-1
value.

## New findings

### 1. (Important) Section 4.6's "Fork (H2)" bullet still describes the pre-review design, directly contradicting L22

**Claim.** Section 4.6, the "Fork (H2)" bullet at lines 552-553 (untouched by
this fold — the diff added a NEW paragraph after this bullet rather than
editing it), reads:

> "The `0x03` decoder arm and its length rule; every seed call site refuses by
> name; the payload class row in the record-class vectors; ..."

This is verbatim the OLD H2 description this fold explicitly replaced in
section 4.1 (the pre-fold text there read "The `0x03` arm in `DecodeMS1`,
every seed call site refusing it by name, a payload class for it that reaches
no screen" — see `git show e9d7895:design/BRAINSTORM_hashlock_phrase.md` around
that section). It asserts three things L22 rejects: (a) a `0x03` decoder arm
(L22: `DecodeMS1` stays unchanged; a separate `DecodeMS1Preimage` is added
instead), (b) call sites gain per-site refusal code ("refuses by name" —
C-2's construction showed this is exactly the fail-open change the review
flagged), and (c) a new payload class exists ("the payload class row" — L22:
"no new class this cycle").

**Evidence.** `git diff e9d7895..c20ec9e -- design/BRAINSTORM_hashlock_phrase.md`
shows this bullet is not touched by the fold — the hunk immediately below it
(a new "Added by the r2 security-software review" paragraph) is an addition,
not an edit to this bullet. The fold commit message's own propagation grep
claims completeness but scoped itself to 4.5 only: *"Propagation grep after
the fold: one stale sentence in 4.5 (H3 carrying the classifier) fixed; 0
remaining."* This 4.6 sentence was missed.

**Remedy.** Rewrite the "Fork (H2)" bullet to match 4.1's H2 description:
`DecodeMS1` stays unchanged; `DecodeMS1Preimage` (new function) and its
restated length rule; `isStrictMs1` gains the prefix test; the payload class
row becomes an *inert-classification* row, not a new-class row.

### 2. (Important) The H2 length-row test addition names the wrong function

**Claim.** Section 4.6's r2-review addition (line ~591-592) reads: *"H2:
`DecodeMS1` length rows for `0x03` at 16, 20, 24, 28, 34 and 46 payload bytes
each refused (mutation: let the arm fall into the shared length switch)"*.
Per L22 and 4.1 (223-227), the length rule for `0x03` belongs to the NEW
`DecodeMS1Preimage`, not `DecodeMS1` — `DecodeMS1` refuses every `0x03` string
unconditionally via its unchanged `default:` arm, regardless of length, so
this mutation (falling into the shared length switch) cannot even occur in
`DecodeMS1` as designed. Naming `DecodeMS1` here risks an H2 implementer
adding length-aware logic to the wrong (and supposed-to-stay-untouched)
function.

**Evidence.** 4.1 (223-227): "a separate `DecodeMS1Preimage` serves only the
new consumer, with the Go length rule restated (the shared `case 16, 20, 24,
28, 32` switch cannot be inherited)." Compare the immediately following,
correctly-worded row in the same paragraph: "one test per `DecodeMS1` caller
... that a `0x03` string is refused" — that row correctly names `DecodeMS1`
because those five callers do call the unchanged function.

**Remedy.** Change "`DecodeMS1` length rows" to "`DecodeMS1Preimage` length
rows" at 4.6 line ~591.

### 3. (Minor) The byte-exact phrase test narrows from two channels to one without saying why

**Claim.** The r2 report's "Test plan additions" table row 8 named byte-exact
testing "through `--in` and `--hashlock-phrase-stdin`". Post-L20, `--in` is no
longer a phrase channel, so dropping it is correct — but the fold's
corresponding 4.6 row (line ~578-580) tests only `--hashlock-phrase-stdin`,
not the other (now sole alternative) phrase channel, `--hashlock-phrase`
(argv). 4.2 states the new byte-verbatim reader applies to "the phrase
channels" (plural, both), so a reader confined to stdin coverage leaves the
argv path's byte-exactness implicit rather than stated as its own test row.

**Evidence.** 4.2 (302-306) names both `--hashlock-phrase` and
`--hashlock-phrase-stdin` as sharing the new reader; 4.6 (578-580) tests only
the stdin channel by name. Since `Source::read_raw`'s admitted-value arm
(`parse.rs:67-85`) returns the raw string before any reader-specific
processing, the same new reader function is plausibly invoked regardless of
channel, which is why this is Minor rather than Important — but the record
does not say so, and 4.6 is still PRESENTED, not ruled.

**Remedy.** Either add `--hashlock-phrase` explicitly to the byte-exact test
row, or add one sentence stating both channels route through the identical
reader call so a single test covers both.

## Confirmed clean

- **Propagation hazard (a)** — no surviving sentence says `--in` carries the
  phrase or lists three phrase channels; every mention of "`--in`" and
  "phrase channel(s)" in the whole record is consistent with L20 (grepped,
  9 hits, all consistent).
- **Propagation hazard (d)** — no surviving sentence lets `--random` run
  without `--out`/`--json`; every `--random` mention (13 hits) is consistent
  with L21.
- **Propagation hazard (e)** — no surviving sentence treats all four
  `unreachable!` sites as refusals; the record correctly differentiates
  functional arms (`decode`, `combine`) from typed refusals (`payload_lang`
  behind `verify`/`derive`), consistent with the r1-caught distinction.
- **Section 4.5 (Order)** — H1b correctly precedes H2, composer spec fold, H2
  plan, implementer, review, merge, flash, H4 walk, then H3; the one stale
  sentence the fold's own commit message named (H3 carrying the classifier)
  is in fact gone.
- **Numbers** — all four cited counts (`[&str; 12]`, five `DecodeMS1`
  callers, six `ClassCodex32Secret` admissions, the hardened H) recomputed
  exactly as the record and the fold commit message state.
- **Citations** — all twelve cited symbols/lines exist in the pinned repos
  and say what the record says (see "Citations and numbers" above); no wrong
  citation found.
- **Rulings L20/L21/L22 stated and followed** in 4.1, 4.2, 4.4, 4.5, section 5
  and 7.1, per the brief's instruction — confirmed present in each of those
  sections except the one stale 4.6 bullet noted in New Finding 1.
- **Section 5 (Defaults)** gained the four new rows the fold commit message
  claims (M-1, M-6, M-7, C-2), each correctly worded against its ruling.
