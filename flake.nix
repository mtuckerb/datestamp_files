{
  inputs = {
    #flake-schemas.url = "github:DeterminateSystems/flake-schemas";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self,  nixpkgs, rust-overlay }: #flake-schemas,
    let
      # Nixpkgs overlays
      overlays = [(import rust-overlay)];      # Helpers for producing system-specific outputs
      supportedSystems = [ "aarch64-darwin" "x86_64-linux"];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs { 
          inherit overlays system; 
        };
        
      });
    in {
      
      devShells = forEachSupportedSystem ({ pkgs }: {
          default = pkgs.mkShell {
          # Pinned packages available in the environment
          packages = with pkgs; [
            rust-bin.stable."1.81.0".default
            cargo-bloat
            cargo-edit
            cargo-outdated
            cargo-udeps
            cargo-watch
            rust-analyzer
            rustup
            rustc
            curl
            libiconv
            cmake
            gcc
          ];
          

          # Environment variables
          env = {
            # RUST_BACKTRACE = "1";
            # CC= "/usr/bin/clang";
            # CXX="/usr/bin/clang++";
          };
        };
      });
    };
}
