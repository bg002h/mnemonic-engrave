# Composer S0 — adversarial execution review of the implementation diff (R0)

**Diff range:** `b19dca7b..9820e618` (9 commits), repository `/scratch/code/shibboleth/descriptor-mnemonic`, worktree `/scratch/code/shibboleth/wt-composer-s0`, branch `composer-s0`.
**Plan:** `/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` (R0 GREEN at mnemonic-engrave `3533638`).
**Lens:** implementation-introduced defects, by EXECUTION. Plan-vs-diff fidelity of the assembled files and of the six hand-applied fragments; Task 8's real blast radius measured against a binary built at the base revision; corpus reproducibility; false-PASS hunt by mutation; a journey walk on the shipped `md`.
**Read-only** on `descriptor-mnemonic`, the worktree (except its own `target/`, per the brief) and `mnemonic-engrave` except this report file. No `.jsonl` file read.

**What I built and ran** (full list at the end): a `md` binary at HEAD in the worktree and a second `md` at the base revision `b19dca7b` in a private `git archive` copy under the scratchpad, then compared the two over 8,129 templates and the whole shipped vector corpus; regenerated the 256-file corpus with `md vectors` into a temp dir and diffed it against the committed one; ran six mutations in a private copy of the worktree; walked the plan's journey plus 40 operator divergences on the shipped CLI; recomputed 44 BIP-380 checksums from the BIP text.

---

## VERDICT: 0C / 1I / 2M / 3N

No Critical. The lowering, the vectors and the corpus are sound and reproducible; every mutation I planted was caught; no reading verb refuses anything it read before; no pre-existing vector file changed. The one Important is a **blast-radius disclosure gap**: Task 8's gate lands on two shipped commands the plan and the CHANGELOG never name, and those two have no `--experimental` escape hatch.

---

## I-1 — Task 8 newly refuses `md descriptor --template` and `md address --template`, which have no `--experimental`; neither the plan nor the CHANGELOG names them

**Where:** `crates/md-cli/src/parse/template.rs:2703` and `:2732` (the `minting` guard), reached from `crates/md-cli/src/cmd/build.rs:65` (`build_descriptor` → `parse_template`, always `Disposition::Refuse`), which is the sole descriptor-construction route for `crates/md-cli/src/cmd/descriptor.rs:118` and `crates/md-cli/src/cmd/address.rs:38`. Record: `CHANGELOG.md:41-51`.

**Constructed input and output.** `T` = the exact template `md compose --experimental --wrapper wsh --path 2of3 --path keyless,sha256=<64 hex>,after=1383520` prints:

```
$ OLD=<md @ b19dca7b>;  NEW=<md @ 9820e618>
$ $OLD address --template "$T" --key @0=<xpub> --key @1=<xpub>
bc1q650nwn7f95xxy4qtxcxkjhcwexlawl3l76yp429pukey0z77l5psw3krq8
note: stdout is watch-only — public keys only, cannot spend

$ $NEW address --template "$T" --key @0=<xpub> --key @1=<xpub>
md: template parse error: miniscript parse failed: All spend paths must require a signature

$ $NEW address --experimental --template "$T" ...
error: unexpected argument '--experimental' found
```

`md descriptor --template` behaves identically (OLD printed the concrete descriptor with its `#0wxhr2rt` checksum; NEW refuses). `--experimental` exists on exactly three subcommands — `encode` (`main.rs:184`), `verify` (`:236`) and the new `compose` (`:292`) — so there is no way to ask `descriptor`/`address` for the shape that `md compose --experimental` and `md encode --experimental` will both happily produce and engrave.

**Why this is Important, not Critical.** No engraved plate becomes unreadable: I minted the card with `NEW encode --experimental` and read it back with `NEW descriptor <chunks>`, `NEW address <chunks>` and `NEW decode <chunks>` — all exit 0 with the right descriptor and the right address. `md verify` (the sole production `Warn` caller, `cmd/verify.rs:57`) is untouched: I ran `verify --template` + `inspect` over all 55 shipped corpus vectors under both binaries — **0 differences**, 10 of the 55 verifying green under both (so the differential is not vacuous), and directly on newly-refused shapes `verify` still returns `OK` exit 0. And none of the 55 corpus templates is newly refused on the minting side either (the corpus regenerates byte-identically through `md vectors`, which uses the same `Refuse` route). So the loss is confined to hand-written templates in the newly-refused classes reaching two watch-only derivation commands — a real, undocumented, unflagged refusal, but not funds-affecting and not a reading-verb regression.

The plan's Task 8 is titled and scoped "`md encode` gates a signature-free spend path"; the round-2 verification's caller table *did* mark `cmd/build.rs:65` "Yes — gated", so this was seen at plan time, but nothing propagated to the shipped record: the CHANGELOG entry names `md encode` in its heading and then names only `md verify`/`md inspect` as the verbs that keep working, which reads as "encode changed, reads did not". No test in the diff pins `descriptor`/`address` behaviour either way.

**Remedy (one sentence):** amend the CHANGELOG's Changed entry to name `md descriptor --template`, `md address --template` and `md vectors` as also gated and to state that the first two have no `--experimental` opt-out, or file a follow-up to add the flag to them.

---

## M-1 — The newly-enforced classes are not only the signature rule: malleability and mixed timelocks are now refused at every minting verb, and no flag relaxes them

**Where:** same site (`parse/template.rs:2732`, `d.sanity_check()`); record `CHANGELOG.md:41-51`.

**Measured.** Over a generated corpus of 7,980 templates (the repo's own generator shape from `format/text.rs:224-250`: every wrapper chain of length 1–3 over `c s a d j n v`, two key bases, five embeddings, under both `wsh(` and `sh(`), **30 templates flip from OLD exit 0 to NEW exit 1**, every one of them with:

```
md: template parse error: miniscript parse failed: Miniscript is malleable
```

e.g. `wsh(or_d(j:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))`. Two hand-built mixed-timelock shapes flip the same way:

```
$ $OLD encode "wsh(and_v(v:pk(@0/<0;1>/*),and_v(v:after(100),after(500000000))))"
md1yqpqqxpye55nxkcqqqqxgmrhxk2qq97ncfkuujc6tl          (exit 0)
$ $NEW encode "wsh(and_v(v:pk(@0/<0;1>/*),and_v(v:after(100),after(500000000))))"
md: template parse error: miniscript parse failed: Contains a combination of heightlock and timelock   (exit 1)
```

These two classes are *not* relaxed by `--experimental` (correctly — `ext_check(ExtParams::new().top_unsafe())` still enforces them, and the error says so verbatim). Both classes describe genuinely defective wallets (an unwitnessed malleation vector; an nLockTime that must be a height and a time at once), so refusing them at mint time is an improvement, not a regression — which is why this is Minor and not Important. But the CHANGELOG illustrates the change with the signature rule alone ("a spend path that requires no signature … It is now refused for `wsh`, `sh(wsh)` and `sh` too"), and lists malleability/resource-limits/timelock-mixing only in the parenthetical about what `--experimental` does *not* relax — which reads as "these were already enforced". Under `wsh`/`sh` they were not enforced at all before this diff. Control for the negative: the same differential over 149 real templates harvested from the base repo's own sources and corpus found **0** differences, with 108 of the 149 accepted by OLD, so the sweep does reach the gate.

**Remedy:** one clause in the CHANGELOG saying these classes become enforceable for `wsh`/`sh` for the first time and that no flag relaxes them.

## M-2 — `composed_templates_encode_and_round_trip_through_the_wire` guards its last assertion behind `if let Ok`, which is a false-PASS shape even though it currently runs

**Where:** `crates/md-codec/tests/compose_lowering.rs:469` (verbatim from the plan, Task 2).

```rust
if let Ok(s) = encode_md1_string(&c.descriptor) {
    assert!(s.starts_with("md1"));
}
```

If `encode_md1_string` ever returns `PayloadTooLongForSingleString` for this list — one extra path, one wider `n`, one origin-encoding change — the assertion silently stops running and the test still reports PASS. **I proved it currently runs** by replacing the guard with `.expect("PROBE-BRANCH-WAS-DEAD")` in a private copy: the test PASSED, so the branch is live today. Recording it because the shape is the one this project's own severity list calls out ("a test that reports a false PASS"), and because its trigger is a size threshold that a later stage will move. Minor because the guarded assertion is weak (a `"md1"` prefix) and the three lines above it — `encode_payload`, `split`, `reassemble` round-trip equality — are unguarded and carry the test's real weight.

**Remedy:** `let s = encode_md1_string(...).expect("a two-path wsh fits one string");`

---

## N-1 — `compose/mod.rs` module declarations relocated relative to the plan's text

`crates/md-codec/src/compose/mod.rs:29-30` puts `pub mod presets;` and `mod tr;` beside `mod lowering;` at the top; the plan's Task 3 and Task 7 blocks append them at the end of the file (the build gate's concatenation order). This is the **only** textual divergence in 11 assembled files. Harmless: verdict *harmless*, no behaviour or test-power change.

## N-2 — `md compose --help` shows `--json` with an empty description

`crates/md-cli/src/main.rs:291-292` (`#[arg(long)] json: bool`) carries no doc comment, so `--help` renders `--json` with a blank right-hand column. Verbatim from the plan; four other subcommands spell it the same way (`main.rs:172, 201, 248, 258`), so it is consistent, not new. Nit.

## N-3 — `md encode`'s refusal of a signature-free path names no way forward, while `md compose`'s does

Journey-visible asymmetry between two commands shipped in the same diff:

```
$ md compose --wrapper wsh --path 2of3 --path keyless,sha256=<H>,after=1383520
md: this policy needs --experimental:
  path 2 has no key (bearer access to whoever holds the preimage)

$ md encode "<that same template>"
md: template parse error: miniscript parse failed: All spend paths must require a signature
```

The second is rust-miniscript's raw string and mentions neither `--experimental` nor which path is at fault. It is pre-existing wording (this is exactly what `tr` said before the diff), and Task 8's whole point is that `wsh` now says it too — so the diff *spreads* a less helpful message to three more wrappers without making it worse. Classified **documentation-only**; Nit.

---

## Plan-vs-diff table

Method: re-ran the build gate's own extractor (same anchor grammar, `Create`/`Add to` append, `Replace` supersedes) over the plan into a scratch tree, ran `rustfmt --edition 2024` (repo pin 1.85.0, `rustfmt 1.8.0-stable`, no `rustfmt.toml` in the repo) on each extracted file, then byte-diffed against the worktree.

| file | verdict |
| --- | --- |
| `crates/md-codec/src/compose/mod.rs` | **divergent (cosmetic)** — `pub mod presets;` / `mod tr;` moved from EOF to line 29-30 beside `mod lowering;`. 8 diff lines, no other change. N-1. |
| `crates/md-codec/src/compose/lowering.rs` | identical (0 diff lines after rustfmt) |
| `crates/md-codec/src/compose/tr.rs` | identical |
| `crates/md-codec/src/compose/presets.rs` | identical |
| `crates/md-codec/tests/compose_lowering.rs` | identical |
| `crates/md-codec/tests/compose_support.rs` | identical |
| `crates/md-codec/tests/compose_crosscheck.rs` | identical |
| `crates/md-codec/tests/compose_vectors.rs` | identical |
| `crates/md-cli/src/cmd/compose.rs` | identical |
| `crates/md-cli/tests/cli_compose.rs` | identical |
| `crates/md-cli/tests/cli_compose_encode_gate.rs` | identical |
| `crates/md-codec/src/lib.rs` | fragment applied correctly — `pub mod compose;` on the line after `pub mod codex32;` (alphabetical position), exactly the plan's Task 1 text |
| `crates/md-codec/Cargo.toml` | fragment applied correctly — one line appended to `[dev-dependencies]` (section starts line 31; the `optional = true` copy is in `[dependencies]`, line 29, so **no duplicate key in one table**). `Cargo.lock` unchanged (0 files in `git diff --name-only -- Cargo.lock`), as the plan predicted |
| `crates/md-cli/src/cmd/mod.rs` | fragment applied correctly — `pub mod compose;` **unguarded**, directly after the `#[cfg(feature = "cli-compiler")] pub mod compile;` pair, so `compose` is unconditional as the plan and CHANGELOG claim |
| `crates/md-cli/src/error.rs` | fragment applied correctly — `Compose(String)` immediately after `BadArg(String)` with the plan's exact doc comment, and the `Display` arm `write!(f, "{m}")` immediately after `BadArg`'s. Verified live: a compose refusal exits **1**, a clap missing-arg exits **2** |
| `crates/md-cli/src/main.rs` | fragment applied correctly — `Compose { .. }` variant and dispatch arm, both directly after their `Compile` counterparts, doc comments and `#[arg]` attributes byte-identical. Only difference: the dispatch arm is `=> cmd::compose::run(...)` rather than the plan's `=> { cmd::compose::run(...) }` — rustfmt/clippy block elision, *harmless* |
| `crates/md-cli/src/parse/template.rs` | fragment applied correctly — the `ms_desc` construction replaced verbatim (`relaxed_err` generic helper, the `minting` guard on the `Wsh`/`Sh` arms only, the unguarded `Tr` arm preserving pre-existing behaviour, the `sanity_check()` behind `Disposition::Refuse`). Only difference: rustfmt braces around the three single-expression match arms. *changes behaviour* — but exactly the behaviour the plan specifies; see I-1/M-1 for the scope the plan's own prose understates |
| `crates/md-codec/src/test_vectors.rs` | fragment applied correctly — 4 `XPUB_JOURNEY_*` consts (byte-equal to `compose_support.rs::XPUB[0..3]`, checked) + 26 `Vector` entries, all `path: None`, both `*_distinct_fingerprints` entries carrying `[0x11;4] [0x22;4] [0x33;4] [0x44;4]`. Its doc claim "the same public keys the keyed entries above bind" is TRUE (the base file already contained `XPUB_JOURNEY_0` 14 times) |
| `CHANGELOG.md` | see I-1/M-1. Every countable claim is true: family 28, MANIFEST compose entries **26** (22 `keyed_compose_*`, 4 `compose_*`), the 2 absentees are exactly `compose_wsh_keyless_hash_only` / `compose_wsh_keyless_hash_path`, **22** `keyed_compose_*.conformance.json` on disk, `Cargo.lock` unchanged, `md-cli` still `0.14.0`, `md-codec` still `0.42.0` |
| `crates/md-codec/tests/vectors/*` (126 files) | **all 126 are additions.** `git diff --name-status b19dca7b..HEAD -- crates/md-codec/tests/vectors` → `126 A`, 0 `M`, 0 `D`; and 0 changed files whose name lacks `compose`. Confirms the implementer's "zero `differ` lines" |

**Known deviation checked for a missed consequence — `force_chunked: true` on all 26.** Not a divergence in outcome: the plan's rule is "false except where the exporter reports `PayloadTooLongForSingleString`", and every one of the 26 exceeds the limit. Measured: the largest single-string vector in the whole corpus is `single_string_boundary` at 101 hex chars of payload; the *smallest* compose vector, `keyed_compose_tr_key_path_only`, is 165 (`md encode` on that template with one key emits 3 chunks). So `false` would have made the exporter fail on all 26. No consequence missed; the compose family simply cannot exercise the single-string phrase form, which 8 pre-existing vectors still do.

---

## Task 8 caller table (checked against the DIFF, not the plan)

`parse_template` (`template.rs:2613`) hard-wires `Disposition::Refuse`; `parse_template_ext` takes it as a parameter. The diff changes only `parse_template_ext`; no caller was added, removed or re-dispositioned. Verdicts below are **measured** with OLD-vs-NEW binaries, not inferred.

| production caller | disposition | changed? | measured |
| --- | --- | --- | --- |
| `cmd/encode.rs:69` `md encode` | `Refuse` (explicit) | **Yes** | sigless `wsh`/`sh(wsh)`/`sh` OLD 0 → NEW 1; malleable OLD 0 → NEW 1; mixed timelocks OLD 0 → NEW 1; `--experimental` admits the sigless case with the loud bearer-access warning, refuses the other two with the "relaxes ONLY the signature rule" message. 149-template control sweep: 0 unintended flips |
| `cmd/verify.rs:52` `md verify` | `Warn` (explicit) | **No** | 55/55 corpus templates identical exit codes OLD vs NEW (10 green, 45 red identically); on a card carrying a sigless `wsh` path and on one carrying a malleable `or_d(j:pk,pk)`, NEW `verify --template` returns `OK` exit 0. The `minting` guard is doing its job |
| `cmd/build.rs:65` `build_descriptor`, used by `cmd/descriptor.rs:118` (`md descriptor --template`) and `cmd/address.rs:38` (`md address --template`) | `Refuse` (via `parse_template`) | **Yes — and this is I-1** | OLD prints descriptor/address, NEW refuses; no `--experimental` on either subcommand. Card-input route (`md descriptor <chunks>`, `md address <chunks>`) unaffected — verified exit 0 with the correct `bc1q650n…` address |
| `cmd/vectors.rs:54` exporter | `Refuse` (via `parse_template`) | **Yes** | This is why the two keyless-wsh vectors are `no-corpus`; disclosed in the CHANGELOG. The 26 exported vectors all pass the gate — the corpus regenerates byte-identically |
| `decompose/mod.rs:478` `md decompose` | `Refuse` (via `parse_template`) | **text only** | The `Err` becomes an advisory `notes` line, never a returned error; `decompose` cannot fail because of the gate. OLD/NEW exit codes identical on the shapes I tried |
| `compile.rs:139`, `format/text.rs` (13 sites), `format/json.rs:409` | `Refuse` | test-only | all inside `#[cfg(test)] mod tests`; no production path |

The `Tr` arm in the `--experimental` branch is deliberately **not** `minting`-guarded, preserving pre-existing behaviour exactly (`from_str`'s tr-only gate already ran unconditionally in the non-experimental branch). No `tr` behaviour changed: confirmed in the 7,980-shape sweep and the corpus differential.

---

## Journey walk

Shipped binary, built from the worktree at `9820e618`. Divergences classified per the standing method.

**The plan's journey — clean.**

```
$ md compose --wrapper tr --path 2of3 --path 1of1,older=26280
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/48'/0'/1'/3'/<0;1>/*,@2/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@3/48'/0'/3'/3'/<0;1>/*),older(26280))})
[stderr] note: stdout is a keyless descriptor template (no keys)     exit 0
$ md encode "<that>"                                                  exit 0, one md1 string
$ md decode <that string>                                             exit 0, template + the four origins
```

Stream discipline is right: the template alone is on stdout (verified with `2>/dev/null`), notes and the EXPERIMENTAL warning on stderr, so `md compose … | md encode` is safe. The NUMS constant `50929b74…03ac0` matches the pre-existing `nums_taproot` corpus vector byte for byte.

**The EXPERIMENTAL gate — coherent one command deep.**

```
$ md compose --wrapper wsh --path 2of3 --path keyless,sha256=<H>,after=1383520
md: this policy needs --experimental:
  path 2 has no key (bearer access to whoever holds the preimage)          exit 1
$ md compose --experimental …
warning: EXPERIMENTAL: path 2 has no key (bearer access …)                 exit 0, template on stdout
$ md encode "<that template>"
md: template parse error: miniscript parse failed: All spend paths must require a signature   exit 1
$ md encode --experimental "<that template>"
warning: --experimental relaxed the signature rule. … THE PLATE IS BEARER ACCESS. …           exit 0, 2 chunks
```

This is the defect Task 8 exists to close, and it is closed. I also swept the other direction: **54 composed templates** across `{tr, wsh, sh-wsh, sh} × {1of1, 2of3, 3of5, 9of9} × 6 second-path variants` all encode at exit 0 — `md compose` never emits something unflagged that `md encode` then refuses.

**What ELSE an operator might type** (40 invocations; every refusal below is exit 1 with a named cause, and the compose refusals name the offending path by ordinal):

| divergence | outcome | class |
| --- | --- | --- |
| `--wrapper WSH` / `sh_wsh` / `wpkh` | `md: --wrapper WSH: expected tr, wsh, sh-wsh or sh` | refusal (good) |
| omit `--wrapper` or `--path` | clap usage, exit **2** | refusal (good) |
| `--path 0of3` / `4of3` / `2of10` | `path 1: 4-of-3 is not admitted (1 <= k <= n <= 9)` | refusal |
| `--path "2 of 3"` | ``path `2 of 3`: k `2 ` is not a small number`` | refusal |
| `--path 2of3,older=0` / `older=65536` / `after=0` | band messages quoting the band and the value | refusal |
| `--path 2of3,older=100,after=100` | `at most one lock per path` | refusal |
| `--path 2of3,unsorted` | `needs --experimental: … key order is part of this wallet` | refusal → flag |
| `--path keyless,sha256=deadbeef` | `sha256 needs 64 hex characters, lowercase` | refusal |
| `--wrapper sh --path 2of3 --path 1of1,older=100` | `legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr` | refusal (names the way out) |
| `--path 2of3 --path 2of3` (duplicate) | accepted → `or_d(multi(2,@0..@2),multi(2,@3..@5))`, six distinct slots at six distinct accounts | **default** — legal, two independent quorums; not our concern |
| `--path 1of1,sha256=<H>` on a *keyed* path | accepted → `and_v(v:pkh(@3/…),sha256(H))` | default, matches §5 with no lock |
| `older=100u` / `after=…t` unit forms | `older(4194404)` / `after(1893456000)` | default (documented in `--help`) |
| 23 malformed `--path` strings incl. `""`, `","`, `"of"`, `older=999999999999999999999` | no panic, no exit > 2 | refusal |
| `md compose --help` | names all four wrappers and the full DSL; `--json` has a blank description | **documentation-only** (N-2) |
| `md encode` refusal wording vs `md compose`'s | rust-miniscript's raw string, no pointer to `--experimental` | **documentation-only** (N-3) |
| `md address --template <experimental shape>` | refused, no flag to get past it | **refusal that should be a flag** — I-1 |

---

## Attacks tried that found nothing

- **A pre-existing vector or template newly refused** (the plan's own Task 8 STOP clause). 149 real templates harvested from the base repo's sources and corpus: 0 flips, 108/149 accepted by OLD. All 55 shipped corpus templates: 0 flips on `verify`, 0 on `inspect`, and the whole corpus re-exports byte-identically through the gated `md vectors`.
- **A reading verb refusing an engraved plate** (the Critical class). Minted plates under OLD carrying (a) a signature-free `wsh` path, (b) a mixed-timelock `wsh`, (c) a malleable `or_d(j:pk,pk)`; then `NEW decode`, `NEW inspect`, `NEW verify --template` on each — all read them, `verify` returning `OK` exit 0.
- **A pre-existing vector file mutated by the regeneration.** `git diff --name-status` over `tests/vectors`: 126 `A`, zero `M`/`D`.
- **The corpus not being what the tool emits.** `md vectors --out <tmp>` then `diff -rq` against the committed directory: zero content differences; the only entries not reproduced are `bip341-wallet-test-vectors.json` and `.gitkeep`, neither of which the exporter writes.
- **A wrong value inside a `.conformance.json`.** Recomputed all **44** BIP-380 descriptor checksums (22 files × 2 chains) with a Python implementation written from the BIP text, independent of md's code: 0 bad. The `*_distinct_fingerprints` vector exports `11111111 22222222 33333333 44444444`, i.e. the fingerprints really reach the corpus.
- **Vacuous or self-satisfying tests.** Grepped the 61 delivered tests for `if let Ok` around assertions, `is_ok()` as the whole assertion, self-equality and empty loops. Four hits, three benign (`is_ok()` on a preset constructor with a hand-traced positive case; `insane.lift().is_ok()` after an `expect_err` on the same input two lines above). The fourth is M-2.
- **A cross-check that cannot fail.** Four mutations to the production lowering in a private copy, each reverted:
  - swap `Tag::OrD`/`Tag::OrI` → `every_family_entry_passes_the_5b_cross_check` **and** `every_preset_passes_the_5b_cross_check` FAIL;
  - left taproot spine instead of right → `every_family_entry_renders_as_listed` + `every_compose_vector_in_the_manifest_is_exactly_what_compose_renders` FAIL;
  - internal key = *last* bare single instead of first → `only_the_first_listed_unlocked_single_key_is_extracted` FAIL;
  - drop the internal key from the first-appearance numbering → the MANIFEST comparison **and** the family cross-check FAIL.
  Baseline and post-revert both 52/52 PASS.
- **`the_cross_check_notices_a_wrong_lowering` being a tautology.** Turned its own `replacen("thresh(2,", "thresh(1,", 1)` into a no-op (`"NOPE(2,"`): the test FAILS. It is a real negative control, and it is self-guarding — if `concrete_policy` ever stops emitting `thresh(2,`, the `assert_ne!` goes red rather than silently passing.
- **A duplicate `miniscript` key in one Cargo table**, a version bump, or a `Cargo.lock` change. None: `[dependencies]` line 29 vs `[dev-dependencies]` line 34; `md-cli 0.14.0` and `md-codec 0.42.0` unchanged at both revisions; `Cargo.lock` untouched.
- **A panic reachable from the CLI.** 23 malformed `--path` strings plus 40 journey invocations: no panic, no exit code above 2.
- **`md compose` emitting something `md encode` refuses.** 54 compose→encode round trips, 0 refusals.

**Residual gap I did not close:** I verified the corpus addresses are *reproducible* and internally consistent (regeneration + §5b cross-check against rust-miniscript's compiler + independent BIP-380 checksums), but I did not re-derive any address with a tool outside the rust-bitcoin/rust-miniscript stack. A shared upstream error in that stack would be invisible to everything I ran.

---

## What I ran

```sh
# fidelity
git -C wt-composer-s0 log --oneline b19dca7b..HEAD; git diff --stat/--name-status b19dca7b..HEAD
python3 <extractor copied from scripts/plan-build-gate-md.sh> <plan> <scratch>   # same anchor grammar
rustfmt --edition 2024 <each extracted file>; diff -u <extracted> <worktree>     # 10/11 zero-diff
git diff b19dca7b..HEAD -- main.rs error.rs cmd/mod.rs lib.rs Cargo.toml parse/template.rs test_vectors.rs CHANGELOG.md

# two binaries
cargo build --locked -p md-cli --bin md                                   # HEAD, in the worktree
git archive b19dca7b | tar -x -C <scratch>/base-copy
CARGO_TARGET_DIR=/scratch/code/shibboleth/.plan-gate-target-exec cargo build --locked -p md-cli --bin md

# differentials (OLD vs NEW)
149 templates grepped from the base repo's own sources + corpus  -> md encode      (0 flips, 108 accepted by OLD)
7,980 generated wrapper-chain templates, wsh and sh, xargs -P 24 -> md encode      (30 flips, all "Miniscript is malleable")
21 hand-built adversarial shapes                                  -> md encode      (5 flips: 2 timelock-mix, 3 sigless)
55 shipped corpus templates x {verify --template, inspect}                          (0 flips; 10 verify green)
md descriptor/--template, md address/--template, md decompose, card-input routes
54 compose -> encode round trips across 4 wrappers x 4 quorums x 6 second paths

# corpus
md vectors --out <tmp>; diff -rq <tmp> crates/md-codec/tests/vectors               # zero content differences
python3 (BIP-380 checksum from the BIP text) over 22 conformance files             # 44/44 good
python3 count of MANIFEST/family/conformance entries; wc -c on *.bytes.hex

# mutation (private copy: git archive HEAD -> <scratch>/exec-review-copy,
#           CARGO_TARGET_DIR=/scratch/code/shibboleth/.plan-gate-target-exec)
cargo nextest run --locked -p md-codec -E 'binary(/^compose_/)' --no-fail-fast     # baseline 52/52
  MUT-A or_d<->or_i | MUT-B left spine | MUT-C rposition internal key
  MUT-D numbering drops the internal key | MUT-E replacen no-op | MUT-F if-let-Ok probe
cargo nextest run --locked -p md-codec -E 'binary(/^compose_/)'                    # restored 52/52

# journey
md compose (40 invocations incl. 23 malformed --path), md compose --help, --json,
md encode / decode / verify / inspect / descriptor / address, with and without --experimental
```

Did not read any `.jsonl` file. Did not re-run the workspace gate (`fmt`/`clippy`/`nextest 1318`) — accepted as already settled per the brief. Nothing in `descriptor-mnemonic`, the worktree (beyond its own `target/`) or `mnemonic-engrave` was modified; all mutations were made in `<scratchpad>/exec-review-copy` and reverted there.
