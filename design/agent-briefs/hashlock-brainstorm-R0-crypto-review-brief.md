# Brief: hashlock-phrase brainstorm, R0 round 0 -- cryptography + Bitcoin programmer lens (opus, single agent)

You are an expert cryptographer and Bitcoin programmer (script, miniscript,
PSBT, BIP-39/BIP-93, KDFs). You are reviewing a DESIGN RECORD, not code. You
are READ-ONLY except for the one report file you write at the end.

## The one question

**Is the hashlock-phrase design in
`/scratch/code/shibboleth/mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md`
cryptographically and Bitcoin-script sound -- and where it is not, what is the
concrete counterexample?** A finding is a construction or an input under which
the design produces a wrong result, an unspendable policy, an anyone-can-spend
path, a guessable secret the record claims is safe, or a claim about Bitcoin or
a library that the authoritative text contradicts. An opinion without a
construction or a citation is not a finding; put it under "Questions for the
operator".

## Scope (and what is NOT in scope)

Read, in this order:

1. `design/BRAINSTORM_hashlock_phrase.md` -- all sections. Section 2 holds the
   operator's rulings (final); section 3 the measured context; section 4 the
   agreed design; section 5 defaults taken for the operator's veto; section 6
   the sections not yet designed.
2. `design/SPEC_wallet_policy_composer.md` sections 4 (the `HASH` row), 6c, 8h,
   8i, 14 (the preimage row) -- the shipped composer's rule the design serves.
3. `design/FOLLOWUPS.md` entries F-132, F-465, F-466, F-467, F-468.
4. `design/S4_journey_walk_2026-09-02.md` section "W-5".
5. In mnemonic-secret (`/scratch/code/shibboleth/mnemonic-secret`):
   `MIGRATION.md`, `design/SPEC_ms_v0_2_kofn.md` lines 20-40,
   `crates/ms-codec/src/consts.rs`, `crates/ms-codec/src/envelope.rs` lines
   180-250, `crates/ms-codec/src/payload.rs`.
6. In the fork (`/scratch/code/shibboleth/seedhammer`): `codex32/mspayload.go`,
   `gui/composer_hash.go`, `passphrase/passphrase.go`.

NOT in scope: a fresh audit of ms, the composer, or the fork; the sections the
record lists as still to be designed (4.4 device leg, 4.5 process, 4.6 testing)
except where a section-2 ruling or a section-4 decision already forecloses
something you can show is unsound; code style; anything you would need to
implement to check.

## Settled facts -- do not re-derive, do not re-litigate

- The rulings L1-L11 are the operator's and final. You may report a
  CONSEQUENCE of a ruling (with a construction); the ruling itself is not a
  finding.
- Every number in section 3 was measured by the controller on this machine or
  on silicon (kdfbench 9,715 it/s; the hashvault phrase lengths 40/38/34; the
  ms1 kind bytes; the fork's decoder arms; the dependency pins). Trust them
  unless you can show a contradicting measurement, in which case cite it.
- Terminology (L2): hashlock phrase / preimage X (32 bytes) / digest H.
- Secret-HANDLING defects (material on argv, in shell history, on a stream, in
  RAM longer than needed, unwiped strings) are NEVER Critical or Important by
  operator ruling 2026-08-27. Record them as Minor with the class named.

## External protocol facts MUST be verified against authoritative text

Cite `file:line` for local sources or a URL plus the quoted sentence for
remote ones. Local sources on this machine: rust-miniscript in
`~/.cargo/registry/src/*/miniscript-*/` (find it with `ls`), the fork's
`third_party/`, `~/.cargo/registry/src/*/pbkdf2-0.12*/`,
`~/.cargo/registry/src/*/bip39-2*/`. Remote: the BIPs repository (BIP-174,
BIP-93, BIP-39, BIP-379/miniscript), RFC 8018, `bitcoin.sipa.be/miniscript`.
Do not answer a protocol question from memory alone.

## Questions to answer -- for each, either construct the counterexample or write "sound" with the reason

Q1 **The hardened method.** X = PBKDF2-HMAC-SHA256(password = phrase bytes,
   salt = "ms-hashlock-v1", c = 100,000, dkLen = 32). Is the construction
   sound? Cover: dkLen < hLen (RFC 8018 block count); the fixed ASCII salt and
   its domain separation from BIP-39 (salt "mnemonic" + passphrase, password =
   the sentence) and from `me`'s sealed-payload PBKDF2 (random salt, 300,000);
   whether an operator using the SAME text as a BIP-39 passphrase or as a
   sealed-payload passphrase and as a hashlock phrase leaks anything across
   them; the iteration count against the "signer at 1/10th of 9,715 it/s"
   budget; and a defensible guesses-per-second figure for PBKDF2-SHA256 at
   100,000 on a current GPU, so the 20-character warning floor in section 5
   rests on a number. State the entropy per character you assume.

Q2 **The plain-sha256 method as an operator choice (L5).** X = SHA-256(phrase
   bytes). What must its warning say that the hardened one's need not? Can the
   two methods ever produce related outputs for one phrase (they must not)?
   Is "if unsure later, try both" sound advice or does it hide a failure
   mode? Is an announced default of `hardened` (section 5) the right default,
   given L3's requirement that the method be backed up outside the tool?

Q3 **Script and PSBT facts.** (a) Miniscript `sha256(H)` requires the witness
   to reveal exactly 32 bytes X with SHA256(X) = H -- cite the compiled script
   and the source. (b) The composer composes the `sha256` fragment only; confirm
   H = single SHA-256 of X, never hash256 (double SHA-256), and that the same
   holds in a tapscript leaf. (c) Section 3.5 claims no hardware signer sees a
   hashlock phrase today and that the coordinator supplies X through the PSBT
   sha256-preimage field. Cite BIP-174's field, and find a counterexample if
   any signer or wallet (Liana, Sparrow, HWI, Ledger, Coldcard, Specter) DOES
   derive or consume a preimage differently. (d) After a spend, is X public in
   every script type the composer emits (wsh, sh(wsh), tr leaf)?

Q4 **The ms1 kind byte `0x03`.** Against BIP-93 codex32 and the ms1 spec:
   payload `[0x03][X:32]` = 33 bytes -> a 75-character string; the share axis
   (MIGRATION.md item 1: a distributed share's first payload byte is a Lagrange
   output, not a prefix); singles keep the legacy `entr` id for a non-entr
   kind (mnem already does this). Is there any input under which a reader
   confuses a preimage with entropy, or a share of one with a single? Is the
   fixed-size `[u8; 32]` API sound with the codec's zeroize contract? Does an
   OLDER reader (ms-cli 0.17.x, a flashed SH2 before the port) fail SAFELY on a
   `0x03` string -- trace the arm you cite.

Q5 **The phrase rule.** Printable ASCII 0x20..0x7E only, at most 100
   characters, bytes exact (no trimming, case folding or Unicode
   normalisation), exactly one trailing LF or CRLF stripped from stdin and
   files. Construct a phrase that passes on one side (host or device) and not
   the other, or that derives differently by any route the record allows. Two
   specific cases: an operator who pastes 64 hex characters INTO THE PHRASE
   slot (deriving a different X from what they meant) -- refusal, warning, or
   nothing; and leading/trailing spaces.

Q6 **Threat model completeness.** Section 3.7 (a spent preimage is public;
   one phrase per policy) was added by the controller after the operator's
   rulings -- confirm, sharpen, or refute it, and name anything else the record
   omits: the preimage ms1 plate as a bearer secret on a keyless wsh path
   (C22); the digest H on a public plate as the guessing oracle; `--random`'s
   entropy source (`getrandom`); the 10-second on-device derivation with the
   phrase in RAM (secret-handling class, Minor at most); F-132's wording.

Q7 **F-467.** Confirm or refute that the hashvault journey's three hashlocks
   (sha256 of 40-, 38- and 34-byte phrases, hashed once) are unspendable under
   the compiled `sha256(H)` script.

Q8 Anything else, strictly within the brainstorm's scope, that a Bitcoin
   programmer would refuse to sign off on.

## Severity

- **Critical**: the tool as designed would produce a wrong result, an
  unspendable policy, an anyone-can-spend path, funds loss, or a claim about
  Bitcoin that the authoritative text contradicts in a way that changes the
  design.
- **Important**: a real defect, a missing case, an unsound assumption that
  must be resolved before the spec is written.
- **Minor / Nit**: recorded, not gating.

## Output -- write this file as your FINAL action, then return a 5-line summary plus the path

`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`

Structure, exactly:

1. Header: date, model, the brainstorm record's git blob or commit you read
   (`git -C /scratch/code/shibboleth/mnemonic-engrave log -1 --format=%h -- design/BRAINSTORM_hashlock_phrase.md`),
   the other files read.
2. One counts line: `C:<n> I:<n> M:<n> N:<n>`.
3. Findings, numbered C-1.., I-1.., M-1.., N-1.., each with: **claim** (one
   sentence); **evidence** (file:line or URL + quoted text); **counterexample
   or construction** (concrete input -> concrete wrong outcome); **remedy**
   (non-authoritative; the controller decides).
4. "Confirmed sound" -- one line per Q1-Q8 item you cleared, with the reason
   and citation.
5. "Questions for the operator" -- judgement calls with no construction.
6. "Sources consulted" -- every file:line and URL.

Rules: write nothing anywhere else; do not edit the brainstorm record; never
read any `*.jsonl` agent output; do not implement, prototype or run cargo
builds (reading and small `python3`/`openssl` checks of a hash value are
fine and encouraged -- paste the command and its output when you do).
