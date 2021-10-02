{
  description = "A ddns client for cloudflare";

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

  outputs = { nixpkgs, flake-utils, rust-overlay, nocargo, ... }@inputs: {
    overlay = final: prev: with final; {
      cloudflare-ddns = nocargo.buildRustCrateFromSrcAndLock {
        src = ./.;
        inherit rustc;
      };
    };
  } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlay nocargo.overlay ];
        };

        rustc = pkgs.rust-bin.nightly.latest.minimal;
      in
      rec {
        defaultPackage = packages."cloudflare-ddns";
        defaultApp = defaultPackage.bin;

        packages."cloudflare-ddns" = pkgs.nocargo.buildRustCrateFromSrcAndLock {
          src = ./.;
          inherit rustc;
        };
      });
}
