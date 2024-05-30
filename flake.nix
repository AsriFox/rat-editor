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
    fenix-pkgs = fenix.packages."${system}";
  in {
    devShells.default = pkgs.mkShell {
      buildInputs = [
        pkgs.helix
        pkgs.pkg-config
        (fenix-pkgs.stable.withComponents [
          "cargo"
          "rustc"
          "rust-std"
          "rust-docs"
          "rust-src"
          "rustfmt"
          "clippy"
        ])
      ];
      shellHook = ''
        PATH=$PATH:${fenix-pkgs.rust-analyzer}/bin:${fenix-pkgs.stable.rustfmt}/bin
        exec fish
      '';
    };
  });
}
