{self, ...}: {
  lib,
  pkgs,
  ...
}: {
  options.programs.gauntlet = {
    enable = lib.mkEnableOption "Gauntlet application launcher";
    package = lib.mkPackageOption pkgs "gauntlet" {};
    service.enable = lib.mkEnableOption "running Gauntlet as a service";
  };
  config.nixpkgs.overlays = [self.overlays.default];
}
