# R0 — IMPLEMENTATION_PLAN_descriptor_input_S2, round 1

**Artifact:** `design/IMPLEMENTATION_PLAN_descriptor_input_S2.md` (223 lines,
`027629c`).
**Reviewed against:** `SPEC_descriptor_input.md` §5.1/§5.2/§5.4/§6/§8/§9/§11,
`design/agent-reports/RECON-S2-fork-seam.md`, `design/FOLLOWUPS.md` F-418,
F-422, F-423, F-424, F-426, F-427, F-428, F-430, and the shipped S1+S3 tree.
**Trees:** mnemonic-engrave `027629c` (plan baseline `4646fa2`), seedhammer
fork `a5e29b4`.

**THE ONE QUESTION:** is this plan buildable as written, and does building it
produce what §5.2's S2 surface requires?

**Counts: 4 Critical / 6 Important / 7 Minor / 3 Nit — verdict RED.**

---

## Method, and what is measured rather than argued

Every finding below was reproduced. The instruments:

- the shipped `me` binary at `027629c` (`cargo build --locked -p
  mnemonic-engrave`, already current);
- a Go probe module against the fork at `a5e29b4` (`replace seedhammer.com =>
  /scratch/code/shibboleth/seedhammer`, go 1.26.3 from the nix store, the
  user's default module cache), importing `nonstandard`, `sysw` and the `qr`
  package directly;
- the shared vector file, read as data.

Nothing in the repo or the fork was modified.

### Verified TRUE — do not re-derive these

| claim | verdict |
| --- | --- |
| P1.1: `admit(d, Path::Descriptor)` implements §5.2's predicate | **TRUE, and stronger than the plan says** — `descriptor::host_admits` (`crates/me-cli/src/descriptor/admit.rs:418`) already IS the predicate verbatim, is public, and is asserted row-by-row against all 71 vectors at `crates/me-cli/tests/descriptor_seam.rs:584`. `Path::Descriptor` is the seven-forms-strict set: `conjunct_1_multi_under_descriptor` (`admit.rs:85-91`) refuses `multi` and runs LAST, after conjuncts 2–8. The md1 widening (`admit.rs:110-112`) is structural only. |
| §5.1's text supports retiring `Row::WindowNotInBuild` post-S2 | **TRUE.** No sentence keeps a window refusal reachable: §5.1's window is conditioned on "a build where the `--as descriptor` path has not shipped", and §5.4's neither-path clause names the window's variant 2 only for "the Specter `/0/*` file **in the S3-only window**". Both producers die: `as_flag.rs:136` (the `descriptor_follower` stub) and `gate.rs:241` (reachable only when `MD1_PATH_SHIPPED == false`). See I3 for what that forces. |
| P0.2's two baseline checks | **BOTH PASS.** `sha256sum` of both vector copies = `542cd492e35149b62c53f940fb755576e0ffd4d086b0e3fcda615fbc43f51974`; `wantSyswClass = 4` (`nonstandard/descriptor_seam_test.go:72`) and the file carries exactly 4 `sysw_class` rows. |
| P0's gate commands resolve | `1.85.0-x86_64-unknown-linux-gnu` is installed; `scripts/gui-shard-test.sh`, `plan-staleness-check.sh`, `plan-cite-check.sh` exist. `go` is NOT on `PATH` (only in the nix store) — the plan already says "and a Go toolchain", so this is a setup note, not a finding. |
| P3.2's sim-walk is available headless | **TRUE.** `runUI`/`pumpUntil`/`sessionWith(records...)` drive a real session to a rendered screen with no hardware — `gui/transaction_walk_test.go:28-50` is the exact template, and its own comment records `runUITouch`/`runUI` in use across 39 test files. P3's "the sim-walk renders" gate is satisfiable. |
| P1 must precede P2 | **TRUE and load-bearing.** `as_flag::Decision::Pack(strings)` feeds `recs` (`main.rs:1444-1446`), which reach `pack_with` → `split` → `admit_check`. Without P1's arm the canonical descriptor is `Class::Unknown` and P2's own pack exits 4. The plan's ordering is right. |
| no container or record length limit binds a canonical descriptor | **TRUE — a negative worth having.** The `MaxRecordLen = 512` in `seal/wire.go:58` belongs to the **seal** container (`seal/container.go:78`), a different path; the **sysw** container has its own opener (`sysw/open.go:67-74`, `splitRecords`) with no per-record bound, and only `MaxSectionLen = 32734` / `RegionLen = 65536`. The largest host-admitted canonical in the vector file is 2320 bytes (`accepted/sh-wsh-sortedmulti-16-keys`) and `qr.Encode(desc.EncodeNoChecksum(), qr.L)` on it returns **no error** (measured; 2311 bytes, inside version-40-L's 2953). Rust `MAX_RECORD_LEN` (`crates/me-cli/src/seal/container.rs:11`) is likewise the seal path only — measured: `me sysw pack` accepts an 805-byte `text:` record at exit 0. |
| every on-device action is in P5.4 | **TRUE.** No phase gate requires a flash or a screen. P0/P1/P2 are host suites, P3 is `go test` + the headless walk, P4 explicitly forbids a physical cut. (One omission, M3.) |
| §9 item 3 (change addresses, testnet) | correctly untouched — S2 packs a string and derives no new addresses; the gap stays with `--as md1`. |
| F-424 (parked `md-codec` publish) | correctly out of scope; owning phase is the operator-gated publish. |
| F-422 | the plan does not smuggle the `/0/*` transform in: P2.1 packs the canonical re-encode and never rewrites a use-site path. |
| F-423, F-426, F-427, F-428 | present with owners (P4, P3.4, — , invariant 1). F-427 is doc-only and correctly unscheduled. |

---

## CRITICAL

### C1 — P1.1's classifier arm silently DISABLES §5.1's gate on the `--as`-omitted path

`me sysw pack` consults the descriptor gate in exactly one place, and only when
record classification has already FAILED:

```
main.rs:1504    if let Err(e) = sysw::admit_check(&recs, admission) {
main.rs:1516        let outcome = mnemonic_engrave::descriptor::consult(&document, &recs);
```

and `admit_check` (`crates/me-cli/src/sysw/mod.rs:403-410`) errs on exactly one
condition:

```rust
if matches!(classify_with(r, adm), record::Class::Unknown) {
    return Err(SyswError::Unclassifiable(i, unknown_reason(r)));
}
```

The moment `classify` gains a Descriptor arm, a single-line admitted descriptor
classifies successfully, `admit_check` returns `Ok`, and the **entire block is
skipped** — §5.4's identification block, §5.1's choice block, and every
`--as`-omitted §6 refusal with it. The run falls through to the pack.

**Measured, today:**

```
$ me sysw pack --in <formats-happy/bip380-sortedmulti-multipath> --out out.bin
rc=2
me: read as: a plain BIP-380 descriptor
      descriptor: wsh(sortedmulti(2,…))#ud8uyjz3
      wallet-id: 9e95257e60aacbb260129dac7b36d9f4
      address 0: bc1q4taqq6q6l8fvguva6ftvrz3qgdjy6p3w2s0ds0nl6qrjw7t0hfhqgrqcwd
      …
me: this input is a wallet descriptor, and `--as` decides how it is packed.
```

Post-P1.1 that same invocation exits **0** and writes a container whose record
is the operator's RAW bytes — no canonical re-encode, no checksum recompute, no
`address 0:` to compare, no choice made.

**What it breaks, by name:**

- §5.1, NORMATIVE: *"It is **required whenever the input is a descriptor** and
  there is no default value. Omitting it is a **usage** error."*
- §5.1's gate invariant 2: *"Every admitted descriptor spelling reaches the
  descriptor surfaces."*
- §5.2: *"The record is the canonical form, not the operator's bytes."*
- §11 item 5 case 1 — the shipped test `item_5_the_five_case_matrix`
  (`crates/me-cli/tests/descriptor_refusals.rs:829-849`) uses this exact vector
  input with `flags: vec![]` and asserts `exit: 2`. It goes red.
- the promoted-key path is worse: a bare `xpub…` record (vector
  `promotion/01-bare-xpub`) would pack VERBATIM while §5.2 requires
  `pkh(xpub…)#77…` — the operator engraves a `pkh` wallet the device inferred
  and they were never shown.

The plan's invariant 2 explicitly scopes itself to *non-descriptor* records
("an input that classified as Mnemonic/Codex32/MdMk/Mt/FreeText before S2
classifies identically after"), so it cannot catch this by construction, and no
phase mentions the interaction. Arm ORDER does not help: order protects records
that already matched an earlier arm; it says nothing about records that
previously fell through to `Unknown`, which is precisely this class.

### C2 — P1.2's named test is unsatisfiable under any implementation of §5.2's predicate

P1.2: *"the 4 `sysw_class: Descriptor` rows classify `Descriptor`; every OTHER
vector row (67) does NOT."*

§5.2's predicate is `descriptor::host_admits`, and it is TRUE on **19** of the
71 rows — the file says so, and `descriptor_seam.rs:584` asserts it green:

```
formats-happy/bluewallet-sh-fixture   formats-happy/bip380-sortedmulti-multipath
formats-happy/json-label-descriptor   promotion/01-bare-xpub
promotion/02-bare-zpub                promotion/05-origin-44h
promotion/06-origin-49h               promotion/07-origin-84h-zpub
promotion/13-children-no-origin       promotion/14-bare-xpub-trailing-newline
accepted/sh-wsh-sortedmulti-16-keys   whitespace/crlf-bip380
whitespace/leading-space-bip380       md1-split/fixed-index
md1-split/multipath-no-wildcard       md1-split/childless
md1-split/mixed-fixed-and-multipath   md1-split/mixed-nowildcard-and-multipath
md1-split/mixed-childless-and-multipath
```

Only 4 carry `sysw_class`. The other 15 are ordinary admitted descriptors —
`md1-split/fixed-index` is `wsh(sortedmulti(2,…/0/*))`, `promotion/05-origin-44h`
is a promotable origin-annotated key. A classifier implementing the predicate
answers `Descriptor` on all 19. **15 counterexamples to "every OTHER row does
NOT."**

The escape hatch fails too. Narrowing the arm to require the record to equal its
own canonical re-encode would satisfy the second clause and break the first: 3 of
the 4 `sysw_class` rows' inputs are not their canonicals — a multi-line
BlueWallet file, a pretty-printed JSON blob, and a bare `xpub…` whose canonical
is `pkh(xpub…)#77…`. Measured. **No implementation satisfies both clauses**, so
P1.2 is a gate that cannot be met, and an implementer resolving it by narrowing
the classifier ships a predicate the spec does not describe.

The `sysw_class` column is not the classifier's population; it is a
deliberately small sample of it (one row per §4 format). The plan reads it as an
exhaustive answer.

### C3 — P3.1's Go arm implements a predicate 17 rows WIDER than the Rust primary, while the plan asserts parity

P1.1 says the predicate is *"implemented ONCE"*; P3.1 says the Go arm calls
`nonstandard.OutputDescriptor`. Those are two different predicates.
`OutputDescriptor` is the scan-door parser — §4's four formats with **none** of
§4.7's conjuncts 2–8.

Measured against the shared vector file, restricted to rows whose input is a
single line (i.e. can be a record) and which the HOST refuses:

```
promotion/15-bare-tpub-host-refused      accepted=true
narrowed/tr-sortedmulti                  accepted=true
narrowed/wpkh-sortedmulti                accepted=true
narrowed/pkh-sortedmulti                 accepted=true
narrowed/sh-wpkh-sortedmulti             accepted=true
narrowed/wsh-of-key                      accepted=true
narrowed/sh-of-key                       accepted=true
narrowed/threshold-zero                  accepted=true
narrowed/threshold-negative              accepted=true
narrowed/threshold-exceeds-keys          accepted=true
narrowed/sh-sortedmulti-16-keys          accepted=true
narrowed/wsh-sortedmulti-21-keys         accepted=true
narrowed/mixed-network                   accepted=true
narrowed/use-site-hardened               accepted=true
narrowed/use-site-non-consecutive        accepted=true
gate/colliding-origin-sortedmulti        accepted=true
gate/duplicate-key-same-use-site         accepted=true
TOTAL: 17
```

Every one of those becomes `ClassDescriptor` on the device under P3.1's arm, and
`gui/sysw_admit.go:37,39,45` admits `ClassDescriptor` to `progBundle`,
`progMultisig` and `progWalletPolicy` — so `wsh(sortedmulti(0,K1,K2))`
(anyone-can-spend, §6: *"treat them as at risk now"*), `k > n` (unspendable),
21 keys (unspendable), mixed-network (no derivable address), a hardened
use-site, and both conjunct-8 key-identity failures all reach a program and a
screen. §4.7's conjuncts are the host's alone today; P3.1 as written does not
port them, and the device is where the plate is cut.

**The plan's own test cannot see this.** P3.3 asserts the 4 `sysw_class` rows
and that "every `device_admits: false` row does not" — all 17 divergent rows are
`device_admits: **true**`. It is a test that passes in both worlds.

§5.2 contains both sentences (the predicate, and "an arm that calls
`nonstandard.OutputDescriptor`") and they contradict each other; the plan
inherits the contradiction instead of ruling it. The fold must either narrow the
Go arm to §4.7 (a real port, larger than P3.1 describes) or amend §5.2 and
accept a stated, tested asymmetry with the device refusing later — but it cannot
claim parity and ship `OutputDescriptor`.

### C4 — §11 item 1's `me sysw show` surface does not exist, and no phase builds it

§11 item 1 and P2.3 both require *"a container whose `me sysw show` reports ONE
`Descriptor` record"*.

`me sysw show` (`crates/me-cli/src/main.rs:1803-1845`) prints `sealed:`,
`pub_len:`, `ct_len:`, `identity:`, `digest:`, then `print_mdmk_confirmation`.
That function emits a per-record line **only** for `Class::MdMk`
(`main.rs:2062-2071`) and delegates to `print_mt_confirmation`, which emits only
for `Class::Mt` (`main.rs:2082-2085`). `class_name` — which does have a
`C::Descriptor => "descriptor"` arm (`main.rs:2254`) — is called from exactly
one site, `main.rs:2197`, and that builds the *secret* list for the `sealing:`
line at pack time. A `Descriptor` record is not secret and never appears there.

**Measured:** a container holding one non-md1/mk1/mt1 record prints no
per-record line at all —

```
$ me sysw show t.bin
sealed:   false
pub_len:  15
ct_len:   0
identity: 25f4809e4cad5c8d5abbc973bf0033f3c90be9677f80c2b650abbea0a58e3cef
digest:   6d2d 5cee 6053 b401 84b8 dda2 59b0 b0e1
rc=0
```

So P2.3's test cannot be written as stated, and the plan's opening paragraph
("§11 items 1 and 4's `--as descriptor` rows close at the desk") is false for
item 1. Either P2 grows a `show` record-listing task — an operator-facing
surface change with its own text, ordering and secrecy questions — or §11 item 1
is amended. The plan does neither, and the omission is invisible until an
implementer sits down to write the assertion.

---

## IMPORTANT

### I1 — P1.3's `--expect` ruling leaves `--as descriptor --expect descriptor` a guaranteed false refusal

P1.3: *"The `--expect` VOCABULARY does not change in S2 (nothing in §11 requires
it)."*

`Kind::Descriptor` does not resolve through `Class`. It resolves by card HRP:

```rust
// crates/me-cli/src/sysw/expect.rs:112
Kind::Descriptor => card_hrp(record) == Some('d'),
```

and its operator-facing description is *"an md1 descriptor card"*
(`expect.rs:96`). A `--as descriptor` pack produces one `Class::Descriptor`
record — a BIP-380 string, not a bech32 card — so `card_hrp` is `None`,
`--expect descriptor` is UNMET, and `main.rs:1483-1489` exits 4 having refused
the artifact the same invocation was asked to build. `--expect` runs on the
substituted `recs` (`main.rs:1477`), so the `--as` path is squarely in its
scope.

The two flags spell the word identically. `me sysw pack --as descriptor --expect
descriptor` is a natural belt-and-braces invocation and it is a hard,
100%-reproducible false refusal on the funds path — the exact shape
`expect.rs:35-41` documents as the reason `Admission` is a parameter: *"A false
refusal carrying a false message, on the funds path, inside the feature added to
prevent exactly that."* The module doc's own justification for excluding
`address` (*"`Class::Address` and `Class::Descriptor` are never produced by
`classify` … a kind that can never be satisfied is worse than an absent one"*)
becomes half-false the moment P1.1 lands, which P1.3 notices — and then rules
the wrong way. The decision needs to be made, not deferred: either
`Kind::Descriptor` becomes `card_hrp=='d' || Class::Descriptor`, or a new kind
is added, or `--as descriptor` refuses `--expect descriptor` with a message that
explains. Silence ships the refusal.

### I2 — the P0.1 flip inventory's known-members list misses three of the four behavioural `DESCRIPTOR_PATH_SHIPPED` consumers

Measured, whole-repo (`grep -rn 'DESCRIPTOR_PATH_SHIPPED' crates/`):

| site | what it does | in P0.1's list? |
| --- | --- | --- |
| `descriptor/gate.rs:42` | the declaration | implicitly (P2.1 flips it) |
| `descriptor/gate.rs:223` | `carriage()` — `descriptor_carries` | **NO** |
| `descriptor/gate.rs:273` | `window_remedy()` — §5.3's window SUBSTITUTION | **NO** (see below) |
| `descriptor/gate.rs:566` | `choice_block()`'s `descriptor_head` | as "the choice block's marking" |
| `main.rs:365` | the clap help conditional | yes (`main.rs:360-373`) |
| `descriptor/as_flag.rs:133` | a COMMENT mentioning it | cited as a consumer — it is not one |

The plan states the consumers as *"`crates/me-cli/src/main.rs:360-373`,
`crates/me-cli/src/descriptor/as_flag.rs:126-138`"*. That is one real consumer
and one non-consumer.

Two consequences the inventory therefore does not carry:

1. **`gate.rs:223` flips an EXIT CODE, not a string.** `descriptor_carries`
   becomes true for every admitted, md1-unrepresentable input, so the four
   `md1-split/*` rows move from `EXIT_REFUSED` (3) + §5.3's refusal to
   `EXIT_USAGE` (2) + the choice block. That is §11 item 5's case 3 (see I4) and
   §6's `md1-fixed-index` / `md1-no-wildcard` rows' reachability.
2. **`gate.rs:273` is a DIFFERENT "two variants" from the one the plan names.**
   The plan's member *"the window refusal's two substituted variants and their
   tests"* is §5.1's window refusal (`identify::window_refusal` — the
   md1-representable vs (a)/(a″)-shaped alternatives). `gate.rs:273` is §5.3's
   window substitution, which flips the REMEDY SENTENCE inside two §6 rows from
   *"The scannable-plate path is not in this build…"* to *"Use `--as
   descriptor`, which carries `/0/*` exactly."* Those two row tests are pinned
   verbatim (`descriptor_refusals.rs:11-16` names them as the window-substituted
   class). Two collections of "two variants", one named, one not — exactly the
   shape the S1+S3 cycle's fold-introduced Important came from.

### I3 — retiring `Row::WindowNotInBuild` forces §6 and §11 spec amendments no phase owns; and P2.4's premise is false

**The forced chain, measured.** `Row::ALL` has **36** entries including
`Row::WindowNotInBuild` (`descriptor/refusal.rs:74-111`), and
`descriptor_refusals.rs:126-141` asserts BOTH `named_row_tests().len() == 36`
AND set-equality with `Row::ALL`'s slugs. Post-S2 the row has no reachable
producer (verified above). §11 item 4 requires *"a test that **reaches** it"*, so
its row test cannot survive; removing the row makes the code's vocabulary 35
while §6's table still lists 36 rows, and set-equality then fails from the other
side. §11 item 5's sibling clause — *"`--as descriptor` in a build where its
path has not shipped exits 3 … BOTH alternative variants tested"* — becomes
untestable at the same moment.

So S2 necessarily amends §6's table and §11 item 5's sibling. The plan's only
scheduled spec touch is P4.2's §5.5 plate cell. P0.1 raises the question
("the spec text governs; cite §5.1's exact sentence in the inventory") and then
no task owns the edit, and P5.1's records reconciliation lists FOLLOWUPS,
CHANGELOG, continuity and memory — not the spec. S2 also falsifies §8's "S2 is
parked until the device is back on the bench", §5.5's *"needs a firmware change
to be readable | **yes, §5.2**"* row, and §9 item 2's "untested by
construction". A diff falsifies text it never touches; nothing here is scheduled
to catch it.

**P2.4's premise is false.** P2.4 instructs: *"§11 item 4's `--as
descriptor`-only §6 rows get their named tests (the S2 set that S1 recorded as
EMPTY-because-parked; enumerate from the P0 inventory)."* The record says the
opposite. `descriptor_refusals.rs:4-5`:

> *"All 36 rows, and the S2-parked set is EMPTY — every §6 trigger is reachable
> in this build."*

The set is empty because **every** §6 row is already reachable and tested, not
because rows were deferred to S2. There are no `--as descriptor`-only §6 rows to
add; S2 subtracts one. An implementer following P2.4 has nothing to enumerate
and a plausible incentive to invent rows.

### I4 — §11 item 5's "admitted but UNCARRIED" case loses its only witness, and invariant 1 forbids the replacement

Case 3 of `item_5_the_five_case_matrix` (`descriptor_refusals.rs:855-862`) uses
`vector_input("md1-split/fixed-index")` and asserts exit 3 + *"md1 cannot carry
this wallet as written"*. Post-S2 `--as descriptor` carries `/0/*` exactly, so
that input becomes CARRIED and flips to the choice block at exit 2. The case
does not change value — it **disappears**.

§5.4 names the full-build witness the case still needs: an admitted wallet
carried by nothing, *"`wsh(multi(…/0/*))` in every build"*. Measured: **no such
row exists in the vector file.** The only `multi` rows are `neither/wsh-multi`
(`md1_admits: true`, so md1 carries it) and `gate/colliding-origin-multi`
(inadmissible at conjunct 8, so it is a refusal, not an uncarried admission).

So restoring §11 item 5's five-case matrix requires a NEW input. Sourcing it
from the vector file changes the file's bytes, which invariant 1 says S2 expects
not to do — and which, by invariant 1's own clause, would drag F-428's two-repo
citation fix and a sha bump into the same commit. P2.2 says only that the matrix
"updates to the full-build truth table (both `--as` values carry → the
omitted-`--as` choice block still exits 2)", which describes case 1 and is silent
on the case that vanishes.

### I5 — P3.2 names no consuming program or entry point, and P5.4 over-claims §9 item 2

§9 item 2 is **three** admission cells. The recon (§3, Layer A) resolves them:
`gui/sysw_admit.go:37,39,45` admit `ClassDescriptor` to `progBundle`,
`progMultisig` and `progWalletPolicy`, and *"no `gui/*.go` file calls
`ctx.sysw.take(sysw.ClassDescriptor)` or `takeAll(...)` anywhere."*

P3.2 says the session *"routes `ClassDescriptor` per the `ClassMt` … checklist"*
and names one cell (`admits(progWalletPolicy, ClassDescriptor)`). It never says
which program's flow gains the consumer, where in that flow, or what the other
two cells become. The three candidates mean materially different things —
`progBundle` is the multi-plate bundle, `progMultisig` supplies a policy for a
multisig backup, `progWalletPolicy` is the policy-card program whose first card
is `ClassMDMK` — and the existing screen, `DescriptorScreen`/`descriptorFlow`
(`gui/gui.go:2727-2741, 3070-3189`), is today reached only from the NFC scan
door and goes straight to a plate. An implementer must invent the routing, and
§11 item 6's on-device acceptance depends on which menu item the operator is
told to press.

Then P5.4 hands the operator *"the §9 item 2 **cells** confirmed on hardware"*.
With one consumer built, two of the three cells remain unexecutable and the
handover names a check the operator cannot perform. §11 item 6 (one record
displayed once) is satisfiable; §9 item 2's discharge, as the plan claims it, is
not.

### I6 — routing every record through `nonstandard.OutputDescriptor` opens two crash paths, unaddressed

`sysw.Classify` is called on **every record of every loaded payload**
(`gui/sysw_session.go:109-115`, the recon's Q2 — one non-test call site). P3.1
puts `nonstandard.OutputDescriptor` on that path. Measured, on single-LINE
inputs (i.e. things that can be records):

```
PANIC  "ab: xpub6C9j4wAxx…"       runtime error: index out of range [3] with length 1
PANIC  "abcdef: xpub6C9j4wAxx…"   runtime error: index out of range [3] with length 3
ok     "Name: my wallet"          accepted=true, then Encode() PANICS: "unknown script"
```

The parse panic is `nonstandard/parse.go`'s `binary.BigEndian.Uint32(fp)` after
only `len(fp) > 4` is checked — §4.2 defect 4, and the vector file already
carries it as `bluewallet/short-fingerprint`, `device_probe: "panic:parse"`. The
second is §4.2's `panic:encode` class: `Name: my wallet` parses to a titled
zero-key descriptor that `OutputDescriptor` RETURNS (its `bw.Title != ""` guard
passes), so post-P3.1 it classifies as `ClassDescriptor` on the device and then
panics when the screen encodes it.

§7's schema goes to some length to keep the *test* from doing this — *"the Go
test asserts the row is host-refused and must NOT feed the input to
`nonstandard.OutputDescriptor`"* — while P3.1 makes production do exactly that
on every record. The plan mentions neither site.

Severity bounded, and stated so honestly: the same panics are already reachable
via `gui/scan.go:87` and `seal/record.go:206`, so S2 widens exposure rather than
creating it. But it moves it onto the payload-load path, where a crash is a
backup that will not open, and the containment decision (recover, pre-filter, or
port §4.7 per C3 — which also fixes this, since the host cascade refuses both
inputs) belongs in the plan.

---

## MINOR

**M1 — the recon checklist is mis-cited.** The plan says *"the `ClassMt`
five-touch-point checklist (recon Q3; 3 of 5 exist)"*. The recon lists **six**
numbered touch points; three exist for Descriptor (constant `sysw/record.go:32`,
admission table `gui/sysw_admit.go:37,39,45`, `txClassName` arm
`gui/transaction.go:286-287`), and the sixth — registering the new consumer in
`gui/sysw_admit_oracle_test.go`'s `syswConsumers` table — is off the plan's
count. Self-enforcing, hence Minor:
`TestEverySyswConsumptionSiteNamesAnAdmittedClass` walks every non-test
`syswOffer`/`take` site by AST and fails until it is registered
(`gui/sysw_admit_oracle_test.go:23-26`). `DescriptorScreen`, which the plan
counts as one of its "3 of 5", is not on the checklist at all.

**M2 — the choice block's shipped branch renders misaligned, and nothing
catches it.** `gate.rs:566-570` returns `"      --as descriptor"` unpadded, and
the format string puts `\n` after it, while `md1_head` is padded
(`"      --as md1          "`) with its description on the SAME line. §5.1's
NORMATIVE block has the descriptor description inline —
`      --as descriptor   the SCANNABLE plate. The device engraves the wallet`.
So P2.2's "loses `(not available in this build)`" is not sufficient: the shipped
branch also needs the padding and the `\n` removed. No test asserts the block
verbatim (`grep SCANNABLE crates/me-cli/tests/` = 0 hits), so the mismatch would
ship silently.

**M3 — P3's gate omits the TinyGo device build §9 item 5 names as unchecked.**
`.github/workflows/test.yml:120-135` runs
`tinygo build -size full -print-stacks -o /dev/null -target pico-plus2
-stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` on every
push — a command that runs locally under `nix develop`. P3's gate is `go test
./...` + shard + vet + gofmt. Fork CI would catch a TinyGo break at P5.3's push,
one phase after the port is declared green, on the branch the device boots. Risk
is genuinely low (`nonstandard` is already in the image via `gui/scan.go`, and
there is no import cycle — `nonstandard` does not import `sysw`), but §9 item 5
names it and the plan's own doctrine is that an unrun gate is a hypothesis.

**M4 — P4.1 names an analytic measurement the code does not offer.** "From
`engrave.Params` and the shipped font metrics, compute how many md1 strings fit"
— the fork's fit mechanism is a **trial** fit: `backup.EngraveText(params,
plate)` then `toPlate(plan, params)`, which returns an error when it does not
fit; `validateDescriptor` (`gui/gui.go:722-736`) uses exactly that loop. Two
facts that shrink the measurement: there is one plate size only
(`backup/backup.go:77`, `plateSize = 85` mm square), and the font size is fixed
(`fontMM()` → `plateFontSizeUR` unless a caller sets `Text.FontSize`,
`backup/backup.go:58-63`). F-423's 2-stroke-width minimum-feature rule therefore
binds only if the implementer reduces `FontSize` — which the plan neither
permits nor forbids, and which P4's gate ("fork suites; the arithmetic pinned by
tests; NO physical cut") could not detect.

**M5 — P4.2 does not state the packing boundary.** `bundlePlate`
(`gui/bundle_flow.go:371-379`) carries card-scoped fields — `cardIdx`,
`cardTotal`, `label`, `kind` — that drive the "Card X of Y | Plate P of Q"
transcription guidance, the abort warning, and `bundlePlateMark`. Packing
greedily WITHIN a card leaves all of them well-defined; packing ACROSS cards
makes every one of them ill-defined and would let a `cardMS1` string share a
plate with a marked md1 string, against `bundlePlateMark`'s own rule that a
cardMS1 plate is never marked. The within-card reading is the natural minimal
change and is what F-423's motivating case needs (a keyed single-sig card is 2
strings), so the risk is low — but the plan should say it. (The ms1-marking half
of this is capped by the 2026-08-27 secret-handling ruling and does not gate;
the operator-guidance half is a plain correctness question.)

**M6 — P0's clippy gate is the fix F-430 says not to make.** The gate adopts
"clippy on BOTH nightly and the CI-pinned **1.85.0** — the F-430 lesson is a
plan gate now". F-430's entry: *"the durable fix removes the conflict (pin the
toolchain locally or run the pinned lint in the gate), not a note to remember
harder."* A plan clause that a human must remember on every fold is the note.
P5.1's FOLLOWUPS reconciliation does not mention F-430 at all, so it stays open
with the manual workaround baked in. (`1.85.0-x86_64-unknown-linux-gnu` is
installed, so the gate does run.)

**M7 — P3.3 does not say which STRING the `sysw_class` column is asserted
over,** and the two readings disagree. Over `canonical`: the plan's second
clause ("every `device_admits: false` row does not") has three measured
counterexamples — `promotion/14-bare-xpub-trailing-newline`,
`whitespace/crlf-bip380`, `whitespace/leading-space-bip380` are all
`device_admits: false` and all have canonicals `OutputDescriptor` ACCEPTS
(measured; they are §7's three deliberate host-wider rows, `wantHostWider = 3`).
Over `input`: three of the four `sysw_class` rows assert a class for a
multi-line string that can never be a record — `sysw/open.go:67-74` splits the
section on `\n`. The `DeviceAdmits *bool` field
(`nonstandard/descriptor_seam_test.go:49`) does at least keep the `panic:parse`
row out of the loop, so the suite will not crash.

---

## NIT

**N1** — P2.3 calls `classify(packed) == Descriptor` "the host-side fixed
point". It is neither a round trip nor a fixed point. §7 requirement 4's fixed
point is `encode(parse(canonical)) == canonical`, which P2.3 does not name and
which is the property that actually protects the seam.

**N2** — P2.1 says of §5.4's identification block "verify, don't assume, and
cite the call site in the report". Verified here so the implementer need not:
`as_flag::run` builds `notes = vec![identify::block(&d, Some(form))]`
(`crates/me-cli/src/descriptor/as_flag.rs:79`) BEFORE the `match form` that
selects the follower, so the block is already path-independent and no work is
required.

**N3** — §5.3(b)'s label warning fires on any `Decision::Pack`
(`as_flag.rs:88-94`), so §11 item 1's JSON exemplar will newly print it on the
descriptor path. That is correct (§5.5: "carries a label | text only, dropped")
but it is a new operator-visible line on a new path, and neither the plan nor a
test names it.

---

## What a fold has to decide, not just fix

Three of the four Criticals are the same shape: the plan treats `classify` as a
private helper for the `--as descriptor` path, when it is the shared admission
predicate for the whole `me sysw pack` surface (C1), the whole vector file (C2),
and the device's payload loader (C3, I6). The fold's first decision is where the
Descriptor arm may be consulted from — and it should be made before P1 is
written, because P1.2's test, P2.2's flip list and P3.3's assertions all inherit
it.

C4 is independent and cheap to rule: either §11 item 1 gets a `me sysw show`
record listing (new surface, needs its own text) or the item is amended to name
a surface that exists.
