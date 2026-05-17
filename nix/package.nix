{
  lib,
  rustPlatform,
  pkg-config,
  makeWrapper,
  libxkbcommon,
  wayland,
}:
rustPlatform.buildRustPackage {
  pname = "which-key-wayland";
  version = "0.1.0";
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config makeWrapper ];
  buildInputs = [ libxkbcommon wayland ];

  postFixup = ''
    wrapProgram $out/bin/which-key-wayland \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [ libxkbcommon wayland ]
      }
  '';
}
