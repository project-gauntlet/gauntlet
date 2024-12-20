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
    inherit (pkgs) alejandra cargo cmake deno gauntlet gtk3 libxkbcommon libGL mkShell nodejs protobuf stdenv xorg wayland;
  in {
    _module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      overlays = [inputs.self.overlays.default];
    };
    devShells.default = mkShell {
      packages = [cargo cmake deno nodejs protobuf];
      shellHook = lib.optionalString stdenv.hostPlatform.isLinux ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath [libxkbcommon libGL xorg.libX11 wayland]}"
        export PATH="$PATH:${lib.makeBinPath [gtk3]}"
      '';
    };
    formatter = alejandra;
    packages.default = gauntlet;
    packages.fetch-rusty-v8-hashes = gauntlet.fetchRustyV8Hashes;
  };
}
