# R0 round 2 — hashlock-phrase brainstorm, security software engineering lens

**Date:** 2026-09-03
**Model:** opus (single agent, read-only)
**Artifact reviewed:** `design/BRAINSTORM_hashlock_phrase.md` at mnemonic-engrave commit `82433fd`
(`git -C /scratch/code/shibboleth/mnemonic-engrave log -1 --format=%h -- design/BRAINSTORM_hashlock_phrase.md` → `82433fd`; repo HEAD is the same commit)
**Brief:** `design/agent-briefs/hashlock-brainstorm-R0-r2-security-software-brief.md`

**The one question:** would I sign off on building `ms hashlock` and the SH2 phrase
screen from this record as written? **No.** Four constructions below produce a
wrong digest, a destroyed preimage, a preimage treated as a seed, or a silently
cleared hash lock, and none of them is caught by the test plan in §4.6 as
presented. All four have cheap remedies that belong in the record before a spec
is written.

**Files read**

*mnemonic-engrave (`82433fd`)*
- `design/BRAINSTORM_hashlock_phrase.md` — all sections (the artifact)
- `design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md` (headings + "Confirmed sound" + "Questions for the operator"; findings bodies not re-derived, per the brief)
- `design/agent-reports/hashlock-brainstorm-R0-r1-fold-verification.md` (whole)
- `crates/mnemonic-io-lib/src/write.rs:1-65`, `src/fd.rs:1-80`, `src/channel.rs:1-45`
- `crates/me-cli/Cargo.toml:35-55`

*mnemonic-secret (`7fc1e58`)*
- `crates/ms-cli/src/argv_guard.rs` (whole, 529 lines)
- `crates/ms-cli/src/parse.rs` (whole, 246 lines)
- `crates/ms-cli/src/out.rs` (whole), `src/advisory.rs` (whole), `src/main.rs` (whole)
- `crates/ms-cli/src/cmd/encode.rs:20-120,180-393`
- `crates/ms-cli/src/cmd/decode.rs`, `derive.rs`, `payload_lang.rs` (dispatch/`--in` sites via grep; the four `unreachable!` sites are settled by r0/r1 and were not re-derived)
- `crates/ms-codec/Cargo.toml`, `crates/ms-cli/Cargo.toml`, `rust-toolchain.toml`, `crates/ms-codec/src/consts.rs:71`
- `.github/workflows/rust.yml:85-215,329-357`; `man-release.yml`, `fuzz-smoke.yml`, `vendor-freshness.yml` (grep for tool installs)

*seedhammer fork (`70008da5`)*
- `gui/composer_hash.go` (whole, 176 lines)
- `gui/composer_sources.go:210-245`, `gui/ms1_decode.go` (whole), `gui/codex32_polish.go:90-130`
- `gui/passphrase_flow.go:30-145`, `gui/passphrase_keyboard.go:1-120`, `gui/passphrase_passproof.go:55-250`
- `gui/unlock_kdf.go:180-270`, `gui/sysw_admit.go:1-110`
- `passphrase/passphrase.go` (whole), `codex32/mspayload.go` (whole)
- `sysw/classify.go` (whole), `sysw/record.go` (class list), `seal/wire.go:32-36,85`

*Crates* — `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pbkdf2-0.12.2/Cargo.toml`

---

## C:4 I:6 M:7 N:3

---

## Findings

### C-1 — `--in FILE` means PHRASE on `ms hashlock`, every printable-ASCII string is a valid phrase, and the argv guard's own refusal text sends an operator holding a preimage plate straight into it

**Claim.** §4.2 gives `ms hashlock` both a `<ms1>` positional source and an
`--in FILE` phrase source; the argv guard refuses the positional and its refusal
message prescribes `--in FILE`; `--in FILE` then derives a *new* preimage from
the ms1 text as though it were a phrase, silently, and the resulting digest's
preimage is on no plate the operator holds.

**Evidence.**
- §4.2: "`--in FILE` (the phrase from a file, same newline rule)" and, two
  bullets later, "`<ms1>` or `-`: a preimage-kind ms1, to re-derive H from a
  plate."
- `crates/ms-cli/src/argv_guard.rs:134-146` `is_ms1_shaped`: `len >= 48`,
  `starts_with("ms1")`, remainder in the bech32 charset — after
  `strip_display_separators`, so the grouped plate form counts too. A preimage
  single is 75 characters (measured:
  `python3 -c "print(9 + -(-33*8//5) + 13)"` → `75`) and every character of
  `ms10hash…` is in the charset (measured: `all(c in CS for c in "0hash")` →
  `True`). So `ms hashlock ms10hash…` is refused as material by layer 2
  (`find_argv_material`, `:340-374`).
- `argv_guard.rs:399-435` `refusal()` — the remedy it prints, with `verb`
  interpolated from the subcommand allowlist:
  `\x20   ms {verb} --in FILE      # read it from a file`. With `hashlock` in
  `SUBCOMMANDS` the operator is told, verbatim, to run
  `ms hashlock --in FILE`.
- The phrase rule (§4.2) admits every printable-ASCII string of 1..100
  characters except an exactly-64-character all-hex one. A 75-character ms1
  passes all of it.
- The one shipped guard against this class does not reach here.
  `crates/ms-cli/src/cmd/encode.rs:66-71` states the safety argument for
  `encode --in` in as many words: *"The phrase-only rule has no input that both
  parses as BIP-39 and reads as entropy."* That premise is **false for a
  hashlock phrase**, which has no parse to fail. And the compensating note at
  `encode.rs:255-273` fires only inside `phrase_parse_error` — i.e. only when
  BIP-39 parsing fails — so on `ms hashlock` it can never fire.

**Construction.**
1. `ms hashlock --hashlock-phrase-stdin --out plate.ms1 < phrase.txt`; engrave
   the `ms10hash…` plate; write down the phrase and the method line, as L3
   requires.
2. Months later, composing a second policy from the same preimage:
   `ms hashlock ms10hash…` → refused by the guard, with
   `ms hashlock --in FILE      # read it from a file` printed as the remedy.
3. `ms hashlock --in plate.ms1` → exit 0, stdout `hash:<H'>` where
   `H' = sha256(PBKDF2("ms10hash…"))`. No refusal, no warning; the card's
   character count reads 75 and the source kind reads "phrase".
4. `me sysw pack --in -` takes `hash:<H'>`; the policy is composed and funded.

The funded path is satisfied only by `X' = PBKDF2("ms10hash…")`. The engraved
plate decodes to `X`, not `X'`. The written-down phrase derives `X`, not `X'`.
**Both backups the record instructs the operator to keep fail to spend the
funds**, and recovery requires reconstructing the exact wrong invocation from
memory. This is the I-6 defect class (a preimage pasted into the phrase slot),
one channel wider, and strictly worse than I-6: the guard advertises the route.

**Remedy (non-authoritative).** Refuse an `ms1`-shaped phrase on every phrase
channel (`--hashlock-phrase`, `--hashlock-phrase-stdin`, `--in`), naming the
positional/`-` route — the same shape as I-6's 64-hex refusal, reusing
`argv_guard::is_ms1_shaped`'s predicate so the two cannot drift. Alternatively,
give the ms1 source its own flag (`--preimage-in FILE`) and refuse `--in`
entirely on this verb, so no spelling of "read it from a file" is ambiguous.
Either way the record should state which of `ms`'s two `--in` meanings this verb
takes, because six verbs mean ms1 and two mean phrase
(`decode.rs:42`, `inspect.rs:43`, `verify.rs:40`, `derive.rs:47`, `repair.rs:101`,
`combine.rs:56` vs `encode.rs:72`, `split.rs:68`).

---

### C-2 — adding a `0x03` arm to `DecodeMS1` converts today's fail-CLOSED refusal into a fail-OPEN success at five call sites, all of which discard the prefix

**Claim.** §4.1 H2 says the fork gains "The `0x03` arm in `DecodeMS1`, every seed
call site refusing it by name". The record does not say what `DecodeMS1` returns
for `0x03`, and every existing caller ignores the prefix, so the natural
implementation (a `case msPrefixPreimage:` returning `(3, 0, X, nil)`) turns a
hashlock preimage into seed entropy at five sites at once — where today it is
safely refused.

**Evidence.**
- `codex32/mspayload.go:39-53`: today `default: return errMSBadPrefix`. Round 0
  Q4 confirmed this as the safe dead end for every flashed SH2.
- **Measured, 5 of 5 non-test call sites discard the prefix:**
  ```
  $ grep -rn -B1 'codex32.DecodeMS1(' --include=*.go . | grep -v '_test.go' | grep ':=\|='
  gui/ms1_decode.go:22:      _, language, entropy, err := codex32.DecodeMS1(scan)
  gui/codex32_polish.go:106: _, _, ent, msErr := codex32.DecodeMS1(scan)
  gui/singlesig_verify.go:185:       _, _, ent, err := codex32.DecodeMS1(s)
  gui/multisig_verify.go:1237:       _, _, ent, err := codex32.DecodeMS1(s)
  bundle/verify.go:138:      _, language, ent, err := codex32.DecodeMS1(str)
  ```
- Reachability is not hypothetical: `gui/codex32_polish.go:106-108` gates the
  "Show secret" button on exactly `msErr == nil`
  (`showSecret := f.Unshared && msErr == nil`), and
  `gui/ms1_decode.go:32-36` then runs `bip39.New(entropy)` and prints the
  24 words. A 32-byte preimage satisfies `DecodeMS1`'s shared length switch
  (`mspayload.go:54-58`, `case 16, 20, 24, 28, 32`) without any new code.
- The classifier cannot save it either. `sysw/classify.go:45-52` runs
  `isStrictMs1` before anything else could match, and `isStrictMs1`
  (`:116-125`) tests only length ≤ 90, HRP `ms1`, and `codex32.New` validity —
  never the prefix byte. A 75-character preimage is `ClassCodex32Secret`, which
  `gui/sysw_admit.go:33-79` admits to **six** programs (progBackupWallet,
  progXpub, progSingleSig, progMultisig, progWalletPolicy, progBip85). So §4.4's
  "classifies as a new secret class that reaches no screen" requires editing
  `isStrictMs1` itself, which the record never says.
- The shared length switch is a second trap: a `0x03` payload of 16/20/24/28
  bytes passes `case 16, 20, 24, 28, 32` unchanged, so I-2's length rule must be
  re-implemented on the Go side and cannot be inherited.

**Construction.** H2 firmware. Operator loads a payload (or types/scans a
string) containing `ms10hash…`, a preimage `ms hashlock --out` produced.
`sysw.Classify` → `ClassCodex32Secret` → admitted to Backup Wallet → the
codex32 confirm screen offers "Show secret" because `DecodeMS1` now returns
`err == nil` → `ms1DecodeFlow` prints the preimage as 24 BIP-39 words → the
operator engraves it as a **seed plate**, or picks Single-sig and derives an
xpub from it and receives funds to that wallet. The preimage becomes public
on-chain at the first hash-path spend (§3.7), so that wallet's master seed is
published: **anyone-can-spend.** Before H2 the identical string is refused.

**Remedy (non-authoritative).** Invert the default: leave `DecodeMS1` refusing
`0x03` (its own doc comment scopes it to "the m-format **secret** payload", i.e.
seeds) and add a separate `DecodeMS1Preimage` that only the new consumer calls.
Then every one of the five sites stays fail-closed with no edit, and the H2
checklist is one added function instead of five audited call sites. If the arm
must live in `DecodeMS1`, the record needs (a) the enumerated five-site
checklist with one test each — the Go-side twin of I-3's `unreachable!` sweep,
and Go gives no exhaustiveness help at all; (b) an explicit statement that
`isStrictMs1` must gain the prefix test, in the same commit; (c) the Go length
rule for `0x03` restated, because the shared switch admits four wrong lengths.

---

### C-3 — `ms hashlock --random` can emit a digest whose preimage exists nowhere, at exit 0

**Claim.** The preimage of a `--random` invocation exists only in the stderr
card unless `--out` or `--json` is given, and the record specifies no refusal for
the combination that suppresses it.

**Evidence.**
- §4.2: stdout is `hash:<64 hex>` only; `--out FILE` carries the preimage;
  the stderr card carries it; `--json` carries it. §4.2 also: "stderr card (off
  with `--no-engraving-card`)".
- `--random`: "No phrase exists, so nothing can be guessed, and nothing can be
  remembered. This plate is the only copy." — the copy names the card as the
  sole copy, and the flag that deletes the card carries no interaction with it.
- The precedent is explicit at `crates/ms-cli/src/cmd/encode.rs:334-341`:
  "`--no-engraving-card`, and any `2>/dev/null`, now throws away the form an
  engraver actually reads (§6c)."
- `--out` is documented as overwriting and truncating
  (`crates/mnemonic-io-lib/src/write.rs:45-65`, `truncate(true)`), with no
  clobber check.

**Construction (three spellings, all exit 0).**
1. `ms hashlock --random --no-engraving-card > rec.txt` — stdout holds the
   digest, the preimage was never written anywhere and is gone at process exit.
2. `ms hashlock --random 2>/dev/null | me sysw pack --in -` — same, and the
   digest is already inside a composer payload.
3. `ms hashlock --random --out plate.ms1` run a second time after the first
   digest was funded — `write_private` truncates, and the only copy of the first
   preimage is destroyed.

In every case the policy is fundable and permanently unspendable through the
hash path.

**Remedy (non-authoritative).** `--random` refuses unless the preimage reaches a
channel that persists: require `--out`, or `--json`, or the card (i.e. refuse
`--random --no-engraving-card` without `--out`/`--json`). Consider refusing an
`--out` whose target exists for `--random` specifically, or writing
`O_EXCL`-style with an explicit `--force`. Note this is a *data-loss* rule, not a
secret-handling one, so it gates.

---

### C-4 — inserting the `Type a hashlock phrase` row into `composerHashEdit` shifts an index-keyed switch whose `default` arm CLEARS the hash lock

**Claim.** §4.4 says `Which hash?` "gains one row, `Type a hashlock phrase`,
placed before `Type 64 hex`". The shipped handler keys every decision on
`len(digests)` arithmetic and its fallthrough arm *removes* the hash lock, so the
direct execution of that instruction makes "Type 64 hex" silently clear the lock
and skips the §8i modal. The presented test plan has no test on the displaced
rows.

**Evidence.** `gui/composer_hash.go:140-176`:
```go
rows = append(rows, "Type 64 hex")
rows = append(rows, "No hash lock")
sel, ok := composerPickScreen(ctx, th, title, "Which hash?", rows)
...
if sel <= len(digests) {                     // §8i modal
    showError(ctx, th, title, composerCopyHashRule())
}
switch {
case sel < len(digests):   ...               // a payload digest
case sel == len(digests):  ...               // Type 64 hex
default:                   st.list.Paths[idx].Hash = nil; return true   // No hash lock
}
```
Insert one row at index `len(digests)` and: the new phrase row lands on the
`sel == len(digests)` arm (running the **hex pad**), "Type 64 hex" lands on
`default` (**clearing the lock** and returning `true`, a successful-looking
edit), and the §8i modal's `sel <= len(digests)` no longer covers the hex row —
the rule §8i exists to state is not shown for the entry it was written for.

**Construction.** H2 firmware, a path whose hash the operator wants to type as
hex. `Which hash?` → `Type 64 hex` → the screen returns immediately, the path
editor shows the path with **no** hash lock, and the operator, having just
"set" it, composes and funds a policy whose keyless hash branch is gone (or, for
a policy where the hash was the only lock on a branch, a branch that is now
unlocked — C22's anyone-can-spend shape). Nothing refuses and nothing warns.

`gui/composer_gates_test.go:664-700` is the only existing test that reaches
`Which hash?`, and it asserts only that the screen was reached and that §8j did
not fire — it selects no row. §4.6's fork bullet adds a harness test for the
**new** row only.

**Remedy (non-authoritative).** State in the record that the row switch becomes
label-keyed or constant-indexed (named row indices computed once), that the
`default` arm must be unreachable rather than meaning "clear", and that the §8i
condition is restated in terms of "the operator is taking a hash" rather than an
index comparison. Add tests for the two **displaced** rows, not only the new one
— this is the [[a-diff-falsifies-text-it-never-touches]] class.

---

### I-1 — `--allow-argv-secret` does not work on a verb the guard's override list does not name, so §4.2's argv sentence is false and the refusal's own remedy is a no-op

**Claim.** §4.2 says `--hashlock-phrase` is "refused on argv without
`--allow-argv-secret`", implying it proceeds with it. The override is gated on a
hard-coded eight-verb list that `hashlock` is not in, so it is refused **with**
the flag too — and the refusal prints "`--allow-argv-secret` proceeds".

**Evidence.** `crates/ms-cli/src/argv_guard.rs:256-269`:
```rust
fn override_applies(argv: &[String]) -> bool {
    argv.iter().any(|t| t == ALLOW_FLAG)
        && matches!(argv.get(1).map(|t| t.trim()),
            Some("encode") | Some("decode") | Some("inspect") | Some("verify")
            | Some("repair") | Some("split") | Some("combine") | Some("derive"))
}
```
and `:433-434`: "If argv is safe where you are … `--allow-argv-secret`
proceeds." Also `:67-80` `SUBCOMMANDS: [&str; 12]` (a fixed-size array, so
adding `hashlock` is a typed edit) — and if it is *not* added, `argv_surface`
returns bare `"ms"`, `refusal()`'s `strip_prefix("ms ").unwrap_or("encode")`
(`:401`) prints a remedy for **`ms encode`**, and the purge `sed` pattern falls
back to the broad form.

**Construction.** `ms hashlock --hashlock-phrase "six diceware words here" --allow-argv-secret`
on the operator's air-gapped box → exit 1 with a message instructing them to add
the flag they just used. Same for `ms hashlock --hex <64 hex> --allow-argv-secret`
and `ms hashlock ms10hash… --allow-argv-secret`, so `--hex HEX` and the `<ms1>`
positional — both presented as sources in §4.2 — are unreachable on argv by any
spelling. (Fail-closed, so no leak; the defect is a tool that states a false fact
about itself and strands two of its own documented channels.)

**Remedy.** The record states the three edits `hashlock` requires in
`argv_guard.rs`: `SUBCOMMANDS` (`[&str; 12]` → 13), `override_applies`'s match,
and `flag_class` (see M-3). One test per edit.

---

### I-2 — "joins `SECRET_FLAGS`" imports a three-part contract; missing the third part derives from whatever stdin happens to carry

**Claim.** Joining `SECRET_FLAGS` does not merely add a refusal: under the
override the flag's value is *replaced by `-`* and routed through a side channel
that the verb must opt into by naming its channel. A verb that builds its
`Source` without `.on("--hashlock-phrase")` reads real stdin instead.

**Evidence.**
- `argv_guard.rs:273-336` `substitute()`: for a token in `SECRET_FLAGS`, the
  value is pushed into `ADMITTED` and the argv value becomes `"-"`.
- `parse.rs:67-85` `Source::read_raw`:
  ```rust
  (None, _) => match crate::argv_guard::admitted(self.channel) {
      Some([first, ..]) => Ok(Zeroizing::new(first.clone())),
      _ => read_stdin(),
  },
  ```
  and `parse.rs:43` — `channel: ""` is the default from `Source::new`, i.e.
  "not a channel the override can admit". `admitted("")` is `None`, so the arm
  falls through to `read_stdin()`.
- Every shipped verb calls `.on(...)` explicitly (`encode.rs:133,151`,
  `derive.rs:387,418,423`, `verify.rs:72,122`, `decode.rs:64`, `inspect.rs:58`,
  `repair.rs:134`); it is a per-call-site discipline with no compiler help.

**Construction.** `ms hashlock --hashlock-phrase "my real phrase" --allow-argv-secret < notes.txt`
(after I-1 is fixed and if `.on("--hashlock-phrase")` is omitted): argv becomes
`--hashlock-phrase -`, `admitted("")` returns `None`, stdin is read, and the
digest is derived from **`notes.txt`**. Exit 0, correct-looking `hash:` record,
a card whose character count matches `notes.txt` and not the phrase. With stdin
at a terminal the same invocation instead blocks forever with no prompt, which
reads as a hang.

**Remedy.** The record names the channel string and says the verb's `Source` is
built `.on("--hashlock-phrase")`; the gate is the shipped one — run the same
invocation with stdin at `/dev/null` (`argv_guard.rs:219-223` describes exactly
this test) and assert the admitted value still arrives.

---

### I-3 — the phrase must not pass through either shipped intake helper, and §4.6 has no test that would notice if it did

**Claim.** §4.2 requires the phrase bytes be used "exactly as typed (no
trimming, case folding or normalisation)". Both intake helpers `ms` ships
mutate the bytes, and either one silently changes X.

**Evidence.**
- `parse.rs:112-115` `read_input` → `strip_whitespace` → `format::strip_display_separators`,
  documented at `:170-177` as stripping "ALL Unicode whitespace PLUS `-` and
  `,`". A phrase `correct-horse battery staple` becomes
  `correcthorsebatterystaple`.
- `parse.rs:124-132` `read_phrase_input` → `normalize_phrase` → trims edges and
  collapses runs. `"  two  spaces "` becomes `"two spaces"`.
- `parse.rs:94-102` `read_in_file` strips **nothing**, so `--in`'s "same newline
  rule" needs code that does not exist yet; only `read_stdin_passphrase`
  (`:139-148`) strips one `\r?\n`, and it is stdin-only.
- §4.6's CLI bullet lists "stdin stripping of exactly one LF or CRLF; `--in
  FILE`" — no byte-exactness assertion. §4.6's *codec* rows do include "a phrase
  with leading, trailing and double spaces", but the codec takes bytes; a
  CLI-layer normalisation passes every codec vector.

**Construction.** Implementer reuses `read_phrase_input` (its name is the closest
match and it already returns `Zeroizing<String>`). Operator's phrase is
`the last plate  rings twice` (a double space). Host derives from
`the last plate rings twice`; the device, whose keyboard stores the fragment
verbatim (`gui/passphrase_flow.go:105-109` `copy(dst, kbd.Fragment)`), derives
from the double-spaced text. Two different digests for one phrase; the operator
discovers it only when the device's `first8..last8` disagrees with the host's —
or, if they used only the host, when the plate and their memory disagree at spend
time. Every test in §4.6 as written passes.

**Remedy.** The record states that the phrase channels use a **new** reader
(bytes verbatim, exactly one trailing `\r?\n` stripped, applied identically to
`--in` and `--hashlock-phrase-stdin`) and explicitly that `read_input` /
`read_phrase_input` must not be used. Add the CLI test rows in the table below.

---

### I-4 — H2 changes the Go payload classifier before its Rust primary, which the record's own §3.6 forbids

**Claim.** §4.1 puts "a payload class for it that reaches no screen" in **H2**
(the fork) and "`me`'s classifier learns the kind on the Rust side (rides the
owed me 0.8.1, F-454)" in **H3**, which §4.5's order places *after* H2, after
the merge, after the flash and after the H4 walk. §3.6 says "Both sides need the
kind, Rust first."

**Evidence.** §4.1 H2 vs H3; §4.5 "Order" — `… H2 implementer … → review →
merge → flash at the operator's word → the H4 walk with the operator → H3 closes
the records (me 0.8.1 with the classifier learning the kind …)`. The project rule
(`CLAUDE.md`, Rust-primary) names "validation, admission" as normative behaviour
that must land in Rust first with test vectors. `sysw/classify.go:11-15` and
`:116-125` declare themselves mirrors of `crates/me-cli/src/seal/record.rs` /
`sysw/mod.rs`, and §3.6 says so too.

**Construction.** As scheduled, the fork ships a class the Rust primary does not
have, and a firmware is flashed on that basis. If the Rust classifier then makes
a different choice (e.g. keeps the string in `ClassCodex32Secret` and refuses
downstream, or picks a different class name/order), the Go port led and the
convergence runs backwards — the exact failure the rule exists to prevent, on the
one behaviour (admission) that decides whether a preimage reaches a seed screen
(C-2).

**Remedy.** Move the `me` classifier change out of H3 and into H1 (or a new H1b)
so it precedes H2, or narrow H2 to "no new class; `isStrictMs1` gains the prefix
test and the string classifies as ClassUnknown/inert" — which is Rust-mirrorable
and fail-closed by `admits`'s absent-is-false default (`gui/sysw_admit.go:88-90`).

---

### I-5 — the `--out` / stdout interaction is unstated, and the nearest precedent does the opposite of what this verb needs

**Claim.** On `ms encode`, `--out` **suppresses** the stdout artifact. On
`ms hashlock` the two channels carry *different* artifacts (preimage vs digest),
so the precedent must be inverted — and the record never says so.

**Evidence.** `crates/ms-cli/src/cmd/encode.rs:207-218` and `:321-332`:
`emit_text(..., artifact_went_to_a_file)` → `if !artifact_went_to_a_file {
println!("{ms1}"); }`, with the reason given in the comment at `:206-213`
("printing it as well would put the same secret on a stream that is usually
redirected into a 0644 one"). §4.2 says only "stdout: one line, `hash:<64 hex>`
… `--out FILE`: the preimage ms1".

**Construction.** Implementer copies encode's shape. `ms hashlock
--hashlock-phrase-stdin --out plate.ms1 < phrase.txt | me sysw pack --in -`
emits **nothing** on stdout; `me sysw pack` receives an empty stream. Visible
rather than silent, but it breaks the record's own headline pipeline, and §4.6's
"stdout is exactly the record line" test is not stated to run with `--out`, so it
would not catch it.

**Remedy.** One sentence in §4.2: `--out` never suppresses the stdout digest
line, because the two channels carry different artifacts. Test row below.

---

### I-6 — the `python3` + `openssl kdf` hard-fail rule names no CI job, and the two candidate jobs differ in a way that decides whether the rule turns CI red

**Claim.** §4.6's "A test executes the `python3` and `openssl kdf` reproductions
and FAILS if either tool is absent, so a skip can never print ok" is the right
rule, but it is unscheduled, and the repository's two candidate jobs run on
different platforms.

**Evidence (measured).**
- `.github/workflows/rust.yml:113-119` — `test (ms-codec)`, `runs-on:
  ubuntu-latest`, `cargo test -p ms-codec`. Single platform.
- `.github/workflows/rust.yml:129-136` — `test (${{ matrix.os }})`, matrix
  `[ubuntu-latest, macos-latest]`, `cargo test -p ms-cli`.
- `.github/workflows/rust.yml:329-357` — `freebsd compile-gate (whole-crate)` at
  toolchain `1.85.0` (compile only).
- No workflow in the repo installs or checks for either tool:
  `grep -rn 'python3\|openssl\|apt-get' .github/workflows/*.yml` returns no
  install step for them (only shells, for the history-purge job at `:274-289`).
- `openssl kdf` is an OpenSSL 3.0+ subcommand (verified here:
  `openssl version` → `OpenSSL 3.6.3`, and the record's hardened vector
  reproduces — see "Sources"). macOS ships LibreSSL as `/usr/bin/openssl`, which
  implements no `kdf` subcommand; I could not exercise a GitHub macOS runner from
  here, so treat that as the risk to confirm rather than as a measured fact.

**Construction.** The reproduction test lands in `ms-cli`'s suite (or `ms-codec`'s
job later gains the macOS matrix row). `cargo test -p ms-cli` on `macos-latest`
fails hard on a missing `openssl kdf`. The predictable repair under time pressure
is a `#[cfg(target_os = "linux")]` or an environment probe — which is exactly the
gate-that-skips-and-prints-ok this rule was written to forbid.

**Remedy.** Name the job in the record: the reproduction test lives in
`ms-codec`, whose job is ubuntu-only, and `rust.yml` gains an explicit
`openssl kdf --help` / `python3 -c 'import hashlib'` preflight step in that job
so absence fails at the step and not inside a test that someone can `#[ignore]`.
Keep the `python3` leg unconditional (stdlib `hashlib.pbkdf2_hmac`, present on
every runner); make the `openssl` leg's presence a CI-asserted precondition
rather than a test-internal probe.

---

### M-1 — the stderr card carries the preimage in text mode with no class advisory, and `ms` has no stdout-mode machinery at all *(class: secret handling — non-gating by the 2026-08-27 ruling)*

`encode` emits `emit_output_class_advisory(PrivateKeyMaterial, stderr)`
unconditionally (`encode.rs:231-234`) because *its* stdout is the secret. On
`ms hashlock` the polarity is inverted — stdout is public, stderr's card is the
secret — so the shipped advisory is correctly silent and **nothing** labels the
card. `ms hashlock … > rec.txt 2>>~/ms.log` and `… 2>&1 | tee session.log` both
put the preimage in a 0644 file while the operator believes they protected the
secret stream. Answering the brief's Q2 directly: of the io-lib machinery, `ms`
uses **only** `write::write_private` and `remedy::history_purge_block`
(`grep -rn 'mode_of\|is_terminal\|fd::' crates/ms-cli/src` → the two hits in
`out.rs:25` and `argv_guard.rs:402` and nothing else), so `--out` mode is handled
(0600 on create *and* on a pre-existing 0644 target, `write.rs:45-65`) and
`--json > file` at 0644 is handled by nothing — F-281 carries whether `ms` should
ever have a stdout mode gate, and this verb does not change that. Suggested
record line: the card's first line names it as carrying the preimage.

### M-2 — JSON-mode errors go to **stdout**, so every new refusal's message and `details()` are an output-channel surface *(class: secret handling)*

`main.rs:276-288`: in `--json` mode the error envelope (`kind`, `message`,
`exit_code`, `details`) is printed on stdout. §4.2 says refusals "name the rule
and never echo the phrase" — good — but the `--hex` refusals carry a preimage,
and nothing in §4.6 asserts the negative for the JSON envelope. Test row below.

### M-3 — `flag_class` has no arm for the new flag, so the refusal misnames what it refused

`argv_guard.rs:378-385`: the match falls through to `_ => "a BIP-39 passphrase"`.
`ms hashlock --hashlock-phrase "…"` would be refused as *"argument 2 on ARGV is
a BIP-39 passphrase, N characters long"* — a false statement about the operator's
material, in the one message whose whole job is to name the class precisely
without echoing the value. One-line fix; list it with the I-1 edits.

### M-4 — the device keyboard constructor is unnamed, and each of the three candidates is wrong in a different way

`gui/passphrase_keyboard.go:80-108`: `NewTextKeyboard` carries a newline key
*and* the settings gear — the file's own comment calls a gear on a secret-entry
screen "not merely useless, it is a defect"; `NewLineKeyboard` keeps the newline
key, which types a character the ASCII rule then refuses. `NewPassphraseKeyboard`
is the right one. Separately, reusing `passphraseEntryFlow`
(`gui/passphrase_flow.go:74-145`) inherits a hard-coded `"Passphrase"` title, the
`PASSPROOF!` trigger (inert with a nil loader — `passphrase_passproof.go:216-219`
returns false when `load == nil`), and `ppEntryError`'s over-length text *"Too
long. At most 100 characters fit on one plate."* — a plate-legibility reason on a
screen where nothing is ever engraved (L7). Name `NewPassphraseKeyboard` and a
new flow function in the record.

### M-5 — `unlockDerive`'s signature cannot express the hashlock salt, and the naive reuse zero-pads it

§4.4 says the countdown "the sealed payload already uses
(`gui/unlock_kdf.go:221-236`)" is reused. That screen's driver is
`gui/unlock_kdf.go:242` `func unlockDerive(ctx *Context, th *Colors, h seal.Header,
pass []byte)` and `seal/wire.go:85` is `Salt [SaltLen]byte` with `SaltLen = 16`
(`wire.go:32`). The hashlock salt `"ms-hashlock-v1"` is **14** bytes; stuffed into
that array and passed as `h.Salt[:]` it becomes 14 ASCII bytes + two `0x00`, and
every device digest silently diverges from the host's. This is caught **only** by
the harness test that compares against the vendored vector — see the test table's
note that it must compare a *constant*, not a value recomputed by the same Go
function the screen calls.

### M-6 — `--method` with `--hex` / `--random` / `<ms1>` is unspecified, and `--json`'s `method` object has no shape for those three sources

§4.2: "`--hex`, `--random` and `<ms1>` take no method: X is given", and
`--json` carries "method {kdf, hash, salt, iterations, dklen} or {hash}". Neither
shape applies when there is no method. Unstated: whether `ms hashlock --hex -
--method sha256` is refused (exit 64) or silently ignores a flag the operator
deliberately set, what the card's method line says, and what `--json` emits —
`null`, an absent key, or a third shape. §4.6's "`--json` schema" test would pin
whatever ships, including the wrong answer.

### M-7 — no phrase channel refuses or prompts on a terminal stdin, so the first thing a new operator types looks like a hang

`ms` calls `is_terminal` nowhere (grep, above). `ms hashlock
--hashlock-phrase-stdin` with no redirect blocks in
`read_stdin`/`read_to_string` (`parse.rs:150-168`) with no prompt. This is the
constellation's recorded `mt` finding ("stdin doesn't mean from the command
line?") and it lands harder here, because the hashlock phrase is the first `ms`
input a human is *meant* to type. Pre-existing shape on `--passphrase-stdin`, so
a documentation-or-prompt call rather than a defect in this design.

---

### N-1 — the record pins the hardened **X** for the W-5 phrase but never its **H**

§4.3 gives `c3e97525…72e22016` for hardened X and both X and H for sha256. The
hardened digest is `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`
(computed below). The corpus should pin H for both methods, since H is what the
policy carries and what the device shows.

### N-2 — H4's live comparison is 64 of 256 bits; say what makes that adequate

§4.4/§4.6: the operator compares `first8..last8` with the host. That is 64 bits,
and 2^64 SHA-256 evaluations is roughly 0.03 s of the Bitcoin network's
hashrate — so it is a transcription check, not a cryptographic one. It is
adequate *because* the full-width lockstep vector runs in CI. Record that
dependency, so nobody later drops the vector and keeps the walk.

### N-3 — `hmac` is not needed as a direct dependency

The brief lists `pbkdf2`, `sha2`, `hmac`; §4.3 correctly names only the first
two. Measured: `pbkdf2-0.12.2/Cargo.toml` `[features] default = ["hmac"]`, and
`me` spells it `pbkdf2 = { version = "0.12", default-features = false, features =
["hmac"] }` (`crates/me-cli/Cargo.toml:45`) with **no** `hmac` line and no
`password-hash` (that is the optional `simple` feature). Copy `me`'s spelling
verbatim so the two consumers cannot drift.

---

## Confirmed sound

**Q1 — input channels, the parts that hold.** The guard runs before
`Cli::try_parse_from` (`main.rs:190-198`), so a new secret flag added to
`SECRET_FLAGS` is refused on raw argv with no clap echo — the "a guard downstream
of the parser has lost" lesson is structurally satisfied for this verb *provided
the flag joins the list*. `argv_candidates` (`argv_guard.rs:104-111`) normalises
four ways including every `=`-split half, so `--hashlock-phrase=<phrase>` is
reached (F-302's class). A flag with no value (`ms hashlock --hashlock-phrase`)
is `continue`d at `:347-352` and lands on clap's "a value is required" with no
material to echo. A UTF-8 BOM, an interior CR, or a second line in an `--in` file
are all non-printable-ASCII and refused by §4.2's rule, fail-closed; an invalid
UTF-8 file is refused earlier by `read_to_string` (`parse.rs:95`). Empty stdin
gives an empty phrase, refused by the non-empty rule. A 101-character phrase is
refused on every channel.

**Q1 — two channels claiming stdin.** `Source::read_raw`'s `(Some(p), Some(_))`
arm (`parse.rs:68-73`) is a runtime backstop behind clap's `ArgGroup`, and §4.6
already pins "two sources exit 64". Sound, but the record should name the
mechanism (one `ArgGroup` over all six sources) rather than the outcome.

**Q3 — fail-closed placement, host.** Every refusal §4.2 names is an
input-validation refusal that precedes derivation, the `--out` write and the
stdout emit; the only ordering the record must fix is `--out`-before-stdout for
`--random` (C-3). The panic surfaces round 0 identified are addressed: the four
`_ => unreachable!` sites are an enumerated H1 checklist with the `decode`/
`combine` split (I-3 as folded, verified by r1), the `0x03` length check precedes
construction with `try_from` rather than `data[1..33]` (I-2), and `getrandom`'s
`.expect` (`shares.rs:43`) fails closed — a panic cannot emit a weak preimage.

**Q3 — fail-closed placement, device.** Back during the 10-second derivation is
safe by construction: `unlockDerive` (`gui/unlock_kdf.go:242-260`) returns
`(nil, false)` on Button1 with `defer d.Wipe()` registered before anything can
return, and the path's hash is only assigned on the confirm modal's CONTINUE
(`composer_hash.go:163,170`) — so Back and power loss mid-derivation both leave
the composer state untouched. Choosing the wrong method and confirming is
recoverable: the method is on the card and in the confirm copy, and L5's "try
each method that shipped with the version named on this card" resolves a 1-bit
ambiguity.

**Q3 — the payload-substitution attacker.** For the **new** leg there is nothing
to substitute: the digest in the confirm modal is one the device computed from a
phrase the operator just typed, so no attacker-supplied value is on the
comparison surface. The `first8..last8` elision applies to the pre-existing
`hash <i>` payload rows (`composer_hash.go:38-41`), which this design does not
change; see N-2 for what 64 bits is and is not worth.

**Q4 — the device's over-length signal is real, and the 101-character row is
constructible.** `gui/passphrase_flow.go:115-119` deliberately does *not* clamp
("Over-length is shown rather than clamped: silently dropping keystrokes at 100
would leave the operator believing a longer passphrase had been entered") and
`TestPassphraseEntryFitsPanel` guards the counter against occlusion. This is the
opposite of `composerHexEntry`'s truncation (`composer_hash.go:77-79`), and it is
the right half to copy. M-6's lockstep 100/101 rows therefore bind on both sides.

**Q4 — a new class reaches no screen by construction.** `gui/sysw_admit.go:88-90`
`admits(p, c) { return admitted[p][c] }` — an absent class is `false` for every
program, so §4.4's "reaches no screen" is structural rather than a list to
maintain. (What is *not* structural is getting the string classified into the
new class at all — C-2.)

**Q5 — the copy carries what the tool cannot detect.** §3.7's four consequences,
the `--random` both-halves line, the brainwallet line, the character count, and
the method line are the right set, and the tool genuinely cannot detect reuse
across policies or passwords. The misuse sequences I could construct that the copy
does *not* prevent are C-1, C-3 and C-4, all of which have refusals as remedies
rather than more copy.

**Q6 — supply chain.** `pbkdf2 0.12.2` declares `rust-version = "1.60"` and its
default feature set is exactly `["hmac"]` (measured above), so `me`'s
`default-features = false, features = ["hmac"]` and a bare `pbkdf2 = "0.12"`
resolve identically — no `password-hash`, no `base64ct`, no `rand_core`. Both
crates are pure Rust with no C and no build script that would trouble the
`reproducible-musl-build` job (`man-release.yml:134-142`) or the FreeBSD
compile-gate at 1.85.0 (`rust.yml:329-357`); the workspace pins
`rust-toolchain.toml` `channel = "1.85.0"`, above both MSRVs. `ms-codec` has no
`[features]` table (`rust.yml:126` states this) and is not `no_std` (it already
depends on `getrandom`), so no feature-flag work is implied. The exact-pin policy
is already in place: `ms-cli/Cargo.toml:19` `ms-codec = { path = "../ms-codec",
version = "=0.7.0" }` → `=0.8.0`, and `me-cli`'s `ms-codec = "0.7"` caret will
not silently follow. The fork's `golang.org/x/crypto/pbkdf2` is already vendored
(`slip39/feistel.go`, `cmd/kdfbench`).

**Q7 — PASSPROOF! cannot leak in by accident.** `ppPassProofOffer`
(`gui/passphrase_passproof.go:216-219`) returns `false` when `load == nil`, and
`load` is a parameter of `passphraseEntryFlow` rather than a package global — so
a hashlock flow that passes `nil` (or that does not call it at all) cannot load
the 95-character public test pattern. The gear key is likewise a per-instance
opt-in (`passphrase_keyboard.go:96-108`). The residue is naming the right
constructor and flow — M-4, not a leak.

**Q7 — the countdown screen carries no cross-flow state.** `unlockDerive` takes
its salt/iterations as parameters and holds only a local `deriver`; the only
state is the `seal.Header` shape, which is M-5.

**Q8 — everything else I would refuse to sign off on is C-1 … C-4 and I-1 …
I-6.** The KDF choice, the kind's placement in ms-codec, the share axis, the id
`hash` decision, the method warnings and the §3.7 copy are all sound as folded;
`RESERVED_ID_BLOCKLIST` (`consts.rs:71`,
`&[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk"]`) takes `*b"hash"` as a
one-line edit, and it is consulted only where a random share-set id is chosen
(`shares.rs:50`) — worth one record sentence that the id on a single is a display
convention that `decode` does not validate, so I-4's mitigation rests on
`ms hashlock` being the only door that creates the kind.

---

## Test plan additions

| test | asserts | the mutation it catches | stage |
| --- | --- | --- | --- |
| `hashlock --in` pointed at a preimage ms1 file | exit non-zero, refusal names the `<ms1>`/`-` route, no `hash:` line on stdout | delete the ms1-shaped check on the phrase channels (C-1) — today's design has no such check, so this test is RED against the record as written | H1 |
| the same, for `--hashlock-phrase` and `--hashlock-phrase-stdin` | identical refusal on all three phrase channels | fix one channel and not the other two | H1 |
| `hashlock --random --no-engraving-card` with no `--out`/`--json` | exit non-zero, refusal names `--out`; and the `2>/dev/null` spelling likewise | remove the "preimage must reach a persistent channel" guard (C-3) | H1 |
| `hashlock --random --out <existing file>` | either refuses or the record's chosen clobber rule, asserted | make `--out` silently truncate for `--random` | H1 |
| `hashlock --hashlock-phrase X --allow-argv-secret` | exit 0 and derives from `X`; then the same invocation with stdin at `/dev/null` still derives from `X` | drop `hashlock` from `override_applies` (I-1); drop `.on("--hashlock-phrase")` (I-2) — the `/dev/null` half is the one that fails on I-2 | H1 |
| `hashlock --hashlock-phrase X --allow-argv-secret < other.txt` | derives from `X`, never from `other.txt` | I-2's missing `.on()` — this row fails where the previous one could pass | H1 |
| argv-guard refusal text for `ms hashlock` | contains "hashlock" (not "encode"), and the class named is not "a BIP-39 passphrase" | `SUBCOMMANDS` not extended (I-1); `flag_class` arm missing (M-3) | H1 |
| byte-exact phrase round trip: `"  a  b "` (leading, interior double, trailing spaces) and `"a-b,c"` through `--in` and `--hashlock-phrase-stdin` | derived X equals the codec vector for those exact bytes | swap the reader for `read_phrase_input` (collapses runs) or `read_input` (strips `-` and `,`) — I-3; **no codec vector can catch this** | H1 |
| `--in FILE` newline rule | `"p\n"`, `"p\r\n"`, `"p"`, `"p \n"` → one trailing `\r?\n` stripped and nothing else; `"p\n\n"` refused (interior LF) | reuse `read_in_file` unchanged (strips nothing) or `read_input` (strips all) | H1 |
| negative content, one row per refusal (empty, non-ASCII, >100, 64-hex, ms1-in-phrase-slot, `--hex` wrong length, wrong ms1 kind, two sources) | the phrase and the preimage appear in neither stdout, stderr, nor the `--json` error envelope | any refusal built with `format!("… {phrase}")`; catches the stdout-JSON envelope path (`main.rs:276-288`, M-2) that a stderr-only assertion misses | H1 |
| `stdout is exactly the record line` run **with** `--out` as well as without | the `hash:` line is printed in both cases | copy `encode`'s `if !artifact_went_to_a_file` suppression (I-5) | H1 |
| `--method` supplied with `--hex` / `--random` / `<ms1>` | the record's chosen behaviour (refuse, or ignore) asserted, plus the `--json` `method` shape for those sources | silently ignoring an operator-set flag; an undefined `method` key (M-6) | H1 |
| `ms hashlock` under ms-cli 0.17.x-equivalent codec (downgrade row) | a `0x03` string refuses with `ReservedPrefixViolation`, never panics | any of the four `unreachable!` sites reached before the refusal (I-3, already planned — keep it) | H1 |
| CI preflight in the `test (ms-codec)` job | `openssl kdf --help` and `python3 -c 'import hashlib'` succeed as a *step*, before any test runs | an `#[ignore]`/`cfg` skip added to the reproduction test when a platform lacks the tool (I-6) | H1 |
| Go `DecodeMS1` length rows for `0x03`: 16, 20, 24, 28, 34, 46 payload bytes | each refused by name | let the `0x03` arm fall into the shared `case 16, 20, 24, 28, 32` switch (`mspayload.go:54-58`) — the Rust length rows do not cover the Go switch | H2 |
| every `DecodeMS1` call site, one test each: `ms1_decode.go:22`, `codex32_polish.go:106`, `singlesig_verify.go:185`, `multisig_verify.go:1237`, `bundle/verify.go:138` | a `0x03` string is refused by name at each, and "Show secret" is not offered | the `case 0x03:` arm returning `nil` error while one site keeps `_` for the prefix (C-2) — **5 of 5 sites discard it today** | H2 |
| `sysw.Classify` on a `0x03` ms1 | the new class (or ClassUnknown), never `ClassCodex32Secret`, and `admits(p, class)` is false for all ten programs | leave `isStrictMs1` unchanged, which matches first (C-2) | H2 |
| `Which hash?` row behaviour for **every** row, by label: each payload digest, `Type a hashlock phrase`, `Type 64 hex`, `No hash lock` | each row reaches its own screen; `Type 64 hex` sets a hash and does not clear one; the §8i modal fires for all three take-a-hash rows and not for `No hash lock` | insert the new row without renumbering the `switch` (C-4) — the presented plan tests only the new row | H2 |
| device derivation harness test compares against the **vendored corpus constant** | the digest equals the literal hex from the Rust-derived corpus file, not a value recomputed by calling the Go derivation | zero-padding the 14-byte salt into `seal.Header.Salt [16]byte` (M-5) — a self-recomputed expectation passes in both worlds | H2 |
| device phrase screen widget identity | no gear key, no newline key, title is `Hashlock phrase`, the over-length message does not mention a plate | building it with `NewTextKeyboard`/`NewLineKeyboard` or reusing `passphraseEntryFlow` (M-4) | H2 |
| device 101-character and 64-hex refusals driven through the real screen | refused, with the counter showing `101/100` | copying `composerHexEntry`'s truncation (`composer_hash.go:77-79`) instead of the passphrase flow's non-clamping counter | H2 |
| H4 walk records BOTH methods' full 64-hex digests, not only `first8..last8` | the device's full digest equals the host's | dropping the CI lockstep vector and relying on the 64-bit human comparison (N-2) | H4 |

---

## Questions for the operator

1. **Should `ms hashlock` have an `--in` flag at all?** C-1's cheapest fix is to
   delete it and give the ms1 source its own flag, so no spelling of "read it
   from a file" is ambiguous across the ten verbs. The cost is one more flag name
   and a divergence from `encode`/`split`.
2. **Should `--random` be allowed to run without `--out`?** C-3's remedy makes
   `--out` (or `--json`) mandatory under `--random`. The alternative is to keep
   the flag combination legal and accept that the card is the only copy, which is
   a data-loss risk the tool could refuse instead.
3. **`--method` with a source that has no method: refuse, or ignore?** Refusing
   costs a keystroke; ignoring means a flag the operator deliberately set does
   nothing, which this codebase elsewhere calls a defect
   (`passphrase_keyboard.go:100-108`).
4. **Does H2 ship a new payload class, or does the preimage stay ClassUnknown
   until `me` 0.8.1 defines the class?** I-4 is a scheduling question with a
   Rust-primary rule attached, and the second answer is both compliant and
   fail-closed.
5. **Should the stderr card's first line say that it carries the preimage?**
   Secret-handling class and therefore non-gating, but the polarity inversion
   (stdout public, stderr secret) is unique to this verb among `ms`'s ten.

---

## Sources consulted

**Commands run on this machine (output pasted in the findings above)**
```
$ git -C /scratch/code/shibboleth/mnemonic-engrave log -1 --format=%h -- design/BRAINSTORM_hashlock_phrase.md
82433fd

$ cd /scratch/code/shibboleth/seedhammer && grep -rn -B1 'codex32.DecodeMS1(' --include=*.go . | grep -v '_test.go' | grep ':=\|='
gui/ms1_decode.go:22:   _, language, entropy, err := codex32.DecodeMS1(scan)
gui/codex32_polish.go:106:      _, _, ent, msErr := codex32.DecodeMS1(scan)
gui/singlesig_verify.go:185:            _, _, ent, err := codex32.DecodeMS1(s)
gui/multisig_verify.go:1237:    _, _, ent, err := codex32.DecodeMS1(s)
bundle/verify.go:138:   _, language, ent, err := codex32.DecodeMS1(str)

$ cd /scratch/code/shibboleth/mnemonic-secret && grep -rn 'mode_of\|is_terminal\|fd::' crates/ms-cli/src/
crates/ms-cli/src/out.rs:25:    mnemonic_io_lib::write::write_private(path, body.as_bytes())
crates/ms-cli/src/argv_guard.rs:402:    let purge = mnemonic_io_lib::remedy::history_purge_block(&surface);
   (no fd::mode_of, no is_terminal — `ms` has no stdout mode gate)

$ openssl version
OpenSSL 3.6.3 9 Jun 2026 (Library: OpenSSL 3.6.3 9 Jun 2026)

$ openssl kdf -keylen 32 -kdfopt digest:SHA256 -kdfopt "pass:correct horse battery staple" \
      -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2
c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016     (lowercased, colons stripped)

$ python3 -c "
import hashlib
x=hashlib.pbkdf2_hmac('sha256', b'correct horse battery staple', b'ms-hashlock-v1', 100000, 32)
print(x.hex()); print(hashlib.sha256(x).hexdigest())
s=hashlib.sha256(b'correct horse battery staple').digest()
print('sha256 X', s.hex()); print('H', hashlib.sha256(s).hexdigest())"
c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
sha256 X c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a
H b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb

$ python3 -c "
CS='qpzry9x8gf2tvdw0s3jn54khce6mua7l'
print('ms1-shaped tail all in charset:', all(c in CS for c in 'ms10hash'[3:]))
print('len of a 33-byte payload ms1:', 9 + -(-33*8//5) + 13)"
ms1-shaped tail all in charset: True
len of a 33-byte payload ms1: 75

$ sed -n '/^\[features\]/,/^\[/p' ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pbkdf2-0.12.2/Cargo.toml
default = ["hmac"]   (simple = ["hmac","password-hash","sha2"]; parallel = ["rayon","std"])
```
Confirms §4.3's hardened X and §7's W-5 digest byte-for-byte; adds the hardened
**H** the record does not pin (N-1).

**mnemonic-engrave (`82433fd`)** — the brainstorm; the two prior R0 reports;
`crates/mnemonic-io-lib/src/{write,fd,channel}.rs`; `crates/me-cli/Cargo.toml`.

**mnemonic-secret (`7fc1e58`)** — `crates/ms-cli/src/{argv_guard,parse,out,advisory,main}.rs`;
`crates/ms-cli/src/cmd/{encode,decode,derive,payload_lang,verify,inspect,repair,split,combine}.rs`
(the last six by grep for `--in`, `Source::new`, `.on(`);
`crates/ms-codec/src/consts.rs`, `Cargo.toml` (both crates), `rust-toolchain.toml`;
`.github/workflows/{rust,man-release,fuzz-smoke,vendor-freshness}.yml`.

**seedhammer fork (`70008da5`)** — `gui/{composer_hash,composer_sources,ms1_decode,codex32_polish,passphrase_flow,passphrase_keyboard,passphrase_passproof,unlock_kdf,sysw_admit}.go`;
`gui/composer_gates_test.go:655-700`; `passphrase/passphrase.go`;
`codex32/mspayload.go`; `sysw/{classify,record}.go`; `seal/wire.go:32-36,85`.

**Crates** — `pbkdf2-0.12.2/Cargo.toml` (features, MSRV).

**Not re-derived, per the brief** — the KDF construction, the script facts, the
guessing rates, the kind byte, and rulings L1–L19. Round 0's arithmetic was
reproduced by the controller and by r1; I re-ran only the two derivations above
because they are the vectors this design's tests will pin.
