# Composer lowering — I1 ruling: the single-key head in `wsh` (miniscript expert review)

Date: 2026-09-01. Reviewer: independent miniscript agent (read-only; this file is the only artifact written).
Tools: rust-miniscript fork `/scratch/code/shibboleth/rust-miniscript-fork` @ `2092faa` (crate 13.0.0), probe crate in the session scratchpad (`probe/cases3.txt` → `probe/out3.txt`), `md 0.14.0`, Core `src/script/miniscript.h` (master, fetched today), Ledger `app-bitcoin-new/src/common/wallet.c` (develop, fetched today), Liana `analysis.rs`/`descriptors/mod.rs` and libnunchuk `src/miniscript/miniscript.cpp` (scratchpad copies), BIP-379 `bip-0379.md` and BIP-388 `bip-0388.mediawiki` (master copies in scratchpad).

## 1. Ruling

**(b) `or_i(pkh(P1), rest)` for a bare single-key head; `or_d(multi(k,…), rest)` for a bare multi-key head; the first review's I1 remedy is CONFIRMED, with two refinements and one explicit price tag.** Measured on the 4-path wallet: (a) `or_d(pkh)` costs **+34 WU on every non-P1 spend** (474/435/506 vs 440/401/472 WU) and pushes K0 into the witness on each of them — `pubkeys_in_witness_items=[K0]` on the P2, P3 and P4 spends — so it forfeits the only thing `pkh` was chosen for, while saving 2 WU on P1 spends. (b) keeps K0 out of both script and witness on every non-P1 spend (`pubkeys_in_witness_items=[]`, `pubkeys_in_script=[K1,K2,K3,K5,K6]`) and is the cheapest form on every non-P1 spend. (c) `or_d(pk)` — the BIP-388 l.249 / compiler form — is the cheapest on P1 spends by **26 WU (6.5 vB) per input** and costs **+10 WU** on every other spend, with K0 in cleartext in the script on every spend; the break-even is P1 ≥ 27.8 % of spends, which a daily key always exceeds. So (c) is fee-optimal for the operator's mix and (b) is privacy-optimal; **given the settled C17 ruling (`pkh` for one key in wsh) only (b) honours it — (a) is dominated, not a compromise.** The price of C17 is therefore exactly 26 WU per daily spend, and if that ruling is ever revisited (c) is the drop-in. The multi head keeps `or_d`: its dissatisfaction is k+1 empty pushes (no key leaks), saving 2 WU on the head spend and costing k WU (1..9) on each other spend; uniform `or_i` would be acceptable at a 2 WU cost per head spend but buys nothing structural. All candidates are `sanity=OK`, non-malleable, in limits, lift-equal, and `md encode` accepts every form. Refinements: reject the flipped `or_i(rest, pkh(P1))` (1 WU, breaks C1 first-appearance numbering) and `t:or_c`/`andor` (same leak as (a), +1 B); state in the rule that the head fragment is chosen at **every** nesting level.

## 2. Evidence tables (probe output verbatim)

Probe: `Miniscript::<PublicKey, Segwitv0>::from_str_insane` → `satisfy()` with a `Satisfier` holding only the listed keys/flags (`o`=older ok, `a`=after ok, `h`=preimage). Signatures are real RFC6979 ECDSA (72 B; K5's happens to be 71 B on this message — identical across candidates, so deltas are exact). `WITNESS_WU` = full witness serialisation = `varint(n+1) + Σ(varint(len)+len) + varint(script) + script` (segwit v0: 1 byte = 1 WU). The non-witness input part (41 B × 4 = 164 WU) is identical across candidates and omitted. `pubkeys_in_witness_items` / `pubkeys_in_script` are byte-matches of the 33-byte keys against the witness items and the witnessScript. In the `L<n> sat …` echo lines below, the shared tail `or_i(and_v(v:multi(2,K1,K2,K3),older(100)),or_i(and_v(v:pkh(K4),after(1000000)),and_v(v:multi(2,K5,K6),and_v(v:sha256(HH),older(200)))))` is abbreviated `REST` (and the multi-head tail `MREST`); the `L<n>:` result lines are untouched.

### 2.1 Two-path baseline (first review L44–L51 rebuilt with WU + reveal tracer)

```
L2 sat wsh or_d(pkh(K0),and_v(v:pkh(K1),older(100))) keys=K0 flags=
L2: items=2 lens=[72,33<031b84c5..>] witness_item_bytes=107 script_size=56 total_wit_bytes(items+script,no CB)=164 WITNESS_WU=165 pubkeys_in_witness_items=[K0] pubkeys_in_script=[]
L3 sat wsh or_d(pkh(K0),and_v(v:pkh(K1),older(100))) keys=K1 flags=o
L3: items=4 lens=[72,33<024d4b6c..>,0,33<031b84c5..>] witness_item_bytes=142 script_size=56 total_wit_bytes(items+script,no CB)=199 WITNESS_WU=200 pubkeys_in_witness_items=[K0,K1] pubkeys_in_script=[]
L4 sat wsh or_i(pkh(K0),and_v(v:pkh(K1),older(100))) keys=K0 flags=
L4: items=3 lens=[72,33<031b84c5..>,1] witness_item_bytes=109 script_size=56 total_wit_bytes(items+script,no CB)=166 WITNESS_WU=167 pubkeys_in_witness_items=[K0] pubkeys_in_script=[]
L5 sat wsh or_i(pkh(K0),and_v(v:pkh(K1),older(100))) keys=K1 flags=o
L5: items=3 lens=[72,33<024d4b6c..>,0] witness_item_bytes=108 script_size=56 total_wit_bytes(items+script,no CB)=165 WITNESS_WU=166 pubkeys_in_witness_items=[K1] pubkeys_in_script=[]
L6 sat wsh or_d(pk(K0),and_v(v:pkh(K1),older(100))) keys=K0 flags=
L6: items=1 lens=[72] witness_item_bytes=73 script_size=66 total_wit_bytes(items+script,no CB)=140 WITNESS_WU=141 pubkeys_in_witness_items=[] pubkeys_in_script=[K0]
L7 sat wsh or_d(pk(K0),and_v(v:pkh(K1),older(100))) keys=K1 flags=o
L7: items=3 lens=[72,33<024d4b6c..>,0] witness_item_bytes=108 script_size=66 total_wit_bytes(items+script,no CB)=175 WITNESS_WU=176 pubkeys_in_witness_items=[K1] pubkeys_in_script=[K0]
L8 sat wsh or_i(pk(K0),and_v(v:pkh(K1),older(100))) keys=K0 flags=
L8: items=2 lens=[72,1] witness_item_bytes=75 script_size=66 total_wit_bytes(items+script,no CB)=142 WITNESS_WU=143 pubkeys_in_witness_items=[] pubkeys_in_script=[K0]
L9 sat wsh or_i(pk(K0),and_v(v:pkh(K1),older(100))) keys=K1 flags=o
L9: items=3 lens=[72,33<024d4b6c..>,0] witness_item_bytes=108 script_size=66 total_wit_bytes(items+script,no CB)=175 WITNESS_WU=176 pubkeys_in_witness_items=[K1] pubkeys_in_script=[K0]
```

### 2.2 Four-path wallet — P1 single K0; P2 2-of-3(K1,K2,K3)+older(100); P3 single K4+after(1000000); P4 2-of-2(K5,K6)+sha256+older(200)

```
## S2a (a) or_d(pkh(K0),rest)
L12: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=286 max_sat_size=Ok(217) max_wit_elems=Ok(9)
L13 sat wsh or_d(pkh(K0),REST) keys=K0 flags=
L13: items=2 lens=[72,33<031b84c5..>] witness_item_bytes=107 script_size=286 total_wit_bytes(items+script,no CB)=396 WITNESS_WU=397 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L14 sat wsh or_d(pkh(K0),REST) keys=K1,K2 flags=o
L14: items=6 lens=[0,72,72,1,0,33<031b84c5..>] witness_item_bytes=184 script_size=286 total_wit_bytes(items+script,no CB)=473 WITNESS_WU=474 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L15 sat wsh or_d(pkh(K0),REST) keys=K4 flags=a
L15: items=6 lens=[72,33<0362c0a0..>,1,0,0,33<031b84c5..>] witness_item_bytes=145 script_size=286 total_wit_bytes(items+script,no CB)=434 WITNESS_WU=435 pubkeys_in_witness_items=[K0,K4] pubkeys_in_script=[K1,K2,K3,K5,K6]
L16 sat wsh or_d(pkh(K0),REST) keys=K5,K6 flags=ho
L16: items=8 lens=[32<07070707..>,0,71,72,0,0,0,33<031b84c5..>] witness_item_bytes=216 script_size=286 total_wit_bytes(items+script,no CB)=505 WITNESS_WU=506 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
## S2b (b) or_i(pkh(K0),rest)
L18: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=286 max_sat_size=Ok(183) max_wit_elems=Ok(8)
L19 sat wsh or_i(pkh(K0),REST) keys=K0 flags=
L19: items=3 lens=[72,33<031b84c5..>,1] witness_item_bytes=109 script_size=286 total_wit_bytes(items+script,no CB)=398 WITNESS_WU=399 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L20 sat wsh or_i(pkh(K0),REST) keys=K1,K2 flags=o
L20: items=5 lens=[0,72,72,1,0] witness_item_bytes=150 script_size=286 total_wit_bytes(items+script,no CB)=439 WITNESS_WU=440 pubkeys_in_witness_items=[] pubkeys_in_script=[K1,K2,K3,K5,K6]
L21 sat wsh or_i(pkh(K0),REST) keys=K4 flags=a
L21: items=5 lens=[72,33<0362c0a0..>,1,0,0] witness_item_bytes=111 script_size=286 total_wit_bytes(items+script,no CB)=400 WITNESS_WU=401 pubkeys_in_witness_items=[K4] pubkeys_in_script=[K1,K2,K3,K5,K6]
L22 sat wsh or_i(pkh(K0),REST) keys=K5,K6 flags=ho
L22: items=7 lens=[32<07070707..>,0,71,72,0,0,0] witness_item_bytes=182 script_size=286 total_wit_bytes(items+script,no CB)=471 WITNESS_WU=472 pubkeys_in_witness_items=[] pubkeys_in_script=[K1,K2,K3,K5,K6]
## S2c (c) or_d(pk(K0),rest)  -- compiler / BIP-388 l.249 form
L24: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=296 max_sat_size=Ok(183) max_wit_elems=Ok(8)
L25 sat wsh or_d(pk(K0),REST) keys=K0 flags=
L25: items=1 lens=[72] witness_item_bytes=73 script_size=296 total_wit_bytes(items+script,no CB)=372 WITNESS_WU=373 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
L26 sat wsh or_d(pk(K0),REST) keys=K1,K2 flags=o
L26: items=5 lens=[0,72,72,1,0] witness_item_bytes=150 script_size=296 total_wit_bytes(items+script,no CB)=449 WITNESS_WU=450 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
L27 sat wsh or_d(pk(K0),REST) keys=K4 flags=a
L27: items=5 lens=[72,33<0362c0a0..>,1,0,0] witness_item_bytes=111 script_size=296 total_wit_bytes(items+script,no CB)=410 WITNESS_WU=411 pubkeys_in_witness_items=[K4] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
L28 sat wsh or_d(pk(K0),REST) keys=K5,K6 flags=ho
L28: items=7 lens=[32<07070707..>,0,71,72,0,0,0] witness_item_bytes=182 script_size=296 total_wit_bytes(items+script,no CB)=481 WITNESS_WU=482 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
## S2d1 (d1) or_i(pk(K0),rest)
L30: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=296 max_sat_size=Ok(183) max_wit_elems=Ok(8)
L31: items=2 lens=[72,1] witness_item_bytes=75 script_size=296 total_wit_bytes(items+script,no CB)=374 WITNESS_WU=375 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
L32: items=5 lens=[0,72,72,1,0] witness_item_bytes=150 script_size=296 total_wit_bytes(items+script,no CB)=449 WITNESS_WU=450 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K3,K5,K6]
## S2d2 (d2) or_i(rest,pkh(K0)) -- P1 in the ELSE arm (push 0 instead of 1)
L34: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=286 max_sat_size=Ok(184) max_wit_elems=Ok(8)
L35: items=3 lens=[72,33<031b84c5..>,0] witness_item_bytes=108 script_size=286 total_wit_bytes(items+script,no CB)=397 WITNESS_WU=398 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L36: items=5 lens=[0,72,72,1,1] witness_item_bytes=151 script_size=286 total_wit_bytes(items+script,no CB)=440 WITNESS_WU=441 pubkeys_in_witness_items=[] pubkeys_in_script=[K1,K2,K3,K5,K6]
L37: items=5 lens=[72,33<0362c0a0..>,1,0,1] witness_item_bytes=112 script_size=286 total_wit_bytes(items+script,no CB)=401 WITNESS_WU=402 pubkeys_in_witness_items=[K4] pubkeys_in_script=[K1,K2,K3,K5,K6]
## S2d3 (d3) t:or_c(pkh(K0),v:rest)
L39: type=Busfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=287 max_sat_size=Ok(217) max_wit_elems=Ok(9)
L40: items=2 lens=[72,33<031b84c5..>] witness_item_bytes=107 script_size=287 total_wit_bytes(items+script,no CB)=397 WITNESS_WU=398 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L41: items=6 lens=[0,72,72,1,0,33<031b84c5..>] witness_item_bytes=184 script_size=287 total_wit_bytes(items+script,no CB)=474 WITNESS_WU=475 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
## S2d4 (d4) andor(pkh(K0),1,rest) -- not emittable on device; measured for completeness
L43: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=287 max_sat_size=Ok(217) max_wit_elems=Ok(9)
L44: items=2 lens=[72,33<031b84c5..>] witness_item_bytes=107 script_size=287 total_wit_bytes(items+script,no CB)=397 WITNESS_WU=398 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
L45: items=6 lens=[0,72,72,1,0,33<031b84c5..>] witness_item_bytes=184 script_size=287 total_wit_bytes(items+script,no CB)=474 WITNESS_WU=475 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6]
## S2e sub-arm types  (pkh(K0) / pk(K0) / multi(2,K0,K1,K2) / REST)
L47: type=Bndusem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=25 max_sat_size=Ok(107) max_wit_elems=Ok(3)
L48: type=Bondusem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=35 max_sat_size=Ok(73) max_wit_elems=Ok(2)
L49: type=Bndusem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=105 max_sat_size=Ok(147) max_wit_elems=Ok(4)
L50: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=258 max_sat_size=Ok(182) max_wit_elems=Ok(7)
## S2f descriptor-level (rust-miniscript's own max_weight_to_satisfy) for (a)(b)(c)
L52: type=Wsh sanity=OK max_weight_to_satisfy=506 spk_len=34 addr=bc1q8a5nmptmvt5mn2ydfyug36ec6m485vechf3cgu5jyx6xuhgmr2vq80g7q4
L53: type=Wsh sanity=OK max_weight_to_satisfy=472 spk_len=34 addr=bc1qvu7ny5mdjakt4g6gprhqecxpd8amwx5dae9h5kvzpv02cnefgs4smc6ghz
L54: type=Wsh sanity=OK max_weight_to_satisfy=482 spk_len=34 addr=bc1qtm0xx5fzhq077zgx37vhx52403a82w9arqvp763yxxl8mvf7vcussvcsq2
## S2g lift equality across candidates
L56: lift_equal=true   (or_d(pkh) vs or_i(pkh))
L57: lift_equal=true   (or_i(pkh) vs or_d(pk))
L58: lift_equal=true   (or_i(pkh) vs or_i(rest,pkh))
L59: lift_equal=true   (or_i(pkh) vs t:or_c(pkh,v:rest))
## S2h compiler cross-check (uniform and 99:1 P1-weighted probabilities) -- head is or_d(pk(K0)) in both
L61: compiled=or_d(pk(031b84c5…),andor(multi(2,024d4b6c…,02531fe6…,03462779…),older(100),or_i(and_v(v:pkh(0362c0a0…
L62: compiled=or_d(pk(031b84c5…),or_i(and_v(v:thresh(2,pkh(024d4b6c…),a:pkh(02531fe6…),a:pkh(03462779…)),older(100)),or_i(and_v(v:pkh(0362c0a0…
```

Summary (WITNESS_WU; deltas in WU, 4 WU = 1 vB):

| spend | (a) `or_d(pkh)` | (b) `or_i(pkh)` | (c) `or_d(pk)` | (b)−(a) | (c)−(b) | K0 revealed under (a)/(b)/(c) |
| --- | --- | --- | --- | --- | --- | --- |
| P1 (K0) | 397 | 399 | **373** | +2 | **−26** | witness / witness / script |
| P2 (K1,K2 + older) | 474 | **440** | 450 | **−34** | +10 | **witness** / no / script |
| P3 (K4 + after) | 435 | **401** | 411 | **−34** | +10 | **witness** / no / script |
| P4 (K5,K6 + sha256 + older) | 506 | **472** | 482 | **−34** | +10 | **witness** / no / script |
| script bytes | 286 | 286 | 296 | 0 | +10 | |
| `max_weight_to_satisfy` | 506 | 472 | 482 | −34 | +10 | |

The or_d dissatisfaction of `pkh(K0)` is the trailing `0,33<031b84c5..>` pair (empty sig + K0, 1 + 34 = 35 B) versus `or_i`'s single selector byte `0` (1 B): +34 WU. `pk` vs `pkh` in the script is `<33> CHECKSIG` (35 B) vs `DUP HASH160 <20> EQUALVERIFY CHECKSIG` (25 B): +10 B. On a P1 spend `pkh` pushes K0 (34 B) plus the `or_i` selector `1` (2 B) that `or_d` avoids: 36 B, minus the 10 B smaller script = 26 WU.

### 2.3 Multi head — P1 2-of-3(K0,K1,K2); P2 K3+older; P3 K4+after; P4 2-of-2+sha256+older

```
L64: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=286 max_sat_size=Ok(185) max_wit_elems=Ok(10)
L65 sat wsh or_d(multi(2,K0,K1,K2),MREST) keys=K0,K1 flags=
L65: items=3 lens=[0,72,72] witness_item_bytes=147 script_size=286 total_wit_bytes(items+script,no CB)=436 WITNESS_WU=437 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K5,K6]
L66: items=6 lens=[72,33<03462779..>,1,0,0,0] witness_item_bytes=112 script_size=286 total_wit_bytes(items+script,no CB)=401 WITNESS_WU=402 pubkeys_in_witness_items=[K3] pubkeys_in_script=[K0,K1,K2,K5,K6]
L67: items=7 lens=[72,33<0362c0a0..>,1,0,0,0,0] witness_item_bytes=113 script_size=286 total_wit_bytes(items+script,no CB)=402 WITNESS_WU=403 pubkeys_in_witness_items=[K4] pubkeys_in_script=[K0,K1,K2,K5,K6]
L68: items=9 lens=[32<07070707..>,0,71,72,0,0,0,0,0] witness_item_bytes=184 script_size=286 total_wit_bytes(items+script,no CB)=473 WITNESS_WU=474 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K5,K6]
L69: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=286 max_sat_size=Ok(183) max_wit_elems=Ok(8)
L70 sat wsh or_i(multi(2,K0,K1,K2),MREST) keys=K0,K1 flags=
L70: items=4 lens=[0,72,72,1] witness_item_bytes=149 script_size=286 total_wit_bytes(items+script,no CB)=438 WITNESS_WU=439 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K5,K6]
L71: items=4 lens=[72,33<03462779..>,1,0] witness_item_bytes=110 script_size=286 total_wit_bytes(items+script,no CB)=399 WITNESS_WU=400 pubkeys_in_witness_items=[K3] pubkeys_in_script=[K0,K1,K2,K5,K6]
L72: items=5 lens=[72,33<0362c0a0..>,1,0,0] witness_item_bytes=111 script_size=286 total_wit_bytes(items+script,no CB)=400 WITNESS_WU=401 pubkeys_in_witness_items=[K4] pubkeys_in_script=[K0,K1,K2,K5,K6]
L73: items=7 lens=[32<07070707..>,0,71,72,0,0,0] witness_item_bytes=182 script_size=286 total_wit_bytes(items+script,no CB)=471 WITNESS_WU=472 pubkeys_in_witness_items=[] pubkeys_in_script=[K0,K1,K2,K5,K6]
## S3b k-scaling, two-path, R=and_v(v:pkh(K20),older(100)) — head spend / R spend, WITNESS_WU
L75/L76 or_d(multi(1,K0,K1)):   178 / 213      L77/L78 or_i: 180 / 212
L79/L80 or_d(multi(2,K0,K1,K2)): 285 / 248      L81/L82 or_i: 287 / 246
L83/L84 or_d(multi(3,K0,K1,K2)): 358 / 249      L85/L86 or_i: 360 / 246
L87/L88 or_d(multi(9,K0..K8)):  1001 / 461      L89/L90 or_i: 1003 / 452
L88: items=12 lens=[72,33<03d79363..>,0,0,0,0,0,0,0,0,0,0] witness_item_bytes=117 script_size=340 total_wit_bytes(items+script,no CB)=460 WITNESS_WU=461 pubkeys_in_witness_items=[K20] pubkeys_in_script=[K0,K1,K2,K3,K4,K5,K6,K7,K8]
```

| head `multi(k,…)` | or_d − or_i on head spend | or_d − or_i on each other spend | or_d wins iff head share > |
| --- | --- | --- | --- |
| k=1 | −2 | +1 | 33 % |
| k=2 | −2 | +2 | 50 % |
| k=3 | −2 | +3 | 60 % |
| k=9 | −2 | +9 | 82 % |

No key is revealed by the multi dissatisfaction (`pubkeys_in_witness_items=[]` on every non-head spend, both forms); the multi keys are in the script under both.

### 2.4 Typing with 1, 2, 7 and 8 following paths (heads (a)/(b)/(c) in that order per block)

```
## S4-1 one following
L93: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=56 max_sat_size=Ok(142) max_wit_elems=Ok(5)
L94: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=56 max_sat_size=Ok(109) max_wit_elems=Ok(4)
L95: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=66 max_sat_size=Ok(108) max_wit_elems=Ok(4)
## S4-2 two following (locked multi, hashed single)
L97: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=203 max_sat_size=Ok(184) max_wit_elems=Ok(7)
L98: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=203 max_sat_size=Ok(150) max_wit_elems=Ok(6)
L99: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=213 max_sat_size=Ok(150) max_wit_elems=Ok(6)
## S4-7 seven following (8 paths total; mixed locked multi / locked single / hashed / hash+lock, height+time locks)
L101: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=618 max_sat_size=Ok(261) max_wit_elems=Ok(12)
L102: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=618 max_sat_size=Ok(227) max_wit_elems=Ok(11)
L103: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=628 max_sat_size=Ok(227) max_wit_elems=Ok(11)
## S4-8 eight following (9 paths; beyond the grammar's 8, typing only)
L105: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=689 max_sat_size=Ok(261) max_wit_elems=Ok(13)
L106: type=Bdsem sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=689 max_sat_size=Ok(227) max_wit_elems=Ok(12)
L107: type=Bsfm sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false script_size=699 max_sat_size=Ok(227) max_wit_elems=Ok(12)
## S4-8 head spend (b) vs (a)
L109: items=3 lens=[72,33<031b84c5..>,1] witness_item_bytes=109 script_size=689 total_wit_bytes(items+script,no CB)=801 WITNESS_WU=802 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6,K8,K9,K10,K12,K13]
L110: items=2 lens=[72,33<031b84c5..>] witness_item_bytes=107 script_size=689 total_wit_bytes(items+script,no CB)=799 WITNESS_WU=800 pubkeys_in_witness_items=[K0] pubkeys_in_script=[K1,K2,K3,K5,K6,K8,K9,K10,K12,K13]
```

### 2.5 `md encode` (0.14.0) on the placeholder templates — all accepted

```
wsh(or_d(pkh(@0/<0;1>/*),REST))           chunk-set-id: 0x73abb  exit=0
wsh(or_i(pkh(@0/<0;1>/*),REST))           chunk-set-id: 0xc3ec3  exit=0
wsh(or_d(pk(@0/<0;1>/*),REST))            chunk-set-id: 0xabbed  exit=0
wsh(or_i(pk(@0/<0;1>/*),REST))            chunk-set-id: 0xc830d  exit=0
wsh(or_d(multi(2,@0,@1,@2 /<0;1>/*),MREST)) chunk-set-id: 0x742ec  exit=0
wsh(or_i(multi(2,@0,@1,@2 /<0;1>/*),MREST)) chunk-set-id: 0xbb549  exit=0
```
(each with the standing "top-level wrapper has no canonical default derivation path … supply --path" warning; `REST`/`MREST` as in §2.2/§2.3 with `@n/<0;1>/*` keys and `sha256(4bb06f8e…94e0)`.)

## 3. Points 1–7

**1. Types and properties — MEASURED (§2.2 S2e, §2.4) + SOURCED (BIP-379).** Sub-arms: `pkh` = `c:pk_h` is `Bndusem`, `pk` is `Bondusem`, `multi(2,…)` is `Bndusem` (L47–L49); the `rest` chain of `and_v`s is `Bsfm` — not `d`, not `u`, `f` (L50). `or_d(X,Z)` needs X `Bdu` (bip-0379.md l.145) — all three heads qualify; result `d=dZ`, `u=uZ` (neither), `s=sX·sZ`, `f=fZ`, `e=eZ`, `m` iff `eX·(sX+sZ)` (l.201) → **`Bsfm`** for (a) and (c). `or_i(X,Z)` needs both `B` (l.146); `d=dX+dZ` (pkh is `d`), `s=sX·sZ`, `f=fX·fZ` (pkh not `f`), `e=eX·fZ + eZ·fX` (pkh `e`, rest `f`) (l.202) → **`Bdsem`** for (b). Measured identically at 1, 2, 7 and 8 following paths (L93–L107). Every candidate: `sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false`; the 9-path (c) is the largest script at 699 B, well inside Core's 3600 B / 100-element policy limits. Being `f` (a, c) versus `d`/`e` (b) at the top level changes nothing: `wsh` requires `B` plus sanity. Nothing here is malleable.

**2. Witness cost table — MEASURED (§2.2 table).** Cheapest for P1: (c) 373 WU, then (a) 397, then (b) 399. Cheapest for every other path: (b) 440/401/472, then (c) +10, then (a) +34. Deltas per input: (b) saves 34 WU (8.5 vB) over (a) on every non-P1 spend and loses 2 WU on P1; (c) saves 26 WU (6.5 vB) over (b) on P1 and loses 10 WU (2.5 vB) on every other spend. rust-miniscript's `max_weight_to_satisfy` (506 / 472 / 482) reproduces the P4 row exactly, so coordinator fee *estimates* (BDK/Liana use this bound) move by the same amounts.

**3. Privacy — MEASURED (reveal tracer).** Under (a) K0 is pushed on every non-P1 spend (`pubkeys_in_witness_items=[K0]` L14–L16, and at every nesting level — the first review's L11). Under (b) K0 appears nowhere on a non-P1 spend (L20–L22: `[]`/`[K4]` in items, K0 absent from script). Under (c) K0 is in the script on every spend (L25–L28). So yes, (b) actually keeps K0 hidden on non-P1 spends. What `rest` leaks on a P1 spend is identical under all three: the whole witnessScript — every `multi` pubkey in cleartext (K1,K2,K3,K5,K6 in `pubkeys_in_script` on L13/L19/L25), the HASH160 of every `pkh` key (K4), all locks and the sha256 image. Only `pkh`-spelled single keys of unspent paths stay hidden (K4 is never revealed except on its own spend). The `pkh` benefit is therefore confined to single-key paths and, for the head, to recovery spends.

**4. `pkh`@`or_i` vs `pk`@`or_d` for the operator's mix — MEASURED, break-even derived.** (c) is cheaper whenever P1 spends exceed 10/(26+10) = **27.8 %** of spends; a daily key is ~100 %, so (c) beats (b) by ≈26 WU (6.5 vB) per P1 input and (b) beats (c) by 10 WU per recovery input. At 10 sat/vB that is ~65 sat per daily input. This is the exact, recurring price of C17, and the privacy it buys materialises only on recovery spends (K0 hidden) — the rare path by construction. I do not re-open C17; I record that (b) is the only lowering consistent with it, and that (c) is the drop-in if it is revisited (BIP-388's own example, Liana's generated form `wsh(or_d(pk(primary),and_v(v:pkh(recovery),older(N))))` — liana_desc.rs l.862, l.971 — and the compiler's output under both uniform and 99:1 P1-weighted probabilities, L61/L62). (a) is not a middle ground: it pays the `pkh` cost and gets none of the benefit.

**5. Interop — SOURCED, none executed against a live wallet/device.**
- *Bitcoin Core:* `miniscript.h` l.235–236 defines both fragments, l.843/845 builds `[X] IFDUP NOTIF [Z] ENDIF` / `IF [X] ELSE [Z] ENDIF`, l.1220–1221 gives the witness-size formulas that match my measurements (`OR_I: (X.sat+1+1) | (Z.sat+1)`, `OR_D: X.sat | (X.dsat+Z.sat)`). No fragment-specific policy beyond BIP-379; `IsSane` treats both identically.
- *Ledger app-bitcoin-new:* `wallet.c` l.55–56 tokens `or_d`/`or_i`, l.29/l.42 `pkh`/`pk_h`, l.1567 "`pkh(key) == c:pk_h(key)`", l.2587–2640 recomputes exactly the BIP-379 `s/f/e/m` rules quoted above, l.1350–1366 only checks both children are miniscript and X is not `W`. `wallet.md` l.33: any "valid SegWit miniscript template" inside `wsh`. Both (b) and (c) register; the device shows the template text.
- *Liana:* import is **lift-based** — `analysis.rs` l.578–589 (`ms.lift()`), l.623 (top-level must be an `or`), l.640–660 (primary = single key or multisig). `or_d` and `or_i` are indistinguishable after lifting; its tests import `or_d(pkh(…),…)` (liana_desc.rs l.2309, l.2350) and `or_i(…,or_i(…,or_d(…)))` (l.1460). But recovery paths must be `thresh(2, older(x), keys)` or n-of-n with `older` (analysis.rs l.208–260): a composer wallet with an `after` or `sha256` path is `IncompatibleDesc` regardless of head — a grammar fact, not a head fact.
- *Nunchuk:* libnunchuk `src/miniscript/miniscript.cpp` is Core's implementation verbatim (header l.1 "Copyright (c) 2019-present The Bitcoin Core developers"; `OR_D`/`OR_I` at l.182/199/291–292). Whether its UI's "checks for correctness" or plain-language rendering distinguishes the two: **UNVERIFIED**.
- *Sparrow:* no miniscript (first review); not re-checked.
- *md 0.14.0:* MEASURED — all six templates encode (§2.5), distinct chunk-set-ids, so the head choice is baked into the template identity and the addresses (L52–L54 differ): **once shipped, changing the head fragment is a wallet-format change.**

**6. General head rule — MEASURED (§2.3).** `or_d` over `multi(k,…)` dissatisfies with k+1 empty pushes versus `or_i`'s one selector byte: **+k WU on every non-head spend, −2 WU on the head spend**, confirmed for k = 1, 2, 3, 9 (+1/+2/+3/+9; −2 each). Break-even head share k/(k+2) (33 %…82 %). For the grammar's head — first-listed, unlocked, unhashed, i.e. the primary path — `or_d` wins by 2 WU per spend on the path that is actually used, leaks nothing (multi keys are in the script either way), and matches BIP-388 l.249 and Liana's `or_d(multi(…),…)` (liana_desc.rs l.912, l.1451, l.1618). Uniform `or_i` would cost 2 WU per primary spend and save k WU per recovery spend; both are sub-vbyte and neither needs a new emitter arm (`or_d` and `or_i` both exist; only `andor` does not). **Keep `or_d` for the bare multi head** — the principle that makes the rule non-arbitrary is: *use `or_d` when the head's dissatisfaction is empty pushes only (`multi`; `pk` would qualify but C17 excludes it), `or_i` when the dissatisfaction would push key material (`pkh`) or does not exist / is malleable (`and_v`, `sha256`).*

**7. Ruling.** Single-key head: **(b) `or_i(pkh(P1), rest)`**. Multi head: **`or_d(multi(k,…), rest)`**. Rejected: (a) dominated (+34 WU and leaks K0 on every non-P1 spend for −2 WU on P1); (c) fee-optimal but contradicts C17 (K0 in the script on every spend); (d2) `or_i(rest, pkh(P1))` saves 1 WU on P1 and costs 1 WU elsewhere while moving P1's key to the last placeholder under C1's first-appearance numbering; (d3) `t:or_c` and (d4) `andor(pkh,1,rest)` have (a)'s exact cost and leak plus 1 B, and `andor` has no emitter arm.

## 4. Minimal rule text

wsh `paths combine` row:

> Listed order, recursive, last path stands alone. At **every** level, head `P` combines with the remaining chain `R` as `or_d(P, R)` **iff `P` is a bare `multi(k,…)` (unlocked, unhashed, n ≥ 2)**; otherwise `or_i(P, R)` — so a bare single key is `or_i(pkh(K), R)`, and locked, hashed or keyless heads are `or_i`. Never `andor`, never `thresh` over paths.
>
> *Why:* `or_d` dissatisfies its head on every other path's spend. For `multi` that is k+1 empty pushes (−2 WU on the head spend, +k WU elsewhere, no key revealed). For `pkh` it is `<empty sig> <pubkey>`: +34 WU on every other spend **and** the head key published, which defeats `pkh`. The BIP-388 / compiler form `or_d(pk(K), R)` is 26 WU cheaper per head spend and 10 WU dearer elsewhere, with K in the script on every spend — the drop-in if C17 (`pkh` for one key in wsh) is ever revisited.

## 5. What I ran

- Extended the scratchpad probe (`probe/patch.py` → `probe/src/main.rs`): `satisfy()` now prints `WITNESS_WU` (count varint + items + script item) and byte-matches every known pubkey against the witness items and the witnessScript; added `H2..H4` distinct sha256 images. Built against the fork at `2092faa` (`cargo build`, opt-level 1); ran `probe/cases3.txt` (89 case lines: S1 baseline, S2 4-path (a)(b)(c)(d1)(d2)(d3)(d4) + sub-arm types + descriptors + lifteq + compiler, S3 multi head + k = 1/2/3/9 scaling, S4 typing at 1/2/7/8 following paths) → `probe/out3.txt` (254 lines, 0 `ERR`).
- `probe/md-encode.sh`: `md encode` on the six placeholder templates → `probe/md-encode.out`.
- Read: BIP-379 l.95–96, 145–146, 201–202, 250–251; BIP-388 l.249; Core `miniscript.h` l.235–236, 843–845, 1220–1221; Ledger `wallet.c` l.29, 42, 55–56, 1278–1366, 1567, 2587–2640 and `wallet.md` l.33–36; Liana `analysis.rs` l.72–78, 186–260, 570–660 and `descriptors/mod.rs` test lines 862, 898, 912, 971, 1451, 1460, 1618, 2309, 2350; libnunchuk `miniscript.cpp` l.1, 63, 182, 199, 291–292; the first review (I1, M3, §3 table) and brainstorm §2 C16/C17/C19, §3.7, §3.8.
- Not done: no bitcoind/Liana/Ledger/Nunchuk execution; no repo file modified.
