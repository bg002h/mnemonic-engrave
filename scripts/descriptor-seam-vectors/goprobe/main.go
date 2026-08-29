package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"

	"github.com/btcsuite/btcd/btcutil/hdkeychain"
	"seedhammer.com/address"
	"seedhammer.com/bip380"
	"seedhammer.com/md"
	"seedhammer.com/nonstandard"
	"seedhammer.com/sysw"
)

type In struct {
	Name  string `json:"name"`
	Input string `json:"input"`
	// WalletID: when true, also compute the Go-side WalletPolicyId over the
	// (a')-materialised policy (EncodeMultisig hard-codes <0;1>/*).
	WalletID bool `json:"wallet_id"`
	Probe    bool `json:"probe"` // if true, do NOT call OutputDescriptor (panic:parse rows)
}

type Out struct {
	Name         string `json:"name"`
	DeviceAdmits *bool  `json:"device_admits,omitempty"`
	ParseErr     string `json:"parse_err,omitempty"`
	ParsePanic   string `json:"parse_panic,omitempty"`
	Canonical    string `json:"canonical,omitempty"`
	EncodePanic  string `json:"encode_panic,omitempty"`
	FixedPoint   *bool  `json:"fixed_point,omitempty"`
	ReparseErr   string `json:"reparse_err,omitempty"`
	Script       string `json:"script,omitempty"`
	MultiType    string `json:"multi_type,omitempty"`
	Threshold    int    `json:"threshold"`
	NKeys        int    `json:"nkeys"`
	Title        string `json:"title,omitempty"`
	Supported    *bool  `json:"supported,omitempty"`
	Addr0        string `json:"address_0,omitempty"`
	Addr0Err     string `json:"address_0_err,omitempty"`
	Addr1        string `json:"address_1,omitempty"`
	Addr1Err     string `json:"address_1_err,omitempty"`
	SyswClass    string `json:"sysw_class,omitempty"`
	WalletID     string `json:"wallet_id,omitempty"`
	WalletIDErr  string `json:"wallet_id_err,omitempty"`
}

func b(v bool) *bool { return &v }

func mtype(t bip380.MultisigType) string {
	if t == bip380.SortedMulti {
		return "SortedMulti"
	}
	return "Singlesig"
}

func classOf(c sysw.Class) string {
	names := []string{"Unknown", "Mnemonic", "Codex32Secret", "Passphrase", "FreeText", "Descriptor", "MDMK", "Address", "Mt", "Tx"}
	if int(c) < len(names) {
		return names[int(c)]
	}
	return fmt.Sprintf("Class(%d)", int(c))
}

func main() {
	var ins []In
	if err := json.NewDecoder(os.Stdin).Decode(&ins); err != nil {
		panic(err)
	}
	outs := make([]Out, 0, len(ins))
	for _, in := range ins {
		o := Out{Name: in.Name}
		o.SyswClass = classOf(sysw.Classify(in.Input))
		if in.Probe {
			outs = append(outs, o)
			continue
		}
		var desc *bip380.Descriptor
		func() {
			defer func() {
				if r := recover(); r != nil {
					o.ParsePanic = fmt.Sprint(r)
				}
			}()
			d, err := nonstandard.OutputDescriptor([]byte(in.Input))
			if err != nil {
				o.ParseErr = err.Error()
				o.DeviceAdmits = b(false)
				return
			}
			desc = d
			o.DeviceAdmits = b(true)
		}()
		if desc == nil {
			outs = append(outs, o)
			continue
		}
		o.Script = desc.Script.String()
		o.MultiType = mtype(desc.Type)
		o.Threshold = desc.Threshold
		o.NKeys = len(desc.Keys)
		o.Title = desc.Title
		func() {
			defer func() {
				if r := recover(); r != nil {
					o.EncodePanic = fmt.Sprint(r)
				}
			}()
			o.Canonical = desc.Encode()
		}()
		if o.Canonical != "" {
			rd, err := bip380.Parse(o.Canonical)
			if err != nil {
				o.ReparseErr = err.Error()
				o.FixedPoint = b(false)
			} else {
				func() {
					defer func() {
						if r := recover(); r != nil {
							o.ReparseErr = "encode panic: " + fmt.Sprint(r)
						}
					}()
					o.FixedPoint = b(rd.Encode() == o.Canonical)
				}()
			}
		}
		func() {
			defer func() {
				if r := recover(); r != nil {
					o.Addr0Err = "panic: " + fmt.Sprint(r)
				}
			}()
			o.Supported = b(address.Supported(desc))
			a0, err := address.Receive(desc, 0)
			if err != nil {
				o.Addr0Err = err.Error()
			} else {
				o.Addr0 = a0
			}
			a1, err := address.Receive(desc, 1)
			if err != nil {
				o.Addr1Err = err.Error()
			} else {
				o.Addr1 = a1
			}
		}()
		if in.WalletID {
			id, err := goWalletID(desc)
			if err != nil {
				o.WalletIDErr = err.Error()
			} else {
				o.WalletID = id
			}
		}
		outs = append(outs, o)
	}
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err := enc.Encode(outs); err != nil {
		panic(err)
	}
}

// goWalletID computes the fork's own WalletPolicyId over the (a')-materialised
// policy: EncodeMultisig hard-codes the device default <0;1>/* use-site, which
// is exactly the plan's scoping for the wallet_id column.
func goWalletID(d *bip380.Descriptor) (string, error) {
	var script md.MultisigScript
	switch d.Script {
	case bip380.P2WSH:
		script = md.MultisigWsh
	case bip380.P2SH_P2WSH:
		script = md.MultisigShWsh
	case bip380.P2SH:
		script = md.MultisigSh
	default:
		return "", fmt.Errorf("wallet_id: script %v has no EncodeMultisig arm", d.Script)
	}
	cos := make([]md.MultisigCosigner, len(d.Keys))
	for i, k := range d.Keys {
		var cc [32]byte
		var pk [33]byte
		if len(k.ChainCode) != 32 {
			return "", fmt.Errorf("key %d: chain code is %d bytes", i, len(k.ChainCode))
		}
		if len(k.KeyData) != 33 {
			return "", fmt.Errorf("key %d: key data is %d bytes", i, len(k.KeyData))
		}
		copy(cc[:], k.ChainCode)
		copy(pk[:], k.KeyData)
		var fp [4]byte
		fp[0] = byte(k.MasterFingerprint >> 24)
		fp[1] = byte(k.MasterFingerprint >> 16)
		fp[2] = byte(k.MasterFingerprint >> 8)
		fp[3] = byte(k.MasterFingerprint)
		origin := make([]md.PathComponent, 0, len(k.DerivationPath))
		for _, p := range k.DerivationPath {
			if p >= hdkeychain.HardenedKeyStart {
				origin = append(origin, md.PathComponent{Hardened: true, Value: p - hdkeychain.HardenedKeyStart})
			} else {
				origin = append(origin, md.PathComponent{Hardened: false, Value: p})
			}
		}
		cos[i] = md.MultisigCosigner{
			ChainCode:        cc,
			CompressedPubkey: pk,
			Fingerprint:      fp,
			FpPresent:        k.MasterFingerprint != 0,
			Origin:           origin,
		}
	}
	strs, _, _, err := md.EncodeMultisig(md.EncodeMultisigRequest{
		Cosigners:  cos,
		K:          uint8(d.Threshold),
		Script:     script,
		OriginMode: md.OriginDivergent,
	})
	if err != nil {
		return "", err
	}
	id, err := md.WalletPolicyIdChunks(strs)
	if err != nil {
		return "", err
	}
	return hex.EncodeToString(id[:]), nil
}
