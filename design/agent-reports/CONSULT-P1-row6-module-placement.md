# CONSULT — P1 row 6: where `write_private` lives in `mnemonic-io-lib`

Question: new module vs `channel` vs `fd`, and whether to add a root re-export.
Scope: placement only. The move itself, and the 0o600-as-mechanism ruling, are
settled by `design/IMPLEMENTATION_PLAN_P1_mt_adopts.md` §4 row 6.

## Recommendation

**A new module: `pub mod write;` → `mnemonic_io_lib::write::write_private`.
No root re-export.**

## Why the two existing modules lose

- **`channel` is disqualified by its own constitution.** Its header says
  "Classification ONLY", "it does not announce anything", and — load-bearing,
  in bold — that `destination` "never touches a path"
  (`crates/mnemonic-io-lib/src/channel.rs:8-11`). Housing the crate's only
  function that opens, truncates, chmods and writes a path inside the module
  whose stated identity is *never touching one* doesn't stretch the header, it
  contradicts it. (The header's "`write_private` stays in `me`" sentence is
  falsified by the move **wherever** the function lands; row 6's fold must
  rewrite it to point at the new home, e.g. "the fix is
  [`super::write::write_private`] tightening the OPEN file".)
- **`fd` is a near-miss, not a fit.** Its header is "MECHANISM ONLY: what was
  **measured** about stdout" (`fd.rs:1`). `write_private` is mechanism, but it
  is the *write* side and takes a *path*; `fd` is the *read* side of fd 1
  specifically. Folding a writer into it widens a deliberately narrow contract
  — the same header that carries the "no disqualifying mask lives here"
  argument — from "observation of stdout" to "observation of stdout, plus a
  file writer", and nobody looking for "how do I write the `--out` file" opens
  a module named `fd`.

## Why a 7th module is justified at 6 modules / ~11 public items

In this crate a module is a **contract boundary, not a code bucket**: each
header is a one-line constitution (classification only; decisions, no
integers; what was measured; shaping only; purge text). None of the six covers
"effectful filesystem write". `write_private`'s contract is distinct and worth
its own header in the house style: *owner-only for the file's entire
existence — `0o600` on CREATE and again on the OPEN file, because CREATE-only
lets a pre-existing 0644 target keep its mode; no policy about whether to
write, and no knowledge of what the bytes are.* One function is thin, but the
boundary is what's being bought — and row 10 gives it a second caller (`mt
encode --out`) immediately.

## Root re-export: no — and adding one would deepen the inconsistency

The root set (`lib.rs:74-76`) re-exports `channel`/`exit`/`records` items and
skips `fd`/`observation`/`remedy` with no inferable rule; §3 of the plan
already flags the `mnemonic_io_lib::history_purge_block` E0425 footgun.
Adding `pub use write::write_private;` does not fix that footgun (it's about
`remedy`), makes the set 4-of-7 with still no rule, and — decisive here —
mints a **second public path** for a symbol that a sibling repo is about to
freeze by git rev. Minimal frozen surface wins: one canonical path. It also
matches how `mt`'s implementer already consumes this crate — the §3 probe
called `fd::mode_of` and `remedy::history_purge_block` module-qualified — and
`write_private` sits conceptually beside `fd` (write side / read side of the
same permission concern), both behind module paths. The real cure for the
root-set inconsistency is an all-or-nothing decision taken deliberately, not
one more accretion in a placement row.

If forced to call the re-export a tie: I ship the no-re-export side, because
an omitted re-export can be added compatibly later, while a shipped one is
pinned the moment `mt` pins the rev.

## Exact `lib.rs` edit

After `pub mod remedy;` (declarations are alphabetical; `write` sorts last):

```rust
/// Owner-only file creation: `0o600` on CREATE **and** re-asserted on the
/// OPEN file, so a pre-existing target cannot keep an older, wider mode.
pub mod write;
```

The `pub use` block at `lib.rs:74-76` is left unchanged.
