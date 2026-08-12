# rust-miniscript TAPROOT nesting depth — verified against source + empirical probe

- Repo: `/scratch/code/shibboleth/rust-miniscript-fork`, crate `miniscript` v13.0.0
  (`Cargo.toml`), HEAD `2092faa` ("descriptor: support hash terminals in WalletPolicy
  translator").
- Resolved dependency: `bitcoin` v0.32.102 (via `cargo build`; registry cache also has
  0.32.8 — both define the same constant value, checked).
- Method: read the enforcing code paths, then built a scratch crate at
  `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/22fd28a4-d68a-47d6-82b1-8a8570fb5417/scratchpad/miniscript-depth-test`
  with a **path dependency** on the fork (fork not modified), generating `tr()`
  descriptor strings of increasing depth and recording the exact failure point
  and error string.

## "Nesting" is two unrelated axes — answer both

### 1. TapTree / Merkle depth (the `{a,b}` branching structure of `tr()`)

- Enforcing constant: `TAPROOT_CONTROL_MAX_NODE_COUNT: usize = 128`, defined in the
  **`bitcoin` crate** (not miniscript itself), `bitcoin-0.32.102/src/taproot/mod.rs:142`
  (identical value at 0.32.8:141 in the older cached copy).
- Enforced in miniscript at two call sites, both re-derived from that constant:
  - `src/descriptor/tr/taptree.rs:253` (`TapTreeBuilder::push_inner_node`, the path a
    parsed `tr()` descriptor string actually takes):
    ```rust
    pub(super) fn push_inner_node(&mut self) -> Result<(), TapTreeDepthError> {
        self.current_height += 1;
        if usize::from(self.current_height) > TAPROOT_CONTROL_MAX_NODE_COUNT {
            return Err(TapTreeDepthError);
        }
        Ok(())
    }
    ```
  - `src/descriptor/tr/taptree.rs:44` (`TapTree::combine`, the programmatic
    tree-building API): `if usize::from(*depth) > TAPROOT_CONTROL_MAX_NODE_COUNT - 1 { return Err(TapTreeDepthError); }`
  - Both agree on the same admissible max: **depth 128 succeeds, depth 129 fails.**
  - Error type: `TapTreeDepthError` (`src/descriptor/tr/taptree.rs:14-21`), `Display`:
    `"maximum Taproot tree depth (128) exceeded"` (`taptree.rs:20`).
- This is reached from an ordinary descriptor string via
  `Tr::from_str` → `expression::Tree::from_str` → `Tr::from_tree` (`src/descriptor/tr/mod.rs:341-395`),
  which drives `TapTreeBuilder::push_inner_node()` once per `{...}` branch
  (`tr/mod.rs:382`) while walking the parsed tree — this is the code path a user
  actually hits typing a `tr()` descriptor.
- **Empirically confirmed**: a right-skewed chain
  `tr(K,{pk(K0),{pk(K1),{pk(K2),...{pk(K127),pk(K128)}...}}})` with **128 nested
  braces** (deepest leaf at BIP-341 depth 128) parses successfully; **129 nested
  braces** fails with exactly:
  ```
  maximum Taproot tree depth (128) exceeded
  ```
  (Matches BIP-341's own hard cap: control block encodes depth as one byte with a
  documented max of 128.)

### 2. Miniscript fragment recursion depth (inside one leaf, or generically)

- Constant: `const MAX_RECURSION_DEPTH: u32 = 402;` — `src/lib.rs:503` (comment cites
  `https://github.com/sipa/miniscript/pull/5` for the number's origin).
- Enforced in **two** places, at the **same value**, on different inputs:
  1. **String parsing, unconditionally, for the whole descriptor** —
     `src/expression/mod.rs:592-597`, inside `Tree::parse_pre_check`, which scans the
     raw string once and tracks a combined `(`/`{` nesting-depth counter
     (`open_paren_stack`) over **the entire input**, tap-tree braces and miniscript
     parens alike:
     ```rust
     if u32::try_from(max_depth).unwrap_or(u32::MAX) > MAX_RECURSION_DEPTH {
         return Err(ParseTreeError::MaxRecursionDepthExceeded {
             actual: max_depth,
             maximum: MAX_RECURSION_DEPTH,
         });
     }
     ```
     Error `Display` (`src/expression/error.rs:108-110`):
     `"maximum recursion depth exceeded (max {maximum}, got {actual})"`.
     This runs for **every** context (Legacy/Segwitv0/Tap) — it is not Tap-specific,
     it is a property of the input string, checked before any tree is built.
  2. **AST construction, per-node, after parsing** — `src/miniscript/mod.rs:333`
     (`Miniscript::from_ast`), which independently checks `tree_height` on every
     constructed AST node:
     ```rust
     if (res.ext.tree_height as u32) > MAX_RECURSION_DEPTH {
         return Err(Error::MaxRecursiveDepthExceeded);
     }
     ```
     `Display` for that error (`src/lib.rs:526-530`): `"Recursive depth over 402 not
     permitted"`. The code comment above it (`lib.rs:329-331`) notes the bound is
     "based on segwitv0... we can relax this in tapscript" but **it is not currently
     relaxed** — it applies uniformly, including to `Tap`. In practice check (1)
     rejects over-deep strings first, so check (2) is defense-in-depth for
     programmatic/non-string AST construction (e.g. building via combinators or
     decoding from raw Script), not something a string-parsing user reaches.
- **Applies to Tap**: yes, confirmed — `Tr::from_str` routes every leaf's miniscript
  through the same `expression::Tree::from_str` and `Miniscript::from_ast`, so a
  single tap-leaf fragment is bound by the same 402.
- **Empirically confirmed**: `tr(K, and_v(v:pk(K400),and_v(v:pk(K399),...and_v(v:pk(K1),pk(K0))...)))`
  with 400 nested `and_v` wraps parses; 401 wraps fails with exactly:
  ```
  maximum recursion depth exceeded (max 402, got 403)
  ```
  (the reported "got" count includes the outer `tr(` and the innermost `pk()`
  parens, so `n` wraps → `max_depth = n + 2`; 401 → 403 > 402).

### 3. Other depth-ish bounds checked and ruled out as "bites first"

- **Per-leaf script size**: `Tap`'s `check_global_consensus_validity`
  (`src/miniscript/context.rs:600-636`) explicitly does **not** apply the legacy
  `MAX_SCRIPT_SIZE` (10,000 bytes, `src/miniscript/limits.rs:15`, only enforced for
  `BareCtx` at `context.rs:733`) — the comment says "No script size checks for
  global consensus rules" and the only ceiling is `Weight::MAX_BLOCK` (~4M WU),
  unreachable in practice. So leaf script size is not a realistic first limit for
  Tap.
- **Number of leaves / control block size**: both are direct functions of Merkle
  depth (`TAPROOT_CONTROL_BASE_SIZE + depth * TAPROOT_CONTROL_NODE_SIZE`,
  `src/descriptor/tr/mod.rs:443`, using `bitcoin`'s `TAPROOT_CONTROL_BASE_SIZE = 33`
  and `TAPROOT_CONTROL_NODE_SIZE = 32`) — already captured by bound #1, not an
  independent ceiling.

## The practical answer

Building a `tr()` descriptor the normal way — nesting `{leaf,leaf}` branches to add
more script paths — **the first wall a user hits is the TapTree Merkle-depth limit
of 128** (`TAPROOT_CONTROL_MAX_NODE_COUNT`, from the `bitcoin` crate, enforced in
miniscript at `src/descriptor/tr/taptree.rs:253`), giving the exact error
`"maximum Taproot tree depth (128) exceeded"`. Empirically verified boundary:
**128 nested `{}` levels parse, 129 fail.**

The crate's own `MAX_RECURSION_DEPTH = 402` (`src/lib.rs:503`) is a *different* axis
— total paren/brace nesting of the whole descriptor string, dominated in practice by
how deeply a *single leaf's* miniscript fragment is nested (e.g. chained `and_v`).
It is real and enforced (empirically: 400 nested `and_v` wraps parse, 401 fail with
`"maximum recursion depth exceeded (max 402, got 403)"`), but for the ordinary
tree-branching case it is never reached — 128 tree levels only contributes
`~130` to that counter, far short of `402`. It would only become the binding
constraint if someone built a pathologically deep *single leaf* rather than a deep
*tree*.

No other depth-shaped bound (per-leaf script size, leaf count, control-block size)
binds before these two in Tap context.

## Empirical harness

Scratch crate (path-dep on the fork, fork untouched):
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/22fd28a4-d68a-47d6-82b1-8a8570fb5417/scratchpad/miniscript-depth-test`
(`Cargo.toml` + `src/main.rs`). Uses `miniscript::descriptor::Tr::<String>::from_str`
(`String: MiniscriptKey` per `src/lib.rs:208`, satisfies `FromStrKey` per
`src/blanket_traits.rs:80`) to avoid needing real secp256k1 keys. Full run output:

```
--- TapTree depth (skewed, braces only) ---
  last success: n=128
  first failure: n=129
  error string: "maximum Taproot tree depth (128) exceeded"
--- Single-leaf miniscript recursion depth (and_v chain) ---
  last success: n=400
  first failure: n=401
  error string: "maximum recursion depth exceeded (max 402, got 403)"
--- smoke tests ---
  n_braces=0 (single leaf, no tree): true
  n_braces=1: true
  n_braces=2: true
```

Build: `cargo build --release` succeeded clean against fork commit `2092faa` with
resolved `bitcoin = 0.32.102`.
