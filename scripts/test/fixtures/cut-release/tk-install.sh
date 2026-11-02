#!/bin/sh
# fixture: the shape of mnemonic-toolkit's scripts/install.sh that cut-release.sh edits
component_info() {
    case "$1" in
        mnemonic)
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.105.1|mnemonic|"
            ;;
    esac
}

first_signed() {
    case "$1" in
        mnemonic)     echo "" ;;
    esac
}
