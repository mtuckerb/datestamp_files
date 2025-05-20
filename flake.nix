{
  inputs = {
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
      
      packages = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "datestamp_files";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          # If your package has any runtime dependencies, list them here
          propagatedBuildInputs = with pkgs; [
            # Add any runtime dependencies here
          ];
        };
      });

      devShells = forEachSupportedSystem ({ pkgs }: {
          default = pkgs.mkShell {
          # Pinned packages available in the environment
          packages = with pkgs; [
            rust-bin.stable."1.82.0".default
            cargo-bloat
            cargo-edit
            cargo-outdated
            cargo-udeps
            cargo-watch
            rust-analyzer
            rustup
            rustc
            curl
            openssl
            pkg-config
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
