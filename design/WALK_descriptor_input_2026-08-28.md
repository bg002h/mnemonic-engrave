# Journey walk — SPEC_descriptor_input §6/§5, live with the operator, 2026-08-28

Method: one journey, operator role-plays with a real artifact; at each step —
what is in hand exactly, what does the tool do, what ELSE might they do.
A divergence earns a change only if the wrong outcome is worse than silence.
Spec at GREEN (`6fff505` + walk-era commits). Baseline binary: `me 0.7.0`.

**Journey 1: a BlueWallet 2-of-3 export (the fork's own `sh` fixture, 14
lines) saved as `wallet.txt`; the operator wants the wallet structure on
steel. S3-era assumption (--as md1 built, S2 parked per F-418).**

---

## W1 — before typing anything, the operator's first thought is a question
## §5.1 cannot answer: "md1 string, or QR for easy scanning and reimport?"

**The moment.** Prompted "what do you type?", the operator did not type — they
said: *"am I encoding this into an md1 string, or am I preparing this to
engrave in a QR code for easy scanning and reimporting later…"*

**The finding.** That IS the `--as` choice — and it is framed in plate and
restore terms, while §5.1's block (the text that exists precisely because "an
operator holding a wallet export does not know which they want") answers in
codec terms: canonical form, BIP-388 decomposition, policy coverage, firmware
need. Verified at source while the operator waited:

- `--as descriptor` → the device's descriptor programs engrave **a QR of the
  canonical descriptor** (`gui/gui.go:693–695`,
  `qr.Encode(desc.EncodeNoChecksum(), qr.L)`). Restore = scan with any phone
  or coordinator (BlueWallet reimports it). The SH2 itself can never read it
  back — no camera.
- `--as md1` → **text card plates** (codex32-style groups, BCH correction up
  to 4 substitutions per string). Restore = human transcription into
  `md decode` / a host tool. Not machine-scannable.

Two different restore PLANS, not two encodings of one artifact. An operator
who picks by §5.1's current text can engrave the wrong plan and discover it
at restore time, years later — worse than silence.

**Classification: documentation (spec change, help text + table).** §5.1's
two flag lines each gain a plate/restore clause ("engraves as a QR — scan to
reimport; the device cannot read it back" / "engraves as error-corrected
text cards — restore by transcription"); §5.5 gains two rows: *on the plate*
and *restored by*.

**Corollary (feeds the plan, F-418 livability):** the operator's FIRST
instinct — scan-and-reimport — is the S2 artifact. In the S3-only window the
scannable plate does not exist. The plan must decide what §5.1's help text
says about `--as descriptor` while S2 is parked (see the earlier
phase-boundary note in the plan inputs).
