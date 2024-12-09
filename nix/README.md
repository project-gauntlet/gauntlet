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

The package lives can be referenced directly with `gauntlet.packages.${system}.default` or integrated with the overlay `gauntlet.overlays.default`.

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

Because this project uses a submodule, the derivation is a little [unconventional](overlay.nix). As you can see, there are also some build practices in use in this repository that create friction with the nix sandbox, so even more patching is required.

There are a few hashes required for this build:

1. `env.RUSTY_V8_ARCHIVE`: `librusty_v8` builds take forever, so best practice is to fetch binaries. If it is ever updated, hashes must be provided for all supported systems by running `nix run .#fetch-rusty-v8-hashes`
2. `tools`: this repo is a submodule
3. `deno-types`: these type stubs are dynamic in the non-nix build script, so we need to pin them and override the script

Please format nix code with `nix fmt` and update the version when a new release is made.

`nix develop` provides the common repository tooling for building and running the project, and there is integration with `nix-direnv`.
