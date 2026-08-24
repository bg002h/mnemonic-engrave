# JOURNEY WALK — Engrave a Transaction

**Live walk with the operator, 2026-08-24.** This is the review of
`SPEC_engrave_transaction.md`, by operator ruling: the walk generates the
refusals rather than the spec imagining them.

**Method.** At each step: what does the operator have in hand *exactly*, what
does the tool do, and **what else might they reasonably do**. Every divergence is
classified **refusal / warning / default / not our concern / documentation
only**, and earns a change *only* if the wrong outcome is worse than telling the
operator nothing.

**Journey A — "I have a signed transaction and I want it on steel."**

---

## Step 1 — the operator's first command

Typed, verbatim, unprompted:

```
mt encode --record < tx.final.psbt | me sysw pack > payload.bin
```

...followed by **"(But I might be piping incorrectly)"**.

**The parenthetical is the first finding.** The operator designed this pipeline
an hour earlier and was not sure it worked. That is a measurement of the design,
not a lapse — and they were right while the spec was wrong.

### A — the pipeline does not compose, and §1 claimed it did

`me sysw pack` takes `--in <FILE>` and `[RECORDS]...` on argv. **There is no
stdin path.** Measured:

```
$ printf 'text:6869\n' | me sysw pack --no-passphrase > out.bin
me: no records: pass them on argv or with --in
exit=2, stdout 0 bytes
```

`SPEC_engrave_transaction.md` §1 says the two tools *"compose over a pipe."*
False.

**The TOOL behaves well** — exit 2, a message naming both real options, empty
stdout, no silently-empty container. So this was a **spec** defect.

**RULED (operator): give `me sysw pack` a stdin path**, so the command as typed
works. Precedence between stdin and `--in` must be stated, and see D below for
what stdin must do when empty.

*Classification: correction to the spec; new input path in `me`.*

### B — argv re-opens the exact hazard `mt` was built to close

`mt`'s README: *"Input comes from a file or stdin, **never an argument**: an
argument lands in shell history and in `ps` for every user on the machine, and
this material is bearer."*

`me sysw pack`'s help: *"Records, on argv. As with `seal`, argv is a PUBLIC
channel — **prefer** `--in` for anything real."*

**Prefer.** So the constellation would ship a tool that refuses transactions on
argv feeding a tool that accepts them with an advisory. `me sysw pack "tx:0100…"`
would work, and the bearer transaction lands in `/proc/<pid>/cmdline` and shell
history.

This is the *"a guard downstream of the parser has already lost"* class with the
polarity reversed: the guard is upstream and the leak is downstream.

*Classification: **REFUSAL**. A `tx:` record on argv must be refused outright.
Worse than telling the operator nothing, because nothing is exactly what argv
exposure does.*

### C — the XOR ruling has no CLI surface

The operator ruled the payload carries the raw transaction **or** its `mt1`
chunks, never both, and that **the operator picks at encode/pack time**.
`mt encode --record` has no way to say which.

*Classification: **needs a flag and a stated default**. OPEN.*

### D — fish masks an upstream failure; `mt`'s discipline already closes it

```
$ false | true
status=0          <- what the operator sees
pipestatus=1 0    <- what happened
```

So `mt … | me sysw pack` reports **`me`'s** status. But:

```
$ mt encode --in bad.hex   ->  exit 1, stdout 0 bytes
```

`mt` emits **nothing** on a failure path, so `me` sees empty input, refuses, and
the pipeline's status is non-zero after all.

*Classification: **not our concern — already safe.*** But the safety rests on
invariants no test states in those terms. **The spec must require both
explicitly**: (1) `mt` contributes nothing to stdout on any failure path;
(2) `me sysw pack` **refuses empty stdin** rather than packing an empty
container — stdin must join the existing "no records" exit-2 path, not bypass it.

### E — `mt` warns the INPUT is world-readable; nothing warns about the OUTPUT

Unprompted, `mt` said:

> `WARNING: bad.hex is mode 0644 — readable by other users on this machine.`
> `A finalized transaction is BEARER…`

The operator's command ends `> payload.bin` — shell redirection, a file `me`
never sees. **The constellation warns about the file you read from and is silent
about the file you carry to the machine.**

Operator, verbatim: **"I didn't realize `>` creates a world readable file…is
there a way to make this a refusal with command line override to permit?"**

**That confusion opened F-244** (below), which is a Critical in shipped code and
has nothing to do with transactions.

*Classification: **REFUSAL with an override**, and it is mechanically possible —
verified, not assumed: a process can `fstat(1)` its own redirected stdout and
sees `S_ISREG` + mode 0644, and sees `S_ISFIFO` for a pipe. So the check fires
exactly on a world-readable regular file and leaves pipes and terminals alone.*

**Proposed rule:**

```
stdout is a TTY           -> nothing. Nothing persists.
stdout is a pipe/FIFO     -> nothing. No file mode exists.
stdout is a regular file  |
or --out FILE             |-> mode grants group or other read?
                                -> REFUSE unless --allow-world-readable

--out additionally creates at 0600 AND fchmods an existing target to 0600
```

The `fchmod` half is load-bearing: `write_private`'s documented residual —
*"0o600 binds on CREATE"* — was **measured true**, so creating carefully is not
enough on its own.

**SCOPE RULED (operator): all of `me`, and `mt` too.** `mt`'s existing
`redirected_output_warning` fires on *any* redirection and never reads the mode,
so today it cries wolf on a 0600 file and warns no harder on a 0644 one.

### F-244 — CRITICAL, and not about transactions at all

Chasing E found that `me sysw pack` writes its container with `std::fs::write`,
mode **0644**, including an **unsealed payload holding a BIP-39 mnemonic in
cleartext**. Full entry in `FOLLOWUPS.md`. Filed `6087c1a`.

**The walk was about transactions and it found a seed exposure in shipped code.**
That is the argument for the method, in one step.

### F — seal-by-default is wrong for this payload, and contradicts a prior ruling

The operator's command carried **no passphrase flag**. Measured:

```
$ me sysw pack "text:6869" > r.bin
passphrase — write this down and store it APART from the machine:
    size letter suffer unlock honey wrist random vocal ramp defy vault govern
```

`me sysw pack` **seals by default**. Fail-closed is the right instinct — and for
this payload it produces:

1. a 12-word passphrase the operator must store apart from the machine;
2. **typing those 12 words on the SeedHammer's on-screen keyboard** to load it;
3. ~31 s of KDF (`gui/sysw_load.go:103` prices it while explaining a related
   failure);
4. a new failure mode — **lose the words, lose the transaction backup**

— to protect a payload whose entire purpose is to become **a steel plate anyone
can read**. The plate is the exposure; sealing protects nothing the engraving
does not publish.

**And it contradicts an existing operator ruling**, `SPEC_mt_qr_DEFERRED.md:189`,
2026-08-23: *"send via payload **unencrypted**."* The natural command produces
the opposite of the operator's own decision — not silently, it prints the
passphrase, but it **takes the decision**, and the cost only lands minutes later
at the device.

The awkward part: seal-by-default is **correct** for mnemonics, which is most of
what `me sysw pack` packs.

*Classification: **DEFAULT**, and it earns a change — a lost-passphrase failure
mode on a durability backup is worse than saying nothing.*

**RULED (operator): default by CONTENT.** `me sysw pack` seals when the payload
holds any `Class::IsSecret()` record and does not when it holds none. The
predicate already exists (`sysw/record.go:37`) and already names
`ClassMnemonic`, `ClassCodex32Secret`, `ClassPassphrase`. **It must say which way
it went, and why, on stderr, every time** — a content-dependent default that is
silent is a worse defect than the one it replaces.

---

## Running classification tally

| # | finding | class | status |
| --- | --- | --- | --- |
| A | pipeline does not compose | spec correction + new stdin path | RULED |
| B | `tx:` record on argv | refusal | RULED |
| C | XOR has no CLI surface | flag + default | **OPEN** |
| D | pipeline safety rests on unstated invariants | two spec requirements | RULED |
| E | world-readable output | refusal + override, all of `me` and `mt` | RULED |
| F-244 | container written 0644 with a cleartext mnemonic | **CRITICAL** | FILED |
| F | seal-by-default wrong here | default by content | RULED |

**Step 2 onward: not yet walked.** The operator has a container and has not yet
got it to the device.
