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

## SELF-3 (Important) — §3f bundles a 146-site refactor with the wire change

§3f proposes replacing `Body::Tr`'s `is_nums: bool` + `key_index: u8` with an
`InternalKey` sum type, justified as retiring the `debug_assert!` at
`tree.rs:148` "by construction".

Measured blast radius:

| surface | occurrences | files |
| --- | --- | --- |
| `is_nums` in `md-codec/src/` (Rust primary) | **88** | 14 |
| NUMS references in the fork's `md/` + `gui/` (Go port) | **58** | 8+ |

The refactor is defensible on its merits. The problem is **bundling**: a
mechanical ~146-site rename landing in the same diff as a semantic wire change
to a funds-critical codec produces a diff nobody can review meaningfully, and
the wire change — the part that can lose money — hides inside the noise. This
project's own rule is that a bundled commit costs a future reviewer the one
diff that matters.

Direction (author's call): stage it as **two commits, refactor first**. A
behaviour-preserving `is_nums`/`key_index` → `InternalKey` change with the wire
format untouched and the full suite green, and only then the version-5 kind bit
on top. `git diff` over the second commit is then exactly the wire change.

This also de-risks the Go port, where the same split applies and where the
Rust-primary rule means the port cannot lead either half.

## SELF-4 (Important) — §4 emits a literal xpub that `md` cannot read back

§4 rules that at kind 1 the keyed descriptor renders "the **derived literal
xpub**". The spec never says whether `md` must ACCEPT one on input. Measured, it
does not, and it fails badly:

    $ md encode "tr(xpub661MyMwAqRbcFswVugWF.../<0;1>/*,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})" --path bip48
    md: template parse error: internal: synthetic key xpub661MyMwAqRbcFswVugWF... not found in key map
        (rendered: xpub661MyMwAqRbcFswVugWF.../<0;1>/*)

Two distinct defects:

1. **An internal invariant leaks as a user-facing parse error.** "internal:
   synthetic key ... not found in key map" is not a refusal anybody can act on.
   Whatever the scope ruling, this string should never reach a user.

2. **The round trip is open.** The spec's §8 criterion 4 requires "encode →
   decode → render is byte-stable", but for kind 1 the *entry* to that loop does
   not exist — md can write a descriptor it cannot read. This project's own
   record is emphatic that a round trip which was never run is not a property
   ("Success is not round-trip"; "A round trip is not a restore test").

Why it is worth more than a refusal, and why I think it changes the design: if
`md encode` RECOGNISES the form — recompute §2's recipe over the parsed leaf
keys and accept the descriptor as kind 1 **only when the xpub matches** — then
the recipe stops being something md merely emits and becomes something md can
**verify**. That yields two things the spec does not currently have:

- an operator can hand md an existing Liana taproot descriptor and engrave it,
  which is the natural other half of "let a user make a Liana-compatible wallet";
- any xpub in that position which is NOT the recipe's output gets caught, rather
  than being silently treated as a spendable key — which is exactly the failure
  mode lens 5 measured on Liana's side (`InvalidKey`, because a non-recipe xpub
  has no origin).

The author must either take that scope or rule it out explicitly and replace the
internal error with a real refusal. Silence is the one option that is wrong.
