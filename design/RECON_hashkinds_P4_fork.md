# RECON — hashkinds phase 4 (the SeedHammer fork)

Measured 2026-09-15 against `/scratch/code/shibboleth/seedhammer` at `0562e81`,
working tree clean. Every count below came from a command. This is recon for
phase 4, not the plan.

## Phase 4 is one phase on purpose

Spec §9: *"Changing `md.SpendPath.Hash` breaks 39 production and 76 test
references atomically; there is no tree where the port has landed and the UI has
not. Splitting it would label fork-native authorship as a convergence port."*

**I tried to check those two numbers with `grep` and RETRACTED the result.**
`grep -rn '\.Hash\b'` gave 51/74, and I briefly recorded that as "the spec has
drifted". It does not measure the same thing: `\.Hash\b` also matches
`chainhash.Hash{}` (a TYPE), `seal.FormatHash(p.Hash)` (a different `[16]byte`
field at `seal/open.go:40`), and `picobin`'s `Image.Hash()` METHOD. The number
is polluted and not comparable to whatever §9 counted.

**The instrument for this question is the COMPILER, not `grep`.** Change
`SpendPath.Hash`'s type and count what fails to build — which phase 4 does on
its first commit anyway, so the number costs nothing to obtain honestly and
nothing is gained by estimating it now. Treat §9's 39/76 as unverified in either
direction.

The argument §9 makes does not depend on the exact figures: the field is read
across the codec, the composer presets and the device UI, so there is no tree
where the port has landed and the UI has not.

## The four code obligations, located

**1. The record parser.** `sysw/composer_records.go`'s `ParseHashRecord` returns
`[32]byte` and demands `len(body) == 64`, so all three tagged kinds are
`ClassUnknown` and inert — verified mechanically by the P3 journey walk calling
the fork's own exported `Classify`. **4 non-test call sites.** This is the Go
twin of phase 3's §6 grammar.

**2. The compose hash slot.** `md/compose.go:167` — `Hash *[32]byte`, whose own
doc comment reads *"optional sha256 preimage"*. The Go twin of what phase 1
changed in `md-codec`.

**3. The lowering.** `md/compose.go:403` — `parts = append(parts, node{tag:
tagSha256, body: hash256Body(*h)})`, hardcoded exactly as the Rust was.

**4. The decode-side shape.** `Sha256Digests` appears **12** times.
SPEC_hashlock_kinds §7.4: `PolicyShape.Sha256Digests` records digests only for
`tagSha256` while setting `Hashlock = true` for all four, so a decoded
`ripemd160` card already shows "hashlock" with no digest. That is a decode-side
behaviour change and the spec says to declare it as one.

## THE TRAP THIS PORT INHERITS AND CANNOT HIDE

Phase 1 found that deriving Rust's `Eq`/`Hash`/`Ord` over a fixed `[u8; 32]`
makes the alloc-gate padding **observable**: two `ripemd160` locks with identical
20-byte digests differing in the padding compared unequal and hashed differently.
Rust could fix it by hand-writing the impls.

**Go cannot.** `==` on a struct holding a `[32]byte` compares all 32 bytes and is
not overridable, so the port must compare the slice explicitly — `bytes.Equal(a.Digest(),
b.Digest())` and a map key built from `(kind, digest[:len])`, never the struct.
Spec §5 puts **eleven** map/set/equality sites on this type, so the trap is
load-bearing wherever those land. The Rust source carries a note saying exactly
this, addressed to whoever writes this port.

## The lockstep fixture must be re-pinned

`sysw/testdata/record_class_vectors.json` and its `.provenance.json`. Phase 3
moved it to **81 rows**, sha256
`d6766fdd7308ce28a23ef954f227d8e4e8431f841b64ea336ea9ea9a09b928dd`, and the
Rust side pins the same constant — `sysw/composer_records_test.go` is measured
against it. Thirteen rows were added: each kind valid, an explicit `sha256:`,
wrong width per kind twice, and three unknown/miscased tokens.

## The device surface, located (§13)

Measured at fork `0562e81`. §13 says *"nothing here is optional"* and calls
§13.2 the cycle's operator-facing Critical.

**Three screens, all in `gui/composer_copy.go`, and all three take `method` and
no kind** — which is exactly the defect §13.2 describes:

| function | line | call sites |
| --- | --- | --- |
| `composerCopyHashlockConfirm(first8last8, method, chars, relation, otherPath)` | `:557` | 2 |
| `composerCopyHashlockReconcile(first8last8, method, chars)` | `:641` | 2 |
| `composerCopyPreimagePlateLead(first8last8, path, chars, method)` | `:717` | 2 |

Six non-test call sites to thread a kind through. The third is the one §13.2
notes *"the fold that wrote that rule missed the screen it condemned"* — it is
in scope, not a bonus.

**The reconcile screen is the Critical.** It instructs *"run `ms hashlock` … if
they differ, do not fund this wallet: build it again"*, and supplies only
`method: sha256` — the other axis. On a correct `hash256` wallet that check
fails, both values are 64 hex, and the operator complying exactly discards a
correct wallet and re-cuts five plates.

**The locator's `hash` row** is `gui/composer_preimage_plate.go:232`,
`hashlockPlateLocator`, which builds `"hash  " + hashlockFirst8Last8(digest)`
and takes a bare `digest [32]byte` — the same type problem as `md/compose.go`.
§13.1's measurement says the kind fits here (35 characters, 10 rows, 1.20 mm
spare at 3.0 mm) and **must not** go on the `method:` line or a new row, either
of which adds an eleventh row and blows the budget at every font rung.

`composerHashlockLocator` (`:256`) is the composer-native twin and
`hashlockPlatesLocator` (`gui/composer_hashlock_plates.go:205`) the Hashlock-flow
one; both feed the same rows.

**The plate's QR text** is bounded in `engrave/engrave.go:503` — *"the bound is
v9 (dim 53), which is what the H6 hashlock phrase plate…"*. §13.1's measurement
has the QR going 194 → 210 bytes and **staying at 53 modules**, so the reserved
envelope holds and ACCEPTANCE item 8 (the operator's ~43-minute QR-scan gate) is
unaffected.

## Toolchain facts, measured now

- **`gofmt -l .` returns FIVE files, exactly as `CLAUDE.md` records** —
  `gui/transaction.go`, `gui/transaction_golden_test.go`,
  `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`. A phase-4
  gate must diff against that five-file set, **not** assert the list is empty.

  **`gofmt` IS NOT ON THE DEFAULT PATH IN THIS SHELL**, and that matters more
  than the number. My first measurement here reported *zero* unformatted files
  and I very nearly wrote a "the baseline is stale, it's 0 now" correction into
  `CLAUDE.md` on the strength of it. The command did not exist —
  `gofmt: command not found` went to stderr while the count came from an empty
  stdout, so a **broken pipeline read as a clean tree**. Every Go command here
  needs:

      export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH

  A phase-4 gate that shells out to `gofmt` without that line does not fail —
  it **passes**, on every tree, forever.
- Go tests: shard with `mnemonic-engrave/scripts/gui-shard-test.sh <pkg> 24` —
  `-parallel` does nothing here because the `gui` package's tests never call
  `t.Parallel()`.

## Not measured here

Whether
`PolicyShape` crosses a wire boundary; the firmware size headroom for four
tokens; whether any Go test asserts the old `ParseHashRecord` signature.
