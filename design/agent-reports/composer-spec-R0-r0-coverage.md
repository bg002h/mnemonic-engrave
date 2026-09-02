# R0 round 0 — `SPEC_wallet_policy_composer.md`, COVERAGE AND TRACEABILITY lens

**Agent-written, verbatim. Nothing folded.** Dispatched 2026-09-01 against
`design/SPEC_wallet_policy_composer.md` at `b452a79`, fork `169073c`,
descriptor-mnemonic `790fc224`, mnemonic-toolkit `d8f06483`, mnemonic-secret and
rust-miniscript-fork at their working-tree heads.

**Scope.** One question, three directions: rulings → spec, normative statements →
acceptance, work items ↔ sections. Plus the cross-document, §13, §14 and
terminology checks named in the brief. **Not re-litigated:** the 29 rulings
(operator's, final) and the §5 lowering (expert-reviewed twice). **Already
gated, not re-run:** structure exit 0, glyph 30/0, citations 30/30 resolve
(existence only) — but citation *substance* WAS re-checked, and one is wrong
(C-1).

**Counts: 2 Critical / 8 Important / 11 Minor / 6 Nit.**

---

## CRITICAL

### C-1 — §4f's origin table contradicts shipped device behaviour for `sh(wsh)`, and calls it "unchanged"

§4f, NORMATIVE, row 1:

> | `wsh`, `sh(wsh)`, `sh` | `m/48'/coin'/account'/2'` (unchanged, `gui/multisig_build.go:1359`) | by ordinal among the slots that master fills (C5/C12) |

The shipped device does not do this. `gui/multisig_build_slots.go:111-130`:

```go
func derivedSlotOrigin(script md.MultisigScript, account uint32) bip32.Path {
	const h = hdkeychain.HardenedKeyStart
	return bip32.Path{48 | h, 0 | h, account | h, multisigScriptTypeComponent(script) | h}
}

func multisigScriptTypeComponent(script md.MultisigScript) uint32 {
	if script == md.MultisigShWsh {
		return 1
	}
	return 2
}
```

So `sh(wsh)` → **`1'`**, not `2'`. And the comment directly above it records that
this is a deliberate funds-safety fix, made at S5:

> "FROM S5 IT IS ALSO TEMPLATE-AWARE (plan §0.1a). BIP-48 assigns the SCRIPT TYPE
> component: 2' to native segwit and 1' to nested segwit, and NOTHING to legacy
> P2SH. Before S5 this returned 2' unconditionally, so an sh(wsh) build stamped
> the native-segwit path onto steel and a BIP-48-aware coordinator reading that
> plate derived at the wrong script-type path."

§4f as written re-introduces exactly that defect, in a NORMATIVE table, on an
irreversible medium, and asserts parity with the code ("unchanged") while
contradicting it.

**Second half of the same finding: the citation is the wrong function.**
`gui/multisig_build.go:1359` is `multisigSharedOrigin()`, whose body is
`bip32.Path{48|h, 0|h, 0|h, 2|h}` — the LOCKED shared origin for `OriginShared`
mode, with the account hard-coded to 0 and the script type hard-coded to 2. It
cannot support the row's "by ordinal among the slots that master fills" claim
under any reading. The cite gate resolves it because the line exists; only
reading the function shows it is the wrong one. (§7d's separate citation to
`gui/multisig_build.go:594-601` for the account-by-ordinal rule IS correct — see
the clean list.)

**Third: no acceptance item touches seed-derived origins at all.** §12.2's
journey packs "`key:` records (and a seed)" and compares "the consent's ids and
addresses", which is a downstream equality; a wrong script-type component
changes the xpub, which changes the id and the address, so the journey *could*
catch it — but only if the vector it compares against was itself derived at the
right path, and §12.1's families have no origin axis.

**Direction of fix:** restate the row per wrapper — wsh → `2'`, sh(wsh) → `1'`,
sh → `2'` (this device's own convention, which `multisigScriptTypeComponent`
announces loudly per §0.1a), tr → `3'` (C28) — cite
`gui/multisig_build_slots.go:111-130`, and add a per-wrapper origin vector to
§12.1. Note also that no §9 item extends `multisigScriptTypeComponent` (its own
comment calls it "the ONE site that decides it") or `md.MultisigScript` for
taproot; that half is I-3(i).

### C-2 — not one acceptance item can fail when a refusal fails to refuse

§11 is a normative section with no content and no gate:

> Collected from §4e, §6b, §7d, §7g. Every refusal names what to do instead;
> none prints an encoding.

Every one of §12's six items is a positive path: vectors of **composable** shape
families, an emulator journey that succeeds, a no-payload walk that succeeds, an
optional parity vector, a glyph/raster copy gate, a cite gate. Nothing in §12
exercises:

- §4e's six structural refusals (no keyed path; keys-and-hash-less path; keyless
  under `tr`; >8 paths or n>9; `sh`/`sh(wsh)` with more than one path or any
  lock/hash; the C5 impossibility);
- §6b's two lock refusals (`blocks > 65535` / `days > 388`; a date or height
  below the `now:` value);
- §7d's all-or-nothing seating refusal;
- §7g's twelve classified divergences — the table is a design artifact with no
  test behind any row;
- and, most sharply, **§4c's stated guarantee**:

  > "The device enforces these tables itself (§6b); it does not rely on md's
  > downstream guard, which today misses the zero-units case."

  That guarantee exists because `older(0x400000)` — a time-based lock of ZERO
  units, i.e. no lock at all — **encodes today** (C20, measured: md1 string
  `md1yqpqqxpye5kuqpqqqqqvkwqu7r50qu85`, filed
  `md-older-zero-time-units-not-refused`). If the device-side table is simply not
  written, all six §12 items still pass and a plate whose "lock" enforces nothing
  goes to steel. §10 item 4 patches md, but it is explicitly marked
  "(independent; filed)" and the §4c sentence says the device must not depend on
  it.

The §8a/§8b/§8f/§8g warning screens have the same shape one tier down: §12.5
gates their **glyphs**, and nothing asserts any of them **fires** on its
condition.

The project's severity rule lists "a refusal that does not refuse" and "a gate
that cannot fail" as still-blocking. Here it is the whole refusal surface, and
one of its members is a funds-safety lock. **Fix:** a refusal/negative vector
family in §12.1 (path list → the named refusal), plus per-refusal assertions
named in §12.2/§12.3, plus one assertion that a lock operand outside §4c is
refused *by the device* on an md build that still accepts it.

---

## IMPORTANT

### I-1 — §6a understates the payload-spec changes and asserts a row that does not exist

`grep -i "wallet policy" design/SPEC_systemwide_payloads.md` → **no matches.**
§3.3.2's table has eight program rows (Backup Wallet, BIP-39 Password, Engrave
Text, Account Xpub, Engrave Bundle, Engrave Single-Sig, Engrave Multisig, BIP-85
Child Seed, plus a dashed Sealed Payload) and seven class columns (Mnem, Cdx32,
Passph, FreeText, Descr, MDMK, Addr). So §6a's

> "Wallet Policy's row becomes: Mnem •, Cdx32 •, Passph •, Descr •, MDMK •, Key
> •, Hash •, Now •."

is false as an instruction. Five distinct gaps:

1. **The row must be CREATED, not amended.** The fork is already ahead of the
   spec doc: `gui/sysw_admit.go:26,28` carries `progWalletPolicy` and
   `progTransaction`, neither of which §3.3.2 has a row for.
2. **The stated row omits two of the ten cells** — FreeText and Addr — so an
   implementer transcribing it has no value for them.
3. **§3.3.1 (record classes, with the `secret?` column) is not mentioned.** It
   needs three new rows; `sysw/session.go`'s secret set is derived from that
   column and all three new classes are non-secret.
4. **§5.3's reserved-prefix list is not mentioned.** §6a correctly says the three
   records *follow* §5.3's rules (reserved prefix, lowercase hex, matched before
   the sniffers, prefixed-but-not-hex → `ClassUnknown`) — verified against
   §5.3.1, which states exactly that — but §5.3 itself enumerates `text:` and
   `pass:` and must gain three more.
5. **§3.3.3's flag rules are not accounted for.** Admitting Mnemonic, Cdx32 and
   Passphrase to Wallet Policy (C12) makes those classes secret at this program
   for the first time, so `syswFlags` (`gui/sysw_admit.go:122-141`) will fire
   **F1** ("this secret is unencrypted in flash; offers erase (§5.5)") and, on a
   weakly-sealed payload, **F2**, inside the composer's seed step. §6b, §7d and
   §7g never mention the flag screens, and F1's erase offer arriving mid-compose
   is an unclassified divergence.

**And no §9 or §10 work item owns editing `SPEC_systemwide_payloads.md`.** It is
a normative artifact with its own R0 history (rounds 0-6 persisted); a change to
it that lands only inside another spec's prose is a change nobody reviews.

### I-2 — the comment that most directly contradicts the change is not named, and the one that is, is misquoted

§6a:

> "The enum comment at `gui/gui.go:191` ('came from OUTSIDE this device', 'needs
> neither a seed requirement') is rewritten (C12)."

- **The second quotation is a splice.** The source (`gui/gui.go:201-203`) reads
  "…bolting that onto either would drag **a seed requirement or a plate census**
  into a flow that needs neither." The spec quotes two non-adjacent fragments as
  one. The plate-census half is falsified by §7f ("Plate census before cutting,
  as Multisig Build does") and the spec does not name it.
- **A third stale clause is not named.** `gui/gui.go:192-193`: "It is **not a
  rename of Multisig** and not an extension of Bundle." C7's migration
  ("eventually migrating standard multisig wallet descriptor policy/template
  authoring in Wallet Policy") plus C6's Build door plus §4a's `sh`/`sh(wsh)`
  admission put that clause under exactly the same pressure as the other two.
  This is also the spec's only contact with **staged-plan D5** ("A new 10th
  navigable program, 'Wallet Policy' — not a rename of Multisig, not an in-place
  extension"), and the spec never reconciles D5 with C7. It is not a
  contradiction — C7 is later and is the operator's — but the reconciliation is
  unwritten.
- **`gui/sysw_admit.go:47-51` is not named at all**, and it is the strongest
  case:

  > "NO seed class. The Wallet Policy program never derives from a secret: its
  > proof is addresses derived from the policy's OWN public keys plus a named
  > wallet id, so admitting a mnemonic would grant a capability the flow has no
  > use for. Least privilege, and it is enforced here rather than by the flow
  > declining to ask."

  C12 is a deliberate reversal of precisely this argument, and the brainstorm
  says so ("Deliberate reversal"). Left as-is, the comment survives the change as
  a printed falsehood sitting on the very table it contradicts.

### I-3 — nine normative sections that require device code have no §9 work item

§9 has nine items; §10 has five. Every one of them traces cleanly to a section
(see Table 3). The failure is the other direction:

| section requiring device code | §9 item |
| --- | --- |
| (a) §4d — the five archetypes "offered as one-tap presets that POPULATE a path list the operator then edits" | **none** (§10 item 3 is host-side Concrete policies + expected templates only) |
| (b) §4e — the six structural refusals, and the picker bounds (1..8 paths, n 1..9) that §4e says "the picker does not offer" | **none** |
| (c) §6b — the kind picker, unit picker, days→512-s-unit conversion, `YYYY-MM-DD`→Unix-at-00:00-UTC conversion, the four echo forms, and the `now:`-relative refusals | item 3 is the **digit-pad widget** only |
| (d) §6c — hashlock entry: pick from `hash:` records, type 64 hex, show the digest at consent | **none** |
| (e) §7b — the path-list screen and "Back preserves everything" | **none** |
| (f) §7d — the **mapping-review screen** (slot → fingerprint + origin) | **none** — and it is the single mitigation §7d offers for the residual mistap hazard the F-216 reconciliation creates |
| (g) §8a, §8b — the two EXPERIMENTAL screens | **none** (and see I-4) |
| (h) §8f, §8g — the NUMS note and the same-seed warning | **none** |
| (i) taproot origin support — a `3'` arm in `multisigScriptTypeComponent` ("the ONE site that decides it") and a taproot member on `md.MultisigScript` | **none** |

(f) and (i) are the two that cost funds if omitted. The brief also asked about
the `now:` writer and the admission row: both ARE present — §10 item 2 ("`now:`
written automatically at pack time") and §9 item 4 ("admission row (§6a)") — see
the clean list.

### I-4 — C16's "unskippable" is dropped

C16, ruling (1): keyless hashlock-only paths are "ADMITTED behind an
**unskippable** EXPERIMENTAL screen naming bearer access". The spec implements
the admission (§4b, §4e), the wsh-only restriction (I2/C16) and the copy (§8a) —
but the word *unskippable*, and any equivalent (an acknowledgement, a
confirm-to-proceed, a second tap), appears nowhere in §4, §7, §8 or §12. Same for
§8b's unsorted-keys screen, which C16 puts "under the same EXPERIMENTAL
warning". PARTIAL implementation of a ruling, and no acceptance item would fail
if both screens were a dismissible toast.

### I-5 — §12.1's vector families have no path-count axis, so the lowering's central rules are untested by the stated families

§12.1's axes are `single/multi keys × none/lock/hash × wsh/tr × sorted/unsorted ×
keyless-wsh` — every one of them a **per-path** attribute. The §5 rules that only
exist at two or more paths are:

- `or_d(P, R)` iff `P` is a bare unlocked, unhashed `multi(k,…)` with n ≥ 2;
  otherwise `or_i(P, R)`; a bare single key is `or_i(pkh(K), R)`;
- "listed order, recursive, **last path stands alone**";
- the tr right spine `{P1,{P2,{P3,P4}}}` and "path k at depth min(k, n−1)";
- `@i` numbering **by first appearance in the emitted text** (indistinguishable
  from listed order at one path);
- internal-key extraction and the "then not a leaf" consequence — §10 item 1 asks
  for exactly one such vector ("a vector where the internal key is not path 1"),
  which is not enough to pin depth or spine shape.

§12.1 is the section an implementer enumerates the corpus from, so the axis has
to be there: name path counts 1, 2, 3, 4 and 8 explicitly, and require at least
one 4-path tr vector (the only shape that exercises `min(k, n−1)` twice).

### I-6 — the three record classes have no host/device lockstep acceptance, and their negative rules have none either

§6a says "host `me sysw pack` + device `seal.Classify`, **lockstep**" and §9 item
8 repeats "lockstep with the host". No §12 item compares the two. This is the
exact shape of F-212, where Go and Rust computed different `WalletPolicyId`s
while 887/887 fork tests passed either way — a cross-language vector is the only
thing that sees it.

Nor is there acceptance for any of §6a's own normative rules: a prefixed record
whose body is not valid hex is `ClassUnknown` and refused; the three prefixes are
matched BEFORE the sniffers; a bare xpub is refused **naming the fix**; origin is
REQUIRED (F-166 pathless is open). §12.2's journey packs `key:` records, which
exercises the positive path only.

### I-7 — §7d's normative seating rules have no acceptance, and three of them diverge silently from the shipped seating code

Normative statements in §7d with no §12 item:

- **"Each key is used at most once."** The shipped `gui/key_card_seating.go:28-30`
  states the opposite rule for its own route: "ONE CARD MAY FILL SEVERAL SLOTS. A
  policy can legitimately seat one master at several accounts, and a card whose
  origin matches two slots fills both." Both are correct for their own route
  (C8's "one of the **remaining** keys" is explicit), but the spec never says the
  two coexist, so a reader has two contradictory rules for the same artifact
  class.
- **"a seated card is RE-MINTED for engraving with the new policy's stub appended
  to its existing stubs (`mk.Encode`)."** No acceptance. Dropping the existing
  stubs silently unindexes the card from the wallet it already belonged to —
  data loss on a recovery artifact, invisible until a restore.
- **"their stubs are IGNORED at seating."** This contradicts `seatKeyCards`'s
  LAYER 1 (`gui/key_card_seating.go:16`: "the card's policy_id_stub **must**
  include this template's stub"). §7d reconciles the *assignment* rule
  ("the operator is never asked to assign a card to a slot") — correctly,
  verbatim, and with the right reason — but not the *stub* rule. An implementer
  reusing `seatKeyCards` for the composer gets a function that refuses every
  payload card, because a card packed before composition necessarily carries some
  other wallet's stub (brainstorm §3.5a).
- **The C29 warning and the C5 informational line.** §12.5 gates their glyphs;
  nothing asserts either fires on its condition, and the C29 case is the one
  where a nominal 2-of-3 is satisfiable by one person.

### I-8 — §7f's engrave surface has no acceptance, and reverses a deferral the spec does not name

§12.2 stops at "engrave choice"; §12.3 at "a keyless-template engrave". Nothing
in §12 covers: the concrete-descriptor **text** plate path, the **QR** plate
path, Full vs Watch-only, the three secret plate forms, the plate census, the
read-back-integrity line ("md1/mk1 carry BCH; a text or QR descriptor carries
only its BIP-380 checksum"), or §7g's last row — "concrete descriptor longer than
the plate holds → REFUSAL by census with the measured ceiling (§13)".

That last one is a refusal gated on a number §13.1 says is not yet measured, so
it is the one §12 gap that cannot be closed by writing a test today — which is
the reason to name it now rather than at plan time.

**And §7f re-opens staged-plan 6d without saying so.** 6d ("engraving a concrete
descriptor") was DEFERRED 2026-08-20 for "unmeasured sizing plus an irreversible
medium while content rules were still moving". §13.1 answers the sizing half;
the content-rules half is answered by §5 being ruled — but neither is stated as
a reversal, so a reader cannot see that a deferral was lifted or why. Relatedly,
**§14 omits D8's named wallet-backup formats** (BSMS/BIP-129, Nunchuk, Sparrow —
staged-plan 6c, "DEFERRED on-device with 6b"), so §7f's "concrete policy" reads
as though it discharged D8.

---

## MINOR

**M-1 — `coin'` is a variable the device does not have.** §4f writes
`m/48'/coin'/account'/…` for every wrapper. `derivedSlotOrigin` hard-codes
`0 | h`, and complex-policy address derivation is mainnet-only by construction:
`gui/policy_address.go:61`, `network := &chaincfg.MainNetParams // D1:
mainnet-only`. Either write `0'` and add mainnet-only to §14, or add a work item.

**M-2 — §7c does not say what the *other* label becomes.** It names the ambiguity
and says it "is fixed in the same change", but the fix has two sites:
`gui/template_engrave.go:70` and `:79` both emit
`fmt.Sprintf("Template-ID: %x", templateID)` for the **4-byte stub**. Only the
composer's own screen has stated copy.

**M-3 — "path" carries two meanings and the spec never disambiguates.** Spend
path (§4 opening sentence, §4b, §5, §7b, §7d "Slot @2, Path 1 key 2 of 3") and
derivation path (§4f "Key origins", §6a "`[fingerprint/path]xpub`", §7d
"fingerprint + origin path" — in the same paragraph as the other sense, §7g
"card origin script type"). §4's first sentence gets it right ("spend paths");
adopt that everywhere the spend sense is meant.

**M-4 — §12.3 does not assert the stub screen.** C9 makes it unconditional and
C26 puts it on the no-payload route explicitly ("shape → (no seating possible) →
the C9 stub-teaching screen → keyless-template consent → engrave"). §12.3 says
only "ends in a keyless-template engrave", so the unconditional half of C9 has no
gate on the one walk that proves it.

**M-5 — `hash:`'s body is a different kind of hex from the other four.** `text:`,
`pass:`, `key:` and `now:` are hex **of UTF-8 text**; `hash:` is hex **of the
digest bytes**. §6a's lead ("All three follow … a lowercase-hex body") papers over
it, and §5.3.1's normative decode rule is "hex-decoded back to UTF-8". State the
two body kinds, or a decoder written from the lead produces mojibake.

**M-6 — F-150 item 1 is not referenced.** The flow C7 deprecates by comment
carries a filed SEVERE defect: "It dead-ends. `buildMultisigPolicyFlow` fails to
deliver a descriptor after configuration — a BLANK SCREEN after pressing next."
D1 forbids folding it in, which is right; but §8e and §14 should record that the
deprecated path stays live *and* broken, so nobody infers the deprecation covers
it.

**M-7 — §10 item 5 changes a shipped negative test.**
`mnemonic-secret/crates/ms-cli/tests/cli_derive_bip48.rs:174-178` currently
asserts `bip48-p2tr` **must be refused**. Name it in the work item.

**M-8 — §12/§15's gate list omits structural gates that exist.** `scripts/`
carries `plan-table-check.sh`, `plan-stepref-check.sh`, `plan-wiring-check.sh`,
`plan-fold-sweep.sh` and `plan-staleness-check.sh` besides the four §15 names.
§15's "A plan's GREEN expires" sentence names neither `plan-staleness-check.sh`
nor a baseline revision for this spec to be compared against.

**M-9 — §7f's "ms1 strings" secret plate form has no citation.** It is true
(`gui/codex32_polish.go:218` `engraveCodex32`, reached from the standalone
codex32 flows), but §3's inventory row for seed material stops at words, and §7f
asserts three secret plate forms with none.

**M-10 — C21's side finding is not carried.** "Liana's import refuses any `after`
or hashlock path regardless of head, so its acceptance wallet must be
`older`-only." That constrains §13.4's import tests and F-449's eventual
acceptance wallet, and it appears nowhere in the spec.

**M-11 — §9 item 2 is a prerequisite for §7e and §12.1, and nothing says so.**
The `pk_h` emitter arm is what makes a composed wsh single-key path derive an
address at all (§3: "Emit Script for fragments | all but `andor`, `pk_h`"), and
address derivation for complex shapes runs through the emitter
(`gui/policy_address.go:44-84`). §7e promises addresses "when seated" and §12.1
requires them per family; neither notes that item 2 gates both. Until it lands,
`walletPolicyAddressLines` prints "This device can't derive addresses for this
policy." (`gui/wallet_policy.go:265`) for exactly the C17 shapes the composer
emits.

---

## NIT

**N-1** — §2's row "C19-C23 | review findings adopted" labels **C22 as adopted**;
C22 was WITHDRAWN by C23 ("I take that back, I do want or_d for multi head"). §5
implements C23 correctly; only the §2 label is wrong.

**N-2** — §2's mapping is incomplete for **C6** (also §6a: the enum comment) and
**C12** (also §6a: the admission-row widening, which is C12's own stated
consequence). Both rows name only §7a / §7d.

**N-3** — §4a's "n ≤ 15 for `sh` (Core `MAX_P2SH_SIGOPS`)" can never bind: §4b
caps n at 9 for every wrapper. Correct, and inert.

**N-4** — §8g's example puts `@0` and `@2` in one path. Under `tr`, §5 makes `@0`
the extracted internal key, which by construction is *not* in a leaf, so the copy
is reachable in wsh only. (It is C29's verbatim wording, so this is a
presentation note, not a ruling question.)

**N-5** — C11's last clause — "Height-based and time-based locks may not mix
within one spend path (miniscript timelock-mixing rule)" — is silently dropped.
§4b's "at most one of `older` or `after`" per path discharges it and §5b's
`sanity_check` would catch a violation, so nothing is wrong; the spec should say
that is why, since the ruling raises it.

**N-6** — §7a cites F-437 as though live. F-437 is **RESOLVED 2026-08-29**
(fork `f2007b7`), and the "SCAN CARDS" naming already shipped at the payload
branch of this very door (`gui/wallet_policy.go:62`).

---

## TABLE 1 — RULINGS → SPEC (29 rows)

`§2 claims` is the spec's own §2 mapping; `verdict` is mine after reading both.

| # | §2 claims | actually implemented at | verdict |
| --- | --- | --- | --- |
| C1 | §4, §5 | §4, §5, §5b, §10.1, §14 r1 | COMPLETE |
| C2 | §4d | §4d, §10.3 | COMPLETE (§4d correctly carries the §3.7 correction: presets, not byte goldens) |
| C3 | §5, §10 | §5, §10.1, §14 r2 | COMPLETE |
| C4 | §7 | §7b→§7d ordering | COMPLETE |
| C5 | §4b, §7d | §4b `KEYS` row, §7d, §7g r9, §14 r3 | COMPLETE |
| C6 | §7a | §7a **+ §6a** (enum comment) | PARTIAL — §2 omits §6a; and the rewrite misses two of three stale clauses (**I-2**) |
| C7 | §8e, §9 | §8e, §9.9, §4a, §12.4, §14 r9 | PARTIAL — D5 reconciliation unwritten (**I-2**); F-150 item 1 unreferenced (**M-6**) |
| C8 | §7d | §7d, §7a, §12.2 | COMPLETE |
| C9 | §7c | §7c, §9.6 | PARTIAL — "unconditionally" has no gate on the no-payload walk (**M-4**); second label site unspecified (**M-2**) |
| C10 | §7f | §7f, §9.7, §13.1 | PARTIAL — no acceptance; 6d reversal and 6c omission unstated (**I-8**) |
| C11 | §6b | §6b, §4c, §9.3 | PARTIAL — flow has no work item (**I-3c**); timelock-mixing clause dropped (**N-5**) |
| C12 | §7d | §7d, §6a, §9.5 | PARTIAL — §2 omits §6a; `sysw_admit.go:47-51` unnamed (**I-2**); sysw doc changes understated (**I-1**) |
| C13 | §7f | §7f | PARTIAL — no acceptance (**I-8**); ms1 plate form uncited (**M-9**) |
| C14 | §9 | §9.9, §14 r11 | COMPLETE (no normative section other than the work item; acceptable — secret-handling is non-gating by the 2026-08-27 ruling) |
| C15 | §10 | §10.1 | COMPLETE |
| C16 | §4 | §4a, §4b, §4e, §5 row 4, §8a, §8b | PARTIAL — **"unskippable" dropped** (**I-4**) |
| C17 | §5 | §5 key-set row, §9.2, §14 r6 | COMPLETE |
| C18 | §5c, §8f | §5, §5c, §8f, §14 r5 | COMPLETE |
| C19 | (in C19-C23) | §5 numbering row; §4b/§4e I2/I3/I4; §5a M3/N1/N2 | COMPLETE |
| C20 | (in C19-C23) | §4c (all four rows sourced), §5 internal-key row, §10.4 | COMPLETE — but §4c's device-enforcement guarantee has no acceptance (**C-2**) |
| C21 | (in C19-C23) | §5 paths-combine row, §5a b1 | COMPLETE — side finding not carried (**M-10**) |
| C22 | (in C19-C23) | withdrawn; §5 implements C23 | mis-LABELLED in §2 (**N-1**) |
| C23 | (in C19-C23) | §5 paths-combine row, §5a b2 | COMPLETE |
| C24 | §6a, §6b | §6a `now:` row, §6b, §10.2 | COMPLETE |
| C25 | §6b, §6c | §6b, §6c, §9.3, §14 r8 | PARTIAL — only the widget has a work item (**I-3c/d**) |
| C26 | §7b, §7e | §7b, §7e, §7d lead, §7g r1, §12.3 | PARTIAL — stub screen not asserted on that walk (**M-4**) |
| C27 | §4f, §7d, §13 | §4f, §7d, §13.5, §14 r4 | COMPLETE |
| C28 | §4f | §4f | **CONTRADICTED** for `sh(wsh)` — see **C-1** |
| C29 | §7d, §7g, §8g | §7d, §7g r10-11, §8g | PARTIAL — warning has no firing gate (**I-7**) |

**Contradiction quoted in full (C28 / §4f vs shipped):**
spec — "`wsh`, `sh(wsh)`, `sh` | `m/48'/coin'/account'/2'` (unchanged,
`gui/multisig_build.go:1359`)";
code — `gui/multisig_build_slots.go:125-130` — "`if script == md.MultisigShWsh {
return 1 }` / `return 2`", under the comment "BIP-48 assigns the SCRIPT TYPE
component: 2' to native segwit and **1' to nested segwit**".

**§2's own table, audited:** 27 of 29 rows point at a section that does implement
the ruling; the two defects are the C22 label (N-1) and the C6/C12 omissions of
§6a (N-2). It is a usable map, not a false one.

---

## TABLE 2 — NORMATIVE STATEMENTS → ACCEPTANCE

`✓` = a §12 item would fail if the rule were violated. `—` = nothing would fail.

| § | rule | §12 item | gap |
| --- | --- | :-: | --- |
| 4a | tr/wsh admit any list; sh/sh(wsh) single unlocked unhashed sortedmulti; n ≤ 15 for sh | — | no wrapper-admission vector; the sh bound is inert (N-3) |
| 4b | KEYS: n 1..9, 1 ≤ k ≤ n, FRESH slots, every slot in exactly one path | 1 (partly) | families cover k/n shapes; **slot-freshness and the 1..9 bound have no vector** |
| 4b | HASH: at most one `sha256(H)` | 1 | ✓ via the none/lock/hash axis |
| 4b | LOCK: at most one `older` or `after` | 1 | ✓ (also discharges C11's mixing rule, N-5) |
| 4b | keyless path: wsh only, EXPERIMENTAL | 1 | ✓ for the emitted text; **— for the EXPERIMENTAL screen and its unskippability (I-4)** |
| 4b | at least one path has KEYS | — | **C-2** |
| 4c | four lock-value ranges, device-enforced independently of md | — | **C-2 — the funds-safety one** |
| 4d | five presets populate an editable path list | — | no vector, no work item (**I-3a**) |
| 4e | six structural refusals | — | **C-2** |
| 4f | seed-derived origins per wrapper (2'/1'/2'/3') | — | **C-1** |
| 5 | paths combine: `or_d` iff bare multi head, else `or_i`; last stands alone | 1 (partly) | **no path-count axis (I-5)** |
| 5 | tr right spine, depth `min(k, n−1)` | 1 (partly) | **I-5** |
| 5 | inside a path: `and_v(v:KEYS, and_v(v:sha256(H), LOCK))` | 1 | ✓ |
| 5 | key set: sortedmulti / multi / pkh ; sortedmulti_a / multi_a / pk | 1 | ✓ via the sorted/unsorted and single/multi axes |
| 5 | unsorted is EXPERIMENTAL | 1 | ✓ for the text; **— for the screen (I-4)** |
| 5 | internal key = first-listed unlocked unhashed one-key path, else NUMS | 1 + §10.1 | ✓ (one vector named; **I-5** wants more) |
| 5 | NUMS spelled raw `H` | 1 | ✓ (byte-pinned in the template text) |
| 5 | `@i` by first appearance; slot labels are those indices | 1 (partly) | text ✓; **the operator-facing LABELS are asserted nowhere** |
| 5 | use-site `/<0;1>/*` on every slot | 1 | ✓ |
| 5b | parses; `sanity_check`; `lift()` ≡ `md compile`; `md encode`→`decode` byte-identical | 1 | ✓ — the strongest item in §12 |
| 6a | three classes, reserved prefixes, matched before sniffers | — | **I-6** |
| 6a | prefixed-but-not-hex → ClassUnknown, refused | — | **I-6** |
| 6a | `key:` origin REQUIRED; bare xpub refused naming the fix | — | **I-6** |
| 6a | `now:` is a LOWER BOUND; never reaches an encoded operand | — | no test that a `now:` value cannot influence the emitted lock |
| 6a | host/device lockstep | — | **I-6 (F-212 shape)** |
| 6a | Wallet Policy admission row + enum comment | — | **I-1, I-2** |
| 6b | kind→unit→digits→echo; four encodings; four echo forms | — | **I-3c** |
| 6b | refusals: >65535 blocks, >388 days, below `now:` | — | **C-2** |
| 6b | without `now:` the copy never says "now" | 5 (partly) | glyph gate sees the strings, not the condition |
| 6c | pick from `hash:` records; type 64 hex; consent shows the digest | — | **I-3d** |
| 7a | ChoiceScreen in EVERY state; choices name their route | 2, 3 (partly) | walks enter via Build; **no assertion the door appears in all three states** |
| 7b | path-list screen; Back preserves everything | — | **I-3e** |
| 7c | stub screen UNCONDITIONAL; id and stub labelled distinctly | 2 ✓ / 3 — | **M-4**, **M-2** |
| 7d | offered only when payload holds keys/seeds, or operator types one | 3 (partly) | the no-payload walk implies it |
| 7d | slot-directed pick list over REMAINING sources | 2 | ✓ (a mis-seated key changes the id the journey compares) |
| 7d | each key used at most once | — | **I-7** |
| 7d | mk1 stubs ignored; card RE-MINTED with stub appended | — | **I-7 — data loss on a key card** |
| 7d | seeds: per-slot hardened accounts by ordinal; per-seed passphrase | 2 (partly) | only if the vector pins the origin — see **C-1** |
| 7d | mapping-review screen precedes consent; Back keeps assignments | — | **I-3f** |
| 7d | C29 warning in-path; C5 informational cross-path | 5 (glyphs only) | **I-7** |
| 7d | seating all-or-nothing; refuse naming both counts | — | **C-2** |
| 7e | consent: summary, id named by kind, addresses 0..1, keyless line, stub lines | 2 ✓ / 3 (partly) | keyless consent line not asserted on walk 3 |
| 7f | form choice; Full/Watch-only; three secret forms; census; read-back line | — | **I-8** |
| 7g | twelve classified divergences | — | rows map to §4e/§6b/§7d refusals — all **C-2** |
| 11 | every refusal names what to do instead; none prints an encoding | — | **C-2** — §11 has no content and no gate |

**Counted:** 45 normative rules; **14 have an acceptance item that can fail, 31
do not.** The 31 concentrate in exactly two places — refusals/warnings (C-2) and
the device flow outside the lowering (I-3).

---

## TABLE 3 — WORK ITEMS ↔ SECTIONS

**§9 device — every item traces (9/9).**

| # | item | requiring section | ok |
| --- | --- | --- | :-: |
| 1 | md tree BUILDER API, byte-identical to the Rust vectors | §5, §12.1 | ✓ |
| 2 | `pk_h` emitter arm, both contexts, + mutation check | §5 key-set row (C17) | ✓ (and see **M-11**) |
| 3 | digit-pad widget | §6b | ✓ (widget only — **I-3c**) |
| 4 | door ChoiceScreen in every state; admission row | §7a; §6a | ✓ (enum-comment rewrite folded in implicitly — **I-2**) |
| 5 | seating pick list; ms1 legs; per-slot accounts; `mk.Encode` re-mint | §7d | ✓ (mapping-review screen missing — **I-3f**) |
| 6 | stub-teaching screen; id/stub label fix | §7c | ✓ (**M-2**) |
| 7 | engrave form choice | §7f, §13.1 | ✓ |
| 8 | three payload classes in `seal.Classify` | §6a | ✓ |
| 9 | deprecation comment; scrub-on-exit | §8e; C14 | ✓ (no work item for C7's FOLLOWUPS entry — nit) |

**§10 host — every item traces (5/5).**

| # | item | requiring section | ok |
| --- | --- | --- | :-: |
| 1 | md-codec `compose` + `md compose` + vectors + §5b cross-check | §5, §5b, §12.1 | ✓ |
| 2 | `me sysw pack`: three classes; **`now:` written automatically at pack time** | §6a | ✓ — the brief's `now:`-writer question: **it is present** |
| 3 | five presets as Concrete policies + expected templates | §4d | ✓ host half only — **I-3a** |
| 4 | `md-older-zero-time-units-not-refused` patch (independent) | §4c note, §3 | ✓ |
| 5 | `ms derive --template bip48-p2tr` | §4f, C28 | ✓ (**M-7**) |

**Sections requiring code with NO work item — I-3 (a)-(i), plus:**

| missing item | brief asked? |
| --- | --- |
| mapping-review screen (§7d) | **yes — confirmed missing** |
| `now:` record writer in `me sysw pack` | yes — **present**, §10 item 2 |
| Wallet Policy admission-row change | yes — **present**, §9 item 4 |
| enum comment rewrite | yes — **present in §6a prose**, not a numbered item; and incomplete (**I-2**) |
| Multisig Build deprecation comment | yes — **present**, §9 item 9 (its FOLLOWUPS half is not) |
| T6b optional parity vector | yes — **present**, §12 item 4, correctly marked optional per C7 |
| editing `SPEC_systemwide_payloads.md` (§3.3.1, §3.3.2, §5.3, §3.3.3) | **missing — I-1** |
| `multisigScriptTypeComponent` / `md.MultisigScript` taproot arm | **missing — I-3(i), C-1** |

---

## CLEAN — checked, and correct

**Citation substance** (re-opened beyond the existence gate; all correct except
C-1's `gui/multisig_build.go:1359`):

- `md/encode.go:159` `writeNode` ✓, `:374` `encodePayload` ✓, `:461`
  `encodeMD1String` ✓ — §9 item 1 names the right two seams.
- `sysw/wire.go:28` → `RegionLen = 65536` ✓ = §3's "64 KiB".
- `txqr/txqr.go:38` → `MaxSymbols = 16` ✓.
- `gui/transaction.go:1145` greedy first-fit text packing ✓; `:1369`
  `qrCeilingBytes` is a **search**, not a constant ✓ — §3's parenthetical is
  exactly right.
- `gui/passphrase_keyboard.go:21` → `ppPageSymbols = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"`
  ✓ — "a digits page mixed with punctuation" is precise.
- `gui/multisig_build.go:594-601` ✓ — the account-by-ordinal rule, and it is
  keyed on the MASTER not the seed id, which is what §7d says.
- `gui/multisig_build.go:738` `buildSeedForSlot` ✓ — per-seed passphrase, screens
  name the slot.
- `gui/derive_xpub.go:104` `seedEntryFlowTitled` ✓.
- `gui/wallet_policy.go:97` ✓ — the no-payload branch does fall straight into the
  `bundleCard` gather loop.
- `gui/key_card_seating.go` — the sentence §7d quotes is **verbatim**, and its
  reconciliation ("that rule seats a template that ALREADY declares its origins;
  a composed template has none, so the operator's choice IS the declaration") is
  the correct reading of the code comment's own reason.
- `crates/md-codec/src/tree.rs:51` `is_nums` ✓; `:92-120` the `1..=32` bounds ✓.
- `rust-miniscript-fork/src/miniscript/limits.rs:35` = 20, `:38` = 999 ✓;
  `src/primitives/absolute_locktime.rs:10` = `0x7FFF_FFFF` ✓.
- `gui/testdata/t6b_multisig_full.md1.txt` exists ✓.
- `mnemonic-toolkit` really does sweep `bip48-tr-multi-a` with component `3`
  (`crates/mnemonic-toolkit/src/cmd/xpub_search/candidate_paths.rs:85`) ✓ — §4f's
  *evidence* for `3'` holds even though the row is wrong for `sh(wsh)`.
- All six §12 gate artifacts exist: `scripts/plan-glyph-check.sh`,
  `scripts/plan-cite-check.sh` (which does honour `CITE_FORK_ROOT`, line 96),
  `scripts/plan-build-gate.sh`, `scripts/plan-build-gate-go.sh`,
  `gui/raster_test.go`.

**§14 out-of-scope reasons — all twelve verified against the cited ruling; every
one says what §14 claims:**
C1 "Not a miniscript tree editor, not an on-device policy compiler" ✓;
C3 "NOT a host-generated fragment menu shipped to the device" ✓;
C5 "The wire stays narrow (F-417 reaffirmed)" + F-417's standing ruling record ✓;
C27-3 "defer, we don't want to waste time on this niche area right now" ✓;
F-449 "its own constellation cycle … does NOT gate the wallet-policy composer" ✓;
F-448 "recon item, not a composer gate" ✓;
C8 "we must first implement payload based gathering as I don't yet have nfc
hardware" ✓;
C25 "on-device preimage derivation DEFERRED" ✓;
C7 "No removal, no redirect, no migration gate this cycle" ✓;
staged-plan **6b** is indeed "QR-series transport — DEFERRED, may be SKIPPED
ENTIRELY (operator ruling 2026-08-20)" ✓;
C14 ✓; and `andor` is genuinely absent from §5's grammar ✓.

**FOLLOWUPS references — all correct:** F-150 items **3** ("Script types are
limited to three, and taproot is absent") and **4** ("No miniscript operators")
are exactly what §1 claims ✓; F-166 is the pathless-origin decoder gap, open ✓;
F-216 is the keyless-template seating item ✓; F-417 ✓; F-437 ✓ (but N-6);
F-448 and F-449 are both filed, both dated 2026-09-01, and **both explicitly say
"does NOT gate the wallet-policy composer"** ✓. Cross-repo:
descriptor-mnemonic `md-older-zero-time-units-not-refused` ✓ and
`md-descriptor-depth0-xpub-ledger-registration` ✓ both exist and say what §10.4
and §13.2 claim; mnemonic-secret `ms-derive-taproot-justifications-stale` ✓ —
its owning phase already reads "template decision follows the composer's origin
ruling", so §10 item 5 lands on an item that was waiting for it.

**Earlier rulings not contradicted:**
**D1** — the spec's self-description as D1's "compose later" is right ✓.
**D2** — §7e keeps proof = addresses + named id ✓.
**D3** — "skipping the gather proceeds to consent WITHOUT address proof" is not
contradicted: §7e keeps "Keyless template - no addresses" and §7d's
all-or-nothing matches the shipped `errSeatSlotUnfilled` ✓.
**D4** — keyless template as a valid goal with its own id: §7c, §7e ✓.
**D6** — §7e/§12.1's "receive and change 0..1" matches `addrProofPerChain = 2`
(`gui/wallet_policy.go:240`) exactly ✓.
**D7, D8** — not contradicted (D8's named formats: see I-8's second half).
**D5** — the only tension, and it is C7's to resolve, not a defect: **I-2**.

**§13 "not verified" is honest as far as it goes.** All five entries are real
gaps, each is a fact the spec does NOT then rely on silently, and the plate-
ceiling entry (item 1) is correctly load-bearing for §7g's census refusal. The
brief's font question: fonts appear in the spec **only** inside §13.1, which is
the right place. The gaps I found outside §13 are M-1 (mainnet-only), M-9 (ms1
plate form) and M-11 (the emitter→address dependency) — none of them a false
statement, all of them ungrounded assertions.

**Not re-derived, per the brief:** the 29 rulings; §5's lowering choices and
§5a's WU measurements; the structure/glyph/cite gate results.
