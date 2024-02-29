{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    # Build dependency
    fontconfig
  ];

  shellHook =
    with pkgs; let
      libs = [
        libGL
        libxkbcommon
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
      ];
    in
    ''
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
        lib.makeLibraryPath libs
      }"
    '';
}
