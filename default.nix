{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "njq";
  version = "0.1";
  cargoHash = "sha256-TI6m0sjg+2VHXNmbJIn0pRfOUKE4nms9DuH9/XZ3Wqw=";
  src = pkgs.lib.cleanSource ./.;
}