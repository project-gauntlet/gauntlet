flake: {
  flake.nixosModules.default = {
    config,
    lib,
    ...
  }: let
    cfg = config.programs.gauntlet;
  in {
    imports = [(import ./common.nix flake)];
    config = lib.mkIf cfg.enable {
      environment.systemPackages = [cfg.package];
      systemd.packages = lib.mkIf cfg.service.enable [cfg.package];
    };
  };
}
