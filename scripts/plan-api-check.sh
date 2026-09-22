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
ALLOW='InternalKey|liana_unspendable_xpub|LIANA_UNSPENDABLE_MARKER|wire_version|is_supported_version|WF_UNSPENDABLE_VERSION|to_miniscript_descriptor_with_network|to_miniscript_descriptor_multipath_with_network|descriptor_from_tlv_entries|kind1_chunks|validate_unspendable_shape|kind1_from_vector|all_cases|all_kind0_tr_vectors|tr_liana_at_use_site|tr_liana_with_sortedmulti_a_leaf|md_encode|md_err|in_crate_tr_liana_with_sortedmulti_a_leaf|wsh_wrapping_tr_liana|all_nums_tr|encode_payload_at_forced_version|compressed_33|NetworkRequiredForUnspendable|NonMinimalWireVersion|UnspendableNotRootTr|UnspendableWithSortedMultiA|UnspendableUseSiteNotCanonical|UnspendableNotRootTr|dump_encodings|dump_ids|md_argv|md_err|case|md_address'

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
            'read_to_string','to_owned','to_str','file_name','display','path','join',
            'into_iter','unwrap_or_else','into_bytes','as_str','as_ref','decode',
            'leaves','miniscript','depth','tap_tree','internal_key','new_empty',
            'standard_multipath'}
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
# THE ALLOW LIST IS A LOOPHOLE UNLESS IT IS VISIBLE. Measured 2026-09-22:
# key_arg / descriptor_with were added here as "symbols a task creates" while
# the plan's own comment called descriptor_with an EXISTING helper. The
# contradiction produced a green gate on a fixture that could not decode.
# So print what was exempted, and say which of those the repo actually has.
exempted = sorted({c for c in cands
                   if (c.split('::')[-1] in KEYWORDS) is False
                   and (allow_re.match(c.split('::')[-1]) or allow_re.match(c))})
if exempted:
    print(f"\nEXEMPTED BY ALLOW ({len(exempted)}) -- each MUST be a symbol this plan CREATES.")
    print("If the plan describes any of these as already existing, that is a defect the")
    print("gate cannot see; check it by hand:")
    for e in exempted:
        name = e.split('::')[-1]
        r = subprocess.run(['grep','-rqE',f'fn {name}',f'{repo}/crates'],capture_output=True)
        print(f"   {e}{'   [ALSO EXISTS in repo -- is it really created here?]' if r.returncode==0 else ''}")
# --- BARE CONSTANT REFERENCES -------------------------------------------
# Measured 2026-09-22: ORIGINLESS_SPENDABLE_TR was referenced and never
# defined, and this gate did not see it because it only extracted CALLS.
consts_used, consts_def = set(), set()
for b in blocks:
    body = re.sub(r'//.*', '', b)
    consts_used |= set(re.findall(r'\b([A-Z][A-Z0-9_]{3,})\b', body))
    consts_def  |= set(re.findall(r'(?:const|static)\s+([A-Z][A-Z0-9_]{3,})', body))
undef_consts = sorted(c for c in consts_used - consts_def
                      if not re.match(r'^(SPEC|BIP|NUMS|JSON|TODO|RIGHT|WRONG|RED|OK|ONE|TWO|NOT|AND|ALSO)', c))
if undef_consts:
    print(f"\nUNDEFINED CONSTANTS ({len(undef_consts)}) -- referenced in a rust block, defined in none:")
    for c in undef_consts:
        r = subprocess.run(['grep','-rqE',f'(const|static) {c}',f'{repo}/crates'],capture_output=True)
        print(f"   {c}{'   [exists in repo]' if r.returncode==0 else '   [NOWHERE]'}")

# --- CROSS-CRATE REACHABILITY -------------------------------------------
# Measured three times (r4/I-2, r5/I-1, r7/I-1): a helper EXISTS but is not
# reachable from the crate the test lives in, because include! and
# CARGO_MANIFEST_DIR are per-crate. The gate cannot decide this, so it REPORTS.
cli_blocks = [b for b in blocks if re.search(r'\bmd(_err)?\(&\[', b)]
cli_helpers = set()
for b in cli_blocks:
    cli_helpers |= set(re.findall(r'\b([a-z_][a-z_0-9]{3,})\s*\(', re.sub(r'//.*','',b)))
cli_helpers -= KEYWORDS | {'md','md_err'}
if cli_helpers:
    print(f"\nCALLED FROM md-cli-SHAPED BLOCKS ({len(cli_helpers)}) -- each must be")
    print("reachable from crates/md-cli/tests/, NOT only from md-codec's test root:")
    for h in sorted(cli_helpers): print(f"   {h}")

print(f"\nchecked {len(cands)} candidate symbols from {len(blocks)} rust blocks")
print("NOT covered: signatures, arities, generics, borrows, macro-built names.")
PY
