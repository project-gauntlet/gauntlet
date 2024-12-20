flake: {
  flake.homeManagerModules.default = {
    config,
    lib,
    pkgs,
    ...
  }: let
    inherit (lib) elem getExe mkIf mkMerge mkOption platforms toList types;
    inherit (pkgs.stdenv.hostPlatform) isLinux isDarwin;
    cfg = config.programs.gauntlet;
    toml = pkgs.formats.toml {};
  in {
    imports = [(import ./common.nix flake)];
    options.programs.gauntlet.config = mkOption {
      inherit (toml) type;
      default = {};
      description = "Application configuration in config.toml";
    };
    config = mkIf cfg.enable {
      home.packages = [cfg.package];
      launchd.agents = mkIf (cfg.service.enable && isDarwin) {
        gauntlet.enable = true;
        gauntlet.config = {
          RunAtLoad = true;
          KeepAlive.Crashed = true;
          ProgramArguments = [(getExe cfg.package) "--minimized"];
        };
      };
      xdg.configFile = mkMerge [
        (mkIf (cfg.service.enable && isLinux) {"systemd/user/gauntlet.service".source = "${cfg.package}/lib/systemd/user/gauntlet.service";})
        (mkIf (cfg.config != {}) {"gauntlet/config.toml".source = toml.generate "gauntlet.config.toml" config.programs.gauntlet.config;})
      ];
    };
  };
}
