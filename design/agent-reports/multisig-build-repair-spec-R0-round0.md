# R0 round 0 — SPEC_multisig_build_repair.md

Reviewer: independent architect (fable), 2026-08-13. Source examined at
`/scratch/code/shibboleth/seedhammer` @ `a10d007` (confirmed by `git log`).
Scope per brief: §4.3 gate soundness, §4.1/§4.2 multi-slot + lifetime, R-3,
can-a-user-do-the-thing, claim-vs-line, staging soundness. Operator decisions
1–4 and the phase boundary were treated as inputs and are not re-litigated.

## Verdict

**RED 2C / 7I**

## Findings table

| id | sev | section(s) | one line |
| --- | --- | --- | --- |
| C1 | Critical | §4.1, §4.3, R-2 | The key-reuse rule discriminates on the wrong variable: it admits a duplicate-key wallet (2-of-3 that is really 1-of-2, on steel) and refuses the legitimate multi-account wallet P3 exists to deliver |
| C2 | Critical | §4.1, §6 P3, §4.5, P5 | The flow tail (steps 4–9: leg derivation, ms1, engrave) is never respecified for multi-slot / multi-seed / divergent origins; a concrete input yields a "Full (seed + keys)" engrave that omits a needed master → unspendable backup; no specified gate can see it |
| I1 | Important | §4.3, §6 P3/P4, §10 Q3/Q5 | The gate "fires on assignment" but no stage defines an assignment mechanism; slot/seed/account binding is parked in open questions the stages depend on |
| I2 | Important | §6 P0/P1 | P0's close gate (completed engrave) and P1's premise (D-1 unreproduced, unfixed) cannot both hold; the stage split is unsound as written |
| I3 | Important | §1, §5.1, §5.4, R-1 | Normative text names NFC as a phase-1 source in three places, against §3's exclusion; §5.1's "exactly two sources" contradicts the shipped seam's SCAN row on NFC-featured hardware |
| I4 | Important | §4.2, §5.2, R-1, §6 P4 | Current-state mischaracterization: TYPED-ONLY is already retired mechanically at all four cited sites — `seedEntryFlow` → `syswSeedPicker` already offers PAYLOAD (and SCAN) for the build flow's self seed today |
| I5 | Important | §6 P0 | "Feed every ClassMDMK record" + the exact-count/no-md1 refusal dead-ends legitimate payloads; slot order = payload record order is identity-bearing and never stated |
| I6 | Important | §4.2, SYSW§3.2.1 | §4.2's lifetime MUSTs are unsatisfiable for payload-sourced seeds: the sysw session retains the record for the process lifetime by SYSW ruling, reachable by other programs |
| I7 | Important | §4.1, §4.3 | Per-seed passphrase binding is unspecified for multi-seed flows; a hoisted single passphrase mints a wrong self key that no specified check can catch |
| M1 | Minor | §4.1 | A depth-0 cosigner card (`Path == "m"`) hits `errMultisigEmptyDivergent` in divergent mode; name the refusal surface |
| M2 | Minor | §6 P0 step 10 | The verify offer readback is NFC-only (`bundleGatherFlow`); the phase-1 emulator walk cannot exercise it — record the blind spot with F-158 |
| M3 | Minor | §2.1 | "reporting `reused []int` when one key occupies several slots" misstates `findUserSlot` — `reused` fires when one **seed** matches ≥2 slots, including the legitimate distinct-origin case (this misreading is C1's root) |

Sound, verified, and worth recording as settled: see "What checked out" at the
end — including that **R-3 is correct** and that `scriptName` has **no
consumers outside `gui`** (resolving §8's flagged unknown).

---

## C1 — the key-reuse rule keys on the wrong discriminator

**Where.** §4.1 bullet 3 ("A seed reused across slots at the SAME origin is a
defect… `findUserSlot` already detects this (`reused []int`) and the
constructor MUST refuse it loudly") and §4.3's NORMATIVE outcomes row 3
("assigned slot, ≥2 **payload keys** match the seed → FAIL LOUDLY — key
reuse").

**The mechanism fact.** `gui/multisig_match.go:34-60`: `findUserSlot` derives
the seed at *each slot's own origin* and collects every match. Its `reused`
therefore fires in **both** of these cases and cannot tell them apart:

- same seed at ≥2 slots under **distinct** origins → **different keys**, the
  legitimate multi-account shape its own doc comment blesses ("the SAME seed
  legitimately appears at >=2 cosigner slots under DISTINCT origins… show a
  notice");
- same seed at ≥2 slots at the **same** origin → **one key twice**, the defect.

§2.1's description ("reporting `reused` when one key occupies several slots")
is the same-origin subcase only; the spec then builds §4.1's refusal on that
misreading.

**False-positive scenario (feature-killing).** The constellation's own
pathological wallet: operator supplies seed for master A; the payload carries
their other account cards A@acct1', A@acct2' plus true cosigners. §4.3 row 3:
two payload keys match the seed → FAIL LOUDLY. §4.1's reading of `reused`:
matches at ≥2 slots → refuse loudly. **Both normative texts refuse the exact
wallet §4.1 exists to make buildable.** P3's flagship feature is dead on
arrival if either is implemented as written.

**False-negative scenario (steel-corrupting).** 2-of-3; operator declares self
slot @0, types seed S; shared origin O = `m/48'/0'/0'/2'`. The payload —
plausibly the card set of a previous wallet — contains the operator's **own**
mk1 (minted from S at O) plus cosigner X. `buildCosignerCards` accepts two
mk1s. Assembly: @0 = derive(S,O) = K, @1 = K (the card), @2 = X. Gate walk:
row 1 — the seed-held slot trivially matches; row 3 — only **one** payload key
matches the seed, `≥2` is false; row 5 — X is unrelated. **Proceeds.** The
engraved `sortedmulti(2, K, K, X)` is satisfiable by the operator alone (two
signatures under K match the duplicated pubkey), so the displayed 2-of-3 is
a 1-of-2. Quorum degradation on steel; unmet stated guarantee ("the
constructor MUST refuse it loudly").

**Also internally inconsistent.** Row 3 counts *payload keys matching the
seed*, which requires deriving-and-comparing **unassigned** material — the
inference R-2 forbids. Rows 3/5 and R-2 cannot all be implemented.

**Fix.** Replace the discriminator with a whole-set post-assembly check, which
is exact, source-independent, and subsumes every arrival shape (seed+card,
card+card, seed+seed):

- **REFUSE iff any two final slots carry an identical 65-byte
  chain code ‖ pubkey.** (Equivalently: the same seed matching ≥2 slots *at
  the same origin*.)
- ≥2 seed matches at **distinct** origins is the legitimate multi-account
  wallet: proceed, with the notice `findUserSlot` already specifies.
- Correct §2.1's and §4.1's characterization of `reused`, and drop or reword
  row 3 so it no longer contradicts R-2.

This also closes the two-cards-same-xpub case (distinct card strings survive
the gather dedup at `bundleDuplicate`, which compares strings, not keys).

## C2 — the engrave tail is never respecified for the wallets §4.1 requires

**Where.** §4.1/§6 P3 respecify *assembly* only ("`SelfSlot int` becomes a
set; `cosignerFromCard` stops discarding origins; `OriginDivergent` is used…").
Nothing respecifies steps 4–9 of `buildMultisigPolicyFlow`
(`gui/multisig_build.go:95-168`), which today are hard-wired to ONE seed and
the LOCKED shared origin:

- step 4: `deriveAccountXpub(mnemonic, passphrase, …, multisigSharedOrigin())`;
- step 9: `deriveMultisigLeg(mnemonic, passphrase, …, multisigSharedOrigin(),
  engraveMd1, full)` — one mk1, and in full mode ONE ms1 from the one
  `mnemonic`;
- `multisigEngraveCards` (gui/multisig_engrave.go) emits exactly
  [ms1?, mk1, md1].

**Wrong-steel scenario (divergent origins).** Operator builds a divergent
policy holding one slot at `m/48'/0'/1'/2'`. Assembly (P3) is correct. Step 9,
untouched by the spec, derives the leg at `multisigSharedOrigin()` =
`m/48'/0'/0'/2'` — an xpub that is **not in the descriptor** — and engraves an
mk1 carrying that key and path, stub-bound to the policy
(`Stubs: [][4]byte{stub}` in `deriveMultisigLeg`). A key card asserting
membership in a wallet that does not contain its key, on steel.

**Unspendable-backup scenario (multi-seed).** 3-of-4; operator holds three
slots via masters A (two account indices) and B; fourth slot is a cosigner.
Full mode ("Full (seed + keys)") engraves **one** ms1 — from the single
`mnemonic` variable, i.e. master A only. B is in no engraving. Lose B: two
accessible legs < k=3 → **funds unspendable**, from a backup the device
labelled "Full (seed + keys)". This is precisely criterion (b) of the gate
question.

**No specified gate can see either.** P0's gate and §4.5 byte-compare the
produced **md1** only; the mk1/ms1 legs are never compared. The on-device
verify (step 10) would catch the wrong mk1 — but it is optional, NFC-readback
only (`multisigVerifyFlow` → `bundleGatherFlow`), and P5's text ("both a `wsh`
and an `sh(wsh)` multisig") does not require a divergent or multi-slot build,
so a shared-origin P5 passes green around both defects.

**Fix.** Add the missing normative requirements (naturally P3's):

1. The leg derivation MUST use each held slot's **declared origin**, never the
   locked shared origin, once origins can diverge.
2. RULE the cardinality: one mk1 **per held slot** (each at its slot's
   origin), or an explicit ruling that one leg suffices and why.
3. In full mode, **every distinct master supplied** MUST have its ms1 engraved
   — or multi-master full mode is refused with a named reason. Silence is not
   an option; the "Full (seed + keys)" label is a claim about the steel.
4. Extend the §4.5 byte comparison to the mk1(s) (and ms1 presence), and
   require P5 to include at least one divergent, multi-slot, multi-master
   build.

## I1 — the gate's trigger has no defined mechanism

§4.3 fires "on **assignment** — wherever the operator has placed both a seed
and a key into the same slot." No stage builds a step in which a slot can hold
both. The current flow shape is: pickers → cosigner cards fill non-self slots →
seed fills self slot. Both-in-one-slot cannot arise structurally, so a literal
implementation ships a gate that never fires — the operator's requirement
("if both seed and key are present, sh2 must verify") silently unmet, while
its tests can still pass against synthetic assignments.

The machinery the trigger needs is exactly what §10 leaves open: Q3 (which
seed/account index binds to which held slot) and Q5 (what the gather screen
becomes). Those are not open questions *for* R0; they are the definition of
the flow's central data structure (slot → source binding), and §4.1, §4.3 and
P0 all depend on it.

**Fix.** Specify the slot-assignment model normatively: for each slot, its
source (payload key | derived-from-seed(seed-id, account-index)), where the
operator sees and confirms it, and the rule for when a payload key is treated
as "the operator's slot" (which is what makes both-present possible). Then
restate the gate's trigger against that model. Q3 must be ruled, not asked.

## I2 — P0's gate and P1's premise are mutually exclusive

P0's gate: an automated test and an emulator walk drive the flow **to a
completed engrave**. P1: "Reproduce D-1 (now possible), fix it… the
reproduction test fails on the unfixed code."

If D-1 (a dead-end between configuration and engrave, field-observed) lives on
the P0 path, P0's gate is red until a fix that belongs to P1 — a stage closing
green is impossible with its successor's defect in its own gate path. If D-1
does *not* manifest, P1's gate ("reproduction test fails on the unfixed
code — demonstrated, not assumed") is unsatisfiable. There is no input on
which both stages close as written.

**Fix.** Move P0's close condition to the D-1 boundary ("the flow is drivable;
either it completes an engrave or it reproduces D-1, captured as a failing
test"), and give the completed-engrave gate to P1. One sentence each; the
work content of both stages is otherwise sound.

## I3 — NFC contradictions in normative text

§3 (operator decision): "NFC is OUT of phase 1 entirely." Against that:

- §1 phase 1: cosigner keys "arriving by **NFC** or payload";
- §5.4: "phase 1 delivers BIP-39 mnemonics from payload **and NFC**";
- R-1 (a RULING): "The self seed MAY come from a payload **or NFC**".

Three normative sentences license the excluded channel; R-1 is the most
authoritative text in the document. Separately, §5.1 mandates "exactly two
sources: PAYLOAD and TYPED" and forbids inert/stubbed entries — but the
*shipped* seed seam (`syswSeedPicker`, gui/derive_xpub.go:161) already offers
a working SCAN row whenever `Features().Has(FeatureNFC)`, i.e. on real
hardware. The spec neither acknowledges this row nor says what phase 1 does
with it (mask it for the constructor — re-opening the per-flow divergence the
seam exists to avoid — or leave it, contradicting "exactly two").

**Fix.** Strike NFC from §1, §5.4 and R-1 (phase 1), and add one ruled
sentence on the existing SCAN row (recommended: it is outside this spec's
scope, remains as shipped on hardware, and the phase-1 *tests/walks* exercise
payload+typed only — which keeps "no inert controls" true, since SCAN works
where offered).

## I4 — the spec's current-state account of TYPED-ONLY is false

Claim-vs-line, the class the brief flagged as high-value. §4.2: "the flow
currently **types** the self seed once"; §5.2: "TYPED-ONLY is a named
invariant in four places… the other three keep their invariant until their own
ruling"; P4: "the payload as a SEED source".

Measured at `a10d007`: `gui/multisig_build.go:68` calls `seedEntryFlow`, whose
own doc reads "**offers every source a seed may come from: typed, scanned, or
the systemwide payload**" (gui/derive_xpub.go:80-117), routing through
`syswSeedPicker`, which offers FROM PAYLOAD whenever the session holds
`ClassMnemonic` — and Engrave Multisig's SYSW§3.3.2 row admits it. The same
holds for the other three "TYPED-ONLY" sites: `bip85.go:271`,
`singlesig.go:33`, `multisig.go:103` all call `seedEntryFlow`. The four cited
comments (`I-3`, `D12`, `I-7`, `I-SCRUB`) are **stale text over a retired
mechanism** — the SYSW journeys work already wired the payload into every seed
entry ("Stage 10 made this the first screen of every seed entry in four
programs", gui/derive_xpub.go:149). This repo's own rule applies: grep the
mechanism, not the claim.

Consequences: R-1 "retires" what is already retired (fine, but it must say
so); §5.2's per-site framework governs comments, not behavior, and its promise
that the other three sites "keep their invariant" is already false; P4's
"seeds from the payload" largely exists, so P4's real content is the
consistency gate + multi-seed handling; and the interim fact that payload
seeds ALREADY flow into the build path **without** the §4.3 gate should be
stated as the current exposure the spec closes.

**Fix.** Rewrite §4.2/§5.2/R-1/P4 against the measured state; retitle R-1 as
ratifying + scoping the existing behavior; have a stage delete or correct the
four stale comments (they are exactly the "comments outlive their conditions"
defect class this project has been burnt by three times in one day).

## I5 — P0's ingest-everything model dead-ends legitimate payloads

P0 item 2: feed **every** `ClassMDMK` record into the gather.
`buildCosignerCards` (gui/multisig_build.go:254-272) then refuses unless the
yield is exactly n−1 mk1 cards and zero md1/ms1. But `ClassMDMK` covers md1
too, and the same program's *supply* path is the reason md1s are admitted
(SYSW§3.3.2). A payload provisioned for Engrave Multisig at large — an md1
plus cards, or the 11-card constellation set of which this wallet needs a
subset — hard-fails with "Gather exactly N−1 cosigner key cards (and no md1)"
and has no on-device remedy. No selection mechanism is specified, and the
selection screen is deferred to open question Q5 inside a stage that must
close.

Additionally: cosigner **slot order is @-index-bearing** (the encoder's
ordering contract, md/encode_multisig.go:13-21 — same keys, different order ⇒
different WalletPolicyId). With scanning gone, order = payload record order,
fixed on the host, unmentioned by the spec. For sortedmulti the *addresses*
are order-independent, so this is identity/coordination rather than funds —
but it must be stated, because the review screen offers no reorder.

**Fix.** P0 must specify: filter to mk1 cards (ignore, don't fail on, md1
records — or rule otherwise with a reason); the behavior for more matching
cards than open slots (a selection step, or a named refusal); and one sentence
pinning slot order (payload record order, shown as @N on the review screen).
Q5 becomes ruled scope, not an open question.

## I6 — §4.2's lifetime MUSTs are unsatisfiable for payload seeds

§4.2: a seed "MUST NOT outlive the flow, MUST NOT be written anywhere, and
MUST NOT be reachable from any other program." For a payload-sourced seed
(P4's headline, and today's reality per I4), the sysw session retains the
`ClassMnemonic` record bytes for the **process lifetime** by explicit SYSW
ruling ("LIFETIME IS THE PROCESS… No flow clears it… Nothing here claims the
records are scrubbed", gui/sysw_session.go:12-18), and every program whose
§3.3.2 row admits the class can `take` the same record. A compliant
implementer must either scrub the session record — violating SYSW§3.2.1 and
reintroducing the per-program cost "once per session" exists to avoid — or
fail the MUST. The mandated mutation-checked scrub test makes this concrete: a
sweep that finds the seed bytes in `syswSession.records` after the flow is
honest and red.

**Fix.** Scope §4.2's MUSTs to *the flow's working copies* (the
`bip39.Mnemonic` buffers and any derivation intermediates), and cite
SYSW§3.2.1 as governing the session record's lifetime, with §5.3's threat
model as the operator-facing statement of that residue.

## I7 — per-seed passphrase binding is unspecified

§4.1 introduces multiple seeds per flow; §4.3 derives "from the seed (plus
passphrase, if any)" — singular. The current flow asks the passphrase question
once, after the one seed. Extended naively, one flow-global passphrase applied
to N seeds builds a descriptor containing keys the operator can only ever
re-derive with a (seed, passphrase) pairing they never chose. For a
*new-seed* slot there is no card to cross-check, so no row of §4.3 can catch
it; the wallet engraves, verifies (the verify re-types the same pairing), and
funds land behind a leg whose passphrase binding exists only in the device's
transient pairing. The ms1 backup carries entropy only — the passphrase is
never on steel — so the wrong binding is invisible in every artifact.

**Fix.** One normative sentence: the passphrase prompt is **per seed**, asked
at that seed's entry, and the (seed, passphrase) pair is the derivation unit
everywhere §4.3 says "seed". (The restore doc already records origins; no
artifact change needed.)

## Minors

- **M1.** In `OriginDivergent` mode a cosigner card whose `Path` is `"m"`
  (depth-0 master xpub — mk1 permits it) yields zero components and trips
  `errMultisigEmptyDivergent` (md/encode_multisig.go:104-106). Legitimate to
  refuse; the spec should name the screen and message so it is a designed
  refusal, not a fall-through "Couldn't assemble" (which §4.3 would rightly
  call a silent-ish failure).
- **M2.** Step 10's verify readback is NFC-only (`multisigVerifyFlow` →
  `bundleGatherFlow`), deliberately payload-refusing (§7.4 self-comparison
  rule — correct). In phase 1 it is therefore exercisable only at P5 on
  hardware. Record it as a named blind spot of the §4.5 walk, owned by the
  F-158 NFC plan.
- **M3.** §2.1's sentence about `reused []int` ("when one key occupies several
  slots") misstates the mechanism (see C1); correct it even though C1's fix
  supersedes its use.

## What checked out (settled facts for the fold and the next round)

- **R-3 is sound.** With card-origin precedence, a SeedHammer-minted card's
  `Path` is by construction the origin its xpub was derived at
  (`deriveMultisigLeg` sets `Path: origin.String()` from the same `origin` it
  derived with), so descriptor origins match reality and `findUserSlot`
  restores correctly. Under the alternative (flow origin wins), a divergent
  cosigner's restore derives at the wrong path, gets zero matches, and is
  refused — fail-closed but wallet-unrestorable on-device. A card that lies
  about its origin also fails closed (derive-at-claimed-origin mismatches).
  Addresses derive from the xpubs, not the origin metadata, so no wrong-key
  descriptor is reachable through R-3. Q2's suggestion of a restore test
  inside phase 1 is nonetheless endorsed — it is the natural gate for C2's
  fixes.
- Claim-vs-line spot checks TRUE: §2.2 D-2 (`SelfSlot int` at
  multisig_build.go:342; `p.N-1` at :61; locked origin at :421-424; origin
  discarded at cosignerFromCard); D-3's three `scriptName` callers exactly as
  cited; `md1_expand.go` honours `InnerWsh` → `P2SH_P2WSH`;
  `deriveMultisigLeg` passes md1 verbatim, no address derivation;
  `ExpandedKey.Xpub` is 65 bytes cc‖pk; `take` returns first-match,
  non-consuming (sysw_session.go:114-124); `NFCReader()` nil /
  `SyswReader()` settable (gui_test.go:445/453); `TestBuildFlow_GatherBeforeSeed`
  exits at the gather under synctest; `writeNode` at encode.go:159 with the
  listed arms; `split` at chunk.go:121; `decodePayloadValidated`'s five
  structural validators; `errOperatorContext` root-tag allow-list at
  md.go:849; "nine sites" for decoder `tagTr` handling is fair under the
  natural counting (7 in md.go decode/summary paths + template_guard +
  template_strip).
- **`scriptName` has no consumers outside `gui`** (grep: 3 non-test callers,
  all gui) — §8's flagged unknown is resolved; §4.4's three-site update is the
  complete set.
- SYSW§3.3.2 verified: Engrave Multisig row admits exactly
  {Mnem, Cdx32, Passph, Descr, MDMK}; the "PERMISSION, not a promise" sentence
  and the §3.1 `bip39.Mnemonic` carrier-type inconsistency note are quoted
  accurately, so §2.1's "carrier, not a permission — no admission change, no
  Rust-primary cycle" conclusion stands, including for P4's payload seeds.
- §4.4's fix table, §4.6's tiering, §5.3, §5.4 and §9 are sound as scoped;
  nothing in phase 1 forecloses phase 2.

## Gate disposition

RED — 2 Critical, 7 Important. The two Criticals are specification defects,
not code defects: both are fixable with bounded edits (C1 a discriminator
change + table row rewrite; C2 four added normative sentences and two gate
extensions). No finding requires revisiting the operator's four settled
decisions. Re-review after fold should scope to: C1's new check semantics, C2's
leg/ms1 rules and gate coverage, and I1's assignment model — the rest are
text-level and a mechanical pass suffices.
