{
  description = "Rust-Flake by blckr";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShellNoCC {
            buildInputs = with pkgs; [
              rustc
              cargo
              gcc
              rust-analyzer
              rustfmt
              clippy
              gdb
              openssl
              openssl.dev
            ];
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
          };
        }
      );
    };
}
