# F-449 spec r4 — closing review

**Verdict: 0 Critical / 3 Important / 5 Minor.**

Reviewer: opus, closing round. Two questions only, per the brief. Artifact
`design/SPEC_liana_unspendable_internal_key.md` at `874317ae`; diff base
`9d909a30` (range includes `b3b46688`, the r3b fold, treated as in scope).

Ground as measured: `seedhammer` at `7b6f2fb`, `mnemonic-engrave` at
`874317ae`, `crates/me-cli` in this repo. Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go`. One probe test was added to the
fork's `gui` package, run, and deleted; `git status --porcelain` in the fork
is clean.

The already-settled list (wire version 8, §2's recipe, §5's no-slot ruling,
§3f's split, §4a's per-surface ruling, §3e's Go binding, the device-derivation
gap, the scope exclusions) was not revisited.

All three Importants are **bookkeeping and hook-placement defects introduced or
carried by this fold**, not defects in the design r3 established. None reaches
Critical: no wrong wallet is engraved (the engrave path uses the artifacts
recomputed at `gui/composer_flow.go:146`, not the ones built at `:98`), and the
two table defects are caught before any code exists.

---

## Q1 — did the fold address each finding?

### new-design report (`f449-spec-r3-new-design.md`, 0C/6I/4M)

| id | verdict | note |
| --- | --- | --- |
| I-1 DEFAULT ROW | **ADDRESSED** | §0b rewritten to seed `Initial` once per entry from the kind in force. Matches the shipped precedent exactly (`gui/composer_shape.go:353-373`; `ChoiceScreen.Initial` at `gui/gui.go:1928`, `seeded` at `:1941`, applied `:1957-1963`). **But §8.9's gate row still encodes the retired rule — see I-A.** |
| I-2 RESET granularity | **ADDRESSED** (substance) | The edit-detector requirement is gone; no `composerShapeSignature` hook is needed. Two consequences: the hook point named is wrong (**I-C**) and §8.9's gate row is stale (**I-A**). |
| I-3 §6a owned/gated | **ADDRESSED** | §8.9 gives all three §6a rows a gate and an owning stage (3, 2, 1b). §6a row 1 now also names the `md`/`gui` dependency. Residue: the stage table's *content* columns still do not name rows 2 and 3, so §8.9's stage column is the only ownership statement (**M-c**, **M-d**). |
| I-4 `[patch.crates-io]` override | **ADDRESSED** | §9a item 1 states the override, the `Cargo.lock` re-resolve and the failure mode ("a wall that reads like a broken workspace"). |
| I-5 §3f breaks `me`'s source | **ADDRESSED** | §9a item 2 states it and names the semantic choice. §3f's own blast-radius list is still `me`-free, but §9a says so in as many words, so an implementer is not misled. |
| I-6 stage 4a gate = status quo | **ADDRESSED** | Stage 4a's gate rewritten to "§8.9's `me` rows", explicitly retiring the round-trip disjunct and citing the measurement that made it vacuous. Verified: `me-cli/src/lib.rs:75-83` → `validate.rs:95-100` is a canonical-form check plus `md_codec::codex32::unwrap_string`, version-agnostic. |
| M-1 "too damaged" not in source | **ADDRESSED** in §6a row 2 ("exit 2 is the atomic-fail code"). The retracted phrase survives one row away in §8.9 (**M-d**). |
| M-2 window + ordering | **PARTIAL** | Window corrected to `96-128` and verified exactly (`:95` closes the `if !composerShapeFlow` block, `:96` `composerSizeAssignments`, `:98` `composerTemplateChunksFor`, `:128` `composerStubDelta`, `:129` `composerStubFlow`). The *ordering* half was **inverted rather than answered** — M-2 concluded the screen must sit **after** `:98` because that is where its inputs come from; r4 rules it must sit **before**, and does not say how the predicate gets a `PolicyShape` (**M-a**). |
| M-3 package boundary | **ADDRESSED** | §6a row 1 names `errWireVersion` at `md/md.go:22` — verified unexported (`var ( errWireVersion = errors.New(...)` inside an unexported block). See **M-c** on the row's closing clause. |
| M-4 probe attributed one function too high | **ADDRESSED** | Now names both: `complexAddressSource` at `gui/policy_address.go:88` (verified: `return complexAddressDeriver(collected, keys)`) and the probe at `:188-190` (verified: `if _, err := src(0, false); err != nil { return nil, false }`). |

### fold-check report (`f449-spec-r3-fold-check.md`, 4 partials)

| id | verdict | note |
| --- | --- | --- |
| I2 (`§8j`'s edit path carries it) | **ADDRESSED BY SUPERSESSION** | r4's predicate reset removes every edit-path hook, so there is nothing for §8j to carry. Verified the spec contains no `§8j` reference (the sole `8j` hit is the checksum `#8jc8gq6v` at line 652) — correctly so under the new design. The defect I2 reported (a kind surviving into a shape with no screen to unset it) is closed by the reachability argument, not by the remedy I2 prescribed. |
| I4 (`md repair` vector) | **ADDRESSED** | §8.9 row, owning stage 2. |
| I5 (`gatherIgnored` gated) | **ADDRESSED** | §8.9 row, owning stage 3. |
| I6 (`bundle.rs:371` fail-open) | **ADDRESSED** | New §8b + §8.9 row, stage 4a. **Independently verified accurate and correctly scoped:** `parse_line` returns `Parsed::Md1Single` *without reading a header* when the chunked flag is clear (`crates/me-cli/src/bundle.rs:196-198`), which is exactly the version-8 single-payload case (§3c: v8 → bit 0 = 0), so it reaches `:371`'s `if let Ok(d) = …` and fails open. The chunked arm already says the right thing — `:210-211` raises `BundleError::Md1WireVersion`, displayed at `:96` as "unsupported md1 wire version". §8b claims no more than that. |

### New file:line citations — 10 resolved, 10 accurate

| citation | claim | verified |
| --- | --- | --- |
| `gui/composer_flow.go:95` | `composerShapeFlow` closes | `:95` is the closing `}` of the `if !composerShapeFlow(...)` block. Loose phrasing (the *function* is `composer_shape.go:636`), but the derived window is exact |
| `gui/composer_flow.go:129` | first `composerStubFlow` | Exact — `forward, drew := composerStubFlow(ctx, th, template, nil, change)` |
| window `96-128` | the placement window | Exact — `:96` `composerSizeAssignments`, `:128` `composerStubDelta` |
| `md/md.go:22` | `errWireVersion` unexported | Exact — `errWireVersion = errors.New("md: wire version mismatch")` inside `var (…)`, lowercase |
| `gui/policy_address.go:88` | `complexAddressSource`'s entry | Exact — the tail of `complexAddressSource`; doc at `:93-94` confirms "Every screen goes through complexAddressSource" |
| `gui/policy_address.go:188-190` | the `src(0,false)` probe | Exact, inside `complexAddressDeriver` |
| `crates/me-cli/src/bundle.rs:371` | `if let Ok(d) = md_codec::decode::decode_md1_string(s) {` | Byte-exact, quoted block matches `:371-376` including the `plates.push` |
| `crates/me-cli/src/lib.rs:75-83` | `convert`'s body | Byte-exact |
| `crates/me-cli/src/validate.rs:95-100` | canonical check + `codex32::unwrap_string` | Exact |
| `crates/me-cli/src/sysw/record.rs:251-252` | `reassemble`/`decode_md1_string` reduced to `.is_ok()` | Exact |

`crates/me-cli/Cargo.toml:26` (`md-codec = "0.42"`) was re-cited in §8b; still exact.

---

## I-A (Important) — §8.9's §0b gate rows state the rule r4 just retired, and two of them cannot fail on the defect they guard

r3b wrote §8.9's four §0b rows against the **r3** text. r4 then rewrote RESET
and DEFAULT ROW and did not propagate. The spec's only gates for those two
rulings now contradict them.

**RESET.** §8.9: *"set kind 1, Back-edit the shape, and assert the kind is 0
again"*. §0b r4: the kind is 0 only if a conjunct is false. Measured — a
class-2-skipped copy of `composerLianaOutsideModelClass`
(`gui/composer_consent.go:381-468`) over `kofn-recovery`'s shape under `tr`:

```
baseline older(26280)        conj1=true  skip2=""                 FIRE(kind kept)=true
edit: older(52560)           conj1=true  skip2=""                 FIRE(kind kept)=true
edit: primary 2of4           conj1=true  skip2=""                 FIRE(kind kept)=true
edit: after(1000000)         conj1=true  skip2="an absolute lock"  FIRE(kind kept)=false
```

Two of three Back-edits leave both conjuncts true, so a correct r4
implementation keeps kind 1 — and **fails this gate**. An implementer who makes
the gate pass has re-implemented r3's blanket reset, which r4 retired for
breaking the Back invariant. The gate does not merely under-test; it pulls the
implementation back to the rejected design.

**DEFAULT ROW.** §8.9: *"the widget opens on the NUMS row, asserted on the
FIRST page"*. Under r4 that is true only on first entry. A test that enters the
screen fresh and asserts row 0 passes **identically** against the r4 rule and
against the always-NUMS defect I-1 reported — it cannot fail on the thing it
guards. The re-entry case, which is the whole of I-1, is untested.

**PLACEMENT** is weaker than its ruling too: §8.9 asserts only "reached before
the first `composerStubFlow`", which a screen sitting anywhere in `:99-:128`
satisfies while violating §0b's actual rule (before `composerTemplateChunksFor`
at `:98`).

Three of §0b's five sub-rulings therefore have a gate that is wrong, toothless,
or both — in the section r3b added under the banner *"a rule with no gate is not
a rule"*.

## I-B (Important) — §8's renumbering broke stage 1b's gate and orphaned mutation testing

r3b inserted a new §8 item 9 ("The operator-facing rulings get vectors too")
and pushed mutation testing to item 10. Two references still mean the old
numbering:

1. **Stage 1b's gate** reads *"§8 vectors **1, 2, 5, 6, 7, 9** — every leg
   runnable in Rust alone"*. Under the new numbering "9" is the operator-facing
   table, whose rows are owned by stages **1b, 2, 3, 4 and 4a** — device and Go
   legs stage 1b cannot run. That is opus M2's own prior finding ("stage 1b
   claiming a gate whose device and Go legs it cannot run") reintroduced
   verbatim. And mutation testing — the item that proves every other property
   can fail — is now named by **no stage gate at all**.
2. **The owning-stage table** still reads *"1 recipe vectors, 6 structure pin,
   7 fixpoint, **9 mutation** | 1b"*, mislabelling item 9 and leaving item 10
   in no row.

The bolded sentence immediately above that table — **"Every §8 item has an
owning stage, and no item is left unowned"** — is therefore false as of this
fold, about the very item whose job is to make the other gates real. Every
other `§8.N` reference in the file (2, 3, 4, 5, 8) is ≤ 8 and unaffected, so
the break is exactly these two.

## I-C (Important) — RESET's hook sits after the template is built, so the copy-to-steel screen can show a kind-1 id for a kind-0 wallet

§0b: *"On every entry to `composerStubFlow`, recompute §0b's two conjuncts…"*.
Measured order inside `composerFlow`'s loop:

```
:85  composerShapeFlow            <- shape edits land here
     [§0b choice screen, per PLACEMENT: after :95, before :98]
:96  composerSizeAssignments
:98  template, err := composerTemplateChunksFor(st)   <- kind -> chunks
:128 change, advertised := composerStubDelta(shown, seen, template)
:129 forward, drew := composerStubFlow(ctx, th, template, nil, change)
     [§0b RESET, per the ruling]
```

The reset runs **after** the template it invalidates. Reproduction:

1. `tr` / `kofn-recovery`. Predicate true, operator picks "Liana xpub", kind 1.
   `:98` builds a kind-1 template; `:129` shows the kind-1 Template-ID and mk1
   stub. Operator copies it and mints cosigner cards.
2. Back (`forward == false` → `continue`).
3. Edits the recovery lock to `after(1000000)`, Done. The choice screen does
   not fire (conjunct 2 is now `"an absolute lock"` — measured above), **and
   the kind is still 1**.
4. `:98` builds a **kind-1** template from the stale kind. `:129` displays its
   Template-ID and mk1 stub.
5. Only now does the reset fire and set kind 0.

So the screen whose own doc comment says *"A false statement here is worse than
a missing one: this is the screen whose whole job is to be copied onto steel"*
(`gui/composer_flow.go:123-127`) displays artifacts for a wallet the state no
longer holds, and cards minted from that stub fail seating later.

Not Critical: `template` is reassigned at `:146` by `composerArtifactsFor`, so
`composerEngraveStep` at `:163` cuts the post-reset kind-0 wallet. The damage is
a false id on the stub screen and cards minted from it — loud at seating, but
after steel.

The fix is one clause: evaluate the predicate **once**, at the point PLACEMENT
already names — before `composerTemplateChunksFor` — and let it both gate the
screen and zero the kind. That is also what makes §0b's own reachability
argument ("the predicate that shows the screen is the same predicate that keeps
the value") literally true rather than approximately true.

---

## Minors

- **M-a (§0b PLACEMENT).** The ruling says the screen must come *before*
  `composerTemplateChunksFor`, but conjunct 2 needs
  `composerLianaOutsideModelClass(root, shape)`, and the only producer of a
  `md.PolicyShape` in the fork is `md.PolicyShapeChunks(strs []string)`
  (`md/policy_shape.go:107`) — chunks required. `composerTemplateChunksFor` is
  pure (`md.ComposeWith(st.list, …).Chunks()`, `gui/composer_flow.go:267-273`),
  so the screen can call it itself; the spec never says it must. r3's M-2 named
  this and r4 inverted the conclusion instead of answering it.
- **M-b (§0b RESET rationale).** *"what is dropped was unrepresentable anyway"*
  is true of conjunct 1 (a real internal key has no seat in §3f's sum type) and
  **false of conjunct 2**: kind 1 over a `decaying-multisig` tree is encodable
  (§3d), renderable (§4) and not refused by §6 — conjunct 2 is a Liana-model
  *usefulness* test, not a representability test. The behaviour is still right;
  the stated reason is what a port implementer will reason from. Relatedly, the
  drop is silent, with `composerStubDelta`'s cause-free banner as the only
  signal — the same signal `composerKeyOrderStep`'s comment judges insufficient.
- **M-c (§6a row 1).** *"a `md/` change, hence stage 3, not a `gui/` one"* is
  inaccurate: only the sentinel is `md/`. `gatherIgnored` is declared at
  `gui/mk1_inspect.go:36`, returned at `gui/md1_gather.go:33,39` and printed at
  `:121`. The scheduling outcome is unaffected (stage 3's content names the
  split explicitly), but the sentence tells an implementer no `gui/` work exists
  at that stage.
- **M-d (§8.9).** The phrase **"too damaged"**, retracted from §6a row 2 by this
  same fold (r3 M-1: the string is nowhere in `descriptor-mnemonic`), survives
  in §8.9's `md repair` gate row. Superseded phrasing one row from its own
  correction.
- **M-e (§9a / stage 4a).** §9a's heading *"Stage 4a is three pieces of work,
  not one"* undercounts: §8b assigns a fourth (`bundle.rs:371`) to stage 4a.
  r4 replaced stage 4a's content cell with "see §9a", so the content column no
  longer names the `bundle.rs` fix that r3b had just put there; only the gate
  column reaches it, via §8.9.

---

## Q2 — are the two new rulings sound?

### (a) RESET as a predicate — the idea is sound and strictly better than r3's; the hook point is wrong

**Is `composerStubFlow` entered on every path that can change the shape? Yes.**
`composerFlow`'s loop has exactly two shape mutators, both upstream of `:98`:
`composerShapeFlow` at `:85` (path list, and the "Change the script" row) and
`composerStartStep` at `:92` on the Back leg (wrapper + preset, via
`composerApplyShapeEdit`). Both reach the loop top by `continue`, and the loop
then unconditionally runs `:98` and `:129`.

**Is there a path to engraving that skips it? No.** `composerEngraveStep`
(`:163`) is reachable only through `:129` → `forward` → `:143`
`composerSeatingStep` → `:146` → `:160` `composerConsentFlow`. The *second*
`composerStubFlow` (`:152`) is conditional on `len(keyed) > 0`; the first is
not. (Ambiguity worth one word in the spec: "every entry to `composerStubFlow`"
names two call sites. Hooking inside the function runs the reset twice per pass,
harmlessly, since nothing between them changes the shape.)

**Does it preserve the Back invariant? Yes — that is its real win.** A pure
navigational Back with no edit leaves both conjuncts unchanged, so the kind
survives. That is precisely what r3's "any re-entry resets" lost and what
I-1/I-2 were about, and it needs no change detector — which matters, because
the one detector that exists (`composerShapeSignature`,
`gui/composer_discard.go:45-65`) was measured blind to the edit that flips the
predicate.

**Does it silently drop a still-valid choice?** It drops on conjunct-2 failure,
where the choice is still *representable* but no longer *useful* — see M-b. The
drop is defensible (the shape is one §0a calls a strict downgrade, and the
screen that could unset it no longer fires), but the spec's justification
overstates and the drop is unannounced.

**The defect is the hook point** (I-C): naming `composerStubFlow` puts the
re-evaluation on the far side of `composerTemplateChunksFor`, the one call
§0b's own PLACEMENT paragraph identifies as the point where the kind becomes
the chunks the operator copies. Move it to PLACEMENT's point and everything in
§0b — predicate, placement, reset, reachability — collapses into a single
evaluation, which is what the section is reaching for.

### (b) DEFAULT ROW seeded once per entry from current state — sound and directly implementable

`ChoiceScreen` already carries the field: `Initial int` (`gui/gui.go:1928`),
applied in `Choose` at `:1957-1963` under a `seeded bool` (`:1941`) whose doc
says *"applied ONCE per screen value and not on every Choose"*. The shipped
precedent is one function long and is the closest analogue there is —
`composerKeyOrderStep` (`gui/composer_shape.go:353-373`): compute `initial`
from the setting in force, construct a fresh `&ChoiceScreen{…, Initial:
initial}`, `Choose`. §0b's screen is the same two-row shape at the same point in
the flow. Implementable as written.

**It avoids the row-zero defect.** `Initial` = the kind in force opens on NUMS
on the first pass for free (the kind's zero value is 0 = NUMS), so the
zero-value trap §0b worries about never arises, and the re-entry case — the one
that produced I-1's six-step reproduction — is correct too. The ruling's two
clauses ("NUMS on first entry, current kind on re-entry") are one clause
written twice; harmless, and the second is the load-bearing half.

**One nuance the wording already forecloses, worth keeping.** If an implementer
retains a single `ChoiceScreen` across loop passes instead of constructing a
fresh one (the `gui/freetext_flow.go:540` / `passphrase_flow.go:406` pattern),
`Initial` applies only on the first `Choose`, and a retained `choice == 1` would
survive a reset that set the kind to 0 — the screen would propose Liana again
for a wallet that is kind 0. §0b's *"Seed the initial **once per screen entry**
from current state, never from a constant"* rules that out. Do not relax it to
the widget's own "once per screen value" phrasing.

Sound. The only defect attached to DEFAULT ROW is its gate (I-A).

---

**ready for implementation: no** — three Importants block: §8.9's §0b gate rows
contradict the rulings they gate (I-A), §8's renumbering left stage 1b's gate
wrong and mutation testing unowned while the text claims otherwise (I-B), and
RESET's hook sits one call too late, so the copy-to-steel stub screen can
display a kind-1 identity for a wallet the reset has just made kind 0 (I-C).
All three are localized edits — two table repairs and one clause moved from
`composerStubFlow` to the point PLACEMENT already names — and none disturbs the
design r3 settled.
