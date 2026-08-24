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
| G | power note silent about the step after the flash | documentation only | RULED |
| H | region write destroys the standing payload | **design intent** — write the courier model down | RULED |
| I | compare screen names no command; re-pack is the risky recovery | affordance + documentation | RULED |
| J | digest does not distinguish sealed from unsealed | not our concern | CLOSED |
| K | txid shown for recognition, never claimed as proof | refusal to overclaim | RULED |
| L | must every output be seen before ENGRAVE | **no** — total + skip, residual accepted | RULED |
| M | the total must not read as a destination amount | refusal to overclaim | RULED |
| N | **the device has no camera** — no on-device read-back, ever | hardware fact; spec must state it | RULED |
| O | **no `mt` verb can read a default plate** — all three take `mt1` strings | missing capability, new P2 scope | RULED |

**Step 4 onward: not yet walked.** The payload is loaded into the session; the
operator has not yet reached the Engrave Transaction program itself.
