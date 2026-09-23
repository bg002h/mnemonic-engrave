# RECON — F-449 stage 2, measured against main at `25acb33c`

Written 2026-09-22, immediately after stage 1b merged. **SPEC §9's stage-2
row was written before 1b executed, and three of its four items already
landed.** Everything below is measured, not read.

## What §9 assigns to stage 2, and what is actually left

| §9 item | state | evidence |
| --- | --- | --- |
| `md compose --unspendable liana\|nums` (default `nums`) | **ABSENT — the stage's real work** | `md compose --help` has no such flag |
| `md descriptor` kind 1 | **DONE in 1b** | `crates/md-cli/tests/liana_kind1_cmd_descriptor.rs`, 2 tests |
| the `UNSPENDABLE(liana)` template substitution rule | **DONE in 1b** | Tasks 5/6; `md encode 'tr(UNSPENDABLE(liana),…)'` works |
| JSON schema version bump (§4a) | **DONE in 1b** | `docs/json-schema-v1.md` documents `md-cli/2` |

So stage 2 is **one new flag plus its acceptance evidence**, not four features.

## The live Liana gate (§8.8) RUNS TODAY

The harness at `harnesses/liana` builds against a **v15.0** checkout at
`/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana` and calls
`LianaDescriptor::from_str` — the same entry point Liana's GUI uses behind
*"Import the wallet"*.

**Measured end to end on main:** `preset-kofn-recovery-tr` round-trips
through `md decompose --emit descriptor` **byte-exact including checksum**,
and Liana then **ACCEPTS** that exact string and returns receive/change
addresses. This is the first time md's own output has been fed to Liana's
importer as a single unbroken chain.

## F-640 — RESOLVED: a nested taptree IS accepted; §8.1 is satisfiable

§8.1 asks for "a nested taptree that Liana ACCEPTS". Two independent
constructions are now refused:

- `preset-decaying-multisig-tr` (the corpus's only nested case)
- a fresh `{multi_a(2,A,B),{and_v(older),and_v(older)}}` built from the
  **real keys of an accepted case**, so key validity is not the variable

Both give *"Descriptor is not compatible with a Liana spending policy."* — a
POLICY-SHAPE refusal, not a parse error. A control (`preset-tiered-recovery-tr`)
imported successfully in the same run, proving the harness and the keys good.

**Care needed:** an earlier attempt of mine refused with *"Error while parsing
xkey"* because I typed an xpub from memory. That is a PARSE refusal and says
nothing about policy. Any future probe must include a known-good control in
the same run, as this one did.

**CORRECTION, 2026-09-22, from the stage-2 R0 review (C-3).** The inference
above — two nested refusals, therefore nesting is the cause — is **unsound**,
and I verified why against Liana's own source rather than taking the reviewer's
word: `liana/src/descriptors/analysis.rs:592-598` calls `tree.lift()` to lift
the taptree into a SEMANTIC POLICY before comparing anything. Tree *shape* is
flattened away. Liana's policy model cannot see nesting at all, so nesting
cannot be what either refusal was about — both must be explained by LEAF
CONTENT, which is also consistent with the corpus, where `hashlock-gated` and
`X20-tr-hashlock-known` are refused at depth 1.

I have since failed to construct an accepted nested taptree three times, each
with a control passing in the same run (the third used two recovery branches
at ONE timelock, so it was not a decaying policy either). That is evidence
about my constructions, not about Liana.

**Status: UNSETTLED.** Whether a nested ACCEPT is constructible is open. What
is now established is the mechanism, which means §8.1's requirement is not
obviously unobtainable and the spec should NOT be amended on the strength of
my refusals. Stage 2 settles it by searching the policy space, not by
inferring from two failures.

**The lesson, recorded because it is mine:** three constructions that all
failed shared a property I never varied, and I read the shared failure as
evidence about the dimension I *had* varied. A negative is only as wide as
what you searched — and here the mechanism was one file away and would have
told me the dimension I was varying was invisible to the thing under test.

## Carried into stage 2

- **F-636, F-638, F-641** — one class: the refusal is right, the explanation
  names the wrong thing.
- **F-640** — the above.
- **C3's evidence legs** — descriptor- and address-equality against the four
  ACCEPT shapes, deferred from 1b Task 8 because they need md-cli. The
  harness is the oracle and it now runs.
- **§6 row 3** — `--unspendable liana` on a path list that already has a bare
  single key must WARN, not silently no-op (fable M-6).


---

## F-640 RESOLVED — 2026-09-22, after the stage-2 R0 review (C-3)

**Liana v15.0 ACCEPTS a nested taptree.** Reproduced by me independently of
the reviewer, with the flat control accepted in the same run:

```
B-nested-verify  ACCEPTED  recovery=[{older 26280, single}, {older 52560, single}]
CONTROL-flat     ACCEPTED  recovery=[{older 26280, single}]
```

The accepted shape is `{multi_a(2,A,B),{and_v(pk,older(26280)),and_v(pk,older(52560))}}`
— genuinely depth 2 — and Liana infers TWO recovery paths from it. **§8.1 is
satisfiable and F-640 closes by vectoring this case**, not by amending the
spec. Full descriptors in `design/agent-reports/f449-plan-stage2-r0.md`
Appendix A.

### Why all three of my probes failed, and why the control did not catch it

The variable was never the tree. It was **the internal key**.

`analysis.rs:596-600` recomputes the unspendable xpub **from the descriptor's
own leaves** and compares it to the one present. On mismatch it does not
refuse directly — it falls through to `or(Key(internal_key), tree)`, which
yields two non-timelocked paths and therefore `IncompatibleDesc`. So **three
distinct causes emit one identical message**:

| cause | example | message |
| --- | --- | --- |
| genuine policy refusal | two recoveries at ONE timelock (`analysis.rs:659`) | *not compatible with a Liana spending policy* |
| internal key ≠ recipe over THESE leaves | my probe 2 | the same string |
| internal key is the raw NUMS point | reviewer's probe F | the same string |

My probe 2 took "real keys from an accepted case" — carrying the accepted
case's **internal key** along with them while changing the **leaf set**. The
recipe then no longer matched, so it refused for a KEY reason that reads
exactly like a POLICY reason.

### The lesson: a control is not sufficient when it carries its own correctness

This is the sharper half, and it generalises beyond Liana. My control passed
in every run — because a control carries its **own** correct internal key.
Control-passes-probe-fails therefore proved nothing about the dimension I
thought I was testing.

**A probe needs its own validity rule, not just a control.** For this harness:
recompute the internal key for the probe's exact leaf set and assert it
before sending. Stage 2's gate must enforce that, or the next person repeats
this with a clean-looking control beside them.

See [[negatives-inherit-the-search-scope]] — and note this is the stronger
form: the negative inherited not the search scope but an *unvaried
precondition* that the error message actively disguised.
