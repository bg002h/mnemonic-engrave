# R0 review: IMPLEMENTATION_PLAN_f449_stage4a_me.md (engrave `bebb532a`)

**Verdict: NOT GREEN. 0 Critical, 1 Important, 4 Minor, 1 Nit.**

The code half of the plan is sound. I re-ran it and it holds on all five lenses.
The one blocking defect is in Task 5 Step 0. That step is the plan's only guard
against its own named Review Focus #5 ("`me` calls a v8 card confirmed on a
device that cannot read it"), and **both of its checks pass on today's fork,
where stage 3 has not landed.**

Reviewer: independent R0 (opus). Scratch copies were
`/scratch/code/shibboleth/.tmp/r0-4a` (the author's patch applied to
`8aea0d36`), `r0-4a-f3` (a pristine tree with only the two Cargo.toml edits),
`r0-4a-fork` (fork `origin/main` `7b6f2fb`) and `r0-4a-walk` (journey fixtures).
Target dirs were under `.tmp/`. The engrave checkout was not modified.

---

## Important

### I-1: Task 5 Step 0's precondition cannot fail, so it does not block the tag it exists to block

**Where:** plan line 904, Task 5 Step 0.

**Counterexample (RUN, today, fork `origin/main` = `7b6f2fb`, where stage 3 has NOT landed):**

```
$ git -C seedhammer log --oneline origin/main | grep -i 'stage 3'
40318b8 md,address: emit segwit-v0 witness scripts (Stage 3)
338e8c8 address,md,gui: derive taproot SCRIPT-PATH addresses (Stage 3)
$ go test ./md/ -run 'Version8|WireVersion'
ok  	seedhammer.com/md	0.003s [no tests to run]       # exit 0
```

The first check matches two commits from an **unrelated earlier cycle's**
"Stage 3". The second exits 0 with an `ok` line because no test name matches:
the "a filtered test run that matches nothing says ok" trap. Both halves are
satisfied by the status quo. An implementer who reads "`grep` returns the
stage-3 merge **and** `go test` shows ok" tags `v0.11.0` and installs it
locally. The plan itself names what follows (line 904, "Cost if wrong"): `me
sysw show` prints `confirmed` for a v8 card (I reproduced this, see the walk
below) while the flashed device treats that card as a SECRET and replaces its
legend.

The project rules make this blocking: "a gate that cannot fail" is a defect in
what a tool claims to have done, and the plan's own Review Focus #5 says Step 0
is the **only** cover for this risk ("This is what no test covers").

**Remedy.** Replace both checks with a behavioural check that fails today.
This one is measured and **FAILS at `7b6f2fb`** with `single v8: md: wire
version mismatch`:

```go
// in a scratch copy of the fork at origin/main: md/zz_v8_precondition_test.go
package md
import "testing"
func TestPreconditionForkReadsV8(t *testing.T) {
	if _, err := Decode("md1cpfdsssj6tvyywtsqrq0zjs4n7gdve74ar402"); err != nil {
		t.Fatalf("single v8: %v", err)
	}
	c := []string{ /* a real v8 chunked set, e.g. the 4 strings below */ }
	if _, err := Reassemble(c); err != nil {
		t.Fatalf("chunked v8: %v", err)
	}
}
```

Run it as `go test -v -run '^TestPreconditionForkReadsV8$' ./md/`, and require
the literal `--- PASS: TestPreconditionForkReadsV8` together with the absence of
`[no tests to run]`. Drop the commit-subject grep, or replace it with the
stage-3 merge **SHA** recorded in stage 3's continuity file together with
`git merge-base --is-ancestor <sha> origin/main`. The chunked v8 set I used is
`md-cli 0.18.0`:
`md encode "tr(UNSPENDABLE(liana),{pk(@0/48'/0'/0'/3'/<0;1>/*),pk(@1/48'/0'/1'/3'/<0;1>/*)})" --key @0=xpub6Bos… --key @1=xpub6CUG… --fingerprint @0=aabbccdd --fingerprint @1=11223344`
(4 chunks, `chunk-set-id: 0xf8817`, saved at `.tmp/r0-4a-walk/v8set.txt`).

---

## Minor

### M-1: Even after Step 0 passes honestly, "confirmed" is still false on every board that has not been reflashed (documentation only)

Step 0 checks **fork main**. The two boards run `bgf5b068f` until someone
reflashes them, and `me` cannot know that (the plan says so). Walk result
(below): the probe build's `me sysw show` prints `public record 0..3: md1/mk1 —
confirmed`, and `me bundle` prints `plate 1/4 md1 policy → push via NFC &
engrave`, for a v8 set that a `bgf5b068f` board refuses. The device fails
closed: the string is still engraveable, and no funds or data are lost. So this
does not block. **Remedy:** add a line to the `[0.11.0]` CHANGELOG entry and
the continuity file: "a version-8 md1 is confirmed only against firmware
containing F-449 stage 3; reflash before engraving one". Optionally, and cheaply
(`me` has already decoded the card, so it knows the version), append that same
note to the `show`/`pack` line for a confirmed v8 md1. That would be a warning,
not a refusal.

### M-2: The merged master carries the v8-confirming `me` before Step 0 can pass, and a standing permission rebuilds from master

Task 4 Step 7 merges to master. Task 5 Step 0 then forbids the tag and the local
install *within this plan*. But the standing "keep local binaries current"
permission (memory index) lets a later session `cargo install` from master, and
that session never reads Step 0. **Remedy:** when Step 0 blocks, write the hold
into the continuity file (and the memory entry) as "do not install `me` from
master until stage 3 is in fork main **and flashed**". Otherwise the guard
lives only in this plan's text.

### M-3: The walk's doc comment now describes the wrong function

At the probe, `sysw/record.rs:179-209` still sits on `mdmk_unconfirmed`, which is
now a 4-line projection. The comment describes the grouping walk (whose body is
now `mdmk_unconfirmed_why`, `:263`). It claims `report_unconfirmed`/`show` "call
**this**", when they now call `_why` (`main.rs:2111`, `:2305`). And it says
"no new variant, no changed shape", which is false of the new `Vec<(usize,
Unconfirmed)>`. `sysw/expect.rs:55-61`'s module doc still names
`mdmk_unconfirmed` as walk 2, and the plan's Step 5 changes `check` to `_why`.
This is the "comments outlive their conditions" class, and the plan fixes the
R2/R6 comment but not these. **Remedy:** move the long doc onto
`mdmk_unconfirmed_why`, give `mdmk_unconfirmed` a one-line "projection of
[`mdmk_unconfirmed_why`]; kept for the frozen vectors" doc, and update
expect.rs's walk-2 line.

### M-4: The walk step's "keep ONE call" rule rests on doc text that M-3 leaves stale

Plan line 798 correctly requires one walk call per invocation, because of the
csid side-effect warning. The only in-code statement of that rule is the stale
comment at `record.rs:206-209`. Fold it into M-3's move so the rule sits on the
function that has the side effect.

## Nit

- **N-1.** `a_good_plate_does_not_carry_a_bad_one` asserts only `code == 4` and
  empty stdout, not *which* plate was refused. It still kills M2, which is its
  purpose. Adding `r.err.contains(V12_NAMED)` would stop it passing if a future
  change refused the bundle for an unrelated reason.

---

## Lens results: what I verified sound

**1. Spec coverage.** All four §9a pieces are present: item 1 is Task 1 Steps
3-4, item 2 is Steps 5-6, item 3 is Task 2, item 4 is Task 3 (plus `--expect`, a
justified third reader). Both §8.9 `me` rows have gates that the **status quo
fails**, which I RAN on the installed `me 0.10.0`: `V12_SINGLE`,
`V4_UNDECODABLE` and `V4_ORIGINLESS_TEMPLATE` each gave `exit=0 … backup needs 1
public plate`, and `pack` on `V12_SINGLE` gave `an md1/mk1 this tool could not
decode`, with no version named. The spec's anti-gate is respected: F6 was
re-measured, and `me --hex` on a v8 chunk is byte-identical between 0.10.0 and
the probe (193 bytes, both builds), so no test relies on convert.

**2. Funds and restore.** F-635's fix is a strict refusal, and every error
path exits before `plates.push`. The only errors raised after the unchunked
loop are `SetIncompleteMd`/`SetIncompleteMk`, both refusals. On a real v8
2-key kind-1 chunked **wallet policy**, the probe gives `backup needs 4 public
plates`, `key_slots: 2`, `keyless_template: false`, which is correct (the
internal key takes no slot, §5). 0.10.0 refused that set outright (`exit 4,
unsupported md1 wire version`). `--expect descriptor` on it refused under 0.10.0
with the false "does not reassemble", and passes under the probe. The
`UnreadableVersion` push makes `out` non-empty, so pack still refuses
(`expect.rs` `check`, read in full). The new refusals of restorable plates are
limited to the strict-decode class: I checked all 30 md1 literals in the crate,
and only the 3 new fixtures plus two converter/seal fixtures that never reach
`bundle` are refused. F-652 owns the origin-less remedy. F5's panic stays
unreachable in production: `md1::address` has no non-test caller (grep over
`src/`). md-codec's 0.43-0.47 CHANGELOG adds no decode-side refusal for v4
payloads (0.45.1 *removed* the mint-policy leak into decode).

**3. False passes.** All 11 new tests fail on 0.10.0 (above, plus the author's
M1). No pre-existing test goes vacuous: no bundle test feeds an undecodable
unchunked md1 (literal sweep above), so the new early refusal cannot pre-empt a
later one a test was aiming at. The walk projection keeps `sysw/vectors.rs`'s
frozen vectors and `the_two_walks_agree…` meaningful (suite green).

**4. Journey walk** (v8 kind-1 set; the probe build vs `me 0.10.0`):

| step | operator has | `me` does | divergence | class |
| --- | --- | --- | --- | --- |
| `me bundle` | 4 v8 chunks | 4 plates, 2 key slots, "push via NFC & engrave" | the board is not reflashed, so the device says "Not an md1 descriptor chunk" (stage 3 fixes the wording) | documentation (M-1) |
| `me bundle` | 0.10.0 still on PATH | refuses, `unsupported md1 wire version`, no version named | an older binary | not our concern |
| `me sysw pack`/`show` | same | `confirmed` ×4 | a board without stage-3 firmware blanks the legend | warning or documentation (M-1); Step 0 must hold (I-1) |
| `me` (convert) | one chunk | NDEF, byte-identical to 0.10.0 | converting a v12 card is silent | not our concern (spec §9: BCH-only surface) |

**5. Execution order.** Every task runs on the tree left by the one before it.
F3 reproduced on a pristine tree: with only the Cargo.toml edits, `cargo check`
prints `Patch miniscript v13.0.0 … was not used` and the same 3 × E0599, and
**writes a `[[patch.unused]]` stanza into Cargo.lock**. `cargo update -p
miniscript` then removes that stanza and leaves a lock whose `+/-` lines are
**identical** to the probe patch's. The Task 1 test file compiles before the
unpin (`serde_json` is a normal dependency, `Cargo.toml:43`). Task 5 Step 8's
`cargo install --locked --path crates/me-cli` honours the root `[patch]`
(installed OK, and the result bundles v8). Both git sources are public:
descriptor-mnemonic `private: false`, and `cf35d61a` resolves via `gh`.
Miniscript `ff4732e5` is an ancestor of upstream master and equals
descriptor-mnemonic's `Cargo.toml:43-44` patch. The release matrix already
builds a git-rev dependency. My re-run of the probe: **nextest 675 run, 675
passed, 2 skipped**.

**Citations I re-checked at baseline:** `release.yml:48`, `bundle.rs:397-398`,
`md1.rs:351-357`, `sysw_cli.rs:670-675`, `expect.rs:202`, `cli.rs:22-23`,
`build-payload.sh:117,122`, `FOLLOWUPS.md:19210/19212`. F-651, F-652 and F-653
exist with `**Status:** OPEN`. v0.10.0 has 7 assets and the CHANGELOG has no
`[0.10.0]` section (F13). All are true.

ready for implementation: no
