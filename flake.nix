{
  description = "Cross-platform foreign window handling library built in Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    # TODO: all systems
    system = "aarch64-darwin";
    pkgs = import nixpkgs {
      inherit system;
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs;
        [
          # TODO
        ]
        ++ (with darwin.apple_sdk.frameworks; [
          CoreFoundation
          ApplicationServices
          AppKit
        ]);
    };
  };
}
