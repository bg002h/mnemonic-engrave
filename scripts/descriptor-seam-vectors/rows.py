# -*- coding: utf-8 -*-
"""Row DEFINITIONS for descriptor_seam_vectors.json.

Host-side columns (host_admits, md1_admits, format, the gate fields, covers)
are AUTHORED from SPEC_descriptor_input.md.  Every device-side and value
column (device_admits, canonical, address_0/1, wallet_id, md_descriptor_contains)
is left for gen.py to MEASURE -- nothing here is transcribed from a report.
"""

CH = [l.strip() for l in open(__file__.rsplit("/", 1)[0] + "/children21.txt")]
K16 = CH[:16]
K21 = CH[:21]

K0 = "xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan"
K1 = "xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge"
K2 = "xpub6DjrnfAyuonMaboEb3ZQZzhQ2ZEgaKV2r64BFmqymZqJqviLTe1JzMr2X2RfQF892RH7MyYUbcy77R7pPu1P71xoj8cDUMNhAMGYzKR4noZ"
K1T = "tpubDEA5aTqMAQTRAxkMULa1ZfBT3GFEXqnFMnDR2GPwAEqmoauXvdgUW38mMnCeDaqoLvmzHhuBMyhtYkJFzr2Sb2AbvMzECU8gpJm13rW2CcW"
F0, F1, F2 = "dc567276", "f245ae38", "c5d87297"
ORG = "48h/0h/0h/2h"

BW0 = "xpub6F148LnjUhGrHfEN6Pa8VkwF8L6FJqYALxAkuHfacfVhMLVY4MRuUVMxr9pguAv67DHx1YFxqoKN8s4QfZtD9sR2xRCffTqi9E8FiFLAYk8"
BW1 = "xpub6DnediUuY8Pcc6Fej8Yt2ZntPCyFdpbHBkNV7EawesRMbc6i9MKKMhKEv4JMMzwDJckaV4czBvNdc6ikwLiZqdUqMd5ZKQGYaQT4cXMeVjf"
BW2 = "xpub6EefrCrMAduhNwnsHb3dAs8DYZSw4f63WyR6DaEByUHjwvPDdhczj15FyBBG4tbEJtf4vRKTv1ng5SPPnWv1Pve1f15EJfiBY5oYDN6VLEC"

SK = "xpub6C9j4wAxxkWN4cq8G4N2mkV6NrGGhnLFCGdh8GsYY1xreEveW5YEXJMjDZWLAcnZ26xqVft5FmgBxPixdMGoVQZMdtEJRRADxrn4facoGnx"
SKZL = "zpub6qpFgGWoG7bKmDDMvmwHBvg6inZAb2KF2Vg8h4fKJ2ickSZ71PsMmRg1FyRWAS6PqPCSzd5CB6PHixx64k6q5svZNZd9bEoCWJuMSkSRzJx"
SKZU = "Zpub72iLoWFEq59hBnNjsSQG211uSabRoNzqLmKocKvrfoZ2Nd81moFdrYXw4gNyisKJ4rGRsD5K4Jmnr8ZrMyFnEN3ED2jYzeGCQ3BE2fiCsDJ"
SKYU = "Ypub6ht5VqaKgPcDLVBd35cdouvQGcSyrm1LReoapw2yHoB9KXJnX965EUso3URPixfNfD9d7jUkbeRExqxHeGqmS8MdLh38QjSi8K7ae5rcihQ"
SKYL = "ypub6WyzNbqt7S3quv2F6R9eyqabYpQieQKk7P9uufmRv2LjhLjskjho9N1sEmTvAXSURk5eF9UdiS2jqgLXM3gpHeExWDvj1KyiEaqi47h3Ef1"
SKT = "tpubDCXMbAzeg2TpLR1yiFM7yfpThyMvhAqJjuDzUpvgsvikPXbMaJPKfk2ZTbb7h7jnp1Vk7FPwnsWEeaDa2D83Nr1ehUyc6wpTYpNURb6Qt26"
SKFP = "4bbaa801"

MNEMONIC = "abandon " * 11 + "about"
MNEMONIC = MNEMONIC.strip()

FORK_PARSE_TEST = "seedhammer nonstandard/parse_test.go @ d402f18"
FORK_BIP380 = "seedhammer bip380/bip380.go @ d402f18, measured"
SPEC = "SPEC_descriptor_input.md"


def key(fp, org, k, tail=""):
    return "[%s/%s]%s%s" % (fp, org, k, tail)


def sm(k, keys, wrap="wsh", form="sortedmulti"):
    inner = "%s(%d,%s)" % (form, k, ",".join(keys))
    if wrap == "wsh":
        return "wsh(%s)" % inner
    if wrap == "sh":
        return "sh(%s)" % inner
    if wrap == "shwsh":
        return "sh(wsh(%s))" % inner
    raise ValueError(wrap)


def std(tail="/<0;1>/*", n=3, form="sortedmulti", wrap="wsh", k=2):
    ks = [key(F0, ORG, K0, tail), key(F1, ORG, K1, tail), key(F2, ORG, K2, tail)][:n]
    return sm(k, ks, wrap, form)


BW_SH_FIXTURE = (
    "# BlueWallet Multisig setup file\n"
    "# this file contains only public keys and is safe to\n"
    "# distribute among cosigners\n"
    "#\n"
    "Name: sh\n"
    "Policy: 2 of 3\n"
    "Derivation: m/48'/0'/0'/2'\n"
    "Format: P2WSH\n"
    "\n"
    "5A0804E3: " + BW0 + "\n"
    "\n"
    "DD4FADEE: " + BW1 + "\n"
    "\n"
    "9BACD5C0: " + BW2 + "\n"
)

JSON_FIXTURE = (
    '{\n'
    '  "label": "Test Multisig 2-of-3",\n'
    '  "blockheight": 481824,\n'
    '  "descriptor": "' + std("/0/*") + '#hfwurrvt",\n'
    '  "devices": [{"type": "other", "label": "Test Multisig 2-of-3 Cosigner 1"}]\n'
    '}\n'
)


def G(open_, outcome, exit_code, refusal_row=None):
    d = {"gate_open": open_, "outcome": outcome, "exit_code": exit_code}
    if refusal_row is not None:
        d["refusal_row"] = refusal_row
    return d


# ---------------------------------------------------------------------------
# Each entry: dict with the AUTHORED fields.  gen.py fills the measured ones.
#   want_addr   -- derive and carry address_0 (+ address_1 when "01")
#   want_wid    -- carry wallet_id (multisig at the device-default use-site)
#   md1_route   -- (template, keys, fingerprints, path) for the Rust md1 route
#   canonical_from -- device-parser input for `canonical` (defaults to `input`)
# ---------------------------------------------------------------------------
ROWS = []


def R(**kw):
    ROWS.append(kw)


# ---- formats-happy (3 new physical rows; the 4th is promotion/bare-xpub) ----
R(name="formats-happy/bluewallet-sh-fixture", input=BW_SH_FIXTURE,
  host_admits=True, md1_admits=True, format="bluewallet",
  source=FORK_PARSE_TEST + " (the shipped `sh` happy path)",
  covers=["formats-happy"],
  want_addr="01", want_wid=True,
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
             [BW0, BW1, BW2], ["5a0804e3", "dd4fadee", "9bacd5c0"], "m/48'/0'/0'/2'"))

R(name="formats-happy/bip380-sortedmulti-multipath", input=std(),
  host_admits=True, md1_admits=True, format="bip380",
  source=FORK_BIP380 + "; keys from " + FORK_PARSE_TEST,
  covers=["formats-happy"],
  want_addr="01", want_wid=True,
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
             [K0, K1, K2], [F0, F1, F2], "m/48'/0'/0'/2'"))

R(name="formats-happy/json-label-descriptor", input=JSON_FIXTURE,
  host_admits=True, md1_admits=False, format="json",
  source=FORK_PARSE_TEST + " (the shipped JSON fixture; /0/* -- see md1-split/fixed-index)",
  covers=["formats-happy"], want_addr="01")

# ---- promotion-near-miss: the fifteen rows of the S4.5 table, in order -----
# Every one also carries `gate` (gate bullet clause 1).
R(name="promotion/01-bare-xpub", input=SK,
  host_admits=True, md1_admits=True, format="promoted-key",
  source=FORK_PARSE_TEST + " (bare-key promotion fixture)",
  covers=["formats-happy", "promotion-near-miss", "gate"],
  gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("pkh(@0/<0;1>/*)", [SK], None, "m/44'/0'/0'"))

R(name="promotion/02-bare-zpub", input=SKZL,
  host_admits=True, md1_admits=True, format="promoted-key",
  source=FORK_PARSE_TEST + " (bare zpub; the key re-serialises to xpub)",
  covers=["promotion-near-miss", "gate"], gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("wpkh(@0/<0;1>/*)", [SK], None, "m/84'/0'/0'"))

R(name="promotion/03-bare-Zpub-refused", input=SKZU,
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (version => 48'/0'/0'/2', not in the promotion loop)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-multisig-cosigner-key"))

R(name="promotion/04-bare-Ypub-refused", input=SKYU,
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (version => 48'/0'/0'/1', not in the promotion loop)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-multisig-cosigner-key"))

R(name="promotion/05-origin-44h", input="[%s/44'/0'/0']%s" % (SKFP, SK),
  host_admits=True, md1_admits=True, format="promoted-key",
  source=SPEC + " S4.5 (P2PKH path)", covers=["promotion-near-miss", "gate"],
  gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("pkh(@0/<0;1>/*)", [SK], [SKFP], "m/44'/0'/0'"))

R(name="promotion/06-origin-49h", input="[%s/49'/0'/0']%s" % (SKFP, SK),
  host_admits=True, md1_admits=True, format="promoted-key",
  source=SPEC + " S4.5 (P2SH_P2WPKH path)", covers=["promotion-near-miss", "gate"],
  gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("sh(wpkh(@0/<0;1>/*))", [SK], [SKFP], "m/49'/0'/0'"))

R(name="promotion/07-origin-84h-zpub", input="[%s/84'/0'/0']%s" % (SKFP, SKZL),
  host_admits=True, md1_admits=True, format="promoted-key",
  source=FORK_PARSE_TEST + " (the origin-annotated zpub fixture)",
  covers=["promotion-near-miss", "gate"], gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("wpkh(@0/<0;1>/*)", [SK], [SKFP], "m/84'/0'/0'"))

R(name="promotion/08-origin-86h-refused", input="[%s/86'/0'/0']%s" % (SKFP, SK),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (taproot single-sig is not promotable)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-path-not-inferable"))

R(name="promotion/09-origin-48h-cosigner-refused", input="[%s/48'/0'/0'/2']%s" % (SKFP, SK),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (a multisig cosigner key is not a wallet)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-path-not-inferable"))

R(name="promotion/10-account-one-refused", input="[%s/84'/0'/1']%s" % (SKFP, SKZL),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (only account 0 qualifies)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-account-not-zero"))

R(name="promotion/11-origin-unhardened-refused", input="[%s/84/0/0]%s" % (SKFP, SKZL),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (the comparison is on hardened values)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-path-not-inferable"))

R(name="promotion/12-fingerprint-no-path-refused", input="[%s]%s" % (SKFP, SK),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 (ParseKey needs originAndPath[8] == '/')",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-fingerprint-no-path"))

R(name="promotion/13-children-no-origin", input=SK + "/<0;1>/*",
  host_admits=True, md1_admits=True, format="promoted-key",
  source=SPEC + " S4.5 (children do not affect the origin comparison)",
  covers=["promotion-near-miss", "gate"], gate=G(True, "as-decides", 2), want_addr="0",
  md1_route=("pkh(@0/<0;1>/*)", [SK], None, "m/44'/0'/0'"))

R(name="promotion/14-bare-xpub-trailing-newline", input=SK + "\n",
  canonical_from=SK,
  host_admits=True, md1_admits=True, format="promoted-key",
  source=SPEC + " S4.5 + S4.6 (the host trims; the device refuses the raw bytes)",
  covers=["promotion-near-miss", "whitespace", "gate"],
  gate=G(True, "as-decides", 2), want_addr="0", addr_from=SK,
  md1_route=("pkh(@0/<0;1>/*)", [SK], None, "m/44'/0'/0'"))

R(name="promotion/15-bare-tpub-host-refused", input=SKT,
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.5 NORMATIVE ruling (host refuses tpub promotion entirely)",
  covers=["promotion-near-miss", "gate"],
  gate=G(True, "descriptor-refusal", 3, "promotion-testnet-key"))

# ---- narrowed-4.7: every shape S4.7 narrows (host false / device true) -----
NAR = SPEC + " S4.7 (host narrows; device measured ACCEPT)"
R(name="narrowed/tr-sortedmulti", input=std(n=2, wrap="wsh").replace("wsh(", "tr(", 1),
  host_admits=False, md1_admits=False, format="bip380", source=NAR, covers=["narrowed-4.7"])
R(name="narrowed/wpkh-sortedmulti", input=std(n=2).replace("wsh(", "wpkh(", 1),
  host_admits=False, md1_admits=False, format="bip380", source=NAR, covers=["narrowed-4.7"])
R(name="narrowed/pkh-sortedmulti", input=std(n=2).replace("wsh(", "pkh(", 1),
  host_admits=False, md1_admits=False, format="bip380", source=NAR, covers=["narrowed-4.7"])
R(name="narrowed/sh-wpkh-sortedmulti", input="sh(wpkh(%s))" % std(n=2)[4:-1],
  host_admits=False, md1_admits=False, format="bip380", source=NAR, covers=["narrowed-4.7"])
R(name="narrowed/wsh-of-key", input="wsh(%s)" % key(F0, ORG, K0, "/<0;1>/*"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; Singlesig, address.Supported false", covers=["narrowed-4.7"])
R(name="narrowed/sh-of-key", input="sh(%s)" % key(F0, ORG, K0, "/<0;1>/*"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; Singlesig, address.Supported false", covers=["narrowed-4.7"])
R(name="narrowed/threshold-zero", input=std(n=2, k=0),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; k=0 is anyone-can-spend", covers=["narrowed-4.7"])
R(name="narrowed/threshold-negative", input=std(n=2, k=-1),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; strconv.Atoi accepts a sign", covers=["narrowed-4.7"])
R(name="narrowed/threshold-exceeds-keys", input=std(n=2, k=5),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; unsatisfiable", covers=["narrowed-4.7"])
R(name="narrowed/sh-sortedmulti-16-keys",
  input=sm(2, ["[%08x/%s]%s/<0;1>/*" % (i + 1, ORG, k) for i, k in enumerate(K16)], "sh"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; BIP-383: <=15 keys directly under sh (547-byte redeemScript)",
  covers=["narrowed-4.7"])
R(name="narrowed/wsh-sortedmulti-21-keys",
  input=sm(2, ["[%08x/%s]%s/<0;1>/*" % (i + 1, ORG, k) for i, k in enumerate(K21)], "wsh"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; OP_CHECKMULTISIG's 20-key limit", covers=["narrowed-4.7"])
R(name="narrowed/mixed-network",
  input=sm(2, [key(F0, ORG, K0, "/<0;1>/*"), key(F1, ORG, K1T, "/<0;1>/*")]),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; address.Receive: multisig descriptor mixes networks", covers=["narrowed-4.7"])
R(name="narrowed/use-site-hardened", input=std(n=2, tail="/<0;1>/*h"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; the device derives the UNhardened child", covers=["narrowed-4.7"])
R(name="narrowed/use-site-non-consecutive", input=std(n=2, tail="/<0;2>/*"),
  host_admits=False, md1_admits=False, format="bip380",
  source=NAR + "; address.Receive errors, address.Supported still true", covers=["narrowed-4.7"])

# ---- accepted-extreme ------------------------------------------------------
R(name="accepted/sh-wsh-sortedmulti-16-keys",
  input=sm(2, ["[%08x/%s]%s/<0;1>/*" % (i + 1, ORG, k) for i, k in enumerate(K16)], "shwsh"),
  host_admits=True, md1_admits=True, format="bip380",
  source=SPEC + " S4.7 conjunct 3 (r6's recorded construction: 16 unhardened "
                "children of the dc567276 fixture key; sh(wsh(...)) redeemScript is 34 bytes)",
  # NO wallet_id: md-cli's depth-3-or-4 key guard (crates/md-cli/src/parse/keys.rs:132)
  # refuses these depth-5 children, so the Rust side of the cross-language id
  # cannot be MEASURED in P0. That guard is the CLI's alone -- it does not bind
  # `me`, which builds the md_codec AST in process, and the guard's own comment
  # records that widening it "cannot move a wallet id or change an encoded md1
  # string". So md1_admits stays TRUE and the addresses are the device route's.
  covers=["accepted-extreme"], want_addr="01")

# ---- narrowed-4.2 ----------------------------------------------------------
BW42 = SPEC + " S4.2 (host refuses; device measured)"
R(name="bluewallet/no-format-header",
  input=BW_SH_FIXTURE.replace("Format: P2WSH\n", ""),
  host_admits=False, md1_admits=False, format="none",
  source=BW42 + " defect 1: parse ACCEPTS, Descriptor.Encode() panics",
  covers=["narrowed-4.2"], device_probe="panic:encode")
R(name="bluewallet/zero-cosigner-lines", input="Name: only\n",
  host_admits=False, md1_admits=False, format="none",
  source=BW42 + " defect 2: exact `Name: only` spelling; 0 keys, Script=Unknown",
  covers=["narrowed-4.2"], device_probe="panic:encode")
R(name="bluewallet/derivation-after-keys",
  input="Name: after\nPolicy: 2 of 2\nFormat: P2WSH\n\n%s: %s\n%s: %s\nDerivation: m/48'/0'/0'/2'\n"
        % ("dc567276", K0, "f245ae38", K1),
  host_admits=False, md1_admits=False, format="none",
  source=BW42 + " defect 3: every key origin empty; the canonical does not re-parse",
  covers=["narrowed-4.2"])
R(name="bluewallet/no-derivation-header",
  input="Name: nodrv\nPolicy: 2 of 2\nFormat: P2WSH\n\n%s: %s\n%s: %s\n"
        % ("dc567276", K0, "f245ae38", K1),
  host_admits=False, md1_admits=False, format="none",
  source=BW42 + " R0's C1: device ACCEPTS the input, its canonical does not re-parse",
  covers=["narrowed-4.2"])
R(name="bluewallet/short-fingerprint",
  input="Name: shortfp\nPolicy: 1 of 1\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n\nab: %s\n" % K0,
  host_admits=False, md1_admits=False, format="none",
  source=BW42 + " defect 4: a fingerprint shorter than 4 bytes used to reach "
                "binary.BigEndian.Uint32 and PANIC; S2's convergence fix (P3.1) makes "
                "the guard `!= 4` and the parser error cleanly, so device_admits is "
                "MEASURABLE and the device_probe marker retired with it",
  covers=["narrowed-4.2"])

# ---- neither ---------------------------------------------------------------
R(name="neither/wsh-multi", input=std(n=2, form="multi"),
  host_admits=False, md1_admits=True, format="bip380",
  source=SPEC + " S4.3/S4.7 conjunct 1: device REFUSES multi; md1 carries it natively",
  covers=["neither"], want_addr="01", md_descriptor_contains="wsh(multi(",
  md1_route=("wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))", [K0, K1], [F0, F1], "m/48'/0'/0'/2'"))
R(name="neither/miniscript",
  input="wsh(or_d(pk(%s),and_v(v:pkh(%s),older(52560))))" % (K0, K1),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.3/S10: miniscript is out of scope for both paths", covers=["neither"])
# S11 item 5 case 3's witness: admitted by NEITHER carrier, so the S5.4 carriage
# rule fires the input's own refusal directly rather than a two-option menu.
# host_admits=false is conjunct 1's PERMANENT `multi` refusal; md1_admits=false
# is the fixed `/0/*` use-site (F-417).  The shipped witness, md1-split/
# fixed-index, became CARRIED when `--as descriptor` shipped in S2.
R(name="neither/wsh-multi-fixed-path", input=std(tail="/0/*", n=2, form="multi"),
  host_admits=False, md1_admits=False, format="bip380",
  source=SPEC + " S4.7 conjunct 1 (multi: permanent) AND S5.3(a) (/0/* has no md1 "
                "form): the S11 item 5 case-3 witness, admitted by neither carrier",
  covers=["neither"])

# ---- version-gap -----------------------------------------------------------
# F-426's live witness: S4.3 REFUSES the version host-side while the device's
# scan door accepts it (P3.4's ypubVer case).  A single-member bullet by
# construction -- when F-426's host half widens, host_admits flips and the
# bullet retires with it.  It cannot stay `neither`: S7 defines that tag as
# rows NEITHER side admits, and this row's device_admits is true.
R(name="version-gap/full-origin-ypub",
  input="sh(wpkh([%s/49h/0h/0h]%s/<0;1>/*))" % (SKFP, SKYL),
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S4.3: `me` admits five versions and `ypub` is not one -- refused "
                "even with a full explicit origin. The DEVICE's parser accepts it "
                "after P3.4's ypubVer case (F-426 device half), which is why the row "
                "is the version-gap witness rather than a `neither` row",
  covers=["version-gap"])

# ---- whitespace (2 new; the third is promotion/14) -------------------------
R(name="whitespace/crlf-bip380", input=std().replace("\n", "") + "\r\n",
  canonical_from=std(),
  host_admits=True, md1_admits=True, format="bip380",
  source=SPEC + " S4.6: CRLF is REFUSED by branch 2; the host normalises first",
  covers=["whitespace"], want_addr="0", addr_from=std(),
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
             [K0, K1, K2], [F0, F1, F2], "m/48'/0'/0'/2'"))
R(name="whitespace/leading-space-bip380", input=" " + std(),
  canonical_from=std(),
  host_admits=True, md1_admits=True, format="bip380",
  source=SPEC + " S4.6: a leading space is REFUSED by branch 2; the host trims",
  covers=["whitespace"], want_addr="0", addr_from=std(),
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
             [K0, K1, K2], [F0, F1, F2], "m/48'/0'/0'/2'"))

# ---- md1-splits ------------------------------------------------------------
S53 = SPEC + " S5.3"
R(name="md1-split/fixed-index", input=std(n=2, tail="/0/*"),
  host_admits=True, md1_admits=False, format="bip380",
  source=S53 + "(a): md1 has no representation for a single fixed child index",
  covers=["md1-splits"], want_addr="0")
R(name="md1-split/multipath-no-wildcard", input=std(n=2, tail="/<0;1>"),
  host_admits=True, md1_admits=False, format="bip380",
  source=S53 + "(a''): md encode collapses <0;1> into <0;1>/*",
  covers=["md1-splits"], want_addr="0")
R(name="md1-split/childless", input=std(n=2, tail=""),
  host_admits=True, md1_admits=True, format="bip380",
  source=S53 + "(a'): the device defaults an empty children list to <0;1>/*",
  covers=["md1-splits"], want_addr="0", want_wid=True,
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", [K0, K1], [F0, F1], "m/48'/0'/0'/2'"))
R(name="md1-split/mixed-fixed-and-multipath",
  input=sm(2, [key(F0, ORG, K0, "/0/*"), key(F1, ORG, K1, "/<0;1>/*")]),
  host_admits=True, md1_admits=False, format="bip380",
  source=S53 + " per-key quantifier (R0 r3's NEW-C1)", covers=["md1-splits"], want_addr="0")
R(name="md1-split/mixed-nowildcard-and-multipath",
  input=sm(2, [key(F0, ORG, K0, "/<0;1>"), key(F1, ORG, K1, "/<0;1>/*")]),
  host_admits=True, md1_admits=False, format="bip380",
  source=S53 + " per-key quantifier (R0 r3's NEW-C1)", covers=["md1-splits"], want_addr="0")
R(name="md1-split/mixed-childless-and-multipath",
  input=sm(2, [key(F0, ORG, K0, ""), key(F1, ORG, K1, "/<0;1>/*")]),
  host_admits=True, md1_admits=True, format="bip380",
  source=S53 + " per-key materialisation (R0 r3's NEW-C1)", covers=["md1-splits"],
  want_addr="0", want_wid=True,
  md1_route=("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", [K0, K1], [F0, F1], "m/48'/0'/0'/2'"))

# ---- gate: 22 new physical rows (clauses 2-8) ------------------------------
GATE2 = SPEC + " S7 gate clause 2: record payloads carrying parentheses, colons or base58"
for nm, s in [
    ("gate/record-text-parentheses", "text: my wallet (2 of 3)"),
    ("gate/record-pass-parentheses", "pass: hunter (2)"),
    ("gate/record-text-inner-colon", "text: note: hello"),
    ("gate/record-seed-mistyped-mnemonic", "seed: abandon abandon abandoz"),
    ("gate/record-tx-zz", "tx: zz"),
    ("gate/record-text-xpub-payload", "text: " + SK),
]:
    R(name=nm, input=s, host_admits=False, md1_admits=False, format="none",
      source=GATE2, covers=["gate"], gate=G(False, "record-refusal", 4))

R(name="gate/mistyped-bare-mnemonic",
  input="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandoz",
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 3; the operand class of sysw_cli.rs:1928",
  covers=["gate"], gate=G(False, "record-refusal", 4))
R(name="gate/deadbeef-fronts-an-xpub", input="deadbeef: " + SK,
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 3 (r17's intended flip; invariant-1 boundary witness). "
                "refusal_row is NOT the device's precedence -- measured at fork 1f09537, "
                "parseBlueWalletDescriptor FAILS this file with `bluewallet: expected 0 "
                "keys, but got 1` (no Policy: header, so nkeys=0 while one key was "
                "appended; parse.go:158 fires before the Title gate at :37). "
                "bluewallet-no-name is nonetheless the right answer by S6's own standard: "
                "the count row's text names a Policy: line this file does not have, so it "
                "would be FALSE about the operator's file. Corrected per IMPL-P1-report F-2",
  covers=["gate"], gate=G(True, "descriptor-refusal", 3, "bluewallet-no-name"))

for nm, s in [
    ("gate/malformed-md1", "md1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzzzzzz"),
    ("gate/malformed-mk1", "mk1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzzzzzz"),
    ("gate/malformed-ms1", "ms1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzzzzzz"),
    ("gate/malformed-mt1", "mt1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzzzzzz"),
]:
    R(name=nm, input=s, host_admits=False, md1_admits=False, format="none",
      source=SPEC + " S7 gate clause 4: the bech32 charset is not a base58check envelope",
      covers=["gate"], gate=G(False, "record-refusal", 4))

R(name="gate/multi-record-mnemonic-first", input=MNEMONIC + "\n" + std(n=2) + "\n",
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 5: the MNEMONIC-FIRST ordering is load-bearing (r19)",
  covers=["gate"], gate=G(True, "multi-record", 4, "multi-record-descriptor"))

for nm, tail in [
    ("gate/buried-bare-Zpub", SKZU),
    ("gate/buried-bare-tpub", SKT),
    ("gate/buried-fingerprint-no-path", "[%s]%s" % (SKFP, SK)),
]:
    R(name=nm, input=MNEMONIC + "\n" + tail + "\n",
      host_admits=False, md1_admits=False, format="none",
      source=SPEC + " S7 gate clause 6 + S6's scope paragraph: the whole input does not "
                    "parse as one descriptor and no individual record does, so step 5's "
                    "generic four-forms text fires",
      covers=["gate"], gate=G(True, "descriptor-refusal", 3, "unparseable"))

R(name="gate/base58check-77-byte-payload", input="__B58_77__",
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 7: valid checksum, 77-byte payload -- one byte short "
                "of an extended-key envelope, so T3's leading-segment test fails",
  covers=["gate"], gate=G(False, "record-refusal", 4))
R(name="gate/lone-open-bracket", input="[",
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 7", covers=["gate"],
  gate=G(True, "descriptor-refusal", 3, "unparseable"))
R(name="gate/xpub-trailing-slash", input=SK + "/",
  host_admits=False, md1_admits=False, format="none",
  source=SPEC + " S7 gate clause 7", covers=["gate"],
  gate=G(True, "descriptor-refusal", 3, "unparseable"))

R(name="gate/colliding-origin-sortedmulti",
  input=sm(2, [key(F0, ORG, K0, "/<0;1>/*"), key(F0, ORG, K1, "/<0;1>/*")]),
  host_admits=False, md1_admits=False, format="bip380",
  source=SPEC + " S4.7 conjunct 8 / S7 clause 8: one (fingerprint, origin) naming two "
                "keys. NO address fields -- a colliding-origin wallet derives "
                "byte-identical addresses to a clean control, so the refusal is the witness",
  covers=["gate"], gate=G(True, "descriptor-refusal", 3, "key-identity"))
R(name="gate/duplicate-key-same-use-site",
  input=sm(2, [key(F0, ORG, K0, "/<0;1>/*"), key(F0, ORG, K0, "/<0;1>/*")]),
  host_admits=False, md1_admits=False, format="bip380",
  source=SPEC + " S4.7 conjunct 8 / S7 clause 8: the same (xpub, use-site) in two slots",
  covers=["gate"], gate=G(True, "descriptor-refusal", 3, "key-identity-duplicate"))
R(name="gate/colliding-origin-multi",
  input=sm(2, [key(F0, ORG, K0, "/<0;1>/*"), key(F0, ORG, K1, "/<0;1>/*")], form="multi"),
  host_admits=False, md1_admits=False, format="bip380",
  source=SPEC + " S4.7 conjunct 8 / S7 clause 8 AS AMENDED 2026-08-29: the multi twin; "
                "conjunct 8 binds BOTH --as paths. Conjunct 1's --as-dependent multi arm "
                "runs AFTER conjuncts 2-8, so this row earns the key-identity refusal "
                "under --as descriptor too -- measured. The previous note said conjunct 1 "
                "refuses first, which was the ordering that produced the C1 Critical "
                "(IMPL-S1S3-adversarial-review)",
  covers=["gate"], gate=G(True, "descriptor-refusal", 3, "key-identity"))
