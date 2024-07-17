{
  description = "A nix development shell and build environment for ibc-types";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          # Set up for Rust builds, use the stable Rust toolchain
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustToolchain = pkgs.rust-bin.stable."1.78.0".default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          # Important environment variables so that the build can find the necessary libraries
          PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
        in with pkgs; with pkgs.lib;
        let
          ibc-types = (craneLib.buildPackage {
            pname = "ibc-types";
            version = "0.14.1";
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ openssl ];
            inherit src system PKG_CONFIG_PATH;
            cargoVendorDir = null;
            cargoExtraArgs = "-p ibc-types";
            meta = {
              description = "Common data structures for Inter-Blockchain Communication (IBC) messages";
              homepage = "https://github.com/penumbra-zone/ibc-types";
              license = [ licenses.mit licenses.asl20 ];
            };
          }).overrideAttrs (_: { doCheck = false; }); # Disable tests to improve build times
        in rec {
          packages = { inherit ibc-types; };
          devShells.default = craneLib.devShell {
            inputsFrom = [ ibc-types ];
            packages = [ cargo-watch cargo-nextest ];
            shellHook = ''
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc} # Required for rust-analyzer
            '';
          };
        }
      );
}
