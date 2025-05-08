{ pkgs ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/e1c2e701296453fe2b46b2824db0a92cb310b311.tar.gz";
  }) {}
}:pkgs.rustPlatform.buildRustPackage rec {
  pname = "njq";
  version = "0.0.2";
  cargoHash = "sha256-Z4whDevtbS1cmkrgdASmTTJbGANPaQWJODK9djgp2QY=";
  src = pkgs.lib.cleanSource ./.;
}