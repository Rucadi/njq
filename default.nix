{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "njq";
  version = "0.1";
  cargoHash = "sha256-CQnNvImlaUiSEZYOqv6wjF7dGruE7/4IWlzE5nHf5u8=";
  src = pkgs.lib.cleanSource ./.;
}