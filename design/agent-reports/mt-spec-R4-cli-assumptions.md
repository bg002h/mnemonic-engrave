# R4 — pre-implementation readiness: CLI, I/O, refusal surface, `sysw` transport

Artifact: `design/SPEC_mt_v0_1.md` at **4527cbc** (started at `d2d1a58`; the
mid-review move touched only §8.4's reference-pair paragraph, verified by
`git diff d2d1a58 4527cbc`).
Scope: CLI, I/O, refusal/warning behaviour, `sysw` transport. Codec/format layer
belongs to another reviewer and is not touched here.
Lens: **not "is this wrong" — "where must an implementer guess, and will the next
implementer guess differently".**

Everything numeric below was computed or executed, never estimated. The
executed checks are named inline with their commands.

---

## Verdict

| severity | count |
| --- | --- |
| **Critical** — unbuildable, or the artifact differs between implementations | **6** |
| **Important** | **9** |
| **Minor** | **6** |
| **Nit** | **2** |
| total | **23** |

The headline: **the spec names exactly zero CLI flags.** `grep -n -- "--[a-z]"`
over the whole 1,712-line spec returns **one** hit, and it is the `--timelocked`
/ `--immediate` pair being *deleted* (§1.7, line 89). Meanwhile the design
requires **seven distinct operator inputs** that cannot come from the PSBT.
That single omission generates B-1 and feeds five other findings.

Second headline: **§10.9's `sysw` ruling has four plausible framings, they
produce four different §8.7c refusal ceilings, and the only one that satisfies
the constellation's own record rules refuses the spec's own largest measured
artifact by 322 bytes.** This confirms R3 lens 3's C-2 and sharpens it with
arithmetic and an executed classifier run — but two of R3's four cited gates are
**mis-attributed** (they are `seal` gates, not `sysw` gates), and two real
`sysw` gates R3 did not name do bind. See `## The sysw gates`.

---

### B-1 — The spec names no CLI flags, and seven operator inputs have nowhere to arrive

**Severity: Critical.** §10.10 (CLI surface), §5, §8.2c, §8.7, §8.8, §6a, §10.4.

**What the spec says.** §10.10's table has four rows: verbs, input, outputs,
stderr, and `flags | none for locktime`. That row is a statement about *locktime*
flags only. No other flag is named anywhere. `grep -n -- "--[a-z]"` on the spec
yields one hit, the deleted locktime pair.

**What an implementer must guess.** The entire argument surface. The design
requires all seven of these, and none of them is derivable from the PSBT:

| input | required by | why the PSBT cannot supply it |
| --- | --- | --- |
| `FROM WALLET <8 hex>` | §5, §10.4 | §6: a transaction cannot say what it spends *from* |
| `TO <wallet id / fp>` | §5, §10.4 | names a counterparty, not a script |
| `TO <free text>` **behind a flag** | §5, §10.4 (flag name explicitly deferred) | unverifiable by construction |
| module size | §4, §8.8, §10.1 (*"the operator picks"*) | a physical choice |
| **plate budget** | §8.7 (*"the operator's **stated** maximum plate count"*) | §8.7's refusal is **unrunnable without it** |
| per-input value or input total | §8.2c (*"`mt` requires the operator to supply"*) | that is the definition of the case |
| node location | §6a | not in the transaction |

**Candidate guesses.** (A) Flags for all seven, invented names
(`--from`, `--to`, `--to-label`, `--module`, `--max-plates`, `--input-value`,
`--rpc-url`). (B) A config file or env vars, on the grounds that seven flags is
a lot. (C) Interactive prompts — which §8.8's *"`mt` says so **at the point of
choice**"* actually reads like (see B-18).

**What the spec's logic implies.** Flags, and the sibling CLI is the model:
`me` is `clap`-derive with `--in`, `--out`, `--iterations`,
`--no-passphrase` (`crates/me-cli/src/main.rs:13-208`). Env vars are ruled out
for anything sensitive by `me`'s own standing reasoning
(`main.rs:170-173`: *"Never argv and never an environment variable"*). So (A) —
but the **names** are pure invention, and two of them are funds-relevant.

**Observable divergence.** Implementer A ships `--max-plates` **required**;
implementer B defaults it to unbounded, so §8.7 — a numbered refusal — **never
fires** and a 6-plate job starts silently. Same PSBT: A refuses, B cuts steel
for two hours. Every runbook, every CI fixture and every operator muscle-memory
is incompatible between the two.

---

### B-2 — What a `sysw` record actually contains: four framings, four different §8.7c ceilings, and the conformant one does not fit

**Severity: Critical.** §10.9, §3, §8.7c.

**What the spec says.** §10.9: the payload goes via `sysw`, unencrypted, and
*"There is no transaction class… Adding one is the work."* §3's table says
bech32 uppercase is *"usable in a `sysw` record? **yes — chosen**"* and that
*"the `sysw` record stores **lowercase**, and `mt` uppercases only when encoding
the QR symbol."* §8.7c computes the largest artifact at **6,932 characters**
against `MAX_SECTION_LEN = 8191`, *"15.4% headroom"*, ceiling *"roughly
4,537 B"*.

**What an implementer must guess.** The **record boundary and the record's own
framing** — which the 8,191 arithmetic silently assumes. §8.7c's 6,932 is bare
payload characters: it counts no HRP, no BCH checksum, and no LF separators.

**Candidate guesses**, with the arithmetic run
(40 payload B/chunk per §3b, 41-bit `mt1` header per §3, 3-char HRP,
13-symbol BCH per `REGULAR_CHECKSUM_SYMBOLS`, LF joins per `sysw::split`):

| framing | bytes for the 3,809 B artifact | fits 8,191? | largest PSBT that fits |
| --- | --- | --- | --- |
| **A** one record per chunk, **bare** bech32 data | 6,977 | ✓ (+1,214) | **4,476 B** |
| **B** one record per chunk, full `mt1` string (HRP + BCH) | **8,513** | **✗ (−322)** | **3,671 B** |
| **C** one record, `tx:<hex of the PSBT>` (mirrors `text:`/`pass:`) | 7,621 | ✓ (+570) | **4,094 B** |
| **D** one record, one `mt1` string over the whole stream | 6,898 | ✓ (+1,293) | **4,525 B** |

§8.7c's stated ceiling is 4,537 B. **No framing produces it.** Four framings,
four ceilings, and the refusal boundary of a numbered refusal moves by up to
**854 bytes** depending on a choice the spec never makes.

**Executed, not inferred** — all three candidate record shapes are refused by the
*shipped* classifier today:

```
$ ./target/debug/me sysw pack "qpzry9x8gf2tvdw0s3jn54khce6mua7l" --no-passphrase --out /tmp/t.bin
me: record 0 … is not a form this container can place: not a BIP-39 mnemonic,
    not an md1/mk1/ms1 string, and not a `text:`/`pass:` record.
$ ./target/debug/me sysw pack "mt1qpzry9x8gf2tvdw0s3jn54khce6mua7lqqqqqq" …   # same refusal
$ ./target/debug/me sysw pack "tx:0200000001" …                               # same refusal
```

**What the spec's logic implies — and it is a catch-22.** §3's *"the `sysw`
record stores lowercase"* and the constellation's `(HRP, chunk_set_id)`
convention both point at **framing B**, one `mt1` string per chunk. But §3a's
*"never stack them"* rule forbids a BCH layer inside a QR-bound artifact, so the
chunk carried for `mt qr` has **no BCH checksum** — which makes it framing A,
which has no `1` separator and therefore cannot classify at all
(`crates/me-cli/src/classify.rs:40-52` needs an HRP before the first `1`;
the bech32 charset excludes `1`). And framing B, the only shape the record
rules admit, **refuses the spec's own largest measured artifact**.

**Observable divergence.** Implementer A ships framing A and must patch
`classify` to recognise a separator-less record — every other constellation tool
now sees a record it cannot place. Implementer C ships `tx:<hex>` and the QR
chunking has to happen on the *device*, so §4's host-side search never reaches
the machine (B-3). Implementer B ships the conformant shape and RCW `wsh` tier 1
at 5 inputs — a real wallet, the spec's own fixture — cannot be sent at all,
with §8.7c's message telling the operator it has 15% headroom.

---

### B-3 — §4's chosen configuration and §5's legend have no channel into the payload

**Severity: Critical.** §2, §4, §5, §10.8, §10.9.

**What the spec says.** §2 lists as a thing this codec exists to specify:
*"which (module size, QR version, ECC level, tiling) configuration is chosen for
a given transaction, **deterministically and with every tie broken**, so two
encoders agree"*. §5 specifies five legend fields and §10.8 adds a per-symbol
`n/m`. §10.9 rules that the payload travels as `sysw`. **No section says how any
of that crosses the wire.**

**What an implementer must guess.** Whether §4's answer and §5's legend text are
(i) carried *in* the payload, and in what field, or (ii) re-derived on the
device from the transaction bytes.

**Candidate guesses.** (A) A second `sysw` record holding a config/legend blob —
requires a *second* new class or a `text:` record, and the legend text contains
**spaces** (`BEARER - ANYONE HOLDING THIS CAN SPEND IT`), so it must be
hex-escaped exactly as `FreeText` is. (B) Device re-derivation: the firmware runs
§4's search itself. (C) Config in the payload, legend re-derived — a split.

**What the spec's logic implies.** (A). §2's own justification for specifying the
search is *"so two encoders agree"*, and §10.17 states the firmware **cannot**
run that search today (*"the fork's only arbitrary-payload QR path is fixed at
`freeTextQRScale = 2` with a compile-time ECC level and one code per plate"*).
If the device re-derived (B), §4's search would be host-side computation that
never reaches the machine and §2's fifth bullet would buy nothing. So the config
must travel — and nothing says in what.

**Observable divergence.** A sends a two-record payload (transaction + config);
B sends one and the device engraves at scale 2 / compile-time ECC, ignoring the
0.60 mm module and the ECC level §4 spent every leftover byte buying. Same PSBT,
physically different plates, and B's plate silently discards the damage
tolerance §1.5 exists to maximise. Neither implementation can read the other's
payload.

---

### B-4 — What `mt qr` writes, and where

**Severity: Critical.** §10.10, §3b.

**What the spec says.** §10.10: *"`mt qr` output | a **SH2 payload** (`sysw`)
carrying the QR"*, and *"stdout carries the artifact, stderr carries everything
the human must see"* (§3b, called *"the first fixed point of §10.10's CLI
contract"*).

**What an implementer must guess.** Three separate things: (i) the
**encapsulation** — bare container, `REGION_LEN`-padded region image, or UF2;
(ii) the **destination** — raw binary on stdout, or a required `--out`; (iii)
**how it reaches the machine**, which no section states.

**Candidate guesses.** The two siblings in this repo disagree, so both are
defensible precedent:

| sibling | encapsulation | destination |
| --- | --- | --- |
| `me sysw pack` | bare container; `--region` pads to 65,536 with `0xFF` | **stdout by default**, `--out` optional (`main.rs:165-168`) |
| `me seal` | **UF2** (`seal::uf2::to_uf2`) | `--out` **required**, *"never stdout, because the passphrase shares that stream"* (`main.rs:104-106`) |

And `me`'s own converter refuses to guess at all:
`me: choose an output mode: --out <file>, --stdout, --hex, or --base64`
(`main.rs:336`).

**What the spec's logic implies.** `me sysw pack`'s shape, since §10.9 names
`sysw` explicitly: a bare container, stdout by default. But `mt qr`'s output is
**binary**, and §10.10's *"stdout carries the artifact"* was written for
`mt string`'s text. Piping ~7 KB of binary at an interactive terminal is what
`me`'s explicit-output-mode rule exists to prevent.

**Observable divergence.** A writes the bare container to stdout; B writes a
65,536-byte region image; C writes UF2. **None of the three files is loadable by
the other two's documented procedure**, and only the region image is directly
usable with `~/bin/sh/sh2-flash`. An operator following A's runbook against B's
binary flashes a 65 KB image where a 7 KB container was expected, or writes a
bare container to `0x10D00000` and leaves the rest of the sector holding whatever
was there before.

---

### B-5 — How the node is located, and whether a timeout is "no node" or an error

**Severity: Critical.** §6a, §8.5, §10.5.

**What the spec says.** §6a: *"The call is **`gettxout <txid> <vout> false`**,
verified against a live Core v25.0.0 node"*, `include_mempool` false. §8.5:
`null` → **refuse**. §6a's new block: no node → a **warning** naming the skipped
checks. §10.5: *"`mt` asks the node it is given and reports what it is told."*
`grep -ci "cookie\|bitcoin.conf\|timeout"` over the spec: **0, 0, 0**.

**What an implementer must guess.** Everything about the connection: URL, port,
network selection, credentials (cookie file vs `rpcuser`/`rpcpassword`), and the
timeout — plus the classification of every non-answer.

**Candidate guesses.** (A) `--rpc-url` + `--rpc-cookie` flags, no default →
"no node" unless asked. (B) Auto-discovery: `~/.bitcoin/.cookie` +
`127.0.0.1:8332`, so it *usually* finds a node. (C) `bitcoin.conf` parsing.

**What the spec's logic implies.** §10.5's *"the node it is given"* is the
closest thing to a ruling and points at **(A), explicit**. §0's offline posture
agrees: a tool that silently reaches for localhost is doing network I/O the
operator did not ask for.

**Observable divergence, and it is funds-relevant.** §8.5 is a **refusal** —
"one of your inputs is already spent, do not cut this plate." Under (A) it never
fires unless the operator passes a flag; under (B) it fires on any box running
Core. Same PSBT, same machine: A engraves 21 minutes of steel for a transaction
that can never confirm, B refuses. The operator cannot tell which tool they have
from the artifact.

**The unresolved sub-cases, each a separate guess:**

| situation | is it "no node"? |
| --- | --- |
| connection refused | yes, by §6a's plain reading |
| **timeout** (no reply) | undecided — §6a says an absent node is *"an absent answer, not a bad one"*, which implies warning-and-proceed |
| **401 / bad credentials** | undecided — the node exists and told you nothing |
| **partial failure**: inputs 0–2 answered, input 3 times out | **undecided, and the worst case.** A downgrades the whole run to the no-node warning; B reports per-input `UNKNOWN`; C errors. If input 3 is the spent one, A and B print different things and only C is loud |
| `-28` loading block index | undecided |

§6a's *"Not a refusal"* clause resolves the first row and gestures at the second.
It resolves none of the other three.

---

### B-6 — The engraved `~<year>` depends on whether a node was reachable

**Severity: Critical.** §8.4 (as amended at `4527cbc`), §5.

**Not the ruled question.** The operator has ruled that the reference pair is a
**source constant**, not a build-machine value; that is settled and is not
reported. What follows is downstream of the ruling and is untouched by it.

**What the spec says.** The estimate is
`reference_time + (target_height − reference_height) × 600 s`, and:
*"a node is reachable at **run** time | the **live** height or MTP — always
preferred, always fresher"*; *"no node at run time | the **embedded** pair"*.
The result is engraved: §5's legend carries `LOCKED TO BLOCK <n> ~<year>`, and
§8.4 says *"it is engraved, and engraved numbers are forever."*

**What an implementer must guess.** Two things. First, when a node **is** used,
what plays the part of `reference_time`? The spec supplies a `reference_time`
only for the embedded pair. Candidates: the **system clock**, the node's
**MTP**, or the node's best-block header time. Second, whether the two paths are
required to agree.

**What the spec's logic implies.** §8.4 already rules *"compare like with like…
a timestamp against the chain's **median-time-past** — the monotonic,
consensus-enforced figure rather than the loosely-constrained header stamp."*
That reasoning applies with equal force here, so the live `reference_time`
should be **MTP**, not the system clock. The spec does not say it, and the
system clock is the obvious implementation.

**Observable divergence — the plate depends on the operator's network, not on
the transaction.** Two runs of `mt qr` on the same PSBT:

- run with a node: `reference = (live_height, live_MTP)`;
- run without: `reference = (963663, 2026-08-23)`.

Both extrapolate at exactly 600 s, so they agree **only while the chain has run
at exactly 600 s/block since the constant was pinned**. It has not and will not.
As soon as the accumulated drift crosses a year boundary in the projection, the
same transaction engraves `~2034` on one operator's plate and `~2035` on
another's — permanently, on steel, with no way to tell which is which. If
`reference_time` is the **system clock**, it is worse: the estimate varies with
the operator's clock, and a run at 23:59 on 31 December differs from a run four
minutes later.

**The cheapest fix is a decision, not code:** state whether the engraved year is
computed from the **embedded constant always** (deterministic, a property of the
release, and the live node only sharpens the *stderr* report) or from the live
chain when available (fresher, non-deterministic). §8.4's own argument for the
constant — *"a checked-in constant makes the estimate a property of the
**release**"* — implies the first, and the current table says the second.

---

### B-7 — "Every refusal names the number that caused it" has no format, and the numbers are neither unique nor ordered

**Severity: Important.** §8.

**What the spec says.** §8 closes: *"Every refusal names the number that caused
it. A refusal that says only 'too large' costs the operator a round trip."*
§10.10 concedes: *"**Still unspecified:** exit codes, and the format of the
refusal messages."*

**What an implementer must guess.** The rendering, *and which number to name*.

**The numbering itself is defective, measured.** §8's list markers, in document
order (`grep -n "^[0-9]\+[a-z]*\. \*\*"` over §8):

```
1, 2, 2b, 2c, 2d, 3, 4, 5, 6, 7, 7c, 7b, 8, 9
```

Two problems, both an implementer's to resolve:

1. **`7c` is printed before `7b`.** An operator reading the spec top-to-bottom
   finds the section ceiling before the chunk ceiling.
2. **Item 1 and item 3 are the same refusal.** Item 1: *"**Not fully finalized**
   → refuse."* Item 3: *"An unsigned or **unfinalized** transaction offered for
   engraving → refuse."* A finalized-PSBT check that fails has **two** numbers,
   and §8's promise cannot be satisfied deterministically.

**Candidate guesses** for the format: `mt: refused (§8.1): …` /
`mt: refusal 8.1: …` / `mt: E8.1: …` / prose naming the number mid-sentence.
For the duplicate: A prints `§8.1`, B prints `§8.3`, C prints both.

**What the spec's logic implies.** `me`'s house style is
`me: <message>` on stderr with the section in prose
(`main.rs:889-892`: *"a section caps at {} bytes"*). So `mt: …` with a
parenthesised `§8.N`. Nothing implies which of 8.1/8.3 wins — that needs a
ruling, and the cheapest is to **delete item 3** as a duplicate of item 1.

**Observable divergence.** An operator's runbook says *"if you see §8.3, your
wallet did not finalize"*. Against implementer A's build that string never
appears. Machine-parsing refusals (a natural CI use) breaks across
implementations.

---

### B-8 — Exit codes

**Severity: Important.** §10.10 (*"Still unspecified: exit codes"*).

**What an implementer must guess.** The whole code space and the mapping of 11
refusals + 4 warning classes onto it.

**Candidate guesses.** (A) `0` / `1`. (B) `me`'s four-code scheme. (C) A code
per refusal number.

**What the spec's logic implies — strongly, and this is nearly decided.**
`me`'s scheme is a sibling constant block (`crates/me-cli/src/main.rs:211-214`):

```rust
const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_REFUSED: i32 = 3;
const EXIT_INVALID: i32 = 4;
```

and it already distinguishes exactly the two things §8 needs: `EXIT_REFUSED`
for a policy refusal (`me: refusing to seal seed material…`, `main.rs:494`)
versus `EXIT_INVALID` for malformed input (`main.rs:313`). An implementer
should adopt it verbatim.

**What is still a genuine guess even after adopting it:** which of §8's items
are `REFUSED` (3) and which are `INVALID` (4). §8.1 "not finalized" is arguably
malformed input; §8.7b "over the 64-chunk container" is arguably a refusal;
§8.2b `SendingTooMuch` could be either. And nothing says whether a **warning**
(§8.2b low fee, §8.2c legacy, §6a no node) leaves exit `0` — it must, by §8.4's
*"warn, never refuse"*, but a CI author will want that in writing.

**Observable divergence.** `mt qr … || alert` in an operator's script fires on
A's build and not on B's for the same low-fee transaction.

---

### B-9 — Input encoding, and what distinguishes "file" from "stdin"

**Severity: Important.** §10.10.

**What the spec says.** *"**input** | **a finalized PSBT, and nothing else** —
from a file or stdin, **equivalently**"*. `grep -ci base64` on the spec: **1**,
and it is §3's efficiency table, not the input.

**What an implementer must guess.** (i) binary / base64 / hex, or a sniff;
(ii) positional path or a flag; (iii) what happens with neither, or both.

**Candidate guesses.**

| | encoding | path |
| --- | --- | --- |
| A | base64 only | `--in FILE`, stdin otherwise — `me`'s shape (`main.rs:16-18`) |
| B | sniff the 5-byte magic `psbt\xff`, else base64 | positional — `me sysw show <file>`'s shape (`main.rs:207`) |
| C | binary + base64 + hex, all sniffed | both accepted |

**What the spec's logic implies.** Sniffing is nearly free and unambiguous:
a binary PSBT starts `70 73 62 74 ff`, and its base64 always starts `cHNidP8`,
so the two cannot be confused. §0's whole flow is *"test it in your wallet
first"* — and wallets emit **both**: `bitcoin-cli finalizepsbt` returns base64
text, Sparrow and Electrum save binary `.psbt` files. Refusing either strands
half the intended users, so **(B) or (C)**. The path shape should follow `me`'s
`--in`, since `mt qr` also needs `--out` (B-4) and a bare positional next to
`--out` reads badly.

**A concrete trap the spec's silence hides.** `me`'s stdin read is
`read_to_string` (`main.rs`), which **fails on non-UTF-8**. An implementer who
copies that shape gets base64-only by accident, and the failure on a binary
PSBT is `cannot read stdin: stream did not contain valid UTF-8` — a message that
sends the operator looking at their terminal, not at their file format.

**Also unspecified and cheap:** trailing newline / surrounding whitespace on
base64 (every shell heredoc adds one), and whether `--in` plus piped stdin is a
usage error. `me seal` already rules the analogous case:
`me: pass records on argv OR via --in, not both` (`main.rs:429`).

**Observable divergence.** An operator with Sparrow's `.psbt` file gets a clean
run from B and `not valid UTF-8` from A.

---

### B-10 — §8.9 "Secrets → refuse" has no meaning on a PSBT input, and the real hazard is uncovered

**Severity: Important.** §8.9, §5, §10.4.

**What the spec says.** In full: *"**Secrets** → refuse, as `me` already does
for `ms1`."*

**What an implementer must guess.** What a "secret" *is* when the input is a
finalized PSBT. BIP-174 defines no private-key field; a finalized PSBT carries
`PSBT_IN_FINAL_SCRIPTSIG`/`_SCRIPTWITNESS`, xpubs and UTXO records — all public.
The refusal has no obvious subject.

**Candidate guesses.** (A) Dead code — nothing can trigger it, ship an
unreachable branch. (B) Scan the PSBT's proprietary/unknown key-value pairs for
things shaped like key material. (C) **Apply it to the operator-supplied
strings** — the `FROM`, `TO` and free-text label.

**What the spec's logic implies — (C), and it is the only reading that protects
anything.** §5's `TO <free text>` is **engraved**, on a **bearer** plate, and
§10.4 rules the label is *"an act of assertion by the operator"* typed on a
command line. An operator who pastes a BIP-39 mnemonic or an `ms1` into
`--to-label` engraves their seed onto a plate that already spends. `me` has
exactly this guard and calls it what it is — a best-effort anti-footgun, not a
boundary (`main.rs:483-495`):

```rust
let is_seed = |r: &String| matches!(classify(r), Ok(Format::Ms))
    || seal::passphrase::is_valid(r);
```

**Observable divergence.** A ships §8.9 as an unreachable branch and the label
path is unguarded; C reuses `me`'s `is_seed` on every operator string. Between
them sits a plate with a seed phrase cut into it. Note also that `is_valid` is a
BIP-39 *checksum* check, so it catches a pasted mnemonic and not a partial one —
worth saying in the spec rather than leaving to the implementer to discover.

---

### B-11 — The success report: no row format, no ordering, and one row needs an input that does not exist

**Severity: Important.** §10.10.

**What the spec says.** Seven rows on stderr *"before any plate is cut"*: every
output; the fee; the locktime; the plate count and engraving time; the
configuration; the headroom; the value provenance.

**Three separate guesses:**

**(a) Format.** Is it a stable `key: value` block (greppable), a table, or
prose? Nothing says. §8.4 *does* pin one row exactly — the locktime line, with
five literal example forms including column alignment
(`LOCKED TO BLOCK 1383520          current height 963663`). No other row gets
that treatment. Implementer A pins all seven the same way; B writes prose;
an operator's `grep -q "^fee:"` works against one.

**Units are half-decided.** §8.2c prints `0.99000000 BTC`, §8.2b prints
`3.2 sat/vB`. So amounts are BTC with 8 decimals and fee rate is sat/vB — the
spec's own examples imply it. But *"the fee | **absolute** and as sat/vB"*
leaves the absolute fee's unit open (BTC by the same logic; sats by every
mempool UI).

**(b) Ordering versus the warnings.** The report *"goes to stderr, **with the
warnings**"*. Both share one stream and nothing orders them. Candidates: report
then warnings; warnings emitted at check time then the report; warnings first as
a banner. Observable: §8.2c's eleven-line legacy block lands above or below a
forty-line report, i.e. on screen or scrolled off it. For a warning whose whole
purpose is to be read before 21 minutes of steel, that is not cosmetic.

**(c) The change-detection row is unsatisfiable.** *"every output | address in
full, amount, and **which are change if a wallet was supplied**"*. Identifying
change requires deriving the wallet's own scripts — i.e. a **descriptor**. The
only wallet input the spec defines is §5's `FROM WALLET <8 hex>`, which §5 itself
says is *"the top 4 bytes of a canonical md1 identity"* and *"a hint, never an
authority — nothing may branch on it."* You cannot derive a scriptPubKey from
4 bytes of a hash, and branching on it is forbidden by the same section.

Implementer A drops change detection (the row silently never fires).
Implementer B adds a `--descriptor` flag the spec never authorised, which pulls
descriptor parsing into a tool §0 says *"holds no private key… does not choose
which UTXOs to spend"*. Observable: B shows the operator that 0.4 BTC of a
0.5 BTC "payment" is change coming back; A shows two outputs and lets them
believe they are sending 0.5.

**(d) The provenance row's enumeration is incomplete.** *"per input:
chain-fetched (§6a), txid-bound (§8.2d), or operator-asserted (§8.2c)"*. A
**segwit input carrying `witness_utxo` and no node** is none of the three: its
value is bound by the signature (BIP-143/341, §8.2c's own table) but nothing
fetched or hashed it. That is the *common* offline case. A labels it
"operator-asserted", which is alarming and false — the operator asserted
nothing. B adds a fourth label. The row exists to tell an operator how much to
trust a number; the two builds tell them different things.

---

### B-12 — A lock below the reference height makes the subtraction negative

**Severity: Important.** §8.4.

**What the spec says.** `estimated unlock = reference_time + (target_height −
reference_height) × 600 s`, and separately: *"A lock that has already passed is
reported the same way, because the two numbers say so: `LOCKED TO BLOCK 900000,
current height 963663` is a plate that is live now."*

**What an implementer must guess.** What the *estimate* does when
`target_height < reference_height` — which is exactly the already-passed case
the spec explicitly says it supports. §5's legend field is
`LOCKED TO BLOCK <n> ~<year>`; the `~<year>` is still there.

**Candidate guesses.** (A) Signed arithmetic → a past year, engraved:
`LOCKED TO BLOCK 900000 ~2024`. (B) `saturating_sub` → 0 → the reference year:
`~2026`. (C) Suppress the `~<year>` when the delta is negative, engraving
`LOCKED TO BLOCK 900000` alone — which changes the legend's character count and
therefore §5's 136-char budget. (D) Unsigned subtraction → with this project's
`debug-assertions = true` profile, `attempt to subtract with overflow` — a
**panic**, mid-run, after the report has already printed.

**What the spec's logic implies.** (A). §8.4's whole argument is *"`mt` states
the two facts and stops"* and *"facts beat a verdict"* — a past year is a fact,
and the operator can read it. But nothing says so, and (D) is what naive `u32`
code does.

**Observable divergence.** Same PSBT with an expired height lock: A engraves
`~2024` (correct and slightly odd-looking), B engraves `~2026` (**false, and
permanent**), C engraves a shorter line, D crashes. Three of the four write
different permanent text.

---

### B-13 — §4's tie-break is still not a total order: tiling orientation

**Severity: Important.** §4.

**What the spec says.** The objective, after R0's fix:
`1. minimise plates → 2. maximise ECC → 3. minimise symbol count →
4. maximise MODULE SIZE → 5. then minimise QR version`, over a search space
including *"rectangular tiling (across × rows)"*.

**What an implementer must guess.** Which of two tilings wins when they tie on
all five keys. `2 across × 3 rows` and `3 across × 2 rows` have identical plate
count, ECC, symbol count, module size and version — the comparison key cannot
separate them, so the winner is again *"whichever the loop reached first"*,
which is the precise defect §4's own correction note was written to remove.

Two more residues in the same key: the **module-size domain is unenumerated**
(§4 lists it in the search space with no set of values), so *"maximise module
size"* has no maximum unless the domain is discrete — see B-18. And **which
plate a symbol lands on** under a multi-plate tiling is unstated, which
determines what §10.8's `n/m` labels say.

**What the spec's logic implies.** §4's step-4 reasoning is *"break toward
legibility, which is the direction the artifact's purpose demands"*, and a
6th key in the same spirit is available: prefer the tiling closest to square,
tie broken toward more rows (a taller stack leaves the legend's 6 reserved lines
undisturbed). The spec implies a *direction*; it does not supply the key.

**Observable divergence.** Two implementations produce visibly different plates
from the same PSBT — a 2×3 grid versus a 3×2 grid — and §2's stated goal
(*"deterministically and with every tie broken, so two encoders agree"*) is not
met. The symbols still decode, so nothing catches it; the plates simply differ.

---

### B-14 — Is the new `Class` secret, and does a bearer transaction in plaintext flash raise a flag?

**Severity: Important.** §10.9, §7.

**What the spec says.** §10.9 rules the payload **unencrypted**, reasoning that
*"the plate the payload produces is bearer and sits in a drawer, so the wire is
not where this artifact's secrecy lives."* §7 says an `mt` plate *"sits nearer
`ms1` than `md1`"* in hazard terms.

**What an implementer must guess.** `Class::is_secret()` for the new variant
(`crates/me-cli/src/sysw/record.rs:47-56`; Go `sysw/record.go:37`).

**Candidate guesses.** (A) `false` — it is not key material, and §10.9 rules the
payload unencrypted. (B) `true` — §7 puts it near `ms1`, and `is_secret()` is
what raises the device's **F1 flag** (`flagSecretInPlaintext`,
`gui/sysw_admit.go:82`), the screen that tells an operator a secret is sitting
unencrypted in flash.

**What the spec's logic implies.** (A), and the existing precedent settles the
tension rather than leaving it: `Class::FreeText` is documented as *"deliberately
NOT secret even though an operator may put anything in it: a class states what
the format **guarantees**, not what a human might do"* (`record.rs:49-52`). A
transaction is public data by construction. So `is_secret() == false`.

**But the consequence must be written down**, because it is not obvious and it
is exactly what §7's threat model cares about: with `is_secret() == false`, a
bearer transaction sitting unencrypted in flash raises **no flag at all** on the
device — not F1, not F2 — because `syswFlags`' `secret` term is
`c.IsSecret() || unconfirmed`, and `unconfirmed` is computed only for
`ClassMDMK` (`sysw/record.rs:118-125`). Implementer B, reading §7, sets it
`true` and the device warns; implementer A, reading §10.9, sets it `false` and
it does not. Same payload, different device behaviour, and only one of them
matches the threat model the spec wrote.

---

### B-15 — Teaching the shared classifier `mt1` changes `me convert`, `me bundle` and `me seal`

**Severity: Important.** §10.9 (Rust-primary ruling), §7.

**What the spec says.** §10.9: the new class *"lands in `me-cli`'s Rust `sysw`
first, with test vectors, and only then ports to the fork's Go."*

**What an implementer must guess.** *Where* in `me-cli` the recognition lands,
and what it does to `me`'s other four surfaces.

**Traced.** `sysw::classify` (`sysw/mod.rs:124-148`) delegates to
`seal::record::validate_record` (`seal/record.rs:117`), which delegates to
`crate::classify::classify` (`classify.rs:40`) — the shared HRP switch over
`md`/`mk`/`ms`. Recognising `mt1` there is the natural place, and `Format` is
matched exhaustively by `validate::validate` (`validate.rs:75`), so the compiler
forces every consumer to answer. The consumers are `me convert`, `me bundle`,
`me seal --plaintext` and `me sysw pack`.

**Candidate guesses.** (A) Add `Format::Mt` to the shared classifier and let
`me convert` handle it like `md1`/`mk1` — i.e. **convert a bearer transaction to
an NDEF payload for tapping against a phone**. (B) Add `Format::Mt` and
**refuse** it in `convert`, as `ms1` is refused. (C) Recognise `mt1` only inside
`sysw::classify`, bypassing the shared switch.

**What the spec's logic implies — (B), unambiguously, and the spec never says
it.** `me`'s crate description is *"…refuses secret `ms1`"*, and its refusal
carries `ConvertError::RefusedSecret` → `EXIT_REFUSED`. §7 places an `mt` plate
*nearer `ms1` than `md1`*: it is spendable by whoever holds it. Pushing a signed
spendable transaction over NFC to an unauthenticated tag is the shape of hazard
`me` refuses `ms1` for. Today `me convert` on an `mt1` string exits
`EXIT_INVALID` with *"unrecognized HRP 'mt' (expected md, mk, or ms)"* — a
refusal by accident. After the class lands, that accident is gone and option (A)
becomes the default behaviour of a tool nobody re-reviewed.

**Observable divergence.** A ships and `me convert < tx.mt1 --hex` emits an NDEF
payload of a spendable transaction. B ships and it exits 3 with a refusal. The
divergence is in a *sibling tool the spec does not mention*, which is exactly
how it gets missed.

---

### B-16 — Nothing says how or when the reference constant is refreshed

**Severity: Minor.** §8.4.

**What the spec says (at `4527cbc`).** *"A checked-in constant makes the
estimate a property of the release, refreshed **deliberately by a maintainer the
way a checkpoint is**, and never a property of a build environment."* §8.4 also
concedes: *"**The reference pair ages.** A binary built in 2026 and run in 2031
carries a five-year-old anchor, and the error grows with that gap."*

**What an implementer must guess.** Where the constant lives, what refreshes it,
on what cadence, and whether a stale one is ever *loud*.

**Candidate guesses.** (A) A `const` in source plus a release-checklist line;
silent thereafter. (B) A `const` plus a **staleness warning** on stderr once the
pair is older than some horizon. (C) A test that fails when the constant is
older than N months, forcing the refresh at CI time.

**What the spec's logic implies.** §8.4 already commits to half of (B):
*"`mt` prints the reference pair alongside the estimate so the operator can see
how fresh it is."* Printing it is specified; **judging** it is not. And §6a's
sibling principle — *"Enumerating the skipped checks is the point; 'no node'
alone tells the operator nothing they can act on"* — argues that a bare date
stamp is the same non-actionable shape.

**Observable divergence.** A ships a constant that is never refreshed and the
`~<year>` degrades silently across releases; C's build goes red at 12 months and
someone updates it. Divergence cost is low per-run and unbounded over years,
which is why it is Minor rather than Important: it degrades, it does not fork.

---

### B-17 — Where `mt` gets `sysw` from, and the build order that implies

**Severity: Minor.** §1.2, §10.9, §10.13.

**What the spec says.** §1.2: `mt` lives in **its own repository**,
`mnemonic-transaction`, with `mt-codec` and an `mt` CLI. §10.9: the new `Class`
*"lands in `me-cli`'s Rust `sysw` first"*. §10.13: `mt-codec` forks `md-codec`'s
machinery into the new repo.

**What an implementer must guess.** How a binary in repo X emits a container
whose normative implementation lives in repo Y.

**Candidate guesses.** (A) `mnemonic-transaction` depends on
`mnemonic-engrave = "0.7"` as a library — it is published, `[lib] name =
"mnemonic_engrave"`, and `pub mod sysw` is exported
(`crates/me-cli/Cargo.toml`, `lib.rs:10`). (B) `mt` shells out to `me sysw
pack`. (C) `mt` reimplements `sysw` — forbidden by the Rust-primary rule, which
names `me-cli` as primary.

**What the spec's logic implies.** (A). But it carries a **build-order
constraint no section states**: for `sysw::classify` to place an `mt1` record,
`me-cli` must be able to validate one, so **`mnemonic-engrave` gains a
dependency on `mt-codec`** — which means `mt-codec` must be published *before*
the class can land in the Rust primary, which must be released *before* `mt` can
depend on it. Three ordered releases across two repos, and a fourth for the Go
port. An implementer who plans a single cycle discovers this on day one of the
`sysw` work.

Worth noting it is not a crate cycle: `mt-codec → mnemonic-engrave → mt-cli` is
acyclic. It is a *release* ordering, not a compile error, which is why nothing
will catch it automatically.

---

### B-18 — "at the point of choice" implies an interactive surface `mt` does not have, over an unenumerated domain

**Severity: Minor.** §8.8, §10.1, §4.

**What the spec says.** §8.8: *"`mt` offers **every size it can engrave** and
suggests 0.60 mm. Sizes below that are **optically unvalidated**, and `mt` says
so **at the point of choice** rather than refusing."* §10.1 repeats it:
*"User picks from all available options, suggesting 0.6."*

**What an implementer must guess.** (i) What "every size it can engrave" *is* —
no set is enumerated anywhere; (ii) what "offers" and "at the point of choice"
mean in a non-interactive CLI.

**Candidate guesses.** For the domain: a continuous range (in which case
§4's *"maximise MODULE SIZE"* tie-break has no maximum), multiples of the
0.30 mm single stroke, or the fork's own ladder. For the surface: a `--module`
flag with the warning printed *after* the choice; a `--list-modules` subcommand
plus a flag; a genuine interactive picker.

**What the spec's logic implies.** A discrete domain — §4 needs a finite set to
maximise over, and §8.8's own vocabulary ("two engraved strokes" for 0.60 mm)
implies multiples of the 0.30 mm stroke. For the surface: a flag, since §10.10's
CLI is batch-shaped and stdout carries the artifact. Under a flag, "at the point
of choice" can only mean *the warning prints when a sub-0.60 value is passed* —
which is after the choice, not at it. That is fine and should be said.

**Observable divergence.** A accepts `--module 0.45` (not a stroke multiple) and
engraves something the machine quantises; B refuses anything off the ladder; C
prompts and cannot be scripted. §4's search returns different answers over
different domains.

---

### B-19 — §8.2c's operator-supplied input value: no flag, no units, no granularity

**Severity: Minor.** §8.2c.

**What the spec says.** *"Where a record is absent, `mt` **requires** the
operator to supply that input's value — or the total across all inputs."*

**What an implementer must guess.** The flag, its unit, and its shape.

**Candidate guesses.** `--input-value 3=1.5` (indexed) vs
`--input-total 2.75` (aggregate) vs both; **BTC vs satoshis**; whether an
indexed form may be repeated; whether supplying a value for an input that
*already* has a UTXO record is an error or an override.

**What the spec's logic implies.** Both forms, since the spec says *"or"*
explicitly. Units: **BTC**, because §8.2c's own worked example is written in
BTC to eight places (`1.00000000 BTC`, `0.01000000 BTC`).

**Observable divergence.** An operator typing `--input-value 0=100000000`
meaning one BTC in sats gets, under a BTC-parsing build, an input of 100 million
BTC — caught by §8.2b's `AbsurdFeeRate` refusal, loudly, which is the saving
grace and why this is Minor rather than Important. The reverse typo
(`0=1.0` meaning 1 sat) is caught by `SendingTooMuch`. Both directions refuse;
neither is silent. But A and B disagree on which invocations work at all.

**One case neither guess covers:** an *override*. If the PSBT carries a
`witness_utxo` and the operator passes a conflicting `--input-value`, is that a
refusal (two sources of truth, §6's own hazard argument) or an override? §6's
*"on disagreement a recoverer would have to guess which to believe. That is a
funds-safety hazard, not a feature"* implies **refuse**. Nothing says it.

---

### B-20 — `mt string`'s stdout framing and the bearer warning's text

**Severity: Minor.** §3b, §10.10.

**What the spec says.** §3b: *"`mt string` **emits a string. That is the whole of
its output.**"* and separately *"a chunked codex32 string"*, up to 64 chunks.
§3b also rules a stderr warning that the artifact is **bearer** — with no text.

**What an implementer must guess.** (i) How multiple chunks are framed on
stdout: one per line, space-separated, or concatenated; (ii) trailing newline;
(iii) the warning's exact wording.

**What the spec's logic implies.** One chunk per line. The constellation's own
convention is LF-separated records everywhere — `me sysw pack --in` reads
*"newline-separated records"*, `me bundle` reads *"newline-separated public
strings"*, and `sysw`'s own record separator is LF *"on the stated grounds that
no constellation string contains a newline"* (`sysw/record.rs:8-11`). So LF-per
chunk, with a trailing newline, is implied — but a spec that says *"emits a
string"* (singular) for something that is up to 64 strings will produce at least
one implementer who concatenates.

**Observable divergence.** A hand engraver's worksheet generated from A's output
has 14 lines; from B's it has one 1,120-character line. Downstream `wc -l` and
any chunk-counting script differ.

**On the warning text**, the divergence is real but low-cost: §3b calls this
warning the *entire* mitigation for `mt string`'s bearer hazard (§7 records it
as an accepted risk with no plate-side mitigation), so it is the one sentence
carrying that whole row of the threat model — and it is unwritten, while §6a,
§8.2b and §8.2c all get their text pinned verbatim. Asymmetric.

---

### B-21 — Legend rendering is unspecified below the field list

**Severity: Minor.** §5, §10.8.

**What the spec says.** Five fields, 136 characters, 6 lines, with per-field
character budgets. Plus §10.8's per-symbol `n/m`.

**What an implementer must guess.** Everything about the *rendering*:

| question | candidates |
| --- | --- |
| field **order** on the plate | §5's table order is `BEARER / FROM / LOCKED / TO / PLATE`, and §3b confirms `BEARER` is *"the first line of a legend `mt` controls"* — so the order **is** implied, and only the first line is stated outright |
| **line breaks** — 5 fields over 6 lines | which field wraps to two? `BEARER…` is 41 chars against §5's ~35/line, so it is the natural candidate — implied, not stated |
| an **absent optional field** (`FROM`, `TO`) | omit the line entirely (5 lines), print the label with nothing after it, or print an explicit `FROM WALLET UNKNOWN`. §5 reserves 6 lines in §4's plate budget either way, so omitting frees space §4 already spent |
| **truncation** of a too-long free-text label | §10.4 **decides this**: *"Refusing with the limit named fits §8's rule…; silent truncation does not"* — implemented, this is a refusal, and it needs a number under B-7 |
| where the `n/m` **sits** relative to its symbol | above / below / left; §10.8 says only *"beside"*. It is *"unpriced"* by §10.14's own admission |
| **case** | every §5 example is uppercase; the fork's engraving font is uppercase-only in practice. Implied, unstated |

**What the spec's logic implies.** Order and first-line are implied; wrap point
is implied by arithmetic; truncation is explicitly decided. **Absent-field
behaviour and `n/m` placement are genuinely open.**

**Observable divergence.** For a transaction with no `TO` supplied, A engraves a
5-line legend and B engraves 6 with a bare `TO`. Both are legible; they are
different permanent artifacts, and A's frees 4.25 mm that §4's search did not
know it had.

---

### B-22 — No severity markers, no tool prefix

**Severity: Nit.** §6a, §8.2b, §8.2c.

Three warning bodies are pinned verbatim and each opens differently:
`WARNING: no bitcoind reachable.` (§6a), `WARNING: fee rate is 3.2 sat/vB.`
(§8.2b), `WARNING: input 0 is a legacy (pre-SegWit) input.` (§8.2c). Two others
are described only as *"loudly warned"* (§10.4, `FROM`/`TO` blank) and one has no
text at all (§3b's bearer warning). Nothing says whether the `mt: ` prefix `me`
uses everywhere applies, whether there is a NOTE/WARNING/ERROR ladder, or how
the loud-but-unwritten two are marked.

`me`'s house style implies `mt: WARNING -- …` (`main.rs:483`) or plain
`mt: warning: …` (`main.rs:315`) — and `me` itself is inconsistent between those
two, so there is no clean precedent to inherit. Low cost; affects only log
grepping.

---

### B-23 — A passphrase on an `mt qr` payload would silently no-op

**Severity: Nit.** §10.9.

§10.9 rules the payload **unencrypted**, so `mt` should offer no passphrase at
all. Recording this because the sibling's default is the *opposite* — `me sysw
pack` **generates** a passphrase unless `--no-passphrase` is passed
(`main.rs:836-868`) — so an implementer mirroring `me`'s CLI produces a sealed
payload by default, against the ruling.

And if they offer one anyway, it does nothing. **Executed:**

```
$ me sysw pack <md1-record> --passphrase-words 4 --out p.bin
passphrase — write this down and store it APART from the machine:
    margin soda scheme home
$ me sysw show p.bin
sealed:   false
ct_len:   0
```

Only `is_secret()` classes are encrypted (`sysw/mod.rs:268-271`), so a payload
holding one non-secret record seals nothing: `ct_len` stays 0, `sealed()` is
`ct_len > 0` (`wire.rs:88-90`), and the operator has written down a passphrase
that protects nothing. Under B-14's implied `is_secret() == false`, a
transaction record behaves exactly this way. **The correct move is to offer no
passphrase flag**, and to say so.

---

## The sysw gates

Every constraint a new transaction `Class` must satisfy, traced through the
executed code. **R3 lens 3's C-2 was right that gates exist and that the spec
names none — but two of its four cited gates do not bind `sysw`.** Verified:
`SplitSection` (`MaxRecords = 24`, `MaxRecordLen = 512`, `seal/wire.go:57-58`)
is called only from `seal/unlock_key.go:92` and `seal/open.go:144` — the
**Sealed Payload** container. `sysw`'s device-side splitter is
`sysw/open.go:67-74`, a bare `strings.Split(s, "\n")` with **no record count or
length cap at all**. Likewise the *"public allow-list… admits only ClassMDMK"*
(`seal/record.go:467-469`) is `seal`'s; `sysw`'s host-side `split`
(`sysw/mod.rs:251-265`) admits any non-secret class, and its
card-set decode rule **reports rather than refuses** by deliberate design
(`sysw/record.rs:96-101`).

Corrected and complete:

| # | gate | where (executed code) | what a new `Class` must satisfy | binds? |
| --- | --- | --- | --- | --- |
| **G1** | **`classify` recognition** | `sysw/mod.rs:124-148` → `seal/record.rs:117` → `classify.rs:40-52` | the record must be placeable. Order is fixed: `pass:` → `text:` → BIP-39 → HRP switch (`md`/`mk`/`ms`). An `mt1` record falls through to `UnknownHrp` today. **Verified by running `me sysw pack`** on all three candidate framings — all three refused | **hard** |
| **G2** | **fail-closed at pack** | `sysw/mod.rs:253` | `Class::Unknown` → `SyswError::Unclassifiable(i, reason)` → `me` exits 4. The host **refuses**; the device is merely inert (`gui/sysw_session.go:100-106` classifies and the record never matches a `take`). An implementer landing Go before Rust gets silence instead of an error | **hard** |
| **G3** | **canonical-string rule (EPD §6.4)** | `validate.rs:63-66`, `seal/record.rs:118-128` | **no interior whitespace, no `-`, no uppercase anywhere.** `RecordError::NotLowercase` fires on the first uppercase char. This is what killed base45 (§3) and what forces §3's *"the record stores lowercase"* | **hard** |
| **G4** | **`MAX_SECTION_LEN = 8191`** | `wire.rs:42`, enforced in `Header::parse:133` and re-run by `bound()` (`mod.rs:310-316`) | `pub_len ≤ 8191`, counting **LF separators**. §8.7c cites this and its arithmetic omits the separators and any per-record framing — see B-2 | **hard** |
| **G5** | **`REGION_LEN = 65536`** | `wire.rs:18`, `mod.rs:311` | total blob ≤ 64 KiB. Never binding while G4 holds; binds the `--region` image | soft |
| **G6** | **LF is the record separator** | `sysw/mod.rs:259`, `sysw/open.go:74` | no record may contain `\n`, and none may be **empty** — `split_records` on a section with a stray LF yields a zero-length record that `classify` returns `Unknown` for → G2 refuses the whole payload | **hard** |
| **G7** | **`Class::is_secret()`** | `sysw/record.rs:47-56`, `sysw/mod.rs:255-256`, Go `sysw/record.go:37` | decides public vs encrypted section, and drives device flag **F1** (`gui/sysw_admit.go:82`). Implied answer: `false` (B-14) — with the consequence that a bearer transaction in plaintext flash raises **no flag** | **decision** |
| **G8** | **device admission table** | `gui/sysw_admit.go:31-45,54` | `admitted[program][class]`; **absent = refused**. A transaction class needs a new `syswProgram` *and* a row. Every existing program refuses it by default, which is the correct fail-closed shape — and means nothing on the device can consume it until §10.17's firmware work lands | **hard (device)** |
| **G9** | **payload authentication before use** | `gui/sysw_session.go:114-118` (`!s.loaded \|\| !s.compared`) | no record reaches a program until the payload's §6.6 digest has been confirmed. Nothing to do for a new class, but it means the operator must compare a digest — a step no §10.10 workflow mentions | procedural |
| **G10** | **`MDMKUnconfirmed` does not extend** | `sysw/record.rs:118-125` (`if super::classify(r) != Class::MdMk { continue; }`) | a transaction record is never decode-confirmed, so `unconfirmed` is always `false` for it and §12.6's "the device treats an undecodable card as a SECRET" safety net does **not** cover it. Decide whether to extend the rule or accept the gap | **decision** |
| **G11** | **cross-language conformance** | `sysw/classify_conformance_test.go:11-40`, `sysw/vectors.rs`, `testdata/sysw_vectors.json`, `sysw/coverage.rs:38-95` | Rust is normative; the Go port is pinned against Rust's answers row by row. A new class needs rows in the conformance test **and** a vector, and `coverage.rs`'s `assert_every_named_test_is_placed` fails the build if a spec test has no entry | **hard** |
| **G12** | **release ordering** | `Cargo.toml` (`mnemonic-engrave 0.7.0`, `md-codec = "0.42"` from the registry) | G1 needs `me-cli` to validate an `mt1`, so `mnemonic-engrave` gains a dep on `mt-codec`; `mt-codec` must publish first, then `mnemonic-engrave`, then `mt`. Acyclic, so nothing errors — it just cannot be done in one cycle (B-17) | procedural |

**One positive result, recorded so nobody re-derives it.** The unsealed `sysw`
container is **byte-deterministic**: the unsealed path returns before salt/IV are
installed (`mod.rs:186-190`, header from `Default`). Two packs of the same record
produce the same SHA-256 —
`8a8ca334…` twice, executed. So the transport contributes **nothing** to §2's
determinism requirement; every nondeterminism risk lives upstream in `mt`
(B-6, B-13).

---

## Ranked decision list

Ranked by divergence cost — what it costs if two implementers answer
differently. Items 1–6 are the ones that make the *artifact* differ or make the
tool unbuildable.

| # | decision | finding | cheapest resolution |
| --- | --- | --- | --- |
| 1 | **The full flag surface** — seven operator inputs have nowhere to arrive; §8.7's plate-budget refusal is unrunnable without one | B-1 | a §10.10 flag table, modelled on `me`'s `clap` surface |
| 2 | **What one `sysw` record contains** — 4 framings, 4 different §8.7c ceilings (3,671 / 4,094 / 4,476 / 4,525 B), and the EPD-conformant one refuses the spec's own largest artifact by 322 B | B-2 | pick the framing, then **recompute §8.7c** |
| 3 | **How §4's configuration and §5's legend reach the device** — currently no channel at all | B-3 | a second record class or a defined field; state which |
| 4 | **What `mt qr` writes and where** — bare container / region image / UF2, and binary on stdout | B-4 | follow `me sysw pack` + `--region`, and require `--out` |
| 5 | **The node: location, credentials, timeout, and what a non-answer means** — §8.5 is a funds-safety refusal that fires or does not | B-5 | explicit `--rpc-*` flags per §10.5; a table of the five non-answer cases |
| 6 | **Whether the engraved `~<year>` comes from the constant or the live chain** — the plate currently depends on the operator's network | B-6 | rule: the engraved year always uses the embedded constant; the live node sharpens only the stderr report |
| 7 | **Refusal format, and which number a duplicate refusal names** — §8.1 vs §8.3 are the same check; `7c` precedes `7b` | B-7 | delete item 3, reorder 7b/7c, pin `mt: refused (§8.N): …` |
| 8 | **Exit codes** | B-8 | adopt `me`'s 0/2/3/4 verbatim; classify each §8 item |
| 9 | **Input encoding and the file/stdin shape** | B-9 | sniff `psbt\xff` / `cHNidP8`; `--in FILE`, stdin otherwise |
| 10 | **What §8.9 refuses** — undefined on a PSBT; the real hazard is a seed in the free-text label | B-10 | reuse `me`'s `is_seed` over every operator-supplied string |
| 11 | **Success-report format, ordering, change detection, provenance labels** | B-11 | pin the seven rows like §8.4 pins the locktime line; drop or fund change detection; add the fourth provenance state |
| 12 | **Negative block delta** when the lock is already passed | B-12 | signed arithmetic, engrave the past year |
| 13 | **§4's tiling tie-break** — the order is still not total | B-13 | add a 6th key |
| 14 | **`Class::is_secret()`**, and whether a bearer transaction in plaintext flash flags | B-14 | `false`, and state the F1 consequence |
| 15 | **`me convert` on an `mt1`** — currently refused by accident, would start converting | B-15 | refuse `Format::Mt` in `convert`, as `ms1` is refused |
| 16 | reference-constant refresh cadence and staleness | B-16 | a CI age check |
| 17 | crate dependency direction and the three-release ordering | B-17 | state it in §10.9 |
| 18 | module-size domain, and what "at the point of choice" means | B-18 | enumerate the ladder; warn on the flag |
| 19 | `--input-value` name, units, and the override-vs-refuse case | B-19 | BTC per §8.2c's example; refuse on conflict per §6 |
| 20 | `mt string` stdout framing and the bearer warning's text | B-20 | one chunk per line; pin the sentence |
| 21 | legend rendering: absent fields and `n/m` placement | B-21 | rides with §10.14's regeneration |
| 22 | severity markers / tool prefix | B-22 | pick one of `me`'s two styles |
| 23 | no passphrase flag on `mt qr` | B-23 | omit it; it would silently no-op |

**The one-line summary for the operator.** The codec decisions are in good shape;
the *tool* is not yet specified. §10.10 is titled "The CLI surface — RULED" and
rules four things (two verbs, PSBT input, stdout/stderr split, no locktime
flags), all of which are sound. What it does not do is describe a command line,
and six of the seven inputs the rest of the spec depends on have no way in.
Fixing item 1 and item 2 collapses roughly half of this list.
