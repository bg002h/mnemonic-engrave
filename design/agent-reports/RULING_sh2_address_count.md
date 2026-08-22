# RULING: device address-proof count stays at 2 per chain

**Date:** 2026-08-22
**Decision requested:** the operator asked for a journey showing the first 5 receive + 5 change addresses for the first 2 accounts, computed on the host AND on the SeedHammer II. The device shows 2 per chain (`addrProofPerChain = 2`, plan D6), and a test pins that. Change the device, make it configurable, or keep it and have the journey compare host-5 against device-2?

## THE RULING

**Leave `addrProofPerChain = 2`. Plan D6 stands. The journey shows host-5 vs device-2 and asserts the device's four addresses byte-equal the host's indices 0–1 of each chain — the device proves a SAMPLE, the host proves the RANGE, and the journey says so honestly.**

## Reasoning

**1. Indices 2–4 add no evidence against any realistic failure.** Every failure
mode this proof exists to catch — wrong key seated, wrong script template,
wrong `sortedmulti` ordering, wrong change path, host and device holding
different policies — diverges at **index 0**. Non-hardened BIP32 children are
not predictable from earlier siblings without the xpub: a device that produces
the correct addresses at receive 0–1 and change 0–1 holds the correct policy,
the correct keys, and a working deriver. There is no accidental mismatch class
that agrees at 0–1 and diverges at 2–4 short of a derivation function
conditioned on the index itself.

**2. Against a deliberate index-conditioned backdoor, 5 buys exactly nothing.**
The screen's count is a compiled-in constant an attacker can read. A deriver
backdoored to go wrong at index ≥ 2 defeats a 2-address screen; the same
backdoor conditioned at index ≥ 5 defeats a 5-address screen identically.
Raising the constant does not shrink this class — it only moves the threshold
the attacker copies out of the source. The correct control for index-dependent
divergence is machine comparison at indices no human screen will ever show,
which is what the Rust-pinned vectors in `gui/wallet_policy_test.go` already do
structurally (device addresses asserted against primary-Rust derivations), and
which the journey's host-side `md address --count 5` extends across the air gap
at indices 0–4.

**3. The consent surface degrades with length; D6 reasoned this and was
right.** The consent screen is what the operator reads before authorising an
irreversible engrave. Four addresses is a real comparison a person performs;
twenty (5 × 2 chains × 2 accounts) is a scroll-past. A proof nobody verifies
is worse than a shorter one people do — it *feels* like more assurance while
delivering less. D6's trade ("prove both chains derive beats proving one chain
five times") is the correct allocation of the operator's finite attention: the
change chain is where a mismatch silently loses funds, and D6 spends the
budget there.

**4. A fixed shape is itself a safety property.** The codebase already treats
"I did not see any addresses" as the observation that must stop an operator
(`walletPolicyAddressLines` never returns empty, by design). That only works
if the operator knows what a correct screen looks like. A constant screen —
always Receive 0, Receive 1, Change 0, Change 1 — is learnable; a configurable
one makes "how many addresses should I see?" unanswerable and turns a
truncated or tampered screen into something plausible. This kills option 2
outright: per-invocation configurability adds a surface, a cross-device
inconsistency vector, and destroys the learned-shape signal, for zero
evidentiary gain (see point 2).

**5. "It's what the operator asked for" is not, by itself, an override of a
ruled design.** D6 was decided with exactly this trade in view — the constant's
comment records it, and the pin test exists to force this conversation rather
than allow a casual bump. The conversation has now happened: the request is
satisfiable at full strength on the host and at sample strength on the device,
and the delta between those two strengths is (per points 1–2) not material.
Nothing about the request reveals a fact D6 lacked. So D6 is **affirmed, not
overridden**, and the honest-labelling requirement below is what reconciles
the journey with it.

## What the implementer changes

**Nothing in the fork's GUI code or tests.**

- `/scratch/code/shibboleth/seedhammer/gui/wallet_policy.go` — untouched.
  `addrProofPerChain = 2` (line 184) and the D6 comment stand as written.
- `/scratch/code/shibboleth/seedhammer/gui/wallet_policy_test.go` — untouched.
  The pin at lines 33–35 and the deliberate literal `2` at line 75 both stay.
  (For the record: that literal exists so the assertion does not move with the
  constant under test — halving the constant to 1 passed the looped form
  unchanged. Anyone changing the constant later must change the pin, the
  literal, AND supply Rust-derived vector addresses for the new indices. None
  of that happens under this ruling.)
- **No Rust change anywhere.** This is fork-native GUI/UX with no Rust
  counterpart — explicitly exempt from the Rust-first rule. Do not go looking
  for a primary-Rust change; there is none to make.

**The journey** (new artifact under
`/scratch/code/shibboleth/mnemonic-engrave/design/journeys/`, following the
existing `build_pdf_*` / `capture_*` pattern):

1. **Host side:** for each of the 2 accounts, run
   `md address --chain <N> --count 5` for receive and change — 20 addresses
   total, recorded in the journey artifact.
2. **Device side:** load each account's card; capture the consent screen —
   Receive 0, Receive 1, Change 0, Change 1 per account (8 addresses total).
3. **The assertion, machine-checked, per account:** the device's four
   addresses are **byte-identical** to the host's indices 0 and 1 of the
   corresponding chain. Subset equality at fixed indices — not "some overlap".
   Any mismatch fails the journey.
4. **The journey text states the asymmetry plainly**, in roughly these words:
   *"The device proves a SAMPLE (indices 0–1, both chains); the host proves
   the RANGE (0–4). Agreement on the sample pins the policy, the keys, and
   the deriver; a divergence confined to indices ≥ 2 would require an
   index-conditioned derivation defect, a class covered by Rust-pinned vectors
   in the fork's test suite, not by a longer consent screen."*

**Optional follow-up (Minor, non-gating, file with an owning phase per the
FOLLOWUPS discipline):** add one high-index Rust-pinned assertion (e.g. index
1000, both chains) to `TestWalletPolicyConsentProvesTheWallet`'s vector set via
`vectorAddress`, exercising the deriver at an index no screen shows. That is
the mechanically correct home for the "diverges only at high indices" concern
— CI eyes, not operator eyes.

## What to tell the operator

You asked for 5 + 5 on both sides. You are getting 5 + 5 on the host and the
existing 2 + 2 on the device, with the journey machine-checking that the
device's sample matches the host's range where they overlap.

**What you give up:** the device screen never displays indices 2–4, so the
on-device confirmation is a sample, not the full requested range.

**Why that costs you nothing real:** every accidental mismatch — wrong policy,
wrong key, wrong ordering, wrong change path — shows up at index 0. The only
class a longer screen would address is a deriver that goes wrong specifically
at higher indices, and since the screen's count is a constant in public
source, an adversary who could build that conditions it just past whatever the
constant is — 5 defends no better than 2. That class is instead covered where
it belongs: cross-implementation vectors compared by machines at indices no
one would put on a screen.

**What you keep:** a consent screen short enough that you actually read it
before an irreversible engrave, and a fixed screen shape — so "the screen
looked different this time" remains an observation you can act on. Plan D6
chose the change chain over more receive indices because the change chain is
where a mismatch silently loses funds; that reasoning survived this
re-examination intact.
