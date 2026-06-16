# `me bundle` Phase A — PLAN architect review — R1 (convergence)

- **Stage:** R1 convergence of the mandatory plan R0 gate. Re-dispatch after the R0 fold.
- **Date:** 2026-06-16
- **Reviewer:** `feature-dev:code-architect` (adversarial; every public type/path verified in pinned-crate source).
- **Scope:** folded `design/IMPLEMENTATION_PLAN_me_bundle_phaseA.md` Tasks 5/6/7 vs `md-codec 0.36.0`, `mk-codec 0.4.0`, cross-checked vs md-codec's own `tests/bch_adversarial.rs`.
- **Verdict:** **GREEN — 0 Critical / 0 Important.** Plan R0 gate converged; plan is execution-ready.

---

## VERBATIM REVIEW OUTPUT

# `me bundle` Phase A — PLAN architect review (R1 convergence gate)

## Per-finding status

**I-1 — RESOLVED.**

Task 5, mk1 `match hdr` block:
```rust
match hdr {
    H::Chunked { chunk_set_id, total_chunks, chunk_index, .. } => Ok(Parsed::Mk1Chunk { … }),
    _ => Err(BundleError::Mk1SingleString(s.to_string())),
}
```
`StringLayerHeader` is confirmed `#[non_exhaustive]` at `mk-codec-0.4.0/src/string_layer/header.rs:33`. The `_` arm unconditionally satisfies E0004 for an external crate. Behavior correct: only `Chunked` maps to `Mk1Chunk`; `SingleString` and any future variants map to `Mk1SingleString`. No compile blocker remains.

**I-2 — RESOLVED.**

Every type/fn in `chunked_md1_vector()` verified public at the exact path written:
- `md_codec::Descriptor` — `lib.rs:46` `pub use encode::{Descriptor, …}`. (md-codec's own test imports `md_codec::encode::Descriptor`; same type via the re-export. Both compile.)
- `md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths}` — `lib.rs:29` `pub mod origin_path`; types pub at `origin_path.rs:19/47/82/91`. Fields exact: `PathComponent { hardened: bool, value: u32 }`; `OriginPath { components: Vec<PathComponent> }`; `PathDecl { n: u8, paths: PathDeclPaths }`; `PathDeclPaths::Divergent(Vec<OriginPath>)`.
- `md_codec::tag::Tag` — `lib.rs:31`; `Tag::Wsh` (`tag.rs:22`), `Tag::SortedMulti` (`tag.rs:31`).
- `md_codec::tlv::TlvSection` + `::new_empty()` — `lib.rs:33`; `tlv.rs:24/43`.
- `md_codec::tree::{Body, Node}` — `lib.rs:37`; `Node { tag, body }` (`tree.rs:9-14`); `Body::Children(Vec<Node>)` (`tree.rs:21`); `Body::MultiKeys { k: u8, indices: Vec<u8> }` (`tree.rs:35-39`).
- `md_codec::use_site_path::UseSitePath` + `::standard_multipath()` — `lib.rs:37`; `use_site_path.rs:49/58`.
- `md_codec::chunk::split` — `lib.rs:43`; returns `Result<Vec<String>, Error>`.

Chunk-count: md-codec's identical 6-cosigner × 15-component builder is documented "≥4 chunks" (`bch_adversarial.rs:40`); the plan's test asserts `chunks.len() >= 2` (always satisfied). The `md_codec::Descriptor` vs `md_codec::encode::Descriptor` path difference is not a mismatch (crate-root re-export). The function compiles against md-codec's external public API exactly as written.

**m-1 — RESOLVED.**

Task 6 `run_bundle` is now two-pass: collect `raw` lines → `Empty` if none → classify-only ms1 pre-scan returning `RefusedSecret` on the first `Format::Ms` (no BCH validation) → validating `parse_line` pass. Satisfies spec §4.2 ("refuse before validating ANY line"). `parse_line`'s own ms1 guard is now unreachable defense-in-depth. Tests `empty_input_is_usage_error` (`"   \n  \n"` → empty → `Empty`) and `ms1_anywhere_refuses_early` (`[MK1_A, MS1, MK1_B]` → pre-scan hits MS1 → `RefusedSecret` without validating MK1_A/MK1_B) both pass.

**m-2 — RESOLVED** (subsumed by I-1's `_` arm). **m-3 — RESOLVED** (subsumed by I-2's concrete `chunked_md1_vector`).

## Issues

### Critical
None.

### Important
None.

### Minor
None.

## Assessment

**I-2 compile check:** Every type/method is public at exactly the path written. The `md_codec::Descriptor` vs `md_codec::encode::Descriptor` apparent discrepancy is the crate-root re-export (`lib.rs:46`) — both identical, both valid. No import path deviates from the real public API.

**No new contradictions:** Task 6's classify-only pre-scan does not conflict with Task 5's `parse_line`. The `parses_*` unit tests call `parse_line` directly, unaffected by the `run_bundle` prologue. The two-pass structure is consistent, not duplicative; `parse_line`'s `RefusedSecret` path is now defense-in-depth.

**Regression sweep:** All 7 other R0 API claims untouched by the folds. `md_codec::chunk::reassemble`, `mk_codec::decode`, `StringLayerHeader::Chunked` fields, `ChunkHeader` fields, serde renames, clap optional subcommand, and `foreign_index1_same_setid` fixtures remain as R0 confirmed. Exit codes (2/3/4), manifest schema, and spec→task coverage unchanged. The plan's placeholder-scan self-review is accurate: both Task-7 helpers are now concrete and hermetic.

## Verdict: GREEN (0 Critical / 0 Important)

Both Important blockers (I-1, I-2) genuinely resolved. I-1's `_` arm compiles against `#[non_exhaustive]`. I-2's `chunked_md1_vector` uses exclusively public types at the exact confirmed paths, mirroring md-codec's own integration test. m-1–m-3 resolved. No fold introduced any new Critical/Important. The plan is ready for execution.

---

## Loop summary (added by main session)
| Round | Verdict | Folded |
|---|---|---|
| R0 | NOT-GREEN | 2 Important (I-1 `StringLayerHeader` non_exhaustive → compile error; I-2 md1 chunked-vector had no concrete public path) + 3 Minor (m-1 ms1 ordering; m-2/m-3 subsumed) |
| **R1** | **GREEN** | none — converged |

Plan R0 gate GREEN. All 8 API claims verified in source. Next: subagent-driven implementation in an isolated worktree, two-stage review per task, then a final whole-diff review.
