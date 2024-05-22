{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    overlays = [ fenix.overlays.default ];
    pkgs = import nixpkgs { inherit system overlays; };
    fenix-toolchain = fenix.packages."${system}".stable;
  in {
    devShells.default = pkgs.mkShell {
      buildInputs = [
        pkgs.pkg-config
        (fenix-toolchain.withComponents [ "cargo" "clippy" "rust-src" "rustc" "rustfmt" ])
      ];
    };
  });
}
