{
  description = "A flake for cross-compiling for RISC-V";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust_toolchain_none = fenix.packages.${system}.fromToolchainFile {
          file = ./timer_interrupt_overhead/rust-toolchain.toml;
          sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
        };
        rust_toolchain_musl = fenix.packages.${system}.fromToolchainFile {
          file = ./zbs/rust-toolchain.toml;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
        };
        riscv64MuslPkgs =
          let
            crossPkgs = import nixpkgs {
              inherit system;
              crossSystem.config = "riscv64-unknown-linux-musl";
            };
          in
          crossPkgs.pkgsCross.riscv64;

      in
      {
        # Defines a development shell named 'default'
        formatter = pkgs.nixpkgs-fmt;
        devShells.default = pkgs.mkShell {
          name = "rust environment";
          nativeBuildInputs = with pkgs; [
            # rust
            rust_toolchain_none
            rust_toolchain_musl
            cargo-make
            tokei

            # musl toolchain
            riscv64MuslPkgs.pkgsStatic.stdenv.cc

            glib

            # Standard build tools
            gnumake
            cmake
            pkg-config

            openocd
          ];
        };
      }
    );
}
