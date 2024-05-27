{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nvim.url = "flake:nixvim";
  };

  outputs = { self, nixpkgs, fenix, flake-utils, nvim, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    overlays = [ fenix.overlays.default ];
    pkgs = import nixpkgs { inherit system overlays; };
    fenix-pkgs = fenix.packages."${system}";
  in {
    devShells.default = pkgs.mkShell {
      buildInputs = [
        pkgs.pkg-config
        fenix-pkgs.stable.defaultToolchain
        (nvim.packages.${system}.default.nixvimExtend {
          plugins.lsp.servers.rust-analyzer = {
            enable = true;
            package = fenix-pkgs.rust-analyzer;
            installCargo = nixpkgs.lib.mkForce false;
            installRustc = nixpkgs.lib.mkForce false;
          };
        })
      ];
    };
  });
}
