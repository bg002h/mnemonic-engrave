{
  # The one picotool (design/PICOTOOL_PIN.md): picotool 2.3.1 built against
  # pico-sdk 2.3.1, from nixpkgs-unstable locked in flake.lock, with
  # nix/patches/picotool-2.3.1-seal-clear-fix.patch applied (F-701: stock 2.3.1
  # cannot `seal --sign --clear` a UF2). The fork (bg002h/seedhammer) and the
  # Sitting image take it as an input:
  #   inputs.mnemonic-engrave.packages.${system}.picotool
  # HOLD (F-701): the seal fix is not yet proven on hardware. Until bench R4
  # boots a 2.3.1-sealed image, the fork does not move to this picotool.
  #
  #   nix build .#picotool               # -> result/bin/picotool, `version -s` = 2.3.1
  #   nix build .#picotool-unpatched     # stock nixpkgs build: CI's negative control only
  #   SEEDHAMMER_DIR=../seedhammer nix develop .#otp
  #                                      # refugium-otp.sh, R (scripts/pico2-bootkey-rehearsal.sh)
  #                                      # and both e2e suites
  description = "mnemonic-engrave: the pinned picotool and the OTP tooling shell";

  # This URL form fetches through the sandbox's HTTPS proxy (the github: form
  # does not); flake.lock pins the rev.
  inputs.nixpkgs.url = "git+https://github.com/NixOS/nixpkgs?ref=nixos-unstable&shallow=1";

  outputs = { self, nixpkgs }:
    let
      # Linux is verified; darwin is listed but unverified.
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
      # The patch is written against picotool 2.3.1 (tag 2041936). A flake.lock
      # bump that moves nixpkgs' picotool must fail evaluation here, not
      # silently apply the patch to another version (or fail it with fuzz).
      picotoolFor = pkgs:
        assert pkgs.lib.assertMsg (pkgs.picotool.version == "2.3.1")
          "flake.nix: nixpkgs picotool is ${pkgs.picotool.version}, but nix/patches/picotool-2.3.1-seal-clear-fix.patch is for 2.3.1 -- re-derive the patch (design/PICOTOOL_PIN.md)";
        pkgs.picotool.overrideAttrs (old: {
          # builtins.path copies the patch alone, so the patched store path
          # depends on the patch's bytes and nixpkgs, not on the rest of this
          # repo (on any Nix version): PICOTOOL_PIN.md can record it.
          patches = (old.patches or [ ]) ++ [
            (builtins.path {
              path = ./nix/patches/picotool-2.3.1-seal-clear-fix.patch;
              name = "picotool-2.3.1-seal-clear-fix.patch";
            })
          ];
        });
    in
    {
      packages = forAll (pkgs: {
        # nixpkgs already builds it against its pico-sdk 2.3.1 with the mbedtls
        # substitution and installs the udev rule; the override only adds the
        # two-hunk seal fix (`version` stays 2.3.1).
        picotool = picotoolFor pkgs;
        default = picotoolFor pkgs;
        # Stock nixpkgs picotool, for scripts/test/seal-clear-test.sh
        # --expect-fail in CI. Not used by any shell; never sign with it.
        picotool-unpatched = pkgs.picotool;
      });

      devShells = forAll (pkgs: {
        otp = pkgs.mkShell {
          packages = [
            (picotoolFor pkgs)
            pkgs.openssl
            pkgs.jq
            pkgs.python3
            pkgs.xxd
            pkgs.tinygo # R's phases build the blinky (R:131-141)
            pkgs.go
          ];
          shellHook = ''
            if [ "$(picotool version -s)" != "2.3.1" ]; then
              echo "devShells.otp: picotool is not 2.3.1 -- flake.lock moved?" >&2
            fi
            [ -n "''${SEEDHAMMER_DIR:-}" ] || echo "devShells.otp: set SEEDHAMMER_DIR to the seedhammer fork checkout (R needs its cmd/picosign)" >&2
          '';
        };
      });
    };
}
