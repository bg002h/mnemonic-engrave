#!/usr/bin/env bash
# The Self-Review mutation table, re-run against $MUTDIR (default verify-real).
export MUTDIR=${MUTDIR:-/scratch/code/shibboleth/.tmp/s4-plan/verify-real}
M="$(dirname "$0")/f449-stage4-mut.py"
r() { printf '%-4s ' "$1"; shift; $M "$@"; }
r M1 md/compose.go '		case UnspendableLiana:
			ikKind = InternalKeyLianaUnspendable' '		case UnspendableLiana:
			ikKind = InternalKeyNUMS' ./md/ 'Liana|Unspendable'
r M2 md/compose_unspendable.go '	return c.requested == UnspendableLiana && c.internalKey() != InternalKeyLianaUnspendable' '	return false' ./md/ 'TestALianaRequest'
r M3 md/compose_unspendable.go '	return c.requested == UnspendableLiana && c.internalKey() != InternalKeyLianaUnspendable' '	return c.requested == UnspendableLiana' ./md/ 'TestALianaRequest|TestComposeLiana'
r M4 md/compose_unspendable.go '		return ErrUnspendableSortedMultiA' '		return nil' ./md/ 'TestValidateUnspendable'
r M5 md/compose_unspendable.go '	return u.hasMultipath && !u.wildcardHardened && len(u.multipath) == 2 &&' '	return true || u.hasMultipath && !u.wildcardHardened && len(u.multipath) == 2 &&' ./md/ 'TestValidateUnspendable'
r M6 md/compose_unspendable.go '		if b.ik == InternalKeyLianaUnspendable && !isRoot {' '		if false {' ./md/ 'TestValidateUnspendable'
r M7 md/policy_shape.go '		return KeyPathLianaUnspendable, true' '		return KeyPathNUMS, true' ./md/ 'TestKeyPathNames'
r M8 md/md.go '		KeyPath:    rootKeyPath(d.tree),' '' ./md/ 'TestKeyPathNames'
r M9 md/policy_shape.go '	return KeyPathNone, false
}' '	return KeyPathNUMS, true
}' ./md/ 'TestAnUnknownInternalKey'
r M10 md/liana.go '	return lianaUnspendableKey(pks), nil
}' '	_ = pks
	return lianaUnspendableKey(nil), nil
}' ./md/ 'TestLianaUnspendableKeyChunks'
r M11 md/compose.go '			wildcardHardened: false,
		},
		tree: tree,' '			wildcardHardened: true,
		},
		tree: tree,' ./md/ 'TestTheComposerCannotReach'
r A1 gui/policy_address.go '		if liana == nil {
			return nil, errUnderivableInternalKey
		}
		return address.DeriveChild(*liana, index, change)' '		return address.NUMSInternalKey()' ./gui/ 'TestDeviceDerives|TestEveryKeyedVector|TestAnUnderivable'
r A2 gui/policy_address.go '	}
	return nil, errUnderivableInternalKey
}' '	}
	return address.NUMSInternalKey()
}' ./gui/ 'TestAnUnderivable'
r A3 gui/policy_address.go '{Type: bip380.RangeDerivation, Index: 0, End: 1},' '{Type: bip380.RangeDerivation, Index: 1, End: 2},' ./gui/ 'TestDeviceDerives|TestEveryKeyedVector'
r C1 gui/composer_consent.go '	if shape.KeyPath == md.KeyPathSpendable {
		unlocked++' '	if shape.KeyPath == md.KeyPathSpendable || shape.KeyPath == md.KeyPathLianaUnspendable {
		unlocked++' ./gui/ 'TestComposerLianaClassRulings|TestComposerUnspendablePredicate'
r C2 gui/composer_consent.go '	if shape.KeyPath == md.KeyPathNUMS {
		return "NUMS key path"' '	if shape.KeyPath != md.KeyPathSpendable {
		return "NUMS key path"' ./gui/ 'TestComposerLianaClassRulings|TestComposerUnspendablePredicate'
r S1 gui/composer_consent.go '		lines = append(lines, composerCopyLianaKeyPath())' '' ./gui/ 'TestEveryKeyPathPrintSite'
r S2 gui/template_engrave.go '		out = append(out, "Key-path: none (Liana key)")' '' ./gui/ 'TestEveryKeyPathPrintSite'
r S3 gui/md1_inspect.go '		return "Key path: Liana key", true' '		return "Key path: NUMS", true' ./gui/ 'TestEveryKeyPathPrintSite'
r S7 gui/wallet_policy.go '	return tpl.Root == md.ScriptTr && tpl.KeyPath == md.KeyPathNone' '	return false' ./gui/ 'TestAnUnnamedKeyPath'
r P1 gui/composer_unspendable.go '	if p, real := c.InternalKeyPath(); real {
		return false, composerUnspendableDrop{keyPath: p + 1}
	}' '	_ = c' ./gui/ 'TestComposerUnspendablePredicateOnEveryTrPreset'
r P2 gui/composer_unspendable.go 'class != "" {' 'false && class != "" {' ./gui/ 'TestComposerUnspendablePredicateOnEveryTrPreset'
r P3 gui/composer_unspendable.go 'composerDeclaredOrigins(st), md.UnspendableLiana)' 'composerDeclaredOrigins(st), md.UnspendableNums)' ./gui/ 'TestComposerUnspendablePredicateOnEveryTrPreset'
r D1 gui/composer_unspendable.go '	initial := 0
	for i, k := range composerUnspendableKinds {
		if k == st.unspendable {
			initial = i
		}
	}' '	initial := 0' ./gui/ 'TestComposerUnspendableDefaultRow|TestComposerKeyPathChoice'
r D2 gui/composer_unspendable.go '	initial := 0
	for i, k := range composerUnspendableKinds {
		if k == st.unspendable {
			initial = i
		}
	}' '	initial := 1' ./gui/ 'TestComposerUnspendableDefaultRow'
r D3 md/compose_unspendable.go '	UnspendableNums UnspendableKind = iota
	// UnspendableLiana is Liana'"'"'s unspendable xpub over the composed leaf set
	// (SPEC §2, wire kind 1).
	UnspendableLiana' '	UnspendableLiana UnspendableKind = iota
	UnspendableNums' ./gui/ 'TestComposerUnspendableDefaultRow'
r R1 gui/composer_unspendable.go '			st.unspendable = md.UnspendableNums
			showError' '			showError' ./gui/ 'TestComposerUnspendableReset'
r R2 gui/composer_unspendable.go '	fire, drop := composerUnspendableFires(st)' '	fire, drop := composerUnspendableFires(st)
	st.unspendable = md.UnspendableNums' ./gui/ 'TestComposerUnspendableReset'
r R3 gui/composer_unspendable.go '			showError(ctx, th, "Key path", composerCopyLianaKeyDropped(composerUnspendableDropCause(drop)))' '			_ = drop' ./gui/ 'TestComposerUnspendableReset'
r F1 gui/composer_copy.go '	return "Which key path? The two are DIFFERENT WALLETS, with different " +' '	return "Which key path? The two are DIFFERENT WALLETS. Which key path? The two are DIFFERENT WALLETS. Which key path? The two are DIFFERENT WALLETS. Which key path? The two are DIFFERENT WALLETS. Which key path? The two are DIFFERENT WALLETS. Which key path? The two are DIFFERENT WALLETS. " +' ./gui/ 'TestComposerUnspendableScreenFirstPage'
r L1 gui/composer_flow.go '		if !composerUnspendableStep(ctx, th, st) {
			continue
		}
		template, err := composerTemplateChunksFor(st)
		if err != nil {
			composerShowRefusal(ctx, th, "Template", err)
			continue
		}' '		template, err := composerTemplateChunksFor(st)
		if err != nil {
			composerShowRefusal(ctx, th, "Template", err)
			continue
		}
		if !composerUnspendableStep(ctx, th, st) {
			continue
		}' ./gui/ 'TestComposerKeyPathChoiceIsPlacedBeforeTheChunks'
r S4 gui/composer_selfcheck.go '	if (st.unspendable == md.UnspendableLiana) != (shape.KeyPath == md.KeyPathLianaUnspendable) {' '	if false {' ./gui/ 'TestComposerSelfCheckSeesTheKeyPath'
r S5 gui/composer_flow.go '	ck, err := composerCompose(st, declared)' '	ck, err := md.ComposeWith(st.list, declared)' ./gui/ 'TestComposerComposesOnlyThroughOneSite'
r S6 gui/composer_unspendable.go '	if err := c.ValidateUnspendableShape(); err != nil {
		return md.Composed{}, err
	}' '' ./gui/ 'TestComposerComposeRefusesSpecSix'
r X1 cmd/emu/expect_composer.json '"numsTemplateId": "8107216456de60d05e57f7fe268824d8",' '' ./cmd/emu/ 'TestShotsComposerExpect'
r I1 gui/policy_address.go '		if liana == nil {
			return nil, errUnderivableInternalKey
		}
		return address.DeriveChild(*liana, index, change)' '		return nil, errUnderivableInternalKey' ./gui/ 'TestInspectNamesTheLianaKind'
r S8 gui/wallet_policy.go '	if len(keys) > 0 && md1KeyPathUnknown(tpl) {' '	if false && md1KeyPathUnknown(tpl) {' ./gui/ '.'
