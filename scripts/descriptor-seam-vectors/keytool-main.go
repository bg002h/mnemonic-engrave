// keytool: deterministic extended-key material for the descriptor seam corpus.
//
//	reversion <xpub> <xpub|tpub|zpub|Zpub|ypub|Ypub|upub|vpub|Upub|Vpub>
//	children  <xpub> <count>      -- unhardened children 0..count-1, re-serialised at depth 1
package main

import (
	"encoding/hex"
	"fmt"
	"os"
	"strconv"

	"github.com/btcsuite/btcd/btcutil/base58"
	"github.com/btcsuite/btcd/btcutil/hdkeychain"
)

var versions = map[string]string{
	"xpub": "0488b21e",
	"tpub": "043587cf",
	"zpub": "04b24746",
	"Zpub": "02aa7ed3",
	"ypub": "049d7cb2",
	"Ypub": "0295b43f",
	"upub": "044a5262",
	"vpub": "045f1cf6",
	"Upub": "024289ef",
	"Vpub": "02575483",
}

func main() {
	switch os.Args[1] {
	case "reversion":
		raw := base58.Decode(os.Args[2])
		if len(raw) != 82 {
			panic(fmt.Sprintf("not an 82-byte base58check payload: %d", len(raw)))
		}
		v, ok := versions[os.Args[3]]
		if !ok {
			panic("unknown version " + os.Args[3])
		}
		vb, _ := hex.DecodeString(v)
		copy(raw[:4], vb)
		fmt.Println(base58.CheckEncode(raw[1:78], raw[0]))
	case "children":
		k, err := hdkeychain.NewKeyFromString(os.Args[2])
		if err != nil {
			panic(err)
		}
		n, _ := strconv.Atoi(os.Args[3])
		for i := 0; i < n; i++ {
			c, err := k.Derive(uint32(i))
			if err != nil {
				panic(err)
			}
			fmt.Println(c.String())
		}
	default:
		panic("usage: keytool reversion|children")
	}
}
