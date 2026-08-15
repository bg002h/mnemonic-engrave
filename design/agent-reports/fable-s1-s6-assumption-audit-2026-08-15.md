# S1–S6 assumption audit — judgment calls made in writing (fable, 2026-08-15)

Brief: find the false or unverified assumptions in the unimplemented remainder
(S1–S6) of `IMPLEMENTATION_PLAN_multisig_build_repair.md`, and settle them under
plan §0.1 ("permissive on input, expressive on output, speak loudly when common
assumptions are made; defaults for spelling, never for stakes, and every default
is printed"). Every check below was RUN unless marked UNVERIFIED. Source paths
are `/scratch/code/shibboleth/seedhammer` at `c94c135`; CLIs are the pinned
oracles (`md 0.13.0`, `mk 0.13.0` = mk-codec 0.5.0 at `a38a908`, `ms 0.15.0` at
`ddfa497`), invoked by absolute path.

Settled facts honoured, not re-derived: S0b green (driver, needle gate, census,
`oracle.DeriveExpected`/`CompareCensus`); mk1 byte-identity (mk-codec 0.5.0);
F-173 `0..n`; F-175 (S1 recordless on the D-1 arm).

---

## 1. DECISIONS TABLE

Each row is a judgment call now settled. "verified" cites the command or
file:line that was actually run/read.

| # | stage | the assumption | verified? (how) | THE DECISION |
|---|---|---|---|---|
| 1 | S1 | Over-supply (payload cards > open slots) is resolved by selection | payload carries 4 cards (`cmd/buildpayloadcards/main.go:53-58`); Trace A needs 2; `buildCosignerCards` exact-count refusal at `gui/multisig_build.go:268-270` | Selection UI **bounded to the open-slot count**, preserving **payload record order** among the selected (no reorder — order is identity-bearing, `md/encode_multisig.go:13-21`). When card count == open slots, **auto-fill and go straight to review** — the review screen is the §0.1 announcement; selection appears only on over-supply. The named cannot-fit refusal remains as a structurally-unreachable backstop, not a reachable arm. |
| 2 | S1 | Under-supply (incl. zero cards) refuses | correct pre-S5: payload is the only cosigner source (spec §3.1); no derived-cosigner mechanism exists until S5's slot model | KEEP the refusal through S4; text names the only real route (rewrite the payload on the host: `me sysw pack`), with the zero-card row included per F-173. **From S5 the refusal narrows**: it fires only when payload cards + held/derived slots together < n, because S5's model lets a typed seed fill an open slot. |
| 3 | S1 | The stage walk can satisfy `takeAll`'s loaded/compared guard | `take` guard read at `gui/sysw_session.go:114-118`; `[compared]` route 2 (operator digest comparison) at `gui/sysw_load.go:160-174`; the S0b driver currently taps Back past the boot offer (`cmd/emu/walk_build_policy.js:157` area, stated in its own comment) | S1's driver grows the **payload leg**: boot offer → Load payload → digest screen → confirm-match (route 2) → Engrave Multisig → Build policy. `assertNoNFC` (presented()==0) is already in the driver (`walk_build_policy.js:181,196`) — keep it on every stage-gate run. |
| 4 | S2 | The md1 is compared **by production** against the primary | **MEASURED TRUE — mechanism proven today.** `md encode "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))" --key @0=… --key @1=… --path "m/48'/0'/0'/2'" --group-size 0 --force-chunked` is **byte-identical, 4/4 chunks**, to the fork's `md.EncodeMultisig` on the same inputs (scratch run, two masters from `ms derive --template bip48-p2wsh`), and deterministic across runs | S2 owns extending the oracle: a new `ExpectKind` ("built-policy") in `oracle/expect.go` invoking **`md encode … --json`** and reading the `chunks` array — measured keys `{chunk_set_id, chunks, network, schema}`. Do NOT parse plain stdout: chunked `md encode` prints a `chunk-set-id: 0x…` line on **stdout** ahead of the strings, which a line-splitter (the `mkEncode` pattern) would ingest as an expected artifact. The built policy's mk1 stub is derivable: `--policy-id-fingerprint` / `wallet-policy-id-fingerprint`. |
| 5 | S2 | Interim foreign-origin refusal (plan picks REFUSE where spec allows refuse-or-warn) | reachable: the card's declared origin survives decode as `mk.Card.Path` (`mk/mk.go:135`); compare against `multisigSharedOrigin()` (`gui/multisig_build.go:421-424`), normalizing `h` vs `'` (the `samePath` precedent, `oracle/expect.go:253`) | **KEEP REFUSE — and it is §0.1-compliant, not just test-shaped.** §0.1 governs inputs that *underdetermine* the output; a card *declaring* an origin the flow cannot yet honour is determined input being **contradicted**, and no clause-1 authority licenses re-stamping a declared origin. The refusal text must speak phase-1 language: name the slot, the card's declared origin, the device's current limit ("this build supports one shared origin until divergent origins land"), and the host route. S5 removes it, as planned. |
| 6 | S2 | Interim duplicate-key refusal | plan's claim "no duplicate check exists anywhere" — confirmed by grep over `gui/` (no chaincode‖pubkey pair comparison outside `findUserSlot`'s match loop) | KEEP. §0.1 itself lists duplicate final-slot keys permanently on the refuse side (quorum degradation: `sortedmulti(2,K,K,X)` spendable by K alone). |
| 7 | S3 | An `sh(wsh)` build is walkable | template picker offers it: `"sh(wsh) (nested segwit)"` at `gui/multisig_build.go:276-282`; the payload cards are policy-agnostic (stub deliberately unchecked, `cmd/buildpayloadcards/main.go:40-43`) | Walk it **at the locked 2' shared origin** — do NOT make the origin template-aware before S5 (see row 10's interaction: it would strand this very walk on S2's refusal). From S2 on, the review screen carries the origin announcement (row 10's text). |
| 8 | S3 | The TYPED-ONLY inventory (9 in `gui/`, 1 in `cmd/emu`) | re-measured 2026-08-15: `grep -rn TYPED-ONLY --include='*.go'` → `gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2, `gui/multisig_build.go` ×1, `cmd/emu/embed_confinement_test.go` ×1 — **exactly as the plan says** | Execute as written; gate scoped to `gui/`, retire the `cmd/emu` citation as stage work. |
| 9 | S4 | The walk's `both`-slot fixtures exist in S0's payload | READ: payload = A@0 (m/48'/0'/0'/2'), A@1 (…/1'/2'), B@0, C@0 + **masterA's ClassMnemonic** (`cmd/buildpayloadcards/main.go:53-58,105`) | Honest direction: assign A@0 as `both` against the payload seed (masterA) → PROCEED. Dishonest: assign B@0 (or C@0) as `both` against masterA → FAIL naming the slot. **No payload change needed** — do not touch the pinned blob. |
| 10 | S4/S5 | Derived-slot origin is `m/48'/0'/account'/2'` for **every** template (spec §4.3 M-B) | **BIP-48 assigns 1' to nested segwit** — measured from the primary: `ms derive --template bip48-p2sh-p2wsh` → `m/48'/0'/0'/1'`; `oracle/expect.go:241-248` maps 1'→p2sh-p2wsh, 2'→p2wsh and refuses others "which BIP-48 does not register" | **RULED, and it is §0.1's own worked example pointed back at this plan.** (a) From S2: the review screen prints the origin with provenance — see §4 for the text. (b) **At S5** (divergence expressible, interim refusal gone): self/`derived` slots default to the template's BIP-48 assignment — 2' for wsh, **1' for sh(wsh)** — announced; payload cards keep their own declared origins (R-3); mixed origins → `OriginDivergent`, which S5 builds anyway. (c) Legacy `sh`: BIP-48 registers nothing — keep the device's 2' convention and SAY SO (§4 text); refusing would block the user and the path is printed in every artifact, so §0.1 clause 2 does not bite. Spec M-B's `…/2'` reading is corrected to "the BIP-48 script-type component for the chosen template", effective S5. **Do not apply (b) before S5**: every payload card declares 2', so a template-aware shared origin plus S2's same-origin refusal would make the S3 walk unwalkable — measured interaction, not a guess. |
| 11 | S4 | The walk-away bound is S4's to rule (idle-scrub vs recorded non-wiping) | `wipeGuard` brackets only the unlock session (plan claim, consistent with `gui/` read); the engrave tail derives everything **before** cutting (`deriveMultisigLeg` at `gui/multisig_build.go:162` precedes `bundleEngrave` at `:168`) | **RULED: scrub the whole seed registry at tail-start** — the moment the last derivation completes, before the first plate cuts. No idle timer (a mid-engrave scrub would kill an hours-long Trace B run for nothing; re-entry is the costly path). The session record stays non-wiping per SYSW§3.2.1; §5.3 already states that residue. Record this in S4's notes as the "explicit recorded decision" the plan demands. This *tightens* §4.2 ("MAY be retained for the duration of the constructor only") — the working copies now end at the constructor/tail boundary. |
| 12 | S5 | The mk1 gate relation is "the two-part mk1 relation" | FALSE — stale text; see §2 item 1 | Every mk1 is compared by **full string equality**, same as md1 and ms1 (§1a as corrected 2026-08-15). |
| 13 | S5 | The primary can produce Trace B's **divergent** md1 for the byte comparison | **MEASURED NO** — see §2 item 2 | **File the upstream md-cli change NOW** (Rust-first, with vectors): a per-key origin flag (`--origin @i=<path>`, or accept origin-qualified `--key @i=[fp/path]xpub`). Land + re-pin before S5 starts. Fallback ONLY if upstream misses S5: primary `md decode --json` equality on {template, k, n, per-slot origins, pubkeys} plus `md verify --template … --key @i=…` — recorded explicitly as a gate deviation (S2's own gate calls decode "the weaker relation"), never silently substituted. |
| 14 | S5 | Trace B's two masters both arrive from the payload | payload carries **one** `ClassMnemonic` (masterA); `take` returns the first match only (`gui/sysw_session.go:114`) | Master B's seed is **TYPED** in the Trace B walk (the seed picker offers typed everywhere, `gui/derive_xpub.go:140-142`). The walk script drives the keyboard via `shTap`. Keyboard-driving is UNVERIFIED (§3) — prove it with one word before S5's walk is written. |
| 15 | S5 | (implied by `0..n` + §4.1's 11-key example) an all-held build needs a payload | 0-card payloads are ruled legitimate; §4.1's own pathological example holds ALL keys | **Post-S5, an all-derived build with zero payload cards MUST assemble** — no payload requirement may gate the flow when no slot sources from one. Covered by unit test, not a new walk (keep walk scope at Trace B). |
| 16 | S5 | ms1-first plate order + "discard" abort wording | CONFIRMED: full mode appends ms1 first (`gui/multisig_engrave.go:11-20`); `bundleAbortWarning` says "discard the engraved plate(s)" (`gui/bundle_flow.go:349-356`) | Ship the DESTROY wording as planned (cards-derived); reordering stays deferred with its filed spec question. |
| 17 | S5 | Re-run mints byte-identical plates (deterministic encoders) | CONFIRMED: `grep -rn "math/rand\|crypto/rand" md/ mk/ codex32/` → zero hits; primary `md encode` chunk-set-id identical across two runs; mk determinism settled (mk-codec 0.5.0) | Pin with the `shToolpath` digest-equality check as planned; put the recovery text on the **abort screen**, as the plan already rules. |
| 18 | S6 | A hardware payload with the walk's cards can be produced | spec §8 verified `me sysw pack` on a 2-chunk card; the 4-card+mnemonic records file comes straight from `go run ./cmd/buildpayloadcards` | Pack the hardware payload from the same records file. UNVERIFIED until run (§3) — run `me sysw pack` over it once, before the flash cycle. |
| 19 | oracle | The pinned `ms` matches the settled ms-cli 0.16.0 ruling | MEASURED: installed+pinned `ms 0.15.0` (`ddfa497`, `oracle/pins.json`); 0.16.0 exists upstream (`mnemonic-secret 98e1f6a`) | No gate is affected today — `oracle/expect.go` uses explicit templates (`bip48-p2wsh`, `bip48-p2sh-p2wsh`), present in 0.15.0. **Re-pin ms (commit + sha256) in its own commit at the next oracle rebuild**; a rebuilt binary without a re-pin fails the tier-2 real-pin test (`oracle/oracle_test.go:283`) by design. |
| 20 | S1–S5 | Plan/spec say `card.Origin` | the field is **`mk.Card.Path`** (string), `mk/mk.go:135`; no `Origin` field exists | Read "the card's origin" as `Card.Path`; compare origins component-wise (normalize `h`/`'`), never by string equality. |

---

## 2. FALSE ASSUMPTIONS FOUND

**1. S5's gate names a relation the plan already abolished.** Plan `:1164`:
"each mk1 must satisfy **the two-part mk1 relation**". §1a (corrected
2026-08-15, `:185-205`) rules the opposite: mk1 is **full string equality**, and
"the weakened two-part relation is gone" — mk-codec 0.5.0 (`a38a908`, =
`oracle/pins.json` mk pin) made `chunk_set_id` payload-derived. Cost: the
implementer greps for a definition that no longer exists, or worse
re-implements the weak relation and under-asserts the gate. Corrected
statement: *every artifact class — md1, mk1, ms1 — compares by full string
equality against the pinned primary.* (Classic incomplete-propagation fold
defect; the superseded phrasing survived in S5.)

**2. S5's divergent md1 byte-comparison has no producer.** The gate assumes the
primary can BUILD Trace B's md1. Measured against `md 0.13.0`:
`--key "@0=[fp/48'/0'/0'/2']xpub…"` → `base58check decode` error; a
concrete-key descriptor → `template contains no @i placeholders`; `md encode
--help` exposes no per-key origin flag; omitting `--path` yields
`path_decl {tag: Shared, data: "m"}` (checked via `md decode --json`). The
codec itself decodes Divergent fine (the fork's divergent encode round-trips
through `md decode --json` with `tag: "Divergent"` and both paths). So the
CLI, not the codec, is the gap. Cost: S5 reaches its gate and discovers the
comparison cannot be run — the exact "gate that has never executed" failure
this cycle already paid for once. Decision: table row 13 (upstream flag now,
Rust-first; named fallback recorded as a deviation).

**3. S2/S5's oracle comparisons name a mechanism the oracle does not have —
but the primary-side mechanism is now PROVEN.** `oracle.ExpectKind` is a
deliberately closed set containing exactly one kind, `KindCosignerCards`
(`oracle/expect.go:60-71`); nothing derives an expected md1 or ms1. That is
stage work S2/S5 own — and the hard half is now measured: the shared-origin
md1-by-production invocation is **byte-identical** to the fork
(`md.EncodeMultisig` vs `md encode … --path "m/48'/0'/0'/2'" --group-size 0
--force-chunked`, 4/4 chunks, deterministic; keys derived via the pinned
`ms derive --template bip48-p2wsh`). Two implementer traps found by running
it: chunked `md encode` prints `chunk-set-id: 0x…` on **stdout** ahead of the
strings (use `--json`, key `chunks`), and `--force-chunked` is required (a
2-of-2 payload is 229 symbols and the CLI errors rather than auto-chunking).

**4. The locked shared origin claims BIP-48's native-segwit path for every
template.** `multisigSharedOrigin()` is `m/48'/0'/0'/2'`
(`gui/multisig_build.go:421-424`) and spec §4.3 M-B bakes `…/2'` into
`derived` slots template-blind — but BIP-48 assigns **1'** to nested segwit
(measured: `ms derive --template bip48-p2sh-p2wsh --json` →
`"account_path":"m/48'/0'/0'/1'"`; `oracle/expect.go:241-248` encodes the same
assignment and refuses everything else) and registers nothing for legacy
`sh`. No S1–S6 stage names this, and the operator's §0.1 sentence names
BIP-assigned script-type paths as its flagship example of an assumption that
must be spoken. It is funds-safe (addresses derive from keys; origins are
printed in every artifact) — which is exactly why it is permitted WITH
announcement rather than refused. Decision: table row 10, announcement text in
§4. Cost if unfixed: an sh(wsh) wallet whose steel tells a BIP-48 coordinator
to derive at the wrong script-type path, silently.

**5. Stale pin text in §1a.** `:149` still says "the pins today are …
mk-codec 0.4.2" three paragraphs above its own correction to 0.5.0. And the
plan-wide "ms-cli 0.16.0" settled ruling coexists with an installed+pinned
`ms 0.15.0`. Neither breaks a gate (row 19); both will cost a careful reader a
re-derivation. Corrected statement: pins are what `oracle/pins.json` says —
that file, not §1a's prose, is the authority.

**6. Small drifts, noted so they cost nobody a search:** the decoy needle
citations are `gui/multisig_build.go:122` and `gui/singlesig.go:95` (plan says
121/94 — its own "re-measure before use" instruction covers this);
`card.Origin` does not exist, the field is `mk.Card.Path` (`mk/mk.go:135`).
Everything else cited by S1–S6 was checked and resolves: the five
`bundleGatherFlow` call sites, the `"Engrave Bundle"` literals at
`multisig_build_flow_test.go:239,249`, `scriptName`'s three callers, the
raster helpers (`runUITouchRaster`/`countInk`/`assertFrameHasBody`),
`take`'s guard, `findUserSlot` deriving at `k.OriginPath`
(`gui/multisig_match.go:35-40`), `buildMultisigSeedHook`
(`gui/multisig_build.go:73`), `bundleEngrave` at `:168` /
`multisigRestoreDocFlow` at `:191`, `errMultisigEmptyDivergent`
(`md/encode_multisig.go:102-106`), `multisigEngraveCards` /
`multisigVerifyFlow` signatures, the empty-census refusal
(`oracle/record.go:166`), and the S0b driver's four-needle + zero-NFC proof
(`cmd/emu/walk_build_policy.js:181-196`, `needle_test.go`,
`nfc_presented_test.go`).

---

## 3. UNVERIFIABLE WITHOUT EXECUTION

Marked so nobody mistakes them for checked. Each with the cheapest settling
experiment.

1. **D-1 itself.** Still unreproduced; the payload feed (S1) is the first time
   the flow past the gather becomes reachable. The plan's two-armed S1 gate is
   the experiment. Nothing cheaper exists — this is by design.
2. **The S1 driver's payload leg** (Load → digest → confirm route 2). The
   mechanism exists (`gui/sysw_load.go:160-174`) and was walked by hand for the
   Load Payload journey; no automated walk has driven it. Experiment: extend
   `walk_build_policy.js` with the three taps and run once — minutes.
3. **Keyboard seed entry under `shTap`** (Trace B needs master B typed; row
   14). Experiment: drive one word ("legal") on the seed-entry screen in the
   emulator and assert the tally; do this before S5's walk is written, not
   during it.
4. **`me sysw pack` over the full 4-card + mnemonic records file** (S6's
   hardware payload). §8 of the spec verified a 2-chunk pack only. Experiment:
   `go run ./cmd/buildpayloadcards | me sysw pack …` once, host-side.
5. **The walk-run procedure end to end on this machine** (serve wasm on a
   fresh port, fire-and-forget `run()`, save `window.__walk`, `go run
   ./cmd/gaterecord`). Documented in `cmd/gaterecord/main.go:14-34` and done
   once for S0; not rehearsed for the build-policy driver. Experiment: one dry
   run of `walk_build_policy.js` today — it needs no S1 code, it ends at the
   gather.
6. **S6 hardware items** (external-coordinator restore, mid-set interruption,
   ms1 read-back): hardware-only by definition; the plan already scopes them
   to the one flash cycle.

---

## 4. REFUSALS THAT SHOULD BECOME PERMISSIVE (and the ones that must not)

Every refusal in S1–S6, run past §0.1. "Announce" texts are operator-facing
drafts; polish freely, do not weaken.

- **S1 over-supply** — ALREADY re-ruled by F-173; this audit adds the shape:
  bounded selection, equal-count auto-fill, record-order preserved (row 1). The
  reachable refusal disappears; the review screen announces: *"Slots @1,@2
  filled from the payload (cards 3 and 4 of 4, in payload order)."*
- **S1 under-supply / zero cards** — STAYS A REFUSAL (clause 1: the device
  cannot invent keys), but must name the real route: *"The payload holds 1
  cosigner card; this policy needs 2. Rewrite the payload on the host
  (`me sysw pack`) and load it again."* From S5, only fires when derived slots
  cannot cover the gap (row 2).
- **S1 `takeAll` before compared** — STAYS (authentication is stakes, clause
  2: a swapped card is invisible with fingerprints omitted by default).
- **S2 foreign-origin (interim)** — STAYS, with the §0.1 reasoning made
  explicit: a *declared* origin the flow cannot honour is contradicted input,
  not an underdetermined spelling; re-stamping it has no authority. Text:
  *"Card @2 declares origin m/48'/0'/1'/2'. This build currently supports one
  shared origin (m/48'/0'/0'/2'). Divergent origins arrive in a later update;
  today, use a card at the shared origin or build this policy on the host."*
  Dies at S5.
- **S2 duplicate final-slot keys** — STAYS PERMANENTLY (§0.1 names it).
- **S4 `both`-slot mismatch / contradicting fingerprint** — STAY (funds
  stakes; the FAIL screen must name causes and say reassignment suppresses the
  check, as the plan already rules).
- **S5 depth-0 cosigner card** — STAYS as a *named* refusal: the encoder
  refuses `Path == "m"` in divergent mode (`md/encode_multisig.go:104-106`)
  and changing that is normative codec behaviour (Rust-first, out of scope).
  Text: *"Card @3 declares no derivation path ('m'). A divergent policy needs
  each card's account path. Re-mint this card on the host with its origin
  path."*
- **The one refusal-shaped default that BECOMES ANNOUNCED PERMISSIVENESS**
  (row 10): the template-blind `…/2'` origin. From S2, review screen: *"Key
  origins: m/48'/0'/0'/2' — the BIP-48 path for native segwit."* For sh(wsh)
  until S5: append *"Note: BIP-48 assigns m/48'/0'/0'/1' to nested segwit;
  this build uses the shared 2' path until per-card origins land."* From S5,
  sh(wsh) derived/self slots default to 1' and the note becomes *"Key origins
  follow BIP-48 for nested segwit (script type 1')."* Legacy sh, always:
  *"No BIP assigns a derivation path for legacy P2SH multisig; using this
  device's convention m/48'/0'/0'/2'. It is recorded on every artifact."*
  Explicitly chosen origins (payload cards) announce nothing — a tool that
  cries DEFAULT when the operator chose is a tool whose warnings get ignored
  (§0.1's own corollary).

---

## 5. WHAT WOULD MOST HELP THE USER DO THE THING (ranked)

1. **S1+S2, unchanged in scope** — the critical path to the first completed
   on-device build (Trace A). Everything in them serves the goal directly.
   With the mechanisms verified above, neither stage should hit a surprise.
2. **File the upstream md-cli divergent-origin flag TODAY** (row 13). It is
   the only S5 gate dependency with external lead time (Rust-first change +
   release + re-pin), and S5 is the flagship. Filing it now costs an hour;
   discovering it at S5 costs the stage.
3. **S2's oracle extension using the proven invocation** (row 4). It is the
   mechanism every later md1 gate reuses; the byte-identity result above means
   it is transcription now, not research.
4. **S5 itself** — the wallet the operator actually described (multi-slot,
   divergent, both masters backed up) is S5's output; S1–S4 are its runway.
5. **The BIP-48 origin announcements** (row 10, §4) — the goal sentence's own
   named example of "speaking loudly", currently absent from every stage.
   Cheap lines, high trust value, and the only §0.1 violation found that no
   stage owned.
6. **S4** protects the thing rather than enabling it — correctly ordered
   before S5 (operator ruling, "Safety first") since the D-5 exposure is live.
   **S3** is small and real (the restore doc lies about nested segwit today).
   **S6** is validation, not delivery, and its divergent/multi-master run is
   what makes the steel trustworthy. None of S1–S6 is padding; the only
   non-user-facing work (S3's comment sweep) is minutes and prevents a
   documented failure class.

Nothing in S1–S6 needs to be removed or reordered. The plan's remaining risk
was concentrated in the four unbuilt mechanisms (md1/ms1 oracle kinds, the
divergent producer, the payload leg, the keyboard leg) and one unspoken
default (BIP-48 script-type origins) — all now either measured, ruled, or
named UNVERIFIED with a cheap experiment attached.
