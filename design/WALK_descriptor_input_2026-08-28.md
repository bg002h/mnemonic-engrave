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


---

## W7 — the walk's own harness erred: there IS no passphrase prompt on this
## journey, and the measured behavior is already right

**The moment.** The walk told the operator "a passphrase prompt appears" and
asked what they would do. They answered: *"passphrase — my wallet file isn't
password protected. So I hit enter to skip decryption of wallet.txt"* — a
DECRYPT-MY-INPUT reading of a prompt that, per the flag docs, would be about
ENCRYPTING the output.

**Then the measurement overturned the premise.** With a valid public-only
record and no passphrase flag, `me sysw pack` does not prompt at all
(`--passphrase-ask` is explicit opt-in). It states:

    sealing:  NOT SEALED — no record in this payload is secret material, so
              there is nothing to encrypt. The container is cleartext: anyone
              holding the file can read it.
    strength: no passphrase — BELOW the threshold

and writes the payload. The walked beat was the walk harness's assertion
from misremembered probes (every prior probe had passed `--no-passphrase`),
not the tool's behavior. Corrected in the moment, per the read-the-whole-
output discipline; also: the FIRST probe of this beat used the stale
`target/release/md` (spaced stdout — §2's own stale-binary trap, third
occurrence this cycle) and refused before reaching passphrase logic at all.

**Classification: no spec change from the prompt confusion — the moment does
not exist on this journey.** Two residues logged: (a) the operator's
decrypt-my-input reading is REAL and waiting for any journey where a
passphrase prompt does appear (secret-carrying payloads; note for that walk);
(b) `strength: no passphrase — BELOW the threshold` reads as a nag on a
payload with nothing to encrypt — pre-existing sysw surface,
secret-handling-adjacent, noted for the sysw backlog, not this cycle's.


---

## W8 — shown the materialised `<0;1>/*`, the operator asked "is this the
## wrong derivation path?" — the confirmation shows it but does not explain it

**The moment.** The §5.4 confirmation sketch put
`wsh(sortedmulti(2,@0/<0;1>/*,…))` in front of the operator — a spelling
their file never contained. Reaction: *"What was fought over for 3 rounds?
Is this the wrong derivation path?"* Mild alarm, unresolvable from the
screen. Alone at the terminal, that is either an uneasy proceed or an
aborted CORRECT pack — both bad, at the journey's highest-stakes moment.

**The gap.** §5.3(a′) requires the confirmation to print the template WITH
the materialised `<0;1>/*` "so the operator sees it." Seeing is not
understanding: the confirmation presents an unexplained novelty exactly
where trust is being asked for. The walk's whole a′ debate (r1 C4 → r2 → r3:
what does a childless file MEAN; answer: what the device derives) is
invisible at the surface that needed it.

**Classification: SPEC change (walk fold).** §5.4 gains: when (a′)
materialisation occurred, the confirmation carries ONE annotation line, in
operator terms, W5-compliant (no internal labels):

    note: your file names no derivation below the key origins; <0;1>/* is
    the receive/change convention the device uses for such files either
    way. Addresses are unchanged by making it explicit.

§5.3(a′)'s "so the operator sees it" clause is upgraded to "sees it and is
told, in one line, why it is there and that addresses are unchanged."


---

## W9 — "is a′ in keeping with a BIP?" — yes, three ways, and the spec argues
## from the weaker authority

**The moment.** Given W8's explanation, the operator asked whether §5.3(a′)'s
materialisation is grounded in a BIP or is our invention.

**The answer, and it is strong.** (1) BIP-48: the operator's own declared
origin `m/48'/0'/0'/2'` is a BIP-48 path, and BIP-48 defines the next two
levels as change (0 = receive, 1 = change) then index — the materialised
tail is BIP-48's own continuation. (2) BIP-388: placeholders end `/**` or
`/<NUM;NUM>/*`, and `/**` ≡ `/<0;1>/*` — an equivalence already
machine-verified in this constellation (F-411: byte-identical md1 cards).
a′ completes a childless file INTO a valid wallet policy. (3) BIP-389
supplies the notation. The device's "convention" is these BIPs implemented.

**Classification: documentation (walk fold), strengthening W8's fix.**
§5.3(a′)'s rationale currently rests on measured device behaviour ("the
device is the reader of both artefacts") — true but private. Add the BIP
grounding as the leading rationale, and W8's confirmation annotation cites
it in operator-checkable form: *"<0;1>/* is the standard receive/change
continuation of your keys' BIP-48 origin (BIP-388's canonical tail)."* An
authority the operator can verify beats "the device does this."


---

## W10 — the operator specifies the confirmation's missing lines: "a wallet
## identifier and first address"

**The moment.** Shown where the payload goes (measured on a pty: the shipped
terminal guard refuses stdout — *"stdout is a TERMINAL, and this payload is
BEARER … Nothing was written"* — with the `--out` + flash remedy), the
operator said what they WANTED at this point on stderr: *"a wallet
identifier and first address."*

**Classification: SPEC change (walk fold) — §5.4's confirmation gains two
lines, both from machinery the constellation already owns:**

1. **`wallet-id:` the WalletPolicyId fingerprint** — the identifier that
   exists "for cross-verifying digital copies of the policy against the
   engraved Template Card." `me` parses host-side on BOTH paths, so one
   identifier is shared across `--as md1` and `--as descriptor`: the same
   wallet prints the same id whichever plate is cut. (F-212's lesson binds:
   the id is cross-language, so the §7 vector rows should carry it too —
   one more column, `wallet_id`, asserted by both suites.)
2. **`address 0:` receive address 0, with the compare prompt** — one line:
   *"compare against your wallet software's first receive address before
   engraving."* This turns the confirmation into an EXECUTABLE check
   against the coordinator the operator already trusts — the same equality
   eight review rounds used as ground truth, handed to the operator.

**Also logged from this beat's measurement (notes, not this cycle's):**
(a) the terminal guard calls every payload "BEARER" — seconds after
`sealing:` said "no record in this payload is secret material"; the two
surfaces should agree (public-only payloads are not bearer). (b) The
guard's remedy names raw `picotool`; the operator's own bench standard is
`sh2-flash` — environment-specific, noted only.


---

## W11 — the operator's QUESTION ("how would a JSON /0/* export arise?")
## exposed a refusal loop in the S3 window, before any step was walked

**The provenance answer, for the record.** A `{label, descriptor}` JSON with
`/0/*` is a Specter-era wallet export — the fork's own `parse_test.go`
fixture is one (`label`/`descriptor`/`blockheight`/`devices` keys). Pre-
multipath software stored the RECEIVE-branch descriptor and derived change
by convention, so every export of that era carries `/0/*`. Such files
outlive their software — the archival user is the engraving tool's core
clientele.

**The composed loop.** For that user, in the S3-only window:

1. `--as md1` → §5.3(a) refuses `/0/*`: "Use `--as descriptor`, which
   carries `/0/*` exactly."
2. `--as descriptor` → W4's window refusal: "not available in this build …
   `--as md1` is available now" — FALSE for this input. Goto 1.

The r5-NEW-I2 defect class (refusals pointing at each other), resurrected by
the phase window. Neither text is wrong alone; the WINDOW composes them into
a loop for exactly the archival journey.

**Classification: SPEC change — amends W4's queued fold before it lands.**
The window refusal's alternative clause is CONDITIONAL: offer `--as md1`
only when the input is md1-representable; for an (a)/(a″)-shaped input it
says instead: *"--as md1 cannot carry this wallet's /0/* path either — no
path in this build engraves this file. It loses nothing by waiting: keep it,
and it packs the day the device update ships."* Symmetrically, §5.3(a)/(a″)'s
remedies, in a build where S2 has not shipped, append: *"(--as descriptor is
not in this build yet — keep the file.)"* No refusal may point at a flag
that refuses in the CURRENT build — the r5 rule, now stated over build
windows, not just admission.


---

## W12 — "are we refusing a common wallet type?" — no permanently, yes in the
## window; and the question yields an unruled option (F-422)

**The moment.** Mid-journey-2, the operator: *"Did you say we are refusing a
common wallet type (specter JSON)?"*

**The precise state.** The type is fully carried by `--as descriptor`
(admitted, parsed, canonical packed, device derives it — measured r1). Only
`--as md1` refuses it, per F-417's ruled wire-format narrowness, with the
carrying path named. The GAP is temporal: in the S3-only window the carrying
path is unbuilt (W11), and pulling its host half forward would not help —
the record needs S2's device classifier to be READABLE, so early packing
mints payloads current firmware refuses at load.

**The unruled option the question surfaced, filed as F-422.** A Specter-era
`/0/*` is the pre-multipath spelling of the full receive+change wallet — the
change branch was implied by convention. `me` could offer an EXPLICIT,
consented, never-default transform ("pack the `<0;1>` wallet this
receive-branch export denotes"), making the archival journey work under
`--as md1` in the window. It packs a different literal descriptor than the
file — the line this cycle refuses to cross silently — so it is an operator
ruling, not a walk disposition.


**W12 addendum — interim ruling.** Mid-journey-2 the operator ruled F-422
STATUS QUO ("no… until I understand it better"): no transform ships; W11's
neither-path text stands; the entry is a decision record until the operator
asks for a fuller brief.


---

# Journey 2: a bare `zpub` — watch-only bequest

**Scene.** A friend's BlueWallet single-sig segwit wallet; the goal is a
steel card their family could use to FIND the funds (watch-only), seed held
elsewhere. In hand: one clipboard line, `zpub6rFR7y4…`. S3-era build.

**Beat 1.** The operator's first move: *"paste the zpub into the terminal
after me sysw pack --as descriptor"* — the SECOND journey whose natural
first reach is the parked path, and here it is not habit: for a bequest the
QR plate is genuinely right (the family scans; nobody transcribes at a
funeral). Logged for the F-418 window record: both walked journeys hit the
window on their first real command.

**Beat 2.** Per the spec (with W11's ordering: parse first, so the window
refusal can be truthful about its `--as md1` offer), the operator sees the
§4.5 promotion announcement — `key as supplied` (their zpub), `inferred
wallet` (`wpkh(xpub…)`, normalised), the version-byte reason — then the
window refusal offering text cards now or QR later.

---

## W13 — the operator looked for W10's wallet-id and first address at the
## refusal, and they are not there: identification was scoped to the
## confirmation, which a refusal never reaches

**The moment.** *"I would first look to see the first address and/or wallet
identifier, like last time, but I don't see it."*

**The gap.** W10 attached `wallet-id:` and `address 0:` to §5.4's
confirmation — a surface that fires only when the pack proceeds. On the
window path (and the md1-representability path), parse SUCCEEDS and the
pack does not: the operator is asked to make the wait-or-switch decision
about a wallet the tool has identified but not shown. The verification is
worth the most exactly here: the friend's phone shows the receive tab;
one address comparison proves the inferred wallet is theirs BEFORE they
archive the key line and wait for the QR build.

**Classification: SPEC change (walk fold), generalising W10.** The
identification block — inferred/canonical wallet, `wallet-id`, `address 0`
with the compare prompt — prints on EVERY successful host-side parse,
before whatever follows (pack, window refusal, §5.3 refusal). One rule:
**parse succeeded ⇒ identify the wallet, always; then say what can or
cannot be done with it.** Refusals follow identification, never replace it.
