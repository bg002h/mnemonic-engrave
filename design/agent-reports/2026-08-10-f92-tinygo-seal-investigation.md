# F-92 investigation — `tinygo test ./seal/`, the wipe caveat, and the real options

**Read-only investigation.** Nothing in `seedhammer-b2b` or `seedhammer` was
modified. All destructive experiments (the build-tag split) were run against a
throwaway copy at
`/tmp/claude-1000/.../scratchpad/seedhammer-b2b-copy`, discarded at the end of
this session. `git status --short` in `seedhammer-b2b` is clean before and
after.

Repo under test: `/scratch/code/shibboleth/seedhammer-b2b` (branch `b2b`,
HEAD `75233b8`). Dev shell: `nix develop /scratch/code/shibboleth/seedhammer`.
TinyGo `0.41.1`, Go `1.26.3` (from `tinygo info`).

---

## 1. Why can't it build? — reproduced, and one layer deeper

**As shipped today**, exactly reproducing F-92's citation:

```
$ cd seedhammer-b2b && nix develop ../seedhammer --command tinygo test ./seal/
FAIL	seedhammer.com/seal	0.000s
# seedhammer.com/seal
seal/open_test.go:508:7: undefined: FileReader
seal/read_test.go:75:8: undefined: FileReader
... (9 lines total, 8 unique call sites + the import cycle isn't the issue)
exit code: 1
```

Root cause, read directly (`seal/read_host.go:1`, `seal/read_tinygo.go:1`):
`FileReader` is defined only in `read_host.go`, tagged `//go:build !tinygo`.
Eight sites across `open_test.go` and `read_test.go` reference it with **no
build tag on the file**, so the compiler sees an undefined identifier under
the `tinygo` build tag. This is a **test-file organization defect**, not a
defect in `seal`'s production code.

**FOLLOWUPS.md's "AMENDED 2026-08-09" note claims that fixing the split moves
the failure to a link error. I reproduced this independently** (not by reading
the note — by doing the split myself in the throwaway copy):

- Created `seal/filereader_test_host.go` (`//go:build !tinygo`) holding
  `writeRegion` + all 7 `FileReader`-dependent tests from `read_test.go`, plus
  `TestOpenFromAReader` moved out of `open_test.go`. Left the two `clampRegion`
  tests untagged in `read_test.go`, matching the file's own comment
  ("the bound lives in an UNTAGGED helper... precisely so a host test can kill
  the unbounded-read mutant").
- `go vet ./seal/` on the split copy: **exit 0**.
- `nix develop ../seedhammer --command go test ./seal/` on the split copy:
  **exit 0**, `ok seedhammer.com/seal 16.316s` (uncached). The split is safe.
- `nix develop ../seedhammer --command tinygo test ./seal/` on the split copy:
  **exit 1**, but now at LINK, not compile:

  ```
  ld.lld: error: undefined symbol: golang.org/x/sys/cpu.cpuid
  ld.lld: error: undefined symbol: golang.org/x/sys/cpu.xgetbv
  ```

**Traced the cause precisely** (not asserted from the FOLLOWUPS prose):
`seal/record.go:8-9` imports `btcaddr "github.com/btcsuite/btcd/address/v2"`
and `"github.com/btcsuite/btcd/chaincfg/v2"` directly (used at
`record.go:178,181` for address-decode validation). `go list -deps ./seal/...`
confirms the chain reaches `golang.org/x/sys/cpu` via
`btcd/chaincfg/v2 → btcd/wire/v2 → golang.org/x/crypto/sha3 → golang.org/x/sys/cpu`.

**Confirmed the mechanism, not just the symptom.** In the vendored module
(`~/go/pkg/mod/golang.org/x/sys@v0.45.0/cpu/`):
- `cpu_gc_x86.go` declares `cpuid`/`xgetbv`, gated `//go:build (386 ||
  amd64 || amd64p32) && gc`.
- `cpu_gc_x86.s` (the actual assembly implementation) carries the **same**
  `&& gc` constraint.
- There is **no `purego` fallback** for x86 in this package (checked: `grep -rl
  purego cpu/*.go` → no hits). The only alternatives are `cpu_gc_x86.s` (gc
  compiler) or `cpu_gccgo_x86.go` (gccgo, via cgo).

TinyGo satisfies the Go-source `gc` build tag (so `cpu_gc_x86.go`'s function
*declarations* get compiled in) but does not assemble TinyGo-incompatible
Plan9 `.s` files the same way `gc` does, so the *bodies* never materialize —
hence "undefined symbol" at link, not at compile. **This is a known, external
TinyGo/x-sys-cpu limitation on amd64 hosts, not something introduced by this
repo and not fixable by editing this repo's own code.**

**Confirmed the device build is unaffected**, exactly as FOLLOWUPS claims —
built the real firmware target from the (unmodified) real repo:

```
$ tinygo build -target pico-plus2 -stack-size 16kb -gc precise -opt 2 \
    -scheduler tasks -o /tmp/.../controller-test.uf2 ./cmd/controller
exit 0, 2,631,168 bytes
```

`cmd/controller/platform_sh2.go` is confirmed to import `seedhammer.com/seal`
(`grep -l "seedhammer.com/seal"`), so this is the real path that ships. ARM
(`GOARCH=arm`) never compiles `cpu_x86.go`/`cpu_gc_x86.*` (all gated on
`386||amd64||amd64p32`), so the amd64-only link failure cannot occur there.

**One correction to FOLLOWUPS' record:** it says "three filesystem call sites
... read `testdata/vectors.json` from disk." Measured: `grep -n
"os.ReadFile.*vectors.json" seal/*_test.go` finds **exactly two** —
`vectors_test.go:59` (`loadVectors`, called by every vector-driven test) and
`vectors_test.go:150` (`TestVectorFileMatchesTheDigestTheREADMERecords`, F-91).
Not load-bearing for the conclusion, but worth fixing in the record per this
project's own "measure it, don't describe it" rule.

---

## 2. What exactly is the untested caveat — and is it theoretical?

The caveat's canonical wording, found at `seal/open.go:53-54` (`Payload.Wipe`)
and echoed at `open.go:233`, `read.go:82`, `record.go:137-138`:

> "Same caveat as the rest of the firmware: TinyGo's GC may copy or retain, so
> this is defence in depth, not a guarantee."

**Enumerated every `clear(` call site in `seal`'s non-test code** (8 sites:
`open.go:58,61,235`, `pbkdf2.go:184,185`, `record.go:163,467`,
`unlock_key.go:88,110`, `session.go:74`). **Every single one targets a `[]byte`
or `[]bip39.Word` slice the calling function already owns a reference to** —
`clear(r.Record)`, `defer clear(plaintext)`, `clear(key)`, `clear(d.u[:])`,
`clear(m)`, etc. `clear()` is a Go-1.21+ language builtin: it writes zero
values into the named slice's backing array **synchronously, as part of
executing that statement** — this is a language guarantee, identical on `gc`
and TinyGo, and it does **not depend on GC mode, moving vs. non-moving
collection, or when/whether the collector runs.** `TestWipeZeroesSecretRecords`
(`open_test.go:619`) and its siblings assert exactly this: that the named
buffer reads all-zero after `Wipe()`. **For this class of assertion, the GC
caveat is not applicable — it would hold trivially under `-gc precise` if the
package could even compile there,** because nothing about it depends on GC
behavior in the first place.

**What the caveat is actually about, read correctly, is a narrower and
different risk:** *other, unintended* copies of secret bytes that no `clear()`
call reaches — created by `append`-triggered reallocation (old backing array
orphaned, unreferenced, un-zeroed), string conversions (immutable, unwipeable
by construction — this is F-88/F-90's territory, already tracked separately),
or compiler-specific escape-analysis / stdlib-internals divergence between
`gc` and TinyGo. **None of `seal`'s own host tests assert this second class
directly** (I grepped for `cap(`/`orphan`/`NeverGrows` patterns in `seal/*.go`
— none exist there).

**But `seal`'s own `clear(m)` at `record.go:163` (inside `Classify`) leans
directly on a guarantee from a *different*, sibling package** — `bip39.Parse`'s
`make(Mnemonic, 0, 24)` fixed-capacity accumulator (`bip39/bip39.go:274`),
whose own comment says explicitly: "Preallocated to the maximum the loop below
enforces, so append NEVER grows and never orphans a partial copy... measured, a
12-word parse orphaned copies holding 1, 2, 4 and 8 words." **This IS exactly
the second, GC/compiler-sensitive risk class, and it IS directly testable
right now — I ran it:**

```
$ nix develop ../seedhammer --command tinygo info -gc precise
garbage collector: precise      # confirms the flag actually selects it

$ nix develop ../seedhammer --command tinygo test -gc precise ./bip39/
ok  	seedhammer.com/bip39	0.172s     # exit 0

$ tinygo test -v -gc precise ./bip39/ | grep -E "PASS|FAIL"
--- PASS: TestParseNeverGrowsItsResult (0.00s)          # asserts cap(got)==24
--- PASS: TestParseZeroesItsAccumulatorOnEveryErrorExit (0.00s)  # asserts the
                                                          # accumulator reads
                                                          # all-zero on every
                                                          # error return
--- PASS: TestParseAccumulatorIsPopulatedBeforeTheErrorExit (0.00s)
```

**This directly refutes F-92's own claim that "`bip39.Parse`'s `make(Mnemonic,
0, 24)` [is] validated only under gc Go on linux/amd64."** `bip39` has no
`FileReader`-style test-tag defect and no `btcd`/address import, so it compiles
and runs cleanly under `tinygo test`, and it does so **under `-gc precise`,
the exact GC the shipping firmware uses** — the one flag flake.nix sets
(`tinygo-flags = "-target pico-plus2 ... -gc precise ..."`) and the one the
caveat names. `TestParseNeverGrowsItsResult` — the test that would fail if
TinyGo's `append`/`growslice` policy diverged from `gc`'s and actually caused
the orphaning the comment describes — **passes under that exact GC.**

**Net answer to the crux question:** the caveat is **partly theoretical and
partly real, and they are different in kind:**
- **Theoretical** for every direct `clear(buf)` call in `seal` — these are
  language-guaranteed, GC-independent, and (via the bip39 result above)
  empirically hold under `-gc precise` for the one case actually testable
  today.
- **Real but narrower than advertised** for the append-orphan /
  escape-analysis / stdlib-internals class — and for that class there is now
  **positive, on-the-actual-GC evidence for the bip39 half of it** (host/amd64
  architecture only — the residual, still-open gap is *architecture*, not GC
  mode: TinyGo's ARM/Cortex-M33 codegen, calling convention, and its own
  `crypto/aes`/`pbkdf2` internals are not exercised by an amd64 host run).

---

## 3. Options, ranked, with cost and what each proves

**Free and already-safe — do regardless of what else is chosen:**
Ship the build-tag split (§1). Cost: near zero (verified: `go vet` clean,
host `go test` passes at 16.3s). Proves nothing new by itself, but is the
prerequisite for every option below, and stops `seal` from being the one
package in the constellation whose own tests cannot compile under TinyGo at
all — right now it is worse off than `bip39`, `slip39`, etc. for no reason
connected to the wipe caveat.

**(a) Fix the amd64 host link failure directly.** Not viable without
upstream/vendoring changes: the `golang.org/x/sys/cpu` gap is external
(confirmed: no `purego` fallback exists in the vendored `v0.45.0`), and
avoiding the `btcd` import means moving address-validation out of
`seal.Classify` — a change to production code's package structure, motivated
purely by a test-build constraint. **Not recommended.**

**(b) Extract wipe-critical code into a package that builds under TinyGo.**
Weaker than it sounds: `Payload`, `Wipe()`, `AdmittedRecord` live in the same
package (`seal`) as `record.go`'s `btcd` import, so Go compiles them into one
test binary — you cannot cherry-pick just the wipe path without physically
splitting `seal` into two packages (a real architectural change to an
actively-developed, security-critical, already-heavily-reviewed package with
an open funds-safety audit). A *lighter* version — a brand-new, standalone
package that merely re-demonstrates `clear()` zeroes a slice — would prove
only a Go-language-conformance fact already covered for free by bip39's
existing, passing test (§2). **Not worth the risk/cost for what it would
prove.**

**(c) Test on the target under `-gc precise`, two sub-routes — recommended:**

- **(c1) qemu-emulated ARM, automated, CI-able.** `tinygo targets` lists
  `cortex-m-qemu`/`riscv-qemu`. Verified `cortex-m-qemu` is `GOARCH=arm`
  (`thumbv7m`), so it never compiles the amd64-gated `cpu_x86.go` files —
  sidesteps the link failure entirely. Verified `tinygo info -target
  cortex-m-qemu -gc precise` reports `garbage collector: precise` — the
  combination is valid. **Blocked today**: `which qemu-system-arm` etc. all
  report "not found" in the dev shell — confirmed no qemu binary on `PATH`.
  Needs a `flake.nix` addition (adding `pkgs.qemu` to the devShell). Also
  needs the 2 `os.ReadFile("testdata/vectors.json")` sites (§1 correction: two,
  not three) converted to `//go:embed`, since a bare-metal qemu target has no
  filesystem. Cost: a small, scoped piece of work (flake.nix change +
  go:embed conversion + `-target cortex-m-qemu -gc precise` wiring), roughly
  plan-sized, not a one-line fix. **What it proves:** TinyGo's actual compiled
  `-gc precise` code path, on ARM, for `seal`'s own `clear()` sites and (if
  ported) `bip39`'s accumulator guarantee — closer to real than a host run,
  but `cortex-m-qemu` emulates an ARMv7-M Cortex-M3 (LM3S6965), not the
  RP2350's ARMv8-M Cortex-M33, so instruction timing, cache behavior and any
  M33-specific codegen remain unexercised.

- **(c2) On real RP2350 silicon, manual, precedent already exists.**
  `cmd/sealread/main.go` is exactly this pattern today: a `//go:build`-free,
  TinyGo-only `main` package (imports `machine`), built with `tinygo build
  -target pico2`, signed, flashed to a spare **Pico 2 rehearsal board** (never
  the SH2 — the file's own header has a detailed board-identification
  procedure), and read back over CDC via `scripts/cdcread.py`. A `cmd/sealwipe`
  sibling — seal a payload, unlock it, call `Wipe()`, then read back the
  region via `unsafe.Pointer` (same trick `sealread` already uses) and print
  PASS/FAIL over serial — would be the most direct proof there is: the actual
  compiled code, the actual `-gc precise` firmware flags, the actual RP2350
  silicon. Cost: a hardware cycle per run (flash + physically read output),
  **not CI-automatable**, and (per `sealread`'s own header) not itself
  regression-tested — a one-shot measurement, not a suite. **What it proves:**
  everything (c1) proves, plus the real architecture — but only for the one
  run performed, not on every future commit.

**(d) Accept and document the gap in SPEC §2.2, narrowed to what's actually
open.** Given §2's finding, this is defensible **only if scoped precisely**:
document that (i) every direct `clear()` call in `seal` is GC-independent by
construction and needs no further verification; (ii) the append-orphan
guarantee `Classify`'s `clear(m)` depends on is verified today under
`-gc precise` for `bip39` on host/amd64 (cite the exact command and PASS
list above); (iii) the one honestly-open gap is architecture, not GC mode —
TinyGo's ARM/RP2350 codegen and stdlib-crypto internals are unexercised by any
host run, and closing it needs (c1) or (c2). Writing "the caveat is
unverified" without this narrowing would overstate what's actually missing;
writing "the caveat is fully resolved" would understate it.

---

## Recommendation

**First choice: do the build-tag split now (free, proven safe), then pursue
(c1) — add qemu to `flake.nix` and get `seal` (and ideally `bip39`, already
qualified) running under `tinygo test -target cortex-m-qemu -gc precise`.**
It is the only option that is both automatable/CI-able and exercises TinyGo's
actual compiled `-gc precise` output rather than a host-compiled analog,
without requiring a production-code refactor. Budget it as its own small
piece of work (flake.nix + go:embed), not a quick fix — do not let "the split
is free" imply the qemu route is free too.

**In parallel, close the informational half immediately, at zero cost:** the
evidence in §2 (bip39's `TestParseNeverGrowsItsResult` /
`TestParseZeroesItsAccumulatorOnEveryErrorExit` passing under `tinygo test
-gc precise ./bip39/`, exit 0) already narrows F-92's own headline claim. This
is not a fix to `seal`, but it is a fact worth folding into F-92 and/or
SPEC §2.2 today: the caveat is **not** "never tested on the toolchain that
ships" in the blanket sense F-92's title states — one of the two named
examples (`bip39.Parse`) already is, today, with no code changes, just not on
the target architecture.

**What none of this proves:** ARM/RP2350-specific codegen or timing
divergence, TinyGo's own `crypto/aes`/PBKDF2 stdlib-port internal buffering
versus `gc`'s, or anything about `seal`'s specific `clear()` sites compiling
and running correctly under TinyGo at all (still blocked on the link error
until (c1) or (c2) is done) — those remain genuinely open until qemu or
on-device testing lands.

**Flag as requested:** the caveat is **not uniformly theoretical** — it
splits cleanly. The `clear()`-reaches-the-named-buffer half is theoretical
(language-guaranteed, evidenced, would hold trivially). The
append-orphan/escape-analysis half is real, narrower than F-92 states, and
partially (bip39, host-arch) already closed by evidence gathered in this
investigation with zero code changes.
