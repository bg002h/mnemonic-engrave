package gui

// F-674 item 1 measurement generator (scratch, not committed to the fork).
// Builds Liana-key tr wallets THROUGH THE COMPOSER'S OWN CODE -- presets from
// composerPresets, seats via composerSeedDerive (the per-seed account rule),
// lowering via composerArtifactsFor (composerCompose + Bind) -- and prints the
// keyed md1 chunks, one wallet per line: name<TAB>chunk chunk ...
// Seeds are named by index into f674Seeds. Env F674_SEATINGS overrides the
// default seating list: "name:preset:i,i,i,i;..."

import (
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"strings"
	"testing"

	"github.com/btcsuite/btcd/chaincfg/v2"
	"seedhammer.com/bip39"
	"seedhammer.com/md"
)

var f674Seeds = []string{
	// 0: master B, the composer payload's seed (b8688df1) -- the demo seed.
	"legal winner thank year wave sausage worth useful legal winner thank yellow",
	// 1: master A (73c5da0a).
	"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
	// 2: zoo.
	"zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
	// 3: BIP-39 vector.
	"letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
	// 4..7: more BIP-39 / SLIP-14 public vectors.
	"all all all all all all all all all all all all",
	"ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
	"scheme spot photo card baby mountain device kick cradle pact join borrow",
	"cat swing flag economy stadium alone churn speed unique patch report train",
}

func f674Preset(name string) md.PathList {
	for _, p := range composerPresets(md.ComposeTr) {
		if p.name == name {
			return p.list
		}
	}
	panic("no preset " + name)
}

func TestF674Measure(t *testing.T) {
	spec := os.Getenv("F674_SEATINGS")
	if spec == "" {
		t.Skip("F674_SEATINGS unset")
	}
	out, err := os.Create(os.Getenv("F674_OUT"))
	if err != nil {
		t.Fatal(err)
	}
	defer out.Close()
	for _, w := range strings.Split(spec, ";") {
		parts := strings.Split(w, ":")
		name, preset := parts[0], parts[1]
		var seats []int
		for _, s := range strings.Split(parts[2], ",") {
			n, _ := strconv.Atoi(s)
			seats = append(seats, n)
		}
		st := &composerState{reg: &seedRegistry{}, list: f674Preset(preset), unspendable: md.UnspendableLiana}
		srcOf := map[int]int{}
		for _, si := range seats {
			if _, ok := srcOf[si]; ok {
				continue
			}
			m, err := bip39.ParseMnemonic(f674Seeds[si])
			if err != nil {
				t.Fatal(err)
			}
			id, err := st.reg.add("seed", m, "", &chaincfg.MainNetParams)
			if err != nil {
				t.Fatal(err)
			}
			seed, _ := st.reg.at(id)
			var fp [4]byte
			binary.BigEndian.PutUint32(fp[:], seed.MasterFP)
			srcOf[si] = len(st.sources)
			st.sources = append(st.sources, composerSource{
				kind: composerSourceSeed, label: "seed", fingerprint: fp, fpPresent: true, seedID: id,
			})
		}
		st.assigned = make([]composerAssignment, len(seats))
		for i, si := range seats {
			a, err := composerSeedDerive(st, uint8(i), srcOf[si])
			if err != nil {
				t.Fatal(err)
			}
			st.assigned[i] = a
		}
		if len(composerSharedSeedInPath(st)) == 0 {
			t.Fatalf("%s: premise: composerSharedSeedInPath must fire (this is a same-seed seating)", name)
		}
		if !composerLianaRefusesSeating(st) {
			t.Fatalf("%s: premise: composerLianaRefusesSeating must be true", name)
		}
		_, keyed, err := composerArtifactsFor(st)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		var pks []string
		for _, a := range st.assigned {
			_, pk, _, err := decodeXpubBytes(a.xpub)
			if err != nil {
				t.Fatal(err)
			}
			pks = append(pks, hex.EncodeToString(pk[:]))
		}
		fmt.Fprintf(out, "%s\t%s\t%s\t%s\n", name, preset, strings.Join(pks, ","), strings.Join(keyed, " "))
	}
}
