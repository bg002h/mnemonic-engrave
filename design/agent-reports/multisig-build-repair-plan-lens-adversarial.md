# Adversarial-lens review — IMPLEMENTATION_PLAN_multisig_build_repair.md

Reviewer: independent adversarial lens (fable), 2026-08-13. Assume a hostile
actor. Plan at `design/IMPLEMENTATION_PLAN_multisig_build_repair.md`; SPECs
`SPEC_multisig_build_repair.md` (GREEN) and `SPEC_systemwide_payloads.md`;
source `/scratch/code/shibboleth/seedhammer` @ `a10d007` (read this session,
not from a report). Nine prior rounds are honest-error; none is repeated here.
Both repos left clean; nothing committed to the fork.

## Verdict

**RED under a threat model — 0 Critical, 2 Important, 3 Minor.** No clean
funds-guaranteed Critical: the design's real backstop — verifying the assembled
descriptor's keys at an external coordinator before funding — genuinely catches
the money-losing attacks, and the plan mandates it. What is broken is the
*on-device* guidance around that backstop. In phase 1 the payload is the ONLY
cosigner source, the pre-engrave review omits the keys, fingerprints are
omitted by default, and when they are included they are **card-self-declared and
unbound to the key** — so the plan's own EXPERIMENTAL warning points a diligent
operator at a check an attacker forges for free. The safety case survives a
careful operator who ignores the device and compares full keys off-device; it
does not survive one who follows the device's instructions.

## Ranked attack table

| # | attack | prereq | damage | sev |
| --- | --- | --- | --- | --- |
| A1 | Cosigner-key substitution in the sole (payload) source; on-device review shows no keys, and the warning's fingerprint check is forgeable | swap ≥1 cosigner card before pack / in transit; know victim's expected fp (usually public) | funds loss (attacker reaches quorum) or locked funds | **Important** |
| A2 | Oracle laundering: S0 pins the byte-identity oracle "by version", spoofable by a substituted binary / PATH | compromise the dev build env | a device derivation defect ships past the whole gate spine | **Important** |
| A3 | Over-supply "selection step" arm forces a choice among indistinguishable cards | pad the payload with an extra attacker card | operator selects the attacker's card | Minor |
| A4 | `PublicDataHash` record-count is a single truncated byte in a funds digest | — | none reachable (join covers it); hardening only | Minor |
| A5 | Plaintext-payload digest is worthless if delivered on the attacker's channel | operator obtains payload + digest from one attacker-controlled channel | tamper undetected | Minor (documented assumption) |

---

## A1 (Important) — the cosigner set has no attacker-resistant on-device check, and the warning misdirects to a forgeable one

**Attacker's position.** Controls one or more cosigner `mk1` cards that will
land in the operator's payload. This is *cheaper than before*: the spec (§3.1)
moves ALL cosigners from tag-in-hand NFC to a flash file, so the bar drops from
"forge/swap a physical tag at the moment of building" to "swap a record in a
file the operator packs, or MITM a plaintext payload in transit." To also fool
an fp-included build the attacker needs the victim's expected 4-byte master
fingerprints — routinely published in coordinator setups, or seen once.

**The concrete sequence.**
1. Attacker replaces a cosigner's `mk1` with one carrying an xpub *they* control,
   and — because `mk.Card.Fingerprint` is a free wire field (`mk/mk.go:136`,
   set verbatim at decode `mk/mk.go:286`), never derived from the xpub (it
   cannot be: the account xpub is chaincode‖pubkey only, F-130) — sets the
   card's `Fingerprint` to the victim's expected value.
2. Operator packs/loads the payload. For the realistic plaintext container
   (cosigner cards are NOT a secret class, so operators use plaintext), the only
   authentication is the operator comparing the displayed digest
   (`gui/sysw_load.go:164-176`). The digest is computed over the attacker's
   bytes (`sysw.PublicDataHash`), so it **matches** whatever `me sysw pack`
   printed for that same file.
3. Build proceeds. `assembleBuildPolicy` places the attacker's cosigner in its
   slot verbatim; `cosignerFromCard(card, includeFp)` copies the attacker's
   declared fingerprint straight through (`gui/multisig_build.go:437-455`).
4. Operator engraves, then funds.

**What the operator sees, and why it does not stop them.** The pre-engrave
policy review (`buildReviewLines`, `gui/multisig_build.go:513-531`) shows **only
the 4-byte policy stub and per-slot `fp`/`(no fp)`** — never the xpubs. With the
default fp choice (`multisigFpChoices` index 0 = "No (omit)",
`gui/multisig_build.go:334`) every slot reads `@N (no fp)`, so there is nothing
to check. With fp included, every slot reads the attacker's *forged* fingerprint,
which matches the operator's expectation. The `EXPERIMENTAL` warning
(`gui/multisig_build.go:229-232`) then instructs: "verify the assembled
descriptor **and the shown policy stub + per-slot fingerprints** against your
coordinator." An operator who verifies the fingerprints — exactly what the device
tells them to do — is fooled, because a matching fingerprint proves nothing about
the key behind it. The stub is a hash *of the attacker's policy*, so it is
self-consistent and equally uninformative.

**Which gate should have caught it, and why it does not.**
- Digest comparison authenticates *bytes vs. the host's printed value*, not
  intent — if the attacker authored/edited the bytes before pack, it matches.
- The seed↔key gate (§4.3) fires ONLY on a `both` slot (the operator's own key);
  a pure cosigner (`payloadKey`) slot is never derivation-checked — correctly,
  the device has no seed for it. So the gate is silent here by design.
- The duplicate-key refusal only catches an *identical* 65-byte key.
- The only real control — comparing the descriptor's **keys** at a coordinator —
  is named in the warning but buried beside a forgeable fingerprint check, and
  the keys themselves appear on-device only in the *post-engrave* restore doc
  (`multisigRestoreLines` renders `desc.Encode()`, `gui/multisig_restore.go`),
  never in the pre-engrave review the operator acts on.

**The bounded fix (plan text).**
1. S1/S4 review + `buildReviewLines`: show a per-slot value the operator can
   reproduce from the cosigner's own key — the full account xpub, or a labelled
   hash of chaincode‖pubkey — not just stub+fp. Keys, not fingerprints, are the
   only attacker-resistant handle.
2. Rewrite the `EXPERIMENTAL` warning (S2/S5): state plainly that **a matching
   fingerprint does NOT verify a key** (the fingerprint is card-declared and
   unbound), and that the operator MUST compare the descriptor's KEYS against an
   *independently obtained* cosigner list before funding. Drop "per-slot
   fingerprints" as a verification instruction.
3. Add a note to the plan (and cross-ref SYSW §5.3): the plaintext cosigner
   payload's digest is only as strong as out-of-band delivery, and the SEALED
   variant is the recommended carrier when the delivery channel is untrusted —
   its AEAD binds the public section as AAD (`sysw/open.go:30`) and a
   downgrade-to-plaintext changes the digest, so an in-transit swap is detected
   without relying on the operator. Note explicitly that "Include fingerprints"
   is an identity/coordinator-matching convenience, **not** a security control.

---

## A2 (Important) — the byte-identity oracle is pinned "by version", which a substituted binary spoofs

**Attacker's position.** Can influence the developer/CI build environment that
runs the S0–S6 gates — a poisoned `PATH`, a trojaned `md`/`mk`/`ms` release, or
a shadowed binary that reports the expected `--version` string but emits
attacker-chosen output. Higher prerequisite than A1 (dev-env compromise), but it
targets the plan's *entire* safety spine.

**The concrete sequence.** S0 deliverable 1 says the walk script "resolves the
primary toolchain **by version**" and "prints the resolved oracle versions."
Version resolution keys on a string the binary itself prints. A shadow binary
that prints `md-codec 0.42.0` and produces output engineered to AGREE with a
malicious device change makes every byte-identity gate (S2, S5) green. The plan
correctly forbids the *vendored fork testdata* oracle and re-pins it (S0-4), but
says nothing that stops a substituted *primary* binary — the oracle it actively
trusts.

**What the operator/reviewer sees.** Green gates and a printed version line that
the malicious binary chose. Nothing distinguishes it from the real primary.

**Which gate should have caught it.** S0 is the stage every later gate trusts,
and §5's blind-spot list names cite-rot but not oracle-substitution.

**The bounded fix.** S0 deliverable 1: pin the oracle by **source, not version
string** — build the primary from a known commit of the pinned repo (or verify
the binary's hash against a recorded value), and print that commit/hash (not just
`--version`) into every gate record. The adjudication rule already assumes a
trusted primary; make "trusted" mean pinned-and-hashed rather than
whatever-answers-`--version`.

---

## A3 (Minor) — the over-supply "selection step" arm asks the operator to choose among indistinguishable cards

Spec P0 item 4 permits "a selection step **or** a named refusal" when the payload
holds more matching cards than open slots. The plan's own test leans safe
(`TestBuildRefusesMoreCardsThanOpenSlots`, S1 test 6), but the arm is not ruled
out. Adversarially, a selection step is the dangerous arm: an attacker pads the
payload with an extra card, and — fingerprints omitted or forged (A1) — the
operator cannot distinguish the attacker's card from the real one at the
selector. **Fix:** rule the selection arm OUT in the plan for exactly this reason;
require exactly `n-1` mk1 cards or a named refusal. (Largely already the plan's
posture; make it explicit and adversarially justified.)

## A4 (Minor) — `PublicDataHash` truncates the record count to one byte

`sysw/pubhash.go:33` writes `byte(len(records))` into a funds-sensitive digest.
Not independently exploitable — `strings.Join(records,"\n")` already makes any
record add/remove visible, and records cannot contain `\n` — so a count-mod-256
collision cannot be reached without also changing the join. Recorded as a
hardening note only: a length field in an authentication digest should not
silently wrap. Drop it as redundant or widen it.

## A5 (Minor) — the plaintext digest defends nothing if delivered on the attacker's channel

Inherent and documented (EPD §6.6 calls it "out-of-band"), restated because the
multisig context makes it sharp: the operator-compared digest is only a control
if the digest reaches the operator on a channel the attacker does not also
control. In a coordinated multisig the payload is often assembled and forwarded
by one party. Fold one sentence into the plan/operator-doc: the digest must be
obtained independently of the payload, or the payload must be sealed.

---

## Adequately defended — probed and sound

- **Swap between digest comparison and use (TOCTOU on flash).** Sound. The region
  is read once into RAM (`gui/sysw_load.go:56`); identity, `sysw.Open`, the
  digest, and the stored records all derive from that one buffer, and consumption
  (`take`) reads the in-RAM `records`, never re-reads flash. A post-load flash
  swap cannot affect a live session, and a re-read re-computes identity and
  re-earns `[compared]`. The firmware never writes flash and a host write needs
  BOOTSEL (firmware halted), so there is no concurrent-write window either.
- **Unauthenticated payload reaching the constructor.** Sound, defense-in-depth.
  `take`/`takeAll` refuse while `!compared` (`gui/sysw_session.go:114-117`), AND
  a session that fails the comparison is unloaded outright (`ctx.sysw = nil`,
  `gui/sysw_load.go:186-201`) — the only `.load()` caller — so an uncompared
  session never survives to be consumed. The plan's `takeAll` compared-gate is
  correct and belt-and-suspenders.
- **The seed↔key gate on `both` slots.** Cryptographically sound. It derives the
  operator's seed at the card's declared origin (R-3) and compares
  chaincode‖pubkey (`findUserSlot`, `gui/multisig_match.go:34-51`). An attacker
  cannot forge a card that passes without the operator's seed — that would be an
  EC preimage — and deriving at the card's declared origin rather than the flow's
  does not weaken this (a mismatched origin just fails the derive-compare). R-3
  being "card origin authoritative" costs nothing here.
- **Duplicate-key refusal (65-byte chaincode‖pubkey).** No quorum-degrading
  near-miss. Same pubkey + different chaincode derives DIFFERENT ranged children
  (chaincode feeds derivation), so it is not a collapse; two distinct account
  keys deriving the same child pubkey is an EC collision. The exact 65-byte check
  subsumes every arrival shape (seed+card, card+card, seed+seed).
- **Record ordering "to a hostile end."** No funds impact for phase 1. All three
  templates are `sortedmulti` (`md/encode_multisig.go:27-29`, keys sorted at
  render), so reordering changes only the order-dependent WalletPolicyId/stub,
  never the addresses. (It can force a chosen stub, but the operator matches the
  stub at their coordinator, which sorts identically.)
- **Poisoned vendored vectors.** Defended and the plan knows it: the harness
  refuses to run against vendored fork testdata (S0,
  `TestOracleHarnessRefusesVendoredTestdata`) and re-pins md testdata from the
  primary (S0-4). Poisoning `md/testdata` satisfies no gate. (The live oracle is
  still A2's gap.)
- **Sealed-variant in-transit tampering.** Defended. The AEAD binds header ‖
  public section as AAD (`sysw/open.go:30`), so a swapped public cosigner record
  fails to open, and a downgrade to plaintext changes the digest. The residual is
  purely that operators will pick the plaintext variant for non-secret cosigner
  cards — which is A1's premise, not a sealed-path defect.
```
