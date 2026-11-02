**⚠ Important: 1 (S3(d), filed as `sh2-me-canonical-depth`, owning phase next me FIX; it does not block this build)**
architect sh2-FD-S1..S4: 4 answered; 3 prior rulings changed

Inputs: FIX doc `design/FIX_bughunt_sh2.md` (cda8f00a) §10 and context; grade
`bughunt-grade-sh2.md`; probe `/scratch/cache/bughunt/sh2/fixdoc-probe/`.
My probes are in `/scratch/cache/bughunt/sh2/architect-stops-probe/`
(copies of the author's slice trees; no repo edited).
Go: `export PATH=$HOME/.local/go/bin:$PATH CGO_ENABLED=0` (go1.26.4).

## S1 — relative repeated `s`: YES, fix-30d covers it (fold needed)

**Answer.** fix-30d covers relative `s`, and the 30d row's check is changed to
read the end point of a cubic. Without that check change the new subtest is a
false PASS risk: the row reads `Args[0]`, which for a cubic is p1; for `s`, p1
is the reflection `2*prevEnd - prevP2`, computed only from the first curve, so
it is `{1 0}` with and without the fix (measured: `s` subtest still FAILs with
`ends at {1 0}` after adding the fix, until the check reads `Args[2]`). The
author's "{0 0} on base" was that p1, not the end point.

**Why.** The grade's 30d ruling is "implicit repeated relative coordinates are
taken as absolute, so the glyph is silently wrong" and it extended 30d to `c`
because `c` "has the same stale-pen fault" (grade lines 71, 162). Relative `s`
in `parseSegments` (`case 's'`, s5tree `cmd/vectorfont/main.go` ~681) has the
identical fault: p2 and p3 of the i>0 pair are offset from the command's start
pen. Same reasoning, same fix shape; leaving it out keeps a wrong glyph path.

**Measured** (copy of the author's fx5 tree plus the change below):
- GREEN: `go test -count=1 -run '^TestBughuntSh230dRepeatedRelative$' ./cmd/vectorfont/`
  -> 5 subtests PASS, `ok  	seedhammer.com/cmd/vectorfont	0.002s`.
- RED (fx5 without the `s` change): `bughunt_sh2_s5_test.go:102: "m0,0 s1,0 1,0 1,0 1,0" ends at {1 0}, want {2 0}`.
- RED on base (s5base main.go, new test): l `{1 0}`, h `{1 0}`, v `{0 1}`,
  c `{1 0}`, s `{1 0}`, each `want {2 0}` / `{0 2}`, all at :102.
- Font regen with the `s` change: `go generate ./font/sh ./font/constant`
  exit 0, `diff -r font/ <s5base>/font` exit 0 (no committed glyph changes).
- Package: only `TestBughuntSh230aSpliceFirstLine` and `...30bRunMissingInput`
  fail, as on the author's copy; both are §6.2 deferred skips.

**Fold text.**
§2.13, replace the 30d bullet's last sentence ("Relative repeated `s` is not
covered (STOP 1).") with:
> The `s` loop does the same as `c` for i>0: `d := newPen.Sub(pen)` is added
> to p2 and p3 (p1 stays the reflection `newPen.Mul(2).Sub(newCtrl2)`).
> In the row, a cubic's end point is `Args[2]`: the check becomes
> `last := segs[len(segs)-1]; got := last.Args[0]; if last.Op == SegmentOpCubeTo { got = last.Args[2] }`.

§6.1, the 30d row becomes:
| `cmd/vectorfont` · `TestBughuntSh230dRepeatedRelative` (subtests `l`, `h`, `v`, the new `c` subtest `{"m0,0 c1,0 1,0 1,0 1,0 1,0 1,0", bezier.Pt(2, 0)}` and the new `s` subtest `{"m0,0 s1,0 1,0 1,0 1,0", bezier.Pt(2, 0)}`; a cubic's end is read from `Args[2]`) | PASS, all five subtests | :102 `"m0,0 l1,0 1,0" ends at {1 0}, want {2 0}`; `"m0,0 h1 1" ends at {1 0}, want {2 0}`; `"m0,0 v1 1" ends at {0 1}, want {0 2}`; `"m0,0 c1,0 1,0 1,0 1,0 1,0 1,0" ends at {1 0}, want {2 0}`; `"m0,0 s1,0 1,0 1,0 1,0" ends at {1 0}, want {2 0}` |

§1 table 30c/30d/49 line: "relative `l`/`h`/`v`/`c`/`s` pen tracking". The
"How checked" RED s5 line: "(with the new `c` and `s` subtests)". §5 holds.
No count changes (still one row). Not Brian's alone.

## S2 — fix-21 wording: replace the string (fold needed; grade wording changed)

**Answer.** The error string is exactly
`"self-check: unseated slot @%d declares %s, not the default %s"`
(no glyph is added). The §7 want-substring for the case "the decoded md1 puts
two slots at ONE origin with no fingerprints" (`composer_selfcheck_test.go:150`)
becomes
`"self-check: unseated slot @1 declares m/48h/0h/0h/2h, not the default m/48'/0'/1'/2'"`.

**Why.** The grade's string fails `TestProductionStringsAreDrawable`
(`after-s2.log`: `composer_selfcheck.go:200 carries '§' (U+00A7): no face has
a glyph for it, so the frame drawing this string renders its title and NOTHING
else`): a holder would see a blank refusal, so the ruled text has a C-class
defect and is replaced. "§4f" is a spec cross-reference a holder cannot use;
the sibling strings in the same function (`:206` "slot @%d declares %s, the
mapping review showed %s") state the fact plainly. Adding `§` to the faces is
a font change with its own review and still prints a spec reference: the
smaller fix wins.

**Measured** (copy of fx2 with only this string and the want-substring changed):
`go test -count=1 -run '^(TestProductionStringsAreDrawable|TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput|TestBughuntSh221SelfCheckUnseatedSlotOrigin)$' ./gui/`
-> all three PASS; `font_coverage_test.go:245: scanned 3395 production string
literal(s) across gui/*.go; 0 undrawable rune site(s)`; `ok  	seedhammer.com/gui	0.099s`.

**Fold text.** §2.5: replace the grade string and the two sentences after it with
> New error string: `"self-check: unseated slot @%d declares %s, not the default %s"`
> (architect S2: the grade's `§4f's` has no glyph; `TestProductionStringsAreDrawable` stays green).

§3 row: "fix-21". §7 row 1 "after": the case now gets
`self-check: unseated slot @1 declares m/48h/0h/0h/2h, not the default m/48'/0'/1'/2'`
and its want-substring changes to that text. §7 row 2
(`TestProductionStringsAreDrawable`): delete it (it stays green). §8 item 4:
drop "(fix-21 string per STOP 2)".

**Minor (record, not gating), M-S2a.** With the case retargeted, the
`"without two distinct fingerprints"` arm (`composer_selfcheck.go:237`) has no
row left (`grep -rn 'without two' gui/*_test.go` finds only a comment at :260).
The whole-diff review should check whether a fault-injection row (two seated
`st.assigned` at one origin, same fingerprint) is cheap; otherwise file it with
owner "end-of-project minors". Not Brian's alone.

## S3 — seam regeneration: take all of it in this build, except (d) (fold needed)

**Answer.** (a) add the 2 F-530 rows; (b) take all 11 device flips (29 true /
45 false) and amend the spec; (c) retire `panic:encode` (count 0); (d) keep
the 16-key row's canonical byte-identical in this build and file the me
canonical change as an Important follow-up owned by the next me FIX.

**Why.** (a) Without the F-530 rows a regeneration writes 72 rows and loses
two committed gate rows, so the regenerated file is wrong. (b)/(c) The seam
must show what the fixed device does. A column pinned at 40/34 fails
`TestDescriptorSeamDeviceColumn` on the fixed fork (FIX row 377), and a
`panic:encode` marker on rows that now refuse is a false label. (d) fix-01
keeps each key's own depth and child, while me's `canonical_string`
(`cascade.rs:262-277`) rebuilds them from the origin. Changing me's canonical
touches the promotion gate (`gate.rs:336-349`), `cascade_tests.rs:249-272` and
spec 405/663-666/1059. That is a separate me change, not the sh2 delta.
`canonical_from` with `origin_serialised` keeps today's canonical, so this
build has no wrong result. me and the fixed device still disagree on that
canonical, though, and that gap is the Important.

**Measured.** The me patch
`/scratch/cache/bughunt/sh2/architect-stops-probe/s3-me.diff`
(sha256 5f719fba…f6, covering rows.py, comment.json and descriptor_seam.rs)
passes `git apply --check` on this worktree. The fork patch `s3-fork.diff`
(b9d95f1d…6f, covering descriptor_seam_test.go) passes the same check on fx3.
The regenerated file has 74 rows, 84550 bytes and sha256
`34093bfd3aa8f303670098838ff6fe76c7cb7485199338088f795d9e15b0674b`
(`gen-s3c.log`).
- me: `cargo nextest run --locked --workspace` gives `691 tests run: 691 passed, 2 skipped`.
- fork: all 6 `TestDescriptorSeam*` PASS. The only other failures are
  `TestDescriptors` in bip380 (6 descriptors) and nonstandard (3 fixtures);
  FIX §7 rows 375-376 already own them (fix-01 fields).

After the flips, 5 of the 14 narrowed-4.7 rows stay device-true:
sh-16-keys, wsh-21-keys, mixed-network, the hardened use-site and the
non-consecutive multipath. 2 of the 5 narrowed-4.2 rows stay true. §2.3,
1519 and the 1887 coverage counts need no edit.

**Fold text.**
- Commit both patches verbatim as `design/FIX_bughunt_sh2-s3-me.diff` and
  `design/FIX_bughunt_sh2-s3-fork.diff`.
- §8 item 1 becomes: "apply `FIX_bughunt_sh2-s3-me.diff`, regenerate,
  sha256 must be `34093bfd…674b`". Item 2 becomes: "copy byte-identically,
  apply `FIX_bughunt_sh2-s3-fork.diff`: sha `34093bfd…674b`,
  `wantDeviceTrue = 29`, `wantDeviceFalse = 45`, `wantPanicEncode = 0`".
- Drop "Held by STOP 3" (and at lines 237-240). Row 377 gets "fixed by the
  re-pin (29/45, probe 0)".
- Delete STOP 3.
- §4 adds these spec texts:
  - after the §4.3 table (line 552):
    > **AMENDED (sh2 bug-hunt, fix-06a/06c).** The last three rows' device column is the pre-fix measurement; the device now refuses them (§4.7 amendment).
  - after §4.7 line 803:
    > **AMENDED (sh2 bug-hunt, fix-06a/06c/06b/50).** The device measurements above are pre-fix. The fixed device REFUSES nine of the fourteen §4.7 shapes (`tr(sortedmulti)`, `wpkh(sortedmulti)`, `pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `wsh(KEY)`, `sh(KEY)`, `k=0`, `k=−1`, `k>n`) and still ACCEPTS five (`n=16` directly under `sh`, `n=21` under `wsh`, the mixed-network multisig, the hardened use site, the non-consecutive multipath). `me`'s refusals are unchanged; the seam rows carry the measured `device_admits` (29 true, 45 false of 74).
  - line 984: replace "even though the DEVICE parses it and then panics on re-encode (§4.2 defect 1)" with "and since the sh2 bug hunt the DEVICE refuses it too (§4.2 amendment)".
  - §7, after "...are unchanged." (1670):
    > **AMENDED (sh2 bug-hunt, fix-06b/50):** the parser now refuses §4.2 defects 1–2, so `"panic:encode"` retired too; no row carries `device_probe`, and both halves pin its count at 0.
  - 1713: replace "`device_admits=true`" with "`device_admits` measured (true before the sh2 bug hunt; nine are false since, §4.7 amendment)".
  - after "R0 r7's NEW-M2);" (1737): " **AMENDED (sh2 bug-hunt):** both rows now carry a measured `device_admits: false` and no `device_probe`."
  - after 2058: " **AMENDED (sh2 bug-hunt):** the sh2 bug-hunt FIX is that later Go cycle for §4.2 defects 1–3 and §4.3's threshold gap."
- §9.1 adds "**Build impact (11):** spec §4.3/§4.7/§7/§9 amendments per FIX §4 (architect S3): `owed-to-build`".
- §9.2 adds an Important entry (not deferred-minor) under a new heading:
  > ### `sh2-me-canonical-depth` — me's promotion canonical rebuilds xpub depth/child from the origin; the fixed fork keeps the key's own (repo: **mnemonic-engrave**; owning phase: **next me FIX**)
  > - **Surfaced:** architect S3. **Where:** `cascade.rs:262-277`, `gate.rs:336-349`, `cascade_tests.rs:249-272`, spec 405/663-666/1059. **What:** after fix-01, me's and the device's canonical differ for a key whose depth/child disagree with its origin. **Seam:** row `sh-wsh-sortedmulti-16-keys` uses `canonical_from`/`origin_serialised` until this lands. **Status:** `open` **Tier:** v1
- §9.2 table adds a row: | S3 | M | `sh2-me-refusal-device-claim` | `refusal.rs:744`, `:779` say "the device's parser accepts" these shapes; true of released firmware, false after fix-06a/06c (fails closed) | next me FIX |

Not Brian's alone.

## S4 — the 39 host stub: `cmd/controller/main_host.go` (fold needed)

**Answer.** Add the file below. Its tag is the exact complement of
`main.go`'s `tinygo && rp`, the same pattern as `cmd/emu/main_notjs.go`.
sha256 `c24f6260ebc818322945d4f1312232ce7a50f0270f44d0647aea16a55bec0a17`.

~~~go
//go:build !(tinygo && rp)

// This file exists so `go build ./...` and `go vet ./...` succeed on a host.
// The seam in lockboot_values.go gives the package host-buildable files, and a
// main package with files must declare main.
package main

import (
	"fmt"
	"os"
)

func main() {
	fmt.Fprintln(os.Stderr, "controller is a device build: build it with tinygo -target pico-plus2")
	os.Exit(2)
}
~~~

**Why.** The seam needs host files, so the package needs a host `main`. A
different seam (moving the values to a non-main package) would be a larger
change. The message names the CI target (`.github/workflows/test.yml:150,153`).

**Measured** (`architect-stops-probe/s39`, fx3 plus the three seam files):
- without the stub, `go build ./...` gives the settled failure;
- with it, `go build ./...` rc=0 (`s39-build.log`), `go vet` with and without
  `-tags refugium` rc=0, gofmt clean;
- `go test -run '^TestBughuntSh239'` ok; `GOOS=js GOARCH=wasm go vet ./cmd/emu/`
  rc=0; `go test ./cmd/emu/` ok;
- `go run ./cmd/controller` prints the line and exits 2.

**Fold text.** §2.14 adds "plus `cmd/controller/main_host.go` (architect S4,
text verbatim)". The §3 table gets the file. §8 item 5 drops "(STOP 4)". §8
item 6 adds: "point 1 of `cmd/emu/confinement_test.go:94` now names
`main_host.go` (the bare build succeeds and exits 2)". Delete STOP 4. Not
Brian's alone.

Counts: Critical 0, Important 1 (sh2-me-canonical-depth, filed; it does not block this build), Minor 2 (M-S2a, sh2-me-refusal-device-claim), Nit 0. 4 answered; 3 prior rulings changed (S1 widens 30d to `s`; S2 replaces the fix-21 string; S3 widens the graded 6b/50 delta).
