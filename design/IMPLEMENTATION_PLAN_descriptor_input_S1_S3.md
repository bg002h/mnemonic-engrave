# IMPLEMENTATION PLAN — descriptor input, S1 + S3

**Status:** DRAFT, round 0. No code before this plan's R0 closes 0C/0I.
**Spec:** `SPEC_descriptor_input.md`, FINAL GREEN at `b949d18` (20 rounds +
the 15-finding walk; closure verdict and leaves-open list in
`design/agent-reports/R0-descriptor-input-spec-r20-closure.md`).
**Phase order:** S1 → S3 per F-418; S2 and F-423 parked for the device.
**Plan baseline revs** (for `scripts/plan-staleness-check.sh`):
mnemonic-engrave `b949d18` · descriptor-mnemonic `6864f377` · seedhammer
fork `d402f18`.

**The one structuring decision, from the cycle's banked lesson:** the
vector file is authored FIRST and is the failing test the parser is built
against. Nothing in P1/P2 is specified in prose that a vector row can pin.

---

## P0 — the vector file, both repos' harnesses, red before green

Everything here is data + test scaffolding; no parser code.

- **P0.1** Author `crates/me-cli/testdata/descriptor_seam_vectors.json` per
  spec §7: ≥ 68 physical rows, 9 tags, 85 tag-slots, per-tag minima
  (`formats-happy` 4 · `promotion-near-miss` 15 · `narrowed-4.7` 14 ·
  `accepted-extreme` 1 · `narrowed-4.2` 5 · `neither` 3 · `whitespace` 3 ·
  `md1-splits` 6 · `gate` 34), row schema with `covers`, `md1_admits`
  (REQUIRED, no default), `canonical` on every `host_admits=true` row,
  `device_probe` markers (`panic:parse` / `panic:encode`), the value fields
  (`address_0`/`address_1`/`md_descriptor_contains`/`wallet_id`), and the
  per-row `sha256` of `input`. Sources of measured values: the r1–r19
  probe corpus (r6's eight desk-run rows and r15's decision table are
  starting material; every address/id re-derived at authoring time via
  `descriptor-mnemonic/target/debug/md` and a Go scratch probe — never
  transcribed from reports). The `wallet_id` values computed over the
  (a′)-materialised policy per §5.4.
- **P0.2** Rust harness `crates/me-cli/tests/descriptor_seam.rs`: pins the
  file's sha256 as a literal; asserts the manifest arithmetic (tag minima,
  85 slots, 68-row floor, `covers`+`md1_admits` present on every row, the
  two permitted overlaps only); asserts row-level invariants that need no
  parser (requirement 5's non-vacuity; `canonical` presence). The
  host-column assertions land **`#[ignore]`-tagged in P0 and un-ignored in
  P1/P2** as their surfaces exist — pin-the-gap style: each ignore names
  the phase that removes it, and P3's exit gate greps for zero remaining.
- **P0.3** Fork side (Rust-first, then vendor): copy the file byte-identically
  to `seedhammer/nonstandard/testdata/descriptor_seam_vectors.json`; add
  `nonstandard/descriptor_seam_test.go` as **`package nonstandard_test`**
  (r6/r19: import-cycle-proof), pinning the same sha256 and asserting the
  DEVICE columns now: `device_admits` via `nonstandard.OutputDescriptor`
  on the input, requirement 4's fixed point on `canonical`, `address_0/1`
  via `address.Receive` where present, `device_probe` rows never fed to
  the panicking function. `sysw_class` rows: skipped with a named reason
  (S2's arm) — the skip is ALLOWED here because the column is optional by
  spec and the skip names its phase.
- **P0 gate:** Go test green on the fork (device truths hold); Rust harness
  green on its non-parser assertions; both sha256 pins byte-equal;
  `cargo nextest run --locked` baseline still green (863+ in
  descriptor-mnemonic untouched; engrave suite untouched).
- **P0 review:** proportional — a sonnet pass over the vector file against
  §7's bullets (fold-vs-manifest, counts by machine), NOT a fresh audit.

## P1 — the cascade and the admission predicate (host columns green)

- **P1.1** New module `crates/me-cli/src/descriptor/` (bin-internal, no new
  deps — spec §4.7 chose a small closed parser over rust-miniscript):
  `cascade.rs` (the four branches in device order, §4.1–§4.5 semantics:
  BlueWallet headers + exactly-8-hex fingerprints + the four normative
  refusals incl. F-419's zero-cosigner; BIP-380 grammar with the five
  version bytes; case-insensitive JSON; promotion with the three paths and
  the announced, key-as-supplied echo), `admit.rs` (the seven shapes + the
  `multi` md1-path twins + conjuncts 2–7), errors carrying §6's cause
  taxonomy. TDD: un-ignore the P0 host-column tests FIRST, watch red, then
  build to green; unit tests only where a vector row cannot reach (the
  cause-selection ranking, per §6's five steps).
- **P1.2** The whitespace normalisation (§4.6) and single-document mode +
  whole-input discriminator with the descriptor-shape gate (§5.1) — the
  gate is IMPLEMENTED FROM THE VECTOR ROWS (the 34 `gate` rows are its
  spec; the prose tests are guidance). The shipped record refusal
  (exit 4) is pinned untouched by the existing `sysw_cli.rs` suite.
- **P1 gate:** all P0-ignored host assertions un-ignored and green except
  the `--as md1`-execution rows (P2's, enumerated); full workspace
  `cargo nextest run --locked` + clippy `-D warnings` + fmt; the Go seam
  test re-run green (file untouched → sha256 unchanged).
- **P1 review:** proportional opus pass over the diff: does the cascade
  match the measured device semantics the spec pins, and does any refusal
  text drift from §6's quoted rows (which P2 will pin verbatim)?

## P2 — `--as md1` end to end (S3 proper)

- **P2.1** `--as <descriptor|md1>` flag surface: `md1` implemented;
  `descriptor` present and refusing with §5.1's window text (both
  variants, the carriage rule, admission-precedes-window, the choice-block
  marking) — the S3-window behaviour IS this build's behaviour.
- **P2.2** The md1 build path: `md_codec::encode::Descriptor` constructed
  in-process (§5.3; per-key rules (a)/(a′)/(a″) with materialisation, the
  `multi` twins, TLV fingerprints/pubkeys, divergent-path mode where keys
  differ), `encode_md1_string`/`split`, records packed as `ClassMDMK`.
- **P2.3** The identification block (§5.4): two tiers, the full line set —
  wallet-id (materialised-policy base, `none` line for no-md1-form
  wallets), address 0 + compare prompt, the watch-only line, the (a′)
  annotation — printed before pack and refusals per the follower rules.
- **P2.4** §6's refusal texts, verbatim, one test per row (§11 item 4's
  S3-bound rows; each asserts TEXT and exit; the five-case item-5 matrix).
- **P2.5** F-421 (the top-level converter's descriptor-shaped referral) —
  in-tool, with-S1-or-after per its filing; one refusal string + one test.
- **P2 gate:** ZERO `#[ignore]` remaining in the seam harness (grep-gated);
  the md1-splits/gate/address/wallet-id/read-back assertions all live;
  full workspace suites + fmt + clippy; `me sysw pack --as md1 --in
  wallet.txt` (the walk's J1 fixture) produces a container whose records
  `md decode` round-trips with the materialised `<0;1>/*` and whose
  address 0 equals the Go derivation (§11 item 2, all four formats — the
  JSON exemplar non-`/0/*` per §11's note).
- **P2 review:** the MANDATORY post-implementation adversarial execution
  review, whole S1+S3 diff, opus (fable per the standing triggers), report
  persisted; plus the two walked journeys re-run as integration tests
  (J1 BlueWallet → md1 cards; J2 bare zpub → refusal-with-identification
  under `--as descriptor`, cards under `--as md1`).

## P3 — acceptance, records, ship

- **P3.1** §11's S3-bound items discharged and named in the commit: items
  2, 3 (counting test), 4 (S3 rows), 5 (five cases); items 1/6 and the
  `--as descriptor` rows recorded as S2-parked (F-418).
- **P3.2** F-416's one-sentence cross-ref lands in
  `SPEC_systemwide_payloads.md` §5.6 (owned "at ship" of this cycle).
- **P3.3** CHANGELOG entry; FOLLOWUPS reconciliation sweep (F-419 → done in
  P1, F-421 → done in P2, F-413 → spec-as-written noted, F-414/F-420/F-422
  → confirmed parked with owners); continuity + memory updates.
- **P3.4** Merge worktree → master; push via `scripts/push-via-staging.sh`;
  the docs-only fast path validates on this cycle's doc pushes and the
  full path on code pushes.

## Out of scope (parked, with owners)

S2 whole (device classifier arm, `--as descriptor` packing, §11 items 1/6,
QR plates) — F-418, awaits the SH2. F-423 plate packing — with S2's
firmware. F-420 (`md`'s referral) — descriptor-mnemonic, with-or-after S1;
stretch if the night allows, own commit + own push. F-413/F-422 — operator
rulings, spec-as-written meanwhile.

## Stop-rules and boundaries (the overnight mandate)

Per `CONTINUITY_2026-08-28-overnight.md`: loops > ~5 rounds park with a
note; no tags/releases/publishes/on-device actions; pushes in scope.
