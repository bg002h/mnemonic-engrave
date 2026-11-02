#!/bin/sh
# fixture stub: prints OK, or FAILED and exits 1 when FAKE_INSTALL_ASSETS=fail
if [ "${FAKE_INSTALL_ASSETS:-}" = fail ]; then
    echo "[install-assets.test] FAILED"
    exit 1
fi
echo "[install-assets.test] OK"
