{
  description = "caja";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # We want to use packages from the binary cache
    flake-utils.url = "github:numtide/flake-utils";
    gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };
  };

  outputs = inputs@{ self, nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachSystem [ "x86_64-linux" ] (system: let
    pkgs = nixpkgs.legacyPackages.${system};
    gitignoreSrc = pkgs.callPackage inputs.gitignore { };
  in rec {
    packages.caja = pkgs.callPackage ./default.nix { inherit gitignoreSrc; };

    legacyPackages = packages;

    defaultPackage = packages.caja;

    devShell = pkgs.mkShell {
      CARGO_INSTALL_ROOT = "${toString ./.}/.cargo";

      buildInputs = with pkgs; [ cargo rustc rust-analyzer faust ];
    };
  });
}
