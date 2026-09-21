#!/usr/bin/env bash
# plan-api-check.sh -- resolve every Rust API an implementation plan CLAIMS
# already exists, against the real source.
#
# WHY THIS EXISTS
#   Three consecutive review rounds on IMPLEMENTATION_PLAN_f449_stage1 each
#   found an API the plan asserted rather than verified:
#     r0  -> `VECTORS` (it is MANIFEST) and `encode::encode_template` (absent)
#     r1  -> `Mode::Keyed`, `render_descriptor`, `ctx.network` (all absent)
#     r2  -> `encode_md1_chunks` (absent; `chunk::split` is the real producer)
#   Same defect three times is a wrong shape, not three mistakes. A reviewer
#   spending a design-review round to report "that function does not exist" is
#   a reviewer acting as a compiler, and the finding lands a full round late.
#
# WHAT IT DOES
#   Extracts candidate identifiers from the plan's ```rust blocks -- qualified
#   paths (md_codec::a::b), method/free-function calls, and type names -- then
#   asks the repo whether each exists. Prints one line per unresolved symbol.
#
# WHAT IT DOES NOT DO -- a gate that hides its blind spot is worse than none
#   - It cannot tell "this API does not exist" from "this task CREATES it".
#     Symbols a task introduces are listed in the plan's Interfaces blocks;
#     pass them via ALLOW (below) or accept them in the output as expected.
#   - It does not type-check. Signatures, arities, generics and borrows are
#     unchecked -- those still need a reviewer's execution pass.
#   - It does not see identifiers built by string concatenation or macros.
set -uo pipefail
PLAN="${1:?usage: plan-api-check.sh <plan.md> [repo-root]}"
REPO="${2:-/scratch/code/shibboleth/descriptor-mnemonic}"

# Symbols the plan itself declares it will CREATE (its Interfaces blocks).
ALLOW='InternalKey|liana_unspendable_xpub|LIANA_UNSPENDABLE_MARKER|wire_version|is_supported_version|WF_UNSPENDABLE_VERSION|to_miniscript_descriptor_with_network|to_miniscript_descriptor_multipath_with_network|descriptor_from_tlv_entries|kind1_chunks|NetworkRequiredForUnspendable|UnspendableWithSortedMultiA|UnspendableUseSiteNotCanonical|UnspendableNotRootTr|dump_encodings|dump_ids|md_argv|md_err|case|md_address'

python3 - "$PLAN" "$REPO" "$ALLOW" <<'PY'
import re, subprocess, sys, pathlib
plan, repo, allow = sys.argv[1], sys.argv[2], sys.argv[3]
text = pathlib.Path(plan).read_text()
blocks = re.findall(r'```rust\n(.*?)```', text, re.S)
cands = set()
for b in blocks:
    b = re.sub(r'//.*', '', b)                       # strip line comments
    cands |= set(re.findall(r'md_codec::([a-z_]+(?:::[a-z_]+)*)', b))
    cands |= set(re.findall(r'\b([a-z_][a-z_0-9]{3,})\s*\(', b))   # calls
    cands |= set(re.findall(r'\b(Error::[A-Za-z]+)', b))
allow_re = re.compile(rf'^({allow})$')
# std / prelude / common-crate methods: not this repo's APIs to resolve.
KEYWORDS = {'assert','assert_eq','assert_ne','println','format','panic','vec','matches',
            'expect','unwrap','unwrap_or_else','unwrap_err','into','from','to_string','len',
            'iter','map','collect','push','push_str','extend','extend_from_slice','contains',
            'starts_with','write','read','for_each_key','serde_json','from_str','if','while',
            'match','return','let','trim','windows','with_capacity','try_from','from_slice',
            'as_slice','as_deref','is_ok','is_err','is_some','is_none','is_some_and',
            'to_byte_array','from_normal_idx','hash','serialize','clone','to_vec','zip',
            'find','filter','lines','join','split','next','count','sort','dedup','get',
            'read_to_string','to_owned','to_str','file_name','display','path','join'}
# Names the PLAN ITSELF defines in a rust block (tests and local helpers) are
# not claims about the repo -- they are the plan's own code.
defined = set(re.findall(r'\bfn ([a-z_][a-z_0-9]*)', '\n'.join(blocks)))
missing = []
for c in sorted(cands):
    name = c.split('::')[-1]
    if name in KEYWORDS or name in defined or allow_re.match(name) or allow_re.match(c): continue
    pat = (name if c.startswith('Error::') else f'fn {name}')
    r = subprocess.run(['grep','-rqE',pat,f'{repo}/crates'],capture_output=True)
    if r.returncode != 0: missing.append(c)
if missing:
    print(f"UNRESOLVED ({len(missing)}) -- each is either a plan defect or a symbol a task creates:")
    for m in missing: print(f"   {m}")
else:
    print("all extracted symbols resolve")
print(f"\nchecked {len(cands)} candidate symbols from {len(blocks)} rust blocks")
print("NOT covered: signatures, arities, generics, borrows, macro-built names.")
PY
