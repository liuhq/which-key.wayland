{
  description = "The which-key.wayland Project uses Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      fenix,
      ...
    }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          self.overlays.default
        ];
      };
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          with fenix.packages.${system};
          combine (
            with stable;
            [
              rustc
              cargo
              clippy
              rustfmt
              rust-src
            ]
          );
      };

      packages.${system} = rec {
        default = which-key-wayland;
        which-key-wayland = pkgs.callPackage ./nix/package.nix { inherit self; };
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustToolchain
          openssl
          pkg-config
          cargo-deny
          rust-analyzer
          lldb

          libxkbcommon

          rumdl # Markdown linter and formatter
        ];

        env = {
          # Required by rust-analyzer
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [ libxkbcommon ]);
        };
      };
    };
}
