# RULING — the wallet-export deadlock (2026-08-22)

Standing in for the operator on one blocking decision, per dispatch. This rules
on tooling; it explicitly does NOT rule on the wallet itself — see §4.

## THE RULING

**Keep the wallet exactly as it is; ship the `addr()`-list Core watch export
(option 2) plus G1 on its own parity merits (option 4); build no hot export
now; and put option 1 — keying tier 4 — in front of the operator as a
question, decided by no one but them.**

## Reasoning

1. **Option 1 is not on the table for an agent.** It is measured to work, and
   it is still not a repair — it is a different wallet. Different addresses
   (funds would have to migrate), and a different contract: tier 4 stops being
   "whoever holds the passphrase, after the block height" and becomes "whoever
   holds the passphrase AND key @6". The keyless tier is the wallet's one
   deliberately key-free escape hatch — plausibly its whole reason to exist —
   and the operator chose it on purpose. Changing spending conditions on a
   funds-bearing vault is risk-set (b) work and, more fundamentally, it is
   authorship of the operator's security policy. A stand-in can recommend it be
   *asked*; only the operator can *answer* it.

2. **Option 3 underdelivers against a measured, cheap win.** The `addr()`
   import path is verified working on a real node (PLAN_export_bitcoin_core.md
   §3.1). "Build nothing" would leave a working, honest Core watch route on
   the floor because the perfect route is impossible. Watching the real wallet
   in Core has standalone value: balance monitoring, receive verification
   against SeedHammer-derived addresses, estate rehearsal.

3. **Option 2 is the honest maximum.** Every descriptor-level route to all
   three targets is closed by the same non-waivable rule, verified by execution
   on six Core binaries, libnunchuk's own checker, and Sparrow's source. No
   tooling we write changes that. The `addr()` list is the entire reachable
   surface for Core, so ship it and say plainly what it is: a fixed list, no
   ranged derivation, re-export needed beyond the exported gap, and the export
   artifact must state its own address count and that limitation in-band.

4. **Sparrow gets a refusal, not an export.** The wsh form silently misimports
   as a wrong-address `sortedmulti` — a funds-loss trap. Phase 3's regression
   test pinning our refusal (making the incidental safety deliberate) proceeds.
   Nunchuk gets nothing new, per the plan: `descriptor`/`bsms` already emit its
   import shapes, and Nunchuk refuses this wallet regardless.

## What gets built, what does not

**Build (in this order):**
- **Phase 1 / G1** — `--allow sigless-branch` parity on `export-wallet`,
  through the R0 gate as planned. Worth doing *on its own merits*, separated
  from the import goal it does not achieve: (a) parity — `build-descriptor` and
  `md encode` already expose the relaxation, and the surface disagreement is a
  trap; (b) it unblocks `--format descriptor`/`bsms` emission of the tr form
  for inspection, archival, and any future target that is saner than today's
  three; (c) the never-silent `emit_allow_notes` warning surface makes it safe.
  Constraint: no help text, doc, or commit message may say it "enables export
  to Core/Nunchuk/Sparrow". It enables *emission*. The targets still refuse.
- **The `addr()`-list export** — `--format bitcoin-core-addresses` per
  PLAN_export_bitcoin_core.md's recommendation: N non-ranged `addr(…)#checksum`
  `importdescriptors` entries, with the gap-limit / no-derivation caveat stated
  in the artifact itself.
- **Phase 3** — the Sparrow-refusal regression test.
- **Documentation of the impossibility, as a deliverable** — one section
  stating the measured minimal pair (tier 4 keyless refuses; tier 4 keyed
  accepts; nothing else matters), so the next person does not re-spend five
  agents rediscovering it.

**Do not build:**
- **Option 1.** Not ours to build; see §4.
- **Hot export (G3 / Phase 4), now or as part of this cycle.** Three reasons,
  in ascending order: (a) it writes spendable key material to disk and was one
  clause of a long request — the plan already gates it on an explicit,
  separate operator go-ahead, and that gate stands; (b) hot export exists
  nowhere in the constellation today, so it is new attack surface, not parity;
  (c) decisive: **it has no consumer for this wallet.** Core refuses the
  descriptor watch *and hot* alike — the sigless rule is checked before
  signing ability matters — so an `export-signer` output for this wallet
  imports into nothing. Building it now produces the most dangerous artifact
  in the plan with zero function. If the operator later ratifies a keyed
  tier 4 (or a different wallet), revisit under the CLI-surface report's §3
  contract: distinct `mnemonic export-signer` subcommand, account-level xprvs
  (never master — the `PubkeyProvider::operator<` duplicate-key false
  positive, reproduced on v29/v31.1), `0600` + `create_new`, always-on
  advisory. That contract is ruled sound; only its trigger is withheld.
- **A `nunchuk` emitter.** Stays deleted per the plan.

## What needs the operator's own signature

Two things, and nothing proceeds on either without it:

1. **Keying tier 4 (option 1).** This is a change to the vault's spending
   conditions and address set — the operator's security design, not a tool.
   Present it as a question with the measured pair attached: *as designed, no
   third-party wallet will ever import this vault at descriptor level; with a
   key on tier 4, Core v29+ imports the wsh form; the price is the passphrase-
   only escape hatch and a funds migration to new addresses.* Recommend
   neither direction. The keyless tier looks deliberate, and interop was
   plausibly the accepted cost; only the operator knows.
2. **Hot export (Phase 4).** Even after any wallet change, the plan's explicit
   go-ahead gate stands unchanged.

## One sentence for the operator

"Your vault is fine and SeedHammer covers it fully — the reason Nunchuk,
Sparrow and Core all refuse it is precisely the tier-4 passphrase-only escape
hatch you chose, so we're shipping the one thing that works (a watch-only
address list for Core, plus emit-side parity), and whether to trade that
escape hatch for importability by putting a key on tier 4 is a wallet-design
decision that's yours alone — say the word either way."
