# F-449 spec r0 — findings by the author, before reviewers returned

Recorded so they survive compaction and so `git diff` over the fold shows them
alongside the reviewers'. Author-found findings are NOT a substitute for the
independent gate.

## SELF-1 (Minor, strengthening) — §8 criterion 3 understates the evidence

§8 criterion 3 says addresses must "agree between `md` and the evidence's
recorded Liana addresses". Measured: the evidence records **Liana's own derived
receive and change addresses, indices 0..2**, for every accepted shape. Example,
`preset-kofn-recovery-tr` at kind 1:

    receive[0] = bc1pj6davmeetfe2uutrjlgxcjytq50ggvyd82xtxx2eazm24hsdhlnqxvh4v4
    change[0]  = bc1pa656rnpy2dqf6gevkxw72fz7kx02zpmru507nft9cft8l0dru3vqejq0yf

So the acceptance test is a genuine **cross-implementation** address check
(md vs Liana) without building Liana, not merely an md-vs-md consistency check.
The spec should say so — it is a stronger gate than the text claims.

## SELF-2 (Important) — §6's "no leaf keys" refusal is UNREACHABLE

§6 row 1 requires REFUSE when `kind = 1` on a `tr` whose tree has no leaf keys,
reasoning that `sha256("")` is a constant shared by every such wallet.

Measured: that state cannot be constructed.

    $ md encode "tr(50929b74...e803ac0)"
    md: template parse error: template contains no @i placeholders
    $ md encode "tr(50929b74...e803ac0/<0;1>/*)"
    md: template parse error: template contains no @i placeholders

The parser requires at least one `@i` placeholder anywhere in the template.
Under `tr` with a NUMS-family internal key the internal key is not a
placeholder, so every placeholder must live in a leaf — therefore the leaf-key
concatenation is non-empty by construction.

Why this is Important and not a Nit: **a refusal that cannot fire is not a
guard.** It reads as safety, it will be "covered" by a test that passes
vacuously, and this project has a standing rule that a gate which cannot fail is
a blocking defect. The remedy direction (author's call, not prescribed here) is
to state it as a pinned invariant with a test that demonstrates WHY it is
unreachable, rather than as a refusal — so that if the placeholder rule ever
changes, the pin fails instead of silently admitting an empty concat.

Note the invariant is load-bearing: if it ever broke, every affected wallet
would share one internal key derived from `sha256("")`.
