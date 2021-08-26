{
  description = "Rust crate cf-ddns";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    nocargo = {
      url = "github:oxalica/nocargo";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.registry-crates-io.follows = "registry-crates-io";
    };

    registry-crates-io = { url = "github:rust-lang/crates.io-index"; flake = false; };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, nocargo, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlay nocargo.overlay ];
        };

        rustc = pkgs.rust-bin.nightly.latest.minimal;

      in
      rec {
        defaultPackage = packages."cf-ddns";
        defaultApp = defaultPackage.bin;

        packages."cf-ddns" = pkgs.nocargo.buildRustCrateFromSrcAndLock {
          src = ./.;
          inherit rustc;
        };

        devShell = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
          ];
        };
      });
}
