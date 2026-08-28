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


---

## W2 — the operator remembers the FLAG but not the TOOL, and the tool
## whose NAME advertises descriptors is the wrong one

**The moment.** Told "you want the QR plate — what do you type?", the
operator typed nothing: *"I remember there as --as options, but I wonder was
that the md-cli or the me command…"*

**The finding.** The constellation has five CLIs, and the capability lives in
`me` — but `md` is the tool literally named for descriptors
(mnemonic-descriptor), so the name pulls the wrong way. Measured, both
wrong-tool moves dead-end generically:

```
md encode "$(cat wallet.txt)"          -> md: template parse error: template contains no @i placeholders
md encode 'wsh(sortedmulti(2,[dc…]xpub…/<0;1>/*,…))'  -> same message
```

No referral to `me`, no recognition that the input is a concrete export
rather than a template. An operator who guesses `md` gets a message about
`@i` placeholders they have never heard of, and can reasonably conclude the
constellation cannot do this at all — worse than silence.

**Classification: remedy in the SIBLING tool — filed as F-420 against
descriptor-mnemonic, not a change to this spec** (§10 already declines `md`
changes for this cycle). `md encode`'s template-parse refusal gains a
referral when the input is descriptor-shaped (a concrete key expression, or
`Key: value` BlueWallet lines): name what the input IS and where it goes —
`me sysw pack --as <descriptor|md1>`. Sequenced to land with or after S1, so
the referral never points at a flag that does not exist yet.

**Within this spec's scope: nothing to change** — once `me` is reached, §5.1
and §6 handle the rest. The operator's problem is REACHING it; the
constellation-uniformity surface owns cross-tool discovery.


---

## W3 — the operator's natural spelling half-parses into a DIFFERENT program,
## and clap's similarity tip points deeper into it

**The moment.** Knowing the tool and wanting the QR plate, the operator
typed: `me --in wallet.txt --as descriptor` — no subcommand. Natural: "tool,
input, output form."

**Measured.** `me`'s TOP LEVEL owns `--in <FILE>` — the original NDEF
converter — so the spelling half-parses: clap accepts `--in`, trips on
`--as`, and tips *"a similar argument exists: '--base64'"* (string
similarity into the converter's flag set). Usage line shown:
`me --in <FILE> --base64`. Nothing names `sysw pack`. And `me --in
wallet.txt` alone RUNS the converter on the BlueWallet file (its refusal
recorded in the commit adding this entry). The corrected spelling today
(`me sysw pack --in … --as …`) tips `--no-passphrase` — pre-S1, expected.

**Classification: remedy, filed as F-421 (in-tool twin of F-420).** When the
top-level converter refuses an input that is descriptor-shaped, it refers to
`me sysw pack --as <descriptor|md1>`. Same sequencing rule as F-420: lands
with or after S1. The clap similarity tip is generic machinery and not worth
fighting; the converter's own refusal is ours and can say the true thing.

---

## W4 — the corrected command, in the S3-only world the plan ships first,
## has NO SPECIFIED OUTCOME — and it is the first command a real user runs

**The moment.** The operator's corrected command is
`me sysw pack --in wallet.txt --as descriptor` — and they chose
`--as descriptor` deliberately (W1: they want the scannable QR plate).
Under F-418, the first shipping build is S3-only. The spec defines
`--as <descriptor|md1>` whole (§5.1) and parks S2 (§8, §11) — but nothing
says what `--as descriptor` DOES in an S3-only build. Clap value error?
Unknown-flag error? Both would be false or useless; neither is specified.

**Classification: SPEC change (fold after the walk), Important-class for the
plan.** §5.1 gains the window clause: in a build where S2 has not shipped,
`--as descriptor` is a REFUSAL, `EXIT_REFUSED` (3), naming the true reason
and the live alternative with the W1 modality difference the operator needs
to decide: *"--as descriptor (the scannable QR plate) needs the device half
of this feature (S2), which this build does not carry. --as md1 is
available now: error-corrected TEXT cards, restored by transcription rather
than scanning. If you need the QR plate, keep the export file; it packs the
day S2 ships."* §6 gains the row; §11 item 5's sibling test pins it. This
also discharges the interim-remedy question from the plan inputs: §6 rows
that name `--as descriptor` stay honest in the window BECAUSE the flag's own
refusal explains the window and preserves the operator's export-file path.


---

## W5 — the operator decoded the window refusal "through a convoluted
## message"; the draft led with mechanism and leaked an internal phase label

**The moment.** Shown W4's drafted refusal, the operator: *"Now I understand
through a convoluted message that what I want to do is not possible yet."*
Understood — but through effort. A refusal the operator must decode is
half-failed, and their one-sentence summary IS the message's correct first
line.

**What was wrong with the draft.** It opened with the flag name, a
parenthetical, and *"the device half of this feature (S2)"* — S2 is this
cycle's internal phase label, meaningless to an operator. Mechanism first,
verdict last; jargon in between.

**The rewrite (outcome first, no internal vocabulary):**

    me: --as descriptor is not available in this build.
          The QR plate needs device firmware this release does not include.
          Available now: --as md1 — error-corrected text cards, restored by
          transcription instead of scanning. Your export file is all you
          need to come back for the QR plate later; nothing is lost by
          waiting.

**Classification: spec fold (with W4's), plus a NORMATIVE rule and a sweep.**

1. W4's window refusal uses the rewrite above.
2. NEW RULE for §6's preamble: **operator-facing refusal text contains no
   internal identifiers** — no phase labels (S1/S2/S3), no follow-up numbers
   (F-4xx), no spec § references inside the quoted text. Those belong in the
   row's annotation column, not in what the operator reads.
3. Sweep §6's quoted texts for existing leaks: at least the multi-record
   row's quoted message contains "(F-414)" today. Fix in the walk fold.


---

## W6 — told "--as md1 is available," the operator asks whether they must run
## `md` FIRST to make the md1 string, or whether `me` converts

**The moment.** *"Now I wonder if I have to use md command to convert
wallet.txt to an md1 string or if me will handle it."*

**Why the confusion is legitimate, twice over.** (1) `md encode` IS the md1
encoder in this constellation, and the operator has used it — "as md1"
plausibly means "as the thing `md` makes." (2) The bring-your-own-string
pipeline genuinely exists: `me sysw pack` accepts md1 strings as records
(`ClassMDMK`) today, so `md encode … ` then `me sysw pack <strings>` is a
real, working path — the expert path §1 exists to obsolete. The spec's
answer — `me` converts in-process, one step, no `md` involved — lives in
§5.3's implementation notes, which no operator reads.

**Classification: documentation, folded with the walk batch.** §5.1's
`--as md1` help line gains the one-step fact and loses its spec-speak
("decompose to a BIP-388 template plus keys" fails the W5 lens): *"--as md1
— converts the descriptor and packs error-corrected md1 text cards in ONE
step; no `md` invocation needed."* The W4 window refusal's alternative line
gains the same three words ("me converts and packs in one step"). The
bring-your-own-string path remains valid and needs no mention here — the
confusion to cure is "must I?", not "may I?".
