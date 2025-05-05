{
  description = "Rust project using Cargo with Nix flake";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; 
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs {
            inherit system;
            };
      rustPackage = import ./default.nix {inherit pkgs;};
    in {
      packages.default = rustPackage;
    });
}
