# F-449 stage 1b — scoped re-review of the whole-branch C-1/I-1 fix

**Scope:** did the fix at `825247c8` address C-1 and I-1, and did it introduce a
new defect — especially OUTSIDE the Liana feature? This is NOT a fresh audit; the
whole-branch review already ran and its findings are not re-derived here.

**Subject:** `/scratch/code/shibboleth/dm-worktrees/f449-stage1b`, tip
`825247c8ab786b976e6531a45c05a1a322269aad`, clean.
**Fix commit:** `482a07e6..825247c8` (4 files, +296/-8).
**Answering:** `design/agent-reports/f449-stage1b-whole-branch.md` (1C/1I/3M/2N).

---

## Verdict

**NOT READY.**

| Finding | Disposition |
| --- | --- |
| **C-1** (Critical) — recogniser compared only the extended key | **ADDRESSED** |
| **I-1** (Important) — marker outside the internal-key position leaked `internal:` | **ADDRESSED for ASCII input; NOT addressed for a reachable multi-byte-UTF-8 subclass, where it now PANICS** |

**New findings: 0 Critical / 1 Important / 0 Minor / 1 Nit.**

- **N-1 (Important)** — `validate_marker_position` slices `&template[pos - 3..pos]`
  without a char-boundary check, so `md encode` / `md verify --template` abort with a
  Rust panic (exit **101**, printing an internal source path and line) instead of the
  clean refusal the I-1 fix exists to deliver. Introduced by this fix.

---

## Method — what was actually run

Three binaries, **built identically** (`cargo build --locked -p md-cli --bin md
--all-features`, toolchain 1.85.0), each in its own `CARGO_TARGET_DIR`:

| name | rev | what it isolates |
| --- | --- | --- |
| `base` | `37367c1f` (main, pre-feature) | the whole branch |
| `prefix` | `482a07e6` (branch tip BEFORE the fix) | **the fix alone** |
| `tip` | `825247c8` (branch tip WITH the fix) | — |

> A first pass built `base` with default features while the supplied tip binary
> carried `cli-compiler`; that manufactured a spurious `md compile` exit-code
> difference (2 vs 1, "compile requires the cli-compiler feature"). All three were
> rebuilt with `--all-features` and every number below is from the matched builds.
> The rebuilt tip was cross-checked against the supplied
> `f449-1b-target/debug/md` and agrees.

**15,147 invocations per binary — 45,441 process runs** — stdout, stderr and exit
code captured and diffed in full:

| stage | invocations | corpus |
| --- | --- | --- |
| 1 | 845 | the 65 vendored vectors × `encode` (plain / `--path bip48` / `--experimental`), `decode`, `decode --json`, `inspect`, `inspect --json`, `bytecode`, `verify`, `descriptor`, `address`, `address --json`, `repair` |
| 2 | 11,302 | 915 template/descriptor literals harvested from `crates/md-cli`, `crates/md-codec`, `docs/`, `design/` and the JSON fixtures × `encode`×3, `verify`, `descriptor --template`, `address --template`, `decompose` × 6 emit modes; plus 46 base-emitted concrete descriptors × 6 emit modes + `--network testnet` |
| fuzz | 3,000 | randomised templates (seed 20260922) over 7 key fragments × 11 wrappers, with 0–2 injected noise characters drawn from `€ é 😀 — NBSP ZWSP “ \t space ñ 中` |

Plus `md vectors --out` run under all three (306 generated files, `diff -r`), the
8 vendored Liana `cases.json` descriptors, and hand-built probes for the marker
position heuristic and the prefixed-origin claim.

---

## Question 1 — did the check on the universal path change anything else?

**No.** `validate_marker_position` is a no-op when the marker is absent
(`match_indices` yields nothing), and that is what the measurements show.

- **`prefix` vs `tip`, across all 15,147 invocations: every single difference has
  `UNSPENDABLE(liana)` in the input.** Differences with NO marker in the input:
  **0**. Nothing that is not about this feature changed message, stage or exit code.
- **Previously-accepted templates that now fail: 0** (`prefix`→`tip` and
  `base`→`tip` both).
- `md vectors --out`: 306 files, byte-identical across `base`, `prefix`, `tip`.
- Stage 1 (845 invocations), `base` vs `tip`: 176 raw differences, **176 of 176
  explained by the deliberate `"schema": "md-cli/1"` → `"md-cli/2"` bump**,
  0 residual. `prefix` vs `tip`: 0 differences at all.
- The 12 corpus + 165 fuzz `prefix`→`tip` differences that are not panics are all
  one thing: the generic `template contains no @i placeholders` refusal replaced by
  the new message naming the marker and its one legal position. That is the I-1 fix.
- `"internal:"` appears in **0** of the 45,441 captured stderrs, under any binary.

Stage 2 showed 144 `base`→`tip` differences. 24 have the marker in the input; the
other 120 are Liana output or Liana guidance — 72 are the extended
"If this wallet's internal key is Liana's derived unspendable key…" continuation
appended to the concrete-descriptor guidance, 48 are `decompose` now emitting
`UNSPENDABLE(liana)` for genuine Liana descriptors. **All 120 are `prefix == tip`**,
so none originate in the fix; they belong to commits `482a07e6` and earlier and were
the whole-branch review's scope, not this one. (Noted because the dispatch brief
recorded the earlier sweep as returning zero differences; against `37367c1f` it does
not, for branch reasons unrelated to the fix.)

## Question 2 — did C-1's tightening break anything legitimate?

**No.** The mirror-image failure mode was hunted directly and does not occur.

| input | base | prefix (buggy) | tip | verdict |
| --- | --- | --- | --- | --- |
| 8 vendored `cases.json` descriptors | exit 0 | recognised | **8/8 recognised, `--emit descriptor` byte-exact, `prefix == tip`** | recognition NOT disabled |
| `…/<2;3>/*` internal key | exit 0, no marker | exit 0, **marker** (the Critical) | exit 0, no marker, **stderr byte-identical to base** | C-1 fixed, base behaviour restored |
| `[73c5da0a/48'/0'/0'/3']…/<0;1>/*` | **exit 1**, depth-inconsistency refusal | exit 0, **marker** | **exit 1, byte-identical message and exit code to base** | pre-existing refusal restored |
| `[73c5da0a/0']…/<0;1>/*` | exit 1, same refusal | exit 0, **marker** | exit 1, byte-identical to base | same |
| `[73c5da0a]…/<0;1>/*` (fingerprint-only origin, depth-consistent) | exit 0, no marker | exit 0, **marker** | exit 0, no marker, identical to base | origin guard correct, no over-refusal |

**The implementer's `check_depth_consistency` claim is verified, and it is genuinely
pre-existing behaviour restored, not a new refusal wearing an old message:**

- `check_depth_consistency` exists at base — `37367c1f:crates/md-cli/src/decompose/mod.rs:368`.
- Its body is **identical** base vs tip (`diff` of lines 360–400: no output).
- The fix commit `825247c8` **does not touch `decompose/mod.rs` at all** (it touches
  `decompose/walk.rs`, `parse/template.rs` and two test files).
- Base and tip produce the **same stderr bytes and the same exit code** for the
  prefixed-origin input, while `prefix` silently succeeds and mints the marker.

Ordinary spendable internal keys, the literal NUMS point and kind-0 vectors all
behave exactly as at base — covered by the 306-file `md vectors` identity, the 65
vector invocations and the 46 concrete-descriptor decompositions, all with 0 residual.

---

## N-1 (Important) — NEW: panic on a misplaced marker preceded by a multi-byte character

`crates/md-cli/src/parse/template.rs:1122`

```rust
let preceded_by_tr_open = pos >= 3 && &template[pos - 3..pos] == "tr(";
```

`pos` is a **byte** offset from `str::match_indices`. `pos - 3` is assumed to be a
UTF-8 character boundary. When the three bytes before the marker straddle a
multi-byte character, the slice panics.

**Reproduction (one line):**

```console
$ md encode 'tr(x€yUNSPENDABLE(liana))'
thread 'main' panicked at crates/md-cli/src/parse/template.rs:1122:56:
byte index 5 is not a char boundary; it is inside '€' (bytes 4..7) of `tr(x€yUNSPENDABLE(liana))`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
$ echo $?
101
```

Base gives a clean `md: template parse error: …` at exit 1 for the same input.

**Measured properties**

- **Introduced by this fix.** `prefix` (`482a07e6`) does not panic on any of these
  inputs; `base` does not either. All 14 fuzz panics are new vs both.
- **Reachable on the universal path**, which is exactly the risk the brief named:

  | verb | base | tip |
  | --- | --- | --- |
  | `md encode <TEMPLATE>` | 1 | **101** |
  | `md encode <TEMPLATE> --experimental` | 1 | **101** |
  | `md encode --in FILE` | 1 | **101** |
  | `md verify --template <TEMPLATE> <md1…>` | 1 | **101** |
  | `md descriptor --template` / `md address --template` | 1 | 1 |
  | `md decompose` | 1 | 1 |

- **Both delivery channels**: argv and `--in FILE`.
- **Rate:** 14 of 3,000 randomised inputs; a single panic site
  (`template.rs:1122:56`) in all 14.
- **Triggering characters are ordinary paste artefacts**, all confirmed panicking:
  NBSP (U+00A0), em dash (U+2014), left smart quote (U+201C), euro (U+20AC),
  `é` (U+00E9), `😀` (U+1F600). The marker is a literal a user is documented to
  type by hand, and a descriptor pasted out of a PDF, wiki or chat routinely
  carries one of these.
- A UTF-8 BOM at the start of an `--in` file does **not** trigger it (checked);
  the hazard is a multi-byte character 1–3 bytes upstream of the marker.

**Why Important and not Minor.** The entire purpose of the I-1 fix is that a
misplaced marker draws a clean refusal naming the marker and its one legal
position, and that SPEC §4a's "an internal invariant string must never reach a
user" is honoured. For this input class the refusal does not refuse: it aborts the
process, prints an internal source path and line number, and exits 101 — strictly
worse than the `internal: synthetic key … not found in key map` string the finding
was filed to remove. Under the project severity rule this is "a refusal that does
not refuse" and an unsound assumption (`pos - 3` is assumed to be a char boundary
and is not), both of which are named as blocking. It is not Critical: nothing is
minted, no wrong card is produced, no funds are exposed, and it fails loudly.

**Why it is not intrinsic.** The check's semantics do not require byte slicing —
`template[..pos].ends_with("tr(")` is boundary-safe by construction and cannot
panic. Offered as evidence that the defect is incidental, **not** as a prescribed
remedy; the implementer should reproduce the panic and choose the fix.

**Not covered by the suite.** The new tests in `liana_input_side.rs` are all-ASCII,
so 1499/1499 green is consistent with this defect. A fix should carry a test whose
template holds a multi-byte character immediately upstream of a misplaced marker.

---

## N-2 (Nit, pre-existing — NOT introduced by this fix)

`wsh(tr(UNSPENDABLE(liana),…))`, `sh(tr(…))` and `xtr(UNSPENDABLE(liana),…)` all
satisfy the `"tr("`-prefix position check, so the marker is substituted and the
failure surfaces as `miniscript parse failed: unrecognized name
'fa1446b119da8e010be1…'` — naming a 64-hex synthetic key the user never typed
rather than the marker they did. No `internal:` leak, and `prefix == tip` on every
such shape, so this predates the fix. It does qualify the doc comment's claim that
the position check is sound because `tr()` cannot nest: text of that shape *does*
reach substitution; it merely happens to die at `Descriptor::from_str` rather than
at `lookup_key`. Worth a follow-up entry for message quality only.

---

## Summary

- **C-1: ADDRESSED.** Recognition correctly narrowed to origin-less
  `MultiXPub` / `Unhardened` wildcard / `derivation_paths == [[0],[1]]`; all 8
  vendored cases still recognised and byte-exact; every rejected shape now matches
  base byte-for-byte on stdout, stderr and exit code; 0 legitimate descriptors lost
  across 15,147 invocations.
- **I-1: ADDRESSED for ASCII, NOT for multi-byte UTF-8** — see N-1.
- **Nothing outside the Liana feature changed.** 0 differences `prefix`→`tip` on any
  marker-free input; 0 residual `base`→`tip` in stage 1 after the deliberate schema
  bump; `md vectors` output byte-identical.
- **Counts: 0C / 1I / 0M / 1N. Merge verdict: NOT READY** — N-1 blocks.

Worktree left clean at `825247c8`; the temporary `482a07e6` worktree was removed.
