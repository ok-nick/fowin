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
      # TODO: no longer needed with latest nix-darwin update
      # nativeBuildInputs = with pkgs;
      #   []
      #   ++ (with darwin.apple_sdk.frameworks; [
      #     libiconv
      #     CoreFoundation
      #     ApplicationServices
      #     AppKit
      #   ]);
    };
  };
}
