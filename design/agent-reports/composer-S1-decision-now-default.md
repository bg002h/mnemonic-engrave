# Decision — the `now:` auto-append default for `me sysw pack` (composer Stage 1)

**Decided:** how `me sysw pack` decides whether to append the pack-time `now:` record.
**By whom:** a STAND-IN ruling for the operator (Brian Goss), made 2026-09-02 so Stage 1 does not stall. **Revocable** — the operator may veto or amend it with one line; until then it binds the S1 plan and the spec edits below.
**Nothing in any repository was edited by this agent.** This file is the only output.

**Read, in full or at the cited lines:**
- `design/SPEC_wallet_policy_composer.md` — STATUS line; C24 row (l.71); §6a (l.259-328, both statements of the append rule); §6b (l.330-360); §7a door (l.373-387); §7d seating (l.428-482); §7g divergence table (l.541-577); §10 item 2 (l.815-819); §12 items 2-4 (l.859-877).
- `design/BRAINSTORM_wallet_policy_composer.md` — §2 rows C14 (l.40), C24 (l.50), C25 (l.51); §3.12 items 4, 9, 19 (l.316-382); §4 item 5 "Entry UX. RULED — C24/C25" (l.408) — the "i5" the operator's "document now for i5" refers to.
- `design/SPEC_systemwide_payloads.md` — §3.3.2 admission table and its Wallet Policy row (l.358-400); §5.3 record forms (l.615-660); §5.4 digest-shown table, "once per payload", §5.4.1 identity (l.730-812).
- `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` — header "Open question for the operator" (l.17-19); contract lines (l.25-27); Task 4 in full (l.934-1157), including the stderr line (l.1094-1098) and the six named casualty tests (l.1148).
- `design/agent-reports/composer-spec-R0-r0-correctness.md` I8 (l.310-345) — the finding that produced the default; `composer-spec-R0-r2-adversarial.md` I-4 (l.337-362); `composer-spec-R0-r3-fold-verification.md` l.46.
- `design/CONTINUITY_composer_2026-09-01.md` l.40-66.
- `design/FOLLOWUPS.md` F-417 (l.14650-14662), F-422 (l.14611-14617).
- Code: `crates/me-cli/src/sysw/identity.rs` (header comment); `crates/me-cli/src/sysw/mod.rs:256-267, 575-590`; `crates/me-cli/tests/sysw_cli.rs:260-278` (`a_secrets_only_payload_reports_no_digest`); `crates/me-cli/tests/descriptor_seam.rs` (pins input sha256s, not payload bytes); `scripts/gen-tx-journey.sh:125-144` (five `me sysw pack` calls, all `tx:`); fork `gui/sysw_session.go:27-82`, `gui/sysw_load.go:150-195`.

## RULING: (c), NARROWED — `me sysw pack` appends `now:` last by default only when the operator's records include at least one `key:` or `hash:` record (the two classes that, like `now:` itself, are admitted at Wallet Policy alone) and none is a `now:`; a new `--now` flag forces the append onto any payload; `--no-now` suppresses it; a supplied `now:` always wins and appends nothing. Seeds, descriptors and md1/mk1 cards do NOT trigger the default.

## Reasons

**What it protects.** Every payload a shipped program consumes stays byte-identical to today. The case that decides it is not the 25 bytes; it is the ceremony. A sealed seed-only payload — Backup Wallet's main case — has `pub_len == 0`, no displayed digest, and "opening is what authenticates it" (payload spec §5.4, `mod.rs:579-580`). The device load flow shows the "Payload Digest" compare screen whenever `PubLen > 0`, and declining it UNLOADS (`gui/sysw_load.go:164-190`, the 2026-08-13 ruling). Under (a) every seed backup packed from today on would gain that screen, with a number that differs on every pack because it hashes the pack time, for an operator who never asked for a time and is not building a policy. That is a new step in an unrelated program's ceremony, plus the pack time of a seed backup sitting in the clear in flash. The F-417 instinct applies: never widen an unrelated surface to make a new one convenient.

**Why `key:`/`hash:` and not the controller's list.** The predicate is structural, not a guess at intent: `now:` rides only on records that share its admission footprint. `key:` and `hash:` exist for nothing but the composer (payload spec §3.3.2: Key, Hash, Now — each admitted at Wallet Policy alone), so a payload holding one IS a composer payload and gets its bound without a flag. A Descriptor or md1 record is the opposite signal: it is the door's "From payload" route (§7a), a policy that already exists with its locks already encoded — no lock is entered, the bound is dead weight — and those two classes feed Engrave Bundle, Single-Sig and Multisig, three shipped programs whose payloads (and the descriptor-input journey verified on hardware 2026-08-29) would change bytes. Concretely: of the plan's six measured casualties, three are descriptor tests (`descriptor_as::item_1…`, `descriptor_as::item_2…`, `sysw_cli::the_descriptor_show_block_leaves_every_other_container_byte_identical`); under the controller's (c) they stay casualties, under this ruling none of the six packs a `key:` or `hash:` (the classes do not exist yet) so none should move — the implementer confirms that by running Task 4 step 4 as written. An mk1 card is the one genuinely ambiguous record (a seating source at Build, or Multisig's supplied-md1 path); it loses to keeping the rule one line long, and `--now` is the remedy.

**Why not (a).** The operator's words were "could be provided as a payload item" — permissive — and the C24 row records the record as operator-authored. A default that fires on every pack turns a permission into a tax on every payload, and puts nondeterminism into the record list of every fixture in the constellation (correctness I8's third point) rather than only the composer's. The record is admitted at one program; its default should be at most as wide as its admission.

**Why not (b).** `now:` exists because the operator will not remember the time; a flag they must remember defeats it in the one journey it serves. Someone who has gone to the trouble of packing `key:` records is composing a policy — that is the moment the bound should arrive unasked. (b) also makes the §12 item 2 journey depend on a flag the docs must carry forever.

**Seeds, decided: not counted.** A seed-only payload used at Build with an absolute lock gets §6b's bare disclaimer ("This device cannot tell the time…"), which the spec already defines (brainstorm §3.12 item 4, journey I-2) and which never lies — a missing bound can only lose a refusal, never invent one. Counting seeds would recreate exactly the Backup Wallet harm above, since the host cannot tell a seed packed for Backup from one packed for Build. The composer does NOT need the bound whenever a seed is present; it needs it when the operator is composing, and `--now` is a one-word way to say so.

**What it costs.** One new flag (`--now`, `conflicts_with = "no_now"`); one predicate in Task 4 that classifies the records before deciding (the classifier is already in the library; a malformed `key:`/`hash:` is `Unknown` and refused by `pack_with` regardless, so the predicate counts only records that classify as `Key` or `Hash`). An operator packing seed-only or card-only for Build who wants a bound and forgets `--now` sees the disclaimer instead of a refusal of an already-past date. The spec's hard rules are untouched: last record, operator-supplied wins, two operator `now:` refused, `--no-now` for fixtures. The I-4 remedy (the refusal names the operator's own index) still holds because the host never appends when a `now:` is present.

## What changes

**`SPEC_wallet_policy_composer.md` §6a, first statement (l.294-298).** Current:

> `me sysw pack` auto-appends `now:` ONLY when the operator's records contain none, so an operator-supplied `now:` wins silently and pins a deliberate bound; two OPERATOR-supplied `now:` records are a host refusal with a remedy (§8n: "Remove one."), and on the device both go inert with the door showing no bound.

Replacement:

> `me sysw pack` auto-appends `now:` ONLY when the operator's records contain none AND at least one of them is a `key:` or `hash:` record — the two classes that, like `now:`, are admitted at Wallet Policy alone — or when `--now` is passed; so an operator-supplied `now:` wins silently and pins a deliberate bound, and a payload holding no composer-only record (a seed for Backup Wallet, a `tx:`, a descriptor or card for the engrave programs) is byte-identical to a pack made before this cycle; two OPERATOR-supplied `now:` records are a host refusal with a remedy (§8n: "Remove one."), and on the device both go inert with the door showing no bound.

**§6a, second statement (l.308-311).** Current:

> `me sysw pack` appends `now:` as the LAST record ONLY when the operator's records hold none (the rule above, one statement of it); `--no-now` suppresses that auto-append so a fixture's pack output stays a pure function of its inputs (§10 item 2).

Replacement:

> `me sysw pack` appends `now:` as the LAST record ONLY when the operator's records hold none and include a `key:` or `hash:` record, or when `--now` is passed (the rule above, one statement of it); `--no-now` suppresses that auto-append so a fixture's pack output stays a pure function of its inputs (§10 item 2). A seed-only or card-only payload therefore carries no bound unless the operator asks; at Build the lock echo then shows §6b's bare disclaimer.

**§10 item 2 (l.815-819).** Current:

> `now:` appended last ONLY when the operator's records hold none (an operator-supplied `now:` wins), `--no-now` suppresses the auto-append for deterministic fixtures.

Replacement:

> `now:` appended last ONLY when the operator's records hold none and include a `key:` or `hash:` record, or when `--now` is passed (an operator-supplied `now:` wins; `--now` and `--no-now` conflict); `--no-now` suppresses the auto-append for deterministic fixtures; a payload with no composer-only record packs byte-identically to today.

**§7g (l.557).** Keep the existing `now:` row; ADD one: `| pack | seed-only or card-only payload for Build, absolute lock intended | DEFAULT: no bound appended; the §6b bare disclaimer at lock entry; --now adds one |`. §12 item 2 needs no change (its journey packs `key:` records, so the bound arrives by default; it also lists a `now:` explicitly, which wins). Payload spec §3.3.2 is untouched — admission does not move.

**S1 plan, Task 4, in one sentence:** the auto-append `if` gains the predicate "no `now:` present AND (`--now` OR some record classifies as `Key` or `Hash`)", `SyswCmd::Pack` gains `now: bool` conflicting with `no_now`, the TEXT-only test `pack_appends_the_pack_time_when_no_now_record_is_given_and_says_so` packs a `hash:` record instead and a sibling asserts a TEXT-only and a seed-only pack carry no `now:` with no flag, the stderr line keeps "Pass --no-now to omit it", and step 4's "SIX pre-existing tests gain `--no-now`" is expected to become zero — run it and record the count rather than assume it. The header's "Open question" is answered by this file; Tasks 1-3, 5-7 do not move.

**Brainstorm §3.12 item 9 and the `now:` clause of item 19:** SUPERSEDED by this ruling. Append a dated item (21) recording "default narrowed to `key:`/`hash:`-bearing payloads; `--now` added; stand-in ruling, `composer-S1-decision-now-default.md`" rather than rewriting either item, so the history of the controller default stays legible.

## What the operator should be told

You ruled that the SeedHammer could be handed the pack time as a payload record; the controller then made `me sysw pack` add that record to EVERY payload by default. I have narrowed it: the record is added automatically only when the payload already holds a `key:` or `hash:` record — the composer-only classes — so a payload built for Build gets its time bound without a flag, while every seed backup, transaction, descriptor and card payload packs exactly as it does today. The reason that decided it: the device shows a digest-compare screen for any payload with a public section and unloads if you decline, so the old default would have added that screen to every sealed seed backup and made its digest change on every pack. A seed-only Build gets the "cannot tell the time" line unless you pass the new `--now`; `--no-now` still turns it off; your own `now:` always wins. Veto with "use (a)/(b)/(c-wide)" and the plan reverts to one `if`.
