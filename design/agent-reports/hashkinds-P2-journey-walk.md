# `ms hashlock` — operator journey walk (phase 2, branch `hashkinds-p2`)

**Lens:** operator journey, not correctness. The question is *"what does a real
person have in hand at each step, and what happens when they reasonably do
something else?"* — findings are **missing things at moments**, not wrong things
in sections.

**Tree:** `/scratch/code/shibboleth/ms-worktrees/hashkinds-p2` @ `dabd518`
(`hashlock: say why ripemd serves two of the four kinds`), clean before and
after this walk.
**Binary:** built with the pinned toolchain — `cargo 1.85.0 (d73d2caf9
2024-12-31)`, `cargo build -p ms-cli --locked` → `./target/debug/ms`
(ms-cli 0.19.0 / ms-codec 0.10.0). Every command below was **executed**; all
output is pasted verbatim.

**Downstream binaries used, so the cross-tool steps are observed and not
assumed:**
- `me` — `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`, **0.9.0**.
- `md` — the installed `md` 0.14.0 has **no `compose` subcommand** (stale
  binary). I rebuilt from `descriptor-mnemonic` HEAD `40c400de` with
  `cargo build -p md-cli --locked` and used `./target/debug/md`, which does.
  Only `target/` was touched; `git status --short` is empty in that repo.

**Scratch:** `/tmp/journeymrSb` (outside both repos), removed at the end. No
tracked file was modified.

---

## Journey A — authoring a `ripemd160` hashlock

The operator has decided on `ripemd160` and has a phrase on paper:
`correct horse battery staple`.

### A.1 — They type the command name and nothing else

    $ ms hashlock
    error: no source given; exactly one source: --hashlock-phrase TEXT,
    --hashlock-phrase-stdin, --hex HEX, an ms1 string (argument, `-`, or
    --in FILE), or --random
    [exit 64]

Same under a real PTY (`script -qec ... /dev/null`). **No hang, no silent
block**, and the message enumerates every source. Good.

### A.2 — They use the stdin channel and type nothing yet

    $ ms hashlock --hashlock-phrase-stdin --kind ripemd160      # on a TTY
    Type the hashlock phrase, then Enter.
    error: the hashlock phrase is empty

**The prompt is printed before the read.** The historic "blocks on a TTY with no
prompt, so the first action looks like a hang" trap is not present here.

### A.3 — The derivation

    $ printf 'correct horse battery staple' \
        | ms hashlock --hashlock-phrase-stdin --kind ripemd160

stdout (one line):

    hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b

stderr (the engraving card):

    THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
    digest:          09e7bb5051d89788fb4e4b374126721dbcc2946b
    for md compose:  --path ... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b
    preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew p73sc ptwxg tjugs pvs8t klufg 89hqj
    preimage (hex):  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
    method:          preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)
    phrase:          28 characters -- write the method line next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries it; ...
    The preimage must be exactly 32 bytes (64 hex characters): the script checks OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).
    One phrase per policy. Spending any path of a wsh wallet publishes this digest. ...
    source:          phrase (stdin)

**What the operator has in hand, exactly:** a 40-hex digest, a 64-hex preimage,
a 75-character ms1 string, and a card. Two things on that card are wrong or
short for a `ripemd160` operator — see **I-1** and **I-4**.

I ran all four kinds. The record is bare for `sha256` (`hash:<64hex>`) and
tagged for the other three (`hash:<kind>:<hex>`), per SPEC S6. **The `preimage
(ms1)` string is byte-identical across all four kinds** —
`ms10hashsq0p7ja…89hqj` — because the preimage is kind-free. That is by design;
§13.1 puts the kind in the plate's QR text and locator row.

### A.4 — They paste the card's operand into their descriptor

The card said, verbatim: `for md compose:  --path ... ripemd160=09e7bb…946b`.

    $ md compose --wrapper wsh --path '2of3,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b'
    md: path `2of3,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b`: unknown option `ripemd160`
    [exit 1]

Control — the `sha256` spelling works:

    $ md compose --wrapper wsh --path '2of3,sha256=3cf5d421…4c12'
    wsh(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/…,@2/…),sha256(3cf5d421…4c12)))
    note: stdout is a keyless descriptor template (no keys)
    [exit 0]

`md compose --help` confirms the grammar accepts only `sha256=HEX`. **The
operator was handed a command fragment that does not exist, with the same
confidence as the one that works**, and the refusal reads like a typo. See
**I-2**.

### A.5 — They try the pipe the help text advertises

`ms hashlock --help` EXAMPLES ends with:
`ms hashlock --hashlock-phrase-stdin < phrase.txt | me sysw pack --out payload.bin`

    $ printf 'correct horse battery staple' \
        | ms hashlock --hashlock-phrase-stdin --kind ripemd160 --no-engraving-card \
        | me sysw pack --no-passphrase --out p1.bin
    me: record 0 (records count from 0) is a `key:`/`hash:`/`now:`/`phrase:` record whose body fails its rule (not exactly 64 lowercase hex characters).
          record 0: hash: must be exactly 64 hex characters
          Build the record with `me sysw pack`'s helpers: … a hash record is `hash:` + the 32-byte digest as 64 lowercase hex; …
    [exit 4]

Identical refusal for `--kind hash256`. `--kind sha256` / no kind packs fine
(exit 0). **`me sysw pack` refuses rather than misparsing — no silent
acceptance.** But the refusal mentions kinds nowhere, and its stated remedy is
the setup for **I-3**.

### A.6 — Cutting the preimage plate

Not reachable from the host in this phase (`me bundle` cannot cut a preimage
plate — spec §13.5, already a recorded follow-up). `ms_codec::hashlock::qr_text`
**does** take a `HashKind` and emits `hash: <kind>` on its own line
(`crates/ms-codec/src/hashlock.rs:377`), so phase 2's half of §13.1 is present;
the renderer that consumes it is phase 4. **Context, not a finding.**

---

## Journey B — verification, months later

The operator holds a preimage plate (an `ms1` string) and a descriptor
containing `ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b`.

### B.1 — They type the plate string on the command line

    $ ms hashlock ms10hashsq0p7ja…89hqj --kind ripemd160
    ms: argument 2 on ARGV … is an ms1 string (or one share of an ms1 share-set), 75 characters long.
          Refused BEFORE the command line was parsed; nothing was read and nothing was written.
          …
          Use a private channel instead:
              ms hashlock --in FILE      # read it from a file
              ms hashlock -              # or pipe it on stdin
          …
    [exit 1]

The `[MS1]` positional is documented in `--help` but the argv guard always
refuses it. The refusal is thorough, names two remedies, and includes a
per-shell history-purge recipe. **Correct behaviour, well delivered.** Note the
refusal runs *before* parsing, so `--kind` is never reached — which is right.

### B.2 — They follow the remedy, kind known

    $ ms hashlock --in plate.ms1 --kind ripemd160
    hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
    [exit 0]
    # card:
    digest:          09e7bb5051d89788fb4e4b374126721dbcc2946b
    for md compose:  --path ... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b
    method:          preimage supplied
    The preimage must be exactly 32 bytes … before OP_SHA256 …
    One phrase per policy. … Never use this phrase as a passphrase …
    source:          preimage supplied (ms1 plate)

The digest matches the descriptor by eye. **The tool never says whether it
matches** — the operator compares 40 hex characters themselves. (Not acted on;
see below.) Note `method: preimage supplied` and the *"Never use this phrase"*
sentence on a route with no phrase — the wording issue of **M-1**.

### B.3 — Same plate, kind NOT known (a plate cut before `hash:` existed)

    $ ms hashlock --in plate.ms1
    hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
    [exit 0]
    # card … then:
    no --kind given; stdout carries the sha256 record. This phrase's digest under each kind:
      sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
      hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
      ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
      hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd

**This is §13.4 working, and it is the best thing in the release.** The operator
whose descriptor says `ripemd160=09e7bb…` scans the table, finds the row, and is
saved from concluding the plate is wrong. I verified the table survives every
way an operator might suppress output:

| invocation | four-digest table present? |
| --- | --- |
| plain | yes (after the card) |
| `--no-engraving-card` | **yes** — card gone, table kept |
| `--json` | yes on stderr, **and** as `digests_by_kind` + `kind_specified:false` in the object |
| `--json --no-engraving-card` | moves into the object (per the code's documented contract) |
| stdout piped to `me sysw pack` | yes — stderr stays on the terminal (observed) |

With `--kind` given, `digests_by_kind` is correctly **absent** from the JSON and
the stderr table is not printed.

### B.4 — They reach for the other verbs

    $ ms verify --in plate.ms1
    error: this is a hashlock preimage plate, not a seed backup; use `ms hashlock <ms1>` (or `ms hashlock --in FILE`) to re-derive its digest

    $ ms inspect --in plate.ms1
    OK: would decode v0.8
    … tag: hash … kind: preimage

`verify` routes them correctly. `inspect` prints `kind: preimage` — a second,
unrelated meaning of "kind" (see **M-2**).

---

## Journey C — the wrong turn

An operator with a `hash160` wallet who has never heard of `--kind`.

### C.1 — The obvious thing

    $ printf 'correct horse battery staple' | ms hashlock --hashlock-phrase-stdin
    hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
    # card:
    for md compose:  --path ... sha256=3cf5d421…4c12
    # then:
    no --kind given; stdout carries the sha256 record. This phrase's digest under each kind:
      … hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd

They compare `sha256=3cf5d4…` against their descriptor's `hash160=b5b72c…`:
mismatch. **The table two lines below contains their value.** The "discard a
good plate" ending is *blocked*, on every output mode I could construct. This is
the cycle's designed mitigation and it holds.

### C.2 — How far can they get toward an unspendable wallet?

Both downstream doors refuse a tagged record:
- `md compose` → `unknown option ripemd160` (A.4)
- `me sysw pack` → `hash: must be exactly 64 hex characters`, exit 4 (A.5)

So the straight path is closed. **The path that is open is the repair the error
message itself prescribes.** The operator holds
`hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488`.
The error says the body must be exactly 64 lowercase hex. They delete the
`hash256:` prefix — and the remaining body **is** exactly 64 hex:

    $ printf 'hash:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488\n' \
        | me sysw pack --no-passphrase --out p4.bin
    sealing:  NOT SEALED …
    digest:   e24a 8844 1177 1c43 e9bf a8ac 453d 3867
    [exit 0]

    $ me sysw show p4.bin
    public record 0: sha256 hashlock (hash:) — 98a20fc2..641cd488

**A `hash256` digest is now in the payload, labelled `sha256 hashlock`.** The
device composes `sha256(98a20fc2…)`; the operator believes they built a
`hash256` wallet; the preimage on their plate does not satisfy the branch. See
**I-3**. `ripemd160` and `hash160` are immune — their digests are 40 hex, so the
strip does not produce an accepted body and the operator stays stuck (safe).

### C.3 — Kind typos

    --kind SHA256      error: invalid value 'SHA256' for '--kind <KIND>': unknown hash kind "SHA256": expected sha256, hash256, ripemd160 or hash160 (lowercase; case is rejected, never folded)
    --kind Ripemd160   (same shape)
    --kind ripemd-160  (same shape)
    --kind rmd160      (same shape)
    --kind hash-160    (same shape)
    --kind sha-256     (same shape)

Six spellings, six identical well-formed refusals that list all four tokens and
state the case policy. **Exemplary. No finding.**

---

## Findings

### Critical — none.

### Important

**I-1 — The engraving card names `OP_SHA256` on all four kinds.**
`crates/ms-cli/src/cmd/hashlock.rs:484` writes, unconditionally:

    The preimage must be exactly 32 bytes (64 hex characters): the script checks OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).

The spec's own bounding fact F1 (`SPEC_hashlock_kinds.md:55-56`) writes the
shape as `OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL` — *"only the digest
width moves"*, and `<hashop>` is the variable. A `ripemd160` script contains
`OP_RIPEMD160`, not `OP_SHA256`. The card is the artifact the operator keeps
beside the preimage, and it asserts the wrong hash function **on precisely the
axis this cycle exists to disambiguate** — so it is also the line a
kind-confused operator would use to self-check, and it would confirm the wrong
answer. The `OP_SIZE 32` half is correct for all four and must survive the fix.
*Classification: **warning/correction** — the line varies with `--kind`.*
*Earns a change:* it is not silence, it is a false statement about the object in
hand. Nothing would be strictly better than this.

**I-2 — A non-sha256 kind yields an operand and a record no shipped tool
accepts, and nothing says so.** Observed in A.4 and A.5: `md compose` answers
`unknown option ripemd160` to the exact fragment the card printed, and
`me sysw pack` answers `hash: must be exactly 64 hex characters` to the exact
pipe `ms hashlock --help`'s EXAMPLES advertises. Neither error mentions kinds;
both read as "you made a typo", not "this kind is not wired up yet". A grep of
`ms hashlock --help` and of `crates/ms-cli/src/` finds **no** operator-facing
text about downstream kind support.
*Classification: **warning** — one line on the card (and in `--help`) when a
non-sha256 kind is chosen.*
*Earns a change:* the tool proposes a specific command fragment with full
confidence. Telling the operator nothing would at least not have proposed a
string that cannot work, so the wrong outcome here is worse than silence.

**I-3 — The `hash256` record is one prefix-strip from a `sha256` record, and the
downstream error instructs the strip.** Full reproduction in C.2. `me sysw show`
labels the result `sha256 hashlock`. This is the ms-side face of the spec's own
§13.2 — *"Both are 64 hex; nothing but a label distinguishes them, and the label
was never printed"*, which §13.2 calls **the cycle's operator-facing Critical**.
§13.2 assigns the device screens; nothing assigns the moment of emission, which
is where the operator first holds both strings.
*Classification: **warning** — at emission, when `--kind hash256` is chosen.*
*Earns a change:* the end state is a funded wallet whose hashlock branch the
operator's plate does not satisfy, reached by following an error message
literally. Graded Important rather than Critical because `ms`'s own record is
correct, the downstream tool refuses, and the operator must hand-edit to get
there.

**I-4 — The card's write-down instruction names the method axis and omits the
kind axis.** `hashlock.rs:482`:

    phrase: 28 characters -- write the method line next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries it; …

With `--kind ripemd160 --emit-record`, the emitted record is
`phrase:68617264656e65642c…` = hex of `hardened,correct horse battery staple` —
**method, no kind**. An operator complying exactly writes down `hardened` and
nothing about `ripemd160`. The spec's own §13.2 states the rule this breaks:
*"A write-down list that omits a field is worse than no list, because the
operator stops writing where the list stops."* §5's two axes are method
(phrase→preimage) and kind (preimage→digest); the card's instruction covers one.
*Classification: **documentation** — the sentence gains the kind.*
*Earns a change:* on the spec's own stated rule. Graded Important rather than
Critical because the kind is recoverable from the descriptor / md1 template, and
because B.3's four-digest table turns the recovery into a lookup.

### Minor

**M-1 — The no-kind fallback calls the object a phrase on every route,
including the two where the card says no phrase exists.** `hashlock.rs:531`:
`"… This phrase's digest under each kind:"`. On `--random` the card prints, two
lines apart:

    No phrase exists, so nothing can be guessed, and nothing can be remembered. …
    no --kind given; stdout carries the sha256 record. This phrase's digest under each kind:

Same on `--hex` and on the ms1-plate route, where the card says
`source: preimage supplied (ms1 plate)`. Journey B's operator holds a plate, not
a phrase. Related wording on the same routes: `method: preimage supplied`
followed by *"Never use this phrase as a passphrase…"*. The digests are correct,
so this costs confidence rather than accuracy — but note commit `cfdb738` was
itself *"the plate-vs-record conflation"*, so this is that class surviving.
*Classification: **documentation only**.*

**M-2 — Two meanings of "kind" in one CLI.** `ms hashlock --kind` is the hash
kind; `ms inspect` prints `kind: preimage`, the payload kind. A Journey B
operator asking *"which kind is my plate?"* gets `kind: preimage` from the verb
whose job is to describe the plate. The value sets do not overlap, so the
misreading is brief. *Classification: **not our concern** — I am not
recommending a rename; `--kind` is the spec's name and is right.*

### Secret handling (logged, never blocking, per the standing rule)

**No new defect found.** The argv guard refused the ms1 positional *before*
parsing; `PR_SET_DUMPABLE` is set; `--out` writes owner-only; the
`PrivateKeyMaterial` advisory fired under `--json`; `--emit-record` keeps the
phrase record on stderr and off stdout; and `--emit-record` with `--hex` /
`--random` is a usage error rather than a silent omission. Recorded as a
measurement, not a finding.

---

## Divergences I am NOT recommending a change for

1. **`for md compose: … sha256=<h>` printed unqualified when `--kind` is
   omitted.** §13.4 says the tool "must not silently assume sha256". It is not
   silent: the `no --kind given…` notice is on the same stream, and I verified
   it survives `--no-engraving-card`, `--json`, and a pipe to `me sysw pack`.
   The code comment at `hashlock.rs:515-528` shows this exact pairing was
   deliberately narrowed in R0 round 5. With the notice always adjacent, the
   wrong outcome is **not** worse than telling the operator nothing.
2. **No `--expect` / compare mode in Journey B.** The operator eyeballs 40 hex
   against a descriptor. That is a feature request, not a divergence — nothing
   the tool says is wrong, and adding it is scope this lens must not generate.
3. **`[MS1]` documented as a positional but always refused by the argv guard.**
   The refusal is excellent, names two remedies, and is a deliberate
   secret-handling protection. By design.
4. **No preimage plate can be cut from the host.** Spec §13.5, already a
   recorded follow-up owned by this cycle's plan. Phase boundary, explicitly
   out of scope per the brief.
5. **`ms1` preimage string is byte-identical across all four kinds.** Correct —
   the preimage is kind-free; §13.1 puts the kind in the QR text (present in
   `qr_text`, verified) and the locator row (phase 4).
6. **Redirecting stderr to `/dev/null` loses the four-digest table.** Not
   plausible operator behaviour for a tool whose every useful line is on stderr,
   and the realistic pipe (stdout → `me sysw pack`) preserves it, which I
   observed. Acting on this would mean moving the table to stdout and corrupting
   the record channel.
7. **Kind-token validation messages.** Six spellings tested, all refused
   identically and informatively. No change.

---

## Counts and verdict

| severity | count |
| --- | --- |
| Critical | 0 |
| Important | 4 (I-1, I-2, I-3, I-4) |
| Minor | 2 (M-1, M-2) |
| Secret-handling (non-blocking) | 0 |
| Divergences examined and declined | 7 |

## **NOT GREEN** — 0 Critical / 4 Important.

The core of phase 2 is sound and the designed mitigation is real: `--kind` is
well validated, the record grammar follows S6, and §13.4's four-digest fallback
survives every output mode I could construct — it closes Journey C's
"discard a good plate" ending, which is the ending that would have cost an
operator five re-cut plates.

What the walk found is that **the card and the help text still describe a
one-kind world at three moments**: the script line names `OP_SHA256` whatever
kind you chose (I-1), the operand and the advertised pipe lead to tools that
refuse the new kinds with no hint that the kind is why (I-2), and the write-down
instruction stops at the method axis (I-4). Each is a *missing thing at a
moment* rather than a wrong thing in a section, which is why five correctness
rounds passed over them. I-3 is the sharpest: the one place where an operator
following a refusal's own printed remedy converts a `hash256` record into a
`sha256` one that every downstream tool accepts and labels as such.
