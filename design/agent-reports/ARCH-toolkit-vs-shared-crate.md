# ARCH ruling — toolkit vs. new shared crate for the IO + safety layer (D5)

**Date:** 2026-08-26. **Question, in the operator's words:** *"I forgot about the
mnemonic-toolkit. It's possible we should be putting functions in the toolkit
instead of the m\* binaries."* **Scope:** D5 of
`design/SPEC_constellation_cli_uniformity.md` only. D1–D4 and D6 are treated as
constraints. Every load-bearing claim below was measured against the repos or
the crates.io API during this ruling; the commands are in §7.

**Mid-flight operator direction, folded before this report was finalised:** the
target architecture is three tiers — `md`/`mk`/`ms`/`mt` dedicated to m\*1
string encoding; `mnemonic-toolkit` for fancy processing (BIP-85, SLIP-39,
Electrum crypto, …); `mnemonic-engrave` for payload prep and communication —
with *"clean, organized, and relatively symmetric"* as a stated design value,
and `mnemonic-gui` explicitly out of scope. The first draft of this ruling was
re-tested against that model before committing; §1a records the test.

---

## 1. The ruling

**D5 stands: the IO + safety layer lives in a new, small, dedicated crate — and
`mnemonic-toolkit` becomes that crate's fifth consumer, not its home.** Under
the operator's three-tier architecture the layer fits **no tier** — it is not
encoding, not fancy processing, not payload prep — and the clean answer is a
**fourth, foundation-tier thing**: a dependency-light crate that all three
tiers consume, which is exactly what D5 already specifies. Toolkit-as-home
fails the tier model outright — every *encoding* binary would depend on the
*fancy-processing* tier, inverting the stated order and dragging SLIP-39,
Electrum crypto and address derivation into tools that encode a string — and it
independently fails five verified facts: (1) it is **not on
crates.io** — the API returns *"crate `mnemonic-toolkit` does not exist"*
despite the manifest's docs.rs metadata — and it **cannot publish as it
stands**, because it depends on `wc-codec` by bare `path` with no `version` key
(also unpublished), which `cargo publish` refuses; (2) `md-cli`, `mk-cli` and
`ms-cli` **are** published crates (0.13.0 / 0.12.1 / 0.14.0, repository fields
pointing at bg002h), and a published crate cannot carry a git or path
dependency, so whatever they share **must itself be a published crates.io
crate** — the one distribution mechanism the toolkit does not offer; (3) the
toolkit's dependency closure (miniscript, bitcoin, aes/cbc/ctr, crypto-bigint,
bip38, bip39, bip322, pbkdf2, sha3, regex, flate2, …) would be dragged into
`mt-cli`, whose entire dependency list today is `mt-codec` + `bitcoin` + `clap`
— a bearer-material binary kept deliberately minimal; (4) the toolkit only
builds correctly under its workspace `[patch.crates-io]` pin of miniscript to
git rev `ff4732e`, and **a patch section does not propagate to dependents**, so
five consumers would silently build the toolkit's code against an unpatched
miniscript — a shared safety layer that behaves differently in situ than as a
dependency is disqualified on that alone; and (5) the coupling direction is
backwards: the code D5 extracts does not exist in the toolkit. The toolkit's
argv and permission machinery is **advisory-grade** — `secret_in_argv_warning`
(`src/secret_advisory.rs:40`) is a stderr warning with no refusal and no
override, and `warn_if_world_readable` is likewise a warning where §6e specifies
a refusal — while the refusal-grade reference implementations live in `me-cli`
(`write_private` at main.rs:856, `Class::is_argv_forbidden`,
`--allow-argv-secret`) and `mt-cli` (the pre-clap raw-argv guard). The toolkit
is one of the five things being brought INTO conformance; it cannot also be the
source of the rule. **What the operator's instinct gets right is absorbed as two
subsidiary rulings:** (a) the constellation's one genuinely canonical shared
layer — display-grouping — is already kept in lockstep by four deliberate copies
pinned to a checksum-gated vectors file (verified: identical sha256
`7147b0ec…` in all four repos), and the new crate must **not** absorb or
re-implement it; and (b) `mnemonic`'s argv phase is **conform, not invent** —
P3 replaces its warning call sites with the shared refusal, keyed on the
taxonomy predicate it already tests (`is_argv_secret_bearing` /
`SECRET_NODE_TYPES_ARGV`), not on a hand-enumerated channel list.

The new crate is **foundation-tier by dependency structure regardless of which
repo hosts it** — a Cargo edge binds consumers to the crate, never to its host
workspace's other members — so the host choice is repo-level only, and it is
**reversible**: once the crate is on crates.io, relocating its repo later costs
consumers nothing (the crates.io identity is stable; the repository field
updates on the next publish). The only irreversible choice is the crates.io
name. On that basis the ruling hosts it, **this cycle**, as a **workspace
member of `mnemonic-engrave`** (beside `me-cli`), **published to crates.io at
0.1.0 when P0 closes GREEN**, and consumed by version everywhere. Rationale for
the host: the extraction source (`me`'s write gate and argv guard) is in this
repo, so P0 is an in-repo refactor plus one new crate rather than a three-repo
migration; this repo already hosts the constellation's cross-cutting spec,
FOLLOWUPS and journeys; and it gives the Rust-primary question a single
unambiguous answer — the shared crate's host repo is primary for the IO +
safety layer, and ports mirror downstream from it, per the standing rule.
**Flagged against the symmetry criterion, deliberately:** this makes
`mnemonic-engrave` special — the payload-prep repo also housing the
constellation-facing foundation crate. If the operator wants the fourth tier to
be a fourth *repo*, the move is the cheap, non-breaking follow-up named in §6;
it is preference-shaped, not measurement-forced, and nothing in this cycle
depends on it. Runner-up considered and declined: a new member crate in the
toolkit workspace would publish just as well, but it moves `me`'s code across
repos for no gain, couples the layer to the constellation's heaviest-churn repo
(0.97.0 — ninety-seven minor versions), keeps the tier smell the operator just
named, and the precedent there (`wc-codec`) is a member crate that never got
published and now blocks its host's publishability.

## 1a. The tier-model re-test, run before committing

Per the mid-flight direction, the draft ruling was re-tested against the
three-tier architecture rather than assumed compatible. Results:

- **Toolkit-as-home fails the tier model independently of §1's five facts** —
  encoders depending on fancy processing is the inversion the operator named.
  The two failures are separate: fixing the toolkit's publishability would not
  fix the tier inversion.
- **"Which tier does the IO + safety layer belong to?" — none.** Refusing argv
  secrets, 0600 writes and exit-code discipline are properties every tier must
  have. A layer needed by all three tiers and owned by none is a foundation
  crate by definition; giving it to any tier's *flagship crate* privileges that
  tier. D5's small crate **is** the fourth thing; this ruling only pins its
  weight (near-zero deps), its distribution (published), and its host.
- **The clean-seam question (direction item 1): tested, and the seam is real
  but the extraction is not this cycle's.** `display_grouping` is genuinely
  pure and dependency-free (132 lines, no imports beyond core) and would lift
  cleanly into the foundation crate — but its three siblings live in *published*
  crates' trees (`md-codec/src/encode.rs:141-164` — a codec crate),
  so collapsing the four copies means a `md-codec` release plus three CLI
  releases for **zero behaviour change**, mid-cycle, while the checksum-gated
  vectors file already holds them in provable lockstep (identical sha256 in all
  four repos). `secret_taxonomy` is the opposite case: it is entangled with the
  toolkit's `NodeType`/`SlotSubkey` universe (wif, bip38, electrum-phrase,
  minikey) — that is per-tool domain knowledge and *should not* move.
  `secret_advisory` is warning-grade and is superseded, not extracted.
- **A symmetry defect the first draft under-flagged, now flagged (direction
  item 3):** `md` alone keeps its grouping copy in its **codec** crate, where
  `mk` and `ms` keep theirs bin-private and the toolkit keeps its own in its
  lib. Presentation code in a codec tier is a tier smell in exactly the
  operator's sense. The eventual collapse of display-grouping into the
  foundation crate fixes it; filed as the ownerless follow-up in §6, not
  smuggled into this cycle.
- **`mnemonic-gui`:** nothing in this ruling adds GUI work beyond what P3
  already carries ("the GUI mirror" regeneration). Noted and set aside per the
  direction.

## 2. What moves, what stays, what is deleted

**Moves into the new crate (extraction, not re-derivation — spec P0's own
words):**

- `write_private` (0600 create-and-write) — from `me-cli/src/main.rs:856`
  (currently binary-private, no exported API; the spec already measured this).
- The pre-parser argv guard **mechanics**: the raw-`std::env::args()` scan shape
  of `mt-cli/src/main.rs` (`validate::command_line_guard`, the "§8.2f RUNS
  BEFORE CLAP" block) generalised to take a per-tool material class, with
  `--allow-argv-secret` parsed on raw argv (§6d's normative ordering), and the
  never-echo / class-and-length refusal wording.
- The write gate: world-readable refusal naming the measured mode + the
  own-mode-only caveat (F-252 wording), `--allow-world-readable`,
  input-refusal-outranks-destination ordering, no-secret-line-before-gates
  (F-246, narrow form).
- §6f exit-code mapping and §6h remedy-text builders.
- `--in FILE` / `-` / `--out FILE` channel helpers.

**Stays where it is:**

- **Display-grouping, all four copies:** `mnemonic-toolkit/src/display_grouping.rs`,
  `mk-cli/src/format.rs`, `ms-cli/src/format.rs`, `md-codec/src/encode.rs:141-164`
  — byte-identical semantics pinned to `design/display-grouping-vectors.tsv`
  (same sha256 in all four repos, checksum-gated in CI). D4/§6c are per-CLI
  *wiring* changes (which stream, which flags survive) using these existing
  functions. The new crate re-implementing grouping would create the fifth copy
  §5 warns about. The foundation crate is the natural **future** home for this
  layer — that collapse also cures the `md`-is-special asymmetry flagged in
  §1a — but it costs a `md-codec` release for zero behaviour change and is
  deferred (§6, question 5).
- `me`'s terminal gate — C-3 scoped it to the binary container; it stays
  `me`-private and the shared crate must **not** export it, or someone will
  wire it into `ms`.
- `me sysw pack --expect` (§6g) — `me-cli`, not the crate. P0 builds both, in
  different places.
- Per-tool material **classification**: `me`'s `Class::is_argv_forbidden` (over
  `me`'s record classes) and the toolkit's `secret_taxonomy` +
  `NodeType::is_argv_secret_bearing` with their parity tests. The shared crate
  takes "this argument is refusable material" as an input; deciding it stays
  where the domain knowledge is.
- The toolkit's `mlock`, `process_hardening`, `secrets`, `secret_string`,
  `seedqr`, and the rest of its lib surface — none of it is the IO layer.

**Deleted, as consumers adopt:**

- `mt-cli`'s local guard plumbing where the crate supersedes it (P1) — the
  behaviour is already correct; the diff is a re-home plus the two §6b/§6d
  additions the spec already schedules.
- `me-cli`'s in-binary copies of the extracted functions (P0 — same commit as
  the extraction).
- The toolkit's `secret_in_argv_warning` **call sites at secret channels**,
  replaced by the crate's refusal (P3). Measured: 66 references across 21
  files, which matches the review's 48-sites-across-20-files order of magnitude
  and is roughly **ten times** the spec's five named channels — see open
  question 3. The helper itself dies when its last call site does.

## 3. The distribution mechanism, decided

**A published crates.io crate, version-depended by all five CLIs; first publish
at the end of P0, operator-gated.** The evidence that this is the only
mechanism that works here:

- `md-cli` 0.13.0, `mk-cli` 0.12.1, `ms-cli` 0.14.0 are live on crates.io with
  bg002h repository fields; local trees are at 0.13.0 / 0.13.0 / 0.16.0 — so
  publishing lags but is alive, and **cargo refuses to publish a crate whose
  dependency is git- or path-only**. A git-rev pin (the `mt-codec` precedent
  the spec discusses) is therefore unavailable to three of the five consumers.
  The spec's §7 P0 analysis — "the rev pin exists precisely to avoid a release
  step, and D5 needs one" — is confirmed from this second, independent
  direction.
- The publish is the cycle's first irreversible action (the name is claimed
  forever), so it sits **after** P0's R0 round closes GREEN, as the standing
  workflow already requires. Until the publish, `me-cli` consumes the crate
  in-workspace by `path + version` (the same form `mt-cli` uses for
  `mt-codec`), which keeps P0 fully testable before anything is public.
- Naming: `me` and `mnemonic` on crates.io are **foreign crates** (0.1.0 and
  1.1.1, unrelated), so short names are already being taken. Candidates
  verified free today: `m-cli-io`, `mcli-io`, `m-cli-safety`, `mstring-io`.
  The name is P0's first decision and needs the operator; availability decays.
- Keep the crate's own dependencies at **zero, or clap alone** — the guard's
  normative pre-clap ordering means the core cannot require clap; if the
  channel helpers want clap integration, feature-gate it. A safety crate five
  repos must track earns trust by having nothing in it.

## 4. What this does to §7's phases

- **P0 — unchanged in content, and its one open blank is now filled**: the
  distribution mechanism is "new workspace member of `mnemonic-engrave`,
  published to crates.io at 0.1.0 on P0 GREEN, operator-gated". Add one
  sentence of negative scope: the crate does not absorb display-grouping and
  does not export a terminal gate.
- **P1, P2 — unaffected** beyond consuming the crate by version instead of an
  unspecified mechanism.
- **P3 — relabelled for `mnemonic`: conform, not implement.** The refusal keys
  on the toolkit's existing `is_argv_secret_bearing` predicate (parity-tested)
  rather than a hand-enumerated five-channel list; the five named channels
  become the gate's *named assertions*, not the sweep's boundary.
- **P4 — unaffected.**
- **No phase disappears; no new phase is created.**

## 5. What it costs

- **One irreversible crates.io publish this cycle**, in a constellation that
  has deferred publishing exactly once (`mt-codec`) specifically because it is
  irreversible. Mitigated, not avoided: gated behind P0's GREEN and the
  operator's hand.
- **Propagation tax on every change**: a refusal-wording tweak becomes a
  version bump plus five manifest/lock bumps. This is inherent to D5 in any
  home — the constellation already pays it for `md-codec`/`mk-codec`/`ms-codec`
  bumps — but it is new for this layer, and mixed refusal-text versions across
  tools will exist between bumps (the spec already accepts mixed states).
- **`mnemonic-engrave` becomes load-bearing for four other repos' builds.** A
  bad publish blocks everyone; yank-and-patch is the recovery.
- **A mild layering inversion for the toolkit**: engrave becomes upstream of
  the toolkit for this one layer. Accepted deliberately — it is the direction
  the code maturity actually runs, and the alternative (toolkit upstream of the
  safety posture it currently fails) is worse.
- **A symmetry debt, named rather than hidden**: hosting the foundation crate
  in `mnemonic-engrave` makes that repo special under the operator's tier
  model, and the four display-grouping copies (one of them in a codec crate)
  remain asymmetric this cycle. Both are flagged in §1a with their reversal
  paths in §6; neither blocks the cycle's value.
- **What it does NOT cost**: no delay to P1/P2 relative to the spec as written
  — the crate was already P0; this ruling only pins where it lives and how it
  ships.

## 6. Open questions

1. **The crate's name.** Needs the operator; verified free today (§3) but
   availability decays. Settled by: operator picks, P0 claims it at publish.
2. **Do `md-cli`/`mk-cli`/`ms-cli` intend to keep publishing?** Local versions
   are ahead of crates.io (0.13.0 vs 0.12.1, 0.16.0 vs 0.14.0), so lag is real.
   If the operator has quietly abandoned crates.io for the CLIs, a git-rev-pin
   scheme becomes viable and the publish could be deferred like `mt-codec`'s.
   The ruling assumes publishing continues because the crates are live and
   owned. Settled by: one question to the operator.
3. **The width of `mnemonic`'s refusal surface.** The spec names five secret
   channels; the toolkit's own warning fires from 21 files, driven by
   `SECRET_NODE_TYPES_ARGV` (phrase, entropy, xprv, wif, ms1, bip38,
   electrum-phrase, seedqr, minikey). Whether all of those sites are *argv
   values of typed flags* (refusable under D3) or include stdin-adjacent forms
   was not resolved here. Settled by: a per-site sweep during P3 planning,
   keyed on the predicate.
4. **Whether any toolkit conformance test must move.** The display-grouping
   conformance tests stay with their copies; but if P0's crate grows shared
   refusal-text goldens, the cross-repo lockstep mechanism (vectors + sha256,
   proven for grouping) should be reused rather than invented again. Settled
   by: P0's test plan.
5. **Whether the fourth tier gets a fourth repo.** This ruling hosts the
   foundation crate in `mnemonic-engrave` for extraction locality and flags
   the resulting asymmetry (§1a). Relocating the crate to its own repo later
   is non-breaking once it is published. Settled by: the operator's preference,
   any time after P0's publish, at the cost of one repo move and one
   repository-field update.
6. **The display-grouping collapse** (four copies → the foundation crate,
   including lifting the copy out of `md-codec`, where presentation code sits
   in a codec crate). Behaviour-neutral, held in lockstep today by the
   checksum-gated vectors, so it earns no place in this cycle. Settled by: an
   ownerless follow-up filed for a later cycle, per the test-infra-is-polish
   rule.

## 7. Facts verified during this ruling (commands, run 2026-08-26)

| claim | how verified |
| --- | --- |
| toolkit not on crates.io; wc-codec not on crates.io | `curl https://crates.io/api/v1/crates/{mnemonic-toolkit,wc-codec}` → `does not exist` |
| toolkit cannot publish as-is | `crates/mnemonic-toolkit/Cargo.toml`: `wc-codec = { path = "../wc-codec" }`, no `version` key |
| toolkit needs a non-propagating patch | root `Cargo.toml:34-35`: `[patch.crates-io] miniscript = { git = …, rev = "ff4732e…" }` |
| md/mk/ms CLIs are published and owned | crates.io API: md-cli 0.13.0 → `github.com/bg002h/descriptor-mnemonic`; ms-cli 0.14.0 → `bg002h/mnemonic-secret`; mk-cli 0.12.1 |
| `me`/`mnemonic` names are foreign on crates.io | crates.io API: `me` 0.1.0, `mnemonic` 1.1.1 exist, unrelated |
| no m\* CLI depends on the toolkit today | grep of every `crates/*/Cargo.toml` in all five repos: zero dependency edges; descriptor-mnemonic's sole mention is a comment about the miniscript patch |
| toolkit argv machinery is warning-grade | `src/secret_advisory.rs:40` `secret_in_argv_warning` — warning, no refusal, no override; `warn_if_world_readable` likewise |
| refusal-grade code lives in me/mt | `me-cli/src/main.rs:856` `write_private`; `:1979` `class.is_argv_forbidden()`; `mt-cli/src/main.rs` pre-clap guard block |
| display-grouping is 4 lockstep copies | `fn render_grouped` in toolkit lib, mk-cli/format.rs, ms-cli/format.rs, md-codec/encode.rs:147; `sha256sum design/display-grouping-vectors.tsv` identical (`7147b0ec…`) in all 4 repos |
| toolkit warning surface ≫ five channels | 66 `secret_in_argv_warning` references across 21 files |
| mt-cli dependency weight | `mt-cli/Cargo.toml`: mt-codec, bitcoin, clap — nothing else |

— architect agent, dispatched by the controller for the D5 ruling.
