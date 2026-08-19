# Recon: real nested-taproot-tree limit vs. the fork's depth-≥2 EXPERIMENTAL gate

**Scope:** ONE question — what actually limits nested taproot trees, and does
`gui/template_engrave.go:86-91`'s "EXPERIMENTAL: taproot depth >= 2 ... PR #953"
gate still have a factual basis now that PR #953 has been merged upstream.
Read-only recon; no design proposed, no code edited except a temporary scratch
example (deleted, verified clean) and a scratch probe crate in this agent's own
scratchpad (not part of any tracked repo).

---

## Verdict

The wire-level cap is **BIP-341's own limit of 128** (`TAPROOT_CONTROL_MAX_NODE_COUNT`,
already settled — not re-derived here), and nothing in this project comes close
to it. PR #953 has indeed been **merged to `rust-bitcoin/rust-miniscript` master**
(commit `ff4732e`, 2026-05-25), but is **NOT in any released version**, including
the newest one on crates.io, **13.1.0** (released 2026-06-09) — verified by
`git merge-base --is-ancestor ff4732e <tag>` against `13.0.0`, `miniscript-13.0.1`,
and `miniscript-13.1.0`: all three say NO; only `upstream/master` says YES. So the
fork's exact wording, "unreleased rust-miniscript >13.1.0", is **still literally
true today (2026-08-18)**. I additionally **reproduced the underlying bug live**
against the vendored 13.0.0 tree: a depth-2 **balanced** 4-leaf `tr()` script
tree — the exact shape this project's own `tap_four_leaf_balanced_round_trips`
test and `md address` both use — fails to round-trip through rust-miniscript's
own `Descriptor::to_string()`/`from_str()` with `taptree branch must have 2
children, but found 1`. So the gate's premise is not just unretracted, it is
freshly confirmed against the exact pinned dependency this project ships.

One real refinement, not a reason to remove the gate: **depth is a proxy, not
the true trigger.** The bug fires on tree *topology* (≥2 sibling-pairs sharing
one absolute depth across different subtrees), not depth per se — a depth-3
**right-skewed chain** (also tested below) round-trips through rust-miniscript's
Display just fine, while some depth-2 **branching** trees do not. Depth-1 is
provably immune (a depth-1 tap-tree can only ever have exactly 2 leaves, which
is exactly the one case the buggy formatter gets right), so "depth ≥ 2" is a
sound, conservative, easy-to-compute *superset* of the actual danger — it never
misses a real case, it just also warns on some safe chains. That is the correct
direction to be wrong in for an EXPERIMENTAL/funds-safety gate. **Recommendation:
KEEP the gate as-is.** The only thing worth tightening is the wording, which
currently doesn't distinguish "unreleased" from "merged-but-unreleased" — it
could truthfully say "merged to rust-miniscript master 2026-05-25, not yet in
a released version (latest: 13.1.0)" instead of just "unreleased", which is a
strictly stronger and equally true claim. This is a wording note, not a defect.

Also confirms, independently: **the #953 bug does not touch this project's own
address-derivation path.** `md address --template` derived correct bech32m P2TR
addresses for both a depth-2 and a depth-3 taptree template end-to-end (below).
`crates/md-codec/src/{derive,to_miniscript}.rs` never call `.to_string()` on a
`miniscript::Descriptor`/`TapTree` on any non-error path (grepped, both files —
every `to_string()` hit is on an `Err` variant). The #953 bug is purely a
**text-serialization** bug (rendering an AST back to descriptor syntax); it does
not affect the Merkle-root/output-key computation the address and wire-encode
paths use, which walk the (correct) internal `depths_leaves` structure directly.
This is exactly what `SPEC_seedhammer_template_engrave.md`'s O1 resolution
already asserted — this recon adds the missing pinned reproduction that
`agent-reports/seedhammer-template-engrave-spec-R0-round0.md`'s finding **M4**
said was absent ("cite no pinned check ... that demonstrates the taptree-display
failure on the shipped crates.io rust-miniscript" — M4 is now answered).

---

## PR #953

**What it changes** (github.com/rust-bitcoin/rust-miniscript/pull/953, fetched
2026-08-18): fixes `TapTree`'s `Display`/`Debug` formatting for **unbalanced**
Taproot trees. The bug was introduced by an earlier commit that flattened
`TapTree` from a recursive enum into a depth-preorder leaf list; the formatter
only tracked depth *changes between adjacent leaves*, which loses which leaves
are siblings whenever two leaves from different subtrees land at the same
absolute depth — it prints a flat run of leaves under one brace pair instead of
the correct nested pairs, and the result fails to re-parse. Fix: an iterative
formatter using an explicit child-count stack that closes a subtree exactly
when it has emitted both children. **Merged** 2026-05-25 into
`rust-bitcoin:master` at commit `ff4732e` ("Merge rust-bitcoin/rust-miniscript#953:
descriptor: fix Taproot tree descriptor formatting"), by apoelstra; three
commits (fix, regression tests, non-empty-`TapTree` assertion).

**Released version check** — machine-checked, not inferred, in
`/scratch/code/shibboleth/rust-miniscript-fork` (remotes: `origin` =
apoelstra/rust-miniscript, `upstream` = rust-bitcoin/rust-miniscript;
`git fetch upstream --tags` run fresh this session):

```
$ git merge-base --is-ancestor ff4732e upstream/master   ; echo $?
0   # YES — on master
$ git merge-base --is-ancestor ff4732e 13.0.0             ; echo $?
1   # NO
$ git merge-base --is-ancestor ff4732e miniscript-13.0.1  ; echo $?
1   # NO
$ git merge-base --is-ancestor ff4732e miniscript-13.1.0  ; echo $?
1   # NO
```

crates.io (`https://crates.io/api/v1/crates/miniscript`, fetched with a proper
UA to avoid the bot-policy 403 the bare-curl request hit): `max_version` =
**13.1.0**, `created_at` 2026-06-09T23:19:25Z — i.e. the newest thing anyone can
actually `cargo add` today does **not** contain the fix. (13.1.0 was cut from a
long-lived `release-13.x` branch that cherry-picked only PR #978/#974, not
master's #953 — `git log --oneline miniscript-13.1.0 -3` shows
`Merge ...#985: Release 13.1.0` / `Merge ...#978: Backport ...#974`, no #953.)

**Is #953 already in vendored 13.0.0?** NO — read, not inferred.
`/scratch/code/shibboleth/descriptor-mnemonic/vendor/miniscript/Cargo.toml:16`
pins `version = "13.0.0"`. Its
`vendor/miniscript/src/descriptor/tr/taptree.rs:29-31` already uses the flat
`depths_leaves: Vec<(u8, Arc<Miniscript<Pk, Tap>>)>` representation (the
flattening PR #953 sits *on top of*), and its `fmt_helper`
(`taptree.rs:87-114`) is **exactly** the buggy adjacent-depth-only algorithm PR
#953's description names:

```rust
fn fmt_helper<Pk: MiniscriptKey>(
    view: &TapTree<Pk>,
    f: &mut fmt::Formatter,
    mut fmt_ms: impl FnMut(&mut fmt::Formatter, &Miniscript<Pk, Tap>) -> fmt::Result,
) -> fmt::Result {
    let mut last_depth = 0;
    for item in view.leaves() {
        if last_depth > 0 { f.write_str(",")?; }
        while last_depth < item.depth() { f.write_str("{")?; last_depth += 1; }
        fmt_ms(f, item.miniscript())?;
        while last_depth > item.depth() { f.write_str("}")?; last_depth -= 1; }
    }
    while last_depth > 0 { f.write_str("}")?; last_depth -= 1; }
    Ok(())
}
```
(`taptree.rs:92-113`, `Display`/`Debug` impls at `:116-120`/`:122-125` both
call this same helper.) So: vendored 13.0.0 has the pre-#953 formatter,
confirmed by reading it, not by the version number alone.

---

## Empirical results

All commands run against `/scratch/code/shibboleth/descriptor-mnemonic`
(`md` v0.32-line, HEAD `89ab0f62`; `vendor/miniscript` pinned 13.0.0). Keys:
account-level xpubs derived on the fly from the well-known "abandon…about" test
mnemonic at `m/48'/0'/{i}'/2'` (depth 4, required by `md`'s `MultiSig`
script-context depth gate for a `tr()` with a script tree — a bare `tr(@0)`
key-path-only template is classified `SingleSig` and accepts depth-3, but any
`tr(...,{...})` is `MultiSig`-context and needs depth 4; this is the same F1
class already recorded in `agent-reports/miniscript-nesting/codec.md`, not
re-litigated here). Derivation done via a temporary
`crates/md-codec/examples/scratch_depth_recon_xpubs.rs` (deleted after use;
`git status --porcelain` on `descriptor-mnemonic` confirmed empty afterward).

### `md` CLI — depth-2 balanced taptree (`tr(@0,{{pk(@1),pk(@2)},{pk(@3),pk(@4)}})`)

```
$ md encode "tr(@0/48'/0'/0'/2'/<0;1>/*,{{pk(@1/48'/0'/1'/2'/<0;1>/*),pk(@2/48'/0'/2'/2'/<0;1>/*)},{pk(@3/48'/0'/3'/2'/<0;1>/*),pk(@4/48'/0'/4'/2'/<0;1>/*)}})"
md15y fdsss jjtvy yw2fd ssj55 jmpp9 ef9kz zwf2q qvppz 3fgjj zjn9g yuhp9 wkxsv 9dn
note: stdout is a keyless descriptor template (no keys)

$ md address --template "<same template>" --key @0=xpub6DkFAXWQ... --key @1=xpub6DzhyrnF... \
             --key @2=xpub6EGx8sPr... --key @3=xpub6E6Z3Ss5... --key @4=xpub6EhpCqtV... --count 3
bc1puafjqu7zk2d87qxrgjwspzxu8vaazrr67sx0f5z8nnnpyt2hzm3q3d3q4l
bc1p5xxyzkh0wz2w3auhlq4fmlcypgxwxtlgk8xvw3cdj8xnara4zhcq256nv3
bc1pt4ccz3sv6et008z8frumdu8aa50zrvy3mnjaysynzyztcygu98aq9yktvu
note: stdout is watch-only — public keys only, cannot spend
```
**Result: full success** — encode, and real bech32m P2TR address derivation
from real xpubs, for the balanced depth-2 shape.

### `md` CLI — depth-3 right-skewed taptree (`tr(@0,{pk(@1),{pk(@2),{pk(@3),pk(@4)}}})`)

```
$ md encode "tr(@0/48'/0'/0'/2'/<0;1>/*,{pk(@1/48'/0'/1'/2'/<0;1>/*),{pk(@2/48'/0'/2'/2'/<0;1>/*),{pk(@3/48'/0'/3'/2'/<0;1>/*),pk(@4/48'/0'/4'/2'/<0;1>/*)}}})"
md15y fdsss jjtvy yw2fd ssj55 jmpp9 ef9kz zwf2q qvppz j3zjj zjn9g ragtl 3v47r 7xs
note: stdout is a keyless descriptor template (no keys)

$ md decode md15yfdsssjjtvyyw2fdssj55jmpp9ef9kzzwf2qqvppzj3zjjzjn9gragtl3v47r7xs
tr(@0/<0;1>/*,{pk(@1/<0;1>/*),{pk(@2/<0;1>/*),{pk(@3/<0;1>/*),pk(@4/<0;1>/*)}}})
note: stdout is a keyless descriptor template (no keys)

$ md address --template "<same template>" --key @0=... --key @1=... --key @2=... --key @3=... --key @4=... --count 3
bc1pk5y6dnxpzsrpj34lgja4u43r96nmcfgpqdf5ghvxu2cn6e5djp7qn59pkq
bc1pctrtz4ng3qr0jw09xr7lqhze3cqypf6cvka6hwz2egaxnz9vy45su8cusc
bc1pglw6lm6hn2gvd5lnkaywzmr9wxz5g0v5g3fw4qcrq9ueyqyq0anqjdrgae
note: stdout is watch-only — public keys only, cannot spend
```
**Result: full success** — encode, exact wire round-trip through `md`'s OWN
decode/render path (byte/text-identical reconstruction, depth-3), and real
address derivation. **`md`'s own toolkit is unaffected by #953 at every depth
tested — this is the same O1 conclusion the spec already reached, now re-run
live rather than cited.**

### rust-miniscript's OWN `Descriptor::to_string()`/`from_str()` round-trip — the actual #953 probe

Built a throwaway probe crate (path-dependency on the vendored fork, fork
untouched) at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/dd8e287f-e1df-4a46-bd1b-18411072006c/scratchpad/miniscript-953-probe`,
parsing raw `tr(<pubkey-hex>,{...})` strings with
`miniscript::Descriptor::<String>::from_str`, then round-tripping through
`.to_string()` → re-parse:

```
--- depth-1 (2 leaf) ---                      stable round-trip? true
--- depth-2 balanced (4 leaf) ---              stable round-trip? false
  rendered: tr(...,{{pk(B),pk(C),pk(D),pk(E)}})     <- WRONG: flattened, only one brace pair
  reparse ERROR: taptree branch must have 2 children, but found 1
--- depth-3 right-skewed ---                   stable round-trip? true
--- left-heavy 3-leaf {{A,B},C} ---            stable round-trip? false
  reparse ERROR: taptree branch must have 2 children, but found 1
--- mixed depth 2/2/3/3 {{A,B},{C,{D,E}}} ---  stable round-trip? false
  reparse ERROR: taptree branch must have 2 children, but found 1
```

This is the **exact** depth-2 balanced shape used by `md`'s own
`tap_four_leaf_balanced_round_trips` test and by the depth-2 case above: it
encodes/derives fine through `md`'s own codec (which never calls
`TapTree::Display`) but **breaks** — genuinely fails to re-parse — if reduced
to descriptor text via rust-miniscript's own `Descriptor::to_string()`, i.e.
exactly the "off-device recovery/reconstruction with generic
rust-miniscript-based tooling" scenario the gate warns about. The depth-3
**chain** does not trigger it — see Verdict for why (it's a topology bug, only
provably ruled out at depth 1).

`descriptor-mnemonic` verified clean after: `git status --porcelain` → empty.

---

## BIP-341 / BIP-388 limits

**BIP-341** (`bip-0341.mediawiki`, "Script validation rules"): the control
block "must have length `33 + 32m`, for a value of `m` that is an integer
between 0 and 128" — i.e. **max Merkle-path/tree depth 128**, matching the
already-settled `TAPROOT_CONTROL_MAX_NODE_COUNT = 128` enforcement in the
`bitcoin` crate. BIP-341's own rationale (quoted): "Why is the Merkle path
length limited to 128? The optimally space-efficient Merkle tree can be
constructed based on the probabilities of the scripts in the leaves, using the
Huffman algorithm. This algorithm will construct branches with lengths
approximately equal to `log₂(1/probability)`, but to have branches longer than
128 you would need to have scripts with an execution chance below 1 in `2^128`.
As that is our security bound, scripts that truly have such a low chance can
probably be removed entirely." Control block size is a direct function of
depth: 33 bytes at `m=0` up to 4,129 bytes at `m=128`.

**BIP-388**: no independent depth/complexity constraint. Its `TREE` expression
grammar is simply recursive and unbounded: "TREE expressions: any SCRIPT
expression [or] An open brace `{`, a TREE expression, a comma `,`, a TREE
expression, and a closing brace `}`." BIP-388 defers entirely to BIP-341 (and,
for miniscript specifically, to "descriptor templates for miniscript are not
formally defined in this version of the document (pending standardization)")
— it imposes no shape restriction of its own.

---

## What the fork gate was protecting against

Verbatim, `gui/template_engrave.go:86-95` (fork HEAD, `f70456f`):
```go
// The depth-≥2 taproot EXPERIMENTAL gate (S5).
if tapDepth >= 2 {
    lines = append(lines,
        "EXPERIMENTAL: taproot depth >= 2",
        "The shipped toolkit CANNOT reconstruct",
        "this taptree (rust-miniscript PR #953).",
        "Recovery needs an UNRELEASED",
        "rust-miniscript >13.1.0.",
        "DO NOT use for real funds until that ships.",
    )
}
```

Verbatim, `design/SPEC_seedhammer_template_engrave.md:94-106` (S5 — the spec
this code implements):
> `### S5 — Taproot depth gate (DD6)`
> `- **depth-1 tr template:** normal path (subject to S1–S4).`
> `- **depth-≥2 tr template:** a SECOND, louder gate. **Wire-level encode/bind
> is CONFIRMED (O1 resolved — no taptree-depth refusal; `encodePayload`
> shape-general; tree serialization byte-faithful).** Engrave it behind:`
> ```
>  ⚠⚠  EXPERIMENTAL — taproot depth-≥2 template
>  The SHIPPED toolkit CANNOT reconstruct this
>  taptree (rust-miniscript taptree-display bug,
>  PR #953). Recovery currently requires an
>  UNRELEASED rust-miniscript >13.1.0.
>  DO NOT use for real funds until that ships.
> ```

And `SPEC_seedhammer_template_engrave.md:36`: "The #953 render defect blocks
only off-device recovery, not the on-device wire codec." — i.e., the authors
were explicit from the start that this gate is about **recoverability of a
human-readable descriptor string via generic off-device tooling**, not about
wire-level encode/decode/bind correctness (which O1 had already confirmed
sound) and not about address derivation (confirmed sound here too).

**Does the condition still hold?** Yes, per the machine checks above: #953 is
merged to master but absent from every released version through 13.1.0
(crates.io's current newest), and the specific render failure it fixes was
reproduced live against the exact vendored 13.0.0 tree this project ships, on
the same shape class `md`'s own tests use. The gate's premise has not gone
stale — it has been independently re-confirmed.

---

## Open / could not determine

- **crates.io curl access:** a bare `curl` (no User-Agent) to
  `crates.io/api/v1/crates/miniscript` was refused by crates.io's data-access
  policy; a request with a UA succeeded. Noted so a future check doesn't waste
  a round rediscovering this.
- **Whether upstream plans to backport #953 to a `13.x` or `14.x` release, or
  when.** Not stated anywhere I found (no open backport PR referenced in the
  changelog fetch, and I did not search rust-miniscript's issue tracker for a
  milestone — out of scope for this recon's one question).
- **The exact literal text of PR #953's own repro example** — WebFetch's
  summary of the PR description rendered a slightly garbled quote (mismatched
  brace/leaf counts); I did not fetch the PR's raw diff/comments to get the
  byte-exact original repro. Not needed: I built and ran an independent repro
  directly against the vendored crate instead of relying on the PR's own
  wording, which is the stronger form of evidence anyway.
- **Whether `agent-reports/miniscript-nesting/taproot-depth.md` and `codec.md`
  (both pre-existing, read as input to this recon) were themselves reviewed
  and folded into any spec** — I read them for settled facts (TapTree Merkle
  depth 128, `MAX_RECURSION_DEPTH` 402, `md-codec`'s own bound table) but did
  not check whether their findings (e.g. `codec.md`'s F5, encoder-has-no-depth-
  guard) were ever actioned. Out of scope for this question.
