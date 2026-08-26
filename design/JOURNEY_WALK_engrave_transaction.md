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

## Step 2 — getting it to the device

The operator asked: **"So the device was in bootsel and payload was flashed and
sh2 is now off?"** — a question about machine STATE, and the uncertainty is the
finding again.

Ground truth: after `picotool load` the SH2 is **still in BOOTSEL** (no reboot
without `-x` or `picotool reboot`), so it is not "off" — it is in the ROM
bootloader running no firmware.

The command is documented, `SPEC_systemwide_payloads.md:869`:

```
picotool load --verify -t bin -o 0x10D00000 payload.bin   # machine in BOOTSEL, laptop power
```

### G — the documented command's power note is right for the flash and silent about the step after it

The fork carries upstream `713aee2`, *"reboot into USB drive mode if 20V/28V is
not available… blocking the user from operating a machine that won't engrave"*.
Verified an ancestor of the fork's `main`:

```
$ git merge-base --is-ancestor 713aee2 main  ->  YES
```

So **rebooting on laptop power brings the SH2 up in USB drive mode, not the
GUI.** The operator never sees *"A systemwide payload is present. Load it?"*, and
the honest conclusion they draw is **"the payload didn't take"** — so they
re-flash a payload that was already written correctly.

The real sequence needs a step no document states: flash on laptop power →
**move to the machine's own 20V/28V supply** → then boot.

*Classification: **documentation only**, and it earns the change — the wrong
outcome is re-flashing a correct payload, and the operator has no way to tell
that from a failed write.*

### H — writing the region destroys the standing payload. RULED: that is intended.

`me sysw pack --region` emits a full 65,536-byte image and
`picotool load -o 0x10D00000` writes all of it, so a transaction payload
**replaces the entire region** — including any seeds, descriptors or passphrase
already there. No tool says so: not `me`, not `picotool`, not the device. There
is no merge; the region holds exactly one container.

**RULED (operator) 2026-08-24: "Overwriting is the desired default behavior."**

This is a ruling against all three options offered (NFC-preferred, warn-and-keep,
read-and-merge), and it is coherent once the model is stated:

> **The systemwide region is a COURIER, not a vault.** It carries what the
> operator is working on to the machine, one job at a time. It was never the
> durable copy of anything, so replacing it loses nothing that was not already
> backed up elsewhere.

*Classification: **not a defect — design intent.*** One cheap residual remains,
**documentation only**: the courier model should be **written down**, because
nothing currently says it and an operator who has not been told will reasonably
read a persistent flash region as storage. The failure that model prevents is not
the overwrite — it is someone treating the region as a backup in the first place.

---

## Step 3 — the compare screen

The device shows the identity digest and asks the operator to compare it
(`gui/sysw_load.go:168`, *"Compare this against what"*). That number was printed
by `me sysw pack` to **stderr, in a terminal, possibly an hour ago**.

Operator, verbatim: **"I see that I was supposed to compare and I go back to pc
and run the pack command again."**

### A CORRECTION TO THIS WALK'S OWN METHOD, recorded because it nearly became a finding

The first reproducibility test packed **the same record twice** and reported
three identical digests as evidence the digest "does not discriminate". That was
a **test artifact** — same input, same output, correct behaviour. Re-run with
three different records it discriminates cleanly:

| record | unsealed | sealed |
| --- | --- | --- |
| `text:6869` | `c679 6b68 …` | `c679 6b68 …` |
| `text:776f726c64` | `dc3f 66f0 …` | `dc3f 66f0 …` |
| `text:646966666572656e74` | `2c65 64b8 …` | `2c65 64b8 …` |

*A control that varies nothing measures nothing.* The alarm was mine, not the
tool's.

**What the corrected measurement establishes**, and it is good news for the
operator's move: the digest is over the RECORDS, so it is **reproducible**,
**identical for sealed and unsealed**, and **independent of the passphrase**
(two different generated passphrases, same digest). Re-running `pack` does give
back the same number.

### I — the operator reached for the risky recovery because the safe one is invisible

**`me sysw show` exists** — *"Print what a container holds, and its digest"* —
and is read-only:

```
$ me sysw show w.bin
sealed:   false
pub_len:  9
ct_len:   0
digest:   c679 6b68 b993 bc10 793a 3de2 8b3d 46e0    <- identical to pack's
```

The operator did not reach for it. They reached for **re-packing**, which works
for the digest and carries a trap on the sealed path: **it prints a brand-new
12-word passphrase every time.** That reads as *"this is a different container"*,
so an operator can reasonably conclude they have just invalidated what is on the
device. They have not — but if they act on that belief and **re-flash** the fresh
container, the words they wrote down earlier are now **wrong**, and the payload on
the device opens only with a passphrase they saw once and did not keep.

Re-packing to recover the digest is safe **only if you do not re-flash**, and
nothing says so at the moment an operator would do it.

**The screen that creates the need never names the command.** *"Compare this
against what"* — against *what*, obtained *how*, is exactly the question the
operator asked, and the device is silent on both.

*Classification: **affordance + documentation**, and it earns the change — the
wrong outcome is a payload whose passphrase the operator no longer holds.*

**RULED (operator) 2026-08-24:** *"Sh2 could put `me sysw show` under the digest
to nudge user to execute the correct command on host."*

So the device prints the command **beneath the digit groups**, on the screen that
creates the need. This is better than documenting it, for the reason the walk
found it in the first place: the operator is **standing at the machine**, and a
manual they would have to go and open is exactly what they cannot reach. It also
steers them off the re-pack path before they take it, rather than warning them
afterwards.

`me sysw pack`'s digest line should carry the same pointer, so the host says it
too.

### J — the digest does not distinguish sealed from unsealed

Identical for both, by construction. With content-based sealing (finding F) the
seal state now depends on payload content the operator may not be tracking, so
the compared number cannot tell them which they hold. `me sysw show` prints
`sealed:` and answers it.

*Classification: **not our concern** — the affordance in I covers it.*

---

## Step 4 — the confirm screen (the unbuilt part)

**Display is 240x240** (`gui/gui_test.go:383`). Drawn honestly, §3.3's confirm
screen is **already at the limit for a 1-in/1-out transaction with no change** —
and the moment there is change, which is the normal case, it does not fit.

Operator, unprompted: **"I want to scroll to see all outputs or show the full
txid so I can compare to host."**

Both asks are right, and together they force a **two-screen split** that resolves
the squeeze — because the screen was trying to do two different jobs at once:

| screen | job | contents |
| --- | --- | --- |
| 1 | **where the money goes** | outputs (paged), amounts, network params, IN count, FEE, locktime |
| 2 | **which transaction** | the **full 64-hex txid**, in 16 groups of 4 — the same shape as the identity digest, for the same reason. `ENGRAVE` lives here and nowhere else |

### K — the txid must be shown for RECOGNITION and must never be claimed as proof

`mt` reports the **txid** (`SPEC_mt_v0_1.md:680`), so both ends name the same
number and the operator's comparison works. But the same line, and a correction
`mt` already paid for, bind the device:

> the txid is blind to the **entire witness region**, which is where the
> signatures live and which is the bulk of every artifact. Damage there
> re-derives the expected id and passes — **not improbably, but always**.

`mt verify`'s own report says it in operator language: *"this check identifies
the transaction. It does NOT prove every byte."*

**The device must carry the same limit on the same screen.** A txid match means
*which transaction*, never *the bytes are right*.

**And byte integrity is already covered elsewhere, which is why this division is
clean:** in transit the record set is protected by the identity digest the
operator compared at load (unsealed) or by AEAD (sealed). The txid's job is
operator recognition — *"yes, that is the one I built"*. Two checks, two jobs,
neither claiming the other's.

*Classification: **refusal to overclaim** — a wording constraint that is
normative, not cosmetic. `mt` withdrew this exact claim after making it.*

### L — must every output be seen before ENGRAVE? RULED: no.

**RULED (operator) 2026-08-24: "Show a total, allow skip."** Page 1 shows the
first output plus a total, and `ENGRAVE` is reachable immediately.

**The argument for it, which is stronger than the one against:** a forced
page-through that operators learn to tap past is **worse** than an honest
summary, because it manufactures the appearance of review. Verification happens
on the host with `mt inspect`; the device is a second look, not the primary
check. Forcing a re-read of what was already checked is how tapping-through
becomes a habit, and the habit then applies to the screen that mattered.

**ACCEPTED RESIDUAL, stated plainly because it is real:** an operator can reach
`ENGRAVE` having seen one destination address of N.

### M — the TOTAL must not be spelled as "the amount you are sending"

**This is a known defect class in this constellation, not a hypothetical.** One
of the five near-miss failures of the `mt` cycle was exactly it:

> print the plate legend -> **printed the sum of all outputs as the destination
> amount**, for steel.

**Change outputs are yours.** A total that reads as *"you are sending X"*
overstates it whenever there is change — the normal case. So the summary line
ruled in L must state what it is counting:

- `N outputs, X.XXXXXXXX BTC total` -- a neutral sum, not a destination claim
- and it MUST NOT be labelled `TO`, `SENDING`, or any word implying a recipient

`TO` remains an **asserted** field (§3.3), taken from the operator's own
`--to-label`, never derived from output values. The two must not be adjacent in a
way that lets one read as the other's value.

*Classification: **refusal to overclaim**, same class as K. Earns a change: the
wrong outcome is an operator reading their own change back as money leaving.*

---

## Step 5 — after the cut

The `mt` cycle's Critical was found at exactly this shape: not a wrong thing in a
section, but a **silent step**. `mt verify` never reported how much of its
`t = 4` budget a plate had consumed, so a plate miscut four times passed as OK
while one scratch from unrecoverable. Nothing was wrong; a step said nothing.

**RULED (operator) 2026-08-24: "Device says to test the plate now."**

### N — THE DEVICE HAS NO CAMERA. It can never read back its own QR.

Checked before writing the ruling into the spec, because *"test the plate"* is an
**affordance** and an affordance needs a mechanism.

`driver/` holds `ap33772s` (USB-PD), `clrc663` and `st25r3916` (**NFC readers**),
`ft6x36` (touch), `ili9488` (display), `tmc2209` (steppers), `mjolnir2` (the
machine), plus `dma`, `pio`, `otp`. **No image sensor.** And `scanner.Scan` has
exactly one feed — `gui/nfc_scan.go:62`, the NFC poller.

The only `camera` strings in the tree are a vestigial `cameraTheme` **colour**
theme (`gui/theme.go:62`), a comment at `gui/derive_xpub.go:197`, and the BIP-39
word *camera* in three wordlists. (The comment is a small instance of a mechanism
outliving its hardware; not worth its own entry, worth not being misled by.)

**The operator's wording was already the implementable one — the device SAYS to
test, it does not test.** Three consequences bind the spec:

1. **The device can never verify its own QR output.** It writes a symbol it has
   no way to read. Plate testing is necessarily the operator's, with an external
   scanner.
2. **It retroactively validates today's FIRST ruling.** "Comprehend + read back"
   was offered and rejected in favour of "comprehend, then cut". Read-back was
   never possible on this hardware — the rejected option was unbuildable, and
   nobody knew that at the time.
3. **It changes F-243's test plate.** The optical test uses an external scanner
   by necessity, and its result can never be checked by the machine that cut it.
   *"Which encoding survives engraving"* is permanently a host-side,
   human-in-the-loop measurement.

*Classification: **not a defect — a hardware fact that removes an option and
constrains a ruling.** It earns a spec change: §4 must state that no on-device
read-back exists, so no future phase plans one.*

**What the device must SAY, per the ruling** — and the `mt` Critical is the
argument for saying it at all rather than dropping the operator back at the
carousel with two pieces of steel and no statement:

```
PLATE 1 OF 2 — CUT
────────────────────────
TEST IT NOW, before you
leave the machine.

Scan the QR. It must read
back as the same transaction.
────────────────────────
This machine has no camera.
It cannot check its own work.
────────────────────────
NEXT PLATE          DONE
```

The last block is the anti-overclaim discipline again: it tells the operator
**why** the job is theirs, which is what makes them do it.

**RULED (operator): the test is "scan, then `mt inspect`."** A host round trip,
and that is the right cost — the plate is still in the machine, which is the
cheapest possible moment to re-cut it.

### O — NO `mt` VERB CAN READ A DEFAULT PLATE

Measured, because the ruling depends on it:

```
$ mt inspect --help
  --in <PATH>   Read the STRINGS from a file. Defaults to stdin
```

`mt inspect`, `mt verify` and `mt decode` **all take `mt1` strings**. The default
plate, by the operator's own step-4 ruling and by F-234, carries **raw
transaction bytes in a QR**.

> **The tool that defines the format cannot read its own default artifact.**

For **broadcasting** this is F-234 working exactly as designed — raw bytes go
straight into `bitcoin-cli sendrawtransaction` and no constellation knowledge is
needed. The gap is in **inspection**: the step just ruled mandatory has no input
path today.

**So the ruling requires `mt inspect` to gain a raw-transaction subject**, so one
verb reads both representations: `mt1` strings from text plates, raw bytes from
QR plates. This is new scope on `mt`, owned by Goal 1's P2.

*Classification: **missing capability**, and it earns the change — a mandatory
step with no tool behind it is the "affordance without a mechanism" shape, caught
twice in one walk (see N).*

*Minor, in passing: `--transaction <PATH>` is advertised in `mt inspect --help`
while its own text says "**`verify` only**" — a flag offered by a verb that does
not honour it.*

### The encoding parameter is far more urgent than F-243 says

F-243 filed the QR encoding as *"can a stranger read this plate in 15 years"*.
The step-5 ruling makes it **"can the operator complete a mandatory step, today,
every single time"**:

- **raw octets** — a phone scanner hands back bytes >= `0x80` that most apps
  render as mojibake or refuse. **The operator's test appears to fail on a good
  plate.**
- **base45 / bech32-uppercase** — pure alphanumeric, every scanner shows clean
  text, but the operator cannot tell it is the RIGHT transaction without a tool
  (which is why the ruling routes through `mt inspect`).

**F-243's test plate is therefore a gate on whether the ruled workflow functions
at all, not only on long-term recoverability.** Its priority should be read that
way.

---

## Finding C, resolved — the XOR form, and what it costs v1

**RULED (operator) 2026-08-24: no default. `mt encode --record` refuses without
`--raw` or `--chunks`.**

> **SUPERSEDED 2026-08-25 (operator, mid side-by-side walk).** The ruling below
> stands as the record of what was decided and why — it is not edited. What
> changed is its *subject*: `--chunks` was measured a **byte-for-byte no-op**,
> identical to bare `mt encode` on stdout and stderr, because the same day's
> bare-records ruling deleted the `tx:` wrapper that had been its only reason to
> exist. A choice with one side is not a choice, so the pair collapsed to a
> single **`--qr`** and the teaching refusal was retired with it. The concern
> that produced this ruling — *a flag required on every invocation gets aliased
> away* — is answered better by the collapse than by the refusal: there is now
> nothing to alias, because the common path takes no flag at all. See
> `SPEC_engrave_transaction.md` §2.2 and `mnemonic-transaction` `1282260`.

The concern raised and overruled: a flag required on every invocation is the
friction that gets aliased away, and the alias re-hides the choice — which is
exactly how finding F happened. **Mitigation adopted: make the refusal TEACH.**
A refusal that states what each choice costs is far less likely to be aliased
away than one that merely blocks:

```
mt: --record needs a form. Say which:

  --raw      the transaction's bytes. QR plates only.
             The device needs no mt1 decoder.

  --chunks   mt1 strings. Text plates.
             The device engraves them verbatim.
```

It belongs on **`mt encode`**, not `me sysw pack`: `mt` owns the transaction and
the chunking, and `me` treats the record body as opaque by design.

### The scope consequence, and the operator's answer to it

With no default an operator may pass `--chunks` on day one, so the device must
handle it — which appeared to put the `mt1` decoder back on v1's critical path.

**RULED (operator): "if chunks passed, chunks get engraved without inspection or
confirmation. If raw passed, inspection offered on device."**

This removes the decoder from v1 entirely, because **chunks are engraved verbatim
as TEXT** — the existing `mdmkText` / `validateMdmk` path the device already uses
for `md1`/`mk1` cards. The device never needs to understand them.

| payload | device does | needs |
| --- | --- | --- |
| **raw** | parse -> comprehend -> confirm -> **QR plates** | tx parser only |
| **chunks** | engrave verbatim -> **text plates** | nothing new |

**Three things this pins:**

1. **It is a deliberate EXCEPTION to the first ruling of the day**
   ("comprehend, then cut"), and the spec must say so **in those words**.
   Otherwise a future reader finds the inconsistency and "fixes" it. The
   justification is that comprehension **moved upstream** — `mt encode` built
   those chunks from a transaction the operator inspected on the host — not that
   it was dropped.
2. **Chunks means text plates ONLY, no QR.** F-234 forbids an `mt1` string as QR
   content, and with no decoder the device cannot turn chunks back into bytes.
   Each form therefore produces exactly one kind of plate, which is simpler than
   the matrix the brainstorm assumed.
3. **ACCEPTED COST, stated plainly:** a chunks plate is cut with the device
   making **no claim whatever** about its content — no destination, no amount, no
   txid. The operator's only verification is the one they did on the host.

### PROPOSED refinement — per-chunk checksum, no decoder

Without a decoder the device cannot distinguish a valid chunk set from garbage,
so a corrupted payload becomes ~21 minutes per plate of scrap, and the plate
looks right.

**The fork already carries the codex32 BCH engine** (`codex32/gf32.go`,
`gf1024.go`, `checksum.go`, `correct.go`) and already exposes validity predicates
for the sibling formats (`codex32.ValidMD`, `ValidMK`). So the device can verify
**each chunk's own BCH checksum** without reassembling anything — far less than a
decoder, and it catches garbage before the machine moves.

*Status: proposed, not ruled.*

---

## Thread 3 — entry divergence: the wrong payload, the right program

**What the operator has:** a loaded session holding mnemonics and descriptors,
and a menu entry promising something they are not.

**RULED (operator) 2026-08-24:** *"Loading payload should only offer programs
that apply to the payload. If no transaction is present in the payload then
engrave transaction should not be an option."*

### P — the carousel is the WRONG PLACE for applicability. RULED, after two retractions.

This finding went through three forms in one exchange, and the third is the
right one. Recording all three, because the reasoning is the value.

**Form 1 (operator):** *"If no transaction is present in the payload then engrave
transaction should not be an option."* — i.e. hide it.

**Measured, and expensive.** The carousel is a contiguous integer range with ONE
conditional endpoint, and that endpoint is already spent on `unlockPayload`:

```go
npages := int(lastNav) + 1        // layoutMainPager:2445
for i := range npages { ... }     // one dot per index, 0..lastNav
if m.prog > m.lastNav() { ... }   // wrap is a BOUND, not a set
```

The enum comment shows the codebase analysed this exact case and avoided it:

> *"inserted mid-enum, conditional visibility means the carousel must **SKIP an
> interior index in both wrap directions**, and `layoutMainPager` fills dot
> `int(page)`, which would then point at the wrong dot."*

So form 1 costs a bound-to-set rewrite and retires the compile-time guard — plus
**carousel positions would shift with payload content**, on a device where a
wrong selection costs ~21 minutes of steel.

**Form 2:** shown but dimmed. Cheaper, positions stable. **RETRACTED by the
operator.**

**Form 3 — RULED, and it dissolves the problem instead of solving it:**

> *"All carousel items shown always because eventually user can use any program
> to start nfc transfer. Maybe we need an 'on payload load' program than runs
> when payload exists and has a menu that offers user only the applicable
> operations like engrave transaction or wallet descriptor or key or seed."*

**The operator supplied a constraint I did not have, and it invalidates the whole
line of reasoning:** every program may eventually start an NFC transfer, so
**payload-independence of the carousel is CORRECT, not a limitation.** I had been
treating the contiguous range as an obstacle. It is the right shape.

**The two screens answer two different questions:**

| | question | content-dependent? |
| --- | --- | --- |
| **the carousel** | what can this MACHINE do? | **never.** All eleven, always |
| **the payload menu** | what can THIS PAYLOAD do? | **by construction** |

**Cost: near zero, and lower than both earlier forms.** Nothing in the carousel
changes — `lastNav`, the compile-time guard, `layoutMainPager` and every wrap
site are untouched. `syswPayloadMenu` (`gui/sysw_unload.go:34`) already exists
and today offers only `LOAD AGAIN` / `UNLOAD`; it gains content-derived entries
above them. And the classification is already computable — `sysw.Classify` returns
`ClassMnemonic`, `ClassDescriptor`, `ClassMDMK`, `ClassPassphrase`, `ClassFreeText`,
`ClassAddress`.

**The original walk question dissolves too.** The operator asked what happens when
you pick *Engrave Transaction* with a seed payload loaded. Under form 3 they are
far less likely to be there at all: after loading, the device shows what the
payload can do. The carousel entry remains reachable and must still refuse
gracefully — but it stops being the primary path, so the refusal is a backstop
rather than the design.

**Message discipline still applies** (finding I): a refusal must name the FIX, not
just the problem — *"this payload holds no transaction — load one with Load
Payload."*

**FOLLOW-THROUGH — RULED (operator): the menu appears RIGHT AFTER A SUCCESSFUL
LOAD.** Today `syswLoadFlow` loads and returns to the carousel. The flow becomes:

```
boot -> "payload present, load it?" -> LOAD -> compare digest
     -> THE PAYLOAD MENU  ("this payload holds: a transaction, 2 seeds")
     -> BACK exits to the carousel
```

This is the plain reading of *"a program that runs when payload exists"*, and it
is what would have prevented the divergence this thread was built to explore: the
operator never has to guess which program applies, because the device just told
them. **The carousel entry stays** — reachable, and refusing gracefully — but it
becomes the backstop rather than the path.

**Accepted cost:** one screen between loading and normal use, on every boot with
a payload present. `BACK` is the exit and must be, for the same reason
`syswUnloadFlow`'s BACK is choice 0: the resting position is the one that costs
nothing.

## Thread 3, case 2 — the payload holds several transactions

Under P's design the payload menu reports *"this payload holds: 3 transactions"*,
so **Engrave Transaction needs a picker**. Four candidates were offered for what
distinguishes entries, and the tension is findings K and M again — **the derived
fields are unique but unreadable, and the readable fields are asserted and may
collide**:

| candidate | unique? | readable? |
| --- | --- | --- |
| txid | **yes, derived** | no |
| `TO` label | no — asserted, three could all read "cold storage" | yes |
| amount | no — two can share one | yes |
| position ("1 of 3") | yes | honest and useless |

### Q — the picker is keyed on the TXID, and the prefix distinguishes without verifying

**RULED (operator) 2026-08-24: txid.**

Consistent with K: the txid is the derived, collision-free identifier, and it is
the one the operator can match against `mt inspect` on the host.

**Two constraints follow, both from discipline this cycle already paid for.**

1. **The picker shows a PREFIX; only the confirm screen shows the full txid.**
   Three 64-hex txids do not fit 240x240. But `mt`'s own help already names where
   truncation turns dangerous:

   > *"Comparing against the 20-bit set id would report a match for any
   > transaction sharing those bits — 1 in 1,048,576 by accident, and **under a
   > second to construct deliberately**."*

   So **the prefix distinguishes, it never verifies.** It separates transactions
   inside a payload the operator packed themselves; the full txid on screen 2 is
   what gets compared. **The spec must say this in those words**, or a later
   reader treats the picker as the check — the same overclaim K and M exist to
   prevent, in a third place.

2. **Two identical txids in one payload is the same transaction packed twice** —
   a duplicate to refuse or collapse, not a picker entry to disambiguate. A
   picker that renders it twice invites the operator to believe there are two
   artifacts.

**The `TO` label is not discarded** — it may ride as a second line, since it is
what an operator actually recognises. But it is **asserted** (§3.3) and must be
rendered as such, never as the entry's identity.

## Thread 2 — stopping mid-cut

**What the operator has:** a plate 20 minutes into a 21-minute cut, and either a
finger on BACK or a power drop.

**The two plate forms fail differently, and only one fails safe in the artifact:**

- **a partial QR** will not scan — Reed-Solomon fails and the finder patterns may
  not even be cut. **Fails safe in the steel itself.**
- **a partial text plate** carries chunks 1-15 of 22 and **every one has a valid
  BCH checksum**. It looks like a real plate. It fails safe only because `mt`
  requires chunks `1..count` to be present — **the safety lives in the host tool,
  not in the artifact.** Worth stating, because an operator holding the steel has
  no way to see the difference.

### R — the legend is cut LAST, and an incomplete plate is discarded

**RULED (operator) 2026-08-24: "Legend should come last. Incomplete plate is
discarded."**

**Why legend-last is more than an ordering choice.** The legend is the plate's
**claim about itself**. Cut last, a plate only claims to be `PLATE 2 OF 3` once
it actually is one; cut first, it is a claim the plate has not earned — a partial
plate asserting completeness. This is the anti-overclaim discipline of K, M and Q
applied to the **artifact** rather than to a screen.

**The invariant it buys, and it is worth writing into the spec explicitly:**

> **An unsigned plate is an unfinished plate.**

Visible at a glance, in a drawer, with no tooling and no scanner — which matters
precisely because the device has no camera (N) and the operator is the only
inspector there is.

**Why no RESUME, with a mechanism behind it rather than a preference.**
Re-clamping cannot guarantee the plate returns to the same origin, and this
machine has already produced a misregistration artefact traced to **Y-axis play
from a loose screw** — found only after four software hypotheses had failed. A
resumed cut would be offset against the first half, and the result would look
like a finished plate. Discard is the sound rule, not merely the simple one.

**The device must SAY it**, on the same principle as the step-5 ruling: it knows
it stopped mid-cut, and the operator is holding twenty minutes of steel they will
be tempted to keep. A plate with no legend is self-evidently unfinished, but only
to someone who has been told the invariant.

*Classification: **default (ordering) + instruction**. Earns the change: the
wrong outcome is an anonymous half-plate entering a stack of good ones.*

---

## Running classification tally

| # | finding | class | status |
| --- | --- | --- | --- |
| A | pipeline does not compose | spec correction + new stdin path | RULED |
| B | `tx:` record on argv | refusal | RULED |
| C | XOR has no CLI surface | **no default — a teaching refusal**; chunks engrave verbatim, no decoder in v1 | RULED |
| D | pipeline safety rests on unstated invariants | two spec requirements | RULED |
| E | world-readable output | refusal + override, all of `me` and `mt` | RULED |
| F-244 | container written 0644 with a cleartext mnemonic | **CRITICAL** | FILED |
| F | seal-by-default wrong here | default by content | RULED |
| G | power note silent about the step after the flash | documentation only | RULED |
| H | region write destroys the standing payload | **design intent** — write the courier model down | RULED |
| I | compare screen names no command; re-pack is the risky recovery | affordance + documentation | RULED |
| J | digest does not distinguish sealed from unsealed | not our concern | CLOSED |
| K | txid shown for recognition, never claimed as proof | refusal to overclaim | RULED |
| L | must every output be seen before ENGRAVE | **no** — total + skip, residual accepted | RULED |
| M | the total must not read as a destination amount | refusal to overclaim | RULED |
| N | **the device has no camera** — no on-device read-back, ever | hardware fact; spec must state it | RULED |
| O | **no `mt` verb can read a default plate** — all three take `mt1` strings | missing capability, new P2 scope | RULED |
| P | applicability belongs in the PAYLOAD MENU, not the carousel — the carousel stays payload-independent | RULED (3rd form) |
| Q | several transactions in one payload — picker keyed on **txid**; prefix distinguishes, never verifies | RULED |
| R | stopping mid-cut — **legend last**, incomplete plate discarded, no resume | RULED |

**Step 4 onward: not yet walked.** The payload is loaded into the session; the
operator has not yet reached the Engrave Transaction program itself.
