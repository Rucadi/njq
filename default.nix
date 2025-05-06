{ pkgs ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/e1c2e701296453fe2b46b2824db0a92cb310b311.tar.gz";
  }) {}
}:pkgs.rustPlatform.buildRustPackage rec {
  pname = "njq";
  version = "0.1";
  cargoHash = "sha256-Hhard2mukdWpYGCO0Sr3MmFiNL/bga2PskWp87Cv0bE=";
  src = pkgs.lib.cleanSource ./.;
}