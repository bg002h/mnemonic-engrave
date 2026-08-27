# CONSULT — P0 row 4, F-259: exit digit and message for the terminal + no-secret refusal

Architect consult, 2026-08-27. Question: the exit digit and replacement message for
`me sysw wipe` refusing a TTY stdout, once `PayloadKind` stops the wipe image
riding the bearer arm. Grounded against `crates/me-cli/src/main.rs` in this
worktree: the exit-vocabulary table (~line 279), `refuse_write_block`
(`WriteBlock::Terminal → EXIT_USAGE`, ~line 902), the bearer message
(`refuse_terminal_destination`, ~line 937), and the placeholder
(`refuse_terminal_fill_image`, ~line 987).

## DIGIT

`2`

The codebase's own vocabulary table discriminates by the operator's next move — 2 means "fix the command line" — and that is exactly this remedy: add `--out`; the condition is a property of the invocation's destination, not a judgment of the data, and 3's promise ("this tool will never do that") is false here because the tool will do the wipe happily, one flag away. The bearer sibling is frozen at 2 for the identical condition and remedy, so forking the digit by payload kind would give a wrapper script nothing actionable while assigning the *less* sensitive payload the *more* severe code. 0 is rejected: F-259 itself rules the refusal stays, and the default fill is `random`, so the 64 KB would carry escape sequences into a terminal nobody is parsing — the paint is the harm, whatever the secrecy.

## MESSAGE

```
me: stdout is a TERMINAL, and this payload is a WIPE image, not a secret.

Writing it here would paint 65536 bytes of raw binary across your scrollback. Nothing was written.

Give it a file, then flash that file:

  me sysw wipe --fill ... --out wipe.bin
  picotool load --verify wipe.bin -t bin -o 0x10D00000

with the machine in BOOTSEL. Or wipe with no image at all:

  picotool erase -r 0x10D00000 0x10D10000

Do NOT pipe into picotool: it sizes its input with fstat, a pipe reports 0 bytes, and the load exits 0 having written nothing — a wipe that wiped nothing.
```

(571 bytes vs the current 572 — measured with `wc -c`, not estimated. If the
implementation keeps the `{len}` interpolation, 65536 is what it renders for a
wipe image: every wipe is exactly `REGION_LEN` = 0x10000 bytes.)

## WHY THIS TEXT IS DERIVED

- **"a WIPE image, not a secret"** — derived from the carried `PayloadKind`: the image is a fill pattern built by `sysw wipe` from `zeros|ones|random`; no key material ever enters it. The line affirms the true classification instead of asserting the false one, which is the whole of F-259's ruling.
- **"65536 bytes of raw binary across your scrollback"** — the observed image length (`REGION_LEN` = 0x10000) and the observed destination (a TTY). True for all three fills. The bearer sentence's "terminal sessions are often logged" clause is *dropped*, not rephrased: logging is a secrecy rationale, and keeping it for a no-secret image would be the same rule-name vestige F-259 exists to remove.
- **"--fill ..."** — the fill is the operator's choice and defaults to `random`; the placeholder's hard-coded `--fill zeros` would state a false command for two of the three invocations (F-260's exact shape). The trailing-`...` elision mirrors the bearer message's own `me sysw pack --region --out payload.bin  ...` idiom. `wipe.bin` (not `payload.bin`) keeps the wipe image from ever shadowing a real payload file, and matches the name the tests already use.
- **"Or wipe with no image at all" / "a wipe that wiped nothing"** — erase sets NOR to 0xFF, which the `--fill` help itself documents as the erased state, so `picotool erase -r 0x10D00000 0x10D10000` genuinely accomplishes the wipe with no image; and the pipe warning stays because its false-success shape is *worst* on a destruction op — a load that exits 0 having written nothing leaves the operator believing a payload destroyed that is still in flash.

## WHAT I REJECTED

- **Exit 0 (write the 64 KB):** F-259 keeps the refusal settled; beyond that, the default random fill would push escape sequences into a live terminal for no reader — the mess is the harm even with nothing secret in the bytes.
- **Exit 3:** it would fork the digit on payload kind while the bearer arm is frozen at 2 for the same condition and remedy — scripts learn nothing, severity inverts, and 3's "this tool will never do that" is untrue when `--out` is the sanctioned path to the same bytes.
- **Keeping "terminal sessions are often logged" or any secrecy language:** true of terminals, but it is the *bearer* rationale — retaining it for a fill image re-writes the defect the fix exists to remove, and softening it ("harmless here, but...") spends words excusing a claim the message no longer needs to make.
