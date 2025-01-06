{inputs, ...}: {
  imports = [
    ./modules/home.nix
    ./modules/nixos.nix
    ./overlay.nix
  ];
  systems = import inputs.systems;
  perSystem = {
    pkgs,
    lib,
    system,
    ...
  }: let
    inherit (lib) makeBinPath makeLibraryPath optionals optionalString;
    inherit (pkgs) alejandra cargo cmake deno gauntlet gtk3 libxkbcommon libGL mkShell nodejs pkg-config protobuf stdenv xorg wayland;
    inherit (stdenv.hostPlatform) isLinux;
  in {
    _module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      overlays = [inputs.self.overlays.default];
    };
    devShells.default = mkShell {
      packages = [cargo cmake deno nodejs protobuf] ++ optionals isLinux [libxkbcommon pkg-config];
      shellHook = optionalString isLinux ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${makeLibraryPath [libGL xorg.libX11 wayland]}"
        export PATH="$PATH:${makeBinPath [gtk3]}"
      '';
    };
    formatter = alejandra;
    packages.default = gauntlet;
    packages.fetch-rusty-v8-hashes = gauntlet.fetchRustyV8Hashes;
  };
}
