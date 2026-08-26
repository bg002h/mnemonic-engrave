# R0 — architect review, round 0

**Artifact:** `design/SPEC_constellation_cli_uniformity.md` (DRAFT, 2026-08-26)
**Question asked:** *If a competent implementer built exactly what this spec says, what would go wrong?*
**Reviewer:** independent context. Every claim below was measured against the built
binaries or the source on 2026-08-26; no behaviour is described from help text alone.

Binaries used:

```
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md
/scratch/code/shibboleth/mnemonic-key/target/debug/mk
/scratch/code/shibboleth/mnemonic-secret/target/debug/ms
/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/target/debug/mt
/scratch/code/shibboleth/mnemonic-engrave/target/debug/me
```

**Verdict up front: NOT GREEN. 4 Critical / 11 Important / 6 Minor / 2 Nit.**

---

## FIRST: the spec's own open item, §8 bullet 1, is now closed — and the answer is "no"

The brief's highest-value task was to get `mk encode` to run. It runs. It needs
`--xpub` + `--origin-path` + one of `--policy-id-stub` / `--from-md1`:

```
$ mk encode --xpub xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc \
    --origin-fingerprint aabbccdd --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344
mk1qp swajp qqsq3 zg3ng j4thn xaq5z g3vs7 zqsrq qdt4w 46h2a t4w46 h2at4 w46h2 at4w4 6h2at 4w46h 2at4w 46h2a t4vp3 k25gs rttm4 zzk4z 4
mk1qp swajp psnz4 v7cjv 3qfjh f76k4 t5pt9 6u0ps drqfq vll8q h7h5a thg83 7pmkf 3dh52 0skns lwyt0
[stderr] note: stdout is watch-only — public keys only, cannot spend
exit 0
```

**`mk` emits NO `chunk-set-id:` header, ever** — not even on a 2-chunk set. Confirmed in
source: `mk` has `--chunk-set-id` only as an *input* flag
(`mnemonic-key/crates/mk-cli/src/cmd/encode.rs:73,105,356`); there is no `println!` of a
header anywhere in `mk-cli`. `md`'s header is emitted at exactly one site,
`descriptor-mnemonic/crates/md-cli/src/cmd/encode.rs:172`, and **only on the chunked
branch**:

```
$ md encode 'wpkh(@0/<0;1>/*)'                  # stdout: md1yq pqqxq q8xtw hw4xw n4qh   (no header)
$ md encode 'wpkh(@0/<0;1>/*)' --force-chunked  # stdout: chunk-set-id: 0x6386e
                                                #         md1fv wrwqq pqqgq psqqq 3uaau 4ctxy l7
```

Consequences that the spec has wrong and that P3 inherits — see **I-1** and **I-9**.

§8 bullet 2 also closes in one command, negatively: `--from-md1` is documented
*Repeatable*, and repeating it accepts a 4-chunk set fine (**M-6**). There is no defect
there and it does not belong in scope.

---

# CRITICAL

## C-1 — §10 / D1: the mandated composition turns an upstream REFUSAL into a silently incomplete payload, at exit 0

**The defect.** §10's acceptance form is a brace group of three producers feeding one
`me sysw pack`; when *one* producer refuses, the group still exits 0, `pack` still exits 0,
and the operator gets a payload with a record silently missing.

**Evidence.** Same three producers, twice, differing only in whether `mt`'s input is valid:

```
# all three succeed
{ md encode 'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --group-size 0
  mk encode --xpub xpub6Den8… --origin-fingerprint aabbccdd \
            --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344 --group-size 0
  mt encode --qr --quiet < tx.hex
} | me sysw pack --out payload2.bin
PIPELINE EXIT=0    -rw------- 954 payload2.bin

# identical, except mt is handed a file containing "deadbeef"
{ …same md… ; …same mk… ; mt encode --qr --quiet --in badtx.hex } | me sysw pack --out payload3.bin
PIPELINE EXIT=0    -rw------- 272 payload3.bin

$ me sysw show payload3.bin
public record 0: md1/mk1 — confirmed
public record 1: md1/mk1 — confirmed
public record 2: md1/mk1 — confirmed
```

`mt` refused (exit 1, 0 bytes on stdout, a full refusal on stderr). `me sysw pack` printed
**nothing** about a missing record and exited 0. The transaction is not in the container.
Substitute `mk` for `mt` and the missing record is a cosigner card — a backup the operator
believes is complete and that cannot restore the wallet.

**Why this is the sharpest finding in the review: the defence exists and D1 steps around it.**
`me` already reasoned about this exact failure and closed the *total* case:

```
$ : | me sysw pack --out empty.bin
exit 2
me: no records on stdin: pass them on argv, with --in, or on stdin.
      An EMPTY input is what a FAILED upstream command leaves behind -- `mt encode --qr > rec.txt`
      writes nothing when it refuses -- so it is refused here rather than packed into a container
      that holds nothing and still flashes.
```

That message names the mechanism precisely. §10's three-producer group guarantees the input
is never empty, so the guard never fires. §2 of the spec asserts *"The pipeline-safety
invariant already holds constellation-wide … It is the one thing that did not need
fixing."* That is true for **one** producer and false for the composition D1 mandates.
Zero bytes from producer 3 of 3 is not silence — it is a shorter payload that packs clean.

A partially-degraded case exists too, and it is only slightly better: feed `pack` 1 of a
2-chunk `mk1` set and it warns but still writes and still exits 0 —
`me: record 0 … an md1/mk1 this tool could not decode; the device will treat it as a SECRET`,
`public record 0: md1/mk1 — unconfirmed`, exit 0.

**What would close it.** The defect is that D1's composition has no record-count contract.
I am not certain of the best fix. Candidates the spec should rule between: `me sysw pack`
grows a `--expect-records N` (or `--expect md=1,mk=2,tx=1`) that the operator states and
`pack` enforces; or §10 abandons the brace group for a form whose exit status is the
conjunction (`set -o pipefail` does not help — the failure is *inside* the group, not in the
pipe); or each producer emits a self-describing count line that `pack` reconciles. What is
NOT acceptable is shipping §10 as the acceptance criterion while a refused producer yields
exit 0.

---

## C-2 — §10: the acceptance criterion cannot be satisfied by the changes §6 describes. Two of its three commands name inputs the tools refuse

**The defect.** §10 defines "done" as a pipeline that has never been run and cannot run,
because `--in FILE` is specified as *"read material from a file"* and neither `md` nor `mk`
accepts the material those filenames imply.

**Evidence.**

`md encode --in wallet.desc` — a `.desc` file holds a concrete output descriptor. `md
encode` refuses one:

```
$ md encode "wsh(multi(2,[aabbccdd/48h/0h/0h/2h]xpub6Den8…/<0;1>/*,[deadbeef/48h/0h/0h/2h]xpub6Bme…/<0;1>/*))"
exit 1
md: template parse error: template contains no @i placeholders
# identical with a #checksum appended
```

`md encode` takes a **BIP-388 template with `@i` placeholders**, plus `--key`/`--fingerprint`
flags. Nothing in §6 gives it descriptor parsing.

`mk encode --in cosigner1.xpub` fails twice over. A bare xpub is refused by mk's existing
file reader, which demands BIP-380 origin notation:

```
$ mk encode --keys bare.xpub --policy-id-stub 11223344
exit 64
error: bare.xpub:1: expected BIP-380 origin notation `[fingerprint/path]xpub`, got "xpub6Den8…"
```

and `mk encode` **requires a policy binding no `.xpub` file carries**:

```
$ mk encode --xpub xpub6Den8… --origin-fingerprint aabbccdd --origin-path "m/48'/0'/0'/2'"
exit 64
error: at least one of --policy-id-stub or --from-md1 is required
```

So the §10 line `mk encode --in cosigner1.xpub` cannot produce a card under any reading of
`--in` that §6 supplies. It also means §10's `md` and `mk` steps are **not independent
producers**: a real cosigner card needs the md1 set from the first command as
`--from-md1` arguments, which is not a pipeline shape at all.

Only the third line works today, and it is the one the spec did not need to change:

```
$ mt encode --qr --quiet < tx.hex | head -c 20
tx:02000000000102935      exit 0
```

**What would close it.** The defect is that the acceptance criterion was written from the
shape the spec wants rather than from a run. Either §10's inputs are restated as what the
tools actually consume (a template file; an origin-notation key file plus a policy binding),
or the spec takes on descriptor/xpub ingestion as scoped work with its own phase — but it
cannot leave a criterion that no implementation of §6 can satisfy. Per the project rule, *a
gate that has never executed is a hypothesis, not a gate.*

---

## C-3 — §6.4: generalising `me`'s terminal refusal to `ms` and `mt` refuses their primary use case, and the gate it installs does not prevent the exposure it names

**The defect.** §6.4 bullet 1 lifts F-253 — *"a terminal is refused for any artifact that is
bearer or secret"* — from `me` to all four. `me`'s own refusal states a reason that is
specific to a binary container and does not generalise; `mt`, the bearer tool the spec
generalises *from*, deliberately prints to a terminal today.

**Evidence.** `me`'s refusal, run on a pty:

```
$ script -qec "me sysw pack --in records.txt" /dev/null
exit 2
me: stdout is a TERMINAL, and this payload is BEARER.

Writing it here would paint raw binary across your scrollback — and terminal sessions are
often logged. Nothing was written.
```

The load-bearing clause is **"paint raw binary across your scrollback"**. `md1`/`mk1`/`ms1`/
`mt1` strings and a `tx:` record are short printable ASCII that a human must *read* in order
to hand-engrave.

`mt` on a pty does not refuse — it prints all nine strings and exits 0:

```
$ script -qec "mt encode --quiet --in tx.hex" /dev/null
exit 0
…
mt1pgej7qqgqqqqgqqqqqqqypfx5rtme6rurwj0eyn82d5t9cde9ac74du7p7kvy7jmv7shd7r68ewudpmt4zggf
… (9 strings)
```

`mt`'s bearer-exposure warning fires only on the *opposite* condition — it says *"stdout is
not a terminal, so the strings went somewhere that keeps them"*. `mt` treats the terminal as
the **safe** disposal, and the file as the dangerous one. §6.4 inverts that for the exact
tool it cites as the reference.

**And the gate does not close the hole it names.** Refusing a terminal directs the operator
to `--out FILE`. To then hand-engrave, they must read the file — `cat secret.txt` — which
puts the material on the same terminal with no gate in the way, and leaves a copy on disk
that would not have existed. The net effect of §6.4 bullet 1 on `ms encode` is to convert a
screen-only exposure into a screen exposure **plus** a disk artifact.

**What would close it.** The defect is that §6.4 lifted a rule without its reason. The
reason F-253 gives is *binary output garbling a logged scrollback*; that predicate is false
for all four CLIs. If a terminal gate is wanted for `ms` at all it needs its own
justification and its own remedy (something the operator can act on without writing the
secret to disk), and `mt`'s current terminal behaviour has to be named as a deliberate
exception or deliberately reversed — the spec currently does neither, and P1 asserts the
opposite (**I-2**).

---

## C-4 — §6.3: adding `--allow-argv-secret` to `mt` reopens the leak `mt` closed, unless the pre-clap ordering is carried across — and §6.3 does not state it

**The defect.** §6.3 says *"`mt` gains the override too, for uniformity."* `mt`'s argv guard
is correct **only because it runs before clap**. An implementer wiring
`--allow-argv-secret` as an ordinary clap flag — the obvious implementation — moves the
decision after clap and reinstates the exact leak.

**Evidence.** `mt-cli/src/main.rs:219-238`:

```rust
fn main() -> std::process::ExitCode {
    // §8.2f RUNS BEFORE CLAP, and that ordering is the whole refusal.
    //
    // `mt encode <hex>` never reached this guard when it sat inside `encode`:
    // clap rejects the unexpected positional argument first, and **clap's error
    // message echoes the entire bearer transaction back to stderr**. So the
    // refusal written to stop a bearer artifact leaking into `ps` and shell
    // history leaked it itself, through the argument parser, with no refusal, no
    // purge command and no warning.
    let argv: Vec<String> = std::env::args().collect();
    if let Err(refusal) = validate::command_line_guard(&argv) { … }
    let cli = Cli::parse();
```

Clap's echo is live — reach it with a token the guard does not classify as a transaction:

```
$ mt encode --qr deadbeefcafe
error: invalid value 'deadbeefcafe' for '[-]'
  [possible values: -]
```

The value is printed back verbatim. Any material admitted past the guard as a positional is
one unrelated clap error away from being echoed.

Separately, §6.3's *"Never echo the argument"* bullet is stated as a property of the
refusal's **text**. It is really a property of **where the check sits in the process**. A
spec that states only the first will be implemented as the first.

**What would close it.** §6.3 must make the ordering normative — the guard, and the override's
own parse, run on raw `std::env::args()` before any argument parser sees the material — and
it must say what happens to the material once the override admits it (does it still reach
clap as a positional, where a later error can echo it?). This is the one place in the spec
where uniformity is being applied *to* the tool that got it right, so the constraint that
made it right has to travel with it.

---

# IMPORTANT

## I-1 — §2, §7 P3, §8: the `mk` row is wrong in a way that changes P3's content, and `md`'s header is conditional

**The defect.** §2 marks `mk`'s non-artifact-lines-on-stdout as *unverified*; §7's P3 bundles
`md, mk — header off stdout`. Measured, `mk` has no header to remove, and `md`'s header
appears only when the encoding chunks.

**Evidence.** See the head of this report: `mk encode` on a 2-chunk card emits two `mk1`
lines and nothing else; `grep -rn "chunk-set-id" mk-cli/src` finds only the input flag;
`md`'s single emission site is `md-cli/src/cmd/encode.rs:172`, inside the chunked branch, and
`md encode 'wpkh(@0/<0;1>/*)'` prints no header.

This matters beyond bookkeeping: P3's gate is *"`md encode | me sysw pack` runs with no flags
and no grep"*. A gate written against the **unchunked** case passes without the header defect
ever being exercised. The gate must use a policy that chunks — measured, a keyed 2-of-2 does:

```
$ md encode 'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --key @0=xpub6Den8… --key @1=xpub6Bme… \
     --fingerprint @0=aabbccdd --fingerprint @1=deadbeef --group-size 0
chunk-set-id: 0xc50b9
md1fc59epspqggqpsgvzzshsj4thnxaaatd7au2uzf2at4w46h2at4w46h2at4w46h2at4wsjw2t0e65cpacg
… (4 md1 lines)

$ md encode … | me sysw pack --out p.bin
exit 4
me: record 0 (records count from 0) is not a form this container can place …
```

**What would close it.** Correct §2's `mk` row to "none" and `md`'s to "`chunk-set-id:`,
chunked output only"; strike §8 bullet 1; and pin P3's gate to a chunking policy, since the
unchunked case cannot fail.

## I-2 — §7 P1: "`mt`'s suite unchanged, 236 tests" is not a sound gate, because adopting the crate necessarily changes `mt`'s surface

**The defect.** P1 claims *"Least risk: it already behaves this way, so a behaviour change
here is a bug in the crate."* Three parts of §6 change `mt` regardless of the crate's
correctness, so "suite unchanged" is unsatisfiable and, as a gate, will be met by weakening
§6 rather than by proving the port.

**Evidence.** The count is right — `grep -rc "#\[test\]" --include='*.rs' crates` over
`mnemonic-transaction` totals **236**. What is wrong is "unchanged":

1. **`mt` has no `--out`, deliberately, and says so in a shipped refusal** that §6.1 would
   falsify:

   ```
   mt encode: REFUSED — §8.2h, stdout is a file of mode 0644 …
     mt has no --out: stdout IS the strings, by design (§3b). So the
     remedies are the shell's:
       umask 077 then re-run; the shell creates it 0600
       chmod 600 <file> then re-run -- `>` truncates but keeps the mode
       --allow-world-readable proceed anyway
   ```

   §6.1 mandates `--out` for every tool. That is a design reversal against the very section
   (`SPEC_mt_v0_1` §3b) that §4 adopts as the constellation's principle, and it invalidates a
   tested refusal string.

2. **`--allow-argv-secret` is new surface on `mt`** (§6.3 says so explicitly).

3. **§6.4's F-246 clause does not describe `mt` today.** §6.4 states *"No line describing the
   artifact prints until every gate that can abort the write has run."* Measured, `mt encode`
   prints its complete report — `TX`, `OUT`, `FEE`, `LOCKTIME`, `INPUTS`, `STATUS`, `CUT`,
   `PREFIX`, and the full suggested legend — **before** the destination refusal:

   ```
   $ mt encode --quiet --in tx.hex > out.txt      # out.txt is 0644
   … TX 4665e0aa… / OUT 1 output(s) / CUT 9 strings, 787 characters / PREFIX … / SUGGESTED LEGEND …
   mt encode: REFUSED — §8.2h, stdout is a file of mode 0644 …
   exit 1, 0 bytes on stdout
   ```

   (F-246's actual title is narrower than §6.4's restatement — `design/FOLLOWUPS.md:10344`:
   *"`me sysw pack` generates and PRINTS a passphrase before it validates the records"*. It
   is about emitting *secret material*, not about any line describing the artifact. §6.4
   generalised it silently, and the generalised form is a real change to `mt`.)

**What would close it.** P1's gate has to become "`mt`'s suite passes, with the diff to it
enumerated and each edit justified by a named §6 change" — and §6.1 has to rule explicitly on
whether `mt` gains `--out`, given that `SPEC_mt_v0_1` §3b rules it out and the spec cites §3b
as authoritative eight lines earlier.

## I-3 — §6.3 / §7 P2: `ms` has **eight** argv channels for secret material, not one — and `ms combine` has no other input channel at all

**The defect.** §1 and §6.3 discuss only `ms encode --phrase`. D3 refuses *secret/bearer
material on argv*; `ms1` strings and codex32 shares are seed-equivalent, so the rule reaches
almost every `ms` verb. P2's gate would pass with a funds-critical verb bricked.

**Evidence.** Every `ms` verb's usage line:

```
ms encode  [OPTIONS] <--phrase <PHRASE>|--hex <HEX>>
ms decode  [OPTIONS] [MS1]
ms verify  [OPTIONS] [MS1]
ms inspect [OPTIONS] [MS1]
ms repair  [OPTIONS] --ms1 <MS1>
ms split   [OPTIONS] --threshold <K> --shares <N> <--phrase <PHRASE>|--hex <HEX>>
ms combine [OPTIONS] <SHARES>...
ms derive  [OPTIONS] [MS1]
```

`ms1` is seed material — `ms decode <ms1>` prints the mnemonic:

```
$ ms decode ms10e-ntrsq-…-34v7f
entropy: 00000000000000000000000000000000
phrase: abandon abandon … about
```

**`ms combine <SHARES>...` takes its shares ONLY as argv positionals.** It has no `--in`, no
`-`. Refusing argv there without first landing §6.1's `--in`/`-` removes the *only* way to
recombine split shares — the recovery path, the one that matters when everything else has
failed. §7's P2 line is *"`ms` — the argv refusal and the 0600 `--out`"*; it does not mention
`--in` for `ms` at all, and its gate (*"round-trip vectors; `ms encode | me sysw pack`
runs"*) exercises neither `combine` nor `repair`.

**What would close it.** §6.3 must enumerate the channels per tool rather than give only a
shape, and P2 must be ordered `--in`/`-` **first**, refusal second, with a gate that runs
`ms combine` and `ms repair` through the private channel.

## I-4 — §6.5: the exit-code space is a **five**-CLI system with recorded decisions, and §6.5 disposes of it in one sentence while supplying no table

**The defect.** §6.5 says *"One table, all four. `mk`'s 2-for-invalid-input becomes 1. Codes
to be fixed in the plan."* There is no table, the scope is wrong, and there is already a
cross-CLI parity ruling that §6.5 would break.

**Evidence — the existing ruling.** `md repair --help`, verbatim:

> Exit codes (**D26 cross-CLI parity** with `ms repair` / `mk repair` / **`mnemonic repair`**):
> 0 — every input was already valid (no corrections applied) 5 — at least one chunk had
> corrections applied (REPAIR_APPLIED) 2 — atomic-fail per plan §1 D28: ANY chunk failing BCH
> capacity fails the whole call …

`mnemonic` is a fifth constellation CLI (`/scratch/code/shibboleth/mnemonic-toolkit`, binary
present at `target/debug/mnemonic`). `ms repair --help` further records a deliberate
*divergence* from that parity — exit **4** rather than 5, *"Cycle F demotion — a corrected
ms1 is an UNVERIFIED candidate that cannot self-verify … D26"*. The non-uniformity is
reasoned and load-bearing.

**Evidence — measured divergence, wider than §2's single row.**

```
                        clap usage error   invalid artifact   repair-applied   repair-uncorrectable
md                            2                   1                  5                  2
mk                           64                   2                  5                  —
ms                           64                   1                  4                  2
mt                            2                   1                 n/a                n/a
me                            2            (pack: 4 = unplaceable; 2 = terminal refusal)
```

Confirmed by running each. `mk repair` on a 1-char-damaged chunk → exit 5. `md encode` with
no template → exit 2, colliding with clap's own 2. `ms repair --ms1` collides numerically with
`me sysw pack`'s exit 4, and both are visible in the same `$?` in a pipeline.

**What would close it.** The table has to exist in the spec, cover the five CLIs that D26
already binds, and state explicitly what happens to codes 4 and 5 — otherwise two
implementers build two different tables, and one of them silently changes what `mnemonic
repair`'s callers read.

## I-5 — §5 D5 / §7: the cited precedent for the shared crate is a git-rev pin to a crate that deliberately publishes nothing, and no phase names the distribution mechanism

**The defect.** D5's rationale is *"Precedent for a shared crate exists: `mt-codec` is
already consumed across repos."* True, but the mechanism is the opposite of what D5 needs,
and §7 never says how the new crate reaches four repos.

**Evidence.** `mnemonic-engrave/crates/me-cli/Cargo.toml:39-58`:

```toml
# A GIT dependency pinned to a rev, not a path and not a publish.
# A git dep works TODAY because the repo is public, needs no deploy key, and --
# the point -- keeps `cargo publish mt-codec` DEFERRED. Publishing is
# irreversible; pinning a rev is not.
mt-codec = { git = "https://github.com/bg002h/mnemonic-transaction", rev = "72b79b87…" }
```

A *different* mechanism does exist and is the real precedent: crates.io version deps —
`me-cli` takes `md-codec = "0.42"`, `mk-codec = "0.4"`, `ms-codec = "0.7"`; `mk-cli` takes
`md-codec = "0.42.0"`. Either way, D5's crate becomes a cross-repo dependency that must be
*released or re-pinned* before any of P1/P2/P3 can consume a change to it — and §7's P0 gate
is only *"its own tests + the R0 spec review"*, with no publish or pin step.

Two further facts the phasing has to absorb:

- **The code being "ported FROM `me`" is not in a library.** `write_private` is at
  `mnemonic-engrave/crates/me-cli/src/main.rs:844`, inside a binary crate; it is not exported
  by `me`'s own `lib.rs`. P0 says *"Ported FROM `mt`/`me`, which already have the tested
  versions"* — the versions exist and are tested, but through the binary's integration tests,
  not through an API. Extraction is fresh work with no existing consumers to hold it steady.
- **Cadences and versions are already independent**: `md-cli` 0.13.0, `mk-cli` 0.13.0,
  `ms-cli` 0.16.0, `mt-cli` 0.1.0, `mnemonic-engrave` 0.7.0.

**What would close it.** §7 needs a P0 sub-step that names the mechanism (publish vs. git-rev
pin) and says what a repo that has not upgraded does — in particular whether a mixed
constellation, where `ms` has the argv guard and `md` does not, is an acceptable intermediate
state or a thing the phasing must avoid. On the evidence, mixed states are unavoidable, so
the spec should say they are fine rather than leave it to be discovered.

## I-6 — §4 vs. D3: the principle is stated absolutely, D3 scopes it, and `mt`'s shipped text contradicts the absolute form

**The defect.** §4's rule is *"**Material never arrives on argv.**"* — no qualifier. D3 says
*"Refuse **secret/bearer** material on argv."* §2's table row heading is bare "material on
argv". An implementer reading §4 removes `md`'s and `mk`'s positionals; one reading D3 leaves
them.

**Evidence.** `mt`'s own refusal explicitly rules the other way, and is shipped:

```
mt encode: REFUSED — §8.2f, a transaction was passed as a command-line argument (678 characters)
  … (md and mk DO take their strings as arguments; md1/mk1 are watch-only, so a leak there
  costs privacy rather than the money.)
```

`md`'s and `mk`'s stderr agree — `note: stdout is watch-only — public keys only, cannot
spend`. If §4's absolute form is implemented, every documented `md`/`mk` example
(`md encode wpkh(@0/<0;1>/*)`, `md verify <STRINGS>… --template …`, `mk decode <MK1>…`) stops
working, and the spec never lists that as a breaking change.

**What would close it.** §4's principle should carry D3's qualifier, or §6 should state the
carve-out explicitly with the watch-only reasoning `mt` already wrote down.

## I-7 — §6.1: "every tool, every verb: stdout carries the canonical artifact and nothing else" is undefined for `decode` and meaningless for `verify`/`inspect` — and silently rewrites two tools' `decode`

**The defect.** §3 and §4 reason entirely about `encode`. §6.1 then applies the rule to every
verb without saying what the "canonical artifact" of a non-encode verb is.

**Evidence — measured stdout, all four, per verb:**

```
$ md decode md1yq-pqqxq-q8xtw-hw4xw-n4qh      →  wpkh(@0/<0;1>/*)                     (bare)
$ mt decode < strings                          →  broadcastable hex                    (bare)
$ mk decode <two mk1 strings>                  →  xpub:                xpub6Den8…      (labelled table)
                                                  origin_fingerprint:  aabbccdd
                                                  origin_path:         48'/0'/0'/2'
                                                  policy_id_stubs:     11223344
                                                  chunks:              2 (long)
$ ms decode ms10e…                             →  entropy: 0000…                       (3 labelled lines)
                                                  phrase: abandon … about
                                                  language: english (12 words, default — verify …)
$ md verify md1… --template 'wpkh(@0/<0;1>/*)' →  OK
$ ms inspect ms10e…                            →  OK: would decode v0.1 / hrp: ms / threshold: 0 / …
```

So §6.1 as written puts `mk decode` and `ms decode` stdout in scope for a rewrite — a
breaking change to two tools' machine-readable surface — and no phase in §7 mentions it. It
is also simply inapplicable to `verify` and `inspect`, whose entire output *is* commentary.

**What would close it.** §6.1 should scope the "artifact and nothing else" rule to the verbs
where an artifact exists (`encode`, and `decode` if the spec rules what `decode`'s artifact
is), and say plainly that `verify`/`inspect` are report verbs exempt from it. If `mk decode`
and `ms decode` are meant to change, that belongs in a phase with a gate.

## I-8 — §6.1: `--json` is asserted "already uniform" and it is not — and that assertion is the stated reason not to touch it

**The defect.** §6.1: *"`--json` unchanged; it is already uniform and already unbroken."*
"Unbroken" is true. "Uniform" is false, and the spec's goal is that a user *not care which
tool they are holding*.

**Evidence** — same flags (`--json --group-size 5 --separator hyphen`) to all three:

```
ms: {"schema_version":"1","ms1":"ms10entrsqq…","language":"english","word_count":12,"entropy_hex":"00…"}
mk: {"chunk_count":2,"code_variant":"long","mk1_strings":["mk1qpswajpqqsq…","mk1qpswajppsnz…"],…}
md: {
      "network": "mainnet",
      "phrase": "md1yqpqqxqq8xtwhw4xwn4qh",
      "schema": "md-cli/1"
    }
```

Three different key names for the artifact (`ms1`, `mk1_strings`, `phrase`), three different
schema keys (`schema_version`, absent, `schema`), and `md` alone pretty-prints. Grouping is
correctly ignored in all three, which is the part §6.1 got right.

**What would close it.** Either retract the "uniform" claim and say `--json` is
explicitly out of scope this cycle (defensible), or bring it in scope. What must not survive
is a false premise being the reason for an exclusion.

## I-9 — §1's `mt` row states exit **3**; `mt` has only 0 and 1

**The defect.** §1's motivating table — the one the whole spec is built on — reports `mt`'s
argv refusal as *"refused, **exit 3**, with purge advice"*. Measured, it exits 1.

**Evidence.**

```
$ mt encode --qr "$(cat tx.hex)" ; echo $?
mt encode: REFUSED — §8.2f, a transaction was passed as a command-line argument (678 characters)
…
1
```

Source confirms there is no third code — `mt-cli/src/main.rs` returns only
`ExitCode::SUCCESS` and `ExitCode::FAILURE` (lines 237, 253, 256), and `grep` for
`exit(3)`/`ExitCode::from(3)` across `mnemonic-transaction/crates` finds nothing.

This is a small fact in a load-bearing place: §6.5 defers the exit-code table, so "3" is
currently the only number the spec offers for an argv refusal, and it is wrong.

**What would close it.** Correct the cell to 1, and note that `mt` currently has no
distinguishable code for a refusal — which is an input to I-4's table, not a defect in `mt`.

## I-10 — §8 bullet 3: deferring the breaking-change enumeration to "the plan" is NOT safe — it already hides a phase-ordering blocker, and §7's phase list omits `mnemonic-engrave` itself

**The defect.** §8's third bullet defers *"which existing invocations break"* to the plan.
Doing the enumeration now surfaces a defect in §7's own ordering: **P2 breaks the scripts
P4's gate depends on**, and the repo those scripts live in is not in §7 at all.

**Evidence.** `mnemonic-engrave`'s own committed journey drivers shell out to `ms` with the
seed on argv — **18 call sites across 7 scripts** under `design/journeys/`:

```
$ grep -rn -- '--phrase\|--hex\|--ms1' design/journeys/*.sh | wc -l
18
$ grep -rln -- '--phrase\|--hex\|--ms1' design/journeys/*.sh
design/journeys/transcript.sh
design/journeys/transcript_hashvault.sh
design/journeys/transcript_pathological.sh
design/journeys/transcript_tr_pathological.sh
design/journeys/derive-rcw-keys.sh
design/journeys/derive-pathological-keys.sh
design/journeys/derive-hashvault-keys.sh

$ sed -n '137p' design/journeys/transcript.sh
  "$MS" encode --phrase "$(cat "$W/inputs/seeds/cosigner-00.seed")"
```

§7's P4 gate is *"a captured journey that regenerates."* D3 lands in **P2**, two phases
earlier, and breaks every one of those drivers. Either P2 must carry the script migration or
P4's gate is unsatisfiable when it is reached.

The `chunk-set-id:` removal (P3) stales committed goldens in the same directory —
**12 files** carry the line, transcripts and generated HTML both:

```
$ grep -rc "chunk-set-id:" design/journeys/* | grep -v ':0'   # 12 files
design/journeys/transcript_rcw.txt:4          design/journeys/out/rcw/rcw-journey.html:4
design/journeys/transcript_hashvault.txt:3    design/journeys/out/hashvault/hashvault-journey.html:3
design/journeys/transcript_pathological.txt:2 design/journeys/out/pathological/journey_pathological.html:2
… (and 6 more)
```

**What would close it.** §7 needs `mnemonic-engrave` listed as an affected repo with its
journey drivers migrated to the private channel *in P2*, and the golden regeneration owned by
P3. The general point stands independently of the specific items: the enumeration is what
found the ordering defect, so it belongs in the spec's scope decision, not after it.

## I-11 — a sixth repo, `mnemonic-gui`, hand-mirrors the flag surface these changes alter, and its drift gate is explicitly scoped to exclude `md`/`mk`/`ms` — so A and B ship green

**The defect.** All four CLIs carry a `gui-schema` subcommand whose stated purpose is to
publish the flag surface for `mnemonic-gui`. §6 changes that surface. The spec never names
`gui-schema` or `mnemonic-gui`.

**Evidence.** `mnemonic-gui` hard-codes the values §6 removes:

```
$ grep -rn "const SEPARATORS" /scratch/code/shibboleth/mnemonic-gui/src/schema/
src/schema/md.rs:24:const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];
src/schema/mk.rs:15:const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];

$ grep -rn 'default_value: Some("5")' /scratch/code/shibboleth/mnemonic-gui/src/schema/
src/schema/mk.rs:71  src/schema/ms.rs:78  src/schema/ms.rs:414   (+ 4 in schema/mnemonic.rs)
```

And the gate that would catch the drift disqualifies itself, at
`mnemonic-gui/tests/schema_mirror_defaults_drift.rs:29-31`:

> `mnemonic` only (the CLI where the eval finding + F6 live). Extending to `md`/`ms`/`mk` is a
> natural follow-on (their pinned binaries + any allowlist entries) — **deliberately out of
> this cycle to stay a bounded add.**

So flipping the `--group-size` default and removing `hyphen`/`comma` produces **zero test
failures in the GUI**, and the GUI keeps offering a dropdown value the CLI has deleted.

**What would close it.** The spec should either take `mnemonic-gui` into scope (regenerate
its mirror in the same phase as the CLI change) or state explicitly that the GUI is expected
to drift and file the re-sync with an owning phase. Silently is the one option that is not
available, because the drift gate is documented as absent.

---

# MINOR

## M-1 — §D4: "the grouped form moves to the stderr card", but three of four tools have no card

`mk encode`'s entire stderr is one line (`note: stdout is watch-only — public keys only,
cannot spend`). `md encode`'s is notes and warnings, not a card. Only `ms` has one (with
`--no-engraving-card` to suppress it) and `mt` has a full report plus a `SUGGESTED LEGEND`.
D4 therefore requires *inventing* an engraving card for `md` and `mk`, which no phase scopes
and whose contents two implementers would render differently. §7 P3 compresses this to three
words: *"grouping to stderr"*.

Related, and worth one sentence in the spec: under D4, `ms encode --no-engraving-card` and
any pipeline using `2>/dev/null` yield **no grouped form anywhere** — which is correct but is
a behaviour operators should be told about, since today they get grouping on stdout
unconditionally.

## M-2 — §6.2 understates the breakage and omits `comma`

§6.2 says *"`ms` currently tolerates hyphens on decode, so its hyphen option is safe *for
`ms`*"*. Measured, **`md` and `mk` tolerate hyphens too**:

```
$ md decode md1yq-pqqxq-q8xtw-hw4xw-n4qh        → wpkh(@0/<0;1>/*)   exit 0
$ mk decode "mk1qp-swajp-…" "mk1qp-swajp-…"      → xpub: xpub6Den8…   exit 0
```

Only `mt` refuses. So the removal costs an option that round-trips in three of four tools,
not one — and all three also offer `comma`, which §6.2 never mentions and which the change
presumably also removes. The cross-tool argument still stands; the spec should just state
the cost accurately, since "one cosmetic option in three tools" is currently counting the
wrong thing.

## M-3 — §6.2 interacts with an OPEN follow-up (F-245) the spec does not mention

`design/FOLLOWUPS.md:10299` — F-245, open: *"`me sysw pack` packs a record's trailing
whitespace VERBATIM into the public section"*. Reproduced:

```
$ printf 'md1yqpqqxqq8xtwhw4xwn4qh \n' > ws.txt ; me sysw pack --in ws.txt --out ws.bin ; echo $?
0        # trailing space accepted, no warning
```

Making the separator whitespace-only makes whitespace the only thing that can appear in a
grouped string, so the two decisions meet. Not a blocker; it belongs in the plan's
reconciliation sweep.

## M-4 — `--allow-argv-secret` names bearer material "secret"

`mt`'s material is bearer, not secret, and the spec is careful about that distinction
everywhere else (§1's table, §6.4's "bearer or secret"). The flag an operator types on `mt`
would be `--allow-argv-secret`. Cosmetic, but this is a flag that will be grepped for in
review — §6.3's own justification for the override is *"It is greppable in a script"* — so
the name should say what it admits.

## M-5 — §6.1's "when neither is given" has three possible antecedents

The bullet list is `--in`, `-`, `--out`, then *"stdout, when neither is given"*. "Neither" is
a two-item word after a three-item list. Presumably it means "when `--out` is not given",
but an implementer could read it as "when neither `--in` nor `--out` is given" and make
stdout behaviour depend on the *input* channel.

## M-6 — §8 bullet 2 is answerable in one command and should not be an open item

*"Whether `mk encode --from-md1` can accept a multi-chunk set at all."* It can — the flag is
documented `Repeatable`, and repeating it works on a real 4-chunk set:

```
$ md encode 'wsh(multi(2,@0/…,@1/…))' --key @0=… --key @1=… --fingerprint … --group-size 0   # 4 md1 lines
$ mk encode --xpub … --origin-path "m/48'/0'/0'/2'" --from-md1 <c0> --from-md1 <c1> --from-md1 <c2> --from-md1 <c3>
exit 0
note: policy f15a7d59 has 2 cosigner(s); 1 of them carded here, 1 not carded
```

The failure recorded in §8 was a space-joined single value, which is a usage error, not a
capability gap. Strike the bullet; there is nothing here for this cycle's scope.

---

# NIT

## N-1 — the `mt` path in the brief is a worktree, and a second checkout exists

`/scratch/code/shibboleth/_work/p3b/mnemonic-transaction` is a git worktree; a separate
checkout sits at `/scratch/code/shibboleth/mnemonic-transaction`. Any `path =` scheme for
D5's crate would see two different depths for the same repo. Worth one line in the plan so
nobody writes `../../mnemonic-secret`.

## N-2 — cited follow-ups are shipped but not marked CLOSED

F-246, F-250, F-251, F-252 and F-253 carry no `CLOSED` marker in their `design/FOLLOWUPS.md`
headers, yet their behaviour is present in the binaries (verified: `mt encode -` works via a
pipe; the F-252 "only the file's OWN mode was checked" text is in the refusal; `me`'s
terminal refusal fires at exit 2). The spec cites them as settled, which is correct about the
code and stale about the record. Not a spec defect — flagged so the plan does not re-open
closed work.

---

# What I verified and found CORRECT (so a later round does not re-derive it)

- §1's `ms` claims: `ms encode --phrase "<12 words>"` → exit 0, no argv warning; stderr
  advises `> file.txt`. Both reproduced.
- §1's `mt` write-gate claim: refuses mode 0644 and names exactly three remedies. Reproduced.
- §2: all four contribute **0 bytes** to stdout on every failure path I could construct.
- §2: `--group-size` defaults — md 5, mk 5, ms 5, mt off. Reproduced from `--help` **and**
  from output.
- §2: `mk` returns exit 2 for an invalid artifact where md/ms/mt return 1. Reproduced.
- §3: `ms encode` default → `me sysw pack` exit 4; `--group-size 0` → exit 0. Reproduced;
  the same holds for `md`.
- §6.4 bullet 3: `--allow-world-readable` does **not** override `me`'s terminal gate —
  `script -qec "me sysw pack --allow-world-readable --in records.txt"` → exit 2, terminal
  refusal. Correct as specified.
- §6.4 bullet 4b: an input refusal outranks a destination refusal in `mt` — with a bad tx
  *and* a 0644 stdout, the first `REFUSED` line is `§8.2e, the reassembled bytes are not a
  Bitcoin transaction`. Correct as specified.
- §6.1: `--json` is genuinely unaffected by `--group-size`/`--separator` in all three.
- §7 P1's count: `grep -rc "#\[test\]"` over `mnemonic-transaction/crates` = **236**. Correct.
- §8a: `./scripts/spec-structure-check.sh design/SPEC_constellation_cli_uniformity.md`
  reports **7 STRUCTURAL DEFECT(S)** — 5 × duplicate-section-6 (subsection parsing), 2 ×
  table-row-cell-count on the P2/P3 rows (unescaped-pipe handling). Both classes are exactly
  as §8a characterises them. §8a is accurate.
- F-244, F-245, F-246, F-247, F-250, F-252, F-253 all exist in `design/FOLLOWUPS.md` at the
  meanings the spec ascribes to them (with the F-246 scope caveat noted in I-2).
- D5's `mt-codec` precedent claim is literally true (`me-cli` consumes it cross-repo) — the
  problem in I-5 is what kind of precedent it is, not whether it exists.

---

## Also verified, and it is a fact the plan needs

`ms`'s own suite is the largest single migration under D3: **31 of 76 test files** under
`mnemonic-secret/crates/*/tests/` reference `--phrase` or `--hex`
(`grep -rl -- '--phrase\|--hex' crates/*/tests/*.rs | wc -l` = 31; `ls crates/*/tests/*.rs |
wc -l` = 76), out of 276 `#[test]` functions. No `--allow-argv-secret` exists anywhere in the
constellation yet. By contrast `mt` needs **zero** test changes for D3/D4/§6.2 — it already
implements all three — which is what makes P1's "least risk" framing right in spirit even
though its gate is wrong (I-2).

---

# Counts

**4 Critical / 11 Important / 6 Minor / 2 Nit**

# Verdict

**NOT GREEN — do not proceed to implementation.** The spec's diagnosis in §1–§4 is sound and
its measurements there hold up; the failures are in §6 and §10, where rules were lifted from
`me` and `mt` without the reasons that made them right, and where "done" is defined by a
pipeline that has never been run. The single most important finding is **C-1**: D1's
composition — a brace group of independent producers into one `me sysw pack` — converts any
producer's refusal into exit 0 and a payload with a record silently missing, and `me` already
built the defence for the total-failure case and documented it, so the spec is stepping
around a guard rather than lacking one.
