{ pkgs ? import <nixpkgs> { } }:

let
  # Define the Rust library build
  rustLib = pkgs.rustPlatform.buildRustPackage rec {
    pname = "libsnix";
    version = "0.1.0";

    src = ./rust_lib;

    cargoLock = {
      lockFile = ./rust_lib/Cargo.lock;
       outputHashes = {
         "snix-eval-0.1.0" = "sha256-ciBpaYvwcjJ1YN2dc6xyQVebsERIKZ+p2q+xs6sO+CE=";
       };
    };

    nativeBuildInputs = [ pkgs.cargo pkgs.rustc ];
    buildInputs = [ ];

    # Build the Rust library as a cdylib
    buildPhase = ''
      cargo build --release
    '';

    installPhase = ''
      mkdir -p $out/lib
      cp target/release/libsnix.so" $out/lib/
      cp target/release/libsnix.so" $out/lib/
    '';
  };

in pkgs.stdenv.mkDerivation rec {
  pname = "snix-project";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [ pkgs.gcc pkgs.pkg-config ];
  buildInputs = [ rustLib ];

  buildPhase = ''
    # Ensure Rust library is available
    export LIBRARY_PATH=${rustLib}/lib:$LIBRARY_PATH

    # Compile C++ code manually with g++ from stdenv
    ${pkgs.stdenv.cc}/bin/g++ cpp/main.cpp -L${rustLib}/lib -lsnix -o njq"
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp njq $out/bin/
  
  '';

  meta = with pkgs.lib; {
    description = "A project combining C++ and Rust code with libsnix";
    license = licenses.mit;
    platforms = platforms.all;
  };
}