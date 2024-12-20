# Nix

The Nix package derivation is currently defined for all [default systems](https://github.com/nix-systems/default), and it can be integrated into NixOS and Home-Manager configurations as below.

## Installation

Here's how to reference the package derivation (and explicitly pin it) in your `flake.nix`:

``` nix
{
  inputs.gauntlet.url = github:project-gauntlet/gauntlet/<gauntlet_version_repository_tag>;
  inputs.gauntlet.inputs.nixpkgs.follows = "nixpkgs";
}
```

The package can then be referenced directly with `gauntlet.packages.${system}.default` or integrated as an overlay with `gauntlet.overlays.default`.

## Configuration

Under `programs.gauntlet`, the options provide the following:

1. `enable`: adds executable to system path
2. `service.enable`: runs daemon with systemd (MacOS launchd not yet supported)

The examples below assume flake inputs are passed to `nixpkgs.lib.nixosSystem` and `home-manager.lib.mkHomeManagerConfiguration` respectively as `inputs` parameter.

### NixOS

``` nix
{inputs, ...}: {
  imports = [inputs.gauntlet.nixosModules.default];
  programs.gauntlet = {
    enable = true;
    service.enable = true;
  };
}
```

### Home-Manager

Once `config.toml` is [supported](../README.md#application-config), Home-Manager can populate its contents with `programs.gauntlet.config`.

``` nix
{inputs, ...}: {
  imports = [inputs.gauntlet.homeManagerModules.default];
  programs.gauntlet = {
    enable = true;
    service.enable = true;
    config = {};
  };
}
```

## Development

When updating dependencies or bumping the project version, please follow these steps to adjust the relevant values at the top of `./nix/overlay.nix`:

1. If there is a new package release, set `version` to that upcoming version tag.
2. If `package-lock.json` has changed, set `npmDepsHash` to `""` and rebuild with `nix build`, copying the actual value back into `npmDepsHash`. This is necessary for `fetchNpmDeps` because `importNpmLock` doesn't work with `git://` dependencies like for `@project-gauntlet/tools`.
3. If `Cargo.lock` has changed, run `nix run .#fetch-rusty-v8-hashes` and replace `RUSTY_V8_ARCHIVE` as instructed if different. Because building `librusty_v8` takes forever, we follow nixpkgs precedent and fetch binaries in a fixed-output-derivation.

When making any changes to nix code, please format with `nix fmt` when done.

When running the project in development, `.#devShells.default` will provide access to all repository tooling. You can access this by running `nix develop`, or `direnv allow` if you have `direnv` + `nix-direnv`.
