# Fold, part A — descriptor-mnemonic (the Rust PRIMARY): two findings from the composer fable review r0

You are the single implementer for the Rust half of the fold. Repo
`/scratch/code/shibboleth/descriptor-mnemonic`, base `922778ad0400957840da26081846fc761deaca41`
(md-codec 0.44.2, md-cli 0.16.2). Work on a branch `fable-r0-fold` in your own
worktree: `git -C /scratch/code/shibboleth/descriptor-mnemonic worktree add -b fable-r0-fold /scratch/code/shibboleth/.tmp/fold-rust 922778ad0400957840da26081846fc761deaca41`.
Commit per finding; commit nothing to master; do NOT push; no sub-agents; never
read `.jsonl`. `export TMPDIR=/scratch/code/shibboleth/.tmp`; own
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/fold-rust-target`; never build
under `/tmp` (tmpfs). Tests: `cargo nextest run --locked` (24 cores; never run
a suite twice for counts — capture once, grep). `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` are part of the gate.

The findings are in
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-funds-safety.md`
(C-1) and `.../composer-fable-r0-steel-restore.md` (I-2). Read those two
sections in full. Reproduce each defect first (RED), then fix. Prescribed
remedies in the reports are not authoritative; what follows is.

## Finding 1 (from lens 1 C-1) — refuse a second key-less path at validate time

MEASURED by the controller with `md compose --wrapper wsh --experimental` at
0.16.2: every path list with TWO OR MORE key-less paths is refused after
lowering ("Miniscript is malleable"), whatever locks they carry and wherever
they sit; every list with at most ONE key-less path is emitted. The reason is
structural: paths chain right-leaning as `or_i(P, rest)` (`or_d` under a
bare-multi head), and `or_i(l, r)` is non-malleable only if `l.safe || r.safe`
(`vendor/miniscript/src/miniscript/types/malleability.rs:265-275`, `safe` =
a satisfaction needs a signature); a key-less path is never safe, and with two
of them some `or_i` has two unsafe arms.

Today the primary reaches that verdict only by the md-cli readback in
`crates/md-cli/src/cmd/compose.rs:~700` (F-600). The device's Go port has no
rust-miniscript, so it cannot read back — it needs the rule stated. Put the
rule where the knowledge belongs:

- `md-codec` compose `validate()` (the function `md/compose.go` ports) refuses
  a path list with more than one key-less path, with its OWN error variant and
  a message in the F-600 voice ("give one of them a key, a timelock does not
  help, or fold them into one path" — say what is true: a lock does NOT make
  it safe). The md-cli readback stays.
- A conformance vector for the refusal (both `[keyed, K, K]` and
  `[keyed, K, K+older(5)]`, and a control `[K+older(5), 2of3]` that is
  ADMITTED), in the vectors directory the Go port vendors from. Name the
  vector file in your report so the fork implementer can vendor it.
- CHANGELOG entries; version bumps md-codec → 0.45.0, md-cli → 0.17.0 (a new
  refusal is a behaviour change).

## Finding 2 (from lens 3 I-2) — host seating fills an ABSENT fingerprint from the seated card

`crates/md-cli/src/seat/compose.rs` (module doc lines 1-14, `compose` at
~70-83) fills only `tlv.pubkeys` and "never overwrites a fingerprint-free
declaration: the policy author's choice not to state one is inherited". That
conflates two things. A composer template minted with an UNSEATED slot (§8p)
has no fingerprint for that slot because the key was not known — not because
anyone chose. When the host seats that slot from a card that carries a
fingerprint, the completed wallet renders that key with NO origin at all
(`to_miniscript.rs:148` drops the origin when the fingerprint is absent) and
its wallet id differs from the id the device would mint for the same three
keys fully seated. Spec §5 l.218: "EVERY slot declares an origin and, when
seated, the master fingerprint of the seated key."

Change: when the declaration for a slot has NO fingerprint and the seated
card carries one, ADD it to `tlv.fingerprints`. NEVER overwrite a present
fingerprint (that rule stands, and its test at `:291` stays). Update the module
doc to say exactly this. Tests: (a) the lens-3 C13 shape — a partial template
(two seated `key:` slots + one unseated at the lowest-free account) completed
with a host-minted 3-chunk card → completed descriptor carries
`[fp/path]` on the third key and `md inspect --json` shows three fingerprints;
(b) the completed card's wallet id EQUALS the id of the same policy composed
fully seated with the same three keys (build both in the test); (c) a present
fingerprint is not overwritten by a card with a different one (refusal or
inheritance — whichever the existing code does, pin it). Say in CHANGELOG that
a host-completed partial template's id changes from what 0.16.x produced, and
why.

## Report — your FINAL action

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-rust-implementation.md`:
commits (SHAs, one line each), the gate output (nextest summary line, fmt,
clippy), each RED test's RED output before the fix, the vector file path,
deviations from this brief with reasons, and anything you could not do.
Return only the verdict line, the branch tip SHA, and the path.
