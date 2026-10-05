{
  # The one picotool (design/PICOTOOL_PIN.md): picotool 2.3.1 built against
  # pico-sdk 2.3.1, from nixpkgs-unstable locked in flake.lock. The fork
  # (bg002h/seedhammer) and the Sitting image take it as an input:
  #   inputs.mnemonic-engrave.packages.${system}.picotool
  #
  #   nix build .#picotool               # -> result/bin/picotool, `version -s` = 2.3.1
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
    in
    {
      packages = forAll (pkgs: {
        # nixpkgs already builds it against its pico-sdk 2.3.1 with the mbedtls
        # substitution and installs the udev rule: no overlay.
        picotool = pkgs.picotool;
        default = pkgs.picotool;
      });

      devShells = forAll (pkgs: {
        otp = pkgs.mkShell {
          packages = [
            pkgs.picotool
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
