# S5 prerequisites — scope-and-sequencing ruling (2026-08-15)

**This is an agent's advisory ruling, standing in for the operator on a sequencing decision; it is not a human decision and binds nothing until the operator adopts it.**

Repo under ruling: `/scratch/code/shibboleth/seedhammer`, `main` @ `80d0c5d05acbeee1ac1aed6a43c549bfb0cbee6e` (verified). Plan: `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (S5 gate at lines 1277–1286). Every tree read for this ruling was left as found (`git status --porcelain` unchanged in seedhammer, mnemonic-secret, descriptor-mnemonic, mnemonic-key; mnemonic-secret's three untracked files pre-existed).

---

**S5 PROCEEDS AFTER: (1) the ms pin bump to ms-cli-v0.16.0 with the one-commit S0 re-anchor, then (2) the built-policy ExpectKind extension landing green as S5's own first, oracle-only work block — in that order, both inside S5's cycle, neither a new plan stage.**

---

## 1a. PREREQUISITE 1 — the ExpectKind for what S5 engraves (F-a)

**RULING: part of S5 — not a separate S0b-style stage — but as S5's first work block ("S5.0"), landing and closing green in `oracle/**` + `cmd/gaterecord/**` BEFORE any engrave-tail device code is written.**

**WHY**

- The ownership question was already ruled, and the ruling holds: `CONTINUITY_2026-08-15c.md:74-75` — *"The new `ExpectKind` for built policies is owned by the first stage that must **mint a record for a built policy**, not by S3."* S5 is that stage: S6's gate is external-coordinator restores plus an ms1 plate read-back on hardware (plan lines 1290–1311) and mints no emulator record. There is exactly one consumer left.
- S0b's precedent does not transfer. S0b existed because scaffolding five stages leaned on was owned by none of them; here the single-consumer condition that justified a separate gated stage is absent, and a separate stage would buy a review round without buying independence.
- The independence a stage boundary would otherwise protect is supplied by the architecture, not the boundary: every expected string comes OUT of the pinned primary binaries (`oracle/expect.go` DeriveExpected invokes `ms`/`mk` — and will invoke `md` — rather than reimplementing anything), `CompareCensus` and its three mutation proofs are already in place, kind-agnostic, and run untagged on every machine (`oracle/expect_test.go:260-328`).
- **But the order inside S5 is forced.** The instrument must exist and be seen to fail before the thing it judges exists: (a) S5's TDD needs the deriver to state expectations against; (b) an instrument authored after the tail invites fitting the instrument to the output. So S5.0 closes (suite green, one `oraclelive` execution recorded) before the first `gui/` edit of the tail.

**CONSEQUENCES**

Files S5.0 changes, concretely:

- `oracle/expect.go` — a new `ExpectKind` for a built policy. It must carry the **engrave mode** (full vs watch-only decides whether ms1s are expected; `InputTuple` has template/n/k/slot_order/fp_choice/origins/seeds and **no mode field** — either two kinds, e.g. `built-policy-full` / `built-policy-watch`, or a mode field on `Expect`). `ArtifactKindFor` must grow the "richer answer" its own comment reserves (`oracle/expect.go:92-94`: *"A kind that engraves several artifact kinds needs a richer answer than this; add it when a stage actually has that shape"* — S5 is that shape: md1 + mk1 + ms1 in one census). New derivation steps: `md encode` (which prints a `chunk-set-id: 0x…` header on stdout that `mkEncode`'s comment at `oracle/expect.go:405-410` already predicts must be refused/consumed deliberately, never adopted as an artifact) and `ms encode` (**`--group-size 0` is mandatory** — the pinned ms defaults to `--group-size 5`, verified via `~/.cargo/bin/ms encode --help`: "[default: 5]" — and `--no-engraving-card` for tooling; the same unbroken-form trap mkEncode documents). Artifact order must be the flow's engrave order (today ms1-first in full mode; S5 defers reordering) and must be mutation-checked, since `CompareCensus` is order-sensitive by design.
- `oracle/expect_test.go` — scope `TestCommittedFingerprintsAreRealAndDistinct` per the re-review's prescription (`s0b-fold-rereview-2026-08-15.md` M-3, lines 272-282): the every-artifact-has-a-fingerprint + ≥2-distinct assertion applies to expectations whose `inputs.expect.kind` is `cosigner-cards`; for the built-policy kind, assert fingerprints on its mk1/ms1 artifacts and none demanded of md1 chunks. Extend the kind-consistency checks in `TestEveryCommittedExpectationBelongsToARecord` (`:166-183`) to the multi-kind answer.
- `cmd/gaterecord` — no structural change expected; its refusal message already points at `oracle.ExpectKind` by design.
- **One-line plan fold**: S5's file-touch matrix (plan lines 1251-1266) lists `gui/**` + the walk script and **no `oracle/**` path**. Add the oracle rows with the F-a rationale so the matrix stays honest. This restates a ruling already made (F-a ownership), so per the proportional-re-review rule it needs the cite gate, not a fresh review round.

**Gate for S5.0** (named, as required): untagged `go test ./oracle/` green including the new kind's refusal paths (unknown kind still refuses; header-line adoption refuses; origin mismatch refuses; fingerprint scoping proven both directions) **and** one `./scripts/oracle-live.sh` execution green on the maintainer machine, exit code recorded. Stated plainly: the new kind's first **end-to-end** execution is S5's own mint — per the lens-closure doctrine S5 cannot close before that mint has RUN, and S5.0 does not pretend to discharge it.

**WHAT I VERIFIED**

- `oracle/expect.go:66-71` — exactly one kind, `KindCosignerCards`; `:95-102` ArtifactKindFor single-kind; `:194-199` the refusal text quoted in the brief.
- `oracle/expect_test.go:234-249` — the fingerprint test loops every record and errors on any artifact with `Fingerprint == ""`; an md1-chunk record fails at the first artifact. Confirms the stacked blocker.
- `oracle/record_test.go:359-375` — `TestS0GateHasARecord` demands only S0; additional stages' records are additive, no hidden stage-name constraint.
- Plan S6 (lines 1290-1311) — hardware gate, no record mint; S5 is the sole remaining consumer.
- `go test ./oracle/` at `80d0c5d`: `ok seedhammer.com/oracle 0.071s` (untagged baseline green).

## 1b. PREREQUISITE 2 — the ms pin (F-177)

**RULING: bump before S5's oracle-extension work begins — the first commit of the S5 cycle. Not deferred. md needs nothing; mk needs nothing for S5.**

**WHY**

- F-177's own schedule fires now: *"Do it when S2 extends the oracle, so the re-anchor happens once rather than twice"* (`FOLLOWUPS.md:6102`). S2 closed without extending the oracle — the extension slid to S5.0 — so the owning moment is the start of this cycle, and by the reconcile-first rule it is due before the phase proceeds.
- Bumping first means everything the S5 cycle mints (S5.0 fixtures, the S5 record and expectation) carries current provenance and is re-anchored **zero** times. Bumping after S5 would stale-out the S5 record the day it lands, because `VerifyRecord` (`oracle/record.go:356-370`) requires **every** pin to appear in **every** record at the pinned commit, untagged, on CI.
- The additive-change argument is true (see §3) but it is an argument living outside the machinery; the pin file exists precisely so identity is recorded rather than argued.

**CONSEQUENCES — the exact chain, and the blast radius as measured**

1. Rebuild `ms` at tag `ms-cli-v0.16.0` = commit `d49d5c099bab89a1738f0d0c3df9306b354d62c3` (annotated tag, target verified via `git rev-parse 'ms-cli-v0.16.0^{}'`; note HEAD `6fdfd36` is one CI-only commit past the tag — pin the tag, it is the release identity). Install to `~/.cargo/bin/ms`.
2. Re-record the `ms` entry in `oracle/pins.json`: commit `d49d5c0…`, new `sha256sum ~/.cargo/bin/ms`, version `ms 0.16.0`. The mnemonic-secret checkout currently shows three untracked files in porcelain, so a script-recorded `checkout_clean_when_recorded` would read false — either clear them first or record the flag honestly with a note.
3. **Re-anchor S0-trace-a with `gaterecord -force` over the SAVED walk** — not `-expect-only`. Measured: the S0 expectation's provenance names `ms 0.15.0` + `mk 0.13.0`, so `TestVendoredExpectationsWereDerivedFromThePinnedToolchain` goes red on a pin move; **and** the S0 *record* embeds resolved `ms@ddfa497`, so `TestEveryGateRecordOnDiskVerifies` → `VerifyRecord` goes red too (`record.go:365-369`). `-expect-only` fixes only the first. F-177 sanctions exactly this: *"an oracle re-pin cannot reach the device path; `gaterecord -force` over the saved walk is the sanctioned rebuild"* — no new emulator walk. The walk file is rewritten byte-identical (same input bytes); the record's census/digests are unchanged; only its resolved-oracle block and timestamp move. Note the tension with `-expect-only`'s doc comment about rewriting `recorded_at` — F-177's explicit sanction governs a pin move, and the commit message should say the walk is the saved one, census byte-identical. (`VerifyRecord`'s error text "re-walk, do not edit it" is wrong advice for a pin move; a wording follow-up, not a blocker.)
4. **One atomic commit**: `oracle/pins.json` + `S0-trace-a.record.json` + `S0-trace-a.expect.json` (+ the byte-identical walk if git sees it, which it should not). Gate output in the commit message: untagged `go test ./oracle/ ./gui/ ./sysw/` green **and** `./scripts/oracle-live.sh` green — the latter is also the empirical proof that 0.16.0 reproduces the committed S0 expectation byte-for-byte.
5. The S2 md1 golden is **untouched** by an ms bump — measured: `gui/testdata/s2_md1_golden.expect.json` names only `('md', 'md 0.13.0')`, and `CheckProvenance` deliberately checks only the oracles an expectation names (`expectfile.go:229-234`).

**md and mk:** `md` pin `5a0a4f41` **is** descriptor-mnemonic HEAD and the `md-cli-v0.13.0` tag — 0 commits behind, no action. `mk` pin `a38a908` lags HEAD by exactly 2 commits — `8dc5dcb` (ci: build `ci/**`) and `3462157` (docs) — neither touching codec or CLI behavior; there is **no newer mk tag** (latest is `mk-cli-v0.12.1`; the pinned binary's `mk 0.13.0` is an as-yet-unreleased version at the mk-codec 0.5.0 semantics commit the plan's §1a correction depends on). No action for S5; **hygiene note for S6/ship**: tag `mk-cli-v0.13.0` so the pin names a release.

**Drift detection — recommended, oraclelive tier:** nothing today asks whether a primary moved under an honest pin (resolution checks only the installed binary's hash), and `oracle/live_test.go`'s package comment even **claims** *"this file is the drift check … it asks whether the primary has changed under a pin that did not move"* while nothing in it does — the comment overstates (the comments-outlive-their-conditions class). Add a `TestPinsAreCurrentWithTheirPrimaries` behind `oraclelive` comparing each `pins.json` commit to the latest `*-cli-v*` tag in the sibling checkout (absence of a checkout fatal under the tag, per the tier's own rule). It must NOT go untagged: CI has no sibling checkouts, so an untagged version needs a skip, and the no-skip directive forbids that shape. Pair it with the re-review's M-2 fix (record `oracle-live.sh`'s exit code in every stage-close report) so "the drift question was answered on date X" is on disk.

**WHAT I VERIFIED**

- `oracle/pins.json` — ms @ `ddfa497…` / sha256 `e63d9cb…` / "ms 0.15.0"; installed `~/.cargo/bin/ms` reports `ms 0.15.0`, sha256 `e63d9cb524c839e94e24bd283620db489d47363a760f6f84541e882ba783cec0` — all three installed binaries match their pins byte-for-byte today (md `9ef480ad…`, mk `030ca218…`), so resolution is currently green and the pin is honest, exactly as F-177 states.
- `git log --oneline ddfa497..ms-cli-v0.16.0` — six commits; `git rev-parse 'ms-cli-v0.16.0^{}'` → `d49d5c0…`.
- Expectation/golden oracle-name blast radius: printed both files' derivation blocks (S0 → ms+mk; S2 golden → md only).

## 2. ORDER, and the red windows

1. **Pin bump commit** (above). Red windows: **(a) unavoidable, local-only** — between installing the new ms and committing the atomic re-anchor, `cmd/gaterecord` and the `oraclelive` tier refuse on the maintainer machine (hash mismatch, by design); keep it short. **(b) avoidable and must be avoided** — committing `pins.json` without the re-anchored record + expectation turns untagged CI red (`TestVendoredExpectations…` and `TestEveryGateRecordOnDiskVerifies`); the atomic commit removes this window entirely.
2. **S5.0, the oracle extension** — no CI red window at any point: the new kind is additive and no record of it exists until S5 mints. Closes green (untagged + one recorded oraclelive run) before any tail code.
3. **S5 proper** — tests-first per the plan, tail, Trace B walk, mint via `cmd/gaterecord`; between tail-landing and mint there is no red (the record simply does not exist yet; only S0's is demanded by name), but S5 may not close until the mint has run and the untagged suite is green over the new record.

## 3. THE TRUSTWORTHINESS QUESTION, plainly

**Yes — but only by a measurement the gate itself cannot see, which is why the pin still moves first.**

With the pin as it stands, S5's byte-identity gate passing **would** mean the engraved artifacts match what today's primary produces, because the 0.15.0→0.16.0 delta is measured byte-inert for every invocation the gate makes:

- `git diff --stat ddfa497..ms-cli-v0.16.0` — 11 files; **`crates/ms-codec/src` is untouched entirely** (only its `tests/parity_smoke.rs` moved), so `ms encode` output cannot have changed.
- The only behavioural source change is `crates/ms-cli/src/cmd/derive.rs` + `format.rs`: a new `bip48` enum value mapping to the **same** path as `bip48-p2wsh`, a `script_type_defaulted` JSON field (always emitted, `false` for explicit templates — and Go's `json.Unmarshal` into `msDeriveJSON` ignores unknown fields), and a stdout/stderr notice emitted **only** when the bare template is used. The gate passes `bip48-p2wsh`/`bip48-p2sh-p2wsh` explicitly (`templateForOrigin`, `oracle/expect.go:317-323`), so fingerprint, path and xpub values are identical between the two versions for every call the oracle chain makes. `mlock.rs` is fmt churn; the rest is tests/CI/docs.

So the stale pin would not launder a wrong byte **today**. But that "yes" rests on a source-diff argument performed outside the machinery, recorded nowhere a gate reader looks, and it expires at the next ms release; meanwhile the gate's text binds to *"the current primary"* and an S5 record minted now would permanently carry `ms 0.15.0` provenance on the strongest gate in the plan. The bump converts an expiring argument into a recorded identity, costs one atomic commit, and per §1b was due at the start of this cycle anyway.

## 4. OTHER BLOCKERS AND NON-BLOCKERS, checked rather than assumed

**The `oraclelive` split does NOT prevent S5's gate from running.** Where the gate actually runs, verified: (a) **mint time** — `cmd/gaterecord` is a command, not a test; its live derivation is unconditional (*"THERE IS NO WAY TO TURN THIS OFF"*, `cmd/gaterecord/main.go:27-31`) and runs on the maintainer machine where the pinned binaries resolve; (b) **every machine incl. CI, untagged** — `TestEveryGateRecordCensusMatchesItsCommittedExpectation`, provenance and kind-consistency tests enforce the minted record with no toolchain and no skip path; (c) **the `oraclelive` tier is only the freshness/drift layer**, invoked by hand (`./scripts/oracle-live.sh` — exists, verified) and compile-checked on CI (`go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`, `.github/workflows/test.yml:68`). One discipline requirement follows: S5's close must include one recorded oraclelive execution, or the freshness arm has never run for the new kind (M-2's gap).

Additional items found, none previously named in the brief:

1. **`Expect` carries no engrave mode** — nothing in the tuple or the expect block says full vs watch-only, which decides whether ms1s are expected. The new kind must encode it (§1a). A derivation that guessed the mode would derive the wrong census and refuse honest walks — fail-closed, but a stage-blocker if unplanned.
2. **`ms encode --group-size` defaults to 5** — a derived ms1 with separators can never match an engraved unbroken string; the new step must pass `--group-size 0` (and `--no-engraving-card`). Fail-closed but confusing; the exact trap mkEncode already documents for mk.
3. **`md encode`'s stdout header** (`chunk-set-id: 0x…`) must be handled explicitly in the new md step — predicted verbatim by `mkEncode`'s comment.
4. **The gate's `ms encode --hex <entropy>` wording vs the inputs file's words**: the deriver holds seed WORDS (`InputsFile.SeedWords`), and `ms encode` accepts `--phrase` or `--hex` into the same encoder. Whichever form the kind invokes, the recorded arg-form in the expectation shows which ran; if the plan's literal `--hex` is insisted on, the entropy must itself come from a primary invocation, not fork code. Implementation detail for the S5 brief, not a blocker.
5. **S5-owned follow-ups come due with the stage** (reconcile-first): **F-182** (end-of-bundle ms1 reminder title on the Build path) and **F-185** (modal body scrolls off the first frame — S5 owns the engrave tail's screens and inherits the class). Neither blocks the byte gate; both block S5's close.
6. **The Trace B walk** must emit census + `perPlateDigest` + `presented === 0` (F-174) or the mint refuses — the plan's S5 matrix names `cmd/emu/walk_trace_a.js` as the stage's walk gate; extending it or adding a sibling is S5 work, noted so the mint's requirements are in the brief.
7. **F-176 is withdrawn and stays withdrawn** — the md-CLI divergent-origin premise was false, measured; nothing on the md side blocks S5, and the decode-equivalence fallback is moot and must not be used (`FOLLOWUPS.md:6015,6067-6077`).

## WHAT I VERIFIED (consolidated command log)

- `git -C seedhammer rev-parse HEAD` → `80d0c5d…`; porcelain clean before and after.
- Read in full: `oracle/expect.go`, `oracle/expect_test.go`, `oracle/oracle.go`, `oracle/expectfile.go`, `oracle/inputsfile.go`, `oracle/live_test.go`, `cmd/gaterecord/main.go`, `oracle/record.go:300-380`, `oracle/record_test.go:355-400`, plan lines 1137–1316, `CONTINUITY_2026-08-15c.md`, FOLLOWUPS F-176/F-177 and the S0b fold re-review M-1/M-2/M-3.
- `ls oracle/gaterecords/` → exactly one record triple+inputs (`S0-trace-a.*`).
- Oracle identity: `~/.cargo/bin/{md,mk,ms} --version` + `sha256sum` — all three match `pins.json` exactly.
- Primary deltas: ms `ddfa497..ms-cli-v0.16.0` (6 commits, diffstat + full `derive.rs`/`format.rs` diff read); mk `a38a908..HEAD` (2 commits: ci, docs); md `5a0a4f41..HEAD` (0 commits).
- Tag: `ms-cli-v0.16.0` is annotated, target `d49d5c0…`, created 2026-08-15.
- Expectation oracle-names: S0 expect → ms+mk; S2 golden → md only (python json read, output pasted in §1b).
- `nix develop --command go test ./oracle/` → `ok seedhammer.com/oracle 0.071s`.
- `scripts/oracle-live.sh` exists; `.github/workflows/test.yml:68` compile-checks the tagged files.
