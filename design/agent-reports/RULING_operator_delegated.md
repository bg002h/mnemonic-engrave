# RULING — the two operator-delegated decisions (2026-08-22)

Standing in for the operator per their instruction ("Ask fable for blocking
questions"), which lifts the reservation in `RULING_export_deadlock.md` §4.
These two decisions are decided here, and owned here. Nothing below is a
recommendation back to the operator; it is the answer, with the one section of
genuinely-theirs information at the end.

Evidence base: `design/PLAN_wallet_file_export.md`, the five
`design/agent-reports/PLAN_export_*.md` reports, `RULING_export_deadlock.md`,
and — decisive for Decision 1 —
`design/fixtures/reasonably-complex-wallet/README.md` plus the fixture inputs
in `design/journeys/inputs-hashvault/`. The measured facts in the dispatch
(six-binary Core matrix, the minimal pair run twice, the Nunchuk/Sparrow
mechanisms, the version floors) are relied on as given, not re-derived.

---

## RULING 1 — Tier 4 stays keyless. The vault is not changed, now or as a
condition of any export work; descriptor-level import into third-party
wallets is accepted as permanently out of scope for this wallet, and the
`addr()`-list (Phase 1b) is its Core watch route.

### Reasoning

**1. The keyless tier is a feature, and the record proves it three ways.**

- The fixture's own README lists *"a spend path with no key at all (tier 4)"*
  as one of the five properties this wallet exists to exercise — alongside the
  hashlocks, both timelock flavours, and the threshold ladder. It is not an
  oversight that survived review; it is on the wallet's specification sheet.
- The design was chosen **through** an explicit warning, not around one.
  `md encode --experimental` relaxes only `requires_sig` and warns on every
  use; the fixture README restates the consequence in bold: *"whoever learns
  H3's preimage can spend tier 4 alone, so if that preimage is engraved, the
  plate is bearer access."* The operator read that and engraved the plates.
  That is informed consent to a bearer-instrument tier, not an accident.
- The tier heights tell the story. Tier 3 (`@5` alone) opens at absolute
  height 1173520 — roughly five years out — and tier 4 at 1383520, roughly
  nine. This is a degrading estate vault: each tier is a weaker condition that
  activates later, and the terminal tier is *presence of the preimage alone*,
  so an heir needs no key custody, no signing device, no software — a plate
  and a height. Keying tier 4 ("passphrase AND key @6") reintroduces exactly
  the failure mode the terminal tier exists to remove: key management by
  whoever comes last.

**2. Interop was never a design goal; it is a want discovered after the
fact.** The wallet predates the export ask (operator ask dated 2026-08-22;
the vault was engraved, device-proven, and journey-documented before it). No
design artifact anywhere in `design/` conditions the vault on Core, Nunchuk,
or Sparrow compatibility. A post-hoc want does not get to rewrite spending
conditions — especially when the want is already partially served: the
`addr()`-list import is verified working on a real node, so the operator can
*watch* this wallet in Core today. What keying buys on top of that is ranged
derivation and descriptor-level watch, in exactly one target (Core v29+, wsh
form only — tr NUMS is still unrepresentable in Nunchuk, Sparrow still has no
miniscript engine). One importer, at the price of the vault's terminal
property plus a full funds migration. That trade loses on its face.

**3. The wallet already has full first-party coverage.** SeedHammer II
derives its addresses from the keyless template plus six key cards; it is
engraved and restorable. The vault is not stranded — it lives on the platform
that was built for it, and Core can watch it by address list. "It lives on
SeedHammer II and nowhere else" overstates the cost: it lives on SeedHammer II
and is *watchable* anywhere `importdescriptors addr(…)` works.

**4. The third option — a separate keyed interop wallet "for watch purposes"
— is a trap, and here is the mechanism.** A watch wallet is only useful if it
watches the funds. A keyed tier-4 variant is, by the plan's own measurement, a
*different wallet* — different scripts, different policy-id, different
addresses. Importing it into Core watches an address set that holds nothing.
So as stated ("for watch purposes") the third option produces a wallet that
watches nothing while doubling the backup surface. The only coherent version
of it is not a watch route at all but a *second vault* — splitting funds
between an interop-friendly wallet and this one — which is a new
wallet-creation decision with its own custody design, not a variant of this
export cycle, and must never be framed as "the watch export for the rcw".
If someone later proposes "just derive the keyed twin so Core can watch",
this paragraph is the refutation: the twin's addresses are not the vault's.

### What changes if adopted

- **Nothing migrates, nothing re-engraves, no address changes.** The plates,
  the engraved template, and the journey PDFs all remain valid.
- The plan proceeds exactly as ruled in `RULING_export_deadlock.md`: Phase 1
  (G1 `--allow` parity, GREEN gate already closed), Phase 1b (`addr()`-list,
  the one working Core route), Phase 3 (Sparrow-refusal regression test).
- The "documentation of the impossibility" deliverable gains one sentence:
  *ruled deliberate 2026-08-22 — tier 4 stays keyless; descriptor-level
  import of this vault is permanently out of scope, by decision rather than
  by omission.* Future sessions cite this file instead of re-running the
  five-agent discovery.
- The operational cost accepted with this ruling, stated so it is owned: the
  Core watch artifact is a **fixed address list** — re-export is required
  beyond the exported gap, and the artifact must keep stating that in-band
  (already an acceptance bullet in Phase 1b).

---

## RULING 2 — Hot export is NOT NOW: do not build it in this cycle or the
next, and build it only on the named trigger below — a measured import
acceptance by a real target for a concretely named wallet, plus a renewed
operator ask for that wallet — with "never" explicitly rejected.

### Reasoning

- **"Now" is dead, and Ruling 1 killed it twice over.** Hot export for this
  wallet has no consumer: Core refuses the descriptor hot and watch alike,
  because the sigless rule fires before signing ability is even considered.
  With tier 4 ruled permanently keyless, that refusal is permanent for this
  vault — not pending, not version-gated. Building `export-signer` today
  produces the single most dangerous artifact in the plan (spendable key
  material on disk, new attack surface, exists nowhere in the constellation)
  with **zero** function. Nothing can import what it writes.
- **"Never" is wrong too.** The contract is already ruled sound (distinct
  `mnemonic export-signer`, account-level xprvs — master xprvs trip the
  `PubkeyProvider::operator<` duplicate-key false positive on v29/v31.1 —
  `--output` required, `0600` + `create_new`, always-on advisory, no
  interactive confirm). The toolkit serves more wallets than this vault, and
  an ordinary keyed wallet hot-loaded into Core v25+ is a legitimate,
  measured-to-work use. Foreclosing that forever because *this* wallet cannot
  use it would confuse one vault's design with the constellation's surface.

### The trigger, stated so a future session recognises it without
re-litigation

Build Phase 4 (`mnemonic export-signer`, under the PLAN_export_cli_surface §3
contract as already ruled — the trigger does not reopen the contract) when
**both** of the following hold:

1. **A named wallet with a measured consumer.** A specific wallet is named,
   and a regtest `importdescriptors` (or the target's equivalent) of **that
   wallet's descriptor with private key material** returns per-entry
   `success: true` on a pinned target version. An IMPORT test, not an emit
   test — the same C1 standard the plan already binds watch exports to. This
   is machine-checkable in an afternoon and cannot fire for the rcw while
   tier 4 is keyless, which is now permanent; for any ordinary keyed wallet
   it fires trivially.
2. **The operator renews the ask against that wallet.** One sentence naming
   the wallet and the hot-load purpose. The 2026-08-22 ask does not carry
   forward — it was made for a wallet that has no consumer, and a standing
   ask with no object must not be inherited by the first wallet that happens
   to qualify.

Both conditions, not either: (1) without (2) is tooling nobody asked for that
writes keys to disk; (2) without (1) is the current deadlock again. When both
fire, implementation is still risk-set (b) work and takes the standard R0
gate; the trigger opens the *decision*, not a bypass.

Anti-re-litigation note: if a future session finds this trigger satisfied, it
should cite this file and proceed to R0 — it does not need to re-ask whether
hot export "should ever" be built. That question is answered: yes, on the
trigger, under the frozen contract.

---

## What the operator should know that could not be decided for them

1. **The rcw's seeds AND preimages are in this repo in plaintext**
   (`design/journeys/inputs-hashvault/seeds/key-{0..5}.seed`,
   `design/journeys/inputs-hashvault/preimages/preimage-{0..2}.txt`). The
   seeds are recognisable test vectors (`abandon ×23 + checksum word`) and
   the preimages are demo phrases, so the fixture instantiation is
   test-material — Ruling 1 was made robust to both readings (fixture and
   funds-bearing) and stands either way. But one check is theirs alone:
   **if any physically engraved plate carries one of these three repo
   preimages while any real funds ever sit behind the corresponding hash,
   that plate is published bearer access** — an emergency, not a design
   question. If every real instantiation uses private preimages of the same
   shape, there is nothing to do.
2. **After height 1383520, plate custody IS the vault's security.** Keeping
   tier 4 keyless (Ruling 1) means that from tier 4's activation onward, the
   H3 preimage — memorised or engraved — spends alone. That is the designed
   property, and it puts the plate's physical security on the same footing as
   a signing key. If that ever becomes uncomfortable as the height
   approaches, the remedy is a deliberate wallet redesign run as its own
   funds-migration project, not a rider on an export cycle.
3. **If descriptor-level Core watch is ever wanted badly enough to migrate
   anyway**, the measured floor is Core v29+ (multipath wsh, keyed tier 4) —
   v24-v28 cannot load even the keyed multipath form. Any estate document
   that assumes "load the descriptor into Bitcoin Core" must pin that
   version floor, or it is describing a procedure that fails on the node the
   heir is most likely to have.
4. **Open Q3 in the plan** (name the flag `--allow` vs `--experimental`)
   remains the one line of genuinely optional operator input in this cycle;
   the plan's `--allow` choice proceeds unless they say otherwise. It is not
   blocking and was not delegated, so it is noted, not ruled.
