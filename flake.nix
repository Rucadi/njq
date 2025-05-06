{
  description = "Rust project using Cargo with Nix flake";
  inputs = {
    nixpkgs.url =  "github:NixOS/nixpkgs/e1c2e701296453fe2b46b2824db0a92cb310b311"; 
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
