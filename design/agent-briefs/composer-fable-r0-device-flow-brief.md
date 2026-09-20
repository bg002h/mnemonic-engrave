# Lens 4 — DEVICE FLOW ADVERSARIAL: shown one policy, cut another; failure states

Read `composer-fable-r0-common.md` first. Your worktree slug: `flow`.
No regtest node needed (Core is lens 1's instrument); if you want one:
`-rpcport=18843 -port=18844`.
Your report: `design/agent-reports/composer-fable-r0-device-flow.md`.

## THE ONE QUESTION

Through any sequence of taps the SH2's real buttons allow — Back legs,
discard / consent / door guards, a preset then edits, seat / unseat / re-seat,
switching a slot's SOURCE (payload `key:` / `hash:` / `now:` / `phrase:`
records vs seeds vs the same seed at another account), paging, the digit
pad, self-check, and the engrave-mode and form pickers — can the operator be
**shown one policy and cut another**? That is: the review / consent /
census / engrave screens state ids, k-of-n, lock kind and value, hash, key
order, NUMS note, same-seed note, form, plate count — and the strings that
reach the plate planner differ. Second half: at each **failure state**
(power loss or reset mid-flow, a malformed or reserved payload record, a
seed filling several slots, the 32-slot / 8-path / 9-per-path / 20-`multi`
ceilings, pad edges — the 2009-01-03 date floor, 65535, 0x400000+u, the
after/height threshold — the all-hash policy, the keyless-wsh experimental
door), does the device REFUSE, WARN, DEFAULT, or stay silent — and is silence
ever worse than telling the operator nothing?

## Method — the harness is the device

The scriptable device is the Go test harness: `gui/run_harness_test.go`,
the walk tests (`gui/*_walk_test.go`, especially
`gui/wallet_policy_descriptor_walk_test.go`, `gui/payload_door_walk_test.go`,
`gui/multisig_build_walk_test.go`), and `gui/composer_*_test.go`. Write NEW
walk tests in your worktree that script the sequences you suspect, assert
on the strings handed to the plate planner (not on screen text alone —
text extraction cannot see what the plate gets), and mutation-check every
assertion you rely on (break the code, show RED, restore). Run them with
`-run '^TestName$' -v` and confirm they ran. If you touch a shared harness
helper, run the whole gui shard set once at the end
(`scripts/gui-shard-test.sh ./gui/ 24`) and paste the count.

The state machine is `gui/composer_state.go` + `composer_flow.go`; the
guards are `composer_discard.go`, `composer_consent.go`, `composer_door.go`;
sources and seating in `composer_sources.go`, `composer_seat.go`; records in
`sysw/composer_records.go`. A navigation complaint is a STATE AUDIT: the
W-6→W-7 Critical was a Back leg that skipped a screen and thereby skipped
the seat-discard guard. Enumerate every Back/Cancel edge and ask which
guard it bypasses and which state it leaves stale (seated slot, chosen
form, engrave mode, pad value, preset flag, the "changed-id" banner's
baseline).

Classify every divergence you find as **refusal / warning / default / not
our concern / documentation only**, and it earns a finding only if the
wrong outcome is worse than telling the operator nothing. That rule is what
keeps this lens from generating cases without limit.

## Hunt list

- The changed-id banner (journey I-5): baseline capture and invalidation on
  every edit path, including edits made AFTER a Back from consent.
- Preset → edit: does any preset flag survive an edit and alter lowering or
  copy (the "picker that opens on row zero proposes a setting" class)?
- Seat a slot from a `key:` record, then switch that slot to a seed, then
  Back: which key is on the card? Same for `hash:` → `phrase:` on a hash
  path; `now:` and the after/date pad.
- Re-seating the same seed at another account after the same-seed notice:
  does the notice re-fire, and does the census count one seed plate?
- Digit pad: leading zeros, 0, 65536, 0x400000 exactly (zero units), values
  straddling 499,999,999 / 500,000,000, a date before 2009-01-03, February
  30 — what reaches the artifact and what the screen shows.
- Paging (§7c stub screen 6/frame, §7d pick list 7/frame, §7e consent
  7/frame): can a consent line be on a page the operator never reached
  when they tap CONTINUE? Can a pick-list row be off the first page such
  that a kind is unchoosable (that class shipped once before)?
- Payload records: a `phrase:` record whose body is not lowercase hex is
  reserved and must be INERT (`sysw/composer_records.go`); a `key:` record
  with a mainnet xpub on a testnet device; a duplicate `key:` (same key,
  two records) — refused as unsupported, never seated twice; a record with
  a leading/trailing space, CRLF, upper-case hex.
- The keyless-wsh EXPERIMENTAL door and every confirm-to-proceed screen:
  dismissed ONLY by a tap on CONTINUE, Back returns to the shape (§8
  header) — test each one, including from the second page of a paged one.
- Self-check (`composer_selfcheck.go`): what does it actually compare, and
  can it pass while the census disagrees?
- Engrave: form picker offers only what seating allows
  (`composerFormsFor`); can a stale `assigned` array offer form A with an
  unseated slot? Full vs Watch-only with a seed that filled several slots.
- Reset mid-flow: is any state persisted across a reboot, and is a
  half-composed policy ever resumed without being re-shown?

## Settled for this lens

- The journey walks already done: `composer-policy-journey.md`,
  `composer-journey-followons-review.md`, `composer-script-preselect-review.md`,
  `composer-S3-plan-R0-r0-journey.md`, `composer-S4-plan-R0-r0-journey.md`,
  and W-1..W-7 (`composer-S4-W*-verification.md`). Read their findings so
  you do not re-file what was fixed; re-test only where you suspect a fix
  did not hold.
- The hardware has no camera and a fixed button set; the emulator is a
  window, not a script target.

## Deliverable specifics

A **divergence table**: sequence (as taps) → screen promise → what reached
the planner → class (refusal/warning/default/n-o-c/doc) → worse than
silence? → severity. And a **state-audit table** for the Back/Cancel edges:
edge → guard it should pass through → does it → stale state left. Every
new test file you wrote, named, with its mutation proof.
