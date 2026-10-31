# mnemonic-engrave

Paths below are relative to this Git root, including when working inside a crate.

## Start here

Read [CLAUDE.md](CLAUDE.md) explicitly; AGENTS.md does not load it automatically.
Read applicable ancestor guidance too. Existing process rules remain applicable;
verify historical architecture/status statements against source.
Read the shared [guide](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/README.md) and [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/WORKFLOW.md).
For sibling checkouts/worktrees, prefer local `../mnemonic-toolkit/docs/architecture/`;
remote links support standalone checkouts while documentation branches await integration.
Check Git status, the current branch, Cargo.toml and Cargo.lock before editing.
Preserve unrelated work and use explicit task-owned paths when staging.

## Where behavior lives

- `crates/me-cli` is package mnemonic-engrave, library mnemonic_engrave, binary me.
  [lib.rs](crates/me-cli/src/lib.rs) exposes host operations; [main.rs](crates/me-cli/src/main.rs)
  owns CLI stream/output policy. Library consumers can disable default CLI features.
- [bundle.rs](crates/me-cli/src/bundle.rs) admits public md/mk, refuses ms/mt, and
  must validate supported descriptors before reporting plate counts.
- [sysw records](crates/me-cli/src/sysw/record.rs) distinguish secret/bearer material.
  Keep seal and sysw container formats separate even where primitives are shared.
- [mnemonic-io-lib](crates/mnemonic-io-lib/src/lib.rs) owns reusable mechanisms;
  callers keep classification, refusal, permissions and output policy.
- [preview.rs](crates/me-cli/src/preview.rs), [preview/go.mod](preview/go.mod) and the
  pinned SeedHammer submodule define the host/device-source boundary.

## Contracts and checks

Design here owns mt's normative spec and independent vector generator as well as
host delivery/container contracts. Use codec/descriptor seams, container/record tests
and Rust/Go integration in [.github/workflows/release.yml](.github/workflows/release.yml).
Follow CLAUDE.md's Rust-primary rule and provenance requirements for normative Go ports,
including checking Rust when investigating a suspected Go-only defect.
For this architecture project, firmware implementation and general Refugium are outside
scope. Gemini owns `design/firmware-architecture/` and GEMINI.md; preserve its work.
Coordinate shared md pins with Refugium without assuming every planned consumer exists.
