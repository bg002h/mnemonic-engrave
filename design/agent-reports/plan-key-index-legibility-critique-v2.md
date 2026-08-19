# Adversarial critique v2 — PLAN_key_index_legibility.md @ 53b2e82

Reviewer: independent read-only agent, 2026-08-19. Scope per brief: attack v2's
headline, root cause, fix direction, representation, edge cases, demonstration,
§4/§5, and omissions. Every claim marked **executed** below was run on this
machine (24 threads) against the release binaries
(`mnemonic-toolkit/target/release/mnemonic`, `descriptor-mnemonic/…/md`,
`mnemonic-key/…/mk`, `mnemonic-engrave/…/me`) and the committed fixture in
`design/journeys/inputs-pathological/`, rebuilt exactly as
`transcript_pathological.sh` builds it (same encode flags, same stub). The
root-cause experiment ran in a scratch crate against
`descriptor-mnemonic/crates/md-codec` (path dep), not against prose.

## Verdict

**v2 is not sound to execute, and the weakest item is its lead.** §1's headline
claim — *"No command in this repo prints that value"* — is **false, executed**:
`mnemonic verify-bundle` in its multisig-template form prints
`wallet-policy-id: ced2270948ecb5af0779249ac7181f4a` on stdout in **17 ms**
from exactly the inputs an operator holds at backup time, and even prints a
warning telling the user to record it. §1's root cause is also **wrong,
disproven by experiment**: the `WalletPolicyId` preimage stores xpubs as
65-byte `[chain_code‖pubkey]` (`md-codec/src/canonicalize.rs:361-363`), so
F-130's zeroed BIP-32 metadata *cannot* move the id; the real cause is the
journey's keyed encode flattening per-key origins (`--path bip48`) and omitting
per-key fingerprints — substituting the true origins + fingerprints into the
decoded keyed md1 reproduces `ced22709…` **byte-exactly** with the xpubs
untouched. Of §1's two candidate homes, (b) is structurally impossible (`me`
cannot know the card→slot assignment the id depends on — §2's own central
fact) and (a) is mis-specified around the wrong mechanism and blocked by a
producer gap the plan does not name. §2 is genuinely close to executable
(representation decided, rendering verified correct) but its acceptance list
dropped two touch points R0 had already named, and its edge table dropped a
producible case. §3's demonstration is real but its stability rests on a
gate the plan never analysed: the engine's *estimate* (not the 16 s realized
time) is what refuses, and it varied 189.6 s → 1284.9 s across identical runs
on this one fast machine against a 3600 s ceiling. §4 is now actionable. The
plan's order (§1 → §2 → §3 → §4) collapses with §1: §3 is buildable **today**
with zero new code.

## Holes

### C1 (Critical) — The headline is false: the recovery target IS printed, by the constellation's own backup-verification command

Plan §1: *"`--expect-wallet-id` needs `ced2270948ecb5af…`. No command in this
repo prints that value."* Table row: printed by **"nothing"**.

Executed — bundle rebuilt per the journey (3 md1 chunks, 11 cards, stub
`5b48af35`), then:

```
$ mnemonic verify-bundle <3 md1 chunks positional> \
    --cosigner @1=… … --cosigner @10=… \
    --from phrase=<master-A> --account 0 --network mainnet
✓ md1_template_match: supplied md1 matches the recomposed wallet's keyless template …
✓ wallet_completed: completed WalletPolicyId ced2270948ecb5af0779249ac7181f4a; first receive bc1qkuknuy6…
OK (multisig template recomposed)
wallet-policy-id: ced2270948ecb5af0779249ac7181f4a
first recv:  bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
your seed completes cosigner slot @0
[wall clock 0.017s, exit 0]
```

Source: `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1052`
(`writeln!(stdout, "wallet-policy-id: {completed_id_hex}")`), computed via
`md_codec::compute_wallet_policy_id(&outcome.completed)` — the identical value
restore matches, through the identical shared engine (`verify_bundle.rs:860-954`,
"verify == restore"). The tool even nudges the operator toward §1's goal
unprompted: *"Record + check --expect-wallet-id or a receive address."*

Three aggravations:

1. **The plan's own §0 transcript shows the value being printed** — `✓
   wallet-id (completed): ced2270948ecb5af0779249ac7181f4a` is restore's own
   stdout. Explicit-assignment restore (no search, seconds, exactly the
   knowledge an operator has at the generation ceremony) prints it too — and R0
   §F3 had already told the author: *"The only producer I found is `restore`
   itself in explicit mode."* v2 hardened that nuanced true statement into a
   false absolute and never checked the one other command whose doc string says
   it shares restore's engine.
2. The word "verify" appears **zero** times in the 266-line plan. Neither
   journey runs `verify-bundle` either (grep: no hits in either transcript
   script) — so the *journey* gap is real, but it is a **documentation/journey
   gap, not a missing feature**, and §1 proposes building a feature.
3. The printed `first recv` doubles as the assignment check: for this unsorted
   `multi()` wallet the address depends on the key→slot assignment, and the
   journey's §9b already prints coordinator-checkable addresses — so the
   explicit-mode warning ("a wrong assignment produces a wrong wallet
   silently") is closed by a cross-check the journey already performs.

Consequences: §1's "The change" is redundant with existing tooling; §3's
"Dependency — Step 2 cannot ship before §1" is **false** (the journey can print
the target today, from an earlier step, satisfying §3's own acceptance); §6's
ordering ("§1 leads: §3 is unbuildable without it") falls; §7.1's two-home
question dissolves.

### C2 (Critical) — The root cause is wrong, and the experiment the plan should have run disproves it

Plan §1: *"`restore` re-serialises the completed descriptor's xpubs with BIP-32
metadata zeroed. … Different serialisation → different descriptor string →
different `WalletPolicyId`. This is F-130."*

The metadata dump is real (executed: committed key-00 xpub parses to `depth=4
parent_fp=1cf29716 child=80000002`; the completed descriptor's xpubs are all
`xpub661…` depth-0 re-serialisations). But the causal arrow into the **id** is
false at the mechanism level:

- `compute_wallet_policy_id`'s per-`@N` preimage is `presence_byte ||
  varint+origin-path bits || varint+use-site bits || fp? || xpub?` where the
  xpub is **`Option<[u8; 65]>` — "65-byte xpub (32 chain-code || 33 compressed
  pubkey)"** (`md-codec/src/canonicalize.rs:352-363`, `identity.rs:186-285`).
  Depth, parent fingerprint and child number are **not in the preimage**; they
  are not even on the md1 wire (`restore.rs:2837-2849` reconstructs from
  65 bytes). Zeroed metadata cannot move this id. F-130's true scope is what
  `journeys/README.md:63` always said: the descriptor **string and its
  checksum** — not the id.
- **Executed, scratch crate against md-codec** — decode the journey's keyed
  md1, then mutate one field at a time:

  | descriptor state | WalletPolicyId |
  | --- | --- |
  | as decoded (`--path bip48` ⇒ Shared origin, no fps) | `232214e4d60c0fa83a6715ba2f7e8ec7` |
  | + real Divergent origins (`48'/0'/{0,1,2,3,0,1,2,3,0,1,2}'/2'`) | `6b6ed3bed6f877ce0a150490203c1e3d` |
  | + real origins **and** real fingerprints (A/B/C) | **`ced2270948ecb5af0779249ac7181f4a`** |
  | fps only, Shared origin kept | `9f5da760a03ccd0c0d8e3ea819a31358` |

  The xpub bytes were **never touched** and the target id appears exactly when
  origins + fingerprints match the real wallet. The id gap is caused by (i)
  `md encode --path` *"flattens Divergent mode to Shared"* (its own doc string,
  `md-cli/src/main.rs:93-97`; the journey passes `--path bip48`) and (ii) the
  journey's keyed encode passing no `--fingerprint @i=` — R0's F3, which v2
  cites facts from but whose cause it replaced with F-130.
- Confirmed from the other side: re-encoding the keyed md1 from
  metadata-zeroed xpubs is **refused** (`md: --key @0: expected depth 4 for
  this script context, got 0`) — and had it encoded, it would carry the same
  65 preimage bytes anyway.

Why Critical: this is v1's exact failure class recommitted — a true finding
("F-130 exists") "connected" to a phenomenon it does not cause, by inference
instead of by running the check. An implementer building §1(a) as specified
("prints the **restored-serialisation** id") would zero xpub metadata,
recompute, get `232214e4…` again, and ship a **fifth** id.

### I1 (Important) — Of §1's two candidate homes, (b) is structurally impossible and (a) is blocked by an unnamed producer gap

- **(b) `me bundle` checklist header** cannot work: the id's per-`@N` records
  are assignment-ordered (that is *why* restore must search 11! of them), and
  §2's own core fact is that `me` **cannot know** which card sits at which
  `@N` for a keyless template. `me` therefore cannot compute `ced22709…` at
  all. The plan offers as a candidate home a design its own §2 proves
  impossible; an implementer discovers this mid-build.
- **(a) `md inspect --restored-form`** has no input to work on: the keyless
  md1 has no keys; the journey's keyed md1 lacks the origins and fingerprints
  the id needs (C2); and `md encode` today cannot embed per-key origins
  (`--path` is a single shared path; encoding keyed **without** `--path` emits
  a partial-decodable card — executed: *"without an explicit origin, `md
  decode`/`md inspect` will only PARTIAL-DECODE this card (origin unspecified,
  exit 4)"*). So (a) requires md-cli/md-codec producer work the plan never
  names.
- The third home — the one that already exists (`verify-bundle` / explicit
  restore, C1) — is absent from the list.

### I2 (Important) — §3's stability analysis is missing: the gate is the estimate, not the 16 s

Executed, same command as the plan's §0 (shuffled cards, id-search): realized
wall **15.3 s**, exit 0 — the 16 s claim reproduces. But:

- What gates is `cap_decision`'s single-threaded **estimate** from a 64-sample
  calibration (`permutation_search.rs`), ceiling 3600 s — not the parallel
  realized time. Printed estimates for the *identical command on the identical
  machine*: **189.6 s** (plan §0), **413.1 s** and **913.7 s** (this session),
  **187.4 / 196.7 / 927.7 / 1284.9 s** (R0's four runs). That is ~7×
  calibration noise with worst-case headroom ~2.8× under the ceiling — on a
  fast 24-core box. On a materially slower runner the estimate plausibly
  crosses 3600 s and the journey step **refuses (exit 1)**, failing a journey
  §3 says "must complete every run". The plan measured once and never analysed
  the gate it documented in the very next sentence.
- Cost, measured: ~293 CPU-seconds per run (1912% CPU for 15.3 s) — a
  40M-candidate search inside a document-generation script, every regeneration.
  Defensible as a demonstration, but the plan should either pass a generous
  `--accept-search-time` in the journey step (removing the machine-dependence)
  or state the refusal as an accepted journey outcome. Note the journeys are
  **not run in CI** today (`.github/workflows/` contains `release.yml` only),
  so the blast radius is operator machines, not CI.

### I3 (Important) — §2's acceptance dropped two touch points R0 had already named

`crates/me-cli/tests/vectors/bundle-md1-mk1.json` contains **two `mk1-chunk`
plates** (executed: plates 2 and 3 of 4) which **will** gain the new fields —
`skip_serializing_if` only spares the md1/ms1 plates — and that golden is
byte-compared twice (`cli.rs:306`, `cli.rs:743-745` — *"no --preview must be
byte-for-byte Phase A"*). §2's sentence *"existing manifest consumers are
unaffected"* is misleading against its own test suite. And
`design/SPEC_me_bundle_phaseA.md` §6 writes the manifest schema out
field-by-field (line 61), so the additive field is a spec delta. Both were
named in R0 (D2/F5); the rewrite lost them. The four *runtime* consumers claim
is correct (verified: exactly `transcript.sh`, `transcript_pathological.sh`,
`build_pdf_pathological.py`, `cli.rs` reference the manifest).

### I4 (Important) — §2's edge table dropped a producible case: the empty origin path

Executed: a depth-0 xpub encodes with `--origin-path m` (`mk encode --xpub
<depth-0> --origin-path m --policy-id-stub 5b48af35` → 2 mk1 chunks, exit 0);
depth-4 xpubs are refused with pathless origins, so the case is exactly the
depth-0/no-path key `key_card.rs:50-56` documents. Rendered per §2's spec it
produces `mk1 [73c5da0a/] chunk 1/2` (trailing slash, empty path) or
`mk1 [path , no fingerprint]` — neither defined. R0's D4; dropped in the
rewrite. (§2's other four cases are sound, and the `set <chunk_set_id>`
disambiguator adequately covers both the same-origin collision and the
multi-wallet-bundle case, since set ids differ across wallets.)

### M1 (Minor) — §5's rationale re-asserts the contradiction the first critique flagged, unacknowledged

*"§4 gets the benefit without the coupling"* — critique I3 showed §4's
convention carries the same coupling one layer down (a key serving wallet A at
`@1` cannot satisfy account=index in wallet B at `@2`; the convention forces a
per-(wallet, index) derivation). §5's **conclusion** (no mk1 wire field) is
right and independently supported — normative wire change, standing decision —
but the stated reason is the one already refuted, and v2 neither fixed nor
acknowledged it. Rationalising, with the right answer.

### M2 (Minor) — §4 never says which "template index" the convention binds

R0's D11: `canonicalize_placeholder_indices` renumbers `@N` to
first-occurrence order at encode; author-order and canonical-order coincide for
this fixture but not in general. The §4 README section needs one sentence
("the `@N` shown by `md decode` of the engraved template") or two writers
diverge.

### N1 (Nit) — Measured constants quoted as if stable

`est. ≤ 189.6048s` and `890788.897152s` are single-run calibration outputs; my
runs printed 413.1 s / 913.7 s and 1,731,868 s for the same commands. The 84 ms
refusal was 151 ms here. Facts hold in kind; the six-significant-figure style
invites false precision (and the estimate variance is load-bearing — see I2).

### N2 (Nit) — Citation points at the checkout, build uses the registry crate

`KeyCard` is cited at `mnemonic-key/crates/mk-codec/src/key_card.rs:34-57`;
`me-cli` builds against registry `mk-codec 0.4.1` (`Cargo.lock`). Same fields
(verified in both). R0 flagged this; unchanged.

## Claims checked

| Claim (v2) | Holds? | Evidence |
| --- | --- | --- |
| `WalletDescriptorTemplateId` = `5b48af35d4321a3ac18b43045e2523cc` | **Yes, executed** | `md inspect` on rebuilt keyless chunks |
| Keyless `WalletPolicyId` = `f89e23f13c697ae62ef10328d71d7e24` | **Yes, executed** | same command |
| Keyed-encode id = `232214e4d60c0fa83a6715ba2f7e8ec7` | **Yes, executed** | `md inspect` on the journey-style keyed encode; also reproduced from the decoded Descriptor in the scratch crate |
| restore matches `ced2270948ecb5af0779249ac7181f4a` | **Yes, executed** | id-search run, exit 0; explicit-mode verify-bundle prints the same |
| …"printed by **nothing**" | **NO — falsified** | `verify-bundle` prints it (stdout, 17 ms, `verify_bundle.rs:1052`); restore prints it in the plan's own §0 excerpt; explicit-mode restore prints it in seconds |
| Root cause = zeroed xpub metadata (F-130) | **NO — disproven** | preimage xpub is 65-byte `[chain_code‖pubkey]` (`canonicalize.rs:361-363`); mutation experiment: origins+fps alone reproduce `ced22709…`, xpubs untouched |
| `committed depth=4 parent_fp=1cf29716 child=80000002` | **Yes, executed** | base58check-parsed key-00.xpub |
| 16 s for the full 11! id-search | **Yes, executed** | 15.3 s wall (293 CPU-s, 24 threads); estimate printed 413.1 s this run — see I2 |
| `--search-address` refused, 84 ms | **Yes in kind, executed** | refused at 151 ms; estimate 1,731,869 s vs plan's 890,789 s (2× calibration noise) |
| Supplying `232214e4…` rejected | **Yes, executed** | full-search NO MATCH (R0 measured exit 4) |
| Seed required for keyless template (`restore.rs:1396`) | **Yes** | floor message at `restore.rs:1390-1400`; identical floor in `verify_bundle.rs:897-904` |
| `me-cli`: serde+serde_json, no `bitcoin` dep | **Yes** | `Cargo.toml` deps; only `bitcoin` hit is the keywords line |
| `mk-codec` does not re-export `bitcoin` types | **Yes** | registry 0.4.1 `lib.rs` `pub use` lines: KeyCard/decode/encode/consts/error only |
| Rendering: `'` markers, `fp/path`, no `m/` | **Yes, executed** | `mk decode` prints `origin_fingerprint: 73c5da0a`, `origin_path: 48'/0'/1'/2'`; bitcoin `Display` verified by R0 (D3) |
| `bundle.rs:279` decodes and discards the `KeyCard` | **Yes** | `mk_codec::decode(&refs).map_err(…)?;` value dropped |
| `manifest.rs:82-108` label; `:228` asserts `"mk1 chunk 1/2"` | **Yes** | read; exact |
| manifest has 4 consumers, one a test | **Yes** | grep: exactly the 4 named files |
| F-130 quote at `journeys/README.md:63` | **Yes, verbatim** | read — note it claims only string+checksum drift, which is precisely why C2's causal extension is wrong |
| §4: `@4` = master B account `0'` (`[b8688df1/48'/0'/0'/2']`) | **Yes** | key-04.xpub header (R0's F7 already adjudicated this against the first critique) |
| §1: "for an 11-key wallet there is no no-target recovery path" | **Yes, with a quibble** | `--accept-search-time` hatch exists; "~10 days" is the single-thread estimate — parallel wall on this box would be ~10–20 h. Not routine either way |

## What v2 still misses

1. **`mnemonic verify-bundle` — entirely.** Not in the plan (0 mentions), not
   in either journey. It is simultaneously the counterexample to §1's headline
   and the cheapest correct implementation of §1's goal.
2. **The real producer-side defect** behind the id confusion: `md encode
   --path`'s silent Divergent→Shared flattening plus no per-key origin flag,
   so no keyed md1 with true origins is producible (R0 F3 / open item). If
   that were fixed (Rust-primary repo, with vectors), `md inspect` on an
   honestly-keyed md1 would print `ced22709…` with **no new inspect flag** —
   the convergence v2's §7.2 gropes for, from the encode side.
3. **§7.2 can be answered now, and the answer is no.** restore and
   verify-bundle already agree on the completed-form id; the outlier is the
   journey's lossy keyed encode. "Normalising" restore to accept `232214e4…`
   would make the target not commit to origins/fingerprints, break any
   already-recorded completed-form id, and still leave the lossy encode lossy.
   Document the id taxonomy; converge the printers via (2); do not weaken the
   matcher.
4. **The zero-code carriers of the operator's suggestion, dropped without a
   decision.** `transcript_pathological.sh` writes `card-index.txt` and
   `build_pdf_pathological.py` captions every plate `@{ki} [{fp}/{path}]` —
   the journey PDF already names `@N` per plate today, which v2 never
   mentions. The first critique's fill-in `@__` ceremony table and the
   hand-marking checklist line (its misses #3/#7) are likewise gone without a
   recorded rejection. §4 does engage the convention half of the operator's
   suggestion (creation-time coordination) — that part is no longer
   sidestepped — but the "order numerically, write it down while everyone is
   in the room" half has cheaper carriers than anything in the plan.
5. **The `tr()` fixture variant** (`wallet-policy-tr.txt`) — still unexercised
   in every round including this one; project memory warns measuring one
   descriptor path gives a wrong answer about the other.

## Recommended changes

1. **Rewrite §1 around what exists** (dissolves C1, C2, I1): the recovery
   target is printed today by `mnemonic verify-bundle` (multisig-template
   form) and by explicit-mode `restore`. §1 becomes: add the verify-bundle
   step to the pathological journey at backup time (cards + own seed +
   explicit `@N=` assignments), cross-check `first recv` against §9b's
   coordinator addresses, and label the printed `wallet-policy-id` as *the*
   recovery target to record. Correct the root-cause paragraph: the id
   differs because the keyed encode flattens origins and omits fingerprints
   (preimage: origin, use-site, fp, 65-byte key — never BIP-32 metadata);
   keep F-130 scoped to string/checksum as its README row already states.
2. **Re-derive §3 and §6 from that**: §3 no longer depends on §1's code (the
   target id comes from the new verify-bundle step); §2 stays independent;
   the order becomes §3-enablement first, everything else parallel.
3. **Fix §3's stability** (I2): pass `--accept-search-time` (say, `2h`) in the
   journey's search step so the machine-dependent estimate cannot refuse, and
   state the measured estimate variance (189.6→1284.9 s on one machine)
   as the reason.
4. **Complete §2's touch list** (I3, I4): name the golden
   `tests/vectors/bundle-md1-mk1.json` regeneration and the
   `SPEC_me_bundle_phaseA.md` §6 schema delta as acceptance items; add the
   empty-path row to the edge table with an exact string (suggest
   `mk1 [73c5da0a, no path] chunk 1/2` / `mk1 [no origin] chunk 1/2`).
5. **File the producer follow-up** (misses #2): per-key origin support in
   `md encode` (Rust-primary, with vectors), so a faithful keyed md1 is
   producible and `md inspect` converges on the completed-form id; answer
   §7.2 as "no — converge the printers, don't weaken the matcher."
6. **§4/§5 patches** (M1, M2): pin "template index" to the canonical
   `md decode` numbering; either acknowledge that the convention shares the
   wire-field's coupling at derivation time (and argue that is acceptable
   because re-derivation is cheap where re-engraving is not) or drop that
   clause of §5's rationale.
7. **Record dispositions** for the dropped zero-code carriers (fill-in table,
   hand-marking line) — a sentence each, even if the answer is no.

## Open / could not determine

- **Behaviour on a genuinely slow machine.** The estimate-crosses-ceiling
  failure mode (I2) is extrapolated from measured single-machine variance,
  not reproduced on slow hardware.
- **Exit codes through my pipelines** were lost to shell plumbing twice; the
  refusal/NO-MATCH *text* was captured verbatim, and R0 measured the codes
  (1 for the ceiling, 4 for NO MATCH). Not re-measured here.
- **Whether `verify-bundle`'s multisig-template path postdates v1** (i.e.
  whether v1's author could have found it). Irrelevant to v2, which was
  written after the binary I ran was built.
- **The tr() variant** — untested, all rounds.
- **Whether any command prints the id in yet another place** (e.g.
  `export-wallet` fed a completed descriptor). Not exhaustively swept beyond
  the falsification found; one counterexample suffices for C1.
