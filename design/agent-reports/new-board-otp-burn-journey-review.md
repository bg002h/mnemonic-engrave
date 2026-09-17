# Failure-states / journey review — OTP burn on the SECOND SeedHammer II

VERDICT: GO

Critical: 0 / Important: 6 / Minor: 2

**Scope.** One question only: at each of the five steps, what ELSE could happen, and
does the operator get told or does the step go silent? Read-only throughout — no
command in this review touched OTP or wrote to any device. Authorities used:
picotool 2.2.0 source (`main.cpp`, `picoboot_connection/picoboot_connection.c`,
fetched from `raspberrypi/picotool` tag `2.2.0`), the installed binary
`/nix/store/j7pl045ik6yb73zvq3n9a52j85d2qnig-picotool-2.2.0-a4/bin/picotool`
(offline `otp list` / `help` only), `scripts/pico2-bootkey-rehearsal.sh`,
`design/RUNBOOK_custom_boot_key.md`, `~/.local/bin/sh/sh2-flash`, and live sysfs.

**Live state observed while writing this (no device commands issued):**

```
/sys/bus/usb/devices/1-10   idVendor 2e8a  idProduct 000f
                            serial   09F50BF63E8D6F46
                            devpath  10   (root-hub port, NOT behind a hub)
                            speed    12   bMaxPower 50mA
sh2-state/sh2-chipid.txt            = 6f463e8d0bf609f5      (the NEW board)
sh2-state/board-f55c45ab83b777c4.chipid = f55c45ab83b777c4  (the OLD board, archived by hand)
~/.sh2/otp-6f463e8d0bf609f5.json    byte-identical to ~/.sh2/my-otp.json  (cmp)
rehearsal-work/{factory,my,third-party}-key.pem  all present
```

---

## FINDINGS

### I-1 — Steps 1 and 3: both irreversible commands print only PRE-write information, and the runbook describes step 1's output as absent when it is not

**Moment.** The operator has just typed
`picotool otp load ~/.sh2/otp-6f463e8d0bf609f5.json` and is watching the terminal.

**What actually happens.** The JSON path of `otp_load_command::execute`
(`main.cpp:7838-7862`) calls `process_otp_json`, which for an array value prints the
row key and then every byte —

```
bootkey1:
  84, 6a, a2, 89, f2, f3, 17, e5, ... d6, 4c, ab, b4,
```

— at `main.cpp:7489-7497`, and only **afterwards** calls `write_func` (the final line
of `process_otp_json`). The function then hits
`// Return now, don't do rest of function` / `return false;` (`main.cpp:7858-7860`),
so the read-back loop and its `"  Verified OK"` / `"  Mismatch at row ..."` lines
(`main.cpp:7887-7901`) **never execute for a JSON file**, even though the command's
own help says *"Load the row range stored in a file into OTP **and verify**"*.

Step 3 has the same shape. `otp_set_command::execute` prints
`ROW 0x004b  OLD_VALUE=0x000001: OTP_DATA_BOOT_FLAGS1` and
`field KEY_VALID (bits 0-3)` (`main.cpp:8021-8030, 8076-8090`), then writes with no
NEW_VALUE line and no verify.

**What goes silent.** The only output either irreversible command produces is the
input echo or the pre-write state. Neither can distinguish "wrote 16 rows" from
"wrote 0 rows". The runbook primes the operator for the wrong signal: it says
*"`otp load` prints no 'verified' confirmation of its own — **the absence of output
is not success**"*. There will not be an absence of output; there will be a
confident 32-byte hex dump that looks exactly like a result. An operator who
memorised "silence means unverified" and then sees a dump can reasonably conclude
the opposite and skip the gate.

**Classification: DOCUMENTATION ONLY (runbook text) — but outcome-changing.**

**Minimal remedy.** In step 2's box, replace *"prints no 'verified' confirmation …
the absence of output is not success"* with: *"it echoes `bootkey1:` and the 32
bytes it is about to write **before** it writes them, and for a JSON file it skips
the read-back its own help advertises — so that dump is your input, not a
readback."* Add one sentence to step 3 saying `otp set` likewise prints only
`OLD_VALUE=`, never the value it wrote.

**Why it earns a change.** The wrong outcome is skipping step 2 or step 4, and step
4 is the only thing between a wrong `KEY_VALID` and a permanently spent slot.

---

### I-2 — Steps 1 and 3: the irreversible commands carry NO board-identity binding, while every read-only gate does

**Moment.** Step 1 and step 3, with two SeedHammer IIs on the bench for the first
time.

**What goes silent.** `--sh2-precheck`, `--sh2-verify-slot` and `--sh2-verify-valid`
all pin the physical board through `sh2_require_seedhammer`
(`scripts/pico2-bootkey-rehearsal.sh:585-613`) and refuse a device whose CHIPID does
not match `sh2-state/sh2-chipid.txt`. The two commands that actually burn OTP are
bare `picotool` lines with no such binding — they target whatever single RP2350 is
in BOOTSEL. The pin guards the steps that cannot hurt you and not the steps that can.

Today's exposure is narrow but real:

- Two SH2s cannot both be targeted — picotool fails with `"Command requires a single
  RP-series device to be targeted."` (`main.cpp:8692-8696`). Refusal. Good.
- Step 1 aimed at the OLD board is refused by the bootrom: its slot-1 ECC rows are
  already programmed, so `PICOBOOT_UNSUPPORTED_MODIFICATION` → `"Attempted to modify
  OTP ECC row(s)"` (`main.cpp:7389-7395`). Refusal. Good.
- **Step 3 aimed at the OLD board succeeds silently as a no-op.** `otp set -s ... 0x2`
  ORs 0x2 into the old board's existing 0x3, clears nothing, and the bootrom accepts
  it. Identical output to the real thing. The operator believes step 3 is done.

**Classification: REFUSAL.**

**Minimal remedy, available now at zero cost.** The RP2350 bootrom exposes the chipid
as its USB serial number. Verified live: `/sys/bus/usb/devices/1-10/serial` reads
`09F50BF63E8D6F46`, which is the brief's `0x09f50bf63e8d6f46`. So bind both
irreversible commands to the board:

```sh
picotool otp load --ser 09F50BF63E8D6F46 ~/.sh2/otp-6f463e8d0bf609f5.json
picotool otp set  --ser 09F50BF63E8D6F46 -s BOOT_FLAGS1.KEY_VALID 0x2
```

**The case matters.** picotool compares with `strcmp` against the USB string
descriptor (`picoboot_connection/picoboot_connection.c:132-136`), so the uppercase
form above is required. A wrong serial matches zero devices and the command refuses
(`"…accessible RP2350 devices in BOOTSEL mode were found found with serial number
%s."`, `main.cpp:4006-4007`) — it can never become a wrong-board write. The only way
this remedy can fail is toward a stall.

---

### I-3 — Step 4: when `KEY_VALID` comes back with a WRONG bit, the gate tells the operator that `otp set -s` cannot help, when it is exactly what is needed

**Moment.** The operator typed `0x4` (or `0x8`) instead of `0x2` at step 3 — the
runbook puts all three values on one line as `# slot 1 (slot 2 = 0x4, slot 3 = 0x8)` —
and is now running `--sh2-verify-valid 1`.

**What happens.** `KEY_VALID` is `0x5`. The script computes `WANT=3`,
`EXTRA = GOT & ~WANT = 4`, `MISSING = WANT & ~GOT = 2`
(`scripts/pico2-bootkey-rehearsal.sh:775-778`), and because `EXTRA != 0` it takes the
first branch and dies with:

> *"It has bits set that should NOT be: 0x4. This is NOT an interrupted write, and
> re-running `otp set -s` cannot fix it — that only ever SETS bits. Some other slot
> has been marked valid. Stop and work out which, and why, before doing anything
> else."* (lines 779-784)

`MISSING` is computed and then never printed — that branch `die`s before reaching
line 785. So the operator is told the extra bit is unfixable and is **not** told that
slot 1's own bit is still missing, nor that setting it is trivial and safe.

**What goes wrong next.** The advice is true of the extra bit and false of the state
as a whole. Two plausible continuations are both bad:

1. Believing `otp set` is useless, the operator burns a key hash into slot 2 — which
   `0x4` has just marked **valid**. That write lands in an already-valid slot with no
   gate in front of it, inverting the whole verify-then-validate structure.
2. The operator proceeds to step 5, gets a dark screen, and is told by step 6's box
   *"do NOT burn another slot — this is a signing or image problem"*. That box is
   correct only when step 3 succeeded. Here it sends them to re-sign forever.

**Classification: WARNING.**

**Minimal remedy.** In the `EXTRA != 0` branch, print `MISSING` too and split the
verdict: *"the extra bit 0x<EXTRA> is permanent and has spent slot N; the missing bit
0x<MISSING> is still fixable — run `picotool otp set -s BOOT_FLAGS1.KEY_VALID
0x<MISSING>` and re-run this check."*

---

### I-4 — Step 5 as planned uses bare `picotool load`, bypassing the one check that separates a bad image from a PD failure

**Moment.** Step 5, holding `seedhammerii-v0.0.0-bgf5b068f.signed.uf2`.

**What goes silent.** The plan is `picotool load --verify <signed.uf2>; picotool
reboot`. `sh2-flash` exists precisely to stop this: given a path already ending
`.signed.uf2` it skips re-signing (`sh2-flash:266-272`), then runs
`picotool info -a "$SIGNED" | grep -q 'signature:.*verified'` and **refuses to flash**
otherwise (`sh2-flash:277-279`), prints the artifact's sha256, checks
`lsusb | grep 2e8a:000f` with a readable error (`sh2-flash:294-299`), confirms, loads,
reboots, and prints the machine-power judgement box. Bare picotool does none of that.

The single failure it misses is the worst-shaped one in this procedure: flashing
`seedhammerii-*.uf2` instead of `*.signed.uf2` produces a dark screen and a device
back in BOOTSEL — pixel-identical to a rejected signature and to the laptop-PD case
the runbook warns about. The standing project rule is already "always `sh2-flash`,
never picotool by hand."

**Classification: DEFAULT (use the tool that already exists).**

**Minimal remedy.** `sh2-flash /path/to/seedhammerii-v0.0.0-bgf5b068f.signed.uf2`.
If bare picotool is preferred, first confirm
`sha256sum` = `7fb899cb4ea3a9fbcfc8f3f0299fa53d9c850eed40ce53a10845a0acebeaaf0c`
and `picotool info -a <file> | grep signature:` reads `verified`.

---

### I-5 — Before step 5: after the flash the two boards are indistinguishable on screen, and the file that records board identity says there is only one SeedHammer II

**Moment.** The instant step 5 completes.

**What goes silent.** The version line is built at `gui/run_flow.go:118-121` as
`"Firmware: " + version + "\nHardware: " + pl.HardwareVersion()` with `" (UNLOCKED)"`
appended. `grep -rn 'UNLOCKED' --include='*.go'` over the fork returns three hits
(`gui/run_flow.go:120`, `cmd/emu/platform.go:340`, one test) and **nothing
board-unique is rendered anywhere**. Both boards will run the same firmware build and
show the same three lines. From that moment the only way to tell them apart is USB in
BOOTSEL, or a physical label.

Meanwhile `design/HARDWARE_INVENTORY.md` lists exactly one SeedHammer II —
`0x77c483b745abf55c`, labelled *"the real machine"* — and its opening paragraph exists
because on 2026-08-07 two RP2350s were in BOOTSEL simultaneously and identifying them
meant reading picotool output against notes in three files. That is the same bench,
about to gain a second identical-looking unit.

**There are also three spellings of one board id in play**, verified arithmetically:

| where | new board | old board |
| --- | --- | --- |
| `picotool info` chipid / HARDWARE_INVENTORY | `0x09f50bf63e8d6f46` | `0x77c483b745abf55c` |
| USB serial (sysfs, `--ser`) | `09F50BF63E8D6F46` | — |
| `sh2-state/*.chipid` (script's row order) | `6f463e8d0bf609f5` | `f55c45ab83b777c4` |

They are the same four 16-bit words in reverse order. An operator hand-comparing
`sh2-chipid.txt` against `lsusb` will see two different strings for one board.

**Classification: DOCUMENTATION ONLY — but it must happen BEFORE step 5**, because
after step 5 the screens match and the cheap distinguisher is gone.

**Minimal remedy.** Physically label the new enclosure now, and add one row to
`design/HARDWARE_INVENTORY.md` carrying **both** spellings plus the note that two
units now trust the same key so `SH2_BOOTKEY_FP` no longer identifies a machine.

---

### I-6 — Step 4: "all three BOOT_FLAGS1 copies agree" compares a MAJORITY against two raw rows, not three raw rows

**Moment.** The last three lines of `--sh2-verify-valid`
(`scripts/pico2-bootkey-rehearsal.sh:802-809`).

**The claim under test (from the brief).** The runbook says
`--sh2-verify-valid` *"fails closed on that warning"* and *"requires all three copies
to agree"*. I checked both against picotool's source.

**The warning claim is TRUE.** `otp_get_command::execute` prints
`" (WARNING - REDUNDANT ROWS AREN'T EQUAL)"` whenever any bit differs across the
redundant copies (`main.cpp:7628-7677`, literal at 7675), and `otp_field` captures
`picotool otp get -n "$sel" 2>&1` and `die`s on a case-insensitive `WARNING` match
(`scripts/pico2-bootkey-rehearsal.sh:178-182`). So a 2-of-3 write is caught. The
runbook's load-bearing sentence stands.

**The three-copies claim is FALSE.** In `otp_get_command::execute`,
`if (redundancy < 0) redundancy = m.reg->redundancy;` (`main.cpp:7592-7594`), and row
`0x04b` **does** resolve to a known register — `picotool otp list -n 0x04b` prints
`ROW 0x004b: OTP_DATA_BOOT_FLAGS1 (RBIT-3)`, while `0x04c` and `0x04d` print nothing
at all. So `read_row_raw24 0x04b` returns the **3-way majority vote**, and
`read_row_raw24 0x04c` / `0x04d` return their own rows. Whenever
`0x04c == 0x04d != 0x04b`, the majority equals the other two, `A == B == C`, and the
script prints `PASS: all three BOOT_FLAGS1 copies agree`. That is the reachable
`{0x04b written, 0x04c/0x04d not}` prefix interruption.

**The gate still fails closed in that state** — by the `WARNING` grep above, and
independently because `KEY_VALID` would then majority-vote to `0x1` and trip the
MISSING branch. So this is a defect in what the tool *claims to have verified*, not
an open hole today. The same shape applies to `--sh2-precheck`'s CRIT1 loop: `0x040`
resolves to `OTP_DATA_CRIT1` while `0x041`-`0x047` do not.

**Classification: WARNING (the tool over-claims).**

**Minimal remedy — one flag, and NOT today.** `picotool otp get -n -c 1 <row>` pins
`settings.otp.redundancy = 1`, so `VALUE` is the true raw row. **Do not make this
change before the burn**: editing a gate script minutes before an irreversible write
is a larger risk than the over-claim, and the gate demonstrably fails closed as it
stands. File it for after step 5.

---

### M-1 — Step 1: the OTP json is named after a board it is not bound to

`~/.sh2/otp-6f463e8d0bf609f5.json` is **byte-identical** to `~/.sh2/my-otp.json`
(verified with `cmp`), because `--make-otp-json` derives the file from the key and the
slot only — no board is consulted. The board-suffixed name is evidence of a check
that does not exist anywhere in the pipeline. Nothing bad follows here (the content is
correct for both boards), but it is the kind of name that makes I-2's missing binding
feel already handled. **DOCUMENTATION ONLY.** Remedy: one clause in the runbook saying
the file is a function of key+slot, not of the unit.

### M-2 — Steps 2 and 4 depend on two `.gitignore`d directories, one of which every document calls disposable

`reject_rehearsal_key` **dies** if any of
`rehearsal-work/{factory-key,my-key,third-party-key}.pem` is missing
(`scripts/pico2-bootkey-rehearsal.sh:636-645`), and `sh2_require_seedhammer` dies if
`sh2-state/sh2-chipid.txt` is gone (lines 605-613). Both directories are ignored
(`.gitignore:23,28`) and `rehearsal-work/` is described throughout as disposable
("the board is CONSUMED"). Had either been cleaned, **step 2 would die after step 1
had already burned the slot**, stranding the operator at a gate with an irreversible
write behind them and no way forward but `ALLOW_UNCHECKED_KEY=1`.

Verified intact right now: all three rehearsal keys present, `sh2-chipid.txt` =
`6f463e8d0bf609f5`. **No action today.** Recorded so it is not rediscovered mid-burn
on the third board. **DOCUMENTATION ONLY** — add "confirm `rehearsal-work/` and
`sh2-state/` still exist" to the runbook's prerequisites.

---

## DIVERGENCES CONSIDERED AND DELIBERATELY NOT ACTIONED

**1. Step 1 — nothing checks whether the operator is on a PD-capable port.**
Correct: nothing checks. But nothing on this host *can*. `/sys/class/typec/` does not
exist on this box and `/sys/class/power_supply/` is empty, so the PD contract is not
observable from the laptop at all; the AP33772S sits on the device side and the
firmware that would drive it is not running in BOOTSEL. The one part of the runbook's
power advice that IS machine-checkable I checked, and it is already satisfied: the
board is on root-hub port 10 directly (`/sys/bus/usb/devices/1-10`, `devpath 10`, no
hub in the path). Adding a prompt that asks "are you on USB-A?" collects an
unverifiable self-report; the runbook already says it in prose, in the right box, at
the right moment. **Fails the "worse than telling them nothing" test.**

**2. Step 3 — the operator drops `-s`.** Traced through
`otp_set_command::execute`: without `-s`, `settings.otp.value` = `0x2` and
`old_raw_value` = `0x1`, so `if (~settings.otp.value & old_raw_value)` is non-zero and
picotool refuses with `"Cannot clear bits in OTP row(s): current value 000001, new
value 000002"` (`main.cpp:8102-8112`). A refusal, not a silent revocation of slot 0.
**Already safe; no change.**

**3. Step 3 — the operator types `0x1`.** With `-s`, `value |= old_raw_value` makes it
a no-op; no bits are cleared so the write is accepted and nothing changes. Step 4 then
reports `KEY_VALID` = `0x1`, `MISSING` = `0x2`, and hands over the exact correct
command. **Already handled by the gate.**

**4. Both boards plugged in at once.** `fail(ERROR_NOT_POSSIBLE, "Command requires a
single RP-series device to be targeted.")` (`main.cpp:8692-8696`) before any OTP
command runs. **Already a refusal.**

**5. Re-running `otp set -s` after a partial step 3.** The runbook says it is safe
because it only sets bits. Verified: `if (settings.otp.ignore_set) settings.otp.value
|= old_raw_value;` (`main.cpp:8100-8103`), plus the field path's
`value |= old_raw_value & ~field->mask` (`main.cpp:8093`). **The advice is correct as
written; no change.**

**6. F-143 — `sh2-flash` compares the key against a recorded constant, not the
device's live OTP.** The brief asks whether that matters here. It does not: both
boards trust the same key, so a live-OTP comparison returns the same verdict as
`SH2_BOOTKEY_FP`, and step 5 is retryable at zero OTP cost. Worth noting separately
that F-143's own stated blocker is now gone — it was deferred because "the device had
already rebooted out of BOOTSEL", a board is in BOOTSEL right now, and `--sh2-precheck`
has since proven the `BOOTKEY*_*` row format on real hardware. **Not a gate for this
burn.**

**7. The success path of `--sh2-verify-valid` has never run.** True, and bounded. The
brief's settled facts plus `--sh2-precheck` passing on this board mean every read
helper that mode uses has already executed against this silicon: `read_slot` and
`key_hash` (via the negative control's correct all-zeros `die`), `otp_field` on both
`BOOT_FLAGS1` fields, `read_row_raw24` on `0x04b`-`0x04d`, the CHIPID pin, and
`reject_rehearsal_key`. Only the final comparison and the closing banner are
first-run, and every failure mode they have is a `die` costing zero OTP. **No change.**

**8. The CHIPID pin is a single mutable file and was hand-edited today.**
`sh2-chipid.txt` now names the new board; the old board's id survives only in an
ad-hoc `board-f55c45ab83b777c4.chipid` that the script never reads, and
`--sh2-precheck` refuses to re-pin when a mismatching pin exists, so switching boards
requires manual file surgery with no documented procedure. This is a genuine
weakening of a safety check — but it acts on steps 2 and 4, which today are pointed at
the correct board, and the remedy that actually closes the wrong-board question is
I-2's `--ser`, which needs no state file at all. **Folded into I-2 rather than
actioned separately.**

**9. `picotool otp load`'s help advertises a verify it does not perform for JSON.**
Real (`main.cpp:7858-7860` returns before the verify loop at 7887-7901), and it is
picotool's defect, not ours. Reporting it upstream changes nothing about this burn,
and our step 2 is the readback. **Captured inside I-1; no separate action.**
