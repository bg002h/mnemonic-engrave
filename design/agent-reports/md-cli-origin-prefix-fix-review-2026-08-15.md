# Adversarial review: md-cli origin-prefix refusal (commit 11b01a9e)

This is an agent's advisory review; it carries no authority beyond its evidence.

- **Repo:** `/scratch/code/shibboleth/descriptor-mnemonic`, commit `11b01a9e` on `main` (parent `5a0a4f41` = `md-cli-v0.13.0`), not pushed. Tree left clean (verified `git status --porcelain` empty after all probes; every mutation reverted via `git checkout`).
- **The one question:** does the commit refuse or mis-encode anything previously encoded correctly, and is refusing the bracketed form right?
- **Answer:** no regression found — every input the new check fires on already failed at the parent (proven by A/B against a parent-commit binary built from `git archive`), and the correct-syntax control output is byte-identical parent-vs-HEAD. **REFUSE is the right call** (reasons at the end). One **Important** finding: the funds-attached test cannot see a swapped origin-to-placeholder mapping — proven by a mutation that passes the whole md-cli suite.

## Findings

### IMPORTANT

**I1 — the round-trip test does not pin the origin→placeholder MAPPING.**
`crates/md-cli/tests/cli_divergent_origin_encode.rs:59` (`divergent_origins_survive_a_decode_round_trip`, asserts at ~93–103).
Concrete failure: the test proves both origins are present and the tag is `Divergent`, but a build that assigns @0's origin to @1 and vice versa — the same funds class the test exists to catch (each cosigner recorded against the wrong account) — passes it, and passes everything else.
Evidence RUN: mutated `make_path_decl` (`crates/md-cli/src/parse/template.rs:539`) to `(0..n).rev()` so the `Divergent` vec is built in reverse; then `cargo test -q -p md-cli` → **true exit 0, 29 suites ok, 0 FAILED** (output preserved in scratchpad `m2-full.out`). The mutated line demonstrably ran: the companion flatten mutation (`all_same = true`) on the same function made the test FAIL, so the code path is exercised.
Minimal fix: the decode JSON exposes the ordered array — `"path_decl": {"data": ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"], "tag": "Divergent"}` — so assert order, e.g. `text.find("48'/0'/0'/2'").unwrap() < text.find("48'/0'/1'/2'").unwrap()` (positions in the `data` array), or parse the JSON and compare `path_decl.data` exactly.

### MINOR

**M1 — hardcoded guidance path is wrong for non-BIP-48 contexts.**
`crates/md-cli/src/parse/template.rs:86-89`. The message instructs `write @{i}/48'/0'/0'/2'/<0;1>/*` verbatim in every context. Evidence RUN: `md encode "wpkh([73c5da0a/84h/0h/0h]@0/*)"` → refusal suggesting the BIP-48 multisig origin + multipath for a single-sig template whose real origin was 84h/0h/0h; same for a `tr()` internal key (86h path). It reads as an instruction, not an example. Minimal fix: echo the user's own bracketed path (strip the fingerprint segment, map `h`→`'`) into the suggestion, or prefix the path with "e.g.".

**M2 — the fingerprint half of the bracket has an md expression the message never names.**
Same site. `[73c5da0a/48h/...]` carries a fingerprint; md models it separately (`--fingerprint @i=HEX`, fingerprints TLV — `parse_template` at `template.rs:2208-2214`). A user following the guidance rewrites the origin and silently loses the fp from their record. Minimal fix: append "pass the fingerprint as `--fingerprint @{i}=<hex>`" to the message.

**M3 — `md compile` emits a template `md encode` refuses (pre-existing, adjacent).**
`crates/md-cli/src/compile.rs:65` (`Policy<String>` accepts any key string). Evidence RUN (built with `--features cli-compiler`): `md compile 'thresh(2,pk([73c5da0a/48h]@0),pk(@1),pk(@2))' --context segwitv0` → **exit 0**, prints `wsh(multi(2,[73c5da0a/48h]@0,@1,@2))`; feeding that to `md encode` → the new refusal, exit 1. Not introduced by this commit (parent's encode also failed on it, with the internal error), and no wrong output can reach steel — but md now refuses its own compile output as "not md syntax". File a follow-up: compile should refuse or translate bracketed policy keys.

### NIT

**N1 — dead disjunct in the refusal test.** `cli_divergent_origin_encode.rs:120` — `stderr.contains("after the placeholder")` can never be true (message says "AFTER the placeholder", uppercase); only `contains("@0/48'")` carries the assertion. Harmless; align the case.

**N2 — whitespace between `]` and `@` escapes the check.** Evidence RUN: `md encode "wpkh([73c5da0a/84h/0h/0h] @0/*)"` → falls through to `miniscript parse failed: base58 encoding error`, exit 1. Same outcome at parent, so not a regression; optionally widen the regex to `\]\s*@(\d+)` to give the good message.

## Clean probes (what I ran, all with true exit codes, never through a pipe)

1. **Parent A/B on the firing class** (parent binary built from `git archive 5a0a4f41` in the scratchpad; repo untouched). All six already **failed at parent, exit 1**: both-keys bracketed sortedmulti (the internal `synthetic key [73c5da0a not found` error), fingerprint-only `wpkh([73c5da0a]@0/*)`, `tr([fp/86h/0h/0h]@0/<0;1>/*)` internal key, `tr(NUMS,multi_a(2,[fp/48h]@0/…,@1/…))` script-path key, bracket on the second key only, empty bracket `[]@0` (parent: miniscript "master fingerprint should be 8 characters"). **Nothing that previously encoded is now refused.**
2. **Control byte-identity:** `md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" --force-chunked` — exit 0 at both, `diff` of outputs empty.
3. **No misfire on bracket-free templates:** `wpkh(@0/*)`-family lexer tests, `tr(@0/<0;1>/*,{pk(@1…),pk(@2…)})` exit 0, `tr(<NUMS hex>,multi_a(2,@0…,@1…))` exit 0.
4. **Literal keys (probe 2):** md rejects literal xpub cosigners regardless of the commit — `wsh(multi(2,XPUB/<0;1>/*,@0/<0;1>/*))` → exit 1 at HEAD with the pre-existing "internal: lexer/substitution divergence" error. `[origin]XPUB` (literal with origin) does NOT trigger the new check (`]x`, not `]@`) and fails identically at HEAD and parent with the old internal error. So there is no legitimate bracketed-origin input for the check to break. The `tr()` NUMS hex literal internal key still works.
5. **Grammar + corpus sweep:** `[` has no role in the template grammar outside an origin prefix (tap trees use `{}`, hash args are hex, checksums are bech32 and stripped by compile); repo-wide grep for `]@` matches only this commit's code/tests and two design docs — no fixture, corpus, or built-in `vectors.rs` template fires the check.
6. **Where the check does not run:** `--key`/`--fingerprint` values go through `parse::keys::parse_key` (no `lex_placeholders`); a bracketed `--key '@0=[fp/…]xpub…'` fails loudly (`base58check decode`, exit 1) — never silent. `md compile` policy input bypasses the check (M3). `decode`/`inspect`/`partial`/`repair` take no template. `md verify --template <bracketed>` with a **valid** md1 string surfaces the new refusal (with an invalid string the codec error fires first, which is fine).
7. **Round trip:** encode of the correct divergent syntax decodes (`--json`) to `path_decl {"data": ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'"], "tag": "Divergent"}`, matching the author's claim.
8. **Mutation coverage of the new tests:** flatten (`all_same = true`) → round-trip test FAILS (caught); refusal block disabled (`if false`) → bracketed test FAILS via its `internal:` assert (caught); swap (`.rev()`) → all green (finding I1). All mutations reverted; final pristine-HEAD run of the new file: 3 passed, 0 failed.

## Pre-existing observation (not this commit, worth a follow-up)

`md decode` **text** output omits origins entirely: the P1 card carrying divergent origins decodes to `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))` with only the "keyless (no keys)" note — identical at parent (verified with the parent binary), and `md-codec/src/render.rs` has no origin handling at all. A user round-tripping through text output silently loses origins — the same silent-drop class this commit fights, one command over. It also means the trap-experiment's *text-render* evidence ("decode rendered no origins") was non-probative on its own; the author's `path_decl`-empty JSON check was the load-bearing evidence and stands.

## Refuse vs support: REFUSE is right

1. **The trap is structural, not incidental.** `path_decl` derives solely from the lexer's group-2 capture (`template.rs:98-111` → `make_path_decl:530`); the bracket content never reaches it. Any accept-by-stripping in `lookup_key` therefore loses the origins by construction — confirming the author's verified silent-drop, independently of their run.
2. **The bracketed form is not alternative syntax for the same datum.** It carries a fingerprint md models separately (fingerprints TLV, `--fingerprint @i=HEX`). Genuine support needs a design: bracket-origin vs post-placeholder-origin conflict on the same @i, bracket-fp vs `--fingerprint` conflict, `h`/`'` normalization, same-@i occurrences with differing brackets. That is a feature, not the lesser bug fix.
3. **BIP-388 alignment.** In wallet policies the `[fp/path]` prefix belongs to the key-information vector, never the template — a bracketed template is invalid BIP-388 too. md's key-information surface (`--key`) already refuses it loudly.
4. Support *could* be built correctly later (map bracket path → origin, bracket fp → fingerprints TLV, with the conflict rules above); refusing today forecloses nothing and converts a misleading `internal:` error into an actionable one. If demand exists, file it as a feature follow-up.

## Verdict

**I1 (Important) blocks until the mapping assertion lands — a two-line test edit; the shipped production code is correct as-is** (the in-order `(0..n)` construction at `template.rs:539-541` is right; only the test's discriminating power is short). No Critical. Refusal direction affirmed.
