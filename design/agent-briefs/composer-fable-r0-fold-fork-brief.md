# Fold, part B — the SeedHammer fork (device): the composer fable review r0, all four lenses

You are the single implementer for the device half of the fold. Repo
`/scratch/code/shibboleth/seedhammer`, base `f5b068faf3049ccf97603dfbaa3709f12893df97`.
Branch `fable-r0-fold` in your own worktree:
`git -C /scratch/code/shibboleth/seedhammer worktree add -b fable-r0-fold /scratch/code/shibboleth/.tmp/fold-fork f5b068faf3049ccf97603dfbaa3709f12893df97`.
Commit per finding (RED test first, then the fix, in one commit whose message
names the finding by lens and id); commit nothing to main; do NOT push; no
sub-agents; never read `.jsonl`. Go is `/scratch/code/shibboleth/.toolchain/go/bin/go`
FIRST on PATH; `export TMPDIR=/scratch/code/shibboleth/.tmp`; never build
under `/tmp`. Tests: `CGO_ENABLED=0 go test ./md/ ./sysw/ ./mk/ …` for codec
packages; `-run '^TestName$' -v` for one gui test (a filter that matches
nothing prints ok — confirm with -v); the whole gui package ONLY through
`/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
from the worktree. `gofmt -l .` baseline is five pre-existing files
(`gui/transaction.go`, `gui/transaction_golden_test.go`,
`gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`); `go vet
./...` clean. Harness facts: EventRouter is one global pointer — release a hold
before the next; the SH2 has no camera and a fixed button set — never drive an
input the hardware lacks; every confirm-to-proceed screen is dismissed only by
CONTINUE and Back returns to the shape (spec §8 header); every FIXED body must
pass the modal-fits assertion (§12 item 5); ASCII only in copy. Copy lives in
`gui/composer_copy.go` and the spec's §8 is its mirror — when you change a
body, quote the new body in your report so the controller can mirror it.

Installed host CLIs on PATH (`~/.cargo/bin`): `md 0.16.2`, `mnemonic 0.103.1`,
`me 0.10.0`. `md` is your ORACLE for finding 1 and finding 2 below.

The four reports, in `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/`:
`composer-fable-r0-funds-safety.md` (L1), `composer-fable-r0-nunchuk.md` (L2),
`composer-fable-r0-steel-restore.md` (L3), `composer-fable-r0-device-flow.md`
(L4). Their test files are preserved verbatim: L4's in Appendix A of its
report; L1's under `/scratch/code/shibboleth/.tmp/fable-funds-work/harness/`
(three `zz_fable_funds_*_test.go`, package gui). ADOPT them: bring them into
the tree as `gui/composer_fable_r0_flow_test.go` and
`gui/composer_fable_r0_funds_test.go` (rename test funcs sensibly, keep the
mutation-proven assertions), make every RED-at-tip test GREEN by fixing the
code, never by weakening the test. Reproduce before fixing. The reports'
remedies are not authoritative; the list below is.

## Findings to fix, in this order

1. **L1 C-1 — at most ONE key-less path per policy.** MEASURED by the
   controller: `md compose --wrapper wsh --experimental` refuses EVERY path
   list with two or more key-less paths, whatever locks they carry and wherever
   they sit (`[keyed, K, K]`, `[keyed, K, K+older(5)]`, `[K+older, K+after]`,
   …), and admits every list with at most one (`[K+older(5), 2of3]`, `[keyed,
   2of2, K]`). Reason: `or_i(l, r)` is non-malleable only if one arm is
   `safe` (needs a signature); a key-less path is never safe, so two of them
   always put two unsafe arms under one `or_i` in the right-leaning chain.
   Port this as a rule in `md.ValidatePathList` (`md/compose.go:449-476`) with
   its own error sentinel; the Rust primary is adding the SAME rule to
   md-codec's `validate()` in parallel (it will land as md-codec 0.45.0 with a
   vector — the controller will hand you the vector path; until then, your
   test asserts against the oracle by shelling out to `md compose` for the
   same lists, and against the literal lists above). Then the gui: the Done
   arm (`gui/composer_shape.go:~539`) maps the new error to a FIXED body in
   §8's voice — say that a second key-less path makes the script malleable
   and that a timelock does not help; give one of them a key or fold them —
   and `composerAddPath` (`:327`) may refuse a second key-less path at
   creation with the same body. Adopt `TestFableRedTwoKeylessPathsAreRefused`.
2. **L1 I-2 — the same KEY at two slots must be refused as unsupported,
   whatever its serialisation.** `composerDuplicateXpub`
   (`gui/composer_review.go:89-98`) compares xpub STRINGS; a re-serialised
   copy (different depth/parent-fp header, same 33-byte pubkey and chain code)
   passes and one signer spends the "2-of-3". Compare KEY MATERIAL (pubkey +
   chain code), like md-codec's `validate_no_duplicate_key_slots` (key + use-
   site; deliberately NOT the origin, because the same SEED at two accounts is
   allowed by operator ruling). Also determine whether the Go port's
   `md/duplicate_keys.go` runs on the mint path (`composerArtifactsFor`) and
   compares material; if the device can mint what `md encode` refuses (verify
   with `md encode` on the lens-1 descriptor), close that too. Refusal copy:
   the existing §7d body ("Two slots resolving to the same xpub → REFUSE") —
   reuse its exact phrasing. Adopt `TestFableRedSameKeyReserializedIsRefused`.
3. **L1 I-3 — the consent names k-of-n for a locked or hashed multi-key
   path**, not `N key(s), custom` (`composerConsentLinesFor`), and the self-
   check (`gui/composer_selfcheck.go`) compares k there too. Adopt
   `TestFableRedConsentNamesThresholdOfLockedMulti`.
4. **L1 I-1 + L3 I-1 (one finding) — a key-less path makes the whole wallet
   un-importable in Bitcoin Core, Nunchuk and Liana, keyed paths included.**
   Measured: Core v25 and v31.1 `getdescriptorinfo`/`importdescriptors` refuse
   ("witnesses without signature exist"); libnunchuk 2.1.1 refuses; Liana
   refuses any hashlock path. The §8a body (`gui/composer_copy.go:76-80`) and
   the consent's `KEY-LESS (EXPERIMENTAL)` restatement
   (`gui/composer_consent.go:91-96`, `:234-237`) must say so: only md-family
   tooling restores this wallet; nothing else will watch or spend it. Keep it
   to the modal.
5. **L2 I-1 — §8f's NUMS note is FALSE for Nunchuk.** libnunchuk 2.1.1
   refuses every NUMS-keyed `tr` the composer emits (7/7: `sortedmulti_a` is
   not a miniscript fragment for its validator, and its DISABLE_KEY_PATH form
   re-renders the key path as an xpub so the round-trip check fails). Rewrite
   the §8f body: Bitcoin Core imports this form; Nunchuk cannot import a NUMS
   policy at all; an operator who wants Nunchuk chooses wsh, or a tr policy
   whose first path is a single key. Do NOT point a Nunchuk user at the
   unspendable-xpub form — that is a DIFFERENT wallet with different
   addresses (measured).
6. **L2 I-2 — a `wsh` policy that mixes time-based and height-based locks
   across paths is refused by Nunchuk ("Timelock mixing"), imported by Core.**
   Add a notice at the review (in the style of the Liana line of §8g) when a
   wsh policy carries both bases; under `tr` the leaves validate separately
   and Nunchuk accepts.
7. **L4 I-1 — the key-order picker opens on the setting in force**
   (`gui/composer_shape.go:300-323`, `composerKeyOrderStep`): `Initial` from
   the current `Sorted`, and the §8b hold fires once per DECLINE, not on every
   pass; a hold-confirmed "Keep my order" survives Back + Done + forward.
   Adopt `TestFableSpecKeyOrderSurvivesBackFromTheStubScreen`.
8. **L4 I-2 — Back on the passphrase keyboard re-asks the question; Back on
   the question declines the seed (un-registers it).**
   (`gui/composer_sources.go:258-274`; the registry `st.reg.add` at 253 runs
   before the question — undo it on decline.) `gui/multisig_build.go:775-787`
   has the identical shape: fix it too if the fix is the same three lines,
   otherwise say so in the report. Adopt
   `TestFableSpecBackOnThePassphraseKeyboardIsADecline`.
9. **L3 I-3 — the device's restore document must not say "Addresses
   unavailable for this policy shape" for a shape the consent screen just
   derived.** `composerRestoreDoc` (`gui/composer_flow.go:517-538`) →
   `multisigRestoreLines` (`gui/multisig_restore.go:23-41`) knows only the flat
   `expandedToDescriptor` route; the consent uses `policyAddressAt`
   (`gui/wallet_policy.go:378`). Route the document through the same
   derivation, so 22 of 22 keyed shapes print Descriptor + first receive/change
   equal to the consent.
10. **Minors with RED tests (L4 M-1..M-4, L1 M-1):** the count pickers open on
    the value in force (`composerCountPick`, `composerKeysEdit`); the engrave-
    mode question is asked only when a SEATED slot is seed-derived (not
    `st.reg.count() > 0`); `composerSecretCards` dedups by master fingerprint
    so one seed typed twice is cut once and the census counts one; the date
    band compares Unix time to `composerDateFloorUnix` so 2009-01-01/02 get
    the floor body (`gui/composer_lock.go:327-334`). Adopt their tests.
11. **L4 M-5** — a key-less path edited to empty: remove it the way
    `composerAddPath` (377-384) does when created empty, or refuse with a body
    that names the actual state ("this path has no key and no hash"); do not
    leave the lock-only body on it. Keep
    `TestFableEmptiedKeylessPathIsRefusedWithTheLockOnlyBody` only if you
    choose the refusal and reword it.
12. **L4 M-6 / L1 M-2 / L3 M-2 (three lenses, one defect) — a `tpub` in a
    `key:` record is admitted as a mainnet key silently.** `sysw.ParseKeyRecord`
    (`sysw/composer_records.go:395-407`) checks depth and child, never the
    version bytes. §4f: complex-policy derivation is mainnet-only by
    construction. Refuse a record whose xpub version is not the device
    network's, classified as unsupported with a one-line reason ("testnet key
    in a mainnet policy"), so the door does not count it and seating never
    offers it. Adopt `TestFableTestnetXpubKeyRecord`, inverted. (The host's
    `me sysw pack` side is the controller's.)

Out of scope here (filed by the controller): the nits, L2 M-1/M-3, L3 M-1/M-3,
Multisig Build beyond item 8.

## Gate — run before writing the report, paste the numbers

`go vet ./...`; `gofmt -l .` == the five-file baseline; codec packages
`go test ./md/ ./sysw/ ./mk/ …` green; the WHOLE gui shard set green with the
count; every adopted test runs (-v) and every previously-RED one is now GREEN;
for each new FIXED body, the modal-fits assertion; firmware size:
`PATH=/nix/var/nix/profiles/default/bin:$PATH nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller`
(baseline at f5b068fa is what you measure first; report both).

## Report — your FINAL action

Write `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/composer-fable-r0-fold-fork-implementation.md`:
per finding — commit SHA, the RED output before, the GREEN after, the new copy
verbatim; the gate output; deviations from this brief with reasons; what you
could not do. Return only the verdict line, the branch tip SHA, and the path.
