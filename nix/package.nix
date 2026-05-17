{
  lib,
  rustPlatform,
  pkg-config,
  makeWrapper,
  libxkbcommon,
  wayland,
  self,
}:
rustPlatform.buildRustPackage {
  pname = "which-key-wayland";
  version =
    let
      cargoToml = lib.importTOML ../which-key-wayland/Cargo.toml;
    in
    cargoToml.package.version + (if self ? shortRev then "+${self.shortRev}" else "");
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = [
    libxkbcommon
    wayland
  ];

  postFixup = ''
    wrapProgram $out/bin/which-key-wayland \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [
          libxkbcommon
          wayland
        ]
      }
  '';
}
