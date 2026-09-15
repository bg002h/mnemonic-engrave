# RECON — hashkinds phase 4 (the SeedHammer fork)

Measured 2026-09-15 against `/scratch/code/shibboleth/seedhammer` at `0562e81`,
working tree clean. Every count below came from a command. This is recon for
phase 4, not the plan.

## Phase 4 is one phase on purpose

Spec §9: *"Changing `md.SpendPath.Hash` breaks 39 production and 76 test
references atomically; there is no tree where the port has landed and the UI has
not. Splitting it would label fork-native authorship as a convergence port."*

**Those two numbers have drifted and the spec should not be trusted for them.**
Measured now:

| spec §9 says | measured at `0562e81` |
| --- | --- |
| 39 production `.Hash` references | **51** |
| 76 test `.Hash` references | **74** |

Not a defect in the argument — the point stands and is if anything stronger —
but it is the *"never hand-count what a tool can count"* class in the spec
itself, and a future reader quoting 39 would be quoting a number that was true
once. Re-measure before citing.

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

## The device surface (§13)

Not yet located in detail: the locator `hash` row, the confirm and reconciliation
bodies, the masked pick lead, and the census row. §6's binding rule applies to
every one of them — *where both axes could be read, both are named or neither
is* — and §13.2 calls the `hash256`-vs-`sha256` confusion the cycle's
operator-facing Critical.

## Toolchain facts, measured now

- **`gofmt -l .` returns 0 files.** A prior record in this constellation said the
  pristine baseline was five (`gui/transaction*.go`, `mt/mt*.go`, measured at
  fork main `fcd1546`, F-499). Those are formatted upstream now, so **a phase-4
  gate can assert `gofmt -l` comes back EMPTY** rather than diffing against a
  five-file allow-list.
- Go tests: shard with `mnemonic-engrave/scripts/gui-shard-test.sh <pkg> 24` —
  `-parallel` does nothing here because the `gui` package's tests never call
  `t.Parallel()`.

## Not measured here

Where the device's hashlock screens live and how many there are; whether
`PolicyShape` crosses a wire boundary; the firmware size headroom for four
tokens; whether any Go test asserts the old `ParseHashRecord` signature.
